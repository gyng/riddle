// State machine: camp ⇄ watch ⇄ death ⇄ report. Owns the engine, the three saved sets, persistence.
import type { Death, Engine, Forecast, Lineage, ReturnReport, Row, RuleSet, Vocabulary } from "./engine/types";
import { selectEngine, type EngineKind } from "./engine/index";
import { readBlob, writeBlob, clearBlob, randomSeed, type SaveBlob } from "./store";
import { renderCamp } from "./ui/camp";
import { renderWatch } from "./ui/watch";
import { renderDeath } from "./ui/death";
import { renderReport } from "./ui/report";
import { closeAllSheets } from "./ui/sheet";

export type Screen =
  | { kind: "camp"; highlight?: number }
  | { kind: "watch" }
  | { kind: "death"; death: Death; lost?: string[] }
  | { kind: "report"; report: ReturnReport };

export type Mounted = { el: HTMLElement; dispose?: () => void };

const OFFLINE_MIN_S = 60;
const SETS = 3;

export class App {
  engine!: Engine;
  kind: EngineKind = "fake";
  version = "";
  lineage!: Lineage;
  vocab!: Vocabulary;
  sets: RuleSet[] = [];
  active = 0;
  loadout: number[] = [];
  screen: Screen = { kind: "camp" };
  private root: HTMLElement;
  private mounted: Mounted | null = null;
  private saveTimer = 0;
  private fcTimer = 0;
  private fcListeners = new Set<(f: Forecast) => void>();
  private changeListeners = new Set<() => void>();

  constructor(root: HTMLElement) { this.root = root; }

  async boot(): Promise<void> {
    const sel = await selectEngine();
    this.engine = sel.engine; this.kind = sel.kind; this.version = sel.version;
    const blob = readBlob();
    let elapsed = 0;
    if (blob) {
      try {
        this.lineage = this.engine.load(blob.engine);
        this.sets = blob.sets; this.active = blob.active; this.loadout = blob.loadout;
        elapsed = Math.max(0, (Date.now() - blob.last_seen) / 1000);
      } catch (e) { console.warn("save rejected, new lineage", e); this.fresh(); }
    } else this.fresh();
    this.normaliseSets();
    this.engine.setRules(this.rules);
    this.engine.loadout(this.loadout);
    this.vocab = this.engine.vocabulary();
    document.addEventListener("visibilitychange", () => { if (document.hidden) this.flush(); });
    window.addEventListener("pagehide", () => this.flush());
    setInterval(() => this.touch(), 30_000);
    if (blob && elapsed >= OFFLINE_MIN_S) {
      const report = this.engine.runOffline(Math.floor(elapsed));
      this.refresh();
      this.go({ kind: "report", report });
    } else this.go({ kind: "camp" });
    this.flush();
  }

  private fresh(): void {
    this.lineage = this.engine.newLineage(randomSeed());
    this.sets = this.lineage.sets.map(cloneSet);
    this.active = this.lineage.active_set ?? 0;
    this.loadout = [];
  }
  private normaliseSets(): void {
    while (this.sets.length < SETS) this.sets.push(this.sets[0] ? cloneSet(this.sets[0]) : { rows: [] });
    this.sets = this.sets.slice(0, SETS);
    if (this.active < 0 || this.active >= SETS) this.active = 0;
  }

  get rules(): RuleSet { return this.sets[this.active]; }

  // --- rules ---
  rulesChanged(): void {
    this.engine.setRules(this.rules);
    this.persist();
    clearTimeout(this.fcTimer);
    this.fcTimer = window.setTimeout(() => this.emitForecast(), 250);
  }
  onForecast(fn: (f: Forecast) => void): () => void { this.fcListeners.add(fn); return () => this.fcListeners.delete(fn); }
  emitForecast(): void {
    if (!this.fcListeners.size) return;
    const f = this.engine.forecast();
    for (const fn of this.fcListeners) fn(f);
  }
  selectSet(i: number): void {
    if (i === this.active || i < 0 || i >= this.sets.length) return;
    this.active = i;
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
  setRulesText(text: string): void {
    const set = this.engine.importRules(text);
    this.sets[this.active] = { rows: set.rows.map(cloneRow) };
    this.rulesChanged();
    this.emitChange();
  }

  // --- lineage ---
  refresh(): void { this.lineage = this.engine.lineage(); this.vocab = this.engine.vocabulary(); this.emitChange(); }
  onChange(fn: () => void): () => void { this.changeListeners.add(fn); return () => this.changeListeners.delete(fn); }
  private emitChange(): void { for (const fn of this.changeListeners) fn(); }
  /** After an engine call that returned a new Lineage: refresh vocab, persist, repaint, re-forecast. */
  afterLineage(): void { this.vocab = this.engine.vocabulary(); this.persist(); this.emitChange(); this.rulesChanged(); }
  buy(id: string): void { this.lineage = this.engine.buy(id); this.vocab = this.engine.vocabulary(); this.persist(); this.emitChange(); this.rulesChanged(); }
  setLoadout(ids: number[]): void { this.loadout = ids; this.engine.loadout(ids); this.persist(); this.emitChange(); }
  resetLineage(): void {
    clearBlob();
    this.fresh();
    this.normaliseSets();
    this.engine.setRules(this.rules);
    this.vocab = this.engine.vocabulary();
    this.flush();
    this.go({ kind: "camp" });
  }

  // --- persistence ---
  blob(): SaveBlob { return { v: 1, engine: this.engine.save(), sets: this.sets, active: this.active, loadout: this.loadout, last_seen: Date.now() }; }
  persist(): void { clearTimeout(this.saveTimer); this.saveTimer = window.setTimeout(() => this.flush(), 400); }
  flush(): void { clearTimeout(this.saveTimer); writeBlob(this.blob()); }
  private touch(): void { if (!document.hidden) this.flush(); }
  exportSave(): string { return JSON.stringify(this.blob()); }
  importSave(text: string): boolean {
    try {
      const b = JSON.parse(text) as SaveBlob;
      if (!b || typeof b.engine !== "string") return false;
      this.lineage = this.engine.load(b.engine);
      this.sets = Array.isArray(b.sets) ? b.sets : this.lineage.sets.map(cloneSet);
      this.active = b.active ?? 0; this.loadout = b.loadout ?? [];
      this.normaliseSets();
      this.engine.setRules(this.rules); this.engine.loadout(this.loadout);
      this.vocab = this.engine.vocabulary();
      this.flush();
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
  void app.boot();
  if (import.meta.env.PROD && "serviceWorker" in navigator) {
    window.addEventListener("load", () => { navigator.serviceWorker.register("/sw.js").catch(() => { /* offline-first is best effort */ }); });
  }
}
