# Rating protocol — how a Riddle build gets a fun score that counts

The coordinator's own card is a baseline, never a decision (`../eval-fun/docs/FRAMEWORK.md`
§4: two independent raters on the same immutable build and horizon, Krippendorff α ≥ 0.67
tentative, ≥ 0.80 decision). This file is the precommitted contract for a rating cohort.

## Common contract (fill per cohort, then never change it)

| Field | Value |
|---|---|
| Build | git commit hash; `tools/wasm.sh --ship` then `cd web && pnpm build`; served from `pnpm preview --port 5230` |
| Save | fresh lineage, seed pinned per rater (`?dev=1&seed=<n>&fresh=1` (production builds honour dev params only with `dev=1`)), raters get different seeds |
| Tier | 1 (one hour plus one absence) unless stated |
| Horizon | 40 min active cold, then `?absent=8h` (simulated overnight), then 20 min on return; at least 3 deaths read, at least 2 rule edits attempted |
| Audio | none in the build → `feel` audio unassessed, not scored |
| Input | pointer at phone size 400×800@3 via the GPU harness (`tools/browser.mjs`); desktop 1280×800 allowed for the editor |
| Tools | the rater may use `node tools/playtest.mjs` for screenshots but must play the decisions itself; the CLI (`examples/cli.rs`) is allowed as a second view of the same lineage |
| Telemetry | none until the card is locked; bot caps (`node tools/gates.mjs`) are applied by the coordinator afterwards |
| Blindness | raters do not read `PLAN.md`, `research/`, `docs/CUT*.md`, other cards or the coordinator's card; they read only this file, `docs/FUN_EVAL_IDLE.md`, `../eval-fun/docs/FRAMEWORK.md` §1–3, and the template |

## Rater steps

1. Play the first 10 minutes without notes. Then screenshot: first minute, first death,
   first edit, the return screen, the first edit after returning.
2. Answer the criterion probes (fun 1–7, play again tomorrow 1–7, recommend 0–10, check in
   tomorrow unprompted 1–7) **before** awarding any axis.
3. Answer the sixteen human probes in writing, one sentence each, quoting something you saw
   (a callout, a death line, a number). No rubric words.
4. Award on the five anchors (0 / 0.3 / 0.6 / 0.8 / 1.0). For 0.8 name the lapse; for 1.0
   record that no lapse occurred across the horizon and name the genre benchmark it beats.
5. Fill the `horizon` block honestly (minutes, absences, runs watched, runs offline, edits).
6. Retell your best run in three sentences at the end of the card (`criterion.delayedRecall`
   is filled the next day by the coordinator re-asking; leave it empty).
7. Save the card as `eval/cards/<build>.<rater>.json` and stop. Do not discuss with anyone.

## Coordinator steps

1. `eval/score.sh eval/cards/<build>.<rater>.json` for each; apply bot caps from the gate table.
2. `node ../eval-fun/tools/fun-reliability.mjs a.json b.json` → α. Below 0.67: publish no
   combined score; list the disputed axes; fix definitions or the game; new blind cards.
3. Gap list = lowest weighted awards on load-bearing axes, with the raters' quoted evidence.
   That list is the next cut's contract.
4. Never rewrite a raw card. Cards are the evidence trail.

## QA cohort (from Cut 12)

The 1.0 anchor needs no material lapse across the horizon; blind cohorts kept finding defects
after shipping. Before every blind cohort, two **QA players** (agents, not blind to the goal,
no scores) play the full rater protocol on fresh seeds from `eval/QA_PROMPT.md` and file every
lapse as `eval/qa/<build>.<qa>.md`: anything misread, anything inert, any number that does not
reconcile, any copy they could not explain, with the screen text or screenshot that shows it.
Every item is fixed or recorded as a deviation with a reason; the build reships; only then the
blind cohort. The QA reports are evidence, not scores; they never touch a card.

## Re-anchoring

Before a cohort, the coordinator re-scores Loop Hero on `eval/presets.json` in the same
sitting (`eval/calibration/loop-hero.scorecard.json`); a drift over 5 points means the anchors
moved and the cohort waits.
