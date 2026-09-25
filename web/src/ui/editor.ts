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
// Cut 10 §3: `▲▼` chips (44 px each) beside the drag handle move a row one step; the first row's ▲ and the last row's ▼ are off.
// Cut 12 §1: the rows are the player's — `max_rows` caps own rows only, card rows sit outside it (the chip reads `3/4 · 2 cards`,
// over budget marks the last own rows, `+` waits on own rows); picking a card verb clears the row's conds (a card row carries
// none) and the picker never offers a card another row already holds.
import type { App } from "../app";
import type { Cond, Row, RowWhy, RuleSet, Verb, Vocabulary } from "../engine/types";
import { h, clear, flash } from "./dom";
import { openSheet } from "./sheet";
import { NUMS, PCT, combosIn, depthNums, condLabel, condName, glossOf, isCardRow, needsN, ownRowCount, rowLabel, sameCond, sameVerb, verbLabel } from "./tokens";

export type Editor = { el: HTMLElement; refresh(): void; paintShadow(): void };
/** Cut 17 §2: the camp's tablets. `compact()` true: each row is one carved tablet (`R1  hp < 30% → drink unknown`), a single
 *  tap target that calls `onTablet(i)` (the camp turns editing on at that row) — a fresh lineage's camp before the `edit` tile;
 *  false: the tablet carries the editor — chips, ▲▼, ×, and `+` under the rows. */
export type EditorOpts = { compact?: () => boolean; onTablet?: (i: number) => void };
/** What the editor edits: the hero's active set, or a companion's own rows. */
export type Binding = { rules(): RuleSet; vocab(): Vocabulary; changed(): void; cardRows?(id: string): Row[] | undefined;
                        shadowedBy?(): (number | null)[];   // QA 92eb880: per row, the earlier row that takes all its moments (the engine's read)
                        nums?(k: string): number[] | undefined;   // Cut 21 §3: a cond's values read off the lineage (`depth ≥` to best + 2); else the table's
                        rowWhy?(): (RowWhy | null)[];             // Cut 23 §3: per row, what it did over the recent sends and why not (the core's `row_why`)
                        inert?(row: Row): string | undefined };   // QA 912e135: a row that cannot act yet, and what it waits on (`identify heal`)
/** QA 912e135 (qaW: the default `hp < 30% → drink heal` ran inert all night — `0/760 · blocked · unknown item` — while `has: heal` read
 *  `⊘ identify heal` in the cond sheet): a row whose verb uses a kind the lineage has not identified (the vocabulary's locked `has:`) and
 *  no packed supply of it (a bought one is known) waits on it — `identify heal` on the tablet. */
export function inertOf(app: App, r: Row): string | undefined {
  const a = r.verb.a;
  if (!a || a === "unknown" || !["drink", "read", "throw"].includes(r.verb.v)) return undefined;
  if (!(app.vocab?.locked ?? []).some((l) => l.cond.k === "item" && l.cond.t === a)) return undefined;
  if ((app.lineage?.supplies ?? []).some((s) => s.kind === a) || (app.lineage?.vault ?? []).some((v) => v.kind === a && app.loadout.includes(v.id))) return undefined;
  return /* copy:callout */ `identify ${a.replace(/_/g, " ")}`;
}
export const heroBinding = (app: App): Binding => ({ rules: () => app.rules, vocab: () => app.vocab, changed: () => app.rulesChanged(), cardRows: (id) => app.cardRows(id), shadowedBy: () => app.shadowedBy(),
  nums: (k) => k === "depth>=" ? depthNums(app.lineage?.best_depth ?? 0, app.vocab) : undefined, rowWhy: () => app.rowWhy(), inert: (r) => inertOf(app, r) });

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
  openSheet(() => h("div", { class: "sheet-body card-rows" }, h("div", { class: "label row-label" }, /* copy:label */ "card"), ...rows.map((r) => h("div", { class: "row locked" }, rowChips(r)))));
}
/** Cut 9 §4: a card's trigger as text — the conds of its first row (`foes ≥ 2 · corridor`); `always` when that row has none. */
export function cardTrigger(cardRows: Row[] | undefined): string | undefined {
  const first = cardRows?.[0];
  if (!first) return undefined;
  return first.conds.length ? first.conds.map(condLabel).join(" · ") : /* copy:rule_token */ "always";
}
/** Cut 6 §4: which rows an over-budget set marks to drop. Cut 12 §1: the last own rows (card rows never count). */
export function dropRows(rows: Row[], max: number): Set<number> {
  const out = new Set<number>(); let n = ownRowCount(rows) - max;
  for (let i = rows.length - 1; i >= 0 && n > 0; i--) if (!isCardRow(rows[i])) { out.add(i); n--; }
  return out;
}

export function renderEditor(bind: Binding, highlight?: number, opts: EditorOpts = {}): Editor {
  const list = h("div", { class: "rows" });
  const foot = h("div", { class: "rows-foot" });
  const el = h("section", { class: "editor" }, list, foot);
  let hl = highlight;
  let hlUntil = highlight !== undefined ? performance.now() + 2400 : 0; // survives the camp's repaint right after mount
  const vocab = (): Vocabulary => bind.vocab();
  const numsOf = (k: string): number[] | undefined => bind.nums?.(k) ?? NUMS[k];

  function rows(): Row[] { return bind.rules().rows; }
  function commit(): void { bind.changed(); refresh(); }
  /** Cut 7 §2: a row the player edits any token of (or adds) is the player's, whatever offered it. */
  function edited(row: Row): void { row.origin = "player"; commit(); }

  function refresh(): void {
    clear(list); clear(foot);
    const compact = opts.compact?.() ?? false;
    el.classList.toggle("compact", compact);
    if (compact) {
      // Cut 23 §3 (AJ: `foe: gas → throw unknown` fired 0/164, no word): a tablet with a why-not (the row sat through a send) opens it on
      // tap — `0/164 · no gas met`, `3/164 · blocked · no scroll` and the reason's gloss — with `edit` under it; else the tap edits
      rows().forEach((row, i) => { const b: HTMLButtonElement = h("button", { class: `row tablet compact${isCardRow(row) ? " locked" : ""}`, "data-i": i,
        onclick: () => { const w = bind.rowWhy?.()[i]; if (w) openWhy(i, b, () => opts.onTablet?.(i)); else opts.onTablet?.(i); } },
        h("span", { class: "rn num" }, `R${i + 1}`), h("span", { class: "rtext" }, rowLabel(row))); list.appendChild(b); });
      paintShadow();
      return;
    }
    const n = ownRowCount(rows()), cards = rows().length - n, max = vocab().max_rows, over = n > max;
    const drop = over ? dropRows(rows(), max) : new Set<number>();
    rows().forEach((row, i) => list.appendChild(rowEl(row, i, drop.has(i))));
    // Cut 8B §4: the bracket sits on the pair's first row and reaches the second (`data-combo` is the name, engine data)
    for (const c of combosIn(rows(), vocab().combos)) {
      const a = list.children[c.rows[0]] as HTMLElement | undefined, b = list.children[c.rows[1]] as HTMLElement | undefined;
      if (!a || !b) continue;
      a.classList.add("combo-a"); b.classList.add("combo-b"); a.dataset.combo = c.name;
    }
    // Cut 12 §1: own rows against the cap, the card rows counted beside (`3/4 · 2 cards`)
    // QA 912e135 (qaW: `4/5 · 1 card` under R1–R5, read as four rows shown): the cap names its unit and the card adds to it — `4/5 rows +
    // 1 card`, the tablets on screen the sum
    foot.append(
      h("span", { class: `num ${over ? "over" : "dim"}` }, `${n}/${max}`, cards ? /* copy:callout */ " rows" : "", cards ? h("small", { class: "dim cards" }, /* copy:callout */ ` + ${cards} card${cards === 1 ? "" : "s"}`) : ""),
      // QA 778fa1b (qaV friction: the new `hp < 50% → …` row landed last, under `foes ≥ 1 → attack nearest`, shadowed until stepped up 4
      // times): it goes in above the first own row with no hp cond (the broad engagement rows), under the hp rows before it
      n < max ? h("button", { class: "btn ghost", onclick: () => { const rs = rows(); const at = rs.findIndex((r) => r.verb.v !== "tactic" && !r.conds.some((c) => c.k === "hp<")); rs.splice(at < 0 ? rs.length : at, 0, defaultRow()); commit(); } }, "+") : "",
    );
    paintShadow();
    if (hl !== undefined && performance.now() < hlUntil) { const r = list.children[hl] as HTMLElement | undefined; if (r) { flash(r, "hl", Math.max(600, hlUntil - performance.now())); r.scrollIntoView({ block: "center" }); } }
    else hl = undefined;
  }

  /** QA 92eb880 (M, N: "R3 `hp < 30% → drink heal` under R1 `hp < 30% → return` … nothing marks it"): a row an earlier one shadows
   *  is a dim tablet carrying `↑ R1` — the row that takes its every moment (the engine's `shadowed_by`, repainted as forecasts land). */
  function paintShadow(): void {
    const sh = bind.shadowedBy?.() ?? [];
    const rows = bind.rules().rows;
    [...list.children].forEach((el, i) => {
      // QA e75ec29 (R: `foe: thief → attack thief` under `foes ≥ 1 → attack nearest` lost its `↑ R3`): the engine's read is what the sims
      // saw (a row that never met a thief is not "shadowed" there); a row an earlier attack row always pre-empts is marked by its form
      const by = sh[i] ?? (rows[i] ? formShadow(rows, i) : null);
      const on = by !== null && by !== undefined && by < i;
      el.classList.toggle("shadowed", on);
      for (const m of el.querySelectorAll(":scope > .rtext > .shadow-mark, :scope > .grip > .shadow-mark, :scope > .shadow-mark, :scope > .rtext > .inert-mark, :scope > .grip > .inert-mark, :scope > .inert-mark")) m.remove();
      // QA 912e135: a row waiting on an identification says so (dim, like a shadowed one); a shadowed row's mark wins
      const inert = !on && rows[i] ? bind.inert?.(rows[i]) : undefined;
      el.classList.toggle("inert", !!inert);
      if (inert) {
        const m = h("small", { class: "inert-mark num" }, `⊘ ${inert}`);
        const text = el.querySelector(":scope > .rtext"), grip = el.querySelector(":scope > .grip");
        if (text) text.appendChild(m); else if (grip) grip.appendChild(m); else el.appendChild(m);
      }
      if (!on) return;
      const mark = h("small", { class: "shadow-mark num", title: `R${by + 1}` }, `↑ R${by + 1}`);
      // compact: inside the tablet's text; editing: under the row's number on its grip (the chips stay the row's cond → verb)
      const text = el.querySelector(":scope > .rtext"), grip = el.querySelector(":scope > .grip");
      if (text) text.appendChild(mark); else if (grip) grip.appendChild(mark); else el.appendChild(mark);
    });
  }

  function defaultRow(): Row {
    // QA a946e04 (S: the new row `hp < 50% → attack nearest` was born dead under `foes ≥ 1 → attack nearest` — "the add had failed"):
    // the first verb no row of the set uses yet (a row that repeats an earlier row's verb under a narrower cond is what shadows)
    const V = vocab(); const used = new Set(rows().map((r) => `${r.verb.v}:${r.verb.a ?? ""}`));
    const v = V.verbs.find((x) => x.v !== "tactic" && !used.has(`${x.v}:${x.a ?? ""}`)) ?? V.verbs[0] ?? { v: "attack" }; const c = V.conds[0] ?? { k: "hp<" };
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
      // QA 912e135 (qaW: `[card] kite archers · foe: ranged · foe: telegraph` over the same conds drawn under it): with the card's rows
      // inline the trigger is theirs to show — the chip keeps the name
      const inner = [h("small", { class: "dim" }, /* copy:rule_token */ "[card]"), " ", id.replace(/_/g, " "), trigger && !cardRows?.length ? h("small", { class: "dim trigger" }, ` · ${trigger}`) : ""];
      chips.appendChild(cardRows?.length ? h("button", { class: "chip verb locked", onclick: () => openRowsSheet(cardRows) }, ...inner) : h("span", { class: "chip verb locked" }, ...inner));
      // Cut 8B §4: the card's rows, inline and dim — rows the player could have written
      if (cardRows?.length) chips.appendChild(h("div", { class: "card-inline" }, ...cardRows.map((r) => rowChips(r))));
    } else {
      row.conds.forEach((c, ci) => chips.appendChild(h("button", { class: "chip cond", onclick: (e: Event) => pickCond(row, ci, rowOf(e)) }, condLabel(c))));
      if (row.conds.length < 2) chips.appendChild(h("button", { class: "chip cond add", onclick: (e: Event) => pickCond(row, row.conds.length, rowOf(e)) }, "+"));
      chips.appendChild(h("span", { class: "arrow" }, "→"));
      chips.appendChild(h("button", { class: "chip verb", onclick: (e: Event) => pickVerb(row, rowOf(e)) }, verbLabel(row.verb)));
    }
    const grip = h("button", { class: "grip", onpointerdown: (e) => startDrag(e as PointerEvent, i) }, "≡", h("small", { class: "rn num" }, `R${i + 1}`));
    const n = rows().length;
    const swap = (to: number): void => { const rs = rows(); const [r] = rs.splice(i, 1); rs.splice(to, 0, r); hl = to; hlUntil = performance.now() + 1600; commit(); };
    const updown = h("div", { class: "updown" },
      h("button", { class: "step up", disabled: i === 0, onclick: () => swap(i - 1) }, "▲"),
      h("button", { class: "step down", disabled: i >= n - 1, onclick: () => swap(i + 1) }, "▼"));
    const x = h("button", { class: "x", onclick: () => { rows().splice(i, 1); commit(); } }, "×");
    return h("div", { class: `row tablet${card ? " locked" : ""}${drop ? " drop" : ""}`, "data-i": i }, grip, updown, chips, x);
  }

  // --- sheets ---
  /** Cut 23 §4: the row a chip sits on — its option sheet opens beside it, never over it. */
  const rowOf = (e: Event): HTMLElement | undefined => ((e.currentTarget as HTMLElement | null)?.closest(".row") as HTMLElement | null) ?? undefined;
  function pickCond(row: Row, ci: number, anchor?: HTMLElement): void {
    const existing = row.conds[ci];
    openSheet((close) => {
      const body = h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "cond"));   // Cut 13 §6: every sheet is titled
      // the row's `×` (remove this cond) sits at the top, above the ~90 tokens (QA on 952e306: "× at the very bottom of a ~90-entry list")
      if (existing) body.appendChild(h("button", { class: "btn ghost wide", onclick: () => { row.conds.splice(ci, 1); edited(row); close(); } }, "×"));
      // QA 92eb880 (M, N: "a threshold change is three taps … the chip opens the whole list, not the value"): a chip with a number
      // opens on its values (the current lit), the other conds under them — a threshold is one tap from the sheet
      if (existing && needsN(existing.k) && numsOf(existing.k)) {
        const pctish = PCT.has(existing.k);
        body.appendChild(h("div", { class: "sheet-head" }, condName(existing.k) + (existing.t ? ` ${existing.t.replace(/_/g, " ")}` : "")));
        body.appendChild(h("div", { class: "grid nums now" }, ...numsOf(existing.k)!.map((n) => h("button", { class: `chip num${n === existing.n ? " on" : ""}`, onclick: () => { row.conds[ci] = { ...existing, n }; edited(row); close(); } }, `${n}${pctish ? "%" : ""}`))));
      }
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
      return body;
    }, { anchor });
  }
  function pickN(body: HTMLElement, c: Cond, done: (n: number) => void, cur?: number): void {
    clear(body);
    const pctish = PCT.has(c.k);
    body.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "cond"));
    body.appendChild(h("div", { class: "sheet-head" }, condName(c.k)));
    const grid = h("div", { class: "grid nums" });
    for (const n of numsOf(c.k) ?? []) grid.appendChild(h("button", { class: `chip num${n === cur ? " on" : ""}`, onclick: () => done(n) }, `${n}${pctish ? "%" : ""}`));
    body.appendChild(grid);
  }
  function pickVerb(row: Row, anchor?: HTMLElement): void {
    openSheet((close) => {
      const grid = h("div", { class: "grid" });
      // QA 23ed91f (K: "`drink heal` is the default R1 verb, but it is missing from the VERB list"): the row's own verb is on the list
      // (lit) even when the vocabulary does not offer it today, so the picker never hides what the row does
      if (!vocab().verbs.some((v) => sameVerb(row.verb, v)) && row.verb.v !== "tactic") grid.appendChild(h("button", { class: "chip verb on", onclick: () => close() }, verbLabel(row.verb)));
      for (const v of vocab().verbs) {
        const on = sameVerb(row.verb, v);
        // Cut 12 §1: a set holds one row per card — a card another row already carries is not offered
        if (v.v === "tactic" && rows().some((r) => r !== row && isCardRow(r) && r.verb.a === v.a)) continue;
        grid.appendChild(h("button", { class: `chip verb${on ? " on" : ""}`, onclick: () => {
          row.verb = { ...v } as Verb;
          // Cut 12 §1: a card row carries no condition — picking a card verb clears the row's conds (they read as an
          // uneditable prefix otherwise) and the row is the card's, not the player's
          if (v.v === "tactic") { row.conds = []; row.origin = "card"; commit(); } else edited(row);
          close();
        } }, verbLabel(v)));
      }
      // QA 92eb880 (M: "`drink heal` is offered only on the row that already has it"): a verb another row holds that the vocabulary does
      // not offer today sits dim with its reason (a drink/read of an unknown kind), never selectable
      const seen = new Set<string>();
      for (const r of rows()) {
        if (r === row || r.verb.v === "tactic" || vocab().verbs.some((v) => sameVerb(r.verb, v)) || sameVerb(r.verb, row.verb)) continue;
        const key = verbLabel(r.verb); if (seen.has(key)) continue; seen.add(key);
        const why = (r.verb.v === "drink" || r.verb.v === "read") && r.verb.a && r.verb.a !== "unknown" ? /* copy:rule_token */ "unknown" : "";
        grid.appendChild(h("span", { class: "chip verb locked off", "aria-disabled": "true" }, "⊘ ", key, why ? h("small", { class: "needs dim" }, why) : ""));
      }
      return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "verb"), grid);
    }, { anchor });
  }

  /** Cut 23 §3: a row's why-not, anchored to its tablet (never over it): the core's line (`0/164 · no gas met`), the reason's gloss
   *  (`no item` → `none in pack`), the row that shadows it; `edit` when opened from a compact tablet. */
  function openWhy(i: number, anchor: HTMLElement, edit?: () => void): void {
    const row = rows()[i], w = bind.rowWhy?.()[i]; if (!row || !w) return;
    const gloss = bind.vocab().why_gloss;
    const reason = w.blocked?.why;
    const g = glossOf(gloss, reason);
    const by = bind.shadowedBy?.()[i] ?? formShadow(rows(), i);
    openSheet((close) => h("div", { class: "sheet-body row-why", "data-row": i },
      h("div", { class: "label row-label" }, /* copy:label */ "why", " ", h("small", { class: "num dim" }, `R${i + 1}`)),
      h("div", { class: "why-row chips-inline dim" }, rowLabel(row)),
      // QA 912e135 (qaW: `8/15317` on R1 and `23/16077` on R3 of one set — "denominators differ"): the count is over the sends the row
      // sat in (an edited row starts over), and the line says how many (`8/15317 · 14 sends`)
      h("div", { class: "why-line num" }, w.text, w.sends > 0 ? h("small", { class: "dim why-sends" }, /* copy:callout */ ` · ${w.sends} send${w.sends === 1 ? "" : "s"}`) : ""),
      g ? h("div", { class: "why-gloss num" }, h("span", { class: "dim" }, `${reason} · `), g) : "",
      by !== null && by !== undefined && by < i ? h("div", { class: "why-gloss num" }, /* copy:callout */ `↑ R${by + 1} first`) : "",
      edit ? h("button", { class: "btn primary wide why-edit", onclick: () => { close(); edit(); } }, /* copy:button */ "edit") : ""), { anchor });
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
      if (Math.abs(ev.clientY - y0) > 4) moved = true;
      me.style.transform = /* copy:none */ `translateY(${ev.clientY - y0}px)`;
      to = others.filter((r) => { const b = r.getBoundingClientRect(); return b.top + b.height / 2 < ev.clientY; }).length;
      others.forEach((r, k) => { r.classList.toggle("before", k === to); r.classList.toggle("after", to === others.length && k === others.length - 1); });
    };
    let moved = false;
    const up = (ev: PointerEvent): void => {
      grip.removeEventListener("pointermove", move); grip.removeEventListener("pointerup", up); grip.removeEventListener("pointercancel", up);
      // Cut 23 §3: a tap on the grip (no drag) opens the row's why-not
      if (!moved && ev.type === "pointerup" && bind.rowWhy?.()[from]) { me.classList.remove("dragging"); me.style.transform = ""; openWhy(from, me); return; }
      me.classList.remove("dragging"); me.style.transform = "";
      rowEls.forEach((r) => r.classList.remove("before", "after"));
      if (to !== from) { const rs = rows(); const [r] = rs.splice(from, 1); rs.splice(to, 0, r); hl = to; hlUntil = performance.now() + 1600; commit(); }
    };
    grip.addEventListener("pointermove", move); grip.addEventListener("pointerup", up); grip.addEventListener("pointercancel", up);
  }

  refresh();
  return { el, refresh, paintShadow };
}

/** QA e75ec29: the earlier row that pre-empts row `j` by its form alone — an attack row (`attack nearest`, or the same target) whose
 *  conditions all hold whenever row j's do and that needs a foe present (`foes ≥ N`, `foe: X`, `adjacent ≥ N`); null when none. */
export function formShadow(rows: Row[], j: number): number | null {
  const rj = rows[j]; if (!rj || rj.verb.v === "tactic") return null;
  const presence = (k: string): boolean => k === "foes>=" || k === "foe_tag" || k === "adj>=";
  const implied = (c: Row["conds"][number], by: Row["conds"]): boolean => by.some((d) => {
    if (c.k === "foes>=" && (c.n ?? 1) <= 1 && (d.k === "foe_tag" || d.k === "adj>=")) return true;
    if (d.k !== c.k || (d.t ?? "") !== (c.t ?? "")) return false;
    if (c.n === undefined || d.n === undefined) return c.n === d.n;
    return c.k === "hp<" || c.k === "foe_hp<" ? d.n <= c.n : c.k === "hp>" || c.k === "foes>=" || c.k === "adj>=" ? d.n >= c.n : d.n === c.n;
  });
  for (let i = 0; i < j; i++) {
    const ri = rows[i];
    if (ri.verb.v !== "attack" || !ri.conds.length || !ri.conds.some((c) => presence(c.k))) continue;
    const a = ri.verb.a ?? "nearest";
    const tagged = rj.conds.find((c) => c.k === "foe_tag")?.t;
    if (a !== "nearest" && !(tagged && (a === tagged || a === `tag:${tagged}`))) continue;
    if (ri.conds.every((c) => implied(c, rj.conds))) return i;
  }
  return null;
}
