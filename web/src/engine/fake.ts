// Fake Engine: a tiny deterministic mini-sim with canned-shaped output so the UI can be built and
// exercised before the Rust core lands. Not game truth. Selected with ?engine=fake or when pkg/ is absent.
import type {
  BonesPile, CageOption, Divergence, DivergenceBranch, DivergenceEnd, FoldBeat, FoldFloor, FoldLine, RowFires, StartOption, ForkOption, Combo, Companion, Cond, Counter, Death, Engine, Entity, Ev, ExitLine, FloorItem, Forecast, ForecastVs, VsMove, Highlight, InvItem, LedgerRow, Lineage, Overlay,
  Patch, ReturnReport, Row, RuleSet, Snapshot, StepResult, Stall, SupplyEntry, Tile, Trace, UnlockInfo, Verb, Vocabulary, Because, KitLadder, RowWhy,
  Oath, OathReward, OathShare, ForecastMove, MovePart, ReportLead, MeterWire, SystemInfo, StandingOrders, WallEdit,
  Packages, Package, PkgOption, Town, Track, GrewLine, Quest, RowOrigin,
  WorkNode, Works, NextPill, WorkerPost, WorkerAct, Rarity,
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
// Cut 16 §3: the fake's bands (warrens D1–3, the Burrows D4–5, the Fens D6–10, the Crypt D11–15) — Cut 26: `FAKE_BANDS` and the route (`biomeOn`)

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
// Cut 26 §1 stand-in: the fake's descent forks at the stairs into D4 (the Burrows, or the Fens early) and into D6 (the Fens, or the
// Crypt early); a far stair swaps the two bands' biomes (the deferred one is taken next, no fork there). Bosses go with their biome,
// on the band's last floor. A route is a set's `route` (fork depths taking the far stair). `?fake_fork=1` learns the D4 fork at boot.
const FAKE_BANDS: [number, number][] = [[4, 5], [6, 10], [11, 15]];
const FAKE_ORDER = ["burrows", "fens", "crypt"];
export const FAKE_FORKS = [4, 6];
const BIOME_BOSS: Record<string, string> = { burrows: "goblin_warlord", fens: "bloat_mother", crypt: "lich" };
const DEV_FORK = typeof location !== "undefined" && ["1", "2"].includes(new URLSearchParams(location.search).get("fake_fork") ?? "");
/** The core ships the first fork only (`descent::OPEN_FORKS`, Cut 26 §3's fallback); `?fake_fork=2` opens the second too, for the chip line's UI. */
const FAKE_OPEN = typeof location !== "undefined" && new URLSearchParams(location.search).get("fake_fork") === "2" ? [4, 6] : [4];
function routeOrder(route: number[] = []): string[] {
  const o = [...FAKE_ORDER];
  FAKE_FORKS.forEach((f, i) => { if (route.includes(f)) [o[i], o[i + 1]] = [o[i + 1], o[i]]; });
  return o;
}
function biomeOn(d: number, route?: number[]): string {
  if (d < FAKE_BANDS[0][0]) return "warrens";
  const i = FAKE_BANDS.findIndex(([a, b]) => d >= a && d <= b);
  return routeOrder(route)[i < 0 ? 2 : i];
}
function bossOn(d: number, route?: number[]): string | undefined {
  const i = FAKE_BANDS.findIndex(([, b]) => b === d);
  return i < 0 ? undefined : BIOME_BOSS[routeOrder(route)[i]];
}
const forkOpen = (f: number, route: number[] = []): boolean => { const i = FAKE_FORKS.indexOf(f); return i === 0 || (i > 0 && !route.includes(FAKE_FORKS[i - 1])); };
/** The route with the stair at `f` set (a far stair drops the neighbouring far stairs it overlaps), as the core's `Route::with`. */
function routeWith(route: number[] = [], f: number, far: boolean): number[] {
  const i = FAKE_FORKS.indexOf(f);
  const out = route.filter((x) => x !== f && (!far || (x !== FAKE_FORKS[i - 1] && x !== FAKE_FORKS[i + 1])));
  if (far) out.push(f);
  return out.sort((a, b) => a - b);
}
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
  vault2: { cost: 3 }, vault3: { cost: 6 }, vault4: { cost: 9 },
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
/** Cut 18 §5: the foe tag a tactic card answers (the core's `meta::card_situation`). */
const CARD_SITUATION: Record<string, string> = { kite_archers: "ranged", gas_step: "gas", pack_break: "pack", thief_guard: "thief", boss_focus: "boss" };
const TACTIC_CARDS = ["corridor_fighting", "kite_archers", "stair_dance", "gas_step", "pack_break", "thief_guard", "boss_focus", "last_stand"];
const COND_UNLOCK: Record<string, string> = { "alert>=": "cond_alert", "turns>": "cond_turns", "loot>=": "cond_loot", on_kill: "cond_on_kill", on_see: "cond_on_see", "party_hp<": "cond_party_hp" };
const REST_CAP_S = 30 * 60, WAKE_S = 20 * 60, BONES_MAX = 3, STUDIED_KILLS = 5, GOLD_LEDGER_CAP = 20, EXITS_CAP = 5, TRACE_TURNS = 10;
/** QA on 778fa1b (qaV): a waystone start is free — the core's `WAYSTONE_TOLL` is 0 (the field and the pass stay). */
const WAYSTONE_TOLL = 0;
// UI dev knob: `?engine=fake&fake_depth=5` starts every run on D5 (boss floor) so the boss HUD can be seen.
const DEV_START_DEPTH = Math.max(1, (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_depth"))) || 1);
// UI/render dev knob: `?engine=fake&fake_god=1` — the hero never falls below 1 hp (tools/gfx-eval.mjs films a boss's entrance, break and fall).
const DEV_GOD = typeof location !== "undefined" && new URLSearchParams(location.search).get("fake_god") === "1";
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
  const rows: Row[] = []; const route: number[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line || line.startsWith("#")) continue;
    if (line.startsWith("⑂")) { route.push(...line.slice(1).trim().split(/\s+/).map(Number).filter((n) => n > 0)); continue; }   // Cut 26 §2
    const [lhs, rhs] = line.split("→").map((s) => s.trim());
    if (rhs === undefined) continue;
    const conds = lhs ? lhs.split("·").map((s) => parseCond(s.trim())).filter((c): c is Cond => !!c) : [];
    const [v, a] = rhs.split(/\s+/);
    rows.push({ conds: conds.slice(0, 2), verb: a ? { v, a } : { v } });
  }
  return route.length ? { rows, route } : { rows };
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
  gold: number; spent: { label: string; price: number; kind: string }[]; line?: ExitLine;    // Cut 6 §1: the ledger line at the exit
  lastRows?: { row: number; why: string }[];                                                  // Cut 6 §3: row accounting of the last action
  prov: Record<string, Prov>;                                                                 // Cut 11 §1: per slot (`item:heal`, `path`), the last event that emptied / blocked it
  startedTotal: number;                                                                       // Cut 11 §5: the lineage tick this run started at (`Snapshot.run.started_turn`)
  twist?: string;                                                                             // Cut 12 §4: the floor's one situation from D3 (`nest`), never the previous floor's kind
  stuckFires: number; lastSig: string; stalled: boolean;                                      // Cut 13 §1: the guard — a row (or a frozen chore) that changed nothing, counted; at STALL_FIRES the run ends stalled
  noted: Set<string>; notes: string[];                                                        // Cut 13 §4: situations noted on this floor (`A den. Something sleeps.`); the run's last two notes (`Death.notes`)
  route: number[]; forkSeen: number;                                                          // Cut 26: the set's route at the send; the deepest fork whose two stairs this run saw
};
/** Cut 13 §1: the fake's guard — this many fires with nothing changed end the run as a stall (the core's `STALL_FIRES`). */
const STALL_FIRES = 30;
/** Cut 27 §1: the core's `forecast::FOLD_CLEAR` — a floor the set clears this often folds on the watch. */
const FOLD_CLEAR = 0.95;
// Cut 27 §1 dev knob: `?engine=fake&fake_fold=N` — the forecast folds D1–N whatever the sims say (a fold on a fresh lineage, for UI work)
const DEV_FOLD = (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_fold"))) || 0;
/** Cut 27 §1 stand-in: the core's `fold::beat` — an event that changes what the run holds, as a chip beat (≤ 3 words). */
function fakeBeat(e: Ev, depth: number): FoldBeat[] {
  const b = (kind: string, text: string): FoldBeat[] => [{ depth, t: e.t, kind, text: text.split(/\s+/).slice(0, 3).join(" ") }];
  switch (e.k) {
    case "steal": return b("theft", `stolen ${e.item}`);
    case "pickup": return e.item.startsWith("gold") ? b("gold", e.item.replace(/^gold /, "")) : b("find", `found ${e.item}`);
    case "use": return b("use", `used ${e.item}`);
    case "fact": return b("fact", e.fact.split(/[:=]/).slice(1).join(" ").replace(/_/g, " ") || e.fact);
    case "max_hp": return b("max_hp", `max ${e.delta > 0 ? "+" : ""}${e.delta}`);
    case "bones": return b("bones", `bones · ${e.items}`);
    case "level": return b("level", `level ${e.level}`);
    case "tame": return e.ok ? b("pet", `tamed ${e.kind.replace(/_/g, " ")}`) : [];
    case "ally": return b("pet", `pet ${e.state}`);
    case "callout": return e.text === "boss down" ? b("boss", e.text) : e.text.startsWith("passage ") ? b("passage", e.text) : [];
    default: return [];
  }
}
/** Cut 27 §1 stand-in: the floor's lowest hero hp at or under 30 % of max, as a `dip` beat (the core's `fold::DIP_SHARE`). */
function fakeDip(events: Ev[], max: number): { t: number; kind: string; text: string } | null {
  let low: { hp: number; t: number } | null = null;
  for (const e of events) if (e.k === "hurt" && e.id === 0 && e.hp <= 0.3 * max && (!low || e.hp < low.hp)) low = { hp: e.hp, t: e.t };
  return low ? { t: low.t, kind: "dip", text: `hp ${low.hp}/${max}` } : null;
}
/** Cut 27 §1 stand-in: the core's `fold::chips` — one chip per kind, the beat alone or a count; gold is the line's `+$`. */
function fakeChips(beats: FoldBeat[]): string[] {
  const ORDER: [string, string][] = [["theft", "thefts"], ["dip", ""], ["boss", "bosses down"], ["pet", "pet beats"], ["find", "finds"], ["use", "used"], ["max_hp", ""], ["fact", "learned"], ["bones", "bones"], ["level", ""], ["hatch", "hatched"]];
  const out: string[] = [];
  for (const [k, many] of ORDER) {
    const of = beats.filter((b) => b.kind === k); if (!of.length) continue;
    if (k === "dip") out.push(of.map((b) => b.text).sort((x, y) => Number(x.split(/[ /]/)[1]) - Number(y.split(/[ /]/)[1]))[0]);
    else if (k === "max_hp") { const n = of.reduce((s, b) => s + Number(b.text.replace(/^max /, "")), 0); out.push(`max ${n > 0 ? "+" : ""}${n}`); }
    else if (k === "level") out.push(of[of.length - 1].text);
    else out.push(of.length === 1 ? of[0].text : `${of.length} ${many}`);
  }
  return out;
}
/** Cut 27 §2 stand-in: the rows whose fires moved between the sent set and the new (the fake counts none: each row present in only one
 *  set, at a nominal rate). */
function fakeFires(prev: RuleSet, rules: RuleSet, n: number): RowFires[] {
  const key = (r: Row): string => JSON.stringify([r.conds, r.verb]);
  const out: RowFires[] = [];
  rules.rows.forEach((r, j) => { if (!prev.rows.some((p) => key(p) === key(r))) out.push({ new_row: j, text: `R${j + 1} ${r.verb.v.replace(/_/g, " ")}`, sent: 0, new: Math.max(1, Math.round(n / 5)) }); });
  prev.rows.forEach((p, k) => { if (!rules.rows.some((r) => key(r) === key(p))) out.push({ sent_row: k, text: `R${k + 1} ${p.verb.v.replace(/_/g, " ")}`, sent: Math.max(1, Math.round(n / 5)), new: 0 }); });
  return out.slice(0, 4);
}
// UI dev knob: `?engine=fake&fake_stall=N` freezes the hero's chores after N turns on a floor (the chore loop the core's guard
// catches), so a run stalls: `keeps $0 · stalling` on the HUD, then the stall verdict screen.
const DEV_STALL = (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_stall"))) || 0;
// Cut 24 §1 dev knob: `?engine=fake&fake_shrug=N` — the run's first N actions with a foe beside the hero shrug every blow (hits for 0 both
// ways, the guard asleep): a fight that cannot progress, for the watch's dead-stretch gate (fights.mjs)
const DEV_SHRUG = (typeof location !== "undefined" && Number(new URLSearchParams(location.search).get("fake_shrug"))) || 0;
let shrugNow = false;
// Cut 25 §3 dev knob: `?engine=fake&fake_drain=1` — from a floor's 20th tick the hero starves every 10 ticks a 0-damage `hurt` (`hunger`, as the core's unlit hunger floor bites), max hp −1 (to 6) and its `hunger −1 max` callout: a drain stretch for the
// watch's gate (fights.mjs / cut25.mjs)
const DEV_DRAIN = typeof location !== "undefined" && new URLSearchParams(location.search).get("fake_drain") === "1";
const shrugRuns = new WeakMap<object, number>();
/** Cut 13 §4: the situation notes the core writes on first sight (verbatim; the client cuts the fight frame in on them). */
// the three-item room is the `vault` inside and the *cage* to the player (core situations.rs `twist_word`; QA on 50bb162)
const PROP_NOTE: Record<string, string> = { shrine: "A shrine. Pray, at a price.", vault: "A cage: three inside, one to take.", nest: "A den. Something sleeps." };
const twistWord = (t: string): string => (t === "vault" ? "cage" : t);
/** Cut 12 §4: the situation kinds a floor rolls one of from D3 (the core's bands; the fake draws from the whole list). */
const TWISTS = ["den", "lock", "captive", "nest", "shrine", "vault", "stray", "hunger"];
/** Cut 11 §1: a provenance entry — the wire's `because` (turn units here, ×10 on the wire) and what kind of event it was. */
type Prov = Because & { kind: "theft" | "use" | "gas" };
type RunLog = { seed: number; rules: RuleSet; depth: number; cause?: string; turns: number; exit: string; known: string[]; cls: string; trait: string; heir: number; hpMargin: number; line?: ExitLine;
                stalled?: boolean; notes?: string[] };   // Cut 13 §1: a stalled run keeps a verdict like a death's; §4: its last two notes

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

function genFloor(rng: Rng, depth: number, flav: (kind: string) => string, known: Set<string>, nextId: () => number, route: number[] = []): Floor {
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
  const biome = biomeOn(depth, route);
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
  const kinds0 = Object.keys(MON).filter((k) => !MON[k].tags.includes("boss") && k !== "blade" && depth >= MON[k].lo && depth <= MON[k].hi);
  // (past the bestiary's deepest band — reachable under `?fake_god=1` — the floor draws from every kind)
  const kinds = kinds0.length ? kinds0 : Object.keys(MON).filter((k) => !MON[k].tags.includes("boss") && k !== "blade");
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
  { const boss = bossOn(depth, route); if (boss) { const [x, y] = spot(last); mons.push(mk(boss, x, y)); } }
  const seen = new Array(W * H).fill(false), visible = new Array(W * H).fill(false);
  return { tiles, seen, visible, overlays: [], rooms, items, mons, roomOf };
}

// --- sim ---
type SimCtx = { rules: RuleSet; unlocks: Set<string>; flav: (k: string) => string; kindOfFlav: (f: string) => string | undefined; tier: (kind: string) => number;
  bones: BonesPile[]; insured: Set<number>; gold: number; spent: { label: string; price: number; kind: string }[] };

/** Cut 21 §1 stand-in: a waystone at each biome's first floor past the Warrens (lit by a bank at or past it). */
const WAYSTONES = [5, 9, 14, 19, 24, 29];
function makeRun(id: number, heir: number, seed: number, cls: string, trait: string, known: Set<string>, brought: InvItem[], ctx: SimCtx, party: Companion[] = [], level = 1, start = 1): Run {
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
    stuckFires: 0, lastSig: "", stalled: false, noted: new Set(), notes: [],
    route: [...(ctx.rules.route ?? [])], forkSeen: 0,
  };
  run.nextId = 1000;
  run.recalled = run.party.map((c) => companionMon(run, c));
  descendTo(run, Math.max(DEV_START_DEPTH, start), ctx, []);
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
  run.stuckFires = 0; run.noted = new Set();   // Cut 13 §1: a new floor is progress; §4: its situations are noted afresh
  // Cut 12 §4: from D3 every floor rolls one situation, never the previous floor's kind
  run.twist = depth >= 3 ? pick(run.rng, TWISTS.filter((t) => t !== run.twist)) : undefined;
  if (run.floor) for (const m of run.floor.mons) if (m.ally && m.cid !== undefined) run.recalled.push(m);
  run.floor = genFloor(run.rng, depth, ctx.flav, run.known, () => run.nextId++, run.route);
  const up = run.floor.tiles.indexOf("stairs_up");
  run.hero.x = up % W; run.hero.y = Math.floor(up / W);
  placeAllies(run);
  // Cut 2 §2: a dead heir's kit lies where it fell; a later heir that steps on it recovers it
  for (const b of ctx.bones) if (b.depth === depth && !run.bonesFound.includes(b)) {
    const room = run.floor.rooms[run.floor.rooms.length - 1]; const x = room.x + Math.floor(room.w / 2), y = room.y + Math.floor(room.h / 2);
    run.floor.items.push({ id: run.nextId++, x, y, kind: "bones", known: true, label: `bones ♟${b.heir}` });
  }
  const biome = biomeOn(depth, run.route);
  ev.push({ t: run.turn, k: "descend", depth, biome });
  ev.push({ t: run.turn, k: "callout", text: `D${depth} ${biome}` });
  const bf = `biome:${biome}`;
  if (!run.known.has(bf)) { run.known.add(bf); run.facts.push(bf); ev.push({ t: run.turn, k: "fact", fact: bf }); }
  updateVis(run);
  forkSeen(run, ev);
}
/** Cut 26 §2 stand-in (the core's `facts::fork_seen`): the hero sees a fork's down stairs — the `fork:<d>` fact once, `TWO STAIRS` once a run. */
function forkSeen(run: Run, ev: Ev[]): void {
  const next = run.depth + 1;
  if (run.forkSeen >= next || !FAKE_OPEN.includes(next) || !forkOpen(next, run.route)) return;
  const s = run.floor.tiles.indexOf("stairs_down");
  if (s < 0 || !run.floor.visible[s]) return;
  run.forkSeen = next;
  const f = `fork:${next}`;   // the fact, silently (the core's `fact_note` has no line for it); the callout is the beat
  if (!run.known.has(f)) { run.known.add(f); run.facts.push(f); ev.push({ t: run.turn, k: "fact", fact: f }); }
  ev.push({ t: run.turn, k: "callout", text: "TWO STAIRS" });
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
function moveHero(run: Run, x: number, y: number, ev: Ev[]): void { run.hero.x = x; run.hero.y = y; ev.push({ t: run.turn, k: "move", id: 0, x, y }); updateVis(run); bossInView(run, ev); forkSeen(run, ev); }
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
  if (shrugNow) { rng(); return { hit: true, dmg: 0 }; }
  const hit = rng() < 0.8; return { hit, dmg: hit ? Math.max(0, ri(rng, atk[0], atk[1]) - def) : 0 };
}
function nameOf(m: Mon): string { return m.name ?? m.kind.replace(/_/g, " "); }
function hurtHero(run: Run, dmg: number, cause: string, ev: Ev[]): void {
  const before = run.hero.hp; run.hero.hp = Math.max(DEV_GOD ? 1 : 0, run.hero.hp - dmg); run.hurt = true;
  run.lastHurt = { cause, dmg, hpBefore: before };
  ev.push({ t: run.turn, k: "hurt", id: 0, dmg, hp: run.hero.hp, cause });
  if (run.hero.hp > 0 && run.hero.hp <= run.hero.max_hp * 0.1 && !run.nearDeath) { run.nearDeath = true; ev.push({ t: run.turn, k: "callout", text: `hp ${run.hero.hp}` }); }
}
function killMon(run: Run, m: Mon, cause: string, ev: Ev[]): void {
  run.floor.mons = run.floor.mons.filter((x) => x !== m);
  ev.push({ t: run.turn, k: "die", id: m.id, cause });
  run.killed = true; run.kills[m.kind] = (run.kills[m.kind] ?? 0) + 1; run.killXp += 1 + run.depth / 5; run.score += m.tags.includes("boss") ? 25 : Math.ceil(m.max_hp / 4);
  if (m.tags.includes("gas")) { for (const [dx, dy] of [[0, 0], ...DIRS]) { const x = m.x + dx, y = m.y + dy; if (inb(x, y) && passable(run.floor.tiles[idx(x, y)])) { run.floor.overlays.push({ x, y, k: "gas", ttl: 4 }); ev.push({ t: run.turn, k: "overlay", x, y, ov: "gas", ttl: 4 }); } } ev.push({ t: run.turn, k: "callout", text: "bloat pops", why: "gas burst" }); learn(run, `foe:${m.kind}:gas`, ev); gasNear(run, m); }
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
    case "in": return biomeOn(run.depth, run.route) === c.t;   // Cut 26 §2
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
  if (r.hit) {
    // a boss's break at half hp (the core's callout and note), then the kill (the fake sends no foe `hurt`: the watch's pacing gates are
    // calibrated to that)
    const was = m.hp; m.hp -= dmg;
    if (m.tags.includes("boss") && was > m.max_hp / 2 && m.hp <= m.max_hp / 2 && m.hp > 0) { const w = m.kind.split("_").pop(); ev.push({ t: run.turn, k: "callout", text: `${w} breaks` }, { t: run.turn, k: "note", text: `${w} breaks` }); }
    if (m.hp <= 0) killMon(run, m, verb, ev);
  }
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
/** Run-clear stand-in for the core's `item::rarity`: a gear kind's depth band + its `+N`; consumables by worth and depth; an unknown
 *  flavour common. (The fake's own kinds; the core's table is the truth.) */
const RARITY_ORDER: Rarity[] = ["common", "uncommon", "rare", "epic", "legendary"];
const GEAR_BAND: Record<string, number> = { dagger: 0, sword: 0, leather: 0, axe: 1, bow: 1, mail: 1, plate: 2, spear: 2, mace: 3, scale: 3 };
const RARE_KINDS = ["strength", "enchant", "recall", "mirror_shard"], UNCOMMON_KINDS = ["poison", "fire", "summon_ally", "regen", "resist_fire", "clarity", "silence", "earthquake", "mirror", "lantern", "bell", "salt", "chalk"];
export function fakeRarity(it: Pick<InvItem, "kind" | "label" | "known">): Rarity {
  if (it.kind in GEAR_BAND) { const p = GEAR_BAND[it.kind] + Number(/\+(\d+)/.exec(it.label)?.[1] ?? 0); return p <= 0 ? "common" : p <= 2 ? "uncommon" : p <= 4 ? "rare" : p <= 7 ? "epic" : "legendary"; }
  if (!it.known) return "common";
  return RARE_KINDS.includes(it.kind) ? "rare" : UNCOMMON_KINDS.includes(it.kind) ? "uncommon" : "common";
}
const withRarity = <T extends InvItem>(it: T): T => { const r = fakeRarity(it); const o = { ...it }; if (r === "common") delete o.rarity; else o.rarity = r; return o; };
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
  // Cut 13 §1: a stalled run comes home as a return that keeps nothing (`returned $0 · $84 carried · keeps 0% · stalled`)
  const keep_pct = run.stalled ? 0 : tier === "bank" ? 100 : tier === "return" ? 60 : 0;
  run.loot_kept = Math.round(run.loot * keep_pct / 100);   // Cut 2 §2: death 0%
  // Cut 6 §1: one arithmetic line the player can check (supplies were paid in camp; a death names its bones instead)
  const spent = run.spent.reduce((n, x) => n + x.price, 0);
  // Cut 10 §3 (as the core): the line leads with the verb and what came home
  const parts = [`${tier === "bank" ? "banked" : tier === "return" ? "returned" : "died"} $${run.loot_kept}`, `$${run.loot} carried`, `keeps ${keep_pct}%`];
  if (run.stalled) parts.push("stalled");
  if (tier === "death") { const kit = bonesKit(run).length; if (kit) parts.push(`bones: ${kit} item${kit === 1 ? "" : "s"} on D${run.depth}`); }
  else if (spent) parts.push(`supplies −$${spent}`);
  // Cut 9 §5: every exit carries its last-5 trace (row accounting included), on the event and on the ledger line
  // Cut 11 §3: every exit trace carries the run's provenance (the `because` events) beside its turns; ticks ×10 as on the events
  const trace: Trace = { turns: scaleTrace(run.trace), provenance: Object.values(run.prov).sort((a, b) => a.t - b.t).map((p) => ({ text: p.text, t: p.t * 10, depth: p.depth })) };
  run.line = { carried: run.loot, keep_pct, kept: run.loot_kept, spent, spent_on: run.spent.map((x) => x.label), text: parts.join(" · "), trace,
    // c30-legible stand-in: the core's `reason` (`engine::exit_reason`), in miniature
    reason: run.stalled ? "stuck · gave up" : tier === "death" ? "slain" : tier === "bank" ? "hurt · banked" : "hurt · went home",   // (Cut 30.5: a record is a beat, never an end)
    news: [{ k: "differ", text: `reached D${run.depth}` }] };   // Cut 24 §2 stand-in: the core's `news` (what was new; else the one thing that differed)
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
      // the core's order (`R2 attack`: the client names the rule — `attack attack nearest` read the id after the verb)
      ev.push({ t: run.turn, k: "callout", text: hpc ? `hp ${Math.round((h.hp / h.max_hp) * 100)}% → R${i + 1} ${r.verb.v.replace("_", " ")}` : `R${i + 1} ${r.verb.v.replace("_", " ")}` });
      done = true; break;
    }
    rows.push({ row: i, why: verbWhy(r.verb), because: becauseOf(run, null, r.verb) });   // Cut 11 §1
    if (!tr.blocked) tr.blocked = `R${i + 1} ${r.verb.v.replace(/_/g, " ")} ✗ ${verbWhy(r.verb)}`;
  }
  if (rows.length) tr.rows = rows;
  if (!done && run.trait === "curious" && !foes.length) { const u = h.inv.find((i) => !i.known); if (u) { useItem(run, u, ctx, ev); fire(-1, { v: POTIONS.includes(u.kind) ? "drink" : "read", a: "unknown" }, "curious: tries unknown"); ev.push({ t: run.turn, k: "callout", text: "curious: tries it" }); done = true; } }
  if (!done && run.trait === "greedy" && foes.length) { const it = run.floor.items.find((i) => cheb(i.x, i.y, h.x, h.y) <= 1); if (it) { if (it.x === h.x && it.y === h.y) pickUp(run, it, ev); else moveHero(run, it.x, it.y, ev); fire(-1, { v: "pick_up" }, "greedy: takes it"); ev.push({ t: run.turn, k: "callout", text: "greedy: grabs it" }); done = true; } }
  let frozen = false;
  if (!done && DEV_STALL && run.floorTurn > DEV_STALL) { frozen = true; tr.blocked ??= "chore pick up ✗ no path"; }   // the dev knob: the chore loop
  if (!done && !frozen) {
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
  // Cut 13 §1: the guard — a fired row (or a frozen chore) that left the world as it was counts; anything that moved resets it
  const sig = `${run.depth}|${h.x},${h.y}|${h.hp}|${run.loot}|${h.inv.length}|${run.floor.mons.reduce((a, m) => a + m.hp, 0)}|${run.floor.items.length}`;
  if (!run.over && (tr.row >= 0 || frozen) && sig === run.lastSig && !shrugNow) run.stuckFires++; else run.stuckFires = 0;
  run.lastSig = sig;
  if (!run.over && run.stuckFires >= STALL_FIRES) {
    run.stalled = true;
    ev.push({ t: run.turn, k: "note", text: "Stalled. Came home empty-handed." }); ev.push({ t: run.turn, k: "callout", text: "stalled" });
    endRun(run, "return", ev);
  }
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
  const ev = simTurn_(run, ctx);
  for (const e of ev) if (e.k === "note") { run.notes.push(e.text); while (run.notes.length > 2) run.notes.shift(); }   // Cut 13 §4: `Death.notes`, whatever ended the turn
  return ev;
}
function simTurn_(run: Run, ctx: SimCtx): Ev[] {
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
  if (DEV_DRAIN && run.floorTurn >= 20 && run.floorTurn % 10 === 0) {
    if (run.floorTurn === 20) ev.push({ t: run.turn, k: "drain", cause: "starving" });   // the core's word opens the stretch
    const bite = run.hero.max_hp > 6;   // the max stops at 6; the bites go on (0-damage `hurt`s, as the core's)
    if (bite) { run.hero.max_hp -= 1; run.hero.hp = Math.min(run.hero.hp, run.hero.max_hp); ev.push({ t: run.turn, k: "max_hp", id: 0, max: run.hero.max_hp, delta: -1, cause: "hunger" }); }
    ev.push({ t: run.turn, k: "hurt", id: 0, dmg: 0, hp: run.hero.hp, cause: "hunger" });
    if (bite) ev.push({ t: run.turn, k: "callout", text: "hunger −1 max" });
  }
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
  if (DEV_SHRUG) { const n = shrugRuns.get(run) ?? 0; shrugNow = n < DEV_SHRUG && run.floor.mons.some((m) => !m.ally && cheb(m.x, m.y, run.hero.x, run.hero.y) <= 1); if (shrugNow) shrugRuns.set(run, n + 1); }
  heroTurn(run, ctx, ev);
  run.hurt = false; run.killed = false;
  if (run.over) return ev;
  const speedy = run.hero.speedT > 0; if (speedy) run.hero.speedT--;
  if (!speedy || run.turn % 2 === 0) for (const m of [...run.floor.mons]) { if (run.floor.mons.includes(m)) monsterTurn(run, m, ev); if (run.over) return ev; }
  if (run.hero.invis > 0) run.hero.invis--; if (run.hero.bashCd > 0) run.hero.bashCd--; if (run.hero.vanishCd > 0) run.hero.vanishCd--; if (run.cleaveCd > 0) run.cleaveCd--;
  if (run.bulwark > 0) { run.bulwark--; if (run.bulwark === 0) run.hero.def -= 3; }
  bossInView(run, ev);                      // Cut 6 §5: a boss that walked into view this turn
  // Cut 13 §4: a situation in view is noted once per floor, the core's line verbatim (the client cuts the fight frame in on it)
  if (!run.over) for (const [k, text] of Object.entries(PROP_NOTE)) { if (run.noted.has(k)) continue; const i = run.floor.tiles.findIndex((t, j) => t === k && run.floor.visible[j]); if (i >= 0) { run.noted.add(k); ev.push({ t: run.turn, k: "note", text }); } }
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
    depth: run.depth, biome: biomeOn(run.depth, run.route), w: W, h: H, tiles: run.floor.tiles.slice(), seen: run.floor.seen.slice(), visible: run.floor.visible.slice(),
    overlays: run.floor.overlays.map((o) => ({ ...o })),
    hero: { id: 0, kind: `hero_${run.cls}`, x: h.x, y: h.y, hp: h.hp, max_hp: h.max_hp, tags: h.invis ? ["invisible"] : [], inv: h.inv.map((i) => ({ ...i })), weapon: h.weapon, armour: h.armour, class: run.cls, trait: run.trait },
    // Cut 4 §3 (UI dev stand-in for the core): a hostile seen before and out of sight now is `remembered`
    entities: run.floor.mons.map((m) => { const vis = run.floor.visible[idx(m.x, m.y)]; if (vis) m.seen = true; const e = ent(m); if (!vis && m.seen && !m.ally) e.remembered = true; return e; }),
    items: run.floor.items.map((i) => ({ ...i })), alert: run.alert, turn: run.turn, loot: run.loot,
    run: { id: run.id, heir: run.heir, started_turn: run.startedTotal },   // Cut 11 §5: the lineage tick this run started at
    stake: stakeOf(run, rules), vision: DEV_VISION,
    room: roomOf(run), rooms: run.floor.rooms.length,   // Cut 7 §4
    ...(run.twist ? { floor_twist: twistWord(run.twist) } : {}),   // Cut 12 §4; the cage's word on the wire
    ...forkSnap(run),                                                // Cut 26 §2
  };
}
/** Cut 26 §2 stand-in: the fork at this floor's down stairs — the other stair drawn beside the real one. */
function forkSnap(run: Run): { fork?: Snapshot["fork"] } {
  const next = run.depth + 1; const i = FAKE_FORKS.indexOf(next);
  if (i < 0 || !FAKE_OPEN.includes(next) || !forkOpen(next, run.route)) return {};
  const taken = biomeOn(next, run.route); const other = taken === FAKE_ORDER[i] ? FAKE_ORDER[i + 1] : FAKE_ORDER[i];
  const s = run.floor.tiles.indexOf("stairs_down"); const sx = s % W, sy = Math.floor(s / W);
  const at = DIRS.map(([dx, dy]) => [sx + dx, sy + dy]).filter(([x, y]) => inb(x, y) && passable(run.floor.tiles[idx(x, y)])).sort((a, b) => a[1] - b[1] || a[0] - b[0])[0] ?? [sx, sy];
  return { fork: { depth: next, taken, other, x: at[0], y: at[1] } };
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
  return { loot: run.loot, brought, return_row: ri >= 0 ? ri : undefined, kept: ri >= 0 ? Math.round(run.loot * (rows[ri].verb.v === "bank" ? 1 : 0.6)) : undefined,
           stalling: run.stuckFires > 0, death_keep: 0 };   // Cut 20 §4: a death keeps nothing in the fake; Cut 13 §1: the guard has fired — `keeps $0 · stalling`
}
function runToEnd(run: Run, ctx: SimCtx, maxTurns = 3000): void { while (!run.over && run.turn < maxTurns) simTurn(run, ctx); if (!run.over) endRun(run, "return", []); }

// --- engine ---
type State = {
  lineage: Lineage; rules: RuleSet; loadout: number[]; killed: string[]; runCounter: number; logs: Record<number, RunLog>; nextItem: number;
  tamedKinds: string[]; bredKinds: string[]; nextCid: number; killCounts: Record<string, number>;
  totalTurns: number;   // Cut 11 §5: the lineage tick (×10 like the events): `GoldLine.t` and `Snapshot.run.started_turn`, as the core's `total_turns`
  kit?: Record<string, number>;   // Cut 23 §1: forge steps owned per ladder (the core's `LineageState.kit`)
};

/** Cut 15 §2: the core's `meta::gold_price` — `150 × cost × (4 + gold_buys) / 4`, integer; 0 for a free unlock. */
const goldPrice = (cost: number, goldBuys: number): number => Math.floor(150 * cost * (4 + goldBuys) / 4);
/** Cut 23 §1: the core's forge (`kit::*`) — a unit price from the lineage's best depth (`100 + 25 × best`), each step a multiple. */
const kitUnit = (best: number): number => 100 + 25 * best;
const KIT_MULT: Record<string, number[]> = { weapon: [1, 4, 9], armour: [1, 4, 8, 14], pack: [2, 5, 10, 16] };
const KIT_ARMOUR = ["leather", "leather +1", "mail", "mail +1"];
/** Cut 23 §1: the row slots on the same ladder (`row5` … `row10`). */
const ROW_MULT: Record<string, number> = { row5: 2, row6: 5, row7: 10, row8: 16, row9: 22, row10: 30 };
/** Cut 23 §3: the core's `why_gloss` — every reason a row did not act → its reason on tap (≤ 3 words). */
const WHY_GLOSS: Record<string, string> = {
  "no path": "way blocked", "no target": "no foe reachable", "no line": "shot blocked", "no bow": "needs a bow", "cooldown": "skill recharging",
  "no item": "none in pack", "unknown item": "kind unidentified", "no unknown": "no unknowns held", "no use": "no effect now", "no leash": "needs a leash",
  "none weak": "none weak enough", "not safe": "foes too near", "no stairs": "stairs not found", "going home": "heading home", "prayed": "prayed already",
  "no shrine": "no shrine here", "no way": "exit unreachable", "card passed": "its rows idle", "card idle": "no trigger foe", "card blocked": "its move blocked", "brave held": "bravery held it", "stuck": "loop guard waits",
  "row guard": "paused: it looped", "same as R": "earlier row covers", "trait first": "trait acted first", "hazard first": "left the hazard", "recall sense": "recall read first",
  "paralysed": "cannot act", "confused": "stumbled instead", "bail": "called home", "locked cond": "cond not bought", "fired, free": "free action",
};
/** Cut 23 §3: what each paid card holds outside the typed vocabulary (the core's `meta::card_carries`). */
const CARD_CARRIES: Record<string, string> = { corridor_fighting: "four rows, one slot", kite_archers: "two rows, one slot", stair_dance: "three rows, one slot", gas_step: "four rows, one slot",
  pack_break: "four rows, one slot", thief_guard: "den raid first", boss_focus: "three rows, one slot", last_stand: "five rows, one slot" };

export class FakeEngine implements Engine {
  private s!: State;
  private live: Run | null = null;
  private lastDeath: Record<number, Death> = {};
  private settled = new Set<number>();
  private goldBuys = 0;   // Cut 15 §2: gold buys so far (the core's `LineageState.gold_buys`)
  private get kitOwned(): Record<string, number> { return (this.s.kit ??= { weapon: 0, armour: 0, pack: 0 }); }   // Cut 23 §1 stand-in, saved with the state
  private kitLadders(deltas = false): KitLadder[] {
    const L = this.s.lineage; const unit = kitUnit(L.best_depth); const w = L.class === "fighter" ? "sword" : L.class === "ranger" ? "bow" : "dagger";
    return (["weapon", "armour", "pack"] as const).map((slot) => {
      const mult = KIT_MULT[slot]; const owned = this.kitOwned[slot] ?? 0;
      const label = (i: number): string => slot === "weapon" ? `${w} +${i + 1}` : slot === "armour" ? KIT_ARMOUR[i] : `pack ${4 + i}`;
      const steps = mult.map((m, i) => ({ label: label(i), price: unit * m, owned: i < owned }));
      const nx = steps[owned]; const night = 1200;
      const next = nx ? { label: nx.label, price: nx.price, affordable: L.gold >= nx.price, nights: L.gold >= nx.price ? 0 : Math.ceil((nx.price - L.gold) / night),
        ...(deltas ? { depth: L.best_depth + 1, delta: 0.03 + 0.02 * owned, pm: 0.02, bank: 0.02, death: -0.02 } : {}) } : undefined;
      return next ? { slot, owned, steps, next } : { slot, owned, steps };
    });
  }
  buyKit(slot: string): Lineage {
    const L = this.s.lineage; const lad = this.kitLadders().find((k) => k.slot === slot);
    if (!lad) throw new Error("unknown slot"); if (!lad.next) throw new Error("top of the ladder"); if (L.gold < lad.next.price) throw new Error("not enough gold");
    this.gold(-lad.next.price, `forge ${lad.next.label}`); this.kitOwned[slot] = (this.kitOwned[slot] ?? 0) + 1; return this.lineage();
  }
  kitDeltas(): KitLadder[] { return this.kitLadders(true); }
  kitEstimates(): KitLadder[] { return this.kitLadders(true); }
  /** Cut 23 §3 stand-in: a row's why-not (`0/164 · no gas met`), from the row's shape; null before the first send. */
  private rowWhy(): (RowWhy | null)[] {
    const sends = this.s.runCounter; if (!sends) return this.s.rules.rows.map(() => null);
    return this.s.rules.rows.map((r, i) => {
      const actions = 40 * sends; const tag = r.conds.find((c) => c.k === "foe_tag")?.t;
      if (tag && Math.abs(hash(`${tag}`)) % 2 === 0) return { sends, actions, fired: 0, matched: 0, unmet: { why: tag, n: actions }, text: `0/${actions} acts · no ${tag} met` };
      if (r.verb.v === "read" || r.verb.v === "throw") { const n = 3 + i; return { sends, actions, fired: 0, matched: n, blocked: { why: "no item", n }, text: `0/${actions} acts · blocked · no item` }; }
      const fired = Math.abs(hash(`${i}:${r.verb.v}`)) % (actions / 2);
      return { sends, actions, fired, matched: fired, text: `${fired}/${actions} acts` };
    });
  }

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
      gold: this.s.lineage.gold, spent: (this.s.lineage.supplies ?? []).filter((it) => !it.free).map((it) => ({ label: it.label, kind: it.kind, price: cat.find((c) => c.kind === it.kind)?.price ?? 0 })) };
  }
  /** Cut 12 §1: the cap on the player's OWN rows; card rows (`{v:"tactic"}`) sit outside it, one per owned card. */
  private maxRows(): number { return 4 + ["row5", "row6", "row7", "row8"].filter((u) => this.s.lineage.unlocks.includes(u)).length; }
  private supplyCap(): number { return Math.min(8, (this.s.lineage.unlocks.includes("supply_cap_5") ? 5 : 3) + (this.s?.kit?.pack ?? 0)); }   // Cut 23 §1: each pack step +1
  /** Cut 12 §1: where a bought card's row goes — before the set's engagement row (the first `attack` / `shoot`), else the end. */
  private cardInsertAt(): number { const i = this.s.rules.rows.findIndex((r) => r.verb.v === "attack" || r.verb.v === "shoot"); return i < 0 ? this.s.rules.rows.length : i; }
  private vaultSlots(): number { return 1 + ["vault2", "vault3", "vault4"].filter((u) => this.s.lineage.unlocks.includes(u)).length; }

  newLineage(seed: number): Lineage {
    const trait = TRAITS[(seed >>> 3) % TRAITS.length];
    const trait_offer = [trait, TRAITS[(((seed >>> 3) % TRAITS.length) + 1 + ((seed >>> 7) % (TRAITS.length - 1))) % TRAITS.length]];   // Cut 13 §2: two traits, the first the default
    const copy = (): RuleSet => ({ rows: PRESET_FIGHTER.rows.map((r) => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb } })) });
    this.s = {
      lineage: {
        seed, heir: 1 + SEED_CHRONICLE.length, trait, trait_offer, class: "fighter", best_depth: 0, marks: 0, facts: ["item:leash"], unlocks: ["tame"], vault: [], graveyard: [], trophies: [], sets: [copy(), copy(), copy()], active_set: 0, ended: false,
        party: [], kennel: [mkCompanion(1, "jackal", 2, ["pack", "fast"], 0), mkCompanion(2, "goblin_archer", 1, ["ranged"], 0)],
        eggs: [{ id: 3, kind: "bloat", tags: ["gas"], gen: 1, hatch_in: 3, from_loss: false }], party_slots: 1, ledger: [],
        gold: 120, supplies: [{ id: 49_999, kind: "leash", known: true, label: "leash", free: true }], classes: Object.fromEntries(CLASSES.map((c) => [c, { level: 1, xp: 0 }])),   // Cut 8B §3: the kennel's leash
        forge: { dagger: { salvaged: 6, craftable: true, tier: 0 } }, renown: 0, rank: 0, keep_pref: "best_weapon", vault_pref: "weapon",
        rest_left_s: 0, bones: [],
        chronicle: SEED_CHRONICLE.map(([trait, cls, depth, deed, end, tail], i) => chronicleLine(i + 1, trait, cls, depth, deed, end, tail)),
      },
      rules: copy(), loadout: [], killed: [], runCounter: 0, logs: {}, nextItem: 50000, tamedKinds: ["jackal", "goblin_archer"], bredKinds: ["bloat"], nextCid: 10, killCounts: {}, totalTurns: 0,
    };
    if (DEV_FORK) this.s.lineage.facts.push(...FAKE_OPEN.map((f) => `fork:${f}`), "biome:burrows", "biome:fens", ...(FAKE_OPEN.length > 1 ? ["biome:crypt"] : []));   // Cut 26 dev knob: the D4 fork seen, both lanes entered
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
    this.s.lineage.vault = this.s.lineage.vault.map(withRarity); this.s.lineage.supplies = (this.s.lineage.supplies ?? []).map(withRarity);   // run-clear stand-in
    for (const c of Object.values(this.s.lineage.classes ?? {})) c.next = c.level < XP_LEVEL_CAP ? xpToNext(c.level) : 0;   // QA 92eb880: the ladder is the engine's (`ClassProg.next`)
    this.s.lineage.ledger = this.ledger();
    this.s.lineage.look = (this.s as { look?: string }).look ?? (["rogue", "ranger"].includes(this.s.lineage.class) ? "female" : "male");   // hero looks
    this.s.lineage.counters = this.counters();
    this.s.lineage.combos = combosIn(this.s.rules.rows, COMBOS);   // Cut 8B §1
    if (this.s.lineage.best_depth >= 1) this.s.lineage.bounty = { depth: this.s.lineage.best_depth + 2 }; else delete this.s.lineage.bounty;   // Cut 20 §5 stand-in: tonight's bounty floor, best + 2 (none before a best)
    this.s.lineage.shadowed_by = shadowField(this.s.rules.rows).shadowed_by;   // QA 92eb880
    // Cut 21 §2 stand-in: the repeat re-packs only the shelf's kinds a row names (`drink heal` → heal); the rest is not re-bought
    { const L = this.s.lineage; if (L.repeat !== undefined) { const named = new Set(this.s.rules.rows.flatMap((r) => (r.verb.a ?? "").split(",")));
      const cat = this.supplyCatalogue(); const kinds = [...new Set((L.supplies ?? []).filter((it) => !it.free && !it.found && named.has(it.kind)).map((it) => it.kind))];
      L.repeat_kinds = kinds; L.repeat_gold = (L.supplies ?? []).filter((it) => kinds.includes(it.kind) && !it.free && !it.found).reduce((a, it) => a + (cat.find((c) => c.kind === it.kind)?.price ?? 0), 0); } }
    this.s.lineage.start ??= 1;
    // QA a946e04 stand-ins: each trait's real rule (the core's `Trait::rule`), and whether the next send starts on `start`
    this.s.lineage.trait_rules = { greedy: "grabs loot once a floor", cowardly: "backs off once a floor under 50%", curious: "tries one unknown a floor", brave: "skips one retreat a floor" };
    { const L = this.s.lineage; const st = L.start ?? 1; L.start_payable = st <= 1 || ((L.waystones ?? []).includes(st) && L.gold >= WAYSTONE_TOLL * st); }
    { const L = this.s.lineage; const qm = L.unlocks.includes("quartermaster");   // QA 23ed91f: what an unwatched exit keeps
      L.keep_auto = L.keep_pref === "best_armour" ? (qm ? ["armour", "weapon"] : ["armour"]) : L.keep_pref === "best_weapon" ? (qm ? ["weapon", "armour"] : ["weapon"]) : []; }
    // Cut 16 §2: the wake's class chips (owned classes, the current first) while the trait offer stands
    { const L = this.s.lineage; const owned = CLASSES.filter((c) => isFreeClass(c) || L.unlocks.includes(c));
      const SIG: Record<string, [string, number]> = { fighter: ["shield_bash", 1], rogue: ["vanish", 1], ranger: ["mark", 7], caster: ["slow", 5] };
      if ((L.trait_offer ?? []).length && owned.length >= 2) L.class_offer = [L.class, ...owned.filter((c) => c !== L.class)].map((c) => ({ class: c, signature: SIG[c]?.[0] ?? "", level: L.classes[c]?.level ?? 1, opens: SIG[c]?.[1] ?? 1 }));
      else delete L.class_offer; }
    // Cut 9 §10: the forge ladder's next rung per kind (`3/5 → craftable`, `6/15 → +1`, `20/40 → +2`; none at the top)
    for (const f of Object.values(this.s.lineage.forge ?? {})) { const rung = FORGE_LADDER.find((r) => f.salvaged < r.need); if (rung) f.next = { ...rung }; else delete f.next; }
    this.s.lineage.kit = this.kitLadders();   // Cut 23 §1
    this.s.lineage.row_why = this.rowWhy();   // Cut 23 §3
    // Cut 26 §2 stand-ins: the seen forks (the chip line), every lit (lane, depth), the rows whose conds this lineage cannot use
    { const L = this.s.lineage; const route = this.s.rules.route ?? [];
      const forks = FAKE_FORKS.filter((f) => L.facts.includes(`fork:${f}`)).map((f) => { const i = FAKE_FORKS.indexOf(f); return { depth: f, near: FAKE_ORDER[i], far: FAKE_ORDER[i + 1], taken: biomeOn(f, route), open: forkOpen(f, route) }; });
      if (forks.length) L.forks = forks; else delete L.forks;
      const lanes = (L.waystones ?? []).map((d) => ({ depth: d, lane: biomeOn(d, route), current: true }));
      if (lanes.length) L.lanes = lanes; else delete L.lanes;
      const v = this.vocabulary(); const open = (c: Cond): boolean => v.conds.some((o) => o.k === c.k && (o.t ?? "") === (c.t ?? "")) || ["hp<", "hp>", "foes>=", "adj>=", "depth>=", "floor_seen>="].includes(c.k);
      const locked = this.s.rules.rows.map((r) => { const c = r.conds.find((x) => !open(x)); return c ? (v.locked?.find((l) => l.cond.k === c.k && (l.cond.t ?? "") === (c.t ?? ""))?.needs ?? (c.k === "in" ? `enter ${c.t}` : "locked")) : null; });
      if (locked.some((x) => x)) L.locked_rows = locked; else delete L.locked_rows; }
    return JSON.parse(JSON.stringify(this.s.lineage)) as Lineage;
  }
  /** Cut 6 §5: bosses whose counter fact is known, with the counter as a row. */
  private counters(): Counter[] {
    // the core merges facts into the lineage as they are learned; the live run's facts count here for the same reason
    return [...new Set([...this.s.lineage.facts, ...(this.live?.facts ?? [])])].flatMap((f) => { const m = /^boss:([a-z_]+):counter$/.exec(f); const c = m && COUNTER[m[1]]; return c ? [{ boss: m[1], row: c.row, text: c.text }] : []; });
  }
  /** Cut 6 §1: every gold movement is a ledger line (`+$50 returned D5`, `−$40 heal potion`), the last 20 kept, oldest first. */
  private gold(delta: number, why: string, n = 0): void {
    const L = this.s.lineage; L.gold += delta;
    const g = (L.gold_ledger ??= []);
    // QA on 778fa1b (qaV): a line counts the supplies it bought or refunded (`GoldLine.n`); same tick and reason merge, as the core's do
    const last = g[g.length - 1];
    if (n > 0 && last && last.t === this.s.totalTurns && last.why === why) { last.delta += delta; last.n = (last.n ?? 0) + n; return; }
    g.push(n > 0 ? { t: this.s.totalTurns, delta, why, n } : { t: this.s.totalTurns, delta, why }); while (g.length > GOLD_LEDGER_CAP) g.shift();   // Cut 11 §5: the lineage tick
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
    if (e && L.supplies.length < this.supplyCap() && L.gold >= e.price) { this.gold(-e.price, e.label, 1); L.supplies.push({ id: this.s.nextItem++, kind: e.kind, known: true, label: e.label }); }
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
    // Cut 26 §2: `in: <biome>` once entered (its `biome:` fact); a biome a seen fork offers and not yet entered is locked (`enter fens`)
    for (const b of FAKE_ORDER) if (L.facts.includes(`biome:${b}`)) conds.push({ k: "in", t: b });
    const gated = conds.filter((c) => !COND_UNLOCK[c.k] || L.unlocks.includes(COND_UNLOCK[c.k]));
    const locked = conds.filter((c) => COND_UNLOCK[c.k] && !L.unlocks.includes(COND_UNLOCK[c.k]))
      .map((cond) => { const u = UNLOCKS[COND_UNLOCK[cond.k]]; return { cond, needs: u.gate && !u.gate(L) ? u.needs ?? "?" : `◆${u.cost}` }; });
    FAKE_FORKS.forEach((f, i) => { if (!L.facts.includes(`fork:${f}`)) return; for (const b of [FAKE_ORDER[i], FAKE_ORDER[i + 1]]) if (!L.facts.includes(`biome:${b}`) && !locked.some((l) => l.cond.k === "in" && l.cond.t === b)) locked.push({ cond: { k: "in", t: b }, needs: `enter ${b}` }); });
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
    return { conds: gated, verbs, max_rows: this.maxRows(), combos: COMBOS, locked, why_gloss: WHY_GLOSS };   // Cut 23 §3: why_gloss
  }
  setRules(set: RuleSet): void {
    // Cut 12 §1: own rows ≤ max_rows and card rows ≤ cards owned (one per card) — refused, never truncated
    const own = set.rows.filter((r) => r.verb.v !== "tactic").length; if (own > this.maxRows()) throw new Error(`${own} rows over ${this.maxRows()}`);
    const cards = set.rows.filter((r) => r.verb.v === "tactic").map((r) => r.verb.a ?? "");
    if (new Set(cards).size !== cards.length) throw new Error("a card twice");
    for (const c of cards) if (!this.s.lineage.unlocks.includes(c)) throw new Error(`card ${c} not owned`);
    // Cut 26 §2: a route takes only seen forks, never two overlapping far stairs; an `in:` cond needs its biome entered
    const route = [...(set.route ?? [])].sort((a, b) => a - b);
    for (const f of route) { if (!FAKE_FORKS.includes(f)) throw new Error(`no fork at D${f}`); if (!this.s.lineage.facts.includes(`fork:${f}`)) throw new Error(`D${f} fork unseen`); }
    if (route.some((f, i) => i > 0 && FAKE_FORKS.indexOf(f) === FAKE_FORKS.indexOf(route[i - 1]) + 1)) throw new Error("forks overlap");
    for (const [i, r] of set.rows.entries()) for (const c of r.conds) if (c.k === "in" && !this.s.lineage.facts.includes(`biome:${c.t}`)) throw new Error(`row ${i + 1}: in ${c.t} is locked (enter ${c.t})`);
    this.home = 0;                                                           // a rule edit opens a fresh stall window
    this.s.rules = { rows: set.rows.map((r) => ({ conds: (r.verb.v === "tactic" ? [] : r.conds.slice(0, 2)).map((c) => ({ ...c })), verb: { ...r.verb } })), name: set.name, ...(route.length ? { route } : {}) };
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

  forecast(): Forecast { this.fcRefined = false; return { ...this.forecastN(20), refined: false, start: this.payableStart(), ...shadowField(this.s.rules.rows) }; }
  forecastEstimate(): Forecast { this.fcRefined = false; return { ...this.forecastN(8), refined: false, start: this.payableStart(), ...shadowField(this.s.rules.rows) }; }
  /** Cut 21 §1 stand-in: the floor the next send starts on — the chosen waystone when lit and the purse pays its toll, else D1. */
  private payableStart(): number { const L = this.s.lineage, st = L.start ?? 1; return st > 1 && (L.waystones ?? []).includes(st) && L.gold >= WAYSTONE_TOLL * st ? st : 1; }   // Cut 13 §5: the first paint is marked (`±6…`)
  /** Cut 6 §9: the same forecast at 100 sims (the client asks 2 s after a quiet paint). Same seeds ⇒ the first 20 agree. */
  /** QA 778fa1b stand-in: whether the last camp panel asked was the refine — `forecastVs` then pairs 100 seeds and says `refined`. */
  private fcRefined = false;
  forecastRefine(): Forecast { this.fcRefined = true; return { ...this.forecastN(100), refined: true, start: this.payableStart(), ...shadowField(this.s.rules.rows) }; }
  /** Cut 22 §3 stand-in: the active set's panel minus `prev`'s on the same 20 seeds — per depth and on the ends, each with its paired
   *  half-width (1.96 σ of the per-seed difference / √N), far tighter than either bar's own ± when the two sets mostly agree. */
  forecastVsEstimate(prev: RuleSet): ForecastVs { return this.forecastVsN(prev, 8); }
  forecastVs(prev: RuleSet): ForecastVs { return this.forecastVsN(prev, this.fcRefined ? 100 : 20); }
  private forecastVsN(prev: RuleSet, N: number): ForecastVs {
    const L = this.s.lineage, known = this.known(), known_to = L.best_depth + 1;
    const a: Run[] = [], b: Run[] = [];
    for (let i = 0; i < N; i++) { const seed = hash(`fc:${L.seed}:${i}`); a.push(this.simOne(seed, this.s.rules, known)); b.push(this.simOne(seed, prev, known)); }
    const move = (f: (r: Run) => number): VsMove => {
      const d = a.map((r, i) => f(r) - f(b[i])), m = d.reduce((x, y) => x + y, 0) / N;
      const v = d.reduce((x, y) => x + (y - m) * (y - m), 0) / Math.max(1, N - 1);
      return { delta: m, pm: 1.96 * Math.sqrt(v / N) };
    };
    const depths = Array.from({ length: Math.min(15, known_to) }, (_, k) => ({ depth: k + 1, ...move((r) => (r.depth >= k + 1 ? 1 : 0)) }));
    return { depths, bank: move((r) => (r.exit === "bank" ? 1 : 0)), death: move((r) => ((r.exit ?? "death") === "death" ? 1 : 0)), return: move((r) => (r.exit === "return" ? 1 : 0)), sims: N, refined: N > 20 };   // QA 778fa1b: the panels paired are the refined ones once the refine ran
  }
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
    // Cut 20 §5 stand-in: the bounty floor (best + 2) is a depth of its own, flagged
    const bountyD = L.best_depth >= 1 ? L.best_depth + 2 : -1;
    const depths: Forecast["depths"] = []; for (let d = 1; d <= Math.min(15, Math.max(known_to, bountyD)); d++) { const p = reach[d] / N; depths.push({ depth: d, reach: p, pm: 1.96 * Math.sqrt((p * (1 - p)) / N), ...(d === bountyD ? { bounty: true } : {}) }); }
    const top = Object.entries(causes).sort((a, b) => b[1] - a[1]).slice(0, 3).map(([cause, n]) => ({ cause, share: n / N }));
    // Cut 10 §2: a boss floor whose counter fact is known and whose row is absent from the set names it: `try: attack boss`
    const route = this.s.rules.route ?? [];
    if (route.length) for (const d of depths) d.biome = biomeOn(d.depth, route);   // Cut 26 §2: the floor's biome on the set's route
    for (const d of depths) {
      const boss = bossOn(d.depth, route); const c = boss && COUNTER[boss];
      if (!c || !this.known().has(`boss:${boss}:counter`) || this.s.rules.rows.some((r) => rowText(r) === rowText(c.row))) continue;
      d.cause ??= boss; d.try = { row: JSON.parse(JSON.stringify(c.row)) as Row, text: c.text, met: d.depth };
    }
    // Cut 18 §3: reach falls to ≤ 5 % below a boss's floor (from over 5 % on it): the row names the wall (`D9 0% · warlord wall`)
    for (const d of depths) { const b = bossOn(d.depth, route); if (b) d.boss = b; }   // Cut 24 §5 stand-in: the boss named on the floor he is met on
    for (const d of depths) { const boss = bossOn(d.depth - 1, route); const above = depths.find((x) => x.depth === d.depth - 1); if (boss && above && d.reach <= 0.05 && above.reach > 0.05) d.wall = boss; }
    // Cut 27 §1 stand-in: each floor's clear (the sims on it that got through it) and the fold — every floor from the start that clears
    // ≥ 95 % and is known (≤ the best depth); `?fake_fold=N` folds D1–N whatever the sims say (UI dev)
    const start = this.payableStart(); let fold_to: number | undefined;
    for (const d of depths) if (d.depth >= start && reach[d.depth] > 0) d.clear = (reach[d.depth + 1] ?? 0) / reach[d.depth];
    for (let d = start; d <= L.best_depth; d++) { const c = depths.find((x) => x.depth === d)?.clear; if (c === undefined || c < FOLD_CLEAR) break; fold_to = d; }
    if (DEV_FOLD > 0) { const to = Math.min(DEV_FOLD, Math.max(L.best_depth, 1)); if (to >= start) { fold_to = Math.max(fold_to ?? 0, to); for (const d of depths) if (d.depth >= start && d.depth <= to) d.clear = 1; } }
    const death = ends.death / N;
    return { depths, causes: top, known_to, sims: N, low: Math.ceil(100 / N), ...(fold_to !== undefined ? { fold_to } : {}), ends: { bank: ends.bank / N, return: ends.return / N, death, gold: ends.gold / N, pm: 1.96 * Math.sqrt((death * (1 - death)) / N) } };   // Cut 13 §5: the ends line's own ±; Cut 23 §2: `low` — a 0 of N prints `<low%`
  }

  private startRun(): Run {
    const L = this.s.lineage; this.s.runCounter++;
    const id = this.s.runCounter; const seed = hash(`run:${L.seed}:${id}`);
    // Cut 21 §1 stand-in: a lit waystone start pays its toll (`waystone D9 −$90`); short of the toll the run starts on D1
    let start = L.start ?? 1;
    if (start > 1 && (!(L.waystones ?? []).includes(start) || L.gold < WAYSTONE_TOLL * start)) start = 1;
    if (start > 1 && WAYSTONE_TOLL > 0) this.gold(-WAYSTONE_TOLL * start, `waystone D${start}`);
    const run = makeRun(id, L.heir, seed, L.class, L.trait, this.known(), this.brought(), this.ctx(), L.party, this.classLevel(), start);
    run.startedTotal = this.s.totalTurns;
    this.s.logs[id] = { seed, rules: JSON.parse(JSON.stringify(this.s.rules)) as RuleSet, depth: 1, turns: 0, exit: "", known: [...this.known()], cls: L.class, trait: L.trait, heir: L.heir, hpMargin: 0 };
    return run;
  }
  send(): Snapshot {
    this.inAbsence = false;   // RUNS_UI: a send ends the absence's fold
    // Cut 27 §1 stand-in: the send's fold, from the camp's panel as it stands (the core's `fold_plan`)
    if (!this.live || this.live.over) { const f = this.forecastN(20); this.foldPlan = f.fold_to !== undefined ? { to: f.fold_to, clears: new Map(f.depths.filter((d) => d.clear !== undefined).map((d) => [d.depth, d.clear as number])) } : null; }
    this.s.lineage.rest_left_s = 0; delete this.s.lineage.trait_offer; return this.peek();   // Cut 13 §2: the send settles the heir's trait
  }
  private foldPlan: { to: number; clears: Map<number, number> } | null = null;
  /** Cut 27 §1 stand-in: the core's `fold()` — the live run stepped through the folded floors, one floor entry per floor (its first
   *  snapshot, its events, its beats), the line's clear/gold/chips, and the whole as one `step()`. */
  fold(): FoldLine {
    const plan = this.foldPlan; this.foldPlan = null;
    if (!this.live || this.live.over) this.live = this.startRun();
    const from = this.live.depth, to = plan?.to ?? 0;
    const cur = (): Snapshot => { const s = snapshot(this.live!, this.s.rules); s.turn *= 10; return s; };
    if (!plan || to < from) return { from, to: from - 1, clear: 1, gold: 0, beats: [], chips: [], floors: [], step: { events: [], snapshot: cur(), run_over: false } };
    const floors: FoldFloor[] = []; const all: Ev[] = [];
    let f: FoldFloor = { depth: from, clear: plan.clears.get(from) ?? 1, gold: 0, snapshot: cur(), events: [], beats: [] }; let loot0 = this.live.loot;
    let last: StepResult = { events: [], snapshot: f.snapshot, run_over: false };
    const close = (loot: number): void => { f.gold = loot - loot0; const dip = fakeDip(f.events, f.snapshot.hero.max_hp); if (dip) f.beats.push({ depth: f.depth, ...dip }); f.beats.sort((a, b) => a.t - b.t); floors.push(f); };
    for (let guard = 0; guard < 5000; guard++) {
      if (this.live.over || this.live.depth > to) break;
      const lootBefore = this.live.loot;
      last = this.step(10); all.push(...last.events);
      const k = last.events.findIndex((e) => e.k === "descend");
      if (k >= 0 && last.snapshot.depth !== f.depth) {
        const old = last.events.slice(0, k), nu = last.events.slice(k);
        f.events.push(...old); f.beats.push(...old.flatMap((e) => fakeBeat(e, f.depth)));
        close(lootBefore); loot0 = lootBefore;
        f = { depth: last.snapshot.depth, clear: plan.clears.get(last.snapshot.depth) ?? 1, gold: 0, snapshot: last.snapshot, events: [...nu], beats: last.snapshot.depth <= to ? nu.flatMap((e) => fakeBeat(e, last.snapshot.depth)) : [] };
      } else { f.events.push(...last.events); f.beats.push(...last.events.flatMap((e) => fakeBeat(e, f.depth))); }
    }
    if (last.run_over && f.depth <= to) close(this.live.loot);
    const beats = floors.flatMap((x) => x.beats);
    const clear = floors.reduce((p, x) => p * x.clear, 1), gold = floors.reduce((g, x) => g + x.gold, 0);
    return { from, to: floors.length ? floors[floors.length - 1].depth : from - 1, clear, gold, beats, chips: fakeChips(beats), floors, step: { ...last, events: all } };
  }
  /** Cut 27 §2 stand-in: the core's `divergence(prev)` — on the forecast's seeds, the first whose ends differ (else the first where the
   *  sets act differently), played in step to the first turn the rows fired differ; both branches' next seconds re-played from a few
   *  turns before it. Turns ×10 on the wire, as `step`'s. */
  divergence(prev: RuleSet): Divergence | null {
    const L = this.s.lineage, N = this.fcRefined ? 100 : 20, known = this.known(), rules = this.s.rules;
    const seeds = Array.from({ length: N }, (_, i) => hash(`fc:${L.seed}:${i}`));
    const a = seeds.map((sd) => this.simOne(sd, rules, known)), b = seeds.map((sd) => this.simOne(sd, prev, known));
    const gap = (x: Run, y: Run): number => ((x.exit === "death") !== (y.exit === "death") ? 1000 : 0) + (x.depth !== y.depth ? 100 + Math.abs(x.depth - y.depth) : 0) + (x.exit !== y.exit ? 10 : 0) + (x.turn !== y.turn ? 1 : 0);
    const order = seeds.map((_, i) => i).sort((i, j) => gap(a[j], b[j]) - gap(a[i], b[i]) || i - j);
    const mk = (sd: number, rs: RuleSet): { run: Run; ctx: SimCtx } => { const ctx = this.ctx(rs); return { run: makeRun(0, L.heir, sd, L.class, L.trait, new Set(known), [], ctx, [], this.classLevel()), ctx }; };
    const plain = (es: Ev[], rs: RuleSet): string => JSON.stringify(es.map((e) => (e.k === "rule" ? { ...e, row: e.row >= 0 ? JSON.stringify(rs.rows[e.row] ? [rs.rows[e.row].conds, rs.rows[e.row].verb] : e.row) : e.row } : e)));
    const firedRow = (es: Ev[]): number | undefined => { for (const e of es) if (e.k === "rule" && e.row >= 0) return e.row; return undefined; };
    for (const i of order.slice(0, 8)) {
      const s = mk(seeds[i], prev), w = mk(seeds[i], rules);
      let t0 = -1, rs: number | undefined, rw: number | undefined;
      for (let t = 0; t < 3000 && t0 < 0; t++) {
        if (s.run.over || w.run.over) { if (s.run.over && w.run.over) break; t0 = s.run.turn; break; }
        const es = simTurn(s.run, s.ctx), ew = simTurn(w.run, w.ctx);
        if (plain(es, prev) !== plain(ew, rules)) { t0 = s.run.turn; rs = firedRow(es); rw = firedRow(ew); }
      }
      if (t0 < 0) continue;
      const from = Math.max(0, t0 - 2), depth = s.run.depth;
      const branch = (rs0: RuleSet, row: number | undefined): DivergenceBranch => {
        const g = mk(seeds[i], rs0);
        while (g.run.turn < from && !g.run.over) simTurn(g.run, g.ctx);
        const snap = (): Snapshot => { const x = snapshot(g.run, rs0); x.turn *= 10; return x; };
        const first = snap(); const events: Ev[] = [];
        while (g.run.turn < t0 + 6 && !g.run.over) { const es = simTurn(g.run, g.ctx); const k = es.findIndex((e) => e.k === "descend"); if (k >= 0) { events.push(...es.slice(0, k)); break; } events.push(...es); }
        for (const e of events) e.t *= 10;
        const verb = row !== undefined ? rs0.rows[row]?.verb : undefined;
        return { ...(row !== undefined ? { row } : {}), text: row !== undefined && verb ? `R${row + 1} ${verb.v.replace(/_/g, " ")}` : "—", snapshot: first, events, end_snapshot: snap() };
      };
      const end = (r: Run): DivergenceEnd => ({ tier: r.stalled ? "stall" : r.exit ?? "death", depth: r.depth, ...(r.exit === "death" && r.cause ? { cause: r.cause } : {}), gold: r.loot_kept });
      const vs = this.forecastVs(prev);
      const mv = [...vs.depths.map((d) => ({ d: Math.abs(d.delta), pm: d.pm ?? 0 })), ...[vs.bank, vs.death, vs.return].map((m) => (typeof m === "object" ? { d: Math.abs(m.delta), pm: m.pm ?? 0 } : { d: 0, pm: 0 }))].sort((x, y) => y.d - x.d)[0] ?? { d: 0, pm: 0 };
      const sent = branch(prev, rs), nu = branch(rules, rw);
      return { seed: i, tick: t0 * 10, depth, ...(rs !== undefined ? { sent_row: rs } : {}), ...(rw !== undefined ? { new_row: rw } : {}), sent_end: end(b[i]), new_end: end(a[i]), sent, new: nu, moved: mv.d, inside: mv.d <= mv.pm, fires: fakeFires(prev, rules, b.length), sims: N };
    }
    return null;
  }
  /** The live run's first frame without a send (the report's `live`): the trait offer stands. */
  private peek(): Snapshot { if (!this.live || this.live.over) this.live = this.startRun(); const snap = snapshot(this.live, this.s.rules); snap.turn *= 10; return snap; }
  /** Cut 13 §2: pick one of the offered traits; an idle live run is rebuilt so the next send wakes with it. */
  setTrait(name: string): Lineage {
    const L = this.s.lineage;
    if (!(L.trait_offer ?? []).includes(name)) throw new Error(`no trait ${name} on offer`);
    L.trait = name; this.dropIdleRun(); return this.lineage();
  }
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
  private prefPick(run: Run): number[] {
    const items = this.carried(run); const pref = this.s.lineage.keep_pref;
    const pickBest = (score: (i: InvItem) => number): number[] => { const c = items.filter((i) => score(i) > 0).sort((a, b) => score(b) - score(a))[0]; return c ? [c.id] : []; };
    if (pref === "best_weapon") return pickBest((i) => WEAPONS[i.kind]?.[1] ?? 0);
    if (pref === "best_armour") return pickBest((i) => ARMOUR[i.kind] ?? 0);
    return [];
  }
  private settle(run: Run, real: boolean, keepIds: number[] = []): { facts: string[]; bests: string[]; marks: number; tamed: string[]; lost: string[]; hatched: string[]; xp: number; levels: number[]; salvaged: { kind: string; n: number; gold: number }[]; score: number; ranks: number[]; spent: { kind: string; n: number; gold: number }[] } {
    const L = this.s.lineage; const log = this.s.logs[run.id];
    if (log) { log.depth = run.depth; log.cause = run.cause; log.turns = run.turn; log.exit = run.exit ?? ""; log.line = run.line; log.hpMargin = run.lastHurt ? run.lastHurt.dmg - run.lastHurt.hpBefore + 1 : 0; if (run.exit === "death") log.turns = run.turn; log.stalled = run.stalled; log.notes = [...run.notes]; }
    if (!real) return { facts: [], bests: [], marks: 0, tamed: [], lost: [], hatched: [], xp: 0, levels: [], salvaged: [], score: 0, ranks: [], spent: [] };
    const facts: string[] = []; const bests: string[] = []; let marks = 0;
    const tamed: string[] = [], lost: string[] = [], hatched: string[] = [];
    this.gold(run.loot_kept, `${run.stalled ? "stalled" : run.exit === "bank" ? "banked" : run.exit === "return" ? "returned" : "died"} D${run.depth}`); L.supplies = [];
    // Cut 13 §3: `auto: restock` rebuys what the run packed, at the return, when the gold allows (the report's SPENT rows)
    const spent: { kind: string; n: number; gold: number }[] = [];
    if (L.unlocks.includes("auto_supply")) for (const p of run.spent) {
      if (L.gold < p.price || L.supplies.length >= this.supplyCap()) continue;
      this.gold(-p.price, p.label, 1); L.supplies.push({ id: this.s.nextItem++, kind: p.kind, known: true, label: p.label });
      const row = spent.find((x) => x.kind === p.kind) ?? (spent.push({ kind: p.kind, n: 0, gold: 0 }), spent[spent.length - 1]); row.n++; row.gold += p.price;
    }
    L.rest_left_s = run.rest_s;                                             // Cut 2 §1: camp rest after every expedition (send skips it)
    // RUNS_UI: the runs log, as the fake can tell it (the core's `run_log`: via, the absence's fold, the clock at the end)
    { const runs = (L.runs ??= []); const away = this.offlineVia;
      runs.push({ id: run.id, heir: run.heir, via: away ? "away" : "watched", ...(away ? { absence: L.absences ?? 0 } : {}), clock_s: L.clock_s ?? 0, start: 1, depth: run.depth,
        tier: run.exit ?? "return", ...(run.line?.reason ? { reason: run.line.reason } : {}), gold: run.loot_kept, found: run.picked.length, turns: run.turn,
        ...(run.depth > L.best_depth ? { best: true } : {}), ...(run.exit === "death" ? { death_id: run.id } : {}) });
      while (runs.length > 60) runs.shift(); }
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
    if (run.line) { run.line.xp = xp; run.line.level_ups = levels.length; }   // QA 92eb880: the exit line carries the run's XP (the wire's)
    // run-clear stand-in: the core's card fields — the end, the floor, a record, the finds (rarest first, with `item::rarity`'s table)
    if (run.line) { run.line.end = run.exit; run.line.reached = run.depth; run.line.new_best = run.depth > L.best_depth;
      run.line.finds = run.picked.filter((it) => it.kind !== "gold").map(withRarity).sort((a, b) => RARITY_ORDER.indexOf(b.rarity ?? "common") - RARITY_ORDER.indexOf(a.rarity ?? "common")).slice(0, 6); }
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
      // Cut 13 §2: the new heir wakes with two traits on offer (never the last one); the first is the default until `setTrait`
      const last = L.trait; const pool = TRAITS.filter((t) => t !== last); const a = (L.seed + L.heir * 7) % pool.length;
      L.heir += 1; L.trait_offer = [pool[a], pool[(a + 1 + ((L.seed >>> 5) % (pool.length - 1))) % pool.length]]; L.trait = L.trait_offer[0];
    }
    // Addendum D: keep into free vault slots, salvage the rest (Cut 2 §2: a death's kit is bones, not salvage)
    const tierMul = run.exit === "bank" ? 1 : 0.6;
    const salv: Record<string, { n: number; gold: number }> = {};
    // Cut 21 §2 stand-in: a found supply of a kind the shelf sells goes onto the shelf (up to the cap), not to salvage
    const sells = new Set(this.supplyCatalogue().map((c) => c.kind)); const shelf: Record<string, number> = {};
    for (const it of run.exit === "death" ? [] : this.carried(run)) {
      if (keepIds.includes(it.id) && L.vault.length < this.vaultSlots()) { L.vault.push({ ...it, id: this.s.nextItem++ }); continue; }
      if (it.known && sells.has(it.kind) && L.supplies.length < this.supplyCap()) { L.supplies.push({ ...it, id: this.s.nextItem++, found: true }); shelf[it.kind] = (shelf[it.kind] ?? 0) + 1; continue; }
      const g = Math.round(salvageOf(it.kind) * tierMul); this.gold(g, `salvaged ${it.kind.replace(/_/g, " ")}`);
      const f = (L.forge[it.kind] ??= { salvaged: 0, craftable: false, tier: 0 }); f.salvaged++; f.craftable = f.salvaged >= 5; f.tier = f.salvaged >= 40 ? 2 : f.salvaged >= 15 ? 1 : 0;
      (salv[it.kind] ??= { n: 0, gold: 0 }).n++; salv[it.kind].gold += g;
    }
    const salvaged = Object.entries(salv).map(([kind, v]) => ({ kind, ...v }));
    if (run.line && Object.keys(shelf).length) run.line.shelved = Object.entries(shelf).map(([kind, n]) => ({ kind, n }));
    // Cut 21 §1 stand-in: a bank lights every waystone at or above its floor
    if (run.exit === "bank") { const lit = WAYSTONES.filter((w) => w <= run.depth); L.waystones = [...new Set([...(L.waystones ?? []), ...lit])].sort((a, b) => a - b); }
    // renown
    const score = 10 * run.depth + run.score + run.hl.reduce((a, b) => a + b.score, 0); L.renown += score; const ranks: number[] = [];
    while (L.renown >= 100 * (L.rank + 1) * (L.rank + 1)) { L.rank++; L.marks++; ranks.push(L.rank); }
    return { facts, bests, marks, tamed, lost, hatched, xp, levels, salvaged, score, ranks, spent };
  }
  private pending: Run | null = null;
  keep(ids: number[]): Lineage {
    const run = this.pending; if (!run) return this.lineage();
    this.pending = null; this.settle(run, true, ids); return this.lineage();
  }
  /** QA 23ed91f: the skipped sheet's call — the fake's preference pick (`autoKeep`). */
  autoKeep(): Lineage { const run = this.pending; if (!run) return this.lineage(); return this.keep(this.prefPick(run)); }
  // hero looks: cosmetic; the class's own until set (`look` rides the save)
  setLook(look: string): Lineage { if (!["male", "female", "cat"].includes(look)) throw new Error(`unknown look ${look}`); (this.s as { look?: string }).look = look; return this.lineage(); }
  setKeepPref(pref: string): Lineage { if (["best_weapon", "best_armour", "none"].includes(pref)) this.s.lineage.keep_pref = pref; return this.lineage(); }
  // Cut 5 stand-ins: the fake places no vaults, so `choose` only answers with the live snapshot; `bail` ends the run on the next step
  private bailed = false;
  bail(): void { if (this.live && !this.live.over) this.bailed = true; }
  choose(_itemId: number): Snapshot { if (!this.live) this.live = this.startRun(); const snap = snapshot(this.live, this.s.rules); snap.turn *= 10; return snap; }
  setVaultPref(pref: string): Lineage { if (["weapon", "armour", "potion", "scroll"].includes(pref)) this.s.lineage.vault_pref = pref; return this.lineage(); }
  /** Cut 19 §1 stand-in: the fake places no cages, so each preference reads as a fixed nudge on the forecast's best-depth bar
   *  (armour the lift AA found, the rest near zero) — enough for the tablet and its picker to paint. */
  cageForecast(): CageOption[] {
    const L = this.s.lineage; const f = this.forecastN(20); const depth = Math.max(1, L.best_depth);
    const reach = f.depths.find((d) => d.depth === depth)?.reach ?? 0; const bank = f.ends?.bank ?? 0; const gold = f.ends?.gold ?? 0;
    const lift: Record<string, number> = { weapon: 0, armour: 0.2, potion: 0.05, scroll: -0.02 };
    const cur = L.vault_pref ?? "weapon"; const base = lift[cur] ?? 0;
    return ["weapon", "armour", "potion", "scroll"].map((pref) => {
      const d = (lift[pref] ?? 0) - base; const r = Math.min(1, Math.max(0, reach + d)); const b = Math.min(1, Math.max(0, bank + (bank > 0 ? d : 0)));
      return { pref, current: pref === cur, depth, reach: r, reach_delta: r - reach, bank: b, bank_delta: b - bank, gold: gold * (1 + d), gold_delta: gold * d,
               delta: bank > 0 ? b - bank : r - reach, pm: 1.96 * Math.sqrt(r * (1 - r) / 20) };
    });
  }
  /** Cut 21 §1 stand-in: the next send starts on D1 or a lit waystone. */
  setStart(depth: number): Lineage {
    const L = this.s.lineage;
    if (depth !== 1 && !(L.waystones ?? []).includes(depth)) throw new Error(`no waystone at D${depth}`);
    L.start = depth; this.dropIdleRun(); return this.lineage();
  }
  /** Cut 21 §1 stand-in: each start as a fixed nudge on the forecast's ends (a deeper start banks more, dies more), like `cageForecast`. */
  startForecast(): StartOption[] {
    const L = this.s.lineage; const f = this.forecastN(20); const bank = f.ends?.bank ?? 0, gold = f.ends?.gold ?? 0;
    const cur = L.start ?? 1; const starts = [1, ...(L.waystones ?? [])];
    const death0 = f.ends?.death ?? 0;   // Cut 22 §4 stand-in: a deeper start dies more (the start picker's `death 61%`)
    const at = (s0: number): { bank: number; gold: number; reach: number; death: number } => { const k = (s0 - 1) / 30; return { bank: Math.max(0, Math.min(1, bank + (bank > 0 ? 0.3 * k : 0) - 0.4 * k * k)), gold: gold * (1 + 2 * k), reach: Math.max(0, 1 - k), death: Math.min(1, death0 + 1.8 * k) }; };
    const c = at(cur);
    return starts.map((st) => { const o = at(st); return { start: st, current: st === cur, toll: st > 1 ? WAYSTONE_TOLL * st : 0, short: st > 1 && L.gold < WAYSTONE_TOLL * st, biome: biomeOn(st, this.s.rules.route), depth: Math.max(1, L.best_depth), reach: o.reach, reach_delta: o.reach - c.reach,
      bank: o.bank, bank_delta: o.bank - c.bank, gold: o.gold, gold_delta: o.gold - c.gold, delta: bank > 0 ? o.bank - c.bank : o.reach - c.reach, pm: 1.96 * Math.sqrt(o.bank * (1 - o.bank) / 20), death: o.death, low: 5 }; });
  }
  /** Cut 26 §2 stand-in: both stairs of a seen fork for the active set — each route's own 20-sim panel, read at the band's last floor. */
  forkForecast(fork: number): ForkOption[] {
    const i = FAKE_FORKS.indexOf(fork); const cur = this.s.rules.route ?? [];
    if (i < 0 || !forkOpen(fork, cur) || !this.s.lineage.facts.includes(`fork:${fork}`)) return [];
    const bar = FAKE_BANDS[i][1]; const L = this.s.lineage; const N = 20;
    const read = (route: number[]): { reach: number; bank: number; gold: number; death: number } => {
      const rules = { ...this.s.rules, route }; let reach = 0, bank = 0, gold = 0, death = 0;
      for (let k = 0; k < N; k++) { const r = this.simOne(hash(`fc:${L.seed}:${k}`), rules, this.known()); if (r.depth >= bar) reach++; if (r.exit === "bank") bank++; if (r.exit === "death") death++; gold += r.loot_kept; }
      return { reach: reach / N, bank: bank / N, gold: gold / N, death: death / N };
    };
    const base = read(cur);
    return [false, true].map((far) => {
      const route = routeWith(cur, fork, far); const current = route.join() === [...cur].sort((a, b) => a - b).join(); const o = current ? base : read(route);
      return { fork, biome: FAKE_ORDER[i + (far ? 1 : 0)], far, current, route, depth: bar, reach: o.reach, reach_delta: o.reach - base.reach, bank: o.bank, bank_delta: o.bank - base.bank,
               gold: o.gold, gold_delta: o.gold - base.gold, death: o.death, death_delta: o.death - base.death, delta: o.bank > 0 || base.bank > 0 ? o.bank - base.bank : o.reach - base.reach,
               pm: 1.96 * Math.sqrt(o.reach * (1 - o.reach) / N), refined: false, low: 5 };
    });
  }
  /** Cut 19 §3 stand-in: the fake has no repeat; the flag is kept on the lineage so the tile can toggle. */
  setRestock(on: boolean): Lineage { (this.s.lineage as Lineage).repeat = on; if (!on) return this.clearSupplies(); return this.lineage(); }
  insure(id: number): Lineage {
    const L = this.s.lineage; const it = L.vault.find((v) => v.id === id);
    if (!it || (L.insured ?? []).includes(id)) return this.lineage();
    const price = Math.ceil(salvageOf(it.kind) * 10 / 4); if (L.gold < price) return this.lineage();   // the camp's own price (ui/salvage.ts)
    this.gold(-price, `insure ${it.label}`); L.insured = [...(L.insured ?? []), id]; return this.lineage();
  }

  /** QA 308f045 stand-in: an item out of the vault, salvaged at a bank's share (the client table ÷ 4, the core's divisor). */
  sellVault(id: number): Lineage {
    const L = this.s.lineage; const i = L.vault.findIndex((v) => v.id === id); if (i < 0) return this.lineage();
    const [it] = L.vault.splice(i, 1); L.insured = (L.insured ?? []).filter((x) => x !== id);
    const coins = Math.floor(salvageOf(it.kind) / 4); if (coins > 0) this.gold(coins, `salvage ${it.label}`); return this.lineage();
  }
  runOfflineQuick(elapsedS: number): ReturnReport { const r = this.runOffline(elapsedS); return { ...r, worst_death_id: r.worst_death?.run_id, worst_death: undefined }; }
  runOfflineSlice(elapsedS: number, _last: boolean): ReturnReport { return this.runOfflineQuick(elapsedS); }
  /** RUNS_UI: an absence's runs are `away` in the log; the first slice after anything else opens a new absence */
  private offlineVia = false;
  private inAbsence = false;
  runOffline(elapsedS: number): ReturnReport {
    const L = this.s.lineage;
    if (!this.inAbsence) { L.absences = (L.absences ?? 0) + 1; this.inAbsence = true; }
    L.clock_s = (L.clock_s ?? 0) + Math.floor(elapsedS);
    this.offlineVia = true;
    try { return this.runOfflineInner(elapsedS); } finally { this.offlineVia = false; }
  }
  private runOfflineInner(elapsedS: number): ReturnReport {
    const L = this.s.lineage;
    const bountyD = L.best_depth >= 1 ? L.best_depth + 2 : 0;   // Cut 20 §5 stand-in: the night's bounty floor as the absence began (none before a best)
    let budget = Math.max(0, Math.floor(elapsedS)); let runs = 0, stall = 0, sampled = false, turnsTotal = 0;
    let rested = 0, banked = 0, returned = 0, stalled = 0; const bonesFound: string[] = []; const exits: ExitLine[] = [];
    // Cut 2 §1: the rest (or wake) after each expedition comes out of the same clock; what is left waits in camp
    const rest = (run: Run): void => { if (run.line) { exits.push(run.line); while (exits.length > EXITS_CAP) exits.shift(); } const r = Math.min(run.rest_s, budget); rested += r; budget -= r; L.rest_left_s = run.rest_s - r; if (run.exit === "bank") banked++; else if (run.exit === "return") { returned++; if (run.stalled) stalled++; } for (const b of run.bonesFound) bonesFound.push(`heir ${b.heir} · D${b.depth} · ${b.items} items`); };
    if ((L.rest_left_s ?? 0) > 0) { const r = Math.min(L.rest_left_s ?? 0, budget); rested += r; budget -= r; L.rest_left_s = (L.rest_left_s ?? 0) - r; }
    const learned: string[] = [], bests: string[] = [], found: InvItem[] = [], deaths: Record<string, number> = {}; let marks = 0; let reel: Highlight[] = [];
    const tamed: string[] = [], hatched: string[] = [], lost: string[] = []; let xpGained = 0, levelUps = 0, renownGained = 0, ranksUp = 0;
    const salvMap: Record<string, { n: number; gold: number }> = {}, spentMap: Record<string, { n: number; gold: number }> = {};
    const take = (r: { tamed: string[]; lost: string[]; hatched: string[]; xp: number; levels: number[]; salvaged: { kind: string; n: number; gold: number }[]; score: number; ranks: number[]; spent: { kind: string; n: number; gold: number }[] }): void => {
      tamed.push(...r.tamed); lost.push(...r.lost); hatched.push(...r.hatched); xpGained += r.xp; levelUps += r.levels.length; renownGained += r.score; ranksUp += r.ranks.length;
      for (const s of r.salvaged) { const m = (salvMap[s.kind] ??= { n: 0, gold: 0 }); m.n += s.n; m.gold += s.gold; }
      for (const s of r.spent) { const m = (spentMap[s.kind] ??= { n: 0, gold: 0 }); m.n += s.n; m.gold += s.gold; }   // Cut 13 §3
    };
    if (this.pending) { const run = this.pending; this.pending = null; take(this.settle(run, true, this.prefPick(run))); this.live = null; }
    let worst: { id: number; depth: number } | null = null;
    let stalledRun: number | null = null;                                  // Cut 13 §1: a stall's verdict opens from the report when no death does
    let deepest = 0;                                                       // the send's deepest floor (the report's `deepest` tile)
    if (this.live && !this.live.over) { const ctx = this.ctx(); while (!this.live.over && budget > 0) { simTurn(this.live, ctx); budget--; this.s.totalTurns += 10; } if (this.live.over) { const r = this.settle(this.live, true, this.prefPick(this.live)); this.settled.add(this.live.id); take(r); learned.push(...r.facts); bests.push(...r.bests); marks += r.marks; runs++; if (this.live.exit === "death") { deaths[this.live.cause ?? "?"] = 1; worst = { id: this.live.id, depth: this.live.depth }; } reel.push(...this.live.hl); rest(this.live); this.live = null; } }
    while (budget > 0 && stall < 20 && runs < 80 && !L.ended) {
      const run = this.startRun(); const ctx = this.ctx();
      while (!run.over && budget > 0) { simTurn(run, ctx); budget--; this.s.totalTurns += 10; }
      if (!run.over) { this.live = run; break; }
      runs++; turnsTotal += run.turn; deepest = Math.max(deepest, run.depth);
      const r = this.settle(run, true, this.prefPick(run)); this.settled.add(run.id); take(r);
      learned.push(...r.facts); bests.push(...r.bests); marks += r.marks;
      stall = r.facts.length || r.bests.length ? 0 : stall + 1;
      this.home = run.exit === "death" || r.bests.some((b) => /^D\d+$/.test(b)) ? 0 : this.home + 1;
      for (const p of run.picked) if ((WEAPONS[p.kind] || ARMOUR[p.kind] !== undefined) && !found.some((f) => f.kind === p.kind)) found.push(p);
      if (run.exit === "death") { deaths[run.cause ?? "?"] = (deaths[run.cause ?? "?"] ?? 0) + 1; if (!worst || run.depth < worst.depth) worst = { id: run.id, depth: run.depth }; }
      else if (run.stalled) stalledRun ??= run.id;
      reel.push(...run.hl);
      rest(run);
    }
    if (budget > 0 && stall >= 20 && turnsTotal > 0) { const extra = Math.floor(budget / (turnsTotal / runs)); if (extra > 0) { sampled = true; const scale = (runs + extra) / runs; for (const k of Object.keys(deaths)) deaths[k] = Math.round(deaths[k] * scale); runs += extra; } }
    reel = reel.sort((a, b) => b.score - a.score).slice(0, 5);
    const pending: string[] = [];
    for (const [u, c] of Object.entries(UNLOCK_COST)) if (!L.unlocks.includes(u) && L.marks >= c && this.unlockVisible(u)) pending.push(`unlock ${u} (${c})`);
    const worstDeath = worst ? this.death(worst.id) : stalledRun !== null ? this.death(stalledRun) : undefined;
    if (worstDeath?.verdict === "gap") pending.push(`patch D${worstDeath.depth} ${worstDeath.cause}`);
    if (!pending.length) pending.push("rules");
    const verdictStall = this.stall(this.home);
    const restLeft = L.rest_left_s ?? 0;
    const live = this.peek(); L.rest_left_s = restLeft;                    // `live` is a peek, not a send: the rest and the trait offer stand
    return { elapsed_s: elapsedS, runs, sampled, learned, bests, found, deaths: Object.entries(deaths).map(([cause, n]) => ({ cause, n })).sort((a, b) => b.n - a.n), pending, reel, marks_earned: marks, worst_death: worstDeath, live, tamed, hatched, lost, xp: { class: L.class, gained: xpGained, level_ups: levelUps },
      salvaged: Object.entries(salvMap).map(([kind, v]) => ({ kind, ...v })), renown: { gained: renownGained, rank: L.rank, ranks_up: ranksUp },
      spent: Object.entries(spentMap).map(([kind, v]) => ({ kind, ...v })),   // Cut 13 §3
      rested_s: rested, banked, returned, stalled, bones_found: bonesFound, stall: verdictStall, deepest, exits,
      lanes: FAKE_BANDS.filter(([a]) => deepest >= a).map(([a, b]) => { const n = biomeOn(a, this.s.rules.route); return `D${a}–${b} · the ${n[0].toUpperCase()}${n.slice(1)}`; }),   // Cut 26 §2 stand-in
      shelved: (() => { const m: Record<string, number> = {}; for (const x of exits) for (const y of x.shelved ?? []) m[y.kind] = (m[y.kind] ?? 0) + y.n; const o = Object.entries(m).map(([kind, n]) => ({ kind, n })); return o.length ? o : undefined; })(),   // Cut 21 §2 stand-in (the kept exits')
      bounty: bountyD <= 0 ? undefined : { depth: bountyD, taken: deepest >= bountyD, gold: deepest >= bountyD ? Math.max(0, ...exits.map((x) => x.kept)) : 0 } };   // Cut 20 §5 stand-in
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
    return { row: at, fired: home, text: `R${at + 1} ${row.verb.v} ended ${home} runs, none past D${depth}`, patches: cands.slice(0, 3) };
  }
  private unlockVisible(u: string): boolean {
    const L = this.s.lineage;
    const pre = UNLOCK_PREREQ[u]; if (pre && !L.unlocks.includes(pre)) return false;
    L.ledger = this.ledger();
    return UNLOCKS[u]?.gate?.(L) ?? true;
  }

  /** QA 23ed91f: the fake's patch deltas are already its forecast's; the same patches, not pending. */
  // QA 524827b stand-in: the whole-run move mirrors the reach (death falls as reach rises); nothing harms
  deathDeltas(runId: number): Patch[] { return this.death(runId).patches.map((p) => ({ ...p, camp_pending: false, whole: { reach: p.forecast_delta, reach_pm: 0.05, death: -p.forecast_delta / 2, death_pm: 0.05 } })); }
  death(runId: number): Death {
    if (this.lastDeath[runId]) return this.lastDeath[runId];
    const log = this.s.logs[runId]; const L = this.s.lineage;
    if (!log) return { run_id: runId, depth: 0, cause: "unknown", margin: "?", verdict: "dice", baseline: 0, trace: { turns: [] }, patches: [], morgue: "" };
    const known = new Set(log.known);
    const replay = makeRun(runId, log.heir, log.seed, log.cls, log.trait, known, [], this.ctx(log.rules)); runToEnd(replay, this.ctx(log.rules));
    const N = 8;
    // Cut 13 §1: a stall's checkpoint is the guard — a candidate survives when the replay leaves the floor or comes home unstalled
    const survive = (rules: RuleSet): number => { let ok = 0; for (let i = 0; i < N; i++) { const r = this.simOne(hash(`p:${log.seed}:${i}`), rules, known, log.cls, log.trait); if (r.depth > log.depth || (log.stalled ? !r.stalled : r.exit !== "death")) ok++; } return ok / N; };
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
    let verdict: Death["verdict"] = log.stalled ? "stall" : [...rootPatches, ...scored].some((p) => p.survive >= 0.6) ? "gap" : "dice";
    // Cut 25 §2 stand-in: a `return` row under the set's first attack row never fired — the `order` verdict (`R5 under R2`), its move leading
    const atk = log.rules.rows.findIndex((r) => r.verb.v === "attack"), home = log.rules.rows.findIndex((r, i) => i > atk && r.verb.v === "return");
    const order = !log.stalled && atk >= 0 && home > atk ? { cause_row: home, order_over: atk } : undefined;
    if (order) verdict = "order";
    // Cut 11 §4: a candidate under the bar is still named, dimmed (`survives 40% · below bar`); `dice` never shows an empty list
    let patches: Patch[] = [...rootPatches, ...scored].map((p) => (p.survive < 0.6 ? { ...p, below_bar: true } : p));
    // Cut 11 §2: a locked-condition root — the unlock as a pseudo-patch (`◆2 cond: alert`, insert_at −1): the client buys, then inserts the row
    if (!L.unlocks.includes("cond_alert") && replay.alert >= 1) { const row: Row = { conds: [{ k: "alert>=", n: 3 }], verb: { v: "return" } }; const sv = survive({ rows: [row, ...log.rules.rows] }); patches.push({ row, insert_at: -1, survive: sv, forecast_delta: Math.round((sv - base) * 100) / 100, root: { text: `◆${UNLOCK_COST.cond_alert} cond: alert` }, ...(sv < 0.6 ? { below_bar: true } : {}) }); }
    patches = patches.slice(0, 4);
    if (order) patches.unshift({ row: log.rules.rows[order.cause_row], insert_at: order.order_over, survive: Math.min(1, base + 0.5), forecast_delta: 0.05, moves_from: order.cause_row });
    // QA 778fa1b stand-in: a patch whose row ends the run says so (`exits`) — the client names its cost (`return early`)
    patches = patches.map((p) => (!p.remove && (p.row.verb.v === "return" || p.row.verb.v === "bank") ? { ...p, exits: true } : p));
    const margin = log.stalled ? "no path" : `${Math.max(1, log.hpMargin)} hp short`;   // a stall's headline: the guard's reason, not an hp margin
    const morgue = [`riddle · seed ${L.seed} · heir ${log.heir} · ${log.cls} · ${log.trait}`, `D${log.depth} · ${replay.cause ?? "?"} · ${margin} · ${verdict} · turn ${log.turns}`, "", ...log.rules.rows.map((r, i) => `R${i + 1} ${rowText(r)}`), "", ...replay.trace.map((t) => `t${t.t * 10} R${t.row + 1} ${verbText(t.verb)} hp${t.hp} foes${t.foes}${t.telegraphs.length ? " " + t.telegraphs.join(",") : ""}`), ...chain.map((c) => `← ${c.text} t${c.t * 10}`)].join("\n");
    const d: Death = { run_id: runId, depth: log.depth, cause: log.stalled ? "stalled" : replay.cause ?? log.cause ?? "?", margin, verdict, baseline: base, trace: { turns: scaleTrace(replay.trace) }, patches, morgue, line: log.line ?? replay.line, rules: log.rules,
      chain: chain.length ? chain.map((c) => ({ text: c.text, t: c.t * 10, depth: c.depth })) : undefined,
      notes: (log.notes ?? replay.notes).slice(-2).filter((n) => !n.includes("saved him")),   // Cut 13 §4; never a `saved him` (QA 92eb880)
      ...(order ?? {}),
      ...(verdict === "dice" && patches.length && patches.every((p) => p.survive <= base) ? { nothing_beats_base: true } : {}) };
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
  /** Cut 15 §2: the same gates as `buy`, paid in gold at `UnlockInfo.gold`; marks untouched; ledger `unlock <id>`. */
  buyUnlockGold(unlock: string): Lineage {
    const L = this.s.lineage; const cost = UNLOCK_COST[unlock];
    if (cost === undefined) throw new Error("unknown unlock");
    if (L.unlocks.includes(unlock)) throw new Error("already owned");
    if (cost === 0) throw new Error("not for gold");
    if (!this.unlockVisible(unlock)) throw new Error("prerequisite missing");
    const u = UNLOCKS[unlock]; if (u?.gate && !u.gate(L)) throw new Error(`needs ${u.needs}`);
    const price = ROW_MULT[unlock] ? ROW_MULT[unlock] * kitUnit(L.best_depth) : goldPrice(cost, this.goldBuys);   // Cut 23 §1: rows on the forge's ladder
    if (L.gold < price) throw new Error("not enough gold");
    this.gold(-price, `unlock ${unlock}`); this.goldBuys++;
    L.unlocks.push(unlock); if (unlock === "party_slot_2") L.party_slots = 2; if (unlock === "party_slot_3") L.party_slots = 3;
    return this.lineage();
  }
  unlockDeltas(): UnlockInfo[] { return this.unlocks(); }
  unlocks(): UnlockInfo[] {
    const L = this.s.lineage;
    const nextOf: Record<string, string> = Object.fromEntries(Object.entries(UNLOCK_PREREQ).map(([n, p]) => [p, n]));
    L.ledger = this.ledger();
    // Cut 9 §2: every card that is not `available` says why — the gate, the missing prerequisite, or `◆2 more` (the core's `needs`)
    return Object.entries(UNLOCKS).map(([id, u]) => {
      const owned = L.unlocks.includes(id); const met = u.gate?.(L) ?? true;
      const needs = owned ? undefined : !met ? u.needs : !this.unlockVisible(id) ? UNLOCK_PREREQ[id]?.replace(/_/g, " ") : L.marks < u.cost ? `◆${u.cost - L.marks} more` : undefined;
      return { id, cost: u.cost, owned, available: !owned && needs === undefined, needs, gold: owned ? 0 : goldPrice(u.cost, this.goldBuys),   // gold: Cut 15 §2
      delta: TACTIC_CARDS.includes(id) && !owned ? ((Math.abs(hash(id)) % 9) - 2) / 100 : undefined,   // delta: Cut 4 §9 stand-in (`reach +4%` on a card)
      ...(TACTIC_CARDS.includes(id) && !owned ? { pm: 0.03 } : {}),                                     // Cut 13 §5: its half-width — within it the client reads `reach ~0`
      rows: UNLOCK_ROWS[id],                                                                             // Cut 6 §6
      ...(TACTIC_CARDS.includes(id) ? { insert_at: this.cardInsertAt() } : {}),                          // Cut 12 §1
      ...(CARD_SITUATION[id] ? { situation: CARD_SITUATION[id] } : {}),                                  // Cut 18 §5: the foe tag the card answers
      // QA e75ec29 / a946e04 stand-in: a card joins the set on its buy only when measured not to hurt there, under 3 card rows in the set
      ...(TACTIC_CARDS.includes(id) && !owned && ((Math.abs(hash(id)) % 9) - 2) >= 0 && this.s.rules.rows.filter((r) => r.verb.v === "tactic").length < 3 ? { auto_insert: true } : {}),
      // QA a946e04 stand-in: the chain's next step and its prices (`row5` → `row6`), and this card's price after one more gold buy
      ...(nextOf[id] && UNLOCKS[nextOf[id]] && !L.unlocks.includes(nextOf[id]) ? { next: { id: nextOf[id], cost: UNLOCKS[nextOf[id]].cost, gold: goldPrice(UNLOCKS[nextOf[id]].cost, this.goldBuys), gold_after_gold: goldPrice(UNLOCKS[nextOf[id]].cost, this.goldBuys + 1) } } : {}),
      ...(CARD_CARRIES[id] ? { carries: CARD_CARRIES[id] } : {}),                                      // Cut 23 §3
      ...(ROW_MULT[id] && !owned ? { gold: ROW_MULT[id] * kitUnit(L.best_depth) } : {}),                  // Cut 23 §1: a row slot on the forge's ladder
      gold_next: owned ? 0 : goldPrice(u.cost, this.goldBuys + 1) };
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
  exportRules(): string { const r = this.s.rules.route ?? []; return [...(r.length ? [`⑂ ${r.join(" ")}`] : []), ...this.s.rules.rows.map(rowText)].join("\n"); }   // Cut 26 §2: the route rides the export (`⑂ 4`)
  importRules(text: string): RuleSet { const set = parseRules(text); this.setRules(set); return JSON.parse(JSON.stringify(this.s.rules)) as RuleSet; }
}

export function createFakeEngine(): Engine { return new FakeEngine(); }

/** QA 92eb880 — the core's `RuleSet::shadowed_by` (every condition usable here): row A above shadows B when A's conditions
 *  hold whenever B's do (plus a foe in view for a striking verb or a foe condition) and A always acts (`return`, `hold`)
 *  or is B's own verb. `{}` when no row is shadowed (the wire omits the field). */
function shadowField(rows: Row[]): { shadowed_by?: (number | null)[] } {
  const strict = (k: string): number => (["hp<", "foe_hp<", "party_hp<", "self_hp<"].includes(k) ? -1 : ["hp>", "self_hp>", "turns>", "foes>=", "adj>=", "floor_seen>=", "depth>=", "alert>=", "loot>="].includes(k) ? 1 : 0);
  const implies = (b: Row["conds"][number], a: Row["conds"][number]): boolean => {
    if (b.k !== a.k || (b.t ?? null) !== (a.t ?? null)) return false;
    const d = strict(a.k); const nb = b.n ?? 0, na = a.n ?? 0;
    return d < 0 ? nb <= na : d > 0 ? nb >= na : nb === na;
  };
  const scope = (r: Row): string => JSON.stringify(r.conds.filter((c) => c.k === "party" || (c.k === "on_see" && c.t === "den") || (c.k === "foe_tag" && c.t === "thief")).map((c) => [c.k, c.t ?? "", c.n ?? 0]).sort());
  const shadows = (a: Row, b: Row): boolean => {
    const same = a.verb.v === b.verb.v && (a.verb.a ?? null) === (b.verb.a ?? null) && scope(a) === scope(b);
    if (!(a.verb.v === "return" || a.verb.v === "hold" || same)) return false;
    const eff = [...b.conds];
    if (["attack", "shield_bash", "cleave", "backstab", "taunt"].includes(b.verb.v) || b.conds.some((c) => c.k === "foe_tag" || c.k === "foe_hp<")) eff.push({ k: "foes>=", n: 1 });
    return a.conds.every((ca) => eff.some((cb) => implies(cb, ca)));
  };
  const out = rows.map((b, j) => { const i = rows.slice(0, j).findIndex((a) => shadows(a, b)); return i >= 0 ? i : null; });
  return out.some((x) => x !== null) ? { shadowed_by: out } : {};
}

// ---------------------------------------------------------------- Cut 28 stand-ins (the core's oaths, attribution, calm, luck)
// Grafted onto the fake's prototype so the class above stays as it was: each wraps the method the core extends and adds the Cut 28 fields.
type Fk = { s: { lineage: Lineage; rules: RuleSet; runCounter: number; oath28?: { board: Oath[]; sworn: string | null; titles: string[]; done: number; sent?: string } };
            lineage(): Lineage; forecastVs(prev: RuleSet): ForecastVs; gold(delta: number, why: string): void; fcRefined: boolean };
/** The stand-in's pool: (kind, chips of a best depth, reward) — the core's `oath::POOL` in miniature. */
const OATH_POOL28: { kind: string; chips: (b: number) => string[]; reward: (b: number) => OathReward }[] = [
  { kind: "lean", chips: (b) => [`reach D${b}`, "no rest"], reward: () => ({ kind: "card", id: "gas_step", label: "card: gas step" }) },
  { kind: "tamer", chips: () => ["tame", "a new kind"], reward: () => ({ kind: "slot", id: "party_slot_2", label: "+1 party" }) },
  { kind: "fire", chips: () => ["Warlord", "fire"], reward: () => ({ kind: "title", id: "Firebrand", label: "title: Firebrand" }) },
  { kind: "slayer", chips: () => ["slay", "Mother"], reward: () => ({ kind: "waystone", id: "14", label: "waystone D14" }) },
  { kind: "bold", chips: (b) => [`reach D${b + 1}`, "no return"], reward: () => ({ kind: "verb", id: "throw", label: "verb: throw" }) },
];
function fakeBoard28(e: Fk): { board: Oath[]; sworn: string | null; titles: string[]; done: number } {
  const L = e.s.lineage; const st = (e.s.oath28 ??= { board: [], sworn: null, titles: [], done: 0 });
  const b = Math.max(1, L.best_depth), price = Math.max(100, Math.round((100 + 25 * b) / 10) * 10);
  while (st.board.length < 3) {
    const i = (hash(`oath:${L.seed}:${st.done + st.board.length}`) + st.board.length) % OATH_POOL28.length;
    const p = OATH_POOL28[(i + st.board.length) % OATH_POOL28.length];
    if (st.board.some((o) => o.kind === p.kind)) { st.done++; continue; }
    const chips = p.chips(b);
    st.board.push({ id: `${p.kind}:${st.done + st.board.length}`, kind: p.kind, chips, text: chips.join(" · "), reward: p.reward(b), price,
      ...(p.kind === "slayer" ? { boss: "bloat_mother", depth: 13, counter: L.facts.some((f) => f.startsWith("boss:bloat_mother:counter")) ? "mother: fire" : "mother: ?" } : {}) });
  }
  return st;
}
{
  const P = FakeEngine.prototype as unknown as Record<string, (...a: unknown[]) => unknown>;
  const lin = P.lineage; P.lineage = function (this: Fk): Lineage {
    const L = lin.call(this) as Lineage; const st = fakeBoard28(this);
    L.oaths = st.board.map((o) => ({ ...o, ...(o.id === st.sworn ? { sworn: true } : {}) })); L.oath = st.sworn; if (st.titles.length) L.titles = [...st.titles];
    // Cut 28b: the board opens at the first wall (the Warlord's floor reached) or once one is sworn or kept
    L.oath_open = L.best_depth >= 8 || !!st.sworn || st.titles.length > 0;
    const mother = L.facts.some((f) => f.startsWith("boss:bloat_mother:counter"));
    L.walls = [{ boss: "goblin_warlord", title: "Warlord", depth: 8, slain: L.best_depth > 8, known: L.facts.some((f) => f.startsWith("boss:goblin_warlord:counter")), fact: "warlord: attack boss", counter: "attack boss" },
      { boss: "bloat_mother", title: "Mother", depth: 13, slain: L.best_depth > 13, known: mother, fact: mother ? "mother: fire" : "mother: ?", ...(mother ? { counter: "throw fire, boss" } : { learn: "meet her" }) }];
    if (L.bounty) L.bounty = { ...L.bounty, pays: "$×2 · item", needs: L.bounty.depth === 13 ? "reach" : "reach", ...(L.bounty.depth === 13 ? { boss: "bloat_mother", fact: mother ? "mother: fire" : "mother: ?" } : {}) };
    return L;
  };
  P.swearOath = function (this: Fk, id: unknown): Lineage {
    const st = fakeBoard28(this); const o = st.board.find((x) => x.id === id); if (!o) throw new Error("no such oath");
    if (st.sworn === o.id) return this.lineage();
    if (this.s.lineage.gold < o.price) throw new Error("not enough gold");
    if (st.sworn) { const was = st.board.find((x) => x.id === st.sworn); if (was) this.gold(Math.floor(was.price / 2), `forswear ${was.text}`); }
    this.gold(-o.price, `oath ${o.text}`); st.sworn = o.id; return this.lineage();
  };
  P.forswearOath = function (this: Fk): Lineage {
    const st = fakeBoard28(this); const o = st.board.find((x) => x.id === st.sworn);
    if (o) this.gold(Math.floor(o.price / 2), `forswear ${o.text}`); st.sworn = null; return this.lineage();
  };
  const share28 = (e: Fk, n: number): OathShare | undefined => {
    const st = fakeBoard28(e); const o = st.board.find((x) => x.id === st.sworn); if (!o) return undefined;
    const drinks = e.s.rules.rows.filter((r) => r.verb.v === "rest").length, returns = e.s.rules.rows.filter((r) => r.verb.v === "return").length;
    const base = o.kind === "lean" ? 0.45 - 0.15 * drinks : o.kind === "bold" ? 0.4 - 0.15 * returns : o.kind === "fire" ? (e.s.rules.rows.some((r) => r.verb.v === "throw") ? 0.3 : 0.02) : o.kind === "slayer" ? 0.04 : 0.2;
    const share = Math.max(0, Math.min(1, base)); const pm = 1.96 * Math.sqrt(share * (1 - share) / n);
    const steps = o.kind === "fire" || o.kind === "slayer" ? [{ k: `D${o.depth ?? 13}`, share: Math.min(1, share * 5) }, { k: "met", share: Math.min(1, share * 3) }, ...(o.kind === "fire" ? [{ k: "burned", share: Math.min(1, share * 1.5) }] : [])]
      : o.kind === "lean" || o.kind === "bold" ? [{ k: o.chips[0].replace(/^reach /, ""), share: Math.min(1, share * 1.6) }] : [];
    return { id: o.id, text: o.text, share, pm, night: 1 - Math.pow(1 - share, 16), ...(steps.length ? { steps } : {}) };
  };
  for (const [name, n] of [["forecast", 20], ["forecastEstimate", 8], ["forecastRefine", 100]] as const) {
    const f = P[name]; P[name] = function (this: Fk): Forecast { const r = f.call(this) as Forecast; const o = share28(this, n); return o ? { ...r, oath: o } : r; };
  }
  const vs = P.forecastVs; P.forecastVs = function (this: Fk, prev: unknown): ForecastVs {
    const r = vs.call(this, prev) as ForecastVs; const st = fakeBoard28(this);
    if (!st.sworn) return r;
    const d = (s: RuleSet): number => s.rows.filter((x) => x.verb.v === "rest" || x.verb.v === "return").length;
    return { ...r, oath: { delta: -0.15 * (d(this.s.rules) - d(prev as RuleSet)), pm: 0.06 } };
  };
  /** The core's `forecastMove`: the stand-in has no state at the send, so the whole is the rows' move, with a `party` part while the kennel is empty. */
  P.forecastMove = function (this: Fk, prev: unknown): ForecastMove | null {
    const rowsMove = (this as unknown as { forecastVs(p: RuleSet): ForecastVs }).forecastVs(prev as RuleSet);
    const rows = JSON.stringify((prev as RuleSet).rows.map((r) => [r.conds, r.verb])) !== JSON.stringify(this.s.rules.rows.map((r) => [r.conds, r.verb]));
    const parts: MovePart[] = [];
    if (!this.s.lineage.party.length && this.s.runCounter > 0) {
      const zero = (m?: number | VsMove): VsMove => ({ delta: 0, pm: 0, base: typeof m === "object" ? m.base : undefined });
      parts.push({ kind: "party", text: "party −1 jackal", move: { depths: rowsMove.depths.map((d) => ({ ...d, delta: -0.04, pm: 0.03 })), bank: { delta: -0.05, pm: 0.04 }, death: { delta: 0.06, pm: 0.04 }, return: zero(rowsMove.return), gold: zero(rowsMove.gold), sims: rowsMove.sims, refined: rowsMove.refined } });
    }
    if (rows) parts.push({ kind: "rows", text: "rows", move: rowsMove });
    if (!parts.length) return null;
    const sum = (k: "bank" | "death"): VsMove => ({ delta: parts.reduce((a, p) => a + ((p.move[k] as VsMove | undefined)?.delta ?? 0), 0), pm: 0.05 });
    const whole: ForecastVs = { ...rowsMove, bank: sum("bank"), death: sum("death") };
    const head = (p: MovePart): number => Math.max(Math.abs((p.move.bank as VsMove | undefined)?.delta ?? 0), Math.abs((p.move.death as VsMove | undefined)?.delta ?? 0));
    const lead = [...parts].sort((a, b) => head(b) - head(a))[0].kind;
    return { whole, parts, lead, rows, state: parts.some((p) => p.kind !== "rows" && p.kind !== "route"), sims: rowsMove.sims ?? 20, refined: !!rowsMove.refined };
  };
  const step = P.step; P.step = function (this: Fk, ticks: unknown): StepResult {
    const r0 = step.call(this, ticks) as StepResult;
    // Cut 28b: the sworn oath's fate, before the exit — `no rest` broken at a rest row, `no return` at a return, kept on a bank, else missed
    const st28 = fakeBoard28(this), sw = st28.board.find((x) => x.id === st28.sworn), xi = r0.events.findIndex((e) => e.k === "exit");
    let r = r0;
    if (sw && xi >= 0) {
      const x = r0.events[xi] as Extract<Ev, { k: "exit" }>;
      const tool = sw.kind === "lean" ? "rest" : sw.kind === "bold" ? "return" : "";
      const used = tool ? r0.events.find((e): e is Extract<Ev, { k: "rule" }> => e.k === "rule" && e.verb.v === tool) : undefined;
      const ev: Ev = used ? { t: used.t, k: "oath", kept: false, row: used.row, cause: tool }
        : x.tier === "bank" ? { t: x.t, k: "oath", kept: true, row: -1, cause: "" } : { t: x.t, k: "oath", kept: false, row: -1, cause: "" };
      r = { ...r0, events: [...r0.events.slice(0, xi), ev, ...r0.events.slice(xi)] };
    }
    // calm: the step's ticks with no rule row ≥ 0 (a chore is calm), no attack/hurt/beat event
    const loud = new Set(r.events.filter((e) => (e.k === "rule" && e.row >= 0 && e.verb.v !== "pick_up" && e.verb.v !== "rest") || ["attack", "hurt", "die", "telegraph", "steal", "descend", "fact", "callout", "tame", "level", "exit"].includes(e.k)).map((e) => e.t));
    const ts = [...new Set(r.events.map((e) => e.t))].sort((a, b) => a - b); const calm: [number, number][] = [];
    for (const t of ts) { if (loud.has(t)) continue; const last = calm[calm.length - 1]; if (last && t - last[1] <= 10) last[1] = t; else calm.push([t, t]); }
    return calm.length ? { ...r, calm } : r;
  };
  const fold = P.fold; P.fold = function (this: Fk): FoldLine {
    const r = fold.call(this) as FoldLine; const h = r.step.snapshot.hero;
    const low = r.floors.length > 0 && h.hp * 2 <= h.max_hp;
    return { ...r, hp: h.hp, max_hp: h.max_hp, low, chips: low && !r.chips.includes(`hp ${h.hp}/${h.max_hp}`) ? [...r.chips, `hp ${h.hp}/${h.max_hp}`] : r.chips };
  };
  const death = P.death; P.death = function (this: Fk, id: unknown): Death {
    const d = death.call(this, id) as Death;
    const turns = d.trace.turns.map((t, i) => ({ ...t, max_hp: Math.max(t.hp, 36 - (i > d.trace.turns.length - 3 ? 4 : 0)) }));
    const steps = turns.length > 2 ? [{ t: turns[turns.length - 2].t, max: 32, delta: -4, cause: "hunger" }] : [];
    const luck = d.verdict === "dice" || d.baseline > 0.5 ? { text: `${d.cause.replace(/_/g, " ")} −6 at 6 hp`, t: turns[turns.length - 1]?.t ?? 0, odds: Math.max(1 / 12, 1 - d.baseline), one_in: Math.max(1, Math.round(1 / Math.max(1 / 12, 1 - d.baseline))) } : undefined;
    return { ...d, trace: { ...d.trace, turns, ...(steps.length ? { max_steps: steps } : {}) }, patches: d.patches.map((p) => (p.survive <= d.baseline ? { ...p, no_gain: true } : p)), ...(luck ? { luck } : {}) };
  };
  const off = P.runOffline; P.runOffline = function (this: Fk, s: unknown): ReturnReport {
    const r = off.call(this, s) as ReturnReport; const st = fakeBoard28(this); const lead: ReportLead[] = [];
    const o = st.board.find((x) => x.id === st.sworn);
    if (o) {
      const sh = share28(this, 20)?.share ?? 0; const kept = Math.round(r.runs * sh); const done = kept > 0;
      const tool = o.kind === "lean" ? "rest" : o.kind === "bold" ? "return" : "";
      const row = tool ? this.s.rules.rows.findIndex((x) => x.verb.v === tool) : -1;
      const broken = !done && row >= 0 ? r.runs : 0, cause = broken ? `R${row + 1} ${tool}` : undefined;
      r.oath = { id: o.id, chips: o.chips, text: o.text, runs: r.runs, kept, done, reward: o.reward, price: o.price, ...(broken ? { broken, cause } : {}) };
      lead.push({ k: "oath", text: done ? `oath kept: ${o.text}` : broken ? `oath broken: ${cause} ×${broken}` : `oath: ${o.text} · 0/${r.runs}` });
      if (done) { if (o.reward.kind === "title") st.titles.push(o.reward.id); st.board = st.board.filter((x) => x.id !== o.id); st.sworn = null; st.done++; }
    }
    if (r.stall) lead.push({ k: "plateau", text: `plateau: none past D${this.s.lineage.best_depth}` });
    if (r.worst_death) lead.push({ k: "death", text: `D${r.worst_death.depth} death · ${r.worst_death.verdict}` });
    for (const b of r.bests.slice(0, 1)) lead.push({ k: "record", text: b });
    if (lead.length) r.lead = lead.slice(0, 4);
    return r;
  };
}

// ---------------------------------------------------------------- Cut 29 stand-ins (the core's curriculum, meters, standing orders, tiers)
// Grafted like Cut 28's: the curriculum read off the fake lineage (`systems::SYSTEMS` in miniature), a meter built from the step's own
// events (the core folds its event stream the same way), the standing orders over the fake's own fields, the late sinks.
const SYSTEMS29: [string, string][] = [["send", ""], ["dial", ""], ["headline", ""], ["edit", "first death"], ["death", "first death"],
  ["exits", "first gold home"], ["loadout", "first gold home"], ["unlocks", "first mark"], ["reorder", "first plateau"], ["vs", "first plateau"],
  ["tags", "first foe fact"], ["party", "first stray"], ["cage", "first cage"], ["walls", "meet Warlord"], ["divergence", "meet Warlord"],
  ["forge", "slay Warlord"], ["start", "slay Warlord"], ["route", "D5 fork twice"], ["automations", "meet Lich"],   // (Cut 30: the oath board left the curriculum — the quest board is its successor)
  ["route2", "an oath kept"], ["heir_pick", "an oath kept"], ["class", "second class"],
  // Cut 30 (core `systems.rs`): the packages' arrivals, the buildings, the quest board, the pen (the editor's group waits for it)
  ["storehouse", "first find kept"], ["kennel", "first tame"], ["stances", "meet Captain"], ["tactics", "slay Warlord"], ["bank", "a night's purse"],
  ["quests", "slay Warlord"], ["temperament", "heir 3"], ["tactic2", "meet Lich"], ["pen", "meet Mother"]];
/** Cut 30: the systems the pen brings (the core's `systems::PEN`) — they open with it, not before. */
const PEN30 = ["pen", "edit", "dial", "unlocks", "reorder", "vs", "tags", "walls", "divergence", "route"];
type Fk29 = { s: { lineage: Lineage; rules: RuleSet; sys29?: { open: string[]; fresh: string[]; plateau: boolean; works: string[]; meters: MeterWire[]; insure: boolean } };
              lineage(): Lineage; gold(delta: number, why: string): void };
const emptyMeter = (): MeterWire => ({ seconds: 0, dealt: { hero: 0, pets: 0, foes: 0 }, taken: { hero: 0, pets: 0, foes: 0 }, dps_dealt: { hero: 0, pets: 0, foes: 0 }, dps_taken: { hero: 0, pets: 0, foes: 0 },
  healed: [], hps: 0, time: { fight: 0, travel: 0, chores: 0, rest: 0 }, time_s: { fight: 0, travel: 0, chores: 0, rest: 0 }, rows: [], actions: 0, supplies: {}, gold: 0, gold_per_min: 0, hits_hero: 0, hits_pets: 0, fights: 0 });
/** The core's `meters::fold` over a stretch of events (ticks: the distinct `t`s; the hero is id 0, every other id a foe — the fake has no pets in fights). */
function meter29(evs: Ev[]): MeterWire {
  const m = emptyMeter(); const ts = new Set<number>(); const rows = new Map<number, number>(); let acts = 0;
  for (const e of evs) {
    ts.add(e.t);
    if (e.k === "attack" && e.hit && e.dmg > 0) { if (e.src === 0) m.dealt.hero += e.dmg; else m.dealt.foes += e.dmg; m.time.fight++; }
    if (e.k === "hurt" && e.dmg > 0) { if (e.id === 0) { m.taken.hero += e.dmg; m.hits_hero++; } else m.taken.foes += e.dmg; }
    if (e.k === "heal") { const h = m.healed.find((x) => x.src === e.src); if (h) h.total += e.amount; else m.healed.push({ src: e.src, total: e.amount, per_s: 0 }); }
    if (e.k === "rule") { rows.set(e.row, (rows.get(e.row) ?? 0) + 1); acts++; if (e.verb.v === "rest") m.time.rest++; }
    if (e.k === "use") m.supplies[e.item] = (m.supplies[e.item] ?? 0) + 1;
    if (e.k === "exit") m.gold += e.loot_kept;
  }
  const ticks = ts.size ? Math.max(...ts) - Math.min(...ts) + 1 : 0; const sec = Math.max(0.1, ticks / 10); const r3 = (x: number): number => Math.round(x * 1000) / 1000;
  m.time.travel = Math.max(0, ticks - m.time.fight - m.time.rest); m.seconds = ticks / 10;
  m.time_s = { fight: m.time.fight / 10, travel: m.time.travel / 10, chores: 0, rest: m.time.rest / 10 };
  m.dps_dealt = { hero: r3(m.dealt.hero / sec), pets: 0, foes: r3(m.dealt.foes / sec) }; m.dps_taken = { hero: r3(m.taken.hero / sec), pets: 0, foes: r3(m.taken.foes / sec) };
  for (const h of m.healed) h.per_s = r3(h.total / sec);
  m.hps = r3(m.healed.reduce((a, h) => a + h.total, 0) / sec); m.actions = acts; m.fights = m.time.fight > 0 ? 1 : 0;
  m.rows = [...rows.entries()].sort((a, b) => a[0] - b[0]).map(([row, fires]) => ({ row, fires, share: r3(fires / Math.max(1, acts)) }));
  m.gold_per_min = r3(m.gold / (sec / 60));
  return m;
}
/** UI dev / test knob: `?engine=fake&systems=all` opens the whole curriculum at once (the bots' `open_all`) — the suites that test other
 *  features on a fresh fake lineage; the curriculum's own tests leave it off. */
const DEV_ALL_SYSTEMS = typeof location !== "undefined" && new URLSearchParams(location.search).get("systems") === "all";
/** `?systems=none`: no curriculum on the wire (an older core's lineage) — the client's own reveal ladder alone, as before Cut 29. */
const DEV_NO_SYSTEMS = typeof location !== "undefined" && new URLSearchParams(location.search).get("systems") === "none";
function sys29(e: Fk29): { open: string[]; fresh: string[]; plateau: boolean; works: string[]; meters: MeterWire[]; insure: boolean } {
  const st = (e.s.sys29 ??= { open: DEV_ALL_SYSTEMS ? SYSTEMS29.map(([id]) => id) : ["send", "headline"], fresh: [], plateau: false, works: [], meters: [], insure: true });   // Cut 30: the dial comes with the pen
  const L = e.s.lineage; const met = (d: number): boolean => L.best_depth >= d;
  const hit: Record<string, boolean> = {
    edit: L.graveyard.length > 0 || L.heir > 1, death: L.graveyard.length > 0 || L.heir > 1, exits: L.gold > 0 || (L.gold_ledger ?? []).some((g) => g.delta > 0), loadout: L.gold > 0,
    unlocks: L.marks > 0, reorder: st.plateau, vs: st.plateau, tags: L.facts.some((f) => f.split(":").length === 3 && f.startsWith("foe:")), party: L.facts.includes("stray") || L.party.length + L.kennel.length > 0,
    cage: L.facts.includes("vault"), walls: met(8), divergence: met(8), forge: met(9), start: met(9), route: (L.forks ?? []).length > 0, oaths: st.plateau || met(8),
    automations: met(18), route2: L.unlocks.includes("route2"), heir_pick: L.unlocks.includes("heir_pick"), class: L.unlocks.includes("ranger") || L.unlocks.includes("caster"),
    storehouse: L.vault.length > 0, kennel: L.party.length + L.kennel.length > 0, stances: met(5), tactics: met(9), bank: L.gold >= 500, quests: met(9),
    temperament: L.heir >= 3, tactic2: met(18), pen: met(13),
  };
  // Cut 30: the pen's group opens with the pen (the Mother met), whatever its Cut 29 trigger said
  for (const id of PEN30) hit[id] = hit.pen;
  for (const [id] of SYSTEMS29) if (!st.open.includes(id) && hit[id]) { st.open.push(id); st.fresh.push(id); }
  return st;
}
{
  const P = FakeEngine.prototype as unknown as Record<string, (...a: unknown[]) => unknown>;
  const lin = P.lineage; P.lineage = function (this: Fk29): Lineage {
    const L = lin.call(this) as Lineage; const st = sys29(this);
    L.systems = DEV_NO_SYSTEMS ? undefined : SYSTEMS29.map(([id, trigger]): SystemInfo => ({ id, open: st.open.includes(id), ...(trigger ? { trigger } : {}), ...(st.fresh.includes(id) ? { new: true } : {}) }));
    const met = [8, 13, 18, 23, 28].filter((d) => L.best_depth >= d).length;
    L.tier = L.best_depth >= 1 || met > 0 ? Math.min(6, 1 + met) : 0;
    L.oath_slots = 1 + (L.unlocks.includes("oath_slot_2") ? 1 : 0) + (L.unlocks.includes("oath_slot_3") ? 1 : 0);
    L.sworn = L.oath ? [L.oath] : [];
    L.oath_draw = L.tier >= 2 ? (L.marks >= 2 ? { cost: 2, available: true } : { cost: 2, available: false, needs: `◆${2 - L.marks} more` }) : { cost: 2, available: false, needs: "meet Warlord" };
    const price = Math.round(10 * (100 + 25 * L.best_depth) * Math.pow(1.25, st.works.length) / 10) * 10;
    if (st.works.length) L.works = [...st.works];
    L.commission = { price, label: ["heir's statue", "camp hall", "chronicle wall", "boss trophies"][st.works.length % 4], available: L.gold >= price };
    L.orders = { keep: L.keep_pref, cage: L.vault_pref ?? "weapon", start: L.start ?? 1, repeat: L.repeat ?? true, insure: st.insure } as StandingOrders;
    L.supply_cap = Math.min(8, (L.unlocks.includes("supply_cap_5") ? 5 : 3) + ((L.kit ?? []).find((k) => k.slot === "pack")?.owned ?? 0));
    const fire = this.s.rules.rows.find((r) => r.origin === "player" && r.verb.v === "throw" && (r.verb.a ?? "").startsWith("fire"));
    if (fire && !(L.repeat_kinds ?? []).includes("fire")) L.repeat_added = [{ kind: "fire", row: "throw fire" }];
    if (st.meters.length) L.meters = { runs: st.meters.slice(-2), night: st.meters[st.meters.length - 1] };
    return L;
  };
  const step = P.step; P.step = function (this: Fk29, ticks: unknown): StepResult {
    const r = step.call(this, ticks) as StepResult; const st = sys29(this);
    const run = meter29(r.events);
    const out: StepResult = { ...r, snapshot: { ...r.snapshot, meters: { run, ...(run.fights ? { fight: run } : {}), fighting: run.time.fight > 0 } } };
    if (r.run_over) { st.meters.push(run); if (st.meters.length > 2) st.meters.shift(); }
    if (r.exit_pending) {
      const vault = this.s.lineage.vault;
      const beats = r.exit_pending.items.some((i) => vault.some((v) => v.kind === i.kind && (i.label ?? "") > (v.label ?? "")));
      out.exit_pending = { ...r.exit_pending, decide: beats || r.exit_pending.items.length > 2, ...(r.exit_pending.items[0] ? { note: `kept ${r.exit_pending.items[0].label}` } : {}) };
    }
    return out;
  };
  const off = P.runOffline; P.runOffline = function (this: Fk29, s: unknown): ReturnReport {
    const before = [...sys29(this).open]; const r = off.call(this, s) as ReturnReport; const st = sys29(this);
    st.plateau ||= !!r.stall;
    const opened = sys29(this).open.filter((x) => !before.includes(x));
    return { ...r, night_marks: r.banked || r.returned ? 1 : 0, ...(opened.length ? { systems_opened: opened } : {}), meters: { ...emptyMeter(), seconds: r.elapsed_s, fights: r.runs, gold: r.gold?.home ?? 0 } };
  };
  const death = P.death; P.death = function (this: Fk29, id: unknown): Death {
    const d = death.call(this, id) as Death; if (d.verdict === "stall") return d;
    const m = emptyMeter(); m.seconds = 8; m.dealt.hero = 24; m.taken.hero = 40; m.dps_dealt.hero = 3; m.dps_taken.hero = 5; m.hits_hero = 9; m.fights = 1; m.time.fight = 80; m.time_s.fight = 8;
    return { ...d, fight: m };
  };
  const fc = P.forecast; P.forecast = function (this: Fk29): Forecast { const f = fc.call(this) as Forecast; return f.ends ? { ...f, ends: { ...f.ends, passage: 0 } } : f; };
  P.seenSystems = function (this: Fk29): Lineage { sys29(this).fresh = []; return this.lineage(); };
  P.wallEdit = function (this: Fk29): WallEdit | null { return this.lineage().wall ?? null; };   // Cut 29 §1: the lazy wall search (was `ReturnReport.wall`)
  P.setOrders = function (this: Fk29, o: unknown): Lineage {
    const x = o as StandingOrders; const L = this.s.lineage;
    if (!["best_weapon", "best_armour", "none"].includes(x.keep)) throw new Error("unknown keep_pref");
    L.keep_pref = x.keep; L.vault_pref = x.cage; L.start = x.start; L.repeat = x.repeat; sys29(this).insure = x.insure; return this.lineage();
  };
  P.drawOath = function (this: Fk29): Lineage {
    const L = this.s.lineage; if (L.marks < 2) throw new Error("not enough marks"); L.marks -= 2;
    const st = (this as unknown as Fk).s.oath28; if (st) { st.board = st.board.filter((o) => o.id === st.sworn); st.done++; }
    return this.lineage();
  };
  P.forswearOathId = function (this: Fk29, id: unknown): Lineage { const fs = P.forswearOath; void id; return fs.call(this) as Lineage; };
  P.commission = function (this: Fk29): Lineage {
    const L = this.lineage(); const c = L.commission!; if (!c.available) throw new Error("not enough gold");
    this.gold(-c.price, `forge commission ${c.label}`); sys29(this).works.push(c.label); return this.lineage();
  };
}

// ---- Cut 30 stand-ins (the core's `packages.rs`, `town.rs`): the packages, the town, the tracks, `grew`. Shapes only — the fake's
// rows stay its own; a package move changes the chips, the stance's name and level, never the fake's sims.
const PKGS30: [string, string, string, string][] = [
  ["steady", "Steady", "stance", ""], ["guarded", "Guarded", "stance", "meet Captain"], ["bold", "Bold", "stance", "slay Warlord"], ["hunter", "Hunter", "stance", "meet Warlord"],
  ["boss_focus", "boss focus", "tactic", "slay Warlord"], ["corridor_fighting", "corridor fighting", "tactic", "slay Warlord"], ["kite_archers", "kite archers", "tactic", "slay Warlord"],
  ["thief_guard", "thief guard", "tactic", "slay Warlord"], ["gas_step", "gas step", "tactic", "slay Warlord"], ["pack_break", "pack break", "tactic", "slay Warlord"],
  ["skittish", "skittish", "temperament", "heir 3"], ["unbowed", "unbowed", "temperament", "heir 3"], ["light_hands", "light hands", "temperament", "heir 3"], ["iron_gut", "iron gut", "temperament", "heir 3"]];
const LEVEL_RUNS30 = [10, 40, 120, 300];
type St30 = { stance: string; tactics: string[]; temperament?: string; runs: Record<string, number>; revoked: string[]; bank: number; interest: number; swapped: number; quest: number; qd?: number; qdone?: boolean; qbest?: number };
type Fk30 = Fk29 & { s: Fk29["s"] & { st30?: St30 } };
const st30 = (e: Fk30): St30 => (e.s.st30 ??= { stance: "steady", tactics: [], runs: {}, revoked: [], bank: 0, interest: 0, swapped: -1, quest: 0 });
const lv30 = (runs: number): number => 1 + LEVEL_RUNS30.filter((n) => runs >= n).length;
function packages30(e: Fk30, L: Lineage): Packages {
  const st = st30(e); const met = (d: number): boolean => L.best_depth >= d;
  const owned = (kind: string, trig: string): boolean => trig === "" || (kind === "temperament" ? L.heir >= 3 : trig === "meet Captain" ? met(5) : trig.startsWith("meet") ? met(8) : met(9));
  const all: Package[] = PKGS30.filter(([, , kind]) => kind !== "temperament" || L.heir >= 3).map(([id, name, kind, trig]) => {
    const runs = st.runs[id] ?? 0; const lv = lv30(runs); const own = owned(kind, trig);
    const slot = st.stance === id || st.temperament === id ? 0 : st.tactics.indexOf(id) >= 0 ? st.tactics.indexOf(id) : undefined;
    return { id, name, kind, level: lv, runs, next_at: LEVEL_RUNS30.find((n) => runs < n), ...(slot !== undefined ? { slot } : {}), owned: own, ...(own ? {} : { trigger: trig }), ...(own && lv < 5 ? { level_price: lv + 1 } : {}) };
  });
  const drills = [["goblin_warlord", 9], ["bloat_mother", 14], ["lich", 19]].filter(([, d]) => met(d as number)).map(([boss]) => ({ boss: boss as string, rows: [{ conds: [{ k: "foe_tag", t: "boss" }, { k: "hp>", n: 30 }], verb: { v: "attack", a: "tag:boss" } }], revoked: st.revoked.includes(boss as string), scar: 0 }));
  return { all, stance: st.stance, tactics: st.tactics, tactic_slots: met(9) ? (met(18) ? 2 : 1) : 0, ...(st.temperament ? { temperament: st.temperament } : {}), temperament_open: L.heir >= 3,
    ...(L.heir >= 3 ? { offer: ["skittish", "unbowed", "iron_gut"] } : {}), drills, scars: met(8) && !met(9) ? [["goblin_warlord", 15]] : [], pen_open: DEV_ALL_SYSTEMS || met(13),
    // (a row an earlier one of the same conditions and role always pre-empts is shadowed by it — the core's `shadowed_by`, in miniature)
    // (the core's `row_label`: a package row is named by its package — `stance:steady` → `Steady`; a pen row has none)
    rows: e.s.rules.rows.map((r, j, all) => { const i = all.findIndex((x, k) => k < j && x.verb.v === r.verb.v && JSON.stringify(x.conds) === JSON.stringify(r.conds));
      const o = /^(stance|tactic|temper):(\w+)$/.exec((r.origin as string | undefined) ?? "");
      const pid = o?.[1] === "stance" ? st.stance : o?.[2];   // (the fake's rows stay Steady's whatever the stance: they are named by the one worn)
      return { label: o ? PKGS30.find(([id]) => id === pid)?.[1] ?? pid ?? "" : "", ...(i >= 0 ? { shadowed_by: i } : {}) }; }) };
}
function town30(e: Fk30, L: Lineage): Town {
  const st = st30(e); const order: [string, string, boolean][] = [["blacksmith", "first gold home", L.gold > 0 || L.best_depth > 2], ["storehouse", "first find kept", L.vault.length > 0], ["kennel", "first tame", L.party.length + L.kennel.length > 0], ["bank", "a night's purse", L.gold >= 500]];
  const built = order.filter(([, , b]) => b); const next = order.find(([, , b]) => !b);
  return { buildings: built.map(([id, trigger]) => ({ id, level: 1, day: 0, trigger })), ...(next ? { next: next[0], next_trigger: next[1] } : {}), bank: st.bank, bank_cap: 3000, interest: st.interest,
    ...(L.best_depth >= 9 ? { quest: quest30(st, L) } : {}), quests_done: st.quest };
}
/** The fake's quest: `reach D<n>` drawn one past the record, its progress the deepest floor a run reached since (the record's floor
 *  under it until a run goes), kept once a run reaches it. */
function quest30(st: St30, L: Lineage): Quest {
  st.qd ??= L.best_depth + 1;
  const done = (st.qbest ?? 0) >= st.qd;
  return { goal: `reach D${st.qd}`, reward: ["title", "row", "slot", "card"][st.quest % 4], progress: Math.min(1, (st.qbest ?? Math.max(0, L.best_depth - 1)) / st.qd), done, swap: st.swapped < 0 };
}
function tracks30(e: Fk30, L: Lineage): Track[] {
  const P = packages30(e, L); const T = town30(e, L);
  const tr = (id: string, stages: [string, string, boolean][]): Track => {
    const done = stages.filter(([, , d]) => d); const next = stages.find(([, , d]) => !d);
    // (a numeric trigger carries its progress: the bank's purse against a night's — the core reads the last night's net, the fake $500)
    return { id, stage: done[done.length - 1]?.[0] ?? stages[0][0], stages: done.length, ...(next ? { next: next[0], trigger: next[1] } : {}), ...(next?.[0] === "bank" ? { progress: Math.min(1, L.gold / 500) } : {}) };
  };
  return [
    tr("character", [["Steady", "", true], ["second stance", "meet Captain", L.best_depth >= 5], ["a tactic", "slay Warlord", L.best_depth >= 9], ["pets", "first stray", L.facts.includes("stray")], ["a temperament", "heir 3", L.heir >= 3], ["the pen", "meet Mother", P.pen_open]]),
    tr("items", [["pack of 3", "", true], ["storehouse", "first find kept", T.buildings.some((b) => b.id === "storehouse")], ["blacksmith steps", "first gold home", T.buildings.some((b) => b.id === "blacksmith")], ["a counter packed", "a drill's item", L.best_depth >= 14]]),
    tr("scale", [["one hero", "", true], ["party slot 2", "a second slot", L.unlocks.includes("party_slot_2")], ["waystones", "slay Warlord", L.best_depth >= 9], ["party slots 3–4", "a fourth slot", L.unlocks.includes("party_slot_4")]]),
    tr("town", [["camp", "", true], ...BUILD30.map(([id, trig]): [string, string, boolean] => [id, trig, T.buildings.some((b) => b.id === id)])]),
  ];
}
const BUILD30: [string, string][] = [["blacksmith", "first gold home"], ["storehouse", "first find kept"], ["kennel", "first tame"], ["bank", "a night's purse"]];
{
  const P = FakeEngine.prototype as unknown as Record<string, (...a: unknown[]) => unknown>;
  const lin = P.lineage; P.lineage = function (this: Fk30): Lineage {
    const L = lin.call(this) as Lineage;
    if (DEV_NO_SYSTEMS) return L;   // an older core's wire: no packages, no town, no tracks (the pen open, as before Cut 30)
    L.packages = packages30(this, L); L.town = town30(this, L); L.tracks = tracks30(this, L);
    return L;
  };
  const off = P.runOffline; P.runOffline = function (this: Fk30, s: unknown): ReturnReport {
    if (DEV_NO_SYSTEMS) return off.call(this, s) as ReturnReport;
    const b = this.s.lineage.best_depth; const g0 = this.s.lineage.gold; const st = st30(this);
    const before = this.lineage(); const owned0 = before.packages!.all.filter((p) => p.owned).map((p) => p.id); const lv0 = lv30(st.runs[st.stance] ?? 0);
    const stages0 = (before.tracks ?? []).map((t) => `${t.id}:${t.stage}`); const q0 = before.town?.quest;
    if (st.qdone) { st.qd = undefined; st.qdone = false; st.qbest = undefined; st.quest++; }   // a new quest the day after one was kept
    const r = off.call(this, s) as ReturnReport;
    st.runs[st.stance] = (st.runs[st.stance] ?? 0) + r.runs; st.interest += Math.floor(st.bank * 0.02); st.bank += Math.floor(st.bank * 0.02); st.swapped = -1;
    if (q0 && r.deepest !== undefined) st.qbest = Math.max(st.qbest ?? 0, r.deepest);
    const after = this.lineage();
    const grew: GrewLine[] = [];
    if (this.s.lineage.gold > g0) grew.push({ track: "items", what: `+$${this.s.lineage.gold - g0}` });
    if (this.s.lineage.best_depth > b) grew.push({ track: "scale", what: `best D${this.s.lineage.best_depth}` });
    grew.push({ track: "character", what: "xp" });
    for (const t of after.tracks ?? []) if (!stages0.includes(`${t.id}:${t.stage}`)) grew.push({ track: t.id, what: t.stage });
    const beats: string[] = [];
    const lv1 = lv30(st.runs[st.stance]); if (lv1 > lv0) { beats.push(`${st.stance.toUpperCase()} L${lv1}`); grew.push({ track: "character", what: `${st.stance[0].toUpperCase()}${st.stance.slice(1)} L${lv1}` }); }
    for (const p of after.packages!.all) if (p.owned && !owned0.includes(p.id)) beats.push(`+${p.name}`);
    const q = after.town?.quest;
    if (q?.done && (!q0 || !q0.done)) { beats.push(`QUEST DONE · ${q.goal}`); st.qdone = true; }
    return { ...r, grew, packages: beats };
  };
  const has = (L: Lineage, id: string): Package => { const p = L.packages!.all.find((x) => x.id === id); if (!p) throw new Error("unknown package"); if (!p.owned) throw new Error("not yet"); return p; };
  P.equipPackage = function (this: Fk30, id: unknown, slot: unknown): Lineage {
    const L = this.lineage(); const p = has(L, id as string); const st = st30(this);
    if (p.kind === "stance") st.stance = p.id; else if (p.kind === "temperament") st.temperament = p.id;
    else { if (!L.packages!.tactic_slots) throw new Error("slot closed"); st.tactics = st.tactics.filter((t) => t !== p.id); st.tactics[Math.min(slot as number, L.packages!.tactic_slots - 1)] = p.id; st.tactics = st.tactics.filter(Boolean); }
    return this.lineage();
  };
  P.unequipPackage = function (this: Fk30, id: unknown): Lineage {
    const st = st30(this); if (st.stance === id) throw new Error("stance never empty");
    st.tactics = st.tactics.filter((t) => t !== id); if (st.temperament === id) st.temperament = undefined; return this.lineage();
  };
  P.pickTemperament = function (this: Fk30, id: unknown): Lineage { if (!(this.lineage().packages!.offer ?? []).includes(id as string)) throw new Error("not on offer"); st30(this).temperament = id as string; return this.lineage(); };
  P.spendLevel = function (this: Fk30, id: unknown): Lineage {
    const L = this.lineage(); const p = has(L, id as string); if (!p.level_price) throw new Error("top level"); if (L.marks < p.level_price) throw new Error("not enough marks");
    this.s.lineage.marks -= p.level_price; st30(this).runs[p.id] = p.next_at ?? p.runs; return this.lineage();
  };
  P.revokeDrill = function (this: Fk30, boss: unknown, revoked: unknown): Lineage {
    const st = st30(this); st.revoked = st.revoked.filter((b) => b !== boss); if (revoked) st.revoked.push(boss as string); return this.lineage();
  };
  P.packageOptions = function (this: Fk30): PkgOption[] {
    const L = this.lineage();
    // (plausible moves on 24 paired sims: most inside the noise, one or two clear — a stance that passes more, a tactic that dies less)
    const h = (id: string): number => [...id].reduce((a, c) => (a * 31 + c.charCodeAt(0)) >>> 0, 7) % 1000 / 1000;
    return L.packages!.all.filter((p) => p.owned && p.slot === undefined).map((p): PkgOption => {
      const x = h(p.id), big = p.id === "guarded" || p.id === "boss_focus", bold = p.id === "bold";
      const d_past = big ? 0.26 + x * 0.1 : bold ? 0.06 : (x - 0.5) * 0.06, d_death = bold ? 0.3 : big ? -0.08 : (x - 0.4) * 0.05;
      return { id: p.id, action: "equip", slot: 0, past: 0.3 + d_past, bank: 0.5 + d_past / 2, death: 0.15 + d_death, d_past, d_bank: d_past / 2, d_death };
    });
  };
  P.packageOptionsFor = function (this: Fk30, _sims: unknown, choices: unknown): PkgOption[] {
    const all = P.packageOptions.call(this) as PkgOption[];
    return (choices as [string, number][]).flatMap(([id, slot]) => {
      const option = all.find((o) => o.id === id);
      return option ? [{ ...option, slot }] : [];
    });
  };
  P.bankDeposit = function (this: Fk30, amount: unknown): Lineage {
    const L = this.lineage(); if (!L.town!.buildings.some((b) => b.id === "bank")) throw new Error("no bank yet");
    const n = Math.min(amount as number, this.s.lineage.gold, L.town!.bank_cap - L.town!.bank); if (n <= 0) throw new Error("bank full");
    this.gold(-n, "bank deposit"); st30(this).bank += n; return this.lineage();
  };
  P.bankWithdraw = function (this: Fk30, amount: unknown): Lineage {
    const st = st30(this); const n = Math.min(amount as number, st.bank); if (n <= 0) throw new Error("bank empty"); st.bank -= n; this.gold(n, "bank withdraw"); return this.lineage();
  };
  const isPkg = (r: Row): boolean => /^(stance|tactic|temper|drill):/.test((r.origin as string | undefined) ?? "");
  const STEADY_ROWS = (): Row[] => [
    { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } },
    { conds: [{ k: "depth>=", n: 2 }], verb: { v: "bank" } }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } }].map((r) => ({ ...r, origin: "stance:steady" as RowOrigin }));
  const newL = P.newLineage; P.newLineage = function (this: Fk30, seed: unknown): Lineage {
    // (`?systems=all`: the pen open from the start, the set the player's own — an old save's `custom` stance, edited in place)
    newL.call(this, seed); if (DEV_NO_SYSTEMS || DEV_ALL_SYSTEMS) return this.lineage();
    // a new lineage climbs on Steady's compiled rows (the core's `packages::init`); the other sets start as it
    this.s.rules = { rows: STEADY_ROWS(), name: "fighter" }; this.s.lineage.sets = [0, 1, 2].map(() => ({ rows: STEADY_ROWS(), name: "fighter" }));
    return this.lineage();
  };
  const setR = P.setRules; P.setRules = function (this: Fk30, set: unknown): void {
    const S = set as RuleSet; if (DEV_NO_SYSTEMS) { setR.call(this, S); return; }
    // the pen's rows are checked against the cap alone and sit above every package row, which keep their place among themselves
    const pen = S.rows.filter((r) => !isPkg(r)), pkg = this.s.rules.rows.filter(isPkg);
    setR.call(this, { ...S, rows: pen });
    const rows = [...this.s.rules.rows.map((r) => ({ ...r, origin: "player" as RowOrigin })), ...pkg.map((r) => ({ ...r, conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb } }))];
    this.s.rules = { ...this.s.rules, rows }; this.s.lineage.sets[this.s.lineage.active_set] = JSON.parse(JSON.stringify(this.s.rules)) as RuleSet;
  };
  P.swapQuest = function (this: Fk30): Lineage { const st = st30(this); if (st.swapped >= 0) throw new Error("swapped today"); st.swapped = 1; st.quest++; st.qd = undefined; st.qbest = undefined; return this.lineage(); };
  const death = P.death; P.death = function (this: Fk30, id: unknown): Death {
    const d = death.call(this, id) as Death; if (DEV_NO_SYSTEMS) return d;
    const L = this.lineage(), Pk = L.packages!; if (Pk.pen_open) return d;
    const step = (L.kit ?? []).map((k) => k.next).filter((n): n is NonNullable<typeof n> => !!n && !!n.affordable).sort((a, b) => a.price - b.price)[0];
    const st = Pk.all.find((p) => p.id === Pk.stance)!;
    const boss = /warlord|mother|lich|master|queen|king/.test(d.cause);
    const hunter = Pk.all.find((p) => p.id === "hunter" && p.owned);
    const lever = step ? { kind: "spend", text: step.label } : boss && hunter && Pk.stance !== "hunter" ? { kind: "package", text: "Hunter" }
      : boss ? { kind: "wait", text: "drill next" } : { kind: "wait", text: `${st.name} L${Math.min(5, st.level + 1)}` };
    return { ...d, lever, package: `${st.name} · ${d.cause === "stall" ? "explore" : "attack nearest"}` };
  };
}

// ---- Cut 30.5 stand-ins (the core's `tree.rs`): the works tree — workers that retire chores done by hand, the four tracks as branches,
// the haul chest, the `next` pill, manual sends before the scout. Shapes only: the fake's sims are its own (a send before the scout is one
// run: an absence then yields only the run in flight).
// (id, name, branch, chore, need, price in tenths of a forge unit, fallback age h, post, beat, tip, the chore's system)
const NODES305: [string, string, string, string, number, number, number, string, string, string, string][] = [
  ["quartermaster", "quartermaster", "trunk", "", 0, 0, 0, "crate", "", "packs heal · drill item", ""],
  ["porter", "porter", "trunk", "chest", 3, 0, 0, "mouth", "AUTO HAUL", "hauls home · while away", ""],
  ["scout", "scout", "trunk", "send", 3, 5, 0, "fire", "AUTO SEND", "sends him · each rest", ""],
  ["armourer", "armourer", "trunk", "wear", 2, 10, 24, "storehouse", "AUTO EQUIP", "wears better finds", "storehouse"],
  ["apprentice", "apprentice", "trunk", "forge", 3, 30, 30, "blacksmith", "AUTO FORGE", "buys forge steps", "forge"],
  ["keeper", "keeper", "items", "keep", 2, 20, 30, "storehouse", "AUTO KEEP", "sorts finds · never asks", "storehouse"],
  ["clerk", "clerk", "town", "deposit", 3, 40, 36, "bank", "AUTO BANK", "banks spare gold", "bank"],
  ["drillmaster", "drillmaster", "character", "level", 2, 30, 36, "tent", "AUTO LEVEL", "levels the stance", ""],
  ["kennel_hand", "kennel-hand", "town", "field", 2, 30, 40, "kennel", "AUTO PETS", "fields best pets", "kennel"],
  ["herald", "herald", "town", "swap", 2, 20, 40, "board", "AUTO QUEST", "swaps stale quests", "quests"],
  ["guide", "guide", "scale", "start", 3, 40, 44, "mouth", "AUTO START", "starts deeper", "start"],
];
type St305 = { hired: string[]; counts: Record<string, number>; chest: number; sent: boolean; paused: string[]; acted: string[]; ranks?: Record<string, number> };
type Fk305 = Fk30 & { s: Fk30["s"] & { st305?: St305 } };
// (client half: the fake's demo lineage — a 7th heir with a chronicle — is an old save: the quartermaster, porter and scout pre-hired, as the
// core maps a save without a tree (docs/CUT30_5.md §2); a heir-1 lineage starts the tree from the quartermaster)
const st305 = (e: Fk305): St305 => (e.s.st305 ??= { hired: (e.s.lineage?.heir ?? 1) > 1 ? ["quartermaster", "porter", "scout"] : ["quartermaster"], counts: {}, chest: 0, sent: false, paused: [], acted: [] });
const on305 = (st: St305, id: string): boolean => st.hired.includes(id) && !st.paused.includes(id);
function works305(e: Fk305, L: Lineage, purse: number): Works {
  const st = st305(e); const age = L.age_h ?? 0; const unit = kitUnit(L.best_depth);
  const sysOpen = (id: string): boolean => !id || (L.systems ?? []).some((x) => x.id === id && x.open) || (id === "storehouse" && L.vault.length > 0) || (id === "forge" && L.best_depth > 1);
  const nodes: WorkNode[] = []; let lit: string | undefined;
  for (const [id, name, branch, chore, need, tenths, fb, post, beat, tip, gate] of NODES305) {
    const count = st.counts[id] ?? 0; const price = Math.round(unit * tenths / 10); const open = sysOpen(gate);
    const ready = open && (count >= need || (fb > 0 && age >= fb));
    const done = st.hired.includes(id);
    let state = done ? "done" : !open ? "shut" : ready ? (lit ? "ready" : "lit") : "open";
    if (state === "lit") lit = id;
    const rank = done && chore ? (st.ranks?.[id] ?? 1) : undefined;
    // (week 2 stand-in: a rank on offer once the scout is hired, a forge unit × the rank less one — the core waits 5 / 9 days of service)
    const edge = (r: number): string | undefined => r < 2 ? undefined : ({ porter: `+${2 * (r - 1)}% hauls`, scout: `−${5 * (r - 1)}% rest`, apprentice: `−${5 * (r - 1)}% steps`, clerk: `${20 + 2.5 * (r - 1)}‰ interest`, guide: r === 2 ? "half toll" : "no toll", drillmaster: "levels −◆1", armourer: "insures its finds" } as Record<string, string>)[id];
    // (the core waits days of service between a worker's ranks; the stand-in offers the lowest-ranked worker first, in the tree's order)
    const low = Math.min(...NODES305.filter(([w, , , c]) => c && st.hired.includes(w)).map(([w]) => st.ranks?.[w] ?? 1));
    const rankNext = rank && rank < 4 && st.hired.includes("scout") ? { rank_price: unit * rank, rank_wait_d: rank === low ? 0 : 1, ...(edge(rank + 1) ? { rank_adds: edge(rank + 1) } : {}) } : {};
    const bonusNow = rank && edge(rank) ? { bonus: edge(rank) } : {};
    nodes.push({ id, kind: "worker", branch, name, state, ...(rank ? { rank, ...rankNext, ...bonusNow } : {}), ...(chore ? { chore, count, need } : {}), price, affordable: purse + st.chest >= price, ...(fb ? { fallback_h: fb } : {}),
      ...(!open && gate ? { trigger: `${gate} built` } : {}), tip, ...(beat ? { beat } : {}), post, ...(st.paused.includes(id) ? { paused: true } : {}) });
  }
  for (const t of L.tracks ?? []) {
    nodes.push({ id: `${t.id}:${t.stage}`, kind: "stage", branch: t.id, name: t.stage, state: "done" });
    if (t.next) nodes.push({ id: `${t.id}:${t.next}`, kind: "stage", branch: t.id, name: t.next, state: "next", ...(t.trigger ? { trigger: t.trigger } : {}) });
  }
  const auto = on305(st, "scout"); const waits = !auto && !st.sent;
  const litN = nodes.find((n) => n.id === lit);
  const counting = nodes.find((n) => n.kind === "worker" && n.state === "open" && n.need);
  const next: NextPill = litN && litN.affordable ? { kind: "buy", node: lit, text: `${litN.name} · ${litN.price ? `$${litN.price}` : "free"}` }
    : st.chest > 0 && !on305(st, "porter") ? { kind: "chest", node: "porter", text: "open chest", have: st.chest }
    : waits ? { kind: "send", node: "scout", text: "send" }
    : litN ? { kind: "gold", node: lit, text: `${litN.name} · $${purse + st.chest}/$${litN.price}`, have: purse + st.chest, need: litN.price }
    : counting ? { kind: "count", node: counting.id, text: `${counting.name} · ${counting.count}/${counting.need}`, have: counting.count, need: counting.need }
    : { kind: "none", text: "" };
  // (the core gates each next rank by days of service, so a worker just promoted waits; the stand-in offers the lowest rank first)
  const litRank = lit ? undefined : nodes.filter((n) => n.rank_wait_d === 0).sort((a, b) => (a.rank ?? 1) - (b.rank ?? 1))[0]?.id;
  return { nodes, ...(lit ? { lit } : {}), ...(litRank ? { lit_rank: litRank } : {}), next, chest: st.chest, waits, sent: st.sent, auto_send: auto, ledger: purse + st.chest + (L.town?.bank ?? 0) };
}
{
  const P = FakeEngine.prototype as unknown as Record<string, (...a: unknown[]) => unknown>;
  const count = (e: Fk305, id: string): void => { const st = st305(e); if (!st.hired.includes(id)) st.counts[id] = (st.counts[id] ?? 0) + 1; };
  const lin = P.lineage; P.lineage = function (this: Fk305): Lineage {
    const L = lin.call(this) as Lineage; if (DEV_NO_SYSTEMS) return L;
    const st = st305(this); const purse = Math.max(0, L.gold - st.chest); L.gold = purse;
    L.tree = works305(this, L, purse);
    const posts: WorkerPost[] = NODES305.filter(([id]) => st.hired.includes(id)).map(([id, , , , , , , post]) => ({ id, post, rank: st.ranks?.[id] ?? 1, ...(st.paused.includes(id) ? { paused: true } : {}) }));
    const litN = L.tree.nodes.find((n) => n.id === L.tree!.lit); if (litN) posts.push({ id: litN.id, post: litN.post ?? "mouth", lit: true, price: litN.price });
    if (L.town) L.town.workers = posts;
    return L;
  };
  const off = P.runOffline; P.runOffline = function (this: Fk305, s: unknown): ReturnReport {
    if (DEV_NO_SYSTEMS) return off.call(this, s) as ReturnReport;
    const st = st305(this); const auto = on305(st, "scout");
    // (before the scout: a send is one run — an absence yields the run in flight, if any)
    const g0 = this.s.lineage.gold, c0 = st.chest; const r = off.call(this, auto ? s : st.sent ? 1 : 0) as ReturnReport; st.sent = false;
    // (the exits' haul went to the chest in `settle`, below, before the porter)
    const gain = Math.max(0, this.s.lineage.gold - g0), toChest = Math.max(0, st.chest - c0); const workers: WorkerAct[] = [];
    if (on305(st, "porter") && gain > 0) { workers.push({ id: "porter", what: `hauled $${gain}`, n: gain, first: !st.acted.includes("porter") }); st.acted.push("porter"); }
    return { ...r, ...(toChest ? { chest: toChest } : {}), ...(workers.length ? { workers } : {}) };
  };
  // client half (c305-client): a real run's exit — watched or offline — lands its haul in the chest until the porter, and the hero is home
  // (before the scout he waits for the next SEND)
  const settle = P.settle; P.settle = function (this: Fk305, run: unknown, real: unknown, ...a: unknown[]): unknown {
    if (DEV_NO_SYSTEMS || !real) return settle.call(this, run, real, ...a);
    const st = st305(this), g0 = this.s.lineage.gold; const r = settle.call(this, run, real, ...a);
    const gain = this.s.lineage.gold - g0; if (gain > 0 && !on305(st, "porter")) st.chest += gain;
    st.sent = false; return r;
  };
  const send = P.send; P.send = function (this: Fk305): Snapshot { const st = st305(this); if (!on305(st, "scout") && !st.sent) { count(this, "scout"); st.sent = true; } return send.call(this) as Snapshot; };
  P.openChest = function (this: Fk305): Lineage { const st = st305(this); if (st.chest <= 0) throw new Error("chest empty"); count(this, "porter"); st.chest = 0; return this.lineage(); };
  P.hire = function (this: Fk305, id: unknown): Lineage {
    const st = st305(this); const L = this.lineage(); const n = L.tree!.nodes.find((x) => x.id === id);
    if (!n || n.kind !== "worker") throw new Error("unknown worker"); if (n.state === "done") throw new Error("hired already"); if (L.tree!.lit !== id) throw new Error("not lit");
    const price = n.price ?? 0; if (L.gold + st.chest < price) throw new Error("not enough gold");
    const fromPurse = Math.min(price, L.gold); this.s.lineage.gold -= price; st.chest -= price - fromPurse; st.hired.push(id as string);
    if (id === "porter") st.chest = 0;
    return this.lineage();
  };
  P.promote = function (this: Fk305, id: unknown): Lineage {
    const st = st305(this); const L = this.lineage(); const n = L.tree!.nodes.find((x) => x.id === id);
    if (!n || L.tree!.lit_rank !== id) throw new Error("not on offer"); const price = n.rank_price ?? 0; if (L.gold + st.chest < price) throw new Error("not enough gold");
    const fromPurse = Math.min(price, L.gold); this.s.lineage.gold -= price; st.chest -= price - fromPurse; (st.ranks ??= {})[id as string] = (n.rank ?? 1) + 1;
    return this.lineage();
  };
  P.setWorker = function (this: Fk305, id: unknown, on: unknown): Lineage {
    const st = st305(this); if (!st.hired.includes(id as string)) throw new Error("not hired");
    st.paused = st.paused.filter((x) => x !== id); if (!on) st.paused.push(id as string); return this.lineage();
  };
  // the chores by hand fill their nodes' counts
  for (const [m, id] of [["buyKit", "apprentice"], ["bankDeposit", "clerk"], ["spendLevel", "drillmaster"], ["swapQuest", "herald"], ["setStart", "guide"], ["setParty", "kennel_hand"], ["hatch", "kennel_hand"], ["sellVault", "keeper"]] as const) {
    const f = P[m]; if (!f) continue;
    P[m] = function (this: Fk305, ...a: unknown[]): unknown { const r = f.apply(this, a); count(this, id); return r; };
  }
  const keep = P.keep; P.keep = function (this: Fk305, ...a: unknown[]): unknown { const r = keep.apply(this, a); count(this, "keeper"); return r; };
  const lo = P.loadout; P.loadout = function (this: Fk305, ids: unknown): unknown { const r = lo.call(this, ids); if ((ids as number[]).length) count(this, "armourer"); return r; };
}
