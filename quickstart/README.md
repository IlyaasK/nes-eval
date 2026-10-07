# NES Eval Quickstart

This directory lets you:

1. **Run an agent in the benchmark's environment**: `docker compose up -d --build`,
   then `./shell.sh` or the timed supervisor `./run-agent.sh`.
2. **Grade a wasm** you or an agent produced: `./grade.sh path/to/nes_emu.wasm`.

Ported from GBA Eval's `quickstart/` (MIT). The environment differs from
GBA Eval's mainly in the target (NES), the oracle's framebuffer format
(palette indices, see below) and the bundled supervisor.

## Zero-to-score

```bash
# from the repo root, with reference/nes-ref.wasm built (see ../README.md)
./quickstart/smoke-test.sh                                  # seconds; should print all ✓
./quickstart/grade.sh candidates/broken/broken.wasm baseline
```

The first grade builds the `nes-eval-grader` image (a few minutes) and, if
`corpus/reference-cache/` is empty, the reference cache (about 30 s).
Output lands in `./results/baseline/`: per-testcase JSON, PNG screenshots,
WAVs, and `summary.json` with the overall score.

## Requirements

- **Docker (default)**: docker with the compose plugin.
- **Native (`grade.sh --native`)**: Rust (rust-toolchain.toml pins 1.96.0);
  uses `target/release/grader`, building it with cargo if missing.

## The environment

`docker compose up -d --build` starts two containers:

| Service | Image | What |
|---|---|---|
| `task` | `nes-eval-task` | Where the agent works, as uid 1000 `agent` in `/task`. Rust 1.96 + `wasm32-unknown-unknown`, wasmtime (pinned), clang/lld, cmake, cc65 (6502 assembler / disassembler), python3 + numpy/scipy, xxd/jq/file, git. Read-only `/task/TASK.md`, `/task/spec/` (ABI, palette, hardware docs in `spec/docs/`) and `/task/dev-roms/`. `/task` is a git repo whose hooks directory is root-owned and empty. |
| `services` | `nes-eval-services` | The oracle: the reference emulator (`reference/nes-ref.wasm`, the grader's reference) behind an HTTP API on `:8001`. On an internal network shared only with `task`; no internet. |

`task` is also on the default network so an agent CLI can reach its model
API; TASK.md tells the agent not to use the internet, nothing enforces it.
Two named volumes persist across `docker compose down`: `task-work`
(`/task`, the agent's work and its git history) and `agent-home`
(`/home/agent`, an installed agent CLI and its login). `docker compose
down -v` wipes both for a fresh start.

### The oracle

Inside `task`, `oracle` is a thin client (`task/oracle-client`) that
forwards to the services container. `oracle help` documents it:

```bash
oracle info
oracle run dev-roms/<game>.nes 120 --dump-frames /task/out    # frame_NNNNN.ppm + .idx per frame
oracle run dev-roms/<game>.nes 600 --replay keys.txt --dump-audio /tmp/ref.wav
oracle run dev-roms/<game>.nes 3000 --dump-final /tmp/last    # only the last frame
```

`.idx` files are the 61 440 little-endian `u16` palette indices that
`emu_framebuffer()` must hold, so a candidate can be diffed byte for byte;
`.ppm` files are the same frame in RGB via `spec/nes_palette.pal`. Audio is
mono i16 at 44 100 Hz. Replays are `<frame> <keys_hex>` lines (bit 0 = A,
1 = B, 2 = Select, 3 = Start, 4 = Up, 5 = Down, 6 = Left, 7 = Right).

The session API drives the reference frame by frame, ABI-style:

```bash
id=$(oracle session start dev-roms/<game>.nes | jq -r .id)
oracle session set-keys "$id" 08                  # Start, from the next frame
oracle session run-frame "$id" 30
oracle session framebuffer "$id" > fb.idx         # 122 880 bytes: u16 LE indices
oracle session framebuffer-rgba "$id" > fb.rgba   # 245 760 bytes: RGBA
oracle session audio "$id" > a.raw                # i16 mono since the last read
oracle session info "$id"
oracle session end "$id"
```

HTTP endpoints (services, port 8001): `POST /oracle` (one-shot CLI),
`POST /oracle/session`, `POST /oracle/session/<id>/set-keys`,
`POST /oracle/session/<id>/run-frame`, `GET /oracle/session/<id>/framebuffer`,
`.../framebuffer-rgba`, `.../audio`, `.../info`, `DELETE /oracle/session/<id>`.
Each `run-frame` replays the session from power-on, so it costs time
proportional to the session's length.

## Running your own agent

```bash
cd quickstart
cp .env.example .env    # add the key(s) your agent needs
docker compose up -d --build

./shell.sh              # interactive shell as uid 1000
./shell.sh oracle info
./shell.sh cargo build --release --lib --target wasm32-unknown-unknown
```

The container ships with no agent CLI. Install one into `agent-home`:

```bash
# Claude Code (native installer → ~/.local/bin/claude)
./shell.sh bash -lc 'curl -fsSL https://claude.ai/install.sh | bash'

# Codex CLI (static binary from the GitHub release; the image has no npm)
./shell.sh bash -lc 'curl -fsSL https://github.com/openai/codex/releases/latest/download/codex-x86_64-unknown-linux-musl.tar.gz \
  | tar -xz -C ~/.local/bin && mv ~/.local/bin/codex-x86_64-unknown-linux-musl ~/.local/bin/codex'

# opencode (pinned release; installs to ~/.opencode/bin)
./shell.sh bash -lc 'curl -fsSL https://opencode.ai/install | bash -s -- --version 1.18.35 --no-modify-path'
```

The compose file passes `ANTHROPIC_API_KEY`, `CLAUDE_CODE_OAUTH_TOKEN`,
`OPENAI_API_KEY`, `CODEX_API_KEY`, `GEMINI_API_KEY`, `GOOGLE_API_KEY` and
`OPENROUTER_API_KEY` through from `.env`.

When the agent has built a wasm, grade it:

```bash
./grade.sh --from-container my-run
```

This copies `/task/target/wasm32-unknown-unknown/release/nes_emu.wasm` out
of the task container to `results/my-run/nes_emu.wasm` and grades it.

## Timed runs: `run-agent.sh`

`run-agent.sh` supervises a benchmark-style run (default 24 hours) from
the host:

- runs `$AGENT_CMD` inside the task container (as `agent`, in `/task`)
  with the contents of TASK.md as the prompt, appended as the last
  argument; whenever the agent exits early it is re-invoked with a short
  "continue working" prompt (`$AGENT_CONTINUE_CMD`, default
  `$AGENT_CMD`), until the time is up, when it is stopped;
- commits `/task` (`git add -A && git commit`) every 30 minutes;
- every 4 hours and at the end, builds the candidate in the container with
  the canonical `cargo build --release --lib --target wasm32-unknown-unknown`,
  copies `nes_emu.wasm` out and grades it into `results/<run>/<checkpoint>/`
  (`checkpoint.json` there records the commit, build status and score);
- writes `results/<run>/run.log`, one `agent-NNN.log` per invocation,
  `run.json` (summary) and `task.bundle` (the agent's git history).

```bash
cd quickstart && docker compose up -d --build

# Claude Code
AGENT_CMD='claude -p --dangerously-skip-permissions --output-format stream-json --verbose' \
AGENT_CONTINUE_CMD='claude -p --continue --dangerously-skip-permissions --output-format stream-json --verbose' \
  ./run-agent.sh --hours 24 --name claude-run

# Codex CLI
AGENT_CMD='codex exec --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check' \
AGENT_CONTINUE_CMD='codex exec resume --last --dangerously-bypass-approvals-and-sandbox --skip-git-repo-check' \
  ./run-agent.sh --hours 24 --name codex-run

# opencode, here with OpenCode Zen's free Space Bunny model at medium thinking
AGENT_CMD='~/.opencode/bin/opencode run -m opencode/space-bunny-free --variant medium --auto --format json' \
AGENT_CONTINUE_CMD='~/.opencode/bin/opencode run --continue -m opencode/space-bunny-free --variant medium --auto --format json' \
  ./run-agent.sh --hours 2 --name space-bunny-run
```

Options: `--hours H` (fractional allowed), `--name RUN`,
`--commit-every MINUTES`, `--grade-every HOURS`, `--native` (grade with
`target/release/grader`). If `AGENT_CMD` mentions `NES_PROMPT` the prompt
is not appended; it is available as `$NES_PROMPT` and in the file
`$NES_PROMPT_FILE` instead (e.g. `AGENT_CMD='my-agent --task-file "$NES_PROMPT_FILE"'`).
`CONTINUE_PROMPT` overrides the continue prompt. Ctrl-C stops the agent,
commits, and exits without a final grade.

A quick dry run with a dummy agent exercises every path:

```bash
AGENT_CMD='bash -c "date >> notes.txt; sleep 20" _' \
  ./run-agent.sh --hours 0.05 --commit-every 1 --grade-every 0.03 --name dummy
```

## Commands

| Script | What it does |
|---|---|
| `smoke-test.sh [docker\|native]` | Preflight: reference wasm, cache, dev ROMs, docs, docker/grader image. |
| `grade.sh <wasm> [name]` | Grade a wasm (docker by default; `--native` for `target/release/grader`). |
| `grade.sh --from-container [name]` | Copy the wasm out of the running task container and grade it. |
| `run-agent.sh` | Timed agent run with periodic commits and graded checkpoints. |
| `shell.sh [cmd]` | Interactive shell (or one-shot command) in the task container. |
| `docker compose up -d --build` | Build and start the task + services containers. |
| `docker compose down` | Stop the containers (volumes kept). |
| `docker compose down -v` | Stop and wipe `/task` and `/home/agent`. |

## Troubleshooting

- **`reference/nes-ref.wasm` missing**: build it with `scripts/build-wasm.sh`
  (needs `third_party/RustyNES`, see the root README).
- **Stale grader image after editing `harness/`**: `./grade.sh --rebuild ...`.
- **`--from-container` can't find the wasm**: is the task container up
  (`docker compose up -d` in `quickstart/`), and has
  `/task/target/wasm32-unknown-unknown/release/nes_emu.wasm` been built?
- **`oracle: service unreachable`**: is the `services` container running
  (`docker compose ps`)? Its log: `docker compose logs services`.

## What's not included

Per-model CLI images, an egress allowlist for the task container, and
remote checkpoint storage. `task/Dockerfile`, `task/TASK.md` and
`run-agent.sh` are the starting points for building them.
