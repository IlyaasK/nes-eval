#!/bin/bash
# Build nes-core as an ABI-conformant wasm (spec/ABI.md).
#
# Pure Rust, wasm32-unknown-unknown: the module exports `memory` and the
# 10 ABI functions and imports nothing.
#
# Output: nes_core.wasm in this directory.

set -euo pipefail
cd "$(dirname "$0")"

cargo build --release --target wasm32-unknown-unknown --lib

WASM="target/wasm32-unknown-unknown/release/nes_core.wasm"
if [ ! -f "$WASM" ]; then
    echo "error: cargo didn't produce $WASM" >&2
    exit 1
fi

cp "$WASM" nes_core.wasm
echo "wasm built: nes_core.wasm ($(stat -c %s nes_core.wasm) bytes)"
echo "grade with: target/release/grader candidates/nes-core/nes_core.wasm corpus/ out/nes-core"
