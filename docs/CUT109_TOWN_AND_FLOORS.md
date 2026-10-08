# Cut 109 — a living town and floors with a plan

*2026-10-08. Owner: "Show home screen more love and polish. Check building sprites art and perspective
consistency. Living town enhancements. More varied dungeon generation and variety." Blind cards on
every cohort name the same two gaps: the camp's lower half is empty black (aesthetic), and "D1–D7
repeat nearly verbatim each run" / "the same rooms every floor" (surprise, pacing; docs/PATH_TO_95.md §2).*

## 1. Town (client only; `web/src/render/town.ts`)

Audit (sprite sheet, rater B's night camp on ad71e72):

- The hero's house drew `town_storehouse_1` (no `town_house_*` art existed), so day 2 showed **two
  storehouses**. → `town_house_1..3` painted in the buildings' high three-quarter camera (art/prompts/town_v2.txt);
  the look grows with the stage (1 · 2 at two buildings · 3 at four), its windows lit at night (`town_lights.json`).
- `town_board` was a flat front elevation among three-quarter buildings. → repainted from above.
- Docs/TOWN.md §3's townsfolk, lamps and fireflies were unbuilt; the camp's lower third was bare grass.
- Night ambient `[.2,.28,.52]` sank everything but the emitters. → `[.27,.34,.58]`.

Dressing arrives with the stage (`PROPS`), its glades kept clear of trees from day 0:
woodpile (camp) · lamppost (1) · well, lamppost (2) · market stall, parked cart (3) · lamppost (4).
Townsfolk (`folk`): the smith at the forge (blacksmith), a child about the fire (2), the merchant at
the stall (3), the carter with his cart on the south road (4). Fireflies from dusk (14, blinking).
All cosmetic, seeded by lineage + day, never read back; the 2D fallback draws the same frame.

Gates: `cut30town`, `first-plot`, `ready-plots`, `town-gpu` (fps with 24 stress walkers), `ctxloss`,
`persistent-camp` unchanged and passing; art-qc 0 warnings; town CPU p95 per frame not above the
pre-cut figure + 0.5 ms on the GPU harness.

## 2. Floors (core; `crates/riddle-core/src/gen.rs`)

Before: one algorithm for five biomes — rectangles 3–6 × 3–5 and L corridors; the same seed gave
the **identical** plan in the Warrens, Burrows, Crypt, Foundry and Sanctum. Caves had one density.

After: each floor draws a **layout** from its biome's pool, then each room a **shape** inside its box:

| biome | layouts (share) | character |
|---|---|---|
| Warrens | classic 70 · halls 30 | familiar small rooms; some bigger, pillared |
| Burrows | tunnels 65 · classic 35 | round chambers, corridors bent through a waypoint |
| Crypt | crypt 65 · catacomb 35 | crosses and pillared halls with chasms · a maze of cells |
| Foundry | works 70 · halls 30 | few big pillared halls |
| Sanctum | sanctum 70 · crypt 30 | a pillared nave mid-floor, round and cross chapels |
| Fens, Deep | cave | density 51–58 %, 0–3 grottoes; the Fens' mere (40 %) |

Generation runs on its **own stream** (`gen::GEN_TAG`: one draw from the caller's Rng seeds it), so a
layout change never moves the rolls after it — combat, loot, situations. After generation every open
tile the stairs cannot reach is walled (a round room can leave a cell touching only at a corner: a
monster spawned there read `rat, no path`).

Invariants kept: `Floor.rooms` are the boxes (room_ref, the lock, situations read them); every shape
keeps its box's centre open (stairs, corridor anchors); doors only where a corridor meets a room
from outside its box; corridor tiles exist on every floor (rules `to corridor`); determinism.
Floors already generated (in a save's run) are serialized and unchanged.

Gates: `gen::tests::layouts_vary_and_connect` (≥ 6 distinct plans per biome in 60 floors, ≥ 30
shaped rooms, every floor connected, every room centre open); all Rust tests; the 307dbed send
fixture re-recorded with its old → new hash and this reason; the routine full gate table
(`node tools/gates.mjs --full --fresh`) passing **without lowering any threshold** — content
retuned if a row moves.

`examples/floors.rs` prints plans for a look (`-- crypt 15 4`); not a gate.
