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
  const blowRow = (b: { t: number; by: string; dmg: number; hp: number }, foes?: number): HTMLElement => h("tr", { class: "blow" },
    h("td", null, `${b.t}`),
    h("td", { class: "r" }, b.by.replace(/_/g, " "), " ", h("small", { class: "dim" }, `−${b.dmg}`)),
    h("td", null, `${b.hp}`),
    h("td", null, `${foes ?? ""}`),
    tele ? h("td", { class: "tele" }) : "");
  const table = h("table", { class: `trace num${ctx.home ? " home" : ""}` },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "R", "hp", "foes", ...(tele ? ["tele"] : [])].map((s) => h("th", null, s)))),
    // QA 524827b (qaAB: `8002 R1 return 12 · 8013 R1 return 6` — hp fell with no row): the blows between two shown actions are rows of
    // their own before the second (engine data: `TraceTurn.blows`), so every hp step reads
    h("tbody", null, ...turns.flatMap((t, i) => [...(i > 0 ? (t.blows ?? []).map((b) => blowRow(b)) : []), h("tr", null,
      h("td", null, `${t.t}`),
      // QA 912e135: a player row's turn names the row as a button when the screen can open it (`ChainCtx.onRow`)
      h("td", { class: "r" }, ...(t.row >= 0 && ctx.onRow ? [rowRef(t.row, verbLabel(t.verb), ctx.onRow)]
        : [t.row >= 0 ? `R${t.row + 1}` : t.row === -1 ? /* copy:label */ "trait" : "·", " ", h("small", { class: "dim" }, verbLabel(t.verb))])),
      h("td", null, `${t.hp}`),
      h("td", null, `${t.foes}`),
      tele ? h("td", { class: "tele" }, t.telegraphs.join(" · ")) : "",
    )]),
    // QA 0c6e126 (qaY: "the last row is never the killing blow — 05 ends at `hp 1`"): a death's table ends on the blow that killed —
    // its tick, what hit and for how much, `hp 0` (engine data: `Trace.blow`)
    // Cut 25 §6 (AN: `14 → 0` on one `goblin −2` row): every blow after the last action, one row each, hp after each (`Trace.blows`,
    // the last is `blow`); an older core's lone `blow`
    ...(trace.blows?.length ? trace.blows : trace.blow ? [trace.blow] : []).map((b) => blowRow(b, turns[turns.length - 1]?.foes))));
  // QA 524827b (qaAA: the table opens at 1 hp exploring — "where 36 hp went takes a replay"): over it, where the hp went since he was
  // last at full hp (engine data: `Trace.hp_lost`, most first; the few largest, the rest counted)
  const lost = trace.hp_lost ?? [];
  // QA 308f045 (qaAC: killer `goblin conjurer −2`, the loss line `goblin −22 · jackal −15 · … · others −2` never naming him): the killing
  // blow's cause is always one of the named, in its place by size (it takes the last named slot when it would fall under `others`)
  const HP_LOST_SHOW = 4, killer = (trace.blows?.length ? trace.blows[trace.blows.length - 1] : trace.blow)?.by;
  let named = lost.slice(0, HP_LOST_SHOW);
  const k = killer ? lost.find((x) => x.by === killer) : undefined;
  if (k && !named.includes(k)) named = [...named.slice(0, HP_LOST_SHOW - 1), k];
  const rest = lost.filter((x) => !named.includes(x)).reduce((a, x) => a + x.dmg, 0);
  // QA 308f045 (qaAC: `since full hp` summing to 72 on a 36-hp hero — "the heals are part of it; nothing says so"): what was healed over
  // the stretch reads beside it (`healed +36`, the core's `Trace.hp_healed`)
  const healed = trace.hp_healed ?? 0;
  const lostLine = lost.length ? h("div", { class: "hp-lost num dim" }, /* copy:callout */ "since full hp",
    ...named.map((x) => h("span", { class: `hp-by${x.by === killer ? " killer" : ""}` }, ` · ${x.by.replace(/_/g, " ")} `, h("b", { class: "down" }, `−${x.dmg}`))),
    rest > 0 ? h("span", { class: "hp-by" }, /* copy:callout */ ` · others −${rest}`) : "",
    healed > 0 ? h("span", { class: "hp-by healed" }, /* copy:callout */ ` · healed `, h("b", { class: "up" }, `+${healed}`)) : "") : null;
  const chain = chainOf(trace, { window: rows, ...ctx });
  if (chain) return lostLine ? [lostLine, table, chain] : [table, chain];
  if (lostLine) { const r = traceTableRest(turns, ctx); return [lostLine, table, ...r]; }
  return [table, ...traceTableRest(turns, ctx)];
}
/** The last turn's row reasons as one line (an older core's chain), or nothing. */
function traceTableRest(turns: Trace["turns"], ctx: ChainCtx): HTMLElement[] {
  const lastRows = turns[turns.length - 1]?.rows ?? [];
  // each `R2 foes appeared after` whole on its line (the list wraps between reasons, never inside one)
  const rowsLine = lastRows.length ? h("div", { class: "rows-line num dim" }, ...lastRows.flatMap((r, i) => [i ? " · " : "", ctx.onRow ? h("span", { class: "rw" }, rowRef(r.row, undefined, ctx.onRow), ` ${r.why}`) : h("span", { class: "rw" }, `R${r.row + 1} ${r.why}`)])) : null;
  return rowsLine ? [rowsLine] : [];
}
/** Cut 9 §5: a `trace` chip; tapping it opens the table (Cut 11 §3: the last 10 turns and the chain) in a sheet.
 *  `null` when the exit carries no trace. `head` is the sheet's header line under the label — the exit's ledger line, engine
 *  data verbatim (QA on 50bb162: "TRACE sheet from the ledger: no header"). */
export function traceChip(trace: Trace | undefined, cls = "chip mini", ctx: ChainCtx = {}, head?: string, label?: string): HTMLElement | null {
  if (!trace?.turns.length) return null;
  // Cut 14 §4: `label` names the chip's exit (`D5 · died · trace`, ui/report.ts); the plain chip stays `trace`
  const home = ctx.home ?? (!!head && /^(banked|returned|driven)\b/.test(head));   // the exit line's first word is the tier (engine data)
  return h("button", { class: cls, onclick: () => openSheet(() => h("div", { class: "sheet-body trace-sheet" },
    h("div", { class: "label row-label" }, /* copy:label */ "trace"),
    head ? h("div", { class: "trace-head ledger-line num dim" }, head) : null,
    ...traceTable(trace, { ...ctx, provenance: true, home }, EXIT_TRACE_ROWS))) }, label ?? /* copy:button */ "trace");
}
