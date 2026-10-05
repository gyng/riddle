//! Cut 27 §1–2 (core): the fold, the waystone passage, the divergence.
use crate::engine::{Game, HERO_ID};
use crate::rules::RuleSet;
use crate::wire::*;

/// A lineage a player has after a few hours on the preset (the metrics' EDITED bot): the common
/// items known, eight rows, the good set, D`best` reached.
fn edited(seed: u64, best: u32) -> Game {
    let mut g = Game::new_literal(seed);
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink"] {
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss"] {
        g.lineage.facts.insert(f.into());
    }
    g.set_rules(crate::probes::good()).expect("good rules");
    g.lineage.best_depth = best;
    g
}

/// What the send's repeat re-bought (the ledger's `repeat` lines).
fn supplies_bought(g: &Game) -> i32 {
    -g.lineage.gold_ledger.iter().filter(|l| l.why.starts_with("repeat")).map(|l| l.delta).sum::<i32>()
}

/// The first seed (from `from`) whose lineage folds at least D1–2 for the good set.
fn folding(from: u64) -> Game {
    for seed in from..from + 40 {
        let g = edited(seed, 6);
        if g.forecast().fold_to.is_some_and(|t| t >= 2) {
            return g;
        }
    }
    panic!("no seed folds D1–2 for the good set");
}

/// Cut 27 §1: a floor's clear is the share of the sims on it that reached the next; the fold is
/// the run of floors from the start at ≥ 95 %, never past the best depth.
#[test]
fn the_forecast_folds_the_floors_it_clears() {
    let g = folding(1);
    let f = g.forecast();
    let to = f.fold_to.unwrap();
    assert!(to <= g.lineage.best_depth, "the fold never passes the best depth ({to})");
    for d in f.start..=to {
        let c = f.depths.iter().find(|x| x.depth == d).and_then(|x| x.clear).unwrap();
        assert!(c >= crate::forecast::FOLD_CLEAR - 1e-9, "D{d} folded at {c}");
    }
    if let Some(next) = f.depths.iter().find(|x| x.depth == to + 1) {
        if to < g.lineage.best_depth {
            assert!(next.clear.is_none_or(|c| c < crate::forecast::FOLD_CLEAR), "D{} clears {:?} but was not folded", to + 1, next.clear);
        }
    }
    // the clear matches the reach it is drawn from: clear(d) = reach(d+1) / reach(d)
    for w in f.depths.windows(2) {
        if let (Some(c), true) = (w[0].clear, w[0].reach > 0.0) {
            assert!((c - w[1].reach / w[0].reach).abs() < 1e-9, "D{} clear {c} vs reach {} / {}", w[0].depth, w[1].reach, w[0].reach);
        }
    }
}

/// Cut 27 §1: the fold plays the same run the steps would (economy and determinism unchanged),
/// opens the watch on the first unfolded floor, and carries every state change of its floors.
#[test]
fn the_fold_is_the_same_run_and_lists_every_state_change() {
    let g = folding(1);
    let save = g.save();
    let mut a = Game::load(&save).unwrap();
    let _ = a.forecast();
    let _ = a.send();
    let line = a.fold();
    assert!(line.to >= line.from, "a fold played: {line:?}");
    assert_eq!(line.floors.len() as u32, line.to - line.from + 1);
    let snap = &line.step.snapshot;
    assert!(line.step.run_over || snap.depth == line.to + 1, "the watch opens below the fold (D{} after D{})", snap.depth, line.to);
    // the same send stepped instead: the same events, tick for tick
    let mut b = Game::load(&save).unwrap();
    let _ = b.forecast();
    let _ = b.send();
    let mut stepped: Vec<Ev> = Vec::new();
    while stepped.len() < line.step.events.len() {
        let r = b.step(1);
        stepped.extend(r.events);
        if r.run_over {
            break;
        }
    }
    assert!(stepped.len() >= line.step.events.len(), "the stepped run ended sooner");
    assert_eq!(&stepped[..line.step.events.len()], &line.step.events[..], "the fold played a different run");
    // every state change on a folded floor is a beat of that floor (a hurt is its floor's dip)
    for f in &line.floors {
        for e in &f.events {
            if crate::fold::state_change(e) {
                assert!(f.beats.iter().any(|b| b.t == e.t() && b.kind != "dip"), "D{}: {e:?} has no beat in {:?}", f.depth, f.beats);
            }
            if let Ev::Hurt { id, hp, .. } = e {
                if *id == HERO_ID && (*hp as f64) <= crate::fold::DIP_SHARE * f.snapshot.hero.entity.max_hp as f64 {
                    assert!(f.beats.iter().any(|b| b.kind == "dip"), "D{}: hp {hp} with no dip", f.depth);
                }
            }
        }
        assert!(f.snapshot.depth == f.depth, "a floor's snapshot is its own");
    }
    for c in &line.chips {
        assert!(crate::rules::word_count(c) <= 3, "chip {c:?} over 3 words");
    }
    assert_eq!(line.beats.len(), line.floors.iter().map(|f| f.beats.len()).sum::<usize>());
    // nothing to fold: no tick
    let mut c = Game::load(&save).unwrap();
    let _ = c.send();
    c.fold_plan = None;
    let t0 = c.run.as_ref().unwrap().turn;
    let empty = c.fold();
    assert!(empty.to < empty.from && empty.floors.is_empty() && c.run.as_ref().unwrap().turn == t0);
}

/// Cut 27 §1: a waystone start the set clears is paid the floors above it (`+$N passage`, into
/// the purse at the send); a set that does not clear them is paid nothing.
#[test]
fn a_waystone_start_pays_the_passage_when_the_set_clears_the_floors_above() {
    let mut paid = 0;
    for seed in 1..30u64 {
        let mut g = edited(seed, 6);
        g.lineage.waystones = vec![5];
        g.set_start(5).unwrap();
        let rules = g.lineage.rules().clone();
        let Some((at, coins)) = crate::forecast::sim_passage(&g, &rules) else { panic!("a D5 start prices a passage") };
        assert_eq!(at, 5);
        let opts = g.start_forecast();
        let gold = g.lineage.gold;
        let snap = g.send();
        assert_eq!(snap.depth, 5);
        assert_eq!(snap.run.passage, coins.max(0));
        assert_eq!(g.lineage.gold - gold, coins.max(0) - supplies_bought(&g), "the passage is paid into the purse at the send");
        assert_eq!(g.lineage.gold_ledger.iter().any(|l| l.why == "passage D5"), coins > 0);
        if coins > 0 {
            paid += 1;
            assert!(g.events.iter().any(|e| matches!(e, Ev::Callout { text, .. } if *text == format!("passage +${coins}"))));
            // the start sheet said so (its gold counts it)
            assert!(opts.iter().any(|o| o.start == 5 && o.passage == coins), "{opts:?}");
            break;
        }
    }
    assert!(paid > 0, "no lineage cleared D1–4 at ≥ 95 %");
    // a set that clears nothing (no rows) is paid nothing
    let mut g = edited(3, 6);
    g.lineage.waystones = vec![5];
    g.set_start(5).unwrap();
    g.set_rules(RuleSet::default()).unwrap();
    assert_eq!(crate::forecast::passage_for(&g, &RuleSet::default(), 5), 0);
    // from D1: none
    let g = edited(3, 6);
    assert_eq!(crate::forecast::sim_passage(&g, &g.lineage.rules().clone()), None);
}

/// Cut 27 §2: an edit that moves the forecast has a scene — a seed, the tick the sets part, both
/// branches' seconds and ends; the same call answers the same; the unedited set has none.
#[test]
fn an_edit_that_moves_the_forecast_has_a_divergence() {
    let mut g = edited(5, 6);
    let sent = g.lineage.rules().clone();
    let _ = g.forecast();
    // cut the heal row: the sets part at the first drink
    let mut edit = sent.clone();
    let i = edit.rows.iter().position(|r| r.verb.v == "drink").expect("the good set drinks");
    edit.rows.remove(i);
    g.set_rules(edit.clone()).unwrap();
    let _ = g.forecast();
    let vs = g.forecast_vs(&sent);
    let d = g.divergence(&sent).expect("a scene for cutting the heal row");
    assert!(d.moved > 0.0 && (d.moved - vs.death.delta.abs()).abs() < 1.0);
    assert!(d.seed < d.sims);
    assert!(d.sent.snapshot.turn <= d.tick && d.new.snapshot.turn <= d.tick, "the window opens before the divergence");
    assert!(d.sent.events.iter().chain(&d.new.events).all(|e| e.t() >= d.sent.snapshot.turn.min(d.new.snapshot.turn)));
    assert!(d.sent.events != d.new.events, "the branches differ");
    let at = |b: &DivergenceBranch| b.events.iter().filter(|e| e.t() == d.tick).cloned().collect::<Vec<_>>();
    assert!(d.sent_row.is_some() || d.new_row.is_some(), "a row fired where they part: t{} {} / {} · {:?} vs {:?}", d.tick, d.sent.text, d.new.text, at(&d.sent), at(&d.new));
    if let Some(r) = d.sent_row {
        assert!((r as usize) < sent.rows.len());
    }
    assert!(!d.fires.is_empty() && d.fires.iter().any(|f| f.sent_row == Some(i as u32) && f.new_row.is_none()), "the cut row's fires: {:?}", d.fires);
    assert_eq!(g.divergence(&sent), Some(d), "deterministic");
    // the set against itself: no scene
    assert_eq!(g.divergence(&edit), None);
}


fn estimate_camp() -> Game {
    let mut g = edited(11, 18);
    g.lineage.light_waystones(18);
    g.lineage.start = 14;
    g.lineage.gold_move(100_000, "estimate test");
    crate::kit::buy_all(&mut g.lineage);
    assert_eq!(crate::forecast::sim_start(&g), 14);
    g
}

#[test]
fn passage_estimate_caches_cannot_change_regular_panels_or_real_sends() {
    use crate::forecast::{camp_panel, camp_panel_outcomes, measure_work, passage_for};
    static WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    crate::forecast::with_sim_width(&WIDTH, || {
        let control = estimate_camp();
        let rules = control.lineage.rules().clone();
        let normal = camp_panel(&control, &rules, 8);
        let quote = passage_for(&control, &rules, 14);
        for estimate_first in [true, false] {
            let mut g = estimate_camp();
            let save = g.save();
            if !estimate_first { assert_eq!(camp_panel(&g, &rules, 8), normal); }
            let small = camp_panel_outcomes(&g, &rules, 8);
            let (again, work) = measure_work(|| camp_panel_outcomes(&g, &rules, 8));
            assert_eq!(small, again);
            assert_eq!(work.simulations, 0, "estimate memo must avoid rerunning passage");
            assert_eq!(camp_panel(&g, &rules, 8), normal, "regular panels cannot reuse estimated gold");
            assert_eq!(passage_for(&g, &rules, 14), quote);
            assert_eq!(g.save(), save, "predictions do not mutate game state");
            let mut actual = estimate_camp();
            g.start_run(Some(42));
            actual.start_run(Some(42));
            assert_eq!(g.save(), actual.save(), "actual sends cannot read approximate quote");
        }
    });
}

#[test]
fn passage_estimate_skips_unused_gold_and_matches_parallel_order() {
    use crate::forecast::{camp_panel, camp_panel_outcomes, measure_work};
    static ONE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    static TWO: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(2);
    let sequential = crate::forecast::with_sim_width(&ONE, || {
        let g = estimate_camp();
        let rules = g.lineage.rules();
        let (outcomes, small) = measure_work(|| camp_panel_outcomes(&g, rules, 8));
        assert!(g.forecast_cache.borrow().is_empty(), "outcome forecast must not quote gold");
        let (normal, full) = measure_work(|| camp_panel(&g, rules, 8));
        assert!(small.simulations < full.simulations, "fixture must exercise skipped passage work");
        for (a, b) in outcomes.iter().zip(&normal) {
            assert_eq!((a.max_depth, a.tier, &a.cause, a.ticks, &a.fires), (b.max_depth, b.tier, &b.cause, b.ticks, &b.fires));
        }
        outcomes
    });
    let parallel = crate::forecast::with_sim_width(&TWO, || {
        let g = estimate_camp();
        camp_panel_outcomes(&g, g.lineage.rules(), 8)
    });
    assert_eq!(sequential, parallel);
}
