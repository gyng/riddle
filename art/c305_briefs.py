#!/usr/bin/env python3
"""Cut 30.5 art (docs/AUTOMATION_TREE.md §C): the town's workers (a standing figure at its post + one idle-work frame), the haul
chest (closed · full · open) and the works sheet's node icons. One list feeds the manifest entries and the Codex prompts.

    python3 art/c305_briefs.py manifest          # add / refresh the manifest entries
    python3 art/c305_briefs.py prompts [ids…]    # write art/prompts/c305_{workers_a,workers_b,chest_icons}.txt (or c305_redo.txt for ids)
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TREE = "(Cut 30.5, docs/AUTOMATION_TREE.md §C; art/town-ids.md.)"
WORKER_RULE = ("a TOWNSFOLK worker standing at their post, facing right in the high three-quarter town view, full body; workers wear "
               "no BLOOD (BLOOD is the hero's); a pale BONE face; the tag object drawn LARGE (it is the role's read at 24 px)")
# id suffix, frame-0 brief, frame-1 (idle work) brief
WORKERS = [
    ("porter",
     "the PORTER: a stocky labourer in a short UMBRA work coat and a flat cap, both hands on the handles of a two-wheeled wooden "
     "HANDCART in front of him (to the right), the cart heaped with two bulging BONE-canvas loot sacks, a GILT coin peeking from one "
     "— the handcart heaped with sacks is the tag",
     "the same porter LEANING forward into the handles, pushing the cart, the near wheel turned a quarter, one foot back"),
    ("armourer",
     "the ARMOURER: a broad figure in a chain-mail shirt under a leather tabard, holding a steel HELMET up in one hand to inspect it, "
     "beside a wooden WEAPON RACK (to the right, as tall as their shoulder) with two swords and a spear standing in it, MIST edges on "
     "the blades — the weapon rack and the held helmet are the tag",
     "the same armourer setting the helmet on top of the rack, the other hand on a sword's hilt in the rack"),
    ("apprentice",
     "the forge APPRENTICE: a YOUNG slight figure in an oversized leather apron, sleeves rolled, a small hammer raised to shoulder "
     "height over a squat black ANVIL in front of them (to the right, waist high), an EMBER-orange glowing iron bar on the anvil "
     "— the anvil and the raised hammer are the tag",
     "the same apprentice with the hammer DOWN on the glowing bar, three or four small EMBER sparks flying off it"),
    ("keeper",
     "the STOREKEEPER: an old stooped figure in a long DUSK coat and a knit cap, a big iron KEY RING hanging at the belt, sweeping "
     "with a tall straw BROOM held diagonally, a small lidded crate by the feet — the broom and the big key ring are the tag",
     "the same keeper with the broom swept to the other side, a little puff of BONE dust at its head"),
    ("clerk",
     "the bank CLERK: a thin upright figure in a long dark frock coat with GILT buttons, round spectacles, a big open LEDGER book "
     "with pale BONE pages held open on one forearm, a quill pen in the other hand, a small coin purse at the belt "
     "— the open pale ledger is the tag",
     "the same clerk with the head bent, writing in the ledger with the quill"),
    ("drillmaster",
     "the DRILLMASTER: a tall broad-shouldered veteran in a padded gambeson and a round kettle helm, one hand behind the back, the "
     "other pointing a long wooden PRACTICE SWORD forward at a straw TRAINING DUMMY on a post (to the right, a little shorter than "
     "them, a crossbar for arms) — the dummy and the pointing practice sword are the tag",
     "the same drillmaster striking the dummy with the practice sword, the dummy tilted back on its post"),
    ("kennel_hand",
     "the KENNEL-HAND: a sturdy figure in a fur-trimmed jerkin and gloves, holding a wooden FEED BUCKET with a big BONE sticking out "
     "of it, a lean HOUND sitting at their feet (to the right) looking up at the bucket — the dog and the bucket are the tag",
     "the same kennel-hand holding the bone out low to the hound, the hound up on its hind legs reaching for it"),
    ("herald",
     "the HERALD: a tall figure in a long tabard with GILT trim and a cap with a long feather, holding a long unrolled pale BONE "
     "SCROLL that hangs down to the knees, a brass HAND BELL held out at shoulder height in the other hand — the hanging scroll "
     "and the bell are the tag",
     "the same herald swinging the bell, the head tipped back calling out, the scroll still hanging"),
    ("guide",
     "the GUIDE: a lean hooded figure in a long MIST-grey travelling cloak, a tall walking staff in one hand and a LIT LANTERN held "
     "out in front at chest height (an EMBER flame inside, the only warm light), a coil of rope on the hip — the lit lantern "
     "and the hood are the tag",
     "the same guide lifting the lantern to shoulder height and peering ahead, the staff planted"),
    ("scout",
     "the SCOUT (the send-worker: sends the hero down): a lean light-footed figure in a short hooded cape over a leather jerkin, a "
     "wide-brimmed hat with a MIST feather, raising a brass SPYGLASS to the eye with one hand, the other arm stretched out pointing "
     "forward and down (toward the dungeon), a short horn at the hip — the raised spyglass and the pointing arm are the tag",
     "the same scout lowering the spyglass and waving the pointing arm forward, a 'go now' beckon"),
    ("quartermaster",
     "the QUARTERMASTER (packs the hero's supplies): a stout figure in a buff padded coat and a leather cap, a big stuffed BACKPACK "
     "with a bedroll strapped on held up in front by its straps, a corked POTION flask with a pale BONE label hanging at the belt, "
     "a coil of rope over the shoulder — the held-up backpack is the tag",
     "the same quartermaster tucking the potion flask into the backpack's top, the pack held against the hip"),
]
CHEST = [
    ("town_haul_chest",
     "the HAUL CHEST, CLOSED: a wide low wooden strongbox (twice as wide as tall) on four short feet, a flat lid, thick iron bands, "
     "an iron carry handle at each end, a GILT lock plate at the front — sturdy and plain"),
    ("town_haul_chest_full",
     "the SAME haul chest as town_haul_chest.png (open it first) but FULL: the lid shut but propped up a finger's width by the heap inside, GILT coins spilling over the front "
     "edge, a coin and a small BONE-white star GLINT on the heap, a tied loot sack sitting on the lid — it must read 'full, come and "
     "open me' at 12 px"),
    ("town_haul_chest_open",
     "the SAME haul chest as town_haul_chest.png (open it first) but OPEN and EMPTY: the lid thrown back upright behind it, the INK-dark inside empty but for one coin, the "
     "front and bands as the closed chest"),
]
ICON_RULE = ("the head and shoulders of the worker in three-quarter view, small, and their tag object drawn LARGE in front of the "
             "chest (half the icon); the object is the read")
ICONS = [
    ("porter", "the PORTER (flat cap) with a two-wheeled HANDCART heaped with sacks in front"),
    ("armourer", "the ARMOURER (chain-mail collar) holding up a steel HELMET, two sword hilts behind"),
    ("apprentice", "a young APPRENTICE raising a hammer over an ANVIL, one EMBER spark"),
    ("keeper", "the STOREKEEPER (knit cap) with a big iron KEY RING held up and a broom handle behind"),
    ("clerk", "the CLERK (spectacles) with an open pale LEDGER and a quill, one GILT coin"),
    ("drillmaster", "the DRILLMASTER (kettle helm) with a wooden PRACTICE SWORD held upright and a straw dummy's head"),
    ("kennel_hand", "the KENNEL-HAND with a HOUND's head beside theirs and a BONE in the hand"),
    ("herald", "the HERALD (feathered cap) ringing a HAND BELL, an unrolled scroll"),
    ("guide", "the GUIDE (hood) holding up a LIT LANTERN, an EMBER flame"),
    ("scout", "the SCOUT (wide-brimmed hat with a MIST feather) holding a brass SPYGLASS to the eye"),
    ("quartermaster", "the QUARTERMASTER (leather cap) holding up a stuffed BACKPACK with a bedroll and a potion flask"),
]


def worker_ids() -> list[str]:
    return [f"town_worker_{w}{s}" for w, *_ in WORKERS for s in ("", "_1")]


def entries() -> list[dict]:
    out = []
    for w, b0, b1 in WORKERS:
        for suf, brief in (("", f"{WORKER_RULE}; {b0}."), ("_1", f"the idle-work frame of town_worker_{w}: {b1}; same scale, the feet "
                                                                  f"and the head top where frame 0 has them, nothing higher than the head.")):
            out.append({"id": f"town_worker_{w}{suf}", "kind": "town", "bg": "keyed", "gen": "1024x1024", "master_h": 96, "texel_h": 48,
                        "description": f"{brief} {TREE}", "brief3": brief})
    for cid, brief in CHEST:
        out.append({"id": cid, "kind": "town", "bg": "keyed", "gen": "1024x1024", "master_h": 48, "texel_h": 24,
                    "description": f"{brief}. {TREE}", "brief3": f"{brief}."})
    return out


def bbox(i: str) -> tuple[int, int]:
    from PIL import Image
    im = Image.open(ROOT / "generated" / f"{i}.png").convert("RGBA")
    a = im.getchannel("A")
    if a.getextrema()[0] == 255:   # opaque on a magenta key
        import numpy as np
        x = np.asarray(im, np.int32)
        m = (abs(x[..., 0] - 255) + x[..., 1] + abs(x[..., 2] - 255)) > 90
        ys, xs = np.nonzero(m)
        return int(xs.max() - xs.min() + 1), int(ys.max() - ys.min() + 1)
    b = a.point(lambda v: 255 if v > 32 else 0).getbbox()
    return b[2] - b[0], b[3] - b[1]


def scale(e: dict) -> int:
    """pack.py scales a frame by its bounding box: keep a worker's idle frame (drawn at frame 0's scale, its bbox height may differ)
    and the chest's states (the lid up, a sack on top) at the scale of their frame 0 / the closed chest"""
    i = e["id"]
    try:
        if i.startswith("town_worker_") and i.endswith("_1"):
            (_, h0), (_, h1) = bbox(i[:-2]), bbox(i)
            return max(8, round(96 * h1 / h0 / 2) * 2)
        if i.startswith("town_haul_chest_"):
            (w0, h0), (w1, h1) = bbox("town_haul_chest"), bbox(i)
            return max(8, round(48 * (h1 / w1) / (h0 / w0) / 2) * 2)
    except FileNotFoundError:
        pass
    return e["master_h"]


def manifest() -> None:
    p = ROOT / "manifest.json"
    m = json.loads(p.read_text())
    new = {e["id"]: e for e in entries()}
    for e in new.values():
        e["master_h"] = scale(e); e["texel_h"] = e["master_h"] // 2
    m["assets"] = [a for a in m["assets"] if a["id"] not in new] + list(new.values())
    p.write_text(json.dumps(m, indent=2, ensure_ascii=False) + "\n")
    print(f"manifest: {len(new)} entries")


def header(src: str) -> str:
    """the approved Cut 30 headers (art/prompts/c30_*.txt), pointed at this worktree"""
    t = (ROOT / "prompts" / src).read_text().split("=== ASSETS ===")[0]
    return t.replace("/home/g/p/riddle/.claude/worktrees/c30art", str(ROOT.parent)) + "=== ASSETS ===\n"


def sprite_prompt(ids: list[str]) -> str:
    gen = ROOT / "generated"
    t = header("c30_town_r3.txt").replace("Proportions (these override \"tall\" in the style above): the object's own shape as briefed; "
                                          "buildings and landmarks fill the canvas width with ~8 % margin.",
                                          "Proportions: a worker is an ordinary adult townsperson (head ~1/6 of the height; the apprentice "
                                          "a youth), the same family as the existing townsfolk; the chest is a small prop.")
    t += (f"Match the existing TOWNSFOLK family: open {gen}/town_smith.png and {gen}/town_merchant.png (same ink, wash, palette, "
          f"scale of detail). Each worker comes as FRAME 0 (standing at the post) then FRAME 1 (idle work): make frame 0 first, then "
          f"OPEN it and make frame 1 as the SAME person (face, clothes, colours, tag object) with only the pose changed; the feet stay "
          f"where they are and nothing in frame 1 rises above the top of the head in frame 0 (the pipeline scales by the bounding box).\n")
    by = {e["id"]: e for e in entries()}
    for n, i in enumerate(ids, 1):
        e = by[i]
        tall = "24 px" if i.startswith("town_worker") else "12 px"
        t += (f"--- {n}. id: {i}\nOutput path: {gen}/{i}.png\nSize: exactly 1024x1024 px (width x height), PNG.\n"
              f"Shown in game at about {tall} tall; judge readability at that size.\nBrief: {e['brief3']}\n\n")
    return t + "When all assets are saved, list the final paths and sizes. Do nothing else.\n"


def icon_prompt(names: list[str]) -> str:
    d = ROOT / "ui/icons"
    t = header("c30_icons.txt")
    t += (f"These are the WORKS SHEET's node icons, one per hired town worker, beside the existing package icons (open "
          f"{d}/pkg_steady.png and {d}/pkg_light_hands.png: match their ink, wash and weight). Each: {ICON_RULE}. The set of eleven must tell "
          f"apart at 40 px by the object alone.\n")
    br = dict(ICONS)
    for n, w in enumerate(names, 1):
        t += f"{n}) {d}/node_{w}.png — 256x256 RGBA — {br[w]}.\n"
    return t + "When all assets are saved, list the final paths and sizes. Do nothing else.\n"


def prompts(ids: list[str]) -> None:
    P = ROOT / "prompts"
    if ids:   # a redo round: sprite ids and node_<w> icons
        sp = [i for i in ids if not i.startswith("node_")]
        ic = [i[5:] for i in ids if i.startswith("node_")]
        if sp:
            (P / "c305_redo.txt").write_text(sprite_prompt(sp))
        if ic:
            (P / "c305_redo_icons.txt").write_text(icon_prompt(ic))
        return
    w = worker_ids()
    (P / "c305_workers_a.txt").write_text(sprite_prompt(w[:10]))
    (P / "c305_workers_b.txt").write_text(sprite_prompt(w[10:]))
    (P / "c305_chest.txt").write_text(sprite_prompt([c for c, _ in CHEST]))
    (P / "c305_icons.txt").write_text(icon_prompt([w for w, _ in ICONS]))
    print("prompts: c305_workers_a, c305_workers_b, c305_chest, c305_icons")


if __name__ == "__main__":
    a = sys.argv[1:]
    {"manifest": lambda: manifest(), "prompts": lambda: prompts(a[1:])}[a[0]]()
