// Fake Engine: a tiny deterministic mini-sim with canned-shaped output so the UI can be built and
// exercised before the Rust core lands. Not game truth. Selected with ?engine=fake or when pkg/ is absent.
import type {
  BonesPile, Combo, Companion, Cond, Counter, Death, Engine, Entity, Ev, ExitLine, FloorItem, Forecast, Highlight, InvItem, LedgerRow, Lineage, Overlay,
  Patch, ReturnReport, Row, RuleSet, Snapshot, StepResult, Stall, SupplyEntry, Tile, Trace, UnlockInfo, Verb, Vocabulary, Because,
} from "./types";
import { CLASSES, XP_LEVEL_CAP, isFreeClass, verbsAt, verbsUpTo, xpToNext } from "./classes";
import { combosIn } from "../ui/tokens";

/** Cut 8B §1: the combo table, as the core's `rules::COMBOS` (adjacent-row verb pairs the engine names). */
export const COMBOS: Combo[] = [
  { a: "shield_bash", b: "backstab", name: "opener" }, { a: "shield_bash", b: "attack", name: "opener" },
  { a: "throw", b: "retreat", name: "hit and fade" }, { a: "throw", b: "back_corridor", name: "hit and fade" },
  { a: "taunt", b: "cleave", name: "bait" }, { a: "shoot", b: "kite", name: "kite" }, { a: "shoot", b: "retreat", name: "kite" },
  { a: "vanish", b: "backstab", name: "ambush" }, { a: "pray", b: "descend", name: "pilgrim" }, { a: "tame", b: "send", name: "handler" },
  { a: "drink unknown", b: "attack", name: "gambler" }, { a: "back_corridor", b: "attack", name: "chokepoint" },
];

type Rng = () => number;
function mulberry32(seed: number): Rng {
  let a = seed | 0;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const ri = (r: Rng, lo: number, hi: number): number => lo + Math.floor(r() * (hi - lo + 1));
const pick = <T>(r: Rng, a: T[]): T => a[Math.floor(r() * a.length)];
const hash = (s: string): number => { let h = 2166136261; for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); } return h >>> 0; };

const W = 24, H = 24, VIS = 7;
const BIOMES: [string, number, number][] = [["warrens", 1, 5], ["fens", 6, 10], ["crypt", 11, 15]];
const biomeOf = (d: number): string => (BIOMES.find(([, lo, hi]) => d >= lo && d <= hi) ?? BIOMES[2])[0];

type MonDef = { hp: number; atk: [number, number]; def: number; tags: string[]; lo: number; hi: number };
const MON: Record<string, MonDef> = {
  rat:             { hp: 4,  atk: [1, 2], def: 0, tags: [],                  lo: 1,  hi: 3 },
  jackal:          { hp: 5,  atk: [1, 3], def: 0, tags: ["pack", "fast"],    lo: 1,  hi: 5 },
  goblin:          { hp: 8,  atk: [2, 4], def: 1, tags: [],                  lo: 2,  hi: 6 },
  goblin_archer:   { hp: 6,  atk: [2, 4], def: 0, tags: ["ranged"],          lo: 3,  hi: 7 },
  goblin_conjurer: { hp: 7,  atk: [1, 3], def: 0, tags: ["caster"],          lo: 4,  hi: 8 },
  monkey:          { hp: 5,  atk: [1, 2], def: 1, tags: ["thief"],           lo: 2,  hi: 6 },
  ogre:            { hp: 20, atk: [4, 8], def: 1, tags: ["heavy"],           lo: 4,  hi: 9 },
  bloat:           { hp: 4,  atk: [0, 1], def: 0, tags: ["gas"],             lo: 6,  hi: 10 },
  pink_jelly:      { hp: 12, atk: [1, 3], def: 0, tags: ["splitter"],        lo: 6,  hi: 10 },
  eel:             { hp: 10, atk: [3, 7], def: 2, tags: ["water"],           lo: 6,  hi: 10 },
  skeleton:        { hp: 12, atk: [3, 6], def: 2, tags: ["undead"],          lo: 11, hi: 15 },
  ghoul:           { hp: 10, atk: [2, 5], def: 1, tags: ["undead", "pack"],  lo: 11, hi: 15 },
  wraith:          { hp: 9,  atk: [2, 4], def: 1, tags: ["undead"],          lo: 12, hi: 15 },
  goblin_warlord:  { hp: 30, atk: [4, 8], def: 2, tags: ["boss"],            lo: 5,  hi: 5 },
  bloat_mother:    { hp: 36, atk: [2, 4], def: 1, tags: ["boss", "gas"],     lo: 10, hi: 10 },
  lich:            { hp: 45, atk: [5, 9], def: 3, tags: ["boss", "undead"],  lo: 15, hi: 15 },
  blade:           { hp: 3,  atk: [1, 3], def: 0, tags: [],                  lo: 99, hi: 99 },
};
const BOSS: Record<number, string> = { 5: "goblin_warlord", 10: "bloat_mother", 15: "lich" };
// Cut 6 §5: the counter each boss teaches on first sight, as a row (the core's fact carries the row text)
const COUNTER: Record<string, { row: Row; text: string }> = {
  goblin_warlord: { row: { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "tag:boss" } }, text: "attack boss" },
  bloat_mother: { row: { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "throw", a: "fire,boss" } }, text: "throw fire, boss" },
  lich: { row: { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "read", a: "silence" } }, text: "read silence" },
};
// Cut 6 §6: a card's rows / an automation's effect as a row, for the sheet behind `[card]` rows and owned autos
const UNLOCK_ROWS: Record<string, Row[]> = {
  corridor_fighting: [{ conds: [{ k: "foes>=", n: 2 }, { k: "in_corridor" }], verb: { v: "hold" } }, { conds: [{ k: "foes>=", n: 2 }], verb: { v: "back_corridor" } }],
  kite_archers: [{ conds: [{ k: "foe_tag", t: "ranged" }], verb: { v: "kite" } }],
  stair_dance: [{ conds: [{ k: "foe_tag", t: "boss" }, { k: "path_stairs" }], verb: { v: "descend" } }],
  gas_step: [{ conds: [{ k: "foe_tag", t: "gas" }, { k: "adj>=", n: 1 }], verb: { v: "retreat" } }],
  pack_break: [{ conds: [{ k: "foe_tag", t: "pack" }], verb: { v: "back_corridor" } }, { conds: [{ k: "adj>=", n: 2 }], verb: { v: "attack", a: "lowest" } }],
  thief_guard: [{ conds: [{ k: "foe_tag", t: "thief" }], verb: { v: "attack", a: "tag:thief" } }],
  boss_focus: [{ conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "tag:boss" } }],
  last_stand: [{ conds: [{ k: "hp<", n: 20 }, { k: "adj>=", n: 2 }], verb: { v: "attack", a: "lowest" } }],
  // automations: one row-like entry, the core's shape (`{conds: [], verb: {v: "auto", a: "keeps best weapon+armour"}}`)
  quartermaster: [{ conds: [], verb: { v: "auto", a: "keeps best weapon+armour" } }],
  auto_supply: [{ conds: [], verb: { v: "auto", a: "restocks supplies" } }],
  auto_insure: [{ conds: [], verb: { v: "auto", a: "insures brought items" } }],
  incubator: [{ conds: [], verb: { v: "auto", a: "hatches eggs at rest" } }],
  bone_sense: [{ conds: [], verb: { v: "auto", a: "paths to bones" } }],
};
const POTIONS = ["heal", "strength", "speed", "invisibility", "poison", "caustic", "confusion", "fire"];
const FLAVOURS = ["blue", "red", "green", "murky", "clear", "amber", "violet", "black"];
const SCROLLS = ["teleport", "blink", "fear", "mapping", "identify", "enchant", "darkness", "summon_ally", "aggravate"];
const RUNES = ["ZELGO", "FOOBIE", "ELBIB", "VERR", "KIRJE", "DAIYEN", "LEP", "PRATYAVAYAH", "TEMOV"];
const THROWABLE = new Set(["poison", "caustic", "confusion", "fire"]);
const TRAITS = ["curious", "greedy", "cowardly", "brave"];
// Cut 5 §2 chronicle line, the contract's grammar: `♟3 the greedy fighter · D7 · took the Warlord · fell to gas · left bones on D7.`
const CHRONICLE_CAP = 40;
function chronicleLine(heir: number, trait: string, cls: string, depth: number, deed: string | undefined, end: string, tail: string | undefined, set?: string): string {
  const parts = [`♟${heir} the ${trait} ${cls}`, `D${depth}`];
  if (set) parts.push(`"${set}" set`);
  if (deed) parts.push(deed);
  parts.push(end);
  if (tail) parts.push(tail);
  return `${parts.join(" · ")}.`;
}
const SEED_CHRONICLE: [string, string, number, string | undefined, string, string | undefined][] = [
  ["curious", "fighter", 3, undefined, "fell to a jackal pack", "left bones on D3"],
  ["greedy", "fighter", 5, "took the Warlord", "banked $140", undefined],
  ["brave", "fighter", 7, "freed a captive", "fell to gas", "left bones on D7"],
  ["cowardly", "rogue", 6, "found ♟3's bones", "returned twice", undefined],
  ["greedy", "rogue", 9, "tamed a jackal", "fell to an ogre", "left bones on D9"],
  ["curious", "ranger", 10, "took the Bloat Mother", "fell to a ghoul", "left bones on D10"],
];
const WEAPONS: Record<string, [number, number]> = { dagger: [2, 4], sword: [3, 7], axe: [4, 9], bow: [2, 6] };
const ARMOUR: Record<string, number> = { leather: 1, mail: 3, plate: 5 };

// Cut 2 §3 catalogue: cost in marks; `needs` = the fact/trophy gate as the player reads it, `gate` decides it.
type UnlockDef = { cost: number; needs?: string; gate?: (L: Lineage) => boolean };
const bossKills = (L: Lineage): number => L.trophies.filter((t) => t.startsWith("boss:")).length;
const hasFact = (re: RegExp) => (L: Lineage): boolean => L.facts.some((f) => re.test(f));
export const UNLOCKS: Record<string, UnlockDef> = {
  row5: { cost: 2 }, row6: { cost: 4 }, row7: { cost: 7 }, row8: { cost: 11 },
  party_slot_2: { cost: 4, needs: "tamed ≥ 1", gate: (L) => L.ledger.filter((r) => r.tamed).length >= 1 },
  party_slot_3: { cost: 9, needs: "tamed ≥ 3", gate: (L) => L.ledger.filter((r) => r.tamed).length >= 3 },
  vault2: { cost: 3 }, vault3: { cost: 6 }, vault4: { cost: 10 },
  // Cut 8B §2–3: the rogue is free at the first bank; `tame` costs nothing and is owned from the start
  rogue: { cost: 0, needs: "bank once", gate: (L) => (L.gold_ledger ?? []).some((g) => g.why.startsWith("banked")) },
  ranger: { cost: 6, needs: "boss 1", gate: (L) => bossKills(L) >= 1 }, caster: { cost: 8, needs: "boss 2", gate: (L) => bossKills(L) >= 2 },
  tame: { cost: 0, needs: "item: leash", gate: hasFact(/^item:leash$/) }, throw: { cost: 2 },
  cond_alert: { cost: 2, needs: "foe: any", gate: hasFact(/^foe:/) }, cond_turns: { cost: 2, needs: "D3", gate: (L) => L.best_depth >= 3 },
  cond_loot: { cost: 2, needs: "D2", gate: (L) => L.best_depth >= 2 }, cond_on_kill: { cost: 2, needs: "foe: any", gate: hasFact(/^foe:/) },
  cond_on_see: { cost: 2, needs: "foe: any", gate: hasFact(/^foe:/) }, cond_party_hp: { cost: 2, needs: "tamed ≥ 1", gate: (L) => L.ledger.some((r) => r.tamed) },
  corridor_fighting: { cost: 3, needs: "foe: pack", gate: hasFact(/^foe:[a-z_]+:pack$/) }, kite_archers: { cost: 3, needs: "foe: ranged", gate: hasFact(/^foe:[a-z_]+:ranged$/) },
  stair_dance: { cost: 3, needs: "D3", gate: (L) => L.best_depth >= 3 }, gas_step: { cost: 3, needs: "foe: gas", gate: hasFact(/^foe:[a-z_]+:gas$/) },
  pack_break: { cost: 3, needs: "foe: pack", gate: hasFact(/^foe:[a-z_]+:pack$/) }, thief_guard: { cost: 3, needs: "foe: thief", gate: hasFact(/^foe:[a-z_]+:thief$/) },
  boss_focus: { cost: 3, needs: "boss 1", gate: (L) => bossKills(L) >= 1 }, last_stand: { cost: 3, needs: "D5", gate: (L) => L.best_depth >= 5 },
  quartermaster: { cost: 5 }, auto_supply: { cost: 4 }, auto_insure: { cost: 6 },
  incubator: { cost: 4, needs: "egg ≥ 1", gate: (L) => L.eggs.length >= 1 }, supply_cap_5: { cost: 3 },
  bone_sense: { cost: 3, needs: "bones", gate: hasFact(/^bones:/) }, third_tag: { cost: 6, needs: "bred ≥ 1", gate: (L) => L.ledger.some((r) => r.bred) },
}; // ids as the core's meta::UNLOCKS
export const UNLOCK_COST: Record<string, number> = Object.fromEntries(Object.entries(UNLOCKS).map(([k, v]) => [k, v.cost]));
const UNLOCK_PREREQ: Record<string, string> = { row6: "row5", row7: "row6", row8: "row7", vault3: "vault2", vault4: "vault3", party_slot_3: "party_slot_2" };
const FORGE_LADDER = [{ need: 5, label: "craftable" }, { need: 15, label: "tier 1" }, { need: 40, label: "tier 2" }];   // Cut 9 §10: the core's `FORGE_LADDER`
const TACTIC_CARDS = ["corridor_fighting", "kite_archers", "stair_dance", "gas_step", "pack_break", "thief_guard", "boss_focus", "last_stand"];
const COND_UNLOCK: Record<string, string> = { "alert>=": "cond_alert", "turns>": "cond_turns", "loot>=": "cond_loot", on_kill: "cond_on_kill", on_see: "cond_on_see", "party_hp<": "cond_party_hp" };
const REST_CAP_S = 30 * 60, WAKE_S = 20 * 60, BONES_MAX = 3, STUDIED_KILLS = 5, GOLD_LEDGER_CAP = 20, EXITS_CAP = 5, TRACE_TURNS = 10;
// UI dev knob: `?engine=fake&fake_depth=5` starts every run on D5 (boss floor) so the boss HUD can be seen.
const DEV_START_DEPTH = Math.max(1, (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_depth"))) || 1);
// UI dev knob: `?engine=fake&fake_vision=4` reports the Deep's sight radius (renderer fog bands follow it).
const DEV_VISION = (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_vision"))) || 7;
const SALVAGE: Record<string, number> = { dagger: 10, sword: 20, axe: 30, bow: 20, leather: 15, mail: 25, plate: 40, leash: 5 };
const salvageOf = (kind: string): number => SALVAGE[kind] ?? (POTIONS.includes(kind) ? 8 : SCROLLS.includes(kind) ? 12 : 0);
const TAG_VERB: Record<string, string> = { ranged: "shoot", gas: "burst", thief: "steal", splitter: "split", pack: "flank", undead: "drain" };
const LEDGER_KINDS = Object.keys({ rat: 1, jackal: 1, goblin: 1, goblin_archer: 1, goblin_conjurer: 1, monkey: 1, ogre: 1, bloat: 1, pink_jelly: 1, eel: 1, skeleton: 1, ghoul: 1, wraith: 1, goblin_warlord: 1, bloat_mother: 1, lich: 1 });
function defaultCompanionRules(tags: string[]): RuleSet {
  const v = tags.map((t) => TAG_VERB[t]).find((x) => x) ?? "attack";
  return { rows: [{ conds: [{ k: "self_hp<", n: 30 }], verb: { v: "recall" } }, { conds: [{ k: "adj>=", n: 1 }], verb: { v } }] };
}
function mkCompanion(id: number, kind: string, level: number, tags: string[], gen: number): Companion {
  const d = MON[kind] ?? MON.rat; const hp = Math.round(d.hp * (1 + 0.15 * (level - 1)));
  return { id, kind, name: kind.replace(/_/g, " "), level, tags, gen, rules: defaultCompanionRules(tags), max_rows: 1 + level, hp, max_hp: hp };
}

const PRESET_FIGHTER: RuleSet = { name: "fighter", rows: [
  { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "unknown" } },
  { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } },
] };

// --- text form of rules (export/import) ---
export function condText(c: Cond): string {
  if (c.n !== undefined) return `${c.k}${c.n}`;
  if (c.t !== undefined) return `${c.k}:${c.t}`;
  return c.k;
}
export function verbText(v: Verb): string { return v.a ? `${v.v} ${v.a}` : v.v; }
export function rowText(r: Row): string { return `${r.conds.map(condText).join(" · ")} → ${verbText(r.verb)}`; }
function parseCond(s: string): Cond | null {
  const m = /^([a-z_]+[<>]=?)(\d+)$/.exec(s);
  if (m) return { k: m[1], n: +m[2] };
  const t = /^([a-z_]+):([a-z_:]+)$/.exec(s);
  if (t) return { k: t[1], t: t[2] };
  return /^[a-z_]+$/.test(s) ? { k: s } : null;
}
export function parseRules(text: string): RuleSet {
  const rows: Row[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    const [lhs, rhs] = line.split("→").map((s) => s.trim());
    if (rhs === undefined) continue;
    const conds = lhs ? lhs.split("·").map((s) => parseCond(s.trim())).filter((c): c is Cond => !!c) : [];
    const [v, a] = rhs.split(/\s+/);
    rows.push({ conds: conds.slice(0, 2), verb: a ? { v, a } : { v } });
  }
  return { rows };
}

// --- world ---
type Rect = { x: number; y: number; w: number; h: number };
type Mon = Entity & { def: number; atk: [number, number]; wind: number; drawn: boolean; stun: number; summoned: boolean; rules?: RuleSet; sent?: boolean; seen?: boolean };
type Floor = { tiles: Tile[]; seen: boolean[]; visible: boolean[]; overlays: Overlay[]; rooms: Rect[]; items: FloorItem[]; mons: Mon[]; roomOf: Int8Array };
type Hero = { x: number; y: number; hp: number; max_hp: number; atk: [number, number]; def: number; inv: InvItem[];
  weapon?: string; armour?: string; invis: number; stun: number; bashCd: number; vanishCd: number; speedT: number };
type TraceTurn = Trace["turns"][number];
type Run = {
  id: number; heir: number; seed: number; rng: Rng; depth: number; turn: number; started_turn: number; floorTurn: number;
  floor: Floor; hero: Hero; loot: number; alert: number; trace: TraceTurn[]; over: boolean; exit?: "bank" | "return" | "death";
  kills: Record<string, number>; hl: Highlight[]; facts: string[]; known: Set<string>;
  lastHurt?: { cause: string; dmg: number; hpBefore: number }; hurt: boolean; killed: boolean; saw: boolean;
  nextId: number; picked: InvItem[]; nearDeath: boolean; kindsSeen: Set<string>; brought: number[]; cls: string; trait: string;
  loot_kept: number; cause?: string; goal?: "descend" | "bank";
  party: Companion[]; recalled: Mon[]; tamed: Mon[]; lostC: Companion[];
  level: number; killXp: number; windUsed: boolean; bulwark: number; cleaveCd: number;
  gear: InvItem[]; score: number;
  bonesFound: BonesPile[]; bonesPiles: BonesPile[]; rest_s: number; insured: Set<number>; broughtItems: InvItem[];
  gold: number; spent: { label: string; price: number }[]; line?: ExitLine;                  // Cut 6 §1: the ledger line at the exit
  lastRows?: { row: number; why: string }[];                                                  // Cut 6 §3: row accounting of the last action
  prov: Record<string, Prov>;                                                                 // Cut 11 §1: per slot (`item:heal`, `path`), the last event that emptied / blocked it
  startedTotal: number;                                                                       // Cut 11 §5: the lineage tick this run started at (`Snapshot.run.started_turn`)
  twist?: string;                                                                             // Cut 12 §4: the floor's one situation from D3 (`nest`), never the previous floor's kind
};
/** Cut 12 §4: the situation kinds a floor rolls one of from D3 (the core's bands; the fake draws from the whole list). */
const TWISTS = ["den", "lock", "captive", "nest", "shrine", "vault", "stray", "hunger"];
/** Cut 11 §1: a provenance entry — the wire's `because` (turn units here, ×10 on the wire) and what kind of event it was. */
type Prov = Because & { kind: "theft" | "use" | "gas" };
type RunLog = { seed: number; rules: RuleSet; depth: number; cause?: string; turns: number; exit: string; known: string[]; cls: string; trait: string; heir: number; hpMargin: number; line?: ExitLine };

const idx = (x: number, y: number): number => y * W + x;
const inb = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < W && y < H;
const passable = (t: Tile): boolean => t !== "wall" && t !== "chasm";
const cheb = (ax: number, ay: number, bx: number, by: number): number => Math.max(Math.abs(ax - bx), Math.abs(ay - by));
const DIRS: [number, number][] = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]];

function los(tiles: Tile[], x0: number, y0: number, x1: number, y1: number): boolean {
  let dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1, err = dx + dy;
  let x = x0, y = y0;
  for (;;) {
    if (x === x1 && y === y1) return true;
    if (!(x === x0 && y === y0) && tiles[idx(x, y)] === "wall") return false;
    const e2 = 2 * err;
    if (e2 >= dy) { err += dy; x += sx; }
    if (e2 <= dx) { err += dx; y += sy; }
  }
}

function genFloor(rng: Rng, depth: number, flav: (kind: string) => string, known: Set<string>, nextId: () => number): Floor {
  const tiles: Tile[] = new Array(W * H).fill("wall");
  const roomOf = new Int8Array(W * H).fill(-1);
  const rooms: Rect[] = [];
  for (let tries = 0; tries < 40 && rooms.length < 7; tries++) {
    const w = ri(rng, 4, 7), h = ri(rng, 3, 6), x = ri(rng, 1, W - w - 2), y = ri(rng, 1, H - h - 2);
    if (rooms.some((r) => x < r.x + r.w + 2 && x + w + 2 > r.x && y < r.y + r.h + 2 && y + h + 2 > r.y)) continue;
    rooms.push({ x, y, w, h });
  }
  rooms.forEach((r, i) => { for (let y = r.y; y < r.y + r.h; y++) for (let x = r.x; x < r.x + r.w; x++) { tiles[idx(x, y)] = "floor"; roomOf[idx(x, y)] = i; } });
  const carve = (x: number, y: number): void => { if (tiles[idx(x, y)] === "wall") tiles[idx(x, y)] = "floor"; };
  for (let i = 1; i < rooms.length; i++) {
    const a = rooms[i - 1], b = rooms[i];
    let x = a.x + (a.w >> 1), y = a.y + (a.h >> 1);
    const tx = b.x + (b.w >> 1), ty = b.y + (b.h >> 1);
    const horizFirst = rng() < 0.5;
    const stepX = (): void => { while (x !== tx) { x += Math.sign(tx - x); carve(x, y); } };
    const stepY = (): void => { while (y !== ty) { y += Math.sign(ty - y); carve(x, y); } };
    if (horizFirst) { stepX(); stepY(); } else { stepY(); stepX(); }
  }
  // doors: corridor tiles touching a room
  for (let y = 1; y < H - 1; y++) for (let x = 1; x < W - 1; x++) {
    if (tiles[idx(x, y)] !== "floor" || roomOf[idx(x, y)] >= 0) continue;
    const n = [[1, 0], [-1, 0], [0, 1], [0, -1]].filter(([dx, dy]) => roomOf[idx(x + dx, y + dy)] >= 0).length;
    if (n === 1 && rng() < 0.7) tiles[idx(x, y)] = "door";
  }
  const biome = biomeOf(depth);
  if (biome === "fens" || biome === "crypt") {
    for (const r of rooms.slice(1, 3)) {
      const px = r.x + ri(rng, 0, Math.max(0, r.w - 3)), py = r.y + ri(rng, 0, Math.max(0, r.h - 2));
      for (let y = py; y < Math.min(r.y + r.h, py + 2); y++) for (let x = px; x < Math.min(r.x + r.w, px + 3); x++) tiles[idx(x, y)] = biome === "fens" ? "water" : "chasm";
    }
  }
  const first = rooms[0], last = rooms[rooms.length - 1];
  tiles[idx(first.x + (first.w >> 1), first.y + (first.h >> 1))] = "stairs_up";
  tiles[idx(last.x + (last.w >> 1), last.y + (last.h >> 1))] = "stairs_down";

  const spot = (r: Rect): [number, number] => {
    for (let k = 0; k < 20; k++) { const x = ri(rng, r.x, r.x + r.w - 1), y = ri(rng, r.y, r.y + r.h - 1); if (tiles[idx(x, y)] === "floor") return [x, y]; }
    return [r.x, r.y];
  };
  // Cut 5 §4 situations as props the renderer draws (the fake's hero walks over them): a shrine, a vault, a nest on D1–5
  if (depth <= 5) for (const t of ["shrine", "vault", "nest"] as const) { if (rng() < 0.7 && rooms.length > 2) { const r = pick(rng, rooms.slice(1, -1)); const [x, y] = spot(r); tiles[idx(x, y)] = t; } }
  const items: FloorItem[] = [];
  const nItems = ri(rng, 2, 4);
  for (let i = 0; i < nItems; i++) {
    const r = pick(rng, rooms.slice(1)); const [x, y] = spot(r);
    const roll = rng();
    let kind: string, label: string, knownIt = true;
    if (depth >= 2 && roll < 0.12) { kind = "leash"; label = "leash"; }
    else if (roll < 0.35) { kind = "gold"; label = `gold ${ri(rng, 5, 10 + depth * 4)}`; }
    else if (roll < 0.7) { kind = pick(rng, POTIONS); const f = flav(kind); knownIt = known.has(`item:${f}=${kind}`); label = knownIt ? `${kind} potion` : `${f} potion?`; }
    else if (roll < 0.9) { kind = pick(rng, SCROLLS); const f = flav(kind); knownIt = known.has(`item:${f}=${kind}`); label = knownIt ? `scroll ${kind}` : `scroll? ${f}`; }
    else { const g = depth < 4 ? ["dagger", "leather", "sword"] : depth < 9 ? ["sword", "mail", "axe", "bow"] : ["axe", "plate", "mail"]; kind = pick(rng, g); label = kind; }
    items.push({ id: nextId(), x, y, kind, known: knownIt, label });
  }
  const mons: Mon[] = [];
  const kinds = Object.keys(MON).filter((k) => !MON[k].tags.includes("boss") && k !== "blade" && depth >= MON[k].lo && depth <= MON[k].hi);
  const n = Math.min(8, 2 + Math.ceil(depth / 2));
  const mk = (kind: string, x: number, y: number): Mon => {
    const d = MON[kind]; const boost = 1 + (depth - d.lo) * 0.08;
    return { id: nextId(), kind, x, y, hp: Math.round(d.hp * boost), max_hp: Math.round(d.hp * boost), tags: d.tags, def: d.def, atk: [Math.round(d.atk[0] * boost), Math.round(d.atk[1] * boost)], wind: 0, drawn: false, stun: 0, summoned: false };
  };
  for (let i = 0; i < n; i++) {
    const kind = pick(rng, kinds); const r = pick(rng, rooms.slice(1)); const [x, y] = spot(r);
    mons.push(mk(kind, x, y));
    if (MON[kind].tags.includes("pack")) { const [px, py] = spot(r); mons.push(mk(kind, px, py)); if (rng() < 0.4) { const [qx, qy] = spot(r); mons.push(mk(kind, qx, qy)); } }
  }
  if (BOSS[depth]) { const [x, y] = spot(last); mons.push(mk(BOSS[depth], x, y)); }
  const seen = new Array(W * H).fill(false), visible = new Array(W * H).fill(false);
  return { tiles, seen, visible, overlays: [], rooms, items, mons, roomOf };
}

// --- sim ---
type SimCtx = { rules: RuleSet; unlocks: Set<string>; flav: (k: string) => string; kindOfFlav: (f: string) => string | undefined; tier: (kind: string) => number;
  bones: BonesPile[]; insured: Set<number>; gold: number; spent: { label: string; price: number }[] };

function makeRun(id: number, heir: number, seed: number, cls: string, trait: string, known: Set<string>, brought: InvItem[], ctx: SimCtx, party: Companion[] = [], level = 1): Run {
  const rng = mulberry32(seed);
  const atkBonus = [3, 6, 9].filter((l) => level >= l).length;
  const base = ({ rogue: 16, ranger: 16, caster: 14 } as Record<string, number>)[cls] ?? 20;
  const hp = base + 2 * (level - 1);
  const hero: Hero = {
    x: 0, y: 0, hp, max_hp: hp, atk: cls === "fighter" ? [2 + atkBonus, 5 + atkBonus] : [2 + atkBonus, 4 + atkBonus], def: 0,
    inv: [], invis: 0, stun: 0, bashCd: 0, vanishCd: 0, speedT: 0,
  };
  for (const it of brought) applyItem(hero, it, ctx.tier(it.kind));
  if (!hero.weapon) { const w = cls === "ranger" ? "bow" : "dagger"; hero.weapon = w; hero.atk = WEAPONS[w]; }
  hero.atk = [hero.atk[0] + atkBonus, hero.atk[1] + atkBonus];
  const run: Run = {
    id, heir, seed, rng, depth: 0, turn: 0, started_turn: 0, floorTurn: 0, floor: undefined as unknown as Floor, hero, loot: 0, alert: 0,
    trace: [], over: false, kills: {}, hl: [], facts: [], known: new Set(known), hurt: false, killed: false, saw: false,
    nextId: 0, picked: [], nearDeath: false, kindsSeen: new Set(), brought: brought.map((b) => b.id), cls, trait, loot_kept: 0,
    party: party.map((c) => ({ ...c, tags: [...c.tags], rules: { rows: c.rules.rows.map((r) => ({ conds: r.conds.map((x) => ({ ...x })), verb: { ...r.verb } })) } })), recalled: [], tamed: [], lostC: [],
    level, killXp: 0, windUsed: false, bulwark: 0, cleaveCd: 0, gear: [], score: 0, bonesFound: [], bonesPiles: ctx.bones, rest_s: 0, insured: ctx.insured, broughtItems: brought.map((b) => ({ ...b })),
    gold: ctx.gold, spent: ctx.spent, prov: {}, startedTotal: 0,
  };
  run.nextId = 1000;
  run.recalled = run.party.map((c) => companionMon(run, c));
  descendTo(run, DEV_START_DEPTH, ctx, []);
  return run;
}
function companionMon(run: Run, c: Companion): Mon {
  const d = MON[c.kind] ?? MON.rat;
  return { id: run.nextId++, cid: c.id, kind: c.kind, name: c.name, x: 0, y: 0, hp: c.hp, max_hp: c.max_hp, tags: [...c.tags], ally: true, def: d.def, atk: d.atk, wind: 0, drawn: false, stun: 0, summoned: false, rules: c.rules };
}
function placeAllies(run: Run): void {
  for (const m of run.recalled) {
    for (const [dx, dy] of DIRS) { const x = run.hero.x + dx, y = run.hero.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)]) && !occupied(run, x, y)) { m.x = x; m.y = y; break; } }
    if (m.x === 0 && m.y === 0) { m.x = run.hero.x; m.y = run.hero.y; }
    run.floor.mons.push(m);
  }
  run.recalled = [];
}
function applyItem(h: Hero, it: InvItem, tier = 0): void {
  if (WEAPONS[it.kind]) { h.weapon = it.kind; h.atk = [WEAPONS[it.kind][0] + tier, WEAPONS[it.kind][1] + tier]; }
  else if (ARMOUR[it.kind] !== undefined) { h.armour = it.kind; h.def = ARMOUR[it.kind] + tier; }
  else h.inv.push(it);
}
function descendTo(run: Run, depth: number, ctx: SimCtx, ev: Ev[]): void {
  run.depth = depth; run.floorTurn = 0; run.alert = 0; run.goal = undefined; run.windUsed = false;
  // Cut 12 §4: from D3 every floor rolls one situation, never the previous floor's kind
  run.twist = depth >= 3 ? pick(run.rng, TWISTS.filter((t) => t !== run.twist)) : undefined;
  if (run.floor) for (const m of run.floor.mons) if (m.ally && m.cid !== undefined) run.recalled.push(m);
  run.floor = genFloor(run.rng, depth, ctx.flav, run.known, () => run.nextId++);
  const up = run.floor.tiles.indexOf("stairs_up");
  run.hero.x = up % W; run.hero.y = Math.floor(up / W);
  placeAllies(run);
  // Cut 2 §2: a dead heir's kit lies where it fell; a later heir that steps on it recovers it
  for (const b of ctx.bones) if (b.depth === depth && !run.bonesFound.includes(b)) {
    const room = run.floor.rooms[run.floor.rooms.length - 1]; const x = room.x + Math.floor(room.w / 2), y = room.y + Math.floor(room.h / 2);
    run.floor.items.push({ id: run.nextId++, x, y, kind: "bones", known: true, label: `bones ♟${b.heir}` });
  }
  ev.push({ t: run.turn, k: "descend", depth, biome: biomeOf(depth) });
  ev.push({ t: run.turn, k: "callout", text: `D${depth} ${biomeOf(depth)}` });
  const bf = `biome:${biomeOf(depth)}`;
  if (!run.known.has(bf)) { run.known.add(bf); run.facts.push(bf); ev.push({ t: run.turn, k: "fact", fact: bf }); }
  updateVis(run);
}
function updateVis(run: Run): void {
  const f = run.floor, h = run.hero;
  f.visible.fill(false);
  for (let y = Math.max(0, h.y - VIS); y <= Math.min(H - 1, h.y + VIS); y++) for (let x = Math.max(0, h.x - VIS); x <= Math.min(W - 1, h.x + VIS); x++) {
    if (los(f.tiles, h.x, h.y, x, y)) { f.visible[idx(x, y)] = true; f.seen[idx(x, y)] = true; }
  }
}
function visibleFoes(run: Run): Mon[] { return run.floor.mons.filter((m) => run.floor.visible[idx(m.x, m.y)] && !m.ally); }
function adjFoes(run: Run): Mon[] { return run.floor.mons.filter((m) => cheb(m.x, m.y, run.hero.x, run.hero.y) <= 1); }
function occupied(run: Run, x: number, y: number): boolean { return run.floor.mons.some((m) => m.x === x && m.y === y) || (run.hero.x === x && run.hero.y === y); }
function inCorridor(run: Run): boolean { return run.floor.roomOf[idx(run.hero.x, run.hero.y)] < 0; }

function bfsStep(run: Run, goal: (x: number, y: number) => boolean, avoidMons = true): [number, number] | null {
  const f = run.floor; const prev = new Int16Array(W * H).fill(-1); const q: number[] = [idx(run.hero.x, run.hero.y)]; prev[q[0]] = q[0];
  let found = -1;
  for (let qi = 0; qi < q.length && found < 0; qi++) {
    const c = q[qi], cx = c % W, cy = Math.floor(c / W);
    for (const [dx, dy] of DIRS) {
      const nx = cx + dx, ny = cy + dy; if (!inb(nx, ny)) continue; const ni = idx(nx, ny);
      if (prev[ni] >= 0 || !passable(f.tiles[ni])) continue;
      if (avoidMons && f.mons.some((m) => m.x === nx && m.y === ny)) continue;
      prev[ni] = c; if (goal(nx, ny)) { found = ni; break; } q.push(ni);
    }
  }
  if (found < 0) return null;
  let cur = found; while (prev[cur] !== q[0]) cur = prev[cur];
  return [cur % W, Math.floor(cur / W)];
}
function moveHero(run: Run, x: number, y: number, ev: Ev[]): void { run.hero.x = x; run.hero.y = y; ev.push({ t: run.turn, k: "move", id: 0, x, y }); updateVis(run); bossInView(run, ev); }
/** Cut 6 §5: seeing the boss is the counter fact (the core learns it on the boss's first telegraph, the same step). */
function bossInView(run: Run, ev: Ev[]): void { for (const m of visibleFoes(run)) if (m.tags.includes("boss")) learn(run, `boss:${m.kind}:counter`, ev); }
function stepAway(run: Run, from: Mon[], ev: Ev[], preferCorridor: boolean): boolean {
  const h = run.hero; let best: [number, number] | null = null, bestScore = -Infinity;
  for (const [dx, dy] of DIRS) {
    const nx = h.x + dx, ny = h.y + dy; if (!inb(nx, ny) || !passable(run.floor.tiles[idx(nx, ny)]) || occupied(run, nx, ny)) continue;
    let s = Math.min(...from.map((m) => cheb(m.x, m.y, nx, ny)));
    if (preferCorridor && run.floor.roomOf[idx(nx, ny)] < 0) s += 1.5;
    if (s > bestScore) { bestScore = s; best = [nx, ny]; }
  }
  if (!best) return false; moveHero(run, best[0], best[1], ev); return true;
}
function hitRoll(rng: Rng, atk: [number, number], def: number): { hit: boolean; dmg: number } {
  const hit = rng() < 0.8; return { hit, dmg: hit ? Math.max(0, ri(rng, atk[0], atk[1]) - def) : 0 };
}
function nameOf(m: Mon): string { return m.name ?? m.kind.replace(/_/g, " "); }
function hurtHero(run: Run, dmg: number, cause: string, ev: Ev[]): void {
  const before = run.hero.hp; run.hero.hp = Math.max(0, run.hero.hp - dmg); run.hurt = true;
  run.lastHurt = { cause, dmg, hpBefore: before };
  ev.push({ t: run.turn, k: "hurt", id: 0, dmg, hp: run.hero.hp, cause });
  if (run.hero.hp > 0 && run.hero.hp <= run.hero.max_hp * 0.1 && !run.nearDeath) { run.nearDeath = true; ev.push({ t: run.turn, k: "callout", text: `hp ${run.hero.hp}` }); }
}
function killMon(run: Run, m: Mon, cause: string, ev: Ev[]): void {
  run.floor.mons = run.floor.mons.filter((x) => x !== m);
  ev.push({ t: run.turn, k: "die", id: m.id, cause });
  run.killed = true; run.kills[m.kind] = (run.kills[m.kind] ?? 0) + 1; run.killXp += 1 + run.depth / 5; run.score += m.tags.includes("boss") ? 25 : Math.ceil(m.max_hp / 4);
  if (m.tags.includes("gas")) { for (const [dx, dy] of [[0, 0], ...DIRS]) { const x = m.x + dx, y = m.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)])) { run.floor.overlays.push({ x, y, k: "gas", ttl: 4 }); ev.push({ t: run.turn, k: "overlay", x, y, ov: "gas", ttl: 4 }); } } ev.push({ t: run.turn, k: "callout", text: "bloat pops" }); learn(run, `foe:${m.kind}:gas`, ev); gasNear(run, m); }
  if (m.tags.includes("splitter") && m.max_hp >= 6) { for (let i = 0; i < 2; i++) { const [dx, dy] = DIRS[i]; const x = m.x + dx, y = m.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)]) && !occupied(run, x, y)) { const c: Mon = { ...m, id: run.nextId++, x, y, hp: Math.ceil(m.max_hp / 2), max_hp: Math.ceil(m.max_hp / 2) }; run.floor.mons.push(c); ev.push({ t: run.turn, k: "spawn", e: ent(c) }); } } learn(run, `foe:${m.kind}:splitter`, ev); }
  if (m.tags.includes("boss")) { run.hl.push({ pattern: "boss", score: 6, t: run.turn, run_id: run.id, text: `${nameOf(m)} slain on D${run.depth}` }); ev.push({ t: run.turn, k: "note", text: `${nameOf(m)} slain` }); }
}
function learn(run: Run, fact: string, ev: Ev[]): void {
  if (run.known.has(fact)) return; run.known.add(fact); run.facts.push(fact);
  ev.push({ t: run.turn, k: "fact", fact });
  const parts = fact.split(":"); ev.push({ t: run.turn, k: "callout", text: parts.length === 3 ? `${parts[1].replace(/_/g, " ")}: ${parts[2]}` : fact.replace(/^item:/, "").replace(/_/g, " ") });
}
function ent(m: Mon): Entity { return { id: m.id, kind: m.kind, name: m.name, x: m.x, y: m.y, hp: m.hp, max_hp: m.max_hp, tags: m.tags, ally: m.ally, telegraph: m.drawn ? "draws" : m.wind ? "winds up" : undefined, cid: m.cid }; }
function companions(run: Run): Mon[] { return run.floor.mons.filter((m) => m.ally && m.cid !== undefined); }
function loseCompanion(run: Run, m: Mon, cause: string, ev: Ev[]): void {
  run.floor.mons = run.floor.mons.filter((x) => x !== m);
  ev.push({ t: run.turn, k: "die", id: m.id, cause });
  ev.push({ t: run.turn, k: "ally", id: m.id, state: "lost" });
  ev.push({ t: run.turn, k: "note", text: `${nameOf(m)} lost to ${cause}` });
  ev.push({ t: run.turn, k: "callout", text: `${nameOf(m)} fell` });   // Cut 10 §3: the core's line (`Ashar fell`); the client prefixes the kind off the ally flag
  const c = run.party.find((p) => p.id === m.cid) ?? { id: m.cid ?? 0, kind: m.kind, name: nameOf(m), level: 1, tags: [...m.tags], gen: 0, rules: { rows: [] }, max_rows: 2, hp: m.max_hp, max_hp: m.max_hp };
  run.lostC.push(c);
  run.hl.push({ pattern: "ally_lost", score: 4, t: run.turn, run_id: run.id, text: `${nameOf(m)} fell on D${run.depth}.` });
}
function hurtMon(run: Run, m: Mon, dmg: number, cause: string, ev: Ev[]): void {
  m.hp -= dmg;
  if (m.ally && m.cid !== undefined) ev.push({ t: run.turn, k: "hurt", id: m.id, dmg, hp: Math.max(0, m.hp), cause });
  if (m.hp <= 0) { if (m.ally && m.cid !== undefined) loseCompanion(run, m, cause, ev); else killMon(run, m, cause, ev); }
}

function useItem(run: Run, it: InvItem, ctx: SimCtx, ev: Ev[], thrownAt?: Mon): void {
  run.hero.inv = run.hero.inv.filter((x) => x !== it);
  const kind = it.kind; const f = ctx.flav(kind); const wasKnown = it.known;
  let outcome = kind;
  if (thrownAt) {
    if (kind === "poison") { thrownAt.hp -= 8; outcome = "poison 8"; } else if (kind === "fire" || kind === "caustic") { const ov = kind === "fire" ? "fire" : "gas"; for (const [dx, dy] of [[0, 0], ...DIRS]) { const x = thrownAt.x + dx, y = thrownAt.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)])) { run.floor.overlays.push({ x, y, k: ov, ttl: 3 }); ev.push({ t: run.turn, k: "overlay", x, y, ov, ttl: 3 }); } } } else if (kind === "confusion") { thrownAt.stun = 3; } else { thrownAt.hp -= 2; }
    if (thrownAt.hp <= 0) killMon(run, thrownAt, "thrown", ev);
  } else if (POTIONS.includes(kind)) {
    if (kind === "heal") { run.hero.hp = Math.min(run.hero.max_hp, run.hero.hp + Math.ceil(run.hero.max_hp / 2)); outcome = `hp ${run.hero.hp}`; }
    else if (kind === "strength") { run.hero.atk = [run.hero.atk[0] + 1, run.hero.atk[1] + 1]; }
    else if (kind === "speed") { run.hero.speedT = 3; }
    else if (kind === "invisibility") { run.hero.invis = 5; }
    else if (kind === "poison") { hurtHero(run, 6, "poison", ev); outcome = "poison"; }
    else if (kind === "caustic" || kind === "fire") { hurtHero(run, 4, kind, ev); }
    else if (kind === "confusion") { run.hero.stun = 2; }
  } else {
    if (kind === "teleport") { const opts: number[] = []; run.floor.tiles.forEach((t, i) => { if (t === "floor" && !occupied(run, i % W, Math.floor(i / W))) opts.push(i); }); const p = pick(run.rng, opts); moveHero(run, p % W, Math.floor(p / W), ev); }
    else if (kind === "blink") { stepAway(run, visibleFoes(run), ev, false); stepAway(run, visibleFoes(run), ev, false); }
    else if (kind === "fear" || kind === "darkness") { for (const m of visibleFoes(run)) m.stun = 4; }
    else if (kind === "mapping") { run.floor.seen.fill(true); }
    else if (kind === "identify") { const u = run.hero.inv.find((x) => !x.known); if (u) { u.known = true; u.label = POTIONS.includes(u.kind) ? `${u.kind} potion` : `scroll ${u.kind}`; learn(run, `item:${ctx.flav(u.kind)}=${u.kind}`, ev); } }
    else if (kind === "enchant") { run.hero.atk = [run.hero.atk[0] + 1, run.hero.atk[1] + 1]; }
    else if (kind === "summon_ally") { const [dx, dy] = DIRS[0]; const h: Mon = { id: run.nextId++, kind: "spectral_hound", x: run.hero.x + dx, y: run.hero.y + dy, hp: 10, max_hp: 10, tags: [], def: 1, atk: [2, 5], wind: 0, drawn: false, stun: 0, summoned: true, ally: true }; run.floor.mons.push(h); ev.push({ t: run.turn, k: "spawn", e: ent(h) }); ev.push({ t: run.turn, k: "ally", id: h.id, state: "freed" }); }
    else if (kind === "aggravate") { run.alert = 5; for (const m of run.floor.mons) { m.x = Math.max(0, Math.min(W - 1, run.hero.x + ri(run.rng, -4, 4))); m.y = Math.max(0, Math.min(H - 1, run.hero.y + ri(run.rng, -4, 4))); if (!passable(run.floor.tiles[idx(m.x, m.y)])) { m.x = run.hero.x; m.y = run.hero.y + 2; } } outcome = "aggravate"; }
  }
  ev.push({ t: run.turn, k: "use", item: it.label, outcome });
  run.prov[`item:${kind}`] = { kind: "use", text: `${thrownAt ? "threw" : POTIONS.includes(kind) ? "drank" : "read"} the ${it.label}, D${run.depth}`, t: run.turn, depth: run.depth };   // Cut 11 §1
  if (!wasKnown) {
    learn(run, `item:${f}=${kind}`, ev);
    for (const o of run.hero.inv) if (o.kind === kind) { o.known = true; o.label = POTIONS.includes(kind) ? `${kind} potion` : `scroll ${kind}`; }
    const bad = ["poison", "caustic", "fire", "confusion", "aggravate"].includes(kind);
    run.hl.push({ pattern: "gamble", score: bad && run.hero.hp > 0 ? 5 : 3, t: run.turn, run_id: run.id, text: bad ? `Drank the unknown. ${kind}. Survived.` : `Tried the unknown: ${kind}.` });
    ev.push({ t: run.turn, k: "note", text: `${it.label} was ${kind}` });
  }
}

/** Cut 11 §1: a gas cloud that lands within 2 tiles of the hero is the `no path` provenance until it clears. */
function gasNear(run: Run, m: Mon): void {
  if (cheb(m.x, m.y, run.hero.x, run.hero.y) <= 2) run.prov.path = { kind: "gas", text: `gas cloud, D${run.depth}`, t: run.turn, depth: run.depth };
}
const provOut = (p: Prov | undefined): Because | undefined => (p ? { text: p.text, t: p.t, depth: p.depth } : undefined);
/** Cut 11 §1: the `because` behind a state reason — the slot's last provenance entry (`none held` → the theft or the drink
 *  that emptied it; `no path` → the gas cloud), the newest item entry for an `unknown` slot. */
function becauseOf(run: Run, c: Cond | null, v: Verb | null): Because | undefined {
  const newestItem = (): Prov | undefined => Object.entries(run.prov).filter(([k]) => k.startsWith("item:")).map(([, p]) => p).sort((a, b) => b.t - a.t)[0];
  if (c) {
    if (c.k === "item" && c.t) return provOut(run.prov[`item:${c.t}`]);
    if (c.k === "unknown_item") return provOut(newestItem());
    return undefined;
  }
  if (!v) return undefined;
  if (v.v === "drink" || v.v === "read" || v.v === "throw") { const a = (v.a ?? "unknown").split(",")[0]; return provOut(a === "unknown" ? newestItem() : run.prov[`item:${a}`]); }
  if (v.v === "retreat" || v.v === "back_corridor") { const g = run.prov.path; return g && run.floor.overlays.some((o) => o.k === "gas") ? provOut(g) : undefined; }
  return undefined;
}
/** Cut 11: the wire's trace ticks are ×10 like the events (the fake sims one hero action per 10 ticks); `because` ticks too. */
function scaleTrace(turns: TraceTurn[]): TraceTurn[] {
  return turns.map((t) => ({ ...t, t: t.t * 10, telegraphs: [...t.telegraphs], rows: t.rows?.map((r) => ({ ...r, because: r.because ? { ...r.because, t: r.because.t * 10 } : undefined })) }));
}
/** Cut 6 §3: why a condition did not hold, as the player reads it (`hp 45% ≥ 30%`, `foes 0 < 1`, `no ranged`, `none held`). */
function condWhy(run: Run, c: Cond): string {
  const h = run.hero, pct = Math.round((h.hp / h.max_hp) * 100), foes = visibleFoes(run);
  switch (c.k) {
    case "hp<": return `hp ${pct}% ≥ ${c.n}%`;
    case "hp>": return `hp ${pct}% ≤ ${c.n}%`;
    case "foes>=": return `foes ${foes.length} < ${c.n ?? 1}`;
    case "adj>=": return `adjacent ${adjFoes(run).length} < ${c.n ?? 1}`;
    case "foe_tag": return `no ${(c.t ?? "").replace(/_/g, " ")}`;
    case "foe_hp<": return foes.length ? `foe hp ≥ ${c.n}%` : "no foe";
    case "item": return "none held";
    case "unknown_item": return "none held";
    case "depth>=": return `D${run.depth} < ${c.n}`;
    case "alert>=": return `alert ${run.alert} < ${c.n}`;
    case "loot>=": return `$${run.loot} < ${c.n}`;
    case "path_stairs": return "stairs unseen";
    case "in_corridor": return "not in corridor";
    case "ally": return "no ally";
    default: return `not ${c.k.replace(/_/g, " ").replace(/[<>]=?$/, "")}`;
  }
}
/** Cut 6 §3: why a verb whose conditions held could not run. */
function verbWhy(v: Verb): string {
  switch (v.v) {
    case "drink": case "read": case "throw": return "none held";
    case "retreat": case "back_corridor": return "no path";
    case "attack": case "shoot": case "tame": return "not in view";
    case "descend": return "no stairs";
    case "pick_up": return "nothing here";
    default: return "blocked";
  }
}
function condHolds(run: Run, c: Cond): boolean {
  const h = run.hero, pct = (h.hp / h.max_hp) * 100, foes = visibleFoes(run);
  switch (c.k) {
    case "hp<": return pct < (c.n ?? 0);
    case "hp>": return pct > (c.n ?? 0);
    case "foes>=": return foes.length >= (c.n ?? 1);
    case "adj>=": return adjFoes(run).length >= (c.n ?? 1);
    case "foe_tag": return foes.some((m) => m.tags.includes(c.t ?? ""));
    case "foe_hp<": return foes.some((m) => (m.hp / m.max_hp) * 100 < (c.n ?? 0));
    case "item": return h.inv.some((i) => i.known && i.kind === c.t);
    case "unknown_item": return h.inv.some((i) => !i.known);
    case "floor_seen>=": { let seen = 0, tot = 0; run.floor.tiles.forEach((t, i) => { if (passable(t)) { tot++; if (run.floor.seen[i]) seen++; } }); return (seen / Math.max(1, tot)) * 100 >= (c.n ?? 0); }
    case "depth>=": return run.depth >= (c.n ?? 0);
    case "alert>=": return run.alert >= (c.n ?? 0);
    case "in_corridor": return inCorridor(run);
    case "path_stairs": { const s = run.floor.tiles.indexOf("stairs_down"); return s >= 0 && run.floor.seen[s]; }
    case "ally": return run.floor.mons.some((m) => m.ally);
    case "loot>=": return run.loot >= (c.n ?? 0);
    case "turns>": return run.turn - run.started_turn > (c.n ?? 0);
    case "on_hurt": return run.hurt;
    case "on_kill": return run.killed;
    case "on_see": return run.saw;
    case "party_hp<": return companions(run).some((m) => (m.hp / m.max_hp) * 100 < (c.n ?? 0));
    case "party": return companions(run).some((m) => m.kind === c.t);
    default: return false;
  }
}
function target(run: Run, a: string | undefined): Mon | undefined {
  const foes = visibleFoes(run); if (!foes.length) return undefined;
  const h = run.hero; const byDist = [...foes].sort((p, q) => cheb(p.x, p.y, h.x, h.y) - cheb(q.x, q.y, h.x, h.y));
  if (a === "lowest") return [...foes].sort((p, q) => p.hp - q.hp)[0];
  if (a?.startsWith("tag:")) { const t = a.slice(4); return byDist.find((m) => m.tags.includes(t)) ?? undefined; }
  return byDist[0];
}
function heroAttack(run: Run, m: Mon, ev: Ev[], verb: string, mult = 1): void {
  const h = run.hero;
  if (cheb(m.x, m.y, h.x, h.y) > 1 && h.weapon !== "bow") { const s = bfsStep(run, (x, y) => cheb(x, y, m.x, m.y) <= 1); if (s) moveHero(run, s[0], s[1], ev); return; }
  const r = hitRoll(run.rng, h.atk, m.def); const dmg = r.dmg * mult;
  ev.push({ t: run.turn, k: "attack", src: 0, dst: m.id, dmg, hit: r.hit, verb });
  if (r.hit) { m.hp -= dmg; if (m.hp <= 0) killMon(run, m, verb, ev); }
}
// returns true if the verb executed
function exec(run: Run, v: Verb, ctx: SimCtx, ev: Ev[]): boolean {
  const h = run.hero; const foes = visibleFoes(run);
  const has = (verb: string): boolean => verbsUpTo(run.cls, run.level).includes(verb) || (verb === "throw" && ctx.unlocks.has("throw"));
  switch (v.v) {
    case "attack": { const m = target(run, v.a); if (!m) return false; heroAttack(run, m, ev, "attack"); return true; }
    case "retreat": if (!foes.length) return false; return stepAway(run, foes, ev, false);
    case "back_corridor": if (!foes.length) return false; if (inCorridor(run)) { const m = target(run, "nearest"); if (m && cheb(m.x, m.y, h.x, h.y) <= 1) { heroAttack(run, m, ev, "attack"); return true; } } return stepAway(run, foes, ev, true);
    case "drink": { if (h.hp >= h.max_hp && (v.a === "heal" || v.a === "unknown")) return false; const it = v.a === "unknown" ? h.inv.find((i) => !i.known && POTIONS.includes(i.kind)) : h.inv.find((i) => i.known && i.kind === (v.a ?? "heal")); if (!it) return false; useItem(run, it, ctx, ev); return true; }
    case "read": { const it = v.a === "unknown" ? h.inv.find((i) => !i.known && SCROLLS.includes(i.kind)) : h.inv.find((i) => i.known && i.kind === v.a); if (!it) return false; useItem(run, it, ctx, ev); return true; }
    case "throw": { if (!has("throw")) return false; const [k, tg] = (v.a ?? "").split(","); const it = k === "unknown" ? h.inv.find((i) => !i.known && POTIONS.includes(i.kind)) : h.inv.find((i) => i.known && i.kind === k && THROWABLE.has(i.kind)); const m = target(run, tg); if (!it || !m || cheb(m.x, m.y, h.x, h.y) > 5) return false; useItem(run, it, ctx, ev, m); return true; }
    case "descend": { const s = run.floor.tiles.indexOf("stairs_down"); if (s < 0 || !run.floor.seen[s]) return false; if (idx(h.x, h.y) === s) { descendTo(run, run.depth + 1, ctx, ev); return true; } const st = bfsStep(run, (x, y) => idx(x, y) === s); if (!st) return false; moveHero(run, st[0], st[1], ev); return true; }
    case "bank": { const s = run.floor.tiles.indexOf("stairs_up"); if (idx(h.x, h.y) === s) { endRun(run, "bank", ev); return true; } const st = bfsStep(run, (x, y) => idx(x, y) === s); if (!st) return false; moveHero(run, st[0], st[1], ev); return true; }
    case "return": endRun(run, "return", ev); return true;
    case "rest": if (foes.length || h.hp >= h.max_hp) return false; h.hp = Math.min(h.max_hp, h.hp + 1); return true;
    case "pick_up": { const it = run.floor.items.find((i) => i.x === h.x && i.y === h.y); if (!it) return false; pickUp(run, it, ev); return true; }
    case "free_captive": return false;
    case "tame": {
      if (!ctx.unlocks.has("tame")) return false;
      const leash = h.inv.find((i) => i.kind === "leash"); if (!leash) return false;
      const m = target(run, v.a); if (!m || m.hp / m.max_hp >= 0.25 || cheb(m.x, m.y, h.x, h.y) > 1 || m.tags.includes("boss") || m.summoned) return false;
      h.inv = h.inv.filter((i) => i !== leash);
      const known = m.tags.filter((t) => run.known.has(`foe:${m.kind}:${t}`)).length;
      const ok = run.rng() < Math.min(0.6, 0.2 + 0.1 * known);
      ev.push({ t: run.turn, k: "tame", id: m.id, kind: m.kind, ok });
      if (ok) { m.ally = true; m.cid = run.nextId++; m.rules = defaultCompanionRules(m.tags); m.drawn = false; m.wind = 0; run.tamed.push(m); learn(run, `tamed:${m.kind}`, ev); ev.push({ t: run.turn, k: "ally", id: m.id, state: "freed" }); ev.push({ t: run.turn, k: "note", text: `tamed a ${nameOf(m)}` }); ev.push({ t: run.turn, k: "callout", text: `tamed ${nameOf(m)}` }); run.hl.push({ pattern: "tamed", score: 5, t: run.turn, run_id: run.id, text: `Leashed a ${nameOf(m)} on D${run.depth}.` }); }
      else { ev.push({ t: run.turn, k: "callout", text: "leash slips" }); const r = hitRoll(run.rng, m.atk, h.def); ev.push({ t: run.turn, k: "attack", src: m.id, dst: 0, dmg: r.dmg, hit: r.hit }); if (r.hit && r.dmg) hurtHero(run, r.dmg, nameOf(m), ev); }
      return true;
    }
    case "recall": { const c = companions(run).find((m) => !v.a || m.kind === v.a); if (!c) return false; run.floor.mons = run.floor.mons.filter((x) => x !== c); run.recalled.push(c); ev.push({ t: run.turn, k: "callout", text: `${nameOf(c)} recalled` }); return true; }
    case "send": { const c = companions(run).find((m) => !v.a || m.kind === v.a); if (!c || !foes.length) return false; c.sent = true; return true; }
    case "shield_bash": { if (!has("shield_bash") || h.bashCd > 0) return false; const m = adjFoes(run)[0]; if (!m) return false; h.bashCd = 5; m.stun = 1; heroAttack(run, m, ev, "shield_bash"); return true; }
    case "vanish": if (!has("vanish") || h.vanishCd > 0 || !foes.length) return false; h.vanishCd = 12; h.invis = 3; return true;
    case "cleave": { if (!has("cleave") || run.cleaveCd > 0) return false; const adj = adjFoes(run); if (adj.length < 2) return false; run.cleaveCd = 6; for (const m of adj) heroAttack(run, m, ev, "cleave"); return true; }
    case "taunt": { if (!has("taunt") || foes.length < 1 || !companions(run).length) return false; for (const m of foes) m.sent = false; ev.push({ t: run.turn, k: "callout", text: "taunt" }); return true; }
    case "second_wind": if (!has("second_wind") || run.windUsed || h.hp / h.max_hp >= 0.5) return false; run.windUsed = true; h.hp = Math.min(h.max_hp, h.hp + Math.ceil(h.max_hp * 0.3)); return true;
    case "bulwark": if (!has("bulwark") || run.bulwark > 0 || !foes.length) return false; run.bulwark = 3; h.def += 3; return true;
    case "backstab": { if (!has("backstab")) return false; const m = adjFoes(run).find((x) => x.stun > 0 || h.invis > 0); if (!m) return false; heroAttack(run, m, ev, "backstab", 2); return true; }
    case "smoke": if (!has("smoke") || !foes.length) return false; for (const m of foes) m.stun = 3; ev.push({ t: run.turn, k: "callout", text: "smoke" }); return true;
    case "ambush": { if (!has("ambush") || h.invis <= 0) return false; const m = adjFoes(run)[0]; if (!m) return false; heroAttack(run, m, ev, "ambush", 3); return true; }
    case "shadowstep": if (!has("shadowstep") || !foes.length) return false; return stepAway(run, foes, ev, false);
    // Cut 2 §4 ranger / caster ladders (approximations: the core is the truth)
    case "shoot": case "bolt": case "volley": case "double_shot": { if (!has(v.v)) return false; const m = target(run, v.a ?? "nearest"); if (!m || cheb(m.x, m.y, h.x, h.y) > 5) return false; const w = h.weapon; h.weapon = "bow"; heroAttack(run, m, ev, v.v, v.v === "volley" || v.v === "double_shot" ? 2 : 1); h.weapon = w; return true; }
    case "kite": { if (!has("kite")) return false; const m = adjFoes(run)[0]; if (!m) return false; return stepAway(run, [m], ev, false); }
    case "trap": case "slow": { if (!has(v.v) || !foes.length) return false; const m = target(run, "nearest"); if (!m || cheb(m.x, m.y, h.x, h.y) > 3) return false; m.stun = v.v === "trap" ? 2 : 1; return true; }
    case "mark": { if (!has("mark")) return false; const m = target(run, "nearest"); if (!m) return false; heroAttack(run, m, ev, "mark", 1.5); return true; }
    case "ward": if (!has("ward") || run.bulwark > 0 || !foes.length) return false; run.bulwark = 3; h.def += 2; return true;
    case "blink": if (!has("blink") || !foes.length) return false; return stepAway(run, foes, ev, false);
    case "nova": { if (!has("nova") || run.cleaveCd > 0) return false; const adj = adjFoes(run); if (!adj.length) return false; run.cleaveCd = 15; for (const m of adj) heroAttack(run, m, ev, "nova"); return true; }
    case "drain": { if (!has("drain")) return false; const m = adjFoes(run)[0]; if (!m) return false; heroAttack(run, m, ev, "drain"); h.hp = Math.min(h.max_hp, h.hp + 5); return true; }
    case "tactic": case "card": {
      if (!ctx.unlocks.has(v.a ?? "")) return false;
      if (v.a === "corridor_fighting") { if (foes.length < 2) return false; return exec(run, { v: "back_corridor" }, ctx, ev); }
      if (v.a === "kite_archers") { const m = adjFoes(run)[0]; if (!m || !foes.length) return false; if (stepAway(run, [m], ev, false)) return true; return exec(run, { v: "attack" }, ctx, ev); }
      if (v.a === "stair_dance") { if (h.hp / h.max_hp >= 0.5 || !foes.length) return false; return exec(run, { v: "descend" }, ctx, ev) || exec(run, { v: "retreat" }, ctx, ev); }
      return false;
    }
    default: return false;
  }
}
function pickUp(run: Run, it: FloorItem, ev: Ev[]): void {
  run.floor.items = run.floor.items.filter((x) => x !== it);
  ev.push({ t: run.turn, k: "pickup", id: it.id, item: it.label });
  if (it.kind === "gold") { run.loot += +it.label.split(" ")[1]; return; }
  if (it.kind === "bones") {
    const heir = +it.label.slice(it.label.indexOf("♟") + 1); const pile = run.bonesPiles.find((b) => b.heir === heir && b.depth === run.depth);
    if (!pile) return;
    run.bonesFound.push(pile); run.loot += pile.items * 8;
    ev.push({ t: run.turn, k: "bones", heir, items: pile.items });
    ev.push({ t: run.turn, k: "note", text: `found the bones of ♟${heir}` }); ev.push({ t: run.turn, k: "callout", text: `bones ♟${heir}` });
    run.hl.push({ pattern: "bones", score: 6, t: run.turn, run_id: run.id, text: `Found ♟${heir}'s bones on D${run.depth}: ${pile.items} item${pile.items === 1 ? "" : "s"}.` });
    return;
  }
  if (it.kind === "leash") learn(run, "item:leash", ev);
  const inv: InvItem = { id: it.id, kind: it.kind, known: it.known, label: it.label };
  if (!it.known && run.rng() < 0.5) inv.hint = ["poison", "caustic", "fire", "confusion", "aggravate"].includes(it.kind) ? "malevolent" : "benevolent";
  run.picked.push({ ...inv });
  if (WEAPONS[it.kind] || ARMOUR[it.kind] !== undefined) run.gear.push({ ...inv });
  if (WEAPONS[it.kind]) { if (!run.hero.weapon || WEAPONS[it.kind][1] > run.hero.atk[1]) { run.hero.weapon = it.kind; run.hero.atk = WEAPONS[it.kind]; ev.push({ t: run.turn, k: "callout", text: `wields ${it.kind}` }); } run.loot += 10; return; }
  if (ARMOUR[it.kind] !== undefined) { if (ARMOUR[it.kind] > run.hero.def) { run.hero.armour = it.kind; run.hero.def = ARMOUR[it.kind]; ev.push({ t: run.turn, k: "callout", text: `wears ${it.kind}` }); } run.loot += 10; return; }
  if (run.hero.inv.length < 10) run.hero.inv.push(inv);
}
function endRun(run: Run, tier: "bank" | "return" | "death", ev: Ev[]): void {
  run.over = true; run.exit = tier;
  const keep_pct = tier === "bank" ? 100 : tier === "return" ? 60 : 0;
  run.loot_kept = Math.round(run.loot * keep_pct / 100);   // Cut 2 §2: death 0%
  // Cut 6 §1: one arithmetic line the player can check (supplies were paid in camp; a death names its bones instead)
  const spent = run.spent.reduce((n, x) => n + x.price, 0);
  const parts = [`$${run.loot} carried`, `${tier} keeps ${keep_pct}% → $${run.loot_kept}`];
  if (tier === "death") { const kit = bonesKit(run).length; if (kit) parts.push(`bones: ${kit} item${kit === 1 ? "" : "s"} on D${run.depth}`); }
  else if (spent) parts.push(`supplies −$${spent}`);
  // Cut 9 §5: every exit carries its last-5 trace (row accounting included), on the event and on the ledger line
  // Cut 11 §3: every exit trace carries the run's provenance (the `because` events) beside its turns; ticks ×10 as on the events
  const trace: Trace = { turns: scaleTrace(run.trace), provenance: Object.values(run.prov).sort((a, b) => a.t - b.t).map((p) => ({ text: p.text, t: p.t * 10, depth: p.depth })) };
  run.line = { carried: run.loot, keep_pct, kept: run.loot_kept, spent, spent_on: run.spent.map((x) => x.label), text: parts.join(" · "), trace };
  ev.push({ t: run.turn, k: "exit", tier, loot_kept: run.loot_kept, line: run.line, trace });
  // Cut 2 §1: camp rest as long as the expedition (one turn ≈ 1 s), capped; a death is a fixed wake
  run.rest_s = tier === "death" ? WAKE_S : Math.min(REST_CAP_S, run.turn);
  ev.push({ t: run.turn, k: "rest", seconds: run.rest_s });
}

function heroTurn(run: Run, ctx: SimCtx, ev: Ev[]): void {
  const h = run.hero;
  const foes = visibleFoes(run);
  const tr: TraceTurn = { t: run.turn, row: -2, verb: { v: "explore" }, hp: h.hp, foes: foes.length, telegraphs: foes.filter((m) => m.drawn || m.wind).map((m) => `${nameOf(m)} ${m.drawn ? "draws" : "winds up"}`) };
  const fire = (row: number, verb: Verb, text: string): void => { tr.row = row; tr.verb = verb; ev.push({ t: run.turn, k: "rule", row, verb, text }); };
  let done = false;
  if (h.stun > 0) { h.stun--; done = true; tr.verb = { v: "stunned" }; }
  if (!done && run.trait === "cowardly" && h.hp / h.max_hp < 0.5 && foes.length && stepAway(run, foes, ev, true)) { fire(-1, { v: "retreat" }, "cowardly: retreat"); ev.push({ t: run.turn, k: "callout", text: "cowardly: retreats" }); done = true; }
  // Cut 6 §3: every row above the fired one gets one reason (the failing cond named, or why the verb could not run)
  const rows: { row: number; why: string; because?: Because }[] = [];
  if (!done) for (let i = 0; i < ctx.rules.rows.length; i++) {
    const r = ctx.rules.rows[i];
    const miss = r.conds.find((c) => !condHolds(run, c));
    if (miss) { rows.push({ row: i, why: condWhy(run, miss), because: becauseOf(run, miss, null) }); continue; }
    if (run.trait === "brave" && (r.verb.v === "retreat" || r.verb.v === "back_corridor") && foes.length === 1) { rows.push({ row: i, why: "brave" }); continue; }
    if (exec(run, r.verb, ctx, ev)) {
      fire(i, r.verb, rowText(r));
      const hpc = r.conds.find((c) => c.k === "hp<");
      ev.push({ t: run.turn, k: "callout", text: hpc ? `hp ${Math.round((h.hp / h.max_hp) * 100)}% → ${r.verb.v.replace("_", " ")} R${i + 1}` : `${r.verb.v.replace("_", " ")} R${i + 1}` });
      done = true; break;
    }
    rows.push({ row: i, why: verbWhy(r.verb), because: becauseOf(run, null, r.verb) });   // Cut 11 §1
    if (!tr.blocked) tr.blocked = `R${i + 1} ${r.verb.v.replace(/_/g, " ")} ✗ ${verbWhy(r.verb)}`;
  }
  if (rows.length) tr.rows = rows;
  if (!done && run.trait === "curious" && !foes.length) { const u = h.inv.find((i) => !i.known); if (u) { useItem(run, u, ctx, ev); fire(-1, { v: POTIONS.includes(u.kind) ? "drink" : "read", a: "unknown" }, "curious: tries unknown"); ev.push({ t: run.turn, k: "callout", text: "curious: tries it" }); done = true; } }
  if (!done && run.trait === "greedy" && foes.length) { const it = run.floor.items.find((i) => cheb(i.x, i.y, h.x, h.y) <= 1); if (it) { if (it.x === h.x && it.y === h.y) pickUp(run, it, ev); else moveHero(run, it.x, it.y, ev); fire(-1, { v: "pick_up" }, "greedy: takes it"); ev.push({ t: run.turn, k: "callout", text: "greedy: grabs it" }); done = true; } }
  if (!done) {
    // chores: pick up here → explore nearest unseen → descend
    const here = run.floor.items.find((i) => i.x === h.x && i.y === h.y);
    if (here) { pickUp(run, here, ev); tr.verb = { v: "pick_up" }; }
    else {
      const seenItem = (x: number, y: number): boolean => run.floor.seen[idx(x, y)] && run.floor.items.some((i) => i.x === x && i.y === y);
      const s = (run.goal !== "descend" && bfsStep(run, seenItem)) || bfsStep(run, (x, y) => !run.floor.seen[idx(x, y)]);
      if (s && run.goal !== "descend") moveHero(run, s[0], s[1], ev);
      else { run.goal = "descend"; const sd = run.floor.tiles.indexOf("stairs_down"); if (idx(h.x, h.y) === sd) { descendTo(run, run.depth + 1, ctx, ev); tr.verb = { v: "descend" }; } else { const st = bfsStep(run, (x, y) => idx(x, y) === sd); if (st) moveHero(run, st[0], st[1], ev); else if (!stepAway(run, foes, ev, false)) { /* stuck */ } } }
    }
  }
  run.trace.push(tr); if (run.trace.length > TRACE_TURNS) run.trace.shift();   // Cut 11 §3: the last 10 hero turns
}
function monsterTurn(run: Run, m: Mon, ev: Ev[]): void {
  const h = run.hero;
  if (m.stun > 0) { m.stun--; return; }
  if (m.ally) { companionTurn(run, m, ev); return; }
  const d = cheb(m.x, m.y, h.x, h.y);
  const sees = d <= VIS && !h.invis && los(run.floor.tiles, m.x, m.y, h.x, h.y);
  if (d > 1) { const c = companions(run).find((a) => cheb(a.x, a.y, m.x, m.y) <= 1); if (c) { const r = hitRoll(run.rng, m.atk, c.def); ev.push({ t: run.turn, k: "attack", src: m.id, dst: c.id, dmg: r.dmg, hit: r.hit }); if (r.hit && r.dmg) hurtMon(run, c, r.dmg, nameOf(m), ev); return; } }
  if (!sees) return;
  if (m.tags.includes("water") && run.floor.tiles[idx(m.x, m.y)] !== "water") return;
  if (m.tags.includes("caster") && !m.summoned && d <= 6) {
    m.summoned = true; ev.push({ t: run.turn, k: "telegraph", id: m.id, what: "conjures" });
    for (const [dx, dy] of DIRS.slice(0, 2)) { const x = m.x + dx, y = m.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)]) && !occupied(run, x, y)) { const b: Mon = { id: run.nextId++, kind: "blade", x, y, hp: 3, max_hp: 3, tags: [], def: 0, atk: [1, 3], wind: 0, drawn: false, stun: 0, summoned: true }; run.floor.mons.push(b); ev.push({ t: run.turn, k: "spawn", e: ent(b) }); } }
    learn(run, `foe:${m.kind}:caster`, ev); ev.push({ t: run.turn, k: "callout", text: "conjurer summons" }); return;
  }
  if (m.tags.includes("ranged") && d >= 2 && d <= 5) {
    if (!m.drawn) { m.drawn = true; ev.push({ t: run.turn, k: "telegraph", id: m.id, what: "draws" }); learn(run, `foe:${m.kind}:ranged`, ev); return; }
    m.drawn = false; const r = hitRoll(run.rng, m.atk, h.def); ev.push({ t: run.turn, k: "attack", src: m.id, dst: 0, dmg: r.dmg, hit: r.hit, verb: "shoot" }); if (r.hit && r.dmg) hurtHero(run, r.dmg, nameOf(m), ev); return;
  }
  if (d <= 1) {
    if (m.tags.includes("thief")) {
      const it = h.inv[0];
      if (it) { h.inv.shift(); ev.push({ t: run.turn, k: "steal", id: m.id, item: it.label }); run.prov[`item:${it.kind}`] = { kind: "theft", text: `${nameOf(m)} took the ${it.label}, D${run.depth}`, t: run.turn, depth: run.depth }; ev.push({ t: run.turn, k: "note", text: `monkey stole ${it.label}` }); ev.push({ t: run.turn, k: "callout", text: "monkey steals" }); run.hl.push({ pattern: "stolen", score: 2, t: run.turn, run_id: run.id, text: `A monkey took the ${it.label}.` }); learn(run, `foe:${m.kind}:thief`, ev); run.floor.mons = run.floor.mons.filter((x) => x !== m); return; }
      // Cut 10 §3: nothing to pocket — gold goes instead, the amount on the wire (`stolen $16`)
      if (run.loot > 0) { const amount = Math.max(1, Math.round(run.loot * 0.4)); run.loot -= amount; ev.push({ t: run.turn, k: "steal", id: m.id, item: "gold", amount }); ev.push({ t: run.turn, k: "note", text: `monkey stole $${amount}` }); run.hl.push({ pattern: "stolen", score: 2, t: run.turn, run_id: run.id, text: `A monkey took $${amount}.` }); learn(run, `foe:${m.kind}:thief`, ev); run.floor.mons = run.floor.mons.filter((x) => x !== m); return; }
    }
    if (m.tags.includes("heavy")) { if (!m.wind) { m.wind = 1; ev.push({ t: run.turn, k: "telegraph", id: m.id, what: "winds up" }); learn(run, `foe:${m.kind}:heavy`, ev); return; } m.wind = 0; const r = hitRoll(run.rng, m.atk, h.def); const dmg = r.dmg * 2; ev.push({ t: run.turn, k: "attack", src: m.id, dst: 0, dmg, hit: r.hit, verb: "smash" }); if (r.hit && dmg) hurtHero(run, dmg, nameOf(m), ev); return; }
    const r = hitRoll(run.rng, m.atk, h.def); ev.push({ t: run.turn, k: "attack", src: m.id, dst: 0, dmg: r.dmg, hit: r.hit });
    if (r.hit && r.dmg) {
      hurtHero(run, r.dmg, nameOf(m), ev);
      if (m.kind === "ghoul" && run.rng() < 0.2) { h.stun = 1; ev.push({ t: run.turn, k: "callout", text: "paralysed" }); learn(run, `foe:ghoul:paralyse`, ev); }
      if (m.kind === "wraith") { h.max_hp = Math.max(1, h.max_hp - 1); h.hp = Math.min(h.hp, h.max_hp); learn(run, `foe:wraith:drain`, ev); }
    }
    return;
  }
  const moves = m.tags.includes("fast") && run.turn % 2 === 0 ? 2 : 1;
  for (let i = 0; i < moves; i++) {
    if (cheb(m.x, m.y, h.x, h.y) <= 1) break;
    const dx = Math.sign(h.x - m.x), dy = Math.sign(h.y - m.y);
    const opts: [number, number][] = [[dx, dy], [dx, 0], [0, dy]];
    for (const [ox, oy] of opts) { const nx = m.x + ox, ny = m.y + oy; if ((ox || oy) && inb(nx, ny) && passable(run.floor.tiles[idx(nx, ny)]) && !occupied(run, nx, ny) && !(m.tags.includes("water") && run.floor.tiles[idx(nx, ny)] !== "water")) { m.x = nx; m.y = ny; ev.push({ t: run.turn, k: "move", id: m.id, x: nx, y: ny }); break; } }
  }
}

function companionTurn(run: Run, m: Mon, ev: Ev[]): void {
  const h = run.hero;
  const foes = run.floor.mons.filter((x) => !x.ally && cheb(x.x, x.y, m.x, m.y) <= 6).sort((p, q) => cheb(p.x, p.y, m.x, m.y) - cheb(q.x, q.y, m.x, m.y));
  const near = foes[0];
  const strike = (t: Mon, verb: string, mult = 1): void => { const r = hitRoll(run.rng, m.atk, t.def); ev.push({ t: run.turn, k: "attack", src: m.id, dst: t.id, dmg: r.dmg * mult, hit: r.hit, verb }); if (r.hit) hurtMon(run, t, r.dmg * mult, nameOf(m), ev); };
  const stepTo = (x: number, y: number): void => { const dx = Math.sign(x - m.x), dy = Math.sign(y - m.y); for (const [ox, oy] of [[dx, dy], [dx, 0], [0, dy]] as [number, number][]) { const nx = m.x + ox, ny = m.y + oy; if ((ox || oy) && inb(nx, ny) && passable(run.floor.tiles[idx(nx, ny)]) && !occupied(run, nx, ny)) { m.x = nx; m.y = ny; ev.push({ t: run.turn, k: "move", id: m.id, x: nx, y: ny }); return; } } };
  const holds = (c: Cond): boolean => {
    switch (c.k) {
      case "self_hp<": return (m.hp / m.max_hp) * 100 < (c.n ?? 0);
      case "foes>=": return foes.length >= (c.n ?? 1);
      case "adj>=": return foes.filter((f) => cheb(f.x, f.y, m.x, m.y) <= 1).length >= (c.n ?? 1);
      case "foe_tag": return foes.some((f) => f.tags.includes(c.t ?? ""));
      case "hp<": return (h.hp / h.max_hp) * 100 < (c.n ?? 0);
      default: return condHolds(run, c);
    }
  };
  const doVerb = (v: Verb): boolean => {
    switch (v.v) {
      case "recall": run.floor.mons = run.floor.mons.filter((x) => x !== m); run.recalled.push(m); ev.push({ t: run.turn, k: "callout", text: `${nameOf(m)} recalled` }); return true;
      case "follow": if (cheb(m.x, m.y, h.x, h.y) > 2) { stepTo(h.x, h.y); return true; } return false;
      case "shoot": if (!near || !m.tags.includes("ranged") || cheb(near.x, near.y, m.x, m.y) > 5) return false; strike(near, "shoot"); return true;
      case "burst": { if (!m.tags.includes("gas") || !near || cheb(near.x, near.y, m.x, m.y) > 1) return false; for (const [dx, dy] of [[0, 0], ...DIRS]) { const x = m.x + dx, y = m.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)])) { run.floor.overlays.push({ x, y, k: "gas", ttl: 4 }); ev.push({ t: run.turn, k: "overlay", x, y, ov: "gas", ttl: 4 }); } } loseCompanion(run, m, "burst", ev); gasNear(run, m); return true; }
      case "attack": case "flank": case "drain": case "steal": case "split": {
        if (!near) return false;
        if (cheb(near.x, near.y, m.x, m.y) <= 1) { strike(near, v.v, v.v === "flank" || v.v === "drain" ? 1.5 : 1); if (v.v === "drain" && m.hp < m.max_hp) m.hp++; return true; }
        if (cheb(near.x, near.y, h.x, h.y) <= 3 || m.sent) { stepTo(near.x, near.y); return true; }
        return false;
      }
      default: return false;
    }
  };
  for (const r of m.rules?.rows ?? []) if (r.conds.every(holds) && doVerb(r.verb)) return;
  if (near && cheb(near.x, near.y, m.x, m.y) <= 1) { strike(near, "attack"); return; }
  if (cheb(m.x, m.y, h.x, h.y) > 2) stepTo(h.x, h.y);
}

function simTurn(run: Run, ctx: SimCtx): Ev[] {
  const ev: Ev[] = [];
  if (run.over) return ev;
  run.turn++; run.floorTurn++;
  run.alert = Math.min(5, Math.floor(run.floorTurn / 40));
  if (run.floorTurn > 600) { endRun(run, "return", ev); return ev; } // safety valve: a stuck hero gives up the floor
  // overlays
  for (const o of run.floor.overlays) {
    if (o.x === run.hero.x && o.y === run.hero.y) hurtHero(run, o.k === "gas" ? 3 : 5, o.k, ev);
    for (const m of [...run.floor.mons]) if (m.x === o.x && m.y === o.y && !m.tags.includes("gas")) hurtMon(run, m, o.k === "gas" ? 3 : 5, o.k, ev);
    o.ttl--;
  }
  run.floor.overlays = run.floor.overlays.filter((o) => o.ttl > 0);
  if (run.hero.hp <= 0) { die(run, ev); return ev; }
  // sightings
  run.saw = false;
  for (const m of visibleFoes(run)) {
    if (!run.kindsSeen.has(m.kind)) { run.kindsSeen.add(m.kind); run.saw = true; learn(run, `foe:${m.kind}`, ev); }
    const same = visibleFoes(run).filter((x) => x.kind === m.kind).length;
    if (same >= 2 && m.tags.includes("pack")) learn(run, `foe:${m.kind}:pack`, ev);
    if (m.tags.includes("fast") && run.turn % 2 === 0) learn(run, `foe:${m.kind}:fast`, ev);
  }
  const hp0 = run.hero.hp;
  heroTurn(run, ctx, ev);
  run.hurt = false; run.killed = false;
  if (run.over) return ev;
  const speedy = run.hero.speedT > 0; if (speedy) run.hero.speedT--;
  if (!speedy || run.turn % 2 === 0) for (const m of [...run.floor.mons]) { if (run.floor.mons.includes(m)) monsterTurn(run, m, ev); if (run.over) return ev; }
  if (run.hero.invis > 0) run.hero.invis--; if (run.hero.bashCd > 0) run.hero.bashCd--; if (run.hero.vanishCd > 0) run.hero.vanishCd--; if (run.cleaveCd > 0) run.cleaveCd--;
  if (run.bulwark > 0) { run.bulwark--; if (run.bulwark === 0) run.hero.def -= 3; }
  bossInView(run, ev);                      // Cut 6 §5: a boss that walked into view this turn
  if (run.hero.hp <= 0) die(run, ev);
  else if (hp0 <= run.hero.max_hp * 0.2 && run.hero.hp > hp0 && visibleFoes(run).length === 0) { run.hl.push({ pattern: "near_death", score: 5, t: run.turn, run_id: run.id, text: `Down to ${hp0} hp on D${run.depth}. Lived.` }); ev.push({ t: run.turn, k: "note", text: `survived at ${hp0} hp` }); }
  return ev;
}
function die(run: Run, ev: Ev[]): void {
  const lh = run.lastHurt; let cause = lh?.cause ?? "unknown";
  const same = adjFoes(run).filter((m) => nameOf(m) === cause).length;
  if (same >= 2) cause += " pack";
  run.cause = cause;
  ev.push({ t: run.turn, k: "die", id: 0, cause });
  ev.push({ t: run.turn, k: "note", text: `died to ${cause} on D${run.depth}` });
  ev.push({ t: run.turn, k: "callout", text: cause });
  const kit = bonesKit(run); if (kit.length) ev.push({ t: run.turn, k: "bones", heir: run.heir, items: kit.length });
  endRun(run, "death", ev);
}
/** What a death leaves on the floor: everything carried plus the equipped kit (insured brought items excepted). */
function bonesKit(run: Run): InvItem[] {
  const out = [...run.gear, ...run.hero.inv, ...run.broughtItems].filter((i, k, a) => a.findIndex((x) => x.id === i.id) === k);
  return out.filter((i) => !(run.brought.includes(i.id) && run.insured.has(i.id)));
}
function snapshot(run: Run, rules?: RuleSet): Snapshot {
  const h = run.hero;
  return {
    depth: run.depth, biome: biomeOf(run.depth), w: W, h: H, tiles: run.floor.tiles.slice(), seen: run.floor.seen.slice(), visible: run.floor.visible.slice(),
    overlays: run.floor.overlays.map((o) => ({ ...o })),
    hero: { id: 0, kind: `hero_${run.cls}`, x: h.x, y: h.y, hp: h.hp, max_hp: h.max_hp, tags: h.invis ? ["invisible"] : [], inv: h.inv.map((i) => ({ ...i })), weapon: h.weapon, armour: h.armour, class: run.cls, trait: run.trait },
    // Cut 4 §3 (UI dev stand-in for the core): a hostile seen before and out of sight now is `remembered`
    entities: run.floor.mons.map((m) => { const vis = run.floor.visible[idx(m.x, m.y)]; if (vis) m.seen = true; const e = ent(m); if (!vis && m.seen && !m.ally) e.remembered = true; return e; }),
    items: run.floor.items.map((i) => ({ ...i })), alert: run.alert, turn: run.turn, loot: run.loot,
    run: { id: run.id, heir: run.heir, started_turn: run.startedTotal },   // Cut 11 §5: the lineage tick this run started at
    stake: stakeOf(run, rules), vision: DEV_VISION,
    room: roomOf(run), rooms: run.floor.rooms.length,   // Cut 7 §4
    ...(run.twist ? { floor_twist: run.twist } : {}),   // Cut 12 §4
  };
}
/** Cut 7 §4: the room the hero stands in (0 = corridor) and the hostiles in it (the fake has no sleep: all awake). */
function roomOf(run: Run): Snapshot["room"] {
  const r = run.floor.roomOf[idx(run.hero.x, run.hero.y)];
  if (r < 0) return { id: 0, hostiles: 0 };
  return { id: r + 1, hostiles: run.floor.mons.filter((m) => !m.ally && m.hp > 0 && run.floor.roomOf[idx(m.x, m.y)] === r).length };
}
/** Cut 2 §7: loot on the hero, brought items (insured = safe), the first row that would bank or return. */
function stakeOf(run: Run, rules?: RuleSet): Snapshot["stake"] {
  const brought = run.broughtItems.map((i) => ({ label: i.label, insured: run.insured.has(i.id) }));
  const rows = rules?.rows ?? []; const ri = rows.findIndex((r) => r.verb.v === "return" || r.verb.v === "bank");
  // Cut 6 §1: what that row would bring home now (the kept number, not the carried one)
  return { loot: run.loot, brought, return_row: ri >= 0 ? ri : undefined, kept: ri >= 0 ? Math.round(run.loot * (rows[ri].verb.v === "bank" ? 1 : 0.6)) : undefined };
}
function runToEnd(run: Run, ctx: SimCtx, maxTurns = 3000): void { while (!run.over && run.turn < maxTurns) simTurn(run, ctx); if (!run.over) endRun(run, "return", []); }

// --- engine ---
type State = {
  lineage: Lineage; rules: RuleSet; loadout: number[]; killed: string[]; runCounter: number; logs: Record<number, RunLog>; nextItem: number;
  tamedKinds: string[]; bredKinds: string[]; nextCid: number; killCounts: Record<string, number>;
  totalTurns: number;   // Cut 11 §5: the lineage tick (×10 like the events): `GoldLine.t` and `Snapshot.run.started_turn`, as the core's `total_turns`
};

export class FakeEngine implements Engine {
  private s!: State;
  private live: Run | null = null;
  private lastDeath: Record<number, Death> = {};
  private settled = new Set<number>();

  private flavourMap(): Map<string, string> {
    const r = mulberry32(this.s.lineage.seed ^ 0x5eed); const f = [...FLAVOURS], m = new Map<string, string>();
    for (const p of POTIONS) { const i = Math.floor(r() * f.length); m.set(p, f.splice(i, 1)[0]); }
    const runes = [...RUNES]; for (const sc of SCROLLS) { const i = Math.floor(r() * runes.length); m.set(sc, runes.splice(i, 1)[0]); }
    return m;
  }
  private ctx(rules = this.s.rules): SimCtx {
    const fm = this.flavourMap(); const inv = new Map([...fm].map(([k, v]) => [v, k]));
    const cat = this.supplyCatalogue();
    return { rules, unlocks: new Set(this.s.lineage.unlocks), flav: (k) => fm.get(k) ?? k, kindOfFlav: (f) => inv.get(f), tier: (k) => this.s.lineage.forge?.[k]?.tier ?? 0,
      bones: this.s.lineage.bones ?? [], insured: new Set(this.s.lineage.insured ?? []),
      gold: this.s.lineage.gold, spent: (this.s.lineage.supplies ?? []).map((it) => ({ label: it.label, price: cat.find((c) => c.kind === it.kind)?.price ?? 0 })) };
  }
  /** Cut 12 §1: the cap on the player's OWN rows; card rows (`{v:"tactic"}`) sit outside it, one per owned card. */
  private maxRows(): number { return 4 + ["row5", "row6", "row7", "row8"].filter((u) => this.s.lineage.unlocks.includes(u)).length; }
  private supplyCap(): number { return this.s.lineage.unlocks.includes("supply_cap_5") ? 5 : 3; }
  /** Cut 12 §1: where a bought card's row goes — before the set's engagement row (the first `attack` / `shoot`), else the end. */
  private cardInsertAt(): number { const i = this.s.rules.rows.findIndex((r) => r.verb.v === "attack" || r.verb.v === "shoot"); return i < 0 ? this.s.rules.rows.length : i; }
  private vaultSlots(): number { return 1 + ["vault2", "vault3", "vault4"].filter((u) => this.s.lineage.unlocks.includes(u)).length; }

  newLineage(seed: number): Lineage {
    const trait = TRAITS[(seed >>> 3) % TRAITS.length];
    const copy = (): RuleSet => ({ rows: PRESET_FIGHTER.rows.map((r) => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb } })) });
    this.s = {
      lineage: {
        seed, heir: 1 + SEED_CHRONICLE.length, trait, class: "fighter", best_depth: 0, marks: 0, facts: ["item:leash"], unlocks: ["tame"], vault: [], graveyard: [], trophies: [], sets: [copy(), copy(), copy()], active_set: 0, ended: false,
        party: [], kennel: [mkCompanion(1, "jackal", 2, ["pack", "fast"], 0), mkCompanion(2, "goblin_archer", 1, ["ranged"], 0)],
        eggs: [{ id: 3, kind: "bloat", tags: ["gas"], gen: 1, hatch_in: 3, from_loss: false }], party_slots: 1, ledger: [],
        gold: 120, supplies: [{ id: 49_999, kind: "leash", known: true, label: "leash", free: true }], classes: Object.fromEntries(CLASSES.map((c) => [c, { level: 1, xp: 0 }])),   // Cut 8B §3: the kennel's leash
        forge: { dagger: { salvaged: 6, craftable: true, tier: 0 } }, renown: 0, rank: 0, keep_pref: "best_weapon", vault_pref: "weapon",
        rest_left_s: 0, bones: [],
        chronicle: SEED_CHRONICLE.map(([trait, cls, depth, deed, end, tail], i) => chronicleLine(i + 1, trait, cls, depth, deed, end, tail)),
      },
      rules: copy(), loadout: [], killed: [], runCounter: 0, logs: {}, nextItem: 50000, tamedKinds: ["jackal", "goblin_archer"], bredKinds: ["bloat"], nextCid: 10, killCounts: {}, totalTurns: 0,
    };
    this.live = null; this.lastDeath = {};
    return this.lineage();
  }
  load(save: string): Lineage {
    const p = JSON.parse(save) as State; this.s = p; this.live = null; this.lastDeath = {};
    const L = p.lineage; // older fake saves: fill addendum fields
    L.party ??= []; L.kennel ??= []; L.eggs ??= []; L.party_slots ??= 1; L.ledger ??= []; L.gold ??= 0; L.supplies ??= [];
    L.classes ??= { fighter: { level: 1, xp: 0 }, rogue: { level: 1, xp: 0 } };
    L.forge ??= {}; L.renown ??= 0; L.rank ??= 0; L.keep_pref ??= "best_weapon"; L.vault_pref ??= "weapon";
    L.rest_left_s ??= 0; L.bones ??= []; for (const c of CLASSES) L.classes[c] ??= { level: 1, xp: 0 };
    p.tamedKinds ??= []; p.bredKinds ??= []; p.nextCid ??= 10; p.killCounts ??= {}; p.totalTurns ??= 0;
    return this.lineage();
  }
  save(): string { return JSON.stringify(this.s); }
  lineage(): Lineage {
    this.s.lineage.ledger = this.ledger();
    this.s.lineage.counters = this.counters();
    this.s.lineage.combos = combosIn(this.s.rules.rows, COMBOS);   // Cut 8B §1
    // Cut 9 §10: the forge ladder's next rung per kind (`3/5 → craftable`, `6/15 → +1`, `20/40 → +2`; none at the top)
    for (const f of Object.values(this.s.lineage.forge ?? {})) { const rung = FORGE_LADDER.find((r) => f.salvaged < r.need); if (rung) f.next = { ...rung }; else delete f.next; }
    return JSON.parse(JSON.stringify(this.s.lineage)) as Lineage;
  }
  /** Cut 6 §5: bosses whose counter fact is known, with the counter as a row. */
  private counters(): Counter[] {
    // the core merges facts into the lineage as they are learned; the live run's facts count here for the same reason
    return [...new Set([...this.s.lineage.facts, ...(this.live?.facts ?? [])])].flatMap((f) => { const m = /^boss:([a-z_]+):counter$/.exec(f); const c = m && COUNTER[m[1]]; return c ? [{ boss: m[1], row: c.row, text: c.text }] : []; });
  }
  /** Cut 6 §1: every gold movement is a ledger line (`+$50 returned D5`, `−$40 heal potion`), the last 20 kept, oldest first. */
  private gold(delta: number, why: string): void {
    const L = this.s.lineage; L.gold += delta;
    const g = (L.gold_ledger ??= []); g.push({ t: this.s.totalTurns, delta, why }); while (g.length > GOLD_LEDGER_CAP) g.shift();   // Cut 11 §5: the lineage tick
  }
  private ledger(): LedgerRow[] {
    const F = this.s.lineage.facts;
    const known = new Set([...F, ...(this.live?.facts ?? [])]);   // Cut 7 §1: the live run's counter fact counts, as in `counters()`
    return LEDGER_KINDS.map((kind) => ({ kind, seen: F.includes(`foe:${kind}`), known: F.includes(`foe:${kind}`) && MON[kind].tags.filter((t) => t !== "boss").every((t) => F.includes(`foe:${kind}:${t}`)), studied: F.includes(`foe:${kind}:studied`), tamed: this.s.tamedKinds.includes(kind), bred: this.s.bredKinds.includes(kind),
      counter: known.has(`boss:${kind}:counter`) && COUNTER[kind] ? { row: COUNTER[kind].row, text: COUNTER[kind].text } : undefined }));   // Cut 7 §1
  }
  private classLevel(cls = this.s.lineage.class): number { return this.s.lineage.classes[cls]?.level ?? 1; }
  private partySlots(): number { return this.s.lineage.unlocks.includes("party_slot_2") ? 2 : 1; }
  setParty(ids: number[]): Lineage {
    const L = this.s.lineage; const all = [...L.kennel, ...L.party];
    const chosen = ids.map((id) => all.find((c) => c.id === id)).filter((c): c is Companion => !!c).slice(0, this.partySlots());
    L.party = chosen; L.kennel = all.filter((c) => !chosen.includes(c)); L.party_slots = this.partySlots(); this.dropIdleRun();
    return this.lineage();
  }
  setCompanionRules(id: number, set: RuleSet): void {
    const c = [...this.s.lineage.kennel, ...this.s.lineage.party].find((x) => x.id === id); if (!c) return;
    c.rules = { rows: set.rows.slice(0, c.max_rows).map((r) => ({ conds: r.conds.slice(0, 2).map((x) => ({ ...x })), verb: { ...r.verb } })) };
  }
  breed(a: number, b: number): Lineage {
    const L = this.s.lineage; const A = L.kennel.find((c) => c.id === a), B = L.kennel.find((c) => c.id === b);
    if (!A || !B || A === B || A.level < 2 || B.level < 2) return this.lineage();
    const extra = B.tags.find((t) => !A.tags.includes(t)); const tags = [...A.tags, ...(extra ? [extra] : [])].slice(0, 3);
    L.kennel = L.kennel.filter((c) => c !== A && c !== B);
    L.eggs.push({ id: this.s.nextCid++, kind: A.kind, tags, gen: Math.max(A.gen, B.gen) + 1, hatch_in: 5, from_loss: false });
    if (!this.s.bredKinds.includes(A.kind)) this.s.bredKinds.push(A.kind);
    return this.lineage();
  }
  hatch(eggId: number): Lineage {
    const L = this.s.lineage; const e = L.eggs.find((x) => x.id === eggId);
    if (e && e.from_loss && L.gold >= 50) { this.gold(-50, `hatch ${e.kind.replace(/_/g, " ")}`); L.eggs = L.eggs.filter((x) => x !== e); L.kennel.push(mkCompanion(this.s.nextCid++, e.kind, 1, e.tags, e.gen)); }
    return this.lineage();
  }
  // Addendum B — supplies
  supplyCatalogue(): SupplyEntry[] {
    const out: SupplyEntry[] = [{ kind: "leash", price: 30, label: "leash" }];
    const ided = new Set<string>();
    for (const f of this.s.lineage.facts) { const m = /^item:[A-Za-z]+=([a-z_]+)$/.exec(f); if (!m) continue; const k = m[1]; ided.add(k); if (POTIONS.includes(k)) out.push({ kind: k, price: 40, label: `${k} potion` }); else if (SCROLLS.includes(k)) out.push({ kind: k, price: 60, label: `scroll ${k}` }); }
    for (const [kind, f] of Object.entries(this.s.lineage.forge ?? {})) {
      if (!f.craftable || out.some((o) => o.kind === kind)) continue;
      if ((POTIONS.includes(kind) || SCROLLS.includes(kind)) && !ided.has(kind)) continue;
      out.push({ kind, price: 2 * salvageOf(kind), label: POTIONS.includes(kind) ? `${kind} potion` : SCROLLS.includes(kind) ? `scroll ${kind}` : kind });
    }
    return out;
  }
  buySupply(kind: string): Lineage {
    const L = this.s.lineage; const e = this.supplyCatalogue().find((x) => x.kind === kind);
    if (e && L.supplies.length < this.supplyCap() && L.gold >= e.price) { this.gold(-e.price, e.label); L.supplies.push({ id: this.s.nextItem++, kind: e.kind, known: true, label: e.label }); }
    return this.lineage();
  }
  clearSupplies(): Lineage { const L = this.s.lineage; for (const s of L.supplies) if (!s.free) this.gold(this.supplyCatalogue().find((x) => x.kind === s.kind)?.price ?? 0, `refund ${s.label}`); L.supplies = []; return this.lineage(); }
  /** Cut 12 §6: one line off the shelf, refunded unless it was free. */
  dropSupply(id: number): Lineage {
    const L = this.s.lineage; const s = L.supplies.find((x) => x.id === id); if (!s) return this.lineage();
    if (!s.free) this.gold(this.supplyCatalogue().find((x) => x.kind === s.kind)?.price ?? 0, `refund ${s.label}`);
    L.supplies = L.supplies.filter((x) => x !== s); return this.lineage();
  }
  companionVocabulary(id: number): Vocabulary {
    const c = [...this.s.lineage.kennel, ...this.s.lineage.party].find((x) => x.id === id);
    const base = this.vocabulary();
    const conds = base.conds.filter((k) => k.k !== "party" && k.k !== "party_hp<").map((k) => (k.k === "hp<" ? { k: "self_hp<" } : k));
    const verbs: Verb[] = [{ v: "attack" }, { v: "follow" }, { v: "recall" }];
    for (const t of c?.tags ?? []) if (TAG_VERB[t]) verbs.push({ v: TAG_VERB[t] });
    return { conds, verbs, max_rows: c?.max_rows ?? 2 };
  }
  private tickEggs(hatched: string[]): void {
    const L = this.s.lineage;
    for (const e of [...L.eggs]) { e.hatch_in -= 1; if (e.hatch_in <= 0) { L.eggs = L.eggs.filter((x) => x !== e); L.kennel.push(mkCompanion(this.s.nextCid++, e.kind, 1, e.tags, e.gen)); hatched.push(e.kind); } }
  }

  vocabulary(): Vocabulary {
    const L = this.s.lineage; const tags = new Set<string>(); const kinds = new Set<string>();
    for (const f of L.facts) { const m = /^foe:[a-z_]+:([a-z]+)$/.exec(f); if (m) tags.add(m[1]); const k = /^item:[A-Za-z]+=([a-z_]+)$/.exec(f); if (k) kinds.add(k[1]); }
    const conds: Cond[] = [{ k: "hp<" }, { k: "hp>" }, { k: "foes>=" }, { k: "adj>=" }];
    for (const t of tags) conds.push({ k: "foe_tag", t });
    conds.push({ k: "foe_hp<" });
    for (const k of kinds) conds.push({ k: "item", t: k });
    conds.push({ k: "unknown_item" }, { k: "floor_seen>=" }, { k: "depth>=" }, { k: "alert>=" }, { k: "in_corridor" }, { k: "path_stairs" }, { k: "ally" }, { k: "loot>=" }, { k: "turns>" }, { k: "on_hurt" }, { k: "on_kill" }, { k: "on_see" });
    if (L.party.length || L.kennel.length) { conds.push({ k: "party_hp<" }); for (const k of new Set([...L.party, ...L.kennel].map((c) => c.kind))) conds.push({ k: "party", t: k }); }
    // Cut 2 §3: condition tokens are unlocks now. Cut 9 §1: the gated ones ride along as `locked` with the gate as text
    // (the fact still missing, else the price) so the sheet shows why, and never offers them.
    const gated = conds.filter((c) => !COND_UNLOCK[c.k] || L.unlocks.includes(COND_UNLOCK[c.k]));
    const locked = conds.filter((c) => COND_UNLOCK[c.k] && !L.unlocks.includes(COND_UNLOCK[c.k]))
      .map((cond) => { const u = UNLOCKS[COND_UNLOCK[cond.k]]; return { cond, needs: u.gate && !u.gate(L) ? u.needs ?? "?" : `◆${u.cost}` }; });
    const verbs: Verb[] = [{ v: "attack", a: "nearest" }, { v: "attack", a: "lowest" }];
    for (const t of tags) verbs.push({ v: "attack", a: `tag:${t}` });
    verbs.push({ v: "retreat" }, { v: "back_corridor" });
    for (const k of kinds) if (POTIONS.includes(k) && !THROWABLE.has(k)) verbs.push({ v: "drink", a: k });
    verbs.push({ v: "drink", a: "unknown" });
    for (const k of kinds) if (SCROLLS.includes(k)) verbs.push({ v: "read", a: k });
    verbs.push({ v: "read", a: "unknown" });
    if (L.unlocks.includes("throw") || verbsUpTo(L.class, this.classLevel()).includes("throw")) { for (const k of kinds) if (THROWABLE.has(k)) verbs.push({ v: "throw", a: `${k},nearest` }); verbs.push({ v: "throw", a: "unknown,nearest" }); }
    verbs.push({ v: "descend" }, { v: "bank" }, { v: "return" }, { v: "rest" }, { v: "pick_up" }, { v: "free_captive" });
    for (const v of verbsUpTo(L.class, this.classLevel())) if (v !== "throw") verbs.push({ v });
    if (L.unlocks.includes("tame")) { verbs.push({ v: "tame", a: "nearest" }); for (const t of tags) verbs.push({ v: "tame", a: `tag:${t}` }); }
    if (L.party.length) { verbs.push({ v: "recall" }, { v: "send" }); }
    for (const c of TACTIC_CARDS) if (L.unlocks.includes(c)) verbs.push({ v: "tactic", a: c });
    return { conds: gated, verbs, max_rows: this.maxRows(), combos: COMBOS, locked };
  }
  setRules(set: RuleSet): void {
    // Cut 12 §1: own rows ≤ max_rows and card rows ≤ cards owned (one per card) — refused, never truncated
    const own = set.rows.filter((r) => r.verb.v !== "tactic").length; if (own > this.maxRows()) throw new Error(`${own} rows over ${this.maxRows()}`);
    const cards = set.rows.filter((r) => r.verb.v === "tactic").map((r) => r.verb.a ?? "");
    if (new Set(cards).size !== cards.length) throw new Error("a card twice");
    for (const c of cards) if (!this.s.lineage.unlocks.includes(c)) throw new Error(`card ${c} not owned`);
    this.home = 0;                                                           // a rule edit opens a fresh stall window
    this.s.rules = { rows: set.rows.map((r) => ({ conds: (r.verb.v === "tactic" ? [] : r.conds.slice(0, 2)).map((c) => ({ ...c })), verb: { ...r.verb } })), name: set.name };
    this.s.lineage.sets[this.s.lineage.active_set] = JSON.parse(JSON.stringify(this.s.rules)) as RuleSet;
  }
  loadout(itemIds: number[]): void { this.s.loadout = itemIds.filter((id) => this.s.lineage.vault.some((v) => v.id === id)); this.dropIdleRun(); }
  /** A live run that has not taken a turn yet (the report's `live` peek) is rebuilt on the next send, so loadout/class/party changes land. */
  private dropIdleRun(): void { if (this.live && !this.live.over && this.live.turn === 0 && this.live !== this.pending) this.live = null; }

  private known(): Set<string> { return new Set(this.s.lineage.facts); }
  private brought(): InvItem[] { return [...this.s.lineage.vault.filter((v) => this.s.loadout.includes(v.id)), ...this.s.lineage.supplies].map((v) => ({ ...v })); }
  private simOne(seed: number, rules: RuleSet, known: Set<string>, cls = this.s.lineage.class, trait = this.s.lineage.trait, brought: InvItem[] = []): Run {
    const ctx = this.ctx(rules); const run = makeRun(0, this.s.lineage.heir, seed, cls, trait, known, brought, ctx, [], this.classLevel(cls)); runToEnd(run, ctx); return run;
  }

  forecast(): Forecast { return this.forecastN(20); }
  /** Cut 6 §9: the same forecast at 100 sims (the client asks 2 s after a quiet paint). Same seeds ⇒ the first 20 agree. */
  forecastRefine(): Forecast { return this.forecastN(100); }
  private forecastN(N: number): Forecast {
    const L = this.s.lineage; const known_to = L.best_depth + 1;
    const reach = new Array(16).fill(0); const causes: Record<string, number> = {};
    const ends = { bank: 0, return: 0, death: 0, gold: 0 };   // Cut 12 §3: how a send ends, and the mean gold brought home
    for (let i = 0; i < N; i++) {
      const r = this.simOne(hash(`fc:${L.seed}:${i}`), this.s.rules, this.known());
      for (let d = 1; d <= r.depth; d++) reach[d]++;
      if (r.exit === "death") causes[r.cause ?? "?"] = (causes[r.cause ?? "?"] ?? 0) + 1;
      ends[r.exit ?? "death"] += 1; ends.gold += r.loot_kept;
    }
    // Cut 9 §3: `pm` = the binomial half-width (1.96 σ, a fraction like `reach`), so a wobble between reads reads as noise
    const depths: Forecast["depths"] = []; for (let d = 1; d <= Math.min(15, known_to); d++) { const p = reach[d] / N; depths.push({ depth: d, reach: p, pm: 1.96 * Math.sqrt((p * (1 - p)) / N) }); }
    const top = Object.entries(causes).sort((a, b) => b[1] - a[1]).slice(0, 3).map(([cause, n]) => ({ cause, share: n / N }));
    // Cut 10 §2: a boss floor whose counter fact is known and whose row is absent from the set names it: `try: attack boss`
    for (const d of depths) {
      const boss = BOSS[d.depth]; const c = boss && COUNTER[boss];
      if (!c || !this.known().has(`boss:${boss}:counter`) || this.s.rules.rows.some((r) => rowText(r) === rowText(c.row))) continue;
      d.cause ??= boss; d.try = { row: JSON.parse(JSON.stringify(c.row)) as Row, text: c.text };
    }
    return { depths, causes: top, known_to, ends: { bank: ends.bank / N, return: ends.return / N, death: ends.death / N, gold: ends.gold / N } };
  }

  private startRun(): Run {
    const L = this.s.lineage; this.s.runCounter++;
    const id = this.s.runCounter; const seed = hash(`run:${L.seed}:${id}`);
    const run = makeRun(id, L.heir, seed, L.class, L.trait, this.known(), this.brought(), this.ctx(), L.party, this.classLevel());
    run.startedTotal = this.s.totalTurns;
    this.s.logs[id] = { seed, rules: JSON.parse(JSON.stringify(this.s.rules)) as RuleSet, depth: 1, turns: 0, exit: "", known: [...this.known()], cls: L.class, trait: L.trait, heir: L.heir, hpMargin: 0 };
    return run;
  }
  send(): Snapshot { this.s.lineage.rest_left_s = 0; if (!this.live || this.live.over) this.live = this.startRun(); const snap = snapshot(this.live, this.s.rules); snap.turn *= 10; return snap; }
  // The fake sims one hero action per turn; `step` takes ticks (Addendum E: 10 per action) so the
  // viewer's clock reads it at the real pace. Event `t` and `snapshot.turn` are scaled ×10.
  private tickAcc = 0;
  step(ticks: number): StepResult {
    if (!this.live) this.live = this.startRun();
    const ctx = this.ctx(); const events: Ev[] = [];
    this.tickAcc += ticks; const turns = Math.floor(this.tickAcc / 10); this.tickAcc -= turns * 10;
    // Cut 5 §5: a bail fires `return` on the hero's next action as a chore (the rules untouched)
    if (this.bailed && turns > 0 && !this.live.over) { this.bailed = false; this.live.turn++; events.push({ t: this.live.turn, k: "rule", row: -2, verb: { v: "return" }, text: "bail → return" }); endRun(this.live, "return", events); }
    for (let i = 0; i < turns && !this.live.over; i++) { events.push(...simTurn(this.live, ctx)); this.s.totalTurns += 10; }
    let exit_pending: StepResult["exit_pending"];
    if (this.live.over && !this.pending && !this.settled.has(this.live.id)) {
      const items = this.live.exit === "death" ? [] : this.carried(this.live);   // Cut 2 §2: a death leaves bones, nothing to keep
      if (items.length) { this.pending = this.live; exit_pending = { items, tier: this.live.exit ?? "death" }; }
      else { const r = this.settle(this.live, true); for (const level of r.levels) events.push({ t: this.live.turn, k: "level", class: this.live.cls, level }); for (const rank of r.ranks) events.push({ t: this.live.turn, k: "rank", rank }); }
      this.settled.add(this.live.id);
    }
    for (const e of events) e.t *= 10;
    const snap = snapshot(this.live, ctx.rules); snap.turn *= 10;
    return { events, snapshot: snap, run_over: this.live.over, exit_pending };
  }
  // apply a finished run to the lineage; returns [newFacts, newBests, marks]
  private carried(run: Run): InvItem[] { return [...run.gear.filter((g) => !run.brought.includes(g.id)), ...run.hero.inv.filter((i) => !run.brought.includes(i.id) && !(this.s.lineage.supplies ?? []).some((s) => s.id === i.id))]; }
  private autoKeep(run: Run): number[] {
    const items = this.carried(run); const pref = this.s.lineage.keep_pref;
    const pickBest = (score: (i: InvItem) => number): number[] => { const c = items.filter((i) => score(i) > 0).sort((a, b) => score(b) - score(a))[0]; return c ? [c.id] : []; };
    if (pref === "best_weapon") return pickBest((i) => WEAPONS[i.kind]?.[1] ?? 0);
    if (pref === "best_armour") return pickBest((i) => ARMOUR[i.kind] ?? 0);
    return [];
  }
  private settle(run: Run, real: boolean, keepIds: number[] = []): { facts: string[]; bests: string[]; marks: number; tamed: string[]; lost: string[]; hatched: string[]; xp: number; levels: number[]; salvaged: { kind: string; n: number; gold: number }[]; score: number; ranks: number[] } {
    const L = this.s.lineage; const log = this.s.logs[run.id];
    if (log) { log.depth = run.depth; log.cause = run.cause; log.turns = run.turn; log.exit = run.exit ?? ""; log.line = run.line; log.hpMargin = run.lastHurt ? run.lastHurt.dmg - run.lastHurt.hpBefore + 1 : 0; if (run.exit === "death") log.turns = run.turn; }
    if (!real) return { facts: [], bests: [], marks: 0, tamed: [], lost: [], hatched: [], xp: 0, levels: [], salvaged: [], score: 0, ranks: [] };
    const facts: string[] = []; const bests: string[] = []; let marks = 0;
    const tamed: string[] = [], lost: string[] = [], hatched: string[] = [];
    this.gold(run.loot_kept, `${run.exit === "bank" ? "banked" : run.exit === "return" ? "returned" : "died"} D${run.depth}`); L.supplies = [];
    L.rest_left_s = run.rest_s;                                             // Cut 2 §1: camp rest after every expedition (send skips it)
    // Cut 2 §2: bones recovered this run leave the lineage; a death leaves a new pile (max 3, oldest expires)
    L.bones = (L.bones ?? []).filter((b) => !run.bonesFound.some((f) => f.heir === b.heir && f.depth === b.depth));
    if (run.exit === "death") {
      const kit = bonesKit(run).length; if (kit) { L.bones.push({ depth: run.depth, heir: run.heir, items: kit }); while (L.bones.length > BONES_MAX) L.bones.shift(); }
      const bf = `bones:${run.depth}`; if (!L.facts.includes(bf)) { L.facts.push(bf); facts.push(bf); }
    }
    // Cut 2 §5: a kind is studied after 5 kills
    for (const [k, n] of Object.entries(run.kills)) { const c = (this.s.killCounts[k] = (this.s.killCounts[k] ?? 0) + n); const sf = `foe:${k}:studied`; if (c >= STUDIED_KILLS && !L.facts.includes(sf)) { L.facts.push(sf); facts.push(sf); } }
    // companions: party members and tamed foes
    const survivors = run.floor ? [...companions(run), ...run.recalled] : [...run.recalled];
    const egg = (c: Companion): void => { L.eggs.push({ id: this.s.nextCid++, kind: c.kind, tags: [...c.tags], gen: c.gen, hatch_in: 5, from_loss: true }); lost.push(c.kind); };
    for (const c of run.lostC) { L.party = L.party.filter((p) => p.id !== c.id); egg(c); }
    if (run.exit === "death") { for (const c of L.party) egg(c); L.party = []; }
    else {
      for (const c of L.party) { const m = survivors.find((s) => s.cid === c.id); if (m) c.hp = c.max_hp; if (run.exit === "bank") { c.level = Math.min(5, c.level + 1); c.max_rows = 1 + c.level; } }
      for (const m of run.tamed) if (survivors.includes(m)) { L.kennel.push(mkCompanion(this.s.nextCid++, m.kind, 1, [...m.tags], 0)); tamed.push(m.kind); if (!this.s.tamedKinds.includes(m.kind)) this.s.tamedKinds.push(m.kind); }
    }
    this.tickEggs(hatched);
    // class xp (Addendum C; Cut 2 §2: death 0)
    const tier = run.exit === "bank" ? 1 : run.exit === "return" ? 0.6 : 0;
    const xp = Math.round((run.killXp + 5 * run.depth) * tier);
    const cl = (L.classes[run.cls] ??= { level: 1, xp: 0 }); cl.xp += xp; const levels: number[] = [];
    while (cl.level < XP_LEVEL_CAP && cl.xp >= xpToNext(cl.level)) { cl.xp -= xpToNext(cl.level); cl.level++; levels.push(cl.level); for (const v of verbsAt(run.cls, cl.level)) facts.push(`verb:${v}`); }
    if (cl.level >= XP_LEVEL_CAP && !L.trophies.includes(`master:${run.cls}`)) { L.trophies.push(`master:${run.cls}`); marks += 2; bests.push(`master ${run.cls}`); }
    for (const f of run.facts) if (!L.facts.includes(f)) { L.facts.push(f); facts.push(f); }
    for (let d = L.best_depth + 1; d <= run.depth; d++) { marks += 1; bests.push(`D${d}`); }
    if (run.depth > L.best_depth) L.best_depth = run.depth;
    // Cut 2 §2: no first-kill marks; a boss kill is a trophy (3 marks) and first kills stay in bests
    for (const k of Object.keys(run.kills)) if (!this.s.killed.includes(k)) { this.s.killed.push(k); const boss = MON[k]?.tags.includes("boss"); if (boss) { marks += 3; L.trophies.push(`boss:${k}`); } bests.push(`first kill ${k.replace(/_/g, " ")}`); run.hl.push({ pattern: "first_kill", score: 3, t: run.turn, run_id: run.id, text: `First ${k.replace(/_/g, " ")} killed, D${run.depth}.` }); }
    if (run.depth >= 5 && !this.s.rules.rows.some((r) => r.verb.v === "drink") && !L.trophies.includes("no_heal_D5")) { L.trophies.push("no_heal_D5"); marks += 2; bests.push("trophy no_heal_D5"); }
    L.marks += marks;
    if (run.exit === "death") {
      L.graveyard.push({ heir: run.heir, depth: run.depth, cause: run.cause ?? "?", deeds: bests.slice(0, 3), death_id: run.id });   // Cut 9 §7: the fake keeps every log, so every grave opens
      // Cut 5 §2: the heir's chronicle line (the set's name, its best deed, its end, its bones)
      const kit = bonesKit(run).length;
      const deed = L.trophies.filter((t) => t.startsWith("boss:")).slice(-1).map((t) => `took the ${t.slice(5).replace(/_/g, " ")}`)[0] ?? bests.find((b) => b.startsWith("first kill"))?.replace(/^first kill /, "first ");
      (L.chronicle ??= []).push(chronicleLine(run.heir, L.trait, run.cls, run.depth, deed, `fell to ${run.cause ?? "?"}`, kit ? `left bones on D${run.depth}` : undefined, this.s.rules.name));
      while (L.chronicle.length > CHRONICLE_CAP) L.chronicle.shift();
      L.vault = L.vault.filter((v) => !run.brought.includes(v.id) || run.insured.has(v.id)); L.insured = (L.insured ?? []).filter((id) => !run.brought.includes(id)); this.s.loadout = [];
      L.heir += 1; L.trait = TRAITS[(L.seed + L.heir * 7) % TRAITS.length];
    } else {
    }
    // Addendum D: keep into free vault slots, salvage the rest (Cut 2 §2: a death's kit is bones, not salvage)
    const tierMul = run.exit === "bank" ? 1 : 0.6;
    const salv: Record<string, { n: number; gold: number }> = {};
    for (const it of run.exit === "death" ? [] : this.carried(run)) {
      if (keepIds.includes(it.id) && L.vault.length < this.vaultSlots()) { L.vault.push({ ...it, id: this.s.nextItem++ }); continue; }
      const g = Math.round(salvageOf(it.kind) * tierMul); this.gold(g, `salvaged ${it.kind.replace(/_/g, " ")}`);
      const f = (L.forge[it.kind] ??= { salvaged: 0, craftable: false, tier: 0 }); f.salvaged++; f.craftable = f.salvaged >= 5; f.tier = f.salvaged >= 40 ? 2 : f.salvaged >= 15 ? 1 : 0;
      (salv[it.kind] ??= { n: 0, gold: 0 }).n++; salv[it.kind].gold += g;
    }
    const salvaged = Object.entries(salv).map(([kind, v]) => ({ kind, ...v }));
    // renown
    const score = 10 * run.depth + run.score + run.hl.reduce((a, b) => a + b.score, 0); L.renown += score; const ranks: number[] = [];
    while (L.renown >= 100 * (L.rank + 1) * (L.rank + 1)) { L.rank++; L.marks++; ranks.push(L.rank); }
    return { facts, bests, marks, tamed, lost, hatched, xp, levels, salvaged, score, ranks };
  }
  private pending: Run | null = null;
  keep(ids: number[]): Lineage {
    const run = this.pending; if (!run) return this.lineage();
    this.pending = null; this.settle(run, true, ids); return this.lineage();
  }
  setKeepPref(pref: string): Lineage { if (["best_weapon", "best_armour", "none"].includes(pref)) this.s.lineage.keep_pref = pref; return this.lineage(); }
  // Cut 5 stand-ins: the fake places no vaults, so `choose` only answers with the live snapshot; `bail` ends the run on the next step
  private bailed = false;
  bail(): void { if (this.live && !this.live.over) this.bailed = true; }
  choose(_itemId: number): Snapshot { if (!this.live) this.live = this.startRun(); const snap = snapshot(this.live, this.s.rules); snap.turn *= 10; return snap; }
  setVaultPref(pref: string): Lineage { if (["weapon", "armour", "potion", "scroll"].includes(pref)) this.s.lineage.vault_pref = pref; return this.lineage(); }
  insure(id: number): Lineage {
    const L = this.s.lineage; const it = L.vault.find((v) => v.id === id);
    if (!it || (L.insured ?? []).includes(id)) return this.lineage();
    const price = Math.ceil(salvageOf(it.kind) * 10 / 4); if (L.gold < price) return this.lineage();   // the camp's own price (ui/salvage.ts)
    this.gold(-price, `insure ${it.label}`); L.insured = [...(L.insured ?? []), id]; return this.lineage();
  }

  runOfflineQuick(elapsedS: number): ReturnReport { const r = this.runOffline(elapsedS); return { ...r, worst_death_id: r.worst_death?.run_id, worst_death: undefined }; }
  runOffline(elapsedS: number): ReturnReport {
    const L = this.s.lineage;
    let budget = Math.max(0, Math.floor(elapsedS)); let runs = 0, stall = 0, sampled = false, turnsTotal = 0;
    let rested = 0, banked = 0, returned = 0; const bonesFound: string[] = []; const exits: ExitLine[] = [];
    // Cut 2 §1: the rest (or wake) after each expedition comes out of the same clock; what is left waits in camp
    const rest = (run: Run): void => { if (run.line) { exits.push(run.line); while (exits.length > EXITS_CAP) exits.shift(); } const r = Math.min(run.rest_s, budget); rested += r; budget -= r; L.rest_left_s = run.rest_s - r; if (run.exit === "bank") banked++; else if (run.exit === "return") returned++; for (const b of run.bonesFound) bonesFound.push(`heir ${b.heir} · D${b.depth} · ${b.items} items`); };
    if ((L.rest_left_s ?? 0) > 0) { const r = Math.min(L.rest_left_s ?? 0, budget); rested += r; budget -= r; L.rest_left_s = (L.rest_left_s ?? 0) - r; }
    const learned: string[] = [], bests: string[] = [], found: InvItem[] = [], deaths: Record<string, number> = {}; let marks = 0; let reel: Highlight[] = [];
    const tamed: string[] = [], hatched: string[] = [], lost: string[] = []; let xpGained = 0, levelUps = 0, renownGained = 0, ranksUp = 0;
    const salvMap: Record<string, { n: number; gold: number }> = {};
    const take = (r: { tamed: string[]; lost: string[]; hatched: string[]; xp: number; levels: number[]; salvaged: { kind: string; n: number; gold: number }[]; score: number; ranks: number[] }): void => {
      tamed.push(...r.tamed); lost.push(...r.lost); hatched.push(...r.hatched); xpGained += r.xp; levelUps += r.levels.length; renownGained += r.score; ranksUp += r.ranks.length;
      for (const s of r.salvaged) { const m = (salvMap[s.kind] ??= { n: 0, gold: 0 }); m.n += s.n; m.gold += s.gold; }
    };
    if (this.pending) { const run = this.pending; this.pending = null; take(this.settle(run, true, this.autoKeep(run))); this.live = null; }
    let worst: { id: number; depth: number } | null = null;
    if (this.live && !this.live.over) { const ctx = this.ctx(); while (!this.live.over && budget > 0) { simTurn(this.live, ctx); budget--; this.s.totalTurns += 10; } if (this.live.over) { const r = this.settle(this.live, true, this.autoKeep(this.live)); this.settled.add(this.live.id); take(r); learned.push(...r.facts); bests.push(...r.bests); marks += r.marks; runs++; if (this.live.exit === "death") { deaths[this.live.cause ?? "?"] = 1; worst = { id: this.live.id, depth: this.live.depth }; } reel.push(...this.live.hl); rest(this.live); this.live = null; } }
    while (budget > 0 && stall < 20 && runs < 80 && !L.ended) {
      const run = this.startRun(); const ctx = this.ctx();
      while (!run.over && budget > 0) { simTurn(run, ctx); budget--; this.s.totalTurns += 10; }
      if (!run.over) { this.live = run; break; }
      runs++; turnsTotal += run.turn;
      const r = this.settle(run, true, this.autoKeep(run)); this.settled.add(run.id); take(r);
      learned.push(...r.facts); bests.push(...r.bests); marks += r.marks;
      stall = r.facts.length || r.bests.length ? 0 : stall + 1;
      this.home = run.exit === "death" || r.bests.some((b) => /^D\d+$/.test(b)) ? 0 : this.home + 1;
      for (const p of run.picked) if ((WEAPONS[p.kind] || ARMOUR[p.kind] !== undefined) && !found.some((f) => f.kind === p.kind)) found.push(p);
      if (run.exit === "death") { deaths[run.cause ?? "?"] = (deaths[run.cause ?? "?"] ?? 0) + 1; if (!worst || run.depth < worst.depth) worst = { id: run.id, depth: run.depth }; }
      reel.push(...run.hl);
      rest(run);
    }
    if (budget > 0 && stall >= 20 && turnsTotal > 0) { const extra = Math.floor(budget / (turnsTotal / runs)); if (extra > 0) { sampled = true; const scale = (runs + extra) / runs; for (const k of Object.keys(deaths)) deaths[k] = Math.round(deaths[k] * scale); runs += extra; } }
    reel = reel.sort((a, b) => b.score - a.score).slice(0, 5);
    const pending: string[] = [];
    for (const [u, c] of Object.entries(UNLOCK_COST)) if (!L.unlocks.includes(u) && L.marks >= c && this.unlockVisible(u)) pending.push(`unlock ${u} (${c})`);
    const worstDeath = worst ? this.death(worst.id) : undefined;
    if (worstDeath?.verdict === "gap") pending.push(`patch D${worstDeath.depth} ${worstDeath.cause}`);
    if (!pending.length) pending.push("rules");
    const verdictStall = this.stall(this.home);
    const restLeft = L.rest_left_s ?? 0;
    const live = this.send(); L.rest_left_s = restLeft;                    // `live` is a peek, not a send: the rest stands
    return { elapsed_s: elapsedS, runs, sampled, learned, bests, found, deaths: Object.entries(deaths).map(([cause, n]) => ({ cause, n })).sort((a, b) => b.n - a.n), pending, reel, marks_earned: marks, worst_death: worstDeath, live, tamed, hatched, lost, xp: { class: L.class, gained: xpGained, level_ups: levelUps },
      salvaged: Object.entries(salvMap).map(([kind, v]) => ({ kind, ...v })), renown: { gained: renownGained, rank: L.rank, ranks_up: ranksUp },
      rested_s: rested, banked, returned, bones_found: bonesFound, stall: verdictStall, exits };
  }
  /** Stall verdict (core README) so the report's section can be seen: a `return` / `bank` row that sent ≥ 4 runs home with no
   *  new depth is named; the candidates (row 10 points deeper as `replace`, the row as `remove`, `hp<90 → rest`) carry the
   *  fake's own 8-sim reach at best + 1. Not game truth: the fake keeps every candidate, the core keeps Δ > 0.02. */
  /** Runs since the last death or new depth: the stall window, on the engine like the core's so 30-minute slices add up. */
  private home = 0;
  private stall(home: number): Stall | undefined {
    const L = this.s.lineage; const rules = this.s.rules;
    const at = rules.rows.findIndex((r) => r.verb.v === "return" || r.verb.v === "bank");
    if (at < 0 || home < 4) return undefined;
    const row = rules.rows[at]; const depth = Math.max(1, L.best_depth); const known = this.known();
    const reach = (rs: RuleSet): number => { let ok = 0; for (let i = 0; i < 8; i++) if (this.simOne(hash(`stall:${L.seed}:${i}`), rs, known).depth > depth) ok++; return ok / 8; };
    const base = reach(rules);
    const deeper: Row = { conds: row.conds.map((c) => c.k === "hp<" && c.n ? { ...c, n: Math.max(5, c.n - 10) } : c.k === "depth>=" && c.n ? { ...c, n: c.n + 1 } : c.k === "loot>=" && c.n ? { ...c, n: c.n * 2 } : { ...c }), verb: { ...row.verb } };
    const cands: Patch[] = [
      { row: deeper, insert_at: at, survive: 0, forecast_delta: 0, replace: true },
      { row, insert_at: at, survive: 0, forecast_delta: 0, remove: true },
    ];
    if (!rules.rows.some((r) => r.verb.v === "rest")) cands.push({ row: { conds: [{ k: "hp<", n: 90 }], verb: { v: "rest" } }, insert_at: 0, survive: 0, forecast_delta: 0 });
    for (const p of cands) {
      const rows = rules.rows.map((r) => JSON.parse(JSON.stringify(r)) as Row);
      if (p.remove) rows.splice(p.insert_at, 1); else if (p.replace) rows[p.insert_at] = p.row; else { rows.splice(p.insert_at, 0, p.row); while (rows.filter((r) => r.verb.v !== "tactic").length > this.maxRows()) rows.splice(rows.map((r) => r.verb.v !== "tactic").lastIndexOf(true), 1); }   // Cut 12 §1: own rows over the cap drop from the end
      p.survive = reach({ rows }); p.forecast_delta = Math.round((p.survive - base) * 100) / 100;
    }
    cands.sort((a, b) => b.forecast_delta - a.forecast_delta);
    return { row: at, fired: home, text: `R${at + 1} ${row.verb.v} ended ${home} runs at D${depth}`, patches: cands.slice(0, 3) };
  }
  private unlockVisible(u: string): boolean {
    const L = this.s.lineage;
    const pre = UNLOCK_PREREQ[u]; if (pre && !L.unlocks.includes(pre)) return false;
    L.ledger = this.ledger();
    return UNLOCKS[u]?.gate?.(L) ?? true;
  }

  death(runId: number): Death {
    if (this.lastDeath[runId]) return this.lastDeath[runId];
    const log = this.s.logs[runId]; const L = this.s.lineage;
    if (!log) return { run_id: runId, depth: 0, cause: "unknown", margin: "?", verdict: "dice", baseline: 0, trace: { turns: [] }, patches: [], morgue: "" };
    const known = new Set(log.known);
    const replay = makeRun(runId, log.heir, log.seed, log.cls, log.trait, known, [], this.ctx(log.rules)); runToEnd(replay, this.ctx(log.rules));
    const N = 8;
    const survive = (rules: RuleSet): number => { let ok = 0; for (let i = 0; i < N; i++) { const r = this.simOne(hash(`p:${log.seed}:${i}`), rules, known, log.cls, log.trait); if (r.depth > log.depth || r.exit !== "death") ok++; } return ok / N; };
    const base = survive(log.rules);
    const vocab = this.vocabulary(); const hasTag = (t: string): boolean => vocab.conds.some((c) => c.k === "foe_tag" && c.t === t);
    const heal = vocab.verbs.some((v) => v.v === "drink" && v.a === "heal");
    const killer = replay.cause?.replace(/ pack$/, "").replace(/ /g, "_") ?? ""; const ktags = MON[killer]?.tags.filter(hasTag) ?? [];
    const cands: Row[] = [
      { conds: [{ k: "foes>=", n: 2 }, { k: "hp<", n: 50 }], verb: { v: "back_corridor" } },
      { conds: [{ k: "hp<", n: 40 }], verb: heal ? { v: "drink", a: "heal" } : { v: "drink", a: "unknown" } },
      { conds: [{ k: "hp<", n: 35 }], verb: { v: "return" } },
      { conds: [{ k: "adj>=", n: 2 }], verb: { v: "retreat" } },
      { conds: [{ k: "floor_seen>=", n: 40 }], verb: { v: "descend" } },
    ];
    for (const t of ktags) cands.unshift({ conds: [{ k: "foe_tag", t }], verb: { v: "retreat" } });
    // Cut 11 §2: the chain — the killing turn's `because`s, root (earliest) first; a theft root puts `foe: thief → attack thief`
    // on top (the thief-guard card when owned), a gas root `foe: gas → retreat`, each carrying the root it answers
    const lastRows = replay.trace[replay.trace.length - 1]?.rows ?? [];
    const chain = lastRows.flatMap((r) => (r.because ? [r.because] : [])).filter((b, i, a) => a.findIndex((x) => x.text === b.text && x.t === b.t) === i).sort((a, b) => a.t - b.t);
    const rootOf = (kind: Prov["kind"]): Prov | undefined => Object.values(replay.prov).filter((p) => p.kind === kind && chain.some((c) => c.text === p.text)).sort((a, b) => a.t - b.t)[0];
    const roots: { row: Row; root: string }[] = [];
    const theft = rootOf("theft"), gas = rootOf("gas");
    if (theft && hasTag("thief")) roots.push({ row: L.unlocks.includes("thief_guard") ? { conds: [], verb: { v: "tactic", a: "thief_guard" } } : { conds: [{ k: "foe_tag", t: "thief" }], verb: { v: "attack", a: "tag:thief" } }, root: theft.text });
    if (gas && hasTag("gas")) roots.push({ row: { conds: [{ k: "foe_tag", t: "gas" }], verb: { v: "retreat" } }, root: gas.text });
    const fresh = (c: Row): boolean => !log.rules.rows.some((r) => rowText(r) === rowText(c));   // a full set still gets patches (Cut 4 §1: overflow is the player's call)
    const score = (row: Row): Patch => { const rules: RuleSet = { rows: [row, ...log.rules.rows] }; const sv = survive(rules); return { row, insert_at: 0, survive: sv, forecast_delta: Math.round((sv - base) * 100) / 100 }; };
    const rootPatches: Patch[] = roots.filter((r) => fresh(r.row)).map((r) => ({ ...score(r.row), root: { text: r.root } }));
    const rootTexts = new Set(rootPatches.map((p) => rowText(p.row)));
    const scored = cands.filter((c) => fresh(c) && !rootTexts.has(rowText(c))).map(score).sort((a, b) => b.survive - a.survive).slice(0, 3);
    const verdict: "gap" | "dice" = [...rootPatches, ...scored].some((p) => p.survive >= 0.6) ? "gap" : "dice";
    // Cut 11 §4: a candidate under the bar is still named, dimmed (`survives 40% · below bar`); `dice` never shows an empty list
    let patches: Patch[] = [...rootPatches, ...scored].map((p) => (p.survive < 0.6 ? { ...p, below_bar: true } : p));
    // Cut 11 §2: a locked-condition root — the unlock as a pseudo-patch (`◆2 cond: alert`, insert_at −1): the client buys, then inserts the row
    if (!L.unlocks.includes("cond_alert") && replay.alert >= 1) { const row: Row = { conds: [{ k: "alert>=", n: 3 }], verb: { v: "return" } }; const sv = survive({ rows: [row, ...log.rules.rows] }); patches.push({ row, insert_at: -1, survive: sv, forecast_delta: Math.round((sv - base) * 100) / 100, root: { text: `◆${UNLOCK_COST.cond_alert} cond: alert` }, ...(sv < 0.6 ? { below_bar: true } : {}) }); }
    patches = patches.slice(0, 4);
    const margin = `${Math.max(1, log.hpMargin)} hp short`;
    const morgue = [`riddle · seed ${L.seed} · heir ${log.heir} · ${log.cls} · ${log.trait}`, `D${log.depth} · ${replay.cause ?? "?"} · ${margin} · ${verdict} · turn ${log.turns}`, "", ...log.rules.rows.map((r, i) => `R${i + 1} ${rowText(r)}`), "", ...replay.trace.map((t) => `t${t.t * 10} R${t.row + 1} ${verbText(t.verb)} hp${t.hp} foes${t.foes}${t.telegraphs.length ? " " + t.telegraphs.join(",") : ""}`), ...chain.map((c) => `← ${c.text} t${c.t * 10}`)].join("\n");
    const d: Death = { run_id: runId, depth: log.depth, cause: replay.cause ?? log.cause ?? "?", margin, verdict, baseline: base, trace: { turns: scaleTrace(replay.trace) }, patches, morgue, line: log.line ?? replay.line,
      chain: chain.length ? chain.map((c) => ({ text: c.text, t: c.t * 10, depth: c.depth })) : undefined };
    this.lastDeath[runId] = d; return d;
  }

  buy(unlock: string): Lineage {
    const L = this.s.lineage; const cost = UNLOCK_COST[unlock];
    if (cost === undefined) throw new Error("unknown unlock");
    if (L.unlocks.includes(unlock)) throw new Error("already owned");
    if (!this.unlockVisible(unlock)) throw new Error("prerequisite missing");
    if (L.marks < cost) throw new Error("not enough marks");
    L.marks -= cost; L.unlocks.push(unlock); if (unlock === "party_slot_2") L.party_slots = 2; if (unlock === "party_slot_3") L.party_slots = 3;
    return this.lineage();
  }
  unlockDeltas(): UnlockInfo[] { return this.unlocks(); }
  unlocks(): UnlockInfo[] {
    const L = this.s.lineage;
    L.ledger = this.ledger();
    // Cut 9 §2: every card that is not `available` says why — the gate, the missing prerequisite, or `◆2 more` (the core's `needs`)
    return Object.entries(UNLOCKS).map(([id, u]) => {
      const owned = L.unlocks.includes(id); const met = u.gate?.(L) ?? true;
      const needs = owned ? undefined : !met ? u.needs : !this.unlockVisible(id) ? UNLOCK_PREREQ[id]?.replace(/_/g, " ") : L.marks < u.cost ? `◆${u.cost - L.marks} more` : undefined;
      return { id, cost: u.cost, owned, available: !owned && needs === undefined, needs,
      delta: TACTIC_CARDS.includes(id) && !owned ? ((Math.abs(hash(id)) % 9) - 2) / 100 : undefined,   // delta: Cut 4 §9 stand-in (`reach +4%` on a card)
      rows: UNLOCK_ROWS[id],                                                                             // Cut 6 §6
      ...(TACTIC_CARDS.includes(id) ? { insert_at: this.cardInsertAt() } : {}) };                        // Cut 12 §1
    });
  }
  setClass(cls: string): Lineage {
    if (!(CLASSES as readonly string[]).includes(cls)) throw new Error("unknown class");
    if (!isFreeClass(cls) && !this.s.lineage.unlocks.includes(cls)) throw new Error(`${cls} not unlocked`);
    this.s.lineage.class = cls; this.dropIdleRun(); return this.lineage();
  }
  selectSet(i: number): Lineage {
    const L = this.s.lineage; L.active_set = Math.max(0, Math.min(L.sets.length - 1, i));
    this.s.rules = JSON.parse(JSON.stringify(L.sets[L.active_set])) as RuleSet; return this.lineage();
  }
  /** Cut 3: a new lineage under `variant`, keeping classes, kennel (party goes home), vault, facts, forge, trophies, rules.
   *  The core insists on the ending first; the UI fake does not, so the ending screen can be exercised from any state. */
  ascend(variant: string): Lineage {
    if (!["no_rest", "short_list", "bones_only", "hunted"].includes(variant)) throw new Error(`unknown variant ${variant}`);
    const old = this.s.lineage; const level = (old.ascension?.level ?? 0) + 1;
    const keep = { classes: old.classes, kennel: [...old.party, ...old.kennel], vault: old.vault, facts: old.facts, forge: old.forge, trophies: old.trophies, sets: old.sets, active_set: old.active_set, ledger: old.ledger, class: old.class };
    this.newLineage(old.seed + level);
    Object.assign(this.s.lineage, keep, { party: [], unlocks: old.unlocks.filter((u) => ["rogue", "ranger", "caster"].includes(u)), ascension: { level, variant } });
    return this.selectSet(old.active_set);
  }
  exportRules(): string { return this.s.rules.rows.map(rowText).join("\n"); }
  importRules(text: string): RuleSet { const set = parseRules(text); this.setRules(set); return JSON.parse(JSON.stringify(this.s.rules)) as RuleSet; }
}

export function createFakeEngine(): Engine { return new FakeEngine(); }
