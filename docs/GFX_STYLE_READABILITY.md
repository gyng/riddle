# Style and trait expressions at game size — 2026-10-06

Follow-up to the owner's request for more personality. The v3 bear/lion/traits
already have gestures, but the hawk and turtle have restrained animal-portrait
poses. Replace those two with individual expressive v4 edits; preserve all v3
sources. Keep the existing warm painted ink family and transparent silhouettes.
Give all eight emblems more room in Tactics (52px choices, 44px equipped mobile)
so brows, mouths and signature props remain legible.

Acceptance: all eight selected emblems decode; alpha corners on new sources;
no icon frame, no icon/text intersections or horizontal overflow at320/400/1440.
Inspect headed Tactics styles and traits screenshots at400/1440. Existing package
selection checks, build/typecheck and copy lint pass. No gameplay changes.

Verified: both new sources RGBA1254px with transparent corners; packed96px.
Headed all-unlocked visual fixture320/400/1440: eight44–52px emblems decode,
no icon/text intersections, no horizontal overflow, no framing. Phone and
desktop styles/traits screenshots inspected and shown. Package selection28
and death actions44 checks pass on isolated no-HMR server3.2s. Initial shared
server selection run failed subscription cleanup; isolated run passed unchanged.
Death test's exact expected Guarded asset updated v3→v4, behavior retained.
Build/typecheck/copy1523/diff pass; existing bundle-size advisory.

Built-in imagegen edits retain exact prompts in
art/prompts/style-personality-v4/{guarded,hunter}.txt. Final sources
art/ui/icons/pkg_{guarded,hunter}_v4.png; runtime counterparts under
web/public/ui/icons/. Other six v3 artworks retained, all eight given more
space. Older sources and glyph fallbacks preserved. Evidence:
scratchpad/style-readability-20261006/. No gameplay or deployment.

Separately completed real-WASM fresh seed2 headed playtest: ten checkpoints,
manual house and first Send, first returned report, scripted hire of Scout
following two additional sends, eight-hour report and historical death.
34.1s total; offline2.1s, no console/page errors. Worker trunk uses engine
harness actions, not a complete UI hire walkthrough. Evidence:
scratchpad/first-session-20261006/. Not a balance or formal fun-eval claim.
