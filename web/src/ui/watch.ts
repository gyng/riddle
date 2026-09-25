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
// Cut 14 §3 — a bank or a return is a beat: the exit event opens the fight frame on the stairs for SCENE_MS with `BANKED $N` /
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
import type { App, Mounted } from "../app";
import type { Ev, ExitLine, Highlight, InvItem, ReturnReport, Row, Snapshot, StepResult, Trace, VaultChoice } from "../engine/types";
import { h, items, replace, spanOf } from "./dom";
import { gem, paintPortrait, paintSprite, portrait, renderBar, renderConsole, tile } from "./frame";
import { icon } from "./skin";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { setBusyHost } from "./progress";
import { vaultSlots } from "./unlocks";
import { kindGlyph, noteText, verbLabel } from "./tokens";
import { traceChip } from "./trace";
import { exitExtras } from "./death";
import { markEnd, recordRun } from "./runlog";
import { audio } from "../audio";

type Tier = "bank" | "return" | "death";
/** QA 92eb880 (N: the `fights` chip read `1.332247798006322×` over the portrait): a rate as the chip shows it — whole from 2×, one
 *  decimal under it (`1.3`), never a float's tail. */
const CAGE_BATCH = 8, CAGE_NEAR = 10, CAGE_CHAIN = 24;   // Cut 20 §3: short batches chained per pump pass near a cage
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
const SKIP_WALL_MS = 4000;          // QA 23ed91f (L: 60 taps of ▶▶| over a 4-minute stall, 300 s on D1 in `fights`): a skip steps for at most this much wall time, then lands live — the press always moves the picture; the next press goes on
const SKIP_END_BATCH = 100;         // ▶▶| in `fast` steps to the run's end in batches this size: a step's cost is its snapshot, not its ticks (≈ 30 ms a call on the fast wasm build, so 10-tick batches took 15 s to a D5 death)
const SKIP_FIGHT_BATCHES = 12_000;  // Cut 10 §1: ▶▶| steps to the next fight or the run's end (the run cap in BATCHes; ≈ 4 000 ticks
                                    // landed on a paced stretch that looked the same — "inert", three QA players on Cut 12)
const ENDING_TICKS = 20;            // Cut 7 §4: the last ticks before any exit play at 1× (Cut 14: 20, was 30)
const SCENE_FOES = 2;               // Cut 7 §4: awake hostiles in the hero's room that make it a scene
const AMBIENT_MS = 10_000, AMBIENT_SHOW_MS = 1500;   // Cut 7 §4: one ambient callout per 10 s, shown 1.5 s whatever the speed
const PUMP_MS = 25;
const PULSE_MS = 120;               // Cut 14 §6: the strip's dot retriggers its beat at most this often
const CATCHUP_RATE = 32;            // Cut 14 §6: `fast` on the map, behind live by more than the lead: the picture catches up at this rate
const CATCHUP_MAX = 200;
const LEAD_PROBE = 120;             // Cut 18 §1: `fast`'s engine lead — a fight is costed (shown or dropped, as under the card) before the picture meets it
const DEAD_RAMP_MS = 300, DEAD_STEP_MS = 200, DEAD_LAND_MS = 150, FAST_MAX = 128;   // Cut 18 §1: `fast`'s dead-stretch ramp (see `deadRate`)            // Cut 14 §6: the engine's biggest step when the world is behind its clock (a paused or hidden viewer)
type Mode = "fights" | "fast";
const RATE: Record<Mode, number> = { fights: 16, fast: 32 };  // fights: the map when it shows without a hold (draining to an exit); fast: the travel (Cut 12 §6, was 8×; Cut 20 §3: 32× from the first tick, was 16× ramping)
const FAST_NEAR = 4;                // Cut 12 §6: `fast` watches anything near at 2× — Cut 14: 4× ("way too slow")
const FIGHT_RATE: Record<Mode, number> = { fights: 2, fast: FAST_NEAR };   // Cut 14: the fight frame's clock (was 1× · 2×)
// Cut 20 §3 (AD: "early runs at 1× too short to follow" — 22–67 s of card-skipped travel): in `fights` on D1–D3 there is no card —
// the travel plays at EARLY_TRAVEL and every fight at EARLY_FIGHT (the fight is what to follow)
// (dev: `?early=0` turns it off, so the card's own gates can run on the fake's gentle D1)
const EARLY_FIGHT = 1.5, EARLY_TRAVEL = 8, EARLY_TRAVEL_MAX = 64;
const EARLY_DEPTH = (() => { try { const q = new URLSearchParams(location.search).get("early"); return (import.meta.env.DEV || new URLSearchParams(location.search).get("dev") === "1") && q !== null ? Number(q) : 3; } catch { return 3; } })();
const AUTO_FAST = 8, AUTO_TAIL = 10; // a tapped card holds the map at 8×; the pump's "fast" threshold; near holds until AUTO_TAIL ticks after the last sighting / hp change (Cut 14: 10, was 20)
const CALLOUT_MIN_MS = 500;         // Cut 12 §6: a callout stays readable at 16×
const EXIT_GRACE_MS = 4000;         // wait for the viewer to drain after an exit, at most this long
const PERSIST_MS = 5000;
const VAULT_WAIT_MS = 10_000;       // Cut 15 §5: the override sheet holds the world this long at most (wall time); Cut 19 §1: it opens only on a tap on
                                    // the cage beat (10 s, was the default path at 6 s) — then the preference picks; watching it is optional, never a toll
const VAULT_GRACE = 50;             // the core's grace (ticks) between a cage opening and the preference's pick (turn.rs VAULT_GRACE)
const CAGE_WAIT_CAP_MS = 8000;      // Cut 19 §1: the world waits for the cage beat's line at most this long (a beat the playhead never reaches)
/** Cut 19 §1: the cage beat's line — `took mail` (the pick's last two words: a callout is ≤ 3 words). */
export const tookText = (label: string): string => /* copy:callout */ `took ${label.trim().split(/\s+/).slice(-2).join(" ")}`;
const BOSS_BANNER_MS = 3000;        // Cut 2 §7: `boss · counter: known|unknown` on first sight
const REST_BEAT_MS = 1400;
const REST_BEAT_FAST_MS = 700;      // QA e75ec29: in `fast` the rest line holds 0.7 s — `fights` on D1–3 ramps its travel now, and `fast` keeps ≤ 0.4 of it (Cut 20 §3)
const VAULT_FULL_MS = 1200;         // QA e75ec29: `vault full` before the report when the preference kept (no sheet)          // Cut 2 §1: `rest 12m` after the exit, before the exit flow continues
const CHORE_CALLOUT: Record<string, string> = { descend: /* copy:callout */ "descend", pick_up: /* copy:callout */ "pick up" }; // explore never (Cut 4 §4)
const CORE_LINE_MS = 1000, CAPTION_LINE_MS = 1500;   // Cut 18 §2: the renderer's callout and caption lifetimes (render/state.ts)
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
const BEAT_RE = /^(A den\.|A cage:|A shrine\.|The cage opens|A cry from the dark|The air stings|The nest wakes|The den wakes|Found heir \d+'s bones|found the bones|Freed the captive|Cut the captive|Lit the shrine)|snatched|\bstole\b|, gone wild\.$|is avenged/i;
/** The cause of a `hurt` as one word: `goblin_archer` → `archer`. */
const oneWord = (cause: string): string => cause.replace(/_/g, " ").trim().split(/\s+/).pop() ?? "";
/** A player row's callout, ≤ 3 words: `R2 · attack nearest`, `R4 · pack break` (QA 23ed91f, L: `R4 PACK BREAK GOBLIN` — the target goes). */
export const rowCallout = (row: number, verb: string): string => `R${row + 1} · ${verb.trim().split(/\s+/).slice(0, 2).join(" ")}`;
/** The hero's hp lost as a callout: `−3 hp · archer` (QA 23ed91f, K: `−1 rat` read as a kill count, "one rat fewer"). */
export const hurtText = (dmg: number, cause: string): string => /* copy:callout */ `−${dmg} hp · ${oneWord(cause)}`;
/** Cut 13 §3: the automations' purchases at this exit — the ledger's outgoings after the newest exit line that are not salvage
 *  (`−$40 heal potion` → `heal potion ×1 · −$40`), per line text. Empty when nothing was bought. */
export function spentOf(ledger: { t: number; delta: number; why: string }[], cat: { kind: string; label: string; price: number }[] = []): { kind: string; n: number; gold: number }[] {
  let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled)\b/.test(ledger[i].why)) i--;
  if (i < 0) return [];
  const rows = new Map<string, { kind: string; n: number; gold: number }>();
  for (const g of ledger.slice(i + 1)) {
    if (g.delta >= 0 || /^(salvage|wake pay|insure)/.test(g.why)) continue;
    // QA 23ed91f (L: `SPENT heal ×1 · −$80` for a restock that rebought two $40 heals; `leash ×1` on a run where nothing was bought):
    // one ledger line can pay for several at the supply's price, and a restock keeps its word
    const bare = g.why.replace(/^(bought|restock)\s+/, "");
    const kind = /^restock\s/.test(g.why) ? /* copy:label */ `restock ${bare}` : bare;
    const price = cat.find((e) => e.label === bare || e.kind === bare)?.price ?? 0;
    const n = price > 0 && -g.delta % price === 0 ? -g.delta / price : 1;
    const r = rows.get(kind) ?? { kind, n: 0, gold: 0 }; r.n += n; r.gold += -g.delta; rows.set(kind, r);
  }
  return [...rows.values()];
}
/** QA a946e04 (T: `+$71 banked · −$40 spent` while the camp went $51 → $32 — the $50 toll left out): the waystone toll this run's send
 *  paid — the ledger's `waystone D5` lines between the exit before the newest and the newest (the send pays before the run), as a
 *  SPENT row (`waystone D5 ×1 · −$50`). */
export function tollOf(ledger: { t: number; delta: number; why: string }[]): { kind: string; n: number; gold: number }[] {
  const isExit = (g: { why: string }): boolean => /^(returned|banked|died|lost|stalled)\b/.test(g.why);
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

export function renderWatch(app: App): Mounted {
  const canvas = h("canvas", { class: "view" });
  // Cut 17 §1: the hero's hp is the ring around the console's portrait (its numbers on the plate under it)
  const hpText = h("span", { class: "num hp-text" });
  const face = portrait(app, { hp: 1, label: hpText });
  // Cut 16 §4: the boss's bar under the hero's while one is in view (`warlord` + a thin track)
  const bossFill = h("span", { class: "fill" }), bossName = h("span", { class: "name" });
  const bossFace = h("span", { class: "face" });   // art pass: the boss's painted headshot (else its sprite crop)
  const bossBar = h("div", { class: "boss-hp", hidden: true }, h("span", { class: "boss-face", "aria-hidden": "true" }, bossFace), bossName, h("span", { class: "track" }, bossFill));
  let bossFaceKind = "";
  const depth = h("span", { class: "num depth" });
  const alert = h("span", { class: "alert num" });
  const ticker = h("div", { class: "ticker" });
  const stake = h("div", { class: "stake num" });
  const banner = h("div", { class: "banner num" });
  // Cut 17 §1: `⏸ / ▶` is the console's gem (the glyph stays the button's text; the icon is drawn over it)
  const pause = gem({ label: "", cls: "hud-btn", onclick: () => togglePause() });
  // the last chosen mode is the next run's (app.watchMode, persisted — QA on e0f87e7: "`fast` chosen in run 3 was not remembered")
  const mode0: Mode = app.watchMode === "fast" ? "fast" : "fights";
  const modeBtn: Record<Mode, HTMLButtonElement> = {
    fights: tile({ id: "fights", cls: "hud-btn", on: mode0 === "fights", icon: "fights", label: /* copy:button */ "fights", onclick: () => setMode("fights") }),
    fast: tile({ id: "fast", cls: "hud-btn", on: mode0 === "fast", icon: "fast", label: /* copy:button */ "fast", onclick: () => setMode("fast") }),
  };
  const skip = tile({ id: "skip", cls: "hud-btn", icon: "skip", label: "▶▶|", onclick: () => skipToEvent() });
  const bail = tile({ id: "bail", cls: "hud-btn bail", icon: "bail", label: /* copy:button */ "bail", onclick: () => doBail() });
  // Cut 10 §1: the interstitial — the ambient line over the map while the travel runs underneath; a tap holds the map at 8×
  const card = h("button", { class: "interstitial num", hidden: true, onclick: () => holdMap() });
  // Cut 14 §6: the scrub strip — the playhead (the viewer's share of the run) and the frontier's dot (beats per engine batch);
  // Cut 17: along the console's top edge
  const scrubHead = h("div", { class: "head" }), scrubDot = h("div", { class: "dot" });
  const scrub = h("div", { class: "scrub", hidden: true }, scrubHead, scrubDot);
  const bar = renderBar(app);
  const busyHost = h("span", { hidden: true });   // the engine's busy label at the end (the next gem says it): not in the corner
  const cons = renderConsole({ portrait: face.el, tiles: [modeBtn.fights, modeBtn.fast, skip, bail], gem: pause, top: scrub });
  const el = h("main", { class: "watch frame" }, bar.el,
    h("div", { class: "stage" }, canvas, card,
      h("div", { class: "hud top" }, depth, alert, bossBar, stake),
      banner, ticker),
    cons.el);

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
  let heroCause: string | undefined;  // the hero's `die` cause (the client-built report counts the death it came from)
  // Cut 7 §4: the scene's room (null = none) and the tick auto may run fast again after one ends; the ending's first tick;
  // the exit batch held back until the viewer is ENDING_TICKS from the exit; the ambient callout limiter; the last alert
  let scene: number | null = null, sceneUntil = -Infinity, endingFrom = Infinity, endingCue = -Infinity;
  // Cut 8A: the fight frame — whether the engine's latest snapshot holds it, the viewer ticks it spans, the frame shown
  let fightOn = false, fightFrom = Infinity, fightUntil = -Infinity, frame: FrameName = "map", lastBlow = -Infinity;
  // Cut 13 §4: the situation beat the frame is holding for (engine ticks), its text shown once at the cut; beats shown so far
  type Beat = { from: number; until: number; text: string; shown: boolean; exit?: boolean; hold?: boolean; cage?: boolean };
  let beat: Beat | null = null, beats = 0;   // hold: Cut 15 §4, the clock at 1× through it (a boss's kill)
  let exitBeatUntil = 0;              // Cut 14 §3: the exit flow waits while the bank / return beat is on screen
  let holdLineUntil = 0;              // Cut 15 §4: a boss kill's `WARLORD DOWN` keeps the ticker this long
  // Cut 18 §1: the beat on screen holds until this wall time — the frame stays on it (no card, no floor load, the playhead eased to
  // the beat's end or the tick before the stairs), its line keeps the ticker; a beat whose tick comes meanwhile waits its turn
  let beatHoldUntil = 0;
  const beatNext: Beat[] = [];
  let heldBeat: Beat | null = null;   // the beat the hold is for
  const descends: number[] = [];      // engine ticks of the run's descends (a held beat stops short of the stairs)
  let deadSince = -1;                 // Cut 18 §1: `fast` — when the current dead stretch began (wall ms; -1 none): its rate ramps
  let chore: { text: string; n: number; shown: string } | null = null;   // Cut 14 §4: the chore callout streak on the ticker (`pick up ×8`)
  const rowFires: number[] = [];      // Cut 14 §4: this run's `rule` events per row (the death screen's least-fired row)
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
  const loads: { snap: Snapshot; rest: Ev[] }[] = [];   // Cut 14 §6: floors the engine reached that the viewer has not (a queue, oldest first)
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
  let pendingExit: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[] } | undefined;
  // what the exit sheet let go, at the engine's worth — the report's `salvaged` rows (QA on 952e306: "camp $76 after
  // 'Returned with $57'; only the gold sheet shows +$19 salvage"); the deepest floor this send reached (its `deepest` tile)
  let salvagedRows: { kind: string; n: number; gold: number }[] = [];
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
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  const kinds = new Map<number, string>(), names = new Map<number, string>();
  const partyAtStart = (app.lineage.party ?? []).map((c) => `${c.kind} · ${c.name}`);
  const tamedIds: number[] = [], lostIds: number[] = [];
  const compLabel = (id: number): string => { const n = names.get(id); return `${kinds.get(id) ?? "?"}${n ? ` · ${n}` : ""}`; };
  const note_ = (e: { id: number; kind: string; name?: string }): void => { kinds.set(e.id, e.kind); if (e.name) names.set(e.id, e.name); };
  // HUD updates released at the viewer's clock
  const hud = { hp: 0, maxHp: 1, depth: 1 };
  // the snapshot the stake line was last painted from (the card's loot is the stake's), and each floor's rooms / situation as the
  // engine reported them (the card names the HUD's floor, which may be behind the engine's — QA on 56f2a1d: HUD `17/40 D4` under
  // `D5 · 15 rooms · a shrine`, `$6 · keeps $3` under `D2 · 16 rooms · $13`)
  let hudSnap: Snapshot | null = null;
  // Cut 19 §2: a return walks to the up-stairs — once the player's return row has fired (the last row to act, at the viewer's clock)
  // the stake line reads `returning` (its `return at 20%` is spent) until another row acts
  let walkingHome = false;
  const floors = new Map<number, { rooms?: number; twist?: string; biome?: string }>();
  // Cut 16 §4: the boss in view as the HUD shows it (released at the viewer's clock), and each boss's break beat once
  let bossHud: { id: number; kind: string; hp: number; max: number } | null = null;
  const broke = new Set<string>();
  const timed: { t: number; f: () => void }[] = [];
  let lastRuleText = "", lastRuleAt = 0, lastShown = "", lastCoreAt = -Infinity;
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
    face.set(p);
    stake.classList.toggle("warn", p < 0.4);
    replace(hpText, `${Math.max(0, hud.hp)}/${hud.maxHp}`);
    replace(depth, `D${hud.depth}`);
    // QA 23ed91f (K: "`!` / `!!` / `!!!` after the depth label, and `alert 1` / `alert 3`"): one name for one thing — the HUD reads
    // `alert 3`, as the callout does when it rises; nothing at 0
    if (snap) replace(alert, snap.alert > 0 ? /* copy:callout */ `alert ${snap.alert}` : "");
    if (cardUp) paintCardText();
  }
  /** Cut 16 §4: the boss bar — the name one word, the track its hp share; hidden with no boss in view. */
  function paintBoss(): void {
    bossBar.hidden = !bossHud; el.dataset.boss = bossHud ? `${bossHud.hp}/${bossHud.max}` : "";
    if (!bossHud) return;
    replace(bossName, oneWord(bossHud.kind));
    if (bossFaceKind !== bossHud.kind) {
      bossFaceKind = bossHud.kind;
      const k = bossHud.kind.replace(/^boss_/, "");
      if (!paintPortrait(bossFace, `boss_${k}`)) paintSprite(bossFace, `boss_${k}`, 26, k);
    }
    bossFill.style.width = `${Math.round(Math.max(0, Math.min(1, bossHud.hp / Math.max(1, bossHud.max))) * 100)}%`;
  }
  /** Cut 16 §4: the boss in the snapshot's view (not an ally), at its tick. */
  function bossFrom(s: Snapshot): void {
    const b = s.entities.find((e) => e.tags.includes("boss") && !e.ally && e.hp > 0 && !!s.visible[e.y * s.w + e.x]);
    const was = bossHud?.id;
    bossHud = b ? { id: b.id, kind: b.kind, hp: b.hp, max: b.max_hp } : null;
    if (b || was !== undefined) paintBoss();
  }
  function hudFrom(s: Snapshot): void { if (s.depth !== hud.depth) hideBeat(); bossFrom(s); floors.set(s.depth, { rooms: s.rooms ?? floors.get(s.depth)?.rooms, twist: s.floor_twist ?? floors.get(s.depth)?.twist, biome: s.biome }); hud.hp = s.hero.hp; hud.maxHp = s.hero.max_hp; hud.depth = s.depth; deepest = Math.max(deepest, s.depth); paintHud(); paintStake(s); }
  // Cut 2 §7: `$47 · sword⚠ · return at D4`; `death: lose all` when no row would bank or return
  let lastLoot: number | undefined, lastLootRun = -1, lootDrop = 0, lootDropUntil = 0, lootWhy = "";
  let lastPickT = -Infinity, lastStealT = -Infinity;   // engine ticks of the last `pickup` / `steal` (the loot's fall names which)
  let tollShort = false;   // QA a946e04 (T): the send's waystone start fell back to D1 (the purse could not pay the toll)
  let stolenGold = 0;   // QA a946e04 (T: `−$36 stolen` on the strip, nothing in STOLEN): what the run's thefts took off the carry (the `steal` amounts)
  function paintStake(s: Snapshot): void {
    hudSnap = s;
    if (cardUp) paintCardText();
    const st = s.stake;
    stake.hidden = !st;
    if (!st) return;
    // QA 92eb880 (M: "gold `$77 → $65` in the den with only `snatched …` lines"): a fall in the loot shows its size beside it for 2.5 s
    // (`$65 −$12`) — a theft of an item takes its worth with it
    if (lastLoot !== undefined && st.loot < lastLoot && s.run.id === lastLootRun) { lootDrop = lastLoot - st.loot + (performance.now() < lootDropUntil ? lootDrop : 0); lootDropUntil = performance.now() + 2500;
      // QA 1a2a4a9 (O, P: `$46 −$8`, `$283 −$100` — "minuses that don't match any line"): the fall says what took it — a thief, or an
      // item used up (the loot counts what he carries at its worth)
      // QA a946e04 (S, T: `−$32`, `−$20`, `−$13 used` with no line): the carry falls on two things only (the core's `loot_add`): a theft,
      // and a swap — a spare weapon or armour dropped for a find (a pickup; the spare counted more). A use never takes from it.
      lootWhy = s.turn - lastStealT <= 20 ? /* copy:label */ "stolen" : s.turn - lastPickT <= 20 ? /* copy:label */ "swapped" : ""; }
    lastLoot = st.loot; lastLootRun = s.run.id;
    // QA 1a2a4a9 (O: the bar's `$0` and the line's `$3 · death: lose all` on one screen, "neither labelled"): the run's own purse says so
    const parts: (string | HTMLElement)[] = [h("span", { class: "carry-w" }, /* copy:label */ "carry"), ` $${st.loot}`];
    if (performance.now() < lootDropUntil && lootDrop > 0) parts.push(" ", h("span", { class: "loot-drop down" }, `−$${lootDrop}${lootWhy ? ` ${lootWhy}` : ""}`));
    // Cut 6 §1: the kept number while a return/bank row exists (`$84 · keeps $50`)
    // Cut 13 §1: while the guard has fired a stall pays nothing, and the line says so before it is lost (`keeps $0 · stalling`)
    // QA 92eb880 (N: "`$75 · keeps $0 · stalling` held ~10 s, then the run returned with `keeps 60%`"): while the guard has fired the run
    // may still come home keeping its share — the line says `stalling` alone; `keeps $0` is the exit's, once it ends stalled
    // Cut 20 §4 (AC: `carry $78 · keeps $78 · bank R4`, then died with $0): what the exit row keeps names its exit, and what a death
    // keeps stands beside it — `bank keeps $78 · death $0`; `keeps` never alone while a death would keep less (`Stake.death_keep`,
    // the death tier's share; an older core without it keeps nothing on a death)
    const dk = st.death_keep ?? 0;
    const exitVerb = st.return_row !== undefined ? verbLabel({ v: app.rules.rows[st.return_row]?.verb.v ?? "return" }) : "";
    if (st.stalling && !overridden) parts.push(" · ", h("span", { class: "kept stalling" }, /* copy:callout */ "stalling"));
    else if (st.kept !== undefined && !overridden) parts.push(" · ", h("span", { class: "kept" }, /* copy:callout */ `${exitVerb || "exit"} keeps $${st.kept}`));
    if (!(st.stalling && !overridden) && (st.kept !== undefined || st.return_row !== undefined || dk > 0)) parts.push(" · ", h("span", { class: `death-keep${dk > 0 ? "" : " lose"}` }, /* copy:callout */ `death $${dk}`));
    for (const b of st.brought) parts.push(" · ", h("span", { class: b.insured ? "" : "risk" }, b.label, b.insured ? "" : "⚠"));
    // QA 1a2a4a9: the core's `Stake.returning` (a return/bank row acted: the run is committed homeward); the client's own guess (the
    // last row to act was a return) stands in for an older core only
    if (overridden || (st.returning ?? walkingHome)) parts.push(" · ", h("span", { class: "returning" }, /* copy:callout */ "returning"));
    else if (st.return_row === undefined) { if (dk <= 0) parts.push(" · ", h("span", { class: "lose" }, /* copy:callout */ "death: lose all")); }
    else parts.push(" · ", returnAt(app.rules.rows[st.return_row], st.return_row));   // `bank at D9` (the verb again: `death $0 · at D9` read as the death's floor)
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
    if (performance.now() < exitBeatUntil || performance.now() < holdLineUntil) return;     // Cut 14 §3: `BANKED $N` keeps the line (Cut 15 §4: `WARLORD DOWN` too)
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
  /** The beat's line off the ticker (and out of the queue): a floor change, or BEAT_MAX_MS after it showed. */
  function hideBeat(): void {
    if (beatHeld()) return;   // Cut 18 §1: the line holds its wall time
    for (let i = tickerQueue.length - 1; i >= 0; i--) if (tickerQueue[i].cls === "beat") tickerQueue.splice(i, 1);
    if (ticker.classList.contains("beat") && ticker.classList.contains("show")) { ticker.classList.remove("show"); scheduleTicker(); }
  }
  /** Cut 13 §4: a situation's note opens the fight frame for SCENE_TICKS from its tick (or rides a fight already framed there);
   *  its text is the callout, shown once the frame is up. */
  function beatAt(t: number, text: string, exit = false, hold = false, isCage = false): void {
    const v = viewerTick();
    // a frame that is up (or opening) before t carries the beat; a fight the probe dropped (`fightFrom` cleared) does not
    const framed = fightFrom <= t && (fightOn || v < fightUntil);
    if (!framed) { fightFrom = t; fightUntil = t + SCENE_TICKS; fightShow = true; }
    else if (Number.isFinite(fightUntil)) fightUntil = Math.max(fightUntil, t + SCENE_TICKS);
    if (exit) fightUntil = Infinity;   // Cut 14 §3: the run is over — the frame holds; the exit flow's own clock (SCENE_MS, real time) lets go
    // an earlier beat the playhead has yet to reach keeps its place; this one takes over at its own tick (QA on 3d71c33: a den's
    // release showed the later `BANKED $13`, which had overwritten it, on D2 ~15 s before the bank)
    const b: Beat = { from: t, until: t + SCENE_TICKS, text, shown: false, exit, hold, cage: isCage };
    if (!(beat && !beat.shown && beat.from < t)) beat = b;
    el.dataset.beats = String(++beats);   // dev: tools count the beats cut in
    // Cut 18 §1: a beat the playhead jumped over (a skip, a seek to live) is not held after the fact
    at(t, () => { if (b.shown) return; if (!b.exit && viewerTick() >= b.until) { b.shown = true; if (b.cage && cage) cage.done = true; return; } beat = b; showBeat(true); });
  }
  /** The beat's line, once the frame is up and the PLAYHEAD has reached the beat's tick (`reached`: released at the viewer's clock).
   *  A fight cut before it (a kept span the viewer replays behind the frontier) never carries a later beat's line (QA on 3d71c33:
   *  `BANKED $13` over a D2 fight in `fast`, ~15 s before the bank at D5). */
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
    // the picture already past the stop (the stairs' fade applied): back to the beat's tick
    if (!b.exit && viewerTick() > beatStop()) seekTo(Math.max(b.from, beatStop() - 1));
    if (b.exit) exitBeatUntil = now + SCENE_MS;
    tickerQueue.length = 0; showTicker(b.text, b.cage ? "beat cage" : "beat", dur);
    // Cut 19 §1: the cage's line is a plate the finger finds (a tap within the hold opens the override)
    if (b.cage) { replace(ticker, h("span", { class: "cage-line" }, b.text)); if (cage) cage.shown = true; }
    el.dataset.held = "1";
    if ("__riddle" in window) ((window as unknown as { __beatLog?: unknown[] }).__beatLog ??= []).push({ text: b.text, from: b.from, until: b.until, v: viewerTick(), frame, ms: Math.round(now) });   // dev
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
  function releaseBeat(): void { beatHoldUntil = 0; holdLineUntil = 0; beatNext.length = 0; el.dataset.held = "0"; }
  /** Cut 14 §4: a chore's callout (`pick up`) repeats on its own line with a count — `pick up ×8` — until another line shows
   *  (rater T: "dead stretches of eight consecutive `pick up` reads"); the count is repainted in place, never queued. */
  function choreCallout(text: string): void {
    if (chore && chore.text === text) {
      chore.n++;
      const line = /* copy:callout */ `${text} ×${chore.n}`;
      const inPlace = ticker.classList.contains("show") && lastShown === chore.shown && !cardUp;
      chore.shown = line;
      if (inPlace) showTicker(line, "", Math.max(CALLOUT_MIN_MS, 1800 / Math.max(1, speed))); else callout(line);
      return;
    }
    chore = { text, n: 1, shown: text };
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
    if (frame === "fight" && /^R\d+ · /.test(lastRuleText) && now - lastRuleAt < CAPTION_LINE_MS) ruleLine(lastRuleText);
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
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
    if (!broke.has(key)) { broke.add(key); beatAt(t, /* copy:callout */ `${name} BREAKS`, false, true); at(t, () => cue("telegraph")); }
    return true;
  }
  function at(t: number, f: () => void): void { timed.push({ t, f }); }
  function release(upTo: number): void {
    if (!timed.length) return;
    const keep: typeof timed = [];
    // Cut 18 §1: under a held beat nothing past its stop is released (the stairs' HUD change waits for the hold)
    for (const x of timed) { if (x.t <= upTo && !(beatHeld() && heldBeat && !heldBeat.exit && x.t > beatStop())) x.f(); else keep.push(x); }
    timed.length = 0; timed.push(...keep);
  }
  const victims = new Map<number, string>();   // id → label, remembered across batches so a kill inside a batch still has a name
  function absorb(evs: Ev[], s: Snapshot): Tier | null {
    let exit: Tier | null = null;
    const heroId = s.hero.id;
    for (const e of s.entities) { if (e.ally) allies.add(e.id); else if (e.id !== heroId) victims.set(e.id, (e.name ?? e.kind).replace(/_/g, " ")); if (e.tags.includes("boss")) bossIds.add(e.id); }
    for (const ev of evs) {
      if (ev.k === "telegraph" || ev.k === "attack" || ev.k === "use" || (ev.k === "hurt" && ev.id === heroId)) near(ev.t);   // Cut 5 §5: always at 1×
      switch (ev.k) {
        // Cut 10 §3: the core's companion-death callout (`Ashar fell`) gets its kind in front: `jackal Ashar fell`
        case "callout": {
          if (ev.text === "explore") break;
          if (ev.text === /* copy:none */ "choose one") break;   // Cut 19 §1: the cage beat names the pick instead
          if (breakBeat(ev.t, ev.text)) break;   // Cut 16 §4: `warlord breaks` is the beat's, not a plain callout
          // Cut 12 §6: a sanity refusal (`drink ✗ no use`) shows once per floor, not once per streak
          if (ev.text.includes("✗")) { const key = `${ev.text}@${s.depth}`; if (refused.has(key)) break; refused.add(key); }
          let f = ev.text;
          // Cut 12 §6: a summoned ally (no name) reads `ally hound fell`; a companion keeps `jackal Ashar fell`
          if (fell && fell.t === ev.t && fell.kind && ev.text === (fell.name ? `${fell.name} fell` : `${fell.kind} fell`)) f = fell.name ? `${fell.kind} ${fell.name} fell` : /* copy:callout */ `ally ${oneWord(fell.kind)} fell`;
          // Cut 12 §6: the core's `rallied!` names the boss whose telegraph it answers (`warlord rallies`)
          else if (ev.text === /* copy:none */ "rallied!") f = /* copy:callout */ `${oneWord(rallyBy ?? "boss")} rallies`;
          at(ev.t, () => { callout(f, f !== ev.text && f.endsWith(" fell") ? "hurt" : "", f !== ev.text && f.endsWith(" fell") ? FELL_MS : undefined); coreLine(); });
          break;
        }
        case "rule": {
          const text = ruleCallout(ev);
          if (ev.row >= 0) rowFires[ev.row] = (rowFires[ev.row] ?? 0) + 1;   // Cut 14 §4
          // Cut 14 §4: a chore (`pick up`) coalesces on its line with a count; a row's callout repeats after 4 s
          if (text && ev.row === -2) at(ev.t, () => choreCallout(text));
          else if (text) at(ev.t, () => {
            const now = performance.now();
            // Cut 18 §2: a telegraph over the fight has the line — the row's callout goes to the ticker (`rule`, shown in the fight frame)
            if (text !== lastRuleText || now - lastRuleAt > 4000) { if (ev.row >= 0 && frame === "fight" && now - lastCoreAt < CORE_LINE_MS) ruleLine(text); else callout(text); }
            lastRuleText = text; lastRuleAt = now;
          });
          if (ev.row >= 0) at(ev.t, () => cue("rule"));   // Cut 10 §4: a player row, never a chore or a trait
          if (ev.row >= 0) { const home = ev.verb.v === "return"; at(ev.t, () => { if (home !== walkingHome) { walkingHome = home; if (hudSnap) paintStake(hudSnap); } }); }   // Cut 19 §2
          break;
        }
        case "hurt": if (bossIds.has(ev.id)) { const id = ev.id, hp = ev.hp; at(ev.t, () => { if (bossHud?.id === id) { bossHud.hp = hp; paintBoss(); } }); }
          if (ev.id === heroId) at(ev.t, () => { hud.hp = ev.hp; paintHud(); if (ev.dmg > 0) { callout(hurtText(ev.dmg, ev.cause), "hurt", HURT_MS); cue("hit", { dmg: ev.dmg }); } }); break;
        // the kill gets its own line (cohort 5: "−3 goblin" was still up after the goblin had dissolved)
        case "die": {
          if (ev.id === heroId) { heroCause = ev.cause; break; }
          // Cut 10 §3: a companion's death (the snapshot's ally flag) is the core's callout to name; the kill line is for hostiles
          if (allies.has(ev.id)) {
            fell = { t: ev.t, name: names.get(ev.id) ?? "", kind: (kinds.get(ev.id) ?? "").replace(/_/g, " ") };
            // under the card or inside a skip the core's `Ashar fell` line is never watched: the fall waits for the ticker
            const f = fell; at(ev.t, () => { if (skipping || cardUp) petLine(f.name ? /* copy:callout */ `${oneWord(f.kind)} ${f.name} fell` : /* copy:callout */ `ally ${oneWord(f.kind)} fell`); });
            break;
          }
          // Cut 15 §4: a boss's kill is a beat — the frame holds on it with `WARLORD DOWN` (its own line, not `slain`)
          if (bossIds.has(ev.id) && !allies.has(ev.id)) { const id = ev.id; at(ev.t, () => { if (bossHud?.id === id) { bossHud = null; paintBoss(); } }); beatAt(ev.t, bossDown(kinds.get(ev.id) ?? victims.get(ev.id) ?? "boss"), false, true); at(ev.t, () => cue("slay")); break; }
          const v = victims.get(ev.id); if (v) at(ev.t, () => { callout(/* copy:callout */ `${v} slain`, "kill", HURT_MS); cue("slay"); });
          break;
        }
        case "telegraph": if (ev.what === "rallies") rallyBy = kinds.get(ev.id) ?? rallyBy; at(ev.t, () => cue("telegraph")); break;
        // Cut 10 §3: a theft names its amount when the engine sends one (`stolen $16`)
        // QA 1a2a4a9 (P: `12/38` → `6/16`, "nothing in the run said why"): the core's `max_hp` event moves the HUD's max at its tick
        // (the `hunger −1 max` callout comes as a callout of its own)
        case "max_hp": if (ev.id === heroId) { const m = ev.max; at(ev.t, () => { hud.maxHp = m; paintHud(); }); } break;
        case "steal": lastStealT = ev.t; if (ev.amount !== undefined && ev.amount > 0) { const n = ev.amount; stolenGold += n; at(ev.t, () => callout(/* copy:callout */ `stolen $${n}`, "hurt", FELL_MS)); } break;
        case "descend": {
          descends.push(ev.t);   // Cut 18 §1
          floors.set(ev.depth, { ...floors.get(ev.depth), biome: ev.biome });
          const rooms = s.depth === ev.depth ? s.rooms : undefined;   // Cut 7 §4: `D3 · 4 rooms` when the snapshot counts them
          at(ev.t, () => { hideBeat(); hud.depth = ev.depth; paintHud(); ambient(rooms ? /* copy:callout */ `D${ev.depth} · ${rooms} rooms` : `D${ev.depth}`, true); });
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
          if (ev.id === heroId) lastPickT = ev.t;
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
          exit = ev.tier; exitLine = ev.line ?? exitLine; exitTrace = ev.trace ?? ev.line?.trace ?? exitTrace;
          if (exitLine && tollShort && exitLine.start_short === undefined) exitLine = { ...exitLine, start: 1, start_short: true };   // `· from D1 · toll short` on the line
          markEnd(runId, ev.t);   // Cut 11 §2: the run log's last replayable tick
          // Cut 7 §4: the last ENDING_TICKS play at 1×; the core's `ending` marker counts only when the exit follows it closely
          // (Cut 10 §1: a foreseen death the hero survived held the map at 1× for minutes)
          // Cut 14: the marker never holds 1× longer than twice ENDING_TICKS before the exit
          endingFrom = Math.min(endingFrom, ev.t - ENDING_TICKS, endingCue >= ev.t - 100 ? Math.max(endingCue, ev.t - 2 * ENDING_TICKS) : Infinity);
          const tier = ev.tier; at(ev.t, () => audio.cue(tier === "bank" ? "exit_bank" : tier === "return" ? "exit_return" : "exit_death"));           // Cut 10 §4
          // Cut 14 §3: the bank and the return are beats — the fight frame on the stairs, the sum as the callout, before the sheet
          if (tier !== "death") beatAt(ev.t, tier === "bank" ? /* copy:callout */ `BANKED $${ev.loot_kept}` : /* copy:callout */ `RETURNED $${ev.loot_kept}`, true);
          break;
        }
        case "ending": endingCue = ev.t; break;                                                                                       // Cut 7 §4: the core's marker (see `exit`)
        case "tame": if (ev.ok) { tamedIds.push(ev.id); kinds.set(ev.id, ev.kind); allies.add(ev.id); victims.delete(ev.id); const id = ev.id; at(ev.t, () => petLine(/* copy:callout */ `tamed ${compLabel(id).replace(" · ", " ").replace(/_/g, " ")}`)); } break;
        case "ally": if (ev.state === "lost") lostIds.push(ev.id); else { allies.add(ev.id); victims.delete(ev.id); } break;
        case "spawn": note_(ev.e); if (ev.e.tags?.includes("boss")) bossIds.add(ev.e.id); break;
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
  function takeLoad(): { snap: Snapshot; rest: Ev[] } | null { return loads.shift() ?? null; }
  /** Cut 14 §6: load every queued floor into the viewer (the last one is the picture; the ones between were never watched). */
  function drainLoads(): void { if (!viewer) return; for (let p = takeLoad(); p; p = takeLoad()) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); } }
  /** Cut 14 §6: the viewer lands on the frontier (live) — every queued floor loaded, the clock at the engine's tick (the ending's
   *  start at most when the run is over, so the walk-out plays). */
  function goLive(): void {
    releaseBeat();   // Cut 18 §1: landing live lets a held beat go
    drainLoads();
    const t = held ? Math.max(viewerTick(), endingFrom) : Math.max(viewerTick(), engineTick);
    seekTo(t); release(t); letGo(t); applyFrame(); applySpeed();
  }
  /** Land the viewer's clock on tick t (both directions; the placeholder viewer's wall clock too). */
  function seekTo(t: number): void {
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
    const up = mode === "fights" && want === "map" && !mapHeld() && !cageWaits() && !done && !exitTier && viewerTick() < endingFrom && !beatHeld();
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
    const text = way ? /* copy:callout */ `D${d}${title ? ` · ${title}` : ""} · waystone`
      : /* copy:callout */ `D${d}${title ? ` · ${title}` : rooms ? ` · ${rooms} rooms` : ""} · ${twist ? withArticle(twist) : `$${hudSnap?.stake?.loot ?? hudSnap?.loot ?? 0}`}`;
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
  function toEnding(): void { const t = Math.max(viewerTick(), endingFrom); if (t > viewerTick()) { release(t); seekTo(t); } applyFrame(); applySpeed(); }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    engineTick = s.turn;
    floors.set(s.depth, { rooms: s.rooms ?? floors.get(s.depth)?.rooms, twist: s.floor_twist ?? floors.get(s.depth)?.twist, biome: s.biome });
    for (const e of s.entities) note_(e);
    sceneFrom(s, r.events);
    if (s.hero.hp < lastHp) near(s.turn);   // hp lost by any means; a rest's +1 per turn is a dead stretch, a drink is a `use` event
    lastHp = s.hero.hp;
    if (s.alert > lastAlert) { const n = s.alert; at(s.turn, () => ambient(/* copy:callout */ `alert ${n}`)); }   // Cut 7 §4
    lastAlert = s.alert;
    const exit = absorb(r.events, s);
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
      if (di >= 0) { last.rest.push(...evs.slice(0, di + 1)); loads.push({ snap: s, rest: evs.slice(di + 1) }); }
      else last.rest.push(...evs);
      return;
    }
    // entities that appear inside this batch must exist before their events apply (they are not tweened in;
    // the first event they own places them)
    (viewer as Viewer & { preload?: (x: Snapshot) => void } | null)?.preload?.(s);
    if (viewer && di >= 0) { viewer.apply(evs.slice(0, di + 1)); loads.push({ snap: s, rest: evs.slice(di + 1) }); }
    else { viewer?.apply(evs); if (viewer?.sync) { const v = viewer; at(s.turn, () => v.sync!(s)); } } // Cut 4 §3: remembered foes
  }
  /** Cut 14 §6: the world's rate — the viewer's decisions read off the engine's own tick; ≥ 1× while the run is live, 0 once it is over. */
  function worldRate(): number {
    if (done || held || exitTier) return 0;
    if (cageWaits()) return 0;                             // Cut 15 §5 / Cut 19 §1: the world waits on the cage beat and the override sheet
    if (!frozen() && beatHeld() && heldBeat && !heldBeat.exit) return Math.max(0, speed);   // Cut 18 §1: a held beat is the watch's own pacing — the world keeps the picture's pace (no catch-up owed after it)
    if (fightOn && app.slowdowns) return mode === "fights" && engineTick >= slowUntil && !(beat && engineTick < beat.until) && !(foeSpans.length && foeSpans[foeSpans.length - 1].until > engineTick) ? RATE.fights : fightRate();   // Cut 15 §4: a chore stretch at the flat rate
    if (mode === "fights") return RATE.fights;
    return !app.slowdowns || overridden || (engineTick >= slowUntil && engineTick >= sceneUntil) ? RATE.fast : FAST_NEAR;
  }
  /** Cut 14 §6: the strip — the playhead at the viewer's share of the run so far, the dot at the frontier. */
  function paintScrub(v: number): void {
    const span = Math.max(1, engineTick - startTick);
    scrubHead.style.left = `${Math.max(0, Math.min(100, ((v - startTick) / span) * 100)).toFixed(1)}%`;
    el.dataset.frontier = String(engineTick); el.dataset.world = String(Math.floor(worldT));
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
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
      if ((cardUp || viewerIdle()) && !beatHeld()) { const p = takeLoad()!; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }   // Cut 18 §1: not under a held beat
    }
    else if (held) {
      // Cut 7 §4: the clock runs on (8× through dead air) to the ending, then the exit batch plays and the exit flow waits for it
      // Cut 10 §1: under the card the viewer jumps to the ending (the walk-out plays at 1×; the card hides there)
      if (cardUp && now < endingFrom) { release(endingFrom); seekTo(endingFrom); now = endingFrom; applyFrame(); applySpeed(); }
      if (now < endingFrom) return;
      const hb = held; held = null; feed(hb.evs, hb.snap);
      exitTier = hb.tier; exitAt = performance.now() + EXIT_GRACE_MS;
      endControls();
      return;
    }
    else if (exitTier) {
      if (!(viewerIdle() || performance.now() > exitAt)) return;
      release(Infinity);
      nextGem();   // QA 92eb880: the walk-out has played — now the gem says what comes next
      if (performance.now() < exitBeatUntil) return;   // Cut 14 §3: `BANKED $N` has its SCENE_MS first
      // Cut 2 §1: `rest 12m` for a beat, then the exit flow continues. Cut 14 §4: the banner sits low (`.rest`), under the frame's
      // callout line, and the ticker yields to it (rater S: `rest 20m` over `OGRE WINDS UP` on the death frame)
      // QA 92eb880 (N: "`rest 20m` … after `0/36` reads as the hero resting instead of dying"): after a death the rest is the next heir's (`♟2 · rest 20m`)
      if (restS !== undefined && !restUntil) { restUntil = performance.now() + (mode === "fast" ? REST_BEAT_FAST_MS : REST_BEAT_MS); ticker.classList.remove("show"); showBanner(exitTier === "death" && snap ? /* copy:callout */ `♟${snap.run.heir + 1} · rest ${spanOf(restS)}` : /* copy:callout */ `rest ${spanOf(restS)}`, REST_BEAT_MS, "rest"); return; }
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
    // Cut 14 §6: the engine's target — the world clock, or the playing viewer's lead, whichever is further; a world far behind its
    // clock (a paused or hidden picture) is caught up in CATCHUP_MAX-tick steps
    // Cut 18 §1: the ramp's lead (0.2 s of picture); `fast` keeps LEAD_PROBE ahead so a fight is costed before the picture meets it
    const lead = Math.max(speed >= AUTO_FAST ? Math.max(LEAD_FAST, Math.ceil(speed * 2)) : LEAD, mode === "fast" && app.slowdowns && !overridden ? LEAD_PROBE : 0);
    const want = Math.max(worldT, playing ? now + lead : -Infinity);
    if (engineTick >= want) return;
    const gap = worldT - engineTick;
    let n = gap > BATCH_FAST ? Math.min(CATCHUP_MAX, Math.ceil(gap)) : Math.min(CATCHUP_MAX, Math.max(speed >= AUTO_FAST ? Math.max(BATCH_FAST, Math.ceil(speed * 0.6)) : BATCH, Math.ceil(want - engineTick)));
    // QA 92eb880 (N: twice `A cage: three inside, one to take.` with no sheet, in `fast`): a batch longer than the cage's 50-tick grace
    // stepped over the whole choice (the sheet reads the batch's last snapshot) — near an unopened cage the engine steps in short batches
    const short = !!snap && cageNear(snap);
    if (short) n = Math.min(n, CAGE_BATCH);
    inflight = true;
    // Cut 20 §3: near a cage the batches are short (each snapshot is checked for the choice) — they chain up to the pump's target in
    // one pass instead of one short batch a pump (the picture in `fast` stood at the frontier ~2 s waiting for the engine to walk past a cage)
    const target = Math.min(want, engineTick + CATCHUP_MAX);
    const chain = (k: number, left: number): void => {
      app.engine.step(k).then((r) => {
        inflight = false;
        if (!disposed && !done) handle(r);
        if (skipQueued) { skipQueued = false; void skipToEvent(); return; }
        if (short && left > 0 && !disposed && !done && !r.run_over && !r.snapshot.vault_choice && !held && !exitTier && !cageWaits() && engineTick < target && snap && cageNear(snap)) { inflight = true; chain(CAGE_BATCH, left - 1); }
      }).catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; endControls(); });
    };
    chain(n, CAGE_CHAIN);
  }
  /** Cut 10 §1: is the engine free to run ahead under the card — `fights`, the card up and not held, no fight found yet, no exit. */
  function travelling(): boolean {
    return mode === "fights" && cardUp && !cardWait && !mapHeld() && !frozen() && !held && !exitTier && !done && !disposed && !fightAhead(viewerTick());
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
      if (!held) return;
    }
    if (!probe) return;
    const show = held || probe.hurt >= SHOW_HURT || probe.low || probe.boss || probe.ally || probe.steal;
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
    const n = rate();
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
    for (const m of Object.keys(modeBtn) as Mode[]) {
      const want = m === mode && r > 0 ? rateText(r) : "";
      if ((modeBtn[m].dataset.rate ?? "") !== want) { if (want) modeBtn[m].dataset.rate = want; else delete modeBtn[m].dataset.rate; }
    }
  }
  function setMode(m: Mode): void {
    if (app.watchMode !== m) { app.watchMode = m; app.persist(); }   // remembered for the next run
    const wasCard = cardUp;
    mode = m; freeze(false, hidden); mapHold = false; cardLive = false; el.dataset.mode = m;
    // Cut 14 §6: leaving the card (`fights` → `fast`) lands live — the travel under it was the world's skip, not a replay owed
    if (wasCard && m === "fast" && !held && !exitTier) goLive();
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
  }
  /** QA 1a2a4a9: a cage the skip met — open, its beat not yet shown. */
  const cageMet = (): boolean => !!cage && !cage.done && !cage.shown;
  /** The skip lands on the cage's beat: every floor loaded, the viewer at the beat's tick, the frame cut to it (its line holds there). */
  function landCage(): void {
    drainLoads();
    const b = beat && beat.cage && !beat.shown ? beat : null, t = b ? b.from : viewerTick();
    seekTo(t); release(t); letGo(t); cardSince = -Infinity; applyFrame(); applySpeed();
  }
  function paintPause(): void { pause.classList.toggle("on", paused); pause.classList.toggle("pulse", paused); replace(pause, icon(paused ? "play" : "pause"), h("span", { class: "gem-glyph" }, paused ? "▶" : "⏸")); }
  let skipQueued = false;
  function skipToCard(): void {
    skipEarly = true; mapHold = false; cardLive = false;
    goLive(); paintCard(frame); cardSince = -Infinity; applyFrame(); applySpeed();
  }
  async function skipToEvent(): Promise<void> {
    skipping = true;
    try { await skipToEvent0(); } finally { skipping = false; }
  }
  async function skipToEvent0(): Promise<void> {
    if (done || !viewer || exitTier) return;
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
    if (mode === "fights" && earlyFloor() && frame !== "fight" && !stalling()) { skipToCard(); return; }
    inflight = true;
    const until = performance.now() + SKIP_WALL_MS;
    // in `fast` the press means the run's end: the engine steps to `run_over` (floors drained on the way, a vault choice left to
    // its grace) and the viewer lands at the ending, which plays as after any skip (QA on 50bb162: the press "plays faster")
    // QA 23ed91f (L: a summoner stall on D6, ▶▶| and `fast` changed nothing): while the run is stalling the press means the run's
    // end in either mode — there is no fight worth landing on in a loop
    if (mode === "fast" || stalling()) {
      try {
        for (let i = 0; i < SKIP_FIGHT_BATCHES && !held && !disposed && performance.now() < until; i++) {
          // QA 1a2a4a9 (P: at 16× the cage's `took leather +1` was gone in ~1.5 s, untappable): a skip stops at a cage — the beat holds
          // its wall time like any other; near an unopened cage the steps are short, so the choice is met inside its grace
          const r = await app.engine.step(snap && cageNear(snap) ? CAGE_BATCH : SKIP_END_BATCH); handle(r);
          if (held || r.run_over || cageMet()) break;
          const p = takeLoad(); if (p) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
        }
      } catch (e) { console.warn("skip failed", e); }
      inflight = false;
      if (held) toEnding();
      else if (cageMet()) landCage();
      else if (!loads.length) { const t = Math.max(viewerTick(), engineTick - BATCH); seekTo(t); release(viewerTick()); letGo(t); applyFrame(); applySpeed(); }
      if (skipQueued) { skipQueued = false; void skipToEvent(); }
      return;
    }
    // Cut 10 §1: inside a shown fight the press means its end — the span's close is known (the engine ran ahead) or is stepped to
    if (frame === "fight") {
      try {
        for (let i = 0; i < 120 && fightOn && !Number.isFinite(fightUntil) && !disposed && performance.now() < until; i++) {
          const r = await app.engine.step(BATCH); handle(r);
          if (held) break;
          const p = takeLoad(); if (p) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
        }
      } catch (e) { console.warn("skip failed", e); }
      inflight = false;
      if (held) toEnding();
      else {
        // Cut 14 §6: the end of the span the playhead is in (a kept one, or the live fight's), whichever is later
        const v = viewerTick(), sp = spans.find((x) => v >= x.from && v < x.until);
        const t = Math.max(Number.isFinite(fightUntil) ? fightUntil : -Infinity, sp?.until ?? -Infinity, Number.isFinite(fightUntil) || sp ? -Infinity : Math.max(v, engineTick));
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
        const p = takeLoad(); if (p) { viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
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
    paintCard(frame); applySpeed();
  }
  /** Cut 19 §1: a tap on the cage beat's line within its hold opens the override. */
  function cageTap(): void {
    if (!cage || cage.done || vaultClose || !ticker.classList.contains("cage") || !(beatHeld() && heldBeat?.cage)) return;
    vaultSheet(cage.vc, cage.pick?.id);
  }
  ticker.onclick = cageTap;
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
      } }, h("b", { class: "glyph" }, kindGlyph(it.kind)), " ", it.label)));
      // the wait as a shrinking bar (QA on 56f2a1d: "closes by itself ~2–4 s later with no timer"): Cut 15 §5, the wall-clock wait
      // (VAULT_WAIT_MS; the world stands meanwhile), width 100% → 0 over it — `vc.left` is the engine's own grace, after that
      const bar = h("i", { style: `transition-duration:${VAULT_WAIT_MS / 1000}s` });
      graceBar = bar;
      const grace = h("div", { class: "grace" }, bar);
      requestAnimationFrame(() => requestAnimationFrame(() => { if (frozen()) graceHold(); else bar.style.width = "0%"; }));
      // the three-item room is the cage to the player (the core's `twist_word`; QA on 3d71c33: "a sheet titled VAULT")
      return h("div", { class: "sheet-body vault-choice" }, h("div", { class: "label row-label" }, /* copy:label */ "cage"),
        full ? h("div", { class: "vault-full dim num" }, /* copy:callout */ "vault full → sold") : "", chips, grace);
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
    for (const b of [modeBtn.fights, modeBtn.fast, skip, bail]) b.disabled = true;
  }
  function nextGem(): void {
    if (el.dataset.next === "1" || !pause.isConnected) return;
    el.dataset.next = "1";
    const next = exitTier === "death" ? /* copy:button */ "verdict" : /* copy:button */ "report";
    const g = gem({ label: next, cls: "next-gem", pulse: true, onclick: () => { exitAt = 0; exitBeatUntil = 0; restUntil = 1; } });
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
      if (pendingExit && pendingExit.items.length && freeSlots > 0) { const p = pendingExit; pendingExit = undefined; clearTimeout(guard); exitSheet(p, freeSlots, () => { done = false; void finish(tier); }); return; }
      // the vault full: the preference keeps (as designed) — and the screen says so before the report (R: "keep chosen for me?")
      const vaultFull = !!(pendingExit?.items.length && fresh);
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
      // QA a946e04 (T: the vault's dagger became `axe +1` behind a `vault full` flash, no sheet): the preference's swap is named —
      // `axe +1 → vault` — and `vault full` stands alone only when nothing went in
      if (vaultFull) {
        const inVault = app.lineage.vault.filter((v) => !vaultBefore.has(v.id)).map((v) => v.label);
        showBanner(inVault.length ? /* copy:callout */ `${inVault.join(", ")} → vault` : /* copy:callout */ "vault full", VAULT_FULL_MS, "rest");
        await new Promise((r) => setTimeout(r, VAULT_FULL_MS));
      }
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
    // QA a946e04 (S: SALVAGED `potion ×2 · $2` beside `blue potion? ×2`): an unknown item is its flavour (`blue potion?`), as the core's rows name it
    p.items.forEach((it, i) => { if (kept.has(it.id)) return; const k = it.known ? it.kind : it.label; const r = rows.get(k) ?? { kind: k, n: 0, gold: 0 }; r.n++; r.gold += p.worth?.[i] ?? salvageValue(it.kind, p.tier); rows.set(k, r); });
    return [...rows.values()].filter((r) => r.gold > 0);
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
    const stalled = tier === "return" && (/\bstalled\b/.test(exitLine?.text ?? "") || (!!snap?.stake?.stalling && (exitLine?.kept ?? 1) === 0));
    if (tier === "death") for (const c of partyAtStart) if (!lost.some((l) => l === c || l.endsWith(c.slice(c.indexOf(" · "))))) lost.push(c);
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
      elapsed_s: Math.round((engineTick - startTick) / 10), runs: 1, sampled: false, learned, bests, found, pending: [],
      deaths: tier === "death" ? [{ cause: heroCause ?? exitLine?.text ?? /* copy:label */ "death", n: 1 }] : [],   // Cut 10 §3: the death it came from
      stolen: exitLine?.stolen?.length ? [...exitLine.stolen.reduce((m, l) => m.set(l, (m.get(l) ?? 0) + 1), new Map<string, number>())].map(([label, n]) => ({ label, n })) : undefined,   // QA e75ec29 (R)
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap!, tamed, hatched: [], lost,
      xp: { class: cls, ...xpOfRun() },
      salvaged: reconcileSalvage(mergeSalvage(exitLine?.salvaged ?? [], salvagedRows), L.gold_ledger ?? []), deepest, renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
      spent: [...(exitLine?.toll !== undefined ? (exitLine.toll > 0 ? [{ kind: /* copy:none */ `waystone D${exitLine.start ?? L.start ?? 1}`, n: 1, gold: exitLine.toll }] : []) : tollOf(L.gold_ledger ?? [])), ...spentRows(L.gold_ledger ?? [])],   // Cut 13 §3: what the automations bought at this exit (`heal ×1 · −$40`); QA a946e04: the send's toll first
      stolen_gold: exitLine?.stolen_gold ?? (stolenGold > 0 ? stolenGold : undefined),   // QA a946e04 (T): the carry's thefts, beside the items
      banked: tier === "bank" ? 1 : 0, returned: tier === "return" ? 1 : 0, stalled: stalled ? 1 : 0,   // Cut 18 §4: the stall the exit line names anywhere in it (`… · stalled · 1 supply back`), or the stake's flag bones_found: bonesFound,   // rest is still ahead: the camp shows it
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
  const spentRows = (ledger: { t: number; delta: number; why: string }[]): { kind: string; n: number; gold: number }[] => spentOf(ledger, app.supplyCat);
  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: InvItem[]; tier: string; worth?: number[]; auto_keep?: number[] }, free: number, then: () => void): void {
    // QA 23ed91f: the owned automations' picks come pre-ticked (`ExitPending.auto_keep`), as many as the free slots take
    const keep = new Set<number>((p.auto_keep ?? []).filter((id) => p.items.some((it) => it.id === id)).slice(0, free));
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
        } }, it.label, " ", /* QA 1a2a4a9 (P: "`axe ⌂` — what ⌂ means"): a kept pick reads where it goes */ keep.has(it.id) ? h("b", null, "→ ", /* copy:label */ "vault") : h("b", { class: "num gold" }, `$${p.worth?.[i] ?? salvageValue(it.kind, p.tier)}`))));   // the engine's worth at this exit (its old client table read 4×)
      };
      paint();
      // the pile once: the exit line carries `bones: 8 items on D4` (the core's), so the client's `bones left` line only stands in
      // for an exit line without it (two QA players on 50bb162: "bones: 8 items on D4 ... bones left · 8 items")
      const bones = p.tier === "death" && bonesLeft !== undefined && !/\bbones:/.test(exitLine?.text ?? "") ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(bonesLeft)}`) : null;
      const ledger = exitLine?.text ? h("div", { class: "ledger-line num dim" }, exitLine.text + exitExtras(exitLine)) : null;   // Cut 6 §1: engine data, verbatim
      const trace = traceChip(exitTrace ?? exitLine?.trace, "chip mini", { rows: app.rules.rows, runId, home: p.tier !== "death" });   // Cut 9 §5: the trace on a chip; Cut 11 §3: with its chain; Cut 14: a home trace's last row is not red
      // the sheet counts picks against free slots, so its label is `keep 0/1`, not the camp's `vault 1/2` (QA on e0f87e7:
      // "VAULT 0/1 while camp shows VAULT 1/2 · same counter")
      // QA 23ed91f (K: "`$5`, `$4`, `$1` on each item: a cost to keep, or a sale price?" and the report's SALVAGED listed `mapping ·
      // teleport · poison` the sheet never offered): the prices are what an unkept item sells for, and the exit's own cut (a
      // return's share, sold before the sheet: `ExitLine.salvaged`) is named under the chips
      const legend = h("div", { class: "keep-legend dim num" }, /* copy:callout */ "unkept → salvage");
      const cut = exitLine?.salvaged?.length ? h("div", { class: "keep-cut dim num" }, /* copy:label */ "sold", " ",
        exitLine.salvaged.map((r) => `${r.kind.replace(/_/g, " ")}${r.n > 1 ? ` ×${r.n}` : ""} $${r.gold}`).join(" · ")) : null;
      return h("div", { class: "sheet-body" },
        h("div", { class: "label row-label" }, /* copy:label */ "keep", " ", count, trace),
        chips, legend, cut, bones, ledger,
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
    // QA a946e04 (T: `start → D5 · $50` at $32 — the run began on D1, no toll, nothing said so): a waystone start the purse could not pay
    // starts on D1, and the watch says so as it opens (the exit's line carries it on: `· from D1 · toll short`)
    { const want = app.lineage.start ?? 1, from = s.run.start ?? (s.depth === 1 ? 1 : want);   // a resumed run deeper down is no fallback
      if (want > 1 && from < want) { tollShort = true; showBanner(/* copy:callout */ `from ${"D" + from} · toll short`, TOLL_BANNER_MS, "rest toll-short"); } }
    for (const e of s.entities) note_(e);
    hudFrom(s);
    const { viewer: v0 } = await makeViewer(canvas);
    if (disposed) { v0.dispose(); return; }
    // Cut 11 §2: every floor load and event batch is kept in the run log, so the death screen's chain can scrub a replay
    const v = recordRun(v0, runId, s.run.started_turn);
    viewer = v; v.resize?.(); v.load(s); el.dataset.frame = frame; fbTick = s.turn; fbAt = performance.now();
    worldT = s.turn; lastPumpMs = performance.now(); scrub.hidden = false; paintScrub(s.turn);   // Cut 14 §6: the world clock starts; the strip shows
    speed = -1; applyFrame(); applySpeed();   // Cut 10 §1: the card and the mode's rate (fights: 16× under it) from the first frame
    if ("__riddle" in window) (window as unknown as { __viewer: Viewer }).__viewer = v;   // dev inspection
    lastHp = s.hero.hp; lastAlert = s.alert; sceneFrom(s);
    pumpTimer = window.setInterval(pump, PUMP_MS);
  }
  void init();
  const onResize = (): void => viewer?.resize?.();
  window.addEventListener("resize", onResize);
  // Cut 14 §6: a hidden tab freezes the picture; back, the world (whose clock ran on wall time) is caught up and the viewer goes live
  const onVisibility = (): void => { freeze(paused, document.hidden); if (!hidden) goLiveOwed = true; applySpeed(); };
  document.addEventListener("visibilitychange", onVisibility);
  return { el, dispose: () => {
    disposed = true; bar.dispose(); if (el.dataset.over === "1") setBusyHost(null); window.removeEventListener("resize", onResize); document.removeEventListener("visibilitychange", onVisibility); clearInterval(pumpTimer); clearTimeout(tickerTimer); clearTimeout(bannerTimer); viewer?.dispose();
    if (vaultClose) { const c = vaultClose; vaultClose = null; c(); }
    if (prepended && !done) void app.engine.setRules(app.rules);
  } };
}
