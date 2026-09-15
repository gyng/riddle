// Real Engine over the wasm-pack output in ./pkg (built by tools/verify.sh). Imported lazily via
// import.meta.glob so the client compiles and runs (falling back to the fake) when pkg/ is absent.
//
// Assumed wasm surface (docs/CUT1.md): `class Game { constructor(seed: number) }` with one method per
// Engine member, camelCased, taking/returning JSON strings (exportRules returns plain text; setRules and
// loadout return nothing). `load` may be a static constructor (`Game.load(json)`) or an instance method;
// both are handled. Values that arrive already-parsed (serde-wasm-bindgen) are accepted too.
import type {
  Death, Engine, Forecast, Lineage, ReturnReport, RuleSet, Snapshot, StepResult, SupplyEntry, Vocabulary,
} from "./types";

type GameObj = Record<string, (...args: unknown[]) => unknown>;
type GameCtor = (new (seed: number) => GameObj) & { load?: (save: string) => GameObj };
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
    if (typeof this.Game.load === "function") { this.g = this.Game.load(save); return this.call<Lineage>("lineage"); }
    if (!this.g) this.g = new this.Game(0);
    const r = this.call<Lineage | undefined>("load", save);
    return r ?? this.call<Lineage>("lineage");
  }
  save(): string { const r = this.game.save(); return typeof r === "string" ? r : JSON.stringify(r); }
  vocabulary(): Vocabulary { return this.call("vocabulary"); }
  setRules(set: RuleSet): void { this.game.setRules(JSON.stringify(set)); }
  loadout(itemIds: number[]): void { this.game.loadout(JSON.stringify(itemIds)); }
  forecast(): Forecast { return this.call("forecast"); }
  send(): Snapshot { return this.call("send"); }
  step(turns: number): StepResult { return this.call("step", turns); }
  runOffline(elapsedS: number): ReturnReport { return this.call("runOffline", elapsedS); }
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
}

/** Resolves to a WasmEngine, or null when pkg/ is not built. */
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
