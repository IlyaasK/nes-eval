# AxROM

> Source: <https://www.nesdev.org/wiki/AxROM> — NESdev Wiki, revision [24152](https://www.nesdev.org/w/index.php?oldid=24152) (last edited 2026-08-27T17:22:11Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=AxROM&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

**AxROM**

|  |  |
|---|---|
| **Company** | Nintendo, Rare, others |
| **Games** | [34 in NesCartDB](https://nescartdb.com/search/advanced?ines=7) |
| **Complexity** | Discrete logic |
| **Boards** | AMROM, ANROM,<br>AN1ROM, AOROM, others |
| **PRG ROM capacity** | 256K |
| **PRG ROM window** | 32K |
| **PRG RAM capacity** | None |
| **PRG RAM window** | n/a |
| **CHR capacity** | 8K |
| **CHR window** | n/a |
| **[Nametable arrangement](Mirroring.md#Nametable_Mirroring)** | 1 page switchable |
| **[Bus conflicts](Bus_conflict.md)** | AMROM/AOROM only |
| **IRQ** | No |
| **Audio** | No |
| **iNES [mappers](Mapper.md)** | [007](AxROM.md) |

**NESCartDB**

|  |
|---|
| [iNES 007](https://nescartdb.com/search/advanced?ines=7) |
| [AxROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-A%25ROM) |
| [AMROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=AMROM) |
| [ANROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=ANROM) |
| [AN1ROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=AN1ROM) |
| [AOROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=AOROM) |

The generic designation **AxROM** refers to Nintendo cartridge boards **NES-AMROM**, **NES-ANROM**, **NES-AN1ROM**, **NES-AOROM**, their [HVC](https://www.nesdev.org/wiki/Family_Computer) counterparts, and clone boards. AxROM and compatible boards are implemented in [iNES](INES.md) format with **iNES Mapper 7**.

* **[Submapper number](https://www.nesdev.org/wiki/NES_2.0_submappers#002,_003,_007:_UxROM,_CNROM,_AxROM)** determines the bus conflict behavior of the game.
  * Submapper 0: Bus conflict behavior undefined, but emulators *shouldn't* emulate bus conflicts
  * Submapper 1: No bus conflicts
  * Submapper 2: AND-type bus conflicts

## Board types

The following AxROM boards are known to exist:

| Board | PRG ROM | Bus conflicts |
|---|---|---|
| AMROM | 128 KB | Yes |
| ANROM | 128 KB | No |
| AN1ROM | 64 KB | No |
| AOROM | 128 / 256 KB | Depends on +CE wiring |

## Overview

* PRG ROM size: up to 256 KB
* PRG ROM bank size: 32 KB
* PRG RAM: None
* CHR capacity: 8 KB RAM
* CHR bank size: Not bankswitched
* Nametable [mirroring](Mirroring.md): Single-screen, mapper-selectable
* Subject to [bus conflicts](Bus_conflict.md): AMROM/AOROM only

## Banks

* CPU $8000-$FFFF: 32 KB switchable PRG ROM bank

## Solder pad config

No solder pad config is needed on the AxROM board family.

## Registers

### Bank select ($8000-$FFFF)

```
7  bit  0
---- ----
xxxM xPPP
   |  |||
   |  +++- Select 32 KB PRG ROM bank for CPU $8000-$FFFF
   +------ Select 1 KB VRAM page for all 4 nametables
```

## Hardware

The AxROM boards contain a [74HC161](https://www.nesdev.org/wiki/74161) binary counter used as a quad D latch (4-bit register). The ANROM and AN1ROM boards also contains a [74HC02](https://www.nesdev.org/wiki/7402) which is used to disable the PRG ROM during writes, thus avoiding [bus conflicts](Bus_conflict.md).

## Various notes

On the AOROM board, special mask ROMs with an additional positive CE on pin 31 (which is connected to PRG R/W) can be used to prevent bus conflicts without an additional chip. Some 128 KB games were made with AOROM to save the cost of a 74HC02. It seems that only *Double Dare* and *Wheel of Fortune* employ this trick noticeably--that is, if emulated with bus conflicts enabled, the games will glitch. [The list of AxROM games in NesCartDB](https://nescartdb.com/search/advanced?ines_op=equal&ines=7) lacks quality coverage of the PCB backs for the AOROM games, so it is hard to determine yet which games may be wired this way.

It is likely that every retail AOROM game could be emulated correctly without emulating bus conflicts.

## Variants

* [BNROM](https://www.nesdev.org/wiki/INES_Mapper_034) is the same as AMROM except it uses a fixed horizontal or vertical mirroring.
* [NES 2.0 Mapper 470](https://www.nesdev.org/wiki/NES_2.0_Mapper_470) adds an outer bank register to AOROM.
* Some emulators allow bit 3 to be used to select up to 512 KB of PRG ROM for an oversized AxROM. In hardware this could be implemented by using an octal latch in place of the quad latch ([74HC377](https://www.nesdev.org/wiki/74377)).
  * The pirate multicart and unlicensed music game *Hot Dance 2000* uses a 512 KB AxROM variant.
  * A ROM Hack of Battletoads expanded the ROM size to 512 KB.

## See also

* [Comprehensive NES Mapper Document](http://nesdev.org/mappers.zip) by \Firebug\, information about mapper's initial state is inaccurate.
