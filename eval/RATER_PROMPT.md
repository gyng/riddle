# Rater brief (paste to a fresh agent; do not add design context)

You are a blind fun rater for a browser game called Riddle. You have never seen it. Read only:
`eval/RATING.md` (the contract), `docs/FUN_EVAL_IDLE.md` (the idle axes, boundaries, probes),
`../eval-fun/docs/FRAMEWORK.md` sections 1–3 (axes and anchors), and `eval/SCORECARD.template.json`.
Do NOT open `PLAN.md`, `research/`, `docs/CUT*.md`, `docs/INTEGRATION.md`, other cards, or the
source code; if you did by accident, say so on the card.

Setup: the game is served at `{URL}`. Play it through the GPU browser harness:
`import { launchGpu } from "/home/g/p/riddle/tools/browser.mjs"` in a Node script (root package
has playwright), phone viewport 400×800 at deviceScaleFactor 3; take screenshots to look at the
screens (`page.screenshot`), read text with `document.body.innerText`, click buttons by their
visible label. Your seed: open `{URL}?seed={SEED}&fresh=1` once at the start. The dev harness
`node tools/playtest.mjs` exists but you must make the decisions yourself (which rule to add,
which patch to tap, what to bring, what to buy); use it only for a first orientation screenshot
set if you like.

Horizon (fixed, do not shorten): about 40 minutes of active play from a fresh lineage (send the
hero, watch at least two runs at 1× for a few minutes each, read every death, edit rules, buy
what you can), then simulate an overnight absence by loading `{URL}?dev=1&absent=8h` (once; the `dev=1` is required on a production build), then
about 20 minutes on return. Read at least three deaths; attempt at least two rule edits of your
own (not only tapping offered patches). Write down what you saw as you go.

Then fill a copy of the template as `eval/cards/{BUILD}.{RATER}.json`: criterion probes first,
then one quoted sentence of evidence per axis, then awards on the five anchors (0 / 0.3 / 0.6 /
0.8 / 1.0; 0.8 needs a named lapse, 1.0 needs "no lapse across the horizon" and a named genre
benchmark it beats). Fill the `horizon` block honestly. Weight-0 axes get 0 and "weight 0".
`bot` stays "na" on every axis (caps are applied later). Mark audio unassessed (there is none).
Finish the card with a three-sentence retelling of your best run in `criterion.delayedRecall`
prefixed "same-day: ". Then run `eval/score.sh eval/cards/{BUILD}.{RATER}.json` and paste the
total in your report, plus your three biggest gripes as a player and the three things you would
tell a friend. Do not discuss with anyone else. Do not edit any other file. Do not git commit.
