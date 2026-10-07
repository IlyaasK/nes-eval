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

## Mappers

The corpus uses mappers 0 (NROM), 1 (MMC1), 2 (UxROM), 3 (CNROM) and
4 (MMC3). MMC1 and MMC3 boards always have at least 8 KiB of PRG-RAM at
`$6000-$7FFF`, even when an NES 2.0 header declares none; a header that
declares more gets that amount.

## Environment

This will be loaded via wasmtime. No threads, no filesystem, no clock, no
main loop. Every import traps when called.
