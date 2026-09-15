// Rule editor: rows as token chips `[cond] [cond] → [verb]`, drag grip to reorder (pointer events),
// tap a chip to swap it from the unlocked vocabulary (bottom sheet). Thumb-sized targets.
import type { App } from "../app";
import type { Cond, Row, RuleSet, Verb, Vocabulary } from "../engine/types";
import { h, clear, flash } from "./dom";
import { openSheet } from "./sheet";
import { NUMS, PCT, condLabel, condName, needsN, sameCond, sameVerb, verbLabel } from "./tokens";

export type Editor = { el: HTMLElement; refresh(): void };
/** What the editor edits: the hero's active set, or a companion's own rows. */
export type Binding = { rules(): RuleSet; vocab(): Vocabulary; changed(): void };
export const heroBinding = (app: App): Binding => ({ rules: () => app.rules, vocab: () => app.vocab, changed: () => app.rulesChanged() });

export function renderEditor(bind: Binding, highlight?: number): Editor {
  const list = h("div", { class: "rows" });
  const foot = h("div", { class: "rows-foot" });
  const el = h("section", { class: "editor" }, list, foot);
  let hl = highlight;
  const vocab = (): Vocabulary => bind.vocab();

  function rows(): Row[] { return bind.rules().rows; }
  function commit(): void { bind.changed(); refresh(); }

  function refresh(): void {
    clear(list); clear(foot);
    rows().forEach((row, i) => list.appendChild(rowEl(row, i)));
    const max = vocab().max_rows;
    foot.append(
      h("span", { class: "dim num" }, `${rows().length}/${max}`),
      rows().length < max ? h("button", { class: "btn ghost", onclick: () => { rows().push(defaultRow()); commit(); } }, "+") : "",
    );
    if (hl !== undefined) { const r = list.children[hl] as HTMLElement | undefined; if (r) { flash(r, "hl", 2400); r.scrollIntoView({ block: "center" }); } hl = undefined; }
  }

  function defaultRow(): Row {
    const V = vocab(); const v = V.verbs[0] ?? { v: "attack" }; const c = V.conds[0] ?? { k: "hp<" };
    return { conds: [{ ...c, n: needsN(c.k) ? 50 : undefined }], verb: { ...v } };
  }

  function rowEl(row: Row, i: number): HTMLElement {
    const chips = h("div", { class: "chips" });
    row.conds.forEach((c, ci) => chips.appendChild(h("button", { class: "chip cond", onclick: () => pickCond(row, ci) }, condLabel(c))));
    if (row.conds.length < 2) chips.appendChild(h("button", { class: "chip cond add", onclick: () => pickCond(row, row.conds.length) }, "+"));
    chips.appendChild(h("span", { class: "arrow" }, "→"));
    chips.appendChild(h("button", { class: "chip verb", onclick: () => pickVerb(row) }, verbLabel(row.verb)));
    const grip = h("button", { class: "grip", onpointerdown: (e) => startDrag(e as PointerEvent, i) }, "≡", h("small", { class: "rn num" }, `R${i + 1}`));
    const x = h("button", { class: "x", onclick: () => { rows().splice(i, 1); commit(); } }, "×");
    return h("div", { class: "row", "data-i": i }, grip, chips, x);
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
      if (to !== from) { const rs = rows(); const [r] = rs.splice(from, 1); rs.splice(to, 0, r); hl = to; commit(); }
    };
    grip.addEventListener("pointermove", move); grip.addEventListener("pointerup", up); grip.addEventListener("pointercancel", up);
  }

  refresh();
  return { el, refresh };
}
