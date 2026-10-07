# Development ROMs

Example ROMs for use while developing and debugging the emulator. **These ROMs are for development only and are not part of the graded corpus.** Each file is redistributed under the license named next to it; the full provenance (source repository, pinned revision, download URL, sha256) and verbatim license text is in the matching `*.LICENSE.txt` file (games) or `test/LICENSE.txt` (test ROMs).

## Games

| File | Title | Author | License | Mapper | Description | Source |
|---|---|---|---|---|---|---|
| `robotfindskitten.nes` | robotfindskitten (NES port) | Damian Yerrick (pinobatch) | Zlib | 0 | NROM-256 with CHR RAM; two-player "Zen simulation": walk robot around and bump into items until one is kitten. | <https://github.com/pinobatch/rfk-nes/releases/tag/v0.10> |
| `rhde.nes` | RHDE: Furniture Fight | Damian Yerrick (pinobatch) | GNU All-Permissive | 0 | NROM-256 with CHR RAM; two-player real-time strategy game: furnish your house, raid the opponent's, rebuild walls with polyomino pieces; Pently music. | <https://github.com/pinobatch/rhde-nes/releases/tag/v0.07> |
| `file-fixers.nes` | File Fixers | Wendel Scardua | MIT (music CC0) | 4 | MMC3, 64 KiB PRG, 16 KiB CHR ROM; computer-themed RPG (Game Off 2021) with FamiTone5 music. | <https://github.com/wendelscardua/file-fixers/releases/tag/0.1.0> |
| `stallar.nes` | Stallar | Wendel Scardua | MIT (music/SFX CC0) | 4 | MMC3, 64 KiB PRG, 8 KiB CHR ROM; game made for Ludum Dare 50 ("Delay the Inevitable") with FamiTone5 music. | <https://github.com/wendelscardua/stallar/releases/tag/0.1.0> |

All games are NTSC and boot to their title screen. Controller 1 (A, B, Select, Start, D-pad) is all that is needed; robotfindskitten and RHDE (a two-player game) also use controller 2.

## Test ROMs (`test/`)

Single-purpose CPU/PPU/APU/mapper tests by Shay Green ("blargg"), copied byte-for-byte from `third_party/RustyNES/tests/roms/` (identical to the files in <https://github.com/christopherpow/nes-test-roms>). Files are named `<suite>__<test>.nes`.

How results are reported: suites marked $6000 write a status byte to $6000 ($80 = running, $81 = press Reset, $00 = passed, other values = failure code) and a NUL-terminated text log from $6004, after writing the signature $DE $B0 $61 to $6001-$6003; they also print the result on screen and beep it. The others (sprite_hit_tests, sprite_overflow_tests, branch_timing_tests) report only on screen and by beeps.

| File | Suite | What it tests | Mapper | Output | Author | License |
|---|---|---|---|---|---|---|
| `test/ppu_vbl_nmi__02-vbl_set_time.nes` | ppu_vbl_nmi | exact time the VBL flag is set | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__03-vbl_clear_time.nes` | ppu_vbl_nmi | exact time the VBL flag is cleared | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__04-nmi_control.nes` | ppu_vbl_nmi | immediate NMI when NMI is enabled while the VBL flag is already set | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__06-suppression.nes` | ppu_vbl_nmi | reading $2002 near the time the VBL flag is set | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__07-nmi_on_timing.nes` | ppu_vbl_nmi | NMI occurrence when enabled near the time the VBL flag is cleared | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__08-nmi_off_timing.nes` | ppu_vbl_nmi | NMI occurrence when disabled near the time the VBL flag is set | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__09-even_odd_frames.nes` | ppu_vbl_nmi | PPU clock skipped every other frame when BG rendering is enabled | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_vbl_nmi__10-even_odd_timing.nes` | ppu_vbl_nmi | timing of the skipped odd-frame clock | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__02.alignment.nes` | sprite_hit_tests | alignment of sprite 0 hit with the background | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__03.corners.nes` | sprite_hit_tests | sprite 0 hit with a single set pixel at each of the four corners | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__04.flip.nes` | sprite_hit_tests | sprite 0 hit for a single-pixel sprite with flipping | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__05.left_clip.nes` | sprite_hit_tests | sprite 0 hit vs. clipping of the left 8 pixels | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__06.right_edge.nes` | sprite_hit_tests | sprite 0 hit at column 255 (ignored) and off the right edge | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__07.screen_bottom.nes` | sprite_hit_tests | sprite 0 hit at the bottom of the screen | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_hit_tests__08.double_height.nes` | sprite_hit_tests | basic sprite 0 hit with 8x16 sprites | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_overflow_tests__2.Details.nes` | sprite_overflow_tests | detailed sprite overflow flag operation | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/sprite_overflow_tests__3.Timing.nes` | sprite_overflow_tests | timing of the sprite overflow flag | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/apu_test__2-len_table.nes` | apu_test | all length counter table entries | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/apu_test__4-jitter.nes` | apu_test | APU clock jitter; basic frame IRQ flag timing | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/apu_test__5-len_timing.nes` | apu_test | length counter clock timing in both frame counter modes | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/apu_test__6-irq_flag_timing.nes` | apu_test | frame IRQ flag set timing | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/apu_test__8-dmc_rates.nes` | apu_test | the DMC's 16 rates | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/cpu_interrupts_v2__1-cli_latency.nes` | cpu_interrupts_v2 | delay before CLI takes effect; basic IRQ handling with the APU frame IRQ | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/cpu_interrupts_v2__3-nmi_and_irq.nes` | cpu_interrupts_v2 | NMI arriving during IRQ vectoring | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/mmc3_test_2__2-details.nes` | mmc3_test_2 | MMC3 scanline counter / IRQ details | 4 | $6000 | blargg (Shay Green) | Public domain |
| `test/mmc3_test_2__4-scanline_timing.nes` | mmc3_test_2 | MMC3 IRQ timing within the scanline | 4 | $6000 | blargg (Shay Green) | Public domain |
| `test/branch_timing_tests__1.Branch_Basics.nes` | branch_timing_tests | branch instruction timing basics | 0 | screen/beeps | blargg (Shay Green) | Public domain |
| `test/cpu_reset__registers.nes` | cpu_reset | CPU register values at power and after reset (asks for Reset) | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/ppu_open_bus.nes` | ppu_open_bus | reads of open-bus PPU bits/registers, including the decay register | 0 | $6000 | blargg (Shay Green) | Public domain |
| `test/instr_timing.nes` | instr_timing | timing of all CPU instructions (except the 12 that freeze the CPU) and branches | 1 | $6000 | blargg (Shay Green) | Public domain |

`cpu_reset__registers.nes` needs the console Reset button to be pressed when it asks (status $81). The public-domain status of the test ROMs is as documented by RustyNES; see the caveat in `test/LICENSE.txt`.
