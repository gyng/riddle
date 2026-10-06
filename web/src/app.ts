// State machine: camp ⇄ watch ⇄ death ⇄ report. Owns the engine proxy (wasm in a worker, or the fake),
// the editing copy of the three saved sets, and persistence.
import type { RowWhy } from "./engine/types";
import type { AsyncEngine, ComboHit, Death, Highlight, Forecast, ForecastMove, ForecastVs, Lineage, Patch, ReturnReport, Row, RowOrigin, RuleSet, SupplyEntry, UnlockInfo, Vocabulary, VsMove } from "./engine/types";
import { combosIn, isCardRow, isFreeSupply, ownRowCount, setWhyGloss } from "./ui/tokens";
import { selectEngine, type EngineKind } from "./engine/index";
import { readBlob, writeBlob, clearBlob, randomSeed, type SaveBlob } from "./store";
import { renderCamp } from "./ui/camp";
import { renderWatch } from "./ui/watch";
import { renderDeath } from "./ui/death";
import { renderReport } from "./ui/report";
import { mergeWorkers } from "./ui/works";   // Cut 30.5
import { mergeClassXp } from "./ui/class-xp";
import { renderEnding } from "./ui/ending";
import { closeAllSheets, onEscapeIdle } from "./ui/sheet";
import { lastRun, type RunLog } from "./ui/runlog";
import { showBusy } from "./ui/progress";
import { audio } from "./audio";
import { debugNote } from "./debug";
import { applySkin } from "./ui/skin";
import { initTips } from "./ui/tips";   // docs/TOOLTIPS.md: keyword tips
import { mergeMeters } from "./ui/meters";
import { basesOf, linSum, readSnap, rulesKey, sharesOf, stateLabel, stateTerms, writeSnap, type StateMove, type StateSnap } from "./ui/attrib";

export type Screen =
  | { kind: "camp"; highlight?: number }
  | { kind: "watch" }
  | { kind: "death"; death: Death; lost?: string[]; kept?: boolean; from?: { report: ReturnReport; absence?: boolean } }   // from: QA 308f045 (qaAD) — a verdict opened from a report leads back to it   // kept: an old death opened from the chronicle (Cut 9 §7); Escape leads back to the camp
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
/** RUNS_UI: the open app's clock asks the engine every LIVE_TICK_MS while a run is live; at home, once the rest is due (and at least every
 *  REST_SYNC_MS, so the core's rest and clock keep with the town's countdown) */
const LIVE_TICK_MS = 2000, REST_SYNC_MS = 60_000;
const SETS = 3;
const SAVE_DEBOUNCE_MS = 1000;
// runOffline is chunked so the progress label can count runs. `runOfflineQuick` skips the worst-death verdict
// (~3 s per slice; one `death(id)` at the end instead), so slices are a flat 30 min. On a stale wasm build
// without it, the full call is used with slices that grow with the absence (80 min for 8 h, 2 h cap).
const OFFLINE_SLICE_S = 30 * 60, OFFLINE_SLICE_MAX_S = 2 * 3600, OFFLINE_SLICES = 6;
/** Cut 28 §2: the quiet the camp keeps before the move's attribution is asked. */
const MOVE_QUIET_MS = 1500;
/** Cut 20 §3: an edit's forecast waits this long for the next edit (was 250 ms; the first paint is due ≤ 1 s after the edit). */
const FC_DEBOUNCE_MS = 30;   // Cut 24 §4: 30 ms (was 100) — a first pass ≤ 1 s in wasm needs the room
const SLOWDOWNS_KEY = "riddle.slowdowns";
const EDITING_KEY = "riddle.editing";
function readSlowdowns(): boolean { try { return localStorage.getItem(SLOWDOWNS_KEY) !== "0"; } catch { return true; } }

export class App {
  engine!: AsyncEngine;
  kind: EngineKind = "fake";
  version = "";
  /** RUNS_UI: the rest's countdown is anchored where the core's `rest_left_s` last moved (a re-read of the same value keeps the anchor) */
  get lineage(): Lineage { return this.lin_; }
  set lineage(L: Lineage) {
    const s = L?.rest_left_s ?? 0;
    if (!this.lin_ || s !== this.restAnchor.s || (this.lin_.live?.turn ?? 0) !== (L.live?.turn ?? 0)) this.restAnchor = { s, at: Date.now() };
    this.lin_ = L;
  }
  private lin_!: Lineage;
  private restAnchor = { s: 0, at: 0 };
  /** RUNS_UI: the hero's rest left now (seconds), counted down from the core's last word */
  restLeftS(): number { return Math.max(0, this.restAnchor.s - (Date.now() - this.restAnchor.at) / 1000); }
  /** RUNS_UI (docs/RUNS_UI.md §2): the open app's clock on the lineage — `advance(ms)` while the town (or a report, a death) is up: the rest
   *  runs out, the next run goes down, a run in flight plays on unwatched. The watch drives its own run (the clock pauses there); a hidden
   *  tab's time is the next tick's, or an absence (`backFromHidden`). Off under automation unless `?runs=1` (the old gates keep their
   *  meaning, as autodismiss does); `?runs=0` turns it off anywhere. */
  private runnerAt = 0;
  private runnerBusy = false;
  private ascensionBusy = false;
  private hiddenAt = 0;
  private liveListeners = new Set<() => void>();
  onLive(fn: () => void): () => void { this.liveListeners.add(fn); return () => this.liveListeners.delete(fn); }
  private emitLive(): void { for (const fn of this.liveListeners) fn(); }
  private watchRead: { view: Screen; selected: number | undefined; promise: Promise<void> } | null = null;
  /** Slots come from Rust after the send, without advancing the town clock.
   * Reuse the same read; a late reply cannot restore a departed watch/selection. */
  syncWatchLineage(): Promise<void> {
    const view = this.view, selected = this.lineage.selected_bloodline;
    if (view.kind !== "watch" || !this.lineage.hero_slots?.length) return Promise.resolve();
    if (this.watchRead?.view === view && this.watchRead.selected === selected) return this.watchRead.promise;
    const read = { view, selected, promise: Promise.resolve() };
    read.promise = this.engine.lineage().then((L) => {
      if (this.view !== view || this.lineage.selected_bloodline !== selected || L.selected_bloodline !== selected) return;
      this.lineage = L; this.emitLive();
    }).catch((e) => console.warn("watch lineage", e)).finally(() => { if (this.watchRead === read) this.watchRead = null; });
    this.watchRead = read;
    return read.promise;
  }
  get runnerOn(): boolean {
    const q = new URLSearchParams(location.search).get("runs");
    if (q === "0") return false;
    return q === "1" || !(typeof navigator !== "undefined" && navigator.webdriver);
  }
  /** one tick of the open app's clock (every second; the engine is asked every 2 s while a run is live, once a rest is due) */
  async runTick(force = false): Promise<void> {
    if (!this.booted || this.offlineRunning || this.offlineIncomplete || this.runnerBusy || !this.engine?.advance || (!force && !this.runnerOn)) return;
    const now = Date.now();
    if (this.view.kind === "watch") {
      this.runnerAt=now;
      if(this.lineage.hero_slots?.length&&!document.hidden){
        this.runnerBusy=true;
        try{await this.syncWatchLineage();}
        finally{this.runnerBusy=false;}
      }
      return;
    }   // the watch plays its own run; its time is not the town's
    if (document.hidden) return;                                       // (the time hidden is the next tick's, or an absence)
    const L = this.lineage, live = !!L.live && L.live.turn > 0;
    if (!live && L.tree?.waits && !L.hero_slots?.some(h=>h.state!=="waits")) { this.runnerAt = now; return; }      // before the scout the hero home waits: nothing runs
    const dt = now - (this.runnerAt || now);
    if (!force && !(live ? dt >= LIVE_TICK_MS : this.restLeftS() <= 0.25 || dt >= REST_SYNC_MS)) return;
    if (dt <= 0) { this.runnerAt = now; return; }
    this.runnerBusy = true;
    try {
      const r = await this.engine.advance(dt);
      this.runnerAt = now;
      const was = L.live?.run_id;
      if (r.ended.length || !!r.live !== live || (r.live && r.live.run_id !== was)) {
        this.lineage = await this.engine.lineage();   // a run ended or began: the town, the purse, the log
        this.emitChange();
      } else if (!live) this.lineage = await this.engine.lineage();   // the rest's sync: its countdown re-anchored, nothing repaints
      else this.lineage = this.lineage.hero_slots?.length ? await this.engine.lineage() : { ...this.lineage, live: r.live ?? null };
      this.emitLive();
    } catch (e) { console.warn("advance", e); this.runnerAt = now; }
    finally { this.runnerBusy = false; }
  }
  /** RUNS_UI: out of the watch mid-run (`town ↻`) — the town, and the lineage read again (the watch began with the one before the send: the
   *  run under way, the send by hand spent), so the lane shows him down there and the open app's clock takes the run on from there */
  leaveWatch(): void {
    this.go({ kind: "camp" });
    this.runnerAt = Date.now();
    void this.engine.lineage().then((L) => { this.lineage = L; this.emitChange(); this.emitLive(); }).catch(() => undefined);
  }
  /** RUNS_UI: back from a hidden tab — an absence's report when it was long enough to be one (the boot's rule), else the clock goes on */
  private async backFromHidden(): Promise<void> {
    const away = (Date.now() - this.hiddenAt) / 1000; this.hiddenAt = 0;
    if (!this.runnerOn || !this.booted || this.offlineRunning || this.offlineIncomplete || away < OFFLINE_MIN_S || this.view.kind === "watch") return;
    await this.absence(Math.floor(away));
  }
  /** An absence: the camp underneath, inert, while the batch runs; then the report (or the camp, when nothing ran before the scout). */
  async absence(elapsed: number): Promise<void> {
    this.markRan();   // Cut 28 §2: the absence sends the rules now
    this.offlineRunning = true;
    this.go({ kind: "camp" });
    const report = await this.runOfflineChunked(Math.floor(elapsed));
    await this.refresh();
    this.adoptSets();
    this.runnerAt = Date.now();
    // Cut 30.5: before the scout an absence with no send in flight ran nothing — the hero waited at home; no empty report
    if (report.runs === 0 && this.lineage.tree && !this.lineage.tree.auto_send) this.go({ kind: "camp" });
    else this.go({ kind: "report", report, absence: true });
    await this.flush();
  }
  private vocab_!: Vocabulary;
  get vocab(): Vocabulary { return this.vocab_; }
  /** Cut 23 §3: the vocabulary's `why_gloss` is every reason line's gloss on tap (the trace's chain rows read it through `whyGloss`). */
  set vocab(v: Vocabulary) { this.vocab_ = v; setWhyGloss(v?.why_gloss); }
  /** Editing copies of the engine's saved sets; `setRules` pushes the active one to the engine. */
  sets: RuleSet[] = [];
  active = 0;
  loadout: number[] = [];
  view: Screen = { kind: "camp" };
  /** Cut 30 §3: the last absence's report — the town walks its runs out of the mouth once, on the camp after it (ui/town.ts) */
  lastAbsence: { report: ReturnReport; played: boolean } | null = null;
  /** True once `boot()` has settled (after the offline batch, if any). Dev inspection. */
  booted = false;
  private root: HTMLElement;
  private dev: DevOptions | null;
  private mounted: Mounted | null = null;
  private saveTimer = 0;
  private lastSave = "";
  private fcTimer = 0;
  private fcInFlight = false;
  private fcAsk = 0;       // Cut 25 §4: the latest forecast asked (only it paints)
  private fcRunning = 0;   // … and how many are in flight
  private fcDirty = false;
  /** Explicit larger forecast request; reject replies after the camp state changes. */
  private refineSeq = 0;
  forecastRefining = false;
  private refineRequest: Promise<void> | null = null;
  private refineRequestKey = "";
  private fcListeners = new Set<(f: Forecast) => void>();
  private changeListeners = new Set<() => void>();
  private rulesListeners = new Set<() => void>();
  private shelfListeners = new Set<() => void>();
  private shelfFull: boolean | null = null;
  private rulesSeq = 0;
  private offlineRunning = false;
  private offlineIncomplete = false;
  /** Runs seen by this client (the wire Lineage has no run counter); persisted in the blob. */
  runsSeen = 0;
  /** The last chosen watch mode; the next watch starts in it (persisted in the blob — QA on e0f87e7: "`fast` chosen in run 3
   *  was not remembered"). */
  watchMode: "fights" | "fast" | "one" = "fights";
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
  /** QA a946e04: bumps with every forecast handed to the listeners (the shaft and the panel tag the one they painted: `data-fc`). */
  forecastSeq = 0;
  /** Cut 22 §3: the last edit's paired move (`vs last · D8 +6 · bank +4`) — the forecast's own `vs`, else `forecastVs(prev)` asked after
   *  the first paint; null until an edit's forecast painted, and cleared by the next edit (the shaft's line goes with it). */
  vs: ForecastVs | null = null;
  /** Cut 22 §3: the set the move is measured against — the one the last painted forecast measured when the edit came (a burst of
   *  edits before the next paint keeps the first's). Null after a set switch or a send: no edit to measure. */
  private vsBase: RuleSet | null = null;
  private fcRules: RuleSet | null = null;   // the set the last painted forecast measured
  private fcShadow: (number | null)[] = [];   // … and its shadowed rows (`Forecast.shadowed_by`)
  private vsBaseShadow: (number | null)[] = [];
  /** QA 778fa1b: the edit changed only rows that never fire — the base's live rows (not shadowed) are the set's live rows now. */
  private deadEdit(): boolean {
    const base = this.vsBase; if (!base || !this.shadow) return false;
    const live = (set: RuleSet, sh: (number | null)[]): RuleSet => ({ rows: set.rows.filter((_, i) => sh[i] === null || sh[i] === undefined), ...routeOf(set) });
    return sameSet(live(base, this.vsBaseShadow), live(this.rules, this.shadow));
  }
  /** Cut 27 §2: the set the camp's `vs sent` measures against (the set sent), while the rules now differ from it; else null. */
  sentSet(): RuleSet | null { return this.vsBase && !sameSet(this.vsBase, this.rules) ? cloneSet(this.vsBase) : null; }
  /** Cut 27 §1: the forecast painted for the rules now (the set a send takes), else null — the watch folds the floors it clears. */
  forecastOfRules(): Forecast | null { const f = this.lastForecast; return f && this.fcRules && sameSet(this.fcRules, this.rules) ? f : null; }
  private fcFresh = false;                  // a forecast painted since the last edit
  private editSeq = 0;
  private vsOff = false;
  private vsListeners = new Set<() => void>();
  onVs(fn: () => void): () => void { this.vsListeners.add(fn); return () => this.vsListeners.delete(fn); }
  private setVs(v: ForecastVs | null): void {
    if (v === this.vs) return;
    // Cut 28 §2: a refined move pairs the sent set on today's lineage (`base`) — after a run, its bases less the snap's are the state's part
    if (v?.refined && this.snap?.ran && this.vsBase && this.snap.rules === rulesKey(this.vsBase) && this.lineage) { const b = basesOf(v); if (Object.keys(b.shares).length) this.noteState(this.snap.rules, b.shares, b.pm, v.sims ?? 0); }
    // QA 778fa1b (qaU: `hp < 50% → attack lowest` added under `foes ≥ 1 → attack nearest` read `vs last · death −10`): an edit that only
    // touched rows that can never fire (`shadowed_by`, on both sides) moves nothing — the sims' wobble is not the edit's, it reads `≈`
    if (v && this.deadEdit()) v = flatVs(v);
    this.vs = v;
    for (const fn of this.vsListeners) { try { fn(); } catch (e) { console.warn("vs listener", e); } }
  }
  /** Cut 22 §3: after a forecast painted for the rules now — the move against the base set: the forecast's own `vs`, else the
   *  engine's `forecastVs(base)`, asked now (behind the paint in the worker's queue, never before it). */
  private measureVs(f: Forecast, asked: RuleSet, afterRefine = false): void {
    const base = this.vsBase;
    if (!base || sameSet(base, asked)) return;
    if (f.vs) { this.setVs(f.vs); return; }
    if (this.vsOff || !this.engine.forecastVs) return;
    const seq = this.editSeq;
    const ask = afterRefine && this.engine.forecastVsRefined ? this.engine.forecastVsRefined : !f.refined && this.engine.forecastVsEstimate ? this.engine.forecastVsEstimate : this.engine.forecastVs;
    // QA 778fa1b (qaU: `death −10` stayed from the first paint while the refined panel beside it read 22 → 27 %): asked again once the
    // explicit refinement lands; a first-pass answer never replaces a refined one that came back first
    void ask.call(this.engine, cloneSet(base)).then((v) => { if (seq === this.editSeq && v && !(this.vs?.refined && !v.refined)) this.setVs(v); })
      .catch((e) => { this.vsOff = true; console.warn("forecastVs unavailable", e); });
  }
  /** Cut 22 §3: a send (or a set switch) starts over — the set that runs is the next edit's base, and no move is shown. */
  /** QA 778fa1b (qaU: "compares to the previous edit, not the last run"; qaV: after a reorder "last" switched from the sent set to the
   *  previous edit): the base is the set that was sent — fixed from the send (or the first paint of a session / a switched-to set) until
   *  the next send, whatever the edits between; the line reads `vs sent`. */
  /** QA 0c6e126 (qaY: `vs sent · D5 88→84%` against an 88 no forecast ever showed — the sent set measured again under a changed cage):
   *  the sent set's shares as the camp painted them (`D5` → 71, `bank` → 0), per term; a term reads `from→to` only from a share painted
   *  here, else its signed move. Cleared with the base. */
  // (qaZ: `D5 46→20%`, `42→18%`, `51→20%` with nothing sent — the base read again on each pass: only the share last painted counts)
  private baseShown = new Map<string, number>();
  private noteBaseShown(f: Forecast): void {
    this.baseMoved = false;   // QA 524827b: the sent set painted under the lineage now — the paired move reads against it again
    const add = (k: string, x: number | undefined): void => { if (typeof x === "number") this.baseShown.set(k, Math.round(x * 100)); };
    for (const d of f.depths) add(`D${d.depth}`, d.reach);
    if (f.ends) { add("bank", f.ends.bank); add("death", f.ends.death); add("stall", f.ends.stall); add("return", f.ends.return); }
  }
  /** QA 0c6e126 (qaY: the invisibility potion R1 drinks, bought — `D5 71→65`, `D6 25→21` on the bars with no word; dropping the
   *  leash put them back): a lineage change's move on the same rules and the same seeds — the forecast after it less the one painted
   *  before, per depth, with the bar's ± (a move inside it reads `≈ ±N`: the sims, not the purchase). Shown under the shaft
   *  (`buy · D5 ≈ ±9`) until the rules or the lineage change again. */
  lmove: { label: string; rules: string; depths: { depth: number; delta: number; pm?: number }[] } | null = null;
  /** Cut 28 §2: the state's part of the forecast's move since the last send (`party −2 jackals · death +24`) — the sent set's refined
   *  shares now less the ones painted before it ran (ui/attrib.ts); its own line under the shaft until the next send. */
  smove: StateMove | null = null;
  /** Cut 28 §2 (core): the camp's move against the set sent, attributed to the state's changes and the rows' (`forecastMove(sent)`, asked
   *  after each refined paint on a background lane); its state parts are the shaft's state lines. Null until it lands; cleared by a send. */
  fmove: ForecastMove | null = null;
  private moveOff = false;
  /** The core attributes the move (`forecastMove`): the client's own read (`smove`) is only the fallback. */
  get moveByCore(): boolean { return !!this.engine?.forecastMove && !this.moveOff; }
  /** Asked once after a send, on the camp's first refined paint (the state changed by the run; purchases after it are `lmove`'s). */
  private moveDue = false;
  private moveTimer = 0;
  /** Asked once the camp is quiet (MOVE_QUIET_MS after a refined paint with no edit): its 2–6 panels never queue beside an edit's refine. */
  private askMove(): void {
    clearTimeout(this.moveTimer);
    if (!this.moveDue) return;
    const seq = this.editSeq;
    this.moveTimer = window.setTimeout(() => { if (seq === this.editSeq) this.askMoveNow(); }, MOVE_QUIET_MS);
  }
  private askMoveNow(): void {
    const e = this.engine; if (!this.moveDue || this.view.kind !== "camp" || this.moveOff || !e.forecastMove || this.overBudget || !this.lineage) return;
    this.moveDue = false;
    const sent = this.vsBase ? cloneSet(this.vsBase) : cloneSet(this.rules), lin = this.lineage;
    void e.forecastMove(sent).then((m) => {
      // (its state parts are the sent set's on the lineage now — an edit meanwhile leaves them standing; the rows' part is `vs sent`'s)
      // the lineage replaced meanwhile (a refresh, a purchase): asked again
      if (lin !== this.lineage) { this.moveDue = true; this.askMove(); return; }
      this.fmove = m;
      for (const fn of this.vsListeners) { try { fn(); } catch (err) { console.warn("vs listener", err); } }
    }).catch((err) => { this.moveOff = true; console.warn("forecastMove unavailable", err); });
  }
  private snap: StateSnap | null = null;
  /** The refined shares of `rules` under the lineage now: after a run (`snap.ran`) the move from the snap is the state's; then they are the snap. */
  private noteState(rules: string, shares: Record<string, number>, pm: Record<string, number>, sims: number): void {
    const lin = linSum(this.lineage), s = this.snap;
    if (s?.ran && s.rules === rules) {
      const label = stateLabel(s.lin, lin), terms = stateTerms(s, shares, pm);
      this.smove = terms.length ? { label: label || /* copy:callout */ "since run", rules, terms } : null;
    }
    this.snap = { seed: this.lineage.seed, rules, lin, shares, pm, sims, ran: false };
    writeSnap(this.snap);
  }
  /** A send of the rules now (a watched run, or an absence): the snap of these rules is what the state's move is read against. */
  private markRan(): void {
    this.smove = null; this.fmove = null; this.moveDue = true;
    if (this.snap && this.snap.rules === rulesKey(this.rules)) { this.snap.ran = true; writeSnap(this.snap); }
  }
  private lmPending: { label: string; before: Forecast | null; rules: string } | null = null;
  private noteLineageMove(f: Forecast): void {
    const p = this.lmPending; if (!p) return;
    const rules = JSON.stringify(this.rules.rows);
    if (rules !== p.rules || !p.before) { this.lmPending = null; return; }
    // the same pass on both sides (the first pass after the change waits for its refine when the one before was refined)
    if ((p.before.sims ?? 0) !== (f.sims ?? 0)) { if ((f.sims ?? 0) > (p.before.sims ?? 0)) this.lmPending = null; return; }
    const was = new Map(p.before.depths.map((d) => [d.depth, d.reach]));
    this.lmove = { label: p.label, rules, depths: f.depths.filter((d) => was.has(d.depth)).map((d) => ({ depth: d.depth, delta: d.reach - was.get(d.depth)!, pm: d.pm })) };
    this.lmPending = null;
  }
  /** Was the sent set's share `pct` for `key` (`D5`, `bank`, `death`, `stall`) painted in this camp? */
  baseWasShown(key: string, pct: number): boolean { return this.baseShown.get(key) === pct; }
  resetVs(): void { this.lmove = null; this.lmPending = null; this.baseShown.clear(); this.baseMoved = false; if (this.lastForecast && this.fcRules && sameSet(this.fcRules, this.rules)) this.noteBaseShown(this.lastForecast); this.vsBase = cloneSet(this.rules); this.vsBaseShadow = [...this.shadowedBy()]; this.fcRules = cloneSet(this.rules); this.fcShadow = [...this.vsBaseShadow]; this.fcFresh = true; this.setVs(null); }
  /** QA 778fa1b (qaV: `D10 ≈ · bank ≈` held 16 s after an edit that took bank 0 → 86 %): an edit's move is being measured — the rules
   *  differ from the base and no move for them has landed yet (the line reads `vs sent …`, never a stale `≈`). */
  // QA 524827b (qaAA: `vs sent…` held > 25 s after a cage change): a refined move that landed and still pairs other sims than the bars
  // painted will not be replaced — the line drops (no `…` forever)
  vsPending(): boolean {
    const v = this.vs, f = this.lastForecast;
    if (v && f && v.refined && v.sims && f.sims && v.sims !== f.sims && f.refined) return false;
    return !this.vsShown() && !this.vsOff && !!this.engine.forecastVs && !!this.vsBase && !this.overBudget && !sameSet(this.vsBase, this.rules);
  }
  /** QA 912e135 (qaW: `D6 61%` read `▲32`, then `▲38` with nothing edited — the refined bars under the first pass's move): the move shown
   *  is the one paired with the forecast painted (the same sims: `base + delta` is the bar's own share); until the refine's move lands
   *  the line reads `vs sent …` and the marks wait. */
  vsShown(): ForecastVs | null {
    const v = this.vs, f = this.lastForecast;
    if (!v || (f && v.sims && f.sims && v.sims !== f.sims)) return null;
    return this.baseMoved ? this.rebased(v) : v;
  }
  /** QA 524827b (qaAA: after a leash bought D6 went 11 → 17 % and read `▼27 · vs sent −27`, one edit earlier 11 % read `▼15` — the
   *  sent set measured again under the new kit, 44 %, a number no bar showed): once the lineage moved (a purchase, a drop, the cage, the
   *  kit…) since the sent set was painted, a term whose painted share differs from the paired base is read against the share painted —
   *  `▼`/`▲` is the shown number now less the sent set's shown one (`D6 26→17%`), with the bar's own ± (two panels, not paired). A term
   *  whose base still reads as painted keeps its paired move. Set by `mutate` / `dropSupply`; cleared when the sent set is painted again. */
  private baseMoved = false;
  private rebased(v: ForecastVs): ForecastVs {
    const n = v.sims ?? this.lastForecast?.sims ?? 0;
    const hw = (p: number): number => n > 0 ? 1.96 * Math.sqrt(Math.max(0, p * (1 - p)) / n) : 0;
    const re = <T extends VsMove>(m: T, key: string): T => {
      const shown = this.baseShown.get(key);
      if (shown === undefined || typeof m.base !== "number" || Math.round(m.base * 100) === shown) return m;
      const now = m.base + m.delta, was = shown / 100;
      return { ...m, base: was, delta: now - was, pm: hw(now) };
    };
    const end = (m: number | VsMove | undefined, key: string): number | VsMove | undefined => typeof m === "object" && m ? re(m, key) : m;
    return { ...v, depths: v.depths.map((d) => { const r = re(d, `D${d.depth}`); return r === d ? d : { ...r, abs_pm: r.pm }; }),
      bank: end(v.bank, "bank"), death: end(v.death, "death"), return: end(v.return, "return"), stall: v.stall ? re(v.stall, "stall") : v.stall };
  }
  /** QA 92eb880: the active set's shadowed rows (`Forecast.shadowed_by`, per row the earlier row that takes all its moments) as of the
   *  last forecast painted for the rules now; `null` until one lands after an edit (then the lineage's own read, for the set it holds). */
  private shadow: (number | null)[] | null = null;
  private shadowEdited = false;
  shadowedBy(): (number | null)[] { return this.shadow ?? (this.shadowEdited ? [] : this.lineage?.shadowed_by ?? []); }
  /** Cut 23 §3: each row's why-not (`Lineage.row_why`, per row of the engine's active set as last fetched), matched to the editing copy
   *  by the row's shape — a row moved keeps its line, a row edited (or new) has none (the core starts it over). */
  rowWhy(): (RowWhy | null)[] {
    const L = this.lineage, why = L?.row_why; if (!why?.length) return this.rules.rows.map(() => null);
    const was = L.sets?.[L.active_set ?? 0]?.rows ?? []; const used = new Set<number>();
    return this.rules.rows.map((r, i) => {
      const k = rowKey(r);
      const j = was[i] && !used.has(i) && rowKey(was[i]) === k ? i : was.findIndex((w, x) => !used.has(x) && rowKey(w) === k);
      if (j < 0) return null; used.add(j); return why[j] ?? null;
    });
  }
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
  // QA 1a2a4a9 (O: "`yours: 0 of 2 rows` — the applied patch doesn't count as mine"): a patch the player applied is his choice too
  playerRows(): number { return this.rules.rows.filter((r) => !isCardRow(r) && ["player", "patch"].includes(r.origin ?? "player")).length; }
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
        this.watchMode = blob.watch === "fast" || blob.watch === "one" ? blob.watch : "fights";
        this.savedOrigins = blob.origins ?? null;
        elapsed = Math.max(0, (Date.now() - blob.last_seen) / 1000);
        loaded = true;
      } catch (e) { console.warn("save rejected, new lineage", e); }
    }
    if (!loaded) await this.fresh(dev?.seed);
    this.adoptSets();
    this.snap = readSnap(this.lineage.seed);
    if (dev?.rules) {
      try {
        const set = await this.engine.importRules(dev.rules);
        this.sets[this.active] = { rows: set.rows.map(cloneRow), ...routeOf(set) };
        await this.engine.setRules(this.rules);
      } catch (e) { console.warn("dev rules rejected", e); }
    }
    await this.engine.loadout(this.loadout);
    this.vocab = await this.engine.vocabulary();
    document.addEventListener("visibilitychange", () => { if (document.hidden) { this.flushSync(); this.hiddenAt = Date.now(); } else if (this.hiddenAt) void this.backFromHidden(); });
    window.addEventListener("pagehide", () => this.flushSync());
    setInterval(() => { if (!document.hidden) void this.flush(); }, 30_000);
    if (dev?.absent) { elapsed = dev.absent; loaded = true; }
    // the camp (last state) shows underneath, inert, while the batch runs (no forecast queued ahead of it)
    if (loaded && elapsed >= OFFLINE_MIN_S) await this.absence(elapsed);
    else this.go({ kind: dev?.autosend ? "watch" : "camp" });
    await this.flush();
    this.booted = true;
    // RUNS_UI: the open app's clock starts now (the absence above covered the time before it)
    this.runnerAt = Date.now();
    setInterval(() => { void this.runTick(); }, 1000);
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
    if (!Number.isSafeInteger(elapsedS) || elapsedS < 0) throw new RangeError("invalid offline elapsed seconds");
    this.offlineRunning = true;
    this.offlineIncomplete = true;
    this.root.inert = true;
    const b = showBusy(/* copy:label */ "offline");
    let merged: ReturnReport | null = null;
    let quick = true, sliced = true, callsCompleted = 0;
    let worst: { id: number; depth: number } | null = null;
    let graves = this.lineage.graveyard.length;
    try {
      let slice = OFFLINE_SLICE_S;
      for (let left = elapsedS, first = true; first || left > 0; left -= slice) {
        first = false;
        let r: ReturnReport;
        if (quick) {
          // (round 3: the slices before the last skip the stall verdict — the merged report is the last slice's —
          // on an engine that has `runOfflineSlice`; `runOfflineQuick` else)
          const last = left <= slice;
          try {
            const n = Math.min(left, slice);
            r = sliced && this.engine.runOfflineSlice
              ? await this.engine.runOfflineSlice(n, last).catch((e) => { if (callsCompleted || !missingOfflineMethod(e, "runOfflineSlice")) throw e; sliced = false; return this.engine.runOfflineQuick(n); })
              : await this.engine.runOfflineQuick(n);
          }
          catch (e) {
            if (callsCompleted || !missingOfflineMethod(e, "runOfflineQuick")) throw e;
            // stale wasm build without runOfflineQuick: the full call, larger slices (each pays the verdict)
            console.warn("runOfflineQuick unavailable, using runOffline", e); quick = false;
            slice = Math.min(OFFLINE_SLICE_MAX_S, Math.max(OFFLINE_SLICE_S, Math.ceil(elapsedS / OFFLINE_SLICES)));
            r = await this.engine.runOffline(Math.min(left, slice));
          }
        } else r = await this.engine.runOffline(Math.min(left, slice));
        callsCompleted++;
        // Current cores settle one report at the real boundary. Older cores
        // still return additive per-slice reports and retain their merge path.
        if (r.slice_pending) continue;
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
    if (!merged) throw new Error("offline report unfinished");
    this.offlineIncomplete = false;
    this.runsSeen += merged.runs;
    return merged;
  }
  totalRuns(): number { return this.runsSeen; }

  /** After the ending, the core owns the next variant and its carry/reset rules.
   *  Refusal preserves the current town. */
  async beginDescent(tier: number): Promise<boolean> {
    if (this.ascensionBusy || !this.engine.beginDescent) return false;
    this.ascensionBusy = true;
    try {
      try { this.lineage = await this.engine.beginDescent(tier); }
      catch (e) { console.warn("descent refused", e); return false; }
      this.loadout = [...(this.lineage.selected_loadout ?? [])];
      this.adoptSets();
      try { this.vocab = await this.engine.vocabulary(); }
      catch (e) { console.warn("descent metadata unavailable", e); }
      try { await this.flush(); } catch (e) { console.warn("descent save unavailable", e); }
      this.go({ kind: "camp" });
      return true;
    } finally { this.ascensionBusy = false; }
  }
  async ascend(variant: string): Promise<boolean> {
    if (this.ascensionBusy) return false;
    this.ascensionBusy = true;
    try {
      try { this.lineage = await this.engine.ascend(variant); }
      catch (e) { console.warn("ascension refused", e); return false; }
      this.loadout = []; this.runsSeen = 0;
      this.adoptSets();
      try {
        await this.engine.loadout([]);
        this.vocab = await this.engine.vocabulary();
      } catch (e) { console.warn("ascension metadata unavailable", e); }
      // The core already ascended: persist and show its new town even if the
      // follow-up metadata read fails. Never retry that irreversible step.
      await this.flush();
      this.go({ kind: "camp" });
      return true;
    } finally { this.ascensionBusy = false; }
  }
  /** After the ending: a fresh lineage that keeps the player's three rule sets (facts, classes, meta reset —
   *  the core has no carry-over method). */
  async again(): Promise<void> {
    const sets = this.sets.map(cloneSet); const active = this.active;
    this.lineage = await this.engine.newLineage(randomSeed());
    this.offlineIncomplete = false;
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
    this.offlineIncomplete = false;
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
  /** Cut 30 §2: on a lineage on packages the core keeps the pen's rows above every package and recompiles (`packages::absorb`) — the
   *  editing copy takes the core's order once the set is in, so the rows, their indices and the forecast's shadow marks are the core's.
   *  Nothing when the order already matches (the common edit) or another edit came since. */
  private adoptCompiled(seq: number): void {
    if (!this.lineage?.packages || this.lineage.packages.literal) return;
    void this.engine.lineage().then((L) => {
      if (seq !== this.rulesSeq) return;
      const key = (rows: Row[]): string => JSON.stringify(rows.map((r) => [r.conds, r.verb, r.origin ?? ""]));
      const core = L.sets?.[L.active_set ?? this.active]?.rows ?? [];
      const same = key(core.map((r) => ({ ...r, origin: r.origin ?? "player" }))) === key(this.rules.rows.map((r) => ({ ...r, origin: r.origin ?? "player" })));
      this.lineage = L;
      if (same) return;
      this.adoptSets(); this.emitChange();
      for (const fn of this.rulesListeners) fn();
    }).catch(() => undefined);
  }
  /** Cut 29 §2: the camp showed the newly opened systems; the next send tells the core (`seenSystems`) — never mid-edit. */
  seenPending = false;

  // --- rules ---
  /** Cut 4 §1: more rows than the vocabulary allows. A patch never evicts a row; the editor shows `5/4` and `send`
   *  and the forecast wait until the player removes one. The engine keeps its last valid set meanwhile.
   *  Cut 12 §1: own rows against `max_rows`; card rows sit outside the cap. */
  get overBudget(): boolean { return this.ownRows() > this.vocab.max_rows; }
  rulesChanged(): void {
    debugNote("rules", "edited");
    this.rowFires = null; this.rowFiresOf = undefined;   // Cut 14 §4: the counts were the set that ran
    // Cut 22 §3: the edit's base is the set the last painted forecast measured; the shown move clears until this edit's lands
    if (!this.vsBase && this.fcFresh && this.fcRules) { this.vsBase = this.fcRules; this.vsBaseShadow = this.fcShadow; if (this.lastForecast) this.noteBaseShown(this.lastForecast); }
    this.fcFresh = false; this.editSeq++; this.setVs(null);
    this.persist();
    clearTimeout(this.fcTimer); this.refineSeq++; this.forecastRefining = false;
    this.shadow = null; this.shadowEdited = true;   // QA 92eb880: the marks wait for the forecast of the rules now
    for (const fn of this.rulesListeners) fn();
    if (this.overBudget) return;
    const seq = ++this.rulesSeq;
    void this.engine.setRules(this.rules).then(() => { if (seq === this.rulesSeq) { this.shelfCheck(); this.adoptCompiled(seq); } }).catch((e) => console.warn("rules rejected", e));
    this.fcTimer = window.setTimeout(() => void this.emitForecast(), FC_DEBOUNCE_MS);
  }
  /** Cut 24 §4: the state a forecast measures — the edit and the lineage it was asked on. */
  private fcKey(): string { this.lineageIds.has(this.lineage) || this.lineageIds.set(this.lineage, ++this.lineageN); return `${this.editSeq}:${this.lineageIds.get(this.lineage)}`; }
  private lineageIds = new WeakMap<object, number>(); private lineageN = 0;
  private refinedKey = ""; private refinedN = 0;   // the requested state whose detailed result painted, and its generation
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
    if (this.lineage.town?.home === false) return;
    if (!this.fcListeners.size) return;
    if (this.refinedKey === this.fcKey() && this.lastForecast?.refined) return;
    // Cut 25 §4: with lanes that can take it (`parallelForecast`) an edit's forecast starts at once — a stale one in flight is left to finish
    // unpainted (it no longer holds the fresh one behind it: 1.5 s after a burst of edits, where the bar is 1.2)
    const par = !!this.engine.parallelForecast;
    if (this.offlineRunning || (this.fcInFlight && !par)) { this.fcDirty = true; return; }
    const ask = ++this.fcAsk;
    this.fcInFlight = true; this.fcDirty = false; this.fcRunning++;
    try {
      const asked = cloneSet(this.rules), key = this.fcKey(), refinedBefore = this.refinedN;
      const estimate = this.engine.forecastEstimate ?? this.engine.forecast;
      const f = await this.busy(/* copy:label */ "forecast", () => estimate.call(this.engine));
      // QA 23ed91f (L: switching to an empty set, the shaft kept the old set's `return 94%` for ~5 s, then flipped): a forecast whose
      // rules changed while it ran is not painted (the shaft stays dimmed `stale`); the next one, for the rules now, is
      if (!this.fcDirty && ask === this.fcAsk) {
        this.fcRules = asked; this.fcShadow = f.shadowed_by ?? []; this.fcFresh = true;
        // Cut 24 §4: the refine of this very state may have painted while this pass ran (it runs beside it) — the first pass is then
        // older news; an explicitly refined answer keeps its requested quality
        if (!(this.refinedKey === key && this.refinedN !== refinedBefore)) { this.publishForecast(f); this.measureVs(f, asked); }
      }
    } catch (e) { console.warn("forecast failed", e); }
    finally { if (--this.fcRunning <= 0) { this.fcRunning = 0; this.fcInFlight = false; } }
    if (this.fcDirty && !this.fcInFlight) await this.emitForecast();
  }
  /** QA a946e04 (S: the shaft and the panel showed two passes at once): one forecast to every listener — each in its own try, so a
   *  listener that throws never leaves the ones after it on the previous pass. */
  private publishForecast(f: Forecast): void {
    this.lastForecast = f; this.shadow = f.shadowed_by ?? []; this.forecastSeq++;
    if (this.vsBase && sameSet(this.vsBase, this.rules)) this.noteBaseShown(f);
    this.noteLineageMove(f);
    if (f.refined && !this.overBudget && this.lineage && this.view.kind === "camp") { const sh = sharesOf(f); this.noteState(rulesKey(this.rules), sh.shares, sh.pm, f.sims ?? 0); this.askMove(); }
    for (const fn of this.fcListeners) { try { fn(f); } catch (e) { console.warn("forecast listener", e); } }
  }
  /** Larger forecasts run only when requested, and can only paint the state asked. */
  refineForecast(): Promise<void> {
    if (!this.engine.forecastRefine || this.offlineRunning || this.overBudget) return Promise.resolve();
    const key = this.fcKey();
    if (this.refinedKey === key && this.lastForecast?.refined) return Promise.resolve();
    if (this.refineRequest && this.refineRequestKey === key) return this.refineRequest;
    const seq = ++this.refineSeq, asked = cloneSet(this.rules);
    this.refineRequestKey = key; this.forecastRefining = true;
    const job = Promise.resolve().then(async () => {
      try {
        if (seq !== this.refineSeq || key !== this.fcKey()) return;
        const f = await this.engine.forecastRefine!();
        if (seq !== this.refineSeq || key !== this.fcKey()) return;
        this.fcRules = asked; this.fcShadow = f.shadowed_by ?? []; this.fcFresh = true;
        this.refinedKey = key; this.refinedN++;
        this.publishForecast(f);
        if (this.vs && !this.vs.refined) for (const fn of this.vsListeners) { try { fn(); } catch (e) { console.warn("vs listener", e); } }
        if (this.vsBase && !sameSet(this.vsBase, this.rules)) { if (f.vs) this.setVs(f.vs); else this.measureVs(f, cloneSet(this.rules), true); }
      } finally {
        if (seq === this.refineSeq) { this.forecastRefining = false; this.refineRequest = null; }
      }
    });
    this.refineRequest = job;
    return job;
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
    this.vsBase = null; this.baseShown.clear(); this.fcFresh = false;   // Cut 22 §3: a switch is not an edit — no move against the other set (its first paint is the base)
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
    // Cut 25 §2: a move — the set's own row at `moves_from` goes above the row at `insert_at` (as `offline::apply_patch`)
    if (p.moves_from !== undefined && p.moves_from >= 0) {
      const from = p.moves_from, at = p.insert_at;
      if (from < rows.length && at >= 0 && at < from) { const [r] = rows.splice(from, 1); rows.splice(at, 0, r); this.rulesChanged(); }
      return at;
    }
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
  /** Cut 29 §1 (E1): a whole set measured by the core (the wall's edit) replaces the active one's rows; a row the set already held keeps
   *  its origin, a new one is the patch's. */
  applyRules(set: RuleSet): void {
    const was = this.rules.rows;
    this.sets[this.active] = { ...this.sets[this.active], rows: set.rows.map((r) => { const k = rowKey(r); const old = was.find((x) => rowKey(x) === k); return { ...cloneRow(r), origin: old?.origin ?? r.origin ?? "patch" }; }), ...routeOf(set) };
    this.rulesChanged();
    this.emitChange();
  }
  async setRulesText(text: string): Promise<void> {
    const set = await this.engine.importRules(text);
    this.sets[this.active] = { rows: set.rows.map((r) => ({ ...cloneRow(r), origin: r.origin ?? "player" })), name: this.sets[this.active]?.name, ...routeOf(set) };   // Cut 26 §2: an export carries its route
    this.rulesChanged();
    this.emitChange();
  }

  // --- lineage ---
  async selectBloodline(id:number):Promise<boolean> {
    if((this.lineage.selected_bloodline??1)===id)return true;
    if(!this.engine.selectBloodline)return false;
    if(this.view.kind==="watch")this.go({kind:"camp"});
    this.lastForecast=null;this.loadout=[];
    const ok=await this.mutate(()=>this.engine.selectBloodline!(id),undefined,true);
    if(ok){this.loadout=[...(this.lineage.selected_loadout??[])];this.persist();this.runnerAt=Date.now();this.go({kind:"camp"});}
    return ok;
  }
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
  /** `adopt` (Cut 30 §2): the call recompiled the set (a package equipped, levelled, a drill revoked) — the editing copy is the core's again. */
  async mutate(fn: () => Promise<Lineage>, move?: string, adopt = false, options?: { saveBeforePaint?: boolean }): Promise<boolean> {
    // QA 0c6e126 (qaY): a purchase, a drop, a cage or a kit step (`move`, its word) — the camp's next forecast for these rules is read
    // against the one painted before it (`lineageMove`)
    // QA 524827b (qaAA: after `+1 row` and `verb: throw` bought, the line under the forecast still read `drop · D6 ≈ ±6` from the leash
    // drop): an unnamed change clears the last move's line — the line names the last move or none
    if (move) { this.lmPending = { label: move, before: this.lastForecast, rules: JSON.stringify(this.rules.rows) }; this.lmove = null; }
    else { this.lmPending = null; this.lmove = null; }
    try { this.lineage = await fn(); } catch (e) { this.lmPending = null; console.warn("engine refused", e); return false; }
    if (adopt) this.adoptSets();
    this.baseMoved = true;
    // A finished manual building must survive navigation from its first paint.
    // Fetch the worker's snapshot before listeners can expose completion; normal
    // edits retain their fast paint/debounced save, without an extra save here.
    const saveBeforePaint = options?.saveBeforePaint === true;
    if (saveBeforePaint) await this.flush();
    await this.afterLineage();
    return true;
  }
  /** Cut 4 §9: a bought tactic card becomes a row `[card] <name>` in the active set (the card only acts as a row:
   *  `{v:"tactic", a:<id>}`), so the player sees where it sits. Cut 12 §1: it sits where it acts — at the catalogue's
   *  `insert_at` when the engine sends one, else before the set's engagement row (the first `attack` / `shoot`), else the
   *  end; card rows sit outside `max_rows`, so a card never overflows the set. */
  async buy(id: string, gold = false, card?: { join: boolean; at?: number }): Promise<boolean> {   // card: what the sheet said the buy does (`joins at R2` / `owned · add separately`), else the catalogue's flag and place
    const u = this.unlockCat.find((x) => x.id === id);
    const at = card ? card.at : u?.insert_at;
    const join = card?.join;
    // Cut 15 §2: `gold` pays the catalogue's gold price instead of marks (the returned lineage repaints the header's $ and ◆)
    const ok = await this.mutate(() => (gold ? this.engine.buyUnlockGold!(id) : this.engine.buy(id)));
    if (ok) audio.cue("unlock");   // Cut 10 §4
    // QA e75ec29 (R: eight cards bought → eight rows inserted, `D7 76%` → `44%`, `stall 40%`): a card joins the set only when the core
    // measured it helps there (`UnlockInfo.auto_insert`: reach not down at its place, stall share not up, < 3 cards in the set); else
    // it is owned, off the set — its chip offers `add` (an older wire without the flag inserts as before)
    const isTactic = this.vocab.verbs.some((v) => v.v === "tactic" && v.a === id);
    // QA a946e04 (T: three cards bought, all three inserted — the catalogue's merge had dropped the flag, and absent read as yes): the
    // flag is honoured strictly — a card joins only on `auto_insert: true`; false or absent, it is owned and its chip offers `add`
    if (ok && isTactic && (join ?? u?.auto_insert === true) && !this.holdsCard(id)) { this.insertCard(id, at); this.emitChange(); }
    else if (ok && isTactic) { /* owned, not in the set */ }
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
    // QA 308f045 (qaAD: `kite archers` added at R1, `gas step` at R2 over `hp < 30% → drink heal`; both absence deaths read `R4 under R2`):
    // a card never goes above the set's safety rows (the core's `meta::safety_end`)
    const rows = this.rules.rows;
    return this.insertRow({ conds: [], verb: { v: "tactic", a: id } }, Math.min(rows.length, Math.max(at ?? engagementRow(rows), safetyEnd(rows))), "card");
  }
  /** Cut 12 §6: one supply off the shelf. An engine without `dropSupply` clears the shelf and rebuys the other bought
   *  lines in order (a free line — the kennel's leash — comes back at the next exit). */
  async dropSupply(id: number): Promise<boolean> {
    const picks = this.lineage.supplies ?? [];
    const it = picks.find((p) => p.id === id); if (!it) return false;
    this.lmPending = { label: /* copy:callout */ "drop", before: this.lastForecast, rules: JSON.stringify(this.rules.rows) }; this.lmove = null;
    try { this.lineage = await this.engine.dropSupply!(id); this.baseMoved = true; await this.afterLineage(); return true; }   // the proxy always has it; an engine without it rejects
    catch (e) { console.warn("dropSupply unavailable, clear + rebuy", e); }
    const rest = picks.filter((p) => p.id !== id && !isFreeSupply(this.lineage, p)).map((p) => p.kind);
    return this.mutate(async () => { let L = await this.engine.clearSupplies(); for (const k of rest) L = await this.engine.buySupply(k); return L; });
  }
  setClass(cls: string): Promise<boolean> { return this.mutate(() => this.engine.setClass(cls)); }
  /** Hero looks: the heirs' cosmetic look; false on an engine without it. */
  setLook(look: string): Promise<boolean> { const e = this.engine; return e.setLook ? this.mutate(() => e.setLook!(look)) : Promise.resolve(false); }
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
    // A transport checkpoint has no completed wall-clock timestamp. Keep the
    // previous durable save so a reload can replay the whole remaining absence.
    if (this.offlineRunning || this.offlineIncomplete) return;
    let saved: string;
    try { saved = await this.engine.save(); } catch (e) { console.warn("save failed", e); return; }
    if (this.offlineRunning || this.offlineIncomplete) return;
    this.lastSave = saved;
    writeBlob(this.blob());
  }
  /** pagehide/visibilitychange cannot await the worker: write the last save string fetched. */
  private flushSync(): void { if (!this.offlineRunning && !this.offlineIncomplete && this.lastSave) writeBlob(this.blob()); }
  private blob(): SaveBlob { return { v: 2, engine: this.lastSave, loadout: this.loadout, last_seen: Date.now(), runs: this.runsSeen, origins: this.sets.map((s) => s.rows.map((r) => r.origin ?? "player")), watch: this.watchMode }; }
  exportSave(): string { return JSON.stringify(this.blob()); }
  async importSave(text: string): Promise<boolean> {
    try {
      const b = JSON.parse(text) as SaveBlob;
      if (!b || typeof b.engine !== "string") return false;
      this.lineage = await this.engine.load(b.engine);
      this.offlineIncomplete = false;
      this.loadout = b.loadout ?? []; this.runsSeen = b.runs ?? 0; this.watchMode = b.watch === "fast" || b.watch === "one" ? b.watch : "fights";
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
    debugNote("screen", screen.kind);
    closeAllSheets();
    this.mounted?.dispose?.();
    if (screen.kind === "camp" && this.lineage.ended) screen = { kind: "ending" };
    this.view = screen;
    if (screen.kind === "report" && screen.absence && this.lastAbsence?.report !== screen.report) this.lastAbsence = { report: screen.report, played: false };
    if (screen.kind === "watch") {
      this.resetVs(); this.markRan();
      // Dev playback parameters set the mode before mounting; controls now live in Speed.
      const sp = this.dev?.speed;
      if (sp) this.watchMode = typeof sp === "number" ? (sp === 1 || sp === 4 || sp === 8 ? "fast" : "fights") : sp === "fast" ? "fast" : "fights";
    }   // Cut 22 §3: the set that runs is the next edit's base
    let m: Mounted;
    switch (screen.kind) {
      case "camp": m = renderCamp(this, screen.highlight); break;
      case "ending": m = renderEnding(this); break;
      case "watch": m = renderWatch(this); break;
      case "death": m = renderDeath(this, screen.death, screen.lost ?? [], !!screen.kept, screen.from); break;
      case "report": m = renderReport(this, screen.report, screen.absence); break;
    }
    this.mounted = m;
    this.root.replaceChildren(m.el);
    // Cut 28 §2: the first camp after a send asks the move's attribution (behind the camp's own first forecast)
    if (screen.kind === "camp" && this.moveDue) this.askMove();
    this.root.dataset.screen = screen.kind;
    document.body.classList.toggle("framed", screen.kind !== "ending");   // Cut 17: sheets unfold above the console
    window.scrollTo(0, 0);
    this.persist();
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

/** Two per-label counts summed (most first); undefined when both lack the field. */
function mergeCounts(x?: { label: string; n: number }[], y?: { label: string; n: number }[]): { label: string; n: number }[] | undefined {
  if (x === undefined && y === undefined) return undefined;
  const m = new Map<string, number>(); for (const r of [...(x ?? []), ...(y ?? [])]) m.set(r.label, (m.get(r.label) ?? 0) + r.n);
  return [...m].map(([label, n]) => ({ label, n })).sort((p, q) => q.n - p.n);
}
/** QA a946e04: stolen rows summed per label, their worth (`gold`) with them. */
function mergeStolen(x?: { label: string; n: number; gold?: number }[], y?: { label: string; n: number; gold?: number }[]): { label: string; n: number; gold?: number }[] | undefined {
  if (x === undefined && y === undefined) return undefined;
  const m = new Map<string, { label: string; n: number; gold?: number }>();
  for (const r of [...(x ?? []), ...(y ?? [])]) { const c = m.get(r.label) ?? { label: r.label, n: 0 }; c.n += r.n; if (r.gold !== undefined) c.gold = (c.gold ?? 0) + r.gold; m.set(r.label, c); }
  return [...m.values()].sort((p, q) => q.n - p.n);
}
/** Cut 25 §5 (AM: five copies of `Lock bloats took him to N HP; R1 drank` in one night's reel): a reel line's shape — its
 *  text with the numbers out (`… to 7 HP …` and `… to 9 HP …` are one shape). */
export function reelShape(text: string): string { return text.replace(/\d+/g, "N"); }
/** Cut 25 §5: two slices' reels merged — one line per shape (the best-scored of it), `n` the times the shape came up across the
 *  absence (`×5` on the report), the top five shapes by score: a shape never repeats. */
export function mergeReel(a: Highlight[], b: Highlight[]): Highlight[] {
  const byShape = new Map<string, Highlight>();
  for (const x of [...a, ...b]) {
    const k = reelShape(x.text), cur = byShape.get(k), n = (cur?.n ?? (cur ? 1 : 0)) + (x.n ?? 1);
    byShape.set(k, !cur || x.score > cur.score ? { ...x, n } : { ...cur, n });
  }
  return [...byShape.values()].sort((x, y) => y.score - x.score).slice(0, 5);
}
function missingOfflineMethod(error: unknown, method: string): boolean {
  const message = error instanceof Error ? error.message : String(error);
  return [`wasm: ${method}`, `no engine method ${method}`, `unknown engine method ${method}`].includes(message);
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
    bloodlines: [...new Set([...(a.bloodlines??[]),...(b.bloodlines??[])].map(s=>s.id))].map(id=>{
      const rows=[...(a.bloodlines??[]),...(b.bloodlines??[])].filter(s=>s.id===id),last=rows[rows.length-1]!;
      const progression = rows.reduce<Pick<ReturnReport, "packages" | "grew">>((merged,s)=>mergeGrew(merged,s),{});
      const xp = rows.reduce<ReturnReport["xp"][] | undefined>((merged,s)=>mergeClassXp(merged,s.xp),undefined);
      const bests = rows.some(s=>s.bests!==undefined) ? [...new Set(rows.flatMap(s=>s.bests??[]))] : undefined;
      const boss_knowledge = rows.some(s=>s.boss_knowledge!==undefined)
        ? [...new Map(rows.flatMap(s=>s.boss_knowledge??[]).map(k=>[k.boss,k])).values()] : undefined;
      return {...last,...progression,xp,bests,boss_knowledge,runs:rows.reduce((n,s)=>n+s.runs,0),gold:rows.reduce((n,s)=>n+s.gold,0),deepest:Math.max(...rows.map(s=>s.deepest))};
    }),
    rested_s: sum(a.rested_s, b.rested_s), banked: sum(a.banked, b.banked), returned: sum(a.returned, b.returned), stalled: sum(a.stalled, b.stalled), driven: sum(a.driven, b.driven),
    bones_found: cat(a.bones_found, b.bones_found),
    new_finds: a.new_finds || b.new_finds ? [...new Set([...(a.new_finds ?? []), ...(b.new_finds ?? [])])] : undefined,   // QA 0c6e126 (qaY): every kind first found this absence
    picked: b.picked ?? a.picked,                         // Cut 16 §1: a state — the last slice knows
    restock_capped: a.restock_capped || b.restock_capped || undefined,   // Cut 19 §3: any slice's repeat stopped at the night's income
    repeat_short: a.repeat_short || b.repeat_short || undefined,         // QA 1a2a4a9: any slice's re-pack ran short
    // Cut 20 §5: the night's bounty — slices of one night add up (taken by any, the coins summed); a later night's floor replaces it
    // QA e75ec29: the floor the player saw before leaving is the first slice's — a later slice's (a new best moved it) never replaces it
    bounty: !a.bounty ? b.bounty : !b.bounty ? a.bounty : a.bounty.depth === b.bounty.depth ? { depth: a.bounty.depth, taken: a.bounty.taken || b.bounty.taken, gold: a.bounty.gold + b.bounty.gold } : a.bounty,
    stolen: mergeStolen(a.stolen, b.stolen),               // QA e75ec29 (R): thefts nothing got back, per label (QA a946e04: with their worth)
    stolen_gold: sum(a.stolen_gold, b.stolen_gold),        // QA a946e04: the carry the thefts took
    // QA a946e04: the waystone the night's pass went unpaid for — the runs from D1 summed across slices of one waystone
    start_short: !a.start_short ? b.start_short : !b.start_short ? a.start_short : { ...b.start_short, runs: a.start_short.runs + (a.start_short.depth === b.start_short.depth ? b.start_short.runs : 0) },
    shelved: mergeCounts(a.shelved?.map((x) => ({ label: x.kind, n: x.n })), b.shelved?.map((x) => ({ label: x.kind, n: x.n })))?.map((x) => ({ kind: x.label, n: x.n })),   // Cut 21 §2: found supplies to the shelf, per kind
    exits: cat(a.exits, b.exits),                          // Cut 6 §1: one ledger line per exit
    heirs: a.heirs?.length && b.heirs?.length ? [Math.min(a.heirs[0], b.heirs[0]), Math.max(a.heirs[1], b.heirs[1])] : b.heirs?.length ? b.heirs : a.heirs,   // QA 912e135
    elapsed_s: a.elapsed_s + b.elapsed_s, runs: a.runs + b.runs, sampled: a.sampled || b.sampled,
    deepest: a.deepest === undefined && b.deepest === undefined ? undefined : Math.max(a.deepest ?? 0, b.deepest ?? 0),
    gold: a.gold || b.gold ? { home: (a.gold?.home ?? 0) + (b.gold?.home ?? 0), salvage: (a.gold?.salvage ?? 0) + (b.gold?.salvage ?? 0), wake: (a.gold?.wake ?? 0) + (b.gold?.wake ?? 0), spent: (a.gold?.spent ?? 0) + (b.gold?.spent ?? 0),
      ...(a.gold?.wake_cap ?? b.gold?.wake_cap) !== undefined ? { wake_cap: b.gold?.wake_cap ?? a.gold?.wake_cap, wake_n: (a.gold?.wake_n ?? 0) + (b.gold?.wake_n ?? 0) } : {},
      ...(a.gold?.lost ?? b.gold?.lost) !== undefined ? { lost: (a.gold?.lost ?? 0) + (b.gold?.lost ?? 0), unkept: (a.gold?.unkept ?? 0) + (b.gold?.unkept ?? 0) } : {} } : undefined,   // QA 912e135
    learned: union(a.learned, b.learned), bests: collapseBests(union(a.bests, b.bests)),
    found: [...a.found, ...b.found],
    deaths: [...deaths].map(([cause, n]) => ({ cause, n })).sort((x, y) => y.n - x.n),
    pending: mergePending(a.pending, b.pending),          // state lines from the last slice; `R1 fired n of m runs` summed across slices
    stall: b.stall,                                       // likewise: the window is on the game, the last slice knows
    reel: mergeReel(a.reel, b.reel),                      // Cut 25 §5: one line per shape, its count in `n`
    marks_earned: a.marks_earned + b.marks_earned, worst_death: worst, live: b.live,
    tamed: [...a.tamed, ...b.tamed], hatched: [...a.hatched, ...b.hatched], lost: [...a.lost, ...b.lost],
    xp: { class: b.xp.class, gained: a.xp.gained + b.xp.gained, level_ups: a.xp.level_ups + b.xp.level_ups },
    salvaged: [...salv].map(([kind, v]) => ({ kind, ...v })),
    spent: spent ? [...spent].map(([kind, v]) => ({ kind, ...v })) : undefined,
    renown: { gained: a.renown.gained + b.renown.gained, rank: b.renown.rank, ranks_up: a.renown.ranks_up + b.renown.ranks_up },
    ...mergeLead(a, b),
    // Cut 29: the night's mark adds up; systems opened in curriculum order (a later slice's after the earlier's); the extra slots'
    // kept oaths and the fallen companions in order (the wall's edit is `engine.wallEdit()`, asked on the report); the night's meter field by field
    night_marks: sum(a.night_marks, b.night_marks),
    systems_opened: a.systems_opened || b.systems_opened ? union(a.systems_opened ?? [], b.systems_opened ?? []) : undefined,
    oaths_kept: cat(a.oaths_kept, b.oaths_kept),
    fallen: cat(a.fallen, b.fallen),
    meters: mergeMeters(a.meters, b.meters),
    ...mergeGrew(a, b),
    workers: mergeWorkers(a.workers, b.workers), chest: sum(a.chest, b.chest) || undefined,   // Cut 30.5: the workers' acts and the haul left in the chest add up
  };
}
/** Cut 30 §4: what grew over an absence adds up across its slices — per track, the gold summed (`+$2400`), a best or a level the
 *  highest (`best D14`, `L7`), the rest once each; the packages' beats in order, a package's levels collapsed to its highest (`STEADY L3`). */
export function mergeGrew(a: Pick<ReturnReport, "grew" | "packages">, b: Pick<ReturnReport, "grew" | "packages">): Pick<ReturnReport, "grew" | "packages"> {
  const out: Pick<ReturnReport, "grew" | "packages"> = {};
  if (a.grew || b.grew) {
    const lines: { track: string; what: string }[] = [];
    for (const g of [...(a.grew ?? []), ...(b.grew ?? [])]) {
      // (a level: the class's `L7`, a package's `Steady L3` — the later slice's stands)
      const gold = /^\+\$(\d+)$/.exec(g.what), num = /^(best D|(?:.+ )?L)(\d+)$/.exec(g.what);
      const at = lines.findIndex((x) => x.track === g.track && (gold ? /^\+\$\d+$/.test(x.what) : num ? x.what.startsWith(num[1]) && /^(best D|(?:.+ )?L)\d+$/.test(x.what) && x.what.slice(num[1].length).match(/^\d+$/) !== null : x.what === g.what));
      if (at < 0) { lines.push({ ...g }); continue; }
      if (gold) lines[at].what = `+$${Number(lines[at].what.slice(2)) + Number(gold[1])}`;
      else if (num && Number(num[2]) > Number(/\d+$/.exec(lines[at].what)![0])) lines[at].what = g.what;
    }
    out.grew = lines;
  }
  if (a.packages || b.packages) {
    const beats: string[] = [];
    for (const x of [...(a.packages ?? []), ...(b.packages ?? [])]) {
      const lv = /^(.+) L(\d+)$/.exec(x);
      const at = lv ? beats.findIndex((y) => y.startsWith(`${lv[1]} L`) && /^.+ L\d+$/.test(y)) : beats.indexOf(x);
      if (at < 0) beats.push(x); else if (lv && Number(lv[2]) > Number(/\d+$/.exec(beats[at])![0])) beats[at] = x;
    }
    out.packages = beats;
  }
  return out;
}
/** Cut 28 §1–2: the sworn oath's night adds up across slices of one oath (a kept one wins: its reward was granted), and the report's
 *  decisions (`lead`, the core's first screen) merge by kind — the later slice's word for a kind, the oath's rebuilt from the merged tally,
 *  in the core's order (oath · plateau · counter · record · death · driven · bounty · pending), ≤ 4. */
function mergeLead(a: ReturnReport, b: ReturnReport): Pick<ReturnReport, "oath" | "lead"> {
  const oath = !a.oath ? b.oath : !b.oath ? a.oath : a.oath.id !== b.oath.id ? (a.oath.done ? a.oath : b.oath)
    : { ...b.oath, runs: a.oath.runs + b.oath.runs, kept: a.oath.kept + b.oath.kept, done: a.oath.done || b.oath.done, reward: b.oath.reward ?? a.oath.reward,
        // Cut 28b: the sends that broke it add up; the cause is the slice's that broke it most
        broken: (a.oath.broken ?? 0) + (b.oath.broken ?? 0), cause: (b.oath.broken ?? 0) >= (a.oath.broken ?? 0) ? b.oath.cause ?? a.oath.cause : a.oath.cause ?? b.oath.cause };
  if (!a.lead && !b.lead) return oath ? { oath } : {};
  const ORDER = ["oath", "plateau", "counter", "record", "death", "driven", "bounty", "pending"];
  const by = new Map<string, { k: string; text: string }>();
  for (const l of [...(a.lead ?? []), ...(b.lead ?? [])]) if (l.k !== "plateau" || (b.lead ?? []).some((x) => x.k === "plateau")) by.set(l.k === "counter" ? `counter:${l.text}` : l.k, l);
  if (oath && (a.oath && b.oath)) by.set("oath", { k: "oath", text: oath.done ? /* copy:none */ `oath kept: ${oath.text}` : oath.broken && oath.cause ? /* copy:none */ `oath broken: ${oath.cause} ×${oath.broken}` : /* copy:none */ `oath: ${oath.text} · ${oath.kept}/${oath.runs}` });
  const rank = (k: string): number => { const i = ORDER.indexOf(k.split(":")[0]); return i < 0 ? ORDER.length : i; };
  const lead = [...by.entries()].sort((x, y) => rank(x[0]) - rank(y[0])).map(([, l]) => l).slice(0, 4);
  return { ...(oath ? { oath } : {}), lead };
}

/** Cut 12 §1: the set's engagement row — the first `attack` / `shoot` — where a bought card goes; the end when there is none. */
export function engagementRow(rows: Row[]): number { const i = rows.findIndex((r) => r.verb.v === "attack" || r.verb.v === "shoot"); return i < 0 ? rows.length : i; }
/** QA 308f045 (core `meta::safety_end`): the place under the safety rows (a heal drink, a return or a bank on hp alone) above the engagement row. */
export function safetyEnd(rows: Row[]): number {
  const safe = (r: Row): boolean => (r.verb.v === "return" || r.verb.v === "bank" || (r.verb.v === "drink" && r.verb.a === "heal")) && r.conds.length > 0 && r.conds.every((c) => c.k === "hp<");
  let end = 0; rows.slice(0, engagementRow(rows)).forEach((r, i) => { if (safe(r)) end = i + 1; });
  return end;
}
export const cloneRow = (r: Row): Row => ({ conds: r.conds.map((c) => ({ ...c })), verb: { ...r.verb }, ...(r.origin ? { origin: r.origin } : {}) });
/** Cut 7 §2: a row's identity for origin carry-over (tokens only, never the origin). */
const rowKey = (r: Row): string => `${r.conds.map((c) => `${c.k}|${c.n ?? ""}|${c.t ?? ""}`).join(" ")} → ${r.verb.v}|${r.verb.a ?? ""}`;
const asOrigin = (o: unknown): RowOrigin | undefined => (o === "preset" || o === "patch" || o === "card" || o === "player" ? o : undefined);
export const cloneSet = (s: RuleSet): RuleSet => ({ rows: s.rows.map(cloneRow), name: s.name, ...routeOf(s) });
/** Cut 26 §2: a set's route (the fork depths whose far stair it takes) — carried whole, absent when it takes every near stair. */
export const routeOf = (s: RuleSet): { route?: number[] } => { const r = (s as RuleSet & { route?: number[] }).route; return r?.length ? { route: [...r] } : {}; };
/** Cut 22 §3: two sets with the same rows (text and order; origins aside). */
/** QA 778fa1b: a move with every delta at 0 (a dead edit's) — the line reads `≈`, the marks show nothing. */
const flatVs = (v: ForecastVs): ForecastVs => {
  const z = (m: ForecastVs["bank"]): ForecastVs["bank"] => (m === undefined ? m : { delta: 0 });
  return { ...v, depths: v.depths.map((d) => ({ ...d, delta: 0, pm: 0 })), bank: z(v.bank), death: z(v.death), return: z(v.return), gold: z(v.gold), stall: v.stall ? { delta: 0 } : undefined };
};
const sameSet = (a: RuleSet, b: RuleSet): boolean => a.rows.length === b.rows.length && a.rows.every((r, i) => rowKey(r) === rowKey(b.rows[i]))
  && JSON.stringify(routeOf(a).route ?? []) === JSON.stringify(routeOf(b).route ?? []);   // Cut 26 §2: a route change is an edit (`vs sent` reads it)

/** `dev` is non-null in dev builds or with `?dev=1` (main.ts): boot options plus `window.__riddle` for inspection
 *  (`__riddle.screen`, `__riddle.text()`, `__riddle.engineBusy`, `__riddle.booted`, and the App itself). */
export function start(dev: DevOptions | null = null): void {
  const root = document.getElementById("app") ?? document.body.appendChild(document.createElement("div"));
  root.id = "app";
  applySkin();   // Cut 17: the frames packed in web/public/ui (tools/ui-skin.py); absent ones keep the flat CSS
  const app = new App(root, dev);
  initTips(app);
  audio.arm();   // Cut 10 §4: the WebAudio context opens on the first gesture
  if (dev) (window as unknown as { __riddle: App }).__riddle = app;
  // Cut 14 §4: the cue log is readable on every build (a rater assesses sound on the cohort build); the App stays dev-only
  Object.defineProperty(window, "__audio", { value: audio, writable: false, configurable: true });
  void app.boot().catch((e) => console.error("boot failed", e));
  if (import.meta.env.PROD && "serviceWorker" in navigator) {
    window.addEventListener("load", () => { navigator.serviceWorker.register(`${import.meta.env.BASE_URL}sw.js`, { scope: import.meta.env.BASE_URL }).catch(() => { /* offline-first is best effort */ }); });
  }
}
