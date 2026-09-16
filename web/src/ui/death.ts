// Death: cause line · last-5 trace (hero actions, t = tick) · candidate patches (tap to insert) · edit · morgue.
import type { App, Mounted } from "../app";
import type { Death } from "../engine/types";
import { h, copyText, items, replace } from "./dom";
import { patchRows } from "./patches";
import { verbLabel } from "./tokens";

const TRACE_ROWS = 5;

export function renderDeath(app: App, d: Death, lost: string[] = []): Mounted {
  const line = h("h1", { class: "death-line" }, /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth} · ${d.margin} · `, h("span", { class: /* copy:none */ `verdict ${d.verdict}` }, d.verdict));
  // The trace holds one row per hero action (~10 ticks apart at base speed); show the last five.
  const turns = d.trace.turns.slice(-TRACE_ROWS);
  const table = h("table", { class: "trace num" },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "R", "hp", "foes", "tele"].map((s) => h("th", null, s)))),
    h("tbody", null, ...turns.map((t) => h("tr", null,
      h("td", null, `${t.t}`),
      h("td", { class: "r" }, t.row >= 0 ? `R${t.row + 1}` : t.row === -1 ? /* copy:label */ "trait" : "·", " ", h("small", { class: "dim" }, verbLabel(t.verb))),
      h("td", null, `${t.hp}`),
      h("td", null, `${t.foes}`),
      h("td", { class: "tele" }, t.telegraphs.join(" · ")),
    ))));
  // Fractions 0..1 from the core: baseline (survival of the unpatched rules) is on every row (Cut 4 §2).
  const patches = patchRows(app, d.patches, d.baseline ?? 0);
  const morgue = h("button", { class: "btn", onclick: async () => { const ok = await copyText(d.morgue); replace(morgue, ok ? "✓" : "×"); setTimeout(() => replace(morgue, /* copy:button */ "morgue"), 900); } }, /* copy:button */ "morgue");
  const edit = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "edit");
  const eggs = lost.length ? h("div", { class: "chips eggs" }, ...lost.map((k) => h("span", { class: "chip egg" }, "◯ ", k.replace(/_/g, " ")))) : null;
  // Cut 2 §2: what this death left on the floor — the pile whose heir the matching grave names; silent when absent
  const L = app.lineage;
  const grave = [...(L.graveyard ?? [])].reverse().find((g) => g.depth === d.depth && g.cause === d.cause);
  const pile = (L.bones ?? []).find((b) => (grave ? b.heir === grave.heir : false) && b.depth === d.depth);
  const bones = pile ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(pile.items)}`) : null;
  const el = h("main", { class: "death" }, line, eggs, bones, table, patches, h("div", { class: "btn-row" }, morgue, edit));
  return { el };
}
