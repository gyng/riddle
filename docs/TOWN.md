# The town — the home screen as a living hub

*2026-09-28, design note for the idle-first pivot (`docs/IDLE_FIRST.md` §7, the town track). The owner: "the home
screen should be a city/town/village/hub — a garden with things moving." No game code. Targets (not assets; the
client matches their layout and hierarchy) in `art/ui/targets/town/`: `phone_day0`, `phone_village`,
`phone_village_night`, `phone_town`, `desktop_town` (prompts `art/prompts/town_targets.txt`); frame from `docs/UI.md`.*

## 1. The idea

The camp stops being a form over a vista and becomes **the place**: a clearing at the mouth of the dungeon that
grows into a walled town as the lineage earns. Every building is an entry point (tap it, its panel opens); every
number the player earned is something standing there. Things move whether or not you touch them: heroes walk to the
dungeon mouth and come back with loot, smoke rises, pets trot, townsfolk multiply with the houses. Coming back after
a night should read at one glance: *the town got bigger, the party is walking home with a sack.*

The scene is a view of `Lineage` (buildings, heroes home/away, pets, best depth, the absence's runs): the core owns
what is built, the client draws it. Walkers are cosmetic, seeded by lineage id + day, never read back.

## 2. Stages (the town track, each building appears on the scene when built)

| Stage | Opens on | On the scene | What it does (one line) |
|---|---|---|---|
| **0 · camp** (day 0) | fresh lineage | campfire, one tent, a supply crate, the dungeon mouth (a cave, two torches), the lone hero; one path | the crate = the pack (loadout); the tent = the hero (portrait); the mouth = send |
| 1 · hamlet | first gold home | **blacksmith** (open forge, anvil) | the forge: kit steps, salvage |
| | first find kept | **storehouse** (the crate grows into a shed) | the vault / keep |
| | first tame | **kennel** (shed, fenced pen) | pet slots, incubation; the party |
| 2 · village | a purse ≥ one night's net | **bank** (stone vault, coin sign) | deposits earn ~2 %/night, capped at 3 nights' net |
| | banks at D13 / 18 / 23 | **houses 1–3** (timber cottages) | a hero slot each: parallel expeditions |
| | first house | **tavern** (two storeys, mug sign) | hire heroes; rumours = today's one bounty |
| | first counter | **library / shrine** (small tower, teal window) | drills, package levels; the pen (rules) when it opens |
| 3 · town | the first fork | **watchtower** (timber tower, flag) | forecast detail: the next band's foes, the route |
| | heir 5 | **hall of heirs** (long hall, banners) | chronicle, ledger, bloodlines, titles |
| | house 3 | a low wall and gate; the mouth becomes a carved **gatehouse** | none: a trophy of scale |

Each building has 3 looks (built · improved · grand), stepped by its own level (forge steps, bank cap, library
levels…). A new building arrives as a one-beat reveal: scaffold → built with a dust puff and a glint, no sentence.
Unbuilt plots are **empty grass**, never locked icons; the *next* stage shows as a staked plot with a small marker
(its cost on tap) so the track's next step is always visible (the IDLE_FIRST gate "the next stage always shown").

## 3. What moves

- **Heroes**: at SEND the hero walks from the fire/their house to the mouth and goes in. While away, they are gone
  (the house's window is dark). On return the absence's runs play out of the mouth as **walking parties**: one per
  run, ≤ 3 on screen, compressed to ~20 s total, each carrying a sack sized by its gold (a glowing chest for a find,
  a pet trotting behind for a tame). A death walks nobody out; the hall lowers a banner, a small stone appears by it.
- **Heroes at home** wander a loop: house → tavern → forge → fire, pausing 2–6 s at each door.
- **Pets** follow their hero, or lie in the kennel pen. **Townsfolk** scale with the scale track: 0 at camp, +2 per house, +1 per 2 heroes, cap 12 (smith at the anvil,
  a merchant, children, a mule cart late).
- **Ambient**: chimney smoke (fx particles), forge sparks, 2-frame banners and flags, fireflies at dusk, birds by
  day; **day/night from the player's local clock** (coming back in the morning shows morning); night lights windows,
  torch posts and the forge through the existing light field. Weather (rain, fog by the deepest biome) is a later knob.
- **Markers** (≤ 3 at once): a coin over the bank when interest is ready, a `!` rune over a building whose panel
  has something new (the reveal ladder's glint), a sword over the forge when a kit step is affordable (today's badge).

## 4. The camp's functions, mapped

| Today (console tile / panel) | In the town |
|---|---|
| SEND gem | stays the gem; tapping the **dungeon mouth** does the same (the hero walks in, the camera dives into D1 → watch) |
| loadout (supplies) | the supply crate at stage 0, then the storehouse |
| vault / keep | storehouse |
| forge | blacksmith |
| party, cage | kennel |
| unlocks, cards, packages | tavern (heroes, recruits, rumours) and library (packages, drills) |
| edit / rule tablets, order | library, once the pen opens (IDLE_FIRST: rules late); until then no tablets on the home screen |
| forecast shaft, route, waystones | a small depth plaque on the mouth (`D12`) at first; the watchtower opens the full shaft |
| ledger, chronicle | hall of heirs |
| report | the returning parties, then the report parchment; panels stay UI.md sheets, anchored above their building |

## 5. Frame and layouts

- **Top bar** (unchanged object, grows with the reveal ladder): portrait-mini, `$`, `◆`, `★`, hero count, `D`best;
  cog right. Day 0 shows `$ 0` only.
- **Console** (unchanged object): portrait well left, command card middle, SEND gem right. The command card becomes
  the **building bar**: one tile per built building, in build order (tile and building open the same panel, badges
  shared). It is the accessible/fast path; the scene is the fun one. Empty sockets stay dark stone.
- **Phone 400 × 800**: bar 48 px · scene ~560 px · console ~190 px. The scene is a portrait crop of one town map
  (~40 × 60 tiles), the mouth at the top, the fire at the centre. The camera zooms out a step per stage (3× → 2×
  texels) so the whole town fits without panning; a vertical drag pans once it does not. Buildings ≥ 44 px tap
  targets; each has a hidden DOM label for tests and screen readers.
- **Desktop 1440 × 900**: the same map as a landscape crop (mouth top centre, town spreading left/right, fields
  and river at the edges). A thin full-width top bar; a WC3-style console centred along the bottom (~960 px); a
  carved minimap plaque bottom-left (the town + the mouth; tap = pan); a hero roster plaque bottom-right (portraits,
  `home`/`away`). Panels open anchored above their building.
- Gates: day 0 ≤ 4 interactive surfaces (IDLE_FIRST §3: mouth, crate, portrait, gem); ≤ 12 elements above the
  fold at 400 × 800 at any stage; 60 fps on the GPU harness with 24 walkers; ≤ 30 fps idle on the camp when no
  input for 10 s, 0 when hidden.

## 6. Tech recommendation — the existing three.js renderer, a `town` scene

Recommended: **draw the town with the existing pixel renderer** (`web/src/render/`) as a second scene beside the
replay view, with DOM hit-targets over the buildings.

- It already has what the scene needs: the atlas and instanced quad layers (`layers.ts`: terrain tiles + building
  and walker sprites, one draw call each), the palette/dither blit that gives the look, the **light field**
  (`light.ts`: night windows, torch posts, the forge, for free), bloom, fx particles (smoke, sparks, dust), and the
  Canvas-2D fallback (`view2d.ts`) for a lost or missing GL context.
- One canvas for camp and watch: SEND becomes a camera dive through the mouth into D1 instead of a screen swap.
- Scene model (`render/town.ts`, client): a fixed tile map with named plots and a path graph (nodes at doors, the
  fire, the mouth); `townState(lineage, absence)` → buildings (plot, look), walkers (sprite, path, carry), markers.
  Walkers step along path edges with a 2-frame bob and flip (as the watch does), depth-sorted by foot y.
- Rejected, DOM/CSS sprites over a painted backdrop: easier hit-tests, but no light field or palette pass, a second
  render path for one look, no dive. DOM keeps what it is good at: labels, tap targets, the frame, panels.
- The core adds `Lineage.town` (buildings, levels, plots) and the heroes' home/away state; nothing else.

## 7. Art list (Codex, ART.md registers, each with a primitive fallback: a coloured block + its icon)

- **Terrain** (register 3, a new `town` ramp, 8 × 8): grass ×4, dirt path ×3 + edges, plaza stone, fence, low
  wall + gate, water + bridge (desktop edges), forest edge, cliff face.
- **Landmarks** (keyed): dungeon mouth ×3 (cave · timber-braced · gatehouse), campfire (2 frames), tent,
  supply crate, staked plot, scaffold.
- **Buildings** (keyed, 3 looks each): storehouse, blacksmith, kennel, bank, house (+2 roof recolours), tavern,
  library/shrine, watchtower, hall of heirs = 27 masters (IDLE_FIRST's 9 × 3).
- **Walkers**: the class sprites and looks exist; add a walk frame per class (4), carry props (sack small/large,
  glowing chest), townsfolk ×4 (smith, merchant, child, carter + mule cart), a dog; pets exist.
- **Ambient**: torch post (`env_torch` exists), banner and flag (2 frames), window glow mask, birds, fireflies,
  smoke/spark/dust as fx (no art).
- **UI**: markers (coin, `!` rune, sword; from `art/ui/icons`), minimap plaque frame, roster plaque frame, building
  tiles for the command card (anvil, paw, coins, mug, book, tower, scroll exist or are close).
