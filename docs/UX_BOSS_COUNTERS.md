# Boss counter actions and explicit state — 2026-10-06

Tactics Details currently gives a boss-only toggle with no action, and divides
core scar percentages by5 into inscrutable counts. Keep the Details fold and
existing controls, replace each row with a compact shared chunky card: existing
boss portrait/fallback, boss name, core drill actions in readable verb labels,
On/Off toggle with explicit accessible enable/disable label, and exact core
boss max HP reduction. No guessed meeting counts or damage-reduction claim.
Scar-only bosses have no fabricated learned action or toggle. Missing revoke
bridge disables toggles. Guard pending click to prevent duplicate mutations.
Preserve revoked drill and scar independently; full row conditions in tooltip.

Acceptance: Warlord attack/Queen silence/Foundry reflect actions, item icons,
multiple rows, zero/non5-multiple scars, no-drill boss, revoked/old bridge/pending,
320/400/1440 touch/overflow/fold-state; exact revoke boss/bool roundtrip. Existing
Tactics selection/geometry, build/typecheck/copy/diff. Headed actual saved-camp
boss counter off/on and save/load, screenshots. No Rust/WASM/tuning/deploy.

Implemented: shared tablet frame, existing foe art/fallback, readable row verbs
with consumable silhouettes, core unlock card carries when available (Foundry
fire at the boss), explicit On/Off/aria labels. Missing card metadata falls back
to readable existing verb label, no action invented. Whole row remains tooltip.
Exact Max HP percentage, no division by5. Pending toggle disables immediately.

Verified:69 checks320/400/1440 pass3.7s; existing selection28 checks3.6s/chrome184
27.7s pass. Checks cover core card gloss, item kind, multiple rows, exact17%,
scar-only/zero, disabled bridge, pending duplicate prevention, fold retention,
On/Off roundtrip, bright text/uncrossed Off and settled44px touch geometry.
Build/typecheck/copy1510/diff pass; existing large-chunk advisory. Initial
build caught unused import, removed. Screenshot caught dark inherited text;
fixed explicit light card colors. During contrast edit accidentally overwrote
CSS with TS, causing build/layout failure; restored HEAD CSS plus scoped styles,
rechecked full geometry/build. Final touch test waits for panel opening animation.

Headed actual saved camp-1 at400/1440 displays four core counters; Warlord Off
persists exact save-load, other drills remain enabled, On works again, Details
stays expanded, no warnings/overflow. Corrected screenshots shown. Queen and
non5-percent coverage are fixtures; no new fresh boss fight/progression claim.
Artifacts scratchpad/boss-counters-20261006. No Rust/WASM/tuning/full audit/deploy.
