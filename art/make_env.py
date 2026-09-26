#!/usr/bin/env python3
"""Register 3 — the 16x16 environment: Codex pixel-art sources -> ramp-indexed tiles per biome.

Sources: art/generated/env_*.png (manifest bg "env" = opaque tile, "env_keyed" = decal/prop on
#0000FF or authored alpha). For each source:

  1. key (pack.key_source) and, for keyed ones, crop to alpha and fit inside the manifest `texels`
     box (aspect kept; props bottom-centred, decals centred); opaque tiles box-downscale whole.
  2. VALUE -> RAMP INDEX. Ramp-class assets (`RAMP`): luminance is cut at fixed quantiles into
     the class's index list, so every floor variant has the same value distribution (the four
     floors tile together whatever Codex painted) and every biome gets the same drawing in its own
     8-colour ramp (`<biome>_env_<name>.png`, with `BIOME_REMAP` where a ramp's roles differ:
     the sanctum's field is pale, the foundry's 3/5 are oranges). Props get a 1-texel ramp-0 rim.
  3. HUE assets (`HUE`: blood, torch, banner) keep their colour: median-cut to <= 8 colours,
     edge texels darkened; one file `env_<name>.png` shared by every biome (the renderer draws
     them sprite-tagged, 30 % tint, so blood stays red and flame stays orange).
  4. torch_1 is derived from torch (the flame's rows lifted one texel, core and body swapped) so
     the two frames register exactly.

Writes art/tiles/*.png (packed by pack.py) and art/tiles/_env_sheet.png (every biome at 4x plus a
dressed sample room). Run: python3 art/make_env.py && python3 art/pack.py && python3 art/art-qc.py
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
from make_tiles import PALETTES, hex_rgb  # noqa: E402
from pack import key_source  # noqa: E402

SRC = ROOT / "generated"
OUT = ROOT / "tiles"

# name -> (ramp index list darkest..lightest, cumulative luminance quantile cuts between them)
RAMP: dict[str, tuple[list[int], list[float]]] = {
    "floor_0": ([1, 2, 3, 4], [0.15, 0.88, 0.992]),
    "floor_1": ([1, 2, 3, 4], [0.15, 0.88, 0.992]),
    "floor_2": ([1, 2, 3, 4], [0.15, 0.88, 0.992]),
    "floor_3": ([1, 2, 3, 4], [0.15, 0.88, 0.992]),
    "wall_face_0": ([0, 1, 2, 3, 4, 5], [0.12, 0.3, 0.55, 0.8, 0.94]),
    "wall_face_1": ([0, 1, 2, 3, 4, 5], [0.12, 0.3, 0.55, 0.8, 0.94]),
    "wall_top": ([1, 2, 3], [0.25, 0.85]),
    "door": ([0, 1, 2, 3, 4, 5], [0.2, 0.38, 0.55, 0.75, 0.92]),
    "stairs_down": ([0, 1, 2, 3, 4, 5], [0.25, 0.42, 0.6, 0.78, 0.92]),
    "stairs_up": ([1, 2, 3, 4, 5, 6], [0.15, 0.35, 0.55, 0.75, 0.92]),
    "water": ([0, 1, 2, 3, 5], [0.2, 0.55, 0.85, 0.96]),
    "chasm": ([0, 1], [0.85]),
    "moss_0": ([2, 3, 4], [0.35, 0.8]),
    "moss_1": ([2, 3, 4], [0.35, 0.8]),
    "crack": ([0, 1, 4], [0.55, 0.85]),
    "rubble": ([0, 2, 3, 5], [0.3, 0.6, 0.85]),
    "barrel": ([0, 1, 2, 3, 4, 5], [0.15, 0.32, 0.5, 0.7, 0.88]),
    "crate": ([0, 1, 2, 3, 4, 5], [0.15, 0.32, 0.5, 0.7, 0.88]),
    "pot": ([0, 1, 2, 3, 4, 5], [0.15, 0.32, 0.5, 0.7, 0.88]),
    "bones": ([0, 3, 5, 6, 7], [0.2, 0.4, 0.65, 0.88]),
    # second art pass: the Cut 5 situation props at the 16-texel density (the 8x8 register stays the fallback)
    "vault": ([0, 1, 2, 3, 4, 6], [0.2, 0.36, 0.52, 0.72, 0.9]),
    "vault_open": ([0, 1, 2, 3, 4, 6], [0.3, 0.45, 0.6, 0.78, 0.92]),
    "nest": ([0, 1, 2, 3, 4, 5, 6], [0.12, 0.25, 0.42, 0.62, 0.82, 0.95]),
}
HUE = {"blood_0", "blood_1", "torch", "banner", "shrine", "item_potion", "item_scroll", "item_weapon", "item_armour", "item_gold"}
PROPS = {"barrel", "crate", "pot", "bones", "torch", "banner", "shrine", "vault", "vault_open", "nest", "item_gold"}   # bottom-anchored, 1-texel dark rim
# a biome whose ramp roles differ from the warrens' shape: ramp index -> this biome's index
BIOME_REMAP: dict[str, dict[int, int]] = {
    # pale dressed stone: slate mortar, pale field (the old sanctum floor's field was 5)
    "sanctum": {1: 3, 2: 5, 3: 6, 4: 7, 0: 1},
    # 3 and 5 are oranges: stone uses the iron greys (4, 6) for its lit values, rust for the field
    "foundry": {3: 4, 4: 6, 5: 6},
}
BIOME_REMAP_KEEP = {"moss_0", "moss_1"}   # moss stays in the biome's own hue steps
# juice pass 2 (2026-09-26): the Burrows gets its own tile art (it was the Warrens' recoloured at runtime, atlas.ts TILE_ALIAS);
# its ramp is web/src/render/palette.ts's (warm ochre earth since juice pass 3, on the Warrens' luminance steps). Env-only: the 8x8 register
# (make_tiles.py) keeps six biomes and the Burrows keeps the runtime alias as its fallback there.
ENV_PALETTES: dict[str, list[str]] = {**PALETTES, "burrows": ["#140b05", "#2b190b", "#43280f", "#5e3f18", "#7a5724", "#a57d3a", "#c9a25a", "#ecd6a0"]}


# juice pass 3 (2026-09-26): the Burrows and the Fens meet at the D5 fork and must tell apart at a glance. A luminance ramp maps
# plank and water (or root and earth) of one value to one colour, so their drawings read as the same speckle recoloured. Their TILES
# are drawn by Codex in a fixed 8-colour palette with hue roles and converted by NEAREST COLOUR (each texel's block median), so the
# water stays water-coloured under a plank-coloured boardwalk and a root stays a root on the earth. Props, decals and the shared
# drawings keep the ramp path. Roles are listed so make_prompts.py can hand the palette to Codex.
FREE_PALETTES: dict[str, list[tuple[str, str]]] = {
    "fens": [("#060f0f", "near-black gaps and deep shadow"), ("#0c2426", "deep water"), ("#174040", "water"),
             ("#2a5d55", "lit water, ripples"), ("#373628", "plank shadow, wood grain"), ("#4f4d3b", "plank (weathered grey wood)"),
             ("#6e6b52", "plank lit edge, pale reed tips"), ("#46663a", "reed and moss green")],
    "burrows": [("#140b05", "near-black gaps and holes"), ("#2b190b", "deep earth shadow"), ("#43280f", "root, dark earth"),
                ("#5e3f18", "packed ochre earth (the field)"), ("#7a5724", "earth lit, pebbles"), ("#a57d3a", "brightest ochre highlight"),
                ("#6b3a1e", "timber (reddish wood)"), ("#8e5530", "timber lit edge")],
}
# a colour swap after the nearest-colour pass, per (biome, class): the Fens' boards read as brick courses where each plank's lit end
# met a near-black butt joint every 16 texels — the lit ends go to the plank colour and the joints to the grain colour, so a floor is
# long boards between dark-water lines
FREE_SWAP: dict[tuple[str, str], dict[str, str]] = {("fens", "floor"): {"#6e6b52": "#4f4d3b", "#060f0f": "#373628"}}
FREE_NAMES = {"floor_0", "floor_1", "floor_2", "floor_3", "wall_face_0", "wall_face_1", "wall_top", "door", "stairs_down", "stairs_up", "water"}


def to_free(biome: str, name: str, texels: tuple[int, int], src: str) -> Image.Image:
    """nearest colour in the biome's FREE palette, per texel, of the median of the texel block's inner half (a Codex pixel grid is
    rarely exact: the median of the block's middle ignores the seams between blocks)."""
    rgba = key_source(SRC / f"{src}.png")
    tw, th = texels
    H, W = rgba.shape[:2]
    pal = np.array([hex_rgb(c) for c, _ in FREE_PALETTES[biome]], np.float32)
    w8 = np.array([0.35, 0.5, 0.25], np.float32)   # a rough perceptual weight (green carries value)
    out = np.zeros((th, tw, 4), np.uint8)
    for y in range(th):
        for x in range(tw):
            y0, y1 = int((y + 0.25) * H / th), int((y + 0.75) * H / th)
            x0, x1 = int((x + 0.25) * W / tw), int((x + 0.75) * W / tw)
            c = np.median(rgba[y0:y1, x0:x1, :3].reshape(-1, 3), axis=0)
            k = int(np.argmin((((pal - c) ** 2) * w8).sum(-1)))
            out[y, x, :3] = pal[k]
            out[y, x, 3] = 255
    for a, b in FREE_SWAP.get((biome, name.split("_")[0]), {}).items():
        m = np.all(out[..., :3] == np.array(hex_rgb(a), np.uint8), axis=-1)
        out[m, :3] = hex_rgb(b)
    return Image.fromarray(out, "RGBA")


def lum(rgb: np.ndarray) -> np.ndarray:
    return rgb[..., 0] * 0.2126 + rgb[..., 1] * 0.7152 + rgb[..., 2] * 0.0722


def load(name: str, texels: tuple[int, int], keyed: bool, src: str | None = None) -> np.ndarray:
    """float RGBA at texel size (alpha binarised). `src`: the source id when it is not `env_<name>` (a per-biome drawing)."""
    rgba = key_source(SRC / f"{src or 'env_' + name}.png")
    tw, th = texels
    if not keyed:
        rgba[..., 3] = 255.0
        im = Image.fromarray(np.clip(rgba, 0, 255).astype(np.uint8), "RGBA").convert("RGB")
        return np.dstack([np.asarray(im.resize((tw, th), Image.Resampling.BOX), dtype=np.float32), np.full((th, tw), 255.0)])
    ys, xs = np.nonzero(rgba[..., 3] > 8)
    rgba = rgba[ys.min(): ys.max() + 1, xs.min(): xs.max() + 1]
    h, w = rgba.shape[:2]
    sc = min(tw / w, th / h)
    nw, nh = max(1, round(w * sc)), max(1, round(h * sc))
    a = rgba[..., 3:4] / 255.0
    pre = np.dstack([rgba[..., :3] * a, rgba[..., 3:4]])
    small = np.asarray(Image.fromarray(np.clip(pre, 0, 255).astype(np.uint8), "RGBA").resize((nw, nh), Image.Resampling.BOX), dtype=np.float32)
    a2 = small[..., 3:4] / 255.0
    rgb = np.where(a2 > 0, small[..., :3] / np.maximum(a2, 1e-6), 0)
    out = np.zeros((th, tw, 4), np.float32)
    ox = (tw - nw) // 2
    oy = th - nh if name in PROPS else (th - nh) // 2
    out[oy:oy + nh, ox:ox + nw, :3] = rgb
    out[oy:oy + nh, ox:ox + nw, 3] = np.where(small[..., 3] >= 128, 255.0, 0.0)
    return out


def edge_mask(alpha: np.ndarray) -> np.ndarray:
    m = alpha > 0
    p = np.pad(m, 1)
    return m & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])


# juice pass 2: an organic floor (earth, mud, cave rock) has no mortar — the flagstones' 15 % darkest texels read as speckle on it
ORGANIC_FLOOR = ([1, 2, 3, 4], [0.05, 0.9, 0.992])
BIOME_CUTS: dict[tuple[str, str], tuple[list[int], list[float]]] = {
    **{("burrows", f"floor_{i}"): ORGANIC_FLOOR for i in range(4)},
    **{("deep", f"floor_{i}"): ORGANIC_FLOOR for i in range(4)},
    ("fens", "floor_1"): ORGANIC_FLOOR, ("fens", "floor_2"): ORGANIC_FLOOR,
}


# juice pass 2: a per-biome drawing keeps its OWN value structure — its luminance, normalised between its 2nd and 98th percentiles, is
# cut at fixed points (a floor's thresholds keep it quiet: most texels on its two middle steps) instead of at fixed quantiles, which
# sprayed a two-value checker or a riveted plate into speckle (the shared drawings keep the quantiles: their floors must tile together)
STRUCT_T: dict[str, list[float]] = {"floor": [0.28, 0.7, 0.9]}
STRUCT_T_BIOME: dict[tuple[str, str], list[float]] = {("sanctum", "floor"): [0.1, 0.45, 0.93]}   # the marble's checker on two adjacent pale steps
STRUCT_OFF = {"fens"}   # the Fens' painted mud and reeds read better on the quantiles (its structure is texture, not lines)
BIOME_IDX: dict[tuple[str, str], list[int]] = {("foundry", "water"): [2, 3, 5, 7]}   # the molten channel keeps the foundry's oranges
REMAP_SKIP = {("foundry", "water")}


def to_index_struct(name: str, px: np.ndarray, biome: str) -> np.ndarray:
    idx_list = BIOME_IDX.get((biome, name), RAMP[name][0])
    L = lum(px[..., :3])
    vis = px[..., 3] > 0
    lo, hi = np.quantile(L[vis], [0.02, 0.98]) if vis.any() else (0.0, 1.0)
    t = np.clip((L - lo) / max(1e-6, hi - lo), 0, 1)
    n = len(idx_list)
    cls = name.split("_")[0]
    cuts = STRUCT_T_BIOME.get((biome, cls)) or STRUCT_T.get(cls, [(i + 1) / n for i in range(n - 1)])
    k = np.searchsorted(np.array(cuts), t, side="right")
    out = np.array(idx_list)[np.clip(k, 0, n - 1)]
    if name in PROPS:
        out = np.where(edge_mask(px[..., 3]), 0, out)
    return np.where(vis, out, -1)


def to_index(name: str, px: np.ndarray, biome: str | None = None) -> np.ndarray:
    """ramp index per texel (-1 = transparent)."""
    idx_list, cuts = BIOME_CUTS.get((biome or "", name), RAMP[name])
    L = lum(px[..., :3])
    vis = px[..., 3] > 0
    qs = np.quantile(L[vis], cuts) if vis.any() else np.zeros(len(cuts))
    k = np.searchsorted(qs, L, side="right")
    out = np.array(idx_list)[np.clip(k, 0, len(idx_list) - 1)]
    if name in PROPS:
        out = np.where(edge_mask(px[..., 3]), 0, out)
    return np.where(vis, out, -1)


def paint(index: np.ndarray, ramp: list[str]) -> Image.Image:
    cols = np.array([hex_rgb(c) for c in ramp], np.uint8)
    h, w = index.shape
    out = np.zeros((h, w, 4), np.uint8)
    vis = index >= 0
    out[vis, :3] = cols[index[vis]]
    out[vis, 3] = 255
    return Image.fromarray(out, "RGBA")


def hue_asset(px: np.ndarray, colors: int = 7) -> np.ndarray:
    im = Image.fromarray(np.clip(px, 0, 255).astype(np.uint8), "RGBA")
    q = np.asarray(im.convert("RGB").quantize(colors=colors, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE).convert("RGB"), np.float32)
    out = px.copy()
    vis = px[..., 3] > 0
    out[vis, :3] = q[vis]
    e = edge_mask(px[..., 3])
    out[e, :3] = np.array([22, 14, 12], np.float32)   # the one ink colour (8th)
    return out


def torch_frame1(px: np.ndarray) -> np.ndarray:
    """flicker: flame texels (warm, bright) lift one row; the brightest two warm colours swap."""
    rgb = px[..., :3]
    warm = (px[..., 3] > 0) & (rgb[..., 0] > 150) & (rgb[..., 0] >= rgb[..., 1]) & (rgb[..., 2] < 140)
    out = px.copy()
    ys, xs = np.nonzero(warm)
    for y, x in sorted(zip(ys, xs)):
        if y > 0 and not warm[y - 1, x] and px[y - 1, x, 3] == 0:
            out[y - 1, x] = px[y, x]
    L = lum(rgb)
    if warm.any():
        levels = sorted({tuple(c) for c in rgb[warm].astype(int)}, key=lambda c: -lum(np.array(c, np.float32)))
        if len(levels) >= 2:
            a, b = np.array(levels[0], np.float32), np.array(levels[1], np.float32)
            ma = warm & np.all(rgb == a, axis=-1)
            mb = warm & np.all(rgb == b, axis=-1)
            out[ma, :3] = b
            out[mb, :3] = a
    del L
    return out


FLAME_CORE, FLAME_BODY = np.array([255, 226, 130, 255], np.float32), np.array([236, 128, 36, 255], np.float32)


def shrine_frames(px: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """the shrine keeps its stone's own greys (a hue asset, one file for every biome) and its candles get a lit flame: on each
    of the two brightest texels with open air above, a core texel, and a body texel over it; frame 1 swaps core and body and
    lifts the tip one texel (the 8x8 shrine's 1 Hz flicker)."""
    base = hue_asset(px, colors=5)
    L = lum(px[..., :3])
    vis = px[..., 3] > 0
    open_above = np.zeros_like(vis)
    open_above[1:] = vis[1:] & ~vis[:-1]
    cand = sorted(zip(*np.nonzero(open_above)), key=lambda yx: -L[yx])[:2]
    f0, f1 = base.copy(), base.copy()
    for y, x in cand:
        if y < 2:
            continue
        f0[y - 1, x] = FLAME_CORE; f0[y - 2, x] = FLAME_BODY
        f1[y - 1, x] = FLAME_BODY; f1[y - 2, x] = FLAME_CORE
        if y >= 3:
            f1[y - 3, x] = FLAME_BODY
    return f0, f1


def shrine_frame1(idx: np.ndarray) -> np.ndarray:
    """the altar light flickers: the brightest texels (the candle flame) lift one row, as the 8x8 shrine's frame 1 did."""
    out = idx.copy()
    top = idx.max()
    ys, xs = np.nonzero(idx == top)
    for y, x in sorted(zip(ys, xs)):
        if y > 0 and idx[y - 1, x] < 0:
            out[y - 1, x] = top
            out[y, x] = max(3, top - 2)
    return out


def nest_frame1(idx: np.ndarray) -> np.ndarray:
    """the den wakes: two lit eye texels open in the darkest hollow (the 8x8 nest's frame 1)."""
    out = idx.copy()
    vis = idx >= 0
    p = np.pad(vis, 1)
    inner = vis & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
    dark = inner & (idx <= 1)
    if not dark.any():
        dark = inner & (idx <= np.min(idx[inner]))
    ys, xs = np.nonzero(dark)
    cy, cx = int(round(ys.mean())), int(round(xs.mean()))
    for dx in (-1, 1):
        x = min(idx.shape[1] - 1, max(0, cx + dx))
        out[cy, x] = 7
    return out


def main() -> int:
    manifest = json.loads((ROOT / "manifest.json").read_text())
    every = [a for a in manifest["assets"] if a["bg"].startswith("env")]
    assets = [a for a in every if not a.get("biome")]
    # juice pass 2: per-biome drawings (`env_<biome>_<name>`, manifest `biome`/`name`) replace the shared drawing for that biome
    own: dict[tuple[str, str], dict] = {(a["biome"], a["name"]): a for a in every if a.get("biome") and (SRC / f"{a['id']}.png").exists()}
    written: dict[str, Image.Image] = {}
    missing = []
    for a in assets:
        name = a["id"][4:]
        if not (SRC / f"{a['id']}.png").exists():
            missing.append(a["id"])
            continue
        tw, th = (int(v) for v in a["texels"].split("x"))
        px = load(name, (tw, th), a["bg"] == "env_keyed")
        if name in HUE:
            out = hue_asset(px)
            written[f"env_{name}"] = Image.fromarray(np.clip(out, 0, 255).astype(np.uint8), "RGBA")
            if name == "shrine":
                f0, f1 = shrine_frames(px)
                del written["env_shrine"]
                for biome in ENV_PALETTES:   # per-biome ids, so the renderer's `<biome>_env_shrine_<f>` lookup finds it everywhere
                    written[f"{biome}_env_shrine_0"] = Image.fromarray(np.clip(f0, 0, 255).astype(np.uint8), "RGBA")
                    written[f"{biome}_env_shrine_1"] = Image.fromarray(np.clip(f1, 0, 255).astype(np.uint8), "RGBA")
                continue
            if name == "torch":
                written["env_torch_1"] = Image.fromarray(np.clip(torch_frame1(out), 0, 255).astype(np.uint8), "RGBA")
                written["env_torch_0"] = written.pop("env_torch")
            continue
        index = to_index(name, px)
        frames = {name: index}
        if name == "shrine":
            frames = {"shrine_0": index, "shrine_1": shrine_frame1(index)}
        elif name == "nest":
            frames = {"nest_0": index, "nest_1": nest_frame1(index)}
        for fname, findex in frames.items():
            for biome, ramp in ENV_PALETTES.items():
                if (biome, name) in own:
                    continue
                idx = findex
                remap = BIOME_REMAP.get(biome)
                if remap and name not in BIOME_REMAP_KEEP:
                    lut = np.array([remap.get(i, i) for i in range(8)])
                    idx = np.where(findex >= 0, lut[np.clip(findex, 0, 7)], -1)
                written[f"{biome}_env_{fname}"] = paint(idx, ramp)
    for (biome, name), a in own.items():
        tw, th = (int(v) for v in a["texels"].split("x"))
        if biome in FREE_PALETTES and name in FREE_NAMES:
            written[f"{biome}_env_{name}"] = to_free(biome, name, (tw, th), a["id"])
            continue
        src = load(name, (tw, th), a["bg"] == "env_keyed", a["id"])
        index = to_index(name, src, biome) if biome in STRUCT_OFF else to_index_struct(name, src, biome)
        remap = BIOME_REMAP.get(biome)
        if remap and name not in BIOME_REMAP_KEEP and (biome, name) not in REMAP_SKIP:
            lut = np.array([remap.get(i, i) for i in range(8)])
            index = np.where(index >= 0, lut[np.clip(index, 0, 7)], -1)
        written[f"{biome}_env_{name}"] = paint(index, ENV_PALETTES[biome])
    for k, im in written.items():
        im.save(OUT / f"{k}.png")
        n = len({tuple(p) for p in np.asarray(im).reshape(-1, 4) if p[3] > 0})
        if n > 8:
            print(f"FAIL {k}: {n} colours")
            return 1
    sheet(written).save(OUT / "_env_sheet.png")
    print(f"wrote {len(written)} env tiles -> {OUT}")
    if missing:
        print("missing sources:", ", ".join(missing))
    return 0


def sheet(t: dict[str, Image.Image], s: int = 4) -> Image.Image:
    """per biome: every env tile at s x, then a dressed 8x5 sample room at s x."""
    biomes = list(ENV_PALETTES)
    names = sorted({k.split("_env_", 1)[1] for k in t if "_env_" in k})
    hue = sorted(k for k in t if k.startswith("env_"))
    cell = 16 * s + 4
    room_w, room_h = 10, 6
    W = max(len(names) + len(hue), room_w * 4 + 2) * cell
    H = len(biomes) * (cell + room_h * 16 * s // 2 + 16)
    img = Image.new("RGBA", (W, H), (24, 22, 20, 255))
    y = 4
    for b in biomes:
        x = 4
        for n in names + hue:
            im = t.get(f"{b}_env_{n}") or t.get(n)
            if im is None:
                continue
            img.alpha_composite(im.resize((im.width * s, im.height * s), Image.Resampling.NEAREST), (x, y))
            x += cell
        y += cell + 4
        img.alpha_composite(sample_room(t, b).resize((room_w * 16 * s // 2, room_h * 16 * s // 2), Image.Resampling.NEAREST), (4, y))
        y += room_h * 16 * s // 2 + 12
    return img


def sample_room(t: dict[str, Image.Image], b: str) -> Image.Image:
    """10x6 tiles: a wall face row with a torch and a banner, floor variants, decals and props."""
    g = lambda n: t.get(f"{b}_env_{n}") or t.get(f"env_{n}")  # noqa: E731
    W, H = 10, 6
    im = Image.new("RGBA", (W * 16, H * 16), (0, 0, 0, 255))
    for x in range(W):
        if g("wall_top"):
            im.alpha_composite(g("wall_top"), (x * 16, 0))
        f = g("wall_face_0" if x % 3 else "wall_face_1")
        if f:
            im.alpha_composite(f if x != 5 or not g("door") else g("door"), (x * 16, 16))
    for yy in range(2, H):
        for x in range(W):
            n = ["floor_0", "floor_1", "floor_0", "floor_2", "floor_0", "floor_3", "floor_1"][(x * 7 + yy * 13) % 7]
            if g(n):
                im.alpha_composite(g(n), (x * 16, yy * 16))
    place = [("moss_0", 1, 3), ("crack", 6, 4), ("rubble", 8, 2), ("blood_0", 4, 3), ("blood_1", 5, 5), ("moss_1", 0, 5),
             ("barrel", 0, 2), ("crate", 9, 2), ("pot", 9, 3), ("bones", 3, 5), ("stairs_down", 7, 5), ("water", 1, 4), ("chasm", 2, 4),
             ("shrine_0", 5, 2), ("vault", 6, 3), ("vault_open", 7, 3), ("nest_1", 8, 4), ("item_potion", 2, 2), ("item_gold", 3, 2),
             ("item_weapon", 4, 4), ("item_scroll", 7, 2), ("item_armour", 6, 5)]
    for n, x, yy in place:
        s = g(n)
        if s:
            im.alpha_composite(s, (x * 16, yy * 16 + 16 - s.height))
    for n, x in (("torch_0", 2), ("torch_1", 7), ("banner", 4)):
        s = g(n)
        if s:
            im.alpha_composite(s, (x * 16, 32 - s.height + (4 if n != "banner" else 8)))
    return im


if __name__ == "__main__":
    raise SystemExit(main())
