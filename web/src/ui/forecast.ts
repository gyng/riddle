// Forecast panel: reach% bars per depth up to known_to, `?` beyond; top causes.
// Cut 4 §8: the row for best+1 names the top cause even at 0% (`D6 0% · goblin warlord`) when causes are known.
// Cut 6 §5: that row also names the boss's counter when the lineage knows it. Cut 6 §9: `app` repaints quietly with
// the refined (100-sim) forecast 2 s after a paint with the rules unchanged (same `onForecast` listener).
// Cut 7 §2: `yours: 3 of 5 rows` under the bars — the rows of the active set the player wrote or edited (`Row.origin`),
// repainted on every edit (the forecast itself waits for the engine).
import type { App } from "../app";
import type { Forecast } from "../engine/types";
import { h, clear, pct, replace } from "./dom";

export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const yours = h("div", { class: "fc-yours num" });
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast"), bars, yours, causes);
  // Cut 8B §4: `· 1 combo` when the set has one (engine data; the count is the client's mirror of `Lineage.combos`)
  const paintYours = (): void => {
    const n = app.playerRows(), m = app.rules.rows.length, k = app.combos().length;
    replace(yours, h("span", { class: n ? "" : "dim" }, /* copy:callout */ `yours: ${n} of ${m} row${m === 1 ? "" : "s"}`),
      k ? h("span", { class: "combos" }, /* copy:label */ ` · ${k} combo${k === 1 ? "" : "s"}`) : "");
  };
  /** The named counter of a boss cause (`goblin_warlord`, `goblin warlord pack`) from `lineage.counters`. */
  const counterFor = (cause: string): string | undefined => {
    const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
    return (app.lineage.counters ?? []).find((c) => c.boss === key || key.endsWith(c.boss))?.text;
  };
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes);
    const next = app.lineage.best_depth + 1;
    for (const d of f.depths) {
      const cause = d.cause ?? (d.depth === next ? f.causes[0]?.cause : undefined);
      // Cut 6 §5: `D6 0% · goblin warlord · counter: attack boss` when the top cause is a boss whose counter row is known
      const counter = cause && d.depth === next ? counterFor(cause) : undefined;
      bars.appendChild(h("div", { class: `bar${cause ? " next" : ""}` },
        h("span", { class: "d num" }, `D${d.depth}`),
        h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` })),
        h("span", { class: "n num" }, pct(d.reach), cause ? h("small", { class: "dim" }, ` · ${cause.replace(/_/g, " ")}`) : "",
          counter ? h("small", { class: "dim" }, /* copy:callout */ ` · counter: ${counter}`) : "")));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    for (const c of f.causes) causes.appendChild(h("span", { class: "cause" }, c.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, pct(c.share))));
  };
  // until the first forecast arrives (≈1 s in the worker): the unknown row only
  bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, "…"), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
  const off = app.onForecast(paint), offRules = app.onRules(paintYours), offChange = app.onChange(paintYours);
  paintYours();
  void app.emitForecast();
  return { el, dispose: () => { off(); offRules(); offChange(); } };
}
