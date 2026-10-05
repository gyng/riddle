# Combat style and trait personality — 2026-10-06

Owner requests more personality in the eight creature icons. Retain their animal
themes and transparent unboxed silhouette, but strengthen facial acting,
asymmetry and signature gestures. Generate individual v3 artwork with built-in
imagegen, using each v2 asset as its reference. Preserve v2 files for comparison.
Use broad ink outlines and three-tone painted fills that read at34–44px.

Acceptance: all eight final assets have alpha and decode in Tactics; no added
icon frames, clipping or horizontal overflow at400/1440. Inspect actual UI
screenshots at both sizes and compare expressions at runtime size. Existing
selection checks, typecheck/build/copy/diff must pass. No gameplay or deployment.

The in-progress per-bloodline training work remains separate and uncommitted
until its own verification is complete.

Verified: eight square RGBA sources with transparent corners and96px packed
assets. Headed all-unlocked visual fixture400/1440: eight34–44px emblems decode,
transparent sockets, no horizontal overflow; screenshots inspected/shown.
Package selection28 checks2.8s and death actions44 checks2.2s pass. First test
invocation used nonexistent selection.mjs; corrected to package-selection.
Production build/typecheck, copy1511, diff pass; existing chunk size advisory.
No new unlock/balance claim; same creature themes, stronger expressions.

Built-in imagegen used. Final sources art/ui/icons/pkg_<id>_v3.png; runtime
web/public/ui/icons/pkg_<id>_v3.png. Exact prompts and reference paths retained
in art/prompts/style-personality-v3/{steady,guarded,bold,hunter,skittish,
unbowed,light_hands,iron_gut}.txt. Sources preserved as supplied; existing
ui-skin packer makes runtime96px PNGs. v2 originals/fallback retained.
Screens/logs scratchpad/style-personality-v3-20261006.
