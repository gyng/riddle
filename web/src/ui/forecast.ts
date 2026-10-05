// Forecast panel: reach% bars per depth up to known_to, `?` beyond; top causes.
// Cut 4 §8: the row for best+1 names the top cause even at 0% (`D6 0% · goblin warlord`) when causes are known.
// Cut 6 §5: that row also names the boss's counter when the lineage knows it. Cut 6 §9: `app` repaints quietly with
// the refined (100-sim) forecast 2 s after a paint with the rules unchanged (same `onForecast` listener).
// Cut 7 §2: `yours: 3 of 5 rows` under the bars — the rows of the active set the player wrote or edited (`Row.origin`),
// repainted on every edit (the forecast itself waits for the engine).
// Cut 9 §3: a bar reads `D4 71% ±6` when the engine sends `pm` (the binomial half-width), so a wobble reads as noise.
// Cut 9 §4 appended `· card R2 first` to the `yours` line; gone since Cut 12 §1 put a card before the engagement row on purpose
// (both QA players on 952e306: "what 'first' means for a row that sits third").
// Cut 10 §2: a boss floor whose counter is known and whose row is absent reads `D9 0% · warlord · try: attack boss` (the
// engine's `try`, or the client's read of `Lineage.counters` against the set); tapping the bar inserts the row at the top.
// Cut 12 §1: `yours: n of m rows` counts own rows (card rows sit outside `max_rows`); §3: one line under the depths says how a
// send ends when the engine sends `ends` (`bank 40% · return 35% · death 25% · ~$54`); §6: a combo is named, not counted.
// Cut 13 §5: the first paint (`Forecast.refined` false) carries `…` after each `±` so the refine's landing does not read as a
// re-roll; the ends line has its own `±` on the death share (`death 5% ±4`, `ForecastEnds.pm`).
// Cut 16 §1: under the ends line, `D3 · D4 · picked clean` (small, dim) while `Lineage.picked` holds depths.
import { conceptCap, conceptIcon } from "./concepts";
import { foeSrc } from "./skin";
import { meterCompare } from "./meters";
import { isWide } from "./frame";
import { sysOpen } from "./systems";
import { BANDS, FORKS, biomeAt, frontiers, routeChips, routeForks } from "./route";
import type { App } from "../app";
import type { Forecast, ForecastTry, ForecastVs, Row, VsMove } from "../engine/types";
import { h, clear, pct, replace } from "./dom";
import { closeAllSheets } from "./sheet";
import { pickedLine } from "./report";
import { shaftOath } from "./oaths";
import { kw } from "./tips";

const sameRow = (a: Row, b: Row): boolean =>
  a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && a.conds.length === b.conds.length &&
  a.conds.every((c, i) => c.k === b.conds[i].k && (c.n ?? "") === (b.conds[i].n ?? "") && (c.t ?? "") === (b.conds[i].t ?? ""));
/** Cut 10 §2: the known-but-absent counter for a boss cause, from `Lineage.counters` (a client fallback for the wire's `try`). */
export function clientTry(app: App, cause: string | undefined): ForecastTry | undefined {
  if (!cause) return undefined;
  const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
  const c = (app.lineage.counters ?? []).find((k) => k.boss === key || key.endsWith(k.boss));
  if (!c || !c.row || typeof c.row === "string") return undefined;
  return app.rules.rows.some((r) => sameRow(r, c.row as Row)) ? undefined : { row: c.row, text: c.text };
}

/** QA 912e135: a forecast killer by name, `unseen <name>` for a kind the bestiary lists as not seen (a hazard or an unlisted cause as is). */
export function killerName(app: App, cause: string): string {
  const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
  const row = (app.lineage.ledger ?? []).find((r) => r.kind === key);
  const name = cause.replace(/_/g, " ");
  // QA 524827b (qaAA: `unmet archer 21%` unexplained): a kind the bestiary has not seen reads `unseen` (the bestiary's own sense)
  return row && !row.seen ? /* copy:callout */ `unseen ${wallName(key)}` : name;
}

/** Cut 18 §3: the sealing boss's kind as one word (`goblin_warlord` → `warlord`). */
export const wallName = (kind: string): string => kind.replace(/_/g, " ").trim().split(/\s+/).pop() ?? kind;

/** Cut 28 §1 (AU, AV: `D14 2% · sealed by mother`, no visible path): the sealing boss's counter where the wall is — the lineage's learned
 *  counter (`mother: fire`), else the core's hint for it (`ForecastDepth.counter_hint`: `mother: fire?`), else `mother: ?` (a fact to learn). */
export function wallCounter(app: Pick<App, "lineage">, kind: string, d?: { counter?: string; counter_hint?: string }): string {
  const name = wallName(kind), key = kind.replace(/ /g, "_");
  // the core's wall (`Lineage.walls`): the counter as the lineage knows it (`mother: fire`, `mother: ?`)
  const w = (app.lineage.walls ?? []).find((x) => x.boss === key || key.endsWith(x.boss) || x.boss.endsWith(key));
  const known = (app.lineage.counters ?? []).find((c) => c.boss === key || key.endsWith(c.boss) || c.boss.endsWith(name));
  // docs/COPY.md pass 2 (both readers took `warlord: aim` for the boss's move): a known counter reads as the rule that beats him
  // (`counter: attack boss`), under the notch that names him (`behind warlord`)
  if (known?.text) return /* copy:callout */ `counter: ${known.text}`;
  // (a known word the lineage has no counter row for: `warlord: aim` → `counter: aim`; an unknown one stays `mother: ?`)
  if (w?.fact) return w.fact.replace(/^[^:]+: (?!\?)/, /* copy:callout */ "counter: ");
  const text = d?.counter ?? known?.text?.replace(/,? ?boss$/, "").replace(/^(attack|throw|read|drink) /, "") ?? (d?.counter_hint ? `${d.counter_hint.replace(/\?$/, "")}?` : "?");
  return `${name}: ${text}`;
}
/** Cut 28 §1 (AV: `bounty D13 · missed` never said what it pays or needs): the bounty says both — `bounty · D13 · $×2 · reach` (its
 *  multiplier, and what it needs: the floor reached; the core's `needs` when it sends one, e.g. `mother: fire`). */
export function bountyText(b: { depth: number; mult?: number; needs?: string; pays?: string; fact?: string }): string {
  // docs/COPY.md pass 4 (`bounty · $×2 · item · reach · warlord: aim`): what it pays, and a need only when it is more than getting there
  const needs = b.needs && b.needs !== "reach" ? /* copy:callout */ ` · needs ${b.needs}` : "";
  return /* copy:callout */ `bounty · D${b.depth} · ${b.pays?.replace(/\$×(\d+)/, "$1× gold") ?? `${b.mult && b.mult > 1 ? b.mult : 2}× gold`}${needs}`;
}
/** Cut 20 §5: the bounty's multiplier as the notch reads it (`×2`; a number on the wire above 1 is the multiplier). */
/** QA 912e135 (qaW: `D8 ×2` in the shaft — "no source"): the multiplier says what it multiplies — the floor's gold (`$×2`). */
/** QA 524827b (qaAA: `D8 $×2 · warlord` unexplained): the multiplier names itself (`bounty $×2`). */
export const bountyMult = (b: boolean | number | undefined): string => /* copy:callout */ `bounty ${typeof b === "number" && b > 1 ? b : 2}× gold`;   // docs/COPY.md pass 5: `$×2` read as "$2"

/** Cut 9 §3: the half-width (a 0..1 fraction like `reach`) in percentage points, never `±0` — a forecast is never exact. */
export const pmPts = (pm: number): number => Math.max(1, Math.round(pm * 100));
/** QA e75ec29 (Q: `D1 100% ±1`, a bound above 100 %): the ± a share shows — none where it reads 0 % or 100 %, and never so wide that
 *  it would cross either end (`99% ±1`, not `99% ±3`). Undefined when there is nothing to show. */
export const pmShown = (share: number, pm: number | undefined): number | undefined => {
  if (pm === undefined) return undefined;
  const r = Math.round(share * 100);
  return r <= 0 || r >= 100 ? undefined : Math.min(pmPts(pm), 100 - r, r);
};

/** Cut 23 §2 (AJ: `death 0%`, and the next run died): a share the sims sampled at 0 of N prints `<N%` (the smallest share one sim
 *  makes, the core's `Forecast.low`: `death <2%`), never `0%`; a share above 0 that rounds to 0 prints `<1%`. Without `low` (an
 *  older core) the share prints as before. */
export function share(x: number, low: number | undefined): string {
  if (low === undefined || !Number.isFinite(x) || Math.round(x * 100) > 0) return pct(x);
  return x > 0 ? "<1%" : `<${Math.max(1, Math.round(low))}%`;
}
/** QA 912e135 (qaW: `bank <2% · return <2% · death 100%` — "sums past 100"): an end's share sampled at N of N prints `>{100 − low}%`
 *  (`death >98%`), so the three ends never read past 100 together; above the low end it prints as `share`. */
export function endShare(x: number, low: number | undefined): string {
  if (low !== undefined && Number.isFinite(x) && Math.round(x * 100) >= 100) return x < 1 ? ">99%" : `>${100 - Math.max(1, Math.round(low))}%`;
  return share(x, low);
}
/** The low end of the forecast now painted (`Forecast.low`, else from its `sims`). */
export const lowOf = (f: { low?: number; sims?: number } | null | undefined): number | undefined => f?.low ?? (f?.sims ? Math.ceil(100 / f.sims) : undefined);

/** Cut 22 §4: a move in whole points, signed (`+6`, `−3`), never a `%` — a delta must not read as a chance. */
export const signedPts = (pts: number): string => `${pts < 0 ? "−" : "+"}${Math.abs(pts)}`;
/** Cut 22 §3: a paired move as it reads — `+6` / `−3`, `≈` inside its own ± (or rounding to 0): no call. Cut 24 §4 (AK: "the live
 *  forecast solves most edits"; AL: own rows `≈`): inside a ± it reads `≈ ±4` — a small real move is unresolved, not "no change";
 *  `≈` alone only where there is no ± (a dead edit's move, `flatVs`: zero by construction). */
export function moveOf(m: VsMove | number | undefined): { pts: number; text: string; dir: "up" | "down" | "flat" } | null {
  if (m === undefined || m === null) return null;
  const mv: VsMove = typeof m === "number" ? { delta: m } : m;
  if (typeof mv.delta !== "number" || !Number.isFinite(mv.delta)) return null;
  // QA 912e135 (qaW: `▲` against the bars' own numbers): with the sent set's share (`base`) the points are the two shown shares'
  // difference (`29% → 61%` is `▲32`, never `▲31` by rounding the delta alone)
  const pts = typeof mv.base === "number" ? Math.round((mv.base + mv.delta) * 100) - Math.round(mv.base * 100) : Math.round(mv.delta * 100);
  const flat = pts === 0 || (mv.pm !== undefined && Math.abs(mv.delta) <= mv.pm);
  // docs/COPY.md pass 2: a move inside its ± reads `same` (the `≈ ±N` of a no-call read as a value and a spread)
  return flat ? { pts, text: /* copy:callout */ "same", dir: "flat" } : { pts, text: signedPts(pts), dir: pts > 0 ? "up" : "down" };
}
/** Cut 29 (owner: misread numbers are restyled, not dropped — `88% ±7 −4` read as a range or a penalty): the ± is drawn as a noise band
 *  (its width the spread's points, its text kept for readers and tools), the move as a signed chip. */
export const bandW = (pm: number): string => `--pmw:${Math.min(36, Math.max(4, pm * 2))}`;
/** Cut 22 §3: a notch's or a gem's move as a tiny mark — `▲6`, `▼3` (nothing inside its ±). */
export function moveMark(m: VsMove | number | undefined, bare = false, worse = false): HTMLElement | "" {
  // a move inside its ± marks nothing on a notch, a bar or a gem (a column of `≈` is noise); the line's `D8 ≈` says it
  const v = moveOf(m); if (!v || v.dir === "flat") return "";
  // `bare`: the arrow alone (a gem in the narrow shaft; its number is on the line under it)
  return h("i", { class: `vsm dlt ${tone(v.dir, worse)}` }, bare ? (v.dir === "up" ? "▲" : "▼") : `${v.dir === "up" ? "+" : "−"}${Math.abs(v.pts)}`);
}
/** QA 778fa1b (qaV: `death −90` and `death 9% ▼` drawn red — "a drop in death drawn as good"): the colour says good or bad, the arrow
 *  and the sign say which way — a share where more is worse (death) takes the other colour. */
const tone = (dir: "up" | "down" | "flat", worse: boolean): string => !worse || dir === "flat" ? dir : dir === "up" ? "down" : "up";
/** Cut 22 §3: the line under the shaft — `vs last · D8 +6 · bank +4`: the depth whose move is the largest outside its ± (else the
 *  frontier's, `D8 ≈`), the bank's move when the gems show, and the death's when it clears its ±. Null without a move to show. */
export function vsLine(app: App, vs: ForecastVs | null, f: Forecast | null, withEnds: boolean): HTMLElement | null {
  if (!sysOpen(app.lineage, "vs")) return null;   // Cut 29 §2: the vs line opens at the first plateau
  // QA 778fa1b (qaV): an edit whose move is still being measured reads `vs sent …`, never the last move or a hollow `≈`
  if (!vs) return app.vsPending() ? h("div", { class: "shaft-vs num rough pending" }, h("span", { class: "vs-label" }, /* copy:callout */ "vs last run", "…")) : null;
  const rough = vs.refined === false;
  const start = forecastStart(app, f), next = Math.max(start, app.lineage.best_depth + 1);
  const ds = vs.depths.filter((d) => d.depth >= start && d.depth <= (f?.known_to ?? Infinity));
  // QA 912e135 (qaW: the patch read `reach D7 +24`, the camp after it `vs sent · D6 +34`): the frontier's move leads when it clears its ±
  // (the floor a death's patch measures, best + 1); else the largest outside its ±
  const moved = ds.map((d) => ({ d, m: moveOf(d)! })).filter((x) => x.m && x.m.dir !== "flat");
  let head = moved.find((x) => x.d.depth === next) ?? moved.sort((a, b) => Math.abs(b.m.pts) - Math.abs(a.m.pts) || b.d.depth - a.d.depth)[0];
  if (!head) { const d = ds.find((x) => x.depth === next) ?? ds[ds.length - 1]; if (d) head = { d, m: moveOf(d)! }; }
  // a first-pass move inside its ± is not yet a call: `…` until the refine's lands (a move outside it shows, dim)
  // QA 912e135 (qaW: "▲ = the shown % minus the sent set's %, or it says what it is measured against"): a term reads the sent set's
  // share and the shown one (`D6 29→61%`, the bar's own number), the move's colour on the arrow; an older core's bare move stays `+6`
  const term = (label: string, m: ReturnType<typeof moveOf>, key: string, worse = false, raw?: VsMove | number, shownKey = key): HTMLElement => {
    const b = typeof raw === "object" && typeof raw.base === "number" ? raw : undefined;
    // QA 0c6e126 (qaY: `D5 88→84%` — "no forecast on screen ever showed 88%"): `from→to` only from a share of the sent set this camp
    // painted (`App.baseWasShown`); measured again under a changed lineage (a purchase, the cage), the term is the signed move
    const from = b ? Math.round(b.base! * 100) : 0;
    const txt = m!.dir === "flat" ? (rough ? "…" : m!.text) : b && app.baseWasShown(shownKey, from) ? `${from}→${Math.round((b.base! + b.delta) * 100)}%` : m!.text;
    return h("span", { class: "vs-term", "data-k": key }, h("i", { class: "sep" }, " · "), label, " ", h("b", { class: `dlt ${tone(m!.dir, worse)}` }, txt));
  };
  const terms: HTMLElement[] = [];
  if (head?.m) terms.push(term(`D${head.d.depth}`, head.m, "depth", false, head.d, `D${head.d.depth}`));
  const bank = moveOf(vs.bank), death = moveOf(vs.death);
  if (withEnds && bank) terms.push(term(/* copy:label */ "full haul", bank, "bank", false, vs.bank));
  if (withEnds && death && death.dir !== "flat") terms.push(term(/* copy:label */ "death", death, "death", true, vs.death));
  // QA 912e135 (qaX: `death −11` in green while the stall share rose 0 → 11 %): a stall that moves outside its ± is its own term, worse up
  const stall = moveOf(vs.stall);
  if (withEnds && stall && stall.dir !== "flat") terms.push(term(/* copy:label */ "stall", stall, "stall", true, vs.stall));
  // Cut 28 §1: with an oath sworn, the edit's move on it is its own term (`oath +12`)
  const oath = moveOf(vs.oath);
  if (withEnds && oath && oath.dir !== "flat") terms.push(term(/* copy:label */ "challenge", oath, "oath", false, vs.oath));
  if (!terms.length) return null;
  // QA 778fa1b (qaU: `death −10` stayed while the refine beside it read 22 → 27 %): a move paired on the first pass trails `…` and
  // reads dim until the refine's is asked again and lands (`ForecastVs.refined`; absent on an older core: no mark)
  return h("div", { class: `shaft-vs num${rough ? " rough" : ""}`, "data-refined": rough ? "0" : vs.refined ? "1" : "" }, h("span", { class: "vs-label" }, /* copy:callout */ "vs last run", rough ? "…" : ""), ...terms);
}

/** Cut 28 §2 (AV: "death jumped 14 → 36 %; I blamed my new rows — the real cause was the party dying"): the state's part of the move
 *  since the send, its own line (`party −2 jackals · death +24`), apart from the rows' `vs sent` (app.ts `smove`, ui/attrib.ts). */
export function stateLine(app: App): HTMLElement | null {
  if (!sysOpen(app.lineage, "vs")) return null;   // Cut 29 §2: the state's part of the move is the vs line's — open with it
  // the core's attribution (`forecastMove`): one line per state part whose move clears its ± (the two largest), each its headline term
  const fm = app.fmove;
  if (fm) {
    const parts = fm.parts.filter((p) => p.kind !== "rows" && p.kind !== "route").map((p) => ({ p, t: headTerm(app, p.move) })).filter((x) => x.t)
      .sort((a, b) => Math.abs(b.t!.m.pts) - Math.abs(a.t!.m.pts)).slice(0, 2);
    if (!parts.length) return null;
    const box = h("div", { class: "shaft-states" });
    for (const { p, t } of parts) box.appendChild(h("div", { class: `shaft-state num dlt-line${fm.refined ? "" : " rough"}`, "data-k": p.kind }, h("span", { class: "vs-label" }, p.text),
      h("span", { class: "vs-term", "data-k": t!.key }, h("i", { class: "sep" }, " · "), t!.key, " ", h("b", { class: `dlt ${tone(t!.m.dir, t!.worse)}` }, t!.m.text))));
    return box;
  }
  const sm = app.moveByCore ? null : app.smove; if (!sm || !sm.terms.length) return null;   // (the client's own read: an older core's fallback)
  return h("div", { class: "shaft-states" }, h("div", { class: "shaft-state num dlt-line", "data-k": "state" }, h("span", { class: "vs-label" }, sm.label),
    ...sm.terms.map((t) => h("span", { class: "vs-term", "data-k": t.key }, h("i", { class: "sep" }, " · "), t.key, " ", h("b", { class: `dlt ${tone(t.pts > 0 ? "up" : "down", t.worse)}` }, signedPts(t.pts))))));
}
/** A move's headline term: the death share when it clears its ±, else the bank's, else the frontier's depth (else the depth that moved most). */
function headTerm(app: App, v: ForecastVs): { key: string; m: NonNullable<ReturnType<typeof moveOf>>; worse: boolean } | null {
  const death = moveOf(v.death), bank = moveOf(v.bank);
  if (death && death.dir !== "flat") return { key: /* copy:label */ "death", m: death, worse: true };
  if (bank && bank.dir !== "flat") return { key: /* copy:label */ "bank", m: bank, worse: false };
  const next = app.lineage.best_depth + 1;
  const ds = v.depths.map((d) => ({ d, m: moveOf(d) })).filter((x): x is { d: typeof x.d; m: NonNullable<typeof x.m> } => !!x.m && x.m.dir !== "flat");
  const hd = ds.find((x) => x.d.depth === next) ?? ds.sort((a, b) => Math.abs(b.m.pts) - Math.abs(a.m.pts))[0];
  return hd ? { key: `D${hd.d.depth}`, m: hd.m, worse: false } : null;
}
/** QA 0c6e126 (qaY: a bought potion's `D5 71→65` read as a loss with no reason): the last lineage change's move on these rules
 *  (`App.lmove`) at the frontier — `buy · D5 ≈ ±9` inside the bar's ±, else `buy · D5 −6`. Null once the rules differ. */
export function lmoveLine(app: App, f: Forecast | null): HTMLElement | null {
  const lm = app.lmove; if (!lm || !f || lm.rules !== JSON.stringify(app.rules.rows)) return null;
  // QA 524827b (qaAB: the cage sheet quoted `armour D6 …`, the line after the pick read `cage · D7 +20`): the depth is the cage sheet's —
  // the frontier while the sims still reach it (over 5 %, the core's `WALL_REACH`), else the floor above it
  const front = Math.max(forecastStart(app, f), app.lineage.best_depth + 1);
  const open = (f.depths.find((x) => x.depth === front)?.reach ?? 0) > 0.05 ? front : Math.max(forecastStart(app, f), front - 1);
  const d = lm.depths.find((x) => x.depth === open) ?? lm.depths.find((x) => x.depth === front) ?? lm.depths[lm.depths.length - 1]; if (!d) return null;
  const m = moveOf({ delta: d.delta, pm: d.pm }); if (!m) return null;
  return h("div", { class: `shaft-lm num dlt-line`, "data-k": lm.label }, h("span", { class: "vs-label" }, lm.label), h("span", { class: "vs-term" }, h("i", { class: "sep" }, " · "), `D${d.depth} `, h("b", { class: `dlt ${m.dir}` }, m.text)));
}
/** Cut 29 §6 (AX: `$81` banked under `~$260`): the waystone passage the send pays into the purse is apart from what a run brings home —
 *  `~$125/run +$135 passage`. */
const passageEl = (p: number | undefined): HTMLElement | "" => p && p > 0 ? h("small", { class: "passage dim" }, /* copy:callout */ ` +$${Math.round(p)} passage`) : "";
export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const yours = h("div", { class: "fc-yours num" });
  const ends = h("div", { class: "fc-ends num dim", hidden: true });
  // Cut 16 §1: the depths picked clean (`Lineage.picked`), small and dim under the ends line — why the `~$N` is lower than it was
  const picked = h("div", { class: "fc-picked num dim", hidden: true });
  const paintPicked = (): void => { const p = app.lineage.picked ?? []; picked.hidden = !p.length; replace(picked, p.length ? pickedLine(p) : ""); };
  // Cut 22 §3: the edit's paired move, the shaft's line, under the ends (and a mark on each bar)
  const vsHost = h("div", { class: "fc-vs", hidden: true });
  const paintVs = (): void => { const line = vsLine(app, app.vsShown(), app.lastForecast, true), st = stateLine(app); vsHost.hidden = !line && !st; replace(vsHost, st ?? "", line ?? ""); };
  // QA 778fa1b (qaU: 94/78/42 then 95/73/36 for the same rules, "no sign it was settling"): the first pass's label trails `…`
  const settling = h("span", { class: "fc-settling", hidden: true }, "…");
  // QA 778fa1b (qaV: `D1 100%` beside `death 100%` read as dying on D1): the bars say what they count — the share that reaches each floor
  // Cut 29 §3: the last two runs side by side (the core's `Lineage.meters.runs`, older first) — the phone's under the forecast (the desktop's
  // is the right column's meters)
  const runs = app.lineage.meters?.runs ?? [];
  const cmp = runs.length >= 2 && !isWide() ? meterCompare(runs[runs.length - 2], runs[runs.length - 1]) : null;
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast", settling), h("div", { class: "label reach-label dim" }, kw("reach")), bars, ends, vsHost, picked, yours, causes, cmp);
  // Cut 8B §4: `· 1 combo` when the set has one (engine data; the count is the client's mirror of `Lineage.combos`)
  // Cut 12 §6: the combo's name (engine data: `Vocabulary.combos[].name`), not `1 combo`
  const paintYours = (): void => {
    const n = app.playerRows(), m = app.ownRows(), combos = app.combos();
    replace(yours, h("span", { class: n ? "" : "dim" }, /* copy:callout */ `written: ${n} of ${m} rule${m === 1 ? "" : "s"}`),
      combos.length ? h("span", { class: "combos" }, ` · ${combos.map((c) => c.name).join(" · ")}`) : "");
  };
  // Cut 12 §3: `bank 40% · return 35% · death 25% · ~$54` — the three shown always so the trade reads; absent on an older core
  const paintEnds = (f: Forecast): void => {
    const e = f.ends;
    ends.hidden = !e;
    if (!e) return;
    // a stall share only when there is one: `bank 0% · return 20% · stall 50% · death 30% · ~$25`
    const stall = e.stall && Math.round(e.stall * 100) > 0 ? /* copy:callout */ ` · stall ${pct(e.stall)}` : "";
    const lo = lowOf(f), eh = (x: number): string => endShare(x, lo);
    const epm = pmShown(e.death, e.pm);
    const pm = epm !== undefined ? h("small", { class: "dim pm band", style: bandW(epm), title: `±${epm}` }, /* copy:none */ ` ±${epm}${f.refined === false ? "…" : ""}`) : "";   // Cut 29: `±6` read as −6 — a band
    // QA 1a2a4a9 (O: `D5 76%` beside `death 100%` read as a contradiction): the split is labelled — how a run ends, not how deep
    replace(ends, h("span", { class: "label ends-label" }, kw("ends", /* copy:label */ "run outcomes")), " ", /* copy:callout */ `full haul ${eh(e.bank)}`, /* copy:callout */ ` · turn back ${eh(e.return)}`, stall, /* copy:callout */ ` · death ${eh(e.death)}`, pm, h("span", { class: "gold" }, /* copy:callout */ ` · avg $${Math.round(e.gold - (e.passage ?? 0))}/run`, passageEl(e.passage)));
  };
  /** The named counter of a boss cause (`goblin_warlord`, `goblin warlord pack`) from `lineage.counters`. */
  const counterFor = (cause: string): string | undefined => {
    const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
    return (app.lineage.counters ?? []).find((c) => c.boss === key || key.endsWith(c.boss))?.text;
  };
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes); paintEnds(f); paintPicked(); paintVs();
    const vsBy = new Map((app.vsShown()?.depths ?? []).map((d) => [d.depth, d]));
    el.dataset.refined = f.refined === undefined ? "" : f.refined ? "1" : "0";   // dev: tools read which pass painted
    // QA 92eb880 (M: "D6 32%±13 → 38%±10 on opening edit"): the first pass paints dim, its ± trailing `…`, until the refine lands
    el.classList.toggle("rough", f.refined === false);
    settling.hidden = f.refined !== false;
    const first = f.refined === false ? "…" : "";   // Cut 13 §5: the first paint's ± trails `…`; the refine's does not
    const next = app.lineage.best_depth + 1;
    // Cut 4 §8 names the top cause on best+1; QA 23ed91f (L: `D9 0% ±1 · rat` for a set that dies on D1–D2): when nobody gets near
    // best+1, the cause sits on the floor where the reach falls most
    let causeAt = next;
    { const byD = new Map(f.depths.map((d) => [d.depth, d.reach])); if ((byD.get(next - 1) ?? 1) < 0.05) {
      let drop = -1; for (const d of f.depths) { const fall = (byD.get(d.depth - 1) ?? 1) - d.reach; if (fall > drop) { drop = fall; causeAt = d.depth; } } } }
    // Cut 21 §1: a waystone start skips the floors above it (they are not run) — the bars start on the start floor; QA a946e04 (T: the
    // tablet read D5 while the sims ran from D1 — `D5 88%`): the floor the forecast's sims started on (`Forecast.start`) when it says
    const start = forecastStart(app, f);
    // Cut 24 §5 (AK: "the warlord forecast on D9, met on D8"): a `try` is the boss's, read on the floor he is met on (`ForecastTry.met`)
    // when that floor is on the panel — the D9 row keeps his wall, the D8 row names him and carries the counter
    const shown = new Set(f.depths.filter((d) => d.depth >= start).map((d) => d.depth));
    const tryAt = new Map<number, NonNullable<typeof f.depths[number]["try"]>>();
    for (const d of f.depths) if (d.try) tryAt.set(d.try.met !== undefined && shown.has(d.try.met) ? d.try.met : d.depth, d.try);
    for (const d of f.depths) {
      if (d.depth < start) continue;
      // Cut 18 §3: a floor the boss above seals (`ForecastDepth.wall`) names him as the cause: `D9 0% · warlord wall`
      const wall = d.wall ? wallName(d.wall) : undefined;
      const cause = wall ? undefined : d.cause ?? (d.depth === causeAt ? f.causes[0]?.cause : undefined);
      // Cut 10 §2: the known-but-absent counter — the wire's `try`, else the client's read of the counters against the set
      const tr = tryAt.get(d.depth) ?? (d.depth === next && !d.try && !tryAt.size ? clientTry(app, cause) : undefined);
      // Cut 24 §5: the floor a boss stands on names him (`D8 · warlord`) unless its cause already does
      const boss = d.boss && !(cause && cause.replace(/_/g, " ").includes(wallName(d.boss))) && !wall ? wallName(d.boss) : undefined;
      // Cut 6 §5: `D6 0% · goblin warlord · counter: attack boss` when the top cause is a boss whose counter row is known (and held)
      const counter = cause && d.depth === next && !tr ? counterFor(cause) : undefined;
      // the try hint rides the track, on the depth's own line, the track as wide as every other row's (QA on 50bb162: `D9 0% ±1`
      // wrapped `· try: attack boss` under it with a shorter bar)
      const track = h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` }));
      // QA e75ec29 (Q: "the D1 bar at 100 % is shorter than the grey D2+ track"): every row's track is one width — the cause, the wall,
      // the bounty and the counter go on a line of their own under the track (`.why`), never into the number's column
      const dpm = pmShown(d.reach, d.pm);
      const why = [
        // QA 524827b (qaAB: a bare `goblin` under D7, `warlord wall` — unexplained): the name says it is the floor's top killer, and the
        // wall says what it does (the boss above seals the stairs)
        cause ? h("small", { class: "dim" }, /* copy:callout */ ` · killer: ${cause.replace(/_/g, " ")}`) : "",
        boss ? h("small", { class: "boss-here" }, ` · ${boss}`) : "",
        wall ? h("small", { class: "wall" }, /* copy:callout */ ` · behind ${wall}`) : "",
        wall && sysOpen(app.lineage, "walls") ? h("small", { class: "wall-counter" }, ` · ${wallCounter(app, d.wall!, d as { counter?: string; counter_hint?: string })}`) : "",   // Cut 28 §1: the wall's path
        // Cut 20 §5: the bounty floor; Cut 28 §1: what it pays and needs (`bounty · $×2 · item · reach`)
        d.bounty ? h("small", { class: "bounty-x" }, ` · ${app.lineage.bounty?.depth === d.depth && (app.lineage.bounty.pays || app.lineage.bounty.needs) ? bountyText({ ...app.lineage.bounty, depth: d.depth }).replace(/^bounty · D\d+ · /, /* copy:callout */ "bounty · ") : bountyMult(d.bounty)}`) : "",
        counter ? h("small", { class: "dim" }, /* copy:callout */ ` · counter: ${counter}`) : "",
      ].filter((x) => x !== "");
      const inner = [
        // gfx raters (every round: "web bars"): each floor is the shaft's hex gem, lit by its reach; the track under it is a thin rail
        h("span", { class: "d num" }, h("span", { class: "hex", style: `--reach:${d.reach.toFixed(3)}`, "aria-hidden": "true" }), `D${d.depth}`),
        tr ? h("span", { class: "track-cell" }, track, h("small", { class: "try" }, /* copy:none */ `try: ${tr.text}`)) : track,
        // a `try` row keeps one line (its hint rides the track; the boss beside the number, as before)
        h("span", { class: "n num" }, share(d.reach, lowOf(f)), dpm !== undefined ? h("small", { class: "dim pm band", style: bandW(dpm), title: `±${dpm}` }, /* copy:none */ ` ±${dpm}${first}`) : "", moveMark(vsBy.get(d.depth)), ...(tr ? why.filter((w) => w instanceof HTMLElement && (w.classList.contains("boss-here") || w.classList.contains("wall"))) : [])),   // gfx round 18 (raters: "the D8 row crams a pill, deltas and tags"): a try row keeps only whose floor it is
        // QA 778fa1b (qaU: a leading `· goblin archer` under the D1 bar): on a line of its own the first cause drops its separator
        !tr && why.length ? h("span", { class: "why num" }, ...why.map((w) => { if (w instanceof HTMLElement && w.firstChild?.nodeType === 3 && /^ · /.test(w.firstChild.textContent ?? "")) { w.firstChild.textContent = (w.firstChild.textContent ?? "").slice(3); w.prepend(h("i", { class: "sep" }, " · ")); } return w; })) : "",
      ];
      // the `try` bar is a button: the row goes in at the top (position is the point), the camp opens on it
      bars.appendChild(tr
        ? h("button", { class: `bar next try${wall ? " walled" : ""}`, onclick: () => { const i = app.applyPatch({ row: tr.row, insert_at: 0, survive: 0, forecast_delta: 0 }); closeAllSheets(); app.go({ kind: "camp", highlight: i }); } }, ...inner)
        : h("div", { class: `bar${cause || wall || boss ? " next" : ""}${wall ? " walled" : ""}` }, ...inner));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, h("span", { class: "hex", "aria-hidden": "true" }), `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    // QA 23ed91f (K: "`jackal 100%` beside `death 1%` — I read it as jackal kills 100%"): a cause's share of the deaths is shown as its
    // share of the sends when the ends are known (`jackal 1%` under `death 1%`), so the two lines speak one unit
    const per = f.ends ? f.ends.death : 1;
    // QA 1a2a4a9 (O: `goblin 26% · ogre 21%` "with no heading"): the killers' line says what it lists
    if (f.causes.length) causes.appendChild(h("span", { class: "label causes-label" }, /* copy:label */ "killers"));
    // QA 912e135 (qaX: `KILLERS goblin warlord 38%` while the ledger had him unseen, no screen naming him): a killer the lineage has not met
    // reads as one — `unmet warlord`
    // gfx round 18 (raters, every round: "give KILLERS small monster portraits"): a met killer shows its face (tools/foe-portraits.py)
    for (const c of f.causes) { const nm = killerName(app, c.cause), src = /unmet/.test(String(nm)) ? null : foeSrc(c.cause); causes.appendChild(h("span", { class: "cause" }, src ? h("img", { class: "foe-face", src, alt: "", draggable: "false", "aria-hidden": "true" }) : "", nm, " ", h("b", { class: "num" }, share(c.share * per, lowOf(f))))); }
  };
  // until the first forecast arrives (≈1 s in the worker): the unknown row only
  bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, "…"), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
  // a rule edit (or a set switch) dims the numbers until the engine's next forecast paints — an empty set's takes seconds and
  // the old set's bars read as the new one's meanwhile (QA on 952e306: "set '2 0' showed set 1's D4 72%")
  const stale = (): void => { paintYours(); paintVs(); el.classList.add("stale"); };
  // QA a946e04 (S: the shaft `D5 22% ±11 · return 78%` beside the panel's `21% ±8 · return 76%` on one screen): the panel and the shaft
  // paint one forecast — `app.lastForecast`, the event both are handed (the first pass `…`, then the refine) — and the panel starts
  // from it, never from a pass of its own
  const fresh = (): void => { const f = app.lastForecast; if (!f) return; el.classList.remove("stale"); el.dataset.fc = String(app.forecastSeq); paint(f); };
  const off = app.onForecast(fresh), offRules = app.onRules(stale), offChange = app.onChange(paintYours);
  const offVs = app.onVs(() => { const f = app.lastForecast; if (f && !el.classList.contains("stale")) paint(f); else paintVs(); });
  paintYours(); paintPicked(); fresh();
  // the first forecast posts after the camp's own fetches (the worker answers in order: a forecast posted first held the
  // supply shop and the unlock shelf behind it — QA B on 952e306: "while FORECAST shows '…' the shop chips and UNLOCKS are gone")
  setTimeout(() => void app.emitForecast(), 0);
  return { el, dispose: () => { off(); offRules(); offChange(); offVs(); } };
}

/** QA a946e04: where the shown forecast's sims started — `Forecast.start` (1 when the purse cannot pay the toll), else the lineage's. */
export const forecastStart = (app: App, f: Forecast | null): number => Math.max(1, f?.start ?? app.lineage.start ?? 1);

/** QA 92eb880: the depth a lone `depth ≥ N → bank` row sends the hero home from (the smallest such N); undefined without one. */
export function bankCap(rows: Row[]): number | undefined {
  const ns = rows.filter((r) => r.verb.v === "bank" && r.conds.length === 1 && r.conds[0].k === "depth>=" && r.conds[0].n !== undefined).map((r) => r.conds[0].n!);
  return ns.length ? Math.min(...ns) : undefined;
}

/** Cut 17 §2 — the depth shaft at the camp well's right edge (docs/UI.md §2's minimap): one notch per depth D1 … D(best+1), lit by
 *  its reach (amber alpha = reach, the `±` a thin halo), `?` past what the forecast knows; under it (from a 3rd row on, the reveal
 *  ladder) three gems — bank · return · death — with their shares and `~$N`. The shaft is one button: it opens the forecast panel
 *  (`onOpen`), where the bars, the causes and the `try` rows live. */
export function renderShaft(app: App, onOpen: () => void, showEnds: () => boolean): { el: HTMLElement; vsEl: HTMLElement; dispose(): void; paint(): void } {
  const notches = h("div", { class: "notches" });
  const ends = h("div", { class: "shaft-ends num", hidden: true });
  // Cut 22 §3: the last edit's paired move, a small line under the shaft (`vs last · D8 +6 · bank +4`), cleared by the next edit. It is
  // the camp's to place (`vsEl`): a row of its own under the well (QA 778fa1b, qaV: pinned over the well it hid R4 and the gems), so the
  // move is in view the moment it lands, whatever the well's scroll, and covers nothing. QA 778fa1b: measured against the set sent
  // (`vs sent`), `…` while an edit's move is pending
  const vsHost = h("div", { class: "shaft-vs-host", hidden: true });
  // Cut 28 §1: the sworn oath rides the shaft, under the notches, with the forecast's share of keeping it
  const oathEl = h("div", { class: "shaft-oath-host" });
  // docs/COPY.md pass 2 (the notches' % read as "success" or "clear rate"): the column says what its numbers are
  const el = h("button", { class: "shaft", onclick: () => onOpen() }, oathEl, h("small", { class: "shaft-head dim" }, kw("reach")), notches, ends);
  let last: Forecast | null = app.lastForecast;
  // notches shown at most: D1 … the deepest (best+1, or the bounty floor). QA e75ec29 (R: "the column starts at D7 but the run starts
  // on D1"): past MAX the shallow floors fold into one notch (`D1–6`, lit by its deepest floor's reach — they are the ones every run
  // passes), so the shaft always starts where the run does
  const MAX = 9;
  const paint = (): void => {
    // Cut 20 §5: the bounty floor (best + 2) carries a notch of its own past best + 1 — `D12 ×2`, a gold glint
    const bountyD = last?.depths.find((d) => d.bounty)?.depth ?? app.lineage.bounty?.depth;
    // Cut 21 §1: the shaft starts where the send does — D1, or the chosen waystone (`Lineage.start`); the floors above it are not run
    const start = forecastStart(app, last);
    // QA a946e04 (T: `depth ≥ 7 → bank` from a D5 start, the shaft stopped at D6 — no notch for the floor it banks on): the set's bank
    // floor gets its notch while the forecast knows it
    const capD = bankCap(app.rules.rows), known0 = last?.known_to ?? 0;
    const next = Math.max(start, app.lineage.best_depth + 1), deepest = Math.max(next, bountyD ?? 0, capD !== undefined && capD <= known0 + 1 ? capD : 0), from = deepest - start + 1 > MAX ? deepest - MAX + 2 : start;
    const byDepth = new Map((last?.depths ?? []).map((d) => [d.depth, d]));
    const vs = app.vsShown(), vsBy = new Map((vs?.depths ?? []).map((d) => [d.depth, d]));
    const known = last?.known_to ?? 0;
    const rough = last?.refined === false, cap = bankCap(app.rules.rows);
    const folded: HTMLElement[] = [];
    if (from > start) {
      const hi = from - 1, d = byDepth.get(hi), reach = d ? d.reach : hi <= known ? 1 : 0;
      // QA 778fa1b (qaV: an empty set's `D1–3 0%` — "D1 is 100%, every run starts there"): a fold whose floors differ reads its span,
      // the first floor's reach to the last's (`100–0%`)
      const d0 = byDepth.get(start), r0 = d0 ? d0.reach : 1, span = hi > start && Math.round(r0 * 100) !== Math.round(reach * 100);
      const n = h("span", { class: `notch fold${d && Math.round(d.reach * 100) === 0 ? " zero" : ""}`, "data-d": hi, "data-from": start },
        h("span", { class: "hex" }), h("span", { class: "dl" }, hi > start ? `D${start}–${hi}` : `D${hi}`), h("small", { class: "dp" }, d || hi <= known ? (span ? `${Math.round(r0 * 100)}–${share(reach, d ? lowOf(last) : undefined)}` : share(reach, d ? lowOf(last) : undefined)) : "?"));
      n.style.setProperty("--reach", reach.toFixed(3));
      folded.push(n);
    }
    // Cut 26 §2–3: once a fork was seen, a band's first notch names its lane on the set's route (`D5 fens`), and the lane the route does
    // not take at that fork hangs under it as a frontier (`crypt · D9 · ?`) — the next goal
    const chips = routeChips(app.rules, app.lineage), route = routeForks(app.rules);
    const front = new Map(frontiers(app.rules, app.lineage).map((f) => [f.fork, f]));
    // a band's first floor: a seen fork's depth (its lane the chip's), or where the forecast's own lane changes (`ForecastDepth.biome`)
    const laneAt = (depth: number): string | undefined => {
      if (!chips.length) return undefined;
      const c = chips.find((x) => x.depth === depth); if (c) return c.taken;
      const b = byDepth.get(depth)?.biome, up = byDepth.get(depth - 1)?.biome;
      if (b && up && b !== up) return b;
      return !b && route.length === 0 && BANDS.some(([a]) => a === depth) && chips.every((x) => (FORKS as readonly number[]).includes(x.depth)) ? biomeAt(route, depth) : undefined;
    };
    // QA 308f045 (qaAC: `D9 fens · sealed` under the route `D5 burrows`, read as the lane not picked — it is the next band): a lane's name
    // says where it runs to (`D9 fens → D13`), the band's last floor
    const laneEnd = (depth: number): number | undefined => BANDS.find(([a]) => a === depth)?.[1];
    replace(notches, ...folded, ...Array.from({ length: deepest - from + 1 }, (_, k) => {
      const depth = from + k, d = byDepth.get(depth);
      const lane = laneAt(depth), fr = front.get(depth);
      const reach = d ? d.reach : depth <= known ? 1 : 0;
      // Cut 18 §3: a walled floor's notch names the boss who seals it (`D9 · warlord`)
      const wall = d?.wall ? wallName(d.wall) : undefined;
      // Cut 24 §5: the floor he stands on names him too (`D8 · warlord`); his wall below keeps its own mark
      const bossHere = d?.boss && !wall ? wallName(d.boss) : undefined;
      // QA 0c6e126 (qaY: `D8 · warlord` over `D9 · warlord` — "two warlords"): under a notch that names him the wall reads `· wall`
      const wallText = wall ? /* copy:callout */ `behind ${wall}` : wall;   // docs/COPY.md pass 2 (`sealed` read as "D9 locked", no boss): who holds the stairs   // QA 524827b (qaAA: `D9 · wall <1%` unexplained): the boss above seals the stairs
      // QA 92eb880 (N: "D7 and D8 read 0% … the D8 label stays gold at 0%"): a notch nobody reaches is dim, label and all; a floor past
      // the set's own `depth ≥ N → bank` row is capped (dim), and the bank floor says so (`D6 · bank`)
      const zero = !!d && Math.round(d.reach * 100) === 0;
      const capped = cap !== undefined && depth > cap, bankHere = cap === depth && !wall;
      const bounty = depth === bountyD;
      const n = h("span", { class: `notch${!d && depth > known ? " unknown" : ""}${depth === next ? " next" : ""}${depth === start && start > 1 ? " start" : ""}${wall ? " walled" : ""}${zero ? " zero" : ""}${capped ? " capped" : ""}${bounty ? " bounty" : ""}`, "data-d": depth },
        h("span", { class: "hex" }), h("span", { class: "dl" }, `D${depth}`, lane ? h("i", { class: "lane", "data-biome": lane }, ` ${lane}`) : "", lane && laneEnd(depth) ? h("i", { class: "lane-to dim" }, ` → D${laneEnd(depth)}`) : "", bounty ? h("i", { class: "bounty-x" }, ` ${bountyMult(d?.bounty)}`, conceptCap("bounty")) : "", wall ? h("i", { class: "wall" }, /* copy:callout */ ` · ${wallText}`) : bankHere ? h("i", { class: "cap" }, /* copy:callout */ " · bank") : bossHere ? h("i", { class: "boss-here" }, ` · ${bossHere}`) : ""),   // (the set's own bank floor keeps its word)
        h("small", { class: "dp" }, d ? share(d.reach, lowOf(last)) : "?", d && pmShown(d.reach, d.pm) !== undefined ? h("i", { class: "pm band", style: bandW(pmShown(d.reach, d.pm)!), title: `±${pmShown(d.reach, d.pm)}` }, /* copy:none */ `±${pmShown(d.reach, d.pm)}${rough ? "…" : ""}`) : "",
          d ? moveMark(vsBy.get(depth)) : ""));   // Cut 22 §3: the edit's move on the notch (`▲6`, `≈`)
      n.style.setProperty("--reach", reach.toFixed(3));
      if (d?.pm !== undefined) n.style.setProperty("--pm", Math.min(1, d.pm * 4).toFixed(3));
      // QA 308f045 (qaAC: `fens · D5 · ?` — "what the `?` asks"): a lane never entered says so (`untried`)
      if (wall && d?.wall && sysOpen(app.lineage, "walls")) n.appendChild(h("small", { class: "wall-counter num" }, wallCounter(app, d.wall, d as { counter?: string; counter_hint?: string })));   // Cut 28 §1
      if (fr) n.appendChild(h("small", { class: `frontier${fr.entered ? " entered" : ""}`, "data-biome": fr.biome }, conceptIcon("fork"), /* copy:callout */ `or ${fr.biome}${fr.entered ? "" : " · untried"}`, conceptCap("fork")   /* docs/COPY.md pass 7: the other stair at this fork (`fens · D5 · untried` read "[elsewhere]" 2/2) */));
      return n;
    }),
    // QA 912e135 (qaW: the first camp's shaft was `D1 100%` alone, then D1–D7 after a death): the floors below the shaft's last are
    // there and unknown — a dim `D2+ ?` under it while the shaft is short
    deepest - from + 1 < MAX ? h("span", { class: "shaft-more num dim" }, `D${deepest + 1}+ ?`) : "");
    const e = last?.ends;
    ends.hidden = !e || !showEnds();
    if (e && !ends.hidden) replace(ends,
      // QA 778fa1b (qaU: the `▲`/`▼` after `return 92%` / `death 8%` clipped at the panel's edge): the arrow rides the number (`b`), raised
      // over its end, inside the column
      h("span", { class: "end bank" }, h("i", { class: "gemdot" }), /* copy:callout */ "full haul", " ", h("b", null, endShare(e.bank, lowOf(last)), moveMark(vs?.bank, true))),
      h("span", { class: "end return" }, h("i", { class: "gemdot" }), /* copy:callout */ "return", " ", h("b", null, endShare(e.return, lowOf(last)), moveMark(vs?.return, true))),
      // QA 23ed91f (L: "`bank 0% · return 0% · death 96%` never sums to 100; `stall` only in the panel"): a stall share is its own gem
      e.stall && Math.round(e.stall * 100) > 0 ? h("span", { class: "end stall" }, h("i", { class: "gemdot" }), /* copy:callout */ "stall", " ", h("b", null, pct(e.stall))) : "",
      h("span", { class: "end death" }, h("i", { class: "gemdot" }), /* copy:callout */ "death", " ", h("b", null, endShare(e.death, lowOf(last)), moveMark(vs?.death, true, true))),
      // QA 778fa1b: the first pass is marked on the gems too — `~$43…` until the refine lands
      h("span", { class: "end gold" }, /* copy:callout */ `avg $${Math.round(e.gold - (e.passage ?? 0))}/run`, passageEl(e.passage), rough ? h("i", { class: "settling" }, "…") : ""));
    replace(oathEl, shaftOath(app)); oathEl.hidden = !oathEl.childElementCount;
    const line = vsLine(app, vs, last, !!e && showEnds()), lm = lmoveLine(app, last), st = stateLine(app);
    vsHost.hidden = !line && !lm && !st; replace(vsHost, st ?? "", lm ?? "", line ?? "");
    el.dataset.vs = line ? "1" : "";
  };
  paint(); if (last) el.dataset.fc = String(app.forecastSeq);
  // QA 23ed91f (K: "the shaft moves with no edit … the ± only shows in the forecast sheet"): the first pass (`refined` false) paints
  // dim until the refine lands, and each notch carries its ± — a move inside it is the sims, not the last tap
  const off = app.onForecast(() => { const f = app.lastForecast; if (!f) return; last = f; el.dataset.fc = String(app.forecastSeq); el.classList.remove("stale"); el.classList.toggle("rough", f.refined === false); paint(); });
  const offRules = app.onRules(() => { el.classList.add("stale"); paint(); });   // the bank cap follows the rows at once
  const offChange = app.onChange(paint);
  const offVs = app.onVs(paint);   // Cut 22 §3: the move lands after the paint (or clears on an edit)
  return { el, vsEl: vsHost, paint, dispose: () => { off(); offRules(); offChange(); offVs(); } };
}
