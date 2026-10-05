# Combat style and trait silhouette clarity — 2026-10-06

Existing eight creature emblems convey personality but square sockets constrain
their silhouette and reduce their face size. Mark expressive emblems in shared
packageIcon; remove their socket border/background/shadow and enlarge artwork
38→44px in48px sockets, equipped mobile28→34px in36px sockets. Preserve
chunky card frame/selected gold outline, hit targets, names and rules. Keep
ordinary tactics/upgrade socket defaults and absent expressive fallback intact.

Acceptance: eight emblems decode and retain square image ratio, unclipped
44px/34px geometry and transparent sockets at400/1440; equipped selection remains
recognizable by card outline, no horizontal overflow. Inspect headed Tactics
styles/traits, run existing selection/geometry and production build/copy/diff.
No new raster assets, game truth changes or deployment.

Verified: headed all-unlocked visual fixture400/1440 decodes all eight,
34–44px square images, transparent/unframed sockets and no horizontal overflow;
captures inspected and shown inline. Fixture exposes traits together, not fresh
hero unlocks. Existing selection28 checks3.6s and chrome184 checks27.3s pass
27.6s batch. Production build/typecheck, copy1504 and diff pass; existing
large-chunk advisory. No new imagegen/assets, Rust/WASM/full balance audit/deploy.
Artifacts scratchpad/style-silhouettes-20261006/{400,1440}-{styles,traits}.png
and icon-preview.mjs, preview/tests/build logs. Ordinary action/upgrade socket
CSS remains unchanged; creature emblems alone receive expressive class.
