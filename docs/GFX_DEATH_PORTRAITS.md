# Larger framed death portraits — 2026-10-06

Owner asks for bigger, framed death portraits. Reuse the packed iron portrait
well: enemy112px outer frame (87px art), fallen hero64px frame (50px art).
Add reusable unitPortrait around existing identity art/fallback. Death enemy
stays centered above the text; hero frame stays beside historical identity.
Other unit labels remain compact and unframed. Keep natural3:4 banner, shared
spacing, tooltip/trace input, and no invented environmental or stall portraits.

Acceptance at320/400/1440: exact112/64 frames, packed well decoration present,
art decoded, portrait above text/center±1px, banner ratio±.01, no overlaps or
horizontal overflow; input/historical identity regressions and actual WASM
screenshots. Build/copy, no deployment.

Removed stale320/390px desktop banner overrides: keep290px width and native
3:4 proportions so the larger portraits do not push the action below the fold.
Six targeted UI suites pass14.2s; real earned8h WASM goblin-archer death at
400/1440 has112/64px frames, native banner ratio, visible lever within the well,
no horizontal overflow or browser warnings. Screenshots shown inline;
scratchpad/death-portraits-20261006/. Build/typecheck/copy1556/diff clean.
