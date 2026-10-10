//! Cut 120 (docs/CUT120_AUTOMATION_FILL.md, core items 1–7): the Legacy order, ranks by order with the rank-II perk,
//! `same for all`, the herald's ascension, the kennel keeper as a worker, the worker ledger, the next worker visible.
use crate::bloodlines::Session;
use crate::engine::{ExitTier, Game};
use crate::tree;

fn points(g: &Game) -> u32 {
    crate::legacy::current(&g.lineage).map_or(0, |b| b.points)
}

fn upgrades(g: &Game) -> Vec<(String, u32)> {
    crate::legacy::current(&g.lineage).map(|b| b.upgrades.iter().map(|(k, v)| (k.clone(), *v)).collect()).unwrap_or_default()
}

fn give_points(g: &mut Game, n: u32) {
    crate::legacy::ensure(&mut g.lineage);
    g.lineage.bloodline.as_mut().unwrap().points += n;
}

/// §1: `balanced` buys health → damage → armour in turn, then waits for the next in turn; a focus takes its upgrade
/// to the cap first; `off` buys nothing. A new lineage starts `balanced`, a save keeps `off`. Announced once.
#[test]
fn the_legacy_order_spends_in_turn() {
    let mut g = Game::new_resident(3);
    g.lineage.orders.legacy = "balanced".into();
    give_points(&mut g, 30);
    tree::at_send(&mut g);
    // 3 + 3 + 3, then 6 + 6 + 6 = 27; the next health is 9 > 3
    assert_eq!(upgrades(&g), vec![("armour".into(), 2), ("damage".into(), 2), ("health".into(), 2)]);
    assert_eq!(points(&g), 3);
    assert_eq!(g.lineage.tree.acts.get(tree::LEGACY_ACT).copied(), Some(6));
    assert_eq!(g.lineage.tree.acts.get(tree::LEGACY_POINTS).copied(), Some(27));
    tree::at_send(&mut g);
    assert_eq!(g.batch.pkg_lines.iter().filter(|l| *l == "LEGACY BALANCED").count(), 1, "announced once");
    assert!(g.lineage.feats.news.iter().any(|n| n.k == "order" && n.text == "legacy balanced"));
    // a focus: damage to its cap (3 + 6 + 9) before the others
    let mut h = Game::new_resident(3);
    h.lineage.orders.legacy = "damage".into();
    give_points(&mut h, 20);
    tree::at_send(&mut h);
    assert_eq!(upgrades(&h), vec![("damage".into(), 3)]);
    assert_eq!(points(&h), 2);
    // capped: balanced after (health first)
    give_points(&mut h, 1);
    tree::at_send(&mut h);
    assert_eq!(upgrades(&h), vec![("damage".into(), 3), ("health".into(), 1)]);
    // off
    let mut o = Game::new_resident(3);
    o.lineage.orders.legacy = "off".into();
    give_points(&mut o, 30);
    tree::at_send(&mut o);
    assert!(upgrades(&o).is_empty());
    // owner option B: a player's new lineage (and a new bloodline) starts `off` · `off`, as a harness's game and a save
    // from before the orders do — the bars measure what a new player gets
    let mut sess = Session::new(3);
    assert_eq!((sess.active.lineage.orders.legacy.as_str(), sess.active.lineage.orders.ranks.as_str()), (tree::NEW_LEGACY, tree::NEW_RANKS));
    assert_eq!((tree::NEW_LEGACY, tree::NEW_RANKS), ("off", "off"));
    sess.build_town("house").unwrap();
    sess.active.lineage.gold_move(1_000, "test income");
    sess.add_bloodline().unwrap();
    assert!(sess.others.values().all(|g| g.lineage.orders.legacy == tree::NEW_LEGACY && g.lineage.orders.ascend == "off"));
    assert_eq!(Game::new_resident(3).lineage.orders.legacy, "off");
    let s = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    assert_eq!((s.lineage.orders.legacy.as_str(), s.lineage.orders.ranks.as_str(), s.lineage.orders.ascend.as_str()), ("off", "off", "off"));
    // through setOrders, validated
    let mut w = g.lineage.standing_orders();
    assert_eq!(w.legacy.as_deref(), Some("balanced"));
    w.legacy = Some("armour".into());
    g.set_orders(&w).unwrap();
    assert_eq!(g.lineage.orders.legacy, "armour");
    w.legacy = Some("hoard".into());
    assert!(g.set_orders(&w).is_err());
}

/// Past the base three: the effects in the offer's order, a fork's first branch, never one shut by depth.
#[test]
fn the_legacy_order_takes_the_effects_after_the_base() {
    let mut g = Game::new_resident(4);
    g.lineage.orders.legacy = "balanced".into();
    g.lineage.best_depth = 8;
    give_points(&mut g, 3 * (3 + 6 + 9) + 18);
    tree::at_send(&mut g);
    let u = upgrades(&g);
    assert!(u.contains(&("restoration".into(), 1)), "{u:?}");
    assert!(!u.iter().any(|(k, _)| k == "mending"), "shut by depth: {u:?}");
    assert_eq!(points(&g), 0);
}

/// A hired worker's rank comes after `RANK_DAYS`.
fn ranked_camp(seed: u64) -> Game {
    let mut g = Game::new_resident(seed);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    g.lineage.day = tree::RANK_DAYS[0];
    g.lineage.orders.ranks = "auto".into();
    g
}

/// §2: `ranks: auto` buys each rank that has come while the purse keeps the reserve; `off` none; a short purse none.
#[test]
fn the_ranks_order_buys_due_ranks_keeping_the_reserve() {
    let mut g = ranked_camp(5);
    g.lineage.gold_move(100_000, "test income");
    let gold = g.lineage.gold;
    tree::at_send(&mut g);
    assert_eq!((tree::rank(&g.lineage, "porter"), tree::rank(&g.lineage, "scout")), (2, 2));
    let spent = g.lineage.tree.acts.get(tree::RANKS_GOLD).copied().unwrap_or(0) as i32;
    assert_eq!(gold - g.lineage.gold, spent);
    assert_eq!(g.lineage.tree.acts.get(tree::RANKS).copied(), Some(2));
    assert!(g.batch.pkg_lines.iter().any(|l| l == "RANKS AUTO"));
    // a purse that cannot keep the reserve buys nothing
    let mut h = ranked_camp(5);
    let r = tree::reserve(&h.lineage);
    let gold = h.lineage.gold;
    h.lineage.gold_move(r - gold, "test");
    tree::at_send(&mut h);
    assert_eq!(tree::rank(&h.lineage, "scout"), 1);
    // off
    let mut o = ranked_camp(5);
    o.lineage.orders.ranks = "off".into();
    o.lineage.gold_move(100_000, "test income");
    tree::at_send(&mut o);
    assert_eq!(tree::rank(&o.lineage, "scout"), 1);
}

/// §2 (Cut 114 §5): the rank-II perk chip — the first perk the old bonus (the default), the second its own.
#[test]
fn the_rank_two_perk_is_a_chip() {
    let mut g = ranked_camp(6);
    g.lineage.tree.ranks.insert("scout".into(), 2);
    let w = tree::wire(&g.lineage, false);
    let scout = w.nodes.iter().find(|n| n.id == "scout").unwrap();
    assert_eq!(scout.perks, vec!["shorter rest".to_string(), "safer start".to_string()]);
    assert_eq!(scout.perk.as_deref(), Some("shorter rest"));
    assert_eq!(scout.bonus.as_deref(), Some("−5% rest"));
    assert_eq!(tree::rest_scaled(&g.lineage, 1000), 950);
    let base = {
        let mut h = Game::load(&g.save()).unwrap();
        h.start_run(None);
        h.run.as_ref().unwrap().hero.max_hp
    };
    g.set_perk("scout", "safer start").unwrap();
    assert_eq!(tree::rest_scaled(&g.lineage, 1000), 1000, "the rest is the first perk's");
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().hero.max_hp, base + base * tree::SAFER_START_PCT / 100);
    let w = tree::wire(&g.lineage, false);
    assert_eq!(w.nodes.iter().find(|n| n.id == "scout").unwrap().bonus.as_deref(), Some("+5% hp"));
    assert!(g.set_perk("scout", "faster").is_err());
    assert!(g.set_perk("guide", "safer start").is_err());
    // the apprentice's second perk keeps a unit more and forgoes the discount
    tree::grant(&mut g.lineage, &["apprentice"]);
    g.lineage.tree.ranks.insert("apprentice".into(), 2);
    let r = tree::reserve(&g.lineage);
    assert_eq!(tree::apprentice_off(&g.lineage), tree::APPRENTICE_OFF_PCT);
    g.set_perk("apprentice", "keeps a reserve").unwrap();
    assert_eq!(tree::apprentice_off(&g.lineage), 0);
    assert_eq!(tree::reserve(&g.lineage), r + crate::kit::unit(g.lineage.best_depth) as i32);
}

/// §3: an order set `same for all` copies to every bloodline, a later change follows, a new bloodline takes it; an
/// order not shared stays on its own bloodline.
#[test]
fn same_for_all_copies_and_follows() {
    let mut s = Session::new(7);
    s.build_town("house").unwrap();
    s.active.lineage.gold_move(2_000, "test income");
    s.add_bloodline().unwrap();
    let mut o = s.active.lineage.standing_orders();
    o.legacy = Some("health".into());
    o.forge = "all".into();
    o.shared = Some(vec!["legacy".into()]);
    s.set_orders(&o).unwrap();
    let other = |s: &Session| s.others.values().next().unwrap().lineage.orders.clone();
    assert_eq!(other(&s).legacy, "health");
    assert_ne!(other(&s).forge, "all", "not shared");
    assert!(other(&s).shared.contains("legacy"));
    o.legacy = Some("damage".into());
    s.set_orders(&o).unwrap();
    assert_eq!(other(&s).legacy, "damage", "a later change follows");
    s.add_bloodline().unwrap();
    assert!(s.others.values().all(|g| g.lineage.orders.legacy == "damage"));
    assert!(s.lineage().orders.shared.unwrap().contains(&"legacy".to_string()));
    o.shared = Some(vec!["gold".into()]);
    assert!(s.set_orders(&o).is_err());
}

/// The King fallen: the clear's state, the scout and the herald hired.
fn cleared(seed: u64) -> Game {
    let mut g = Game::new_resident(seed);
    tree::grant(&mut g.lineage, &["porter", "scout", "herald"]);
    g.lineage.ended = true;
    g.lineage.best_depth = crate::descent::ENDING_DEPTH;
    g.lineage.endgame = Some(crate::endgame::Progress { tier: 0, unlocked: 1, cleared: Some(0) });
    g
}

/// §4: after the clear the herald carries the next ascension's dungeon at a send under `ascend: on` — never without
/// the order, the herald or the scout; a new lineage's order is `off`.
#[test]
fn the_herald_ascends_only_under_the_order() {
    let mut g = cleared(9);
    assert_eq!(g.lineage.orders.ascend, "off");
    g.start_run(None);
    assert!(g.lineage.ended, "no order, no ascension");
    let mut g = cleared(9);
    g.lineage.orders.ascend = "on".into();
    g.set_worker("herald", false).unwrap();
    g.start_run(None);
    assert!(g.lineage.ended, "a paused herald carries no order");
    let mut g = cleared(9);
    g.lineage.orders.ascend = "on".into();
    g.start_run(None);
    assert!(!g.lineage.ended);
    assert_eq!(g.lineage.endgame.as_ref().map(|p| p.tier), Some(1));
    assert_eq!(g.lineage.best_depth, 0);
    assert_eq!(g.run.as_ref().unwrap().difficulty, 1, "the send is the new descent's");
    assert_eq!(g.lineage.tree.acts.get(tree::HERALD_ASCEND).copied(), Some(1));
    assert!(g.batch.pkg_lines.iter().any(|l| l == "HERALD ASCENDS"));
    let lines = tree::report_acts(&g.lineage, &Default::default(), &g.lineage.tree.acts);
    let h = lines.iter().find(|l| l.id == "herald").unwrap();
    assert_eq!((h.what.as_str(), h.first, h.items.clone()), ("ascended", true, vec!["descent 1".to_string()]));
}

/// §5: the kennel keeper is a node of the tree (chore `breed`, need 2, fallback 48 h, the kennel's); before his hire
/// the order breeds nothing and a breed by hand counts toward him; hired, he breeds under the order and his acts count.
#[test]
fn the_kennel_keeper_is_a_worker() {
    let n = tree::def("kennel_keeper").unwrap();
    assert_eq!((n.chore, n.need, n.fallback_h, n.gate, n.post), ("breed", 2, 48, "kennel", "kennel"));
    let pet = |id: u32, kind: &str, level: u32| {
        let def = crate::defs::monster_def(kind);
        let tags: Vec<String> = def.tags.iter().map(|t| t.to_string()).collect();
        let mut life = crate::pets::PetLife { role: crate::pets::role_of(kind).word().into(), ..Default::default() };
        life.xp = crate::pets::LEVEL_XP[(level - 1) as usize];
        crate::wire::Companion { id, kind: kind.into(), name: format!("P{id}"), level, tags: tags.clone(), gen: 0, rules: crate::probes::default_companion_rules(&tags, level), max_rows: 2, hp: def.hp, max_hp: def.hp, life }
    };
    let runs = |g: &mut Game, n: u32| {
        for _ in 0..n {
            g.lineage.rest_left = 0;
            g.start_run(None);
            g.run.as_mut().unwrap().over = Some(ExitTier::Return);
            g.finish_run();
        }
    };
    let camp = |hire: bool| {
        let mut g = Game::new_resident(8);
        tree::grant(&mut g.lineage, &["porter", "scout"]);
        if hire {
            tree::grant(&mut g.lineage, &["kennel_keeper"]);
        }
        g.lineage.party = vec![pet(11, "skeleton", 3)];
        g.lineage.kennel = vec![pet(12, "jackal", 2)];
        g
    };
    let mut g = camp(false);
    runs(&mut g, crate::pets::BREED_EVERY);
    assert!(g.lineage.eggs.iter().all(|e| e.from_loss), "unhired: the order waits");
    // a breed by hand counts toward him once the kennel stands
    g.build_town("kennel").ok();
    if tree::chore_open(&g.lineage, n) {
        g.breed(11, 12).unwrap();
        assert_eq!(tree::count(&g.lineage, "kennel_keeper"), 1);
    }
    let mut g = camp(true);
    let acts0 = g.lineage.tree.acts.clone();
    runs(&mut g, crate::pets::BREED_EVERY);
    assert!(g.lineage.eggs.iter().any(|e| !e.from_loss), "hired: he breeds");
    let lines = tree::report_acts(&g.lineage, &acts0, &g.lineage.tree.acts);
    let k = lines.iter().find(|l| l.id == "kennel_keeper").expect("his line");
    assert!(k.first && k.what.starts_with("sorted") && k.items.iter().any(|i| i == "bred 1"), "{k:?}");
    let w = tree::wire(&g.lineage, false);
    assert!(w.nodes.iter().any(|n| n.id == "kennel_keeper" && n.post.as_deref() == Some("kennel")));
}

/// §6, the gate: over a real absence every worker act appears on the ledger (a node's line count = its `tree.acts`
/// delta; every node that acted has a line), and the gold reconciles — the lines' purse moves equal the ledger's
/// worker terms (the apprentice, sinks, hires, bank).
#[test]
fn the_worker_ledger_matches_the_acts_and_reconciles() {
    let mut checked = 0;
    for seed in [11u64, 12, 13] {
        let mut g = Game::new_resident(seed);
        for b in ["storehouse", "blacksmith", "bank", "kennel"] {
            let _ = g.build_town(b);
        }
        tree::grant(&mut g.lineage, &["porter", "scout", "armourer", "apprentice", "keeper", "clerk", "drillmaster", "kennel_hand", "guide"]);
        g.lineage.gold_move(20_000, "test income");
        g.lineage.orders.forge = "all".into();
        g.lineage.orders.ranks = "auto".into();
        g.lineage.orders.legacy = "balanced".into();
        g.lineage.day = tree::RANK_DAYS[0];
        let before = g.lineage.tree.acts.clone();
        let r = crate::offline::run_offline_quick(&mut g, 8 * 3600);
        let after = &g.lineage.tree.acts;
        for n in tree::NODES {
            let d = after.get(n.id).copied().unwrap_or(0) - before.get(n.id).copied().unwrap_or(0);
            let line = r.workers.iter().find(|w| w.id == n.id);
            assert_eq!(line.map_or(0, |w| w.n), d, "seed {seed} {}: line vs acts", n.id);
        }
        for (key, id) in [(tree::RANKS, "ranks"), (tree::LEGACY_ACT, "legacy")] {
            let d = after.get(key).copied().unwrap_or(0) - before.get(key).copied().unwrap_or(0);
            assert_eq!(r.workers.iter().find(|w| w.id == id).map_or(0, |w| w.n), d, "seed {seed} {id}");
        }
        let ledger = r.gold.as_ref().and_then(|g| g.ledger.clone()).expect("the ledger");
        let term = |k: &str| ledger.terms.iter().find(|t| t.label == k).map_or(0, |t| t.amount);
        let lines: i32 = r.workers.iter().filter(|w| ["apprentice", "clerk", "ranks"].contains(&w.id.as_str())).map(|w| w.gold).sum();
        assert_eq!(lines, term("apprentice") + term("forge") + term("sinks") + term("hires") + term("bank"), "seed {seed}: {:?} vs {:?}", r.workers, ledger.terms);
        if r.workers.iter().any(|w| w.id == "ranks") {
            let rk = r.workers.iter().find(|w| w.id == "ranks").unwrap();
            assert_eq!(-rk.gold, -term("hires"));
            assert!(!rk.items.is_empty());
            checked += 1;
        }
        assert!(r.workers.iter().any(|w| w.id == "apprentice" && w.gold < 0), "seed {seed}: the apprentice spent");
    }
    assert!(checked > 0, "a rank bought by order in some absence");
}

/// §7: each unhired worker's node carries its count (`2/3`) and the hours to its fallback; the next worker is named.
#[test]
fn the_next_worker_shows_progress_and_eta() {
    let mut g = Game::new_resident(14);
    tree::grant(&mut g.lineage, &["porter", "scout"]);
    g.lineage.town.built.push(("storehouse".into(), 0));
    tree::did(&mut g.lineage, "wear");
    g.lineage.clock_s = 10 * 3600;
    let w = tree::wire(&g.lineage, false);
    let a = w.nodes.iter().find(|n| n.id == "armourer").unwrap();
    assert_eq!(a.progress.as_deref(), Some("1/2"));
    assert_eq!((a.fallback_h, a.eta_h), (Some(24), Some(14)));
    assert_eq!(w.next_worker.as_deref(), Some("armourer"));
    let p = w.nodes.iter().find(|n| n.id == "porter").unwrap();
    assert!(p.progress.is_none() && p.eta_h.is_none(), "hired: none");
    // the fallback reached: lit, no eta; its post carries the count and the fallback
    g.lineage.clock_s = 30 * 3600;
    let w = tree::wire(&g.lineage, false);
    assert_eq!(w.lit.as_deref(), Some("armourer"));
    assert!(w.nodes.iter().find(|n| n.id == "armourer").unwrap().eta_h.is_none());
    let post = tree::posts(&g.lineage).into_iter().find(|p| p.lit).unwrap();
    assert_eq!((post.progress.as_deref(), post.fallback_h), (Some("1/2"), Some(24)));
}

/// §6 (coordinator, the client's 3-bloodline audit: `◆102` against a bloodline's spend of 54): the town's acts sum
/// every bloodline's Legacy buys; the shown bloodline's `legacy` line is its own — its `◆` equals that bloodline's
/// `spent` delta, its count its ranks' — and the bloodlines' spends sum to the town's `legacy◆`.
#[test]
fn the_legacy_line_is_the_shown_bloodlines_own() {
    let mut s = Session::new(5);
    s.build_town("house").unwrap();
    s.active.lineage.gold_move(5_000, "test income");
    s.add_bloodline().unwrap();
    s.add_bloodline().unwrap();
    tree::grant(&mut s.active.lineage, &["porter", "scout"]);
    let ids: Vec<u32> = std::iter::once(s.selected).chain(s.others.keys().copied()).collect();
    for id in &ids {
        s.select_bloodline(*id).unwrap();
        s.active.lineage.orders.legacy = "balanced".into();
        give_points(&mut s.active, 20 + 10 * *id);
    }
    s.select_bloodline(ids[0]).unwrap();
    let game = |s: &Session, id: u32| if id == s.selected { s.active.clone() } else { s.others[&id].clone() };
    let spent = |g: &Game| crate::legacy::current(&g.lineage).map_or(0, |b| b.spent);
    let ranks = |g: &Game| crate::legacy::current(&g.lineage).map_or(0, |b| b.upgrades.values().sum::<u32>());
    let before: Vec<Game> = ids.iter().map(|id| game(&s, *id)).collect();
    let town0 = s.active.lineage.tree.acts.get(tree::LEGACY_POINTS).copied().unwrap_or(0);
    let r = s.run_offline_mode(8 * 3600, false, true);
    let after: Vec<Game> = ids.iter().map(|id| game(&s, *id)).collect();
    let deltas: Vec<u32> = before.iter().zip(&after).map(|(b, a)| spent(a) - spent(b)).collect();
    assert!(deltas.iter().filter(|d| **d > 0).count() >= 2, "several bloodlines bought: {deltas:?}");
    let town = s.active.lineage.tree.acts.get(tree::LEGACY_POINTS).copied().unwrap_or(0) - town0;
    assert_eq!(town, deltas.iter().sum::<u32>(), "the town's acts sum every bloodline's");
    let line = r.workers.iter().find(|w| w.id == tree::LEGACY_ACT).expect("the shown bloodline bought");
    assert!(line.what.ends_with(&format!(" · ◆{}", deltas[0])), "{} vs spent {}", line.what, deltas[0]);
    assert_eq!(line.n, ranks(&after[0]) - ranks(&before[0]), "its count is the shown bloodline's");
}
