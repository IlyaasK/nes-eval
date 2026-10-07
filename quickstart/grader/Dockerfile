# Grader image — the grader binary, nothing else.
#
# The grader loads both the reference and the candidate as wasm modules
# in wasmtime; no native emulator, no third_party/ needed at build time.
# The reference wasm (reference/nes-ref.wasm), the corpus and its
# reference cache are read from the repo clone mounted at /repo:
#
#   docker run --rm -v "$PWD":/repo:ro -v "$PWD/results/run":/out \
#     -v /path/to/dir-with-wasm:/wasm:ro \
#     nes-eval-grader /wasm/candidate.wasm corpus/ /out
#
# The grader's default reference path is relative (reference/nes-ref.wasm),
# so the repo root MUST be mounted at /repo (the image's WORKDIR).
#
# Build:  docker build -f quickstart/grader/Dockerfile -t nes-eval-grader .

FROM rust:1.96-bookworm AS builder

WORKDIR /build
# Workspace manifest + lock, the harness crates, and the palette lockstep
# embeds. The member list is cut down to the harness crates so cargo
# resolves the workspace without the GPL reference sources or
# third_party/ (the lock file still pins every dependency version).
COPY Cargo.toml Cargo.lock ./
COPY harness/ harness/
COPY spec/nes_palette.pal spec/
RUN sed -i 's#^members = .*#members = ["harness/lockstep", "harness/grader"]#; /^rustynes-core/d' Cargo.toml

RUN cargo build --release -p grader && strip target/release/grader

# ─── Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates tini ffmpeg \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/grader /usr/local/bin/grader

WORKDIR /repo
ENTRYPOINT ["/usr/bin/tini", "--", "/usr/local/bin/grader"]
