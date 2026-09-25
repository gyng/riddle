//! Cut 25 §1: kit vs rules. On a cohort set's own lineage (the set played for `hours` from a
//! fresh D0 lineage that owns its tokens, the shelf packed), the whole forge's move of the send's
//! bank share against the set's best single-row edit — both paired on the same sims (the same
//! seeds, the same floors: `floor_streams`). Shared by `examples/lever.rs` and `metrics.rs`.
#![allow(dead_code)]
use riddle_core::engine::ExitTier;
use riddle_core::forecast::SimResult;
use riddle_core::{Cond, Game, Row, RuleSet, Verb};

pub struct Lever {
    pub best_depth: u32,
    pub level: u32,
    pub base_bank: f64,
    /// The whole forge's paired bank move (every step of every ladder, the shelf refilled to its cap).
    pub kit: f64,
    /// Each ladder topped alone (weapon, armour, pack).
    pub kit_slots: [f64; 3],
    /// The best single-row edit's paired bank move, and the edit.
    pub row: f64,
    pub row_label: String,
    pub cands: usize,
    /// The largest loss of dropping one of the set's non-exit rows (the move writing it made),
    /// on the unkitted and on the kitted lineage, and the row.
    pub drop: (f64, String),
    pub drop_kitted: (f64, String),
    pub kit_bank: f64,
}

/// The tag the lever's sims share (any fixed tag: base, kit and every edit replay the same seeds).
const TAG: u64 = 0x001E_7E25;

pub fn cohort_game(set: &RuleSet, seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss"] {
        g.lineage.facts.insert(f.into());
    }
    for r in &set.rows {
        if let Some(c) = r.card() {
            g.lineage.unlocks.insert(c.into());
        }
        for c in &r.conds {
            if let Some(u) = riddle_core::meta::cond_unlock(&c.k) {
                g.lineage.unlocks.insert(u.into());
            }
        }
    }
    g.set_rules_raw(set.clone()).unwrap_or_else(|e| panic!("cohort set {:?}: {e}", set.name));
    g
}

/// The shelf packed for the set: two heals and one of every other kind its rows name, then heals
/// to the cap (a bigger pack carries more).
pub fn fill_shelf(g: &mut Game) {
    let gold = g.lineage.gold;
    g.lineage.gold = 1_000_000;
    let kinds = g.lineage.row_kinds();
    let cat = g.supply_catalogue();
    for k in kinds.iter().filter(|k| cat.iter().any(|e| e.kind == **k) && k.as_str() != "leash") {
        for _ in 0..if k == "heal" { 2 } else { 1 } {
            let _ = g.buy_supply(k);
        }
    }
    for _ in 0..riddle_core::kit::SUPPLY_CAP_MAX {
        if g.buy_supply("heal").is_err() {
            break;
        }
    }
    g.lineage.gold = gold;
}

fn sims_of(g: &Game, rules: &RuleSet, n: u32, prefix: Vec<SimResult>) -> Vec<SimResult> {
    riddle_core::forecast::simulate_budget_from(g, rules, n, TAG, 999, u64::MAX, prefix)
}

fn bank(r: &SimResult) -> f64 {
    (r.tier == ExitTier::Bank && !r.timed_out) as u8 as f64
}

fn share(rs: &[SimResult]) -> f64 {
    rs.iter().map(bank).sum::<f64>() / rs.len().max(1) as f64
}

/// Every single-row edit of `set` the lever weighs: a row removed, a row moved up (to any
/// place above) or down one, a number notched (hp ±10, others ±1 — never an exit row's depth
/// lowered: a shallower bank buys its share with the depth, not with play), and, under the cap,
/// the stock rows added at the top.
pub fn one_row_edits(g: &Game, set: &RuleSet) -> Vec<(String, RuleSet)> {
    let mut out: Vec<(String, RuleSet)> = Vec::new();
    let n = set.rows.len();
    let mut push = |label: String, s: RuleSet| {
        if s != *set && s.validate().is_ok() && !out.iter().any(|(_, o)| *o == s) {
            out.push((label, s));
        }
    };
    for i in 0..n {
        if n > 1 {
            let mut s = set.clone();
            s.rows.remove(i);
            push(format!("drop R{}", i + 1), s);
        }
        for j in 0..i {
            let mut s = set.clone();
            let r = s.rows.remove(i);
            s.rows.insert(j, r);
            push(format!("R{} above R{}", i + 1, j + 1), s);
        }
        if i + 1 < n {
            let mut s = set.clone();
            s.rows.swap(i, i + 1);
            push(format!("R{} below R{}", i + 1, i + 2), s);
        }
        let exit = matches!(set.rows[i].verb.v.as_str(), "bank" | "return");
        for (ci, c) in set.rows[i].conds.iter().enumerate() {
            let Some(v) = c.n else { continue };
            let step = if c.k.starts_with("hp") { 10 } else { 1 };
            for sign in [-1, 1] {
                if exit && c.k == "depth>=" && sign < 0 {
                    continue;
                }
                let nv = v + sign * step;
                if nv < 1 || (c.k.starts_with("hp") && nv > 95) {
                    continue;
                }
                let mut s = set.clone();
                s.rows[i].conds[ci].n = Some(nv);
                push(format!("R{} {}{}", i + 1, c.k, nv), s);
            }
        }
    }
    if n < g.lineage.max_rows() {
        let stock = [
            Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")),
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "gas"), Cond::n("adj>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
        ];
        for r in stock {
            if set.rows.contains(&r) {
                continue;
            }
            let mut s = set.clone();
            s.rows.insert(0, r.clone());
            push(format!("+ {} at R1", r.describe()), s);
        }
    }
    out
}

/// The lever on one (set, seed): the set played `hours` from a fresh lineage (its own best depth,
/// class level, facts), then the shelf filled; `sims` paired sends each for the base, the whole
/// forge, each ladder, and the best of the one-row edits (screened on a quarter of the sims, the
/// top three measured on all of them).
pub fn measure(set: &RuleSet, seed: u64, hours: u64, sims: u32) -> Lever {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    fill_shelf(&mut g);
    let _ = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
    g.run = None;
    g.pending_exit = None;
    g.lineage.gold = 0;
    fill_shelf(&mut g);
    let rules = g.lineage.rules().clone();
    let base = sims_of(&g, &rules, sims, Vec::new());
    let kitted = |slots: &[&str]| -> f64 {
        let mut k = g.sim_clone();
        for s in slots {
            k.lineage.kit.insert((*s).into(), riddle_core::kit::mults(s).len() as u32);
        }
        fill_shelf(&mut k);
        let a = sims_of(&k, &rules, sims, Vec::new());
        riddle_core::forecast::paired(&a, &base, bank).delta
    };
    let kit = kitted(&riddle_core::kit::KIT_SLOTS);
    let kit_slots = [kitted(&["weapon"]), kitted(&["armour"]), kitted(&["pack"])];
    let mut edits = one_row_edits(&g, &rules);
    // the offered patches: the absence's last deaths' own (the verdict's candidates, applied as the camp applies them)
    let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall).map(|(id, _)| *id).collect();
    let max_rows = g.lineage.max_rows();
    for id in ids.iter().rev().take(3) {
        let Some(d) = g.death(*id) else { continue };
        for p in &d.patches {
            let s = riddle_core::offline::apply_patch(&rules, p, max_rows);
            if s != rules && !edits.iter().any(|(_, o)| *o == s) {
                edits.push((format!("patch {}", p.row.describe()), s));
            }
        }
    }
    let screen = (sims / 4).max(24).min(sims);
    let base_s: Vec<SimResult> = base.iter().take(screen as usize).cloned().collect();
    let mut scored: Vec<(f64, usize)> = edits.iter().enumerate().map(|(i, (_, s))| (share(&sims_of(&g, s, screen, Vec::new())) - share(&base_s), i)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    let (mut row, mut row_label) = (f64::MIN, String::from("none"));
    for &(_, i) in scored.iter().take(3) {
        let a = sims_of(&g, &edits[i].1, sims, Vec::new());
        let d = riddle_core::forecast::paired(&a, &base, bank).delta;
        if d > row {
            row = d;
            row_label = edits[i].0.clone();
        }
    }
    let mut kg = g.sim_clone();
    riddle_core::kit::buy_all(&mut kg.lineage);
    fill_shelf(&mut kg);
    let kbase = sims_of(&kg, &rules, sims, Vec::new());
    let drop_on = |gg: &Game, b: &[SimResult]| -> (f64, String) {
        let mut best = (0.0f64, String::from("none"));
        for i in 0..rules.rows.len() {
            if matches!(rules.rows[i].verb.v.as_str(), "bank" | "return") || engagement(&rules.rows[i]) {
                continue;
            }
            let mut s = rules.clone();
            s.rows.remove(i);
            let a = sims_of(gg, &s, sims, Vec::new());
            let d = -riddle_core::forecast::paired(&a, b, bank).delta;
            if d > best.0 {
                best = (d, format!("R{} {}", i + 1, rules.rows[i].describe()));
            }
        }
        best
    };
    let drop = drop_on(&g, &base);
    let drop_kitted = drop_on(&kg, &kbase);
    let level = g.lineage.classes.get(g.lineage.class.name()).map(|c| c.level).unwrap_or(1);
    Lever { best_depth: g.lineage.best_depth, level, base_bank: share(&base), kit, kit_slots, row: row.max(0.0), row_label, cands: edits.len(), drop, drop_kitted, kit_bank: share(&kbase) }
}

/// The set's broad engagement row (`foes ≥ N → attack | shoot nearest`, nothing else): dropping
/// it is "never fight", not a lever — the lever's drops skip it.
pub fn engagement(r: &Row) -> bool {
    matches!(r.verb.v.as_str(), "attack" | "shoot") && r.verb.a.as_deref() == Some("nearest") && r.conds.iter().all(|c| c.k == "foes>=")
}

/// One (set, seed) of the gate: `(kit move, best row move, the row)` — paired bank shares on the
/// set's lineage after `hours` (its shelf full). The row move is the largest of: a non-exit,
/// non-engagement row dropped on the forged hero (what the row carries once the forge is bought),
/// the same on the bare hero, and the best one-row edit or offered patch on the bare hero
/// (screened on a quarter of the sims, the best three on all). Stops once a row beats the kit by
/// `margin` (the table prints the first that does).
pub fn gate(set: &RuleSet, seed: u64, hours: u64, sims: u32, margin: f64) -> (f64, f64, String) {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    fill_shelf(&mut g);
    let _ = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
    g.run = None;
    g.pending_exit = None;
    g.lineage.gold = 0;
    fill_shelf(&mut g);
    let rules = g.lineage.rules().clone();
    let base = sims_of(&g, &rules, sims, Vec::new());
    let mut kg = g.sim_clone();
    riddle_core::kit::buy_all(&mut kg.lineage);
    fill_shelf(&mut kg);
    let kbase = sims_of(&kg, &rules, sims, Vec::new());
    let kit = riddle_core::forecast::paired(&kbase, &base, bank).delta;
    let mut best = (0.0f64, String::from("none"));
    let rows: Vec<usize> = (0..rules.rows.len()).filter(|&i| !matches!(rules.rows[i].verb.v.as_str(), "bank" | "return") && !engagement(&rules.rows[i])).collect();
    for (gg, b, tag) in [(&kg, &kbase, "forged"), (&g, &base, "bare")] {
        for &i in &rows {
            let mut s = rules.clone();
            s.rows.remove(i);
            let d = -riddle_core::forecast::paired(&sims_of(gg, &s, sims, Vec::new()), b, bank).delta;
            if d > best.0 {
                best = (d, format!("drop R{} ({tag})", i + 1));
            }
            if best.0 > kit + margin {
                return (kit, best.0, best.1);
            }
        }
    }
    let edits = one_row_edits(&g, &rules);
    let screen = (sims / 4).max(16).min(sims);
    let base_s: Vec<SimResult> = base.iter().take(screen as usize).cloned().collect();
    let mut scored: Vec<(f64, usize)> = edits.iter().enumerate().map(|(i, (_, s))| (share(&sims_of(&g, s, screen, Vec::new())) - share(&base_s), i)).collect();
    scored.sort_by(|a, b| b.0.total_cmp(&a.0));
    for &(_, i) in scored.iter().take(3) {
        let d = riddle_core::forecast::paired(&sims_of(&g, &edits[i].1, sims, Vec::new()), &base, bank).delta;
        if d > best.0 {
            best = (d, edits[i].0.clone());
        }
    }
    (kit, best.0, best.1)
}
