// Death: cause line · ledger line (Cut 6 §1) · last-5 trace (hero actions, t = tick) · row accounting of the last action
// (Cut 6 §3: `R1 none held · R2 no path`, engine data) · candidate patches (tap to insert) · edit · morgue.
// Cut 9 §7: also reached from the chronicle sheet for a kept death (`engine.death(id)`, `Screen.kept`); `edit` leads back to
// the camp, and so does Escape with no sheet open (app.ts; QA on 952e306: "old death screen, no back/close, Escape inert").
// Cut 11 §2: under the trace the row accounting is the chain (ui/chain.ts) — each `because` links the replay when this
// session watched the run; root patches and unlock pseudo-patches (ui/patches.ts). §5: the ledger line opens the gold
// sheet filtered to this run.
// Cut 13 §1: a stalled run's verdict (`verdict: "stall"`) mounts here too — the guard's moment as the headline, the trace and
// the patches like a death's. §4: the run's last two notes (`Death.notes`, engine data verbatim) sit under the headline.
// §5: a `dice` death names what the forecast said for that depth — the camp's own reach line, verbatim (`forecast said D4 100%`)
// when the last forecast knows the floor (QA on 50bb162: "`forecast said 36%` while the camp forecast read `D4 100% ±1`").
import type { App, Mounted } from "../app";
import type { Death } from "../engine/types";
import { morgueVerbs } from "./chain";
import { h, copyText, items, pct } from "./dom";
import { openGoldSheet } from "./gold";
import { patchRows } from "./patches";
import { openSheet } from "./sheet";
import { gem, portrait, renderBar, renderConsole, tile } from "./frame";
import { lostLabel } from "./tokens";
import { traceTable } from "./trace";

/** Cut 10 §3: the core's `3 over` margin reads `3 hp short` wherever it is displayed (`N hp short` and others pass through). */
export const marginText = (m: string): string => m.replace(/^(\d+) over$/, /* copy:callout */ "$1 hp short");

/** The headline's margin segment: a stall's is the guard's reason (`no path`); an hp margin (`3 over` / `3 hp short`) is left
 *  out — four QA players read `1 hp short` as the hp left (the morgue still carries it); an empty margin is no segment. */
export const headlineMargin = (m: string): string => /^\d+ (over|hp short)$/.test(m) ? "" : marginText(m);

export function renderDeath(app: App, d: Death, lost: string[] = [], kept = false): Mounted {
  // a stall's margin is the guard's reason (or empty): the headline never carries an empty segment
  const seg = headlineMargin(d.margin ?? "");
  const margin = seg ? ` · ${seg}` : "";
  // Cut 17 §4: the line is laid on the defeat banner — the cause and depth in the display face, the verdict in the seal under it
  // (one word, engine data: `gap` · `dice` · `stall`); the text reads as before (`goblin archer · D6 · gap`)
  const line = h("h1", { class: "death-line" }, h("span", { class: "cause" }, /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth}${margin}`), h("span", { class: "sep" }, " · "), h("span", { class: /* copy:none */ `verdict ${d.verdict}` }, d.verdict));
  // Cut 13 §4: the run's last two notes, engine data verbatim (`The green one: fire. Gambled: fire potion.`)
  const notes = d.notes?.length ? h("div", { class: "death-notes num dim" }, ...d.notes.slice(-2).map((n) => h("div", { class: "note" }, n))) : null;
  // Cut 13 §5: a `dice` death says what the forecast said for that depth — the reach the camp showed for the floor, verbatim
  // an old death (the chronicle) was sent under another forecast: today's would be a false number (QA on 56f2a1d: `forecast said D7 0%`)
  const said = d.verdict === "dice" && !kept ? forecastSaid(app, d.depth) : undefined;
  const forecastLine = said !== undefined ? h("div", { class: "forecast-said num dim" }, /* copy:callout */ `forecast said D${d.depth} ${pct(said)}`) : null;
  // Cut 6 §1: the exit's arithmetic, verbatim from the engine (`$144 carried · death keeps 0% → $0 · bones: 7 items on D5`)
  // Cut 11 §5: tappable — the gold sheet filtered to this run's movements
  const ledger = d.line?.text ? h("div", { class: "ledger-line num dim" }, h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, d.line) }, d.line.text)) : null;
  // The trace holds one row per hero action (~10 ticks apart at base speed); the last five, with the row accounting of
  // the last action under it (Cut 6 §3). Cut 9 §5: the table lives in ui/trace.ts, shared with every exit.
  // Cut 11 §2: the accounting is the chain; the rules that ran label its rows (the morgue's, else the editing copy)
  // The run's own rules label the accounting (`R1 drink unknown · no use`, as the editor spells it); a death from before
  // the wire carried them falls back to the morgue's short forms
  const trace = traceTable(d.trace, { rows: d.rules?.rows ?? app.rules.rows, verbs: d.rules ? undefined : morgueVerbs(d.morgue), runId: d.run_id, chain: d.chain });
  // Fractions 0..1 from the core: baseline (survival of the unpatched rules) is on every row (Cut 4 §2).
  const patches = patchRows(app, d.patches, d.baseline ?? 0, d.trace);   // Cut 14 §4: the trace names the least-fired row on a full set
  // The morgue is the shareable text of the run: show it in a sheet (the clipboard is a bonus, not the point).
  const openMorgue = (): void => {
    void copyText(d.morgue);
    openSheet(() => h("div", { class: "morgue" }, h("div", { class: "label row-label" }, /* copy:label */ "morgue"), h("pre", { class: "morgue-text" }, d.morgue)));
  };
  // Cut 10 §3: `◯ jackal Ashar fell` (a companion leaves an egg); Cut 12 §6: a summoned ally reads `ally hound fell`, no egg
  const eggs = lost.length ? h("div", { class: "chips eggs" }, ...lost.map((k) => h("span", { class: "chip egg" }, k.includes(" · ") ? "◯ " : "", lostLabel(k)))) : null;
  // Cut 2 §2: what this death left on the floor — the pile whose heir the matching grave names; silent when absent, and silent
  // when the exit line already says it (`… · bones: 8 items on D4`; two QA players on 50bb162 read the pair as two piles)
  const L = app.lineage;
  const grave = [...(L.graveyard ?? [])].reverse().find((g) => g.depth === d.depth && g.cause === d.cause);
  const pile = (L.bones ?? []).find((b) => (grave ? b.heir === grave.heir : false) && b.depth === d.depth);
  const bones = pile && !/\bbones:/.test(d.line?.text ?? "") ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(pile.items)}`) : null;
  // Cut 17 §4: the frame — the banner over the dimmed floor (the line, the seal), the trace on a parchment panel, the patches as
  // tablets with a gauge; the console: edit · morgue · camp, and the gem applies the top patch (its survival on the stone)
  const top = topPatch(patches);
  const bar = renderBar(app);
  const face = portrait(app, { hp: 0, dead: true, label: `D${d.depth}` });
  const gemBtn = top
    ? gem({ label: top.label, cls: "patch-gem", pulse: true, onclick: () => top.btn.click() })
    : gem({ label: /* copy:button */ "edit", pulse: true, onclick: () => app.go({ kind: "camp" }) });
  const cons = renderConsole({ portrait: face.el, gem: gemBtn, tiles: [
    top ? tile({ id: "edit", label: /* copy:button */ "edit", icon: "edit", onclick: () => { app.editing = true; app.go({ kind: "camp" }); } }) : null,
    tile({ id: "morgue", label: /* copy:button */ "morgue", icon: "morgue", onclick: openMorgue }),
    tile({ id: "camp", label: /* copy:button */ "camp", icon: "camp", onclick: () => app.go({ kind: "camp" }) }),
  ] });
  const well = h("div", { class: "well death-well" },
    h("div", { class: "defeat" }, h("div", { class: "banner-cloth" }, line), notes, forecastLine, ledger, eggs, bones),
    h("div", { class: "parchment trace-panel" }, ...trace),
    patches);
  const el = h("main", { class: "death frame" }, bar.el, well, cons.el);
  return { el, dispose: () => bar.dispose() };
}

/** Cut 17 §4: the gem's patch — the first that applies (not a held row's `at R2`, not a below-bar alternative): its button and its
 *  number (`92%`, the survival the patch reads; an unlock pseudo-patch's `buy`). The tablet it stands for is lit. */
function topPatch(patches: HTMLElement): { btn: HTMLButtonElement; label: string } | null {
  const btn = patches.querySelector<HTMLButtonElement>("button.patch:not(.below)");
  if (!btn) return null;
  btn.classList.add("top");
  const unlock = btn.classList.contains("unlock");
  const surv = /(\d+%)/.exec(btn.querySelector(".surv")?.textContent ?? "")?.[1];
  return { btn, label: unlock ? /* copy:button */ "buy" : surv ?? "" };
}


/** Cut 13 §5: the last forecast's reach at `depth` (a 0..1 fraction, the number the camp's bar showed); undefined without a
 *  forecast, or when the floor is past what it knew (`known_to`). */
export function forecastSaid(app: App, depth: number): number | undefined {
  const f = app.lastForecast; if (!f || depth > f.known_to) return undefined;
  return f.depths.find((x) => x.depth === depth)?.reach;
}
