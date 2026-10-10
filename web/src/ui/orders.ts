// Cut 120 §8 (docs/CUT120_AUTOMATION_FILL.md): the orders sheet's Cut 120 half — the Legacy order, the ranks order, the rank-II perk
// chips (`setPerk`), the herald's ascend order (after the King), and `same for all` on each shareable order when the town keeps more than
// one bloodline (`StandingOrders.shared`: a set copies the order to every bloodline, and later changes follow). All truth is the core's:
// the client draws the wire's orders and sends the whole set back through `setOrders`.
import type { Lineage, StandingOrders, WorkNode } from "../engine/types";
import { kingSlain } from "./king-eta";

export const LEGACY_ORDERS = ["balanced", "health", "damage", "armour", "off"] as const;
/* copy:button */
export const LEGACY_WORD: Record<string, string> = { balanced: "balanced", health: "health", damage: "damage", armour: "armour", off: "off" };
/* copy:tooltip */
export const LEGACY_TIP: Record<string, string> = { balanced: "health → damage → armour in turn · then effects", health: "health first · then balanced", damage: "damage first · then balanced", armour: "armour first · then balanced", off: "Legacy by hand" };
export const RANKS_ORDERS = ["auto", "off"] as const;
/* copy:button */
export const RANKS_WORD: Record<string, string> = { auto: "auto", off: "off" };
export const ASCEND_ORDERS = ["off", "on"] as const;
/* copy:button */
export const ASCEND_WORD: Record<string, string> = { off: "off", on: "on" };
/** the order keys `same for all` can carry (core `tree::SHAREABLE`) */
export const SHAREABLE = ["insure", "forge", "wall", "sink", "heir", "kennel", "legacy", "ranks", "ascend"] as const;
export type ShareKey = (typeof SHAREABLE)[number];

/** More than one bloodline in the town: each shareable row offers `same for all`. */
export const manyBloodlines = (L: Pick<Lineage, "hero_slots">): boolean => (L.hero_slots?.length ?? 0) > 1;
export const isShared = (o: StandingOrders, key: ShareKey): boolean => (o.shared ?? []).includes(key);
/** The orders with `key` toggled in `shared` (the rest of the set as it stands). */
export function toggleShared(o: StandingOrders, key: ShareKey): Partial<StandingOrders> {
  const cur = o.shared ?? [];
  return { shared: cur.includes(key) ? cur.filter((k) => k !== key) : [...cur, key] };
}

const hiredHands = (L: Pick<Lineage, "tree">): WorkNode[] => (L.tree?.nodes ?? []).filter((n) => n.kind === "worker" && n.state === "done" && n.id !== "quartermaster");
/** The Legacy order: once the core sends it and the bloodline has Legacy to spend (or the order is already set off its default). */
export const legacyOrderShown = (L: Lineage): boolean => !!L.orders?.legacy && (!!L.bloodline || !!L.legacy_upgrades?.length || L.orders.legacy !== "off");
/** The ranks order: once a hand is hired. */
export const ranksOrderShown = (L: Lineage): boolean => !!L.orders?.ranks && hiredHands(L).length > 0;
/** The herald's ascend order: only after the King (and only on a core that sends it). */
export const ascendOrderShown = (L: Lineage): boolean => !!L.orders?.ascend && kingSlain(L);
/** The rank-II perk rows: each hired worker the core offers a perk chip for (`WorkNode.perks`). */
export const perkNodes = (L: Pick<Lineage, "tree">): WorkNode[] => hiredHands(L).filter((n) => (n.perks?.length ?? 0) > 1);

/** The orders summary's Cut 120 bits — only a choice off the quiet default reads (`legacy damage`, `ascend on`). */
export function cut120OrderBits(o: StandingOrders): string[] {
  return [o.legacy && o.legacy !== "balanced" && o.legacy !== "off" ? /* copy:callout */ `legacy ${LEGACY_WORD[o.legacy] ?? o.legacy}` : "",
    o.ascend === "on" ? /* copy:callout */ "ascend on" : ""].filter(Boolean);
}
