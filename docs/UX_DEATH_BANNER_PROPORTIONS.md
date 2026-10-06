# Death banner proportions and centered enemy — 2026-10-06

Owner correction: the cloth is squashed and the enemy should sit above the
text, centered horizontally. Preserve the packed banner's384×512 (3:4) ratio
and natural background scaling, with a vertical portrait/name stack inside
the existing trace control. Keep portraits in the unit guide and fallen hero
identity. No changes to death data or input semantics.

Acceptance: portrait center within1px of banner center, portrait above cause,
normal banner ratio within0.01 of0.75 at320/400/1440, no horizontal overflow;
existing tooltip/trace and identity checks, actual WASM screenshots, build/copy.

Verified: four client gates pass14.7s. Unit icon geometry asserts native ratio,
center and portrait-above-text at320/400/1440. Death tooltip/trace, historical
identity and action checks remain green. Headed actual WASM Warlord death400/
1440 preserves ratio with no warnings/overflow; screenshots shown. Build/
typecheck, copy1555 and diff checks pass (existing bundle advisory).
Evidence: scratchpad/death-banner-20261006/. No deployment.
