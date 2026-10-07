# Tricky-to-emulate games

> Source: <https://www.nesdev.org/wiki/Tricky-to-emulate_games> — NESdev Wiki, revision [24361](https://www.nesdev.org/w/index.php?oldid=24361) (last edited 2026-10-06T19:24:17Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=Tricky-to-emulate_games&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

At the very least the following games depend on hard-to-emulate or just obscure behavior. (If you're looking for a good first game for your new emulator, try anything made in 1984 or earlier, such as *Donkey Kong*.)

*Abarenbou Tengu* (J), *Captain Tsubasa 2* (J), *Noah's Ark* (E), *Rampart* (U), *Zombie Nation* (U)
:   These refer to CHR ROM banks outside their size. A CHR ROM of the correct size will wrap the addresses correctly by discarding the most significant bits, as do most emulators. But if you are developing a flash-cart that just pre-programs its flash/SRAM memory with the CHR ROM data without address wrapping, graphical bugs will happen. If you want to simulate that behaviour in emulator, increase CHR-ROM banks number in iNES header twice and paste zeros in the the CHR area.

*Adventures of Lolo 2*, *Ms. Pac-Man* (Tengen), and *Spelunker*
:   These games rely on proper timing for when the CPU polls for interrupts. When NMI becomes enabled while the vblank flag is already set, the resulting NMI occurs late enough in the instruction that another instruction is able to execute before the NMI is serviced.

*Air Fortress* (J)
:   Expects RAM to be enabled at power-up, as it clears WRAM before enabling it ($E000 D4). If MMC1 powers up with RAM disabled, the values written in the init routine go nowhere. If RAM is not enabled is used and the RAM's power-up value is anything but $00, unwanted color emphasis gets applied until the reset button is pressed. (The North American version enables WRAM first.)

*Arkista's Ring*
:   Crashes after completing the first loop if a read from PPU open bus returns 1 on bit 6, which is 0 unless using a mod such as NESRGB or Hi-Def NES that uses EXT output.

*Astyanax*
:   The round intro screen inadvertently attempts to reload the [MMC3 scanline counter](MMC3.md#IRQ_Specifics) with only one timer clock between $C001 writes (due to a [palette RAM](PPU_palettes.md#Palette_RAM) transfer toggling PPU A12 while rendering is disabled). It has been observed to cause the status bar handler to miss one frame on an NEC MMC3B cartridge, leading to PRNG desynchronizations in Tool-Assisted Speedruns. The exact counter behaviour appears to depend on the ASIC manufacturer and revision, and hardware research is ongoing.[[1]](#cite_note-1)

*Balloon Fight* and *Mario Bros.*
:   These games read the [PPU nametables](PPU_nametables.md) through [PPUDATA](PPU_registers.md#PPUDATA) ($2007). *Balloon Fight* uses it to determine the current tile of each star in the background when twinkling them. (The code is at $D603.) *Mario Bros.* uses it for background collision detection. The scroll split in the "Balloon Trip" mode of *Balloon Fight* also depends to an extent on the correct number of CPU cycles from the start of NMI to the start of display, but it's not particularly picky.

*Bandit Kings of Ancient China*, *Gemfire*, *L'Empereur*, *Nobunaga's Ambition II*, *Romance of the Three Kingdoms II*, and *Uncharted Waters*
:   Koei's MMC5 RPGs and strategy games use 8×8-pixel attributes and large work RAM.

*Bases Loaded II*
:   The screen glitches after a pitch is thrown (screenshot) if writing $00 then $80 to [PPUCTRL](PPU_registers.md#PPUCTRL) during vertical blank does not cause an additional [NMI](NMI.md).

*Batman: Return of the Joker*, *Dragon Quest*, and *Milon's Secret Castle*
:   These read level data and control logic from CHR ROM. The $2007 read must take into account not only the 1-byte delay (see entry for *Super Mario Bros.*) but also CHR bank switching. *Batman: RotJ* also executes code from PRG RAM.

*Battletoads*
:   Infamous among emulator developers for requiring fairly precise CPU and PPU timing (including the cycle penalty for crossing pages) and a fairly robust sprite 0 implementation. Because it continuously streams animation frames into CHR RAM, it leaves rendering disabled for a number of scanlines into the visible frame to gain extra VRAM upload time and then enables it. If the timing is off so that the background image appears too high or too low at this point, a sprite zero hit will fail to trigger, hanging the game. This usually occurs immediately upon entering the first stage if the timing is off by enough, and might cause random hangs at other points otherwise.

*Battletoads & Double Dragon* and *Low G Man*
:   They read from WRAM at $6000–$7FFF despite there being none on the cartridge, relying on the values produced by [open bus behavior](Open_bus_behavior.md).[[2]](#cite_note-2) Additionally, *LGM* disables WRAM through $A001, which some emulators disregard in order to kludge [MMC6](https://www.nesdev.org/wiki/MMC6) games into working. If WRAM is present and enabled, some pre-loaded values will cause *BT&DD* to crash at the end of stage 1 when Abobo makes his first appearance and *LGM* to crash when playing boss music.[[3]](#cite_note-3)

*Bee 52*
:   This needs accurate DMC timing and relies on [PPUSTATUS](PPU_registers.md#PPUSTATUS) bit 5 (sprite overflow) as well. Also updates the color palette mid-frame.

*Bill & Ted's Excellent Adventure* and some other [MMC1](MMC1.md) games
:   These depend on the mapper ignoring successive writes; see [iNES Mapper 001](MMC1.md) (the talk page for that page might be informative too). *Bill & Ted* also turns off and re-enables rendering midframe to switch CHR banks (such as in the black border above dialog boxes).

*Burai Fighter* (U)
:   It accesses [PPUDATA](PPU_registers.md#PPUDATA) during rendering to draw the scorebar. Incorrect emulation clips the scorebar to half size. See the notes on accessing [PPUDATA](PPU_registers.md#PPUDATA) during rendering on the [PPU scrolling](PPU_scrolling.md) page.

*B-Wings*, *Fantasy Zone II*, *Demon Sword* / *Fudō Myōō Den*, *Fushigi no Umi no Nadia*, *Krusty's Fun House*, *Trolls in Crazyland*, *Over Horizon*, *Super Xevious: GAMP no Nazo*, and *Zippy Race*
:   They write to CHR ROM and expect the writes to have no effect.[[4]](#cite_note-4)[[5]](#cite_note-5)

*Captain Planet*, *Dirty Harry*, *Infiltrator*, *Mad Max*, *Paperboy*, *The Last Starfighter*
:   Mindscape games which rely on the [open bus](Open_bus_behavior.md) behavior of controller reads and expects them to return exactly 0x40 or 0x41; see [Standard controller](Standard_controller.md).

*Cobra Triangle* and *Ironsword: Wizards and Warriors II*
:   They rely on the dummy read for the `sta $4000,X` instruction to acknowledge pending APU IRQs.

*Crystalis*
:   Uses MMC3 scanline for a moving vertical split to wrap the playfield while accommodating the status bar. Incorrect MMC3 timing will create a moving seam as you wander up and down the map.

*Crystalis*, *Fantastic Adventures of Dizzy*, *Fire Hawk*, *Indiana Jones and the Last Crusade*, *StarTropics*, and *Super Off Road*
:   These do [mid-frame palette changes](https://www.nesdev.org/wiki/Palette_change_mid_frame).

*Cybernoid: The Fighting Machine*
:   Relies on [bus conflicts](Bus_conflict.md) and uninitialized RAM state - see the [Game bugs](https://www.nesdev.org/wiki/Game_bugs) page for details. Also uses [color $0D](https://www.nesdev.org/wiki/Color_$0D_games) and [color emphasis](https://www.nesdev.org/wiki/Colour_emphasis).

*Daydreamin' Davey*
:   Changes the background palette color mid-frame; also does an OAM DMA for its sprite-only status bar below the split. (See also: Stunt Kids.)

*Devil World*
:   Writes to [PPUADDR](PPU_registers.md#PPUADDR) and [PPUSCROLL](PPU_registers.md#PPUSCROLL) mid-frame for scroll splits during gameplay, using a combination of cycle-timed code (for the maze portion) and sprite 0 hit (for the bottom status area). The initial scroll (for the top portion of the status area) is also set this way outside of vblank, as most of it is already spent reading nametable data through [PPUDATA](PPU_registers.md#PPUDATA).

*Door Door*
:   Writes to the PRG-ROM area frequently through a pointer explicitly set to $8080. Harmless on NROM. Additionally, the game polls controller input multiple times per frame in some instances (such as the Title screen or when pausing) due to the code causing the CPU to spinwait on reading the controller input.

*Final Fantasy*, *River City Ransom*, *Apple Town Story*[[6]](#cite_note-6), *Impossible Mission II*[[7]](#cite_note-7) amongst others
:   Use the semi-random contents of RAM on powerup to seed their RNGs. Emulators often provide fully deterministic emulated powerup state, and these games will seem to be deterministic when they weren't intended to be.

*Fire Hawk*, *Mig 29 Soviet Fighter*, and *Time Lord*
:   These need accurate DMC timing because they [abuse APU DMC IRQ](APU_DMC.md#Usage_of_DMC_for_syncing_to_video) to split the screen.

*Galaxian*
:   Requires proper handling of bit 4 of the [P register](Status_flags.md) for /IRQ.

*G.I. Joe* and *Mickey in Letterland*
:   These turn [sprite display](PPU_registers.md#PPUMASK) off and leave the background on. Correct timing of MMC3 IRQs requires that the sprite fetches still clock the scanline counter when either is enabled.[[1]](https://forums.nesdev.org/viewtopic.php?f=3&t=14103)

*Graphic Editor Hokusai*, *Jingorou*, *Kosodate Gokko*, and *Quick Hunter*
:   These are unlicensed [FDS](https://www.nesdev.org/wiki/Family_Computer_Disk_System) disk utilities/copiers which operate on arbitrary disks. They also require correct parsing of non-standard [disk images](https://www.nesdev.org/wiki/FDS_disk_format) and accurate handling of low-level disk drive accesses to avoid triggering copy protection measures.[[8]](#cite_note-8) This is particularly problematic when using the popular [FDS file format](https://www.nesdev.org/wiki/FDS_file_format), which omits gaps and CRCs.

*Huge Insect*
:   Depends on OAM corruption present on PPU revision E and later; see [OAMADDR precautions](PPU_registers.md#OAMADDR_precautions).[[9]](#cite_note-9)

*Ishin no Arashi*
:   Plays sound effects through [MMC5 pulse channels](https://www.nesdev.org/wiki/MMC5_audio) and times them using $5015 reads.

*Jurassic Park*
:   The wobbling OCEAN logo on the title screen is very sensitive to slight delay in the [MMC3](MMC3.md) IRQ and could have incorrectly scrolled lines if mistimed.

*Kira Kira Star Night DX*
:   Entering the code to access the debug screen gives the CPU slightly less than a single frame to load the entire menu tilemap. If the timing is significantly off, the menu will appear glitched. Additionally, the main gameplay backgrounds rely on correct MMC3 timing, or else the parallax effects appear glitched.

*The Legend of Zelda*
:   Writes to [PPUADDR](PPU_registers.md#PPUADDR) midframe to [set the coarse Y scroll](PPU_scrolling.md#$2006_(PPUADDR)_second_write_(w_is_1)) for vertical scrolling between screens. See the [Game bugs](https://www.nesdev.org/wiki/Game_bugs) page for buggy behavior arising from its implementation.

*The Magic of Scheherazade*
:   It maps two non-contiguous PRG ROM pages next to each other, then executes code across the page boundary. Emulators which use pointers to fetch sequential instruction bytes from ROM will fail when taking damage in the RPG-style battles. (Use password `5W` to test this easily.)

*Marble Madness*, *Mother* (J), and *Pirates*
:   These switch CHR banks mid-scanline to draw text boxes (such as at the beginning of each *MM* level). Getting these to render correctly requires fairly precise timing.

*Micro Machines*
:   Requires correct values when reading [OAMDATA](PPU_registers.md#OAMDATA) ($2004) during rendering, and also relies on proper background color selection when rendering is disabled and the VRAM address points to the palette (see the "background palette hack" on [PPU palettes](PPU_palettes.md)).

*Punch-Out!!*
:   MMC2 snoops PPU fetches. If the PPU does not fetch the 34th tile, the ring will be glitched.

*Puzznic* and *Reflect World* (FDS)
:   These use [unofficial](https://www.nesdev.org/wiki/Programming_with_unofficial_opcodes#Watermarking_instructions) opcode $89, which is a two-byte NOP on 6502 and BIT #imm on 65C02. ([Puzznic tasvideos discussion](http://tasvideos.org/forum/viewtopic.php?p=306520#306520)) The instruction in *Puzznic* is $89 $00; emulating $89 as a single-byte NOP will trigger a BRK that [causes the screen to shake](http://tasvideos.org/forum/viewtopic.php?p=306559#306559).

*Rollerblade Racer*
:   Has an unusual status bar using only sprites with the background disabled. (See also: *Daydreamin' Davey*.)

*Shinsenden*
:   Depends on the MMC1 reset bit (bit 7) *not* being ignored on successive writes, unlike the data bit (bit 0). Selecting the 4th option of the command menu ("みる", "look") accidentally triggers illegal instruction $7F, an RMW instruction which in this case first writes with bit 7 clear and then bit 7 set. The second write triggers a mapper reset that prevents the game from crashing.

*Slalom*
:   Does a JSR while the stack pointer is 0, so that half of the return address ends up at $0100 and the other half at $01FF. Also executes code from RAM.

*Solar Jetman*
:   Enables the decimal bit through manipulation of a [flag byte](Status_flags.md) pushed to the stack and expects addition to continue to operate in binary.

*Space Shuttle Project*
:   Performs [X/Y scroll splits](PPU_scrolling.md#Split_X/Y_scroll) during some scenes (but X scroll is seemingly always 0) by writing to [PPUADDR](PPU_registers.md#PPUADDR) and [PPUSCROLL](PPU_registers.md#PPUSCROLL) mid-frame, along with [PPUCTRL](PPU_registers.md#PPUCTRL) to switch BG pattern table access.[[10]](#cite_note-10) (Note: Let the title demo run to see these)

*Spot* and *Quattro Sports*
:   These poll input multiple times per frame, and may not respond to emulated input that can only change at one specific time during the frame[[11]](#cite_note-11)[[12]](#cite_note-12). Emulators generally don't have the option to poll the controller many times per frame in real-time, so the solutions used may need to compromise (e.g. game-specific solution, user option to decide when during the frame input changes, non-deterministic input change time). Tepples created a test ROM to explore this behaviour: [Telling LYs?](https://forums.nesdev.org/viewtopic.php?t=18998)

*Star Trek – 25th Anniversary*
:   Forces MMC3 to fire IRQ at scanline 0 which on some MMC3 versions or flashcarts causes glitching during split-screen scenes.

*StarTropics*
:   Disables rendering at the top of the status bar to change palettes, but also re-enables sprites when rendering comes back on. For hardware mapper emulation, the specific timing is critical. If the MMC3 IRQ timing is delayed by a cycle or two, this will begin to cause all sprites to flicker erratically. Even with a correctly timed hardware implementation, there are still some subtle corruption interactions to do with [turning rendering off, then re-enabling sprites mid-frame](PPU_sprite_evaluation.md#Rendering_disable_or_enable_during_active_scanline) which might not be possible to emulate correctly with current knowledge[[13]](#cite_note-13).

*Stunt Kids*
:   Mid-frame OAM DMA for split-screen gameplay.

*Sunsoft Tile Editor (Kakeru-kun)*, *Sunsoft Map Editor (Chizuko-san)*
:   Relies on graphics that are already loaded from the BIOS boot sequence (such as the menu font), and relies on loading and saving of arbitrary disk images, which may not be properly formatted, contain a "KYODAKU-" boot file, or are completely blank. Additionally, it assumes it can fully or partially wipe, format, and rewrite the entire disk contents, and not just single files.

*Super Mario Bros.*
:   This is [probably the hardest game to emulate](https://forums.nesdev.org/viewtopic.php?p=22022#22022) among the most popular [NROM](NROM.md) games, which are generally the first targets against which an emulator author tests their work. It relies on JMP indirect, correct palette mirroring (otherwise the sky will be black; see [PPU palettes](PPU_palettes.md)), sprite 0 detection (otherwise the game will freeze on the title screen), the 1-byte delay when reading from CHR ROM through [PPUDATA](PPU_registers.md#PPUDATA) (see [The PPUDATA read buffer](PPU_registers.md#The_PPUDATA_read_buffer_(post-fetch))), proper behavior of the nametable selection bits of [PPUSTATUS](PPU_registers.md#PPUSTATUS) and [PPUADDR](PPU_registers.md#PPUADDR), and correct handling of multiple controllers (otherwise the game will refuse to progress past the title screen[[14]](#cite_note-14)). In addition, there are several bad dumps floating around, some of which were ripped from pirate multicarts whose cheat menus leave several key parameters in RAM.

*Super Mario Bros. 3*, *Mystery World Dizzy*, *Double Action 53*, and *Haunted: Halloween '86: The Curse of Possum Hollow*
:   This relies on an interaction between the [sprite priority](PPU_sprite_priority.md) bit and the OAM index to put sprites behind the background. *SMB3* uses it for powerups sprouting from blocks. *Mystery World Dizzy* puts Dizzy behind a blue pillar ([screenshot](https://forums.nesdev.org/viewtopic.php?p=210736#p210736)). *RHDE: Furniture Fight* in *DA53* uses it for characters behind furniture. *HH86* uses it when Donny or Tami passes behind a telephone pole or steps into polluted water.

*Teenage Mutant Ninja Turtles*
:   Uses Y scroll values greater than 239, causing the PPU to read the attribute table as nametable data before looping back to the same nametable instead of rolling to the next nametable down.

*Time Lord*
:   This is sensitive to the power-on state of the NES. The Vblank flag in PPUSTATUS must be set for the first time within 240 scanlines, otherwise there will be a frame IRQ which is never acknowledged, which will mess up the DMC IRQs used elsewhere and cause the game to crash.

*Tonkachi Editor*
:   This is an unlicensed hex editor which can view and edit files on arbitrary disks (known for bundling the earliest *Super Mario Bros.* hack commonly referred to as "Tonkachi Mario"). It manually polls the byte transfer flag from $4030.D7 for disk accesses instead of using BIOS routines or IRQs.

*Ultimate Stuntman*, *Skate or Die 2*
:   *Ultimate Stuntman* plays PCM drum samples on the DMC channel during idle portions of the frame. *Skate or Die 2* does it on the title screen. (See also: *Battletoads* introduction.)

*Wario's Woods*
:   Uses [MMC3](MMC3.md) IRQ with unusual configuration of BG using CHR page $1000 and sprites using CHR page $0000. On some [CPU-PPU alignments](PPU_frame_timing.md#CPU-PPU_Clock_Alignment) (assigned randomly at reset), [the IRQ receives an extra clock on every second frame](MMC3.md#IRQ_Specifics), causing the last 48 pixels of the green ground to flicker, but not on all resets[[15]](#cite_note-15).

*Wizards and Warriors 3*
:   It writes new tile graphics for the sprites at the screen split after the sprites have been drawn, but before the frame has ended. Emulators which draw the sprites all at once using graphics data from the end of the frame will have glitches in the main character's sprite.

*The Young Indiana Jones Chronicles* and *Zelda II: The Adventure of Link*
:   These access [PPUDATA](PPU_registers.md#PPUDATA) during rendering to perform a glitchy y scroll. *Young Indy* uses it to make the screen shake when cannonballs hit the ground, and *Zelda II* uses it to skip scanlines on the title screen. See the notes on accessing [PPUDATA](PPU_registers.md#PPUDATA) during rendering on [PPU scrolling](PPU_scrolling.md) page.

## Troubleshooting games

If a scroll split doesn't work, and a garbage sprite shows up around the intended split point, then the game is probably trying to use a sprite 0 hit, but either the wrong tile data is loaded or the background is scrolled to a position that doesn't overlap correctly.
This could be a problem with nametable [mirroring](Mirroring.md), with CHR bankswitching in [mappers](Mapper.md) that support it, or with the CPU and PPU timing of whatever happened above the split. *Battletoads*, for one, uses 1-screen mirroring and requires exact timing to get the background scroll position dead-on.

## See also

* [Game bugs](https://www.nesdev.org/wiki/Game_bugs): These games have glitches on NES hardware, so don't go "fixing" them while breaking your emulator.
* [Sprite overflow games](https://www.nesdev.org/wiki/Sprite_overflow_games)
* [Unofficial opcode games](CPU_unofficial_opcodes.md#Games_using_unofficial_opcodes)
* [List of games that run code from outside of PRG-ROM](https://www.nesdev.org/wiki/List_of_games_that_run_code_from_outside_of_PRG-ROM)

## References

1. [↑](#cite_ref-1) [forum thread](https://forums.nesdev.org/viewtopic.php?t=26832): MMC3 $C001 write glitch example: Astyanax
2. [↑](#cite_ref-2) [thread](https://forums.nesdev.org/viewtopic.php?p=184535#p184535): Battletoads Double Dragon Powerpak Freeze
3. [↑](#cite_ref-3) [thread](https://forums.nesdev.org/viewtopic.php?p=184539#p184539): Battletoads Double Dragon Powerpak Freeze - pre-loading $FF reliably crashes *BT&DD*, $00 reliably does not.
4. [↑](#cite_ref-4) [thread](https://forums.nesdev.org/viewtopic.php?p=192541#p192541): KrzysioCart - Home made cartridge that support>80% NES games
5. [↑](#cite_ref-5) [thread](https://forums.nesdev.org/viewtopic.php?p=211244#p211244): Hong Kong 212 PCB,128+1024?
6. [↑](#cite_ref-6) <http://forums.nesdev.org/viewtopic.php?p=183064#p183064>
7. [↑](#cite_ref-7) <http://forums.nesdev.org/viewtopic.php?f=3&t=18111>
8. [↑](#cite_ref-8) [thread](https://forums.nesdev.org/viewtopic.php?t=18777): Copy protection methods on FDS disks
9. [↑](#cite_ref-9) [forum thread](https://forums.nesdev.org/viewtopic.php?t=12407): Huge Insect does not fully start
10. [↑](#cite_ref-10) [forum thread](https://forums.nesdev.org/viewtopic.php?t=26650): Split X/Y scrolling in Space Shuttle Project
11. [↑](#cite_ref-11) [forum thread](https://forums.nesdev.org/viewtopic.php?t=16355): Spot bug in Mesen and Nintaco
12. [↑](#cite_ref-12) [forum thread](https://forums.nesdev.org/viewtopic.php?f=5&t=1798): Quattro Sports BMX Simulator uses extra controller?
13. [↑](#cite_ref-13) [forum thread](https://forums.nesdev.org/viewtopic.php?p=283981#p283981): OAM corruption in StarTropics
14. [↑](#cite_ref-14) [forum thread:](https://forums.nesdev.org/viewtopic.php?t=26186) Diagnosing controller signals with *Super Mario Bros.*
15. [↑](#cite_ref-15) [forum post:](https://forums.nesdev.org/viewtopic.php?p=284036#p284036) discussion of Wario's Woods behaviour on real hardware

## External links

* [Software Index](https://www.smspower.org/Development/Software-Index) on SMS Power
* [Tricky-to-emulate games](https://gbdev.gg8.se/wiki/articles/Tricky-to-emulate_games) on GbdevWiki
* [ACCURACY.md](https://github.com/nba-emu/NanoBoyAdvance/blob/master/docs/ACCURACY.md#game-compatibility) from NanoBoyAdvance docs
