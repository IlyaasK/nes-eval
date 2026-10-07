# Myths

> Source: <https://www.nesdev.org/wiki/Myths> — NESdev Wiki, revision [23006](https://www.nesdev.org/w/index.php?oldid=23006) (last edited 2025-09-25T23:26:59Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=Myths&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

This page documents some **myths** about the NES hardware that originate in obsolete documents and emulators.

## NTSC picture height

**Myth:** The NTSC NES picture is only 224 pixels tall. (Seen in [nesfreq.txt](http://nesdev.org/nesfreq.txt), [a post by chaotic_thought on ngemu.com](http://ngemu.com/threads/true-resolution-of-psx-320x224.59392/#post-826552), and elsewhere)

**Fact:** The NTSC NES picture is 242 pixels tall: 240 lines of picture and 2 lines of vertical border.
The PPU fetches and generates a signal for all 240 lines, even if TVs [cut off the edges](Overscan.md).

Ideally, NES games place nametable [mirroring](Mirroring.md) glitches in the overscan, and some emulators simulate overscan in order to hide these glitches. For example, PocketNES for Game Boy Advance hides the top 16 pixels, the bottom 11, and 8 on the left and right sides. The Mega Man Anniversary Collection (GameCube/XBOX/PS2) likewise hides the top/bottom 8 lines, plus an extra left/right clipping to hide the attribute color glitch on scrolling. Also, the Wii Virtual Console (NES) does not display the top and bottom 8 lines.

## Scrolling registers

**Myth:** Usage of the [PPUADDR](PPU_registers.md#PPUADDR) ($2006) register is needed to scroll. (Seen in [Nestech.txt section 10: Programming the NES](https://www.nesdev.org/wiki/Nestech.txt#Programming_the_NES))

**Fact:** The proper way to [set the scroll position](PPU_scrolling.md) is to write the upper bits of the X and Y coordinates to [PPUCTRL](PPU_registers.md#PPUCTRL) ($2000), and then bits 0-7 of the X and Y coordinates to [PPUSCROLL](PPU_registers.md#PPUSCROLL) ($2005). The NES will update the VRAM address register near the end of the pre-render scanline (261 on NTSC, 311 on PAL).

PPUADDR ($2006) is needed to scroll only when changing the vertical part of the scroll position during rendering time. This could happen if rendering is turned on late to free up more VRAM update time, or the screen is split.

*Super Mario Bros.* appears to zero out PPUADDR after writing a buffer to PPUDATA, but this was later discovered to be a workaround for a rare [palette corruption](PPU_registers.md#Palette_corruption) bug instead.

## Mappers

### MMC4

**Myth:** The [MMC4](https://www.nesdev.org/wiki/MMC4) is used in the Japanese version of *Mike Tyson's Punch-Out!!* (source: [Nintendo Entertainment System Documentation v0.40 by Y0SHi](https://gamefaqs.gamespot.com/nes/916386-nes/faqs/2949))

**Fact:** The MMC4 is used only in three Japan-only games published by Nintendo in the *Famicom Wars* and *Fire Emblem* series. Known Japanese versions of *Mike Tyson's Punch-Out!!* use the same [MMC2](https://www.nesdev.org/wiki/MMC2) as their American and PAL counterparts.

### MMC5

**Myth:** The [MMC5](https://www.nesdev.org/wiki/MMC5) supports [memory for 4-screen nametables](Mirroring.md#4-Screen) (source: [Nintendo Entertainment System Documentation v0.40 by Y0SHi](https://gamefaqs.gamespot.com/nes/916386-nes/faqs/2949))

**Fact:** MMC5 uses the 2 KiB of RAM in the Control Deck for two nametables. It has its own 1 KiB ExRAM, and one of the four possible modes for ExRAM uses it as a third nametable. It also supports fill mode, a fourth limited-function nametable filled with 960 copies of a single tile number. So in total, while the MMC5 supports four different modes for each nametable, it does not support memory for the sort of 4-screen nametables seen in *Napoleon Senki* and *Gauntlet*.

### VRC6

**Myth:** The VRC6 is a very complex mapper even superior to the MMC5.

**Fact:** The [VRC6](https://www.nesdev.org/wiki/VRC6) is a decent mapper able to do standard PRG and CHR bankswitching, a CPU cycle counter, and 3 extra sound channels. The [MMC5](https://www.nesdev.org/wiki/MMC5) has extended video possibilities, a true scanline counter, and countless features that the VRC6 lacks, but only has 2 extra sound channels. Rumor has been made that the VRC6 was superior to the MMC5 because the MMC5 *Castlevania III: Dracula's Curse* was censored, and (like other 72-pin games) didn't use extra sound. But in fact, *Castlevania III* doesn't even come close to using all MMC5 capabilities, and it likely used MMC5 because it supported the VRC6's PRG ROM and CHR ROM bankswitching modes and was cheaper than Konami getting the VRC6 approved through Nintendo of America and Nintendo of Europe.

## Largest game

**Myth:** *Dragon Quest/Warrior IV* (DQ4) is the largest NES game, having 1 MiB (1,048,576 bytes) of ROM. (Source: [NES Technical/Emulation/Development FAQ version 1.4](http://nesdev.org/NESTechFAQ.htm#biggestgame) via [Reddit](https://www.reddit.com/r/todayilearned/comments/2qdz5q/til_the_biggest_offical_nes_game_ever_made_was/))

**Fact:** Both the Japanese *Dragon Quest IV* and the American *Dragon Warrior IV* releases use [SUROM](MMC1.md#SxROM_board_types), as pictured at [NesCartDB's entry for *Dragon Warrior IV*](http://bootgod.dyndns.org:7777/profile.php?id=1276). They use 512 KiB PRG ROM and 8 KiB CHR RAM, which is not larger than quite a few other licensed games. This rumor was due to a 1MiB overdump of *Dragon Quest IV* floating around.

Games larger than *Dragon Quest IV* include the following:

* The largest licensed Famicom game is *Metal Slader Glory* (512 KiB PRG + 512 KiB CHR + [MMC5](https://www.nesdev.org/wiki/MMC5)).
* The largest licensed NES game is *Kirby's Adventure* (512 KiB PRG + 256 KiB CHR + [MMC3](MMC3.md)).
* The largest unlicensed non-pirate NES game from the original era is *Action 52* (1,536 KiB PRG + 512 KiB CHR + [custom mapper](https://www.nesdev.org/wiki/INES_Mapper_228)).
* The largest unlicensed non-pirate NES production from the modern era is *A Winner Is You* (64 MiB), which is a music cartridge and not a game.
* One large "Hong Kong Original" port is *[Final Fantasy VII](https://en.wikipedia.org/wiki/Final_Fantasy_VII_(NES_video_game))* (2 MiB PRG + CHR RAM).
* Some pirate multicarts are 4 MiB or larger.

The largest games cannot be represented in the the original [iNES](INES.md) format, which has a practical limit of 2 MiB PRG ROM and 1 MiB CHR ROM. [NES 2.0](NES_2.0.md) should be used instead.

## Old programs

**Myth:** If a binary file has a .nes file extension, it will work as intended on an NES, and emulators should be tweaked to match how it is supposed to work.

**Fact:** No, especially older NES programs tend to have been tested only on bad emulators. Emulators should match the behavior of an NES or at least that of an accurate emulator. Dirty iNES headers might break it.

A lot of emulators, especially prior to about 2005, were based on incomplete knowledge of how the NES works. Some old demos expect all internal memory ($0000-$07FF) to be $00. Since then, public knowledge of the [quirks of the NES hardware behavior](Errata.md) has grown, and emulators such as Mesen and Nintendulator more faithfully reproduce the misbehaviors in sloppy or [cargo-cult-programmed](https://en.wikipedia.org/wiki/Cargo_cult_programming) code.
See [Program Compatibility](https://www.nesdev.org/wiki/Program_compatibility) for a list of homebrew known to have problems on an NES.

## Old tutorials

**Myth:** [GBAGuy's NES tutorial](http://patater.com/gbaguy/nesasm.htm) is worth following.

**Fact:** Old tutorials like these are full of cargo-cult programming because the authors apparently didn't fully understand the hardware. For example, this tutorial in particular treats the [OAM address](PPU_registers.md) register as 16-bit (just like [PPUADDR](PPU_registers.md#PPUADDR)) and attempts to initialize variables in [system RAM](https://en.wikipedia.org/wiki/.bss) using .db statements - a lot of the programs don't even work on a NES. [NES 101](http://nesdev.org/NES101.zip) and [Nerdy Nights](https://nerdy-nights.nes.science) are considered better. Even the webmaster of Patater.com now recommends Nerdy Nights.

## PPU details

**Myth:** [OAMADDR](PPU_registers.md#OAMADDR) ($2003) must be cleared on VBlank ending.

**Fact:** The PPU itself sets the sprite address to zero at the end of VBlank, but due to a design flaw it can result in minor sprite RAM corruption if it was nonzero beforehand - in particular, it can cause values from one 8-byte "page" of sprite RAM to leak into another due to its lack of proper memory refreshing. Some Chinese games actually rely on this behavior and will lock up otherwise.

**Myth:** "There is 16k of internal VRAM" (source [Nestech.txt 0.40 by Y0shi](https://www.gamefaqs.com/nes/916386-nes/faqs/2949))

**Fact:** The PPU has an address range of 14 bits addressing 16 KiB, but there's only 2 KiB of internal VRAM, typically used for nametables. This was corrected to "16 kbits" in later editions of [Nestech.txt](https://www.nesdev.org/wiki/Nestech.txt).

## Color emphasis

**Myth:** Enabling more than one [color emphasis](https://www.nesdev.org/wiki/Colour_emphasis) bit at once will damage the PPU, or at least cause the TV to lose sync. (Source: [Nintendo Entertainment System Architecture](http://fms.komkon.org/EMUL8/NES.html) by Marat)

**Fact:** Enabling multiple color emphasis bits is perfectly safe - in fact, [some licensed games](https://www.nesdev.org/wiki/Colour-emphasis_games) including *Felix the Cat* and *Just Breed* enable all of them simultaneously to dim the screen. On the other hand, enabling all emphasis bits results in an unreadable white screen on an RGB PPU, such as that in the Famicom Titler or the [Sharp C1](https://www.nesdev.org/wiki/Sharp_C1) (Famicom TV). Worse, setting the PPU into *slave* mode (by setting the *master/slave* bit in [PPUCTRL](PPU_registers.md#PPUCTRL)) is theoretically capable of causing actual damage, as it results in high current draw from the EXT pins (due to them trying to output +5V despite being wired to GND).
