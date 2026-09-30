//! Cut 29 §1, experiment E1 (docs/PROGRESSION.md §7: the dayplayer's strength wall at D13/D17–18,
//! 7–12 days, no probed lever moved it): at a wall held `WALL_DAYS` days of the lineage's clock,
//! the report offers the plateau search's best one-row edit from the vocabulary the lineage owns —
//! a row dropped, moved up or down, a number notched, a stock row written at the top or in place of
//! a row — scored on the share of the camp panel's sends that pass the record; up to two greedy
//! steps. Offered only when it passes the record on ≥ `WALL_BAR` of the sends and beats the set by
//! `WALL_GAIN`. Nothing applies it: the player takes it (`ReturnReport.wall`), or not.
//! Record spikes (Cut 29 core): measured where the sends meet the wall — from the deepest lit
//! waystone at or above the record (the offer then moves the start: `start D24`), past the floor the
//! set reaches on ≥ `WALL_REACH` of its sends when the record was one lucky send's (`wall_floor`);
//! the set's last exit row is never dropped or written over.
use crate::engine::Game;
use crate::forecast::{camp_panel, SimResult};
use crate::rules::{Cond, Row, RuleSet, Verb};

/// Days at the same best depth before the wall's edit is searched (once a day).
pub const WALL_DAYS: u32 = 2;
/// The share of sends past the record the offered edit reaches, at least.
pub const WALL_BAR: f64 = 0.10;
/// Its gain over the set as written (each step), at least.
pub const WALL_GAIN: f64 = 0.05;
/// Sims a candidate is screened on, the best `TOP` measured on `FULL`; greedy steps.
pub const SCREEN: u32 = 12;
pub const FULL: u32 = 48;
pub const TOP: usize = 3;
pub const STEPS: usize = 2;

/// (the share of the sends past floor `at`, the share reaching it) on the camp's panel.
fn shares(g: &Game, set: &RuleSet, n: u32, at: u32) -> (f64, f64) {
    let rs: Vec<SimResult> = camp_panel(g, set, n);
    let k = rs.len().max(1) as f64;
    (rs.iter().filter(|r| r.max_depth > at).count() as f64 / k, rs.iter().filter(|r| r.max_depth >= at).count() as f64 / k)
}

/// The share of the sends that must reach a floor for it to be below the wall (`wall_floor`).
pub const WALL_REACH: f64 = 0.25;

/// The floor the sends meet the wall on: the record, or — when the record was a lucky send's (the
/// set reaches it on fewer than `WALL_REACH` of its sends) — the deepest floor the set does reach
/// that often. Rater AP s1 reached D33 on day 4 with a set that dies to the Lurker Queen's calls at
/// D28 on most sends: past D33 every candidate read 0, and the search offered nothing for six days.
pub fn wall_floor(g: &Game, set: &RuleSet) -> u32 {
    let best = g.lineage.best_depth;
    let rs: Vec<SimResult> = camp_panel(g, set, FULL);
    let k = rs.len().max(1) as f64;
    let from = g.lineage.start.max(1);
    (from..=best).rev().find(|&d| rs.iter().filter(|r| r.max_depth >= d).count() as f64 / k >= WALL_REACH).unwrap_or(best)
}

/// The search's objective: the share past the record, and — a step toward it when none passes (the
/// D28 probe: dropping a row that walked the hero off the Queen took the share reaching her floor
/// 0.5 % → 82 % with none past it yet) — a quarter of the share reaching it.
fn score(s: (f64, f64)) -> f64 {
    s.0 + REACH_WEIGHT * s.1
}

/// The weight of the share reaching the record in the objective (`score`).
pub const REACH_WEIGHT: f64 = 0.25;

/// The set's broad engagement row (`foes ≥ N → attack | shoot nearest`): never dropped.
fn engagement(r: &Row) -> bool {
    matches!(r.verb.v.as_str(), "attack" | "shoot") && r.verb.a.as_deref() == Some("nearest") && r.conds.iter().all(|c| c.k == "foes>=")
}

/// The stock rows a wall's search may write (each only when the lineage's vocabulary holds it).
fn stock(best: u32, at: u32) -> Vec<Row> {
    vec![
        Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")),
        Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
        Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
        Row::new(vec![Cond::t("foe_tag", "gas"), Cond::n("adj>=", 1)], Verb::new("retreat")),
        Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
        Row::new(vec![Cond::n("depth>=", (best + 1) as i32)], Verb::new("bank")),
        Row::new(vec![Cond::n("hp<", 40)], Verb::new("retreat")),
        // the Lurker Queen's counter on the lurkers she calls (the D28 probe: past her 0 → 6.5–10 %)
        Row::new(vec![Cond::t("foe_tag", "summoned"), Cond::n("depth>=", 28)], Verb::arg("read", "silence")),
        Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", at as i32)], Verb::arg("read", "silence")),
        // the Foundry's counter card on the iron golems and their master (the rater lineages' D19–23
        // wall: every one owned the card and knew the fact, no goal set wrote it — reach D23 0.3 % → 90 %)
        Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")),
    ]
}

/// The counter row of the band boss at the wall (on the wall's floor `at` or the next), when its
/// counter fact is held — as the fact reads, and gated to his floor (the rater lineages' D33 wall:
/// the Mirror King's `cadence` owned and known, written by no goal set).
fn counters(g: &Game, at: u32) -> Vec<Row> {
    let mut out = Vec::new();
    for &(kind, depth) in crate::descent::BOSS_DEPTHS.iter().filter(|(_, d)| *d == at || *d == at + 1) {
        if let Some(row) = crate::facts::boss_counter_row(&g.lineage.facts, kind) {
            let mut gated = row.clone();
            gated.conds.push(Cond::n("depth>=", depth as i32));
            out.push(row);
            out.push(gated);
        }
    }
    out
}

/// Every one-row edit of `set` the search weighs, writable by this lineage (its vocabulary, its row
/// cap), labelled (`drop R6`, `R2 above R1`, `R3 hp<40`, `+ hp < 90% → rest at R1`, `R4 → …`).
pub fn edits(g: &Game, set: &RuleSet, at: u32) -> Vec<(String, RuleSet)> {
    let vocab = g.vocabulary();
    let max_rows = g.lineage.max_rows();
    let conds_ok = |r: &Row| r.conds.iter().all(|c| vocab.conds.iter().any(|v| v.same_token(c)) && !vocab.locked.iter().any(|l| l.cond.same_token(c)));
    // a card row: the card owned and not already in the set (its conditions the lineage's words)
    let writable = |r: &Row| match r.card() {
        Some(card) => g.lineage.unlocks.contains(card) && !set.rows.iter().any(|x| x.card() == Some(card)) && conds_ok(r),
        None => conds_ok(r) && vocab.verbs.contains(&r.verb),
    };
    let mut out: Vec<(String, RuleSet)> = Vec::new();
    let mut push = |label: String, s: RuleSet| {
        if s != *set && s.validate().is_ok() && s.own_rows() <= max_rows && !out.iter().any(|(_, o)| *o == s) {
            out.push((label, s));
        }
    };
    let n = set.rows.len();
    // the set's way home: its last exit row is never dropped or written over (rater AO s1: `R2 →
    // reflect_melee → reflect read` took the set's only bank; after the ascension every send died,
    // nine days without a mark or a coin)
    let exit = |r: &Row| matches!(r.verb.v.as_str(), "bank" | "return");
    let exits = set.rows.iter().filter(|r| exit(r)).count();
    let keep = |r: &Row| engagement(r) || (exit(r) && exits == 1);
    for i in 0..n {
        if n > 1 && !keep(&set.rows[i]) {
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
            let steps: &[i32] = if c.k == "depth>=" && exit { &[1, 2] } else if c.k.starts_with("hp") { &[-10, 10] } else { &[-1, 1] };
            for d in steps {
                let nv = v + d;
                if nv < 1 || (c.k.starts_with("hp") && nv > 95) {
                    continue;
                }
                let mut s = set.clone();
                s.rows[i].conds[ci].n = Some(nv);
                push(format!("R{} {}{}", i + 1, c.k, nv), s);
            }
        }
    }
    for r in stock(g.lineage.best_depth, at).into_iter().chain(counters(g, at)).filter(|r| writable(r)) {
        if set.rows.contains(&r) {
            continue;
        }
        let mut s = set.clone();
        s.rows.insert(0, r.clone());
        push(format!("+ {} at R1", r.describe()), s);
        for i in 0..n {
            if keep(&set.rows[i]) {
                continue;
            }
            let mut s = set.clone();
            s.rows[i] = r.clone();
            push(format!("R{} → {}", i + 1, r.describe()), s);
        }
    }
    out
}

/// The wall's edit for the lineage as it stands: (the set, the share past the record before and
/// after, the edits taken), when one passes `WALL_BAR` and beats the set by `WALL_GAIN`.
///
/// Measured where the sends meet the wall: from the deepest lit waystone at or above the record
/// when that is deeper than the sends' start (the offer then moves the start there: `start D24`).
/// A record one lucky D1 send set (rater AP: D33 on day 4 with a set that reaches D33 on ~2 % of
/// its sends) left every candidate at 0 past it on a D1 panel — the search was blind there.
pub fn search(g: &Game) -> Option<crate::wire::WallEdit> {
    let start = g.lineage.rules().clone();
    let best = g.lineage.best_depth;
    let deep = g.lineage.stones().into_iter().filter(|w| *w <= best && *w > g.lineage.start.max(1)).max();
    let moved: Option<Game> = deep.map(|d| {
        let mut s = g.sim_clone();
        s.lineage.start = d;
        s
    });
    let from: &Game = moved.as_ref().unwrap_or(g);
    let at = wall_floor(from, &start);
    let base = shares(g, &start, FULL, at);
    let mut cur = start.clone();
    let mut cur_s = if moved.is_some() { shares(from, &cur, FULL, at) } else { base };
    let mut taken: Vec<String> = deep.map(|d| format!("start D{d}")).into_iter().collect();
    for _ in 0..STEPS {
        let cands = edits(from, &cur, at);
        let mut scr: Vec<(f64, usize)> = cands.iter().enumerate().map(|(i, (_, s))| (score(shares(from, s, SCREEN, at)), i)).collect();
        scr.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut top: Option<((f64, f64), usize)> = None;
        for &(_, i) in scr.iter().take(TOP) {
            let v = shares(from, &cands[i].1, FULL, at);
            if top.is_none_or(|t| score(v) > score(t.0) + 1e-9) {
                top = Some((v, i));
            }
        }
        match top {
            Some((v, i)) if score(v) > score(cur_s) + WALL_GAIN => {
                taken.push(cands[i].0.clone());
                cur = cands[i].1.clone();
                cur_s = v;
            }
            _ => break,
        }
    }
    let offered = !taken.is_empty() && cur_s.0 >= WALL_BAR && (score(cur_s) > score(base) + WALL_GAIN || cur_s.0 > base.0 + WALL_GAIN);
    offered.then_some(crate::wire::WallEdit { depth: at, edits: taken, rules: cur, before: base.0, after: cur_s.0, sims: FULL, start: deep })
}

/// Whether the lineage stands at a wall: its best depth held `WALL_DAYS` days of its clock.
pub fn at_wall(l: &crate::engine::LineageState) -> bool {
    l.day >= l.best_day + WALL_DAYS && l.best_depth > 0
}
