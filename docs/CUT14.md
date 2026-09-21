# Cut 14 — The number settles, the screen shows the fight

*Contract, 2026-09-22. Cohort 10 (build 2cb9e88): 72.4 · 72.4, α 0.755, mean 72.4.
`docs/PLATEAU.md` "Cohort 10": the stall was an engine loop the gate's sets did not carry (fixed
10b59b9, 255d4de; the gate table now plays every cohort's own sets). What is left is one
number both raters could not trust, one verdict that contradicts its own margin line, a
render that carries the fight in callouts, and a handful of small lapses.*

## Design

### 1. The forecast is a paired measurement (core; decisions, clarity, mastery)

S: "the same six rows read 94/6, then 79/21, then 72/28 (±18–21)"; T: "±22 … a 10-point edit
is unreadable"; "a patch worse than base (survives 42 % · base 50 %) was offered first".

- **Paired seeds.** `forecast_tag` drops the rules from its hash: every set a lineage forecasts
  is played on the same dungeons (lineage seed × depth × sim index), so an edit's delta is a
  paired difference, not two draws. Re-reading a set still repeats its number (Cut 6 §9 holds).
- **The ends line at the refine's count.** `ENDS_SIMS` is 20 (±22 at 50 %); the refine pass
  runs the ends at 50 (`REFINE_ENDS_SIMS`), and the refined line is the shown one. The first
  paint keeps its `…` mark (Cut 13 §5).
- **Nothing below base is offered.** A patch whose `survive` is under the death's `baseline`
  is not on the death screen — the root patch included (its measured number goes to the
  unlock's own sheet as a delta, where it reads as information, not advice). The dice fallback
  (Cut 11 §4) stays: when nothing is over the bar, the best candidates are shown *flagged*
  `below_bar`, and the client renders them under `nothing beats base` rather than as patches.
- Gate: on 30 seeds, the paired delta of a one-notch edit (`hp < 30` → `hp < 25` on the good
  set) has |Δ| ≤ 8 pts at D5 on ≥ 90 % of seeds; two refined reads of an unchanged set are
  identical; the refined ends `±` ≤ 14; no death screen carries a patch under its baseline.

### 2. A verdict never contradicts its margin line (core; failure, attribution)

S: "death 1 read as a rule gap (7 unknown unused, heal row inert) but was stamped `dice`".

- `N unknown unused` is on the margin only when a drink/read-unknown candidate survived the
  replays ≥ 50 % — and then the verdict is `gap` with that patch first. When the unknowns
  would not have saved him (the replays say so), the margin says what would: nothing, or the
  telegraph the dice screen already names (`the burst · 9 hp`).
- The same for `heal unused`: it is on the line only when the heal-drinking candidate
  survives ≥ 50 %.
- Gate: on 30 seeds no `dice` death's margin names an unused item; every `gap` death's first
  patch fired in ≥ 50 % of its replays (holds already).

### 3. The map frame carries the fight (client/render; feel, aesthetic)

T: "in ten screenshots of fights I never saw a foe sprite"; "a black screen with a lit patch
of olive tiles and one well-drawn hero"; "the bank moment is a sheet, not an event". S: "five
foes on one tile rendered as one smear"; "foe sprites a fraction of the hero's size".

- **Bigger read in the map frame.** `BASE_TEXELS` 200 → 160 (≈ 19 tiles across at 400 CSS px):
  a 32-px foe reads at ≥ 1.5 tiles' worth of screen; the fight frame is unchanged. Measure
  the frame time on the GPU harness before and after (`tools/browser.mjs`, 60 fps holds).
- **A stack fans in the map frame** the way it does in the fight frame (Cut 8A's sideways
  nudge): two or more foes on one tile spread ±⅓ tile, their names never on one row.
- **The bank and the return are beats.** A `bank`/`return` exit opens the fight frame for
  `SCENE_MS` on the stairs with `BANKED $N` / `RETURNED $N` as its callout before the exit
  sheet; in `fast` too (like a situation's beat, Cut 13 §4).
- **The lit patch is not the whole screen.** Remembered tiles keep 0.6; the room the hero
  stands in is lit whole once seen (its tiles at 1.0 while the hero is inside it) so a fight
  has a floor under it.
- Gate: `fights.mjs`: a fake run with two foes on one tile draws two names on two rows in the
  map frame; a bank exit opens the frame with `BANKED`; a screenshot at 400×800 on seed 601's
  first fight shows a foe ≥ 24 CSS px tall (`tools/playtest.mjs` measures the entity rect).

### 4. Small lapses (client)

- The report's `trace` chips carry their exit (`D5 · died · trace`, `D6 · banked · trace`)
  (S: "the seventh unlabelled TRACE button").
- A patch tap on a full set does not insert and then ask: the death screen's patch chip reads
  `↑ R3` on a full set (it replaces the least-fired row) and the editor opens on that row
  (S: "an offered patch pushed me to `6/5 · drop one` with no warning").
- The ticker coalesces a repeated chore callout: eight `pick up` reads become `pick up ×8`
  (T: "dead stretches of eight consecutive `pick up` reads").
- The `rest 20m` overlay never covers the death frame's telegraph line (S).
- The stalled tile says what it cost: `2 STALLED · $161 lost` (T: "`2 STALLED` says nothing
  about what the stalls cost").
- `window.__audio` is on every build (read-only cue log), so a rater can assess sound.
- Gate: `screens.mjs` checks the trace chip labels and the stall tile; `clarity.mjs` the
  coalesced callout and the overlay; copy-lint at 0.

### 5. Process (coordinator)

Mechanical QA (`examples/qa.rs`, `screens.mjs`) → one transcript-first and one interactive
QA player → the blind cohort 11 with rule exports (`eval/RATER_PROMPT.md`), the exports into
the gate table before the next cut.

Not this cut: expression (0.6 for six cohorts). Its answer is content — a second class worth
playing, a card that changes the archetype — and it needs its own cut with the dayplayer's
content bars (M7).

## Gates

| Gate | Bar |
|---|---|
| Paired forecast: one-notch edit |Δ| ≤ 8 pts on ≥ 90 % of seeds; refined reads identical; refined ends ± ≤ 14 | test, 30 seeds |
| No death screen carries a patch under its baseline | 30 seeds (`examples/qa.rs`) |
| No `dice` death's margin names an unused item | 30 seeds (`examples/qa.rs`) |
| Map frame: foe ≥ 24 CSS px; stacks fan; bank/return beats; 60 fps on the GPU harness | fights.mjs, playtest.mjs |
| Trace chips labelled; stall tile carries its cost; coalesced callouts | screens.mjs, clarity.mjs |
| Bots hold: DEFAULT dies by D6, EDITED ≥ +15, RANDOM/PASSIVE lose, LEARNED ≤ 2 floors, dice ≤ 5 %, stalls ≤ 1 % on DEFAULT and every cohort set | `node tools/gates.mjs --full` |
| Cohort 11: mean ≥ 78; decisions or failure at 1.0 from one rater; α ≥ 0.80 | two blind cards |

## Tracks

- **Core**: §1, §2, the two `qa.rs` invariants, the paired-delta test.
- **Client**: §3, §4, their tests.
- **Coordinator**: §5.
