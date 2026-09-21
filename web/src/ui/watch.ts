// Watch: viewer canvas full-bleed; HUD (hp, depth, alert), speed fights · fast, ▶▶| skip, ⏸, callout ticker, bail.
// Cut 9 §9: `slow` is gone; Cut 10 §1: `auto` is now `fast` (8× through dead stretches, 1× near) and the default is `fights`.
//
// Cut 10 §1 — the watch is the fights. In `fights` mode the map is never watched: while the frame is the map, an
// interstitial card (`D3 · 4 rooms · $47`, the ambient line) covers the viewer for at least CARD_MS while the engine runs
// the travel at 16× underneath (bigger batches, a longer lead, a faster pump); the moment a scene opens the fight frame cuts
// in at 1× on the fight's first tick (the viewer is seeked back if its clock overshot under the card). Tapping the card holds
// the map at 8× (the old auto) until the next fight. `▶▶|` in `fights` steps the engine to the next fight (judged on its
// snapshots, floors drained on the way) and seeks the viewer to that first frame in one press; inside a fight it jumps to
// the fight's end. `data-mode`, `data-card` and `data-fights` (fights shown so far) on the element for tooling.
// Cut 10 §4 — cues (web/src/audio.ts) fire as their events are released at the viewer's clock: hit · slay · rule · telegraph
// · exit · level; the combat ones only while the fight frame is up or the clock runs at 1×.
// Cut 9 §5: the exit event's trace (every tier) rides on the exit sheet as a `trace` chip and on the report's exit line.
//
// Cut 13 §1 — a stall is a run the player can read: the stake reads `keeps $0 · stalling` while the guard has fired
// (`Stake.stalling`), and a run that comes home stalled (`… · stalled` on its line) gets the verdict screen like a death
// (`engine.death(runId)` answers with `verdict: "stall"`; an older core without the record falls back to the report).
// Cut 13 §4 — the beats are on screen: a `note` whose text is a situation's (`A den. Something sleeps.`, a theft, the cage,
// the captive, the stray, a heir's bones) opens the fight frame for SCENE_MS in `fights` and `fast` alike, the note as the
// callout (engine data, verbatim, its own `.beat` line); two callouts on one tick queue; the ticker wraps, never clips.
//
// Cut 12 §6 — `fast` is faster: travel at 16×, a fight at 2× (`fights` keeps its fights at 1×). `▶▶|` in `fast` is the same
// press as in `fights`: the next fight's first frame (the mode's own frame predicate, floors drained on the way) or, inside a
// fight, its end — it used to seek "the next interesting event", which on a busy floor sat inside the engine's ≤ 32-tick lead,
// so the seek landed where the viewer already was (rater P: "did nothing on four tries"). §4: the card names the floor's
// situation (`D4 · 9 rooms · a nest`, `Snapshot.floor_twist`). A sanity refusal (`drink ✗ no use`) shows once per floor; a
// summoned ally's fall reads `ally hound fell`; the boss's rally names the boss (`warlord rallies`, from its telegraph).
//
// Pacing (Addendum E): the viewer owns the clock (10 ticks/s × speed). The engine worker is pumped in
// 10-tick batches whenever it is fewer than LEAD ticks ahead of the viewer, so events always arrive
// before the viewer needs them and the engine never runs far ahead (≤ 22 ticks, under the viewer's
// dead-air threshold). HUD hp/depth and the ticker are queued by tick and released at the viewer's
// clock, so what the numbers say matches what the sprites do.
//
// Cut 5 §5 — auto cadence (the default): the clock runs at 8× through dead stretches and drops to 1× while
// anything is near. "Near" is read off the engine, which is ≤ 22 ticks ahead of the viewer: a hostile in view
// or an item within 3 tiles in a step's snapshot, a telegraph / attack / hero hp change in its events. Each
// sighting holds 1× until AUTO_TAIL ticks after it (so hp unchanged for 20 ticks is the fast condition), and the
// viewer slows *before* the foe walks into frame because the engine saw it first. At 8× the pump runs the same
// LEAD with a 50 ms interval and a larger batch, so the engine still never runs dry or far ahead. Bail (§5)
// turns the auto rate into a flat 8× and the stake line reads `returning` until the exit sheet.
//
// Cut 5 §4 — the vault choice: a step whose snapshot carries `vault_choice` opens a sheet with the three items
// as chips and holds the clock at 1×; a tap is `choose(id)`. The engine's 50-tick grace runs watched or not, so
// an unanswered sheet closes on its own when `vault_choice` leaves the snapshot (the lineage's `vault_pref` took).
//
// Cut 7 §4 — rooms are scenes: a step whose `snapshot.room` holds ≥ SCENE_FOES awake hostiles (absent `room`: that many
// hostiles in view) opens a scene, and auto holds 1× until the room is clear or the hero leaves it (the id changes),
// AUTO_TAIL after; corridors and empty rooms run at 8×, telegraphs / attacks / hp loss still hold 1× on their own. A
// hostile in view or an item near no longer slows on its own (that was "30–60 s of pick up"). Ambient callouts keep the
// fast stretches legible: `D3 · 4 rooms` on a floor, `$47` on gold, `alert 3` when the clock ticks — one per 10 s,
// shown at 4× and up (the floor line at any speed). The run's end plays at 1×: a batch carrying the `exit` is held
// back from the viewer until its clock is ENDING_TICKS from the exit (so no dead-air jump swallows the walk-out), and
// auto runs 1× from there (a core `ending` marker sets the same point). At 8× the pump keeps the engine ≥ LEAD_FAST
// ticks ahead so the exit is seen in time.
//
// Cut 8A — the fight frame (docs/PLATEAU.md §A): the viewer cuts to its close fight framing (`setFrame("fight")`) when a
// scene opens, when any hostile stands adjacent to the hero, or when a boss is in view, and cuts back to the map AUTO_TAIL
// ticks after the last of those stops holding. Entry and exit are released at the viewer's clock (the engine runs ahead),
// so the cut lands when the foes are on screen. The fight frame runs at 1× whatever the mode; the map frame keeps the
// cadence above. `data-frame="map|fight"` on the element for tooling.
import type { App, Mounted } from "../app";
import type { Ev, ExitLine, Highlight, InvItem, ReturnReport, Row, Snapshot, StepResult, Trace, VaultChoice } from "../engine/types";
import { h, items, replace, spanOf } from "./dom";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt, xpToNext } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { vaultSlots } from "./unlocks";
import { kindGlyph, verbLabel } from "./tokens";
import { traceChip } from "./trace";
import { markEnd, recordRun } from "./runlog";
import { audio } from "../audio";

type Tier = "bank" | "return" | "death";
type FrameName = "map" | "fight";
/** Cut 8A: the real renderer's frame cut (web/src/render/index.ts); the placeholder viewer has none. */
type FrameViewer = Viewer & { setFrame?(frame: FrameName, focus?: { x: number; y: number; radius: number }): void };
const LEAD = 12, BATCH = 10;        // ticks: pump when the engine is < LEAD ahead; step BATCH at a time
const BATCH_FAST = 12;              // at 8× the viewer eats 4 ticks per pump; a bigger batch keeps the queue fed through a slow step
const LEAD_FAST = 32;               // Cut 7 §4: at 8× the engine stays ≥ ENDING_TICKS ahead, so an exit is seen before its last 30 ticks
const BATCH_FIGHTS = 16;            // Cut 10 §1: under the card the engine steps this many ticks per call, chained flat out (≈ 1 ms a call)
const CARD_MS = 1500, CARD_BEAT_MS = 500;   // Cut 10 §1: the interstitial's minimum on a new floor (`D4 · 17 rooms`), and between fights on the same floor
const FIGHT_TAIL = 12;              // Cut 10 §1: in `fights` the frame lets go this many ticks after the last hold (the kill's dissolve), not AUTO_TAIL
const BLOW_TICKS = 20;              // Cut 10 §1: in `fights` the frame holds on an adjacent hostile, a blow in the last 20 ticks, or a boss in view
// Cut 10 §1: under the card the engine runs each fight to its end first and only a fight that cost something is shown — the hero
// hurt ≥ SHOW_HURT hp in it, or under SHOW_HP of max hp, a boss, a companion fallen, a theft, or the run ending in it; the rest
// pass under the card (a DEFAULT run's fights alone ran ~110 s at 1×; the gate is 90 s with ≥ 3 shown)
const SHOW_HURT = 4, SHOW_HP = 0.25;
const SKIP_FIGHT_BATCHES = 12_000;  // Cut 10 §1: ▶▶| steps to the next fight or the run's end (the run cap in BATCHes; ≈ 4 000 ticks
                                    // landed on a paced stretch that looked the same — "inert", three QA players on Cut 12)
const ENDING_TICKS = 30;            // Cut 7 §4: the last ticks before any exit play at 1×
const SCENE_FOES = 2;               // Cut 7 §4: awake hostiles in the hero's room that make it a scene
const AMBIENT_MS = 10_000, AMBIENT_SHOW_MS = 1500;   // Cut 7 §4: one ambient callout per 10 s, shown 1.5 s whatever the speed
const PUMP_MS = 25;
type Mode = "fights" | "fast";
const RATE: Record<Mode, number> = { fights: 16, fast: 16 };  // fights: the map when it shows without a hold (draining to an exit); fast: the travel (Cut 12 §6, was 8×)
const FAST_NEAR = 2;                // Cut 12 §6: `fast` watches a fight (and anything near) at 2×; `fights` keeps 1×
const AUTO_FAST = 8, AUTO_TAIL = 20; // a tapped card holds the map at 8×; the pump's "fast" threshold; near holds until AUTO_TAIL ticks after the last sighting / hp change
const CALLOUT_MIN_MS = 500;         // Cut 12 §6: a callout stays readable at 16×
const EXIT_GRACE_MS = 4000;         // wait for the viewer to drain after an exit, at most this long
const PERSIST_MS = 5000;
const BOSS_BANNER_MS = 3000;        // Cut 2 §7: `boss · counter: known|unknown` on first sight
const REST_BEAT_MS = 1400;          // Cut 2 §1: `rest 12m` after the exit, before the exit flow continues
const CHORE_CALLOUT: Record<string, string> = { descend: /* copy:callout */ "descend", pick_up: /* copy:callout */ "pick up" }; // explore never (Cut 4 §4)
const HURT_MS = 600;                // Cut 4 §4: `−7 archer` in red
const FELL_MS = 1400;               // Cut 10 §3: `jackal Ashar fell` stays long enough to read a name
const SCENE_MS = 4000, SCENE_TICKS = 40;   // Cut 13 §4: a situation's beat holds the fight frame this long (~4 s at 1×)
const CALLOUT_QUEUE = 3, SAME_TICK_MS = 40;   // Cut 13 §4: callouts that land on one tick (one pump pass) wait their turn, at most this many
/** Cut 13 §4: the notes that are beats — the core's situation lines (verbatim), a theft, the stray, a heir's bones. */
const BEAT_RE = /^(A den\.|A cage:|A shrine\.|The cage opens|A cry from the dark|The air stings|The nest wakes|The den wakes|Found heir \d+'s bones|found the bones|Freed the captive|Cut the captive|Lit the shrine)|snatched|\bstole\b|, gone wild\.$|is avenged/i;
/** The cause of a `hurt` as one word: `goblin_archer` → `archer`. */
const oneWord = (cause: string): string => cause.replace(/_/g, " ").trim().split(/\s+/).pop() ?? "";
/** Cut 12 §4: `nest` → `a nest`, `orchard` → `an orchard` (one word after the article). */
export const withArticle = (w: string): string => { const x = oneWord(w); return `${/^[aeiou]/i.test(x) ? "an" : "a"} ${x}`; };

export function renderWatch(app: App): Mounted {
  const canvas = h("canvas", { class: "view" });
  const hpFill = h("span", { class: "fill" });
  const hpText = h("span", { class: "num" });
  const depth = h("span", { class: "num depth" });
  const alert = h("span", { class: "alert num" });
  const ticker = h("div", { class: "ticker" });
  const stake = h("div", { class: "stake num" });
  const banner = h("div", { class: "banner num" });
  const pause = h("button", { class: "hud-btn", onclick: () => togglePause() }, "⏸");
  // the last chosen mode is the next run's (app.watchMode, persisted — QA on e0f87e7: "`fast` chosen in run 3 was not remembered")
  const mode0: Mode = app.watchMode === "fast" ? "fast" : "fights";
  const modeBtn: Record<Mode, HTMLButtonElement> = {
    fights: h("button", { class: `hud-btn${mode0 === "fights" ? " on" : ""}`, onclick: () => setMode("fights") }, /* copy:button */ "fights"),
    fast: h("button", { class: `hud-btn${mode0 === "fast" ? " on" : ""}`, onclick: () => setMode("fast") }, /* copy:button */ "fast"),
  };
  const skip = h("button", { class: "hud-btn", onclick: () => skipToEvent() }, "▶▶|");
  const bail = h("button", { class: "hud-btn bail", onclick: () => doBail() }, /* copy:button */ "bail");
  // Cut 10 §1: the interstitial — the ambient line over the map while the travel runs underneath; a tap holds the map at 8×
  const card = h("button", { class: "interstitial num", hidden: true, onclick: () => holdMap() });
  const el = h("main", { class: "watch" }, canvas, card,
    h("div", { class: "hud top" }, h("div", { class: "hp" }, h("span", { class: "track" }, hpFill), hpText), depth, alert, pause, stake),
    banner, ticker,
    h("div", { class: "hud bottom" }, modeBtn.fights, modeBtn.fast, skip, bail));

  let viewer: Viewer | null = null;
  let mode: Mode = mode0, paused = false, slowUntil = -Infinity, lastHp = NaN;
  // Cut 10 §1: the card's state — up, since when (its minimum), a tap holding the map, a fight waiting on the minimum; fights shown
  let cardUp = false, cardSince = 0, cardMin = CARD_MS, cardDepth = 0, cardText = "", mapHold = false, cardWait = false, fights = 0;
  el.dataset.mode = mode; el.dataset.fights = "0"; el.dataset.card = "0";
  const allies = new Set<number>();   // Cut 10 §3: ids the snapshot flags as allies, so a companion's death reads `jackal Ashar fell`
  let fell: { t: number; name: string; kind: string } | null = null;   // the companion whose death the core's own callout (`Ashar fell`) names next
  let heroCause: string | undefined;  // the hero's `die` cause (the client-built report counts the death it came from)
  // Cut 7 §4: the scene's room (null = none) and the tick auto may run fast again after one ends; the ending's first tick;
  // the exit batch held back until the viewer is ENDING_TICKS from the exit; the ambient callout limiter; the last alert
  let scene: number | null = null, sceneUntil = -Infinity, endingFrom = Infinity, endingCue = -Infinity;
  // Cut 8A: the fight frame — whether the engine's latest snapshot holds it, the viewer ticks it spans, the frame shown
  let fightOn = false, fightFrom = Infinity, fightUntil = -Infinity, frame: FrameName = "map", lastBlow = -Infinity;
  // Cut 13 §4: the situation beat the frame is holding for (engine ticks), its text shown once at the cut; beats shown so far
  let beat: { from: number; until: number; text: string; shown: boolean } | null = null, beats = 0;
  // Cut 10 §1: the fight the engine is running through under the card (its cost so far), and whether the found fight is to be shown
  let probe: { hurt: number; low: boolean; boss: boolean; ally: boolean; steal: boolean } | null = null, fightShow = false;
  let held: { evs: Ev[]; snap: Snapshot; tier: Tier } | null = null;
  let lastAmbient = -Infinity, ambientUntil = 0, lastAlert = 0;
  const refused = new Set<string>();  // Cut 12 §6: sanity refusals shown (`drink ✗ no use@3`): once per text per floor
  let rallyBy: string | undefined;    // Cut 12 §6: the kind whose `rallies` telegraph came last, so the core's `rallied!` names it
  let speed = 1, done = false, disposed = false, overridden = false, tickerTimer = 0, bannerTimer = 0, pumpTimer = 0;
  let tickerAt = 0, tickerMs = 0; const tickerQueue: { text: string; cls: string; ms: number }[] = [];   // Cut 13 §4: callouts waiting their turn
  // Cut 2: rest after the exit, bones left (death) / found, bosses already announced
  let restS: number | undefined, restUntil = 0, bonesLeft: number | undefined;
  let exitLine: ExitLine | undefined;   // Cut 6 §1: the exit's ledger line (exit sheet, report, death)
  let exitTrace: Trace | undefined;     // Cut 9 §5: the exit's last-5 trace (on the event, or on its line)
  const bonesFound: string[] = []; const bossSeen = new Set<number>();
  let counters = app.lineage.counters ?? [];   // Cut 6 §5: bosses with a named counter row, re-read on a sighting
  let snap: Snapshot | null = null;
  let runId = -1, engineTick = 0, startTick = 0, inflight = false, lastPersist = performance.now();
  let pendingLoad: { snap: Snapshot; rest: Ev[] } | null = null;
  let exitTier: Tier | null = null, exitAt = 0;
  let pendingExit: { items: InvItem[]; tier: string; worth?: number[] } | undefined;
  // what the exit sheet let go, at the engine's worth — the report's `salvaged` rows (QA on 952e306: "camp $76 after
  // 'Returned with $57'; only the gold sheet shows +$19 salvage"); the deepest floor this send reached (its `deepest` tile)
  let salvagedRows: { kind: string; n: number; gold: number }[] = [];
  let deepest = 0;
  // Cut 5 §4: the open vault sheet's close, and the cage it was opened for (a dismissed sheet is not reopened)
  let vaultClose: (() => void) | null = null, vaultKey = "";
  let prepended = false;              // bail fell back to the row prepend (an engine without `bail`)
  const cls = app.lineage.class;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  const kinds = new Map<number, string>(), names = new Map<number, string>();
  const partyAtStart = (app.lineage.party ?? []).map((c) => `${c.kind} · ${c.name}`);
  const tamedIds: number[] = [], lostIds: number[] = [];
  const compLabel = (id: number): string => { const n = names.get(id); return `${kinds.get(id) ?? "?"}${n ? ` · ${n}` : ""}`; };
  const note_ = (e: { id: number; kind: string; name?: string }): void => { kinds.set(e.id, e.kind); if (e.name) names.set(e.id, e.name); };
  // HUD updates released at the viewer's clock
  const hud = { hp: 0, maxHp: 1, depth: 1 };
  const timed: { t: number; f: () => void }[] = [];
  let lastRuleText = "", lastRuleAt = 0, lastShown = "";
  // placeholder viewer (no clock): a wall clock at 10 ticks/s × speed stands in
  let fbTick = 0, fbAt = performance.now();
  function viewerTick(): number {
    if (viewer?.tick) return viewer.tick();
    const now = performance.now(); fbTick += ((now - fbAt) / 1000) * 10 * speed; fbAt = now;
    return Math.min(fbTick, engineTick);
  }
  const viewerIdle = (): boolean => viewer?.idle ? viewer.idle() : true;

  function paintHud(): void {
    const p = hud.maxHp ? hud.hp / hud.maxHp : 0;
    hpFill.style.width = `${Math.round(Math.max(0, p) * 100)}%`;
    hpFill.classList.toggle("low", p < 0.3);
    stake.classList.toggle("warn", p < 0.4);
    replace(hpText, `${Math.max(0, hud.hp)}/${hud.maxHp}`);
    replace(depth, `D${hud.depth}`);
    if (snap) replace(alert, "!".repeat(Math.max(0, Math.min(5, snap.alert))));
  }
  function hudFrom(s: Snapshot): void { hud.hp = s.hero.hp; hud.maxHp = s.hero.max_hp; hud.depth = s.depth; deepest = Math.max(deepest, s.depth); paintHud(); paintStake(s); }
  // Cut 2 §7: `$47 · sword⚠ · return at D4`; `death: lose all` when no row would bank or return
  function paintStake(s: Snapshot): void {
    const st = s.stake;
    stake.hidden = !st;
    if (!st) return;
    const parts: (string | HTMLElement)[] = [`$${st.loot}`];
    // Cut 6 §1: the kept number while a return/bank row exists (`$84 · keeps $50`)
    // Cut 13 §1: while the guard has fired a stall pays nothing, and the line says so before it is lost (`keeps $0 · stalling`)
    if (st.stalling && !overridden) parts.push(" · ", h("span", { class: "kept stalling" }, /* copy:callout */ "keeps $0 · stalling"));
    else if (st.kept !== undefined && !overridden) parts.push(" · ", h("span", { class: "kept" }, /* copy:callout */ `keeps $${st.kept}`));
    for (const b of st.brought) parts.push(" · ", h("span", { class: b.insured ? "" : "risk" }, b.label, b.insured ? "" : "⚠"));
    if (overridden) parts.push(" · ", h("span", { class: "returning" }, /* copy:callout */ "returning"));
    else if (st.return_row === undefined) parts.push(" · ", h("span", { class: "lose" }, /* copy:callout */ "death: lose all"));
    else parts.push(" · ", returnAt(app.rules.rows[st.return_row], st.return_row));
    replace(stake, ...parts);
  }
  function returnAt(row: Row | undefined, i: number): string {
    const v = verbLabel({ v: row?.verb.v ?? "return" });
    const d = row?.conds.find((c) => c.k === "depth>=" && c.n !== undefined); if (d) return /* copy:callout */ `${v} at D${d.n}`;
    const hp = row?.conds.find((c) => c.k === "hp<" && c.n !== undefined); if (hp) return /* copy:callout */ `${v} at ${hp.n}%`;
    return `${v} R${i + 1}`;
  }
  function showBanner(text: string, ms: number, cls = ""): void {
    replace(banner, text); banner.className = `banner num show ${cls}`;
    clearTimeout(bannerTimer); bannerTimer = window.setTimeout(() => banner.classList.remove("show"), ms);
  }
  function bossSighted(s: Snapshot): void {
    for (const e of s.entities) {
      if (!e.tags.includes("boss") || bossSeen.has(e.id) || !s.visible[e.y * s.w + e.x]) continue;
      bossSeen.add(e.id);
      const isFact = (f: string): boolean => f === `boss:${e.kind}:counter` || f.startsWith(`boss:${e.kind}:counter=`);   // Cut 6: the fact may carry the row
      const known = (): boolean => app.lineage.facts.some(isFact) || learned.some(isFact);
      // Cut 6 §5: the counter named as a row (`boss · counter: attack boss`). The fact lands on the sighting step itself (the
      // core learns it on the boss's first telegraph), so the lineage is re-read as the banner is released at the viewer's clock.
      at(s.turn, () => {
        const show = (): void => {
          const named = counters.find((c) => c.boss === e.kind)?.text;
          showBanner(named ? /* copy:callout */ `boss · counter: ${named}` : known() ? /* copy:callout */ "boss · counter: known" : /* copy:callout */ "boss · counter: unknown", BOSS_BANNER_MS, "boss");
        };
        app.engine.lineage().then((L) => { counters = L.counters ?? counters; }).catch(() => { /* the mounted lineage's counters stand */ }).finally(() => { if (!disposed) show(); });
      });
    }
  }
  function callout(text: string, cls = "", ms: number = Math.max(CALLOUT_MIN_MS, 1800 / Math.max(1, speed))): void {
    if (cardUp) return;                                                                    // Cut 10 §1: nothing under the card is watched
    if (cls !== "ambient" && cls !== "hurt" && cls !== "beat" && performance.now() < ambientUntil) return;   // Cut 7 §4: an ambient keeps the ticker for its 1.5 s
    // Cut 13 §4: a second callout on the same tick (one pump pass) waits its turn instead of replacing the first before it was
    // read (rater Q: two labels on one line); while a queue is pending later ones join it in order (the oldest drops past the
    // cap); otherwise a later tick's callout takes the ticker at once, as before
    const showing = ticker.classList.contains("show");
    if (showing && text !== lastShown && (performance.now() - tickerAt < SAME_TICK_MS || tickerQueue.length)) {
      tickerQueue.push({ text, cls, ms: Math.min(ms, CALLOUT_MIN_MS) }); while (tickerQueue.length > CALLOUT_QUEUE) tickerQueue.shift();
      scheduleTicker(); return;
    }
    tickerQueue.length = 0;
    showTicker(text, cls, ms);
  }
  function showTicker(text: string, cls: string, ms: number): void {
    lastShown = text; tickerAt = performance.now(); tickerMs = ms;
    replace(ticker, text); ticker.className = `ticker show ${cls}`;
    scheduleTicker();
  }
  /** The ticker's next move: the queued callout once the current has had CALLOUT_MIN_MS, else the hide at the current's end. */
  function scheduleTicker(): void {
    clearTimeout(tickerTimer);
    const shownFor = performance.now() - tickerAt;
    const wait = Math.max(0, (tickerQueue.length ? CALLOUT_MIN_MS : tickerMs) - shownFor);
    tickerTimer = window.setTimeout(() => { if (disposed) return; const q = tickerQueue.shift(); if (q) showTicker(q.text, q.cls, q.ms); else ticker.classList.remove("show"); }, wait);
  }
  /** Cut 13 §4: a situation's note opens the fight frame for SCENE_TICKS from its tick (or rides a fight already framed there);
   *  its text is the callout, shown once the frame is up. */
  function beatAt(t: number, text: string): void {
    const v = viewerTick();
    const framed = fightOn || (v < fightUntil && t >= fightFrom);
    if (!framed) { fightFrom = t; fightUntil = t + SCENE_TICKS; fightShow = true; }
    else if (Number.isFinite(fightUntil)) fightUntil = Math.max(fightUntil, t + SCENE_TICKS);
    beat = { from: t, until: t + SCENE_TICKS, text, shown: false };
    el.dataset.beats = String(++beats);   // dev: tools count the beats cut in
    at(t, () => showBeat());
  }
  function showBeat(): void {
    if (!beat || beat.shown || frame !== "fight") return;
    beat.shown = true; callout(beat.text, "beat", SCENE_MS);
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
    if (ev.row >= 0) return `R${ev.row + 1} · ${verbLabel(ev.verb)}`;
    if (ev.row === -1) return ev.text;                       // trait deviation, e.g. "cowardly → retreat"
    return CHORE_CALLOUT[ev.verb.v] ?? null;                  // chores are silent (pillar 2)
  }
  // Cut 7 §4: ambient callouts — at most one per AMBIENT_MS, only while the clock runs fast; the floor line shows at any
  // speed and is never skipped for an earlier one (it still starts the 10 s)
  function ambient(text: string, always = false): void {
    const now = performance.now();
    if (cardUp) return;   // Cut 10 §1: the card is the ambient line
    if (!always && (speed < AUTO_FAST || now - lastAmbient < AMBIENT_MS)) return;
    lastAmbient = now; ambientUntil = now + AMBIENT_SHOW_MS;
    callout(text, "ambient", AMBIENT_SHOW_MS);
  }
  function at(t: number, f: () => void): void { timed.push({ t, f }); }
  function release(upTo: number): void {
    if (!timed.length) return;
    const keep: typeof timed = [];
    for (const x of timed) { if (x.t <= upTo) x.f(); else keep.push(x); }
    timed.length = 0; timed.push(...keep);
  }
  const victims = new Map<number, string>();   // id → label, remembered across batches so a kill inside a batch still has a name
  function absorb(evs: Ev[], s: Snapshot): Tier | null {
    let exit: Tier | null = null;
    const heroId = s.hero.id;
    for (const e of s.entities) { if (e.ally) allies.add(e.id); else if (e.id !== heroId) victims.set(e.id, (e.name ?? e.kind).replace(/_/g, " ")); }
    for (const ev of evs) {
      if (ev.k === "telegraph" || ev.k === "attack" || ev.k === "use" || (ev.k === "hurt" && ev.id === heroId)) near(ev.t);   // Cut 5 §5: always at 1×
      switch (ev.k) {
        // Cut 10 §3: the core's companion-death callout (`Ashar fell`) gets its kind in front: `jackal Ashar fell`
        case "callout": {
          if (ev.text === "explore") break;
          // Cut 12 §6: a sanity refusal (`drink ✗ no use`) shows once per floor, not once per streak
          if (ev.text.includes("✗")) { const key = `${ev.text}@${s.depth}`; if (refused.has(key)) break; refused.add(key); }
          let f = ev.text;
          // Cut 12 §6: a summoned ally (no name) reads `ally hound fell`; a companion keeps `jackal Ashar fell`
          if (fell && fell.t === ev.t && fell.kind && ev.text === (fell.name ? `${fell.name} fell` : `${fell.kind} fell`)) f = fell.name ? `${fell.kind} ${fell.name} fell` : /* copy:callout */ `ally ${oneWord(fell.kind)} fell`;
          // Cut 12 §6: the core's `rallied!` names the boss whose telegraph it answers (`warlord rallies`)
          else if (ev.text === /* copy:none */ "rallied!") f = /* copy:callout */ `${oneWord(rallyBy ?? "boss")} rallies`;
          at(ev.t, () => callout(f, f !== ev.text && f.endsWith(" fell") ? "hurt" : "", f !== ev.text && f.endsWith(" fell") ? FELL_MS : undefined));
          break;
        }
        case "rule": {
          const text = ruleCallout(ev);
          // a chore (`pick up`) shows once per streak: not again until another callout intervened
          if (text) at(ev.t, () => { const now = performance.now(); if (ev.row === -2 ? text !== lastShown : text !== lastRuleText || now - lastRuleAt > 4000) callout(text); lastRuleText = text; lastRuleAt = now; });
          if (ev.row >= 0) at(ev.t, () => cue("rule"));   // Cut 10 §4: a player row, never a chore or a trait
          break;
        }
        case "hurt": if (ev.id === heroId) at(ev.t, () => { hud.hp = ev.hp; paintHud(); if (ev.dmg > 0) { callout(`−${ev.dmg} ${oneWord(ev.cause)}`, "hurt", HURT_MS); cue("hit", { dmg: ev.dmg }); } }); break;
        // the kill gets its own line (cohort 5: "−3 goblin" was still up after the goblin had dissolved)
        case "die": {
          if (ev.id === heroId) { heroCause = ev.cause; break; }
          // Cut 10 §3: a companion's death (the snapshot's ally flag) is the core's callout to name; the kill line is for hostiles
          if (allies.has(ev.id)) { fell = { t: ev.t, name: names.get(ev.id) ?? "", kind: (kinds.get(ev.id) ?? "").replace(/_/g, " ") }; break; }
          const v = victims.get(ev.id); if (v) at(ev.t, () => { callout(/* copy:callout */ `${v} slain`, "kill", HURT_MS); cue("slay"); });
          break;
        }
        case "telegraph": if (ev.what === "rallies") rallyBy = kinds.get(ev.id) ?? rallyBy; at(ev.t, () => cue("telegraph")); break;
        // Cut 10 §3: a theft names its amount when the engine sends one (`stolen $16`)
        case "steal": if (ev.amount !== undefined && ev.amount > 0) { const n = ev.amount; at(ev.t, () => callout(/* copy:callout */ `stolen $${n}`, "hurt", FELL_MS)); } break;
        case "descend": {
          const rooms = s.depth === ev.depth ? s.rooms : undefined;   // Cut 7 §4: `D3 · 4 rooms` when the snapshot counts them
          at(ev.t, () => { hud.depth = ev.depth; paintHud(); ambient(rooms ? /* copy:callout */ `D${ev.depth} · ${rooms} rooms` : `D${ev.depth}`, true); });
          break;
        }
        case "fact": {
          learned.push(ev.fact);
          // Cut 6 §5: the counter learned mid-fight (the boss's first telegraph) names itself: `boss · counter: attack boss`
          const m = /^boss:([a-z_]+):counter(?:=|$)/.exec(ev.fact);
          if (m) at(ev.t, () => { app.engine.lineage().then((L) => { counters = L.counters ?? counters; }).catch(() => { /* keep */ }).finally(() => {
            const named = counters.find((c) => c.boss === m[1])?.text; if (named && !disposed) showBanner(/* copy:callout */ `boss · counter: ${named}`, BOSS_BANNER_MS, "boss");
          }); });
          break;
        }
        case "pickup": {
          if (ev.id === heroId) found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item });
          const gold = /^gold\b\D*(\d+)/.exec(ev.item);   // Cut 7 §4: `$47` on a gold pickup (`gold (47)` core, `gold 47` fake)
          if (gold) at(ev.t, () => ambient(`$${gold[1]}`));
          break;
        }
        case "note":
          notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text });
          if (BEAT_RE.test(ev.text)) beatAt(ev.t, ev.text);   // Cut 13 §4: the situations cut in like fights
          break;
        case "exit": {
          exit = ev.tier; exitLine = ev.line ?? exitLine; exitTrace = ev.trace ?? ev.line?.trace ?? exitTrace;
          markEnd(runId, ev.t);   // Cut 11 §2: the run log's last replayable tick
          // Cut 7 §4: the last ENDING_TICKS play at 1×; the core's `ending` marker counts only when the exit follows it closely
          // (Cut 10 §1: a foreseen death the hero survived held the map at 1× for minutes)
          endingFrom = Math.min(endingFrom, ev.t - ENDING_TICKS, endingCue >= ev.t - 100 ? endingCue : Infinity);
          const tier = ev.tier; at(ev.t, () => audio.cue(tier === "bank" ? "exit_bank" : tier === "return" ? "exit_return" : "exit_death"));           // Cut 10 §4
          break;
        }
        case "ending": endingCue = ev.t; break;                                                                                       // Cut 7 §4: the core's marker (see `exit`)
        case "tame": if (ev.ok) { tamedIds.push(ev.id); kinds.set(ev.id, ev.kind); allies.add(ev.id); victims.delete(ev.id); } break;
        case "ally": if (ev.state === "lost") lostIds.push(ev.id); else { allies.add(ev.id); victims.delete(ev.id); } break;
        case "spawn": note_(ev.e); break;
        case "level": for (const v of verbsAt(ev.class, ev.level)) learned.push(`verb:${v}`); at(ev.t, () => { callout(`${ev.class} L${ev.level}`); audio.cue("level"); }); break;
        case "rank": at(ev.t, () => callout(`★${ev.rank}`)); break;
        case "rest": restS = ev.seconds; break;
        case "bones":
          if (ev.heir === s.run.heir) bonesLeft = ev.items;                                        // this heir's kit, left on death
          else { bonesFound.push(/* copy:callout */ `D${s.depth} · ${items(ev.items)}`); at(ev.t, () => callout(`♟${ev.heir} · ${ev.items}`)); }
          break;
        default: break;
      }
    }
    return exit;
  }
  /** Cut 10 §4: a combat cue, only while the fight is watched (the fight frame up, or the clock at 1×). */
  function cue(name: "hit" | "slay" | "rule" | "telegraph", opts?: { dmg?: number }): void { if (frame === "fight" || speed <= FAST_NEAR) audio.cue(name, opts); }
  // Cut 5 §5 / Cut 7 §4: what holds auto at 1× — a scene (a room with SCENE_FOES awake hostiles), the hero's hp moving
  const hostile = (e: { ally?: boolean; kind: string; tags: string[] }): boolean => !e.ally && e.kind !== "bones" && e.kind !== "captive" && !e.tags.includes("captive") && !e.tags.includes("ally");
  /** Cut 7 §4: the hero's room and its awake hostiles; without `room` in the wire, the hostiles in view stand in (one "room"). */
  function roomOf(s: Snapshot): { id: number; hostiles: number } {
    return s.room ?? { id: -1, hostiles: s.entities.filter((e) => hostile(e) && !e.remembered && s.visible[e.y * s.w + e.x]).length };
  }
  function sceneFrom(s: Snapshot, evs: Ev[] = []): void {
    const r = roomOf(s);
    const open = r.hostiles >= SCENE_FOES || (scene !== null && r.id === scene && r.hostiles > 0);
    if (open) { scene = r.id; sceneUntil = Infinity; }
    else if (scene !== null) { scene = null; sceneUntil = s.turn + AUTO_TAIL; }
    el.dataset.scene = scene === null ? "0" : "1";   // dev: tools sample the cadence off the DOM
    fightFrom_(s, open, evs);
  }
  /** Cut 8A: the fight frame holds while a scene is open, a hostile is adjacent to the hero, or a boss is in view; it lets
   *  go AUTO_TAIL ticks after. Ticks are the engine's; `applyFrame` cuts at the viewer's clock. Cut 10 §1: the frame starts
   *  at the batch's first blow (attack, telegraph, hero hurt) when it has one, else at the batch's end — the fight's first frame. */
  function fightFrom_(s: Snapshot, sceneOpen: boolean, evs: Ev[]): void {
    const hx = s.hero.x, hy = s.hero.y;
    const seen = (e: { x: number; y: number; remembered?: boolean }): boolean => !e.remembered && !!s.visible[e.y * s.w + e.x];
    const within = (r: number): boolean => s.entities.some((e) => hostile(e) && seen(e) && Math.max(Math.abs(e.x - hx), Math.abs(e.y - hy)) <= r);
    const boss = s.entities.some((e) => e.tags.includes("boss") && hostile(e) && seen(e));
    const blow = evs.find((e) => e.k === "attack" || e.k === "telegraph" || (e.k === "hurt" && e.id === s.hero.id));
    if (blow) lastBlow = Math.max(lastBlow, ...evs.filter((e) => e.k === "attack" || e.k === "telegraph" || (e.k === "hurt" && e.id === s.hero.id)).map((e) => e.t));
    // Cut 10 §1: `fights` frames the blows — an adjacent hostile, a blow in the last BLOW_TICKS, a boss in view — not the whole scene
    // (a room's approach ran at 1× and cost the gate; the wire has no awake flag, so a sleeper two tiles off would hold it too);
    // `fast` keeps Cut 8A's scene / adjacent / boss
    const on = mode === "fights" ? within(1) || s.turn - lastBlow <= BLOW_TICKS || boss : sceneOpen || within(1) || boss;
    if (on) {
      if (!fightOn && viewerTick() >= fightUntil) { fightFrom = Math.min(s.turn, blow?.t ?? s.turn); fightShow = false; }
      fightUntil = Infinity;
    }
    else if (fightOn) fightUntil = s.turn + (mode === "fights" ? FIGHT_TAIL : AUTO_TAIL);
    fightOn = on;
  }
  /** The pending floor load, taken (a helper: TypeScript narrows `pendingLoad` to null across the awaited loop). */
  function takeLoad(): { snap: Snapshot; rest: Ev[] } | null { const p = pendingLoad; pendingLoad = null; return p; }
  /** Land the viewer's clock on tick t (both directions; the placeholder viewer's wall clock too). */
  function seekTo(t: number): void {
    const fv = viewer as (Viewer & { seek?: (t: number) => void }) | null;
    if (fv?.seek) fv.seek(t); else fv?.skipToEvent();
    fbTick = t; fbAt = performance.now();
  }
  /** Cut 10 §1: under the card the engine has found the next fight ahead of the viewer's clock (the cut seeks the viewer there). */
  const fightAhead = (v: number): boolean => cardUp && fightShow && fightFrom < Infinity && v < fightFrom;
  function applyFrame(): void {
    const v = viewerTick();
    // Cut 10 §1: under the card only a fight the probe chose is wanted (the viewer's clock holds there; the cut seeks to it)
    const wantFight = mode === "fights" && cardUp && !mapHold ? fightShow && fightFrom < Infinity && v < fightUntil : v >= fightFrom && v < fightUntil;
    let want: FrameName = wantFight ? "fight" : "map";
    // Cut 10 §1: a fight waits for the card's minimum (the clock holds at 0 meanwhile; the seek below lands on the first frame)
    cardWait = mode === "fights" && wantFight && frame === "map" && cardUp && performance.now() < cardSince + cardMin;
    if (cardWait) want = "map";
    el.dataset.span = `${fightFrom}:${fightUntil}:${fightOn ? 1 : 0}:${fightShow ? 1 : 0}:${engineTick}`;   // dev: the fight span the frame follows
    if (want === frame) { paintCard(want); return; }
    if (want === "fight") {
      fights++; el.dataset.fights = String(fights); mapHold = false;
      // the viewer lands on the fight's first frame: back if its clock overshot under the card, forward if the engine ran ahead
      // (the travel's queued HUD updates are released under the card, so the ticker is clean at the cut)
      if (mode === "fights" && Math.abs(v - fightFrom) > 1) { seekTo(fightFrom); release(fightFrom); }
    }
    paintCard(want);
    frame = want; (viewer as FrameViewer | null)?.setFrame?.(want);
    el.dataset.frame = want;
    if (want === "fight") showBeat();   // Cut 13 §4: the beat's line at the cut
  }
  /** Cut 10 §1: the interstitial is up while `fights` shows the map, unless a tap holds the map, the vault sheet is up, or the
   *  run's ending plays. Its line is the ambient one: `D3 · 4 rooms · $47`. */
  function paintCard(want: FrameName): void {
    const up = mode === "fights" && want === "map" && !mapHold && !vaultClose && !done && !exitTier && viewerTick() < endingFrom;
    if (up !== cardUp) {
      cardUp = up; card.hidden = !up; el.dataset.card = up ? "1" : "0";
      // a card per floor: the full minimum when the floor is new, a beat between fights on the same floor
      if (up) { cardSince = performance.now(); cardMin = hud.depth !== cardDepth ? CARD_MS : CARD_BEAT_MS; cardDepth = hud.depth; ticker.classList.remove("show"); }
    }
    if (!up) return;
    // the engine's floor (it runs ahead under the card: the line describes where the next fight is), not the HUD's
    // Cut 12 §4: the floor's one situation, one word after its article (`D4 · 9 rooms · a nest`); the gold line when there is none
    const d = snap?.depth ?? hud.depth, rooms = snap?.rooms, twist = snap?.floor_twist;
    const text = /* copy:callout */ `D${d}${rooms ? ` · ${rooms} rooms` : ""} · ${twist ? withArticle(twist) : `$${snap?.stake?.loot ?? snap?.loot ?? 0}`}`;
    if (text !== cardText) { cardText = text; replace(card, text); }
  }
  function holdMap(): void { if (!cardUp) return; mapHold = true; cardWait = false; paintCard("map"); applySpeed(); }
  function near(t: number): void { slowUntil = Math.max(slowUntil, t + AUTO_TAIL); }
  /** Cut 12 §6: after a skip lands on tick t, the near / scene hold from the span skipped over is let go (the landing decides). */
  function letGo(t: number): void { slowUntil = Math.min(slowUntil, t); sceneUntil = Math.min(sceneUntil, t); }
  /** Cut 12 §6: a skip that found the run's end lands the viewer at the ending (its last ENDING_TICKS still play at 1×) —
   *  a fight that held the exit used to leave the press with nothing visible until the clock got there on its own. */
  function toEnding(): void { const t = Math.max(viewerTick(), endingFrom); if (t > viewerTick()) { release(t); seekTo(t); } applyFrame(); applySpeed(); }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    engineTick = s.turn;
    for (const e of s.entities) note_(e);
    sceneFrom(s, r.events);
    if (s.hero.hp < lastHp) near(s.turn);   // hp lost by any means; a rest's +1 per turn is a dead stretch, a drink is a `use` event
    lastHp = s.hero.hp;
    if (s.alert > lastAlert) { const n = s.alert; at(s.turn, () => ambient(/* copy:callout */ `alert ${n}`)); }   // Cut 7 §4
    lastAlert = s.alert;
    const exit = absorb(r.events, s);
    snap = s;
    hud.maxHp = s.hero.max_hp; paintHud(); paintStake(s); bossSighted(s);
    // the card follows the step at once: under ▶▶|'s step loop and the travel chain the engine's answers starve the pump's
    // timer, and the card (painted only there) named the floor before the one the HUD's load had just painted (QA on
    // e0f87e7: "`D1 · 16 rooms · $18` while the HUD reads `32/40 D2`")
    if (cardUp) paintCard("map");
    if (!exit) vaultFrom(s);   // no choice on a run that just ended
    if (r.exit_pending) pendingExit = r.exit_pending;
    // Cut 7 §4: the exit batch waits (pump) until the viewer is ENDING_TICKS from the exit, then plays at 1×
    if (exit) { held = { evs: r.events, tier: exit, snap: s }; el.dataset.ending = "1"; if (vaultClose) { vaultClose(); vaultClose = null; } return; }
    feed(r.events, s);
    if (performance.now() - lastPersist > PERSIST_MS) { lastPersist = performance.now(); app.persist(); }
  }
  function feed(evs: Ev[], s: Snapshot): void {
    // entities that appear inside this batch must exist before their events apply (they are not tweened in;
    // the first event they own places them)
    (viewer as Viewer & { preload?: (x: Snapshot) => void } | null)?.preload?.(s);
    const di = evs.findIndex((e) => e.k === "descend");
    if (viewer && di >= 0) { viewer.apply(evs.slice(0, di + 1)); pendingLoad = { snap: s, rest: evs.slice(di + 1) }; }
    else { viewer?.apply(evs); if (viewer?.sync) { const v = viewer; at(s.turn, () => v.sync!(s)); } } // Cut 4 §3: remembered foes
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
    if (vaultClose && !document.querySelector(".vault-choice")) vaultClose = null;   // dismissed by backdrop / Escape: the engine's grace decides
    applyFrame();
    applySpeed();
    let now = viewerTick();
    el.dataset.tick = String(now);            // dev: tools sample the cadence off the DOM
    release(cardWait ? Math.min(now, fightFrom) : now);   // Cut 10 §1: a fight waiting on the card keeps its HUD at the first frame
    if (pendingLoad) {
      // Cut 10 §1: under the card the floor changes at once (nothing is watched); otherwise the viewer drains first
      if (cardUp || viewerIdle()) { const p = pendingLoad; pendingLoad = null; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
      return;
    }
    if (held) {
      // Cut 7 §4: the clock runs on (8× through dead air) to the ending, then the exit batch plays and the exit flow waits for it
      // Cut 10 §1: under the card the viewer jumps to the ending (the walk-out plays at 1×; the card hides there)
      if (cardUp && now < endingFrom) { release(endingFrom); seekTo(endingFrom); now = endingFrom; applyFrame(); applySpeed(); }
      if (now < endingFrom) return;
      const hb = held; held = null; feed(hb.evs, hb.snap);
      exitTier = hb.tier; exitAt = performance.now() + EXIT_GRACE_MS;
      endControls();
      return;
    }
    if (exitTier) {
      if (!(viewerIdle() || performance.now() > exitAt)) return;
      release(Infinity);
      // Cut 2 §1: `rest 12m` for a beat, then the exit flow continues
      if (restS !== undefined && !restUntil) { restUntil = performance.now() + REST_BEAT_MS; showBanner(/* copy:callout */ `rest ${spanOf(restS)}`, REST_BEAT_MS); return; }
      if (performance.now() < restUntil) return;
      void finish(exitTier); return;
    }
    if (inflight) return;
    // Cut 10 §1: travel under the card — the engine steps flat out (chained calls) until it finds the next fight or the exit
    if (travelling()) { inflight = true; travel(); return; }
    // Cut 10 §1: through a shown fight's tail the engine waits at its close, so the next fight opens under the card (and is costed
    // there) instead of merging into this one at 1×
    if (mode === "fights" && !mapHold && !vaultClose && !fightOn && Number.isFinite(fightUntil) && now < fightUntil && !(beat && now < beat.until)) return;   // Cut 13 §4: a beat plays on at 1×
    const lead = speed >= AUTO_FAST ? LEAD_FAST : LEAD;
    if (speed <= 0 || engineTick - now >= lead) return;
    inflight = true;
    app.engine.step(speed >= AUTO_FAST ? BATCH_FAST : BATCH).then((r) => { inflight = false; if (!disposed && !done) handle(r); if (skipQueued) { skipQueued = false; void skipToEvent(); } })
      .catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; endControls(); });
  }
  /** Cut 10 §1: is the engine free to run ahead under the card — `fights`, the card up and not held, no fight found yet, no exit. */
  function travelling(): boolean {
    return mode === "fights" && cardUp && !cardWait && !mapHold && !held && !exitTier && !done && !disposed && !fightAhead(viewerTick());
  }
  function travel(): void {
    app.engine.step(BATCH_FIGHTS).then((r) => {
      if (disposed || done) { inflight = false; return; }
      handle(r);
      probeFight(r);
      if (pendingLoad && viewer) { const p = pendingLoad; pendingLoad = null; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
      if (travelling()) travel(); else inflight = false;
    }).catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; endControls(); });
  }
  /** Cut 10 §1: the fight the engine is running through under the card, costed as it goes; at its close (or the run's end
   *  inside it) the fight is either shown — the viewer cuts to its first frame — or dropped (the frame span is cleared). */
  function probeFight(r: StepResult): void {
    const s = r.snapshot, heroId = s.hero.id;
    if (fightOn || held) {
      probe ??= { hurt: 0, low: false, boss: false, ally: false, steal: false };
      for (const ev of r.events) {
        if (ev.k === "hurt" && ev.id === heroId) probe.hurt += ev.dmg;
        else if (ev.k === "die" && allies.has(ev.id)) probe.ally = true;
        else if (ev.k === "steal") probe.steal = true;
      }
      if (s.hero.max_hp > 0 && s.hero.hp / s.hero.max_hp < SHOW_HP) probe.low = true;
      if (s.entities.some((e) => e.tags.includes("boss") && hostile(e) && !e.remembered && s.visible[e.y * s.w + e.x])) probe.boss = true;
      if (!held) return;
    }
    if (!probe) return;
    const show = held || probe.hurt >= SHOW_HURT || probe.low || probe.boss || probe.ally || probe.steal;
    probe = null;
    if (show && fightFrom < Infinity) fightShow = true;
    else if (beat && viewerTick() < beat.until && fightFrom <= beat.from) fightUntil = beat.until;   // Cut 13 §4: the beat keeps its frame
    else { fightFrom = Infinity; fightUntil = -Infinity; }
  }
  /** The rate the clock should run at right now. `fights`: 16× under the card (8× when a tap holds the map, 0 while a fight
   *  waits for the card's minimum); `fast`: 8× / 1× by what is near (flat 8× while bailing). Cut 7 §4: 1× through a scene and
   *  through the run's last ENDING_TICKS (bailing too: the walk-out is still the end). */
  function rate(): number {
    if (paused) return 0;
    if (vaultClose) return 1;                 // Cut 5 §4: the vault sheet holds the clock at 1× while the engine's grace runs
    if (frame === "fight") return mode === "fast" ? FAST_NEAR : 1;   // Cut 8A: a fight is watched at 1× (Cut 12 §6: `fast` at 2×)
    const v = viewerTick();
    if (v >= endingFrom) return 1;
    if (mode === "fights") return cardWait || cardUp ? 0 : mapHold ? AUTO_FAST : RATE.fights;   // the clock holds under the card: the cut seeks
    return overridden || (v >= slowUntil && v >= sceneUntil) ? RATE.fast : FAST_NEAR;
  }
  function applySpeed(): void {
    const n = rate();
    if (n === speed) return;
    if (!viewer?.tick) viewerTick();          // placeholder clock: bank the ticks run at the old rate first
    speed = n; viewer?.setSpeed(n);
    el.dataset.speed = String(n);
    for (const m of Object.keys(modeBtn) as Mode[]) modeBtn[m].classList.toggle("slowed", m === mode && n <= FAST_NEAR);
  }
  function setMode(m: Mode): void {
    if (app.watchMode !== m) { app.watchMode = m; app.persist(); }   // remembered for the next run
    mode = m; paused = false; mapHold = false; el.dataset.mode = m;
    for (const k of Object.keys(modeBtn) as Mode[]) modeBtn[k].classList.toggle("on", k === m);
    paintPause(); paintCard(frame); applySpeed();
  }
  function togglePause(): void { paused = !paused; paintPause(); applySpeed(); }
  function paintPause(): void { pause.classList.toggle("on", paused); replace(pause, paused ? "▶" : "⏸"); }
  let skipQueued = false;
  async function skipToEvent(): Promise<void> {
    if (done || !viewer || exitTier) return;
    // a floor change waits for the viewer to drain; a skip drains it now instead of at 1×
    if (pendingLoad) {
      // drain the old floor, apply the new one, and drop its queued events straight into place
      const p = pendingLoad; pendingLoad = null;
      const fv = viewer as Viewer & { seek?: (t: number) => void };
      viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest);
      if (fv.seek) fv.seek(engineTick);
      fbTick = engineTick; release(viewerTick());
      return;
    }
    // Cut 10 §1: under the card the engine is already running to the next fight (or has found it): the press waives the card's minimum
    if (mode === "fights" && cardUp && !mapHold) { cardSince = -Infinity; if (inflight) return; applyFrame(); applySpeed(); return; }
    // a press while a step is in flight is not lost: one skip is queued behind it
    if (inflight) { skipQueued = true; return; }
    if (held) { endingFrom = 0; return; } // skip overrides the ending hold: the pump releases the exit batch now
    inflight = true;
    // Cut 10 §1: inside a shown fight the press means its end — the span's close is known (the engine ran ahead) or is stepped to
    if (frame === "fight") {
      try {
        for (let i = 0; i < 120 && fightOn && !Number.isFinite(fightUntil) && !disposed; i++) {
          const r = await app.engine.step(BATCH); handle(r);
          if (held) break;
          const p = takeLoad(); if (p) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
        }
      } catch (e) { console.warn("skip failed", e); }
      inflight = false;
      if (held) toEnding();
      else { const t = Number.isFinite(fightUntil) ? fightUntil : Math.max(viewerTick(), engineTick); seekTo(t); release(t); letGo(t); applyFrame(); applySpeed(); }
      if (skipQueued) { skipQueued = false; void skipToEvent(); }
      return;
    }
    // Cut 10 §1: a press outside a fight means "the next fight, now": the engine steps until its snapshot opens the fight frame
    // (`fightOn`, the mode's own predicate, judged per batch in `handle`), floors drained on the way, and the viewer lands on that
    // first frame. Cut 12 §6: `fast` too — its old target, "the next interesting event" (a pickup, a fact, a spawn), sat inside the
    // engine's ≤ 32-tick lead on a busy floor, so the seek landed where the viewer already was and the press read as inert.
    let landed = false;
    try {
      // a fight the engine opened ahead of the viewer's clock is the one to land on (no stepping); one the viewer is inside is
      // stepped to its close, then the next
      const ahead = fightOn && fightFrom > viewerTick();
      const inFight = fightOn && !ahead;
      let hit = ahead;
      const max = inFight ? 120 : SKIP_FIGHT_BATCHES;
      for (let i = 0; i < max && !hit && !disposed; i++) {
        const r = await app.engine.step(BATCH);
        handle(r);
        hit = r.run_over || (inFight ? !fightOn : fightOn);
        if (held) break;
        // a floor change on the way: load it now (the skip is the drain), the queued events straight into place
        const p = takeLoad(); if (p) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
      }
      landed = fightOn && fightFrom > viewerTick();
    } catch (e) { console.warn("skip failed", e); }
    inflight = false;
    if (held) toEnding();
    else if (!pendingLoad) {
      // the viewer lands on the found fight's first frame (`fights`: the card's minimum waived, the press asked for it), else where
      // the engine stopped — replaying the skipped span at 1× is what made ▶▶| feel dead in a fight
      const t = landed ? fightFrom : Math.max(viewerTick(), engineTick - BATCH);
      seekTo(t); release(viewerTick()); letGo(t);
      if (landed) { cardSince = -Infinity; applyFrame(); applySpeed(); }
    }
    if (skipQueued) { skipQueued = false; void skipToEvent(); }
  }
  // Cut 5 §5: `return` fires on the next hero action as a chore (`engine.bail()`, the rules untouched); the run plays out to
  // the exit at 8× under `returning`, then the exit sheet. An engine without `bail` gets the row prepend, restored at the exit.
  function doBail(): void {
    if (done || overridden) return;
    overridden = true; bail.classList.add("on"); bail.disabled = true; if (snap) paintStake(snap);
    callout(/* copy:callout */ "returning", "", 1800);
    app.engine.bail().catch((e) => {
      console.warn("bail", e); prepended = true;
      void app.engine.setRules({ rows: [{ conds: [], verb: { v: "return" } }, ...app.rules.rows] }).catch((e2) => console.warn("bail", e2));
    });
    setMode(mode);
  }
  // Cut 5 §4: the vault choice sheet — opens once per cage, closes when the engine's snapshot no longer carries it
  function vaultFrom(s: Snapshot): void {
    const vc = s.vault_choice;
    // a choice on a run that has ended is no choice (a `choose` after the exit trips the core: seed 12's death on a vault tile)
    if (!vc || !vc.items.length || held || exitTier || done) { if (vaultClose) { vaultClose(); vaultClose = null; } return; }
    const key = vc.items.map((it) => it.id).join(",");
    if (vaultClose || key === vaultKey) return;
    vaultKey = key;
    // Cut 10 §1: found under the card, the viewer joins the engine at the vault (the grace runs at 1× from here)
    if (cardUp) { release(s.turn); seekTo(s.turn); }
    vaultSheet(vc);
  }
  function vaultSheet(vc: VaultChoice): void {
    let sent = false;
    openSheet((close) => {
      vaultClose = () => { vaultClose = null; close(); paintCard(frame); applySpeed(); };
      const chips = h("div", { class: "chips" }, ...vc.items.map((it) => h("button", { class: "chip item", onclick: () => {
        if (sent) return; sent = true;
        app.engine.choose(it.id).catch((e) => console.warn("choose", e)).finally(() => vaultClose?.());   // the next step's snapshot carries the pickup
      } }, h("b", { class: "glyph" }, kindGlyph(it.kind)), " ", it.label)));
      return h("div", { class: "sheet-body vault-choice" }, h("div", { class: "label row-label" }, /* copy:label */ "vault"), chips);
    });
    paintCard(frame); applySpeed();
  }
  function xpGained(): number {
    const c = app.lineage.classes?.[cls] ?? { level: 1, xp: 0 }; let g = c.xp - before.xp;
    for (let l = before.level; l < c.level; l++) g += xpToNext(l);
    return Math.max(0, g);
  }
  /** An engine call that hangs must never strand the player on the black exit screen. */
  function bounded<T>(p: Promise<T>, ms: number, what: string): Promise<T | undefined> {
    return new Promise((res) => {
      const t = window.setTimeout(() => { console.warn(`${what}: no answer in ${ms} ms`); res(undefined); }, ms);
      p.then((v) => { clearTimeout(t); res(v); }, (e) => { clearTimeout(t); console.warn(what, e); res(undefined); });
    });
  }
  /** The run is over (the exit batch is on the viewer): the mode buttons, ▶▶| and bail have nothing left to act on and go
   *  disabled; ⏸ goes (unpaused first, so the walk-out drains) and leaves the top-right to the `verdict` busy label
   *  (QA on 50bb162: "fights · fast · ▶▶| · bail still live on a dead hero; VERDICT sits over the pause button"). */
  function endControls(): void {
    if (el.dataset.over === "1") return;
    el.dataset.over = "1";
    if (paused) { paused = false; paintPause(); applySpeed(); }
    pause.hidden = true;
    for (const b of [modeBtn.fights, modeBtn.fast, skip, bail]) b.disabled = true;
  }
  async function finish(tier: Tier): Promise<void> {
    if (done) return;
    done = true; clearInterval(pumpTimer); card.hidden = true; cardUp = false; el.dataset.card = "0";
    endControls();
    // whatever happens below, the player reaches a screen with buttons
    const guard = window.setTimeout(() => { if (!disposed && app.view.kind === "watch") { console.warn("exit flow stalled; falling back to camp"); app.go({ kind: "camp" }); } }, 20_000);
    try {
      if (prepended) await bounded(app.engine.setRules(app.rules), 8000, /* copy:none */ "setRules after bail");
      // The keep sheet is a decision only when there is a free vault slot; otherwise the engine keeps by preference.
      const freeSlots = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
      if (pendingExit && pendingExit.items.length && freeSlots > 0) { const p = pendingExit; pendingExit = undefined; clearTimeout(guard); exitSheet(p, () => { done = false; void finish(tier); }); return; }
      // the sheet skipped (the vault full): the engine keeps by preference and the rest is salvage all the same — its rows
      // are built here too (QA on e0f87e7: "no SALVAGED block at all when the vault is full … yet the gold sheet shows +$16
      // salvage"); what the vault gained across the keep is what was kept, matched to the pending items by kind
      const skipped = pendingExit; pendingExit = undefined;
      const vaultBefore = new Set(app.lineage.vault.map((v) => v.id));
      if (skipped) await bounded(app.engine.keep([]), 8000, "keep by preference");
      // (`refresh` resolves void, so a sentinel tells a timeout from success)
      if (!(await bounded(app.refresh().then(() => true), 8000, "refresh at exit")) && !disposed) { clearTimeout(guard); app.go({ kind: "camp" }); return; }
      if (skipped) {
        const kept = new Set<number>();
        const gained = app.lineage.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.kind);
        for (const kind of gained) { const it = skipped.items.find((x) => x.kind === kind && !kept.has(x.id)); if (it) kept.add(it.id); }
        salvagedRows = letGoRows(skipped, kept);
      }
    } finally { /* guard cleared on every normal path below */ }
    await finishAfterRefresh(tier, guard);
  }
  /** What an exit let go, per kind at the engine's worth at this exit (`salvageValue` when the wire has none). */
  function letGoRows(p: { items: InvItem[]; tier: string; worth?: number[] }, kept: Set<number>): { kind: string; n: number; gold: number }[] {
    const rows = new Map<string, { kind: string; n: number; gold: number }>();
    p.items.forEach((it, i) => { if (kept.has(it.id)) return; const r = rows.get(it.kind) ?? { kind: it.kind, n: 0, gold: 0 }; r.n++; r.gold += p.worth?.[i] ?? salvageValue(it.kind, p.tier); rows.set(it.kind, r); });
    return [...rows.values()].filter((r) => r.gold > 0);
  }
  async function finishAfterRefresh(tier: Tier, guard: number): Promise<void> {
    clearTimeout(guard);
    app.runsSeen += 1;
    if (disposed) return;
    tamed.push(...tamedIds.map(compLabel));
    lost.push(...lostIds.map(compLabel));
    // Cut 13 §1: a run that came home stalled (`… · stalled` on its line; the stake was `stalling` at the exit) gets a verdict screen
    // like a death's — the core records the stall, `death(runId)` answers `verdict: "stall"`; an older core falls back to the report
    const stalled = tier === "return" && (/\bstalled\b/.test(exitLine?.text ?? "") || (!!snap?.stake?.stalling && (exitLine?.kept ?? 1) === 0));
    if (tier === "death") for (const c of partyAtStart) if (!lost.some((l) => l === c || l.endsWith(c.slice(c.indexOf(" · "))))) lost.push(c);
    if (tier === "death" || stalled) {
      try {
        const death = await app.busy(/* copy:label */ "verdict", () => app.engine.death(runId));
        if (stalled && death.verdict !== "stall") throw new Error(`no stall verdict (${death.verdict})`);
        death.line ??= exitLine;   // Cut 6 §1: the verdict may lack the line; the exit event carried it
        if (!disposed) app.go({ kind: "death", death, lost });
        return;
      } catch (e) { console.warn(stalled ? "no stall record; the report shows the run" : "no death record; the report counts the death", e); }   // Cut 10 §3: never `1 runs · 0 deaths` after a death
    }
    const L = app.lineage;
    const bests: string[] = []; for (let d = before.best + 1; d <= L.best_depth; d++) bests.push(`D${d}`);
    // a find labelled at pickup by its flavour (`red potion?`) reads by its kind once the same send learned it
    // (`item:red=confusion` in `learned`; QA on 952e306: "LEARNED confusion red, FOUND still red potion?")
    const idents = new Map(learned.map((f) => /^item:([a-z_]+)=([a-z_]+)$/.exec(f)).filter((m): m is RegExpExecArray => !!m).map((m) => [m[1], m[2]]));
    for (const it of found) {
      const m = /^([a-z_]+) (potion|scroll)\?$/.exec(it.label);
      const kind = m && idents.get(m[1]);
      if (kind) { it.label = `${kind.replace(/_/g, " ")} ${m![2]}`; it.kind = kind; }
    }
    const report: ReturnReport = {
      elapsed_s: Math.round((engineTick - startTick) / 10), runs: 1, sampled: false, learned, bests, found, pending: [],
      deaths: tier === "death" ? [{ cause: heroCause ?? exitLine?.text ?? /* copy:label */ "death", n: 1 }] : [],   // Cut 10 §3: the death it came from
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap!, tamed, hatched: [], lost,
      xp: { class: cls, gained: xpGained(), level_ups: (L.classes?.[cls]?.level ?? 1) - before.level },
      salvaged: reconcileSalvage(mergeSalvage(exitLine?.salvaged ?? [], salvagedRows), L.gold_ledger ?? []), deepest, renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
      spent: spentRows(L.gold_ledger ?? []),   // Cut 13 §3: what the automations bought at this exit (`heal ×1 · −$40`)
      banked: tier === "bank" ? 1 : 0, returned: tier === "return" ? 1 : 0, bones_found: bonesFound,   // rest is still ahead: the camp shows it
      exits: exitLine ? [{ ...exitLine, trace: exitLine.trace ?? exitTrace }] : undefined,            // Cut 6 §1; Cut 9 §5: with its trace
    };
    app.go({ kind: "report", report });
  }

  /** The exit's own cut (on the line, from the engine) plus what the keep sheet let go, per kind. */
  function mergeSalvage(a: { kind: string; n: number; gold: number }[], b: { kind: string; n: number; gold: number }[]): { kind: string; n: number; gold: number }[] {
    const m = new Map<string, { kind: string; n: number; gold: number }>();
    for (const r of [...a, ...b]) { const x = m.get(r.kind) ?? { kind: r.kind, n: 0, gold: 0 }; x.n += r.n; x.gold += r.gold; m.set(r.kind, x); }
    return [...m.values()].filter((r) => r.gold > 0);
  }
  /** The rows' gold made to sum to the ledger's salvage for this exit — the `salvage` lines after the newest exit line (the
   *  gold sheet's own slicing, ui/gold.ts) — by moving the difference onto the largest row(s); without a salvage line the rows
   *  stand. The report's SALVAGED then reconciles with the gold sheet by construction (QA on e0f87e7: "SALVAGED $12 vs +$17
   *  salvage"; "$5 vs +$8": items the sheet never listed, and worths the client's table read differently). */
  function reconcileSalvage(rows: { kind: string; n: number; gold: number }[], ledger: { t: number; delta: number; why: string }[]): { kind: string; n: number; gold: number }[] {
    let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled)\b/.test(ledger[i].why)) i--;
    const lines = ledger.slice(i + 1).filter((g) => /^salvage/.test(g.why));
    if (!lines.length || !rows.length) return rows;
    let diff = lines.reduce((a, g) => a + g.delta, 0) - rows.reduce((a, r) => a + r.gold, 0);
    const out = rows.map((r) => ({ ...r })).sort((a, b) => b.gold - a.gold);
    for (const r of out) { if (!diff) break; const take = Math.max(-r.gold, diff); r.gold += take; diff -= take; }
    return out.filter((r) => r.gold > 0);
  }
  /** Cut 13 §3: the automations' purchases at this exit — the ledger's outgoings after the newest exit line that are not salvage
   *  (`−$40 heal potion` → `heal potion ×1 · −$40`), per line text. Empty when nothing was bought. */
  function spentRows(ledger: { t: number; delta: number; why: string }[]): { kind: string; n: number; gold: number }[] {
    let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled)\b/.test(ledger[i].why)) i--;
    if (i < 0) return [];
    const rows = new Map<string, { kind: string; n: number; gold: number }>();
    for (const g of ledger.slice(i + 1)) {
      if (g.delta >= 0 || /^(salvage|wake pay|insure)/.test(g.why)) continue;
      const kind = g.why.replace(/^(bought|restock)\s+/, "");
      const r = rows.get(kind) ?? { kind, n: 0, gold: 0 }; r.n++; r.gold += -g.delta; rows.set(kind, r);
    }
    return [...rows.values()];
  }
  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: InvItem[]; tier: string; worth?: number[] }, then: () => void): void {
    const free = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
    const keep = new Set<number>();
    let sent = false;
    openSheet((close) => {
      const chips = h("div", { class: "chips" });
      const count = h("span", { class: "num dim" });
      const paint = (): void => {
        replace(count, `${keep.size}/${free}`);
        replace(chips, ...p.items.map((it, i) => h("button", { class: `chip item${keep.has(it.id) ? " on" : ""}`, onclick: () => {
          // the picks full: the next chip swaps in for the oldest pick (a Set keeps insertion order) — QA on e0f87e7:
          // "VAULT 1/1 after picking one item: tapping a second chip does nothing"
          if (keep.has(it.id)) keep.delete(it.id);
          else { if (keep.size >= free) { const oldest = keep.values().next().value; if (oldest === undefined) return; keep.delete(oldest); } keep.add(it.id); }
          paint();
        } }, it.label, " ", keep.has(it.id) ? h("b", null, "⌂") : h("b", { class: "num gold" }, `$${p.worth?.[i] ?? salvageValue(it.kind, p.tier)}`))));   // the engine's worth at this exit (its old client table read 4×)
      };
      paint();
      const bones = p.tier === "death" && bonesLeft !== undefined ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(bonesLeft)}`) : null;
      const ledger = exitLine?.text ? h("div", { class: "ledger-line num dim" }, exitLine.text) : null;   // Cut 6 §1: engine data, verbatim
      const trace = traceChip(exitTrace ?? exitLine?.trace, "chip mini", { rows: app.rules.rows, runId });   // Cut 9 §5: the trace on a chip; Cut 11 §3: with its chain
      // the sheet counts picks against free slots, so its label is `keep 0/1`, not the camp's `vault 1/2` (QA on e0f87e7:
      // "VAULT 0/1 while camp shows VAULT 1/2 · same counter")
      return h("div", { class: "sheet-body" },
        h("div", { class: "label row-label" }, /* copy:label */ "keep", " ", count, trace),
        chips, bones, ledger,
        h("button", { class: "btn primary wide", onclick: () => {
          if (sent) return; sent = true;
          salvagedRows = letGoRows(p, keep);
          app.engine.keep([...keep]).then((L) => { app.lineage = L; }).catch((e) => console.warn("keep", e)).finally(() => { close(); then(); });
        } }, /* copy:button */ "keep"));
    });
  }

  async function init(): Promise<void> {
    let s: Snapshot;
    try { s = await app.engine.send(); } catch (e) { console.warn("send failed", e); if (!disposed) app.go({ kind: "camp" }); return; }
    if (disposed) return;
    snap = s; runId = s.run.id; engineTick = startTick = s.turn;
    for (const e of s.entities) note_(e);
    hudFrom(s);
    const { viewer: v0 } = await makeViewer(canvas);
    if (disposed) { v0.dispose(); return; }
    // Cut 11 §2: every floor load and event batch is kept in the run log, so the death screen's chain can scrub a replay
    const v = recordRun(v0, runId, s.run.started_turn);
    viewer = v; v.resize?.(); v.load(s); el.dataset.frame = frame; fbTick = s.turn; fbAt = performance.now();
    speed = -1; applyFrame(); applySpeed();   // Cut 10 §1: the card and the mode's rate (fights: 16× under it) from the first frame
    if ("__riddle" in window) (window as unknown as { __viewer: Viewer }).__viewer = v;   // dev inspection
    lastHp = s.hero.hp; lastAlert = s.alert; sceneFrom(s);
    pumpTimer = window.setInterval(pump, PUMP_MS);
  }
  void init();
  const onResize = (): void => viewer?.resize?.();
  window.addEventListener("resize", onResize);
  return { el, dispose: () => {
    disposed = true; window.removeEventListener("resize", onResize); clearInterval(pumpTimer); clearTimeout(tickerTimer); clearTimeout(bannerTimer); viewer?.dispose();
    if (vaultClose) { const c = vaultClose; vaultClose = null; c(); }
    if (prepended && !done) void app.engine.setRules(app.rules);
  } };
}
