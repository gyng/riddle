// Cut 20 §3 — two lanes. The engine answers in order, one call at a time; the camp's slow measures (the refine's 100 sims, the
// unlock deltas, the cage options, a death's patch deltas: seconds each in wasm) queued ahead of the forecast an edit asked for,
// and the first paint waited 4–8 s behind them. The slow measures are read-only queries on the lineage (`&self`, or a cache
// the result does not depend on), so they run on a second engine — a mirror loaded from the first's `save()` whenever the
// first has changed since (bit-identical answers: same seed, same state) — and the foreground (`forecast`, `setRules`,
// `step`, …) never waits behind them. The fake shares one state and needs no mirror (`mirror: false`).
import type { AsyncEngine } from "./types";

// Cut 24 §4 (AK: "each edit makes you wait 3–7 s for the forecast to settle") — a third lane for the refine: behind the forge's and the
// unlocks' measures (3 s and 2 s each in wasm) on the one background lane, an edit's refine waited for them, and every edit in a burst
// queued a refine of its own. The refine (and the move paired with it, `forecastVsRefined`) runs on its own mirror, latest only: a
// refine asked while another waits to start takes its place (the superseded ask answers with the newer's result — the app drops it
// by its sequence), so a burst of edits costs at most the refine in flight plus the latest one.
/** The calls that run on the refine lane when there is one (latest only per call). */
export const REFINE = new Set<string>(["forecastRefine", "forecastVsRefined"]);
/** The calls that run on the background lane. */
export const BACKGROUND = new Set<string>(["forecastRefine", "unlockDeltas", "cageForecast", "deathDeltas", "kitDeltas", "startForecast", "forkForecast"]);
// Cut 25 §4 (AM: "~8 s for forge estimates on a D11 lineage after an absence"): measured on a D11 lineage after an 8 h absence (headed,
// real wasm) the forge's `kitDeltas` (~6 s there) queued behind the camp's `unlockDeltas` (~9.7 s) on the one background lane — 15 s
// from the camp's paint; `startForecast` ran on the foreground, ahead of an edit's forecast. The slow measures now run on lanes of their
// own (a mirror each), so no measure waits behind another's: the unlock shelf's, the forge's (and the start picker's), the rest (the cage,
// a death's patches). One lane on a machine with few cores (`MEASURE_LANES`).
const MEASURE_LANE: Record<string, number> = { unlockDeltas: 0, kitDeltas: 1, startForecast: 1, forkForecast: 1, cageForecast: 2, deathDeltas: 2, forecastRefine: 2, forecastVs: 2, divergence: 2 };
const MEASURE_LANES = typeof navigator !== "undefined" && (navigator.hardwareConcurrency ?? 4) >= 6 ? 3 : 1;
/** Foreground calls that leave the lineage as it was (the mirror stays in sync across them). */
const READ_ONLY = new Set<string>(["save", "vocabulary", "forecast", "forecastVs", "lineage", "exportRules", "importRules", "unlocks", "supplyCatalogue", "companionVocabulary"]);

type Calls = Record<string, (...a: unknown[]) => Promise<unknown>>;

export function twoLanes(fg: AsyncEngine, bgOf: () => Promise<AsyncEngine | null>, opts: { mirror: boolean }, refineOf?: () => Promise<AsyncEngine | null>): AsyncEngine {
  const F = fg as unknown as Calls;
  // `gen` counts the foreground's mutations; `fullGen` is the last that was not a `setRules` (Cut 24 §4: a mirror synced since then
  // takes the latest rules alone — an edit's refine starts at once, no 1 MB save through two workers)
  let gen = 0, fullGen = 0, lastRules: unknown[] | null = null;
  /** One mirrored engine and its sync state: a call on it reloads the foreground's save first when the lineage changed since. */
  type Mirror = ((m: string, a: unknown[]) => Promise<unknown>) & { cheap?: () => boolean };
  const mirrorOf = (of: () => Promise<AsyncEngine | null>): Mirror => {
    let eng: Promise<AsyncEngine | null> | null = null, mirrorGen = -1, ready = false;
    const run: Mirror = async (m, a) => {
      eng ??= of().catch(() => null);
      const b = (await eng) as unknown as Calls | null;
      if (!b || typeof b[m] !== "function") return F[m](...a);   // no second engine: the one lane
      if (opts.mirror && mirrorGen !== gen) {
        const at = gen, rules = lastRules;
        const reload = async (): Promise<void> => { const save = (await F.save()) as string; await b.load(save); };
        if (mirrorGen >= 0 && mirrorGen >= fullGen && rules && typeof b.setRules === "function") { try { await b.setRules(...rules); } catch { await reload(); } }
        else await reload();
        mirrorGen = at;
      }
      ready = true;
      const r = await b[m](...a);
      if (opts.mirror && m === "deathDeltas") mirrorGen = -1;   // `&mut` (it caches the verdict): the next sync reloads
      return r;
    };
    /** Cut 25 §4: up and in step but for the rules (its sync is a `setRules`, never a save through the foreground). */
    run.cheap = () => ready && mirrorGen >= 0 && mirrorGen >= fullGen && !!lastRules;
    return run;
  };
  // Cut 25 §4: a mirror and an order per measure lane, made on first use
  // (a measure whose own lane is busy — the forge's, behind a measure of the rules before an edit — takes an idle lane when there is one:
  // the forge opened just after an edit waited ~5 s behind the stale one)
  const lanes: { run: Mirror; chain: Promise<unknown>; busy: number }[] = [];
  const laneAt = (i: number) => lanes[i] ??= { run: mirrorOf(bgOf), chain: Promise.resolve(), busy: 0 };
  const onBackground = (m: string, a: unknown[]): Promise<unknown> => {
    const own = Math.min(MEASURE_LANES - 1, MEASURE_LANE[m] ?? 0);
    let lane = laneAt(own);
    if (lane.busy && m !== "deathDeltas") for (let i = 0; i < MEASURE_LANES; i++) { const l = laneAt(i); if (!l.busy) { lane = l; break; } }   // (a death's patches stay on their lane: its cache)
    const L = lane; L.busy++;
    const p = L.chain.then(() => L.run(m, a)); L.chain = p.catch(() => undefined).finally(() => { L.busy--; }); return p;
  };
  // Cut 24 §4: the refine lane — its own mirror, one call at a time, at most one waiting per call (the latest ask)
  const refineLane = refineOf ? latestOnly(mirrorOf(refineOf)) : null;
  const out: Calls = {};
  // Cut 25 §4 (clarity: an edit 250 ms after another painted in 1.5 s — its forecast queued behind the stale one on the foreground): while
  // the foreground is busy, an edit's `forecast` runs on an idle measure lane already in step but for the rules (bit-identical answers)
  let fgBusy = 0;
  const onFg = (m: string, a: unknown[]): Promise<unknown> => {
    if (!READ_ONLY.has(m)) { gen++; if (m === "setRules") lastRules = a; else fullGen = gen; }
    fgBusy++; const p = F[m](...a); void p.catch(() => undefined).finally(() => { fgBusy--; }); return p;
  };
  for (const m of Object.keys(F)) {
    if (refineLane && REFINE.has(m)) out[m] = (...a: unknown[]) => refineLane(m, a);
    else if (BACKGROUND.has(m)) out[m] = (...a: unknown[]) => onBackground(m, a);
    else if (m === "forecast" && opts.mirror) out[m] = (...a: unknown[]) => {
      const L = fgBusy > 0 ? lanes.find((l) => l && !l.busy && l.run.cheap?.()) : undefined;
      if (!L) return onFg(m, a);
      L.busy++; const p = L.chain.then(() => L.run(m, a)); L.chain = p.catch(() => undefined).finally(() => { L.busy--; }); return p;
    };
    else out[m] = (...a: unknown[]) => onFg(m, a);
  }
  // QA 778fa1b: the refine runs on its lane, so the refined camp panel is cached on that lane's engine — the edit's move asked again after
  // it (`forecastVsRefined`: `forecastVs` on the same lane, behind the refine) pairs the refined panels (`ForecastVs.refined`)
  if (typeof F.forecastVs === "function") out.forecastVsRefined = refineLane ? (...a: unknown[]) => refineLane("forecastVs", a, "forecastVsRefined") : (...a: unknown[]) => onBackground("forecastVs", a);
  // Cut 27 §2: the edit's scene reads the paired panels the refine just ran — on the refine's lane, behind it (latest only), else a measure lane
  if (typeof F.divergence === "function") out.divergence = refineLane ? (...a: unknown[]) => refineLane("divergence", a) : (...a: unknown[]) => onBackground("divergence", a);
  if (refineLane) out.refineLane = true as unknown as Calls[string];
  if (opts.mirror) out.parallelForecast = true as unknown as Calls[string];   // Cut 25 §4   // the app starts an edit's refine beside its first pass
  return out as unknown as AsyncEngine;
}

/** Cut 24 §4: a lane that runs one call at a time and keeps at most one ask waiting per key (the latest; the superseded asks answer
 *  with its result). `run(m, a)` does the call; `key` names the slot (default `m`). */
export function latestOnly(run: (m: string, a: unknown[]) => Promise<unknown>): (m: string, a: unknown[], key?: string) => Promise<unknown> {
  type Job = { m: string; a: unknown[]; waiters: { res: (r: unknown) => void; rej: (e: unknown) => void }[] };
  const waiting = new Map<string, Job>();   // insertion order is the lane's order; a superseding ask keeps its slot's place
  let busy = false;
  const pump = async (): Promise<void> => {
    if (busy) return;
    const next = waiting.entries().next();
    if (next.done) return;
    const [k, job] = next.value; waiting.delete(k);
    busy = true;
    try { const r = await run(job.m, job.a); for (const w of job.waiters) w.res(r); } catch (e) { for (const w of job.waiters) w.rej(e); }
    busy = false;
    void pump();
  };
  return (m, a, key = m) => new Promise((res, rej) => {
    const was = waiting.get(key);
    waiting.set(key, { m, a, waiters: [...(was?.waiters ?? []), { res, rej }] });
    void pump();
  });
}

/** Dev (`?engine=fake&fake_lag=1`): the fake behind a worker's timing — one lane's calls in order, each taking the wasm's
 *  order of magnitude for its kind, so the clarity gate can measure an edit's first paint against the slow measures. */
const LAG_MS: Record<string, number> = { forecast: 350, forecastRefine: 2500, unlockDeltas: 3000, cageForecast: 3000, deathDeltas: 3000, kitDeltas: 3000, death: 500 };   // Cut 25 §4: the forge's measure too
export function lagged(e: AsyncEngine, lag: Record<string, number> = LAG_MS): AsyncEngine {
  const E = e as unknown as Calls;
  let chain: Promise<unknown> = Promise.resolve();
  const out: Calls = {};
  for (const m of Object.keys(E)) {
    out[m] = (...a: unknown[]) => {
      const p = chain.then(() => new Promise((res) => setTimeout(res, lag[m] ?? 0))).then(() => E[m](...a));
      chain = p.catch(() => undefined);
      return p;
    };
  }
  return out as unknown as AsyncEngine;
}
