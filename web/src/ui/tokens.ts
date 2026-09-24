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
