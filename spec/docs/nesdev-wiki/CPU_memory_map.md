# CPU memory map

> Source: <https://www.nesdev.org/wiki/CPU_memory_map> — NESdev Wiki, revision [21671](https://www.nesdev.org/w/index.php?oldid=21671) (last edited 2024-03-19T15:05:13Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=CPU_memory_map&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

<table>
<tr><th>Address range</th><th>Size</th><th>Device</th></tr>
<tr><td>$0000–$07FF</td><td>$0800</td><td>2 KB internal RAM</td></tr>
<tr><td>$0800–$0FFF</td><td>$0800</td><td rowspan="3"><a href="Mirroring.md#Memory_Mirroring">Mirrors</a> of $0000–$07FF</td></tr>
<tr><td>$1000–$17FF</td><td>$0800</td></tr>
<tr><td>$1800–$1FFF</td><td>$0800</td></tr>
<tr><td>$2000–$2007</td><td>$0008</td><td><a href="PPU_registers.md">NES PPU registers</a></td></tr>
<tr><td>$2008–$3FFF</td><td>$1FF8</td><td>Mirrors of $2000–$2007 (repeats every 8 bytes)</td></tr>
<tr><td>$4000–$4017</td><td>$0018</td><td><a href="APU.md">NES APU</a> and <a href="2A03.md">I/O registers</a></td></tr>
<tr><td>$4018–$401F</td><td>$0008</td><td>APU and I/O functionality that is normally disabled. See <a href="https://www.nesdev.org/wiki/CPU_Test_Mode">CPU Test Mode</a>.</td></tr>
<tr><td>$4020–$FFFF<br/><i>• $6000–$7FFF<br/>• $8000–$FFFF</i></td><td>$BFE0<br/><i>$2000<br/>$8000</i></td><td>Unmapped. Available for cartridge use.<br/><i>Usually cartridge RAM, when present.<br/>Usually cartridge ROM and <a href="Mapper.md">mapper</a> registers.</i></td></tr></table>

* Some parts of the 2 KiB of internal RAM at $0000–$07FF have predefined purposes dictated by the 6502 architecture:
  * $0000-$00FF: The zero page, which can be accessed with fewer bytes and cycles than other addresses
  * $0100–$01FF: The page containing the stack, which can be located anywhere here, but typically starts at $01FF and grows downward

:   Games may divide up the rest however the programmer deems useful. See [Sample RAM map](https://www.nesdev.org/wiki/Sample_RAM_map) for an example allocation strategy for this RAM. Most commonly, $0200-$02FF is used for the OAM buffer to be copied to PPU OAM during vblank.

* The unmapped space at $4020-$FFFF can be used by cartridges for any purpose, such as ROM, RAM, and registers. Many common mappers place ROM and save/work RAM in these locations:
  * $6000–$7FFF: Battery-backed save or work RAM (usually referred to as WRAM or PRG-RAM)
  * $8000–$FFFF: ROM and mapper registers (see [MMC1](MMC1.md) and [UxROM](UxROM.md) for examples)

:   The cartridge is able to passively observe reads from and writes to any address in the CPU address space, even outside this unmapped space, except for reads from $4015, the only readable register that is internal to the CPU. The cartridge can map writable registers anywhere, but its readable memory can only be placed where it does not interfere with other readable hardware, which would produce a [bus conflict](Bus_conflict.md). While cartridges can map readable memory at $4000-$4014 and $4018-$401F, a quirk in the 2A03's register decoding can cause DMA to misbehave if the CPU is halted while reading from $4000-$401F, so it is recommended that cartridges only map readable memory from $4020-$FFFF.

* If using [DPCM playback](APU_DMC.md), samples are limited to the following practical range:
  * $C000–$FFF1: DPCM sample data

:   Sample playback wraps around from $FFFF to $8000. The highest sample starting address is $FFC0 and longest sample is $FF1 bytes, so the full DPCM range is $C000-$FFFF and $8000-$8FB0, but making use of the wraparound is challenging because of banking and the presence of the CPU vectors.

* The CPU expects interrupt vectors in a fixed place at the end of the unmapped space:
  * $FFFA–$FFFB: NMI vector, which points at an [NMI](NMI.md) handler
  * $FFFC–$FFFD: Reset vector, which points at [code to initialize the NES chipset](https://www.nesdev.org/wiki/Init_code)
  * $FFFE–$FFFF: IRQ/BRK vector, which may point at a mapper's [interrupt](IRQ.md) handler (or, less often, a handler for APU interrupts)

:   These vectors are supplied by the cartridge. Unless a mapper fixes $FFFA–$FFFF to some known bank (normally by fixing an entire bank-sized region at the top of the address space, such as $C000-$FFFF, to a specific bank) or uses some sort of reset detection, the vectors (and a suitable reset code stub) must be present in all banks.

* Reading from memory that is not mapped to anything normally returns [open bus](Open_bus_behavior.md). The cartridge hardware may affect open bus behavior across the entire CPU address space, such as by pulling bits high or low.
