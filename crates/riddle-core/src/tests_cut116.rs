//! Cut 116 (docs/CUT116_VARIED_DESCENT.md): the descent varies by heir — band-boss affixes, the D9
//! and D14 forks open, a wandering champion once per band at most.
use crate::descent::{affix_pool, boss_affix, guest_for, heir_affixes, Affix, Route, BANDS, BOSS_DEPTHS};
use crate::engine::{populate_floor, Game};

/// A run of `g`'s on `depth`, its floor populated afresh.
fn floor_at(g: &mut Game, depth: u32) -> crate::engine::Run {
    g.start_run(Some(3));
    let mut run = g.run.take().unwrap();
    run.depth = depth;
    run.monsters.clear();
    populate_floor(&mut run, &[], &Default::default(), None);
    run
}

/// §1: the draw is pure (same lineage seed and heir ⇒ same descent), varies by heir, stays in each
/// boss's pool, and leaves the Mirror King bare.
#[test]
fn affixes_are_drawn_per_heir_and_deterministic() {
    for seed in [1u64, 7, 307] {
        assert_eq!(heir_affixes(seed, 3), heir_affixes(seed, 3));
        for (k, _) in BOSS_DEPTHS {
            for heir in 1..40 {
                match boss_affix(seed, heir, k) {
                    Some(a) => assert!(affix_pool(k).contains(&a), "{k}: {a:?}"),
                    None => assert_eq!(k, "mirror_king"),
                }
            }
            if k != "mirror_king" {
                let seen: std::collections::BTreeSet<Affix> = (1..40).filter_map(|h| boss_affix(seed, h, k)).collect();
                assert!(seen.len() >= 3, "{k} varies by heir: {seen:?}");
            }
        }
        let descents: std::collections::BTreeSet<Vec<(String, Affix)>> = (1..20).map(|h| heir_affixes(seed, h)).collect();
        assert!(descents.len() >= 15, "heirs meet different descents");
    }
    let g = Game::new_resident(5);
    assert_eq!(g.lineage.affixes(), heir_affixes(5, g.lineage.heir));
}

/// §1: the boss spawns with his affix — its mechanics, its hp price, the wire's label (the entity's
/// modifiers, the floor chart's row, the lineage's list) — and a pinned plain boss is untouched.
#[test]
fn a_boss_carries_his_affix_onto_the_floor_and_the_wire() {
    let mut g = Game::new_resident(11);
    g.lineage.affix_pin = Some(Vec::new());
    let plain = floor_at(&mut g, 8);
    let p = plain.monsters.iter().find(|m| m.kind == "goblin_warlord").expect("the Warlord");
    assert!(p.modifiers.is_none());
    for a in affix_pool("goblin_warlord") {
        let mut g = Game::new_resident(11);
        g.lineage.affix_pin = Some(vec![("goblin_warlord".into(), *a)]);
        let run = floor_at(&mut g, 8);
        let m = run.monsters.iter().find(|m| m.kind == "goblin_warlord").unwrap();
        assert_eq!(m.modifiers.and_then(|x| x.affix), Some(*a));
        assert_eq!(m.max_hp, p.max_hp * a.hp_pct() / 100);
        match a {
            Affix::Armoured => assert_eq!(m.def, p.def + 1),
            Affix::Swift => assert_eq!(m.speed, p.speed + 2),
            Affix::Enraged => assert_eq!(m.modifiers.unwrap().elite, Some(crate::endgame::Elite::Frenzied)),
            _ => {}
        }
        let wire = g.lineage.to_wire();
        assert!(wire.affixes.iter().any(|x| x.boss == "goblin_warlord" && x.depth == 8 && x.affix == a.word()));
    }
    // the floor chart names it on his floor
    let mut g = Game::new_resident(11);
    g.lineage.affix_pin = Some(vec![("goblin_warlord".into(), Affix::Swift)]);
    g.lineage.best_depth = 8;
    let f = g.forecast();
    assert_eq!(f.depths.iter().find(|d| d.depth == 8).and_then(|d| d.affix.clone()).as_deref(), Some("swift"));
    assert!(f.depths.iter().filter(|d| d.depth != 8).all(|d| d.affix.is_none()));
    // the wire's entity carries it (the boss bar's `Warlord · swift`)
    let run = floor_at(&mut g, 8);
    let m = run.monsters.iter().find(|m| m.kind == "goblin_warlord").unwrap();
    let e = crate::engine::monster_entity(m, &Default::default());
    assert_eq!(serde_json::to_value(&e).unwrap()["modifiers"]["affix"], "swift");
}

/// §1: a regenerating boss heals at tier 0 (the tier's own mechanic carries the affix).
#[test]
fn a_regenerating_affix_heals_at_tier_zero() {
    for (pin, heals) in [(Some(Affix::Regenerating), true), (None, false)] {
        let mut g = Game::new_resident(11);
        g.lineage.affix_pin = Some(pin.map(|a| ("goblin_warlord".to_string(), a)).into_iter().collect());
        let mut run = floor_at(&mut g, 8);
        let mi = run.monsters.iter().position(|m| m.kind == "goblin_warlord").unwrap();
        run.monsters[mi].hp = 5;
        run.monsters[mi].awake = true;
        g.run = Some(run);
        let (r, mut cx) = g.ctx();
        crate::turn::tick_regen_and_auras(r, &mut cx);
        assert_eq!(r.monsters[mi].hp > 5, heals, "{pin:?}");
    }
}

/// §1: a brood boss keeps a fuller guard than a plain one (over seeds).
#[test]
fn a_brood_boss_keeps_a_fuller_guard() {
    let escorts = |pin: Vec<(String, Affix)>| -> usize {
        (1..=12u64)
            .map(|s| {
                let mut g = Game::new_resident(s);
                g.lineage.affix_pin = Some(pin.clone());
                floor_at(&mut g, 8).monsters.iter().filter(|m| m.kind == "goblin" && !m.summoned).count()
            })
            .sum()
    };
    assert!(escorts(vec![("goblin_warlord".into(), Affix::Brood)]) > escorts(Vec::new()));
}

/// §2: the D9 and D14 forks are open to every lineage.
#[test]
fn the_d9_and_d14_forks_are_open() {
    for f in [5, 9, 14] {
        assert!(crate::descent::fork_open_for(false, f), "D{f}");
    }
    assert!(!crate::descent::fork_open_for(false, 19) && !crate::descent::fork_open_for(false, 24));
    let r = Route::from_forks(&[14]).unwrap();
    assert_eq!((r.biome(14).name(), r.biome(19).name()), ("foundry", "crypt"));
}

/// §3: a guest stands strictly inside a band (never the band's first floor nor its boss's), on a draw
/// as pure as the affixes, about half the heirs × bands; a populated floor holds it, named and tagged.
#[test]
fn a_wandering_champion_walks_a_non_boss_floor() {
    let mut met = 0;
    let mut n = 0;
    for heir in 1..60 {
        for (b, (first, last)) in BANDS.iter().enumerate() {
            n += 1;
            if let Some(g) = guest_for(9, heir, Route::BASE, b) {
                met += 1;
                assert!(g.depth > *first && g.depth < *last, "{g:?}");
                assert_eq!(Some(g.clone()), guest_for(9, heir, Route::BASE, b));
                assert!(Route::BASE.boss(g.depth).is_none());
                assert!(!crate::defs::monster_def(&g.kind).tags.contains(&"water"));
            }
        }
    }
    assert!(met * 100 > n * 35 && met * 100 < n * 65, "{met}/{n}");
    // a lineage whose heir meets one: the floor holds him, named, the wire tags him
    let mut seed = 1;
    let (game, guest) = loop {
        let g = Game::new_resident(seed);
        if let Some(x) = g.lineage.guests(Route::BASE).into_iter().next() {
            break (g, x);
        }
        seed += 1;
    };
    let mut game = game;
    let run = floor_at(&mut game, guest.depth);
    let m = run.monsters.iter().find(|m| m.guest).expect("the guest on his floor");
    assert_eq!((m.kind.as_str(), m.name.as_deref()), (guest.kind.as_str(), Some(guest.name.as_str())));
    assert!(crate::engine::monster_entity(m, &Default::default()).tags.contains(&"guest".to_string()));
    // switched off (a probe's pin): no guest
    game.lineage.guest_pin = Some(false);
    assert!(game.lineage.guests(Route::BASE).is_empty());
}

/// §3: a guest slain is a beat — the note, the run's news, the reel's episode, the heir's deed.
#[test]
fn a_slain_guest_is_a_story_beat() {
    let mut seed = 1;
    let (mut g, guest) = loop {
        let g = Game::new_resident(seed);
        if let Some(x) = g.lineage.guests(Route::BASE).into_iter().next() {
            break (g, x);
        }
        seed += 1;
    };
    let run = floor_at(&mut g, guest.depth);
    g.run = Some(run);
    let mi = g.run.as_ref().unwrap().monsters.iter().position(|m| m.guest).unwrap();
    {
        let (r, mut cx) = g.ctx();
        let hp = r.monsters[mi].hp;
        crate::turn::damage_monster(r, &mut cx, mi, hp + 100, &crate::turn::Src::Fire);
    }
    let r = g.run.as_ref().unwrap();
    assert_eq!(r.guests_slain, vec![guest.name.clone()]);
    assert!(r.episodes.iter().any(|e| matches!(&e.resolution, crate::sifter::Resolution::Champion { name, .. } if *name == guest.name)));
    let news = g.run_news(r, crate::engine::ExitTier::Return, 0);
    assert!(news.iter().any(|n| n.text == format!("slew {}", guest.name)), "{news:?}");
    let line = crate::sifter::story_line(r.episodes.iter().find(|e| matches!(e.resolution, crate::sifter::Resolution::Champion { .. })).unwrap());
    assert!(crate::sifter::story_ok(&line), "{line}");
}
