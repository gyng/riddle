# QA brief (paste to a fresh agent; no scores, lapses only)

You are a QA player for a browser game called Riddle, an auto-played roguelike whose hero runs
on chosen combat styles/tactics and later optional written rules, with manual town
construction, hired automation, offline runs and deaths that name their causes. You are not a rater and you award nothing. Your job is to play the rater
protocol and file **every lapse**: anything you misread, anything that did nothing when tapped,
any number that does not reconcile with another number on screen, any copy you could not
explain from what the game showed you, anything that contradicts what the game said earlier.
Read `eval/RATING.md` (the protocol) and `docs/FUN_EVAL_IDLE.md` §"human probes" so you know
what a rater looks at. Do not read PLAN.md, research/, docs/CUT*.md or eval/cards/.

Setup: the game is served at `{URL}`. Play it through the browser driver, headless (you file
lapses in text and numbers, not feel): from /home/g/p/riddle,
`node tools/driver.mjs --dir scratchpad/{QA} --port {PORT} &` keeps one phone viewport
(400×800, 2×) open for the whole session; command it with `tools/drive.sh {PORT} '{"op":"text"}'`
(ops: goto, text, shot, click by visible label, tap, type, press, eval, js, wait, buttons, log,
quit, and the compound `state`, `act` (click + wait for the engine + the new text), `watch`
(`{"op":"watch","ms":60000,"every":1000,"shots":4}`: a watch in one call), `send_and_watch`
(`{"op":"send_and_watch","mode":"fast","skip":true}`: send, the mode, `▶▶|`, to the exit sheet
or the death / report screen, with the lines that appeared and the shots) and `sheets` (every
sheet this screen opens, read in a copy of the page — the session's game is untouched) — the
file's header lists them). Screenshots to look, `text` to read, click buttons by their
visible label; add `--headed` only if a lapse is about the render itself. Notes and screenshots
under `scratchpad/{QA}/`. Open `{URL}?dev=1&runs=1&seed={SEED}&fresh=1` once at the start.

Horizon: use the full current `eval/RATING.md` horizon: 40 minutes cold, then
20m/4h/8h simulated absences and 5/5/20 minutes on their returns. Start empty;
use only available controls and earned state. Watch two runs in fights and two
in fast; Speed opens mode/Skip controls. Read every death and because-link, and
inspect the death trace/replay when offered. Make four configuration changes
of your own, including available tactics, equipment or Legacy choices. Attempt
four hand edits of written rules; if the pen is locked, record the attempted
entry, what the UI says and the lock honestly. Do not manufacture an unlock or
claim that a tactic selection was a written-row edit. When available, inspect
drop/reorder and repeat/supply controls. Buy affordable upgrades and workers
and read their sheets; open available home/report windows. Read each return's
runs, depth, gold, class XP and earned Legacy against the total/owner/detail
views. Check what carries into the next actual send and whether your chosen
configuration persists. Reproduce anything odd once before filing it. Report
actual minutes/absences/deaths/attempted and completed edits separately; a short
or blocked session is incomplete QA, not a passed horizon.

Keep `runs=1` on every navigation, including absence/return URLs, so the
open-app clock behaves as it does for a player. Do not rate a paused simulation.

File `eval/qa/{BUILD}.{QA}.md`: one line per lapse — `screen · what you saw (quoted) · what you
expected · repro (seed, step)` — grouped under `defect` (wrong or inert), `misread` (you got it
wrong and the screen let you), `unexplained` (copy or number with no source on screen), and
`friction` (more taps than the action needs). End with the count per group. Do not edit any
other file. Do not git commit.


**Budget (hard):** at most 200 tool calls and 120 screenshot/image reads for the whole session. Look at each screenshot once; prefer the driver's text op over screenshots. When you reach the budget, stop and file what you have.
