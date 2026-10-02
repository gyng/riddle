#!/usr/bin/env python3
"""Pack Riddle art into web/public/art/atlas.png + atlas.json (+ title.png).

Keyed sprites (bg == "keyed" in art/manifest.json), from art/generated/<id>.png:
  1. chroma-key -> RGBA. Sources with a transparent corner carry authored alpha
     and are used as-is; opaque sources are keyed off flat #0000FF with an alpha
     ramp on RGB distance (transparent <= LO, opaque >= HI) and de-spilled.
  2. crop to alpha bounds.
  3. box-resample (premultiplied) to the manifest master_h (hero 96, monsters
     64-96, bosses 128, summons 48: 2x the runtime sprite-texel height so the
     renderer can downscale nearest by 2 or use as-is), binarise alpha.
  4. dilate the silhouette by OUTLINE_PX and paint the ring dark ink; darken
     the 1-px inner edge, so the ink line survives the downscale.
  5. quantise opaque pixels to <= QUANT_COLOURS (median cut, no dither).
Tiles, overlays and item glyphs from art/tiles/*.png (8x8 RGBA; register 3's `*env_*` at 16x16,
16x24 torch, 16x20 banner — art/make_env.py) copy through.
Everything is shelf-packed with GUTTER-px gutters; atlas.json is
{ "frames": { "<id>": {"x","y","w","h"} }, "meta": {...} }.
The title (bg == "bleed") goes to web/public/art/title.png at its manifest size.

Run: python3 art/pack.py          (add --no-quant to skip step 5)
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from make_tiles import BOSS_FLASH, PALETTES, style_ramp  # noqa: E402  single source of truth for ramps

ROOT = Path(__file__).resolve().parent
SRC = ROOT / "generated"
TILES = ROOT / "tiles"
DST = ROOT.parent / "web" / "public" / "art"

LO = 30.0  # distance from #0000FF below which alpha = 0
HI = 130.0  # distance above which alpha = 255
OUTLINE_PX = 2
INK = (22, 18, 20)
QUANT_COLOURS = 32
GUTTER = 1
ATLAS_W = 1024


def key_source(src: Path) -> np.ndarray:
    """Return float RGBA (0..255, straight alpha) with the background removed."""
    im = Image.open(src).convert("RGBA")
    rgba = np.asarray(im, dtype=np.float32)
    rgb = rgba[..., :3]
    a = rgba[..., 3] / 255.0
    corners = (a[0, 0], a[0, -1], a[-1, 0], a[-1, -1])
    if min(corners) < 0.95:
        return rgba  # authored alpha
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    kr = key_red(rgb)
    dist = np.sqrt((r - kr) ** 2 + g * g + (b - 255.0) ** 2)
    alpha = np.clip((dist - LO) / (HI - LO), 0.0, 1.0)
    spill = alpha < 1.0
    if kr:   # art direction phase 2: a magenta key (#FF00FF, far from the moonlit blues) — pull r and b down to g where they spill
        ex = np.maximum(0.0, np.minimum(r, b) - g)
        r = np.where(spill, r - ex, r)
        b = np.where(spill, b - ex, b)
    else:
        b = np.where(spill, np.minimum(b, np.maximum(r, g)), b)
    return np.dstack([r, g, b, alpha * 255.0])


def key_red(rgb: np.ndarray) -> float:
    """The source's key: pure blue #0000FF (v1/v2) or magenta #FF00FF (phase 2), by the corners' red channel."""
    c = np.array([rgb[0, 0], rgb[0, -1], rgb[-1, 0], rgb[-1, -1]])
    return 255.0 if float(np.median(c[:, 0])) > 128 else 0.0


def crop_to_alpha(rgba: np.ndarray, thresh: float = 8.0) -> np.ndarray:
    ys, xs = np.nonzero(rgba[..., 3] > thresh)
    if len(xs) == 0:
        raise ValueError("empty alpha")
    return rgba[ys.min() : ys.max() + 1, xs.min() : xs.max() + 1]


def resample_master(rgba: np.ndarray, master_h: int) -> np.ndarray:
    """Premultiplied box downscale to master_h, then binarised alpha."""
    h, w = rgba.shape[:2]
    inner_h = master_h - 2 * OUTLINE_PX
    scale = inner_h / h
    new_w = max(1, int(round(w * scale)))
    a = rgba[..., 3:4] / 255.0
    pre = np.dstack([rgba[..., :3] * a, rgba[..., 3:4]])
    im = Image.fromarray(np.clip(pre, 0, 255).astype(np.uint8), "RGBA")
    im = im.resize((new_w, inner_h), Image.Resampling.BOX)
    out = np.asarray(im, dtype=np.float32)
    a2 = out[..., 3:4] / 255.0
    rgb = np.where(a2 > 0, out[..., :3] / np.maximum(a2, 1e-6), 0.0)
    alpha = np.where(out[..., 3] >= 128, 255.0, 0.0)
    return np.dstack([np.clip(rgb, 0, 255), alpha])


def dilate(mask: np.ndarray, n: int) -> np.ndarray:
    m = mask.copy()
    for _ in range(n):
        p = np.pad(m, 1)
        m = (
            p[1:-1, 1:-1] | p[:-2, 1:-1] | p[2:, 1:-1] | p[1:-1, :-2] | p[1:-1, 2:]
            | p[:-2, :-2] | p[:-2, 2:] | p[2:, :-2] | p[2:, 2:]
        )
    return m


def outline(rgba: np.ndarray) -> np.ndarray:
    """Pad by OUTLINE_PX, dilate the silhouette into a dark ink ring, darken the inner edge."""
    p = OUTLINE_PX
    rgba = np.pad(rgba, ((p, p), (p, p), (0, 0)))
    mask = rgba[..., 3] >= 128
    ring = dilate(mask, p) & ~mask
    inner = mask & ~erode(mask, 1)
    out = rgba.copy()
    out[ring, :3] = INK
    out[ring, 3] = 255
    out[inner, :3] = out[inner, :3] * 0.55
    return out


def erode(mask: np.ndarray, n: int) -> np.ndarray:
    m = mask.copy()
    for _ in range(n):
        p = np.pad(m, 1, constant_values=False)
        m = (
            p[1:-1, 1:-1] & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
        )
    return m


def quantise(rgba: np.ndarray, colours: int) -> np.ndarray:
    im = Image.fromarray(rgba.astype(np.uint8), "RGBA")
    rgb = im.convert("RGB").quantize(colors=colours, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE)
    q = np.asarray(rgb.convert("RGB"), dtype=np.float32)
    out = rgba.copy()
    opaque = rgba[..., 3] >= 128
    out[opaque, :3] = q[opaque]
    return out


RIM = (234, 223, 197)   # BONE


def moon_rim(before: np.ndarray, after: np.ndarray, k: float = 0.6) -> np.ndarray:
    """art direction phase 2 (the owner: "the hero must be readable"; blind round 27: "the hero a tiny grey blob"): a thin BONE moon
    rim on the hero — the silhouette's top- and left-facing edge pixels (just inside the ink ring) take the moon instead of the ring's
    darkening. `before` is the resampled master, `after` the same padded and outlined."""
    p = OUTLINE_PX
    m = np.pad(before[..., 3] >= 128, p)
    q = np.pad(m, 1)
    edge = m & ~q[:-2, 1:-1] & q[2:, 1:-1]   # top-facing (the moon is above), not a one-texel strand (a bowstring, a hair)
    e = np.pad(edge, ((0, 0), (1, 1)))
    edge &= e[:, :-2] | e[:, 2:]   # runs of two or more (a lone step on a diagonal would read as a dotted line)
    rows = np.where(m.any(1))[0]
    if rows.size:   # the upper 60 % of the figure (head, shoulders, the raised weapon), where the moon lands
        edge[int(rows[0] + 0.6 * (rows[-1] - rows[0])):] = False
    src = np.pad(before[..., :3], ((p, p), (p, p), (0, 0)))
    out = after.copy()
    out[edge, :3] = src[edge] * (1 - k) + np.array(RIM, np.float32) * k
    return out


EMBER_CORE, EMBER_BODY = np.array([255, 216, 140], np.float32), np.array([232, 146, 58], np.float32)


def warm_mask(rgb: np.ndarray) -> np.ndarray:
    """a flame's texels: bright, red well over blue, green between a third and three quarters of red (EMBER #e8923a is 0.63; GILT #b89448, 0.8, stays gold)"""
    r, g, b = rgb[..., 0], rgb[..., 1], rgb[..., 2]
    return (r > 170) & (r - b > 90) & (g > 0.35 * r) & (g < 0.74 * r)


def keep_emissive(src_rgba: np.ndarray, master: np.ndarray) -> np.ndarray:
    """Cut 30 town: a torch flame, a forge mouth or a lit window is a few hundred px in a 1536 px master and the box downscale
    averages it into the dark wood around it (the mouth's two torches came out (112, 92, 79): unlit). Where a master texel was
    at least a fifth flame in the source, it takes EMBER (half flame or more: the pale core), so the warm lights survive at game size."""
    h, w = master.shape[:2]
    inner_h = h
    m = (warm_mask(src_rgba[..., :3]) & (src_rgba[..., 3] > 128)).astype(np.float32) * 255
    frac = np.asarray(Image.fromarray(m.astype(np.uint8), "L").resize((w, inner_h), Image.Resampling.BOX), np.float32) / 255
    out = master.copy()
    op = out[..., 3] >= 128
    body = op & (frac >= 0.2)
    core = op & (frac >= 0.5)
    out[body, :3] = out[body, :3] * 0.25 + EMBER_BODY * 0.75
    out[core, :3] = out[core, :3] * 0.2 + EMBER_CORE * 0.8
    return out


def build_sprite(src: Path, master_h: int, quant: bool, rim: bool = False, emissive: bool = False) -> Image.Image:
    rgba = key_source(src)
    rgba = crop_to_alpha(rgba)
    hi = rgba
    rgba = resample_master(rgba, master_h)
    if emissive:
        rgba = keep_emissive(hi, rgba)
    out = outline(rgba)
    if rim:
        out = moon_rim(rgba, out)
    elif emissive:   # Cut 30 town: the moon on the roofs and the tops of props (§6: MIST rims on top edges), a lighter touch than the hero's
        out = moon_rim(rgba, out, k=0.45)
    rgba = out
    if quant:
        q = quantise(rgba, QUANT_COLOURS)
        if emissive:   # the flame's two colours survive the median cut (a small cluster is the first thing it merges)
            keep = np.all(np.abs(rgba[..., :3] - q[..., :3]) > 0, -1) & warm_mask(rgba[..., :3])
            q[keep, :3] = rgba[keep, :3]
        rgba = q
    return Image.fromarray(np.clip(rgba, 0, 255).astype(np.uint8), "RGBA")


def shelf_pack(items: list[tuple[str, Image.Image]], width: int) -> tuple[dict, int]:
    """Shelf packer: rows of decreasing height, GUTTER between everything."""
    frames: dict[str, dict] = {}
    x = y = GUTTER
    row_h = 0
    for key, im in sorted(items, key=lambda kv: (-kv[1].height, kv[0])):
        w, h = im.size
        if x + w + GUTTER > width:
            x = GUTTER
            y += row_h + GUTTER
            row_h = 0
        frames[key] = {"x": x, "y": y, "w": w, "h": h}
        x += w + GUTTER
        row_h = max(row_h, h)
    return frames, y + row_h + GUTTER


def main(argv: list[str]) -> int:
    quant = "--no-quant" not in argv
    manifest = json.loads((ROOT / "manifest.json").read_text())
    DST.mkdir(parents=True, exist_ok=True)

    items: list[tuple[str, Image.Image]] = []
    meta_sprites: dict[str, dict] = {}
    missing: list[str] = []
    for asset in manifest["assets"]:
        aid = asset["id"]
        src = SRC / f"{aid}.png"
        if asset["bg"].startswith("env"):
            continue  # register 3: art/make_env.py writes art/tiles/*env_*.png, packed below with the tiles
        if not src.exists():
            missing.append(aid)
            continue
        if asset["bg"] == "keyed":
            im = build_sprite(src, int(asset["master_h"]), quant, rim=aid.startswith(("hero_", "walk_")),
                              emissive=asset.get("kind") == "town")
            items.append((aid, im))
            meta_sprites[aid] = {"kind": asset["kind"], "texel_h": asset["texel_h"], "master_h": asset["master_h"]}
            print(f"  {aid:<20} {im.width}x{im.height}  keyed")
        else:
            w = int(asset.get("size_px_w", 1536))
            h = int(asset.get("size_px_h", 1024))
            im = Image.open(src).convert("RGB")
            if im.size != (w, h):
                im = im.resize((w, h), Image.Resampling.LANCZOS)
            im.save(DST / f"{aid}.png", optimize=True)
            print(f"  {aid:<20} {w}x{h}  bleed -> {aid}.png")

    tile_ids: list[str] = []
    for p in sorted(TILES.glob("*.png")):
        if p.name.startswith("_"):
            continue
        im = Image.open(p).convert("RGBA")
        items.append((p.stem, im))
        tile_ids.append(p.stem)
    print(f"  tiles: {len(tile_ids)} from {TILES}")

    frames, height = shelf_pack(items, ATLAS_W)
    atlas = Image.new("RGBA", (ATLAS_W, height), (0, 0, 0, 0))
    by_id = dict(items)
    for key, f in frames.items():
        atlas.paste(by_id[key], (f["x"], f["y"]))
    atlas.save(DST / "atlas.png")
    data = {
        "frames": frames,
        "meta": {
            "atlas": {"w": ATLAS_W, "h": height, "gutter": GUTTER},
            "tile": 8,
            "sprites": meta_sprites,
            "tiles": tile_ids,
            "palettes": {**PALETTES, "burrows": style_ramp("burrows"), "town": style_ramp("town"), "boss_flash": BOSS_FLASH},
        },
    }
    (DST / "atlas.json").write_text(json.dumps(data, indent=1, sort_keys=True) + "\n")
    print(f"packed {len(frames)} frames -> {DST / 'atlas.png'} ({ATLAS_W}x{height})")
    if missing:
        print(f"WARN: missing generated sources (renderer falls back to primitives): {', '.join(missing)}",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
