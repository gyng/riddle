# Shared unit icon and label — 2026-10-06

Owner asks for a component pairing unit icon with text elsewhere, with verified
spacing. Extend unit-icon.ts with unitLabel: fixed identity art, wrapping text,
8px gap, vertical centering, min-width0 and max-width100%. Reuse in guide,
report victories, historical hero identity, rich enemy tips, forecasts/shaft
and live boss identity (retaining its existing portrait). Keep centered death
killer above the text. Unseen guide entries and environmental causes stay as
before. Historical ownership and reports stay authoritative.

Acceptance: gap8±1px and vertical-center±1px, no overlap/overflow with long names
at320/400/1440; reference/report/tooltip/live/forecast regressions; real WASM
screenshots phone/desktop; build/typecheck/copy. No deployment.

Verified: component checks48 geometry assertions at320/400/1440 (16 per width)
including long and unbroken names, historical hero class and missing-art glyph.
Nine targeted client gates pass23.1s; final unit-label and clarity:paint2/2 pass
29.4s (five paint checks). Headed actual WASM measured14 visible labels onphone
and18 ondesktop:8px gaps, centers within1px, bounds fit; screenshots shown.
Final build/typecheck, copy1556 and diff clean; existing bundle advisory remains.
Evidence: scratchpad/unit-label-20261006/. No Rust/art/deployment changes.

Live boss spacing also measured8px/center±1px at320/400/1440 with actual
watch HUD assembly; enemy input/owner regressions remain green.
