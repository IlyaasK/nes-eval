# NES Emulator Wasm ABI

Export these 10 C-ABI functions, plus `memory`, from a `wasm32-unknown-unknown` cdylib:

```c
int       emu_init(void);           // setup, returns 1/0
uint8_t*  emu_rom_buffer(void);     // ≥8 MiB, stable pointer
int       emu_load_rom(int len);    // parse iNES / NES 2.0 image + power on, returns 1/0
int       emu_reset(void);          // identical state to load_rom (cold boot, not the Reset button)
void      emu_set_keys(uint32_t k); // controller 1, active-high, latched
void      emu_run_frame(void);      // advance one frame
uint16_t* emu_framebuffer(void);    // 256×240 palette indices, see below
int16_t*  emu_audio_buffer(void);   // i16 mono
int       emu_audio_samples(void);  // samples since last call (drains)
int       emu_audio_rate(void);     // must return 44100
```

Key bits (the order the controller shifts them out of `$4016`):
0=A, 1=B, 2=Select, 3=Start, 4=Up, 5=Down, 6=Left, 7=Right.

## Framebuffer

One `uint16_t` per pixel, row-major, 256 wide × 240 tall (all 240 rendered
scanlines, no overscan cropping). Each value is

```
(emphasis << 6) | colour
```

where `colour` is the 6-bit value read from palette RAM for that pixel and
`emphasis` is PPUMASK bits 7–5 shifted down (bit 0 = red, bit 1 = green,
bit 2 = blue). Values above `0x1FF` are masked off. The grader converts
indices to RGB with the fixed table in `spec/nes_palette.pal` (512 entries ×
3 bytes, `R G B`, indexed by the same 9-bit value), so the choice of display
palette never affects the score.

## Frames

`emu_run_frame` advances until the PPU leaves the pre-render scanline (261)
for scanline 0. The framebuffer then holds scanlines 0–239 of the frame just
finished. On odd frames with rendering enabled, the pre-render line is one
dot shorter, as on hardware.

The first `emu_run_frame` after `emu_load_rom` or `emu_reset` emulates
nothing: it returns with an all-zero framebuffer, and `emu_audio_samples`
then reports 0. Every later call advances one frame as described above, so
the second call returns the console's first frame.

## Power-on state

`emu_load_rom` and `emu_reset` must produce exactly this state, which the
reference uses:

- NTSC 2A03 / 2C02 timing.
- CPU work RAM, PRG-RAM (battery or not), palette RAM, OAM and nametable
  VRAM all zero.
- CPU registers at the first instruction fetch: `A=X=Y=0`, `S=$FD`,
  `P=$24`, `PC` = the reset vector at `$FFFC`.
- At that same moment the PPU is at scanline 0, dot 25 (the reset sequence
  has run with the PPU clocking from scanline 0, dot 0).
- CPU/PPU clock alignment and APU cycle parity are fixed, not randomized.
- PPU warm-up (NES behaviour, not Famicom): for the first 29 658 CPU
  cycles after power-on, writes to `$2000`, `$2001`, `$2005` and `$2006`
  are ignored.

## Audio

Mono, signed 16-bit, 44 100 Hz, full scale at ±32767. Samples are produced
during `emu_run_frame`. `emu_audio_samples` returns how many samples have
been produced since its previous call, and `emu_audio_buffer` then points to
them.

The five channels are summed in analog on the console: nothing in the
hardware produces a 44 100 Hz stream, so the output rate, the band-limiting
resampling and the output filtering are convention. The rest of this section
specifies the reference's convention exactly. An emulator whose APU produces
the same per-CPU-cycle channel values — pulse 1 and pulse 2 (0–15), triangle
(0–15), noise (0–15), DMC (0–127); see "What the mixer combines" below for
exactly what those values are — reproduces the reference's `int16` stream
sample for sample, per-frame sample counts included, by running the stages
below in the order given and in the stated arithmetic. Unless stated otherwise
every value is `f32`, every multiply and add is rounded on its own (no fused
multiply-add, no reassociation), and the operation order written here is the
order to evaluate in. Expansion
(cartridge) audio is silent: it is zero for mappers 0–4, and nothing else
feeds this pipeline.

The APU advances exactly one cycle per CPU cycle from power-on onwards,
including the 8-cycle power-on reset sequence, so each of those cycles
contributes one mixer value below. (`phase`, in the resampler, is therefore
`8 * step` when the first `emu_run_frame` call is made.)

### What the mixer combines

The tables below are indexed by the channels' raw per-cycle outputs. The
channels' own gates are applied by the channels, not by the mixer, and this is
where emulators diverge most from the reference:

- **pulse 1 / pulse 2** — the envelope output (0–15), or 0 when the channel's
  length counter is zero, when the sweep unit is muting it (timer period < 8
  or sweep target period > `$7FF`), or on the half of the duty cycle whose
  table value is 0.
- **triangle** — the sequencer's current value (0–15; the sequence is
  `15, 14, …, 0, 0, 1, …, 15`), **not** gated by the length counter or the
  linear counter: those only stop the sequencer from advancing, and a stopped
  sequencer still drives the DAC. A fresh power-on leaves the sequencer on its
  first step, so the triangle term of the mixer is 15 (index contribution 45,
  value 0.2554771 — not 0) from the first cycle until something clocks the
  sequencer. An emulator that gates the triangle's *output* on its counters
  starts every stream from a different level.
- **noise** — the envelope output (0–15), or 0 when the length counter is zero
  or when the LFSR's bit 0 is set.
- **DMC** — the 7-bit DAC level (0–127) as written, including by a direct
  `$4011` load, whether or not the channel is enabled.

### Mixer

Once per CPU cycle, after the channels have been clocked, their outputs are
combined through two tables computed once at power-on:

```
pulse_table[0] = 0.0
pulse_table[i] = 95.52 / (8128.0 / i + 100.0)       1 <= i <= 30
tnd_table[0]   = 0.0
tnd_table[i]   = 163.67 / (24329.0 / i + 100.0)     1 <= i <= 202

m = pulse_table[pulse1 + pulse2] + tnd_table[3*triangle + 2*noise + dmc]
```

`i` is the table index converted to `f32`; `m` lies in `[0.0, 1.0)` (the
largest reachable value is `pulse_table[30] + tnd_table[202]` = 0.9999801).
A fresh power-on is *not* `m` = 0: the triangle term above is 45, so `m` is
`tnd_table[45]` = 0.2554771 on the first cycle. No DC offset and no gain are
applied — the two non-linear mappings above are the whole mixer. Spot values
to check a table against: `pulse_table[1]` = 0.011609139, `pulse_table[30]` =
0.257512569, `tnd_table[45]` = 0.255477101, `tnd_table[127]` = 0.561346233,
`tnd_table[172]` = 0.677869439, `tnd_table[202]` = 0.742467582.

### Resampler (CPU rate → 44 100 Hz)

The per-cycle mixer value `m` is turned into 44 100 Hz samples by a
polyphase-FIR band-limited resampler: each change in `m` is scattered into a
delta ring buffer at its exact fractional output-sample position, the buffer
is integrated, and each integrated sample is filtered (next section).

State, all reset by `emu_load_rom` / `emu_reset`:

```
step       : f64   = 44100.0 / 1789773.0   ( = 0.02463999624533390678, f64 bits 0x3F993B3A2830680D)
phase      : f64   = 0.0
head       : int   = 32
primed     : bool  = false
delta_ring : f32[4096] = all 0.0
integrator : f32   = 0.0
held       : f32   = 0.0
```

`step` is that exact `f64` division of `44100.0` by `1789773.0`; `phase`
advances in `f64`.

The kernel `K[256][32]` of `f32`, also built once at power-on:

```
for p in 0..256:
    for t in 0..32:
        x    = (t as f32) - 16.0 + 0.5 - (p as f32)/256.0
        sinc = 0.92                       if |x| < 1e-6
             = sin(2*pi*0.46*x) / (pi*x)  otherwise        (f32 sine)
        w    = 0.42 - 0.5*cos(2*pi*((t + 0.5)/32)) + 0.08*cos(4*pi*((t + 0.5)/32))
        K[p][t] = sinc * w
    sum = K[p][0] + K[p][1] + ... + K[p][31]                (f32, tap order)
    if |sum| > 1.1920929e-7:                                (f32 EPSILON)
        scale every K[p][t] by 1.0/sum                      (so each row sums to 1.0)
```

(`pi` is the `f32` constant 3.14159274, bits 0x40490FDB; the expressions
above are evaluated left to right, so e.g. `2*pi*0.46*x` is
`((2*pi)*0.46)*x`.) Each row is a
32-tap Blackman-windowed sinc with a cutoff of `0.46` of the output rate,
normalised to unit DC gain. Anchor values: `K[0][16]` = `K[0][15]` =
0.62911510467529296875, `K[128][16]` = 0.92000108957290649414, `K[255][0]` =
-0.00000897894187801285.

Per CPU cycle, with mixer value `m`:

```
v     = clamp(m, -4.0, 4.0)  if m is finite, else 0.0        (f32)
delta = v - held ;  held = v

if delta != 0.0:
    row = K[ clamp(trunc(f32(phase) * 256.0), 0, 255) ]      (f32(phase) rounds the f64 to f32 first)
    for t in 0..32:
        delta_ring[(head - 16 + t) mod 4096] += delta * row[t]     (rounded product, then rounded add)

phase += step
while phase >= 1.0:
    phase -= 1.0
    head  += 1
    slot   = (head - 17) mod 4096
    integrator += delta_ring[slot]        (f32 add)
    delta_ring[slot] = 0.0
    if not primed:
        if head > 48: primed = true
        continue                          (this emission is discarded)
    emit filter(integrator)
```

Indices are taken modulo the 4096-slot ring. The ring size is not
observable: any power-of-two ring of 32 slots or more gives identical output,
because a scatter's 32-slot window never wraps onto a slot that has already
been emitted. The clamp to `[-4.0, 4.0]` and the non-finite-to-zero rule are
a backstop only — the mixer never leaves `[-4.0, 4.0]` — but they are part of
the definition. `delta` is only scattered when it is non-zero (an optimisation:
scattering zeros would be numerically identical).

Notes on the order, which the sample counts and the values both depend on:

- A cycle's phase wrap emits **after** that cycle's scatter, so the emitted
  slot `head - 17` can already carry this cycle's delta (that is its `t = 0`
  tap). Each scatter's 32 taps are consumed by the 32 wraps that follow it and
  each consumed slot is zeroed, so every tap lands on exactly one output
  sample: a delta scattered on the cycle a wrap happens reaches that same
  sample, and its remaining taps reach the next 31.
- `head` starts at 32 and the first 17 emissions of the resampler's life are
  discarded (`primed` only becomes true on the 17th, which is still
  discarded). Those 17 samples are the resampler's start-up latency; after
  them, one emitted sample corresponds to one `phase` wrap.

### Output filter

Each emitted sample passes through three one-pole sections in series — 90 Hz
high-pass, 440 Hz high-pass, 14 kHz low-pass — all at `fs = 44100`. These
corners are fixed: nothing selects a different filter model.

```
hp90  : c = exp(-2*pi*90/fs)      = 0.98725903034210205078125   (f32 bits 0x3F7CBD02)
hp440 : c = exp(-2*pi*440/fs)     = 0.939235150814056396484375  (f32 bits 0x3F7071B7)
lp14k : a = 1 - exp(-2*pi*14000/fs) = 0.86394035816192626953125 (f32 bits 0x3F5D2B32)
```

with each argument evaluated as `((-2.0*pi)*cutoff)/fs` in `f32` before the
exponential. Every section keeps two `f32` states, `prev_in` and `prev_out`,
starting at `0.0`, and processes one sample `x` as

```
high-pass : y = c * (prev_out + x - prev_in)      (times ((prev_out + x) - prev_in))
low-pass  : y = prev_out + a * (x - prev_out)
           then prev_in = x ; prev_out = y
```

in the order hp90 → hp440 → lp14k. The result is the sample that is buffered
and later drained.

### Conversion, and what `emu_audio_samples` returns

Each buffered sample `y` becomes

```
int16 = (clamp(y, -1.0, 1.0) * 32767.0) as i16
```

that is: clamp to `[-1.0, 1.0]` in `f32`, multiply by `32767.0` in `f32`
(rounding to nearest), then truncate toward zero. `1.0` → 32767, `-1.0` →
-32767, `0.5` → 16383. Non-finite inputs convert to 0, though the pipeline
cannot produce one.

`emu_audio_samples` drains: it reports how many samples were produced since
the previous call and `emu_audio_buffer` points at those. A frame's samples
are exactly the `emit`s that happened during that frame's cycles; nothing is
rounded, dropped or padded at a frame boundary, and the resampler's fractional
`phase` carries over.

### Expected counts

The per-frame counts follow from the console's frame timing (see "Frames"),
the `f64` phase accumulator and the 17 discarded start-up samples, and are a
good check on an implementation:

- the first `emu_run_frame` after power-on or reset emulates nothing, so it
  reports 0 samples;
- the second call reports 716 samples: that frame is where the 17 discarded
  wraps are consumed, and the accumulator has wrapped 733 times in total by
  the time the call returns (the power-on reset sequence's 8 cycles leave
  `phase` at 8 × `step` = 0.197, and the frame adds its own CPU cycles);
- every later call reports 733 or 734 samples.

733.8 average samples per frame is `44100 * 29780.67 / 1789773`: an NTSC frame
is 29 780.67 CPU cycles on average, and fewer or more by a few cycles
depending on whether rendering is on (the pre-render line's odd-frame dot
skip, ~1 cycle) and on DMA stalls. Which of 733/734 a given frame gets is
decided by its own cycle count plus the carried fractional `phase`. The 8
power-on cycles are not a rounding detail either: they put `phase` at 0.197
when the first frame starts, and that offset is what selects the reference's
exact 733/734 pattern (7 or 9 cycles would shift some frames' counts). The
counts are not rounded, resampled or clamped at frame boundaries — they are
exactly the wraps that happened while the frame ran, which is what makes them
reproducible.

### Exactness

Everything above is integer, `f32` or `f64` arithmetic with a fixed operation
order, so two implementations that follow it produce bit-identical streams,
with one exception: the kernel table depends on the accuracy of the
`sin`/`cos` used to build it. An implementation whose `sin`/`cos` are
correctly rounded to `f32`, or that computes the table in `f64` and rounds
each coefficient to `f32` once, differs from the table above by at most a few
units in the last place; measured over several minutes of mixed channel
audio, that shifts the emitted `f32` samples by less than `1e-5` and changes
the `int16` output by at most 1 (and the per-frame sample counts not at all).
The three filter coefficients are given exactly, so they carry no such
ambiguity.

## Mappers

The corpus uses mappers 0 (NROM), 1 (MMC1), 2 (UxROM), 3 (CNROM) and
4 (MMC3). MMC1 and MMC3 boards always have at least 8 KiB of PRG-RAM at
`$6000-$7FFF`, even when an NES 2.0 header declares none; a header that
declares more gets that amount.

## Environment

This will be loaded via wasmtime. No threads, no filesystem, no clock, no
main loop. Every import traps when called.
