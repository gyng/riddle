#!/usr/bin/env python3
"""Art direction phase 2: before/after contact sheets from what ships (web/public/art atlas, web/public/ui), at game size.

    python3 art/p2sheet.py sprites <out.png> <id|kind:hero|kind:monster|kind:boss|kind:summon ...>
    python3 art/p2sheet.py tiles <out.png> [biome ...]      # per biome: every painted piece + a dressed sample room
    python3 art/p2sheet.py ui <out.png> [frames icons portraits foes backdrops deco fx]
    python3 art/p2sheet.py pair <before.png> <after.png> <out.png>   # stack two sheets with labels
    python3 art/p2sheet.py qc <outdir> <ids|kind:…>   # each sprite (with the hero) on three floors → art-qc.py --style

Sprites draw at their runtime device size on a phone at 2x (the master 1:1 — a tile texel is 4 device px, the hero 1.5
tiles), over the Warrens, Fens and Crypt floors, and once more 3x nearest so the grid can be judged.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
PUB = ROOT.parent / "web/public"
BG = (13, 12, 20, 255)
FG = (234, 223, 197, 255)


def atlas() -> tuple[Image.Image, dict]:
    return Image.open(PUB / "art/atlas.png").convert("RGBA"), json.loads((PUB / "art/atlas.json").read_text())


def frame(a: Image.Image, fr: dict, fid: str) -> Image.Image | None:
    f = fr["frames"].get(fid)
    return a.crop((f["x"], f["y"], f["x"] + f["w"], f["y"] + f["h"])) if f else None


def floor(a: Image.Image, fr: dict, biome: str, w: int, h: int) -> Image.Image:
    im = Image.new("RGBA", (w, h), BG)
    for i, y in enumerate(range(0, h, 64)):
        for j, x in enumerate(range(0, w, 64)):
            t = frame(a, fr, f"{biome}_env_floor_{(i * 3 + j * 5) % 4}") or frame(a, fr, f"{biome}_floor")
            if t:
                im.alpha_composite(t.resize((64, 64), Image.Resampling.NEAREST), (x, y))
    return im


def room(a: Image.Image, fr: dict, biome: str) -> Image.Image:
    """a game frame's ground: the biome's dressed sample room (a wall face row with its moonlit ledge and a torch, then floor), 4x
    nearest (a tile texel = 4 device px), cropped 320x224 — the frame a sprite is judged in (§10)"""
    sys.path.insert(0, str(ROOT))
    from make_env import sample_room  # noqa: E402
    t = {k: frame(a, fr, k) for k in fr["frames"] if "env_" in k}
    r = sample_room({k: v for k, v in t.items() if not k.startswith(("env_item", "env_blood"))}, biome)
    r = r.resize((r.width * 4, r.height * 4), Image.Resampling.NEAREST)
    return r.crop((0, 40, 320, 264))


def label(im: Image.Image, xy: tuple[int, int], text: str) -> None:
    ImageDraw.Draw(im).text(xy, text, fill=FG)


def sprites(out: str, ids: list[str]) -> None:
    a, fr = atlas()
    man = json.loads((ROOT / "manifest.json").read_text())["assets"]
    want: list[str] = []
    for x in ids:
        want += [m["id"] for m in man if m.get("kind") == x[5:]] if x.startswith("kind:") else [x]
    want = [w for w in want if w in fr["frames"]]
    cols = 6
    cw, ch = 3 * 140, 170 + 3 * 96 + 30
    rows = (len(want) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * cw, max(1, rows) * ch), BG)
    for i, fid in enumerate(want):
        s = frame(a, fr, fid)
        x0, y0 = (i % cols) * cw, (i // cols) * ch
        for k, b in enumerate(("warrens", "fens", "crypt")):
            g = floor(a, fr, b, 136, 160)
            sc = min(1.0, 128 / max(s.width, s.height)) if s.height > 128 else 1.0
            ss = s.resize((max(1, round(s.width * sc)), max(1, round(s.height * sc))), Image.Resampling.NEAREST) if sc < 1 else s
            g.alpha_composite(ss, ((136 - ss.width) // 2, 150 - ss.height))
            sheet.alpha_composite(g, (x0 + k * 140, y0))
        big = s.resize((s.width * 3, s.height * 3), Image.Resampling.NEAREST)
        if big.width > cw - 8:
            big = big.resize((cw - 8, round(big.height * (cw - 8) / big.width)), Image.Resampling.NEAREST)
        sheet.alpha_composite(big, (x0 + 4, y0 + 166))
        label(sheet, (x0 + 4, y0 + ch - 22), fid)
    sheet.convert("RGB").save(out)
    print(f"{out}: {len(want)} sprites")


def tiles(out: str, biomes: list[str]) -> None:
    sys.path.insert(0, str(ROOT))
    from make_env import sample_room  # noqa: E402
    a, fr = atlas()
    t = {k: frame(a, fr, k) for k in fr["frames"] if "env_" in k}
    biomes = biomes or ["warrens", "burrows", "fens", "crypt", "foundry", "deep", "sanctum"]
    s = 4
    names = sorted({k.split("_env_", 1)[1] for k in t if "_env_" in k and k.split("_env_")[0] in biomes})
    W = max(len(names) * (16 * s + 2), 10 * 16 * s) + 8
    rowh = 32 * s + 6 + 6 * 16 * s + 24
    sheet = Image.new("RGBA", (W, len(biomes) * rowh), BG)
    for bi, b in enumerate(biomes):
        y = bi * rowh
        label(sheet, (4, y + 2), b)
        x = 4
        for n in names:
            im = t.get(f"{b}_env_{n}")
            if im is not None:
                sheet.alpha_composite(im.resize((im.width * s, im.height * s), Image.Resampling.NEAREST), (x, y + 14 + 32 * s - im.height * s))
            x += 16 * s + 2
        room = sample_room(t, b)
        sheet.alpha_composite(room.resize((room.width * s, room.height * s), Image.Resampling.NEAREST), (4, y + 20 + 32 * s))
    sheet.convert("RGB").save(out)
    print(f"{out}: {len(biomes)} biomes")


def ui(out: str, groups: list[str]) -> None:
    groups = groups or ["frames", "icons", "portraits", "foes", "backdrops", "deco", "fx"]
    tiles_: list[tuple[str, Image.Image]] = []
    for g in groups:
        for p in sorted((PUB / "ui" / g).glob("*")):
            if p.suffix in (".png", ".webp"):
                im = Image.open(p).convert("RGBA")
                im.thumbnail((200, 200))
                tiles_.append((f"{g}/{p.stem}", im))
    cols = 10
    sheet = Image.new("RGBA", (cols * 210, ((len(tiles_) + cols - 1) // cols) * 226), (40, 38, 50, 255))
    for i, (n, im) in enumerate(tiles_):
        x, y = (i % cols) * 210, (i // cols) * 226
        sheet.alpha_composite(im, (x + (210 - im.width) // 2, y + (200 - im.height) // 2))
        label(sheet, (x + 4, y + 206), n[:30])
    sheet.convert("RGB").save(out)
    print(f"{out}: {len(tiles_)} ui pieces")


def qc(outdir: str, ids: list[str], biomes: tuple[str, ...] = ("warrens", "fens", "crypt")) -> int:
    """docs/ART_DIRECTION.md §10: every sprite composited as a game frame (it and the hero on the place's floor, 1:1 device px) and run
    through `art-qc.py --style`; returns the style check's exit code"""
    import subprocess
    a, fr = atlas()
    man = json.loads((ROOT / "manifest.json").read_text())["assets"]
    want: list[str] = []
    for x in ids:
        want += [m["id"] for m in man if m.get("kind") == x[5:]] if x.startswith("kind:") else [x]
    Path(outdir).mkdir(parents=True, exist_ok=True)
    hero = frame(a, fr, "hero_fighter")
    paths = []
    for fid in want:
        s = frame(a, fr, fid)
        if s is None:
            continue
        for b in biomes:
            g = room(a, fr, b)
            if fid.startswith("hero"):
                g.alpha_composite(s, ((320 - s.width) // 2, 200 - s.height))
            else:
                ss = s if s.height <= 160 else s.resize((round(s.width * 160 / s.height), 160), Image.Resampling.NEAREST)
                g.alpha_composite(hero, (30, 200 - hero.height))
                g.alpha_composite(ss, (min(320 - ss.width, 150), 200 - ss.height))
            q = Path(outdir) / f"{fid}__{b}.png"
            g.convert("RGB").save(q)
            paths.append(str(q))
    return subprocess.call([sys.executable, str(ROOT / "art-qc.py"), "--style", *paths])


def pair(b: str, a: str, out: str) -> None:
    A, B = Image.open(b).convert("RGB"), Image.open(a).convert("RGB")
    W = max(A.width, B.width)
    im = Image.new("RGB", (W, A.height + B.height + 48), BG[:3])
    d = ImageDraw.Draw(im)
    d.text((6, 4), "BEFORE", fill=(192, 21, 48))
    im.paste(A, (0, 20))
    d.text((6, A.height + 26), "AFTER (phase 2)", fill=(192, 21, 48))
    im.paste(B, (0, A.height + 44))
    im.save(out)
    print(out)


if __name__ == "__main__":
    cmd, rest = sys.argv[1], sys.argv[2:]
    {"sprites": lambda: sprites(rest[0], rest[1:]), "tiles": lambda: tiles(rest[0], rest[1:]), "ui": lambda: ui(rest[0], rest[1:]),
     "pair": lambda: pair(*rest), "qc": lambda: sys.exit(qc(rest[0], rest[1:]))}[cmd]()
