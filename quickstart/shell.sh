#!/usr/bin/env bash
# Open a shell (or run a one-shot command) inside the task container as
# uid 1000, the same user + environment the agent works in.
#
# Usage:
#   ./shell.sh                        # interactive shell
#   ./shell.sh cargo build --release --lib --target wasm32-unknown-unknown
#   ./shell.sh oracle info
#   ./shell.sh bash -lc 'oracle run dev-roms/<game>.nes 60 --dump-frames /task/out'
#
# Requires: `docker compose up -d` from this directory first.

set -euo pipefail
cd "$(dirname "$0")"

SERVICE="task"

if ! docker compose ps --services --filter "status=running" 2>/dev/null | grep -qx "$SERVICE"; then
    echo "error: '$SERVICE' service is not running." >&2
    echo "bring it up first:" >&2
    echo "  cd $(pwd) && docker compose up -d" >&2
    exit 1
fi

# No TTY when stdin isn't one (scripts, pipes, CI).
TTY_FLAG=()
[ -t 0 ] && [ -t 1 ] || TTY_FLAG=(-T)

if [ $# -eq 0 ]; then
    exec docker compose exec "${TTY_FLAG[@]}" "$SERVICE" bash -l
fi

exec docker compose exec "${TTY_FLAG[@]}" "$SERVICE" "$@"
