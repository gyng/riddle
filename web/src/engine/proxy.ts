// AsyncEngine proxies: the wasm engine behind a Web Worker (calls are posted and answered in order),
// and the fake engine wrapped so ?engine=fake keeps the same async surface.
import type { AsyncEngine, Engine } from "./types";
import type { Req, Res } from "./worker";

const METHODS: (keyof Engine)[] = [
  "newLineage", "load", "save", "vocabulary", "setRules", "loadout", "forecast", "send", "step", "runOffline", "death",
  "buy", "lineage", "exportRules", "importRules", "setParty", "setCompanionRules", "breed", "hatch", "companionVocabulary",
  "buySupply", "clearSupplies", "supplyCatalogue", "keep", "setKeepPref", "insure", "sellVault", "runOfflineQuick", "unlocks", "unlockDeltas", "setClass", "selectSet", "ascend",
  "bail", "choose", "setVaultPref",
  "forecastRefine",   // Cut 6: optional on the engine; the proxy rejects when the engine lacks it (the client treats that as "no refine")
  "dropSupply",       // Cut 12 §6: optional likewise (the client falls back to clear + rebuy)
  "setTrait",         // Cut 13 §2: optional likewise (no offer without it)
  "setLook",          // hero looks: optional likewise (the class's own look without it)
  "buyUnlockGold",    // Cut 15 §2: optional likewise (the sheet's `$ buy` is off without a gold price)
  "autoKeep",         // QA 23ed91f: optional likewise (the skipped keep sheet's preference keep)
  "deathDeltas",      // QA 23ed91f: optional likewise (the camp's reach for a death's patches, after `death`)
  "cageForecast",     // Cut 19 §1: optional likewise (the cage tablet's per-option deltas)
  "setRestock",       // Cut 19 §3: optional likewise (the loadout's repeat toggle)
  "setStart",         // Cut 21 §1: optional likewise (the start tablet)
  "startForecast",    // Cut 21 §1: optional likewise (the start picker's per-start moves)
  "forecastVs",       // Cut 22 §3: optional likewise (the edit's paired move under the shaft)
  "buyKit",           // Cut 23 §1: optional likewise (the forge)
  "kitDeltas",        // Cut 23 §1: optional likewise (the forge steps' measured moves)
  "forkForecast",     // Cut 26 §2: optional likewise (the fork chip's two stairs)
  "fold",             // Cut 27 §1: optional likewise (the watch steps the folded floors itself without it)
  "divergence",       // Cut 27 §2: optional likewise (no scene without it)
  "runOfflineSlice",  // round 3: optional likewise (`runOfflineQuick` for every slice without it)
  "forecastMove",     // Cut 28 §2: optional likewise (the move against the sent set attributed to state and rows)
  "swearOath",        // Cut 28 §1: optional likewise (no oath board without it)
  "forswearOath",     // Cut 28 §1: optional likewise
];

/** Wraps a synchronous Engine (the fake) so every call resolves on a microtask. */
export function asyncify(e: Engine): AsyncEngine {
  const out: Record<string, (...a: unknown[]) => Promise<unknown>> = {};
  for (const m of METHODS) {
    out[m] = (...a: unknown[]) => new Promise((res, rej) => {
      const fn = e[m] as ((...x: unknown[]) => unknown) | undefined;
      if (typeof fn !== "function") { rej(new Error(`no engine method ${m}`)); return; }
      try { res(fn.apply(e, a)); } catch (err) { rej(err); }
    });
  }
  return out as unknown as AsyncEngine;
}

/** Starts the engine worker; resolves to the proxy and the wasm version, or null when pkg/ is absent. */
export async function workerEngine(): Promise<{ engine: AsyncEngine; version: string } | null> {
  let worker: Worker;
  try { worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" }); } catch { return null; }
  const pending = new Map<number, { res: (r: unknown) => void; rej: (e: Error) => void }>();
  let next = 1;
  worker.onmessage = (ev: MessageEvent<Res>) => {
    const p = pending.get(ev.data.id);
    if (!p) return;
    pending.delete(ev.data.id);
    if (ev.data.ok) p.res(ev.data.r); else p.rej(new Error(ev.data.e));
  };
  worker.onerror = (ev) => { for (const p of pending.values()) p.rej(new Error(ev.message || "engine worker failed")); pending.clear(); };
  const call = (m: string, a: unknown[]): Promise<unknown> => new Promise((res, rej) => {
    const id = next++;
    pending.set(id, { res, rej });
    worker.postMessage({ id, m, a } satisfies Req);
  });
  let version: unknown;
  try { version = await call("init", []); } catch { version = null; }
  if (typeof version !== "string") { worker.terminate(); return null; }
  const out: Record<string, (...a: unknown[]) => Promise<unknown>> = {};
  for (const m of METHODS) out[m] = (...a: unknown[]) => call(m, a);
  return { engine: out as unknown as AsyncEngine, version };
}
