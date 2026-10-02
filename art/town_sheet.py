#!/usr/bin/env python3
"""Cut 30 town v1: the whole town set composed on one phone scene (400 x 800 CSS px at 2x: the bar 48, the scene 560, the console
192), day and night side by side — the art's contact sheet and a reference layout for the renderer (render/town.ts owns the real one).

One world unit (= one env texel) is one CSS px: a terrain cell is 16 units; a keyed sprite is texel_h x SPRITE_SCALE (0.5, the
watch's) units tall (a hero 24, a building 64), its 2x master box-resampled to the 2x canvas. Night is the art as painted (moonlit) a step darker, with EMBER pools from art/town_lights.json and the walkers'
own MIST light; day lifts the ambient toward a cool daylight and drops the pools.

    python3 art/town_sheet.py <out.png> [--labels]      # reads web/public/art/atlas.{png,json} (run pack.py first)
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
PUB = ROOT.parent / "web/public"
S = 2   # device px per CSS px
W, BAR, SCENE, CONSOLE = 400, 48, 560, 192
COLS, ROWS = W // 16, SCENE // 16 + 1

# the plan of the v1 town (cells): the mouth at the top, a 2-wide path down to the fire's plaza, a cross path to the four buildings
PATH = set()
for y in range(6, 34):   # the main street, mouth to the south edge
    PATH |= {(y, 11), (y, 12)}
for x in range(3, 22):   # the upper cross street, in front of the blacksmith and the bank
    PATH |= {(14, x), (15, x)}
for x in range(3, 22):   # the lower cross street, in front of the storehouse and the kennel
    PATH |= {(27, x), (28, x)}
PLAZA = {(y, x) for y in range(17, 23) for x in range(9, 15)}
PATH -= PLAZA

# sprites: (id, anchor x, anchor y) in CSS px = world units (bottom-centre), drawn in y order
PROPS = [
    ("town_mouth_cave", 192, 100), ("town_board", 262, 104), ("town_flag_0", 124, 100),
    ("town_blacksmith_1", 86, 222), ("town_bank_1", 312, 222),
    ("town_storehouse_1", 80, 430), ("town_kennel_1", 316, 432),
    ("town_campfire_0", 192, 318), ("town_tent", 268, 330), ("town_crate", 296, 334),
    ("town_plot", 300, 512), ("town_scaffold", 100, 520),
    ("walk_fighter", 188, 168), ("town_sack_large", 196, 160),
    ("walk_ranger", 196, 400), ("town_chest_glow", 204, 392),
    ("walk_rogue", 128, 250), ("walk_caster", 252, 252), ("town_sack_small", 259, 245),
]
TREES = [(8, 40), (28, 30), (44, 60), (366, 36), (388, 64), (350, 74), (12, 130), (392, 140), (10, 330), (392, 330),
         (16, 490), (384, 490), (40, 556), (360, 556), (230, 560), (156, 556)]
FENCE = [(32, x) for x in range(15, 22)]
LABELS = {"town_mouth_cave": "mouth", "town_board": "board", "town_blacksmith_1": "blacksmith", "town_bank_1": "bank",
          "town_storehouse_1": "storehouse", "town_kennel_1": "kennel", "town_campfire_0": "campfire", "town_tent": "tent",
          "town_crate": "crate", "town_plot": "plot", "town_scaffold": "scaffold"}


def atlas() -> tuple[Image.Image, dict]:
    a = Image.open(PUB / "art/atlas.png").convert("RGBA")
    fr = json.loads((PUB / "art/atlas.json").read_text())["frames"]
    return a, fr


def frame(a: Image.Image, fr: dict, k: str) -> Image.Image | None:
    f = fr.get(k)
    return a.crop((f["x"], f["y"], f["x"] + f["w"], f["y"] + f["h"])) if f else None


def path_tile(y: int, x: int) -> str:
    on = lambda yy, xx: (yy, xx) in PATH or (yy, xx) in PLAZA  # noqa: E731
    g = {c: not on(y + dy, x + dx) for c, (dy, dx) in {"n": (-1, 0), "s": (1, 0), "e": (0, 1), "w": (0, -1)}.items()}
    sides = "".join(c for c in "nesw" if g[c])
    if not sides:
        for c, (dy, dx) in {"ne": (-1, 1), "nw": (-1, -1), "se": (1, 1), "sw": (1, -1)}.items():
            if not on(y + dy, x + dx):
                return f"dirt_inner_{c}"
        return f"dirt_{(x * 7 + y * 3) % 3}"
    if len(sides) == 1:
        return f"dirt_edge_{sides}"
    if sides in ("ne", "es", "sw", "nw"):
        return "dirt_corner_" + {"ne": "ne", "es": "se", "sw": "sw", "nw": "nw"}[sides]
    if sides in ("ns", "ew"):
        return f"dirt_edge_{sides}"
    if len(sides) == 3:
        open_ = [c for c in "nesw" if c not in sides][0]
        return "dirt_end_" + {"n": "s", "s": "n", "e": "w", "w": "e"}[open_]
    return "dirt_isle"


def scene(a: Image.Image, fr: dict, night: bool | None, labels: bool) -> Image.Image:
    """night True / False: the sheet's mock grades; None: the art as painted (the style QC's frame)"""
    im = Image.new("RGBA", (W * S, SCENE * S), (13, 12, 20, 255))
    t = lambda n: frame(a, fr, f"town_env_{n}")  # noqa: E731
    for y in range(ROWS):
        for x in range(COLS):
            if (y, x) in PLAZA:
                n = f"plaza_{(x + y) % 2}"
            elif (y, x) in PATH:
                n = path_tile(y, x)
            else:
                n = f"grass_{[0, 0, 1, 0, 2, 0, 3][(x * 5 + y * 11) % 7]}"
            tile = t(n)
            if tile:
                im.alpha_composite(tile.resize((16 * S, 16 * S), Image.Resampling.NEAREST), (x * 16 * S, y * 16 * S))
    for y, x in FENCE:
        f = t("fence")
        if f:
            im.alpha_composite(f.resize((16 * S, 16 * S), Image.Resampling.NEAREST), (x * 16 * S, y * 16 * S))
    items = [(k, ax, ay) for k, ax, ay in PROPS] + [("town_env_tree_0", ax, ay) for ax, ay in TREES]
    lights = json.loads((ROOT / "town_lights.json").read_text()) if (ROOT / "town_lights.json").exists() else {}
    pools: list[tuple[float, float, float, tuple]] = []
    for k, ax, ay in sorted(items, key=lambda i: i[2]):
        f = frame(a, fr, k)
        if f is None:
            continue
        if k.startswith("town_env_"):
            f = f.resize((f.width * S, f.height * S), Image.Resampling.NEAREST)
            ox, oy = ax * S - f.width // 2, ay * S - f.height
        else:   # a keyed sprite: texel_h x SPRITE_SCALE (0.5) world units, its master 2 px a texel -> master px / 4 units
            f = f.resize((max(1, round(f.width * S / 4)), max(1, round(f.height * S / 4))), Image.Resampling.BOX)
            ox, oy = ax * S - f.width // 2, ay * S - f.height
        im.alpha_composite(f, (ox, oy))
        for dx, dy, st in lights.get(k, []):
            pools.append((ax + dx, ay + dy, st, (232, 146, 58)))
        if k.startswith("walk_") and night:
            pools.append((ax, ay - 20, 0.35, (164, 188, 214)))
    rgb = np.asarray(im, np.float32)[..., :3]
    if night is None:
        out = rgb
    elif night:
        amb = np.array([0.74, 0.78, 0.92], np.float32)
        yy, xx = np.mgrid[0:SCENE * S, 0:W * S].astype(np.float32) / S
        light = np.zeros_like(rgb)
        for px, py, st, col in pools:
            r = 14 + 22 * st
            d2 = ((xx - px) ** 2 + (yy - py) ** 2) / (r * r)
            light += (np.exp(-d2 * 2.2) * 0.9 * st)[..., None] * (np.array(col, np.float32) / 255)
        out = rgb * (amb + light * 1.3)
    else:
        lum = rgb @ np.array([0.2126, 0.7152, 0.0722], np.float32)
        day = rgb * 0.55 + lum[..., None] * 0.45   # the moon's blue eased toward daylight neutrals
        out = day * np.array([1.62, 1.6, 1.48], np.float32) + 14   # (blind read: the first day grade read as dusk)
    img = Image.fromarray(np.clip(out, 0, 255).astype(np.uint8), "RGB")
    if labels:
        d = ImageDraw.Draw(img)
        for k, ax, ay in PROPS:
            if k in LABELS:
                d.text((ax * S - 20, ay * S + 2), LABELS[k], fill=(255, 255, 0))
    return img


def phone(a: Image.Image, fr: dict, night: bool | None, labels: bool) -> Image.Image:
    im = Image.new("RGB", (W * S, (BAR + SCENE + CONSOLE) * S), (13, 12, 20))
    bar = PUB / "ui/frames/bar.png"
    con = PUB / "ui/frames/console.png"
    if bar.exists():
        im.paste(Image.open(bar).convert("RGBA").resize((W * S, BAR * S)), (0, 0), Image.open(bar).convert("RGBA").resize((W * S, BAR * S)))
    im.paste(scene(a, fr, night, labels), (0, BAR * S))
    if con.exists():
        c = Image.open(con).convert("RGBA").resize((W * S, CONSOLE * S))
        im.paste(c, (0, (BAR + SCENE) * S), c)
    gem = PUB / "ui/frames/gem.png"
    if gem.exists():   # the SEND gem sits right in the console (docs/TOWN.md §5); no text
        gm = Image.open(gem).convert("RGBA").resize((120 * S, 120 * S))
        im.paste(gm, ((W - 136) * S, (BAR + SCENE + 36) * S), gm)
    return im


def qc(outdir: str) -> int:
    """docs/ART_DIRECTION.md §10 on the town: every town sprite and walker as a game frame (on the town's grass and path beside a walking
    hero, 2x device px, the art as painted) plus every ground tile class as a patch, through `art-qc.py --style` (tint: town)"""
    import subprocess
    a, fr = atlas()
    man = json.loads((ROOT / "manifest.json").read_text())["assets"]
    ids = [m["id"] for m in man if m.get("kind") == "town" and m["id"] in fr] + [k for k in sorted(fr) if k.startswith("walk_")]
    Path(outdir).mkdir(parents=True, exist_ok=True)
    t = lambda n: frame(a, fr, f"town_env_{n}").resize((32, 32), Image.Resampling.NEAREST)  # noqa: E731

    def ground(path_rows: tuple[int, ...] = (4, 5)) -> Image.Image:
        g = Image.new("RGBA", (320, 200))
        for y in range(7):
            for x in range(10):
                n = ("dirt_edge_n" if y == path_rows[0] else "dirt_edge_s" if y == path_rows[-1] else "dirt_0") if y in path_rows \
                    else f"grass_{(x * 3 + y) % 4}"
                g.alpha_composite(t(n), (x * 32, y * 32))
        return g

    def small(f: Image.Image) -> Image.Image:
        return f.resize((max(1, round(f.width / 2)), max(1, round(f.height / 2))), Image.Resampling.BOX)

    hero = small(frame(a, fr, "walk_fighter"))
    # the frame's context, as the dungeon's QC room has its walls: a building in the back corner (the town's frame always has one)
    ctx = small(frame(a, fr, "town_storehouse_1"))
    ctx = ctx.resize((ctx.width * 3 // 4, ctx.height * 3 // 4), Image.Resampling.BOX)
    paths = []
    for fid in ids:
        g = ground()
        if not any(fid.startswith(f"town_{b}") for b in ("blacksmith", "storehouse", "kennel", "bank", "mouth")):
            g.alpha_composite(ctx, (320 - ctx.width, 120 - ctx.height))
        g.alpha_composite(hero, (30, 170 - hero.height))
        sp = small(frame(a, fr, fid))
        g.alpha_composite(sp, (min(320 - sp.width, 150), max(0, 176 - sp.height)))
        q = Path(outdir) / f"{fid}__town.png"
        g.convert("RGB").save(q)
        paths.append(str(q))
    for cls, names in (("grass", ["grass_0", "grass_1", "grass_2", "grass_3"]), ("path", ["dirt_0", "dirt_1", "dirt_2", "dirt_edge_n", "dirt_corner_se", "dirt_inner_nw"]),
                       ("plaza", ["plaza_0", "plaza_1"])):
        g = ground((2, 3, 4))
        for i in range(10):
            g.alpha_composite(t(names[i % len(names)]), (i * 32, 96))
        g.alpha_composite(hero, (150, 150 - hero.height))
        g.alpha_composite(ctx, (320 - ctx.width, 96 - ctx.height))
        q = Path(outdir) / f"ground_{cls}__town.png"
        g.convert("RGB").save(q)
        paths.append(str(q))
    a2, fr2 = atlas()
    for night in (None,):   # the whole phone too, the scene as painted (the renderer's grades come after: blit.ts)
        q = Path(outdir) / "phone__town.png"
        phone(a2, fr2, night, False).save(q)
        paths.append(str(q))
    return subprocess.call([sys.executable, str(ROOT / "art-qc.py"), "--style", *paths])


def main(argv: list[str]) -> int:
    if argv[:1] == ["qc"]:
        return qc(argv[1] if len(argv) > 1 else str(ROOT.parent / "scratchpad/c30art/qc"))
    out = argv[0] if argv else str(ROOT.parent / "scratchpad/c30art/town_sheet.png")
    labels = "--labels" in argv
    a, fr = atlas()
    day, night = phone(a, fr, False, labels), phone(a, fr, True, labels)
    sheet = Image.new("RGB", (day.width * 2 + 16 * S, day.height), (13, 12, 20))
    sheet.paste(day, (0, 0))
    sheet.paste(night, (day.width + 16 * S, 0))
    Path(out).parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    print(out, sheet.size)
    return 0


if __name__ == "__main__":
    raise SystemExit(main([a for a in sys.argv[1:]]))
