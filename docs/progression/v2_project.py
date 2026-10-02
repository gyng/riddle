#!/usr/bin/env python3
"""Project the v2 progression curve (docs/PROGRESSION_V2.md) for IDLE, PICKED and TUNED over 30 days
(and a sketch to 90), against the cadence targets.

Two steps:

  1. --calibrate LOG: read a verbose Cut 30 dayplayer log
       dayplayer --seeds 8 --days 14 --bots idle,picked,tuned --verbose 2> LOG
     and write docs/progression/v2_calibration.json: per bot, the median best depth at each 8 h
     check-in (the climb of a fresh lineage, i.e. of a first expedition), runs per check-in, and
     *today's* cadence (what each check-in's report announced: a dump of 5-15 things on day 1).
  2. (default) the projection: the v2 curriculum (systems with a trigger AND a lineage-age gate, at
     most one new system per check-in), the automation ladder, the prestige layers (expedition ->
     era -> dynasty) and the four tracks' stages, played on a real-time schedule of sessions per
     profile, over the calibrated climb. Writes docs/progression/v2_projection.json and prints the
     cadence table against the targets.

What is measured: the climb per bot (depth by hours of a fresh lineage), runs per hour, today's
cadence. What is assumed (every constant below is a design knob, named in PROGRESSION_V2.md §5):
the renown formula and its effect on the climb, the head start, the parallel-hero speed-up, the
sessions per day, the age gates. The model is a schedule, not a simulation: it answers "when does
each thing land and how often", not "is it balanced".

    python3 docs/progression/v2_project.py [--calibrate LOG] [--days 30] [--sketch 90]
"""
import argparse, json, math, re, statistics as st, collections

HERE = "docs/progression/"
BOSSES = {8: "Warlord", 13: "Mother", 18: "Lich", 23: "Foundry", 28: "Queen", 33: "Mirror King"}
BOTTOM = 34

# ---------------------------------------------------------------- calibration (measured)

def calibrate(log, out):
    ci_best = collections.defaultdict(list)     # (bot, day, ci) -> [best]
    ci_runs = collections.defaultdict(list)
    ann = collections.defaultdict(list)          # (bot, day) -> [announced items per report]
    day_best = collections.defaultdict(list)     # (bot, day) -> [best] (per seed)
    for line in open(log):
        m = re.match(r"\s+\[(\w+)\] day (\d+) ci (\d+) runs (\d+) .* best D(\d+)", line)
        if m:
            b, d, c, r, best = m.group(1), int(m.group(2)), int(m.group(3)), int(m.group(4)), int(m.group(5))
            ci_best[(b, d, c)].append(best)
            ci_runs[(b, d, c)].append(r)
            continue
        m = re.match(r"\s+\[(\w+)\] s(\d+) day (\d+) D(\d+)", line)
        if m:
            day_best[(m.group(1), int(m.group(3)))].append(int(m.group(4)))
            continue
        m = re.match(r"\s+\[(\w+)\] day (\d+) (.+)$", line)
        if m and " ci " not in line and " wall D" not in line:
            ann[(m.group(1), int(m.group(2)))].append(len(m.group(3).split(" · ")))
    bots = sorted({k[0] for k in ci_best})
    cal = {"source": log, "bots": {}}
    for b in bots:
        pts, runs = [], []
        for (bb, d, c), v in sorted(ci_best.items()):
            if bb == b:
                pts.append({"hours": (d - 1) * 24 + (c + 1) * 8, "best": st.median(v), "n": len(v)})
                runs.append(st.median(ci_runs[(bb, d, c)]))
        days = sorted({d for (bb, d) in day_best if bb == b})
        cal["bots"][b] = {
            "climb": pts,
            "runs_per_checkin": st.median(runs) if runs else 16,
            "best_by_day": {d: {"median": st.median(day_best[(b, d)]), "min": min(day_best[(b, d)]), "max": max(day_best[(b, d)])} for d in days},
            # today's cadence: announced items per report (a report that announced anything), by day
            "announced_by_day": {d: {"reports": len(ann[(b, d)]), "items_median": st.median(ann[(b, d)]), "items_max": max(ann[(b, d)])} for d in days if ann[(b, d)]},
        }
    json.dump(cal, open(out, "w"), indent=1)
    print(f"wrote {out}: " + ", ".join(f"{b} {len(cal['bots'][b]['climb'])} pts" for b in bots))


# ---------------------------------------------------------------- the v2 design (knobs)

# Profiles: sessions (hours of the lineage's clock at which the player looks) and what they tap.
#   first session: continuous play, a check-in every 5 min of it (the watch, the report, the camp)
#   later: check-ins spread over the waking day (GameAnalytics: 5.3 sessions/day for idle players)
PROFILES = {
    "IDLE":   {"first_min": 20, "per_day": 3, "bot": "IDLE",   "picks": False, "pen": False},
    "PICKED": {"first_min": 45, "per_day": 4, "bot": "PICKED", "picks": True,  "pen": False},
    "TUNED":  {"first_min": 90, "per_day": 6, "bot": "TUNED",  "picks": True,  "pen": True},
}
ACTIVE_RUN_MIN = 4.0          # a watched run (50 cards: 14 runs in 55 active min)

# Renown (★), the expedition's currency. Lifetime-based (Cookie Clicker's cube root, AdCap's square
# root): each expedition adds glory G += g(peak), convex in the peak (D13 5 · D18 10 · D23 17 · D28 26 ·
# D34 38), and ★ = floor(3·sqrt(G)); a same-depth reset still pays, less each time, a deeper one pays
# more, and a 20 h cooldown means a short reset loop is never the optimum.
def glory(peak):
    return int(peak * peak / 30)


def stars_of(G):
    return int(3 * math.sqrt(G))

EXPEDITION_GATE = {"best_ever": 13, "age_h": 36, "cooldown_h": 20}   # the Mother met; lineage ≥ 36 h; ≥ 20 h since the last
ERA_GATE = {"best": BOTTOM, "expeditions": 2, "age_h": 10 * 24, "cooldown_h": 7 * 24}   # the Mirror King slain; ≥ 2 expeditions this era; a week since the last era
DYNASTY_GATE = {"eras": 3, "age_h": 45 * 24}

def speed(stars, era, lanes):
    """How much faster an expedition climbs than a fresh lineage (h_eff = h × speed)."""
    return (1 + 0.15 * math.sqrt(stars)) * (1.2 ** era) * (1 + 0.25 * (lanes - 1))

def lift(stars, era):
    """Floors ★ and eras add to the fresh climb at the same hours (walls yield to time, not HP)."""
    return 2.5 * math.sqrt(stars / 5) + 2 * era

def head_start(prev_peak, systems):
    """Where a new expedition starts: the deepest boss floor ≤ half the last peak (the run-back is one
    absence, not a week: research/idle-attraction.md "minutes, not hours"); the base camp (expedition
    2's reward) raises it to 60 %."""
    frac = 0.6 if "base camp" in systems else 0.5
    return max([1] + [d for d in BOSSES if d <= frac * prev_peak])

# The curriculum: (id, track, kind, trigger, min lineage age in hours). kind: system (a new verb,
# panel or building that changes what a check-in can do) | stage (a new thing on a track) |
# auto (an automation rung) | prestige. At most ONE system/auto/prestige opens per check-in (the
# queue; one automation rung besides, on its own queue); stages are not capped. A trigger met before its age waits in the queue; an idle player
# ages at the same rate, so the age gate never blocks him more than an engaged one.
def T(**k):
    return k

CURRICULUM = [
    # --- day 0 (the first session)
    ("send · watch · report", "character", "system", T(), 0),
    ("blacksmith", "town", "system", T(runs=1), 0),
    ("storehouse", "items", "system", T(runs=3), 0),
    ("kennel · first pet", "scale", "system", T(best=5), 0.25),
    ("a second stance", "character", "system", T(best=8), 0.5),
    # --- day 1
    ("tactics", "character", "system", T(slain=8), 4),
    ("bank", "town", "system", T(runs=20), 8),
    ("quest board", "town", "system", T(slain=8), 12),
    ("rogue", "character", "system", T(banks=1), 16),
    ("waystones", "scale", "system", T(slain=8), 20),
    # --- days 2-3
    ("temperament", "character", "system", T(heirs=3), 24),
    ("library · drills", "town", "system", T(best=13), 30),
    ("expedition", "prestige", "prestige", T(best_ever=13), 36),
    ("ranger", "character", "system", T(slain=13), 48),
    ("tactic slot 2", "character", "system", T(best=18), 56),
    # --- week 1
    ("the pen", "character", "system", T(best_ever=13, expeditions=1), 72),
    ("house 1 · second hero", "scale", "system", T(expeditions=1), 72),
    ("tavern", "town", "system", T(expeditions=1, runs=150), 96),
    ("caster", "character", "system", T(slain=18), 96),
    ("loot properties", "items", "system", T(expeditions=2), 110),
    ("watchtower · routes", "town", "system", T(expeditions=2), 130),
    ("base camp", "scale", "system", T(expeditions=2), 140),
    # --- weeks 2-4
    ("house 2 · third hero", "scale", "system", T(expeditions=3), 8 * 24),
    ("relics", "items", "system", T(expeditions=4), 10 * 24),
    ("hall of heirs", "town", "system", T(heirs=5), 11 * 24),
    ("era", "prestige", "prestige", T(best=BOTTOM, expeditions=3), 10 * 24),
    ("advanced classes", "character", "system", T(expeditions=4, slain=23), 14 * 24),
    ("house 3 · expedition board", "scale", "system", T(expeditions=5), 17 * 24),
    ("era variant: no rest", "prestige", "system", T(eras=1), 18 * 24),
    ("bloodlines", "character", "system", T(eras=1), 20 * 24),
    ("relic sets", "items", "system", T(eras=1, expeditions=6), 24 * 24),
    # --- months 2+
    ("era variant: short list", "prestige", "system", T(eras=2), 30 * 24),
    ("company · lanes follow a record", "scale", "system", T(eras=2), 34 * 24),
    ("era variant: bones only", "prestige", "system", T(eras=3), 42 * 24),
    ("dynasty", "prestige", "prestige", T(eras=3), 45 * 24),
    ("era variant: hunted", "prestige", "system", T(eras=4), 56 * 24),
    ("a new biome lane (content)", "items", "system", T(dynasties=1), 60 * 24),
]

# §4 rule 3: a fallback age for skill/milestone triggers, so nothing waits on a skill or a tap the idle player
# lacks (hours of lineage age; the trigger counts as met past it).
FALLBACK_AGE_H = {
    "temperament": 72, "the pen": 120, "expedition": 72, "house 1 · second hero": 5 * 24,
    "house 2 · third hero": 12 * 24, "relics": 14 * 24, "advanced classes": 18 * 24, "house 3 · expedition board": 20 * 24,
    "relic sets": 26 * 24, "auto-expedition": 21 * 24, "lanes: auto-send heroes": 22 * 24,
}

# The automation ladder: each rung retires a chore the player has already done by hand (Trimps,
# Antimatter Dimensions: automation is earned), each named, shown and revocable; policy rungs
# (stance swap, wall edit) are opt-in toggles, never silent.
AUTOMATION = [
    ("restock (quartermaster)", T(), 0),                                   # the idle floor: given
    ("auto-equip the better find", T(runs=6), 0.5),
    ("standing order: forge steps", T(forge_steps=3), 20),
    ("keep / salvage sorter", T(runs=60), 28),
    ("bank sweep", T(bank_days=3), 48),
    ("quest claim + reroll", T(quests=2), 60),
    ("stance at the wall (opt-in)", T(expeditions=1, picks=True), 80),
    ("kennel: field + hatch", T(runs=250), 100),
    ("route and start stone", T(expeditions=2), 150),
    ("wall edit as a drill (opt-in)", T(expeditions=3, pen=True), 9 * 24),
    ("auto-expedition", T(expeditions=5), 13 * 24),
    ("lanes: auto-send heroes", T(expeditions=5, lanes=3), 16 * 24),
    ("auto-era with a chosen variant", T(eras=2), 35 * 24),
]


# ---------------------------------------------------------------- the model

def climb_fn(cal_bot):
    pts = [(0.0, 1.0), (0.25, 4.0), (1.0, 7.0)] + [(p["hours"], float(p["best"])) for p in cal_bot["climb"]]
    pts.sort()
    # monotone envelope
    env, m = [], 0
    for h, d in pts:
        m = max(m, d)
        env.append((h, m))

    def f(h):
        if h <= env[0][0]:
            return env[0][1]
        for (h0, d0), (h1, d1) in zip(env, env[1:]):
            if h <= h1:
                return d0 + (d1 - d0) * (h - h0) / (h1 - h0)
        # past the measured 14 days: the last week's slope, halving each week
        (hA, dA), (hB, dB) = env[-8], env[-1]
        slope = (dB - dA) / max(hB - hA, 1)
        extra, t, s = 0.0, min(h - hB, 168 * 40), slope
        while t > 0:
            step = min(t, 168)
            extra += s * step
            s /= 2
            t -= step
        return dB + extra
    return f, env[-1][1]


def schedule(p, days):
    """Check-in times in hours of the lineage's clock."""
    ts = [m / 60 for m in range(5, p["first_min"] + 1, 5)]
    for d in range(days):
        base = d * 24
        n = p["per_day"]
        # waking hours 8:00-23:00 relative to a first session at 20:00 on day 0
        for i in range(n):
            t = base + 12 + (i + 1) * 15 / (n + 1)       # spread over the next day's waking hours
            if t > ts[-1]:
                ts.append(t)
    return ts


def met(trig, s):
    for k, v in trig.items():
        if k == "picks":
            if v and not s["picks"]:
                return False
        elif k == "pen":
            if v and not s["pen"]:
                return False
        elif s.get(k, 0) < v:
            return False
    return True


CLASS_LEVEL_RUNS = [3, 10, 30, 80, 110, 150, 220, 330, 450]       # L2..L10 (IDLE measured: L4 day 1, L7 day 3, L10 day 9)
PKG_LEVEL_RUNS = [10, 40, 120, 300]                                # Cut 30 packages L2..L5
PACKAGE_SLOTS = ["send · watch · report", "tactics", "temperament", "tactic slot 2"]   # the stance, then the slots


def project(name, p, cal, days):
    climb, fresh_plateau = climb_fn(cal["bots"][p["bot"]])

    memo = {}

    def hours_to(floor):
        if floor not in memo:
            lo, hi = 0.0, 24 * 120.0
            for _ in range(40):
                mid = (lo + hi) / 2
                lo, hi = (mid, hi) if climb(mid) < floor else (lo, mid)
            memo[floor] = hi
        return memo[floor]
    rph_off = cal["bots"][p["bot"]]["runs_per_checkin"] / 8.0
    s = collections.Counter()
    s.update({"picks": p["picks"], "pen": p["pen"]})
    ages = {c[0]: c[4] for c in CURRICULUM}
    ages.update({a[0]: a[2] for a in AUTOMATION})
    kinds = {c[0]: (c[2], c[1]) for c in CURRICULUM}
    kinds.update({a[0]: ("auto", "automation") for a in AUTOMATION})
    opened, systems, events, queue = set(), set(), [], []
    slot_runs = {}                       # package slot -> runs when it opened
    G, stars, era, dyn = 0, 0, 0, 0
    peak_ever, exp_start, exp_peak, exp_peak_t, start_floor, last_exp = 0, 0.0, 0, 0.0, 1, -1e9
    last_t, prev_best = 0.0, 0
    exp_hist, eod, era_peak, last_era = [], {}, 0, -1e9
    ts = schedule(p, days)
    for ci, t in enumerate(ts):
        first_session = t <= p["first_min"] / 60 + 1e-9
        dt = t - last_t
        lanes = 1 + sum(x in systems for x in ("house 1 · second hero", "house 2 · third hero", "house 3 · expedition board"))
        runs = (dt * 60 / ACTIVE_RUN_MIN if first_session else dt * rph_off) * lanes
        s["runs"] += runs
        h_eff = hours_to(start_floor) + (t - exp_start) * speed(stars, era, lanes)
        best = min(BOTTOM, max(start_floor, int(climb(h_eff) + lift(stars, era))))
        if first_session:
            best = min(best, 1 + int(2.2 * math.sqrt(s["runs"])))    # the watched first hour: D6-8 (dayplayer)
        best = max(best, prev_best)
        new = []
        for d in BOSSES:
            if prev_best < d <= best and f"{BOSSES[d]} met" not in opened:
                opened.add(f"{BOSSES[d]} met")
                new.append(("stage", "character", f"{BOSSES[d]} met"))
        if best > peak_ever and peak_ever > 0:
            new.append(("record", "character", f"record D{best}"))
        if era > 0:
            for d in BOSSES:
                if era_peak < d <= best:
                    new.append(("record", "character", f"era {era} · {BOSSES[d]} met"))
            era_peak = max(era_peak, best)
        if best > exp_peak:
            exp_peak, exp_peak_t = best, t
        prev_best = best
        peak_ever = max(peak_ever, best)
        s.update({"best": 0})
        s["best"], s["best_ever"] = best, peak_ever
        s["slain"] = max([0] + [d for d in BOSSES if best > d])
        s["banks"] = int(s["runs"] > 2)
        s["heirs"] = 1 + int(s["runs"] / 25)
        s["forge_steps"] = min(12, int(math.log2(1 + s["runs"] / 3)) + (2 if p["picks"] else 0))
        s["bank_days"] = max(0, int((t - 8) / 24)) if "bank" in systems else 0
        q = int(max(0, t - 12) / 24 * 0.5) if "quest board" in systems else 0     # a quest kept every 2nd day (PROGRESSION.md §7's keep rate)
        if q > s["quests"]:
            new.append(("stage", "town", f"quest done {q}"))
        s["quests"] = q
        s["lanes"], s["eras"], s["dynasties"] = lanes, era, dyn
        if s["runs"] >= 2 and "first death · heir 2" not in opened:
            opened.add("first death · heir 2")
            new.append(("stage", "character", "first death · heir 2"))
        for lv, r in enumerate(CLASS_LEVEL_RUNS, start=2):
            if s["runs"] >= r and f"class L{lv}" not in opened:
                opened.add(f"class L{lv}")
                new.append(("stage", "character", f"class L{lv}"))
        for slot, r0 in slot_runs.items():
            for lv, r in enumerate(PKG_LEVEL_RUNS, start=2):
                key = f"{slot} L{lv}"
                if s["runs"] - r0 >= r and key not in opened:
                    opened.add(key)
                    new.append(("stage", "character", f"package L{lv} ({slot})"))
        # the curriculum: stages uncapped; ONE system/auto/prestige opens per check-in (first session: every 5 min)
        for cid, track, kind, trig, age in CURRICULUM:
            if cid not in opened and cid not in queue and (met(trig, s) or t >= FALLBACK_AGE_H.get(cid, 1e9)):
                queue.append(cid)
        for aid, trig, age in AUTOMATION:
            if aid not in opened and aid not in queue and (met(trig, s) or t >= FALLBACK_AGE_H.get(aid, 1e9)):
                queue.append(aid)
        # one system (or prestige) and one automation rung per check-in: a rung sits on its building,
        # not in the reveal queue, so an idle player's rungs don't wait behind his systems
        ready = sorted((q for q in queue if ages[q] <= t), key=lambda q: {"prestige": 0, "system": 1, "auto": 2}[kinds[q][0]])
        picks = [x for x in ready if kinds[x][0] != "auto"][:1] + [x for x in ready if kinds[x][0] == "auto"][:1]
        for q in picks:
            queue.remove(q)
            opened.add(q)
            k, track = kinds[q]
            if k == "system":
                systems.add(q)
            if q in PACKAGE_SLOTS:
                slot_runs[q] = s["runs"]
            new.append((k, track, q))
        # relics: one per boss first slain in an expedition once relics are open (a ledger that fills)
        if "relics" in systems and s["slain"] and f"relic {era}.{s['expeditions']}.{s['slain']}" not in opened:
            opened.add(f"relic {era}.{s['expeditions']}.{s['slain']}")
            new.append(("stage", "items", f"relic ({BOSSES[s['slain']]})"))
        # the expedition: a one-tap claim on the report (IDLE takes it too — it is the harvest, not a pick)
        if "expedition" in opened and t - last_exp >= EXPEDITION_GATE["cooldown_h"] and not (best >= BOTTOM and "era" in opened and sum(e["era"] == era for e in exp_hist) >= ERA_GATE["expeditions"] and t - last_era >= ERA_GATE["cooldown_h"]):
            gain = stars_of(G + glory(exp_peak)) - stars
            # the report suggests it (the tap is the player's): a big gain after a check-in with no new
            # floor, or any gain after a day with no new floor
            quiet = t - exp_peak_t
            first = not exp_hist
            last_peak = exp_hist[-1]["peak"] if exp_hist and exp_hist[-1]["era"] == era else 0
            matched = exp_peak >= last_peak - 2          # the report suggests it once the old peak is (nearly) back
            if first or (matched and gain >= max(2, 0.25 * stars) and quiet >= 8) or (gain >= 1 and quiet >= 48):
                G += glory(exp_peak)
                stars += gain
                s["expeditions"] += 1
                exp_hist.append({"t": round(t, 1), "day": int(t // 24) + 1, "peak": exp_peak, "gain": gain, "stars": stars, "era": era})
                new.append(("prestige", "prestige", f"expedition {s['expeditions']} · ★+{gain}"))
                start_floor = head_start(exp_peak, systems)
                exp_start, exp_peak, exp_peak_t, last_exp, prev_best = t, start_floor, t, t, start_floor
        if "era" in opened and best >= BOTTOM and sum(e["era"] == era for e in exp_hist) >= ERA_GATE["expeditions"] and t - last_era >= ERA_GATE["cooldown_h"]:
            era += 1
            G = int(G ** 0.5)
            stars = stars_of(G)
            new.append(("prestige", "prestige", f"era {era}"))
            start_floor, era_peak, last_era = 1, 1, t
            exp_start, exp_peak, exp_peak_t, last_exp, prev_best = t, 1, t, t, 1
        if "dynasty" in opened and era >= DYNASTY_GATE["eras"] and dyn == 0:
            dyn = 1
            new.append(("prestige", "prestige", "dynasty 1"))
        for k, track, what in new:
            events.append({"t": t, "day": max(1, int(t // 24) + 1), "kind": k, "track": track, "what": what})
        last_t = t
        eod[max(1, int(t // 24) + 1)] = (best, len(systems), stars, era, s["expeditions"], sorted(systems))
    per_day, last = [], (1, 0, 0, 0, 0, [])
    for d in range(1, days + 1):
        ev = [e for e in events if e["day"] == d]
        last = eod.get(d, last)
        per_day.append({
            "day": d, "best": last[0], "systems_open": last[1], "stars": last[2], "era": last[3], "expeditions": last[4],
            "new_things": len(ev), "new_systems": sum(1 for e in ev if e["kind"] in ("system", "prestige", "auto")),
            "what": [e["what"] for e in ev],
        })
    return events, per_day, exp_hist


def cadence(events, per_day, days):
    fs = [e["t"] * 60 for e in events if e["t"] <= 1.0]
    gaps_first_hour = [b - a for a, b in zip([0] + fs, fs)]
    day_new = [d["new_things"] > 0 for d in per_day[:days]]
    gap, cur = 0, 0
    for x in day_new:
        cur = 0 if x else cur + 1
        gap = max(gap, cur)
    sys_days = [d for d in per_day[:days] if d["new_systems"] > 0]
    return {
        "first_10_min_new": sum(1 for e in events if e["t"] <= 1 / 6 + 1e-9),
        "first_hour_new": len(fs),
        "first_hour_longest_gap_min": round(max(gaps_first_hour), 1) if gaps_first_hour else None,
        "day1_systems_open": per_day[0]["systems_open"],
        "day1_new_systems": per_day[0]["new_systems"],
        "days_with_something_new": sum(day_new),
        "days_with_a_system": len(sys_days),
        "longest_gap_days": gap,
        "systems_open_by_day": [d["systems_open"] for d in per_day[:days]],
        "max_beats_per_checkin": max(collections.Counter(e["t"] for e in events if e["day"] <= days).values()),
    }


TARGETS = {
    "first_10_min_new": (">=", 2), "first_hour_new": (">=", 6), "first_hour_longest_gap_min": ("<=", 15),
    "day1_systems_open": ("<=", 10),
    "days_with_something_new@30": (">=", 26), "longest_gap_days@30": ("<=", 2),
    "days_with_a_system@7": (">=", 6), "days_with_a_system@30": (">=", 16),
    "longest_gap_days@90": ("<=", 7),
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--calibrate")
    ap.add_argument("--cal", default=HERE + "v2_calibration.json")
    ap.add_argument("--out", default=HERE + "v2_projection.json")
    ap.add_argument("--days", type=int, default=30)
    ap.add_argument("--sketch", type=int, default=90)
    a = ap.parse_args()
    if a.calibrate:
        calibrate(a.calibrate, a.cal)
        return
    cal = json.load(open(a.cal))
    out = {"method": __doc__.strip().splitlines()[:3], "targets": {k: list(v) for k, v in TARGETS.items()}, "profiles": {}}
    rows = []
    for name, p in PROFILES.items():
        ev, pdays, exps = project(name, p, cal, a.sketch)
        c30 = cadence(ev, pdays, a.days)
        c7 = cadence(ev, pdays, 7)
        c90 = cadence(ev, pdays, a.sketch)
        checks = {
            "first_10_min_new": c30["first_10_min_new"], "first_hour_new": c30["first_hour_new"],
            "first_hour_longest_gap_min": c30["first_hour_longest_gap_min"], "day1_systems_open": c30["day1_systems_open"],
            "days_with_something_new@30": c30["days_with_something_new"], "longest_gap_days@30": c30["longest_gap_days"],
            "days_with_a_system@7": c7["days_with_a_system"], "days_with_a_system@30": c30["days_with_a_system"],
            "longest_gap_days@90": c90["longest_gap_days"],
        }
        def first_day(pred):
            return next((e["day"] for e in ev if pred(e)), None)
        milestones = {
            "first_expedition_day": first_day(lambda e: e["what"].startswith("expedition 1")),
            "first_era_day": first_day(lambda e: e["what"] == "era 1"),
            "first_D34_day": first_day(lambda e: e["what"] in ("record D34", "era 1")),
            "pen_day": first_day(lambda e: e["what"] == "the pen"),
            "second_hero_day": first_day(lambda e: e["what"].startswith("house 1")),
            "expeditions_by_30": pdays[a.days - 1]["expeditions"], "eras_by_90": pdays[-1]["era"],
            "systems_open_day_1_3_7_14_30": [pdays[d - 1]["systems_open"] for d in (1, 3, 7, 14, 30)],
            "autos_by_day": {d: sum(1 for e in ev if e["kind"] == "auto" and e["day"] <= d) for d in (1, 3, 7, 14, 30, 90)},
        }
        verdict = {k: (v is not None and (v >= TARGETS[k][1] if TARGETS[k][0] == ">=" else v <= TARGETS[k][1])) for k, v in checks.items()}
        today = cal["bots"][p["bot"]].get("announced_by_day", {})
        out["profiles"][name] = {"profile": p, "milestones": milestones, "cadence_30": c30, "cadence_7": c7, "cadence_90": c90, "checks": checks, "pass": verdict,
                                 "expeditions": exps, "days": pdays, "events": ev, "today_announced_by_day": today}
        rows.append((name, checks, verdict, pdays))
        print(name, json.dumps(milestones))
    json.dump(out, open(a.out, "w"), indent=1)
    keys = list(TARGETS)
    print("check".ljust(30) + "".join(n.rjust(10) for n, *_ in rows) + "   target")
    for k in keys:
        print(k.ljust(30) + "".join((f"{r[1][k]}{'' if r[2][k] else '!'}").rjust(10) for r in rows) + f"   {TARGETS[k][0]} {TARGETS[k][1]}")
    print("\nday  " + "".join(f"{n:>26}" for n, *_ in rows))
    for d in [1, 2, 3, 4, 5, 7, 10, 14, 21, 30, 45, 60, 90]:
        cells = []
        for n, c, v, pd in rows:
            x = pd[d - 1]
            cells.append(f"D{x['best']} sys{x['systems_open']} ★{x['stars']} e{x['expeditions']} E{x['era']} +{x['new_things']}".rjust(26))
        print(f"{d:>3}  " + "".join(cells))


if __name__ == "__main__":
    main()
