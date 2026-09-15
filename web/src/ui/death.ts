// Death: cause line · last-5 trace · candidate patches (tap to insert) · edit · morgue.
import type { App, Mounted } from "../app";
import type { Death } from "../engine/types";
import { h, copyText, replace } from "./dom";
import { rowLabel, verbLabel } from "./tokens";

export function renderDeath(app: App, d: Death): Mounted {
  const line = h("h1", { class: "death-line" }, /* copy:death_line */ `${d.cause} · D${d.depth} · ${d.margin} · `, h("span", { class: /* copy:none */ `verdict ${d.verdict}` }, d.verdict));
  const table = h("table", { class: "trace num" },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "R", "hp", "foes", "tele"].map((s) => h("th", null, s)))),
    h("tbody", null, ...d.trace.turns.map((t) => h("tr", null,
      h("td", null, `${t.t}`),
      h("td", { class: "r" }, t.row >= 0 ? `R${t.row + 1}` : t.row === -1 ? "trait" : "·", " ", h("small", { class: "dim" }, verbLabel(t.verb))),
      h("td", null, `${t.hp}`),
      h("td", null, `${t.foes}`),
      h("td", { class: "tele" }, t.telegraphs.join(" · ")),
    ))));
  const patches = h("div", { class: "patches" }, ...d.patches.map((p) => {
    const delta = Math.round(p.forecast_delta * 100);
    return h("button", { class: "patch", onclick: () => { const i = app.insertRow(p.row, p.insert_at); app.go({ kind: "camp", highlight: i }); } },
      h("span", { class: "chips-inline" }, rowLabel(p.row)),
      h("span", { class: `num delta ${delta >= 0 ? "up" : "down"}` }, `${delta >= 0 ? "+" : "−"}${Math.abs(delta)}%`));
  }));
  const morgue = h("button", { class: "btn", onclick: async () => { const ok = await copyText(d.morgue); replace(morgue, ok ? "✓" : "×"); setTimeout(() => replace(morgue, /* copy:button */ "morgue"), 900); } }, /* copy:button */ "morgue");
  const edit = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "edit");
  const el = h("main", { class: "death" }, line, table, patches, h("div", { class: "btn-row" }, morgue, edit));
  return { el };
}
