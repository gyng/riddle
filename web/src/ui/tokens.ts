// Player-facing labels for rule tokens. Nouns and numbers; every label is a rule_token (≤ 3 words).
import type { Combo, ComboHit, Cond, InvItem, Lineage, Row, Verb } from "../engine/types";

/* copy:rule_token */
const COND: Record<string, string> = {
  "hp<": "hp <", "hp>": "hp >", "foes>=": "foes ≥", "adj>=": "adjacent ≥", foe_tag: "foe:", "foe_hp<": "foe hp <",
  item: "has:", unknown_item: "unknown item", "floor_seen>=": "seen ≥", "depth>=": "depth ≥", "alert>=": "alert ≥",
  in_corridor: "corridor", path_stairs: "stairs seen", ally: "ally", "loot>=": "loot ≥", "turns>": "turns >",
  on_hurt: "on hurt", on_kill: "on kill", on_see: "on see",
  "self_hp<": "self hp <", "party_hp<": "party hp <", party: "party:",
  in: "in:", biome: "in:",   // Cut 26 §2: `in: fens` — the lane the hero is in (open once the biome was entered)
};
/* copy:rule_token */
const VERB: Record<string, string> = {
  gunner_tactic: "Gun handling",
  attack: "attack", retreat: "retreat", back_corridor: "to corridor", drink: "drink", read: "read", throw: "throw",
  descend: "descend", bank: "bank", return: "return", rest: "rest", pick_up: "pick up", free_captive: "free captive",
  shield_bash: "shield bash", vanish: "vanish", card: "tactic", explore: "explore", stunned: "stunned",
  tame: "tame", recall: "recall", send: "send", follow: "follow", shoot: "shoot", burst: "burst", steal: "steal",
  split: "split", flank: "flank", drain: "drain",
  cleave: "cleave", taunt: "taunt", second_wind: "second wind", bulwark: "bulwark", backstab: "backstab", smoke: "smoke",
  ambush: "ambush", shadowstep: "shadowstep",
  kite: "kite", volley: "volley", trap: "trap", mark: "mark", double_shot: "double shot",
  bolt: "bolt", ward: "ward", blink: "blink", slow: "slow", nova: "nova", tactic: "tactic",
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
  // QA 524827b (qaAA: `pray row`, `pray trait`, `on see stray` unexplained): what the prayer asks for (a lent row, a new trait), the
  // stray a lost heir's pet
  nest: "nest", shrine: "shrine", vault: "vault", stray: "stray pet", row: "for row", trait: "for trait",
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
  // QA 308f045 (qaAC: `ally hound fell` — "no hound shown anywhere before"): an unnamed ally is a summon (the scroll's hound) and says so
  return /* copy:callout */ `summoned ${w} fell`;
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
/** Cut 30 §2: a package's compiled row (`stance:steady`, `tactic:boss_focus`, `temper:skittish`, `drill:lich`) — outside the row cap
 *  (the core's `Row::is_pkg`); the pen's rows are the player's own. */
export const isPkgRow = (r: Row): boolean => /^(stance|tactic|temper|drill|style|class):/.test((r.origin as string | undefined) ?? "");
export const ownRowCount = (rows: Row[]): number => rows.filter((r) => !isCardRow(r) && !isPkgRow(r)).length;
export function rowLabel(r: Row): string { return `${r.conds.map(condLabel).join(" · ")} → ${verbLabel(r.verb)}`; }
/** docs/COPY.md §2 (owner: "R1/R2 labels don't make sense to humans, can't remember"): a rule is named by what it says — its action,
 *  and the one condition that places it (`return at 20%`, `drink heal at 30%`, `attack nearest`) — never by a number. Two rules
 *  that would read alike take their next condition. The tablet's small ordinal is position only, never a name. */
const qualOf = (c: Cond, verb: string): string | undefined => {
  switch (c.k) {
    case "hp<": case "self_hp<": return c.n !== undefined ? /* copy:rule_token */ `at ${c.n}%` : undefined;
    case "hp>": return c.n !== undefined ? /* copy:rule_token */ `over ${c.n}%` : undefined;
    case "depth>=": return c.n !== undefined ? /* copy:rule_token */ `at D${c.n}` : undefined;
    case "foe_tag": return c.t && !verb.includes(nice(c.t)) ? /* copy:rule_token */ `vs ${nice(c.t)}` : undefined;
    case "foes>=": return c.n !== undefined && c.n > 1 ? /* copy:rule_token */ `vs ${c.n}+` : undefined;
    default: return undefined;
  }
};
function nameAt(r: Row, depth: number): string {
  const v = verbLabel(r.verb);
  const q = r.conds.map((c) => qualOf(c, v)).filter((x): x is string => !!x);
  // depth 1: the action and its first placing condition; deeper (two rules would read alike): its other conditions, `if …` last
  const extra = r.conds.filter((c) => !qualOf(c, v)).map((c) => /* copy:rule_token */ `if ${condLabel(c)}`);
  return [v, ...(depth === 1 ? q.slice(0, 1) : [...q, ...extra].slice(0, depth))].join(" ");
}
export function ruleName(rows: Row[], i: number): string {
  const r = rows[i];
  if (!r) return /* copy:callout */ "a rule";
  for (let depth = 1; depth <= 3; depth++) {
    const name = nameAt(r, depth);
    if (!rows.some((x, j) => j !== i && nameAt(x, depth) === name)) return name;
  }
  return nameAt(r, 3);
}
/** The rules the screen being drawn talks about (the camp's set, a death's set as it ran): `nameRefs` names core text's rules by them. */
let refRowsOf: () => Row[] = () => [];
export function setRefRows(rows: Row[] | (() => Row[]) | undefined): void { refRowsOf = typeof rows === "function" ? rows : () => rows ?? []; }
/** The name of rule `i` of the rules the screen talks about. */
export const refName = (i: number): string => ruleName(refRowsOf(), i);
export const refRowsHas = (i: number): boolean => !!refRowsOf()[i];
/** The core and the wire still write a rule as `R2` (traces, notes, the reel, pending lines). On screen it goes: where the rule's verb
 *  follows (`R2 attack nearest no path`, `R1 drank`) the id is dropped — the verb names it; elsewhere (`same as R2`, `R5 under R2`,
 *  `cut R3`, `R2 first`) it becomes the rule's name. */
const NOT_VERB = /^(?:first|fired|ended|under|above|below|and|or|is|was|now)\b/;
export function nameRefs(s: string): string {
  if (!/\bR\d/.test(s)) return s;
  const rows = refRowsOf(), re = /\bR(\d{1,2})\b/g;
  let out = "", i = 0, m: RegExpExecArray | null;
  while ((m = re.exec(s))) {
    out += s.slice(i, m.index);
    const at = m.index + m[0].length, idx = Number(m[1]) - 1, row = rows[idx];
    const next = /^ (\S*)/.exec(s.slice(at))?.[1] ?? "";
    if (next && /^[a-z?]/.test(next) && !NOT_VERB.test(next)) {
      // its action follows: `R2 return` → the rule's name (`return at 20%`), the action's words taken; another verb form
      // (`R1 drank`, `R4 hit archer`) keeps its words and loses the id
      const words = row ? verbLabel(row.verb).split(" ") : [];
      if (row && next === words[0]) {
        let k = at;
        for (const w of words) { const mm = s.slice(k).startsWith(` ${w}`) && !/^[\w]/.test(s.slice(k + w.length + 1)) ? ` ${w}` : ""; if (!mm) break; k += mm.length; }
        out += ruleName(rows, idx); i = k;
      } else i = at + 1;
    } else { out += row ? ruleName(rows, idx) : /* copy:callout */ "a rule"; i = at; }
    re.lastIndex = i;
  }
  return out + s.slice(i);
}
/** docs/COPY.md pass 4 (`heir 15` read as a level or an age): the heir is the family's 15th — `15th heir`. */
export const heirOrd = (n: number): string => { const t = n % 100, u = n % 10; return /* copy:label */ `${n}${t >= 11 && t <= 13 ? "th" : u === 1 ? "st" : u === 2 ? "nd" : u === 3 ? "rd" : "th"} heir`; };
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
export const noteText = (t: string): string => t.replace(/ \(\d+\)(?=[.?!]?$)/, "").replace(/^([^.!?:]+: [a-z]+(?: [a-z]+)?)\.$/, "$1")
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
