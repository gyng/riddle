// Replay state: the floor as last loaded plus the animated consequences of applied events.
// No three.js here.
//
// Playback model (CUT1 Addendum E): `Ev.t` is a tick; the viewer runs a tick clock at
// 10 ticks/s × speed (0 = pause) and applies every queued event whose t ≤ clock, in order, so
// events with the same t land together and actors never wait for each other. Tweens are timed
// in ticks from the event's own t (not from when it was applied), so late-fed events still line
// up. A move tweens across the tile over the actor's action interval, inferred by peeking at
// the actor's next queued event (default 10 ticks, clamped 3..20), progress quantised to whole
// ticks (10 fps cadence at 1×). Callouts stay on a real-time 1 s cadence.
import { heroKind } from "./look";
import { nameRefs } from "../ui/tokens";   // docs/COPY.md: a rule by its words on the canvas too
import type { Ev, GunSnap, Overlay, Snapshot, Tile, FloorItem, Entity } from "./types";
import { KNOWN_ENTITY_KINDS } from "./palette";

export const TICKS_PER_S = 10;
export const VISION_R = 7; // default sight radius; a snapshot's `vision` (Cut 3: the Deep is 4) overrides per floor
const LUNGE_T = 2, HURT_T = 1.5, DIE_T = 12, SPAWN_T = 2, LEASH_T = 4, SHAKE_T = 3, GLYPH_T = 15;
const FIGHT_HURT_T = 2, FIGHT_SHAKE_T = 3; // Cut 8A: in the fight frame a hit flashes 2 ticks and the screen shakes 3
const CAPTION_MS = 1500, NAME_MAX = 12;   // Cut 8A: the firing row as a caption (real time); names under hostiles
const MOVE_DEFAULT = 10, MOVE_MIN = 3, MOVE_MAX = 20;
const IDLE_TAIL = 6; // ticks after the last event before idle() reports true
const BOSS_FLASH_T = 6; // ticks the palette flashes to `boss_flash` when a boss is first seen/spawned

export type EntState = {
  id: number; kind: string; ally: boolean; hero: boolean; cid: number | null;
  x: number; y: number;            // logical tile
  px: number; py: number;          // render tile pos (float)
  move: { fx: number; fy: number; tx: number; ty: number; t0: number; dur: number } | null;
  lunge: { dx: number; dy: number; t0: number } | null;
  shake: { t0: number } | null;
  flashUntil: number;
  fade: number;                    // 0 = solid, 1 = gone
  dying: { t0: number } | null;
  spawning: { t0: number } | null;
  hp: number; maxHp: number;
  flip: boolean;
  glyph: string | null; glyphT: number; telegraph: string | null;
  ringFrom: number;                // companion ring shown once clock ≥ ringFrom
  remembered: boolean;             // Cut 4 §3: pursued but unseen; drawn dimmed at its last seen tile, never tweened
  fresh?: boolean;                 // preloaded from a snapshot before its events: the first move places it
  neutral: boolean;                // a captive: not a hostile, so the leading camera (index.ts) ignores it
  boss: boolean;
  elite?: "shielded" | "frenzied" | "leeching";
  name: string;                    // Cut 8A: the label under a hostile in the fight frame (`goblin`, `Morog`), ≤ NAME_MAX chars
};

/** Cut 8A: what a hostile is called under its sprite: its given name, else its kind as words; over NAME_MAX chars the last word. */
export function entityName(e: { kind: string; name?: string }): string {
  if (e.name && e.name.length <= NAME_MAX) return e.name;
  const words = e.kind.replace(/^boss_/, "").split("_");
  const full = words.join(" ");
  return (full.length <= NAME_MAX ? full : words[words.length - 1]!).slice(0, NAME_MAX);
}

/** The core's foe kinds (defs.rs MONSTERS, the heroes' and the captive's aside) — a row's caption strips a title no drawn foe answers to. */
const FOE_KINDS = [...KNOWN_ENTITY_KINDS.filter((k) => !k.startsWith("hero_") && k !== "captive"), "spectral_blade", "spectral_hound", "goblin_captain",
  "iron_golem", "forge_imp", "bell_sentinel", "slag_crawler", "smith", "lurker", "deep_eel", "cave_troll", "siren"];
export type Callout = { text: string; until: number; t?: number }; // real-time ms; t: the event's tick (a seek drops the ones before it)
export type ScreenShake = { t0: number; amp: number };  // Cut 8A: ticks; amp in env texels
export type Leash = { from: number; to: number; t0: number; ok: boolean };
export type Projectile = { path: [number, number][]; t0: number };

const HERO_FALLEN_FADE = 0.5;   // QA e75ec29: the fallen hero's body stays, half dimmed
const INTERESTING = new Set<string>(["attack", "die", "telegraph", "use", "exit", "tame"]);
/** The global fade at the run's end: the last frame stays lit, dimmed at most this much. */
export const EXIT_DIM = 0.3;
export const PROPS = new Set<string>(["shrine", "vault", "vault_open", "nest"]); // Cut 5 §4: floor-standing props in `tiles`

/** Cut 26 §2: a fork floor's down stair and the lane it opens (`fens`), `taken` on the stair the set's route takes. */
export type StairInfo = { x: number; y: number; biome?: string; taken?: boolean };
export class ReplayState {
  w = 0; h = 0; biome = "warrens"; depth = 1;
  /** Cut 26 §2: a fork floor's two stairs (`Snapshot.fork`): the floor's own down stairs, the lane the route takes (`taken`), and the
   *  other stair drawn beside it (`other`) — empty on a floor with one stair */
  stairs: StairInfo[] = [];
  tiles: Tile[] = [];
  seen: Uint8Array = new Uint8Array(0);
  visible: Uint8Array = new Uint8Array(0);
  overlays: Overlay[] = [];
  items: FloorItem[] = [];
  ents = new Map<number, EntState>();
  gun: GunSnap | null = null;
  heroId = -1;
  clock = 0;          // ticks (float)
  speed = 1;
  fadeTarget = 0;     // global fade (0 = visible, 1 = dark; EXIT_DIM at the run's end)
  fade = 0;
  callout: Callout | null = null;
  leash: Leash | null = null;
  projectiles: Projectile[] = [];
  loaded = false;
  visionDirty = true;
  /** Tiles the engine reported visible in the latest step (unioned into `visible` after each LOS pass). */
  engineVisible = new Uint8Array(0);
  cameraSnap = false; // index.ts snaps the camera to the hero and clears this
  bossFlashUntil = -Infinity; // clock < this → index.ts renders with the `boss_flash` palette
  vision = VISION_R;  // presentation LOS radius and the fog bands (index.ts) follow the floor's vision
  nestWoken = new Set<number>(); // Cut 5 §4: tile indices of nests shown awake (a `nest` fact, or a hostile spawned adjacent)
  // Cut 8A: the fight frame (index.ts sets it). While up: hits flash longer, hurts shake the screen, the firing row is a caption.
  fight = false;
  caption: Callout | null = null;
  private iconRows = new Set<number>();
  private meaningfulRows = new Set<number>();
  private tacticTick = -Infinity;
  private heroAttack: { t: number; dst: number } | null = null;
  tacticTarget: { t: number; id: number } | null = null;
  setTacticRows(rows: number[], meaningful: number[]): void { this.iconRows = new Set(rows); this.meaningfulRows = new Set(meaningful); }
  screenShake: ScreenShake | null = null;
  /** juice (docs/JUICE.md): every event applied in play, for render-only feedback (fx.ts); never called while a seek or a skip
   *  replays the past (`bulk`) */
  onEvent: ((ev: Ev) => void) | null = null;
  private bulk = false;
  private ended = false;
  private snap: Snapshot | null = null;
  private queue: Ev[] = [];
  private log: Ev[] = [];     // applied since load, in order (for seek)
  private lastT = -Infinity;
  private wholeTick = 0;

  load(s: Snapshot): void {
    this.snap = s;
    this.log = [];
    this.queue = [];
    this.reset(s);
    this.cameraSnap = true;
  }

  private reset(s: Snapshot): void {
    this.gun = s.hero.gun ? { ...s.hero.gun } : null;
    this.w = s.w; this.h = s.h; this.biome = s.biome; this.depth = s.depth;
    // (the fork's other stair is not a tile of the floor: it is drawn at (x, y) beside the real one — render-only, never game truth)
    this.stairs = [];
    this.vision = s.vision ?? VISION_R;
    this.tiles = s.tiles.slice();
    if (s.fork) {
      const real = s.tiles.indexOf("stairs_down"), i = s.fork.y * s.w + s.fork.x;
      if (real >= 0) this.stairs.push({ x: real % s.w, y: Math.floor(real / s.w), biome: s.fork.taken, taken: true });
      if (i >= 0 && i < this.tiles.length && i !== real && this.tiles[i] !== "wall") { this.tiles[i] = "stairs_down"; this.stairs.push({ x: s.fork.x, y: s.fork.y, biome: s.fork.other }); }
    }
    this.seen = Uint8Array.from(s.seen, (b) => (b ? 1 : 0));
    this.visible = Uint8Array.from(s.visible, (b) => (b ? 1 : 0));
    this.engineVisible = Uint8Array.from(s.visible, (b) => (b ? 1 : 0));
    this.overlays = s.overlays.map((o) => ({ ...o }));
    this.items = s.items.map((i) => ({ ...i }));
    this.ents.clear();
    this.heroId = s.hero.id;
    this.clock = s.turn;
    this.wholeTick = Math.floor(s.turn);
    this.lastT = -Infinity;
    this.addEntity(s.hero, true);
    for (const e of s.entities) this.addEntity(e, false);
    this.fade = this.fadeTarget = 0;
    this.bossFlashUntil = -Infinity;
    this.callout = null;
    this.caption = null;
    this.tacticTick = -Infinity; this.heroAttack = null; this.tacticTarget = null;
    this.ended = false;
    this.screenShake = null;
    this.leash = null;
    this.projectiles = [];
    this.nestWoken.clear();
    this.loaded = true;
    this.computeVision();
  }

  /** QA 1a2a4a9 (P: `R2 ATTACK GOBLIN` over a conjurer and a blade; `R5 ATTACK GOBLIN` with no goblin drawn): the core's row text names
   *  the target by its kind's title (`attack goblin conjurer`) — the caption names it as its tag does (`attack conjurer`), and a target
   *  no hostile on this floor answers to is left out (`attack`). A tail with no kind's title in it passes through. */
  private targetAsTagged(tail: string): string {
    const low = tail.toLowerCase();
    let best: EntState | null = null, len = 0, named = 0;
    for (const e of this.ents.values()) {
      if (e.hero || e.ally || e.neutral) continue;
      const title = e.kind.replace(/^boss_/, "").replace(/_/g, " ").toLowerCase();
      if (!low.endsWith(` ${title}`)) continue;
      named = Math.max(named, title.length);
      if (!e.dying && !e.remembered && title.length > len) { best = e; len = title.length; }
    }
    if (best) return `${tail.slice(0, tail.length - len).trim()} ${best.name}`;
    for (const k of FOE_KINDS) { const title = k.replace(/_/g, " "); if (low.endsWith(` ${title}`)) named = Math.max(named, title.length); }
    // a title the floor knew (dead or out of sight) or a word the renderer cannot place: the verb alone when a foe word ends it
    if (named) return tail.slice(0, tail.length - named).trim();
    return tail;
  }
  private addEntity(e: Entity, hero: boolean): EntState {
    const st: EntState = {
      id: e.id, kind: hero ? heroKind(e.kind) : e.kind, ally: !!e.ally, hero, cid: e.cid ?? null, x: e.x, y: e.y, px: e.x, py: e.y,
      move: null, lunge: null, shake: null, flashUntil: -Infinity, fade: 0, dying: null, spawning: null,
      hp: e.hp, maxHp: e.max_hp, flip: false, glyph: e.telegraph ? "!" : null, glyphT: this.clock, telegraph: e.telegraph ?? null,
      ringFrom: -Infinity, remembered: !!e.remembered,
      neutral: e.kind === "captive" || (e.tags ?? []).includes("captive"),
      boss: (e.tags ?? []).includes("boss"),
      name: entityName(e), elite:e.modifiers?.elite,
    };
    this.ents.set(e.id, st);
    return st;
  }

  get hero(): EntState | undefined { return this.ents.get(this.heroId); }

  tile(x: number, y: number): Tile | undefined {
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return undefined;
    return this.tiles[y * this.w + x];
  }

  apply(evs: Ev[]): void {
    if (evs.length === 0) return;
    const wasEmpty = this.queue.length === 0;
    for (const e of evs) this.queue.push(e);
    // dead air: if the stream resumes far ahead of the clock, jump to just before it
    if (wasEmpty && this.queue[0]!.t > this.clock + 30) this.clock = this.queue[0]!.t - 1;
  }

  /** Cut 4 §3: adopt `remembered` from a step's snapshot of this floor. A flagged entity sits at the snapshot's
   *  (last seen) tile with no tween; one the snapshot no longer flags is unflagged and snapped to where the
   *  snapshot puts it, so the moves that follow tween from the right tile. Entities the viewer has never seen
   *  (remembered or newly in view) are added at the snapshot's tile. */
  sync(s: Snapshot): void {
    if (!this.loaded || s.depth !== this.depth) return;
    if (s.vision !== undefined && s.vision !== this.vision) { this.vision = s.vision; this.visionDirty = true; } // a lantern picked up
    // The engine's visibility is truth; the presentation LOS (computed from the hero's tweened tile, which lags)
    // only ever adds to it. Without this union a foe that hits the hero from a tile the lagging LOS has not
    // reached yet is culled from the draw (cohort 5: "foes are never drawn" in a fight frame).
    if (s.visible.length === this.visible.length) { for (let i = 0; i < s.visible.length; i++) { const v = s.visible[i] ? 1 : 0; this.engineVisible[i] = v; if (v) this.seen[i] = 1; } this.visionDirty = true; }
    // Cut 5 §4: a prop that changed (vault → vault_open) or a tile the floor rewrote lands as is; no tween
    if (s.tiles.length === this.tiles.length) for (let i = 0; i < s.tiles.length; i++) if (s.tiles[i] !== this.tiles[i]) this.tiles[i] = s.tiles[i]!;
    const flagged = new Map<number, Entity>();
    for (const e of s.entities) if (e.id !== this.heroId) flagged.set(e.id, e);
    for (const [id, e] of this.ents) {
      if (e.hero || e.dying) continue;
      const se = flagged.get(id);
      const rem = !!se?.remembered;
      if (rem === e.remembered) continue;
      e.remembered = rem;
      if (se) { e.x = e.px = se.x; e.y = e.py = se.y; }
      e.move = null; e.lunge = null; e.glyph = null;
    }
    // newcomers: a remembered foe, or one the core lists for the first time (its snapshot carries only entities in
    // view, and first sight has no event of its own: a sleeper in the next room would otherwise never be drawn)
    for (const se of flagged.values()) if (!this.ents.has(se.id)) this.addEntity(se, false);
    // likewise floor items: the snapshot lists the seen ones, so an item first seen after the load lands here
    const known = new Set(this.items.map((i) => i.id));
    for (const it of s.items) if (!known.has(it.id)) this.items.push({ ...it });
  }

  /** Add entities (and items) the viewer has never held, at their snapshot tile, WITHOUT touching known ones.
   *  Called before a batch's events so a monster that appears and fights inside the batch receives its own
   *  move/attack/hurt events (cohort 5: foes attacked and killed between two syncs were never drawn). */
  preload(s: Snapshot): void {
    if (!this.loaded || s.depth !== this.depth) return;
    for (const se of s.entities) if (se.id !== this.heroId && !this.ents.has(se.id)) { const e = this.addEntity(se, false); e.fresh = true; }
    const known = new Set(this.items.map((i) => i.id));
    for (const it of s.items) if (!known.has(it.id)) this.items.push({ ...it });
  }

  idle(): boolean { return this.queue.length === 0 && this.clock >= this.lastT + IDLE_TAIL; }
  pending(): number { return this.queue.length; }
  tickNow(): number { return Math.floor(this.clock); }

  // Advance the tick clock and apply due events. dt = real ms.
  tick(dt: number, now: number): void {
    if (!this.loaded) return;
    this.clock += (dt / 1000) * TICKS_PER_S * this.speed;
    const fs = (dt / 350) * Math.max(this.speed, 0.5);
    this.fade += Math.sign(this.fadeTarget - this.fade) * Math.min(fs, Math.abs(this.fadeTarget - this.fade));
    if (this.callout && now > this.callout.until) this.callout = null;
    if (this.caption && now > this.caption.until) this.caption = null;
    if (this.speed <= 0) return;
    this.drain(this.clock);
    this.settle();
  }

  // Apply every queued event with t ≤ upTo.
  private drain(upTo: number): void {
    while (this.queue.length > 0 && this.queue[0]!.t <= upTo) {
      const ev = this.queue.shift()!;
      this.log.push(ev);
      this.applyOne(ev);
    }
  }

  // Fast-forward: apply instantly until the head is "interesting", then start playing there.
  skipToEvent(): void {
    let first = true;
    this.bulk = true;
    while (this.queue.length > 0) {
      const head = this.queue[0]!;
      if (!first && INTERESTING.has(head.k)) break;
      first = false;
      this.log.push(this.queue.shift()!);
      this.applyOne(head);
    }
    this.bulk = false;
    this.clock = this.queue.length > 0 ? this.queue[0]!.t : this.lastT + IDLE_TAIL;
    this.wholeTick = Math.floor(this.clock);
    this.finishTweens();
    this.settle();
  }

  // Tick scrub: rebuild from the last loaded snapshot and replay everything with t ≤ t.
  seek(t: number): void {
    if (!this.snap) return;
    const all = this.log.concat(this.queue);
    this.log = [];
    this.queue = [];
    this.reset(this.snap);
    this.bulk = true;
    for (const ev of all) {
      if (ev.t <= t) { this.log.push(ev); this.applyOne(ev); }
      else this.queue.push(ev);
    }
    this.bulk = false;
    this.clock = t;
    this.wholeTick = Math.floor(t);
    // the replayed past sets no text: a caption or callout from an event before the landing tick is stale (QA on 3d71c33: a chain
    // clip opened on `R5 ATTACK GOBLIN`, a row fired before its window, over the den it links to)
    if (this.caption && (this.caption.t ?? -Infinity) < t) this.caption = null;
    if (this.callout && (this.callout.t ?? -Infinity) < t) this.callout = null;
    this.settle();
    this.cameraSnap = true;
  }

  // The actor's action interval: ticks until its next own action in the queue (move, attack,
  // telegraph, pickup, tame), clamped; default 10 (base speed) when nothing is queued yet.
  private moveDur(id: number, t: number): number {
    for (const e of this.queue) {
      if (e.t <= t) continue;
      if (e.t > t + MOVE_MAX) break;
      const actor = e.k === "attack" ? e.src : e.k === "move" || e.k === "telegraph" || e.k === "pickup" ? e.id : e.k === "tame" ? this.heroId : -1;
      if (actor === id) return Math.max(MOVE_MIN, e.t - t);
    }
    return MOVE_DEFAULT;
  }

  private applyOne(ev: Ev): void {
    const t = ev.t;
    if (!this.bulk) this.onEvent?.(ev);
    this.lastT = Math.max(this.lastT, t);
    // `see` is not in the wire types yet (engine track): accept {e: Entity} or {id} and flash for a boss
    if ((ev as { k: string }).k === "see") {
      const sv = ev as unknown as { e?: Entity; id?: number };
      const tags = sv.e?.tags ?? (sv.id !== undefined ? this.snap?.entities.find((x) => x.id === sv.id)?.tags : undefined);
      if (tags?.includes("boss")) this.bossFlashUntil = t + BOSS_FLASH_T;
      return;
    }
    switch (ev.k) {
      case "gun": this.gun = ev.state ? { ...ev.state } : null; break;
      case "move": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        // start from wherever the previous tween would be at t (chained moves stay continuous); a remembered
        // foe is never tweened (Cut 4 §3): it lands on the tile
        if (e.remembered || e.fresh) { e.move = null; e.px = ev.x; e.py = ev.y; e.fresh = false; }   // preloaded: its first move places it, no tween from the end tile
        else {
          const [sx, sy] = this.posAt(e, t);
          e.move = { fx: sx, fy: sy, tx: ev.x, ty: ev.y, t0: t, dur: this.moveDur(ev.id, t) };
        }
        if (ev.x !== e.x) e.flip = ev.x < e.x;
        e.x = ev.x; e.y = ev.y;
        e.glyph = null;
        if (e.hero) this.visionDirty = true;
        break;
      }
      case "attack": {
        if (ev.src === this.heroId && !this.ended) {
          this.heroAttack = { t, dst: ev.dst };
          if (this.tacticTick === t) this.tacticTarget = { t, id: ev.dst };
        }
        const s = this.ents.get(ev.src), d = this.ents.get(ev.dst);
        if (!s) break;
        const dx = d ? Math.sign(d.x - s.x) : 0, dy = d ? Math.sign(d.y - s.y) : 0;
        s.lunge = { dx: dx * 3, dy: dy * 3, t0: t };
        if (dx !== 0) s.flip = dx < 0;
        s.glyph = null;
        break;
      }
      case "recover": {
        const e=this.ents.get(ev.id);
        if(e&&!e.dying) e.hp=ev.hp;
        break;
      }
      case "hurt": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.hp = ev.hp;
        e.flashUntil = t + (this.fight ? FIGHT_HURT_T : HURT_T);
        // Cut 8A: the screen shakes ±2 env texels for the hero being hurt, ±1 for anyone else; a bigger one wins
        if (this.fight && ev.dmg > 0) {
          const amp = e.hero ? 2 : 1;
          if (!this.screenShake || this.clock >= this.screenShake.t0 + FIGHT_SHAKE_T || amp >= this.screenShake.amp) this.screenShake = { t0: t, amp };
        }
        break;
      }
      case "rule": {
        if (!this.ended && ev.verb.v === 'attack' && this.meaningfulRows.has(ev.row)) {
          this.tacticTick = t;
          if (this.heroAttack?.t === t) this.tacticTarget = { t, id: this.heroAttack.dst };
        }
        // Cut 8A: the firing row as a caption at the top of the fight frame: `R2 attack goblin`; a trait deviation reads as
        // its own text (`cowardly > retreat`); chores (row -2) stay silent (pillar 2)
        if (this.ended || ev.row < -1 || this.iconRows.has(ev.row) || ev.verb.v === "gunner_tactic") break;
        const tail = ev.text.includes("→") ? ev.text.slice(ev.text.lastIndexOf("→") + 1).trim() : ev.text;
        // QA 23ed91f (L: `R4 PACK BREAK GOBLIN`, 4 words): a callout is ≤ 3 words — the row number and at most two of the verb's
        // (the target goes first: `R4 pack break`, `R2 attack goblin`)
        const text = (ev.row >= 0 ? capWords(this.targetAsTagged(tail), 3) : ev.text.replace(/→/g, ">")).slice(0, 24);
        this.caption = { text, until: performance.now() + CAPTION_MS, t };
        break;
      }
      case "die": {
        if (ev.id === this.heroId) { this.ended = true; this.caption = null; }
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.dying = { t0: t };
        e.glyph = null;
        break;
      }
      case "telegraph": {
        const e = this.ents.get(ev.id);
        if (!e) break;
        e.glyph = /summon|cast|conjure/.test(ev.what) ? "*" : "!";
        e.glyphT = t; e.telegraph = ev.what;
        break;
      }
      case "projectile":
        if (ev.path.length > 0) this.projectiles.push({ path: ev.path.map(([x, y]) => [x, y]), t0: t });
        break;
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
        if (h) { h.glyph = "*"; h.glyphT = t - GLYPH_T + 4; } // brief
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
        e.spawning = { t0: t };
        e.fade = 1;
        if (ev.e.tags?.includes("boss")) this.bossFlashUntil = t + BOSS_FLASH_T;
        if (!e.ally && !e.neutral) this.wakeNests(e.x, e.y);
        break;
      }
      case "fact":
        if (/nest/.test(ev.fact)) this.wakeNests();
        break;
      case "steal": {
        const h = this.hero;
        if (h) h.flashUntil = t + HURT_T;
        break;
      }
      case "ally": {
        const e = this.ents.get(ev.id);
        if (e) { e.ally = ev.state === "freed"; e.ringFrom = t; }
        break;
      }
      case "descend":
        this.fadeTarget = 1;
        break;
      case "exit":
        this.ended = true; this.caption = null;
        // the run's end keeps its last frame lit: the corpse's floor, dimmed a little, never black (QA on 50bb162:
        // "map is black at death"); the next floor's load resets the fade
        this.fadeTarget = EXIT_DIM;
        break;
      case "callout":
        if (/^Gunner\s+✗/.test(ev.text)) break; // Ordinary automatic fallback is not a combat failure.
        if (ev.text === "choose one") break;   // Cut 19 §1: the cage is the watch's beat (`took mail`), never a `CHOOSE ONE` over the hero
        // Cut 22 (AH: the boss moment's "clutter of overlapping text"): a boss's break is the watch's beat (`WARLORD BREAKS` on the line
        // under the fight) — one line wins, never a second `WARLORD BREAKS` over the hero at once
        if (/^(?:the )?[a-z]+ breaks\.?$/i.test(ev.text.trim())) break;
        this.callout = { text: nameRefs(ev.text).slice(0, 24), until: performance.now() + 1000, t };
        break;
      case "tame": {
        // leash arc hero → target over LEASH_T ticks, then flash + ring (ok) or shake (fail)
        const h = this.hero, e = this.ents.get(ev.id);
        if (!h || !e) break;
        this.leash = { from: h.id, to: e.id, t0: t, ok: ev.ok };
        if (ev.ok) { e.flashUntil = t + LEASH_T + 2; e.ally = true; e.ringFrom = t + LEASH_T; e.glyph = null; }
        else e.shake = { t0: t + LEASH_T };
        if (e.x !== h.x) h.flip = e.x < h.x;
        break;
      }
      case "max_hp": {   // QA 1a2a4a9: hunger bites the max — the hero's bar reads the new ceiling
        const e = this.ents.get(ev.id); if (e) { e.maxHp = ev.max; e.hp = Math.min(e.hp, ev.max); }   // (the core clamps the hp with no hp event)
        break;
      }
      case "hatch":
        this.callout = { text: ev.kind.replace(/_/g, " ").slice(0, 24), until: performance.now() + 1000, t };
        break;
      default:
        break; // rule, fact, note, level, …: nothing to draw
    }
  }

  // Cut 5 §4: nests within one tile of (x, y) wake; with no point, every nest on the floor (a `nest` fact was learned).
  private wakeNests(x?: number, y?: number): void {
    for (let ty = 0; ty < this.h; ty++) for (let tx = 0; tx < this.w; tx++) {
      if (this.tiles[ty * this.w + tx] !== "nest") continue;
      if (x === undefined || y === undefined || Math.max(Math.abs(tx - x), Math.abs(ty - y)) <= 1) this.nestWoken.add(ty * this.w + tx);
    }
  }

  // Where an entity's render position is at tick t (its current tween evaluated at t).
  private posAt(e: EntState, t: number): [number, number] {
    if (!e.move) return [e.px, e.py];
    const p = Math.max(0, Math.min(1, (t - e.move.t0) / e.move.dur));
    return [e.move.fx + (e.move.tx - e.move.fx) * p, e.move.fy + (e.move.ty - e.move.fy) * p];
  }

  // Force all tweens to their end states (after a skip).
  private finishTweens(): void {
    for (const e of this.ents.values()) {
      if (e.move) { e.px = e.move.tx; e.py = e.move.ty; e.move = null; }
      e.lunge = null; e.shake = null;
      if (e.spawning) { e.spawning = null; e.fade = 0; }
    }
    for (const [id, e] of this.ents) if (e.dying) { if (e.hero) e.fade = HERO_FALLEN_FADE; else this.ents.delete(id); }
    this.projectiles = [];
    this.leash = null;
    this.screenShake = null;
    if (this.fadeTarget > 0) this.fade = this.fadeTarget;
  }

  // Per-frame evaluation of tweens from the tick clock.
  private settle(): void {
    const c = this.clock;
    // overlay TTLs are in ticks
    const wt = Math.floor(c);
    if (wt > this.wholeTick) {
      const n = wt - this.wholeTick;
      for (const o of this.overlays) o.ttl -= n;
      this.overlays = this.overlays.filter((o) => o.ttl > 0);
      this.wholeTick = wt;
    }
    for (const [id, e] of this.ents) {
      if (e.move) {
        const raw = Math.min(1, (c - e.move.t0) / e.move.dur);
        const p = raw >= 1 ? 1 : Math.floor((c - e.move.t0)) / e.move.dur; // whole-tick steps
        e.px = e.move.fx + (e.move.tx - e.move.fx) * p;
        e.py = e.move.fy + (e.move.ty - e.move.fy) * p;
        if (raw >= 1) e.move = null;
      }
      if (e.lunge && c - e.lunge.t0 >= LUNGE_T * 2) e.lunge = null;
      if (e.shake && c - e.shake.t0 >= SHAKE_T) e.shake = null;
      if (e.glyph && c - e.glyphT >= GLYPH_T) e.glyph = null;
      if (e.dying) {
        const p = Math.min(1, (c - e.dying.t0) / DIE_T);
        // QA e75ec29 (Q: "final frame shows no hero sprite at 0/36 … the body stays where he fell"): the fallen hero dims to
        // HERO_FALLEN_FADE and stays drawn where he fell through the walk-out; only foes dissolve away
        e.fade = e.hero ? Math.min(HERO_FALLEN_FADE, Math.floor(p * 4) / 4) : Math.floor(p * 4) / 4;
        if (p >= 1 && !e.hero && !e.boss) this.ents.delete(id);   // gfx round 10: a boss's body stays (the renderer draws his death pose, or nothing)
      } else if (e.spawning) {
        const p = Math.min(1, (c - e.spawning.t0) / SPAWN_T);
        e.fade = 1 - Math.floor(p * 4) / 4;
        if (p >= 1) { e.spawning = null; e.fade = 0; }
      }
    }
    this.projectiles = this.projectiles.filter((p) => c < p.t0 + p.path.length);
    if (this.leash && c - this.leash.t0 >= LEASH_T + SHAKE_T) this.leash = null;
    if (this.visionDirty) this.computeVision();
  }

  // Current lunge/shake offset in env texels (lunge: LUNGE_T ticks out, LUNGE_T back;
  // shake: ±2 texels alternating each half tick).
  lungeOffset(e: EntState): [number, number] {
    if (e.shake && this.clock >= e.shake.t0) {
      const f = Math.floor((this.clock - e.shake.t0) * 2);
      return [f % 2 === 0 ? 2 : -2, 0];
    }
    if (!e.lunge) return [0, 0];
    return this.clock - e.lunge.t0 < LUNGE_T ? [e.lunge.dx, e.lunge.dy] : [0, 0];
  }

  flashing(e: EntState): boolean { return e.flashUntil > this.clock; }

  // Cut 8A: the fight frame's screen shake in env texels, alternating each half tick for FIGHT_SHAKE_T ticks.
  shakeOffset(): [number, number] {
    const s = this.screenShake;
    if (!s || this.clock < s.t0 || this.clock >= s.t0 + FIGHT_SHAKE_T) return [0, 0];
    const f = Math.floor((this.clock - s.t0) * 2);
    return [f % 2 === 0 ? s.amp : -s.amp, f % 4 === 1 ? s.amp : f % 4 === 3 ? -s.amp : 0];
  }
  ringShown(e: EntState): boolean { return e.ally && !e.hero && this.clock >= e.ringFrom; }
  tacticMarked(e: EntState): boolean { return !this.ended && !e.dying && !e.remembered && this.tacticTarget?.id === e.id && this.clock >= this.tacticTarget.t && this.clock < this.tacticTarget.t + 5; }

  // Leash progress 0..1 while the arc is being drawn, or null.
  leashProgress(): number | null {
    if (!this.leash) return null;
    return Math.min(1, (this.clock - this.leash.t0) / LEASH_T);
  }

  // Projectile positions in tile coords (float), 1 tile per tick along the path.
  projectilePositions(): [number, number][] {
    const out: [number, number][] = [];
    for (const p of this.projectiles) {
      const u = this.clock - p.t0;
      if (u < 0) continue;
      const i = Math.min(p.path.length - 1, Math.floor(u));
      const a = p.path[i]!, b = p.path[Math.min(p.path.length - 1, i + 1)]!;
      const f = Math.min(1, u - i);
      out.push([a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f]);
    }
    return out;
  }

  // Presentation-only vision: radius 7 line of sight from the hero's logical tile; walls and
  // doors block. Unioned into `seen`; replaces `visible`.
  computeVision(): void {
    this.visionDirty = false;
    const h = this.hero;
    if (!h) return;
    this.visible.fill(0);
    const R = this.vision;
    for (let dy = -R; dy <= R; dy++) for (let dx = -R; dx <= R; dx++) {
      if (dx * dx + dy * dy > R * R + R) continue;
      const tx = h.x + dx, ty = h.y + dy;
      if (tx < 0 || ty < 0 || tx >= this.w || ty >= this.h) continue;
      if (this.los(h.x, h.y, tx, ty)) { const i = ty * this.w + tx; this.visible[i] = 1; this.seen[i] = 1; }
    }
    if (this.engineVisible.length === this.visible.length) for (let i = 0; i < this.visible.length; i++) if (this.engineVisible[i]) this.visible[i] = 1;
  }

  private los(x0: number, y0: number, x1: number, y1: number): boolean {
    const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0);
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

/** The first `n` words of a caption's verb (`pack break goblin` → `pack break`). */
export const capWords = (s: string, n: number): string => s.trim().split(/\s+/).slice(0, n).join(" ");
