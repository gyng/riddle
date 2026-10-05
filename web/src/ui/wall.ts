// Cut 29: background wall search returns an exact rule set and starting floor.
// The report paints first; the offer explains its target, odds and ordered rules.
import type { App } from "../app";
import type { WallEdit } from "../engine/types";
import { h } from "./dom";
import { rowChips } from "./editor";

/** The day's offer: the lineage's cached one at once, else ask the core (in the background) — null when there is none. */
export async function wallOffer(app: App): Promise<WallEdit | null> {
  if (app.lineage.wall) return app.lineage.wall;
  if (!app.engine.wallEdit) return null;
  try { return (await app.engine.wallEdit()) ?? null; } catch (e) { console.warn("wallEdit", e); return null; }
}
const pct = (x: number): string => `${Math.round(x * 100)}%`;
/** Review the measured rules before applying the exact Rust offer. */
export function wallTablet(app: App, w: WallEdit, onApply: () => void): HTMLElement {
  const review = h("details", { class: "wall-rules" },
    h("summary", null, /* copy:button */ "Review rules"),
    h("div", { class: "wall-compare" },
      h("section", { class: "wall-current" }, h("div", { class: "label" }, /* copy:label */ "Current rules"), ...app.rules.rows.map((r, i) => h("div", { class: "wall-rule" }, h("small", { class: "num dim" }, i + 1), rowChips(r)))),
      h("section", { class: "wall-suggested" }, h("div", { class: "label" }, /* copy:label */ "Suggested rules"), ...w.rules.rows.map((r, i) => h("div", { class: "wall-rule" }, h("small", { class: "num dim" }, i + 1), rowChips(r))))));
  return h("div", { class: "patch tablet wall-edit", "data-depth": w.depth },
    h("div", { class: "wall-head num" }, h("b", null, /* copy:label */ `Reach D${w.depth + 1}`), h("small", { class: "dim" }, /* copy:callout */ ` · ${w.sims} samples`)),
    w.start && w.start !== app.lineage.start ? h("div", { class: "wall-start num" }, /* copy:label */ "Start floor", /* copy:callout */ ` · D${app.lineage.start ?? 1} → D${w.start}`) : null,
    h("div", { class: "wall-foot num" }, h("span", { class: "dim" }, /* copy:label */ "Before", ` ${pct(w.before)}`), " → ", h("b", { class: "up" }, /* copy:label */ "With fix", ` ${pct(w.after)}`)),
    review,
    h("button", { class: "chip wall-apply", onclick: async () => { if (w.start && w.start !== app.lineage.start && app.engine.setStart) await app.mutate(() => app.engine.setStart!(w.start!)); app.applyRules(w.rules); onApply(); } }, /* copy:button */ "Apply fix"));
}
