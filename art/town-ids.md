# Town art ids (Cut 30 §3, the Split art row) — the renderer's binding list

Branch `c30-art`. The renderer binds by id and draws the **fallback** until the id is in `web/public/art/atlas.json`
`frames` (sprites, tiles) or `web/src/ui/skin.json` `icons` (package icons). A missing id is a coloured block + its icon:
art never blocks the game. Reference layout and contact sheet: `python3 art/town_sheet.py` (day | night, 400 × 800 at 2×).

## Units, sizes and anchors

- **World unit = one env texel** (as `layers.ts`). A terrain cell is **16 × 16 units**.
- **Keyed sprites** are drawn at `texel_h × SPRITE_SCALE` (0.5, `palette.ts`, the watch's scale) units tall, so a hero is
  **24 units**, a building **64** (4 cells), the mouth **64 × 101**. The atlas frame is the 2× master: **units = frame px / 4**
  (the "units" column below). The building/hero ratio (64 : 24) is the target images' (people about a third of a house).
- **Anchor: bottom-centre** of the frame for every keyed sprite (a building's front-wall base, a walker's feet, a prop's
  foot). Depth-sort by anchor y. Each building's door is within ±10 units of the anchor x at the anchor y: path walkers to
  the anchor.
- **Terrain tiles** (`town_env_*`, 16 × 16 px = units, opaque): top-left on the grid, seamless.
- **Env props** (`town_env_fence` 16 × 16, `_low_wall`, `_gate` 16 × 24, `_tree_0` 16 × 32, `_tree_1` 16 × 24, `_bridge`): px =
  units, bottom-centre on their cell; alpha outside the object.
- **Lights**: `art/town_lights.json` = `{id: [[dx, dy, strength], …]}`, the warm emitters painted into each building, mouth
  and campfire (torch flames, the forge mouth, lit windows) as offsets in units from the anchor (dy < 0 is up), strongest
  first, ≤ 4 each. Feed them to the light field at night (`python3 art/town_lights.py` regenerates it from the atlas).
- Facing: walkers face **right**; flip for left. Walk = alternate `walk_<class>` and `hero_<class>` (bob 1 unit).
- Atlas: `pack.py` now packs **1024 px wide** (the 256 px building masters made a 512-wide atlas 5474 px tall; 1024 × 2404
  stays under every GPU's 4096). `atlas.ts`'s `Sheet(512, 512)` sprite sheet is too small for the buildings at full size:
  draw town frames straight from the atlas texture, or give the town its own sheet.

## Terrain (the town ramp, painterly register: `art/town_tiles.py`, run by `make_env.py`)

Grass is the dark moss field; the path is the pale moonlit ground (a clear value step above the grass); the plaza between.

| id | what | fallback |
|---|---|---|
| `town_env_grass_0..3` | grass ×4 (calm base · flowers · stones · long), one mean so a field never checkers | `#344143` |
| `town_env_dirt_0..2` | path ×3 (base · puddle · prints) | `#7d98ad` |
| `town_env_dirt_edge_{n,e,s,w}` | path cell, grass along that side (a fringe 3–5 texels deep, 4 at both ends: runs join) | dirt |
| `town_env_dirt_corner_{ne,nw,se,sw}` | path cell, outer corner: grass along both named sides | dirt |
| `town_env_dirt_inner_{ne,nw,se,sw}` | path cell, grass only in that corner (inside of a bend) | dirt |
| `town_env_dirt_edge_ns`, `_edge_ew` | a one-cell path running east–west / north–south | dirt |
| `town_env_dirt_end_{n,e,s,w}` | a dead end capped on that side (grass on it and both flanks) | dirt |
| `town_env_dirt_isle` | a lone path cell (grass all round) | dirt |
| `town_env_dirt_edge` | alias of `_edge_n` (the phase-2 id) | |
| `town_env_plaza_0..1` | cobbled plaza ×2 | `#567383` |
| `town_env_fence` | fence run (repeat along x) | 2 px dark rails |
| `town_env_low_wall`, `_gate`, `_water`, `_cliff`, `_bridge`, `_tree_0`, `_tree_1` | later stages and dressing | |

Picking a path tile: for a path cell, the sides whose neighbour is not path (n, e, s, w): none → `dirt_inner_<c>` if a
diagonal neighbour is grass, else `dirt_<0..2>`; one → `edge_<s>`; two adjacent → `corner_<ns><ew>`; two opposite →
`edge_ns`/`edge_ew`; three → `end_<the side opposite the open one>`; four → `isle`. Plaza cells count as path.
`art/town_sheet.py path_tile()` is the reference.

## Landmarks and props (keyed, `kind: "town"`)

| id | frame px | units | what | fallback (block + icon) |
|---|---|---|---|---|
| `town_mouth_cave` | 404 × 256 | 101 × 64 | the dungeon mouth, two lit torches (tap = send) | INK block + `depth` |
| `town_mouth_timber`, `_gate` | ~390 × 256 | ~97 × 64 | later mouths | |
| `town_campfire_0`, `_1` | 69 × 64 | 17 × 16 | campfire, 2 frames (~180 ms) | EMBER dot + `v_rest` |
| `town_tent` | 133 × 128 | 33 × 32 | the hero's tent (= portrait) | DUSK block + `camp` |
| `town_crate` | 44 × 48 | 11 × 12 | supply crate (= the pack) | UMBRA block + `loadout` |
| `town_plot` | 71 × 64 | 18 × 16 | staked plot (the next building) | outline + `alert` |
| `town_scaffold` | 197 × 224 | 49 × 56 | scaffold (the build beat) | outline block |
| `town_board` | 95 × 112 | 24 × 28 | notice board by the mouth (the quest) | UMBRA block + `chronicle` |
| `town_flag_0`, `_1` | 82 × 144 | 20 × 36 | BLOOD flag on a pole, 2 frames | |

## Buildings (keyed, 256 px masters = 64 units tall)

| id | frame px | units | v1 use | fallback |
|---|---|---|---|---|
| `town_blacksmith_1` / `_2` / `_3` | 328 / 339 / 323 × 256 | ~82 × 64 | look 1; looks 2–3 by forge steps | DUSK block + `forge` |
| `town_storehouse_1` (`_2`, `_3` exist) | 285 × 256 | 71 × 64 | look 1 | DUSK block + `vault` |
| `town_kennel_1` (`_2`, `_3` exist) | 285 × 256 | 71 × 64 | look 1 | DUSK block + `party` |
| `town_bank_1` / `_2` / `_3` | 258 / 315 / 385 × 256 | 64 / 79 / 96 × 64 | look 1; looks 2–3 by the bank cap | DUSK block + `gold` |

## Walkers and carry (keyed)

| id | frame px | units | what |
|---|---|---|---|
| `walk_fighter`, `_rogue`, `_ranger`, `_caster` | ~69 × 96 | ~17 × 24 | the walk frame per class (BONE moon rim, like `hero_*`) |
| `town_sack_small` | 21 × 24 | 5 × 6 | carried sack (small purse); at the walker's hand ≈ (+7, −8) |
| `town_sack_large` | 30 × 32 | 8 × 8 | big sack |
| `town_chest_glow` | 32 × 32 | 8 × 8 | glowing chest (a find), carried in front ≈ (+6, −8) |
| `town_smith`, `_merchant`, `_child`, `_carter`, `_cart`, `_dog` | | | townsfolk (exist; not v1) |

Fallback for walkers: the class's `hero_<class>` frame (always present); for carry: a 4 × 4 BONE / GILT block.

## Package icons (UI: `art/ui/icons/` → `tools/ui-skin.py` → `web/public/ui/icons/<id>.png`, listed in `skin.json icons`)

| id | package | subject |
|---|---|---|
| `pkg_steady` | stance Steady | a large round shield, a sword crossed behind |
| `pkg_guarded` | stance Guarded | a tower shield, iron cross-bands |
| `pkg_bold` | stance Bold | a charging spear with a BLOOD pennant |
| `pkg_hunter` | stance Hunter | a drawn bow, a MIST crosshair at the arrowhead |
| `pkg_skittish` | temperament skittish | a hare's head, ears laid back |
| `pkg_unbowed` | temperament unbowed | a crested helm, cracked but unbent |
| `pkg_light_hands` | temperament light hands | an open hand, a GILT coin rising |
| `pkg_iron_gut` | temperament iron gut | an iron-banded flask |

Tactics reuse the card icons (`v_*`). Fallback: the package's first letter on the chip (no icon).
