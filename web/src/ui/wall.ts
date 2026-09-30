// Cut 29 §1 (E1): at a best depth held two days, the core searches the one-row edit that passes the wall (`engine.wallEdit()`, lazy,
// on a background lane — up to a minute in wasm; `Lineage.wall` once the day's search is cached). The client never waits on it: the
// report and the camp paint first, and the offer lands as a patch tablet when it comes — the edit's words (the core's `drop R6` named
// by the rule's words), the share of sends past the record before → after, and `apply`, which takes the whole set it was measured on
// (and the start it was measured from: `start D24`, a lit waystone at the wall).
import type { App } from "../app";
import type { WallEdit } from "../engine/types";
import { h } from "./dom";

/** The day's offer: the lineage's cached one at once, else ask the core (in the background) — null when there is none. */
export async function wallOffer(app: App): Promise<WallEdit | null> {
  if (app.lineage.wall) return app.lineage.wall;
  if (!app.engine.wallEdit) return null;
  try { return (await app.engine.wallEdit()) ?? null; } catch (e) { console.warn("wallEdit", e); return null; }
}
const pct = (x: number): string => `${Math.round(x * 100)}%`;
/** The offer as a patch tablet: `wall D17` · the edits · `past D17 12→38%` · apply. */
export function wallTablet(app: App, w: WallEdit, onApply: () => void): HTMLElement {
  return h("div", { class: "patch tablet wall-edit", "data-depth": w.depth },
    h("div", { class: "wall-head num" }, h("b", null, /* copy:label */ `wall D${w.depth}`), h("small", { class: "dim" }, /* copy:callout */ ` · ${w.sims} sends`)),
    h("div", { class: "chips wall-edits" }, ...w.edits.map((e) => h("span", { class: "chip wall-e" }, e))),
    h("div", { class: "wall-foot num" }, /* copy:callout */ `past D${w.depth} `, h("span", { class: "dim" }, pct(w.before)), "→", h("b", { class: "up" }, pct(w.after)), " ",
      h("button", { class: "chip wall-apply", onclick: async () => { if (w.start && w.start !== app.lineage.start && app.engine.setStart) await app.mutate(() => app.engine.setStart!(w.start!)); app.applyRules(w.rules); onApply(); } }, /* copy:button */ "apply fix")));
}
