# CPU

> Source: <https://www.nesdev.org/wiki/CPU> — NESdev Wiki, revision [24046](https://www.nesdev.org/w/index.php?oldid=24046) (last edited 2026-07-17T20:22:16Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=CPU&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The NES CPU core is based on the 6502 processor and runs at approximately 1.79 MHz (1.66 MHz in a PAL NES). It is made by [Ricoh](http://en.wikipedia.org/wiki/Ricoh) and lacks the MOS6502's decimal mode. In the NTSC NES, the [RP2A03](http://en.wikipedia.org/wiki/Ricoh_2A03) chip contains the CPU and APU; in the PAL NES, the CPU and APU are contained within the [RP2A07](http://en.wikipedia.org/wiki/Ricoh_2A03) chip.

## Sections

* [CPU instructions](6502_instructions.md)
* [CPU addressing modes](CPU_addressing_modes.md)
* [CPU memory map](CPU_memory_map.md)
* [CPU power-up state](CPU_power_up_state.md)
* [CPU registers](CPU_registers.md)
* [CPU status flag behavior](Status_flags.md)
* [CPU interrupts](CPU_interrupts.md)
* [Unofficial opcodes](CPU_unofficial_opcodes.md)
* [CPU pin-out and signals](https://www.nesdev.org/wiki/CPU_pinout), and other [hardware pin-outs](https://www.nesdev.org/wiki/Hardware_pinout)

## Frequencies

The CPU generates its clock signal by dividing the master clock signal.

| Rate | NTSC NES/Famicom | PAL NES | Dendy |
|---|---|---|---|
| Color subcarrier frequency *fsc* (exact) | 3579545.45 Hz (315/88 MHz) | 4433618.75 Hz | 4433618.75 Hz |
| Color subcarrier frequency *fsc* (approx.) | 3.579545 MHz | 4.433619 MHz | 4.433619 MHz |
| Master clock frequency 6*fsc* | 21.477272 MHz | 26.601712 MHz | 26.601712 MHz |
| Clock divisor *d* | 12 | 16 | 15 |
| CPU clock frequency 6*fsc*/*d* | 1.789773 MHz (~559 ns per cycle) | 1.662607 MHz (~601 ns per cycle) | 1.773448 MHz (~564 ns per cycle) |

* The vast majority of PAL famiclones use a chipset or NOAC with this timing. A small number have UMC UA6540+6541, which also uses PAL NES timing.[[1]](#cite_note-1)

## Notes

* There are no differences in the execution of illegal 6502 opcodes between the 2A03 and the 2A07.
* Every cycle on 6502 is either a read or a write cycle.
* A printer friendly version covering all section is available [here](https://www.nesdev.org/wiki/CPU_ALL).
* Emulator authors may wish to emulate the NTSC NES/Famicom CPU at 21441960 Hz ((341×262−0.5)×4×60) to ensure a synchronised/stable 60 frames per second.[[2]](#cite_note-2)

## See also

* [Cycle reference chart](Cycle_reference_chart.md)
* [2A03 technical reference](http://nesdev.org/2A03%20technical%20reference.txt) by Brad Taylor. (Pretty old at this point; information on the wiki might be more up-to-date.)

## References

1. [↑](#cite_ref-1) [nesdev forum: Eugene.S provides a list of famiclones](https://forums.nesdev.org/viewtopic.php?f=3&t=17213#p216082)
2. [↑](#cite_ref-2) [nesdev forum: Mesen - NES Emulator](http://forums.nesdev.org/viewtopic.php?p=223679#p223679)
