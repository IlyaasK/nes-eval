//! 2A03 APU: two pulse channels, triangle, noise, DMC, frame counter, the
//! nonlinear mixer, and a box-filter resampler to 44.1 kHz.

const LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const DUTY: [[u8; 8]; 4] = [
    [0, 1, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 0, 0, 0],
    [1, 0, 0, 1, 1, 1, 1, 1],
];

const TRIANGLE: [u8; 32] = [
    15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,
    12, 13, 14, 15,
];

/// Noise timer periods in CPU cycles (NTSC).
const NOISE_PERIOD: [u16; 16] =
    [4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068];

/// DMC output periods in CPU cycles (NTSC).
const DMC_RATE: [u16; 16] =
    [428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54];

/// CPU clock is 236.25 MHz / 11 / 12 = 39375000 / 22 Hz. The resampler
/// steps a fixed-point phase by `SAMPLE_RATE * 22` per CPU cycle and emits
/// a sample each time it passes 39375000, which is exact.
pub const SAMPLE_RATE: u32 = 44_100;
const PHASE_STEP: u32 = SAMPLE_RATE * 22;
const PHASE_WRAP: u32 = 39_375_000;

#[derive(Default)]
struct Envelope {
    start: bool,
    looping: bool,
    constant: bool,
    volume: u8,
    divider: u8,
    decay: u8,
}

impl Envelope {
    fn write(&mut self, val: u8) {
        self.looping = val & 0x20 != 0;
        self.constant = val & 0x10 != 0;
        self.volume = val & 0x0F;
    }

    fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay = 15;
            self.divider = self.volume;
        } else if self.divider == 0 {
            self.divider = self.volume;
            if self.decay > 0 {
                self.decay -= 1;
            } else if self.looping {
                self.decay = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    fn output(&self) -> u8 {
        if self.constant { self.volume } else { self.decay }
    }
}

#[derive(Default)]
struct Pulse {
    ones_complement: bool,
    enabled: bool,
    duty: u8,
    halt: bool,
    env: Envelope,
    sweep_enabled: bool,
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_reload: bool,
    sweep_divider: u8,
    period: u16,
    timer: u16,
    seq: u8,
    length: u8,
}

impl Pulse {
    fn write(&mut self, reg: u16, val: u8) {
        match reg & 3 {
            0 => {
                self.duty = val >> 6;
                self.halt = val & 0x20 != 0;
                self.env.write(val);
            }
            1 => {
                self.sweep_enabled = val & 0x80 != 0;
                self.sweep_period = (val >> 4) & 7;
                self.sweep_negate = val & 0x08 != 0;
                self.sweep_shift = val & 7;
                self.sweep_reload = true;
            }
            2 => self.period = (self.period & 0x700) | val as u16,
            _ => {
                self.period = (self.period & 0xFF) | ((val as u16 & 7) << 8);
                if self.enabled {
                    self.length = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.seq = 0;
                self.env.start = true;
            }
        }
    }

    fn sweep_target(&self) -> u16 {
        let change = self.period >> self.sweep_shift;
        if self.sweep_negate {
            let sub = change + self.ones_complement as u16;
            self.period.saturating_sub(sub)
        } else {
            self.period + change
        }
    }

    fn muted(&self) -> bool {
        self.period < 8 || (!self.sweep_negate && self.sweep_target() > 0x7FF)
    }

    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            self.seq = (self.seq + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }

    fn clock_sweep(&mut self) {
        if self.sweep_divider == 0 && self.sweep_enabled && self.sweep_shift > 0 && !self.muted() {
            self.period = self.sweep_target();
        }
        if self.sweep_divider == 0 || self.sweep_reload {
            self.sweep_divider = self.sweep_period;
            self.sweep_reload = false;
        } else {
            self.sweep_divider -= 1;
        }
    }

    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }

    fn output(&self) -> u8 {
        if self.length == 0 || self.muted() || DUTY[self.duty as usize][self.seq as usize] == 0 {
            0
        } else {
            self.env.output()
        }
    }
}

#[derive(Default)]
struct Triangle {
    enabled: bool,
    control: bool,
    linear_load: u8,
    linear: u8,
    linear_reload: bool,
    period: u16,
    timer: u16,
    seq: u8,
    length: u8,
}

impl Triangle {
    fn write(&mut self, reg: u16, val: u8) {
        match reg & 3 {
            0 => {
                self.control = val & 0x80 != 0;
                self.linear_load = val & 0x7F;
            }
            1 => {}
            2 => self.period = (self.period & 0x700) | val as u16,
            _ => {
                self.period = (self.period & 0xFF) | ((val as u16 & 7) << 8);
                if self.enabled {
                    self.length = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.linear_reload = true;
            }
        }
    }

    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period;
            if self.length > 0 && self.linear > 0 {
                self.seq = (self.seq + 1) & 31;
            }
        } else {
            self.timer -= 1;
        }
    }

    fn clock_linear(&mut self) {
        if self.linear_reload {
            self.linear = self.linear_load;
        } else if self.linear > 0 {
            self.linear -= 1;
        }
        if !self.control {
            self.linear_reload = false;
        }
    }

    fn clock_length(&mut self) {
        if !self.control && self.length > 0 {
            self.length -= 1;
        }
    }

    fn output(&self) -> u8 {
        TRIANGLE[self.seq as usize]
    }
}

struct Noise {
    enabled: bool,
    halt: bool,
    env: Envelope,
    mode: bool,
    period: u16,
    timer: u16,
    lfsr: u16,
    length: u8,
}

impl Noise {
    fn write(&mut self, reg: u16, val: u8) {
        match reg & 3 {
            0 => {
                self.halt = val & 0x20 != 0;
                self.env.write(val);
            }
            1 => {}
            2 => {
                self.mode = val & 0x80 != 0;
                self.period = NOISE_PERIOD[(val & 0x0F) as usize];
            }
            _ => {
                if self.enabled {
                    self.length = LENGTH_TABLE[(val >> 3) as usize];
                }
                self.env.start = true;
            }
        }
    }

    fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.period - 1;
            let tap = if self.mode { 6 } else { 1 };
            let feedback = (self.lfsr ^ (self.lfsr >> tap)) & 1;
            self.lfsr = (self.lfsr >> 1) | (feedback << 14);
        } else {
            self.timer -= 1;
        }
    }

    fn clock_length(&mut self) {
        if !self.halt && self.length > 0 {
            self.length -= 1;
        }
    }

    fn output(&self) -> u8 {
        if self.length == 0 || self.lfsr & 1 != 0 { 0 } else { self.env.output() }
    }
}

struct Dmc {
    irq_enabled: bool,
    looping: bool,
    period: u16,
    timer: u16,
    level: u8,
    sample_addr: u16,
    sample_len: u16,
    addr: u16,
    remaining: u16,
    shift: u8,
    bits: u8,
    silence: bool,
    buffer: Option<u8>,
    irq: bool,
}

impl Dmc {
    fn write(&mut self, reg: u16, val: u8) {
        match reg & 3 {
            0 => {
                self.irq_enabled = val & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq = false;
                }
                self.looping = val & 0x40 != 0;
                self.period = DMC_RATE[(val & 0x0F) as usize];
            }
            1 => self.level = val & 0x7F,
            2 => self.sample_addr = 0xC000 | ((val as u16) << 6),
            _ => self.sample_len = ((val as u16) << 4) | 1,
        }
    }

    fn restart(&mut self) {
        self.addr = self.sample_addr;
        self.remaining = self.sample_len;
    }

    fn clock_timer(&mut self) {
        if self.timer > 0 {
            self.timer -= 1;
            return;
        }
        self.timer = self.period - 1;
        if !self.silence {
            if self.shift & 1 != 0 {
                if self.level <= 125 {
                    self.level += 2;
                }
            } else if self.level >= 2 {
                self.level -= 2;
            }
        }
        self.shift >>= 1;
        self.bits -= 1;
        if self.bits == 0 {
            self.bits = 8;
            match self.buffer.take() {
                Some(b) => {
                    self.silence = false;
                    self.shift = b;
                }
                None => self.silence = true,
            }
        }
    }
}

pub struct Apu {
    pulse1: Pulse,
    pulse2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,

    five_step: bool,
    irq_inhibit: bool,
    frame_irq: bool,
    frame_cycle: u32,
    pending_reset: u8,

    pulse_table: [f32; 31],
    tnd_table: [f32; 203],
    phase: u32,
    acc: f32,
    acc_n: u32,
    hp_prev_in: f32,
    hp_prev_out: f32,
    pub samples: Vec<i16>,
}

impl Apu {
    pub fn new() -> Apu {
        let mut pulse_table = [0f32; 31];
        for (n, v) in pulse_table.iter_mut().enumerate().skip(1) {
            *v = 95.52 / (8128.0 / n as f32 + 100.0);
        }
        let mut tnd_table = [0f32; 203];
        for (n, v) in tnd_table.iter_mut().enumerate().skip(1) {
            *v = 163.67 / (24329.0 / n as f32 + 100.0);
        }
        Apu {
            pulse1: Pulse { ones_complement: true, ..Default::default() },
            pulse2: Pulse::default(),
            triangle: Triangle::default(),
            noise: Noise {
                enabled: false,
                halt: false,
                env: Envelope::default(),
                mode: false,
                period: NOISE_PERIOD[0],
                timer: 0,
                lfsr: 1,
                length: 0,
            },
            dmc: Dmc {
                irq_enabled: false,
                looping: false,
                period: DMC_RATE[0],
                timer: DMC_RATE[0] - 1,
                level: 0,
                sample_addr: 0xC000,
                sample_len: 1,
                addr: 0xC000,
                remaining: 0,
                shift: 0,
                bits: 8,
                silence: true,
                buffer: None,
                irq: false,
            },
            five_step: false,
            irq_inhibit: false,
            frame_irq: false,
            // Power-on behaves as if $4017 = $00 was written ~10 cycles
            // before the first instruction.
            frame_cycle: 10,
            pending_reset: 0,
            pulse_table,
            tnd_table,
            phase: 0,
            acc: 0.0,
            acc_n: 0,
            hp_prev_in: 0.0,
            hp_prev_out: 0.0,
            samples: Vec::with_capacity(4096),
        }
    }

    pub fn irq(&self) -> bool {
        self.frame_irq || self.dmc.irq
    }

    pub fn read_status(&mut self) -> u8 {
        let mut v = 0;
        if self.pulse1.length > 0 {
            v |= 0x01;
        }
        if self.pulse2.length > 0 {
            v |= 0x02;
        }
        if self.triangle.length > 0 {
            v |= 0x04;
        }
        if self.noise.length > 0 {
            v |= 0x08;
        }
        if self.dmc.remaining > 0 {
            v |= 0x10;
        }
        if self.frame_irq {
            v |= 0x40;
        }
        if self.dmc.irq {
            v |= 0x80;
        }
        self.frame_irq = false;
        v
    }

    pub fn write(&mut self, addr: u16, val: u8, cycle: u64) {
        match addr {
            0x4000..=0x4003 => self.pulse1.write(addr, val),
            0x4004..=0x4007 => self.pulse2.write(addr, val),
            0x4008..=0x400B => self.triangle.write(addr, val),
            0x400C..=0x400F => self.noise.write(addr, val),
            0x4010..=0x4013 => self.dmc.write(addr, val),
            0x4015 => {
                self.pulse1.enabled = val & 0x01 != 0;
                self.pulse2.enabled = val & 0x02 != 0;
                self.triangle.enabled = val & 0x04 != 0;
                self.noise.enabled = val & 0x08 != 0;
                if !self.pulse1.enabled {
                    self.pulse1.length = 0;
                }
                if !self.pulse2.enabled {
                    self.pulse2.length = 0;
                }
                if !self.triangle.enabled {
                    self.triangle.length = 0;
                }
                if !self.noise.enabled {
                    self.noise.length = 0;
                }
                self.dmc.irq = false;
                if val & 0x10 == 0 {
                    self.dmc.remaining = 0;
                } else if self.dmc.remaining == 0 {
                    self.dmc.restart();
                }
            }
            0x4017 => {
                self.five_step = val & 0x80 != 0;
                self.irq_inhibit = val & 0x40 != 0;
                if self.irq_inhibit {
                    self.frame_irq = false;
                }
                // The sequencer restarts 3 or 4 CPU cycles later depending
                // on which half of the APU cycle the write lands in.
                self.pending_reset = if cycle & 1 == 1 { 4 } else { 3 };
            }
            _ => {}
        }
    }

    /// Address the DMC wants to read, if its sample buffer is empty and
    /// bytes remain. The bus performs the read and stalls the CPU.
    pub fn dmc_fetch_addr(&self) -> Option<u16> {
        if self.dmc.buffer.is_none() && self.dmc.remaining > 0 { Some(self.dmc.addr) } else { None }
    }

    pub fn dmc_fill(&mut self, byte: u8) {
        let d = &mut self.dmc;
        d.buffer = Some(byte);
        d.addr = if d.addr == 0xFFFF { 0x8000 } else { d.addr + 1 };
        d.remaining -= 1;
        if d.remaining == 0 {
            if d.looping {
                d.restart();
            } else if d.irq_enabled {
                d.irq = true;
            }
        }
    }

    fn quarter_frame(&mut self) {
        self.pulse1.env.clock();
        self.pulse2.env.clock();
        self.noise.env.clock();
        self.triangle.clock_linear();
    }

    fn half_frame(&mut self) {
        self.pulse1.clock_length();
        self.pulse2.clock_length();
        self.triangle.clock_length();
        self.noise.clock_length();
        self.pulse1.clock_sweep();
        self.pulse2.clock_sweep();
    }

    fn clock_frame_counter(&mut self) {
        if self.pending_reset > 0 {
            self.pending_reset -= 1;
            if self.pending_reset == 0 {
                self.frame_cycle = 0;
                if self.five_step {
                    self.quarter_frame();
                    self.half_frame();
                }
                return;
            }
        }
        self.frame_cycle += 1;
        let irq_allowed = !self.five_step && !self.irq_inhibit;
        match self.frame_cycle {
            7457 | 22371 => self.quarter_frame(),
            14913 => {
                self.quarter_frame();
                self.half_frame();
            }
            29828 if irq_allowed => self.frame_irq = true,
            29829 if !self.five_step => {
                self.quarter_frame();
                self.half_frame();
                if irq_allowed {
                    self.frame_irq = true;
                }
            }
            29830 if !self.five_step => {
                if irq_allowed {
                    self.frame_irq = true;
                }
                self.frame_cycle = 0;
            }
            37281 => {
                self.quarter_frame();
                self.half_frame();
            }
            37282 => self.frame_cycle = 0,
            _ => {}
        }
    }

    /// Advances one CPU cycle. `cycle` is the CPU cycle count.
    pub fn clock(&mut self, cycle: u64) {
        self.clock_frame_counter();
        self.triangle.clock_timer();
        self.noise.clock_timer();
        self.dmc.clock_timer();
        if cycle & 1 == 0 {
            self.pulse1.clock_timer();
            self.pulse2.clock_timer();
        }
        self.mix();
    }

    fn mix(&mut self) {
        let p = (self.pulse1.output() + self.pulse2.output()) as usize;
        let tnd = 3 * self.triangle.output() as usize
            + 2 * self.noise.output() as usize
            + self.dmc.level as usize;
        self.acc += self.pulse_table[p] + self.tnd_table[tnd];
        self.acc_n += 1;

        self.phase += PHASE_STEP;
        if self.phase >= PHASE_WRAP {
            self.phase -= PHASE_WRAP;
            let x = self.acc / self.acc_n as f32;
            self.acc = 0.0;
            self.acc_n = 0;
            // First-order high-pass at ~90 Hz, as on the console's output.
            let y = x - self.hp_prev_in + 0.987_26 * self.hp_prev_out;
            self.hp_prev_in = x;
            self.hp_prev_out = y;
            let s = (y * 32767.0).clamp(-32768.0, 32767.0) as i16;
            if self.samples.len() < self.samples.capacity() {
                self.samples.push(s);
            }
        }
    }
}
