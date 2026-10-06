# Meter QA follows current controls — 2026-10-06

Old cut29:meters expects a removed watch command-bar meter tile. Current
controls place it inside Speed → Run controls. Update that test path and keep
meter values/units, remembered toggle, death/report and phone comparison
assertions. Add desktop camp native disclosure keyboard checks. Fake fixtures
are presentation checks; realWASM camp flows verified independently.

Acceptance: targeted meters/merge execute all assertions and pass. No app,
core, art, WASM, balance or deployment changes; no numeric gate weakening.

Second stale setup: comparison fixture contained two completed meters with
best_depth0; current first-load camp hides its main panel until progress. Set
fixturebest_depth4, without changing meter values or expected outcomes.
Final isolated cut29:meters,merge executes12 checks; townday0/desk also pass,
2/2 gates4.4s. Numeric merge/death/report/phone values retained; five desktop
fold/duration/keyboard/originalcomparison assertions added. Watch fixture
checks units/toggle on synthetic zero meter, not real game throughput. Existing
actualWASM first/secondreturn checks independently covered camp. JS syntax and
diff pass. No appbuild required for test-only change. Evidence scratchpad/
meter-qa-path-20261006/; no new player UI/screenshot/deploy.
