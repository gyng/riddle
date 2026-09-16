#!/usr/bin/env python3
"""Hand-authored 8x8 environment tiles for Riddle (art/tiles/*.png).

Every texel is placed by rule, not by random noise: tiles are ASCII templates
indexed into an 8-colour biome ramp (0 = darkest .. 7 = lightest), with an
ordered 4x4 Bayer dither where a tile needs a mid-value. Walls carry a 1-px
dark rim (index 0) so tiles and sprites share the "everything has a dark
edge" rule (research/art-tech.md section 4 mitigations). Downwell register:
mostly flat with a little dither.

Outputs:
  art/tiles/<biome>_<tile>.png   biome in {warrens, fens, crypt},
                                 tile in {floor, floor_alt, wall, door,
                                          stairs_down, stairs_up, water, chasm}
  art/tiles/gas_0/1.png, fire_0/1.png   RGBA overlays, 2 frames each
  art/tiles/potion|scroll|weapon|armour|gold|bones.png   RGBA item glyphs
  art/tiles/<biome>_bones_0/1.png   RGBA prop, 2 frames (skull + bones on nothing;
                                    frame 1 adds a glint) in the biome ramp
  art/tiles/_sheet.png           4x review sheet

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
# Digits index the biome ramp. Letters are dither cells resolved per texel:
#   'a' = 50% Bayer between idx 3 and 4 (water body)
#   'b' = 25% Bayer between idx 0 and 1 (chasm lip)
#   'c' = 25% Bayer between idx 2 and 1 (floor grit)
# Templates are per tile, with optional per-biome overrides for character.

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
}

DOOR = [  # planked door in a dark frame, knob at idx 6
    "00000000",
    "05505500",
    "05505500",
    "05505500",
    "05505560",
    "04404400",
    "04404400",
    "00000000",
]

STAIRS_DOWN = [  # a pit of steps descending to the lower-right, lit step on top
    "22222222",
    "20000002",
    "20444402",
    "20033302",
    "20022202",
    "20011102",
    "20000002",
    "22222222",
]

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

WATER = [  # dithered body with two wave dashes (idx 6)
    "aaaaaaaa",
    "aa66aaaa",
    "aaaaaaaa",
    "aaaaaaaa",
    "aaaaaa66",
    "aaaaaaaa",
    "a66aaaaa",
    "aaaaaaaa",
]

CHASM = [  # near-black with a dithered lip at the top (light falls in from above)
    "bbbbbbbb",
    "bbbbbbbb",
    "00000000",
    "00000000",
    "00000000",
    "00000000",
    "00000000",
    "00000000",
]

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


def resolve(ch: str, x: int, y: int) -> int | None:
    """Map a template character to a ramp index (None = transparent)."""
    if ch.isdigit():
        return int(ch)
    b = BAYER4[y % 4][x % 4]
    if ch == "a":
        return 4 if b < 8 else 3
    if ch == "b":
        return 1 if b < 4 else 0
    if ch == "c":
        return 1 if b < 4 else 2
    raise ValueError(ch)


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
            "floor": FLOOR[biome],
            "floor_alt": FLOOR_ALT[biome],
            "wall": WALL[biome],
            "door": DOOR,
            "stairs_down": STAIRS_DOWN,
            "stairs_up": STAIRS_UP,
            "water": WATER,
            "chasm": CHASM,
        }
        for name, rows in spec.items():
            tiles[f"{biome}_{name}"] = ramp_tile(rows, ramp)
        for i, rows in enumerate(BONES):
            tiles[f"{biome}_bones_{i}"] = ramp_keyed_tile(rows, ramp)
    for i, rows in enumerate(GAS):
        tiles[f"gas_{i}"] = keyed_tile(rows, GAS_PAL)
    for i, rows in enumerate(FIRE):
        tiles[f"fire_{i}"] = keyed_tile(rows, FIRE_PAL)
    for name, rows in ITEMS.items():
        tiles[name] = keyed_tile(rows, ITEM_PALS[name])
    tiles["bones"] = keyed_tile(BONES_ITEM, BONES_PAL)
    return tiles


def sheet(tiles: dict[str, Image.Image], scale: int = 4) -> Image.Image:
    """4x review sheet: one row per biome, then overlays + items over each floor."""
    order = ["floor", "floor_alt", "wall", "door", "stairs_down", "stairs_up", "water", "chasm"]
    extras = ["gas_0", "gas_1", "fire_0", "fire_1", "potion", "scroll", "weapon", "armour", "gold",
              "bones", "{b}_bones_0", "{b}_bones_1"]
    cell = T * scale
    pad = 6
    label_h = 12
    cols = max(len(order), len(extras))
    w = pad + cols * (cell + pad) + 80
    rows = len(PALETTES) * 2
    h = pad + rows * (cell + label_h + pad) + label_h + 4 * cell + pad
    im = Image.new("RGBA", (w, h), (24, 22, 20, 255))
    d = ImageDraw.Draw(im)
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
    # a 3x3 sample room per biome at 4x, walls around floors, to judge the rim rule
    d.text((pad, y), "room", fill=(230, 220, 200, 255))
    for bi, biome in enumerate(PALETTES):
        x0 = 80 + pad + bi * (3 * cell + pad * 2)
        room = [["wall"] * 4, ["wall", "floor", "floor_alt", "wall"], ["wall", "water", "stairs_down", "door"], ["wall"] * 4]
        for ry, row in enumerate(room):
            for rx, name in enumerate(row):
                im.paste(tiles[f"{biome}_{name}"].resize((cell, cell), Image.Resampling.NEAREST),
                         (x0 + rx * cell, y + label_h + ry * cell))
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
