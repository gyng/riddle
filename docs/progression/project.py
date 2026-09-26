#!/usr/bin/env python3
"""Project the proposed progression (docs/PROGRESSION.md) onto the measured timeline.

Reads docs/progression/timeline.json (examples/progression.rs) and the wall probe
(PROG_WALLS_ONLY run, `--walls`), replays each lineage's measured days under the proposal's
income, catalogue, system curriculum and gold sinks, and writes docs/progression/proposal.json
with the same per-day shape plus the per-day system table. What is measured is kept (the depth
curve, the bosses, the trophies, the runs and banks, the gold brought home); what the proposal
changes is recomputed from it; the one behavioural assumption (a stalled player pushes when the
plateau prompt or a `bold` oath asks, and a push the probe says passes the record on >= 10 % of
sends breaks the stall the next day) is applied only where the probe measured it.

    python3 docs/progression/project.py [--timeline docs/progression/timeline.json] [--walls walls.json] [--out docs/progression/proposal.json]
"""
import argparse, json, collections, statistics as st

BOSSES = [("goblin_warlord", 8), ("bloat_mother", 13), ("lich", 18), ("foundry_master", 23), ("lurker_queen", 28), ("mirror_king", 33)]

# --- the proposed catalogue: (id, marks, tier). Tier k opens when k band bosses are *met*
# (their floor reached: the wall is the teacher) — tier 1 at the first bank.
CATALOGUE = [
    # T1 · first bank
    ("row5", 3, 1), ("throw", 3, 1), ("vault2", 3, 1),
    # T2 · the Warlord met (D8)
    ("row6", 4, 2), ("corridor_fighting", 4, 2), ("pack_break", 4, 2), ("thief_guard", 4, 2), ("cond_party_hp", 2, 2),
    # T3 · the Mother met (D13)
    ("row7", 5, 3), ("ranger", 6, 3), ("boss_focus", 4, 3), ("vault3", 5, 3), ("oath_slot_2", 6, 3),
    # T4 · the Lich met (D18)
    ("row8", 6, 4), ("caster", 8, 4), ("reflect_read", 5, 4), ("vault4", 6, 4),
    # T5 · the Foundry Master met (D23)
    ("row9", 7, 5), ("cadence", 5, 5), ("lantern_rig", 5, 5), ("recall_sense", 6, 5), ("vault5", 7, 5),
    # T6 · the Lurker Queen met (D28)
    ("row10", 8, 6), ("oath_slot_3", 8, 6),
]
ROWS = [c for c in CATALOGUE if c[0].startswith("row")]
OATH_DRAW = 2          # ◆ per fresh oath (repeatable, once the board is open)
# the oath pool's rewards — reserved: never sold for marks, only kept (a goal of your own that ends
# in something new; the board no longer empties when the catalogue is bought out)
OATH_POOL = ["party_slot_2", "verb:hold", "title:warlord-burner", "waystone:D9", "card:gas_step", "party_slot_3", "route_2",
             "card:last_stand", "title:firebrand", "heir_pick", "waystone:D14", "party_slot_4", "title:pyre", "waystone:D19",
             "set_per_band", "title:foundry"]
OATH_KEEP_EVERY = 2    # a sworn oath is kept every 2nd day (Cut 28: each pool oath keepable >= 20 %/night by some set; 3-4 nights a day)
NIGHT_MARK_MIN_S = 4 * 3600
# gold-only automations, in forge units (policy-neutral chores: measured 0/N on every lineage)
AUTOMATIONS = [("quartermaster", 4), ("auto_insure", 5), ("incubator", 3), ("supply_cap_5", 3), ("bone_sense", 3), ("third_tag", 5)]

# --- the system curriculum: (system, trigger, how the trigger is read off the timeline)
SYSTEMS = [
    ("send", "day 0"),
    ("dial", "day 0"),               # thresholds on the preset rows only
    ("forecast:headline", "day 0"),
    ("edit", "first death"),
    ("death screen", "first death"),
    ("exits (bank/return/rest)", "first gold home"),
    ("loadout", "first gold home"),
    ("unlocks", "first mark"),
    ("reorder", "first plateau"),
    ("forecast:vs line", "first plateau"),
    ("foe tags + cards", "first foe fact"),
    ("party/taming", "first stray"),
    ("cage", "first cage"),
    ("boss wall + counters", "the Warlord met"),
    ("forecast:divergence", "the Warlord met"),
    ("forge", "the Warlord slain"),
    ("waystones/start", "the Warlord slain"),
    ("route (D5 fork)", "the D5 fork seen twice"),
    ("oaths", "the first plateau or the Warlord met"),
    ("automations (gold)", "the Lich met"),
    ("route 2 (D9 fork)", "an oath kept (bold)"),
    ("heir pick", "an oath kept"),
]


def boss_met(best):
    return sum(1 for _, d in BOSSES if best >= d)


def systems_open(day, best, bosses_slain, stalled_before, rec, oath_rewards=()):
    """The systems open at the end of `day` (1-based), from what the lineage did by then."""
    out = {"send", "dial", "forecast:headline"}
    if day >= 1:
        # a fresh lineage dies, banks, earns a mark and meets its first foe facts in its first hour (dayplayer first hour: best >= D6 on 10/10)
        out |= {"edit", "death screen", "exits (bank/return/rest)", "loadout", "unlocks", "foe tags + cards", "party/taming", "cage"}
    if stalled_before or day >= 2:
        out |= {"reorder", "forecast:vs line"}
    if best >= 8:
        out |= {"boss wall + counters", "forecast:divergence"}
    if bosses_slain >= 1:
        out |= {"forge", "waystones/start"}
    if best >= 6 and day >= 2:
        out.add("route (D5 fork)")
    if best >= 8 or stalled_before:
        out.add("oaths")
    if best >= 18:
        out.add("automations (gold)")
    if "route_2" in oath_rewards:
        out.add("route 2 (D9 fork)")
    if "heir_pick" in oath_rewards:
        out.add("heir pick")
    return out


def project(l, walls, a):
    s = l["summary"]
    days = l["days"]
    rater = s["mode"] == "rater"
    sched = [8 * 3600, 20 * 60, 4 * 3600, 4 * 3600, 7 * 3600 + 40 * 60] if rater else [8 * 3600] * 3
    marks = 0.0
    owned = set()
    out_days = []
    bosses_cum = 0
    prev_systems = set()
    purse = 0
    unit = None
    autos = set()
    commissions = 0
    stall_run = 0
    # the push assumption: a stalled day whose probe says the exits pushed to the record + 1 pass it on >= 10 % of sends
    push_ok = {d: v for d, v in walls.get((s["name"], s["seed"]), [])}
    shift = 0              # floors the pushes added over the measured curve
    oath_open_day = None
    pool = list(OATH_POOL)
    oath_rewards = []
    for i, x in enumerate(days):
        day = i + 1
        src = dict(x["marks"]["by_source"])
        frontier = src.pop("frontier", 0)
        bosses_cum += src.get("boss", 0) // 3
        # depth (projected): the measured curve, plus a push (plateau prompt / `bold` oath) where the probe says it passes
        pushed = False
        if not x["new_best"] and stall_run >= 2 and push_ok.get(day - 1, 0) >= 0.10:
            shift += 1
            pushed = True
        best = x["best_depth"] + shift
        new_best = x["new_best"] or pushed
        stall_run = 0 if new_best else stall_run + 1
        # income: everything measured but the frontier bank; a pushed floor's depth mark; the night's mark
        # (one per >= 4 h absence that banked)
        nights = 1 if x["banked"] + x["returned"] > 0 else 0
        income = sum(src.values()) + nights + (1 if pushed else 0)
        tier = 1 + boss_met(best) if x["banked"] > 0 or day > 1 else 0
        per_ci = [income / len(sched)] * len(sched)
        if day == 1:
            per_ci = [income * w for w in ([0.5, 0.1, 0.2, 0.1, 0.1] if rater else [0.6, 0.25, 0.15])]
        bought, draws, worst = [], 0, 0.0
        for inc in per_ci:
            marks += inc
            # rows first (the only unlock measured meaningful on most lineages), then the cheapest open
            while True:
                opts = [c for c in CATALOGUE if c[0] not in owned and c[2] <= tier and c[1] <= marks]
                if not opts:
                    break
                opts.sort(key=lambda c: (not c[0].startswith("row"), c[1], c[0]))
                c = opts[0]
                owned.add(c[0])
                marks -= c[1]
                bought.append({"id": c[0], "cost": c[1], "tier": c[2]})
            # the late sink: a fresh oath draw once the board is open, when nothing open is left to buy or
            # the marks would pass a reserve of 6
            while oath_open_day is not None and marks >= OATH_DRAW and not [c for c in CATALOGUE if c[0] not in owned and c[2] <= tier]:
                marks -= OATH_DRAW
                draws += 1
            worst = max(worst, marks)
        # gold: measured income; measured forge spend and restock; the automations once (forge units);
        # oaths at a quarter of the day's net a slot; commissions (policy-neutral lineage works) take the purse
        # above one day's net
        gin = x["gold"]["earned"]
        restock = x["gold"]["by_sink"].get("unledgered", 0) + x["gold"]["by_sink"].get("insure", 0)
        gout_forge = sum(f["gold"] for f in x["forge"])
        unit = unit or (100 + 25 * max(best, 1))
        net = max(gin - restock, 0)
        purse += gin - gout_forge - restock - x["gold"]["by_sink"].get("oath", 0) + 0
        purse = max(purse, 0)
        auto_spent = 0
        if best >= 18:
            for aid, mult in AUTOMATIONS:
                if aid not in autos and purse >= mult * unit:
                    autos.add(aid)
                    purse -= mult * unit
                    auto_spent += mult * unit
        oath_slots = (1 if oath_open_day is not None else 0) + ("oath_slot_2" in owned) + ("oath_slot_3" in owned)
        oath_spent = max(0, min(purse, int(oath_slots * 0.25 * net)))
        purse -= oath_spent
        comm_spent = 0
        while purse > max(net, 10 * unit):
            price = int(10 * unit * 1.25 ** commissions)
            if purse - price < 0:
                break
            purse -= price
            comm_spent += price
            commissions += 1
        # oaths: a sworn oath is kept every OATH_KEEP_EVERY days a slot while the pool lasts, each keep an unlock
        kept = []
        if oath_open_day is not None:
            k = day - oath_open_day
            # one slot: every OATH_KEEP_EVERY days; two or more: 4 days in 5 (1 - 0.45^2 ~ 0.8)
            due = (k % OATH_KEEP_EVERY == OATH_KEEP_EVERY - 1) if oath_slots <= 1 else (k % 5 != 4)
            if due and pool:
                kept.append(pool.pop(0))
        oath_rewards.extend(kept)
        systems = systems_open(day, best, bosses_cum, stall_run > 0, owned, oath_rewards)
        if "oaths" in systems and oath_open_day is None:
            oath_open_day = day
        new_sys = sorted(systems - prev_systems)
        prev_systems = systems
        out_days.append({
            "day": day,
            "marks": {"earned": round(income, 1), "by_source": {**src, "night": nights, "push_depth": int(pushed)}, "frontier_removed": frontier,
                      "spent": sum(b["cost"] for b in bought) + OATH_DRAW * draws, "spent_on": bought, "oath_draws": draws,
                      "unspent_end": round(marks, 1), "unspent_max": round(worst, 1)},
            "gold": {"earned": gin, "spent_restock": restock, "spent_forge": gout_forge, "spent_automations": auto_spent, "spent_oaths": oath_spent,
                     "spent_commissions": comm_spent, "end": purse, "net_day": net},
            "best_depth": best, "new_best": new_best, "pushed": pushed, "tier_open": tier,
            "systems_open": sorted(systems), "systems_new": new_sys, "systems_count": len(systems),
            "oath_kept": kept,
            "unlock_day": bool(bought) or bool(kept) or bool(new_sys and day > 1),
        })
    ud = sum(1 for d in out_days if d["unlock_day"])
    ud_content = sum(1 for d in out_days if d["marks"]["spent_on"] or d["oath_kept"])
    stall, cur = 0, 0
    for d in out_days:
        cur = 0 if d["new_best"] else cur + 1
        stall = max(stall, cur)
    purse_ratio = max((d["gold"]["end"] / max(d["gold"]["net_day"], 10 * (unit or 300))) for d in out_days[3:])
    return {
        "summary": {"name": s["name"], "seed": s["seed"], "mode": s["mode"],
                    "unlock_days": ud, "unlock_days_content_only": ud_content,
                    "marks_unspent_worst_after_day2": max(d["marks"]["unspent_max"] for d in out_days[2:]),
                    "stall_projected": stall, "stall_measured": s["stall_raw"], "final_best_projected": out_days[-1]["best_depth"],
                    "gold_end": out_days[-1]["gold"]["end"], "purse_days_worst": round(purse_ratio, 1), "owned_end": len(owned),
                    "oaths_kept": len(oath_rewards)},
        "days": out_days,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--timeline", default="docs/progression/timeline.json")
    ap.add_argument("--walls", default="docs/progression/walls.json")
    ap.add_argument("--out", default="docs/progression/proposal.json")
    a = ap.parse_args()
    t = json.load(open(a.timeline))
    walls = collections.defaultdict(list)
    if a.walls:
        for l in json.load(open(a.walls))["lineages"]:
            for x in l["days"]:
                for w in x["probe"]:
                    if w["id"] == "push":
                        walls[(l["name"], l["seed"])].append((x["day"], w["beyond"] + w["beyond_base"]))
    outs = [project(l, walls, a) for l in t["lineages"]]
    for o in outs:
        s = o["summary"]
        print(f'{s["name"]:24} s{s["seed"]} unlock-days {s["unlock_days"]:>2} (content {s["unlock_days_content_only"]:>2}) ◆worst {s["marks_unspent_worst_after_day2"]:>5} stall {s["stall_projected"]:>2} (measured {s["stall_measured"]:>2}) D{s["final_best_projected"]} $end {s["gold_end"]:>7} purse/day {s["purse_days_worst"]} owned {s["owned_end"]} oaths {s["oaths_kept"]}')
    agg = {}
    for m in ["dayplayer", "dayplayer+sinks", "rater"]:
        xs = [o["summary"] for o in outs if o["summary"]["mode"] == m]
        if xs:
            agg[m] = {"n": len(xs), "unlock_days_mean": round(st.mean(x["unlock_days"] for x in xs), 1), "unlock_days_min": min(x["unlock_days"] for x in xs),
                      "marks_unspent_worst": max(x["marks_unspent_worst_after_day2"] for x in xs), "stall_projected_max": max(x["stall_projected"] for x in xs),
                      "stall_le3": sum(1 for x in xs if x["stall_projected"] <= 3), "unlock_ge10": sum(1 for x in xs if x["unlock_days"] >= 10)}
    print(json.dumps(agg, indent=1))
    # the per-day system table: how many lineages have each system open, by day
    sys_table = []
    for day in range(len(outs[0]["days"])):
        row = {"day": day + 1}
        for m in ["dayplayer", "rater"]:
            ls = [o for o in outs if o["summary"]["mode"] == m]
            row[m] = {name: sum(1 for o in ls if name in o["days"][day]["systems_open"]) for name, _ in SYSTEMS}
            row[m + "_median_open"] = st.median(o["days"][day]["systems_count"] for o in ls)
        sys_table.append(row)
    doc = {"systems_by_day": sys_table,"projection_of": a.timeline, "oath_pool": OATH_POOL, "oath_keep_every_days": OATH_KEEP_EVERY, "method": __doc__.strip().splitlines()[2:8], "catalogue": CATALOGUE, "oath_draw": OATH_DRAW,
           "automations_gold_units": AUTOMATIONS, "systems": SYSTEMS, "aggregate": agg, "lineages": outs}
    json.dump(doc, open(a.out, "w"), indent=1)


if __name__ == "__main__":
    main()
