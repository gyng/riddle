# Direct tester-dump choice diagnostics — 2026-10-06

Let choice-check consume the existing versioned tester export directly, as well
as raw engine saves. Validate dumps through readDebug before compiling/running.
Feed the exact embedded engine string to Rust without parsing/reserializing it
(JS integers cannot safely roundtrip Rust seeds). Preserve source SHA, exact
engine SHA, capture/build metadata and current native binary identity in the
manifest. This is a current-runtime diagnostic, not a recreation of the old
build. No gameplay changes, new export UI, or deployment.

Acceptance: actual multihero tester dump versus exact extracted engine yields
identical reports/final saves for baseline and an equipped control; selected
bloodline preserved. Input immutable, rejected schema/partial dump before
simulation/output creation. Existing raw save behavior preserved. Warm run
measured, no general performance claim.

Verified actual Settings export fixture400-dump.json: selected bloodline2,
two active heroes, in-flight session before scout. Baseline/Steady1h reports
and final engine saves exactly match extracted raw input; source SHA, exact
engine SHA and binary SHA confirmed. Warm dump0.09s(build0.06s), raw0.08s;
pre-scout single-run limit remains, not a general runtime bound. Unsupported
schema/format and missing engine reject before Cargo/output. JS syntax/diff
checks pass. Evidence scratchpad/dump-choice-check-20261006/. No app/core/WASM
changes, browser screenshot unnecessary for this CLI-only improvement.
