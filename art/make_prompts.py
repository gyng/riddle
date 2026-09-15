#!/usr/bin/env python3
"""Build codex batch prompts (art/prompts/batchN.txt) from art/manifest.json.

Usage: python3 art/make_prompts.py [N id id id ...]   (no args = default batches)
Each prompt embeds the ART.md preamble, the chroma-key colour science, exact output
paths and sizes, and the "touch nothing outside art/generated/" rule.
"""
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
MAN = json.loads((ROOT / "manifest.json").read_text())
BY_ID = {a["id"]: a for a in MAN["assets"]}

DEFAULT_BATCHES = {
    1: ["hero_fighter", "hero_rogue", "rat", "jackal", "goblin", "goblin_archer"],
    2: ["goblin_conjurer", "monkey", "ogre", "bloat", "pink_jelly", "eel"],
    3: ["skeleton", "ghoul", "wraith", "captive", "spectral_blade", "spectral_hound"],
    4: ["boss_goblin_warlord", "boss_bloat_mother", "boss_lich", "title"],
}

HEADER = """You are generating painted sprite masters for the game "Riddle" (a phone roguelike).
Use your built-in image_gen tool, one call per asset below. Do NOT write code, do NOT touch
anything outside /home/g/p/riddle/art/generated/. Do not read or modify any other file in the
repository. Do not leave _inspection_*.png or any other scratch file behind. If a file already
exists at an output path, OVERWRITE it (these are regenerations).

For each asset: generate at the stated size, then save the final PNG (sRGB, RGB or RGBA) to the
exact output path given. If the generated image is not exactly the stated size, resample to it
with PIL (Lanczos). For keyed sprites, self-check that all four corner pixels are pure blue
(0,0,255) within distance 30 and that the subject contains NO mid-blue pixels; if the corners are
not blue or the subject contains royal/ultramarine blue, regenerate that asset once.

=== STYLE PREAMBLE (art/ART.md, keyed register) — the FIRST and loudest rule ===
Loose 1980s Japanese watercolour-anime key art painted INSIDE bold, clean, confident dark ink
contour lines (the anime cel principle). The previous project round failed twice: once by being
"too rendered" (tight airbrushed detail, game-asset gloss, no white paper), once by "loose wet
formless blobs with no outline". Do neither. Every sprite MUST have a continuous dark ink outline
around its outer silhouette and its major interior forms, line weight 4-8 px at 1024 px.
- Medium: loose transparent watercolour and gouache on cold-press paper, washes left wet and
  unblended, pigment blooms, granulation, dry-brush edges, white/ivory paper showing through in
  every lit area. Large simple wash shapes. ONE single flat indigo-umber shadow mass on the
  shadow side. Pick 3-4 signature details and drop the rest.
- Light: single key light from the UPPER-LEFT, always. Lit side near paper-white.
- Era: 1980s OVA / anime-movie box art energy. Heroes and the captive are unmistakably ANIME:
  expressive anime faces, large eyes, hair as flowing locks and masses. Monsters are anime
  bestiary creatures. NOT western storybook, NOT D&D illustration, NOT concept-art realism.
- Palette: muted field, jewel-bright accents; clean hues (no muddy mid-greys), high value
  contrast (paper-white vs deep shadow) because the sprite is tinted toward a dungeon palette later.
- Never: text, letters, logos, signatures, watermarks, photorealism, airbrushed volume, dense
  noisy detail, 3D-render shading, modern digital gradients, pixel art, video-game-asset gloss.

=== READABILITY (these sprites are shown TINY) ===
Each sprite is box-downscaled to 24-64 px tall on a phone. So: ONE big readable mass, one clear
gesture, a simple silhouette with no thin spindly extras. The creature's defining feature (its
"tag", stated per asset) must be visible in the SILHOUETTE alone. Full body, nothing cropped,
subject centred with ~10% margin on every side, FACING RIGHT (the game mirrors for left).

=== CHROMA KEY (keyed sprites) ===
Background is a flat, uniform, edge-to-edge pure blue #0000FF = RGB(0,0,255), painted as one solid
fill (no paper texture, no vignette, no gradient in the background). The subject must stay far
from that key: every pixel inside the subject must be at least RGB-distance 150 from (0,0,255).
Any blue-ish material on the subject uses CYAN/TEAL mid-tones (green channel high) or BLUE-BLACK
depths (all channels low). Ultramarine, royal blue, cobalt and sky blue are forbidden on the
subject. No blue glow, no blue rim light, no blue reflections. No matte spill.
NO floor, NO ground plane, NO contact shadow, NO cast-shadow pool, NO scenery, NO glow halo,
NO pedestal — the subject floats on flat blue and the game engine draws its own shadow.

=== ASSETS ===
"""

TITLE_RULES = """Register for this asset: BLEED key art (not keyed). Painted edge to edge, full bleed, no blue key,
no transparency. Same medium (loose 1980s anime watercolour key art with bold ink) but this one is a
composed scene. ABSOLUTELY NO TEXT, letters, logotype or title anywhere in the image.
"""


def asset_block(i: int, a: dict) -> str:
    out = f"/home/g/p/riddle/art/generated/{a['id']}.png"
    w, h = a["gen"].split("x")
    lines = [f"--- {i}. id: {a['id']}  ({a['kind']})",
             f"Output path: {out}",
             f"Size: exactly {w}x{h} px (width x height), PNG."]
    if a["bg"] == "keyed":
        lines.append(f"Shown in game at about {a['texel_h']} px tall; judge readability at that size.")
        lines.append(f"Background: flat pure blue #0000FF, keyed register (rules above).")
    else:
        lines.append(TITLE_RULES.rstrip())
    lines.append("Brief: " + a["description"])
    return "\n".join(lines) + "\n\n"


def build(n: int, ids: list[str]) -> Path:
    body = HEADER + "".join(asset_block(i + 1, BY_ID[i_]) for i, i_ in enumerate(ids))
    body += ("When all assets are saved, list the final paths and their pixel sizes. "
             "Do nothing else.\n")
    p = ROOT / "prompts" / f"batch{n}.txt"
    p.write_text(body)
    return p


if __name__ == "__main__":
    if len(sys.argv) > 2:
        n = int(sys.argv[1])
        print(build(n, sys.argv[2:]))
    else:
        for n, ids in DEFAULT_BATCHES.items():
            print(build(n, ids))
