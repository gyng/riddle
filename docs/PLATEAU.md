# The plateau — what three cohorts say, and the two moves that are not polish

*2026-09-17. Six blind raters, three decision-grade or tentative cohorts.*

| Cohort | Build | Cards | α | Mean |
|---|---|---|---|---|
| 1 | 6e691ec (Cut 2 + stall) | 62.4 · 64.9 | 0.887 | 63.7 |
| 2 | 186d0ac (Cuts 3–5) | 63.5 · 68.4 | 0.809 | 66.0 |
| 3 | 207cc0e (Cuts 6–7) | 70.8 · 59.2 | 0.716 | 65.0 |

Criterion probes across all six cards: fun 4–5 / 7, play again 4–5 / 7, recommend 4–6 / 10.
They did not move while the axis scores did. That is the rubric's own validity check (§6):
the rubric is measuring what the cuts changed, and what the cuts changed is not what makes a
rater want to come back.

## What moved, and what did not

Moved as designed: story 0.45 → 0.70 (episodes), tension 0.60 → 0.80 (stakes HUD), surprise
0.60 → 0.80 (situations), clarity back to 0.60 (ledger, row accounting). Return, failure and
attribution have been 0.7–0.8 since cohort 1: the death screen and the return report are the
game's strengths and every rater says so unprompted ("the death screen is the best part").

Did not move in three cohorts: **pacing 0.6/0.6/0.45, expression 0.6/0.6/0.45, mastery
0.6/0.6/0.6, feel 0.3/0.3/0.45, aesthetic 0.3/0.45/0.3.** The evidence is the same sentence
six times:

- "20–80 s of *pick up* with nothing moving but the gold counter" (A, B, C, D, E, F)
- "no foe visible in any frame at phone size" (A, B, E); "most of the phone screen is void" (F)
- "hits are a one-line callout, no flash, no screen reaction" (E)
- "the row cap with two defaults leaves little room to be anyone in particular" (F); "every set
  funnels through hp thresholds plus foes ≥" (E); "3 of 7 rows are defaults or patches" (D)
- "the next thing to learn is behind marks I don't have" (F); "never shown" (C)

## Why polish cannot fix these

**The watch.** The renderer draws a faithful roguelike map: 8×8 tiles, a 3-tile hero, a fog
disc, at phone scale. A roguelike map is a *plan view for a player who is making moves*. Our
player is not making moves. What a spectator needs is a **scene**: the foe in frame, the blow
landing, the screen reacting. Auto cadence (Cut 5) and rooms-as-scenes (Cut 7) made the dead
time skippable and the fights findable; they did not make a fight *look like anything*. Feel
and aesthetic are weight-1 axes (the rubric lets them be thin), but pacing is weight 3 and its
in-run half is "is watching worth it", which is the same problem. Three cuts of cadence work
moved pacing by nothing.

**The rules.** Rows 4 → 8 is Cut 2's choice (Stuck In Time's brittle long programs were the
warning). It keeps decisions sharp, and `decisions` sits at 0.7–0.8. But expression is not
decisions: it is *whether the thing I made is recognisably mine*, and a five-row threshold
list cannot be. FF12's expression came from **combos between rows** (Reverse + Decoy) and a
**roster** (who is in the party). Riddle has neither at the player's fingertips: cards are
opaque bundles, companions are rare, classes are locked behind marks the first hour cannot
earn.

## The two moves

### A. The watch becomes a scene (feel, aesthetic, pacing in-run)

Not a new renderer: a **second camera** on the same data. When a scene opens (Cut 7's room
rule), the viewer cuts to a **fight frame**: a close orthographic framing of the room at 2×
env scale (16-px tiles at k=8 on a phone, the hero 6 tiles tall on screen), the foes named
under their sprites, hit flashes, a screen shake on the hero being hurt, a hp bar over each
combatant, the firing row as a caption. When the scene closes, cut back to the map. Two
cameras, one event stream. Dead stretches stay at 8× on the map (they are travel); fights are
*watched* in the fight frame. This is what every auto-battler does (Super Auto Pets, Loop Hero's
combat panel) and what the spectator research asked for ("visible near-misses", "the watcher
must be able to wince").

Cost: renderer 2–3 days (the layers exist; a second camera, a caption layer, hp bars, flash/
shake), client wiring 1 day. Gate: cohort 4 feel ≥ 0.6 and pacing ≥ 0.6 from both raters.

### B. Rows become a roster (expression, mastery)

Keep the row cap. Make the *things in the rows* richer than thresholds, and make the roster
part of the identity:

1. **Combos are real and named.** Two adjacent rows that interact (a stun then a backstab, a
   throw then a step back, taunt then cleave) light up as a *combo* in the editor with a name
   the player did not write (`opener`, `kite`, `bait`), and the chronicle credits the combo
   ("♟3 · the bait-and-cleave fighter"). The engine already resolves the interactions; the
   editor and the chronicle must *say* them. This is FF12's Reverse + Decoy moment made
   visible, and it costs one table of adjacent-verb pairs.
2. **The first hour ends with a second class.** `rogue` is free at first bank (not 4 marks);
   the class sheet's ladders are visible from the start (Cut 5 did the ladders). A second
   class in the first hour is the difference between "a set" and "my set".
3. **A companion in the first hour.** The D1–5 stray (Cut 5) and a starting leash in the
   preset supplies, so a tame happens in most first hours; the companion's own two rows are
   the roster.
4. **Cards open.** A card row expands inline into its rows (Cut 6 gave the sheet; make it the
   editor's default view), so a bought card reads as rows the player could have written.

Cost: core 1 day (combo table, chronicle credit, free rogue, preset leash), client 1 day.
Gate: cohort 4 expression ≥ 0.6 from both, ≥ 0.8 from one.

## What I am not proposing

- More content depth: FULL reaches D29; no rater got past D8. Depth is not the bottleneck.
- More vocabulary: E and F both said the vocabulary is fine and the *shape* is the limit.
- Removing the row cap: it protects `decisions` (0.7–0.8) and the verdict's legibility.
- Tuning: bars are green; the raters' complaints are not about balance.

## Honest ceiling

On this rubric the greats sit at 84–91 (Hades 90.3, Balatro 90.3, Celeste 85.9). Cookie Clicker
and Loop Hero are 74–75. A 95 is above every calibration title; the campaign's realistic target
is "fun" (≥ 72) with no gates, and the two moves above are what stands between 65 and that.
Beyond it, the remaining weight is feel/aesthetic (audio, a world worth sitting in), which is a
third move (sound design, a camp scene) and a different kind of work.

## Cohort 4 (build 1cb869d: fight frame + roster) — the plateau broke

| Rater | Seed | Total | Gates |
|---|---|---|---|
| G | 71 | 71.9 | none |
| H | 83 | 67.6 | none |

α = 0.901. Mean **69.8** (63.7 → 66.0 → 65.0 → 69.8). Criterion probes moved for the first
time: fun 5/5, play again 5/5, recommend 6/6 (cohorts 1–2: 4/4/5). Feel 0.30 → 0.60,
aesthetic 0.30 → 0.60, tension/decisions/attribution 0.80, mastery 0.70. Both raters name the
fight frame and the chronicle unprompted ("the fight frame shouts R1 TAME JACKAL — TAMED!";
"you tame a rat called Skix who then dies").

Remaining lost weight (1.2 each): story, pacing, expression, clarity. Their evidence is now
small and specific (below), not structural.

## Cohorts 5–6 — "fun"

| Cohort | Build | Cards | α | Mean | Verdict |
|---|---|---|---|---|---|
| 5 | be954f5 (Cut 9) | 65.7 · 70.0 | 0.855 | 67.9 | promising |
| 6 | 98b23c9 (watch fix + wake pay) | 74.1 · 75.1 | 0.969 | **74.6** | **fun**, no gates |

Cohort 5 did not move because both raters still saw foes vanish at the hit; measurement found
three renderer faults (a 4-tick corpse under a 0.6 s hit line; entities that appeared mid-batch
unknown to the viewer until the end sync; a foe north of the hero hidden behind the hero's
48-texel sprite in the fight frame) and the fix took attack-callout frames with no drawn foe from
41/198 to 0/99. Cohort 6 then landed both raters in the fun band with the highest agreement of the
campaign. On the same rubric: Loop Hero 74.1, Cookie Clicker 75.5, Into the Breach 83.6, Celeste
85.9, Hades and Balatro 90.3.

Trajectory: 63.7 → 66.0 → 65.0 → 69.8 → 67.9 → **74.6**. Since cohort 1: story 0.45 → 0.80,
expression 0.60 → 0.80, surprise 0.60 → 0.80, tension 0.60 → 0.80, feel and aesthetic 0.30 → 0.60,
progression 0.70 → 0.80. Unmoved: pacing 0.60, clarity 0.60. Criterion probes for both raters:
fun 5/7, play again 5/7, recommend 6/10, check in unprompted 5/7 (cohort 1: 4–5 / 4–5 / 4–5 / 2–4).

## What stands between 74.6 and the greats

The two unmoved load-bearing axes carry the same evidence in every cohort:

- **pacing 0.6**: "~1 floor per minute at 1×, 20–30 s stretches of pick up; fast is ~2×; ▶▶| is
  not an instant skip; a D8 run costs 6–8 minutes of screen time" (K); "no move-up/down, only
  drag" (L). The in-run half is *travel*. The fix that is not polish: travel is not watched at all.
  The watch becomes fights only (the fight frame) with a map *interstitial* between them ("D3 ·
  4 rooms · 40 s") that the player can expand; a run's screen time becomes its fights.
- **clarity 0.6**: "rest 20m never explained", "heal potion $40 greyed out with $165", "1 RUNS · 0
  DEATHS right after a death", "Ashar slain read as a foe when it was my jackal", "BANKED 0 beside
  fourteen $61 return lines", "3 over never explained", "the patch's position hid the real answer".
  Each is one line of copy or one number; none is structural.
- **mastery 0.7**: "the warlord is a wall with no ramp: D9 0% on every configuration except the
  one I found" (K). The counter is discoverable but the forecast reads as a cliff. A forecast row
  that names the *known counter row* when the shown depth is a boss floor and the set lacks it
  (`D9 0% · warlord · try: attack boss`) turns the wall into a ramp.

Those three, plus audio (feel/aesthetic are weight 1 and have no sound at all), are the next cut.

## Cohort 7 (build a7c47e9: Cut 10 — fights mode, the wall as a ramp, clarity rows, sound)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| M | 131 | 76.8 | none |
| N | 137 | 74.1 | none |

α = 0.878. Mean **75.5** (cohort 6: 74.6). Nine axes at 0.8 (decisions, surprise, story,
failure, progression, return, attribution, mastery for one rater, pacing for one, clarity for
one). Nothing below 0.6. Criterion for both: fun 5, play again 5, recommend 6, check in 5.

Trajectory: 63.7 → 66.0 → 65.0 → 69.8 → 67.9 → 74.6 → **75.5**.

### The honest read

Every load-bearing axis has now been awarded 0.8 by at least one blind rater, and the 0.8
anchor ("strong throughout, one named lapse") is where a rater lands when the thing is *good but
not the best they know*. The next anchor, 1.0, requires "no material lapse across the horizon and
a named genre benchmark does not do it better". No rater has awarded a 1.0 on any axis, and the
calibration greats (84–91) are games that earn several. The step from 75 to 85 is not the sum of
more one-line fixes; it is an axis where Riddle becomes the reference title. The two candidates,
from what raters *praise* unprompted in every cohort, are `failure` ("the death screen is the
game") and `attribution` ("every death is yours and it tells you why"). Those are already 0.8
everywhere and never 1.0 because of the same named lapse: **the decisive cause is sometimes hidden**
(M: "'R2 no item' while 'A thief snatched the heal potion' lives only in the morgue"; N: "monkeys
stole my heal potions for five runs and the trace only ever said 'R1 no item'"). A death screen
that traced the *chain* (the theft three floors earlier that emptied the potion slot the drink
row needed) would be the thing no other roguelike does.

The weight-1 axes (feel, aesthetic, autonomy at 0.6) are worth 2.7 points each in total; even
at 1.0 they add ~3. The remaining ten points live in the 0.8 → 1.0 step on the weight-3 axes.

## Cohort 8 (build eec0f3f: Cut 11 — the death screen traces the chain)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| O | 149 | 75.1 | none |
| P | 157 | 72.4 | none |

α = 0.877. Mean **73.8** (cohort 7: 75.5). Failure and attribution 0.8 from both; the chain was
praised by both unprompted (O: "a because-link whose WATCH replay showed my own rule drinking the
potion — you always know whose fault it was"; P: "a per-rule why-not, a WATCH link and
odds-labelled patches"). The cohort 8 gate (a first 1.0) was not met.

Trajectory: 63.7 → 66.0 → 65.0 → 69.8 → 67.9 → 74.6 → 75.5 → **73.8**.

### The second plateau, and what it is made of

Four cohorts at 74–76 with α ≥ 0.85. The lost weight (9.7 of 37) splits three ways:

1. **expression 0.6 · 0.6, every cohort** (1.2 lost, the largest single item). O: "two rows are
   the game's cards and two are its defaults; the patch list steered me toward the same shapes
   every death"; P: "three of six rows are prefab cards and only two are my own sentences". O's
   first gripe: "rows are the choke point — every patch tap lands as `7/6 · drop one`, +1 row
   climbed to ◆11, cards I bought auto-insert as rows". The roster (Cut 8B) made rows *readable*
   as identity; it did not make room for the player's own sentences.
2. **Nine axes at 0.8 · 0.8 with a named lapse each** (5.4 lost). This cohort's lapses are, by
   count, *defects*: the first send after a night resumed a run the batch had left at D2 with an
   empty pack (P: "`no item ← never found` beside 5/5 supplies"); a patch offered `foe: boss →
   attack boss` while it sat at R2 (O); cards insert at the bottom below `attack nearest` where
   they never fire (P) and re-adding one glues an uneditable `hp < 50%` onto it (P); `+1 row ⊘
   rows full` stays dimmed at 4/4 (P); `▶▶|` inert on four tries (P); the swap chore dropped a
   bought heal for a poison (O: "no row of mine could touch it"); the thief guard card does not
   stop a den (O). Two of these are fixed already (the night's run, the duplicate patch). The
   1.0 anchor is "no material lapse across the horizon"; a build that ships with seven defects a
   rater meets in an hour cannot earn it on any axis, whatever its design does.
3. **The middle floors repeat** (surprise 0.6 for O, tension 0.6 and pacing 0.6 for P): "the same
   D3 den, the D6 archer corridor four times, `jackal · D4` killed heroes 1 and 2 identically";
   "runs 4–8 repeated the D4–D6 archer/jackal loop with near-identical `returned $NN` endings".
   And the night: "rested 287m — the hero idled for five of the eight hours" (P).

### The read

Cut 11 was the right feature and it was found and praised; it could not move an anchor because
the anchor above 0.8 is not "better feature" but "no lapse". The campaign's method (cut → ship →
blind cohort) finds lapses *after* shipping, two raters at a time. The next cut adds a step:
**a QA cohort plays the rater protocol on the build before the blind cohort does**, files lapses
only (no scores), and the build ships to raters only when that list is empty. Alongside it, the
one structural item still at 0.6 from every rater: the player's rows are theirs, and the game's
(cards, defaults) do not crowd them out.

## Cohort 9 (build 39def99: Cut 12 — your rows, no lapse; two QA rounds before it)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| Q | 401 | 75.1 | none |
| R | 402 | 72.4 | none |

α = 0.938 (the highest on record). Mean **73.8** — the same number as cohort 8. Nine axes at
0.8 from both (decisions, mastery, surprise, story, failure, progression, return, attribution;
clarity and tension 0.8 from Q only); expression 0.6 · 0.6 for the fifth cohort. The Cut 12 gate
(mean ≥ 78, expression ≥ 0.8 from one rater, any axis at 1.0) was not met.

Trajectory: 63.7 → 66.0 → 65.0 → 69.8 → 67.9 → 74.6 → 75.5 → 73.8 → **73.8**.

### What the two QA rounds bought, and what they did not

The QA cohort worked as a lapse-finder: 131 lines on the first build, 52 defects on the reship,
every one closed before the raters played (`eval/qa/*.triage.md`). The raters' cards carry none
of cohort 8's defects, and the praise is specific ("every death tells you exactly which line you
wrote killed him"; "the forecast answers every edit in three seconds"; "a locked 60 fps watch").
The number did not move because the lapses the raters name are not defects — they are one
mechanic and three designs:

1. **Stalls** (Q: clarity, mastery, tension, failure, return, attribution; R: failure,
   attribution — six axes name it). "Three of 16 overnight runs stalled and paid $0 on
   $255/$100/$82 carried"; "kept 0 % of $134 … contradictory, punitive, and not a stake I
   chose". Measured after the cohort: the DEFAULT set stalled on 5.2 % of sends; three engine
   loops caused nearly all of them (a chore pathing to an item the full pack would never take, a
   guard's ignore lifted by the ignored foe's own arrow, the coward's retreat streak resetting
   every other action). Fixed in cffa2d9: 0.5 %. The forfeit itself stays (a gated invariant).
2. **Traits overriding rows** (attribution, both: "R4 retreat — brave held", "curious drank heal
   at 24/36 hp"; Q's build description ends "the hero's trait can override your rule"). Legible,
   and still a loss the player cannot own.
3. **The night's economy is opaque** (progression, return: "I left with $142 and came back to
   $6 and the report never said where it went"; "$2131 after the night had almost nothing to
   buy"). The restock's spending is only in the ledger; the strength potion the trait drinks at
   full HP is rebought sixteen times.
4. **The beats are not on screen** (story, failure, pacing): "the thieves, 'Grul is avenged',
   'A vault: three under a cage' live only in the reel or the morgue"; "the fights mode skipping
   whole floors"; "death 2's real cause ('A thief snatched the heal potion') was only in the
   morgue text". And the forecast's noise (decisions, both: "±10 between re-rolls", "±15 made
   some comparisons coin flips"; mastery R: "'95 % bank' then 'spectral blade · dice'").

### The read

The QA step is now part of the method (it removed the defect class from the cards). What is
left at 0.8 is design, and the four items above are the next contract's — the stall mechanic
first, because it is the one thing both raters would tell a friend about *against* the game.

## Cohort 10 (build 2cb9e88: Cut 13 — no stake the player did not choose; a mechanical QA round before it)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| S | 601 | 72.4 | none |
| T | 602 | 72.4 | none |

α = 0.755 (two cards; tentative). Mean **72.4** — the lowest since cohort 6. Neither card's
gate was met (mean ≥ 78, attribution or failure at 1.0 from one rater, α ≥ 0.80). The two
totals agree and the axes do not: S is at 0.6 on clarity, tension, feel, pacing; T is at 0.8 on
all four and at 0.3 on feel and aesthetic. Attribution and failure hold 0.8 · 0.8 for the
third cohort; expression 0.6 · 0.6 for the sixth.

Trajectory: 63.7 → 66.0 → 65.0 → 69.8 → 67.9 → 74.6 → 75.5 → 73.8 → 73.8 → **72.4**.

### What Cut 13 bought, and what it did not

The stall verdict, the chosen trait, the ledger's SPENT and the beats on screen all landed
without a lapse: no card names a trait override, a missing night's gold, or a beat the reel
kept to itself; both retell a run in the first person ("Zelak the Goblin Captain is avenged"
after S's summoner row; "Ashusk the ogre is avenged" two heirs after the ogre killed T's heir
5). The QA round before the cohort held (22 wire invariants, 95 screen checks, two QA
players): the cards carry no reconciliation lapse. What kept the number where it is:

1. **Stalls, still** (T: clarity, tension, pacing, failure, attribution — five axes; "three
   stalls ended runs by fiat"; "every row marked `stuck ← paced 12 turns, foes ignored`, which
   I could not fix or explain"; "30–50 s stalling loops eat a third of some watches"). The
   verdict screen exists now and T read it three times; it did not help, because the loop was
   not a missing row. Reproduced after the cohort on seed 602 with T's own set: the den's
   monkeys circle the hero on the D4 stairs, `foes ≥ 2 → to corridor` fires on alternate
   actions and the descend chore steps back each time, so the hero never gets an action *on*
   the stairs. Fixed in 10b59b9 (the step that lands on the stairs takes them in the same
   action, after the den's pounce): 24 sends, 0 stalls; DEFAULT 0.0 % over 462 sends. The
   lesson is the one cohort 9 taught: a stall is an engine loop until proven otherwise, and
   the verdict's patches cannot name an engine loop.
2. **The forecast's noise** (S: clarity, decisions; T: decisions — "the same six rows read
   94/6, then 79/21, then 72/28"; "±22 … a 10-point edit is unreadable"; a patch offered
   "worse than base (survives 42 % · base 50 %)"). §5 put the `±` on the wire; the raters
   read the swing anyway, because 100 sims on a set whose runs end D3–D8 have a half-width of
   ±10 and consecutive edits re-seed. The number itself has to settle: same seeds across
   edits (paired sims, so an edit's delta is a difference, not two draws), and a patch below
   base is not offered.
3. **Feel and aesthetic** (S 0.6 · 0.6; T 0.3 · 0.3 — "in ten screenshots of fights I never
   saw a foe sprite"; "five foes on one tile rendered as one smear"; "a black screen with a
   lit patch of olive tiles and one well-drawn hero"; "the bank moment is a sheet, not an
   event"). The fight frame reads through callouts; the render does not carry the fight. This
   is `PLATEAU.md` move A again, unpaid since cohort 4: foe sprites at the hero's scale, a
   stack that fans, the map lit beyond the hero's view, the bank as a beat.
4. **The preset's first death** (S: clarity, failure, attribution — "death 1 read as a rule
   gap (7 unknown unused, heal row inert) but was stamped `dice`"). The dice fallback (Cut 11
   §4) is right that no candidate row survived; it is wrong to say `dice` when the screen's
   own margin line names eight unused unknowns. A death with `N unknown unused` and a drink
   row that never fired is a `gap` (`drink unknown`), whatever the replays say.
5. **Expression** (both 0.6: "after an hour the set is what any fighter converges on"; T:
   "only 3 of 6 rows are mine; the rows the game wrote are the ones that fire most"). Sixth
   cohort at 0.6; the patches and cards win the set. Move B's roster is in; what is missing
   is a reason to diverge (a second class played, a card that changes the archetype).

Smaller, all fixable in a round: the report's TRACE buttons unlabelled in a 16-row list under
the salvage table (S); `6/5 · drop one` after an offered patch with no warning (S); L1 runs
30–100 s, "too short to watch for minutes" (S); eight consecutive `pick up` reads (T);
D1–3 watched from full hp on every run (T); the `rest 20m` overlay over the telegraph line
at the death frame (S); audio unassessed by both (`window.__audio` null on the production
build — the probe hook is dev-only).

### The read

The mechanic both raters named first in cohort 9 is at 0.0 % on the sets we can measure, and
was still the first thing T named — because T's own set found a loop the gate's sets did
not. The gate now carries the cohort's sets, and the next cohort's sets go in after it. The
remaining points are two designs (a settled forecast; the fight the render can show), one
verdict rule (`gap` when unknowns sat unused), and the sixth 0.6 on expression, which no
polish has moved and which the next contract has to address with content, not chrome.

## Cohort 11 (build 8231993: Cut 14 — paired forecast, the fight on screen, the world on the wall clock)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| U | 701 | 66.5 | none |
| V | 702 | 70.8 | none |

α = 0.897 (reliable). Mean **68.7** — down 3.7 from cohort 10. Both raters' sets stall 0.0 %
(added to the gate table). Decisions, failure, return and attribution hold 0.8 from both;
**progression falls to 0.6 from both** (0.8 in cohorts 9 and 10), clarity 0.6 · 0.6, and
expression is at 0.6 for the seventh cohort.

Trajectory: … 73.8 → 73.8 → 72.4 → **68.7**.

### The read

Cut 14 did what it said (no card names forecast noise, a clipped caption, an unlabelled
trace, a stall), and the number fell, because the next layer down is now visible and both
raters found the same one first:

1. **The economy has no pull** (progression, tension, return — both). U: "bail at 2/38 hp →
   home with $103 in ~3 s, zero risk"; a return row made all 16 night runs come home; "returns
   paid ◆0 while deaths paid ◆+7/+8, so playing safe stalled the unlock track"; "$600+ and
   nothing worth buying". V: "$1488 banked with nothing worth buying"; restock "heal ×32 ·
   −$1280" overnight; "+1 row costs ◆11 and most returns pay ◆0". Marks come from new depths,
   first kills and trophies — a safe set earns none; gold buys supplies only. The idle loop's
   two currencies do not meet a decision.
2. **Ownership leaks through the Cut 14 patch rule** (U, V): `↑ R3` replaced "my D8 bank
   row" / "my drink unknown row" silently. The least-fired row is not the least-wanted row.
3. **The watch still hides its peaks** (feel, pacing): the Warlord kill "only as ticker text in
   a few seconds"; many 1× frames are the black floor card; name tags collide (`CAPTIMONKEY`);
   11 s of `pick up`; the speed chips never show the rate.
4. **Companions and cages do not matter** (V): pets fall early as one line; the cage sheet
   "expired before I could tap it every time".
5. Smaller: `hazard first` on every row of a death footer; `R1 retreat caught him.` on a run he
   survived; `survives 0% · base 0%` still shown (dimmed, below bar); a death U caused with a
   chase rule stamped `dice`; the forecast's bank % above what late runs delivered.

Next contract: the economy first (a bank/return that earns marks, gold with a sink that
changes play, a restock that asks), then patch ownership, then the watch's peaks.

## Cohort 12 (build 238bd67: Cut 15 — frontier marks, gold buys, patches ask, the watch's peaks)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| W | 901 | 69.2 | none |
| X | 902 | 66.5 | none |

α = 0.859 (reliable). Mean **67.9**. Progression back to 0.8 from W (0.6 from X); decisions,
failure, return, attribution hold 0.8; everything else 0.6 for both.

Trajectory: … 73.8 → 73.8 → 72.4 → 68.7 → **67.9**.

### The read

Three cohorts down. Each cut closed what the last cards named, and the cards moved to the
next layer without the total moving up; the lapse-driven loop has stopped paying.

1. **A defect that hid in plain sight** (W): the forge put `heal potion` on the shelf before
   heal was identified; `drink heal` read `unknown item` all night and 28 found heals were
   salvaged. Fixed after the cohort (1951fe7).
2. **The Warlord fight loops** (X: ~7 min of `shields up · warlord rallies · goblin slain`
   at 2× with heal potions cycling hp 11 ↔ 30; QA J: 4+ min). X's set stalls 2.3 % in the
   gate table (the cohort-set gate caught it on its first run).
3. **Gold still has no pull** (X: $2486 with only supplies to buy; restock spent $2400 of the
   night's $3090 without asking). Gold buys exist but read as a hunch on a 35-tile wall.
4. **Tension leaves once a safe set is found** (X: no deaths in 20 runs after the rest patch;
   W: 0 deaths in 16). A set that returns at 10 % hp never dies.
5. Unexplained walls (`D9 0% · fire`), classes bought with nowhere to use them, the stale
   `counter: attack boss` banner, repeated `avenged` lines, cages still expiring (W: three).

What the four 0.8 axes share — decisions, failure, return, attribution — is the core loop:
write a row, read the forecast, read the death. What is stuck at 0.6 for every cohort —
feel, aesthetic, pacing, expression, autonomy, clarity, surprise — is the game around it:
what the watch looks like, what there is to want, how a run differs from the last. The next
cut cannot be another lapse list; it has to change what a night *is* (a reason to want the
second class, a floor that is not the Warrens again, a boss that is a fight, not a loop).

## Cohort 13 (build 32971ad: Cut 16 — freshness, a class at the wake, the Burrows, the Warlord's break; Cut 17 — the frame)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| Y | 1101 | 67.3 | none |
| Z | 1102 | 68.1 | none |

α = 0.863 (reliable). Mean **67.7** — flat against cohort 12 (67.9). Aesthetic 0.6 · 0.6 and feel
0.3 · 0.6: the new frame did not move either. Decisions, failure, return, attribution hold 0.8
from both; progression 0.8 (Y), story 0.8 (Z).

### The read

The frame is not what the raters look at for an hour: they look at the watch. Both name the
watch's timing and legibility first, and neither mentions the chrome except the death screen's
`APPLY` sending the heir at once.

1. **The watch swallows its peaks and holds its dead time** (feel, pacing): `GOBLIN WARLORD
   DOWN` "went by in about a second" in `fights`; every cage holds ~30 s (Cut 15 §5's wait) —
   "about 90 s of one 5-minute watch"; `fast` read slower than `fights`; monsters stack on one
   tile and "the warlord sprite hid my hero completely"; stacked callouts hide the fight.
2. **Walls without a reason** (clarity): `D9 0%` for every set "with no reason given, until I met
   the Goblin Warlord" — the forecast knows the boss seals D8 and does not say so.
3. **Contradictions** (clarity, failure): `drink ✗ no item` with `heal potion ×4` in FOUND; the
   stall screen blaming a jackal while its trace shows the rater's gas-retreat ping-pong; a
   stall keeping 0 % beside a death's $40 wake pay; `RUNS 1 · DEATHS 0` after a stall; `APPLY`
   inserting at R1 above drink heal and sending the next heir at once.
4. **An inert middle** (decisions, expression): every card reads `reach ~0`; gold idles at $2016
   (the gold price is on the unlock sheet, not on the tile); bank dominates return.
5. **Sameness** (surprise, pacing): D1–D7 the same mix on the same tiles; nights of 13 × `D8 ·
   BANKED` or `returned $31–55`.

Structural note: the anchors cap an axis at 0.8 unless the rater saw "no lapse across the
horizon"; every card names at least one lapse on every axis it rates 0.8, so the totals have
lived in 63–76 for thirteen cohorts. 95 needs 1.0 on most axes — a watch, a forecast and a night
with *no* lapse an hour-long player can find. The next cuts have to close whole axes, not lists.

## Cohort 14 (build 205d408: Cut 18 — the watch keeps its peaks; no contradiction on a screen)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AA | 1301 | 69.2 | none |
| AB | 1302 | 69.7 | none |

α = 0.808 (reliable). Mean **69.5** — up 1.8, the first rise in five cohorts. Feel back to 0.6
from both (0.3 · 0.6 in cohort 13); mastery 0.8 (AA). Neither card names the watch's timing,
the hidden hero or a wall without a reason: Cut 18's items landed.

### The read

1. **The cage is a decision the player does not get to make** (AA, AB: "closes before a tap
   lands"; AA: the vault's `cage` setting "raised bank-at-D7 from 54 % to 90 %, more than all my
   rule edits", found at ~25 min). The biggest lever in the game is a hidden preference.
2. **A safe set has no stakes** (AB: `hp < 20% → return` → death 1 %, a night of 0 deaths,
   banked at D12 every run). A return is an instant exit at 60 %: no risk between the rule and
   home.
3. **Admin**: the same three supplies re-bought every send (AA); restock spending more than the
   night brought home without a word (AB); `+1 row` offered then gone.
4. **The death screen still argues with itself**: GAP stamped on a death where the player's
   own row fired (AA); `+ drop one` without naming the row; a 33 % patch ranked above two 100 %
   ones after the reach re-rank (AA).
5. **Repetition**: the monkey steals nearly every run; `Zeleth … is avenged` three times; every
   card `reach ~0`.

## Cohort 15 (build fba365b: Cut 19 — the cage at camp, a return that walks home, admin gone; the art passes; two QA rounds)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AC | 1501 | 74.6 | none |
| AD | 1502 | 74.1 | none |

α = 0.969 (the highest on record). Mean **74.35** — **+4.85**, both verdicts "fun". Ten axes at 0.8
from AC (clarity, surprise, story and **aesthetic** among them — the art pass registered), eleven
from AD; tension, feel, expression, pacing (and autonomy for AC) hold 0.6 for both.

Trajectory: … 72.4 → 68.7 → 67.9 → 67.7 → 69.5 → **74.35**.

### The read

1. **Thieves are a tax** (both, first): "monkeys and dens steal the $40 potions I buy in nearly
   every run"; "the den wakes … on nearly every run and took the same gear each time". Cut 19's
   thinning only follows a lineage that *lost* to a den; a lineage that is robbed and lives is
   robbed again.
2. **Pets die nearly every run** (AD: 8 pets lost in a session): taming reads as a tax.
3. **Money lines that don't add up** (AC): `carry $78 · keeps $78 · bank R4` then died $0 (the
   stake says what the bank row would keep, not what a death keeps); a silent repeat charge on
   death; a patch promising `reach D7 +20%` that read D7 0 % once applied.
4. **Speed**: `fast` "barely faster than 1×" (both); early runs at 1× "too short to follow"
   (AD); the forecast settling 4–8 s after each edit (AD).
5. **Stakes after the absence** (AC: 15 of 16 banked; the second half had little at stake).

## Cohort 16 (build 44193ac: Cut 20 — thieves and pets untaxed, the watch moves, honest money lines, the bounty)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AE | 1701 | 73.0 | none |
| AF | 1702 (unpinned: the brief lacked `dev=1`; fixed 3de6979) | 73.5 | none |

α = 0.938. Mean **73.25** — flat against 74.35; both "fun". Nine axes at 0.8 from AF (aesthetic
among them), tension, feel, expression, pacing, autonomy (and mastery, AF) at 0.6.

### The read

The taxes Cut 20 went after are gone from the cards (no pet complaints; thefts one line each).
What both name first now:

1. **The first floors are the same every run** (AE: "D1–D7 play out almost the same each time:
   monkey theft, cage, the jackal gone wild, shrine — the first minute of every watch"; AF:
   "D1–5 repeat the same beats every run"; "60 s of `pick up ×N` on D1–3"). The lineage reaches
   D13–D20 and still starts every run at D1.
2. **The economy undercuts the player** (AE: "sells heal potions he finds for $2 while I pay $40
   each"; a strength potion no rule drinks re-bought 16 times overnight; AF: "every run netted
   about $0 after the absence"; a stall charged the repeat on top of the loss).
3. **Verdicts and words**: GAP on a death the player's own drink-unknown row caused (AE); `picked
   clean`, `restock capped` unexplained; the depth picker stops at 12 (AE banked at D20).
4. **The trace explains only the last tick** (AF: why `hp<30% → bank` never fired over six ticks
   at 9/42).

## Cohort 17 (build 6da3ed0: Cut 21 — waystones, supplies not a leak, words and verdicts)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AG | 1901 | 70.3 | none |
| AH | 1902 | 71.9 | none |

α = 0.968. Mean **71.1** — **down 2.15**. Both "promising".

### The read

The last three cuts each added a sink to the gold loop, and together they made it a treadmill:
the repeat restock (Cut 19) re-buys a $40 heal every send, the waystone toll (Cut 21) charges
per start, and a D1 thief takes the heal just bought (AH: "about 6 times in 18 runs, sometimes 8
seconds after I paid $40"). A safe run brings home $50–80 at a 60 % return: AG "+$49 returned ·
−$120 spent"; AH "−$170 spent against +$47 returned"; both "a treadmill". Progression and
tension both read it.

1. **The economy is a treadmill** (both, first).
2. **Late edits sit inside the forecast's noise** (AH: "most edits moved the forecast less than
   its ±10–13 error"); AG: "the forecast feels like a solver".
3. **The verdict still misses own-row deaths** for attack rows (AH: R4 `attack ranged` chasing an
   archer through six foes, sealed GAP).
4. **D1–D4 still repeat** (AG); the waystone only helps from D5.
5. **Misleading numbers**: `D9 · bank +3%` beside death 61 % (AG); a delta read as a chance.

## Cohort 18 (build afa0eed: Cut 22 + QA U/V — the gold loop pays, free waystones, bloodless-dance guard)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AI | 2101 | 71.4 | none |
| AJ | 2102 | 66.5 | none |

α = 0.897. Mean **68.95** — **down 2.15**. Both "promising".

### The read

Cut 22 fixed the early treadmill and exposed the other end: after the absence both raters held
~$2100 and nothing worth buying (supplies at $22–50, a $1650 row). Gold went from a leak to a
non-decision.

1. **Gold stops mattering** (both, first): AI "$2073 and nothing to spend it on"; AJ "$2131 …
   nothing meaningful to buy", while a late run still netted −$117 in restocks.
2. **Deaths look the same** (AJ): 4 of 5 watched deaths and all 3 offline were `return too late`
   at 1–2 HP on the walk home; the camp read `death 0%` and the next run died.
3. **Only exit rows matter** (AJ): most other edits read `≈`; paid cards are rows he could type;
   `foe: gas → throw unknown` fired 0/164 with no word. AI: `foe: heavy → read summon ally` never
   fired against Drix, unexplained.
4. **Undecodable words** (both): `read ✗ no use`, `attack ✗ no target`, `IT SHELLS`, `−$11 swap →
   folded scroll?`, a patch `survives 92% … reach D5 −88`.
5. **Editor friction** (both): the option sheet covers chips and eats taps; a drop confirm undid
   itself; the supply × is tiny; a layout jump bought a confusion potion; ~5 s to settle.
6. **The leash is stolen nearly every run** (AI) until he dropped it.

Kept: the forecast as experiment (`vs sent · D6 −56 · bank +31`), row-naming deaths, the reel
crediting rows, plateau notes that lead to an edit, bones and avenging.

## Cohort 19 (build 4b15a61: Cut 23 + QA W/X — the forge, rows answer, hero looks, the juice pass)

| Rater | Seed | Total | Gates |
|---|---|---|---|
| AK | 2301 | 71.4 | none |
| AL | 2302 | 71.9 | none |

α = 0.905. Mean **71.65** — **up 2.7**. Both "promising". The forge and the answering rows landed
(AL: mastery and tension 0.8; best D4 → D14 in an hour); feel and aesthetic stay 0.6 from both
despite the juice pass — pacing, not polish, is what they name.

### The read

1. **Fights that can't progress play out in full** (both): AL's first Goblin Warlord ran > 4 min
   at 1× on `R7 attack nearest` with the boss bar full until ▶▶|; AK's fights run took 5.6 min
   with ~100 s of retreat ↔ pack break against D9 archers. The dance guard (Cut QA-V) did not
   catch either.
2. **Repetition** (AK): Ashul the jackal on D3, the shrine line, identical end-of-run summaries,
   run after run. Surprise 0.6 from both.
3. **The forge's prices climb with best depth** (AL): leather +1 $1000 → $1100 → $1400, mail
   $2800 → $3600 — a deeper best makes the same step dearer; a thief took the leather +1 he had
   just bought (kit must never be loot).
4. **The forecast solves** (AK): "I mostly tried things and read the number"; edits wait 3–7 s
   to settle; many own rows read `≈` (AL).
5. **Seams**: the keep sheet salvaged the tapped item (both); the reel credited `R2 drink heal`
   when `R4 read teleport` fired (AL); `Ulak is avenged` then Ulak again (AK); the warlord
   forecast on D9, met on D8 (AK); `leather +1 · death +7` (AL).

Kept: rows that name their failure, boss counters learned then written (`D9 <2% → 89%`),
named foes and bones, the overnight shopping list, the plateau note naming the ceiling row.
