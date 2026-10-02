# Town art ids (Cut 30 §3, the Split art row) — the renderer's binding list

Branch `c30-art`. Every id below either exists now (**have**) or is coming (**new**). The renderer binds by id and draws
the **fallback** until the id is in `web/public/art/atlas.json` `frames` (sprites, tiles) or `web/src/ui/skin.json`
`icons` (package icons). Nothing here blocks the game: a missing id is a coloured block + its icon.

## Units and anchors

- **World unit = one env texel.** A terrain tile is 16 × 16 units (one map cell). Sprite frames in the atlas are
  *masters* at 2 px per unit: draw a frame at `w/2 × h/2` units (`meta.sprites[id].texel_h` = `h/2`), nearest.
- **Keyed sprites** (buildings, props, walkers, carry): anchor **bottom-centre** of the frame = the ground point (a
  building's front-wall base, a walker's feet). Depth-sort by anchor y.
- **Terrain tiles** (`town_env_*`, 16 × 16): anchor top-left on the 16-unit grid, seamless, opaque.
- **Env props** (`town_env_fence`, `_low_wall`, `_gate` 16 × 24, `_tree_0` 16 × 32, `_tree_1` 16 × 24, `_bridge`): anchor
  bottom-centre on their cell (taller ones overhang the cell above). Alpha outside the object.
- **Lights** (night, the light field): `art/town_lights.json` lists each sprite's warm emitters as `[dx, dy, strength]`
  in units from the anchor (dy < 0 is up): torches, the forge, the campfire, lit windows.
- Facing: walkers and townsfolk face **right**; flip for left.

## Terrain (town ramp, 16 × 16, painterly register)

| id | what | fallback |
|---|---|---|
| `town_env_grass_0..3` | night grass ×4 (calm base · flowers · stones · long) | ramp `#344143` |
| `town_env_dirt_0..2` | trodden path ×3 (base · puddle · prints) | ramp `#567383` |
| `town_env_dirt_edge_{n,e,s,w}` **new** | path cell with grass on that side | dirt fallback |
| `town_env_dirt_corner_{ne,nw,se,sw}` **new** | path cell, outer corner: grass on both named sides | dirt fallback |
| `town_env_dirt_inner_{ne,nw,se,sw}` **new** | path cell, grass only in that corner (a path's inside bend) | dirt fallback |
| `town_env_dirt_edge` | (have; = `_edge_n`, kept as an alias) | |
| `town_env_plaza_0..1` | cobbled plaza ×2 | ramp `#455a63` |
| `town_env_fence` | fence run, 16 × 16 prop (repeat along x; posts at the cell edges) | 2 px dark rails |
| `town_env_low_wall`, `town_env_gate` (16 × 24) | later stages (have) | |
| `town_env_water`, `_cliff`, `_bridge`, `_tree_0` (16 × 32), `_tree_1` (16 × 24) | edges and dressing (have) | |

## Landmarks and props (keyed, `kind: "town"`)

| id | frame (master px) | units (w × h) | what | fallback (block + icon) |
|---|---|---|---|---|
| `town_mouth_cave` | 201 × 128 | 100 × 64 | the dungeon mouth, two torches (tap = send) | INK block + `depth` |
| `town_mouth_timber`, `town_mouth_gate` | ~195 × 128 | ~97 × 64 | later mouths (have; not v1) | |
| `town_campfire_0`, `_1` | 52 × 48 | 26 × 24 | campfire, 2 frames (~180 ms) | EMBER dot + `v_rest` |
| `town_tent` | 83 × 80 | 41 × 40 | the hero's tent (= portrait) | DUSK block + `camp` |
| `town_crate` | 37 × 40 | 18 × 20 | supply crate (= the pack) | UMBRA block + `loadout` |
| `town_plot` | 44 × 40 | 22 × 20 | staked plot (the next building) | outline + `alert` |
| `town_scaffold` | 85 × 96 | 42 × 48 | scaffold (the build beat) | outline block |
| `town_board` **new** | ~64 × 64 | ~32 × 32 | notice board by the mouth (the quest) | UMBRA block + `chronicle` |
| `town_flag_0`, `_1` | 28 × 48 | 14 × 24 | BLOOD flag, 2 frames (have) | |

## Buildings (keyed, 128 px masters = 64 units tall)

| id | frame | units | v1 use | fallback |
|---|---|---|---|---|
| `town_blacksmith_1` / `_2` / `_3` | 163 / 169 / 161 × 128 | ~82 × 64 | look 1, looks 2–3 by forge steps | DUSK block + `forge` |
| `town_storehouse_1` (`_2`, `_3` have) | 142 × 128 | 71 × 64 | look 1 | DUSK block + `vault` |
| `town_kennel_1` (`_2`, `_3` have) | 142 × 128 | 71 × 64 | look 1 | DUSK block + `party` |
| `town_bank_1` / `_2` / `_3` | 129 / 157 / 191 × 128 | 64 / 78 / 95 × 64 | look 1, looks 2–3 by bank cap | DUSK block + `gold` |

The front door of each building sits within ±8 units of the anchor x, at the anchor y (walkers path to the anchor).

## Walkers and carry (keyed)

| id | frame | units | what |
|---|---|---|---|
| `walk_fighter`, `walk_rogue`, `walk_ranger`, `walk_caster` | ~68 × 96 | ~34 × 48 | the walk frame per class; alternate with `hero_<class>` for the 2-frame walk, bob 1 unit |
| `town_sack_small` | 21 × 24 | 10 × 12 | carried sack (gold below the cut) — draw at the walker's hand, ~(+8, −14) |
| `town_sack_large` | 30 × 32 | 15 × 16 | big sack |
| `town_chest_glow` | 32 × 32 | 16 × 16 | glowing chest (a find) |
| `town_smith`, `_merchant`, `_child`, `_carter`, `_cart`, `_dog` | | | townsfolk (have; not v1) |

Fallback for walkers: the class's `hero_<class>` frame (always present); for carry: a 4 × 4 BONE / GILT block.

## Package icons (UI, `art/ui/icons/` → `web/public/ui/icons/<id>.png`, listed in `skin.json icons`)

| id | package | subject |
|---|---|---|
| `pkg_steady` | stance Steady | a planted round shield over a sword |
| `pkg_guarded` | stance Guarded | a tower shield, raised |
| `pkg_bold` | stance Bold | a sword thrust forward, a BLOOD streak |
| `pkg_hunter` | stance Hunter | a drawn bow with a target arrow |
| `pkg_skittish` | temperament skittish | a hare's head, ears back |
| `pkg_unbowed` | temperament unbowed | a helm with a straight crest, unbent |
| `pkg_light_hands` | temperament light hands | an open hand with a coin |
| `pkg_iron_gut` | temperament iron gut | a flask with an iron band |

Tactics reuse the card icons (`v_*`). Fallback: the package's first letter on the chip (no icon).
