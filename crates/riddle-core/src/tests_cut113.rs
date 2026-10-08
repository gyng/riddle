//! Cut 113 §2–3: the forge's two branches a tier, and the return's pick.
use crate::engine::Game;

fn fresh(seed: u64) -> Game {
    let mut g = Game::new_resident(seed);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    g
}

fn hero_of(g: &Game) -> crate::hero::Hero {
    let mut h = crate::hero::Hero::new(g.lineage.class, crate::geom::Pos::new(0, 0));
    crate::kit::equip(&g.lineage, &mut h);
    h
}

#[test]
fn forge_tiers_offer_two_branches_priced_alike_and_default_is_the_old_ladder() {
    let mut g = fresh(3);
    g.lineage.kit_unit = Some(100);
    g.lineage.gold = 100_000;
    let lads = crate::kit::ladders(&g.lineage);
    let w = lads.iter().find(|l| l.slot == "weapon").unwrap();
    assert_eq!(w.branches.iter().map(|b| (b.id.as_str(), b.default)).collect::<Vec<_>>(), vec![("aim", true), ("edge", false)]);
    let a = lads.iter().find(|l| l.slot == "armour").unwrap();
    assert_eq!(a.branches.iter().map(|b| b.id.as_str()).collect::<Vec<_>>(), vec!["plate", "pace"]);
    assert!(lads.iter().find(|l| l.slot == "pack").unwrap().branches.is_empty());
    // the default branch forges exactly the old ladder: no `forged` on the arm, aim to +3
    let mut d = g.clone();
    for _ in 0..4 {
        crate::kit::buy(&mut d, "weapon").unwrap();
    }
    let h = hero_of(&d);
    let arm = h.weapon.as_ref().unwrap();
    assert!(arm.forged.is_none() && d.lineage.kit_alt.is_empty());
    assert_eq!(h.hit_pct(), 92);
    // the same tiers by branch name, the defaults named: the same kit, the same price
    let mut n = g.clone();
    for b in ["aim", "aim", "aim", "edge"] {
        crate::kit::buy(&mut n, &format!("weapon:{b}")).unwrap();
    }
    assert_eq!(n.lineage.gold, d.lineage.gold);
    assert_eq!(hero_of(&n).weapon.as_ref().unwrap(), arm);
    assert!(n.lineage.kit_lean.is_empty());
    // the edge at tier 1: a harder blow, no aim; the apprentice leans the same way after
    let mut e = g.clone();
    let before = e.lineage.gold;
    crate::kit::buy(&mut e, "weapon:edge").unwrap();
    assert_eq!(before - e.lineage.gold, before - g.lineage.gold + crate::kit::price(&g.lineage, "weapon", 0) as i32);
    let h = hero_of(&e);
    assert_eq!(h.hit_pct(), 80);
    assert_eq!(h.atk().0, hero_of(&g).atk().0 + crate::kit::EARLY_EDGE);
    assert_eq!(e.lineage.kit_lean.get("weapon").map(String::as_str), Some("edge"));
    crate::kit::buy_step_off(&mut e.lineage, "weapon", 0).unwrap();
    assert_eq!(crate::kit::branch_counts(&e.lineage, "weapon"), [0, 2]);
    // pace: speed in tenths, no armour piece
    let mut p = g.clone();
    crate::kit::buy(&mut p, "armour:pace").unwrap();
    let h = hero_of(&p);
    assert!(h.armour.is_none());
    assert_eq!(h.pace, crate::kit::PACE_TENTHS[0]);
    let mut q = g.clone();
    crate::kit::buy(&mut q, "armour:plate").unwrap();
    assert!(hero_of(&q).armour.is_some() && q.lineage.kit_lean.is_empty());
    assert!(crate::kit::buy(&mut g.clone(), "weapon:plate").is_err());
}

#[test]
fn every_return_offers_a_pick_sized_by_the_absence_that_waits_and_is_seeded() {
    assert_eq!([20u64, 60, 240, 480, 960].map(crate::returns::size), [1, 2, 3, 4, 5]);
    let mut g = fresh(5);
    g.send();
    let r = crate::offline::run_offline_counts(&mut g, 20 * 60);
    let pick = r.pick.expect("a 20m return carries a pick");
    assert_eq!(pick.size, 1);
    assert_eq!(pick.offers.len(), 3);
    assert_eq!(g.lineage().return_pick, Some(pick.clone()));
    // seeded: the same lineage draws the same offers
    let mut twin = fresh(5);
    twin.send();
    assert_eq!(crate::offline::run_offline_counts(&mut twin, 20 * 60).pick, Some(pick.clone()));
    // an untaken pick waits and grows (never lost): the next absence adds its minutes
    let mut kept = g.clone();
    let r2 = crate::offline::run_offline_counts(&mut kept, 8 * 3600);
    let grown = r2.pick.unwrap();
    assert_eq!(grown.minutes, 20 + 480);
    assert_eq!(grown.size, 4);
    // taking one closes it: Legacy points
    let pts = |g: &Game| g.lineage.bloodline.as_ref().map_or(0, |b| b.points);
    let p0 = pts(&kept);
    kept.take_return_pick("legacy").unwrap();
    assert_eq!(pts(&kept), p0 + crate::returns::LEGACY_POINTS[3]);
    assert!(kept.lineage.return_pick.is_none() && kept.lineage().return_pick.is_none());
    assert!(kept.take_return_pick("legacy").is_err());
}

#[test]
fn an_untaken_pick_never_moves_the_game() {
    let mut a = fresh(7);
    a.send();
    crate::offline::run_offline_counts(&mut a, 3600);
    assert!(a.lineage.return_pick.is_some());
    let mut b = a.clone();
    b.lineage.return_pick = None;
    for _ in 0..3 {
        let ra = crate::offline::run_offline_counts(&mut a, 8 * 3600);
        let rb = crate::offline::run_offline_counts(&mut b, 8 * 3600);
        assert_eq!((ra.runs, ra.deepest, a.lineage.gold, a.lineage.best_depth), (rb.runs, rb.deepest, b.lineage.gold, b.lineage.best_depth));
    }
}
