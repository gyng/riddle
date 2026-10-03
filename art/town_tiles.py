#!/usr/bin/env python3
"""Cut 30 town v1: the town's ground in the painterly register (art/refine.py `painterly()`), and the path's edges.

The town's terrain sources (`envp_town_*`, Codex) come through `painted.convert` as plain 16x16 paintings; on their own the grass and
the dirt sat at one value (both grey-blue speckle). Here each ground class is laid on the town's style ramp inside its own value band,
the way `refine.py` bands the dungeon's classes — grass dark and moss-green (the town's tint), the path a clear step lighter and
neutral, the plaza between them with INK joints — then given the painterly pass (washes, blotches, strokes along the grain, the
band-limited smoothing, pooled pigment, paper). Then the path's borders are composed from the calm grass and path tiles:

  town_env_dirt_edge_{n,e,s,w}        a path cell with grass along that side (a ragged fringe; one profile, so runs join)
  town_env_dirt_corner_{ne,nw,se,sw}  outer corner: grass along both named sides
  town_env_dirt_inner_{ne,nw,se,sw}   inner corner: grass only in that corner (the inside of a bend)

A fringe's profile is 4 texels deep at both ends of every side, so an edge meets its neighbours and the corners. Grass on the north
side of a path casts a one-texel shadow onto it (the moon is above); the other sides pool a lighter line. Run by art/make_env.py
after `refine_all` (or alone: it re-converts the town sources itself, so it never compounds).

    python3 art/town_tiles.py        # writes art/tiles/town_env_*.png and adds the derived ids to art/tiles/_painted.json
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
from make_tiles import style_ramp  # noqa: E402
from refine import lum, painterly, ramp_at  # noqa: E402

SRC = ROOT / "generated"
OUT = ROOT / "tiles"

# class -> (members, ramp band lo/hi, gamma, material colour lean (rgb), lean amount, painterly class)
GROUND: dict[str, tuple[list[str], float, float, float, tuple[int, int, int], float, str]] = {
    "grass": (["grass_0", "grass_1", "grass_2", "grass_3"], 0.14, 0.42, 1.1, (58, 84, 52), 0.42, "floor"),
    "dirt": (["dirt_0", "dirt_1", "dirt_2"], 0.34, 0.6, 0.9, (122, 112, 96), 0.3, "floor"),   # (the coordinator: the path read as bright stripy ribbons; 0.42-0.84 before, at full contrast)
    "plaza": (["plaza_0", "plaza_1"], 0.2, 0.66, 0.9, (96, 104, 112), 0.15, "floor"),
}
EXTREMES = {"grass": (7, 0), "dirt": (2, 0), "plaza": (8, 2)}   # (grass glints read as a regular dash pattern at 1x: none)   # per class: % of texels to INK, % to MIST
SQUEEZE = {"dirt": 0.5}   # per class: the painting's contrast kept (the path's grain read as stripes at 0.85)
PROFILE = [4, 4, 5, 5, 4, 4, 3, 4, 5, 5, 5, 4, 3, 3, 4, 4]   # the fringe's depth along a side (texels); 4 at both ends


def _src(name: str) -> np.ndarray:
    from painted import convert
    from pack import key_source
    im = convert(SRC / f"envp_town_{name}.png", "env", (16, 16), key_source)
    return np.asarray(im.convert("RGBA"), np.float32)


def ground() -> dict[str, np.ndarray]:
    ramp = np.array([[int(h[i:i + 2], 16) for i in (1, 3, 5)] for h in style_ramp("town")], np.float32)
    out: dict[str, np.ndarray] = {}
    for cls, (names, lo, hi, gam, lean, amt, pk) in GROUND.items():
        have = [n for n in names if (SRC / f"envp_town_{n}.png").exists()]
        if not have:
            continue
        tiles = {n: _src(n) for n in have}
        L_all = np.concatenate([lum(t).ravel() for t in tiles.values()])
        p2, p98 = np.percentile(L_all, [2, 98])
        for n, a in tiles.items():
            L = lum(a)
            u = np.clip((L - p2) / max(1e-3, p98 - p2), 0, 1) ** gam
            u = 0.5 + (u - 0.5) * SQUEEZE.get(cls, 0.85)
            t = lo + (hi - lo) * u
            rgb = ramp_at(ramp, t)
            g = rgb @ np.array([0.2126, 0.7152, 0.0722], np.float32)
            leanc = np.array(lean, np.float32)
            leanc = leanc - (leanc @ np.array([0.2126, 0.7152, 0.0722], np.float32))   # the lean's colour, not its value
            rgb = g[..., None] + (rgb - g[..., None]) * 0.6 + leanc * amt * 1.6
            chroma = a[..., :3] - (L * 255)[..., None]
            rgb = rgb + chroma * 0.15   # a trace of the painting's own colour (flowers, a puddle's glint)
            op = np.ones(L.shape, bool)
            rgb = painterly(rgb, u, op, pk, f"town_env_{n}", ramp)
            # the moonlit extremes (§3: every frame spans INK to MIST): the deepest gaps between blades / ruts / joints toward INK,
            # the brightest few texels catch the moon (dew on the grass, a pebble on the path, a cobble's lit corner)
            lv = lum(np.dstack([rgb, np.full(L.shape, 255.0)]))
            dk, lt = EXTREMES[cls]
            deep = lv <= np.percentile(lv, dk)
            glint = lv >= np.percentile(lv, 100 - lt) if lt else np.zeros(lv.shape, bool)
            rgb[deep] = rgb[deep] * 0.35 + ramp[0] * 0.65
            rgb[glint] = rgb[glint] * 0.15 + ramp[6] * 0.85
            out[n] = np.clip(rgb, 0, 255)
        # the variants of a class share one mean (a darker variant read as a checker of squares across a field)
        mean = np.mean([out[n].mean(axis=(0, 1)) for n in tiles], axis=0)
        for n in tiles:
            out[n] = np.clip(out[n] - out[n].mean(axis=(0, 1)) + mean, 0, 255)
    return out


def _masks() -> dict[str, np.ndarray]:
    y, x = np.mgrid[0:16, 0:16]
    prof = np.array(PROFILE)
    side = {
        "n": y < prof[x],
        "s": y > 15 - prof[x],
        "w": x < prof[y],
        "e": x > 15 - prof[y],
    }
    m: dict[str, np.ndarray] = {f"edge_{k}": v for k, v in side.items()}
    m["edge_ns"] = side["n"] | side["s"]   # a one-cell path running east-west
    m["edge_ew"] = side["e"] | side["w"]   # ... and north-south
    for k in "nesw":   # a dead end capped on side k: grass on k and both flanks
        flank = "ew" if k in "ns" else "ns"
        m[f"end_{k}"] = side[k] | side[flank[0]] | side[flank[1]]
    m["isle"] = side["n"] | side["e"] | side["s"] | side["w"]   # a lone path cell (a doorstep)
    for c in ("ne", "nw", "se", "sw"):
        m[f"corner_{c}"] = side[c[0]] | side[c[1]]
        cy = 0 if c[0] == "n" else 15
        cx = 15 if c[1] == "e" else 0
        d = np.hypot(y - cy, x - cx)
        ang = np.arctan2(np.abs(y - cy), np.abs(x - cx))   # 0 along the row, pi/2 down the column
        r = 4.0 + 1.0 * np.sin(2 * ang) + 0.6 * np.sin(5 * ang)   # 4 at both ends (meets the edges' profile), a lobe between
        m[f"inner_{c}"] = d < r
    return m


def edges(g: dict[str, np.ndarray]) -> dict[str, np.ndarray]:
    grass, dirt = g["grass_0"], g["dirt_0"]
    out = {}
    for name, m in _masks().items():
        rgb = np.where(m[..., None], grass, dirt).copy()
        # the boundary: path texels touching grass. Below grass (grass to the north) a moon shadow; elsewhere a lighter pooled line.
        p = np.pad(m, 1, mode="edge")
        north = ~m & p[:-2, 1:-1]
        touch = ~m & (p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:])
        rgb[north] *= 0.72
        rgb[touch & ~north] *= 0.88
        # the grass's fringe tips (grass texels touching path) a touch lighter: blades catching the moon
        q = np.pad(~m, 1, mode="edge")
        tips = m & (q[:-2, 1:-1] | q[2:, 1:-1] | q[1:-1, :-2] | q[1:-1, 2:])
        rgb[tips] = rgb[tips] * 0.85 + np.array([96, 120, 92], np.float32) * 0.15
        out[f"dirt_{name}"] = np.clip(rgb, 0, 255)
    return out


def hill(g: dict[str, np.ndarray]) -> np.ndarray:
    """town_env_cliff: the dark hillside the mouth is cut into (the coordinator, Cut 30 integration: the old ledge tile, its MIST
    rims repeated every 16 texels, read as a striped blue wall or water). The calm grass laid two steps down toward UMBRA/DUSK, a
    few boulders (a lit upper-left rim, an INK foot) and scrub; no horizontal run, so rows of it never stripe. Seamless (the
    grass is; the boulders sit inside the tile)."""
    ramp = np.array([[int(h[i:i + 2], 16) for i in (1, 3, 5)] for h in style_ramp("town")], np.float32)
    base = g.get("grass_2", g["grass_0"]).copy()
    lv = base @ np.array([0.2126, 0.7152, 0.0722], np.float32)
    u = (lv - lv.min()) / max(1e-3, float(lv.max() - lv.min()))
    slate = np.array([34, 40, 44], np.float32)
    rgb = slate[None, None] * (0.62 + 0.5 * u[..., None]) + (base - lv[..., None]) * 0.22   # a trace of the grass's own green
    # boulders: (cx, cy, rx, ry)
    y, x = np.mgrid[0:16, 0:16].astype(np.float32)
    for cx, cy, rx, ry in ((4.5, 5.0, 2.4, 1.6), (11.5, 11.5, 1.6, 1.1)):
        d = ((x - cx) / rx) ** 2 + ((y - cy) / ry) ** 2
        body = d < 1.0
        rgb[body] = np.array([38, 42, 50], np.float32) + (rgb[body] - rgb[body].mean(0)) * 0.4
        rim = body & (((x - cx + 0.9) / rx) ** 2 + ((y - cy + 0.9) / ry) ** 2 >= 1.0)   # the upper-left edge the moon finds
        rgb[rim] = rgb[rim] * 0.7 + ramp[2] * 0.3
        foot = ~body & (((x - cx) / rx) ** 2 + ((y - cy - 1.0) / ry) ** 2 < 1.0)   # its shadow on the slope below
        rgb[foot] = rgb[foot] * 0.5 + ramp[0] * 0.5
    return np.clip(rgb, 0, 255)


MIST = np.array([164, 188, 214], np.float32)


def save(tid: str, rgb: np.ndarray) -> None:
    """<= 24 colours; the moon's glints (a handful of texels the median cut would merge into the mid-tones) kept as one MIST"""
    lv = rgb @ np.array([0.2126, 0.7152, 0.0722], np.float32)
    glint = lv >= 150
    q = np.asarray(Image.fromarray(rgb.astype(np.uint8), "RGB").quantize(23, method=Image.Quantize.MEDIANCUT).convert("RGB")).copy()
    q[glint] = MIST.astype(np.uint8)
    Image.fromarray(q, "RGB").convert("RGBA").save(OUT / f"town_env_{tid}.png")


def build(out_dir: Path = OUT) -> list[str]:
    global OUT
    OUT = out_dir
    g = ground()
    written = []
    for n, rgb in g.items():
        save(n, rgb)
        written.append(f"town_env_{n}")
    if "grass_0" in g and "dirt_0" in g:
        for n, rgb in edges(g).items():
            save(n, rgb)
            written.append(f"town_env_{n}")
        save("dirt_edge", edges(g)["dirt_edge_n"])   # the phase-2 id, kept as an alias of the north edge
        written.append("town_env_dirt_edge")
    if "grass_0" in g:
        save("cliff", hill(g))
        written.append("town_env_cliff")
    pj = out_dir / "_painted.json"
    have = set(json.loads(pj.read_text())) if pj.exists() else set()
    pj.write_text(json.dumps(sorted(have | set(written))) + "\n")
    return written


if __name__ == "__main__":
    w = build()
    print(f"town ground: {len(w)} tiles -> {OUT}")
