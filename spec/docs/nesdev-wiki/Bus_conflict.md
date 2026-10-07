# Bus conflict

> Source: <https://www.nesdev.org/wiki/Bus_conflict> — NESdev Wiki, revision [24010](https://www.nesdev.org/w/index.php?oldid=24010) (last edited 2026-07-04T17:50:48Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=Bus_conflict&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

A **bus conflict** occurs when two logic devices output different values on the same [bus](https://en.wikipedia.org/wiki/Bus_(computing)) line. When two signals are asserted at the same voltage, the one with less impedance generally wins. In the NES, this generally happens when a program writes to a [mapper](Mapper.md) whose registers overlap ROM but the ROM does not shut off its output, causing a potential conflict on the PRG data bus. Most ASIC based mappers include logic to disable the ROM's output enable during writes, putting the ROM's outputs into a high-impedance state and preventing the bus conflict. But many mappers, especially [discrete logic mappers](https://www.nesdev.org/wiki/Category:Discrete_logic_mappers), do not.

## Programming around bus conflicts

If you are using a mapper with bus conflicts, make sure that all devices on the bus are asserting the same value by writing to a ROM location that already contains the value that you are writing. For instance, to switch to bank 5 in [UNROM or UOROM](UxROM.md), write a 5 to a ROM location that already contains a 5.

One common way to do this is to perform an immediate load and then store over the opcode:

```
@loadInstruction:
  ldy #5
  sty @loadInstruction+1
```

To switch to a bank based on the value of a variable, put it in an indexed register and then perform an absolute indexed store:

```
  lda curMapBank
  tax
  sta bankBytes,x
; ...
bankBytes:
  .byt $00, $01, $02, $03, $04, $05, $06, $07
```

## Emulating bus conflicts

Many emulators have incorrectly assumed that the CPU "wins" all bus conflicts; that is, that the mapper circuitry sees the signals from the CPU more strongly than the signals from the PRG ROM and acts solely on the CPU. Quite a few early programs in [iNES](INES.md) format were developed without taking bus conflicts into account and do not work correctly when run on real hardware. In general, the authors of these programs did not know at the time that bus conflicts existed. These programs can, however, be made to run by adding ROM-disabling circuitry like that of [ANROM](AxROM.md) or the positive chip enable of the PRG ROM chips used with [AOROM](AxROM.md). [NES 2.0 submappers](https://www.nesdev.org/wiki/NES_2.0_submappers) can be used to specify that these programs must be run without bus conflicts.

The following classes of iNES files will often contain bugs causing bus conflicts:

* Old homebrew ROMs: Old documents did not mention the possibility of bus conflicts.
* Mapper hacks: Early RAM cartridges, such as those by Front Fareast, implement [mapper 2](UxROM.md) and [mapper 3](CNROM.md) without bus conflicts. Several games on the FFE CD are ports of [MMC1](MMC1.md) games to this mapper 2 variant; GoodNES lists these with `[hM02]`. For this and other reasons (the code to operate an MMC1 was slightly larger than that for mapper 2), early English translations of the Famicom game *Final Fantasy II* were likewise made to run on this mapper 2 variant.
* Buggy games, homebrew, or hacks: Code or tables to avoid bus conflicts may have been written incorrectly, or a jump may have sent the program counter to somewhere that isn't code.

It has been confirmed through [testing](http://forums.nesdev.org/viewtopic.php?p=109708#p109708) that both the CPU and the mask ROMs used in the NES era drive a 0 more strongly than a 1, as one would expect based on the logic's implementation. Sunsoft UNROM games *Shanghai II* and *Pescatore* attempt to work around AND-type bus conflict this way by writing the bankswitch register at $C000 that contains the value $07.
This implies that an emulator should use the bitwise AND of the value from the CPU and the value from the ROM. However, programmers must not rely on this undefined behavior. Logging a warning when emulating a bus conflict can help modern developers identify bugs in their games and potentially help debug issues in an emulator's PRG ROM bank switching.

## See also

* [Mappers with bus conflicts](https://www.nesdev.org/wiki/Category:Mappers_with_bus_conflicts)
* [Open bus](Open_bus_behavior.md) - The opposite condition where nothing is currently trying to output to the bus.
