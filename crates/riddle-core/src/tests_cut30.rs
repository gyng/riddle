//! Cut 30: heir traits — conditions on the hero, never actions (docs/CUT30.md, docs/TRAITS.md).
use crate::engine::Game;
use crate::rules::{Cond, Row, Verb};
use crate::tests::{add_monster, arena, rules, ticks};
use crate::traits::{self, Cost, Gift, Shape, Tier, When};
use crate::wire::Ev;

/// Every generated shape (common, uncommon allowed) and the twists: the pool a test wakes from.
fn pool() -> Vec<(Shape, bool)> {
    traits::generated().into_iter().map(|s| (s, true)).chain(traits::twists().into_iter().map(|s| (s, false))).collect()
}

/// §2: the generator's static filters and the lexicon — every generated shape has a head, a chip ≤ 3
/// words and a formula; the rejected pairs (a quiet fury, a crowded rest) are never generated.
#[test]
fn the_generator_names_every_shape_and_rejects_the_useless() {
    let g = traits::generated();
    assert!(g.len() >= 24 * 3, "{} shapes", g.len());
    for s in &g {
        assert_ne!(s.head(), "?", "{s:?}");
        assert!(crate::rules::word_count(&s.chip()) <= 4, "{}", s.chip());
        assert!(s.formula(false).contains('?') && !s.formula(true).contains('?'), "{}", s.formula(true));
    }
    assert!(!g.iter().any(|s| s.when == When::Quiet && matches!(s.gift, Gift::Fury | Gift::Guard)));
    assert!(!g.iter().any(|s| matches!(s.when, When::Crowded | When::Boss) && s.gift == Gift::Rested));
    assert!(!g.iter().any(|s| s.when == When::Deep && s.gift == Gift::Quick && s.cost == Cost::Slow));
    let w = Shape::new(When::Hurt, Gift::Fury, Cost::Frail);
    assert_eq!(w.chip(), "wrathful · frail");
    assert_eq!(w.formula(true), "[hurt] → fury +1 · frail");
    assert_eq!(w.formula(false), "[hurt] → fury ? · frail");
    assert_eq!(w.at(Tier::Uncommon).formula(true), "[hurt] → fury +2 · frail");
    // the checked-in table parses (it may be short while it is measured; never malformed)
    assert!(traits::table().shipped.iter().all(|e| e.shape().is_some()), "{:?}", traits::table());
}

/// §3: heirs 1–2 are neutral; the third wakes with three cards — distinct `when`s, deterministic, the
/// first the heir's until a pick, the first ever a common with a cost; a pick sticks through a save
/// and the send spends the offer; a name not on offer is refused.
#[test]
fn the_wake_offers_three_cards_with_different_whens() {
    let p = pool();
    let mut g = Game::new_literal(11);
    assert!(g.lineage.heirs.born.is_none() && g.lineage.heirs.offer.is_empty(), "the first heir is neutral");
    assert!(g.lineage().heir_traits.is_none());
    g.lineage.heir = 2;
    traits::wake_from(&mut g.lineage, &p);
    assert!(g.lineage.heirs.offer.is_empty() && g.lineage.heirs.born.is_none(), "heir 2 is neutral");
    for seed in 1..=30u64 {
        let mut g = Game::new_literal(seed);
        g.lineage.heir = 3;
        let mut h = g.lineage.clone();
        traits::wake_from(&mut g.lineage, &p);
        traits::wake_from(&mut h, &p);
        assert_eq!(g.lineage.heirs, h.heirs, "seed {seed}: deterministic");
        let o = &g.lineage.heirs.offer;
        assert_eq!(o.len(), 3, "seed {seed}");
        let mut whens: Vec<When> = o.iter().map(|c| c.shape.when).collect();
        whens.sort();
        whens.dedup();
        assert_eq!(whens.len(), 3, "seed {seed}: {o:?}");
        assert_eq!(g.lineage.heirs.born, Some(o[0].shape));
        assert_eq!(o[0].source, "fresh");
        assert!(o[0].shape.tier == Tier::Common && o[0].shape.cost != Cost::None, "seed {seed}: the first card ever shows its cost: {o:?}");
        let pick = o[2].shape.chip();
        crate::traits::pick(&mut g.lineage, &pick).unwrap();
        assert_eq!(g.lineage.heirs.born.map(|s| s.chip()), Some(pick.clone()));
        assert!(crate::traits::pick(&mut g.lineage, "nobody").is_err());
        let back = Game::load(&g.save()).unwrap();
        assert_eq!(back.lineage.heirs, g.lineage.heirs, "the pick and the offer survive a save");
        let w = g.lineage().heir_traits.expect("the wire carries the traits");
        assert_eq!(w.offer.len(), 3);
        assert_eq!(w.born.as_ref().map(|c| c.chip.clone()), Some(pick));
        assert!(w.offer.iter().all(|c| c.formula.contains('?')), "an unlearned gift reads `?`");
        g.send();
        assert!(g.lineage.heirs.offer.is_empty(), "the send spends the offer");
        assert_eq!(g.run.as_ref().unwrap().worn.born, g.lineage.heirs.born, "the run wears the pick");
    }
}

/// §3: the dead heir's born trait comes back twisted (one part re-rolled); the blood slot (heir 5)
/// takes the parent's born trait; a blood trait that never went live in a banked run fades a tier
/// (uncommon → common → gone), one that did keeps it; `cut` empties the slot; the bloodline leans.
#[test]
fn blood_passes_fades_and_twists() {
    let p = pool();
    let mut g = Game::new_literal(5);
    let l = &mut g.lineage;
    l.heir = 5;
    let parent = Shape::new(When::Hurt, Gift::Fury, Cost::Frail);
    l.heirs.born = Some(parent);
    traits::wake_from(l, &p);
    assert_eq!(l.heirs.blood, Some(parent), "the empty blood slot takes the parent's born trait");
    let twist = l.heirs.offer.iter().find(|c| c.source == "twist").expect("a twisted card");
    let same = (twist.shape.when == parent.when) as u8 + (twist.shape.gift == parent.gift) as u8 + (twist.shape.cost == parent.cost) as u8;
    assert_eq!(same, 2, "one part re-rolled: {:?}", twist.shape);
    // an uncommon blood trait unused by its heir fades; used in a bank, it keeps its tier
    l.heirs.blood = Some(parent.at(Tier::Uncommon));
    l.heirs.blood_live = true;
    traits::wake_from(l, &p);
    assert_eq!(l.heirs.blood.map(|s| s.tier), Some(Tier::Uncommon), "used: kept");
    traits::wake_from(l, &p);
    assert_eq!(l.heirs.blood.map(|s| s.tier), Some(Tier::Common), "unused: faded");
    assert!(l.heirs.faded.is_some());
    traits::wake_from(l, &p);
    assert_eq!(l.heirs.blood, None, "a common fades out of the family");
    // cut: the slot empties for this heir and the next does not refill it from the cut heir's born
    l.heirs.blood = Some(parent);
    traits::cut_blood(l).unwrap();
    assert_eq!(l.heirs.blood, None);
    traits::wake_from(l, &p);
    assert_eq!(l.heirs.blood, None, "a cut blood slot is not refilled at once");
    // the bloodline: heir 12; half the fresh draws take its `when`
    assert!(traits::set_bloodline(l, "deep").is_err(), "not before heir 12");
    l.heir = 12;
    traits::set_bloodline(l, "deep").unwrap();
    assert!(traits::set_bloodline(l, "hurt").is_err(), "chosen once");
    let mut deep = 0;
    for h in 12..52 {
        l.heir = h;
        l.heirs.born = None;
        traits::wake_from(l, &p);
        deep += l.heirs.offer.iter().filter(|c| c.source == "fresh" && c.shape.when == When::Deep).count();
    }
    assert!(deep >= 15, "the family leans deep: {deep} of 40 wakes");
}

/// §3 marked: the same kind killed the last three heirs — the wake offers `grudge` against it.
#[test]
fn a_grudge_three_deep_is_a_marked_card() {
    let mut g = Game::new_literal(6);
    let l = &mut g.lineage;
    l.heir = 4;
    for h in 1..=3 {
        l.graveyard.push(crate::wire::Grave { heir: h, depth: 4, cause: "ogre".into(), deeds: Vec::new(), death_id: None });
    }
    traits::wake_from(l, &pool());
    let m = l.heirs.offer.iter().find(|c| c.source == "marked").expect("a marked card");
    assert_eq!((m.shape.when, m.shape.gift, m.shape.tier), (When::Kin, Gift::Fury, Tier::Marked));
    assert_eq!(l.heirs.kin.as_deref(), Some("ogre"));
    assert_eq!(l.heirs.offer[0].source, "fresh", "card 1 stays a fresh draw");
}

/// §1: an older save's temperament maps onto a shape (`cowardly` → `skittish`, `curious` → `iron
/// gut`), and a bot's lineage is neutral for good.
#[test]
fn old_saves_map_their_temperament_and_bots_stay_neutral() {
    let g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    assert_eq!(g.lineage.heirs.born.map(|s| s.head()), Some("iron gut"), "curious → iron gut");
    let mut l = Game::new_literal(3).lineage;
    l.heirs = Default::default();
    l.trait_ = crate::hero::Trait::Cowardly;
    traits::upgrade(&mut l);
    assert_eq!(l.heirs.born.map(|s| s.head()), Some("skittish"));
    l.trait_ = crate::hero::Trait::Brave;
    assert_eq!(traits::legacy(l.trait_).head(), "unbowed");
    assert_eq!(traits::legacy(crate::hero::Trait::Greedy).head(), "light hands");
    let mut b = Game::new_literal(4);
    traits::neutral(&mut b.lineage);
    b.lineage.heir = 9;
    traits::wake_from(&mut b.lineage, &pool());
    assert!(b.lineage.heirs.offer.is_empty() && b.lineage.heirs.born.is_none() && b.lineage.heirs.blood.is_none());
    assert!(!traits::arrived(&b.lineage));
}

/// §1 + §4: a gift is live in its context and never acts — `wrathful` adds its fury while hurt and not
/// above; the first live turn learns its fact and stamps its word (once a floor); every live action
/// leaves a trace mark; the rows still choose every verb.
#[test]
fn a_live_gift_changes_the_blow_never_the_verb() {
    let mut g = arena();
    let w = Shape::new(When::Hurt, Gift::Fury, Cost::None);
    g.run.as_mut().unwrap().worn.born = Some(w);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    add_monster(&mut g, "ogre", 6, 5);
    let evs = ticks(&mut g, 30);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.hero.gift.fury, 0, "healthy: not live");
    assert!(!g.lineage.facts.contains("trait:wrathful"));
    assert!(!evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "WRATH")));
    g.run.as_mut().unwrap().hero.hp = 12;
    let evs = ticks(&mut g, 60);
    let run = g.run.as_ref().unwrap();
    assert!(g.lineage.facts.contains("trait:wrathful"), "the first live turn is a fact");
    assert_eq!(evs.iter().filter(|e| matches!(e, Ev::Callout { text, .. } if text == "WRATH")).count(), 1, "stamped once a floor");
    assert!(run.trace.iter().any(|t| t.gift.as_deref() == Some("fury +1")), "{:?}", run.trace.iter().map(|t| &t.gift).collect::<Vec<_>>());
    assert!(run.gift.born > 0);
    // the verbs are the rows' and the chores' alone
    assert!(evs.iter().all(|e| !matches!(e, Ev::Rule { row: -1, .. })), "no trait-chosen action");
    // the hero's blow reads the gift while live
    let mut h = crate::hero::Hero::new(crate::hero::Class::Fighter, crate::geom::Pos::new(0, 0));
    let base = h.atk();
    h.gift.fury = 1;
    assert_eq!(h.atk(), (base.0 + 1, base.1 + 1));
    h.gift.guard = 1;
    assert_eq!(h.blunt(3), 2);
    h.gift.slow = 1;
    assert_eq!(h.speed(), 9);
}

/// §1 costs: `frail` takes max hp at the send and the verdict's neutral counterfactual gives it back;
/// `thin` heals ⅔; `mend` moves the hp an hp-row reads.
#[test]
fn costs_are_on_and_mend_moves_hp() {
    let mut g = Game::new_literal(7);
    g.lineage.heirs.born = Some(Shape::new(When::Quiet, Gift::Mend, Cost::Frail));
    g.send();
    let run = g.run.as_mut().unwrap();
    let full = crate::hero::Class::Fighter.base_hp();
    assert_eq!(run.hero.max_hp, full - full / 10, "frail: −10 % max hp");
    assert_eq!(run.worn.frail_hp, full / 10);
    let mut n = run.clone();
    traits::neutralize(&mut n);
    assert_eq!(n.hero.max_hp, full);
    assert!(n.worn.is_empty() && n.hero.gift.is_zero());
    let mut t = run.clone();
    t.worn.born = Some(Shape::new(When::Hurt, Gift::Fury, Cost::Thin));
    assert_eq!(traits::heal_pct(&t), traits::THIN_PCT);
    assert_eq!(traits::heal_pct(&n), 100);
    // mend while quiet: hp climbs with no foe in view
    let mut g = arena();
    g.run.as_mut().unwrap().worn.born = Some(Shape::new(When::Quiet, Gift::Mend, Cost::None));
    g.run.as_mut().unwrap().hero.hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    ticks(&mut g, 50);
    assert!(g.run.as_ref().unwrap().hero.hp >= 13, "{}", g.run.as_ref().unwrap().hero.hp);
}

/// §4: `if: trait <head>` and `if: gift live` are gated by the trait's fact, offered by the editor once
/// learned, and read the run's worn traits and the gift's context.
#[test]
fn trait_conditions_are_fact_gated() {
    let mut g = arena();
    let w = Shape::new(When::Hurt, Gift::Guard, Cost::None);
    g.run.as_mut().unwrap().worn.born = Some(w);
    let tc = Cond::t("trait", "stubborn");
    let lc = Cond::flag("gift_live");
    assert!(!g.vocabulary().conds.iter().any(|c| c.k == "trait" || c.k == "gift_live"), "not before the fact");
    {
        let (run, cx) = g.ctx();
        assert!(!traits::cond_holds(run, &cx, &tc), "locked without the fact");
        assert_eq!(traits::cond_reason(&cx, &tc), "locked cond");
    }
    g.lineage.facts.insert("trait:stubborn".into());
    let v = g.vocabulary();
    assert!(v.conds.contains(&tc) && v.conds.contains(&lc));
    assert!(tc.valid() && lc.valid());
    assert_eq!(Row::new(vec![lc.clone()], Verb::arg("attack", "nearest")).describe(), "gift live → attack");
    {
        let (run, cx) = g.ctx();
        assert!(traits::cond_holds(run, &cx, &tc));
        assert!(!traits::cond_holds(run, &cx, &Cond::t("trait", "wrathful")));
        assert!(!traits::cond_holds(run, &cx, &lc), "healthy: the gift is not live");
    }
    // hurt: a `gift live` row fires
    g.run.as_mut().unwrap().hero.hp = 10;
    add_monster(&mut g, "rat", 6, 5);
    rules(&mut g, vec![Row::new(vec![lc], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })), "the `gift live` row acts while hurt");
}

/// c30-legible: every exit line says why the run ended in ≤ 3 words; a fresh lineage's first run banks its new best.
#[test]
fn every_exit_names_its_reason() {
    let mut deaths = 0;
    for seed in [3u64, 1001, 1004] {
        let mut g = crate::Game::new(seed);
        for k in 0..5 {
            g.send();
            let mut line = None;
            for _ in 0..400 {
                let r = g.step(200);
                for e in &r.events {
                    if let crate::Ev::Exit { line: Some(l), .. } = e {
                        line = Some(l.clone());
                    }
                }
                if r.run_over {
                    break;
                }
            }
            g.auto_keep();
            let l = line.expect("an exit line");
            let why = l.reason.clone().expect("a reason");
            assert!(why.split_whitespace().filter(|w| *w != "·").count() <= 3, "{why}");
            if k == 0 {
                assert_eq!(why, "banks every record", "seed {seed}: the first run banks its new best");
            }
            if l.text.starts_with("died") {
                deaths += 1;
                assert!(why.starts_with("slain") || why == "starved", "a death names its killer: {why}");
            }
        }
    }
    assert!(deaths >= 1, "the seeds hold a death (seed 1004's fifth run)");
}
