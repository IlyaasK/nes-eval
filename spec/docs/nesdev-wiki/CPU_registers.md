# CPU registers

> Source: <https://www.nesdev.org/wiki/CPU_registers> — NESdev Wiki, revision [21223](https://www.nesdev.org/w/index.php?oldid=21223) (last edited 2023-08-09T21:52:40Z), retrieved 2026-10-06. Authors: NESdev Wiki contributors ([history](https://www.nesdev.org/w/index.php?title=CPU_registers&action=history)). Images omitted. See [SOURCES.md](../SOURCES.md) for license.

The registers on the NES CPU are just like on the 6502. There is the accumulator, 2 indexes, a program counter, the stack pointer, and the status register. Unlike many CPU families, members do not have generic groups of registers like say, R0 through R7.

### Accumulator

**A** is byte-wide and along with the [arithmetic logic unit](https://en.wikipedia.org/wiki/arithmetic_logic_unit) (ALU), supports using the status register for carrying, overflow detection, and so on.

### Indexes

**X** and **Y** are byte-wide and used for several addressing modes. They can be used as loop counters easily, using INC/DEC and branch instructions. Not being the accumulator, they have limited addressing modes themselves when loading and saving.

### Program Counter

The 2-byte program counter **PC** supports 65536 direct (unbanked) memory locations, however not all values are sent to the cartridge. It can be accessed either by allowing CPU's internal fetch logic increment the address bus, an interrupt (NMI, Reset, IRQ/BRQ), and using the RTS/JMP/JSR/Branch instructions.

### Stack Pointer

**S** is byte-wide and can be accessed using interrupts, pulls, pushes, and transfers.
It indexes into a 256-byte [stack](https://www.nesdev.org/wiki/Stack) at $0100-$01FF.

### Status Register

**P** has 6 bits used by the ALU but is byte-wide. PHP, PLP, arithmetic, testing, and branch instructions can access this register. See [status flags](Status_flags.md).
