// The last-5 trace as a table (hero actions, t = tick) with the row accounting of the last action under it
// (Cut 6 §3: `R1 none held · R2 no path`, engine data). Cut 9 §5: shared by the death screen, the exit sheet's
// `trace` chip and the report's exit lines — every exit has one now, not only a death.
import type { Trace } from "../engine/types";
import { h } from "./dom";
import { openSheet } from "./sheet";
import { verbLabel } from "./tokens";

export const TRACE_ROWS = 5;

/** The table and, when the last turn carries row accounting, the dim `R1 why · R2 why` line. */
export function traceTable(trace: Trace): HTMLElement[] {
  const turns = trace.turns.slice(-TRACE_ROWS);
  const table = h("table", { class: "trace num" },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "R", "hp", "foes", "tele"].map((s) => h("th", null, s)))),
    h("tbody", null, ...turns.map((t) => h("tr", null,
      h("td", null, `${t.t}`),
      h("td", { class: "r" }, t.row >= 0 ? `R${t.row + 1}` : t.row === -1 ? /* copy:label */ "trait" : "·", " ", h("small", { class: "dim" }, verbLabel(t.verb))),
      h("td", null, `${t.hp}`),
      h("td", null, `${t.foes}`),
      h("td", { class: "tele" }, t.telegraphs.join(" · ")),
    ))));
  const lastRows = turns[turns.length - 1]?.rows ?? [];
  const rowsLine = lastRows.length ? h("div", { class: "rows-line num dim" }, lastRows.map((r) => `R${r.row + 1} ${r.why}`).join(" · ")) : null;
  return rowsLine ? [table, rowsLine] : [table];
}
/** Cut 9 §5: a `trace` chip; tapping it opens the table in a sheet. `null` when the exit carries no trace. */
export function traceChip(trace: Trace | undefined, cls = "chip mini"): HTMLElement | null {
  if (!trace?.turns.length) return null;
  return h("button", { class: cls, onclick: () => openSheet(() => h("div", { class: "sheet-body trace-sheet" }, ...traceTable(trace))) }, /* copy:button */ "trace");
}
