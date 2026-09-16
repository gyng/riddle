// State machine: camp ⇄ watch ⇄ death ⇄ report. Owns the engine proxy (wasm in a worker, or the fake),
// the editing copy of the three saved sets, and persistence.
import type { AsyncEngine, Death, Forecast, Lineage, Patch, ReturnReport, Row, RuleSet, Vocabulary } from "./engine/types";
import { selectEngine, type EngineKind } from "./engine/index";
import { readBlob, writeBlob, clearBlob, randomSeed, type SaveBlob } from "./store";
import { renderCamp } from "./ui/camp";
import { renderWatch } from "./ui/watch";
import { renderDeath } from "./ui/death";
import { renderReport } from "./ui/report";
import { renderEnding } from "./ui/ending";
import { closeAllSheets } from "./ui/sheet";
import { showBusy } from "./ui/progress";

export type Screen =
  | { kind: "camp"; highlight?: number }
  | { kind: "watch" }
  | { kind: "death"; death: Death; lost?: string[] }
  | { kind: "report"; report: ReturnReport }
  | { kind: "ending" };

export type Mounted = { el: HTMLElement; dispose?: () => void };

/** Dev-only boot options (parsed from the URL in main.ts; `tools/dev.sh` + `tools/playtest.mjs`). */
export type DevOptions = {
  seed?: number;      // new lineage with this seed on a fresh boot (ignored while a save exists, unless `fresh`)
  fresh?: boolean;    // clear the save first
  absent?: number;    // seconds: treat last_seen as that far back, so the offline report runs
  rules?: string;     // rule-set text for `importRules`, applied to the active set before anything else
  speed?: number;     // watch speed to press on entering a run (1 slow | 4 fast | 8 auto, the default)
  autosend?: boolean; // send straight from boot (the camp is skipped so its forecast does not queue ahead of `send`)
};
/** What a rater or script sees: the mounted screen, or `exit` while the exit sheet is up over a run. */
export type DevScreen = "camp" | "watch" | "death" | "report" | "ending" | "exit";

const OFFLINE_MIN_S = 60;
const SETS = 3;
const SAVE_DEBOUNCE_MS = 1000;
// runOffline is chunked so the progress label can count runs. `runOfflineQuick` skips the worst-death verdict
// (~3 s per slice; one `death(id)` at the end instead), so slices are a flat 30 min. On a stale wasm build
// without it, the full call is used with slices that grow with the absence (80 min for 8 h, 2 h cap).
const OFFLINE_SLICE_S = 30 * 60, OFFLINE_SLICE_MAX_S = 2 * 3600, OFFLINE_SLICES = 6;

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
  view: Screen = { kind: "camp" };
  /** True once `boot()` has settled (after the offline batch, if any). Dev inspection. */
  booted = false;
  private root: HTMLElement;
  private dev: DevOptions | null;
  private mounted: Mounted | null = null;
  private saveTimer = 0;
  private lastSave = "";
  private fcTimer = 0;
  private fcInFlight = false;
  private fcDirty = false;
  private fcListeners = new Set<(f: Forecast) => void>();
  private changeListeners = new Set<() => void>();
  private rulesListeners = new Set<() => void>();
  private offlineRunning = false;
  /** Runs seen by this client (the wire Lineage has no run counter); persisted in the blob. */
  runsSeen = 0;

  constructor(root: HTMLElement, dev: DevOptions | null = null) { this.root = root; this.dev = dev; }

  async boot(): Promise<void> {
    const dev = this.dev;
    const sel = await selectEngine();
    this.engine = sel.engine; this.kind = sel.kind; this.version = sel.version;
    if (dev?.fresh) clearBlob();
    const blob = readBlob();
    let elapsed = 0;
    let loaded = false;
    if (blob) {
      try {
        this.lineage = await this.engine.load(blob.engine);
        this.loadout = blob.loadout;
        this.runsSeen = blob.runs ?? 0;
        elapsed = Math.max(0, (Date.now() - blob.last_seen) / 1000);
        loaded = true;
      } catch (e) { console.warn("save rejected, new lineage", e); }
    }
    if (!loaded) await this.fresh(dev?.seed);
    this.adoptSets();
    if (dev?.rules) {
      try {
        const set = await this.engine.importRules(dev.rules);
        this.sets[this.active] = { rows: set.rows.map(cloneRow) };
        await this.engine.setRules(this.rules);
      } catch (e) { console.warn("dev rules rejected", e); }
    }
    await this.engine.loadout(this.loadout);
    this.vocab = await this.engine.vocabulary();
    document.addEventListener("visibilitychange", () => { if (document.hidden) this.flushSync(); });
    window.addEventListener("pagehide", () => this.flushSync());
    setInterval(() => { if (!document.hidden) void this.flush(); }, 30_000);
    if (dev?.absent) { elapsed = dev.absent; loaded = true; }
    if (loaded && elapsed >= OFFLINE_MIN_S) {
      // the camp (last state) shows underneath, inert, while the batch runs (no forecast queued ahead of it)
      this.offlineRunning = true;
      this.go({ kind: "camp" });
      const report = await this.runOfflineChunked(Math.floor(elapsed));
      await this.refresh();
      this.adoptSets();
      this.go({ kind: "report", report });
    } else this.go({ kind: dev?.autosend ? "watch" : "camp" });
    await this.flush();
    this.booted = true;
  }

  // --- dev inspection (window.__riddle in dev builds or with ?dev=1) ---
  /** The screen a rater sees: the mounted screen, or `exit` while the exit sheet is open over a run. */
  get screen(): DevScreen {
    if (this.view.kind === "watch" && document.querySelector(".sheet-wrap")) return "exit";
    return this.view.kind;
  }
  /** Is the progress bar up (offline batch, forecast, verdict)? */
  get engineBusy(): boolean { return !!document.querySelector(".busy"); }
  /** Everything on screen as text: the mounted screen, open sheets, the busy label. */
  text(): string { return document.body.innerText; }

  /** Runs an engine call behind the progress bar. */
  async busy<T>(label: string, fn: () => Promise<T>): Promise<T> {
    const b = showBusy(label);
    try { return await fn(); } finally { b.done(); }
  }

  /** `runOfflineQuick` in 30-minute slices, reports merged client-side, progress label `runs N · best Dk`; then one
   *  `death(id)` for the deepest slice's worst death (deepest, ties → later: the core's own ordering, read off the
   *  graveyard entries each slice adds) becomes the merged report's `worst_death`. */
  async runOfflineChunked(elapsedS: number): Promise<ReturnReport> {
    this.offlineRunning = true;
    this.root.inert = true;
    const b = showBusy(/* copy:label */ "offline");
    let merged: ReturnReport | null = null;
    let quick = true;
    let worst: { id: number; depth: number } | null = null;
    let graves = this.lineage.graveyard.length;
    try {
      let slice = OFFLINE_SLICE_S;
      for (let left = elapsedS; left > 0; left -= slice) {
        let r: ReturnReport;
        if (quick) {
          try { r = await this.engine.runOfflineQuick(Math.min(left, slice)); }
          catch (e) {
            if (merged) throw e;
            // stale wasm build without runOfflineQuick: the full call, larger slices (each pays the verdict)
            console.warn("runOfflineQuick unavailable, using runOffline", e); quick = false;
            slice = Math.min(OFFLINE_SLICE_MAX_S, Math.max(OFFLINE_SLICE_S, Math.ceil(elapsedS / OFFLINE_SLICES)));
            r = await this.engine.runOffline(Math.min(left, slice));
          }
        } else r = await this.engine.runOffline(Math.min(left, slice));
        merged = merged ? mergeReports(merged, r) : r;
        if (r.worst_death_id !== undefined && !r.worst_death) {
          const L = await this.engine.lineage();
          const depth = Math.max(0, ...L.graveyard.slice(graves).map((g) => g.depth));
          graves = L.graveyard.length;
          if (!worst || depth >= worst.depth) worst = { id: r.worst_death_id, depth };
        }
        const best = Math.max(this.lineage.best_depth, ...merged.bests.map((x) => Number(/^D(\d+)$/.exec(x)?.[1] ?? 0)));
        b.set(`runs ${merged.runs} · best D${best}`);
      }
      if (worst && !merged!.worst_death) {
        b.set(/* copy:label */ "verdict");
        try { merged!.worst_death = await this.engine.death(worst.id); } catch (e) { console.warn("worst death", e); }
      }
    } finally { b.done(); this.root.inert = false; this.offlineRunning = false; }
    this.runsSeen += merged!.runs;
    return merged!;
  }
  totalRuns(): number { return this.runsSeen; }

  /** Cut 3: after the ending, ascend under a variant (`no_rest | short_list | bones_only | hunted`): the engine keeps
   *  classes, kennel, vault, facts, forge, trophies and rules. An engine without `ascend` falls back to `again()`. */
  async ascend(variant: string): Promise<void> {
    try { this.lineage = await this.engine.ascend(variant); }
    catch (e) { console.warn("ascend unavailable, starting again", e); return this.again(); }
    this.loadout = []; this.runsSeen = 0;
    this.adoptSets();
    await this.engine.loadout([]);
    this.vocab = await this.engine.vocabulary();
    await this.flush();
    this.go({ kind: "camp" });
  }
  /** After the ending: a fresh lineage that keeps the player's three rule sets (facts, classes, meta reset —
   *  the core has no carry-over method). */
  async again(): Promise<void> {
    const sets = this.sets.map(cloneSet); const active = this.active;
    this.lineage = await this.engine.newLineage(randomSeed());
    for (let i = 0; i < sets.length; i++) { await this.engine.selectSet(i); await this.engine.setRules(sets[i]).catch(() => { /* a set the fresh vocabulary rejects stays the preset */ }); }
    await this.engine.selectSet(active);
    this.lineage = await this.engine.lineage();
    this.loadout = []; this.runsSeen = 0;
    this.adoptSets();
    await this.engine.loadout([]);
    this.vocab = await this.engine.vocabulary();
    await this.flush();
    this.go({ kind: "camp" });
  }

  private async fresh(seed?: number): Promise<void> {
    this.lineage = await this.engine.newLineage(seed ?? randomSeed());
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
  /** Cut 4 §1: more rows than the vocabulary allows. A patch never evicts a row; the editor shows `5/4` and `send`
   *  and the forecast wait until the player removes one. The engine keeps its last valid set meanwhile. */
  get overBudget(): boolean { return this.rules.rows.length > this.vocab.max_rows; }
  rulesChanged(): void {
    this.persist();
    clearTimeout(this.fcTimer);
    for (const fn of this.rulesListeners) fn();
    if (this.overBudget) return;
    void this.engine.setRules(this.rules).catch((e) => console.warn("rules rejected", e));
    this.fcTimer = window.setTimeout(() => void this.emitForecast(), 250);
  }
  /** Fires on every rule edit (the camp gates `send` on `overBudget`); `onChange` is for lineage changes. */
  onRules(fn: () => void): () => void { this.rulesListeners.add(fn); return () => this.rulesListeners.delete(fn); }
  onForecast(fn: (f: Forecast) => void): () => void { this.fcListeners.add(fn); return () => this.fcListeners.delete(fn); }
  async emitForecast(): Promise<void> {
    if (!this.fcListeners.size) return;
    if (this.fcInFlight || this.offlineRunning) { this.fcDirty = true; return; }
    this.fcInFlight = true; this.fcDirty = false;
    try {
      const f = await this.busy(/* copy:label */ "forecast", () => this.engine.forecast());
      for (const fn of this.fcListeners) fn(f);
    } catch (e) { console.warn("forecast failed", e); }
    finally { this.fcInFlight = false; }
    if (this.fcDirty) await this.emitForecast();
  }
  /** Cut 5 §6: the set's name (≤ 12 chars; empty clears it to the default). It rides the RuleSet through `setRules`, so the
   *  engine save carries it and the core can quote it in the chronicle. Renaming a set that is not active selects it first. */
  renameSet(i: number, name: string): void {
    if (i < 0 || i >= this.sets.length) return;
    const n = name.trim().slice(0, 12);
    this.sets[i].name = n || undefined;
    if (i !== this.active) { this.selectSet(i); return; }
    this.rulesChanged();
    this.emitChange();
  }
  selectSet(i: number): void {
    if (i === this.active || i < 0 || i >= this.sets.length) return;
    this.active = i;
    void this.engine.selectSet(i).catch((e) => console.warn("selectSet", e));
    this.rulesChanged();
    this.emitChange();
  }
  /** Inserts even when the set is full (Cut 4 §1: overflow is the player's decision, see `overBudget`). */
  insertRow(row: Row, at: number): number {
    const rows = this.rules.rows;
    const i = Math.max(0, Math.min(rows.length, at));
    rows.splice(i, 0, cloneRow(row));
    this.rulesChanged();
    return i;
  }
  /** A patch from the death screen or the report's stall: insert before `insert_at`, or (stall) replace / remove the row
   *  there. Returns the row to highlight in the camp (none after a removal). Replace never overflows; insert may. */
  applyPatch(p: Patch): number | undefined {
    const rows = this.rules.rows;
    if (p.remove) { if (p.insert_at < rows.length) rows.splice(p.insert_at, 1); this.rulesChanged(); return undefined; }
    if (p.replace && p.insert_at < rows.length) { rows[p.insert_at] = cloneRow(p.row); this.rulesChanged(); return p.insert_at; }
    return this.insertRow(p.row, p.insert_at);
  }
  async setRulesText(text: string): Promise<void> {
    const set = await this.engine.importRules(text);
    this.sets[this.active] = { rows: set.rows.map(cloneRow), name: this.sets[this.active]?.name };
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
  /** Cut 4 §9: a bought tactic card becomes a row `[card] <name>` at the end of the active set (the card only acts as
   *  a row: `{v:"tactic", a:<id>}`), so the player sees where it sits; over a full set that is an overflow decision. */
  async buy(id: string): Promise<boolean> {
    const ok = await this.mutate(() => this.engine.buy(id));
    if (ok && this.vocab.verbs.some((v) => v.v === "tactic" && v.a === id) && !this.rules.rows.some((r) => r.verb.v === "tactic" && r.verb.a === id)) {
      this.insertRow({ conds: [], verb: { v: "tactic", a: id } }, this.rules.rows.length);
      this.emitChange();
    }
    return ok;
  }
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
  private blob(): SaveBlob { return { v: 2, engine: this.lastSave, loadout: this.loadout, last_seen: Date.now(), runs: this.runsSeen }; }
  exportSave(): string { return JSON.stringify(this.blob()); }
  async importSave(text: string): Promise<boolean> {
    try {
      const b = JSON.parse(text) as SaveBlob;
      if (!b || typeof b.engine !== "string") return false;
      this.lineage = await this.engine.load(b.engine);
      this.loadout = b.loadout ?? []; this.runsSeen = b.runs ?? 0;
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
    if (screen.kind === "camp" && this.lineage.ended) screen = { kind: "ending" };
    this.view = screen;
    let m: Mounted;
    switch (screen.kind) {
      case "camp": m = renderCamp(this, screen.highlight); break;
      case "ending": m = renderEnding(this); break;
      case "watch": m = renderWatch(this); break;
      case "death": m = renderDeath(this, screen.death, screen.lost ?? []); break;
      case "report": m = renderReport(this, screen.report); break;
    }
    this.mounted = m;
    this.root.replaceChildren(m.el);
    this.root.dataset.screen = screen.kind;
    window.scrollTo(0, 0);
    this.persist();
    // dev `?speed=4`: press the matching HUD speed button as the run mounts (the watch owns its clock; 1 slow · 4 fast · 8 auto)
    if (screen.kind === "watch" && this.dev?.speed) {
      const want = ({ 1: "slow", 4: "fast", 8: "auto" } as Record<number, string>)[this.dev.speed] ?? `${this.dev.speed}×`;
      for (const b of m.el.querySelectorAll<HTMLButtonElement>("button.hud-btn")) if (b.textContent === want) { b.click(); break; }
    }
  }
}

/** Merge two offline reports (a then b): sums, unions in order, reel top 5, worst = deeper (ties: later). */
export function mergeReports(a: ReturnReport, b: ReturnReport): ReturnReport {
  const union = (x: string[], y: string[]): string[] => [...new Set([...x, ...y])];
  // `rank 1 … rank 9` and `fighter L2 … L5` collapse to the highest of each ladder
  const collapseBests = (xs: string[]): string[] => {
    const top = new Map<string, [number, string]>(); const out: string[] = [];
    for (const x of xs) {
      const m = /^(rank|D|(\w+) L)(\d+)$/.exec(x);
      if (!m) { out.push(x); continue; }
      const key = m[1]; const n = Number(m[3]);
      if (!top.has(key) || top.get(key)![0] < n) top.set(key, [n, x]);
    }
    return [...out, ...[...top.values()].map(([, x]) => x)];
  };
  const deaths = new Map<string, number>();
  for (const d of [...a.deaths, ...b.deaths]) deaths.set(d.cause, (deaths.get(d.cause) ?? 0) + d.n);
  const salv = new Map<string, { n: number; gold: number }>();
  for (const s of [...a.salvaged, ...b.salvaged]) { const m = salv.get(s.kind) ?? { n: 0, gold: 0 }; m.n += s.n; m.gold += s.gold; salv.set(s.kind, m); }
  const worst = !a.worst_death ? b.worst_death : !b.worst_death ? a.worst_death : b.worst_death.depth >= a.worst_death.depth ? b.worst_death : a.worst_death;
  // Cut 2 fields: the report picks its layout by presence, so they stay undefined only when both sides lack them
  const sum = (x?: number, y?: number): number | undefined => x === undefined && y === undefined ? undefined : (x ?? 0) + (y ?? 0);
  const cat = (x?: string[], y?: string[]): string[] | undefined => x === undefined && y === undefined ? undefined : [...(x ?? []), ...(y ?? [])];
  return {
    rested_s: sum(a.rested_s, b.rested_s), banked: sum(a.banked, b.banked), returned: sum(a.returned, b.returned),
    bones_found: cat(a.bones_found, b.bones_found),
    elapsed_s: a.elapsed_s + b.elapsed_s, runs: a.runs + b.runs, sampled: a.sampled || b.sampled,
    learned: union(a.learned, b.learned), bests: collapseBests(union(a.bests, b.bests)),
    found: [...a.found, ...b.found],
    deaths: [...deaths].map(([cause, n]) => ({ cause, n })).sort((x, y) => y.n - x.n),
    pending: b.pending,                                   // decisions waiting now (a state, not a delta)
    stall: b.stall,                                       // likewise: the window is on the game, the last slice knows
    reel: [...a.reel, ...b.reel].sort((x, y) => y.score - x.score).slice(0, 5),
    marks_earned: a.marks_earned + b.marks_earned, worst_death: worst, live: b.live,
    tamed: [...a.tamed, ...b.tamed], hatched: [...a.hatched, ...b.hatched], lost: [...a.lost, ...b.lost],
    xp: { class: b.xp.class, gained: a.xp.gained + b.xp.gained, level_ups: a.xp.level_ups + b.xp.level_ups },
    salvaged: [...salv].map(([kind, v]) => ({ kind, ...v })),
    renown: { gained: a.renown.gained + b.renown.gained, rank: b.renown.rank, ranks_up: a.renown.ranks_up + b.renown.ranks_up },
  };
}

export const cloneRow = (r: Row): Row => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb } });
export const cloneSet = (s: RuleSet): RuleSet => ({ rows: s.rows.map(cloneRow), name: s.name });

/** `dev` is non-null in dev builds or with `?dev=1` (main.ts): boot options plus `window.__riddle` for inspection
 *  (`__riddle.screen`, `__riddle.text()`, `__riddle.engineBusy`, `__riddle.booted`, and the App itself). */
export function start(dev: DevOptions | null = null): void {
  const root = document.getElementById("app") ?? document.body.appendChild(document.createElement("div"));
  root.id = "app";
  const app = new App(root, dev);
  if (dev) (window as unknown as { __riddle: App }).__riddle = app;
  void app.boot().catch((e) => console.error("boot failed", e));
  if (import.meta.env.PROD && "serviceWorker" in navigator) {
    window.addEventListener("load", () => { navigator.serviceWorker.register("/sw.js").catch(() => { /* offline-first is best effort */ }); });
  }
}
