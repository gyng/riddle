#!/usr/bin/env python3
"""Cut 30 town: the warm light emitters painted into each town sprite (EMBER flames, a lit forge, lit windows), as offsets in
world units (= env texels) from the sprite's anchor (bottom-centre of its atlas frame). The renderer feeds them to the light
field at night (docs/TOWN.md §3). Read from the packed atlas: bright warm texels, clustered.

    python3 art/town_lights.py          # writes art/town_lights.json {id: [[dx, dy, strength], ...]} (dy negative = up)
"""
from __future__ import annotations

import json
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
ATLAS = ROOT.parent / "web/public/art"


def emitters(rgba: np.ndarray, max_n: int = 4) -> list[list[float]]:
    rgb = rgba[..., :3].astype(np.float32)
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    warm = (rgba[..., 3] > 0) & (r > 140) & (r - b > 80) & (g > 0.35 * r) & (g < 0.78 * r)
    if warm.sum() < 3:
        return []
    from refine import _regions
    lab = _regions(warm, False)
    h, w = warm.shape
    out = []
    for i in range(1, lab.max() + 1):
        ys, xs = np.nonzero(lab == i)
        if len(ys) < 2:
            continue
        # master px -> world units (half), relative to the bottom-centre
        out.append([float((xs.mean() - w / 2) / 2), float((ys.mean() - h) / 2), len(ys)])
    if not out:
        return []
    out.sort(key=lambda e: -e[2])
    merged: list[list[float]] = []   # one light per flame or window group: clusters within 6 units join (weighted)
    for e in out:
        for m in merged:
            if abs(m[0] - e[0]) <= 6 and abs(m[1] - e[1]) <= 6:
                n = m[2] + e[2]
                m[0], m[1], m[2] = (m[0] * m[2] + e[0] * e[2]) / n, (m[1] * m[2] + e[1] * e[2]) / n, n
                break
        else:
            merged.append(list(e))
    out = sorted(([round(float(m[0]), 1), round(float(m[1]), 1), m[2]] for m in merged), key=lambda e: -e[2])
    out = out[:max_n]
    top = max(e[2] for e in out)
    return [[e[0], e[1], round(min(1.0, (e[2] / top) ** 0.5), 2)] for e in out]


def main() -> None:
    import sys
    sys.path.insert(0, str(ROOT))
    a = np.asarray(Image.open(ATLAS / "atlas.png").convert("RGBA"))
    fr = json.loads((ATLAS / "atlas.json").read_text())["frames"]
    res = {}
    for k, f in sorted(fr.items()):
        if not k.startswith(("town_mouth", "town_campfire", "town_blacksmith", "town_storehouse", "town_kennel", "town_bank")):
            continue
        e = emitters(a[f["y"]:f["y"] + f["h"], f["x"]:f["x"] + f["w"]])
        if e:
            res[k] = e
    (ROOT / "town_lights.json").write_text(json.dumps(res, indent=1) + "\n")
    for k, v in res.items():
        print(k, v)


if __name__ == "__main__":
    main()
