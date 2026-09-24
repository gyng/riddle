// Cut 10 §4 — sound. Sparse, synthesised in-browser (WebAudio, no assets, no loading). Every cue is one instrument: two
// oscillators through one gain envelope, ≤ 200 ms (the death note is the one exception: a single low note fading over 1 s).
// A camp drone: a very quiet two-note pad (root + fifth) whose pentatonic root follows the biome of the next floor.
//
//   audio.cue("hit", { dmg })   hero hurt: a low thud, pitch by damage
//   audio.cue("slay")           a foe dies: a short descending pair
//   audio.cue("rule")           a player row fires (not a chore): a soft tick
//   audio.cue("telegraph")      a rising two-note warning
//   audio.cue("exit_bank" | "exit_return" | "exit_death")   resolved chord / the same unresolved / one low note fading 1 s
//   audio.cue("level" | "unlock")   a bright arpeggio
//   audio.drone("fens" | null)  the camp pad on / off
//
// Autoplay policy: the context is created on the first user gesture (`unlock`, wired to pointerdown/keydown once); cues
// before that are dropped. `mute` is persisted (`riddle.mute`); nothing schedules while muted. Dev: `window.__audio`.

export type CueName = "hit" | "slay" | "rule" | "telegraph" | "exit_bank" | "exit_return" | "exit_death" | "level" | "unlock";
type Note = { f: number; at: number; dur: number; wave?: OscillatorType };
/** A cue: up to two oscillator voices (each a list of notes played back to back) and one envelope (peak, total length). */
type CueSpec = { voices: Note[][]; peak: number; len: number };

const MUTE_KEY = "riddle.mute";
export const CUE_MAX_S = 0.2;        // every cue ends within this (the gate's bar) …
export const DEATH_FADE_S = 1.0;     // … except the death note, which fades over 1 s (Cut 10 §4)
const DRONE_GAIN = 0.02, DRONE_RAMP_S = 1.2;
/** Pentatonic roots by biome (A minor pentatonic: A · D · E), the camp pad's root. */
const DRONE_ROOT: Record<string, number> = { warrens: 110, burrows: 123.47, fens: 146.83, crypt: 164.81 };
/** Cut 7 bands (crates/riddle-core/src/descent.rs): D1–4 warrens, D5–8 burrows (Cut 16 §3), D9–13 fens, D14+ crypt. */
export const biomeOf = (depth: number): string => (depth <= 4 ? "warrens" : depth <= 8 ? "burrows" : depth <= 13 ? "fens" : "crypt");

const CUES: Record<Exclude<CueName, "hit">, CueSpec> = {
  slay: { voices: [[{ f: 392, at: 0, dur: 0.07, wave: "square" }, { f: 262, at: 0.07, dur: 0.11, wave: "square" }], [{ f: 196, at: 0, dur: 0.18, wave: "triangle" }]], peak: 0.18, len: 0.18 },
  rule: { voices: [[{ f: 1320, at: 0, dur: 0.03, wave: "square" }], [{ f: 2640, at: 0, dur: 0.02, wave: "sine" }]], peak: 0.07, len: 0.04 },
  telegraph: { voices: [[{ f: 523, at: 0, dur: 0.08, wave: "triangle" }, { f: 784, at: 0.08, dur: 0.1, wave: "triangle" }], [{ f: 1046, at: 0.08, dur: 0.1, wave: "sine" }]], peak: 0.14, len: 0.18 },
  exit_bank: { voices: [[{ f: 262, at: 0, dur: 0.2, wave: "triangle" }], [{ f: 392, at: 0.04, dur: 0.16, wave: "triangle" }]], peak: 0.2, len: 0.2 },      // C · G: resolved
  exit_return: { voices: [[{ f: 262, at: 0, dur: 0.2, wave: "triangle" }], [{ f: 370, at: 0.04, dur: 0.16, wave: "triangle" }]], peak: 0.18, len: 0.2 },    // C · F#: unresolved
  exit_death: { voices: [[{ f: 65.4, at: 0, dur: DEATH_FADE_S, wave: "triangle" }], []], peak: 0.24, len: DEATH_FADE_S },                                  // one low C, fading
  level: { voices: [[{ f: 523, at: 0, dur: 0.05, wave: "square" }, { f: 659, at: 0.05, dur: 0.05, wave: "square" }, { f: 784, at: 0.1, dur: 0.05, wave: "square" }, { f: 1046, at: 0.15, dur: 0.05, wave: "square" }], [{ f: 2093, at: 0.15, dur: 0.05, wave: "sine" }]], peak: 0.12, len: 0.2 },
  unlock: { voices: [[{ f: 659, at: 0, dur: 0.05, wave: "square" }, { f: 784, at: 0.05, dur: 0.05, wave: "square" }, { f: 988, at: 0.1, dur: 0.05, wave: "square" }, { f: 1319, at: 0.15, dur: 0.05, wave: "square" }], [{ f: 2637, at: 0.15, dur: 0.05, wave: "sine" }]], peak: 0.12, len: 0.2 },
};
/** `hit`: a low thud whose pitch falls with the damage (1 hp ≈ 150 Hz, 20+ hp ≈ 55 Hz), the second voice a click. */
function hitSpec(dmg: number): CueSpec {
  const d = Math.max(1, Math.min(24, dmg));
  const f = 150 - (95 * (d - 1)) / 23;
  return { voices: [[{ f, at: 0, dur: 0.12, wave: "triangle" }], [{ f: f * 4, at: 0, dur: 0.02, wave: "square" }]], peak: 0.16 + 0.1 * (d / 24), len: 0.12 };
}

type Ctx = AudioContext;
type Drone = { a: OscillatorNode; b: OscillatorNode; g: GainNode; biome: string };

class Audio {
  private ctx: Ctx | null = null;
  private mutedFlag = readMute();
  private drone_: Drone | null = null;
  private wantDrone: string | null = null;
  private armed = false;
  /** Dev: the cues scheduled so far (name, seconds), for tests that assert on events (the spy sees the nodes). */
  readonly log: { cue: CueName; at: number }[] = [];

  get muted(): boolean { return this.mutedFlag; }
  get unlocked(): boolean { return !!this.ctx; }
  setMuted(m: boolean): void {
    this.mutedFlag = m;
    try { localStorage.setItem(MUTE_KEY, m ? "1" : "0"); } catch { /* private mode: not persisted */ }
    if (m) this.stopDrone(); else this.applyDrone();
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
      const AC = (window as unknown as { AudioContext?: new () => Ctx; webkitAudioContext?: new () => Ctx }).AudioContext ?? (window as unknown as { webkitAudioContext?: new () => Ctx }).webkitAudioContext;
      if (!AC) return;
      try { this.ctx = new AC(); } catch { return; }
    }
    if (this.ctx.state === "suspended") void this.ctx.resume().catch(() => { /* still locked: the next gesture tries again */ });
    this.applyDrone();
  }

  /** Play a cue now. Dropped while muted or before the first gesture. */
  cue(name: CueName, opts: { dmg?: number } = {}): boolean {
    if (this.mutedFlag || !this.ctx) return false;
    const spec = name === "hit" ? hitSpec(opts.dmg ?? 4) : CUES[name];
    const ctx = this.ctx, t0 = ctx.currentTime;
    const env = ctx.createGain();
    env.gain.setValueAtTime(0, t0);
    env.gain.linearRampToValueAtTime(spec.peak, t0 + 0.005);
    if (name === "exit_death") env.gain.exponentialRampToValueAtTime(0.0005, t0 + spec.len);
    else env.gain.setTargetAtTime(0, t0 + Math.max(0.005, spec.len - 0.06), 0.02);
    env.connect(ctx.destination);
    for (const voice of spec.voices) {
      if (!voice.length) continue;
      const osc = ctx.createOscillator();
      osc.type = voice[0].wave ?? "square";
      for (const n of voice) osc.frequency.setValueAtTime(n.f, t0 + n.at);
      const end = Math.max(...voice.map((n) => n.at + n.dur));
      osc.connect(env);
      osc.start(t0);
      osc.stop(t0 + end);
    }
    this.log.push({ cue: name, at: t0 });
    return true;
  }

  /** The camp pad: on with a biome (root by biome), off with null. */
  drone(biome: string | null): void { this.wantDrone = biome; this.applyDrone(); }
  private applyDrone(): void {
    const want = this.mutedFlag ? null : this.wantDrone;
    if (!want) { this.stopDrone(); return; }
    if (!this.ctx) return;
    if (this.drone_ && this.drone_.biome === want) return;
    this.stopDrone();
    const ctx = this.ctx, t0 = ctx.currentTime, root = DRONE_ROOT[want] ?? DRONE_ROOT.warrens;
    const g = ctx.createGain(); g.gain.setValueAtTime(0, t0); g.gain.linearRampToValueAtTime(DRONE_GAIN, t0 + DRONE_RAMP_S); g.connect(ctx.destination);
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
}

function readMute(): boolean { try { return localStorage.getItem(MUTE_KEY) === "1"; } catch { return false; } }

export const audio = new Audio();
