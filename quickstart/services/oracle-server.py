#!/usr/bin/env python3
"""Oracle HTTP proxy for a single agent.

Two surfaces:

  POST /oracle
      One-shot CLI proxy. The agent's /usr/local/bin/oracle forwards argv +
      cwd; we run /usr/local/bin/oracle locally and return stdout/stderr/
      returncode. All --dump-frames/--dump-audio/--replay paths have
      already been staged into /task/.oracle-out by the thin client.

  POST   /oracle/session                    {rom} → {id, info}
  POST   /oracle/session/<id>/set-keys      {keys}
  POST   /oracle/session/<id>/run-frame     {count}
  GET    /oracle/session/<id>/framebuffer       → 256*240*2 bytes, u16 LE palette indices
  GET    /oracle/session/<id>/framebuffer-rgba  → 256*240*4 bytes, RGBA
  GET    /oracle/session/<id>/audio             → raw i16 mono LE, 44100 Hz (drains)
  GET    /oracle/session/<id>/info              → JSON state
  DELETE /oracle/session/<id>

The session API keeps per-session replay state (rom, cur_frame, event
list) on the services side. Each run-frame call re-invokes the oracle
binary from power-on with the accumulated replay, writing only the last
frame (`--dump-final`) and the new frames' audio (`--from`), so a call
costs O(current_frame) emulation. Audio from successive run-frame calls
accumulates until read.

Rate limits: per-token bucket (burst 100 req, refill 10 req/s). Max 5
concurrent sessions per token. Idle sessions > 30 min are reaped.

The reference emulator lives only here: the agent's container has no
oracle executable and no way to ptrace or process_vm_readv this process.

Forked from GBA Eval's quickstart/services/oracle-server.py (MIT).
"""
from __future__ import annotations
import hmac
import json
import logging
import os
import shutil
import subprocess
import threading
import time
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

BIND = ("0.0.0.0", 8001)
ORACLE = "/usr/local/bin/oracle"
TASK_ROOT = "/task"
SESSION_ROOT = os.path.join(TASK_ROOT, ".oracle-sessions")

MAX_ARGS = 32
MAX_ARG_LEN = 4096
MAX_WALL_SECONDS = 600

SESSION_IDLE_TTL = 30 * 60        # 30 min
MAX_CONCURRENT_SESSIONS = 5       # per token
MAX_EVENTS_PER_SESSION = 100_000  # protects against pathological growth
MAX_RUN_COUNT = 100_000           # frames per run-frame call
MAX_PENDING_AUDIO = 64 << 20      # bytes of undrained audio kept per session
RATE_BURST = 100                  # token bucket size
RATE_REFILL = 10.0                # tokens per second

FB_W, FB_H = 256, 240
FB_BYTES = FB_W * FB_H * 2        # u16 palette indices
RGBA_BYTES = FB_W * FB_H * 4
AUDIO_RATE = 44100

EXPECTED_TOKEN = os.environ.get("NES_SERVICES_TOKEN", "")
CHILD_ENV = {"PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
             "HOME": "/tmp", "LANG": "C.UTF-8"}

logging.basicConfig(level=logging.INFO, format="%(asctime)s %(message)s")
log = logging.getLogger("oracle-server")

# ─── Session registry ────────────────────────────────────────────────

_sessions_lock = threading.Lock()
_sessions: dict[str, dict] = {}
# _sessions[sid] = {
#   "token": str,              # owning agent token (or "" in dev)
#   "rom": str,                # absolute path under /task
#   "cur_frame": int,          # frames run so far
#   "events": list[(frame, keys)],
#   "last_fb": bytes,          # u16 LE indices of the last frame run
#   "last_rgba": bytes,        # RGBA of the last frame run
#   "audio": bytearray,        # i16 mono LE since last drain
#   "dir": str,                # /task/.oracle-sessions/<sid>/
#   "busy": bool,              # a run-frame call is in flight
#   "last_activity": float,
# }


# ─── Rate limiter ────────────────────────────────────────────────────

_buckets_lock = threading.Lock()
_buckets: dict[str, tuple[float, float]] = {}  # token → (tokens, last_update)


def _rate_check(token: str) -> bool:
    """Token bucket. Returns True if call allowed."""
    now = time.time()
    with _buckets_lock:
        tokens, last = _buckets.get(token, (float(RATE_BURST), now))
        tokens = min(float(RATE_BURST), tokens + (now - last) * RATE_REFILL)
        if tokens < 1.0:
            _buckets[token] = (tokens, now)
            return False
        _buckets[token] = (tokens - 1.0, now)
        return True


# ─── Path validation (shared with /oracle) ───────────────────────────

def _sanitize_cwd(raw: str) -> str:
    if not raw:
        return TASK_ROOT
    cwd = os.path.realpath(raw)
    if cwd == TASK_ROOT or cwd.startswith(TASK_ROOT + "/"):
        return cwd
    return TASK_ROOT


def _paths_inside_task(args: list[str]) -> str | None:
    for a in args:
        if not a.startswith("/"):
            continue
        resolved = os.path.realpath(a)
        if resolved != TASK_ROOT and not resolved.startswith(TASK_ROOT + "/"):
            return a
    return None


def _resolve_rom(raw: str, cwd: str) -> str | None:
    """ROM path must resolve under /task."""
    p = raw if raw.startswith("/") else os.path.join(cwd, raw)
    resolved = os.path.realpath(p)
    if resolved != TASK_ROOT and not resolved.startswith(TASK_ROOT + "/"):
        return None
    if not os.path.isfile(resolved):
        return None
    return resolved


# ─── Session operations ──────────────────────────────────────────────

def _reap_idle():
    cutoff = time.time() - SESSION_IDLE_TTL
    with _sessions_lock:
        stale = [sid for sid, s in _sessions.items()
                 if s["last_activity"] < cutoff and not s["busy"]]
        for sid in stale:
            _cleanup_session_locked(sid)
    if stale:
        log.info("reaped %d idle session(s)", len(stale))


def _cleanup_session_locked(sid: str):
    s = _sessions.pop(sid, None)
    if s and os.path.isdir(s["dir"]):
        shutil.rmtree(s["dir"], ignore_errors=True)


def _count_sessions_for(token: str) -> int:
    with _sessions_lock:
        return sum(1 for s in _sessions.values() if s["token"] == token)


def _write_replay(s: dict) -> str:
    path = os.path.join(s["dir"], "replay.txt")
    with open(path, "w") as f:
        f.writelines(f"{frame} {keys:02x}\n" for frame, keys in s["events"])
    return path


def _ppm_to_rgba(data: bytes) -> bytes | None:
    """P6 PPM as written by the oracle ("P6\\n256 240\\n255\\n" + RGB) → RGBA."""
    header = f"P6\n{FB_W} {FB_H}\n255\n".encode()
    if not data.startswith(header):
        return None
    rgb = data[len(header):]
    if len(rgb) != FB_W * FB_H * 3:
        return None
    out = bytearray(RGBA_BYTES)
    out[0::4] = rgb[0::3]
    out[1::4] = rgb[1::3]
    out[2::4] = rgb[2::3]
    out[3::4] = b"\xff" * (FB_W * FB_H)
    return bytes(out)


def _wav_pcm(path: str) -> bytes:
    """PCM payload of the oracle's canonical 44-byte-header WAV."""
    try:
        with open(path, "rb") as f:
            wav = f.read()
    except OSError:
        return b""
    return wav[44:] if len(wav) >= 44 and wav[:4] == b"RIFF" else b""


def _session_run_frames(sid: str, count: int) -> tuple[int, str]:
    """Advance the session by `count` frames. Returns (status_code, err)."""
    with _sessions_lock:
        s = _sessions.get(sid)
        if not s:
            return 404, "session not found"
        if s["busy"]:
            return 409, "a run-frame call is already in flight for this session"
        s["busy"] = True
        rom = s["rom"]
        cur = s["cur_frame"]
        new_cur = cur + count
        replay_path = _write_replay(s)
        sdir = s["dir"]
    try:
        frames_dir = os.path.join(sdir, "frames")
        audio_path = os.path.join(sdir, "audio.wav")
        shutil.rmtree(frames_dir, ignore_errors=True)
        try:
            os.remove(audio_path)
        except OSError:
            pass
        try:
            # One run from power-on: the last frame for the framebuffer
            # endpoints, the audio of the `count` new frames for /audio.
            proc = subprocess.run(
                [ORACLE, "run", rom, str(new_cur),
                 "--replay", replay_path,
                 "--from", str(cur),
                 "--dump-final", frames_dir,
                 "--dump-audio", audio_path],
                cwd=TASK_ROOT, capture_output=True, text=True,
                timeout=MAX_WALL_SECONDS, env=CHILD_ENV,
            )
        except subprocess.TimeoutExpired:
            return 504, "oracle timeout"
        except Exception as e:
            return 500, f"oracle error: {e}"
        if proc.returncode != 0:
            return 500, f"oracle rc={proc.returncode}: {proc.stderr[-500:]}"

        stem = os.path.join(frames_dir, f"frame_{new_cur - 1:05d}")
        try:
            with open(stem + ".idx", "rb") as f:
                fb = f.read()
            with open(stem + ".ppm", "rb") as f:
                rgba = _ppm_to_rgba(f.read())
        except OSError:
            fb, rgba = b"", None
        if len(fb) != FB_BYTES or rgba is None:
            return 500, "oracle produced no framebuffer"
        audio = _wav_pcm(audio_path)
        shutil.rmtree(frames_dir, ignore_errors=True)

        with _sessions_lock:
            s = _sessions.get(sid)
            if not s:
                return 404, "session vanished mid-run"
            s["cur_frame"] = new_cur
            s["last_fb"] = fb
            s["last_rgba"] = rgba
            s["audio"] += audio
            if len(s["audio"]) > MAX_PENDING_AUDIO:
                del s["audio"][:len(s["audio"]) - MAX_PENDING_AUDIO]
            s["last_activity"] = time.time()
        return 200, ""
    finally:
        with _sessions_lock:
            if sid in _sessions:
                _sessions[sid]["busy"] = False


# ─── HTTP handler ────────────────────────────────────────────────────

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        log.info("%s - " + fmt, self.address_string(), *args)

    def _reply(self, code: int, obj):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _reply_binary(self, code: int, data: bytes):
        self.send_response(code)
        self.send_header("Content-Type", "application/octet-stream")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def _token(self) -> str:
        got = self.headers.get("Authorization") or ""
        return got[7:] if got.startswith("Bearer ") else ""

    def _check_token(self) -> bool:
        if not EXPECTED_TOKEN:
            return True
        return hmac.compare_digest(self._token(), EXPECTED_TOKEN)

    def _rate_limited(self) -> bool:
        return not _rate_check(self._token() or "_noauth")

    def _read_json(self) -> dict | None:
        length = int(self.headers.get("Content-Length") or 0)
        if length < 0 or length > 64 * 1024:
            return None
        if length == 0:
            return {}
        try:
            obj = json.loads(self.rfile.read(length))
        except Exception:
            return None
        return obj if isinstance(obj, dict) else None

    def _owned_session_locked(self, sid: str) -> dict | None:
        s = _sessions.get(sid)
        if not s or (EXPECTED_TOKEN and s["token"] != self._token()):
            return None
        return s

    # ── GET ──────────────────────────────────────────────────────────

    def do_GET(self):
        if self.path == "/health":
            self._reply(200, {"ok": True, "service": "oracle"})
            return
        if not self._check_token():
            self._reply(401, {"error": "unauthorized"})
            return
        if self._rate_limited():
            self._reply(429, {"error": "rate limit"})
            return

        # /oracle/session/<id>/framebuffer | framebuffer-rgba | audio | info
        parts = self.path.strip("/").split("/")
        if len(parts) == 4 and parts[0] == "oracle" and parts[1] == "session":
            sid, action = parts[2], parts[3]
            _reap_idle()
            with _sessions_lock:
                s = self._owned_session_locked(sid)
                if s is None:
                    self._reply(404, {"error": "session not found"})
                    return
                s["last_activity"] = time.time()
                if action == "framebuffer":
                    body = s["last_fb"] or b"\x00" * FB_BYTES
                elif action == "framebuffer-rgba":
                    body = s["last_rgba"] or b"\x00" * RGBA_BYTES
                elif action == "audio":
                    body = bytes(s["audio"])
                    s["audio"] = bytearray()   # drain on read
                elif action == "info":
                    body = {
                        "id": sid,
                        "rom": s["rom"],
                        "cur_frame": s["cur_frame"],
                        "events": len(s["events"]),
                        "keys": s["events"][-1][1] if s["events"] else 0,
                        "pending_audio_samples": len(s["audio"]) // 2,
                    }
                else:
                    self._reply(404, {"error": "unknown session action"})
                    return
            if isinstance(body, dict):
                self._reply(200, body)
            else:
                self._reply_binary(200, body)
            return

        self._reply(404, {"error": "not found"})

    # ── POST ─────────────────────────────────────────────────────────

    def do_POST(self):
        if not self._check_token():
            self._reply(401, {"error": "unauthorized"})
            return
        if self._rate_limited():
            self._reply(429, {"error": "rate limit"})
            return

        if self.path == "/oracle":
            self._handle_one_shot()
            return
        if self.path == "/oracle/session":
            self._handle_session_start()
            return

        parts = self.path.strip("/").split("/")
        if len(parts) == 4 and parts[0] == "oracle" and parts[1] == "session":
            sid, action = parts[2], parts[3]
            if action == "set-keys":
                self._handle_session_set_keys(sid)
                return
            if action == "run-frame":
                self._handle_session_run_frame(sid)
                return

        self._reply(404, {"error": "not found"})

    # ── DELETE ───────────────────────────────────────────────────────

    def do_DELETE(self):
        if not self._check_token():
            self._reply(401, {"error": "unauthorized"})
            return
        if self._rate_limited():
            self._reply(429, {"error": "rate limit"})
            return
        parts = self.path.strip("/").split("/")
        if len(parts) == 3 and parts[0] == "oracle" and parts[1] == "session":
            sid = parts[2]
            with _sessions_lock:
                if self._owned_session_locked(sid) is None:
                    self._reply(404, {"error": "session not found"})
                    return
                _cleanup_session_locked(sid)
            self._reply(200, {"ok": True})
            return
        self._reply(404, {"error": "not found"})

    # ── One-shot /oracle ─────────────────────────────────────────────

    def _handle_one_shot(self):
        req = self._read_json()
        if req is None:
            self._reply(400, {"error": "bad json"})
            return
        args = req.get("args") or []
        if not isinstance(args, list) or len(args) > MAX_ARGS:
            self._reply(400, {"error": "bad args"})
            return
        for a in args:
            if not isinstance(a, str) or len(a) > MAX_ARG_LEN:
                self._reply(400, {"error": "arg type/len"})
                return
        bad = _paths_inside_task(args)
        if bad is not None:
            self._reply(400, {"error": f"path outside /task: {bad}"})
            return
        cwd = _sanitize_cwd(req.get("cwd") or TASK_ROOT)

        t0 = time.time()
        try:
            proc = subprocess.run(
                [ORACLE, *args], cwd=cwd,
                capture_output=True, text=True,
                timeout=MAX_WALL_SECONDS, env=CHILD_ENV,
            )
            out, err, rc = proc.stdout, proc.stderr, proc.returncode
        except subprocess.TimeoutExpired as e:
            out, err, rc = e.stdout or "", (e.stderr or "") + "\noracle: timed out", 124
        except Exception as e:
            out, err, rc = "", f"oracle server error: {e}", 1
        dur = time.time() - t0
        log.info("args=%r cwd=%s rc=%d dur=%.2fs", args[:8], cwd, rc, dur)
        self._reply(200, {
            "stdout": out, "stderr": err, "returncode": rc,
            "duration_s": round(dur, 3),
        })

    # ── Session start ────────────────────────────────────────────────

    def _handle_session_start(self):
        _reap_idle()
        req = self._read_json()
        if req is None:
            self._reply(400, {"error": "bad json"})
            return
        raw_rom = req.get("rom") or ""
        if not isinstance(raw_rom, str) or len(raw_rom) > MAX_ARG_LEN:
            self._reply(400, {"error": "bad rom"})
            return
        cwd = _sanitize_cwd(req.get("cwd") or TASK_ROOT)
        rom = _resolve_rom(raw_rom, cwd)
        if rom is None:
            self._reply(400, {"error": f"rom path invalid or outside /task: {raw_rom}"})
            return

        token = self._token()
        if _count_sessions_for(token) >= MAX_CONCURRENT_SESSIONS:
            self._reply(429, {"error": f"max {MAX_CONCURRENT_SESSIONS} concurrent sessions"})
            return

        sid = uuid.uuid4().hex
        sdir = os.path.join(SESSION_ROOT, sid)
        os.makedirs(sdir, exist_ok=True)
        try:
            os.chmod(sdir, 0o2775)
        except OSError:
            pass

        with _sessions_lock:
            _sessions[sid] = {
                "token": token,
                "rom": rom,
                "cur_frame": 0,
                "events": [],
                "last_fb": b"",
                "last_rgba": b"",
                "audio": bytearray(),
                "dir": sdir,
                "busy": False,
                "last_activity": time.time(),
            }
        log.info("session start sid=%s rom=%s", sid, rom)
        self._reply(200, {
            "id": sid,
            "rom": rom,
            "info": {
                "width": FB_W, "height": FB_H,
                "framebuffer_bytes": FB_BYTES,
                "framebuffer_format": "u16 little-endian palette indices, (emphasis << 6) | colour",
                "framebuffer_rgba_bytes": RGBA_BYTES,
                "audio_format": f"i16 mono little-endian, {AUDIO_RATE} Hz",
                "audio_rate": AUDIO_RATE,
            },
        })

    # ── Session set-keys ─────────────────────────────────────────────

    def _handle_session_set_keys(self, sid: str):
        req = self._read_json()
        if req is None:
            self._reply(400, {"error": "bad json"})
            return
        keys = req.get("keys")
        if isinstance(keys, str):
            try:
                keys = int(keys, 16)
            except ValueError:
                self._reply(400, {"error": "keys must be a hex string or int"})
                return
        if not isinstance(keys, int) or keys < 0 or keys > 0xFF:
            self._reply(400, {"error": "keys must be 0..0xFF"})
            return

        with _sessions_lock:
            s = self._owned_session_locked(sid)
            if s is None:
                self._reply(404, {"error": "session not found"})
                return
            if len(s["events"]) >= MAX_EVENTS_PER_SESSION:
                self._reply(400, {"error": "too many events in session"})
                return
            # Keys apply from the next frame to run. Coalesce: a second
            # set-keys before running a frame overwrites the first.
            if s["events"] and s["events"][-1][0] == s["cur_frame"]:
                s["events"][-1] = (s["cur_frame"], keys)
            else:
                s["events"].append((s["cur_frame"], keys))
            s["last_activity"] = time.time()
            cur = s["cur_frame"]
        self._reply(200, {"ok": True, "cur_frame": cur})

    # ── Session run-frame ────────────────────────────────────────────

    def _handle_session_run_frame(self, sid: str):
        req = self._read_json() or {}
        count = req.get("count", 1)
        if not isinstance(count, int) or count < 1 or count > MAX_RUN_COUNT:
            self._reply(400, {"error": f"count must be 1..{MAX_RUN_COUNT}"})
            return
        with _sessions_lock:
            if self._owned_session_locked(sid) is None:
                self._reply(404, {"error": "session not found"})
                return
        status, err = _session_run_frames(sid, count)
        if status != 200:
            self._reply(status, {"error": err})
            return
        with _sessions_lock:
            s = _sessions.get(sid)
            if s is None:
                self._reply(404, {"error": "session vanished"})
                return
            body = {
                "ok": True, "cur_frame": s["cur_frame"],
                "framebuffer_bytes": FB_BYTES,
                "pending_audio_samples": len(s["audio"]) // 2,
            }
        self._reply(200, body)


# ─── Main ────────────────────────────────────────────────────────────

def _reaper_thread():
    while True:
        time.sleep(60)
        try:
            _reap_idle()
        except Exception as e:
            log.warning("reaper error: %s", e)


def main():
    # umask 002 so agent (uid 1000) can read+unlink files services
    # (uid 5000) wrote — both share group `nes` via setgid on /task.
    os.umask(0o002)
    os.makedirs(SESSION_ROOT, exist_ok=True)
    try:
        os.chmod(SESSION_ROOT, 0o2775)
    except OSError:
        pass
    threading.Thread(target=_reaper_thread, daemon=True).start()
    server = ThreadingHTTPServer(BIND, Handler)
    server.daemon_threads = True
    log.info("oracle-server listening on %s:%d", *BIND)
    server.serve_forever()


if __name__ == "__main__":
    main()
