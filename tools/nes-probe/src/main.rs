//! Corpus-authoring tool. Runs the reference core (RustyNES, natively) on a
//! ROM with an optional replay and reports what the screen did, so testcase
//! frame counts and gameplay replays can be written without a GUI.
//!
//! Usage: `nes-probe <rom.nes> <frames> [replay.txt] [--png-every N --out DIR]`
//!
//! Prints `change <frame>` for every frame whose picture differs from the
//! previous one, then `last_change <frame>`. With `--png-every N`, writes
//! `DIR/f<frame>.png` every N frames and for the final frame, mapped through
//! the grader's palette so the images match what the grader compares.
//!
//! Replays use the corpus format (`<frame> <keys_hex>`, see
//! harness/lockstep/src/replay.rs).

use std::path::PathBuf;
use std::process::ExitCode;

use lockstep::palette::index_to_rgba;
use lockstep::{InputReplay, NES_PIXELS};
use rustynes_core::{Buttons, Nes};

struct Args {
    rom: PathBuf,
    frames: u32,
    replay: Option<PathBuf>,
    png_every: Option<u32>,
    out: Option<PathBuf>,
}

fn parse(mut it: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut positional = Vec::new();
    let (mut png_every, mut out) = (None, None);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--png-every" => {
                let n = it.next().ok_or("--png-every needs N")?;
                png_every = Some(n.parse::<u32>().map_err(|e| format!("--png-every: {e}"))?.max(1));
            }
            "--out" => out = Some(PathBuf::from(it.next().ok_or("--out needs DIR")?)),
            _ if a.starts_with("--") => return Err(format!("unknown flag {a}")),
            _ => positional.push(a),
        }
    }
    let (rom, frames, replay) = match positional.as_slice() {
        [r, f] => (r, f, None),
        [r, f, p] => (r, f, Some(PathBuf::from(p))),
        _ => return Err("usage: nes-probe <rom.nes> <frames> [replay.txt] [--png-every N --out DIR]".into()),
    };
    if png_every.is_some() != out.is_some() {
        return Err("--png-every and --out go together".into());
    }
    let frames = frames.parse().map_err(|e| format!("bad frame count {frames:?}: {e}"))?;
    Ok(Args { rom: PathBuf::from(rom), frames, replay, png_every, out })
}

fn run(args: Args) -> Result<(), String> {
    let replay = match &args.replay {
        Some(p) => InputReplay::from_file(p).map_err(|e| format!("replay {}: {e}", p.display()))?,
        None => InputReplay::new(),
    };
    let rom = std::fs::read(&args.rom).map_err(|e| format!("read {}: {e}", args.rom.display()))?;
    let mut nes = Nes::from_rom(&rom).map_err(|e| format!("load {}: {e}", args.rom.display()))?;
    if let Some(dir) = &args.out {
        std::fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }

    let mut prev: Vec<u16> = Vec::new();
    let mut last_change = 0u32;
    let mut rgba = Box::new([0u32; NES_PIXELS]);
    for frame in 0..args.frames {
        #[allow(clippy::cast_possible_truncation)] // controller 1 is 8 bits
        nes.set_buttons(0, Buttons::from_bits_retain(replay.keys_at(frame) as u8));
        nes.run_frame();
        let idx = nes.index_framebuffer();
        if idx != prev.as_slice() {
            println!("change {frame}");
            last_change = frame;
            prev.clear();
            prev.extend_from_slice(idx);
        }
        if let (Some(every), Some(dir)) = (args.png_every, &args.out) {
            if frame % every == 0 || frame + 1 == args.frames {
                for (dst, &px) in rgba.iter_mut().zip(idx) {
                    *dst = index_to_rgba(px);
                }
                let path = dir.join(format!("f{frame:05}.png"));
                lockstep::media::write_png(&path, &rgba).map_err(|e| e.to_string())?;
            }
        }
    }
    println!("last_change {last_change}");
    Ok(())
}

fn main() -> ExitCode {
    match parse(std::env::args().skip(1)).and_then(run) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            ExitCode::from(2)
        }
    }
}
