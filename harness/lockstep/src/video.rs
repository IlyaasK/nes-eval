//! Structural frame comparison.
//!
//! Per-frame score is `ssim_floored` — classical block SSIM (Wang et al.
//! 2004) on 8×8 non-overlapping blocks with a sub-JND perceptual floor,
//! computed on BT.601 luma of the palette-mapped RGB. Every emulator's
//! pixels come from the same fixed palette (`crate::palette`), so no
//! colour normalisation is needed before the luma projection.
//!
//! The metric targets structural/edge fidelity. It is intentionally less
//! sensitive to uniform global rendering drift (same edges, shifted luma)
//! — those cases show up in the `audit_luma_mae` diagnostic instead.
//!
//! Forked from GBA Eval. NES changes: 256×240 frames, 8×8 blocks (the
//! frame tiles exactly into 32×30 of them, aligned to NES tiles; GBA Eval's
//! 10×10 would leave a 6-pixel column unscored), no 5-bit quantization.

use crate::{NES_H, NES_PIXELS, NES_W};

/// Project to BT.601 luma. Returns a 256×240 row-major luma buffer in the
/// range [0, 255].
fn framebuffer_to_luma(fb: &[u32; NES_PIXELS]) -> Vec<f32> {
    let mut out = vec![0.0f32; NES_PIXELS];
    for (i, &px) in fb.iter().enumerate() {
        let r = (px & 0xFF) as f32;
        let g = ((px >> 8) & 0xFF) as f32;
        let b = ((px >> 16) & 0xFF) as f32;
        out[i] = 0.299 * r + 0.587 * g + 0.114 * b;
    }
    out
}

/// Per-block perceptual floor for `ssim_floored`. Block defects
/// below this are treated as sub-JND and contribute zero.
pub const SSIM_FLOORED_PERCEPTUAL_FLOOR: f32 = 0.15;

/// Block size for `ssim_floored`. The 256×240 frame splits into
/// 32×30 = 960 non-overlapping 8×8 blocks.
const SSIM_FLOORED_BLOCK: usize = 8;

/// Classical L·C·S block SSIM with a sub-JND floor, averaged over
/// all non-overlapping 8×8 blocks of the BT.601 luma produced by
/// `framebuffer_to_luma`.
///
/// Aggregation: per block, compute 1 − SSIM_block; contribute it if
/// ≥ PERCEPTUAL_FLOOR, else zero. Mean over blocks. Result is a
/// structural defect in [0, 1].
pub fn ssim_floored(ref_fb: &[u32; NES_PIXELS], cand_fb: &[u32; NES_PIXELS]) -> f32 {
    let r = framebuffer_to_luma(ref_fb);
    let c = framebuffer_to_luma(cand_fb);
    let w = NES_W;
    let h = NES_H;
    let block = SSIM_FLOORED_BLOCK;
    // Wang et al. 2004 constants on an 8-bit luma range. Same as
    // the reference SSIM paper; K1=0.01, K2=0.03.
    let c1 = (0.01 * 255.0f32).powi(2);
    let c2 = (0.03 * 255.0f32).powi(2);
    let mut sum_defect = 0.0f32;
    let mut n_total = 0u32;
    let mut y = 0;
    while y + block <= h {
        let mut x = 0;
        while x + block <= w {
            let mut s_r = 0.0f32;
            let mut s_c = 0.0f32;
            let mut ss_r = 0.0f32;
            let mut ss_c = 0.0f32;
            let mut sxy = 0.0f32;
            let n = (block * block) as f32;
            for dy in 0..block {
                for dx in 0..block {
                    let i = (y + dy) * w + (x + dx);
                    let rv = r[i];
                    let cv = c[i];
                    s_r += rv;
                    s_c += cv;
                    ss_r += rv * rv;
                    ss_c += cv * cv;
                    sxy += rv * cv;
                }
            }
            let mu_r = s_r / n;
            let mu_c = s_c / n;
            let var_r = (ss_r / n - mu_r * mu_r).max(0.0);
            let var_c = (ss_c / n - mu_c * mu_c).max(0.0);
            let cov = sxy / n - mu_r * mu_c;
            let l = (2.0 * mu_r * mu_c + c1) / (mu_r * mu_r + mu_c * mu_c + c1);
            let cs = (2.0 * cov + c2) / (var_r + var_c + c2);
            let block_ssim = (l * cs).clamp(0.0, 1.0);
            let block_defect = 1.0 - block_ssim;
            n_total += 1;
            if block_defect >= SSIM_FLOORED_PERCEPTUAL_FLOOR {
                sum_defect += block_defect;
            }
            x += block;
        }
        y += block;
    }
    if n_total == 0 {
        return 0.0;
    }
    (sum_defect / n_total as f32).clamp(0.0, 1.0)
}

/// Mean per-pixel luma absolute error. Audit-only diagnostic: a
/// candidate that gets edge structure right but applies a small
/// systematic luma shift can still score high on `ssim_floored` (the
/// CS term dominates within each block) while producing a non-zero
/// MAE here, and that's the signal we want to see separately from the
/// main score.
///
/// Runs on the same luma as `ssim_floored`, so both diagnostics speak the
/// same units.
pub fn luma_mae(ref_fb: &[u32; NES_PIXELS], cand_fb: &[u32; NES_PIXELS]) -> f32 {
    let ref_luma = framebuffer_to_luma(ref_fb);
    let cand_luma = framebuffer_to_luma(cand_fb);
    let mut acc = 0.0f32;
    for i in 0..NES_PIXELS {
        acc += (ref_luma[i] - cand_luma[i]).abs();
    }
    acc / NES_PIXELS as f32
}

/// True iff any pixel's colour differs between the two framebuffers.
/// Used as the ref-motion gate: "static" ref transitions are excluded
/// from the τ estimator because a p90 driven by mostly-identical frames
/// collapses τ to the floor even when the motion-frame population is
/// perfectly well-defined.
pub fn ref_in_motion(prev: &[u32; NES_PIXELS], curr: &[u32; NES_PIXELS]) -> bool {
    (0..NES_PIXELS).any(|i| crate::rgb(prev[i]) != crate::rgb(curr[i]))
}

// ─────────────────────────────────────────────────────────────────────────
// τ calibration
// ─────────────────────────────────────────────────────────────────────────

/// Lower bound on τ. Frame-level defect below this is treated as
/// "within reference noise" regardless of how static the replay is —
/// catches candidates with small luma drift on
/// near-static replays where p90(ref-ref defect) collapses to ~0.
///
pub const TAU_MIN: f32 = 0.005;

/// Upper bound on τ. High-motion replays should not buy unbounded
/// forgiveness — a black-screen candidate on a rhythm game with heavy
/// motion has a legitimate structural defect even if ref-to-ref
/// defect runs high. Caps the "the reference moves a lot, so the
/// candidate gets forgiven a lot" compensation.
///
pub const TAU_MAX: f32 = 0.35;

/// Percentile of the motion-gated ref-to-ref `ssim_floored` distribution
/// used as the adaptive τ. 90 matches the pixel-diff pipeline's
/// percentile so the "exclude top 10% scene cuts" rationale carries over.
pub const TAU_PERCENTILE: f32 = 0.90;

/// Shape parameter of the per-frame sigmoid. Kept at 4 from the
/// pixel-diff era — frame_score(0.5τ) ≈ 0.94, frame_score(τ) = 0.5,
/// frame_score(2τ) ≈ 0.06, so the "fraction of frames close enough"
/// reading carries over unchanged.
pub const SHARPNESS: i32 = 4;

/// Tight τ for end-state scoring (verdict-screen ROMs). Deliberately
/// set well below the noise a structural metric can produce on any
/// visible text/digit difference — at 1e-4 with SHARPNESS=4, an
/// `ssim_floored` defect of even ~0.001 (a single wrong glyph pixel)
/// gives D/τ ≈ 10 and collapses the score to effectively zero. A
/// bit-exact verdict framebuffer still scores 1 (D=0). This is the
/// "essentially exact pixel match" rule — anything short of
/// bit-identical on a PASS/FAIL grid is a fail.
pub const ENDSTATE_TAU: f32 = 1.0e-4;

/// Clamp the adaptive τ from a motion-gated ref-to-ref `ssim_floored`
/// series. Equivalent to `delta_threshold_clamped` in the old pixel-
/// diff world, but in structural-defect units. `defects` is mutated
/// (sorted).
pub fn defect_threshold_clamped(defects: &mut [f32]) -> f32 {
    if defects.is_empty() {
        return TAU_MIN;
    }
    defects.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((defects.len() as f32) * TAU_PERCENTILE) as usize;
    let idx = idx.min(defects.len() - 1);
    defects[idx].clamp(TAU_MIN, TAU_MAX)
}
