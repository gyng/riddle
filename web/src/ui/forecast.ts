// Forecast panel: reach% bars per depth up to known_to, `?` beyond; top causes.
// Cut 4 §8: the row for best+1 names the top cause even at 0% (`D6 0% · goblin warlord`) when causes are known.
import type { App } from "../app";
import type { Forecast } from "../engine/types";
import { h, clear, pct } from "./dom";

export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast"), bars, causes);
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes);
    const next = app.lineage.best_depth + 1;
    for (const d of f.depths) {
      const cause = d.cause ?? (d.depth === next ? f.causes[0]?.cause : undefined);
      bars.appendChild(h("div", { class: `bar${cause ? " next" : ""}` },
        h("span", { class: "d num" }, `D${d.depth}`),
        h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` })),
        h("span", { class: "n num" }, pct(d.reach), cause ? h("small", { class: "dim" }, ` · ${cause.replace(/_/g, " ")}`) : "")));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    for (const c of f.causes) causes.appendChild(h("span", { class: "cause" }, c.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, pct(c.share))));
  };
  // until the first forecast arrives (≈1 s in the worker): the unknown row only
  bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, "…"), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
  const off = app.onForecast(paint);
  void app.emitForecast();
  return { el, dispose: off };
}
