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
Tiles, overlays and item glyphs from art/tiles/*.png (8x8 RGBA) copy through.
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
from make_tiles import BOSS_FLASH, PALETTES  # noqa: E402  single source of truth for ramps

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
ATLAS_W = 512


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
    dist = np.sqrt(r * r + g * g + (b - 255.0) ** 2)
    alpha = np.clip((dist - LO) / (HI - LO), 0.0, 1.0)
    spill = alpha < 1.0
    b = np.where(spill, np.minimum(b, np.maximum(r, g)), b)
    return np.dstack([r, g, b, alpha * 255.0])


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


def build_sprite(src: Path, master_h: int, quant: bool) -> Image.Image:
    rgba = key_source(src)
    rgba = crop_to_alpha(rgba)
    rgba = resample_master(rgba, master_h)
    rgba = outline(rgba)
    if quant:
        rgba = quantise(rgba, QUANT_COLOURS)
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
        if not src.exists():
            missing.append(aid)
            continue
        if asset["bg"] == "keyed":
            im = build_sprite(src, int(asset["master_h"]), quant)
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
            "palettes": {**PALETTES, "boss_flash": BOSS_FLASH},
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
