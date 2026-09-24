// State machine: camp ⇄ watch ⇄ death ⇄ report. Owns the engine proxy (wasm in a worker, or the fake),
// the editing copy of the three saved sets, and persistence.
import type { AsyncEngine, ComboHit, Death, Forecast, Lineage, Patch, ReturnReport, Row, RowOrigin, RuleSet, SupplyEntry, UnlockInfo, Vocabulary } from "./engine/types";
import { combosIn, isCardRow, isFreeSupply, ownRowCount } from "./ui/tokens";
import { selectEngine, type EngineKind } from "./engine/index";
import { readBlob, writeBlob, clearBlob, randomSeed, type SaveBlob } from "./store";
import { renderCamp } from "./ui/camp";
import { renderWatch } from "./ui/watch";
import { renderDeath } from "./ui/death";
import { renderReport } from "./ui/report";
import { renderEnding } from "./ui/ending";
import { closeAllSheets, onEscapeIdle } from "./ui/sheet";
import { lastRun, type RunLog } from "./ui/runlog";
import { showBusy } from "./ui/progress";
import { audio } from "./audio";
import { applySkin } from "./ui/skin";

export type Screen =
  | { kind: "camp"; highlight?: number }
  | { kind: "watch" }
  | { kind: "death"; death: Death; lost?: string[]; kept?: boolean }   // kept: an old death opened from the chronicle (Cut 9 §7); Escape leads back to the camp
  | { kind: "report"; report: ReturnReport; absence?: boolean }   // absence: Cut 10 §3, the tiles fade in (the merged report is complete)
  | { kind: "ending" };

export type Mounted = { el: HTMLElement; dispose?: () => void };

/** Dev-only boot options (parsed from the URL in main.ts; `tools/dev.sh` + `tools/playtest.mjs`). */
export type DevOptions = {
  seed?: number;      // new lineage with this seed on a fresh boot (ignored while a save exists, unless `fresh`)
  fresh?: boolean;    // clear the save first
  absent?: number;    // seconds: treat last_seen as that far back, so the offline report runs
  rules?: string;     // rule-set text for `importRules`, applied to the active set before anything else
  speed?: number | string;   // watch mode to press on entering a run: `fights` (the default) | `fast`; numbers map 1 · 4 · 8 → fast, 16 → fights (Cut 10 §1)
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
const REFINE_MS = 2000;
const SLOWDOWNS_KEY = "riddle.slowdowns";
const EDITING_KEY = "riddle.editing";
function readSlowdowns(): boolean { try { return localStorage.getItem(SLOWDOWNS_KEY) !== "0"; } catch { return true; } }

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
  /** Cut 6 §9: the quiet second pass (100 sims) 2 s after a paint with the rules unchanged; off once the engine lacks it. */
  private refineTimer = 0;
  private refineSeq = 0;
  private refineOff = false;
  private fcListeners = new Set<(f: Forecast) => void>();
  private changeListeners = new Set<() => void>();
  private rulesListeners = new Set<() => void>();
  private shelfListeners = new Set<() => void>();
  private shelfFull: boolean | null = null;
  private rulesSeq = 0;
  private offlineRunning = false;
  /** Runs seen by this client (the wire Lineage has no run counter); persisted in the blob. */
  runsSeen = 0;
  /** The last chosen watch mode; the next watch starts in it (persisted in the blob — QA on e0f87e7: "`fast` chosen in run 3
   *  was not remembered"). */
  watchMode: "fights" | "fast" = "fights";
  /** Cut 17 §2: the camp's tablets carry the editor (chips, ▲▼, ×) while on; off, each row is one carved tablet (the targets'
   *  camp). Off by default; the `edit` tile toggles it, a tap on a tablet or the death's `edit` turns it on. A per-viewer
   *  preference (localStorage `riddle.editing`). */
  get editing(): boolean { try { return localStorage.getItem(EDITING_KEY) === "1"; } catch { return this.editingMem; } }
  set editing(on: boolean) { this.editingMem = on; try { localStorage.setItem(EDITING_KEY, on ? "1" : "0"); } catch { /* private mode: this session only */ } }
  private editingMem = false;
  /** Cut 6 §6: the unlock catalogue as last fetched by the camp; a card's rows back the editor's `[card]` sheet. */
  unlockCat: UnlockInfo[] = [];
  /** Cut 13 §5: the last forecast painted (the refine when it landed), so a `dice` death can say what it said for that depth. */
  lastForecast: Forecast | null = null;
  /** Cut 14: the watch's dynamic slowdowns (the fight frame's 2× / 4×, the near and scene holds); off, the clock runs the mode's
   *  flat rate. Persisted in localStorage like `mute` (`riddle.slowdowns`), on by default; the settings sheet toggles it. */
  slowdowns = readSlowdowns();
  setSlowdowns(on: boolean): void { this.slowdowns = on; try { localStorage.setItem(SLOWDOWNS_KEY, on ? "1" : "0"); } catch { /* private mode: not persisted */ } }
  /** Cut 14 §4: how often each row of the active set fired, as last measured — the watched run's `rule` events (ui/watch.ts) or
   *  the absence's `R1 fired n of m runs` lines (ui/report.ts); the death screen's patch chip names the least-fired row on a
   *  full set (Cut 15 §3: the drop sheet's counts and its least-fired mark). Cleared by any rule edit: the counts are the set that ran. */
  rowFires: number[] | null = null;
  /** Cut 15 §3: the denominator of `rowFires` (the absence's `of 16 runs`); undefined = the sum of the counts (a watched run's fires). */
  rowFiresOf: number | undefined = undefined;
  /** The supply catalogue as last fetched by the camp: the shop paints from it at once on the next camp, then refetches (the
   *  worker answers in order, so a fetch behind a forecast is seconds away — QA B on 952e306: "the shop chips are gone"). */
  supplyCat: SupplyEntry[] = [];
  cardRows(id: string): Row[] | undefined { return this.unlockCat.find((u) => u.id === id)?.rows; }
  /** Cut 7 §2: row origins from the save blob, consumed by the first `adoptSets` (the engine's sets carry none). */
  private savedOrigins: string[][] | null = null;
  /** Cut 7 §2: the rows of the active set that are the player's own (`yours: 3 of 5 rows`). Cut 12 §1: card rows never count. */
  playerRows(): number { return this.rules.rows.filter((r) => !isCardRow(r) && (r.origin ?? "player") === "player").length; }
  /** Cut 12 §1: the rows `max_rows` caps — every row that is not a card's (`{v:"tactic"}`). */
  ownRows(): number { return ownRowCount(this.rules.rows); }
  cardRowCount(): number { return this.rules.rows.length - this.ownRows(); }
  /** Cut 12 §1: the set has no free own row (`+1 row` opens; `5/4 · drop one` past it). */
  get rowsFull(): boolean { return this.ownRows() >= this.vocab.max_rows; }
  /** Cut 8B §1: the active set's combos as the editor sees them now (the vocabulary's table over the editing copy). */
  combos(): ComboHit[] { return combosIn(this.rules.rows, this.vocab?.combos); }
  /** Cut 11 §2: the last watched run's event log (ui/runlog.ts), which the death screen's chain links replay. */
  runLog(): RunLog | null { return lastRun(); }

  constructor(root: HTMLElement, dev: DevOptions | null = null) {
    this.root = root; this.dev = dev;
    // QA on 952e306 ("old death screen, no back/close, Escape inert"): Escape with no sheet open leaves a kept death for the camp
    onEscapeIdle(() => { if (this.view.kind === "death" && this.view.kept) this.go({ kind: "camp" }); });
  }

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
        this.watchMode = blob.watch === "fast" ? "fast" : "fights";
        this.savedOrigins = blob.origins ?? null;
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
      this.go({ kind: "report", report, absence: true });
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

  /** `runOfflineQuick` in 30-minute slices, reports merged client-side under the bare `offline` bar (Cut 10 §3: no running
   *  numbers — a partial count read as a broken report; the tiles fade in once the merged report is complete); then one
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
  /** Take the editing copies from the engine's lineage (the engine is the source of truth).
   *  Cut 7 §2: the engine's rows carry no origin (unless the core tags them), so each adopted row inherits the origin of
   *  the row it replaces — by index when the text matches, else the first unclaimed row with that text — from the
   *  previous editing copy, or from the save blob on the first adopt. A row nothing accounts for is the shipped preset
   *  on the first adopt (a fresh lineage, a pre-Cut 7 save) and the player's afterwards. */
  private adoptSets(): void {
    const prev = this.sets, saved = this.savedOrigins; this.savedOrigins = null;
    const first = !prev.length;
    this.sets = (this.lineage.sets ?? []).map((s, i) => {
      const set = cloneSet(s);
      if (first) { set.rows.forEach((r, j) => { r.origin ??= asOrigin(saved?.[i]?.[j]) ?? "preset"; }); return set; }
      const was = prev[i]?.rows ?? []; const claimed = new Set<number>();
      set.rows.forEach((r, j) => {
        if (r.origin) return;
        const key = rowKey(r);
        const k = was[j] && !claimed.has(j) && rowKey(was[j]) === key ? j : was.findIndex((w, x) => !claimed.has(x) && rowKey(w) === key);
        if (k >= 0) claimed.add(k);
        r.origin = (k >= 0 ? was[k].origin : undefined) ?? "player";
      });
      return set;
    });
    while (this.sets.length < SETS) this.sets.push(this.sets[0] ? cloneSet(this.sets[0]) : { rows: [] });
    this.sets = this.sets.slice(0, SETS);
    this.active = this.lineage.active_set ?? 0;
    if (this.active < 0 || this.active >= SETS) this.active = 0;
  }

  get rules(): RuleSet { return this.sets[this.active]; }

  // --- rules ---
  /** Cut 4 §1: more rows than the vocabulary allows. A patch never evicts a row; the editor shows `5/4` and `send`
   *  and the forecast wait until the player removes one. The engine keeps its last valid set meanwhile.
   *  Cut 12 §1: own rows against `max_rows`; card rows sit outside the cap. */
  get overBudget(): boolean { return this.ownRows() > this.vocab.max_rows; }
  rulesChanged(): void {
    this.rowFires = null; this.rowFiresOf = undefined;   // Cut 14 §4: the counts were the set that ran
    this.persist();
    clearTimeout(this.fcTimer); clearTimeout(this.refineTimer); this.refineSeq++;
    for (const fn of this.rulesListeners) fn();
    if (this.overBudget) return;
    const seq = ++this.rulesSeq;
    void this.engine.setRules(this.rules).then(() => { if (seq === this.rulesSeq) this.shelfCheck(); }).catch((e) => console.warn("rules rejected", e));
    this.fcTimer = window.setTimeout(() => void this.emitForecast(), 250);
  }
  /** Cut 12 §6: the unlock shelf's `+1 row ⊘ fill rows` is the engine's read of its own set, so it repaints once a rule edit
   *  crosses `max_rows` — after `setRules` resolved (the rules listeners fire before the engine call). */
  private shelfCheck(): void {
    const full = this.rowsFull;
    if (full === this.shelfFull) return;
    this.shelfFull = full;
    for (const fn of this.shelfListeners) fn();
  }
  /** Fires on every rule edit (the camp gates `send` on `overBudget`); `onChange` is for lineage changes. */
  onRules(fn: () => void): () => void { this.rulesListeners.add(fn); return () => this.rulesListeners.delete(fn); }
  /** Cut 12 §6: fires after the engine took a set that crossed `max_rows` (the unlock shelf refetches its catalogue). */
  onShelf(fn: () => void): () => void { this.shelfFull = this.rowsFull; this.shelfListeners.add(fn); return () => this.shelfListeners.delete(fn); }
  onForecast(fn: (f: Forecast) => void): () => void { this.fcListeners.add(fn); return () => this.fcListeners.delete(fn); }
  async emitForecast(): Promise<void> {
    if (!this.fcListeners.size) return;
    if (this.fcInFlight || this.offlineRunning) { this.fcDirty = true; return; }
    this.fcInFlight = true; this.fcDirty = false;
    try {
      const f = await this.busy(/* copy:label */ "forecast", () => this.engine.forecast());
      this.lastForecast = f;
      for (const fn of this.fcListeners) fn(f);
      this.scheduleRefine();
    } catch (e) { console.warn("forecast failed", e); }
    finally { this.fcInFlight = false; }
    if (this.fcDirty) await this.emitForecast();
  }
  /** Cut 6 §9: after the forecast paints and the rules stay unchanged for REFINE_MS, `forecastRefine` (100 sims) repaints
   *  quietly (no progress bar). A rule edit or a fresh forecast cancels it; an engine without it is asked once. */
  private scheduleRefine(): void {
    clearTimeout(this.refineTimer);
    if (this.refineOff || !this.engine.forecastRefine) return;
    const seq = ++this.refineSeq;
    this.refineTimer = window.setTimeout(async () => {
      if (seq !== this.refineSeq || this.fcInFlight || this.offlineRunning || this.overBudget) return;
      try {
        const f = await this.engine.forecastRefine!();
        if (seq !== this.refineSeq) return;
        this.lastForecast = f;
        for (const fn of this.fcListeners) fn(f);
      } catch (e) { this.refineOff = true; console.warn("forecastRefine unavailable", e); }
    }, REFINE_MS);
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
  /** Inserts even when the set is full (Cut 4 §1: overflow is the player's decision, see `overBudget`).
   *  Cut 7 §2: a row without an origin is the player's. */
  insertRow(row: Row, at: number, origin: RowOrigin = row.origin ?? "player"): number {
    const rows = this.rules.rows;
    const i = Math.max(0, Math.min(rows.length, at));
    rows.splice(i, 0, { ...cloneRow(row), origin });
    this.rulesChanged();
    return i;
  }
  /** A patch from the death screen or the report's stall: insert before `insert_at`, or (stall) replace / remove the row
   *  there. Returns the row to highlight in the camp (none after a removal). Replace never overflows; insert may. */
  applyPatch(p: Patch): number | undefined {
    const rows = this.rules.rows;
    if (p.remove) { if (p.insert_at < rows.length) rows.splice(p.insert_at, 1); this.rulesChanged(); return undefined; }
    if (p.replace && p.insert_at < rows.length) { rows[p.insert_at] = { ...cloneRow(p.row), origin: p.row.origin ?? "patch" }; this.rulesChanged(); return p.insert_at; }
    return this.insertRow(p.row, p.insert_at, p.row.origin ?? "patch");
  }
  /** Cut 14 §4: a patch onto a full set — row `drop` goes and the patch row lands where it was measured (`insert_at`, one up
   *  when the dropped row sat above it), so the set never crosses `max_rows` (rater S: "an offered patch pushed me to `6/5 ·
   *  drop one` with no warning"). Returns the patch row's index. */
  applyPatchOver(p: Patch, drop: number): number {
    const rows = this.rules.rows;
    if (drop >= 0 && drop < rows.length) rows.splice(drop, 1);
    return this.insertRow(p.row, drop >= 0 && drop < p.insert_at ? p.insert_at - 1 : p.insert_at, p.row.origin ?? "patch");
  }
  async setRulesText(text: string): Promise<void> {
    const set = await this.engine.importRules(text);
    this.sets[this.active] = { rows: set.rows.map((r) => ({ ...cloneRow(r), origin: r.origin ?? "player" })), name: this.sets[this.active]?.name };
    this.rulesChanged();
    this.emitChange();
  }

  // --- lineage ---
  async refresh(): Promise<void> { this.lineage = await this.engine.lineage(); this.vocab = await this.engine.vocabulary(); this.emitChange(); }
  onChange(fn: () => void): () => void { this.changeListeners.add(fn); return () => this.changeListeners.delete(fn); }
  private emitChange(): void { for (const fn of this.changeListeners) fn(); }
  /** After an engine call that returned a new Lineage: repaint from it at once, then refresh vocab (a second repaint when it
   *  changed), persist, re-forecast. The vocabulary round-trip queues behind whatever the worker is on (a forecast is ~1.5 s),
   *  and the strip used to wait for it: the `◆` read one purchase behind on four buys in a row (QA on 50bb162). */
  async afterLineage(): Promise<void> {
    this.persist(); this.emitChange();
    const before = JSON.stringify(this.vocab);
    this.vocab = await this.engine.vocabulary();
    if (JSON.stringify(this.vocab) !== before) this.emitChange();
    this.rulesChanged();
  }
  /** Runs an engine call that returns a Lineage and adopts it. Errors (unaffordable, locked) are swallowed after a warn. */
  async mutate(fn: () => Promise<Lineage>): Promise<boolean> {
    try { this.lineage = await fn(); } catch (e) { console.warn("engine refused", e); return false; }
    await this.afterLineage();
    return true;
  }
  /** Cut 4 §9: a bought tactic card becomes a row `[card] <name>` in the active set (the card only acts as a row:
   *  `{v:"tactic", a:<id>}`), so the player sees where it sits. Cut 12 §1: it sits where it acts — at the catalogue's
   *  `insert_at` when the engine sends one, else before the set's engagement row (the first `attack` / `shoot`), else the
   *  end; card rows sit outside `max_rows`, so a card never overflows the set. */
  async buy(id: string, gold = false): Promise<boolean> {
    const u = this.unlockCat.find((x) => x.id === id);
    const at = u?.insert_at;
    // Cut 15 §2: `gold` pays the catalogue's gold price instead of marks (the returned lineage repaints the header's $ and ◆)
    const ok = await this.mutate(() => (gold ? this.engine.buyUnlockGold!(id) : this.engine.buy(id)));
    if (ok) audio.cue("unlock");   // Cut 10 §4
    if (ok && this.vocab.verbs.some((v) => v.v === "tactic" && v.a === id) && !this.holdsCard(id)) { this.insertCard(id, at); this.emitChange(); }
    // a verb unlock's `reach +21%` was measured with its canonical row at the top (the catalogue sends `rows` + `insert_at`
    // for it); the buy inserts that row so the number holds (QA on e0f87e7: "bought, forecast identical")
    // …only while the set has room for it: over the cap it is the sheet's row to add by hand (rater R on 39def99: `6/5 · drop one`
    // after a verb purchase)
    else if (ok && u?.rows?.length === 1 && u.rows[0].verb.v !== "tactic" && u.rows[0].verb.v !== "auto" && u.insert_at !== undefined && !this.rowsFull && !this.rules.rows.some((r) => sameRowShape(r, u.rows![0]))) {
      this.insertRow(cloneRow(u.rows[0]), u.insert_at, "patch"); this.emitChange();
    }
    return ok;
  }
  /** Cut 12 §1: does the active set hold this card's row? */
  holdsCard(id: string): boolean { return this.rules.rows.some((r) => isCardRow(r) && r.verb.a === id); }
  /** A card's row into the active set where it acts: at `at` (the catalogue's `insert_at`, which the core sends only while the
   *  card is unowned), else before the engagement row — the client's mirror of the core's `card_insert_at` over the editing copy.
   *  Also the owned card's `insert` after the player dropped its row (QA on 952e306: "no way to re-insert a dropped card (◆3
   *  spent)"). Returns the row's index. */
  insertCard(id: string, at?: number): number {
    return this.insertRow({ conds: [], verb: { v: "tactic", a: id } }, at ?? engagementRow(this.rules.rows), "card");
  }
  /** Cut 12 §6: one supply off the shelf. An engine without `dropSupply` clears the shelf and rebuys the other bought
   *  lines in order (a free line — the kennel's leash — comes back at the next exit). */
  async dropSupply(id: number): Promise<boolean> {
    const picks = this.lineage.supplies ?? [];
    const it = picks.find((p) => p.id === id); if (!it) return false;
    try { this.lineage = await this.engine.dropSupply!(id); await this.afterLineage(); return true; }   // the proxy always has it; an engine without it rejects
    catch (e) { console.warn("dropSupply unavailable, clear + rebuy", e); }
    const rest = picks.filter((p) => p.id !== id && !isFreeSupply(this.lineage, p)).map((p) => p.kind);
    return this.mutate(async () => { let L = await this.engine.clearSupplies(); for (const k of rest) L = await this.engine.buySupply(k); return L; });
  }
  setClass(cls: string): Promise<boolean> { return this.mutate(() => this.engine.setClass(cls)); }
  setLoadout(ids: number[]): void { this.loadout = ids; void this.engine.loadout(ids); this.persist(); this.emitChange(); }
  async resetLineage(): Promise<void> {
    clearBlob();
    await this.fresh();
    this.sets = [];   // Cut 7 §2: a fresh lineage's rows are the preset again
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
  private blob(): SaveBlob { return { v: 2, engine: this.lastSave, loadout: this.loadout, last_seen: Date.now(), runs: this.runsSeen, origins: this.sets.map((s) => s.rows.map((r) => r.origin ?? "player")), watch: this.watchMode }; }
  exportSave(): string { return JSON.stringify(this.blob()); }
  async importSave(text: string): Promise<boolean> {
    try {
      const b = JSON.parse(text) as SaveBlob;
      if (!b || typeof b.engine !== "string") return false;
      this.lineage = await this.engine.load(b.engine);
      this.loadout = b.loadout ?? []; this.runsSeen = b.runs ?? 0; this.watchMode = b.watch === "fast" ? "fast" : "fights";
      this.savedOrigins = b.origins ?? null; this.sets = [];   // Cut 7 §2: the imported blob's origins, not the old sets'
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
      case "death": m = renderDeath(this, screen.death, screen.lost ?? [], !!screen.kept); break;
      case "report": m = renderReport(this, screen.report, screen.absence); break;
    }
    this.mounted = m;
    this.root.replaceChildren(m.el);
    this.root.dataset.screen = screen.kind;
    document.body.classList.toggle("framed", screen.kind !== "ending");   // Cut 17: sheets unfold above the console
    window.scrollTo(0, 0);
    this.persist();
    // dev `?speed=fast|fights|N`: press the matching HUD mode button as the run mounts (the watch owns its clock; Cut 10 §1:
    // `fights` is the default, `fast` is the old auto; 1 · 4 · 8 map to fast, 16 to fights)
    if (screen.kind === "watch" && this.dev?.speed) {
      const sp = this.dev.speed;
      const want = typeof sp === "number" ? ({ 1: "fast", 4: "fast", 8: "fast", 16: "fights" } as Record<number, string>)[sp] ?? "fights" : sp;
      for (const b of m.el.querySelectorAll<HTMLButtonElement>("button.hud-btn")) if (b.textContent === want) { b.click(); break; }
    }
  }
}

/** `R1 fired 0 of 15 runs: …` lines are per slice (Cut 9); the same row's counts add up across an absence. */
function mergePending(a: string[], b: string[]): string[] {
  const re = /^(R\d+) fired (\d+) of (\d+) runs(.*)$/;
  const sums = new Map<string, [number, number, string]>();
  for (const line of [...a, ...b]) { const m = re.exec(line); if (!m) continue; const cur = sums.get(m[1]) ?? [0, 0, m[4]]; sums.set(m[1], [cur[0] + Number(m[2]), cur[1] + Number(m[3]), m[4]]); }
  const out: string[] = [];
  for (const line of b) { const m = re.exec(line); if (!m) { out.push(line); continue; } const [f, n, rest] = sums.get(m[1])!; out.push(`${m[1]} fired ${f} of ${n} runs${rest}`); }
  return out;
}
/** Merge two offline reports (a then b): sums, unions in order, reel top 5, worst = deeper (ties: later). */
/** The same conds (key, tag, number) and verb: a row the set already carries. */
function sameRowShape(a: Row, b: Row): boolean {
  return a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && a.conds.length === b.conds.length && a.conds.every((c, i) => c.k === b.conds[i].k && (c.t ?? "") === (b.conds[i].t ?? "") && c.n === b.conds[i].n);
}

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
  // Cut 13 §3: the night's purchases add up across slices like the salvage; absent only when both sides lack the field
  const spent = a.spent === undefined && b.spent === undefined ? undefined : new Map<string, { n: number; gold: number }>();
  if (spent) for (const s of [...(a.spent ?? []), ...(b.spent ?? [])]) { const m = spent.get(s.kind) ?? { n: 0, gold: 0 }; m.n += s.n; m.gold += s.gold; spent.set(s.kind, m); }
  const worst = !a.worst_death ? b.worst_death : !b.worst_death ? a.worst_death : b.worst_death.depth >= a.worst_death.depth ? b.worst_death : a.worst_death;
  // Cut 2 fields: the report picks its layout by presence, so they stay undefined only when both sides lack them
  const sum = (x?: number, y?: number): number | undefined => x === undefined && y === undefined ? undefined : (x ?? 0) + (y ?? 0);
  const cat = <T,>(x?: T[], y?: T[]): T[] | undefined => x === undefined && y === undefined ? undefined : [...(x ?? []), ...(y ?? [])];
  return {
    rested_s: sum(a.rested_s, b.rested_s), banked: sum(a.banked, b.banked), returned: sum(a.returned, b.returned), stalled: sum(a.stalled, b.stalled),
    bones_found: cat(a.bones_found, b.bones_found),
    picked: b.picked ?? a.picked,                          // Cut 16 §1: a state — the last slice knows
    exits: cat(a.exits, b.exits),                          // Cut 6 §1: one ledger line per exit
    elapsed_s: a.elapsed_s + b.elapsed_s, runs: a.runs + b.runs, sampled: a.sampled || b.sampled,
    deepest: a.deepest === undefined && b.deepest === undefined ? undefined : Math.max(a.deepest ?? 0, b.deepest ?? 0),
    gold: a.gold || b.gold ? { home: (a.gold?.home ?? 0) + (b.gold?.home ?? 0), salvage: (a.gold?.salvage ?? 0) + (b.gold?.salvage ?? 0), wake: (a.gold?.wake ?? 0) + (b.gold?.wake ?? 0), spent: (a.gold?.spent ?? 0) + (b.gold?.spent ?? 0) } : undefined,
    learned: union(a.learned, b.learned), bests: collapseBests(union(a.bests, b.bests)),
    found: [...a.found, ...b.found],
    deaths: [...deaths].map(([cause, n]) => ({ cause, n })).sort((x, y) => y.n - x.n),
    pending: mergePending(a.pending, b.pending),          // state lines from the last slice; `R1 fired n of m runs` summed across slices
    stall: b.stall,                                       // likewise: the window is on the game, the last slice knows
    reel: [...a.reel, ...b.reel].sort((x, y) => y.score - x.score).slice(0, 5),
    marks_earned: a.marks_earned + b.marks_earned, worst_death: worst, live: b.live,
    tamed: [...a.tamed, ...b.tamed], hatched: [...a.hatched, ...b.hatched], lost: [...a.lost, ...b.lost],
    xp: { class: b.xp.class, gained: a.xp.gained + b.xp.gained, level_ups: a.xp.level_ups + b.xp.level_ups },
    salvaged: [...salv].map(([kind, v]) => ({ kind, ...v })),
    spent: spent ? [...spent].map(([kind, v]) => ({ kind, ...v })) : undefined,
    renown: { gained: a.renown.gained + b.renown.gained, rank: b.renown.rank, ranks_up: a.renown.ranks_up + b.renown.ranks_up },
  };
}

/** Cut 12 §1: the set's engagement row — the first `attack` / `shoot` — where a bought card goes; the end when there is none. */
export function engagementRow(rows: Row[]): number { const i = rows.findIndex((r) => r.verb.v === "attack" || r.verb.v === "shoot"); return i < 0 ? rows.length : i; }
export const cloneRow = (r: Row): Row => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb }, ...(r.origin ? { origin: r.origin } : {}) });
/** Cut 7 §2: a row's identity for origin carry-over (tokens only, never the origin). */
const rowKey = (r: Row): string => `${r.conds.map((c) => `${c.k}|${c.n ?? ""}|${c.t ?? ""}`).join(" ")} → ${r.verb.v}|${r.verb.a ?? ""}`;
const asOrigin = (o: unknown): RowOrigin | undefined => (o === "preset" || o === "patch" || o === "card" || o === "player" ? o : undefined);
export const cloneSet = (s: RuleSet): RuleSet => ({ rows: s.rows.map(cloneRow), name: s.name });

/** `dev` is non-null in dev builds or with `?dev=1` (main.ts): boot options plus `window.__riddle` for inspection
 *  (`__riddle.screen`, `__riddle.text()`, `__riddle.engineBusy`, `__riddle.booted`, and the App itself). */
export function start(dev: DevOptions | null = null): void {
  const root = document.getElementById("app") ?? document.body.appendChild(document.createElement("div"));
  root.id = "app";
  applySkin();   // Cut 17: the frames packed in web/public/ui (tools/ui-skin.py); absent ones keep the flat CSS
  const app = new App(root, dev);
  audio.arm();   // Cut 10 §4: the WebAudio context opens on the first gesture
  if (dev) (window as unknown as { __riddle: App }).__riddle = app;
  // Cut 14 §4: the cue log is readable on every build (a rater assesses sound on the cohort build); the App stays dev-only
  Object.defineProperty(window, "__audio", { value: audio, writable: false, configurable: true });
  void app.boot().catch((e) => console.error("boot failed", e));
  if (import.meta.env.PROD && "serviceWorker" in navigator) {
    window.addEventListener("load", () => { navigator.serviceWorker.register("/sw.js").catch(() => { /* offline-first is best effort */ }); });
  }
}
