#!/usr/bin/env python3
"""The painted 16x16 tile register (the owner's call, gfx round 18): per biome ~15 pieces painted by Codex in the target mockups'
lit-stone language (art/ui/targets/watch.png), converted to 16x16 at full colour (<= PAINT_COLOURS per tile, no 8-colour ramp) and
written over the ramp register's `art/tiles/<biome>_env_<name>.png`, so the renderer draws them 1:1 with the sprites (a tile is 16
texels on the sprite grid, an 8-env-texel world quad) with no id change; the ramp register stays the fallback for any piece not painted.

    python3 art/painted.py prompts [biome ...]   # writes art/prompts/paint_<biome>.txt (one Codex batch per biome)
    python3 art/painted.py manifest              # adds the `envp_<biome>_<name>` entries to art/manifest.json (idempotent)
    (conversion runs inside art/make_env.py: `convert_painted()`)
"""
from __future__ import annotations

import json
import sys
from pathlib import Path

import numpy as np
from PIL import Image

ROOT = Path(__file__).resolve().parent
PAINT_COLOURS = 24   # per tile (art-qc allows this for the painted register, `_painted.json`)

# the pieces: name -> (bg, texels, what) — the drawing brief shared by every biome; the biome adds its materials
PIECES: dict[str, tuple[str, str, str]] = {
    "floor_0": ("env", "16x16", "the common FLOOR: one large square flagstone filling the tile, its four edges a thin dark mortar line (about 1/16 of the width), the stone face worn smooth with gentle value variation, lit a touch brighter toward the upper-left"),
    "floor_1": ("env", "16x16", "a FLOOR variant: the same size stone split into two or three slabs by dark mortar lines, one small chip or crack"),
    "floor_2": ("env", "16x16", "a FLOOR variant: the common flagstone with a creeping patch of moss / grime in one corner (a quarter of the tile at most)"),
    "floor_3": ("env", "16x16", "a FLOOR variant: the common flagstone, a little darker and more worn, a faint old stain"),
    "wall_face_0": ("env", "16x16", "the FRONT FACE of a wall seen from a top-down three-quarter camera: the top quarter is the lit CAPSTONE LEDGE (a bright edge running the full width), below it three courses of dressed stone blocks with dark mortar, darkening toward the bottom; seamless left-to-right"),
    "wall_face_1": ("env", "16x16", "a WALL FACE variant: the same capstone ledge and block courses, one block cracked or missing a corner, a trickle of grime; seamless left-to-right, same ledge height as the other face"),
    "wall_top": ("env", "16x16", "the TOP of a thick wall seen straight down, exactly like the target's wall tops: a grid of 2x2 heavy dressed CAPSTONE BLOCKS, each block lit on its upper-left edge and shadowed on its lower-right, MID-GREY stone a little LIGHTER than the floor (the walls must read as solid built masonry around the rooms, never as black void), dark mortar between the blocks; seamless in every direction"),
    "door": ("env", "16x16", "a DOORWAY front-on in a wall: a stone arch framing a dark IRON PORTCULLIS (a grid of black bars over deep shadow), the arch stones lit on their upper-left"),
    "stairs_down": ("env", "16x16", "STAIRS GOING DOWN seen from above: four stone steps descending away into black darkness at the bottom, each step's lip lit, a stone frame on both sides"),
    "stairs_up": ("env", "16x16", "STAIRS GOING UP seen from above: four stone steps rising toward the viewer, the top step brightest, a stone frame on both sides"),
    "water": ("env", "16x16", "still dark WATER filling the tile: deep, calm, a few soft lighter ripple strokes, no hard edges; seamless in every direction"),
    "chasm": ("env", "16x16", "a CHASM: an almost black void falling away, the faintest rock texture deep inside; seamless in every direction"),
    "barrel": ("env_keyed", "16x16", "a wooden BARREL standing upright, its round lid visible on top, two dark iron hoops, lit upper-left; the object only, on flat pure blue #0000FF"),
    "crate": ("env_keyed", "16x16", "a wooden CRATE with a diagonal cross-brace and iron corner plates, lit upper-left; the object only, on flat pure blue #0000FF"),
    "pot": ("env_keyed", "16x16", "a clay POT / urn with a dark mouth and a cracked lip, lit upper-left; the object only, on flat pure blue #0000FF"),
    # gfx round 21 (raters: "rooms are bare grey tile fields"): the dressing — the renderer's prop names, painted per biome
    "skulls": ("env_keyed", "16x16", "a low PILE OF SKULLS AND BONES, four or five skulls with dark eye sockets and a few long bones, lit upper-left; the object only, on flat pure blue #0000FF"),
    "chest": ("env_keyed", "16x16", "a closed TREASURE CHEST with iron bands and a lock plate, domed lid, lit upper-left; the object only, on flat pure blue #0000FF"),
    "rack": ("env_keyed", "16x20", "a WEAPON RACK standing against a wall, front view: two spears and a sword upright in a wooden frame, a round shield leaning at its foot; the object only, on flat pure blue #0000FF"),
    "statue": ("env_keyed", "16x24", "a small broken STATUE on a square plinth, front view: a hooded kneeling knight carved in the biome's stone, one arm broken off; the object only, on flat pure blue #0000FF"),
    "bones": ("env_keyed", "16x16", "a scatter of old BONES and a cracked skull lying flat on the floor, seen from above; the object only, on flat pure blue #0000FF"),
}

# per biome: the materials and light; the Burrows and the Fens (the D5 fork) stay distinct at a glance
BIOMES: dict[str, str] = {
    "warrens": "EXACTLY the target watch.png dungeon: grey-green dressed flagstones with olive moss in the seams, charcoal mortar, dried blood specks here and there, warm amber torchlight on grey stone. Walls: grey stone blocks with a pale capstone ledge.",
    "burrows": "a dug-out BURROW, warm and earthy (nothing grey): packed ochre-brown earth floors with pebbles and root tendrils instead of flagstones (the floor 'stone' is a slab of packed earth), walls of rough sandstone blocks shored with REDDISH TIMBER beams, a warm orange-brown palette.",
    "fens": "a drowned FEN, cool and wet (nothing ochre): the floor is a sunken BOARDWALK of weathered grey-green planks with dark teal bog water showing between them and reed tufts at the edges; walls of mossy piled stones and rotten logs; a cold teal-green palette with pale grey wood.",
    "crypt": "a CRYPT: cold blue-grey granite flagstones with carved borders, walls of tomb-niche blocks with engraved runes, faint violet shadows, bone-ivory accents.",
    "foundry": "a FOUNDRY: soot-black iron floor plates with rivets and scorched brick, walls of blackened brick with glowing orange seams and rust streaks, a hot orange-and-charcoal palette.",
    "deep": "the DEEP caves: wet blue-black cave rock floors, rounded stone walls glistening with moisture, tiny bioluminescent cyan fungus specks, an abyssal navy-and-slate palette.",
    "sanctum": "a SANCTUM: pale cream marble flagstones with thin gold inlay lines, walls of polished pale stone with gilt trim, a calm ivory-and-gold palette (keep the values mid, never pure white).",
}

HEADER = """You are painting ENVIRONMENT TILES for the game "Riddle" (a phone roguelike seen top-down). FIRST look at
/home/g/p/riddle/art/ui/targets/watch.png (you may read that one image; read nothing else in the repository): its dungeon is
the TARGET LOOK — painterly dark-fantasy pixel art, lit stone rooms, moss, torchlight, crisp readable forms.

Use your built-in image_gen tool, one call per tile below. Do NOT write code beyond a PIL resample/alpha fix. Do NOT touch
anything outside /home/g/p/riddle/art/generated/. Do not leave _inspection_*.png or any other scratch file behind. If a file
exists at an output path, OVERWRITE it.

HOW THE TILES ARE USED (critical): each 1024x1024 image is box-downscaled to a 16x16 tile and drawn tiled edge to edge in a
top-down room, beside hand-painted characters about 1.5 tiles tall. So:
- Paint each tile as if it were a 16x16 pixel-art tile blown up 64x: BOLD shapes, 2-4 large forms, mortar / seam lines about
  64 px thick at 1024, no fine hairline detail, no text, no frame, no border, no vignette, no drop shadow.
- Opaque tiles fill the whole square edge to edge and must TILE SEAMLESSLY with themselves (floors and wall tops in every
  direction, wall faces left-to-right). Keep the floors' overall value and hue the same across variants so they mix.
- Floors are MID-DARK and calm (the characters must read on top of them); the wall tops are built masonry a little lighter than
  the floor (never black); the wall faces' ledge is the brightest line in the tile set.
- Light from the upper-left, warm; no strong cast shadows.
- Keyed props (barrel, crate, pot): the object only, centred, filling about 70% of the tile, on a flat uniform pure blue
  #0000FF background, no ground shadow, no blue on the object.
Save each as a 1024x1024 PNG at the exact path.

BIOME: {biome_desc}

=== TILES ===
"""


def ids(biome: str) -> list[str]:
    return [f"envp_{biome}_{n}" for n in PIECES]


def write_prompts(biomes: list[str]) -> None:
    for b in biomes:
        lines = [HEADER.format(biome_desc=BIOMES[b])]
        for i, (n, (bg, tx, what)) in enumerate(PIECES.items(), 1):
            lines.append(f"{i}) /home/g/p/riddle/art/generated/envp_{b}_{n}.png — {what}.")
        (ROOT / "prompts" / f"paint_{b}.txt").write_text("\n".join(lines) + "\n")
        print(f"art/prompts/paint_{b}.txt ({len(PIECES)} tiles)")


def add_manifest() -> None:
    p = ROOT / "manifest.json"
    m = json.loads(p.read_text())
    have = {a["id"] for a in m["assets"]}
    n = 0
    for b, bd in BIOMES.items():
        for name, (bg, tx, what) in PIECES.items():
            aid = f"envp_{b}_{name}"
            if aid in have:
                continue
            m["assets"].append({"id": aid, "kind": "env_tile" if bg == "env" else "env_prop", "bg": bg, "gen": "1024x1024", "texels": tx,
                                "biome": b, "name": name, "register": "painted", "description": f"{what}. Biome: {bd}"})
            n += 1
    p.write_text(json.dumps(m, indent=2, ensure_ascii=False) + "\n")
    print(f"manifest: +{n} painted entries")


def convert(src: Path, bg: str, texels: tuple[int, int], key_source) -> Image.Image:
    """1024 painting -> a 16x16 tile at full colour: box (area) downscale, a light local-contrast lift (a box average softens the
    forms), <= PAINT_COLOURS colours; keyed props: keyed, cropped to alpha, fitted bottom-centred, a 1-texel dark rim."""
    tw, th = texels
    if bg == "env":
        im = Image.open(src).convert("RGB").resize((tw, th), Image.Resampling.BOX)
        a = np.asarray(im, np.float32)
        m = a.mean(axis=(0, 1), keepdims=True)
        a = np.clip(m + (a - m) * 1.12, 0, 255)
        im = Image.fromarray(a.astype(np.uint8), "RGB").quantize(PAINT_COLOURS, method=Image.Quantize.MEDIANCUT).convert("RGBA")
        return im
    rgba = key_source(src)
    m = rgba[..., 3] > 16
    # (stray specks and dashes Codex leaves on the key are not the object): the band of rows, then of columns, holding the most pixels
    def band(counts: np.ndarray) -> tuple[int, int]:
        on = counts > 0; best, cur, bs, s0 = (0, len(counts) - 1), 0, -1, 0
        for i, v in enumerate(list(on) + [False]):
            if v and cur == 0: s0 = i
            if v: cur += int(counts[i])
            elif cur: 
                if cur > bs: bs, best = cur, (s0, i - 1)
                cur = 0
        return best
    r0, r1 = band(m.sum(1)); m2 = m[r0:r1 + 1]; c0, c1 = band(m2.sum(0))
    rgba = rgba[r0:r1 + 1, c0:c1 + 1].copy()
    lab = rgba[..., 3] > 16
    rgba[..., 3] = np.where(lab, rgba[..., 3], 0)
    h, w = rgba.shape[:2]
    s = min((tw - 2) / w, (th - 1) / h)
    nw, nh = max(1, round(w * s)), max(1, round(h * s))
    obj = Image.fromarray(rgba.astype(np.uint8), "RGBA").resize((nw, nh), Image.Resampling.BOX)
    o = np.asarray(obj).copy()
    o[..., 3] = np.where(o[..., 3] >= 128, 255, 0)
    out = np.zeros((th, tw, 4), np.uint8)
    x0, y0 = (tw - nw) // 2, th - nh
    out[y0:y0 + nh, x0:x0 + nw] = o
    solid = out[..., 3] > 0
    rim = np.zeros_like(solid)
    pad = np.pad(solid, 1)   # (np.roll wrapped the bottom row's rim onto the top row: a stray dash over every prop)
    rim |= pad[1:-1, 2:] | pad[1:-1, :-2] | pad[2:, 1:-1] | pad[:-2, 1:-1]
    rim &= ~solid
    out[rim] = (14, 10, 8, 255)
    im = Image.fromarray(out, "RGBA")
    q = im.convert("RGB").quantize(PAINT_COLOURS - 1, method=Image.Quantize.MEDIANCUT).convert("RGBA")
    qa = np.asarray(q).copy(); qa[..., 3] = out[..., 3]
    return Image.fromarray(qa, "RGBA")


def convert_all(src_dir: Path, out_dir: Path, key_source) -> list[str]:
    m = json.loads((ROOT / "manifest.json").read_text())
    written = []
    for a in m["assets"]:
        if a.get("register") != "painted":
            continue
        src = src_dir / f"{a['id']}.png"
        if not src.exists():
            continue
        tw, th = (int(v) for v in a["texels"].split("x"))
        tid = f"{a['biome']}_env_{a['name']}"
        convert(src, a["bg"], (tw, th), key_source).save(out_dir / f"{tid}.png")
        written.append(tid)
    (out_dir / "_painted.json").write_text(json.dumps(sorted(written)) + "\n")
    return written


if __name__ == "__main__":
    cmd = sys.argv[1] if len(sys.argv) > 1 else ""
    if cmd == "prompts":
        write_prompts(sys.argv[2:] or list(BIOMES))
    elif cmd == "manifest":
        add_manifest()
    else:
        print(__doc__)
