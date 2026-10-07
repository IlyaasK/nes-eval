#!/usr/bin/env bash
# Grade a candidate .wasm against the full corpus. Prints the score.
#
# Default path uses docker (only prerequisite: docker). --native runs
# target/release/grader on the host (built with cargo if missing).
#
# Usage:
#   ./grade.sh path/to/nes_emu.wasm                       # docker, auto-named
#   ./grade.sh path/to/nes_emu.wasm my-run                # docker, named run
#   ./grade.sh --reference my-ref.wasm cand.wasm my-run   # use a custom reference wasm
#   ./grade.sh --from-container my-run                    # grab wasm from running task container, grade it
#   ./grade.sh --native path/to/nes_emu.wasm              # host-side grader
#   ./grade.sh --rebuild ...                              # rebuild the nes-eval-grader image first
#
# --reference defaults to reference/nes-ref.wasm (RustyNES). Pass any wasm
# implementing spec/ABI.md to grade against a different reference (e.g.
# grade two candidates against each other).
#
# Output: ./results/<name>/ at repo root (<name> may contain slashes).
# summary.json has the overall score; grade.sh echoes it on success.

set -euo pipefail
cd "$(dirname "$0")/.."
REPO="$PWD"

MODE="docker"
FROM_CONTAINER=0
REBUILD=0
WASM=""
NAME=""
REFERENCE=""
IMAGE="nes-eval-grader"
WASM_IN_CONTAINER="/task/target/wasm32-unknown-unknown/release/nes_emu.wasm"

while [ $# -gt 0 ]; do
    case "$1" in
        --native)         MODE="native"; shift ;;
        --docker)         MODE="docker"; shift ;;
        --from-container) FROM_CONTAINER=1; shift ;;
        --rebuild)        REBUILD=1; shift ;;
        --reference)      REFERENCE="$2"; shift 2 ;;
        -h|--help)
            sed -n '2,20p' "$0" | sed 's/^# \{0,1\}//'
            exit 0 ;;
        --*) echo "unknown flag: $1" >&2; exit 2 ;;
        *)
            if [ -z "$WASM" ] && [ "$FROM_CONTAINER" -eq 0 ]; then
                WASM="$1"
            elif [ -z "$NAME" ]; then
                NAME="$1"
            else
                echo "unexpected arg: $1" >&2; exit 2
            fi
            shift ;;
    esac
done

mkdir -p "$REPO/results"
[ -f "$REPO/results/.gitignore" ] || printf '*\n' > "$REPO/results/.gitignore"

# ── Pull wasm from the running task container if requested ──────────
if [ "$FROM_CONTAINER" -eq 1 ]; then
    [ -n "$WASM" ] && { echo "error: --from-container and a wasm path are mutually exclusive" >&2; exit 2; }
    NAME="${NAME:-nes_emu-$(date -u +%Y%m%dT%H%M%SZ)}"
    mkdir -p "$REPO/results/$NAME"
    WASM="$REPO/results/$NAME/nes_emu.wasm"
    echo "→ extracting $WASM_IN_CONTAINER from the task container ..."
    if ! docker compose --project-directory "$REPO/quickstart" cp \
            "task:$WASM_IN_CONTAINER" "$WASM" >/dev/null 2>&1; then
        echo "error: could not copy $WASM_IN_CONTAINER" >&2
        echo "       is the task container up? (cd quickstart && docker compose up -d)" >&2
        echo "       has the candidate been built inside it?" >&2
        exit 1
    fi
fi

[ -z "$WASM" ] && { echo "usage: $0 [--native|--docker|--from-container] [--reference <ref.wasm>] <wasm> [name]" >&2; exit 2; }
[ -f "$WASM" ] || { echo "error: $WASM not found" >&2; exit 1; }
[ -n "$REFERENCE" ] && [ ! -f "$REFERENCE" ] && { echo "error: reference wasm $REFERENCE not found" >&2; exit 1; }
[ -f reference/nes-ref.wasm ] || { echo "error: reference/nes-ref.wasm missing (scripts/build-wasm.sh builds it)" >&2; exit 1; }

NAME="${NAME:-$(basename "${WASM%.wasm}")-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT="$REPO/results/$NAME"
mkdir -p "$OUT" corpus/reference-cache

if [ -z "$(ls -A corpus/reference-cache 2>/dev/null)" ]; then
    echo "note: corpus/reference-cache/ is empty; the grader will build it on this run (~30 s+)." >&2
fi

# ── Dispatch ────────────────────────────────────────────────────────
if [ "$MODE" = "docker" ]; then
    command -v docker >/dev/null 2>&1 || { echo "error: docker not found. install docker or use --native." >&2; exit 1; }
    docker info >/dev/null 2>&1 || { echo "error: docker daemon not reachable." >&2; exit 1; }

    if [ "$REBUILD" -eq 1 ] || ! docker image inspect "$IMAGE" >/dev/null 2>&1; then
        echo "→ building $IMAGE image (a few minutes cold)…"
        docker build -f quickstart/grader/Dockerfile -t "$IMAGE" .
    fi

    # Mount the wasm by its containing directory so the grader sees a
    # stable in-container path wherever it lives on the host. Repo
    # read-only, except the reference cache (rewritten if stale) and the
    # results dir. Runs as the host user so outputs aren't root-owned.
    WASM_DIR="$(cd "$(dirname "$WASM")" && pwd)"
    WASM_NAME="$(basename "$WASM")"
    REF_ARGS=()
    REF_MOUNTS=()
    if [ -n "$REFERENCE" ]; then
        REF_DIR="$(cd "$(dirname "$REFERENCE")" && pwd)"
        REF_NAME="$(basename "$REFERENCE")"
        REF_MOUNTS=(-v "$REF_DIR":/ref:ro)
        REF_ARGS=(--reference /ref/"$REF_NAME")
        echo "── Grading $WASM (docker) ref=$REFERENCE → $OUT ──"
    else
        echo "── Grading $WASM (docker) → $OUT ──"
    fi
    docker run --rm --user "$(id -u):$(id -g)" \
        -v "$REPO":/repo:ro \
        -v "$REPO/corpus/reference-cache":/repo/corpus/reference-cache \
        -v "$WASM_DIR":/wasm:ro \
        "${REF_MOUNTS[@]}" \
        -v "$OUT":/out \
        "$IMAGE" "${REF_ARGS[@]}" /wasm/"$WASM_NAME" corpus/ /out

else
    if [ "$REBUILD" -eq 1 ] || [ ! -x target/release/grader ]; then
        command -v cargo >/dev/null 2>&1 || { echo "error: cargo not found (needed to build target/release/grader)." >&2; exit 1; }
        cargo build --release -p grader
    fi
    WASM_ABS="$(cd "$(dirname "$WASM")" && pwd)/$(basename "$WASM")"
    REF_FLAG=()
    if [ -n "$REFERENCE" ]; then
        REF_ABS="$(cd "$(dirname "$REFERENCE")" && pwd)/$(basename "$REFERENCE")"
        REF_FLAG=(--reference "$REF_ABS")
        echo "── Grading $WASM (native) ref=$REFERENCE → $OUT ──"
    else
        echo "── Grading $WASM (native) → $OUT ──"
    fi
    target/release/grader "${REF_FLAG[@]}" "$WASM_ABS" corpus/ "$OUT"
fi

# ── Summary ─────────────────────────────────────────────────────────
SUMMARY="$OUT/summary.json"
if [ -f "$SUMMARY" ]; then
    echo
    echo "── summary ──"
    if command -v jq >/dev/null 2>&1; then
        jq . "$SUMMARY"
        OVERALL=$(jq -r '.overall // empty' "$SUMMARY" 2>/dev/null || true)
        [ -n "$OVERALL" ] && echo && echo "overall: $OVERALL"
    else
        cat "$SUMMARY"
    fi
fi
