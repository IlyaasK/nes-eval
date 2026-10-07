# CPU unofficial opcodes

> Source: <https://www.nesdev.org/wiki/CPU_unofficial_opcodes> — NESdev Wiki, revision [23975](https://www.nesdev.org/w/index.php?oldid=23975) (last edited 2026-06-29T14:07:55Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=CPU_unofficial_opcodes&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

**Unofficial opcodes**, sometimes called **illegal opcodes** or **undocumented opcodes**, are [CPU instructions](6502_instructions.md) that were officially left unused by the original design. The 6502 family datasheet from MOS Technology does not specify or document their function, but they actually do perform various operations.

Some of these instructions are useful; some are not predictable; some do nothing but burn cycles; some halt the CPU until reset.
Most NMOS 6502 cores interpret them the same way, although there are slight differences with the less stable instructions.
CMOS variants of the 6502 handle them completely differently, and later CPUs in the same family (e.g. 65C02, HuC6280, 65C816) were free to implement new instructions in the place of the unused ones.

An [accurate](https://www.nesdev.org/wiki/Accuracy) NES emulator must implement all instructions, not just the official ones. A small number of games use them (see [below](#Games_using_unofficial_opcodes)).

## Arrangement

The microcode of the 6502 is compressed into a 130-entry decode ROM.
Instead of 256 entries telling how to process each separate opcode, it's encoded as combinational logic post-processing the output of a "sparse" ROM that acts in some ways like a programmable logic array (PLA).
Each entry in the ROM means "if these bits are on, and these bits are off, do things on these six cycles."[[1]](#cite_note-1)

Many instructions activate multiple lines of the decode ROM at once.
Often this is on purpose, such as one line for the addressing mode and one for the opcode part.
But many of the unofficial opcodes simultaneously trigger parts of the ROM that were intended for completely unrelated instructions.

Perhaps the pattern is easier to see by shuffling the 6502's opcode matrix.
This table lists all 6502 opcodes, 32 columns per row.
The columns are colored by bits 1 and 0:
00 red, 01 green, 10 blue, and 11 gray.

|  | +00 | +01 | +02 | +03 | +04 | +05 | +06 | +07 | +08 | +09 | +0A | +0B | +0C | +0D | +0E | +0F | +10 | +11 | +12 | +13 | +14 | +15 | +16 | +17 | +18 | +19 | +1A | +1B | +1C | +1D | +1E | +1F |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 00 | BRK | ORA<br>(d,x) | **STP** | **SLO**<br>(d,x) | **NOP**<br>d | ORA<br>d | ASL<br>d | **SLO**<br>d | PHP | ORA<br>#i | ASL | **ANC**<br>#i | **NOP**<br>a | ORA<br>a | ASL<br>a | **SLO**<br>a | BPL<br>*+d | ORA<br>(d),y | **STP** | **SLO**<br>(d),y | **NOP**<br>d,x | ORA<br>d,x | ASL<br>d,x | **SLO**<br>d,x | CLC | ORA<br>a,y | **NOP** | **SLO**<br>a,y | **NOP**<br>a,x | ORA<br>a,x | ASL<br>a,x | **SLO**<br>a,x |
| 20 | JSR<br>a | AND<br>(d,x) | **STP** | **RLA**<br>(d,x) | BIT<br>d | AND<br>d | ROL<br>d | **RLA**<br>d | PLP | AND<br>#i | ROL | **ANC**<br>#i | BIT<br>a | AND<br>a | ROL<br>a | **RLA**<br>a | BMI<br>*+d | AND<br>(d),y | **STP** | **RLA**<br>(d),y | **NOP**<br>d,x | AND<br>d,x | ROL<br>d,x | **RLA**<br>d,x | SEC | AND<br>a,y | **NOP** | **RLA**<br>a,y | **NOP**<br>a,x | AND<br>a,x | ROL<br>a,x | **RLA**<br>a,x |
| 40 | RTI | EOR<br>(d,x) | **STP** | **SRE**<br>(d,x) | **NOP**<br>d | EOR<br>d | LSR<br>d | **SRE**<br>d | PHA | EOR<br>#i | LSR | **ALR**<br>#i | JMP<br>a | EOR<br>a | LSR<br>a | **SRE**<br>a | BVC<br>*+d | EOR<br>(d),y | **STP** | **SRE**<br>(d),y | **NOP**<br>d,x | EOR<br>d,x | LSR<br>d,x | **SRE**<br>d,x | CLI | EOR<br>a,y | **NOP** | **SRE**<br>a,y | **NOP**<br>a,x | EOR<br>a,x | LSR<br>a,x | **SRE**<br>a,x |
| 60 | RTS | ADC<br>(d,x) | **STP** | **RRA**<br>(d,x) | **NOP**<br>d | ADC<br>d | ROR<br>d | **RRA**<br>d | PLA | ADC<br>#i | ROR | **ARR**<br>#i | JMP<br>(a) | ADC<br>a | ROR<br>a | **RRA**<br>a | BVS<br>*+d | ADC<br>(d),y | **STP** | **RRA**<br>(d),y | **NOP**<br>d,x | ADC<br>d,x | ROR<br>d,x | **RRA**<br>d,x | SEI | ADC<br>a,y | **NOP** | **RRA**<br>a,y | **NOP**<br>a,x | ADC<br>a,x | ROR<br>a,x | **RRA**<br>a,x |
| 80 | **NOP**<br>#i | STA<br>(d,x) | **NOP**<br>#i | **SAX**<br>(d,x) | STY<br>d | STA<br>d | STX<br>d | **SAX**<br>d | DEY | **NOP**<br>#i | TXA | **XAA**<br>#i | STY<br>a | STA<br>a | STX<br>a | **SAX**<br>a | BCC<br>*+d | STA<br>(d),y | **STP** | **AHX**<br>(d),y | STY<br>d,x | STA<br>d,x | STX<br>d,y | **SAX**<br>d,y | TYA | STA<br>a,y | TXS | **TAS**<br>a,y | **SHY**<br>a,x | STA<br>a,x | **SHX**<br>a,y | **AHX**<br>a,y |
| A0 | LDY<br>#i | LDA<br>(d,x) | LDX<br>#i | **LAX**<br>(d,x) | LDY<br>d | LDA<br>d | LDX<br>d | **LAX**<br>d | TAY | LDA<br>#i | TAX | **LAX**<br>#i | LDY<br>a | LDA<br>a | LDX<br>a | **LAX**<br>a | BCS<br>*+d | LDA<br>(d),y | **STP** | **LAX**<br>(d),y | LDY<br>d,x | LDA<br>d,x | LDX<br>d,y | **LAX**<br>d,y | CLV | LDA<br>a,y | TSX | **LAS**<br>a,y | LDY<br>a,x | LDA<br>a,x | LDX<br>a,y | **LAX**<br>a,y |
| C0 | CPY<br>#i | CMP<br>(d,x) | **NOP**<br>#i | **DCP**<br>(d,x) | CPY<br>d | CMP<br>d | DEC<br>d | **DCP**<br>d | INY | CMP<br>#i | DEX | **AXS**<br>#i | CPY<br>a | CMP<br>a | DEC<br>a | **DCP**<br>a | BNE<br>*+d | CMP<br>(d),y | **STP** | **DCP**<br>(d),y | **NOP**<br>d,x | CMP<br>d,x | DEC<br>d,x | **DCP**<br>d,x | CLD | CMP<br>a,y | **NOP** | **DCP**<br>a,y | **NOP**<br>a,x | CMP<br>a,x | DEC<br>a,x | **DCP**<br>a,x |
| E0 | CPX<br>#i | SBC<br>(d,x) | **NOP**<br>#i | **ISC**<br>(d,x) | CPX<br>d | SBC<br>d | INC<br>d | **ISC**<br>d | INX | SBC<br>#i | NOP | **SBC**<br>#i | CPX<br>a | SBC<br>a | INC<br>a | **ISC**<br>a | BEQ<br>*+d | SBC<br>(d),y | **STP** | **ISC**<br>(d),y | **NOP**<br>d,x | SBC<br>d,x | INC<br>d,x | **ISC**<br>d,x | SED | SBC<br>a,y | **NOP** | **ISC**<br>a,y | **NOP**<br>a,x | SBC<br>a,x | INC<br>a,x | **ISC**<br>a,x |

Key: a is a 16-bit absolute address, and d is an 8-bit zero page address. Entries in bold represent unofficial opcodes.

But if we rearrange it so that columns with the same bits 1-0
are close together, correlations become easier to see:

|  | +00 | +04 | +08 | +0C | +10 | +14 | +18 | +1C | +01 | +05 | +09 | +0D | +11 | +15 | +19 | +1D | +02 | +06 | +0A | +0E | +12 | +16 | +1A | +1E | +03 | +07 | +0B | +0F | +13 | +17 | +1B | +1F |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 00 | BRK | **NOP**<br>d | PHP | **NOP**<br>a | BPL<br>*+d | **NOP**<br>d,x | CLC | **NOP**<br>a,x | ORA<br>(d,x) | ORA<br>d | ORA<br>#i | ORA<br>a | ORA<br>(d),y | ORA<br>d,x | ORA<br>a,y | ORA<br>a,x | **STP** | ASL<br>d | ASL | ASL<br>a | **STP** | ASL<br>d,x | **NOP** | ASL<br>a,x | **SLO**<br>(d,x) | **SLO**<br>d | **ANC**<br>#i | **SLO**<br>a | **SLO**<br>(d),y | **SLO**<br>d,x | **SLO**<br>a,y | **SLO**<br>a,x |
| 20 | JSR<br>a | BIT<br>d | PLP | BIT<br>a | BMI<br>*+d | **NOP**<br>d,x | SEC | **NOP**<br>a,x | AND<br>(d,x) | AND<br>d | AND<br>#i | AND<br>a | AND<br>(d),y | AND<br>d,x | AND<br>a,y | AND<br>a,x | **STP** | ROL<br>d | ROL | ROL<br>a | **STP** | ROL<br>d,x | **NOP** | ROL<br>a,x | **RLA**<br>(d,x) | **RLA**<br>d | **ANC**<br>#i | **RLA**<br>a | **RLA**<br>(d),y | **RLA**<br>d,x | **RLA**<br>a,y | **RLA**<br>a,x |
| 40 | RTI | **NOP**<br>d | PHA | JMP<br>a | BVC<br>*+d | **NOP**<br>d,x | CLI | **NOP**<br>a,x | EOR<br>(d,x) | EOR<br>d | EOR<br>#i | EOR<br>a | EOR<br>(d),y | EOR<br>d,x | EOR<br>a,y | EOR<br>a,x | **STP** | LSR<br>d | LSR | LSR<br>a | **STP** | LSR<br>d,x | **NOP** | LSR<br>a,x | **SRE**<br>(d,x) | **SRE**<br>d | **ALR**<br>#i | **SRE**<br>a | **SRE**<br>(d),y | **SRE**<br>d,x | **SRE**<br>a,y | **SRE**<br>a,x |
| 60 | RTS | **NOP**<br>d | PLA | JMP<br>(a) | BVS<br>*+d | **NOP**<br>d,x | SEI | **NOP**<br>a,x | ADC<br>(d,x) | ADC<br>d | ADC<br>#i | ADC<br>a | ADC<br>(d),y | ADC<br>d,x | ADC<br>a,y | ADC<br>a,x | **STP** | ROR<br>d | ROR | ROR<br>a | **STP** | ROR<br>d,x | **NOP** | ROR<br>a,x | **RRA**<br>(d,x) | **RRA**<br>d | **ARR**<br>#i | **RRA**<br>a | **RRA**<br>(d),y | **RRA**<br>d,x | **RRA**<br>a,y | **RRA**<br>a,x |
| 80 | **NOP**<br>#i | STY<br>d | DEY | STY<br>a | BCC<br>*+d | STY<br>d,x | TYA | **SHY**<br>a,x | STA<br>(d,x) | STA<br>d | **NOP**<br>#i | STA<br>a | STA<br>(d),y | STA<br>d,x | STA<br>a,y | STA<br>a,x | **NOP**<br>#i | STX<br>d | TXA | STX<br>a | **STP** | STX<br>d,y | TXS | **SHX**<br>a,y | **SAX**<br>(d,x) | **SAX**<br>d | **XAA**<br>#i | **SAX**<br>a | **AHX**<br>(d),y | **SAX**<br>d,y | **TAS**<br>a,y | **AHX**<br>a,y |
| A0 | LDY<br>#i | LDY<br>d | TAY | LDY<br>a | BCS<br>*+d | LDY<br>d,x | CLV | LDY<br>a,x | LDA<br>(d,x) | LDA<br>d | LDA<br>#i | LDA<br>a | LDA<br>(d),y | LDA<br>d,x | LDA<br>a,y | LDA<br>a,x | LDX<br>#i | LDX<br>d | TAX | LDX<br>a | **STP** | LDX<br>d,y | TSX | LDX<br>a,y | **LAX**<br>(d,x) | **LAX**<br>d | **LAX**<br>#i | **LAX**<br>a | **LAX**<br>(d),y | **LAX**<br>d,y | **LAS**<br>a,y | **LAX**<br>a,y |
| C0 | CPY<br>#i | CPY<br>d | INY | CPY<br>a | BNE<br>*+d | **NOP**<br>d,x | CLD | **NOP**<br>a,x | CMP<br>(d,x) | CMP<br>d | CMP<br>#i | CMP<br>a | CMP<br>(d),y | CMP<br>d,x | CMP<br>a,y | CMP<br>a,x | **NOP**<br>#i | DEC<br>d | DEX | DEC<br>a | **STP** | DEC<br>d,x | **NOP** | DEC<br>a,x | **DCP**<br>(d,x) | **DCP**<br>d | **AXS**<br>#i | **DCP**<br>a | **DCP**<br>(d),y | **DCP**<br>d,x | **DCP**<br>a,y | **DCP**<br>a,x |
| E0 | CPX<br>#i | CPX<br>d | INX | CPX<br>a | BEQ<br>*+d | **NOP**<br>d,x | SED | **NOP**<br>a,x | SBC<br>(d,x) | SBC<br>d | SBC<br>#i | SBC<br>a | SBC<br>(d),y | SBC<br>d,x | SBC<br>a,y | SBC<br>a,x | **NOP**<br>#i | INC<br>d | NOP | INC<br>a | **STP** | INC<br>d,x | **NOP** | INC<br>a,x | **ISC**<br>(d,x) | **ISC**<br>d | **SBC**<br>#i | **ISC**<br>a | **ISC**<br>(d),y | **ISC**<br>d,x | **ISC**<br>a,y | **ISC**<br>a,x |

Thus the 00 (red) block is mostly control instructions,
01 (green) is ALU operations,
and 10 (blue) is read-modify-write (RMW) operations and data movement instructions involving X.
The RMW instructions (all but row 80 and A0) in columns +06, +0E, +16, and +1E
have the same addressing modes as the corresponding ALU operations.

The 11 (gray) block is unofficial opcodes combining the operations of instructions from the ALU and RMW blocks.
all of them having the same addressing mode as the corresponding ALU opcode.
The RMW+ALU instructions that affect memory are easiest to understand
because their RMW part completes during the opcode and the ALU part completes during the next opcode's fetch.
Column +0B, on the other hand, has no extra cycles; everything completes during the next opcode's fetch.
This causes instructions to have strange mixing properties.
Some even [differ based on analog effects](https://www.nesdev.org/wiki/Visual6502wiki/6502_Opcode_8B_(XAA,_ANE)).

## Games using unofficial opcodes

The use of unofficial opcodes is rare in NES games. It appears to occur mostly in late or unlicensed titles:

* *Beauty and the Beast* (E) (1994) uses $80 (a 2-byte NOP).[[2]](#cite_note-2)
* *Disney's Aladdin* (E) (December 1994) uses $07 (SLO). This is Virgin's port of the Game Boy game, itself a port of the Genesis game, not any of the [pirate originals](https://bootleggames.fandom.com/wiki/Special:PrefixIndex/Aladdin).
* *Dynowarz: Destruction of Spondylus* (April 1990) uses 1-byte NOPs $DA and $FA on the first level when your dino throws his fist.
* *F-117A Stealth Fighter* uses $89 (a 2-byte NOP).
* *文字广场+排雷* (romanized in GoodNES as Cantonese "Gaau Hok Gwong Cheung (Ch)"): After selecting the left game (排雷) from this 2-in-1 multicart, a glitchy [32 KiB bankswitch](https://www.nesdev.org/wiki/INES_Mapper_242) causes the CPU to get lost in non-code ROM space that only with correct emulation of unofficial opcodes, including $8B (XAA immediate), will have it eventually reach a BRK instruction that properly branches to that game's reset handler.
* *Infiltrator* uses $89 (a 2-byte NOP).
* *Ninja Jajamaru-kun* uses $04 (a 2-byte NOP) when your ninja collides with an enemy, as a consequence of branching into the middle of an instruction.
* *Puzznic* (all regions) (US release November 1990) uses $89 (a 2-byte NOP).
* *Super Cars* (U) (February 1991) uses $B3 (LAX).

### Homebrew games

* The MUSE music engine, used in *Driar* and *STREEMERZ: Super Strength Emergency Squad Zeta*, uses $8F (SAX), $B3 (LAX), and $CB (AXS).[[3]](#cite_note-3)
* *[Attribute Zone](https://www.nesdev.org/wiki/User:Zzo38/Attribute_Zone)* uses $0B (ANC), $2F (RLA), $4B (ALR), $A7 (LAX), $B3 (LAX), $CB (AXS), $D3 (DCP) and $DB (DCP).
* The port of *Zork* to the Famicom uses a few unofficial opcodes.
* *Eyra, the Crow Maiden* uses several unofficial opcodes.

## See also

* [Programming with unofficial opcodes](https://www.nesdev.org/wiki/Programming_with_unofficial_opcodes)

## External links

* [6502 opcode matrix including unofficial opcodes](http://www.oxyron.de/html/opcodes02.html)
* [65C02](http://www.oxyron.de/html/opcodesc02.html) and [65816](http://www.oxyron.de/html/opcodes816.html)
* [Illegal opcodes](https://en.wikipedia.org/wiki/Illegal_opcode) at Wikipedia.
* [65xx Processor Data](http://www.romhacking.net/documents/318/)
* [6502_cpu.txt](http://nesdev.org/6502_cpu.txt)
* [No More Secrets v1.00](https://csdb.dk/release/?id=258111)

## References

1. [↑](#cite_ref-1) Michael Steil. "[How MOS 6502 Illegal Opcodes really work](https://www.pagetable.com/?p=39)". *Pagetable*, 2008-07-29. Accessed 2019-11-10.
2. [↑](#cite_ref-2) [puNES 0.20 changelog](http://forums.nesdev.org/viewtopic.php?f=3&t=6928) indicating $80 opcode in *Beauty and the Beast*.
3. [↑](#cite_ref-3) <http://forums.nesdev.org/viewtopic.php?p=100957#p100957>
