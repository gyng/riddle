#!/usr/bin/env python3
"""gfx round 10: killer portraits — each monster's and boss's painted master (art/generated/<id>.png, Codex, already QC'd for the atlas)
cut to its head and shoulders on a dark vignette, 96 px (48 CSS px at 2x) → web/public/ui/foes/<kind>.webp; web/src/ui/skin.json `foes`
lists them (the client references only listed files: art never blocks). Re-run after art lands:  python3 tools/foe-portraits.py
"""
import json, sys
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "art"))
from pack import key_source, crop_to_alpha  # noqa: E402

OUT = ROOT / "web/public/ui/foes"
MANIFEST = ROOT / "web/src/ui/skin.json"
PX = 96


def portrait(src: Path) -> Image.Image:
    rgba = crop_to_alpha(key_source(src))
    h, w = rgba.shape[:2]
    # the head and shoulders: the top of the figure, a square as wide as ~70 % of its width (or its height's upper half)
    side = int(min(w, h) * 0.72)
    a = rgba[:, :, 3] > 16
    cols = np.where(a[: max(1, side // 2)].any(axis=0))[0]
    cx = int(cols.mean()) if cols.size else w // 2
    x0 = max(0, min(w - side, cx - side // 2))
    y0 = 0
    if w > 1.15 * h:   # a beast lying long (jackal, rat, eel): it faces right — its head is the right end
        side = int(h * 0.95); x0 = max(0, w - side); y0 = max(0, (h - side) // 2)
    crop = Image.fromarray(rgba[y0:y0 + side, x0:x0 + side].astype(np.uint8), "RGBA")
    bg = Image.new("RGBA", crop.size, (0, 0, 0, 255))
    d = ImageDraw.Draw(bg)
    for i in range(24, 0, -1):   # a warm dark vignette behind the face
        t = i / 24
        r = int(side * 0.75 * t)
        c = (int(18 + 44 * (1 - t)), int(12 + 26 * (1 - t)), int(8 + 14 * (1 - t)), 255)
        d.ellipse([side // 2 - r, int(side * 0.42) - r, side // 2 + r, int(side * 0.42) + r], fill=c)
    bg = bg.filter(ImageFilter.GaussianBlur(side / 40))
    bg.alpha_composite(crop)
    return bg.convert("RGB").resize((PX, PX), Image.LANCZOS)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    man = json.loads((ROOT / "art/manifest.json").read_text())
    foes = []
    for a in man["assets"]:
        if a["kind"] not in ("monster", "boss", "summon") or a["id"].endswith("_dead"):
            continue
        src = ROOT / "art/generated" / f"{a['id']}.png"
        if not src.exists():
            continue
        kind = a["id"].removeprefix("boss_")
        portrait(src).save(OUT / f"{kind}.webp", quality=86, method=6)
        foes.append(kind)
    skin = json.loads(MANIFEST.read_text())
    skin["foes"] = sorted(foes)
    MANIFEST.write_text(json.dumps(skin, indent=1) + "\n")
    print(f"foe-portraits: {len(foes)} -> web/public/ui/foes/")


if __name__ == "__main__":
    main()
