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

`--style <png>...` is the art direction's consistency check instead (docs/ART_DIRECTION.md §10): per image, the value
range, the shadow share, the warm cast, the blood accent's budget, the off-palette hues and the mean distance to the
named palette (+ the place's tint, read from the file name). Hard fails: contrast, cast, accent, off-palette.
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
PAINTED = set(json.loads((ROOT / "tiles/_painted.json").read_text())) if (ROOT / "tiles/_painted.json").exists() else set()   # gfx round 18
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

    optional = {a["id"] for a in assets if a.get("register") == "painted"}   # gfx round 18: a painted piece not yet drawn falls back to the ramp tile
    if optional - generated_ids:
        warnings.append(f"painted register: {len(optional - generated_ids)} pieces not drawn yet (the ramp register stands in)")
    town = {a["id"] for a in assets if a.get("kind") == "town"}   # art direction phase 2: the town's sprites (Cut 30) fall back to a block + icon
    if town - generated_ids:
        warnings.append(f"town: {len(town - generated_ids)} sprites not drawn yet (the town scene draws a block + its icon)")
    optional |= town
    missing_sources = sorted(manifest_ids - generated_ids - optional)
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
            if a.get("biome"):   # juice pass 2: a per-biome drawing converts to that biome's tile only
                if f"{a['biome']}_env_{a['name']}" not in tile_ids:
                    failures.append(f"{a['id']}: no converted tile (run art/make_env.py)")
                continue
            if not ({f"warrens_env_{name}", f"warrens_env_{name}_0", f"env_{name}", f"env_{name}_0"} & tile_ids):
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
            cap = 24 if fid in PAINTED else 8   # gfx round 18: the painted register (art/painted.py) keeps <= 24
            if len(colours) > cap:
                failures.append(f"{fid}: tile uses {len(colours)} colours (> {cap})")
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
        dist_m = np.sqrt((rgb[..., 0] - 255) ** 2 + rgb[..., 1] ** 2 + (rgb[..., 2] - 255) ** 2)   # phase 2's magenta key
        leaked = visible & ((dist <= 30) | (dist_m <= 30))
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


# docs/ART_DIRECTION.md §2 — keep in step with web/src/render/wash.ts WASH_PALETTE / WASH_TINT
STYLE_PALETTE = {
    "ink": "#0d0c14", "umbra": "#1c1b2b", "dusk": "#2b3350", "moon": "#4d6c99", "mist": "#a4bcd6", "bone": "#eadfc5",
    "blood": "#c01530", "clot": "#5c0b1c", "ember": "#e8923a", "gilt": "#b89448",
}
STYLE_TINT = {
    "warrens": "#4a3b2c", "burrows": "#5a4527", "fens": "#2d5752", "crypt": "#2f2c4f", "foundry": "#5a2a1e",
    "deep": "#1f2e4f", "sanctum": "#6b6048", "town": "#3a4a3a", "boss": "#2f2c4f",
}


def _lab(rgb: np.ndarray) -> np.ndarray:
    """sRGB 0..1 (…, 3) → CIE L*a*b* (D65)"""
    c = np.where(rgb <= 0.04045, rgb / 12.92, ((rgb + 0.055) / 1.055) ** 2.4)
    xyz = c @ np.array([[0.4124, 0.3576, 0.1805], [0.2126, 0.7152, 0.0722], [0.0193, 0.1192, 0.9505]]).T
    xyz /= np.array([0.95047, 1.0, 1.08883])
    f = np.where(xyz > 0.008856, np.cbrt(xyz), 7.787 * xyz + 16 / 116)
    return np.stack([116 * f[..., 1] - 16, 500 * (f[..., 0] - f[..., 1]), 200 * (f[..., 1] - f[..., 2])], -1)


def _hexlab(h: str) -> np.ndarray:
    return _lab(np.array([int(h[i:i + 2], 16) / 255 for i in (1, 3, 5)]))


def style_check(paths: list[str]) -> int:
    fails = 0
    pal_hex = dict(STYLE_PALETTE)
    for path in paths:
        name = Path(path).stem
        tint = next((v for k, v in STYLE_TINT.items() if k in name), None)
        pal = {**pal_hex, **({"tint": tint} if tint else {})}
        im = Image.open(path).convert("RGB")
        im.thumbnail((384, 384), Image.BOX)
        lab = _lab(np.asarray(im, dtype=np.float64) / 255).reshape(-1, 3)
        L, a, b = lab[:, 0], lab[:, 1], lab[:, 2]
        C = np.hypot(a, b)
        H = np.degrees(np.arctan2(b, a)) % 360
        pl = np.stack([_hexlab(h) for h in pal.values()])
        dist = np.sqrt(((lab[:, None, :] - pl[None, :, :]) ** 2).sum(-1)).min(1)
        def hue(h: str) -> float:
            v = _hexlab(h); return float(np.degrees(np.arctan2(v[2], v[1])) % 360)
        allowed = [hue(pal[k]) for k in ("blood", "clot", "ember", "gilt", "moon", "mist", "dusk", "umbra")] + ([hue(tint)] if tint else [])
        hd = np.min([np.abs((H - h0 + 180) % 360 - 180) for h0 in allowed], axis=0)
        chroma = C > 22
        off = float((chroma & (hd > 24)).mean())
        bh = hue(pal["blood"])
        blood = float(((C > 30) & (np.abs((H - bh + 180) % 360 - 180) < 18)).mean())
        p5, p99 = np.percentile(L, [5, 99])
        shadow = float((L < 25).mean())
        mid = (L > 20) & (L < 75)
        cast = float(b[mid].mean()) if mid.any() else 0.0
        sheet = "sheet" in name
        checks = {
            "contrast": (p5 <= 14 and p99 >= 62, f"L* p5 {p5:.0f} p99 {p99:.0f} (≤14 · ≥62: INK to MIST)"),
            "cast": (cast <= 12, f"mid-tone b* {cast:+.1f} (≤ +12: no warm brown cast)"),
            "accent": (0.002 <= blood <= 0.12, f"blood {blood * 100:.1f}% (0.2–12%)"),
            "off-palette": (off <= 0.15, f"off-hue chroma {off * 100:.1f}% (≤15%)"),
        }
        warn = {
            "shadow": (sheet or shadow >= 0.4, f"L*<25 {shadow * 100:.0f}% (≥40% on a frame)"),
            "palette ΔE": (float(dist.mean()) <= 14, f"mean ΔE to the palette {dist.mean():.1f} (≤14)"),
        }
        bad = [k for k, (ok, _) in checks.items() if not ok]
        fails += bool(bad)
        print(f"{'FAIL' if bad else 'PASS'} {name}" + (f"  (tint {tint})" if tint else ""))
        for k, (ok, msg) in {**checks, **warn}.items():
            print(f"   {'ok ' if ok else ('BAD' if k in checks else 'warn')} {k:12} {msg}")
    print(f"style QC: {len(paths)} images, {fails} failing")
    return 1 if fails else 0


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--style":
        raise SystemExit(style_check(sys.argv[2:]))
    raise SystemExit(main())
