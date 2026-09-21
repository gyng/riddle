// Forecast panel: reach% bars per depth up to known_to, `?` beyond; top causes.
// Cut 4 §8: the row for best+1 names the top cause even at 0% (`D6 0% · goblin warlord`) when causes are known.
// Cut 6 §5: that row also names the boss's counter when the lineage knows it. Cut 6 §9: `app` repaints quietly with
// the refined (100-sim) forecast 2 s after a paint with the rules unchanged (same `onForecast` listener).
// Cut 7 §2: `yours: 3 of 5 rows` under the bars — the rows of the active set the player wrote or edited (`Row.origin`),
// repainted on every edit (the forecast itself waits for the engine).
// Cut 9 §3: a bar reads `D4 71% ±6` when the engine sends `pm` (the binomial half-width), so a wobble reads as noise.
// Cut 9 §4 appended `· card R2 first` to the `yours` line; gone since Cut 12 §1 put a card before the engagement row on purpose
// (both QA players on 952e306: "what 'first' means for a row that sits third").
// Cut 10 §2: a boss floor whose counter is known and whose row is absent reads `D9 0% · warlord · try: attack boss` (the
// engine's `try`, or the client's read of `Lineage.counters` against the set); tapping the bar inserts the row at the top.
// Cut 12 §1: `yours: n of m rows` counts own rows (card rows sit outside `max_rows`); §3: one line under the depths says how a
// send ends when the engine sends `ends` (`bank 40% · return 35% · death 25% · ~$54`); §6: a combo is named, not counted.
// Cut 13 §5: the first paint (`Forecast.refined` false) carries `…` after each `±` so the refine's landing does not read as a
// re-roll; the ends line has its own `±` on the death share (`death 5% ±4`, `ForecastEnds.pm`).
import type { App } from "../app";
import type { Forecast, ForecastTry, Row } from "../engine/types";
import { h, clear, pct, replace } from "./dom";
import { closeAllSheets } from "./sheet";

const sameRow = (a: Row, b: Row): boolean =>
  a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && a.conds.length === b.conds.length &&
  a.conds.every((c, i) => c.k === b.conds[i].k && (c.n ?? "") === (b.conds[i].n ?? "") && (c.t ?? "") === (b.conds[i].t ?? ""));
/** Cut 10 §2: the known-but-absent counter for a boss cause, from `Lineage.counters` (a client fallback for the wire's `try`). */
export function clientTry(app: App, cause: string | undefined): ForecastTry | undefined {
  if (!cause) return undefined;
  const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
  const c = (app.lineage.counters ?? []).find((k) => k.boss === key || key.endsWith(k.boss));
  if (!c || !c.row || typeof c.row === "string") return undefined;
  return app.rules.rows.some((r) => sameRow(r, c.row as Row)) ? undefined : { row: c.row, text: c.text };
}

/** Cut 9 §3: the half-width (a 0..1 fraction like `reach`) in percentage points, never `±0` — a forecast is never exact. */
export const pmPts = (pm: number): number => Math.max(1, Math.round(pm * 100));

export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const yours = h("div", { class: "fc-yours num" });
  const ends = h("div", { class: "fc-ends num dim", hidden: true });
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast"), bars, ends, yours, causes);
  // Cut 8B §4: `· 1 combo` when the set has one (engine data; the count is the client's mirror of `Lineage.combos`)
  // Cut 12 §6: the combo's name (engine data: `Vocabulary.combos[].name`), not `1 combo`
  const paintYours = (): void => {
    const n = app.playerRows(), m = app.ownRows(), combos = app.combos();
    replace(yours, h("span", { class: n ? "" : "dim" }, /* copy:callout */ `yours: ${n} of ${m} row${m === 1 ? "" : "s"}`),
      combos.length ? h("span", { class: "combos" }, ` · ${combos.map((c) => c.name).join(" · ")}`) : "");
  };
  // Cut 12 §3: `bank 40% · return 35% · death 25% · ~$54` — the three shown always so the trade reads; absent on an older core
  const paintEnds = (f: Forecast): void => {
    const e = f.ends;
    ends.hidden = !e;
    if (!e) return;
    // a stall share only when there is one: `bank 0% · return 20% · stall 50% · death 30% · ~$25`
    const stall = e.stall && Math.round(e.stall * 100) > 0 ? /* copy:callout */ ` · stall ${pct(e.stall)}` : "";
    const pm = e.pm !== undefined ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${pmPts(e.pm)}${f.refined === false ? "…" : ""}`) : "";
    replace(ends, /* copy:callout */ `bank ${pct(e.bank)} · return ${pct(e.return)}`, stall, /* copy:callout */ ` · death ${pct(e.death)}`, pm, h("span", { class: "gold" }, ` · ~$${Math.round(e.gold)}`));
  };
  /** The named counter of a boss cause (`goblin_warlord`, `goblin warlord pack`) from `lineage.counters`. */
  const counterFor = (cause: string): string | undefined => {
    const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
    return (app.lineage.counters ?? []).find((c) => c.boss === key || key.endsWith(c.boss))?.text;
  };
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes); paintEnds(f);
    el.dataset.refined = f.refined === undefined ? "" : f.refined ? "1" : "0";   // dev: tools read which pass painted
    const first = f.refined === false ? "…" : "";   // Cut 13 §5: the first paint's ± trails `…`; the refine's does not
    const next = app.lineage.best_depth + 1;
    for (const d of f.depths) {
      const cause = d.cause ?? (d.depth === next ? f.causes[0]?.cause : undefined);
      // Cut 10 §2: the known-but-absent counter — the wire's `try`, else the client's read of the counters against the set
      const tr = d.try ?? (d.depth === next ? clientTry(app, cause) : undefined);
      // Cut 6 §5: `D6 0% · goblin warlord · counter: attack boss` when the top cause is a boss whose counter row is known (and held)
      const counter = cause && d.depth === next && !tr ? counterFor(cause) : undefined;
      // the try hint rides the track, on the depth's own line, the track as wide as every other row's (QA on 50bb162: `D9 0% ±1`
      // wrapped `· try: attack boss` under it with a shorter bar)
      const track = h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` }));
      const inner = [
        h("span", { class: "d num" }, `D${d.depth}`),
        tr ? h("span", { class: "track-cell" }, track, h("small", { class: "try" }, /* copy:none */ `try: ${tr.text}`)) : track,
        h("span", { class: "n num" }, pct(d.reach), d.pm !== undefined ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${pmPts(d.pm)}${first}`) : "",
          cause ? h("small", { class: "dim" }, ` · ${cause.replace(/_/g, " ")}`) : "",
          counter ? h("small", { class: "dim" }, /* copy:callout */ ` · counter: ${counter}`) : ""),
      ];
      // the `try` bar is a button: the row goes in at the top (position is the point), the camp opens on it
      bars.appendChild(tr
        ? h("button", { class: `bar next try`, onclick: () => { const i = app.applyPatch({ row: tr.row, insert_at: 0, survive: 0, forecast_delta: 0 }); closeAllSheets(); app.go({ kind: "camp", highlight: i }); } }, ...inner)
        : h("div", { class: `bar${cause ? " next" : ""}` }, ...inner));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    for (const c of f.causes) causes.appendChild(h("span", { class: "cause" }, c.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, pct(c.share))));
  };
  // until the first forecast arrives (≈1 s in the worker): the unknown row only
  bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, "…"), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
  // a rule edit (or a set switch) dims the numbers until the engine's next forecast paints — an empty set's takes seconds and
  // the old set's bars read as the new one's meanwhile (QA on 952e306: "set '2 0' showed set 1's D4 72%")
  const stale = (): void => { paintYours(); el.classList.add("stale"); };
  const fresh = (f: Forecast): void => { el.classList.remove("stale"); paint(f); };
  const off = app.onForecast(fresh), offRules = app.onRules(stale), offChange = app.onChange(paintYours);
  paintYours();
  // the first forecast posts after the camp's own fetches (the worker answers in order: a forecast posted first held the
  // supply shop and the unlock shelf behind it — QA B on 952e306: "while FORECAST shows '…' the shop chips and UNLOCKS are gone")
  setTimeout(() => void app.emitForecast(), 0);
  return { el, dispose: () => { off(); offRules(); offChange(); } };
}
