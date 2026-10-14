#!/usr/bin/env python3
"""Render a codex rollout session JSONL as a Markdown transcript.

Usage: scripts/render-codex-transcript.py <rollout-*.jsonl> > transcript.md

Codex stores one rollout per thread under ~/.codex/sessions/YYYY/MM/DD/,
appending across `codex exec resume` invocations, so one file covers a whole
run. Its reasoning items carry only encrypted content (no summaries), so the
transcript shows the conversation, the tool calls and their outputs, and
notes where reasoning was elided.

Long outputs are cut to --max-output characters (default 4000) with a note
giving the original length; the rollout file stays the source of truth.
"""

import argparse
import datetime
import json
import re
import sys


def ts(iso):
    try:
        return datetime.datetime.fromisoformat(iso.replace("Z", "+00:00")).strftime("%H:%M:%S")
    except (ValueError, AttributeError):
        return "??:??:??"


def fence(text):
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


def text_of(content):
    if isinstance(content, str):
        return content
    parts = []
    for c in content or []:
        if isinstance(c, dict) and c.get("text"):
            parts.append(c["text"])
    return "\n".join(parts)


CMD_RE = re.compile(r'cmd\s*:\s*"((?:[^"\\]|\\.)*)"', re.S)


def tool_commands(raw):
    """Pull `cmd:"..."` strings out of codex's exec input; fall back to raw."""
    out = []
    for m in CMD_RE.finditer(raw):
        try:
            out.append(json.loads('"' + m.group(1) + '"'))
        except ValueError:
            out.append(m.group(1))
    if not out:
        return raw
    if len(out) == 1:
        return out[0]
    return "\n\n".join(f"# command {i}\n{c}" for i, c in enumerate(out, 1))


def tool_output(payload):
    chunks = []
    for c in payload.get("output") or []:
        text = c.get("text") if isinstance(c, dict) else str(c)
        if not text:
            continue
        try:
            j = json.loads(text)
            if isinstance(j, dict) and "output" in j:
                text = j["output"]
        except ValueError:
            pass
        chunks.append(text)
    return "\n".join(chunks)


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

    meta = next((e["payload"] for e in events if e.get("type") == "session_meta"), {})
    calls = sum(1 for e in events if (e.get("payload") or {}).get("type") == "custom_tool_call")
    reasonings = sum(1 for e in events if (e.get("payload") or {}).get("type") == "reasoning")
    compactions = sum(1 for e in events if e.get("type") == "compacted")
    tok = {}
    for e in events:
        p = e.get("payload") or {}
        if e.get("type") == "token_usage_record" or p.get("type") == "token_usage_record":
            for k, v in (p.get("usage") or p).items():
                if isinstance(v, (int, float)):
                    tok[k] = tok.get(k, 0) + v

    out = sys.stdout
    out.write(f"# Transcript: {meta.get('session_id', args.session)}\n\n")
    out.write(f"- Codex {meta.get('cli_version', '?')}, cwd `{meta.get('cwd', '?')}`, source `{meta.get('source', '?')}`\n")
    out.write(f"- Started: {ts(meta.get('timestamp'))}; {calls} tool calls, {compactions} compactions\n")
    if tok:
        keep = {k: v for k, v in tok.items() if k in ("input_tokens", "output_tokens", "cached_input_tokens", "reasoning_output_tokens")}
        out.write("- Tokens: " + ", ".join(f"{v:,} {k}" for k, v in keep.items()) + "\n")
    out.write(
        f"\nCodex does not write readable reasoning into its rollouts: the {reasonings} reasoning\n"
        "items hold encrypted content only, so they are noted but not reproduced here.\n\nTimes are UTC.\n\n"
    )

    outputs = {}
    for e in events:
        p = e.get("payload") or {}
        if p.get("type") == "custom_tool_call_output":
            outputs[p.get("call_id")] = p

    for e in events:
        p = e.get("payload") or {}
        t = p.get("type")
        when = ts(e.get("timestamp"))
        if e.get("type") == "compacted":
            out.write(f"---\n\n## context compacted · {when}\n\n")
        elif t == "message":
            role = p.get("role")
            if role in ("developer", "system"):
                continue  # harness boilerplate (skills, permissions)
            text = text_of(p.get("content")).strip()
            if not text:
                continue
            out.write(f"---\n\n## {role} · {when}\n\n")
            if role == "user":
                text = "\n".join("> " + ln if ln else ">" for ln in text.splitlines())
            out.write(text + "\n\n")
        elif t == "reasoning":
            out.write(f"_(reasoning, {when}; encrypted by codex, not shown)_\n\n")
        elif t == "custom_tool_call":
            out.write(f"---\n\n## tool · {when}\n\n")
            out.write(block(tool_commands(p.get("input", "")), args.max_output))
            o = outputs.get(p.get("call_id"))
            res = tool_output(o) if o else ""
            if res.strip():
                out.write("\n<details><summary>output</summary>\n\n")
                out.write(block(res.rstrip(), args.max_output))
                out.write("\n</details>\n\n")


if __name__ == "__main__":
    main()
