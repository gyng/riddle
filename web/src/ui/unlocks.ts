// Unlock presentation. Ids, costs and `needs` (the gate as text) come from the engine's `unlocks()`
// catalogue (crates/riddle-core/src/meta.rs); this file only adds labels and the display chain (row6 is
// hidden until row5 is owned, etc.) so the camp shows one step at a time.
// Cut 9 §2: every buy goes through `openUnlockSheet` — the card's rows / effect, `◆cost`, `needs` when gated, `reach ±N%`
// when known, then `buy`. No card buys on its own tap; a disabled card still opens the sheet to show its gate.
import type { App } from "../app";
import { revealed } from "./reveal";
import { engagementRow } from "../app";
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

/** Cut 25 §6: the chain step a card follows (`row6` → `row5`), undefined for a first step. */
export const afterOf = (id: string): string | undefined => AFTER[id];
/** An unlock's shelf label (`+1 row`, `card: kite archers`). */
export const labelOf = (id: string): string => LABEL[id] ?? id.replace(/_/g, " ");
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
 *  `at end`; other unlocks carry the bare delta. Cut 13 §5: the `±` rides the delta when the catalogue sends `pm`
 *  (`reach +7% ±5`); a delta within its own half-width reads `reach ~0` — noise shown as noise. */
export function deltaLabel(u: UnlockInfo, d: number, rows?: number): string {
  // QA a946e04 (T: `reach ~0 at R6 · vs packs` on a 5-row set): a place past the set's last row is its end
  const at = u.insert_at !== undefined && (rows === undefined || u.insert_at < rows) ? u.insert_at : undefined;
  const where = !isCard(u) ? "" : at !== undefined ? /* copy:unlock_card */ ` at R${at + 1}` : /* copy:unlock_card */ " at end";
  // Cut 18 §5: a card whose best reach is within its ± names when it matters (`reach ~0 at R1 · vs archers`) — every card read
  // `reach ~0 at R4` to both raters, so they skipped them all
  // Cut 22 §4: a move is signed points (`reach +12 ±4`), `≈` inside its ± — never a `%`, which reads as a chance
  // Cut 24 §4: `≈` carries the ± it sits inside (`reach ≈ ±5 at R1`) — unresolved, not "no change"
  if (deltaIsNoise(u)) return /* copy:unlock_card */ `reach ≈${u.pm ? ` ±${Math.max(1, Math.round(u.pm * 100))}` : ""}${where}${u.situation ? ` · ${situationLabel(u.situation)}` : ""}`;
  const pm = u.pm !== undefined ? ` ±${Math.max(1, Math.round(u.pm * 100))}` : "";
  return /* copy:unlock_card */ `reach ${d > 0 ? "+" : "−"}${Math.abs(d)}${pm}${where}`;
}
/** QA 92eb880 (N: "`corridor fighting · reach ~0 at R6` bought … the shaft went bank 81 % → 44 %, stall 35 %"): a card that raises the
 *  stall share at its place says so — `stall +11%` (the engine's `UnlockInfo.stall`, 0..1); empty under a point. */
export function stallLabel(u: UnlockInfo): string {
  const pts = u.stall !== undefined && !u.owned ? Math.round(u.stall * 100) : 0;
  return pts > 0 ? /* copy:callout */ `stall +${pts}` : "";
}
/** Cut 18 §5: the foe tag a card answers, as the foes it meets (`ranged` → `vs archers`). */
/* copy:unlock_card */
const SITUATION: Record<string, string> = { ranged: "archers", gas: "gas", pack: "packs", thief: "thieves", boss: "bosses", caster: "casters", heavy: "brutes", summoner: "summoners" };
export const situationLabel = (tag: string): string => /* copy:unlock_card */ `vs ${SITUATION[tag] ?? tag.replace(/_/g, " ")}`;
/** Cut 18 §5: an unlock the lineage can buy with gold now — a gold price, gold enough, and no gate but the marks (a fact, a
 *  prerequisite, `fill rows` stop gold too). */
export function goldAffordable(u: UnlockInfo, gold: number): boolean {
  const g = goldPrice(u);
  return g > 0 && gold >= g && (!u.needs || /^◆\d+ more$/.test(u.needs));
}
/** Cut 18 §5: both prices on a tile — `◆3 · $450` (the gold path seen without opening the sheet); `◆3` alone without a gold price. */
export function priceLabel(u: UnlockInfo): string {
  const g = goldPrice(u);
  // QA 92eb880 (M: "`class: rogue ⊘ bank once` shows no price while ranger/caster do"): a door that costs nothing reads `free`
  // QA 1a2a4a9 (O: "`◆2 · $300` read as one price; the sheet reveals either buys it"): the two prices read as a choice — `◆2 / $300`
  return [u.cost ? `◆${u.cost}` : "", g ? `$${g}` : ""].filter(Boolean).join(/* copy:label */ " or ") || (u.owned ? "" : /* copy:label */ "free");
}
/** QA 92eb880 (M: "`+1 ROW` and `+1 VAULT` say nothing of what they give"): a counted unlock's effect as numbers — `rows 4 → 5`,
 *  `vault 1 → 2`, `party 1 → 2`, `supplies 3 → 5`; undefined for others. */
export function effectLine(app: App, u: UnlockInfo): string | undefined {
  const L = app.lineage;
  if (/^row\d+$/.test(u.id)) return /* copy:callout */ `rows ${app.vocab.max_rows} → ${app.vocab.max_rows + 1}`;
  // QA 912e135 (qaW: `vault 1 → 2` before any vault tile existed, then `VAULT 0/2`): while the vault is not yet on the console its first
  // slot was never seen — the line reads what the buy shows (`vault · 2 slots`)
  if (/^vault\d$/.test(u.id)) { const n = vaultSlots(L.unlocks); return revealed(app).has("vault") ? /* copy:callout */ `vault ${n} → ${n + 1}` : /* copy:callout */ `vault · ${n + 1} slots`; }
  if (/^party_slot_\d$/.test(u.id)) return /* copy:callout */ `party ${L.party_slots} → ${L.party_slots + 1}`;
  return undefined;
}
/** QA 92eb880 (N: "`verb: throw` sheet has only prices, no line of what it does"): a verb unlock shows the verb it adds as a chip. */
const VERB_OF: Record<string, string> = { throw: "throw", tame: "tame" };
/** Cut 13 §5: |delta| within its half-width. */
export const deltaIsNoise = (u: UnlockInfo): boolean => u.delta !== undefined && u.pm !== undefined && Math.abs(u.delta) <= u.pm;
/** The delta in whole points as the shelf shows it; 0 = nothing to show (no delta, or a bare 0 without a `pm` to call it noise). */
export function deltaPts(u: UnlockInfo): number {
  if (u.delta === undefined) return 0;
  const d = Math.round(u.delta * 100);
  return d || (deltaIsNoise(u) ? 1e-9 : 0);   // a `~0` still paints (a truthy non-integer the label never prints)
}
/** The delta's class: `up` · `down` · `flat` (`~0`). */
export const deltaClass = (u: UnlockInfo, d: number): string => (deltaIsNoise(u) ? "flat" : d > 0 ? "up" : "down");
/** Cut 15 §2: the card's gold price as the catalogue sends it (0: owned, free, or an old core without gold prices). */
export const goldPrice = (u: UnlockInfo): number => (u.owned ? 0 : u.gold ?? 0);
/** Cut 9 §2: the sheet behind an unlock card; `after` runs once a buy went through (the report repaints itself with it).
 *  Cut 12 §1: a card never takes a row (card rows sit outside `max_rows`). Cut 15 §2: both prices on the title line
 *  (`◆3 · $450`) and two buttons, `◆ buy` (marks) and `$ buy` (gold, the price climbing per gold buy), each off when short. */
export function openUnlockSheet(app: App, u: UnlockCard, after?: () => void): void {
  openSheet((close) => {
    const d = deltaPts(u);
    // the gate as of now, not as of the card's paint: a card painted before a buy or a report can carry a stale `available`
    // (QA B on 952e306: "CLASS: RANGER ◆6 · ⊘ ◆2 more has an active buy; tapping it did nothing"); any `needs` or a marks
    // shortfall against the live lineage turns `buy` off
    const short = Math.max(0, u.cost - app.lineage.marks);
    const gateNeeds = u.needs && !/^◆\d+ more$/.test(u.needs) ? u.needs : undefined;   // a gate that is not the marks
    const needs = gateNeeds ?? (short ? /* copy:unlock_card */ `◆${short} more` : undefined);
    const can = (u.available || (!!u.needs && !gateNeeds)) && !needs;
    // gold buys past a marks shortfall, never past another gate (a fact, a prerequisite, `fill rows`)
    const gold = goldPrice(u), canGold = gold > 0 && !gateNeeds && app.lineage.gold >= gold;
    // a short `$ buy` says by how much, as a supply chip does (QA on 3d71c33: "`$ buy` disabled at $210 vs $300 with no `$90 short`")
    const goldShort = gold > 0 && !gateNeeds && app.lineage.gold < gold ? gold - app.lineage.gold : 0;
    let sent = false;
    // the card's place as this sheet shows it (the catalogue's `insert_at`, never past the set's end)
    const joinAt = u.insert_at !== undefined ? Math.min(u.insert_at, app.rules.rows.length) : undefined;
    const go = (withGold: boolean) => (): void => {
      if (sent) return; sent = true;
      void app.buy(u.id, withGold, isCard(u) ? { join: u.auto_insert === true, at: joinAt } : undefined).then((ok) => { close(); if (ok) after?.(); });   // QA a946e04: the buy does what the sheet said, where it said
    };
    const buy = h("button", { class: `btn primary buy marks${can ? "" : " off"}`, disabled: !can, onclick: go(false) }, "◆ ", /* copy:button */ "buy");
    const buyGold = gold ? h("button", { class: `btn buy gold${canGold ? "" : " off"}`, disabled: !canGold, onclick: go(true) }, "$ ", /* copy:button */ "buy") : "";
    return h("div", { class: "sheet-body unlock-sheet" },
      h("div", { class: "label row-label" }, u.label, " ", h("span", { class: "num cost" }, `◆${u.cost}`, gold ? h("span", { class: "gold-price" }, /* copy:label */ ` or $${gold}`) : "")),
      u.rows?.length ? h("div", { class: "card-rows" }, ...u.rows.map((r) => h("div", { class: "row locked" }, rowChips(r))))
        : VERB_OF[u.id] ? h("div", { class: "card-rows" }, h("div", { class: "row locked" }, h("div", { class: "chips" }, h("span", { class: "chip verb locked" }, VERB_OF[u.id])))) : "",
      effectLine(app, u) ? h("div", { class: "effect-line num" }, effectLine(app, u)!) : "",
      // Cut 23 §3 (AJ: "paid cards are rows I could type"): what the card holds that no typed row can (the core's `carries`)
      u.carries ? h("div", { class: "carries num" }, h("span", { class: "dim" }, /* copy:label */ "holds "), u.carries) : "",
      // QA 92eb880 (N: "`⊘ ◆1 more` while `$ buy` is enabled"): a marks shortfall the gold covers is no lock — the line drops its `⊘`
      needs ? h("div", { class: "needs-line dim" }, canGold && !gateNeeds ? "" : "⊘ ", needs.replace(/_/g, " ")) : "",
      d ? h("div", { class: `num delta ${deltaClass(u, d)}` }, deltaLabel(u, d, app.rules.rows.length)) : "",   // Cut 10 §3 / Cut 12 §1 / Cut 13 §5
      stallLabel(u) ? h("div", { class: "num delta down stall-risk" }, stallLabel(u)) : "",   // QA 92eb880
      h("div", { class: "buy-pair" }, buy, buyGold),
      goldShort ? h("div", { class: "needs-line dim num gold-short" }, /* copy:callout */ `$${goldShort} short`) : "",
      // QA a946e04 (S: `+1 ROW ◆2 or $300 · each $ buy +25%`, then `◆4 or $600` after a ◆ buy): a chain's card names what the next step
      // costs — the engine's `next` (its marks; its gold after a ◆ buy, and after a $ buy when that differs) — rather than a rate
      nextPrice(app, u) ?? (gold && u.cost ? h("div", { class: "dim num gold-climb" }, /* copy:callout */ `each $ buy +${goldClimb(gold, u.cost)}%`) : ""),
      // QA a946e04 (T: three cards bought, all three went into the rules): a card that will not join the set on its buy says so — it is
      // owned, and its chip's `add` puts it in
      isCard(u) ? h("div", { class: `dim num card-joins${u.auto_insert === true ? " joins" : ""}` },
        u.auto_insert === true ? /* copy:unlock_card */ `joins at R${(joinAt ?? app.rules.rows.length) + 1}` : (u.owned ? /* copy:unlock_card */ "owned · add to rules" : /* copy:unlock_card */ "buy, then add to rules")) : "");   // QA 912e135 (qaW: `after buy · add separately` unexplained)   // QA 778fa1b (qaV: `owned · add separately` before the buy read as owned)
  });
}
/** QA a946e04: the chain's next step as the sheet shows it — `next ◆4 or $600` (its price once this one is bought with marks) and,
 *  when a $ buy would move it, `$ buy → $750`. The engine's `UnlockInfo.next`, else the catalogue's own entry for the next step
 *  (its gold price is today's — a ◆ buy leaves it). Null when the card is no chain's step. */
function nextPrice(app: App, u: UnlockInfo): HTMLElement | null {
  const id = Object.entries(AFTER).find(([, prev]) => prev === u.id)?.[0];
  const cat = id ? app.unlockCat.find((x) => x.id === id) : undefined;
  // Cut 25 §6 (AN: row 8 bought, the sheet read `next ◆8 or $6050` — "cheaper next, and I'm at the cap?"): the last step of a chain, or
  // one whose next waits on a gate the marks and gold cannot pass (`3 boss kinds`), reads `max` (and the gate), never a price
  const gate = cat?.needs && !/^◆\d+ more$/.test(cat.needs) && !["rows full", "fill rows", u.id].includes(cat.needs) ? cat.needs : undefined;
  if (/^(row|vault|party_slot_)/.test(u.id) && (!id || gate)) return h("div", { class: "dim num next-price max" }, /* copy:callout */ "max", gate ? h("span", { class: "gate" }, ` · ⊘ ${gate.replace(/_/g, " ")}`) : "");
  const n = u.next ?? (cat && !cat.owned ? { id: cat.id, cost: cat.cost, gold: cat.gold ?? 0, gold_after_gold: 0 } : undefined);
  if (!n) return null;
  const price = [n.cost ? `◆${n.cost}` : "", n.gold ? `$${n.gold}` : ""].filter(Boolean).join(/* copy:label */ " or ");
  if (!price) return null;
  return h("div", { class: "dim num next-price" }, /* copy:callout */ `next ${price}`,
    // QA 778fa1b (qaV: `next ◆4 or $600 · $ buy → $750` — "two next prices on one line"; qaU: a $ buy raised `verb: throw` and `cond:
    // alert` too, unsaid): a $ buy raises every $ price — the line says so, by how much, not a second price for this one
    n.gold_after_gold && n.gold && n.gold_after_gold !== n.gold ? h("span", { class: "after-gold" }, /* copy:callout */ ` · each $ buy: $ prices +${Math.round((n.gold_after_gold / n.gold - 1) * 100)}%`) : "");   // QA 912e135 (qaW: `$ buy: all $ +25%` unexplained)
}
/** The core's gold price is `GOLD_PER_MARK × cost × (4 + gold_buys) / 4` (meta.rs): each gold buy raises every gold price by
 *  1 / (4 + gold_buys) — 25 % at the first, 20 % at the second, … — read back off this card's price. */
export function goldClimb(gold: number, cost: number): number {
  const n = Math.max(0, Math.round((gold * 4) / (GOLD_PER_MARK * cost)) - 4);
  return Math.round(100 / (4 + n));
}
const GOLD_PER_MARK = 150;   // core meta.rs
/** Cut 6 §4: a tactic card (it becomes a row when bought). */
export const isCard = (u: UnlockInfo): boolean => /^card:/.test(LABEL[u.id] ?? "");
/** The sheet behind an owned shelf chip: the buy sheet's title line, the rows, and — for a card whose row the set no longer
 *  holds — `insert`, which puts the card's row back where a buy would (before the engagement row: `app.insertCard`; the
 *  catalogue carries `insert_at` only while unowned) and opens the camp on it (QA on 952e306: "owned card chip opens a
 *  titleless read-only sheet, no way to re-insert a dropped card (◆3 spent)"). */
export function openOwnedSheet(app: App, u: UnlockCard): void {
  openSheet((close) => {
    const insert = isCard(u) && !app.holdsCard(u.id)
      ? h("button", { class: "btn primary wide", onclick: () => { close(); addCard(app, u); } }, /* copy:button */ "add")
      : "";
    return h("div", { class: "sheet-body unlock-sheet" },
      h("div", { class: "label row-label" }, u.label),
      u.rows?.length ? h("div", { class: "card-rows" }, ...u.rows.map((r) => h("div", { class: "row locked" }, rowChips(r)))) : "",
      insert);
  });
}
/** QA e75ec29: an owned card off the set goes in at its measured place (`insert_at`, sent for owned cards too), else before the
 *  engagement row; the camp opens on it. */
export function addCard(app: App, u: UnlockInfo): void {
  // never past the engagement row (a catalogue's `insert_at` measured on another set would land it where it never acts)
  const eng = engagementRow(app.rules.rows);
  const i = app.insertCard(u.id, u.insert_at !== undefined ? Math.min(u.insert_at, eng) : undefined);
  app.go({ kind: "camp", highlight: i });
}
/** Cut 6 §6: owned entries that carry rows (cards, automations) — the shelf keeps them as chips that open their rows. */
export function ownedRows(catalogue: UnlockInfo[]): UnlockCard[] {
  // QA 92eb880 (N: "`verb: throw` and `class: rogue` bought, never appear in the `· owned` row"): every owned purchase but the counted
  // steps (`+1 row` · `+1 vault` · `+1 party`, which the counts already show) and the free class
  return catalogue.filter((u) => u.owned && (u.rows?.length || (LABEL[u.id] && !/^(row\d+|vault\d|party_slot_\d)$/.test(u.id) && !isFreeClass(u.id))))
    .map((u) => ({ ...u, label: LABEL[u.id] ?? u.id.replace(/_/g, " "), gated: false }));
}
export const vaultSlots = (owned: string[]): number => 1 + ["vault2", "vault3", "vault4", "vault5"].filter((u) => owned.includes(u)).length;
export const supplyCap = (owned: string[]): number => (owned.includes("supply_cap_5") ? 5 : 3);

/** Classes the picker lists: the known ladders ∪ whatever the lineage carries levels for ∪ class unlocks in the
 *  catalogue. `owned` = the free class or its unlock bought. */
export function classList(L: Lineage, catalogue: UnlockInfo[] = []): { cls: string; owned: boolean; level: number }[] {
  const ids = new Set<string>([...CLASSES, ...Object.keys(L.classes ?? {}), ...catalogue.filter((u) => /^class:/.test(LABEL[u.id] ?? "")).map((u) => u.id)]);
  return [...ids].map((cls) => ({ cls, owned: isFreeClass(cls) || L.unlocks.includes(cls), level: L.classes?.[cls]?.level ?? 1 }));
}
