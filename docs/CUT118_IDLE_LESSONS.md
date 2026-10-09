# Cut 118: lessons from the idle genre

*2026-10-10. Owner: "implement all recommendations" from research/IDLE_STEAM_2026-10.md. Starts after Cut 117
lands (Cut 117 already holds item 5: one ledger, one horizon, fixed-seed advice). All gates hold without a
lowered threshold; IDLE's games change only where an item says so and the IDLE bars are re-measured.*

## Core (riddle-core truth)

1. **Choose when the boss comes.** A `challenge` order: the player picks which met band boss the next send
   seeks (one of the band's bosses ahead within reach), or `in order` (the default; IDLE). The forecast
   prices each choice. Gate: IDLE unchanged (default); no single order dominates on PICKED's panel.
2. **Restricted walls (trials).** One optional trial per real week, drawn from (week, lineage seed): a boss
   plus a restriction (`no boss focus`, `brood + swift`). Opt in from camp; a clear pays a legacy bonus and
   a trial mark. Never required; IDLE never opts in. Gate: every trial answerable by arrived tactics (probe).
3. **Away finds that scale with time.** Each completed run while away seals one find (cosmetic, a legacy
   shard, a small set piece); opened on return in the report/reel. A second table opens after 6 h away. A
   find log of sets gives small permanent bonuses (≤ 2 % each). Never decides a wall. Gate: IDLE bars hold;
   8 h brings ≥ 1.8× the finds of 4 h on the median seed.
4. **Gold sinks that renew.** After `Kit complete`: per-send supplies (a ration or a one-run hireling, priced
   by depth) under the apprentice's order, works that open a fork early, and gold → legacy at a falling rate
   (a tithe). No sink returns more than it costs. Gate: purse at day 14 on PICKED is no longer monotone-
   rising; workers-daily holds.
5. **Fast early floors.** Floors whose forecast survival is ≥ 99 % resolve as one beat (the watch plays them
   at 8× or skips with a summary line). First D8 boss ≤ 15 min of first launch on the dev walk. Gate: IDLE D8
   day 1 holds; replay hash recorded with a reason if it moves.

## Client

6. **One tap on return.** The report becomes one card: the highlights (finds, bosses, deaths, gold sum), then
   `collect & send`, with standing orders carried; detail folded. At most one decision prompt per return.
   Gate: ≤ 2 taps from opening to the next send on a routine return (cut118 test).
7. **A control ladder.** The automation you've earned shown as one rail: weights → orders → workers → pen,
   lit as unlocked, the next rung's condition shown. Within the clutter budget.
8. **A town crier.** One diegetic line (and a small animation) in town per notable act since the last visit:
   a boss slain, a set completed, a trial cleared. ≤ 3 words in chrome; longer in the crier's scroll.
9. **The ending in sight.** A forecast line of when the King may fall at the current pace (`King · ~day 9`),
   and `uncapped` stays a headline of the away screen.

Gates: focused tests per item; the routine full gate; the full client suite; a fresh blind pair.

## Round 2 amendments (research/IDLE_STEAM_2026-10_ROUND2.md)

- §1 challenge: **boss tokens**, earned from runs (one per band boss slain), banked across heirs. Spending one
  seeks that boss on the next send. No tokens means `in order`.
- §2 trials: the last 4 weeks stay open; every trial can be cleared offline; the legacy bonus is capped (≤ 1 day's legacy).
- §3 finds: one reveal, best first; the sealed count is shown on the away screen; never a click per find.
- §4 sinks: they add and never require upkeep; the tithe shows its rate; one sink per screen.
- §5 early floors: the swift descent is earned (3 clears of a band boss → the floors above it swift) and announced.
- §6 return: the one prompt shows its default; nothing routes through a shop.
- §7 ladder: each rung names the chore it retires and the feat or time that lit it. **Feats light rungs**
  (a trial or boss clear), with the existing time fallback so IDLE is never gated.
- §8 crier: in the hero's voice, from real trace events, short. Grows into **the hero's diary** (a scroll of
  dated lines from the trace, not chrome).
- §9 ending: a % bar to the King, and what comes after the King.
- New: **a roster preview of the next wall** (its boss, affix, guard and their counters) on the camp chart.
- New: **requests from the hero**: one wish at a time (`a lantern`, `a pet`), upside only, never decaying;
  granting one pays a small morale or legacy bonus.

## Owner amendment (2026-10-10): heroes yolo; death is progress

Owner: "yes [heroes yolo], but also think about how to make deaths core to progression/gameplay and less
punishing / less upsetting for the player". Measured basis: the scout's wall order bought no depth (mean best
28.58 vs 28.61, D18 at the same hour) and 26 % more gold in a game whose late gripe is gold with nothing to buy.

1. **No wall order.** The scout's `at a wall` order (`bank · carry · push`) is retired; every send pushes.
   "Bank" means only the exit share again. The forecast's `hold` line goes with it. Record checkpoints still
   secure carry permanently, even on death.
2. **The wall learns you back (siege).** Each heir who dies at a band boss adds to that boss's *siege*: a lineage
   counter per boss (tries, the best hp the boss was left on). Each try gives the next heir a small, capped edge
   against that boss alone (e.g. +4 % damage per try up to +20 %, kept until the boss falls, then cleared and
   remembered in the chronicle). Shown as `Queen · 4 tries · best 22 %` on the death screen, the report and the
   wall preview. Deterministic; IDLE gets it too. Gate: IDLE bars hold (King never in a fortnight); dice deaths ≤ 5 %.
3. **The grave.** A dead heir's carry lies where he fell, as a named grave (`Ada's pack · $1 240`). The next heir to
   reach that floor recovers it (one grave per floor, the newest kept; older ones settle into the chronicle).
   The death's exit share stays 0 % and the 30 % heir purse floor is unchanged; the gold comes home only when a
   later heir walks there. Gate: no gold created (a grave holds only lost carry); the ledger shows `recovered`.
4. **A memorial, not a loss.** The town keeps a graveyard: a stone per fallen heir with the cause as an epitaph
   (`fell to the Queen's mirror, D28`). On the death screen the lead line is the progress (`the Queen · try 4 ·
   +16 %`), then the cause, the trace and the lever, as now. A slain wall boss names the heirs who wore it down.
5. **An echo (optional, if the bars allow).** On the next try at the same wall the fallen heir's ghost fights one
   round beside the new heir (a callout `Ada's echo`). Cut if it moves a bar.

Every death still names its cause, trace and cheapest lever (AGENTS.md).

### Owner amendment 2 (2026-10-10): death feeds the line, with no added micromanagement

Echo (item 5) is **dropped**. Added, under a no-new-taps rule ("also consider reduction of micromanagement /
automation ladder; heir choice should be in a way that's automatable"):

6. **Heir choice as a standing order.** On each death the core offers 3 heirs (trait draws from the lineage
   seed). One of them always leans toward answering the killer (a gift that answers the killer's tags: Guard vs a
   boss, Quick vs swift, Mend vs poison). Which heir succeeds is set by a **standing order, set once**:
   `answer the killer` (default) · `strongest` · `surprise me`. Nobody is prompted on death: the order picks, both
   offline and online. The report names the choice (`Bram · Guard on bosses · answers the Queen`). The player may
   change the order (one chip) or, on the death screen, swap to another offered heir before the next send (an
   optional fold, never a prompt). The order is a rung of the control ladder (`heirs` · retires `picking heirs` ·
   lit by the first death). IDLE takes the default. Gate: the IDLE bars hold; no new required tap (cut118 taps test).
7. **Deeds → Legacy.** A death pays Legacy for what that heir did (firsts, the depth past the line's record, siege
   tries), within the current Legacy economy (redistributed, not inflated: total Legacy by day 14 within ±10 %).
   The death screen leads with it: `+3 Legacy · Queen try 4`.
8. **Bloodline titles.** A siege won names the line (`Queensbane`), with a small lasting perk against that boss's
   kind (≤ 3 %; no wall-deciding), shown on the hero card, the graveyard and the chronicle. One title per boss.

## Core: built and measured (2026-10-10, uncommitted)

All truth in `crates/riddle-core/src/feats.rs` (one `LineageState.feats`, serde-default, empty on an old save) with
hooks in `engine.rs` (send, run end, snapshot, xp, rest, news days), `tree.rs` (the apprentice's sinks, the retired
wall order), `traits.rs` (the heir order), `systems.rs` (feats light rungs, `lit_by`), `oath.rs` (the roster
preview), `returns.rs` (the pick's default), `offline.rs`/`bloodlines.rs` (the report). Every draw is a pure hash of
the lineage seed and the run/week/heir — no game dice. `feats.off` pins every Cut 118 system off (probes, tests).

**Owner amendment (heroes yolo; death is progress).**
- *No wall order.* `tree::wall_hold` is always `None` (`tree::WALL_ORDER_RETIRED`); every send pushes;
  `Forecast.hold` is never set (the field stays on the wire, absent). The scout's wall ledger (`note_wall`) is still
  kept (old saves, reads). Record checkpoints unchanged. The four Cut 114/117 tests of the order are kept `#[ignore]`.
- *Siege.* A death on a band boss's floor (boss not slain that run) adds a try to `feats.siege[boss]` (tries, the
  lowest hp % he was left on, the heirs' names). The next heirs' blows on him alone take +`SIEGE_PCT` 2 % a try, cap
  `SIEGE_CAP` 10 % — **deviation: cut from the owner's 4 %/20 %**, which moved `tuned-picked` (TUNED vs PICKED at D33
  1.08 < 1.15, 16 seeds: the siege does for PICKED what the pen's counter rows do for TUNED); at 2/10 it is 1.17–1.18 PASS (`Run.siege`, `turn.rs` after the affix's ward). Slain: the siege clears, the heirs are named
  (`feats.worn`), a report line `siege_won`.
- *Graves.* A death's lost carry (`ExitLine.carried − kept`) lies at its floor (`feats.graves`, one a floor — the
  newest keeps the older's gold). A later run that reaches the floor brings it home at its end: `gold_move(+gold,
  "recovered Ada's pack")`, a ledger term `recovered` (the ledger also gains `sinks`). No gold is made: the graves
  and the recovered tally sum exactly to the carry lost (test). The death's exit share stays 0 %, the heir purse as is.
- *Memorial.* `feats.stones` (cap 40): heir, name, depth, cause, epitaph (`fell to the Mother, D13`), day, try,
  edge, grave gold, Legacy. `Death.memorial` (filled by `Game::death`): `lead` `+3 Legacy · Mother try 4 · +16%`.
- *Echo:* not built (owner).

**Owner amendment 2.**
- *Heir order* (`StandingSwitches.heir`: `answer` default · `strongest` · `surprise`). At the wake one card always
  answers the last killer (`traits::answer_gift`: a boss → Guard on bosses, gas/poison/drain → Mend, fast → Quick,
  else Guard): when none of the three does, the last fresh card gives way (never a twist or a marked card). The order
  picks the worn card; nobody is prompted. The swap is the existing `setTrait(chip)` before the next send. The rung:
  system `heirs` (trigger `first death`, opens in the death screen's unit — no extra reveal). The report: a `heir`
  line (`Bram · guard on bosses · answers the Mother`). **Deviation:** `traits.json` ships no shape today (`shipped`
  is empty), so in the current game the offer is empty and the order has nothing to pick; it acts as soon as shapes
  ship (tested on the generator's table). IDLE's games are unchanged by it.
- *Deeds → Legacy.* A siege try pays `DEED_LEGACY` 1 more to the heir who made it (the run's own Legacy is the
  rest of the lead line). Probe `deeds_stay_within_ten_percent_of_legacy` (IDLE fortnight, 4 seeds): deeds are
  1.7–2.1 % of all Legacy (1 505–2 092) — within ±10 %.
- *Titles.* A siege won names the line (`Motherbane`, into `Lineage.titles` too), +`TITLE_PCT` 3 % on that boss
  (his kind alone), one a boss.

**Cut 118 items.**
1. *Boss tokens:* `feats.tokens` (+1 per band boss slain, cap 3, kept across heirs). `seekBoss(boss)` sets the order;
   the next send spends one and starts at the deepest lit stone at or above him, with no wall hold. `seekForecast()`
   prices each (reach, past, death from his stone, one camp panel each). A sought boss slain is a feat.
2. *Trials:* `trial_of(week, seed)` — a band boss (not the King) and a rule: `no boss focus · kite archers ·
   corridor fighting`, or two of his affixes (both worn: `populate_floor` applies every affix on him). Weeks open:
   this and the three before. `setTrial(week)` opts in; the next send while away plays it (from his stone, pushing).
   A clear (boss slain, the bar unbroken by the set sent) pays `TRIAL_LEGACY` 10 (≤ the pick's largest, 12; a day's
   runs pay ≫ that) and a mark. Probe: every boss × rule answered by packages (stances answer every bar; every affix
   has a breaker) — `trials_are_weekly_answerable_and_pay_capped`.
3. *Away finds:* one sealed per run ending away (table 2 after 6 h), opened on the return: `ReturnReport.finds`
   (`sealed`, `best`, `finds` best first, `legacy` from shards, `sets` completed). Sets (3 × 3 pieces): Sleeper's rest
   −2 %, Scholar's class xp +2 %, Hearth +1 Legacy a return. Gate (test `away_finds_scale_with_time`, 5 seeds): 8 h / 4 h finds 1.78 · 1.89 · 1.89 · 2.00 · 2.00 — median **1.89 ≥ 1.8** PASS.
4. *Sinks after Kit complete* (`StandingSwitches.sink`: `both` default · `ration` · `tithe` · `off`, the
   apprentice's): a ration a send (1 unit, +10 % max hp for that run), the tithe on the hour (≤ 3 points, the price
   rising a unit per 6 points bought; reads purse + bank above 20 units), the survey work (`buySurvey`, 20 units:
   the D19/D24 forks open). Each takes gold and returns none (test). Hand: `tithe(n)`.
5. *Swift floors:* a band boss cleared 3× (`kill_counts`) makes the floors above him swift — `Snapshot.swift`,
   `feats.swift_to`, a news line `D1–7 swift` once. Presentation only: the sim is untouched.
6. *Feats light the ladder:* a trial clear or a sought boss slain lights the next unit whose trigger has come but
   whose age has not (outside the one-a-report budget); `SystemInfo.lit_by` names trigger / `time` / `feat: …`.
7. *The hero's wish:* one at a time from the first return, never decaying; `grantWish()` pays half a unit for 2–3
   Legacy; the next comes a day on.
8. *Wire:* `Lineage.feats` (`FeatsWire`), `Lineage.king_eta_h`, `Lineage.king_pct`, `ReturnReport.finds`,
   `ReturnReport.feats[]` (`k · text · day`), `News.day`, `BossWall.affix · affix_counter · guard · guard_counter`,
   `ReturnPick.default` (`legacy`), `SystemInfo.lit_by` (`trigger` already existed), `Snapshot.swift`,
   `StandingOrders.sink · heir`, `Death.memorial`. Inputs (wasm): `seekBoss(boss)`, `seekForecast()`,
   `setTrial(week)` (negative opts out), `tithe(n)`, `buySurvey()`, `grantWish()`; the heir and sink orders through
   `setOrders`; the heir swap is `setTrait(chip)`. `web/src/engine/types.ts` carries every field; the native bridge
   is regenerated.

**307dbed hash.** aa808dca72425139 → **2f3eb706b3d24c7c**: the graves (a later send brings a dead heir's carry home,
so the purse hashed after each send moves). With the graves alone off the tree hashes to aa808dca72425139 exactly;
with every Cut 118 system pinned off (`feats.off`) too (test `the_307dbed_hash_holds_with_the_systems_pinned_off`).
News days are stripped from the hash as a new read of the same run. The siege, the heir order and the retired wall
order move nothing in these ten sends.

**Gates.** `cargo test --workspace --profile fast`: 759 pass, 0 fail, 6 ignored (the four retired wall-order tests,
the deeds probe, one older); 17 new in `tests_cut118.rs`. Clippy `--all-targets -D warnings` clean; wasm rebuilt;
`tsc` clean; copy-lint 0. Metrics `--cut30` (8 rows) all PASS (stance walls bold 3 · guarded 1 · hunter 2; Steady
the safest). Dayplayer rows, 16 seeds (`--full --rows`), siege at 4/20 for the whole table, then 2/10 re-run:

| row | at siege 4/20 (whole table) | final (siege 2/10) |
|---|---|---|
| IDLE D8 day 1 · D13 median day · D23 by day 12 | 16/16 · 1.7 · 16/16 (4.3) | 16/16 · 1.7 · 16/16 (4.3) PASS |
| IDLE stall · gold every day · King in a fortnight | 3 d · 14/14 · 0/16 | 3 d · 14/14 · 0/16 PASS |
| stance L3 / L5 (IDLE median day) | 1 · 6 | 1 · 6 PASS |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 2.50 · 3.00 · 3.75 | 2.50 · 3.00 · 3.75 PASS |
| IDLE never out-paces PICKED | 98 % | 98 % PASS |
| TUNED beats PICKED at D33 (≥ 1.15) | **1.08 FAIL** | **1.17–1.18 PASS** (fail-fast, 2 seeds left) |
| RANDOM slower than PICKED | 16/16 · 16/16 | 16/16 · 16/16 PASS |
| workers-daily (mean best ≥ by hand − 1) | 31.16 vs 30.82 | 31.15 vs 30.77 PASS |
| automation-pays (D18 sooner or deeper mean best) | D18 48 vs 48 h · 29.09 vs 28.42 | D18 48 vs 48 h · 29.13 vs 28.56 PASS |
| hands-idle · idle-delta · conserved | −4 · +0 · −32 · +8 h · 0 off | same PASS |
| return-pick · grew · idle/picked stages · reach · stalls · porter · scout · nodes lit/bought/required | PASS (672/672, 10.0, 12.0, 16/16, ≤ 0.15 %, 9.6 min, 8.9 min, 0 · 0) | not re-run (no input to them moved) |
| each-system | PASS (packages 24/68 h→D23 · pen 33.1/32.8 · forge 33.1/28.7 · pets 32.5/33.4 % · bank · quests) | not re-run |
| nothing-required | **FAIL** (seed 14 forge +64 h) | **FAIL** (same) — deviation below |

Dice deaths (`metrics --quick`, final tree): **4.3 % ≤ 5 % PASS** (n = 303, death-weighted, ±2.5). That legacy quick
table also lists five cohort-set rows failing (lanes D5 EDITED, thief guard 25 %, card↔chore loops worst 1.8 %,
return row 36/40, waystone $/h 37/48) — legacy cohorts outside the routine gate, not compared against HEAD here.

**Deviation — `nothing-required` (TUNED − forge, seed 14).** With the amendment's required pieces on, one seed's
TUNED − forge reaches D29 64–80 h after IDLE from day 5 (cap 48 h); every other clause (43) holds. HEAD passes; with
every Cut 118 knob off the row passes; and it still fails with each one alone off — graves off (+64 h), the wall
order restored (fails on seed 4, +80 h), finds' bonuses off (+80 h), the siege at 0 (+64 h), finds and siege off
(+80 h). It is the combination of the owner's yolo + graves on one fragile seed (the row was already noted fragile in
Cut 117), not one system; no threshold was lowered. Next: measure the wall's push vs the forge-less build on seed 14
(the Queen, D28) — a lighter siege edge for a heir without a forge would be the first lever.

**PICKED's purse (item 4's gate).** With the sinks the wealth (purse + bank) of PICKED stops climbing once the kit is
complete: seed 1 hovers $16–27 k from day 2 to day 14 (before the bank-reading tithe it climbed every day to $430 k), falling
on 4 of 13 days. The dayplayer prints the row as an info (`Cut 118: PICKED wealth no longer monotone by day 14`).

