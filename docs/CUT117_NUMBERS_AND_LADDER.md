# Cut 117: numbers that agree, a ladder that moves

*2026-10-10. Ten blind cohorts. 8cf9050 (Cut 116) is the best so far: A 70.2, B 67.6, α .866, mean 68.9.
Varied descent raised surprise and return to 0.8 on A. The remaining gripes are about **trust**
(numbers that disagree, advice that flips) and **sameness** (the same boss order, a purse with nothing
to buy). Goal: 80.*

## 1. Numbers that agree (clarity, mastery)

- King death header `18 hp` vs trace `0/50`: the header reads the deciding moment's hp, as the trace does.
- Report gold head: `$0 GOLD EARNED · purse −$6789 · forge −$6750 · lost $2620` must reconcile in one line
  (earned − spent = Δpurse), each term named. Gate: a core test that the terms sum.
- Forecast vs outcome: `D33 88%` then banks at D30/D32. The forecast names the horizon it prices
  (`reach D33 · 88%` vs `bank by D33`) and the scout's wall order is part of it.
- Preview noise: the 8-sample previews swing. Show a range or hide a delta smaller than its noise.
  Gate: no shown delta flips sign between two consecutive reads of the same camp.

## 2. Advice that holds (autonomy)

- `TRY` never swaps a tactic silently. It names what it replaces and asks with one tap. It is stable
  across reads (same camp → same advice).
- The `Warlord tactic` button opens the sheet *on* that tactic.
- Quest and breed taps show a result (a toast, a changed chip).

## 3. A ladder that moves (surprise, pacing)

- Affixes must change which answer is best (Cut 116 gap: 1 of 20). Each affix gets a counter that is
  a *different* tactic or variant from the plain wall's best. Gate: ≥ 8 of 20 boss × affix pairs change
  the best answer; stance row and IDLE bars hold.
- Band bosses may swap order within a band pair per heir (D13 ↔ D18 bosses, D23 ↔ D28) where the
  biome allows. Gate: IDLE bars hold; dice deaths ≤ 5 %.

## 4. A purse with a use (return, progression)

- After `Kit complete`, gold buys something that matters: a deeper tree node, an heir's
  starting kit, or Ascension's entry. The apprentice never leaves `$0 budget` without saying why.
- 4 h vs 8 h returns differ in what they bring (one more wall tried, a named find).

## 5. Friction

- Check-in taps: target ≤ 4 sheet taps per return (ui QA script counts).
- Hunger's max-hp loss names its answer (a ration, a cook, a tactic).
- D1–D4 pickups at 1×: shorten the walk to a pickup on early floors.

Gates: focused tests per item; routine full gate without a lowered threshold; full client suite;
a fresh blind pair.

## Core: built and measured (2026-10-10, uncommitted)

**§1 numbers that agree.**
- *Header hp.* Cause: the client's `hero at N hp` read `blow.hp + blow.dmg`, and the killing blow's `dmg` is the raw hit
  (overkill: an 18 blow on 5 hp read `at 18 hp` over a trace whose rows read 5 → 0). `Death.moment_hp` (the hp the
  killing blow landed on = the trace's row before it: the previous blow's hp, else the last action's; the morgue's
  `blow N at M hp`) and `Death.moment_max_hp`. Test `death_moment_hp_is_the_traces_row_before_the_blow` (overkill met).
- *Gold ledger.* Every `gold_move` is tallied by term (`LineageState.gold_tally`, `engine::gold_term`); an absence
  diffs it from its start (`Absence.tally_before`) into `GoldSummary.ledger { earned, spent, net, terms[{label, amount}] }`:
  `carried`, `lost` (−, carried + lost = income kept), `heir`, `apprentice` (−, from his acts), `forge` (−, by hand),
  `works`, `supplies`, `tolls`, `hires`, `bank`, `other`. Σ terms = `net` = purse delta exactly. Test
  `absence_ledger_terms_sum_to_the_purse_change` (two 8 h absences with the apprentice forging, and a Session):
  terms sum to the purse delta, `other` is only what the tally filed there (nothing bypassed `gold_move`).
- *Forecast horizon.* `Forecast.hold { depth, stop, order: bank|carry, share, ends }`: the scout's wall order priced
  on the same sims (`forecast::held_ends`: a `bank` send that reaches the wall's stairs banks there with its carry on
  arriving; a `carry` keeps at least it). The depths' reach stays *reach if pushed*. Test
  `the_forecast_names_the_scouts_wall_order`.
- *Preview stability.* The sends were already seeded from camp state (`forecast_tag`: lineage seed × depth, sim
  index); two reads of one camp (memo warm, or a fresh load) price every move identically — test
  `the_same_camp_reads_the_same_advice`. The moving inputs are real camp state (the purse: +$37 on a $40 purse changes
  what the repeat packs; `next_run_id` rotates a resting named foe). Each `PkgOption` now carries `noise` (95 % half
  width of `d_past` over the paired sends) and `even` (sign test |better − worse| ≤ 1.96·√(better + worse) and
  |d_past| ≤ noise): an even move shows no delta.
- *TRY flip (client finding).* `packages::fix_pick` no longer offers a tactic whose `take_fix` would take off
  another answer to the same killer (the King: cadence ↔ boss focus in the last slot). Test
  `consecutive_king_deaths_do_not_alternate_the_fix`.

**§4 apprentice.** `ReturnReport.supply_budget { income, spent, left, reason }` when the repeat was limited
(`no_income` · `income_spent` · `purse_short`), and the apprentice's `WorkerAct.reason` carries the same code (the
repeat spends only away income, never his forge purse). Test `a_limited_supply_budget_names_its_reason`.

**§3 affixes that change the answer — deviation.** Built: each affix has its breakers
(`descent::affix_breakers`; past the Warlord armoured → boss focus · Hunter, swift → corridor fighting · kite archers,
brood → pack break · corridor fighting, vampiric → kite archers · gas step, regenerating → Hunter · gas step, enraged →
kite archers · Hunter; the Warlord's are stances, before any tactic arrives: Hunter, Guarded, Bold; never Steady).
Unbroken, the boss is warded (`AFFIX_WARD` % off the hero's blows) and furious (`AFFIX_FURY` % on his); broken, he
takes +`AFFIX_BREAK_DEALT` % and deals −`AFFIX_BREAK_TAKEN` % (250 / 65); the brood's hp price 85 % (was 95).
Tooltips name the breakers (`Affix::counter`). Probe `examples/varied_descent.rs affixes` (4 IDLE lineages, 48
paired sends; prints the top four answers and the count):

| ward / fury | answer changes | answerability | IDLE · PICKED · stance rows |
|---|---|---|---|
| 0 / 0 (Cut 116) | 1/20 | PASS | PASS |
| 45 / 50 | 10–13/20 (breakers win at the Mother and Lich) | PASS (10) | PICKED D13 1.50 (edge); stance walls FAIL (guarded none) |
| 35 / 40 | — | — | stance L5 day 8 FAIL, PICKED D13 1.25 FAIL, idle-delta −24 h FAIL |
| 30 / 30 | 7/20 | PASS | idle-delta −16 h FAIL (bound: a check-in) |
| **20 / 20 (kept)** | **4/20** | **PASS (all 20)** | **all PASS** |

Kept 20 / 20: the best value whose targeted rows hold (idle-d8/d13/d23/stall/gold/king, stance, picked-idle 2.25 ·
5.25 · 9.25, outpace, tuned-picked, random-picked, nothing-required, hands-idle, idle-delta −8 h; metrics `--cut30`
all PASS, stance walls bold 2 guarded 1 hunter 2). The 8/20 bar is **not met**. Why: the wall's pass rate is mostly
the walk, not the boss (even 4× damage on the boss moved a breaker ~10 pts), so an affix changes the best answer only
when it costs the plain best (Guarded/Bold) heavily — and that harshness slows IDLE/PICKED and flips the stance and
idle-delta rows. The probe is also lineage-noisy (the IDLE lineages move with every tuning; the plain best itself
swings guarded/bold/thief guard/gas step). Next: break the affix's escort/floor as well as the boss, or measure on
fixed lineages.
`each-system` (1-seed LOO row) is fragile here: its `pets` clause (TUNED with pets dies less) failed at 45/50, 30/30,
15-fury and with breakers alone (21.7 % vs 13.6 %); at the kept 20 / 20 it PASSES (pets 17.2 vs 20.7 % deaths;
packages 16/24 h→D23, pen 33.5/32.6, forge 33.5/30.1). The routine full gate was not run here (targeted rows only).

**307dbed hash** 9cfbd23b2027a263 → aa808dca72425139 (affix ward/fury/breakers, the brood's price); with affixes and
guests pinned off it still hashes to ddd4c46c7dc03d58 exactly.

**Tests.** `cargo test --workspace --profile fast`: 747 pass (7 new in `tests_cut117.rs`); clippy `--all-targets -D
warnings` clean; wasm rebuilt (no new exports); `tsc` clean with the new wire fields in `web/src/engine/types.ts`.
