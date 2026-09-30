// Cut 10 §4 — sound. Sparse, synthesised in-browser (WebAudio, no assets, no loading). Every cue is one instrument: at most two
// oscillators and one noise burst (a shared white-noise buffer through one biquad) under ONE gain envelope, ≤ 200 ms (the death
// note is the one exception: a single low note fading over 1 s). A camp drone: a very quiet two-note pad (root + fifth) whose
// pentatonic root follows the biome of the next floor.
//
// Juice pass 2 (docs/JUICE.md §6): per foe-family timbres (flesh · bone · ooze · metal · spirit) for the hero's blows (`strike`),
// the blows he takes (`hit`) and the kills (`slay`); every combat cue is jittered (pitch, filter, noise slice, length) so no two
// in a row are identical; boss moments (`boss_in` · `boss_break` · `boss_down`); the chrome (`click` · `buy` · `edit` · `verdict`)
// and the Cut 27 surfaces (`fold` · `scene` · `scene_end`); per-biome ambience beds under the watch (`bed`, a looped noise texture
// rendered once per biome plus sparse one-shots — drips, clanks, crackle, a far bell — at random times, pans and pitches, never an
// oscillator); a master bus into a soft limiter (linear to −3 dBFS, a tanh shoulder under full scale) so a stack of cues never clips.
//
//   audio.cue("hit", { dmg, kind })     hero hurt: a thud whose pitch falls with the damage, the attacker's family on top
//   audio.cue("strike", { dmg, kind })  the hero's blow lands on a foe of `kind`
//   audio.cue("slay", { kind })         a foe dies, by family
//   audio.cue("rule")                   a player row fires (not a chore): a soft tick
//   audio.cue("telegraph")              a rising two-note warning
//   audio.cue("exit_bank" | "exit_return" | "exit_death")   resolved chord / the same unresolved / one low note fading 1 s
//   audio.cue("level" | "unlock" | "buy")   bright arpeggios / a coin
//   audio.cue("boss_in" | "boss_break" | "boss_down")        a sub drop · a crack · a falling boom
//   audio.cue("click" | "edit" | "verdict" | "fold" | "scene" | "scene_end", { up })   the chrome
//   audio.drone("fens" | null)          the camp pad on / off
//   audio.bed("fens" | null)            the watch's ambience bed on / off (biome by biome, a crossfade between)
//
// Autoplay policy: the context is created on the first user gesture (`unlock`, wired to pointerdown/keydown once); cues
// before that are dropped. `mute` is persisted (`riddle.mute`); nothing schedules while muted. Dev: `window.__audio`.
// Measurement (docs/JUICE.md §6): `renderCue` / `renderBed` render into an OfflineAudioContext through the same master chain.

export type CueName = "hit" | "strike" | "slay" | "rule" | "telegraph" | "exit_bank" | "exit_return" | "exit_death" | "level" | "unlock" | "buy"
  | "boss_in" | "boss_break" | "boss_down" | "click" | "edit" | "verdict" | "fold" | "scene" | "scene_end";
export type Family = "flesh" | "bone" | "ooze" | "metal" | "spirit";
export type CueOpts = { dmg?: number; kind?: string; up?: boolean };
type Osc = { wave: OscillatorType; f: number; f1?: number; at?: number; dur: number; steps?: [number, number][] };   // steps: [f, at]
type Noise = { type: BiquadFilterType; f: number; f1?: number; q?: number; at?: number; dur: number; level: number };
/** A cue: ≤ 2 oscillators, ≤ 1 noise burst, one envelope (peak, attack, total length, decay shape). */
type Spec = { osc: Osc[]; noise?: Noise; peak: number; len: number; att?: number; exp?: boolean };

const MUTE_KEY = "riddle.mute";
export const CUE_MAX_S = 0.2;        // every cue ends within this (the gate's bar) …
export const DEATH_FADE_S = 1.0;     // … except the death note, which fades over 1 s (Cut 10 §4)
const DRONE_GAIN = 0.02, DRONE_RAMP_S = 1.2;
const BUS_GAIN = 0.85, BED_GAIN = 0.16, BED_FADE_S = 1.4;
/** Pentatonic roots by biome (A minor pentatonic: A · D · E), the camp pad's root. */
const DRONE_ROOT: Record<string, number> = { warrens: 110, burrows: 123.47, fens: 146.83, crypt: 164.81, foundry: 98, deep: 82.41, sanctum: 196 };
/** Cut 7 bands (crates/riddle-core/src/descent.rs): D1–4 warrens, D5–8 burrows (Cut 16 §3), D9–13 fens, D14+ crypt. */
export const biomeOf = (depth: number): string => (depth <= 4 ? "warrens" : depth <= 8 ? "burrows" : depth <= 13 ? "fens" : "crypt");

/** The sound a foe's body makes (render/fx.ts's particle families, the same kinds). */
export function familyOf(kind = ""): Family {
  if (/skeleton|bone|lich/.test(kind)) return "bone";
  if (/golem|sentinel|bell|smith|foundry|iron|slag|forge|warden/.test(kind)) return "metal";
  if (/bloat|jelly|slime|eel/.test(kind)) return "ooze";
  if (/wraith|spectral|shade|siren|echo|mirror|acolyte|ghost/.test(kind)) return "spirit";
  return "flesh";
}

// ---- the cue book --------------------------------------------------------------------------------------------------------------
type Rng = () => number;
/** ±j (a fraction) around 1 */
const jit = (r: Rng, j: number): number => 1 + (r() * 2 - 1) * j;

/** a blow landing on a body of `fam` (the hero's strike, or the attacker's flavour over the hero's thud) */
function impact(fam: Family, dmg: number, r: Rng): Spec {
  const d = Math.max(1, Math.min(24, dmg)), heavy = d / 24, p = jit(r, 0.09), fq = jit(r, 0.2), L = jit(r, 0.12);
  switch (fam) {
    case "bone": return { osc: [{ wave: "square", f: 640 * p, f1: 470 * p, dur: 0.05 * L }, { wave: "triangle", f: 1280 * p * jit(r, 0.05), dur: 0.035 }],
      noise: { type: "bandpass", f: 2600 * fq, q: 2.5, dur: 0.06 * L, level: 0.9 }, peak: 0.14 + 0.06 * heavy, len: 0.09 * L };
    case "ooze": return { osc: [{ wave: "sine", f: 200 * p, dur: 0.15 * L, steps: [[310 * p, 0.03], [120 * p, 0.07]] }],
      noise: { type: "lowpass", f: 450 * fq, f1: 1600 * fq, q: 4, dur: 0.12 * L, level: 0.7 }, peak: 0.13 + 0.06 * heavy, len: 0.16 * L };
    case "metal": { const f = 440 * p; return { osc: [{ wave: "square", f, dur: 0.18 }, { wave: "triangle", f: f * 2.76 * jit(r, 0.02), dur: 0.16 }],
      noise: { type: "highpass", f: 3200 * fq, dur: 0.04, level: 0.6 }, peak: 0.15 + 0.05 * heavy, len: 0.19, exp: true }; }
    case "spirit": return { osc: [{ wave: "sine", f: 720 * p, f1: 340 * p, dur: 0.17 * L }, { wave: "sine", f: 1080 * p, f1: 520 * p, dur: 0.14 * L }],
      noise: { type: "bandpass", f: 1900 * fq, f1: 900 * fq, q: 1.2, dur: 0.16 * L, level: 0.45 }, peak: 0.08 + 0.05 * heavy, len: 0.18 * L, att: 0.015 };
    default: { const f = (130 - 60 * heavy) * p; return { osc: [{ wave: "triangle", f: f * 1.4, f1: f * 0.55, dur: 0.11 * L }, { wave: "sine", f: f * 2.1, f1: f, dur: 0.05 }],
      noise: { type: "lowpass", f: 1100 * fq, f1: 300, q: 1, dur: 0.07 * L, level: 0.85 }, peak: 0.14 + 0.08 * heavy, len: 0.12 * L }; }
  }
}
/** `hit` (the hero hurt): a low thud whose pitch falls with the damage (1 hp ≈ 150 Hz, 20+ hp ≈ 55 Hz), the attacker's family on top */
function hitSpec(dmg: number, fam: Family, r: Rng): Spec {
  const d = Math.max(1, Math.min(24, dmg)), f = (150 - (95 * (d - 1)) / 23) * jit(r, 0.05), top = impact(fam, dmg, r);
  const flavour: Osc = fam === "flesh" ? { wave: "square", f: f * 4 * jit(r, 0.08), dur: 0.02 } : { ...top.osc[0]!, dur: Math.min(top.osc[0]!.dur, 0.1) };
  return { osc: [{ wave: "triangle", f, f1: f * 0.7, dur: 0.12 }, flavour], noise: top.noise ? { ...top.noise, level: top.noise.level * 0.8 } : undefined,
    peak: 0.16 + 0.1 * (d / 24), len: 0.13 };
}
function slaySpec(fam: Family, r: Rng): Spec {
  const p = jit(r, 0.06), fq = jit(r, 0.2);
  switch (fam) {
    case "bone": return { osc: [{ wave: "square", f: 900 * p, dur: 0.16, steps: [[640 * p, 0.03], [1100 * p, 0.06], [560 * p, 0.09], [820 * p, 0.12]] }],
      noise: { type: "bandpass", f: 2200 * fq, q: 1.5, dur: 0.16, level: 0.8 }, peak: 0.1, len: 0.17 };
    case "ooze": return { osc: [{ wave: "sine", f: 320 * p, f1: 55 * p, dur: 0.18 }], noise: { type: "lowpass", f: 1800 * fq, f1: 200, q: 6, dur: 0.16, level: 0.8 }, peak: 0.16, len: 0.19 };
    case "metal": return { osc: [{ wave: "square", f: 330 * p, f1: 165 * p, dur: 0.19 }, { wave: "triangle", f: 910 * p, dur: 0.19 }], noise: { type: "highpass", f: 2500 * fq, dur: 0.05, level: 0.7 }, peak: 0.17, len: 0.2, exp: true };
    case "spirit": return { osc: [{ wave: "sine", f: 660 * p, f1: 1320 * p, dur: 0.19 }, { wave: "sine", f: 990 * p, f1: 1980 * p, dur: 0.17 }], noise: { type: "highpass", f: 3000 * fq, dur: 0.18, level: 0.3 }, peak: 0.07, len: 0.2, att: 0.03 };
    default: return { osc: [{ wave: "square", f: 392 * p, dur: 0.18, steps: [[262 * p, 0.07]] }, { wave: "triangle", f: 196 * p, dur: 0.18 }], noise: { type: "lowpass", f: 900 * fq, dur: 0.06, level: 0.5 }, peak: 0.16, len: 0.18 };
  }
}
const arp = (fs: number[], wave: OscillatorType, top: number, peak: number): Spec => ({
  osc: [{ wave, f: fs[0]!, dur: 0.05 * fs.length, steps: fs.slice(1).map((f, i) => [f, 0.05 * (i + 1)] as [number, number]) }, { wave: "sine", f: top, at: 0.05 * (fs.length - 1), dur: 0.05 }], peak, len: 0.2 });

function specOf(name: CueName, o: CueOpts, r: Rng): Spec {
  const fam = familyOf(o.kind);
  switch (name) {
    case "hit": return hitSpec(o.dmg ?? 4, fam, r);
    case "strike": return impact(fam, o.dmg ?? 4, r);
    case "slay": return slaySpec(fam, r);
    case "rule": { const p = jit(r, 0.03); return { osc: [{ wave: "square", f: 1320 * p, dur: 0.03 }, { wave: "sine", f: 2640 * p, dur: 0.02 }], peak: 0.07, len: 0.04 }; }
    case "telegraph": { const p = jit(r, 0.02); return { osc: [{ wave: "triangle", f: 523 * p, dur: 0.18, steps: [[784 * p, 0.08]] }, { wave: "sine", f: 1046 * p, at: 0.08, dur: 0.1 }], peak: 0.14, len: 0.18 }; }
    case "exit_bank": return { osc: [{ wave: "triangle", f: 262, dur: 0.2 }, { wave: "triangle", f: 392, at: 0.04, dur: 0.16 }], peak: 0.2, len: 0.2 };      // C · G: resolved
    case "exit_return": return { osc: [{ wave: "triangle", f: 262, dur: 0.2 }, { wave: "triangle", f: 370, at: 0.04, dur: 0.16 }], peak: 0.18, len: 0.2 };    // C · F#: unresolved
    case "exit_death": return { osc: [{ wave: "triangle", f: 65.4, dur: DEATH_FADE_S }], peak: 0.24, len: DEATH_FADE_S, exp: true };                    // one low C, fading
    case "level": return arp([523, 659, 784, 1046], "square", 2093, 0.12);
    case "unlock": return arp([659, 784, 988, 1319], "square", 2637, 0.12);
    case "buy": { const p = jit(r, 0.02); return { osc: [{ wave: "square", f: 1319 * p, dur: 0.12, steps: [[1760 * p, 0.045]] }, { wave: "triangle", f: 2637 * p, at: 0.045, dur: 0.07 }], noise: { type: "highpass", f: 5000, dur: 0.03, level: 0.25 }, peak: 0.08, len: 0.13 }; }
    // boss moments: a stronger beat inside the same 200 ms — a sub drop under a saw, a crack of steel, a falling boom
    case "boss_in": return { osc: [{ wave: "square", f: 62, f1: 38, dur: 0.2 }, { wave: "sawtooth", f: 124, f1: 70, dur: 0.2 }], noise: { type: "lowpass", f: 700, f1: 120, q: 2, dur: 0.2, level: 1 }, peak: 0.3, len: 0.2, att: 0.004 };
    case "boss_break": return { osc: [{ wave: "square", f: 1500 * jit(r, 0.04), f1: 620, dur: 0.12 }, { wave: "triangle", f: 150, f1: 60, dur: 0.18 }], noise: { type: "highpass", f: 1800, f1: 5000, dur: 0.14, level: 1 }, peak: 0.26, len: 0.19 };
    case "boss_down": return { osc: [{ wave: "triangle", f: 98, f1: 41, dur: 0.2 }, { wave: "square", f: 196, dur: 0.2, steps: [[147, 0.07], [110, 0.14]] }], noise: { type: "lowpass", f: 1400, f1: 150, q: 1, dur: 0.2, level: 1 }, peak: 0.3, len: 0.2, exp: true };
    // the chrome: quiet, short, varied
    case "click": { const p = jit(r, 0.12); return { osc: [{ wave: "sine", f: 1700 * p, f1: 1100 * p, dur: 0.018 }], noise: { type: "highpass", f: 4200 * jit(r, 0.2), dur: 0.012, level: 0.35 }, peak: 0.1, len: 0.025 }; }
    case "edit": { const p = jit(r, 0.08); return { osc: [{ wave: "triangle", f: 520 * p, f1: 430 * p, dur: 0.035 }], noise: { type: "bandpass", f: 1500 * jit(r, 0.2), q: 2, dur: 0.03, level: 0.7 }, peak: 0.12, len: 0.045 }; }
    case "verdict": return { osc: [{ wave: "triangle", f: 92, f1: 48, dur: 0.16 }, { wave: "square", f: 184, f1: 92, dur: 0.05 }], noise: { type: "lowpass", f: 800, f1: 160, q: 1, dur: 0.1, level: 1 }, peak: 0.22, len: 0.17 };
    case "fold": return { osc: [{ wave: "sine", f: 660, f1: 990, at: 0.06, dur: 0.12 }], noise: { type: "bandpass", f: 400, f1: 2600, q: 1.4, dur: 0.18, level: 0.8 }, peak: 0.07, len: 0.2, att: 0.03 };
    case "scene": return { osc: [{ wave: "triangle", f: 440, f1: 330, dur: 0.16 }], noise: { type: "bandpass", f: 2600, f1: 450, q: 1.4, dur: 0.18, level: 0.8 }, peak: 0.07, len: 0.2, att: 0.03 };
    case "scene_end": return o.up
      ? { osc: [{ wave: "triangle", f: 523, dur: 0.2 }, { wave: "triangle", f: 784, at: 0.03, dur: 0.17 }], peak: 0.09, len: 0.2 }
      : { osc: [{ wave: "triangle", f: 523, dur: 0.2 }, { wave: "triangle", f: 494, at: 0.03, dur: 0.17 }], peak: 0.09, len: 0.2 };
  }
}

// ---- the synth (any context: live or offline) ----------------------------------------------------------------------------------
const noiseBufs = new WeakMap<BaseAudioContext, Map<number, AudioBuffer>>();
/** 0.5 s of white noise at `level` (quantised to 0.05; one buffer per level per context — the envelope is the cue's one gain). */
function noiseBuf(ctx: BaseAudioContext, level: number): AudioBuffer {
  const q = Math.max(1, Math.min(20, Math.round(level * 20)));
  let m = noiseBufs.get(ctx); if (!m) { m = new Map(); noiseBufs.set(ctx, m); }
  let b = m.get(q);
  if (!b) {
    b = ctx.createBuffer(1, Math.floor(ctx.sampleRate * 0.5), ctx.sampleRate);
    const d = b.getChannelData(0); let s = 0x9e3779b9 ^ q;
    for (let i = 0; i < d.length; i++) { s ^= s << 13; s ^= s >>> 17; s ^= s << 5; d[i] = (((s >>> 0) / 4294967296) * 2 - 1) * (q / 20); }
    m.set(q, b);
  }
  return b;
}
function schedule(ctx: BaseAudioContext, dest: AudioNode, t0: number, sp: Spec, r: Rng): void {
  const env = ctx.createGain(), att = sp.att ?? 0.004;
  env.gain.setValueAtTime(0, t0);
  env.gain.linearRampToValueAtTime(sp.peak, t0 + att);
  if (sp.exp) env.gain.exponentialRampToValueAtTime(0.0005, t0 + sp.len);
  else env.gain.setTargetAtTime(0, t0 + Math.max(att + 0.001, sp.len * 0.35), Math.max(0.006, sp.len * 0.16));
  env.connect(dest);
  for (const v of sp.osc.slice(0, 2)) {
    const osc = ctx.createOscillator(), at = t0 + (v.at ?? 0), end = t0 + Math.min(sp.len, (v.at ?? 0) + v.dur);
    osc.type = v.wave;
    osc.frequency.setValueAtTime(v.f, at);
    for (const [f, a] of v.steps ?? []) osc.frequency.setValueAtTime(f, at + a);
    if (v.f1) osc.frequency.exponentialRampToValueAtTime(v.f1, end);
    osc.connect(env); osc.start(at); osc.stop(end);
  }
  const n = sp.noise;
  if (n) {
    const src = ctx.createBufferSource(), flt = ctx.createBiquadFilter(), at = t0 + (n.at ?? 0), end = t0 + Math.min(sp.len, (n.at ?? 0) + n.dur);
    src.buffer = noiseBuf(ctx, n.level);
    flt.type = n.type; flt.Q.value = n.q ?? 0.8;
    flt.frequency.setValueAtTime(n.f, at);
    if (n.f1) flt.frequency.exponentialRampToValueAtTime(n.f1, end);
    src.connect(flt); flt.connect(env);
    src.start(at, r() * 0.25); src.stop(end);   // a fresh slice of the noise every time
  }
}
/** The master chain: a bus gain into a soft limiter — exactly linear up to LIM_KNEE (−3 dBFS), then a tanh shoulder that never
 *  passes full scale. (A DynamicsCompressorNode was measured first: its detector clamped every cue's onset by 6–7 dB and added
 *  1.7 dB of makeup gain — it flattened the transients the hits are made of. docs/JUICE.md §6.) */
const LIM_KNEE = 0.708;
let limCurve: Float32Array | null = null;
function limiterCurve(): Float32Array {
  if (limCurve) return limCurve;
  const n = 4097, c = new Float32Array(n), room = 1 - LIM_KNEE;
  for (let i = 0; i < n; i++) {
    const x = (i / (n - 1)) * 2 - 1, a = Math.abs(x);   // a WaveShaper maps its input's ±1 onto the curve (beyond it: the ends, 0.93)
    c[i] = Math.sign(x) * (a <= LIM_KNEE ? a : LIM_KNEE + room * Math.tanh((a - LIM_KNEE) / room));
  }
  return (limCurve = c);
}
function masterChain(ctx: BaseAudioContext): GainNode {
  const bus = ctx.createGain(); bus.gain.value = BUS_GAIN;
  const lim = ctx.createWaveShaper(); lim.curve = limiterCurve() as Float32Array<ArrayBuffer>; lim.oversample = "2x";
  bus.connect(lim); lim.connect(ctx.destination);
  return bus;
}
/** A signature of a cue's jittered parameters (dev log: two consecutive hits never share one). */
const sig = (sp: Spec): string => [...sp.osc.map((o) => `${o.wave}${o.f.toFixed(1)}`), sp.noise ? `n${sp.noise.f.toFixed(0)}` : "", sp.len.toFixed(3)].join("·");

// ---- ambience beds -----------------------------------------------------------------------------------------------------------
type BedDef = { lp: number; hp: number; rumble: number; air: number; tone: [number, number][]; shots: ShotKind[]; rate: number; level: number };
type ShotKind = "drip" | "clank" | "crackle" | "bell" | "skitter" | "bubble" | "chime" | "wind";
/** per biome: the texture's colour (one-pole low / high cuts, a sub rumble, airy hiss), a faint tone bed (freq, level), the one-shots and their rate (per second) */
const BEDS: Record<string, BedDef> = {
  warrens: { lp: 500, hp: 40, rumble: 0.6, air: 0.08, tone: [], shots: ["drip", "skitter"], rate: 0.25, level: 1 },
  burrows: { lp: 380, hp: 30, rumble: 0.8, air: 0.05, tone: [], shots: ["skitter", "skitter", "drip"], rate: 0.35, level: 1 },
  fens: { lp: 900, hp: 120, rumble: 0.3, air: 0.25, tone: [], shots: ["bubble", "drip", "drip", "bubble"], rate: 0.5, level: 0.9 },
  crypt: { lp: 650, hp: 90, rumble: 0.35, air: 0.18, tone: [[55, 0.05], [82.4, 0.03]], shots: ["wind", "bell"], rate: 0.12, level: 0.9 },
  foundry: { lp: 300, hp: 25, rumble: 1, air: 0.1, tone: [[49, 0.06]], shots: ["clank", "crackle", "crackle", "clank"], rate: 0.6, level: 1 },
  deep: { lp: 220, hp: 20, rumble: 1, air: 0.03, tone: [[36.7, 0.07]], shots: ["drip"], rate: 0.18, level: 1 },
  sanctum: { lp: 2400, hp: 300, rumble: 0.1, air: 0.35, tone: [[392, 0.012], [587.3, 0.008], [784, 0.006]], shots: ["chime"], rate: 0.1, level: 0.8 },
};
const BED_LOOP_S = 6;
function mulberry(seed: number): Rng { let a = seed >>> 0; return () => { a |= 0; a = (a + 0x6d2b79f5) | 0; let t = Math.imul(a ^ (a >>> 15), 1 | a); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; }; }
/** the bed's texture: BED_LOOP_S of stereo filtered noise (+ a faint tone), seamless (the tail crossfaded into the head), RMS-normalised */
function bedTexture(ctx: BaseAudioContext, biome: string): AudioBuffer {
  const def = BEDS[biome] ?? BEDS.warrens!, sr = ctx.sampleRate, n = Math.floor(sr * BED_LOOP_S), x = Math.floor(sr * 0.5);
  const buf = ctx.createBuffer(2, n, sr);
  for (let ch = 0; ch < 2; ch++) {
    const r = mulberry(biome.length * 131 + ch * 7 + 1), raw = new Float32Array(n + x);
    const a = Math.exp((-2 * Math.PI * def.lp) / sr), b = Math.exp((-2 * Math.PI * def.hp) / sr), c = Math.exp((-2 * Math.PI * 70) / sr);
    let lp0 = 0, lp = 0, hpIn = 0, hp = 0, sub = 0, lfo = r() * 6.28;
    for (let i = 0; i < n + x; i++) {
      const w = r() * 2 - 1;
      lp0 = a * lp0 + (1 - a) * w; lp = a * lp + (1 - a) * lp0;   // the body (two poles: a dark bed, not a hiss)
      hp = b * (hp + lp - hpIn); hpIn = lp;         // minus the lowest mud
      sub = c * sub + (1 - c) * (r() * 2 - 1);     // a sub rumble
      const swell = 0.75 + 0.25 * Math.sin(lfo + (i / sr) * 2 * Math.PI * (1 / BED_LOOP_S));   // one breath a loop (seamless)
      let v = hp * swell + sub * def.rumble * 3 + w * def.air * 0.03;
      for (const [f, l] of def.tone) v += l * Math.sin((2 * Math.PI * Math.round(f * BED_LOOP_S) * i) / n) * (0.8 + 0.2 * Math.sin((2 * Math.PI * i) / n + ch));
      raw[i] = v;
    }
    const d = buf.getChannelData(ch);
    for (let i = 0; i < n; i++) d[i] = raw[i]!;
    for (let i = 0; i < x; i++) { const t = i / x; d[i] = raw[n + i]! * Math.cos(t * Math.PI / 2) + raw[i]! * Math.sin(t * Math.PI / 2); }   // equal-power seam
    let ss = 0; for (let i = 0; i < n; i++) ss += d[i]! * d[i]!;
    const k = (0.05 * def.level) / Math.max(1e-6, Math.sqrt(ss / n));
    for (let i = 0; i < n; i++) d[i] = d[i]! * k;
  }
  return buf;
}
/** one ambience one-shot, synthesised to a buffer (never an oscillator node): its own pitch, length and level each time */
function shotBuffer(ctx: BaseAudioContext, kind: ShotKind, r: Rng): AudioBuffer {
  const sr = ctx.sampleRate, len = kind === "bell" || kind === "chime" ? 2.2 : kind === "wind" ? 2.6 : kind === "skitter" || kind === "crackle" ? 0.5 : 0.7;
  const n = Math.floor(sr * len), buf = ctx.createBuffer(1, n, sr), d = buf.getChannelData(0);
  const p = 0.8 + r() * 0.45;
  for (let i = 0; i < n; i++) {
    const t = i / sr; let v = 0;
    switch (kind) {
      case "drip": { const f = 900 * p * (1 + 1.6 * Math.min(1, t / 0.05)); v = Math.sin(2 * Math.PI * f * t) * Math.exp(-t * 38) * 0.5 + (t > 0.18 ? Math.sin(2 * Math.PI * f * 0.9 * t) * Math.exp(-(t - 0.18) * 30) * 0.15 : 0); break; }
      case "clank": { const f = 310 * p; v = (Math.sin(2 * Math.PI * f * t) + 0.6 * Math.sin(2 * Math.PI * f * 2.76 * t) + 0.3 * Math.sin(2 * Math.PI * f * 5.4 * t)) * Math.exp(-t * 9) * 0.25; break; }
      case "crackle": v = (r() < 0.012 ? (r() * 2 - 1) : 0) * Math.exp(-t * 4) * 0.8; break;
      case "skitter": v = (r() < 0.05 * (1 - t * 1.5) ? (r() * 2 - 1) * 0.4 : 0); break;
      case "bubble": { const f = 260 * p * (1 + 2.5 * t); v = Math.sin(2 * Math.PI * f * t) * Math.exp(-t * 14) * 0.4 * Math.min(1, t * 80); break; }
      case "bell": { const f = 164.8 * p; v = (Math.sin(2 * Math.PI * f * t) + 0.5 * Math.sin(2 * Math.PI * f * 2.4 * t) + 0.25 * Math.sin(2 * Math.PI * f * 5.95 * t)) * Math.exp(-t * 1.6) * 0.22; break; }
      case "chime": { const f = [784, 988, 1175, 1319][Math.floor(p * 7) % 4]!; v = (Math.sin(2 * Math.PI * f * t) + 0.3 * Math.sin(2 * Math.PI * f * 3 * t)) * Math.exp(-t * 2.2) * 0.12 * Math.min(1, t * 200); break; }
      case "wind": v = (r() * 2 - 1) * Math.sin(Math.PI * t / len) ** 2 * 0.12; break;
    }
    d[i] = v;
  }
  if (kind === "wind") { let lp = 0; const a = Math.exp((-2 * Math.PI * (500 * p)) / sr); for (let i = 0; i < n; i++) { lp = a * lp + (1 - a) * d[i]!; d[i] = lp * 3; } }
  return buf;
}

type Drone = { a: OscillatorNode; b: OscillatorNode; g: GainNode; biome: string };
type Bed = { src: AudioBufferSourceNode; g: GainNode; biome: string; timer: number };

/** Cut 24 §5: at most BURST_N cues in one task (a skip's released backlog is one sound, not fifteen stacked). */
const BURST_N = 3;
/** the chrome's cues (clicks) never take a burst slot; the same one at most once per CLICK_GAP_S (a double-fired tap is one click) */
const CHROME: ReadonlySet<CueName> = new Set(["click", "edit"]);
const CLICK_GAP_S = 0.035;
const DUCK = new Set<string>(["boss_in", "boss_break", "boss_down", "verdict", "exit_death", "exit_bank", "exit_return"]);
const DUCK_DB = 9;
class Audio {
  private ctx: AudioContext | null = null;
  private bus: GainNode | null = null;
  private mutedFlag = readMute();
  private drone_: Drone | null = null;
  private wantDrone: string | null = null;
  private bed_: Bed | null = null;
  private wantBed: string | null = null;
  private textures = new Map<string, AudioBuffer>();
  private armed = false;
  private rng: Rng = Math.random;
  private lastChrome: { name: CueName | ""; at: number } = { name: "", at: -1 };
  /** Dev: the cues scheduled so far (name, seconds, the jittered signature), for tests that assert on events (the spy sees the nodes). */
  readonly log: { cue: CueName; at: number; v: string }[] = [];
  private burst = 0;   // cues played in the current task (reset on its microtask checkpoint)

  get muted(): boolean { return this.mutedFlag; }
  get unlocked(): boolean { return !!this.ctx; }
  setMuted(m: boolean): void {
    this.mutedFlag = m;
    try { localStorage.setItem(MUTE_KEY, m ? "1" : "0"); } catch { /* private mode: not persisted */ }
    if (m) { this.stopDrone(); this.stopBed(); } else { this.applyDrone(); this.applyBed(); }
  }
  /** Wire the first-gesture unlock (idempotent). */
  arm(): void {
    if (this.armed) return; this.armed = true;
    const once = (): void => { this.unlock(); document.removeEventListener("pointerdown", once, true); document.removeEventListener("keydown", once, true); };
    document.addEventListener("pointerdown", once, true); document.addEventListener("keydown", once, true);
  }
  /** Create (or resume) the context; safe to call from a gesture handler. */
  unlock(): void {
    if (!this.ctx) {
      const AC = (window as unknown as { AudioContext?: new () => AudioContext; webkitAudioContext?: new () => AudioContext }).AudioContext ?? (window as unknown as { webkitAudioContext?: new () => AudioContext }).webkitAudioContext;
      if (!AC) return;
      try { this.ctx = new AC(); this.bus = masterChain(this.ctx); } catch { this.ctx = null; return; }
    }
    if (this.ctx.state === "suspended") void this.ctx.resume().catch(() => { /* still locked: the next gesture tries again */ });
    this.applyDrone(); this.applyBed();
  }

  /** Play a cue now. Dropped while muted or before the first gesture. */
  cue(name: CueName, opts: CueOpts = {}): boolean {
    if (this.mutedFlag || !this.ctx || !this.bus) return false;
    const ctx = this.ctx, t0 = ctx.currentTime;
    if (CHROME.has(name)) { const now = performance.now() / 1000; if (this.lastChrome.name === name && now - this.lastChrome.at < CLICK_GAP_S) return false; this.lastChrome = { name, at: now }; }
    else {
      // Cut 24 §5 (AK: "a burst of ~15 cues at one timestamp at a run's end"): a picture that lands on the ending releases its queued
      // events in one task — past BURST_N cues in one task the rest of the burst is dropped (the exit's own cue always plays)
      if (!this.burst) { this.burst = 0; queueMicrotask(() => { this.burst = 0; }); }
      if (!name.startsWith("exit_") && this.burst >= BURST_N) return false;
      this.burst++;
    }
    const sp = specOf(name, opts, this.rng);
    schedule(ctx, this.bus, t0, sp, this.rng);
    if (DUCK.has(name)) this.duck(t0);
    this.log.push({ cue: name, at: t0, v: sig(sp) });
    if (this.log.length > 2000) this.log.splice(0, 1000);
    return true;
  }

  /** gfx round 2 (docs/JUICE.md §10): a big moment (a boss's entrance, break or fall, the verdict, a run's end) ducks the ambience bed
   *  under it — down DUCK_DB in 40 ms, held, back over ~0.9 s — so the moment owns the room. `ducks` counts them (dev / tests). */
  ducks = 0;
  private duck(t0: number): void {
    const b = this.bed_; if (!b) return;
    this.ducks++;
    const g = b.g.gain, low = BED_GAIN * Math.pow(10, -DUCK_DB / 20);
    g.cancelScheduledValues(t0); g.setValueAtTime(g.value, t0); g.linearRampToValueAtTime(low, t0 + 0.04);
    g.setValueAtTime(low, t0 + 0.65); g.linearRampToValueAtTime(BED_GAIN, t0 + 1.55);
  }

  /** The camp pad: on with a biome (root by biome), off with null. */
  drone(biome: string | null): void { this.wantDrone = biome; this.applyDrone(); }
  private applyDrone(): void {
    const want = this.mutedFlag ? null : this.wantDrone;
    if (!want) { this.stopDrone(); return; }
    if (!this.ctx || !this.bus) return;
    if (this.drone_ && this.drone_.biome === want) return;
    this.stopDrone();
    const ctx = this.ctx, t0 = ctx.currentTime, root = DRONE_ROOT[want] ?? DRONE_ROOT.warrens!;
    const g = ctx.createGain(); g.gain.setValueAtTime(0, t0); g.gain.linearRampToValueAtTime(DRONE_GAIN, t0 + DRONE_RAMP_S); g.connect(this.bus);
    const a = ctx.createOscillator(); a.type = "triangle"; a.frequency.setValueAtTime(root, t0);
    const b = ctx.createOscillator(); b.type = "sine"; b.frequency.setValueAtTime(root * 1.5 + 0.7, t0);   // a fifth, slightly detuned: a slow beat
    a.connect(g); b.connect(g); a.start(t0); b.start(t0);
    this.drone_ = { a, b, g, biome: want };
  }
  private stopDrone(): void {
    const d = this.drone_; if (!d || !this.ctx) { this.drone_ = null; return; }
    this.drone_ = null;
    const t = this.ctx.currentTime;
    d.g.gain.cancelScheduledValues(t); d.g.gain.setValueAtTime(d.g.gain.value, t); d.g.gain.linearRampToValueAtTime(0, t + 0.5);
    d.a.stop(t + 0.55); d.b.stop(t + 0.55);
  }
  /** Dev / tests: is the pad up, and for which biome. */
  droneState(): string | null { return this.drone_?.biome ?? null; }

  /** The watch's ambience: a biome's bed on (a crossfade from the last one), off with null. */
  bed(biome: string | null): void { this.wantBed = biome; this.applyBed(); }
  private applyBed(): void {
    const want = this.mutedFlag ? null : this.wantBed;
    if (!want) { this.stopBed(); return; }
    if (!this.ctx || !this.bus) return;
    if (this.bed_ && this.bed_.biome === want) return;
    this.stopBed();
    const ctx = this.ctx, t0 = ctx.currentTime, key = BEDS[want] ? want : "warrens";
    let tex = this.textures.get(key);
    if (!tex) { tex = bedTexture(ctx, key); this.textures.set(key, tex); }
    const g = ctx.createGain(); g.gain.setValueAtTime(0, t0); g.gain.linearRampToValueAtTime(BED_GAIN, t0 + BED_FADE_S); g.connect(this.bus);
    const src = ctx.createBufferSource(); src.buffer = tex; src.loop = true; src.connect(g); src.start(t0, this.rng() * BED_LOOP_S);
    const bed: Bed = { src, g, biome: want, timer: 0 };
    const def = BEDS[key]!;
    const next = (): void => {
      if (this.bed_ !== bed || !this.ctx) return;
      if (this.ctx.state === "running" && !document.hidden) this.shot(def.shots[Math.floor(this.rng() * def.shots.length)]!, g);
      bed.timer = window.setTimeout(next, (-Math.log(1 - this.rng() * 0.95) / def.rate) * 1000);   // a Poisson clock: no rhythm to tire of
    };
    bed.timer = window.setTimeout(next, 800 + this.rng() * 1500);
    this.bed_ = bed;
  }
  private shot(kind: ShotKind, dest: AudioNode): void {
    const ctx = this.ctx!, t = ctx.currentTime, src = ctx.createBufferSource();
    src.buffer = shotBuffer(ctx, kind, this.rng);
    if (typeof ctx.createStereoPanner === "function") { const p = ctx.createStereoPanner(); p.pan.value = this.rng() * 1.6 - 0.8; src.connect(p); p.connect(dest); }
    else src.connect(dest);
    src.start(t);
  }
  private stopBed(): void {
    const b = this.bed_; if (!b || !this.ctx) { this.bed_ = null; return; }
    this.bed_ = null; clearTimeout(b.timer);
    const t = this.ctx.currentTime;
    b.g.gain.cancelScheduledValues(t); b.g.gain.setValueAtTime(b.g.gain.value, t); b.g.gain.linearRampToValueAtTime(0, t + 0.8);
    b.src.stop(t + 0.85);
  }
  /** Dev / tests: is a bed up, and for which biome. */
  bedState(): string | null { return this.bed_?.biome ?? null; }
}

function readMute(): boolean { try { return localStorage.getItem(MUTE_KEY) === "1"; } catch { return false; } }

export const audio = new Audio();

// ---- measurement (docs/JUICE.md §6: rendered offline, through the same master chain) -------------------------------------------
/** Render one cue into an OfflineAudioContext (mono, `sr`), `pre` = before the limiter. Seeded so a run is reproducible. */
export async function renderCue(name: CueName, opts: CueOpts = {}, seed = 1, sr = 48000, pre = false): Promise<Float32Array> {
  const len = name === "exit_death" ? 1.3 : 0.4;
  const ctx = new OfflineAudioContext(1, Math.floor(sr * len), sr), r = mulberry(seed);
  const dest = pre ? ctx.destination : masterChain(ctx);
  if (pre) { /* the raw instrument × the bus gain, no limiter */ const g = ctx.createGain(); g.gain.value = BUS_GAIN; g.connect(ctx.destination); schedule(ctx, g, 0.01, specOf(name, opts, r), r); }
  else schedule(ctx, dest, 0.01, specOf(name, opts, r), r);
  return (await ctx.startRendering()).getChannelData(0).slice();
}
/** Render `seconds` of a biome's bed (the texture plus its one-shots at their rate, seeded), left channel. */
export async function renderBed(biome: string, seconds = 12, seed = 1, sr = 48000): Promise<Float32Array> {
  const ctx = new OfflineAudioContext(2, Math.floor(sr * seconds), sr), r = mulberry(seed), bus = masterChain(ctx);
  const g = ctx.createGain(); g.gain.value = BED_GAIN; g.connect(bus);
  const src = ctx.createBufferSource(); src.buffer = bedTexture(ctx, biome); src.loop = true; src.connect(g); src.start(0);
  const def = BEDS[biome] ?? BEDS.warrens!;
  for (let t = 0.8 + r() * 1.5; t < seconds - 0.5; t += -Math.log(1 - r() * 0.95) / def.rate) {
    const s = ctx.createBufferSource(); s.buffer = shotBuffer(ctx, def.shots[Math.floor(r() * def.shots.length)]!, r); s.connect(g); s.start(t);
  }
  return (await ctx.startRendering()).getChannelData(0).slice();
}
/** The cue book's names (measurement enumerates it). */
export const CUE_NAMES: CueName[] = ["hit", "strike", "slay", "rule", "telegraph", "exit_bank", "exit_return", "exit_death", "level", "unlock", "buy",
  "boss_in", "boss_break", "boss_down", "click", "edit", "verdict", "fold", "scene", "scene_end"];
export const BED_BIOMES = Object.keys(BEDS);
