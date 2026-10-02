//! Cut 30.5 (docs/CUT30_5.md): the works tree — the node table, the chest and its conservation, the
//! workers' standing orders, the manual send before the scout, the old-save mapping.
use crate::offline::run_offline_counts;
use crate::tree::{self, NODES};
use crate::Game;

/// A send by hand, played out unwatched (the dayplayer's model): the run in flight, then home.
fn send_by_hand(g: &mut Game) -> crate::wire::ReturnReport {
    g.send();
    run_offline_counts(g, 1)
}

fn conserved(g: &Game) -> bool {
    let l = &g.lineage;
    l.gold as i64 + l.town.bank as i64 == l.tree.ledger && tree::purse(l) + l.tree.chest == l.gold && l.tree.chest >= 0
}

#[test]
fn the_node_table_lights_one_at_a_time_in_order() {
    // the order: the trunk (the quartermaster given, the porter free, the scout) then the branches; every chore names one node
    let ids: Vec<&str> = NODES.iter().map(|n| n.id).collect();
    assert_eq!(&ids[..5], &["quartermaster", "porter", "scout", "armourer", "apprentice"]);
    assert_eq!(tree::def("porter").unwrap().price_tenths, 0, "the porter is free");
    assert!(NODES.iter().skip(2).all(|n| n.price_tenths > 0), "the rest cost gold");
    for n in NODES.iter().filter(|n| !n.chore.is_empty()) {
        assert_eq!(NODES.iter().filter(|m| m.chore == n.chore).count(), 1, "{}", n.chore);
        assert!(n.name.split_whitespace().count() <= 2 && n.tip.split_whitespace().count() <= 10 && n.beat.split_whitespace().count() <= 2, "{}", n.id);
    }
    let mut g = Game::new(5);
    assert!(tree::hired(&g.lineage, "quartermaster") && !tree::hired(&g.lineage, "porter"));
    assert_eq!(tree::lit(&g.lineage).map(|n| n.id), None, "nothing lit at 0:00");
    assert_eq!(g.lineage().tree.unwrap().next.unwrap().kind, "send", "the pill says send");
    // three sends by hand, the chest never opened: the scout lights (the porter is not ready)
    for _ in 0..3 {
        let r = send_by_hand(&mut g);
        assert_eq!(r.runs, 1);
        assert!(conserved(&g));
    }
    assert!(g.lineage.tree.chest > 0, "the hauls wait in the chest");
    assert_eq!(tree::count(&g.lineage, "scout"), 3);
    assert_eq!(tree::lit(&g.lineage).map(|n| n.id), Some("scout"));
    // the price is in forge units
    let p = tree::price(&g.lineage, tree::def("scout").unwrap());
    assert_eq!(p, ((crate::kit::unit_of(&g.lineage) * 5 + 5) / 10) as i32);
    // opening the chest three times lights the porter ahead of the scout (the tree's order): one lit
    let mut h = g.clone();
    tree::open_chest(&mut h.lineage).unwrap();
    for _ in 0..20 {
        if tree::count(&h.lineage, "porter") >= 3 {
            break;
        }
        send_by_hand(&mut h);
        let _ = tree::open_chest(&mut h.lineage);
    }
    assert_eq!(tree::lit(&h.lineage).map(|n| n.id), Some("porter"));
    let w = h.lineage().tree.unwrap();
    assert_eq!(w.nodes.iter().filter(|n| n.state == "lit").count(), 1, "one lit");
    assert_eq!(w.nodes.iter().find(|n| n.id == "scout").unwrap().state, "ready", "the scout waits its turn");
    assert!(h.hire("scout").is_err(), "not lit");
    h.hire("porter").unwrap();
    assert_eq!(h.lineage.tree.chest, 0);
    assert_eq!(tree::lit(&h.lineage).map(|n| n.id), Some("scout"));
    // a node whose chore does not exist yet is shut (the bank before it is built), and its gate is named
    let w = h.lineage().tree.unwrap();
    let clerk = w.nodes.iter().find(|n| n.id == "clerk").unwrap();
    assert_eq!((clerk.state.as_str(), clerk.trigger.as_deref()), ("shut", Some("bank built")));
    // the stages of the four tracks are the branches' nodes
    assert!(w.nodes.iter().any(|n| n.kind == "stage" && n.id == "town:bank" && n.state != "done"));
    // the fallback: a node's chore open, the lineage old enough, it lights without the count
    let mut f = Game::new(5);
    crate::tree::grant(&mut f.lineage, &["porter", "scout"]);
    f.lineage.systems.insert("storehouse".into());
    f.lineage.clock_s = 25 * 3600;
    assert_eq!(tree::lit(&f.lineage).map(|n| n.id), Some("armourer"));
}

#[test]
fn manual_send_before_the_scout_yields_the_run_in_flight() {
    let mut g = Game::new(11);
    // an absence with no send: the hero waits, nothing runs
    let r = run_offline_counts(&mut g, 8 * 3600);
    assert_eq!(r.runs, 0, "no send, no run");
    assert!(g.waits());
    // a send, then a long absence: the one run, home, waiting again
    g.send();
    assert!(!g.waits());
    let r = run_offline_counts(&mut g, 8 * 3600);
    assert_eq!(r.runs, 1, "a send before the scout is one run");
    assert!(g.waits() && !g.lineage.tree.sent);
    // watched: the step plays the sent run, then the hero waits (no run begins by itself)
    g.send();
    let mut over = false;
    for _ in 0..4000 {
        if g.step(50).run_over {
            over = true;
            break;
        }
    }
    assert!(over);
    let n = g.lineage.next_run_id;
    for _ in 0..40 {
        assert!(g.step(500).run_over);
    }
    assert_eq!(g.lineage.next_run_id, n, "the hero waits at camp");
    g.auto_keep();
    // the scout lifts it: three sends by hand, then hired; offline is uncapped again
    send_by_hand(&mut g);
    assert_eq!(tree::lit(&g.lineage).map(|n| n.id), Some("scout"));
    if g.lineage.gold < tree::price(&g.lineage, tree::def("scout").unwrap()) {
        for _ in 0..4 {
            send_by_hand(&mut g);
        }
    }
    g.hire("scout").unwrap();
    assert!(tree::auto_send(&g.lineage) && !g.waits());
    let r = run_offline_counts(&mut g, 8 * 3600);
    assert!(r.runs >= 10, "{} runs after the scout", r.runs);
    // switched off, a send is by hand again
    g.set_worker("scout", false).unwrap();
    assert_eq!(run_offline_counts(&mut g, 4 * 3600).runs, 0);
    // a harness's literal lineage always sends itself
    let mut lit = Game::new_literal(11);
    assert!(run_offline_counts(&mut lit, 8 * 3600).runs >= 10);
}

#[test]
fn the_chest_conserves_gold_never_caps_and_never_moves_the_sim() {
    // two IDLE lineages with the scout granted, one opens the chest at every check-in: the same game
    let mut a = Game::new(23);
    let mut b = Game::new(23);
    for g in [&mut a, &mut b] {
        tree::grant(&mut g.lineage, &["scout"]);
    }
    let mut peak = 0;
    for ci in 0..9 {
        let ra = run_offline_counts(&mut a, 8 * 3600);
        let rb = run_offline_counts(&mut b, 8 * 3600);
        assert!(conserved(&a) && conserved(&b), "check-in {ci}");
        assert_eq!((ra.runs, ra.banked, ra.returned), (rb.runs, rb.banked, rb.returned), "check-in {ci}");
        assert!(b.lineage.tree.chest >= peak.min(b.lineage.gold), "the chest gives way only to spending");
        peak = b.lineage.tree.chest;
        if a.lineage.tree.chest > 0 {
            let before = a.lineage.gold;
            tree::open_chest(&mut a.lineage).unwrap();
            assert_eq!(a.lineage.gold, before, "opening moves gold from the chest to the purse");
        }
        assert_eq!(a.lineage.gold, b.lineage.gold);
        assert_eq!(a.lineage.best_depth, b.lineage.best_depth);
        assert_eq!(a.lineage.total_turns, b.lineage.total_turns);
        assert_eq!(a.lineage.facts, b.lineage.facts);
    }
    assert!(b.lineage.tree.chest > 1000, "the chest holds the haul: {}", b.lineage.tree.chest);
    assert_eq!(tree::purse(&b.lineage), b.lineage.gold - b.lineage.tree.chest);
    // a manual buy draws on the purse alone; a hire on the purse then the chest
    assert!(crate::kit::buy(&mut b, "weapon").is_err() || tree::purse(&b.lineage) >= 0);
    assert_eq!(b.lineage().gold, tree::purse(&b.lineage), "the wire's gold is the purse");
}

#[test]
fn workers_keep_their_standing_orders() {
    let mut g = Game::new(29);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    for _ in 0..6 {
        run_offline_counts(&mut g, 8 * 3600);
        assert!(conserved(&g));
    }
    // the apprentice: forge steps at the sends, the reserve kept
    let mut a = g.clone();
    let steps = |g: &Game| crate::kit::KIT_SLOTS.iter().map(|s| crate::kit::owned(&g.lineage, s)).sum::<u32>();
    let s0 = steps(&a);
    tree::grant(&mut a.lineage, &["apprentice"]);
    let r = run_offline_counts(&mut a, 8 * 3600);
    assert!(steps(&a) > s0, "the apprentice bought a step");
    assert!(r.workers.iter().any(|w| w.id == "apprentice" && w.first), "{:?}", r.workers);
    assert!(conserved(&a));
    // never in a sim: a forecast leaves the lineage as it was
    let before = serde_json::to_string(&a.lineage).unwrap();
    let _ = a.forecast();
    assert_eq!(serde_json::to_string(&a.lineage).unwrap(), before);
    // the clerk: the bank fills once built
    let mut c = g.clone();
    if crate::town::built(&c.lineage, "bank") {
        tree::grant(&mut c.lineage, &["clerk"]);
        run_offline_counts(&mut c, 8 * 3600);
        assert!(c.lineage.town.bank > 0, "the clerk banked");
        assert!(conserved(&c));
    }
    // the drillmaster: marks into the stance's level
    let mut d = g.clone();
    d.lineage.marks += 20;
    let lv = d.lineage.pkg.level(&d.lineage.pkg.stance.clone());
    tree::grant(&mut d.lineage, &["drillmaster"]);
    run_offline_counts(&mut d, 3600);
    assert!(d.lineage.pkg.level(&d.lineage.pkg.stance.clone()) > lv || lv >= 5);
    // the guide: a start a band under the record, never shallower than the start set
    let mut s = g.clone();
    tree::grant(&mut s.lineage, &["guide"]);
    run_offline_counts(&mut s, 3600);
    let want = tree::guide_pick(&s.lineage, false);
    assert_eq!(s.lineage.start.max(1), want, "the guide's start");
    // a stone whose sends stop paying is given up
    let mut t = s.clone();
    if want > 1 {
        for _ in 0..4 {
            tree::note_start(&mut t.lineage, want, false, true);
        }
        assert!(tree::guide_pick(&t.lineage, false) < want, "the guide steps back");
    }
    // the armourer: the best vault weapon/armour goes with each send
    let mut w = g.clone();
    tree::grant(&mut w.lineage, &["armourer"]);
    if !w.lineage.vault.is_empty() {
        let n = w.lineage.tree.acts.get("armourer").copied().unwrap_or(0);
        run_offline_counts(&mut w, 3600);
        assert!(w.lineage.tree.acts.get("armourer").copied().unwrap_or(0) > n, "the armourer brought the vault");
    }
    // the keeper: a watched exit never asks
    let mut k = g.clone();
    tree::grant(&mut k.lineage, &["keeper"]);
    k.send();
    for _ in 0..4000 {
        let r = k.step(50);
        if r.run_over {
            assert!(r.exit_pending.is_none_or(|p| !p.decide));
            break;
        }
    }
    // a literal lineage: no worker acts
    let mut l = Game::new_literal(29);
    tree::grant(&mut l.lineage, &["apprentice", "clerk"]);
    run_offline_counts(&mut l, 8 * 3600);
    assert!(l.lineage.tree.acts.is_empty());
}

#[test]
fn chores_by_hand_count_toward_their_worker() {
    let mut g = Game::new(31);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    for _ in 0..3 {
        run_offline_counts(&mut g, 8 * 3600);
    }
    let ids: Vec<u32> = g.lineage.vault.iter().map(|v| v.id).collect();
    if !ids.is_empty() {
        g.loadout(ids.clone());
        g.loadout(ids.clone());
        assert_eq!(tree::count(&g.lineage, "armourer"), ids.len() as u32, "each find counts once");
    }
    g.lineage.gold += 100_000;
    g.lineage.tree.ledger += 100_000;
    if crate::kit::buy(&mut g, "weapon").is_ok() {
        assert_eq!(tree::count(&g.lineage, "apprentice"), 1);
    }
    g.lineage.marks += 5;
    let st = g.lineage.pkg.stance.clone();
    if g.spend_level(&st).is_ok() {
        assert_eq!(tree::count(&g.lineage, "drillmaster"), 1);
    }
    assert!(conserved(&g));
}

#[test]
fn an_old_save_gets_its_workers_up_to_the_scout() {
    let mut g = Game::new(37);
    tree::grant(&mut g.lineage, &["scout"]);
    run_offline_counts(&mut g, 8 * 3600);
    let mut v: serde_json::Value = serde_json::from_str(&g.save()).unwrap();
    v["lineage"].as_object_mut().unwrap().remove("tree");
    let old = Game::load(&v.to_string()).unwrap();
    for id in tree::LEGACY {
        assert!(tree::hired(&old.lineage, id), "{id}");
    }
    assert!(!tree::hired(&old.lineage, "armourer"));
    assert_eq!(old.lineage.tree.chest, 0, "no haul waits in a chest");
    assert!(conserved(&old));
    assert!(tree::auto_send(&old.lineage));
    // a save with the tree keeps it
    let again = Game::load(&g.save()).unwrap();
    assert_eq!(again.lineage.tree, g.lineage.tree);
}

#[test]
fn a_worker_ranks_up_with_service() {
    let mut g = Game::new(41);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    assert!(tree::lit_rank(&g.lineage).is_none(), "no rank on the first day");
    let porter = tree::def("porter").unwrap();
    assert_eq!(tree::rank_wait(&g.lineage, porter), Some(tree::RANK_DAYS[0]));
    g.lineage.day = tree::RANK_DAYS[0];
    assert_eq!(tree::lit_rank(&g.lineage).map(|n| n.id), Some("porter"), "the first hired worker's rank on offer");
    g.lineage.gold += 10_000;
    g.lineage.tree.ledger += 10_000;
    let before = crate::town::stage_set(&g.lineage);
    assert_eq!(g.promote("porter").unwrap(), 2);
    assert!(g.promote("porter").is_err(), "one rank at a time, III waits its days");
    assert!(crate::town::stage_set(&g.lineage).len() > before.len(), "a rank is a town stage");
    assert!(conserved(&g));
    let w = g.lineage().tree.unwrap();
    assert_eq!(w.nodes.iter().find(|n| n.id == "porter").unwrap().rank, Some(2));
    // a hire on offer comes first: no rank while a node is lit
    for _ in 0..3 {
        g.send();
        run_offline_counts(&mut g, 1);
    }
    if tree::lit(&g.lineage).is_some() {
        assert!(tree::lit_rank(&g.lineage).is_none());
    }
}

/// A practised Guarded hero with an empty heal pack recovers between fights before abandoning the walk.
#[test]
fn practised_guarded_recovers_in_an_empty_room_without_heals() {
    for (runs, hp) in [(120, 42), (220, 37)] {
        let mut g = Game::new(7);
        g.lineage.pkg.owned.insert("guarded".into());
        g.lineage.pkg.runs.insert("guarded".into(), runs);
        g.equip_package("guarded", 0).unwrap();
        g.send();
        let r = g.run.as_mut().unwrap();
        r.monsters.clear();
        r.hero.inv.retain(|i| i.kind != "heal");
        r.hero.max_hp = 100;
        r.hero.hp = hp;
        r.hero.energy = 100;
        g.tick();
        let r = g.run.as_ref().unwrap();
        assert!(r.over.is_none() && r.homeward.is_none(), "the empty pack alone must not end a recoverable walk");
        assert!(r.hero.hp > hp, "Guarded rests between fights instead of banking");
    }
}

#[test]
fn guarded_recovers_the_last_wounds_before_exploring_again() {
    let mut g = Game::new(7);
    g.lineage.pkg.owned.insert("guarded".into());
    g.lineage.pkg.runs.insert("guarded".into(), 220);
    g.equip_package("guarded", 0).unwrap();
    g.send();
    let r = g.run.as_mut().unwrap();
    r.monsters.clear();
    r.hero.inv.retain(|i| i.kind != "heal");
    r.hero.max_hp = 100;
    r.hero.hp = 95;
    r.hero.energy = 100;
    let pos = r.hero.pos;
    g.tick();
    let r = g.run.as_ref().unwrap();
    assert!(r.hero.hp > 95, "the guard recovers even its final wounds");
    assert_eq!(r.hero.pos, pos, "rest comes before exploring another room");
    assert!(r.homeward.is_none() && r.over.is_none());
}
