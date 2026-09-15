// Unlock catalogue (docs/CUT1.md "Meta"): id → cost, label. Ids are a client assumption until the core ships.
export type Unlock = { id: string; cost: number; label: string; after?: string };

/* copy:unlock_card */
export const UNLOCKS: Unlock[] = [
  { id: "row5", cost: 2, label: "+1 row" },
  { id: "row6", cost: 3, label: "+1 row", after: "row5" },
  { id: "row7", cost: 4, label: "+1 row", after: "row6" },
  { id: "row8", cost: 5, label: "+1 row", after: "row7" },
  { id: "verb:throw", cost: 2, label: "verb: throw" },
  { id: "class:rogue", cost: 4, label: "class: rogue" },
  { id: "vault2", cost: 3, label: "+1 vault" },
  { id: "vault3", cost: 5, label: "+1 vault", after: "vault2" },
  { id: "card:corridor_fighting", cost: 3, label: "card: corridor fighting" },
  { id: "card:kite_archers", cost: 3, label: "card: kite archers" },
  { id: "card:stair_dance", cost: 3, label: "card: stair dance" },
];

export function available(owned: string[]): Unlock[] {
  return UNLOCKS.filter((u) => !owned.includes(u.id) && (!u.after || owned.includes(u.after)));
}
export const vaultSlots = (owned: string[]): number => 1 + ["vault2", "vault3"].filter((u) => owned.includes(u)).length;
