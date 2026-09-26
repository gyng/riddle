//! Cut 28 §1: the oaths' gate. On a cohort set's own lineage (the set played `OATH_HOURS` from a
//! fresh lineage that owns its tokens, the shelf packed — as the lever's), each pool oath is drawn
//! and sworn; a greedy plateau search from the set over one-row edits finds the set that keeps it
//! most often (`oath_best`), another the set that banks most often (`bank_best`); the gate reads how
//! many rows the two differ by (≥ 2: different oaths want different sets) and the chance the oath's
//! best set keeps it in a night of 16 sends (≥ 20 %: a writable set completes it). Shared by
//! `examples/oaths.rs` and `metrics.rs`.
#![allow(dead_code)]
use riddle_core::engine::ExitTier;
use riddle_core::forecast::SimResult;
use riddle_core::{Cond, Game, Row, RuleSet, Verb};

/// Hours the set plays before the oaths are drawn.
pub const OATH_HOURS: u64 = 8;
/// Sims a candidate is screened on, and the sims the best few are measured on.
pub const SCREEN: u32 = 16;
pub const FULL: u32 = 48;
/// Greedy steps of each search, and the candidates measured on `FULL` per step.
pub const STEPS: usize = 3;
pub const TOP: usize = 3;
/// A step is taken when it gains more than this on the objective.
pub const GAIN: f64 = 0.02;
/// The seeds every candidate replays (fixed: base, edits and oaths share the floors).
const TAG: u64 = 0x0A7B_2028;

/// One oath measured on one set's lineage.
#[derive(Clone, Debug, Default)]
pub struct OathRead {
    pub kind: String,
    pub offered: bool,
    pub text: String,
    pub best_depth: u32,
    /// The oath share of a send (0..1): with the set as written, with the oath's best set.
    pub share_set: f64,
    pub share_best: f64,
    /// The bank share of the bank-optimal set and of the oath's best set.
    pub bank_best: f64,
    pub bank_oath: f64,
    /// Rows the oath's best set and the bank-optimal set differ by.
    pub diff: usize,
    pub edits: Vec<String>,
    pub bank_edits: Vec<String>,
    /// The oath's best set.
    pub best: RuleSet,
}

impl OathRead {
    /// The chance a night of 16 sends keeps the oath at least once, with its best set.
    pub fn night(&self) -> f64 {
        riddle_core::oath::night(self.share_best)
    }
}

/// The set's lineage after `OATH_HOURS` of its own sends (as `lever::measure` plays it).
pub fn lineage(set: &RuleSet, seed: u64) -> Game {
    let mut g = crate::lever::cohort_game(set, seed);
    g.lineage.gold = 400;
    crate::lever::fill_shelf(&mut g);
    let _ = riddle_core::offline::run_offline_counts(&mut g, OATH_HOURS * 3600);
    g.run = None;
    g.pending_exit = None;
    g.deaths.clear();
    g.lineage.gold = 0;
    crate::lever::fill_shelf(&mut g);
    g
}

/// `g` with the pool oath `kind` drawn for its lineage and sworn (None: the lineage is offered none).
pub fn sworn(g: &Game, kind: &str) -> Option<Game> {
    let o = riddle_core::oath::draw_kind(&g.lineage, kind, 9_999)?;
    let mut s = g.sim_clone();
    s.lineage.oath_sworn = Some(o.id.clone());
    s.lineage.oaths = vec![o];
    Some(s)
}

/// `g` playing `set`, its shelf packed for it (the repeat re-buys what its rows name).
fn prepared(g: &Game, set: &RuleSet) -> Game {
    let mut p = g.sim_clone();
    let _ = p.set_rules_raw(set.clone());
    for r in &set.rows {
        if let Some(c) = r.card() {
            p.lineage.unlocks.insert(c.into());
        }
    }
    crate::lever::fill_shelf(&mut p);
    p.lineage.last_supplies = p.lineage.supplies.iter().filter(|s| !s.free).map(|s| s.kind.clone()).collect();
    p
}

fn sims(g: &Game, set: &RuleSet, n: u32) -> Vec<SimResult> {
    let p = prepared(g, set);
    riddle_core::forecast::simulate_budget_from(&p, set, n, TAG, 999, u64::MAX, Vec::new())
}

/// (oath share, bank share, mean progress toward the oath) of `set` over `n` sends.
pub fn shares(g: &Game, set: &RuleSet, n: u32) -> (f64, f64, f64) {
    let rs = sims(g, set, n);
    let k = rs.len().max(1) as f64;
    (rs.iter().filter(|r| r.oath).count() as f64 / k, rs.iter().filter(|r| r.tier == ExitTier::Bank && !r.timed_out).count() as f64 / k, rs.iter().map(|r| r.oath_progress).sum::<f64>() / k)
}

/// The rows an oath's search may write beyond the stock ones.
fn oath_rows(kind: &str, goal: u32) -> Vec<Row> {
    let boss = Cond::t("foe_tag", "boss");
    match kind {
        "fire" | "slayer" => vec![Row::new(vec![boss.clone()], Verb::arg("throw", "fire,tag:boss")), Row::new(vec![boss], Verb::arg("attack", "tag:boss"))],
        "tamer" => vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tame", "nearest")), Row::new(vec![Cond::t("foe_tag", "fast")], Verb::arg("tame", "tag:fast"))],
        "lean" | "bold" => vec![Row::new(vec![Cond::n("depth>=", goal as i32)], Verb::new("bank")), Row::new(vec![Cond::n("hp<", 40)], Verb::new("retreat"))],
        _ => Vec::new(),
    }
}

/// The stock rows every search may add (`lever::one_row_edits`' own, and a deeper bank).
fn stock(goal: u32) -> Vec<Row> {
    vec![
        Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")),
        Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
        Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
        Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
        Row::new(vec![Cond::n("depth>=", goal as i32)], Verb::new("bank")),
    ]
}

/// The one-row edits of `set` a search weighs: a row dropped, a number notched (hp ±10, depth
/// ±1/+2), a stock or oath row added at the top (under the cap) or in place of a row.
pub fn edits(g: &Game, set: &RuleSet, kind: Option<&str>) -> Vec<(String, RuleSet)> {
    let goal = (g.lineage.best_depth + 1).max(3);
    let mut out: Vec<(String, RuleSet)> = Vec::new();
    let mut push = |label: String, s: RuleSet| {
        if s != *set && s.validate().is_ok() && s.own_rows() <= g.lineage.max_rows() && !out.iter().any(|(_, o)| *o == s) {
            out.push((label, s));
        }
    };
    let n = set.rows.len();
    let engagement = |r: &Row| crate::lever::engagement(r);
    for i in 0..n {
        if n > 1 && !engagement(&set.rows[i]) {
            let mut s = set.clone();
            s.rows.remove(i);
            push(format!("drop R{} {}", i + 1, set.rows[i].describe()), s);
        }
        for (ci, c) in set.rows[i].conds.iter().enumerate() {
            let Some(v) = c.n else { continue };
            // (never an exit row's depth lowered — as the lever's: a shallower bank buys its share with the depth, not with play)
            let exit = matches!(set.rows[i].verb.v.as_str(), "bank" | "return");
            let steps: &[i32] = if c.k == "depth>=" && exit { &[1, 2] } else if c.k == "depth>=" { &[-1, 1] } else if c.k.starts_with("hp") { &[-10, 10] } else { &[] };
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
    // the oath's own cut: every row its constraint forbids, in one edit (a player who swore `no drink`
    // takes his drink rows out)
    let forbids = |r: &Row| match kind {
        Some("lean") => r.verb.v == "rest",
        Some("bold") => r.verb.v == "return",
        _ => false,
    };
    if set.rows.iter().filter(|r| forbids(r)).count() > 1 {
        let mut s = set.clone();
        s.rows.retain(|r| !forbids(r));
        push("drop every forbidden row".into(), s);
    }
    let mut adds = stock(goal);
    if let Some(k) = kind {
        adds.extend(oath_rows(k, goal));
    }
    // (never a row the oath forbids)
    adds.retain(|r| !forbids(r));
    for r in &adds {
        if set.rows.contains(r) {
            continue;
        }
        let mut s = set.clone();
        s.rows.insert(0, r.clone());
        push(format!("+ {} at R1", r.describe()), s);
        let at = riddle_core::meta::safety_end(set).min(set.rows.len());
        if at > 0 {
            let mut s = set.clone();
            s.rows.insert(at, r.clone());
            push(format!("+ {} at R{}", r.describe(), at + 1), s);
        }
        for i in 0..n {
            if engagement(&set.rows[i]) {
                continue;
            }
            let mut s = set.clone();
            s.rows[i] = r.clone();
            push(format!("R{} → {}", i + 1, r.describe()), s);
        }
    }
    out
}

/// A greedy plateau search from `start`: each step screens every one-row edit on `SCREEN` sends,
/// measures the best `TOP` on `FULL`, and takes the best when it gains more than `GAIN` on the
/// objective (`score` of (oath share, bank share)); up to `steps` steps. (the set found, its
/// (oath, bank) shares on `FULL`, the edits taken).
pub fn climb(g: &Game, start: &RuleSet, kind: Option<&str>, steps: usize, score: impl Fn((f64, f64, f64)) -> f64) -> (RuleSet, (f64, f64, f64), Vec<String>) {
    let mut cur = start.clone();
    let mut cur_s = shares(g, &cur, FULL);
    let mut taken = Vec::new();
    for _ in 0..steps {
        let cands = edits(g, &cur, kind);
        let mut screened: Vec<(f64, usize)> = cands.iter().enumerate().map(|(i, (_, s))| (score(shares(g, s, SCREEN)), i)).collect();
        screened.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut best: Option<(f64, (f64, f64, f64), usize)> = None;
        for &(_, i) in screened.iter().take(TOP) {
            let sh = shares(g, &cands[i].1, FULL);
            if best.is_none_or(|b| score(sh) > b.0 + 1e-9) {
                best = Some((score(sh), sh, i));
            }
        }
        match best {
            Some((s, sh, i)) if s > score(cur_s) + GAIN => {
                taken.push(cands[i].0.clone());
                cur = cands[i].1.clone();
                cur_s = sh;
            }
            _ => break,
        }
    }
    (cur, cur_s, taken)
}

/// Rows two sets differ by: the larger of the rows each holds that the other does not (by
/// conditions and verb; a moved row is the same row, a notched one a different one).
pub fn row_diff(a: &RuleSet, b: &RuleSet) -> usize {
    let key = |r: &Row| (r.conds.clone(), r.verb.clone());
    let ka: Vec<_> = a.rows.iter().map(key).collect();
    let kb: Vec<_> = b.rows.iter().map(key).collect();
    let only = |x: &Vec<(Vec<Cond>, Verb)>, y: &Vec<(Vec<Cond>, Verb)>| {
        let mut y = y.clone();
        x.iter()
            .filter(|k| match y.iter().position(|o| o == *k) {
                Some(i) => {
                    y.remove(i);
                    false
                }
                None => true,
            })
            .count()
    };
    only(&ka, &kb).max(only(&kb, &ka))
}

/// The bank-optimal set of `set`'s lineage (a plateau search on the bank share).
pub fn bank_best(g: &Game, set: &RuleSet) -> (RuleSet, f64, Vec<String>) {
    let (s, sh, taken) = climb(g, set, None, 2, |(_, bank, _)| bank);
    (s, sh.1, taken)
}

/// One oath on one set's lineage `g`, against its bank-optimal set (`bank`; `diff` is 0 without).
pub fn measure(g: &Game, set: &RuleSet, kind: &str, bank: Option<&(RuleSet, f64, Vec<String>)>) -> OathRead {
    let mut read = OathRead { kind: kind.into(), best_depth: g.lineage.best_depth, bank_best: bank.map_or(0.0, |b| b.1), bank_edits: bank.map(|b| b.2.clone()).unwrap_or_default(), ..Default::default() };
    let Some(sg) = sworn(g, kind) else { return read };
    read.offered = true;
    read.text = riddle_core::oath::text(riddle_core::oath::sworn(&sg.lineage).unwrap());
    read.share_set = shares(&sg, set, FULL).0;
    // (past a plateau of zero the search reads how close the sends came: `oath::progress`)
    let score = |(oath, bank, progress): (f64, f64, f64)| oath + 0.3 * progress + 0.01 * bank;
    let (mut best, mut sh, mut taken) = climb(&sg, set, Some(kind), STEPS, score);
    // a second start: the set with every row the oath forbids taken out (a player who swore `no
    // drink` starts there) — the better of the two plateaus
    let forbidden = |r: &Row| match kind {
        "lean" => r.verb.v == "rest",
        "bold" => r.verb.v == "return",
        _ => false,
    };
    if set.rows.iter().any(forbidden) {
        let conformed = RuleSet { rows: set.rows.iter().filter(|r| !forbidden(r)).cloned().collect(), ..set.clone() };
        let (b2, s2, mut t2) = climb(&sg, &conformed, Some(kind), STEPS, score);
        if score(s2) > score(sh) + 1e-9 {
            t2.insert(0, "drop every forbidden row".into());
            (best, sh, taken) = (b2, s2, t2);
        }
    }
    read.share_best = sh.0;
    read.bank_oath = sh.1;
    read.diff = bank.map_or(0, |b| row_diff(&best, &b.0));
    read.edits = taken;
    read.best = best;
    read
}
