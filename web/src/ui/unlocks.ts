// Unlock presentation. Ids and costs come from the engine's `unlocks()` catalogue
// (crates/riddle-core/src/meta.rs); this file only adds labels and the display chain (row6 is
// hidden until row5 is owned, etc.) so the camp shows one step at a time.
import type { UnlockInfo } from "../engine/types";

/* copy:unlock_card */
const LABEL: Record<string, string> = {
  row5: "+1 row", row6: "+1 row", row7: "+1 row", row8: "+1 row",
  throw: "verb: throw", rogue: "class: rogue", vault2: "+1 vault", vault3: "+1 vault",
  corridor_fighting: "card: corridor fighting", kite_archers: "card: kite archers", stair_dance: "card: stair dance",
  tame: "verb: tame", party_slot_2: "+1 party",
};
const AFTER: Record<string, string> = { row6: "row5", row7: "row6", row8: "row7", vault3: "vault2" };

export type UnlockCard = UnlockInfo & { label: string };

/** Catalogue entries worth showing: not owned, and the previous step of a chain owned. */
export function visible(catalogue: UnlockInfo[]): UnlockCard[] {
  const owned = new Set(catalogue.filter((u) => u.owned).map((u) => u.id));
  return catalogue
    .filter((u) => !u.owned && (!AFTER[u.id] || owned.has(AFTER[u.id])))
    .map((u) => ({ ...u, label: LABEL[u.id] ?? u.id.replace(/_/g, " ") }));
}
export const vaultSlots = (owned: string[]): number => 1 + ["vault2", "vault3"].filter((u) => owned.includes(u)).length;
