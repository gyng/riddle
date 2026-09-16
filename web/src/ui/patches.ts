// Patch rows, shared by the death screen and the report's stall section: tappable `cond → verb` with the survival
// (or reach) share and the forecast delta; tapping applies the patch and opens the camp on the row.
import type { App } from "../app";
import type { Patch } from "../engine/types";
import { h, pct } from "./dom";
import { rowLabel } from "./tokens";

/** Fractions 0..1 from the core: survive, forecast_delta. A stall patch marks its target row: `R1 ↻` replaces it, `R1 −` removes it. */
export function patchRows(app: App, patches: Patch[]): HTMLElement {
  return h("div", { class: "patches" }, ...patches.map((p) => {
    const delta = Math.round(p.forecast_delta * 100);
    const target = p.remove || p.replace ? h("small", { class: "dim target" }, `R${p.insert_at + 1} ${p.remove ? "−" : "↻"} `) : "";
    return h("button", { class: `patch${p.remove ? " remove" : ""}`, onclick: () => { const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i }); } },
      h("span", { class: "chips-inline" }, target, rowLabel(p.row)),
      h("span", { class: "patch-nums" },
        h("span", { class: "num surv" }, pct(p.survive)),
        h("span", { class: `num delta ${delta >= 0 ? "up" : "down"}` }, `${delta >= 0 ? "+" : "−"}${Math.abs(delta)}%`)));
  }));
}
