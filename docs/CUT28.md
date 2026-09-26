# Cut 28 — A goal of your own; the answer names its cause

*Contract, 2026-09-27. Cohort 23 (build 631fe23, three absences): 70.8 · 72.4, α 0.968, mean 71.6
(+5.9 over the control's 65.7). `docs/PATH_TO_95.md` P3; `docs/PLATEAU.md` "Cohort 23".*

## Design

### 1. Oaths — goals the player picks (core + client; expression, autonomy, progression, decisions) — P3

Both: after the absence every run is the same D13 bank; gold piles up; forge steps `≈`.

- **An oath board** in camp: three standing oaths per lineage drawn from a pool, each a
  **constraint + a distinct reward that is not a stat** (policy stays the lever — the Cut 25 lever
  row holds): e.g. `D10 · no drink` → a card; `tame 3 kinds` → a party slot; `Warlord · fire` → a
  chronicle title and a trophy; `bank D14` → a waystone; `no return · D9` → a unique verb.
- **An oath costs gold to swear** (the gold sink): priced from the lineage's income, refunded on
  completion into its reward; a sworn oath is shown on the shaft and priced by the forecast (`oath ·
  D10 no drink · 34%`), and the divergence scene can show an edit's effect on it.
- **Different oaths want different sets**: gate — each oath's best set (plateau search) differs from
  the bank-optimal set by ≥ 2 rows on the cohort sets; each oath is completable (≥ 20 % per night)
  by some set a rater could write.
- **The wall has a path**: the D14 Mother (and every band boss) exposes its counter as a fact the
  lineage can learn (`mother: fire`) and an oath or bounty points at it; `bounty D13` shows what it
  pays and needs (`bounty · D13 · $×2 · reach`).
- Gate: metrics rows (oath diversity, completability, the lever row with every oath reward owned);
  ui.mjs; bots hold with oaths off (DEFAULT/EDITED never swear).

### 2. The answer names its cause (core + client; clarity, failure, mastery)

- **The forecast move attributes state, not only rows**: an edit-free change (a pet died, the kit,
  the purse, the route, a fact learned) shows as its own line (`party −2 jackals · death +24`), and
  the divergence scene runs only on row edits; when both changed, each gets its share (paired
  panels with and without the state change).
- **Max hp in the trace**: a drain or curse shows its max-hp step so `R1 hp not <30%` at 6/9 reads.
- **Luck-leaning deaths** read as what they are: a death most replays survive leads with the
  rare event that killed him (`a crit at 6 hp · 1 in 6`) rather than a verdict stamp that reads as
  blame; patches read `no gain` without a green move beside them.
- **Reports lead with decisions**: the first screen of a report is what changed and what to do
  (plateau, oath progress, a new counter learned); salvage and bones fold under a `details` tap.
- Gate: tests; ui.mjs; qa.rs (a forecast move's attributed parts sum to the whole ± its noise).

### 3. No dead time below the fold (client + core; pacing, feel)

- Mid-run stretches with no decision and no threat (a pick-up chain, an empty corridor) play at the
  travel rate in every mode including 1× (the 1× mode keeps fights and beats at 1×).
- The watch's mode is chosen visibly at the send (the gem names it) and remembered.
- Gate: fights.mjs — no stretch > 5 s at 1× without a fight, a beat, a pickup of note or a descent.

### 4. Defects (both raters)

- The `kite archers` card left the forecast on `…` forever (a hang — reproduce, fix, test).
- Watch stamps never overlap chips or name plates (layout test).
- A one-slot vault never salvages a caged item without offering the keep choice.
- `send skips rest` never sends a heir below ~50 % hp without saying so (`9/40`).
- The reel never repeats a line shape more than twice across an absence's runs; `took him to 40 HP`
  (wrong direction) fixed.
- A pet is never named as a grudge killer.
- The class picker on the portrait opens.
- Gate: tests, ui.mjs, qa.rs.

## Gates

| Gate | Bar |
|---|---|
| Oath best sets differ from bank-optimal by ≥ 2 rows; each completable ≥ 20 %/night; lever row with oath rewards | metrics |
| Forecast move attribution sums; max hp in traces; luck deaths lead with the event | qa.rs, tests |
| No 1× stretch > 5 s without an event | fights.mjs |
| The defects | tests, ui.mjs |
| Bots, dice, stalls, dances, lever, lanes, divergence, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 24 (three absences): mean ≥ 74; expression or autonomy 0.8 from ≥ 1; α ≥ 0.80 | two blind cards |

## Deviations (recorded)

- Oath pool: `dry` (`D… · no drink`) cut — its best set was the bank-optimal set minus one row (the
  ≥ 2-row gate); `quiet` cut (no set kept it). Pool: `bold` (no return) → a verb, `tamer` → a party
  slot, `fire` → a title, `slayer` → the waystone past the boss, `lean` (no rest) → a card. Price:
  max(forge unit, ½ last night's net); keeping spends it into the reward; forswearing refunds half;
  one oath at a time. Measured: 19/19 oath·set pairs differ by ≥ 2 rows; 19/19 keepable ≥ 20 %/night;
  the lever row holds with every reward owned (min gap +34).
- The `kite archers` `…` hang did not reproduce in the core (50+ lineages native, a D14 save in
  wasm); a regression test prices the card at every place; the client drops a refine unanswered
  after 15 s so the shaft never sits on `…`.
- `send skips rest` at 9/40 was the fold handing off a hurt hero — the hp rides the fold line.
- `forecastMove` runs on measure lane 1 (beside the edits' vs and divergence on lane 2 it held the
  refine after a burst).
