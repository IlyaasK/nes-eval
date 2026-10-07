# NROM

> Source: <https://www.nesdev.org/wiki/NROM> — NESdev Wiki, revision [23660](https://www.nesdev.org/w/index.php?oldid=23660) (last edited 2026-03-15T08:55:15Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=NROM&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

**NROM**

|  |  |
|---|---|
| **Company** | Nintendo, others |
| **Boards** | NROM, HROM*, RROM, RTROM, SROM, STROM |
| **PRG ROM capacity** | 16K or 32K |
| **PRG ROM window** | n/a |
| **PRG RAM capacity** | 2K or 4K in *Family BASIC* only |
| **PRG RAM window** | n/a |
| **CHR capacity** | 8K |
| **CHR window** | n/a |
| **[Nametable arrangement](Mirroring.md#Nametable_Mirroring)** | Fixed H or V, controlled by solder pads (*V only) |
| **[Bus conflicts](Bus_conflict.md)** | Yes |
| **IRQ** | No |
| **Audio** | No |
| **iNES [mappers](Mapper.md)** | [000](NROM.md) |

**NESCartDB**

|  |
|---|
| [iNES 000](https://nescartdb.com/search/advanced?ines=0) |
| [NROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-NROM) |
| [HROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-HROM) |
| [RROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-RROM) |
| [RTROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-RTROM) |
| [SROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-SROM) |
| [STROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-STROM) |

The generic designation **NROM** refers to the Nintendo cartridge boards NES-NROM-128, NES-NROM-256, their [HVC](https://www.nesdev.org/wiki/Family_Computer) counterparts, and clone boards. The [iNES](INES.md) format assigns **mapper 0** to NROM.

The suffixes 128 and 256 refer to kilobits by Nintendo's own designation; not kilobytes (as is the case with [UNROM 512](https://www.nesdev.org/wiki/UNROM_512), for example).

Although mapper 0 does not have any submappers, other fields in the [NES 2.0](NES_2.0.md) header may be used to specify the following:

* (Battery-backed) PRG-RAM size, for *Family BASIC*
* PRG-ROM size in exponent-multiplier form, for *Galaxian*

## Banks

* CPU $6000-$7FFF: Unbanked PRG-RAM, mirrored as necessary to fill entire 8 KiB window, write protectable with an external switch. (*Family BASIC* only)
* CPU $8000-$BFFF: First 16 KiB of PRG-ROM.
* CPU $C000-$FFFF: Last 16 KiB of PRG-ROM (NROM-256) or mirror of $8000-$BFFF (NROM-128).
* PPU $0000-$1FFF: 8 KiB CHR-ROM.

All banks are fixed.

## Solder pad config

* Vertical arrangement ("Horizontal mirroring") : 'H' disconnected, 'V' connected.
* Horizontal arrangement ("Vertical mirroring") : 'H' connected, 'V' disconnected.

## Registers

None. This normally has no mapping capabilities whatsoever, so the presence of [bus conflicts](Bus_conflict.md) are irrelevant from a software perspective.

Nevertheless, tile animation can be done by swapping between pattern tables $0000 and $1000, using [PPUCTRL](PPU_registers.md#PPUCTRL) bits 4-3 as a "poor man's [CNROM](CNROM.md)".

## Variants

These board variants do not affect emulation:

* [RROM and SROM](https://www.nesdev.org/wiki/Mask_ROM_pinout#Variants) use different CHR-ROM pinouts.
* RTROM and STROM split the PRG-ROM into two 8KiB ROMs.
* HROM is an early variant without the solder pads - only the horizontal arrangement ("vertical mirroring") is available, as if the 'H' pad were selected.

A few cartridges require NES 2.0 features to accurately specify:

* *Family BASIC* contains 2KiB PRG-RAM (or 4KiB PRG-RAM for V3) accessible at $6000-$7FFF and [decoded with a 74HC20](https://www.nesdev.org/wiki/PRG_RAM_circuit), that is backed with 2 AA batteries. The cartridge is equipped with a back up switch that, when enabled, disables all access to PRG-RAM in order to prevent possible data corruption when the Famicom is turned on or off. Most emulators will map 8KiB of PRG-RAM to $6000-$7FFF if the battery flag is set (for iNES compatibility) - the NES 2.0 PRG-RAM size field should be used to denote the exact size.
* [*Galaxian*](https://nescartdb.com/profile/view/1808/galaxian) uses a smaller 8KiB PRG-ROM chip. iNES dumps of this game are therefore all overdumps. NES 2.0 is capable of specifying this size using the exponent-multiplier form.

Some unlicensed games and modern homebrew programs use mapper 0 with 8KiB of CHR-RAM, which would not be supported on official NROM PCBs without rewiring ([BNROM](https://www.nesdev.org/wiki/INES_Mapper_034) may be a better option for reproduction boards). Most emulators will do the right thing with mapper 0 + CHR-RAM, however.

[NROM-368](https://www.nesdev.org/wiki/NROM-368) is a recent invention that allows addressing more PRG-ROM without bankswitching.

## See also

* [Programming NROM](https://www.nesdev.org/wiki/Programming_NROM)
