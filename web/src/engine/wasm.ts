// Real Engine over the wasm-pack output in ./pkg (built by tools/verify.sh). Imported lazily via
// import.meta.glob so the client compiles and runs (falling back to the fake) when pkg/ is absent.
//
// Wasm surface (crates/riddle-wasm/src/lib.rs): `class Game { constructor(seed) }` with one method per
// Engine member, camelCased, taking/returning JSON strings (exportRules returns plain text; setRules,
// loadout and setCompanionRules return nothing; `load` is an instance method, `fromSave` a static).
// Errors surface as thrown JsError (invalid rules, unaffordable unlocks, …).
//
// This synchronous engine runs inside the Web Worker (./worker.ts); the main thread talks to it
// through the AsyncEngine proxy in ./proxy.ts.
import type {
  Death, Engine, Forecast, Lineage, ReturnReport, RuleSet, Snapshot, StepResult, SupplyEntry, UnlockInfo, Vocabulary,
} from "./types";

type GameObj = Record<string, (...args: unknown[]) => unknown>;
type GameCtor = (new (seed: number) => GameObj) & { fromSave?: (save: string) => GameObj };
type Pkg = { default?: (input?: unknown) => Promise<unknown>; Game: GameCtor; version?: () => string };

const mods = import.meta.glob<Pkg>("./pkg/riddle_wasm.js");

function j<T>(x: unknown): T { return (typeof x === "string" ? JSON.parse(x) : x) as T; }

export class WasmEngine implements Engine {
  private g: GameObj | null = null;
  private readonly Game: GameCtor;
  readonly version: string;
  constructor(Game: GameCtor, version: string) { this.Game = Game; this.version = version; }
  private get game(): GameObj { if (!this.g) throw new Error("no lineage"); return this.g; }
  private call<T>(name: string, ...args: unknown[]): T {
    const fn = this.game[name]; if (typeof fn !== "function") throw new Error(`wasm: ${name}`);
    return j<T>(fn.apply(this.game, args));
  }
  newLineage(seed: number): Lineage { this.g = new this.Game(seed >>> 0); return this.call<Lineage>("lineage"); }
  load(save: string): Lineage {
    if (typeof this.Game.fromSave === "function") { this.g = this.Game.fromSave(save); return this.call<Lineage>("lineage"); }
    if (!this.g) this.g = new this.Game(0);
    return this.call<Lineage>("load", save);
  }
  save(): string { const r = this.game.save(); return typeof r === "string" ? r : JSON.stringify(r); }
  vocabulary(): Vocabulary { return this.call("vocabulary"); }
  setRules(set: RuleSet): void { this.game.setRules(JSON.stringify(set)); }
  loadout(itemIds: number[]): void { this.game.loadout(JSON.stringify(itemIds)); }
  forecast(): Forecast { return this.call("forecast"); }
  send(): Snapshot { return this.call("send"); }
  step(turns: number): StepResult { return this.call("step", turns); }
  runOffline(elapsedS: number): ReturnReport { return this.call("runOffline", elapsedS); }
  runOfflineQuick(elapsedS: number): ReturnReport { return this.call("runOfflineQuick", elapsedS); }
  death(runId: number): Death { return this.call("death", runId); }
  buy(unlock: string): Lineage { return this.call("buy", unlock); }
  lineage(): Lineage { return this.call("lineage"); }
  exportRules(): string { const r = this.game.exportRules(); return typeof r === "string" ? r : JSON.stringify(r); }
  importRules(text: string): RuleSet { return this.call("importRules", text); }
  // Addendum A
  setParty(ids: number[]): Lineage { return this.call("setParty", JSON.stringify(ids)); }
  setCompanionRules(id: number, set: RuleSet): void { this.game.setCompanionRules(id, JSON.stringify(set)); }
  breed(a: number, b: number): Lineage { return this.call("breed", a, b); }
  hatch(eggId: number): Lineage { return this.call("hatch", eggId); }
  companionVocabulary(id: number): Vocabulary { return this.call("companionVocabulary", id); }
  // Addendum B
  buySupply(kind: string): Lineage { return this.call("buySupply", kind); }
  clearSupplies(): Lineage { return this.call("clearSupplies"); }
  supplyCatalogue(): SupplyEntry[] { return this.call("supplyCatalogue"); }
  // Addendum D
  keep(ids: number[]): Lineage { return this.call("keep", JSON.stringify(ids)); }
  setKeepPref(pref: string): Lineage { return this.call("setKeepPref", pref); }
  insure(id: number): Lineage { return this.call("insure", id); }
  // core additions
  unlocks(): UnlockInfo[] { return this.call("unlocks"); }
  unlockDeltas(): UnlockInfo[] { return this.call("unlockDeltas"); }
  setClass(cls: string): Lineage { return this.call("setClass", cls); }
  selectSet(i: number): Lineage { return this.call("selectSet", i); }
  // Cut 3
  ascend(variant: string): Lineage { return this.call("ascend", variant); }
}

/** Resolves to a WasmEngine, or null when pkg/ is not built. Works on the main thread and in a worker. */
export async function loadWasmEngine(): Promise<WasmEngine | null> {
  const loader = Object.values(mods)[0];
  if (!loader) return null;
  try {
    const pkg = await loader();
    if (pkg.default) await pkg.default();
    if (typeof pkg.Game !== "function") return null;
    return new WasmEngine(pkg.Game, typeof pkg.version === "function" ? pkg.version() : "?");
  } catch (e) {
    console.warn("wasm engine unavailable", e);
    return null;
  }
}
