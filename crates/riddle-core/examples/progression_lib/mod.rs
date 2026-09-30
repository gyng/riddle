//! The progression timeline's lineages (docs/PROGRESSION.md), shared by `examples/progression.rs`
//! (the measured timeline, with each purchase's leave-one-out move) and `examples/metrics.rs` (Cut 29
//! §1: the progression rows — unlock days, marks unspent, stalls, the purse — on the replays alone).
#![allow(dead_code)]
use riddle_core::forecast::{camp_panel, paired, SimResult, FORECAST_SIMS};
use riddle_core::rules::{Cond, Row, Verb};
use riddle_core::{Game, RuleSet};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone)]
pub enum Mode {
    /// The dayplayer's policy; `sinks`: plus forge steps and oaths.
    Day { sinks: bool },
    /// A cohort set as the goal.
    Rater(RuleSet),
}

// ---------------------------------------------------------------------------------------------
// The dayplayer's hands (examples/dayplayer.rs, copied: an example cannot import another).

pub fn boss_at(depth: u32) -> Option<&'static str> {
    riddle_core::descent::boss_for(depth)
}

pub fn counter_rows(boss: &str) -> Vec<Row> {
    match boss {
        "goblin_warlord" => vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))],
        "bloat_mother" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 9)], Verb::arg("throw", "fire,tag:boss"))],
        "lich" => vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:summoned")), Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))],
        "foundry_master" => vec![Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")), Row::new(vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 19)], Verb::arg("attack", "tag:buffer"))],
        "lurker_queen" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 28)], Verb::arg("read", "silence"))],
        "mirror_king" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 33)], Verb::arg("tactic", "cadence"))],
        _ => vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))],
    }
}

pub fn row_unlock(row: &Row) -> Option<String> {
    match (row.verb.v.as_str(), row.verb.a.as_deref()) {
        ("tactic", Some(card)) => Some(card.to_string()),
        ("throw", _) => Some("throw".into()),
        _ => None,
    }
}

pub fn bank_row() -> Row {
    Row::new(vec![Cond::n("hp<", 40), Cond::n("depth>=", 3)], Verb::new("bank")).from("player")
}

pub fn write_own_rows(g: &mut Game, bank: bool) -> u32 {
    let mut n = 0;
    let has_exit = g.lineage.rules().rows.iter().any(|r| matches!(r.verb.v.as_str(), "bank" | "return"));
    if bank && !has_exit && insert_row(g, bank_row(), 0) {
        n += 1;
    }
    let situations = std::iter::once("stray").chain(riddle_core::descent::SITUATION_DEPTHS.iter().map(|(w, _)| *w));
    for what in situations {
        if !g.lineage.facts.contains(what) {
            continue;
        }
        let row = riddle_core::probes::situation_answer(what).from("player");
        let vocab = g.vocabulary();
        let ok = row.conds.iter().all(|c| vocab.conds.iter().any(|v| v.k == c.k && v.t == c.t)) && vocab.verbs.iter().any(|v| v.v == row.verb.v && v.a == row.verb.a);
        if !ok || g.lineage.rules().rows.contains(&row) {
            continue;
        }
        if g.lineage.rules().own_rows() < g.vocabulary().max_rows && insert_row(g, row, 0) {
            n += 1;
        }
    }
    n
}

pub fn insert_row(g: &mut Game, row: Row, at: i32) -> bool {
    if at < 0 {
        return false;
    }
    let at = at as usize;
    let max_rows = g.vocabulary().max_rows;
    let mut rules = g.lineage.rules().clone();
    if rules.rows.contains(&row) {
        return false;
    }
    if !row.is_card() && rules.own_rows() >= max_rows {
        let plain = |r: &Row| r.verb.v == "attack" && !r.verb.a.as_deref().is_some_and(|a| a.starts_with("tag:"));
        let drop = (0..rules.rows.len()).rev().find(|&i| !rules.rows[i].is_card() && !plain(&rules.rows[i])).unwrap_or(rules.rows.len() - 1);
        rules.rows.remove(drop);
    }
    let at = at.min(rules.rows.len());
    rules.rows.insert(at, row);
    g.set_rules(rules).is_ok()
}

// ---------------------------------------------------------------------------------------------
// The rater's hands: the set as far as the lineage can write it.

pub fn verb_ok(g: &Game, v: &Verb) -> bool {
    let vocab = g.vocabulary();
    let head = |a: Option<&str>| a.map(|a| a.split(',').next().unwrap_or(a).to_string());
    vocab.verbs.iter().any(|x| x.v == v.v && (x.a == v.a || head(x.a.as_deref()) == head(v.a.as_deref())))
}

pub fn row_ok(g: &Game, r: &Row) -> bool {
    if let Some(c) = r.card() {
        return g.lineage.unlocks.contains(c);
    }
    let vocab = g.vocabulary();
    let conds = r.conds.iter().all(|c| vocab.conds.iter().any(|v| v.same_token(c)) && !vocab.locked.iter().any(|l| l.cond.same_token(c)));
    conds && verb_ok(g, &r.verb)
}

/// The rater set as the lineage can write it now: its writable rows in order, the own rows cut
/// to the cap keeping the set's plain engagement row (the spine), every owned card row.
pub fn project(g: &Game, set: &RuleSet) -> RuleSet {
    let cap = g.lineage.max_rows();
    let ok: Vec<&Row> = set.rows.iter().filter(|r| row_ok(g, r)).collect();
    let plain = |r: &Row| r.verb.v == "attack" && r.verb.a.as_deref() == Some("nearest") && !r.is_card();
    let spine = ok.iter().rposition(|r| plain(r));
    let mut rows: Vec<Row> = Vec::new();
    let mut own = 0;
    for (i, r) in ok.iter().enumerate() {
        if r.is_card() {
            rows.push((*r).clone());
            continue;
        }
        let room = if spine.is_some_and(|s| s > i) { cap.saturating_sub(1) } else { cap };
        if own < room || Some(i) == spine {
            rows.push((*r).clone());
            own += 1;
        }
    }
    let route: Vec<u32> = set.route.iter().copied().filter(|f| g.lineage.facts.contains(&format!("fork:{f}"))).collect();
    RuleSet { rows, name: set.name.clone(), route }
}

/// What the set needs bought, in the set's order: row slots, condition words, cards, `throw`.
pub fn needed(g: &Game, set: &RuleSet) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let own = set.own_rows();
    if own > g.lineage.max_rows() {
        if let Some(u) = riddle_core::meta::UNLOCKS.iter().find(|u| riddle_core::meta::is_row_unlock(u.id) && !g.lineage.unlocks.contains(u.id)) {
            out.push(u.id.into());
        }
    }
    for r in &set.rows {
        for c in &r.conds {
            if let Some(u) = riddle_core::meta::cond_unlock(&c.k) {
                out.push(u.into());
            }
        }
        if let Some(u) = row_unlock(r) {
            out.push(u);
        }
    }
    let mut seen = BTreeSet::new();
    out.retain(|u| !g.lineage.unlocks.contains(u) && seen.insert(u.clone()));
    out
}

// ---------------------------------------------------------------------------------------------
// Instruments.

#[derive(Default, Clone)]
pub struct Buy {
    pub id: String,
    pub cost: u32,
    pub via: String,
    pub gold: u32,
}

#[derive(Default, Clone)]
pub struct DayRec {
    pub checkins: u32,
    pub runs: u32,
    pub banked: u32,
    pub returned: u32,
    pub deaths: u32,
    pub marks_src: BTreeMap<String, i64>,
    pub marks_spent: u32,
    pub buys: Vec<Buy>,
    pub marks_end: u32,
    pub marks_max: u32,
    pub gold_in: BTreeMap<String, i64>,
    pub gold_out: BTreeMap<String, i64>,
    pub gold_end: i32,
    pub gold_max: i32,
    pub best: u32,
    pub new_best: bool,
    pub counter_known: bool,
    pub reveal: Vec<String>,
    pub level: u32,
    pub rank: u32,
    pub rows: usize,
    pub own_rows: usize,
    pub max_rows: usize,
    pub kit: String,
    pub oaths: Vec<String>,
    pub oath_unlocks: Vec<String>,
    pub ascended: bool,
    pub measured: Vec<Value>,
    pub left: Vec<Value>,
    pub marks_lost: u32,
    pub frontier_ci: Vec<i64>,
    pub wall: Vec<Value>,
    /// Cut 29: oaths kept, systems opened (the curriculum) and open at the day's end, the wall's edit
    /// taken (E1), the worst purse against the day's net (`gold / max(1.5 × last_day_net, 10 units)`,
    /// check-ins after day 3), oath draws and commissions bought.
    pub oaths_kept: u32,
    pub systems_opened: Vec<String>,
    pub systems_open: usize,
    pub wall_taken: Vec<String>,
    pub purse_ratio: f64,
    pub draws: u32,
    pub works: u32,
}

pub fn add(m: &mut BTreeMap<String, i64>, k: &str, v: i64) {
    if v != 0 {
        *m.entry(k.into()).or_insert(0) += v;
    }
}

/// The reveal ladder (web/src/ui/reveal.ts `earned`), read off the lineage.
pub fn reveal(g: &Game) -> Vec<&'static str> {
    let l = &g.lineage;
    let mut out = Vec::new();
    if !l.graveyard.is_empty() || l.heir > 1 || !l.chronicle.is_empty() {
        out.push("edit");
    }
    if l.gold > 0 || l.gold_ledger.iter().any(|x| x.delta > 0) || l.supplies.iter().any(|s| !s.free) {
        out.push("loadout");
    }
    if l.marks > 0 || l.unlocks.iter().any(|u| u != "tame") || l.rank > 0 {
        out.push("unlocks");
    }
    if !l.vault.is_empty() || !g.loadout.is_empty() || l.unlocks.iter().any(|u| u.starts_with("vault")) {
        out.push("vault");
    }
    if !l.forge.is_empty() {
        out.push("forge");
    }
    let lads = riddle_core::kit::ladders(l);
    if lads.iter().any(|k| k.owned > 0 || k.next.as_ref().is_some_and(|n| n.affordable)) {
        out.push("kit");
    }
    if l.party.len() + l.kennel.len() + l.eggs.len() > 0 {
        out.push("party");
    }
    if l.sets.iter().any(|s| s.own_rows() >= 3) || l.unlocks.iter().any(|u| riddle_core::meta::is_row_unlock(u)) {
        out.push("gems");
    }
    if l.heir >= 5 {
        out.push("heirs");
    }
    if l.rank > 0 || l.renown > 0 {
        out.push("rank");
    }
    if l.facts.contains("vault") {
        out.push("cage");
    }
    if !l.waystones.is_empty() {
        out.push("start");
    }
    if l.oaths.iter().any(|o| l.gold >= o.price) || l.oath_sworn.is_some() || !l.titles.is_empty() {
        out.push("oaths");
    }
    let classes = ["rogue", "ranger", "caster"].iter().filter(|c| l.unlocks.contains(**c)).count();
    if classes > 0 {
        out.push("class");
    }
    out
}

/// Marks by source over an absence, from the lineage before and after (nothing is spent during
/// one): new depth, first bank per depth, boss kinds, trophies by family, ranks; the rest is the
/// frontier bank's mark (Cut 15 §1).
pub fn marks_by_source(a: &Game, b: &Game, night: u32, rec: &mut DayRec) {
    let (la, lb) = (&a.lineage, &b.lineage);
    let total = lb.marks as i64 - la.marks as i64;
    let mut known = 0i64;
    let mut parts: Vec<(&str, i64)> = vec![
        ("depth", lb.best_depth.saturating_sub(la.best_depth) as i64),
        ("home", lb.banked_depths.len().saturating_sub(la.banked_depths.len()) as i64),
        ("boss", 3 * lb.kills.iter().filter(|k| !la.kills.contains(*k) && riddle_core::defs::monster_def(k).boss).count() as i64),
    ];
    for t in lb.trophies.iter().filter(|t| !la.trophies.contains(*t)) {
        parts.push(if t.starts_with("oath:") {
            ("oath", 0)
        } else if t.starts_with("studied_all_") {
            ("trophy:studied_all", 3)
        } else if t.starts_with("master:") {
            ("trophy:mastery", 2)
        } else if t.starts_with("studied:") || t.starts_with("home:") || t.starts_with("slain:") || t.starts_with("bones:") || t.starts_with("ledger:") {
            ("trophy:lifetime", 2)
        } else {
            ("trophy:run", 2)
        });
    }
    parts.push(("rank", lb.rank.saturating_sub(la.rank) as i64));
    // Cut 29 §1: the night's mark (the frontier mark is gone)
    parts.push(("night", night as i64));
    for (k, v) in parts {
        add(&mut rec.marks_src, k, v);
        known += v;
    }
    // (anything left unexplained — must stay 0: the frontier's source is gone)
    add(&mut rec.marks_src, "frontier", total - known);
}

/// Gold by source and sink: the ledger's lines written after `t0` (a residual when the ledger's
/// cap dropped lines is `unledgered`).
pub fn gold_by_line(g: &Game, t0: u64, gold0: i32, rec: &mut DayRec) {
    let mut sum = 0i64;
    for x in g.lineage.gold_ledger.iter().filter(|x| x.t > t0) {
        let w = x.why.as_str();
        let d = x.delta as i64;
        sum += d;
        let k = if w.starts_with("salvage") {
            "salvage"
        } else if w.starts_with("wake pay") {
            "wake_pay"
        } else if w.starts_with("passage") {
            "passage"
        } else if w.starts_with("forge ") {
            "forge"
        } else if w.starts_with("unlock ") {
            "unlock"
        } else if w.starts_with("oath ") || w.starts_with("forswear ") {
            "oath"
        } else if w.starts_with("insure") {
            "insure"
        } else if w.starts_with("hatch") || w.starts_with("egg") {
            "hatch"
        } else if riddle_core::engine::is_exit_why(w) {
            "exits"
        } else if d < 0 {
            "supplies"
        } else {
            "other"
        };
        if d >= 0 {
            add(&mut rec.gold_in, k, d);
        } else {
            add(&mut rec.gold_out, k, -d);
        }
    }
    let resid = (g.lineage.gold - gold0) as i64 - sum;
    if resid > 0 {
        add(&mut rec.gold_in, "unledgered", resid);
    } else {
        add(&mut rec.gold_out, "unledgered", -resid);
    }
}

// ---- the measure: a purchase's move on the camp's panel, paired seeds ----

pub struct Move {
    pub bank: f64,
    pub bank_pm: f64,
    pub depth: f64,
    pub depth_pm: f64,
    pub reach: f64,
    pub reach_pm: f64,
    pub at: u32,
    pub gold: f64,
    pub beyond: f64,
    pub beyond_pm: f64,
    pub beyond_base: f64,
}

pub fn panel_move(with: &Game, without: &Game) -> Move {
    let a = camp_panel(with, with.lineage.rules(), FORECAST_SIMS);
    let b = camp_panel(without, without.lineage.rules(), FORECAST_SIMS);
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let ind = |x: bool| if x { 1.0 } else { 0.0 };
    let best = without.lineage.best_depth;
    let reach_at = |d: u32| b.iter().filter(|r| r.max_depth >= d).count() as f64 / n.max(1) as f64;
    let at = (2..=best + 1).min_by(|&x, &y| (reach_at(x) - 0.5).abs().total_cmp(&(reach_at(y) - 0.5).abs()).then(y.cmp(&x))).unwrap_or(best + 1);
    let bank = paired(a, b, |r: &SimResult| ind(r.tier == riddle_core::engine::ExitTier::Bank && !r.timed_out));
    let depth = paired(a, b, |r: &SimResult| r.max_depth as f64);
    let reach = paired(a, b, |r: &SimResult| ind(r.max_depth >= at));
    let gold = paired(a, b, |r: &SimResult| r.loot_kept as f64);
    let bey = paired(a, b, |r: &SimResult| ind(r.max_depth > best));
    Move { beyond: bey.delta, beyond_pm: bey.pm, beyond_base: bey.base, bank: bank.delta, bank_pm: bank.pm, depth: depth.delta, depth_pm: depth.pm, reach: reach.delta, reach_pm: reach.pm, at, gold: gold.delta }
}

/// A move that clears its own ± by a readable amount on one of bank share, reach at the
/// half-reach floor (≥ 3 points) or mean depth (≥ 0.25 floors).
pub fn meaningful(m: &Move) -> bool {
    (m.bank > m.bank_pm && m.bank >= 0.03) || (m.reach > m.reach_pm && m.reach >= 0.03) || (m.depth > m.depth_pm && m.depth >= 0.25)
}

pub fn move_json(id: &str, m: &Move) -> Value {
    json!({"id": id, "bank": r3(m.bank), "bank_pm": r3(m.bank_pm), "reach": r3(m.reach), "reach_pm": r3(m.reach_pm), "at": m.at,
           "depth": r3(m.depth), "depth_pm": r3(m.depth_pm), "gold": m.gold.round(), "beyond": r3(m.beyond), "beyond_pm": r3(m.beyond_pm), "beyond_base": r3(m.beyond_base), "meaningful": meaningful(m)})
}

pub fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The lineage without unlock `id` (leave-one-out): the card's row gone, the rows fitted to the
/// cap, the vault loadout and the party trimmed to their slots.
pub fn without(g: &Game, id: &str) -> Game {
    let mut s = g.sim_clone();
    s.lineage.unlocks.remove(id);
    let mut rules = s.lineage.rules().clone();
    rules.rows.retain(|r| r.card() != Some(id));
    let _ = s.set_rules_raw(rules);
    let slots = s.lineage.vault_slots();
    s.loadout.truncate(slots);
    let ps = s.lineage.party_slots() as usize;
    while s.lineage.party.len() > ps {
        let c = s.lineage.party.pop().unwrap();
        s.lineage.kennel.push(c);
    }
    s
}

/// The policy's own answer to owning `id` (for measuring an unbought one): the card's row, the
/// set re-projected (a rater) or the rows written (the dayplayer), the vault and kennel fielded.
pub fn with(g: &Game, id: &str, mode: &Mode) -> Game {
    let mut s = g.sim_clone();
    s.lineage.unlocks.insert(id.into());
    respond(&mut s, id, mode);
    s
}

pub fn respond(g: &mut Game, id: &str, mode: &Mode) {
    match mode {
        Mode::Rater(set) => {
            let p = project(g, set);
            let _ = g.set_rules_raw(p);
        }
        Mode::Day { .. } => {
            if let Some((row, at)) = riddle_core::meta::delta_row(&g.lineage, id) {
                if row.is_card() {
                    let mut rules = g.lineage.rules().clone();
                    if !rules.rows.contains(&row) {
                        rules.rows.insert(at.min(rules.rows.len()), row);
                        let _ = g.set_rules_raw(rules);
                    }
                }
            }
            write_own_rows(g, false);
        }
    }
    field(g);
}

pub fn field(g: &mut Game) {
    let ids: Vec<u32> = g.lineage.vault.iter().map(|i| i.id).collect();
    g.loadout(ids);
    let slots = g.lineage.party_slots() as usize;
    if g.lineage.party.len() < slots && !g.lineage.kennel.is_empty() {
        let mut ids: Vec<u32> = g.lineage.party.iter().map(|c| c.id).collect();
        let mut k: Vec<_> = g.lineage.kennel.iter().filter(|c| !ids.contains(&c.id)).collect();
        k.sort_by_key(|c| std::cmp::Reverse(c.level));
        for c in k.iter().take(slots - ids.len()) {
            ids.push(c.id);
        }
        let _ = g.set_party(ids);
    }
}

// ---------------------------------------------------------------------------------------------
// The lineage's fortnight.

pub fn buy_marks(g: &mut Game, id: &str, rec: &mut DayRec) -> bool {
    let cost = riddle_core::meta::unlock_cost(id);
    if g.buy(id).is_ok() {
        rec.marks_spent += cost;
        rec.buys.push(Buy { id: id.into(), cost, via: "marks".into(), gold: 0 });
        true
    } else {
        false
    }
}

pub fn buy_gold(g: &mut Game, id: &str, rec: &mut DayRec) -> bool {
    let price = g.unlocks().iter().find(|u| u.id == id).map(|u| u.gold).unwrap_or(0);
    if g.buy_unlock_gold(id).is_ok() {
        rec.buys.push(Buy { id: id.into(), cost: riddle_core::meta::unlock_cost(id), via: "gold".into(), gold: price });
        true
    } else {
        false
    }
}

/// The oath a player swears for the day: each unsworn oath the purse pays (with $150 to spare) read
/// on the camp's forecast as the sworn one (`Forecast.oath`: the share of the sends that keep it) —
/// the one the set keeps most, when a night of sends keeps it at least half the time (`oath::night`).
/// Cut 29 core: the replay swore the board's first oath whatever the set could keep (rater AS s2:
/// `Warlord · fire` sworn nine days running by a set that throws no fire; `tame · a new kind` 53
/// times on empty days by sets with no tame row) — PROGRESSION.md §7 projected the oaths a player
/// keeps, and a player reads the forecast's oath line before swearing.
pub fn pick_oath(g: &Game) -> Option<String> {
    let sworn = riddle_core::oath::sworn_ids(&g.lineage);
    let mut best: Option<(f64, String)> = None;
    for o in g.lineage.oaths.iter().filter(|o| !sworn.contains(&o.id) && g.lineage.gold >= o.price + 150) {
        let mut h = g.sim_clone();
        h.lineage.oath_sworn = Some(o.id.clone());
        let panel = camp_panel(&h, h.lineage.rules(), FORECAST_SIMS);
        let p = riddle_core::oath::share(&h.lineage, &panel).map_or(0.0, |s| s.share);
        if riddle_core::oath::night(p) >= 0.5 && best.as_ref().is_none_or(|b| p > b.0) {
            best = Some((p, o.id.clone()));
        }
    }
    best.map(|b| b.1)
}

/// Gold sinks a player of this build uses: every forge step the purse pays with $150 to spare
/// for the shelf, the oaths the set keeps (`pick_oath`, read once a day: `read`) in every free
/// slot, and commissions with the rest.
pub fn spend_gold(g: &mut Game, rec: &mut DayRec) {
    spend_gold_reading(g, rec, true)
}
pub fn spend_gold_reading(g: &mut Game, rec: &mut DayRec, read: bool) {
    loop {
        let lads = riddle_core::kit::ladders(&g.lineage);
        let Some((slot, price)) = lads.iter().filter_map(|l| l.next.as_ref().map(|n| (l.slot.clone(), n.price))).min_by_key(|x| x.1) else { break };
        if g.lineage.gold < price as i32 + 150 || riddle_core::kit::buy(g, &slot).is_err() {
            break;
        }
        rec.buys.push(Buy { id: format!("forge:{slot}"), cost: 0, via: "forge".into(), gold: price });
    }
    // Cut 29 §1: every free oath slot, the first standing oath the purse pays with $150 to spare
    if riddle_core::oath::open(&g.lineage) {
        riddle_core::oath::refresh(&mut g.lineage);
        loop {
            let sworn = riddle_core::oath::sworn_ids(&g.lineage);
            if sworn.len() >= riddle_core::oath::slots(&g.lineage) {
                break;
            }
            if !read {
                break;
            }
            let Some(o) = pick_oath(g).and_then(|id| g.lineage.oaths.iter().find(|o| o.id == id).cloned()) else { break };
            if g.swear_oath(&o.id).is_err() {
                break;
            }
            rec.oaths.push(format!("swore {} → {} (${})", riddle_core::oath::text(&o), o.reward.label, o.price));
        }
    }
    // Cut 29 §5: the rest of the purse past a night's re-pack commissions the lineage's works
    let reserve = 150 + riddle_core::oath::price(&g.lineage);
    while g.lineage.gold >= riddle_core::kit::commission_price(&g.lineage) + reserve {
        if g.commission().is_err() {
            break;
        }
        rec.works += 1;
    }
}

/// Cut 29 §1: marks past the reserve of 8 with nothing on the shelf draw fresh oaths (◆2).
pub fn draw_down(g: &mut Game, rec: &mut DayRec) {
    while g.lineage.marks > 8 && g.draw_oath().is_ok() {
        rec.draws += 1;
    }
}

#[derive(Clone)]
pub struct Out {
    pub name: String,
    pub mode: String,
    pub seed: u64,
    pub days: Vec<DayRec>,
    pub rules: String,
}

/// `measure`: the leave-one-out moves and the wall probe (the timeline); off, the lineage alone (the
/// metrics' progression rows).
pub fn play(name: String, mode: Mode, seed: u64, days: usize, schedule: &[u64], verbose: bool, measure: bool) -> Out {
    let walls_only = std::env::var("PROG_WALLS_ONLY").is_ok() || !measure;
    let probe_walls = measure;
    let mut goal: Option<RuleSet> = match &mode {
        Mode::Rater(set) => Some(set.clone()),
        _ => None,
    };
    let mut g = Game::new_literal(seed);
    let mut table = Vec::new();
    // dayplayer state
    let mut last_best = 0u32;
    let mut stalled_days = 0usize;
    let mut counters_done: Vec<String> = Vec::new();
    for day in 0..days {
        // `PROG_SNAP=dir`: the lineage's save at each day's start (`dir/<name>-s<seed>-d<day>.json`, day 1-based) — the probes' input
        if let Ok(dir) = std::env::var("PROG_SNAP") {
            let _ = std::fs::write(format!("{dir}/{}-s{seed}-d{}.json", name.replace(['/', ' '], "_"), day + 1), g.save());
        }
        let mut d = DayRec::default();
        let best0 = g.lineage.best_depth;
        for (ci, &elapsed) in schedule.iter().enumerate() {
            let before = g.sim_clone();
            let t0 = g.lineage.total_turns;
            let gold0 = g.lineage.gold;
            let unl0 = g.lineage.unlocks.clone();
            let rep = riddle_core::offline::run_offline_quick(&mut g, elapsed);
            d.checkins += 1;
            d.runs += rep.runs;
            d.banked += rep.banked;
            d.returned += rep.returned;
            d.deaths += rep.deaths.iter().map(|x| x.n).sum::<u32>();
            let f0 = d.marks_src.get("frontier").copied().unwrap_or(0);
            marks_by_source(&before, &g, rep.night_marks, &mut d);
            d.oaths_kept += rep.oath.as_ref().is_some_and(|o| o.done) as u32 + rep.oaths_kept.len() as u32;
            d.systems_opened.extend(rep.systems_opened.iter().cloned());
            d.frontier_ci.push(d.marks_src.get("frontier").copied().unwrap_or(0) - f0);
            for u in g.lineage.unlocks.iter().filter(|u| !unl0.contains(*u)) {
                d.oath_unlocks.push(u.clone());
            }
            if let Some(o) = &g.batch.oath {
                d.oaths.push(format!("{} · {}", riddle_core::oath::text(&o.0), if o.3 { "kept" } else { "unkept" }));
            }
            // camp decisions
            let t1 = g.lineage.total_turns;
            let gold1 = g.lineage.gold;
            gold_by_line(&g, t0, gold0, &mut d);
            match &mode {
                Mode::Day { sinks } => {
                    write_own_rows(&mut g, day == 0);
                    if g.lineage.ended && g.ascend("no_rest").is_ok() {
                        d.marks_lost += 0;
                        d.ascended = true;
                        last_best = 0;
                        stalled_days = 0;
                        counters_done.clear();
                    }
                    if let Some(id) = rep.worst_death_id {
                        if let Some(death) = g.death(id) {
                            if death.verdict == "gap" {
                                if let Some(p) = death.patches.first() {
                                    if p.survive > death.baseline + 0.15 && p.forecast_delta >= 0.0 {
                                        insert_row(&mut g, p.row.clone(), p.insert_at);
                                    }
                                }
                            }
                        }
                    }
                    if let Some(stall) = &rep.stall {
                        if let Some(p) = stall.patches.first() {
                            let max_rows = g.vocabulary().max_rows;
                            let applied = if p.replace || p.remove {
                                let rules = riddle_core::offline::apply_patch(g.lineage.rules(), p, max_rows);
                                rules != *g.lineage.rules() && g.set_rules(rules).is_ok()
                            } else {
                                insert_row(&mut g, p.row.clone(), p.insert_at)
                            };
                            if applied {
                                stalled_days = 0;
                            }
                        }
                    }
                    // Cut 29 §1 (E1): the wall's edit, taken as the client offers it (`Game::wall_edit`, lazy; fresh once a day)
                    let fresh = g.lineage.wall_day != Some(g.lineage.day);
                    if let Some(w) = &g.wall_edit().filter(|_| fresh) {
                        if g.set_rules(w.rules.clone()).is_ok() {
                            if let Some(s) = w.start {
                                let _ = g.set_start(s);
                            }
                            d.wall_taken.push(w.edits.join(" ; "));
                            stalled_days = 0;
                        }
                    }
                    let best = g.lineage.best_depth;
                    if let Some(boss) = boss_at(best) {
                        if stalled_days >= 1 && riddle_core::facts::boss_counter_known(&g.lineage.facts, boss) && !counters_done.contains(&boss.to_string()) {
                            let rows = counter_rows(boss);
                            let missing: Vec<String> = rows.iter().filter_map(row_unlock).filter(|u| !g.lineage.unlocks.contains(u)).collect();
                            for u in &missing {
                                buy_marks(&mut g, u, &mut d);
                            }
                            let usable = rows.iter().all(|r| verb_ok(&g, &r.verb) && r.conds.iter().all(|c| g.vocabulary().conds.iter().any(|v| v.k == c.k && v.t == c.t)));
                            if usable {
                                for (i, r) in rows.into_iter().enumerate() {
                                    insert_row(&mut g, r, i as i32);
                                }
                                counters_done.push(boss.to_string());
                            } else if boss == "lurker_queen" {
                                let rest = g.lineage.rules().rows.iter().position(|r| r.verb.v == "rest");
                                if g.lineage.unlocks.contains("noise_discipline") || buy_marks(&mut g, "noise_discipline", &mut d) {
                                    let row = Row::new(vec![Cond::n("hp<", 90)], Verb::arg("tactic", "noise_discipline"));
                                    let mut rules = g.lineage.rules().clone();
                                    if let Some(i) = rest {
                                        rules.rows[i] = row;
                                        let _ = g.set_rules(rules);
                                    } else {
                                        insert_row(&mut g, row, i32::MAX);
                                    }
                                }
                            } else if boss == "bloat_mother" && g.lineage.unlocks.contains("throw") {
                                let row = Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 9)], Verb::arg("throw", "unknown,tag:boss"));
                                if !g.lineage.rules().rows.contains(&row) {
                                    insert_row(&mut g, row, 0);
                                }
                            }
                        }
                    }
                    if stalled_days >= 2 && boss_at(best).is_none_or(|b| counters_done.contains(&b.to_string())) {
                        let mut rules = g.lineage.rules().clone();
                        let has_rest = rules.rows.iter().any(|r| r.verb.v == "rest");
                        if !has_rest && rep.deaths.is_empty() {
                            let at = rules.rows.len();
                            if insert_row(&mut g, Row::new(vec![Cond::n("hp<", 70)], Verb::new("rest")), at as i32) {
                                stalled_days = 0;
                            }
                        } else if let Some(i) = rules.rows.iter().position(|r| r.verb.v == "return" && r.conds.iter().any(|c| c.k == "hp<")) {
                            let last = rules.rows.len() - 1;
                            let c = rules.rows[i].conds.iter_mut().find(|c| c.k == "hp<").unwrap();
                            let n = c.n.unwrap_or(30);
                            if n > 20 {
                                c.n = Some(n - 10);
                            } else if i < last {
                                let r = rules.rows.remove(i);
                                rules.rows.push(r);
                            }
                            if rules != *g.lineage.rules() && g.set_rules(rules).is_ok() {
                                stalled_days = 0;
                            }
                        }
                    }
                    // 3. the cheapest affordable unlock, spending down to a reserve of 8
                    let mut bought = 0;
                    loop {
                        if bought > 0 && g.lineage.marks <= 8 {
                            break;
                        }
                        let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.available && !u.gold_only && u.cost <= g.lineage.marks).collect();
                        opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                        let Some(u) = opts.first() else { break };
                        if !buy_marks(&mut g, &u.id, &mut d) {
                            break;
                        }
                        bought += 1;
                        if let Some((row, at)) = riddle_core::meta::delta_row(&g.lineage, &u.id) {
                            if row.is_card() {
                                insert_row(&mut g, row, at as i32);
                            }
                        }
                    }
                    // 3a. one gold buy per visit when the purse holds twice its price
                    let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.gold > 0 && u.needs.as_deref().is_none_or(|n| n.starts_with('◆') || n.starts_with('$')) && g.lineage.gold >= 2 * u.gold as i32).collect();
                    opts.sort_by(|a, b| a.gold.cmp(&b.gold).then(a.id.cmp(&b.id)));
                    if let Some(u) = opts.first() {
                        if buy_gold(&mut g, &u.id.clone(), &mut d) {
                            if let Some((row, at)) = riddle_core::meta::delta_row(&g.lineage, &u.id) {
                                if row.is_card() {
                                    insert_row(&mut g, row, at as i32);
                                }
                            }
                        }
                    }
                    if *sinks {
                        spend_gold(&mut g, &mut d);
                    }
                    draw_down(&mut g, &mut d);
                    field(&mut g);
                }
                Mode::Rater(_) => {
                    // Cut 29 §1 (E1): the wall's edit, taken as the client offers it — the goal set becomes the set with it
                    let fresh = g.lineage.wall_day != Some(g.lineage.day);
                    if let Some(w) = &g.wall_edit().filter(|_| fresh) {
                        goal = Some(w.rules.clone());
                        if let Some(s) = w.start {
                            let _ = g.set_start(s);
                        }
                        d.wall_taken.push(w.edits.join(" ; "));
                    }
                    let set = goal.clone().unwrap_or_default();
                    let set = &set;
                    if g.lineage.ended && g.ascend("no_rest").is_ok() {
                        d.ascended = true;
                    }
                    // what the set needs first (cheapest first), then the dayplayer's rule; the set is
                    // re-written after each buy (a row slot opens only once the rows are full)
                    let reproject = |g: &mut Game| {
                        let p = project(g, set);
                        if g.set_rules(p.clone()).is_err() {
                            let _ = g.set_rules_raw(p);
                        }
                    };
                    reproject(&mut g);
                    loop {
                        let need = needed(&g, set);
                        let cat = g.unlocks();
                        let mut opts: Vec<_> = cat.iter().filter(|u| need.contains(&u.id) && u.available).collect();
                        opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                        let Some(u) = opts.first() else { break };
                        if !buy_marks(&mut g, &u.id.clone(), &mut d) {
                            break;
                        }
                        reproject(&mut g);
                    }
                    let mut bought = 0;
                    loop {
                        if bought > 0 && g.lineage.marks <= 8 {
                            break;
                        }
                        // (a row slot is `fill rows`-gated while the set has a free row: the dayplayer's rule)
                        let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.available && !u.gold_only && u.cost <= g.lineage.marks).collect();
                        opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                        let Some(u) = opts.first() else { break };
                        if !buy_marks(&mut g, &u.id, &mut d) {
                            break;
                        }
                        bought += 1;
                    }
                    let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.gold > 0 && u.needs.as_deref().is_none_or(|n| n.starts_with('◆') || n.starts_with('$')) && g.lineage.gold >= 2 * u.gold as i32).collect();
                    opts.sort_by(|a, b| a.gold.cmp(&b.gold).then(a.id.cmp(&b.id)));
                    if let Some(u) = opts.first() {
                        buy_gold(&mut g, &u.id.clone(), &mut d);
                    }
                    // (the oaths read on the forecast at the day's first check-in: a player swears for the day)
                    spend_gold_reading(&mut g, &mut d, ci == 0);
                    draw_down(&mut g, &mut d);
                    let p = project(&g, set);
                    if g.set_rules(p.clone()).is_err() {
                        let _ = g.set_rules_raw(p);
                    }
                    field(&mut g);
                }
            }
            gold_by_line(&g, t1, gold1, &mut d);
            if day >= 3 {
                let unit = riddle_core::kit::unit_of(&g.lineage) as f64;
                let bar = (1.5 * g.lineage.last_day_net.max(0) as f64).max(10.0 * unit);
                d.purse_ratio = d.purse_ratio.max(g.lineage.gold.max(0) as f64 / bar);
                if g.lineage.gold as f64 > 0.9 * bar && std::env::var("PROG_PURSE").is_ok() {
                    eprintln!("{name} s{seed} day {} ci {ci}: purse ${} bar {bar:.0} (net {} unit {unit}) commission ${} oath ${} works {} forge next {:?}", day + 1, g.lineage.gold, g.lineage.last_day_net, riddle_core::kit::commission_price(&g.lineage), riddle_core::oath::price(&g.lineage), g.lineage.works.len(), riddle_core::kit::ladders(&g.lineage).iter().filter_map(|l| l.next.as_ref().map(|n| n.price)).min());
                }
            }
            if g.lineage.marks > 8 && std::env::var("PROG_MARKS").is_ok() {
                let cat: Vec<String> = g.unlocks().into_iter().filter(|u| !u.owned).map(|u| format!("{}◆{}{}", u.id, u.cost, u.needs.map(|n| format!("[{n}]")).unwrap_or_default())).collect();
                let mut h = g.sim_clone();
                let tried = h.draw_oath();
                eprintln!("{name} s{seed} day {} ci {ci}: ◆{} tier {} best D{} draw {:?} {tried:?} sworn {:?} oaths {:?} | {}", day + 1, g.lineage.marks, riddle_core::meta::tier(&g.lineage), g.lineage.best_depth, riddle_core::oath::draw_wire(&g.lineage).needs, riddle_core::oath::sworn_ids(&g.lineage), g.lineage.oaths.iter().map(|o| format!("{}:{}", o.id, o.reward.label)).collect::<Vec<_>>(), cat.join(" "));
            }
            d.marks_max = d.marks_max.max(g.lineage.marks);
            d.gold_max = d.gold_max.max(g.lineage.gold);
            let _ = ci;
        }
        // Day end: the day's state, then the measures.
        d.best = g.lineage.best_depth;
        d.new_best = d.best > best0 || d.ascended;
        d.counter_known = boss_at(d.best).is_none_or(|b| riddle_core::facts::boss_counter_known(&g.lineage.facts, b));
        d.marks_end = g.lineage.marks;
        d.gold_end = g.lineage.gold;
        d.reveal = reveal(&g).into_iter().map(String::from).collect();
        d.systems_open = g.lineage.systems.len();
        d.level = g.lineage.class_level();
        d.rank = g.lineage.rank;
        d.rows = g.lineage.rules().rows.len();
        d.own_rows = g.lineage.rules().own_rows();
        d.max_rows = g.lineage.max_rows();
        d.kit = riddle_core::kit::KIT_SLOTS.iter().map(|s| format!("{}{}", &s[..1], riddle_core::kit::owned(&g.lineage, s))).collect::<Vec<_>>().join("");
        // each unlock bought today, leave-one-out on the day's end state
        let bought: Vec<String> = d.buys.iter().filter(|b| b.via != "forge").map(|b| b.id.clone()).chain(d.oath_unlocks.iter().cloned()).collect();
        for id in bought.iter().filter(|_| !walls_only) {
            if !g.lineage.unlocks.contains(id) {
                continue;
            }
            let m = panel_move(&g.sim_clone(), &without(&g, id));
            d.measured.push(move_json(id, &m));
        }
        // the forge steps bought today, together (leave-them-out)
        let forged: Vec<String> = d.buys.iter().filter(|b| b.via == "forge").map(|b| b.id.clone()).collect();
        if !forged.is_empty() && !walls_only {
            let mut s = g.sim_clone();
            for f in &forged {
                let slot = f.trim_start_matches("forge:");
                if let Some(n) = s.lineage.kit.get_mut(slot) {
                    *n = n.saturating_sub(1);
                }
            }
            let m = panel_move(&g.sim_clone(), &s);
            d.measured.push(move_json(&format!("forge×{}", forged.len()), &m));
        }
        // affordable and left unbought (marks), the policy's answer measured
        let left: Vec<String> = g.unlocks().into_iter().filter(|u| !u.owned && u.available).map(|u| u.id).collect();
        for id in left.iter().take(if walls_only { 0 } else { 8 }) {
            let m = panel_move(&with(&g, id, &mode), &g.sim_clone());
            d.left.push(move_json(id, &m));
        }
        // the wall probe on a stalled day: what breaks it — the next unslain boss's counter rows
        // written (their unlocks and fire identified, a row slot each), the whole forge, both
        if !d.new_best && probe_walls {
            let best = g.lineage.best_depth;
            if let Some((boss, bd)) = riddle_core::descent::BOSS_DEPTHS.iter().copied().find(|(k, bd)| *bd >= best.saturating_sub(1) && !g.lineage.kills.contains(*k)) {
                let counter = |s: &mut Game| {
                    for k in ["fire", "silence"] {
                        if let Some(f) = riddle_core::item::ident_fact(&s.lineage.flavours, k) {
                            s.lineage.facts.insert(f);
                        }
                    }
                    let rows = counter_rows(boss);
                    let mut rules = s.lineage.rules().clone();
                    for (i, r) in rows.iter().enumerate() {
                        if let Some(u) = row_unlock(r) {
                            s.lineage.unlocks.insert(u);
                        }
                        if !rules.rows.contains(r) {
                            rules.rows.insert(i.min(rules.rows.len()), r.clone());
                        }
                    }
                    for u in ["row5", "row6", "row7", "row8", "row9", "row10"] {
                        if s.lineage.max_rows() >= rules.own_rows() {
                            break;
                        }
                        s.lineage.unlocks.insert(u.into());
                    }
                    let _ = s.set_rules_raw(rules);
                };
                let base = g.sim_clone();
                let mut c = g.sim_clone();
                counter(&mut c);
                let mut k = g.sim_clone();
                riddle_core::kit::buy_all(&mut k.lineage);
                let mut ck = g.sim_clone();
                counter(&mut ck);
                riddle_core::kit::buy_all(&mut ck.lineage);
                // the exits pushed: every `depth ≥ N → bank|return` row moved to the record + 1 (what the
                // plateau note and the `bold` oath ask), and the same with the counter written
                let push = |s: &mut Game| {
                    let mut rules = s.lineage.rules().clone();
                    for r in rules.rows.iter_mut().filter(|r| matches!(r.verb.v.as_str(), "bank" | "return")) {
                        for c in r.conds.iter_mut().filter(|c| c.k == "depth>=") {
                            c.n = Some(c.n.unwrap_or(0).max(best as i32 + 1));
                        }
                    }
                    let _ = s.set_rules_raw(rules);
                };
                let mut p = g.sim_clone();
                push(&mut p);
                let mut pc = g.sim_clone();
                counter(&mut pc);
                push(&mut pc);
                for (what, s) in [("counter", &c), ("kit", &k), ("counter+kit", &ck), ("push", &p), ("push+counter", &pc)] {
                    let m = panel_move(s, &base);
                    let mut j = move_json(what, &m);
                    j["boss"] = json!(boss);
                    j["boss_depth"] = json!(bd);
                    d.wall.push(j);
                }
                // the other lanes: each fork above the record taken the other way (the forks past D5 are
                // closed to players today — `descent::OPEN_FORKS` — but every route is playable in a sim)
                for forks in [vec![5u32], vec![9], vec![14], vec![19], vec![9, 14], vec![5, 9, 14, 19]] {
                    if forks.iter().any(|f| *f > best) {
                        continue;
                    }
                    let mut r = g.sim_clone();
                    let mut rules = r.lineage.rules().clone();
                    let mut route: Vec<u32> = rules.route.clone();
                    for f in &forks {
                        if let Some(i) = route.iter().position(|x| x == f) {
                            route.remove(i);
                        } else {
                            route.push(*f);
                        }
                    }
                    route.sort_unstable();
                    rules.route = route;
                    let _ = r.set_rules_raw(rules);
                    let m = panel_move(&r, &base);
                    let mut j = move_json(&format!("route{forks:?}"), &m);
                    j["boss"] = json!(boss);
                    d.wall.push(j);
                }
            }
        }
        if std::env::var("PROG_OATHS").is_ok() {
            eprintln!("{name} s{seed} day {} D{} ◆{} ${} kept {} draws {} works {} oaths {:?} board {:?} unl {} wall {:?} start {} lday {} wday {:?}", day + 1, d.best, d.marks_end, d.gold_end, d.oaths_kept, d.draws, d.works, d.oaths, g.lineage.oaths.iter().map(|o| format!("{}:{}", o.kind, o.reward.label)).collect::<Vec<_>>(), unlock_day(&d), d.wall_taken, g.lineage.start, g.lineage.day, g.lineage.wall_day);
        }
        if verbose {
            eprintln!("{name} s{seed} day {} best D{} marks {} gold {} buys {:?}", day + 1, d.best, d.marks_end, d.gold_end, d.buys.iter().map(|b| format!("{}:{}", b.id, b.via)).collect::<Vec<_>>());
        }
        // dayplayer stall bookkeeping (its own)
        if d.best >= riddle_core::descent::ENDING_DEPTH || d.best > last_best || d.ascended {
            last_best = d.best;
            stalled_days = 0;
        } else {
            stalled_days += 1;
        }
        table.push(d);
    }
    let mode_s = match &mode {
        Mode::Day { sinks: false } => "dayplayer".to_string(),
        Mode::Day { sinks: true } => "dayplayer+sinks".to_string(),
        Mode::Rater(_) => "rater".to_string(),
    };
    Out { name, mode: mode_s, seed, days: table, rules: g.lineage.rules().rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | ") }
}

pub fn day_json(i: usize, d: &DayRec) -> Value {
    json!({
        "day": i + 1, "checkins": d.checkins, "runs": d.runs, "banked": d.banked, "returned": d.returned, "deaths": d.deaths,
        "marks": {"earned": d.marks_src.values().sum::<i64>(), "by_source": d.marks_src, "spent": d.marks_spent,
                  "spent_on": d.buys.iter().filter(|b| b.via == "marks").map(|b| json!({"id": b.id, "cost": b.cost})).collect::<Vec<_>>(),
                  "unspent_end": d.marks_end, "unspent_max": d.marks_max},
        "gold": {"earned": d.gold_in.values().sum::<i64>(), "by_source": d.gold_in, "spent": d.gold_out.values().sum::<i64>(), "by_sink": d.gold_out,
                 "end": d.gold_end, "max": d.gold_max},
        "best_depth": d.best, "new_best": d.new_best, "wall_counter_known": d.counter_known,
        "reveal": d.reveal,
        "unlocks_bought": d.buys.iter().filter(|b| b.via != "forge").map(|b| json!({"id": b.id, "via": b.via, "cost": b.cost, "gold": b.gold})).collect::<Vec<_>>(),
        "unlocks_from_oaths": d.oath_unlocks,
        "forge": d.buys.iter().filter(|b| b.via == "forge").map(|b| json!({"slot": b.id, "gold": b.gold})).collect::<Vec<_>>(),
        "kit": d.kit, "oaths": d.oaths,
        "measured": d.measured, "affordable_left": d.left, "wall_probe": d.wall, "frontier_per_checkin": d.frontier_ci,
        "level": d.level, "rank": d.rank, "rows": d.rows, "own_rows": d.own_rows, "max_rows": d.max_rows, "ascended": d.ascended,
    })
}

/// The per-lineage summary: the bars and where the curve goes flat.
pub fn summary(o: &Out) -> Value {
    let days = &o.days;
    let unlock_days = days.iter().filter(|d| unlock_day(d)).count();
    let content_days = days.iter().filter(|d| content_day(d)).count();
    let meaningful_days: Vec<usize> = days.iter().enumerate().filter(|(_, d)| d.measured.iter().any(|m| m["meaningful"] == json!(true) && !m["id"].as_str().unwrap_or("").starts_with("forge"))).map(|(i, _)| i + 1).collect();
    let (stall_raw, stall_known) = stalls(o);
    let marks_worst = days.iter().skip(2).map(|d| d.marks_max).max().unwrap_or(0);
    let gold_worst = days.iter().map(|d| d.gold_max).max().unwrap_or(0);
    let last_new_best = days.iter().rposition(|d| d.new_best).map(|i| i + 1);
    let bought: Vec<Value> = days.iter().flat_map(|d| d.measured.iter().cloned()).collect();
    let useless: Vec<String> = bought.iter().filter(|m| m["meaningful"] == json!(false)).map(|m| m["id"].as_str().unwrap_or("").to_string()).collect();
    let frontier: i64 = days.iter().map(|d| d.marks_src.get("frontier").copied().unwrap_or(0)).sum();
    json!({
        "name": o.name, "mode": o.mode, "seed": o.seed,
        "final_best": days.last().map(|d| d.best), "last_new_best_day": last_new_best,
        "days_with_unlock": unlock_days, "days_with_content": content_days, "days_with_meaningful_unlock": meaningful_days.len(), "last_meaningful_unlock_day": meaningful_days.last(),
        "stall_raw": stall_raw, "stall_counter_known": stall_known,
        "marks_unspent_worst_after_day2": marks_worst, "gold_worst": gold_worst, "purse_ratio_worst": r3(purse_worst(o)),
        "frontier_marks": frontier, "systems_open_by_day": days.iter().map(|d| d.systems_open).collect::<Vec<_>>(),
        "bought_measured": bought.len(), "bought_no_move": useless,
        "rules_end": o.rules,
    })
}

/// Cut 29 §1 (docs/PROGRESSION.md G1): a day with an unlock bought (marks or gold), an oath kept or a
/// system opened; `content_day` the same without the systems.
pub fn unlock_day(d: &DayRec) -> bool {
    content_day(d) || !d.systems_opened.is_empty()
}
pub fn content_day(d: &DayRec) -> bool {
    d.buys.iter().any(|b| b.via != "forge") || !d.oath_unlocks.is_empty() || d.oaths_kept > 0
}

/// (longest run of days without a new best, the same counting only walls whose counter is known).
pub fn stalls(o: &Out) -> (usize, usize) {
    let (mut stall_raw, mut stall_known, mut cur_r, mut cur_k) = (0, 0, 0, 0);
    for d in &o.days {
        if d.new_best {
            cur_r = 0;
            cur_k = 0;
        } else {
            cur_r += 1;
            if d.counter_known {
                cur_k += 1;
            } else {
                cur_k = 0;
            }
        }
        stall_raw = stall_raw.max(cur_r);
        stall_known = stall_known.max(cur_k);
    }
    (stall_raw, stall_known)
}

pub fn purse_worst(o: &Out) -> f64 {
    o.days.iter().map(|d| d.purse_ratio).fold(0.0, f64::max)
}

/// Cut 29 §1: the progression rows over the rater lineages (`examples/metrics.rs`): (name, value, pass).
/// Unlock days ≥ 10/14 (mean); marks unspent ≤ 8 at the worst check-in after day 2 (every lineage);
/// the longest best-depth stall (counter known) ≤ 3 days on ≥ 13/18 lineages; the purse ≤ max(1.5 ×
/// the last day's net, 10 forge units) at every check-in after day 3 (every lineage); no frontier marks.
pub fn bars(outs: &[Out]) -> Vec<(String, String, bool)> {
    let raters: Vec<&Out> = outs.iter().filter(|o| o.mode == "rater").collect();
    if raters.is_empty() {
        return Vec::new();
    }
    let n = raters.len();
    let days = raters[0].days.len();
    let unlock: Vec<usize> = raters.iter().map(|o| o.days.iter().filter(|d| unlock_day(d)).count()).collect();
    let content: Vec<usize> = raters.iter().map(|o| o.days.iter().filter(|d| content_day(d)).count()).collect();
    let mean = unlock.iter().sum::<usize>() as f64 / n as f64;
    let marks = raters.iter().map(|o| o.days.iter().skip(2).map(|d| d.marks_max).max().unwrap_or(0)).max().unwrap_or(0);
    let stall_ok = raters.iter().filter(|o| stalls(o).1 <= 3).count();
    let need = (n * 13).div_ceil(18);
    let purse = raters.iter().map(|o| purse_worst(o)).fold(0.0, f64::max);
    let frontier: i64 = raters.iter().flat_map(|o| o.days.iter()).map(|d| d.marks_src.get("frontier").copied().unwrap_or(0)).sum();
    vec![
        (format!("Progression: days with an unlock, oath kept or system ≥ 10 / {days} (mean, {n} rater lineages)"), format!("{mean:.1} (content {:.1})", content.iter().sum::<usize>() as f64 / n as f64), mean >= 10.0),
        ("Progression: marks unspent at any check-in after day 2 ≤ 8".into(), format!("{marks}"), marks <= 8),
        (format!("Progression: longest stall (counter known) ≤ 3 d on ≥ {need}/{n} lineages"), format!("{stall_ok}/{n}"), stall_ok >= need),
        ("Progression: purse ≤ max(1.5 days' net, 10 units) after day 3".into(), format!("{purse:.2}×"), purse <= 1.0 + 1e-9),
        ("Progression: no marks for standing still (frontier source)".into(), format!("{frontier}"), frontier == 0),
    ]
}


/// The rater sets the timeline replays: the last four cohorts' cards (`eval/cards/<build>.*.rules.json`).
pub const RATER_BUILDS: [&str; 3] = ["631fe23", "420f27c", "307dbed"];
pub fn rater_sets() -> Vec<(String, RuleSet)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            RATER_BUILDS.iter().any(|b| stem.starts_with(b)).then_some(())?;
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .collect();
    sets.sort_by(|x, y| x.0.cmp(&y.0));
    sets
}

/// The rater's day: three absences (8 h · 20 m · 4 h · 4 h · 7 h 40 m); the dayplayer's: 3 × 8 h.
pub fn three_absence() -> Vec<u64> {
    vec![8 * 3600, 20 * 60, 4 * 3600, 4 * 3600, 7 * 3600 + 40 * 60]
}
