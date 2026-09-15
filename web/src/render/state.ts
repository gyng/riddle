// Replay state: the floor as last loaded plus the animated consequences of applied events.
// No three.js here. Time is an "anim clock" in ms that advances at dt*speed (0 = paused); event
// durations below are at 1× speed. Motion progress is quantised to STEP ms of anim time so the
// world moves at ~10 fps cadence (at 4× the same steps run 4× faster).
import type { Ev, Overlay, Snapshot, Tile, FloorItem, Entity } from "./types";

export const STEP = 100; // ms of anim time per motion frame
export const VISION_R = 7;

export type EntState = {
  id: number; kind: string; ally: boolean; hero: boolean;
  x: number; y: number;            // logical tile
  px: number; py: number;          // render tile pos (float)
  move: { fx: number; fy: number; tx: number; ty: number; t0: number; dur: number } | null;
  lunge: { dx: number; dy: number; t0: number } | null;
  flashUntil: number;
  fade: number;                    // 0 = solid, 1 = gone
  dying: { t0: number; dur: number } | null;
  spawning: { t0: number; dur: number } | null;
  hp: number; maxHp: number;
  flip: boolean;
  glyph: string | null; glyphTurn: number;
};

export type Callout = { text: string; until: number }; // real-time ms

type Batch = { evs: Ev[]; ends: number };

const DUR: Record<Ev["k"], number> = {
  move: 200, attack: 200, hurt: 100, die: 400, rule: 0, telegraph: 300, pickup: 150, use: 300,
  fact: 0, overlay: 80, spawn: 200, steal: 250, ally: 250, descend: 600, exit: 700, note: 0, callout: 150,
};
const INTERESTING = new Set<Ev["k"]>(["attack", "die", "telegraph", "use", "exit"]);

export class ReplayState {
  w = 0; h = 0; biome = "warrens"; depth = 1;
  tiles: Tile[] = [];
  seen: Uint8Array = new Uint8Array(0);
  visible: Uint8Array = new Uint8Array(0);
  overlays: Overlay[] = [];
  items: FloorItem[] = [];
  ents = new Map<number, EntState>();
  heroId = -1;
  turn = 0;
  clock = 0;          // anim ms
  speed = 1;
  fadeTarget = 0;     // global fade (0 = visible, 1 = dark)
  fade = 0;
  callout: Callout | null = null;
  loaded = false;
  private queue: Ev[] = [];
  private active: Batch | null = null;
  visionDirty = true;

  load(s: Snapshot): void {
    this.w = s.w; this.h = s.h; this.biome = s.biome; this.depth = s.depth;
    this.tiles = s.tiles.slice();
    this.seen = Uint8Array.from(s.seen, (b) => (b ? 1 : 0));
    this.visible = Uint8Array.from(s.visible, (b) => (b ? 1 : 0));
    this.overlays = s.overlays.map((o) => ({ ...o }));
    this.items = s.items.map((i) => ({ ...i }));
    this.ents.clear();
    this.heroId = s.hero.id;
    this.turn = s.turn;
    this.addEntity(s.hero, true);
    for (const e of s.entities) this.addEntity(e, false);
    this.queue = [];
    this.active = null;
    this.fade = this.fadeTarget = 0;
    this.callout = null;
    this.loaded = true;
    this.computeVision();
  }

  private addEntity(e: Entity, hero: boolean): EntState {
    const st: EntState = {
      id: e.id, kind: e.kind, ally: !!e.ally, hero, x: e.x, y: e.y, px: e.x, py: e.y,
      move: null, lunge: null, flashUntil: 0, fade: 0, dying: null, spawning: null,
      hp: e.hp, maxHp: e.max_hp, flip: false, glyph: e.telegraph ? "!" : null, glyphTurn: this.turn,
    };
    this.ents.set(e.id, st);
    return st;
  }

  get hero(): EntState | undefined { return this.ents.get(this.heroId); }

  tile(x: number, y: number): Tile | undefined {
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return undefined;
    return this.tiles[y * this.w + x];
  }

  apply(evs: Ev[]): void { for (const e of evs) this.queue.push(e); }
  idle(): boolean { return this.queue.length === 0 && this.active === null; }
  pending(): number { return this.queue.length; }

  // Advance the anim clock and run events. dt = real ms.
  tick(dt: number, now: number): void {
    if (!this.loaded) return;
    this.clock += dt * this.speed;
    // global fade toward target (real-time-ish: uses anim clock, but exit/descend set a long batch)
    const fs = (dt / 350) * Math.max(this.speed, 0.5);
    this.fade += Math.sign(this.fadeTarget - this.fade) * Math.min(fs, Math.abs(this.fadeTarget - this.fade));
    if (this.callout && now > this.callout.until) this.callout = null;
    if (this.speed <= 0) return;
    let guard = 64;
    while (guard-- > 0) {
      if (this.active) {
        if (this.clock < this.active.ends) break;
        this.finish(this.active);
        this.active = null;
      }
      const b = this.nextBatch();
      if (!b) break;
      this.active = b;
    }
    this.settle();
  }

  // Fast-forward: finish the current batch, then apply instantly until the head is "interesting".
  skipToEvent(): void {
    if (this.active) { this.finish(this.active); this.active = null; }
    let first = true;
    while (this.queue.length > 0) {
      const head = this.queue[0]!;
      if (!first && INTERESTING.has(head.k)) break;
      first = false;
      const b = this.nextBatch();
      if (!b) break;
      this.finish(b);
    }
    this.settle();
  }

  private nextBatch(): Batch | null {
    const ev = this.queue.shift();
    if (!ev) return null;
    const evs: Ev[] = [ev];
    // moves in the same turn by different actors animate together
    if (ev.k === "move") {
      const ids = new Set([ev.id]);
      while (this.queue.length > 0) {
        const n = this.queue[0]!;
        if (n.k !== "move" || n.t !== ev.t || ids.has(n.id)) break;
        ids.add(n.id);
        evs.push(this.queue.shift()!);
      }
    }
    let dur = 0;
    for (const e of evs) { this.start(e); dur = Math.max(dur, DUR[e.k]); }
    return { evs, ends: this.clock + dur };
  }

  private advanceTurn(t: number): void {
    if (t <= this.turn) return;
    for (let k = this.turn; k < t; k++) {
      for (const o of this.overlays) o.ttl -= 1;
      this.overlays = this.overlays.filter((o) => o.ttl > 0);
    }
    this.turn = t;
    for (const e of this.ents.values()) if (e.glyph && t >= e.glyphTurn + 2) e.glyph = null;
  }

  private start(ev: Ev): void {
    this.advanceTurn(ev.t);
    const c = this.clock;
    switch (ev.k) {
      case "move": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.move = { fx: e.px, fy: e.py, tx: ev.x, ty: ev.y, t0: c, dur: DUR.move };
        if (ev.x !== e.x) e.flip = ev.x < e.x;
        e.x = ev.x; e.y = ev.y;
        if (e.hero) { e.glyph = null; this.visionDirty = true; }
        else if (e.glyph) e.glyph = null;
        break;
      }
      case "attack": {
        const s = this.ents.get(ev.src), d = this.ents.get(ev.dst);
        if (!s) break;
        const dx = d ? Math.sign(d.x - s.x) : 0, dy = d ? Math.sign(d.y - s.y) : 0;
        s.lunge = { dx: dx * 3, dy: dy * 3, t0: c };
        if (dx !== 0) s.flip = dx < 0;
        s.glyph = null;
        break;
      }
      case "hurt": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.hp = ev.hp;
        e.flashUntil = c + DUR.hurt;
        break;
      }
      case "die": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.dying = { t0: c, dur: DUR.die };
        e.glyph = null;
        break;
      }
      case "telegraph": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.glyph = /summon|cast|conjure/.test(ev.what) ? "*" : "!";
        e.glyphTurn = ev.t;
        break;
      }
      case "pickup": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        const at = this.items.filter((i) => i.x === e.x && i.y === e.y);
        const pick = at.find((i) => i.kind === ev.item || i.label === ev.item) ?? at[0];
        if (pick) this.items = this.items.filter((i) => i !== pick);
        break;
      }
      case "use": {
        const h = this.hero;
        if (h) { h.glyph = "*"; h.glyphTurn = ev.t; }
        break;
      }
      case "overlay": {
        const i = this.overlays.findIndex((o) => o.x === ev.x && o.y === ev.y && o.k === ev.ov);
        if (ev.ttl <= 0) { if (i >= 0) this.overlays.splice(i, 1); }
        else if (i >= 0) this.overlays[i]!.ttl = ev.ttl;
        else this.overlays.push({ x: ev.x, y: ev.y, k: ev.ov, ttl: ev.ttl });
        break;
      }
      case "spawn": {
        const e = this.addEntity(ev.e, false);
        e.spawning = { t0: c, dur: DUR.spawn };
        e.fade = 1;
        break;
      }
      case "steal": {
        const h = this.hero;
        if (h) h.flashUntil = c + DUR.hurt;
        break;
      }
      case "ally": {
        const e = this.ents.get(ev.id);
        if (e) e.ally = ev.state === "freed";
        break;
      }
      case "descend":
      case "exit":
        this.fadeTarget = 1;
        break;
      case "callout":
        this.callout = { text: ev.text.slice(0, 24), until: performance.now() + 1000 };
        break;
      case "rule":
      case "fact":
      case "note":
        break;
    }
  }

  // Force a batch's animations to their end states.
  private finish(b: Batch): void {
    for (const ev of b.evs) {
      switch (ev.k) {
        case "move": { const e = this.ents.get(ev.id); if (e) { e.px = e.x; e.py = e.y; e.move = null; } break; }
        case "attack": { const e = this.ents.get(ev.src); if (e) e.lunge = null; break; }
        case "hurt": { const e = this.ents.get(ev.id); if (e) e.flashUntil = 0; break; }
        case "die": this.ents.delete(ev.id); break;
        case "spawn": { const e = this.ents.get(ev.e.id); if (e) { e.spawning = null; e.fade = 0; } break; }
        case "steal": { const h = this.hero; if (h) h.flashUntil = 0; break; }
        case "use": { const h = this.hero; if (h && h.glyph === "*") h.glyph = null; break; }
        case "descend": case "exit": this.fade = 1; break;
        default: break;
      }
    }
  }

  // Per-frame evaluation of tweens from the anim clock.
  private settle(): void {
    const c = this.clock;
    for (const e of this.ents.values()) {
      if (e.move) {
        const raw = Math.min(1, (c - e.move.t0) / e.move.dur);
        const steps = Math.max(1, Math.round(e.move.dur / STEP));
        const p = Math.floor(raw * steps) / steps;
        e.px = e.move.fx + (e.move.tx - e.move.fx) * p;
        e.py = e.move.fy + (e.move.ty - e.move.fy) * p;
        if (raw >= 1) { e.px = e.move.tx; e.py = e.move.ty; e.move = null; }
      }
      if (e.lunge && c - e.lunge.t0 >= DUR.attack) e.lunge = null;
      if (e.dying) {
        const p = Math.min(1, (c - e.dying.t0) / e.dying.dur);
        e.fade = Math.floor(p * 4) / 4; // 4 dissolve steps
      } else if (e.spawning) {
        const p = Math.min(1, (c - e.spawning.t0) / e.spawning.dur);
        e.fade = 1 - Math.floor(p * 4) / 4;
        if (p >= 1) { e.spawning = null; e.fade = 0; }
      }
    }
    if (this.visionDirty) this.computeVision();
  }

  // Current lunge offset in env texels for drawing (2 frames: out, back).
  lungeOffset(e: EntState): [number, number] {
    if (!e.lunge) return [0, 0];
    const p = (this.clock - e.lunge.t0) / DUR.attack;
    return p < 0.5 ? [e.lunge.dx, e.lunge.dy] : [0, 0];
  }

  flashing(e: EntState): boolean { return e.flashUntil > this.clock; }

  // Presentation-only vision: radius 7 line of sight from the hero's logical tile; walls and
  // doors block. Unioned into `seen`; replaces `visible`.
  computeVision(): void {
    this.visionDirty = false;
    const h = this.hero;
    if (!h) return;
    this.visible.fill(0);
    const R = VISION_R;
    for (let dy = -R; dy <= R; dy++) for (let dx = -R; dx <= R; dx++) {
      if (dx * dx + dy * dy > R * R + R) continue;
      const tx = h.x + dx, ty = h.y + dy;
      if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
      if (this.los(h.x, h.y, tx, ty)) { const i = ty * this.w + tx; this.visible[i] = 1; this.seen[i] = 1; }
    }
  }

  private los(x0: number, y0: number, x1: number, y1: number): boolean {
    let dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0);
    const sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1;
    let err = dx + dy, x = x0, y = y0;
    for (;;) {
      if (x === x1 && y === y1) return true;
      if (!(x === x0 && y === y0)) {
        const t = this.tile(x, y);
        if (t === "wall" || t === "door") return false;
      }
      const e2 = 2 * err;
      if (e2 >= dy) { err += dy; x += sx; }
      if (e2 <= dx) { err += dx; y += sy; }
    }
  }
}
