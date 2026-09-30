//! Cut 27 §4 (the `pack ↔ pick up` loop, the gem) and §5 (the seams): tests.
use crate::engine::Game;
use crate::rules::{Cond, Row, RuleSet, Verb};
use crate::tests::{add_monster, arena, give, rules, ticks};
use crate::wire::*;

/// AS's set at the D5 stall (cohort 22; `eval/cards/420f27c.raterAS-stall.rules.json`): the
/// `pack break` card at R8.
fn as_stall_set() -> RuleSet {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards/420f27c.raterAS-stall.rules.json")).expect("AS's stall set");
    RuleSet::parse(&text).expect("parses")
}

/// A lineage owning everything, a levelled fighter, `set` — one send from D5 to its end (sims):
/// the run's card ↔ chore loops, whether it stalled, and its loops' causes.
fn send_from_d5(set: &RuleSet, seed: u64) -> (u32, bool, Vec<String>) {
    let mut g = Game::new(seed);
    for u in crate::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    crate::probes::learn_everything(&mut g);
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 4, xp: 0, next: 0 });
    g.set_rules_raw(set.clone()).unwrap();
    g.sim = true;
    g.start_run(None);
    g.descend_to(5);
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 60_000 {
        g.tick();
        g.events.clear();
        n += 1;
    }
    let r = g.run.as_ref().unwrap();
    (r.card_loops, r.timed_out, r.loop_causes.clone())
}

/// §4: the seeds whose sends looped `R8 pack ↔ pick up` / `explore` / `descend` (the card
/// falling back to a corridor from a pack it had given up on, or one idling out of reach, and a
/// chore stepping out again) and `R5 free ↔ descend` (the row walking to another neutral than
/// the captive it saw) before the fix: no card ↔ chore loop, no stall.
#[test]
fn as_stall_set_no_longer_loops_the_card_with_a_chore() {
    let set = as_stall_set();
    for seed in [325u64, 2554, 1127, 1523, 2837] {
        let (loops, stalled, causes) = send_from_d5(&set, seed);
        assert_eq!(loops, 0, "seed {seed}: {causes:?}");
        assert!(!causes.iter().any(|c| c.starts_with("R8 pack ↔") || c.starts_with("R5 free ↔")), "seed {seed}: {causes:?}");
        assert!(!stalled || !causes.iter().any(|c| c.contains('↔')), "seed {seed}: stalled on a loop {causes:?}");
    }
}

/// §4: the `pack break` card falls back to a corridor only from a pack within `PACK_NEAR`, and
/// never twice from the same tile.
#[test]
fn pack_break_does_not_fall_back_from_a_far_pack() {
    let mut g = arena();
    g.lineage.unlocks.insert("pack_break".into());
    // two jackals idling at the far end of the room (9+ tiles), asleep: not closing
    for (x, y) in [(14, 2), (14, 3)] {
        let id = add_monster(&mut g, "jackal", x, y);
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.awake = false;
    }
    rules(&mut g, vec![Row::new(vec![], Verb::arg("tactic", "pack_break"))]);
    let before = g.run.as_ref().unwrap().hero.pos;
    ticks(&mut g, 12);
    let run = g.run.as_ref().unwrap();
    assert!(run.card_fell.is_none(), "fell back from a pack 10 tiles off (hero {before:?} → {:?})", run.hero.pos);
}

/// §4: the gem is the best whole-run patch that does not harm (an exit's reach loss counts), and
/// it leads.
#[test]
fn the_gem_is_the_best_whole_run_patch_and_leads() {
    let mk = |v: &str, whole: PatchWhole| Patch { no_gain: false, row: Row::new(vec![Cond::n("hp<", 20)], Verb { v: v.into(), a: None }), insert_at: 0, survive: 1.0, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 6, forecast_pm: 0.1, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None, whole: Some(whole), gem: false, restores: None };
    let w = |reach: f64, death: f64| PatchWhole { reach, reach_pm: 0.05, death, death_pm: 0.05, ..Default::default() };
    // AS: `cut R8` (reach −4 ± 5) under an exit that costs the reach (−90) and a small gain.
    let mut ps = vec![mk("return", w(-0.9, -0.1)), mk("retreat", w(0.02, 0.0)), mk("rest", w(-0.04, 0.0)), mk("descend", w(0.3, 0.2))];
    crate::trace::pick_gem(&mut ps);
    assert!(ps[0].gem && ps[0].row.verb.v == "retreat", "{:?}", ps.iter().map(|p| (&p.row.verb.v, p.gem)).collect::<Vec<_>>());
    assert_eq!(ps.iter().filter(|p| p.gem).count(), 1);
    // none may be: no gem
    let mut ps = vec![mk("return", w(-0.9, -0.1)), mk("descend", w(0.3, 0.2))];
    crate::trace::pick_gem(&mut ps);
    assert!(ps.iter().all(|p| !p.gem));
}

/// §5: a drive-off whose counter the set already holds is `order` — the held row and the row
/// above it that acted most.
#[test]
fn a_drive_off_with_its_counter_in_the_set_is_order() {
    let counter = crate::facts::counter_row("goblin_warlord");
    let set = RuleSet {
        rows: vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::new("bank")),
            Row::new(vec![Cond::t("foe_tag", "ranged")], Verb::arg("attack", "tag:ranged")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::new("back_corridor")),
            counter.clone(),
        ],
        name: None,
        route: Vec::new(),
    };
    let turn = |row: i32| TraceTurn { max_hp: 0, t: 0, row, verb: Verb::new("attack"), hp: 10, foes: 3, rule_foes: 3, telegraphs: Vec::new(), blocked: None, rows: None, blows: Vec::new(), gift: None };
    let trace = vec![turn(1), turn(2), turn(1), turn(1), turn(-2)];
    assert_eq!(crate::trace::driven_order(&set, &counter, &trace), (Some(3), Some(1)));
    let without = RuleSet { rows: set.rows[..3].to_vec(), ..set.clone() };
    assert_eq!(crate::trace::driven_order(&without, &counter, &trace), (None, None));
}

/// §5: the sends remember the last distinct sets; a row taken out since is a death's `restore`
/// candidate, at its old index.
#[test]
fn a_row_taken_out_since_the_last_send_is_remembered() {
    let gas = Row::new(vec![Cond::t("foe_tag", "gas")], Verb::new("retreat"));
    let a = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), gas.clone(), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None, route: Vec::new() };
    let b = RuleSet { rows: vec![a.rows[0].clone(), a.rows[2].clone()], ..a.clone() };
    let mut g = Game::new(3);
    g.set_rules_raw(a.clone()).unwrap();
    g.start_run(None);
    g.run = None;
    g.set_rules_raw(b.clone()).unwrap();
    g.start_run(None);
    assert_eq!(g.lineage.sent_sets.len(), 2);
    assert_eq!(crate::trace::removed_rows(&g.lineage.sent_sets, &b), vec![(gas, 1)]);
    // the same set sent again is not a new entry
    g.run = None;
    g.start_run(None);
    assert_eq!(g.lineage.sent_sets.len(), 2);
}

/// §5 (AS: `R1 fired 0 of 8 runs: foe: boss → drink strength` beside a trace's `drunk
/// strength`): the chore never drinks a strength potion a row of the set names.
#[test]
fn the_chore_leaves_a_row_named_boost_to_its_row() {
    let mut g = arena();
    let f = crate::item::ident_fact(&g.lineage.flavours, "strength").unwrap();
    g.lineage.facts.insert(f);
    give(&mut g, "strength");
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("drink", "strength"))]);
    let evs = ticks(&mut g, 20);
    assert!(g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "strength"), "{evs:?}");
    // without the row the chore drinks it
    let mut g = arena();
    let f = crate::item::ident_fact(&g.lineage.flavours, "strength").unwrap();
    g.lineage.facts.insert(f);
    give(&mut g, "strength");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    ticks(&mut g, 20);
    assert!(!g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "strength"));
}

/// §5 (AT: `Krak the jackal, gone wild` — "is that my hatched jackal?"): the stray's line says
/// how it was lost.
#[test]
fn a_stray_line_says_how_it_was_lost() {
    let mut g = arena();
    g.lineage.lost.push(crate::engine::Lost { kind: "jackal".into(), name: "Uleth".into(), gen: 2, heir: 1, why: "fell D7 to ogre".into() });
    let id = add_monster(&mut g, "jackal", 6, 5);
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.stray = true;
        m.name = Some("Uleth".into());
        m.awake = false;
    }
    g.lineage.unlocks.insert("tame".into());
    g.lineage.facts.insert("item:leash".into());
    give(&mut g, "leash");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tame", "nearest"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text == "Uleth, gone wild: fell D7 to ogre.")), "{evs:?}");
}
