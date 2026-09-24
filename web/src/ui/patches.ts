// Patch rows, shared by the death screen and the report's stall section: tappable `cond → verb` with the survival
// (or reach) share and the forecast delta; tapping applies the patch and opens the camp on the row.
// Cut 11 §2: a patch with `root` names the chain's root under the row (`← den took the heal`, accent); `insert_at: -1` is
// an unlock pseudo-patch (`◆2 cond: alert · buy`): the tap buys the cond's unlock, then opens the camp on the row (the
// set's own locked row, or the patch's row inserted at the top when the set lacks it); §4: a
// `below_bar` candidate renders dimmed with `survives 40% · below bar` — the named alternative of a `dice` death.
// Cut 14 §4's silent `↑ R3` is withdrawn (Cut 15 §3: "the caster patch replaced my drink unknown row"): on a full set an insert
// patch reads `+ drop one` and the tap opens a sheet of the set's own rows (card rows never), each with its fired count when known
// (`R5 depth ≥ 8 → bank · 0/16`), the least-fired marked when it is the only one at its count; tapping a row drops it and the patch
// lands where it was measured (`App.applyPatchOver`); dismissing the sheet (`×`, Escape, the backdrop) leaves the set whole and the death screen up.
import type { App } from "../app";
import type { Patch, Row, Trace } from "../engine/types";
import { h, pct } from "./dom";
import { closeX, openSheet } from "./sheet";
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
/** Cut 14 §4 (Cut 15 §3: the row the drop sheet marks): the row a patch replaces on a full set — the own row (never a card's) that fired least: `app.rowFires` (the
 *  watched run's rule events, or the absence's usage lines), else the rows the trace shows firing; among equals the lowest in
 *  the list (the one the rows above it overshadow). −1 when the set has no own row. */
export function leastFiredRow(app: App, trace?: Trace): number {
  const rows = app.rules.rows, fires = firesOf(app, trace);
  let best = -1;
  rows.forEach((r, i) => { if (isCardRow(r)) return; if (best < 0 || (fires[i] ?? 0) <= (fires[best] ?? 0)) best = i; });
  return best;
}

/** Cut 19 §4: the own row the core says an insert on a full set drops (`Patch.drops`), −1 when absent or not an own row of the set now. */
export function dropsOf(app: App, p: Patch): number {
  const i = p.drops; if (i === undefined || i < 0) return -1;
  const r = app.rules.rows[i];
  return r && !isCardRow(r) ? i : -1;
}

/** The least-fired own row when its count is unique among the own rows, else −1 (the drop sheet's `↓`). */
function uniqueLeast(app: App, trace?: Trace): number {
  const least = leastFiredRow(app, trace); if (least < 0) return -1;
  const fires = firesOf(app, trace), n = (i: number): number => fires[i] ?? 0;
  return app.rules.rows.some((r, i) => i !== least && !isCardRow(r) && n(i) === n(least)) ? -1 : least;
}
/** Each row's fires: `app.rowFires`, else the rows the trace shows firing. */
function firesOf(app: App, trace?: Trace): number[] {
  const fires: number[] = app.rowFires ? [...app.rowFires] : app.rules.rows.map(() => 0);
  if (!app.rowFires && trace) for (const t of trace.turns) if (t.row >= 0) fires[t.row] = (fires[t.row] ?? 0) + 1;
  return fires;
}

/** QA 92eb880 (M, the worst death: `DICE` over three patches all `survives 100% · below bar`): `nothingBeatsBase` — the core's
 *  `Death.nothing_beats_base`: one line over the block says so (`nothing beats base · base 100%`) and no patch reads `below bar`.
 *  `depth`: the floor a stall patch's reach is measured on (`reach D7 17%`; N: "reach of which floor?"). */
export type PatchOpts = { nothingBeatsBase?: boolean; depth?: number };
export function patchRows(app: App, patches: Patch[], baseline?: number, trace?: Trace, opts: PatchOpts = {}): HTMLElement {
  const head = opts.nothingBeatsBase && patches.length
    ? h("div", { class: "patches-head num dim" }, /* copy:death_line */ `nothing beats base · base ${pct(baseline ?? 1)}`) : null;
  return h("div", { class: `patches${head ? " none-beats" : ""}` }, head, ...patches.map((p) => {
    const delta = Math.round(p.forecast_delta * 100);
    // QA 23ed91f (K: "`reach 92% · base 8% · reach +83%`: 92 − 8 ≠ 83, and `reach` twice"): a stall patch's base is the rounded reach
    // less the rounded delta, so the three numbers add up; its delta then drops the word (`+84%`)
    const unlock = p.insert_at < 0;
    // A row the set already holds (an old death opened from the chronicle, a patch tapped twice) is not inserted again:
    // the row reads `at R2` and the tap opens the camp on it (QA: "tapped patch → R1 inserted AGAIN → 5/4")
    const held = unlock || p.remove || p.replace ? -1 : app.rules.rows.findIndex((r) => sameRow(r, p.row));
    const stallish = baseline === undefined && held < 0 && !p.below_bar;   // the first line already says `reach`
    // Cut 15 §3: an insert onto a full set asks which own row to drop (`+ drop one`)
    const full = !unlock && !p.remove && !p.replace && held < 0 && app.rowsFull && app.rules.rows.some((r) => !isCardRow(r));
    const target = p.remove || p.replace ? h("small", { class: "dim target" }, `R${p.insert_at + 1} ${p.remove ? "−" : "↻"} `)
      // Cut 19 §4: the core names the row the insert drops (`Patch.drops`, the dead run's least-fired own row) — `+ drop R5`; the tap
      // still opens the drop sheet on it (marked), so the player may drop another
      : full ? h("small", { class: "dim target" }, dropsOf(app, p) >= 0 ? /* copy:callout */ `+ drop R${dropsOf(app, p) + 1}` : /* copy:callout */ "+ drop one", " ") : "";
    const line = held >= 0
      ? /* copy:callout */ `at R${held + 1}`
      : p.below_bar
      ? opts.nothingBeatsBase ? /* copy:callout */ `survives ${pct(p.survive)}` : /* copy:callout */ `survives ${pct(p.survive)} · below bar`
      : baseline === undefined
        ? /* copy:callout */ `reach ${opts.depth !== undefined ? `D${opts.depth} ` : ""}${pct(p.survive)} · base ${Math.max(0, Math.round(p.survive * 100) - delta)}%`
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
        : full
          ? (): void => openDropSheet(app, p, trace)
          : (): void => { const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i }); };
    const label = h("span", { class: "chips-inline" }, target, unlock ? h("span", { class: "unlock-label" }, p.root?.text ?? rowLabel(p.row), " · ", h("b", null, /* copy:button */ "buy")) : rowLabel(p.row));
    // an unlock's second line is the row it inserts once bought; a root patch's is the chain's root it answers
    const root = unlock ? (p.root ? h("small", { class: "dim" }, rowLabel(p.row)) : "") : p.root ? h("small", { class: "root" }, "← ", p.root.text) : "";
    return h("button", { class: `patch tablet${p.remove ? " remove" : ""}${p.below_bar || held >= 0 ? " below" : ""}${unlock ? " unlock" : ""}`, onclick, ...(full ? { "data-full": "1" } : {}) },
      h("span", { class: "patch-main" }, label, root),
      h("span", { class: "patch-nums" },
        // Cut 17 §4: `survives N %` as a gauge on the patch tablet (the number stays beside it)
        unlock || held >= 0 ? "" : h("span", { class: "gauge", "aria-hidden": "true" }, h("i", { style: `width:${Math.round(Math.max(0, Math.min(1, p.survive)) * 100)}%` })),
        h("span", { class: "num surv" }, line),
        reachSpan(p, stallish)));
  }));
}

/** A patch's reach line: `reach +8%`; with the camp's own measure (`deathDeltas`) the depth and its ± — `reach D6 +8% ±3`, and
 *  `reach D6 ~0` inside the ± (as an unlock card's); `reach …` while the camp's measure is pending (`camp_pending`: the verdict's
 *  12-sim estimate is not the camp's number — QA 23ed91f, K: "`reach +8%` … the shaft went D5 79% → 78%"). A stall patch's first
 *  line already says `reach`: its delta is bare (`+84%`). */
function reachSpan(p: Patch, stallish = false): HTMLElement {
  if (p.camp_pending) return h("span", { class: "num delta pending" }, /* copy:callout */ "reach …");
  const delta = Math.round(p.forecast_delta * 100);
  const pm = p.forecast_pm !== undefined ? Math.max(1, Math.round(p.forecast_pm * 100)) : undefined;
  const flat = delta === 0 || (pm !== undefined && Math.abs(delta) <= pm);
  const word = stallish ? "" : /* copy:label */ "reach ";
  const at = p.forecast_depth !== undefined ? `D${p.forecast_depth} ` : "";
  const pmTag = pm !== undefined && !flat ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${pm}`) : "";
  // QA 92eb880 (M: "`reach D7 ~0` … the camp then shows D7 12%"): a move inside the ± reads as a move, `+0%`, never as a reach of ~0
  return flat ? h("span", { class: "num delta flat" }, `${word}${at}+0%`, pm !== undefined ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${pm}`) : "")
    : h("span", { class: `num delta ${delta > 0 ? "up" : "down"}` }, `${word}${at}${delta > 0 ? "+" : "−"}${Math.abs(delta)}%`, pmTag);
}

/** QA 23ed91f: the camp's reach for a death's patches landed (`deathDeltas`, same order): each patch takes its numbers, and each
 *  tablet in `el` (patchRows' own, in order) repaints its reach line. */
export function fillReach(el: HTMLElement, patches: Patch[], filled: Patch[]): void {
  const buttons = [...el.querySelectorAll<HTMLElement>(":scope > button.patch")];
  patches.forEach((p, i) => {
    const f = filled[i]; if (!f) return;
    Object.assign(p, { forecast_delta: f.forecast_delta, forecast_depth: f.forecast_depth, forecast_pm: f.forecast_pm, camp_pending: false });
    buttons[i]?.querySelector(".delta")?.replaceWith(reachSpan(p));
  });
  el.dataset.reach = "camp";
  // QA 92eb880 (N: the lit patch read `reach D8 −4%`, applied, and every floor fell; `rest 75%` lit over `return 100%`): once the camp's
  // reach is in, the tablets re-rank by it — a gain first, then the ones inside the ± (the higher survival first), a loss last and dim
  // (`.neg`: the gem never applies it); a held row or a below-bar alternative stays under them
  const rank = (p: Patch, b: HTMLElement): number[] => {
    if (b.classList.contains("below") && !b.classList.contains("neg")) return [4, 0, 0];
    const d = Math.round(p.forecast_delta * 100), pm = p.forecast_pm !== undefined ? Math.max(1, Math.round(p.forecast_pm * 100)) : 0;
    const flat = d === 0 || Math.abs(d) <= pm;
    return [p.insert_at < 0 ? 1 : flat ? 1 : d > 0 ? 0 : 3, flat ? 0 : -d, -p.survive];
  };
  const keyed = patches.map((p, i) => ({ b: buttons[i], k: buttons[i] ? rank(p, buttons[i]) : [9, 0, 0], i })).filter((x) => x.b);
  for (const x of keyed) x.b.classList.toggle("neg", x.k[0] === 3);
  keyed.sort((a, b) => a.k[0] - b.k[0] || a.k[1] - b.k[1] || a.k[2] - b.k[2] || a.i - b.i);
  for (const x of keyed) el.appendChild(x.b);
}

/** Cut 15 §3: the drop sheet — the set's own rows (a card's row never; it sits outside `max_rows`), `R5 <row> · 0/16` with the
 *  run's fired count out of all fires when it is known (the watched run's rule events, or the absence's `R5 fired 0 of 16 runs`),
 *  the least-fired row marked (`.least`, `↓`) when its count is unique. A tap drops that row and inserts the patch at its measured
 *  place; the `×`, the backdrop or Escape closes the sheet and nothing changes. */
export function openDropSheet(app: App, p: Patch, trace?: Trace): void {
  const rows = app.rules.rows;
  const fires = app.rowFires, total = app.rowFiresOf ?? (fires ? fires.reduce((a, b) => a + b, 0) : 0);
  // QA on 3d71c33: the `↓` marks the least-fired row only when it is the only one at that count — among ties (nothing fired, two
  // rows at 0) no row is singled out (the lowest in the list was an arbitrary pick)
  // Cut 19 §4: the row the core named (`+ drop R5`) is the marked one
  const named = dropsOf(app, p);
  const least = named >= 0 ? named : uniqueLeast(app, trace);
  openSheet((close) => h("div", { class: "sheet-body drop-sheet" },
    h("div", { class: "label row-label" }, /* copy:label */ "drop", " ", h("small", { class: "dim" }, rowLabel(p.row)), closeX(close)),
    ...rows.map((r, i) => isCardRow(r) ? null : h("button", {
      class: `drop-row${i === least ? " least" : ""}`, "data-row": String(i),
      onclick: () => { close(); const at = app.applyPatchOver(p, i); app.go({ kind: "camp", highlight: at }); },
    },
    h("b", { class: "num" }, `R${i + 1}`), " ", h("span", { class: "chips-inline" }, rowLabel(r)),
    fires ? h("span", { class: "num fired dim" }, ` · ${fires[i] ?? 0}/${total}`) : "",
    i === least ? h("span", { class: "num mark" }, "↓") : ""))));
}
