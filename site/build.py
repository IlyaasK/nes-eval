#!/usr/bin/env python3
"""Build the NES Eval website (served at https://ilya.as/nes-eval/).

    python3 site/build.py <out_dir> [--media <grades_dir>]

Reads graded checkpoints from results/<run>/ (kept locally, not in git) and
the hand-written posts in site/posts/, and writes a static site to <out_dir>.
--media also cuts the reference | candidate | difference clips from
`grader --emit-video` output directories named <grades_dir>/g-<slug>/.
"""

import html
import json
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SITE = ROOT / "site"
RESULTS = ROOT / "results"
BASE = "/nes-eval"  # URL prefix on ilya.as
REPO = "https://github.com/IlyaasK/nes-eval"

TESTCASES = json.loads((ROOT / "corpus/testcases.json").read_text())["testcases"]
TC = {t["id"]: t for t in TESTCASES}
SECTIONS = ["procedural", "replay", "audio"]
WEIGHTS = {"procedural": 0.20, "replay": 0.60, "audio": 0.20}

GAME_NAMES = {
    "falling-gameplay": "Falling",
    "thwaite-gameplay": "Thwaite",
    "double-action-blaster-guys-gameplay": "Double Action Blaster Guys",
    "minekart-madness-gameplay": "Minekart Madness",
    "nova-the-squirrel-gameplay": "Nova the Squirrel",
    "concentration-room-gameplay": "Concentration Room",
}

# Every graded entry. `run` is a results/<run>/ directory with run.json and
# per-checkpoint grades; `grade` is a single grade directory (baselines).
ENTRIES = [
    dict(
        slug="gpt-6.1-sol", name="GPT-6.1 Sol", kind="model",
        run="codex-gpt-6.1-sol-24h", harness="codex 0.162.1", thinking="high",
        hours=24, env="12af20f",
        attempt="https://github.com/IlyaasK/nes-eval-attempt-gpt-6.1-sol",
        facts=[
            ("Code", "7,199 lines of Rust; 96 commits; 44 analysis tools of its own"),
            ("Tool calls", "1,250; 105 oracle runs and 17 oracle sessions"),
            ("Tokens", "227.9M input (222.9M cached), 1.88M output (1.31M reasoning)"),
            ("Agent sessions", "5 (1 exit, 3 watchdog restarts, 1 ran to the deadline)"),
        ],
        blurb="Five of the six homebrew games replay identically to the reference, "
              "every pixel of every frame; in the sixth, Falling, 4 of 5,800 frames "
              "differ slightly. Audio is exact from hour 12. The one "
              "failure is <code>nmi-and-brk</code>, blargg's NMI-hijacking-BRK test, "
              "which is the entire remaining 1.4%. Its own <code>PROGRESS.md</code> "
              "ends mid-investigation: <em>\"Task not complete; keep improving.\"</em>",
    ),
    dict(
        slug="deepseek-v4.1-flash", name="DeepSeek V4.1 Flash", kind="model",
        run="deepseek-v4.1-flash-24h", harness="omp 18.11.0", thinking="max",
        hours=24, env="77c709b",
        attempt="https://github.com/IlyaasK/nes-eval-attempt-deepseek-v4.1-flash",
        facts=[
            ("Code", "7,329 lines of Rust; 92 commits; 59 Python tools of its own"),
            ("Tool calls", "3,422; 190 shell commands invoking the oracle directly"),
            ("Tokens", "27.7M input, 4.25M output (3.12M reasoning), 1.59B cache reads; about $9"),
            ("Agent sessions", "72 (omp -p ends each turn; median 8.8 minutes)"),
        ],
        blurb="Replays real games almost perfectly (four of six score 0.9998 or "
              "better) but fails five test ROMs outright: the four Holy Mapperel "
              "board tests and blargg's official-opcode suite. One constant in its "
              "frame loop, fitted to the reference, is worth 0.36 of its score; see "
              f"<a href=\"{BASE}/blog/reward-hacking/\">the post on reward hacking</a>.",
    ),
    dict(
        slug="space-bunny-free", name="Space Bunny Free", kind="model",
        run="space-bunny-free", harness="opencode 1.18.35", thinking="medium",
        hours=2, env="f74eee9",
        attempt="https://github.com/IlyaasK/nes-eval-attempt-space-bunny-free",
        facts=[
            ("Code", "3,538 lines of Rust: CPU, PPU, APU, mappers 0–4"),
            ("Tool calls", "78 oracle runs; reference dumps for 39 ROMs"),
            ("Working time", "1h34m of 2h: a free-tier rate limit stalled opencode"),
        ],
        blurb="Every 8-pixel tile is drawn mirrored (nestest's menu reads "
              "\"stset llA nuR\"), which destroys the edges the video metric "
              "compares. A 2-hour run on a free model, kept as the first data point.",
    ),
    dict(
        slug="qwen3.8-27b", name="Qwen3.8 27B", kind="model",
        run="omp-qwen3.8-27b-hetzner-24h", harness="omp 18.10.1", thinking="medium",
        hours=24, env="bbfe423",
        attempt="https://github.com/IlyaasK/nes-eval-attempt-qwen3.8-27b-omp",
        facts=[
            ("Code", "2,677 lines of Rust; 43 saved commits, none of which compile"),
            ("Tool calls", "545: 304 reads, 76 edits, 41 writes, 75 shell commands"),
            ("Tokens", "8.2M input, 1.5M output, 27.1M cache reads; 11 compactions"),
        ],
        blurb="Never produced a binary. For the first 8 hours the crate had no "
              "<code>src/lib.rs</code>, so nothing could be built; it got down to 2 "
              "compile errors at hour 13:45, then regressed to 9.",
    ),
    dict(
        slug="nes-core", name="nes-core", kind="baseline", grade="g-nes-core",
        blurb="A 2,600-line emulator I wrote from the NESdev wiki without looking "
              "at any other emulator, graded once and never tuned against the corpus. "
              "The NES counterpart of GBA Eval's <code>gba-core</code>. Most of its "
              "replay loss turned out to be the same unstated first-frame convention "
              "the DeepSeek run fitted: with a one-line change it scores 0.9426 "
              f"(<a href=\"{BASE}/blog/reward-hacking/#nes-core\">details</a>).",
        attempt=f"{REPO}/tree/main/candidates/nes-core",
        facts=[("Code", "2,600 lines of Rust, MIT")],
    ),
]

# Fixed reference points from README.md (graded with the same grader).
REFERENCE_POINTS = [
    ("RustyNES against itself", 1.0, 1.0, 1.0, 1.0),
    ("Reference + no triangle/noise channel", 0.9452, 1.0, 1.0, 0.7261),
    ("Reference + A and B swapped", 0.5389, 1.0, 0.3282, 0.7100),
    ("Reference + picture one scanline late", 0.3257, 0.0, 0.2095, 1.0),
    ("Reference + all three bugs", 0.1948, 0.0, 0.1500, 0.5241),
    ("Blank screen", 0.0039, 0.0, 0.0065, 0.0),
]

POSTS = [
    dict(
        slug="porting-gba-eval", date="2026-10-16",
        title="Porting GBA Eval to the NES",
        summary="What changed when the method moved to a different console: "
                "palette indices instead of colours, a power-on state that has to "
                "be specified, determinism from the reference cache to the JSON "
                "output, and the grader bugs I found before any model ran.",
    ),
    dict(
        slug="reward-hacking", date="2026-10-16",
        title="Did any model hack the grader?",
        summary="Four models, 24 hours each. No model special-cased a test ROM, "
                "but one constant fitted to the reference is worth 0.36 of a score, "
                "one model worked out which emulator it was graded against, and the "
                "harness itself had a bug that ended a run early.",
    ),
]

# Clips cut by --media: (file, grade slug, testcase, first frame, frames, title, caption)
CLIPS = [
    ("sol-nova", "sol", "nova-the-squirrel-gameplay", 1080, 720,
     "GPT-6.1 Sol on Nova the Squirrel: frame-exact",
     "The difference panel stays black: all 5,400 frames match the reference "
     "pixel for pixel, as in five of the six games."),
    ("deepseek-noskip-dabg", "deepseek-noskip", "double-action-blaster-guys-gameplay", 150, 720,
     "DeepSeek V4.1 Flash with one constant flipped",
     "The published build scores 1.0000 on this game. With "
     "<code>FIRST_FRAME_SKIPS = false</code> the emulator runs one frame ahead "
     "of the reference, every recorded button press reaches the game one frame "
     "later than when the replay was recorded, and the playthrough diverges."),
    ("nescore-thwaite", "nes-core", "thwaite-gameplay", 120, 720,
     "nes-core on Thwaite: one frame ahead, from the first call",
     "The baseline emulates a whole frame on its first <code>emu_run_frame</code> "
     "call; the reference emulates nothing on that call. From then on the "
     "baseline is one frame ahead, the replayed inputs land a frame late, and "
     "5,835 of 6,000 frames differ. The same emulator with that one call skipped "
     "scores 0.9998 here."),
    ("deepseek-nova", "deepseek", "nova-the-squirrel-gameplay", 1080, 720,
     "DeepSeek V4.1 Flash on Nova the Squirrel: a near-miss",
     "4,973 of 5,400 frames differ: sprites and some background tiles are drawn "
     "with the wrong graphics or colours. Positions and game state stay in step "
     "with the reference, so the replay still scores 0.9835. Drawing errors that "
     "don't change behaviour cost little; desynchronisation costs everything."),
    ("spacebunny-nestest", "space-bunny", "nestest", 0, 120,
     "Space Bunny Free on nestest: mirrored tiles",
     "Every 8-pixel tile is drawn right to left. Text that is almost right "
     "scores about the same as a blank screen, because the metric compares "
     "edges."),
]


# ─── helpers ──────────────────────────────────────────────────────────────

def esc(s):
    return html.escape(str(s), quote=True)


def fmt(x, nd=4):
    return "—" if x is None else f"{x:.{nd}f}"


def score_class(x):
    if x is None:
        return "cna"
    if x >= 0.999:
        return "c4"
    if x >= 0.95:
        return "c3"
    if x >= 0.6:
        return "c2"
    if x > 0.05:
        return "c1"
    return "c0"


def bar(x):
    w = 0 if x is None else max(0.0, min(1.0, x)) * 100
    return f'<span class="bar"><i style="width:{w:.1f}%"></i></span>'


def page(title, body, active="", desc="", wide=False):
    nav = [
        ("Leaderboard", f"{BASE}/", "home"),
        ("Results", f"{BASE}/results/", "results"),
        ("Watch", f"{BASE}/watch/", "watch"),
        ("Play", f"{BASE}/play/", "play"),
        ("Blog", f"{BASE}/blog/", "blog"),
        ("Code", REPO, ""),
    ]
    links = "".join(
        f'<a href="{href}"{" class=on" if key and key == active else ""}>{esc(t)}</a>'
        for t, href, key in nav
    )
    full_title = "NES Eval" if title == "NES Eval" else f"{title} · NES Eval"
    desc = desc or ("A benchmark that asks AI coding agents to write an NES emulator "
                    "in 24 hours, graded frame by frame against a reference emulator.")
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{esc(full_title)}</title>
<meta name="description" content="{esc(desc)}">
<meta property="og:title" content="{esc(full_title)}">
<meta property="og:description" content="{esc(desc)}">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Press+Start+2P&family=IBM+Plex+Mono:ital,wght@0,400;0,600;1,400&display=swap">
<link rel="stylesheet" href="{BASE}/style.css">
</head>
<body>
<header class="site"><div class="wrap{' wide' if wide else ''}">
<a class="logo" href="{BASE}/">NES<span>·</span>EVAL</a>
<nav class="site">{links}</nav>
</div></header>
<main class="wrap{' wide' if wide else ''}">
{body}
</main>
<footer class="site"><div class="wrap{' wide' if wide else ''}">
Built by <a href="https://ilya.as/">Ilyaas Kapadia</a>, modelled on Mechanize's
<a href="https://gbaeval.com">GBA Eval</a>. Grader, corpus and environment:
<a href="{REPO}">github.com/IlyaasK/nes-eval</a>.
</div></footer>
</body>
</html>
"""


def write(out, rel, text):
    p = out / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(text)


# ─── data ─────────────────────────────────────────────────────────────────

def load_entry(e, grades_dir):
    if e["kind"] == "baseline":
        d = grades_dir / e["grade"]
        e["final_dir"] = d
        e["summary"] = json.loads((d / "summary.json").read_text())
        e["checkpoints"] = []
        return
    run_dir = RESULTS / e["run"]
    run = json.loads((run_dir / "run.json").read_text())
    e["run_json"] = run
    cps = []
    for cp in run["checkpoints"]:
        s = run_dir / cp["checkpoint"] / "summary.json"
        summ = json.loads(s.read_text()) if s.exists() and cp.get("overall") is not None else None
        cps.append(dict(cp, summary=summ))
    e["checkpoints"] = cps
    final = run_dir / "final"
    e["final_dir"] = final
    s = final / "summary.json"
    e["summary"] = json.loads(s.read_text()) if s.exists() and cps[-1].get("overall") is not None else None


def overall(e):
    return e["summary"]["overall"] if e.get("summary") else None


def section(e, name):
    return e["summary"]["sections"][name]["score"] if e.get("summary") else None


def tc_json(e, tc):
    p = e["final_dir"] / f"{tc}.json"
    return json.loads(p.read_text()) if p.exists() else None


def tc_score(e, tc):
    if not e.get("summary"):
        return None
    return e["summary"]["per_testcase"].get(tc, {}).get("video_score")


def tc_audio(e, tc):
    if not e.get("summary"):
        return None
    return e["summary"]["per_testcase"].get(tc, {}).get("audio_score")


def strip_svg(j, bins=600):
    """Per-frame penalty, 1 - 1/(1 + (d/τ)^4), max-pooled into `bins` columns."""
    hist = j["histogram"]
    tau = j["frame_diff_threshold"]
    n = len(hist)
    cols = [0.0] * bins
    for i, d in enumerate(hist):
        pen = 1.0 - 1.0 / (1.0 + (d / tau) ** 4) if d > 0 else 0.0
        b = min(bins - 1, i * bins // n)
        cols[b] = max(cols[b], pen)
    rects = "".join(
        f'<rect x="{b}" y="{34 - max(1.5, p * 34):.1f}" width="1" height="{max(1.5, p * 34):.1f}"/>'
        for b, p in enumerate(cols) if p > 0.002
    )
    return (f'<svg class="strip" viewBox="0 0 {bins} 34" preserveAspectRatio="none" '
            f'role="img" aria-label="per-frame penalty">{rects}</svg>')


# ─── pages ────────────────────────────────────────────────────────────────

def leaderboard_rows(entries):
    rows = []
    graded = sorted([e for e in entries if e["kind"] == "model"],
                    key=lambda e: -(overall(e) if overall(e) is not None else -1))
    for i, e in enumerate(graded, 1):
        o = overall(e)
        cells = "".join(f'<td class="num">{fmt(section(e, s))}</td>' for s in SECTIONS)
        if o is None:
            cells = '<td colspan="3" class="muted">never compiled</td>'
        rows.append(
            f'<tr><td class="num">{i if o is not None else "—"}</td>'
            f'<td><a href="{BASE}/model/{e["slug"]}/">{esc(e["name"])}</a>'
            f'<div class="small muted">{esc(e["harness"])} · {esc(e["thinking"])} thinking</div></td>'
            f'<td class="num">{e["hours"]} h</td>'
            f'<td class="num"><strong>{fmt(o)}</strong>{bar(o)}</td>{cells}</tr>'
        )
    return "\n".join(rows)


def build_index(out, entries):
    base = next(e for e in entries if e["kind"] == "baseline")
    ref_rows = "".join(
        f'<tr class="dim"><td>{esc(n)}</td><td class="num">{o:.4f}</td>'
        f'<td class="num">{p:.4f}</td><td class="num">{r:.4f}</td><td class="num">{a:.4f}</td></tr>'
        for n, o, p, r, a in REFERENCE_POINTS
    )
    nc = (f'<tr><td><a href="{BASE}/model/nes-core/">nes-core</a> baseline'
          f'<div class="small muted">0.9426 once the first-call convention is matched; '
          f'<a href="{BASE}/blog/reward-hacking/#nes-core">why</a></div></td>'
          f'<td class="num">{fmt(overall(base))}</td>'
          + "".join(f'<td class="num">{fmt(section(base, s))}</td>' for s in SECTIONS) + "</tr>")
    posts = "".join(
        f'<li><span class="date">{p["date"]}</span><a href="{BASE}/blog/{p["slug"]}/">{esc(p["title"])}</a>'
        f'<p>{esc(p["summary"])}</p></li>' for p in POSTS
    )
    n_frames = sum(t["frames"] for t in TESTCASES)
    n_proc = sum(1 for t in TESTCASES if t["section"] == "procedural")
    n_games = sum(1 for t in TESTCASES if t["section"] == "replay")
    body = f"""
<h1>NES Eval</h1>
<p class="lede">An AI coding agent gets an empty Rust project, the NES hardware
documentation and a black-box reference emulator, and has 24 hours to write an
NES emulator that compiles to WebAssembly. The emulator is then run in lockstep
with the reference on {n_proc} test ROMs and {n_games} recorded homebrew
playthroughs, and every one of {n_frames:,} frames of picture and sound is
compared.</p>
<p>It is a port of Mechanize's <a href="https://gbaeval.com">GBA Eval</a> to a
different console. The grader and metrics are theirs; the reference
(<a href="https://github.com/doublegate/RustyNES">RustyNES</a>), the ABI, the
corpus and the agent environment are new, and so are the problems the NES
raised. <a href="{BASE}/blog/porting-gba-eval/">How the port works</a> ·
<a href="{BASE}/blog/reward-hacking/">Did any model hack the grader?</a> ·
<a href="{BASE}/play/">Run the emulators in your browser</a></p>

<h2>Leaderboard</h2>
<div class="scroll"><table>
<tr><th class="num">#</th><th>Model</th><th class="num">Time</th><th class="num">Overall</th>
<th class="num">Procedural</th><th class="num">Replay</th><th class="num">Audio</th></tr>
{leaderboard_rows(entries)}
</table></div>
<p class="small muted">v1.0 results. Each attempt repository holds the model's
working directory saved every 15 minutes, the full transcript and every graded
checkpoint. The grader is deterministic: rebuilding a model's final commit and
grading it reproduces these numbers to the last digit.</p>

<h3>Reference points</h3>
<div class="scroll"><table>
<tr><th>Candidate</th><th class="num">Overall</th><th class="num">Procedural</th><th class="num">Replay</th><th class="num">Audio</th></tr>
{nc}
{ref_rows}
</table></div>

<figure>
<video class="nes" src="{BASE}/clips/sol-nova.mp4" autoplay muted loop playsinline preload="metadata"></video>
<div class="clip-labels"><span>Reference</span><span>GPT-6.1 Sol</span><span>Difference</span></div>
<figcaption>GPT-6.1 Sol's emulator replaying Nova the Squirrel next to the
reference. The difference panel is black because every frame matches.
<a href="{BASE}/watch/">More clips</a>.</figcaption>
</figure>

<h2>How it is graded</h2>
<p><code>overall = 0.60 × replay + 0.20 × procedural + 0.20 × audio</code>, GBA Eval's weights.</p>
<div class="cards">
<div class="card"><div class="w">60%</div><h3>Replay</h3><p>Six homebrew games,
5,400–6,000 frames each, driven by recorded button presses. Every frame is
compared with block SSIM; one frame of timing drift desynchronises the whole
playthrough.</p></div>
<div class="card"><div class="w">20%</div><h3>Procedural</h3><p>{n_proc} self-checking
test ROMs (blargg, kevtris's nestest, Holy Mapperel) for the CPU, PPU, APU and
mappers, scored on their final verdict screen.</p></div>
<div class="card"><div class="w">20%</div><h3>Audio</h3><p>Per-frame log-mel
spectral distance against the reference's PCM, on the four APU mixer tests and
all six games.</p></div>
</div>
<p>Candidates hand over 9-bit palette indices rather than RGB, so a different
colour palette cannot cost points, and the power-on state is pinned in the ABI
so a correct emulator can match the reference exactly. Full rubric:
<a href="{REPO}#scoring-rubric">README</a>; per-testcase scores:
<a href="{BASE}/results/">Results</a>.</p>

<h2>Blog</h2>
<ul class="postlist">{posts}</ul>
"""
    write(out, "index.html", page("NES Eval", body, "home"))


def build_results(out, entries):
    cols = [e for e in entries if e.get("summary")]
    head = "".join(f'<th class="cell"><a href="{BASE}/model/{e["slug"]}/">{esc(e["name"])}</a></th>' for e in cols)

    def table(ids, getter):
        rows = []
        for tc in ids:
            t = TC[tc]
            name = GAME_NAMES.get(tc, tc)
            cells = "".join(
                f'<td class="cell {score_class(getter(e, tc))}">{fmt(getter(e, tc), 3)}</td>' for e in cols
            )
            rows.append(f'<tr><td title="{esc(t["description"])}"><code>{esc(name)}</code>'
                        f'<div class="small muted">{esc(t["subsystem"])} · {t["frames"]:,} frames</div></td>{cells}</tr>')
        return f'<div class="scroll"><table><tr><th>Testcase</th>{head}</tr>{"".join(rows)}</table></div>'

    replay = [t["id"] for t in TESTCASES if t["section"] == "replay"]
    proc = [t["id"] for t in TESTCASES if t["section"] == "procedural"]
    audio = [t["id"] for t in TESTCASES if t["audio_subsystem"]]
    summ = "".join(
        f'<tr><td>{s}</td>' + "".join(f'<td class="cell {score_class(section(e, s))}">{fmt(section(e, s), 3)}</td>' for e in cols) + "</tr>"
        for s in SECTIONS
    ) + '<tr><td><strong>overall</strong></td>' + "".join(
        f'<td class="cell {score_class(overall(e))}"><strong>{fmt(overall(e), 3)}</strong></td>' for e in cols) + "</tr>"
    body = f"""
<h1>Results</h1>
<p>Every graded testcase for every candidate that produced a binary. Qwen3.8 27B
is absent because none of its commits compiled. Hover a testcase for what it
checks.</p>
<div class="scroll"><table><tr><th>Section</th>{head}</tr>{summ}</table></div>
<h2>Replay <span class="muted small">60% · video score, mean over every frame</span></h2>
{table(replay, tc_score)}
<h2>Procedural <span class="muted small">20% · video score of the final verdict screen</span></h2>
{table(proc, tc_score)}
<h2>Audio <span class="muted small">20% · log-mel spectral score</span></h2>
{table(audio, tc_audio)}
"""
    write(out, "results/index.html", page("Results", body, "results"))


def copy_endscreens(out, e, tc):
    dst = out / "img" / e["slug"]
    dst.mkdir(parents=True, exist_ok=True)
    for side in ("ref", "cand"):
        src = e["final_dir"] / f"{tc}.{side}.png"
        if src.exists():
            shutil.copy(src, dst / f"{tc}.{side}.png")


def build_model(out, e):
    facts = [("Model" if e["kind"] == "model" else "Candidate", e["name"])]
    if e["kind"] == "model":
        facts += [("Harness", e["harness"]), ("Thinking", e["thinking"]),
                  ("Time limit", f'{e["hours"]} hours'),
                  ("Environment", f'NES Eval <a href="{REPO}/commit/{e["env"]}"><code>{e["env"]}</code></a>')]
    facts += e.get("facts", [])
    facts.append(("Source", f'<a href="{e["attempt"]}">{esc(e["attempt"].replace("https://", ""))}</a>'))
    if any(m[0] == e["slug"] for m in PLAY_MODELS):
        facts.append(("Play", f'<a href="{BASE}/play/?game=nova-the-squirrel-gameplay&amp;model={e["slug"]}&amp;mode=replay">'
                              f'run it next to the reference in your browser</a>'))
    meta = "".join(f"<tr><th>{k}</th><td>{v}</td></tr>" for k, v in facts)

    parts = [f'<h1>{esc(e["name"])}</h1>', f'<p class="lede">{e["blurb"]}</p>',
             f'<table>{meta}</table>']

    if e.get("summary"):
        parts.append('<h2>Score</h2><div class="scroll"><table><tr><th>Checkpoint</th>'
                     '<th class="num">Overall</th><th class="num">Procedural</th>'
                     '<th class="num">Replay</th><th class="num">Audio</th></tr>')
        cps = e["checkpoints"] or [dict(checkpoint="final", summary=e["summary"])]
        for cp in cps:
            s = cp["summary"]
            label = cp["checkpoint"].split("-")[-1].replace("final", "final")
            if s is None:
                parts.append(f'<tr><td>{label}</td><td colspan="4" class="muted">build failed (rc={cp.get("build_rc")})</td></tr>')
                continue
            parts.append(f'<tr><td>{label}</td><td class="num"><strong>{fmt(s["overall"])}</strong></td>'
                         + "".join(f'<td class="num">{fmt(s["sections"][x]["score"])}</td>' for x in SECTIONS)
                         + "</tr>")
        parts.append("</table></div>")

        parts.append('<h2>Replays</h2><p class="small muted">The strip under each game '
                     'is the per-frame penalty: empty means the frame matched the '
                     'reference, a full-height mark means it scored zero.</p>')
        for tc, name in GAME_NAMES.items():
            j = tc_json(e, tc)
            if not j:
                continue
            # Any luma difference in a frame means some pixel differs; the scored
            # SSIM floors tiny differences away, so this is the stricter count.
            nz = [i for i, m in enumerate(j["audit_luma_mae_frames"]) if m > 0]
            note = ("identical to the reference in every frame" if not nz else
                    f'{len(nz):,} of {j["n_frames"]:,} frames differ, the first at frame {nz[0]}')
            parts.append(f'<h3>{esc(name)} <span class="small">{fmt(tc_score(e, tc))}</span></h3>'
                         f'<p class="small muted">{note} · audio {fmt(tc_audio(e, tc))}</p>{strip_svg(j)}')

        parts.append('<h2>Procedural</h2><div class="scroll"><table><tr><th>Test ROM</th>'
                     '<th>Subsystem</th><th class="num">Score</th></tr>')
        failed = []
        for t in TESTCASES:
            if t["section"] != "procedural":
                continue
            s = tc_score(e, t["id"])
            parts.append(f'<tr><td title="{esc(t["description"])}"><code>{t["id"]}</code></td>'
                         f'<td>{t["subsystem"]}</td><td class="num {score_class(s)}">{fmt(s)}</td></tr>')
            if s is not None and s < 0.5:
                failed.append(t["id"])
        parts.append("</table></div>")
        if failed:
            shown = failed[:6]
            more = (f' Showing {len(shown)} of {len(failed)}; every score is in the table above.'
                    if len(failed) > len(shown) else "")
            parts.append('<h3>Failed verdict screens</h3><p class="small muted">Reference on the '
                         f'left, this candidate on the right.{more}</p>')
            for tc in shown:
                copy_endscreens(out, e, tc)
                parts.append(
                    f'<p><code>{tc}</code> <span class="muted small">{esc(TC[tc]["description"])}</span></p>'
                    f'<div class="pair"><figure><img class="nes" src="{BASE}/img/{e["slug"]}/{tc}.ref.png" alt="reference end screen"></figure>'
                    f'<figure><img class="nes" src="{BASE}/img/{e["slug"]}/{tc}.cand.png" alt="candidate end screen"></figure></div>')
    else:
        parts.append('<h2>Checkpoints</h2><table><tr><th>Checkpoint</th><th>Build</th><th>Score</th></tr>')
        for cp in e["checkpoints"]:
            parts.append(f'<tr><td>{cp["checkpoint"].split("-")[-1]}</td>'
                         f'<td>failed (<code>rc={cp["build_rc"]}</code>)</td><td class="muted">not graded</td></tr>')
        parts.append("</table>")
    write(out, f'model/{e["slug"]}/index.html', page(e["name"], "\n".join(parts), "home"))


def build_watch(out):
    items = []
    for file, slug, tc, start, _, title, cap in CLIPS:
        cand = title.split(" on ")[0].split(" with ")[0]
        live = f'{BASE}/play/?game={tc}&model={PLAY_IDS[slug]}&mode=replay'
        items.append(f"""<h2>{esc(title)}</h2>
<figure>
<video class="nes" src="{BASE}/clips/{file}.mp4" controls muted loop playsinline preload="metadata"></video>
<div class="clip-labels"><span>Reference</span><span>{esc(cand)}</span><span>Difference</span></div>
<figcaption>{cap} <span class="muted">Frames {start}–{start + _clip_len(file)} of <code>{tc}</code>.</span>
<a href="{live}">Run the whole replay live in your browser</a>.</figcaption>
</figure>""")
    body = ("<h1>Watch</h1><p>Each clip is the grader's own lockstep output: the "
            "reference on the left, the candidate in the middle, and their per-pixel "
            "difference on the right, at 60 frames per second with the same recorded "
            "inputs fed to both.</p>" + "\n".join(items))
    write(out, "watch/index.html", page("Watch", body, "watch"))


def _clip_len(file):
    return next(c[4] for c in CLIPS if c[0] == file)


# Emulators offered on the Play page: (id, label, wasm source, results page).
# Relative sources are under the repository; "@grades/" under --media's dir.
PLAY_MODELS = [
    ("gpt-6.1-sol", "GPT-6.1 Sol", "results/codex-gpt-6.1-sol-24h/final/nes_emu.wasm", "gpt-6.1-sol"),
    ("deepseek-v4.1-flash", "DeepSeek V4.1 Flash", "results/deepseek-v4.1-flash-24h/final/nes_emu.wasm", "deepseek-v4.1-flash"),
    # The published DeepSeek source with FIRST_FRAME_SKIPS set to false (see
    # the reward-hacking post); built from the attempt repository's final commit.
    ("deepseek-flag-off", "DeepSeek V4.1 Flash, FIRST_FRAME_SKIPS = false", "@grades/wasm/deepseek-flag-off.wasm", "deepseek-v4.1-flash"),
    ("nes-core", "nes-core (baseline)", "candidates/nes-core/nes_core.wasm", "nes-core"),
    ("space-bunny-free", "Space Bunny Free", "results/space-bunny-free/final/nes_emu.wasm", "space-bunny-free"),
]
# CLIPS grade slugs → Play model ids.
PLAY_IDS = {"sol": "gpt-6.1-sol", "deepseek": "deepseek-v4.1-flash", "deepseek-noskip": "deepseek-flag-off",
            "nes-core": "nes-core", "space-bunny": "space-bunny-free"}


def build_play(out, grades_dir):
    dst = out / "play"
    shutil.rmtree(dst, ignore_errors=True)
    for sub in ("wasm", "roms", "replays", "licenses"):
        (dst / sub).mkdir(parents=True)
    shutil.copy(ROOT / "reference/nes-ref.wasm", dst / "wasm/reference.wasm")
    shutil.copy(ROOT / "spec/nes_palette.pal", dst / "nes_palette.pal")
    shutil.copy(SITE / "play.js", dst / "play.js")

    models = []
    for mid, label, src, page_slug in PLAY_MODELS:
        path = grades_dir / src[len("@grades/"):] if src.startswith("@grades/") else ROOT / src
        shutil.copy(path, dst / f"wasm/{mid}.wasm")
        models.append(dict(id=mid, label=label, wasm=f"{BASE}/play/wasm/{mid}.wasm",
                           page=f"{BASE}/model/{page_slug}/"))

    games = []
    for t in sorted(TESTCASES, key=lambda t: (t["section"] != "replay", t["section"], t["id"])):
        rom_src = ROOT / "corpus/roms" / t["rom_name"]
        shutil.copy(rom_src, dst / f'roms/{t["id"]}.nes')
        g = dict(id=t["id"], label=GAME_NAMES.get(t["id"], t["id"]), group=t["section"],
                 frames=t["frames"], blurb=t["description"], rom=f'{BASE}/play/roms/{t["id"]}.nes',
                 replay=None, license=None)
        if t["replay"]:
            shutil.copy(ROOT / "corpus/replays" / t["replay"], dst / "replays" / t["replay"])
            g["replay"] = f'{BASE}/play/replays/{t["replay"]}'
        if t["rom_name"].startswith("homebrew/"):
            lic = rom_src.with_suffix(".LICENSE.txt")
            shutil.copy(lic, dst / "licenses" / lic.name)
            g["license"] = f"{BASE}/play/licenses/{lic.name}"
        else:
            doc = "holy_mapperel-README.md" if "holy_mapperel" in t["rom_name"] else "LICENSES-RustyNES-tests.md"
            g["license"] = f"{REPO}/blob/main/corpus/roms/test/{doc}"
        games.append(g)

    cfg = dict(reference=f"{BASE}/play/wasm/reference.wasm", palette=f"{BASE}/play/nes_palette.pal",
               models=models, games=games)

    def options(items, selected):
        return "".join(f'<option value="{esc(i["id"])}"{" selected" if i["id"] == selected else ""}>{esc(i["label"])}</option>'
                       for i in items)

    game_opts = (f'<optgroup label="Games (replay section)">{options([g for g in games if g["group"] == "replay"], "nova-the-squirrel-gameplay")}</optgroup>'
                 f'<optgroup label="Test ROMs (procedural section)">{options([g for g in games if g["group"] != "replay"], "")}</optgroup>')
    pad = "".join(f'<button type="button" data-key="{k}">{lbl}</button>'
                  for k, lbl in (("LEFT", "◀"), ("UP", "▲"), ("DOWN", "▼"), ("RIGHT", "▶"),
                                 ("SELECT", "Select"), ("START", "Start"), ("B", "B"), ("A", "A")))
    body = f"""
<h1>Play</h1>
<p class="lede">The reference emulator and a model's emulator, both running in
your browser as the same WebAssembly the grader loads, fed the same input every
frame. Replay the grader's own recorded playthrough, or play it yourself and see
where the two machines stop agreeing.</p>

<div class="play-controls">
<label>Game <select id="play-game">{game_opts}</select></label>
<label>Emulator <select id="play-model">{options(models, "gpt-6.1-sol")}</select></label>
<label>Input <select id="play-mode"><option value="replay" selected>Grader replay</option><option value="free">Your keyboard</option></select></label>
<label>Sound <select id="play-sound"><option value="off" selected>Off</option><option value="ref">Reference</option><option value="cand">Emulator</option></select></label>
<button type="button" id="play-pause">Pause</button>
<button type="button" id="play-reset">Power cycle</button>
<label class="file">Load your own .nes<input type="file" id="play-file" accept=".nes"></label>
</div>
<p id="play-note" class="small muted"></p>
<p id="play-status" class="small" role="status"></p>

<div class="screens">
<figure><canvas id="cv-ref" width="256" height="240"></canvas><figcaption>Reference (RustyNES)</figcaption></figure>
<figure><canvas id="cv-cand" width="256" height="240"></canvas><figcaption id="play-cand-label">Emulator</figcaption></figure>
<figure><canvas id="cv-diff" width="256" height="240"></canvas><figcaption>Difference: <span class="hl">differing pixels</span> over the dimmed reference</figcaption></figure>
</div>

<div class="stats">
<div><span>Frame</span><strong id="stat-frame">0</strong></div>
<div><span>Frames that differ</span><strong id="stat-diverging">0</strong></div>
<div><span>First difference</span><strong id="stat-first">—</strong></div>
<div><span>Pixels off, this frame</span><strong id="stat-pixels">0</strong></div>
</div>

<div class="pad" aria-label="Controller">{pad}</div>
<p class="small muted">Keyboard: arrows or WASD, <kbd>X</kbd> = A, <kbd>Z</kbd> = B,
<kbd>Enter</kbd> = Start, <kbd>Shift</kbd> = Select. Switch Input to "Your keyboard" to play;
in "Grader replay" both emulators get exactly the inputs the grader used.</p>

<h2>What you're looking at</h2>
<p>This is the grader's lockstep loop, moved into the browser: each frame, both
emulators get the same controller state, run one frame, and their 256×240 palette
indices are compared. "Frames that differ" counts frames with any differing pixel,
which is stricter than the score, since the video metric forgives differences too
small to see. Nothing here is scored, and nothing leaves your browser.</p>
<p>Things to try:</p>
<ul>
<li><a href="?game=nova-the-squirrel-gameplay&amp;model=deepseek-v4.1-flash&amp;mode=replay">Nova the Squirrel on DeepSeek V4.1 Flash</a>:
sprites drawn with the wrong graphics, but the game stays in step.</li>
<li><a href="?game=double-action-blaster-guys-gameplay&amp;model=deepseek-flag-off&amp;mode=replay">Double Action Blaster Guys on DeepSeek with
<code>FIRST_FRAME_SKIPS = false</code></a>: one frame of offset and the replay goes
somewhere else (<a href="{BASE}/blog/reward-hacking/#constant">why</a>).</li>
<li><a href="?game=nestest&amp;model=space-bunny-free&amp;mode=replay"><code>nestest</code> on Space Bunny Free</a>: every tile mirrored.</li>
<li><a href="?game=nmi-and-brk&amp;model=gpt-6.1-sol&amp;mode=replay"><code>nmi-and-brk</code> on GPT-6.1 Sol</a>: the one test it fails.</li>
<li><a href="?game=falling-gameplay&amp;model=gpt-6.1-sol&amp;mode=free">Falling on GPT-6.1 Sol with your keyboard</a>:
inputs nobody recorded, and the emulators still have to agree.</li>
</ul>
<p class="small muted">The reference is <a href="https://github.com/doublegate/RustyNES">RustyNES</a>
(GPL-3.0) built for the NES Eval ABI from <a href="{REPO}/tree/main/reference"><code>reference/</code></a>.
Model emulators are the final commits of their
<a href="{BASE}/">attempt repositories</a>. Homebrew games are redistributed under their own
licenses, linked next to each game.</p>
<script>window.NES_PLAY = {json.dumps(cfg)};</script>
<script src="{BASE}/play/play.js" defer></script>
"""
    write(out, "play/index.html", page("Play", body, "play",
                                       "Run the NES Eval reference and a model's emulator side by side in your browser.",
                                       wide=True))


def build_blog(out):
    items = "".join(
        f'<li><span class="date">{p["date"]}</span><a href="{BASE}/blog/{p["slug"]}/">{esc(p["title"])}</a>'
        f'<p>{esc(p["summary"])}</p></li>' for p in POSTS
    )
    write(out, "blog/index.html", page("Blog", f'<h1>Blog</h1><ul class="postlist">{items}</ul>', "blog"))
    for p in POSTS:
        frag = (SITE / "posts" / f'{p["slug"]}.html').read_text().replace("{{BASE}}", BASE)
        body = (f'<article class="prose"><p class="small"><a href="{BASE}/blog/">← Blog</a></p>'
                f'<h1>{esc(p["title"])}</h1><p class="byline">Ilyaas Kapadia · {p["date"]}</p>'
                f'{frag}</article>')
        write(out, f'blog/{p["slug"]}/index.html', page(p["title"], body, "blog", p["summary"]))
    for src in (SITE / "posts" / "img").glob("*"):
        dst = out / "blog" / "img" / src.name
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(src, dst)


# ─── media ────────────────────────────────────────────────────────────────

def build_clips(out, grades_dir):
    """Stack <tc>.ref.mp4 | <tc>.cand.mp4 | <tc>.diff.mp4 side by side, 2× nearest."""
    dst = out / "clips"
    dst.mkdir(parents=True, exist_ok=True)
    for file, slug, tc, start, n, *_ in CLIPS:
        g = grades_dir / f"g-{slug}"
        ins = [g / f"{tc}.{k}.mp4" for k in ("ref", "cand", "diff")]
        missing = [str(p) for p in ins if not p.exists()]
        if missing:
            sys.exit(f"missing video: {missing}")
        args = ["ffmpeg", "-y", "-loglevel", "error"]
        for p in ins:
            args += ["-i", str(p)]
        # Cut by frame number, not timestamp: the encoders run at the NES's
        # 60.0988 Hz, so seconds don't map to whole frames.
        trims = "".join(
            f"[{i}:v]trim=start_frame={start}:end_frame={start + n},setpts=PTS-STARTPTS[v{i}];"
            for i in range(3)
        )
        args += [
            "-filter_complex",
            trims + "[v0][v1][v2]hstack=inputs=3,scale=iw*2:ih*2:flags=neighbor,format=yuv420p[v]",
            "-map", "[v]", "-an", "-c:v", "libx264", "-preset", "slow", "-crf", "26",
            "-tune", "animation", "-movflags", "+faststart", str(dst / f"{file}.mp4"),
        ]
        subprocess.run(args, check=True)
        print(f"clip {file}.mp4 {(dst / f'{file}.mp4').stat().st_size / 1e6:.1f} MB")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    out = Path(sys.argv[1]).resolve()
    grades_dir = Path.home() / "nes-site-work"
    media = "--media" in sys.argv
    if media:
        grades_dir = Path(sys.argv[sys.argv.index("--media") + 1]).resolve()
    out.mkdir(parents=True, exist_ok=True)
    entries = [dict(e) for e in ENTRIES]
    for e in entries:
        load_entry(e, grades_dir)
    shutil.rmtree(out / "img", ignore_errors=True)  # end screens are re-copied below
    shutil.copy(SITE / "style.css", out / "style.css")
    build_index(out, entries)
    build_results(out, entries)
    for e in entries:
        build_model(out, e)
    build_watch(out)
    build_play(out, grades_dir)
    build_blog(out)
    if media:
        build_clips(out, grades_dir)
    print(f"site written to {out}")


if __name__ == "__main__":
    main()
