//! nes-core: a plain from-scratch NES emulator, NES Eval's baseline
//! candidate. Implements the C ABI in spec/ABI.md for wasm32-unknown-unknown.

pub mod apu;
pub mod bus;
pub mod cart;
pub mod cpu;
pub mod ppu;

use std::cell::UnsafeCell;

use bus::Bus;
use cart::Cart;
use cpu::Cpu;
use ppu::PIXELS;

/// One NES console.
pub struct Nes {
    pub cpu: Cpu,
    pub bus: Bus,
}

impl Nes {
    /// Parses the ROM and powers on in the state spec/ABI.md requires.
    pub fn new(rom: &[u8]) -> Option<Nes> {
        let cart = Cart::new(rom)?;
        let mut bus = Bus::new(cart);
        // The reset sequence has run with the PPU clocking from scanline 0,
        // dot 0; the first instruction fetch happens at dot 25.
        bus.ppu.scanline = 0;
        bus.ppu.dot = 25;
        bus.cycles = 8;
        let cpu = Cpu::new(&bus);
        Some(Nes { cpu, bus })
    }

    pub fn set_keys(&mut self, keys: u8) {
        self.bus.set_keys(keys);
    }

    /// Runs until the PPU moves from the pre-render line to scanline 0.
    pub fn run_frame(&mut self) {
        self.bus.ppu.frame_done = false;
        while !self.bus.ppu.frame_done {
            self.cpu.step(&mut self.bus);
        }
    }

    /// The last completed frame: 256x240 `(emphasis << 6) | colour` values.
    pub fn frame(&self) -> &[u16; PIXELS] {
        &self.bus.ppu.output
    }

    /// Samples produced so far; the caller drains them by clearing the Vec.
    pub fn audio(&mut self) -> &mut Vec<i16> {
        &mut self.bus.apu.samples
    }

    /// Side-effect-free CPU memory read (RAM, cartridge), for tools.
    pub fn peek(&self, addr: u16) -> u8 {
        self.bus.peek(addr)
    }
}

// ── wasm ABI ────────────────────────────────────────────────────────────

const ROM_BUFFER_SIZE: usize = 8 * 1024 * 1024;

struct State {
    nes: Option<Nes>,
    rom: Vec<u8>,
    rom_len: usize,
    framebuffer: [u16; PIXELS],
    audio_out: Vec<i16>,
    audio_drained: bool,
}

struct Global(UnsafeCell<Option<Box<State>>>);

// wasm32-unknown-unknown is single-threaded; the grader calls one export
// at a time.
unsafe impl Sync for Global {}

static STATE: Global = Global(UnsafeCell::new(None));

fn state() -> &'static mut State {
    // SAFETY: single-threaded, and no export holds the reference across
    // another export call.
    unsafe {
        let slot = &mut *STATE.0.get();
        slot.get_or_insert_with(|| {
            Box::new(State {
                nes: None,
                rom: vec![0; ROM_BUFFER_SIZE],
                rom_len: 0,
                framebuffer: [0; PIXELS],
                audio_out: Vec::with_capacity(8192),
                audio_drained: false,
            })
        })
    }
}

fn power_on(st: &mut State) -> i32 {
    st.nes = Nes::new(&st.rom[..st.rom_len]);
    st.framebuffer.fill(0);
    st.audio_out.clear();
    st.audio_drained = false;
    st.nes.is_some() as i32
}

#[no_mangle]
pub extern "C" fn emu_init() -> i32 {
    state();
    1
}

#[no_mangle]
pub extern "C" fn emu_rom_buffer() -> *mut u8 {
    state().rom.as_mut_ptr()
}

#[no_mangle]
pub extern "C" fn emu_load_rom(len: i32) -> i32 {
    let st = state();
    if len <= 0 || len as usize > ROM_BUFFER_SIZE {
        return 0;
    }
    st.rom_len = len as usize;
    power_on(st)
}

#[no_mangle]
pub extern "C" fn emu_reset() -> i32 {
    let st = state();
    if st.rom_len == 0 {
        return 0;
    }
    power_on(st)
}

#[no_mangle]
pub extern "C" fn emu_set_keys(keys: u32) {
    if let Some(nes) = state().nes.as_mut() {
        nes.set_keys(keys as u8);
    }
}

#[no_mangle]
pub extern "C" fn emu_run_frame() {
    let st = state();
    let Some(nes) = st.nes.as_mut() else { return };
    if st.audio_drained {
        st.audio_out.clear();
        st.audio_drained = false;
    }
    nes.run_frame();
    st.framebuffer.copy_from_slice(&nes.frame()[..]);
    let room = st.audio_out.capacity() - st.audio_out.len();
    let samples = nes.audio();
    let n = samples.len().min(room);
    st.audio_out.extend_from_slice(&samples[..n]);
    samples.clear();
}

#[no_mangle]
pub extern "C" fn emu_framebuffer() -> *const u16 {
    state().framebuffer.as_ptr()
}

#[no_mangle]
pub extern "C" fn emu_audio_buffer() -> *const i16 {
    state().audio_out.as_ptr()
}

#[no_mangle]
pub extern "C" fn emu_audio_samples() -> i32 {
    let st = state();
    if st.audio_drained {
        return 0;
    }
    st.audio_drained = true;
    st.audio_out.len() as i32
}

#[no_mangle]
pub extern "C" fn emu_audio_rate() -> i32 {
    apu::SAMPLE_RATE as i32
}
