// Cut 29 §2: the systems open one at a time (the core's curriculum, `Lineage.systems`). The core decides; this file answers "is it
// open" for every surface that gates on it, maps the systems onto the reveal ladder's steps (reveal.ts), and clears the core's
// `new` marks once the camp has shown them (`seenSystems`). No tutorial text: the reveal is the glint.
import type { Lineage } from "../engine/types";

export type SystemId = "send" | "dial" | "headline" | "edit" | "death" | "exits" | "loadout" | "unlocks" | "reorder" | "vs" | "tags" | "party" | "cage"
  | "walls" | "divergence" | "forge" | "start" | "route" | "oaths" | "automations" | "route2" | "heir_pick" | "class" | "traits" | "blood"
  // Cut 30 (core `systems.rs`): the packages' arrivals, the town's buildings, the quest board, the pen
  | "stances" | "tactics" | "tactic2" | "temperament" | "quests" | "bank" | "storehouse" | "kennel" | "pen";

/** Open when the core says so; an engine without the curriculum (an old core) or an id it does not list is open. */
export function sysOpen(L: Pick<Lineage, "systems"> | undefined, id: SystemId): boolean {
  const s = L?.systems;
  if (!s?.length) return true;
  const x = s.find((y) => y.id === id);
  return x ? x.open : true;
}
/** Opened since the camp last looked (the core's `new`, cleared by `seenSystems`). */
export function sysNew(L: Pick<Lineage, "systems"> | undefined, id: SystemId): boolean {
  return !!L?.systems?.find((y) => y.id === id)?.new;
}
export const anyNew = (L: Pick<Lineage, "systems"> | undefined): boolean => !!L?.systems?.some((s) => s.new);
/** The core curriculum is on the wire. */
export const hasCurriculum = (L: Pick<Lineage, "systems"> | undefined): boolean => !!L?.systems?.length;

/** A system's name on its reveal plaque (≤ 2 words) and its icon (a packed one, else a CSS glyph). */
/* copy:label */
const LABEL: Partial<Record<SystemId, string>> = { edit: "edit", death: "verdicts", exits: "exits", loadout: "loadout", unlocks: "unlocks", reorder: "reorder",
  vs: "vs line", tags: "foe tags", party: "party", cage: "cages", walls: "boss walls", divergence: "replay diff", forge: "forge", start: "waystones",
  route: "route", oaths: "oaths", automations: "automations", route2: "second route", heir_pick: "heir pick", class: "classes", traits: "traits", blood: "bloodline",
  stances: "stances", tactics: "tactics", tactic2: "tactic slot", temperament: "temperament", quests: "quests", bank: "bank", storehouse: "storehouse", kennel: "kennel", pen: "the pen" };
const ICON: Partial<Record<SystemId, [string, string]>> = { edit: ["edit", "✎"], death: ["morgue", "☠"], exits: ["bail", "⇡"], loadout: ["loadout", "⚗"],
  unlocks: ["unlocks", "◆"], reorder: ["", "⇅"], vs: ["", "⇄"], tags: ["", "⌖"], party: ["party", "☗"], cage: ["vault", "▦"], walls: ["depth", "⛨"],
  divergence: ["", "⑂"], forge: ["forge", "⚒"], start: ["depth", "⌂"], route: ["", "⑂"], oaths: ["renown", "✠"], automations: ["gold", "⚙"],
  route2: ["", "⑂"], heir_pick: ["", "♛"], class: ["", "⚔"], traits: ["", "✦"], blood: ["", "✦"],
  stances: ["", "⛉"], tactics: ["unlocks", "✦"], tactic2: ["unlocks", "✦"], temperament: ["", "☯"], quests: ["renown", "✠"], bank: ["gold", "$"], storehouse: ["vault", "▦"], kennel: ["party", "☗"], pen: ["edit", "✎"] };
export const systemLabel = (id: string): string => LABEL[id as SystemId] ?? id.replace(/_/g, " ");
export const systemIcon = (id: string): [string, string] => ICON[id as SystemId] ?? ["", "✦"];
