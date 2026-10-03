// Cut 30 §3 — the town hub (docs/TOWN.md, targets art/ui/targets/town/): the camp as a place. A fixed tile map with named plots and
// a path graph; `townState(lineage, absence)` turns the core's `Lineage.town` into what stands and what walks (buildings with their
// look, the next plot staked, the hero's loop, the absence's parties walking out of the mouth, pets, markers); the view draws it in
// the pixel pipeline's register — one low-res target of town texels (instanced quads, `layers.ts`), upscaled k× with nearest
// sampling, lit by a small light pass (the fire, the mouth's torches, the forge, windows at night; day and night from the local
// clock). The Canvas-2D fallback (`view2d.ts` `drawTown2D`) draws the same frame while the GL context is lost or when none exists.
//
// Coordinates: texels, x right, y DOWN the map (a tile is 16 texels); three's world is (x, −y). Every sprite is anchored at its foot
// (bottom centre) and depth-sorted by foot y. Walkers are cosmetic: seeded by the lineage's seed + the local day, never read back.
// Art never blocks the scene: every sprite id has a primitive fallback (a coloured block in the thing's shape) until the atlas has it.
import * as THREE from "three";
import { QuadLayer, BAYER_GLSL } from "./layers";
import { drawTown2D } from "./view2d";
import type { Lineage, ReturnReport } from "../engine/types";
import LIGHTS from "./town_lights.json";   // a copy of art/town_lights.json (`python3 art/town_lights.py`): each sprite's painted warm emitters

export const TILE = 16;
export const MAP_W = 56, MAP_H = 40;
const CX = 28, OY = 5;   // the layout's centre column; its rows start OY tiles down (the cliff above the mouth)
const T = (x: number, y: number): Pt => ({ x: x * TILE, y: (y + OY) * TILE });
export type Pt = { x: number; y: number };

// ---- the map: plots, paths -------------------------------------------------------------------------------------------------

export const BUILDINGS = ["blacksmith", "storehouse", "kennel", "bank"] as const;
export type BuildingId = (typeof BUILDINGS)[number];
export type PlotId = "mouth" | "fire" | "tent" | "crate" | "board" | BuildingId;
/** each plot's foot (texels) and its drawn height */
export const PLOTS: Record<PlotId, Pt & { h: number }> = {
  mouth: { ...T(CX, 6.6), h: 64 },
  fire: { ...T(CX, 14.7), h: 16 },
  tent: { ...T(CX + 3.3, 14.0), h: 32 },
  crate: { ...T(CX + 5.0, 14.9), h: 16 },
  board: { ...T(CX - 2.2, 7.6), h: 28 },   // Cut 30 §5: the notice board by the mouth (the quest), from the Warlord slain
  blacksmith: { ...T(CX - 4.6, 10.7), h: 64 },
  bank: { ...T(CX + 4.6, 10.7), h: 64 },
  storehouse: { ...T(CX - 4.5, 19.6), h: 64 },
  kennel: { ...T(CX + 4.7, 19.8), h: 64 },
};
const NODES: Record<string, Pt> = {
  in: T(CX, 6.3), mouth: T(CX, 7.5), cross: T(CX, 11.6), smith: T(CX - 4.4, 11.0), bank: T(CX + 4.4, 11.0),
  fireN: T(CX, 13.1), fireW: T(CX - 1.5, 14.95), fireE: T(CX + 1.5, 14.95), tent: T(CX + 2.45, 14.45), crate: T(CX + 4.3, 15.55),
  fireS: T(CX, 16.7), south: T(CX, 18.6), store: T(CX - 4.3, 19.9), kennel: T(CX + 4.3, 20.1), exit: T(CX, 25),
};
const EDGES: [string, string, number][] = [   // [a, b, half-width of the dirt in tiles]
  ["in", "mouth", 0.9], ["mouth", "cross", 0.8], ["cross", "smith", 0.5], ["cross", "bank", 0.5], ["cross", "fireN", 0.8],
  ["fireN", "fireW", 0.55], ["fireN", "fireE", 0.55], ["fireE", "tent", 0.5], ["tent", "crate", 0.45], ["fireW", "fireS", 0.55],
  ["fireE", "fireS", 0.55], ["fireS", "south", 0.7], ["south", "store", 0.5], ["south", "kennel", 0.5], ["south", "exit", 0.7],
];
const ADJ = new Map<string, string[]>();
for (const [a, b] of EDGES) { ADJ.set(a, [...(ADJ.get(a) ?? []), b]); ADJ.set(b, [...(ADJ.get(b) ?? []), a]); }
const dist = (a: Pt, b: Pt): number => Math.hypot(a.x - b.x, a.y - b.y);
/** the shortest route between two nodes (Dijkstra over the path graph), as points */
export function route(from: string, to: string): Pt[] {
  const d = new Map<string, number>([[from, 0]]), prev = new Map<string, string>(), open = new Set([from]);
  while (open.size) {
    let u = ""; let best = Infinity;
    for (const n of open) if ((d.get(n) ?? Infinity) < best) { best = d.get(n)!; u = n; }
    open.delete(u);
    if (u === to) break;
    for (const v of ADJ.get(u) ?? []) {
      const nd = best + dist(NODES[u]!, NODES[v]!);
      if (nd < (d.get(v) ?? Infinity)) { d.set(v, nd); prev.set(v, u); open.add(v); }
    }
  }
  const out: string[] = [to];
  while (out[0] !== from) { const p = prev.get(out[0]!); if (!p) return [NODES[from]!, NODES[to]!]; out.unshift(p); }
  return out.map((n) => NODES[n]!);
}
const nearestNode = (p: Pt): string => { let b = "fireN", bd = Infinity; for (const [k, n] of Object.entries(NODES)) { if (k === "in" || k === "exit") continue; const e = dist(p, n); if (e < bd) { bd = e; b = k; } } return b; };

// ---- the state: what stands and what walks ----------------------------------------------------------------------------------

export type Marker = { at: PlotId | "chest"; kind: "coin" | "rune" | "sword" | "chest"; label?: string };
/** Cut 30.5 (docs/AUTOMATION_TREE.md §3C): the haul chest by the mouth (before the porter) and the workers at their posts */
export type ChestState = { state: "closed" | "full" | "open"; gold: number; count?: number; need?: number };
export type WorkerSpot = { id: string; post: string; lit: boolean; price?: number; paused: boolean; rank: number };
/** the chest's foot, beside the path under the mouth */
export const CHEST_AT: Pt & { h: number } = { ...T(CX + 1.6, 8.3), h: 12 };
/** a worker's spot (texels, its foot): by id, else by its post's plot — art/town-ids.md's posts (the sheet's units × ~0.6 onto the map's
 *  tiles), the herald and the scout moved to the map's own board (left of the mouth) */
const WORKER_AT: Record<string, Pt> = {
  quartermaster: T(CX + 4.9, 15.7), porter: T(CX + 1.1, 10.2), scout: T(CX - 1.1, 8.4), armourer: T(CX - 2.1, 19.4), apprentice: T(CX - 1.9, 11.6),
  keeper: T(CX - 5.9, 19.8), clerk: T(CX + 2.8, 11.7), drillmaster: T(CX + 1.4, 16.3), kennel_hand: T(CX + 2.4, 19.7), herald: T(CX - 3.6, 7.9),
  guide: T(CX + 1.65, 7.35),
};
export const workerAt = (w: { id: string; post: string }): Pt | null => WORKER_AT[w.id] ?? (PLOTS as Record<string, Pt | undefined>)[w.post] ?? null;
/** a worker's frames (art/town-ids.md: `town_worker_<id>`, `_1` the idle beat) */
export const workerIds = (id: string, frame = 0): string[] => frame ? [`town_worker_${id}_1`, `town_worker_${id}`] : [`town_worker_${id}`];
export type Party = { at: number; cls: string; sack?: "small" | "large"; chest?: boolean; pet?: string; to: "tent" | "store" };
export type TownState = {
  seed: number; day: number;
  stage: number;                                   // buildings standing (0 = the camp)
  buildings: { id: BuildingId; look: number; fresh: boolean }[];
  staked?: { id: BuildingId; trigger: string };
  hero: { cls: string; look?: string };
  pets: string[];                                  // companions following the hero
  penned: string[];                                // companions lying in the kennel's pen
  parties: Party[];                                // the absence's runs walking out of the mouth
  markers: Marker[];                               // ≤ 3
  board: boolean;                                  // the quest board stands by the mouth
  depth: number;                                   // the best depth (the mouth's plaque)
  chest?: ChestState;                              // Cut 30.5: the haul chest (none once the porter carries the hauls)
  workers: WorkerSpot[];                           // Cut 30.5: hired workers at their posts; the lit node's greyed
};
export type TownOpts = { board?: boolean; seen?: string[]; opened?: string[]; kit?: { n: number; price?: number }; now?: Date; absenceNew?: boolean; chestOpen?: boolean };

const hashN = (...xs: number[]): number => { let h = 2166136261; for (const x of xs) { h ^= x | 0; h = Math.imul(h, 16777619); h ^= h >>> 13; } return h >>> 0; };
export const localDay = (now = new Date()): number => Math.floor((now.getTime() - now.getTimezoneOffset() * 60000) / 86400000);
const PARTIES_MAX = 6;
/** The core's town (`Lineage.town`) and the absence's report → the scene. Pure (walkers are seeded, never read back). */
export function townState(L: Lineage, absence?: ReturnReport | null, o: TownOpts = {}): TownState {
  const town = L.town;
  const built = (town?.buildings ?? []).filter((b): b is typeof b & { id: BuildingId } => (BUILDINGS as readonly string[]).includes(b.id));
  const seen = new Set(o.seen ?? built.map((b) => b.id));
  const buildings = built.map((b) => ({ id: b.id, look: Math.max(1, Math.min(3, b.level || 1)), fresh: !seen.has(b.id) }));
  const has = (id: string): boolean => buildings.some((b) => b.id === id);
  const next = town?.next && (BUILDINGS as readonly string[]).includes(town.next) && !has(town.next) ? { id: town.next as BuildingId, trigger: town.next_trigger ?? "" } : undefined;
  const day = localDay(o.now);
  const seed = hashN(L.seed ?? 0, day);
  // the absence's parties: one per run that came home (a death walks nobody out), the biggest first, ≤ PARTIES_MAX
  const parties: Party[] = [];
  if (absence) {
    const exits = (absence.exits ?? []).filter((x) => !x.cause && (x.keep_pct > 0 || x.kept > 0) && !/^died\b/.test(x.text));
    const home = exits.length ? exits.map((x) => x.kept) : Array.from({ length: Math.min(PARTIES_MAX, Math.max(0, (absence.returned ?? 0) + (absence.banked ?? 0))) }, () => Math.round((absence.gold?.home ?? 0) / Math.max(1, (absence.returned ?? 0) + (absence.banked ?? 0))));
    const order = home.map((g, i) => ({ g, i })).sort((a, b) => b.g - a.g || a.i - b.i).slice(0, PARTIES_MAX).sort((a, b) => a.i - b.i);
    const finds = (absence.found?.length ?? 0) > 0 || (absence.new_finds?.length ?? 0) > 0;
    const tame = absence.tamed?.[0];
    const big = Math.max(60, ...home) * 0.5;
    order.forEach(({ g }, k) => {
      const chest = finds && k === order.length - 1, pet = tame && k === 0 ? tame : undefined;
      parties.push({ at: 0, cls: L.class, sack: g <= 0 ? undefined : g >= big ? "large" : "small", chest, pet, to: chest && has("storehouse") ? "store" : "tent" });
    });
    // ~20 s in all, ≤ 3 on screen (a walk out is ~6 s)
    const span = 20 - 6, gap = parties.length > 1 ? Math.max(2.2, span / (parties.length - 1)) : 0;
    parties.forEach((p, i) => { p.at = 0.6 + i * gap; });
  }
  // markers (≤ 3): a sword over the forge when a kit step is affordable, a coin over the bank when a night's interest came in, a `!`
  // over a building not yet opened since it was built
  const markers: Marker[] = [];
  // Cut 30.5: the chest before the porter — closed and empty from day 0 (drawn, not a surface), full with gold (a `!` and the sum), open
  // just after a tap; the porter's hire retires it
  const W = L.tree, porter = W?.nodes.find((n) => n.id === "porter");
  const chest: ChestState | undefined = W && porter && (porter.state !== "done" || porter.paused) ? { state: o.chestOpen ? "open" : W.chest > 0 ? "full" : "closed", gold: W.chest, count: porter.count, need: porter.need } : undefined;
  if (chest?.state === "full") markers.push({ at: "chest", kind: "chest", label: `$${chest.gold}` });
  const workers: WorkerSpot[] = (town?.workers ?? []).map((w) => ({ id: w.id, post: w.post, lit: !!w.lit, price: w.price, paused: !!w.paused, rank: w.rank ?? 1 }));
  if (has("blacksmith") && (o.kit?.n ?? 0) > 0) markers.push({ at: "blacksmith", kind: "sword", label: o.kit?.price ? `$${o.kit.price}` : undefined });
  if (has("bank") && (town?.bank ?? 0) > 0 && o.absenceNew) markers.push({ at: "bank", kind: "coin" });
  const opened = new Set(o.opened ?? []);
  for (const b of buildings) if (markers.length < 3 && !opened.has(b.id) && !markers.some((m) => m.at === b.id)) markers.push({ at: b.id, kind: "rune" });
  return {
    seed, day, stage: buildings.length, buildings, staked: next, hero: { cls: L.class || "fighter", look: L.look },
    pets: (L.party ?? []).slice(0, 2).map((c) => c.kind), penned: has("kennel") ? (L.kennel ?? []).slice(0, 2).map((c) => c.kind) : [],
    parties, markers: markers.slice(0, 3), depth: L.best_depth ?? 0, board: !!o.board, chest, workers,
  };
}

// ---- the atlas: town sprites cut to their drawn size, a primitive for anything not packed ----------------------------------------

type Frame = { x: number; y: number; w: number; h: number };
export type Slot = Frame & { u0: number; v0: number; u1: number; v1: number };
let packed: Promise<{ img: HTMLImageElement; frames: Record<string, Frame> } | null> | null = null;
function loadPacked(): Promise<{ img: HTMLImageElement; frames: Record<string, Frame> } | null> {
  packed ??= fetch(`${import.meta.env.BASE_URL}art/atlas.json`).then((r) => r.json()).then((j: { frames?: Record<string, Frame>; image?: string }) => new Promise((res) => {
    const img = new Image();
    img.onload = () => res({ img, frames: j.frames ?? {} });
    img.onerror = () => res(null);
    img.src = `${import.meta.env.BASE_URL}art/${j.image ?? "atlas.png"}`;
  })).catch(() => null) as Promise<{ img: HTMLImageElement; frames: Record<string, Frame> } | null>;
  return packed;
}
const ATLAS = 1024;
/** a walker's coat hue by class (rgb 0..255, applied to the mid-tones at a pixel's own value) */
const WALKER_TINT: Record<string, number[]> = { rogue: [120, 70, 140], ranger: [80, 140, 72], caster: [70, 96, 200], fighter: [150, 160, 176] };
/** fallback aspect (w / h) per id family */
const ASPECT: [RegExp, number][] = [[/^town_mouth/, 1.55], [/^town_(blacksmith|bank|storehouse|kennel)_/, 1.25], [/^town_tent/, 1.05], [/^town_board/, 0.85], [/^town_campfire/, 1.1],
  [/^town_plot/, 1.1], [/^town_scaffold/, 0.9], [/^(hero_|walk_|town_(smith|merchant|carter|child))/, 0.7], [/^town_(sack|chest)/, 0.95], [/^town_worker_(porter|armourer|apprentice|drillmaster|kennel_hand)/, 1.2], [/^town_worker_/, 0.75], [/^town_haul_chest/, 1.5], [/^town_flag/, 0.6],
  [/^env_torch/, 0.66], [/^town_env_tree/, 0.5], [/^town_env_/, 1], [/^fx_/, 1]];
export class TownAtlas {
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  version = 0;
  private slots = new Map<string, Slot>();
  private sx = 1; private sy = 1; private sh = 0;
  private src: { img: HTMLImageElement; frames: Record<string, Frame> } | null = null;
  ready = false;
  private lastId = "";
  constructor() {
    this.canvas = document.createElement("canvas"); this.canvas.width = ATLAS; this.canvas.height = ATLAS;
    this.ctx = this.canvas.getContext("2d", { willReadFrequently: true })!;
    void loadPacked().then((p) => { this.src = p; this.ready = true; this.reset(); });
  }
  has(id: string): boolean { return !!this.src?.frames[id]; }
  /** Cut 30.5: the slot of `ids` greyed — a lit node's worker not yet hired (art/town_sheet.py `grey()`: the values only, lifted toward a
   *  pale MIST grey, alpha × 0.8), cut once into its own slot */
  grey(ids: string | string[], h: number): Slot {
    const base = this.get(ids, h);
    const key = `grey:${Array.isArray(ids) ? ids.join("|") : ids}@${h}`;   // (a reset — the packed atlas landing — clears it with the rest)
    const hit = this.slots.get(key); if (hit) return hit;
    const w = base.w;
    if (this.sx + w + 1 > ATLAS) { this.sy += this.sh + 1; this.sx = 1; this.sh = 0; }
    if (this.sy + h + 1 > ATLAS) { this.reset(); return this.get(ids, h); }
    const x = this.sx, y = this.sy; this.sx += w + 1; this.sh = Math.max(this.sh, h);
    const d = this.ctx.getImageData(base.x, base.y, base.w, base.h), p = d.data;
    for (let i = 0; i < p.length; i += 4) {
      const l = 0.2126 * p[i]! + 0.7152 * p[i + 1]! + 0.0722 * p[i + 2]!;
      p[i] = l * 0.4 + 164 * 0.42; p[i + 1] = l * 0.4 + 188 * 0.42; p[i + 2] = l * 0.4 + 214 * 0.42; p[i + 3] = p[i + 3]! * 0.8;
    }
    this.ctx.putImageData(d, x, y);
    const slot: Slot = { x, y, w, h, u0: x / ATLAS, v0: 1 - (y + h) / ATLAS, u1: (x + w) / ATLAS, v1: 1 - y / ATLAS };
    this.slots.set(key, slot); this.version++;
    return slot;
  }
  private reset(): void { this.slots.clear(); this.sx = 1; this.sy = 1; this.sh = 0; this.ctx.clearRect(0, 0, ATLAS, ATLAS); this.version++; }
  /** the slot of `id` drawn `h` texels tall (the first id of `ids` the atlas has, else a primitive of the first) */
  get(ids: string | string[], h: number): Slot {
    const list = Array.isArray(ids) ? ids : [ids];
    const id = list.find((x) => this.src?.frames[x]) ?? list[0]!;
    const key = `${id}@${h}`;
    const s = this.slots.get(key); if (s) return s;
    const f = this.src?.frames[id];
    const w = f ? Math.max(1, Math.round(h * f.w / f.h)) : Math.max(1, Math.round(h * (ASPECT.find(([re]) => re.test(id))?.[1] ?? 1)));
    if (this.sx + w + 1 > ATLAS) { this.sy += this.sh + 1; this.sx = 1; this.sh = 0; }
    if (this.sy + h + 1 > ATLAS) this.reset();   // (never in practice: a full sheet starts over)
    const x = this.sx, y = this.sy; this.sx += w + 1; this.sh = Math.max(this.sh, h);
    const slot: Slot = { x, y, w, h, u0: x / ATLAS, v0: 1 - (y + h) / ATLAS, u1: (x + w) / ATLAS, v1: 1 - y / ATLAS };
    this.slots.set(key, slot);
    this.lastId = id;
    if (f && this.src) this.cut(this.src.img, f, slot); else drawPrimitive(this.ctx, id, x, y, w, h);
    this.version++;
    return slot;
  }
  /** a packed master cut to its drawn size by area (smoothed, then the alpha thresholded: a crisp silhouette) */
  private cut(img: HTMLImageElement, f: Frame, s: Slot): void {
    if (f.w === s.w && f.h === s.h) { this.ctx.imageSmoothingEnabled = false; this.ctx.drawImage(img, f.x, f.y, f.w, f.h, s.x, s.y, s.w, s.h); return; }
    const c = document.createElement("canvas"); c.width = s.w; c.height = s.h;
    const g = c.getContext("2d", { willReadFrequently: true })!;
    g.imageSmoothingEnabled = true; g.imageSmoothingQuality = "high";
    g.drawImage(img, f.x, f.y, f.w, f.h, 0, 0, s.w, s.h);
    const d = g.getImageData(0, 0, s.w, s.h), p = d.data;
    for (let i = 3; i < p.length; i += 4) p[i] = p[i]! < 110 ? 0 : 255;
    // the art note ("the four walkers look alike"): a walker's coat takes its class's hue — mid-tones, never the BLOOD cloak or the ink
    const tint = WALKER_TINT[/^(?:walk|hero)_(\w+?)(?:_|$)/.exec(f === undefined ? "" : this.lastId)?.[1] ?? ""];
    if (tint) for (let i = 0; i < p.length; i += 4) {
      if (!p[i + 3]) continue;
      const r = p[i]!, gg = p[i + 1]!, b = p[i + 2]!, l = (0.2126 * r + 0.7152 * gg + 0.0722 * b) / 255;
      if (l < 0.12 || l > 0.8 || (r > gg * 1.5 && r > b * 1.4)) continue;
      const m = 0.42 * (1 - Math.abs(l - 0.42) * 1.6);
      p[i] = r + (tint[0]! * l * 2 - r) * m; p[i + 1] = gg + (tint[1]! * l * 2 - gg) * m; p[i + 2] = b + (tint[2]! * l * 2 - b) * m;
    }
    this.ctx.putImageData(d, s.x, s.y);
  }
}
/** the primitive for an id the atlas lacks: a coloured block in the thing's shape (docs/TOWN.md §7: "a coloured block + its icon") */
function drawPrimitive(g: CanvasRenderingContext2D, id: string, x: number, y: number, w: number, h: number): void {
  const R = (c: string, a: number, b: number, cw: number, ch: number): void => { g.fillStyle = c; g.fillRect(x + Math.round(a), y + Math.round(b), Math.max(1, Math.round(cw)), Math.max(1, Math.round(ch))); };
  const tri = (c: string, x0: number, y0: number, x1: number, x2: number, yb: number): void => { g.fillStyle = c; g.beginPath(); g.moveTo(x + x0, y + y0); g.lineTo(x + x1, y + yb); g.lineTo(x + x2, y + yb); g.closePath(); g.fill(); };
  const speck = (base: string, dark: string, n: number): void => { R(base, 0, 0, w, h); for (let i = 0; i < n; i++) R(dark, (hashN(i, x, y) % w), (hashN(y, i, 7) % h), 1, 1); };
  if (/^town_env_grass/.test(id)) speck("#3d5a2a", "#2f4720", 22);
  else if (/^town_env_dirt/.test(id)) speck("#7a5a3a", "#5e442a", 18);
  else if (/^town_env_plaza/.test(id)) { speck("#8a8478", "#6b665c", 10); R("#5c574e", 0, 7, w, 1); R("#5c574e", 7, 0, 1, 8); }
  else if (/^town_env_cliff/.test(id)) speck("#3b3a3f", "#2a292e", 26);
  else if (/^town_env_water/.test(id)) speck("#2a4a62", "#3f6a86", 12);
  else if (/^town_env_tree/.test(id)) { R("#4a321f", w * 0.4, h * 0.65, w * 0.2, h * 0.35); g.fillStyle = "#24401f"; g.beginPath(); g.ellipse(x + w / 2, y + h * 0.4, w * 0.5, h * 0.38, 0, 0, Math.PI * 2); g.fill(); }
  else if (/^town_mouth/.test(id)) { R("#4a474e", 0, h * 0.15, w, h * 0.85); g.fillStyle = "#0c0b10"; g.beginPath(); g.ellipse(x + w / 2, y + h * 0.62, w * 0.2, h * 0.36, 0, Math.PI, 0); g.fill(); R("#0c0b10", w * 0.3, h * 0.62, w * 0.4, h * 0.38); }
  else if (/^town_(blacksmith|bank|storehouse|kennel)_/.test(id)) {
    const col = /blacksmith/.test(id) ? ["#4b4650", "#7a2a1c"] : /bank/.test(id) ? ["#b8b0a0", "#5a6f86"] : /storehouse/.test(id) ? ["#7a5534", "#5a3a22"] : ["#8c6a44", "#6a4a2a"];
    R(col[0]!, w * 0.08, h * 0.42, w * 0.84, h * 0.58); tri(col[1]!, w / 2, 0, -w * 0.02 + w * 0.02, w, h * 0.46);
    R("#1e1914", w * 0.42, h * 0.68, w * 0.16, h * 0.32);
    if (/blacksmith/.test(id)) R("#e07a2a", w * 0.16, h * 0.7, w * 0.14, h * 0.1);
    if (/bank/.test(id)) { g.fillStyle = "#e3c24a"; g.beginPath(); g.arc(x + w * 0.5, y + h * 0.3, Math.max(2, h * 0.08), 0, Math.PI * 2); g.fill(); }
  }
  else if (/^town_tent/.test(id)) { tri("#c9b48a", w / 2, 0, 0, w, h); tri("#2a2218", w / 2, h * 0.45, w * 0.36, w * 0.64, h); }
  else if (/^town_board/.test(id)) { R("#4a321f", w * 0.15, h * 0.3, 2, h * 0.7); R("#4a321f", w * 0.8, h * 0.3, 2, h * 0.7); R("#6a5034", 0, 0, w, h * 0.6); R("#d8c8a0", w * 0.2, h * 0.12, w * 0.3, h * 0.25); }
  else if (/^town_crate/.test(id)) { R("#8a6034", 0, 0, w, h); R("#5a3c1e", 0, h / 2, w, 1); R("#5a3c1e", w / 2, 0, 1, h); }
  else if (/^town_campfire/.test(id)) { R("#4a321f", 0, h * 0.75, w, h * 0.25); tri(/_1$/.test(id) ? "#ffcf5a" : "#f08a2a", w / 2, 0, w * 0.15, w * 0.85, h * 0.8); }
  else if (/^town_plot/.test(id)) { for (const a of [0.1, 0.9]) R("#6a4a2a", w * a - 1, h * 0.2, 2, h * 0.8); R("#d8c8a0", w * 0.1, h * 0.35, w * 0.8, 1); }
  else if (/^town_scaffold/.test(id)) { for (let i = 0; i <= 3; i++) { R("#7a5534", (w - 2) * i / 3, 0, 2, h); R("#7a5534", 0, (h - 2) * i / 3, w, 2); } }
  else if (/^town_sack/.test(id)) { g.fillStyle = "#b89a62"; g.beginPath(); g.ellipse(x + w / 2, y + h * 0.6, w * 0.45, h * 0.4, 0, 0, Math.PI * 2); g.fill(); R("#e3c24a", w * 0.4, h * 0.1, w * 0.2, h * 0.25); }
  else if (/^town_haul_chest/.test(id)) {   // the haul chest: closed · full (a coin heap and a glint) · open (the lid up behind, dark inside)
    const open = /_open$/.test(id), full = /_full$/.test(id);
    if (open) { R("#4a3220", w * 0.08, 0, w * 0.84, h * 0.4); R("#0d0c14", w * 0.12, h * 0.4, w * 0.76, h * 0.14); }
    R("#5a3c22", w * 0.06, h * 0.45, w * 0.88, h * 0.47); R("#2b3350", w * 0.06, h * 0.6, w * 0.88, 1); R("#2b3350", w * 0.3, h * 0.45, 1, h * 0.47); R("#2b3350", w * 0.7, h * 0.45, 1, h * 0.47);
    if (!open) R("#6a4a2c", w * 0.04, h * 0.32, w * 0.92, h * 0.15);
    R("#b89448", w * 0.45, h * 0.52, w * 0.1, h * 0.14);
    if (full) { R("#b89448", w * 0.12, h * 0.22, w * 0.76, h * 0.1); R("#eadfc5", w * 0.62, h * 0.08, 1, h * 0.16); R("#eadfc5", w * 0.56, h * 0.15, w * 0.14, 1); }
  }
  else if (/^town_worker_/.test(id)) {   // a worker: a townsperson (no BLOOD) and the role's tag in its own colour (art/town-ids.md, workers)
    const role = id.replace(/^town_worker_|_1$/g, "");
    const coat: Record<string, string> = { porter: "#3a3448", armourer: "#5a6070", apprentice: "#5a3c22", keeper: "#2b3350", clerk: "#1c1b2b", drillmaster: "#6a5a40", kennel_hand: "#4a3a2a", herald: "#2b3350", guide: "#7d8ea4", scout: "#4a3a2a", quartermaster: "#6a5a40" };
    const fw = /porter|armourer|apprentice|drillmaster|kennel_hand/.test(role) ? w * 0.55 : w;   // the figure's share (the tag object right of it)
    R(coat[role] ?? "#3a3448", fw * 0.22, h * 0.28, fw * 0.56, h * 0.5); R("#eadfc5", fw * 0.32, h * 0.06, fw * 0.36, h * 0.2);
    R("#1c1b2b", fw * 0.26, h * 0.78, fw * 0.18, h * 0.22); R("#1c1b2b", fw * 0.56, h * 0.78, fw * 0.18, h * 0.22);
    const T = (c: string, a: number, b: number, cw: number, ch: number): void => R(c, fw + (w - fw) * a, h * b, (w - fw) * cw, h * ch);
    if (role === "porter") { T("#6a4a2c", 0.05, 0.45, 0.9, 0.3); T("#eadfc5", 0.15, 0.25, 0.7, 0.22); T("#1c1b2b", 0.3, 0.72, 0.35, 0.28); }
    else if (role === "armourer") { T("#6a4a2c", 0.15, 0.2, 0.12, 0.8); T("#6a4a2c", 0.75, 0.2, 0.12, 0.8); T("#a4bcd6", 0.35, 0.1, 0.08, 0.7); T("#a4bcd6", 0.55, 0.1, 0.08, 0.7); }
    else if (role === "apprentice") { T("#0d0c14", 0.1, 0.55, 0.8, 0.2); T("#0d0c14", 0.35, 0.75, 0.3, 0.25); T("#e8923a", 0.25, 0.48, 0.5, 0.07); }
    else if (role === "drillmaster") { T("#b89448", 0.4, 0.15, 0.2, 0.3); T("#6a4a2c", 0.47, 0.45, 0.06, 0.55); T("#6a4a2c", 0.1, 0.32, 0.8, 0.06); }
    else if (role === "kennel_hand") { T("#4d6c99", 0.1, 0.55, 0.7, 0.25); T("#4d6c99", 0.55, 0.4, 0.3, 0.2); T("#1c1b2b", 0.15, 0.8, 0.1, 0.2); }
    else if (role === "keeper") { R("#b89448", fw * 0.7, h * 0.12, 1, h * 0.88); R("#b89448", fw * 0.62, h * 0.85, fw * 0.3, h * 0.15); }
    else if (role === "clerk") R("#eadfc5", fw * 0.5, h * 0.4, fw * 0.45, h * 0.16);
    else if (role === "herald") { R("#eadfc5", fw * 0.6, h * 0.42, fw * 0.25, h * 0.4); R("#b89448", fw * 0.78, h * 0.25, fw * 0.16, h * 0.1); }
    else if (role === "guide") { R("#e8923a", fw * 0.76, h * 0.4, fw * 0.18, h * 0.12); R("#6a4a2c", fw * 0.12, h * 0.05, 1, h * 0.95); }
    else if (role === "scout") { R("#1c1b2b", fw * 0.15, h * 0.02, fw * 0.7, h * 0.08); R("#b89448", fw * 0.62, h * 0.12, fw * 0.32, h * 0.06); R("#eadfc5", fw * 0.78, h * 0.35, fw * 0.2, h * 0.06); }
    else if (role === "quartermaster") { R("#5a3c22", fw * 0.5, h * 0.3, fw * 0.45, h * 0.35); R("#eadfc5", fw * 0.5, h * 0.26, fw * 0.45, h * 0.06); }
  }
  else if (/^town_chest/.test(id)) { R("#7a4a1e", 0, h * 0.3, w, h * 0.7); R("#ffd76a", 0, h * 0.3, w, 2); R("#ffd76a", w * 0.42, h * 0.45, w * 0.16, h * 0.2); }
  else if (/^town_flag/.test(id)) { R("#4a321f", 0, 0, 1, h); R(/_1$/.test(id) ? "#a01a28" : "#c01530", 1, 1, w - 1, h * 0.4); }
  else if (/^env_torch/.test(id)) { R("#4a321f", w * 0.4, h * 0.4, w * 0.2, h * 0.6); R(/_1$/.test(id) ? "#ffcf5a" : "#f08a2a", w * 0.25, 0, w * 0.5, h * 0.45); }
  else if (/^fx_glint/.test(id)) { const c = Math.floor(w / 2); R("#fffbe8", c, 0, 1, h); R("#fffbe8", 0, c, w, 1); R("#ffffff", c - 1, c - 1, 3, 3); }
  else if (/^fx_dot_/.test(id)) R(`#${id.slice(7)}`, 0, 0, w, h);
  else if (/^(hero_|walk_)/.test(id)) {
    const body = /rogue/.test(id) ? "#4a3a5a" : /ranger/.test(id) ? "#3f5a32" : /caster/.test(id) ? "#3a4a7a" : "#8a8f98";
    R("#c01530", w * 0.15, h * 0.32, w * 0.3, h * 0.5); R(body, w * 0.25, h * 0.3, w * 0.5, h * 0.45);
    R("#e8dcc8", w * 0.32, h * 0.08, w * 0.36, h * 0.22); R("#2a2420", w * 0.3, h * 0.75, w * 0.15, h * 0.25); R("#2a2420", w * 0.55, h * 0.75, w * 0.15, h * 0.25);
  }
  else { R("#7a6a4a", w * 0.1, h * 0.35, w * 0.8, h * 0.45); R("#5a4a32", w * 0.15, h * 0.75, w * 0.12, h * 0.25); R("#5a4a32", w * 0.7, h * 0.75, w * 0.12, h * 0.25); R("#7a6a4a", w * 0.65, h * 0.15, w * 0.3, h * 0.3); }   // a beast
}

// ---- the frame: what one paint draws (both views read it) -----------------------------------------------------------------------

export type Quad = { s: Slot; x: number; y: number; z: number; w: number; h: number; flip: boolean; dim: number; fade: number };
export type Light = { x: number; y: number; r: number; c: [number, number, number] };
export type TownFrame = { x0: number; y0: number; w: number; h: number; k: number; W: number; H: number; quads: Quad[]; n: number; ground: Quad[]; groundKey: string; lights: Light[]; amb: [number, number, number]; night: number; live: number };

/** hour (0..24) → night 0..1: day 7–17, dusk 17–21, night 21–5, dawn 5–7 */
export function nightOf(hour: number): number {
  const h = ((hour % 24) + 24) % 24;
  if (h >= 21 || h < 5) return 1;
  if (h >= 7 && h < 17) return 0;
  return h < 7 ? 1 - (h - 5) / 2 : (h - 17) / 4;
}
const mix3 = (a: number[], b: number[], t: number): [number, number, number] => [a[0]! + (b[0]! - a[0]!) * t, a[1]! + (b[1]! - a[1]!) * t, a[2]! + (b[2]! - a[2]!) * t];
const AMB_DAY = [1.1, 1.08, 1.06], AMB_DUSK = [0.86, 0.68, 0.62], AMB_NIGHT = [0.2, 0.28, 0.52];   // moonlight: cool and low, so the warm windows and the fire carry the night (the coordinator: night read as just darker)
export function ambientOf(night: number): [number, number, number] {
  return night < 0.5 ? mix3(AMB_DAY, AMB_DUSK, night * 2) : mix3(AMB_DUSK, AMB_NIGHT, (night - 0.5) * 2);
}

// ---- walkers ------------------------------------------------------------------------------------------------------------------

type Leg = { t0: number; t1: number; a: Pt; b: Pt };
type Walker = { id: string; sprite: string[]; walk: string[]; h: number; legs: Leg[]; from: number; to: number; carry?: string; carryH?: number; lag: number;
  follow?: Walker; emerge?: boolean; vanish?: boolean; extend?: (w: Walker) => void; pet?: boolean };
const SPEED = 40;
const PATH_DIM = 0.75;   // the path graded down a step (the art note: brighter and stripier than the buildings beside the dark grass; the tile itself is calmer and darker since Cut 30 integration, so less of it here)   // texels a second (~2 tiles)
function legsAlong(pts: Pt[], t0: number, speed = SPEED, out: Leg[] = []): number {
  let t = t0;
  for (let i = 1; i < pts.length; i++) { const d = dist(pts[i - 1]!, pts[i]!); if (d < 0.01) continue; out.push({ t0: t, t1: t + d / speed, a: pts[i - 1]!, b: pts[i]! }); t += d / speed; }
  return t;
}
function walkerAt(w: Walker, t: number): { x: number; y: number; moving: boolean; dx: number } | null {
  if (t < w.from || t > w.to) return null;
  const legs = w.legs;
  if (w.extend && legs.length && legs[legs.length - 1]!.t1 < t + 30) w.extend(w);
  if (!legs.length) return null;
  if (t <= legs[0]!.t0) return { x: legs[0]!.a.x, y: legs[0]!.a.y, moving: false, dx: 0 };
  let lo = 0, hi = legs.length - 1;
  while (lo < hi) { const m = (lo + hi + 1) >> 1; if (legs[m]!.t0 <= t) lo = m; else hi = m - 1; }
  const L = legs[lo]!;
  if (t >= L.t1) return { x: L.b.x, y: L.b.y, moving: false, dx: 0 };
  const f = (t - L.t0) / (L.t1 - L.t0);
  return { x: L.a.x + (L.b.x - L.a.x) * f, y: L.a.y + (L.b.y - L.a.y) * f, moving: true, dx: L.b.x - L.a.x };
}

// ---- particles (cosmetic) -------------------------------------------------------------------------------------------------------

type Part = { x: number; y: number; vx: number; vy: number; age: number; life: number; kind: "smoke" | "ember" | "spark" | "dust"; c: string; s: number };
const SMOKE = ["9a958c", "7d7a74", "b4afa4"], EMBER = ["ffb04a", "ff7a2a", "ffd76a"], DUST = ["b89a6a", "9a7e56", "cdb48a"];

// ---- the view ---------------------------------------------------------------------------------------------------------------------

export type TownStats = { fps: number; frames: number; cpuMs: number; cpuP95: number; walkers: number; mode: "gl" | "2d"; glLost: boolean; k: number; night: number; idle: boolean; running: boolean;
  buildings: string[]; staked?: string; markers: string[]; view: [number, number, number, number]; chest?: string; workers: string[]; ranks: Record<string, number> };
export type TownView = {
  readonly el: HTMLElement;
  setState(s: TownState): void;
  /** At SEND: the hero walks to the mouth and goes in (≤ `ms`); resolves when he is in. */
  walkIn(ms?: number): Promise<void>;
  resize(): void;
  /** a plot's sprite box in CSS px (relative to the view's box); null when nothing stands there */
  rectOf(id: PlotId | "staked" | "chest" | `worker:${string}`): { x: number; y: number; w: number; h: number } | null;
  onLayout(fn: () => void): void;
  stats(): TownStats;
  stress(n: number): void;
  setHour(h: number | null): void;
  /** input seen (the idle cap lifts) */
  poke(): void;
  dispose(): void;
};
const IDLE_MS = 10_000, IDLE_FPS = 20, SOFT_FPS = 2;

let sharedAtlas: TownAtlas | null = null;
let sharedGl: { canvas: HTMLCanvasElement; gl: GLTown; lost: boolean } | null = null;
export function createTownView(host: HTMLElement): TownView {
  const el = host;
  // one GL canvas and context for every mount (a camp re-mounts on every return): taken here, given back on dispose
  const kept = sharedGl && !sharedGl.gl.isLost() ? sharedGl : null; if (sharedGl && !kept) sharedGl.gl.dispose(); sharedGl = null;
  const glCanvas = kept?.canvas ?? document.createElement("canvas");
  glCanvas.className = "town-gl";
  glCanvas.setAttribute("aria-hidden", "true");
  el.appendChild(glCanvas);
  const atlas = (sharedAtlas ??= new TownAtlas());   // one sheet for every mount (the camp re-mounts on every return; the cuts are not free)
  let gl: GLTown | null = null;
  if (kept) { gl = kept.gl; gl.restored(); }
  else try { gl = new GLTown(glCanvas, atlas); } catch (e) { console.warn("town: no WebGL, the 2D view stands in", e); gl = null; }
  let c2d: HTMLCanvasElement | null = null;
  const view2d = (): HTMLCanvasElement => {
    if (c2d) return c2d;
    c2d = document.createElement("canvas"); c2d.className = "town-2d"; c2d.setAttribute("aria-hidden", "true");
    el.appendChild(c2d); return c2d;
  };
  let glLost = false;
  const onLost = (e: Event): void => { e.preventDefault(); glLost = true; layoutDirty = true; };
  const onRestored = (): void => { glLost = false; gl?.restored(); layoutDirty = true; };
  glCanvas.addEventListener("webglcontextlost", onLost);
  glCanvas.addEventListener("webglcontextrestored", onRestored);

  let state: TownState | null = null;
  let walkers: Walker[] = [];
  const parts: Part[] = [];
  let t = 0, last = performance.now(), raf = 0, frames = 0, disposed = false;
  const fpsWin: number[] = [], cpu: number[] = [];
  let lastInput = performance.now(), lastDraw = 0;
  let hourOverride: number | null = null;
  let layoutDirty = true;
  const layoutFns: (() => void)[] = [];
  let builtAt = new Map<string, number>();   // fresh buildings: the scene time their scaffold went up
  let hero: Walker | null = null;
  let walkingIn: { t0: number; t1: number; done: () => void } | null = null;
  let stressN = 0;
  const arrive = new Map<string, Leg[]>();   // Cut 30.5: a worker hired on this mount walks out of the tent to his post
  const F: TownFrame = { x0: 0, y0: 0, w: 1, h: 1, k: 1, W: 1, H: 1, quads: [], n: 0, ground: [], groundKey: "", lights: [], amb: [1, 1, 1], night: 0, live: 0 };
  const rng = (n: number): number => (hashN(state?.seed ?? 1, n) % 100000) / 100000;

  // -- the walkers of a state --
  function build(): void {
    const s = state!; walkers = [];
    const cls = s.hero.cls, heroIds = [...(s.hero.look ? [`hero_${cls}_${s.hero.look}`] : []), `hero_${cls}`, "hero_fighter"], walkIds = [`walk_${cls}`, ...heroIds];
    let k = 0;
    const partyEnd = s.parties.length ? s.parties[s.parties.length - 1]!.at + 7 : 0;
    for (const p of s.parties) {
      const legs: Leg[] = [];
      const end = legsAlong(route("in", p.to), p.at, SPEED, legs);
      const w: Walker = { id: `party${k++}`, sprite: heroIds, walk: walkIds, h: 24, legs, from: p.at, to: end + 0.4, lag: 0, emerge: true, vanish: true,
        carry: p.chest ? "town_chest_glow" : p.sack ? `town_sack_${p.sack}` : undefined, carryH: p.chest ? 8 : p.sack === "large" ? 8 : 6 };
      walkers.push(w);
      if (p.pet) walkers.push({ id: `${w.id}pet`, sprite: [p.pet, "town_dog"], walk: [p.pet, "town_dog"], h: 14, legs, from: p.at + 0.7, to: end + 1.1, lag: 0.7, follow: w, emerge: true, vanish: true, pet: true });
    }
    // the hero at home: tent → forge → fire (→ the crate while no forge stands), a pause of 2–6 s at each
    const stops = ["tent", s.buildings.some((b) => b.id === "blacksmith") ? "smith" : "crate", "fireW"];
    let i = Math.floor(rng(1) * 3), n = 0;
    hero = { id: "hero", sprite: heroIds, walk: walkIds, h: 24, legs: [], from: partyEnd, to: Infinity, lag: 0,
      extend: (w) => {
        let tt = w.legs.length ? w.legs[w.legs.length - 1]!.t1 : partyEnd;
        for (let j = 0; j < 6; j++) {
          const a = stops[i % 3]!, b = stops[(i + 1) % 3]!; i++;
          const pause = 2 + rng(100 + n++) * 4;
          const at = NODES[a]!; w.legs.push({ t0: tt, t1: tt + pause, a: at, b: at }); tt += pause;
          tt = legsAlong(route(a, b), tt, SPEED * 0.8, w.legs);
        }
      } };
    hero.extend!(hero);
    walkers.push(hero);
    s.pets.forEach((p, j) => walkers.push({ id: `pet${j}`, sprite: [p, "town_dog"], walk: [p, "town_dog"], h: 14, legs: hero!.legs, from: partyEnd, to: Infinity, lag: 0.9 + j * 0.6, follow: hero!, pet: true }));
    addStress();
  }
  function addStress(): void {
    walkers = walkers.filter((w) => !w.id.startsWith("stress"));
    const names = Object.keys(NODES).filter((x) => x !== "in" && x !== "exit"), classes = ["fighter", "rogue", "ranger", "caster"];
    for (let j = 0; j < stressN; j++) {
      const cls = classes[j % 4]!; let at = names[j % names.length]!; let m = 0;
      const w: Walker = { id: `stress${j}`, sprite: [`hero_${cls}`], walk: [`walk_${cls}`, `hero_${cls}`], h: 24, legs: [], from: t, to: Infinity, lag: 0,
        extend: (ww) => { let tt = ww.legs.length ? ww.legs[ww.legs.length - 1]!.t1 : t; for (let q = 0; q < 6; q++) { const nx = names[hashN(j, m++, 77) % names.length]!; if (nx === at) continue; ww.legs.push({ t0: tt, t1: tt + 0.6, a: NODES[at]!, b: NODES[at]! }); tt = legsAlong(route(at, nx), tt + 0.6, SPEED, ww.legs); at = nx; } } };
      w.extend!(w); walkers.push(w);
    }
  }

  // -- the frame --
  const push = (s: Slot, x: number, y: number, z: number, flip = false, dim = 1, fade = 0, w = s.w, h = s.h): void => {
    let q = F.quads[F.n];
    if (!q) { q = { s, x, y, z, w, h, flip, dim, fade }; F.quads[F.n] = q; }
    else { q.s = s; q.x = x; q.y = y; q.z = z; q.w = w; q.h = h; q.flip = flip; q.dim = dim; q.fade = fade; }
    F.n++;
  };
  const zOf = (footY: number): number => 1 + footY / 2000;
  const inView = (x: number, y: number, w: number, h: number): boolean => x + w / 2 >= F.x0 - 4 && x - w / 2 <= F.x0 + F.w + 4 && y >= F.y0 - 4 && y - h <= F.y0 + F.h + 4;
  function sprite(ids: string | string[], h: number, x: number, y: number, o: { flip?: boolean; dim?: number; fade?: number; z?: number } = {}): void {
    const s = atlas.get(ids, h);
    if (!inView(x, y, s.w, s.h)) return;
    push(s, Math.round(x), Math.round(y), o.z ?? zOf(y), !!o.flip, o.dim ?? 1, o.fade ?? 0);
  }
  /** the terrain (rebuilt when the stage, the view or the atlas changes) */
  function ground(): void {
    const s = state!; const key = `${atlas.version}|${s.stage}|${s.buildings.map((b) => b.id).join(",")}|${s.staked?.id ?? ""}`;
    if (key === F.groundKey) return;
    F.groundKey = key; F.ground = [];
    const plaza = s.stage >= 4;
    const standing = new Set(s.buildings.map((b) => DOOR[b.id]));   // a building's spur is trodden once it stands
    // the path cells (the dirt along the graph's edges and round the fire; the plaza counts as path), then each cell's tile by the
    // art's picking rule (art/town-ids.md: the sides whose neighbour is grass name the edge, corner, end or inner piece)
    const path = new Uint8Array(MAP_W * MAP_H);
    for (let y = 5 + OY; y < MAP_H; y++) for (let x = 0; x < MAP_W; x++) {
      const c = { x: (x + 0.5) * TILE, y: (y + 0.5) * TILE };
      if (dist(c, PLOTS.fire) / TILE < (plaza ? 2.1 : 1.6) || onPath(c, standing)) path[y * MAP_W + x] = 1;
    }
    const at = (x: number, y: number): boolean => x < 0 || x >= MAP_W || y >= MAP_H ? false : y < 5 + OY || !!path[y * MAP_W + x];
    const pathTile = (x: number, y: number, hh: number): string => {
      const n = !at(x, y - 1), e = !at(x + 1, y), so = !at(x, y + 1), w = !at(x - 1, y);
      const k = (n ? 1 : 0) + (e ? 1 : 0) + (so ? 1 : 0) + (w ? 1 : 0);
      if (k === 0) {
        const d = !at(x + 1, y - 1) ? "ne" : !at(x - 1, y - 1) ? "nw" : !at(x + 1, y + 1) ? "se" : !at(x - 1, y + 1) ? "sw" : "";
        return d ? `town_env_dirt_inner_${d}` : `town_env_dirt_${hh < 0.5 ? 0 : hh < 0.85 ? 1 : 2}`;
      }
      if (k === 1) return `town_env_dirt_edge_${n ? "n" : e ? "e" : so ? "s" : "w"}`;
      if (k === 2) { if (n && so) return "town_env_dirt_edge_ns"; if (e && w) return "town_env_dirt_edge_ew"; return `town_env_dirt_corner_${n ? "n" : "s"}${e ? "e" : "w"}`; }
      if (k === 3) return `town_env_dirt_end_${!n ? "s" : !e ? "w" : !so ? "n" : "e"}`;
      return "town_env_dirt_isle";
    };
    for (let y = 0; y < MAP_H; y++) for (let x = 0; x < MAP_W; x++) {
      const c = { x: (x + 0.5) * TILE, y: (y + 0.5) * TILE };
      const hh = hashN(x, y, 5) % 1000 / 1000;
      let id: string;
      const fire = dist(c, PLOTS.fire) / TILE;
      if (y < hillFoot(x)) id = "town_env_cliff";
      else if (x <= CX - 18 && x >= CX - 19) id = "town_env_water";
      else if (plaza && fire < 2.1) id = `town_env_plaza_${hh < 0.6 ? 0 : 1}`;
      else if (path[y * MAP_W + x]) id = pathTile(x, y, hh);
      else id = `town_env_grass_${hh < 0.62 ? 0 : hh < 0.84 ? 1 : hh < 0.95 ? 2 : 3}`;
      const sl = atlas.get([id, "town_env_dirt_0"], TILE);
      // the hillside above the mouth falls away into the dark toward the top (its shoulder against the night), each cell flipped by
      // its hash so the boulders never line up in rows (the coordinator: the band read as a striped wall or water)
      const hill = y < hillFoot(x) ? hillDim(y) : 1;
      F.ground.push({ s: sl, x: x * TILE + TILE / 2, y: (y + 1) * TILE, z: 0, w: TILE, h: TILE, flip: hill < 1 && hh > 0.5, dim: id.includes("dirt") ? PATH_DIM : id.includes("plaza") ? 0.85 : hill, fade: 0 });
    }
  }
  /** the hill's foot: a ragged line (runs of two columns, one in three, a tile further down, never by the mouth), not a ruled edge */
  const hillFoot = (x: number): number => 5 + OY + (Math.abs(x - CX) > 4 && hashN(x >> 1, 0, 7) % 3 === 0 ? 1 : 0);
  const hillDim = (y: number): number => 0.3 + 0.7 * Math.pow(Math.min(1, (y + 1) / (5 + OY)), 1.6);
  const DOOR: Record<string, string> = { blacksmith: "smith", bank: "bank", storehouse: "store", kennel: "kennel" };
  const SPURS = new Set(Object.values(DOOR));
  const onPath = (c: Pt, standing: Set<string>): boolean => EDGES.some(([a, b, r]) => (!SPURS.has(b) || standing.has(b)) && segDist(c, NODES[a]!, NODES[b]!) <= r * TILE);
  /** the forest: trees on a jittered grid, cleared from the paths, the plots and the cliff */
  let trees: (Pt & { k: boolean; d?: number })[] = [];
  function forest(): void {
    trees = [];
    const keep = (p: Pt): boolean => {
      if (p.y < (5.6 + OY) * TILE || (p.x > (CX - 19.8) * TILE && p.x < (CX - 16.2) * TILE)) return false;
      if (EDGES.some(([a, b, r]) => segDist(p, NODES[a]!, NODES[b]!) <= (r + 0.9) * TILE)) return false;
      if (dist(p, PLOTS.fire) < 3 * TILE) return false;
      for (const [id, pl] of Object.entries(PLOTS)) {
        const hw = id === "mouth" ? 4.2 * TILE : id === "fire" || id === "crate" ? 1.4 * TILE : 3.3 * TILE;
        if (Math.abs(p.x - pl.x) < hw && p.y > pl.y - pl.h * 0.55 && p.y < pl.y + 1.6 * TILE) return false;
      }
      return true;
    };
    for (let gy = 5.8 + OY; gy < MAP_H; gy += 1.35) for (let gx = 0.5; gx < MAP_W; gx += 1.25) {
      const j = hashN(Math.round(gx * 10), Math.round(gy * 10), 9);
      const p = { x: (gx + ((j % 100) / 100 - 0.5) * 0.9) * TILE, y: (gy + (((j >> 8) % 100) / 100 - 0.5) * 0.8) * TILE };
      if ((j >> 16) % 100 < 18) continue;
      if (keep(p)) trees.push({ ...p, k: (j >> 4) % 3 === 0 });
    }
    // a thin wood up the hillside (dimmed with the slope), clear of the mouth: the band reads as a hill, not a wall
    for (let gy = 2.2; gy < 5.4 + OY; gy += 1.5) for (let gx = 0.5; gx < MAP_W; gx += 1.6) {
      const j = hashN(Math.round(gx * 10), Math.round(gy * 10), 13);
      const p = { x: (gx + ((j % 100) / 100 - 0.5) * 1.2) * TILE, y: (gy + (((j >> 8) % 100) / 100 - 0.5) * 0.9) * TILE };
      if ((j >> 16) % 100 < 55 || Math.abs(p.x - PLOTS.mouth.x) < 4.6 * TILE) continue;
      trees.push({ ...p, k: (j >> 4) % 2 === 0, d: hillDim(p.y / TILE - 0.5) * 0.85 });
    }
    trees.sort((a, b) => a.y - b.y);
  }
  forest();

  let flickNow = 1;
  /** the warm emitters painted into a sprite (art/town_lights.json: [dx, dy, strength] from its foot), at `base` strength by day */
  function emitters(id: string, p: Pt, night: number, r: number, base: number, gain = 1.5): void {
    const L = LIGHTS as Record<string, number[][]>;
    const list = L[id] ?? L[id.replace(/_\d$/, "_1")] ?? [];
    for (const [dx, dy, k] of list) {
      const a = (base + night * gain) * (k ?? 1) * flickNow; if (a < 0.04 || F.lights.length >= MAX_LIGHTS) continue;
      F.lights.push({ x: p.x + dx!, y: p.y + dy!, r: r * (0.7 + 0.3 * (k ?? 1)) + night * 12, c: [1.0 * a, 0.64 * a, 0.3 * a] });
    }
  }
  function frame(): void {
    const s = state!;
    F.n = 0; F.lights.length = 0;
    const hour = hourOverride ?? (new Date().getHours() + new Date().getMinutes() / 60);
    F.night = nightOf(hour); F.amb = ambientOf(F.night);
    const night = F.night;
    ground();
    // trees (two kinds), the cliff's rim of trees
    const t0s = atlas.get("town_env_tree_0", 32), t1s = atlas.get("town_env_tree_1", 24);
    for (let i = 0; i < trees.length; i++) { const p = trees[i]!, sl = p.k ? t1s : t0s; if (inView(p.x, p.y, sl.w, sl.h)) push(sl, Math.round(p.x), Math.round(p.y), zOf(p.y), false, p.d ?? 1); }
    // the mouth, its torches, the depth plaque's light
    const m = PLOTS.mouth;
    sprite(["town_mouth_cave"], m.h, m.x, m.y);
    const fl = Math.floor(t * 6) & 1;
    void fl;
    emitters("town_mouth_cave", m, night, 30, 0.5, 0.7);   // the torches a step under the windows and the fire (at 1.5 the mouth read ablaze)
    // the camp: the fire, the tent, the crate
    const f = PLOTS.fire;
    sprite([`town_campfire_${Math.floor(t * 5) & 1}`, "town_campfire_0"], f.h, f.x, f.y);
    const flick = 0.9 + 0.1 * Math.sin(t * 11) * Math.sin(t * 3.7);
    F.lights.push({ x: f.x, y: f.y - 8, r: (46 + night * 44) * flick, c: [1.0, 0.6, 0.26].map((v) => v * (0.45 + night * 1.5) * flick) as [number, number, number] });
    flickNow = flick;
    sprite("town_tent", PLOTS.tent.h, PLOTS.tent.x, PLOTS.tent.y);
    if (night > 0.3) F.lights.push({ x: PLOTS.tent.x, y: PLOTS.tent.y - 6, r: 14, c: [0.9, 0.55, 0.25] });
    sprite("town_crate", PLOTS.crate.h, PLOTS.crate.x, PLOTS.crate.y);
    if (s.board) sprite("town_board", PLOTS.board.h, PLOTS.board.x, PLOTS.board.y);
    // the buildings: scaffold → built (dust, a glint)
    for (const b of s.buildings) {
      const p = PLOTS[b.id];
      const at = builtAt.get(b.id);
      const up = at === undefined || t - at >= 1.3;
      if (!up) { sprite("town_scaffold", 56, p.x, p.y); continue; }
      sprite([`town_${b.id}_${b.look}`, `town_${b.id}_1`], p.h, p.x, p.y);
      if (at !== undefined && t - at < 2.4) {
        const g = (t - at - 1.3) / 1.1;
        sprite("fx_glint", 9, p.x + 18, p.y - p.h + 8 + g * 4, { z: 3, fade: Math.max(0, g * 1.2 - 0.2) });
      }
      // the painted emitters (art/town_lights.json: the forge's mouth, lit windows): the forge always, the windows at night
      emitters(`town_${b.id}_${b.look}`, p, night, b.id === "blacksmith" ? 26 : 18, b.id === "blacksmith" ? 0.35 : 0);
    }
    if (s.staked) {
      const p = PLOTS[s.staked.id];
      sprite("town_plot", 16, p.x, p.y);
    }
    // Cut 30.5: the haul chest by the mouth — full, it hops and glints; open, the lid up a moment after the tap
    if (s.chest) {
      const c = CHEST_AT, st = s.chest.state, full = st === "full";
      const hop = full ? -Math.round(Math.max(0, Math.sin(t * 4)) * 2) : 0;
      sprite([`town_haul_chest${st === "closed" ? "" : `_${st}`}`, "town_haul_chest"], c.h, c.x, c.y + hop);
      if (full) {
        const g = (t * 0.7) % 1;
        sprite("fx_glint", 7, c.x + 5, c.y - c.h + 1 + hop, { z: 3, fade: Math.min(1, Math.abs(g - 0.5) * 2.4) });
        if (F.lights.length < MAX_LIGHTS) F.lights.push({ x: c.x, y: c.y - 6, r: 14 + night * 8, c: [0.9, 0.72, 0.3] });
      }
    }
    // Cut 30.5: the workers at their posts — an idle beat (two frames, desynchronised); the lit node's worker greyed, still; a new hire
    // walks out of the tent to his post
    for (const w of s.workers) {
      const p = workerAt(w); if (!p) continue;
      let x = p.x, y = p.y, moving = false, flip = false;
      const legs = arrive.get(w.id);
      if (legs?.length) {
        const at = walkerAt({ legs, from: legs[0]!.t0, to: Infinity } as Walker, t);
        if (at && t < legs[legs.length - 1]!.t1) { x = at.x; y = at.y; moving = at.moving; flip = at.dx < -0.01; }
        else arrive.delete(w.id);
      }
      const k = hashN(w.id.length, w.id.charCodeAt(0), w.id.charCodeAt(w.id.length - 1));
      const beat = 0.9 + (k % 50) / 100, phase = (k % 97) / 31;
      const step = moving && (Math.floor(t * 6.5) & 1) === 1;
      const frameN = w.lit || moving ? 0 : Math.floor((t + phase) / beat) & 1;
      if (w.lit) { const g = atlas.grey(workerIds(w.id), 24); if (inView(x, y, g.w, g.h)) push(g, Math.round(x), Math.round(y), zOf(y), flip, 1, 0); }
      else sprite(workerIds(w.id, frameN), 24, x, y - (step ? 1 : 0), { flip, dim: w.paused ? 0.7 : 1 });
      // week 2: his rank, quiet — a gilt pip over his head per rank past the first (II ·, III ··)
      if (!w.lit && w.rank > 1) for (let r = 0; r < w.rank - 1; r++) sprite("fx_dot_d8b45a", 2, x - (w.rank - 2) * 1.5 + r * 3, y - 26 - (step ? 1 : 0), { z: zOf(y) + 0.0003 });
      // at night the walkers' small MIST light: the dark coats would sink into the night grass
      if (night > 0.3 && F.lights.length < MAX_LIGHTS) F.lights.push({ x, y: y - 20, r: 18, c: [0.64 * 0.35 * night, 0.74 * 0.35 * night, 0.84 * 0.35 * night] });
    }
    // the pen: penned pets lie by the kennel
    s.penned.forEach((k, j) => { const p = PLOTS.kennel; sprite([k, "town_dog"], 10, p.x - 20 + j * 12, p.y + 8, { flip: j % 2 === 1 }); });
    // walkers
    let live = 0;
    for (const w of walkers) {
      const at = w.follow ? walkerAt(w.follow, t - w.lag) : walkerAt(w, t);
      if (!at || (w.follow && t < w.from) || t > w.to) continue;
      let x = at.x, y = at.y;
      if (w.follow && at.moving) { x -= Math.sign(at.dx) * 4; y += 2; }
      const flip = at.dx < -0.01;
      let fade = 0;
      if (w.emerge) { const age = t - w.from; if (age < 0.5) fade = 1 - age / 0.5; }
      if (w.vanish && w.to - t < 0.5) fade = Math.max(fade, 1 - (w.to - t) / 0.5);
      if (w === hero && walkingIn) { const g = (t - walkingIn.t0) / (walkingIn.t1 - walkingIn.t0); if (g > 0.75) fade = Math.min(1, (g - 0.75) * 4); }
      const step = at.moving && (Math.floor(t * 6.5 + w.lag * 3) & 1) === 1;
      const ids = step ? w.walk : w.sprite;
      sprite(ids, w.h, x, y - (step ? 1 : 0), { flip, fade });
      if (w.carry) {
        const back = (flip ? -1 : 1) * (w.carry === "town_chest_glow" ? 6 : 7);
        sprite(w.carry, w.carryH ?? 6, x + back, y - 8 + (w.carryH ?? 6) - (step ? 1 : 0), { z: zOf(y) + 0.0002, flip, fade });
        if (w.carry === "town_chest_glow") F.lights.push({ x: x + back, y: y - 8, r: 18, c: [1.0, 0.85, 0.4] });
      }
      live++;
    }
    F.live = live;
    // particles: smoke over the fire and the forge's chimney, embers, forge sparks, the build's dust
    tickParts(night);
    for (const p of parts) {
      const life = p.age / p.life;
      sprite(`fx_dot_${p.c}`, p.s, p.x, p.y, { z: p.kind === "smoke" ? 2.5 : 2.6, fade: p.kind === "smoke" ? Math.max(0, life * 1.3 - 0.3) : Math.max(0, life * 1.6 - 0.6) });
      if (p.kind === "spark" && night > 0.5 && F.lights.length < MAX_LIGHTS) F.lights.push({ x: p.x, y: p.y, r: 6, c: [1, 0.6, 0.2] });
    }
  }
  let emitT = 0, sparkT = 0, chimT = 0;
  function tickParts(night: number): void {
    void night;
    const dt = Math.min(0.1, t - emitT); emitT = t;
    const f = PLOTS.fire;
    const rand = Math.random;
    if (rand() < dt * 5) parts.push({ x: f.x + (rand() - 0.5) * 4, y: f.y - 10, vx: (rand() - 0.5) * 2, vy: -10 - rand() * 4, age: 0, life: 2.4 + rand(), kind: "smoke", c: SMOKE[Math.floor(rand() * 3)]!, s: rand() < 0.5 ? 2 : 3 });
    if (rand() < dt * 4) parts.push({ x: f.x + (rand() - 0.5) * 6, y: f.y - 6, vx: (rand() - 0.5) * 6, vy: -22 - rand() * 10, age: 0, life: 0.6 + rand() * 0.4, kind: "ember", c: EMBER[Math.floor(rand() * 3)]!, s: 1 });
    const smith = state!.buildings.find((b) => b.id === "blacksmith" && !(builtAt.has(b.id) && t - builtAt.get(b.id)! < 1.3));
    if (smith) {
      const p = PLOTS.blacksmith;
      if (t - sparkT > 1.5 + (hashN(Math.floor(t)) % 10) / 10) { sparkT = t; for (let i = 0; i < 7; i++) parts.push({ x: p.x - 14, y: p.y - 8, vx: (rand() - 0.5) * 40, vy: -20 - rand() * 25, age: 0, life: 0.4 + rand() * 0.3, kind: "spark", c: EMBER[Math.floor(rand() * 3)]!, s: 1 }); }
      if (t - chimT > 0.45) { chimT = t; parts.push({ x: p.x + 14 + (rand() - 0.5) * 2, y: p.y - p.h + 2, vx: 3 + rand() * 2, vy: -8 - rand() * 3, age: 0, life: 2.6 + rand(), kind: "smoke", c: SMOKE[Math.floor(rand() * 3)]!, s: 2 }); }
    }
    for (const [id, at] of builtAt) if (t - at >= 1.3 && t - at < 1.3 + dt + 0.001 && !dusted.has(id)) {
      dusted.add(id); const p = PLOTS[id as PlotId];
      for (let i = 0; i < 22; i++) { const a = rand() * Math.PI * 2, v = 14 + rand() * 22; parts.push({ x: p.x + Math.cos(a) * 14, y: p.y - 4, vx: Math.cos(a) * v, vy: Math.sin(a) * v * 0.4 - 6, age: 0, life: 0.7 + rand() * 0.5, kind: "dust", c: DUST[Math.floor(rand() * 3)]!, s: 2 }); }
    }
    for (let i = parts.length - 1; i >= 0; i--) {
      const p = parts[i]!; p.age += dt;
      if (p.age >= p.life) { parts.splice(i, 1); continue; }
      if (p.kind === "smoke") { p.vx += Math.sin(t * 1.3 + i) * dt * 2; p.s = p.age / p.life > 0.5 ? 3 : p.s; }
      if (p.kind === "spark") p.vy += 70 * dt;
      if (p.kind === "dust") { p.vx *= 1 - dt * 3; p.vy *= 1 - dt * 3; }
      p.x += p.vx * dt; p.y += p.vy * dt;
    }
    if (parts.length > 220) parts.splice(0, parts.length - 220);
  }
  const dusted = new Set<string>();

  // -- the camera: the stage's box fits the view, k = whole device px per texel --
  function fit(): void {
    const s = state; if (!s) return;
    const r = el.getBoundingClientRect(); const dpr = window.devicePixelRatio || 1;
    const W = Math.max(1, Math.round(r.width * dpr)), H = Math.max(1, Math.round(r.height * dpr));
    // the box: what stands (the mouth, the camp, the buildings, the staked plot) with a margin
    let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    const add = (p: Pt & { h: number }, hw: number): void => { x0 = Math.min(x0, p.x - hw); x1 = Math.max(x1, p.x + hw); y0 = Math.min(y0, p.y - p.h); y1 = Math.max(y1, p.y + 6); };
    add(PLOTS.mouth, 52); add(PLOTS.fire, 30); add(PLOTS.tent, 18); add(PLOTS.crate, 10);
    for (const b of s.buildings) add(PLOTS[b.id], 44);
    for (const w of s.workers) { const p = workerAt(w); if (p) add({ ...p, h: 24 }, 10); }
    if (s.staked) add({ ...PLOTS[s.staked.id], h: 22 }, 24);
    const M = 10; x0 -= M; x1 += M; y0 -= M; y1 += M + 4;
    // k: whole device px per texel — the largest that fits the box, or one more when that crops ≤ a margin's worth on each side
    const kf = Math.min(W / (x1 - x0), H / (y1 - y0));
    const up = Math.ceil(kf), fitsUp = W / up >= x1 - x0 - 2 * M - 4 && H / up >= y1 - y0 - 2 * M - 8;
    // (a low-density screen — fewer than 2 device px a texel — scales by a fraction instead: the town at k = 1 was a postage stamp)
    const k = up > kf && fitsUp ? up : kf < 2 ? Math.max(1, Math.floor(kf * 8) / 8) : Math.floor(kf);
    const w = Math.ceil(W / k), h = Math.ceil(H / k);
    let cx = (x0 + x1) / 2, cy = (y0 + y1) / 2;
    cx = Math.max(w / 2, Math.min(MAP_W * TILE - w / 2, cx)); cy = Math.max(h / 2, Math.min(MAP_H * TILE - h / 2, cy));
    const nx0 = Math.round(cx - w / 2), ny0 = Math.round(cy - h / 2);
    if (nx0 !== F.x0 || ny0 !== F.y0 || w !== F.w || h !== F.h || k !== F.k || W !== F.W || H !== F.H) { F.x0 = nx0; F.y0 = ny0; F.w = w; F.h = h; F.k = k; F.W = W; F.H = H; layoutDirty = true; }
  }

  function loop(now: number): void {
    raf = 0;
    if (disposed) return;
    if (document.hidden) { last = now; return; }   // hidden: no frames (visibilitychange resumes)
    if (!el.isConnected) { last = now; raf = requestAnimationFrame(loop); return; }   // (not mounted yet)
    const idle = now - lastInput > IDLE_MS;
    // the frame cap: none while input arrives, IDLE_FPS after 10 s without; a software GL under automation (juice off: the headless
    // suites, seven browsers on the CPU) draws at SOFT_FPS — the scene is cosmetic and its frames were the suite's load
    const cap = document.documentElement.dataset.juice === "off" ? SOFT_FPS : idle ? IDLE_FPS : 0;
    if (cap && now - lastDraw < 1000 / cap - 2) { raf = requestAnimationFrame(loop); return; }
    const dt = Math.min(0.1, (now - last) / 1000); last = now; lastDraw = now;
    t += dt;
    if (state) {
      const c0 = performance.now();
      fit();
      frame();
      if (atlas.version !== atlasSeen) { atlasSeen = atlas.version; F.groundKey = F.groundKey + "*"; }
      if (gl && !glLost) { gl.render(F); if (c2d) c2d.style.display = "none"; }
      else { const c = view2d(); c.style.display = "block"; drawTown2D(c, F, atlas.canvas); }
      cpu.push(performance.now() - c0); if (cpu.length > 120) cpu.shift();
      frames++; fpsWin.push(now); while (fpsWin.length && fpsWin[0]! < now - 1000) fpsWin.shift();
      if (walkingIn && t >= walkingIn.t1) { const d = walkingIn.done; walkingIn = null; d(); }
      if (layoutDirty) { layoutDirty = false; for (const fn of layoutFns) fn(); }
    }
    raf = requestAnimationFrame(loop);
  }
  let atlasSeen = -1;
  const start = (): void => { if (!raf && !disposed) { last = performance.now(); raf = requestAnimationFrame(loop); } };
  const onVis = (): void => { if (!document.hidden) start(); };
  document.addEventListener("visibilitychange", onVis);
  const poke = (): void => { lastInput = performance.now(); };
  const inputs = ["pointerdown", "pointermove", "keydown", "wheel", "touchstart"] as const;
  for (const ev of inputs) window.addEventListener(ev, poke, { passive: true });
  const ro = typeof ResizeObserver !== "undefined" ? new ResizeObserver(() => { fit(); layoutDirty = true; }) : null;
  ro?.observe(el);

  const css = (x: number, y: number): Pt => { const dpr = window.devicePixelRatio || 1; return { x: (x - F.x0) * F.k / dpr, y: (y - F.y0) * F.k / dpr }; };
  return {
    el,
    setState(s) {
      const prev = state; state = s;
      // a building the player has not seen yet goes up on this mount: scaffold → built
      for (const b of s.buildings) if (b.fresh && !builtAt.has(b.id) && !prev?.buildings.some((x) => x.id === b.id)) builtAt.set(b.id, t + 0.4);
      // a worker hired since the last state (he stood greyed, or nowhere): out of the tent, along the paths, to his post
      if (prev) for (const w of s.workers) {
        if (w.lit || prev.workers.some((x) => x.id === w.id && !x.lit)) continue;
        const p = workerAt(w); if (!p) continue;
        const legs: Leg[] = []; legsAlong([NODES.tent!, ...route("tent", nearestNode(p)), p], t + 0.2, SPEED * 1.3, legs); arrive.set(w.id, legs);
      }
      const key = (x: TownState | null): string => x ? JSON.stringify([x.hero, x.pets, x.parties.length, x.buildings.map((b) => b.id), x.seed]) : "";
      if (!prev || key(prev) !== key(s)) build();
      fit(); layoutDirty = true; start();
    },
    walkIn(ms = 700) {
      if (!hero || !state) return Promise.resolve();
      if (walkingIn) return new Promise((res) => { const d = walkingIn!.done; walkingIn!.done = () => { d(); res(); }; });
      const at = walkerAt(hero, t) ?? { x: NODES.tent!.x, y: NODES.tent!.y };
      const pts = [{ x: at.x, y: at.y }, ...route(nearestNode(at), "in")];
      let len = 0; for (let i = 1; i < pts.length; i++) len += dist(pts[i - 1]!, pts[i]!);
      const dur = ms / 1000;
      hero.from = Math.min(hero.from, t); hero.legs = []; hero.extend = undefined; hero.to = t + dur + 0.05; hero.vanish = false;
      legsAlong(pts, t, Math.max(SPEED, len / dur), hero.legs);
      poke();
      return new Promise((res) => { walkingIn = { t0: t, t1: t + dur, done: res }; start(); });
    },
    resize() { fit(); layoutDirty = true; },
    rectOf(id) {
      if (!state) return null;
      if (id === "chest" || id.startsWith("worker:")) {   // (the chest and the workers stand off the plots)
        const wk = id === "chest" ? null : state.workers.find((w) => `worker:${w.id}` === id);
        const p = id === "chest" ? (state.chest ? CHEST_AT : null) : wk ? workerAt(wk) : null;
        if (!p) return null;
        const s = id === "chest" ? atlas.get(["town_haul_chest"], CHEST_AT.h) : atlas.get(workerIds(wk!.id), 24);
        const a = css(p.x - s.w / 2, p.y - s.h), b = css(p.x + s.w / 2, p.y);
        return { x: a.x, y: a.y, w: b.x - a.x, h: b.y - a.y };
      }
      const plot = id === "staked" ? (state.staked ? PLOTS[state.staked.id] : null) : PLOTS[id as PlotId];
      if (!plot) return null;
      const ids: Record<string, string[]> = { mouth: ["town_mouth_cave"], fire: ["town_campfire_0"], tent: ["town_tent"], crate: ["town_crate"], board: ["town_board"] };
      let s: Slot;
      if (id === "staked") s = atlas.get("town_plot", 16);
      else if ((BUILDINGS as readonly string[]).includes(id)) { const b = state.buildings.find((x) => x.id === id); if (!b) return null; s = atlas.get([`town_${id}_${b.look}`, `town_${id}_1`], plot.h); }
      else s = atlas.get(ids[id] ?? [id], plot.h);
      const a = css(plot.x - s.w / 2, plot.y - s.h), b = css(plot.x + s.w / 2, plot.y);
      return { x: a.x, y: a.y, w: b.x - a.x, h: b.y - a.y };
    },
    onLayout(fn) { layoutFns.push(fn); },
    stats() {
      const s = state;
      const sorted = [...cpu].sort((a, b) => a - b);
      return { fps: fpsWin.length, frames, cpuMs: cpu[cpu.length - 1] ?? 0, cpuP95: sorted[Math.floor(sorted.length * 0.95)] ?? 0, walkers: F.live, mode: gl && !glLost ? "gl" : "2d", glLost, k: F.k, night: F.night, idle: performance.now() - lastInput > IDLE_MS, running: !!raf,
        buildings: s?.buildings.map((b) => b.id) ?? [], staked: s?.staked?.id, markers: s?.markers.map((m) => `${m.kind}@${m.at}`) ?? [], view: [F.x0, F.y0, F.w, F.h],
        chest: s?.chest?.state, workers: s?.workers.map((w) => `${w.id}${w.lit ? "(lit)" : w.paused ? "(off)" : ""}`) ?? [],
        ranks: Object.fromEntries((s?.workers ?? []).filter((w) => !w.lit && w.rank > 1).map((w) => [w.id, w.rank])) };
    },
    stress(n) { stressN = n; if (state) addStress(); },
    setHour(h) { hourOverride = h; },
    poke,
    dispose() {
      disposed = true; if (raf) cancelAnimationFrame(raf); raf = 0;
      document.removeEventListener("visibilitychange", onVis);
      for (const ev of inputs) window.removeEventListener(ev, poke);
      ro?.disconnect(); glCanvas.removeEventListener("webglcontextlost", onLost); glCanvas.removeEventListener("webglcontextrestored", onRestored);
      glCanvas.remove(); c2d?.remove();
      // the context is kept for the next mount (never left for the browser to reap: a dozen camps held a dozen contexts, and Chrome's
      // cap then forced the oldest live one — the watch's — lost); a lost one is let go
      if (gl && !glLost) { sharedGl?.gl.dispose(); sharedGl = { canvas: glCanvas, gl, lost: false }; } else gl?.dispose();
    },
  };
}
function segDist(p: Pt, a: Pt, b: Pt): number {
  const dx = b.x - a.x, dy = b.y - a.y, l2 = dx * dx + dy * dy;
  const u = l2 ? Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / l2)) : 0;
  return Math.hypot(p.x - (a.x + u * dx), p.y - (a.y + u * dy));
}

// ---- the GL view: one target of town texels, the light pass upscaling it ---------------------------------------------------------

const MAX_LIGHTS = 24;
const BLIT_VERT = /* glsl */ `varying vec2 vUv; void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;
const BLIT_FRAG = /* glsl */ `
uniform sampler2D tex;
uniform vec2 uSize;      // the target, texels
uniform float uK;        // device px per texel
uniform float uDevH;
uniform vec3 uAmb;
uniform float uNight;
uniform vec4 uL[${MAX_LIGHTS}];   // x, y (texels from the view's top-left), radius, unused
uniform vec3 uC[${MAX_LIGHTS}];
uniform int uN;
uniform vec2 uOrigin;    // the view's top-left in world texels (the moonlight's patches stay on the ground)
uniform float uTime;
${BAYER_GLSL}
void main() {
  vec2 px = floor(vec2(gl_FragCoord.x, uDevH - gl_FragCoord.y) / uK);
  vec2 uv = vec2((px.x + 0.5) / uSize.x, 1.0 - (px.y + 0.5) / uSize.y);
  vec3 c = texture2D(tex, uv).rgb;
  vec3 light = vec3(0.0);
  float d4 = bayer4(px);
  for (int i = 0; i < ${MAX_LIGHTS}; i++) {
    if (i >= uN) break;
    vec2 d = px - uL[i].xy;
    float f = max(0.0, 1.0 - length(d * vec2(1.0, 1.25)) / uL[i].z);
    f = floor(f * f * 6.0 + d4) / 6.0;   // banded, dithered falloff: a pixel-art pool
    light += uC[i] * f;
  }
  // moonlight in drifting patches (the art note: the grass read flat): a slow, broad value swell over the ground, banded
  vec2 w = px + uOrigin;
  float m = sin(w.x * 0.021 + uTime * 0.04) * sin(w.y * 0.027 - uTime * 0.03) + 0.5 * sin((w.x + w.y) * 0.013 + 1.7);
  m = floor((0.5 + 0.33 * m) * 5.0 + d4) / 5.0;
  // night: what the warm light does not reach is graded toward the moon (cool, desaturated), so a lit window or the fire reads
  // against a blue world rather than a darker copy of the day
  float lit = clamp(dot(light, vec3(0.45, 0.35, 0.2)) * 2.0, 0.0, 1.0);
  float lum = dot(c, vec3(0.299, 0.587, 0.114));
  c = mix(c, vec3(0.62, 0.8, 1.2) * lum, 0.65 * uNight * (1.0 - lit));
  vec3 col = c * (uAmb * (0.84 + 0.36 * m) + light) + light * light * (0.05 + 0.12 * uNight) * (0.4 + uNight);
  // a soft vignette: the corners a step down (the night's more)
  vec2 q = px / uSize - 0.5;
  col *= 1.0 - (0.18 + 0.22 * uNight) * smoothstep(0.32, 0.75, length(q * vec2(1.0, 0.8)));
  gl_FragColor = vec4(col, 1.0);
}`;
class GLTown {
  private renderer: THREE.WebGLRenderer;
  private rt: THREE.WebGLRenderTarget;
  private scene = new THREE.Scene();
  private camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 1, 200);
  private tex: THREE.CanvasTexture;
  private groundL: QuadLayer; private objL: QuadLayer;
  private blitScene = new THREE.Scene(); private blitCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private mat: THREE.ShaderMaterial;
  private groundKey = ""; private atlasV = -1;
  private lv = Array.from({ length: MAX_LIGHTS }, () => new THREE.Vector4());
  private lc = Array.from({ length: MAX_LIGHTS }, () => new THREE.Vector3());
  private canvas: HTMLCanvasElement; private atlas: TownAtlas;
  constructor(canvas: HTMLCanvasElement, atlas: TownAtlas) {
    this.canvas = canvas; this.atlas = atlas;
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: false, alpha: false, stencil: false, depth: false, powerPreference: "high-performance", premultipliedAlpha: false });
    this.renderer.setPixelRatio(1);
    this.rt = new THREE.WebGLRenderTarget(4, 4, { minFilter: THREE.NearestFilter, magFilter: THREE.NearestFilter, generateMipmaps: false, depthBuffer: true, stencilBuffer: false, colorSpace: THREE.NoColorSpace });
    this.tex = new THREE.CanvasTexture(atlas.canvas);
    Object.assign(this.tex, { minFilter: THREE.NearestFilter, magFilter: THREE.NearestFilter, generateMipmaps: false, colorSpace: THREE.NoColorSpace, flipY: true, premultiplyAlpha: false });
    this.groundL = new QuadLayer(this.tex, MAP_W * MAP_H + 16, 1, 0);
    this.objL = new QuadLayer(this.tex, 1600, 1, 1);
    this.scene.add(this.groundL.mesh, this.objL.mesh);
    this.camera.position.set(0, 0, 100);
    this.mat = new THREE.ShaderMaterial({ vertexShader: BLIT_VERT, fragmentShader: BLIT_FRAG, depthTest: false, depthWrite: false,
      uniforms: { tex: { value: this.rt.texture }, uSize: { value: new THREE.Vector2(4, 4) }, uK: { value: 1 }, uDevH: { value: 4 }, uAmb: { value: new THREE.Vector3(1, 1, 1) }, uNight: { value: 0 },
        uL: { value: this.lv }, uC: { value: this.lc }, uN: { value: 0 }, uOrigin: { value: new THREE.Vector2() }, uTime: { value: 0 } } });
    this.blitScene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.mat));
  }
  restored(): void { this.groundKey = ""; this.atlasV = -1; this.tex.needsUpdate = true; }
  /** the GL context is gone (forced lost on dispose, or by the browser) */
  isLost(): boolean { return this.renderer.getContext().isContextLost(); }
  render(F: TownFrame): void {
    const r = this.renderer;
    if (this.canvas.width !== F.W || this.canvas.height !== F.H) r.setSize(F.W, F.H, false);
    if (this.rt.width !== F.w || this.rt.height !== F.h) this.rt.setSize(F.w, F.h);
    if (this.atlas.version !== this.atlasV) { this.atlasV = this.atlas.version; this.tex.needsUpdate = true; }
    if (F.groundKey !== this.groundKey) {
      this.groundKey = F.groundKey; const L = this.groundL; L.begin();
      for (const q of F.ground) L.push(q.x, -q.y, 0, q.w, q.h, q.s.u0, q.s.v0, q.s.u1, q.s.v1);
      L.end();
    }
    const O = this.objL; O.begin();
    for (let i = 0; i < F.n; i++) { const q = F.quads[i]!; O.push(q.x, -q.y, q.z, q.w, q.h, q.s.u0, q.s.v0, q.s.u1, q.s.v1, q.dim, 0, q.fade, q.flip ? 1 : 0); }
    O.end();
    const c = this.camera; c.left = F.x0; c.right = F.x0 + F.w; c.top = -F.y0; c.bottom = -(F.y0 + F.h); c.updateProjectionMatrix();
    r.setRenderTarget(this.rt); r.setClearColor(0x1b2412, 1); r.clear(); r.render(this.scene, c);
    const u = this.mat.uniforms;
    (u.uSize!.value as THREE.Vector2).set(F.w, F.h); u.uK!.value = F.k; u.uDevH!.value = F.H; (u.uAmb!.value as THREE.Vector3).set(...F.amb); u.uNight!.value = F.night;
    const n = Math.min(MAX_LIGHTS, F.lights.length);
    for (let i = 0; i < n; i++) { const l = F.lights[i]!; this.lv[i]!.set(l.x - F.x0, l.y - F.y0, l.r, 0); this.lc[i]!.set(...l.c); }
    u.uN!.value = n; (u.uOrigin!.value as THREE.Vector2).set(F.x0, F.y0); u.uTime!.value = performance.now() / 1000;
    r.setRenderTarget(null); r.render(this.blitScene, this.blitCam);
  }
  // (the context is let go, not left for the browser to reap: a camp mounted a dozen times held a dozen contexts, and Chrome's cap
  // then forced the oldest live one — the watch's — lost)
  dispose(): void { this.groundL.dispose(); this.objL.dispose(); this.mat.dispose(); this.rt.dispose(); this.tex.dispose(); this.renderer.dispose(); this.renderer.forceContextLoss(); }
}
