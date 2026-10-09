# Top Steam idle games: lessons for Riddle (2026-10-10)

A research subagent read the Steam store page and the review API (English, sorted by helpfulness).
A small model summarised the review pages, so quotes are excerpts of that summary. **[inf]** marks inference.

| Game | Hook | Lessons for Riddle |
|---|---|---|
| Scapewatch: Idle MMO (88%, playtest) | queue tasks, close the tab; weight and reroll tasks; 30 skills | control that grows over time is praised; a collection log is praised; RNG as the whole game, a slow first 4 h and offline hours sold as DLC are punished (the base offline window was doubled to 16 h) |
| Melvor Idle (~92%) | idle RuneScape, offline simulated exactly | offline equals online builds trust; late-game gold piles up once its sink (Township) turns into a source |
| Loop Hero (~91%) | autonomous hero, you shape the road, a boss meter | the player controls when the boss arrives → tension; punished: low agency, repetition after 1–2 h |
| Unnamed Space Idle (~92%) | loadouts, layered prestige | new systems retire old ones; punished: opaque wording |
| NGU Idle (95%) | rebirths, humour | rebirth cycles that feel alike are punished |
| Gnorp Apologue / Nodebuster (~97%) | short, juicy, a real ending | every upgrade felt; an ending without the grind |
| Rusty's Retirement (97%) | desktop-edge cosy farm, robots | cosy, readable at a glance; punished: slow automation, few late sinks |
| Soda Dungeon 2 (~93%) | party picking + auto mode; credits earned away **[inf]** | building the party is praised; too needy, alike rebirths |
| Idle Champions (76%) | formations, restricted variants | restrictions force off-meta answers; punished: pop-ups and FOMO |
| Revolution Idle / Perfect Tower II / LBR / Idleon | many layers and currencies | needing a guide, 50+ currencies, neediness and late RNG are all punished |

**Rewarded:** offline that matches online; visibly growing control; collections that feed power; restrictions that change the best build; an ending; cosy diegetic art.
**Punished:** offline caps; RNG instead of choice; slow first hours; opaque numbers; needy "idle" loops; alike prestige cycles; gold with nothing to buy; bloat.

## Suggestions (priority order)

1. **Choose when the boss comes** (Loop Hero): a challenge chip for which band boss is next, or a boss meter; IDLE keeps a default order. Targets: sameness, tension, autonomy.
2. **Restricted variant walls** that rotate weekly, optional with a bonus (Idle Champions). Targets: mastery, surprise.
3. **A payoff that scales with time away**: each run while away banks a sealed find, opened on return; a find log with set bonuses (Soda Dungeon 2, Scapewatch). Targets: 4 h ≈ 8 h, return.
4. **Gold sinks that renew**: consumables per send, works that open forks, gold → legacy at a falling rate (lesson from Melvor's Township). Targets: idle gold.
5. **One ledger, one horizon, fixed-seed advice** (Melvor's exactness). Already Cut 117 §1–2.
6. **A single tap on return**: highlights, then Collect & send; at most one decision prompt (Scapewatch). Targets: tap count, pacing.
7. **Collapse the early floors**: safe floors resolve fast; the first boss in about 15 min (Nodebuster, Gnorp). Targets: pacing, feel.
8. **A visible control ladder**: weights → rerolls → orders → pen, read as progress (Scapewatch). Targets: autonomy.
9. **A town crier** line or animation for notable acts (Scapewatch broadcasts). Targets: feel, surprise.
10. **Keep offline uncapped and show the ending**: a forecast of when the King falls (Gnorp, Nodebuster). Targets: pacing.

Sources: the Steam store and review API for app ids 4671380, 1267910, 1282730, 2471100, 1147690, 1473350,
3107330, 2666510, 946050, 627690, 1476970, 1454400, 1468260, 2763740, 1353300, 1197260; Steam discussion threads
for Melvor (offline, gold), Loop Hero, Soda Dungeon 2 and Idle Champions; vaporlens.app; raijin.gg.
