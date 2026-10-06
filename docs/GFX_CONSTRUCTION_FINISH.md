# Construction finish — 2026-10-06

House completion lacks the later buildings' glint. Later buildings schedule
scaffold/dust even with reduced motion. Use the same short completion glint for
the house; suppress construction timers (and therefore scaffold/dust/glint) for
all buildings when reduced motion is requested. Preserve manual construction,
first-house arrival timing, art fallbacks, saved towns, and game truth.

Verify actual WASM manual house and earned forge construction in normal and
reduced motion on phone/desktop; inspect headed completion screenshots and town
render sprite ids. Normal house scaffold then house/glint; reduced house/forge
immediate without construction glint/scaffold. Existing plot/save checks plus
build/typecheck/copy lint. No new art, game balance or deployment.

Verified headed real WASM seed3 at400/1440, normal/reduced: explicit manual
house, legal send/positive return and manual forge. Captured actual sprite
requests via QA-only TownAtlas instrumentation: normal scaffold→building/glint;
reduced immediate building without scaffold/glint. Four cases clean. Screenshot
house completion shown inline; evidence scratchpad/construction-finish-20261006/.
Firstplot320/400/1440, save39 and readyplot24/fallback checks pass14.3s.
Build/typecheck/copy1540 pass, existing bundle advisory. Ready plots no longer
also draw the obsolete tiny renderer plot under their painted foundation.
No new runtime instrumentation or bitmap; no core/WASM/balance/deployment.
