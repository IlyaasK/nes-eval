//! Writes `spec/nes_palette.pal`: the 512-entry RGB table the grader uses to
//! turn `(emphasis << 6) | colour` framebuffer values into pixels. It is
//! RustyNES's default 2C02 lookup, byte for byte.
//!
//! Usage: `gen_palette <out.pal>`

use rustynes_core::rustynes_ppu::{PpuPalette, build_rgba_lut};

fn main() -> std::io::Result<()> {
    let out = std::env::args().nth(1).unwrap_or_else(|| "spec/nes_palette.pal".into());
    let lut = build_rgba_lut(PpuPalette::Composite2C02);
    let bytes: Vec<u8> = lut.iter().flat_map(|px| [px[0], px[1], px[2]]).collect();
    std::fs::write(&out, &bytes)?;
    eprintln!("wrote {} entries to {out}", lut.len());
    Ok(())
}
