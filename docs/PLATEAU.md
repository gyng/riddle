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
