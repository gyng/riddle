// Cut 20 §3 — two lanes. The engine answers in order, one call at a time; the camp's slow measures (the refine's 100 sims, the
// unlock deltas, the cage options, a death's patch deltas: seconds each in wasm) queued ahead of the forecast an edit asked for,
// and the first paint waited 4–8 s behind them. The slow measures are read-only queries on the lineage (`&self`, or a cache
// the result does not depend on), so they run on a second engine — a mirror loaded from the first's `save()` whenever the
// first has changed since (bit-identical answers: same seed, same state) — and the foreground (`forecast`, `setRules`,
// `step`, …) never waits behind them. The fake shares one state and needs no mirror (`mirror: false`).
import type { AsyncEngine } from "./types";

/** The calls that run on the background lane. */
export const BACKGROUND = new Set<string>(["forecastRefine", "unlockDeltas", "cageForecast", "deathDeltas"]);
/** Foreground calls that leave the lineage as it was (the mirror stays in sync across them). */
const READ_ONLY = new Set<string>(["save", "vocabulary", "forecast", "lineage", "exportRules", "importRules", "unlocks", "supplyCatalogue", "companionVocabulary"]);

type Calls = Record<string, (...a: unknown[]) => Promise<unknown>>;

export function twoLanes(fg: AsyncEngine, bgOf: () => Promise<AsyncEngine | null>, opts: { mirror: boolean }): AsyncEngine {
  const F = fg as unknown as Calls;
  let gen = 0, mirrorGen = -1;
  let bg: Promise<AsyncEngine | null> | null = null;
  let chain: Promise<unknown> = Promise.resolve();   // the background lane's own order
  const background = async (m: string, a: unknown[]): Promise<unknown> => {
    bg ??= bgOf().catch(() => null);
    const b = (await bg) as unknown as Calls | null;
    if (!b || typeof b[m] !== "function") return F[m](...a);   // no second engine: the one lane
    if (opts.mirror && mirrorGen !== gen) {
      const at = gen;
      const save = (await F.save()) as string;
      await b.load(save);
      mirrorGen = at;
    }
    const r = await b[m](...a);
    if (opts.mirror && m === "deathDeltas") mirrorGen = -1;   // `&mut` (it caches the verdict): the next sync reloads
    return r;
  };
  const out: Calls = {};
  for (const m of Object.keys(F)) {
    if (BACKGROUND.has(m)) {
      out[m] = (...a: unknown[]) => { const p = chain.then(() => background(m, a)); chain = p.catch(() => undefined); return p; };
    } else {
      out[m] = (...a: unknown[]) => { if (!READ_ONLY.has(m)) gen++; return F[m](...a); };
    }
  }
  return out as unknown as AsyncEngine;
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
