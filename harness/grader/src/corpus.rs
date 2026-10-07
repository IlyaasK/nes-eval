//! Corpus manifest: which ROMs, which replays, what each one tests.
//!
//! `corpus/testcases.json` is the source of truth. ROMs are referenced by
//! SHA-256 — the manifest doesn't care where the file lives, only that
//! its hash matches. All ROMs are homebrew or open-source test suites.
//!
//! Every map here is a `BTreeMap`: iteration order feeds float sums and
//! JSON key order, and both must be identical run to run.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use lockstep::InputReplay;

#[derive(Debug, Deserialize)]
pub struct Corpus {
    pub testcases: Vec<TestCase>,

    #[serde(skip)]
    rom_index: BTreeMap<String, PathBuf>,
    #[serde(skip)]
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct TestCase {
    pub id: String,
    /// Which of the three grading sections this belongs to.
    pub section: String,  // "procedural" | "replay"
    /// Subsystem tag for weighting within the section.
    pub subsystem: String,
    /// Audio subsystem tag, if this ROM is expected to produce audio.
    /// null/absent = no audio expected.
    #[serde(default)]
    pub audio_subsystem: Option<String>,
    /// Lowercase hex SHA-256 of the ROM.
    pub rom_sha256: String,
    /// Human-readable, for logs. Not used for lookup.
    pub rom_name: String,
    /// Path relative to `corpus/replays/`. Empty string = no input.
    #[serde(default)]
    pub replay: String,
    /// Frames to run.
    pub frames: u32,
    /// How to aggregate per-frame video scores.
    /// `FrameMean` (default) scores the mean over all frames — the
    /// right thing for timing-sensitive comparisons. `EndState` scores
    /// only the final frame — the right thing for self-checking ROMs
    /// that print PASS/FAIL on screen, where cycle drift during the
    /// test run is irrelevant as long as the final verdict matches.
    #[serde(default)]
    pub scoring_mode: ScoringMode,
    #[allow(dead_code)]
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScoringMode {
    /// Mean of per-frame sigmoid scores over the full histogram.
    #[default]
    FrameMean,
    /// Only the final frame's sigmoid score. For tests whose verdict
    /// lives in the final framebuffer (blargg's suites, nestest, etc.)
    /// and where inter-frame timing drift isn't part of what's being
    /// measured.
    #[serde(rename = "endstate")]
    EndState,
}

impl Corpus {
    pub fn load(dir: &Path) -> Result<Self> {
        let manifest = dir.join("testcases.json");
        let json = std::fs::read_to_string(&manifest)
            .with_context(|| format!("reading {manifest:?}"))?;
        let mut corpus: Corpus = serde_json::from_str(&json)
            .with_context(|| format!("parsing {manifest:?}"))?;

        corpus.root = dir.to_path_buf();
        corpus.rom_index = index_roms(&dir.join("roms"))?;

        // Validate sections
        for tc in &corpus.testcases {
            if !["procedural", "replay"].contains(&tc.section.as_str()) {
                eprintln!("warning: testcase {} has unknown section '{}'", tc.id, tc.section);
            }
        }

        Ok(corpus)
    }

    pub fn load_rom(&self, tc: &TestCase) -> Result<Vec<u8>> {
        let path = self.rom_index.get(&tc.rom_sha256).ok_or_else(|| {
            anyhow::anyhow!(
                "no ROM with sha256={} found under {:?}/roms/. \
                 Expected: {}.",
                tc.rom_sha256, self.root, tc.rom_name
            )
        })?;
        std::fs::read(path).with_context(|| format!("reading {path:?}"))
    }

    pub fn load_replay(&self, tc: &TestCase) -> Result<InputReplay> {
        if tc.replay.is_empty() {
            return Ok(InputReplay::new());
        }
        let path = self.root.join("replays").join(&tc.replay);
        InputReplay::from_file(&path)
            .with_context(|| format!("reading replay {path:?}"))
    }

    /// sha256 of the raw replay file bytes, or of the empty string for
    /// no-input testcases. Used as a cache key — same inputs, same
    /// reference output, same cache.
    pub fn replay_sha256(&self, tc: &TestCase) -> Option<String> {
        if tc.replay.is_empty() {
            return Some(crate::ref_cache::sha256_hex(&[]));
        }
        let path = self.root.join("replays").join(&tc.replay);
        crate::ref_cache::sha256_file(&path)
    }
}

/// Walk `roms/` recursively, hash every `.nes`, build sha → path.
/// Directory entries are sorted so duplicate-ROM notes are reproducible.
fn index_roms(dir: &Path) -> Result<BTreeMap<String, PathBuf>> {
    let mut index = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(d) = stack.pop() {
        let mut entries: Vec<PathBuf> = match std::fs::read_dir(&d) {
            Ok(e) => e.flatten().map(|e| e.path()).collect(),
            Err(_) => continue, // dir doesn't exist yet, fine
        };
        entries.sort();
        for path in entries {
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) != Some("nes") {
                continue;
            }
            let bytes = std::fs::read(&path)?;
            let hash = format!("{:x}", Sha256::digest(&bytes));
            if let Some(prev) = index.insert(hash.clone(), path.clone()) {
                // Two files with the same hash — duplicate ROM. Harmless
                // but worth knowing.
                eprintln!("note: duplicate ROM: {prev:?} == {path:?}");
            }
        }
    }

    eprintln!("indexed {} ROMs under {dir:?}", index.len());
    Ok(index)
}

// ─────────────────────────────────────────────────────────────────────────
// Aggregate summary — one per candidate, denormalized for downstream consumers.
// ─────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct SectionScore {
    pub score: f64,
    pub weight: f64,
    pub subsystems: BTreeMap<String, f64>,
}

/// No timestamp: the brief keeps wall-clock time out of the grading path,
/// and a reproducible `summary.json` is easier to diff across runs.
#[derive(Debug, Serialize)]
pub struct Summary {
    pub candidate: String,
    pub candidate_sha256: String,

    /// Per-testcase scores.
    pub per_testcase: BTreeMap<String, TestCaseScore>,

    /// The three grading sections.
    pub sections: BTreeMap<String, SectionScore>,

    /// Weighted overall. The overall sort key.
    pub overall: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TestCaseScore {
    pub video_score: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_score: Option<f64>,
    pub section: String,
    pub subsystem: String,
    /// True when the candidate couldn't run this testcase (typically a
    /// load_rom trap from a wasm whose linear memory is too small for
    /// the ROM). Such entries score 0 — the candidate is responsible
    /// for declaring enough memory — but they still appear in the
    /// summary so consumers can render a placeholder panel instead of
    /// silently dropping the testcase.
    #[serde(skip_serializing_if = "is_false")]
    pub skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Summary {
    pub fn new(candidate: String, wasm_bytes: &[u8]) -> Self {
        Self {
            candidate,
            candidate_sha256: format!("{:x}", Sha256::digest(wasm_bytes)),
            per_testcase: BTreeMap::new(),
            sections: BTreeMap::new(),
            overall: 0.0,
        }
    }
}
