// Player-facing labels for rule tokens. Nouns and numbers; every label is a rule_token (≤ 3 words).
import type { Cond, Row, Verb } from "../engine/types";

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
};
export const PCT = new Set(["hp<", "hp>", "foe_hp<", "self_hp<", "party_hp<", "floor_seen>="]);
export const NUMS: Record<string, number[]> = {
  "hp<": [10, 20, 25, 30, 40, 50, 60, 75], "hp>": [25, 50, 75, 90], "foes>=": [1, 2, 3, 4], "adj>=": [1, 2, 3],
  "foe_hp<": [25, 50], "self_hp<": [20, 30, 50], "party_hp<": [25, 50], "floor_seen>=": [25, 50, 75, 100], "depth>=": [2, 3, 5, 8, 10, 12], "alert>=": [1, 2, 3, 4, 5],
  "loot>=": [25, 50, 100, 200], "turns>": [50, 100, 200, 400],
};
export const needsN = (k: string): boolean => k in NUMS;
const nice = (s: string): string => s.replace(/_/g, " ");

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
  const a = v.a.startsWith("tag:") ? v.a.slice(4) : v.a.replace(",nearest", "");
  return `${name} ${nice(a)}`;
}
export function rowLabel(r: Row): string { return `${r.conds.map(condLabel).join(" · ")} → ${verbLabel(r.verb)}`; }
export const sameCond = (a: Cond, b: Cond): boolean => a.k === b.k && a.t === b.t;
export const sameVerb = (a: Verb, b: Verb): boolean => a.v === b.v && a.a === b.a;
