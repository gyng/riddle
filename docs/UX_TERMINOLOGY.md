# Plain-language terminology audit — 2026-10-05

Owner: audit and fix obscure terminology. Prioritize labels that name an action,
resource or result. Character names, dungeon names and Chronicle remain. This
changes presentation, not rule verbs, wire identifiers, saved keys or game rules.

| Previous wording | Player-facing wording | Where |
| --- | --- | --- |
| packages | tactics | town control, panel, currency caption, glossary |
| stance | combat style | equipped choice, selection, glossary |
| tactic | extra tactic | optional slots, glossary |
| temperament | personality | equipped choice, selection, glossary |
| drills | boss counters | Details, glossary |
| rows | rules | Tactics Details |
| the pen | custom rules | glossary, rule-source fallback |
| levels packages | upgrade tokens | first-time resource caption |
| marks | upgrade tokens | report, glossary; ◆ stays the resource icon |
| renown | reputation | report, ending, glossary |
| vault | stored gear / storage | town control, inventory, loot destination, glossary |
| cage | loot choice / loot preference | live item selection / automatic choice settings |
| party | companions | town control, companions panel |
| loadout | supplies | town control; equipment is selected in stored gear |
| trace | decision log / log | full control / short exit caption |
| oaths | challenges | panel, report, glossary |
| forswear | cancel | refund action; amount unchanged |
| camp | town | return controls |
| past +N | deeper +N | optional tactic outcome comparisons |
| death ±N | deaths ±N | optional tactic outcome comparisons |
| bank ±N | full haul ±N | optional tactic outcome comparisons |
| banked | full haul | run result labels |
| ends | run outcomes | forecast header |
| reach Dn | floor n | Tactics forecast headline |
| works | workers / town upgrades | automation sheet / permanent town improvements |
| orders | run setup | supplies, loot and start preferences |
| ledger | enemy guide | creature knowledge, counters and taming |

Tooltips explain percentage-point changes, token spending, stored gear surviving
hero replacement and automatic loot preference. Existing Progress stopped and
no rule/bad luck verdict wording already avoid plateau/gap/dice in primary UI.
Rust supplies a brief behavior description for every combat style, extra tactic
and personality, including custom rules; the client displays it without deriving
policy. Lore names such as Unbowed now have a visible plain-language purpose.

Tactics simplification contract: UX_TACTICS.md. Expanding Details exposed a sheet
placement bug: ResizeObserver watched the prepended close stud. Observe the built
content instead, so expanded controls remain scrollable and within the available
screen area. Keep word budgets, numeric sim gates and paid-action confirmations.

Challenge messages also read Challenge complete/failed; the board reads
Challenges, active slots read Active, and boss counter toggles read Counter.
First-home and report tests previously assumed an already removed crate/class
bar and unfolded details. Checked unchanged HEAD to confirm those expectations
were stale, then updated them to current first-home/bloodline contracts. Numeric
4/5/12-control gates stay unchanged. An unavailable building caption is passive
text; when ready it becomes a keyboard-accessible Build control. This also
removes its duplicate interaction from the town density count.
