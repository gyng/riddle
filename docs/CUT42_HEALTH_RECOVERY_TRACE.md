# Cut42 — record floor recovery in maximum-health traces

Real earned8h report: decision log says maxhp58→31/drain−27 while all last
10actions have52/52. turn::descend already restores up to5maxHP (capped at
base), but never records that in the max_hp event/trace. Folded losses therefore
invent starting maxima and omit real gains. Gameplay recovery already exists.

- Keep exact existing floor recovery arithmetic and timing. Emit existing
  hero_max_hp with exact positive delta/cause recovery only when changed.
- Meaningful core test: actual descent from drained max,5gain, capped smaller
  gain, no recovery event at full max; matching snapshot/event/runtrace values.
- Preserve original cut28d trace/per-turn/old-wire tests and current report/log
  tests. For earned report, displayed summary agrees with final per-turn max.
- Exact paired8h earned WASM report and full save after excluding only known
  intentional max_steps audit fields and explicitly named renderable-event counters. No gameplay/balance/timer changes.
- Rebuild realWASM; quick tests/TS/copy/clippy/build and scoped UI/recovery QA.
  Routine broad balance remains separately scoped; don't weaken any gate.

Acceptance:

- Actual-descent focused test PASS:31→36 (+5),38→40 (+2),40→40 (no event),
  snapshot/event/full-runtrace/exittrace agree. Initial compile-only probe used
  HeroSnap.max_hp instead of entity.max_hp, retained; corrected test terminal0.
-676RustPASS/oneignored36.71s. Initial tools/verify --quick terminal1 because
  native-host socket checks lacked network access;6other tool files andTS/copy
  pass. Native-host local-access rerun2PASS. Final all7tool files run together
  with local-access:14/14PASS3.08s, terminal0; /tmp/riddle-cut42-tools-final.log. Do not call the
  initial quick command green. Fast all-targetclippy terminal0, copy1734zero,
  fastrealWASM5482294bytes, webbuild/diffPASS.
- Originalcut28d17 +cut2016 +checkpointgold9per400/1440checks3/3PASS23.2s,
  /tmp/riddle-cut42-ui.log terminal0. No threshold or old-wire check relaxed.
- Matched real8h earnedGunner4run WASMreport/save: identical after intentional
  max_steps fields and three auditedrenderable-event counters. First comparison
  found counterdeltas24batch/11deatharchive/7floorstart. engine::tick counts
  renderable events, engine::end_run adds them tobatch; all references inspected,
  these are accounting only. Explicit normalization allowlist asserts those
  exact deltas; it does not omit arbitrary gameplay fields or other counters.
  This is an evidence-based acceptance amendment to the initial max_steps-only
  exclusion. Failedcomparison retained, no claim of rawsave byte-equality.
  Phone/desktop candidate rawreport/rawsave fullyequal to eachother. Evidence
  scratchpad/health-recovery-cut42-20261007/comparison.json andcompare.py.
- Final earnedlog arc52→52 agrees withlastturn52/52 instead of58→31/drain−27;
  existing maxLine appropriately absent afterfullrecovery, no TS suppression
  added. All10turnIDs/exitheader exact, inspection/close savebyte-exact,
  zerooverflow/pageerrors bothwidths. Candidate screenshots shown.

Core gameplay recovery arithmetic/timing unchanged. Prior fresh routine18-case
balance pass predates this trace fix; no new balance/exhaustive/full-client
certification. Latest broadclient91/112 still failing;95fun unverified.
No deployment.
