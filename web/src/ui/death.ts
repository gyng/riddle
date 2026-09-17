// Death: cause line · ledger line (Cut 6 §1) · last-5 trace (hero actions, t = tick) · row accounting of the last action
// (Cut 6 §3: `R1 none held · R2 no path`, engine data) · candidate patches (tap to insert) · edit · morgue.
// Cut 9 §7: also reached from the chronicle sheet for a kept death (`engine.death(id)`); `edit` leads back to the camp.
// Cut 11 §2: under the trace the row accounting is the chain (ui/chain.ts) — each `because` links the replay when this
// session watched the run; root patches and unlock pseudo-patches (ui/patches.ts). §5: the ledger line opens the gold
// sheet filtered to this run.
import type { App, Mounted } from "../app";
import type { Death } from "../engine/types";
import { morgueVerbs } from "./chain";
import { h, copyText, items } from "./dom";
import { openGoldSheet } from "./gold";
import { patchRows } from "./patches";
import { openSheet } from "./sheet";
import { traceTable } from "./trace";

/** Cut 10 §3: the core's `3 over` margin reads `3 hp short` wherever it is displayed (`N hp short` and others pass through). */
export const marginText = (m: string): string => m.replace(/^(\d+) over$/, /* copy:callout */ "$1 hp short");

export function renderDeath(app: App, d: Death, lost: string[] = []): Mounted {
  const line = h("h1", { class: "death-line" }, /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth} · ${marginText(d.margin)} · `, h("span", { class: /* copy:none */ `verdict ${d.verdict}` }, d.verdict));
  // Cut 6 §1: the exit's arithmetic, verbatim from the engine (`$144 carried · death keeps 0% → $0 · bones: 7 items on D5`)
  // Cut 11 §5: tappable — the gold sheet filtered to this run's movements
  const ledger = d.line?.text ? h("div", { class: "ledger-line num dim" }, h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, d.line) }, d.line.text)) : null;
  // The trace holds one row per hero action (~10 ticks apart at base speed); the last five, with the row accounting of
  // the last action under it (Cut 6 §3). Cut 9 §5: the table lives in ui/trace.ts, shared with every exit.
  // Cut 11 §2: the accounting is the chain; the rules that ran label its rows (the morgue's, else the editing copy)
  const trace = traceTable(d.trace, { rows: app.rules.rows, verbs: morgueVerbs(d.morgue), runId: d.run_id, chain: d.chain });
  // Fractions 0..1 from the core: baseline (survival of the unpatched rules) is on every row (Cut 4 §2).
  const patches = patchRows(app, d.patches, d.baseline ?? 0);
  // The morgue is the shareable text of the run: show it in a sheet (the clipboard is a bonus, not the point).
  const morgue = h("button", { class: "btn", onclick: () => {
    void copyText(d.morgue);
    openSheet(() => h("div", { class: "morgue" }, h("pre", { class: "morgue-text" }, d.morgue)));
  } }, /* copy:button */ "morgue");
  const edit = h("button", { class: "btn primary", onclick: () => app.go({ kind: "camp" }) }, /* copy:button */ "edit");
  const eggs = lost.length ? h("div", { class: "chips eggs" }, ...lost.map((k) => h("span", { class: "chip egg" }, "◯ ", k.replace(/_/g, " ").replace(" · ", " "), /* copy:callout */ " fell"))) : null;   // Cut 10 §3
  // Cut 2 §2: what this death left on the floor — the pile whose heir the matching grave names; silent when absent
  const L = app.lineage;
  const grave = [...(L.graveyard ?? [])].reverse().find((g) => g.depth === d.depth && g.cause === d.cause);
  const pile = (L.bones ?? []).find((b) => (grave ? b.heir === grave.heir : false) && b.depth === d.depth);
  const bones = pile ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(pile.items)}`) : null;
  const el = h("main", { class: "death" }, line, ledger, eggs, bones, ...trace, patches, h("div", { class: "btn-row" }, morgue, edit));
  return { el };
}
