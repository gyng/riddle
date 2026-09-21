# Cut 12 — Your rows, and no lapse

*Contract, 2026-09-17. Cohort 8 (build eec0f3f): 75.1 · 72.4, α 0.877, mean 73.8, "fun". Four
cohorts at 74–76. `docs/PLATEAU.md` "The second plateau": expression is 0.6 from every rater in
every cohort; nine axes sit at 0.8 with a defect for a lapse; the middle floors repeat.*

## Design

### 1. The rows are yours (core + client; expression)

O: "rows are the choke point — every patch tap lands as `7/6 · drop one`, cards I bought
auto-insert as rows"; P: "three of six rows are prefab cards and only two are my own sentences".

- **A card brings its row.** Rows whose verb is a tactic card do not count against `max_rows`;
  a set holds at most one row per owned card. The editor chip counts the player's rows
  (`3/4 · 2 cards`); `yours: n of m` counts the same way. Core: `set_rules` validates
  (own rows ≤ `max_rows`, card rows ≤ cards owned) instead of truncating; every place that
  slices `rows[..max_rows]` uses the same rule.
- **A card sits where it acts.** `UnlockInfo.insert_at`: before the first row whose verb is
  `attack` / `shoot` (the set's engagement row), else the end. The client inserts there; the
  card's delta (Cut 10 §3) is measured there. Both raters moved every card up by hand.
- **Re-adding a card never carries a condition.** Picking a card verb clears and locks the
  row's conds (P: "an uneditable `hp < 50%` prefix").
- Gate: EDITED beats DEFAULT by ≥ 15 (unchanged); the dayplayer content bars do not worsen; a
  test that a 4-row lineage holds 4 own rows + 2 card rows and the fifth own row is refused.

### 2. Chores never take what a row needs (core; attribution)

O: "`swapped for the poison` named an engine chore no row of mine could touch"; "the thief
guard card I bought changed nothing — the monkey still took the heal".

- The pickup swap chores (`turn.rs`, "a full pack swaps its cheapest consumable for a dearer
  one") never drop an item kind a row of the set names (`drink heal`, `throw fire`, `read
  teleport`) nor a supply packed at camp.
- `thief_guard` gains `on see den → attack nearest` as its first row (the raid), so the thief
  answer answers the den. Its sheet lists five rows.
- Gate: on 30 seeds with `hp<30 → drink heal` in the set and a heal packed, no `swapped for`
  provenance line names `heal` (test); den snatches per run with the card ≤ 20% of without.

### 3. The forecast knows how a send ends (core + client; decisions, clarity)

P: "the forecast shows survival but never expected gold, and cannot tell 'went home' from
'died' once a return rule exists".

- `Forecast.ends: { bank, return, death, gold }` from the same sims (rates, and the mean gold
  brought home per send). The panel adds one line under the depths:
  `bank 40% · return 35% · death 25% · ~$54`.
- Gate: two reads of an unchanged set agree within the shown ±; a set with `depth ≥ 8 → bank`
  shows bank > 0 on a lineage that reaches D8.

### 4. The floors do not repeat (core + client; surprise, tension, pacing)

O: "the same D3 den, the D6 archer corridor four times"; P: "runs 4–8 repeated the D4–D6
archer/jackal loop with near-identical `returned $NN` endings".

- From D3 every floor rolls **one situation** of its band (den, lock, captive, nest, shrine,
  vault, stray, hunger), never the previous floor's kind. Today they are a rate.
- The interstitial card names it: `D4 · 9 rooms · a nest` (`Snapshot.floor_twist?: string`,
  one word). The reel's routine return line carries it: `D6, the nest: returned $54`.
- Gate: DEFAULT still dies by D6, EDITED ≥ +15, dice ≤ 5%, quiet per-tick unchanged; over 8
  floors on 30 seeds the situation sequence holds ≥ 4 kinds.

### 5. The night is not half rest (core; return, pacing)

P: "`rested 287m` — the hero idled for five of the eight hours".

- Rest after a return or bank is **half** the run's length (was: the run's length); the 20-min
  wake after a death stays. `REST_MIN_TICKS` stays.
- Gate: dayplayer content bars (unlock days, stall) do not worsen; `verify --full` green.

### 6. Defects from cohort 8 (client unless noted)

| Rater evidence | Change |
|---|---|
| P: "`▶▶\|` did nothing on four tries" (fast mode, a return run) | reproduce in `fast`; fix; the fights test covers both modes |
| P: "`fast` is still slowed on fights" | `fast`: fights 2×, travel 16× (`fights` keeps 1× fights) |
| P: "`+1 row ⊘ rows full` stays dimmed at 4/4" | the unlock shelf repaints when a rule edit crosses `max_rows`; the gate reads `⊘ fill rows` (both QA players read `rows full` as a state) |
| P: "supplies showed `1/5 leash` I never bought" | a free supply reads `leash · kennel` (was `· found` — the kennel gives it; QA: "bought one labelled found") |
| O: "the supplies `×` clears the whole list (I lost the leash)" | `×` removes one line |
| O: "`drink ✗ no use` fired every few seconds" | a sanity refusal callout shows once per row per floor |
| O: "`◯ spectral hound fell`, `rallied!`, `1 combo` never explained" | `ally hound fell` · `warlord rallies` · the combo's name |
| P: the night resumed a D2 run with an empty pack | **done** (a2a1c83, core) |
| O: a patch offered a row already in the set | **done** (9fbcfc4, core) |

### 7. A QA cohort before the blind cohort (process)

The 1.0 anchor is "no material lapse across the horizon". Cohorts find lapses after shipping.
From this cut: once §1–6 land and gates pass, `tools/ship.sh`, then **two QA players** (agents,
not blind to the goal, no scores) play the full rater protocol on fresh seeds and file every
lapse: anything misread, anything inert, any number that does not reconcile, any copy they
could not explain. Everything on the list is fixed or recorded as a deviation with a reason;
the build reships; only then the blind cohort. `eval/RATING.md` gains the step.

### 8. The QA cohort's list (952e306 → the reship)

Two QA players on 952e306 filed 131 lines (`eval/qa/952e306.qaA.md`, `.qaB.md`); the triage
(`eval/qa/952e306.triage.md`) closes each as a fix or a deviation. The fixes that were
contract-level: the forecast's ends from a run-to-exit panel with a `stall` share; gold in one
unit (a pile's amount is coins; the exit sheet prices at the engine's worth); `drop_supply`
in core (the client's clear-and-rebuy fallback wrote refund/rebuy pairs); the report's tile
is the send's `deepest`; exits say `stalled` and count unused supplies; the accounting links
`stuck` to its moment and names the row that drank; a held patch inserts nothing. Deviations
kept: stalls forfeit the loot (a gated invariant — `DEFAULT yields 0 xp/gold`), `▶▶|` is
next-fight, the trace is the state before each action.

A second pair on the reship (`eval/qa/e0f87e7.qaC.md`, `.qaD.md`; triage `.triage.md`):
round 1's fixes held; the round's own list (`▶▶|` to the next fight *or the end*, a foe at the
elbow inside the stuck window, SALVAGED reconciled to the ledger on every path, `· N more`
that expands, the keep sheet's `keep 0/1`, the mode remembered, the card following the HUD,
report traces with `watch`, the clip holding its last frame, `found ♟3's bones`, `none past
D12`, the `try:` hint reading card rows, a vaulted kind identified, killers from the ends
panel, a verb unlock's buy inserting its measured row) is fixed; stalls' forfeit and the
absence's vault churn stay recorded.

## Gates

| Gate | Bar |
|---|---|
| Card rows outside `max_rows`; fifth own row refused | test |
| Cards insert before the engagement row; delta measured there | test |
| Swap chores never drop a row-named kind or a packed supply | test, 30 seeds |
| Thief guard cuts den snatches | ≤ 20% of without, 30 seeds |
| Forecast `ends` stable within ± and bank > 0 with a bank row | test |
| One situation per floor from D3, ≥ 4 kinds over 8 floors | 30 seeds |
| Bots and content bars hold (§1, §4, §5) | `node tools/gates.mjs --full` |
| Every §6 row has a test or a screenshot | checklist in the report |
| QA list empty on the shipped build | two QA reports |
| Cohort 9: mean ≥ 78; expression ≥ 0.8 from one rater; any axis awarded 1.0; α ≥ 0.80 | two blind cards |

## Tracks

- **Core** (`crates/**`): §1 core, §2, §3 core, §4 core, §5, gates.
- **Client** (`web/src/**`): §1 client, §3 line, §4 interstitial + reel line, §6.
- **Coordinator**: §7.
