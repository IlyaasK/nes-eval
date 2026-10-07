//! Black-box reference oracle.
//!
//! The agent calls this to observe reference behaviour on any ROM + input
//! sequence. It runs the same reference wasm the grader uses, through the
//! same 10-function ABI (spec/ABI.md), but the agent never sees the source
//! or the binary: in the task environment this runs in the services
//! sidecar and the agent talks to it over HTTP.
//!
//! Usage:
//!   oracle run <rom> <frames> [--replay <file>] [--dump-frames <dir>] [--dump-final <dir>]
//!                             [--dump-audio <file>] [--from <n>]
//!   oracle info
//!
//! Usage is tracked (frames executed) but not limited.
//!
//! Forked from GBA Eval's harness/oracle (MIT) and adapted to the NES:
//! 256×240 palette-index frames, mono audio at 44 100 Hz.

use std::env;
use std::fs;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use grader::wasm_candidate::WasmCandidate;
use lockstep::media::write_wav;
use lockstep::{InputReplay, Reference, NES_H, NES_PIXELS, NES_W};

/// Default path of the reference wasm inside the services container.
/// quickstart/services/Dockerfile copies reference/nes-ref.wasm here, the
/// exact binary the grader uses. Override with ORACLE_REF_WASM.
const DEFAULT_REF_WASM: &str = "/opt/nes-eval/nes-ref.wasm";

/// Fuel budgets: the grader's defaults (corpus/grader.yaml).
const FUEL_PER_FRAME: u64 = 500_000_000;
const FUEL_LOAD_ROM: u64 = 60_000_000_000;

fn load_reference(rom: &[u8]) -> Box<dyn Reference> {
    let wasm_path = env::var("ORACLE_REF_WASM").unwrap_or_else(|_| DEFAULT_REF_WASM.to_string());
    let wasm_bytes = fs::read(&wasm_path).unwrap_or_else(|e| {
        eprintln!(
            "error: read reference wasm {wasm_path}: {e}\n\
             Set ORACLE_REF_WASM or copy reference/nes-ref.wasm to {DEFAULT_REF_WASM}."
        );
        exit(1);
    });
    let mut reference =
        WasmCandidate::new(&wasm_bytes, "reference".to_string(), FUEL_PER_FRAME, FUEL_LOAD_ROM)
            .unwrap_or_else(|e| {
                eprintln!("error: reference wasm init: {e:?}");
                exit(1);
            });
    reference.load_rom(rom).unwrap_or_else(|e| {
        eprintln!("error: reference rejected the ROM: {e:?}");
        exit(1);
    });
    Box::new(reference)
}

// ─────────────────────────────────────────────────────────────────────────
// Usage tracking (informational, no limit)
// ─────────────────────────────────────────────────────────────────────────

const USAGE_FILE: &str = ".oracle_frames_used";

fn read_usage() -> u64 {
    fs::read_to_string(USAGE_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn record_usage(frames: u64) {
    let used = read_usage();
    let _ = fs::write(USAGE_FILE, format!("{}\n", used + frames));
}

// ─────────────────────────────────────────────────────────────────────────
// Frame output
// ─────────────────────────────────────────────────────────────────────────

/// Binary PPM (P6), RGB, mapped through spec/nes_palette.pal.
fn write_ppm(path: &Path, fb: &[u32; NES_PIXELS]) -> std::io::Result<()> {
    let mut out = Vec::with_capacity(16 + NES_PIXELS * 3);
    write!(out, "P6\n{NES_W} {NES_H}\n255\n")?;
    for &px in fb.iter() {
        out.extend_from_slice(&[px as u8, (px >> 8) as u8, (px >> 16) as u8]);
    }
    fs::write(path, out)
}

/// Raw palette indices, exactly what emu_framebuffer() must hold:
/// 256×240 little-endian u16, row-major, `(emphasis << 6) | colour`.
fn write_idx(path: &Path, idx: &[u16; NES_PIXELS]) -> std::io::Result<()> {
    let mut w = BufWriter::new(fs::File::create(path)?);
    for &v in idx.iter() {
        w.write_all(&v.to_le_bytes())?;
    }
    w.flush()
}

/// Write frame `frame` as `<dir>/frame_NNNNN.ppm` and `.idx`.
fn dump_frame(dir: &Path, frame: u32, reference: &dyn Reference) {
    let stem = dir.join(format!("frame_{frame:05}"));
    let res = write_ppm(&stem.with_extension("ppm"), reference.framebuffer())
        .and_then(|_| write_idx(&stem.with_extension("idx"), reference.index_framebuffer()));
    if let Err(e) = res {
        eprintln!("error: write frame {frame} to {}: {e}", dir.display());
        exit(1);
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Commands
// ─────────────────────────────────────────────────────────────────────────

fn cmd_help() {
    println!(
        "\
NES Eval Oracle — black-box reference emulator

The oracle runs a reference NES emulator on any ROM and returns the exact
framebuffer and audio output. Use it to compare your emulator's output
against the reference and iterate until they match.

COMMANDS:

  oracle help
      Show this help message.

  oracle info
      Print JSON with frames used so far, framebuffer format, resolution
      and audio format.

  oracle run <rom> <frames> [options]
      Power on the reference with <rom> (state per spec/ABI.md) and run it
      for <frames> frames. Prints a JSON summary to stdout.

      Options:
        --replay <file>       Feed a recorded input sequence (format below)
        --dump-frames <dir>   Write every frame as <dir>/frame_NNNNN.ppm (RGB
                              image) and <dir>/frame_NNNNN.idx (raw palette
                              indices, the bytes emu_framebuffer() must hold)
        --dump-final <dir>    Write only the last frame, same two files
        --dump-audio <file>   Write the audio as a mono 16-bit WAV, 44100 Hz
        --from <n>            --dump-frames and --dump-audio cover frames >= n
                              only (the emulator still runs from power-on)

  oracle session ...
      Stateful, ABI-shaped API: start a ROM, set keys, run frames, read the
      framebuffer and audio. Run `oracle session help` for details (task
      container only; provided by its oracle client).

EXAMPLES:

  # What does the reference draw in the first second of a game?
  oracle run dev-roms/<game>.nes 60 --dump-frames /tmp/ref

  # Compare your frame 59 with the reference, byte for byte
  cmp /tmp/ref/frame_00059.idx my_frame_59.idx

  # Run with inputs and capture audio
  oracle run dev-roms/<game>.nes 600 --replay my-inputs.txt --dump-audio /tmp/ref.wav

FRAMES:
  Frame N is the frame completed by the (N+1)-th emu_run_frame() call after
  emu_load_rom(); the keys for frame N are set with emu_set_keys() before
  that call.

PIXEL FORMAT:
  256x240. .idx files: 61440 little-endian u16 values, row-major, each
  (emphasis << 6) | colour, as emu_framebuffer() in spec/ABI.md. PPM files
  are the same frame mapped to RGB through spec/nes_palette.pal.

AUDIO FORMAT:
  Mono signed 16-bit at 44100 Hz, as emu_audio_buffer() in spec/ABI.md
  (about 735 samples per frame).

REPLAY FORMAT:
  Text, one line per change: `<frame> <keys_hex>`. Keys are active-high:
  bit 0=A, 1=B, 2=Select, 3=Start, 4=Up, 5=Down, 6=Left, 7=Right.
  The keys stay held until the next line. `#` starts a comment.

USAGE:
  `oracle info` shows how many frames you have run so far. There is no
  limit; use the oracle as much as you need."
    );
}

fn cmd_info() {
    let info = serde_json::json!({
        "abi_version": 1,
        "frames_used": read_usage(),
        "resolution": { "width": NES_W, "height": NES_H },
        "framebuffer_format": "u16 little-endian per pixel, (emphasis << 6) | colour, row-major (.idx dumps)",
        "image_format": "PPM P6 RGB via spec/nes_palette.pal (.ppm dumps)",
        "audio_format": "i16 mono",
        "audio_rate": 44100,
        "key_bits": "0=A 1=B 2=Select 3=Start 4=Up 5=Down 6=Left 7=Right",
    });
    println!("{}", serde_json::to_string_pretty(&info).unwrap());
}

struct RunArgs {
    rom: PathBuf,
    frames: u32,
    replay: Option<PathBuf>,
    dump_frames: Option<PathBuf>,
    dump_final: Option<PathBuf>,
    dump_audio: Option<PathBuf>,
    from: u32,
}

fn cmd_run(a: RunArgs) {
    let inputs = match &a.replay {
        Some(p) => InputReplay::from_file(p).unwrap_or_else(|e| {
            eprintln!("error: read replay {}: {e}", p.display());
            exit(1);
        }),
        None => InputReplay::new(),
    };
    let rom = fs::read(&a.rom).unwrap_or_else(|e| {
        eprintln!("error: read {}: {e}", a.rom.display());
        exit(1);
    });
    let mut reference = load_reference(&rom);

    for dir in [&a.dump_frames, &a.dump_final].into_iter().flatten() {
        fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("error: create {}: {e}", dir.display());
            exit(1);
        });
    }

    // The ABI leaves the reference at frame 0 after load_rom; kept for
    // parity with the grader, which burns boot frames before comparing.
    for _ in 0..reference.boot_frames() {
        reference.run_frame();
        let _ = reference.drain_audio();
    }

    let audio_rate = reference.audio_rate();
    let mut audio: Vec<i16> = Vec::new();
    for frame in 0..a.frames {
        reference.set_keys(inputs.keys_at(frame));
        reference.run_frame();
        let samples = reference.drain_audio();
        if frame < a.from {
            continue;
        }
        audio.extend_from_slice(&samples);
        if let Some(dir) = &a.dump_frames {
            dump_frame(dir, frame, &*reference);
        }
    }
    if let (Some(dir), Some(last)) = (&a.dump_final, a.frames.checked_sub(1)) {
        dump_frame(dir, last, &*reference);
    }

    if let Some(path) = &a.dump_audio {
        write_wav(path, &audio, audio_rate).unwrap_or_else(|e| {
            eprintln!("error: write wav {}: {e}", path.display());
            exit(1);
        });
        eprintln!("audio: {} mono samples @ {audio_rate} Hz -> {}", audio.len(), path.display());
    }

    record_usage(a.frames as u64);

    let summary = serde_json::json!({
        "frames_executed": a.frames,
        "from_frame": a.from,
        "frames_used_total": read_usage(),
        "audio_rate": audio_rate,
        "audio_samples": audio.len(),
    });
    println!("{}", serde_json::to_string(&summary).unwrap());
}

// ─────────────────────────────────────────────────────────────────────────
// CLI
// ─────────────────────────────────────────────────────────────────────────

fn parse_flag(args: &mut Vec<String>, flag: &str) -> Option<String> {
    let idx = args.iter().position(|a| a == flag)?;
    if idx + 1 >= args.len() {
        eprintln!("error: {flag} needs a value");
        exit(2);
    }
    let val = args.remove(idx + 1);
    args.remove(idx);
    Some(val)
}

fn parse_u32(s: &str, what: &str) -> u32 {
    s.parse().unwrap_or_else(|_| {
        eprintln!("error: invalid {what} '{s}'");
        exit(2);
    })
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        cmd_help();
        exit(2);
    }

    match args[0].as_str() {
        "help" | "--help" | "-h" => cmd_help(),
        "info" => cmd_info(),
        "run" => {
            let mut rest: Vec<String> = args[1..].to_vec();
            let replay = parse_flag(&mut rest, "--replay").map(PathBuf::from);
            let dump_frames = parse_flag(&mut rest, "--dump-frames").map(PathBuf::from);
            let dump_final = parse_flag(&mut rest, "--dump-final").map(PathBuf::from);
            let dump_audio = parse_flag(&mut rest, "--dump-audio").map(PathBuf::from);
            let from = parse_flag(&mut rest, "--from").map_or(0, |s| parse_u32(&s, "--from"));
            if let Some(unknown) = rest.iter().find(|a| a.starts_with("--")) {
                eprintln!("error: unknown option '{unknown}'. See `oracle help`.");
                exit(2);
            }
            let rom: PathBuf = match rest.first() {
                Some(p) => p.into(),
                None => {
                    eprintln!("error: missing <rom> argument");
                    exit(2);
                }
            };
            let frames = rest.get(1).map_or(60, |s| parse_u32(s, "frame count"));
            if rest.len() > 2 {
                eprintln!("error: unexpected argument '{}'", rest[2]);
                exit(2);
            }
            cmd_run(RunArgs { rom, frames, replay, dump_frames, dump_final, dump_audio, from });
        }
        other => {
            eprintln!("error: unknown command '{other}'. Use 'help', 'info' or 'run'.");
            exit(2);
        }
    }
}
