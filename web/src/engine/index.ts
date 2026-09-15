// Engine selection: ?engine=fake forces the fake; otherwise wasm when pkg/ is built, else fake.
import type { Engine } from "./types";
import { createFakeEngine } from "./fake";
import { loadWasmEngine } from "./wasm";

export type EngineKind = "fake" | "wasm";

export async function selectEngine(): Promise<{ engine: Engine; kind: EngineKind; version: string }> {
  const want = new URLSearchParams(location.search).get("engine");
  if (want !== "fake") {
    const w = await loadWasmEngine();
    if (w) return { engine: w, kind: "wasm", version: w.version };
  }
  return { engine: createFakeEngine(), kind: "fake", version: "fake" };
}
