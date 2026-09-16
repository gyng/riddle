#!/usr/bin/env python3
"""Hand-authored 8x8 environment tiles for Riddle (art/tiles/*.png).

Every texel is placed by rule, not by random noise: tiles are ASCII templates
indexed into an 8-colour biome ramp (0 = darkest .. 7 = lightest), with an
ordered 4x4 Bayer dither where a tile needs a mid-value. Walls carry a 1-px
dark rim (index 0) so tiles and sprites share the "everything has a dark
edge" rule (research/art-tech.md section 4 mitigations). Downwell register:
mostly flat with a little dither.

Outputs:
  art/tiles/<biome>_<tile>.png   biome in {warrens, fens, crypt, foundry, deep, sanctum},
                                 tile in {floor, floor_alt, wall, door,
                                          stairs_down, stairs_up, water, chasm}
  art/tiles/gas_0/1.png, fire_0/1.png   RGBA overlays, 2 frames each
  art/tiles/potion|scroll|weapon|armour|gold|bones.png   RGBA item glyphs
  art/tiles/<biome>_bones_0/1.png   RGBA prop, 2 frames (skull + bones on nothing;
                                    frame 1 adds a glint) in the biome ramp
  art/tiles/<biome>_shrine_0/1.png  RGBA prop (CUT5 §4): stele on a plinth, lit top texel;
                                    frame 1 lifts the light one texel
  art/tiles/<biome>_vault.png, <biome>_vault_open.png
                                    RGBA prop (CUT5 §4): iron cage with a keyhole plate;
                                    _open = bars parted, floor visible through the frame
  art/tiles/<biome>_nest_0/1.png    RGBA prop (CUT5 §4): straw/bone mound; frame 0 eyes
                                    closed (slits), frame 1 two dark eye texels open
  art/tiles/_sheet.png           4x review sheet (three rows per biome: tiles, over-floor, sample room)

Run: python3 art/make_tiles.py
"""
from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
OUT = ROOT / "tiles"
T = 8  # texels per tile

# 8-colour ramps, index 0 darkest -> 7 lightest (art/ART.md "Register 2").
PALETTES: dict[str, list[str]] = {
    "warrens": ["#14120d", "#2e2a1c", "#4a4326", "#6b6a2f", "#8c7a3c", "#b09a5a", "#d4c58a", "#efe6c0"],
    "fens": ["#0c1416", "#1a2b2e", "#24443f", "#2f6a5a", "#4d8a72", "#6f9f8a", "#9dbfa8", "#d6e6da"],
    "crypt": ["#0b0a14", "#1c1a30", "#33304f", "#4f4d6d", "#77738c", "#a39fae", "#d3cfc9", "#f1ede0"],
    # Cut 3 (docs/CUT3.md). Ramps stay monotone in luminance so the tint pass can index by value.
    # foundry: rust / ember / iron - warm dark oranges, two iron greys (4, 6), one hot yellow (7).
    "foundry": ["#120b08", "#2c1a12", "#4e2a18", "#8a3f1c", "#5b5a5e", "#c2622a", "#9a9598", "#f4c040"],
    # deep: black / ink-blue / bone - near-black floor (2), blue-black walls (3, 4), bone highlights (6, 7).
    "deep": ["#030306", "#0c0e1a", "#181b30", "#1e2340", "#2c3560", "#5a5f78", "#b8b09a", "#ece4cc"],
    # sanctum: white / gold / slate - slate shadows (0-3), one gold (4), pale stone (5-7).
    "sanctum": ["#1a1c24", "#3c404e", "#666a78", "#9a9aa0", "#c9a84a", "#d9d4c6", "#ebe6d8", "#fbf7ee"],
}

# One-shot palette flash (CUT2 §7, boss sighted): the viewer swaps the biome ramp for this
# ramp for a frame or two. Same shape as a biome ramp (index 0 darkest .. 7 lightest) so the
# tint pass needs no special case; crimson-to-white-hot so every biome flashes "alarm".
# Shipped in atlas.json meta.palettes["boss_flash"]; no tiles are authored in it.
BOSS_FLASH = ["#1a0608", "#4a0d12", "#8c1a1e", "#c8321a", "#f08a1e", "#ffd85a", "#fff3b0", "#ffffff"]

BAYER4 = [
    [0, 8, 2, 10],
    [12, 4, 14, 6],
    [3, 11, 1, 9],
    [15, 7, 13, 5],
]


def hex_rgb(h: str) -> tuple[int, int, int]:
    h = h.lstrip("#")
    return int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)


# --- ASCII templates ------------------------------------------------------
# Digits index the biome ramp. Letters are ordered-dither cells resolved per texel from
# DITHER: (idx if bayer < threshold, idx otherwise, threshold out of 16).
DITHER: dict[str, tuple[int, int, int]] = {
    "a": (4, 3, 8),   # 50% between 4 and 3 (water body)
    "b": (1, 0, 4),   # 25% of 1 over 0 (chasm lip)
    "c": (1, 2, 4),   # 25% of 1 over 2 (floor grit)
    "d": (3, 1, 4),   # 25% of 3 over 1 (deep chasm lip: a little ink-blue over blue-black)
    "e": (3, 2, 8),   # 50% between 3 and 2 (sanctum pool: slate greys)
    "f": (5, 3, 8),   # 50% between 5 and 3 (foundry molten channel: two oranges)
}
# A template is either a list of rows (every biome) or a dict {biome: rows, "*": default rows}.


def tpl(spec, biome: str) -> list[str]:
    if isinstance(spec, dict):
        return spec[biome] if biome in spec else spec["*"]
    return spec


FLOOR = {
    "warrens": [
        "22222222",
        "22122222",
        "22222222",
        "22222212",
        "22222222",
        "21222322",
        "22222222",
        "22221222",
    ],
    "fens": [
        "22222222",
        "22232222",
        "22222222",
        "21222222",
        "22222232",
        "22222222",
        "22212222",
        "22222222",
    ],
    "crypt": [  # dressed flagstones: faint mortar line
        "22222122",
        "22222122",
        "22222122",
        "11111111",
        "22122222",
        "22122222",
        "22122222",
        "22122222",
    ],
    "foundry": [  # cooled slag: dark rust field, an orange fleck and iron grit
        "22222222",
        "22222232",
        "22222222",
        "21222222",
        "22222221",
        "22232222",
        "22222222",
        "22221222",
    ],
    "deep": [  # near-black cave floor; the fewest marks of any biome (vision 4)
        "22222222",
        "22222222",
        "22212222",
        "22222232",
        "22222222",
        "22222212",
        "22222222",
        "21222222",
    ],
    "sanctum": [  # pale dressed flagstones: light mortar (3) on pale stone (5), a lit texel per slab
        "55555355",
        "56555355",
        "55555365",
        "33333333",
        "55355555",
        "55356555",
        "55355555",
        "55355555",
    ],
}

FLOOR_ALT = {
    "warrens": [  # a bone shard and a crack
        "22222222",
        "22222252",
        "22222522",
        "21222222",
        "12222222",
        "22222222",
        "22221222",
        "22222222",
    ],
    "fens": [  # moss tuft (idx 3) and a wet spot (idx 1)
        "22222222",
        "23322222",
        "23332222",
        "22222222",
        "22222212",
        "22222112",
        "22222222",
        "22222222",
    ],
    "crypt": [  # cracked flagstone
        "22222122",
        "22212122",
        "22212122",
        "11111111",
        "22122222",
        "22122212",
        "22122122",
        "22122222",
    ],
    "foundry": [  # a diagonal crack with one ember (5) still hot in it
        "22222222",
        "22222212",
        "22222122",
        "22221222",
        "22212222",
        "22522222",
        "22222222",
        "22222222",
    ],
    "deep": [  # a wet pool (3) and one bone chip (6)
        "22222222",
        "22222222",
        "22233222",
        "22333222",
        "22222222",
        "22222262",
        "22222222",
        "22222222",
    ],
    "sanctum": [  # cracked slab with a gold inlay fleck (4)
        "55555355",
        "55535355",
        "55535355",
        "33333333",
        "55355555",
        "55355545",
        "55355355",
        "55355555",
    ],
}

WALL = {
    "warrens": [  # rough packed-earth and rock, full 1-px rim
        "00000000",
        "04433310",
        "03333310",
        "01111110",
        "03144330",
        "03133330",
        "01111110",
        "00000000",
    ],
    "fens": [  # slick slab stone with a slime drip
        "00000000",
        "04333310",
        "03333310",
        "03331110",
        "01113310",
        "03433310",
        "03331110",
        "00000000",
    ],
    "crypt": [  # dressed masonry, tight mortar
        "00000000",
        "04441440",
        "03331330",
        "01111110",
        "04413340",
        "03313330",
        "01111110",
        "00000000",
    ],
    "foundry": [  # riveted iron plates (4) with light rivets (6), a dark seam, one rust patch (3)
        "00000000",
        "06444460",
        "04444440",
        "01111110",
        "04434440",
        "06444460",
        "01111110",
        "00000000",
    ],
    "deep": [  # blue-black cave rock, vertical cracks, a bone-lit corner upper-left
        "00000000",
        "04431330",
        "03431330",
        "03411330",
        "01133130",
        "03333130",
        "01331110",
        "00000000",
    ],
    "sanctum": [  # white marble courses with a gold trim course (4) at mid-height
        "00000000",
        "07666510",
        "06666510",
        "01111110",
        "04444440",
        "01111110",
        "06651660",
        "00000000",
    ],
}

DOOR = {
    "*": [  # planked door in a dark frame, knob at idx 6
        "00000000",
        "05505500",
        "05505500",
        "05505500",
        "05505560",
        "04404400",
        "04404400",
        "00000000",
    ],
    "foundry": [  # iron door: grey plates (4), dark seams, light knob (6)
        "00000000",
        "04414410",
        "04414410",
        "04414410",
        "04414460",
        "01111110",
        "04414410",
        "00000000",
    ],
}

STAIRS_DOWN = {
    "*": [  # a pit of steps descending to the lower-right, lit step on top
        "22222222",
        "20000002",
        "20444402",
        "20033302",
        "20022202",
        "20011102",
        "20000002",
        "22222222",
    ],
    "deep": [  # the top step is bone-lit (6) so the stairs are findable on a near-black floor
        "22222222",
        "20000002",
        "20666602",
        "20444402",
        "20333302",
        "20111102",
        "20000002",
        "22222222",
    ],
}

STAIRS_UP = [  # steps rising toward the top; risers as dark lines
    "22222222",
    "20000002",
    "20666602",
    "20000002",
    "20555502",
    "20000002",
    "20444402",
    "22222222",
]

WATER = {
    "*": [  # dithered body with two wave dashes (idx 6)
        "aaaaaaaa",
        "aa66aaaa",
        "aaaaaaaa",
        "aaaaaaaa",
        "aaaaaa66",
        "aaaaaaaa",
        "a66aaaaa",
        "aaaaaaaa",
    ],
    "foundry": [  # molten channel: two oranges dithered, hot-yellow (7) dashes
        "ffffffff",
        "ff77ffff",
        "ffffffff",
        "ffffffff",
        "ffffff77",
        "ffffffff",
        "f77fffff",
        "ffffffff",
    ],
    "sanctum": [  # still reflecting pool: slate greys dithered, white (7) dashes
        "eeeeeeee",
        "ee77eeee",
        "eeeeeeee",
        "eeeeeeee",
        "eeeeee77",
        "eeeeeeee",
        "e77eeeee",
        "eeeeeeee",
    ],
}

CHASM = {
    "*": [  # near-black with a dithered lip at the top (light falls in from above)
        "bbbbbbbb",
        "bbbbbbbb",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
    ],
    "deep": [  # no light falls in; the lip is a dusting of ink-blue so it separates from the floor
        "dddddddd",
        "dddddddd",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
        "00000000",
    ],
}

# Overlays and item glyphs use their own small palettes ('.' = transparent).
GAS_PAL = {"k": "#2b3a10", "d": "#6f9a2a", "m": "#9ac43a", "l": "#d8e87a"}
GAS = [
    [
        "........",
        "..kkk...",
        ".kmmlk..",
        ".kmmmk.k",
        "..kkkkdk",
        "....kmlk",
        ".....kk.",
        "........",
    ],
    [
        "........",
        ".....kk.",
        "....kmlk",
        ".kk.kmmk",
        "kmlk.kk.",
        "kmmmk...",
        ".kkk....",
        "........",
    ],
]
FIRE_PAL = {"k": "#3a0c05", "r": "#c8321a", "o": "#f08a1e", "y": "#ffd85a", "w": "#fff3b0"}
FIRE = [
    [
        "....k...",
        "...kok..",
        ".k.kyok.",
        ".kokyok.",
        "kroyywok",
        "krooyork",
        "krrooork",
        ".kkkkkk.",
    ],
    [
        "..k.....",
        ".kok..k.",
        ".koyk.k.",
        "kroyokok",
        "kroywork",
        "krooyork",
        "krrooork",
        ".kkkkkk.",
    ],
]

ITEM_PALS = {
    "potion": {"k": "#1a1410", "r": "#c8321a", "p": "#e86a5a", "w": "#fff3e0", "c": "#8c6a3c"},
    "scroll": {"k": "#1a1410", "p": "#efe6c0", "s": "#c9b98a", "r": "#c8321a"},
    "weapon": {"k": "#1a1410", "s": "#b9c0c8", "w": "#f1f3f5", "h": "#6b4a2a", "g": "#d9a441"},
    "armour": {"k": "#1a1410", "s": "#8e97a3", "l": "#c8cfd6", "d": "#4f5763", "r": "#c8321a"},
    "gold": {"k": "#3a2a08", "g": "#d9a441", "l": "#ffe08a", "d": "#a8781e"},
}
ITEMS = {
    "potion": [
        "...kk...",
        "...cc...",
        "...kk...",
        "..kppk..",
        ".kprrpk.",
        ".kwrrrk.",
        ".krrrrk.",
        "..kkkk..",
    ],
    "scroll": [
        "........",
        ".kkkkkk.",
        "kpsppppk",
        "kpsprppk",
        "kpsprppk",
        "kpsppppk",
        ".kkkkkk.",
        "........",
    ],
    "weapon": [
        "......kk",
        ".....kwk",
        "....kwsk",
        "...kwsk.",
        "kk.ksk..",
        "kgkgk...",
        ".khkk...",
        "kkk.....",
    ],
    "armour": [
        "........",
        ".kk..kk.",
        "kslkklsk",
        "kslsslsk",
        "kdssssdk",
        ".kssssk.",
        ".kdrrdk.",
        "..kkkk..",
    ],
    "gold": [
        "........",
        "........",
        "...kkk..",
        "..klggk.",
        ".kkgddkk",
        "klggkggk",
        "kgddkddk",
        ".kkk.kk.",
    ],
}

# Bones prop (CUT2): a dead heir's pile. Drawn ON NOTHING ('.' = transparent) so it sits over
# any floor; ramp-indexed so each biome gets its own bone colour. Skull upper-left (rounded
# crown, two socket texels, tooth row), one long bone falling diagonally to the lower-left,
# a chip at the bottom-right. Frame 1 = same pile + one lit sparkle texel above the bone and
# the near socket catching light (the "slight glint" that says "something is here").
BONES = [
    [
        ".000....",
        "0777....",  # crown, lit from upper-left
        "0171..00",
        "0666..07",
        ".000.070",
        "...0070.",
        "..0600..",
        "..00.00.",
    ],
    [
        ".000....",
        "0777.7..",  # sparkle texel
        "0131..00",  # near socket catches the light
        "0666..07",
        ".000.070",
        "...0070.",
        "..0600..",
        "..00.00.",
    ],
]
BONES_PAL = {"k": "#1a1410", "w": "#f1ede0", "b": "#c9c0a8", "e": "#3a3020", "g": "#7a7060"}
BONES_ITEM = [  # generic item glyph (same drawing as frame 0, own palette)
    ".kkk....",
    "kwwwk...",
    "kekek.kk",
    "kbbbk.kw",
    ".kkk.kwk",
    "...kkwk.",
    "..kbkk..",
    "..kk.kk.",
]

# --- CUT5 §4 situation props --------------------------------------------
# All three are drawn ON NOTHING ('.' = transparent), ramp-indexed, and carry a ramp-0 rim
# wherever the silhouette meets the floor (the "dark edge" rule). Each frame is a template
# per tpl(): a row list for every biome, or {biome: rows, "*": default}. Every frame <= 8
# colours by construction.

# Shrine: a small stele (5 face, 6 lit glyph, 3 carved glyph) on a wider plinth (4), with one
# lit texel (7) floating above the stele = the altar light. Frame 1 lifts the light one texel
# (the 2 Hz flip reads as a flicker). Sanctum: pale stele with the one gold glyph on a slate
# plinth. Foundry: iron stele (4) with an ember glyph (5) on a rust plinth (3).
SHRINE = [
    {
        "*": [
            "........",
            "....7...",  # altar light
            "..00000.",
            "..05650.",  # stele face, lit glyph texel
            "..05350.",  # carved glyph
            "..05550.",
            ".0444440",  # plinth
            ".0000000",
        ],
        "sanctum": [
            "........",
            "....7...",
            "..00000.",
            "..06460.",  # gold glyph on pale stone
            "..06360.",
            "..06660.",
            ".0333330",  # slate plinth
            ".0000000",
        ],
        "foundry": [
            "........",
            "....7...",
            "..00000.",
            "..04640.",  # iron face, grey-lit glyph
            "..04540.",  # ember glyph
            "..04440.",
            ".0333330",  # rust plinth
            ".0000000",
        ],
    },
    {
        "*": [
            "....7...",  # light one texel higher
            "........",
            "..00000.",
            "..05650.",
            "..05350.",
            "..05550.",
            ".0444440",
            ".0000000",
        ],
        "sanctum": [
            "....7...",
            "........",
            "..00000.",
            "..06460.",
            "..06360.",
            "..06660.",
            ".0333330",
            ".0000000",
        ],
        "foundry": [
            "....7...",
            "........",
            "..00000.",
            "..04640.",
            "..04540.",
            "..04440.",
            ".0333330",
            ".0000000",
        ],
    },
]

# Vault: a front-on iron cage. Posts and rails are ramp-0 (the silhouette's dark edge), the
# top rail catches light (4 with 5 ends), two lit iron bars (4) stand inside, and a 2x2 lock
# plate (6, one dark keyhole texel) hangs between them under a dark cross rail. The floor
# shows through the four gaps (enclosed openings, so the bars need no rim of their own).
# `vault_open`: the bars have swung flush against the posts, the interior is open floor.
VAULT = [
    "00000000",
    "05444450",  # top rail
    "0.4..4.0",  # iron bars, floor through the gaps
    "0.4004.0",  # cross rail = plate rim
    "0.4664.0",  # lock plate
    "0.4064.0",  # keyhole
    "0.4004.0",
    "00000000",
]
VAULT_OPEN = [
    "00000000",
    "05444450",
    "04....40",  # bars folded onto the posts
    "04....40",
    "04....40",
    "04....40",
    "04....40",
    "00000000",
]

# Nest: a low mound of straw (4, lit 5, shadowed 3), six texels tall so it reads "pile" not
# "head", with two bone chips (7, 6) on the left and a lit patch upper-right. Frame 0: eyes
# closed (two 3 slits in the lit patch). Frame 1: eyes open (two ramp-0 texels). Sanctum: a
# pale bone-and-cloth mound (6/7, slate 3) with a gold fleck (4) for the bone chip, so it
# does not read as the floor.
NEST = [
    {
        "*": [
            "........",
            "........",
            "..00000.",
            ".0455540",
            "04535340",  # slits: eyes closed
            "04744450",  # bone chip
            "03463330",  # bone chip, shadowed straw
            ".000000.",
        ],
        "sanctum": [
            "........",
            "........",
            "..00000.",
            ".0677760",
            "06737360",
            "06466670",
            "03643330",
            ".000000.",
        ],
    },
    {
        "*": [
            "........",
            "........",
            "..00000.",
            ".0455540",
            "04505040",  # eyes open
            "04744450",
            "03463330",
            ".000000.",
        ],
        "sanctum": [
            "........",
            "........",
            "..00000.",
            ".0677760",
            "06707060",
            "06466670",
            "03643330",
            ".000000.",
        ],
    },
]


def resolve(ch: str, x: int, y: int) -> int | None:
    """Map a template character to a ramp index (None = transparent)."""
    if ch.isdigit():
        return int(ch)
    lo, hi, thresh = DITHER[ch]
    return lo if BAYER4[y % 4][x % 4] < thresh else hi


def ramp_tile(rows: list[str], ramp: list[str]) -> Image.Image:
    im = Image.new("RGBA", (T, T))
    px = im.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            idx = resolve(ch, x, y)
            px[x, y] = (*hex_rgb(ramp[idx]), 255)
    return im


def ramp_keyed_tile(rows: list[str], ramp: list[str]) -> Image.Image:
    """Ramp-indexed like ramp_tile, but '.' is transparent (props drawn on nothing)."""
    im = Image.new("RGBA", (T, T), (0, 0, 0, 0))
    px = im.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != ".":
                px[x, y] = (*hex_rgb(ramp[int(ch)]), 255)
    return im


def keyed_tile(rows: list[str], pal: dict[str, str]) -> Image.Image:
    im = Image.new("RGBA", (T, T), (0, 0, 0, 0))
    px = im.load()
    for y, row in enumerate(rows):
        for x, ch in enumerate(row):
            if ch != ".":
                px[x, y] = (*hex_rgb(pal[ch]), 255)
    return im


def count_colours(im: Image.Image) -> int:
    return len({p for p in im.convert("RGBA").getcolors(256) or [] if p[1][3] > 0})


def build() -> dict[str, Image.Image]:
    tiles: dict[str, Image.Image] = {}
    for biome, ramp in PALETTES.items():
        spec = {
            "floor": FLOOR,
            "floor_alt": FLOOR_ALT,
            "wall": WALL,
            "door": DOOR,
            "stairs_down": STAIRS_DOWN,
            "stairs_up": STAIRS_UP,
            "water": WATER,
            "chasm": CHASM,
        }
        for name, template in spec.items():
            tiles[f"{biome}_{name}"] = ramp_tile(tpl(template, biome), ramp)
        for i, rows in enumerate(BONES):
            tiles[f"{biome}_bones_{i}"] = ramp_keyed_tile(rows, ramp)
        for i, frame in enumerate(SHRINE):
            tiles[f"{biome}_shrine_{i}"] = ramp_keyed_tile(tpl(frame, biome), ramp)
        tiles[f"{biome}_vault"] = ramp_keyed_tile(VAULT, ramp)
        tiles[f"{biome}_vault_open"] = ramp_keyed_tile(VAULT_OPEN, ramp)
        for i, frame in enumerate(NEST):
            tiles[f"{biome}_nest_{i}"] = ramp_keyed_tile(tpl(frame, biome), ramp)
    for i, rows in enumerate(GAS):
        tiles[f"gas_{i}"] = keyed_tile(rows, GAS_PAL)
    for i, rows in enumerate(FIRE):
        tiles[f"fire_{i}"] = keyed_tile(rows, FIRE_PAL)
    for name, rows in ITEMS.items():
        tiles[name] = keyed_tile(rows, ITEM_PALS[name])
    tiles["bones"] = keyed_tile(BONES_ITEM, BONES_PAL)
    return tiles


def sheet(tiles: dict[str, Image.Image], scale: int = 4) -> Image.Image:
    """4x review sheet: per biome, a row of tiles, a row of overlays/items/props over the floor,
    and a 7x4 sample room (walls around every special floor + bones, shrine, vault open/closed,
    nest) to judge the rim rule."""
    order = ["floor", "floor_alt", "wall", "door", "stairs_down", "stairs_up", "water", "chasm"]
    extras = ["gas_0", "gas_1", "fire_0", "fire_1", "potion", "scroll", "weapon", "armour", "gold",
              "bones", "{b}_bones_0", "{b}_bones_1",
              "{b}_shrine_0", "{b}_shrine_1", "{b}_vault", "{b}_vault_open", "{b}_nest_0", "{b}_nest_1"]
    room = [
        ["wall", "wall", "wall", "wall", "wall", "wall", "wall"],
        ["wall", "floor", "floor_alt", "stairs_up", "floor+shrine_0", "floor", "wall"],
        ["wall", "water", "floor+bones_0", "floor", "floor+vault", "floor_alt+nest_1", "door"],
        ["wall", "chasm", "stairs_down", "floor_alt", "floor+vault_open", "floor+nest_0", "wall"],
    ]
    cell = T * scale
    pad = 6
    label_h = 12
    cols = max(len(order), len(extras))
    w = pad + cols * (cell + pad) + 80
    per_biome = 2 * (cell + label_h + pad) + (len(room) * cell + label_h + pad)
    h = pad + len(PALETTES) * per_biome + pad
    im = Image.new("RGBA", (w, h), (24, 22, 20, 255))
    d = ImageDraw.Draw(im)

    def tile_at(biome: str, name: str) -> Image.Image:
        if "+" in name:  # "floor+bones_0": prop composited over that floor
            base, prop = name.split("+")
            comp = tiles[f"{biome}_{base}"].copy()
            comp.alpha_composite(tiles[f"{biome}_{prop}"])
            return comp
        return tiles[f"{biome}_{name}"]

    y = pad
    for biome in PALETTES:
        d.text((pad, y), biome, fill=(230, 220, 200, 255))
        for i, name in enumerate(order):
            x = 80 + pad + i * (cell + pad)
            im.paste(tiles[f"{biome}_{name}"].resize((cell, cell), Image.Resampling.NEAREST), (x, y + label_h))
            d.text((x, y), name[:10], fill=(180, 170, 150, 255))
        y += cell + label_h + pad
        # overlays and items composited over this biome's floor
        d.text((pad, y), "over floor", fill=(180, 170, 150, 255))
        floor = tiles[f"{biome}_floor"]
        for i, name in enumerate(extras):
            name = name.format(b=biome)
            x = 80 + pad + i * (cell + pad)
            comp = floor.copy()
            comp.alpha_composite(tiles[name])
            im.paste(comp.resize((cell, cell), Image.Resampling.NEAREST), (x, y + label_h))
            d.text((x, y), name.replace(f"{biome}_", "")[:10], fill=(180, 170, 150, 255))
        y += cell + label_h + pad
        # sample room at 4x, walls around floors, to judge the rim rule and the special floors
        d.text((pad, y), "room", fill=(180, 170, 150, 255))
        x0 = 80 + pad
        for ry, row in enumerate(room):
            for rx, name in enumerate(row):
                im.paste(tile_at(biome, name).resize((cell, cell), Image.Resampling.NEAREST),
                         (x0 + rx * cell, y + label_h + ry * cell))
        y += len(room) * cell + label_h + pad
    return im


def main() -> int:
    OUT.mkdir(parents=True, exist_ok=True)
    tiles = build()
    bad = []
    for name, im in tiles.items():
        im.save(OUT / f"{name}.png")
        n = count_colours(im)
        if n > 8:
            bad.append((name, n))
    sheet(tiles).save(OUT / "_sheet.png")
    print(f"wrote {len(tiles)} tiles + _sheet.png -> {OUT}")
    if bad:
        print("FAIL: >8 colours:", bad)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
