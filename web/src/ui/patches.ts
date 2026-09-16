// Patch rows, shared by the death screen and the report's stall section: tappable `cond → verb` with the survival
// (or reach) share and the forecast delta; tapping applies the patch and opens the camp on the row.
import type { App } from "../app";
import type { Patch } from "../engine/types";
import { h, pct } from "./dom";
import { rowLabel } from "./tokens";

/** Fractions 0..1 from the core: survive, forecast_delta. A stall patch marks its target row: `R1 ↻` replaces it, `R1 −` removes it.
 *  Cut 4 §2: a row reads `survives 100% · base 75%` (death: `baseline` is the unpatched survival) or `reach 40% · base 35%`
 *  (stall: `survive` is the patched reach, base = survive − delta), then `reach +5%` only when the delta is not 0. */
export function patchRows(app: App, patches: Patch[], baseline?: number): HTMLElement {
  return h("div", { class: "patches" }, ...patches.map((p) => {
    const delta = Math.round(p.forecast_delta * 100);
    const target = p.remove || p.replace ? h("small", { class: "dim target" }, `R${p.insert_at + 1} ${p.remove ? "−" : "↻"} `) : "";
    const line = baseline === undefined
      ? /* copy:callout */ `reach ${pct(p.survive)} · base ${pct(Math.max(0, p.survive - p.forecast_delta))}`
      : /* copy:callout */ `survives ${pct(p.survive)} · base ${pct(baseline)}`;
    return h("button", { class: `patch${p.remove ? " remove" : ""}`, onclick: () => { const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i }); } },
      h("span", { class: "chips-inline" }, target, rowLabel(p.row)),
      h("span", { class: "patch-nums" },
        h("span", { class: "num surv" }, line),
        delta ? h("span", { class: `num delta ${delta > 0 ? "up" : "down"}` }, /* copy:callout */ `reach ${delta > 0 ? "+" : "−"}${Math.abs(delta)}%`) : ""));
  }));
}
