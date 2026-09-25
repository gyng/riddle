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
export const BACKGROUND = new Set<string>(["forecastRefine", "unlockDeltas", "cageForecast", "deathDeltas", "kitDeltas"]);
/** Foreground calls that leave the lineage as it was (the mirror stays in sync across them). */
const READ_ONLY = new Set<string>(["save", "vocabulary", "forecast", "forecastVs", "lineage", "exportRules", "importRules", "unlocks", "supplyCatalogue", "companionVocabulary"]);

type Calls = Record<string, (...a: unknown[]) => Promise<unknown>>;

export function twoLanes(fg: AsyncEngine, bgOf: () => Promise<AsyncEngine | null>, opts: { mirror: boolean }, refineOf?: () => Promise<AsyncEngine | null>): AsyncEngine {
  const F = fg as unknown as Calls;
  // `gen` counts the foreground's mutations; `fullGen` is the last that was not a `setRules` (Cut 24 §4: a mirror synced since then
  // takes the latest rules alone — an edit's refine starts at once, no 1 MB save through two workers)
  let gen = 0, fullGen = 0, lastRules: unknown[] | null = null;
  /** One mirrored engine and its sync state: a call on it reloads the foreground's save first when the lineage changed since. */
  const mirrorOf = (of: () => Promise<AsyncEngine | null>): ((m: string, a: unknown[]) => Promise<unknown>) => {
    let eng: Promise<AsyncEngine | null> | null = null, mirrorGen = -1;
    return async (m, a) => {
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
      const r = await b[m](...a);
      if (opts.mirror && m === "deathDeltas") mirrorGen = -1;   // `&mut` (it caches the verdict): the next sync reloads
      return r;
    };
  };
  const background = mirrorOf(bgOf);
  let chain: Promise<unknown> = Promise.resolve();   // the background lane's own order
  const onBackground = (m: string, a: unknown[]): Promise<unknown> => { const p = chain.then(() => background(m, a)); chain = p.catch(() => undefined); return p; };
  // Cut 24 §4: the refine lane — its own mirror, one call at a time, at most one waiting per call (the latest ask)
  const refineLane = refineOf ? latestOnly(mirrorOf(refineOf)) : null;
  const out: Calls = {};
  for (const m of Object.keys(F)) {
    if (refineLane && REFINE.has(m)) out[m] = (...a: unknown[]) => refineLane(m, a);
    else if (BACKGROUND.has(m)) out[m] = (...a: unknown[]) => onBackground(m, a);
    else out[m] = (...a: unknown[]) => { if (!READ_ONLY.has(m)) { gen++; if (m === "setRules") lastRules = a; else fullGen = gen; } return F[m](...a); };
  }
  // QA 778fa1b: the refine runs on its lane, so the refined camp panel is cached on that lane's engine — the edit's move asked again after
  // it (`forecastVsRefined`: `forecastVs` on the same lane, behind the refine) pairs the refined panels (`ForecastVs.refined`)
  if (typeof F.forecastVs === "function") out.forecastVsRefined = refineLane ? (...a: unknown[]) => refineLane("forecastVs", a, "forecastVsRefined") : (...a: unknown[]) => onBackground("forecastVs", a);
  if (refineLane) out.refineLane = true as unknown as Calls[string];   // the app starts an edit's refine beside its first pass
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
const LAG_MS: Record<string, number> = { forecast: 350, forecastRefine: 2500, unlockDeltas: 3000, cageForecast: 3000, deathDeltas: 3000, death: 500 };
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
