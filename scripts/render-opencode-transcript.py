#!/usr/bin/env python3
"""Render an `opencode export <session>` JSON file as a Markdown transcript.

Usage: scripts/render-opencode-transcript.py session.json > transcript.md

Each step shows the model's reasoning (collapsed), its text, and every tool
call with its input and output. Long outputs are cut to --max-output
characters (default 4000) with a note giving the original length; the full
export stays the source of truth.
"""

import argparse
import datetime
import json
import sys


def ts(ms):
    return datetime.datetime.fromtimestamp(ms / 1000, datetime.timezone.utc).strftime("%H:%M:%S")


def fence(text):
    """A code fence longer than any backtick run inside `text`."""
    longest = run = 0
    for ch in text:
        run = run + 1 if ch == "`" else 0
        longest = max(longest, run)
    return "`" * max(3, longest + 1)


def block(text, limit):
    note = ""
    if len(text) > limit:
        note = f"\n_(output cut: {limit:,} of {len(text):,} characters shown)_\n"
        text = text[:limit]
    f = fence(text)
    return f"{f}\n{text}\n{f}\n{note}"


def main():
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("session")
    ap.add_argument("--max-output", type=int, default=4000)
    args = ap.parse_args()

    with open(args.session) as f:
        d = json.load(f)
    info = d["info"]
    out = sys.stdout
    msgs = d["messages"]
    start = msgs[0]["info"]["time"]["created"] if msgs else 0
    tok = {"input": 0, "output": 0, "reasoning": 0, "cache_read": 0}
    steps = tools = 0
    for m in msgs:
        for p in m.get("parts", []):
            if p.get("type") == "step-finish":
                steps += 1
                t = p.get("tokens", {})
                tok["input"] += t.get("input", 0)
                tok["output"] += t.get("output", 0)
                tok["reasoning"] += t.get("reasoning", 0)
                tok["cache_read"] += t.get("cache", {}).get("read", 0)
            elif p.get("type") == "tool":
                tools += 1

    out.write(f"# Transcript: {info.get('title', info['id'])}\n\n")
    out.write(f"- Session: `{info['id']}` (opencode {info.get('version', '?')})\n")
    model = info.get("model") or {}
    if model:
        out.write(f"- Model: `{model.get('providerID', '?')}/{model.get('modelID', model.get('id', '?'))}`\n")
    out.write(f"- Started: {ts(start)} UTC; {len(msgs)} messages, {steps} model steps, {tools} tool calls\n")
    out.write(
        f"- Tokens: {tok['input']:,} input, {tok['cache_read']:,} cache reads, "
        f"{tok['output']:,} output ({tok['reasoning']:,} reasoning)\n\n"
    )
    out.write("Times are UTC and elapsed time since the session started.\n\n")

    for m in msgs:
        mi = m["info"]
        created = mi.get("time", {}).get("created", start)
        elapsed = (created - start) // 1000
        role = mi.get("role", "?")
        out.write(f"---\n\n## {role} · {ts(created)} (+{elapsed // 3600}h{elapsed % 3600 // 60:02d}m)\n\n")
        for p in m.get("parts", []):
            kind = p.get("type")
            if kind == "text" and p.get("text", "").strip():
                text = p["text"].strip()
                if role == "user":
                    # Prompts (TASK.md) carry their own headings; quote them
                    # so they don't break the transcript's structure.
                    text = "\n".join("> " + line if line else ">" for line in text.splitlines())
                out.write(text + "\n\n")
            elif kind == "reasoning" and p.get("text", "").strip():
                out.write("<details><summary>reasoning</summary>\n\n")
                out.write(block(p["text"].strip(), args.max_output))
                out.write("\n</details>\n\n")
            elif kind == "tool":
                st = p.get("state", {})
                inp = st.get("input", {})
                if p.get("tool") == "bash" and "command" in inp:
                    shown = inp["command"]
                else:
                    shown = json.dumps(inp, indent=1, ensure_ascii=False)
                out.write(f"**{p.get('tool')}** ({st.get('status', '?')})\n\n")
                out.write(block(shown, args.max_output))
                result = st.get("output") or st.get("error") or ""
                if isinstance(result, str) and result.strip():
                    out.write("<details><summary>output</summary>\n\n")
                    out.write(block(result.rstrip(), args.max_output))
                    out.write("\n</details>\n\n")
            elif kind == "compaction":
                out.write("_(context compacted)_\n\n")


if __name__ == "__main__":
    main()
