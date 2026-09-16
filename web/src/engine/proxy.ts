// AsyncEngine proxies: the wasm engine behind a Web Worker (calls are posted and answered in order),
// and the fake engine wrapped so ?engine=fake keeps the same async surface.
import type { AsyncEngine, Engine } from "./types";
import type { Req, Res } from "./worker";

const METHODS: (keyof Engine)[] = [
  "newLineage", "load", "save", "vocabulary", "setRules", "loadout", "forecast", "send", "step", "runOffline", "death",
  "buy", "lineage", "exportRules", "importRules", "setParty", "setCompanionRules", "breed", "hatch", "companionVocabulary",
  "buySupply", "clearSupplies", "supplyCatalogue", "keep", "setKeepPref", "insure", "runOfflineQuick", "unlocks", "unlockDeltas", "setClass", "selectSet", "ascend",
];

/** Wraps a synchronous Engine (the fake) so every call resolves on a microtask. */
export function asyncify(e: Engine): AsyncEngine {
  const out: Record<string, (...a: unknown[]) => Promise<unknown>> = {};
  for (const m of METHODS) {
    out[m] = (...a: unknown[]) => new Promise((res, rej) => {
      try { res((e[m] as (...x: unknown[]) => unknown).apply(e, a)); } catch (err) { rej(err); }
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
