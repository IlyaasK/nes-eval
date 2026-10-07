//! Lockstep frame comparison against a reference emulator.
//!
//! The grader drives this. It instantiates a reference and a candidate
//! (both wasm modules implementing spec/ABI.md, hosted in wasmtime), feeds
//! both the same ROM and the same frame-indexed input log, and compares
//! framebuffers every frame.
//!
//! Everything is generic over the `Reference` trait. Adding another
//! reference is one more impl with no changes to `lockstep()`.
//!
//! Forked from GBA Eval (github.com/mechanize-work/gba-eval, MIT) and
//! adapted to the NES: 256×240 frames, palette-index framebuffers mapped
//! through one fixed palette, mono audio.

pub mod audio;
pub mod media;
pub mod palette;
pub mod replay;
pub mod result;
pub mod video;
pub mod video_encode;

pub use replay::InputReplay;
pub use result::{CompareResult, FrameDiff};
pub use audio::{derive_threshold as derive_audio_threshold, score_logmel};
pub use video::{
    defect_threshold_clamped, luma_mae, ref_in_motion,
    ENDSTATE_TAU, SHARPNESS, TAU_MAX, TAU_MIN, TAU_PERCENTILE,
};

pub const NES_W: usize = 256;
pub const NES_H: usize = 240;
pub const NES_PIXELS: usize = NES_W * NES_H;

// ─────────────────────────────────────────────────────────────────────────
// Reference trait — what every comparison target implements.
// ─────────────────────────────────────────────────────────────────────────

/// An emulator we can drive frame-by-frame.
///
/// Both sides of a comparison implement this — the reference and the
/// candidate. `lockstep()` doesn't care which is which.
pub trait Reference {
    fn name(&self) -> &str;

    /// Advance one frame. After this returns, `framebuffer()` holds the
    /// frame just rendered (scanlines 0–239, see spec/ABI.md "Frames").
    fn run_frame(&mut self);

    /// Active-high controller 1, NES shift order. Bit 0=A, 1=B, 2=Select,
    /// 3=Start, 4=Up, 5=Down, 6=Left, 7=Right. Latched until next call.
    fn set_keys(&mut self, keys: u16);

    /// 256×240 pixels, 0xAABBGGRR, already mapped from palette indices
    /// through `palette::index_to_rgba`. Alpha is 0xFF. The same buffer
    /// gets overwritten each `run_frame()`.
    fn framebuffer(&self) -> &[u32; NES_PIXELS];

    /// The same frame as `framebuffer()`, before palette mapping:
    /// `(emphasis << 6) | colour` per pixel, masked to 9 bits. This is
    /// what the reference cache stores.
    fn index_framebuffer(&self) -> &[u16; NES_PIXELS];

    /// Drain audio produced since the last drain: mono i16 samples,
    /// ~735 per frame at 44.1 kHz. Backends that don't support audio
    /// return empty.
    fn drain_audio(&mut self) -> Vec<i16> {
        Vec::new()
    }

    /// Sample rate of `drain_audio()` output in Hz. The ABI fixes it at
    /// 44 100.
    fn audio_rate(&self) -> u32 {
        44_100
    }

    /// Frames between init and "the framebuffer holds game frame 0".
    /// The ABI says `load_rom()` leaves you AT frame 0, so conformant
    /// candidates and the reference return 0. Kept from GBA Eval for
    /// candidates that wrap an emulator with a warmup cost (optional
    /// `emu_boot_frames` export).
    ///
    /// `lockstep()` burns each side's boot frames before comparing, so
    /// "frame N" means the same thing on both.
    fn boot_frames(&self) -> u32 {
        0
    }
}

/// Forward to the boxed impl. Lets `lockstep()`'s `&mut impl Reference`
/// signature accept a `Box<dyn Reference>` — the grader doesn't need
/// to know the concrete type at all.
///
/// Every defaulted trait method MUST be forwarded here — the trait
/// default would silently shadow the boxed impl's override otherwise.
impl Reference for Box<dyn Reference> {
    fn name(&self) -> &str { (**self).name() }
    fn run_frame(&mut self) { (**self).run_frame() }
    fn set_keys(&mut self, keys: u16) { (**self).set_keys(keys) }
    fn framebuffer(&self) -> &[u32; NES_PIXELS] { (**self).framebuffer() }
    fn index_framebuffer(&self) -> &[u16; NES_PIXELS] { (**self).index_framebuffer() }
    fn drain_audio(&mut self) -> Vec<i16> { (**self).drain_audio() }
    fn audio_rate(&self) -> u32 { (**self).audio_rate() }
    fn boot_frames(&self) -> u32 { (**self).boot_frames() }
}

// ─────────────────────────────────────────────────────────────────────────
// Pixel comparison
// ─────────────────────────────────────────────────────────────────────────

/// The RGB part of a framebuffer pixel, alpha masked.
///
/// GBA Eval quantized to 5 bits per channel here because GBA emulators
/// expand 5-bit colour to 8 bits in different ways. NES candidates hand
/// over palette indices instead, and every index goes through the same
/// fixed table (`palette::index_to_rgba`), so two pixels have the same RGB
/// exactly when the candidates computed the same NES colour. No
/// quantization is needed.
#[inline]
pub fn rgb(px: u32) -> u32 {
    px & 0x00FF_FFFF
}

/// Per-frame count of pixels whose colour differs.
pub fn diff_frame(a: &[u32; NES_PIXELS], b: &[u32; NES_PIXELS]) -> usize {
    let mut count = 0;
    for i in 0..NES_PIXELS {
        if rgb(a[i]) != rgb(b[i]) {
            count += 1;
        }
    }
    count
}

/// Per-frame diff with first-mismatch coordinates. Slower; for diagnostics.
pub fn diff_frame_detail(
    frame: u32,
    a: &[u32; NES_PIXELS],
    b: &[u32; NES_PIXELS],
) -> FrameDiff {
    let mut count = 0;
    let mut first = None;
    for i in 0..NES_PIXELS {
        if rgb(a[i]) != rgb(b[i]) {
            if first.is_none() {
                first = Some((i % NES_W, i / NES_W, a[i], b[i]));
            }
            count += 1;
        }
    }
    FrameDiff { frame, differing_pixels: count, first_diff: first }
}

// ─────────────────────────────────────────────────────────────────────────
// Lockstep — the comparison kernel
// ─────────────────────────────────────────────────────────────────────────

/// A frame "diverges" if it differs by more than this many pixels.
/// Not zero — sprite-edge races and similar can produce a handful of
/// pixel-level differences even on correct emulators. Real bugs
/// produce hundreds.
pub const NOISE_FLOOR: usize = 8;

/// Drive both emulators in lockstep, applying the same inputs each frame.
///
/// Each side burns its `boot_frames()` first to align "frame 0" to the
/// same game state. After that, frame N means the same thing on both.
///
/// `inputs` is indexed by post-boot frame number.
/// RMS threshold below which a frame is considered silent. ~-50 dB in i16
/// space.
const SILENCE_RMS: f64 = 100.0;

pub fn lockstep(
    reference: &mut impl Reference,
    candidate: &mut impl Reference,
    n_frames: u32,
    inputs: &InputReplay,
    mut video: Option<&mut video_encode::VideoEncoder>,
) -> LockstepOutput {
    // Drain boot-period audio from both sides so frame 0 starts with
    // an empty buffer on both. Skipping this leaks boot samples into
    // frame 0's first drain and shifts the log-mel analysis grid for
    // the rest of the replay.
    for _ in 0..reference.boot_frames() {
        reference.run_frame();
        let _ = reference.drain_audio();
    }
    for _ in 0..candidate.boot_frames() {
        candidate.run_frame();
        let _ = candidate.drain_audio();
    }

    let mut r = CompareResult::new(n_frames);
    let audio_rate = reference.audio_rate();
    let mut ref_audio: Vec<i16> = Vec::new();
    let mut cand_audio: Vec<i16> = Vec::new();

    // Track consecutive-reference `ssim_floored` defects on frames
    // where the ref actually changes. The motion gate keeps long idle
    // stalls from collapsing the p90 estimator to zero.
    let mut prev_ref_fb: [u32; NES_PIXELS] = [0; NES_PIXELS];
    let mut ref_defects: Vec<f32> = Vec::with_capacity(n_frames as usize);
    // Per-frame "is this a new run" flags, for the run-collapsed audit
    // score. Frame 0 always starts a new run; later frames start a new
    // run iff the reference changed since the prior frame.
    let mut new_run: Vec<bool> = Vec::with_capacity(n_frames as usize);

    for frame in 0..n_frames {
        let keys = inputs.keys_at(frame);
        reference.set_keys(keys);
        candidate.set_keys(keys);
        reference.run_frame();
        candidate.run_frame();

        // ── Video ──
        let ref_fb = reference.framebuffer();
        let cand_fb = candidate.framebuffer();

        // Mirror frames into the encoder before the scoring math — the
        // video is a visual record of exactly what was scored. A write
        // failure (ffmpeg died, disk full) drops video for the rest of
        // the testcase but shouldn't abort scoring: log once and null
        // the encoder so later frames short-circuit cheaply.
        if let Some(enc) = video.as_deref_mut() {
            if let Err(e) = enc.push(ref_fb, cand_fb) {
                eprintln!("warning: video encoder failed on frame {frame}: {e}");
                video = None;
            }
        }

        // Structural defect: floored-SSIM on 8×8 blocks of luma. Zero
        // when ref/cand agree on every pixel's colour; grows as
        // edges/gradients diverge. Global luma drift with preserved
        // edges is captured separately as `audit_luma_mae`, not scored.
        let mut defect = video::ssim_floored(ref_fb, cand_fb);
        // Blank-frame gate: when the candidate is flat (≥99.9% one
        // colour) but the reference isn't, force the defect to its
        // max so the sigmoid can't forgive a stuck/black candidate
        // via a permissive τ.
        if is_flat_frame(cand_fb) && !is_flat_frame(ref_fb) {
            defect = 1.0;
        }
        r.histogram.push(defect);

        // Luma MAE — audit only, not scored in v1. Written per-frame so
        // the final replay-level mean is a simple average downstream.
        r.audit_luma_mae_frames.push(video::luma_mae(ref_fb, cand_fb));

        // Pixel-count diagnostics. Independent of the scored metric;
        // for human-readable reports where pixel counts are legible.
        let raw_diff = diff_frame(ref_fb, cand_fb);
        if raw_diff > NOISE_FLOOR {
            r.diverging_frames += 1;
            r.total_diff_pixels += raw_diff as u64;
            if r.first_diverge_frame.is_none() {
                r.first_diverge_frame = Some(frame);
            }
            if raw_diff > r.max_diff_pixels {
                r.max_diff_pixels = raw_diff;
                r.max_diff_frame = Some(frame);
            }
        }

        let in_motion = frame == 0 || video::ref_in_motion(&prev_ref_fb, ref_fb);
        new_run.push(in_motion);
        if frame > 0 && in_motion {
            ref_defects.push(video::ssim_floored(&prev_ref_fb, ref_fb));
        }
        prev_ref_fb.copy_from_slice(ref_fb);

        // ── Audio ──
        let ra = reference.drain_audio();
        let ca = candidate.drain_audio();

        let ref_rms = audio::rms(&ra);
        let cand_rms = audio::rms(&ca);
        r.audio_rms_ratio.push(if ref_rms > SILENCE_RMS {
            Some((cand_rms / ref_rms) as f32)
        } else {
            None
        });

        ref_audio.extend_from_slice(&ra);
        cand_audio.extend_from_slice(&ca);
    }

    // Audio scoring: per-frame log-mel L1 → sigmoid with τ = p90 of
    // the reference's own adjacent-frame diffs (silent pairs excluded).
    // Mirrors the video scorer — see `audio::score_logmel_detailed`.
    r.audio_diff_threshold = audio::derive_threshold(&ref_audio, audio_rate) as f32;
    if let Some(s) = audio::score_logmel_detailed(
        &ref_audio,
        &cand_audio,
        audio_rate,
        r.audio_diff_threshold as f64,
    ) {
        r.audio_score = Some(s.mean);
        r.audio_frame_scores = s.per_frame.into_iter()
            .map(|opt| opt.map(|v| v as f32))
            .collect();
    }
    r.frame_diff_threshold = video::defect_threshold_clamped(&mut ref_defects);

    // Run-collapsed replay score — diagnostic only. Groups consecutive
    // same-state reference frames into runs, averages frame scores
    // inside each run, then averages across runs.
    let tau = r.frame_diff_threshold;
    let mut run_sum = 0.0f64;
    let mut run_len = 0u32;
    let mut run_count = 0u32;
    let mut deduped = 0.0f64;
    for (i, &defect) in r.histogram.iter().enumerate() {
        if new_run[i] && run_len > 0 {
            deduped += run_sum / run_len as f64;
            run_count += 1;
            run_sum = 0.0;
            run_len = 0;
        }
        run_sum += CompareResult::frame_score(defect, tau);
        run_len += 1;
    }
    if run_len > 0 {
        deduped += run_sum / run_len as f64;
        run_count += 1;
    }
    r.replay_score_deduped = if run_count > 0 {
        (deduped / run_count as f64) as f32
    } else {
        1.0
    };

    LockstepOutput { result: r, ref_audio, cand_audio, audio_rate }
}

/// Minimum fraction of pixels matching the first pixel for a frame
/// to count as "flat" (candidate stuck on a single colour).
/// 99.9% leaves room for small overlays on near-solid backgrounds.
pub const FLAT_FRAME_THRESHOLD: f64 = 0.999;

/// True when ≥ `FLAT_FRAME_THRESHOLD` of the frame's pixels match its
/// first pixel.
pub fn is_flat_frame(fb: &[u32; NES_PIXELS]) -> bool {
    let first = rgb(fb[0]);
    let matching = fb.iter().filter(|&&p| rgb(p) == first).count();
    matching as f64 / NES_PIXELS as f64 >= FLAT_FRAME_THRESHOLD
}

/// Everything `lockstep()` produces. `CompareResult` serializes to JSON
/// for downstream playback; the audio buffers are written as separate WAV files.
pub struct LockstepOutput {
    pub result: CompareResult,
    pub ref_audio: Vec<i16>,
    pub cand_audio: Vec<i16>,
    pub audio_rate: u32,
}
