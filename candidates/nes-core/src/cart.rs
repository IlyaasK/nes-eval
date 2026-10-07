//! Cartridge: iNES / NES 2.0 parsing and mappers 0-4.
//!
//! Every mapper is expressed as a set of bank offsets: four 8 KiB PRG
//! windows at $8000/$A000/$C000/$E000 and eight 1 KiB CHR windows across
//! PPU $0000-$1FFF. Register writes recompute the offsets, so reads are a
//! table lookup no matter the mapper.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mirroring {
    Horizontal,
    Vertical,
    SingleLow,
    SingleHigh,
    FourScreen,
}

#[derive(Default)]
struct Mmc1 {
    shift: u8,
    count: u8,
    control: u8,
    chr0: u8,
    chr1: u8,
    prg: u8,
    last_write_cycle: u64,
}

#[derive(Default)]
struct Mmc3 {
    select: u8,
    regs: [u8; 8],
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
}

enum Mapper {
    Nrom,
    Mmc1(Mmc1),
    Uxrom,
    Cnrom,
    Mmc3(Mmc3),
}

pub struct Cart {
    prg: Vec<u8>,
    chr: Vec<u8>,
    chr_is_ram: bool,
    prg_ram: Vec<u8>,
    prg_map: [usize; 4],
    chr_map: [usize; 8],
    mirroring: Mirroring,
    four_screen: bool,
    mapper: Mapper,
}

impl Cart {
    /// Parses an iNES or NES 2.0 image. Returns `None` for malformed images
    /// and unsupported mappers.
    pub fn new(rom: &[u8]) -> Option<Cart> {
        if rom.len() < 16 || &rom[0..4] != b"NES\x1A" {
            return None;
        }
        let f6 = rom[6];
        let f7 = rom[7];
        let nes2 = f7 & 0x0C == 0x08;

        let (prg_units, chr_units, mapper_num) = if nes2 {
            let prg = rom[4] as usize | ((rom[9] as usize & 0x0F) << 8);
            let chr = rom[5] as usize | ((rom[9] as usize & 0xF0) << 4);
            let m = (f6 >> 4) as u16 | (f7 & 0xF0) as u16 | ((rom[8] as u16 & 0x0F) << 8);
            (prg, chr, m)
        } else {
            // Old dumps sometimes carry junk ("DiskDude!") in bytes 7-15;
            // ignore the upper mapper nibble when bytes 12-15 are not zero.
            let junk = rom[12..16].iter().any(|&b| b != 0);
            let hi = if junk { 0 } else { f7 & 0xF0 };
            (rom[4] as usize, rom[5] as usize, ((f6 >> 4) | hi) as u16)
        };

        let mut offset = 16;
        if f6 & 0x04 != 0 {
            offset += 512; // trainer
        }
        let prg_len = prg_units * 0x4000;
        let chr_len = chr_units * 0x2000;
        if prg_len == 0 || rom.len() < offset + prg_len {
            return None;
        }
        let prg = rom[offset..offset + prg_len].to_vec();
        offset += prg_len;
        let chr_is_ram = chr_len == 0;
        let chr = if chr_is_ram {
            let size = if nes2 {
                let shifts = [rom[11] & 0x0F, rom[11] >> 4];
                let total: usize = shifts
                    .iter()
                    .filter(|&&s| s != 0)
                    .map(|&s| 64usize << s)
                    .sum();
                if total == 0 { 0x2000 } else { total.max(0x2000) }
            } else {
                0x2000
            };
            vec![0; size]
        } else {
            let end = (offset + chr_len).min(rom.len());
            let mut c = rom[offset..end].to_vec();
            c.resize(chr_len, 0);
            c
        };

        // PRG RAM: NES 2.0 states its size; iNES 1.0 can't, so give 8 KiB.
        let prg_ram_len = if nes2 {
            [rom[10] & 0x0F, rom[10] >> 4]
                .iter()
                .filter(|&&s| s != 0)
                .map(|&s| 64usize << s)
                .sum()
        } else {
            0x2000
        };

        let four_screen = f6 & 0x08 != 0;
        let mirroring = if four_screen {
            Mirroring::FourScreen
        } else if f6 & 0x01 != 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let mapper = match mapper_num {
            0 => Mapper::Nrom,
            1 => Mapper::Mmc1(Mmc1 { control: 0x0C, ..Default::default() }),
            2 => Mapper::Uxrom,
            3 => Mapper::Cnrom,
            4 => Mapper::Mmc3(Mmc3::default()),
            _ => return None,
        };

        let mut cart = Cart {
            prg,
            chr,
            chr_is_ram,
            prg_ram: vec![0; prg_ram_len],
            prg_map: [0; 4],
            chr_map: [0; 8],
            mirroring,
            four_screen,
            mapper,
        };
        // Discrete boards power on with every bank register at zero.
        cart.update_banks(0);
        Some(cart)
    }

    fn prg_banks_8k(&self) -> usize {
        self.prg.len() / 0x2000
    }

    fn chr_banks_1k(&self) -> usize {
        (self.chr.len() / 0x400).max(1)
    }

    fn set_prg_8k(&mut self, slot: usize, bank: usize) {
        self.prg_map[slot] = (bank % self.prg_banks_8k()) * 0x2000;
    }

    fn set_prg_16k(&mut self, slot: usize, bank: usize) {
        self.set_prg_8k(slot * 2, bank * 2);
        self.set_prg_8k(slot * 2 + 1, bank * 2 + 1);
    }

    fn set_chr_1k(&mut self, slot: usize, bank: usize) {
        self.chr_map[slot] = (bank % self.chr_banks_1k()) * 0x400;
    }

    fn set_chr_4k(&mut self, slot: usize, bank: usize) {
        for i in 0..4 {
            self.set_chr_1k(slot * 4 + i, bank * 4 + i);
        }
    }

    fn set_chr_8k(&mut self, bank: usize) {
        for i in 0..8 {
            self.set_chr_1k(i, bank * 8 + i);
        }
    }

    /// Recomputes bank offsets from the mapper's registers. `discrete` is
    /// the last value latched by a discrete-logic board (UxROM, CNROM).
    fn update_banks(&mut self, discrete: u8) {
        let last16 = self.prg.len() / 0x4000 - 1;
        match &self.mapper {
            Mapper::Nrom => {
                self.set_prg_16k(0, 0);
                self.set_prg_16k(1, 1);
                self.set_chr_8k(0);
            }
            Mapper::Uxrom => {
                self.set_prg_16k(0, discrete as usize);
                self.set_prg_16k(1, last16);
                self.set_chr_8k(0);
            }
            Mapper::Cnrom => {
                self.set_prg_16k(0, 0);
                self.set_prg_16k(1, 1);
                self.set_chr_8k(discrete as usize);
            }
            Mapper::Mmc1(m) => {
                let (control, chr0, chr1, prg) = (m.control, m.chr0, m.chr1, m.prg);
                self.mirroring = match control & 3 {
                    0 => Mirroring::SingleLow,
                    1 => Mirroring::SingleHigh,
                    2 => Mirroring::Vertical,
                    _ => Mirroring::Horizontal,
                };
                // SUROM: CHR register bit 4 selects the 256 KiB PRG half.
                let outer = if self.prg.len() > 0x40000 { chr0 as usize & 0x10 } else { 0 };
                let bank = (prg as usize & 0x0F) | outer;
                match (control >> 2) & 3 {
                    0 | 1 => {
                        self.set_prg_16k(0, bank & !1);
                        self.set_prg_16k(1, bank | 1);
                    }
                    2 => {
                        self.set_prg_16k(0, outer);
                        self.set_prg_16k(1, bank);
                    }
                    _ => {
                        self.set_prg_16k(0, bank);
                        self.set_prg_16k(1, (last16 & 0x0F) | outer);
                    }
                }
                if control & 0x10 == 0 {
                    self.set_chr_8k(chr0 as usize >> 1);
                } else {
                    self.set_chr_4k(0, chr0 as usize);
                    self.set_chr_4k(1, chr1 as usize);
                }
            }
            Mapper::Mmc3(m) => {
                let r = m.regs;
                let prg_mode = m.select & 0x40 != 0;
                let chr_inv = m.select & 0x80 != 0;
                let second_last = self.prg_banks_8k() - 2;
                let last = self.prg_banks_8k() - 1;
                if prg_mode {
                    self.set_prg_8k(0, second_last);
                    self.set_prg_8k(2, r[6] as usize);
                } else {
                    self.set_prg_8k(0, r[6] as usize);
                    self.set_prg_8k(2, second_last);
                }
                self.set_prg_8k(1, r[7] as usize);
                self.set_prg_8k(3, last);
                let (two_k, one_k) = if chr_inv { (4, 0) } else { (0, 4) };
                self.set_chr_1k(two_k, r[0] as usize & 0xFE);
                self.set_chr_1k(two_k + 1, r[0] as usize | 1);
                self.set_chr_1k(two_k + 2, r[1] as usize & 0xFE);
                self.set_chr_1k(two_k + 3, r[1] as usize | 1);
                for i in 0..4 {
                    self.set_chr_1k(one_k + i, r[2 + i] as usize);
                }
            }
        }
    }

    /// CPU read from $4020-$FFFF. `None` means nothing drives the bus.
    pub fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            0x8000..=0xFFFF => {
                let slot = (addr as usize >> 13) & 3;
                Some(self.prg[self.prg_map[slot] + (addr as usize & 0x1FFF)])
            }
            0x6000..=0x7FFF if !self.prg_ram.is_empty() => {
                Some(self.prg_ram[(addr as usize - 0x6000) % self.prg_ram.len()])
            }
            _ => None,
        }
    }

    /// CPU write to $4020-$FFFF. `cycle` is the CPU cycle count, used by
    /// MMC1 to ignore writes on consecutive cycles.
    pub fn cpu_write(&mut self, addr: u16, val: u8, cycle: u64) {
        if (0x6000..0x8000).contains(&addr) {
            if !self.prg_ram.is_empty() {
                let len = self.prg_ram.len();
                self.prg_ram[(addr as usize - 0x6000) % len] = val;
            }
            return;
        }
        if addr < 0x8000 {
            return;
        }
        let mut discrete = 0;
        match &mut self.mapper {
            Mapper::Nrom => return,
            Mapper::Uxrom | Mapper::Cnrom => discrete = val,
            Mapper::Mmc1(m) => {
                let consecutive = cycle == m.last_write_cycle + 1;
                m.last_write_cycle = cycle;
                if consecutive {
                    return;
                }
                if val & 0x80 != 0 {
                    m.shift = 0;
                    m.count = 0;
                    m.control |= 0x0C;
                } else {
                    m.shift |= (val & 1) << m.count;
                    m.count += 1;
                    if m.count == 5 {
                        let v = m.shift;
                        match (addr >> 13) & 3 {
                            0 => m.control = v,
                            1 => m.chr0 = v,
                            2 => m.chr1 = v,
                            _ => m.prg = v,
                        }
                        m.shift = 0;
                        m.count = 0;
                    }
                }
            }
            Mapper::Mmc3(m) => match (addr >> 13 & 3, addr & 1) {
                (0, 0) => m.select = val,
                (0, _) => m.regs[(m.select & 7) as usize] = val,
                (1, 0) => {
                    if !self.four_screen {
                        self.mirroring =
                            if val & 1 == 0 { Mirroring::Vertical } else { Mirroring::Horizontal };
                    }
                }
                (1, _) => {} // PRG RAM protect: not emulated, RAM always enabled
                (2, 0) => m.irq_latch = val,
                (2, _) => {
                    m.irq_counter = 0;
                    m.irq_reload = true;
                }
                (_, 0) => {
                    m.irq_enabled = false;
                    m.irq_pending = false;
                }
                _ => m.irq_enabled = true,
            },
        }
        self.update_banks(discrete);
    }

    pub fn chr_read(&self, addr: u16) -> u8 {
        let a = addr as usize & 0x1FFF;
        self.chr[(self.chr_map[a >> 10] + (a & 0x3FF)) % self.chr.len()]
    }

    pub fn chr_write(&mut self, addr: u16, val: u8) {
        if self.chr_is_ram {
            let a = addr as usize & 0x1FFF;
            let len = self.chr.len();
            self.chr[(self.chr_map[a >> 10] + (a & 0x3FF)) % len] = val;
        }
    }

    /// Maps a PPU nametable address ($2000-$3EFF) to an offset in the
    /// PPU's 4 KiB nametable memory (the upper 2 KiB is only used by
    /// four-screen boards).
    pub fn nt_index(&self, addr: u16) -> usize {
        let a = addr as usize & 0x0FFF;
        let table = a >> 10;
        let off = a & 0x3FF;
        let page = match self.mirroring {
            Mirroring::Horizontal => table >> 1,
            Mirroring::Vertical => table & 1,
            Mirroring::SingleLow => 0,
            Mirroring::SingleHigh => 1,
            Mirroring::FourScreen => table,
        };
        page * 0x400 + off
    }

    /// Called once per rendered scanline (dot 260) while rendering is on.
    /// Approximates the MMC3's PPU A12 edge counter.
    pub fn scanline_clock(&mut self) {
        if let Mapper::Mmc3(m) = &mut self.mapper {
            if m.irq_counter == 0 || m.irq_reload {
                m.irq_counter = m.irq_latch;
                m.irq_reload = false;
            } else {
                m.irq_counter -= 1;
            }
            if m.irq_counter == 0 && m.irq_enabled {
                m.irq_pending = true;
            }
        }
    }

    pub fn irq(&self) -> bool {
        matches!(&self.mapper, Mapper::Mmc3(m) if m.irq_pending)
    }
}
