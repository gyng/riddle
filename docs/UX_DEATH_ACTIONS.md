# Readable death action and illustrated recommendation — 2026-10-06

Actual Warlord death shows `Steady · foes 1+ → attack` and `Buy sword +1`
without an item icon. Preserve the simple cause/banner/one recommended action.
Render the server-selected cause row (else its last nonnegative trace row) using
the shared readable rule name from the death's own rules, not the current camp's
edited rules. Keep historical package/drill attribution. Missing historical
rows can use the matching trace verb; otherwise show only the known source.
Do not infer an action or condition from opaque row-description text.

Use shared unframed item icons for spend recommendations and expressive style
icons for identified package recommendations. Unknown art retains existing
primitive fallback. No rarity guesses, new controls or simulation edits. Also
fix targeted consumable rules to draw the item before their comma-separated
combat target rather than a miscellaneous glyph.

Acceptance at400/1440: saved real Warlord death reads `Steady · attack nearest`,
has a decoded sword silhouette, no operators/overflow; historical rules win over
edited camp rules; cause-row selection, multiple trace rows, negative-row chores,
drills and missing-history fallbacks are checked. Recommendation behaviour
remains exact: spend opens Forge, package equips only its owned choice, wait
respects historical-death send restriction. Targeted fire/unknown potion/scroll
rule icons keep their item family. Scoped client checks, tsc/copy/build/screens.
Push, no deployment or full balance audit for this rendering-only change.

## Results

- Scoped death-actions client check:44 assertions at400/1440 in2.2s,
  including historical rules, cause row0, missing history, drill attribution,
  exact recommendation controls and targeted consumable item families.
- Existing item-icons214 and chrome geometry184 checks pass. TypeScript/web
  production build, copy1492 and diff checks pass.
- Actual release-WASM saved Warlord death3 from camp-0 at400/1440:
  `Steady · attack nearest`, decoded unframed sword, zero warnings/overflow.
  Screens inspected/shown; artifacts scratchpad/death-actions-20261006/.
- Separate headed live Captain diagnostic timed out waiting for its rectangle
  after90s. No capture or live-boss flow certification. Inspect live state before
  retrying; this diagnostic did not require or justify changing game rules.
- No Rust changes, full balance audit or deployment.
