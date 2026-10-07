#!/usr/bin/env bash
# Preflight check — verifies the repo + tooling are set up to grade and to
# build the agent environment. Fast (seconds). Does NOT run a grade (that
# is `./quickstart/grade.sh <wasm>`).
#
# Usage: ./quickstart/smoke-test.sh [docker|native]
# Exits 0 on pass, non-zero on the first failure.

set -euo pipefail
cd "$(dirname "$0")/.."

pass() { printf "  \033[32m✓\033[0m %s\n" "$1"; }
warn() { printf "  \033[33m!\033[0m %s\n     %s\n" "$1" "$2"; }
fail() { printf "  \033[31m✗\033[0m %s\n     %s\n" "$1" "$2"; exit 1; }

MODE="${1:-docker}"

echo "── nes-eval smoke test (mode=$MODE) ──"

# ── Repo layout ─────────────────────────────────────────────────────
[ -f Cargo.toml ]            || fail "repo root Cargo.toml"   "are you in the nes-eval clone?"
[ -f corpus/testcases.json ] || fail "corpus/testcases.json"  "corpus/ missing"
[ -f corpus/grader.yaml ]    || fail "corpus/grader.yaml"     "grader config missing"
[ -f spec/ABI.md ]           || fail "spec/ABI.md"            "spec missing"
pass "repo layout"

[ -f reference/nes-ref.wasm ] || fail "reference/nes-ref.wasm" \
    "reference wasm missing; build it with scripts/build-wasm.sh (needs third_party/RustyNES, see README.md)"
pass "reference wasm present"

# ── Reference cache (gitignored; rebuilt by the grader when absent) ──
if ls corpus/reference-cache/*.refcache >/dev/null 2>&1; then
    pass "reference cache present ($(ls corpus/reference-cache/*.refcache | wc -l) testcases)"
else
    warn "corpus/reference-cache is empty" \
         "the first grade builds it (~30 s); or run: target/release/grader --precompute corpus/"
fi

# ── Agent environment inputs ────────────────────────────────────────
[ -f quickstart/task/TASK.md ] || fail "quickstart/task/TASK.md" "task prompt missing"
ls corpus/roms/dev/*.nes >/dev/null 2>&1 || fail "corpus/roms/dev/*.nes" "dev ROMs missing"
pass "dev ROMs present ($(ls corpus/roms/dev/*.nes | wc -l) games)"
[ -f spec/docs/SOURCES.md ] || fail "spec/docs/SOURCES.md" "hardware docs missing"
pass "hardware docs present"

# ── Baseline candidate (any wasm to grade) ──────────────────────────
if [ -f candidates/broken/broken.wasm ]; then
    pass "baseline candidate present (candidates/broken/broken.wasm)"
else
    warn "candidates/broken/broken.wasm missing" "scripts/build-wasm.sh builds it"
fi

# ── Mode-specific: toolchain ────────────────────────────────────────
if [ "$MODE" = "docker" ]; then
    command -v docker >/dev/null 2>&1 || fail "docker on PATH" "install docker"
    docker info >/dev/null 2>&1        || fail "docker daemon reachable" "start docker daemon"
    docker compose version >/dev/null 2>&1 || fail "docker compose" "install the compose plugin"
    pass "docker + compose reachable"

    if docker image inspect nes-eval-grader >/dev/null 2>&1; then
        pass "nes-eval-grader image built"
        # The grader prints usage and exits 2 with no args. Capture output
        # first so pipefail + the non-zero exit don't trip the check.
        grader_out="$(docker run --rm nes-eval-grader 2>&1 || true)"
        echo "$grader_out" | grep -q 'usage:' \
            || fail "grader binary functional in image" "image is built but grader didn't print its usage line"
        pass "grader binary functional"
    else
        warn "nes-eval-grader image not yet built" \
             "the first grade.sh run builds it (a few minutes cold)"
    fi
elif [ "$MODE" = "native" ]; then
    command -v cargo >/dev/null 2>&1 || fail "cargo on PATH" "install the Rust toolchain (rust-toolchain.toml pins 1.96.0)"
    pass "cargo available"
    if [ -x target/release/grader ]; then
        pass "target/release/grader built"
    else
        warn "target/release/grader not built" "grade.sh --native builds it (cargo build --release -p grader)"
    fi
else
    fail "mode=$MODE" "expected 'docker' or 'native'"
fi

echo
echo "✓ all checks passed. next step:"
echo "    ./quickstart/grade.sh candidates/broken/broken.wasm baseline"
