#!/usr/bin/env bash
# Determinism gate (CI). Fails unless:
#   1. regenerating every reference cache from scratch produces byte-identical
#      .refcache files, and
#   2. grading the same candidate twice produces byte-identical results
#      (per-testcase JSON, summary.json, PNGs, WAVs).
#
# Usage: scripts/determinism-gate.sh [candidate.wasm]   (default: candidates/broken/broken.wasm)
set -euo pipefail
cd "$(dirname "$0")/.."

candidate=${1:-candidates/broken/broken.wasm}
grader=target/release/grader
cargo build --release -p grader

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

echo "== 1/2 reference cache: generate twice from scratch"
for run in a b; do
  rm -rf corpus/reference-cache
  "$grader" --precompute corpus/ >/dev/null 2>"$work/precompute-$run.log" \
    || { cat "$work/precompute-$run.log"; exit 1; }
  cp -r corpus/reference-cache "$work/cache-$run"
done
diff -r "$work/cache-a" "$work/cache-b"
echo "   $(ls "$work/cache-a" | wc -l) cache files byte-identical"

echo "== 2/2 candidate: grade $candidate twice"
for run in a b; do
  "$grader" "$candidate" corpus/ "$work/grade-$run" >/dev/null 2>"$work/grade-$run.log" \
    || { cat "$work/grade-$run.log"; exit 1; }
done
diff -r "$work/grade-a" "$work/grade-b"
echo "   $(ls "$work/grade-a" | wc -l) result files byte-identical"

echo "determinism gate: PASS"
