// State machine: camp ⇄ watch ⇄ death ⇄ report. Owns the engine proxy (wasm in a worker, or the fake),
// the editing copy of the three saved sets, and persistence.
import type { AsyncEngine, Death, Forecast, Lineage, ReturnReport, Row, RuleSet, Vocabulary } from "./engine/types";
import { selectEngine, type EngineKind } from "./engine/index";
import { readBlob, writeBlob, clearBlob, randomSeed, type SaveBlob } from "./store";
import { renderCamp } from "./ui/camp";
import { renderWatch } from "./ui/watch";
import { renderDeath } from "./ui/death";
import { renderReport } from "./ui/report";
import { closeAllSheets } from "./ui/sheet";
import { showBusy } from "./ui/progress";

export type Screen =
  | { kind: "camp"; highlight?: number }
  | { kind: "watch" }
  | { kind: "death"; death: Death; lost?: string[] }
  | { kind: "report"; report: ReturnReport };

export type Mounted = { el: HTMLElement; dispose?: () => void };

const OFFLINE_MIN_S = 60;
const SETS = 3;
const SAVE_DEBOUNCE_MS = 1000;

export class App {
  engine!: AsyncEngine;
  kind: EngineKind = "fake";
  version = "";
  lineage!: Lineage;
  vocab!: Vocabulary;
  /** Editing copies of the engine's saved sets; `setRules` pushes the active one to the engine. */
  sets: RuleSet[] = [];
  active = 0;
  loadout: number[] = [];
  screen: Screen = { kind: "camp" };
  private root: HTMLElement;
  private mounted: Mounted | null = null;
  private saveTimer = 0;
  private lastSave = "";
  private fcTimer = 0;
  private fcInFlight = false;
  private fcDirty = false;
  private fcListeners = new Set<(f: Forecast) => void>();
  private changeListeners = new Set<() => void>();

  constructor(root: HTMLElement) { this.root = root; }

  async boot(): Promise<void> {
    const sel = await selectEngine();
    this.engine = sel.engine; this.kind = sel.kind; this.version = sel.version;
    const blob = readBlob();
    let elapsed = 0;
    let loaded = false;
    if (blob) {
      try {
        this.lineage = await this.engine.load(blob.engine);
        this.loadout = blob.loadout;
        elapsed = Math.max(0, (Date.now() - blob.last_seen) / 1000);
        loaded = true;
      } catch (e) { console.warn("save rejected, new lineage", e); }
    }
    if (!loaded) await this.fresh();
    this.adoptSets();
    await this.engine.loadout(this.loadout);
    this.vocab = await this.engine.vocabulary();
    document.addEventListener("visibilitychange", () => { if (document.hidden) this.flushSync(); });
    window.addEventListener("pagehide", () => this.flushSync());
    setInterval(() => { if (!document.hidden) void this.flush(); }, 30_000);
    if (loaded && elapsed >= OFFLINE_MIN_S) {
      const report = await this.busy(/* copy:label */ "offline", () => this.engine.runOffline(Math.floor(elapsed)));
      await this.refresh();
      this.adoptSets();
      this.go({ kind: "report", report });
    } else this.go({ kind: "camp" });
    await this.flush();
  }

  /** Runs an engine call behind the progress bar. */
  async busy<T>(label: string, fn: () => Promise<T>): Promise<T> {
    const done = showBusy(label);
    try { return await fn(); } finally { done(); }
  }

  private async fresh(): Promise<void> {
    this.lineage = await this.engine.newLineage(randomSeed());
    this.loadout = [];
  }
  /** Take the editing copies from the engine's lineage (the engine is the source of truth). */
  private adoptSets(): void {
    this.sets = (this.lineage.sets ?? []).map(cloneSet);
    while (this.sets.length < SETS) this.sets.push(this.sets[0] ? cloneSet(this.sets[0]) : { rows: [] });
    this.sets = this.sets.slice(0, SETS);
    this.active = this.lineage.active_set ?? 0;
    if (this.active < 0 || this.active >= SETS) this.active = 0;
  }

  get rules(): RuleSet { return this.sets[this.active]; }

  // --- rules ---
  rulesChanged(): void {
    void this.engine.setRules(this.rules).catch((e) => console.warn("rules rejected", e));
    this.persist();
    clearTimeout(this.fcTimer);
    this.fcTimer = window.setTimeout(() => void this.emitForecast(), 250);
  }
  onForecast(fn: (f: Forecast) => void): () => void { this.fcListeners.add(fn); return () => this.fcListeners.delete(fn); }
  async emitForecast(): Promise<void> {
    if (!this.fcListeners.size) return;
    if (this.fcInFlight) { this.fcDirty = true; return; }
    this.fcInFlight = true; this.fcDirty = false;
    try {
      const f = await this.busy(/* copy:label */ "forecast", () => this.engine.forecast());
      for (const fn of this.fcListeners) fn(f);
    } catch (e) { console.warn("forecast failed", e); }
    finally { this.fcInFlight = false; }
    if (this.fcDirty) await this.emitForecast();
  }
  selectSet(i: number): void {
    if (i === this.active || i < 0 || i >= this.sets.length) return;
    this.active = i;
    void this.engine.selectSet(i).catch((e) => console.warn("selectSet", e));
    this.rulesChanged();
    this.emitChange();
  }
  insertRow(row: Row, at: number): number {
    const rows = this.rules.rows;
    const i = Math.max(0, Math.min(rows.length, at));
    if (rows.length >= this.vocab.max_rows) rows.pop();
    rows.splice(i, 0, cloneRow(row));
    this.rulesChanged();
    return i;
  }
  async setRulesText(text: string): Promise<void> {
    const set = await this.engine.importRules(text);
    this.sets[this.active] = { rows: set.rows.map(cloneRow) };
    this.rulesChanged();
    this.emitChange();
  }

  // --- lineage ---
  async refresh(): Promise<void> { this.lineage = await this.engine.lineage(); this.vocab = await this.engine.vocabulary(); this.emitChange(); }
  onChange(fn: () => void): () => void { this.changeListeners.add(fn); return () => this.changeListeners.delete(fn); }
  private emitChange(): void { for (const fn of this.changeListeners) fn(); }
  /** After an engine call that returned a new Lineage: refresh vocab, persist, repaint, re-forecast. */
  async afterLineage(): Promise<void> { this.vocab = await this.engine.vocabulary(); this.persist(); this.emitChange(); this.rulesChanged(); }
  /** Runs an engine call that returns a Lineage and adopts it. Errors (unaffordable, locked) are swallowed after a warn. */
  async mutate(fn: () => Promise<Lineage>): Promise<boolean> {
    try { this.lineage = await fn(); } catch (e) { console.warn("engine refused", e); return false; }
    await this.afterLineage();
    return true;
  }
  buy(id: string): Promise<boolean> { return this.mutate(() => this.engine.buy(id)); }
  setClass(cls: string): Promise<boolean> { return this.mutate(() => this.engine.setClass(cls)); }
  setLoadout(ids: number[]): void { this.loadout = ids; void this.engine.loadout(ids); this.persist(); this.emitChange(); }
  async resetLineage(): Promise<void> {
    clearBlob();
    await this.fresh();
    this.adoptSets();
    await this.engine.setRules(this.rules);
    this.vocab = await this.engine.vocabulary();
    await this.flush();
    this.go({ kind: "camp" });
  }

  // --- persistence ---
  persist(): void { clearTimeout(this.saveTimer); this.saveTimer = window.setTimeout(() => void this.flush(), SAVE_DEBOUNCE_MS); }
  async flush(): Promise<void> {
    clearTimeout(this.saveTimer);
    try { this.lastSave = await this.engine.save(); } catch (e) { console.warn("save failed", e); return; }
    writeBlob(this.blob());
  }
  /** pagehide/visibilitychange cannot await the worker: write the last save string fetched. */
  private flushSync(): void { if (this.lastSave) writeBlob(this.blob()); }
  private blob(): SaveBlob { return { v: 2, engine: this.lastSave, loadout: this.loadout, last_seen: Date.now() }; }
  exportSave(): string { return JSON.stringify(this.blob()); }
  async importSave(text: string): Promise<boolean> {
    try {
      const b = JSON.parse(text) as SaveBlob;
      if (!b || typeof b.engine !== "string") return false;
      this.lineage = await this.engine.load(b.engine);
      this.loadout = b.loadout ?? [];
      this.adoptSets();
      await this.engine.setRules(this.rules); await this.engine.loadout(this.loadout);
      this.vocab = await this.engine.vocabulary();
      await this.flush();
      this.go({ kind: "camp" });
      return true;
    } catch { return false; }
  }

  // --- screens ---
  go(screen: Screen): void {
    closeAllSheets();
    this.mounted?.dispose?.();
    this.screen = screen;
    let m: Mounted;
    switch (screen.kind) {
      case "camp": m = renderCamp(this, screen.highlight); break;
      case "watch": m = renderWatch(this); break;
      case "death": m = renderDeath(this, screen.death, screen.lost ?? []); break;
      case "report": m = renderReport(this, screen.report); break;
    }
    this.mounted = m;
    this.root.replaceChildren(m.el);
    this.root.dataset.screen = screen.kind;
    window.scrollTo(0, 0);
    this.persist();
  }
}

export const cloneRow = (r: Row): Row => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb } });
export const cloneSet = (s: RuleSet): RuleSet => ({ rows: s.rows.map(cloneRow), name: s.name });

export function start(): void {
  const root = document.getElementById("app") ?? document.body.appendChild(document.createElement("div"));
  root.id = "app";
  const app = new App(root);
  if (import.meta.env.DEV) (window as unknown as { __riddle: App }).__riddle = app; // dev inspection only
  void app.boot().catch((e) => console.error("boot failed", e));
  if (import.meta.env.PROD && "serviceWorker" in navigator) {
    window.addEventListener("load", () => { navigator.serviceWorker.register("/sw.js").catch(() => { /* offline-first is best effort */ }); });
  }
}
