# UxROM

> Source: <https://www.nesdev.org/wiki/UxROM> — NESdev Wiki, revision [24147](https://www.nesdev.org/w/index.php?oldid=24147) (last edited 2026-08-26T15:12:24Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=UxROM&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

**UxROM**

|  |  |
|---|---|
| **Company** | Nintendo, others |
| **Games** | [155 in NesCartDB](https://nescartdb.com/search/advanced?ines=2) |
| **Complexity** | Discrete logic |
| **Boards** | UNROM, UOROM |
| **PRG ROM capacity** | 256K/4096K |
| **PRG ROM window** | 16K + 16K fixed |
| **PRG RAM capacity** | None |
| **CHR capacity** | 8K |
| **CHR window** | n/a |
| **[Nametable arrangement](Mirroring.md#Nametable_Mirroring)** | Fixed H or V, controlled by solder pads |
| **[Bus conflicts](Bus_conflict.md)** | Yes/No |
| **IRQ** | No |
| **Audio** | No |
| **iNES [mappers](Mapper.md)** | [002](UxROM.md), [094](https://www.nesdev.org/wiki/INES_Mapper_094), [180](https://www.nesdev.org/wiki/INES_Mapper_180) |

**NESCartDB**

|  |
|---|
| [iNES 002](https://nescartdb.com/search/advanced?ines=2) |
| [UxROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=-U%25ROM) |
| [UNROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=UNROM) |
| [UOROM](https://nescartdb.com/search/advanced?unif_op=LIKE+`%25%40%25`&unif=UOROM) |

The generic designation *UxROM* refers to the Nintendo cartridge boards *NES-UNROM*, *NES-UOROM*, their [HVC](https://www.nesdev.org/wiki/Family_Computer) counterparts, [HVC-UN1ROM](https://www.nesdev.org/wiki/INES_Mapper_094), and clone boards.

* **[iNES](INES.md) Mapper 002** is the implementation of the most common usage of UxROM compatible boards, described in this article.
  * Submapper 0: Bus conflict behavior unknown
  * Submapper 1: No bus conflicts
  * Submapper 2: AND-type bus conflicts
* [iNES Mapper 094](https://www.nesdev.org/wiki/INES_Mapper_094) describes UN1ROM, used only in *Senjou no Ookami*.
* [iNES Mapper 180](https://www.nesdev.org/wiki/INES_Mapper_180) describes a reconfiguration of UNROM used only in *Crazy Climber*.

Example games:

* *Mega Man*
* *Castlevania*
* *Contra*
* *Duck Tales*
* *Metal Gear*

## Banks

* CPU $8000-$BFFF: 16 KB switchable PRG ROM bank
* CPU $C000-$FFFF: 16 KB PRG ROM bank, fixed to the last bank

## Solder pad config

* Horizontal mirroring : 'H' disconnected, 'V' connected.
* Vertical mirroring : 'H' connected, 'V' disconnected.

## Registers

### Bank select ($8000-$FFFF)

```
7  bit  0
---- ----
xxxx pPPP
     ||||
     ++++- Select 16 KB PRG ROM bank for CPU $8000-$BFFF
          (UNROM uses bits 2-0; UOROM uses bits 3-0)
```

Emulator implementations of **iNES mapper 2** treat this as a full 8-bit bank select register, without bus conflicts. This allows the mapper to be used for similar boards that are compatible.

To make use of all 8-bits for a 4 MB PRG ROM, an [NES 2.0](NES_2.0.md) header must be used ([iNES](INES.md) can only effectively go to 2 MB).

The original UxROM boards used by Nintendo were subject to bus conflicts, and the relevant games all work around this in software.
Some emulators (notably FCEUX) will have bus conflicts by default, but others have none.
[NES 2.0 submappers](https://www.nesdev.org/wiki/NES_2.0_submappers#002.2C_003.2C_007:_UxROM.2C_CNROM.2C_AxROM) were assigned to accurately specify whether the game should be emulated with bus conflicts.

## Hardware

The UNROM, UN1ROM, and UOROM boards contain a [74HC161](https://www.nesdev.org/wiki/74161) binary counter used as a quad D latch (4-bit register) and a [74HC32](https://www.nesdev.org/wiki/7432) quad 2-input OR gate to make one bank always visible.

The circuit behaves as if it were as follows:

```
      /PRGSEL               A14  A13-A0
       |                      |      |
       |                      |      |
       | D2-D0-.       ,------'      |
       |       |     . |             |
       | ,-----+--.  |`+.            |
       | |Register+--+0  `.          |
       | `--------'  |    |__.       |
       |       |     |    |  |       |
       | R/W --'   7-+1  ,'  |       |
       |             | ,'    |       |
       |             |'      |       |
       |                     |       |
      ,+---------------------+-------+-----.
      |/CE              A16-A14  A13-A0    |
      |         128K by 8 bit ROM     D7-D0+-- to 2A03 data bus
      |                                    |
      `------------------------------------'
```

(This diagram is for UNROM. UOROM has one more bit in the register, multiplexer output, and address input A17, and the multiplexer's other input is $F. Homebrew boards capable of 512 KiB have one more of each, and the multiplexer's input is $1F.)

The quad OR gate here acts as a multiplexer.
A [74HC02](https://www.nesdev.org/wiki/7402) quad NOR gate can be used instead if the banks are stored in reverse order in the ROM. If the program is 128 KiB or smaller, the 7402 way leaves one NOR gate free to invert R/W into /OE to avoid [bus conflicts](Bus_conflict.md).[[1]](http://forums.nesdev.org/viewtopic.php?p=111686#p111686)

If an actual multiplexer ([74HC157](https://www.nesdev.org/wiki/74157) quad 2:1) is cheaper than an OR gate, a third-party UxROM-compatible board can use that instead of the 74HC32, as [kyuusaku suggested](http://forums.nesdev.org/viewtopic.php?p=93516#p93516).

## Variants

* The [mapper](https://www.nesdev.org/wiki/INES_Mapper_071) used in Codemasters games published by Camerica extends UxROM with [CIC](https://www.nesdev.org/wiki/CIC_lockout_chip) defeat circuitry, in addition being devoid of bus conflicts.
* Nintendo's [HVC-UN1ROM](https://www.nesdev.org/wiki/INES_Mapper_094) board moves the bankswitching bits within the byte.
* *Crazy Climber* replaces the [74HC32](https://www.nesdev.org/wiki/7432) quad-OR gate by a [74HC08](https://www.nesdev.org/wiki/7408) quad-AND gate, so that the first bank is fixed at $8000-$BFFF and the switchable bank is present at $C000-$FFFF. This configuration is assigned to [iNES Mapper 180](https://www.nesdev.org/wiki/INES_Mapper_180), which uses the same UNROM PCB.
* With an 8-bit latch ([74HC377](https://www.nesdev.org/wiki/74377) or an additional 74HC161) and an additional 74HC32 to control A18-A21, a third-party board implementing this mapper can switch 4 MiB of PRG ROM.
  * [Battle Kid 2: Mountain of Torment](https://www.nesdev.org/wiki/User:Sivak/Battle_Kid_2:_Mountain_of_Torment) implements a 512kB UxROM mapper.
  * [UNROM 512](https://www.nesdev.org/wiki/UNROM_512) implements a superset of 512kB UxROM, with additional CHR-RAM banking and the possibility of flash save.
* The homebrew game *Alwa's Awakening* adds 8 KiB of battery-backed PRG-RAM in the usual $6000-$7FFF address range. A few Subor educational cartridges have non-battery-backed PRG-RAM in the same range as well. Emulators should support this PRG-RAM at least in the presence of a NES 2.0 header that explicitly specifies it.

## See also

* [Programming UNROM](https://www.nesdev.org/wiki/Programming_UNROM)

## External links

* [Comprehensive NES Mapper Document](http://nesdev.org/mappers.zip) by \Firebug\, information about mapper's initial state is inaccurate.
* NES mapper list by Disch [[2]](https://romhack.ing/database/content/entry/ktNw5JQBNs8FWu0CfZdn/nes-mapper-list)
* [Converting UNROM to UOROM](https://forums.nesdev.org/viewtopic.php?p=179581#p179581)
