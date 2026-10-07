//! Ricoh 2A03 CPU core (6502 without decimal mode).
//!
//! Instructions are executed one at a time, but every bus access (including
//! the dummy reads and writes real hardware performs) costs one cycle, so
//! cycle counts and page-crossing penalties match the hardware.

use crate::bus::Bus;

const C: u8 = 0x01;
const Z: u8 = 0x02;
const I: u8 = 0x04;
const D: u8 = 0x08;
const B: u8 = 0x10;
const U: u8 = 0x20;
const V: u8 = 0x40;
const N: u8 = 0x80;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Imm,
    Zp,
    Zpx,
    Zpy,
    Abs,
    Absx,
    Absy,
    Indx,
    Indy,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Access {
    Read,
    Write,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Interrupt {
    None,
    Nmi,
    Irq,
}

pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
    pub pc: u16,
    pending: Interrupt,
    jammed: bool,
}

impl Cpu {
    /// Power-on state at the first instruction fetch (spec/ABI.md).
    pub fn new(bus: &Bus) -> Cpu {
        let pc = bus.peek(0xFFFC) as u16 | (bus.peek(0xFFFD) as u16) << 8;
        Cpu { a: 0, x: 0, y: 0, s: 0xFD, p: 0x24, pc, pending: Interrupt::None, jammed: false }
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn set_zn(&mut self, v: u8) {
        self.p &= !(Z | N);
        if v == 0 {
            self.p |= Z;
        }
        self.p |= v & N;
    }

    fn set_flag(&mut self, flag: u8, on: bool) {
        if on {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let v = bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn fetch16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        hi << 8 | lo
    }

    fn push(&mut self, bus: &mut Bus, v: u8) {
        bus.write(0x100 | self.s as u16, v);
        self.s = self.s.wrapping_sub(1);
    }

    fn pull(&mut self, bus: &mut Bus) -> u8 {
        self.s = self.s.wrapping_add(1);
        bus.read(0x100 | self.s as u16)
    }

    /// Dummy read of the next opcode byte (implied / accumulator ops).
    fn idle(&mut self, bus: &mut Bus) {
        bus.read(self.pc);
    }

    /// Computes an effective address. Indexed modes perform the dummy read
    /// at the un-carried address when the index crosses a page, and always
    /// for writes and read-modify-writes.
    fn addr(&mut self, bus: &mut Bus, mode: Mode, access: Access) -> u16 {
        match mode {
            Mode::Imm => {
                let a = self.pc;
                self.pc = self.pc.wrapping_add(1);
                a
            }
            Mode::Zp => self.fetch(bus) as u16,
            Mode::Zpx | Mode::Zpy => {
                let base = self.fetch(bus);
                bus.read(base as u16);
                let idx = if mode == Mode::Zpx { self.x } else { self.y };
                base.wrapping_add(idx) as u16
            }
            Mode::Abs => self.fetch16(bus),
            Mode::Absx | Mode::Absy => {
                let base = self.fetch16(bus);
                let idx = if mode == Mode::Absx { self.x } else { self.y };
                self.indexed(bus, base, idx, access)
            }
            Mode::Indx => {
                let ptr = self.fetch(bus);
                bus.read(ptr as u16);
                let p = ptr.wrapping_add(self.x);
                let lo = bus.read(p as u16) as u16;
                let hi = bus.read(p.wrapping_add(1) as u16) as u16;
                hi << 8 | lo
            }
            Mode::Indy => {
                let ptr = self.fetch(bus);
                let lo = bus.read(ptr as u16) as u16;
                let hi = bus.read(ptr.wrapping_add(1) as u16) as u16;
                self.indexed(bus, hi << 8 | lo, self.y, access)
            }
        }
    }

    fn indexed(&mut self, bus: &mut Bus, base: u16, idx: u8, access: Access) -> u16 {
        let addr = base.wrapping_add(idx as u16);
        if addr & 0xFF00 != base & 0xFF00 || access == Access::Write {
            bus.read((base & 0xFF00) | (addr & 0x00FF));
        }
        addr
    }

    fn load(&mut self, bus: &mut Bus, mode: Mode) -> u8 {
        let a = self.addr(bus, mode, Access::Read);
        bus.read(a)
    }

    fn store(&mut self, bus: &mut Bus, mode: Mode, v: u8) {
        let a = self.addr(bus, mode, Access::Write);
        bus.write(a, v);
    }

    /// Read-modify-write: read, write the old value back, write the result.
    fn rmw(&mut self, bus: &mut Bus, mode: Mode, f: fn(&mut Cpu, u8) -> u8) -> u8 {
        let a = self.addr(bus, mode, Access::Write);
        let v = bus.read(a);
        bus.write(a, v);
        let r = f(self, v);
        bus.write(a, r);
        r
    }

    // ── ALU ─────────────────────────────────────────────────────────────

    fn adc(&mut self, v: u8) {
        let sum = self.a as u16 + v as u16 + (self.p & C) as u16;
        let r = sum as u8;
        self.set_flag(C, sum > 0xFF);
        self.set_flag(V, (self.a ^ r) & (v ^ r) & 0x80 != 0);
        self.a = r;
        self.set_zn(r);
    }

    fn sbc(&mut self, v: u8) {
        self.adc(!v);
    }

    fn compare(&mut self, reg: u8, v: u8) {
        self.set_flag(C, reg >= v);
        self.set_zn(reg.wrapping_sub(v));
    }

    fn asl(&mut self, v: u8) -> u8 {
        self.set_flag(C, v & 0x80 != 0);
        let r = v << 1;
        self.set_zn(r);
        r
    }

    fn lsr(&mut self, v: u8) -> u8 {
        self.set_flag(C, v & 1 != 0);
        let r = v >> 1;
        self.set_zn(r);
        r
    }

    fn rol(&mut self, v: u8) -> u8 {
        let r = (v << 1) | (self.p & C);
        self.set_flag(C, v & 0x80 != 0);
        self.set_zn(r);
        r
    }

    fn ror(&mut self, v: u8) -> u8 {
        let r = (v >> 1) | ((self.p & C) << 7);
        self.set_flag(C, v & 1 != 0);
        self.set_zn(r);
        r
    }

    fn inc(&mut self, v: u8) -> u8 {
        let r = v.wrapping_add(1);
        self.set_zn(r);
        r
    }

    fn dec(&mut self, v: u8) -> u8 {
        let r = v.wrapping_sub(1);
        self.set_zn(r);
        r
    }

    fn branch(&mut self, bus: &mut Bus, cond: bool) {
        let off = self.fetch(bus) as i8;
        if !cond {
            return;
        }
        bus.read(self.pc);
        let target = self.pc.wrapping_add(off as u16);
        if target & 0xFF00 != self.pc & 0xFF00 {
            bus.read((self.pc & 0xFF00) | (target & 0x00FF));
        }
        self.pc = target;
    }

    // ── Interrupts ──────────────────────────────────────────────────────

    /// Pushes PC and P and jumps through a vector. An NMI that is pending
    /// by the end of the fourth cycle hijacks IRQ and BRK sequences.
    fn interrupt(&mut self, bus: &mut Bus, brk: bool, nmi: bool) {
        self.push(bus, (self.pc >> 8) as u8);
        self.push(bus, self.pc as u8);
        let use_nmi = nmi || bus.nmi_pending;
        if use_nmi {
            bus.nmi_pending = false;
        }
        let flags = if brk { self.p | B | U } else { (self.p & !B) | U };
        self.push(bus, flags);
        let vector = if use_nmi { 0xFFFA } else { 0xFFFE };
        self.p |= I;
        let lo = bus.read(vector) as u16;
        let hi = bus.read(vector + 1) as u16;
        self.pc = hi << 8 | lo;
    }

    /// Executes one instruction (or one interrupt entry sequence).
    pub fn step(&mut self, bus: &mut Bus) {
        match self.pending {
            Interrupt::Nmi | Interrupt::Irq => {
                let nmi = self.pending == Interrupt::Nmi;
                if nmi {
                    bus.nmi_pending = false;
                }
                self.pending = Interrupt::None;
                bus.read(self.pc);
                bus.read(self.pc);
                self.interrupt(bus, false, nmi);
                return;
            }
            Interrupt::None => {}
        }

        if self.jammed {
            bus.read(0xFFFF);
            return;
        }

        let i_before = self.p & I;
        let op = self.fetch(bus);
        self.execute(bus, op);
        if op == 0x00 {
            // BRK is an interrupt sequence: no poll at its end.
            return;
        }

        // CLI, SEI and PLP change I after the interrupt poll, so the poll
        // sees the old value.
        let i_flag = match op {
            0x58 | 0x78 | 0x28 => i_before,
            _ => self.p & I,
        };
        if bus.nmi_poll && bus.nmi_pending {
            self.pending = Interrupt::Nmi;
        } else if bus.irq_poll && i_flag == 0 {
            self.pending = Interrupt::Irq;
        }
    }

    fn execute(&mut self, bus: &mut Bus, op: u8) {
        match op {
            // ── Control flow and implied ops ────────────────────────────
            0x00 => {
                self.fetch(bus);
                self.interrupt(bus, true, false);
            }
            0x20 => {
                let lo = self.fetch(bus) as u16;
                bus.read(0x100 | self.s as u16);
                self.push(bus, (self.pc >> 8) as u8);
                self.push(bus, self.pc as u8);
                let hi = bus.read(self.pc) as u16;
                self.pc = hi << 8 | lo;
            }
            0x40 => {
                self.idle(bus);
                bus.read(0x100 | self.s as u16);
                let p = self.pull(bus);
                self.p = (p & !B) | U;
                let lo = self.pull(bus) as u16;
                let hi = self.pull(bus) as u16;
                self.pc = hi << 8 | lo;
            }
            0x60 => {
                self.idle(bus);
                bus.read(0x100 | self.s as u16);
                let lo = self.pull(bus) as u16;
                let hi = self.pull(bus) as u16;
                self.pc = hi << 8 | lo;
                self.fetch(bus);
            }
            0x4C => self.pc = self.fetch16(bus),
            0x6C => {
                let ptr = self.fetch16(bus);
                let lo = bus.read(ptr) as u16;
                // The pointer's high byte doesn't carry across a page.
                let hi = bus.read((ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF)) as u16;
                self.pc = hi << 8 | lo;
            }
            0x08 => {
                self.idle(bus);
                self.push(bus, self.p | B | U);
            }
            0x28 => {
                self.idle(bus);
                bus.read(0x100 | self.s as u16);
                let p = self.pull(bus);
                self.p = (p & !B) | U;
            }
            0x48 => {
                self.idle(bus);
                self.push(bus, self.a);
            }
            0x68 => {
                self.idle(bus);
                bus.read(0x100 | self.s as u16);
                self.a = self.pull(bus);
                self.set_zn(self.a);
            }

            0x10 => self.branch(bus, self.p & N == 0),
            0x30 => self.branch(bus, self.p & N != 0),
            0x50 => self.branch(bus, self.p & V == 0),
            0x70 => self.branch(bus, self.p & V != 0),
            0x90 => self.branch(bus, self.p & C == 0),
            0xB0 => self.branch(bus, self.p & C != 0),
            0xD0 => self.branch(bus, self.p & Z == 0),
            0xF0 => self.branch(bus, self.p & Z != 0),

            0x18 | 0x38 | 0x58 | 0x78 | 0xB8 | 0xD8 | 0xF8 => {
                self.idle(bus);
                match op {
                    0x18 => self.p &= !C,
                    0x38 => self.p |= C,
                    0x58 => self.p &= !I,
                    0x78 => self.p |= I,
                    0xB8 => self.p &= !V,
                    0xD8 => self.p &= !D,
                    _ => self.p |= D,
                }
            }
            0x88 | 0xA8 | 0xC8 | 0xE8 | 0x98 | 0x8A | 0x9A | 0xAA | 0xBA | 0xCA => {
                self.idle(bus);
                match op {
                    0x88 => self.y = self.dec(self.y),
                    0xC8 => self.y = self.inc(self.y),
                    0xCA => self.x = self.dec(self.x),
                    0xE8 => self.x = self.inc(self.x),
                    0xA8 => {
                        self.y = self.a;
                        self.set_zn(self.y);
                    }
                    0x98 => {
                        self.a = self.y;
                        self.set_zn(self.a);
                    }
                    0x8A => {
                        self.a = self.x;
                        self.set_zn(self.a);
                    }
                    0xAA => {
                        self.x = self.a;
                        self.set_zn(self.x);
                    }
                    0xBA => {
                        self.x = self.s;
                        self.set_zn(self.x);
                    }
                    _ => self.s = self.x, // TXS
                }
            }
            // NOP, including the unofficial single-byte ones.
            0xEA | 0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => self.idle(bus),

            // Accumulator shifts.
            0x0A => {
                self.idle(bus);
                self.a = self.asl(self.a);
            }
            0x2A => {
                self.idle(bus);
                self.a = self.rol(self.a);
            }
            0x4A => {
                self.idle(bus);
                self.a = self.lsr(self.a);
            }
            0x6A => {
                self.idle(bus);
                self.a = self.ror(self.a);
            }

            // ── Group-encoded instructions ──────────────────────────────
            _ => self.execute_grouped(bus, op),
        }
    }

    /// Instructions following the regular aaabbbcc encoding.
    fn execute_grouped(&mut self, bus: &mut Bus, op: u8) {
        use Mode::*;
        let aaa = op >> 5;
        let bbb = (op >> 2) & 7;
        match op & 3 {
            // ORA AND EOR ADC STA LDA CMP SBC
            1 => {
                let mode = [Indx, Zp, Imm, Abs, Indy, Zpx, Absy, Absx][bbb as usize];
                match aaa {
                    0 => {
                        let v = self.load(bus, mode);
                        self.a |= v;
                        self.set_zn(self.a);
                    }
                    1 => {
                        let v = self.load(bus, mode);
                        self.a &= v;
                        self.set_zn(self.a);
                    }
                    2 => {
                        let v = self.load(bus, mode);
                        self.a ^= v;
                        self.set_zn(self.a);
                    }
                    3 => {
                        let v = self.load(bus, mode);
                        self.adc(v);
                    }
                    4 => {
                        if mode == Imm {
                            self.fetch(bus); // $89: unofficial NOP #imm
                        } else {
                            self.store(bus, mode, self.a);
                        }
                    }
                    5 => {
                        self.a = self.load(bus, mode);
                        self.set_zn(self.a);
                    }
                    6 => {
                        let v = self.load(bus, mode);
                        self.compare(self.a, v);
                    }
                    _ => {
                        let v = self.load(bus, mode);
                        self.sbc(v);
                    }
                }
            }
            // ASL ROL LSR ROR STX LDX DEC INC
            2 => {
                let xy_swap = aaa == 4 || aaa == 5;
                let mode = match bbb {
                    0 => Imm,
                    1 => Zp,
                    3 => Abs,
                    5 if xy_swap => Zpy,
                    5 => Zpx,
                    7 if xy_swap => Absy,
                    7 => Absx,
                    _ => {
                        self.jam_or_nop(bus, op);
                        return;
                    }
                };
                if mode == Imm && aaa != 5 {
                    // $82/$C2/$E2: NOP #imm; $02/$22/$42/$62: jam.
                    if aaa >= 4 {
                        self.fetch(bus);
                    } else {
                        self.jammed = true;
                    }
                    return;
                }
                match aaa {
                    0 => {
                        self.rmw(bus, mode, Cpu::asl);
                    }
                    1 => {
                        self.rmw(bus, mode, Cpu::rol);
                    }
                    2 => {
                        self.rmw(bus, mode, Cpu::lsr);
                    }
                    3 => {
                        self.rmw(bus, mode, Cpu::ror);
                    }
                    4 => {
                        if mode == Absy {
                            // $9E SHX
                            let a = self.addr(bus, mode, Access::Write);
                            self.store_unstable(bus, a, self.x);
                        } else {
                            self.store(bus, mode, self.x);
                        }
                    }
                    5 => {
                        self.x = self.load(bus, mode);
                        self.set_zn(self.x);
                    }
                    6 => {
                        self.rmw(bus, mode, Cpu::dec);
                    }
                    _ => {
                        self.rmw(bus, mode, Cpu::inc);
                    }
                }
            }
            // BIT STY LDY CPY CPX, plus unofficial NOPs
            0 => {
                let mode = match bbb {
                    0 => Imm,
                    1 => Zp,
                    3 => Abs,
                    5 => Zpx,
                    7 => Absx,
                    _ => {
                        self.jam_or_nop(bus, op);
                        return;
                    }
                };
                match (aaa, mode) {
                    (1, Zp) | (1, Abs) => {
                        let v = self.load(bus, mode);
                        self.set_flag(Z, self.a & v == 0);
                        self.set_flag(V, v & 0x40 != 0);
                        self.set_flag(N, v & 0x80 != 0);
                    }
                    (4, Absx) => {
                        // $9C SHY
                        let a = self.addr(bus, mode, Access::Write);
                        self.store_unstable(bus, a, self.y);
                    }
                    (4, Imm) => {
                        self.fetch(bus);
                    }
                    (4, _) => self.store(bus, mode, self.y),
                    (5, _) => {
                        self.y = self.load(bus, mode);
                        self.set_zn(self.y);
                    }
                    (6, Imm) | (6, Zp) | (6, Abs) => {
                        let v = self.load(bus, mode);
                        self.compare(self.y, v);
                    }
                    (7, Imm) | (7, Zp) | (7, Abs) => {
                        let v = self.load(bus, mode);
                        self.compare(self.x, v);
                    }
                    _ => {
                        // Unofficial NOPs with an operand: they still read it.
                        self.load(bus, mode);
                    }
                }
            }
            // Unofficial combined ops.
            _ => self.execute_unofficial(bus, op, aaa, bbb),
        }
    }

    fn jam_or_nop(&mut self, bus: &mut Bus, op: u8) {
        if op & 0x1F == 0x12 || op & 0x1F == 0x02 {
            self.jammed = true;
        } else {
            self.idle(bus);
        }
    }

    /// Value written by SHX/SHY/SHA/TAS: reg & (high byte of address + 1).
    fn store_unstable(&mut self, bus: &mut Bus, addr: u16, reg: u8) {
        let v = reg & ((addr >> 8) as u8).wrapping_add(1);
        bus.write(addr, v);
    }

    fn execute_unofficial(&mut self, bus: &mut Bus, op: u8, aaa: u8, bbb: u8) {
        use Mode::*;
        let xy_swap = aaa == 4 || aaa == 5;
        if bbb == 2 {
            // Immediate-mode oddities.
            let v = self.fetch(bus);
            match op {
                0x0B | 0x2B => {
                    self.a &= v;
                    self.set_zn(self.a);
                    self.set_flag(C, self.a & 0x80 != 0);
                }
                0x4B => {
                    self.a &= v;
                    self.a = self.lsr(self.a);
                }
                0x6B => {
                    self.a &= v;
                    self.a = (self.a >> 1) | ((self.p & C) << 7);
                    self.set_zn(self.a);
                    self.set_flag(C, self.a & 0x40 != 0);
                    self.set_flag(V, ((self.a >> 6) ^ (self.a >> 5)) & 1 != 0);
                }
                0x8B => {
                    self.a = (self.a | 0xEE) & self.x & v;
                    self.set_zn(self.a);
                }
                0xAB => {
                    self.a = (self.a | 0xEE) & v;
                    self.x = self.a;
                    self.set_zn(self.a);
                }
                0xCB => {
                    let ax = self.a & self.x;
                    self.set_flag(C, ax >= v);
                    self.x = ax.wrapping_sub(v);
                    self.set_zn(self.x);
                }
                _ => self.sbc(v), // $EB
            }
            return;
        }
        let mode = match bbb {
            0 => Indx,
            1 => Zp,
            3 => Abs,
            4 => Indy,
            5 if xy_swap => Zpy,
            5 => Zpx,
            6 => Absy,
            7 if xy_swap => Absy,
            _ => Absx,
        };
        match aaa {
            0 => {
                let r = self.rmw(bus, mode, Cpu::asl);
                self.a |= r;
                self.set_zn(self.a);
            }
            1 => {
                let r = self.rmw(bus, mode, Cpu::rol);
                self.a &= r;
                self.set_zn(self.a);
            }
            2 => {
                let r = self.rmw(bus, mode, Cpu::lsr);
                self.a ^= r;
                self.set_zn(self.a);
            }
            3 => {
                let r = self.rmw(bus, mode, Cpu::ror);
                self.adc(r);
            }
            4 => match op {
                0x93 | 0x9F => {
                    let a = self.addr(bus, mode, Access::Write);
                    self.store_unstable(bus, a, self.a & self.x);
                }
                0x9B => {
                    self.s = self.a & self.x;
                    let a = self.addr(bus, mode, Access::Write);
                    self.store_unstable(bus, a, self.s);
                }
                _ => self.store(bus, mode, self.a & self.x),
            },
            5 => {
                let v = self.load(bus, mode);
                if op == 0xBB {
                    let r = v & self.s;
                    self.a = r;
                    self.x = r;
                    self.s = r;
                } else {
                    self.a = v;
                    self.x = v;
                }
                self.set_zn(self.a);
            }
            6 => {
                let r = self.rmw(bus, mode, |_, v| v.wrapping_sub(1));
                self.compare(self.a, r);
            }
            _ => {
                let r = self.rmw(bus, mode, |_, v| v.wrapping_add(1));
                self.sbc(r);
            }
        }
    }
}
