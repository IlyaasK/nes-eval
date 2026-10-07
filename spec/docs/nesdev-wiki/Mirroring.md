# Mirroring

> Source: <https://www.nesdev.org/wiki/Mirroring> — NESdev Wiki, revision [24072](https://www.nesdev.org/w/index.php?oldid=24072) (last edited 2026-08-05T09:36:17Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=Mirroring&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

There are two types of **mirroring** on the NES:

* **[Memory Mirroring](#Memory_Mirroring)** is when the same memory may be accessed at multiple addresses, causing an apparent duplication.
* **[Nametable Mirroring](#Nametable_Mirroring)** describes the layout of the NES' 2x2 [background nametable](PPU_nametables.md) graphics, usually achieved by mirrored memory.

## Memory Mirroring

Memory mirroring refers to the appearance of memory or I/O registers at more than one range of addresses, with the same byte being accessible at more than one address.
This occurs when the full address isn't completely *decoded*, that is, when a chip ignores one or more address lines.
Because completely decoding an address usually takes a lot more pins on a chip, incomplete decoding is used to reduce the hardware required; if the mirror occupies otherwise unused address space, it poses no problems.

Within the NES, many things are mirrored:

* System memory at $0000-$07FF is mirrored at $0800-$0FFF, $1000-$17FF, and $1800-$1FFF - attempting to access memory at, for example, $0173 is the same as accessing memory at $0973, $1173, or $1973.
* PPU I/O registers at $2000-$2007 are mirrored at $2008-$200F, $2010-$2017, $2018-$201F, and so forth, all the way up to $3FF8-$3FFF.
* The single registers on most [simple mappers](https://www.nesdev.org/wiki/Category:Discrete_logic_mappers) are mirrored throughout $8000-$FFFF.
* Registers on many common ASIC mappers (such as the MMC1 and MMC3) are also mirrored, in groups, throughout $8000-$FFFF.
* Nametable mirroring, as described below, happens due to memory mirroring within PPU $2000-$2FFF (hence its name). However, in this case the memory mirroring is intentional and necessary.
* In [NROM](NROM.md)-128, the 16k PRG ROM is mirrored into both $8000-$BFFF and $C000-$FFFF.
* In most [mappers](Mapper.md), banks past the end of PRG or CHR ROM show up as mirrors of earlier banks. For example, [UNROM](UxROM.md) PRG banks 8-15 are duplicates of banks 0-7 respectively. [Non-power-of-two ROM size](https://www.nesdev.org/wiki/Non-power-of-two_ROM_size) may imply more complicated mirroring.

## Nametable Mirroring

Nametable mirroring affects what is shown past the right and bottom edges of the current nametable. When mirroring is enabled for a particular axis (horizontal and/or vertical), the coordinates simply wrap around on the current nametable. A background "mirrored" in this way is repeated, *not* flipped. When mirroring is disabled, a second nametable is used. There are four common combinations of mirroring outlined below. **A**, **B**, **C**, and **D** in the diagrams refer to screens of 1024 bytes each.

### Horizontal

*[Image omitted]*

A **vertical arrangement** of the nametables results in **horizontal mirroring**, which makes a **32x60 tilemap**.

This is most commonly used for games which only scroll vertically or in all directions.

Doing any horizontal scrolling using horizontal mirroring is hard to do smoothly because the data on the right of the screen is immediately show on the left due to mirroring. Clever use of hardware left-side screen clipping will hide all name table glitches, but because the attribute tables have a resolution of 2x2 tiles, there will always be attribute glitches on the left and/or the right side of the screen. The best possible way to hide it is to have 4 pixels with potentially wrong attributes on both sides, but most commercial games did worse than that having usually 8 or even more glitchy pixels, so that is why so many NES games have color glitches on the border of the screen.

Some televisions [overscan](Overscan.md) up to 8 pixels on both left and right border, but most don't. Perfectionist programmers could use solid black sprites on the right border to hide attribute glitches and make the screen look symmetrical and hide absolutely all attribute glitches, as in the game *Alfred Chicken*, but very few games do this because it reduces the number of sprites per scanline to 7 and wastes a lot of [OAM](PPU_OAM.md) space (roughly 1/4 in 8x16 pixel sprite mode).

To configure a cartridge board for horizontal mirroring, connect PPU A11 to CIRAM A10.
On cartridge boards made by Nintendo, this is selected by shorting the "V" solder pad (for "vertical arrangement").

### Vertical

*[Image omitted]*

A **horizontal arrangement** of the nametables results in **vertical mirroring**, which makes a **64x30 tilemap**.

This is most commonly used for games which only scroll horizontally. Games that scroll vertically (by any amount and without status bar) and that never scroll horizontally by more than one screen would use this mirroring (e.g. Lode Runner, Bomberman, Fire Emblem, Crystal Mines), so that they don't have to load anything when scrolling horizontally.

Of course it is also used for games which scroll in both directions without a status bar. Because data that is on the top/bottom of the screen will immediately show up on the other side, a clever use of NTSC [overscan](Overscan.md) can make it glitch-less multidirectional scrolling, but glitches will appear on PAL televisions (and NTSC televisions with a overscan range which is a little off). The best possible way to hide glitches is to make 4 pixels with wrong tiles and 4 additional pixels with wrong color on both sides, but most commercial games did much worse than this, that's why they look so bad if overscan is disabled.

Perfectionist programmers could use raster split to hide glitches (and possibly also provide more blanking time to update VRAM) as in the games *Jurassic Park* and *M.C. Kids*, but it was rarely done because it complicates the code a lot for little benefits.

To configure a cartridge board for vertical mirroring, connect PPU A10 to CIRAM A10.
On cartridge boards made by Nintendo, this is selected by shorting the "H" solder pad (for "horizontal arrangement").

### Single-Screen

*[Image omitted]*

Single-screen arrangement/mirroring is only available with certain mappers, such as the [AxROM](AxROM.md), [SxROM](MMC1.md#SxROM_board_types), and [TLSROM](https://www.nesdev.org/wiki/TLSROM) boards, resulting in **two 32x30 tilemaps**. Some references may refer to this arrangement as a lower bank or upper bank being mirrored across all 4 nametables - this corresponds to screen A and the hidden screen B in the diagram, respectively.

Popular mappers with single-screen arrangements allow switching between the two screens. The main advantage is that this allows using a status bar at the top or bottom of the screen while also allowing the playfield to extend equally in any direction - this can be done by storing the status bar in one nametable, rendering the playfield in the other nametable, and switching mirroring (and scrolling parameters) at the appropriate screen location during rendering.

There are many other things that can be drastically simplified when using 1-screen mirroring: The formulas used to calculate PPU addresses for data to be updated are significantly simpler, and if the status bar has a variable size or is scrolling, all this would be a headache without 1-screen mirroring.

When this mirroring is used to scroll horizontally, similar glitches and scrolling problems that those of horizontal mirroring will happen. However, as long as there is a status bar, no glitches will happen vertically since the data that falls off the bottom (or the top) of the screen will come in the area that is "hidden" by the status bar, regardless of overscan factors.

There are several different ways to configure a cartridge board for single-screen mirroring. Some boards have single screen mirroring which is mapper controlled, like MMC1 and AxROM. For AxROM connect the output of a register (e.g. [74HC161](https://www.nesdev.org/wiki/74161)) to CIRAM A10 (AxROM latches PRG D4). A simpler way to create one screen mirroring that's fixed (non-changable by software) is to simply connect CIRAM A10 to Gnd or Vcc.
The board that implements [iNES Mapper 218](https://www.nesdev.org/wiki/INES_Mapper_218) wires CIRAM A10 to PPU A10, A11, A12, or A13, so that the game can store tiles in CIRAM without having any CHR ROM or CHR RAM in the cartridge.

### 4-Screen

*[Image omitted]*

With additional RAM and/or PPU address mapping present on the cartridge, 4 unique nametables can be addressed through the PPU bus, creating a **64x60 tilemap**, allowing for more flexible screen layouts. Very few games made use of this lack of mirroring.

Games known to use 4-screen RAM nametables:

* *Rad Racer II*
* *Gauntlet*
* *Napoleon Senki*
* *Rocman X (Sachen)*
* Every [Vs. System](https://www.nesdev.org/wiki/Vs._System) game

Example games using 4-screen with [ROM nametables](https://www.nesdev.org/wiki/Category:Mappers_with_ROM_nametables):

* *Final Lap*
* *King of Kings*

Mappers known to be used with 4 screens of nametables:

* [MMC3](MMC3.md) (*Rad Racer II*).
* [iNES 206](https://www.nesdev.org/wiki/INES_Mapper_206) implements a subset of MMC3 features, and can use 4-screen RAM (*Gauntlet*).
* [iNES 77](https://www.nesdev.org/wiki/INES_Mapper_077) maps RAM across the PPU memory space, combining with internal VRAM to provide 4 RAM nametables, and a combination of CHR-RAM and ROM for pattern tables (*Napoleon Senki*).
* [Vs. System](https://www.nesdev.org/wiki/Vs._System) had twice as much VRAM as the NES, giving a permanent 4-screen setup. This is most visible as [iNES 99](https://www.nesdev.org/wiki/INES_Mapper_099), but [several other mappers](https://www.nesdev.org/wiki/Vs._System#See_also) were also used on this hardware.
* [Namco 163](https://www.nesdev.org/wiki/INES_Mapper_019) allows 1k CHR-ROM pages to be arbitrarily mapped into the 4 nametable screens (*Final Lap*, *King of Kings*).
* The [JY Company](https://www.nesdev.org/wiki/J.Y._Company_ASIC) mapper allows 1k CHR-ROM pages to be arbitrarily mapped into the 4 nametable screens.
* [UNROM 512](https://www.nesdev.org/wiki/UNROM_512) and [GTROM](https://www.nesdev.org/wiki/GTROM) are homebrew mappers with 4-screen configurations.

Other mappers capable of uncommon 4-screen layouts:

* [VRC6](https://www.nesdev.org/wiki/VRC6) allows 1k CHR-ROM pages to be arbitrarily mapped into the 4 nametable screens.
* [MMC5](https://www.nesdev.org/wiki/MMC5) can use its internal RAM to create a 3rd nametable, while procedurally generating a blank data page for a 4th, allowing (just barely) 4 different screens to be mapped at once.

The [iNES](INES.md) format can specify 4 nametables in the header, allowing 4-screen RAM nametables to be applied to any mapper that doesn't structurally conflict with this (if supported by the emulator).

There are several ways to implement extra nametable RAM on a cartridge board:

* Add an extra 2 KiB of RAM on the board and combine it with the CIRAM already present in the console with a decoder chip in order to create linear accessible 4k block of RAM at $2000-$2FFF.
* Add a [6264](https://www.nesdev.org/wiki/6264_static_RAM) 8 KiB RAM on the board, replacing the CIRAM present in the console. This effectively "wastes" the CIRAM chip as a whole, 4 KB of (normally unused) extra memory at $3000-$3EFF. However this leads to a simpler, single-chip solution (in addition, 8 KiB RAM chips are today more common and less expensive than 2 KiB ones).
* Add a larger RAM on the board and map it to the entire PPU address space. This allows 8 KiB of pattern tables at $0000-$1FFF and 4 KiB of nametables at $2000-$2FFF sharing the same RAM chip.

### Other

* [Image omitted]

  Diagonal
* [Image omitted]

  L-shaped
* [Image omitted]

  3-screen vertical
* [Image omitted]

  3-screen horizontal
* [Image omitted]

  3-screen diagonal
* [Image omitted]

  1-screen fixed

Other uncommon types of mirroring are available in other boards, such as [TxSROM](https://www.nesdev.org/wiki/INES_Mapper_119) variations of the MMC3, extended techniques available to the [MMC5](https://www.nesdev.org/wiki/MMC5), arbitrary VRAM mirroring arrangements by the [Namco 163](https://www.nesdev.org/wiki/INES_Mapper_019), or ROM mirroring arrangements using mappers that allow [ROM nametables](https://www.nesdev.org/wiki/Category:Mappers_with_ROM_nametables).

**Diagonal mirroring** (CIRAM A10 = PA11 XOR PA10) would facilitate changes in scrolling direction without having to flip between Horizontal and Vertical mirroring.

**L-shaped mirroring** (CIRAM A10 = PA11 OR PA10), seen in Sachen's [374N](https://www.nesdev.org/wiki/INES_Mapper_243) and [8259 family](https://www.nesdev.org/wiki/Sachen_8259), allows scrolling in four directions as long as scrolling changes directions only at screen boundaries.

Unusual cases:

* *Castlevania 3* uses the third nametable RAM available on the [MMC5](https://www.nesdev.org/wiki/MMC5) (illustration)
* *Laser Invasion* uses the third nametable RAM available on the [MMC5](https://www.nesdev.org/wiki/MMC5)
* *After Burner* uses the ROM nametables available on the [Sunsoft-4](https://www.nesdev.org/wiki/INES_Mapper_068) (iNES 68), but it only supports 1/H/V arrangements.
* *Mighty Morphin Power Rangers III, IV (JY Company)* uses ROM nametables in a 3-Screen horizontal configuration, with the lower screen being used in the status bar.

### Mirroring chart

This table lists the more simple and easy to understand mirroring and scrolling techniques. There are a huge variety of more complicated techniques. For a more comprehensive survey, see: [List of games by mirroring technique](https://www.nesdev.org/wiki/List_of_games_by_mirroring_technique)

| Scrolling Type | Mirroring | Example Games | Comment |
|---|---|---|---|
| None | Any | *Donkey Kong*, *Tennis* | With only a single fixed screen, any mirroring type can be used. |
| Horizontal Only | Vertical | *Super Mario Bros.*, *Gimmick!* | A status bar at the top is easy to accomplish with a [sprite-0 hit](PPU_OAM.md#Sprite_zero_hits) (see *Super Mario Bros.*). |
| Vertical Only | Horizontal | *Ice Climber*, *Gun.Smoke* | Without a status bar, horizontal mirroring is the best choice for vertical-only scrolling. With a status bar, vertical or single-screen mirroring give you a place in the nametable to render the status bar, and the scrolling seam should be hidden under the bar. |
| Alternating Horizontal/Vertical | Mapper switches H/V | *Metroid*, *Air Fortress* | Motion is limited to a single axis at any given time, and the direction can only change when a new screen is reached. |
| Limited Bidirectional | Horizontal/Vertical | *Super Mario Bros. 3*, *Fire Emblem* | By limiting one of the scrolling axes to only 2-screens wide, this makes unlimited scrolling in the other axis simple. With unlimited horizontal scrolling there will be unavoidable attribute glitches at one side of the screen (see *Super Mario Bros. 3*), but with unlimited vertical scrolling this can be hidden by [overscan](Overscan.md) in NTSC regions (see *Fire Emblem*). |
| Unlimited Bidirectional | Various | *Castlevania II*, *Battletoads*, *Crystalis*, *Final Fantasy* | Unlimited scrolling in both axes at once is an advanced technique requiring a game-specific solution. |

The best way to understand the mirroring techniques used in a game, use a debugging emulator to look at the nametables. Status bars typically require a scrolling split at a timed location on the screen. This can be done most easily with a mapper based [IRQ](IRQ.md), but can also be accomplished with a [sprite-0 hit](PPU_OAM.md#Sprite_zero_hits) or other techniques.

## See also

* [PPU scrolling](PPU_scrolling.md)
