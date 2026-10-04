// Development only: the same Rust bindings over an ordered loopback transport.
import type { AsyncEngine } from "./types";
import api from "./native-api.json";
export async function nativeEngine(): Promise<{ engine: AsyncEngine; version: string }> {
  const session = crypto.randomUUID();
  let tail: Promise<unknown> = Promise.resolve();
  const call = (method: string, args: unknown[] = []): Promise<unknown> => {
    const result = tail.then(async () => {
      const response = await fetch("/__native/rpc", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify({ session, method, args }) });
      const value = await response.json();
      if (!response.ok || !value.ok) throw new Error(value.e ?? value.error ?? "native engine unavailable");
      return value.r;
    });
    tail = result.catch(() => {}); return result;
  };
  // Release all lane processes on reload; rapid HMR rounds must not fill the
  // session cap with pages that no longer exist. BFCache pages retain theirs.
  window.addEventListener('pagehide', (event) => {
    if (!event.persisted) navigator.sendBeacon('/__native/close', JSON.stringify({ session }));
  });
  if (await call("__bridge") !== api.bridgeSha256) throw new Error("native bridge is stale; run tools/native-dev.sh again");
  const version = String(await call("version"));
  const engine: Record<string, (...args: unknown[]) => Promise<unknown>> = {};
  for (const method of api.methods) engine[method.name] = async (...args) => {
    const wire = args.map((a, i) => method.args[i]?.type === "&str" && typeof a !== "string" ? JSON.stringify(a) : a);
    const r = await call(method.name, wire);
    if (method.void) return undefined;
    return method.raw ? r : typeof r === "string" ? JSON.parse(r) : r;
  };
  return { engine: engine as unknown as AsyncEngine, version };
}
