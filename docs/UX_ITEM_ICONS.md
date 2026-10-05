# Item icon coverage — 2026-10-06

Owner requires icons for all items. Reuse the existing item icon/rarity module
and packed art. Every item row, chip and selectable offer has an icon, including
unidentified finds, Forge salvage/future steps, shops, repeat suggestions and
report found/kept/stolen/shelved/salvaged/spent/identified items. Preserve labels,
counts, prices, identity knowledge and Rust-owned rarity. Unidentified items
use their visible potion/scroll family; never reveal their underlying identity.
All unrecognized kinds retain a primitive fallback. Pack uses existing art.

Audit other item-bearing UI (inventory, exit choices, rules and live log) and
record uncovered surfaces honestly. Scope compact icons so400/1440 layouts do
not overflow. Numeric gates: icon per item in each audited semantic list;
all catalogue kinds produce an image or fallback; no broken referenced images;
unknown labels retain their question mark; scoped copy/tsc/build and client
geometry/Forge/report checks pass. Capture desktop/mobile checkpoints. No deploy.

Owner follow-up: remove item frames/background wells for clear silhouettes.
All rarity tiers use transparent unframed icons with a small contour shadow;
names keep rarity colours. Remove rectangular glint/outline effects. Preserve
one-time find arrival animations and identity/rarity data. Send must remain
square/circular at every breakpoint, with no background-image distortion.


Acceptance:204 catalogue/image/fallback/semantic checks plus10 unframed rarity
checks pass across400/1440. The original item art already has transparency;
no raster edits were needed. Added icons to Forge salvage/future/current kit,
supply offers/repeat suggestions/sell confirmations, item actions/conditions
(including compact and read-only rules), report item sections and learned
identities, plus14px unframed item marks in the live text combat log. Existing
stored gear, packed supplies, cage/exit choices and run-clear finds already use
the shared module. Narrative paragraphs and aggregate resource totals remain
text; this is semantic item display coverage, not automatic icon insertion
into arbitrary prose. New/unpacked kinds always retain a primitive fallback.

Real shipping400/1440 Forge/shop checkpoints pass: an icon in every salvage
and shop offer, no broken displayed item images and no horizontal overflow.
Mobile salvage names/counts use two lines; shop item names/prices wrap within
fixed84px cells. Rarity colours/data remain Rust-owned. Controls/screens are
in scratchpad/item-icons-20261006/.

Send's old generic .send flex:1 expanded the square gem to96×86/124×108.
The gem now has flex:none and aspect-ratio:1, with artwork fitted using contain.
Actual measurements86×86/108×108 pass at400/1440. No artwork was reshaped.

Tsc, copy lint (1479 literals), production build and six scoped client gates
pass: item icons214, Forge33, report20, geometry176, Forge lifecycle68,
forecast paint5. Real Rust rarity gate also passes. Historical run-clear's rim
expectation was updated for the owner's explicit silhouette request; the
whole run-clear walk was not rerun. Historical Cut25 failures from the Forge
cache turn remain disclosed in PERF_FORGE_CACHE.md, not claimed resolved.
No Rust or balance change; no deployment. Next priority: simplify raw report
unlock IDs and wall-edit text, then reduce selected Tactics query work.
