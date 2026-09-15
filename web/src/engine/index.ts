// Engine selection: ?engine=fake forces the fake; otherwise the wasm engine in its worker when pkg/ is
// built, else the fake. Either way the app sees an AsyncEngine.
import type { AsyncEngine } from "./types";
import { createFakeEngine } from "./fake";
import { asyncify, workerEngine } from "./proxy";

export type EngineKind = "fake" | "wasm";

export async function selectEngine(): Promise<{ engine: AsyncEngine; kind: EngineKind; version: string }> {
  const want = new URLSearchParams(location.search).get("engine");
  if (want !== "fake") {
    const w = await workerEngine();
    if (w) return { engine: w.engine, kind: "wasm", version: w.version };
  }
  return { engine: asyncify(createFakeEngine()), kind: "fake", version: "fake" };
}
