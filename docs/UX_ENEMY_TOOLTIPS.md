# Detailed enemy tooltips — 2026-10-06

Owner requests hover details on Bloat Mother and similar enemies. Extend the
existing single tooltip plate with custom detail bodies; preserve hover/focus,
tap/longpress, Escape and viewport placement. Attach to report boss portrait/
name rows and seen enemy-guide names. Show existing art, known boss floor,
learned traits and counter, seen/studied/victory knowledge. Never invent combat
stats or leak selected hero knowledge into another bloodline's reward.

Verify actual Mother report hover phone/desktop, focus/tap/Escape/bounds and
missing/other-owner knowledge; existing boss and tooltip checks/build/copy.
No core/balance/newart/deployment.

Implemented shared detailHost callback in existing single plate (WeakMap,
no added per-row listeners). Applied report portrait/name, seen guide names,
forecast killers/bosses/walls. Enemy data is existing Lineage ledger/walls/
facts/counters. Report rows for another bloodline show Unavailable rather
than selected-hero traits/counters; explicit victory remains Defeated. No HP/
attack stats invented or hardcoded boss floor table. Counter comma becomes
plain “at boss”. Missing art/data remain valid tooltip states.

Verified actual shippedWASM earnedseed3 Mother report400/1440 hover; headed
screenshots shown/clean. Enemy presentation fixtures320/400/1440 hover/tap/
longpress/keyboard/Escape/viewport/owner isolation pass; reportbosses54 checks
pass; full existing glossary70/70 pass123.1s. Final targetedenemy+boss gates
7.7s. Build/typecheck/copy1554 pass, existing bundle advisory. Evidence
scratchpad/enemy-tooltips-20261006/. No core/balance/newart/WASM/deployment.
