#!/usr/bin/env python3
"""Art direction phase 2: the UI, portraits, pets, items and the town's prompts under docs/ART_DIRECTION.md.

    python3 art/phase2_ui.py      # writes art/prompts/p2_*.txt (one Codex batch each) and prints their names

Ids and sizes are the ones the client already loads (tools/ui-skin.py, tools/foe-portraits.py, art/manifest.json), so a
regenerated file drops in; the town's new pieces go to art/ui/town/ (tools/ui-skin.py packs them).
"""
from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
from make_prompts import style_preamble  # noqa: E402

REFS = "/home/g/p/riddle/art/ui/targets/style/ui_sheet.png, /home/g/p/riddle/art/ui/targets/style/death.png, " \
       "/home/g/p/riddle/art/ui/targets/style/watch_warrens.png"


def head(what: str, outdir: str, refs: str = REFS, extra: str = "") -> str:
    return f"""You are painting {what} for the game "Riddle" (a gothic dark-fantasy roguelike on a phone held in portrait). FIRST look
at these approved STYLE TARGETS (you may open these images and nothing else in the repository): {refs}. They are the target look.

Use your built-in image_gen tool, one call per asset below. Do NOT write code beyond a PIL resample / alpha fix. Do NOT modify
anything outside {outdir}. Save each final PNG (sRGB) to the exact path at the exact size; resample with PIL (Lanczos) if needed,
keeping the aspect (crop the long side, never stretch). Delete any scratch or _inspection_ files you create. OVERWRITE existing
files. When done, look at each output once and regenerate it once if it breaks a rule below.

=== STYLE (docs/ART_DIRECTION.md, approved) ===
{style_preamble()}
{extra}
=== ASSETS ===
"""


ALPHA = """ALPHA: every asset marked RGBA has a TRUE TRANSPARENT background outside the object (the four canvas corners alpha 0, no
checkerboard, no key colour, no black or paper fill). OPAQUE assets fill the canvas edge to edge.
"""

UI_RULES = """=== UI RULES ===
- Iron and stone are INK and WASH: INK plates with hammered hatching, a MIST moonlit bevel along the TOP edge, GILT rivets
  (small, sparingly), never glossy metal, never chrome, never warm amber light on the edges. Parchment is BONE paper with
  watercolour bloom stains and an inked, deckled edge.
- Frames are EMPTY containers the game fills: no text, no letters, no numbers, no icons on any frame.
- NINE-SLICE (frames marked [9-slice, inset N]): CSS stretches them with that inset on all four sides, so every ornament (corner
  caps, rivet clusters) sits INSIDE the N-px corner squares, the edges between the corners are a plain band uniform along its
  length, and the centre is a plain low-contrast fill with no focal detail.
""" + ALPHA

FRAMES = [
    ("bar", "1024x128", "RGBA [9-slice, inset 40] the TOP BAR: a long blackened-iron plate drawn as ink and wash (INK body, hammered hatching, a MIST moonlit rim along its top edge, two small GILT rivets near each end, inside the corners); a flat dark centre"),
    ("console", "1024x512", "RGBA [9-slice, inset 96] the bottom CONSOLE: a slab of dark stone and blackened iron in INK/UMBRA washes with ink hatching, a MIST bevel along the top edge, heavy iron corner caps with a small BONE skull boss in each corner only; a plain dark centre"),
    ("panel", "512x512", "RGBA [9-slice, inset 56] a framed PANEL: a thin blackened-iron frame with small corner brackets around BONE parchment (watercolour bloom stains, an inked deckled edge just inside the frame); the parchment centre evenly lit and calm"),
    ("tablet", "1024x160", "RGBA [9-slice, inset 48] a RULE TABLET: a long slab of dark carved stone in UMBRA/DUSK washes with a thin iron border and a MIST rim on its top edge; a plain centre"),
    ("tablet_card", "1024x160", "RGBA [9-slice, inset 48] the same rule tablet with a worn GILT border (the recommended/card row) — gilt only on the border, the slab as dark as the plain tablet"),
    ("tile", "256x256", "RGBA [9-slice, inset 48] a square COMMAND TILE: a raised dark iron-and-stone tile, INK face with hatching, a MIST bevel on the top and left edges, a dark bottom edge; an empty face"),
    ("tile_pressed", "256x256", "RGBA [9-slice, inset 48] the same tile PRESSED: the bevel inverted (dark on top, a faint MIST line at the bottom), sunk, darker face"),
    ("well", "384x384", "RGBA [no slice] the PORTRAIT WELL: a round blackened-iron ring, ink-hatched, a thin GILT inner edge, four small rivets, a MIST glint on its top arc; the inside of the ring FULLY TRANSPARENT (a portrait shows through)"),
    ("gem", "384x384", "RGBA [no slice] the PRIMARY ACTION GEM (SEND): a big round faceted BLOOD-red gem #c01530 with CLOT depths and one pale MIST moonlit highlight on its upper facets, set in an ornate blackened-iron and GILT mount with four small points (like the SEND gem of ui_sheet.png) — but with NO TEXT on it (the game writes the word)"),
    ("gem_red", "384x384", "RGBA [no slice] the DANGER GEM (bail): the same mount holding a darker, smaller-faceted CLOT-red gem #5c0b1c with a thin crack across it and a dim BLOOD glint — clearly darker and more ominous than the primary gem; no text"),
    ("banner", "768x1024", "RGBA [no slice] the DEATH BANNER: a tall torn hanging banner of BLOOD-red cloth (CLOT in its folds, ink-drawn tears, ragged dripping lower edge) hanging from a spiked blackened-iron bar with a small BONE skull boss at the bar's centre and short chains at its ends; the cloth's centre plain (the words YOU DIED are laid over it by the game)"),
    ("gauge", "512x48", "RGBA [9-slice, inset 16] an empty GAUGE trough: a dark recessed iron channel in INK with a thin MIST rim along its top edge (the fill is drawn by code)"),
    ("stud", "96x96", "RGBA [no slice] a round iron CLOSE-STUD: a small blackened-iron boss with an engraved X cross in its face (the one exception to 'no marks'), a MIST glint at the top"),
    ("seal", "256x256", "RGBA [no slice] an empty round wax SEAL ring in BLOOD-red wax (CLOT shadows), its centre flat and empty (a word is written in it by code)"),
    ("button", "1024x256", "RGBA a wide horizontal carved BUTTON PLAQUE seen straight on, filling the canvas with a few px of margin: a raised slab of dark carved wood in UMBRA/DUSK washes, its face EMPTY and flat in the middle, a bevelled top edge lit cold MIST and a dark underside, a thin worn GILT moulding all round, and at each short end a blackened-iron end cap with a small round BLOOD gem and two rivets. The left and right 18 % hold the end caps; the middle 64 % is plain face and moulding only (it is stretched horizontally)"),
    ("scroll", "768x1152", "RGBA [9-slice, insets: top 160, bottom 160, left 72, right 72] a hanging PARCHMENT SCROLL seen straight on: a thick rolled curl of BONE parchment across the whole top edge and another across the whole bottom edge (their curled ends inside the left/right 72 px), the sheet between them BONE paper with watercolour bloom stains, an inked deckled edge darkening toward the sides; transparent outside the paper"),
]

ICONS = [
    ("edit", "a stone mason's chisel"), ("loadout", "a leather satchel"), ("unlocks", "an ornate old key"), ("ledger", "an open book"),
    ("vault", "an iron-bound chest"), ("forge", "an anvil"), ("party", "a paw print"), ("chronicle", "a rolled scroll"),
    ("fights", "two crossed swords"), ("fast", "a double chevron pointing right"), ("skip", "a play triangle against a vertical bar"),
    ("bail", "a small tattered flag on a pole"), ("pause", "two vertical bars"), ("play", "a play triangle"), ("morgue", "a skull"),
    ("camp", "a tent"), ("trace", "a magnifying lens over a scroll"), ("gold", "a small stack of coins (GILT washes)"),
    ("mark", "a faceted diamond (MIST washes, the one cold jewel)"), ("renown", "a five-point star"),
    ("depth", "a stairway descending into darkness"), ("settings", "a gear cog"),
    ("v_attack", "a single broad sword, point up, slightly diagonal"), ("v_drink", "a round glass potion flask with a cork (the liquid a small BLOOD fill)"),
    ("v_read", "a partly unrolled parchment scroll with a wax seal (no writing)"), ("v_throw", "a small clay flask with a burning rag in its neck (a small EMBER flame)"),
    ("v_retreat", "a heavy curved arrow bending back to the left, like a U-turn"), ("v_corridor", "a narrow stone archway seen from the front"),
    ("v_descend", "a short flight of stone stairs going down into a dark opening"), ("v_rest", "a small campfire of crossed logs with a low EMBER flame"),
    ("v_shoot", "a recurve bow with a nocked arrow"), ("v_magic", "a jagged lightning bolt inside a thin rune circle (MIST)"),
    ("v_shadow", "a hooded mask half dissolving into smoke"), ("v_shield", "a round iron-rimmed shield with a central boss"),
    ("v_explore", "an old hanging lantern with a small EMBER flame"),
]

ICON_RULES = """=== ICON RULES (ui_sheet.png's resource icons and command-tile icons) ===
Each icon is ONE object drawn as a BONE ink drawing: a confident INK contour, one line weight, BONE and MIST washes inside with
granulation, a hint of the pixel grid in its stepped edges, cold light from above. No gradients, no glossy metal, no warm amber,
no tile or frame behind it, no text, no letters, no numbers. It must read at 32-40 px: a strong simple silhouette filling ~75 % of
the canvas, centred. Only where the subject says so, one small accent: BLOOD, EMBER, GILT or MIST.
""" + ALPHA


def frames_prompt() -> str:
    body = head("UI SKIN FRAMES", "/home/g/p/riddle/art/ui/frames/", extra=UI_RULES)
    for i, (n, size, what) in enumerate(FRAMES, 1):
        body += f"{i}) /home/g/p/riddle/art/ui/frames/{n}.png — {size} — {what}.\n"
    return body


def icons_prompt(part: list[tuple[str, str]]) -> str:
    body = head("UI ICONS", "/home/g/p/riddle/art/ui/icons/", extra=ICON_RULES)
    for i, (n, what) in enumerate(part, 1):
        body += f"{i}) /home/g/p/riddle/art/ui/icons/{n}.png — 256x256 RGBA — {what}.\n"
    return body


SCENES = """=== SCENE RULES ===
Painted scenes are the style at full strength: ~60 % INK/UMBRA, cold moonlight from above as shapes (shafts, rims), one or two small
EMBER pools with a visible flame, BLOOD only where the subject says. Paper grain and halftone in the shadows. No text, no UI, no
frame, no faces seen close.
""" + ALPHA

BACK = [
    ("backdrops/death", "1024x1536", "OPAQUE — the spot where a lone hero fell in the dungeon: a cracked flagstone floor at a low three-quarter angle, a dropped longsword with a MIST edge and a dented round shield on the stones, a torn BLOOD-red cloak crumpled beside them (the one accent), a thin shaft of moonlight from a crack above falling across them, a guttering candle stub far left with a tiny EMBER pool, the far corridor dissolving into INK. Keep the upper 45 % dark and low-detail (a banner and text sit over it); detail in the lower half"),
    ("backdrops/report", "1024x1536", "OPAQUE — the room behind the report's parchment scroll: a dark stone chamber at night; on the LEFT a hanging tattered BLOOD banner, a sword and a dented helm on a heap of gear with a few GILT coins; on the RIGHT an iron wall sconce with a small EMBER candle; a tall window at the top letting a shaft of cold moonlight into the room. Keep the centre dark and calm (the scroll covers it)"),
    ("deco/pillar", "512x1536", "OPAQUE — a tall carved stone wall panel seen straight on, filling the canvas: at the bottom a hooded stone knight statue in an alcove resting both hands on a sword, above it a hanging tattered BLOOD banner on an iron rod, above that an iron sconce with a small EMBER flame, the rest carved stone blocks with worn relief. DARK overall (it sits behind UI panels): mostly INK/UMBRA/DUSK, the flame and a few MIST moonlit rims the only bright spots"),
    ("deco/brazier", "256x384", "RGBA — an iron tripod brazier drawn in ink and wash with a bowl of coals and a tall EMBER flame (a flame is EMBER), centred, filling ~85 % of the height, no ground, no shadow"),
    ("deco/candle", "256x384", "RGBA — a short thick melted wax CANDLE, BONE wax with drips, on a small iron dish, a small bright EMBER flame with a soft glow only just around it, seen front-on, the object only"),
    ("fx/shield", "512x512", "RGBA — a boss's great battle SHIELD seen straight on, filling ~85 %: a heavy round shield of dark planks bound in blackened iron (INK/DUSK washes, MIST rims on the top edge), a riveted rim, a big iron boss with a crude BONE skull emblem, dents, and one deep jagged CRACK running top to bottom (it is about to split). A strong INK outline so it reads at 90 px"),
    ("fx/shard_0", "256x256", "RGBA — one broken FRAGMENT of that shield: a jagged wedge of dark planks with a bent piece of the iron rim and two rivets, splintered edges, INK outline"),
    ("fx/shard_1", "256x256", "RGBA — another fragment: a chunk of the central iron boss with part of the BONE skull emblem, torn metal edges, INK outline"),
    ("fx/shard_2", "256x256", "RGBA — a long splintered plank shard with one rivet, pale raw wood at the break (BONE), INK outline"),
    ("fx/shard_3", "256x256", "RGBA — a small curved piece of the iron rim, bent, bright MIST metal at the torn ends, INK outline"),
]


def scenes_prompt() -> str:
    body = head("BACKDROPS, DECORATION AND EFFECT ART", "/home/g/p/riddle/art/ui/ (backdrops/, deco/, fx/ only)", extra=SCENES)
    for i, (n, size, what) in enumerate(BACK, 1):
        body += f"{i}) /home/g/p/riddle/art/ui/{n}.png — {size} — {what}.\n"
    return body


PORTRAIT_RULES = """=== PORTRAIT RULES (hero_sheet.png's circular portrait is the model) ===
A head-and-shoulders BUST filling a 1024x1024 OPAQUE square (the game masks it to a circle, so keep the face inside the central
70 % circle): the face in three-quarter view looking right, cold moonlight from above-left, a HALFTONE screen in the shadows, the
paper showing through the highlights, loose ink hatching in a dark INK/DUSK washed background. Sharp gothic-anime features (narrow
eyes, a few confident ink strokes), a clear pale BONE face. Heroes wear the BLOOD-red cloak collar. No text, no frame, no ring.
"""

HERO_LOOKS = {
    "male": "a young MAN, short dark hair (nothing past the collar)",
    "female": "a young WOMAN, a long dark braid falling over one shoulder",
    "cat": "a cat-folk hero: a human face with yellow eyes, wild dark hair and two LARGE upright black CAT EARS on top of the head",
}
HERO_CLASS = {
    "fighter": "the WARRIOR: dull steel pauldron with GILT trim, a BLOOD cloak collar clasped with a GILT brooch, the hilt of a longsword over the shoulder",
    "rogue": "the ROGUE: dark UMBRA leathers, a pushed-back BLOOD hood around the neck, a dagger hilt at the shoulder",
    "ranger": "the RANGER: dark leathers, a BLOOD hooded cloak collar, BONE-fletched arrows in a quiver over the shoulder",
    "caster": "the CASTER: layered DUSK robes, a BLOOD mantle and cowl at the neck, the top of a staff with a pale MIST crystal beside the face",
}


def hero_portraits_prompt() -> str:
    body = head("HERO PORTRAITS", "/home/g/p/riddle/art/ui/portraits/", refs="/home/g/p/riddle/art/ui/targets/style/hero_sheet.png, "
                "/home/g/p/riddle/art/ui/targets/style/watch_warrens.png (its portrait well)", extra=PORTRAIT_RULES)
    i = 0
    for c, cd in HERO_CLASS.items():
        for lk, ld in HERO_LOOKS.items():
            i += 1
            spr = f"/home/g/p/riddle/art/generated/hero_{c}_{lk}.png"
            body += (f"{i}) /home/g/p/riddle/art/ui/portraits/hero_{c}_{lk}.png — 1024x1024 OPAQUE — {cd}; {ld}. The SAME character as the "
                     f"game sprite {spr} (open it: match its hair, kit and colours).\n")
    return body


OTHER_PORTRAITS = [
    ("pet_rat", "a big dungeon RAT (a pet): matted UMBRA/DUSK fur with MIST rims, a glinting BONE eye, round ears — alert, loyal, a little comic"),
    ("pet_jackal", "a lean JACKAL (a pet): big pointed ears, narrow snout, dusty DUSK/MOON fur with MIST rims, a BLOOD scrap of cloth tied at the neck as a collar"),
    ("pet_monkey", "a mischievous dungeon MONKEY (a pet): UMBRA fur, a cheeky grin, clutching a small BONE canvas sack"),
    ("pet_goblin", "a tamed GOBLIN (a pet): cold DUSK-green skin, long ears with MIST rims, a sheepish grin of BONE teeth, a BLOOD rag tied round its head"),
    ("captive", "a CAPTIVE adventurer: a gaunt young face looking up hopefully, a heavy iron collar and chain (INK with MIST glints), torn DUSK tunic"),
    ("boss_goblin_captain", "the goblin CAPTAIN: cold DUSK-green skin, a horned iron half-helm with MIST rims, a battered war horn at the lips, a BLOOD sash"),
    ("boss_goblin_warlord", "the GOBLIN WARLORD: a gaunt goblin king, cold DUSK-green skin, a spiked iron crown, glowing BONE eyes, a tattered BLOOD-and-INK mantle"),
]


def other_portraits_prompt() -> str:
    body = head("PORTRAITS (pets, the captive, two bosses)", "/home/g/p/riddle/art/ui/portraits/",
                refs="/home/g/p/riddle/art/ui/targets/style/hero_sheet.png, /home/g/p/riddle/art/ui/targets/style/boss.png",
                extra=PORTRAIT_RULES.replace("Heroes wear the BLOOD-red cloak collar.", "BLOOD only where the subject says."))
    for i, (n, what) in enumerate(OTHER_PORTRAITS, 1):
        src = {"pet_rat": "rat", "pet_jackal": "jackal", "pet_monkey": "monkey", "pet_goblin": "goblin", "captive": "captive",
               "boss_goblin_captain": "goblin_captain", "boss_goblin_warlord": "boss_goblin_warlord"}[n]
        body += (f"{i}) /home/g/p/riddle/art/ui/portraits/{n}.png — 1024x1024 OPAQUE — {what}. The same creature as the game sprite "
                 f"/home/g/p/riddle/art/generated/{src}.png (open it: match it).\n")
    return body


def write(name: str, body: str) -> str:
    (ROOT / "prompts" / f"{name}.txt").write_text(body + "When all assets are saved, list the final paths and sizes. Do nothing else.\n")
    return name


def main() -> None:
    names = [write("p2_ui_frames", frames_prompt()), write("p2_ui_icons_a", icons_prompt(ICONS[:18])),
             write("p2_ui_icons_b", icons_prompt(ICONS[18:])), write("p2_ui_scenes", scenes_prompt()),
             write("p2_portraits_heroes", hero_portraits_prompt()), write("p2_portraits_other", other_portraits_prompt())]
    print(" ".join(names))


if __name__ == "__main__" and len(sys.argv) == 1:
    main()


# ---- the town (docs/TOWN.md §7; Cut 30's town v1: the camp, blacksmith, storehouse, kennel, bank, walkers) ----
# id: (master_h, texel_h, gen, brief). Keyed sprites in the atlas (kind "town"); the renderer's town scene (Cut 30) keys by id and
# falls back to a coloured block + its icon.
_B = ("seen from the town's high three-quarter camera (the roof and the front wall both visible), a small building on the pixel grid with "
      "INK line and washes, steep dark roofs with MIST moonlit ridges, warm EMBER light only in its windows/door, standing on nothing")
TOWN: dict[str, tuple[int, int, str, str]] = {
    "town_mouth_cave": (128, 64, "1536x1024", "the DUNGEON MOUTH, stage one: a black cave opening cut into a rocky hillside, rough rock framing it with MIST-lit ledges, two iron torches on poles either side with small EMBER flames, a few bones at the threshold; the opening INK-black and deep"),
    "town_mouth_timber": (128, 64, "1536x1024", "the DUNGEON MOUTH, stage two: the same cave opening now braced with heavy timber beams and a lintel, a hanging BLOOD cloth strip, two iron torches with EMBER flames, a small wooden sign post (no letters)"),
    "town_mouth_gate": (128, 64, "1536x1024", "the DUNGEON MOUTH, stage three: a carved stone GATEHOUSE arch over the cave, gothic pointed arch, a raised iron portcullis, two BLOOD banners, two braziers with EMBER flames"),
    "town_campfire_0": (48, 24, "1024x1024", "a CAMPFIRE: a ring of stones around crossed logs with a lively EMBER flame (two tongues), frame 1 of 2"),
    "town_campfire_1": (48, 24, "1024x1024", "the same CAMPFIRE, frame 2 of 2: the flame tongues shifted and one taller, a spark above"),
    "town_tent": (80, 40, "1024x1024", "the hero's TENT: a single A-frame canvas tent of DUSK-grey canvas with a BLOOD pennant on its pole, the flap tied open showing an INK interior, guy ropes and pegs"),
    "town_crate": (40, 20, "1024x1024", "the SUPPLY CRATE: a wooden crate with iron corners, a coil of rope and a small sack on top"),
    "town_plot": (40, 20, "1024x1024", "a STAKED PLOT: four wooden stakes with a string between them marking a square of bare earth, a small MIST-white rag tied to one stake"),
    "town_scaffold": (96, 48, "1024x1024", "a SCAFFOLD: a timber frame of a house under construction, poles and cross-braces, a ladder, a few stacked planks; no walls yet"),
}
for bname, looks in {
    "blacksmith": ("an open-fronted timber SMITHY with a stone chimney, an anvil at the front, a glowing EMBER forge inside",
                   "a larger stone-and-timber SMITHY with a tiled roof, two anvils, a quench barrel, a weapon rack, the forge glowing EMBER",
                   "a grand stone FORGE-HALL with a tall chimney, iron gates, a great bellows, the forge roaring EMBER, a BLOOD banner"),
    "storehouse": ("a small wooden STORE SHED with a plank door, crates and a barrel stacked by it",
                   "a timber STOREHOUSE with a loft door and a hoist beam, sacks and crates",
                   "a stone WAREHOUSE with iron-bound double doors, a hoist, a GILT lock plate"),
    "kennel": ("a small wooden KENNEL shed with a fenced pen and a water trough",
               "a longer KENNEL with three stalls, a fenced run, a hay pile",
               "a stone-and-timber BEAST HALL with a large fenced yard, carved beast-head gables"),
    "bank": ("a small squat stone VAULT HOUSE with an iron door and a hanging coin sign (a GILT disc, no letters)",
             "a stone BANK with two short columns, iron-barred windows, a coin sign",
             "a grand stone BANK with four columns, a pediment, a heavy iron vault door, GILT trim"),
}.items():
    for i, what in enumerate(looks, 1):
        TOWN[f"town_{bname}_{i}"] = (128, 64, "1536x1024", f"{what}; {_B}. Look {i} of 3 ({['built', 'improved', 'grand'][i - 1]}): the three looks of one building must read as the same place growing")
TOWN.update({
    "town_smith": (80, 40, "1024x1024", "a TOWNSFOLK smith walking right: a burly figure in a heavy UMBRA leather apron, a hammer on the shoulder, a pale BONE face; the town's people wear no BLOOD"),
    "town_merchant": (80, 40, "1024x1024", "a TOWNSFOLK merchant walking right: a stout figure in a long DUSK coat and a wide hat, a GILT-buckled satchel, a pale BONE face"),
    "town_child": (56, 28, "1024x1024", "a TOWNSFOLK child running right: a small figure in a DUSK tunic, a wooden toy sword raised, a pale BONE face"),
    "town_carter": (80, 40, "1024x1024", "a TOWNSFOLK carter walking right leading by a rope: a lean figure in a hooded MOON-grey cloak with a staff, a pale BONE face"),
    "town_cart": (64, 32, "1536x1024", "a MULE CART going right: a grey mule pulling a small two-wheeled wooden cart loaded with sacks and a barrel"),
    "town_dog": (40, 20, "1024x1024", "a town DOG trotting right: a shaggy DUSK-grey dog with a curled tail, MIST rims, a BLOOD neckerchief"),
    "town_sack_small": (24, 12, "1024x1024", "a small loot SACK of BONE canvas tied with a cord, a GILT coin peeking out"),
    "town_sack_large": (32, 16, "1024x1024", "a large bulging loot SACK of BONE canvas, tied, GILT coins spilling at its mouth"),
    "town_chest_glow": (32, 16, "1024x1024", "a small iron-bound treasure CHEST, the lid cracked open, a cold MIST glow spilling out (a found treasure)"),
    "town_flag_0": (48, 24, "1024x1024", "a tall wooden pole with a swallow-tailed BLOOD flag streaming right, frame 1 of 2"),
    "town_flag_1": (48, 24, "1024x1024", "the same flag pole, frame 2 of 2: the BLOOD flag's ripple shifted"),
})


def town_manifest() -> int:
    import json
    p = ROOT / "manifest.json"
    m = json.loads(p.read_text())
    have = {a["id"]: a for a in m["assets"]}
    n = 0
    for aid, (mh, th, gen, what) in TOWN.items():
        entry = {"id": aid, "kind": "town", "bg": "keyed", "gen": gen, "master_h": mh, "texel_h": th,
                 "description": what + ". (Art direction phase 2, docs/TOWN.md §7.)", "brief3": what + "."}
        if aid in have:
            have[aid].update(entry)
        else:
            m["assets"].append(entry); n += 1
    p.write_text(json.dumps(m, indent=2, ensure_ascii=False) + "\n")
    return n


def town_prompts() -> list[str]:
    import make_prompts as mp
    prop = ("the object's own shape as briefed; townsfolk are readable little people (head about one fifth of the height, a pale BONE face "
            "patch, walking right); buildings and landmarks fill the canvas width with ~8 % margin")
    ids = list(TOWN)
    groups = {"p2_town_a": [i for i in ids if i.startswith(("town_mouth", "town_campfire", "town_tent", "town_crate", "town_plot", "town_scaffold"))],
              "p2_town_b": [i for i in ids if i.startswith(("town_blacksmith", "town_storehouse"))],
              "p2_town_c": [i for i in ids if i.startswith(("town_kennel", "town_bank"))],
              "p2_town_d": [i for i in ids if i.startswith(("town_smith", "town_merchant", "town_child", "town_carter", "town_cart", "town_dog", "town_sack", "town_chest", "town_flag"))]}
    out = []
    for name, part in groups.items():
        body = mp.header3(proportion=prop, tiny="buildings ~64 px tall, people ~40 px, props 12-24 px").replace(
            "ONE full-body figure, a three-quarter view from slightly above, facing RIGHT", "ONE object or figure, seen from the high three-quarter town camera (people face RIGHT)")
        body = body.replace("/home/g/p/riddle/art/ui/targets/style/hero_sheet.png,", "/home/g/p/riddle/art/ui/targets/style/town.png, /home/g/p/riddle/art/ui/targets/style/hero_sheet.png,")
        for k, aid in enumerate(part, 1):
            mh, th, gen, what = TOWN[aid]
            body += mp.sprite_block3(k, aid, gen, th, what + ".")
        write(name, body)
        out.append(name)
    return out


if __name__ == "__main__" and len(sys.argv) > 1 and sys.argv[1] == "town":
    print(f"manifest +{town_manifest()} town ids")
    print(" ".join(town_prompts()))


# ---- items, effects and the shared dressing (register 3's hue assets and ramp decals: art/make_env.py converts them) ----
ITEMS = {
    "env_item_potion": "a round-bellied glass POTION flask with a short neck and a cork, three-quarter view: MIST-rimmed glass holding a BLOOD-red liquid in its lower two thirds, one BONE highlight, INK contour; fills ~60 % of the tile",
    "env_item_scroll": "a rolled parchment SCROLL lying diagonally: a BONE paper roll, the rolled ends darker spirals, tied in the middle with a BLOOD wax seal, INK contour; ~70 % of the tile",
    "env_item_weapon": "a short SWORD lying diagonally (hilt lower-left, tip upper-right): a MIST blade with a BONE edge line, a worn GILT crossguard and pommel, an UMBRA grip, INK contour; spans the tile",
    "env_item_armour": "a steel BREASTPLATE seen front-on with short shoulder guards: MOON/MIST steel washes lit from above, a line of rivets, a small BLOOD cloth trim at the collar, INK contour; ~70 % of the tile",
    "env_item_gold": "a small pile of COINS: 5-7 worn GILT coins stacked and scattered, BONE glints on their top edges, INK contour; ~60 % of the tile, bottom-centred",
    "env_torch": "a WALL TORCH SCONCE front view (a TALL piece, 16x24: paint on a 1024x1536 canvas): an INK iron bracket with a ring holding a short torch upright, burning with a bright EMBER flame (BONE core, EMBER body, CLOT tips, two or three tongues) in the top third",
    "env_banner": "a WALL BANNER hanging from a short INK iron rod (16x20: paint on a 1024x1280 canvas): a WIDE BLOOD-red cloth (CLOT folds, ragged swallowtail hem) with a bold BONE sigil (an upright sword crossed by a bar), a thin GILT trim along the top",
    "env_blood_0": "a BLOOD splatter on stone seen from above: one irregular splash of BLOOD with CLOT at its heart and a few droplets flying off; only the blood",
    "env_blood_1": "a smear and drip trail of BLOOD (CLOT darker), elongated, with 3-4 small droplets; only the blood",
    "env_shrine": "a small stone SHRINE ALTAR front view from slightly above, perfectly SYMMETRICAL, filling most of the tile: a wide low altar block of DUSK stone with a MIST-lit top edge, and on its back centre a short rounded stele with a carved BONE glyph; two tiny candles on the altar with EMBER flames",
    "env_vault": "a locked iron CAGE gate seen front-on from slightly above, filling most of the tile: a heavy INK iron frame with a riveted top rail and two thick posts, four MIST-lit iron bars with the dark interior between them, a square GILT padlock at the centre",
    "env_vault_open": "the SAME iron cage OPENED: the barred door swung open to the right (seen edge-on as a narrow barred strip at the right post), the GILT padlock hanging open, the dark EMPTY interior clearly visible",
    "env_nest": "a beast's NEST mound, three-quarter top-down: a low WIDE dome of heaped straw and twigs (DUSK and MOON washes, MIST on top), with a nearly black round burrow HOLE in the front centre",
    "env_bones": "scattered old skeleton remains seen from above: one BONE skull with two INK eye sockets upper-left, three or four long bones lying crossed below and right; BONE and MIST washes with INK contour only",
    "env_moss_0": "a patch of dark moss and tiny weeds growing in floor cracks (cold DUSK-green washes with a few MIST tips), irregular, about a third of the tile, spreading from one corner",
    "env_moss_1": "a smaller patch of dark moss (cold DUSK-green washes) along a crack, a few tiny BONE flowers",
    "env_crack": "a network of INK floor cracks with a few pale MIST broken stone chips, as if a heavy blow cracked the flagstone; only the cracks and chips",
    "env_rubble": "a small scatter of rubble: 5-7 broken stone chunks and pebbles (DUSK/MOON washes, MIST lit tops, INK contour)",
}


def items_prompt() -> str:
    body = head("PIXEL ITEM AND DRESSING SPRITES", "/home/g/p/riddle/art/generated/",
                refs="/home/g/p/riddle/art/ui/targets/style/watch_warrens.png, /home/g/p/riddle/art/ui/targets/style/watch_fens.png",
                extra="""=== HOW THESE ARE USED (critical) ===
Each asset is a TINY game sprite: the 1024 px image is box-downscaled to a 16x16 (torch 16x24, banner 16x20) pixel grid and
drawn on the dungeon floor. Paint it AS that tiny tile enlarged (each cell a ~64 px square, stepped edges, washes inside the
cells), BOLD simple shapes, an INK contour one cell thick, light from above. The object only, centred, on a TRANSPARENT
background (or, if the tool gives an opaque image, a flat uniform pure MAGENTA #FF00FF edge to edge); no floor, no cast
shadow, no glow halo (a flame may glow inside its own shape), no text, no magenta on the object.
""")
    for i, (aid, what) in enumerate(ITEMS.items(), 1):
        gen = "1024x1536" if aid == "env_torch" else ("1024x1280" if aid == "env_banner" else "1024x1024")
        body += f"{i}) /home/g/p/riddle/art/generated/{aid}.png — {gen} — {what}.\n"
    return body


if __name__ == "__main__" and len(sys.argv) > 1 and sys.argv[1] == "items":
    print(write("p2_items", items_prompt()))
