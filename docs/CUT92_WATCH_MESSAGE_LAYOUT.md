# Cut92 — message layout and already drawn nameplates

Full Cut28w and repeated isolated runs catch one goblin plate over “alert1/8”.
Geometry shows ticker y490.8125/h25.1875, goblin y489/h19; top HUD y0/h67.1875.
This is the lower message stack, not the top HUD. The stack reflows as combat
history and wrapped causes change, after its previous reservation was sampled.

Refresh geometry once after a released event batch finishes its DOM changes.
A new keep-out reservation also removes overlapping already-drawn DOM nameplates
immediately. Next renderer frame computes their positions again. No engine or
pacing changes; no additional renderer frame or per-frame layout read.

Gates: unchanged Cut28w zero overlaps, watch status/log/console and tactic
observer behavior; new deterministic actual-DOM test fails before the fix
(two visible plates stay two and the reserved label remains) and must pass
without waiting for another renderer frame. Debug labels reflect actual removed
DOM plates. Keep-out comparison uses existing CSS positions and fixed tag height.

Validation: six scoped suites PASS72.6s; types, copy and build PASS.
