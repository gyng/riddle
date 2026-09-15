# Calibration run — idle presets (2026-09-15)

Five reference titles scored on the idle presets in `eval/presets.json` by one evaluator
from own play plus the reviews in `research/`. Anchors as `../eval-fun/docs/FRAMEWORK.md` §3;
idle boundaries as `docs/FUN_EVAL_IDLE.md`. `eval/score.sh eval/calibration/<name>.scorecard.json`.

| Title | Preset | Score | Verdict | Reputation |
|---|---|---:|---|---|
| Cookie Clicker | idler | 75.5 | fun | the reference idler; eval-fun scored it 74.7 on its own idler preset |
| Loop Hero | idle-roguelike | 74.1 | fun | 87 Metacritic, Very Positive; "one more loop" then late grind |
| Melvor Idle | idle-rpg | 65.4 | promising (gated: surprise) | Very Positive; "completionist, not traditional fun" |
| Mediocre idle RPG (composite) | idle-rpg | 30.8 | not yet | the 2-star auto-battler with timers |
| Progress Quest | idle-rpg | 28.5 | not yet | a joke that works for an hour |

## What the run says

- Adding `return` and `attribution` moved Cookie Clicker by under one point: the idle
  axes do not inflate a game that already had a good check-in. They are not free points.
- Loop Hero lands in *fun*, not *super fun*, on the preset built for Riddle. The gaps are
  `return` (it does not reward absence) and `pacing` (late grind). That is the target
  Riddle must beat, and the two axes it must beat it on are exactly the ones an idle
  roguelike exists for.
- Melvor is gated on `surprise` at weight 2 in `idle-rpg`. That is the intended reading:
  the community's own verdict is that it retains without surprising. If a second rater
  finds this harsh, lower `surprise` to 1 in `idle-rpg`, not the award.
- Progress Quest and the composite sit 40+ points below the greats and fail six gates
  each, all in the engine and idle-core layers. Discrimination holds.

## Caveats

One evaluator, one sitting; α not measurable. Cards exist so a second rater can score
blind. Loop Hero is the re-anchor title: re-score it in the same sitting whenever the
rubric is used after a gap of a month; a drift of more than 5 points means the anchors moved.
