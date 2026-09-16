//! Death records: trace, margin, morgue, and the verdict (gap/dice) with candidate patches
//! found by replaying the last ~100 ticks with one added row from the unlocked vocabulary.
use crate::engine::{DeathRec, ExitTier, Game, Run};
use crate::item::is_identified;
use crate::rng::{splitmix, Rng};
use crate::rules::{Cond, Row, RuleSet, Vocabulary};
use crate::wire::{Death, Ev, Patch, Trace};
use std::collections::BTreeSet;

pub const REPLAYS: u32 = 20;
pub const SURVIVE_BAR: f64 = 0.6;
/// How many survival-ranked candidates get a full forecast delta before the final cut to three.
pub const DELTA_CANDIDATES: usize = 6;
/// A patch must beat the unpatched baseline by this much to count as the fix.
pub const PATCH_MARGIN: f64 = 0.15;
pub const TRACE_LEN: usize = 10;

pub fn death_record(game: &Game, run: &Run) -> DeathRec {
    let turns: Vec<_> = run.trace.iter().rev().take(TRACE_LEN).rev().cloned().collect();
    let cause = run.death_cause.clone().unwrap_or_else(|| "unknown".into());
    let mut margin = format!("{} over", run.death_blow.max(0));
    let facts = &game.lineage.facts;
    let fl = &game.lineage.flavours;
    if run.hero.inv.iter().any(|i| i.kind == "heal" && is_identified(facts, fl, "heal")) {
        margin.push_str(" · heal unused");
    }
    let unknown = run.hero.inv.iter().filter(|i| i.is_consumable() && !is_identified(facts, fl, &i.kind)).count();
    if unknown > 0 {
        margin.push_str(&format!(" · {unknown} unknown unused"));
    }
    let rules = game.lineage.rules().clone();
    let vocab = game.vocabulary();
    let death = Death {
        run_id: run.id,
        depth: run.depth,
        cause: cause.clone(),
        margin,
        verdict: "dice".into(),
        baseline: 0.0,
        trace: Trace { turns },
        patches: Vec::new(),
        morgue: morgue(game, run, &rules),
    };
    let (t10, t10_facts) = match game.history.front() {
        Some((r, f)) => (Some(r.clone()), f.clone()),
        None => (None, BTreeSet::new()),
    };
    DeathRec { death, t10, t10_facts, rules, vocab, verdict_done: false, deltas_done: false }
}

fn morgue(game: &Game, run: &Run, rules: &RuleSet) -> String {
    let mut s = String::new();
    s.push_str(&format!("Riddle morgue · seed {} · heir {} · run {}\n", game.lineage.seed, run.heir, run.id));
    s.push_str(&format!(
        "D{} ({}) · tick {} · slain by {} · {} HP over\n",
        run.depth,
        run.biome().name(),
        run.turn,
        run.death_cause.clone().unwrap_or_default(),
        run.death_blow
    ));
    s.push_str(&format!("class {} L{} · trait {}\n", run.hero.class.name(), run.hero.level, run.trait_.name()));
    s.push_str("rules:\n");
    for (i, r) in rules.rows.iter().enumerate() {
        s.push_str(&format!("  R{} {}\n", i + 1, r.describe()));
    }
    s.push_str("trace:\n");
    for t in run.trace.iter().rev().take(TRACE_LEN).rev() {
        let row = match t.row {
            -1 => "trait".to_string(),
            -2 => "chore".to_string(),
            r => format!("R{}", r + 1),
        };
        s.push_str(&format!("  t{} {} {} hp {} foes {} {}\n", t.t, row, t.verb.short(), t.hp, t.foes, t.telegraphs.join(",")));
    }
    s.push_str("notes:\n");
    for (t, n) in run.notes.iter().rev().take(8).rev() {
        s.push_str(&format!("  t{t} {n}\n"));
    }
    s.push_str(&format!("rules json: {}\n", serde_json::to_string(rules).unwrap_or_default()));
    s
}

/// The candidate rows: every vocabulary verb under a small set of condition combinations.
pub fn candidates(vocab: &Vocabulary, state: &Run, facts: &BTreeSet<String>, flavours: &crate::item::Flavours) -> Vec<Row> {
    let known_tags: Vec<String> = vocab.conds.iter().filter(|c| c.k == "foe_tag").filter_map(|c| c.t.clone()).collect();
    // Every family: consumables / ID policy / escape (hp<N), retreat (foes, adj, hurt),
    // targeting (foe_tag), and the unconditioned row (which must beat the baseline clearly).
    let mut cond_sets: Vec<Vec<Cond>> = vec![
        vec![],
        vec![Cond::n("hp<", 20)],
        vec![Cond::n("hp<", 30)],
        vec![Cond::n("hp<", 50)],
        vec![Cond::n("foes>=", 1)],
        vec![Cond::n("foes>=", 2)],
        vec![Cond::n("adj>=", 1)],
        vec![Cond::n("hp<", 50), Cond::n("foes>=", 2)],
        vec![Cond::n("hp<", 60), Cond::n("adj>=", 1)],
        vec![Cond::flag("on_hurt")],
        vec![Cond::n("hp<", 40), Cond::flag("on_hurt")],
    ];
    for t in &known_tags {
        cond_sets.push(vec![Cond::t("foe_tag", t)]);
    }
    let has_unknown = state.hero.inv.iter().any(|i| i.is_consumable() && !is_identified(facts, flavours, &i.kind));
    let mut verbs = Vec::new();
    for v in &vocab.verbs {
        let a = v.a.as_deref().unwrap_or("");
        let ok = match v.v.as_str() {
            "drink" | "read" => {
                if a == "unknown" {
                    has_unknown
                } else {
                    state.hero.inv.iter().any(|i| i.kind == a)
                }
            }
            "throw" => state.hero.inv.iter().any(|i| i.kind == a.split(',').next().unwrap_or("")),
            "pick_up" | "free_captive" | "tame" | "recall" | "send" | "rest" => false,
            "attack" => a != "lowest",
            _ => true,
        };
        if ok {
            verbs.push(v.clone());
        }
    }
    let mut out = Vec::new();
    for v in &verbs {
        for cs in &cond_sets {
            // Quitting or diving unconditionally is not a policy; require a condition.
            if cs.is_empty() && matches!(v.v.as_str(), "return" | "bank" | "descend") {
                continue;
            }
            out.push(Row::new(cs.clone(), v.clone()));
        }
    }
    out
}

/// One replay of the last ticks with `rules`, reseeded. Returns (survived, patch row fired).
fn replay(base: &Game, rules: &RuleSet, ticks: u32, nonce: u64, watch_row: i32) -> (bool, bool) {
    let mut g = base.sim_clone();
    let _ = g.set_rules(rules.clone());
    if let Some(run) = g.run.as_mut() {
        run.rng = Rng::derive(base.lineage.seed ^ splitmix(nonce), run.turn as u64);
    }
    let mut fired = false;
    for _ in 0..ticks {
        g.tick();
        if !fired {
            fired = g.events.iter().any(|e| matches!(e, Ev::Rule { row, .. } if *row == watch_row));
        }
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            break;
        }
    }
    let survived = g.run.as_ref().is_some_and(|r| r.over != Some(ExitTier::Death));
    (survived, fired)
}

/// Compute the verdict and patches (survival only) for a recorded death.
pub fn compute_verdict(game: &Game, rec: &mut DeathRec) {
    if rec.verdict_done {
        return;
    }
    rec.verdict_done = true;
    let Some(t10) = rec.t10.clone() else { return };
    let mut base = game.sim_clone();
    base.lineage.facts = rec.t10_facts.clone();
    base.lineage.heir = t10.heir;
    base.lineage.trait_ = t10.trait_;
    base.lineage.class = t10.hero.class;
    base.lineage.party.clear();
    base.lineage.supplies.clear();
    let death_turn = game.deaths.get(&rec.death.run_id).map(|_| 0).unwrap_or(0);
    let _ = death_turn;
    let last_t = rec.death.trace.turns.last().map(|t| t.t).unwrap_or(t10.turn + 100);
    let ticks = (last_t.saturating_sub(t10.turn)).max(1) + 1;
    base.run = Some(t10.clone());
    // Where to insert: top, and before the row that fired most in the trace.
    let mut fired_idx: Option<usize> = None;
    let mut counts = vec![0u32; rec.rules.rows.len()];
    for t in &rec.death.trace.turns {
        if t.row >= 0 && (t.row as usize) < counts.len() {
            counts[t.row as usize] += 1;
        }
    }
    if let Some((i, c)) = counts.iter().enumerate().max_by_key(|(i, c)| (**c, usize::MAX - *i)) {
        if *c > 0 && i > 0 {
            fired_idx = Some(i);
        }
    }
    let positions: Vec<usize> = std::iter::once(0).chain(fired_idx).collect();
    // Baseline: the unpatched rules under the same reseeded replays. If they survive most of
    // the time the death was the dice, not the policy; a patch must beat the baseline clearly.
    let mut base_survived = 0u32;
    for i in 0..REPLAYS {
        let (s, _) = replay(&base, &rec.rules, ticks, 0xBA5E_0000 | i as u64, -99);
        base_survived += s as u32;
    }
    let baseline = base_survived as f64 / REPLAYS as f64;
    rec.death.baseline = baseline;
    let cands = candidates(&rec.vocab, &t10, &rec.t10_facts, &game.lineage.flavours);
    let max_rows = rec.vocab.max_rows.max(rec.rules.rows.len() + 1);
    let mut scored: Vec<(f64, Row, usize)> = Vec::new();
    for (ci, row) in cands.iter().enumerate() {
        for &pos in &positions {
            let mut rules = rec.rules.clone();
            rules.rows.insert(pos, row.clone());
            rules.rows.truncate(max_rows);
            let mut survived = 0u32;
            let mut failed = 0u32;
            let mut any_fired = false;
            for i in 0..REPLAYS {
                let nonce = (ci as u64) << 32 | (pos as u64) << 16 | i as u64;
                let (s, f) = replay(&base, &rules, ticks, nonce, pos as i32);
                any_fired |= f;
                if s {
                    survived += 1;
                } else {
                    failed += 1;
                }
                if i == 0 && !f {
                    break; // the row never fires here: identical to the original
                }
                if failed as f64 > REPLAYS as f64 * (1.0 - SURVIVE_BAR) {
                    break;
                }
            }
            if !any_fired {
                continue;
            }
            let rate = survived as f64 / REPLAYS as f64;
            if rate >= SURVIVE_BAR {
                scored.push((rate, row.clone(), pos));
            }
        }
    }
    // Rank by survival; among equals prefer rows that change behaviour least (fewer conds, top).
    // Rank by how much the row beats the unpatched baseline, then by simplicity.
    // Ties: a conditioned row (a policy) beats an unconditioned one; then fewer conditions.
    scored.sort_by(|a, b| {
        (b.0 - baseline)
            .partial_cmp(&(a.0 - baseline))
            .unwrap()
            .then(a.1.conds.is_empty().cmp(&b.1.conds.is_empty()))
            .then(a.1.conds.len().cmp(&b.1.conds.len()))
            .then(a.2.cmp(&b.2))
    });
    if let Some(best) = scored.first() {
        if best.0 >= SURVIVE_BAR {
            rec.death.verdict = "gap".into();
        }
    }
    let mut seen_verbs: Vec<String> = Vec::new();
    rec.death.patches = scored
        .into_iter()
        .filter(|(_, row, _)| {
            let key = format!("{}:{}", row.verb.v, row.verb.a.clone().unwrap_or_default());
            if seen_verbs.contains(&key) {
                false
            } else {
                seen_verbs.push(key);
                true
            }
        })
        .take(8)
        .map(|(rate, row, pos)| Patch { row, insert_at: pos, survive: rate, forecast_delta: 0.0 })
        .collect();
}

/// Fill in each patch's full-forecast delta at the death's depth.
pub fn compute_deltas(game: &Game, rec: &mut DeathRec) {
    if rec.deltas_done {
        return;
    }
    rec.deltas_done = true;
    if rec.death.patches.is_empty() {
        return;
    }
    let depth = (rec.death.depth + 1).min(game.lineage.best_depth + 1).max(1);
    let sims = crate::forecast::DELTA_SIMS;
    // Forecasts cost ~20 sims each: rank by survival edge first and only forecast the top few.
    let baseline0 = rec.death.baseline;
    rec.death.patches.sort_by(|a, b| (b.survive - baseline0).partial_cmp(&(a.survive - baseline0)).unwrap().then(a.row.conds.len().cmp(&b.row.conds.len()).reverse()));
    rec.death.patches.truncate(DELTA_CANDIDATES);
    let base = crate::forecast::reach_with(game, &rec.rules, depth, sims, 0xDE17A);
    for p in rec.death.patches.iter_mut() {
        let mut rules = rec.rules.clone();
        rules.rows.insert(p.insert_at.min(rules.rows.len()), p.row.clone());
        rules.rows.truncate(rec.vocab.max_rows.max(rules.rows.len()));
        let r = crate::forecast::reach_with(game, &rules, depth, sims, 0xDE17A);
        p.forecast_delta = r - base;
    }
    // A patch must beat the baseline by 0.15 or move the forecast by 0.02; an unconditioned row
    // must beat the baseline by 0.30. Rank by (survive − baseline), then by the delta; keep three.
    let baseline = rec.death.baseline;
    rec.death.patches.retain(|p| {
        let edge = p.survive - baseline;
        (edge > PATCH_MARGIN || p.forecast_delta > 0.02) && (!p.row.conds.is_empty() || edge >= 0.3)
    });
    rec.death.patches.sort_by(|a, b| {
        (b.survive - baseline).partial_cmp(&(a.survive - baseline)).unwrap().then(b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap())
    });
    rec.death.patches.truncate(3);
}

/// The full death for a run id, computing verdict and deltas on first request.
pub fn death(game: &mut Game, run_id: u32) -> Option<Death> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    compute_deltas(game, &mut rec);
    let d = rec.death.clone();
    game.deaths.insert(run_id, rec);
    Some(d)
}

/// Verdict only (no forecast deltas), for the metrics.
pub fn verdict(game: &mut Game, run_id: u32) -> Option<String> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    let v = rec.death.verdict.clone();
    game.deaths.insert(run_id, rec);
    Some(v)
}
