//! Cut 114 §3 (blind 77030eb): the scout's order at a wall, and a floor the guard gave up carries the stall's record.
use crate::engine::ExitTier;
use crate::tree::{self, WallLedger, WALL_DEATHS, WALL_RETRY};
use crate::Game;

/// A resident lineage with the scout at work and a record at `best`, a run started (turn 0).
fn walled(seed: u64, best: u32) -> Game {
    let mut g = Game::new_resident(seed);
    tree::grant(&mut g.lineage, &["scout"]);
    g.lineage.best_depth = best;
    g.start_run(None);
    g
}

fn die_at(g: &mut Game, depth: u32) {
    let mut run = g.run.clone().unwrap();
    run.depth = depth;
    run.max_depth = depth;
    tree::note_wall(g, &run, ExitTier::Death);
}

// Cut 118, owner amendment ("heroes yolo"): the scout's wall order is retired — every send pushes and the forecast
// holds nothing (`tests_cut118::the_wall_order_is_retired`); this test of the order is kept, ignored.
#[test]
#[ignore = "Cut 118: the wall order is retired"]
fn deaths_at_one_floor_near_the_record_make_a_wall_and_a_send_past_it_forgets_it() {
    let mut g = walled(3, 28);
    die_at(&mut g, 20);
    assert!(g.lineage.tree.wall.is_none(), "a death on the walk is no wall's");
    die_at(&mut g, 28);
    assert_eq!(tree::wall_hold(&g.lineage), None, "one death is not yet a wall");
    die_at(&mut g, 28);
    assert_eq!(g.lineage.tree.wall.as_ref().map(|w| (w.depth, w.deaths)), Some((28, WALL_DEATHS)));
    assert_eq!(tree::wall_hold(&g.lineage), Some((28, true)), "the default order banks before it");
    // the order is the player's: `carry` carries the haul home and lets him on; `push` sends him on
    g.lineage.orders.wall = "carry".into();
    assert_eq!(tree::wall_hold(&g.lineage), Some((28, false)));
    g.lineage.orders.wall = "push".into();
    assert_eq!(tree::wall_hold(&g.lineage), None);
    g.lineage.orders.wall = "bank".into();
    // a paused scout carries no order
    g.set_worker("scout", false).unwrap();
    assert_eq!(tree::wall_hold(&g.lineage), None);
    g.set_worker("scout", true).unwrap();
    // a send past the wall forgets it
    let mut run = g.run.clone().unwrap();
    run.max_depth = 29;
    tree::note_wall(&mut g, &run, ExitTier::Bank);
    assert!(g.lineage.tree.wall.is_none());
}

// Cut 118, owner amendment ("heroes yolo"): the scout's wall order is retired — every send pushes and the forecast
// holds nothing (`tests_cut118::the_wall_order_is_retired`); this test of the order is kept, ignored.
#[test]
#[ignore = "Cut 118: the wall order is retired"]
fn the_bank_order_tries_the_wall_again_after_retries_or_a_stronger_hero() {
    let mut g = walled(4, 28);
    let s = tree::strength(&g.lineage);
    g.lineage.tree.wall = Some(WallLedger { depth: 28, deaths: 2, strength: s, held: WALL_RETRY - 1 });
    assert_eq!(tree::wall_hold(&g.lineage), Some((28, true)));
    g.lineage.tree.wall.as_mut().unwrap().held = WALL_RETRY;
    assert_eq!(tree::wall_hold(&g.lineage), Some((28, false)), "the retry tries it, its haul carried");
    g.lineage.tree.wall.as_mut().unwrap().held = 0;
    g.lineage.tree.wall.as_mut().unwrap().strength = s ^ 1;
    assert_eq!(tree::wall_hold(&g.lineage), Some((28, false)), "a stronger hero tries it");
    // a start at or under the wall never meets its stairs
    g.lineage.tree.wall.as_mut().unwrap().strength = s;
    g.lineage.start = 28;
    assert_eq!(tree::wall_hold(&g.lineage), None);
}

#[test]
fn a_held_send_banks_at_the_stairs_to_the_wall_and_the_report_names_it() {
    let mut g = walled(5, 12);
    let acts0 = g.lineage.tree.acts.clone();
    tree::scout_sent(&mut g);
    g.run.as_mut().unwrap().wall_hold = Some((6, true));
    g.run.as_mut().unwrap().loot = 40;
    g.descend_to(6);
    let run = g.run.clone().unwrap();
    assert_eq!(run.over, Some(ExitTier::Bank), "banked at the stairs");
    assert_eq!(run.depth, 5, "never set foot on the wall's floor");
    assert!(run.wall_held);
    let reason = crate::engine::exit_reason(&run, ExitTier::Bank, 12, g.lineage.rules());
    assert_eq!(reason, "before D6 · banked");
    let gold = g.lineage.gold;
    g.finish_run();
    assert!(g.lineage.gold >= gold + 40, "the bank keeps the whole carry");
    assert_eq!(g.lineage.tree.acts.get(tree::SCOUT_HELD), Some(&1));
    let acts = tree::report_acts(&g.lineage, &acts0, &g.lineage.tree.acts);
    let scout = acts.iter().find(|a| a.id == "scout").expect("the scout's line");
    assert_eq!(scout.items, vec!["banked before D6 ×1".to_string()]);
    assert!(scout.first, "the order's first act is announced");
}

#[test]
fn a_carried_send_secures_the_haul_at_the_stairs_and_goes_on() {
    let mut g = walled(6, 12);
    g.run.as_mut().unwrap().wall_hold = Some((6, false));
    g.run.as_mut().unwrap().loot = 55;
    let secured = g.run.as_ref().unwrap().secured;
    g.descend_to(6);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.over, None, "the heir goes on");
    assert_eq!(run.depth, 6);
    assert_eq!(run.secured, secured + 55, "the haul is secured, safe whatever the exit");
    assert_eq!(run.wall_carried, 55);
}

// Cut 118, owner amendment ("heroes yolo"): the scout's wall order is retired — every send pushes and the forecast
// holds nothing (`tests_cut118::the_wall_order_is_retired`); this test of the order is kept, ignored.
#[test]
#[ignore = "Cut 118: the wall order is retired"]
fn the_scouts_send_carries_his_order_and_a_send_by_hand_does_not() {
    let mut g = walled(7, 28);
    g.finish_run();
    g.lineage.rest_left = 0;
    let s = tree::strength(&g.lineage);
    g.lineage.tree.wall = Some(WallLedger { depth: 28, deaths: 2, strength: s, held: 0 });
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().wall_hold, None, "a send by hand pushes");
    tree::scout_sent(&mut g);
    assert_eq!(g.run.as_ref().unwrap().wall_hold, Some((28, true)));
}

#[test]
fn the_wall_order_is_a_standing_order_a_client_without_it_leaves_standing() {
    let mut g = Game::new_resident(8);
    let mut o = g.lineage.standing_orders();
    assert_eq!(o.wall.as_deref(), Some("bank"), "the default");
    o.wall = Some("push".into());
    g.set_orders(&o).unwrap();
    assert_eq!(g.lineage.orders.wall, "push");
    let mut old = g.lineage.standing_orders();
    old.wall = None;
    g.set_orders(&old).unwrap();
    assert_eq!(g.lineage.orders.wall, "push", "an order set without it stands");
    let mut bad = g.lineage.standing_orders();
    bad.wall = Some("dig".into());
    assert!(g.set_orders(&bad).is_err());
    // a save from before the order reads the default
    let raw: crate::wire::StandingSwitches = serde_json::from_str(r#"{"insure":true,"forge":"half"}"#).unwrap();
    assert_eq!(raw.wall, "bank");
}

#[test]
fn a_floor_the_guard_gave_up_carries_the_stalls_record() {
    let mut g = walled(9, 12);
    for _ in 0..200 {
        if g.run.as_ref().is_some_and(|r| r.over.is_some() || !r.trace.is_empty()) {
            break;
        }
        g.tick();
    }
    {
        let run = g.run.as_mut().unwrap();
        assert!(run.over.is_none() && !run.trace.is_empty());
        run.stuck_fires = 2;
        run.stuck_cause = Some("goblin, no path".into());
        run.bail = true;
    }
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    let run = g.run.clone().unwrap();
    assert!(crate::engine::gave_up(&run, ExitTier::Return));
    assert_eq!(crate::engine::exit_reason(&run, ExitTier::Return, 12, g.lineage.rules()), "stuck · went home");
    let id = run.id;
    g.finish_run();
    let rec = g.deaths.get(&id).expect("a record");
    assert!(rec.stall, "the stall's record");
    assert_eq!(rec.death.verdict, "stall");
    assert!(rec.death.cause.starts_with("stalled · goblin"), "{}", rec.death.cause);
    assert_eq!(rec.death.margin, "went home");
    assert!(!rec.death.trace.turns.is_empty(), "with its trace");
    assert_eq!(g.lineage.run_log.last().and_then(|r| r.death_id), Some(id), "the runs log links it");
    assert!(g.last_exit.as_ref().is_some_and(|l| !l.text.starts_with("stalled")), "it still went home with its share");
}
