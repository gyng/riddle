# Path to 95 — the arithmetic, the stuck axes, and the structural bets

*2026-09-26. Inputs: 42 blind cards (`eval/cards/*.rater*.json`, cohorts 1–21) + the Cut 1 self
card, `eval/presets.json` (`idle-roguelike`), `../eval-fun/tools/fun-score.mjs`,
`../eval-fun/docs/FRAMEWORK.md`, `docs/FUN_EVAL_IDLE.md`, `docs/PLATEAU.md`. The bet it
recommends is `docs/CUT26.md`.*

## 1. The arithmetic

**The scorer** (`eval/score.sh` → `fun-score.mjs`): weight = preset weight / 37 × 100, award =
human × weight, capped at 0.6/0.8 only for a `red`/`amber` bot; every Riddle card is `bot: "na"`,
so **no cap has ever applied**. Criterion probes never enter the total. The gate (a weight ≥ 2
axis under 0.6) only blocks *super fun* (≥ 85); ≥ 72 fun, ≥ 55 promising. Social (w0) unscored.

| Axis | w | pts | plateau mean (C15–21, 14 cards) | 0.6 / 0.8 cards | lost vs 1.0 |
|---|---|---|---|---|---|
| expression | 3 | 8.11 | **0.60** | 14 / 0 | 3.24 |
| pacing | 3 | 8.11 | **0.60** | 14 / 0 | 3.24 |
| surprise | 3 | 8.11 | 0.70 | 7 / 7 | 2.43 |
| clarity | 3 | 8.11 | 0.71 | 6 / 8 | 2.32 |
| story, progression | 3 | 8.11 | 0.79 | 1 / 13 | 1.74 each |
| decisions, failure, return, attribution | 3 | 8.11 | 0.80 | 0 / 14 | 1.62 each |
| tension | 2 | 5.41 | 0.64 | 11 / 3 | 1.93 |
| mastery | 2 | 5.41 | 0.70 | 7 / 7 | 1.62 |
| feel, autonomy | 1 | 2.70 | **0.60** | 14 / 0 | 1.08 each |
| aesthetic | 1 | 2.70 | 0.69 | 8 / 6 | 0.85 |
| **total** | 37 | 100 | **72.2** | | 27.8 |

Marginal value of one 0.2 step (0.6 → 0.8 or 0.8 → 1.0): **+1.62** on a weight-3 axis, **+1.08** on
weight 2, **+0.54** on weight 1 (0.6 → 1.0: 3.24 / 2.16 / 1.08).

What the eight-cohort stuck list is worth: **feel + autonomy + aesthetic all the way to 1.0 buy
3.2 points.** Expression + pacing to 0.8 buy 3.2; to 1.0, 6.5. The stuck list is half cheap and
half the two most expensive axes in the preset.

**Ceilings (scorer-verified on synthetic cards):**

| Card | Total |
|---|---|
| Cohort 21 (AO · AP); AO + expression, pacing, surprise at 0.8 | 73.5 · 70.8; 78.4 |
| **every axis 0.8** | **80.0** |
| every axis 0.8, decisions/failure/return/attribution at 1.0 | 86.5 |
| every axis 1.0 except feel/autonomy/aesthetic at 0.6 | 96.8 |
| …and mastery, tension at 0.8 | 94.6 |
| every axis 1.0 except three weight-3 at 0.8 (or two, plus the weight-1s at 0.8) | 95.1 |

So **95 is not reachable on 0.8s**: the all-0.8 ceiling is 80. 95 leaves a deficit budget of 5.0
points, which means **at least 10 of the 15 scored axes at 1.0**, no load-bearing axis at 0.6,
and at most three weight-3 axes at 0.8. In 43 cards (645 awards) **no rater has ever awarded a
1.0**. The 1.0 anchor also demands "a named genre benchmark does not do it better; a player names
it unprompted as *why* the game is fun".

Calibration on the same scorer: Chess 90.9, Hades 90.3, Balatro 90.3, Stardew 88.5, Celeste 85.9,
Into the Breach 83.6; **Loop Hero, the nearest comparable, 74.1 on this very preset** (it earns no
1.0 either). Riddle's plateau mean (72.2) is Loop Hero's score. **95 is above every title the
rubric was calibrated on.** The honest target is the *super fun* band (≥ 85, no gate): the
realistic recipe is all axes at 0.8 (+7.8 from today) and four at 1.0 — decisions, failure,
return, attribution, the four the raters already praise unprompted in every "tell a friend"
(`death screen names the row`, `the forecast as you edit`, `the overnight report`) — 86.5. 95
additionally needs 1.0 on story, progression, clarity, surprise, expression, pacing: the game
would have to be the genre's reference on nearly every axis at once.

**Protocol caps no design change moves** (§4 E):
- *Audio*: "unassessed" on every card since sound shipped (Cut 10); agent raters cannot listen, and
  read it as an untested layer (AA: "Sound unassessed, so not above adequate"). `eval/RATING.md`
  still says "Audio: none in the build" (stale).
- *One simulated absence*: pacing's cadence loop is never observed (S: "20-min and 4-h absences
  untested, which caps this at adequate"; Y); `return`'s lapse is "simulated in the same sitting" (AO).
- *A one-hour horizon on a D34 game*: AO, AL, AP name "the next lesson not reached in the horizon".

## 2. The stuck axes

Anchors (FRAMEWORK §3, same for every axis): **0.8** "Directly present throughout the agreed
horizon, with one named material lapse". **1.0** "Direct evidence across the whole agreed
horizon, no material lapse, and a named genre benchmark does not do it better; a player names
it unprompted as *why* the game is fun". **0.6** "present often enough to support the loop, but
routine repetition, friction or an untested required layer keeps it from strong".

### Expression (w3; 0.6 on 14/14 plateau cards; 0.8 only in cohorts 6–7)
Definition: "the player's choices make something that is recognisably *theirs*". Probe: "describe
your build to a friend".
- AO: "the set converged to 'foe: boss → attack boss', 'foe: ranged → attack ranged' … — **the
  facts pick the row, I only order them**." AP: "most of my stylistic rows forecast '≈', so the
  set converged on one shape … any player would reach."
- AG: "the forecast pulled me toward one local optimum; tried rogue and cleave, the numbers
  vetoed both instantly, so the build is 'what the shaft liked'." AF, AE, AD: "converged on
  whatever the forecast scored best"; AI: "the bank-when-hurt shape feels like the obvious optimum".
- The game writes the set: T "only 3 of 6 rows are mine"; AK "four of eleven rows are bought
  cards"; AM "the pieces are the ones the game's patches and cards named". Sets 2/3 stay empty (AC, U).

**Root cause:** one linear descent, one scalar objective (bank share × gold), and an exact oracle
pricing it. A single optimum exists and the forecast reveals it in seconds, so every player's
set converges on it; non-optimal verbs (~40 exist; W "used ~8") read `≈`. Expression needs
*several defensible optima*, which needs situations or goals the player picks between.

### Pacing (w3; 0.6 on 14/14; 0.8 only O, N, T)
Boundary: report in-run, check-in and **cadence** separately, then judge the whole.
- In-run is now fine (AP, AM, AI: "no dead stretch"). Administration: AO "camp admin (two-tap
  forge, nested sheets, a tooltip over a button) is heavier than the decisions in it"; AL "every check-in is forge + unlocks + loadout + edit
  sheets"; AH "admin … outweighs the watching"; AE, AN, AM the same (sheets, 5–9 s waits).
- Cadence: AP "Cadence repetition after return keeps it at 0.6"; AP: after the absence every
  run is "the Warlord down in ~6 s then a bank at D10–13"; AH/AG "D1–D4 filler every run".

**Root cause:** every run re-walks floors the set has already solved (the start economy favours
D1: AP "the gold economy pushes one answer (start D1, bank low)"), and a check-in is mostly
maintenance around one decision. Cut after cut made the watch denser; the loop's *shape* (same
floors, then a camp of sheets) did not change. Plus the protocol never sees cadence.

### Surprise (w3; 7 × 0.6, 7 × 0.8; fell back to 0.6 · 0.6 in cohort 21)
Boundary: "materially different situations the rater directly encountered … how they changed play".
- AO: "the first six floors repeated rats/jackals/goblins in every one of 8 watched runs"; AE
  "D1–D7 repeat nearly verbatim each run"; AF, AG, AH, AI the same through four cuts of rotation.
- AL: "the same Warlord on D8 and Bloat Mother on D13 showed up every run"; AP "the same D8
  Warlord every run"; stock lines as wallpaper (monkey thefts: AC, AD, AN, V, AA).
- What earns 0.8 is a biome or boss that **changed a row** (AE: Bloat Mother "made me buy throw").

**Root cause:** PLAN's persistent macro ("per lineage seed the descent is fixed: biome order,
wall bosses") plus a D1 start means the first minutes and the walls are identical for the whole
lineage. Floor regeneration varies tiles, not situations.

### Feel (w1; 0.6 on 14/14; 0.3 as late as cohort 13)
Boundary: "experienced response and consequence in each claimed modality … If audio is
unassessed, mark it unassessed".
- Praise is stable ("callouts land on the beat": 'WARLORD BREAKS'). Lapses: AO/AG/AI/AM "much of 1× is a small sprite in black rooms" / "the lower half is black";
  AP "several camp taps needed a second tap"; AL/AJ long `attack nearest` fights; S: "the forecast
  bars re-drawing after every chip tap is the most responsive thing in the game".
- Audio unassessed on every card since sound shipped.

**Root cause:** the player's hands are only ever on the camp; the watch has no input, so feel is
judged on a spectator view of a plan-view map, and the one responsive act (an edit) answers with
a number, not a consequence. Half of the remaining gap is the audio protocol.

### Autonomy (w1; 0.6 on all 42 blind cards; 0.3 on the Cut 1 self card)
Probe: "What did you decide to do that the game didn't ask for?"
- Every plateau card names the same limit: AG "one dungeon, one direction (down), goals are depth
  or gold"; AD "I can't pick floors or branches, only thresholds"; AI "the dungeon is a single
  descent, so freedom is in the policy, not the route"; AN "the goal (next depth) is always the
  same one"; AH "the forecast solves most goals before the run, so exploration happens in a
  spreadsheet more than in the dungeon".

**Root cause:** no route and no goal besides depth/gold. Waystones and bank depth are knobs on
the same axis.

### Aesthetic (w1), mastery (w2), clarity (w3), tension (w2) — shorter
- *Aesthetic* (0.8 in cohorts 15–20 for 6 cards, back to 0.6): "dim and small on a phone" (AO),
  "camp's lower half is empty black" (AC, AE, AG, AH), "D1–D8 corridors are one brown palette"
  (AN); audio unassessed. Root: the same first biome every run, and the protocol.
- *Mastery*: "the forecast taught me more than the runs" (AK), "hill-climbing the forecast number
  by ±$10" (AF), "reading forecast deltas rather than understanding the dungeon" (AE, AD, Y);
  post-absence plateau with no visible next lesson (AO, AP, AL, AB). Root: the oracle again — the
  edit's answer is a delta, never the mechanism — and one linear wall at a time.
- *Clarity*: 8 × 0.8, 6 × 0.6; the lapses rotate every cohort (a GAP beside `unpatched 10/12`, an
  unmarked locked cond, `reach D5 −76`). Seams, not structure; QA rounds handle them.
- *Tension* reached 0.8 · 0.8 in cohort 21; its lapse is the same as surprise's: "D1–D6 read 100%
  in the forecast and played as a formality every run" (AO).

## 3. The three recurring root causes (not the seam of the week)

1. **One corridor, one number, one oracle.** Linear descent × single scalar × exact forecast ⇒
   one optimum per lineage, found by hill-climbing. Kills expression, autonomy, mastery ("the
   shaft liked it"), and half of surprise (same walls). Cuts 12–25 fixed lapses *around* it.
2. **Every run replays the solved past.** D1 start + fixed descent ⇒ every watch and night opens
   the same (surprise, pacing cadence, tension's formality, aesthetic's "one brown palette").
3. **The only verb is at camp, and the camp answers in numbers.** The watch has no lever; the
   edit answers with `≈`/`+7`. Feel, in-run pacing and mastery all read this.

## 4. Structural proposals

Each is judged against the hard invariants (AGENTS.md): policy load-bearing bots (DEFAULT ≤ D6,
EDITED +15, RANDOM/PASSIVE lose, LEARNED ≤ +2), determinism, offline uncapped, no tutorial
text, copy budgets, facts learned / policy written.

**P1 — The descent forks; the route is written.** (→ `docs/CUT26.md`) *Moves: expression, autonomy, surprise, mastery, pacing (cadence), story (routes retell).*
At each band boundary after the Warrens (D5, D9, D14, D19, D24) the stairs fork: the band's own
biome or the next one early (an adjacent swap of the base order; a deferred biome must be taken
next band, so every biome sits within one band of where it is tuned). The route is a chip in the
rule set — policy, exported, forecast — and `in: <biome>` becomes a fact-gated cond, so covering
both lanes costs rows under the cap. Existing content (7 biomes, 6 bosses) makes ~13 routes.
- Evidence: autonomy's single sentence on 14/14 cards; expression's convergence; surprise's "same
  Warlord, same Mother every run"; mastery's "no next lesson" (a second lane *is* one).
- Risk: bots must hold on every lane (DEFAULT meets the Fens at D5); tuning each biome at ±1 band
  (stats scale by depth via `tier_depth`, traits by biome — the anti-pillar "no HP inflation"
  holds only if the ±1 limit holds); forecast cost for pricing two lanes; gate-table time ×routes
  (sample lanes per seed as verdicts are sampled). Determinism: descent = f(seed, route).
- Gate: **no set dominates both lanes** (each fork's per-lane best set loses ≥ 15 pts on the
  other lane, paired seeds); both lanes viable; bots per lane; rule-set diversity ≥ 0.5.

**P2 — Runs start where the doubt is; solved floors fold.** *Moves: pacing, surprise (as encountered), tension, feel, aesthetic.*
The sim still plays D1 onward (economy and determinism unchanged), but floors the set clears
≥ 95 % (paired forecast) play as one interstitial line each (`D1–6 · 100% · +$84 · 2 thefts`),
and the watch opens at the first uncertain floor; a lit waystone start is never dominated on
gold/hr by D1 for a set that clears the band ≥ 95 %.
- Evidence: AO/AP/AE/AF/AG/AH "D1–6 every run"; AP "economy pushes start D1"; tension's formality.
- Risk: low; the reel/report must still carry a folded floor's beats (thefts are stakes). Copy:
  the fold line is a count, not a sentence.
- Gate: fights.mjs — watched screen time on floors forecast ≥ 95 % ≤ 5 s per floor; metrics:
  waystone gold/hr ≥ D1 gold/hr on cohort sets.

**P3 — Oaths: goals the player picks, priced by the forecast.** *Moves: expression, autonomy, story, progression, decisions.*
A camp board of three standing oaths per lineage, each a constraint + distinct reward
(`D10 · no drink` → a card; `tame 3 kinds` → a party slot; `Warlord · fire` → a title in the
chronicle). The forecast prices the active oath. Different oaths want different sets.
- Evidence: autonomy "goals are depth or gold"; expression; PLAN's promised "trophy ledger
  rewarding different policies" never shipped as goals; progression's "gold piled up" (AO).
- Risk: a reward must not be stats (policy stays the lever — the Cut 25 metrics row); oath text
  within copy budgets (constraint as chips, no sentences).
- Gate: each oath's best set differs from the bank-optimal set by ≥ 2 rows; cohort sets hold.

**P4 — The edit is a scene: the divergence replay.** *Moves: feel (the core act gets a consequence), mastery, clarity, attribution.*
On each edit that moves the forecast ≥ 5 pts, find the first tick on a paired seed where the new
set fires a different row, and play that moment in the watch pane as a 3–5 s before/after
(`R5 now` → `lives · D9` vs `dies · D7`). The number stays; the scene carries *why*.
- Evidence: mastery "the forecast taught me more than the runs"; feel S "the forecast bars …
  the most responsive thing"; expression AG "the numbers vetoed both instantly".
- Risk: perf (one extra traced paired replay; must land inside Cut 25's refine ≤ 3 s); none to
  bots or determinism (pure read of paired sims).
- Gate: divergence found for ≥ 90 % of ≥ 5-pt edits on cohort sets; scene ≤ 1.5 s after refine.

**P5 — A lever inside the watched run: patch at the stairs.** *Moves: feel, pacing (in-run), tension, autonomy.*
While watching, an edit may be armed to take effect at the next stairs (logged into the run's
event stream as `rules@tick`); the run's verdict and trace credit the swapped set.
- Evidence: tension boundary ("watched fights whose outcome cannot change anything"); pacing
  "a camp of sheets"; autonomy.
- Risk: **anti-pillar "no presence rewards"** — a watched run can be strictly better than an
  offline one; mitigate by making it a one-per-run edit, not steering, and never an exit.
  Determinism: replay needs (seed, rules, edits). Attribution: the trace must name which set.
- Gate: replay-hash with mid-run edits; bots untouched; offline yield unaffected.

**E — The eval measures what it can (not design; run it first as a control).**
- Fix `eval/RATING.md`'s stale "Audio: none"; one human listening pass by the owner on the cohort
  build, filed as audio evidence the agent raters may cite (FRAMEWORK: sound counts only when
  someone listened) — worth up to 2.2 points that no design can earn from an agent rater.
- Horizon: three simulated absences (`?absent=20m`, `4h`, `8h`) instead of one, so pacing's
  cadence loop and `return` are observable; FUN_EVAL_IDLE §4 already asks for them.
- Run it as a **control cohort on 307dbed** first, so its effect is separable from Cut 26's (a new
  horizon is a new cohort, FRAMEWORK §4). A Tier-2 card every other cohort lets mastery and
  progression see past D14.

## 5. Recommendation

**Cut 26 = P1** (the forked descent, route as policy). It is the only proposal whose evidence is
a sentence on *every* plateau card (autonomy) and that attacks root cause 1 at its source: a second
defensible optimum per fork is expression by construction, a lane is a surprise the player chose,
and the unchosen lane is the next lesson. It re-uses shipped content. Expected on success:
expression and surprise 0.8 (+3.2), autonomy 0.8 (+0.5), mastery 0.8 from both (+0.5) ⇒
~76–78 from cohort 21's 72.15.
Then **P2 + P4** (Cut 27: pacing and feel — the watch opens at doubt, the edit is a scene), then
**P3** (Cut 28). Run **E** now, on the current build. Stop measuring success by cohort mean
alone: the criterion probes (fun 5/7 and play again 5/7 on 35 of 36 cards since cohort 4; AM
6/6) are the rubric's validity check, and a structural cut should move them.
