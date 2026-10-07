# Cut105 — report time scope

QA-b saw1h metered after20m away and rest totals inconsistent with the away
clock. Core adds the whole completed run meter at exit, including ticks before
an absence (engine.rs5669); offline rested_s counts assigned post-run rest,
possibly still pending (offline.rs138/316). These values are not away duration.

Name report meters Completed runs. On absence reports show Includes before away
and Camp rest separate; watched reports show Whole runs and Camp rest separate.
Name rest Rest assigned with Includes pending. Only details/desktop diagnostics
change; actual elapsed, meter rates/seconds, rest, report/save/clock unchanged.

Gates:320/400/1440 reports with20m elapsed/1h meter/30m assigned rest retain exact
numbers and readable scope; multi-run aggregate and single-exit fallback remain
correct; watched report wording; actual report/death reading and saved state
unchanged, no horizontal overflow. No simulation changes.

Final43890 terminal5/5 scoped report/death suitesPASS9.2s, log
/tmp/riddle-cut105-client.log. Build51375PASS includes types;copy1768zero and
diff pass. Scope fragments obey existing word budgets (callouts3/labels2),
no lint relaxation. Diagnostic screenshots320/400/1440 in
scratchpad/qa-cut105-time;400shown inline. Fixture demonstrates scope, not
a scored play. No core/WASM changes; broad acceptance remains pending.
