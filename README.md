# NES Eval

A deterministic benchmark for NES emulators, modelled on
[GBA Eval](https://github.com/mechanize-work/gba-eval). A candidate emulator
compiled to WebAssembly is run in lockstep with a reference emulator
([RustyNES](https://github.com/doublegate/RustyNES)) on test ROMs and
recorded gameplay. Every frame's picture and sound is compared, and the
result is a score from 0 to 1.

The repository contains the grader and reference, the corpus, the
environment an AI agent works in to build an emulator (`quickstart/`), and
a baseline emulator (`candidates/nes-core`).

## Quick start

```sh
git clone --recurse-submodules https://github.com/IlyaasK/nes-eval && cd nes-eval
git lfs pull                                # corpus/reference-cache/ (~70 MB)
rustup toolchain install 1.96.0 --target wasm32-unknown-unknown   # pinned in rust-toolchain.toml

cargo build --release -p grader
target/release/grader candidates/nes-core/nes_core.wasm corpus/ out/nes-core
```

`scripts/build-wasm.sh` rebuilds the reference and the broken candidates;
`target/release/grader --precompute corpus/` regenerates the reference
cache (~30 s) if it is missing or stale. Docker users can skip the Rust
toolchain: `quickstart/grade.sh <candidate.wasm> <name>` grades in a
container.

Grading writes `out/<dir>/summary.json`, plus for every testcase a
`<id>.json` with per-frame results, the final frame from each side
(`<id>.ref.png`, `<id>.cand.png`) and both audio tracks (`<id>.ref.wav`,
`<id>.cand.wav`). `--emit-video` adds reference / candidate / difference
MP4s (needs `ffmpeg`).

## Running an agent

[`quickstart/`](quickstart/README.md) is the environment a model works
in, ported from GBA Eval's: a task container with the Rust toolchain,
[`TASK.md`](quickstart/task/TASK.md), the ABI, an offline copy of the
relevant NESdev wiki pages (`spec/docs/`, public domain), and example ROMs
that are not in the graded corpus (`corpus/roms/dev/`). The reference
emulator runs in a separate sidecar container; the agent can query it as a
black box through the `oracle` command but never sees its code.
`quickstart/run-agent.sh` supervises a timed run (24 hours by default):
it restarts the agent if it stops, commits its work every 30 minutes, and
grades a checkpoint every 4 hours.

The task container needs internet access to reach the model's API.
GBA Eval's own runs added an egress allowlist; this public setup, like
GBA Eval's public quickstart, does not, so "no internet" is an
instruction in `TASK.md`, not something enforced.

## Results

| Candidate | Overall | Procedural | Replay | Audio |
|---|---|---|---|---|
| reference (RustyNES) | **1.0000** | 1.0000 | 1.0000 | 1.0000 |
| `nes-core` baseline (below) | **0.5811** | 0.8300 | 0.4569 | 0.7050 |
| `bug-scanline`: picture one scanline late | 0.3257 | 0.0000 | 0.2095 | 1.0000 |
| `bug-ab-swap`: A and B swapped | 0.5389 | 1.0000 | 0.3282 | 0.7100 |
| `bug-no-tri-noise`: no triangle or noise channel | 0.9452 | 1.0000 | 1.0000 | 0.7261 |
| `broken`: all three | **0.1948** | 0.0000 | 0.1500 | 0.5241 |
| blank-screen stub | 0.0039 | 0.0000 | 0.0065 | 0.0000 |

**`nes-core`** ([`candidates/nes-core`](candidates/nes-core/README.md)) is
the counterpart of GBA Eval's `gba-core` (which scores about 0.53 there):
an original, MIT-licensed, 2,600-line emulator written from the NESdev
wiki without looking at any other emulator, and graded once without
tuning. It passes 19 of the 23 test ROMs. Its replay scores show how the
benchmark treats a decent emulator. Every game differs from the reference
within the first 16 frames, at boot. In Minekart Madness and Nova the
Squirrel that early difference washes out (88 and 2 differing frames of
5,400; scores 0.997 and 0.999). In the other four games the run never
recovers: once the game state is a frame off, the replayed inputs land at
different moments and the playthrough diverges (95–97% of frames
differ), the way a small timing error derails a tool-assisted speedrun.

Each injected bug lands where it should. A one-line video offset fails
every self-checking ROM's final screen and most replay frames, but leaves
audio untouched. Swapped buttons can't affect test ROMs that need no
A or B, but games that use them play out differently, so replay and audio
both drop; Falling, steered with the D-pad only, still scores 1.0. Missing
channels cost only audio. Per-testcase results are in each run's output
directory; for example `bug-ab-swap` first diverges from the reference on
Double Action Blaster Guys at frame 91, one frame after the replay first
presses A.

## How it works

1. **Reference cache** (`grader --precompute`). The reference wasm runs
   every testcase once. Every frame's picture and sound is stored in
   `corpus/reference-cache/<id>.refcache`, together with the score
   thresholds derived from it. Each cache records the hashes of the
   reference wasm, ROM and replay, and is regenerated when any of them
   changes.
2. **Lockstep run.** The candidate is loaded in wasmtime with every import
   trapping, a fuel limit per call, and NaN canonicalisation on. It gets the
   same ROM and the same button presses, frame by frame.
3. **Scoring.** Per frame, picture and sound are compared with the cached
   reference (below), then aggregated by subsystem and section.

## Candidate ABI

[`spec/ABI.md`](spec/ABI.md). A `wasm32-unknown-unknown` module exporting
10 C functions: load a ROM, set controller 1, run one frame, and read the
framebuffer and audio. Compared with GBA Eval's ABI:

- **Framebuffer is palette indices, not RGB.** Each pixel is
  `(emphasis << 6) | colour`, 9 bits in a `u16`. The grader maps indices
  to RGB with one fixed table, [`spec/nes_palette.pal`](spec/nes_palette.pal)
  (RustyNES's 2C02 palette). NES emulators disagree about palettes, and a
  palette choice is not emulation accuracy, so it cannot cost points.
- **Power-on state is part of the spec.** On hardware, RAM contents and
  CPU/PPU alignment at power-on are random, and games read them. The ABI
  pins them to what the reference does (zeroed RAM, `P=$24`, PPU at
  scanline 0 dot 25 at the first instruction), so a correct emulator can
  match exactly.
- **Audio is mono** i16 at 44 100 Hz, like the NES.

## Scoring rubric

`overall = 0.20 × procedural + 0.60 × replay + 0.20 × audio`
(GBA Eval's weights; [`corpus/grader.yaml`](corpus/grader.yaml)).
Within a section, each subsystem is the mean of its testcases, and the
section is the weighted mean of its subsystems:

| Section | Subsystem weights |
|---|---|
| procedural | cpu 0.35, ppu 0.30, mapper 0.20, apu 0.15 |
| replay | gameplay 1.00 |
| audio | apu 0.50, game_audio 0.50 |

**Video**, per frame. Floored block SSIM on BT.601 luma over the 960 8×8
blocks of the frame. Each block's defect is `1 − SSIM`; defects under
0.15 are treated as invisible. The frame defect `d` is the mean over
blocks; it is forced to 1 when the candidate's frame is one flat colour
and the reference's isn't. The frame score is `1 / (1 + (d/τ)^4)`, where
`τ` is the 90th percentile of the reference's own frame-to-frame defects
on that testcase, clamped to [0.005, 0.35]. A testcase that moves a lot
tolerates more error than a static one.

- `replay` testcases score the mean over all frames.
- `procedural` testcases score only the final frame (`endstate`, with a
  fixed τ of 0.02), because a self-checking ROM's verdict is its final
  screen.

**Audio**: per 1/60 s step, the L1 distance between candidate and
reference log-mel spectra (40 bands, 80 Hz to 8 kHz, each frame
mean-subtracted so overall loudness doesn't count). Scored with the same
sigmoid and a τ derived the same way from the reference. Steps where both
sides are silent are skipped. A testcase contributes an audio score only
if it has an `audio_subsystem` and the reference isn't silent.

The video and audio metrics are GBA Eval's, ported. GBA Eval explains how
they arrived at them in the blog post "Iterating on the grading strategy".
NES-specific changes: 8×8 blocks (the NES frame divides exactly into 32×30
of them, on tile boundaries; 10×10 would leave a 6-pixel column unscored),
no 5-bit colour quantisation (indices make it unnecessary), and mono audio.

## Reference cache format

`corpus/reference-cache/<testcase>.refcache`: a zstd stream wrapping one
bincode-1 record (little-endian, fixed-width integers, `u64` length before
each string and vector). Fields in order:

| Field | Type | Meaning |
|---|---|---|
| `magic` | `[u8; 8]` | `NESREFC\0` |
| `version` | `u32` | 1 |
| `ref_wasm_sha256` | string | hex SHA-256 of the reference wasm |
| `rom_sha256` | string | hex SHA-256 of the ROM |
| `replay_sha256` | string | hex SHA-256 of the replay file (of `""` if none) |
| `frame_count` | `u32` | frames recorded |
| `audio_rate` | `u32` | 44 100 |
| `frames` | `Vec<u16>` | `frame_count × 61 440` palette indices, frame by frame, row by row |
| `audio_samples_per_frame` | `Vec<u32>` | mono samples produced in each frame |
| `audio` | `Vec<i16>` | all samples, concatenated |
| `frame_diff_threshold` | `f32` | video τ |
| `audio_diff_threshold` | `f32` | audio τ (0 if silent) |

Defined in [`harness/grader/src/ref_cache.rs`](harness/grader/src/ref_cache.rs).
The cache is committed through Git LFS; `--precompute` rebuilds it in
about 30 seconds.

## Corpus

[`corpus/testcases.json`](corpus/testcases.json) lists every testcase:
ROM (by SHA-256), replay, frame count, section and subsystem. Replays are
text, one `<frame> <keys_hex>` line per change in held buttons (bit 0 = A,
1 = B, 2 = Select, 3 = Start, 4 = Up, 5 = Down, 6 = Left, 7 = Right).

**Procedural** (23 self-checking ROMs, final screen scored):

| Subsystem | Testcases |
|---|---|
| cpu | nestest (Start runs it), instr_test-v5 official_only, cpu_timing_test6, instr_misc, cpu_interrupts_v2/2 |
| ppu | ppu_vbl_nmi/01 and /05, sprite_hit_tests/01 and /09, sprite_overflow_tests/1, oam_read |
| apu | apu_test/1, /3, /7; apu_mixer square, triangle, noise, dmc (these four also score audio) |
| mapper | mmc3_test_2/1; Holy Mapperel for MMC1, UxROM, CNROM and MMC3 |

The reference passes every one of these (each final screen shows
"Passed" or the expected result). `mmc3_test_2/4-scanline_timing` was
dropped because the reference itself fails it ("Failed #9"); including it
would reward copying a reference bug.

**Replay** (6 homebrew games, every frame scored, audio scored):

| Game | Author | License | Mapper | Frames | What the replay does |
|---|---|---|---|---|---|
| Falling | Jesse Williams | MIT | 0 | 5800 | Day run: coins, extra life, deaths, game over; then a Night run |
| Thwaite | Damian Yerrick | GPL-3.0 | 0 | 6000 | Two full waves of missile defence |
| Double Action Blaster Guys | NovaSquirrel | GPL-3.0 / Zlib | 0 | 5400 | Clears levels 1–2, plays into level 3 |
| Minekart Madness | Matt Hughson | MIT | 1 | 5400 | Rails, ramps, gems, two crashes |
| Nova the Squirrel | NovaSquirrel | GPL-3.0 code, CC-BY-NC-SA assets | 1 | 5400 | Level 1-1 through two doors, deaths, respawns |
| Concentration Room | Damian Yerrick | GPL-3.0 + ROM exception | 0 | 6000 | Solo round, beats the CPU 3–2, most of the 5×4 round |

Every ROM was downloaded from its author's own release or repository. Its
license text and source URLs are in `corpus/roms/homebrew/<game>.LICENSE.txt`.
Each replay was authored against the reference with `tools/nes-probe` and
checked frame by frame from screenshots; each file's header lists what
happens when.

All mappers used are 0, 1, 3 or 4 (2 appears in the procedural set).

**Non-commercial content.** Nova the Squirrel's assets (graphics, levels
and so on) are licensed CC-BY-NC-SA-4.0, so the corpus as shipped may only be
used non-commercially. To use NES Eval commercially, remove the
`nova-the-squirrel-gameplay` testcase from `corpus/testcases.json` and
delete its ROM and replay; everything else is permissively licensed or
GPL. The GPL ROMs (Thwaite, Double Action Blaster Guys, Nova's code) come
with their source at the tag named in each `.LICENSE.txt`.

## Determinism

[`scripts/determinism-gate.sh`](scripts/determinism-gate.sh) is the CI
gate. It regenerates every reference cache twice from scratch and requires
the files to be byte-identical, then grades a candidate twice and requires
every output file to be byte-identical. What makes that hold:

- RustyNES's core uses no clock, thread or OS randomness, and the ABI pins
  power-on state.
- wasm is deterministic except for NaN bit patterns; the grader turns on
  NaN canonicalisation.
- The grader never reads the clock (GBA Eval's `summary.json` timestamp is
  removed), and every map is a `BTreeMap`, so summing order and JSON key
  order are fixed. Testcases run in parallel, but each has its own state
  and results are combined in manifest order.
- `scripts/build-wasm.sh` produces the same wasm bytes on every build with
  the pinned toolchain.

## Known limits

- **Candidates are graded against RustyNES, not hardware.** Where RustyNES
  is wrong, a more accurate candidate loses points. That is why the
  procedural set only includes tests the reference passes. A model that
  has memorised RustyNES's source also inherits its exact behaviour; a
  candidate graded here should be checked for copied code separately.
- **Audio scoring ignores loudness.** Each spectrum is mean-subtracted
  before comparison (GBA Eval's choice), so a sound with the right shape
  but the wrong volume scores well; loudness is only reported, as
  `audio_rms_ratio`. The `bug-no-tri-noise` candidate shows the cost:
  blargg's `apu_mixer/triangle` cancels the triangle against an inverted
  DMC copy, so without the triangle the leftover DMC tone is about 10×
  louder than the reference's near-silence, but has the same spectral
  shape, and scores 0.961 on audio. The noise test (0.058) and the game
  replays (0.39 to 0.92) do catch the bug.
- **The palette is not tested.** Because candidates hand over indices, a
  wrong colour-to-RGB table is invisible. That is intended.
- The reference wasm's hash depends on its source line numbers (panic
  messages embed them), so any edit to `reference/src/lib.rs` invalidates
  every cache.

## Layout

| Path | What | License |
|---|---|---|
| `spec/ABI.md`, `spec/nes_palette.pal` | Candidate interface and palette | MIT |
| `spec/docs/` | Offline NESdev wiki pages given to the agent | Public domain (`spec/docs/SOURCES.md`) |
| `harness/lockstep`, `harness/grader`, `harness/oracle` | Comparison, grading, and the black-box oracle, forked from GBA Eval | MIT (`harness/LICENSE`) |
| `reference/` | RustyNES behind the ABI; builds the reference and broken candidates | GPL-3.0-or-later |
| `candidates/nes-core` | Baseline emulator, written from scratch | MIT |
| `candidates/broken` | Reference with injected bugs (built by `scripts/build-wasm.sh`) | GPL-3.0-or-later |
| `quickstart/` | Agent task container, oracle sidecar, grading and run scripts | MIT |
| `tools/nes-probe` | Runs the reference natively on a ROM + replay, prints screen changes, writes PNGs; used to author replays | GPL-3.0-or-later |
| `corpus/` | Manifest, replays, grader config, reference cache | MIT |
| `corpus/roms/` | Graded (`homebrew/`, `test/`) and agent-visible (`dev/`) ROMs | Each ROM's own license, next to it |
| `scripts/` | Wasm build and determinism gate | MIT |
| `third_party/RustyNES` | Reference emulator (submodule, commit `30e6b98`) | GPL-3.0-or-later |

The grader loads the GPL reference wasm at runtime across the ABI and
does not link it, the same arrangement GBA Eval uses with Mesen2.
