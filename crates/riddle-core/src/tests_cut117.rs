//! Cut 117 (docs/CUT117_NUMBERS_AND_LADDER.md): numbers that agree — the death header's hp, the
//! absence's gold ledger, the forecast's horizon, stable previews — and affixes that change the answer.
use crate::engine::{ExitTier, Game};
use crate::rules::{Row, RuleSet, Verb};
use crate::tests::{add_monster, arena, ticks};

/// §1 (blind 8cf9050 A: a King death's header `hero at 18 hp` over a trace ending `0/50`): the
/// header's hp is the hp the killing blow landed on — the trace's row before it — never the
/// blow's damage (an overkill read as the hp he had).
#[test]
fn death_moment_hp_is_the_traces_row_before_the_blow() {
    let mut overkill = 0;
    for seed in 0..6u32 {
        let mut g = arena();
        {
            let run = g.run.as_mut().unwrap();
            run.hero.hp = 3 + seed as i32 % 3;
            run.hero.armour = None.into();
        }
        for (x, y) in [(5, 5), (5, 4), (5, 6)] {
            add_monster(&mut g, "ogre", x, y);
        }
        g.set_rules_raw(RuleSet { rows: vec![Row::new(vec![], Verb::new("hold"))], name: None, route: Vec::new() }).unwrap();
        ticks(&mut g, 400);
        let run = g.run.as_ref().unwrap();
        assert_eq!(run.over, Some(ExitTier::Death));
        let id = run.id;
        g.finish_run();
        let d = g.death(id).unwrap();
        let blow = d.trace.blow.clone().expect("the killing blow");
        let hp = d.moment_hp.expect("a death's moment hp");
        // the trace's row before the killing blow: the previous blow's hp, else the last action's
        let before = if d.trace.blows.len() >= 2 { d.trace.blows[d.trace.blows.len() - 2].hp } else { d.trace.turns.last().unwrap().hp };
        assert_eq!(hp, before, "header hp {hp} vs the trace's row {before} (blow {}): {}", blow.dmg, d.morgue);
        assert!(hp >= 1 && hp <= blow.dmg, "the blow {} landed on {hp} hp", blow.dmg);
        assert!(d.moment_max_hp >= hp);
        assert!(d.morgue.contains(&format!("at {hp} hp")), "{}", d.morgue);
        if blow.dmg > hp {
            overkill += 1;
        }
    }
    assert!(overkill > 0, "an ogre's blow overkills a hero at 3–5 hp at least once");
}

/// The terms of a report's ledger sum to the purse's change, and nothing bypassed the ledger (`other` is
/// only what the tally itself filed there).
fn assert_ledger(r: &crate::wire::ReturnReport, tally0: &std::collections::BTreeMap<String, i64>, l: &crate::engine::LineageState, gold0: i32) {
    let gold = r.gold.as_ref().expect("an absence reports its gold");
    let net = gold.net.expect("the purse's change");
    assert_eq!(net, l.gold - gold0);
    let ledger = gold.ledger.as_ref().expect("the absence's ledger");
    let sum: i32 = ledger.terms.iter().map(|t| t.amount).sum();
    assert_eq!(sum, net, "the terms add up to the purse's change: {ledger:?}");
    assert_eq!((ledger.net, ledger.earned - ledger.spent), (net, net), "{ledger:?}");
    assert!(ledger.earned >= 0 && ledger.spent >= 0);
    let other = l.gold_tally.get("other").copied().unwrap_or(0) - tally0.get("other").copied().unwrap_or(0);
    let shown = ledger.terms.iter().find(|t| t.label == "other").map_or(0, |t| t.amount);
    assert_eq!(i64::from(shown), other, "no movement bypassed the ledger: {ledger:?}");
    let kept = ledger.terms.iter().filter(|t| t.label == "carried" || t.label == "lost").map(|t| t.amount).sum::<i32>();
    assert_eq!(kept, gold.home + gold.salvage + gold.passage, "carried + lost is the income kept: {gold:?}");
    if gold.lost > 0 {
        assert!(ledger.terms.iter().any(|t| t.label == "lost" && t.amount == -gold.lost));
    }
    assert!(ledger.terms.iter().all(|t| t.amount != 0));
}

/// §1 (blind 8cf9050 B: `$0 GOLD EARNED · purse −$6789 · forge −$6750 · lost $2620`): an absence's gold
/// ledger names each term and its terms sum exactly to the purse's change — the apprentice's forge apart.
#[test]
fn absence_ledger_terms_sum_to_the_purse_change() {
    let mut forged = false;
    for seed in [29u64, 3] {
        let mut g = Game::new_resident(seed);
        crate::tree::grant(&mut g.lineage, &["scout", "porter", "apprentice"]);
        g.lineage.orders.forge = "all".into();
        g.lineage.best_depth = 12;
        g.lineage.gold_move(40_000, "test income");
        let (gold0, tally0) = (g.lineage.gold, g.lineage.gold_tally.clone());
        let r = crate::offline::run_offline(&mut g, 8 * 3600);
        assert!(r.runs > 0);
        assert_ledger(&r, &tally0, &g.lineage, gold0);
        let terms = &r.gold.as_ref().unwrap().ledger.as_ref().unwrap().terms;
        forged |= terms.iter().any(|t| t.label == "apprentice" && t.amount < 0);
        assert!(terms.iter().any(|t| t.label == "carried"), "{terms:?}");
    }
    assert!(forged, "the apprentice's steps are a term of their own");
    // the session (the town's purse across bloodlines) reconciles too
    let mut s = crate::bloodlines::Session::new(5);
    crate::tree::grant(&mut s.active.lineage, &["scout"]);
    s.active.lineage.gold_move(500, "test income");
    let (gold0, tally0) = (s.active.lineage.gold, s.active.lineage.gold_tally.clone());
    let r = s.run_offline(4 * 3600);
    assert_ledger(&r, &tally0, &s.active.lineage, gold0);
}

/// §1 (blind 8cf9050 A: the 8-sample previews swing between reads): the same camp gives the same advice —
/// two consecutive reads (on one game, its memo warm, and on a fresh load of the same save) price every
/// move alike — and each move carries its noise band, an even move flagged so no delta is shown.
#[test]
fn the_same_camp_reads_the_same_advice() {
    let text = include_str!("fixtures/save_307dbed.json");
    let mut g = Game::load(text).unwrap();
    g.lineage.pkg.literal = false;
    let fresh = || {
        let mut h = Game::load(text).unwrap();
        h.lineage.pkg.literal = false;
        h
    };
    let sims = crate::forecast::PREVIEW_SIMS;
    let a = crate::packages::options(&g, sims);
    assert!(!a.is_empty(), "the camp offers moves");
    let again = crate::packages::options(&g, sims);
    let other = crate::packages::options(&fresh(), sims);
    let js = |o: &Vec<crate::packages::PkgOption>| serde_json::to_string(o).unwrap();
    assert_eq!(js(&a), js(&again), "a re-read of the same camp");
    assert_eq!(js(&a), js(&other), "a fresh load of the same camp");
    for o in &a {
        assert!(o.noise >= 0.0 && o.noise.is_finite());
        if o.better == 0 && o.worse == 0 {
            assert!(o.even, "{} plays alike: even", o.id);
        }
        if o.even {
            assert!(o.d_past.abs() <= o.noise + 1e-9, "{}: an even move's delta is within its band", o.id);
        }
    }
    // the forecast preview and its paired move, read twice
    let f1 = serde_json::to_string(&crate::forecast::forecast_estimate(&g)).unwrap();
    let f2 = serde_json::to_string(&crate::forecast::forecast_estimate(&fresh())).unwrap();
    assert_eq!(f1, f2);
}

/// §4 (blind 8cf9050 B: `Supplies limited · $0 budget` after the apprentice's `−$6750`): a limited repeat says
/// why on the report (`supply_budget`) and on the apprentice's line (`WorkerAct.reason`).
#[test]
fn a_limited_supply_budget_names_its_reason() {
    let mut seen = 0;
    for seed in [29u64, 3, 7, 11] {
        let mut g = Game::new_resident(seed);
        crate::tree::grant(&mut g.lineage, &["scout", "porter", "apprentice"]);
        g.lineage.orders.forge = "all".into();
        g.lineage.best_depth = 12;
        g.lineage.gold_move(40_000, "test income");
        let r = crate::offline::run_offline(&mut g, 8 * 3600);
        let Some(b) = r.supply_budget.as_ref() else {
            assert!(!r.restock_capped && !r.repeat_short);
            continue;
        };
        seen += 1;
        assert!(["no_income", "income_spent", "purse_short"].contains(&b.reason.as_str()), "{b:?}");
        assert_eq!(b.left, (b.income - b.spent).max(0));
        if b.reason == "no_income" {
            assert!(b.income <= 0);
        }
        if let Some(a) = r.workers.iter().find(|w| w.id == "apprentice") {
            assert_eq!(a.reason.as_deref(), Some(b.reason.as_str()), "the apprentice's line says why");
        }
    }
    // an absence that brought nothing home: the repeat is capped at $0 and the report says `no_income`
    let mut g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    g.lineage.gold_move(5_000, "test income");
    g.lineage.last_supplies = vec!["heal".into(); 3];
    g.lineage.last_supply_origins = Vec::new();
    g.lineage.supplies.clear();
    g.lineage.restock_off = false;
    g.lineage.last_wasted.clear();
    g.lineage.theft_skip.clear();
    g.offline = true;
    g.batch = Default::default();
    let bought = g.restock();
    assert!(bought.is_empty(), "nothing came home: nothing bought ({bought:?})");
    if g.batch.restock_capped {
        let facts = g.lineage.facts.clone();
        let class = g.lineage.class.name().to_string();
        let rank = g.lineage.rank;
        let r = crate::offline::report_with(&mut g, 3600, &facts, &class, rank, false, true, false);
        let b = r.supply_budget.expect("a capped repeat reports its budget");
        assert_eq!((b.income, b.reason.as_str()), (0, "no_income"), "{b:?}");
        seen += 1;
    }
    assert!(seen > 0, "a limited repeat was met");
    // a report with no limit names none; the reason rides on the apprentice's line only
    let mut r = crate::wire::ReturnReport { workers: vec![crate::tree::WorkerAct { id: "apprentice".into(), ..Default::default() }, crate::tree::WorkerAct { id: "porter".into(), ..Default::default() }], ..Default::default() };
    crate::offline::supply_reason(&mut r);
    assert!(r.workers.iter().all(|w| w.reason.is_none()));
    r.supply_budget = Some(crate::wire::SupplyBudget { income: 0, spent: 0, left: 0, reason: "no_income".into() });
    crate::offline::supply_reason(&mut r);
    assert_eq!(r.workers[0].reason.as_deref(), Some("no_income"));
    assert!(r.workers[1].reason.is_none());
}

/// §2 (client QA: the King's `TRY` alternated — cadence into the last slot, then boss focus over it, then cadence
/// again): a death's tactic fix never takes off another answer to the same killer; consecutive King deaths settle.
#[test]
fn consecutive_king_deaths_do_not_alternate_the_fix() {
    use crate::packages;
    for extra in [None, Some("kite_archers")] {
        let mut g = Game::new_resident(12);
        g.lineage.kills.insert("goblin_warlord".into());
        for id in ["cadence", "boss_focus", "kite_archers"] {
            g.lineage.pkg.owned.insert(id.to_string());
        }
        packages::recompile(&mut g.lineage);
        if let Some(t) = extra {
            packages::equip(&mut g.lineage, t, 0).unwrap();
        }
        let slots = packages::tactic_slots(&g.lineage);
        let mut taken: Vec<String> = Vec::new();
        for _ in 0..4 {
            let Some((id, v, _)) = packages::fix_pick(&g.lineage, "mirror_king") else { break };
            let before = g.lineage.pkg.tactics.clone();
            packages::take_fix(&mut g.lineage, &id, v).unwrap();
            for t in &before {
                if ["cadence", "boss_focus"].contains(&t.as_str()) {
                    assert!(g.lineage.pkg.tactics.contains(t), "taking {id} took off {t}, another answer to the King ({slots} slots)");
                }
            }
            taken.push(id);
        }
        let mut seen = taken.clone();
        seen.dedup();
        assert_eq!(seen.len(), taken.len(), "the fix does not repeat: {taken:?}");
        assert!(taken.first().map(String::as_str) == Some("cadence"), "the King's own counter first: {taken:?}");
    }
}

/// §1 (blind 8cf9050 B: `D33 88%`, then the scout banked the sends at D30/D32): the forecast carries the scout's
/// wall order — the floor it stops before and the send's ends under it — and prices the order on its own sims.
// Cut 118, owner amendment ("heroes yolo"): the scout's wall order is retired — every send pushes and the forecast
// holds nothing (`tests_cut118::the_wall_order_is_retired`); this test of the order is kept, ignored.
#[test]
#[ignore = "Cut 118: the wall order is retired"]
fn the_forecast_names_the_scouts_wall_order() {
    use crate::engine::ExitTier;
    use crate::forecast::{held_ends, SimResult};
    let sim = |max_depth: u32, tier: ExitTier, loot_kept: i32, arrive: Vec<(u32, u32, i32)>| SimResult { max_depth, tier, cause: None, loot_kept, timed_out: false, ticks: 1, loot: 0, fires: Vec::new(), oath: false, oath_progress: 0.0, oath_steps: 0, passage: 0, arrive, hunger: (0, 0) };
    // two sims past the wall on 5 (one dies on 6 having carried 40 to it, one banks on 7), one dies on 3
    let ended = vec![sim(6, ExitTier::Death, 0, vec![(1, 0, 0), (5, 10, 40), (6, 20, 60)]), sim(7, ExitTier::Bank, 90, vec![(1, 0, 0), (5, 10, 30)]), sim(3, ExitTier::Death, 0, vec![(1, 0, 0)])];
    let h = held_ends(&ended, 5, true);
    assert_eq!((h.depth, h.stop, h.order.as_str()), (5, 4, "bank"));
    assert!((h.share - 2.0 / 3.0).abs() < 1e-9);
    assert!((h.ends.bank - 2.0 / 3.0).abs() < 1e-9 && (h.ends.death - 1.0 / 3.0).abs() < 1e-9, "{h:?}");
    assert!((h.ends.gold - 70.0 / 3.0).abs() < 1e-9, "banked with the carry on arriving: {h:?}");
    let c = held_ends(&ended, 5, false);
    assert!((c.ends.death - 2.0 / 3.0).abs() < 1e-9, "a carry order goes on: {c:?}");
    assert!((c.ends.gold - (40.0 + 90.0) / 3.0).abs() < 1e-9, "the carry is kept on the death past it: {c:?}");
    // on a camp: the scout hired and two heirs dead on D3 — the forecast names the order; `push` names none
    let mut g = Game::new_resident(5);
    crate::tree::grant(&mut g.lineage, &["scout"]);
    g.lineage.best_depth = 3;
    let s = crate::tree::strength(&g.lineage);
    g.lineage.tree.wall = Some(crate::tree::WallLedger { depth: 3, deaths: 2, strength: s, held: 0 });
    g.lineage.orders.wall = "bank".into();
    let f = crate::forecast::forecast(&g);
    let hold = f.hold.expect("the scout's order rides on the forecast");
    assert_eq!((hold.depth, hold.stop, hold.order.as_str()), (3, 2, "bank"));
    let reach3 = f.depths.iter().find(|d| d.depth == 3).unwrap().reach;
    assert!((hold.share - reach3).abs() < 1e-9, "the sends that get to the stairs are those that reach the wall");
    assert!(hold.ends.bank + 1e-9 >= f.ends.as_ref().unwrap().bank);
    g.lineage.orders.wall = "push".into();
    assert!(crate::forecast::forecast(&g).hold.is_none());
}

/// §3: every affix a band boss can draw has its breakers — packages that exist, never Steady (the default), and past
/// the Warlord never the plain walls' usual best (Guarded, Bold) — and a set wearing one is read as wearing it.
#[test]
fn every_affix_has_its_breakers() {
    use crate::descent::{affix_breakers, affix_pool, BOSS_DEPTHS};
    for (boss, _) in BOSS_DEPTHS {
        for a in affix_pool(boss) {
            let b = affix_breakers(boss, *a);
            assert!(!b.is_empty(), "{boss} {a:?}");
            for id in b {
                assert!(crate::packages::def(id).is_some(), "{id}");
                assert_ne!(*id, "steady");
                if boss != "goblin_warlord" {
                    assert!(!["guarded", "bold"].contains(id), "{boss} {a:?}: {id}");
                }
            }
        }
    }
    let mut g = Game::new_resident(12);
    g.lineage.kills.insert("goblin_warlord".into());
    g.lineage.pkg.owned.insert("boss_focus".into());
    crate::packages::equip(&mut g.lineage, "boss_focus", 0).unwrap();
    assert!(crate::packages::wears(g.lineage.rules(), "boss_focus"));
    assert!(crate::packages::wears(g.lineage.rules(), "steady"));
    assert!(!crate::packages::wears(g.lineage.rules(), "hunter"));
}
