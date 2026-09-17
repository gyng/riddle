# QA brief (paste to a fresh agent; no scores, lapses only)

You are a QA player for a browser game called Riddle, an auto-played roguelike whose hero runs
on rules the player writes (≤ 8 rows `conds → verb`), with offline runs and a death screen that
names the rule. You are not a rater and you award nothing. Your job is to play the rater
protocol and file **every lapse**: anything you misread, anything that did nothing when tapped,
any number that does not reconcile with another number on screen, any copy you could not
explain from what the game showed you, anything that contradicts what the game said earlier.
Read `eval/RATING.md` (the protocol) and `docs/FUN_EVAL_IDLE.md` §"human probes" so you know
what a rater looks at. Do not read PLAN.md, research/, docs/CUT*.md or eval/cards/.

Setup: the game is served at `{URL}`. Play it through the GPU browser harness:
`import { launchGpu } from "/home/g/p/riddle/tools/browser.mjs"` in a Node script run from
/home/g/p/riddle (the root package has playwright), phone viewport 400×800 at
deviceScaleFactor 3; one browser context for the whole session; screenshots to look, `document.body.innerText`
to read, click buttons by their visible label. Scripts, notes and screenshots under
`scratchpad/{QA}/`. Open `{URL}?dev=1&seed={SEED}&fresh=1` once at the start.

Horizon: 40 minutes of active play from a fresh lineage (send, watch two runs in `fights` and
two in `fast`, use `▶▶|` in both, read every death and every because-link, tap `watch` on one,
edit rules by hand at least four times, buy every card and automation you can afford and read
each sheet, drop a row, reorder rows, pack supplies, set the vault preference, open every sheet
in the camp and the report), then load `{URL}?dev=1&absent=8h` once, then 20 minutes on return
(read the report line by line: do RUNS, BANKED, RETURNED, DEATHS, the ledger and the gold sheet
reconcile? do the supplies at camp match what the next run carries? does the first run after
the return start fresh under your edited rules?). Reproduce anything odd once before filing it.

File `eval/qa/{BUILD}.{QA}.md`: one line per lapse — `screen · what you saw (quoted) · what you
expected · repro (seed, step)` — grouped under `defect` (wrong or inert), `misread` (you got it
wrong and the screen let you), `unexplained` (copy or number with no source on screen), and
`friction` (more taps than the action needs). End with the count per group. Do not edit any
other file. Do not git commit.
