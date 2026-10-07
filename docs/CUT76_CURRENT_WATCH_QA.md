# Cut76 — current watch and historical controls QA

Mechanical pre-cohort checks still reference old storage/loot wording, stalled
HUD text, retired lanes and the tablet-first opening. Repair fixtures to cover
current public controls and preserve accounting/ownership/replay/clock bounds.
Never alter the game or weaken timing/balance limits just to satisfy stale QA.

First isolate qaj and cut13 to distinguish actual failures from the earlier
parallel browser resource errors. Retain failed logs and each unsupported
fixture. Use storage slot / loot choice / Path blocked words where currently
rendered, manual first house for real new towns, and Heroes/Details/Run log
instead of retired lane controls. Current normal and reduced motion remain
separately verified; no false full-suite pass from scoped tests.

Numeric acceptance: qaj original disabled-purchase equality/shortfall, delayed
beat ownership, live HUD through loot overlay, refusal/choice and drop-row
ranking checks; cut13 actual stall→verdict/report fallback and original report/
forecast/notes/trait/refusal checks; real runsUI manual opening, leaving watch
preserves same run/clock, exact replay event hash, manual one-run limit, scout
rest/auto-send, absence folds and ≤12/≤5 initial interaction budgets. If a
controlled fixture cannot represent a current path, replace it with a stated
current fixture that still tests the original behavior and record scope.
No manufactured rating, gameplay grants in earned QA, deployment or95 claim.

## Presentation migrations and current scope

Qaj still uses literal fake authoring fixtures, not earned current-player saves.
Keep its 32 checks: updated storage/loot labels and visible Speed menu controls.
Purchase buttons retain the original opacity <=0.5 and equality bounds; shared
chunky CSS had overridden the old .4 value. Restore .4 on disabled purchases.
The harness treats modeless loot as live HUD, while protecting caller modals.

Cut13 retains stall payout/verdict, notes, traits, row ownership/refusals, trace
provenance, report reconciliation, and its original beat/Skip/timing bounds.
The current forecast does not automatically refine: open Forecast, explicitly
request More samples, require a larger sample count, preserve numeric bands,
and clear rough quality only on completion. Its progress belongs to the button
in the heading, rather than an obsolete global blocking strip. A GPU geometry
probe shows button right326.000007629 vs heading right326: use1e-4 CSS px only
for floating-point rectangle containment, not a visible-pixel layout tolerance.
Empty salvage now lives under Forge Details; report accounting checks scope
Details tiles to avoid counting the new three-tile summary twice. Full haul is
the current label. Wait for a numeric playhead before timing visible menu Skip.

One product issue found: Speed stayed enabled after an ended run. Disable its
entry alongside existing run controls; assert visible Speed/Town disabled and
mode/Skip refused during verdict work. Controlled GPU400/1440 captures have
no overflow/errors. These use fake death injection, not earned progression.

Run-history/manual opening/Hero roster migration remains outstanding. This cut
is not a full client-suite certification or an independent fun95 evaluation.

## Intermittent loot fixture / iteration speed

The final sequential pair has Cut13 PASS77checks77.7s, followed by Qaj FAIL
1assertion (fast synthetic cage tap),0errors103.1s. An earlier isolated Qaj
PASS32checks100.6s does not supersede that failure. The pre-split invocation
qaj:loot started before the split code was saved, so it ran the entire old
walk and failed the same cage check; retain its112.2s log rather than call it a
10-second targeted run. Current qaj:loot alone PASS10checks10.7s.

Add eight independently selectable sections (exit/clips/cards/rows/purchases/
loot/rate/drop). Default still runs all32 checks, no shortened timing bounds.
Unknown sections are rejected. Capture watch/ticker/recent-step state on loot
failure. Replace the clip setup's retired inline Skip with the actual menu.
The synthetic cage previously injected a50-tick choice at the end of any probe,
unlike actual nearby cages'8-tick batches. Before its injection, cap fixture
steps to8 as actual CAGE_BATCH does; the full rerun must verify this change.
This is a source-grounded fixture diagnosis, not a proven real-game cage defect
or a change to the viewer's8s presentation cap.

RunUI manual section now constructs the real first house with no save grants.
It checks an empty start/no send, first Ready resident with XP/Details, manual
sending, no early Run log, and original <=5 main functions/<=12 controls.
Its first run fails6 main functions because Send gem and dungeon mouth are
counted separately. Both perform the same action; count those as one in both
manual and live states, following the existing per-function budget definition.
Corrected manual section PASS6.0s. The live/replay/rest/log/density/hero sections
still reference retired lanes and remain unverified; do not claim runsUI green.

## Verified checkpoint

Final current Qaj all eight sections PASS34checks89.2s (clip-link count varies
with the captured run); actual manual opening PASS6.0s, sequential2/2 in95.5s.
Earlier final-pair Cut13 PASS77checks77.4s, including disabled ended Speed/Town,
mode/Skip refusal, explicit forecast feedback, and unchanged numeric timing.
Real-WASM live section PASS18.4s: leaving preserves run ID and advancing town
clock, visible Hero jump resumes it, one manual send yields one town record,
Hero Details → Run log reaches it. Real-WASM replay section PASS19.1s: current
visible Speed/exit controls, whole run completion before history inspection,
exact FNV watched-event/replay equality, log replay identity and all floors.
Opening uses actual manual first-house construction, no grants. The original
replay test's forced camp fallback is replaced by an explicit completed-run
check, so an unfinished run cannot masquerade as replay coverage.

Build/TypeScript, syntax, copy1742zero and diff checks PASS; existing web chunk
warning remains. Controlled ended-watch GPU screenshots400/1440 and sampling
geometry screenshot400 shown; artifacts scratchpad/qa-cut76. Logs/tmp/riddle-
cut76-*. Rust/core/WASM unchanged. No deployment or independent fun score.
Remaining RunUI rests/log/density/heroes sections require migration/verification;
full historical99/117 audit is not superseded by these scoped successes.


## Active heroes and complete RunUI checkpoint — 2026-10-07

Paid founding exposed an actual Heroes window defect: replacing its body removed
“Active heroes”, and later roster changes did not repaint the open window.
Keep the heading and refresh the connected window through the roster's existing
onChange/onLive paint. Re-enable an attached founding button after a refused
mutation using current affordability/cap; no gameplay or Rust changes.

Replace retired fake-four-lanes coverage with an actual untrimmed earned town,
public paid founding from one to three bloodlines at320/400/1440, exact price
checks, fourth-slot refusal with exact save preservation, hero jump and Details
selection, unchanged per-slot Legacy, desktop left placement, no overflow and
original <=12 mobile above-fold functions. Current Rust cap is3; unsupported
fourth-lane folding is explicitly retired, not claimed as tested. Controlled
live/rest/ready display changes test open-window refresh only and leave the
Rust save identical. Actual automated new heroes show Ready with wire state
rests at zero remaining time; assert actual identity/state and Ready action,
not manual-only waits. Scope windows to real visible sheet-wrap elements:
exit animations clone their contents into inert sheet-ghosts.

Scout/rest/history/density tests earn automation through actual public sends
and paid hires, not save grants. House opens Details then Run log. Counts and
replay links are inspected after a real2h absence. Browser errors now fail the
suite. All seven RunUI sections together PASS64.2s on isolated no-HMR server
(headed GPU); prior full run failed only the three manual-state assumptions.
Build, TypeScript, copy1742zero, syntax, diff and watch-control checks PASS.
Responsive GPU screenshots shown from scratchpad/qa-cut76-heroes. Full client
audit is running separately; these checks do not certify the historical full
suite or an independent fun score. No deployment;95 remains unverified.


## Full audit and resource reproduction — 2026-10-07

Full headless audit on1fc8c82 terminates FAIL72/120 in523.8s. This supersedes the
older99/117 as current full-suite evidence; it is not a game regression count.
Many failures contain ERR_INSUFFICIENT_RESOURCES, Target crashed, shader or
WebGL context errors. Low-concurrency2 control screens/cut13/hero-presence is
FAIL1/3 in53.4s: presence passes, screens loses its page at navigation, Cut13
still reports48 resource errors. Reducing pool width alone does not solve it;
no browser errors are filtered and no test is certified from that control.

Screens alone headed/GPU completes48.2s with1 assertion and0 browser errors:
“dungeon” appears inert under the280ms hash sample. Town.send deliberately
awaits a650ms entrance walk. Require actual watch navigation within5s after
that particular click before sampling. This preserves the inert-control gate
and checks the real navigation instead of exempting the control. The corrected
walk must pass before being claimed verified. Investigate other isolated
specific failures separately; full headless suite remains failing.

The client runner now prints completed jobs/diagnostics immediately, retaining
all checks, scheduling, counts and exit codes. Temporary controlled child jobs
prove early failing-job output before a1.2s job, one result per job, diagnostic
preservation and2/3 exit1. This improves feedback latency, not suite runtime.


Final corrected Screens GPU walk PASS57checks,9 screens,30 button clicks,
0 errors in50.0s (48.7s walk). Dungeon must actually enter watch within5s;
its650ms walk remains unchanged. The earlier repeat still loaded the old
fixture (failed1 inert dungeon assertion in102.6s); it is not evidence for the
corrected file. Runner syntax/diff and failure-output unit checks3/3 pass.
Full headless72/120 remains the authoritative full-suite failure; isolated
Screens success does not certify it or the independent rating horizon.
