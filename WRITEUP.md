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

The one model result so far is a small, free model on a short run; the
last section covers what it shows and what it doesn't.

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
frames. A likely cause, not yet confirmed: RustyNES ignores PPU register
writes for the first 29,658 CPU cycles after power-on (the NES's
documented warm-up), and `nes-core` doesn't model that.
Both are legitimate behaviours a reasonable emulator could get "wrong"
relative to the reference, so both are now in the spec. I didn't change
`nes-core` to match, because a baseline tuned against the corpus stops
being a baseline. This is the main thing I learned from building it:
**an underspecified ABI looks fine until an independent implementation
hits it.**

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
is the interesting part. In two games it stays frame-exact (0.997 and
0.999). In the other four, a one-frame difference at boot means every
recorded button press lands at a slightly different moment, and the
playthrough diverges for good: 95 to 97% of frames differ.

That is what replay grading measures. It doesn't ask whether the
emulator looks right; it asks whether it behaves identically, and small
timing errors compound. It's also a warning about difficulty: the score
is sensitive to exact timing, which is what keeps frontier models from
all scoring 0.99, but it can also make the scores swing on a single
boot-time detail.

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
  when its process exits, never noticed. The supervisor should also
  watch for silence.
- **The score is harsh on near-misses, by design.** Replay grading asks
  whether the emulator behaves identically, and a structural metric
  gives no credit for "the right tiles, flipped". That is the right call
  for a benchmark about exactness, but it means early-stage emulators
  bunch near zero; checkpoint scores (0.0578 at one hour) say little
  about how close a model is to a breakthrough.

## What's next

The benchmark runs end to end: one model has been through the full
pipeline, from task container to published attempt. The open questions
need stronger models and full-length runs:
- **Difficulty.** An NES emulator is much smaller than a GBA emulator. If
  every frontier model reaches 0.95, the benchmark can't tell them apart,
  and the fix is harder tests or shorter runs.
- **Memorisation.** NES emulators are everywhere in training data. A high
  score might mean engineering, or recall. Comparing the same model on GBA
  Eval and NES Eval, and checking the code for copied structure, would
  separate the two.
- **Gaming.** GBA Eval saw a model try to game its grader. NES Eval's
  held-out games aren't visible to the agent, but the agent can query the
  reference as a black box for any ROM it has, which is worth watching.

Code, corpus and environment: <https://github.com/IlyaasK/nes-eval>.
