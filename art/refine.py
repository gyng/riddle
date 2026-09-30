#!/usr/bin/env python3
"""Art direction phase 2, the owner's call ("the previous tileset had more readability — refine it"): the round-26 painted register's
value structure and layout (art/tiles_r26/, the 16x16 tiles as they shipped at 7253bb9) repainted in the guide's palette.

Each tile's luminance is normalised per (place, class) — every floor of a place shares one range, so they still tile together — and
laid on the place's style ramp (INK · UMBRA · DUSK' · · MOON' · · MIST · BONE, make_tiles.style_ramp) inside a band per class that
keeps the classes apart by value: floors a calm mid band, wall tops a step lighter, wall faces dark courses under a MIST capstone with
an INK line where the wall meets the floor, water dark with moon glints, the chasm near black. A quarter of the old tile's own hue is
kept (so the Burrows' earth stays ochre and the Fens' water teal inside the tint rule). Props get a BONE moon rim on their top edge so
they pop from the floor. Written over the painted conversion by art/make_env.py (`refine_all`); `REFINE` below names what it covers.

    python3 art/refine.py sheet <out.png>   # old (r26) · phase-2 paint · refined, per place, as sample rooms at game zoom
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
R26 = ROOT / "tiles_r26"
sys.path.insert(0, str(ROOT))
from make_tiles import style_ramp  # noqa: E402

# class → (ramp position low, high) for the tile's p2..p98 luminance (0 = INK, 1 = BONE); the gamma lifts or sinks the middle
BANDS: dict[str, tuple[float, float, float]] = {
    "floor": (0.22, 0.5, 1.0),
    "wall_top": (0.34, 0.7, 1.0),
    "wall_face": (0.08, 0.9, 0.9),
    "door": (0.02, 0.8, 1.0),
    "stairs_down": (0.04, 0.92, 0.7),   # (blind check: "the stairs read as a dark box" — the step lips lifted)
    "stairs_up": (0.1, 0.88, 1.0),
    "water": (0.1, 0.66, 1.0),   # (blind check: "the water reads as a chasm" — lifted off the void, moon glints below)
    "chasm": (0.0, 0.2, 1.0),
    "prop": (0.06, 0.94, 0.95),
}
HUE_KEEP = 0.2   # of the old tile's own colour (its chroma around its luminance)
SAT = {"fens": 0.6, "deep": 0.55, "crypt": 0.7}   # per place: the refined tile's saturation (default 0.85)
FLOOR_CALM = 0.8  # a floor's contrast inside its band (the old floors' speckle damped)
TILE_CLASSES = ("floor", "wall_top", "wall_face", "door", "stairs_down", "stairs_up", "water", "chasm")


def klass(name: str) -> str:
    for k in TILE_CLASSES:
        if name == k or name.startswith(k + "_"):
            return k
    return "prop"


def lum(a: np.ndarray) -> np.ndarray:
    return (a[..., :3] @ np.array([0.2126, 0.7152, 0.0722], np.float32)) / 255


def ramp_at(ramp: np.ndarray, t: np.ndarray) -> np.ndarray:
    x = np.clip(t, 0, 1) * (len(ramp) - 1)
    i = np.clip(np.floor(x).astype(int), 0, len(ramp) - 2)
    f = (x - i)[..., None]
    return ramp[i] * (1 - f) + ramp[i + 1] * f


def load_r26() -> dict[str, np.ndarray]:
    return {p.stem: np.asarray(Image.open(p).convert("RGBA"), np.float32) for p in sorted(R26.glob("*.png"))}


def refine_all(out_dir: Path) -> list[str]:
    old = load_r26()
    by: dict[tuple[str, str], list[str]] = {}
    for tid in old:
        biome, name = tid.split("_env_", 1)
        by.setdefault((biome, klass(name)), []).append(tid)
    written = []
    for (biome, k), tids in by.items():
        ramp = np.array([[int(h[i:i + 2], 16) for i in (1, 3, 5)] for h in style_ramp(biome)], np.float32)
        lo_t, hi_t, gam = BANDS[k]
        # one value range per (place, class): the p2..p98 of every opaque texel of the class's tiles together
        L_all = np.concatenate([lum(old[t])[old[t][..., 3] > 0] for t in tids])
        p2, p98 = np.percentile(L_all, [2, 98])
        for tid in tids:
            a = old[tid].copy()
            op = a[..., 3] > 0
            L = lum(a)
            u = np.clip((L - p2) / max(1e-3, p98 - p2), 0, 1) ** gam
            if k == "floor":
                u = 0.5 + (u - 0.5) * FLOOR_CALM
            t = lo_t + (hi_t - lo_t) * u
            rgb = ramp_at(ramp, t)
            chroma = a[..., :3] - (L * 255)[..., None]
            rgb = rgb + chroma * HUE_KEEP
            # (the tint already leans the mid-tones; a ramp this saturated on top of the old hue went cyan/cobalt — pull the saturation
            # back toward the tile's grey so only the tint and a trace of the old hue remain)
            g = (rgb @ np.array([0.2126, 0.7152, 0.0722], np.float32))[..., None]
            rgb = g + (rgb - g) * SAT.get(biome, 0.85)
            name = tid.split("_env_", 1)[1]
            if k == "wall_face":   # the wall meets the floor on an INK line; its top row catches the moon (the capstone's lit lip)
                rgb[-1] = ramp[0]
                rgb[0] = rgb[0] * 0.4 + ramp[6] * 0.6
            if k == "water":       # the brightest ripples catch the moon (MIST glints), so water is never a hole
                g = u > np.percentile(u, 88)
                rgb[g] = rgb[g] * 0.35 + ramp[6] * 0.65
            if k == "stairs_down":  # each step's lit lip (the row's brightest run) in MIST
                g = u > np.percentile(u, 82)
                rgb[g] = rgb[g] * 0.45 + ramp[6] * 0.55
            if k == "prop":        # a BONE moon rim on the top-facing edge so a prop pops from the floor
                q = np.pad(op, 1)
                rim = op & ~q[:-2, 1:-1] & q[2:, 1:-1]
                rgb[rim] = rgb[rim] * 0.45 + ramp[7] * 0.55
            out = np.dstack([np.clip(rgb, 0, 255), np.where(op, 255, 0)]).astype(np.uint8)
            im = Image.fromarray(out, "RGBA")
            q = im.convert("RGB").quantize(24 if k != "prop" else 23, method=Image.Quantize.MEDIANCUT).convert("RGBA")
            qa = np.asarray(q).copy()
            qa[..., 3] = out[..., 3]
            Image.fromarray(qa, "RGBA").save(out_dir / f"{tid}.png")
            written.append(tid)
    return written


def sheet(out: str) -> None:
    """old (r26) · phase-2 paint (the current atlas) · refined (art/tiles), per place, each a dressed sample room at game zoom"""
    from make_env import sample_room
    import subprocess
    subprocess.run(["git", "show", "7253bb9:web/public/art/atlas.png"], stdout=open("/tmp/_r26_atlas.png", "wb"), check=True, cwd=ROOT.parent)
    subprocess.run(["git", "show", "7253bb9:web/public/art/atlas.json"], stdout=open("/tmp/_r26_atlas.json", "wb"), check=True, cwd=ROOT.parent)

    def tiles_from(png: str, js: str) -> dict[str, Image.Image]:
        a = Image.open(png).convert("RGBA")
        fr = json.load(open(js))["frames"]
        return {k: a.crop((f["x"], f["y"], f["x"] + f["w"], f["y"] + f["h"])) for k, f in fr.items() if "env_" in k}

    old = tiles_from("/tmp/_r26_atlas.png", "/tmp/_r26_atlas.json")
    cur = tiles_from(str(ROOT.parent / "scratchpad/style/phase2/before_refine_atlas.png"), str(ROOT.parent / "scratchpad/style/phase2/before_refine_atlas.json")) \
        if (ROOT.parent / "scratchpad/style/phase2/before_refine_atlas.png").exists() else {}
    new = tiles_from(str(ROOT.parent / "web/public/art/atlas.png"), str(ROOT.parent / "web/public/art/atlas.json"))
    from PIL import ImageDraw
    biomes = ["warrens", "burrows", "fens", "crypt", "foundry", "deep", "sanctum"]
    W, H = 160 * 3, 96 * 3
    cols = [("round 26", old), ("phase 2", cur), ("refined", new)]
    img = Image.new("RGB", (len(cols) * (W + 10), len(biomes) * (H + 22)), (13, 12, 20))
    d = ImageDraw.Draw(img)
    for r, b in enumerate(biomes):
        for c, (lab, t) in enumerate(cols):
            if not t:
                continue
            room = sample_room(t, b).resize((W, H), Image.Resampling.NEAREST)
            x, y = c * (W + 10), r * (H + 22)
            img.paste(room.convert("RGB"), (x, y + 18))
            d.text((x + 4, y + 3), f"{b} · {lab}", fill=(234, 223, 197))
    img.save(out)
    print(out)


if __name__ == "__main__":
    if sys.argv[1:2] == ["sheet"]:
        sheet(sys.argv[2])
