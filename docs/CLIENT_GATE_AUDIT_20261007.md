# Client gate audit — 2026-10-07

Latest broad c8187a5: **91/112PASS811.4s**, terminal1, session75668.
Evidence /tmp/riddle-post-cut37-all-client.log. Previous80/111 retained below.
New hero-grounding case accounts for the extra total. Chrome184, fights48,
clarity:card14, qaAC18 and current watch checks pass in this full run.
Remaining21: autodismiss, clarity:core,watch (one job), cut12, cut13, cut20,
cut22, cut26, cut27, cut29, cut30, cut305, layout, legible, locked-cond,
patch-overflow, qa21, qa778, qa9, qaj, runsui, screens.

Post-suite scoped layout57/57PASS89.3s on current manual-house/earned-roster
coverage (CUT38). No production UI/core changes in that coverage correction.
Forecast qa21 waits for automatic refine despite explicit-only sampling;
next check must request More samples visibly and preserve shaft/panel equality.
A focused pass never replaces the broad failing verdict.95unverified.

Full client run before public Gunner exposure: 80/111 gates pass in581.6s.
Raw evidence /tmp/riddle-gunner-current-all-client.log, session65678 terminal1.
This is a failing broad suite. Focused/current native gates do not override it.
The new offer was still hidden when this run started; do not attribute these
failures to public Gunner exposure without a paired reproduction.

## Findings and next checks

- live-roster expects `Live D1/D7`, replaced by the owner's richer depth/activity
  presence. Update to exact backward-compatible `D1/D7 · Delving` (no activity
  metadata), retaining slot identity, XP/Legacy, request ownership and stale
  reply checks. Await the first actual async request instead of a fixed30ms.
- ambient-motes fails the reduced-motion transition after a fixed100ms. Reproduced
  while native gate fills the machine; retry in isolation and inspect media-query
  state before changing production logic or weakening the requirement.
- clarity:paint fails actual3s refine timing (3.17–4.11s). Final timing check
  overlapped native compilation. Rerun quietly; if still failing, profile the
  actual WASM candidate work. Approximate forecasts are authorized, not a slower
  budget or a false pass.
- layout/looks/legible and several old cuts assume a resident/free-standing Send
  before the owner's manual-house opening. Verify against current empty-town
  contract, preserving the manual construction assertion and actual send checks.
- clarity:card/cut28w expect old visible speed chips/mode controls; owner requested
  compact Speed menu. Re-exercise mode selection and picture/core timing through
  that menu rather than restoring removed clutter for old selectors.
- editor/patch gates target an always-visible early Edit tile. The current pen is
  late and optional. Use legitimately unlocked pen fixtures and the present menu;
  retain edit/patch/refusal/result requirements.
- chrome packages center, report/trace chips, forecast consistency, replay folds,
  restock confirmation and run-clear failures remain unclassified. Inspect actual
  screens and source before assigning them to old tests or game defects.

No failing cases removed. Any retired contract must be explicitly documented,
with current behavior and replacement coverage; numeric gameplay bars stay intact.

## First confirmed resolutions

Quiet targeted repeat: live-roster PASS (19 checks at400/1440); clarity:paint
PASS (5 checks,26.8s own run). The3s refine bar remains unchanged. Compilation
and CPU-saturating gates must not overlap this latency reading.

Reduced-motion diagnostic on fresh5379: narrow view media change delivered and
layer removed; wide view query already true but event had not arrived at100ms
(events empty,layer1). Await the actual query AND removal/recreation within1s,
retaining all exact counts/lifecycle/reduced-motion assertions. Production logic
unchanged. Do not label this a game bug without a failed bounded event check.
Raw diagnostic /tmp/riddle-motes-diagnostic.log and script under
scratchpad/gunner-public-qa-20261007/motes-diagnostic.mjs.

Final ambient-motes/live-roster targeted run2/2PASS10.3s;400/1440. Full baseline
remains80/111; these focused corrections are not a new full-suite verdict.

## Shared window fix and current opening flows

`chrome` exposed an actual shared CSS defect: the old framed-sheet selector
forced bottom padding0 after centered windows computed the HUD inset. At320px,
window center was64px low and bottom836 crossed console top788. Centered framed
windows now explicitly retain their computed bottom inset plus16px gutter.
Unchanged184 checks PASS at320/400/768/1440, including content expansion, HUD
resize and viewport resize. Actual earned WASM Tactics320/400/1440 has center
error<2px, no overlap/overflow/pageerrors; screenshots shown. Source/evidence
scratchpad/window-audit-20261007; final TS/copy1730zero/build PASS.

`looks` now manually constructs the free opening house and uses active Hero
Details → Appearance. All16 original appearance/headshot/save/reload/renderer
identity checks PASS7.7s. Hidden camp portrait stud is no longer the user flow.

`runclear` now manually builds a house before Send. Its first full repeat reached
all cases and found only the retired banked/collected wording expectation.
The check now asserts exact Rust reason in data-why AND its collected wording,
retaining20-word/rarity/exit/death/report/autodismiss checks.
Final all-parts GPU rerun PASS125.7s (/tmp/riddle-runclear-current-final.log,
session14943 terminal0). No simulation or numeric budget changed.
Full baseline remains80/111, not a new full-suite verdict.

## Compact watch audit — 2026-10-07

`fights` initially passes all46 checks after checking the actual speed/town tile
IDs plus Fights only annotation instead of expecting the retired bare label.
`clarity:card` now opens Speed to inspect mode/rate chips and invokes Skip through
its actual menu. Investigation found App.screen called any watch sheet `exit`,
including Speed; exclude this settings sheet from that inspection status.

Actual card timeout bug: a queued fight and cardWait exempted a floor card from
its maximum, allowing1292–1348ms spans versus the unchanged1200ms bar. Remove
those exemptions; the live map replaces the card while travel reaches the next
fight. A repeat reads630/1110ms, but mode checks on the old cached server still
fail. Preserve raw results, do not claim the whole module green from this.

The old Send-pill persistence flow is superseded by the owner's compact watch
contract (amendment in CUT28). Speed selection is sampled synchronously because
Pause freezes only the picture; the world continues. Assert mode/label, active
menu closure and unchanged lineage at the selection, then Town/flush/reload.
Ignore the inert close-animation copy. Existing render numeric gates remain.

First-load camp defers forecasts. Fold render fixtures request a genuine forecast
with existing GOOD rules and use the supported client-fold path; a no-fold run
cannot certify this overlay. This exposed4, then2, nameplate overlaps at the dock
animation's end (goblin on alert1/8). Reserve the banner's whole animated travel,
even at initial opacity0, and refresh immediately on docking. Read one banner
rectangle rather than every chip. Core fold correctness is still cut27, unresolved.

Real earned Gunner WASM watch400/1440: Speed Fast selection, Town exit and reload
persistence PASS; no page errors/overflow/HUD overlap. Selected mode was unreadable
against parchment; give it a dark inset background, plus4.5 contrast floor in
watch-console. Evidence scratchpad/watch-menu-qa-20261007, final screenshots shown.

IMPORTANT: no-HMR test server caches transforms across source edits. Final current
source checks run sequentially on fresh5382, /tmp/riddle-watch-final-gates.log.
Earlier failures remain /tmp/riddle-watch-menu-current.log,
/tmp/riddle-clarity-card-raw-repeat.log, /tmp/riddle-clarity-card-cap.log,
/tmp/riddle-cut28w-speed-{current,repeat,clientfold,good}.log and
/tmp/riddle-cut28w-dock-fixed.log. A parallel intermediate fights repeat aborted
waiting for fast map; preserve /tmp/riddle-fights-card-cap.log. Final acceptance
must be read from the fresh run, not inferred from any partial pass above.

Fresh5382 first final run /tmp/riddle-watch-final-gates.log terminal1:watch-console
PASS all5widths (including selected contrast); clarity card rate/timing pass but
cage fixture misses first batch; cut28w one remaining goblin/damage-ticker overlap;
fights selector reaches an inert close ghost and fails to switch back. Fixes:
exclude inert sheet ghosts when choosing controls; install cage injection at camp
before Send; refresh keep-out immediately when ticker text changes and reserve
its area during opacity transitions. Final repeat uses the runner's fresh server,
sequential1browser, /tmp/riddle-watch-final-repeat.log. Do not use the older cached
server results as verification of these final fixes.

### Card experiment rejected; retained checkpoint scope

`/tmp/riddle-watch-final-repeat.log` terminal1:cut28w6checks PASS74.0s
(2449frames,901callouts,37captions,2391plates,1640docked,zero collisions);
watch-console5widths PASS26.2s. Clarity still fails cage and card spans1238/1380ms.
Fights fails the synthetic pair's second label. Two-hostile fixture now clears
unrelated foes (the crowd-label culling contract is a different concern).

Removing goLive from card expiry did not settle the problem:
`/tmp/riddle-watch-preserve-replay.log` terminal1, clarity still has1355ms/cage
failures; fights retains one ending-jump assertion219→219. No exceptions/bar
changes accepted. REVERTED all cardExpired/cap runtime experiments to original
behavior. Card pacing/cage/ending probes remain unresolved; inspect actual
per-frame card/held/cage timeline and first engine batches before another fix.
No claim of a successful card timeout fix from a partial repeat.

Retained production work: selected mode contrast, correct Speed-menu inspection
status, full animated fold rectangle reserved immediately, immediate keep-out
refresh for changing ticker text. Invisible inactive UI still skips reservation;
appearing fold/ticker/banner reserves at initial opacity0. Final retained-source
checks: /tmp/riddle-watch-safe-checkpoint.log. Full suite is still failing;
focused passes do not change its baseline80/111 verdict.

Final retained-source focused GPU run2/2PASS102.1s:cut28w6checks75.5s,
watch-console5widths26.2s. /tmp/riddle-watch-safe-checkpoint.log terminal0.
TS/copy1730zero PASS. No new full-client verdict; no95score claim.

## Profiled watch readbacks and corner visibility

Cut36 changes atlas context storage to suit CPU reads: matched headed cold
profiles reduce sampled getImageData1109.63→150.93ms median (86.40%), complete
atlas/normal RGBA hashes unchanged. Separate from sim/build performance.
Immediate cage frame avoids a skipped40tickbeat across a614ms first-frame stall;
original card cap/skip pacing retained. Candidate clarity:card14checks, cut28w6
and watch-console5widths pass. Corrected hero-box grounding aligns overlap
avoidance with the drawn reflected stack fan. Identical corner boss coverage
84.375%→0%, new hero-grounding passes400/1440 and fails preserved baseline84%.

Subsequent fights original hero coverage passes; clock164→164 failure was an
inspection race: engine ending flag precedes asynchronous picture seek. Trace
skip.json compares actual picture and DOMclock; wait for both ending and clock
advance within the same20s, retaining original two assertions. Final watch
acceptance result must come from /tmp/riddle-watch-cut36-final.log.

## Next actual current-control issue

qaAC repeat-confirmation failure reproduced21.8s. Diagnostic old loadout badge
rect0×0, no overlay, click rejected as invisible. Do not force it or restore the
retired tile. Actual Run setup → repeat pack off calls setOrders → Rust
set_restock(false), refunding nonfree/nonfound supplies, with no confirmation
in its current UI. docs/CUT37_SUPPLY_CONFIRM.md specifies restoring the first-tap
refund safeguard on this visible path and updating coverage. Implementation and
real-save acceptance pending. All other15qaAC checks pass on this repeat.

Final Cut36 watch run terminal1: fights48checks PASS200.1s; clarity card12/14,
spans1218/1237ms exceed unchanged1200ms. Cage passes, original card timing still
unresolved despite an earlier14check pass. Preserve raw repeat and investigate
remaining GPU stalls/scheduling; do not label the module green or95fun achieved.

## Current supply control and quiet fallback follow-up

qaAC hidden-tile fixture replaced with visible Run setup Auto restock control;
actual refund safeguard restored with shared twoTap. First paid-shelf tap refuses,
second mutates once; free/found excluded, already-off no-op and empty direct.
Final18/18PASS12.1s. Real earned WASM400/1440 two paid26 refunds exactly52, free
leash retained; flush/reload exact. Separate earned D20 camp foundheal/freeleash
retained with exact52refund, no grants. Centered window fixes actual desktop HUD
overlap and simplifies old standing orders/repeat pack wording. CUT37 evidence.

Fallback canvas reads matched median169.24→16.55ms,90.22% lower, identical
atlas/normal pixels. Quiet current clarity card14/14PASS66.4s (bar1200 unchanged).
Earlier1208ms repeat overlapped real QA and stays contention evidence. Current
scope does not certify arbitrary load or a new all-client result; refresh broad
suite next. No95fun claim or deployment.
