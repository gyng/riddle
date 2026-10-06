# Boss approach framing — 2026-10-06

The real desktop Warlord approach can put its sprite above the canvas while
the hero remains visible. The existing fight target uses tile centres and
the ordinary zoom; a large boss and distant hero exceed that view together.
Account for full sprite bounds and the boss's entrance lift. Widen the fight
view when necessary, retain a stable body scale through the encounter, and keep
both actors within the canvas. Do not change core events or reveal hidden foes.
The entrance temporarily gets more space for its36-unit drop, then cuts to the
closer body framing. Floor loads and physical viewport changes reset the scale.

Acceptance: replay the recorded real rally snapshot at400/1440 using the
watch's canvas size and texel base. Inspect headed entrance samples and settled
frames; zero hero/boss clipping where both can fit. Check approach from four
sides, hidden/remembered/ally bosses, fixed focus, small views, death and floor
reset. Existing hero containment and telegraph checks still pass. Verify
typecheck/build/copy lint. This is renderer geometry QA, not a performance or
balance benchmark. No deployment.

Verified: the previous renderer fails on the recorded desktop snapshot with
boss y−128 in a645px canvas. Paired settled record now keeps the boss at
y93.3/height160 and the hero at y440/height80, scale10 versus16 before; phone
remains at scale8. Entrance uses a temporary wider view. Shared camera limits
reserve the top96px plus the existing hero margin; fixed focus retains its
own scale, and hidden/remembered/ally bosses do not affect the new framing.

tools/boss-framing-check.mjs passes46 cases/124 contained GPU samples across
400/1440, including actual drawn lift. Diagnostic rectangles now add `drawn`
bounds after squash/lift while retaining existing natural bounds. Existing
hero check passes30 cases/174 samples; fights46 and warning34 checks pass157s.
Typecheck/build, copy1528/zero violations and diff check pass; existing bundle
advisory. Temporary baseline module removed after its expected failure.

Headed real WASM app loaded the recorded pre-encounter save from the advanced
town, then watched the actual run: phone tick7917 and desktop7906, scale8,
both combatants contained with full HUD/log, no page errors or overflow.
Screenshots inspected and shown. These live screenshots use different ticks
and are not paired FPS evidence; saved-camp strength does not establish new
player balance. Evidence: scratchpad/boss-framing-20261006/. No Rust/WASM or
deployment. Next gameplay review: how first-boss defeat changes the player's
available choices; avoid another cosmetic pass without new play evidence.
