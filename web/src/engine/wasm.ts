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
  CageOption, Death, DescentOffer, Divergence, FoldLine, KitLadder, ForecastVs, StartOption, ForkOption, Engine, Forecast, Lineage, Patch, ReturnReport, RuleSet, Snapshot, StepResult, SupplyEntry, UnlockInfo, Vocabulary, ForecastMove, StandingOrders, WallEdit, PkgOption, Advance, Replay, ManualAct } from "./types";

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
  forecastEstimate(): Forecast { return this.call(typeof this.game.forecastEstimate === "function" ? "forecastEstimate" : "forecast"); }
  send(): Snapshot { return this.call("send"); }
  step(turns: number): StepResult { return this.call("step", turns); }
  runOffline(elapsedS: number): ReturnReport { return this.call("runOffline", elapsedS); }
  runOfflineQuick(elapsedS: number): ReturnReport { return this.call("runOfflineQuick", elapsedS); }
  runOfflineSlice(elapsedS: number, last: boolean): ReturnReport { return this.call("runOfflineSlice", elapsedS, last); }
  death(runId: number): Death { return this.call("death", runId); }
  deathDeltas(runId: number): Patch[] { return this.call("deathDeltas", runId); }
  buy(unlock: string): Lineage { return this.call("buy", unlock); }
  buyUnlockGold(unlock: string): Lineage { return this.call("buyUnlockGold", unlock); }   // Cut 15 §2
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
  autoKeep(): Lineage { return this.call("autoKeep"); }
  setKeepPref(pref: string): Lineage { return this.call("setKeepPref", pref); }
  setLook(look: string): Lineage { return this.call("setLook", look); }   // hero looks
  insure(id: number): Lineage { return this.call("insure", id); }
  sellVault(id: number): Lineage { return this.call("sellVault", id); }   // QA 308f045: an item out of the vault, salvaged
  // core additions
  unlocks(): UnlockInfo[] { return this.call("unlocks"); }
  unlockDeltas(): UnlockInfo[] { return this.call("unlockDeltas"); }
  setClass(cls: string): Lineage { return this.call("setClass", cls); }
  selectSet(i: number): Lineage { return this.call("selectSet", i); }
  // Cut 3
  descentOffer(tier:number): DescentOffer { return this.call("descentOffer", tier); }
  beginDescent(tier:number): Lineage { return this.call("beginDescent", tier); }
  ascend(variant: string): Lineage { return this.call("ascend", variant); }
  // Cut 5
  bail(): void { this.game.bail(); }
  choose(itemId: number): Snapshot { return this.call("choose", itemId); }
  setVaultPref(pref: string): Lineage { return this.call("setVaultPref", pref); }
  // Cut 6 §9: throws `wasm: forecastRefine` on a build without it (the client stops asking)
  forecastRefine(): Forecast { return this.call("forecastRefine"); }
  // Cut 12 §6: throws `wasm: dropSupply` on a build without it (the client clears and rebuys the rest)
  dropSupply(id: number): Lineage { return this.call("dropSupply", id); }
  // Cut 13 §2: throws `wasm: setTrait` on a build without it (the camp shows no chips without an offer anyway)
  setTrait(name: string): Lineage { return this.call("setTrait", name); }
  // Cut 19: throw `wasm: cageForecast` / `wasm: setRestock` on a build without them
  // QA 524827b: `refined` — the options on the camp's refined pass (the sheet's current option is the camp's number)
  cageForecast(refined?: boolean): CageOption[] { return this.call("cageForecast", refined); }
  setRestock(on: boolean): Lineage { return this.call("setRestock", on); }
  // Cut 21 §1: throw `wasm: setStart` / `wasm: startForecast` on a build without them
  setStart(depth: number): Lineage { return this.call("setStart", depth); }
  startForecast(refined?: boolean): StartOption[] { return this.call("startForecast", refined); }   // QA 308f045: on the camp's pass
  // Cut 26 §2: throws `wasm: forkForecast` on a build without it (the fork chip's sheet then shows the chips alone)
  forkForecast(fork: number, refined?: boolean): ForkOption[] { return this.call("forkForecast", fork, refined); }   // QA 308f045: on the camp's pass
  // Cut 22 §3: throws `wasm: forecastVs` on a build without it (the client then reads `Forecast.vs`, or shows no move)
  forecastVsEstimate(prev: RuleSet): ForecastVs { return this.call(typeof this.game.forecastVsEstimate === "function" ? "forecastVsEstimate" : "forecastVs", JSON.stringify(prev)); }
  forecastVs(prev: RuleSet): ForecastVs { return this.call("forecastVs", JSON.stringify(prev)); }
  // Cut 23 §1: throw `wasm: buyKit` / `wasm: kitDeltas` on a build without them (the camp shows no forge)
  buyKit(slot: string): Lineage { return this.call("buyKit", slot); }
  takeReturnPick(id: string): Lineage { return this.call("takeReturnPick", id); }   // Cut 113 §3
  kitDeltas(): KitLadder[] { return this.call("kitDeltas"); }
  kitEstimates(): KitLadder[] { return this.call(typeof this.game.kitEstimates === "function" ? "kitEstimates" : "kitDeltas"); }
  // Cut 27: throw `wasm: fold` / `wasm: divergence` on a build without them (the watch then plays every floor; the camp shows no scene)
  fold(): FoldLine { return this.call("fold"); }
  divergence(prev: RuleSet): Divergence | null { return this.call("divergence", JSON.stringify(prev)); }
  // Cut 28: throw `wasm: forecastMove` / `wasm: swearOath` / `wasm: forswearOath` on a build without them
  forecastMove(prev: RuleSet): ForecastMove | null { return this.call("forecastMove", JSON.stringify(prev)); }
  swearOath(id: string): Lineage { return this.call("swearOath", id); }
  forswearOath(): Lineage { return this.call("forswearOath"); }
  // Cut 29: throw `wasm: <name>` on a build without them
  drawOath(): Lineage { return this.call("drawOath"); }
  forswearOathId(id: string): Lineage { return this.call("forswearOathId", id); }
  commission(): Lineage { return this.call("commission"); }
  seenSystems(): Lineage { return this.call("seenSystems"); }
  wallEdit(): WallEdit | null { return this.call("wallEdit"); }
  setOrders(orders: StandingOrders): Lineage { return this.call("setOrders", JSON.stringify(orders)); }
  // Cut 30: throw `wasm: <name>` on a build without them
  equipPackage(id: string, slot: number): Lineage { return this.call("equipPackage", id, slot); }
  unequipPackage(id: string): Lineage { return this.call("unequipPackage", id); }
  pickTemperament(id: string): Lineage { return this.call("pickTemperament", id); }
  spendLevel(id: string): Lineage { return this.call("spendLevel", id); }
  setTacticVariant(id: string, variant: number): Lineage { return this.call("setTacticVariant", id, variant); }
  takeFix(id: string, variant: number): Lineage { return this.call("takeFix", id, variant); }
  takeControl(on: boolean): void { this.game.takeControl(on); }
  act(action: ManualAct): void { this.game.act(JSON.stringify(action)); }
  revokeDrill(boss: string, revoked: boolean): Lineage { return this.call("revokeDrill", boss, revoked); }
  packageOptions(sims: number): PkgOption[] { return this.call("packageOptions", sims); }
  packageOptionsFor(sims: number, choices: [string, number][]): PkgOption[] {
    if (typeof this.g?.packageOptionsFor !== "function") return this.packageOptions(sims).filter((o) => o.action === "equip" && choices.some(([id, slot]) => o.id === id && o.slot === slot));
    return this.call("packageOptionsFor", sims, JSON.stringify(choices));
  }
  packageOptionsForKey(sims: number, choices: [string, number][]): string { return this.call("packageOptionsForKey", sims, JSON.stringify(choices)); }
  packageOptionsKey(sims: number): string { return this.call("packageOptionsKey", sims); }
  bankDeposit(amount: number): Lineage { return this.call("bankDeposit", amount); }
  bankWithdraw(amount: number): Lineage { return this.call("bankWithdraw", amount); }
  swapQuest(): Lineage { return this.call("swapQuest"); }
  // Cut 30.5: throw `wasm: <name>` on a build without them
  hire(id: string): Lineage { return this.call("hire", id); }
  selectBloodline(id: number): Lineage { return this.call("selectBloodline", id); }
  addBloodline(): Lineage { return this.call("addBloodline"); }
  upgradeHero(id: string): Lineage { return this.call("upgradeHero", id); }
  upgradeHeroNext(id: string): Lineage { return this.call("upgradeHeroNext", id); }
  setSpecialization(id:string):Lineage { return this.call("setSpecialization",id); }
  respecLegacy():Lineage { return this.call("respecLegacy"); }
  buildTown(id: string): Lineage { return this.call("buildTown", id); }
  openChest(): Lineage { return this.call("openChest"); }
  setWorker(id: string, on: boolean): Lineage { return this.call("setWorker", id, on); }
  promote(id: string): Lineage { return this.call("promote", id); }
  // RUNS_UI: throw `wasm: <name>` on a build without them
  advance(elapsedMs: number): Advance { return this.call("advance", elapsedMs); }
  replay(runId: number): Replay | null { return this.call("replay", runId); }
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
