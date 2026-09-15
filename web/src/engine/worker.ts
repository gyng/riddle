// Web Worker hosting the wasm engine. One message per Engine call, answered in order (the engine is
// synchronous, so calls never interleave). Protocol: {id, m, a} → {id, ok:true, r} | {id, ok:false, e}.
// The first message must be {id, m:"init"}; it answers with the version string or null when pkg/
// is absent (the main thread then falls back to the fake engine).
import { loadWasmEngine, type WasmEngine } from "./wasm";

export type Req = { id: number; m: string; a: unknown[] };
export type Res = { id: number; ok: true; r: unknown } | { id: number; ok: false; e: string };

let engine: WasmEngine | null = null;

self.onmessage = async (ev: MessageEvent<Req>) => {
  const { id, m, a } = ev.data;
  let res: Res;
  try {
    if (m === "init") {
      engine = await loadWasmEngine();
      res = { id, ok: true, r: engine ? engine.version : null };
    } else if (!engine) {
      res = { id, ok: false, e: "engine not initialised" };
    } else {
      const fn = (engine as unknown as Record<string, (...x: unknown[]) => unknown>)[m];
      if (typeof fn !== "function") throw new Error(`no engine method ${m}`);
      res = { id, ok: true, r: fn.apply(engine, a) };
    }
  } catch (e) {
    res = { id, ok: false, e: e instanceof Error ? e.message : String(e) };
  }
  (self as unknown as Worker).postMessage(res);
};
