//! RustyNES behind the NES Eval wasm ABI (`spec/ABI.md`).
//!
//! Built as `reference/nes-ref.wasm`, this is the grading reference. The
//! `bug-*` cargo features inject one deliberate emulator bug each, one per
//! score section, to show the grader penalises real faults
//! (`scripts/build-wasm.sh` builds `candidates/broken*`):
//!
//! - `bug-scanline`: video is one scanline late (row `y` shows scanline `y - 1`);
//! - `bug-ab-swap`: A and B are swapped;
//! - `bug-no-tri-noise`: the triangle and noise channels are missing.
//!
//! The `broken` feature enables all three.

use rustynes_core::{Buttons, Nes};

const ROM_CAPACITY: usize = 8 * 1024 * 1024;
const PIXELS: usize = 256 * 240;
const AUDIO_RATE: u32 = 44_100;

#[cfg(feature = "bug-no-tri-noise")]
const BROKEN_CHANNEL_MASK: u8 = 0x33; // pulse 1, pulse 2, DMC, expansion; no triangle/noise

struct State {
    rom: Vec<u8>,
    rom_len: usize,
    nes: Option<Nes>,
    keys: u8,
    framebuffer: Vec<u16>,
    pending_audio: Vec<i16>,
    drained_audio: Vec<i16>,
}

static mut STATE: Option<State> = None;

/// The wasm module is single-threaded and the ABI functions are never
/// re-entered, so handing out one `&mut` at a time is sound.
fn state() -> Option<&'static mut State> {
    // SAFETY: see above; no other reference to STATE is live.
    unsafe { (*(&raw mut STATE)).as_mut() }
}

fn power_on(st: &mut State) -> bool {
    let Ok(mut nes) = Nes::from_rom_with_sample_rate(&st.rom[..st.rom_len], AUDIO_RATE) else {
        st.nes = None;
        return false;
    };
    #[cfg(feature = "bug-no-tri-noise")]
    nes.set_apu_channel_mask(BROKEN_CHANNEL_MASK);
    #[cfg(not(feature = "bug-no-tri-noise"))]
    let _ = &mut nes;
    st.nes = Some(nes);
    st.keys = 0;
    st.framebuffer.fill(0);
    st.pending_audio.clear();
    st.drained_audio.clear();
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_init() -> i32 {
    let st = State {
        rom: vec![0; ROM_CAPACITY],
        rom_len: 0,
        nes: None,
        keys: 0,
        framebuffer: vec![0; PIXELS],
        pending_audio: Vec::with_capacity(4096),
        drained_audio: Vec::with_capacity(4096),
    };
    // SAFETY: single-threaded; no reference to STATE is live here.
    unsafe { *(&raw mut STATE) = Some(st) };
    1
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_rom_buffer() -> *mut u8 {
    state().map_or(core::ptr::null_mut(), |st| st.rom.as_mut_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_load_rom(len: i32) -> i32 {
    let Some(st) = state() else { return 0 };
    let Ok(len) = usize::try_from(len) else { return 0 };
    if len > ROM_CAPACITY {
        return 0;
    }
    st.rom_len = len;
    i32::from(power_on(st))
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_reset() -> i32 {
    let Some(st) = state() else { return 0 };
    if st.rom_len == 0 {
        return 0;
    }
    i32::from(power_on(st))
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_set_keys(keys: u32) {
    let Some(st) = state() else { return };
    #[allow(clippy::cast_possible_truncation)] // only the 8 controller bits exist
    let keys = keys as u8;
    #[cfg(feature = "bug-ab-swap")]
    let keys = (keys & !0b11) | ((keys & 1) << 1) | ((keys >> 1) & 1);
    st.keys = keys;
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_run_frame() {
    let Some(st) = state() else { return };
    let Some(nes) = st.nes.as_mut() else { return };
    nes.set_buttons(0, Buttons::from_bits_retain(st.keys));
    nes.run_frame();

    let src = nes.index_framebuffer();
    #[cfg(not(feature = "bug-scanline"))]
    for (dst, &px) in st.framebuffer.iter_mut().zip(src) {
        *dst = px & 0x1FF;
    }
    #[cfg(feature = "bug-scanline")]
    {
        st.framebuffer[..256].copy_from_slice(&src[..256]);
        for (dst, &px) in st.framebuffer[256..].iter_mut().zip(&src[..PIXELS - 256]) {
            *dst = px & 0x1FF;
        }
    }

    for s in nes.drain_audio() {
        #[allow(clippy::cast_possible_truncation)] // clamped to i16 range first
        st.pending_audio.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_framebuffer() -> *const u16 {
    state().map_or(core::ptr::null(), |st| st.framebuffer.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_audio_samples() -> i32 {
    let Some(st) = state() else { return 0 };
    core::mem::swap(&mut st.pending_audio, &mut st.drained_audio);
    st.pending_audio.clear();
    i32::try_from(st.drained_audio.len()).unwrap_or(i32::MAX)
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_audio_buffer() -> *const i16 {
    state().map_or(core::ptr::null(), |st| st.drained_audio.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn emu_audio_rate() -> i32 {
    AUDIO_RATE as i32
}
