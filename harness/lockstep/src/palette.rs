//! Palette-index → RGB mapping shared by every emulator the grader drives.
//!
//! The ABI framebuffer holds `(emphasis << 6) | colour` per pixel
//! (spec/ABI.md). The grader maps it through `spec/nes_palette.pal`, which
//! is RustyNES's default 2C02 table, so reference and candidate pixels land
//! in the same RGB space no matter which display palette either emulator
//! would have used.

const PAL_BYTES: &[u8; 512 * 3] = include_bytes!("../../../spec/nes_palette.pal");

/// 512-entry `0xFFBBGGRR` lookup built from `spec/nes_palette.pal`.
pub static INDEX_TO_RGBA: [u32; 512] = {
    let mut lut = [0u32; 512];
    let mut i = 0;
    while i < 512 {
        let r = PAL_BYTES[i * 3] as u32;
        let g = PAL_BYTES[i * 3 + 1] as u32;
        let b = PAL_BYTES[i * 3 + 2] as u32;
        lut[i] = 0xFF00_0000 | (b << 16) | (g << 8) | r;
        i += 1;
    }
    lut
};

/// Map one framebuffer value to `0xFFBBGGRR`. Bits above the 9-bit index
/// are ignored, as the ABI says.
#[inline]
pub fn index_to_rgba(idx: u16) -> u32 {
    INDEX_TO_RGBA[(idx & 0x1FF) as usize]
}
