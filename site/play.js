// NES Eval Play: the reference and one candidate emulator, both compiled to
// the NES Eval wasm ABI (spec/ABI.md), run in lockstep in the browser with
// the same input every frame, plus a per-pixel difference panel.
//
// Configuration comes from window.NES_PLAY, written by site/build.py.
"use strict";

const CFG = window.NES_PLAY;
const W = 256, H = 240, PIXELS = W * H;
const FRAME_MS = 1000 / 60.0988; // NTSC frame rate
const KEY = { A: 1, B: 2, SELECT: 4, START: 8, UP: 16, DOWN: 32, LEFT: 64, RIGHT: 128 };
const KEYMAP = {
  ArrowUp: KEY.UP, KeyW: KEY.UP, ArrowDown: KEY.DOWN, KeyS: KEY.DOWN,
  ArrowLeft: KEY.LEFT, KeyA: KEY.LEFT, ArrowRight: KEY.RIGHT, KeyD: KEY.RIGHT,
  KeyX: KEY.A, KeyL: KEY.A, KeyZ: KEY.B, KeyK: KEY.B,
  Enter: KEY.START, ShiftLeft: KEY.SELECT, ShiftRight: KEY.SELECT,
};

// ─── emulator wrapper ────────────────────────────────────────────────────

const moduleCache = new Map();

async function compile(url) {
  if (!moduleCache.has(url)) {
    moduleCache.set(url, fetch(url).then((r) => {
      if (!r.ok) throw new Error(`${url}: HTTP ${r.status}`);
      return r.arrayBuffer();
    }).then((b) => WebAssembly.compile(b)));
  }
  return moduleCache.get(url);
}

class Emu {
  static async create(url) {
    const mod = await compile(url);
    // The ABI forbids imports; trap like the grader does if a module has any.
    const imports = {};
    for (const i of WebAssembly.Module.imports(mod)) {
      (imports[i.module] ??= {})[i.name] = () => { throw new Error(`import ${i.module}.${i.name} called`); };
    }
    const inst = await WebAssembly.instantiate(mod, imports);
    return new Emu(inst.exports);
  }

  constructor(e) {
    this.e = e;
    this.trapped = null;
    this.copy = new Uint16Array(PIXELS);
    if (!e.emu_init()) throw new Error("emu_init returned 0");
  }

  load(rom) {
    const e = this.e;
    this.trapped = null;
    this.copy.fill(0);
    try {
      const ptr = e.emu_rom_buffer();
      new Uint8Array(e.memory.buffer, ptr, rom.length).set(rom);
      if (!e.emu_load_rom(rom.length)) { this.trapped = "rejected the ROM"; return; }
      const boot = e.emu_boot_frames ? e.emu_boot_frames() : 0;
      for (let i = 0; i < boot; i++) { e.emu_run_frame(); this.drain(); }
    } catch (err) {
      this.trapped = `trapped on load: ${err.message}`;
    }
  }

  // One frame. A trapped emulator keeps its last picture, as in the grader.
  frame(keys) {
    if (this.trapped) return;
    const e = this.e;
    try {
      e.emu_set_keys(keys);
      e.emu_run_frame();
      const ptr = e.emu_framebuffer();
      const bytes = new Uint8Array(e.memory.buffer, ptr, PIXELS * 2);
      new Uint8Array(this.copy.buffer).set(bytes);
    } catch (err) {
      this.trapped = `trapped: ${err.message}`;
    }
  }

  drain() {
    if (this.trapped) return null;
    const e = this.e;
    try {
      const n = e.emu_audio_samples();
      if (n <= 0) return null;
      const ptr = e.emu_audio_buffer();
      const out = new Int16Array(n);
      new Uint8Array(out.buffer).set(new Uint8Array(e.memory.buffer, ptr, n * 2));
      return out;
    } catch (err) {
      this.trapped = `trapped: ${err.message}`;
      return null;
    }
  }
}

// ─── replays ─────────────────────────────────────────────────────────────

function parseReplay(text) {
  const events = [];
  for (const raw of text.split("\n")) {
    const line = raw.split("#")[0].trim();
    if (!line) continue;
    const [f, k] = line.split(/\s+/);
    events.push([parseInt(f, 10), parseInt(k, 16)]);
  }
  events.sort((a, b) => a[0] - b[0]);
  return events;
}

function keysAt(events, frame) {
  let keys = 0;
  for (const [f, k] of events) { if (f > frame) break; keys = k; }
  return keys;
}

// ─── audio ───────────────────────────────────────────────────────────────

const WORKLET = `
class NesAudio extends AudioWorkletProcessor {
  constructor() {
    super();
    this.q = []; this.cur = null; this.i = 0; this.queued = 0;
    this.port.onmessage = (e) => {
      if (e.data === "clear") { this.q = []; this.cur = null; this.queued = 0; return; }
      this.q.push(e.data); this.queued += e.data.length;
      while (this.queued > 8820 && this.q.length > 1) this.queued -= this.q.shift().length;
    };
  }
  process(_, outputs) {
    const out = outputs[0][0];
    for (let k = 0; k < out.length; k++) {
      if (!this.cur || this.i >= this.cur.length) {
        this.cur = this.q.shift() || null; this.i = 0;
        if (!this.cur) { out.fill(0, k); break; }
        this.queued -= this.cur.length;
      }
      out[k] = this.cur[this.i++];
    }
    return true;
  }
}
registerProcessor("nes-audio", NesAudio);`;

let audioCtx = null, audioNode = null;

async function ensureAudio() {
  if (audioCtx) { await audioCtx.resume(); return; }
  audioCtx = new AudioContext({ sampleRate: 44100 });
  const url = URL.createObjectURL(new Blob([WORKLET], { type: "text/javascript" }));
  await audioCtx.audioWorklet.addModule(url);
  audioNode = new AudioWorkletNode(audioCtx, "nes-audio", { outputChannelCount: [1] });
  audioNode.connect(audioCtx.destination);
}

function pushAudio(samples) {
  if (!audioNode || !samples) return;
  const f = new Float32Array(samples.length);
  for (let i = 0; i < samples.length; i++) f[i] = samples[i] / 32768;
  audioNode.port.postMessage(f, [f.buffer]);
}

// ─── page state ──────────────────────────────────────────────────────────

const $ = (id) => document.getElementById(id);
const els = {
  game: $("play-game"), model: $("play-model"), mode: $("play-mode"), sound: $("play-sound"),
  reset: $("play-reset"), pause: $("play-pause"), file: $("play-file"),
  status: $("play-status"), note: $("play-note"), candLabel: $("play-cand-label"),
  frame: $("stat-frame"), diverging: $("stat-diverging"), first: $("stat-first"), pixels: $("stat-pixels"),
  canvases: [$("cv-ref"), $("cv-cand"), $("cv-diff")],
};

const ctxs = els.canvases.map((c) => c.getContext("2d"));
const images = ctxs.map((c) => c.createImageData(W, H));
const views = images.map((img) => new Uint32Array(img.data.buffer));

let lut = null;         // 9-bit palette index → 0xAABBGGRR
let ref = null, cand = null;
let rom = null, romName = "";
let replay = null, replayLen = 0;
let liveKeys = 0;
let running = false;
let frame = 0, diverging = 0, firstDiverge = -1, lastPixels = 0;
let generation = 0;     // bumps on every (re)load so stale async work is dropped

async function loadPalette() {
  const pal = new Uint8Array(await (await fetch(CFG.palette)).arrayBuffer());
  lut = new Uint32Array(512);
  for (let i = 0; i < 512; i++) {
    lut[i] = 0xff000000 | (pal[i * 3 + 2] << 16) | (pal[i * 3 + 1] << 8) | pal[i * 3];
  }
}

function setStatus(text) { els.status.textContent = text; }

function resetCounters() {
  frame = 0; diverging = 0; firstDiverge = -1; lastPixels = 0;
  if (audioNode) audioNode.port.postMessage("clear");
}

function currentGame() { return CFG.games.find((g) => g.id === els.game.value); }
function currentModel() { return CFG.models.find((m) => m.id === els.model.value); }

async function loadAll({ customRom } = {}) {
  const gen = ++generation;
  running = false;
  setStatus("Loading…");
  const model = currentModel();
  const game = currentGame();
  try {
    const [r, c] = await Promise.all([Emu.create(CFG.reference), Emu.create(model.wasm)]);
    if (customRom) {
      rom = customRom.bytes; romName = customRom.name;
    } else {
      rom = new Uint8Array(await (await fetch(game.rom)).arrayBuffer());
      romName = game.label;
    }
    const wantReplay = els.mode.value === "replay" && !customRom && game.replay;
    if (wantReplay) {
      replay = parseReplay(await (await fetch(game.replay)).text());
      replayLen = game.frames;
    } else {
      replay = null; replayLen = 0;
    }
    if (gen !== generation) return;
    ref = r; cand = c;
    ref.load(rom); cand.load(rom);
    resetCounters();
    els.candLabel.textContent = model.label;
    updateNote(game, model, !!customRom);
    running = true;
    els.pause.textContent = "Pause";
    setStatus(ref.trapped ? `Reference ${ref.trapped}` : "");
    draw();
  } catch (err) {
    if (gen === generation) setStatus(`Failed to load: ${err.message}`);
  }
}

function updateNote(game, model, custom) {
  const parts = [];
  if (custom) {
    parts.push(`Your ROM: ${romName}. Not part of the graded corpus.`);
  } else if (replay) {
    parts.push(`Replaying the grader's recorded inputs for <code>${game.id}</code> (${game.frames.toLocaleString()} frames). ${game.blurb}`);
  } else {
    parts.push(`Freeplay on <code>${game.id}</code>: your keys go to both emulators. ${game.blurb}`);
  }
  if (game.license && !custom) parts.push(`<a href="${game.license}">Game license and source</a>.`);
  if (model.page) parts.push(`<a href="${model.page}">This emulator's scores</a>.`);
  els.note.innerHTML = parts.join(" ");
}

function step() {
  if (replay && frame >= replayLen) { running = false; els.pause.textContent = "Resume"; setStatus("Replay finished."); return; }
  const keys = replay ? keysAt(replay, frame) : liveKeys;
  ref.frame(keys);
  cand.frame(keys);
  const ra = ref.drain(), ca = cand.drain();
  if (els.sound.value === "ref") pushAudio(ra);
  else if (els.sound.value === "cand") pushAudio(ca);

  const a = ref.copy, b = cand.copy;
  let n = 0;
  for (let i = 0; i < PIXELS; i++) if ((a[i] & 0x1ff) !== (b[i] & 0x1ff)) n++;
  lastPixels = n;
  if (n) { diverging++; if (firstDiverge < 0) firstDiverge = frame; }
  frame++;
}

function draw() {
  if (!ref || !lut) return;
  const a = ref.copy, b = cand.copy;
  const [vr, vc, vd] = views;
  for (let i = 0; i < PIXELS; i++) {
    const ia = a[i] & 0x1ff, ib = b[i] & 0x1ff;
    const pa = lut[ia];
    vr[i] = pa; vc[i] = lut[ib];
    // Matching pixels: the reference at 25% brightness; differing: yellow.
    vd[i] = ia === ib ? (0xff000000 | ((pa >>> 2) & 0x003f3f3f)) : 0xff00d4ff;
  }
  for (let k = 0; k < 3; k++) ctxs[k].putImageData(images[k], 0, 0);
  els.frame.textContent = replay ? `${frame.toLocaleString()} / ${replayLen.toLocaleString()}` : frame.toLocaleString();
  els.diverging.textContent = diverging.toLocaleString();
  els.first.textContent = firstDiverge < 0 ? "—" : firstDiverge.toLocaleString();
  els.pixels.textContent = lastPixels.toLocaleString();
  if (cand.trapped) setStatus(`${currentModel().label} ${cand.trapped}`);
}

let acc = 0, last = performance.now();
function tick(now) {
  requestAnimationFrame(tick);
  const dt = now - last; last = now;
  if (!running) { acc = 0; return; }
  acc += Math.min(dt, 100);
  let n = 0;
  while (acc >= FRAME_MS && n < 4 && running) { step(); acc -= FRAME_MS; n++; }
  if (n) draw();
}

// ─── input ───────────────────────────────────────────────────────────────

function typing(e) { return e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement; }

window.addEventListener("keydown", (e) => {
  const bit = KEYMAP[e.code];
  if (bit === undefined || typing(e)) return;
  liveKeys |= bit; e.preventDefault();
});
window.addEventListener("keyup", (e) => {
  const bit = KEYMAP[e.code];
  if (bit === undefined) return;
  liveKeys &= ~bit; e.preventDefault();
});
window.addEventListener("blur", () => { liveKeys = 0; });

for (const btn of document.querySelectorAll("[data-key]")) {
  const bit = KEY[btn.dataset.key];
  const down = (e) => { e.preventDefault(); liveKeys |= bit; btn.classList.add("on"); };
  const up = (e) => { e.preventDefault(); liveKeys &= ~bit; btn.classList.remove("on"); };
  btn.addEventListener("pointerdown", down);
  btn.addEventListener("pointerup", up);
  btn.addEventListener("pointerleave", up);
  btn.addEventListener("pointercancel", up);
}

// ─── controls ────────────────────────────────────────────────────────────

function syncUrl() {
  const p = new URLSearchParams({ game: els.game.value, model: els.model.value, mode: els.mode.value });
  history.replaceState(null, "", `?${p}`);
}

els.game.addEventListener("change", () => {
  const g = currentGame();
  if (els.mode.value === "replay" && !g.replay) els.mode.value = "free";
  syncUrl(); loadAll();
});
els.model.addEventListener("change", () => { syncUrl(); loadAll(); });
els.mode.addEventListener("change", () => {
  if (els.mode.value === "replay" && !currentGame().replay) {
    setStatus("This testcase has no recorded inputs: it runs without any.");
  }
  syncUrl(); loadAll();
});
els.reset.addEventListener("click", () => loadAll());
els.pause.addEventListener("click", () => {
  if (!ref) return;
  if (replay && frame >= replayLen) { loadAll(); return; }
  running = !running;
  els.pause.textContent = running ? "Pause" : "Resume";
});
els.sound.addEventListener("change", async () => {
  if (els.sound.value !== "off") await ensureAudio();
  else if (audioNode) audioNode.port.postMessage("clear");
});
els.file.addEventListener("change", async () => {
  const f = els.file.files[0];
  if (!f) return;
  els.mode.value = "free";
  await loadAll({ customRom: { name: f.name, bytes: new Uint8Array(await f.arrayBuffer()) } });
  els.file.value = "";
});

// ─── start ───────────────────────────────────────────────────────────────

(async () => {
  const p = new URLSearchParams(location.search);
  if (CFG.games.some((g) => g.id === p.get("game"))) els.game.value = p.get("game");
  if (CFG.models.some((m) => m.id === p.get("model"))) els.model.value = p.get("model");
  if (p.get("mode") === "replay" || p.get("mode") === "free") els.mode.value = p.get("mode");
  if (typeof WebAssembly === "undefined") { setStatus("This browser has no WebAssembly."); return; }
  await loadPalette();
  await loadAll();
  requestAnimationFrame(tick);
})();
