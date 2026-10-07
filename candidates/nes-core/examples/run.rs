//! Development runner: plays a ROM headless and reports what it shows.
//!
//! cargo run --release --example run -- <rom.nes> <frames> [replay.txt] [out.ppm]
//!
//! Prints blargg-style test status from $6000 (status byte, then the
//! "DE B0 61" signature and text from $6004) and optionally writes the
//! final frame as a PPM using spec/nes_palette.pal.

use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: run <rom.nes> <frames> [replay.txt|-] [out.ppm]");
        std::process::exit(2);
    }
    let rom = fs::read(&args[1]).expect("read rom");
    let frames: u32 = args[2].parse().expect("frames");
    let replay = parse_replay(args.get(3).filter(|s| s.as_str() != "-"));
    let mut nes = nes_core::Nes::new(&rom).expect("unsupported rom");

    let mut keys = 0u8;
    let mut samples = 0usize;
    let mut peak = 0i32;
    for f in 0..frames {
        for &(at, k) in &replay {
            if at == f {
                keys = k;
            }
        }
        nes.set_keys(keys);
        nes.run_frame();
        let a = nes.audio();
        samples += a.len();
        peak = peak.max(a.iter().map(|&s| (s as i32).abs()).max().unwrap_or(0));
        a.clear();
    }

    let sig = [nes.peek(0x6001), nes.peek(0x6002), nes.peek(0x6003)];
    if sig == [0xDE, 0xB0, 0x61] {
        let mut text = String::new();
        let mut a = 0x6004u16;
        while a < 0x7000 && nes.peek(a) != 0 {
            text.push(nes.peek(a) as char);
            a += 1;
        }
        println!("status={:#04x}\n{}", nes.peek(0x6000), text.trim_end());
    } else {
        println!("no blargg signature");
    }
    println!("audio: {samples} samples, peak {peak}");

    if let Some(out) = args.get(4) {
        let pal = fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../spec/nes_palette.pal"))
            .expect("palette");
        let mut ppm = format!("P6\n256 240\n255\n").into_bytes();
        for &idx in nes.frame().iter() {
            let i = (idx & 0x1FF) as usize * 3;
            ppm.extend_from_slice(&pal[i..i + 3]);
        }
        fs::write(out, ppm).expect("write ppm");
    }
}

fn parse_replay(path: Option<&String>) -> Vec<(u32, u8)> {
    let Some(path) = path else { return Vec::new() };
    fs::read_to_string(path)
        .expect("read replay")
        .lines()
        .filter(|l| !l.trim_start().starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let mut it = l.split_whitespace();
            let f = it.next().unwrap().parse().unwrap();
            let k = u8::from_str_radix(it.next().unwrap(), 16).unwrap();
            (f, k)
        })
        .collect()
}
