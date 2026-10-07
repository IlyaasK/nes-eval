# 6502 cycle times

> Source: <https://www.nesdev.org/wiki/6502_cycle_times> — NESdev Wiki, revision [23578](https://www.nesdev.org/w/index.php?oldid=23578) (last edited 2026-03-06T11:03:39Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=6502_cycle_times&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

| Mnemonic | Description | Imp | Imm | ZP | ZP,X | ZP,Y | Abs | Abs,X | Abs,Y | Ind | Ind,X | Ind,Y | Acc | Rel |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ADC | Add with carry |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| AND | Bitwise AND with A |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| BIT | Bit test |  |  | 3 |  |  | 4 |  |  |  |  |  |  |  |
| CMP | Compare A |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| CPX | Compare X |  | 2 | 3 |  |  | 4 |  |  |  |  |  |  |  |
| CPY | Compare Y |  | 2 | 3 |  |  | 4 |  |  |  |  |  |  |  |
| EOR | Bitwise XOR with A |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| LDA | Load A |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| LDX | Load X |  | 2 | 3 |  | 4 | 4 |  | 4+ |  |  |  |  |  |
| LDY | Load Y |  | 2 | 3 | 4 |  | 4 | 4+ |  |  |  |  |  |  |
| ORA | Bitwise OR with A |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| SBC | Subtract with carry |  | 2 | 3 | 4 |  | 4 | 4+ | 4+ |  | 6 | 5+ |  |  |
| STA | Store A |  |  | 3 | 4 |  | 4 | 5 | 5 |  | 6 | 6 |  |  |
| STX | Store X |  |  | 3 |  | 4 | 4 |  |  |  |  |  |  |  |
| STY | Store Y |  |  | 3 | 4 |  | 4 |  |  |  |  |  |  |  |
| ASL | Arithmetic shift left |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  | 2 |  |
| DEC | Decrement memory |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  |  |  |
| INC | Increment memory |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  |  |  |
| LSR | Logical shift right |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  | 2 |  |
| ROL | Rotate left |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  | 2 |  |
| ROR | Rotate right |  |  | 5 | 6 |  | 6 | 7 |  |  |  |  | 2 |  |
| PHA | Push A | 3 |  |  |  |  |  |  |  |  |  |  |  |  |
| PHP | Push processor status | 3 |  |  |  |  |  |  |  |  |  |  |  |  |
| PLA | Pull A | 4 |  |  |  |  |  |  |  |  |  |  |  |  |
| PLP | Pull processor status | 4 |  |  |  |  |  |  |  |  |  |  |  |  |
| BRK | Break | 7 |  |  |  |  |  |  |  |  |  |  |  |  |
| JMP | Jump |  |  |  |  |  | 3 |  |  | 5 |  |  |  |  |
| JSR | Jump to subroutine |  |  |  |  |  | 6 |  |  |  |  |  |  |  |
| RTI | Return from interrupt | 6 |  |  |  |  |  |  |  |  |  |  |  |  |
| RTS | Return from subroutine | 6 |  |  |  |  |  |  |  |  |  |  |  |  |
| BCC | Branch if carry clear |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BCS | Branch if carry set |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BEQ | Branch if equal |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BMI | Branch if minus |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BNE | Branch if not equal |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BPL | Branch if plus |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BVC | Branch if overflow clear |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| BVS | Branch if overflow set |  |  |  |  |  |  |  |  |  |  |  |  | 2++ |
| CLC | Clear carry | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| CLD | Clear decimal | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| CLI | Clear interrupt disable | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| CLV | Clear overflow | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| DEX | Decrement X | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| DEY | Decrement Y | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| INX | Increment X | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| INY | Increment Y | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| NOP | No operation | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| SEC | Set carry | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| SED | Set decimal | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| SEI | Set interrupt disable | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TAX | Transfer A to X | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TAY | Transfer A to Y | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TSX | Transfer S to X | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TXA | Transfer X to A | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TXS | Transfer X to S | 2 |  |  |  |  |  |  |  |  |  |  |  |  |
| TYA | Transfer Y to A | 2 |  |  |  |  |  |  |  |  |  |  |  |  |

+: Plus 1 cycle if page crossed
++: Plus 1 cycle if branch taken, and 1 more cycle if page crossed
