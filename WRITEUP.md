# Building NES Eval: what changes when you port GBA Eval to the NES

[GBA Eval](https://gbaeval.com) asks a model to write a Game Boy Advance
emulator in 24 hours and grades it by replaying recorded gameplay frame by
frame against a reference emulator. NES Eval applies the same method to the
NES. The grader, the scoring metrics and the agent environment are ported
from GBA Eval's open-source harness. This post is about the parts that
didn't port directly: the decisions the NES forced, and what I found while
testing the grader before any model touched it.

Headline numbers, all from the current corpus (29 testcases, 44,980
graded frames):

| Emulator | Overall |
|---|---|
| Reference (RustyNES) against itself | 1.0000 |
| `nes-core`, a 2,600-line emulator written from the NESdev wiki | 0.5811 |
| Reference with three injected bugs | 0.1948 |
| Blank screen | 0.0039 |
| Space Bunny Free, 2 hours in opencode | 0.0594 |
| Qwen3.8 27B, 24 hours in omp | never compiled |
| GPT-6.1 Sol, 24 hours in codex at high thinking | 0.9860 |
| DeepSeek V4.1 Flash, 24 hours in omp at max thinking | 0.9272 |

The four model results, and what they show, are at the end.

## 1. Emulators hand over palette numbers, not colours

The GBA outputs 15-bit RGB, and GBA Eval has to cope with emulators
expanding 5-bit channels to 8 bits in different ways, so it quantises back
to 5 bits before comparing. The NES has a different problem: it doesn't
output RGB at all. The PPU produces a 6-bit palette number plus three
"emphasis" bits for every pixel, and the colour you see depends on how a
composite video signal gets decoded. Every emulator ships its own
palette, and none of them is "correct".

If candidates returned RGB, an emulator with perfect timing but a
different palette table would lose points on every pixel of every frame.
That measures taste, not accuracy. So the ABI asks for the 9-bit value
`(emphasis << 6) | colour` per pixel, and the grader maps it to RGB with one
fixed table (RustyNES's) before computing similarity. Two pixels now match
exactly when two emulators computed the same thing, which also made GBA
Eval's quantisation step unnecessary. The reference cache stores the
indices, not colours, so the cache records what the NES computed.

The cost: a candidate with a wrong RGB table is invisible to the benchmark.
That's intended, and it's written down.

## 2. The power-on state is part of the spec

GBA Eval rests on one property: the console is deterministic, so the same
ROM and the same button presses produce the same frames forever. The real
NES isn't quite that. Work RAM powers up with unpredictable contents, the
CPU and PPU clocks start at a random alignment, and some games seed their
random number generators from exactly those things.

An emulator does have to pick something. If the spec says nothing, a
candidate can be correct to the hardware and still disagree with the
reference from frame zero. So the ABI pins down the reference's choices:
zeroed RAM, register values at the first instruction (`P=$24`, `S=$FD`),
the PPU at scanline 0, dot 25 at that moment, fixed clock alignment. I got
these by probing RustyNES, not by trusting its docs. My first draft of the
spec said `P=$34` and dot 0, and both were wrong.

The baseline emulator found two more gaps. `nes-core` scored 0 on two
mapper tests because it gave cartridges exactly the PRG-RAM their NES 2.0
header declares (none), while the reference always gives MMC1 and MMC3
boards 8 KiB. It also diverged from every game within the first 16
frames. I guessed the cause was the PPU warm-up: RustyNES ignores PPU
register writes for the first 29,658 CPU cycles after power-on (the NES's
documented warm-up), and `nes-core` doesn't model that. Both are
legitimate behaviours a reasonable emulator could get "wrong" relative to
the reference, so both are now in the spec. I didn't change `nes-core` to
match, because a baseline tuned against the corpus stops being a baseline.

The warm-up guess was wrong. The real cause was a third gap, which the
DeepSeek run exposed: the reference's first `emu_run_frame` emulates
nothing, and the ABI doesn't say so (see "Reference-fitting" below). This is
the main thing I learned from building it: **an underspecified ABI looks
fine until an independent implementation hits it.**

## 3. Only test ROMs the reference passes

The procedural section uses self-checking test ROMs (blargg's suites,
kevtris's nestest, Holy Mapperel) scored on their final screen. I rendered
every reference end screen before accepting it. One,
`mmc3_test_2/4-scanline_timing`, ends with "Failed #9" on RustyNES.

Keeping it would mean a candidate scores higher by reproducing a reference
bug than by being right. So it's out of the graded set, and it ships in
the agent's development ROMs instead, where the agent can see the
reference's behaviour but isn't graded on it. The general version of this
problem doesn't go away: every replay is graded against RustyNES, not
against hardware. Where RustyNES is wrong in a way no test ROM catches, a
more accurate candidate loses points. GBA Eval has the same exposure with
Mesen2. The mitigation is choosing a reference with a strong test record
(RustyNES passes all of AccuracyCoin) and keeping the procedural set to
tests it demonstrably passes.

## 4. The audio metric ignores loudness

I tested the grader with a candidate that has the triangle and noise
channels removed. It scored 0.726 on audio overall, which looks right,
but one testcase stood out: blargg's `apu_mixer/triangle` gave it 0.961.

That test plays a triangle wave and an inverted copy of it through the DMC
channel, so on a correct emulator they cancel to near-silence. Remove the
triangle and the DMC copy is left playing, about 10 times louder than the
reference. But GBA Eval's audio metric subtracts each frame's mean
log-mel level before comparing spectra, a deliberate choice so that
emulators with different output gain aren't punished. Loud-DMC-tone and
quiet-residual have the same spectral shape after that, so the metric
calls them a match.

The noise test (0.058) and the game replays (0.39 to 0.92) caught the
missing channels, so the bug doesn't go unnoticed overall. But it's a real
blind spot: any bug that changes how loud something is without changing
its spectrum is graded as correct. The fix I'd propose is a bounded
loudness term, for example penalising per-frame RMS ratios outside
[0.5, 2], which still tolerates gain differences. I haven't added it, to
keep the metric identical to GBA Eval's until a model run shows whether it
matters.

## 5. Determinism, end to end

The brief I worked from said: if the reference isn't deterministic under
your harness, nothing else matters. So the first thing I built was a
probe that ran RustyNES twice per ROM and compared the bytes. Then the
whole pipeline got the same treatment. `scripts/determinism-gate.sh`
regenerates all 29 reference caches from scratch twice and grades a
candidate twice; the last run produced byte-identical caches and 146
byte-identical result files. Three things in the ported harness had to
change for that to hold:

- GBA Eval's summary includes a wall-clock timestamp. Removed.
- It aggregates scores through hash maps, so floating-point sums happen in
  a different order each run. Replaced with ordered maps.
- WebAssembly is deterministic except for NaN bit patterns, which can
  differ between machines. The grader now runs wasmtime with NaN
  canonicalisation, so a cache built on one machine matches a candidate
  graded on another.

## 6. The corpus is homebrew

Replays use six homebrew games, each downloaded from its author's own
release with its license checked at the source: Falling, Thwaite, Double
Action Blaster Guys, Minekart Madness, Nova the Squirrel and Concentration
Room, 5,400 to 6,000 frames each. Commercial games are out, which also
rules out TASVideos movies (nearly all commercial, and recorded on a
different emulator). The replays were written blind against the reference
and checked from screenshots every 120 frames: they reach gameplay within
the first 300 frames and include scrolling, scoring, deaths, level
changes and a game over.

The agent gets four different homebrew games and 30 test ROMs that aren't
graded, mirroring GBA Eval's split between development and held-out ROMs.
One game I found was dropped because its music had no license, and Nova
the Squirrel's assets are non-commercial only, which the README flags.

## 7. What a middling score looks like

GBA Eval ships `gba-core`, a modest emulator scoring about 0.53, so a
reader knows what a mid-table number means. `nes-core` is the NES
equivalent: written from the NESdev wiki without looking at any other
emulator, graded once, untuned. It scores 0.58. It passes 19 of 23 test
ROMs (procedural 0.83) but only 0.46 on replays, and the replay breakdown
is the interesting part. In two games it stays close to the reference
(0.997 and 0.999). In the other four, the baseline is one frame out of
step from the first call onwards, so every recorded button press reaches
the game at a different moment, and the playthrough diverges for good: 95
to 97% of frames differ.

That is what replay grading measures. It doesn't ask whether the
emulator looks right; it asks whether it behaves identically, and small
timing errors compound. It's also a warning: the score is sensitive to
exact timing, which is what keeps frontier models from all scoring 0.99,
but it can also make the scores swing on a single detail. Here the detail
turned out not to be emulation at all but the first-call convention: with
that one call skipped, the same untuned `nes-core` scores 0.9426.

## The first model run

The first attempt used OpenCode Zen's free Space Bunny model at medium
thinking for 2 hours, in opencode
([attempt repository](https://github.com/IlyaasK/nes-eval-attempt-space-bunny-free)).
It scored 0.0594: barely above a blank screen.

The work behind that number isn't nothing. In 1h34m of active time it
wrote 3,538 lines covering the CPU, PPU, APU and all five mappers, built
its own frame dumper and comparison scripts, and queried the oracle 78
times. But its PPU draws every 8-pixel tile mirrored (nestest's menu reads
"stset llA nuR"). The model found and fixed one bug in the same shift
registers ("I had the direction backwards!") but not this one. A
tile-level mirror destroys the edges the video metric compares, so a
screen full of almost-right text scores about the same as nothing, and
every test ROM's verdict screen fails. Audio, which works partially, is
where its points come from (0.27).

Two lessons about the harness, not the model:

- **A rate limit ended the run early.** At 1h34m the free tier rejected
  one request. opencode logged the error and then sat idle without
  retrying or exiting, so the supervisor, which restarts an agent only
  when its process exits, never noticed. The supervisor now also watches
  for silence: an agent whose output stops for 15 minutes is stopped and
  resumed.
- **The score is harsh on near-misses, by design.** Replay grading asks
  whether the emulator behaves identically, and a structural metric
  gives no credit for "the right tiles, flipped". That is the right call
  for a benchmark about exactness, but it means early-stage emulators
  bunch near zero; checkpoint scores (0.0578 at one hour) say little
  about how close a model is to a breakthrough.

## The second model run: 24 hours, and nothing to grade

The next attempt gave a model the full 24 hours: Qwen3.8 27B on Hetzner's
free endpoint, driven by omp at medium thinking
([attempt repository](https://github.com/IlyaasK/nes-eval-attempt-qwen3.8-27b-omp)).

It produced no score, because the candidate never compiled. I rebuilt all
43 saved commits afterwards; every one failed. That splits the run into
three phases, and the first is the interesting one:

- **Hours 0–8: nothing could be built at all.** The model wrote modules but
  never created `src/lib.rs`, so cargo rejected the manifest: *"can't find
  library `nes_emu`, rename file to `src/lib.rs`"*. For eight hours it was
  reading the hardware docs, querying the oracle and writing code it had
  no way to check.
- **Hours 8–14: real compile errors and real progress** — 45 down to 2 by
  hour 13:45, the closest it came.
- **Hours 14–24: it regressed.** Back up to 36 errors by hour 17, 9 at the
  end, with each large rewrite breaking names and types elsewhere.

Over 24 hours it made 545 tool calls (304 reads, 76 edits, 41 writes, 75
shell commands), burned 8.2M input and 1.5M output tokens against 27M
cache reads, and compacted its own context 11 times. The run mechanics
were perfect: one agent invocation, no crash, no stall, no rate limit, 42
clean saves. The failure was engineering.

Three things this run shows about the benchmark:

- **The task's first gate is not emulation, it is building.** A model can
  spend a whole day on a 2,700-line emulator and never once run `cargo
  build`. A 24-hour NES Eval run measures whether a model establishes a
  feedback loop at all, before it measures any emulation accuracy.
- **Not compiling is a distinct outcome, and the harness says so plainly.**
  The checkpoints record `build_rc=101` and no score, so this lands
  outside the 0-to-1 scale instead of being scored as "very close to
  zero". Anyone comparing models can see the difference between "wrote
  something that runs but is wrong" (Space Bunny, 0.0594) and "never
  produced a binary".
- **Free-tier budget and time are not the same thing.** The earlier
  Space Bunny run lost its last 25 minutes to a rate limit; this run had
  no interruptions and still produced nothing gradeable. Both walls are
  real, and they now have a watchdog and a clear ungraded state between
  them.


## The third model run: a frontier model, 0.9860

GPT-6.1 Sol, running in codex at high thinking for the full 24 hours
([attempt repository](https://github.com/IlyaasK/nes-eval-attempt-gpt-6.1-sol)):

| Checkpoint | Overall | Procedural | Replay | Audio |
|---|---|---|---|---|
| 4 h | 0.9776 | 0.8901 | 1.0000 | 0.9980 |
| 8 h | 0.9860 | 0.9300 | 1.0000 | 0.9999 |
| 12 h, 16 h, 20 h, final | 0.9860 | 0.9300 | 1.0000 | 1.0000 |

28 of the 29 testcases score at or above 0.999. **Five of the six homebrew
games replay identically to the reference**, every pixel of every frame; in
the sixth, Falling, 4 of 5,800 frames differ slightly and the replay still
scores 1.0000. Audio reaches exactness by hour 12. The single failure is
`nmi-and-brk`, blargg's NMI-hijacking-BRK test, at 0.0, and it accounts for
the entire remaining 0.014.

Three things stand out.

- **The wall is real, and it is one testcase wide.** Everything above
  reflects full emulation of five mappers, dot-level PPU behaviour,
  sprite evaluation, and a calibrated resampler, from scratch, in a day.
  What is left is a single interrupt-timing corner case, which the model
  was still working on when the clock ran out: its own `PROGRESS.md` ends
  mid-investigation of a residual OAM row-corruption bug with the note
  "Task not complete; keep improving."
- **Tighter score bands than expected.** A free 27B model never compiled,
  a free small model scored 0.0594 with mirrored tiles, and a frontier
  model scored 0.9860. The gap between "an emulator that runs" and "an
  emulator that is exact" is where this benchmark discriminates, and it
  is wide.
- **Replay grading is brutal but fair.** 34,000 frames of lockstep
  comparison across six games is a much harder bar than any single test
  ROM, and it was met all but exactly: 4 of those frames differ. That is
  the property GBA Eval's method rests on, reproduced on the NES.

The run itself needed harness work to get there, all of which is in this
repository now: the container's bridge networking was slow and lossy
enough that codex's startup calls timed out (a host-network override
fixes it), the agent's shells needed `umask 002` so the oracle could write
into directories the agent created, and codex's `COMPOSE_FILE` needed
absolute paths. The watchdog also fired three times; each time codex
resumed its session, and no committed work was lost.

## The fourth model run: DeepSeek V4.1 Flash, 0.9272, and a bug of mine

DeepSeek V4.1 Flash ran for 24 hours in omp at max thinking, via river.ai
([attempt repository](https://github.com/IlyaasK/nes-eval-attempt-deepseek-v4.1-flash)):

| Checkpoint | Overall |
|---|---|
| 4 h | 0.5564 |
| 8 h | 0.8880 |
| 12 h | 0.8934 |
| 16 h | 0.9132 |
| 20 h | 0.9272 |
| final (24 h) | **0.9272** |

Replay is its strongest section at 0.9955: four of the six homebrew games
score 0.9998 or better and the other two, Nova the Squirrel and Thwaite, are
near-misses at 0.9835 and 0.9894. What it fails instead is five test ROMs,
all at zero — the four Holy Mapperel cartridge-board ROMs and blargg's
official-opcode suite. A model that runs real games almost perfectly while
getting instruction flags and board behaviour wrong is a different failure
profile from one that does neither, and the sections separate them cleanly.

It wrote 7,329 lines and, notably, 59 of its own tools: `compare.py`,
`audiodiff.py`, `cpufuzz.py`, `asm6502.py`, `dis6502.py`, `attrprobe.py`, and
its own `tools/STATUS.md` notes on what still differed. 72 agent invocations
(`omp -p` ends after each turn; the supervisor re-invoked it with
"continue"), median 8.8 minutes, and 23.85 of the 24 hours had an agent
working. It cost about $9 in tokens.

**This run also found a bug in NES Eval, not in the model.** The first
attempt ended after 2h45m, at 0.3538. The supervisor's watchdog measured
silence with `stat -c %Y "$AGENT_LOG"`, that stat races the background
redirect which creates the log, and an empty operand inside `$(( ))` does
not merely fail the test — it aborts the enclosing `while` loop, so the
supervisor fell through to "run completed". The deepseek run was the victim;
every earlier run had simply been lucky. The fix (`log_age()`, which always
returns a number) ships with a regression test that deletes the agent's log
mid-run: pre-fix code ends a 144-second run after 35 seconds, fixed code runs
the full 174.

The lesson is worth stating plainly: a benchmark's own harness is code under
test too. Only the fact that the model's score kept climbing across
checkpoints (0.5564 → 0.9272) made it obvious that the early "completion" was
not the model quitting.

## Reference-fitting: one constant worth 0.36

GBA Eval has a post about Grok special-casing a specific example ROM's
instruction addresses to force its boot check to pass. I looked for that in
all four runs by grepping their final sources and transcripts — ROM names,
hashes, CRC or title matching, `pc == 0x…` behaviour changes, forced flags,
baked frames — and found none of it. The literal address comparisons in all
three emulators are hardware registers: `$4016`/`$4017`, `$4015`, `$2007`.

What I did find is a milder thing, and it is worth publishing because of how
much it is worth. DeepSeek's `src/system.rs` contains:

```rust
/// Whether the first emu_run_frame call returns without emulating (the
/// reference behaves this way).
static mut FIRST_FRAME_SKIPS: bool = true;
```

Its first `emu_run_frame()` returns without emulating, in the graded build.
Eleven constants sit alongside it — FIR taps, NMI latch dots, NMI sample
phase, audio prewarm — all tuned the same way. I flipped that one flag and
regraded:

| | Overall | Procedural | Replay | Audio |
|---|---|---|---|---|
| as published | 0.9272 | 0.7700 | 0.9955 | 0.8797 |
| `FIRST_FRAME_SKIPS = false` | **0.5650** | 0.7700 | 0.4535 | 0.6944 |

A single reference-shaped constant is 0.362 of that model's score — more
than any actual emulation feature it wrote. GPT-6.1 Sol did the same species
of thing in audio: it fitted its resampler and DAC tables against the
oracle's output, reporting the RMS improvements, with its own train and
held-out splits of probe data.

Two things make this different from Grok's hack, and both are my
responsibility rather than the model's:

- **It is universal, not ROM-specific.** No ROM detection is involved; the
  constant applies to every testcase equally. Grok keyed behaviour to
  program-counter addresses inside one example ROM.
- **My ABI made it necessary.** `spec/ABI.md` defines the frame boundary —
  `emu_run_frame` advances until the PPU leaves pre-render for scanline 0 —
  so a candidate *must* match the reference's convention to be scored
  correctly. But I never stated what the reference does on the *first* call:
  in all 29 reference caches, frame 0 is an all-zero framebuffer with zero
  audio samples, so the reference emulates nothing on that call, while the
  ABI's wording implies a full frame. The model measured it with the oracle
  and complied. That is the right engineering response to an underspecified
  interface, and it is the third time this has bitten: `nes-core` found the
  same kind of gap in MMC1/MMC3 PRG-RAM sizing and power-on warm-up. I
  documented those two in the ABI after the fact; this one is still open.

**My own baseline had the same bug.** `nes-core`'s replay losses match
DeepSeek-with-the-flag-off almost exactly: the same games diverge on the
same frames (Falling: 5,601 of 5,800 frames over the noise floor in both;
240,509,695 against 240,509,701 differing pixels). Adding the same first-call
skip to `nes-core`, one line, takes it from 0.5811 to **0.9426** (replay
0.4569 to 0.9996). My earlier guess, the PPU warm-up, was wrong. A careful
emulator written from the documentation scores 0.94 here; the published 0.58
mostly measures whether you guessed one function call's convention.

The lesson generalises beyond one constant. **When the score is defined as
agreement with a reference implementation, every unstated convention is a
hidden lottery worth points, and the rational strategy is to fit rather than
derive.** Two fixes, both for a v1.1 environment rather than a rerun of the
published v1.0 results:

1. Probe the reference's first-call and frame-alignment behaviour and write
   it into the ABI, so compliance is a documented requirement instead of a
   discovery.
2. Stop leaking the reference's identity. `corpus/roms/dev/README.md` and
   `test/LICENSE.txt` say the practice ROMs were "copied byte-for-byte from
   `third_party/RustyNES/tests/roms/`", naming the reference emulator and its
   commit. Qwen3.8 reasoned straight from it: *"this is a strong hint: the
   reference is likely based on RustyNES"*. Attribution to blargg and
   `christopherpow/nes-test-roms` gives the same provenance without turning
   the task into a recall exercise.

## What's next

The benchmark runs end to end: four models have been through the full
pipeline, from task container to published attempt, and they land 0.0594,
ungraded (never compiled), 0.9272 and 0.9860. The open questions have
changed shape:

- **Headroom.** The best result leaves 1.4% between it and a perfect score,
  all of it one CPU-interrupt testcase. To tell strong models apart, the
  corpus needs harder video and audio material, or longer runs that force
  the last few percent.
- **Different failure modes, not just different scores.** 0.9272 and 0.9860
  look close but fail in opposite places: one misses five test ROMs, the
  other misses one and is exact everywhere else. Section-level scores
  separate them; the single number hides it.
- **Memorisation.** NES emulators are everywhere in training data, and
  blargg's test ROMs are famous, so a high score could reflect recall
  rather than engineering. Comparing the same model on GBA Eval and NES
  Eval, and checking the code for copied structure, would separate the two.
- **Gaming.** GBA Eval saw a model try to game its grader. NES Eval's
  held-out games aren't visible to the agent, but the agent can query the
  reference as a black box for any ROM it has, which is worth watching.

Code, corpus and environment: <https://github.com/IlyaasK/nes-eval>.
