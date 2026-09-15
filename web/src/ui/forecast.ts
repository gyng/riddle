// Forecast panel: reach% bars per depth up to known_to, `?` beyond; top causes.
import type { App } from "../app";
import type { Forecast } from "../engine/types";
import { h, clear, pct } from "./dom";

export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast"), bars, causes);
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes);
    for (const d of f.depths) {
      bars.appendChild(h("div", { class: "bar" },
        h("span", { class: "d num" }, `D${d.depth}`),
        h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` })),
        h("span", { class: "n num" }, pct(d.reach))));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    for (const c of f.causes) causes.appendChild(h("span", { class: "cause" }, c.cause, " ", h("b", { class: "num" }, pct(c.share))));
  };
  const off = app.onForecast(paint);
  app.emitForecast();
  return { el, dispose: off };
}
