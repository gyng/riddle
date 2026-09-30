#!/usr/bin/env python3
"""Art direction phase 2, the owner's call ("the previous tileset had more readability — refine it"): the round-26 painted register's
value structure and layout (art/tiles_r26/, the 16x16 tiles as they shipped at 7253bb9) repainted in the guide's palette.

Each tile's luminance is normalised per (place, class) — every floor of a place shares one range, so they still tile together — and
laid on the place's style ramp (INK · UMBRA · DUSK' · · MOON' · · MIST · BONE, make_tiles.style_ramp) inside a band per class that
keeps the classes apart by value: floors a calm mid band, wall tops a step lighter, wall faces dark courses under a MIST capstone with
an INK line where the wall meets the floor, water dark with moon glints, the chasm near black. A quarter of the old tile's own hue is
kept (so the Burrows' earth stays ochre and the Fens' water teal inside the tint rule). Props get a BONE moon rim on their top edge so
they pop from the floor. Written over the painted conversion by art/make_env.py (`refine_all`); `REFINE` below names what it covers.

The painterly pass (`painterly()`, the owner after the refined set: "more painterly, retain the readability, more similar to the original
but with more style") keeps those bands and adds, per tile: a share of the round-26 material colour back (`MATERIAL`, inside the cast
bar), a wash bloom, watercolour blotches with pooled rims, each stone its own graded wash (lit upper left, pigment pooled at its foot),
brush strokes along the grain, an edge-preserving smoothing inside each value band (one soft wash, not pixel mottle), pigment pooled
against darker lines, hand-inked mortar lines and prop contours (uneven weight, heavier on the shadow side), a chroma bleed, quiet
paper grain and BONE paper in the lights; the texel's value may move ≤ 18/255 from the refined one.

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
# the painterly pass (the owner, after the refined set: "more painterly, retain the readability, more similar to the original but with
# more style"): per place, how much of the round-26 tile's own material colour comes back (its chroma, on top of HUE_KEEP) and the
# saturation after it — the Burrows' earth, the Fens' teal, the Foundry's heat, the Sanctum's pale stone, inside the tint rule's cast bar
MATERIAL = {"warrens": 0.6, "burrows": 0.16, "fens": 0.5, "crypt": 0.4, "foundry": 0.75, "deep": 0.4, "sanctum": 0.6}
PSAT = {"warrens": 1.0, "burrows": 1.05, "fens": 0.9, "crypt": 0.85, "foundry": 1.05, "deep": 0.8, "sanctum": 0.95}
PAINTERLY = True
SMOOTH, SMOOTH_BAND = 0.6, 0.12   # the wash's smoothing inside one value band (painterly: a soft graded wash, not mottle)
WASH_GRAD = {"floor": 0.16, "wall_top": 0.2, "wall_face": 0.14}   # the value fall across one stone's wash, lit corner to pooled edge
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
            if PAINTERLY:   # the old tile's material colour back (a share of its chroma), then the place's saturation
                rgb = rgb + chroma * MATERIAL.get(biome, 0.35)
                g = (rgb @ np.array([0.2126, 0.7152, 0.0722], np.float32))[..., None]
                rgb = g + (rgb - g) * PSAT.get(biome, 1.0)
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
            if PAINTERLY:
                rgb = painterly(rgb, u, op, k, tid, ramp)
            out = np.dstack([np.clip(rgb, 0, 255), np.where(op, 255, 0)]).astype(np.uint8)
            im = Image.fromarray(out, "RGBA")
            q = im.convert("RGB").quantize(24 if k != "prop" else 23, method=Image.Quantize.MEDIANCUT).convert("RGBA")
            qa = np.asarray(q).copy()
            qa[..., 3] = out[..., 3]
            Image.fromarray(qa, "RGBA").save(out_dir / f"{tid}.png")
            written.append(tid)
    return written


def _seed(s: str) -> int:
    import zlib
    return zlib.crc32(s.encode())


def _periodic_field(rng: np.random.Generator, n: int = 16, waves: int = 4, max_k: int = 2) -> np.ndarray:
    """a smooth field that tiles on n (a sum of a few random plane waves with integer wave numbers), -1..1"""
    y, x = np.mgrid[0:n, 0:n].astype(np.float32) / n
    f = np.zeros((n, n), np.float32)
    for _ in range(waves):
        kx, ky = rng.integers(-max_k, max_k + 1, 2)
        if kx == 0 and ky == 0:
            kx = 1
        f += np.cos(2 * np.pi * (kx * x + ky * y) + rng.uniform(0, 2 * np.pi)) * rng.uniform(0.5, 1)
    return f / max(1e-3, np.abs(f).max())


def _nb(a: np.ndarray, dy: int, dx: int, wrap: bool) -> np.ndarray:
    if wrap:
        return np.roll(a, (dy, dx), (0, 1))
    p = np.pad(a, [(1, 1), (1, 1)] + [(0, 0)] * (a.ndim - 2), mode="edge")
    h, w = a.shape[:2]
    return p[1 - dy:1 - dy + h, 1 - dx:1 - dx + w]


def _regions(mask: np.ndarray, wrap: bool) -> np.ndarray:
    """4-connected labels of a boolean mask (wrapping on a seamless tile); 0 = outside"""
    h, w = mask.shape
    lab = np.zeros((h, w), int)
    n = 0
    for y in range(h):
        for x in range(w):
            if mask[y, x] and not lab[y, x]:
                n += 1
                st = [(y, x)]
                lab[y, x] = n
                while st:
                    cy, cx = st.pop()
                    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                        ny, nx = cy + dy, cx + dx
                        if wrap:
                            ny, nx = ny % h, nx % w
                        elif not (0 <= ny < h and 0 <= nx < w):
                            continue
                        if mask[ny, nx] and not lab[ny, nx]:
                            lab[ny, nx] = n
                            st.append((ny, nx))
    return lab


def painterly(rgb: np.ndarray, u: np.ndarray, op: np.ndarray, k: str, tid: str, ramp: np.ndarray) -> np.ndarray:
    """watercolour on the pixel grid, the value bands kept: a wash bloom (a smooth colour/value drift across the tile, shared by a
    place's class so floors still tile), brush strokes along the piece's grain (floors diagonal, walls along their courses, props
    upright), pigment pooled against every darker line, a hand-inked contour (mortar valleys and a prop's silhouette in INK at an
    uneven weight, heavier on the shadow side), a chroma bleed across boundaries, paper grain, and BONE paper in the lights.
    Seamless tiles (every class but props) wrap every neighbourhood."""
    h, w = u.shape
    wrap = k != "prop"
    biome = tid.split("_env_", 1)[0]
    rng = np.random.default_rng(_seed(tid))
    crng = np.random.default_rng(_seed(biome + k))
    out = rgb.astype(np.float32).copy()
    L = lambda a: (a @ np.array([0.2126, 0.7152, 0.0722], np.float32))  # noqa: E731
    lum0 = L(out)
    # 1. wash bloom: value ±, and a drift between the place's tinted mid (ramp[3]) and a cooler MIST-ward lean
    bloom = _periodic_field(crng, 16, 3, 1) if (h, w) == (16, 16) else _periodic_field(crng, max(h, w), 3, 1)[:h, :w]
    amp = {"floor": 0.1, "wall_top": 0.11, "wall_face": 0.1, "water": 0.12, "chasm": 0.06}.get(k, 0.07)
    out *= (1 + amp * bloom)[..., None]
    lean = np.where(bloom[..., None] > 0, ramp[3], ramp[5])
    out = out + (lean - out) * (0.12 * np.abs(bloom))[..., None]
    # 2. watercolour blotches: a patch of wash (a thresholded smooth field) a touch lighter inside, its rim pooled darker — the
    # cauliflower edge a wet wash leaves, blocked on the grid
    blot = _periodic_field(rng, 16, 4, 2) if (h, w) == (16, 16) else _periodic_field(rng, max(h, w), 4, 2)[:h, :w]
    inside = blot > rng.uniform(0.15, 0.4)
    rim = np.zeros((h, w), bool)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        rim |= inside & ~_nb(inside, dy, dx, wrap)
    bamp = {"floor": 0.1, "wall_top": 0.08, "wall_face": 0.07, "water": 0.1, "prop": 0.05}.get(k, 0.06)
    out[inside & ~rim] *= 1 + bamp
    out[rim & op] *= 1 - bamp * (0.9 if k == "floor" else 1.6)   # (floors calm: a full rim read as specks)
    # 2b. each stone its own wash: the regions between the dark lines (a slab, a block, a course) each take their own tone — a value
    # step and a lean between the tint's mid and the moonlit mid — as if each were laid in with its own stroke of the brush
    if k in ("floor", "wall_top", "wall_face", "door", "stairs_down", "stairs_up"):
        lab = _regions(u >= np.percentile(u, 22), wrap)
        ramt = {"floor": 0.05, "wall_top": 0.08, "wall_face": 0.07}.get(k, 0.05)
        for r in range(1, lab.max() + 1):
            m = lab == r
            out[m] *= 1 + ramt * rng.uniform(-1, 1)
            tgt = ramp[3] if rng.random() < 0.5 else ramp[5]
            out[m] = out[m] + (tgt - out[m]) * rng.uniform(0.0, 0.1)
            # the wash dries lighter where the moon falls (upper left) and pools pigment toward its lower edge
            ys, xs = np.nonzero(m)
            if len(ys) >= 6:
                yn = (ys - ys.min()) / max(1, ys.max() - ys.min())
                xn = (xs - xs.min()) / max(1, xs.max() - xs.min())
                out[ys, xs] *= (1 + WASH_GRAD.get(k, 0.12) * (0.5 - (0.7 * yn + 0.3 * xn)))[:, None]
                low = m & ~_nb(m, -1, 0, wrap)   # the region's bottom texels (the one below is outside it)
                out[low] *= 0.95 if k == "floor" else 0.88
    # 3. brush strokes along the grain: dabs two texels wide, 3–6 long, lighter or darker, the stroke's tail pooled darker
    strokes = np.zeros((h, w), np.float32)
    dirs = {"floor": [(1, 1), (1, -1)], "wall_top": [(0, 1), (1, 1)], "wall_face": [(0, 1)], "door": [(1, 0)], "water": [(0, 1)],
            "stairs_down": [(0, 1)], "stairs_up": [(0, 1)], "chasm": [(1, 1)], "prop": [(1, 0), (1, 1)]}[k]
    n_str = {"floor": 4, "wall_top": 5, "wall_face": 5, "water": 5, "prop": 3}.get(k, 4)
    for _ in range(n_str):
        dy, dx = dirs[rng.integers(len(dirs))]
        py, px = (dx != 0), (dx == 0)   # the stroke's width: across its direction
        y0, x0 = rng.integers(0, h), rng.integers(0, w)
        ln = int(rng.integers(3, 7))
        v = rng.choice([-1.0, 1.0]) * rng.uniform(0.6, 1.0)
        for i in range(ln):
            for j in (0, 1):
                yy, xx = y0 + dy * i + py * j, x0 + dx * i + px * j
                if wrap:
                    yy, xx = yy % h, xx % w
                elif not (0 <= yy < h and 0 <= xx < w):
                    continue
                strokes[yy, xx] += v * (0.6 if j else 1.0) - (0.8 if i == ln - 1 else 0.0)
    smag = {"floor": 0.09, "wall_top": 0.1, "wall_face": 0.1, "water": 0.12, "prop": 0.06}.get(k, 0.06)
    out *= (1 + smag * np.clip(strokes, -1.5, 1.2))[..., None]
    # 3b. the wash lies smooth: each texel takes the mean of its 3x3 neighbours that sit in the same value band (the same wash —
    # never across a mortar line, a rim or a silhouette), twice, so a stone reads as one soft graded wash rather than pixel mottle
    for _ in range(2):
        acc = out * 1.0
        wsum = np.ones((h, w, 1), np.float32)
        for dy in (-1, 0, 1):
            for dx in (-1, 0, 1):
                if dy == dx == 0:
                    continue
                nu = _nb(u, dy, dx, wrap)
                ok = (np.abs(nu - u) < SMOOTH_BAND) & (_nb(op, dy, dx, wrap) if not wrap else True)
                m = ok[..., None].astype(np.float32) * (0.7 if dy and dx else 1.0)
                acc += _nb(out, dy, dx, wrap) * m
                wsum += m
        out = out * (1 - SMOOTH) + (acc / wsum) * SMOOTH
    # 4. pigment pooling: a wash darkens where it meets a darker line (a 1-texel rim inside the lighter shape)
    darker = np.zeros((h, w), bool)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        n = _nb(u, dy, dx, wrap)
        darker |= (n < u - 0.22)
    if not wrap:
        darker &= op
    out[darker] *= 0.84
    # 5. the hand-inked line: mortar valleys (the tile's darkest texels that sit below both neighbours on an axis) toward INK at an
    # uneven weight; a prop's silhouette inked, heavier on the shadow side (bottom/right), the moon rim on top left alone
    ink = ramp[0]
    wgt = rng.uniform(0.3, 0.65, (h, w)).astype(np.float32)
    if k in ("wall_face", "wall_top", "floor", "door", "stairs_down", "stairs_up"):
        valley = np.zeros((h, w), bool)
        for dy, dx in ((1, 0), (0, 1)):
            valley |= (u < _nb(u, dy, dx, wrap) - 0.12) & (u < _nb(u, -dy, -dx, wrap) - 0.12)
        valley &= u < np.percentile(u, 30)
        # a line, not a dot: an inked texel needs an inked neighbour (lone dark texels read as specks on a calm floor)
        line = np.zeros((h, w), bool)
        for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)):
            line |= _nb(valley, dy, dx, wrap)
        valley &= line
        f = {"floor": 0.55, "wall_top": 0.8}.get(k, 1.0)
        out[valley] = out[valley] + (ink - out[valley]) * (wgt[valley] * f)[..., None]
    if k == "prop":
        q = np.pad(op, 1)
        below = op & ~q[2:, 1:-1]
        right = op & ~q[1:-1, 2:]
        left = op & ~q[1:-1, :-2]
        edge = below | right | left
        wt = np.where(below | right, wgt + 0.25, wgt * 0.8)
        out[edge] = out[edge] + (ink - out[edge]) * np.clip(wt[edge], 0, 0.9)[..., None]
    # 6. bleed: a fifth of the neighbours' colour (not their value) runs across each boundary
    lum1 = L(out)
    chroma = out - lum1[..., None]
    acc = chroma.copy()
    cnt = np.ones((h, w, 1), np.float32)
    for dy, dx in ((1, 0), (-1, 0), (0, 1), (0, -1)):
        nc = _nb(chroma, dy, dx, wrap)
        m = _nb(op, dy, dx, wrap)[..., None].astype(np.float32) if not wrap else 1.0
        acc += nc * m
        cnt += m
    out = lum1[..., None] + chroma * 0.78 + (acc / cnt) * 0.22
    # 7. paper: a fine grain on every texel and BONE paper showing through the lightest washes
    grain = rng.uniform(-1, 1, (h, w)).astype(np.float32)
    out *= (1 + 0.012 * grain)[..., None]   # (quiet: the renderer lays the screen paper grain; a louder one read as pixel noise)
    lit = np.clip((L(out) / 255 - 0.5) / 0.3, 0, 1)
    out = out + (ramp[7] - out) * (0.12 * lit)[..., None]
    # keep the band: the painterly pass may not move a texel's mean value far from where the refined pass put it
    lum2 = L(out)
    drift = np.clip(lum2 - lum0, -18, 18) - (lum2 - lum0)
    out += drift[..., None] * 0.6
    return out


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
