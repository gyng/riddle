// Juice (docs/JUICE.md) — render-only feedback read off the event stream (never game truth): particles (blood, sparks, dust, embers,
// gas puffs, glints, motes), popping damage numbers, squash and stretch, the short "pop" lights a hit, a spell or a death throws on
// the light field, hit-stop and slow-mo (the viewer's clock rate), a kick shake, and the vignette's pulses. Every piece keys off
// the quality tier (quality.ts): `low` draws and changes nothing. Reduced motion keeps colour (lights, numbers in place, the
// static vignette) and drops motion (shake, hit-stop, slow-mo, squash, rising numbers, ambient particles).
//
// Particles are CPU-simulated (≤ POOL, structure of arrays) and drawn as one instanced quad layer per tag — `emit` (a = 0.625:
// they bloom) and `matte` (a = 0.5) — plus one layer for the numbers (the atlas's bitmap font, fill recoloured per instance).
import * as THREE from "three";
import { BAYER_GLSL } from "./layers";
import { EMISSIVE_TAG } from "./bloom";
import type { FieldLight } from "./light";
import type { Quality } from "./quality";
import type { Ev } from "./types";
import type { Slot } from "./atlas";

const VERT = /* glsl */ `
attribute vec3 iPos;
attribute vec3 iSize;   // w, h (world units), shape (0 square, 1 disc)
attribute vec4 iUV;
attribute vec4 iCol;    // rgb, alpha (a screen-door dissolve below 1)
varying vec2 vUv;
varying vec2 vLoc;
varying vec4 vCol;
varying float vShape;
void main() {
  vUv = vec2(mix(iUV.x, iUV.z, uv.x), mix(iUV.y, iUV.w, uv.y));
  vLoc = uv; vCol = iCol; vShape = iSize.z;
  vec2 p = iPos.xy + vec2((uv.x - 0.5) * iSize.x, uv.y * iSize.y);
  gl_Position = projectionMatrix * modelViewMatrix * vec4(p, iPos.z, 1.0);
}`;

const FRAG = /* glsl */ `
uniform sampler2D map;
uniform float uTag;
uniform float uText;    // 1: font cells — the glyph's fill takes the colour; the ring is dropped for a 1-texel shadow
uniform vec2 uTexel;    // one atlas texel in uv
varying vec2 vUv;
varying vec2 vLoc;
varying vec4 vCol;
varying float vShape;
${BAYER_GLSL}
void main() {
  vec4 c = texture2D(map, vUv);
  if (c.a < 0.5) discard;
  if (vShape > 0.5 && length(vLoc - 0.5) > 0.5) discard;
  if (vCol.a < 0.999 && vCol.a <= bayer4(gl_FragCoord.xy)) discard;
  float l = dot(c.rgb, vec3(0.299, 0.587, 0.114));
  // a ring round a 3-px glyph read as a box at pop size: a number keeps its fill and a shadow one texel down-right
  if (uText > 0.5 && l <= 0.4) {
    float sh = dot(texture2D(map, vUv - uTexel).rgb, vec3(0.299, 0.587, 0.114));
    if (sh <= 0.4) discard;
  }
  vec3 col = uText > 0.5 ? (l > 0.4 ? vCol.rgb : vCol.rgb * 0.18) : c.rgb * vCol.rgb;
  gl_FragColor = vec4(col, uTag);
}`;

/** one instanced quad layer with a colour per instance (juice's particles and numbers) */
export class TintLayer {
  readonly mesh: THREE.Mesh;
  private geom: THREE.InstancedBufferGeometry;
  private a: { pos: THREE.InstancedBufferAttribute; size: THREE.InstancedBufferAttribute; uv: THREE.InstancedBufferAttribute; col: THREE.InstancedBufferAttribute };
  private n = 0;
  readonly capacity: number;
  constructor(texture: THREE.Texture, capacity: number, tag: number, text: boolean, renderOrder: number) {
    this.capacity = capacity;
    const base = new THREE.PlaneGeometry(1, 1); base.translate(0.5, 0.5, 0);
    const g = new THREE.InstancedBufferGeometry();
    g.index = base.index; g.setAttribute("position", base.getAttribute("position")); g.setAttribute("uv", base.getAttribute("uv"));
    const mk = (n: number): THREE.InstancedBufferAttribute => { const x = new THREE.InstancedBufferAttribute(new Float32Array(capacity * n), n); x.setUsage(THREE.DynamicDrawUsage); return x; };
    this.a = { pos: mk(3), size: mk(3), uv: mk(4), col: mk(4) };
    g.setAttribute("iPos", this.a.pos); g.setAttribute("iSize", this.a.size); g.setAttribute("iUV", this.a.uv); g.setAttribute("iCol", this.a.col);
    g.instanceCount = 0;
    this.geom = g;
    const mat = new THREE.ShaderMaterial({ vertexShader: VERT, fragmentShader: FRAG, transparent: false, depthTest: true, depthWrite: true, side: THREE.DoubleSide,
      uniforms: { map: { value: texture }, uTag: { value: tag }, uText: { value: text ? 1 : 0 },
        uTexel: { value: new THREE.Vector2(1 / ((texture.image as { width?: number } | null)?.width || 1024), -1 / ((texture.image as { height?: number } | null)?.height || 1024)) } } });
    this.mesh = new THREE.Mesh(g, mat);
    this.mesh.frustumCulled = false; this.mesh.renderOrder = renderOrder;
  }
  begin(): void { this.n = 0; }
  push(x: number, y: number, z: number, w: number, h: number, s: Slot, r: number, g: number, b: number, a = 1, shape = 0): void {
    if (this.n >= this.capacity) return;
    const i = this.n++;
    const P = this.a.pos.array as Float32Array, S = this.a.size.array as Float32Array, U = this.a.uv.array as Float32Array, C = this.a.col.array as Float32Array;
    P[i * 3] = x; P[i * 3 + 1] = y; P[i * 3 + 2] = z;
    S[i * 3] = w; S[i * 3 + 1] = h; S[i * 3 + 2] = shape;
    U[i * 4] = s.u0; U[i * 4 + 1] = s.v0; U[i * 4 + 2] = s.u1; U[i * 4 + 3] = s.v1;
    C[i * 4] = r; C[i * 4 + 1] = g; C[i * 4 + 2] = b; C[i * 4 + 3] = a;
  }
  end(): void {
    this.geom.instanceCount = this.n;
    this.mesh.visible = this.n > 0;
    if (this.n === 0) return;
    for (const x of [this.a.pos, this.a.size, this.a.uv, this.a.col]) { x.needsUpdate = true; x.clearUpdateRanges(); x.addUpdateRange(0, this.n * x.itemSize); }
  }
  dispose(): void { this.geom.dispose(); (this.mesh.material as THREE.Material).dispose(); }
}

// ---- particles ---------------------------------------------------------------------------------------------------------------

const POOL = 900;
type Rgb = [number, number, number];
const C = {
  blood: [0.66, 0.07, 0.05] as Rgb, bloodDark: [0.36, 0.03, 0.03] as Rgb,
  goo: [0.62, 0.72, 0.18] as Rgb, pink: [0.95, 0.45, 0.7] as Rgb,
  spark: [1, 0.9, 0.55] as Rgb, ember: [1, 0.55, 0.16] as Rgb, soul: [0.78, 0.86, 1] as Rgb,
  gas: [0.5, 0.62, 0.18] as Rgb, gold: [1, 0.84, 0.35] as Rgb, heal: [0.5, 1, 0.55] as Rgb, magic: [0.62, 0.7, 1] as Rgb,
  summon: [0.75, 0.42, 1] as Rgb, mote: [1, 0.92, 0.72] as Rgb, dust: [0.55, 0.5, 0.42] as Rgb,
};
/** juice pass 3: a boss's own colour — its fall's motes, ring, light and flash take it (a cream plume over every boss read as a pale blob) */
const BOSS_COLS: [RegExp, Rgb][] = [
  [/warlord|captain/, [0.55, 0.85, 0.22]], [/bloat/, [0.72, 0.82, 0.2]], [/lich/, [0.4, 0.95, 0.85]], [/foundry|master/, [1, 0.5, 0.14]],
  [/lurker|queen/, [0.62, 0.42, 1]], [/mirror|king/, [1, 0.82, 0.38]],
];
const bossCol = (kind: string): Rgb => BOSS_COLS.find(([re]) => re.test(kind))?.[1] ?? C.gold;
const SPARKY = /skeleton|golem|sentinel|warden|mirror|echo|bell|slag|forge_imp|iron/;
const GHOSTLY = /wraith|spectral|shade|lich|siren/;

/** what the viewer tells juice about one entity this frame (world env texels: feet at x, y; h = drawn height) */
export type EntView = { x: number; y: number; h: number; kind: string; hero: boolean; boss: boolean; maxHp: number; ally: boolean };

export type JuiceCtx = {
  ent(id: number): EntView | null;
  heroId: number;
  speed: number;
  fight: boolean;
  quiet: boolean;
  cam: [number, number];            // snapped camera centre (world)
  half: [number, number];           // half the view in world units
  fires: [number, number][];        // visible fire overlays (world centre of the tile)
  gases: [number, number][];        // visible gas overlays
  waters: [number, number][];       // gfx round 5: visible water tiles (world centre): glints on the surface, a low mist
  torches: readonly (readonly [number, number])[];   // torch flames (world)
  motes: boolean;                   // the hero stands in a lit room (dust motes drift in it)
  dust: Rgb;                        // a mid colour of the biome's ramp (death dust)
};

type Num = { x: number; y: number; text: string; col: Rgb; t0: number; big: boolean; id?: number; n?: number };
type Pop = { x: number; y: number; r: number; c: Rgb; t0: number; life: number };
type Squash = { t0: number; kind: "hurt" | "lunge" | "spawn" | "die" | "drop"; ax: number; ay: number };

export class Juice {
  readonly emit: TintLayer;
  readonly matte: TintLayer;
  readonly nums: TintLayer;
  private q: Quality;
  private white: () => Slot;
  private font: (ch: string) => Slot;
  // SoA particle pool
  private px = new Float32Array(POOL); private py = new Float32Array(POOL); private vx = new Float32Array(POOL); private vy = new Float32Array(POOL);
  private life = new Float32Array(POOL); private max = new Float32Array(POOL); private size = new Float32Array(POOL);
  private floor = new Float32Array(POOL); private grav = new Float32Array(POOL); private drag = new Float32Array(POOL);
  private r0 = new Float32Array(POOL); private g0 = new Float32Array(POOL); private b0 = new Float32Array(POOL);
  private fadeTo = new Float32Array(POOL);   // colour multiplier reached at the end of life (embers darken)
  private glow = new Uint8Array(POOL); private disc = new Uint8Array(POOL); private alpha = new Float32Array(POOL); private sway = new Float32Array(POOL);
  private live = 0;
  private queue: Ev[] = [];
  private numbers: Num[] = [];
  private pops: Pop[] = [];
  private squashes = new Map<number, Squash>();
  private stopUntil = 0; private stopCool = 0;
  private slowUntil = 0; private slowRate = 1;
  private kick = { t0: -1e9, amp: 0, dur: 0 };
  private vig = { t0: -1e9, c: [0, 0, 0] as Rgb, a: 0, dur: 1 };
  private deadAt = -1;
  private bossMet = new Set<number>();   // juice pass 2: bosses whose entrance played (once each)
  private lastBoss = -1;
  private bossPlayed = new Set<number>();
  private rnd = 0x2545f491;
  readonly breaks: number[] = [];
  domNums = true;   // gfx round 6: numbers drawn by the viewer's DOM layer (the list below), not as bitmap quads
  readonly shown: { x: number; y: number; text: string; col: Rgb; sc: number; a: number; big: boolean }[] = [];   // gfx round 1: bosses whose guard broke this frame (the viewer drains it)
  private emitAcc = 0;
  clock = 0;   // the replay clock (ticks) as of the last update: squash is timed in ticks

  constructor(env: THREE.Texture, q: Quality, white: () => Slot, font: (ch: string) => Slot) {
    this.q = q;
    this.emit = new TintLayer(env, POOL, EMISSIVE_TAG, false, 8);
    this.matte = new TintLayer(env, POOL, 0.5, false, 8);
    this.nums = new TintLayer(env, 256, 0.5, true, 9);
    this.white = white; this.font = font;
  }

  private on(): boolean { return this.q.at("med"); }
  private moving(): boolean { return this.on() && this.q.motion; }
  private rand(): number { this.rnd ^= this.rnd << 13; this.rnd ^= this.rnd >>> 17; this.rnd ^= this.rnd << 5; return ((this.rnd >>> 0) % 100000) / 100000; }

  /** the replay state's hook: every event applied in play (not while a seek or a skip replays the past) */
  onEvent(ev: Ev): void { if (this.on() && this.queue.length < 400) this.queue.push(ev); }

  reset(): void { this.bossMet.clear(); this.bossPlayed.clear(); this.lastBoss = -1; this.queue.length = 0; this.numbers.length = 0; this.pops.length = 0; this.squashes.clear(); this.live = 0; this.deadAt = -1; this.slowUntil = 0; this.stopUntil = 0; this.vig.a = 0; }

  /** juice pass 2: a boss is in view for the first time (the viewer's per-frame check — the core sends no `see`) */
  bossIn(id: number): void { if (this.on() && !this.bossMet.has(id)) { this.bossMet.add(id); this.queue.push({ t: this.clock, k: "see", id } as Ev); } }

  /** the replay clock's rate this frame: 0 in a hit-stop, < 1 in a slow-mo, else 1 */
  timeScale(now: number): number {
    if (!this.moving()) return 1;
    if (now < this.stopUntil) return 0;
    if (now < this.slowUntil) return this.slowRate;
    return 1;
  }

  /** the kick shake this frame (whole env texels) */
  shake(now: number): [number, number] {
    if (!this.moving()) return [0, 0];
    const p = (now - this.kick.t0) / this.kick.dur;
    if (p < 0 || p >= 1) return [0, 0];
    const a = Math.round(this.kick.amp * (1 - p) * (1 - p) + 0.49);
    if (a <= 0) return [0, 0];
    const f = Math.floor((now - this.kick.t0) / 33);
    const s = (f * 2654435761) >>> 0;
    return [((s & 3) - 1.5 > 0 ? 1 : -1) * a, (((s >> 2) & 3) - 1.5 > 0 ? 1 : -1) * (f & 1 ? a : 0)];
  }

  /** the vignette: [r, g, b, pulse strength], the static darkening at the corners, the desaturation */
  vignette(now: number, heroHp: number): { c: Rgb; a: number; base: number; desat: number } {
    if (!this.on()) return { c: [0, 0, 0], a: 0, base: 0, desat: 0 };
    let c: Rgb = [0.55, 0.02, 0.02], a = 0;
    if (heroHp > 0 && heroHp < 0.3) {   // low hp: a heartbeat (static under reduced motion)
      const beat = this.q.motion ? 0.5 + 0.5 * Math.pow(Math.max(0, Math.sin((now / 1000) * Math.PI * 2 * 1.1)), 6) : 0.6;
      a = (0.12 + 0.2 * (1 - heroHp / 0.3)) * beat;
    }
    const p = (now - this.vig.t0) / this.vig.dur;
    if (p >= 0 && p < 1) { const va = this.vig.a * (this.q.motion ? (1 - p) * (1 - p) : 0.5 * (1 - p)); if (va > a) { a = va; c = this.vig.c; } }
    let desat = 0;
    if (this.deadAt > 0) { const d = Math.min(1, (now - this.deadAt) / 1200); desat = 0.55 * d; a = Math.max(a, 0.32 * d); c = [0.4, 0.02, 0.02]; }
    return { c, a, base: 0.17, desat };
  }

  /** squash & stretch for an entity (multipliers on its drawn width and height; feet stay put) */
  squash(id: number, clock: number): [number, number] {
    if (!this.moving()) return [1, 1];
    const s = this.squashes.get(id);
    if (!s) return [1, 1];
    const dt = clock - s.t0;
    if (s.kind === "hurt") { const p = dt / 2.5; if (p >= 1 || p < 0) return [1, 1]; const e = (1 - p) * Math.cos(p * Math.PI * 1.5); return [1 + 0.16 * e, 1 - 0.14 * e]; }
    if (s.kind === "lunge") { const p = dt / 3; if (p >= 1 || p < 0) return [1, 1]; const e = Math.sin(p * Math.PI) * (1 - p * 0.5); return [1 + 0.12 * e * s.ax - 0.05 * e * s.ay, 1 + 0.1 * e * s.ay - 0.05 * e * s.ax]; }
    if (s.kind === "spawn") { const p = dt / 3; if (p >= 1 || p < 0) return [1, 1]; const b = 1 + 2.7 * Math.pow(p - 1, 3) + 1.7 * Math.pow(p - 1, 2); return [0.7 + 0.3 * b, 0.4 + 0.6 * b]; }
    if (s.kind === "drop") {   // falling (stretched), the landing (squashed), the settle
      if (dt < 0 || dt >= 7) return [1, 1];
      if (dt < 3) return [0.9, 1.12];
      const p = (dt - 3) / 4, e = (1 - p) * Math.cos(p * Math.PI * 1.5); return [1 + 0.22 * e, 1 - 0.2 * e];
    }
    const p = Math.min(1, dt / 12); return [1 + 0.25 * p, 1 - 0.45 * p];
  }

  /** gfx round 5: an entity's drop height this frame (world units above its feet) — a boss falling into his arena over 3 ticks */
  lift(id: number, clock: number): number {
    if (!this.moving()) return 0;
    const s = this.squashes.get(id); if (!s || s.kind !== "drop") return 0;
    const dt = clock - s.t0; if (dt < 0 || dt >= 3) return 0;
    const p = dt / 3; return Math.round(36 * (1 - p * p));
  }
  /** the first 0.7 tick of a hit on a foe flashes near-white (med+; the hero keeps the viewer's half flash — QA 1a2a4a9) */
  hitFlash(id: number, clock: number, hero: boolean): number {
    if (!this.on() || hero) return 0;
    const s = this.squashes.get(id);
    // 307dbed control rater AR ("the ogre drew as a checkerboard blob"): at 0.85 a struck foe turned a cream silhouette on every blow of
    // a long fight — a big sprite read as a placeholder; the struck foe keeps its drawing under a lighter flash
    return s && s.kind === "hurt" && clock - s.t0 >= 0 && clock - s.t0 < 0.7 ? 0.6 : 0;
  }

  /** the pop lights live this frame (tile coordinates, row space — light.ts) */
  lights(now: number, out: FieldLight[]): void {
    if (!this.on()) return;
    this.pops = this.pops.filter((p) => now - p.t0 < p.life);
    for (const p of this.pops) {
      const k = 1 - (now - p.t0) / p.life;
      out.push({ x: p.x / 8, y: -p.y / 8, r: p.r, c: [p.c[0] * k, p.c[1] * k, p.c[2] * k] });
    }
  }

  private pop(x: number, y: number, r: number, c: Rgb, life: number, now: number): void { if (this.pops.length < 16) this.pops.push({ x, y, r, c, t0: now, life }); }

  private spawn(x: number, y: number, vx: number, vy: number, life: number, size: number, c: Rgb, o: { glow?: boolean; grav?: number; floor?: number; drag?: number; disc?: boolean; alpha?: number; fadeTo?: number; sway?: number } = {}): void {
    if (this.live >= POOL) return;
    const i = this.live++;
    this.px[i] = x; this.py[i] = y; this.vx[i] = vx; this.vy[i] = vy; this.life[i] = life; this.max[i] = life; this.size[i] = size;
    this.r0[i] = c[0]; this.g0[i] = c[1]; this.b0[i] = c[2];
    this.glow[i] = o.glow ? 1 : 0; this.grav[i] = o.grav ?? 0; this.floor[i] = o.floor ?? -1e9; this.drag[i] = o.drag ?? 0;
    this.disc[i] = o.disc ? 1 : 0; this.alpha[i] = o.alpha ?? 1; this.fadeTo[i] = o.fadeTo ?? 1; this.sway[i] = o.sway ?? 0;
  }

  private burst(n: number, x: number, y: number, speed: number, c: Rgb, life: number, size: number, o: Parameters<Juice["spawn"]>[7] = {}, up = 0): void {
    for (let i = 0; i < n; i++) {
      const a = this.rand() * Math.PI * 2, s = speed * (0.35 + 0.65 * this.rand());
      this.spawn(x, y, Math.cos(a) * s, Math.sin(a) * s * 0.8 + up, life * (0.6 + 0.4 * this.rand()), size, c, o);
    }
  }

  /** process this frame's events, run the simulation, and push every quad (particles and numbers) */
  update(dtMs: number, now: number, clock: number, x: JuiceCtx): void {
    this.clock = clock;
    this.emit.begin(); this.matte.begin(); this.nums.begin();
    if (!this.on()) { this.live = 0; this.queue.length = 0; this.numbers.length = 0; this.shown.length = 0; this.emit.end(); this.matte.end(); this.nums.end(); return; }
    for (const ev of this.queue) this.handle(ev, now, x);
    this.queue.length = 0;
    const sim = x.speed > 0 ? Math.min(2.5, Math.max(1, x.speed)) * (dtMs / 1000) : 0;
    this.ambient(dtMs, x);
    this.step(sim);
    this.draw(now, x);
  }

  private handle(ev: Ev, now: number, x: JuiceCtx): void {
    const mv = this.q.motion;
    switch (ev.k) {
      case "hurt": {
        const e = x.ent(ev.id);
        if (!e || ev.dmg <= 0) break;
        const cx = e.x, cy = e.y + e.h * 0.45;
        const heavy = e.maxHp > 0 && ev.dmg >= e.maxHp * 0.15;
        if (e.boss) this.lastBoss = ev.id;
        this.squashes.set(ev.id, { t0: ev.t, kind: "hurt", ax: 0, ay: 0 });
        const n = Math.min(14, 5 + ev.dmg);
        if (SPARKY.test(e.kind)) this.burst(n, cx, cy, 55, C.spark, 0.35, 0.5, { glow: true, grav: -90, drag: 2 }, 12);
        else if (GHOSTLY.test(e.kind)) this.burst(n, cx, cy, 22, C.soul, 0.7, 1, { glow: true, drag: 1.5, alpha: 0.8 }, 10);
        else if (/bloat|jelly|slime/.test(e.kind)) this.burst(n, cx, cy, 40, /pink/.test(e.kind) ? C.pink : C.goo, 0.8, 1, { grav: -160, floor: e.y - 2 - this.rand() * 3 }, 25);
        else { this.burst(n, cx, cy, 42, C.blood, 0.9, 1, { grav: -170, floor: e.y - 1 - this.rand() * 4 }, 22); this.burst(3, cx, cy, 20, C.bloodDark, 1.1, 1, { grav: -170, floor: e.y - 2 }, 10); }
        this.burst(3, cx, cy, 70, C.spark, 0.12, 0.5, { glow: true, drag: 6 });   // the impact's glint
        if (e.hero) {
          this.pop(cx, cy, 2.6, [1.1, 0.22, 0.12], 220, now);
          this.flashVig([0.7, 0.03, 0.02], heavy ? 0.42 : 0.26, heavy ? 380 : 240, now);
          if (!x.fight) this.kickShake(heavy ? 2 : 1, heavy ? 200 : 120, now);
          if (heavy && x.speed <= 1.5) this.hitStop(85, now);
        } else {
          this.pop(cx, cy, 2.2, [1.0, 0.8, 0.5], 150, now);
          if (e.boss && x.speed <= 1.5) this.hitStop(45, now);
        }
        // gfx round 1 (raters: "stacked '4' numbers"): blows on one body within 350 ms add up on one number (re-popped), not a stack
        const prev = this.numbers.find((m) => m.id === ev.id && now - m.t0 < 350);
        if (!x.quiet && prev) { prev.n = (prev.n ?? 0) + ev.dmg; prev.text = String(prev.n); prev.t0 = now; prev.big = prev.big || heavy; prev.x = cx; }
        else if (!x.quiet) this.numbers.push({ x: cx, y: e.y + e.h + 1, text: String(ev.dmg), col: e.hero ? [1, 0.3, 0.22] : e.ally ? [0.7, 1, 0.6] : [1, 0.96, 0.85], t0: now, big: heavy, id: ev.id, n: ev.dmg });
        if (this.numbers.length > 24) this.numbers.shift();
        break;
      }
      case "attack": {
        const s = x.ent(ev.src), d = x.ent(ev.dst);
        if (!s) break;
        const ax = d ? Math.sign(d.x - s.x) : 0, ay = d ? Math.sign(d.y - s.y) : 0;
        this.squashes.set(ev.src, { t0: ev.t, kind: "lunge", ax: Math.abs(ax), ay: Math.abs(ay) });
        if (!ev.hit && d) this.burst(4, d.x, d.y + 2, 18, x.dust, 0.35, 1, { drag: 3, alpha: 0.7 }, 6);   // a whiff: dust at the dodger's feet
        // gfx round 1 (raters: "no slash arc, the hit lacks impact"): a melee blow that lands draws a bright crescent across the target,
        // swept from the attacker's side — white-gold from the hero and his allies, blood-red from a foe
        if (ev.hit && d && Math.abs(d.x - s.x) <= 14 && Math.abs(d.y - s.y) <= 14) this.slash(s, d, s.hero || s.ally);
        break;
      }
      case "die": {
        const e = x.ent(ev.id);
        if (!e) break;
        this.squashes.set(ev.id, { t0: ev.t, kind: "die", ax: 0, ay: 0 });
        const cx = e.x, cy = e.y + e.h * 0.35;
        this.burst(16, cx, e.y + 2, 30, x.dust, 0.8, 1, { drag: 2.5, alpha: 0.85 }, 8);
        if (!e.boss) this.burst(6, cx, cy, 10, C.soul, 1.3, 0.5, { glow: true, drag: 0.5, sway: 6 }, 16);
        if (!SPARKY.test(e.kind) && !GHOSTLY.test(e.kind)) this.burst(8, cx, cy, 50, C.blood, 0.9, 1, { grav: -170, floor: e.y - 1 - this.rand() * 4 }, 20);
        if (!e.boss) this.pop(cx, cy, 3, e.hero ? [0.9, 0.1, 0.05] : [1, 0.62, 0.3], 320, now);
        if (e.hero) { this.deadAt = now; if (mv) { this.slowUntil = now + 1100; this.slowRate = 0.35; } this.kickShake(3, 320, now); this.flashVig([0.6, 0.02, 0.02], 0.5, 900, now); }
        else if (e.boss) {
          // juice pass 2: the boss's fall is the floor's biggest beat — a dust ring, a ring and motes, a flash, a slow-mo. Pass 3 (the
          // "pale blob": a cream plume, a white-gold light and a long flash washed the Warlord out before he fell): all in the boss's own
          // colour, smaller and shorter — the light a small pop at his chest, the motes few and rising off him, the flash and slow-mo brief
          const bc = bossCol(e.kind);
          if (mv) { this.slowUntil = now + 700; this.slowRate = 0.3; }
          this.kickShake(3, 420, now); this.flashVig(bc, 0.3, 480, now);
          this.ring(28, cx, e.y + 1, 60, x.dust, 0.6, false); this.ring(16, cx, e.y + 1, 32, bc, 0.5, true);
          this.burst(10, cx, e.y + e.h * 0.7, 8, bc, 0.9, 0.5, { glow: true, drag: 0.6, sway: 5 }, 18);
          this.pop(cx, cy, 3.5, [bc[0] * 0.8, bc[1] * 0.8, bc[2] * 0.8], 420, now);
          // gfx round 1 (raters: "a boss kill deserves a loot burst"): his hoard spills — gold fountains up and rains down around him
          for (let i = 0; i < 26; i++) { const a = -Math.PI / 2 + (this.rand() - 0.5) * 2.2; const sp = 50 + this.rand() * 60; this.spawn(cx + (this.rand() - 0.5) * 6, e.y + e.h * 0.5, Math.cos(a) * sp * 0.6, -Math.sin(a) * sp, 1.4 + this.rand() * 0.8, this.rand() < 0.5 ? 1 : 0.5, C.gold, { glow: true, grav: -150, floor: e.y - 3 - this.rand() * 8, drag: 0.5 }); }
        }
        else if (x.speed <= 1.5) this.hitStop(55, now);
        break;
      }
      case "spawn": {
        const e = x.ent(ev.e.id);
        if (!e) break;
        this.squashes.set(ev.e.id, { t0: ev.t, kind: "spawn", ax: 0, ay: 0 });
        this.burst(10, e.x, e.y + 4, 16, e.ally ? C.heal : C.summon, 0.7, 0.5, { glow: true, drag: 1.5, sway: 4 }, 14);
        this.pop(e.x, e.y + 4, 2.5, e.ally ? [0.3, 0.8, 0.4] : [0.6, 0.3, 0.9], 400, now);
        if (e.boss) this.bossEntrance(ev.e.id, e, now, x);
        break;
      }
      case "use": {
        const h = x.ent(x.heroId);
        if (!h) break;
        const c = /heal|regen|life/.test(ev.outcome) ? C.heal : /fire|burn|caustic|poison/.test(ev.outcome) ? C.ember : C.magic;
        for (let i = 0; i < 16; i++) { const a = (i / 16) * Math.PI * 2; this.spawn(h.x + Math.cos(a) * 5, h.y + 2 + Math.sin(a) * 2, -Math.sin(a) * 8, 14 + this.rand() * 10, 0.9, 0.5, c, { glow: true, drag: 1, sway: 3 }); }
        this.pop(h.x, h.y + 6, 3.5, [c[0] * 0.9, c[1] * 0.9, c[2] * 0.9], 550, now);
        break;
      }
      case "pickup": {
        const e = x.ent(ev.id);
        if (!e) break;
        const gold = /gold|coin/.test(ev.item);
        this.burst(gold ? 12 : 6, e.x, e.y + 3, 26, gold ? C.gold : C.mote, 0.6, 0.5, { glow: true, drag: 3 }, 16);
        if (gold) this.pop(e.x, e.y + 3, 2, [1, 0.8, 0.3], 400, now);
        break;
      }
      case "tame": {
        const e = x.ent(ev.id);
        if (!e || !ev.ok) break;
        this.burst(14, e.x, e.y + 4, 20, C.heal, 1, 0.5, { glow: true, drag: 1.5, sway: 4 }, 14);
        this.pop(e.x, e.y + 4, 3, [0.35, 0.9, 0.4], 600, now);
        break;
      }
      case "see": {
        const id = (ev as { id: number }).id, e = x.ent(id);
        if (e?.boss) this.bossEntrance(id, e, now, x);
        break;
      }
      case "callout": case "note": {   // juice pass 2: `warlord breaks` — the boss's break is a hit of its own
        if (this.lastBoss >= 0 && /\bbreaks?\b/i.test((ev as { text: string }).text)) {
          const e = x.ent(this.lastBoss); if (!e || !e.boss) break;
          const cx = e.x, cy = e.y + e.h * 0.5;
          // pass 3: the break's pop lit the hero beside the boss into a cream silhouette — a smaller, dimmer, shorter light
          this.burst(22, cx, cy, 80, C.spark, 0.4, 0.5, { glow: true, grav: -120, drag: 2 }, 20);
          // gfx round 1 (raters: "no shield-shatter shards"): his guard breaks into iron shards that fly, fall and lie on the floor
          for (let i = 0; i < 18; i++) { const a = this.rand() * Math.PI * 2, sp = 40 + this.rand() * 55; this.spawn(cx + Math.cos(a) * 3, cy + Math.sin(a) * 3, Math.cos(a) * sp, Math.sin(a) * sp * 0.7 + 30, 1.1 + this.rand() * 0.6, this.rand() < 0.4 ? 1.5 : 1, i % 3 ? [0.62, 0.64, 0.7] : [0.9, 0.92, 1], { grav: -190, floor: e.y - 2 - this.rand() * 6, drag: 0.8 }); }
          this.ring(22, cx, e.y + 1, 50, C.ember, 0.35, true);
          this.pop(cx, cy, 3, [0.9, 0.45, 0.14], 380, now);
          this.flashVig([1, 0.5, 0.12], 0.3, 420, now); this.kickShake(3, 380, now);
          if (x.speed <= 1.5) this.hitStop(110, now);
          this.squashes.set(this.lastBoss, { t0: this.clock, kind: "hurt", ax: 0, ay: 0 });
          this.breaks.push(this.lastBoss);   // gfx round 1: the viewer draws his guard shattering over him (tags.ts `shatter`)
        }
        break;
      }
      default: break;
    }
  }

  /** juice pass 2: a boss steps in — a dark ring of dust, rising violet motes, a violet pop light, a shake and a held breath */
  private bossEntrance(id: number, e: EntView, now: number, x: JuiceCtx): void {
    this.bossMet.add(id); this.lastBoss = id;
    if (this.bossPlayed.has(id)) return;
    this.bossPlayed.add(id);
    this.ring(32, e.x, e.y + 1, 60, x.dust, 0.55, false);
    this.burst(16, e.x, e.y + e.h * 0.4, 12, [0.7, 0.4, 1], 1.4, 0.5, { glow: true, drag: 0.6, sway: 5 }, 22);
    this.pop(e.x, e.y + e.h * 0.5, 6.5, [0.75, 0.3, 1.1], 1000, now);
    this.flashVig([0.35, 0.05, 0.45], 0.55, 1100, now); this.kickShake(3, 520, now);
    if (x.speed <= 1.5) this.hitStop(140, now);
    this.squashes.set(id, { t0: this.clock, kind: "drop", ax: 0, ay: 0 });   // gfx round 5 (raters: "the boss doesn't step in"): he drops in
  }
  /** gfx round 1: a slash — a crescent of bright points across the target's chest, swept away from the attacker, gone in ~0.2 s */
  private slash(s: EntView, d: EntView, friendly: boolean): void {
    const cx = d.x, cy = d.y + d.h * 0.5, side = Math.sign(d.x - s.x) || (this.rand() < 0.5 ? -1 : 1);
    // gfx round 4 (raters, round 2: "no slash arc" — the 11 half-texel points lived 0.2 s): a fuller crescent, two strokes deep, ~0.3 s
    const r = Math.max(7, Math.min(14, d.h * 0.5)), a0 = side > 0 ? 2.3 : 0.85, span = 2.1 * (side > 0 ? -1 : 1);
    const c: Rgb = friendly ? [1, 0.95, 0.75] : [1, 0.35, 0.22], c2: Rgb = friendly ? [1, 0.75, 0.35] : [0.8, 0.12, 0.08];
    for (let i = 0; i < 18; i++) {
      const u = i / 17, a = a0 + span * u, w = Math.sin(u * Math.PI), rr = r * (0.82 + 0.3 * w);
      const x0 = cx + Math.cos(a) * rr - side * 2, y0 = cy + Math.sin(a) * rr * 0.9, vx = -Math.sin(a) * 22 * Math.sign(span), vy = Math.cos(a) * 22 * Math.sign(span);
      this.spawn(x0, y0, vx, vy, 0.16 + 0.18 * w, w > 0.45 ? 1.5 : 1, c, { glow: true, drag: 7, fadeTo: 0.5 });
      if (w > 0.3) this.spawn(x0 - Math.cos(a) * 1.5, y0 - Math.sin(a) * 1.35, vx * 0.8, vy * 0.8, 0.12 + 0.14 * w, 1, c2, { glow: true, drag: 7, fadeTo: 0.4 });
    }
  }
  /** a flat ring (top-down: squashed vertically) of n particles flying out from (cx, cy) */
  private ring(n: number, cx: number, cy: number, speed: number, c: Rgb, life: number, glow: boolean): void {
    for (let i = 0; i < n; i++) {
      const a = (i / n) * Math.PI * 2 + this.rand() * 0.1, s = speed * (0.9 + 0.2 * this.rand());
      this.spawn(cx + Math.cos(a) * 2, cy + Math.sin(a) * 1.2, Math.cos(a) * s, Math.sin(a) * s * 0.55, life, 1, c, { glow, drag: 3.2, alpha: glow ? 1 : 0.85 });
    }
  }

  private hitStop(ms: number, now: number): void { if (!this.moving() || now < this.stopCool) return; this.stopUntil = now + ms; this.stopCool = now + ms + 220; }
  private kickShake(amp: number, dur: number, now: number): void { if (now - this.kick.t0 < this.kick.dur && this.kick.amp > amp) return; this.kick = { t0: now, amp, dur }; }
  private flashVig(c: Rgb, a: number, dur: number, now: number): void { this.vig = { t0: now, c, a, dur }; }

  /** embers over fire, puffs over gas, a spark off a torch, motes in a lit room (all real-time rates, only near the camera) */
  private ambient(dtMs: number, x: JuiceCtx): void {
    if (!this.q.motion || x.speed <= 0) return;
    const dt = dtMs / 1000;
    const near = (p: readonly [number, number]): boolean => Math.abs(p[0] - x.cam[0]) < x.half[0] + 8 && Math.abs(p[1] - x.cam[1]) < x.half[1] + 8;
    for (const f of x.fires) if (near(f) && this.rand() < 9 * dt) this.spawn(f[0] + (this.rand() - 0.5) * 7, f[1] - 3 + this.rand() * 3, (this.rand() - 0.5) * 4, 9 + this.rand() * 9, 0.7 + this.rand() * 0.6, this.rand() < 0.3 ? 1 : 0.5, C.ember, { glow: true, fadeTo: 0.25, sway: 5 });
    for (const g of x.gases) if (near(g) && this.rand() < 4.5 * dt) this.spawn(g[0] + (this.rand() - 0.5) * 7, g[1] - 3 + this.rand() * 5, (this.rand() - 0.5) * 3, 1.5 + this.rand() * 2, 1.6 + this.rand(), 2 + this.rand() * 1.5, [C.gas[0] * 0.45, C.gas[1] * 0.5, C.gas[2] * 0.4], { disc: true, glow: true, drag: 0.3, sway: 2, fadeTo: 0.3 });   // gfx round 1: a glowing puff (the 55 % screen-door read as a checker)
    for (const w of x.waters) {   // gfx round 5: the pools catch the light — a glint that winks out, now and then a breath of mist
      if (this.rand() < 0.55 * dt) this.spawn(w[0] + (this.rand() - 0.5) * 7, w[1] + (this.rand() - 0.5) * 6, 0, 0, 0.35 + this.rand() * 0.35, this.rand() < 0.3 ? 1 : 0.5, [0.75, 0.95, 1], { glow: true, fadeTo: 0.2 });
      if (this.rand() < 0.08 * dt) this.spawn(w[0] + (this.rand() - 0.5) * 6, w[1] - 2, (this.rand() - 0.5) * 2, 1 + this.rand(), 2.5 + this.rand(), 2.5, [0.16, 0.22, 0.24], { disc: true, glow: true, drag: 0.3, sway: 1.5, fadeTo: 0.2 });
    }
    for (const t of x.torches) if (near(t) && this.rand() < 0.9 * dt) this.spawn(t[0] + (this.rand() - 0.5) * 2, t[1] + 2, (this.rand() - 0.5) * 3, 7 + this.rand() * 6, 0.6 + this.rand() * 0.5, 0.5, C.ember, { glow: true, fadeTo: 0.3, sway: 3 });
    if (x.motes && this.q.at("high")) {
      this.emitAcc += dt * 3;
      while (this.emitAcc > 1) {
        this.emitAcc -= 1;
        this.spawn(x.cam[0] + (this.rand() - 0.5) * x.half[0] * 1.6, x.cam[1] + (this.rand() - 0.5) * x.half[1] * 1.6, (this.rand() - 0.5) * 2, (this.rand() - 0.3) * 1.5, 3 + this.rand() * 3, 0.5, C.mote, { alpha: 0.5, sway: 1.5 });
      }
    }
  }

  private step(dt: number): void {
    if (dt <= 0) return;
    let n = this.live;
    for (let i = 0; i < n; i++) {
      this.life[i] -= dt;
      if (this.life[i]! <= 0) {   // swap-remove
        n--;
        for (const arr of [this.px, this.py, this.vx, this.vy, this.life, this.max, this.size, this.floor, this.grav, this.drag, this.r0, this.g0, this.b0, this.fadeTo, this.alpha, this.sway] as Float32Array[]) arr[i] = arr[n]!;
        this.glow[i] = this.glow[n]!; this.disc[i] = this.disc[n]!;
        i--; continue;
      }
      if (this.py[i]! <= this.floor[i]! && this.grav[i]! < 0) { this.vx[i] = 0; this.vy[i] = 0; continue; }   // landed: a drop stays as a stain
      const d = Math.max(0, 1 - this.drag[i]! * dt);
      this.vx[i] = this.vx[i]! * d; this.vy[i] = (this.vy[i]! + this.grav[i]! * dt) * d;
      const sw = this.sway[i]! ? Math.sin((this.life[i]! * 7 + i) * 1.3) * this.sway[i]! : 0;
      this.px[i] = this.px[i]! + (this.vx[i]! + sw) * dt; this.py[i] = this.py[i]! + this.vy[i]! * dt;
    }
    this.live = n;
  }

  private draw(now: number, x: JuiceCtx): void {
    const w = this.white();
    for (let i = 0; i < this.live; i++) {
      const p = this.life[i]! / this.max[i]!;   // 1 → 0
      const k = this.fadeTo[i]! + (1 - this.fadeTo[i]!) * p;
      const a = this.alpha[i]! * Math.min(1, p * 2.5);
      const s = this.size[i]!;
      const L = this.glow[i] ? this.emit : this.matte;
      // whole target pixels (half env texels): crisp
      const X = Math.round(this.px[i]! * 2) / 2, Y = Math.round(this.py[i]! * 2) / 2;
      L.push(X, Y, 3.9, s, s, w, this.r0[i]! * k, this.g0[i]! * k, this.b0[i]! * k, a, this.disc[i]!);
    }
    // numbers: pop in at 2× for 90 ms, then rise and dissolve over 800 ms
    this.shown.length = 0;
    this.numbers = this.numbers.filter((n) => now - n.t0 < 900);
    for (const n of this.numbers) {
      const t = now - n.t0;
      // gfx round 1 (raters: "damage '4' tiny glyphs"): full env size, popping in at 2× (a heavy blow 1.4×)
      const sc = (t < 90 && this.q.motion ? 1.15 : 0.75) * (n.big ? 1.3 : 1);
      const rise = this.q.motion ? 10 * (1 - Math.pow(1 - Math.min(1, t / 800), 3)) : 2;
      const a = t > 550 ? 1 - (t - 550) / 350 : 1;
      // gfx round 6 (raters, every round: "damage digits tiny/crude", "the 80 reads as '$0'"): the numbers are DOM glyphs in the game's
      // face with an outline (tags.ts `numbers`), placed from here; the bitmap quads stay for a viewer with no DOM layer
      if (this.domNums) this.shown.push({ x: n.x, y: n.y + rise, text: n.text, col: n.col, sc, a, big: n.big });
      else this.text(n.text, n.x, n.y + rise, sc, n.col, a);
    }
    this.emit.end(); this.matte.end(); this.nums.end();
    void x;
  }

  private text(s: string, cx: number, y: number, sc: number, c: Rgb, a: number): void {
    const cw = 5 * sc, ch = 7 * sc, adv = 4 * sc;
    const width = s.length * adv + sc;
    let X = Math.round((cx - width / 2) / sc) * sc;
    const Y = Math.round(y / sc) * sc;
    for (const chr of s) { this.nums.push(X + cw / 2, Y, 3.95, cw, ch, this.font(chr), c[0], c[1], c[2], a); X += adv; }
  }

  dispose(): void { this.emit.dispose(); this.matte.dispose(); this.nums.dispose(); }
}
