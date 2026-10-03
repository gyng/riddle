//! RUNS_UI (docs/RUNS_UI.md): the runs log, the absences it folds by, the open app's clock
//! (`advance`) and the replay of a held run from its send.
use crate::engine::{events_hash, RUN_LOG_CAP};
use crate::offline::run_offline_quick;
use crate::tree;
use crate::wire::{Ev, RunRec};
use crate::Game;
use std::collections::BTreeMap;

fn with_scout(seed: u64) -> Game {
    let mut g = Game::new(seed);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    g
}

fn upto_exit(evs: &[Ev]) -> &[Ev] {
    match evs.iter().position(|e| matches!(e, Ev::Exit { .. })) {
        Some(k) => &evs[..=k],
        None => evs,
    }
}

fn replay_events(g: &Game, id: u32) -> Vec<Ev> {
    g.replay(id).expect("a capsule").floors.into_iter().flat_map(|f| f.events).collect()
}

/// A watched run: send, step to its end (bail after `bail_at` steps when given; a cage's pick taken).
fn watched(g: &mut Game, bail_at: Option<u32>) -> (u32, Vec<Ev>) {
    let id = g.send().run.id;
    let mut evs = Vec::new();
    for i in 0..4000 {
        if Some(i) == bail_at {
            g.bail();
        }
        let r = g.step(25);
        if let Some(vc) = r.snapshot.vault_choice.as_ref().filter(|_| !r.run_over) {
            if let Some(it) = vc.items.last() {
                let _ = g.choose(it.id);
            }
        }
        evs.extend(r.events);
        if r.run_over {
            break;
        }
    }
    (id, evs)
}

#[test]
fn a_watched_run_replays_to_the_same_events() {
    let mut chose = false;
    for seed in [11u64, 12, 13] {
        let mut g = with_scout(seed);
        g.tap = Some(BTreeMap::new());
        for _ in 0..3 {
            let (id, evs) = watched(&mut g, None);
            let rep = g.replay(id).unwrap();
            let again = replay_events(&g, id);
            assert_eq!(events_hash(upto_exit(&evs)), events_hash(upto_exit(&again)), "seed {seed} run {id}: the stream as watched");
            let tapped = &g.tap.as_ref().unwrap()[&id];
            if rep.hash != events_hash(tapped) {
                let k = tapped.iter().zip(&again).position(|(a, b)| a != b);
                panic!("seed {seed} run {id}: {} vs {} events, first diff {:?}: {:?} / {:?}", tapped.len(), again.len(), k, k.map(|k| &tapped[k]), k.map(|k| &again[k]));
            }
            assert!(rep.floors.len() as u32 >= g.lineage.run_log.last().unwrap().depth + 1 - g.lineage.run_log.last().unwrap().start, "a floor a depth");
            assert!(rep.ticks > 0);
            chose |= g.capsules.0.iter().any(|c| !c.inputs.is_empty());
            assert_eq!(g.lineage.run_log.last().unwrap().via, "watched");
            g.auto_keep();
        }
    }
    assert!(chose, "a cage pick was replayed");
}

#[test]
fn a_bail_replays_at_its_tick() {
    let mut g = with_scout(21);
    let (id, evs) = watched(&mut g, Some(6));
    let cap = g.capsules.0.iter().find(|c| c.id == id).unwrap();
    assert!(cap.inputs.iter().any(|(_, i)| *i == crate::engine::Input::Bail), "the bail is on the capsule");
    assert_eq!(events_hash(upto_exit(&evs)), events_hash(upto_exit(&replay_events(&g, id))));
    // the replay without the input differs (the bail is load-bearing)
    let mut h = g.clone();
    h.capsules.0.iter_mut().for_each(|c| c.inputs.clear());
    assert_ne!(events_hash(upto_exit(&replay_events(&h, id))), events_hash(upto_exit(&evs)));
}

#[test]
fn an_absences_runs_replay_to_their_tapped_events() {
    let mut g = with_scout(3);
    g.tap = Some(BTreeMap::new());
    run_offline_quick(&mut g, 1800);
    run_offline_quick(&mut g, 1800);
    let tap = g.tap.take().unwrap();
    let ids: Vec<u32> = g.lineage.run_log.iter().filter(|r| r.sampled.is_none()).map(|r| r.id).collect();
    assert!(ids.len() >= 2, "an hour runs more than one");
    for id in &ids {
        assert_eq!(g.replay(*id).unwrap().hash, events_hash(&tap[id]), "run {id}");
    }
    assert!(g.lineage.run_log.iter().all(|r| r.via == "away" && r.absence == Some(1)), "two slices, one absence");
    // the clock stamps rise inside the absence
    let c: Vec<u64> = g.lineage.run_log.iter().map(|r| r.clock_s).collect();
    assert!(c.windows(2).all(|w| w[0] <= w[1]) && *c.last().unwrap() > 0);
}

#[test]
fn advance_runs_the_open_apps_clock() {
    let mut g = with_scout(7);
    let mut live = None;
    let mut ended = Vec::new();
    let mut lives = 0;
    for _ in 0..2000 {
        let a = g.advance(2000);
        if let Some(l) = a.live.as_ref() {
            assert!(l.hp > 0 && l.max_hp >= l.hp && l.depth >= 1);
            if let Some(p) = live.as_ref().map(|p: &crate::wire::LiveRun| p.run_id) {
                if p == l.run_id {
                    lives += 1;
                }
            }
        }
        live = a.live;
        ended.extend(a.ended);
        if ended.len() >= 2 {
            break;
        }
    }
    assert!(lives > 3, "the run stays in flight between calls");
    assert!(ended.len() >= 2, "runs end, rest, and the next goes");
    let rec = g.lineage.run_log.iter().find(|r| r.id == ended[0]).unwrap();
    assert_eq!(rec.via, "town");
    assert_eq!(rec.absence, None);
    assert!(g.lineage().runs.len() >= 2);
    assert!(g.replay(ended[0]).is_some(), "a town run replays");
    // determinism: the same sequence on the same seed lands the same save
    let play = |steps: &[u64]| {
        let mut g = with_scout(9);
        for s in steps {
            g.advance(*s);
        }
        crate::save::save(&g)
    };
    let steps: Vec<u64> = (0..400).map(|i| 700 + (i * 37) % 2900).collect();
    assert_eq!(play(&steps), play(&steps));
    // the clock moves by the whole budget
    let mut h = with_scout(9);
    let c0 = h.lineage.clock_s;
    for _ in 0..30 {
        h.advance(1000);
    }
    assert_eq!(h.lineage.clock_s - c0, 30);
}

#[test]
fn before_the_scout_advance_waits_but_a_send_by_hand_runs_on() {
    let mut g = Game::new(5);
    let a = g.advance(600_000);
    assert!(a.ended.is_empty() && a.live.is_none());
    assert!(g.lineage().tree.unwrap().waits, "the hero waits at home");
    // a send by hand left mid-watch goes on unwatched to its end, then he waits again
    g.send();
    g.step(30);
    let a = g.advance(2000);
    assert!(a.live.is_some(), "in flight");
    let mut ended = Vec::new();
    for _ in 0..3000 {
        ended.extend(g.advance(2000).ended);
        if !ended.is_empty() {
            break;
        }
    }
    assert_eq!(ended.len(), 1);
    assert_eq!(g.lineage.run_log.last().unwrap().via, "town");
    let a = g.advance(3_600_000);
    assert!(a.ended.is_empty() && a.live.is_none(), "home, waiting");
}

#[test]
fn the_log_folds_by_absence_and_keeps_sixty() {
    let mut g = with_scout(4);
    run_offline_quick(&mut g, 3600);
    run_offline_quick(&mut g, 1800);
    let first: Vec<&RunRec> = g.lineage.run_log.iter().collect();
    assert!(first.iter().all(|r| r.absence == Some(1)));
    let n1 = first.len();
    watched(&mut g, None);
    g.auto_keep();
    run_offline_quick(&mut g, 3600);
    let log = &g.lineage.run_log;
    assert_eq!(log[n1].via, "watched");
    assert!(log[n1 + 1..].iter().all(|r| r.absence == Some(2)), "a send ends the absence");
    assert_eq!(g.lineage().absences, 2);
    // a long absence on a set that stalls extrapolates: one `sampled` record
    let mut s = Game::new_literal(2);
    let mut found = false;
    for _ in 0..6 {
        run_offline_quick(&mut s, 24 * 3600);
        if s.lineage.run_log.iter().any(|r| r.sampled.is_some_and(|n| n > 0) && r.id == 0) {
            found = true;
            break;
        }
    }
    assert!(found, "a sampled absence carries its `+N`");
    // the cap
    let mut l = Game::new(1).lineage;
    for i in 0..(RUN_LOG_CAP as u32 + 10) {
        l.push_run(RunRec { id: i + 1, ..Default::default() });
    }
    assert_eq!(l.run_log.len(), RUN_LOG_CAP);
    assert_eq!(l.run_log[0].id, 11);
}
