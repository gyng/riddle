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
/** `stall`: a stall's verdict screen — the core's `survive` is the share of replays that end the loop (the hero came home either way):
 *  `unstuck 100% · base 8%`, never `survives` (QA 1a2a4a9, P: "`survives 100%` for a hero who came home"). */
export type PatchOpts = { nothingBeatsBase?: boolean; depth?: number; stall?: boolean; select?: (btn: HTMLButtonElement) => void };
export function patchRows(app: App, patches: Patch[], baseline?: number, trace?: Trace, opts: PatchOpts = {}): HTMLElement {
  const head = opts.nothingBeatsBase && patches.length
    ? h("div", { class: "patches-head num dim" }, /* copy:death_line */ `nothing beats unpatched ${pct(baseline ?? 1)}`) : null;
  const rows = patches.map((p) => {
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
    // QA a946e04 (S: `R1 − hp < 20% · foes ≥ 1 → drink unknown` — "delete R1?"): a cut reads as one (`cut R1`); a replace keeps `R1 ↻`
    const target = p.remove || p.replace ? h("small", { class: "dim target" }, p.remove ? /* copy:callout */ `cut R${p.insert_at + 1} ` : `R${p.insert_at + 1} ↻ `)
      // Cut 19 §4: the core names the row the insert drops (`Patch.drops`, the dead run's least-fired own row) — `+ drop R5`; the tap
      // still opens the drop sheet on it (marked), so the player may drop another
      : "";
    // QA 1a2a4a9 (O: `+ drop R2 hp < 40% → drink heal` read as "put it at R2"): the drop trails the row it makes room for — `drops R2`
    const dropTag = full ? h("small", { class: "dim target drop-tag" }, " · ", dropsOf(app, p) >= 0 ? /* copy:callout */ `drops R${dropsOf(app, p) + 1}` : /* copy:callout */ "drops one") : "";
    // QA a946e04 (T: `retreat · survives 75% · base 75%` listed like a fix): a death patch that survives no more than the rules as they
    // are changes nothing — dim, `no gain`
    const noGain = baseline !== undefined && !opts.stall && !p.below_bar && held < 0 && !unlock && Math.round(p.survive * 100) <= Math.round(baseline * 100);
    const line = held >= 0
      ? /* copy:callout */ `at R${held + 1}`
      : p.below_bar
      ? opts.nothingBeatsBase ? /* copy:callout */ `survives ${pct(p.survive)}` : /* copy:callout */ `survives ${pct(p.survive)} · below bar`
      : baseline === undefined
        ? /* copy:callout */ `reach ${opts.depth !== undefined ? `D${opts.depth} ` : ""}${pct(p.survive)} · base ${Math.max(0, Math.round(p.survive * 100) - delta)}%`
        // QA a946e04 (S: `base 25%` on every patch — "the base of what?"): the rules as they ran, replayed — `unpatched 25%`
        : opts.stall ? /* copy:callout */ `unstuck ${pct(p.survive)} · unpatched ${pct(baseline)}`
        : noGain ? /* copy:callout */ `survives ${pct(p.survive)} · no gain` : /* copy:callout */ `survives ${pct(p.survive)} · unpatched ${pct(baseline)}`;
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
    const label = h("span", { class: "chips-inline" }, target, unlock ? h("span", { class: "unlock-label" }, p.root?.text ?? rowLabel(p.row), " · ", h("b", null, /* copy:button */ "buy")) : rowLabel(p.row), dropTag);
    // an unlock's second line is the row it inserts once bought; a root patch's is the chain's root it answers
    const root = unlock ? (p.root ? h("small", { class: "dim" }, rowLabel(p.row)) : "") : p.root ? h("small", { class: "root" }, "← ", p.root.text) : "";
    // QA 1a2a4a9 (O: "tapping a patch card applied it and jumped to camp; I meant to select it"): with `opts.select` (the death screen)
    // a tap lights the tablet and the gem applies the lit one — one model: tablets choose, the gem acts
    const btn: HTMLButtonElement = h("button", { class: `patch tablet${p.remove ? " remove" : ""}${p.below_bar || held >= 0 || opts.nothingBeatsBase || noGain ? " below" : ""}${noGain ? " no-gain" : ""}${unlock ? " unlock" : ""}${held >= 0 ? " held" : ""}`,
      onclick: opts.select ? () => opts.select!(btn) : onclick, ...(full ? { "data-full": "1" } : {}) },
      h("b", { class: "rank num", "aria-hidden": "true" }), h("span", { class: "patch-main" }, label, root),
      h("span", { class: "patch-nums" },
        // Cut 17 §4: `survives N %` as a gauge on the patch tablet (the number stays beside it)
        unlock || held >= 0 ? "" : h("span", { class: "gauge", "aria-hidden": "true" }, h("i", { style: `width:${Math.round(Math.max(0, Math.min(1, p.survive)) * 100)}%` })),
        h("span", { class: "num surv" }, line),
        reachSpan(p, stallish)));
    applyOf.set(btn, onclick);
    return btn;
  });
  // Cut 20 §4 (AD: "`nothing beats base` … but lists three patches anyway"): under that header the candidates are no patches — dim,
  // folded behind `others` (a tap unfolds them; the gem stays `edit`)
  if (head) {
    const fold = h("div", { class: "patches-fold", hidden: true }, ...rows);
    const more: HTMLButtonElement = h("button", { class: "mini more patches-more", onclick: () => { fold.hidden = false; more.remove(); } }, /* copy:button */ "others", h("small", { class: "num dim" }, ` ${rows.length}`));
    const box = h("div", { class: "patches none-beats" }, head, more, fold); renumber(fold); return box;
  }
  const box = h("div", { class: "patches" }, ...rows); renumber(box); return box;
}
/** QA e75ec29 (Q: "I read the gem as the best fix … rank by what is shown or show the ranking key"): the tablets carry their place
 *  (`1.` `2.` `3.`), renumbered whenever the order changes (the camp's reach landing re-ranks them). */
function renumber(host: HTMLElement): void {
  host.querySelectorAll<HTMLElement>(":scope > button.patch > .rank").forEach((r, i) => { r.textContent = `${i + 1}.`; });
}
const EXIT_VERBS = new Set(["return", "bank"]);
/** Each patch tablet's action (apply, buy, open the drop sheet, open the camp on a held row) — the gem's, when tablets only select. */
export const applyOf = new WeakMap<HTMLElement, () => void | Promise<void>>();

/** A patch's reach line: `reach +8%`; with the camp's own measure (`deathDeltas`) the depth and its ± — `reach D6 +8% ±3`, and
 *  `reach D6 ~0` inside the ± (as an unlock card's); `reach …` while the camp's measure is pending (`camp_pending`: the verdict's
 *  12-sim estimate is not the camp's number — QA 23ed91f, K: "`reach +8%` … the shaft went D5 79% → 78%"). A stall patch's first
 *  line already says `reach`: its delta is bare (`+84%`). */
function reachSpan(p: Patch, stallish = false): HTMLElement {
  // QA a946e04 (S: `hp < 20% → return · reach D7 +0%` — "a return ends the run"): an exit row's patch takes him home; how deep the
  // camp's runs reach is not its measure (its survival is)
  if (!stallish && EXIT_VERBS.has(p.row.verb.v) && !p.remove) return h("span", { class: "num delta exit", hidden: true });
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
  const host = el.querySelector<HTMLElement>(":scope > .patches-fold") ?? el;   // Cut 20 §4: the folded block's tablets
  const buttons = [...host.querySelectorAll<HTMLElement>(":scope > button.patch")];
  patches.forEach((p, i) => {
    const f = filled[i]; if (!f) return;
    Object.assign(p, { forecast_delta: f.forecast_delta, forecast_depth: f.forecast_depth, forecast_pm: f.forecast_pm, camp_pending: false });
    buttons[i]?.querySelector(".delta")?.replaceWith(reachSpan(p));
  });
  el.dataset.reach = "camp";
  // QA 92eb880 (N: the lit patch read `reach D8 −4%`, applied, and every floor fell; `rest 75%` lit over `return 100%`): once the camp's
  // reach is in, the tablets re-rank by it — a gain first, then the ones inside the ± (the higher survival first), a loss last and dim
  // (`.neg`: the gem never applies it); a held row or a below-bar alternative stays under them
  // QA 1a2a4a9 (P: `return 100% / rest 67% / read unknown 33%` became `return / read unknown 33% / rest 67%` once the reach landed):
  // Cut 19 §4's order, the core's `rank_patches` — survival first, the reach only inside SURVIVE_BAND of the best survival left
  const pool = patches.map((p, i) => ({ p, b: buttons[i], i })).filter((x) => x.b);
  const pmOf = (p: Patch): number => p.forecast_pm !== undefined ? Math.max(1, Math.round(p.forecast_pm * 100)) : 0;
  const moveOf = (p: Patch): number => { const d = Math.round(p.forecast_delta * 100); return p.insert_at < 0 || Math.abs(d) <= pmOf(p) ? 0 : d; };
  const below = pool.filter((x) => x.b.classList.contains("below") && !x.b.classList.contains("neg"));
  const live = pool.filter((x) => !below.includes(x));
  // a loss stays dim (`.neg`, never the gem's) but keeps its survival's place: a 67 % row never drops under a 33 % one (the core
  // sinks a loss under DELTA_SINK; here the reach is the camp's, landing after the player has read the list — the order moves less)
  for (const x of live) x.b.classList.toggle("neg", moveOf(x.p) < 0);
  const ordered = [...rankBand(live, moveOf), ...below];
  for (const x of ordered) host.appendChild(x.b);
  renumber(host);
}

/** Cut 19 §4 (core `trace::SURVIVE_BAND`): survival decides a place when two patches' survival differs by more than 10 pts. */
export const SURVIVE_BAND = 0.10;
/** The core's `rank_patches`, over the camp's reach: each place goes to the patch with the best reach move (when any moves it up;
 *  else the best survival) among those within SURVIVE_BAND of the best survival left — an order, not a pairwise rule. Ties keep
 *  the incoming order. */
export function rankBand<T extends { p: Patch; i: number }>(xs: T[], moveOf: (p: Patch) => number): T[] {
  const pool = [...xs], out: T[] = [];
  const byMove = pool.some((x) => moveOf(x.p) > 0);
  while (pool.length) {
    const top = Math.max(...pool.map((x) => x.p.survive));
    let best = -1;
    pool.forEach((x, j) => {
      if (x.p.survive < top - SURVIVE_BAND - 1e-9) return;
      if (best < 0) { best = j; return; }
      const q = pool[best], d = moveOf(x.p) - moveOf(q.p), s = x.p.survive - q.p.survive;
      if (byMove ? d > 0 || (d === 0 && s > 1e-9) : s > 1e-9 || (Math.abs(s) <= 1e-9 && d > 0)) best = j;
    });
    out.push(pool.splice(best, 1)[0]);
  }
  return out;
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
