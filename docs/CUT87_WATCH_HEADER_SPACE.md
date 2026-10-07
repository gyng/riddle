# Cut 87 — keep creature names clear of the live HUD

Full and serial headless Cut28w both captured a goblin nameplate overlapping
alert 1/8. Keep-out geometry sampled individual HUD children every100ms,
so newly appearing text could occupy a gap reserved by the previous sample.

Reserve the whole existing top HUD lane. Its empty portions belong to the HUD,
and later children can appear there safely. Retain100ms layout sampling, all
combat/callout/caption placement and the map camera. This uses fewer header
rectangle reads rather than adding a per-frame measurement.

Gate: unchanged Cut28w observes >200frames, >20texts, >20plates and docked folds,
with zero overlaps; existing watch status/console and combat identity checks.
TypeScript/copy/build pass. No core or performance-score claim.
