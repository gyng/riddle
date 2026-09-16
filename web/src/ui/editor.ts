// Rule editor: rows as token chips `[cond] [cond] → [verb]`, drag grip to reorder (pointer events),
// tap a chip to swap it from the unlocked vocabulary (bottom sheet). Thumb-sized targets.
// Cut 4 §1: over budget (a patch on a full set) shows `5/4` in red and marks the rows the engine would drop;
// nothing is evicted. Cut 4 §9: a tactic-card row (`{v:"tactic"}`) is a locked chip `[card] thief guard`:
// not editable, but deletable and draggable, so the player sees where the card sits.
// Cut 6 §4: over budget, the rows marked to drop are the card rows first (a card is the newest row), then the last
// player rows. Cut 6 §6: tapping a `[card]` chip opens a sheet with the card's rows as read-only chips (`cardRows`).
import type { App } from "../app";
import type { Cond, Row, RuleSet, Verb, Vocabulary } from "../engine/types";
import { h, clear, flash } from "./dom";
import { openSheet } from "./sheet";
import { NUMS, PCT, condLabel, condName, needsN, sameCond, sameVerb, verbLabel } from "./tokens";

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

  function refresh(): void {
    clear(list); clear(foot);
    const n = rows().length, max = vocab().max_rows, over = n > max;
    const drop = over ? dropRows(rows(), max) : new Set<number>();
    rows().forEach((row, i) => list.appendChild(rowEl(row, i, drop.has(i))));
    foot.append(
      h("span", { class: `num ${over ? "over" : "dim"}` }, `${n}/${max}`),
      n < max ? h("button", { class: "btn ghost", onclick: () => { rows().push(defaultRow()); commit(); } }, "+") : "",
    );
    if (hl !== undefined && performance.now() < hlUntil) { const r = list.children[hl] as HTMLElement | undefined; if (r) { flash(r, "hl", Math.max(600, hlUntil - performance.now())); r.scrollIntoView({ block: "center" }); } }
    else hl = undefined;
  }

  function defaultRow(): Row {
    const V = vocab(); const v = V.verbs[0] ?? { v: "attack" }; const c = V.conds[0] ?? { k: "hp<" };
    return { conds: [{ ...c, n: needsN(c.k) ? 50 : undefined }], verb: { ...v } };
  }

  function rowEl(row: Row, i: number, drop = false): HTMLElement {
    const chips = h("div", { class: "chips" });
    const card = row.verb.v === "tactic";
    if (card) {
      row.conds.forEach((c) => chips.appendChild(h("span", { class: "chip cond locked" }, condLabel(c))));
      if (row.conds.length) chips.appendChild(h("span", { class: "arrow" }, "→"));
      const id = row.verb.a ?? "";
      const inner = [h("small", { class: "dim" }, /* copy:rule_token */ "[card]"), " ", id.replace(/_/g, " ")];
      const cardRows = bind.cardRows?.(id);
      chips.appendChild(cardRows?.length ? h("button", { class: "chip verb locked", onclick: () => openRowsSheet(cardRows) }, ...inner) : h("span", { class: "chip verb locked" }, ...inner));
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
          if (needsN(c.k)) { pickN(body, c, (n) => { row.conds[ci] = { ...c, n }; commit(); close(); }, existing && sameCond(existing, c) ? existing.n : undefined); return; }
          row.conds[ci] = { ...c }; commit(); close();
        } }, condName(c.k) + (c.t ? ` ${c.t.replace(/_/g, " ")}` : "")));
      }
      body.appendChild(grid);
      if (existing) body.appendChild(h("button", { class: "btn ghost wide", onclick: () => { row.conds.splice(ci, 1); commit(); close(); } }, "×"));
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
        grid.appendChild(h("button", { class: `chip verb${on ? " on" : ""}`, onclick: () => { row.verb = { ...v } as Verb; commit(); close(); } }, verbLabel(v)));
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
