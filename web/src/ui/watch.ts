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
// Cut 12 §6 — `fast` is faster: travel at 16×, a fight at 2× (`fights` kept its fights at 1×; Cut 14: 4× and 2×). `▶▶|` in `fast` is the run's
// END: the engine steps to `run_over` and the ending plays as it does after any skip (the last ENDING_TICKS at 1×, then the
// exit flow) — it used to reach the next fight, as in `fights`, and five players read it as "plays faster" (QA on 50bb162:
// ">2 min to the death screen"); in `fights` the press stays "the next fight or the end" (Cut 10 §1). §4: the card names the floor's
// situation (`D4 · 9 rooms · a nest`, `Snapshot.floor_twist`). A sanity refusal (`drink ✗ no use`) shows once per floor; a
// summoned ally's fall reads `ally hound fell`; the boss's rally names the boss (`warlord rallies`, from its telegraph).
//
// Cut 14 (QA) — the dynamic slowdowns were "way too slow": the fight frame plays at 2× in `fights` (was 1×) and 4× in `fast`
// (`FIGHT_RATE`; was 2×), `near` holds 10 ticks after the last sighting / hp change (AUTO_TAIL, was 20), the ending's 1× is the
// last 20 ticks (ENDING_TICKS, was 30; the core's `ending` marker is clamped to twice that). A settings toggle `slowdowns`
// (`app.slowdowns`, persisted `riddle.slowdowns`, on by default) turns every hold off: `rate()` then runs the mode's flat rate
// — the card's 0, the vault sheet's 1×, pause and the ending's 1× stand; the fight frame still opens, only the clock changes.
//
// Cut 14 §6 — the world runs on the wall clock. Two clocks: the *world* clock (`worldT`, engine ticks) advances on wall time at
// `worldRate()` — the same decisions as the viewer's `rate()` (fight 2×/4×, near, scene, flat 16×) read off the ENGINE's tick —
// and never stops while a run is live; the pump steps the engine to `max(worldT, viewer + LEAD)` (LEAD only while the picture
// plays), in catch-up batches of up to CATCHUP_MAX when it is behind. `⏸` freezes the picture only (the viewer's clock); the gap
// grows and the event queue is the DVR buffer; resume replays from the playhead; `▶▶|` from behind lands on the frontier (live).
// A hidden tab (`visibilitychange`) freezes the picture too; on return the world is caught up (the wall clock ran, `worldT`
// grew) and the viewer seeks to live — to the ending's start when the run ended meanwhile, so the walk-out is never skipped.
// Floors the viewer has not reached queue (`loads`); the run's end is a world event: `held` keeps only the last ENDING_TICKS and
// the exit flow waits for the replay to get there. The scrub strip (`.scrub`) under the canvas: the playhead at the viewer's
// share of the run, a dot at the frontier that beats once per engine batch; `data-frontier` · `data-world` · `data-pulses` for
// tooling. `slowdowns: off` flattens the world rate too. Fight spans the viewer has yet to reach are kept (`spans`) so a paused
// or behind viewer still cuts to each fight in `fast`. (QA on 56f2a1d) A frozen picture is frozen whole: no floor load, no
// release, no frame cut, no travel under the card, the ticker's timer stopped; the batches wait (`frozenFeed`) for the thaw. The
// card names the HUD's floor and the stake's loot (`paintCardText`), never the engine's latest snapshot.
//
// Cut 14 §3 — a bank or a return is a beat: the exit event opens the fight frame on the stairs for SCENE_MS with `COLLECTED $N` /
// `RETURNED $N` as its callout (`beatAt`, the exit flow waits `exitBeatUntil`), in `fights` and `fast` alike. §4: a repeated chore
// callout coalesces on its line (`pick up ×8`, `choreCallout`); the `rest 20m` banner sits low (`.banner.rest`) so it never covers
// the death frame's callout; the run's row-fire counts go to `app.rowFires` for the death screen's drop sheet (Cut 15 §3).
//
// Cut 15 §4 — the watch's peaks are on screen. A boss's kill is a beat like the bank's: the fight frame holds on the kill (SCENE_TICKS at
// 1×, SCENE_MS) with `WARLORD DOWN` (the boss's name, ≤ 2 words, + DOWN) as its callout, in `fights` and `fast`. The floor card is
// short: its minimum is CARD_MS (1 s; the gate is 1.2 s) on a new floor, CARD_BEAT_MS between fights and before any beat, and a card that has
// been up CARD_MAX_MS with no fight found gives way to the live map at the mode's flat rate (`cardLive`, until the next fight). The
// lit mode chip carries its clock as small digits (`data-rate` → `fast 16`, `fights 2` in a fight; CSS `::after`, so the chip's
// text stays its word). In `fights` a chore stretch inside the fight frame (no foe near, no beat, no near hold at the playhead:
// the pick-up after a kill) runs at the flat rate; only a fight, a beat or a near hold slows the clock.
//
// Cut 16 §3 — the first floor of a biome in the run names it on the card: `D5 · the Burrows · a shrine` (the title in the rooms'
// place). §4 — while a boss is in view (the snapshot's visible entities tagged `boss`), a thin bar under the hero's: its name,
// one word (`warlord`), and its hp (the snapshot's, then each `hurt` on it at the viewer's clock); gone when he dies or leaves
// view. `warlord breaks` (the core's callout and note, once at half hp) is a beat like the kill: the fight frame holds
// SCENE_MS on `WARLORD BREAKS`.
//
// Cut 20 §3 — the watch moves. `fast` takes ≤ 40 % of `fights`'s wall time over a run: its dead stretches start at 32× (no 16×
// ramp-in), its fights stay at 4×, a situation's beat holds 1 s (BEAT_HOLD_FAST_MS; a boss's kill or break, the bank and the cage keep
// theirs), the walk-out plays at 2×, and near a cage the short batches chain per pump pass. `fights` on D1–D3 shows no card: the
// travel at 8× (EARLY_TRAVEL), every fight at 1.5× (EARLY_FIGHT) — early runs were card-skipped travel "too short to follow".
// §4: the stake names the exit that keeps and what a death keeps (`carry $78 · bank keeps $78 · death $0 · bank at D9`,
// `Stake.death_keep`); `keeps` never stands alone.
//
// Pacing (Addendum E): the viewer owns the clock (10 ticks/s × speed). The engine worker is pumped in
// 10-tick batches whenever it is fewer than LEAD ticks ahead of the viewer, so events always arrive
// before the viewer needs them and the engine never runs far ahead (≤ 22 ticks, under the viewer's
// dead-air threshold). HUD hp/depth and the ticker are queued by tick and released at the viewer's
// clock, so what the numbers say matches what the sprites do.
//
// Cut 5 §5 — auto cadence (the default): the clock runs at 8× through dead stretches and drops to 1× while
// anything is near (Cut 12 §6: 2×; Cut 14: 4×, and the fight frame's own 2× / 4× — see the block above). "Near" is read off the engine, which is ≤ 22 ticks ahead of the viewer: a hostile in view
// or an item within 3 tiles in a step's snapshot, a telegraph / attack / hero hp change in its events. Each
// sighting holds until AUTO_TAIL ticks after it (so hp unchanged for 10 ticks is the fast condition), and the
// viewer slows *before* the foe walks into frame because the engine saw it first. At 8× the pump runs the same
// LEAD with a 50 ms interval and a larger batch, so the engine still never runs dry or far ahead. Bail (§5)
// turns the auto rate into a flat 8× and the stake line reads `returning` until the exit sheet.
//
// Cut 5 §4 — the vault choice: a step whose snapshot carries `vault_choice` opens a sheet with the three items
// as chips and holds the clock at 1×; a tap is `choose(id)`. The engine's 50-tick grace runs watched or not, so
// an unanswered sheet closes on its own when `vault_choice` leaves the snapshot (the lineage's `vault_pref` took).
// Cut 15 §5 — the cage waits for the player who is watching: while the sheet is open the pump does not step the engine (the world
// clock stands, the picture plays up to the frontier and holds) for up to VAULT_WAIT_MS of wall time; then the sheet closes and the
// engine's grace and preference proceed as before. The sheet's bar shrinks over that wall-clock wait. Offline and in sims nothing
// changes (the choice is an input, as `choose` always was).
// Cut 19 §1 — the cage is a beat, not a sheet: a snapshot carrying `vault_choice` cuts in the beat `took mail` (the preference's
// pick, `VaultChoice.pick`) held like a situation's; the world waits while it holds (CAGE_WAIT_CAP_MS at most before its line shows);
// a tap on the line within its hold opens the three as the override sheet (the world waits while it is open, ≤ VAULT_WAIT_MS).
// Untouched, the hold ends and the engine's grace and the preference take the pick — no sheet the player can miss.
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
import { namedCounters } from "./counter-name";
import { enemyHost } from "./enemy-tips";
import { unitLabel } from "./unit-icon";
import "../legible.css";
import { compactLine, meterPanel } from "./meters";
import { combatLogEvents, gunShotText, telegraphText } from "./combat-log";
import { tacticObserver } from "./tactic-observer";
import type { App, Mounted } from "../app";
import { gunStatus } from "./gun-status";
import type { EncounterModifiers, GunSnap, Ev, ExitLine, FoldLine, Highlight, InvItem, ReturnReport, Snapshot, SnapMeters, StepResult, Trace, VaultChoice } from "../engine/types";
import { h, clear, items, replace, spanOf } from "./dom";
import { flyCoins, gem, paintPortrait, paintSprite, portrait, renderBar, renderConsole, tile, wideCols } from "./frame";
import { icon } from "./skin";
import { syncLook } from "./look";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt } from "../engine/classes";
import { openSheet, openWindow } from "./sheet";
import { salvageValue } from "./salvage";
import { mergeFinds } from "./report";
import { itemIcon, itemName } from "./items";   // run-clear
import { setBusyHost } from "./progress";
import { vaultSlots } from "./unlocks";
import { glossOf, noteText, setRefRows, verbLabel } from "./tokens";
import { EXIT_TRACE_ROWS, traceTable } from "./trace";
import { drivenDeath, exitExtras } from "./death";
import { laneTitle, seenForks } from "./route";
import { oathBeat } from "./oaths";
import { lastRun, markEnd, recordRun } from "./runlog";
import { FoldTally, carriedOf, foldFloors, stretchShare } from "./fold";
import { foldFloorsOf, openFoldReplay } from "./replay";
import { audio, type CueName, type CueOpts } from "../audio";
import { controlPanel } from "./control";   // take control (a secondary mode): the hero by hand
import { kwHost } from "./tips";   // RUNS_UI: the live badge's tip
import { goldWords } from "./gold-words";
   // RUNS_UI: the town tile's first-watches caption

type Tier = "bank" | "return" | "death";
/** QA 92eb880 (N: the `fights` chip read `1.332247798006322×` over the portrait): a rate as the chip shows it — whole from 2×, one
 *  decimal under it (`1.3`), never a float's tail. */
const CAGE_BATCH = 8, CAGE_NEAR = 10, CAGE_CHAIN = 24;   // Cut 20 §3: short batches chained per pump pass near a cage
/** The alert's top (the core's cap: `run.alert` never passes 8) — the HUD reads `alert 3/8`. */
const ALERT_TOP = 8;
/** QA 92eb880: an unopened cage (`vault` tile) seen within CAGE_NEAR tiles of the hero. */
export function cageNear(s: Snapshot): boolean {
  const hx = s.hero.x, hy = s.hero.y;
  for (let y = Math.max(0, hy - CAGE_NEAR); y <= Math.min(s.h - 1, hy + CAGE_NEAR); y++)
    for (let x = Math.max(0, hx - CAGE_NEAR); x <= Math.min(s.w - 1, hx + CAGE_NEAR); x++) { const i = y * s.w + x; if (s.tiles[i] === "vault" && s.seen[i]) return true; }
  return false;
}
export const rateText = (r: number): string => (r >= 2 ? String(Math.round(r)) : String(Math.round(r * 10) / 10));
type FrameName = "map" | "fight";
/** Cut 8A: the real renderer's frame cut (web/src/render/index.ts); the placeholder viewer has none. */
type FrameViewer = Viewer & { setFrame?(frame: FrameName, focus?: { x: number; y: number; radius: number }): void };
const LEAD = 12, BATCH = 10;        // ticks: pump when the engine is < LEAD ahead; step BATCH at a time
const BATCH_FAST = 12;              // at 8× the viewer eats 4 ticks per pump; a bigger batch keeps the queue fed through a slow step
const LEAD_FAST = 32;               // Cut 7 §4: at 8× the engine stays ≥ ENDING_TICKS ahead, so an exit is seen before its last 30 ticks
const BATCH_FIGHTS = 16;            // Cut 10 §1: under the card the engine steps this many ticks per call, chained flat out (≈ 1 ms a call)
const TOLL_BANNER_MS = 3000;   // QA a946e04: `from D1 · toll short` as the watch opens
const CARD_MS = 1000, CARD_BEAT_MS = 500;   // Cut 10 §1: the interstitial's minimum on a new floor (`D4 · 17 rooms`), and between fights on the same floor (Cut 15 §4: ≤ 1.2 s with the pump's slack under load, was 1.5)
const CARD_MAX_MS = 1000;                   // Cut 15 §4: the card is never up longer; past it with no fight found, the live map at the flat rate
const FOE_NEAR = 3;                         // Cut 15 §4: a hostile within this many tiles keeps the fight frame's slow clock in `fights`
const FIGHT_TAIL = 12;              // Cut 10 §1: in `fights` the frame lets go this many ticks after the last hold (the kill's dissolve), not AUTO_TAIL
const BLOW_TICKS = 20;              // Cut 10 §1: in `fights` the frame holds on an adjacent hostile, a blow in the last 20 ticks, or a boss in view
// Cut 10 §1: under the card the engine runs each fight to its end first and only a fight that cost something is shown — the hero
// hurt ≥ SHOW_HURT hp in it, or under SHOW_HP of max hp, a boss, a companion fallen, a theft, or the run ending in it; the rest
// pass under the card (a DEFAULT run's fights alone ran ~110 s at 1×; the gate is 90 s with ≥ 3 shown)
const SHOW_HURT = 4, SHOW_HP = 0.25;
const STALL_FLAT_TICKS = 300;       // a stall flag that has held this long is a stall: flat rate, ▶▶| to the run's end
// QA 524827b (qaAB: ▶▶| every 0.4 s in `fights`, a D10 run still took 114 s — each floor started the count over): presses in quick
// succession on any floors, this many, also take the run's end (the player is asking for it, not for the next fight)
const PRESS_ANY = 8;
const PRESS_GAP_MS = 2500, PRESS_LOOP = 3;   // QA 778fa1b (qaV): ▶▶| pressed again within this long on the same floor, this many times over, is a loop — the run's end
const SKIP_WALL_MS = 4000;          // QA 23ed91f (L: 60 taps of ▶▶| over a 4-minute stall, 300 s on D1 in `fights`): a skip steps for at most this much wall time, then lands live — the press always moves the picture; the next press goes on
const SKIP_END_BATCH = 100;         // ▶▶| in `fast` steps to the run's end in batches this size: a step's cost is its snapshot, not its ticks (≈ 30 ms a call on the fast wasm build, so 10-tick batches took 15 s to a D5 death)
const SKIP_FIGHT_BATCHES = 12_000;  // Cut 10 §1: ▶▶| steps to the next fight or the run's end (the run cap in BATCHes; ≈ 4 000 ticks
                                    // landed on a paced stretch that looked the same — "inert", three QA players on Cut 12)
const ENDING_TICKS = 20;            // Cut 7 §4: the last ticks before any exit play at 1× (Cut 14: 20, was 30)
const SCENE_FOES = 2;               // Cut 7 §4: awake hostiles in the hero's room that make it a scene
const AMBIENT_MS = 10_000, AMBIENT_SHOW_MS = 1500;   // Cut 7 §4: one ambient callout per 10 s, shown 1.5 s whatever the speed
const PUMP_MS = 25;
const KEEP_MS = 100;                // Cut 28 §4: the keep-out rects' refresh (a layout read)
const PULSE_MS = 120;               // Cut 14 §6: the strip's dot retriggers its beat at most this often
const CATCHUP_RATE = 32;            // Cut 14 §6: `fast` on the map, behind live by more than the lead: the picture catches up at this rate
const CATCHUP_MAX = 200;
const LEAD_PROBE = 120;             // Cut 18 §1: `fast`'s engine lead — a fight is costed (shown or dropped, as under the card) before the picture meets it
// Cut 24 §1 (AL: a Warlord fight > 4 min at 1× with the boss bar full; AK: ~100 s of retreat ↔ pack break): the watch never shows more
// than 15 s without an hp change, a kill, a pickup or a descent — a stretch DEAD_TICKS past the last of them plays as travel (the mode's
// travel rate, ramping like `fast`'s dead stretch), in any mode and any frame, the fight's included, and lands at the next one
const DEAD_TICKS = 70, DEAD_MAX_FIGHTS = 48;   // seven actions (a tick is a tenth of one): 3.5 s of a fight at 2× (Cut 28 §3: was 100)
// Cut 28 §3 (AU: ~50 s of `pick up ×N` and 9–12 s gaps at 1×): the plain 1× goes dead after 3.5 s of nothing (35 ticks at 1×, was 10 s),
// so no 1× stretch passes 5 s without a fight, a beat, a pickup of note or a descent — the travel plays the rest
const DEAD_TICKS_ONE = 35;
// QA ad71e72 (rater A: 40 s–2 min of a boss standoff at D8 / D10 / D18 with no new line — the dead stretch's rate rides the engine's
// short lead, ~16–30× however long the stretch): a dead stretch the picture has played DEAD_JUMP_MS of wall time jumps — the engine
// steps ahead in big batches (≤ DEAD_JUMP_WALL_MS) to the next move, the stairs, a beat or the end, and the picture lands JUMP_LAND
// ticks before it; with none found it lands on the frontier and plays on (another jump DEAD_JUMP_MS later)
const DEAD_JUMP_MS = 2500, DEAD_JUMP_WALL_MS = 1200, JUMP_LAND = 15;
const KILL_LEAD = 3;                // blind 5331f40: a jump that meets a boss's unshown kill lands this many ticks before it
// blind 7f7fc2b (A: three 55–90 s static stretches after the fold line; B: the hero static > 30 s with the run over): a jump held short
// of an unshown kill waits at most KILL_WAIT_MS of wall time for the picture to play it — then the kill lapses and the jump goes on
const KILL_WAIT_MS = 3000;
// …and a picture whose clock has not moved for STUCK_MS while the run is live, unfrozen and nothing deliberate holds it (a beat, the
// cage, the exit flow, a fold) is landed live and the card let go — whatever held it, the watch never freezes
const STUCK_MS = 4000;
const CALM_MIN = 10;                // Cut 28 §3: a core calm stretch this long (ticks, 1 s at 1×) or more plays as travel
const RISE_FREE = 16, RISE_MS = 150;   // blind 1fb7786 (B): the clock jumps freely up to the fights travel; past it, it doubles at most every RISE_MS
const NEWS_MAX_MS = 8000;           // blind 77030eb (B): Normal never stands longer without news — the stretch is jumped
const ONE_MAX = 4;                  // blind 5331f40: `one` (Speed Normal) — a dead stretch travels at most 4× (then jumps, announced)
const DEAD_RAMP_MS = 300, DEAD_STEP_MS = 200, DEAD_LAND_MS = 150, FAST_MAX = 128;   // Cut 18 §1: `fast`'s dead-stretch ramp (see `deadRate`)            // Cut 14 §6: the engine's biggest step when the world is behind its clock (a paused or hidden viewer)
/** Cut 25 §3 (AN: ~55 s of max hp draining 41 → 17 with only numbers moving): a drain — the hero's hp or max hp falling with no blow
 *  (hunger, poison, a curse). The core's `drain` (its word, or `true`) when it sends one, else a cause the word table knows. A drain is
 *  no progress (the dead stretch plays it as travel) and no fight (no near hold); its word shows once a floor. */
/* copy:callout */
const DRAIN_WORDS: Record<string, string> = { hunger: "starving", starving: "starving", starve: "starving", poison: "poisoned", poisoned: "poisoned", curse: "cursed", cursed: "cursed", drain: "drained", drained: "drained", bleed: "bleeding", bleeding: "bleeding", wither: "withering" };
const drainWord = (cause: string): string => DRAIN_WORDS[cause] ?? cause.replace(/_/g, " ");
/** blind c4705f9 (B: "hunger I never understood"): the hunger's word names its cause where it shows (`starving · no light`: an unlit hunger
 *  floor bites the max; a lit shrine or a lantern stops it) — other drains say their word alone. */
export const drainSaid = (word: string, cause: string): string => /^(hunger|starv)/.test(cause) ? /* copy:callout */ `${word} · no light` : word;
const DRAIN_CALLOUT = /^(hunger|poison|curse|drain(ed)?|bleed) [−-]\d+( max)?$/;
export function drainOf(e: Ev): string | null {
  if (e.k !== "hurt" && e.k !== "max_hp") return null;
  const d = (e as { drain?: string | boolean }).drain;
  if (d === false) return null;
  if (typeof d === "string" && d) return drainWord(d);
  if (d === true) return drainWord(e.cause);
  // without the core's word, only a cause no blow carries (hunger, a curse) reads as one — poison may be a fight's (the core's `drain` says when not)
  return /^(hunger|starv|curse)/.test(e.cause) ? drainWord(e.cause) : null;
}
// Cut 25 §3 (AM: "I never found a plain 1x"): `one` — the plain 1×: every frame at 1×, no card; a dead stretch (a drain included) still
// plays as travel (from AUTO_FAST, ramping), as in every mode
type Mode = "fights" | "fast" | "one";
const RATE: Record<Mode, number> = { fights: 16, fast: 32, one: 1 };  // fights: the map when it shows without a hold (draining to an exit); fast: the travel (Cut 12 §6, was 8×; Cut 20 §3: 32× from the first tick, was 16× ramping)
const FAST_NEAR = 4;                // Cut 12 §6: `fast` watches anything near at 2× — Cut 14: 4× ("way too slow")
const FIGHT_RATE: Record<Mode, number> = { fights: 2, fast: FAST_NEAR, one: 1 };   // Cut 14: the fight frame's clock (was 1× · 2×)
// Cut 20 §3 (AD: "early runs at 1× too short to follow" — 22–67 s of card-skipped travel): in `fights` on D1–D3 there is no card —
// the travel plays at EARLY_TRAVEL and every fight at EARLY_FIGHT (the fight is what to follow)
// (dev: `?early=0` turns it off, so the card's own gates can run on the fake's gentle D1)
const EARLY_FIGHT = 1.5, EARLY_TRAVEL = 8, EARLY_TRAVEL_MAX = 64;
const EARLY_DEPTH = (() => { try { const q = new URLSearchParams(location.search).get("early"); return (import.meta.env.DEV || new URLSearchParams(location.search).get("dev") === "1") && q !== null ? Number(q) : 3; } catch { return 3; } })();
const AUTO_FAST = 8, AUTO_TAIL = 10; // a tapped card holds the map at 8×; the pump's "fast" threshold; near holds until AUTO_TAIL ticks after the last sighting / hp change (Cut 14: 10, was 20)
const CALLOUT_MIN_MS = 500;         // Cut 12 §6: a callout stays readable at 16×
/** Cut 23 §3: a tap within this long of a reasoned line (a `✗` refusal, a shouted word) opens its reason; the reason shows this long. */
const WHY_TAP_MS = 6000, WHY_SHOW_MS = 3500;
const EXIT_GRACE_MS = 4000;         // wait for the viewer to drain after an exit, at most this long
const PERSIST_MS = 5000;
const VAULT_WAIT_MS = 10_000;       // Cut 15 §5: the override sheet holds the world this long at most (wall time); Cut 19 §1: it opens only on a tap on
                                    // the cage beat (10 s, was the default path at 6 s) — then the preference picks; watching it is optional, never a toll
const VAULT_GRACE = 50;             // the core's grace (ticks) between a cage opening and the preference's pick (turn.rs VAULT_GRACE)
const CAGE_WAIT_CAP_MS = 8000;      // Cut 19 §1: the world waits for the cage beat's line at most this long (a beat the playhead never reaches)
/** Cut 19 §1: the cage beat's line — `took mail` (the pick's last two words: a callout is ≤ 3 words). */
export const tookText = (label: string): string => /* copy:callout */ `took ${label.trim().split(/\s+/).slice(-2).join(" ")}`;
const BOSS_BANNER_MS = 3000;
// Cut 27 §1: a folded stretch's line holds at least this long as the interstitial (the stepping under it is ~0.1–0.3 s a floor), and
// the fold gives up (lands live) after FOLD_WALL_MS of stepping — a stall or a loop on a solved floor is watched, never folded away
const FOLD_MIN_MS = 1400, FOLD_WALL_MS = 8000, FOLD_DOCK_MS = 5000;        // Cut 2 §7: `counter: known|unknown` on first sight (Cut 22: the boss bar's lit second line)
const REST_BEAT_MS = 1400;
const REST_BEAT_FAST_MS = 700;      // QA e75ec29: in `fast` the rest line holds 0.7 s — `fights` on D1–3 ramps its travel now, and `fast` keeps ≤ 0.4 of it (Cut 20 §3)
const METER_PAINT_MS = 400;          // Cut 29 §3: the compact meter repaints at most this often (its numbers move every snapshot)
const METERS_KEY = "riddle.meters";
const readMetersOn = (): boolean => { try { return localStorage.getItem(METERS_KEY) === "1"; } catch { return false; } };
const writeMetersOn = (on: boolean): void => { try { localStorage.setItem(METERS_KEY, on ? "1" : "0"); } catch { /* a per-viewer convenience */ } };
const VAULT_FULL_MS = 1200;         // QA e75ec29: `vault full` before the report when the preference kept (no sheet)          // Cut 2 §1: `rest 12m` after the exit, before the exit flow continues
const CHORE_CALLOUT: Record<string, string> = { descend: /* copy:callout */ "descend", pick_up: /* copy:callout */ "pick up" }; // explore never (Cut 4 §4)
const CORE_LINE_MS = 1000, CAPTION_LINE_MS = 1500;   // Cut 18 §2: the renderer's callout and caption lifetimes (render/state.ts)
const KILL_MS = 60_000;             // QA 524827b: the killing blow's line stays on the 0-hp frame
const HURT_MS = 600;                // Cut 4 §4: `−7 archer` in red
const FELL_MS = 1400;               // Cut 10 §3: `jackal Ashar fell` stays long enough to read a name
const SCENE_MS = 4000, SCENE_TICKS = 40;   // Cut 13 §4: a situation's beat holds the fight frame this long (~4 s at 1×)
// Cut 18 §1: a beat holds in WALL time — the frame on it, its line on the ticker — whatever the mode's rate: BEAT_HOLD_MS for a
// situation, SCENE_MS for a boss's kill / break and the bank (rater Y: "`GOBLIN WARLORD DOWN` went by in about a second": the kill
// was the floor's last blow, the stairs a few ticks on — the descend cut the line and faded the frame)
const BEAT_HOLD_MS = 3000;
// Cut 20 §3: in `fast` a situation's beat (a den, a cage, a theft) holds a third as long — its line still reads (1 s, twice the callout minimum); a boss's kill or
// break and the bank keep SCENE_MS in both modes (`fast` ≤ 0.4 × `fights` over a run: five situations a floor held it at ~0.6)
const BEAT_HOLD_FAST_MS = 1000;
const BEAT_HOLD_EARLY_MS = 1800;           // QA e75ec29 (R: `fights` 28–40 s on D1, a third of it two 3 s beats — a shrine, a theft): `fights` on D1–D3
const FAST_ENDING = 2;                     // Cut 20 §3: `fast` plays the walk-out (the last ENDING_TICKS) at 2×
const PET_GAP_MS = 1200;                   // QA e75ec29: queued companion lines, one per this
/** QA e75ec29: the core's companion notes — `Skog joins.`, `Skog: level 3.` — as ticker lines (`Skog joins`, `Skog level 3`). */
const PET_NOTE_RE = /^([A-Z][a-z]+)(?: joins\.|: level (\d+)\.)$/;
const petNoteLine = (t: string): string => { const m = PET_NOTE_RE.exec(t)!; return m[2] ? /* copy:callout */ `${m[1]} level ${m[2]}` : /* copy:callout */ `${m[1]} joins`; };
const BEAT_MAX_MS = 6000;                  // a beat's line is gone after this whatever the clock did, and at a floor change (QA on 50bb162: `A den.` over D2→D4 for 15 s)
const CALLOUT_QUEUE = 3, SAME_TICK_MS = 40;   // Cut 13 §4: callouts that land on one tick (one pump pass) wait their turn, at most this many
/** Cut 13 §4: the notes that are beats — the core's situation lines (verbatim), a theft, the stray, a heir's bones. */
// Cut 24 §2: each biome band's omen (`omen:<biome>`, three variants each, core chronicle.rs) cuts in like a situation
const BEAT_RE = /^(A den\.|A cage:|A shrine\.|The cage opens|A cry from the dark|The air stings|The nest wakes|The den wakes|Found heir \d+'s bones|found the bones|Freed the captive|Cut the captive|Lit the shrine|A loose brick|A rat's hoard|A dropped purse|A dead delver's pack|Old bones clutch|A satchel under|A clean spring|Rain through a crack|Sweet water in the reeds|A bell tolls|Bells in the dark|A far bell|A cooling ingot|Slag with silver|A smith's lost purse|A drowned purse|Coins in a lurker's|A glint under the black water|Offerings on a cold altar|Coins in a dry font|A pilgrim's purse)|snatched|\bstole\b|, gone wild\b|is avenged/i;
/** The cause of a `hurt` as one word: `goblin_archer` → `archer`. */
const oneWord = (cause: string): string => cause.replace(/_/g, " ").trim().split(/\s+/).pop() ?? "";
/** A player row's callout, ≤ 3 words: `R2 · attack nearest`, `R4 · pack break` (QA 23ed91f, L: `R4 PACK BREAK GOBLIN` — the target goes). */
export const rowCallout = (_row: number, verb: string): string => verb.trim().split(/\s+/).slice(0, 3).join(" ");
/** The hero's hp lost as a callout: `−3 hp · archer` (QA 23ed91f, K: `−1 rat` read as a kill count, "one rat fewer"). */
export const hurtText = (dmg: number, cause: string): string => /* copy:callout */ `−${dmg} hp · ${oneWord(cause)}`;
/** Cut 13 §3: the automations' purchases at this exit — the ledger's outgoings after the newest exit line that are not salvage
 *  (`−$40 heal potion` → `heal potion ×1 · −$40`), per line text. Empty when nothing was bought. */
export function spentOf(ledger: { t: number; delta: number; why: string; n?: number }[], cat: { kind: string; label: string; price: number }[] = []): { kind: string; n: number; gold: number }[] {
  let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled|driven)\b/.test(ledger[i].why)) i--;
  if (i < 0) return [];
  const rows = new Map<string, { kind: string; n: number; gold: number }>();
  for (const g of ledger.slice(i + 1)) {
    if (g.delta >= 0 || /^(salvage|wake pay|insure)/.test(g.why)) continue;
    // QA 23ed91f (L: `SPENT heal ×1 · −$80` for a restock that rebought two $40 heals; `leash ×1` on a run where nothing was bought):
    // one ledger line can pay for several at the supply's price, and a restock keeps its word
    const bare = g.why.replace(/^(bought|restock)\s+/, "");
    const kind = /^restock\s/.test(g.why) ? /* copy:label */ `restock ${bare}` : bare;
    // QA 778fa1b (qaV: `repeat heal ×1 · −$104` for four $26 heals): the core's own count (`GoldLine.n`), else the supply's price
    const price = cat.find((e) => e.label === bare || e.kind === bare.replace(/^repeat\s+/, ""))?.price ?? 0;
    const n = g.n !== undefined && g.n > 0 ? g.n : price > 0 && -g.delta % price === 0 ? -g.delta / price : 1;
    const r = rows.get(kind) ?? { kind, n: 0, gold: 0 }; r.n += n; r.gold += -g.delta; rows.set(kind, r);
  }
  return [...rows.values()];
}
/** QA a946e04 (T: `+$71 banked · −$40 spent` while the camp went $51 → $32 — the $50 toll left out): the waystone toll this run's send
 *  paid — the ledger's `waystone D5` lines between the exit before the newest and the newest (the send pays before the run), as a
 *  SPENT row (`waystone D5 ×1 · −$50`). */
export function tollOf(ledger: { t: number; delta: number; why: string }[]): { kind: string; n: number; gold: number }[] {
  const isExit = (g: { why: string }): boolean => /^(returned|banked|died|lost|stalled|driven)\b/.test(g.why);
  let i = ledger.length - 1; while (i >= 0 && !isExit(ledger[i])) i--;
  if (i < 0) return [];
  let j = i - 1; while (j >= 0 && !isExit(ledger[j])) j--;
  const rows = new Map<string, { kind: string; n: number; gold: number }>();
  for (const g of ledger.slice(j + 1, i)) {
    if (g.delta >= 0 || !/^waystone\b/.test(g.why)) continue;
    const r = rows.get(g.why) ?? { kind: g.why, n: 0, gold: 0 }; r.n++; r.gold += -g.delta; rows.set(g.why, r);
  }
  return [...rows.values()];
}
/** Cut 16 §3: a biome's title on its first floor's card. */
/* copy:callout */
const BIOME_TITLE: Record<string, string> = { warrens: "the Warrens", burrows: "the Burrows", fens: "the Fens", crypt: "the Crypt", foundry: "the Foundry", deep: "the Deep", sanctum: "the Sanctum" };
/** Cut 16 §4: the boss's break — the core's callout `warlord breaks` / note `The Warlord breaks.` → `WARLORD BREAKS`. */
const BREAK_RE = /^(?:the )?([a-z]+) breaks\.?$/i;
/** Cut 15 §4: a boss kill's callout — the boss's name (its last two words, upper case) + DOWN: `goblin_warlord` → `GOBLIN WARLORD DOWN`. */
export const bossDown = (kind: string): string => /* copy:callout */ `${kind.replace(/_/g, " ").trim().split(/\s+/).slice(-2).join(" ").toUpperCase()} DOWN`;
/** Cut 12 §4: `nest` → `a nest`, `orchard` → `an orchard` (one word after the article). */
export const withArticle = (w: string): string => { const x = oneWord(w); return `${/^[aeiou]/i.test(x) ? "an" : "a"} ${x}`; };

/** Describe the watched picture separately from the core's current floor. */
export function watchStatus(picture: number, live: number, paused: boolean, ended: boolean): { kind: string; label: string; detail: string } {
  if (ended) return { kind: "ended", label: /* copy:label */ "Run ended", detail: /* copy:label */ `Watching D${picture}` };
  if (paused) return { kind: "paused", label: /* copy:label */ "Watch paused", detail: /* copy:label */ "continues away" };
  if (picture !== live) return { kind: "earlier", label: /* copy:label */ `Watching D${picture}`, detail: /* copy:label */ `Live D${live}` };
  return { kind: "live", label: /* copy:label */ "Live delve", detail: /* copy:label */ "continues away" };
}

export function renderWatch(app: App): Mounted {
  setRefRows(() => app.rules.rows);
  const tactics = tacticObserver(app.rules.rows, app.lineage.packages);
  const canvas = h("canvas", { class: "view" });
  // Cut 17 §1: the hero's hp is the ring around the console's portrait (its numbers on the plate under it)
  const hpText = h("span", { class: "num hp-text" });
  const gun = gunStatus();
  const face = portrait(app, { hp: 1, label: h("span", { class: "watch-vitals" }, hpText, gun.el) });
  let playedGun: GunSnap | null = null;
  function paintGun(): void {
    if (viewer?.gun) playedGun = viewer.gun();
    gun.paint(playedGun, viewer ? viewerTick() : 0);
  }
  // Cut 16 §4: the boss's bar under the hero's while one is in view (`warlord` + a thin track)
  const bossFill = h("span", { class: "fill" }), bossName = h("span", { class: "name" });
  const bossFace = h("span", { class: "face" });   // art pass: the boss's painted headshot (else its sprite crop)
  // Cut 22 (AH: "'WARLORD RALLIES' overlapped by the 'BOSS · COUNTER: ATTACK BOSS' banner and goblin nameplates stacked — cluttered at the
  // key moment"): the counter is the boss bar's second line (`counter: attack boss`, lit for BOSS_BANNER_MS, then dim) — the top lane;
  // the hero's callout keeps the middle, the ticker's beat the bottom, and the renderer draws only the boss's plate while he is in view
  const bossCounter = h("span", { class: "counter num", hidden: true });
  let counterTimer = 0, quietTimer = 0;
  const bossBar = h("div", { class: "boss-hp", hidden: true }, unitLabel("", bossName, { art: h("span", { class: "boss-face", "aria-hidden": "true" }, bossFace) }), h("span", { class: "track" }, bossFill), bossCounter);
  let bossFaceKind = "";
  const depth = h("span", { class: "num depth" });
  const alert = h("span", { class: "alert num" });
  const ticker = h("div", { class: "ticker" });
  // c30-legible: the run's end, why — its own line under the ticker (`hurt · banked`, `hurt · went home`, `slain · jackal`)
  const whyLine = h("div", { class: "beat-why num", "aria-live": "polite" });
  let whyTimer2 = 0;
  function showWhy(text: string, ms: number): void {
    text = goldWords(text);
    replace(whyLine, text); whyLine.classList.add("show"); el.classList.add("has-why");
    clearTimeout(whyTimer2); whyTimer2 = window.setTimeout(() => { whyLine.classList.remove("show"); el.classList.remove("has-why"); }, ms);
    if ("__riddle" in window) ((window as unknown as { __whyLog?: string[] }).__whyLog ??= []).push(text);   // dev
  }
  // Cut 23 §3: a reasoned line's reason, on tap (the line's text → the reason), over the ticker
  const whyTip = h("div", { class: "why-tip num", "aria-live": "polite" });
  const whyOf = new Map<string, string>();
  let lastWhy: { text: string; why: string; at: number } | null = null, whyTimer = 0;
  const stake = h("div", { class: "stake num" });
  const banner = h("div", { class: "banner num" });
  // Cut 17 §1: `⏸ / ▶` is the console's gem (the glyph stays the button's text; the icon is drawn over it)
  const pause = gem({ label: "", cls: "hud-btn", onclick: () => togglePause() });
  // the last chosen mode is the next run's (app.watchMode, persisted — QA on e0f87e7: "`fast` chosen in run 3 was not remembered")
  const mode0: Mode = app.watchMode === "fast" || app.watchMode === "one" ? app.watchMode : "fights";
  const modeBtn: Record<Mode, HTMLButtonElement> = {
    fights: tile({ id: "fights", cls: "hud-btn game-control", on: mode0 === "fights", icon: "fights", label: /* copy:button */ "fights only", onclick: () => setMode("fights") }),   // docs/COPY.md pass 3: `fights` read as a combat log 6/6
    fast: tile({ id: "fast", cls: "hud-btn game-control", on: mode0 === "fast", icon: "fast", label: /* copy:button */ "fast", onclick: () => setMode("fast") }),
    one: tile({ id: "one", cls: "hud-btn game-control", on: mode0 === "one", icon: "one", glyph: "1×", label: /* copy:button */ "normal", onclick: () => setMode("one") }),   // docs/COPY.md pass 5: `1` over `1×` read "no idea"
  };
  const skip = tile({ id: "skip", cls: "hud-btn game-control", icon: "skip", label: "▶▶|", onclick: () => skipToEvent() });
  const bail = tile({ id: "bail", cls: "hud-btn bail game-control", icon: "bail", label: /* copy:button */ "bail", onclick: () => doBail() });
  // RUNS_UI (docs/RUNS_UI.md): back to the town while he goes on — leaving the watch never stops the run (the town's lane shows it live;
  // the open app's clock plays it on, unwatched). The ↻ on the tile is the mark; its tip says the rest
  const toTown = tile({ id: "town", cls: "hud-btn town-btn", icon: "camp", glyph: "⌂", label: /* copy:button */ "Town menu", onclick: () => { if (!done) app.leaveWatch(); } });
  // (its ↻ — he keeps going — is drawn on the tile's corner: runs.css `.town-btn::after`; the first watches also carry the one-time caption
  // `he keeps going` over it — the owner's concept rule: an icon and a one-time ≤ 3-word caption; its words are CSS's, not the tile's text)

  // Cut 10 §1: the interstitial — the ambient line over the map while the travel runs underneath; a tap holds the map at 8×
  const card = h("button", { class: "interstitial num", hidden: true, onclick: () => holdMap() });
  // Cut 27 §1: the fold line — the interstitial over a folded stretch (`D1–6 · 100% · +$84` and its chips), docked under the HUD once the
  // watch opens below it; a tap plays the folded floors
  const foldHead = h("span", { class: "fold-head num" }), foldChips = h("span", { class: "fold-chips num" });
  const foldLine = h("button", { class: "fold-line", hidden: true, onclick: () => foldTap() }, foldHead, foldChips);
  // Cut 14 §6: the scrub strip — the playhead (the viewer's share of the run) and the frontier's dot (beats per engine batch);
  // Cut 17: along the console's top edge
  const scrubHead = h("div", { class: "head" }), scrubDot = h("div", { class: "dot" });
  const scrub = h("div", { class: "scrub", hidden: true }, scrubHead, scrubDot);
  // The exit's explicit resource repaint must still name the hero in this replay.
  const bar = renderBar(app, { watch: true, heir: app.lineage.live?.heir ?? app.lineage.heir, trait: app.lineage.trait });
  const busyHost = h("span", { hidden: true });   // the engine's busy label at the end (the next gem says it): not in the corner
  // Cut 29 §3: the compact meter over the stage (the fight in progress, else the run so far), toggled from the command card and
  // remembered per viewer; the desktop's right column carries the run's whole breakdown
  let metersOn = readMetersOn(), lastMeters: SnapMeters | undefined, meterPaintAt = 0, meterTimer = 0;
  const meterBox = h("div", { class: "meter-box", hidden: !metersOn });
  const meterTile = tile({ id: "meters", cls: "meter-btn game-control", on: metersOn, icon: "meters", glyph: "▤", label: /* copy:button */ "meters", onclick: () => {
    metersOn = !metersOn; writeMetersOn(metersOn); meterTile.classList.toggle("on", metersOn); meterBox.hidden = !metersOn; paintMeters(true);
  } });
  const speedBtn = tile({ id: "speed", cls: "hud-btn", icon: "fast", label: /* copy:button */ "Speed", onclick: () => {
    openWindow((close) => {
      for (const m of Object.keys(modeBtn) as Mode[]) modeBtn[m].onclick = close;
      return h("div", { class: "sheet-body watch-options" },
        h("div", { class: "label" }, /* copy:label */ "Watch speed"),
        h("div", { class: "chips" }, modeBtn.one, modeBtn.fights, modeBtn.fast),
        h("div", { class: "label" }, /* copy:label */ "Run controls"),
        h("div", { class: "chips" }, skip, meterTile, bail));
    });
  } });
  const speedMode = h("small", { class: "watch-speed-mode" }, mode0 === "one" ? /* copy:label */ "Normal" : mode0 === "fast" ? /* copy:label */ "Fast" : /* copy:label */ "Fights only");
  speedBtn.querySelector(".tl")!.append(" ", speedMode);
  // blind c4705f9 (A, B: `fights only` 16× persisted across runs and both believed they watched at 1×): the tile carries the clock the
  // picture plays at now (`16×`, `2×` in a fight, `1×`), lit whenever it is not normal speed
  const speedRate = h("b", { class: "watch-speed-rate num", "aria-hidden": "true" });
  speedBtn.append(speedRate); speedBtn.dataset.mode = mode0;
  const cons = renderConsole({ portrait: face.el, tiles: [speedBtn, toTown], gem: pause, top: scrub, compact: true });
  const combatRows = h("ol", { class: "combat-lines", "aria-live": "off" });
  let logFollowing = true;
  const logLatest = h("button", { class: "log-latest", hidden: true, onclick: () => {
    logFollowing = true; combatRows.scrollTop = combatRows.scrollHeight; logLatest.hidden = true;
  } }, /* copy:button */ "Latest ↓");
  combatRows.tabIndex = 0;
  combatRows.setAttribute("aria-label", /* copy:label */ "Combat history");
  combatRows.addEventListener("scroll", () => {
    logFollowing = combatRows.scrollHeight - combatRows.clientHeight - combatRows.scrollTop <= 2;
    logLatest.hidden = logFollowing;
  });
  const combatLog = h("div", { class: "combat-log", "aria-label": /* copy:label */ "Combat log" }, logLatest, combatRows);
  let logDepth = app.lineage.live?.depth ?? 1;
  let repeatedRule: { key: string; el: HTMLElement; count: number; badge: HTMLElement } | undefined;
  function logEvent(ev: Ev, heroId: number, names: ReadonlyMap<number, string>, warningText?: string): void {
    const ruleKey = ev.k === "rule" && ev.row >= 0 ? JSON.stringify([logDepth, ev.row, ev.text, ev.verb?.v, ev.verb?.a]) : undefined;
    if (ruleKey && repeatedRule?.key === ruleKey && combatRows.lastElementChild === repeatedRule.el) {
      repeatedRule.count++;
      repeatedRule.badge.textContent = ` ×${repeatedRule.count}`;
      repeatedRule.el.dataset.tick = String(ev.t);
      if (logFollowing) combatRows.scrollTop = combatRows.scrollHeight;
      return;
    }
    repeatedRule = undefined;
    let text: (string | HTMLElement)[] | undefined;
    const who = (id: number): string => names.get(id) ?? (id === heroId ? /* copy:label */ "Hero" : /* copy:label */ "Foe");
    const damage = (amount: number, id: number): HTMLElement => h("span", { class: amount < 0 ? "log-heal" : id === heroId ? "log-hurt" : "log-damage" }, `${amount < 0 ? "+" : "−"}${Math.abs(amount)} hp`);
    const item = (name: string): HTMLElement => h("span", { class: /gold|coin|\$/.test(name.toLowerCase()) ? "log-gold" : "log-item" }, itemIcon({kind:name, label:name}, {size:"xs"}), name);
    if (ev.k === "attack") text = [`${who(ev.src)} → ${who(ev.dst)} · ${ev.src === heroId && gunShotText(ev.verb) ? `${gunShotText(ev.verb)} ` : ""}`, ev.hit ? (ev.dmg > 0 ? damage(ev.dmg, ev.dst) : /* copy:label */ "blocked") : /* copy:label */ "miss"];   // QA ad71e72: never `−0 hp`
    else if (ev.k === "hurt") text = [`${ev.cause === "hero" ? who(heroId) : ev.cause.replace(/_/g, " ")} → ${who(ev.id)} · `, damage(ev.dmg, ev.id)];
    else if (ev.k === "heal" || ev.k === "recover") text = [`${who(ev.id)} · ${ev.src.replace(/_/g, " ")} `, damage(-ev.amount, ev.id)];
    else if (ev.k === "telegraph") text = [h("span", { class: "log-warning" }, `${who(ev.id)} · ${warningText ?? ev.what}`)];
    else if (ev.k === "die") text = [h("span", { class: "log-fell" }, `${who(ev.id)} · fell`)];
    else if (ev.k === "use") text = [item(ev.item), ` · ${ev.outcome}`];
    else if (ev.k === "pickup") text = [`${who(ev.id)} · `, item(ev.item)];
    else if (ev.k === "callout") text = [ev.text];
    else if (ev.k === "rule" && ev.row >= 0) text = [ev.text];
    else if (ev.k === "descend") { logDepth = ev.depth; text = [h("span", { class: "log-floor" }, /* copy:callout */ `Floor ${ev.depth}`)]; }
    else if (ev.k === "exit") text = [ev.line?.text ?? `${ev.tier} · $${ev.loot_kept}`];
    if (!text) return;
    const row = h("li", { "data-tick": String(ev.t), "data-first-tick": String(ev.t) }, h("small", { class: "log-depth" }, `D${logDepth} · `), ...text);
    if (ruleKey) {
      const badge = h("span", { class: "log-repeat log-depth num" });
      row.appendChild(badge); repeatedRule = { key: ruleKey, el: row, count: 1, badge };
    }
    combatRows.appendChild(row);
    while (combatRows.childElementCount > 80) {
      const first = combatRows.firstElementChild as HTMLElement;
      const height = logFollowing ? 0 : first.offsetHeight;
      first.remove();
      if (!logFollowing) combatRows.scrollTop = Math.max(0, combatRows.scrollTop - height);
    }
    if (logFollowing) combatRows.scrollTop = combatRows.scrollHeight;
  }
  const wideMeters = h("div", { class: "meters-live" });
  const wide = wideCols(app, wideMeters, () => Number(depth.dataset.floor ?? app.lineage.live?.depth ?? app.lineage.start ?? 1));   // desktop: the rules left, the shaft right (wide.css) — the run's meters under the shaft
  function paintMeters(now = false): void {
    if (disposed || !lastMeters) return;
    const t = performance.now(), remaining = METER_PAINT_MS - (t - meterPaintAt);
    if (!now && remaining > 0) {
      // Retry the latest already-reached snapshot, even if the picture pauses
      // before another batch lands. Never read the engine's future snapshot.
      if (!meterTimer) meterTimer = window.setTimeout(() => { meterTimer = 0; paintMeters(); }, remaining);
      return;
    }
    clearTimeout(meterTimer); meterTimer = 0; meterPaintAt = t;
    if (metersOn) replace(meterBox, compactLine(lastMeters));
    if (wide.slot) replace(wideMeters, meterPanel(lastMeters.run, app.rules.rows, { title: /* copy:label */ "this run" }));
  }
  // Distinguish the watched picture from the core's ongoing run; leaving town still lets him continue.
  const watchLabel = h("span", { class: "watch-status-label" }, /* copy:label */ "Live delve");
  const watchDetail = h("span", { class: "lb-auto" }, " · ", /* copy:label */ "continues away");
  const liveBadge = kwHost(h("span", { class: "live-badge", "data-live": "1" }, h("i", { class: "lane-beat", "aria-hidden": "true" }), watchLabel, watchDetail), "live");
  // take control: the panel (a toggle, the pad, the turn's actions); an action kicks one engine step (the world waits for the next)
  let manualKick = false;
  const ctl = controlPanel(app.engine as unknown as Parameters<typeof controlPanel>[0], () => { manualKick = true; });
  const el = h("main", { class: "watch frame" }, bar.el,
    h("div", { class: "stage" }, canvas, card, foldLine,
      h("div", { class: "hud top" }, depth, liveBadge, alert, bossBar, stake),
      meterBox, banner, h("div", { class: "watch-messages" }, whyTip, ticker, whyLine, tactics.el, combatLog)),
    ctl.el, cons.el, ...wide.els);

  let viewer: Viewer | null = null;
  let mode: Mode = mode0, paused = false, slowUntil = -Infinity, lastHp = NaN;
  paintPause();
  // Cut 10 §1: the card's state — up, since when (its minimum), a tap holding the map, a fight waiting on the minimum; fights shown
  let cardUp = false, cardSince = 0, cardMin = CARD_MS, cardDepth = 0, cardText = "", mapHold = false, cardWait = false, fights = 0;
  // Cut 15 §4: when the card last went up (the cap's clock: `cardSince` is waived by a press), and the live map the cap gave way to
  let cardShownAt = 0, cardLive = false;
  // QA on 3d71c33: the floor's card shows once per floor entry — a card between fights on a floor already carded is a ghost (the
  // travel still runs under it, the picture holds, nothing is drawn over the map: `data-ghost`)
  let cardGhost = false;
  // QA e75ec29 (R: "`▶▶|` in `fights` changed nothing visible"): on the first floors (no card) a press outside a fight means the
  // next fight worth watching — the card goes up and the travel runs under it (fights that cost nothing pass there) until the next
  // shown fight cuts in; inside a fight the press jumps to the fight's end and then does the same
  let skipEarly = false;
  // QA e75ec29 (R: "the pet is never in the text … the whole pet life happened inside the skip"): a companion's news — tamed, joins,
  // levels, falls — waits here when it lands under the card or inside a skip, and takes the ticker once the picture is watched
  const petNews: string[] = [];
  let skipping = false, lastPetAt = 0;
  const foeSpans: { from: number; until: number }[] = [];   // Cut 15 §4: engine-tick spans with a hostile near (the fight frame's slow clock in `fights`)
  const bossIds = new Set<number>();                          // Cut 15 §4: entities tagged `boss` (a kill of one is a beat)
  el.dataset.mode = mode; el.dataset.fights = "0"; el.dataset.card = "0";
  const allies = new Set<number>();   // Cut 10 §3: ids the snapshot flags as allies, so a companion's death reads `jackal Ashar fell`
  let fell: { t: number; name: string; kind: string } | null = null;   // the companion whose death the core's own callout (`Ashar fell`) names next
  let killBlow = "";                  // QA 524827b (qaAA): the hero's killing blow as its callout (`−1 hp · monkey`), once absorbed
  let heroCause: string | undefined;  // the hero's `die` cause (the client-built report counts the death it came from)
  // Cut 7 §4: the scene's room (null = none) and the tick auto may run fast again after one ends; the ending's first tick;
  // the exit batch held back until the viewer is ENDING_TICKS from the exit; the ambient callout limiter; the last alert
  let scene: number | null = null, sceneUntil = -Infinity, endingFrom = Infinity, endingCue = -Infinity;
  // Cut 8A: the fight frame — whether the engine's latest snapshot holds it, the viewer ticks it spans, the frame shown
  let fightOn = false, fightFrom = Infinity, fightUntil = -Infinity, frame: FrameName = "map", lastBlow = -Infinity;
  // Cut 13 §4: the situation beat the frame is holding for (engine ticks), its text shown once at the cut; beats shown so far
  type Beat = { from: number; until: number; text: string; shown: boolean; exit?: boolean; hold?: boolean; cage?: boolean; why?: string };
  let beat: Beat | null = null, beats = 0;   // hold: Cut 15 §4, the clock at 1× through it (a boss's kill)
  let exitBeatUntil = 0;              // Cut 14 §3: the exit flow waits while the bank / return beat is on screen
  let holdLineUntil = 0;              // Cut 15 §4: a boss kill's `WARLORD DOWN` keeps the ticker this long
  // Cut 18 §1: the beat on screen holds until this wall time — the frame stays on it (no card, no floor load, the playhead eased to
  // the beat's end or the tick before the stairs), its line keeps the ticker; a beat whose tick comes meanwhile waits its turn
  let beatHoldUntil = 0;
  const beatNext: Beat[] = [];
  // blind 5331f40 (both Mirror King kills ended off-view: `Run ended · Watching D33`, then the report): a boss's kill is a witnessed
  // beat — no jump (live, the ending, a dead stretch's jump, a skip) lands past one the playhead has not shown; `floorFrom` the tick the
  // viewer's floor opened (a kill on a floor already left cannot be shown again)
  const killBeats: Beat[] = []; let floorFrom = -Infinity;
  const killWait = new Map<Beat, number>();   // blind 7f7fc2b: when a jump was first held short of the kill (wall ms)
  let killsLapsed = 0;
  let heldBeat: Beat | null = null;   // the beat the hold is for
  const descends: number[] = [];      // engine ticks of the run's descends (a held beat stops short of the stairs)
  // Cut 24 §1: the engine ticks where the watch moved — a blow that landed, a hurt, a drink, a kill, a pickup, a descent (ascending)
  const progress: number[] = [];
  let newsKey = "", newsAt = performance.now();   // blind 77030eb: Normal's news watchdog (NEWS_MAX_MS)
  let jumpsSaid = 0; let deadFrom = -1, lastJumpAt = 0, jumps = 0, stuckTick = -1, stuckAt = 0, unsticks = 0;   // QA ad71e72: the dead stretch's wall start, the jumps, the stuck-picture watchdog
  let keepClose: (() => void) | null = null;   // blind 1fb7786: the open keep sheet's close (it keeps the ticked picks) — the gem's tap goes on through it
  let riseAt = 0;                     // blind 1fb7786: when the clock last eased up (`eased`)
  let deadSince = -1;                 // Cut 18 §1: `fast` — when the current dead stretch began (wall ms; -1 none): its rate ramps
  let chore: { text: string; n: number; shown: string; depth: number } | null = null;   // Cut 14 §4: the chore callout streak on the ticker (`pick up ×8`)
  const rowFires: number[] = [];      // Cut 14 §4: this run's `rule` events per row (the death screen's least-fired row)
  // Cut 10 §1: the fight the engine is running through under the card (its cost so far), and whether the found fight is to be shown
  let probe: { hurt: number; low: boolean; boss: boolean; ally: boolean; steal: boolean } | null = null, fightShow = false;
  let held: { evs: Ev[]; snap: Snapshot; tier: Tier } | null = null;
  let lastAmbient = -Infinity, ambientUntil = 0, lastAlert = 0;
  const refused = new Set<string>();  // Cut 12 §6: sanity refusals shown (`drink ✗ no use@3`): once per text per floor
  const drainsShown = new Set<string>();
  // Cut 25 §3 (core): the drain stretch the core opened (`Ev drain`, its word) — until a foe is in view, the stairs or the run's end the
  // hero's `hurt` / `max_hp` are the drain's; `drainEvs`: the events read as a drain (no progress, no near hold)
  let drainOn: string | null = null;
  const drainEvs = new WeakSet<Ev>();
  const isDrain = (e: Ev): boolean => drainEvs.has(e) || !!drainOf(e);
  const drainTicks: number[] = [];   // Cut 25 §3: the hero's drain ticks (a drain since the last move is a dead stretch at once)   // Cut 25 §3: a drain's word shown (`starving@9`): once per floor
  let rallyBy: string | undefined;    // Cut 12 §6: the kind whose `rallies` telegraph came last, so the core's `rallied!` names it
  let speed = 1, done = false, disposed = false, overridden = false, tickerTimer = 0, bannerTimer = 0, pumpTimer = 0;
  let tickerAt = 0, tickerMs = 0; const tickerQueue: { text: string; cls: string; ms: number }[] = [];   // Cut 13 §4: callouts waiting their turn
  // Cut 2: rest after the exit, bones left (death) / found, bosses already announced
  let restS: number | undefined, restUntil = 0, bonesLeft: number | undefined;
  let exitLine: ExitLine | undefined;   // Cut 6 §1: the exit's ledger line (exit sheet, report, death)
  let exitTrace: Trace | undefined;     // Cut 9 §5: the exit's last-5 trace (on the event, or on its line)
  const bonesFound: string[] = []; const bossSeen = new Set<number>();
  let counters = namedCounters(app.lineage);   // Cut 6 §5: bosses with a named counter row, re-read on a sighting
  let snap: Snapshot | null = null;
  let runId = -1, engineTick = 0, startTick = 0, inflight = false, lastPersist = performance.now();
  const loads: { snap: Snapshot; rest: Ev[]; at?: number }[] = [];   // at: Cut 25 §3, the descend tick that opens the floor   // Cut 14 §6: floors the engine reached that the viewer has not (a queue, oldest first)
  // Cut 14 §6: the world clock (engine ticks, wall time × worldRate), the pump's last wall stamp, the hidden tab, the seek to live owed
  // after a catch-up, the engine batches landed (the dot's beats), fight spans the viewer has yet to reach
  let worldT = 0, lastPumpMs = performance.now(), hidden = false, goLiveOwed = false, pulses = 0, lastPulseMs = 0;
  const spans: { from: number; until: number }[] = [];
  // Cut 14 §6 (QA on 56f2a1d: the HUD ran D1 → D3 and a fight cut in after the tap): while the picture is frozen (⏸, a hidden tab)
  // nothing reaches the viewer, the HUD or the ticker — the engine's batches wait here (in order) and are fed on the thaw; the
  // pump neither drains a floor, releases a queued HUD item, cuts a frame nor travels under the card; the world steps on its clock
  const frozenFeed: { evs: Ev[]; s: Snapshot }[] = [];
  let frozenAt = 0;
  // Cut 14 §6: the near and scene holds as the PLAYHEAD meets them — every near tick and every scene's span (engine ticks), so a
  // viewer behind the world slows where the events are, not where the engine is (`slowUntil` / `sceneUntil` stay the world's)
  const nearTicks: number[] = [], scenes: { from: number; until: number }[] = [];
  let exitTier: Tier | null = null, exitAt = 0;
  let pendingExit: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[]; decide?: boolean; note?: string } | undefined;
  // what the exit sheet let go, at the engine's worth — the report's `salvaged` rows (QA on 952e306: "camp $76 after
  // 'Returned with $57'; only the gold sheet shows +$19 salvage"); the deepest floor this send reached (its `deepest` tile)
  let salvagedRows: { kind: string; n: number; gold: number }[] = [];
  // Cut 24 §5 (AK, AL: "I tapped `pearly potion?`, then keep — SALVAGED listed it"): the engine kept the tapped one (a twin of the same
  // flavour, or the return's cut, was what sold) and the vault named it by its kind — the report says what went in (`KEPT strength → vault`)
  let keptLabels: string[] = [];
  let deepest = 0;
  // Cut 5 §4: the open vault sheet's close, and the cage it was opened for (a dismissed sheet is not reopened)
  let vaultClose: (() => void) | null = null, vaultKey = "", vaultAt = 0;   // Cut 15 §5: vaultAt — when the sheet opened (wall ms)
  let graceBar: HTMLElement | null = null;   // the cage sheet's shrinking bar (held while the picture is frozen)
  let vaultItems: InvItem[] = [], vaultChosen = false;   // the open cage's items, and whether the player picked (else the wait did)
  // Cut 19 §1: the open cage — its choice, the pick the beat named, when it arrived (wall ms), whether its line has shown, and done
  // (the hold is over, a tap chose, ▶▶| let it go): the world waits while `cageWaits()`
  let cage: { vc: VaultChoice; pick?: InvItem; at: number; shown: boolean; done: boolean } | null = null;
  function cageWaits(): boolean {
    if (vaultClose) return true;   // the override sheet is open
    if (!cage || cage.done) return false;
    if (cage.shown) return beatHeld() && !!heldBeat?.cage;
    return performance.now() - cage.at < CAGE_WAIT_CAP_MS;
  }
  /** The bar at its share of the wait left, standing (⏸) or running out over the rest of it. */
  function graceHold(): void { if (!graceBar) return; const left = Math.max(0, 1 - (performance.now() - vaultAt) / VAULT_WAIT_MS); graceBar.style.transitionDuration = "0s"; graceBar.style.width = `${(left * 100).toFixed(1)}%`; }
  function graceRun(): void {
    const bar = graceBar; if (!bar) return;
    const leftMs = Math.max(0, VAULT_WAIT_MS - (performance.now() - vaultAt));
    requestAnimationFrame(() => { bar.style.transitionDuration = `${(leftMs / 1000).toFixed(2)}s`; bar.style.width = "0%"; });
  }
  let prepended = false;              // bail fell back to the row prepend (an engine without `bail`)
  const cls = app.lineage.class;
  let recordBeat = false;   // run-clear: the run's `NEW BEST` stamp, once
  const RECORD_MS = 2400;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  const kinds = new Map<number, string>(), names = new Map<number, string>();
  const bossHeard = new Set<number>();   // juice pass 2: the bosses whose entrance was heard (once each)
  const partyAtStart = (app.lineage.party ?? []).map((c) => `${c.kind} · ${c.name}`);
  const tamedIds: number[] = [], lostIds: number[] = [];
  const compLabel = (id: number): string => { const n = names.get(id); return `${kinds.get(id) ?? "?"}${n ? ` · ${n}` : ""}`; };
  const note_ = (e: { id: number; kind: string; name?: string }): void => { kinds.set(e.id, e.kind); if (e.name) names.set(e.id, e.name); };
  // HUD updates released at the viewer's clock
  const hud = { hp: 0, maxHp: 1, depth: 1 };
  /** blind c4705f9 (B: `hp 7/25 · max -26` "from hunger I never understood"): the max a drain took this run, by its word (`starving −26`) */
  const maxLoss = new Map<string, number>();
  // the snapshot the stake line was last painted from (the card's loot is the stake's), and each floor's rooms / situation as the
  // engine reported them (the card names the HUD's floor, which may be behind the engine's — QA on 56f2a1d: HUD `17/40 D4` under
  // `D5 · 15 rooms · a shrine`, `$6 · keeps $3` under `D2 · 16 rooms · $13`)
  let hudSnap: Snapshot | null = null;
  // Cut 19 §2: a return walks to the up-stairs — once the player's return row has fired (the last row to act, at the viewer's clock)
  // the stake line reads `returning` (its `return at 20%` is spent) until another row acts
  let walkingHome = false;
  const floors = new Map<number, { rooms?: number; twist?: string; biome?: string }>();
  // Cut 16 §4: the boss in view as the HUD shows it (released at the viewer's clock), and each boss's break beat once
  let bossHud: { id: number; kind: string; hp: number; max: number; tags:string[]; modifiers?:EncounterModifiers } | null = null;
  const broke = new Set<string>();
  const timed: { t: number; f: () => void }[] = [];
  let lastRuleIsRow = false;   // docs/COPY.md: a row's callout has no `R2 ·` marker any more
  let lastRuleText = "", lastRuleAt = 0, lastShown = "", lastCoreAt = -Infinity;
  /** gfx round 2: the wide frame's read-only tablet of the rule that just acted glows (a phone has no rules column: nothing to light). */
  const lightRow = (row: number): void => { const t = el.querySelector<HTMLElement>(`.rules-col .row[data-i="${row}"]`); if (!t) return; t.classList.remove("firing"); void t.offsetWidth; t.classList.add("firing"); };
  // placeholder viewer (no clock): a wall clock at 10 ticks/s × speed stands in
  let fbTick = 0, fbAt = performance.now();
  function viewerTick(): number {
    if (viewer?.tick) return viewer.tick();
    const now = performance.now(); fbTick += ((now - fbAt) / 1000) * 10 * speed; fbAt = now;
    return Math.min(fbTick, engineTick);
  }
  const viewerIdle = (): boolean => viewer?.idle ? viewer.idle() : true;

  let statusKey = "";
  function paintWatchStatus(): void {
    const state = snap?.manual && !held && !exitTier ? { kind: "live", label: /* copy:label */ "Hand control", detail: snap.awaiting ? /* copy:label */ "awaits order" : /* copy:label */ "acting" } : watchStatus(hud.depth, snap?.depth ?? hud.depth, paused, !!held || !!exitTier);
    const key = `${state.kind}:${state.label}:${state.detail}`;
    if (key === statusKey) return;
    statusKey = key;
    liveBadge.dataset.status = state.kind;
    liveBadge.dataset.live = state.kind === "live" ? "1" : "0";
    watchLabel.textContent = state.label;
    watchDetail.textContent = ` · ${state.detail}`;
    paintKeepOut(true);
  }
  function paintHud(): void {
    const p = hud.maxHp ? hud.hp / hud.maxHp : 0;
    face.set(p);
    stake.classList.toggle("warn", p < 0.4);
    const lost = [...maxLoss].filter(([, n]) => n > 0);
    replace(hpText, /* copy:callout */ `${Math.max(0, hud.hp)}/${hud.maxHp} hp`, ...lost.map(([w, n]) => h("small", { class: "hp-max-loss", "data-cause": w, title: /* copy:tooltip */ `max hp −${n} · ${w}` }, /* copy:callout */ ` ${w} −${n}`)));   // docs/COPY.md pass 5: `28/36` read as XP or rooms
    replace(depth, `D${hud.depth}`);
    depth.dataset.floor = String(hud.depth); wide.paintDepth?.();
    paintWatchStatus();
    // QA 23ed91f (K: "`!` / `!!` / `!!!` after the depth label, and `alert 1` / `alert 3`"): one name for one thing — the HUD reads
    // `alert 3`, as the callout does when it rises; nothing at 0
    // QA 0c6e126 (qaY: `alert 1` / `alert 2` with no scale): the level out of its top (the core's cap, 8: each rise calls wanderers)
    if (snap) {
      const text = snap.alert > 0 ? /* copy:callout */ `alert ${snap.alert}/${ALERT_TOP}` : "";
      if (alert.textContent !== text) { replace(alert, text); paintKeepOut(true); }
    }
    if (cardUp) paintCardText();
    // QA 524827b (qaAA: the final frame `0/36` under `−1 hp · jackal`, the trace's last blow the monkey's): at 0 hp the line names the
    // killing blow — whatever a skip released last, a held beat or a queue kept on the ticker
    if (hud.hp <= 0 && killBlow && !cardUp && lastShown !== killBlow) { tickerQueue.length = 0; showTicker(killBlow, "hurt", KILL_MS); }
  }
  /** Cut 16 §4: the boss bar — the name one word, the track its hp share; hidden with no boss in view. */
  function paintBoss(): void {
    bossBar.hidden = !bossHud; el.dataset.boss = bossHud ? `${bossHud.hp}/${bossHud.max}` : "";
    if (!bossHud) return;
    replace(bossName, oneWord(bossHud.kind));
    if (bossFaceKind !== bossHud.kind) {
      if (bossFaceKind) { bossCounter.hidden = true; clear(bossCounter); }   // another boss: his own counter line, when sighted
      bossFaceKind = bossHud.kind;
      enemyHost(bossName, bossHud.kind, app.lineage, false, () => ({modifiers:bossHud?.modifiers,modifier_catalogue:hudSnap?.modifier_catalogue,alive:!!bossHud&&bossHud.hp>0,status_effects:bossHud?.tags.includes("hexed")?app.lineage.class_styles?.offers.filter(o=>o.id==="hexbinder").map(o=>o.effect):[]}));
      const k = bossHud.kind.replace(/^boss_/, "");
      if (!paintPortrait(bossFace, `boss_${k}`)) paintSprite(bossFace, `boss_${k}`, 26, k);
    }
    bossFill.style.width = `${Math.round(Math.max(0, Math.min(1, bossHud.hp / Math.max(1, bossHud.max))) * 100)}%`;
  }
  /** Cut 16 §4: the boss in the snapshot's view (not an ally), at its tick. */
  function bossFrom(s: Snapshot): void {
    const b = s.entities.find((e) => e.tags.includes("boss") && !e.ally && e.hp > 0 && !!s.visible[e.y * s.w + e.x]);
    const was = bossHud?.id;
    bossHud = b ? { id: b.id, kind: b.kind, hp: b.hp, max: b.max_hp, tags:b.tags, modifiers:b.modifiers } : null;
    if (b && b.id !== was && !bossHeard.has(b.id)) { bossHeard.add(b.id); cue("boss_in"); }   // juice pass 2: a boss's entrance is a beat you hear
    if (b || was !== undefined) paintBoss();
  }
  function hudFrom(s: Snapshot): void { if (s.depth !== hud.depth) hideBeat(); if (!disposed) audio.bed(s.biome); bossFrom(s); floors.set(s.depth, { rooms: s.rooms ?? floors.get(s.depth)?.rooms, twist: s.floor_twist ?? floors.get(s.depth)?.twist, biome: s.biome }); playedGun = s.hero.gun ?? null; paintGun(); hud.hp = s.hero.hp; hud.maxHp = s.hero.max_hp; hud.depth = s.depth; deepest = Math.max(deepest, s.depth); paintHud(); paintStake(s); }
  // Cut 2 §7: `$47 · sword⚠ · return at D4`; `death: lose all` when no row would bank or return
  let lastLoot: number | undefined, lastLootRun = -1, lootDrop = 0, lootDropUntil = 0, lootWhy = "";
  let lastLootTurn = -Infinity, lastSwapped = 0, lastSecured = 0;   // counters of the snapshot the strip shows
  let lastPickT = -Infinity, lastStealT = -Infinity, lastPickItem = "", lootItem = "";   // engine ticks of the last `pickup` / `steal` (the loot's fall names which)
  let tollShort = false;   // QA a946e04 (T): the send's waystone start fell back to D1 (the purse could not pay the toll)
  let stolenGold = 0;   // QA a946e04 (T: `−$36 stolen` on the strip, nothing in STOLEN): what the run's thefts took off the carry (the `steal` amounts)
  function paintStake(s: Snapshot): void {
    // QA 912e135 (qaW: `carry $110` → `$102 −$8 swap → ashen scroll?` — the core's carry never fell there; the death line's `−$2 swapped`):
    // the stake paints from two clocks (a batch's snapshot at its tick, a floor load's older one when the viewer reaches the floor) — an
    // older snapshot of the same run is behind what the strip already shows, and never reads as a fall
    if (s.stake && s.run.id === lastLootRun && s.turn < lastLootTurn) return;
    hudSnap = s;
    if (s.meters) { lastMeters = s.meters; paintMeters(); }   // Cut 29 §3: the meter as the viewer reaches the batch
    if (cardUp) paintCardText();
    const st = s.stake;
    stake.hidden = !st;
    if (!st) return;
    // QA 912e135: a `swap` is the core's own counter rising (`Stake.swapped`, the exit line's `swapped` at the end) — one source for the
    // strip and the death line, its size the counter's rise; an older core falls back to the pickup's window
    const swapD = st.swapped !== undefined && s.run.id === lastLootRun ? st.swapped - lastSwapped : 0;
    const secured = st.death_keep ?? 0;
    const securedD = s.run.id === lastLootRun ? Math.max(0, secured - lastSecured) : 0;
    // A record checkpoint transfers carry into secured gold. Only the net fall
    // beyond that transfer is a loss; explicit swap deltas remain authoritative.
    const carryDrop = lastLoot !== undefined && s.run.id === lastLootRun ? Math.max(0, lastLoot - st.loot - securedD) : 0;
    // QA 92eb880 (M: "gold `$77 → $65` in the den with only `snatched …` lines"): a fall in the loot shows its size beside it for 2.5 s
    // (`$65 −$12`) — a theft of an item takes its worth with it
    if (lastLoot !== undefined && s.run.id === lastLootRun && (swapD > 0 || carryDrop > 0)) {
      // QA 0c6e126 (qaY: `−$5 swap → waxen scroll?` read as a price to pay): a swap's fall names what it left on the floor (the core's
      // `Stake.swap_left`) — `−$5 left axe`: the carry fell because a dearer item stayed behind; an older core keeps the find
      const why = swapD > 0 ? (st.swap_left ? /* copy:label */ "left" : /* copy:label */ "swap") : s.turn - lastStealT <= 20 ? /* copy:label */ "stolen"
        : st.swapped === undefined && s.turn - lastPickT <= 20 ? /* copy:label */ "swap" : "", item = why === "left" ? st.swap_left! : why === "swap" ? lastPickItem : "";
      // QA 778fa1b (qaV: `−$33 → −$42 → −$58 swapped` — "swapped for what?"): a swap's fall is its own, beside the find it made room for
      // (`−$37 swap → leather +1`); only falls of one cause and one find inside the window add up
      const same = performance.now() < lootDropUntil && why === lootWhy && item === lootItem;
      const fell = (why === "swap" || why === "left") && st.swapped !== undefined ? swapD : carryDrop;
      lootDrop = fell + (same ? lootDrop : 0); lootDropUntil = performance.now() + 2500; lootItem = item;
      // QA 1a2a4a9 (O, P: `$46 −$8`, `$283 −$100` — "minuses that don't match any line"): the fall says what took it — a thief, or an
      // item used up (the loot counts what he carries at its worth)
      // After excluding checkpoint transfers, a loss is theft or a pack swap — a spare weapon or armour dropped for a find.
      // Item use never subtracts from carry (the core's loot_add).
      lootWhy = why; }
    lastLoot = st.loot; lastLootRun = s.run.id; lastLootTurn = s.turn; lastSwapped = st.swapped ?? 0; lastSecured = secured;
    // QA 1a2a4a9 (O: the bar's `$0` and the line's `$3 · death: lose all` on one screen, "neither labelled"): the run's own purse says so
    // blind 1fb7786 (A: `Carried $0 · Secured $6988`, then `Carried $1329` — "don't reconcile"): `Carried` is the core's own carried
    // (`Run::carried`: the secured gold with the carry since — the exit line's `$N carried`), so a checkpoint never reads as a fall to $0
    // and Secured is always the part of Carried a death keeps; the stake's `loot` is only the carry at risk
    const parts: (string | HTMLElement)[] = [h("span", { class: "carry-w" }, /* copy:label */ "Carried"), ` $${Math.max(0, st.loot) + secured}`];
    if (performance.now() < lootDropUntil && lootDrop > 0) parts.push(" ", h("span", { class: "loot-drop down" }, `−$${lootDrop}${lootWhy ? ` ${lootWhy}` : ""}${lootItem ? `${lootWhy === "left" ? " " : " → "}${lootItem.replace(/_/g, " ")}` : ""}`));
    // blind 7f7fc2b (A: `Secured $0` every run after heir 1, "nothing ever secured"): carry is secured only past the record (Cut 30.5's
    // checkpoint) — below it, with nothing yet secured, the line names the floor that secures it (`Secured past D21`)
    const record = app.lineage.best_depth ?? 0;
    parts.push(" · ", h("span", { class: "kept" }, secured === 0 && s.depth <= record ? /* copy:callout */ `Secured past D${record}` : /* copy:callout */ `Secured $${secured}`));
    if (st.returning ?? walkingHome) parts.push(" · ", h("span", { class: "returning" }, /* copy:callout */ "Heading home"));
    if (st.stalling && s.depth >= hud.depth && !overridden) parts.push(" · ", h("span", { class: "stalling" }, /* copy:callout */ "Path blocked"));
    replace(stake, ...parts);
  }
  function showBanner(text: string, ms: number, cls = ""): void {
    replace(banner, text); banner.className = `banner num show ${cls}`;
    clearTimeout(bannerTimer); bannerTimer = window.setTimeout(() => banner.classList.remove("show"), ms);
  }
  /** Cut 22: the boss's counter on his bar's second line, lit for `ms`, then dim (it stays while he is in view). */
  function showCounter(text: string, ms: number): void {
    if (folding) return;
    replace(bossCounter, text); bossCounter.hidden = false; bossCounter.classList.add("lit");
    clearTimeout(counterTimer); counterTimer = window.setTimeout(() => bossCounter.classList.remove("lit"), ms);
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
          showCounter(named ? /* copy:callout */ `counter: ${named}` : known() ? /* copy:callout */ "counter: known" : /* copy:callout */ "counter: unknown", BOSS_BANNER_MS);
        };
        app.engine.lineage().then((L) => { counters = L.counters ? namedCounters(L) : counters; }).catch(() => { /* the mounted lineage's counters stand */ }).finally(() => { if (!disposed) show(); });
      });
    }
  }
  function callout(text: string, cls = "", ms: number = Math.max(CALLOUT_MIN_MS, 1800 / Math.max(1, speed))): void {
    if (cardUp || folding) return;                                                         // Cut 10 §1: nothing under the card is watched (Cut 27 §1: nor under the fold)
    if (performance.now() < exitBeatUntil || performance.now() < holdLineUntil) return;     // Cut 14 §3: `COLLECTED $N` keeps the line (Cut 15 §4: `WARLORD DOWN` too)
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
  /** QA e75ec29: a companion's line — at once when the picture is watched, else queued (`petNews`) for the next watched moment. */
  function petLine(text: string): void {
    if (folding) return;   // Cut 27 §1: a folded floor's companion news is a chip on its line
    if (skipping || cardUp || !petFree()) { if (!petNews.includes(text)) petNews.push(text); return; }
    lastPetAt = performance.now(); callout(text, "ally", FELL_MS);
  }
  const petFree = (): boolean => { const now = performance.now(); return now >= exitBeatUntil && now >= holdLineUntil && now >= ambientUntil && !beatHeld(); };
  /** The queued companion lines, one per PET_GAP_MS, once the card is down and no skip or beat holds the line. */
  function flushPets(): void {
    if (!petNews.length || skipping || cardUp || frozen() || !petFree() || performance.now() - lastPetAt < PET_GAP_MS) return;
    lastPetAt = performance.now(); callout(petNews.shift()!, "ally", FELL_MS);
  }
  function showTicker(text: string, cls: string, ms: number): void {
    if (chore && text !== chore.shown) chore = null;   // Cut 14 §4: another line ends the chore streak
    lastShown = text; tickerAt = performance.now(); tickerMs = ms;
    replace(ticker, text); ticker.className = `ticker show ${cls}${whyOf.has(text) ? " has-why" : ""}`;
    paintKeepOut(true);
    scheduleTicker();
  }
  /** The ticker's next move: the queued callout once the current has had CALLOUT_MIN_MS, else the hide at the current's end. */
  function scheduleTicker(): void {
    clearTimeout(tickerTimer);
    const shownFor = performance.now() - tickerAt;
    const wait = Math.max(0, (tickerQueue.length ? CALLOUT_MIN_MS : tickerMs) - shownFor);
    tickerTimer = window.setTimeout(() => { if (disposed) return; const q = tickerQueue.shift(); if (q) showTicker(q.text, q.cls, q.ms); else ticker.classList.remove("show"); }, wait);
  }
  /** The beat's line off the ticker (and out of the queue): a floor change, or BEAT_MAX_MS after it showed. */
  function hideBeat(): void {
    if (beatHeld()) return;   // Cut 18 §1: the line holds its wall time
    for (let i = tickerQueue.length - 1; i >= 0; i--) if (tickerQueue[i].cls === "beat") tickerQueue.splice(i, 1);
    if (ticker.classList.contains("beat") && ticker.classList.contains("show")) { ticker.classList.remove("show"); scheduleTicker(); }
  }
  /** Cut 13 §4: a situation's note opens the fight frame for SCENE_TICKS from its tick (or rides a fight already framed there);
   *  its text is the callout, shown once the frame is up. */
  function beatAt(t: number, text: string, exit = false, hold = false, isCage = false, why?: string): Beat {
    const v = viewerTick();
    // a frame that is up (or opening) before t carries the beat; a fight the probe dropped (`fightFrom` cleared) does not
    const framed = fightFrom <= t && (fightOn || v < fightUntil);
    if (!framed) { fightFrom = t; fightUntil = t + SCENE_TICKS; fightShow = true; }
    else if (Number.isFinite(fightUntil)) fightUntil = Math.max(fightUntil, t + SCENE_TICKS);
    if (exit) fightUntil = Infinity;   // Cut 14 §3: the run is over — the frame holds; the exit flow's own clock (SCENE_MS, real time) lets go
    // an earlier beat the playhead has yet to reach keeps its place; this one takes over at its own tick (QA on 3d71c33: a den's
    // release showed the later `COLLECTED $13`, which had overwritten it, on D2 ~15 s before the bank)
    const b: Beat = { from: t, until: t + SCENE_TICKS, text, shown: false, exit, hold, cage: isCage, why };
    if (!(beat && !beat.shown && beat.from < t)) beat = b;
    el.dataset.beats = String(++beats);   // dev: tools count the beats cut in
    // Cut 18 §1: a beat the playhead jumped over (a skip, a seek to live) is not held after the fact
    at(t, () => { if (b.shown) return; if (!b.exit && viewerTick() >= b.until) { b.shown = true; if (b.cage && cage) cage.done = true; return; } beat = b; showBeat(true); });
    return b;
  }
  /** The beat's line, once the frame is up and the PLAYHEAD has reached the beat's tick (`reached`: released at the viewer's clock).
   *  A fight cut before it (a kept span the viewer replays behind the frontier) never carries a later beat's line (QA on 3d71c33:
   *  `COLLECTED $13` over a D2 fight in `fast`, ~15 s before the bank at D5). */
  function showBeat(reached = false): void {
    if (!beat || beat.shown || frame !== "fight") return;
    if (!reached && viewerTick() < beat.from - 1) return;
    // Cut 18 §1: a beat that comes while another is held waits for it — never replaces its line inside the hold (the exit's goes at once)
    if (beatHeld() && !beat.exit) { if (!beatNext.includes(beat)) beatNext.push(beat); return; }
    beat.shown = true;
    holdBeat(beat);
  }
  /** Cut 18 §1: the beat takes the line at once and holds it and the frame in wall time — SCENE_MS for a boss's kill or break and the
   *  bank / return (Cut 14 §3, Cut 15 §4: the exit flow waits `exitBeatUntil`), BEAT_HOLD_MS for a situation. */
  function holdBeat(b: Beat): void {
    const now = performance.now(), dur = b.exit || b.hold ? SCENE_MS : mode === "fast" && !b.cage ? BEAT_HOLD_FAST_MS : earlyFloor() && !b.cage ? BEAT_HOLD_EARLY_MS : BEAT_HOLD_MS;   // the cage's line is a tap target (its override): its full hold
    // the playhead already past the beat's span (it reached the tick before the engine's batch did): back to the beat's tick, so the
    // hold is on the beat, not on the frame after it
    if (!b.exit && viewerTick() > b.from + 4 && viewerTick() >= b.until - 4) { seekTo(b.from); }
    beatHoldUntil = now + dur; holdLineUntil = now + dur; heldBeat = b;
    // Cut 22 (AH: the boss moment's "clutter of overlapping text"): a boss's break or kill and the exit hold one line — nothing drawn
    // over the hero meanwhile (a telegraph's `RALLIED!` over `WARLORD BREAKS`)
    if (b.hold || b.exit) { viewer?.setQuiet?.(true); clearTimeout(quietTimer); quietTimer = window.setTimeout(() => viewer?.setQuiet?.(false), dur); }
    // the picture already past the stop (the stairs' fade applied): back to the beat's tick
    if (!b.exit && viewerTick() > beatStop()) seekTo(Math.max(b.from, beatStop() - 1));
    if (b.exit) exitBeatUntil = now + SCENE_MS;
    tickerQueue.length = 0; showTicker(b.text, b.cage ? "beat cage" : "beat", dur);
    // Cut 19 §1: the cage's line is a plate the finger finds (a tap within the hold opens the override)
    if (b.cage) { replace(ticker, h("span", { class: "cage-line" }, b.text)); if (cage) cage.shown = true; }
    // c30-legible (the owner: "I didn't understand … why the run ended early"): the end's reason under its sum, the core's ≤ 3 words
    // (`hurt · banked`, `hurt · went home`); a death's beat is its reason alone (`slain · jackal`)
    // (its own line under the ticker: the ticker's text stays the beat's own)
    if (b.why) showWhy(b.why, dur);
    el.dataset.held = "1";
    if ("__riddle" in window) ((window as unknown as { __beatLog?: unknown[] }).__beatLog ??= []).push({ text: b.text, why: b.why, from: b.from, until: b.until, v: viewerTick(), frame, ms: Math.round(now) });   // dev
  }
  const beatHeld = (): boolean => performance.now() < beatHoldUntil;
  /** Cut 18 §1: the tick a held beat's picture stops at — its span's end, or two ticks before the next stairs (a descend fades the
   *  frame and loads the next floor: the kill's frame would go with it). */
  function beatStop(): number {
    const b = heldBeat; if (!b) return Infinity;
    const d = descends.find((t) => t > b.from);
    return Math.min(b.until, d !== undefined ? d - 2 : Infinity);
  }
  /** Cut 18 §1: a ▶▶| (or a frozen skip) lets the beat go. */
  function releaseBeat(): void { beatHoldUntil = 0; holdLineUntil = 0; beatNext.length = 0; el.dataset.held = "0"; clearTimeout(quietTimer); viewer?.setQuiet?.(false); }
  /** Cut 14 §4: a chore's callout (`pick up`) repeats on its own line with a count — `pick up ×8` — until another line shows
   *  (rater T: "dead stretches of eight consecutive `pick up` reads"); the count is repainted in place, never queued. */
  function choreCallout(text: string): void {
    // QA 524827b (qaAA: `descend ×13`, `pick up ×10` — the approach to D4 in one line): a streak is one floor's; the stairs start another
    if (chore && chore.depth !== hud.depth) chore = null;
    if (chore && chore.text === text) {
      chore.n++;
      const line = /* copy:callout */ `${text} ×${chore.n}`;
      const inPlace = ticker.classList.contains("show") && lastShown === chore.shown && !cardUp;
      chore.shown = line;
      if (inPlace) showTicker(line, "", Math.max(CALLOUT_MIN_MS, 1800 / Math.max(1, speed))); else callout(line);
      return;
    }
    chore = { text, n: 1, shown: text, depth: hud.depth };
    callout(text);
  }
  /** Cut 18 §2: one line over the fight. The renderer draws the core's callout (a telegraph: `archer draws`) over the hero and hides
   *  the row's caption meanwhile (rater Z: `R4 HIT RANGED` stacked over `ARCHER DRAWS`); the row then reads on the ticker. */
  function ruleLine(text: string): void {
    const now = performance.now();
    if (cardUp || now < exitBeatUntil || now < holdLineUntil) return;
    showTicker(text, "rule", Math.max(CALLOUT_MIN_MS, CORE_LINE_MS));
  }
  /** A core callout was released: a row's caption from the last CAPTION_LINE_MS is now hidden by it — its text goes to the ticker. */
  function coreLine(): void {
    const now = performance.now(); lastCoreAt = now;
    if (frame === "fight" && lastRuleIsRow && now - lastRuleAt < CAPTION_LINE_MS) ruleLine(lastRuleText);
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
    if (ev.verb.v === "gunner_tactic") return null; // Automatic gun handling: real shot/skill/reload cues already show.
    if (tactics.owns(ev.row)) return null; // Known tactics use their icon; history retains their words.
    if (ev.row >= 0) return rowCallout(ev.row, verbLabel(ev.verb));
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
  /** Cut 16 §4: a boss's break (the callout or the note, whichever lands first) is a beat that holds the fight frame on
   *  `WARLORD BREAKS`; true when the text was one. */
  function breakBeat(t: number, text: string): boolean {
    const m = BREAK_RE.exec(text.trim()); if (!m) return false;
    const name = oneWord(m[1]).toUpperCase(), key = `${name}@${hud.depth}`;
    if (!broke.has(key)) { broke.add(key); beatAt(t, /* copy:callout */ `${name} BREAKS`, false, true); at(t, () => cue("boss_break")); }
    return true;
  }
  function at(t: number, f: () => void): void { timed.push({ t, f }); }
  function release(upTo: number): void {
    if (!timed.length) return;
    const keep: typeof timed = [];
    // Cut 18 §1: under a held beat nothing past its stop is released (the stairs' HUD change waits for the hold)
    for (const x of timed) { if (x.t <= upTo && !(beatHeld() && heldBeat && !heldBeat.exit && x.t > beatStop())) x.f(); else keep.push(x); }
    const released = timed.length - keep.length;
    timed.length = 0; timed.push(...keep);
    // Combat history and wrapped causes can move the whole message stack.
    // Measure after the released batch has completed all of its DOM changes.
    if (released) paintKeepOut(true);
  }
  const victims = new Map<number, string>();   // id → label, remembered across batches so a kill inside a batch still has a name
  const logActors = new Map<number, string>(); // includes companions; captured per batch for delayed display
  let exitOath = "";   // Cut 28b: an oath decided at the run's end, for the exit's beat
  function absorb(evs: Ev[], s: Snapshot): Tier | null {
    let exit: Tier | null = null;
    const heroId = s.hero.id;
    for (const e of s.entities) { if (e.ally) allies.add(e.id); else if (e.id !== heroId) victims.set(e.id, (e.name ?? e.kind).replace(/_/g, " ")); if (e.tags.includes("boss")) bossIds.add(e.id); }
    // A spawned actor can die before the batch's final snapshot. Its spawn
    // still carries the real identity; current snapshot names take precedence.
    for (const ev of evs) if (ev.k === "spawn") {
      note_(ev.e);
      logActors.set(ev.e.id, (ev.e.name || ev.e.kind).replace(/_/g, " "));
    }
    logActors.set(heroId, s.hero.name || /* copy:label */ "Hero");
    for (const e of s.entities) logActors.set(e.id, (e.name || e.kind).replace(/_/g, " "));
    const logged = combatLogEvents(evs, heroId, kinds, tactics.owns);
    if (logged.length) {
      // Capture only actors in these rows; movement-only batches copy no names.
      const actorNames = new Map<number, string>();
      const capture = (id: number): void => { const name = logActors.get(id); if (name) actorNames.set(id, name); };
      capture(heroId); // A hurt-only batch still names its actual hero source.
      for (const ev of logged) {
        if (ev.k === "attack") { capture(ev.src); capture(ev.dst); }
        else if ("id" in ev) capture(ev.id);
      }
      for (const ev of logged) {
        const warning = ev.k === "telegraph" ? telegraphText(kinds.get(ev.id), ev.what) : undefined;
        at(ev.t, () => logEvent(ev, heroId, actorNames, warning));
      }
    }
    for (const ev of evs) {
      if (ev.k === "drain") drainOn = drainWord(ev.cause);
      else if (ev.k === "descend" || ev.k === "exit" || (ev.k === "attack" && ev.dst === heroId)) drainOn = null;   // the stairs, the end, a blow: the stretch is over
      else if (drainOn && (ev.k === "hurt" || ev.k === "max_hp") && ev.id === heroId) drainEvs.add(ev);
      if (ev.k === "telegraph" || ev.k === "attack" || ev.k === "use" || (ev.k === "hurt" && ev.id === heroId && !isDrain(ev))) near(ev.t);   // Cut 25 §3: a drain is no fight   // Cut 5 §5: always at 1×
      switch (ev.k) {
        case "gun": at(ev.t, () => { playedGun = ev.state; paintGun(); }); break;
        // Cut 10 §3: the core's companion-death callout (`Ashar fell`) gets its kind in front: `jackal Ashar fell`
        case "callout": {
          if (ev.text === "explore" || /^Gunner\s+✗/.test(ev.text)) break; // Default gun handling yields to chores without a warning.
          // Cut 26 §2: once a floor. QA 308f045 (qaAD: `TWO STAIRS` seen once in ten runs, inside a `▶▶|`; fights and fast runs crossed D4 → D5
          // without it): the fork is a beat, cut in as the situations are (the fight frame, its line), in every mode — not a callout the mode drops
          if (/^two stairs$/i.test(ev.text)) { const key = `stairs@${s.depth}`; if (!refused.has(key)) { refused.add(key); beatAt(ev.t, /* copy:callout */ "TWO STAIRS"); } break; }
          if (ev.text === /* copy:none */ "choose one") break;   // Cut 19 §1: the cage beat names the pick instead
          // c305-core: a new record is the core's callout (`new best · D5`) — one beat, drawn as run-clear's gilt stamp (`NEW BEST D5`), once a run
          { const m = /^new best · D(\d+)$/i.exec(ev.text); if (m) { if (!recordBeat) { recordBeat = true; const d = Number(m[1]); at(ev.t, () => { if (folding) return; showBanner(/* copy:callout */ `NEW BEST D${d}`, RECORD_MS, "record-beat"); cue("level"); }); } break; } }
          if (breakBeat(ev.t, ev.text)) break;   // Cut 16 §4: `warlord breaks` is the beat's, not a plain callout
          // Cut 12 §6: a sanity refusal (`drink ✗ no use`) shows once per floor, not once per streak
          if (ev.text.includes("✗")) { const key = `${ev.text}@${s.depth}`; if (refused.has(key)) break; refused.add(key); }
          // Cut 25 §3: a drain's own callouts (`hunger −1 max`, bite after bite) — never; its word shows once a floor instead (`starving`)
          if (DRAIN_CALLOUT.test(ev.text)) break;
          let f = ev.text;
          // Cut 12 §6: a summoned ally (no name) reads `ally hound fell`; a companion keeps `jackal Ashar fell`
          if (fell && fell.t === ev.t && fell.kind && ev.text === (fell.name ? `${fell.name} fell` : `${fell.kind} fell`)) f = fell.name ? `${fell.kind} ${fell.name} fell` : /* copy:callout */ `summoned ${oneWord(fell.kind)} fell`;
          // Cut 12 §6: the core's `rallied!` names the boss whose telegraph it answers (`warlord rallies`)
          else if (ev.text === /* copy:none */ "rallied!") f = /* copy:callout */ `${oneWord(rallyBy ?? "boss")} rallies`;
          // Cut 23 §3 (AI, AJ: `read ✗ no use`, `IT SWELLS` — "unclear"): a `✗` refusal and a shouted word carry their reason on tap —
          // the core's `why`, else a refusal's gloss off the vocabulary (`no use` → the core's words)
          const why = ev.why ?? (ev.text.includes("✗") ? glossOf(app.vocab?.why_gloss, ev.text) : undefined);
          if (why) whyOf.set(f, why);
          at(ev.t, () => { if (why) noteWhy(f, why); callout(f, f !== ev.text && f.endsWith(" fell") ? "hurt" : "", f !== ev.text && f.endsWith(" fell") ? FELL_MS : undefined); coreLine(); });
          break;
        }
        case "rule": {
          at(ev.t, () => tactics.fire(ev));
          const text = ruleCallout(ev);
          if (ev.row >= 0) rowFires[ev.row] = (rowFires[ev.row] ?? 0) + 1;   // Cut 14 §4
          // Cut 14 §4: a chore (`pick up`) coalesces on its line with a count; a row's callout repeats after 4 s
          if (text && ev.row === -2) at(ev.t, () => choreCallout(text));
          else if (text) at(ev.t, () => {
            const now = performance.now();
            // Cut 18 §2: a telegraph over the fight has the line — the row's callout goes to the ticker (`rule`, shown in the fight frame)
            if (text !== lastRuleText || now - lastRuleAt > 4000) { if (ev.row >= 0 && frame === "fight" && now - lastCoreAt < CORE_LINE_MS) ruleLine(text); else callout(text); }
            lastRuleText = text; lastRuleAt = now; lastRuleIsRow = ev.row >= 0;
          });
          if (ev.row >= 0) at(ev.t, () => cue("rule"));   // Cut 10 §4: a player row, never a chore or a trait
          if (ev.row >= 0) { const row = ev.row; at(ev.t, () => lightRow(row)); }   // gfx round 2: the desktop's rules column lights the rule that acted
          if (ev.row >= 0) { const home = ev.verb.v === "return"; at(ev.t, () => { if (home !== walkingHome) { walkingHome = home; if (hudSnap) paintStake(hudSnap); } }); }   // Cut 19 §2
          break;
        }
        case "drain": {   // Cut 25 §3 (core): the stretch's word, once (a floor)
          if (!drainTicks.length || drainTicks[drainTicks.length - 1] < ev.t) { drainTicks.push(ev.t); if (drainTicks.length > 64) drainTicks.shift(); }
          const w = drainWord(ev.cause), key = `${w}@${s.depth}`;
          at(ev.t, () => { el.dataset.drain = String(ev.t); if (!drainsShown.has(key)) { drainsShown.add(key); callout(drainSaid(w, ev.cause), "hurt", HURT_MS * 2); } });
          break;
        }
        case "hurt": if (bossIds.has(ev.id)) { const id = ev.id, hp = ev.hp; at(ev.t, () => { if (bossHud?.id === id) { bossHud.hp = hp; paintBoss(); } }); }
          if (ev.id === heroId) {
            // Cut 25 §3: a drain (no blow: hunger, poison, a curse) — no number per bite; its word once a floor (`starving`)
            const dw = drainOf(ev) ?? (drainEvs.has(ev) ? drainOn : null);
            if (dw) { if (ev.hp <= 0) killBlow = dw; if (!drainTicks.length || drainTicks[drainTicks.length - 1] < ev.t) { drainTicks.push(ev.t); if (drainTicks.length > 64) drainTicks.shift(); }
              at(ev.t, () => { el.dataset.drain = String(ev.t); });   // tooling: the last drain the picture reached
              const key = `${dw}@${s.depth}`; at(ev.t, () => { hud.hp = ev.hp; paintHud(); if (!drainsShown.has(key)) { drainsShown.add(key); callout(drainSaid(dw, ev.cause), "hurt", HURT_MS * 2); } }); break; }
            if (ev.hp <= 0 && ev.dmg > 0) killBlow = hurtText(ev.dmg, ev.cause);   // the blow the 0-hp frame names (`paintHud`)
            at(ev.t, () => { hud.hp = ev.hp; paintHud(); if (ev.dmg > 0) { callout(hurtText(ev.dmg, ev.cause), "hurt", HURT_MS); cue("hit", { dmg: ev.dmg, kind: ev.cause }); } });
          } else if (ev.dmg > 0 && !allies.has(ev.id)) { const kind = kinds.get(ev.id) ?? victims.get(ev.id), dmg = ev.dmg; at(ev.t, () => cue("strike", { dmg, kind })); }   // juice pass 2: the blow lands, by the foe's family
          break;
        // the kill gets its own line (cohort 5: "−3 goblin" was still up after the goblin had dissolved)
        case "die": {
          if (ev.id === heroId) at(ev.t, () => tactics.end());
          if (ev.id === heroId) { heroCause = ev.cause; break; }
          // Cut 10 §3: a companion's death (the snapshot's ally flag) is the core's callout to name; the kill line is for hostiles
          if (allies.has(ev.id)) {
            fell = { t: ev.t, name: names.get(ev.id) ?? "", kind: (kinds.get(ev.id) ?? "").replace(/_/g, " ") };
            // under the card or inside a skip the core's `Ashar fell` line is never watched: the fall waits for the ticker
            const f = fell; at(ev.t, () => { if (skipping || cardUp) petLine(f.name ? /* copy:callout */ `${oneWord(f.kind)} ${f.name} fell` : /* copy:callout */ `summoned ${oneWord(f.kind)} fell`); });
            break;
          }
          // Cut 15 §4: a boss's kill is a beat — the frame holds on it with `WARLORD DOWN` (its own line, not `slain`)
          if (bossIds.has(ev.id) && !allies.has(ev.id)) { const id = ev.id; at(ev.t, () => { if (bossHud?.id === id) { bossHud = null; paintBoss(); } }); { const kb = beatAt(ev.t, bossDown(kinds.get(ev.id) ?? victims.get(ev.id) ?? "boss"), false, true); if (!folding) killBeats.push(kb); } at(ev.t, () => cue("boss_down")); break; }
          const v = victims.get(ev.id), vk = kinds.get(ev.id) ?? v; if (v) at(ev.t, () => { callout(/* copy:callout */ `${v} slain`, "kill", HURT_MS); cue("slay", { kind: vk }); });
          break;
        }
        case "telegraph": if (ev.what === "rallies") rallyBy = kinds.get(ev.id) ?? rallyBy; at(ev.t, () => cue("telegraph")); break;
        // Cut 10 §3: a theft names its amount when the engine sends one (`stolen $16`)
        // QA 1a2a4a9 (P: `12/38` → `6/16`, "nothing in the run said why"): the core's `max_hp` event moves the HUD's max at its tick
        // (the `hunger −1 max` callout comes as a callout of its own)
        case "max_hp": if (ev.id === heroId) {
          const m = ev.max, dw = ev.delta < 0 ? drainOf(ev) ?? (drainEvs.has(ev) ? drainOn : null) : null, key = dw ? `${dw}@${s.depth}` : "";
          if (dw && (!drainTicks.length || drainTicks[drainTicks.length - 1] < ev.t)) { drainTicks.push(ev.t); if (drainTicks.length > 64) drainTicks.shift(); }
          // QA 308f045 (qaAC: `36/36` → `22/22` → `12/24` with no cause): the shrine's price names itself (`shrine −7 max`)
          const priced = ev.cause === "shrine" && ev.delta < 0 ? /* copy:callout */ `shrine −${-ev.delta} max` : null;
          const said = dw ? drainSaid(dw, ev.cause) : dw, bite = -ev.delta;   // blind c4705f9 (B): `starving · no light`
          at(ev.t, () => { hud.maxHp = m; if (dw) maxLoss.set(dw, (maxLoss.get(dw) ?? 0) + bite); paintHud(); if (priced) callout(priced, "hurt", HURT_MS * 2); else if (said && !drainsShown.has(key)) { drainsShown.add(key); callout(said, "hurt", HURT_MS * 2); } });
        } break;
        case "steal": lastStealT = ev.t; if (ev.amount !== undefined && ev.amount > 0) { const n = ev.amount; stolenGold += n; at(ev.t, () => callout(/* copy:callout */ `stolen $${n}`, "hurt", FELL_MS)); } break;
        case "descend": {
          descends.push(ev.t);   // Cut 18 §1
          at(ev.t, () => { chore = null; });   // Cut 25 §3: a floor's chore count starts over
          floors.set(ev.depth, { ...floors.get(ev.depth), biome: ev.biome });
          const rooms = s.depth === ev.depth ? s.rooms : undefined;   // Cut 7 §4: `D3 · 4 rooms` when the snapshot counts them
          // Cut 26 §2: on a forked lineage the stairs into a band name the lane taken (`D5 · fens`)
          const up = floors.get(ev.depth - 1)?.biome ?? (s.depth === ev.depth - 1 ? s.biome : undefined);
          const lane = ev.biome && up && up !== ev.biome && ev.biome !== "warrens" && seenForks(app.lineage).length ? ev.biome : undefined;
          at(ev.t, () => { hideBeat(); hud.depth = ev.depth; paintHud(); if (hudSnap) paintStake(hudSnap); ambient(lane ? /* copy:callout */ `D${ev.depth} · ${lane}` : rooms ? /* copy:callout */ `D${ev.depth} · ${rooms} rooms` : `D${ev.depth}`, true); });
          // run-clear (the owner, 2026-10-02: a record no longer ends a run — it is a beat and a checkpoint, and he carries on): the
          // first floor past the lineage's record this run stamps a gilt `NEW BEST D5` over the floor's arrival — the boss stamps' look,
          // but it never holds the frame or takes a fight's beat (he walks on; the card's `new best` badge says it again)
          // (c305-core 2ec1cb0: the core says it — the `new best · D5` callout, a beat and a checkpoint; the stamp is drawn from it below)
          break;
        }
        case "fact": {
          learned.push(ev.fact);
          // Cut 26 §2: the first time a hero stands on a fork floor's two stairs (the fact `fork D5`) the watch says so — `TWO STAIRS`,
          // once a floor (a core callout of the same words stands for it)
          if (/^fork\b/i.test(ev.fact)) { const key = `stairs@${s.depth}`; if (!refused.has(key)) { refused.add(key); beatAt(ev.t, /* copy:callout */ "TWO STAIRS"); } }
          // Cut 6 §5: the counter learned mid-fight (the boss's first telegraph) names itself: `boss · counter: attack boss`
          const m = /^boss:([a-z_]+):counter(?:=|$)/.exec(ev.fact);
          if (m) at(ev.t, () => { app.engine.lineage().then((L) => { counters = L.counters ? namedCounters(L) : counters; }).catch(() => { /* keep */ }).finally(() => {
            const named = counters.find((c) => c.boss === m[1])?.text; if (named && !disposed) showCounter(/* copy:callout */ `counter: ${named}`, BOSS_BANNER_MS);
          }); });
          break;
        }
        case "pickup": {
          // Cut 25 §3: `pick up ×N` counts the tries since the last thing picked up (it grew across a floor: `×441`)
          if (ev.id === heroId) at(ev.t, () => { if (chore?.text === CHORE_CALLOUT.pick_up) chore = null; });
          if (ev.id === heroId) { lastPickT = ev.t; lastPickItem = /^gold\b/.test(ev.item) ? lastPickItem : ev.item; }
          const gold = /^gold\b\D*(\d+)/.exec(ev.item);   // Cut 7 §4: `+$47` on a gold pickup (`gold (47)` core, `gold 47` fake)
          // QA 92eb880 (M: "FOUND gold `$1 · $2 ×3 · …` = $27 but `$38 carried`"): gold is the ledger line's, not a find — FOUND lists items;
          // the ambient line is a gain (`+$3`, N: "a bare `$3` line")
          if (ev.id === heroId && !gold) found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item });
          if (gold) at(ev.t, () => ambient(`+$${gold[1]}`));
          break;
        }
        case "note":
          notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text });
          if (breakBeat(ev.t, ev.text)) break;
          // Cut 19 §1: the cage's opening is its own beat (`took mail`, vaultFrom) when the batch's snapshot carries the choice
          if (/^The cage opens\b/.test(ev.text) && s.vault_choice?.items.length) break;
          if (BEAT_RE.test(ev.text)) beatAt(ev.t, noteText(ev.text));   // Cut 13 §4: the situations cut in like fights
          else if (PET_NOTE_RE.test(ev.text)) { const line = petNoteLine(ev.text); at(ev.t, () => petLine(line)); }   // QA e75ec29: `Skog joins.` · `Skog: level 3.`
          break;
        case "exit": {
          at(ev.t, () => tactics.end());
          exit = ev.tier; exitLine = ev.line ?? exitLine; exitTrace = ev.trace ?? ev.line?.trace ?? exitTrace;
          if (exitLine && tollShort && exitLine.start_short === undefined) exitLine = { ...exitLine, start: 1, start_short: true };   // `· from D1 · toll short` on the line
          markEnd(runId, ev.t);   // Cut 11 §2: the run log's last replayable tick
          // Cut 7 §4: the last ENDING_TICKS play at 1×; the core's `ending` marker counts only when the exit follows it closely
          // (Cut 10 §1: a foreseen death the hero survived held the map at 1× for minutes)
          // Cut 14: the marker never holds 1× longer than twice ENDING_TICKS before the exit
          endingFrom = Math.min(endingFrom, ev.t - ENDING_TICKS, endingCue >= ev.t - 100 ? Math.max(endingCue, ev.t - 2 * ENDING_TICKS) : Infinity);
          const tier = ev.tier; at(ev.t, () => audio.cue(tier === "bank" ? "exit_bank" : tier === "return" ? "exit_return" : "exit_death"));           // Cut 10 §4
          // Cut 14 §3: the bank and the return are beats — the fight frame on the stairs, the sum as the callout, before the sheet
          // QA 0c6e126 (qaZ: `RETURNED $115` over a run the report's tiles counted `1 DRIVEN`): the beat's word is the exit line's own lead
          // (`driven` · `stalled` · `lost thread`), `RETURNED` only for a return
          const lead = ev.line?.driven ? "driven" : /^(stalled|lost thread|driven)\b/.exec(ev.line?.text ?? "")?.[1];
          // QA 308f045 (qaAC: `DRIVEN $0` at 19/36 over `carry $186` — "nothing says a drive-off also loses the carry"): an end that kept
          // less than it carried says what it lost (`DRIVEN $0 · −$186`)
          const lostC = ev.line ? Math.max(0, ev.line.carried - ev.line.kept) : 0;
          const oathW = exitOath ? ` · ${exitOath}` : ""; exitOath = "";
          const why = ev.line?.reason;
          // (a death's reason rides its last frame, no beat: the death screen comes as it did)
          if (tier === "death") { if (why) at(ev.t, () => showWhy(why, SCENE_MS)); }
          else beatAt(ev.t, (tier === "bank" ? /* copy:callout */ `COLLECTED $${ev.loot_kept}` : lead ? /* copy:callout */ `${(lead === "driven" ? /* copy:callout */ "repelled" : lead).toUpperCase()} $${ev.loot_kept}${lostC > 0 && ev.line!.kept <= 0 && !oathW ? ` · −$${lostC}` : ""}` : /* copy:callout */ `RETURNED $${ev.loot_kept}`) + oathW, true, false, false, why);
          break;
        }
        // Cut 28b (owner: "it's not clear what oaths do"): the sworn oath's fate is a beat as it happens — `OATH KEPT`, or `OATH BROKEN · R2 return`
        // (the row that used the tool it forbade); a miss (the run ended short of it) is the exit line's, not a beat
        // (decided by the run's end — a depth oath kept by a bank — it rides the exit's own beat: `COLLECTED $120 · OATH KEPT`)
        case "oath": if (ev.kept || ev.cause) { if (evs.some((x) => x.k === "exit" && x.t === ev.t)) exitOath = ev.kept ? /* copy:callout */ "OATH KEPT" : /* copy:callout */ "OATH BROKEN"; else beatAt(ev.t, oathBeat(ev, app.rules.rows), false, true); } break;
        case "ending": endingCue = ev.t; break;                                                                                       // Cut 7 §4: the core's marker (see `exit`)
        case "tame": if (ev.ok) { tamedIds.push(ev.id); kinds.set(ev.id, ev.kind); allies.add(ev.id); victims.delete(ev.id); const id = ev.id; at(ev.t, () => petLine(/* copy:callout */ `tamed ${compLabel(id).replace(" · ", " ").replace(/_/g, " ")}`)); } break;
        case "ally": if (ev.state === "lost") lostIds.push(ev.id); else {
          // QA 308f045 (qaAC: the death's `ally hound fell` with no hound anywhere before): a summon (an unnamed ally the scroll called)
          // says so as it arrives (`hound summoned`)
          const fresh = !allies.has(ev.id), kind = kinds.get(ev.id), named = names.has(ev.id);
          if (fresh && kind && !named && ev.state === "freed" && /spectral/.test(kind)) at(ev.t, () => petLine(/* copy:callout */ `${oneWord(kind)} summoned`));
          allies.add(ev.id); victims.delete(ev.id); } break;
        case "spawn": note_(ev.e); if (ev.e.tags?.includes("boss")) bossIds.add(ev.e.id); break;
        case "level": for (const v of verbsAt(ev.class, ev.level)) learned.push(`verb:${v}`); at(ev.t, () => { callout(`${ev.class} L${ev.level}`); if (!folding) audio.cue("level"); }); break;
        case "rank": at(ev.t, () => callout(`★${ev.rank}`)); break;
        case "rest": restS = ev.seconds; break;
        case "bones":
          if (ev.heir === s.run.heir) bonesLeft = ev.items;                                        // this heir's kit, left on death
          else { bonesFound.push(/* copy:callout */ `D${s.depth} · ${items(ev.items)}`); at(ev.t, () => callout(/* copy:callout */ `heir ${ev.heir} · ${ev.items}`)); }
          break;
        default: break;
      }
    }
    // Cut 25 §3: a foe in view at the batch's end ends the drain stretch (the core opens the next with its own `drain`)
    if (drainOn && s.entities.some((e) => !e.ally && e.id !== heroId && e.hp > 0 && !!s.visible[e.y * s.w + e.x])) drainOn = null;
    return exit;
  }
  /** Cut 10 §4: a combat cue, only while the fight is watched (the fight frame up, or the clock at 1×). */
  function cue(name: CueName, opts?: CueOpts): void { if (!folding && (frame === "fight" || speed <= FAST_NEAR)) audio.cue(name, opts); }
  // Cut 5 §5 / Cut 7 §4: what holds auto at 1× — a scene (a room with SCENE_FOES awake hostiles), the hero's hp moving
  const hostile = (e: { ally?: boolean; kind: string; tags: string[] }): boolean => !e.ally && e.kind !== "bones" && e.kind !== "captive" && !e.tags.includes("captive") && !e.tags.includes("ally");
  /** Cut 7 §4: the hero's room and its awake hostiles; without `room` in the wire, the hostiles in view stand in (one "room"). */
  function roomOf(s: Snapshot): { id: number; hostiles: number } {
    return s.room ?? { id: -1, hostiles: s.entities.filter((e) => hostile(e) && !e.remembered && s.visible[e.y * s.w + e.x]).length };
  }
  function sceneFrom(s: Snapshot, evs: Ev[] = []): void {
    const r = roomOf(s);
    const open = r.hostiles >= SCENE_FOES || (scene !== null && r.id === scene && r.hostiles > 0);
    if (open) { scene = r.id; sceneUntil = Infinity; const last = scenes[scenes.length - 1]; if (!last || last.until !== Infinity) scenes.push({ from: s.turn, until: Infinity }); }
    else if (scene !== null) { scene = null; sceneUntil = s.turn + AUTO_TAIL; const last = scenes[scenes.length - 1]; if (last && last.until === Infinity) last.until = sceneUntil; }
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
    // Cut 15 §4: a hostile near (or a boss in view) is a fight's own time; the rest of the frame's hold is a chore stretch
    const foeNear = within(FOE_NEAR) || boss, lastFoe = foeSpans[foeSpans.length - 1];
    if (foeNear) { if (!lastFoe || lastFoe.until !== Infinity) foeSpans.push({ from: s.turn - BATCH_FIGHTS, until: Infinity }); }
    else if (lastFoe && lastFoe.until === Infinity) lastFoe.until = s.turn + AUTO_TAIL;
    const blow = evs.find((e) => e.k === "attack" || e.k === "telegraph" || (e.k === "hurt" && e.id === s.hero.id));
    if (blow) lastBlow = Math.max(lastBlow, ...evs.filter((e) => e.k === "attack" || e.k === "telegraph" || (e.k === "hurt" && e.id === s.hero.id)).map((e) => e.t));
    // Cut 10 §1: `fights` frames the blows — an adjacent hostile, a blow in the last BLOW_TICKS, a boss in view — not the whole scene
    // (a room's approach ran at 1× and cost the gate; the wire has no awake flag, so a sleeper two tiles off would hold it too);
    // `fast` keeps Cut 8A's scene / adjacent / boss
    // Cut 18 §1: `fast` frames what `fights` frames (a scene's approach at 4× kept `fast` in the frame 150 ticks where `fights` showed 55)
    void sceneOpen;
    const on = within(1) || s.turn - lastBlow <= BLOW_TICKS || boss;
    if (on) {
      // Cut 14 §6: a live viewer still inside the last span (its tail playing) merges the two as before; a picture that is behind
      // (paused, hidden, or `fast` where the frame follows the spans) keeps the closed span for its replay and the new fight gets its own
      if (!fightOn && (viewerTick() >= fightUntil || paused || hidden || mode === "fast")) {
        closeSpan();
        fightFrom = Math.min(s.turn, blow?.t ?? s.turn); fightShow = false;
      }
      fightUntil = Infinity;
    }
    else if (fightOn) { fightUntil = s.turn + (mode === "fights" ? FIGHT_TAIL : AUTO_TAIL); closeSpan(); }
    fightOn = on;
  }
  /** Cut 14 §6: the closed fight span (`fightFrom`..`fightUntil`) kept for a viewer that has not reached it; once per span. */
  function closeSpan(): void {
    if (!Number.isFinite(fightUntil) || fightFrom === Infinity || viewerTick() >= fightUntil) return;
    const last = spans[spans.length - 1];
    if (last && last.from === fightFrom) last.until = fightUntil; else spans.push({ from: fightFrom, until: fightUntil });
  }
  /** The next queued floor load, taken. */
  function takeLoad(): { snap: Snapshot; rest: Ev[]; at?: number } | null { return loads.shift() ?? null; }
  /** Cut 14 §6: load every queued floor into the viewer (the last one is the picture; the ones between were never watched). */
  function drainLoads(): void { if (!viewer) return; for (let p = takeLoad(); p; p = takeLoad()) loadFloor(p); }
  /** A queued floor into the viewer. QA (qaj under load: `COLLECTED $N` at t816, then the picture at t791): a load resets the clock to
   *  its snapshot's turn — a picture already past it (the walk-out plays on past the frontier) is put back where it was, never rewound. */
  function loadFloor(p: { snap: Snapshot; rest: Ev[]; at?: number }): void {
    if (!viewer) return;
    const v = viewer.tick?.() ?? viewerTick();
    viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); floorFrom = p.at ?? p.snap.turn;
    if (Number.isFinite(v) && v > p.snap.turn + 1) seekTo(v);
  }
  /** Cut 14 §6: the viewer lands on the frontier (live) — every queued floor loaded, the clock at the engine's tick (the ending's
   *  start at most when the run is over, so the walk-out plays). */
  const focusHero=():void=>{goLive();canvas.scrollIntoView({block:"nearest"});};
  window.addEventListener("riddle:focus-hero",focusHero);
  function goLive(): void {
    releaseBeat();   // Cut 18 §1: landing live lets a held beat go
    // blind 5331f40: live (or the ending's start) at most — short of a boss's kill not yet shown, whose floor stays loaded
    const want = held ? Math.max(viewerTick(), endingFrom) : Math.max(viewerTick(), engineTick);
    while (loads.length && !killAhead(loads[0].at ?? Infinity)) loadFloor(takeLoad()!);
    const t = killStop(want);
    seekTo(t); release(t); letGo(t); applyFrame(); applySpeed();
  }
  /** Land the viewer's clock on tick t (both directions; the placeholder viewer's wall clock too). */
  /** blind 5331f40: the first boss kill the playhead has not shown, on the viewer's floor (or a floor still queued), in (v, t]. */
  function killAhead(t: number): Beat | undefined {
    const v = viewerTick(), now = performance.now();
    for (const b of killBeats) {
      if (b.shown || b.from <= v || b.from > t || b.from < floorFrom) continue;
      // blind 7f7fc2b: a kill the picture has not played within KILL_WAIT_MS of the first jump held short of it lapses (never a hang)
      const w = killWait.get(b);
      if (w !== undefined && now - w > KILL_WAIT_MS) { b.shown = true; killsLapsed++; el.dataset.killsLapsed = String(killsLapsed); continue; }
      return b;
    }
    return undefined;
  }
  /** blind 5331f40: a forward jump to t stops KILL_LEAD ticks short of a boss's kill the picture has yet to show (never back). */
  function killStop(t: number): number {
    const k = killAhead(t); if (!k) return t;
    if (!killWait.has(k)) killWait.set(k, performance.now());
    return Math.max(viewerTick(), k.from - KILL_LEAD);
  }
  /** blind 7f7fc2b: every kill up to t given up (take control: the picture lands on the hero now). */
  function killsPassed(t: number): void { for (const b of killBeats) if (!b.shown && b.from <= t) b.shown = true; }
  /** A queued floor is loaded only when no unshown kill comes before its stairs (the kill's floor would go with the load). */
  function loadNext(): void { if (loads.length && killAhead(loads[0].at ?? Infinity)) return; const p = takeLoad(); if (p) loadFloor(p); }
  function seekTo(t: number): void {
    t = killStop(t);
    const fv = viewer as (Viewer & { seek?: (t: number) => void }) | null;
    if (fv?.seek) fv.seek(t); else fv?.skipToEvent();
    fbTick = t; fbAt = performance.now();
  }
  /** Cut 10 §1: under the card the engine has found the next fight ahead of the viewer's clock (the cut seeks the viewer there). */
  const fightAhead = (v: number): boolean => cardUp && ((fightShow && fightFrom < Infinity && v < fightFrom) || spans.length > 0);   // Cut 14 §6: a kept span is a fight ahead too
  function applyFrame(): void {
    const v = viewerTick();
    // Cut 10 §1: under the card only a fight the probe chose is wanted (the viewer's clock holds there; the cut seeks to it)
    while (spans.length && spans[0].until <= v) spans.shift();   // Cut 14 §6: spans the playhead has passed
    const next = spans[0];   // Cut 14 §6: the nearest kept span — the one the playhead is in, or the next to cut to from under the card
    const inSpan = !!next && v >= next.from;
    // Cut 15 §4: a beat's own span holds the frame too (a later fight re-opening `fightFrom` no longer cuts a boss's kill short)
    const inBeat = !!beat && v >= beat.from && v < beat.until;
    const wantFight = mode === "fights" && cardUp && !mapHeld() ? (fightShow && fightFrom < Infinity && v < fightUntil) || !!next : (v >= fightFrom && v < fightUntil) || inSpan || inBeat;
    let want: FrameName = wantFight ? "fight" : "map";
    // QA on 3d71c33 (a floor card between `drink ✗ no item` and `−2 blade`): in `fights` the frame never gives way to the card while
    // the fight is still on at the playhead — a hostile near, a blow's hold, a beat (`choreAt` false)
    if (mode === "fights" && frame === "fight" && want === "map" && !cardUp && !mapHeld() && app.slowdowns && v < endingFrom && !choreAt(v)) want = "fight";
    // Cut 10 §1: a fight waits for the card's minimum (the clock holds at 0 meanwhile; the seek below lands on the first frame)
    // Cut 15 §4: a beat waits on the card no longer than CARD_BEAT_MS (its frame is the beat's: a boss's kill, a situation, the bank)
    const beatDue = !!beat && !beat.shown && beat.from >= fightFrom && beat.from < fightUntil;
    cardWait = mode === "fights" && wantFight && frame === "map" && cardUp && performance.now() < cardSince + (beatDue ? Math.min(cardMin, CARD_BEAT_MS) : cardMin);
    if (cardWait) want = "map";
    if (beatHeld() && frame === "fight") { want = "fight"; cardWait = false; }   // Cut 18 §1: a held beat keeps its frame
    el.dataset.span = `${fightFrom}:${fightUntil}:${fightOn ? 1 : 0}:${fightShow ? 1 : 0}:${engineTick}`;   // dev: the fight span the frame follows
    el.dataset.spans = spans.map((sp) => `${sp.from}-${sp.until}`).join(",");   // dev: Cut 14 §6, the kept spans
    if (want === frame) { paintCard(want); return; }
    if (want === "fight") {
      fights++; el.dataset.fights = String(fights); mapHold = false; cardLive = false; skipEarly = false;
      // the viewer lands on the fight's first frame: back if its clock overshot under the card, forward if the engine ran ahead
      // (the travel's queued HUD updates are released under the card, so the ticker is clean at the cut)
      // Cut 14 §6: a kept span ahead is cut to at its own start; a span the playhead is in plays on from where it is
      const target = next ? (v < next.from ? next.from : null) : fightFrom;
      if (mode === "fights" && target !== null && Math.abs(v - target) > 1) { seekTo(target); release(target); }
    }
    paintCard(want);
    frame = want; (viewer as FrameViewer | null)?.setFrame?.(want);
    el.dataset.frame = want;
    if (want === "fight") showBeat();   // Cut 13 §4: the beat's line at the cut
  }
  /** Cut 10 §1: the interstitial is up while `fights` shows the map, unless a tap holds the map, the vault sheet is up, or the
   *  run's ending plays. Its line is the ambient one: `D3 · 4 rooms · $47`. */
  function paintCard(want: FrameName): void {
    const up = mode === "fights" && want === "map" && !mapHeld() && !cageWaits() && !done && !exitTier && viewerTick() < endingFrom && !beatHeld() && !folding;
    if (up !== cardUp) {
      cardUp = up; el.dataset.card = up ? "1" : "0";
      // a card per floor: the full minimum when the floor is new, a beat between fights on the same floor — drawn only on a new floor
      // (QA on 3d71c33: `D6 · 16 rooms · a cage` 4× on one floor); the one between fights is a ghost
      if (up) { cardSince = cardShownAt = performance.now(); cardGhost = hud.depth === cardDepth; cardMin = cardGhost ? CARD_BEAT_MS : CARD_MS; cardDepth = hud.depth; ticker.classList.remove("show"); }
      card.hidden = !up || cardGhost; el.dataset.ghost = up && cardGhost ? "1" : "0";
    }
    if (up) paintCardText();
  }
  /** The card's line — one position on screen with the HUD: the HUD's floor (its rooms and situation as the engine reported that
   *  floor) and the stake's loot, never the engine's latest snapshot (the DVR queue and the viewer's clock keep the HUD behind it;
   *  the HUD's own paints repaint the card). Cut 12 §4: the floor's one situation, one word after its article
   *  (`D4 · 9 rooms · a nest`); the gold line when there is none. */
  function paintCardText(): void {
    const d = hud.depth, f = floors.get(d), rooms = f?.rooms, twist = f?.twist;
    // the floor changed under the card (the travel drained a load): this is the new floor's card, drawn
    if (cardUp && d !== cardDepth) { cardDepth = d; if (cardGhost) { cardGhost = false; card.hidden = false; el.dataset.ghost = "0"; } }
    // Cut 16 §3: the biome's first floor in the run names it in the rooms' place (`D5 · the Burrows · a shrine`)
    const prev = floors.get(d - 1)?.biome, first = [...floors.keys()].every((k) => k >= d);
    const title = f?.biome && (prev ? prev !== f.biome : first) ? BIOME_TITLE[f.biome] : undefined;
    // Cut 21 §1: a run sent from a waystone names it on its first floor (`D9 · the Fens · waystone`)
    const way = first && d > 1 && d === (app.lineage.start ?? 1);
    // Cut 27 §1: the passage the waystone start was paid (`+$84 passage`: the skipped floors' gold, when the set clears them ≥ 95 %)
    const passage = way ? hudSnap?.run.passage ?? snap?.run.passage ?? 0 : 0;
    const text = way ? /* copy:callout */ `D${d}${title ? ` · ${title}` : ""} · waystone${passage > 0 ? ` · +$${passage} passage` : ""}`
      // QA 912e135 (qaX: `D2 · 16 rooms · $16` beside the header's `$16` — "which $"): the card's gold is the carry, named as the strip names it
      : /* copy:callout */ `D${d}${title ? ` · ${title}` : rooms ? ` · ${rooms} rooms` : ""} · ${twist ? withArticle(twist) : `carry $${hudSnap ? carriedOf(hudSnap) : 0}`}`;
    if (text !== cardText) { cardText = text; replace(card, text); }
  }
  /** Cut 20 §3: `fights` on the first floors — no card, the map at EARLY_TRAVEL, every fight at EARLY_FIGHT. */
  function earlyFloor(): boolean { return mode === "fights" && hud.depth <= EARLY_DEPTH && !overridden && !skipEarly; }
  /** A tap on the card or the first floors: the map is watched (no card). */
  function mapHeld(): boolean { return mapHold || earlyFloor(); }
  function fightRate(): number { return earlyFloor() ? EARLY_FIGHT : FIGHT_RATE[mode]; }
  function holdMap(): void { if (!cardUp) return; mapHold = true; cardWait = false; paintCard("map"); applySpeed(); }
  /** Cut 15 §4: the card has been up CARD_MAX_MS and the travel found no fight — the live map at the flat rate until the next fight. */
  function cardExpired(): void { mapHold = true; cardLive = true; cardWait = false; paintCard("map"); goLive(); }
  function near(t: number): void { slowUntil = Math.max(slowUntil, t + AUTO_TAIL); if (nearTicks[nearTicks.length - 1] !== t) nearTicks.push(t); }
  /** Cut 12 §6: after a skip lands on tick t, the near / scene hold from the span skipped over is let go (the landing decides). */
  function letGo(t: number): void {
    slowUntil = Math.min(slowUntil, t); sceneUntil = Math.min(sceneUntil, t);
    while (nearTicks.length && nearTicks[0] < t) nearTicks.shift();
    while (scenes.length && scenes[0].until <= t) scenes.shift();
    const open = scenes[scenes.length - 1]; if (open && open.until === Infinity && open.from < t) open.until = t;   // the next snapshot re-opens a live scene
  }
  /** Cut 14 §6: is the playhead at v inside a near hold (AUTO_TAIL after a near tick) or a scene's span? */
  function heldAt(v: number, withScenes = true): boolean {
    while (nearTicks.length && nearTicks[0] + AUTO_TAIL <= v) nearTicks.shift();
    while (scenes.length && scenes[0].until <= v) scenes.shift();
    return (nearTicks.length > 0 && nearTicks[0] <= v) || (withScenes && scenes.length > 0 && scenes[0].from <= v);   // Cut 18 §1: `fast`'s map slows for a blow, not a room's sleepers
  }
  /** Cut 15 §4: is the playhead at v in a chore stretch — no hostile near, no beat, no near hold (the fight frame runs the flat rate)? */
  function choreAt(v: number): boolean {
    while (foeSpans.length > 1 && foeSpans[0].until <= v) foeSpans.shift();
    if (foeSpans.some((f) => f.from <= v && v < f.until)) return false;
    if (beat && v >= beat.from && v < beat.until) return false;
    return !heldAt(v);
  }
  /** Cut 12 §6: a skip that found the run's end lands the viewer at the ending (its last ENDING_TICKS still play at 1×) —
   *  a fight that held the exit used to leave the press with nothing visible until the clock got there on its own. */
  function toEnding(): void { const t = killStop(Math.max(viewerTick(), endingFrom)); if (t > viewerTick()) { release(t); seekTo(t); } applyFrame(); applySpeed(); }   // blind 5331f40: short of an unshown kill
  let handGoing = false;
  function letHandGo(): void {
    if (handGoing || !snap) return;
    handGoing = true; snap = { ...snap, manual: false, awaiting: false };
    Promise.resolve().then(() => app.engine.takeControl?.(false)).catch(() => { /* the run is over: nothing to let go */ }).finally(() => { handGoing = false; });
  }
  function manualPump(): void {
    paintWatchStatus();
    if (cardUp) cardExpired();
    applyFrame(); applySpeed();
    let now = viewerTick();
    // (the picture never runs past the frontier: the next action's events play from where he stands)
    if (now > engineTick) { seekTo(engineTick); now = viewerTick(); }
    // blind 7f7fc2b (B: control taken over a picture stuck behind the world — arrows and `descend` moved nothing visible): in hand
    // the picture is the hero now — every queued floor in, any kill behind the frontier given up, the clock on the frontier
    else if (engineTick - now > 2) { releaseBeat(); killsPassed(engineTick); drainLoads(); seekTo(engineTick); release(engineTick); now = viewerTick(); }
    el.dataset.tick = String(now); el.dataset.engineTick = String(engineTick);
    release(now); paintScrub(now);
    if (loads.length && viewerIdle()) loadFloor(takeLoad()!);
    worldT = engineTick; lastPumpMs = performance.now();
    if (inflight || (snap!.awaiting && !manualKick)) return;
    manualKick = false; inflight = true;
    app.engine.step(BATCH).then((r) => { inflight = false; if (!disposed && !done) handle(r); })
      .catch((e) => { inflight = false; console.warn("step failed", e); });
  }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    ctl.paint(r.run_over || held || exitTier ? null : s);
    wide.setPresence?.(s, r.run_over);
    engineTick = s.turn;
    floors.set(s.depth, { rooms: s.rooms ?? floors.get(s.depth)?.rooms, twist: s.floor_twist ?? floors.get(s.depth)?.twist, biome: s.biome });
    for (const e of s.entities) note_(e);
    sceneFrom(s, r.events);
    if (s.hero.hp < lastHp) near(s.turn);   // hp lost by any means; a rest's +1 per turn is a dead stretch, a drink is a `use` event
    lastHp = s.hero.hp;
    if (s.alert > lastAlert) { const n = s.alert; at(s.turn, () => ambient(/* copy:callout */ `alert ${n}/${ALERT_TOP}`)); }   // Cut 7 §4
    lastAlert = s.alert;
    const exit = absorb(r.events, s);
    if (folding) folding.tally.absorb(r.events, s);   // Cut 27 §1: the stretch's state changes, chips on its line
    noteProgress(r.events, s);   // (after `absorb`: the batch's bosses are known)
    noteCalm(r.calm);
    snap = s;
    // Cut 14 §6: the stake and the max hp land at the viewer's clock like the rest of the HUD (the picture may be behind the world)
    at(s.turn, () => { hud.maxHp = s.hero.max_hp; paintHud(); paintStake(s); bossFrom(s); }); bossSighted(s);
    // Cut 14 §6: the frontier's dot beats per batch (the animation retriggered by a reflow — at most every PULSE_MS: the travel
    // chain lands a batch a millisecond, and a reflow each would starve the page)
    pulses++; el.dataset.pulses = String(pulses);
    if (performance.now() - lastPulseMs >= PULSE_MS) { lastPulseMs = performance.now(); scrubDot.classList.remove("beat"); void scrubDot.offsetWidth; scrubDot.classList.add("beat"); }
    // the card names the HUD's floor (QA on e0f87e7: "`D1 · 16 rooms · $18` while the HUD reads `32/40 D2`"; on 56f2a1d the
    // reverse, the card ahead of the HUD): the HUD's paints repaint it; here only a floor's rooms / situation first reported
    if (cardUp && !frozen()) paintCardText();
    if (mode === "fast" && app.slowdowns) probeFight(r);   // Cut 18 §1: `fast` costs its fights ahead of the picture, as the card does
    if (!exit) vaultFrom(s);   // no choice on a run that just ended
    if (r.exit_pending) pendingExit = r.exit_pending;
    // Cut 7 §4: the exit batch waits (pump) until the viewer is ENDING_TICKS from the exit, then plays at 1×
    // Cut 14 §6: only the ending itself is held (a catch-up batch may reach back far past it); the rest feeds now
    if (exit) { held = { evs: r.events.filter((e) => e.t >= endingFrom), tier: exit, snap: s }; el.dataset.ending = "1"; if (vaultClose) { vaultClose(); vaultClose = null; } feed(r.events.filter((e) => e.t < endingFrom), s); return; }
    feed(r.events, s);
    if (performance.now() - lastPersist > PERSIST_MS) { lastPersist = performance.now(); app.persist(); }
  }
  function feed(evs: Ev[], s: Snapshot): void {
    if (frozen()) { frozenFeed.push({ evs, s }); return; }   // Cut 14 §6: the picture is frozen; fed on the thaw
    // a batch may hold several floors when the world is catching up: the snapshot is the last one's, so the split is at the last
    // descend (the floors between are not watched — the wire carries one snapshot per step)
    let di = -1; for (let i = evs.length - 1; i >= 0; i--) if (evs[i].k === "descend") { di = i; break; }
    // Cut 14 §6: floors the viewer has not reached queue; a later batch appends to the newest floor (or opens the next)
    if (loads.length) {
      const last = loads[loads.length - 1];
      if (di >= 0) { last.rest.push(...evs.slice(0, di + 1)); loads.push({ snap: s, rest: evs.slice(di + 1), at: evs[di].t }); }
      else last.rest.push(...evs);
      return;
    }
    // entities that appear inside this batch must exist before their events apply (they are not tweened in;
    // the first event they own places them)
    (viewer as Viewer & { preload?: (x: Snapshot) => void } | null)?.preload?.(s);
    if (viewer && di >= 0) { viewer.apply(evs.slice(0, di + 1)); loads.push({ snap: s, rest: evs.slice(di + 1), at: evs[di].t }); }
    else { viewer?.apply(evs); if (viewer?.sync) { const v = viewer; at(s.turn, () => v.sync!(s)); } } // Cut 4 §3: remembered foes
  }
  /** Cut 14 §6: the world's rate — the viewer's decisions read off the engine's own tick; ≥ 1× while the run is live, 0 once it is over. */
  function worldRate(): number {
    if (done || held || exitTier) return 0;
    if (cageWaits()) return 0;                             // Cut 15 §5 / Cut 19 §1: the world waits on the cage beat and the override sheet
    if (!frozen() && beatHeld() && heldBeat && !heldBeat.exit) return Math.max(0, speed);   // Cut 18 §1: a held beat is the watch's own pacing — the world keeps the picture's pace (no catch-up owed after it)
    if (fightOn && app.slowdowns) return mode === "fights" && engineTick >= slowUntil && !(beat && engineTick < beat.until) && !(foeSpans.length && foeSpans[foeSpans.length - 1].until > engineTick) ? RATE.fights : fightRate();   // Cut 15 §4: a chore stretch at the flat rate
    if (mode === "fights") return RATE.fights;
    if (mode === "one") return hidden ? RATE.fast : 1;   // Cut 25 §3: a hidden tab's world runs on (offline is never slower)
    return !app.slowdowns || overridden || (engineTick >= slowUntil && engineTick >= sceneUntil) ? RATE.fast : FAST_NEAR;
  }
  /** Cut 14 §6: the strip — the playhead at the viewer's share of the run so far, the dot at the frontier. */
  function paintScrub(v: number): void {
    const span = Math.max(1, engineTick - startTick);
    scrubHead.style.left = `${Math.max(0, Math.min(100, ((v - startTick) / span) * 100)).toFixed(1)}%`;
    el.dataset.frontier = String(engineTick); el.dataset.world = String(Math.floor(worldT));
  }
  /** Blind c4705f9 (A, B: `stolen fire potion` over `Carried $2468`; the summary chips over the boss bar): the docked fold line sits under
   *  the HUD's actual bottom (the stake line, an alert and the boss bar make it taller than the CSS's fixed 70 px), never over it. */
  function dockBelowHud(): void {
    if (foldLine.hidden || !foldLine.classList.contains("docked")) return;
    const hud = el.querySelector<HTMLElement>(".hud.top"), host = foldLine.offsetParent as HTMLElement | null;
    if (!hud || !host) return;
    const below = Math.round(hud.getBoundingClientRect().bottom - host.getBoundingClientRect().top + 4);
    const top = `${Math.max(70, below)}px`;
    if (foldLine.style.top !== top) foldLine.style.top = top;
  }
  /** Cut 28 §4: the DOM over the canvas — the HUD's line, the docked fold line's head and chips, the banner, the ticker, the reason —
   *  handed to the renderer as keep-out rects (canvas CSS px) every KEEP_MS, so no pixel callout, caption or name plate lands on them. */
  let keepAt = 0;
  function paintKeepOut(force = false): void {
    dockBelowHud();
    const now = performance.now();
    if (!viewer?.setKeepOut || (!force && now - keepAt < KEEP_MS)) return;
    keepAt = now;
    const c = canvas.getBoundingClientRect();
    const els: Element[] = [...el.querySelectorAll(".hud.top"), combatLog, tactics.el];
    if (!foldLine.hidden && foldLine.classList.contains("docked")) els.push(foldLine);
    for (const x of [banner, ticker, whyLine, whyTip]) if (x.classList.contains("show")) els.push(x);
    const rects: { x: number; y: number; w: number; h: number }[] = [];
    for (const x of els) {
      if ((x as HTMLElement).hidden) continue;
      const r = x.getBoundingClientRect();
      const appearing = x === foldLine || x === ticker || x === banner;
      if (r.width < 1 || r.height < 1 || (!appearing && getComputedStyle(x).opacity === "0")) continue;
      // Reserve the dock animation's full 12px travel, including its final resting position.
      const dockTravel = x === foldLine ? 12 : 0;
      rects.push({ x: r.left - c.left - 2, y: r.top - c.top - 2 - dockTravel, w: r.width + 4, h: r.height + 4 + 2 * dockTravel });
    }
    viewer.setKeepOut(rects);
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
    // blind 7f7fc2b (B: `Hand control · awaits order` with the run over — arrows and `descend` inert): once the run is over or the exit
    // flow holds, the panel goes and a hand still on the hero is let go (the core never waits on an order nobody can give)
    if (held || exitTier) { if (!ctl.el.hidden) ctl.paint(null); if (snap.manual) letHandGo(); }
    // take control: no fold, no jump, no travel, no world clock — the engine steps when the hero acts (the core waits for him)
    if (snap.manual && !held && !exitTier) { manualPump(); return; }
    paintWatchStatus();
    paintGun();
    paintKeepOut();
    // Cut 27 §1: while a stretch folds the world stands under its line (the fold steps it); the line holds its minimum, then docks
    if (folding) { paintScrub(viewerTick()); if (!folding.stepping && performance.now() >= folding.holdUntil) endFold(); return; }
    if (!inflight && !frozen() && foldDue()) { void foldRun(); return; }
    if (vaultClose && !document.querySelector(".vault-choice")) { vaultClose = null; if (cage) cage.done = true; }   // dismissed by backdrop / Escape: the engine's grace decides
    if (vaultClose && !frozen() && performance.now() - vaultAt > VAULT_WAIT_MS) vaultClose();    // Cut 15 §5: the wait is over; the engine's grace and preference proceed (⏸ stops the wait)
    const still = frozen();   // Cut 14 §6: a frozen picture — no cut, no release, no floor load; the world below steps on
    // Cut 18 §1: the hold is over — a beat that waited for it takes the line (the frame is still on the fight)
    if (!still && !beatHeld() && el.dataset.held === "1") { el.dataset.held = "0"; const nb = beatNext.shift(); if (nb && !nb.shown && frame === "fight") { nb.shown = true; beat = nb; holdBeat(nb); } }
    if (!still && cardUp && !cardWait && !mapHeld() && !held && performance.now() - cardShownAt > CARD_MAX_MS && !fightAhead(viewerTick())) cardExpired();   // Cut 15 §4
    if (!still) applyFrame();
    applySpeed();
    if (!still) flushPets();
    if (!still && ticker.classList.contains("beat") && ticker.classList.contains("show") && performance.now() - tickerAt > BEAT_MAX_MS) hideBeat();
    let now = viewerTick();
    el.dataset.tick = String(now);            // dev: tools sample the cadence off the DOM
    if (!still && unstuck(now)) now = viewerTick();
    // Cut 14 §6: the world clock — wall time at the world's rate; the live playhead pulls it along (it is never behind the picture)
    const nowMs = performance.now(); const dtMs = nowMs - lastPumpMs; lastPumpMs = nowMs;
    // in `fights` the card and the travel are the world's skip (the engine is already ahead of the picture): the clock only
    // counts while the picture is frozen; in `fast` it always does
    // Cut 15 §4: …and the world's clock is the frontier the travel reached (a stale clock behind it left a picture frozen on the card
    // with the world standing until the clock walked back up to the frontier — the shorter card made that common)
    if (mode === "fights" && !paused && !hidden) worldT = engineTick;
    else worldT += (dtMs / 1000) * 10 * worldRate();
    worldT = Math.max(worldT, now);
    paintScrub(now);
    if (!still) release(cardWait ? Math.min(now, fightFrom) : now);   // Cut 10 §1: a fight waiting on the card keeps its HUD at the first frame
    if (still) { /* the picture is frozen: straight to the world's step below */ }
    else if (goLiveOwed && !inflight && (worldT - engineTick <= LEAD_FAST || held)) { goLiveOwed = false; goLive(); now = viewerTick(); }   // Cut 14 §6: back from a hidden tab, the world caught up (Cut 18: before the floor queue — goLive drains it)
    else if (loads.length) {
      // Cut 10 §1: under the card the floor changes at once (nothing is watched); otherwise the viewer drains first (Cut 14 §6:
      // never while the picture is frozen; the world below steps on regardless)
      // Cut 25 §3 (AM: "one watch frame at D4 was an empty black board with only `A dropped purse in the dust.`"): the descend's fade had
      // gone to black and the new floor's own beat (the purse, on its first tick) held the load behind it — a beat on the floor the load
      // opens never holds it (only one on the floor being left: the kill's frame)
      const beatHere = beatHeld() && !(heldBeat && loads[0].at !== undefined && heldBeat.from >= loads[0].at);
      if ((cardUp || viewerIdle()) && !beatHere) { loadFloor(takeLoad()!); }   // Cut 18 §1: not under a held beat
    }
    else if (held) {
      // Cut 7 §4: the clock runs on (8× through dead air) to the ending, then the exit batch plays and the exit flow waits for it
      // Cut 10 §1: under the card the viewer jumps to the ending (the walk-out plays at 1×; the card hides there)
      // blind 5331f40: …short of a boss's kill not yet shown (the climax plays, then the walk-out)
      { const to = killStop(endingFrom); if (cardUp && now < to) { release(to); seekTo(to); now = to; applyFrame(); applySpeed(); } }
      if (now < endingFrom) return;
      const hb = held; held = null; feed(hb.evs, hb.snap);
      exitTier = hb.tier; exitAt = performance.now() + EXIT_GRACE_MS;
      endControls();
      return;
    }
    else if (exitTier) {
      if (!(viewerIdle() || performance.now() > exitAt)) return;
      // Cut 28 (a loaded machine: the 1× walk-out outlasted the grace and `COLLECTED $N` took the line at t791 of an exit at t800): the grace
      // seeks the picture to the frontier first — the exit's line is released with the picture on the exit, never before it
      if (!viewerIdle() && viewerTick() < engineTick) { const to = killStop(engineTick); seekTo(to); applyFrame(); if (to < engineTick) return; }
      if (beatHeld() && heldBeat && killBeats.includes(heldBeat)) return;   // blind 5331f40: the kill's beat holds its wall time before the exit flow
      release(Infinity);
      nextGem();   // QA 92eb880: the walk-out has played — now the gem says what comes next
      if (performance.now() < exitBeatUntil) return;   // Cut 14 §3: `COLLECTED $N` has its SCENE_MS first
      // Cut 2 §1: `rest 12m` for a beat, then the exit flow continues. Cut 14 §4: the banner sits low (`.rest`), under the frame's
      // callout line, and the ticker yields to it (rater S: `rest 20m` over `OGRE WINDS UP` on the death frame)
      // QA 92eb880 (N: "`rest 20m` … after `0/36` reads as the hero resting instead of dying"): after a death the rest is the next heir's (`♟2 · rest 20m`)
      // QA 778fa1b (qaU: `♟2 · rest 20m` on the last frame before the verdict announced the next heir before the death was read): after a
      // death no rest beat — the death screen comes first, the camp's rest line says the rest. qaV (`REST 20M` drawn over `RETURNED $96`
      // after a ▶▶|): the rest beat takes the frame alone — the exit's callout goes with it
      if (restS !== undefined && !restUntil && exitTier !== "death") { restUntil = performance.now() + (mode === "fast" ? REST_BEAT_FAST_MS : REST_BEAT_MS); ticker.classList.remove("show"); ticker.classList.add("cut"); hideBeat(); showBanner(/* copy:callout */ `rest ${spanOf(restS)}`, REST_BEAT_MS, "rest"); return; }
      if (performance.now() < restUntil) return;
      void finish(exitTier); return;
    }
    if (inflight || held || exitTier) return;
    if (cageWaits()) return;   // Cut 15 §5 / Cut 19 §1: the world waits on the cage (the chips answer at once: the engine is idle)
    // Cut 10 §1: travel under the card — the engine steps flat out (chained calls) until it finds the next fight or the exit
    if (travelling()) { inflight = true; travel(); return; }
    // Cut 10 §1: through a shown fight's tail the engine waits at its close, so the next fight opens under the card (and is costed
    // there) instead of merging into this one at 1× — Cut 14 §6: not while the picture is frozen (the world goes on)
    const playing = !paused && !hidden && speed > 0;
    // Cut 14 §6: in `fights` the card is the world's own skip — the travel above moved the engine to the next fight; while the card
    // waits its minimum the engine is ahead of the picture, and the clock below has nothing to add
    if (mode === "fights" && (cardUp || cardWait) && !paused && !hidden) return;
    if (playing && mode === "fights" && !mapHeld() && !vaultClose && !fightOn && Number.isFinite(fightUntil) && now < fightUntil && !(beat && now < beat.until)) return;   // Cut 13 §4: a beat plays on at 1×
    // blind 77030eb (B: Normal stood ~2.5 min with no new line): Normal never plays NEWS_MAX_MS of wall time without news (a move, a floor,
    // a jump) — the stretch is jumped, and the jump says `skipped ahead`
    if (mode === "one") {
      const key = `${progressBefore(viewerTick())}|${snap?.depth ?? 0}|${jumps}`;
      const wall = performance.now();
      if (key !== newsKey || !playing || beatHeld() || viewerTick() >= endingFrom) { newsKey = key; newsAt = wall; }
      else if (wall - newsAt > NEWS_MAX_MS && !cageWaits() && !vaultClose && !(snap && cageNear(snap))) { newsAt = wall; void deadJump(); return; }
    }
    // QA ad71e72: a long dead stretch jumps (the engine steps ahead to the next move; the picture lands just before it)
    if (playing && el.dataset.dead === "1" && deadFrom > 0 && performance.now() - Math.max(deadFrom, lastJumpAt) > DEAD_JUMP_MS && !cageWaits() && !vaultClose && !beatHeld() && !(snap && cageNear(snap))) { void deadJump(); return; }
    // Cut 14 §6: the engine's target — the world clock, or the playing viewer's lead, whichever is further; a world far behind its
    // clock (a paused or hidden picture) is caught up in CATCHUP_MAX-tick steps
    // Cut 18 §1: the ramp's lead (0.2 s of picture); `fast` keeps LEAD_PROBE ahead so a fight is costed before the picture meets it
    // watch-normal-news (Normal stood ~10.7 s from its first frame on a loaded machine): `one`'s dead stretch travels at up to ONE_MAX,
    // but at LEAD the engine fed it only BATCH ticks a pump — at a few frames a second the picture crawled at ~1.4× behind the frontier
    // (the landing never outruns `engineTick`) and the jump came late; a dead stretch keeps LEAD_FAST ahead so the travel is the picture's
    const lead = Math.max(speed >= AUTO_FAST ? Math.max(LEAD_FAST, Math.ceil(speed * 2)) : LEAD, mode === "fast" && app.slowdowns && !overridden ? LEAD_PROBE : 0,
      mode === "one" && el.dataset.dead === "1" ? LEAD_FAST : 0);
    const want = Math.max(worldT, playing ? now + lead : -Infinity);
    if (engineTick >= want) return;
    const gap = worldT - engineTick;
    let n = gap > BATCH_FAST ? Math.min(CATCHUP_MAX, Math.ceil(gap)) : Math.min(CATCHUP_MAX, Math.max(speed >= AUTO_FAST ? Math.max(BATCH_FAST, Math.ceil(speed * 0.6)) : BATCH, Math.ceil(want - engineTick)));
    // QA 92eb880 (N: twice `A cage: three inside, one to take.` with no sheet, in `fast`): a batch longer than the cage's 50-tick grace
    // stepped over the whole choice (the sheet reads the batch's last snapshot) — near an unopened cage the engine steps in short batches
    const short = !!snap && cageNear(snap);
    if (short) n = Math.min(n, CAGE_BATCH);
    // Cut 24 §1: a dead stretch's longer lead is walked in `fights`'s own batches — a fight's close is met within one (the engine waits there)
    if (el.dataset.dead === "1" && mode === "fights") n = Math.min(n, BATCH_FIGHTS);
    inflight = true;
    // Cut 20 §3: near a cage the batches are short (each snapshot is checked for the choice) — they chain up to the pump's target in
    // one pass instead of one short batch a pump (the picture in `fast` stood at the frontier ~2 s waiting for the engine to walk past a cage)
    const target = Math.min(want, engineTick + CATCHUP_MAX);
    const chain = (k: number, left: number): void => {
      app.engine.step(k).then((r) => {
        inflight = false;
        if (!disposed && !done) handle(r);
        if (skipQueued) { skipQueued = false; void skipToEvent(); return; }
        // Cut 24 §1: a dead stretch in `fights` walks to its lead in its own short batches, chained (a fight's close is met within one)
        const go = left > 0 && !disposed && !done && !r.run_over && !r.snapshot.vault_choice && !held && !exitTier && !cageWaits() && engineTick < target && !!snap;
        if (go && short && cageNear(snap!)) { inflight = true; chain(CAGE_BATCH, left - 1); }
        else if (go && !short && fightOn && el.dataset.dead === "1" && mode === "fights" && !cageNear(snap!)) { inflight = true; chain(BATCH_FIGHTS, left - 1); }
      }).catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; endControls(); });
    };
    chain(n, CAGE_CHAIN);
  }
  /** QA ad71e72: a dead stretch's jump — the engine steps ahead in big batches until the frontier holds the next thing worth a frame
   *  (a move, the stairs, a beat, the end, a cage) or DEAD_JUMP_WALL_MS passes; the picture lands JUMP_LAND ticks before it. */
  async function deadJump(): Promise<void> {
    inflight = true; lastJumpAt = performance.now();
    const v0 = viewerTick(), t0 = performance.now();
    const ahead = (): number => Math.min(progressAfter(v0), descends.find((t) => t > v0) ?? Infinity, beat && !beat.shown && beat.from > v0 ? beat.from : Infinity,
      ...beatNext.filter((x) => !x.shown && x.from > v0).map((x) => x.from), held ? endingFrom : Infinity);
    try {
      while (!disposed && !done && !held && !exitTier && ahead() === Infinity && performance.now() - t0 < DEAD_JUMP_WALL_MS) {
        const r = await app.engine.step(SKIP_END_BATCH);
        if (disposed || done) break;
        handle(r);
        if (r.run_over || r.snapshot.vault_choice || (cage && !cage.done) || cageNear(r.snapshot)) break;
      }
    } catch (e) { console.warn("jump failed", e); }
    inflight = false;
    if (disposed || done) return;
    const t = killStop(Math.max(viewerTick(), Math.min(engineTick, ahead() - JUMP_LAND)));
    if (t > viewerTick() + 1) {
      release(t); seekTo(t); letGo(t);
      // blind 5331f40 (A): in `one` the picture never ramps past ONE_MAX — a jump over a dead stretch says so
      if (mode === "one") { jumpsSaid++; el.dataset.jumpsSaid = String(jumpsSaid); callout(/* copy:callout */ "skipped ahead", "beat"); }
    }
    jumps++; el.dataset.jumps = String(jumps); lastJumpAt = performance.now();
    applyFrame(); applySpeed();
  }
  /** QA ad71e72: the stuck-picture watchdog — the viewer's clock unmoved for STUCK_MS with the run live and nothing deliberate holding
   *  it: the card is let go and the picture lands live (the pump then steps the engine on as ever). True when it acted. */
  function unstuck(v: number): boolean {
    const now = performance.now();
    if (v !== stuckTick) { stuckTick = v; stuckAt = now; return false; }
    if (now - stuckAt < STUCK_MS) return false;
    if (folding || held || exitTier || done || inflight || cageWaits() || vaultClose || beatHeld() || foldDue()) { stuckAt = now; return false; }
    stuckAt = now; unsticks++; el.dataset.unstuck = String(unsticks);
    console.warn("watch: picture stuck", { v, engine: engineTick, card: cardUp, cardWait, frame, span: el.dataset.span, spans: el.dataset.spans });
    cardWait = false; mapHold = true; cardLive = true; releaseBeat();
    if (fightUntil > engineTick && !fightOn) fightUntil = engineTick;   // a fight's tail the engine waits on is over
    goLive();
    return true;
  }
  /** Cut 10 §1: is the engine free to run ahead under the card — `fights`, the card up and not held, no fight found yet, no exit. */
  function travelling(): boolean {
    return mode === "fights" && cardUp && !cardWait && !mapHeld() && !frozen() && !held && !exitTier && !done && !disposed && !fightAhead(viewerTick()) && !foldDue();
  }
  function travel(): void {
    app.engine.step(BATCH_FIGHTS).then((r) => {
      if (disposed || done) { inflight = false; return; }
      handle(r);
      probeFight(r);
      drainLoads();   // under the card the floor changes at once
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
      // QA ad71e72 (rater A: 40 s–2 min of a still picture at D8 / D10 / D18): a fight already worth showing is shown now, not at its
      // close — a boss standoff (the Warlord's shield wall, the Lich's endless dead) ran thousands of ticks under the card first
      if (!held) { if (!fightShow && fightFrom < Infinity && (probe.hurt >= SHOW_HURT || probe.low || probe.boss || probe.ally || probe.steal)) fightShow = true; return; }
    }
    if (!probe) return;
    const show = fightShow || held || probe.hurt >= SHOW_HURT || probe.low || probe.boss || probe.ally || probe.steal;
    probe = null;
    if (show && fightFrom < Infinity) fightShow = true;
    else if (beat && viewerTick() < beat.until && fightFrom <= beat.from) { fightUntil = beat.until; closeSpan(); }   // Cut 13 §4: the beat keeps its frame
    else if (mode === "fast" && viewerTick() >= fightFrom) { /* Cut 18 §1: the picture is already in it — it plays out */ }
    else {
      const i = spans.findIndex((sp) => sp.from === fightFrom); if (i >= 0) spans.splice(i, 1);   // Cut 14 §6: a dropped fight is no span
      if (mode === "fast") forget(fightFrom, fightUntil);
      fightFrom = Infinity; fightUntil = -Infinity;
    }
  }
  /** Cut 18 §1: `fast` drops a fight `fights` would drop (it cost nothing) — its near ticks and scenes go with it, so the picture
   *  passes it at the dead stretch's rate (`fast` watched every fight at 4×, `fights` only the ones that cost: `fast` was slower). */
  function forget(from: number, until: number): void {
    if (!Number.isFinite(from)) return;
    const lo = from - BLOW_TICKS, hi = Number.isFinite(until) ? until : engineTick;
    for (let i = nearTicks.length - 1; i >= 0; i--) if (nearTicks[i] >= lo && nearTicks[i] <= hi) nearTicks.splice(i, 1);
    for (let i = scenes.length - 1; i >= 0; i--) if (scenes[i].until !== Infinity && scenes[i].from >= lo && scenes[i].until <= hi + AUTO_TAIL) scenes.splice(i, 1);
  }
  /** The rate the clock should run at right now. `fights`: 16× under the card (8× when a tap holds the map, 0 while a fight
   *  waits for the card's minimum); `fast`: 8× / 1× by what is near (flat 8× while bailing). Cut 7 §4: 1× through a scene and
   *  through the run's last ENDING_TICKS (bailing too: the walk-out is still the end). */
  /** Cut 13 §1: the guard has fired (`keeps $0 · stalling`), the player has not bailed, and it has held STALL_FLAT_TICKS (a flag that
   *  flickers for a sample mid-fight — QA 23ed91f, L — is no stall to hurry through). */
  let stallSince = -1;
  function stalling(): boolean {
    if (!snap?.stake?.stalling || overridden) { stallSince = -1; return false; }
    if (stallSince < 0) stallSince = snap.turn;
    return snap.turn - stallSince >= STALL_FLAT_TICKS;
  }
  /** Cut 24 §1: the batch's moves (an hp change, a kill, a pickup, a descent) as progress ticks. With a boss in view (the batch's
   *  snapshot) only his hp, his fall, a descent or the exit move it — the Warlord's summons hitting the hero and dying under his
   *  shield wall is a dead stretch (the coordinator's read of AL's four minutes: the hero's bar moved, the boss's never did). */
  // blind 77030eb (B: ~2.5 min at the start of D7 at Normal with no new line, the badge flicking 1×/4×; a ~3 min iron-golem stalemate,
  // hp see-sawing 20–40): a move is one the player can see — the hero's, or a foe's once in his sight (blows out of sight broke every
  // dead stretch before its jump) — and a blow moves the fight only when it takes a body to a new low since the last kill, find,
  // drink or descent (a see-saw of hits and heals, or blows on a foe that heals them back, is a stalemate: a dead stretch)
  const sighted = new Set<number>(), lows = new Map<number, number>();
  function noteProgress(evs: Ev[], s: Snapshot): void {
    const boss = s.entities.some((e) => e.tags.includes("boss") && !e.ally && e.hp > 0 && !!s.visible[e.y * s.w + e.x]);
    const heroId = s.hero.id;
    for (const e of s.entities) if (s.visible[e.y * s.w + e.x]) sighted.add(e.id);
    const seen = (id: number): boolean => id === heroId || sighted.has(id);
    for (const e of evs) {
      let moved: boolean;
      if (boss) moved = e.k === "descend" || e.k === "exit" || ((e.k === "hurt" || e.k === "die") && bossIds.has(e.id)) || (e.k === "attack" && e.hit && e.dmg > 0 && bossIds.has(e.dst));
      else if (e.k === "die") moved = seen(e.id);
      else if (e.k === "hurt") {
        const low = lows.get(e.id) ?? Infinity;
        moved = !isDrain(e) && e.dmg > 0 && seen(e.id) && e.hp < low;
        if (e.hp < low) lows.set(e.id, e.hp);
      } else moved = (e.k === "pickup" && pickupOfNote(e)) || e.k === "descend" || e.k === "use" || e.k === "exit";
      if (!boss && (e.k === "die" || e.k === "descend" || e.k === "use" || (e.k === "pickup" && moved))) lows.clear();   // a decisive move: the lows start over
      if (moved && (!progress.length || e.t > progress[progress.length - 1])) progress.push(e.t);
    }
    while (progress.length > 64 && progress[1] < viewerTick() - 4000) progress.shift();   // the playhead never seeks back that far
  }
  /** Cut 28 §3: a pickup of note — no gold, a kind this run has not picked up before (a `pick up ×N` chain of coins and repeats is a
   *  chore, no move). Judged once per event (the batch may be re-noted). */
  const pickedKinds = new Set<string>(), pickupNote = new WeakMap<Ev, boolean>();
  function pickupOfNote(e: Extract<Ev, { k: "pickup" }>): boolean {
    const was = pickupNote.get(e); if (was !== undefined) return was;
    const kind = e.item.replace(/\s*[×x]\d+$/, "").trim();
    const note = !/^gold\b/.test(kind) && !pickedKinds.has(kind);
    pickedKinds.add(kind); pickupNote.set(e, note);
    return note;
  }
  /** Cut 28 §3 (core): the steps' calm stretches — run ticks with no decision and no threat (a pick-up chain, an empty corridor); the
   *  playhead inside one of CALM_MIN ticks or more plays at the travel rate at once, in every mode (1× included), landing at its end.
   *  Absent on the wire (an older core): the dead-stretch rule alone (DEAD_TICKS past the last move, pickups of note only). */
  const calm: [number, number][] = [];
  function noteCalm(c?: [number, number][]): void {
    for (const [a, b] of c ?? []) {
      const last = calm[calm.length - 1];
      if (last && a <= last[1] + 1) last[1] = Math.max(last[1], b); else calm.push([a, b]);
    }
    while (calm.length > 64 && calm[0][1] < viewerTick() - 4000) calm.shift();
  }
  const calmAt = (v: number): [number, number] | undefined => {
    // a beat (a cage's, an exit's, a situation's) is never inside a calm stretch: the stretch ends before it; a cage the world waits on
    // or a held beat is no calm at all
    if (cage && !cage.done || vaultClose || beatHeld()) return undefined;
    const r = calm.find(([a, b]) => v >= a && v < b && b - a >= CALM_MIN);
    if (!r) return undefined;
    const bs = [beat, ...beatNext].filter((x): x is Beat => !!x && !x.shown && x.from >= v).map((x) => x.from);
    const end = Math.min(r[1], ...bs.map((f) => f - 1), endingFrom - 1);
    return end - v >= 1 ? [r[0], end] : undefined;
  };
  /** The last progress tick at or before v (the run's start when none). */
  function progressBefore(v: number): number {
    for (let i = progress.length - 1; i >= 0; i--) if (progress[i] <= v) return progress[i];
    return progress.length ? progress[0] : v;
  }
  const progressAfter = (v: number): number => progress.find((t) => t > v) ?? Infinity;
  /** Cut 24 §1: is the playhead DEAD_TICKS past the last move (and not on a boss's kill or break, the card, the ending)? */
  function deadAt(v: number): boolean {
    // Cut 25 §3: a drain since the last move makes the stretch dead at once (no DEAD_TICKS wait: 10 s at the plain 1×) — the bites are its only news
    const p = progressBefore(v);
    if (v - p < (mode === "one" ? DEAD_TICKS_ONE : DEAD_TICKS) && !calmAt(v) && !drainTicks.some((t) => t > p && t <= v)) return false;
    if (beat?.hold && v >= beat.from && v < beat.until) return false;
    return !(mode === "fights" && (cardUp || cardWait));
  }
  /** Cut 24 §1: a dead stretch's rate — the travel's, ramping as `fast`'s dead stretch does, landing on the next move DEAD_LAND_MS out. */
  function deadStretch(was: number, v: number): number {
    deadSince = was < 0 ? performance.now() : was;
    const base = earlyFloor() ? EARLY_TRAVEL : mode === "one" ? ONE_MAX : RATE[mode], age = performance.now() - deadSince;
    // `fights` tops out at DEAD_MAX_FIGHTS (a stretch the core cuts at 60 actions is ~1 s there; its engine lead stays short of a cage);
    // blind 5331f40 (A: "even on Speed Normal the badge jumped 1× … 128× … 16×"): `one` never ramps past ONE_MAX — a long dead
    // stretch is jumped instead (DEAD_JUMP_MS), and the jump is announced (`skipped ahead`)
    let r = age < DEAD_RAMP_MS ? base : Math.min(mode === "fights" ? DEAD_MAX_FIGHTS : mode === "one" ? ONE_MAX : FAST_MAX, base * 2 ** (1 + Math.floor((age - DEAD_RAMP_MS) / DEAD_STEP_MS)));
    // lands DEAD_LAND_MS before the next move, the stairs, the ending; never past the frontier (the engine is stepped to the lead)
    // (and before a beat not yet shown — a cage's, a situation's: at 128× the playhead jumped a cage's whole beat and its tap with it)
    const d = descends.find((t) => t >= v), b = beat && !beat.shown && beat.from >= v ? beat.from : Infinity;
    const cm = calmAt(v);
    const next = Math.min(cm && v - progressBefore(v) < DEAD_TICKS ? cm[1] + 1 : Infinity, progressAfter(v), d ?? Infinity, b, ...beatNext.filter((x) => !x.shown && x.from >= v).map((x) => x.from), endingFrom >= v ? endingFrom : Infinity, engineTick);
    if (Number.isFinite(next)) r = Math.min(r, ((next - v) * 100) / DEAD_LAND_MS);
    return Math.max(base, r);   // never under the travel's rate: the engine is stepped to the playing picture's lead (a 0 at the frontier stood both)
  }
  function rate(): number {
    const was = deadSince;
    deadSince = -1;
    let r = baseRate(was);
    // Cut 18 §1: a held beat's picture eases to its stop (the span's end, or short of the stairs) over the hold's wall time — never
    // faster than the mode would play it, never past the stop; the exit's beat plays the walk-out as before
    if (r > 0 && app.slowdowns && beatHeld() && heldBeat && !heldBeat.exit) {
      const v = viewerTick(), stop = beatStop(), left = Math.max(100, beatHoldUntil - performance.now());
      r = v >= stop ? 0 : Math.min(r, Math.max(0.25, ((stop - v) * 100) / left));
      el.dataset.hold = `${heldBeat.from}-${heldBeat.until}:${stop}`;   // dev: the held beat's span and its stop
      deadSince = -1;
    } else if (el.dataset.hold) delete el.dataset.hold;   // no beat held: tools read the mode's own rate
    return r;
  }
  /** The rate before a held beat's easing (`was`: the dead stretch's start carried over, `fast`). */
  function baseRate(was: number): number {
    if (paused || hidden || goLiveOwed) return 0;   // Cut 14 §6: the picture freezes (and stays put until the seek to live); the world (worldRate) goes on
    if (vaultClose) return viewerTick() < engineTick ? 1 : 0;   // Cut 5 §4 / Cut 15 §5: 1× up to the frontier, where the world waits for the tap
    const v = viewerTick();
    if (v >= endingFrom) return mode === "fast" ? FAST_ENDING : 1;   // the walk-out is seen whatever the toggle (Cut 20 §3: at 2× in `fast`)
    // QA 23ed91f (L: `fast` ran a 4-minute summoner stall at the fight's slow clock): a stall is watched at the mode's flat rate — no
    // fight, near or scene hold
    const flat = !app.slowdowns || stalling();   // Cut 14: `slowdowns` off — no fight, near or scene hold; the mode's flat rate
    // Cut 24 §1: nothing has moved for DEAD_TICKS — the travel's rate, whatever the frame
    const dead = deadAt(v);
    if ((el.dataset.dead === "1") !== dead) { el.dataset.dead = dead ? "1" : "0"; deadFrom = dead ? performance.now() : -1; }
    el.dataset.progress = String(progressBefore(v));
    if (dead) return deadStretch(was, v);
    if (mode === "one") return 1;   // Cut 25 §3: the plain 1× — every live frame at 1× (the dead stretch above still travels)
    if (frame === "fight" && !flat && beat?.hold && v >= beat.from && v < beat.until) return 1;   // Cut 15 §4: a boss's kill holds SCENE_MS
    if (frame === "fight" && !flat) return choreAt(v) ? (earlyFloor() ? earlyTravel(was, v) : RATE[mode]) : fightRate();   // Cut 18 §1: `fast`'s chore stretch at its flat rate too   // Cut 8A: a fight is watched slow (Cut 14: 2× in `fights`, 4× in `fast`); Cut 15 §4: a chore stretch in `fights` at the flat rate
    if (mode === "fights") return cardWait || cardUp ? 0 : earlyFloor() && !flat ? earlyTravel(was, v) : mapHold ? (cardLive ? RATE.fights : AUTO_FAST) : RATE.fights;   // the clock holds under the card: the cut seeks
    if (!app.slowdowns) return RATE.fast;   // Cut 14: `slowdowns` off — the mode's flat rate, no ramp either
    if (flat || overridden || !heldAt(v, false)) { deadSince = was < 0 ? performance.now() : was; return deadRate(v); }
    return FAST_NEAR;
  }
  /** QA e75ec29 (R: `fights` spent 28–40 s on D1 — a D1 is ~1 400 ticks, 17 s at a flat 8×): the first floors' travel starts at
   *  EARLY_TRAVEL and ramps like `fast`'s dead stretch (to EARLY_TRAVEL_MAX), landing at EARLY_TRAVEL before the next hold. */
  function earlyTravel(was: number, v: number): number { deadSince = was < 0 ? performance.now() : was; return Math.max(EARLY_TRAVEL, deadRate(v, EARLY_TRAVEL, EARLY_TRAVEL_MAX)); }   // never under 8×: in `fights` the engine is stepped to the playing picture's lead
  /** Cut 18 §1: `fast` is never slower than `fights` — a dead stretch plays at the flat 16× for DEAD_RAMP_MS, then doubles every
   *  DEAD_STEP_MS up to FAST_MAX (the card in `fights` skips the travel in ≤ 1 s; 16× through a 600-tick floor took ~4 s: rater Y's
   *  "`fast` mode was slower than `fights`"); never so fast that the next hold the engine has seen comes sooner than DEAD_LAND_MS
   *  away. Cut 14 §6: a picture behind live walks at least CATCHUP_RATE. */
  function deadRate(v: number, base = RATE.fast, max = FAST_MAX): number {
    const age = performance.now() - deadSince;
    let r = age < DEAD_RAMP_MS ? base : Math.min(max, base * 2 ** (1 + Math.floor((age - DEAD_RAMP_MS) / DEAD_STEP_MS)));
    const next = nextHold(v);
    if (Number.isFinite(next)) r = Math.min(r, Math.max(base, ((next - v) * 100) / DEAD_LAND_MS));
    r = Math.min(r, Math.max(0, ((engineTick - v) * 100) / DEAD_LAND_MS));   // never past the frontier (the first batch at boot)
    if (engineTick - v > LEAD_FAST + BATCH_FAST) r = Math.max(r, CATCHUP_RATE);
    return r;
  }
  /** blind 1fb7786 (B: "sped itself between fights, 1× to 107×, then dropped back — jumpy"): a rise past RISE_FREE eases — the clock at
   *  most doubles every RISE_MS from what it shows (a carried dead-stretch age, a catch-up or a ramp never lands 1× → 128× in one frame).
   *  Falls are never held back: the landing (`deadStretch`, `deadRate`) already slows toward the next move, and a hold is a hold. */
  function eased(n: number): number {
    const now = performance.now(), dt = Math.min(RISE_MS, Math.max(0, now - riseAt));
    riseAt = now;
    if (n <= RISE_FREE || n <= speed) return n;
    const cap = Math.max(RISE_FREE, speed) * 2 ** (dt / RISE_MS);
    return n <= cap ? n : Math.round(cap * 10) / 10;
  }
  /** Cut 18 §1: the first tick after v where the picture must slow — a near tick, a scene, a kept fight span, the live fight, a beat,
   *  the ending. */
  function nextHold(v: number): number {
    // the frontier and the next stairs too: the picture never runs past what the engine has fed it (at 128× a floor's end came
    // before the pump saw the viewer idle — the next floor's load then set the clock back)
    let n = Math.min(endingFrom >= v ? endingFrom : Infinity, engineTick);
    const d = descends.find((t) => t >= v); if (d !== undefined) n = Math.min(n, d);
    const near = nearTicks.find((t) => t >= v); if (near !== undefined) n = Math.min(n, near);
    for (const sp of spans) if (sp.from >= v) { n = Math.min(n, sp.from); break; }
    if (Number.isFinite(fightFrom) && fightFrom >= v) n = Math.min(n, fightFrom);
    if (beat && !beat.shown && beat.from >= v) n = Math.min(n, beat.from);
    return n;
  }
  function applySpeed(): void {
    const n = eased(rate());
    if (n === speed) { paintRate(); return; }
    if (!viewer?.tick) viewerTick();          // placeholder clock: bank the ticks run at the old rate first
    speed = n; viewer?.setSpeed(n);
    el.dataset.speed = String(n);
    for (const m of Object.keys(modeBtn) as Mode[]) modeBtn[m].classList.toggle("slowed", m === mode && n <= FAST_NEAR);
    paintRate();
  }
  /** Cut 15 §4: the lit chip's clock as digits (`data-rate`, drawn small by CSS): the viewer's rate, the travel's under the card,
   *  none while the picture is frozen, the world waits (the vault) or the run is over. */
  function paintRate(): void {
    // QA 1a2a4a9 (P: `fast 0.5×` mid-fight while `fights` read 2×): a held beat's eased clock is the beat's, not the mode's — the chip
    // reads the mode's fight rate through it
    const r = paused || hidden || done || exitTier ? 0 : beatHeld() ? fightRate() : speed > 0 ? speed : cardUp || cardWait ? RATE[mode] : 0;
    // (paused or between pictures: the mode's own clock, so the tile never reads blank)
    const shown = `${rateText(r > 0 ? r : RATE[mode])}×`;
    if (speedRate.textContent !== shown) { speedRate.textContent = shown; speedBtn.dataset.rate = shown; }
    speedBtn.classList.toggle("sped", (r > 0 ? r : RATE[mode]) > 1);
    for (const m of Object.keys(modeBtn) as Mode[]) {
      const want = m === mode && r > 0 ? rateText(r) : "";
      if ((modeBtn[m].dataset.rate ?? "") !== want) { if (want) modeBtn[m].dataset.rate = want; else delete modeBtn[m].dataset.rate; }
    }
  }
  function setMode(m: Mode): void {
    if (app.watchMode !== m) { app.watchMode = m; app.persist(); }   // remembered for the next run
    const wasCard = cardUp;
    speedMode.textContent = m === "one" ? /* copy:label */ "Normal" : m === "fast" ? /* copy:label */ "Fast" : /* copy:label */ "Fights only";
    mode = m; freeze(false, hidden); mapHold = false; cardLive = false; el.dataset.mode = m; speedBtn.dataset.mode = m;
    // Cut 14 §6: leaving the card (`fights` → `fast`) lands live — the travel under it was the world's skip, not a replay owed
    if (wasCard && m !== "fights" && !held && !exitTier) goLive();
    for (const k of Object.keys(modeBtn) as Mode[]) modeBtn[k].classList.toggle("on", k === m);
    paintPause(); paintCard(frame); applySpeed();
  }
  function togglePause(): void { freeze(!paused, hidden); paintPause(); applySpeed(); }
  const frozen = (): boolean => paused || hidden;
  /** Cut 14 §6: ⏸ and a hidden tab freeze the picture: the ticker's timer stops (its line and queue resume where they were), and
   *  the thaw feeds the batches the world stepped meanwhile. */
  function freeze(p: boolean, hid: boolean): void {
    const was = frozen(); paused = p; hidden = hid;
    if (!was && frozen()) { frozenAt = performance.now(); clearTimeout(tickerTimer); if (vaultClose) graceHold(); }
    else if (was && !frozen()) {
      const d = performance.now() - frozenAt; tickerAt += d; ambientUntil += d; if (vaultClose) { vaultAt += d; graceRun(); } if (cage) cage.at += d; if (exitBeatUntil > frozenAt) exitBeatUntil += d; if (holdLineUntil > frozenAt) holdLineUntil += d; if (beatHoldUntil > frozenAt) beatHoldUntil += d;   // the cage's wait stood still too
      cardShownAt += d; cardSince += d;   // Cut 15 §4: a frozen card's time does not count
      if (ticker.classList.contains("show") || tickerQueue.length) scheduleTicker();
      for (const x of frozenFeed.splice(0)) feed(x.evs, x.s);
    }
    paintWatchStatus();
  }
  /** QA 1a2a4a9: a cage the skip met — open, its beat not yet shown. */
  const cageMet = (): boolean => !!cage && !cage.done && !cage.shown;
  /** The skip lands on the cage's beat: every floor loaded, the viewer at the beat's tick, the frame cut to it (its line holds there). */
  function landCage(): void {
    drainLoads();
    const b = beat && beat.cage && !beat.shown ? beat : null, t = b ? b.from : viewerTick();
    seekTo(t); release(t); letGo(t); cardSince = -Infinity; applyFrame(); applySpeed();
  }
  function paintPause(): void { pause.setAttribute("aria-label", paused ? /* copy:button */ "Resume watch" : /* copy:button */ "Pause watch"); pause.classList.toggle("on", paused); pause.classList.toggle("pulse", paused); replace(pause, icon(paused ? "play" : "pause"), h("span", { class: "gem-glyph" }, paused ? "▶" : "⏸")); }
  // Cut 27 §1 — solved floors fold: the floors the camp's forecast says the sent set clears ≥ 95 % (`foldFloors`) are stepped flat out
  // under the fold line, never watched; the watch opens at the first floor below the bar. The world stands while the line holds.
  let foldSet = new Set<number>();
  type Fold = { tally: FoldTally; from: number; to: number; t0: number; holdUntil: number; stepping: boolean; skip: boolean; replay: boolean; core?: FoldLine };
  let folding: Fold | null = null;
  let lastFold: { from: number; to: number; core?: FoldLine } | null = null;
  const foldsDone: string[] = [];
  let dockTimer = 0;
  /** The last floor of the folded stretch that opens on `d` (the plan). */
  const planTo = (d: number): number => { let t = d; while (foldSet.has(t + 1)) t++; return t; };
  function paintFold(f: Fold): void {
    const L = f.core;
    if (L) {
      // the core's line (`fold()`): its clear through the stretch, the gold it added, its chips (≤ 3 words each, by kind)
      replace(foldHead, `${L.to > L.from ? `D${L.from}–${L.to}` : `D${L.from}`} · ${Math.round(L.clear * 100)}%${L.gold ? ` · ${L.gold > 0 ? "+" : "−"}$${Math.abs(L.gold)}` : ""}`);
      // Cut 28 §3: a hero handed off low (`low`: hp ≤ half his max) — his `hp 9/40` chip leads the line (the core's, else built here)
      const hpChip = L.low && L.hp !== undefined && L.max_hp !== undefined ? L.chips.find((c) => /^hp\b/.test(c)) ?? `hp ${L.hp}/${L.max_hp}` : undefined;
      const chipList = hpChip ? [hpChip, ...L.chips.filter((c) => c !== hpChip)] : L.chips;
      replace(foldChips, ...chipList.map((c) => h("i", { class: `fchip k-${c === hpChip ? "hp" : coreKind(c, L)}`, "data-k": c === hpChip ? "hp" : coreKind(c, L) }, c)));
      foldLine.dataset.kinds = [...new Set(L.beats.map((b) => b.kind))].join(",");
      foldLine.dataset.src = "core";
      return;
    }
    replace(foldHead, f.tally.head(f.to));
    const chips = f.tally.list();
    replace(foldChips, ...chips.map((c) => h("i", { class: `fchip k-${c.k}`, "data-k": c.k }, c.text)));
    foldLine.dataset.kinds = [...new Set(chips.map((c) => c.k))].join(",");
    foldLine.dataset.src = "client";
  }
  /** A core chip's kind (for its colour): the kind of the beat whose words it carries, else by its first word. */
  const coreKind = (chip: string, L: FoldLine): string => {
    const b = L.beats.find((x) => x.text === chip); if (b) return b.kind === "theft" ? "stolen" : b.kind === "dip" ? "hp" : b.kind === "find" ? "found" : b.kind;
    return /^stolen\b/.test(chip) ? "stolen" : /^hp\b/.test(chip) ? "hp" : /\bfinds?\b|^found\b/.test(chip) ? "found" : "beat";
  };
  const foldedAway = new Set<number>();   // floors a fold already stepped (a fold that gave up on one never folds it again)
  /** Is the floor the picture meets next (a queued load, else the HUD's own floor with the engine on it) the first of a folded stretch? */
  function foldDue(): boolean {
    if (folding || held || exitTier || done || !foldSet.size || overridden || !snap || !viewer) return false;
    const d = loads.length ? loads[0].snap.depth : hud.depth;
    if (!foldSet.has(d) || foldedAway.has(d)) return false;
    return loads.length ? (cardUp || viewerIdle()) && !beatHeld() : snap.depth === d;
  }
  /** Step the folded stretch flat out under its line; the watch lands on the first floor below the bar (or the run's ending). */
  async function foldRun(): Promise<void> {
    if (folding || !viewer || !snap) return;
    const from = loads.length ? loads[0].snap.depth : snap.depth, to = planTo(from);
    const tally = new FoldTally(from, stretchShare(app.forecastOfRules(), from, to, foldStartOf()));
    tally.loot0 = carriedOf(hudSnap ?? snap);
    const f: Fold = { tally, from, to, t0: performance.now(), holdUntil: Infinity, stepping: true, skip: false, replay: false };
    folding = f; el.dataset.fold = "1";
    releaseBeat(); clearTimeout(dockTimer);
    cardUp = false; card.hidden = true; el.dataset.card = "0"; ticker.classList.remove("show"); tickerQueue.length = 0;
    foldLine.classList.remove("docked", "faded"); foldLine.style.top = ""; foldLine.hidden = false; paintFold(f);
    inflight = true;
    // Cut 27 §1 (core): right after the send the core plays the stretch itself (`fold()`: the line, its floors for the replay, the whole
    // stretch as one step); a core without it — or a stretch later in the run — is stepped here, batch by batch
    let core: FoldLine | null | undefined;
    if (app.engine.fold && from === startDepth && snap.turn === startTick) { try { core = await app.engine.fold(); } catch (e) { console.warn("fold unavailable", e); core = undefined; } }
    if (disposed) return;
    if (core) {
      if (core.to < core.from) { folding = null; el.dataset.fold = "0"; foldLine.hidden = true; foldSet.clear(); inflight = false; applyFrame(); applySpeed(); return; }   // nothing folded: the watch as ever
      f.core = core; f.to = core.to;
      handle(core.step);
      if (cage && !cage.done) { cage.done = true; cage.shown = true; }
      drainLoads(); paintFold(f);
    }
    try {
      drainLoads();
      while (!core && !disposed && !held && foldSet.has(snap.depth) && performance.now() - f.t0 < FOLD_WALL_MS) {
        const r = await app.engine.step(SKIP_END_BATCH);
        if (disposed) return;
        handle(r);
        if (cage && !cage.done) { cage.done = true; cage.shown = true; }   // Cut 19 §1: under the fold the preference picks (its pick is a chip)
        drainLoads();
        paintFold(f);
        if (r.run_over || held || stalling()) break;
      }
    } catch (e) { console.warn("fold failed", e); }
    inflight = false;
    if (disposed) return;
    f.stepping = false;
    if (!core) f.to = Math.max(from, Math.min(planTo(from), foldSet.has(snap.depth) ? snap.depth : snap.depth - 1));
    paintFold(f);
    f.holdUntil = f.skip ? 0 : f.t0 + FOLD_MIN_MS;
    // the picture lands at the first floor below the bar — its stairs' tick (the floor the engine is on), or the ending's start
    const d = descends[descends.length - 1];
    const t = held ? Math.max(viewerTick(), endingFrom) : Math.max(viewerTick(), d !== undefined && snap.depth > from ? d : engineTick);
    seekTo(t); release(t); letGo(t);
    worldT = Math.max(worldT, engineTick); lastPumpMs = performance.now();
    if (f.replay) { f.replay = false; foldTap(); }
  }
  /** The fold's line has held its minimum (or a press waived it): it docks under the HUD and the watch plays on from the landing. */
  function endFold(): void {
    const f = folding; if (!f) return;
    folding = null; el.dataset.fold = "0";
    lastFold = { from: f.from, to: f.to, core: f.core };
    for (let d = f.from; d <= Math.max(f.to, snap?.depth ?? f.to); d++) if (foldSet.has(d)) foldedAway.add(d);
    foldsDone.push(`${f.from}-${f.to}`); el.dataset.folded = foldsDone.join(",");
    const ms = Math.round(performance.now() - f.t0), floors = f.to - f.from + 1;
    if ("__riddle" in window) ((window as unknown as { __foldLog?: unknown[] }).__foldLog ??= []).push({ from: f.from, to: f.to, floors, ms, perFloor: Math.round(ms / floors), src: f.core ? "core" : "client", kinds: f.core ? [...new Set(f.core.beats.map((b) => b.kind))] : [...f.tally.kinds()], chips: f.core ? f.core.chips : f.tally.list().map((c) => c.text), shown: [...foldChips.children].map((c) => c.textContent), head: foldHead.textContent });   // dev
    foldLine.classList.add("docked");
    dockTimer = window.setTimeout(() => foldLine.classList.add("faded"), FOLD_DOCK_MS);
    if (hudSnap) { hudFrom(hudSnap); }
    if (held) toEnding(); else { applyFrame(); applySpeed(); }
    // The restored HUD can move the docked chips; reserve their final geometry.
    paintKeepOut(true);
  }
  /** A tap on the fold line plays the folded floors (the run log's, at the travel rate); the watch's picture freezes meanwhile. */
  function foldTap(): void {
    if (folding?.stepping) { folding.replay = true; return; }
    const span = folding ? { from: folding.from, to: folding.to, core: folding.core } : lastFold;
    const log = lastRun(); if (!span) return;
    // the core's floors (each from its first tick) when it folded, else the run log's (the floors the watch loaded under the line)
    const floors = span.core ? span.core.floors.map((x) => ({ snap: x.snapshot, evs: x.events })) : log && log.runId === runId ? foldFloorsOf(log, span.from, span.to) : [];
    if (!floors.length) return;
    if (folding) folding.holdUntil = 0;
    const wasPaused = paused;
    if (!wasPaused) { freeze(true, hidden); paintPause(); applySpeed(); }
    el.dataset.foldReplay = "1";
    openFoldReplay(floors, span.from, span.to, () => { el.dataset.foldReplay = "0"; if (!wasPaused && !disposed && paused) { freeze(false, hidden); paintPause(); applySpeed(); } });
  }
  /** The floor the send started on (the forecast's sims start there too; a fold never runs past what they measured). */
  const foldStartOf = (): number => (snap?.run.start ?? startDepth);
  let startDepth = 1;

  let skipQueued = false;
  function skipToCard(): void {
    skipEarly = true; mapHold = false; cardLive = false;
    goLive(); paintCard(frame); cardSince = -Infinity; applyFrame(); applySpeed();
  }
  let stuckFight = -Infinity;   // QA 778fa1b (qaV): the fight a ▶▶| could not close (its `fightFrom`)
  let lastPressAt = -Infinity, pressDepth = -1, pressRun = 0, pressAny = 0;   // QA 778fa1b (qaV): presses in a row on one floor
  async function skipToEvent(): Promise<void> {
    skipping = true;
    try { await skipToEvent0(); } finally { skipping = false; lastPressAt = performance.now(); }
  }
  async function skipToEvent0(): Promise<void> {
    if (done || !viewer || exitTier) return;
    if (folding) { folding.skip = true; folding.holdUntil = 0; return; }   // Cut 27 §1: the press lets the fold line go (the watch lands below it)
    // QA on 3d71c33: under the cage sheet ▶▶| picks nothing — the sheet goes and the engine's grace and the preference decide
    if (vaultClose) vaultClose();
    if (cage) cage.done = true;   // Cut 19 §1: …and the cage beat: the preference picks
    releaseBeat();   // Cut 18 §1: the press lets a held beat go
    // Cut 14 §6: `▶▶|` is "live": a frozen picture resumes; a viewer behind the frontier (a pause, a hidden tab, a slow fight while
    // the world ran on) lands on it first — the ending's start at most, so the walk-out plays — and the press then means what it
    // meant live: the fight's end or the next fight in `fights`, the run's end in `fast`, landing on the new frontier
    if (paused) { freeze(false, hidden); paintPause(); }
    const behind = held ? viewerTick() < endingFrom : loads.length > 0 || engineTick - viewerTick() > LEAD_FAST + BATCH_FAST;   // past the live lead
    if (behind && !(mode === "fights" && cardUp && !mapHeld())) goLive();   // …then the mode's own skip runs from live
    // Cut 10 §1: under the card the engine is already running to the next fight (or has found it): the press waives the card's minimum
    if (mode === "fights" && cardUp && !mapHeld()) { cardSince = -Infinity; if (inflight) return; applyFrame(); applySpeed(); return; }
    // a press while a step is in flight is not lost: one skip is queued behind it
    if (inflight) { skipQueued = true; return; }
    if (held) { toEnding(); return; }   // the run is over: the ending plays (Cut 14 §6: never skipped blind)
    // QA 778fa1b (qaV: ▶▶| every 400 ms for 360 s in `fights`, retreat ↔ attack against two ogres on D8, the floor never changed): presses
    // in quick succession that leave him on the same floor are a loop the fights keep opening — the fourth takes the run's end
    const nowMs = performance.now();
    if (nowMs - lastPressAt < PRESS_GAP_MS && hud.depth === pressDepth) pressRun++; else { pressRun = 0; pressDepth = hud.depth; }
    pressAny = nowMs - lastPressAt < PRESS_GAP_MS ? pressAny + 1 : 0;
    if (pressAny >= PRESS_ANY) pressRun = Math.max(pressRun, PRESS_LOOP);
    lastPressAt = nowMs;
    if (mode === "fights" && earlyFloor() && frame !== "fight" && !stalling() && pressRun < PRESS_LOOP) { skipToCard(); return; }
    inflight = true;
    const until = performance.now() + SKIP_WALL_MS;
    // in `fast` the press means the run's end: the engine steps to `run_over` (floors drained on the way, a vault choice left to
    // its grace) and the viewer lands at the ending, which plays as after any skip (QA on 50bb162: the press "plays faster")
    // QA 23ed91f (L: a summoner stall on D6, ▶▶| and `fast` changed nothing): while the run is stalling the press means the run's
    // end in either mode — there is no fight worth landing on in a loop
    // QA 778fa1b (qaV: ▶▶| every 400 ms for 360 s in `fights` against two ogres, retreat ↔ attack; `fast` ended it in 3 s): a fight that
    // does not close within the press's fight budget is a loop — the press takes the run's end, as in `fast`
    const toRunEnd = async (budget: number): Promise<void> => {
      inflight = true;
      try {
        for (let i = 0; i < SKIP_FIGHT_BATCHES && !held && !disposed && performance.now() < budget; i++) {
          const r = await app.engine.step(snap && cageNear(snap) ? CAGE_BATCH : SKIP_END_BATCH); handle(r);
          if (held || r.run_over || cageMet()) break;
          loadNext();
        }
      } catch (e) { console.warn("skip failed", e); }
      inflight = false;
      if (held) toEnding();
      else if (cageMet()) landCage();
      else if (!loads.length) { const t = Math.max(viewerTick(), engineTick - BATCH); seekTo(t); release(viewerTick()); letGo(t); applyFrame(); applySpeed(); }
    };
    if (mode === "fast" || stalling() || pressRun >= PRESS_LOOP) {
      // QA 1a2a4a9 (P: at 16× the cage's `took leather +1` was gone in ~1.5 s, untappable): a skip stops at a cage — the beat holds
      // its wall time like any other; near an unopened cage the steps are short, so the choice is met inside its grace
      await toRunEnd(until);
      if (skipQueued) { skipQueued = false; void skipToEvent(); }
      return;
    }
    // Cut 10 §1: inside a shown fight the press means its end — the span's close is known (the engine ran ahead) or is stepped to
    if (frame === "fight") {
      try {
        for (let i = 0; i < 120 && fightOn && !Number.isFinite(fightUntil) && !disposed && performance.now() < until; i++) {
          const r = await app.engine.step(BATCH); handle(r);
          if (held) break;
          loadNext();
        }
      } catch (e) { console.warn("skip failed", e); }
      inflight = false;
      // a fight one press could not close is marked; a second press in the same fight takes the run's end
      if (!held && !disposed && fightOn && !Number.isFinite(fightUntil)) {
        if (stuckFight === fightFrom) { await toRunEnd(performance.now() + SKIP_WALL_MS); if (skipQueued) { skipQueued = false; void skipToEvent(); } return; }
        stuckFight = fightFrom;
      }
      if (held) toEnding();
      else {
        // Cut 14 §6: the end of the span the playhead is in (a kept one, or the live fight's), whichever is later
        const v = viewerTick(), sp = spans.find((x) => v >= x.from && v < x.until);
        const t = killStop(Math.max(Number.isFinite(fightUntil) ? fightUntil : -Infinity, sp?.until ?? -Infinity, Number.isFinite(fightUntil) || sp ? -Infinity : Math.max(v, engineTick)));   // blind 5331f40: short of an unshown kill
        seekTo(t); release(t); letGo(t); applyFrame(); applySpeed();
        // QA e75ec29: then on to the next fight worth watching — the card takes the travel as soon as the frame gives way
        if (mode === "fights" && earlyFloor()) { if (frame !== "fight") skipToCard(); else skipEarly = true; }
      }
      if (skipQueued) { skipQueued = false; void skipToEvent(); }
      return;
    }
    // Cut 10 §1: a press outside a fight means "the next fight, now": the engine steps until its snapshot opens the fight frame
    // (`fightOn`, the mode's own predicate, judged per batch in `handle`), floors drained on the way, and the viewer lands on that
    // first frame (`fights`; `fast` took the end above).
    let landed = false;
    try {
      // a fight the engine opened ahead of the viewer's clock is the one to land on (no stepping); one the viewer is inside is
      // stepped to its close, then the next
      const ahead = fightOn && fightFrom > viewerTick();
      const inFight = fightOn && !ahead;
      let hit = ahead;
      const max = inFight ? 120 : SKIP_FIGHT_BATCHES;
      for (let i = 0; i < max && !hit && !disposed && performance.now() < until; i++) {
        const r = await app.engine.step(snap && cageNear(snap) ? Math.min(BATCH, CAGE_BATCH) : BATCH);
        handle(r);
        hit = r.run_over || (inFight ? !fightOn : fightOn);
        if (held || cageMet()) break;   // QA 1a2a4a9: a cage stops the skip (its beat, then the next press goes on)
        // a floor change on the way: load it now (the skip is the drain), the queued events straight into place
        loadNext();
      }
      landed = fightOn && fightFrom > viewerTick();
    } catch (e) { console.warn("skip failed", e); }
    inflight = false;
    if (held) toEnding();
    else if (cageMet()) landCage();
    else if (!loads.length) {
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
      tactics.disable(); viewer?.setTacticRows?.([], []); // Legacy prepend changes every row index.
      void app.engine.setRules({ rows: [{ conds: [], verb: { v: "return" } }, ...app.rules.rows] }).catch((e2) => console.warn("bail", e2));
    });
    setMode(mode);
  }
  // Cut 5 §4: the vault choice — once per cage; Cut 19 §1: a beat (`took mail`), the sheet only on a tap on it
  function vaultFrom(s: Snapshot): void {
    const vc = s.vault_choice;
    // a choice on a run that has ended is no choice (a `choose` after the exit trips the core: seed 12's death on a vault tile)
    if (!vc || !vc.items.length || held || exitTier || done) {
      if (vaultClose) { vaultClose(); vaultClose = null; }
      // QA 23ed91f (K: "the cage sheet closed by itself about 30 s later … nothing said what had been taken"): once the cage is gone
      // from the snapshot, a pick the player did not make (the preference's) is named off the hero's pack — unless the beat named it
      if (vaultItems.length && !vc) {
        const got = !vaultChosen && !held && !exitTier && !done ? vaultItems.find((it) => s.hero.inv.some((x) => x.id === it.id)) : undefined;
        const said = cage?.shown ? cage.pick?.id : undefined;
        vaultItems = [];
        if (got && got.id !== said) { tickerQueue.length = 0; showTicker(tookText(got.label), "", BEAT_HOLD_MS); }   // over the card too (the ticker sits above it): the pick is news
      }
      if (!vc) cage = null;
      return;
    }
    const key = vc.items.map((it) => it.id).join(",");
    if (key === vaultKey) { if (cage) cage.vc = vc; return; }
    if (frozen()) return;   // Cut 14 §6: not over a frozen picture (a later snapshot inside the grace opens it on the thaw)
    vaultKey = key; vaultItems = vc.items; vaultChosen = false;
    // Cut 10 §1: found under the card, the viewer joins the engine at the vault (the world waits from here)
    if (cardUp) { release(s.turn); seekTo(s.turn); }
    // Cut 19 §1: the pick the preference will take (`VaultChoice.pick`; an older core: the first item) as the beat, cut in at the
    // cage's opening tick (the grace's start)
    const pick = vc.items.find((it) => it.id === vc.pick) ?? vc.items[0];
    cage = { vc, pick, at: performance.now(), shown: false, done: false };
    const t0 = Math.max(startTick, s.turn - Math.max(0, VAULT_GRACE - (vc.left ?? VAULT_GRACE)));
    beatAt(t0, tookText(pick.label), false, false, true);
    el.dataset.cage = "beat";
    paintCard(frame); applyFrame(); applySpeed();
  }
  /** Cut 19 §1: a tap on the cage beat's line within its hold opens the override. */
  function cageTap(): void {
    if (!cage || cage.done || vaultClose || !ticker.classList.contains("cage") || !(beatHeld() && heldBeat?.cage)) return;
    vaultSheet(cage.vc, cage.pick?.id);
  }
  ticker.onclick = () => { if (cage && !cage.done && ticker.classList.contains("cage")) { cageTap(); return; } whyTap(); };
  canvas.onclick = (): void => whyTap();
  /** Cut 23 §3: the last line with a reason (a `✗` refusal, a shouted word) and when it was released. */
  function noteWhy(text: string, why: string): void { lastWhy = { text, why, at: performance.now() }; }
  /** Cut 23 §3: a tap on the line (or the picture) within WHY_TAP_MS of a reasoned line opens its reason under it — `read ✗ no use`
   *  `→ nothing to learn` — for WHY_SHOW_MS. */
  function whyTap(): void {
    const w = lastWhy; if (!w || performance.now() - w.at > WHY_TAP_MS) return;
    replace(whyTip, h("span", { class: "dim" }, w.text), " → ", h("b", null, w.why));
    whyTip.classList.add("show"); whyTip.dataset.text = w.text;
    clearTimeout(whyTimer); whyTimer = window.setTimeout(() => whyTip.classList.remove("show"), WHY_SHOW_MS);
  }
  function vaultSheet(vc: VaultChoice, pickId?: number): void {
    let sent = false;
    vaultAt = performance.now(); vaultItems = vc.items; vaultChosen = false;
    el.dataset.cage = "sheet";
    // QA on 3d71c33: a full vault salvages the pick at the exit — the sheet says so; QA 1a2a4a9 (P: "what the vault has to do with
    // the cage"): …and what it means for the pick (`vault full → sold`: he uses it this run, the exit sells it)
    const full = app.lineage.vault.length >= vaultSlots(app.lineage.unlocks);
    // the cage is modeless: the HUD stays live around it (⏸ keeps it, ▶▶| lets it go — QA on 3d71c33: "click intercepted")
    openSheet((close) => {
      vaultClose = () => { vaultClose = null; graceBar = null; if (cage) cage.done = true; el.dataset.cage = "done"; close(); paintCard(frame); applySpeed(); };
      const chips = h("div", { class: "chips" }, ...vc.items.map((it) => h("button", { class: `chip item${it.id === pickId ? " on" : ""}`, onclick: () => {
        if (sent) return; sent = true; vaultChosen = true;
        releaseBeat();
        app.engine.choose(it.id).catch((e) => console.warn("choose", e)).finally(() => { vaultClose?.(); if (!disposed) { tickerQueue.length = 0; showTicker(tookText(it.label), "", 2400); } });   // the next step's snapshot carries the pickup
      } }, itemIcon(it, { size: "s" }), itemName(it))));   // run-clear: the cage's three in their rarity rims
      // the wait as a shrinking bar (QA on 56f2a1d: "closes by itself ~2–4 s later with no timer"): Cut 15 §5, the wall-clock wait
      // (VAULT_WAIT_MS; the world stands meanwhile), width 100% → 0 over it — `vc.left` is the engine's own grace, after that
      const bar = h("i", { style: `transition-duration:${VAULT_WAIT_MS / 1000}s` });
      graceBar = bar;
      const grace = h("div", { class: "grace" }, bar);
      requestAnimationFrame(() => requestAnimationFrame(() => { if (frozen()) graceHold(); else bar.style.width = "0%"; }));
      // the three-item room is the cage to the player (the core's `twist_word`; QA on 3d71c33: "a sheet titled VAULT")
      return h("div", { class: "sheet-body vault-choice" }, h("div", { class: "label row-label" }, /* copy:label */ "loot choice"),
        full ? h("div", { class: "vault-full dim num" }, /* copy:callout */ "storage full → sold") : "", chips, grace);
    }, { modeless: true });
    paintCard(frame); applySpeed();
  }
  /** QA 92eb880 (N: `fighter +0 · L4 ↑1` — the client's 40·L² ladder was not the core's): the run's XP is the exit line's (`ExitLine.xp` /
   *  `level_ups`); an older build without them shows the gain only inside one level (nothing is computed off a ladder here). */
  function xpOfRun(): { gained: number; level_ups: number } {
    const c = app.lineage.classes?.[cls] ?? { level: 1, xp: 0 };
    if (exitLine?.xp !== undefined) return { gained: exitLine.xp, level_ups: exitLine.level_ups ?? Math.max(0, c.level - before.level) };
    return { gained: c.level === before.level ? Math.max(0, c.xp - before.xp) : 0, level_ups: Math.max(0, c.level - before.level) };
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
    if (paused) { freeze(false, hidden); paintPause(); applySpeed(); }
    // QA 23ed91f (K: "the header already reads ♟2 · curious · $40 while ♟1's last frame is on screen"): the bar holds the heir
    // that ran until the next screen (the exit's refresh brings the next heir's lineage)
    bar.freeze();
    // K: "the primary gem is gone; a tiny VERDICT label sits at the top right": the gem slot keeps a gem — what comes next (the
    // verdict, the report); a tap cuts the walk-out short. The busy label goes into it (the corner's stays hidden)
    // QA 92eb880 (N: "VERDICT appears while the hero is still up (8/36), three more hits follow"): during the walk-out the gem slot holds
    // the stilled pause; the verdict / report gem comes once the last frame has played (`nextGem`)
    pause.disabled = true;
    for (const b of [modeBtn.fights, modeBtn.fast, modeBtn.one, skip, bail, speedBtn, toTown]) b.disabled = true;
    paintWatchStatus();   // Keep the ended state visible while its last picture plays.
  }
  function nextGem(): void {
    if (el.dataset.next === "1" || !pause.isConnected) return;
    el.dataset.next = "1";
    const next = exitTier === "death" ? /* copy:button */ "verdict" : /* copy:button */ "report";
    // blind 1fb7786 (A: "REPORT gem unresponsive"): the gem stands above the keep sheet's backdrop — a tap there means go on: the sheet
    // closes keeping its ticked picks (`exitSheet`'s close) and the flow goes to the report
    const g = gem({ label: next, cls: "next-gem", pulse: true, onclick: () => { exitAt = 0; exitBeatUntil = 0; restUntil = 1; keepClose?.(); } });
    pause.replaceWith(g);
    setBusyHost(busyHost);
  }
  async function finish(tier: Tier): Promise<void> {
    if (done) return;
    done = true; clearInterval(pumpTimer); card.hidden = true; cardUp = false; el.dataset.card = "0"; scrub.hidden = true;   // Cut 14 §6: no run live
    endControls(); nextGem();
    // whatever happens below, the player reaches a screen with buttons
    const guard = window.setTimeout(() => { if (!disposed && app.view.kind === "watch") { console.warn("exit flow stalled; falling back to camp"); app.go({ kind: "camp" }); } }, 20_000);
    try {
      if (prepended) await bounded(app.engine.setRules(app.rules), 8000, /* copy:none */ "setRules after bail");
      // The keep sheet is a decision only when there is a free vault slot; otherwise the engine keeps by preference.
      // QA e75ec29 (R: from run 5 on no keep sheet, the finds salvaged): the camp's lineage still lists the vault items the send
      // brought along (the engine took them out of the vault for the run; they come home among the pending items, pre-ticked) —
      // counted twice, a 2-slot vault with one item brought read full. The slots are the engine's, read now.
      const fresh = pendingExit?.items.length ? await bounded(app.engine.lineage(), 4000, "lineage at exit") : undefined;
      const vaultNow = fresh ?? app.lineage;
      const freeSlots = Math.max(0, vaultSlots(vaultNow.unlocks) - vaultNow.vault.length);
      // Cut 29 §4 (AW: "one ▲ per tap"): the core says whether the sheet is a decision (`decide`: a find beats something in the full
      // vault, or finds outnumber its free slots). Not one → settled by the standing order (`autoKeep`) with its one line (`note`).
      // A decision on a full vault opens the sheet as a swap: one pick, which the core keeps in place of the vault's weakest.
      const decide = pendingExit?.decide;
      if (pendingExit && pendingExit.items.length && (decide === undefined ? freeSlots > 0 : decide)) { const p = pendingExit; pendingExit = undefined; clearTimeout(guard); exitSheet(p, Math.max(1, freeSlots), () => { done = false; void finish(tier); }, freeSlots === 0); return; }
      const settledNote = decide === false ? pendingExit?.note : undefined;
      // the vault full: the preference keeps (as designed) — and the screen says so before the report (R: "keep chosen for me?")
      const vaultFull = !!(pendingExit?.items.length && fresh && (decide === false ? true : freeSlots === 0));
      // the sheet skipped (the vault full): the engine keeps by preference and the rest is salvage all the same — its rows
      // are built here too (QA on e0f87e7: "no SALVAGED block at all when the vault is full … yet the gold sheet shows +$16
      // salvage"); what the vault gained across the keep is what was kept, matched to the pending items by kind
      const skipped = pendingExit; pendingExit = undefined;
      const vaultBefore = new Set(app.lineage.vault.map((v) => v.id));
      // QA 23ed91f (L: `auto: keep weapon+armour` owned, the vault full → mail · axe · sword all salvaged): the skipped sheet
      // resolves by the preference and the owned automations (`autoKeep`); an old build keeps nothing
      if (skipped) await bounded(app.engine.autoKeep ? app.engine.autoKeep() : app.engine.keep([]), 8000, "keep by preference");
      // (`refresh` resolves void, so a sentinel tells a timeout from success)
      if (!(await bounded(app.refresh().then(() => true), 8000, "refresh at exit")) && !disposed) { clearTimeout(guard); app.go({ kind: "camp" }); return; }
      if (!disposed) {
        bar.paint();
        const target = bar.el.querySelector<HTMLElement>(".gold");
        if (target && app.lineage.town?.auto_collect && (exitLine?.kept ?? 0) > 0) flyCoins(target, exitLine!.kept);
      }
      // QA a946e04 (T: the vault's dagger became `axe +1` behind a `vault full` flash, no sheet): the preference's swap is named —
      // `axe +1 → vault` — and `vault full` stands alone only when nothing went in
      if (vaultFull) {
        const inVault = app.lineage.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.label);
        // Cut 29 §4: a settled exit says its one line (`kept leather +1`); with nothing new kept it says nothing unless the vault is full
        const full = vaultSlots(app.lineage.unlocks) <= app.lineage.vault.length;
        if (settledNote || inVault.length || full) {
          showBanner(settledNote ?? (inVault.length ? /* copy:callout */ `${inVault.join(", ")} → storage` : /* copy:callout */ "storage full"), VAULT_FULL_MS, "rest keep-note");
          await new Promise((r) => setTimeout(r, VAULT_FULL_MS));
        }
      }
      if (skipped) {
        const kept = new Set<number>();
        const gained = app.lineage.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.kind);
        for (const kind of gained) { const it = skipped.items.find((x) => x.kind === kind && !kept.has(x.id)); if (it) kept.add(it.id); }
        salvagedRows = letGoRows(skipped, kept);
        keptLabels = app.lineage.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.label);
      }
    } finally { /* guard cleared on every normal path below */ }
    await finishAfterRefresh(tier, guard);
  }
  /** What an exit let go, per kind at the engine's worth at this exit (`salvageValue` when the wire has none). */
  function letGoRows(p: { items: InvItem[]; tier: string; worth?: number[] }, kept: Set<number>): { kind: string; n: number; gold: number }[] {
    const rows = new Map<string, { kind: string; n: number; gold: number }>();
    // QA a946e04 (S: SALVAGED `potion ×2 · $2` beside `blue potion? ×2`): an unknown item is its flavour (`blue potion?`), as the core's rows name it
    p.items.forEach((it, i) => { if (kept.has(it.id)) return; const k = it.known ? it.kind : it.label; const r = rows.get(k) ?? { kind: k, n: 0, gold: 0 }; r.n++; r.gold += p.worth?.[i] ?? salvageValue(it.kind, p.tier); rows.set(k, r); });
    return [...rows.values()];   // QA 912e135 (qaX: the forge counted a `$0` leash the SALVAGED list left out): every item let go is named
  }
  async function finishAfterRefresh(tier: Tier, guard: number): Promise<void> {
    clearTimeout(guard);
    app.runsSeen += 1;
    if (disposed) return;
    app.rowFires = app.rules.rows.map((_, i) => rowFires[i] ?? 0); app.rowFiresOf = undefined;   // Cut 14 §4 / Cut 15 §3: the run's fires per row, for the drop sheet
    tamed.push(...tamedIds.map(compLabel));
    lost.push(...lostIds.map(compLabel));
    // Cut 13 §1: a run that came home stalled (`… · stalled` on its line; the stake was `stalling` at the exit) gets a verdict screen
    // like a death's — the core records the stall, `death(runId)` answers `verdict: "stall"`; an older core falls back to the report
    // QA 308f045 (qaAC: one run read `1 STALLED · $0 lost` and `1 DRIVEN` under `1 RUNS`): one outcome per run — a drive-off (the line's
    // `driven`, the core's word) is never a stall, whatever the stake said while the boss's shields held
    const stalled = tier === "return" && !exitLine?.driven && !/^driven\b/.test(exitLine?.text ?? "") && (/\bstalled\b/.test(exitLine?.text ?? "") || (!!snap?.stake?.stalling && (exitLine?.kept ?? 1) === 0));
    if (tier === "death") for (const c of partyAtStart) if (!lost.some((l) => l === c || l.endsWith(c.slice(c.indexOf(" · "))))) lost.push(c);
    // Cut 26 §6 (AP): a drive-off opens its verdict (the exit line's `driven`: the boss, the defence, the counter to write)
    if (tier === "return" && exitLine?.driven && !stalled) { app.go({ kind: "death", death: drivenDeath(exitLine, runId, exitTrace), lost }); return; }
    if (tier === "death" || stalled) {
      try {
        const death = await app.busy(/* copy:label */ "verdict", () => app.engine.death(runId));
        if (stalled && death.verdict !== "stall") throw new Error(`no stall verdict (${death.verdict})`);
        death.line ??= exitLine;   // Cut 6 §1: the verdict may lack the line; the exit event carried it
        if (tollShort && death.line && death.line.start_short === undefined) death.line = { ...death.line, start: 1, start_short: true };
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
      elapsed_s: Math.round((engineTick - startTick) / 10), runs: 1, sampled: false, learned, bests, found, pending: [], packages: exitLine?.packages,
      deaths: tier === "death" ? [{ cause: heroCause ?? exitLine?.text ?? /* copy:label */ "death", n: 1 }] : [],   // Cut 10 §3: the death it came from
      stolen: exitLine?.stolen?.length ? [...exitLine.stolen.reduce((m, l) => m.set(l, (m.get(l) ?? 0) + 1), new Map<string, number>())].map(([label, n]) => ({ label, n })) : undefined,   // QA e75ec29 (R)
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap!, tamed, hatched: [], lost,
      legacy_earned: exitLine?.legacy_earned,
      xp: { class: cls, ...xpOfRun() },
      salvaged: reconcileSalvage(mergeSalvage(exitLine?.salvaged ?? [], salvagedRows), L.gold_ledger ?? []), kept: keptLabels.length ? keptLabels : undefined, deepest, renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
      spent: [...(exitLine?.toll !== undefined ? (exitLine.toll > 0 ? [{ kind: /* copy:none */ `waystone D${exitLine.start ?? L.start ?? 1}`, n: 1, gold: exitLine.toll }] : []) : tollOf(L.gold_ledger ?? [])), ...spentRows(L.gold_ledger ?? [])],   // Cut 13 §3: what the automations bought at this exit (`heal ×1 · −$40`); QA a946e04: the send's toll first
      stolen_gold: exitLine?.stolen_gold ?? (stolenGold > 0 ? stolenGold : undefined),   // QA a946e04 (T): the carry's thefts, beside the items
      banked: tier === "bank" ? 1 : 0, returned: tier === "return" ? 1 : 0, stalled: stalled ? 1 : 0, driven: exitLine?.driven ? 1 : 0,   // Cut 18 §4: the stall the exit line names anywhere in it (`… · stalled · 1 supply back`), or the stake's flag bones_found: bonesFound,   // rest is still ahead: the camp shows it
      exits: exitLine ? [{ ...exitLine, trace: exitLine.trace ?? exitTrace }] : undefined,            // Cut 6 §1; Cut 9 §5: with its trace
      // Cut 25 §6 (AM: `RUNS 1 · DEATHS 0` after the last heir's death read as the lineage's): the runs tile names the heir who ran (`♟2`)
      heirs: snap?.run?.heir !== undefined ? [snap.run.heir, snap.run.heir] : undefined,
      lanes: runLanes(),   // Cut 26 §2: the lanes this run walked (`D5–7 · the Fens`), once a fork was seen
    };
    app.go({ kind: "report", report });
  }

  /** Cut 26 §2: the lanes the watched run walked, below the Warrens — each stretch of floors in one biome, as far as he got. */
  function runLanes(): string[] | undefined {
    if (!seenForks(app.lineage).length) return undefined;
    const ds = [...floors.entries()].filter(([, f]) => f.biome && f.biome !== "warrens").sort((a, b) => a[0] - b[0]);
    const out: { a: number; b: number; biome: string }[] = [];
    for (const [d, f] of ds) { const last = out[out.length - 1]; if (last && last.biome === f.biome && d === last.b + 1) last.b = d; else out.push({ a: d, b: d, biome: f.biome! }); }
    return out.length ? out.map((x) => /* copy:callout */ `${x.a === x.b ? `D${x.a}` : `D${x.a}–${x.b}`} · ${laneTitle(x.biome)}`) : undefined;
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
    let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled|driven)\b/.test(ledger[i].why)) i--;
    const lines = ledger.slice(i + 1).filter((g) => /^salvage/.test(g.why));
    if (!lines.length || !rows.length) return rows;
    let diff = lines.reduce((a, g) => a + g.delta, 0) - rows.reduce((a, r) => a + r.gold, 0);
    const out = rows.map((r) => ({ ...r })).sort((a, b) => b.gold - a.gold);
    for (const r of out) { if (!diff) break; const take = Math.max(-r.gold, diff); r.gold += take; diff -= take; }
    return out.filter((r) => r.n > 0);   // QA 912e135: a `$0` item is still named (the forge counts it)
  }
  const spentRows = (ledger: { t: number; delta: number; why: string; n?: number }[]): { kind: string; n: number; gold: number }[] => spentOf(ledger, app.supplyCat);
  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[] }, free: number, then: () => void, swap = false): void {
    // QA 23ed91f: the owned automations' picks come pre-ticked (`ExitPending.auto_keep`), as many as the free slots take
    const keep = new Set<number>((p.auto_keep ?? []).filter((id) => p.items.some((it) => it.id === id)).slice(0, free));
    let sent = false;
    keepClose = null;
    // QA 308f045 (qaAC: `mail $0 · sword $0 · … · unkept → salvage` after a drive-off that kept nothing): an exit whose salvage pays nothing
    // prices nothing — the chips carry no `$0` and the legend says the unkept are lost
    const worthOf = (i: number): number => p.worth?.[i] ?? salvageValue(p.items[i].kind, p.tier);
    const unpaid = p.items.length > 0 && p.items.every((_, i) => worthOf(i) <= 0);
    openSheet((close) => {
      keepClose = close;
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
        } }, itemIcon(it, { size: "s" }), itemName(it), " ", /* QA 1a2a4a9 (P: "`axe ⌂` — what ⌂ means"): a kept pick reads where it goes */ keep.has(it.id) ? h("b", null, "→ ", /* copy:label */ "stored gear") : unpaid ? "" : h("b", { class: "num gold" }, `$${worthOf(i)}`))));   // the engine's worth at this exit (its old client table read 4×)
      };
      paint();
      // the pile once: the exit line carries `bones: 8 items on D4` (the core's), so the client's `bones left` line only stands in
      // for an exit line without it (two QA players on 50bb162: "bones: 8 items on D4 ... bones left · 8 items")
      const bones = p.tier === "death" && bonesLeft !== undefined && !/\bbones:/.test(exitLine?.text ?? "") ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(bonesLeft)}`) : null;
      const ledger = exitLine?.text ? h("div", { class: "ledger-line num dim" }, exitLine.text + exitExtras(exitLine)) : null;   // Cut 6 §1: engine data, verbatim
      // Cut 24 §2: what was new this run leads the sheet's lines (`record: D10 · avenged Ulak`), before the counts
      const newsT = mergeFinds((exitLine?.news ?? []).filter((n) => n.k !== "differ")).map((n) => n.text);
      const news = newsT.length ? h("div", { class: "keep-news num" }, newsT.slice(0, 3).join(" · ")) : null;
      // Cut 9 §5: the trace on a chip; Cut 11 §3: with its chain; Cut 14: a home trace's last row is not red. QA 524827b (qaAB: the TRACE
      // replaced the item grid — `‹` back before an item could be kept): the chip unfolds the trace under the chips, in the sheet
      const tr = exitTrace ?? exitLine?.trace;
      const traceBox = h("div", { class: "keep-trace", hidden: true });
      const trace = tr?.turns.length ? h("button", { class: "chip mini", onclick: () => {
        if (!traceBox.childElementCount) traceBox.append(...traceTable(tr, { rows: app.rules.rows, runId, home: p.tier !== "death", provenance: true }, EXIT_TRACE_ROWS));
        traceBox.hidden = !traceBox.hidden;
      } }, /* copy:button */ "decision log") : null;
      // the sheet counts picks against free slots, so its label is `keep 0/1`, not the camp's `vault 1/2` (QA on e0f87e7:
      // "VAULT 0/1 while camp shows VAULT 1/2 · same counter")
      // QA 23ed91f (K: "`$5`, `$4`, `$1` on each item: a cost to keep, or a sale price?" and the report's SALVAGED listed `mapping ·
      // teleport · poison` the sheet never offered): the prices are what an unkept item sells for, and the exit's own cut (a
      // return's share, sold before the sheet: `ExitLine.salvaged`) is named under the chips
      const legend = h("div", { class: "keep-legend dim num" }, unpaid ? /* copy:callout */ "unkept → lost" : /* copy:callout */ "unkept → salvage");
      const cut = exitLine?.salvaged?.length ? h("div", { class: "keep-cut dim num" }, /* copy:label */ "sold", " ",
        exitLine.salvaged.map((r) => `${r.kind.replace(/_/g, " ")}${r.n > 1 ? ` ×${r.n}` : ""} $${r.gold}`).join(" · ")) : null;
      return h("div", { class: "sheet-body" },
        // QA 0c6e126 (qaZ: `KEEP 1/1` after a bought slot, read as "the vault is one slot"): the count is of the free slots — `keep 1/1 free`
        // Cut 29 §4: a full vault's decision is a swap — the pick replaces the vault's weakest (the core's `keep`)
        h("div", { class: "label row-label" }, /* copy:label */ "keep", " ", count, h("small", { class: "dim" }, swap ? /* copy:label */ " swap" : /* copy:label */ " free"), trace),
        chips, legend, cut, bones, news, ledger, traceBox,
        h("button", { class: "btn primary wide", onclick: () => submit(close) }, /* copy:button */ "keep"));
    // blind 1fb7786 (A, twice: "REPORT gem unresponsive … only a reload fixed it"): the backdrop (a tap on the gem under it), Escape or
    // the stud closed the sheet without its `keep`, and the exit flow waited on it for good — every close keeps the ticked picks
    }, { onClose: () => submit(() => {}) });
    function submit(close: () => void): void {
      if (sent) return; sent = true; keepClose = null;
      salvagedRows = letGoRows(p, keep);
      const vaultBefore = new Set(app.lineage.vault.map((v) => v.id));
      app.engine.keep([...keep]).then((L) => { app.lineage = L; keptLabels = L.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.label); }).catch((e) => console.warn("keep", e)).finally(() => { close(); then(); });
    }
  }

  async function init(): Promise<void> {
    let s: Snapshot;
    if (app.seenPending && app.engine.seenSystems) { app.seenPending = false; try { app.lineage = await app.engine.seenSystems(); } catch { /* the glint repeats */ } }   // Cut 29 §2
    try { s = await app.engine.send(); } catch (e) { console.warn("send failed", e); if (!disposed) app.go({ kind: "camp" }); return; }
    if (disposed) return;
    await app.syncWatchLineage();
    if (disposed) return;
    wide.setPresence?.(s, false);
    snap = s; logDepth = s.depth; runId = s.run.id; engineTick = startTick = s.turn;
    // QA a946e04 (T: `start → D5 · $50` at $32 — the run began on D1, no toll, nothing said so): a waystone start the purse could not pay
    // starts on D1, and the watch says so as it opens (the exit's line carries it on: `· from D1 · toll short`)
    { const want = app.lineage.start ?? 1, from = s.run.start ?? (s.depth === 1 ? 1 : want);   // a resumed run deeper down is no fallback
      if (want > 1 && from < want) { tollShort = true; showBanner(/* copy:callout */ `from ${"D" + from} · toll short`, TOLL_BANNER_MS, "rest toll-short"); } }
    for (const e of s.entities) note_(e);
    hudFrom(s);
    // Cut 27 §1: the floors this send folds — the camp's forecast for the rules sent, from the floor the sims started on (a send whose
    // start fell back — a toll short — is not the one the forecast measured: nothing folds)
    { const f = app.forecastOfRules(); startDepth = s.run.start ?? s.depth; if (f && (f.start ?? 1) === startDepth) foldSet = foldFloors(f, startDepth); el.dataset.foldPlan = [...foldSet].join(","); }
    const { viewer: v0 } = await makeViewer(canvas);
    if (disposed) { v0.dispose(); return; }
    // Cut 11 §2: every floor load and event batch is kept in the run log, so the death screen's chain can scrub a replay
    const v = recordRun(v0, runId, s.run.started_turn);
    v.setTacticRows?.(tactics.observedRows, tactics.meaningfulRows);
    syncLook(app); viewer = v; v.resize?.(); v.load(s); el.dataset.frame = frame; fbTick = s.turn; fbAt = performance.now();
    worldT = s.turn; lastPumpMs = performance.now(); scrub.hidden = false; paintScrub(s.turn);   // Cut 14 §6: the world clock starts; the strip shows
    speed = -1; applyFrame(); applySpeed();   // Cut 10 §1: the card and the mode's rate (fights: 16× under it) from the first frame
    if ("__riddle" in window) (window as unknown as { __viewer: Viewer }).__viewer = v;   // dev inspection
    lastHp = s.hero.hp; lastAlert = s.alert; sceneFrom(s); progress.push(s.turn);
    if (foldDue()) void foldRun();   // Cut 27 §1: the watch opens at the first floor below the bar
    pumpTimer = window.setInterval(pump, PUMP_MS);
  }
  void init();
  const onResize = (): void => viewer?.resize?.();
  window.addEventListener("resize", onResize);
  // Cut 14 §6: a hidden tab freezes the picture; back, the world (whose clock ran on wall time) is caught up and the viewer goes live
  const onVisibility = (): void => { freeze(paused, document.hidden); if (!hidden) goLiveOwed = true; applySpeed(); };
  document.addEventListener("visibilitychange", onVisibility);
  return { el, dispose: () => {
    disposed = true; ctl.dispose(); audio.bed(null); clearTimeout(meterTimer); clearTimeout(dockTimer); bar.dispose(); window.removeEventListener("riddle:focus-hero",focusHero); wide.dispose(); if (el.dataset.over === "1") setBusyHost(null); window.removeEventListener("resize", onResize); document.removeEventListener("visibilitychange", onVisibility); clearInterval(pumpTimer); clearTimeout(tickerTimer); clearTimeout(bannerTimer); clearTimeout(counterTimer); clearTimeout(quietTimer); viewer?.dispose();
    if (vaultClose) { const c = vaultClose; vaultClose = null; c(); }
    tactics.dispose();
    if (prepended && !done) void app.engine.setRules(app.rules);
  } };
}
