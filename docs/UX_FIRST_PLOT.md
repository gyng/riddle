# First house: visible construction — 2026-10-06

The actual empty-town walk shows a dark scene and a flat floating Build house
label, with the tiny staked sprite barely visible. Keep the sparse first load,
manual house construction and hero arrival. Make the first plot visibly staked
with a chalk foundation overlay and give the action the same chunky material
as other controls, with a hammer icon and readable Free cost. The overlay is
presentation only; use Rust's home/staked state and the renderer's plot bounds.

Acceptance at400/1440: no hero or built house before click; plot visible and
one clear Build house action; primary target at least44px; no horizontal
overflow/clipping; keyboard activates one construction only; house/hero appear
only after real Rust construction and overlay disappears. Existing intro/send
flow remains exact; materials have packed-art and CSS fallbacks. Tsc/copy/build,
scoped town/geometry checks and real-engine desktop/mobile checkpoint screenshots.
No Rust/balance changes or deployment.

A generated desktop/mobile target is in docs/targets/empty-town-target-v1.png,
with the exact built-in imagegen prompt in empty-town.prompt.txt. It is a design
reference, not shipped renderer art. Scenic embellishments in the concept are
not implementation requirements; retain the current town scene/assets.

Implemented: code-native chalk/stakes overlay at the renderer's empty-house
plot, shared chunky construction control with hammer fallback and Free cost,
viewport clamping and one in-flight construction action across keyboard/map
clicks. The overlay disappears on Rust home state; no simulation edits.
Management windows now observe HUD/footer size changes as well as content.
Geometry tests await the actual opening animation: fixed sleeps could inspect
an unfinished animation in headed windows competing for focus.

Evidence: real release-WASM400/1440 opening controls pass, with no home/hero
before construction, minimum44px targets, packed frame, no clipping/overflow,
Space/Space/click invoking only one pending Rust action, then one hero and Send
ready. Existing town day0/desktop checks pass; headed window geometry184 checks pass,
including HUD resize/restore at320/400/768/1440. Tsc/build, copy1492 and diff
checks pass. After-change headed first-session/scout/8h walks complete at both
sizes in32.8/29.7s; eight-hour return3.0/3.5s. No console/page errors. Two
pre-existing WebGL warnings remain (getQueryParameter enum and texture-copy
bounds); these are not fixed or claimed clean. Seed1 has no worst death, so
this walk does not cover a boss encounter or death screen.

Local evidence: scratchpad/first-plot-20261006/{400,1440}-{empty,home}.png;
scratchpad/player-walk-20261006/after-{mobile,desktop}/ and after-walk.log.
The generated target remains preview-only, not game art. No deployment.
Next: live boss/death readability and renderer warning investigation, then
content/gameplay decisions exposed by those walks. Full balance audit is not
required for these presentation-only changes and was not run.
