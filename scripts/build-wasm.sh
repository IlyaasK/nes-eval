#!/usr/bin/env bash
# Build the wasm modules the repo ships, all from the reference/ crate:
#   reference/nes-ref.wasm                    RustyNES behind spec/ABI.md (the grading reference)
#   candidates/broken/broken.wasm             all three injected bugs
#   candidates/broken/bug-scanline.wasm       video one scanline late
#   candidates/broken/bug-ab-swap.wasm        A and B swapped
#   candidates/broken/bug-no-tri-noise.wasm   triangle and noise channels missing
set -euo pipefail
cd "$(dirname "$0")/.."

out=target/wasm32-unknown-unknown/release/nes_reference.wasm
build() {
  cargo build --release -p nes-reference --lib --target wasm32-unknown-unknown "$@"
}

build
cp "$out" reference/nes-ref.wasm

mkdir -p candidates/broken
build --features broken
cp "$out" candidates/broken/broken.wasm
for bug in bug-scanline bug-ab-swap bug-no-tri-noise; do
  build --features "$bug"
  cp "$out" "candidates/broken/$bug.wasm"
done

sha256sum reference/nes-ref.wasm candidates/broken/*.wasm
