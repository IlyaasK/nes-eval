//! 2C02 PPU, stepped one dot at a time.
//!
//! The background uses the hardware's fetch pipeline (nametable,
//! attribute, two pattern bytes every 8 dots into 16-bit shift registers)
//! and the loopy v/t/x/w scroll registers. Sprites are evaluated in one go
//! at dot 257 of each line for the next line and their patterns fetched at
//! the same moment; the per-dot sprite evaluation state machine and its
//! overflow bug are not modelled.

use crate::cart::Cart;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;
pub const PIXELS: usize = WIDTH * HEIGHT;

const PRE_RENDER: u16 = 261;

pub struct Ppu {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    pub oam: [u8; 256],
    vram: [u8; 4096],
    palette: [u8; 32],

    v: u16,
    t: u16,
    x: u8,
    w: bool,
    read_buffer: u8,
    io_latch: u8,

    pub scanline: u16,
    pub dot: u16,
    odd_frame: bool,
    suppress_vbl: bool,
    nmi_cancel: bool,
    pub frame_done: bool,

    // Background pipeline.
    nt_byte: u8,
    at_bits: u8,
    pat_lo: u8,
    pat_hi: u8,
    bg_lo: u16,
    bg_hi: u16,
    at_lo: u16,
    at_hi: u16,

    // Sprites for the line being drawn.
    sp_count: usize,
    sp_lo: [u8; 8],
    sp_hi: [u8; 8],
    sp_attr: [u8; 8],
    sp_x: [u8; 8],
    sp_zero_in_line: bool,

    frame: Box<[u16; PIXELS]>,
    pub output: Box<[u16; PIXELS]>,
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam: [0; 256],
            vram: [0; 4096],
            palette: [0; 32],
            v: 0,
            t: 0,
            x: 0,
            w: false,
            read_buffer: 0,
            io_latch: 0,
            scanline: 0,
            dot: 0,
            odd_frame: false,
            suppress_vbl: false,
            nmi_cancel: false,
            frame_done: false,
            nt_byte: 0,
            at_bits: 0,
            pat_lo: 0,
            pat_hi: 0,
            bg_lo: 0,
            bg_hi: 0,
            at_lo: 0,
            at_hi: 0,
            sp_count: 0,
            sp_lo: [0; 8],
            sp_hi: [0; 8],
            sp_attr: [0; 8],
            sp_x: [0; 8],
            sp_zero_in_line: false,
            frame: Box::new([0; PIXELS]),
            output: Box::new([0; PIXELS]),
        }
    }

    /// State of the /NMI output (active = true).
    pub fn nmi_line(&self) -> bool {
        self.status & 0x80 != 0 && self.ctrl & 0x80 != 0
    }

    /// True once after a $2002 read that should cancel an already-raised NMI.
    pub fn take_nmi_cancel(&mut self) -> bool {
        std::mem::take(&mut self.nmi_cancel)
    }

    fn rendering(&self) -> bool {
        self.mask & 0x18 != 0
    }

    fn sprite_height(&self) -> u16 {
        if self.ctrl & 0x20 != 0 { 16 } else { 8 }
    }

    fn palette_index(addr: u16) -> usize {
        let a = addr as usize & 0x1F;
        if a & 0x13 == 0x10 { a & 0x0F } else { a }
    }

    fn mem_read(&self, addr: u16, cart: &Cart) -> u8 {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => cart.chr_read(addr),
            0x2000..=0x3EFF => self.vram[cart.nt_index(addr)],
            _ => self.palette[Self::palette_index(addr)],
        }
    }

    fn mem_write(&mut self, addr: u16, val: u8, cart: &mut Cart) {
        let addr = addr & 0x3FFF;
        match addr {
            0x0000..=0x1FFF => cart.chr_write(addr, val),
            0x2000..=0x3EFF => self.vram[cart.nt_index(addr)] = val,
            _ => self.palette[Self::palette_index(addr)] = val & 0x3F,
        }
    }

    // ── CPU interface ───────────────────────────────────────────────────

    pub fn read_reg(&mut self, reg: u16, cart: &Cart) -> u8 {
        let val = match reg & 7 {
            2 => {
                // Reading one dot before VBlank is set returns it clear and
                // suppresses it for this frame; reading just after it was
                // set returns it set but cancels the NMI.
                if self.scanline == 241 && self.dot == 1 {
                    self.suppress_vbl = true;
                }
                if self.scanline == 241 && (2..=3).contains(&self.dot) {
                    self.nmi_cancel = true;
                }
                let v = (self.status & 0xE0) | (self.io_latch & 0x1F);
                self.status &= !0x80;
                self.w = false;
                v
            }
            4 => {
                let b = self.oam[self.oam_addr as usize];
                if self.oam_addr & 3 == 2 { b & 0xE3 } else { b }
            }
            7 => {
                let addr = self.v & 0x3FFF;
                let v = if addr >= 0x3F00 {
                    // Palette reads are immediate; the buffer gets the
                    // nametable byte underneath.
                    self.read_buffer = self.mem_read(addr - 0x1000, cart);
                    (self.mem_read(addr, cart) & 0x3F) | (self.io_latch & 0xC0)
                } else {
                    let b = self.read_buffer;
                    self.read_buffer = self.mem_read(addr, cart);
                    b
                };
                self.increment_v();
                v
            }
            _ => self.io_latch,
        };
        self.io_latch = val;
        val
    }

    pub fn write_reg(&mut self, reg: u16, val: u8, cart: &mut Cart) {
        self.io_latch = val;
        match reg & 7 {
            0 => {
                self.ctrl = val;
                self.t = (self.t & 0xF3FF) | ((val as u16 & 3) << 10);
            }
            1 => self.mask = val,
            3 => self.oam_addr = val,
            4 => {
                self.oam[self.oam_addr as usize] = val;
                self.oam_addr = self.oam_addr.wrapping_add(1);
            }
            5 => {
                if !self.w {
                    self.t = (self.t & 0xFFE0) | (val as u16 >> 3);
                    self.x = val & 7;
                } else {
                    self.t = (self.t & 0x8C1F) | ((val as u16 & 7) << 12) | ((val as u16 & 0xF8) << 2);
                }
                self.w = !self.w;
            }
            6 => {
                if !self.w {
                    self.t = (self.t & 0x80FF) | ((val as u16 & 0x3F) << 8);
                } else {
                    self.t = (self.t & 0xFF00) | val as u16;
                    self.v = self.t;
                }
                self.w = !self.w;
            }
            7 => {
                self.mem_write(self.v, val, cart);
                self.increment_v();
            }
            _ => {}
        }
    }

    fn increment_v(&mut self) {
        let step = if self.ctrl & 0x04 != 0 { 32 } else { 1 };
        self.v = self.v.wrapping_add(step) & 0x7FFF;
    }

    // ── Scroll helpers (loopy) ──────────────────────────────────────────

    fn inc_coarse_x(&mut self) {
        if self.v & 0x001F == 31 {
            self.v &= !0x001F;
            self.v ^= 0x0400;
        } else {
            self.v += 1;
        }
    }

    fn inc_y(&mut self) {
        if self.v & 0x7000 != 0x7000 {
            self.v += 0x1000;
        } else {
            self.v &= !0x7000;
            let mut y = (self.v & 0x03E0) >> 5;
            if y == 29 {
                y = 0;
                self.v ^= 0x0800;
            } else if y == 31 {
                y = 0;
            } else {
                y += 1;
            }
            self.v = (self.v & !0x03E0) | (y << 5);
        }
    }

    fn copy_x(&mut self) {
        self.v = (self.v & !0x041F) | (self.t & 0x041F);
    }

    fn copy_y(&mut self) {
        self.v = (self.v & !0x7BE0) | (self.t & 0x7BE0);
    }

    // ── Rendering ───────────────────────────────────────────────────────

    fn load_bg_shifters(&mut self) {
        self.bg_lo = (self.bg_lo & 0xFF00) | self.pat_lo as u16;
        self.bg_hi = (self.bg_hi & 0xFF00) | self.pat_hi as u16;
        self.at_lo = (self.at_lo & 0xFF00) | if self.at_bits & 1 != 0 { 0xFF } else { 0 };
        self.at_hi = (self.at_hi & 0xFF00) | if self.at_bits & 2 != 0 { 0xFF } else { 0 };
    }

    fn shift_bg(&mut self) {
        self.bg_lo <<= 1;
        self.bg_hi <<= 1;
        self.at_lo <<= 1;
        self.at_hi <<= 1;
    }

    fn bg_fetch(&mut self, cart: &Cart) {
        match (self.dot - 1) % 8 {
            0 => {
                self.load_bg_shifters();
                self.nt_byte = self.mem_read(0x2000 | (self.v & 0x0FFF), cart);
            }
            2 => {
                let v = self.v;
                let addr = 0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                let shift = ((v >> 4) & 4) | (v & 2);
                self.at_bits = (self.mem_read(addr, cart) >> shift) & 3;
            }
            4 => {
                let addr = self.bg_pattern_addr();
                self.pat_lo = self.mem_read(addr, cart);
            }
            6 => {
                let addr = self.bg_pattern_addr() + 8;
                self.pat_hi = self.mem_read(addr, cart);
            }
            7 => self.inc_coarse_x(),
            _ => {}
        }
    }

    fn bg_pattern_addr(&self) -> u16 {
        ((self.ctrl as u16 & 0x10) << 8) + self.nt_byte as u16 * 16 + ((self.v >> 12) & 7)
    }

    /// Selects the (up to eight) sprites on the next line and fetches
    /// their pattern bytes.
    fn evaluate_sprites(&mut self, cart: &Cart) {
        let line = self.scanline;
        let height = self.sprite_height();
        let mut count = 0;
        self.sp_zero_in_line = false;
        for i in 0..64 {
            let y = self.oam[i * 4] as u16;
            if line < y || line - y >= height {
                continue;
            }
            if count == 8 {
                self.status |= 0x20;
                break;
            }
            let tile = self.oam[i * 4 + 1] as u16;
            let attr = self.oam[i * 4 + 2];
            let mut row = line - y;
            if attr & 0x80 != 0 {
                row = height - 1 - row;
            }
            let addr = if height == 16 {
                let table = (tile & 1) * 0x1000;
                let mut t = tile & 0xFE;
                if row >= 8 {
                    t += 1;
                    row -= 8;
                }
                table + t * 16 + row
            } else {
                ((self.ctrl as u16 & 0x08) << 9) + tile * 16 + row
            };
            let mut lo = self.mem_read(addr, cart);
            let mut hi = self.mem_read(addr + 8, cart);
            if attr & 0x40 != 0 {
                lo = lo.reverse_bits();
                hi = hi.reverse_bits();
            }
            if i == 0 {
                self.sp_zero_in_line = true;
            }
            self.sp_lo[count] = lo;
            self.sp_hi[count] = hi;
            self.sp_attr[count] = attr;
            self.sp_x[count] = self.oam[i * 4 + 3];
            count += 1;
        }
        self.sp_count = count;
    }

    fn render_pixel(&mut self) {
        let x = (self.dot - 1) as usize;
        let mask = self.mask;

        let mut bg = 0u8;
        let mut bg_pal = 0u8;
        if mask & 0x08 != 0 && (x >= 8 || mask & 0x02 != 0) {
            let bit = 15 - self.x as u16;
            bg = (((self.bg_hi >> bit) & 1) << 1 | ((self.bg_lo >> bit) & 1)) as u8;
            bg_pal = (((self.at_hi >> bit) & 1) << 1 | ((self.at_lo >> bit) & 1)) as u8;
        }

        let mut sp = 0u8;
        let mut sp_pal = 0u8;
        let mut sp_behind = false;
        if mask & 0x10 != 0 && (x >= 8 || mask & 0x04 != 0) {
            for i in 0..self.sp_count {
                let off = x as i32 - self.sp_x[i] as i32;
                if !(0..8).contains(&off) {
                    continue;
                }
                let bit = 7 - off;
                let p = ((self.sp_hi[i] >> bit) & 1) << 1 | ((self.sp_lo[i] >> bit) & 1);
                if p == 0 {
                    continue;
                }
                if i == 0 && self.sp_zero_in_line && bg != 0 && x != 255 {
                    self.status |= 0x40;
                }
                sp = p;
                sp_pal = (self.sp_attr[i] & 3) + 4;
                sp_behind = self.sp_attr[i] & 0x20 != 0;
                break;
            }
        }

        let addr = match (bg, sp) {
            (0, 0) => 0,
            (0, _) => sp_pal * 4 + sp,
            (_, 0) => bg_pal * 4 + bg,
            _ if sp_behind => bg_pal * 4 + bg,
            _ => sp_pal * 4 + sp,
        };
        let colour = self.palette[Self::palette_index(addr as u16)];
        self.put_pixel(x, colour);
    }

    fn put_pixel(&mut self, x: usize, colour: u8) {
        let colour = if self.mask & 0x01 != 0 { colour & 0x30 } else { colour & 0x3F };
        let emphasis = (self.mask >> 5) as u16;
        self.frame[self.scanline as usize * WIDTH + x] = (emphasis << 6) | colour as u16;
    }

    /// Advances one PPU dot.
    pub fn tick(&mut self, cart: &mut Cart) {
        let line = self.scanline;
        let dot = self.dot;
        let visible = line < 240;
        let pre = line == PRE_RENDER;

        if self.rendering() && (visible || pre) {
            if (2..=257).contains(&dot) || (322..=337).contains(&dot) {
                self.shift_bg();
            }
            if (1..=256).contains(&dot) || (321..=336).contains(&dot) {
                self.bg_fetch(cart);
            }
            if dot == 256 {
                self.inc_y();
            }
            if dot == 257 {
                self.load_bg_shifters();
                self.copy_x();
                if visible {
                    self.evaluate_sprites(cart);
                } else {
                    self.sp_count = 0;
                    self.sp_zero_in_line = false;
                }
            }
            if pre && (280..=304).contains(&dot) {
                self.copy_y();
            }
            if dot == 260 {
                cart.scanline_clock();
            }
        } else if visible && dot == 257 {
            self.sp_count = 0;
            self.sp_zero_in_line = false;
        }

        if visible && (1..=256).contains(&dot) {
            if self.rendering() {
                self.render_pixel();
            } else {
                // With rendering off the PPU outputs the backdrop, or the
                // palette entry v points at if v is inside palette RAM.
                let addr = if self.v & 0x3F00 == 0x3F00 { self.v } else { 0x3F00 };
                let colour = self.palette[Self::palette_index(addr)];
                self.put_pixel((dot - 1) as usize, colour);
            }
        }

        if line == 241 && dot == 1 {
            if !self.suppress_vbl {
                self.status |= 0x80;
            }
            self.suppress_vbl = false;
        }
        if pre && dot == 1 {
            self.status &= !0xE0;
        }

        // Advance; on odd frames with rendering on, the pre-render line
        // skips its last dot.
        self.dot += 1;
        if pre && self.dot == 340 && self.odd_frame && self.rendering() {
            self.dot = 341;
        }
        if self.dot > 340 {
            self.dot = 0;
            self.scanline += 1;
            if self.scanline > PRE_RENDER {
                self.scanline = 0;
                self.odd_frame = !self.odd_frame;
                self.frame_done = true;
                self.output.copy_from_slice(&self.frame[..]);
            }
        }
    }
}
