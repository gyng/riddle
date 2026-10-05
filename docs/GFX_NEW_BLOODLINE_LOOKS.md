# New bloodlines arrive with distinct looks — 2026-10-06

Real three-hero roster uses identical male fighter art. On add_bloodline only,
choose the first unused Class::LOOKS cosmetic among current resolved looks;
existing three supported looks/male-female-cat portraits and sprites reused.
No initial-town default change, class/stat/RNG change, old save migration or
reassignment of existing heroes. Existing custom choices count as occupied;
each new look is inherited by its bloodline and remains editable via Appearance.
If all looks are occupied, retain class default fallback for future slot caps.

Acceptance: normal three fighter slots resolve male/female/cat; chosen cat in
first slot means next two choose male/female; adding/selecting/loading never
changes existing choices; new heirs preserve their chosen look. Same core runs
and complete raw state except lineage.look for cloned sessions with/without
cosmetics. Rust tests/clippy, rebuilt WASM, UI appearance regressions/build/copy,
headed real legal three-slot purchase roster/menu screens400/1440 with no
overflow/warnings. No art generation or deployment.

Verified:555 fast workspace tests pass36.84s/1 ignored; clippy, rebuilt real
WASM5244053bytes, build/typecheck/copy1511/diff pass (existing chunk advisory).
Core tests cover chosen-cat first slot, class-default female first slot, all
three defaults, failed-full-slot no mutation, exact save/load/select, inherited
looks and old save missing look unchanged. Actual three-run reports identical
and full Session raw state equal after removing only each lineage.look against
a clone with cosmetics cleared; no RNG/stat changes.

Existing slot/report appearance54 and Appearance route45 client checks across
320/400/1440 pass5.7s batch. Headed legal seed15 saved camp/new slot2/two12h
slices/new slot3: male/female/cat core looks and roster asset ids, female hero
menu, reload/selection/purchase retains existing look choices, no warnings/
horizontal overflow400/1440. Screens inspected/shown. Reuses existing anime
cat-folk portrait with cat ears; no new art, class defaults, historical portrait
backfill or view geometry. Only new residents get automatic unused appearance.
Artifacts scratchpad/new-bloodline-looks-20261006/{400,1440}-{roster,hero}.png,
*-real.json, real-preview.mjs and test/build logs. Existing runtime workers may
spend gold while headed UI checks run; no screenshot gold invariance claim.

Next priority: measure current multihero catch-up cost before changing shared
clock advancement. This is distinct from old single-hero timing evidence and
should preserve exact shared-town spending and worker ordering.
