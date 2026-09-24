#!/usr/bin/env python3
"""Deterministic shipping gates for Riddle's raster art (adapted from tacticalswap).

Run after `python3 art/pack.py`. Technical integrity only; the 48 px
composite review (`python3 art/review.py`) stays mandatory.

Fails on: manifest/source/atlas drift (missing sources, unregistered PNGs in
art/generated/, keyed ids missing from atlas.json, tile files missing from
the atlas, stale frames), keyed source short edge < 1024, title dimensions,
frame dimensions (keyed frame height == master_h, tiles 8x8, frames inside
the atlas, no overlapping frames, gutters), residual pure-key pixels in any
frame, blue semi-transparent spill, empty or implausible frame alpha, tile
colour counts > 8, and atlas.png/atlas.json being older than their sources.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
GENERATED = ROOT / "generated"
TILES = ROOT / "tiles"
PUBLISHED = ROOT.parent / "web" / "public" / "art"


def main() -> int:
    manifest = json.loads((ROOT / "manifest.json").read_text())
    assets = manifest["assets"]
    failures: list[str] = []
    warnings: list[str] = []

    ids = [a["id"] for a in assets]
    dupes = sorted({i for i in ids if ids.count(i) > 1})
    if dupes:
        failures.append(f"duplicate manifest ids: {', '.join(dupes)}")
    manifest_ids = set(ids)
    keyed_ids = {a["id"] for a in assets if a["bg"] == "keyed"}
    generated_ids = {p.stem for p in GENERATED.glob("*.png")}
    tile_ids = {p.stem for p in TILES.glob("*.png") if not p.name.startswith("_")}

    missing_sources = sorted(manifest_ids - generated_ids)
    if missing_sources:
        failures.append(f"missing generated sources: {', '.join(missing_sources)}")
    orphans = sorted(generated_ids - manifest_ids)
    if orphans:
        failures.append(f"unregistered generated sources: {', '.join(orphans)}")
    if not tile_ids:
        failures.append("no tiles in art/tiles (run make_tiles.py)")

    atlas_png = PUBLISHED / "atlas.png"
    atlas_json = PUBLISHED / "atlas.json"
    if not atlas_png.exists() or not atlas_json.exists():
        failures.append("missing web/public/art/atlas.png or atlas.json (run pack.py)")
        return report(failures, warnings, 0)
    data = json.loads(atlas_json.read_text())
    frames: dict[str, dict] = data["frames"]
    atlas = np.asarray(Image.open(atlas_png).convert("RGBA"))
    ah, aw = atlas.shape[:2]

    # freshness: atlas must not be older than any source it packs
    newest_src = max([p.stat().st_mtime for p in list(GENERATED.glob("*.png")) + list(TILES.glob("*.png"))] + [0])
    if atlas_png.stat().st_mtime < newest_src:
        failures.append("atlas.png is older than art/generated or art/tiles (re-run pack.py)")

    # register 3: every env source present must have been converted (its warrens tile or its hue asset)
    for a in assets:
        if a["bg"].startswith("env") and a["id"] in generated_ids:
            name = a["id"][4:]
            if not ({f"warrens_env_{name}", f"env_{name}", f"env_{name}_0"} & tile_ids):
                failures.append(f"{a['id']}: no converted tile (run art/make_env.py)")
    expected_frames = (keyed_ids & generated_ids) | tile_ids
    missing_frames = sorted(expected_frames - set(frames))
    if missing_frames:
        failures.append(f"atlas.json missing frames: {', '.join(missing_frames)}")
    stale = sorted(set(frames) - expected_frames)
    if stale:
        failures.append(f"stale atlas frames: {', '.join(stale)}")

    by_id = {a["id"]: a for a in assets}
    boxes: list[tuple[str, int, int, int, int]] = []
    checked = 0
    for fid, f in frames.items():
        x, y, w, h = int(f["x"]), int(f["y"]), int(f["w"]), int(f["h"])
        if x < 0 or y < 0 or x + w > aw or y + h > ah:
            failures.append(f"{fid}: frame outside atlas")
            continue
        for oid, ox, oy, ow, oh in boxes:
            if x < ox + ow + 1 and ox < x + w + 1 and y < oy + oh + 1 and oy < y + h + 1:
                failures.append(f"{fid}: overlaps or lacks a 1-px gutter against {oid}")
                break
        boxes.append((fid, x, y, w, h))
        fr = atlas[y : y + h, x : x + w]
        alpha = fr[..., 3]
        rgb = fr[..., :3].astype(np.float32)
        checked += 1
        if fid in tile_ids:
            env = fid.startswith("env_") or "_env_" in fid   # register 3 (art/make_env.py): 16 texels wide, <= 32 tall
            if (w, h) != (8, 8) and not (env and w == 16 and 16 <= h <= 32):
                failures.append(f"{fid}: tile frame {w}x{h}, expected {'16x16..16x32' if env else '8x8'}")
            colours = {tuple(p) for p in fr.reshape(-1, 4) if p[3] > 0}
            if len(colours) > 8:
                failures.append(f"{fid}: tile uses {len(colours)} colours (> 8)")
            if alpha.max() == 0:
                failures.append(f"{fid}: empty tile")
            continue
        a = by_id[fid]
        if h != int(a["master_h"]):
            failures.append(f"{fid}: frame height {h}, expected master_h {a['master_h']}")
        if w < 8 or w > 4 * h:
            failures.append(f"{fid}: implausible frame width {w} for height {h}")
        visible = alpha > 8
        frac = float(visible.mean())
        if not 0.15 <= frac <= 0.98:
            failures.append(f"{fid}: implausible visible-alpha coverage {frac:.1%}")
        if alpha.max() == 0:
            failures.append(f"{fid}: empty alpha")
        dist = np.sqrt(rgb[..., 0] ** 2 + rgb[..., 1] ** 2 + (rgb[..., 2] - 255) ** 2)
        leaked = visible & (dist <= 30)
        if leaked.any():
            failures.append(f"{fid}: {int(leaked.sum())} visible pure-key pixels remain")
        fringe = (alpha > 0) & (alpha < 250)
        spill = fringe & (rgb[..., 2] > np.maximum(rgb[..., 0], rgb[..., 1]) + 18)
        if spill.any():
            failures.append(f"{fid}: {int(spill.sum())} semi-transparent blue-spill pixels")
        # the outline ring: every visible pixel touching transparency must be dark ink
        p = np.pad(visible, 1)
        edge = visible & ~(p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:])
        lum = rgb[..., 0] * 0.2126 + rgb[..., 1] * 0.7152 + rgb[..., 2] * 0.0722
        bright_edge = edge & (lum > 90)
        if bright_edge.sum() > 0.02 * max(1, edge.sum()):
            failures.append(f"{fid}: {int(bright_edge.sum())} edge pixels are not dark ink (outline lost)")

    for a in assets:
        src = GENERATED / f"{a['id']}.png"
        if not src.exists():
            continue
        im = Image.open(src)
        if a["bg"] in ("keyed", "env", "env_keyed") and min(im.size) < 1024:
            failures.append(f"{a['id']}: source short edge {min(im.size)} < 1024")
        if a["bg"] == "bleed":
            pub = PUBLISHED / f"{a['id']}.png"
            want = (int(a.get("size_px_w", 1536)), int(a.get("size_px_h", 1024)))
            if not pub.exists():
                failures.append(f"{a['id']}: missing published {pub.name}")
            elif Image.open(pub).size != want:
                failures.append(f"{a['id']}: published {Image.open(pub).size}, expected {want}")

    return report(failures, warnings, checked)


def report(failures: list[str], warnings: list[str], checked: int) -> int:
    print(f"art QC: checked {checked} atlas frames; warnings {len(warnings)}; failures {len(failures)}")
    for w in warnings:
        print(f"WARN: {w}")
    for f in failures:
        print(f"FAIL: {f}", file=sys.stderr)
    if failures:
        return 1
    print("PASS: manifest parity, atlas frames, dimensions, gutters, alpha, key spill, outline, tile colours")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
