// Unlock presentation. Ids, costs and `needs` (the gate as text) come from the engine's `unlocks()`
// catalogue (crates/riddle-core/src/meta.rs); this file only adds labels and the display chain (row6 is
// hidden until row5 is owned, etc.) so the camp shows one step at a time.
// Cut 9 §2: every buy goes through `openUnlockSheet` — the card's rows / effect, `◆cost`, `needs` when gated, `reach ±N%`
// when known, then `buy`. No card buys on its own tap; a disabled card still opens the sheet to show its gate.
import type { App } from "../app";
import type { Lineage, UnlockInfo } from "../engine/types";
import { CLASSES, isFreeClass } from "../engine/classes";
import { h } from "./dom";
import { rowChips } from "./editor";
import { openSheet } from "./sheet";

/* copy:unlock_card */
const LABEL: Record<string, string> = {
  row5: "+1 row", row6: "+1 row", row7: "+1 row", row8: "+1 row", row9: "+1 row", row10: "+1 row",
  party_slot_2: "+1 party", party_slot_3: "+1 party", party_slot_4: "+1 party",
  vault2: "+1 vault", vault3: "+1 vault", vault4: "+1 vault", vault5: "+1 vault",
  rogue: "class: rogue", ranger: "class: ranger", caster: "class: caster",
  tame: "verb: tame", throw: "verb: throw",
  cond_alert: "cond: alert", cond_turns: "cond: turns", cond_loot: "cond: loot", cond_on_kill: "cond: on kill", cond_on_see: "cond: on see", cond_party_hp: "cond: party hp",
  corridor_fighting: "card: corridor fighting", kite_archers: "card: kite archers", stair_dance: "card: stair dance", gas_step: "card: gas step",
  pack_break: "card: pack break", thief_guard: "card: thief guard", boss_focus: "card: boss focus", last_stand: "card: last stand",
  quartermaster: "auto: keep weapon+armour", auto_supply: "auto: restock", auto_insure: "auto: insure",
  incubator: "eggs: 1 rest", supply_cap_5: "supplies 3 → 5", bone_sense: "path: bones", third_tag: "breed: 3 tags",
  // Cut 3 tier 2
  cadence: "card: cadence", noise_discipline: "card: noise discipline", reflect_read: "card: reflect read", deep_march: "card: deep march",
  lantern_rig: "sight: lantern rig", recall_sense: "auto: recall sense",
};
const AFTER: Record<string, string> = { row6: "row5", row7: "row6", row8: "row7", row9: "row8", row10: "row9", vault3: "vault2", vault4: "vault3", vault5: "vault4", party_slot_3: "party_slot_2", party_slot_4: "party_slot_3" };

export type UnlockCard = UnlockInfo & { label: string; gated: boolean };
/** Cut 10 §3: a `+1 row` card waits until the set is full: a client-side gate when the core sends none. The gate reads as a
 *  requirement, `⊘ fill rows` — both QA players on 952e306 read the core's `rows full` as a state ("rows full vs 2/4?"), so the
 *  core's own text is rewritten too. */
const ROWS_GATE = /* copy:unlock_card */ "fill rows";
export function withRowsGate(u: UnlockCard, rows: number, max: number): UnlockCard {
  if (!/^row\d+$/.test(u.id) || u.owned) return u;
  if (u.needs === "rows full") u = { ...u, needs: ROWS_GATE };
  if (rows >= max) return u;
  return { ...u, available: false, gated: true, needs: ROWS_GATE };   // over `◆2 more`: the marks would be wasted either way
}

/** Catalogue entries worth showing: not owned, and the previous step of a chain owned.
 *  `gated`: unavailable with a `needs` gate still missing (the core sends `needs` only while unmet). */
export function visible(catalogue: UnlockInfo[]): UnlockCard[] {
  const owned = new Set(catalogue.filter((u) => u.owned).map((u) => u.id));
  return catalogue
    .filter((u) => !u.owned && (!AFTER[u.id] || owned.has(AFTER[u.id])))
    .map((u) => ({ ...u, label: LABEL[u.id] ?? u.id.replace(/_/g, " "), gated: !u.available && !!u.needs }));
}
/** Cut 10 §3 / Cut 12 §1: a card's reach delta says where the card goes — `reach +4% at R3` (the catalogue's `insert_at`), else
 *  `at end`; other unlocks carry the bare delta. */
export function deltaLabel(u: UnlockInfo, d: number): string {
  const where = !isCard(u) ? "" : u.insert_at !== undefined ? /* copy:unlock_card */ ` at R${u.insert_at + 1}` : /* copy:unlock_card */ " at end";
  return /* copy:unlock_card */ `reach ${d > 0 ? "+" : "−"}${Math.abs(d)}%${where}`;
}
/** Cut 9 §2: the sheet behind an unlock card; `after` runs once a buy went through (the report repaints itself with it).
 *  Cut 12 §1: a card never takes a row (card rows sit outside `max_rows`). */
export function openUnlockSheet(app: App, u: UnlockCard, after?: () => void): void {
  openSheet((close) => {
    const d = u.delta === undefined ? 0 : Math.round(u.delta * 100);
    // the gate as of now, not as of the card's paint: a card painted before a buy or a report can carry a stale `available`
    // (QA B on 952e306: "CLASS: RANGER ◆6 · ⊘ ◆2 more has an active buy; tapping it did nothing"); any `needs` or a marks
    // shortfall against the live lineage turns `buy` off
    const short = Math.max(0, u.cost - app.lineage.marks);
    const needs = u.needs ?? (short ? /* copy:unlock_card */ `◆${short} more` : undefined);
    const can = u.available && !needs;
    let sent = false;
    const buy = h("button", { class: `btn primary wide buy${can ? "" : " off"}`, disabled: !can, onclick: () => {
      if (sent) return; sent = true;
      void app.buy(u.id).then((ok) => { close(); if (ok) after?.(); });
    } }, /* copy:button */ "buy");
    return h("div", { class: "sheet-body unlock-sheet" },
      h("div", { class: "label row-label" }, u.label, " ", h("span", { class: "num cost" }, `◆${u.cost}`)),
      u.rows?.length ? h("div", { class: "card-rows" }, ...u.rows.map((r) => h("div", { class: "row locked" }, rowChips(r)))) : "",
      needs ? h("div", { class: "needs-line dim" }, "⊘ ", needs.replace(/_/g, " ")) : "",
      d ? h("div", { class: `num delta ${d > 0 ? "up" : "down"}` }, deltaLabel(u, d)) : "",   // Cut 10 §3 / Cut 12 §1
      buy);
  });
}
/** Cut 6 §4: a tactic card (it becomes a row when bought). */
export const isCard = (u: UnlockInfo): boolean => /^card:/.test(LABEL[u.id] ?? "");
/** The sheet behind an owned shelf chip: the buy sheet's title line, the rows, and — for a card whose row the set no longer
 *  holds — `insert`, which puts the card's row back where a buy would (before the engagement row: `app.insertCard`; the
 *  catalogue carries `insert_at` only while unowned) and opens the camp on it (QA on 952e306: "owned card chip opens a
 *  titleless read-only sheet, no way to re-insert a dropped card (◆3 spent)"). */
export function openOwnedSheet(app: App, u: UnlockCard): void {
  openSheet((close) => {
    const insert = isCard(u) && !app.holdsCard(u.id)
      ? h("button", { class: "btn primary wide", onclick: () => { close(); const i = app.insertCard(u.id); app.go({ kind: "camp", highlight: i }); } }, /* copy:button */ "insert")
      : "";
    return h("div", { class: "sheet-body unlock-sheet" },
      h("div", { class: "label row-label" }, u.label),
      u.rows?.length ? h("div", { class: "card-rows" }, ...u.rows.map((r) => h("div", { class: "row locked" }, rowChips(r)))) : "",
      insert);
  });
}
/** Cut 6 §6: owned entries that carry rows (cards, automations) — the shelf keeps them as chips that open their rows. */
export function ownedRows(catalogue: UnlockInfo[]): UnlockCard[] {
  return catalogue.filter((u) => u.owned && u.rows?.length).map((u) => ({ ...u, label: LABEL[u.id] ?? u.id.replace(/_/g, " "), gated: false }));
}
export const vaultSlots = (owned: string[]): number => 1 + ["vault2", "vault3", "vault4", "vault5"].filter((u) => owned.includes(u)).length;
export const supplyCap = (owned: string[]): number => (owned.includes("supply_cap_5") ? 5 : 3);

/** Classes the picker lists: the known ladders ∪ whatever the lineage carries levels for ∪ class unlocks in the
 *  catalogue. `owned` = the free class or its unlock bought. */
export function classList(L: Lineage, catalogue: UnlockInfo[] = []): { cls: string; owned: boolean; level: number }[] {
  const ids = new Set<string>([...CLASSES, ...Object.keys(L.classes ?? {}), ...catalogue.filter((u) => /^class:/.test(LABEL[u.id] ?? "")).map((u) => u.id)]);
  return [...ids].map((cls) => ({ cls, owned: isFreeClass(cls) || L.unlocks.includes(cls), level: L.classes?.[cls]?.level ?? 1 }));
}
