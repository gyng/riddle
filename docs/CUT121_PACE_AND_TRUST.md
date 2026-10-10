# Cut 121: a longer road, numbers you can trust, a watch you can read

*2026-10-10. Cohort 1a7d834 (Cuts 117–120): A 62.7, B 72.7 (the best single card), α .583, mean 67.7. Both
raters cleared the dungeon, King included, inside the first session (~65 active minutes + 12 h away);
Ascension then restarts at D1 rats. Cut 120 found why: Legacy is cheap beside its income (≈110 ◆ on day 1,
≈2 300 by day 14; the whole base costs 54), so a player who spends it, as every rater does, flattens the
walls. Goal: 80.*

## 1. A longer road (pacing, progression, tension)

- **Legacy priced for days, not hours.** Reprice the Legacy steps on a curve, so a spending player's
  power grows over the fortnight. Gate: PICKED with Legacy spent by hand (a new `PICKED+L` bot: spends
  Legacy `balanced` at each check-in) does not slay the King before day 7 on the median seed, and never
  before day 4. IDLE bars hold. Legacy `balanced` as a default becomes possible again (Cut 120 option C):
  measure it and report.
- **Gold has a late use.** After Kit complete, gold buys depth-scaled consumables and Legacy through the
  tithe, at a rate that does not shortcut the walls. Gate: a rater-like bot's purse at day 3 is spent
  below 50 % (not $60k idle).
- **Return cadence.** An 8 h return never pays less than a 4 h one on the same camp, short of a death
  spiral. Investigate A's case (+$10 823 at 8 h vs +$14 723 at 4 h).
- **Ascension 1 is not D1 rats again.** Start it at the deepest lit waystone minus a band, with ascension
  modifiers visible.

## 2. Numbers you can trust (clarity, mastery)

- The death header's floor foe vs the killer: `goblin · D8` beside `fell to the Warlord`. Make them one
  story (the killer and his summons).
- `fell to the Mother` while the trace says `not in view`: attribute gas and summons deaths to the source
  only when it is in view, else to the hazard.
- `−$2807 GOLD CHANGE` led a return: the headline must separate income from spending (`+$X earned ·
  −$Y spent by workers`), and the tile shows earned.
- The forecast `death 75%` on an easy clear, and per-run `$` swings from one threshold change: show
  ranges, and "compare · same" only when it really is the same at the wall that stalls.

## 3. A watch you can read (feel, clarity)

- Speed never changes on its own without a visible reason (a `fights 2×` chip that says why).
- The swift-floor chip card never covers a fight. Swift floors are a one-line strip at the top.
- Deep floors that crawl with `rest +5 hp` lines: compress repeated rest lines into one counter.
- Find the hero: a ring or marker on the hero in a crowd, and a gentle camera lead.

## 4. Words (expression, clarity)

- The jargon `picks 29% · taught 2% · chores 69%`, `VS LAST RUN`, and `drink fire` vs throw: plain words or a
  tooltip, and one name per action.
- A boss rule that misfires on a non-boss (`siren`): warn on the rule (`also matches: siren`).

Gates: focused tests per item; the routine full gate; the full client suite; a fresh blind pair.

## Core: built and measured

*Core agent, 2026-10-11. Gates: `node tools/gates.mjs --full --rows <the IDLE/PICKED/TUNED/RANDOM rows,picked-l-king,picked-l-purse>`
(16 seeds, no threshold lowered); fast tests (789 passed, 6 ignored) and clippy (workspace, all targets) clean; wasm rebuilt (`tools/wasm.sh`); no
export changed (no codegen); `node tools/gates.mjs --fast` all PASS (qa 10 seeds, metrics 8 rows, IDLE 2 seeds).*

### §1 A longer road

- **`PICKED+L`** (`dayplayer`): PICKED that buys Legacy `balanced` by hand at each check-in (`tree::legacy_pick` under a
  `balanced` order, `legacy::buy_next`); it shares PICKED's game until its first buy (`Q::Legacy`). Rows `picked-l-king`
  (gated: the King's day median ≥ 7, min ≥ 4; an unslain seed counts as day 15) and `picked-l-purse`; info rows: King day ·
  D23 · D28 · D33 hours, upgrades bought.
- **Measured at HEAD content: King day median 2.0, min 2** (16/16 slain on days 2–3, every one at his first meeting; the whole
  216-◆ tree bought on day 1; PICKED+L earns ≈ 216 ◆ day 1, ≈ 580 day 2, ≈ 1 000 day 4, ≈ 1 500 day 7). PICKED without Legacy
  slew him at median day 4.5 (3–7).
- What the experiments found: one rank of armour/damage/health is a cliff at the King — +6 HP · +2 damage · +2 armour turned
  ~20–80 deaths at him into a first-try kill; his hp barely mattered (400 vs 450 hp: same days), his attack did.
- **Legacy on a curve** (`legacy.rs`): `CAP` 3 → 5; a root's ranks cost `ROOT_PRICES` 15 · 600 · 1 500 · 3 000 · 6 000 (was
  3 · 6 · 9); the D8 effects 800, the D18 ones 2 000 (were 18 · 36). The whole tree is ≈ 41 700 ◆ (was 216). `balanced` now buys
  the cheaper of the next root rank in turn and the next effect (`tree::legacy_pick`).
- **The King** 220 → 500 hp and 13–19 → 19–25 (`defs.rs`).
- **Result: PICKED+L King day median 7.0, min 6** (days 6–8; D23 · D28 · D33 at 24 · 32 · 44 h median; 11 upgrades bought).
  Every other row is unchanged from HEAD (Legacy is bought by no other bot; the King stands past every milestone):
  IDLE D8 16/16 · D13 1.7 · D23 16/16 (4.7) · stall 3 · gold 14 · King 0/16 · stance 1 · 6 · return pick 672/672 · grew 672/672 ·
  stages 11.0; PICKED ≥ 1.5× IDLE 2.50 · 2.75 · 3.45; outpace 97 %; PICKED stages 12.0; reach 16/16; stalls 0 %; TUNED vs PICKED
  D33 1.17; RANDOM 16/16 · 16/16 (+4 · +52 h).
- **Deviation (recorded):** PICKED without Legacy now never slays the King in 14 days (0/16; ~165–310 tries, his least hp left
  1–48 %). No bar asks it; a non-spending engaged player meets a hard wall that only Legacy, scars and the siege's edge wear down.
  IDLE never reaches a try that threatens him (least hp left 98 %).
- **Option C re-measured** (Legacy `balanced` for new lineages *and* `Game::new_resident`, with this curve and King; a temporary
  build, 16 seeds, IDLE · PICKED · PICKED+L · TUNED · RANDOM, the routine bars): **every bar measured passes** — IDLE D8 16/16 ·
  D13 1.0 · D23 16/16 (2.3, was 4.7) · stall 2 · gold 14 · King 0/16 (closest 76 % left) · stance 1 · 6 · stages 10.5; PICKED ≥ 1.5×
  IDLE 1.50 · 2.17 · 2.33 (D13 exactly at the bar); outpace 100 %; reach 16/16; stalls ≤ 0.33 %; TUNED vs PICKED D33 1.18;
  RANDOM 16/16 · 14/16 (+4 · +20 h, 14/16 at the bar); PICKED+L King 7.0 · min 5 (PICKED, buying by order at the sends, the
  same: 7.0); TUNED slays him day 3–14 (median 5). **Not switched:** the rows the routine full gate adds — DAILY/AWAY
  (automation-pays, workers-daily), HANDS, IDLE-delta, the nodes rows — were not measured under C in the time, and two bars sit
  exactly at their thresholds (PICKED ≥ 1.5× at D13 1.50; RANDOM 14/16). `tree::NEW_LEGACY` stays `off` (option B);
  `tree::new_legacy()` is the one switch point (a lineage's `orders` now take it at `LineageState` creation).
- **§1 Ascension:** a numbered descent (`begin_descent`, the herald's `carry_on`) starts at the deepest lit stone at or under
  `deepest − ASCENT_BAND` (15), never above `ASCENT_MIN` (D9); the stones to it stay lit and the record starts there
  (`endgame::ascent_start`; from a cleared line with D29 lit: D14). Modifiers on the wire: `Lineage.descent`
  (`tier · start · hp_pct · atk_pct · modifiers[]`, the ones at work at that tier).
- **§1 Gold's late use:** `picked-l-purse` passes at HEAD and after (100 %, 14/16 seeds with Kit complete by day 3; income since
  ≈ $40 500): the tithe and rations take nearly all of it. `examples/cadence --sinks` (16 day-3 camps, 8 h, sinks `both` vs `off`):
  no sink returns more gold than it costs (gain ≤ cost on 16/16; the largest gain $3 639 against $21 659 spent).
- **§1 Return cadence:** `examples/cadence DIR` (64 PICKED+L camps, days 1–4, 4 h vs 8 h from the same camp): the 8 h
  **income** is never below the 4 h one (median ×1.93, least ×1.30); the purse's **net** was below on 3 camps, every time the
  apprentice's tithe (workers' spend 0 → $9 900; 0 → $28 347). Rater A's `+$10 823` vs `+$14 723` is that: the tithe starts once
  the town's wealth passes 20 units, so a longer absence spends more of the same income. Fixed by §2's report split (the
  headline leads with income), not by the sinks.

### §2 Numbers you can trust (core)

- **Death attribution:** the run records the killing blow's summoner (`Run.death_summoner`, `turn::summoner_of`: the floor's
  summoning boss first, else the nearest summoner) and a hazard's source boss only when he was in view at the blow
  (`Run.death_source`, `turn::hazard_source`). Wire: `Death.summoned_by`, `Death.source` (short names: `Warlord`, `Mother`).
  The stone's epitaph names the killing blow, as the header does (`feats::killer_phrase`): `fell to goblin, D8`,
  `fell to goblin, summoned by the Warlord, D8`, `fell to gas, D13`, `fell to the Mother's gas, D13`, `fell to the Warlord, D8`
  (was: the floor's unslain boss whatever killed him). The siege's try still counts at his floor.
- **Report:** `GoldSummary.earned` (carry kept, heir purse, passages, graves, fetched packs) and `GoldSummary.spent_by_workers`
  (the apprentice's steps and sinks, the ranks order's buys).
- **Forecast bands:** `ForecastEnds.death_lo/death_hi` (95 % Wilson) and `gold_lo/gold_hi` (mean ± 1.96·sd/√n, ≥ 0), on the
  forecast's and the hold's ends. **Compare `same`:** `PkgOption.even` is false when any wall read leans past the noise
  (`packages::wall_differs`: |better − worse| > 1.28·√(better + worse), or every differing send one way, ≥ 2).
- **`RowStat.fired_on`** (coordinator's extra): a foe-tag row's fires by the foe kind it acted on (`RowTally.fired_on`, halved
  with the tally), each `{kind, boss, n}` — the tablet's `also matches: siren`.
- Tests: `tests_cut121.rs` (8); the Cut 32 / Cut 120 Legacy tests now read the curve (`ROOT_PRICES`, `EFFECT_PRICES`, `CAP`):
  each rank dearer than the last, bought in order.
- **307dbed hash:** did not move (its ten sends never reach the King nor buy Legacy); nothing to re-record.

### Wire fields added (`web/src/engine/types.ts`, none renamed)

`Death.summoned_by`, `Death.source`; `GoldSummary.earned`, `GoldSummary.spent_by_workers`; `ForecastEnds.death_lo`,
`death_hi`, `gold_lo`, `gold_hi` (forecast and hold); `Lineage.descent` (`AscensionWire`); `RowStat.fired_on` (`FiredOn`).
