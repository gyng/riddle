# Cut 13 — No stake the player did not choose

*Contract, 2026-09-21. Cohort 9 (build 39def99): 75.1 · 72.4, α 0.938, mean 73.8. Five cohorts
at 74–76. `docs/PLATEAU.md` "Cohort 9": the QA step emptied the defect class; what is left at
0.8 is one mechanic and three designs, and both raters name the mechanic first.*

## Design

### 1. A stall is a run the player can read (core + client; failure, attribution, tension)

Q: "'stall' is never explained and it's the most expensive outcome in the game … no patch is
offered, so I can't name the row I should have written"; R: "the reel blamed 'R3 drink heal
caught him' while the trace said 'R1 throw unknown stuck' — contradictory, punitive, and not a
stake I chose".

- **Fewer of them** — done (cffa2d9): the three engine loops (an item the full pack would never
  take, the guard's ignore lifted by an arrow, the coward's streak) took the DEFAULT set from
  5.2 % to 0.5 % of sends. Gate: ≤ 1 % on DEFAULT and on the cohort's own final rule sets
  (`eval/cards/*.json` carry them), over 30 seeds × 12 sends.
- **A verdict for the one that happens.** A stalled run gets a death-style screen: the guard's
  moment as the headline (`stalled · D4 · archer, no path`), the trace, the row accounting, and
  patches measured like a death's (the checkpoint is the first guard; a candidate "survives"
  when the replay leaves the floor). The reel line names the same cause as the trace (one
  `Resolution::Stalled { cause }`, no `R3 drink heal caught him`).
- **The stake is said before it is lost.** The HUD's `keeps $28` reads `keeps $0 · stalling`
  from the first guard on (`Snapshot.stuck_fires`), so nothing is advertised that a stall pays
  nothing on.
- Gate: on 30 seeds every stall has a verdict screen with ≥ 1 patch that fired in ≥ 50 % of
  its replays; the reel's cause equals the trace's; `DEFAULT yields 0 xp/gold` holds.

### 2. The heir's trait is chosen (core + client; attribution, expression, autonomy)

Both: "R4 retreat — brave held", "curious drank heal at 24/36 hp" — "two losses I could not
own". Q's own build description ends "the hero's trait can override your rule".

- At the wake of a new heir the camp offers **two traits** (drawn from the lineage seed, never
  the last one); the player taps one before the first send (a chip beside `♟3`; the default is
  the first, so a send without a tap still goes). The trait's rule is one line on the chip:
  `brave · holds a retreat under 25 %`, `curious · drinks an unknown when clear`.
- A trait never overrides a row more than once per floor (`trait_last` becomes per floor), and
  its deviation is a callout that names it (`brave → hold`, already) *and* a because-link on
  the row it held (`← brave held it, D4 · t3120`).
- Gate: the heir's trait is on the wire (`Lineage.trait_offer`, `set_trait`); a test that a
  chosen trait sticks; the trait deviation count per run ≤ 1 per floor over 30 seeds.

### 3. The night's ledger is on the report (core + client; progression, return)

Q: "I left with $142 and came back to $6 and the report never said where it went"; R: "$2131
after the night had almost nothing to buy".

- `ReturnReport.spent: Vec<SalvageRow>` — what the automations bought during the absence, per
  kind (`heal ×16 · −$640`); the report's `SPENT` section beside `SALVAGED`; the tiles' gold
  reconcile: `banked + returned + salvage − spent = header delta`.
- A supply the trait drinks at full HP is not rebought: `auto: restock` skips a kind the last
  run used with no effect (`Ev::Use … outcome "nothing"` / a heal at full HP).
- The shop grows with the night: the catalogue offers what the lineage has identified *and*
  the forge's craftables at their tier — R's $2131 has something to buy. Gate: the dayplayer's
  "days with ≥ 1 unlock" bar moves toward its 10/14 (informational), marks unspent ≤ 8 holds.

### 4. The beats are on screen (client; story, failure, pacing)

Q: "the best beats — the thieves, 'Grul is avenged', 'A vault: three under a cage' — live only
in the reel or the morgue"; "the fights mode skipping whole floors"; R: "the death screen
headline hid the best story ('The green one: fire. Gambled: fire potion.')".

- The situations cut in like fights: a `note` event whose text is a situation's (the den wakes,
  a thief snatches, the vault opens, a captive, the stray) opens the fight frame for its beat
  (`SCENE_MS`), in `fights` and `fast` alike.
- The death screen carries the run's last two notes under the headline (`Death.notes`, ≤ 8
  words each: `The green one: fire. Gambled: fire potion.`).
- A callout never clips (Q: "callouts clip at the right edge", "JACKALCKAL"): the ticker
  measures and wraps; two callouts on one tick queue.
- Gate: fights.mjs — a fake run with a den and a thief opens the frame twice outside a fight;
  a death with notes shows them; a 40-char callout renders whole at 400 px.

### 5. The forecast's noise is shown as noise (core + client; decisions, mastery)

Both: "±10 between re-rolls", "±15 made some comparisons coin flips", "'95 % bank' then
'spectral blade · dice'"; Q: "unlock deltas read 'pack break +7 %' then '−14 %' a minute apart".

- A catalogue delta within its own half-width reads `reach ~0` (not `+7 %` / `−14 %`); the
  `±` is on the delta as on the bars (`UnlockInfo.pm`).
- The refine pass (100 sims) is the shown number once it lands; the first paint at 50 is
  marked (`…` after the `±`) so a re-read does not look like a re-roll.
- The ends line carries its own `±` (`death 5 % ±4`); a `dice` death's screen says the
  forecast's death share at that depth (`forecast said 5 % · this was the 5`).
- Gate: two reads of an unchanged set agree exactly after the refine; a delta shown as
  non-zero has |delta| > pm on 30 seeds' catalogues.

### 6. QA before the cohort, mechanically first (process; `docs/ITERATION_SPEED.md` §1.1–1.2)

- `examples/qa.rs`: the wire invariants the four QA players reconciled by hand (header ==
  ledger, RUNS == exits, ends sum to 1, `death 0 ⇒ no killers`, a learned flavour never
  prints `?`, camp supplies == the next run's `brought`, no patch already in the set, a
  dropped supply's refund == its price), 30 seeds, in the gate table.
- `web/tests/screens.mjs`: the QA brief's itinerary as a walk with a screen lint (no
  duplicated segment on a line, every enabled button changes the text, every sheet titled,
  header numbers == report numbers).
- Then one transcript-first QA player per build (reads the walk's dumps, reproduces) and one
  interactive; the blind cohort only after the list is empty.

## Gates

| Gate | Bar |
|---|---|
| Stalls ≤ 1 % of sends on DEFAULT and the cohort-9 sets; every stall has a verdict with a firing patch; reel cause == trace cause | 30 seeds |
| A chosen trait sticks; ≤ 1 trait deviation per floor | test, 30 seeds |
| Report gold reconciles: banked + returned + salvage − spent == delta | test, 30 seeds |
| Situations cut in; death notes shown; no clipped callout | fights.mjs, clarity.mjs |
| Deltas within their ± read `~0`; refine is the shown number | test |
| Wire invariants (`examples/qa.rs`) and the screen lint pass | gate table, `pnpm test` |
| Bots hold: DEFAULT dies by D6, EDITED ≥ +15, RANDOM/PASSIVE lose, LEARNED ≤ 2 floors, dice ≤ 5 % | `node tools/gates.mjs --full` |
| Cohort 10: mean ≥ 78; attribution or failure at 1.0 from one rater; α ≥ 0.80 | two blind cards |

## Tracks

- **Core**: §1 verdict + HUD field + `Resolution::Stalled`, §2, §3, §5 core, `examples/qa.rs`.
- **Client**: §1 screen, §2 chip, §3 section, §4, §5 panel, `screens.mjs`.
- **Coordinator**: §6, the QA rounds, cohort 10.
