#!/usr/bin/env python3
"""Render an omp (oh-my-pi) session JSONL file as a Markdown transcript.

Usage: scripts/render-omp-transcript.py <session.jsonl> > transcript.md

Sessions live in the agent's home: ~/.omp/agent/sessions/<project>/<id>.jsonl
(subagent sessions sit next to it, in a directory of the same name).

Each step shows the model's reasoning (collapsed), its text, and every tool
call with its arguments and output. Long outputs are cut to --max-output
characters (default 4000) with a note giving the original length; the full
session file stays the source of truth.
"""

import argparse
import datetime
import json
import sys


def ts(iso):
    try:
        return datetime.datetime.fromisoformat(iso.replace("Z", "+00:00")).strftime("%H:%M:%S")
    except (ValueError, AttributeError):
        return "??:??:??"


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

    events = []
    with open(args.session) as f:
        for line in f:
            try:
                events.append(json.loads(line))
            except ValueError:
                continue
    if not events:
        sys.exit("no events")

    out = sys.stdout
    meta = next((e for e in events if e.get("type") == "session"), {})
    title = next((e.get("title") for e in events if e.get("type") == "title" and e.get("title")), "")
    model = next((e.get("model") for e in events if e.get("type") == "model_change"), "?")
    level = next((e.get("thinkingLevel") for e in events if e.get("type") == "thinking_level_change"), "?")
    start = min((e.get("timestamp") for e in events if e.get("timestamp")), default="")
    msgs = [e for e in events if e.get("type") == "message"]
    calls = sum(1 for m in msgs for c in m["message"].get("content", []) if c.get("type") == "toolCall")
    compactions = sum(1 for e in events if e.get("type") == "compaction")
    totals = {}
    for m in msgs:
        u = m["message"].get("usage") or {}
        for k, v in u.items():
            if isinstance(v, (int, float)):
                totals[k] = totals.get(k, 0) + v

    out.write(f"# Transcript: {title or meta.get('id', args.session)}\n\n")
    out.write(f"- Session: `{meta.get('id', '?')}` in `{meta.get('cwd', '?')}`\n")
    out.write(f"- Model: `{model}`, thinking `{level}`\n")
    out.write(f"- Started: {ts(start)}; {len(msgs)} messages, {calls} tool calls, {compactions} compactions\n")
    if totals:
        keep = {k: v for k, v in totals.items() if k in ("input", "output", "cacheRead", "reasoningTokens", "totalTokens")}
        out.write("- Tokens: " + ", ".join(f"{v:,} {k}" for k, v in keep.items()) + "\n")
    out.write("\nTimes are UTC.\n\n")

    for e in events:
        kind = e.get("type")
        if kind == "compaction":
            out.write(f"---\n\n## context compacted · {ts(e.get('timestamp'))}\n\n")
            out.write("<details><summary>summary the model was given instead of its history</summary>\n\n")
            out.write(block(e.get("summary", ""), args.max_output))
            out.write("\n</details>\n\n")
        elif kind == "custom_message":
            out.write(f"_(system nudge: {e.get('customType')})_\n\n")
        elif kind != "message":
            continue
        else:
            m = e["message"]
            role = m.get("role")
            if role == "toolResult":
                continue  # rendered with its call below
            who = "agent" if role == "assistant" else role
            out.write(f"---\n\n## {who} · {ts(e.get('timestamp'))}\n\n")
            for c in m.get("content", []):
                ct = c.get("type")
                if ct == "text" and c.get("text", "").strip():
                    text = c["text"].strip()
                    if role == "user":
                        # Prompts (TASK.md) carry their own headings; quote them.
                        text = "\n".join("> " + ln if ln else ">" for ln in text.splitlines())
                    out.write(text + "\n\n")
                elif ct == "thinking" and c.get("thinking", "").strip():
                    out.write("<details><summary>reasoning</summary>\n\n")
                    out.write(block(c["thinking"].strip(), args.max_output))
                    out.write("\n</details>\n\n")
                elif ct == "toolCall":
                    a = c.get("arguments") or {}
                    shown = a.get("command") if "command" in a else json.dumps(a, indent=1, ensure_ascii=False)
                    out.write(f"**{c.get('name')}**\n\n")
                    out.write(block(str(shown), args.max_output))
                    res = next(
                        (
                            r["message"]
                            for r in msgs
                            if r["message"].get("role") == "toolResult"
                            and any(x.get("toolCallId") == c.get("id") for x in r["message"].get("content", []))
                        ),
                        None,
                    )
                    if res:
                        text = "\n".join(x.get("text", "") for x in res.get("content", []) if x.get("type") == "text")
                        if text.strip():
                            out.write("<details><summary>output</summary>\n\n")
                            out.write(block(text.rstrip(), args.max_output))
                            out.write("\n</details>\n\n")


if __name__ == "__main__":
    main()
