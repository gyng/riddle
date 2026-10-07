# Cut98 — isolate the boss-break watch check

The current full client run missed the fast Warlord break. Repeating all of
fights takes about3min, while the relevant fixture covers just two modes.
Extract that existing fixture into one shared function and allow
`node tests/run.mjs fights:break`. Default fights still executes every original
check. Preserve25s observation, 2s hold, actual fight frame, name and HP bars.
Do not use engine state shortcuts or invent a new fixture. Preserve diagnostics
for a missing beat. This reduces diagnostic iteration, not verification scope.

Gates: targeted check executes original fights/fast fixtures and identical
assertions; full default fights still runs all original checks; diagnose any
reproduced app failure before declaring full green.

Validation, 2026-10-07: targeted `fights:break` passes all six original
assertions in 9.5s (fixture 8.9s). Default `fights` passes all 48 checks
in 175.4s. Serialized fights/qa21/enemy-tips/captive-fact-copy passes 4/4
in 203.4s; log `/tmp/riddle-cut98-full-watch-recheck.log`.
The earlier broad client run finished 120/122 in 1278.5s: one missing fast
break assertion and one forecast locator timeout. Both pass unchanged on
retry; their initial failures remain recorded, with no proven root cause.
This is a narrower diagnostic command, not a whole-suite or simulation speed
improvement. No app behavior, Rust, numeric assertion or timeout changed.
