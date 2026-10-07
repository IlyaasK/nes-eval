//! Reference cache: run the reference once per (reference, ROM, replay)
//! and reuse its per-frame output for every candidate grade.
//!
//! The reference's output for a (rom, replay) pair is deterministic, so
//! there's no reason to re-run it on every grade. `grader --precompute`
//! captures it into `corpus/reference-cache/<testcase>.refcache`.
//!
//! # File format (`.refcache`, version 1)
//!
//! A zstd stream (level 3) wrapping one bincode-1 record (default
//! options: little-endian, fixed-width integers, `u64` length prefix on
//! every `String` and `Vec`, fields in declaration order):
//!
//! | field                    | type          | meaning |
//! |--------------------------|---------------|---------|
//! | `magic`                  | `[u8; 8]`     | `b"NESREFC\0"` |
//! | `version`                | `u32`         | `1` |
//! | `ref_wasm_sha256`        | `String`      | hex sha256 of the reference wasm |
//! | `rom_sha256`             | `String`      | hex sha256 of the ROM |
//! | `replay_sha256`          | `String`      | hex sha256 of the replay file (of `""` for no replay) |
//! | `frame_count`            | `u32`         | frames recorded |
//! | `audio_rate`             | `u32`         | Hz, 44 100 under the ABI |
//! | `frames`                 | `Vec<u16>`    | `frame_count × 61 440` palette indices, frame-major, row-major |
//! | `audio_samples_per_frame`| `Vec<u32>`    | `frame_count` mono sample counts |
//! | `audio`                  | `Vec<i16>`    | `sum(audio_samples_per_frame)` mono samples |
//! | `frame_diff_threshold`   | `f32`         | video τ for this replay |
//! | `audio_diff_threshold`   | `f32`         | audio τ; 0.0 when the reference is silent |
//!
//! Frames hold the ABI's `(emphasis << 6) | colour` values, not RGB: the
//! palette is applied when the cache is read, so the cache records what
//! the NES computed and stays valid if the display palette changes.
//!
//! # Invalidation
//!
//! A cache is used only if its magic, version and all three hashes match
//! the current inputs. Anything else is treated as absent and regenerated.

use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use lockstep::palette::index_to_rgba;
use lockstep::{defect_threshold_clamped, derive_audio_threshold, video, Reference, NES_PIXELS};

const CACHE_MAGIC: [u8; 8] = *b"NESREFC\0";
/// Bumped when the layout changes; mismatch triggers regenerate.
const CACHE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct ReferenceCache {
    magic: [u8; 8],
    version: u32,
    ref_wasm_sha256: String,
    rom_sha256: String,
    replay_sha256: String,
    frame_count: u32,
    audio_rate: u32,
    frames: Vec<u16>,
    audio_samples_per_frame: Vec<u32>,
    audio: Vec<i16>,
    frame_diff_threshold: f32,
    audio_diff_threshold: f32,
}

/// Compute the sha256 hex digest of the given bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Hash a file's contents. Returns `None` if the file can't be read.
pub fn sha256_file(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| sha256_hex(&b))
}

/// Path where the cache for a given testcase lives inside `corpus/`.
pub fn cache_path_for(corpus_dir: &Path, testcase_id: &str) -> PathBuf {
    corpus_dir
        .join("reference-cache")
        .join(format!("{testcase_id}.refcache"))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CacheStatus {
    /// Cache at path already current — `make_reference` was not called.
    AlreadyCurrent,
    /// Cache was absent or stale; a fresh one was written.
    Wrote,
}

/// Ensure the cache at `path` is current for `(ref_wasm_sha256,
/// rom_sha256, replay_sha256)`. If so, no-op. Otherwise calls
/// `make_reference`, plays the replay through it, and writes the cache.
pub fn ensure_written<F>(
    path: &Path,
    replay: &lockstep::InputReplay,
    frame_count: u32,
    ref_wasm_sha256: &str,
    rom_sha256: &str,
    replay_sha256: &str,
    make_reference: F,
) -> Result<CacheStatus>
where
    F: FnOnce() -> Result<Box<dyn Reference>>,
{
    if let Ok(Some(_)) = load(path, ref_wasm_sha256, rom_sha256, replay_sha256) {
        return Ok(CacheStatus::AlreadyCurrent);
    }
    let mut reference = make_reference()?;
    write(
        path,
        reference.as_mut(),
        replay,
        frame_count,
        ref_wasm_sha256.to_string(),
        rom_sha256.to_string(),
        replay_sha256.to_string(),
    )?;
    Ok(CacheStatus::Wrote)
}

/// Write a cache file by running the given reference through the full
/// replay, capturing every framebuffer + audio drain.
pub fn write(
    path: &Path,
    reference: &mut dyn Reference,
    replay: &lockstep::InputReplay,
    frame_count: u32,
    ref_wasm_sha256: String,
    rom_sha256: String,
    replay_sha256: String,
) -> Result<()> {
    let mut frames = Vec::with_capacity(frame_count as usize * NES_PIXELS);
    let mut audio_samples_per_frame = Vec::with_capacity(frame_count as usize);
    let mut audio = Vec::new();
    let mut ref_defects: Vec<f32> = Vec::with_capacity(frame_count as usize);
    let mut prev_fb: [u32; NES_PIXELS] = [0; NES_PIXELS];

    for _ in 0..reference.boot_frames() {
        reference.run_frame();
        let _ = reference.drain_audio(); // discard boot-period audio
    }

    for frame in 0..frame_count {
        reference.set_keys(replay.keys_at(frame));
        reference.run_frame();

        frames.extend_from_slice(reference.index_framebuffer());

        let fb = reference.framebuffer();
        if frame > 0 && video::ref_in_motion(&prev_fb, fb) {
            ref_defects.push(video::ssim_floored(&prev_fb, fb));
        }
        prev_fb.copy_from_slice(fb);

        let samples = reference.drain_audio();
        audio_samples_per_frame.push(samples.len() as u32);
        audio.extend(samples);
    }

    let frame_diff_threshold = defect_threshold_clamped(&mut ref_defects);
    let audio_rate = reference.audio_rate();
    let audio_diff_threshold = derive_audio_threshold(&audio, audio_rate) as f32;

    let cache = ReferenceCache {
        magic: CACHE_MAGIC,
        version: CACHE_VERSION,
        ref_wasm_sha256,
        rom_sha256,
        replay_sha256,
        frame_count,
        audio_rate,
        frames,
        audio_samples_per_frame,
        audio,
        frame_diff_threshold,
        audio_diff_threshold,
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {parent:?}"))?;
    }
    let file = std::fs::File::create(path).with_context(|| format!("create {path:?}"))?;
    let mut zstd_writer =
        zstd::Encoder::new(BufWriter::new(file), 3).context("init zstd encoder")?;
    bincode::serialize_into(&mut zstd_writer, &cache).context("bincode serialize")?;
    zstd_writer.finish().context("zstd finish")?;
    Ok(())
}

/// Reads + decompresses the cache, validates it.
///
/// Returns `Ok(Some(cache))` if the cache exists and is valid for the
/// given inputs, `Ok(None)` if it is absent or stale, or `Err` on I/O
/// errors and internally inconsistent files.
pub fn load(
    path: &Path,
    expected_ref_wasm_sha256: &str,
    expected_rom_sha256: &str,
    expected_replay_sha256: &str,
) -> Result<Option<CachedReference>> {
    if !path.exists() {
        return Ok(None);
    }
    let file = std::fs::File::open(path).with_context(|| format!("open {path:?}"))?;
    let zstd_reader = zstd::Decoder::new(BufReader::new(file)).context("init zstd decoder")?;
    let Ok(cache) = bincode::deserialize_from::<_, ReferenceCache>(zstd_reader) else {
        eprintln!("note: {path:?} is not a v{CACHE_VERSION} NES reference cache; recomputing.");
        return Ok(None);
    };

    if cache.magic != CACHE_MAGIC || cache.version != CACHE_VERSION {
        eprintln!("note: {path:?} has an old cache format; recomputing.");
        return Ok(None);
    }
    if cache.ref_wasm_sha256 != expected_ref_wasm_sha256
        || cache.rom_sha256 != expected_rom_sha256
        || cache.replay_sha256 != expected_replay_sha256
    {
        return Ok(None);
    }

    let expected_frames = cache.frame_count as usize * NES_PIXELS;
    if cache.frames.len() != expected_frames {
        bail!(
            "{path:?} corrupt: frames has {} pixels, expected {expected_frames}",
            cache.frames.len()
        );
    }
    if cache.audio_samples_per_frame.len() != cache.frame_count as usize {
        bail!(
            "{path:?} corrupt: audio_samples_per_frame has {} entries, expected {}",
            cache.audio_samples_per_frame.len(),
            cache.frame_count
        );
    }
    let expected_audio: u64 = cache.audio_samples_per_frame.iter().map(|&n| u64::from(n)).sum();
    if cache.audio.len() as u64 != expected_audio {
        bail!(
            "{path:?} corrupt: audio has {} samples, expected {expected_audio}",
            cache.audio.len()
        );
    }

    Ok(Some(CachedReference::new(cache)))
}

/// Replays a cached reference run. Implements `Reference` so the grader's
/// lockstep loop drives it exactly like a live emulator; it just advances
/// a cursor.
pub struct CachedReference {
    cache: ReferenceCache,
    frame_cursor: i64,
    index_scratch: Box<[u16; NES_PIXELS]>,
    rgba_scratch: Box<[u32; NES_PIXELS]>,
    audio_offset: usize,
}

impl CachedReference {
    fn new(cache: ReferenceCache) -> Self {
        Self {
            cache,
            frame_cursor: -1,
            index_scratch: Box::new([0; NES_PIXELS]),
            rgba_scratch: Box::new([0; NES_PIXELS]),
            audio_offset: 0,
        }
    }
}

impl Reference for CachedReference {
    fn name(&self) -> &str {
        "reference (cached)"
    }

    fn run_frame(&mut self) {
        self.frame_cursor += 1;
        let Ok(idx) = usize::try_from(self.frame_cursor) else { return };
        if idx >= self.cache.frame_count as usize {
            // Past the end: keep the last real frame.
            return;
        }
        let start = idx * NES_PIXELS;
        self.index_scratch
            .copy_from_slice(&self.cache.frames[start..start + NES_PIXELS]);
        for (dst, &px) in self.rgba_scratch.iter_mut().zip(self.index_scratch.iter()) {
            *dst = index_to_rgba(px);
        }
    }

    fn set_keys(&mut self, _keys: u16) {
        // No-op: the replay's inputs are already baked into the cached frames.
    }

    fn framebuffer(&self) -> &[u32; NES_PIXELS] {
        &self.rgba_scratch
    }

    fn index_framebuffer(&self) -> &[u16; NES_PIXELS] {
        &self.index_scratch
    }

    fn drain_audio(&mut self) -> Vec<i16> {
        let Ok(idx) = usize::try_from(self.frame_cursor) else { return Vec::new() };
        if idx >= self.cache.frame_count as usize {
            return Vec::new();
        }
        let n = self.cache.audio_samples_per_frame[idx] as usize;
        let end = self.audio_offset + n;
        let slice = &self.cache.audio[self.audio_offset..end];
        self.audio_offset = end;
        slice.to_vec()
    }

    fn audio_rate(&self) -> u32 {
        self.cache.audio_rate
    }
}

/// Quick statistics for logging + CI sanity.
pub fn cache_size_summary(corpus_dir: &Path) -> Result<String> {
    let dir = corpus_dir.join("reference-cache");
    if !dir.exists() {
        return Ok("no cache".into());
    }
    let mut count = 0;
    let mut total_bytes = 0u64;
    for entry in std::fs::read_dir(&dir).with_context(|| format!("readdir {dir:?}"))? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("refcache") {
            count += 1;
            total_bytes += entry.metadata()?.len();
        }
    }
    Ok(format!("{count} cache files, {:.1} MB on disk", total_bytes as f64 / 1024.0 / 1024.0))
}
