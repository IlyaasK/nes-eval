# nes-core

NES Eval's baseline candidate: a plain, from-scratch NES emulator in Rust,
compiled to `wasm32-unknown-unknown` against [`spec/ABI.md`](../../spec/ABI.md).
It is the NES counterpart of GBA Eval's `gba-core`: an honest emulator
written straight from public hardware documentation (the
[NESdev wiki](https://www.nesdev.org/wiki/)), without heroics, to show what
a working but imperfect emulator scores. No other emulator's source was
consulted. MIT licensed.

```sh
candidates/nes-core/build.sh          # -> candidates/nes-core/nes_core.wasm
target/release/grader candidates/nes-core/nes_core.wasm corpus/ out/nes-core
```

`cargo run --release --example run -- <rom.nes> <frames> [replay.txt|-] [out.ppm]`
(from this directory) runs a ROM natively, prints blargg-style results from
`$6000` and writes the last frame as a PPM.

## Layout

| File | What |
|---|---|
| `src/lib.rs` | `Nes` (power-on, frame loop) and the 10 ABI exports |
| `src/cpu.rs` | 2A03 CPU |
| `src/bus.rs` | CPU memory map, controller, OAM DMA, DMC fetch stall, interrupt lines |
| `src/ppu.rs` | 2C02 PPU |
| `src/apu.rs` | APU, mixer, resampler |
| `src/cart.rs` | iNES / NES 2.0 parsing, mappers 0-4 |

## What it does

- **Timing model.** Every CPU bus access (dummy reads and writes included)
  first advances the machine one CPU cycle: three PPU dots and one APU
  cycle. Instruction cycle counts, page-cross penalties and the interrupt
  poll on the second-to-last cycle come out of that.
- **CPU.** All official opcodes and most unofficial ones (SLO, RLA, SRE,
  RRA, SAX, LAX, DCP, ISB, ANC, ALR, ARR, AXS, SBC $EB, multi-byte NOPs,
  approximations of the unstable SHA/SHX/SHY/TAS/LAS/XAA/LXA; KIL jams).
  NMI edge latch, level IRQ, CLI/SEI/PLP poll delay, NMI hijacking of
  BRK/IRQ, OAM DMA (513/514 cycles), DMC DMA (fixed 4-cycle stall).
- **PPU.** Dot-stepped background pipeline with loopy v/t/x/w, sprites
  (8x8 and 8x16, priority, left clipping), sprite-0 hit, simple sprite
  overflow, VBlank/NMI with $2002 read suppression, odd-frame dot skip,
  palette mirroring, greyscale and emphasis into the index output,
  backdrop/`v`-in-palette colour when rendering is off.
- **APU.** Pulse x2 (envelope, sweep with ones'/two's complement, length),
  triangle (linear counter), noise (both LFSR modes), DMC (sample fetch,
  loop, IRQ), 4/5-step frame counter with IRQ and delayed $4017 reset, the
  nonlinear mixer from lookup tables, box-filter decimation to 44.1 kHz
  (exact rational step), 90 Hz high-pass, i16 mono.
- **Mappers.** NROM, MMC1 (incl. SUROM 512 KiB PRG, consecutive-write
  ignore), UxROM, CNROM, MMC3 (IRQ clocked once per rendered line at
  dot 260). PRG RAM: the size from a NES 2.0 header, 8 KiB for iNES 1.0.

## Deliberately not implemented

- MMC3 IRQ from real PPU A12 edges (a per-scanline clock instead; `$2006`
  A12 toggles don't clock it, nor does a `$1000` background table move it).
- The per-dot sprite evaluation state machine, its overflow bug, and
  `$2004` reads during rendering. Sprites for a line are evaluated and
  fetched all at once at dot 257.
- PPU open-bus decay, the `$2007` read/write increment glitch during
  rendering, the PPU's power-on warm-up period (register writes ignored
  until the first pre-render line).
- Branch-instruction interrupt delay quirks, DMC DMA cycle-exact stalls
  and its interaction with `$4016`/`$2007` reads.
- MMC1 PRG-RAM disable bit and SxROM PRG-RAM banking; MMC3 PRG-RAM protect.
- Mappers other than 0-4, PAL, controller 2, save files.

## Test ROMs (native runner, not the graded corpus)

Pass: instr_test_v5 01-16 except 03/07 (unofficial LXA, SHX/SHY),
instr_misc, instr_timing, cpu_timing_test6, nestest, cpu_interrupts_v2/1-2,
apu_test 1-8, ppu_vbl_nmi 01-04, 06, 07, 09, sprite_hit_tests 01-11,
sprite_overflow_tests 1, 2, 5, oam_read, Holy Mapperel 0/1/2/3/4.

Fail: cpu_interrupts_v2/3-5, ppu_vbl_nmi 05, 08, 10, sprite_overflow 3-4,
every mmc3_test_2 ROM (A12 timing), ppu_open_bus.

## Score

Measured once, with no tuning against the corpus:

| Overall | Procedural | Replay | Audio |
|---|---|---|---|
| 0.5811 | 0.8300 | 0.4569 | 0.7050 |
