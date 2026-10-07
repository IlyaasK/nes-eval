#!/usr/bin/env bash
# Supervise a timed agent run in the task container.
#
# Runs on the host, next to `docker compose up -d`. For --hours (default 24):
#   - invokes $AGENT_CMD inside the task container (as `agent`, cwd /task)
#     with TASK.md as the prompt; whenever the agent exits before time is
#     up, re-invokes $AGENT_CONTINUE_CMD (default: $AGENT_CMD) with a short
#     "continue working" prompt;
#   - commits /task (`git add -A && git commit`) every --commit-every
#     minutes (default 30);
#   - every --grade-every hours (default 4) and once at the end, builds the
#     candidate in the container with the canonical command, copies
#     nes_emu.wasm out and grades it into results/<run>/<checkpoint>/;
#   - stops the agent at the deadline, makes a final commit and grade, and
#     saves the agent's git history as results/<run>/task.bundle.
#
# Usage:
#   AGENT_CMD='claude -p --dangerously-skip-permissions' ./quickstart/run-agent.sh
#   ./quickstart/run-agent.sh --hours 24 --name my-run --grade-every 6
#   ./quickstart/run-agent.sh --hours 0.05 --commit-every 1 --grade-every 0.02   # quick test
#
# Options:
#   --hours H          run length (fractional OK; default 24)
#   --name RUN         results/<RUN>/ (default run-<UTC timestamp>)
#   --commit-every M   minutes between checkpoint commits (default 30)
#   --grade-every H    hours between graded checkpoints (default 4)
#   --native           grade with target/release/grader instead of the docker image
#
# Environment:
#   AGENT_CMD            command run in the container; the prompt is appended
#                        as its last argument, unless the command mentions
#                        NES_PROMPT or NES_PROMPT_FILE (then it is exported as
#                        $NES_PROMPT and written to $NES_PROMPT_FILE instead).
#                        Also read from quickstart/.env.
#   AGENT_CONTINUE_CMD   command for re-invocations (default: AGENT_CMD), e.g.
#                        one that resumes the previous session.
#   CONTINUE_PROMPT      prompt for re-invocations (default below).
#   RESTART_DELAY        seconds to wait before re-invoking an agent that ran
#                        for under a minute (default 30), so a broken command
#                        doesn't spin.
#
# Output (results/<run>/): run.log (supervisor log), agent-NNN.log (each
# invocation's output), <checkpoint>/ (grade output + nes_emu.wasm +
# build.log + checkpoint.json), run.json (summary), task.bundle.

set -euo pipefail
cd "$(dirname "$0")/.."
REPO="$PWD"
QS="$REPO/quickstart"

HOURS=24
RUN=""
COMMIT_MIN=30
GRADE_HOURS=4
GRADE_MODE=()

while [ $# -gt 0 ]; do
    case "$1" in
        --hours)        HOURS="$2"; shift 2 ;;
        --name)         RUN="$2"; shift 2 ;;
        --commit-every) COMMIT_MIN="$2"; shift 2 ;;
        --grade-every)  GRADE_HOURS="$2"; shift 2 ;;
        --native)       GRADE_MODE=(--native); shift ;;
        -h|--help)      sed -n '2,46p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown argument: $1 (see --help)" >&2; exit 2 ;;
    esac
done

# AGENT_CMD may come from quickstart/.env (compose's env file).
if [ -z "${AGENT_CMD:-}" ] && [ -f "$QS/.env" ]; then
    AGENT_CMD="$(sed -n 's/^AGENT_CMD=//p' "$QS/.env" | tail -n 1)"
fi
[ -n "${AGENT_CMD:-}" ] || { echo "error: set AGENT_CMD (see --help)" >&2; exit 2; }
AGENT_CONTINUE_CMD="${AGENT_CONTINUE_CMD:-$AGENT_CMD}"
RESTART_DELAY="${RESTART_DELAY:-30}"

secs() { awk -v x="$1" -v m="$2" 'BEGIN { v = x * m; if (v < 1) v = 1; printf "%d", v }'; }
TOTAL_S="$(secs "$HOURS" 3600)"
COMMIT_S="$(secs "$COMMIT_MIN" 60)"
GRADE_S="$(secs "$GRADE_HOURS" 3600)"

RUN="${RUN:-run-$(date -u +%Y%m%dT%H%M%SZ)}"
RUN_DIR="$REPO/results/$RUN"
[ -e "$RUN_DIR" ] && { echo "error: $RUN_DIR already exists" >&2; exit 1; }
mkdir -p "$RUN_DIR"
[ -f "$REPO/results/.gitignore" ] || printf '*\n' > "$REPO/results/.gitignore"
LOG="$RUN_DIR/run.log"

dc() { docker compose --project-directory "$QS" "$@"; }
in_task() { dc exec -T task "$@"; }
log() { printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$*" | tee -a "$LOG" >&2; }
now() { date +%s; }
fmt_elapsed() { local s=$(( $1 - START )); printf '%dh%02dm' $((s / 3600)) $(((s % 3600) / 60)); }

dc ps --services --filter status=running 2>/dev/null | grep -qx task \
    || { echo "error: task container not running (cd quickstart && docker compose up -d)" >&2; exit 1; }
dc ps --services --filter status=running 2>/dev/null | grep -qx services \
    || { echo "error: services container not running (cd quickstart && docker compose up -d)" >&2; exit 1; }

START="$(now)"
DEADLINE=$((START + TOTAL_S))
log "run $RUN: ${HOURS}h (until $(date -u -d "@$DEADLINE" +%Y-%m-%dT%H:%M:%SZ)), commit every ${COMMIT_MIN}m, grade every ${GRADE_HOURS}h"
log "AGENT_CMD: $AGENT_CMD"
[ "$AGENT_CONTINUE_CMD" != "$AGENT_CMD" ] && log "AGENT_CONTINUE_CMD: $AGENT_CONTINUE_CMD"

TASK_PROMPT="$(in_task cat /task/TASK.md)"

continue_prompt() {
    local left=$(( (DEADLINE - $(now)) / 60 ))
    if [ -n "${CONTINUE_PROMPT:-}" ]; then
        printf '%s' "$CONTINUE_PROMPT"
    else
        printf 'Your previous session ended, but the task is not over: about %dh%02dm of working time remain. Continue working on the task described in /task/TASK.md (read it again). Your earlier work is in /task; `git log` shows its history. Keep improving the emulator: find what still differs from the oracle, in video and in audio, and fix it.' \
            $((left / 60)) $((left % 60))
    fi
}

# ── Agent invocations ───────────────────────────────────────────────
AGENT_PID=""
AGENT_N=0
AGENT_STARTED=0

start_agent() {   # $1 = command, $2 = prompt
    local cmd="$1" prompt="$2" remaining=$(( DEADLINE - $(now) ))
    case "$cmd" in
        *NES_PROMPT*) ;;
        *) cmd="$cmd \"\$NES_PROMPT\"" ;;
    esac
    AGENT_N=$((AGENT_N + 1))
    AGENT_STARTED="$(now)"
    local alog; alog="$(printf '%s/agent-%03d.log' "$RUN_DIR" "$AGENT_N")"
    log "agent #$AGENT_N start (${remaining}s left) -> $(basename "$alog")"
    # Inside the container: write the prompt file, record our pid, then
    # exec into `timeout`, which kills the agent's whole process group at
    # the deadline (or when stop_agent signals that pid).
    dc exec -T -e NES_PROMPT="$prompt" -e NES_AGENT_CMD="$cmd" task bash -c '
        printf "%s" "$NES_PROMPT" > /tmp/nes-prompt.txt
        export NES_PROMPT_FILE=/tmp/nes-prompt.txt
        echo $$ > /tmp/nes-agent.pid
        cd /task
        exec timeout --kill-after=30 "$1" bash -lc "$NES_AGENT_CMD"
    ' _ "$remaining" < /dev/null > "$alog" 2>&1 &
    AGENT_PID=$!
}

agent_running() { [ -n "$AGENT_PID" ] && kill -0 "$AGENT_PID" 2>/dev/null; }

reap_agent() {
    local rc=0
    wait "$AGENT_PID" || rc=$?
    log "agent #$AGENT_N exited rc=$rc after $(( $(now) - AGENT_STARTED ))s"
    AGENT_PID=""
}

stop_agent() {
    agent_running || return 0
    log "stopping agent #$AGENT_N"
    in_task bash -c 'kill -TERM "$(cat /tmp/nes-agent.pid)" 2>/dev/null' || true
    local i
    for i in $(seq 1 60); do
        agent_running || break
        sleep 1
    done
    if agent_running; then
        in_task bash -c 'kill -KILL "$(cat /tmp/nes-agent.pid)" 2>/dev/null' || true
        kill "$AGENT_PID" 2>/dev/null || true
    fi
    reap_agent
}

# ── Checkpoints ─────────────────────────────────────────────────────
COMMITS=0

commit() {   # $1 = message
    local out rc=0
    out="$(in_task bash -c '
        cd /task || exit 1
        git add -A || exit 1
        if git diff --cached --quiet; then echo "nothing to commit"; exit 0; fi
        git commit -q --no-verify -m "$1" && echo "committed $(git rev-parse --short HEAD)"
    ' _ "$1" 2>&1)" || rc=$?
    log "commit: ${out//$'\n'/ } (rc=$rc)"
    case "$out" in committed*) COMMITS=$((COMMITS + 1)) ;; esac
}

CHECKPOINTS=()
CP_N=0

grade_checkpoint() {   # $1 = checkpoint name
    local cp="$1" cpdir="$RUN_DIR/$1" build_rc=0 grade_rc=0 have_wasm=0 overall="" head
    mkdir -p "$cpdir"
    commit "supervisor checkpoint $cp ($(fmt_elapsed "$(now)"))"
    head="$(in_task git -C /task rev-parse HEAD 2>/dev/null || true)"
    log "checkpoint $cp: building at ${head:0:12}"
    in_task env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS bash -lc \
        'cd /task && cargo build --release --lib --target wasm32-unknown-unknown' \
        > "$cpdir/build.log" 2>&1 || build_rc=$?
    in_task test -f /task/target/wasm32-unknown-unknown/release/nes_emu.wasm && have_wasm=1
    if [ "$have_wasm" -eq 1 ]; then
        [ "$build_rc" -ne 0 ] && log "checkpoint $cp: build failed (rc=$build_rc); grading the last wasm built"
        "$QS/grade.sh" "${GRADE_MODE[@]}" --from-container "$RUN/$cp" > "$cpdir/grade.log" 2>&1 || grade_rc=$?
        if [ -f "$cpdir/summary.json" ]; then
            overall="$(awk -F'[:,]' '/"overall"/ { gsub(/[ }]/, "", $2); print $2; exit }' "$cpdir/summary.json")"
        fi
        log "checkpoint $cp: grade rc=$grade_rc overall=${overall:-n/a}"
    else
        log "checkpoint $cp: no nes_emu.wasm (build rc=$build_rc); not graded"
    fi
    cat > "$cpdir/checkpoint.json" <<EOF
{"checkpoint": "$cp", "elapsed_s": $(( $(now) - START )), "commit": "$head", "build_rc": $build_rc, "wasm": $([ "$have_wasm" -eq 1 ] && echo true || echo false), "grade_rc": $grade_rc, "overall": ${overall:-null}}
EOF
    CHECKPOINTS+=("$cp")
}

finish() {
    local status="$1"
    stop_agent
    commit "supervisor final commit ($status, $(fmt_elapsed "$(now)"))"
    if [ "$status" = "completed" ]; then
        grade_checkpoint final
    fi
    if in_task git -C /task bundle create /tmp/task.bundle --all >/dev/null 2>&1 \
        && dc cp task:/tmp/task.bundle "$RUN_DIR/task.bundle" >/dev/null 2>&1; then
        log "saved git history to $RUN_DIR/task.bundle"
    fi
    {
        printf '{"run": "%s", "status": "%s", "hours": %s, "agent_invocations": %d, "commits": %d, "checkpoints": [' \
            "$RUN" "$status" "$HOURS" "$AGENT_N" "$COMMITS"
        local i sep=""
        for i in "${CHECKPOINTS[@]}"; do
            printf '%s%s' "$sep" "$(cat "$RUN_DIR/$i/checkpoint.json")"; sep=", "
        done
        printf ']}\n'
    } > "$RUN_DIR/run.json"
    log "run $RUN $status: $AGENT_N agent invocation(s), $COMMITS commit(s), ${#CHECKPOINTS[@]} checkpoint(s); see $RUN_DIR"
}

on_signal() {
    trap - INT TERM
    log "interrupted"
    finish interrupted
    exit 130
}
trap on_signal INT TERM

# ── Main loop ───────────────────────────────────────────────────────
NEXT_COMMIT=$((START + COMMIT_S))
NEXT_GRADE=$((START + GRADE_S))

start_agent "$AGENT_CMD" "$TASK_PROMPT"

while [ "$(now)" -lt "$DEADLINE" ]; do
    if ! agent_running; then
        reap_agent
        ran=$(( $(now) - AGENT_STARTED ))
        if [ "$ran" -lt 60 ]; then
            log "agent ran ${ran}s; waiting ${RESTART_DELAY}s before re-invoking"
            sleep "$RESTART_DELAY"
        fi
        [ "$(now)" -lt "$((DEADLINE - 10))" ] || break
        start_agent "$AGENT_CONTINUE_CMD" "$(continue_prompt)"
    fi
    if [ "$(now)" -ge "$NEXT_GRADE" ] && [ "$NEXT_GRADE" -lt "$DEADLINE" ]; then
        CP_N=$((CP_N + 1))
        grade_checkpoint "$(printf 'cp%02d-%s' "$CP_N" "$(fmt_elapsed "$NEXT_GRADE")")"
        while [ "$NEXT_GRADE" -le "$(now)" ]; do NEXT_GRADE=$((NEXT_GRADE + GRADE_S)); done
        NEXT_COMMIT=$(( $(now) + COMMIT_S ))
    elif [ "$(now)" -ge "$NEXT_COMMIT" ]; then
        commit "supervisor checkpoint $(fmt_elapsed "$(now)")"
        NEXT_COMMIT=$(( $(now) + COMMIT_S ))
    fi
    sleep 5
done

trap - INT TERM
finish completed
