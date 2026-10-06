# Client gate audit — 2026-10-07

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
