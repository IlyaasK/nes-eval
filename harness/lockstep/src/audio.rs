//! Audio comparison: per-frame log-mel spectrogram diff with a
//! per-replay sigmoid threshold.
//!
//! Mirrors the video scoring formula (see `CompareResult::frame_score`):
//!
//! 1. For every STFT frame, compute log-mel vectors of reference and
//!    candidate (`LOGMEL_N_MEL` = 40 bins, Hann-windowed, FFT_SIZE =
//!    1024). STFT hop = `audio_rate / 60`, one analysis frame per
//!    emulated frame — independent of drain-boundary jitter so the two
//!    sides always share the same analysis grid.
//! 2. Compute `d(n) = L1(ref_logmel[n], cand_logmel[n])` with **per-frame
//!    mean subtraction (CMN)** — each vector's own mean is removed
//!    before the L1, so a constant gain offset cancels and the metric
//!    measures spectral shape rather than absolute loudness. Loudness
//!    is reported separately as `audio_rms_ratio`.
//! 3. Derive a per-replay τ from the reference's own adjacent-frame L1
//!    diffs (also CMN'd): `τ = quantile(AUDIO_QUANTILE, {L1(ref[n],
//!    ref[n-1])})`, silent-silent frame pairs excluded.
//! 4. Per-frame score:
//!    `score(d, τ) = 1 / (1 + (d/τ)^AUDIO_SHARPNESS)` — same shape as
//!    the video sigmoid.
//! 5. Mean over active frames (frames where at least one side is
//!    non-silent). All-silent replays return `None`.
//!
//! Forked from GBA Eval, which scored the left channel of a stereo
//! stream. NES audio is mono, so every buffer here is plain mono i16.

use std::sync::Arc;

use rustfft::{num_complex::Complex, Fft, FftPlanner};

// ── Log-mel analysis constants ──────────────────────────────────────────

/// FFT size. Covers one 60-Hz frame at 44 100 Hz (~735 samples) with
/// some headroom; power of two for FFT efficiency.
pub const LOGMEL_FFT_SIZE: usize = 1024;

/// Number of mel bins. 40 is standard for speech/music analysis and
/// gives a per-frame L1 diff in a comfortable numerical range.
pub const LOGMEL_N_MEL: usize = 40;

/// Low edge of the mel filterbank. Below ~80 Hz there's mostly DC
/// leakage / DMA hum rather than musical content.
pub const LOGMEL_F_MIN: f64 = 80.0;

/// High edge cap. Clamped by Nyquist when the audio rate is lower.
pub const LOGMEL_F_MAX_CAP: f64 = 8000.0;

/// Added inside `log(·)` to avoid `log(0)` on silent frames.
pub const LOGMEL_EPS: f64 = 1e-10;

// ── Scoring constants ───────────────────────────────────────────────────

/// Quantile of the reference's adjacent-frame L1 diffs used as τ.
/// 0.90 mirrors the video side (`TAU_PERCENTILE`).
pub const AUDIO_QUANTILE: f64 = 0.90;

/// Floor applied under the per-replay τ. Keeps very-quiet reference
/// audio (sparse chiptune, near-silent ambience) from collapsing τ
/// toward zero and demanding bit-exactness from candidates.
pub const AUDIO_DELTA_FLOOR: f64 = 25.0;

/// Sigmoid sharpness. Same value the video scorer uses — d = τ/2 → 0.94,
/// d = τ → 0.50, d = 2·τ → 0.06.
pub const AUDIO_SHARPNESS: i32 = 4;

/// Per-STFT-frame RMS below which the analysis frame is treated as
/// silent. Same scale as the existing `SILENCE_RMS` in `lib.rs` — ~-50
/// dB in i16 space.
pub const LOGMEL_SILENCE_RMS: f64 = 100.0;

/// Per-STFT-frame RMS below which a frame has no usable signal — the
/// i16 buffer is essentially flat zero and the log-mel vector is pure
/// floor. Short-circuits the CMN shape comparison so a null-vs-non-null
/// pair scores zero directly instead of accidentally getting partial
/// credit for matching the flat-zero shape. Set well below
/// `LOGMEL_SILENCE_RMS` so quiet-but-non-null candidates take the CMN path.
pub const LOGMEL_NULL_RMS: f64 = 2.0;

// ── Legacy envelope metric (kept for the RMS-ratio timeline) ────────────

/// RMS of a mono i16 buffer.
pub fn rms(buf: &[i16]) -> f64 {
    if buf.is_empty() {
        return 0.0;
    }
    let sum: f64 = buf.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    (sum / buf.len() as f64).sqrt()
}

// ── Log-mel pipeline ────────────────────────────────────────────────────

/// Precomputed per-replay analysis bits: FFT plan, Hann window, mel
/// filterbank. Share one instance across every frame of a single
/// comparison; building it touches `FftPlanner` which is not free.
pub struct LogMelContext {
    audio_rate: u32,
    fft: Arc<dyn Fft<f64>>,
    window: Vec<f64>,
    /// Row-major `[N_MEL][FFT_SIZE/2 + 1]`.
    filters: Vec<Vec<f64>>,
    hop: usize,
}

impl LogMelContext {
    pub fn new(audio_rate: u32) -> Self {
        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(LOGMEL_FFT_SIZE);
        let window = hann_window(LOGMEL_FFT_SIZE);
        let filters = mel_filterbank(audio_rate);
        let hop = (audio_rate as usize / 60).max(1);
        Self { audio_rate, fft, window, filters, hop }
    }

    pub fn audio_rate(&self) -> u32 {
        self.audio_rate
    }

    pub fn hop(&self) -> usize {
        self.hop
    }
}

/// One analysis frame's log-mel vector.
pub type LogMelFrame = [f64; LOGMEL_N_MEL];

fn hz_to_mel(f: f64) -> f64 {
    2595.0 * (1.0 + f / 700.0).log10()
}

fn mel_to_hz(m: f64) -> f64 {
    700.0 * (10f64.powf(m / 2595.0) - 1.0)
}

fn hann_window(n: usize) -> Vec<f64> {
    (0..n)
        .map(|i| 0.5 - 0.5 * (2.0 * std::f64::consts::PI * i as f64 / (n as f64 - 1.0)).cos())
        .collect()
}

fn mel_filterbank(audio_rate: u32) -> Vec<Vec<f64>> {
    let n_bins = LOGMEL_FFT_SIZE / 2 + 1;
    let fmax = (audio_rate as f64 / 2.0).min(LOGMEL_F_MAX_CAP);
    let mel_min = hz_to_mel(LOGMEL_F_MIN);
    let mel_max = hz_to_mel(fmax);
    let mut boundaries = Vec::with_capacity(LOGMEL_N_MEL + 2);
    for i in 0..(LOGMEL_N_MEL + 2) {
        let m = mel_min + (mel_max - mel_min) * (i as f64) / (LOGMEL_N_MEL as f64 + 1.0);
        let hz = mel_to_hz(m);
        boundaries.push(hz * LOGMEL_FFT_SIZE as f64 / audio_rate as f64);
    }
    let mut filters = Vec::with_capacity(LOGMEL_N_MEL);
    for f in 0..LOGMEL_N_MEL {
        let lo = boundaries[f];
        let mid = boundaries[f + 1];
        let hi = boundaries[f + 2];
        let mut row = vec![0.0; n_bins];
        for k in 0..n_bins {
            let kf = k as f64;
            let w = if kf <= lo || kf >= hi {
                0.0
            } else if kf <= mid {
                (kf - lo) / (mid - lo).max(1e-9)
            } else {
                (hi - kf) / (hi - mid).max(1e-9)
            };
            row[k] = w;
        }
        filters.push(row);
    }
    filters
}

fn compute_logmel_one(
    ctx: &LogMelContext,
    window: &[f64],
    buf: &mut Vec<Complex<f64>>,
) -> LogMelFrame {
    buf.clear();
    buf.resize(LOGMEL_FFT_SIZE, Complex { re: 0.0, im: 0.0 });
    let n = window.len().min(LOGMEL_FFT_SIZE);
    for i in 0..n {
        buf[i] = Complex {
            re: window[i] * ctx.window[i],
            im: 0.0,
        };
    }
    ctx.fft.process(buf);
    let n_bins = LOGMEL_FFT_SIZE / 2 + 1;
    let mut out = [0.0f64; LOGMEL_N_MEL];
    for (i, row) in ctx.filters.iter().enumerate() {
        let mut e = 0.0;
        for (k, &w) in row.iter().enumerate().take(n_bins) {
            e += w * buf[k].norm_sqr();
        }
        out[i] = (e + LOGMEL_EPS).ln();
    }
    out
}

/// Per-frame RMS of an f64 sample slice.
fn rms_slice(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return 0.0;
    }
    let s: f64 = slice.iter().map(|x| x * x).sum();
    (s / slice.len() as f64).sqrt()
}

/// Per-analysis-frame flags derived from the raw i16 RMS of each STFT
/// window. Two thresholds, not one:
///
/// * `silent` (RMS < `LOGMEL_SILENCE_RMS`) — "quiet enough to ignore
///   for τ derivation and silent-silent pair skipping." ~-50 dB FS.
/// * `null`   (RMS < `LOGMEL_NULL_RMS`)   — "no signal at all." Only
///   the CMN short-circuit in `score_logmel` consults this.
#[derive(Clone, Copy, Debug)]
pub struct FrameFlags {
    pub silent: bool,
    pub null: bool,
}

/// Compute a log-mel spectrogram over the fixed STFT grid (hop =
/// `audio_rate/60`, window = `LOGMEL_FFT_SIZE`). Also returns a parallel
/// `FrameFlags` per analysis frame — `silent` feeds τ derivation and
/// silent-silent pair exclusion; `null` feeds the CMN short-circuit in
/// `score_logmel`.
///
/// Short buffers produce fewer analysis frames. The last partial window
/// is dropped rather than zero-padded to avoid biasing the distribution
/// toward quiet values at the tail.
pub fn log_mel_spectrogram(
    ctx: &LogMelContext,
    mono_i16: &[i16],
) -> (Vec<LogMelFrame>, Vec<FrameFlags>) {
    let samples: Vec<f64> = mono_i16.iter().map(|&s| f64::from(s)).collect();

    let hop = ctx.hop;
    let win = LOGMEL_FFT_SIZE;
    if samples.len() < win {
        return (Vec::new(), Vec::new());
    }
    let n_frames = 1 + (samples.len() - win) / hop;
    let mut spec = Vec::with_capacity(n_frames);
    let mut flags = Vec::with_capacity(n_frames);
    let mut fft_buf: Vec<Complex<f64>> = Vec::with_capacity(win);
    for f in 0..n_frames {
        let start = f * hop;
        let slice = &samples[start..start + win];
        let rms = rms_slice(slice);
        flags.push(FrameFlags {
            silent: rms < LOGMEL_SILENCE_RMS,
            null: rms < LOGMEL_NULL_RMS,
        });
        spec.push(compute_logmel_one(ctx, slice, &mut fft_buf));
    }
    (spec, flags)
}

/// L1 distance between two log-mel vectors, after subtracting each
/// vector's own mean (cepstral mean normalization). Gain-invariant: a
/// constant log-amplitude offset on either side cancels via the mean.
pub fn l1_diff(a: &LogMelFrame, b: &LogMelFrame) -> f64 {
    let n = LOGMEL_N_MEL as f64;
    let a_mean: f64 = a.iter().sum::<f64>() / n;
    let b_mean: f64 = b.iter().sum::<f64>() / n;
    let mean_delta = a_mean - b_mean;
    let mut s = 0.0;
    for (x, y) in a.iter().zip(b.iter()) {
        s += ((x - y) - mean_delta).abs();
    }
    s
}

/// Adjacent-frame L1 diffs of a reference spectrogram, with
/// silent-silent pairs excluded. Feeds the per-replay τ derivation.
pub fn adjacent_diffs_active(spec: &[LogMelFrame], flags: &[FrameFlags]) -> Vec<f64> {
    if spec.len() < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(spec.len() - 1);
    for n in 1..spec.len() {
        if flags[n].silent && flags[n - 1].silent {
            continue;
        }
        out.push(l1_diff(&spec[n - 1], &spec[n]));
    }
    out
}

/// Linear-interpolated quantile of a slice. Non-destructive — sorts a
/// copy internally.
fn quantile(values: &[f64], q: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let mut s: Vec<f64> = values.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = q * (s.len() as f64 - 1.0);
    let lo = idx.floor() as usize;
    let hi = idx.ceil() as usize;
    if lo == hi {
        s[lo]
    } else {
        let t = idx - lo as f64;
        s[lo] * (1.0 - t) + s[hi] * t
    }
}

/// Per-replay τ: `AUDIO_QUANTILE`-th quantile of the reference's
/// adjacent-frame active-pair L1 diffs. Returns 0.0 if the reference
/// has no active pairs (all silence).
pub fn audio_threshold(spec: &[LogMelFrame], flags: &[FrameFlags]) -> f64 {
    let diffs = adjacent_diffs_active(spec, flags);
    if diffs.is_empty() {
        0.0
    } else {
        quantile(&diffs, AUDIO_QUANTILE).max(AUDIO_DELTA_FLOOR)
    }
}

/// Sigmoid per-frame audio score. Mirrors `CompareResult::frame_score`.
/// `τ ≤ 0` (the all-silent-reference case) returns 1.0 for zero diff
/// and 0.0 otherwise — callers shouldn't invoke the scorer at all in
/// that case, but this keeps the function total.
pub fn frame_score_audio(diff: f64, tau: f64) -> f64 {
    if tau <= 0.0 {
        return if diff <= 0.0 { 1.0 } else { 0.0 };
    }
    let r = diff / tau;
    1.0 / (1.0 + r.powi(AUDIO_SHARPNESS))
}

/// Result of `score_logmel_detailed`: the scalar mean score plus the
/// per-analysis-frame breakdown.
///
/// `per_frame[i]` is `None` when both sides were silent at analysis
/// frame `i` (excluded from the mean) and `Some(score)` otherwise —
/// including hard-0 values for null mismatches. Length equals the
/// reference spectrogram's frame count, which is ~`n_emulated_frames`
/// but not exactly equal (STFT windowing).
#[derive(Debug, Clone)]
pub struct AudioScore {
    pub mean: f64,
    pub per_frame: Vec<Option<f64>>,
}

/// End-to-end audio score for a (ref, cand) pair against a precomputed
/// reference threshold τ. Convenience wrapper returning just the scalar
/// — callers that also want the per-frame series use
/// `score_logmel_detailed`.
///
/// Returns `None` if the reference has no non-silent analysis frames
/// (nothing meaningful to grade — same semantics as before).
pub fn score_logmel(
    ref_buf: &[i16],
    cand_buf: &[i16],
    audio_rate: u32,
    tau: f64,
) -> Option<f64> {
    score_logmel_detailed(ref_buf, cand_buf, audio_rate, tau).map(|s| s.mean)
}

/// Per-frame audio score series plus the mean. Same scoring logic as
/// `score_logmel`, but emits the full breakdown so consumers can render
/// a per-frame penalty timeline that agrees with the scalar byte-for-byte.
pub fn score_logmel_detailed(
    ref_buf: &[i16],
    cand_buf: &[i16],
    audio_rate: u32,
    tau: f64,
) -> Option<AudioScore> {
    let ctx = LogMelContext::new(audio_rate);
    let (ref_spec, ref_flags) = log_mel_spectrogram(&ctx, ref_buf);
    if ref_spec.is_empty() || ref_flags.iter().all(|f| f.silent) {
        return None;
    }
    let (cand_spec, cand_flags) = log_mel_spectrogram(&ctx, cand_buf);
    // If the candidate produced fewer analysis frames (ran short), or
    // any frame is truly null (~zero i16 RMS), the log-mel vector is
    // pure floor — no spectral shape to compare. Those frames take a
    // hard 0 via the null check below; floor_frame is still used so
    // missing cand frames line up positionally, but its l1_diff result
    // is never consulted for null cases.
    let floor_frame: LogMelFrame = [LOGMEL_EPS.ln(); LOGMEL_N_MEL];

    let n = ref_spec.len();
    let mut per_frame: Vec<Option<f64>> = Vec::with_capacity(n);
    let mut sum = 0.0;
    let mut count = 0u32;
    for i in 0..n {
        let rf = ref_flags[i];
        let cf = cand_flags.get(i).copied().unwrap_or(FrameFlags { silent: true, null: true });
        if rf.silent && cf.silent {
            per_frame.push(None);
            continue;
        }
        // Hard-0 when exactly one side produced no signal at all. CMN
        // is a shape comparison and there is no shape in a null frame;
        // short-circuit so silence-vs-loud reads as the miss it is.
        // "quiet but non-null" (cand at −60 dB with real content) takes
        // the CMN path — that's the whole point of CMN.
        if rf.null != cf.null {
            per_frame.push(Some(0.0));
            count += 1;
            continue;
        }
        let cand_frame = cand_spec.get(i).unwrap_or(&floor_frame);
        let d = l1_diff(&ref_spec[i], cand_frame);
        let s = frame_score_audio(d, tau);
        per_frame.push(Some(s));
        sum += s;
        count += 1;
    }
    if count == 0 {
        None
    } else {
        Some(AudioScore { mean: sum / count as f64, per_frame })
    }
}

/// Convenience: build context + spectrogram + τ in one call, for
/// callers (e.g. refcache precompute) that only have the reference.
pub fn derive_threshold(ref_buf: &[i16], audio_rate: u32) -> f64 {
    let ctx = LogMelContext::new(audio_rate);
    let (spec, flags) = log_mel_spectrogram(&ctx, ref_buf);
    audio_threshold(&spec, &flags)
}
