# Cut75 — readable windows under reduced motion

Cut74's screenshot suggests persistent parchment transparency. Inspect the
actual rendered layers before changing material: current centered game windows
have opaque background #e1d6bd, but legacy unfold opacity/scale animation still
runs under prefers-reduced-motion:reduce. Screenshot caught opacity0.219512.
Respect reduced motion across ordinary sheets and camp panels so content is
fully visible immediately. Keep existing frames, center geometry, nested window
navigation, normal-preference motion and alpha-based outer frame shadows.

Numeric acceptance: reduce preference gives no sheet/panel opening animation,
opacity1/identity transform on the initial rendered frame; actual earned
Details, Run log/card, Tactics, Legacy and settings at320/400/1440. Exact saved
state through read-only opening/closing. Normal preference retains bounded
opening animation and settles opacity1, no hidden-parent interference. Packed
skin and primitive fallback remain legible with no overflow. Screenshot after
opening/settling, not during an unrelated transition. Inspect scroll asset
center alpha and settled noncenter surfaces before declaring them defective.
No fabricated rating, deployment, gameplay/balance changes.

Validation2026-10-07: settled native earned historical card has opacity1 and
opaque rgb(225,214,189) before the change; earlier capture is transient unfold,
not persistent transparent material. Packed panel/scroll center alpha253/255,
no persistent center defect established, so no material/image edits. Initial
settle inspector incorrectly matches hidden parent and child (strict violation);
corrected to visible non-under window and preserved failure.

Fix is four CSS lines: under reduce, ordinary sheets and camp panels suppress
legacy unfold. Actual built GPU42cases =7windows/panels×3widths×2preferences:
Hero/Legacy, Run log, Run card, Tactics, Forge, Settings, Forecast panel.
Reduce immediate animationnone/opacity1/identity; normal bounded j-settle/
j-sheet2 ≤0.5s and settledopacity1. Normal initial inspector assumes animation
already starts and fails; actual existing two-frame arrived gate intentionally
paints first frame whole, then starts animation. Final check waits for that
existing gate, not weakened duration/opacity bounds. All six sessions exact
entire earned save, nooverflow/page errors. Full42results and corrected GPU
captures scratchpad/qa-cut75; screenshot shown. Dedicated built fallback follow-up removes panel/scroll skin classes: actual
earned card320/400/1440 has opaque rgb(207,197,171), immediate opacity1, no
animation/transform, exact full save/nooverflow/errors. Phone visual inspected.
This verifies the window fallback, not every primitive sprite/control asset.

Three scoped browser jobs PASS15.5s: chunky-controls, run-card-gold,
post-clear-town. Build/TS/copy1742zero/diff PASS, existing chunk warning.
Isolated qaj exits1 after87.6s at retired +1 vault text selector (actual +1
storage slot); no assertions failed before abort. Its earlier resource crash
is not reproduced alone; remaining checks not certified. Full historical
99/117 audit remains authoritative; no new full-suite green/95rating.
Logs /tmp/riddle-cut75-{inspect,settled*,windows*,motion-debug,client,build,qaj-solo}.
Owned5436 stopped; shared5219 retained. No gameplay/core/deployment change.

Fallback evidence scratchpad/qa-cut75/fallback-results.json and fallback-card-*;
/tmp/riddle-cut75-fallback.log. Initial fallback script creation used the wrong
working directory; corrected absolute scratchpad path, failure retained.
