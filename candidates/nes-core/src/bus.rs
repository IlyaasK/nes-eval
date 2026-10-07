//! CPU address space. Every CPU bus access goes through `read`/`write`,
//! which first advance the rest of the machine by one CPU cycle (three PPU
//! dots, one APU cycle). Instruction timing therefore falls out of the
//! number of bus accesses the CPU makes, dummy reads included.

use crate::apu::Apu;
use crate::cart::Cart;
use crate::ppu::Ppu;

pub struct Bus {
    ram: [u8; 2048],
    pub ppu: Ppu,
    pub apu: Apu,
    pub cart: Cart,
    pub cycles: u64,
    open_bus: u8,

    keys: u8,
    pad_shift: u8,
    strobe: bool,

    nmi_line_prev: bool,
    /// Latched NMI edge, cleared when the CPU services it.
    pub nmi_pending: bool,
    /// Interrupt inputs as sampled at the start of the latest cycle, i.e. at
    /// the end of the previous one. After an instruction's last cycle they
    /// hold the state the CPU polls (end of the second-to-last cycle).
    pub nmi_poll: bool,
    pub irq_poll: bool,
}

impl Bus {
    pub fn new(cart: Cart) -> Bus {
        Bus {
            ram: [0; 2048],
            ppu: Ppu::new(),
            apu: Apu::new(),
            cart,
            cycles: 0,
            open_bus: 0,
            keys: 0,
            pad_shift: 0,
            strobe: false,
            nmi_line_prev: false,
            nmi_pending: false,
            nmi_poll: false,
            irq_poll: false,
        }
    }

    pub fn set_keys(&mut self, keys: u8) {
        self.keys = keys;
    }

    fn irq_line(&self) -> bool {
        self.apu.irq() || self.cart.irq()
    }

    /// Runs the PPU and APU for one CPU cycle.
    fn clock(&mut self) {
        self.cycles += 1;
        for _ in 0..3 {
            self.ppu.tick(&mut self.cart);
            let line = self.ppu.nmi_line();
            if line && !self.nmi_line_prev {
                self.nmi_pending = true;
            }
            self.nmi_line_prev = line;
        }
        self.apu.clock(self.cycles);
    }

    /// One CPU cycle, plus any DMC sample fetch it triggers (which stalls
    /// the CPU for four cycles).
    pub fn tick(&mut self) {
        self.nmi_poll = self.nmi_pending;
        self.irq_poll = self.irq_line();
        self.clock();
        if let Some(addr) = self.apu.dmc_fetch_addr() {
            for _ in 0..3 {
                self.clock();
            }
            let byte = self.cart.cpu_read(addr).unwrap_or(self.open_bus);
            self.clock();
            self.apu.dmc_fill(byte);
        }
    }

    /// Side-effect-free read used for the reset vector.
    pub fn peek(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF],
            0x4020..=0xFFFF => self.cart.cpu_read(addr).unwrap_or(0),
            _ => 0,
        }
    }

    pub fn read(&mut self, addr: u16) -> u8 {
        self.tick();
        let val = match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF],
            0x2000..=0x3FFF => {
                let v = self.ppu.read_reg(addr, &self.cart);
                if self.ppu.take_nmi_cancel() {
                    self.nmi_pending = false;
                }
                v
            }
            0x4015 => self.apu.read_status() | (self.open_bus & 0x20),
            0x4016 => {
                if self.strobe {
                    self.pad_shift = self.keys;
                }
                let bit = self.pad_shift & 1;
                self.pad_shift = (self.pad_shift >> 1) | 0x80;
                bit | (self.open_bus & 0xE0)
            }
            0x4017 => self.open_bus & 0xE0,
            0x4000..=0x401F => self.open_bus,
            _ => self.cart.cpu_read(addr).unwrap_or(self.open_bus),
        };
        // $4015 reads don't drive the external data bus.
        if addr != 0x4015 {
            self.open_bus = val;
        }
        val
    }

    pub fn write(&mut self, addr: u16, val: u8) {
        self.tick();
        self.open_bus = val;
        match addr {
            0x0000..=0x1FFF => self.ram[addr as usize & 0x7FF] = val,
            0x2000..=0x3FFF => self.ppu.write_reg(addr, val, &mut self.cart),
            0x4014 => self.oam_dma(val),
            0x4016 => {
                self.strobe = val & 1 != 0;
                if self.strobe {
                    self.pad_shift = self.keys;
                }
            }
            0x4000..=0x4017 => self.apu.write(addr, val, self.cycles),
            0x4018..=0x401F => {}
            _ => self.cart.cpu_write(addr, val, self.cycles),
        }
    }

    /// OAM DMA: one idle cycle (two if it starts on an odd cycle), then 256
    /// read/write pairs into $2004.
    fn oam_dma(&mut self, page: u8) {
        self.tick();
        if self.cycles & 1 == 1 {
            self.tick();
        }
        let base = (page as u16) << 8;
        for i in 0..256 {
            let b = self.read(base + i);
            self.write(0x2004, b);
        }
    }
}
