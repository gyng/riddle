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
fn first_home_blocks_runs_and_survives_reload_without_spending() {
    let mut g = Game::new(17);
    let original = g.save();
    assert!(!g.lineage().town.home);
    assert_eq!(g.lineage().town.next.as_deref(), Some("house"));
    assert!(g.try_send().is_err());
    assert_eq!(g.save(), original, "refusal cannot mutate the game");
    let report = run_offline_counts(&mut g, 8 * 3600);
    assert_eq!(report.runs, 0);
    assert!(g.run.is_none());
    assert!(g.lineage.hero_legacy.iter().all(|h| h.points == 0));
    let gold = g.lineage.gold;
    g.build_town("house").unwrap();
    assert_eq!(g.lineage.gold, gold);
    assert!(g.build_town("house").is_err());
    let loaded = Game::load(&g.save()).unwrap();
    assert!(loaded.lineage().town.home);
    assert!(g.try_send().is_ok());
}

#[test]
fn missing_home_and_legacy_fields_preserve_old_residents_and_collect_chests() {
    let g = Game::new_resident(17);
    let mut saved: serde_json::Value = serde_json::from_str(&g.save()).unwrap();
    saved["lineage"]["town"].as_object_mut().unwrap().remove("home");
    saved["lineage"]["town"].as_object_mut().unwrap().remove("auto_collect");
    saved["lineage"]["town"].as_object_mut().unwrap().remove("gold_v");
    saved["lineage"].as_object_mut().unwrap().remove("hero_legacy");
    let mut old = Game::load(&saved.to_string()).unwrap();
    assert!(old.lineage().town.home);
    assert!(old.lineage.town.auto_collect);
    assert!(old.try_send().is_ok());
    saved["lineage"].as_object_mut().unwrap().remove("town");
    assert!(Game::load(&saved.to_string()).unwrap().lineage().town.home);
}

#[test]
fn hero_legacy_awards_once_archives_on_death_and_is_not_class_xp() {
    use crate::engine::ExitTier;
    let mut g = Game::new_resident(17);
    g.lineage.orders.legacy = "off".into(); // (Cut 120 §1: the points kept, not spent by the order)
    for tier in [ExitTier::Bank, ExitTier::Bank, ExitTier::Death] {
        g.start_run(None);
        let r = g.run.as_mut().unwrap();
        r.max_depth = 4;
        r.over = Some(tier);
        if tier == ExitTier::Death { r.hero.hp = 0; r.death_cause = Some("rat".into()); }
        g.finish_run();
        let before = g.lineage.hero_legacy.clone();
        assert!(g.finish_run().is_none());
        assert_eq!(g.lineage.hero_legacy, before);
    }
    assert_eq!(g.lineage.bloodline.as_ref().unwrap().points, 7);
    assert_eq!(g.lineage.hero_legacy[0].runs, 3);
    assert_eq!(g.lineage.hero_legacy[0].best_depth, 4);
    assert_eq!(g.lineage.hero_legacy.last().unwrap().heir, 2);
    assert_eq!(g.lineage.hero_legacy.last().unwrap().points, 0);
    let saved = g.save();
    assert_eq!(Game::load(&saved).unwrap().lineage.hero_legacy, g.lineage.hero_legacy);
    g.lineage.classes.get_mut("fighter").unwrap().xp += 100;
    assert_eq!(g.lineage.bloodline.as_ref().unwrap().points, 7);
}

#[test]
fn automatic_haul_collection_conserves_gold_and_readies_the_porter() {
    use crate::engine::ExitTier;
    let mut g = Game::new_resident(17);
    for _ in 0..3 {
        g.start_run(None);
        let r = g.run.as_mut().unwrap();
        r.loot = 100;
        r.secured = 20;
        r.over = Some(ExitTier::Bank);
        g.finish_run();
        assert_eq!(g.lineage.tree.chest, 0);
        assert_eq!(tree::purse(&g.lineage), g.lineage.gold);
        assert!(conserved(&g));
    }
    assert_eq!(tree::count(&g.lineage, "porter"), 3);
    assert_eq!(tree::lit(&g.lineage).map(|n| n.id), Some("porter"));
    g.hire("porter").unwrap();
    assert!(conserved(&g));
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
    let mut g = Game::new_resident(5);
    g.lineage.town.auto_collect = false; // Legacy manual collection remains supported.
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
    let mut f = Game::new_resident(5);
    crate::tree::grant(&mut f.lineage, &["porter", "scout"]);
    f.lineage.systems.insert("storehouse".into());
    f.lineage.clock_s = 25 * 3600;
    assert_eq!(tree::lit(&f.lineage).map(|n| n.id), Some("armourer"));
}

#[test]
fn manual_send_before_the_scout_yields_the_run_in_flight() {
    let mut g = Game::new_resident(11);
    g.lineage.town.auto_collect = false; // Legacy manual collection remains supported.
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
    let mut a = Game::new_resident(23);
    a.lineage.town.auto_collect = false; // Legacy manual collection remains supported.
    let mut b = Game::new_resident(23);
    b.lineage.town.auto_collect = false; // Legacy manual collection remains supported.
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
    let mut g = Game::new_resident(29);
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
    // (the premise: a find in the vault that beats the forged kit — blind b8dd77c's seed-29 lineage kept a
    // dragging mail alone, which the armourer rightly leaves; a +3 sword beats the bare kit)
    let id = w.lineage.next_vault_id;
    w.lineage.next_vault_id += 1;
    let mut sword = crate::item::Item::new(id, "sword");
    sword.enchant = 3;
    sword.known = true;
    w.lineage.vault.push(sword);
    {
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
    let mut g = Game::new_resident(31);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    for _ in 0..3 {
        run_offline_counts(&mut g, 8 * 3600);
    }
    g.build_town("blacksmith").unwrap();
    if !g.lineage.vault.is_empty() { g.build_town("storehouse").unwrap(); }
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
    let mut g = Game::new_resident(37);
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
    let mut g = Game::new_resident(41);
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
        let mut g = Game::new_resident(7);
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
    let mut g = Game::new_resident(7);
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


#[test]
fn guide_income_counts_actual_checkpoint_passage_and_rounded_exit_payment() {
    use crate::engine::ExitTier;
    for (tier, secured, loot, passage, timed_out, paid) in [
        (ExitTier::Death, 1221, 1452, 0, false, true),
        (ExitTier::Death, 0, 0, 533, false, true),
        (ExitTier::Death, 0, 0, 0, false, false),
        (ExitTier::Return, 0, 1, 0, false, false),
        (ExitTier::Return, 1221, 1452, 0, true, true),
        (ExitTier::Return, 0, 1452, 0, true, false),
    ] {
        let mut g = crate::tests::arena();
        g.lineage.gold_move(100_000, "test reserve");
        g.lineage.best_depth = 28;
        g.lineage.tree.stones_best = 28;
        let run = g.run.as_mut().unwrap();
        run.start = 19;
        run.depth = 28;
        run.max_depth = 28;
        run.secured = secured;
        run.loot = loot;
        run.passage = passage;
        run.timed_out = timed_out;
        run.over = Some(tier);
        if tier == ExitTier::Death { run.hero.hp = 0; run.death_cause = Some("lurker".into()); }
        let expected_kept = run.kept(tier);
        let gold = g.lineage.gold;
        g.finish_run().unwrap();
        assert_eq!(g.lineage.gold - gold, expected_kept, "passage was paid at send, not twice at exit");
        assert_eq!(g.last_exit.as_ref().unwrap().kept, expected_kept);
        assert_eq!(g.lineage.tree.stones.get(&19), Some(&(1, u32::from(paid))), "{tier:?}, secured={secured}, loot={loot}, passage={passage}, timeout={timed_out}");
    }
}

fn guide_quote_camp() -> Game {
    use crate::rules::{Cond, Row, Verb};
    let mut g = Game::new_resident(11);
    g.lineage.best_depth = 28;
    g.lineage.light_waystones(28);
    g.lineage.gold_move(100_000, "test reserve");
    crate::kit::buy_all(&mut g.lineage);
    tree::grant(&mut g.lineage, &["porter", "guide"]);
    g.lineage.pkg.pen_open = true;
    g.lineage.pkg.pen = vec![Row::new(vec![Cond::n("depth>=", 2)], Verb::new("bank")).from("player")];
    crate::packages::recompile(&mut g.lineage);
    g
}

#[test]
fn guide_changed_start_prices_current_camp_before_its_send_snapshot() {
    static WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(2);
    crate::forecast::with_sim_width(&WIDTH, || {
        let mut g = guide_quote_camp();
        g.lineage.start = 14;
        g.passage = Some((14, 1));
        let mut priced = g.clone();
        tree::at_send(&mut priced);
        assert_eq!(priced.lineage.start, 24);
        let expected = crate::forecast::sim_passage(&priced, priced.lineage.rules());
        assert!(expected.is_some_and(|(floor, coins)| floor == 24 && coins > 0));
        g.start_run(Some(42));
        assert_eq!(g.passage, expected);
        assert_eq!(g.run.as_ref().unwrap().start, 24);
        assert_eq!(g.run.as_ref().unwrap().passage, expected.unwrap().1);
        assert_eq!(g.sent_state.as_ref().unwrap().lineage.start, 24);
    });
}

#[test]
fn guide_unchanged_start_keeps_existing_quote_and_none_without_repricing() {
    for quote in [Some((24, 7777)), None] {
        let mut g = guide_quote_camp();
        g.lineage.start = 24;
        g.passage = quote;
        g.start_run(Some(42));
        assert_eq!(g.lineage.start, 24);
        assert_eq!(g.passage, quote, "an unchanged start must not launch a passage panel");
        assert_eq!(g.run.as_ref().unwrap().passage, quote.map_or(0, |q| q.1));
    }
    let mut g = guide_quote_camp().sim_clone();
    g.lineage.start = 14;
    g.passage = Some((14, 7777));
    g.start_run(Some(42));
    assert_eq!(g.lineage.start, 14, "sims do not run the guide");
    assert_eq!(g.passage, Some((14, 7777)), "sims must not recursively price passage");
}

#[test]
fn guide_hourly_start_change_prices_passage_before_the_next_send() {
    static WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(2);
    crate::forecast::with_sim_width(&WIDTH, || {
        for quote in [Some((14, 1)), None] {
            let mut g = guide_quote_camp();
            g.lineage.start = 14;
            g.passage = quote;
            tree::at_hour(&mut g);
            assert_eq!(g.lineage.start, 24);
            let expected = crate::forecast::sim_passage(&g, g.lineage.rules());
            assert!(expected.is_some_and(|(floor, coins)| floor == 24 && coins > 0));
            g.start_run(Some(42));
            assert_eq!(g.passage, expected, "hourly changes happen before start_run captures its old floor");
            assert_eq!(g.run.as_ref().unwrap().passage, expected.unwrap().1);
        }
    });
}

#[test]
fn legacy_purchases_change_real_stats_persist_and_follow_the_bloodline() {
    use crate::engine::ExitTier;
    let mut g = Game::new_resident(19);
    g.lineage.orders.legacy = "off".into(); // (Cut 120 §1: purchases by hand)
    g.lineage.bloodline.as_mut().unwrap().points = 54;
    let xp = g.lineage.classes.clone(); let gold = g.lineage.gold; let rules = g.lineage.rules().clone();
    let mut baseline = g.clone(); baseline.start_run(Some(991));
    let base = baseline.run.as_ref().unwrap().hero.clone();
    let key = crate::forecast::lineage_key(&g);
    for id in crate::legacy::IDS { for _ in 0..3 { g.upgrade_hero(id).unwrap(); } }
    assert_ne!(key, crate::forecast::lineage_key(&g));
    assert_eq!(g.lineage.classes, xp); assert_eq!(g.lineage.gold, gold); assert_eq!(g.lineage.rules(), &rules);
    let h = g.lineage.bloodline.as_ref().unwrap(); assert_eq!((h.points,h.spent), (0,54));
    let mut g = Game::load(&g.save()).unwrap(); g.start_run(Some(991));
    let hero = &g.run.as_ref().unwrap().hero;
    assert_eq!(hero.max_hp,base.max_hp+9); assert_eq!(hero.max_hp_base,base.max_hp_base+9);
    assert_eq!(hero.atk(),(base.atk().0+3,base.atk().1+3)); assert_eq!(hero.def(),base.def()+3);
    let mut armour_only = crate::hero::Hero::new(crate::hero::Class::Fighter, crate::geom::Pos::new(0,0));
    armour_only.legacy_armour=3;
    for roll in 1..20 { assert!(armour_only.blunt(roll)>0); }
    let r=g.run.as_mut().unwrap();r.over=Some(ExitTier::Death);r.hero.hp=0;r.death_cause=Some("rat".into());g.finish_run().unwrap();
    assert_eq!(g.lineage.bloodline.as_ref().unwrap().spent,54); assert_eq!(g.lineage.hero_legacy[1].points,0);
    assert!(g.lineage.hero_legacy[1].upgrades.is_empty());
    g.start_run(Some(991)); assert_eq!(g.run.as_ref().unwrap().hero.legacy_armour,3);
}

#[test]
fn legacy_refuses_poor_unknown_capped_and_away_purchases_without_mutation() {
    let mut g = Game::new_resident(20);
    for id in ["health","unknown"] { let before=g.save(); assert!(g.upgrade_hero(id).is_err());assert_eq!(before,g.save()); }
    g.lineage.bloodline.as_mut().unwrap().points=100;
    for (rank,price) in [(1,3),(2,6),(3,9)] {
        let before=g.lineage.bloodline.as_ref().unwrap().points;g.upgrade_hero("health").unwrap();
        assert_eq!(g.lineage.bloodline.as_ref().unwrap().points,before-price);assert_eq!(g.lineage.bloodline.as_ref().unwrap().upgrades["health"],rank);
    }
    let before=g.save();assert!(g.upgrade_hero("health").is_err());assert_eq!(before,g.save());
    let key=crate::forecast::lineage_key(&g);g.lineage.bloodline.as_mut().unwrap().points+=1;assert_eq!(key,crate::forecast::lineage_key(&g));
    g.send();let before=g.save();assert!(g.upgrade_hero("damage").is_err());assert_eq!(before,g.save());
    assert!(g.lineage().legacy_upgrades.iter().all(|u| !u.affordable));
}

#[test]
fn old_chest_migration_conserves_all_gold_and_is_idempotent_including_live_runs() {
    for live in [false,true] {
        let mut g=Game::new_resident(21);g.lineage.town.gold_v=0;g.lineage.town.auto_collect=false;
        g.lineage.gold_move(1234,"returned D4");g.lineage.town.bank=50;g.lineage.tree.ledger+=50;
        g.lineage.heir_best=7;g.lineage.hero_legacy.clear();g.lineage.bloodline=None;
        if live { g.send(); }
        let ledger=g.lineage.tree.ledger;let gold=g.lineage.gold;let counts=g.lineage.tree.done.clone();
        let g=Game::load(&g.save()).unwrap();assert!(g.lineage.town.auto_collect);assert_eq!(g.lineage.tree.chest,0);
        assert_eq!(g.lineage.gold,gold);assert_eq!(g.lineage.town.bank,50);assert_eq!(g.lineage.tree.ledger,ledger);
        assert_eq!(g.lineage.tree.done,counts);assert_eq!(g.lineage.hero_legacy[0].best_depth,7);
        assert_eq!(g.lineage.bloodline.as_ref().unwrap().points,0);assert_eq!(g.run.is_some(),live);
        let save=g.save();assert_eq!(Game::load(&save).unwrap().save(),save);
    }
}


#[test]
fn scout_actions_count_automatic_runs_not_manual_sends_and_survive_reload() {
    let mut g = Game::new_resident(2);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    let count = |g: &Game| g.lineage.tree.acts.get("scout").copied().unwrap_or(0);
    g.send();
    g.send(); // A live run resumed by hand is still no scout act.
    assert_eq!(count(&g), 0);
    let id = g.lineage.next_run_id;
    let first = run_offline_counts(&mut g, 8 * 3600);
    let started = g.lineage.next_run_id - id;
    assert!(started > 0);
    assert_eq!(count(&g), started);
    let act = first.workers.iter().find(|a| a.id == "scout").unwrap();
    assert_eq!(act.n, started);
    assert_eq!(act.what, format!("sent {started}"));
    assert!(act.first);
    let mut loaded = Game::load(&g.save()).unwrap();
    let before = count(&loaded);
    let later = run_offline_counts(&mut loaded, 3600);
    let act = later.workers.iter().find(|a| a.id == "scout").unwrap();
    assert_eq!(act.n, count(&loaded) - before);
    assert!(!act.first);
    loaded.set_worker("scout", false).unwrap();
    let before = count(&loaded);
    run_offline_counts(&mut loaded, 3600);
    assert_eq!(count(&loaded), before);
    loaded.send();
    run_offline_counts(&mut loaded, 3600);
    assert_eq!(count(&loaded), before);
}

#[test]
fn scout_actions_cover_live_clock_and_do_not_change_gameplay_or_simulations() {
    let mut g = Game::new_resident(2);
    tree::grant(&mut g.lineage, &["scout"]);
    let id = g.lineage.next_run_id;
    g.advance(100);
    assert_eq!(g.lineage.tree.acts.get("scout"), Some(&(g.lineage.next_run_id - id)));
    assert_eq!(g.lineage.next_run_id - id, 1);
    let mut expected = g.save();
    tree::scout_sent(&mut g);
    let mut after: serde_json::Value = serde_json::from_str(&g.save()).unwrap();
    let mut before: serde_json::Value = serde_json::from_str(&expected).unwrap();
    before["lineage"]["tree"].as_object_mut().unwrap().remove("acts");
    after["lineage"]["tree"].as_object_mut().unwrap().remove("acts");
    assert_eq!(before, after, "bookkeeping changes only the worker ledger");
    for kind in 0..4 {
        let mut excluded = match kind { 0 | 3 => Game::new_resident(2), 1 => Game::new_literal(2), _ => g.sim_clone() };
        if kind == 3 { tree::grant(&mut excluded.lineage, &["scout"]); excluded.set_worker("scout", false).unwrap(); }
        expected = excluded.save();
        tree::scout_sent(&mut excluded);
        assert_eq!(excluded.save(), expected, "unhired/paused/literal/sim must not record a scout act");
    }
}


#[test]
fn hero_names_are_read_only_stable_and_follow_heir_history() {
    for seed in [0, 2, u64::MAX] {
        let mut g = Game::new_resident(seed);
        let raw = g.save();
        let first = g.lineage().hero_legacy[0].name.clone();
        assert!(!first.is_empty());
        assert_eq!(g.save(), raw, "wire names must not mutate saves or RNG");
        assert_eq!(Game::load(&raw).unwrap().lineage().hero_legacy[0].name, first);
        g.lineage.heir += 1;
        crate::legacy::ensure(&mut g.lineage);
        let wire = g.lineage();
        assert_eq!(wire.hero_legacy[0].name, first);
        assert_ne!(wire.hero_legacy.last().unwrap().name, first);
        assert_eq!(wire.hero_legacy.last().unwrap().name, crate::legacy::hero_identity(seed, 2, 1));
    }
}


#[test]
fn hero_slot_looks_resolve_defaults_and_keep_chosen_bloodlines_independent() {
    let mut s = crate::bloodlines::Session::new(2);
    s.build_town("house").unwrap();
    assert_eq!(s.lineage().hero_slots[0].look, "male");
    s.active.lineage.gold = 500;
    s.active.lineage.tree.ledger = 500;
    s.set_look("female").unwrap();
    s.add_bloodline().unwrap();
    s.select_bloodline(2).unwrap();
    s.set_look("cat").unwrap();
    let slots = s.lineage().hero_slots;
    assert_eq!(slots[0].look, "female");
    assert_eq!(slots[1].look, "cat");
    let before = s.save();
    assert!(s.set_look("unknown").is_err());
    assert_eq!(s.save(), before);
    let mut loaded = crate::bloodlines::Session::load(&before).unwrap();
    assert_eq!(loaded.lineage().hero_slots, slots);
    loaded.select_bloodline(1).unwrap();
    assert_eq!(loaded.lineage().hero_slots, slots);
    assert_eq!(loaded.lineage().look, "female");
}


#[test]
fn recorded_death_hero_is_historical_and_old_wire_is_compatible() {
    let mut g = Game::new_resident(2);
    g.lineage.bloodline_id = 2;
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    let before = g.save();
    let recorded = crate::trace::death_record(&g, run).death;
    assert_eq!(g.save(), before, "recording identity cannot change game or RNG");
    let hero = recorded.hero.as_ref().unwrap();
    assert_eq!(hero.name, crate::legacy::hero_identity(2, run.heir, 2));
    assert_eq!(hero.heir, run.heir);
    assert_eq!(hero.class, run.hero.class.name());
    assert_eq!(hero.bloodline_id, 2);
    g.lineage.heir += 1;
    g.lineage.bloodline_id = 3;
    let raw = serde_json::to_value(&recorded).unwrap();
    let restored: crate::wire::Death = serde_json::from_value(raw.clone()).unwrap();
    assert_eq!(restored.hero, recorded.hero);
    assert_ne!(restored.hero.as_ref().unwrap().name, crate::legacy::hero_identity(2, g.lineage.heir, g.lineage.bloodline_id));
    let mut old = raw;
    old.as_object_mut().unwrap().remove("hero");
    let loaded: crate::wire::Death = serde_json::from_value(old).unwrap();
    assert!(loaded.hero.is_none());
    assert!(serde_json::to_value(loaded).unwrap().get("hero").is_none());
}

#[test]
fn run_training_beats_reach_every_exit_copy_without_repeating() {
    use crate::engine::ExitTier;
    for tier in [ExitTier::Bank, ExitTier::Death] {
        let mut g = Game::new_resident(2);
        g.lineage.pkg.runs.insert("steady".into(), 9);
        g.lineage.pkg.met_runs.insert("goblin_warlord".into(), 1);
        g.lineage.facts.insert("boss:goblin_warlord:counter".into());
        for round in 0..2 {
            g.start_run(None);
            let i = g.lineage.rules().rows.iter().position(|r| r.origin.as_deref() == Some("stance:steady")).unwrap();
            let r = g.run.as_mut().unwrap();
            r.row_fired[i] = 1;
            r.bosses_met.push("goblin_warlord".into());
            r.over = Some(tier);
            if tier == ExitTier::Death { r.hero.hp = 0; r.death_cause = Some("rat".into()); }
            let id = r.id;
            g.finish_run().unwrap();
            let line = g.last_exit.as_ref().unwrap();
            assert_eq!(line.packages, g.batch.exits.last().unwrap().packages);
            if tier == ExitTier::Death { assert_eq!(line.packages, g.deaths[&id].death.line.as_ref().unwrap().packages); }
            if round == 0 {
                assert!(line.packages.contains(&"STEADY L2".into()));
                assert!(line.packages.contains(&"DRILLED · Warlord".into()));
            } else {
                assert!(!line.packages.iter().any(|s| s.starts_with("DRILLED") || s == "STEADY L2"));
            }
            let mut raw = serde_json::to_value(line).unwrap();
            let copy: crate::wire::ExitLine = serde_json::from_value(raw.clone()).unwrap();
            assert_eq!(copy.packages, line.packages);
            raw.as_object_mut().unwrap().remove("packages");
            assert!(serde_json::from_value::<crate::wire::ExitLine>(raw).unwrap().packages.is_empty());
        }
    }
}


#[test]
fn bloodline_families_distinguish_colliding_given_names_and_preserve_archives() {
    use crate::legacy::{hero_identity,hero_name};
    for seed in 0..128 {
        for heir in 1..=25 {
            let names:std::collections::BTreeSet<_>=(1..=3).map(|id|hero_identity(seed,heir,id)).collect();
            assert_eq!(names.len(),3);
            assert!(names.iter().all(|name|name.starts_with(&format!("{} ",hero_name(seed,heir)))));
            for id in 1..=3 {assert_ne!(hero_identity(seed,heir,id),hero_identity(seed,heir+1,id));}
        }
    }
    assert_ne!(hero_identity(2,1,4),hero_identity(2,1,5));
    let mut s=crate::bloodlines::Session::new(2);s.build_town("house").unwrap();
    s.active.lineage.gold_move(1000,"test income");s.add_bloodline().unwrap();s.add_bloodline().unwrap();
    // Force the witnessed collision without consuming game RNG.
    s.others.get_mut(&2).unwrap().lineage.seed=s.active.lineage.seed;
    s.others.get_mut(&3).unwrap().lineage.seed=s.active.lineage.seed;
    let before=s.save();let slots=s.lineage().hero_slots;
    assert_eq!(slots.iter().map(|h|&h.hero_name).collect::<std::collections::BTreeSet<_>>().len(),3);
    assert_eq!(s.save(),before,"wire names cannot mutate save or RNG");
    s.select_bloodline(2).unwrap();assert_eq!(s.lineage().hero_slots,slots);
    assert_eq!(s.lineage().hero_legacy.last().unwrap().name,slots[1].hero_name);
    let restored=crate::bloodlines::Session::load(&s.save()).unwrap();assert_eq!(restored.lineage().hero_slots,slots);
    let mut g=Game::new_resident(2);
    g.lineage.hero_legacy[0].name="Wren".into();
    let raw=g.save();assert_eq!(g.lineage().hero_legacy[0].name,"Wren");assert_eq!(g.save(),raw);
    g.start_run(None);let death=crate::trace::death_record(&g,g.run.as_ref().unwrap()).death;
    let mut old=serde_json::to_value(death).unwrap();old["hero"]["name"]=serde_json::json!("Wren");
    let kept:crate::wire::Death=serde_json::from_value(old).unwrap();assert_eq!(kept.hero.unwrap().name,"Wren");
}

#[test]
fn snapshot_actor_identity_survives_load_heir_advance_and_slot_switch() {
    let mut s = crate::bloodlines::Session::new(2);
    s.build_town("house").unwrap();
    s.active.lineage.gold = 500;
    s.active.lineage.tree.ledger = 500;
    s.add_bloodline().unwrap();
    let first = s.send();
    let first_name = first.hero.entity.name.clone().unwrap();
    assert_eq!(first_name, s.lineage().hero_slots[0].hero_name);
    let before = s.save();
    assert_eq!(s.snapshot().hero.entity.name.as_deref(), Some(first_name.as_str()));
    assert_eq!(s.save(), before, "snapshot identity never mutates save or RNG");
    assert_eq!(crate::bloodlines::Session::load(&before).unwrap().snapshot().hero.entity.name, first.hero.entity.name);
    s.select_bloodline(2).unwrap();
    let second = s.send();
    assert_eq!(second.hero.entity.name.as_deref(), Some(s.lineage().hero_slots[1].hero_name.as_str()));
    assert_ne!(first.hero.entity.name, second.hero.entity.name);
    s.select_bloodline(1).unwrap();
    s.active.lineage.heir += 1;
    assert_eq!(s.snapshot().hero.entity.name, first.hero.entity.name, "a retained run uses its own heir");
    let replay = s.replay(first.run.id).unwrap();
    assert!(replay.floors.iter().all(|f| f.snapshot.hero.entity.name == first.hero.entity.name));
    let mut old = serde_json::to_value(first).unwrap();
    old["hero"].as_object_mut().unwrap().remove("name");
    let old: crate::wire::Snapshot = serde_json::from_value(old).unwrap();
    assert!(old.hero.entity.name.is_none(), "old snapshots still decode");
}

/// Blind ad71e72 (A: `$8 GOLD EARNED` while the purse rose ~$1000 on a D19 start; `$6712` with
/// the purse up $78): the report's gold counts the waystone passage the sends were paid, and
/// states the purse's actual change over the absence.
#[test]
fn absence_gold_counts_passage_and_states_the_purse_change() {
    let mut g = guide_quote_camp();
    tree::grant(&mut g.lineage, &["scout"]);
    g.lineage.start = 24;
    let before = g.lineage.gold;
    let r = run_offline_counts(&mut g, 8 * 3600);
    let gold = r.gold.expect("an absence reports its gold");
    assert!(r.runs > 0, "the absence ran");
    assert_eq!(gold.net, Some(g.lineage.gold - before), "net is the purse's change");
    assert!(gold.passage > 0, "waystone sends were paid passage: {gold:?}");
    assert_eq!(gold.home + gold.salvage + gold.passage + gold.wake - gold.spent, g.lineage.gold - before, "with nothing else bought, the parts reconcile: {gold:?}");
    let saved = crate::wire::GoldSummary { net: None, passage: 0, ledger: None, ..gold.clone() };
    let text = serde_json::to_string(&saved).unwrap();
    assert!(!text.contains("passage") && !text.contains("net"), "old wire unchanged when absent: {text}");
}

/// Blind c4705f9 (A, B: `purse −$12562` while away, the apprentice's `+4 steps` the only word): the report names
/// each step the apprentice reached and the purse it spent, to the coin of the purse's fall.
#[test]
fn apprentice_itemises_its_purchases() {
    let mut g = Game::new_resident(29);
    tree::grant(&mut g.lineage, &["apprentice"]);
    // (the whole spare purse: the itemising, not the order, is under test)
    g.lineage.orders.forge = "all".into();
    g.lineage.best_depth = 12;
    g.lineage.gold = 40_000;
    let before = (*g.lineage.tree.acts).clone();
    let (gold0, kit0): (i32, Vec<u32>) = (g.lineage.gold, crate::kit::KIT_SLOTS.iter().map(|s| crate::kit::owned(&g.lineage, s)).collect());
    tree::at_send(&mut g);
    let acts = tree::report_acts(&g.lineage, &before, &g.lineage.tree.acts);
    let a = acts.iter().find(|w| w.id == "apprentice").expect("the apprentice bought steps");
    assert_eq!(a.spent, gold0 - g.lineage.gold, "spent reconciles with the purse: {a:?}");
    assert!(a.spent > 0 && a.n > 0);
    let moved: Vec<&str> = crate::kit::KIT_SLOTS.iter().zip(&kit0).filter(|(s, k)| crate::kit::owned(&g.lineage, s) > **k).map(|(s, _)| *s).collect();
    assert_eq!(a.items.len(), moved.len(), "one item per slot bought: {a:?}");
    for (s, item) in moved.iter().zip(&a.items) {
        assert_eq!(item, &crate::kit::step_label(&g.lineage, s, crate::kit::owned(&g.lineage, s) as usize - 1));
    }
    // the bookkeeping keys are no worker of their own
    assert!(acts.iter().all(|w| !w.id.contains(['$', ':'])));
    // a second look with nothing bought names nothing
    let again = (*g.lineage.tree.acts).clone();
    assert!(tree::report_acts(&g.lineage, &again, &g.lineage.tree.acts).is_empty());
}

/// Blind 1fb7786 (A, B: `forge −$20925 · purse +$823`, "−$24500 then −$21875 of my gold without asking"): the
/// apprentice forges under a standing order — `half` (the default) half of each haul home and never the purse the
/// player left, `all` the spare purse, `off` nothing; the order is the player's, round-trips and refuses nonsense.
#[test]
fn apprentice_forges_under_its_standing_order() {
    let base = || {
        let mut g = Game::new_resident(29);
        tree::grant(&mut g.lineage, &["porter", "apprentice"]);
        g.lineage.best_depth = 12;
        g.lineage.gold = 40_000;
        g
    };
    let spend = |g: &mut Game| {
        let before = g.lineage.gold;
        tree::at_send(g);
        before - g.lineage.gold
    };
    // the default: the purse the player left is his — nothing is forged until a haul comes home
    let mut h = base();
    assert_eq!(h.lineage().orders.forge.as_str(), "half", "half is the default");
    assert_eq!(spend(&mut h), 0, "the purse left behind is never forged");
    h.lineage.gold_move(20_000, "returned D12");
    let spent = spend(&mut h);
    assert!(spent > 0 && spent <= 10_000, "half of the haul at most: spent {spent}");
    assert!(tree::purse(&h.lineage) >= 50_000, "the player keeps his purse and half the haul: {}", tree::purse(&h.lineage));
    assert!(h.lineage.tree.forge_budget >= 0 && h.lineage.tree.forge_budget <= 10_000 - spent);
    // `all`: the spare purse down to the reserve, as before
    let mut a = base();
    a.lineage.orders.forge = "all".into();
    let all = spend(&mut a);
    assert!(all > 20_000, "all spends the spare purse: {all}");
    // `off`: nothing, haul or not
    let mut o = base();
    let mut orders = o.lineage().orders;
    orders.forge = "off".into();
    o.set_orders(&orders).unwrap();
    o.lineage.gold_move(20_000, "returned D12");
    assert_eq!(spend(&mut o), 0, "off forges nothing");
    assert_eq!(o.lineage().orders.forge, "off");
    // a nonsense order is refused and changes nothing
    orders.forge = "most".into();
    assert!(o.set_orders(&orders).is_err());
    assert_eq!(o.lineage.orders.forge, "off");
    // the order survives a save
    let o2 = Game::load(&o.save()).unwrap();
    assert_eq!(o2.lineage.orders.forge, "off");
    // an old save (no order written) reads `half`
    let sw: crate::wire::StandingSwitches = serde_json::from_str(r#"{"insure":true}"#).unwrap();
    assert_eq!(sw.forge, "half");
}

/// Blind b8dd77c (B: `purse −$11784` across a 4 h absence under the default `half` order): the apprentice
/// never spends past half of the hauls home since his hire — over real absences, each one's spend is at
/// most the budget it began with and half its own hauls — and his report line names the order.
#[test]
fn apprentice_half_order_spends_half_the_hauls_since_hire() {
    let mut g = Game::new_resident(41);
    tree::grant(&mut g.lineage, &["porter", "scout", "apprentice"]);
    assert_eq!(g.lineage.orders.forge, "half");
    let (mut checked, mut reported) = (0, false);
    for _ in 0..12 {
        let budget0 = g.lineage.tree.forge_budget as i64;
        let spent0 = g.lineage.tree.acts.get(tree::APPRENTICE_SPENT).copied().unwrap_or(0) as i64;
        let t0 = g.lineage.total_turns;
        let r = g.run_offline(3600);
        let spent = g.lineage.tree.acts.get(tree::APPRENTICE_SPENT).copied().unwrap_or(0) as i64 - spent0;
        if let Some(a) = r.workers.iter().find(|w| w.id == "apprentice" && w.spent > 0) {
            assert!(a.items.iter().any(|i| i == tree::APPRENTICE_HALF), "the order is named: {a:?}");
            reported = true;
        }
        // (the ledger keeps its last lines: an absence it no longer holds whole is not reconciled)
        if g.lineage.gold_ledger.first().is_none_or(|x| x.t >= t0) {
            continue;
        }
        let half: i64 = g.lineage.gold_ledger.iter().filter(|x| x.t >= t0 && x.delta > 0 && tree::is_haul(&x.why)).map(|x| x.delta as i64 * 50 / 100).sum();
        // (a merged salvage line halves once where the budget halved each coin: a coin a line)
        assert!(spent <= budget0 + half + 16, "spent {spent} > budget {budget0} + half the hauls {half}");
        checked += 1;
    }
    assert!(checked >= 3 && reported, "checked {checked} absences, reported {reported}");
}
