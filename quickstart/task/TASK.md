# Task: Implement a Nintendo Entertainment System Emulator

Build an NES emulator that matches a reference emulator's output — both video and audio — across a suite of real games and test ROMs. The target is a fully cycle-accurate implementation of the NTSC NES (the 2A03 CPU and APU, the 2C02 PPU, and cartridge mappers 0 (NROM), 1 (MMC1), 2 (UxROM), 3 (CNROM) and 4 (MMC3)) that plays arbitrary games end-to-end fully correctly, including handling user input (button presses, directional pad, held inputs) so that games progress past title screens, through menus, and into actual gameplay. Rendering a correct idle frame is not enough; the emulator must produce correct output for any input sequence. Audio and video are both first-class outputs, you should work on both in parallel as both must be accurate and are extremely difficult to implement correctly. The power-on state is not random: it is specified exactly in `spec/ABI.md`, and your emulator must reproduce it. You must continue working towards this goal indefinitely.

A truly correct NES emulator is not achievable — there is always more to fix. If you believe you are done, you are mistaken. Keep working.

## What's in the container

| Path | Description |
|------|-------------|
| `spec/ABI.md` | The interface your wasm must export, including the exact power-on state |
| `spec/nes_palette.pal` | The palette used to turn your framebuffer's palette indices into colours |
| `spec/docs/` | NES hardware reference (start at `spec/docs/index.md`) |
| `dev-roms/` | Example NES ROMs |

Do not attempt to access the internet — the test corpus and grading happen offline against the bundled reference, and pulling code from external sources is not what's being evaluated.

Toolchains available: Rust 1.96 with `wasm32-unknown-unknown`, wasmtime, clang with `wasm-ld`, cmake, cc65 (`ca65`, `ld65`, `da65`) for writing or disassembling 6502 code, python3 with numpy and scipy.

## Tools

**`oracle`** — an NES emulator. You do not have access to the source code. Run `oracle help` to see its interface.

## What you produce

A `.wasm` file implementing the functions in `spec/ABI.md`, built for `wasm32-unknown-unknown` as a cdylib.

## Project layout

Your project must be a Rust cargo project at the **root** of `/task/`:

```
/task/
├── Cargo.toml      # [package] name = "nes_emu"
└── src/
    └── lib.rs      # [lib] crate-type = ["cdylib", "rlib"]
```

Required:
- Package name is `nes_emu` (underscore, not dash). The artifact will be `target/wasm32-unknown-unknown/release/nes_emu.wasm`.
- Cargo.toml lives at `/task/Cargo.toml`, **not** in a subdirectory like `/task/nes-emu/Cargo.toml`.
- Do not change the package name or move Cargo.toml once work is underway — external tooling expects a stable artifact path at `target/wasm32-unknown-unknown/release/nes_emu.wasm`.
- Your cdylib exports the functions in `spec/ABI.md`.

The build is always `cargo build --release --lib --target wasm32-unknown-unknown` run from `/task/` with no `.cargo/config.toml` and no custom `RUSTFLAGS`. Write code that doesn't require those.

## Memory

Wasm linear memory grows on demand at runtime via `memory.grow`. You do not need to statically reserve tens of MB of buffers; allocate with `Vec`/`Box` and let the allocator grow the heap as it needs. Large `static mut` arrays that inflate the initial linear-memory size beyond tens of MB can fail to link.
