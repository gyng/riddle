# Cut107 — interrupted gate jobs are not completed evidence

Native dayplayer SIGSEGV left a status:null cache, which the gate repeatedly
reuses rather than executing the incomplete leg. Preserve original crash
log/kernel evidence; this harness repair does not fix or waive that crash.

Only integer exit status plus string output is a completed leg. Interrupted
workers are not cached; older status:null records are cache misses. Preserve
all genuine completed legs and numeric/exit failures. Print termination signal
so a failed child no longer looks like an unexplained exit null. Gate assertions,
seed counts, runtime keys and simulation unchanged.

Gates: orchestration interruption, completed-leg reuse, corrupt/legacy-null
cache misses, SIGSEGV diagnostic/no completed cache, retry only incomplete leg,
no passing gate when child is interrupted even with PASS output. Fixtures are
harness tests, not game evidence. Native crash still requires investigation.

2026-10-07: recovery regression PASS (host0.75s), including legacy-null
replay and SIGSEGV with misleading PASS output rejected/no result cache;
genuine status1 numeric failure remains failed/cached. Old test fixture gained
its missing runtime-key stub for current harness; no real game evidence is
produced by this test. Default sandbox suppresses captured child stderr
(minimal stderr probe confirms); same test on host passes unchanged.

Actual full gate recovery PASS: /tmp/riddle-cut107-gate-recovery.log, metrics/
qa reused their genuinely completed jobs; dayplayer prints full18case bars
from completed same-binary/key debugger jobs. Original interrupted leg and
full failure log preserved in scratchpad/qa-cut107-native. Original native
SIGSEGV root cause remains unresolved. Current fullverify and fresh fullnative
run queued AFTER live wholeclient to repeat original concurrent gate work
without client timing interference. No raters released yet.
