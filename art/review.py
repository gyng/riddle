#!/usr/bin/env python3
"""Per-sprite QC for keyed sources (tacticalswap README round-3 gates).

For each keyed manifest id present in art/generated/:
  - corner exactness: all four corners within 30 of #0000FF, or transparent
  - flood-fill from the border over near-key (dist <= 130) to find ENCLOSED key
    regions (pure key inside genuine silhouette openings is fine; near-key
    inside solid subject is a defect - judge visually)
  - band census: pixels in the 30..130 distance band inside the subject
  - short edge >= 1024
Then writes art/qc/review.png: every sprite packed through pack.build_sprite,
shown at 1x master and at runtime size (nearest /2) over each biome's floor
and wall tiles, so the 48 px read can be judged with the eye.

Run: python3 art/review.py [id ...]
"""
from __future__ import annotations

import json
import sys
from collections import deque
from pathlib import Path

import numpy as np
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
from make_tiles import PALETTES  # noqa: E402  biome order for the composite panels
from pack import build_sprite  # noqa: E402

SRC = ROOT / "generated"
TILES = ROOT / "tiles"
QC = ROOT / "qc"


def key_dist(rgb: np.ndarray) -> np.ndarray:
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    return np.sqrt(r * r + g * g + (b - 255.0) ** 2)


def flood_border(cand: np.ndarray) -> np.ndarray:
    h, w = cand.shape
    seen = np.zeros_like(cand)
    q: deque[tuple[int, int]] = deque()
    for x in range(w):
        for y in (0, h - 1):
            if cand[y, x] and not seen[y, x]:
                seen[y, x] = True
                q.append((y, x))
    for y in range(h):
        for x in (0, w - 1):
            if cand[y, x] and not seen[y, x]:
                seen[y, x] = True
                q.append((y, x))
    while q:
        y, x = q.popleft()
        for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
            if 0 <= ny < h and 0 <= nx < w and cand[ny, nx] and not seen[ny, nx]:
                seen[ny, nx] = True
                q.append((ny, nx))
    return seen


def census(path: Path) -> dict:
    im = Image.open(path).convert("RGBA")
    a = np.asarray(im)
    rgb = a[..., :3].astype(np.float32)
    alpha = a[..., 3]
    h, w = alpha.shape
    corners = [(0, 0), (0, w - 1), (h - 1, 0), (h - 1, w - 1)]
    authored = min(alpha[y, x] for y, x in corners) < 242
    out = {"size": (w, h), "authored_alpha": bool(authored), "short_edge_ok": min(w, h) >= 1024}
    if authored:
        vis = alpha > 8
        out["corners_ok"] = True
        out["enclosed_pure"] = 0
        out["enclosed_near"] = 0
        out["band"] = int(((key_dist(rgb) <= 130) & vis).sum())
        return out
    d = key_dist(rgb)
    out["corners_ok"] = all(d[y, x] <= 30 for y, x in corners)
    out["corner_dist"] = [round(float(d[y, x]), 1) for y, x in corners]
    near = d <= 130
    bg = flood_border(near)
    enclosed = near & ~bg
    out["enclosed_pure"] = int((enclosed & (d <= 30)).sum())
    out["enclosed_near"] = int((enclosed & (d > 30)).sum())
    out["band"] = int(((d > 30) & (d <= 130) & bg).sum())  # fringe only
    return out


def review_sheet(ids: list[str], manifest: dict) -> Path:
    by_id = {a["id"]: a for a in manifest["assets"]}
    biomes = list(PALETTES)
    floors = {b: Image.open(TILES / f"{b}_floor.png").convert("RGBA") for b in biomes}
    walls = {b: Image.open(TILES / f"{b}_wall.png").convert("RGBA") for b in biomes}
    # each row: id label | master 1x | runtime over every biome's floor+wall, in PALETTES order
    # (runtime = nearest /2, tiles at 2x so 1 env texel = 2 px, 1 sprite texel = 1 px -> option C)
    rows = []
    for aid in ids:
        a = by_id[aid]
        master = build_sprite(SRC / f"{aid}.png", int(a["master_h"]), True)
        rt = master.resize((max(1, master.width // 2), max(1, master.height // 2)), Image.Resampling.NEAREST)
        panels = []
        for b in biomes:
            pw, ph = rt.width + 32, rt.height + 32
            bg = Image.new("RGBA", (pw, ph))
            f2 = floors[b].resize((16, 16), Image.Resampling.NEAREST)
            w2 = walls[b].resize((16, 16), Image.Resampling.NEAREST)
            for y in range(0, ph, 16):
                for x in range(0, pw, 16):
                    bg.paste(w2 if y == 0 else f2, (x, y))
            bg.alpha_composite(rt, (16, ph - rt.height - 8))
            panels.append(bg)
        rows.append((aid, master, panels))
    label_w = 150
    scale = 3  # magnify the whole row so a phone-sized read can be judged on a desktop
    row_h = max(max(m.height, max(p.height for p in ps)) for _, m, ps in rows) + 8
    total_w = label_w + max(m.width + sum(p.width + 6 for p in ps) for _, m, ps in rows) + 16
    sheet = Image.new("RGBA", (total_w, row_h * len(rows) + 8), (40, 38, 36, 255))
    d = ImageDraw.Draw(sheet)
    y = 4
    for aid, m, ps in rows:
        d.text((4, y), aid, fill=(230, 220, 200, 255))
        x = label_w
        sheet.alpha_composite(m, (x, y))
        x += m.width + 6
        for p in ps:
            sheet.alpha_composite(p, (x, y))
            x += p.width + 6
        y += row_h
    QC.mkdir(exist_ok=True)
    out = QC / "review.png"
    sheet.resize((sheet.width * scale, sheet.height * scale), Image.Resampling.NEAREST).save(out)
    return out


def main(argv: list[str]) -> int:
    manifest = json.loads((ROOT / "manifest.json").read_text())
    keyed = [a["id"] for a in manifest["assets"] if a["bg"] == "keyed"]
    ids = [i for i in (argv or keyed) if (SRC / f"{i}.png").exists() and i in keyed]
    for aid in ids:
        c = census(SRC / f"{aid}.png")
        flag = "" if (c["corners_ok"] and c["short_edge_ok"] and c["enclosed_near"] < 200) else "  <-- CHECK"
        print(f"{aid:<20} {c['size'][0]}x{c['size'][1]} alpha={c['authored_alpha']} corners={c['corners_ok']}"
              f" band={c['band']} enclosed_pure={c['enclosed_pure']} enclosed_near={c['enclosed_near']}{flag}")
    if ids:
        print("review sheet:", review_sheet(ids, manifest))
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
