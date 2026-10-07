# Reference-oracle sidecar.
#
#   POST :8001/oracle              — one-shot oracle CLI proxy
#   POST :8001/oracle/session/*    — stateful session API
#
# Sits on the private compose network with the `task` container. The
# agent-side `oracle` thin client (quickstart/task/oracle-client) forwards
# argv here over HTTP. The reference wasm and the oracle binary exist only
# in this image.
#
# Build context MUST be the repo root (the compose file handles this).

# ─── Builder: oracle binary ──────────────────────────────────────────
FROM rust:1.96-bookworm AS builder

WORKDIR /build
# Only what the oracle needs: the workspace manifest + lock, the harness
# crates, and the palette that lockstep embeds. The workspace member list
# is cut down to the harness crates so cargo resolves the workspace
# without the GPL reference sources or third_party/ (the lock file still
# pins every dependency version). rust-toolchain.toml is left out: the
# image's 1.96 toolchain is the pinned one.
COPY Cargo.toml Cargo.lock ./
COPY harness/ harness/
COPY spec/nes_palette.pal spec/
RUN sed -i 's#^members = .*#members = ["harness/lockstep", "harness/grader", "harness/oracle"]#; /^rustynes-core/d' Cargo.toml

RUN cargo build --release -p oracle && strip target/release/oracle

# ─── Runtime ─────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
        python3 ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

# Oracle binary + the reference wasm it loads, the same file the grader
# uses (reference/nes-ref.wasm), so oracle output is byte-identical to
# the graded reference.
COPY --from=builder --chown=root:root --chmod=0755 \
    /build/target/release/oracle /usr/local/bin/oracle
RUN install -d -m 0755 /opt/nes-eval
COPY --chown=root:root --chmod=0644 \
    reference/nes-ref.wasm /opt/nes-eval/nes-ref.wasm

# HTTP server + entrypoint.
COPY --chown=root:root --chmod=0755 \
    quickstart/services/oracle-server.py /usr/local/bin/oracle-server
COPY --chown=root:root --chmod=0755 \
    quickstart/services/run.sh /usr/local/bin/services-run

# Dedicated uid 5000. Shared group `nes` (gid 3000) matches the task
# image's group so both sides can read+write /task files.
RUN groupadd -g 3000 nes && \
    useradd -r -u 5000 -m -d /home/services -G nes services

USER services
EXPOSE 8001
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["/usr/local/bin/services-run"]
