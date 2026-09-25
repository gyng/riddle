// Player-facing labels for rule tokens. Nouns and numbers; every label is a rule_token (≤ 3 words).
import type { Combo, ComboHit, Cond, InvItem, Lineage, Row, Verb } from "../engine/types";

/* copy:rule_token */
const COND: Record<string, string> = {
  "hp<": "hp <", "hp>": "hp >", "foes>=": "foes ≥", "adj>=": "adjacent ≥", foe_tag: "foe:", "foe_hp<": "foe hp <",
  item: "has:", unknown_item: "unknown item", "floor_seen>=": "seen ≥", "depth>=": "depth ≥", "alert>=": "alert ≥",
  in_corridor: "corridor", path_stairs: "stairs seen", ally: "ally", "loot>=": "loot ≥", "turns>": "turns >",
  on_hurt: "on hurt", on_kill: "on kill", on_see: "on see",
  "self_hp<": "self hp <", "party_hp<": "party hp <", party: "party:",
};
/* copy:rule_token */
const VERB: Record<string, string> = {
  attack: "attack", retreat: "retreat", back_corridor: "to corridor", drink: "drink", read: "read", throw: "throw",
  descend: "descend", bank: "bank", return: "return", rest: "rest", pick_up: "pick up", free_captive: "free captive",
  shield_bash: "shield bash", vanish: "vanish", card: "card", explore: "explore", stunned: "stunned",
  tame: "tame", recall: "recall", send: "send", follow: "follow", shoot: "shoot", burst: "burst", steal: "steal",
  split: "split", flank: "flank", drain: "drain",
  cleave: "cleave", taunt: "taunt", second_wind: "second wind", bulwark: "bulwark", backstab: "backstab", smoke: "smoke",
  ambush: "ambush", shadowstep: "shadowstep",
  kite: "kite", volley: "volley", trap: "trap", mark: "mark", double_shot: "double shot",
  bolt: "bolt", ward: "ward", blink: "blink", slow: "slow", nova: "nova", tactic: "card",
  // Cut 3: chores the trace names, a mirror companion's copy, the cadence card's filler
  hold: "hold", wait: "wait", shuffle: "shuffle", paralysed: "paralysed", stumble: "stumble", mimic: "mimic", feint: "feint",
  // Cut 5: the shrine's verb (`pray row` lends a row, `pray trait` swaps the trait)
  pray: "pray",
};
/* copy:rule_token */
/** Verb arguments and condition targets (cards, item kinds, foe tags) — Cut 3 names the ones an underscore→space
 *  would misread; everything else falls through `nice()`. */
const ARG: Record<string, string> = {
  cadence: "cadence", noise_discipline: "noise discipline", reflect_read: "reflect read", deep_march: "deep march",
  lantern_rig: "lantern rig", recall_sense: "recall sense",
  spear: "spear", mace: "mace", scale: "scale", regen: "regen", resist_fire: "resist fire", clarity: "clarity", recall: "recall",
  silence: "silence", earthquake: "earthquake", mirror: "mirror", lantern: "lantern", bell: "bell", salt: "salt", chalk: "chalk", mirror_shard: "mirror shard",
  reflect_melee: "reflects melee", reflect: "reflects", alarm: "alarm", blind: "blind", aura: "aura", gaze: "gaze", healer: "healer", echo: "echo", buffer: "buffer",
  // Cut 5 §4: situations (`on_see: nest`, `pray row`) and the stray — plain words
  nest: "nest", shrine: "shrine", vault: "vault", stray: "stray", row: "row", trait: "trait",
};
/** Cut 5 §4: an item kind's glyph for the vault choice — weapon · armour · potion · scroll. */
const WEAPON_KINDS = new Set(["dagger", "sword", "axe", "bow", "spear", "mace"]), ARMOUR_KINDS = new Set(["leather", "mail", "plate", "scale"]);
const POTION_KINDS = new Set(["heal", "strength", "speed", "invisibility", "poison", "caustic", "confusion", "fire", "potion"]);
export function kindGlyph(kind: string): string {
  if (WEAPON_KINDS.has(kind)) return "⚔";
  if (ARMOUR_KINDS.has(kind)) return "⛨";
  if (POTION_KINDS.has(kind)) return "⚗";
  return "▤";
}
export const PCT = new Set(["hp<", "hp>", "foe_hp<", "self_hp<", "party_hp<", "floor_seen>="]);
export const NUMS: Record<string, number[]> = {
  "hp<": [10, 20, 25, 30, 40, 50, 60, 75], "hp>": [25, 50, 75, 90], "foes>=": [1, 2, 3, 4], "adj>=": [1, 2, 3],
  "foe_hp<": [25, 50], "self_hp<": [20, 30, 50], "party_hp<": [25, 50], "floor_seen>=": [25, 50, 75, 100], "depth>=": [2, 3, 4, 5, 6, 7, 8, 10, 12], "alert>=": [1, 2, 3, 4, 5],
  "loot>=": [25, 50, 100, 200], "turns>": [50, 100, 200, 400],
};
export const needsN = (k: string): boolean => k in NUMS;
/** Cut 21 §3 (AE banked at D20; the picker stopped at 12): `depth ≥` offers every depth from 2 to the lineage's best + 2 (never
 *  fewer than the old table's, D2–8), and any depth the vocabulary names (a cond it sends with its `n`). */
export function depthNums(best: number, vocab?: { conds: Cond[]; depth_max?: number }): number[] {
  // QA a946e04 (T: `2–8` at best D4 and D5, `2–9` at D7 — the contract's own range, best + 2 never under 8): the engine's bound
  // (`Vocabulary.depth_max`) when it sends one; the same rule else
  const hi = vocab?.depth_max ?? Math.max(8, best + 2);
  const out = new Set<number>(Array.from({ length: hi - 1 }, (_, i) => i + 2));
  for (const c of vocab?.conds ?? []) if (c.k === "depth>=" && c.n !== undefined && c.n >= 2) out.add(c.n);
  return [...out].sort((a, b) => a - b);
}
const nice = (s: string): string => ARG[s] ?? s.replace(/_/g, " ");

export function condName(k: string): string { return COND[k] ?? nice(k); }
export function condLabel(c: Cond): string {
  const name = condName(c.k);
  if (c.n !== undefined) return `${name} ${c.n}${PCT.has(c.k) ? "%" : ""}`;
  if (c.t !== undefined) return `${name} ${nice(c.t)}`;
  return name;
}
export function verbLabel(v: Verb): string {
  const name = VERB[v.v] ?? nice(v.v);
  if (!v.a) return name;
  // `tag:thief` → `thief`; `fire,tag:thief` → `fire thief`; a trailing `,nearest` is the default target and stays silent
  const parts = v.a.split(",");
  const a = parts.filter((x, i) => i === 0 || x !== "nearest").map((x) => nice(x.startsWith("tag:") ? x.slice(4) : x)).join(" ");
  return `${name} ${a}`;
}
/** Cut 12 §6: a lost ally as the death screen and the report name it — a companion `jackal Ashar fell` (the watch's
 *  `kind · name`), a summoned ally with no name `ally hound fell`. */
export function lostLabel(k: string): string {
  const i = k.indexOf(" · ");
  if (i >= 0) return /* copy:callout */ `${k.slice(0, i).replace(/_/g, " ")} ${k.slice(i + 3)} fell`;
  const w = k.replace(/_/g, " ").trim().split(/\s+/).pop() ?? k;
  return /* copy:callout */ `ally ${w} fell`;
}
/** Cut 12 §6: a supply the camp gave rather than sold — the wire's `free`, else the kennel's leash (a lineage that has never
 *  tamed carries one free leash; the core sends no flag yet). */
export function isFreeSupply(L: Lineage, it: InvItem): boolean {
  if (it.free !== undefined) return it.free;
  // an engine without the flag (an old save): the kennel's leash is the only free line, and only before a tame
  return it.kind === "leash" && !(L.ledger ?? []).some((r) => r.tamed);
}
/** Cut 12 §1: a tactic card's row (`{v:"tactic"}`) — outside `max_rows`, one per owned card. */
export const isCardRow = (r: Row): boolean => r.verb.v === "tactic";
export const ownRowCount = (rows: Row[]): number => rows.filter((r) => !isCardRow(r)).length;
export function rowLabel(r: Row): string { return `${r.conds.map(condLabel).join(" · ")} → ${verbLabel(r.verb)}`; }
export const sameCond = (a: Cond, b: Cond): boolean => a.k === b.k && a.t === b.t;
export const sameVerb = (a: Verb, b: Verb): boolean => a.v === b.v && a.a === b.a;

/** Cut 8B §1: does a combo pattern name this verb? `throw` matches every throw; `drink unknown` only the gamble. */
export function verbIs(pat: string, v: Verb): boolean {
  const i = pat.indexOf(" ");
  return i < 0 ? v.v === pat : v.v === pat.slice(0, i) && v.a === pat.slice(i + 1);
}
/** Cut 8B §1: every combo in a row list, in row order — the client's mirror of the core's `combos_in`, so a pair lights up
 *  as it is written (the engine's own `Lineage.combos` arrives after `setRules`). */
export function combosIn(rows: Row[], table: Combo[] | undefined): ComboHit[] {
  const out: ComboHit[] = [];
  if (!table?.length) return out;
  for (let i = 0; i + 1 < rows.length; i++) {
    const c = table.find((t) => verbIs(t.a, rows[i].verb) && verbIs(t.b, rows[i + 1].verb));
    if (c) out.push({ rows: [i, i + 1], name: c.name });
  }
  return out;
}

/** QA 1a2a4a9 (O: `The monkey stole the leash (3).` — "the (3) refers to nothing"): an item's charges in the core's label (`leash (3)`)
 *  leave a note's text; the note names the thing, not its count. */
// QA 778fa1b (qaU: `Goblin Captain: telegraph.` read as a sentence under the seal): a fact note (`Name: tag.`) drops its stop
// QA 778fa1b (qaU: `Recovered heir 1's bones` beside `♟1` everywhere else): an heir reads as the bar names him
export const noteText = (t: string): string => t.replace(/ \(\d+\)(?=[.?!]?$)/, "").replace(/^([^.!?:]+: [a-z]+(?: [a-z]+)?)\.$/, "$1").replace(/\bheir (\d+)'s\b/g, "♟$1's")
  // QA 778fa1b (qaU, qaV: `R1 returned; died to jackal.` read as a return that worked): a return row that fired and did not get him home
  .replace(/\bR(\d+) (return|bank)(?:ed); died\b/g, /* copy:diary_line */ "R$1 $2 too late; died");

/** Cut 23 §3: a reason's gloss — the core's `Vocabulary.why_gloss` (reason prefix → ≤ 3 words), the longest prefix that matches
 *  (`same as R2` → `same as R`); undefined when none does. A `✗` callout passes its reason (`read ✗ no use` → `no use`). */
export function glossOf(table: Record<string, string> | undefined, why: string | undefined): string | undefined {
  if (!table || !why) return undefined;
  const w = why.includes("✗") ? why.slice(why.indexOf("✗") + 1).trim() : why.trim();
  let best: string | undefined;
  for (const k of Object.keys(table)) if (w.startsWith(k) && (!best || k.length > best.length)) best = k;
  return best !== undefined ? table[best] : undefined;
}

/** Cut 23 §3: the vocabulary's gloss table as last fetched (the app sets it), for reason lines drawn without the app at hand. */
let whyTable: Record<string, string> | undefined;
export function setWhyGloss(t: Record<string, string> | undefined): void { whyTable = t; }
export const whyGloss = (why: string | undefined): string | undefined => glossOf(whyTable, why);
