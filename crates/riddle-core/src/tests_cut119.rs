//! Cut 119 (docs/CUT119_COMPANIONS.md): companions with depth.
use crate::engine::{ExitTier, Game};
use crate::pets::{self, Role};
use crate::wire::Companion;

fn pet(id: u32, kind: &str, name: &str, level: u32) -> Companion {
    let def = crate::defs::monster_def(kind);
    let tags: Vec<String> = def.tags.iter().map(|t| t.to_string()).collect();
    let mut life = pets::PetLife { role: pets::role_of(kind).word().into(), ..Default::default() };
    life.xp = pets::LEVEL_XP[(level.max(1) - 1) as usize];
    Companion { id, kind: kind.into(), name: name.into(), level, tags: tags.clone(), gen: 0, rules: crate::probes::default_companion_rules(&tags, level), max_rows: 2, hp: def.hp, max_hp: def.hp, life }
}

fn camp(seed: u64) -> Game {
    let mut g = Game::new_resident(seed);
    crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
    g.lineage.best_depth = 14;
    for k in ["goblin_warlord", "bloat_mother"] {
        g.lineage.kills.insert(k.into());
    }
    g.lineage.waystones = vec![5, 9];
    g.lineage.gold_move(5_000, "test income");
    g
}

/// A death on `depth` with `carry` lost, the party standing beside the heir.
fn die_with_party(g: &mut Game, depth: u32, carry: i32) -> u32 {
    g.start_run(None);
    g.descend_to(depth);
    let run = g.run.as_mut().unwrap();
    run.loot = carry;
    run.death_cause = Some("ogre".into());
    run.over = Some(ExitTier::Death);
    let id = run.id;
    g.finish_run();
    id
}

/// Item 0: the default tame row compiles below the guard rows and above the fallback, announced once, revocable;
/// pinned off with the systems.
#[test]
fn the_tame_row_compiles_for_everyone_and_is_revocable() {
    let mut g = Game::new_resident(3);
    crate::packages::recompile(&mut g.lineage);
    let rows = &g.lineage.rules().rows;
    let at = rows.iter().position(|r| r.origin.as_deref() == Some(pets::TAME_ORIGIN)).expect("the tame row");
    let attack = rows.iter().position(|r| r.verb.v == "attack" && r.origin.as_deref() == Some("stance:steady")).unwrap();
    let drink = rows.iter().position(|r| r.verb.v == "drink").unwrap();
    assert!(drink < at && at < attack, "below the guard rows, above the fallback: {rows:?}");
    g.revoke_tame(true).unwrap();
    assert!(!g.lineage.rules().rows.iter().any(|r| r.origin.as_deref() == Some(pets::TAME_ORIGIN)));
    assert!(g.lineage.to_wire().pets.unwrap().tame_revoked);
    g.revoke_tame(false).unwrap();
    assert!(g.lineage.rules().rows.iter().any(|r| r.origin.as_deref() == Some(pets::TAME_ORIGIN)));
    // announced once, at the first run's end
    for _ in 0..2 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
        g.finish_run();
        g.auto_keep();
    }
    assert_eq!(g.batch.pkg_lines.iter().filter(|l| l.as_str() == "TAMES STRAYS").count(), 1, "announced once");
    let mut h = Game::new_resident(3);
    crate::pets::pin_off(&mut h.lineage);
    assert!(!h.lineage.rules().rows.iter().any(|r| r.origin.as_deref() == Some(pets::TAME_ORIGIN)));
}

/// Item 0's gate in small: a new resident lineage, sends alone, owns a pet within its first runs on most seeds.
#[test]
fn pets_come_to_everyone() {
    let mut owned = 0;
    for seed in 1..=8u64 {
        let mut g = Game::new_resident(seed);
        for _ in 0..30 {
            g.lineage.rest_left = 0;
            g.start_run(None);
            g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            g.finish_run();
            g.auto_keep();
            if !g.lineage.party.is_empty() || !g.lineage.kennel.is_empty() {
                owned += 1;
                break;
            }
        }
    }
    assert!(owned >= 6, "a pet on {owned}/8 seeds within 30 runs");
}

/// Item 1: a death with a pet standing brings a share of the lost carry home (`fetched`, out of the grave: no gold
/// made), never paying better than a return; the grudge on a boss killer; the reel line names the pet.
#[test]
fn a_pet_fetches_the_fallen_heirs_pack() {
    let mut g = camp(5);
    g.lineage.party = vec![pet(7_000_001, "jackal", "Rook", 1)];
    let gold0 = g.lineage.gold;
    die_with_party(&mut g, 6, 1_000);
    let fetched = g.lineage.gold_tally.get("fetched").copied().unwrap_or(0);
    let grave: i32 = g.lineage.feats.graves.iter().map(|x| x.gold).sum();
    assert!(fetched > 0, "the pet fetched: {:?}", g.lineage.gold_tally);
    assert_eq!(fetched as i32 + grave, 1_000, "the grave holds the rest: no gold made");
    assert!(fetched * 100 < 1_000 * 60 && pets::FETCH_MAX_PCT + 30 < 60, "a death never pays better than a return");
    assert!(g.lineage.gold - gold0 >= fetched as i32);
    assert!(g.lineage.feats.news.iter().any(|n| n.k == "fetched" && n.text.starts_with("Rook brought")), "{:?}", g.lineage.feats.news);
    assert!(g.lineage.party[0].life.fetched > 0);
    // pinned off: nothing fetched, the grave holds it all
    let mut h = camp(5);
    crate::pets::pin_off(&mut h.lineage);
    h.lineage.party = vec![pet(7_000_001, "jackal", "Rook", 1)];
    die_with_party(&mut h, 6, 1_000);
    assert_eq!(h.lineage.gold_tally.get("fetched").copied().unwrap_or(0), 0);
}

/// Item 1 (research §1): a pet killed is lamed, its level kept — it sits out `LAME_RUNS` runs; gone only at its
/// `GONE_FALLS`-th fall, with a named cause.
#[test]
fn a_pet_goes_down_not_gone() {
    let mut g = camp(6);
    g.lineage.party = vec![pet(7_000_002, "ogre", "Greth", 4)];
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    for m in run.monsters.iter_mut().filter(|m| m.cid.is_some()) {
        m.hp = 0;
    }
    run.fell_why.push(("Greth".into(), "fell D1 to goblin".into()));
    run.over = Some(ExitTier::Return);
    g.finish_run();
    let c = g.lineage.party.iter().find(|c| c.name == "Greth").expect("still the line's");
    assert_eq!(c.level, 4, "level kept");
    assert_eq!(c.life.lame, pets::LAME_RUNS);
    assert!(g.batch.fallen.iter().any(|f| f.name == "Greth" && f.lamed && !f.why.is_empty()));
    // lamed: not fielded; a run nearer the field each run
    g.start_run(None);
    assert!(!g.run.as_ref().unwrap().monsters.iter().any(|m| m.cid == Some(7_000_002)));
    g.run.as_mut().unwrap().over = Some(ExitTier::Return);
    g.finish_run();
    assert_eq!(g.lineage.party[0].life.lame, pets::LAME_RUNS - 1);
}

/// Item 3: xp from every run, levels to L7, the role's signatures at L3 and L5 announced once, bench pets a quarter.
#[test]
fn levels_from_every_run_with_signatures() {
    assert_eq!(pets::level_for(0), 1);
    assert_eq!(pets::level_for(u32::MAX), pets::CAP);
    let mut g = camp(7);
    let mut c = pet(1, "skeleton", "Bram", 2);
    c.life.xp = pets::LEVEL_XP[2] - 1;
    g.start_run(None);
    g.run.as_mut().unwrap().max_depth = 9;
    let run = g.run.clone().unwrap();
    let lines = pets::grow(&mut c, &run, ExitTier::Death);
    assert_eq!(c.level, 3);
    assert_eq!(lines, vec!["Bram L3 · taunt".to_string()]);
    assert!(pets::grow(&mut c, &run, ExitTier::Return).is_empty(), "announced once");
    assert_eq!(pets::role(&c), Role::Guard);
    assert_eq!(pets::signature(Role::Mender, 5), Some("revive once"));
    // real stats: hp and attack grow with the level
    let lo = pet(2, "skeleton", "A", 1);
    let hi = pet(3, "skeleton", "B", 6);
    assert!(crate::engine::pet_max_hp(&hi, 1) > crate::engine::pet_max_hp(&lo, 1));
    let (ma, mb) = (crate::engine::companion_monster(1, &lo, crate::geom::Pos::new(0, 0)), crate::engine::companion_monster(2, &hi, crate::geom::Pos::new(0, 0)));
    assert!(mb.atk.1 > ma.atk.1);
    // the bench: a quarter
    g.lineage.kennel = vec![pet(4, "rat", "Nib", 1)];
    let _ = pets::bench(&mut g.lineage, &run);
    assert_eq!(g.lineage.kennel[0].life.xp, 9 * pets::BENCH_PCT / 100);
}

/// Item 4: the kennel keeper's order breeds while away — the generation in the name, the pedigree in the stats,
/// surplus released — with no manual breed; `off` breeds nothing.
#[test]
fn the_keeper_breeds_and_releases() {
    let mut g = camp(8);
    g.lineage.party = vec![pet(11, "skeleton", "Rook", 3)];
    g.lineage.kennel = vec![pet(12, "jackal", "Fang", 2)];
    for _ in 0..pets::BREED_EVERY {
        g.lineage.rest_left = 0;
        g.start_run(None);
        g.run.as_mut().unwrap().over = Some(ExitTier::Return);
        g.finish_run();
    }
    let egg = g.lineage.eggs.iter().find(|e| !e.from_loss).expect("a bred egg").clone();
    assert_eq!(egg.gen, 1);
    assert!(!egg.sire.is_empty());
    for _ in 0..6 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        g.run.as_mut().unwrap().over = Some(ExitTier::Return);
        g.finish_run();
    }
    let pup = g.lineage.all_companions().find(|c| c.life.bred).expect("hatched").clone();
    assert_eq!(pup.name, format!("{} II", egg.sire));
    // (gate fix: the `bred` line names the generation the pup hatches as — `Rook egg · II` hatches `Rook II`)
    let bred: Vec<&str> = g.lineage.feats.news.iter().filter(|n| n.k == "bred").map(|n| n.text.as_str()).collect();
    assert!(bred.contains(&format!("{} egg · II", egg.sire).as_str()), "bred lines {bred:?}");
    assert!(pets::max_hp(&pup, 100) > pets::max_hp(&Companion { gen: 0, ..pup.clone() }, 100), "pedigree");
    assert!(g.lineage.party.len() + g.lineage.kennel.len() <= g.lineage.party_slots() as usize + pets::KENNEL_SPARE);
    let mut h = camp(8);
    h.lineage.orders.kennel = "off".into();
    h.lineage.party = vec![pet(11, "skeleton", "Rook", 3)];
    for _ in 0..pets::BREED_EVERY {
        h.lineage.rest_left = 0;
        h.start_run(None);
        h.run.as_mut().unwrap().over = Some(ExitTier::Return);
        h.finish_run();
    }
    assert!(h.lineage.eggs.iter().all(|e| e.from_loss));
    assert_eq!(pets::base_name("Rook III"), "Rook");
    assert_eq!(pets::roman(4), "IV");
}

/// Item 5: a pet synergy forms from the role and the build alone (no level gate), names the build, once.
#[test]
fn pet_synergies_form_from_role_and_build() {
    let mut g = camp(9);
    g.lineage.party = vec![pet(21, "goblin_archer", "Skix", 1)];
    assert!(pets::synergies(&g.lineage).is_empty());
    g.lineage.pkg.owned.insert("kite_archers".into());
    g.lineage.pkg.tactics = vec!["kite_archers".into()];
    crate::packages::recompile(&mut g.lineage);
    let s: Vec<&str> = pets::synergies(&g.lineage).iter().map(|s| s.id).collect();
    assert_eq!(s, vec!["falconer"]);
    assert_eq!(g.lineage.to_wire().packages.build.and_then(|b| b.pet_synergy).as_deref(), Some("Falconer"));
    g.start_run(None);
    assert_ne!(g.run.as_ref().unwrap().pets.syn & pets::syn_bit("falconer"), 0);
}

/// The role spread holds at the kinds: every role has kinds, and the strays' first kinds cover the four roles.
#[test]
fn roles_are_spread() {
    let firsts: std::collections::BTreeSet<Role> = (1..=40u64).map(|s| pets::role_of(pets::first_kind(s))).collect();
    assert_eq!(firsts.len(), 4);
    for r in pets::ROLES {
        assert!(crate::defs::MONSTERS.iter().filter(|m| !m.boss).any(|m| pets::role_of(m.kind) == r));
    }
}

/// E: the 307dbed send hash moves with Cut 119 on (the default tame row tames the first stray, a pet walks every
/// send); with the companions pinned off it is Cut 118's 2f3eb706b3d24c7c exactly.
#[test]
fn the_307dbed_hash_restores_with_pets_pinned_off() {
    let mut g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    crate::pets::pin_off(&mut g.lineage);
    assert_eq!(format!("{:016x}", crate::tests::sends_hash(&mut g, 10)), "2f3eb706b3d24c7c");
}

/// Gate fix (the coordinator): the report's `heir` line names the trait the heir actually wears — read after the
/// wake's temperament card is worn (`packages::wear` rewrites the born trait), never the order's earlier card.
#[test]
fn the_heir_line_names_the_worn_trait() {
    for seed in 1..=12u64 {
        let mut g = camp(seed);
        g.lineage.heir = 4;
        g.lineage.graveyard.push(crate::wire::Grave { heir: 4, depth: 13, cause: "bloat_mother".into(), deeds: Vec::new(), death_id: None });
        g.lineage.new_heir();
        assert!(g.lineage.pkg.temperament.is_some(), "seed {seed}: a temperament card worn");
        let born = g.lineage.heirs.born.expect("born");
        let line = g.lineage.feats.news.iter().rev().find(|n| n.k == "heir").expect("an heir line").text.clone();
        assert!(line.contains(&born.chip()), "seed {seed}: {line} vs worn {}", born.chip());
    }
}
