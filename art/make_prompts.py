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
    5: ["hero_ranger", "hero_caster"],
    # Cut 3: biomes 4-6, one batch per biome (monsters + that biome's boss)
    6: ["iron_golem", "forge_imp", "bell_sentinel", "slag_crawler", "smith", "boss_foundry_master"],
    7: ["lurker", "deep_eel", "cave_troll", "siren", "mirror_shade", "boss_lurker_queen"],
    8: ["warden", "acolyte", "echo", "sentinel", "boss_mirror_king"],
    # Art pass (2026-09-24): keyed register v2, the heroes and the D1-D8 cast first
    14: ["hero_fighter", "hero_rogue", "hero_ranger", "hero_caster", "ogre"],
    15: ["rat", "jackal", "goblin", "goblin_archer", "goblin_conjurer"],
    16: ["goblin_captain", "monkey", "bloat", "boss_goblin_warlord"],
    # Art pass (2026-09-24): register 3, the 16x16 environment (art/make_env.py converts)
    10: ["env_floor_0", "env_floor_1", "env_floor_2", "env_floor_3", "env_wall_face_0", "env_wall_face_1", "env_wall_top", "env_door"],
    11: ["env_stairs_down", "env_stairs_up", "env_water", "env_chasm", "env_moss_0", "env_moss_1", "env_crack", "env_rubble"],
    12: ["env_blood_0", "env_blood_1", "env_torch", "env_barrel", "env_crate", "env_pot", "env_banner", "env_bones"],
}

HEADER = """You are generating painted CHARACTER SPRITE masters for the game "Riddle" (a phone roguelike seen
top-down). FIRST look at /home/g/p/riddle/art/ui/targets/watch.png (you may read that one image; read
nothing else in the repository): its hero (brown hair, crimson cape, sword) and its green goblins
(sword goblin, archer) are the TARGET LOOK for every sprite. Use your built-in image_gen tool, one
call per asset below. Do NOT write code beyond a PIL resample, do NOT touch anything outside
/home/g/p/riddle/art/generated/. Do not leave _inspection_*.png or any other scratch file behind.
If a file already exists at an output path, OVERWRITE it (these are regenerations in a new style).

For each asset: generate at the stated size, then save the final PNG (sRGB) to the exact output
path. If the image is not exactly the stated size, resample to it with PIL (Lanczos). Self-check
that all four corner pixels are pure blue (0,0,255) within distance 30 (or fully transparent) and
that the subject contains NO mid-blue pixels; regenerate that asset once if not.

=== STYLE (art/ART.md, keyed register v2) — the FIRST and loudest rule ===
A crisp top-down dark-fantasy RPG CHARACTER SPRITE, like the characters in watch.png, painted at
high resolution with a WATERCOLOUR-INSPIRED fill texture:
- Proportions: chunky and readable — the head about one third of the body height, stocky torso,
  short sturdy legs planted apart, hands and weapon drawn LARGE. A 3/4 view from slightly above,
  facing RIGHT.
- Outline: ONE thick, clean, continuous NEAR-BLACK outline around the whole silhouette (12-16 px at
  1024 px — it must survive a downscale to 40-64 px), plus thinner dark lines on the major interior
  forms. Hard, crisp edges everywhere. No soft or feathered edges, no loose bleeding washes.
- Shading: 3 tones per material — a lit tone, a mid tone, one shadow tone — with the key light from
  the UPPER-LEFT; strong value contrast between lit and shadow. Inside each flat tone a SUBTLE
  watercolour wash texture (gentle granulation / pigment variation, a few dry-brush flecks) — the
  fills must NOT be smooth digital gradients, but they are NOT loose wet washes either and no white
  paper shows through.
- Palette: grounded, earthy dark fantasy like watch.png — moss / olive greens, crimson and oxblood
  cloth, warm leather browns, dull steel greys, bone ivory — with one or two saturated accents.
  Clean hues, no muddy greys (the game tints sprites 30 % toward the dungeon's palette).
- Faces: small, simple, readable (two dark eyes, a brow line); heroes keep a gentle anime flavour
  (hair as a few big locks), monsters are grotesque but charming like watch.png's goblins.
- Never: text, letters, logos, signatures, watermarks, photorealism, 3D-render shading, airbrushed
  volume, glossy game-asset rendering, dense noisy detail, stair-stepped fake pixel art (the game
  downsamples; paint smooth crisp shapes), loose watercolour-anime illustration.

=== READABILITY (these sprites are shown TINY) ===
Each sprite is box-downscaled to 24-64 px tall on a phone. ONE big readable mass, one clear gesture,
a simple silhouette with no thin spindly extras. The creature's defining feature (its "tag", stated
per asset) must be visible in the SILHOUETTE alone. Full body, nothing cropped, subject centred with
~10% margin on every side.

=== CHROMA KEY ===
Background is a flat, uniform, edge-to-edge pure blue #0000FF = RGB(0,0,255), one solid fill. The
subject must stay far from that key: every pixel inside the subject at least RGB-distance 150 from
(0,0,255). Any blue-ish material uses CYAN/TEAL mid-tones (green channel high) or BLUE-BLACK depths
(all channels low). Ultramarine, royal blue, cobalt and sky blue are forbidden on the subject. No
blue glow, rim light or reflections. NO floor, NO ground plane, NO contact shadow, NO cast-shadow
pool, NO scenery, NO glow halo, NO pedestal — the subject floats on flat blue.

=== ASSETS ===
"""

ENV_HEADER = """You are generating ENVIRONMENT PIXEL ART for the game "Riddle" (a phone roguelike seen top-down).
FIRST look at /home/g/p/riddle/art/ui/targets/watch.png (you may read that one image; read nothing
else in the repository). Its dungeon is the target: top-down dark-fantasy PIXEL ART — dressed stone
walls with a lit capstone ledge and a brick front face, worn flagstone floors with moss and dried
blood, iron torch sconces, barrels, crates, bones, a portcullis, crimson banners. Match that
material language, that value range and that crisp pixel-art register.

Use your built-in image_gen tool, one call per asset below. Do NOT write code beyond a PIL
resample; do NOT touch anything outside /home/g/p/riddle/art/generated/. Do not leave
_inspection_*.png or any other scratch file behind. If a file already exists at an output path,
OVERWRITE it.

=== HOW THESE ARE USED (critical) ===
Each asset is a TINY pixel-art game tile: the game box-downscales the 1024 px image to the stated
texel grid (16x16, 16x24 or 16x32 pixels). So paint it AS that tiny pixel-art tile, enlarged:
- A crisp pixel grid of the stated size, every pixel a flat square block of one colour
  (e.g. 16x16 -> each pixel is a 64x64 px flat block). No anti-aliasing, no gradients inside a
  block, no blur, no painterly texture, no noise finer than one block.
- Few, clear shapes; strong value structure; a 1-block dark edge where an object meets the
  floor; key light from the UPPER-LEFT, always (lit top/left edges, shaded bottom/right edges).
- Top-down three-quarter camera like watch.png (you see floors from above, wall FRONT faces and
  object fronts slightly from above).
- Opaque TILES ("bg: env") fill the whole square edge to edge and must TILE SEAMLESSLY with
  copies of themselves (the edges continue across). Keep them muted and low-contrast: floors are
  background and must never compete with the characters that stand on them.
- The game recolours every tile into a per-dungeon 8-colour ramp by VALUE, so what matters most is
  a clean value structure (dark mortar, mid stone, a few lit edges). Paint environment tiles in
  neutral warm grey-olive stone like watch.png.
- Keyed decals/props ("bg: env_keyed"): the object on a flat, uniform, edge-to-edge pure blue
  #0000FF background (RGB 0,0,255), nothing else in the image; no floor under it, no cast
  shadow pool, no glow halo. Keep every object pixel at least RGB-distance 150 from (0,0,255): no
  blue anywhere on the object. Object centred; ~6% margin.
- Never: text, letters, logos, signatures, watermarks, photorealism, 3D render, smooth shading.

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


def env_block(i: int, a: dict) -> str:
    out = f"/home/g/p/riddle/art/generated/{a['id']}.png"
    tw, th = a["texels"].split("x")
    gw, gh = a["gen"].split("x")
    return (f"--- {i}. id: {a['id']}  ({a['kind']}, bg: {a['bg']})\n"
            f"Output path: {out}\n"
            f"Size: exactly {gw}x{gh} px PNG, painted as a {tw}x{th}-pixel pixel-art "
            f"{'tile' if a['bg'] == 'env' else 'sprite'} (in-game size {tw}x{th}); "
            f"{f'the object is tall and narrow ({tw}:{th}), centred, filling the height; blue key left and right' if tw != th else ''}\n"
            f"Brief: {a['description']}\n\n")


def build(n: int, ids: list[str]) -> Path:
    if all(BY_ID[i]["bg"].startswith("env") for i in ids):
        body = ENV_HEADER + "".join(env_block(i + 1, BY_ID[x]) for i, x in enumerate(ids))
        body += "When all assets are saved, list the final paths and their pixel sizes. Do nothing else.\n"
        p = ROOT / "prompts" / f"batch{n}.txt"
        p.write_text(body)
        return p
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
