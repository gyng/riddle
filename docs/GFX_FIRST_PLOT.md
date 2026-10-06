# House plot and small personality silhouettes — 2026-10-06

Owner: trait icons remain unreadable; improve build-house marker/plot toward
approved target UI and polish. Eight combat style/personality icons use broad pixel shapes,
bone highlights/dark contours, few accents, no enclosing icon boxes. Keep names
and mechanics. Judge at actual44–52px, not the large source illustration.

First house: empty marked stone foundation with timber stakes and pale string;
no house or hero before explicit manual build. Reuse the chunky carved button
module, connect marker to plot, preserve44px hit target/keyboard and in-flight
construction dedup. Completed house gets a short reveal beat and hero arrival;
reduced-motion removes animation. Existing towns do not replay construction.

Acceptance: actual WASM empty/home at320/400/1440, visible plot/button in bounds,
no overlap, overflow or errors; build only on action, exactly once while busy;
hero arrival/send continue. Missing art keeps primitive foundation fallback.
Traits decoded/unframed at actual UI size. Existing first-plot, selection and
death-action checks, typecheck/build/copy lint. Headed screenshots shown.
No deployment or gameplay/balance changes.

Source assets (built-in imagegen, one call per asset):
art/ui/icons/pkg_<id>_v7.png (eight styles/traits),
art/ui/deco/house_plot_v2.png. Exact prompts: art/prompts/readable-traits-v7/
and art/prompts/house_plot_v2.txt. Runtime icons96px; plot384×247WebP27,056bytes
(164×108CSS overlay), SVG foundation fallback. Source alpha preserved by
existing skin packing, transparent corners checked.

House checks: real headed WASM320/400/1440 manual build and busy dedup;
night, blocked-art fallback, reduced-motion, compact320×640 all pass.
Saved-home reload follows explicit flush (the existing persistence is debounced;
this does not certify a reload before that save). Missing-art request deliberately
aborted for fallback QA. Initial edge harness ran during packing and hit an
unmounted DOM; corrected boot/DOM wait. Reduced-motion capture initially checked
Send before it existed; corrected actual element wait. No production workaround.
House construction uses existing scaffold/dust, 0.5s reveal then1.22s arrival,
Send available at1.8s; no repeated reveal on mounted existing town. Empty towns
no longer emit smoke/embers from their unbuilt campfire. Whole164×108plot and
plaque share one build guard; keyboard and repeated clicks never duplicate.
Screenshots: scratchpad/house-traits-20261006/. No gameplay tuning or deployment.

Final eight v7 icons decoded at44–52px, unframed, no label overlap/overflow
or page errors in headed320/400/1440 captures. Shapes: upright sword, shield,
diagonal strike, crossbow; retreating boot, raised fist, hand/coin, potion.
Seven sources1254²; Steady1246×1263; all RGBA/corneralpha0. Broad silhouettes
visually reviewed in game; no name-free recognition study claimed.
Final first-plot320/400/1440 +selection28 +death44 +town desktop passed12.4s;
separate valid town day0/desk sections passed4.2s (earlier map/plots were invalid
section names and covered only desk). Build/typecheck and copy1535/zero
violations pass, existing bundle advisory. No full balance/performance claim.
