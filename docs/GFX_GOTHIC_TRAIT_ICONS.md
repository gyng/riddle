# Moonlit combat styles and traits — 2026-10-06

Owner correction: replace the sports-mascot impression with the established
Vampire Hunter D: Bloodlust × Downwell direction from ART_DIRECTION.md.
Use large gestures and worn relic silhouettes, moonlit ink-and-wash
paint over visible pixel clusters, limited palette, restrained blood accents.
Keep transparent silhouettes without enclosing circles, tiles or badges.

Steady: upright sword. Guarded: broad pointed shield.
Bold: diagonal striking blade. Hunter: crossbow.
Skittish: a retreating boot. Unbowed: a fist breaking a chain.
Light Hands: a hand taking a large coin. Iron Gut: an iron-bound potion bottle.
These pictures describe existing choices; mechanics and unlocks stay unchanged.

Acceptance: eight distinct RGBA sources with transparent corners; packed96px
runtime assets; all eight selected at44–52px, unframed, decoded and clear of
labels at320/400/1440. Inspect headed phone/desktop captures. Package selection,
death actions, typecheck/build and copy lint must pass. No deployment.

Built-in imagegen, one call per final asset.
Sources: art/ui/icons/pkg_<id>_v6.png. Runtime: web/public/ui/icons/.
Exact final prompts: art/prompts/readable-traits-v6/.

Owner rejected the first human portrait traits after seeing the actual tactics
window: expressions were lost at44–52px. Layout checks cannot certify
readability. Replace them with one large object/gesture, three broad value
masses, thick outlines and sparse texture; no fine portrait or armour detail.

Verified final v6: eight RGBA sources, corner alpha0; seven1254px square,
Steady1243×1266; packed96px via the existing skin pipeline. Eight new manifest
entries; packageIcon prefers v6 and preserves old fallbacks. Rejected v5
portraits are only scratchpad artifacts, not shipped.

Headed fake all-unlocked tactics window at320/400/1440: all eight v6 icons
decoded at44–52px, unframed, no icon/label intersections, horizontal overflow
or page errors. Phone traits and desktop styles inspected and shown inline.
This checks layout and permits visual review; it does not certify player
recognition without names. Final selection28 + death-actions44 pass3.5s;
typecheck/build and copy1532/zero violations pass. Existing bundle advisory.
No gameplay/Rust/WASM/deployment changes.
Evidence: scratchpad/gothic-traits-20261006/, final *-v6.log and screenshots.
