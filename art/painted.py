#!/usr/bin/env python3
"""The painted 16x16 tile register (the owner's call, gfx round 18): per biome ~15 pieces painted by Codex in the target mockups'
lit-stone language (art/ui/targets/watch.png), converted to 16x16 at full colour (<= PAINT_COLOURS per tile, no 8-colour ramp) and
written over the ramp register's `art/tiles/<biome>_env_<name>.png`, so the renderer draws them 1:1 with the sprites (a tile is 16
texels on the sprite grid, an 8-env-texel world quad) with no id change; the ramp register stays the fallback for any piece not painted.

    python3 art/painted.py prompts [biome ...]   # writes art/prompts/paint_<biome>.txt (one Codex batch per biome)
    python3 art/painted.py manifest              # adds the `envp_<biome>_<name>` entries to art/manifest.json (idempotent)
    (conversion runs inside art/make_env.py: `convert_painted()`)
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
MOON_LIFT = {"burrows": 0.8, "fens": 0.4, "deep": 0.75, "foundry": 0.45}   # art direction phase 2: see convert()
EDGE_LIFT = {"burrows"}
CALM = {"fens": 0.7}   # places whose floors are damped to background (convert)
PAINT_COLOURS = 24   # per tile (art-qc allows this for the painted register, `_painted.json`)

# the pieces: name -> (bg, texels, what) — the drawing brief shared by every biome; the biome adds its materials
PIECES: dict[str, tuple[str, str, str]] = {
    "floor_0": ("env", "16x16", "the common FLOOR: one large square flagstone filling the tile, its four edges a thin dark mortar line (about 1/16 of the width), the stone face worn smooth with gentle value variation, lit a touch brighter toward the upper-left"),
    "floor_1": ("env", "16x16", "a FLOOR variant: the same size stone split into two or three slabs by dark mortar lines, one small chip or crack"),
    "floor_2": ("env", "16x16", "a FLOOR variant: the common flagstone with a creeping patch of moss / grime in one corner (a quarter of the tile at most)"),
    "floor_3": ("env", "16x16", "a FLOOR variant: the common flagstone, a little darker and more worn, a faint old stain"),
    "wall_face_0": ("env", "16x16", "the FRONT FACE of a wall seen from a top-down three-quarter camera: the top quarter is the lit CAPSTONE LEDGE (a bright edge running the full width), below it three courses of dressed stone blocks with dark mortar, darkening toward the bottom; seamless left-to-right"),
    "wall_face_1": ("env", "16x16", "a WALL FACE variant: the same capstone ledge and block courses, one block cracked or missing a corner, a trickle of grime; seamless left-to-right, same ledge height as the other face"),
    "wall_top": ("env", "16x16", "the TOP of a thick wall seen straight down, exactly like the target's wall tops: a grid of 2x2 heavy dressed CAPSTONE BLOCKS, each block lit on its upper-left edge and shadowed on its lower-right, MID-GREY stone a little LIGHTER than the floor (the walls must read as solid built masonry around the rooms, never as black void), dark mortar between the blocks; seamless in every direction"),
    "door": ("env", "16x16", "a DOORWAY front-on in a wall: a stone arch framing a dark IRON PORTCULLIS (a grid of black bars over deep shadow), the arch stones lit on their upper-left"),
    "stairs_down": ("env", "16x16", "STAIRS GOING DOWN seen from above: four stone steps descending away into black darkness at the bottom, each step's lip lit, a stone frame on both sides"),
    "stairs_up": ("env", "16x16", "STAIRS GOING UP seen from above: four stone steps rising toward the viewer, the top step brightest, a stone frame on both sides"),
    "water": ("env", "16x16", "still dark WATER filling the tile: deep, calm, a few soft lighter ripple strokes, no hard edges; seamless in every direction"),
    "chasm": ("env", "16x16", "a CHASM: an almost black void falling away, the faintest rock texture deep inside; seamless in every direction"),
    "barrel": ("env_keyed", "16x16", "a wooden BARREL standing upright, its round lid visible on top, two dark iron hoops, lit upper-left; the object only, on flat pure blue #0000FF"),
    "crate": ("env_keyed", "16x16", "a wooden CRATE with a diagonal cross-brace and iron corner plates, lit upper-left; the object only, on flat pure blue #0000FF"),
    "pot": ("env_keyed", "16x16", "a clay POT / urn with a dark mouth and a cracked lip, lit upper-left; the object only, on flat pure blue #0000FF"),
    # gfx round 21 (raters: "rooms are bare grey tile fields"): the dressing — the renderer's prop names, painted per biome
    "skulls": ("env_keyed", "16x16", "a low PILE OF SKULLS AND BONES, four or five skulls with dark eye sockets and a few long bones, lit upper-left; the object only, on flat pure blue #0000FF"),
    "chest": ("env_keyed", "16x16", "a closed TREASURE CHEST with iron bands and a lock plate, domed lid, lit upper-left; the object only, on flat pure blue #0000FF"),
    "rack": ("env_keyed", "16x20", "a WEAPON RACK standing against a wall, front view: two spears and a sword upright in a wooden frame, a round shield leaning at its foot; the object only, on flat pure blue #0000FF"),
    "statue": ("env_keyed", "16x24", "a small broken STATUE on a square plinth, front view: a hooded kneeling knight carved in the biome's stone, one arm broken off; the object only, on flat pure blue #0000FF"),
    "bones": ("env_keyed", "16x16", "a scatter of old BONES and a cracked skull lying flat on the floor, seen from above; the object only, on flat pure blue #0000FF"),
}

# gfx round 22 (raters, every round: the Fens "a flat undressed teal field"): pieces only one biome has — its own dressing
OWN_PIECES: dict[str, dict[str, tuple[str, str, str]]] = {
    "fens": {
        "reeds": ("env_keyed", "16x20", "a tall clump of marsh REEDS and two brown CATTAILS rising from a little tuft of wet grass, front view, pale straw-green blades with darker teal shadows, lit upper-left; the object only, on flat pure blue #0000FF"),
        "reeds_1": ("env_keyed", "16x16", "a low tuft of marsh GRASS and short reeds, bushy, straw-green and teal, a few pale seed heads, lit upper-left; the object only, on flat pure blue #0000FF"),
        "lilies": ("env_keyed", "16x16", "three round green LILY PADS floating flat, seen from straight above, one with a small pale pink-white flower, each pad with its notch, lit upper-left; the object only, on flat pure blue #0000FF"),
        "log": ("env_keyed", "16x16", "a short fallen ROTTEN LOG lying left-to-right, mossy on top, a pale fungus bracket on its side, dark cut end showing rings, lit upper-left; the object only, on flat pure blue #0000FF"),
        "ruin": ("env_keyed", "16x24", "a broken RUINED STONE PILLAR, front view: a short fluted column snapped off at an angle on a square base, green moss and hanging vines on it, weathered grey-green stone, lit upper-left; the object only, on flat pure blue #0000FF"),
        "lantern": ("env_keyed", "16x24", "a crooked wooden POST driven into the ground with an iron LANTERN hanging from a hook at its top, the lantern glowing WARM AMBER-ORANGE (the brightest, warmest thing in the swamp), front view, lit upper-left; the object only, on flat pure blue #0000FF"),
        "stump": ("env_keyed", "16x16", "a short mossy TREE STUMP with gnarled roots spreading into the mud, a few small pale mushrooms on it, lit upper-left; the object only, on flat pure blue #0000FF"),
    },
}


# art direction phase 2 (docs/TOWN.md §7, Cut 30's town v1): the town's terrain is a place of its own — only its own pieces
OWN_PIECES["town"] = {
    "grass_0": ("env", "16x16", "short night GRASS filling the tile: tufts blocked in square cells, the tinted DUSK/MOON greens, a few MIST dew glints; calm and low-contrast; seamless in every direction"),
    "grass_1": ("env", "16x16", "a GRASS variant: the same grass with a few small pale BONE flowers and one darker tuft"),
    "grass_2": ("env", "16x16", "a GRASS variant: the same grass with two small stones half sunk in it"),
    "grass_3": ("env", "16x16", "a GRASS variant: the same grass, a little darker and longer, windswept"),
    "dirt_0": ("env", "16x16", "a packed DIRT PATH filling the tile: pale trodden earth (MOON-grey with the tint), a few pebbles and cart ruts, no grass; seamless in every direction"),
    "dirt_1": ("env", "16x16", "a DIRT PATH variant: the same trodden earth with a small puddle reflecting a MIST streak"),
    "dirt_2": ("env", "16x16", "a DIRT PATH variant: the same trodden earth with two faint boot prints"),
    "dirt_edge": ("env", "16x16", "where GRASS meets a DIRT PATH: the top third of the tile grass, a ragged grass fringe, the lower two thirds trodden dirt; seamless left-to-right"),
    "plaza_0": ("env", "16x16", "PLAZA STONE: four worn square cobbles with INK joints, moonlit tops; seamless in every direction"),
    "plaza_1": ("env", "16x16", "a PLAZA STONE variant: the same cobbles, one cracked with a tuft of grass"),
    "water": ("env", "16x16", "still dark STREAM WATER: INK-teal depths with a few MIST moon streaks; seamless in every direction"),
    "cliff": ("env", "16x16", "the FACE of a rocky hill seen from the front: stacked dark rock ledges, MIST rims on each ledge top, INK below; seamless left-to-right"),
    "fence": ("env_keyed", "16x16", "a short run of wooden FENCE seen from the front: two posts and two rails of weathered dark wood, MIST rims on the tops, INK contour; the object only"),
    "low_wall": ("env_keyed", "16x16", "a short run of LOW STONE WALL seen from the front: dry-stacked stones two courses high, moonlit capstones, INK contour; the object only"),
    "gate": ("env_keyed", "16x24", "a TOWN GATE seen from the front: two stone gate posts with a wooden double gate between them, iron bands, a small lantern with an EMBER flame on one post, INK contour; the object only"),
    "bridge": ("env_keyed", "16x16", "a short wooden footbridge of planks seen from above at the high three-quarter angle, rope rails, MIST rims; the object only"),
    "tree_0": ("env_keyed", "16x32", "a tall dark PINE TREE: stacked INK/DUSK needle masses with MIST rims on their tops, a short trunk; the object only"),
    "tree_1": ("env_keyed", "16x24", "a gnarled leafless OAK: twisting INK branches with a few MIST-lit edges; the object only"),
}


def pieces(biome: str) -> dict[str, tuple[str, str, str]]:
    if biome == "town":
        return dict(OWN_PIECES["town"])
    return {**PIECES, **OWN_PIECES.get(biome, {})}


# per biome: the materials and light; the Burrows and the Fens (the D5 fork) stay distinct at a glance
BIOMES: dict[str, str] = {
    "warrens": "EXACTLY the target watch.png dungeon: grey-green dressed flagstones with olive moss in the seams, charcoal mortar, dried blood specks here and there, warm amber torchlight on grey stone. Walls: grey stone blocks with a pale capstone ledge.",
    "burrows": "a dug-out BURROW, warm and earthy (nothing grey): packed ochre-brown earth floors with pebbles and root tendrils instead of flagstones (the floor 'stone' is a slab of packed earth), walls of rough sandstone blocks shored with REDDISH TIMBER beams, a warm orange-brown palette.",
    "fens": "a drowned FEN, cool and wet (nothing ochre): the floor is a sunken BOARDWALK of weathered grey-green planks with dark teal bog water showing between them and reed tufts at the edges; walls of mossy piled stones and rotten logs; a cold teal-green palette with pale grey wood.",
    "crypt": "a CRYPT: cold blue-grey granite flagstones with carved borders, walls of tomb-niche blocks with engraved runes, faint violet shadows, bone-ivory accents.",
    "foundry": "a FOUNDRY: soot-black iron floor plates with rivets and scorched brick, walls of blackened brick with glowing orange seams and rust streaks, a hot orange-and-charcoal palette.",
    "deep": "the DEEP caves: wet blue-black cave rock floors, rounded stone walls glistening with moisture, tiny bioluminescent cyan fungus specks, an abyssal navy-and-slate palette.",
    "sanctum": "a SANCTUM: pale cream marble flagstones with thin gold inlay lines, walls of polished pale stone with gilt trim, a calm ivory-and-gold palette (keep the values mid, never pure white).",
    "town": "the TOWN at night (docs/TOWN.md): a moonlit clearing at the dungeon's mouth, grass, dirt paths, cobbles, fences.",
}

HEADER = """You are painting ENVIRONMENT TILES for the game "Riddle" (a phone roguelike seen top-down). FIRST look at
/home/g/p/riddle/art/ui/targets/watch.png (you may read that one image; read nothing else in the repository): its dungeon is
the TARGET LOOK — painterly dark-fantasy pixel art, lit stone rooms, moss, torchlight, crisp readable forms.

Use your built-in image_gen tool, one call per tile below. Do NOT write code beyond a PIL resample/alpha fix. Do NOT touch
anything outside /home/g/p/riddle/art/generated/. Do not leave _inspection_*.png or any other scratch file behind. If a file
exists at an output path, OVERWRITE it.

HOW THE TILES ARE USED (critical): each 1024x1024 image is box-downscaled to a 16x16 tile and drawn tiled edge to edge in a
top-down room, beside hand-painted characters about 1.5 tiles tall. So:
- Paint each tile as if it were a 16x16 pixel-art tile blown up 64x: BOLD shapes, 2-4 large forms, mortar / seam lines about
  64 px thick at 1024, no fine hairline detail, no text, no frame, no border, no vignette, no drop shadow.
- Opaque tiles fill the whole square edge to edge and must TILE SEAMLESSLY with themselves (floors and wall tops in every
  direction, wall faces left-to-right). Keep the floors' overall value and hue the same across variants so they mix.
- Floors are MID-DARK and calm (the characters must read on top of them); the wall tops are built masonry a little lighter than
  the floor (never black); the wall faces' ledge is the brightest line in the tile set.
- Light from the upper-left, warm; no strong cast shadows.
- Keyed props (barrel, crate, pot): the object only, centred, filling about 70% of the tile, on a flat uniform pure blue
  #0000FF background, no ground shadow, no blue on the object.
Save each as a 1024x1024 PNG at the exact path.

BIOME: {biome_desc}

=== TILES ===
"""


# ---- Art direction phase 2 (docs/ART_DIRECTION.md): the register redrawn in moonlit ink and wash, per-place tint ----
STYLE_TINT = {"warrens": "#4a3b2c", "burrows": "#5a4527", "fens": "#2d5752", "crypt": "#2f2c4f", "foundry": "#5a2a1e", "deep": "#1f2e4f",
              "sanctum": "#6b6048", "town": "#3a4a3a"}


def tinted(biome: str) -> tuple[str, str]:
    """§2's tint rule (web/src/render/wash.ts): DUSK 60 % toward the tint, MOON 40 % toward tint x 1.7"""
    hx = lambda h: [int(h[i:i + 2], 16) for i in (1, 3, 5)]  # noqa: E731
    t = hx(STYLE_TINT[biome])
    dusk = [round(d + (c - d) * 0.6) for d, c in zip(hx("#2b3350"), t)]
    moon = [round(m + (min(255, c * 1.7) - m) * 0.4) for m, c in zip(hx("#4d6c99"), t)]
    return "#%02x%02x%02x" % tuple(dusk), "#%02x%02x%02x" % tuple(moon)


BIOMES3: dict[str, str] = {
    "warrens": "the WARRENS (the first dungeon, goblin warrens): rough dressed stone flagstones and block walls exactly like watch_warrens.png — cold stone washes, moonlit capstones, a few dark moss seams, rare dried BLOOD specks. Tint: umber.",
    "burrows": "the BURROWS, a dug-out burrow — a DIFFERENT PLACE from every stone dungeon: floors of PACKED EARTH with pebbles and dark root tendrils (no flagstones, no mortar grid), walls of rough earth banks shored with dark TIMBER beams and posts. Tint: ochre — the earthiest place, but still lit by cold moonlight (no amber cast).",
    "fens": "the FENS, a drowned marsh exactly like watch_fens.png — a DIFFERENT PLACE from the Burrows at a glance: the floor is a BOARDWALK of long weathered planks with black WATER showing in the gaps, open pools with MIST moon streaks, reed tufts; walls of piled mossy stones and rotten logs. Tint: cold teal — the wettest, coldest place.",
    "crypt": "the CRYPT, like boss.png's hall: large engraved burial slabs with carved borders, walls of tomb niches with skulls, carved tomb doors, BONE accents. Tint: indigo — solemn, orderly.",
    "foundry": "the FOUNDRY: riveted iron floor plates, gratings, soot-black firebrick walls with thin EMBER seams glowing between the bricks (the only warm light), rust streaks. Tint: rust — hard-edged, industrial.",
    "deep": "the DEEP caves: unworked wet cave rock (no masonry, every edge irregular), fissures, pale MIST crystals and tiny cold fungus specks, black pools. Tint: navy — the darkest place.",
    "sanctum": "the SANCTUM: a pale temple of polished marble — diagonal checker and rosette inlays with thin GILT lines, fluted pilasters, gilded doors. Tint: pale gold — the brightest place, but keep the stone in mid values (BONE only on lit edges), still moonlit.",
    "town": "the TOWN at night, exactly like town.png: a moonlit clearing at the mouth of the dungeon — night grass, pale trodden dirt paths, worn cobbles, weathered fences, dark pines. Tint: moss — calm and cool, lit by a huge pale moon; warm light only from windows and fires the game adds.",
}

# phase 2: the shared briefs say "flagstone" and "dressed stone"; three places are not masonry at all, so their floors and walls get their
# own briefs (the D5 fork's two lanes — the Burrows and the Fens — must tell apart at a glance: earth and timber vs planks over water)
BRIEF3: dict[str, dict[str, str]] = {
    "fens": {
        "floor_0": "the common FLOOR: a sunken BOARDWALK — three LONG weathered grey-green PLANKS running left-to-right across the whole tile, INK-black WATER showing in the narrow gaps between them, the planks' top edges lit MIST; NO flagstones, NO mortar grid; seamless left-to-right",
        "floor_1": "a FLOOR variant: the same boardwalk planks with one plank missing — a strip of black water with a MIST moon glint where it was",
        "floor_2": "a FLOOR variant: the same boardwalk planks with a clump of dark moss and a reed tuft growing through a gap at one corner",
        "floor_3": "a FLOOR variant: the same boardwalk planks, older and darker, one plank cracked along its length, a nail head",
        "wall_face_0": "the FRONT FACE of a wall: a PALISADE of upright rotten LOG posts standing in black water, their rounded tops lit MIST (the ledge), hanging moss between them; seamless left-to-right",
        "wall_face_1": "a WALL FACE variant: the same log palisade, one log broken shorter, a rope lashing; seamless left-to-right",
        "wall_top": "the TOP of the palisade seen straight down: the round cut ends of heaped logs and piled mossy stones, lit on their upper-left; seamless in every direction",
        "door": "a DOORWAY in the palisade: a gap between two thick log posts with a lashed plank gate half open onto darkness",
        "stairs_down": "STAIRS GOING DOWN: a short wooden LADDER-STAIR of planks descending into black water and darkness, rope rails",
        "stairs_up": "STAIRS GOING UP: wooden plank steps rising toward the viewer out of the water, the top step lit MIST",
    },
    "burrows": {
        "floor_0": "the common FLOOR: PACKED EARTH filling the tile — a trodden ochre-tinted earth field with a few pebbles and one dark root tendril; NO flagstones, NO mortar lines; calm; seamless in every direction",
        "floor_1": "a FLOOR variant: the same packed earth with a thick dark ROOT crossing it and a scatter of pebbles",
        "floor_2": "a FLOOR variant: the same packed earth with claw scrapes and a small pile of loose soil",
        "floor_3": "a FLOOR variant: the same packed earth, a little darker and damp",
        "wall_face_0": "the FRONT FACE of a wall: a rough EARTH BANK held up by dark TIMBER shoring — two upright posts and a horizontal beam across the top (the beam's top edge is the lit MIST ledge), roots hanging out of the earth between them; seamless left-to-right",
        "wall_face_1": "a WALL FACE variant: the same earth bank and timber shoring, one post leaning, earth spilling from behind a cracked plank; seamless left-to-right",
        "wall_top": "the TOP of the earth wall seen straight down: heaped packed earth with roots and a few buried stones, lit on their upper-left, never black; seamless in every direction",
        "door": "a DOORWAY: a low tunnel mouth dug into the earth bank, framed by a timber lintel and two posts, a lashed plank door ajar onto darkness",
        "stairs_down": "STAIRS GOING DOWN: rough steps cut into packed earth, each edged with a timber board, descending into darkness",
        "stairs_up": "STAIRS GOING UP: timber-edged earth steps rising toward the viewer, the top step lit",
    },
    "deep": {
        "floor_0": "the common FLOOR: unworked wet CAVE ROCK, irregular slabs and fissures (NO straight mortar lines, NO square stones), a few tiny pale MIST crystal specks; seamless in every direction",
        "floor_1": "a FLOOR variant: the same cave rock with a wide fissure and loose stones",
        "floor_2": "a FLOOR variant: the same cave rock with a patch of small pale cave mushrooms",
        "floor_3": "a FLOOR variant: the same cave rock, a shallow black puddle catching a MIST glint",
        "wall_face_0": "the FRONT FACE of a cave wall: jagged unworked rock with rounded bulges and dripstone, the top edge a MIST-lit ridge, no courses, no blocks; seamless left-to-right",
        "wall_face_1": "a CAVE WALL variant: the same jagged rock with a vein of pale crystals",
        "wall_top": "the TOP of cave rock seen straight down: lumpy irregular rock masses lit on their upper-left, never black; seamless in every direction",
        "door": "a DOORWAY: a narrow natural rock CLEFT opening onto darkness, jagged edges",
        "stairs_down": "STAIRS GOING DOWN: rough natural rock ledges descending into black",
        "stairs_up": "STAIRS GOING UP: rough natural rock ledges rising toward the viewer",
    },
}


HEADER3 = """You are painting ENVIRONMENT TILES for the game "Riddle" (a gothic dark-fantasy roguelike on a phone, seen top-down at a
high three-quarter angle). FIRST look at these approved STYLE TARGETS (you may open these images and nothing else in the
repository): /home/g/p/riddle/art/ui/targets/style/watch_warrens.png, /home/g/p/riddle/art/ui/targets/style/watch_fens.png,
/home/g/p/riddle/art/ui/targets/style/boss.png, /home/g/p/riddle/art/ui/targets/style/town.png. Their dungeon (and town) floors and walls are the TARGET LOOK.

Use your built-in image_gen tool, one call per tile below. Do NOT write code beyond a PIL resample / alpha fix. Do NOT touch
anything outside /home/g/p/riddle/art/generated/. Do not leave _inspection_*.png or any other scratch file behind. OVERWRITE
existing files.

=== STYLE (docs/ART_DIRECTION.md, approved) ===
{preamble}
=== THIS PLACE ===
{biome_desc}
Its tinted mid-tones: DUSK here is {dusk}, MOON here is {moon} (use these instead of the plain DUSK/MOON); INK, UMBRA, MIST,
BONE, BLOOD, EMBER, GILT are unchanged.

=== HOW THE TILES ARE USED (critical) ===
Each 1024x1024 image is box-downscaled to a 16x16 tile and drawn tiled edge to edge in a top-down room, beside characters
about 1.5 tiles tall. So:
- Paint each tile as a 16x16 PIXEL GRID blown up 64x: every cell a 64 px square, the forms blocked in those square cells with
  STEPPED edges — but each cell a transparent watercolour WASH (granulation, pigment pooling at the wash edge), not a flat
  digital fill. INK joints / mortar / gaps about one cell (64 px) thick. BOLD shapes, 2-4 large forms, no hairline detail.
- Opaque tiles fill the square edge to edge and TILE SEAMLESSLY with themselves (floors and wall tops in every direction,
  wall faces left-to-right). The four floor variants share one overall value and hue so they mix.
- VALUES: floors are mid-dark and calm (UMBRA and the tinted DUSK, the tinted MOON on the lit side of each stone, MIST only as
  a rare moonlit edge) — the characters must read on top of them; wall tops are built masonry a little lighter than the
  floor (never black void); a wall face's top ledge is the brightest line of the set (a MIST moonlit rim).
- LIGHT: cold moonlight from above (lit top edges, shadowed bottom edges). NO warm light, NO torch glow, NO amber or brown
  cast in any tile (the game adds torch pools itself). Paper grain very faint; halftone only in the deepest joints.
- Keyed props: the object only, centred, filling about 70 % of the square, INK contour, on a TRANSPARENT background (or, if
  the tool gives an opaque image, flat uniform pure MAGENTA #FF00FF edge to edge); no ground shadow, no magenta on the
  object. A flame (lantern, brazier) may be EMBER; everything else stays in the palette.
Save each as a 1024x1024 PNG at the exact path.

=== TILES ===
"""


def write_prompts3(biomes: list[str], split: bool = True) -> list[str]:
    """phase 2: art/prompts/p2_tiles_<biome>_{a,b}.txt — (a) the opaque tiles, (b) the props and the place's own pieces"""
    import sys as _s
    _s.path.insert(0, str(ROOT))
    from make_prompts import style_preamble
    out = []
    for b in biomes:
        dusk, moon = tinted(b)
        head = HEADER3.format(preamble=style_preamble(), biome_desc=BIOMES3[b], dusk=dusk, moon=moon)
        allp = list(pieces(b).items())
        groups = {"a": [x for x in allp if x[1][0] == "env"], "b": [x for x in allp if x[1][0] != "env"]} if split else {"": allp}
        for g, items in groups.items():
            lines = [head]
            for i, (n, (bg, tx, what)) in enumerate(items, 1):
                what = BRIEF3.get(b, {}).get(n, what)
                tall = f" (a TALL piece: {tx} texels — paint it on a 1024x1536 canvas, the object filling the height)" if tx != "16x16" else ""
                lines.append(f"{i}) /home/g/p/riddle/art/generated/envp_{b}_{n}.png — {what}{tall}.")
            name = f"p2_tiles_{b}_{g}" if g else f"p2_tiles_{b}"
            (ROOT / "prompts" / f"{name}.txt").write_text("\n".join(lines) + "\nWhen all tiles are saved, list the final paths. Do nothing else.\n")
            out.append(name)
    return out


def ids(biome: str) -> list[str]:
    return [f"envp_{biome}_{n}" for n in pieces(biome)]


def write_prompts(biomes: list[str], only: set[str] | None = None, tag: str = "") -> None:
    for b in biomes:
        lines = [HEADER.format(biome_desc=BIOMES[b])]
        for i, (n, (bg, tx, what)) in enumerate(((n, v) for n, v in pieces(b).items() if not only or n in only), 1):
            lines.append(f"{i}) /home/g/p/riddle/art/generated/envp_{b}_{n}.png — {what}.")
        (ROOT / "prompts" / f"paint_{b}{tag}.txt").write_text("\n".join(lines) + "\n")
        print(f"art/prompts/paint_{b}{tag}.txt ({len(lines) - 1} tiles)")


def add_manifest() -> None:
    p = ROOT / "manifest.json"
    m = json.loads(p.read_text())
    have = {a["id"] for a in m["assets"]}
    n = 0
    for b, bd in BIOMES.items():
        for name, (bg, tx, what) in pieces(b).items():
            aid = f"envp_{b}_{name}"
            if aid in have:
                continue
            m["assets"].append({"id": aid, "kind": "env_tile" if bg == "env" else "env_prop", "bg": bg, "gen": "1024x1024", "texels": tx,
                                "biome": b, "name": name, "register": "painted", "description": f"{what}. Biome: {bd}"})
            n += 1
    p.write_text(json.dumps(m, indent=2, ensure_ascii=False) + "\n")
    print(f"manifest: +{n} painted entries")


def convert(src: Path, bg: str, texels: tuple[int, int], key_source) -> Image.Image:
    """1024 painting -> a 16x16 tile at full colour: box (area) downscale, a light local-contrast lift (a box average softens the
    forms), <= PAINT_COLOURS colours; keyed props: keyed, cropped to alpha, fitted bottom-centred, a 1-texel dark rim."""
    tw, th = texels
    if bg == "env":
        im = Image.open(src).convert("RGB").resize((tw, th), Image.Resampling.BOX)
        a = np.asarray(im, np.float32)
        m = a.mean(axis=(0, 1), keepdims=True)
        a = np.clip(m + (a - m) * 1.12, 0, 255)
        if src.stem.startswith("envp_") and src.stem.split("_")[1] in CALM and "_floor_" in src.stem:
            # blind round 27 (the Fens "noisy blue stripes"): a floor is background — each plank smoothed along its grain (a
            # wrapped 5-texel run, so it still tiles), the texel speckle and contrast damped toward the tile's mean
            sm = sum(np.roll(a, d, axis=1) for d in (-2, -1, 0, 1, 2)) / 5
            a = a * 0.35 + sm * 0.65
            m = a.mean(axis=(0, 1), keepdims=True)
            a = m + (a - m) * CALM[src.stem.split("_")[1]]
        k = MOON_LIFT.get(src.stem.split("_")[1], 0.0) if src.stem.startswith("envp_") else 0.0
        if src.stem.startswith("envp_") and src.stem.split("_")[1] in CALM and "_floor_" in src.stem:
            k = 0.0   # (no moon sparkle on a calm floor: the walls carry the moon)
        if src.stem.split("_")[1] in EDGE_LIFT and "_floor_" in src.stem:
            k *= 0.3   # (the earth floors stay calm: the ledges carry the moon)
        if k:   # phase 2 (§3: every frame spans INK to MIST): the places painted darkest get their lit edges lifted toward MIST
            lum = (a @ np.array([0.2126, 0.7152, 0.0722], np.float32)) / 255
            if src.stem.split("_")[1] in EDGE_LIFT:   # the earth's lit edges are few and dim: lift the tile's brightest ~15 % instead
                t = min(0.22, float(np.percentile(lum, 85)))
                hi = np.clip((lum - t) / max(0.05, float(lum.max()) - t), 0, 1)[..., None] ** 1.2
            else:
                hi = np.clip((lum - 0.22) / 0.3, 0, 1)[..., None] ** 1.5
            a = a + (np.array([164, 188, 214], np.float32) - a) * k * hi
        im = Image.fromarray(a.astype(np.uint8), "RGB").quantize(PAINT_COLOURS, method=Image.Quantize.MEDIANCUT).convert("RGBA")
        return im
    rgba = key_source(src)
    m = rgba[..., 3] > 16
    # (stray specks and dashes Codex leaves on the key are not the object): the band of rows, then of columns, holding the most pixels
    def band(counts: np.ndarray) -> tuple[int, int]:
        on = counts > 0; best, cur, bs, s0 = (0, len(counts) - 1), 0, -1, 0
        for i, v in enumerate(list(on) + [False]):
            if v and cur == 0: s0 = i
            if v: cur += int(counts[i])
            elif cur: 
                if cur > bs: bs, best = cur, (s0, i - 1)
                cur = 0
        return best
    r0, r1 = band(m.sum(1)); m2 = m[r0:r1 + 1]; c0, c1 = band(m2.sum(0))
    rgba = rgba[r0:r1 + 1, c0:c1 + 1].copy()
    lab = rgba[..., 3] > 16
    rgba[..., 3] = np.where(lab, rgba[..., 3], 0)
    h, w = rgba.shape[:2]
    s = min((tw - 2) / w, (th - 1) / h)
    nw, nh = max(1, round(w * s)), max(1, round(h * s))
    obj = Image.fromarray(rgba.astype(np.uint8), "RGBA").resize((nw, nh), Image.Resampling.BOX)
    o = np.asarray(obj).copy()
    o[..., 3] = np.where(o[..., 3] >= 128, 255, 0)
    out = np.zeros((th, tw, 4), np.uint8)
    x0, y0 = (tw - nw) // 2, th - nh
    out[y0:y0 + nh, x0:x0 + nw] = o
    solid = out[..., 3] > 0
    rim = np.zeros_like(solid)
    pad = np.pad(solid, 1)   # (np.roll wrapped the bottom row's rim onto the top row: a stray dash over every prop)
    rim |= pad[1:-1, 2:] | pad[1:-1, :-2] | pad[2:, 1:-1] | pad[:-2, 1:-1]
    rim &= ~solid
    out[rim] = (14, 10, 8, 255)
    im = Image.fromarray(out, "RGBA")
    q = im.convert("RGB").quantize(PAINT_COLOURS - 1, method=Image.Quantize.MEDIANCUT).convert("RGBA")
    qa = np.asarray(q).copy(); qa[..., 3] = out[..., 3]
    return Image.fromarray(qa, "RGBA")


def convert_all(src_dir: Path, out_dir: Path, key_source) -> list[str]:
    m = json.loads((ROOT / "manifest.json").read_text())
    written = []
    for a in m["assets"]:
        if a.get("register") != "painted":
            continue
        src = src_dir / f"{a['id']}.png"
        if not src.exists():
            continue
        tw, th = (int(v) for v in a["texels"].split("x"))
        tid = f"{a['biome']}_env_{a['name']}"
        convert(src, a["bg"], (tw, th), key_source).save(out_dir / f"{tid}.png")
        written.append(tid)
    (out_dir / "_painted.json").write_text(json.dumps(sorted(written)) + "\n")
    return written


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd == "prompts":
        write_prompts(sys.argv[2:] or list(BIOMES))
    elif cmd == "own":   # python3 art/painted.py own <biome>: a batch of only that biome's own pieces
        write_prompts([sys.argv[2]], set(OWN_PIECES[sys.argv[2]]), "_own")
    elif cmd == "p2":   # python3 art/painted.py p2 [biome ...]: art direction phase 2 prompts
        print(write_prompts3(sys.argv[2:] or list(BIOMES)))
    elif cmd == "manifest":
        add_manifest()
    else:
        print(__doc__)
