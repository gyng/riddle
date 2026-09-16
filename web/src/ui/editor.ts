// Rule editor: rows as token chips `[cond] [cond] → [verb]`, drag grip to reorder (pointer events),
// tap a chip to swap it from the unlocked vocabulary (bottom sheet). Thumb-sized targets.
// Cut 4 §1: over budget (a patch on a full set) shows `5/4` in red and marks the rows the engine would drop;
// nothing is evicted. Cut 4 §9: a tactic-card row (`{v:"tactic"}`) is a locked chip `[card] thief guard`:
// not editable, but deletable and draggable, so the player sees where the card sits.
// Cut 6 §4: over budget, the rows marked to drop are the card rows first (a card is the newest row), then the last
// player rows. Cut 6 §6: tapping a `[card]` chip opens a sheet with the card's rows as read-only chips (`cardRows`).
// Cut 8B §4: a `[card]` row shows its rows inline underneath as dim read-only chips (the sheet stays for the shelf's
// automations); two adjacent rows that make a combo (`vocab.combos`, engine data) carry a small bracket with the
// combo's name to the right of the pair (`⌐ opener`), repainted on every edit.
// Cut 9 §1: the cond sheet shows the vocabulary's `locked` tokens dim with their `needs` text and no handler. Cut 9 §4: a
// `[card]` chip carries the card's trigger (`[card] pack break · foes ≥ 2`, the first inline row's conds).
import type { App } from "../app";
import type { Cond, Row, RuleSet, Verb, Vocabulary } from "../engine/types";
import { h, clear, flash } from "./dom";
import { openSheet } from "./sheet";
import { NUMS, PCT, combosIn, condLabel, condName, needsN, sameCond, sameVerb, verbLabel } from "./tokens";

export type Editor = { el: HTMLElement; refresh(): void };
/** What the editor edits: the hero's active set, or a companion's own rows. */
export type Binding = { rules(): RuleSet; vocab(): Vocabulary; changed(): void; cardRows?(id: string): Row[] | undefined };
export const heroBinding = (app: App): Binding => ({ rules: () => app.rules, vocab: () => app.vocab, changed: () => app.rulesChanged(), cardRows: (id) => app.cardRows(id) });

/** Cut 6 §6: a row as read-only chips (`foe: ranged → kite`), shared by the card sheet and the shelf. */
export function rowChips(row: Row): HTMLElement {
  const chips = h("div", { class: "chips" });
  row.conds.forEach((c) => chips.appendChild(h("span", { class: "chip cond locked" }, condLabel(c))));
  if (row.conds.length) chips.appendChild(h("span", { class: "arrow" }, "→"));
  chips.appendChild(h("span", { class: "chip verb locked" }, verbLabel(row.verb)));
  return chips;
}
/** Cut 6 §6: the sheet behind a `[card]` row or an owned automation: its rows as chips, nothing else. */
export function openRowsSheet(rows: Row[]): void {
  openSheet(() => h("div", { class: "sheet-body card-rows" }, ...rows.map((r) => h("div", { class: "row locked" }, rowChips(r)))));
}
/** Cut 9 §4: a card's trigger as text — the conds of its first row (`foes ≥ 2 · corridor`); `always` when that row has none. */
export function cardTrigger(cardRows: Row[] | undefined): string | undefined {
  const first = cardRows?.[0];
  if (!first) return undefined;
  return first.conds.length ? first.conds.map(condLabel).join(" · ") : /* copy:rule_token */ "always";
}
/** Cut 9 §4: the 1-based index of the first `[card]` row that sits above any of the player's rows (every non-card row: a
 *  card eats the turn from whatever is under it, whoever offered that row), else undefined. */
export function cardAbovePlayer(rows: Row[]): number | undefined {
  const last = rows.map((r) => r.verb.v !== "tactic").lastIndexOf(true);
  const i = rows.findIndex((r, k) => r.verb.v === "tactic" && k < last);
  return i >= 0 ? i + 1 : undefined;
}
/** Cut 6 §4: which rows an over-budget set marks to drop: card rows (newest) first, then the last rows. */
export function dropRows(rows: Row[], max: number): Set<number> {
  const out = new Set<number>(); let n = rows.length - max;
  for (let i = rows.length - 1; i >= 0 && n > 0; i--) if (rows[i].verb.v === "tactic") { out.add(i); n--; }
  for (let i = rows.length - 1; i >= 0 && n > 0; i--) if (!out.has(i)) { out.add(i); n--; }
  return out;
}

export function renderEditor(bind: Binding, highlight?: number): Editor {
  const list = h("div", { class: "rows" });
  const foot = h("div", { class: "rows-foot" });
  const el = h("section", { class: "editor" }, list, foot);
  let hl = highlight;
  let hlUntil = highlight !== undefined ? performance.now() + 2400 : 0; // survives the camp's repaint right after mount
  const vocab = (): Vocabulary => bind.vocab();

  function rows(): Row[] { return bind.rules().rows; }
  function commit(): void { bind.changed(); refresh(); }
  /** Cut 7 §2: a row the player edits any token of (or adds) is the player's, whatever offered it. */
  function edited(row: Row): void { row.origin = "player"; commit(); }

  function refresh(): void {
    clear(list); clear(foot);
    const n = rows().length, max = vocab().max_rows, over = n > max;
    const drop = over ? dropRows(rows(), max) : new Set<number>();
    rows().forEach((row, i) => list.appendChild(rowEl(row, i, drop.has(i))));
    // Cut 8B §4: the bracket sits on the pair's first row and reaches the second (`data-combo` is the name, engine data)
    for (const c of combosIn(rows(), vocab().combos)) {
      const a = list.children[c.rows[0]] as HTMLElement | undefined, b = list.children[c.rows[1]] as HTMLElement | undefined;
      if (!a || !b) continue;
      a.classList.add("combo-a"); b.classList.add("combo-b"); a.dataset.combo = c.name;
    }
    foot.append(
      h("span", { class: `num ${over ? "over" : "dim"}` }, `${n}/${max}`),
      n < max ? h("button", { class: "btn ghost", onclick: () => { rows().push(defaultRow()); commit(); } }, "+") : "",
    );
    if (hl !== undefined && performance.now() < hlUntil) { const r = list.children[hl] as HTMLElement | undefined; if (r) { flash(r, "hl", Math.max(600, hlUntil - performance.now())); r.scrollIntoView({ block: "center" }); } }
    else hl = undefined;
  }

  function defaultRow(): Row {
    const V = vocab(); const v = V.verbs[0] ?? { v: "attack" }; const c = V.conds[0] ?? { k: "hp<" };
    return { conds: [{ ...c, n: needsN(c.k) ? 50 : undefined }], verb: { ...v }, origin: "player" };
  }

  function rowEl(row: Row, i: number, drop = false): HTMLElement {
    const chips = h("div", { class: "chips" });
    const card = row.verb.v === "tactic";
    if (card) {
      row.conds.forEach((c) => chips.appendChild(h("span", { class: "chip cond locked" }, condLabel(c))));
      if (row.conds.length) chips.appendChild(h("span", { class: "arrow" }, "→"));
      const id = row.verb.a ?? "";
      const cardRows = bind.cardRows?.(id);
      // Cut 9 §4: the card's trigger — its first row's conds as text — on the chip: `[card] pack break · foes ≥ 2`
      const trigger = cardTrigger(cardRows);
      const inner = [h("small", { class: "dim" }, /* copy:rule_token */ "[card]"), " ", id.replace(/_/g, " "), trigger ? h("small", { class: "dim trigger" }, ` · ${trigger}`) : ""];
      chips.appendChild(cardRows?.length ? h("button", { class: "chip verb locked", onclick: () => openRowsSheet(cardRows) }, ...inner) : h("span", { class: "chip verb locked" }, ...inner));
      // Cut 8B §4: the card's rows, inline and dim — rows the player could have written
      if (cardRows?.length) chips.appendChild(h("div", { class: "card-inline" }, ...cardRows.map((r) => rowChips(r))));
    } else {
      row.conds.forEach((c, ci) => chips.appendChild(h("button", { class: "chip cond", onclick: () => pickCond(row, ci) }, condLabel(c))));
      if (row.conds.length < 2) chips.appendChild(h("button", { class: "chip cond add", onclick: () => pickCond(row, row.conds.length) }, "+"));
      chips.appendChild(h("span", { class: "arrow" }, "→"));
      chips.appendChild(h("button", { class: "chip verb", onclick: () => pickVerb(row) }, verbLabel(row.verb)));
    }
    const grip = h("button", { class: "grip", onpointerdown: (e) => startDrag(e as PointerEvent, i) }, "≡", h("small", { class: "rn num" }, `R${i + 1}`));
    const x = h("button", { class: "x", onclick: () => { rows().splice(i, 1); commit(); } }, "×");
    return h("div", { class: `row${card ? " locked" : ""}${drop ? " drop" : ""}`, "data-i": i }, grip, chips, x);
  }

  // --- sheets ---
  function pickCond(row: Row, ci: number): void {
    const existing = row.conds[ci];
    openSheet((close) => {
      const body = h("div", { class: "sheet-body" });
      const grid = h("div", { class: "grid" });
      for (const c of vocab().conds) {
        if (row.conds.some((rc, j) => j !== ci && sameCond(rc, c))) continue;
        const on = existing && sameCond(existing, c);
        grid.appendChild(h("button", { class: `chip cond${on ? " on" : ""}`, onclick: () => {
          if (needsN(c.k)) { pickN(body, c, (n) => { row.conds[ci] = { ...c, n }; edited(row); close(); }, existing && sameCond(existing, c) ? existing.n : undefined); return; }
          row.conds[ci] = { ...c }; edited(row); close();
        } }, condName(c.k) + (c.t ? ` ${c.t.replace(/_/g, " ")}` : "")));
      }
      // Cut 9 §1: locked tokens (engine data: `Vocabulary.locked`) sit dim after the offered ones with their gate as text and
      // no handler at all — a `span`, not a button — so the sheet never offers what a run would refuse
      for (const l of vocab().locked ?? []) {
        if (vocab().conds.some((c) => sameCond(c, l.cond))) continue;
        grid.appendChild(h("span", { class: "chip cond locked off", "aria-disabled": "true" }, "⊘ ", condName(l.cond.k) + (l.cond.t ? ` ${l.cond.t.replace(/_/g, " ")}` : ""), h("small", { class: "needs dim" }, l.needs.replace(/_/g, " "))));
      }
      body.appendChild(grid);
      if (existing) body.appendChild(h("button", { class: "btn ghost wide", onclick: () => { row.conds.splice(ci, 1); edited(row); close(); } }, "×"));
      return body;
    });
  }
  function pickN(body: HTMLElement, c: Cond, done: (n: number) => void, cur?: number): void {
    clear(body);
    const pctish = PCT.has(c.k);
    body.appendChild(h("div", { class: "sheet-head" }, condName(c.k)));
    const grid = h("div", { class: "grid nums" });
    for (const n of NUMS[c.k]) grid.appendChild(h("button", { class: `chip num${n === cur ? " on" : ""}`, onclick: () => done(n) }, `${n}${pctish ? "%" : ""}`));
    body.appendChild(grid);
  }
  function pickVerb(row: Row): void {
    openSheet((close) => {
      const grid = h("div", { class: "grid" });
      for (const v of vocab().verbs) {
        const on = sameVerb(row.verb, v);
        grid.appendChild(h("button", { class: `chip verb${on ? " on" : ""}`, onclick: () => { row.verb = { ...v } as Verb; edited(row); close(); } }, verbLabel(v)));
      }
      return h("div", { class: "sheet-body" }, grid);
    });
  }

  // --- drag to reorder ---
  function startDrag(e: PointerEvent, from: number): void {
    e.preventDefault();
    const rowEls = Array.from(list.children) as HTMLElement[];
    const me = rowEls[from]; if (!me) return;
    const grip = e.currentTarget as HTMLElement;
    grip.setPointerCapture(e.pointerId);
    const y0 = e.clientY; let to = from;
    me.classList.add("dragging");
    const others = rowEls.filter((r) => r !== me);
    const move = (ev: PointerEvent): void => {
      me.style.transform = /* copy:none */ `translateY(${ev.clientY - y0}px)`;
      to = others.filter((r) => { const b = r.getBoundingClientRect(); return b.top + b.height / 2 < ev.clientY; }).length;
      others.forEach((r, k) => { r.classList.toggle("before", k === to); r.classList.toggle("after", to === others.length && k === others.length - 1); });
    };
    const up = (): void => {
      grip.removeEventListener("pointermove", move); grip.removeEventListener("pointerup", up); grip.removeEventListener("pointercancel", up);
      me.classList.remove("dragging"); me.style.transform = "";
      rowEls.forEach((r) => r.classList.remove("before", "after"));
      if (to !== from) { const rs = rows(); const [r] = rs.splice(from, 1); rs.splice(to, 0, r); hl = to; hlUntil = performance.now() + 1600; commit(); }
    };
    grip.addEventListener("pointermove", move); grip.addEventListener("pointerup", up); grip.addEventListener("pointercancel", up);
  }

  refresh();
  return { el, refresh };
}
