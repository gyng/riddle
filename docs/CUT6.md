# Cut 6 — Clarity: every number reconciles, every silence speaks

*Contract, 2026-09-17. Cohort 2 (build 186d0ac, raters C seed 37 and D seed 41): 63.5 (gated
clarity 0.3) and 68.4, α 0.809, mean 66.0 (cohort 1: 63.7). Story 0.45 → 0.70, tension
0.60 → 0.80 as intended. Clarity fell to 0.45 and is now the largest deficit (lost weight
1.65), then surprise / pacing / expression (1.20 each). Both raters' clarity evidence names the
same four silences.*

## Findings → changes

| # | Rater evidence | Change | Track |
|---|---|---|---|
| 1 | C: "bailed with $84 in the header, report says Returned with $50, camp shows $68; each death quietly takes ~$110"; D: "$94 header, reel $56, camp $69; bank vs return unexplained" | **The ledger line.** Every exit produces one arithmetic line the player can check, shown on the exit sheet, the report and the death screen: `$84 carried · return keeps 60% → $50 · supplies −$12 → $68` (death: `$144 carried · death keeps 0% → $0 · bones: 7 items on D5`). The HUD stake shows `$84 · return keeps $50` while a return row exists (the kept number, not the carried one). Supplies and insurance purchases appear as their own lines in the camp gold tooltip-free ledger: a `gold` sheet listing the last 20 gold movements (`+$50 returned D5`, `−$40 heal`, `−$8 insure sword`). | core (`Lineage.ledger: Vec<GoldLine>`, `ExitLine` on `Ev::Exit`) + client |
| 2 | C: "'hp<30% → drink heal' never fired at 3 HP across three deaths with heal potions bought" | **Bug.** `find_consumable` demands the *flavour* be identified even for an item bought by name; a shop-bought heal is unusable until the player happens to identify a found heal. Fix: items bought or crafted are `known: true` on the item itself; the identified check applies to found items only. Test: buy heal, hp 3 → `drink heal` fires. | core |
| 3 | C: "the trace only lists rules that fired, so I cannot tell if it is a bug or a mechanic"; cohort 1 said the same about retreat | **Every row is accounted for at the low point.** The death trace's last turn expands: for each row above the one that fired, one word why it did not (`R1 drink heal · none held`, `R2 retreat · no path`, `R3 attack boss · not in view`). Cut 4 added `blocked` for rows whose conds held; this adds `skipped: <reason>` for rows whose conds did not hold (the failing cond named: `hp 8% ≥ 30%` reads as `hp not < 30%`). Shown as a second, dim line under the trace: `R1 none held · R2 no path`. | core (`TraceTurn.rows: [{row, why}]`) + client |
| 4 | C: "send goes grey with no message when a bought card pushes rows to 6/5"; D: "send goes silently grey at 7/6" | The over-budget state says what to do: the disabled `send` reads `6/5 · drop one` (copy ≤ 3 words), and the row the engine would drop is the *card* by default (a card is the newest row), not the player's last rule. Buying a card with a full set asks first: the card's buy button reads `◆3 · takes a row` when rows are full. | client |
| 5 | C: "'boss · counter: known' never says what the counter is"; D: "the D6+ wall never moved" | The boss banner and the death screen name the counter as a row: `boss · counter: attack boss` / `throw fire, boss` / `read silence`, from the learned fact; the death screen for a boss death shows the counter row as the first patch whenever the fact is known (it is, by construction, the best candidate). Forecast `D6 0% · goblin warlord · counter: attack boss`. | core (fact carries the row text) + client |
| 6 | D: "the [card] kite archers row cannot be opened"; C: "tapping a card row explains nothing"; D: "bought autos vanish" | Tapping a card row or an automation in the unlock shelf opens a one-line sheet with the card's rows as chips (`foe: ranged → kite` …) or the automation's effect as a row (`quartermaster: keeps best weapon + armour`). No sentences; rows are the language. | client (`unlocks()` gains `rows?: Row[]` per card/automation from the core) |
| 7 | C: "'rest 20m' does nothing"; D: "every return shows VAULT 0/0" | Done in 7941fa7 (keep sheet only with a free slot). `rest 20m` becomes a tappable chip that reads `send skips rest` for a beat. | client |
| 8 | C: "every offered patch was a flavour of hp<20% → return (give up)" | When a boss counter is known, the escape family is ranked below targeting on boss deaths; a `return` patch is never the only patch shown on a boss death. | core (trace.rs ranking) |
| 9 | D: "forecast wobbles ±10 points between reads of near-identical sets" | Forecast seeds are fixed per (rules hash, depth) so re-reading the same set gives the same number; the sim count for the shown depth rises from 50 to 100 when the rule set is unchanged for 2 s (a second, quieter pass that refines the number). | core |

## Gates

- Ledger arithmetic: on every exit, `carried × keep% − spent == gold delta` (test over 30 seeds).
- Bought heal fires: unit test.
- Trace accounting: every death trace's last turn lists every row above the fired one with a
  reason from the reason table (unit test on 100 deaths: 100%).
- Forecast determinism: same rules + same lineage → identical forecast (test).
- Boss death shows the counter row first when the fact is known (test on 30 boss deaths).
- Copy: all new strings inside budgets; `node tools/copy-lint.mjs` clean.
- Cohort 3: `clarity` ≥ 0.6 from both raters, α ≥ 0.67.

## Tracks

- **Core** (`crates/**`): 1 (ledger + exit line), 2, 3, 5 (counter row text in the fact), 6
  (`rows` on unlocks), 8, 9.
- **Client** (`web/src/**`): 1 (sheet + HUD), 3 (dim line), 4, 5 (banner/forecast), 6, 7.
