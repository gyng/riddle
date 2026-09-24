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
import type { App } from "../app";
import type { Forecast, ForecastTry, Row } from "../engine/types";
import { h, clear, pct, replace } from "./dom";
import { closeAllSheets } from "./sheet";
import { pickedLine } from "./report";

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

/** Cut 18 §3: the sealing boss's kind as one word (`goblin_warlord` → `warlord`). */
export const wallName = (kind: string): string => kind.replace(/_/g, " ").trim().split(/\s+/).pop() ?? kind;

/** Cut 20 §5: the bounty's multiplier as the notch reads it (`×2`; a number on the wire above 1 is the multiplier). */
export const bountyMult = (b: boolean | number | undefined): string => `×${typeof b === "number" && b > 1 ? b : 2}`;

/** Cut 9 §3: the half-width (a 0..1 fraction like `reach`) in percentage points, never `±0` — a forecast is never exact. */
export const pmPts = (pm: number): number => Math.max(1, Math.round(pm * 100));
/** QA e75ec29 (Q: `D1 100% ±1`, a bound above 100 %): the ± a share shows — none where it reads 0 % or 100 %, and never so wide that
 *  it would cross either end (`99% ±1`, not `99% ±3`). Undefined when there is nothing to show. */
export const pmShown = (share: number, pm: number | undefined): number | undefined => {
  if (pm === undefined) return undefined;
  const r = Math.round(share * 100);
  return r <= 0 || r >= 100 ? undefined : Math.min(pmPts(pm), 100 - r, r);
};

export function renderForecast(app: App): { el: HTMLElement; dispose(): void } {
  const bars = h("div", { class: "fc-bars" });
  const causes = h("div", { class: "fc-causes" });
  const yours = h("div", { class: "fc-yours num" });
  const ends = h("div", { class: "fc-ends num dim", hidden: true });
  // Cut 16 §1: the depths picked clean (`Lineage.picked`), small and dim under the ends line — why the `~$N` is lower than it was
  const picked = h("div", { class: "fc-picked num dim", hidden: true });
  const paintPicked = (): void => { const p = app.lineage.picked ?? []; picked.hidden = !p.length; replace(picked, p.length ? pickedLine(p) : ""); };
  const el = h("section", { class: "forecast" }, h("div", { class: "label" }, /* copy:label */ "forecast"), bars, ends, picked, yours, causes);
  // Cut 8B §4: `· 1 combo` when the set has one (engine data; the count is the client's mirror of `Lineage.combos`)
  // Cut 12 §6: the combo's name (engine data: `Vocabulary.combos[].name`), not `1 combo`
  const paintYours = (): void => {
    const n = app.playerRows(), m = app.ownRows(), combos = app.combos();
    replace(yours, h("span", { class: n ? "" : "dim" }, /* copy:callout */ `yours: ${n} of ${m} row${m === 1 ? "" : "s"}`),
      combos.length ? h("span", { class: "combos" }, ` · ${combos.map((c) => c.name).join(" · ")}`) : "");
  };
  // Cut 12 §3: `bank 40% · return 35% · death 25% · ~$54` — the three shown always so the trade reads; absent on an older core
  const paintEnds = (f: Forecast): void => {
    const e = f.ends;
    ends.hidden = !e;
    if (!e) return;
    // a stall share only when there is one: `bank 0% · return 20% · stall 50% · death 30% · ~$25`
    const stall = e.stall && Math.round(e.stall * 100) > 0 ? /* copy:callout */ ` · stall ${pct(e.stall)}` : "";
    const epm = pmShown(e.death, e.pm);
    const pm = epm !== undefined ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${epm}${f.refined === false ? "…" : ""}`) : "";
    // QA 1a2a4a9 (O: `D5 76%` beside `death 100%` read as a contradiction): the split is labelled — how a run ends, not how deep
    replace(ends, h("span", { class: "label ends-label" }, /* copy:label */ "ends"), " ", /* copy:callout */ `bank ${pct(e.bank)} · return ${pct(e.return)}`, stall, /* copy:callout */ ` · death ${pct(e.death)}`, pm, h("span", { class: "gold" }, ` · ~$${Math.round(e.gold)}`));
  };
  /** The named counter of a boss cause (`goblin_warlord`, `goblin warlord pack`) from `lineage.counters`. */
  const counterFor = (cause: string): string | undefined => {
    const key = cause.replace(/ pack$/, "").trim().replace(/ /g, "_");
    return (app.lineage.counters ?? []).find((c) => c.boss === key || key.endsWith(c.boss))?.text;
  };
  const paint = (f: Forecast): void => {
    clear(bars); clear(causes); paintEnds(f); paintPicked();
    el.dataset.refined = f.refined === undefined ? "" : f.refined ? "1" : "0";   // dev: tools read which pass painted
    // QA 92eb880 (M: "D6 32%±13 → 38%±10 on opening edit"): the first pass paints dim, its ± trailing `…`, until the refine lands
    el.classList.toggle("rough", f.refined === false);
    const first = f.refined === false ? "…" : "";   // Cut 13 §5: the first paint's ± trails `…`; the refine's does not
    const next = app.lineage.best_depth + 1;
    // Cut 4 §8 names the top cause on best+1; QA 23ed91f (L: `D9 0% ±1 · rat` for a set that dies on D1–D2): when nobody gets near
    // best+1, the cause sits on the floor where the reach falls most
    let causeAt = next;
    { const byD = new Map(f.depths.map((d) => [d.depth, d.reach])); if ((byD.get(next - 1) ?? 1) < 0.05) {
      let drop = -1; for (const d of f.depths) { const fall = (byD.get(d.depth - 1) ?? 1) - d.reach; if (fall > drop) { drop = fall; causeAt = d.depth; } } } }
    for (const d of f.depths) {
      // Cut 18 §3: a floor the boss above seals (`ForecastDepth.wall`) names him as the cause: `D9 0% · warlord wall`
      const wall = d.wall ? wallName(d.wall) : undefined;
      const cause = wall ? undefined : d.cause ?? (d.depth === causeAt ? f.causes[0]?.cause : undefined);
      // Cut 10 §2: the known-but-absent counter — the wire's `try`, else the client's read of the counters against the set
      const tr = d.try ?? (d.depth === next ? clientTry(app, cause) : undefined);
      // Cut 6 §5: `D6 0% · goblin warlord · counter: attack boss` when the top cause is a boss whose counter row is known (and held)
      const counter = cause && d.depth === next && !tr ? counterFor(cause) : undefined;
      // the try hint rides the track, on the depth's own line, the track as wide as every other row's (QA on 50bb162: `D9 0% ±1`
      // wrapped `· try: attack boss` under it with a shorter bar)
      const track = h("span", { class: "track" }, h("span", { class: "fill", style: `width:${Math.round(d.reach * 100)}%` }));
      // QA e75ec29 (Q: "the D1 bar at 100 % is shorter than the grey D2+ track"): every row's track is one width — the cause, the wall,
      // the bounty and the counter go on a line of their own under the track (`.why`), never into the number's column
      const dpm = pmShown(d.reach, d.pm);
      const why = [
        cause ? h("small", { class: "dim" }, ` · ${cause.replace(/_/g, " ")}`) : "",
        wall ? h("small", { class: "wall" }, /* copy:callout */ ` · ${wall} wall`) : "",
        d.bounty ? h("small", { class: "bounty-x" }, ` · ${bountyMult(d.bounty)}`) : "",   // Cut 20 §5: the bounty floor
        counter ? h("small", { class: "dim" }, /* copy:callout */ ` · counter: ${counter}`) : "",
      ].filter((x) => x !== "");
      const inner = [
        h("span", { class: "d num" }, `D${d.depth}`),
        tr ? h("span", { class: "track-cell" }, track, h("small", { class: "try" }, /* copy:none */ `try: ${tr.text}`)) : track,
        // a `try` row keeps one line (its hint rides the track; the boss beside the number, as before)
        h("span", { class: "n num" }, pct(d.reach), dpm !== undefined ? h("small", { class: "dim pm" }, /* copy:none */ ` ±${dpm}${first}`) : "", ...(tr ? why : [])),
        !tr && why.length ? h("span", { class: "why num" }, ...why) : "",
      ];
      // the `try` bar is a button: the row goes in at the top (position is the point), the camp opens on it
      bars.appendChild(tr
        ? h("button", { class: `bar next try${wall ? " walled" : ""}`, onclick: () => { const i = app.applyPatch({ row: tr.row, insert_at: 0, survive: 0, forecast_delta: 0 }); closeAllSheets(); app.go({ kind: "camp", highlight: i }); } }, ...inner)
        : h("div", { class: `bar${cause || wall ? " next" : ""}${wall ? " walled" : ""}` }, ...inner));
    }
    bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, `D${f.known_to + 1}+`), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
    // QA 23ed91f (K: "`jackal 100%` beside `death 1%` — I read it as jackal kills 100%"): a cause's share of the deaths is shown as its
    // share of the sends when the ends are known (`jackal 1%` under `death 1%`), so the two lines speak one unit
    const per = f.ends ? f.ends.death : 1;
    // QA 1a2a4a9 (O: `goblin 26% · ogre 21%` "with no heading"): the killers' line says what it lists
    if (f.causes.length) causes.appendChild(h("span", { class: "label causes-label" }, /* copy:label */ "killers"));
    for (const c of f.causes) causes.appendChild(h("span", { class: "cause" }, c.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, pct(c.share * per))));
  };
  // until the first forecast arrives (≈1 s in the worker): the unknown row only
  bars.appendChild(h("div", { class: "bar unknown" }, h("span", { class: "d num" }, "…"), h("span", { class: "track" }), h("span", { class: "n" }, "?")));
  // a rule edit (or a set switch) dims the numbers until the engine's next forecast paints — an empty set's takes seconds and
  // the old set's bars read as the new one's meanwhile (QA on 952e306: "set '2 0' showed set 1's D4 72%")
  const stale = (): void => { paintYours(); el.classList.add("stale"); };
  const fresh = (f: Forecast): void => { el.classList.remove("stale"); paint(f); };
  const off = app.onForecast(fresh), offRules = app.onRules(stale), offChange = app.onChange(paintYours);
  paintYours(); paintPicked();
  // the first forecast posts after the camp's own fetches (the worker answers in order: a forecast posted first held the
  // supply shop and the unlock shelf behind it — QA B on 952e306: "while FORECAST shows '…' the shop chips and UNLOCKS are gone")
  setTimeout(() => void app.emitForecast(), 0);
  return { el, dispose: () => { off(); offRules(); offChange(); } };
}

/** QA 92eb880: the depth a lone `depth ≥ N → bank` row sends the hero home from (the smallest such N); undefined without one. */
export function bankCap(rows: Row[]): number | undefined {
  const ns = rows.filter((r) => r.verb.v === "bank" && r.conds.length === 1 && r.conds[0].k === "depth>=" && r.conds[0].n !== undefined).map((r) => r.conds[0].n!);
  return ns.length ? Math.min(...ns) : undefined;
}

/** Cut 17 §2 — the depth shaft at the camp well's right edge (docs/UI.md §2's minimap): one notch per depth D1 … D(best+1), lit by
 *  its reach (amber alpha = reach, the `±` a thin halo), `?` past what the forecast knows; under it (from a 3rd row on, the reveal
 *  ladder) three gems — bank · return · death — with their shares and `~$N`. The shaft is one button: it opens the forecast panel
 *  (`onOpen`), where the bars, the causes and the `try` rows live. */
export function renderShaft(app: App, onOpen: () => void, showEnds: () => boolean): { el: HTMLElement; dispose(): void; paint(): void } {
  const notches = h("div", { class: "notches" });
  const ends = h("div", { class: "shaft-ends num", hidden: true });
  const el = h("button", { class: "shaft", onclick: () => onOpen() }, notches, ends);
  let last: Forecast | null = app.lastForecast;
  // notches shown at most: D1 … the deepest (best+1, or the bounty floor). QA e75ec29 (R: "the column starts at D7 but the run starts
  // on D1"): past MAX the shallow floors fold into one notch (`D1–6`, lit by its deepest floor's reach — they are the ones every run
  // passes), so the shaft always starts where the run does
  const MAX = 9;
  const paint = (): void => {
    // Cut 20 §5: the bounty floor (best + 2) carries a notch of its own past best + 1 — `D12 ×2`, a gold glint
    const bountyD = last?.depths.find((d) => d.bounty)?.depth ?? app.lineage.bounty?.depth;
    const next = app.lineage.best_depth + 1, deepest = Math.max(next, bountyD ?? 0), from = deepest > MAX ? deepest - MAX + 2 : 1;
    const byDepth = new Map((last?.depths ?? []).map((d) => [d.depth, d]));
    const known = last?.known_to ?? 0;
    const rough = last?.refined === false, cap = bankCap(app.rules.rows);
    const folded: HTMLElement[] = [];
    if (from > 1) {
      const hi = from - 1, d = byDepth.get(hi), reach = d ? d.reach : hi <= known ? 1 : 0;
      const n = h("span", { class: `notch fold${d && Math.round(d.reach * 100) === 0 ? " zero" : ""}`, "data-d": hi, "data-from": 1 },
        h("span", { class: "hex" }), h("span", { class: "dl" }, `D1–${hi}`), h("small", { class: "dp" }, d || hi <= known ? pct(reach) : "?"));
      n.style.setProperty("--reach", reach.toFixed(3));
      folded.push(n);
    }
    replace(notches, ...folded, ...Array.from({ length: deepest - from + 1 }, (_, k) => {
      const depth = from + k, d = byDepth.get(depth);
      const reach = d ? d.reach : depth <= known ? 1 : 0;
      // Cut 18 §3: a walled floor's notch names the boss who seals it (`D9 · warlord`)
      const wall = d?.wall ? wallName(d.wall) : undefined;
      // QA 92eb880 (N: "D7 and D8 read 0% … the D8 label stays gold at 0%"): a notch nobody reaches is dim, label and all; a floor past
      // the set's own `depth ≥ N → bank` row is capped (dim), and the bank floor says so (`D6 · bank`)
      const zero = !!d && Math.round(d.reach * 100) === 0;
      const capped = cap !== undefined && depth > cap, bankHere = cap === depth && !wall;
      const bounty = depth === bountyD;
      const n = h("span", { class: `notch${!d && depth > known ? " unknown" : ""}${depth === next ? " next" : ""}${wall ? " walled" : ""}${zero ? " zero" : ""}${capped ? " capped" : ""}${bounty ? " bounty" : ""}`, "data-d": depth },
        h("span", { class: "hex" }), h("span", { class: "dl" }, `D${depth}`, bounty ? h("i", { class: "bounty-x" }, ` ${bountyMult(d?.bounty)}`) : "", wall ? h("i", { class: "wall" }, /* copy:callout */ ` · ${wall}`) : bankHere ? h("i", { class: "cap" }, /* copy:callout */ " · bank") : ""),
        h("small", { class: "dp" }, d ? pct(d.reach) : "?", d && pmShown(d.reach, d.pm) !== undefined ? h("i", { class: "pm" }, /* copy:none */ `±${pmShown(d.reach, d.pm)}${rough ? "…" : ""}`) : ""));
      n.style.setProperty("--reach", reach.toFixed(3));
      if (d?.pm !== undefined) n.style.setProperty("--pm", Math.min(1, d.pm * 4).toFixed(3));
      return n;
    }));
    const e = last?.ends;
    ends.hidden = !e || !showEnds();
    if (e && !ends.hidden) replace(ends,
      h("span", { class: "end bank" }, h("i", { class: "gemdot" }), /* copy:callout */ "bank", " ", h("b", null, pct(e.bank))),
      h("span", { class: "end return" }, h("i", { class: "gemdot" }), /* copy:callout */ "return", " ", h("b", null, pct(e.return))),
      // QA 23ed91f (L: "`bank 0% · return 0% · death 96%` never sums to 100; `stall` only in the panel"): a stall share is its own gem
      e.stall && Math.round(e.stall * 100) > 0 ? h("span", { class: "end stall" }, h("i", { class: "gemdot" }), /* copy:callout */ "stall", " ", h("b", null, pct(e.stall))) : "",
      h("span", { class: "end death" }, h("i", { class: "gemdot" }), /* copy:callout */ "death", " ", h("b", null, pct(e.death))),
      h("span", { class: "end gold" }, `~$${Math.round(e.gold)}`));
  };
  paint();
  // QA 23ed91f (K: "the shaft moves with no edit … the ± only shows in the forecast sheet"): the first pass (`refined` false) paints
  // dim until the refine lands, and each notch carries its ± — a move inside it is the sims, not the last tap
  const off = app.onForecast((f) => { last = f; el.classList.remove("stale"); el.classList.toggle("rough", f.refined === false); paint(); });
  const offRules = app.onRules(() => { el.classList.add("stale"); paint(); });   // the bank cap follows the rows at once
  const offChange = app.onChange(paint);
  return { el, paint, dispose: () => { off(); offRules(); offChange(); } };
}
