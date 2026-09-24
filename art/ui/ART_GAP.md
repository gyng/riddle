# In-game art vs `art/ui/targets/watch.png` — gap list (2026-09-24)

Baseline: headed GPU (D3D12, RTX 3080), 400×800 @ 3×, `?dev=1&seed=601&fresh=1&autosend=1`, map and
fight frames on D1–D3 (the Warrens). Shots: scratchpad `art-pass/before_*.png`.

| # | Gap | Target (watch.png) | Now | Asset / renderer |
|---|---|---|---|---|
| 1 | **Tile resolution** | ~16 px of pixel detail per tile: flagstones, mortar, chips | 8×8 texels; after the 8-colour quantise a floor is a flat field with speckles | both: 16×16 tiles (asset) + a tile slot that keeps its size (renderer: `atlas.ts` forced 8×8) |
| 2 | **Wall depth** | walls have a lit capstone ledge + a brick FRONT face toward the viewer, and a dark TOP mass behind | every wall tile is the same brick square with a dark rim on all sides: walls read as isolated blocks / "barcode" | renderer: autotile by the neighbour below (face vs top) and edge-rim the tops; asset: `wall_face`, `wall_top` |
| 3 | **Floor variation** | 3–4 slab layouts, cracks, moss in seams, dried blood, rubble | one floor + one `floor_alt` | asset: 4 floor variants + moss/crack/rubble/blood decals; renderer: hash-by-tile choice |
| 4 | **Props / dressing** | torch sconces every few tiles, barrels, crates, pots, bones, banners, a portcullis | none (only situation props: shrine/vault/nest, 8×8) | asset: torch, barrel, crate, pot, banner, bones, portcullis door; renderer: render-only placement seeded by tile position |
| 5 | **Lighting** | warm torch pools, dark falloff between them; high value range | flat: lit/remembered dim + two fog bands around the hero | renderer: torch lights in the blit (pre-quantise value lift + a warm tint), capped count |
| 6 | **Palette / value range** | near-black to warm bone highlights, blood red and torch orange keep their hue | the whole env is quantised into the olive ramp: no red, no orange | renderer: decals/flames drawn sprite-tagged (30 % tint) so they keep hue; ramps unchanged |
| 7 | **Sprite register** | crisp pixel-art characters | watercolour-anime masters box-downscaled to 48/32 texels, ink ring, 30 % tinted — at runtime they already read as crisp pixel sprites | asset: keyed register v2 (user direction mid-pass) — every sprite regenerated, ART.md updated |
| 8 | **Name tags / hp bars** | serif name over a framed hp bar | bitmap caps under the feet; hp bar in the fight frame | client/UI track (not art) |

Priority: 1 + 2 carry most of the difference (the frame is mostly floor and wall), then 5, 4, 3.

## After the pass (same scenes; scratchpad `art-pass/side_by_side.png`)

Closed: 1 (16×16 tiles, register 3), 2 (face/top autotiling + rimmed tops), 3 (4 floor variants, moss/crack/rubble/
blood/bones decals), 4 (torches, banners, barrels, crates, pots, portcullis doors), 5 (12 torch lights + a soft hero
light in the blit; ambient a ramp step down, dither only inside pools), 6 (blood/flame/banner keep hue), 7 (keyed register
v2: every sprite regenerated; see ART.md), plus portraits (well, top bar, party cards, boss bar).

Remaining:
- The map frame is still darker and busier than the target at 20 tiles across (the target frames ~14): the fog bands
  (0.8/0.6 by distance) and the remembered-tile dim (0.6) stack with the ambient; a tuning pass on those, not art.
- Runs of adjacent door tiles (the generator's room mouths) render as an arcade of portcullises; a door in a vertical
  wall is still drawn front-on.
- Situation props (shrine/vault/nest) and item glyphs are still the 8×8 register (visibly chunkier beside the 16×16 set).
- Sanctum's pale ramp still blooms near a torch/the hero (the lift is scaled by value, not enough on marble).
- Name tags / hp bars are the bitmap font, not the target's serif plates (client track).
- Sprites: `forge_imp`'s flask came out red (tag weaker); `rat` reads small; `goblin_captain`/warlord skin is grey-olive.
