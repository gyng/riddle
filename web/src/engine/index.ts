// Engine selection: ?engine=fake forces the fake; otherwise the wasm engine in its worker when pkg/ is
// built, else the fake. Either way the app sees an AsyncEngine.
import type { AsyncEngine } from "./types";
import { createFakeEngine } from "./fake";
import { asyncify, workerEngine } from "./proxy";
import { lagged, twoLanes } from "./lanes";

export type EngineKind = "fake" | "wasm";

// Cut 20 §3: two lanes (engine/lanes.ts) — the slow read-only measures run on a mirror engine in a second worker, so an edit's
// forecast never queues behind them. Cut 24 §4: a third lane (a third worker) for the refine, latest only. Dev: `?lanes=0` keeps one lane (the old order); `?fake_lag=1` gives the fake a worker's timing.
export async function selectEngine(): Promise<{ engine: AsyncEngine; kind: EngineKind; version: string }> {
  const q = new URLSearchParams(location.search);
  const want = q.get("engine"), dev = import.meta.env.DEV;
  const oneLane = dev && q.get("lanes") === "0";
  const oneRefine = dev && q.get("lanes") === "2";   // Cut 24 §4: `?lanes=2` keeps the refine on the background lane (the Cut 20 order)
  if (want !== "fake") {
    const w = await workerEngine();
    if (w) return { engine: oneLane ? w.engine : twoLanes(w.engine, async () => (await workerEngine())?.engine ?? null, { mirror: true }, oneRefine ? undefined : async () => (await workerEngine())?.engine ?? null), kind: "wasm", version: w.version };
  }
  const fake = asyncify(createFakeEngine());
  if (dev && q.get("fake_lag") === "1") {
    const fg = lagged(fake);
    return { engine: oneLane ? fg : twoLanes(fg, async () => lagged(fake), { mirror: false }, oneRefine ? undefined : async () => lagged(fake)), kind: "fake", version: "fake" };
  }
  return { engine: fake, kind: "fake", version: "fake" };
}
