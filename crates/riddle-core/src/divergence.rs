//! Cut 27 §2 (P4: "the edit is a scene"): the divergence replay. On an edit, the core finds on a
//! paired panel seed the first tick at which the new set acts differently from the sent set and
//! returns both branches' next seconds for the renderer, with each branch's whole-run end. A
//! pure read of the camp's paired sims (the same seeds the forecast's `vs` pairs): no state moves.
use crate::engine::{ExitTier, Game};
use crate::forecast::{camp_panel, camp_sims, forecast_tag, sim_game, sim_passage, SimResult};
use crate::rules::{Row, RuleSet};
use crate::wire::{Divergence, DivergenceBranch, DivergenceEnd, Ev, RowFires};

/// Ticks of each branch shown before the divergence tick (the moment's context).
pub const DIVERGENCE_LEAD: u32 = 12;
/// Ticks of each branch shown after it (3–5 s on the renderer).
pub const DIVERGENCE_AFTER: u32 = 60;
/// Seeds played in step at most (the ones whose ends differ first).
pub const DIVERGENCE_SEEDS: usize = 8;
/// Ticks played in step at most over all seeds tried (each tick is one of each branch): the
/// search stays inside an edit's refine (Cut 25 §4) on a deep lineage.
pub const DIVERGENCE_TICK_BUDGET: u64 = 30_000;
/// Past the first tick the branches act differently, the rows are looked for this many ticks more.
pub const ROW_HORIZON: u32 = 3_000;
/// A checkpoint of both branches every this many ticks (the window replays from the last one
/// at least `DIVERGENCE_LEAD` before the divergence).
const CHECKPOINT_EVERY: u32 = 40;

impl Game {
    /// Cut 27 §2: the active set's divergence from `prev` (`divergence::divergence`).
    pub fn divergence(&self, prev: &RuleSet) -> Option<Divergence> {
        divergence(self, prev)
    }
}

/// How a sim ended, as the scene's caption reads it.
fn end_of(r: &SimResult) -> DivergenceEnd {
    let tier = if r.timed_out {
        "stall"
    } else {
        match r.tier {
            ExitTier::Bank => "bank",
            ExitTier::Return => "return",
            ExitTier::Death => "death",
        }
    };
    DivergenceEnd { tier: tier.into(), depth: r.max_depth, cause: r.cause.clone(), gold: r.loot_kept }
}

/// How much two sims' ends differ: a death against none first, then the floor reached, then the
/// exit (0: alike).
fn end_gap(a: &SimResult, b: &SimResult) -> u32 {
    let death = |r: &SimResult| r.tier == ExitTier::Death;
    if death(a) != death(b) {
        1000 + a.max_depth.abs_diff(b.max_depth)
    } else if a.max_depth != b.max_depth {
        100 + a.max_depth.abs_diff(b.max_depth)
    } else if a.tier != b.tier || a.timed_out != b.timed_out {
        10
    } else if a.loot_kept != b.loot_kept || a.fires != b.fires {
        1
    } else {
        0
    }
}

/// A row as the scene names it (`R5 bank`), ≤ 3 words.
fn row_words(i: usize, row: &Row) -> String {
    crate::chronicle::clamp_words(&format!("R{} {}", i + 1, row.verb.short()), 3)
}

/// The rows a tick fired, as (row index, the row) of `rules` — a card's rows are its own.
fn fired(events: &[Ev], rules: &RuleSet) -> Option<(u32, Row)> {
    events.iter().find_map(|e| match e {
        Ev::Rule { row, .. } if *row >= 0 => rules.rows.get(*row as usize).map(|r| (*row as u32, r.clone())),
        _ => None,
    })
}

/// Whether a tick fired a row of `set` that `other` lacks (by conditions and verb).
fn lacking(events: &[Ev], set: &RuleSet, other: &RuleSet) -> bool {
    events.iter().any(|e| match e {
        Ev::Rule { row, .. } if *row >= 0 => set.rows.get(*row as usize).is_some_and(|r| !other.rows.iter().any(|o| o.conds == r.conds && o.verb == r.verb)),
        _ => false,
    })
}

/// A tick's events with the row indices taken out (a row moved or inserted shifts every index
/// below it; what fired is compared by the row itself, `same_rows`).
fn plain(events: &[Ev]) -> Vec<Ev> {
    events
        .iter()
        .map(|e| match e {
            Ev::Rule { t, row, verb, text } => Ev::Rule { t: *t, row: if *row >= 0 { 0 } else { *row }, verb: verb.clone(), text: text.clone() },
            e => e.clone(),
        })
        .collect()
}

/// Whether the rows two ticks fired are the same rows (by conditions and verb).
fn same_rows(a: &[Ev], ra: &RuleSet, b: &[Ev], rb: &RuleSet) -> bool {
    let rows = |es: &[Ev], rs: &RuleSet| -> Vec<Option<(Vec<crate::rules::Cond>, crate::rules::Verb)>> {
        es.iter()
            .filter_map(|e| match e {
                Ev::Rule { row, .. } if *row >= 0 => Some(rs.rows.get(*row as usize).map(|r| (r.conds.clone(), r.verb.clone()))),
                _ => None,
            })
            .collect()
    };
    rows(a, ra) == rows(b, rb)
}

/// Cut 27 §2: the scene of the active set's edit against `prev` (the sent set) — see
/// `wire::Divergence`. The panels are the camp's (the forecast's pass: cached when the camp
/// painted its `vs`); the seeds are tried ends-apart first, in step, under
/// `DIVERGENCE_TICK_BUDGET`.
pub fn divergence(game: &Game, prev: &RuleSet) -> Option<Divergence> {
    let rules = game.lineage.rules().clone();
    let sims = camp_sims(game, &rules);
    let a = camp_panel(game, &rules, sims);
    let b = camp_panel(game, prev, sims);
    let n = a.len().min(b.len());
    if n == 0 {
        return None;
    }
    // the paired move's headline (`forecast_vs`'s, over the seeds both panels ran): the largest
    // move over the shaft and the ends, and its ±
    let (pa_, pb_) = (&a[..n], &b[..n]);
    let ind = |x: bool| if x { 1.0 } else { 0.0 };
    let last = (game.lineage.best_depth + 1).max(game.lineage.bounty.unwrap_or(0));
    let mut moves: Vec<(f64, f64)> = (1..=last)
        .map(|d| {
            let m = crate::forecast::paired(pa_, pb_, |r| ind(r.max_depth >= d));
            (m.delta, m.pm)
        })
        .collect();
    for f in [
        &(|r: &SimResult| ind(r.tier == ExitTier::Bank && !r.timed_out)) as &dyn Fn(&SimResult) -> f64,
        &|r: &SimResult| ind(r.tier == ExitTier::Death),
        &|r: &SimResult| ind(r.tier == ExitTier::Return && !r.timed_out),
        &|r: &SimResult| ind(r.timed_out),
    ] {
        let m = crate::forecast::paired(pa_, pb_, f);
        moves.push((m.delta, m.pm));
    }
    let (moved, pm) = moves.iter().copied().fold((0.0f64, 0.0f64), |acc, (d, p)| if d.abs() > acc.0 { (d.abs(), p) } else { acc });
    let inside = moved <= pm + 1e-9;
    let mut order: Vec<(u32, usize)> = (0..n).map(|i| (end_gap(&a[i], &b[i]), i)).collect();
    order.sort_by(|x, y| y.0.cmp(&x.0).then(x.1.cmp(&y.1)));
    let tag = forecast_tag(game, game.lineage.best_depth + 1);
    let (pa, pn) = (sim_passage(game, prev), sim_passage(game, &rules));
    let mut spent = 0u64;
    let mut found = None;
    // the first seed where the sets part at a row; else the first where they act differently at all
    // (a seed whose two sims ended alike to the coin, with every row firing as often, played alike:
    // it is not tried)
    for &(_, i) in order.iter().filter(|(gap, _)| *gap > 0).take(DIVERGENCE_SEEDS) {
        if spent >= DIVERGENCE_TICK_BUDGET {
            break;
        }
        if let Some((by_rows, d)) = in_step(game, prev, &rules, tag, i as u32, pa, pn, &mut spent) {
            if by_rows {
                found = Some((i, d));
                break;
            }
            found.get_or_insert((i, d));
        }
    }
    let (i, (tick, depth, sent, new)) = found?;
    Some(Divergence {
        seed: i as u32,
        tick,
        depth,
        sent_row: sent.row,
        new_row: new.row,
        sent_end: end_of(&b[i]),
        new_end: end_of(&a[i]),
        sent,
        new,
        moved,
        inside,
        fires: row_fires(&b[..n], prev, &a[..n], &rules),
        sims: n as u32,
    })
}

/// Seed `i` of both sets played in step: the first tick they act differently, its floor, and
/// both branches' windows. None: they never do (or the budget ran out).
#[allow(clippy::too_many_arguments)]
fn in_step(game: &Game, prev: &RuleSet, rules: &RuleSet, tag: u64, i: u32, pa: Option<(u32, i32)>, pn: Option<(u32, i32)>, spent: &mut u64) -> Option<(bool, (u32, u32, DivergenceBranch, DivergenceBranch))> {
    let mut s = sim_game(game, prev, tag, i, pa);
    let mut w = sim_game(game, rules, tag, i, pn);
    // the start's own events (a set packs its row kinds: the starts may differ already)
    let (e0s, e0w) = (std::mem::take(&mut s.events), std::mem::take(&mut w.events));
    let turn = |g: &Game| g.run.as_ref().map_or(0, |r| r.turn);
    let over = |g: &Game| g.run.as_ref().is_none_or(|r| r.over.is_some());
    // checkpoints (tick, sent, new): the last two
    let mut marks: Vec<(u32, Game, Game)> = vec![(0, s.sim_clone(), w.sim_clone())];
    // A tick parts the sets when either fires a row the other set lacks (for a pure reorder, when the
    // rows they fire differ): the scene is the rows. The first tick they act differently at all (a
    // chore, a step: the packs differed) is kept as the fallback, and the search for the rows goes on
    // `ROW_HORIZON` ticks past it.
    let reorder = {
        let keys = |r: &RuleSet| {
            let mut k: Vec<String> = r.rows.iter().map(|x| format!("{:?}{:?}", x.conds, x.verb)).collect();
            k.sort();
            k
        };
        keys(prev) == keys(rules)
    };
    let parts = |es: &[Ev], ew: &[Ev]| -> bool {
        if reorder {
            !same_rows(es, prev, ew, rules)
        } else {
            lacking(es, prev, rules) || lacking(ew, rules, prev)
        }
    };
    type At = (u32, u32, Option<(u32, Row)>, Option<(u32, Row)>);
    let at = |t: u32, g: &Game, es: &[Ev], ew: &[Ev]| -> At { (t, g.run.as_ref().map_or(1, |r| r.depth), fired(es, prev), fired(ew, rules)) };
    let mut first: Option<At> = (plain(&e0s) != plain(&e0w) || !same_rows(&e0s, prev, &e0w, rules)).then(|| at(0, &s, &e0s, &e0w));
    let mut hit: Option<At> = parts(&e0s, &e0w).then(|| at(0, &s, &e0s, &e0w));
    let mut first_mark: Option<(u32, Game, Game)> = None;
    while hit.is_none() {
        let t = turn(&s);
        if *spent >= DIVERGENCE_TICK_BUDGET || t >= crate::engine::MAX_TURNS_PER_RUN || first.as_ref().is_some_and(|f| t > f.0 + ROW_HORIZON) {
            break;
        }
        if over(&s) || over(&w) {
            // (one branch's run ended here, the other's goes on: they act differently now)
            if first.is_none() && over(&s) != over(&w) {
                first = Some(at(t, &s, &[], &[]));
            }
            break;
        }
        if t > 0 && t.is_multiple_of(CHECKPOINT_EVERY) {
            marks.push((t, s.sim_clone(), w.sim_clone()));
            if marks.len() > 2 {
                marks.remove(0);
            }
        }
        s.tick();
        w.tick();
        *spent += 1;
        let (es, ew) = (std::mem::take(&mut s.events), std::mem::take(&mut w.events));
        let t = turn(&s);
        if first.is_none() && (plain(&es) != plain(&ew) || !same_rows(&es, prev, &ew, rules)) {
            first = Some(at(t, &s, &es, &ew));
            // (the checkpoint its window would replay from, kept past the later ones)
            first_mark = marks.iter().rev().find(|m| m.0 <= t.saturating_sub(DIVERGENCE_LEAD)).map(|m| (m.0, m.1.sim_clone(), m.2.sim_clone()));
        }
        if parts(&es, &ew) {
            hit = Some(at(t, &s, &es, &ew));
        }
    }
    let by_rows = hit.is_some();
    let (t0, depth, row_s, row_w) = hit.or(first)?;
    if !by_rows {
        if let Some(m) = first_mark {
            marks = vec![m];
        }
    }
    // the checkpoints are the last two before where the stepping stopped: the window replays from
    // the latest at least LEAD before `t0` (or from the start)
    let from = t0.saturating_sub(DIVERGENCE_LEAD);
    let (_, cs, cw) = marks.into_iter().rev().find(|(t, _, _)| *t <= from).unwrap_or_else(|| (0, sim_game(game, prev, tag, i, pa), sim_game(game, rules, tag, i, pn)));
    let branch = |mut g: Game, set: &RuleSet, row: Option<(u32, Row)>, spent: &mut u64| -> DivergenceBranch {
        // (a checkpoint from before the run's first tick carries the start's events: they are shown)
        let mut events: Vec<Ev> = Vec::new();
        while turn(&g) < from && !over(&g) {
            g.tick();
            *spent += 1;
            g.events.clear();
        }
        let snapshot = g.snapshot();
        if turn(&g) == 0 {
            events.append(&mut g.events);
        }
        while turn(&g) < t0 + DIVERGENCE_AFTER && !over(&g) {
            g.tick();
            *spent += 1;
            let es = std::mem::take(&mut g.events);
            if let Some(k) = es.iter().position(|e| matches!(e, Ev::Descend { .. })) {
                events.extend(es[..k].iter().cloned());
                break;
            }
            events.extend(es);
        }
        let end_snapshot = g.snapshot();
        let (row, text) = match row {
            Some((r, ref rr)) => (Some(r), row_words(r as usize, rr)),
            None => (None, "—".to_string()),
        };
        let _ = set;
        DivergenceBranch { row, text, snapshot, events, end_snapshot }
    };
    let sent = branch(cs, prev, row_s, spent);
    let new = branch(cw, rules, row_w, spent);
    Some((by_rows, (t0, depth, sent, new)))
}

/// Each row's mean fires per send in both sets over the paired panel, matched by the row
/// (conditions and verb): the rows whose fires moved, the largest move first (at most 4).
fn row_fires(b: &[SimResult], prev: &RuleSet, a: &[SimResult], rules: &RuleSet) -> Vec<RowFires> {
    let n = a.len().min(b.len()).max(1) as f64;
    let mean = |panel: &[SimResult], row: &Row| {
        let k = crate::forecast::row_key(row);
        panel.iter().map(|r| r.fires.iter().find(|(x, _)| *x == k).map_or(0, |f| f.1) as f64).sum::<f64>() / n
    };
    let key = |r: &Row| (r.conds.clone(), r.verb.clone());
    let mut out: Vec<RowFires> = Vec::new();
    for (j, r) in rules.rows.iter().enumerate() {
        let si = prev.rows.iter().position(|p| key(p) == key(r));
        let sent = si.map_or(0.0, |_| mean(b, r));
        out.push(RowFires { sent_row: si.map(|k| k as u32), new_row: Some(j as u32), text: row_words(j, r), sent, new: mean(a, r) });
    }
    for (k, p) in prev.rows.iter().enumerate() {
        if !rules.rows.iter().any(|r| key(r) == key(p)) {
            out.push(RowFires { sent_row: Some(k as u32), new_row: None, text: row_words(k, p), sent: mean(b, p), new: 0.0 });
        }
    }
    out.retain(|f| (f.new - f.sent).abs() >= 0.05);
    let lift = |f: &RowFires| (f.new + 0.5).max(f.sent + 0.5) / (f.new + 0.5).min(f.sent + 0.5);
    out.sort_by(|x, y| lift(y).total_cmp(&lift(x)));
    out.truncate(4);
    out
}
