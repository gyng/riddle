//! Cut 115 (docs/CUT115_BUILDS_FROM_TACTICS.md): builds from tactics — the build's name and synergy, who chose each
//! row (the credit), variants from L1 with distinct main rows, the compare's variant moves, and fixes as picks.
use crate::packages;
use crate::Game;

/// A resident lineage with the tactic slot open (the Warlord slain) and `owned` arrived.
fn picked(seed: u64, owned: &[&str]) -> Game {
    let mut g = Game::new_resident(seed);
    g.lineage.kills.insert("goblin_warlord".into());
    for id in owned {
        g.lineage.pkg.owned.insert(id.to_string());
    }
    packages::recompile(&mut g.lineage);
    g
}

#[test]
fn every_tactics_variants_differ_in_the_main_row_from_l1() {
    for d in packages::PACKAGES.iter().filter(|d| packages::variants(d.id).is_some()) {
        let rows = |v: u8| -> Vec<crate::rules::Row> {
            let mut r = packages::tactic_lead_rows(d.id, v);
            r.extend(packages::tactic_rows_v(d.id, 1, v));
            r
        };
        let (a, b) = (rows(0), rows(1));
        assert_ne!(a.first(), b.first(), "{}: the two variants lead with the same row at L1", d.id);
        assert_eq!(a.len(), 1, "{}: the first variant is the card alone at L1", d.id);
        assert!(b.iter().any(|r| r.card() == Some(d.id)), "{}: the second keeps the card", d.id);
    }
    // the pick is open at L1, and a variant that commits leads the stance's guard rows
    let mut g = picked(4, &["boss_focus"]);
    packages::equip(&mut g.lineage, "boss_focus", 0).unwrap();
    packages::set_variant(&mut g.lineage, "boss_focus", 1).unwrap();
    let rows = &g.lineage.rules().rows;
    let lead = rows.iter().position(|r| r.origin.as_deref() == Some("tactic:boss_focus")).unwrap();
    let guard = rows.iter().position(|r| r.origin.as_deref() == Some("stance:steady")).unwrap();
    assert!(lead < guard, "boss first swings before the heal");
}

#[test]
fn an_equipped_tactics_rows_count_as_picked_and_the_school_as_default() {
    let mut g = picked(5, &["corridor_fighting"]);
    packages::equip(&mut g.lineage, "corridor_fighting", 0).unwrap();
    let p = &g.lineage.pkg;
    assert_eq!(packages::credit("tactic:corridor_fighting", p), "picked");
    assert_eq!(packages::credit("stance:steady", p), "default");
    assert_eq!(packages::credit("stance:guarded", p), "picked");
    assert_eq!(packages::credit("drill:goblin_warlord", p), "taught");
    assert_eq!(packages::credit("patch", p), "taught");
    assert_eq!(packages::credit("player", p), "picked");
    assert_eq!(packages::credit("chores", p), "chores");
    assert_eq!(packages::credit("temper:skittish", p), "default", "the wake's draw is no pick");
    // the share is the core's, read from the fired rows' origins on the wire
    let origins = [("tactic:corridor_fighting", 6u32), ("stance:steady", 2), ("chores", 2)].into_iter().map(|(k, n)| (k.to_string(), n)).collect();
    let shares = packages::credit_shares(&origins, p);
    let get = |c: &str| shares.iter().find(|s| s.credit == c).map(|s| s.share);
    assert_eq!((get("picked"), get("default"), get("chores"), get("taught")), (Some(0.6), Some(0.2), Some(0.2), None));
    // a real absence: the tactic's fires are metered under its origin and credited `picked`
    let mut g = picked(5, &["corridor_fighting"]);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    packages::equip(&mut g.lineage, "corridor_fighting", 0).unwrap();
    crate::offline::run_offline_counts(&mut g, 4 * 3600);
    let night = g.lineage.night_meter.clone();
    let m = if night.is_empty() { g.lineage.last_night_meter.clone() } else { night };
    let fired = m.origins.get("tactic:corridor_fighting").copied().unwrap_or(0);
    let wire = crate::meters::wire_for(&m, Some(&g.lineage.pkg));
    assert!(!wire.credit.is_empty(), "the meter carries its credit");
    if fired > 0 {
        assert!(wire.credit.iter().any(|c| c.credit == "picked" && c.fires >= fired), "the tactic's fires are picked: {:?}", wire.credit);
    }
}

#[test]
fn idle_wears_no_build_and_plays_no_synergy() {
    let mut g = Game::new_resident(9);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    crate::offline::run_offline_counts(&mut g, 8 * 3600);
    assert_eq!(packages::build_name(&g.lineage), None);
    assert!(packages::build_wire(&g.lineage).is_none());
    assert_eq!(packages::build_mask(g.lineage.rules()), 0);
    assert!(g.lineage.chronicle.iter().all(|l| !l.contains("Bulwark")));
}

#[test]
fn a_pair_names_the_build_and_plays_its_synergy() {
    let mut g = picked(6, &["guarded", "corridor_fighting", "boss_focus"]);
    packages::equip(&mut g.lineage, "boss_focus", 0).unwrap();
    assert_eq!(packages::build_name(&g.lineage).as_deref(), Some("Steady slayer"));
    packages::equip(&mut g.lineage, "guarded", 0).unwrap();
    assert_eq!(packages::build_name(&g.lineage).as_deref(), Some("Guarded slayer"));
    assert_eq!(packages::build_mask(g.lineage.rules()), 0);
    packages::equip(&mut g.lineage, "corridor_fighting", 0).unwrap();
    assert_eq!(packages::build_name(&g.lineage).as_deref(), Some("Bulwark"));
    assert_eq!(packages::build_mask(g.lineage.rules()), packages::synergy_bit("bulwark"));
    let w = packages::wire(&g.lineage).build.unwrap();
    assert_eq!((w.name.as_str(), w.synergy.as_deref()), ("Bulwark", Some("bulwark")));
    assert!(w.picks.iter().any(|p| p == "corridor fighting · at three"));
    // a literal set (the harnesses') plays none
    let lit = Game::new_literal(6);
    assert_eq!(packages::build_mask(lit.lineage.rules()), 0);
    assert_eq!(packages::build_name(&lit.lineage), None);
    // every synergy holds a tactic (IDLE wears none), names are ≤ 2 words, effects ≤ 6
    for s in packages::SYNERGIES.iter() {
        assert!(s.pair.iter().any(|p| packages::def(p).is_some_and(|d| d.kind == packages::Kind::Tactic)), "{}", s.id);
        assert!(s.name.split_whitespace().count() <= 2 && s.effect.split_whitespace().filter(|w| *w != "·").count() <= 6, "{}", s.id);
    }
}

#[test]
fn iron_lungs_takes_two_off_the_gas() {
    let hurt = |pair: bool| -> (i32, bool) {
        let mut g = picked(8, &["gas_step"]);
        g.lineage.heir = 3;
        packages::equip(&mut g.lineage, "gas_step", 0).unwrap();
        if pair {
            packages::wear(&mut g.lineage, Some("iron_gut"));
            g.lineage.pkg.temperament_chosen = true;
            packages::recompile(&mut g.lineage);
        }
        g.start_run(None);
        let (r, mut cx) = g.ctx();
        let before = r.hero.hp;
        crate::turn::damage_hero(r, &mut cx, 5, &crate::turn::Src::Gas);
        let said = cx.events.iter().any(|e| matches!(e, crate::wire::Ev::Callout { text, .. } if text == "Iron lungs"));
        (before - r.hero.hp, said)
    };
    assert_eq!(hurt(false), (5, false));
    assert_eq!(hurt(true), (5 - packages::IRON_LUNGS, true));
}

#[test]
fn a_death_offers_the_tactic_that_answers_it_and_taking_it_is_taught() {
    let mut g = picked(12, &["gas_step", "kite_archers", "boss_focus"]);
    assert_eq!(packages::fix_pick(&g.lineage, "goblin_archer").map(|x| (x.0, x.1, x.2)), Some(("kite_archers".into(), 0, "kite archers · hunt".into())));
    assert_eq!(packages::fix_pick(&g.lineage, "bloat").map(|x| x.0), Some("gas_step".into()));
    assert_eq!(packages::fix_pick(&g.lineage, "goblin_warlord").map(|x| (x.0, x.1)), Some(("boss_focus".into(), 0)), "a summoner: summons first");
    assert_eq!(packages::fix_pick(&g.lineage, "rat"), None, "nothing answers a rat");
    // before the pen opens, the lever leads with it
    let lever = packages::lever(&g.lineage, "goblin_archer").unwrap();
    assert_eq!((lever.kind.as_str(), lever.id.as_deref(), lever.variant), ("tactic", Some("kite_archers"), Some(0)));
    packages::take_fix(&mut g.lineage, "kite_archers", 1).unwrap();
    assert_eq!(g.lineage.pkg.tactics, vec!["kite_archers".to_string()]);
    assert_eq!(g.lineage.pkg.variants.get("kite_archers"), Some(&1));
    assert_eq!(packages::credit("tactic:kite_archers", &g.lineage.pkg), "taught", "the fix did the learning");
    assert_eq!(packages::fix_pick(&g.lineage, "goblin_archer").map(|x| x.1), Some(0), "the other variant is still on offer");
    // the player's own pick makes it his
    packages::set_variant(&mut g.lineage, "kite_archers", 0).unwrap();
    assert_eq!(packages::credit("tactic:kite_archers", &g.lineage.pkg), "picked");
    // no slot, no tactic fix
    let none = Game::new_resident(12);
    assert_eq!(packages::fix_pick(&none.lineage, "goblin_archer"), None);
}

#[test]
fn the_compare_prices_a_worn_tactics_other_variant() {
    let mut g = picked(7, &["pack_break", "guarded"]);
    packages::equip(&mut g.lineage, "pack_break", 0).unwrap();
    let opts = packages::options_for(&g, 8, &[("pack_break#1".into(), 1), ("guarded".into(), 0)]);
    let v = opts.iter().find(|o| o.action == "variant").expect("the variant move is priced");
    assert_eq!((v.id.as_str(), v.slot, v.n), ("pack_break", 1, 8));
    assert!(opts.iter().any(|o| o.action == "equip" && o.id == "guarded"));
    // the worn variant itself is no move
    assert!(packages::options_for(&g, 8, &[("pack_break#0".into(), 0)]).is_empty());
    // the walls a compare reads: each lit stone's next band boss at or under the record's next floor, deepest first
    let mut l = g.lineage.clone();
    l.best_depth = 14;
    let walls = packages::compare_walls(&l);
    assert!(walls.iter().all(|(s, w)| s <= w && *w <= 15 && crate::descent::boss_for(*w).is_some()), "{walls:?}");
    assert!(walls.windows(2).all(|x| x[0].1 > x[1].1));
}
