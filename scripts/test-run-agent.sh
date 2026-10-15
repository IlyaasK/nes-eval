#!/usr/bin/env bash
# Regression test: a run must survive an unreadable agent log.
#
# Why this exists: the stall watchdog reads `stat -c %Y "$AGENT_LOG"`. That
# stat races the background redirect that creates the file, and an empty
# operand inside `$(( ))` does not merely fail the test — it aborts the
# enclosing `while` loop, so the supervisor fell straight through to
# `finish completed`. On 2026-10-15 that ended a 24h run after 2h45m.
#
# The test starts a run with a deliberately silent agent, deletes its log
# part-way through, and requires the run to still last the requested time.
#
# Usage: scripts/test-run-agent.sh          (needs `docker compose up -d`)
set -euo pipefail
cd "$(dirname "$0")/.."
REPO="$PWD"
RUN=test-log-age
HOURS=0.04          # 144 s
MIN_SECONDS=$(( $(printf '%.0f' "$(awk "BEGIN{print $HOURS*3600*0.8}")") ))

rm -rf "results/$RUN"
echo "== starting a ${HOURS}h run with a silent agent (expect it to last >= ${MIN_SECONDS}s)"
# The supervisor appends the prompt to AGENT_CMD unless the command mentions
# NES_PROMPT, so use a wrapper that ignores trailing arguments and stays
# alive: the stall check only evaluates the log's age while an agent runs.
AGENT_CMD='bash -c "sleep 900" _' AGENT_CONTINUE_CMD='bash -c "sleep 900" _' \
    "${RUN_AGENT:-quickstart/run-agent.sh}" --hours "$HOURS" --name "$RUN" --commit-every 60 --grade-every 1 \
    >"results/$RUN.supervisor.out" 2>&1 &
sup=$!

start=$(date +%s)
log=""
for _ in $(seq 1 40); do
    sleep 2
    log=$(ls "results/$RUN"/agent-*.log 2>/dev/null | head -1)
    [ -n "$log" ] && break
done
if [ -z "$log" ]; then
    echo "FAIL: no agent log appeared — the supervisor did not get as far as starting the agent"
    exit 1
fi
# Delete the log the watchdog stats: this is the race, forced.
echo "== removing $log (forcing the stat to fail)"
rm -f "$log"

wait "$sup" || true          # the supervisor exits when the run finishes/aborts
elapsed=$(( $(date +%s) - start ))

status=$(python3 -c "import json;print(json.load(open('results/$RUN/run.json'))['status'])")
echo "== run ended after ${elapsed}s, status=$status (requested ${HOURS}h = 144s, floor ${MIN_SECONDS}s)"
if [ "$elapsed" -ge "$MIN_SECONDS" ]; then
    echo "PASS: the run lasted its requested duration despite the missing log"
    exit 0
fi
echo "FAIL: the run ended early — a missing agent log killed the supervisor loop"
exit 1
