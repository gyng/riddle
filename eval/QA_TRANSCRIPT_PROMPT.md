# QA brief, transcript-first (paste to a fresh agent; no scores, lapses only)

You are a QA player for a browser game called Riddle, an auto-played roguelike whose hero runs
on rules the player writes (≤ 8 rows `conds → verb`), with offline runs and a death screen that
names the rule. You award nothing. Your job is to file **every lapse**: anything you misread,
anything inert, any number that does not reconcile with another number on screen, any copy you
could not explain from what the game showed, anything that contradicts what it said earlier.
Read `eval/RATING.md` (the protocol) and `docs/FUN_EVAL_IDLE.md` §"human probes" so you know
what a rater looks at. Do not read PLAN.md, research/, docs/CUT*.md, eval/cards/ or other QA
reports.

Two passes, in this order:

1. **The transcript.** `{WALK}` holds a scripted walk of the whole loop on seed {SEED}: one
   `NN-<screen>.txt` (the screen's full text) and `.png` per screen, in order, and
   `summary.txt` (one line per dump with its wall time). Read every `.txt` in order in one
   sitting, looking at a `.png` whenever a line's layout matters. File the lapses you can
   see in text alone — the report's numbers against the header's, a line that repeats a
   segment, a label with no source, a pill that names nothing, a because-link that
   contradicts the trace, a screen with no way out. Note the dump each one is in.
2. **The reproduction.** Start the driver from /home/g/p/riddle:
   `node tools/driver.mjs --dir scratchpad/{QA} --port {PORT} &` (headless phone 400×800 2×;
   `tools/drive.sh {PORT} '{"op":"text"}'`; ops: goto, text, shot, click by visible label,
   tap, type, press, eval, js, wait, buttons, log, quit; compound: `state`, `act`, `watch`,
   `send_and_watch` (a send watched to its end in one call) and `sheets` (every sheet a screen
   opens, read in a copy of the page) — the file's header). Open
   `{URL}?dev=1&seed={SEED}&fresh=1`, then reproduce each transcript lapse once (the walk is
   deterministic on its seed: the same sends give the same runs), and play what the walk did
   not decide: **four hand edits of your own**, a patch tapped, two purchases with their
   sheets read, a supply dropped, the vault preference set, `?absent=8h` once, the return
   report read line by line against the gold sheet. ~15 minutes, not 60.

File `eval/qa/{BUILD}.{QA}.md`: one line per lapse — `screen · what you saw (quoted) · what
you expected · repro (seed, dump or step)` — grouped under `defect` (wrong or inert), `misread`
(you got it wrong and the screen let you), `unexplained` (copy or number with no source on
screen) and `friction` (more taps than the action needs); mark each `transcript` or `live`.
End with the count per group. Do not edit any other file. Do not git commit.


**Budget (hard):** at most 200 tool calls and 120 screenshot/image reads for the whole session. Look at each screenshot once; prefer the driver's text op over screenshots. When you reach the budget, stop and file what you have.
