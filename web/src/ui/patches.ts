// Patch rows, shared by the death screen and the report's stall section: tappable `cond → verb` with the survival
// (or reach) share and the forecast delta; tapping applies the patch and opens the camp on the row.
// Cut 11 §2: a patch with `root` names the chain's root under the row (`← den took the heal`, accent); `insert_at: -1` is
// an unlock pseudo-patch (`◆2 cond: alert · buy`): the tap buys the cond's unlock, then opens the camp on the row (the
// set's own locked row, or the patch's row inserted at the top when the set lacks it); §4: a
// `below_bar` candidate renders dimmed with `survives 40% · below bar` — the named alternative of a `dice` death.
// Cut 14 §4: on a full set an insert patch reads `↑ R3` — the own row that fired least (`leastFiredRow`) goes and the patch
// lands where it was measured (`App.applyPatchOver`), so the tap never leaves the editor at `6/5 · drop one` (rater S).
import type { App } from "../app";
import type { Patch, Row, Trace } from "../engine/types";
import { h, pct } from "./dom";
import { isCardRow, rowLabel, sameCond, sameVerb } from "./tokens";

const sameRow = (a: Row, b: Row): boolean => a.conds.length === b.conds.length && a.conds.every((c, i) => sameCond(c, b.conds[i]) && c.n === b.conds[i].n) && sameVerb(a.verb, b.verb);

/** The unlock behind a pseudo-patch: the wire's `unlock` when sent, else the row's first cond that is an unlockable token
 *  (`alert>=` → `cond_alert`, `on_kill` → `cond_on_kill`; core meta.rs / fake COND_UNLOCK). */
export function unlockOf(app: App, p: Patch): string | undefined {
  if (p.unlock) return p.unlock;
  const locked = new Set((app.vocab?.locked ?? []).map((l) => l.cond.k));
  const offered = new Set((app.vocab?.conds ?? []).map((c) => c.k));
  const c = p.row.conds.find((x) => locked.has(x.k)) ?? p.row.conds.find((x) => !offered.has(x.k));
  return c ? `cond_${c.k.replace(/[<>]=?$/, "")}` : undefined;
}

/** Fractions 0..1 from the core: survive, forecast_delta. A stall patch marks its target row: `R1 ↻` replaces it, `R1 −` removes it.
 *  Cut 4 §2: a row reads `survives 100% · base 75%` (death: `baseline` is the unpatched survival) or `reach 40% · base 35%`
 *  (stall: `survive` is the patched reach, base = survive − delta), then `reach +5%`, or `reach ~0` when the delta rounds to 0
 *  (QA on 50bb162: "reach missing on some patches"; the unlock cards say it that way). */
/** Cut 14 §4: the row a patch replaces on a full set — the own row (never a card's) that fired least: `app.rowFires` (the
 *  watched run's rule events, or the absence's usage lines), else the rows the trace shows firing; among equals the lowest in
 *  the list (the one the rows above it overshadow). −1 when the set has no own row. */
export function leastFiredRow(app: App, trace?: Trace): number {
  const rows = app.rules.rows;
  const fires: number[] = app.rowFires ? [...app.rowFires] : rows.map(() => 0);
  if (!app.rowFires && trace) for (const t of trace.turns) if (t.row >= 0) fires[t.row] = (fires[t.row] ?? 0) + 1;
  let best = -1;
  rows.forEach((r, i) => { if (isCardRow(r)) return; if (best < 0 || (fires[i] ?? 0) <= (fires[best] ?? 0)) best = i; });
  return best;
}

export function patchRows(app: App, patches: Patch[], baseline?: number, trace?: Trace): HTMLElement {
  return h("div", { class: "patches" }, ...patches.map((p) => {
    const delta = Math.round(p.forecast_delta * 100);
    const unlock = p.insert_at < 0;
    // A row the set already holds (an old death opened from the chronicle, a patch tapped twice) is not inserted again:
    // the row reads `at R2` and the tap opens the camp on it (QA: "tapped patch → R1 inserted AGAIN → 5/4")
    const held = unlock || p.remove || p.replace ? -1 : app.rules.rows.findIndex((r) => sameRow(r, p.row));
    // Cut 14 §4: an insert onto a full set replaces the least-fired own row, and says which (`↑ R3`)
    const drop = !unlock && !p.remove && !p.replace && held < 0 && app.rowsFull ? leastFiredRow(app, trace) : -1;
    const target = p.remove || p.replace ? h("small", { class: "dim target" }, `R${p.insert_at + 1} ${p.remove ? "−" : "↻"} `)
      : drop >= 0 ? h("small", { class: "dim target" }, /* copy:callout */ `↑ R${drop + 1} `) : "";
    const line = held >= 0
      ? /* copy:callout */ `at R${held + 1}`
      : p.below_bar
      ? /* copy:callout */ `survives ${pct(p.survive)} · below bar`
      : baseline === undefined
        ? /* copy:callout */ `reach ${pct(p.survive)} · base ${pct(Math.max(0, p.survive - p.forecast_delta))}`
        : /* copy:callout */ `survives ${pct(p.survive)} · base ${pct(baseline)}`;
    const onclick = unlock
      ? async (): Promise<void> => {
          const id = unlockOf(app, p);
          if (!id || !(await app.buy(id))) return;   // refused (marks, gate): the row would carry a locked cond
          // the core's pseudo-patch names the set's own locked row (nothing to insert: the camp opens on it); a row the
          // set lacks is inserted at the top
          const have = app.rules.rows.findIndex((r) => sameRow(r, p.row));
          const i = have >= 0 ? have : app.insertRow(p.row, 0, "patch");
          app.go({ kind: "camp", highlight: i });
        }
      : held >= 0
        ? (): void => app.go({ kind: "camp", highlight: held })
        : drop >= 0
          ? (): void => { const i = app.applyPatchOver(p, drop); app.go({ kind: "camp", highlight: i }); }
          : (): void => { const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i }); };
    const label = h("span", { class: "chips-inline" }, target, unlock ? h("span", { class: "unlock-label" }, p.root?.text ?? rowLabel(p.row), " · ", h("b", null, /* copy:button */ "buy")) : rowLabel(p.row));
    // an unlock's second line is the row it inserts once bought; a root patch's is the chain's root it answers
    const root = unlock ? (p.root ? h("small", { class: "dim" }, rowLabel(p.row)) : "") : p.root ? h("small", { class: "root" }, "← ", p.root.text) : "";
    return h("button", { class: `patch${p.remove ? " remove" : ""}${p.below_bar || held >= 0 ? " below" : ""}${unlock ? " unlock" : ""}`, onclick, ...(drop >= 0 ? { "data-drop": String(drop) } : {}) },
      h("span", { class: "patch-main" }, label, root),
      h("span", { class: "patch-nums" },
        h("span", { class: "num surv" }, line),
        delta ? h("span", { class: `num delta ${delta > 0 ? "up" : "down"}` }, /* copy:callout */ `reach ${delta > 0 ? "+" : "−"}${Math.abs(delta)}%`) : h("span", { class: "num delta flat" }, /* copy:callout */ "reach ~0")));
  }));
}
