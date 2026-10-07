# PPU

> Source: <https://www.nesdev.org/wiki/PPU> — NESdev Wiki, revision [23576](https://www.nesdev.org/w/index.php?oldid=23576) (last edited 2026-03-06T08:25:59Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=PPU&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The NES PPU, or Picture Processing Unit, generates a composite video signal with 240 lines of pixels, designed to be received by a television. When the Famicom chipset was designed in the early 1980s, it was considered quite an advanced 2D picture generator for video games.

It has its own address space, which typically contains 10 kilobytes of memory: 8 kilobytes of ROM or RAM on the Game Pak (possibly more with one of the common [mappers](Mapper.md)) to store the shapes of background and sprite tiles, plus 2 kilobytes of RAM in the console to store a map or two. Two separate, smaller address spaces hold a palette, which controls which colors are associated to various indices, and OAM (Object Attribute Memory), which stores the position, orientation, shape, and color of the sprites, or independent moving objects. These are internal to the PPU itself, and while the palette is made of static memory, OAM uses dynamic memory (which will slowly decay if the PPU is not rendering).

### Programmer's reference

* [Registers](PPU_registers.md)
* [Pattern tables](PPU_pattern_tables.md) (tile graphics for background and sprites)
* Background graphics
  * [Nametables](PPU_nametables.md)
  * [Attribute tables](PPU_attribute_tables.md)
* [OAM](PPU_OAM.md) (sprites)
* [Palettes](PPU_palettes.md)
* [Memory map](PPU_memory_map.md)

* **[All PPU documentation on one page](https://www.nesdev.org/wiki/PPU_programmer_reference)** (printer-friendly)

### Hardware behaviors

* [Frame timing](PPU_frame_timing.md)
* [Power up state](PPU_power_up_state.md)
* [NMI](NMI.md)
* [Clock rate](Cycle_reference_chart.md#Clock_rates) and other NTSC/PAL/Dendy differences
* [NTSC video](NTSC_video.md)
* [Scrolling](PPU_scrolling.md)
* [Rendering](PPU_rendering.md)
* [Sprite evaluation](PPU_sprite_evaluation.md)
* [Sprite priority](PPU_sprite_priority.md)
* [PPU signals](https://www.nesdev.org/wiki/PPU_signals)
* [Overscan](Overscan.md)
* [PPU pinout](https://www.nesdev.org/wiki/PPU_pinout)
* NTSC PPU frame timing diagram
* [Visual 2C02](https://www.nesdev.org/wiki/Visual_2C02): A hardware-level PPU simulator
* [List of known PPU versions and variants](PPU_variants.md)

### Notes

* The [NTSC video](NTSC_video.md) signal is made up of 262 scanlines, and 20 of those are spent in vblank state. After the program has received an NMI, it has about 2270 cycles to update the palette, sprites, and nametables as necessary before rendering begins.
* On NTSC systems, the PPU divides the master clock by 4 while the CPU uses the master clock divided by 12. Since both clocks are fed off the same master clock, this means that there are **exactly** three PPU ticks per CPU cycle, with no drifting over time (though the clock alignment might vary depending on when you press the Reset button).
* On PAL systems, the PPU divides the master clock by 5 while the CPU uses the master clock divided by 16. As a result, there are exactly 3.2 PPU ticks per CPU cycle.

### See also

* [2C02 technical reference](http://nesdev.org/2C02%20technical%20reference.TXT) by Brad Taylor. (Pretty old at this point; information on the wiki might be more up-to-date.)
