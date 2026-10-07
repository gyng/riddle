# Cut90 — refresh text keep-out when HUD content changes

Cut87 full-lane reservation passes once but repeated/full and serial Cut28w
still catch one goblin nameplate on alert1/8. HUD text can wrap or grow between
the100ms geometry samples. Refresh geometry when status or alert text changes,
retaining whole-lane reservation and the ordinary100ms cadence. No extra
per-frame layout reads when those strings stay unchanged.

Gate: unchanged Cut28w zero overlaps with >200frames/>20texts/>20plates/docked
folds; watch status/console pass. Full Cut30 retains41checks; death fixture
expects current source-only Steady label when no historical trace/rules were
provided (deathAction intentionally omits invented row syntax).

Full client116/120 in616.9s: quiet Chrome184PASS and hero appearancePASS;
Cut30 old source-only assertion corrected and41PASS. Cut28w repeated overlap
requires the event refresh above (four affected suites PASS117.0s). Chrome
HUD resize test synchronizes on actual ResizeObserver centering within2s
instead of a100ms sleep; preserve all184 checks,2px/bounds/overflow/keyboard
assertions. It must fail if geometry does not settle, rather than reading the
old panel position during callback scheduling. Updated Chrome184PASS27.9s.
