// The trace as a table (hero actions, t = tick) with the row accounting of the last action under it
// (Cut 6 §3: `R1 none held · R2 no path`, engine data). Cut 9 §5: shared by the death screen, the exit sheet's
// `trace` chip and the report's exit lines — every exit has one now, not only a death.
// Cut 11 §2–3: when the wire carries a `because`, the accounting is the chain (ui/chain.ts: `R1 drink heal · no item ←
// den took the heal, D3 [watch]`); exit traces show their last 10 turns, the death screen keeps 5 above its chain.
import type { Trace } from "../engine/types";
import { chainOf, rowRef, type ChainCtx } from "./chain";
import { h } from "./dom";
import { openSheet } from "./sheet";
import { verbLabel } from "./tokens";

export const TRACE_ROWS = 5;
export const EXIT_TRACE_ROWS = 10;

/** The table and, under it, the chain (Cut 11) or the dim `R1 why · R2 why` line when the trace carries no `because`. */
export function traceTable(trace: Trace, ctx: ChainCtx = {}, rows = TRACE_ROWS): HTMLElement[] {
  const turns = trace.turns.slice(-rows);
  // QA 23ed91f (K: "the `tele` column is empty on every row"): the column only when a shown turn has a telegraph
  const tele = turns.some((t) => t.telegraphs.length > 0);
  const table = h("table", { class: `trace num${ctx.home ? " home" : ""}` },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "R", "hp", "foes", ...(tele ? ["tele"] : [])].map((s) => h("th", null, s)))),
    h("tbody", null, ...turns.map((t) => h("tr", null,
      h("td", null, `${t.t}`),
      // QA 912e135: a player row's turn names the row as a button when the screen can open it (`ChainCtx.onRow`)
      h("td", { class: "r" }, ...(t.row >= 0 && ctx.onRow ? [rowRef(t.row, verbLabel(t.verb), ctx.onRow)]
        : [t.row >= 0 ? `R${t.row + 1}` : t.row === -1 ? /* copy:label */ "trait" : "·", " ", h("small", { class: "dim" }, verbLabel(t.verb))])),
      h("td", null, `${t.hp}`),
      h("td", null, `${t.foes}`),
      tele ? h("td", { class: "tele" }, t.telegraphs.join(" · ")) : "",
    ))));
  const chain = chainOf(trace, { window: rows, ...ctx });
  if (chain) return [table, chain];
  const lastRows = turns[turns.length - 1]?.rows ?? [];
  // each `R2 foes appeared after` whole on its line (the list wraps between reasons, never inside one)
  const rowsLine = lastRows.length ? h("div", { class: "rows-line num dim" }, ...lastRows.flatMap((r, i) => [i ? " · " : "", ctx.onRow ? h("span", { class: "rw" }, rowRef(r.row, undefined, ctx.onRow), ` ${r.why}`) : h("span", { class: "rw" }, `R${r.row + 1} ${r.why}`)])) : null;
  return rowsLine ? [table, rowsLine] : [table];
}
/** Cut 9 §5: a `trace` chip; tapping it opens the table (Cut 11 §3: the last 10 turns and the chain) in a sheet.
 *  `null` when the exit carries no trace. `head` is the sheet's header line under the label — the exit's ledger line, engine
 *  data verbatim (QA on 50bb162: "TRACE sheet from the ledger: no header"). */
export function traceChip(trace: Trace | undefined, cls = "chip mini", ctx: ChainCtx = {}, head?: string, label?: string): HTMLElement | null {
  if (!trace?.turns.length) return null;
  // Cut 14 §4: `label` names the chip's exit (`D5 · died · trace`, ui/report.ts); the plain chip stays `trace`
  const home = !!head && /^(banked|returned)\b/.test(head);   // the exit line's first word is the tier (engine data)
  return h("button", { class: cls, onclick: () => openSheet(() => h("div", { class: "sheet-body trace-sheet" },
    h("div", { class: "label row-label" }, /* copy:label */ "trace"),
    head ? h("div", { class: "trace-head ledger-line num dim" }, head) : null,
    ...traceTable(trace, { ...ctx, provenance: true, home }, EXIT_TRACE_ROWS))) }, label ?? /* copy:button */ "trace");
}
