// Unlock presentation. Ids, costs and `needs` (the gate as text) come from the engine's `unlocks()`
// catalogue (crates/riddle-core/src/meta.rs); this file only adds labels and the display chain (row6 is
// hidden until row5 is owned, etc.) so the camp shows one step at a time.
import type { Lineage, UnlockInfo } from "../engine/types";
import { CLASSES, isFreeClass } from "../engine/classes";

/* copy:unlock_card */
const LABEL: Record<string, string> = {
  row5: "+1 row", row6: "+1 row", row7: "+1 row", row8: "+1 row",
  party_slot_2: "+1 party", party_slot_3: "+1 party",
  vault2: "+1 vault", vault3: "+1 vault", vault4: "+1 vault",
  rogue: "class: rogue", ranger: "class: ranger", caster: "class: caster",
  tame: "verb: tame", throw: "verb: throw",
  cond_alert: "cond: alert", cond_turns: "cond: turns", cond_loot: "cond: loot", cond_on_kill: "cond: on kill", cond_on_see: "cond: on see", cond_party_hp: "cond: party hp",
  corridor_fighting: "card: corridor fighting", kite_archers: "card: kite archers", stair_dance: "card: stair dance", gas_step: "card: gas step",
  pack_break: "card: pack break", thief_guard: "card: thief guard", boss_focus: "card: boss focus", last_stand: "card: last stand",
  quartermaster: "auto: keep weapon+armour", auto_supply: "auto: restock", auto_insure: "auto: insure",
  incubator: "eggs: 1 rest", supply_cap_5: "supplies 3 → 5", bone_sense: "path: bones", third_tag: "breed: 3 tags",
};
const AFTER: Record<string, string> = { row6: "row5", row7: "row6", row8: "row7", vault3: "vault2", vault4: "vault3", party_slot_3: "party_slot_2" };

export type UnlockCard = UnlockInfo & { label: string; gated: boolean };

/** Catalogue entries worth showing: not owned, and the previous step of a chain owned.
 *  `gated`: unavailable with a `needs` gate still missing (the core sends `needs` only while unmet). */
export function visible(catalogue: UnlockInfo[]): UnlockCard[] {
  const owned = new Set(catalogue.filter((u) => u.owned).map((u) => u.id));
  return catalogue
    .filter((u) => !u.owned && (!AFTER[u.id] || owned.has(AFTER[u.id])))
    .map((u) => ({ ...u, label: LABEL[u.id] ?? u.id.replace(/_/g, " "), gated: !u.available && !!u.needs }));
}
export const vaultSlots = (owned: string[]): number => 1 + ["vault2", "vault3", "vault4"].filter((u) => owned.includes(u)).length;
export const supplyCap = (owned: string[]): number => (owned.includes("supply_cap_5") ? 5 : 3);

/** Classes the picker lists: the known ladders ∪ whatever the lineage carries levels for ∪ class unlocks in the
 *  catalogue. `owned` = the free class or its unlock bought. */
export function classList(L: Lineage, catalogue: UnlockInfo[] = []): { cls: string; owned: boolean; level: number }[] {
  const ids = new Set<string>([...CLASSES, ...Object.keys(L.classes ?? {}), ...catalogue.filter((u) => /^class:/.test(LABEL[u.id] ?? "")).map((u) => u.id)]);
  return [...ids].map((cls) => ({ cls, owned: isFreeClass(cls) || L.unlocks.includes(cls), level: L.classes?.[cls]?.level ?? 1 }));
}
