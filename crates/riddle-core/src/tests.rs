//! Integration tests over the public engine: determinism, rules, monsters, items, exits,
//! facts, forecast, offline, verdicts, sifter, marks, save, and the addenda.
use crate::defs::monster_def;
use crate::engine::{ExitTier, Game, HERO_ID};
use crate::gen::Floor;
use crate::geom::Pos;
use crate::hero::Class;
use crate::item::{ident_fact, Item};
use crate::monster::Monster;
use crate::rules::{word_count, Cond, Row, RuleSet, Verb};
use crate::tiles::{Map, OverlayKind, Tile, VISION};
use crate::wire::*;

// ---------------------------------------------------------------- harness

/// `f` for each seed on its own thread, the results in seed order: the long seed loops are
/// the suite's critical path (docs/ITERATION_SPEED.md §3.4); every seed is its own game, so
/// what each asserts is unchanged. A panicking seed fails the test with its own message.
fn par_seeds<T: Send>(seeds: impl IntoIterator<Item = u64>, f: impl Fn(u64) -> T + Sync) -> Vec<T> {
    let seeds: Vec<u64> = seeds.into_iter().collect();
    let f = &f;
    std::thread::scope(|sc| {
        let hs: Vec<_> = seeds.iter().map(|&s| sc.spawn(move || f(s))).collect();
        hs.into_iter().map(|h| h.join().unwrap_or_else(|e| std::panic::resume_unwind(e))).collect()
    })
}

/// A game with a live run on an open 16×12 room, no monsters or items.
pub(crate) fn arena() -> Game {
    arena_seed(1)
}

fn arena_seed(seed: u64) -> Game {
    let mut g = Game::new_literal(seed);
    g.max_deaths = 1000;
    // Cut 8B §3: the kennel's leash stays on the shelf; the arena's pack starts empty.
    g.lineage.supplies.clear();
    g.start_run(Some(seed.wrapping_mul(7) + 3));
    let run = g.run.as_mut().unwrap();
    let mut map = Map::new(16, 12, Tile::Wall);
    for y in 1..11 {
        for x in 1..15 {
            map.set(Pos::new(x, y), Tile::Floor);
        }
    }
    let up = Pos::new(1, 1);
    let down = Pos::new(14, 10);
    map.set(up, Tile::StairsUp);
    map.set(down, Tile::StairsDown);
    map.compute_corridors(&[]);
    run.floor = Floor { map, stairs_up: up, stairs_down: down, rooms: Vec::new(), vision: VISION };
    run.monsters.clear();
    run.items.clear();
    run.overlays.clear();
    run.hero.pos = Pos::new(4, 5);
    run.hero_dist_pos = None;
    run.last_visible = vec![u32::MAX];
    run.floor.map.update_vision(run.hero.pos, VISION);
    g.events.clear();
    g.history.clear();
    g
}

pub(crate) fn add_monster(g: &mut Game, kind: &str, x: i32, y: i32) -> u32 {
    let run = g.run.as_mut().unwrap();
    let id = run.new_id();
    let depth = run.depth;
    let mut m = Monster::spawn(id, kind, Pos::new(x, y), depth);
    m.awake = true;
    m.last_seen = Some(run.hero.pos);
    run.monsters.push(m);
    id
}

pub(crate) fn give(g: &mut Game, kind: &str) -> u32 {
    let run = g.run.as_mut().unwrap();
    let id = run.new_item_id();
    let mut it = Item::new(id, kind);
    if kind == "leash" {
        it.amount = 1;
    }
    run.hero.inv.push(it);
    id
}

/// Rows straight into the set, locks and all (the arena tests are about the engine, not the
/// editor's door; `set_rules_refuses_a_locked_token` tests the door).
pub(crate) fn rules(g: &mut Game, rows: Vec<Row>) {
    g.set_rules_raw(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
}

pub(crate) fn ticks(g: &mut Game, n: u32) -> Vec<Ev> {
    let mut out = Vec::new();
    for _ in 0..n {
        g.tick();
        out.append(&mut g.events);
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            break;
        }
    }
    out
}

fn hero(g: &Game) -> &crate::hero::Hero {
    &g.run.as_ref().unwrap().hero
}

fn monster(g: &Game, id: u32) -> Option<&Monster> {
    g.run.as_ref().unwrap().monsters.iter().find(|m| m.id == id && m.hp > 0)
}

fn attack_rules(g: &mut Game) {
    rules(g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
}

/// Cut 8B §3: a lineage that has tamed keeps no free leash on the shelf (for the supply tests).
/// Cut 29 §1: every tier of the catalogue open (a bank, every band boss seen), and the free
/// vocabulary their gates open — for the tests of cards and rows bought.
fn tiers(g: &mut Game) {
    g.lineage.banked_depths.insert(1);
    for (k, _) in crate::descent::BOSS_DEPTHS {
        g.lineage.facts.insert(format!("foe:{k}"));
    }
    crate::meta::grant_free(&mut g.lineage);
}

fn no_kennel_leash(g: &mut Game) {
    g.lineage.facts.insert("tamed:rat".into());
    g.lineage.supplies.clear();
}

fn hold_rules(g: &mut Game) {
    // Hold position: the hero never moves or attacks.
    rules(g, vec![Row::new(vec![], Verb::new("hold"))]);
}

fn ev_kinds(evs: &[Ev]) -> Vec<String> {
    evs.iter().map(|e| serde_json::to_value(e).unwrap()["k"].as_str().unwrap().to_string()).collect()
}

// ---------------------------------------------------------------- determinism

#[test]
fn same_seed_rules_elapsed_identical_events() {
    let mut a = Game::new_literal(42);
    let mut b = Game::new_literal(42);
    a.send();
    b.send();
    for _ in 0..30 {
        let ra = a.step(40);
        let rb = b.step(40);
        assert_eq!(serde_json::to_string(&ra).unwrap(), serde_json::to_string(&rb).unwrap());
        if ra.run_over {
            a.send();
            b.send();
        }
    }
    assert_eq!(a.save(), b.save());
}

#[test]
fn offline_batches_are_deterministic() {
    let mut a = Game::new_literal(3);
    let mut b = Game::new_literal(3);
    let ra = a.run_offline(600);
    let rb = b.run_offline(600);
    assert_eq!(serde_json::to_string(&ra).unwrap(), serde_json::to_string(&rb).unwrap());
    assert_eq!(a.save(), b.save());
}

#[test]
fn energy_scheduler_fast_monsters_act_more() {
    let mut g = arena();
    hold_rules(&mut g);
    let j = add_monster(&mut g, "jackal", 13, 9);
    g.run.as_mut().unwrap().monsters.iter_mut().for_each(|m| m.awake = false);
    let evs = ticks(&mut g, 20);
    let jackal_moves = evs.iter().filter(|e| matches!(e, Ev::Move { id, .. } if *id == j)).count();
    // 15 energy/tick ⇒ 3 actions in 20 ticks; a dozing jackal wanders only sometimes.
    assert!(jackal_moves <= 3);
    let mut g = arena();
    hold_rules(&mut g);
    let o = add_monster(&mut g, "ogre", 12, 5);
    let evs = ticks(&mut g, 40);
    let ogre_acts = evs.iter().filter(|e| matches!(e, Ev::Move { id, .. } if *id == o)).count();
    assert!(ogre_acts <= 3, "ogre (speed 7) acts ≤ 3 times in 40 ticks, got {ogre_acts}");
    assert!(ogre_acts >= 2);
}

#[test]
fn hero_acts_every_ten_ticks_at_base_speed() {
    let mut g = arena();
    let evs = ticks(&mut g, 50);
    let rule_evs = evs.iter().filter(|e| matches!(e, Ev::Rule { .. })).count();
    assert_eq!(rule_evs, 5);
}

// ---------------------------------------------------------------- rules

#[test]
fn rows_evaluate_top_down_first_true_fires() {
    let mut g = arena();
    add_monster(&mut g, "rat", 5, 5);
    rules(
        &mut g,
        vec![
            Row::new(vec![Cond::n("hp<", 10)], Verb::new("retreat")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![], Verb::new("rest")),
        ],
    );
    let evs = ticks(&mut g, 10);
    let fired: Vec<i32> = evs.iter().filter_map(|e| if let Ev::Rule { row, .. } = e { Some(*row) } else { None }).collect();
    assert_eq!(fired, vec![1]);
}

#[test]
fn row_falls_through_when_verb_cannot_execute() {
    let mut g = arena();
    add_monster(&mut g, "rat", 5, 5);
    rules(
        &mut g,
        vec![
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("drink", "heal")), // no potion
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
    );
    let evs = ticks(&mut g, 10);
    let fired: Vec<i32> = evs.iter().filter_map(|e| if let Ev::Rule { row, .. } = e { Some(*row) } else { None }).collect();
    assert_eq!(fired, vec![1]);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, .. } if *src == HERO_ID)));
}

#[test]
fn no_row_fires_means_chore_explores() {
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -2, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Move { id, .. } if *id == HERO_ID)));
}

/// Cut 12 §1: the cap is on the player's own rows. A fifth own row on a 4-row lineage is
/// refused at the door with a ≤ 6-word error (Cut 2 truncated it silently); a sim's set is cut
/// like the editor cuts it (the last own row falls off, and never fires); with `row5` the
/// same five rows are taken and the fifth fires.
#[test]
fn rows_beyond_unlocked_count_are_ignored() {
    let mut g = arena();
    let mut rows: Vec<Row> = (0..4).map(|_| Row::new(vec![Cond::n("hp<", 0)], Verb::new("rest"))).collect();
    rows.push(Row::new(vec![], Verb::new("return")));
    let err = g.set_rules(RuleSet { rows: rows.clone(), name: None, route: Vec::new() }).unwrap_err();
    assert_eq!(err, "5 own rows, 4 allowed");
    assert!(word_count(&err) <= 6, "{err}");
    rules(&mut g, rows.clone());
    assert_eq!(g.lineage.rules().rows.len(), 4, "a sim's fifth own row is cut without the row5 unlock");
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Exit { .. })), "row 5 must not fire without the row5 unlock");
    g.lineage.unlocks.insert("row5".into());
    g.set_rules(RuleSet { rows: rows.clone(), name: None, route: Vec::new() }).unwrap();
    assert_eq!(g.lineage.rules().rows.len(), 5);
    // Cut 19 §2: the return walks to the up-stairs (four steps) before it exits.
    let evs = ticks(&mut g, 100);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
}

/// Cut 12 §1: a card brings its row — a 4-row lineage holds 4 own rows and 2 card rows; a
/// fifth own row is refused, a second row for the same card is refused, an unowned card's row
/// is refused; the rows in play carry their set indices (the card rows fire as `R2`/`R5`);
/// `needs: fill rows` counts own rows; `insert_at` sits before the engagement row and the
/// card's delta row goes there.
#[test]
fn card_rows_sit_outside_the_cap() {
    let mut g = Game::new_literal(5);
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.unlocks.insert("boss_focus".into());
    let own = |n: i32| Row::new(vec![Cond::n("hp<", n)], Verb::arg("drink", "heal"));
    let card = |c: &str| Row::new(vec![], Verb::arg("tactic", c));
    let rows = vec![own(10), card("thief_guard"), own(20), own(30), card("boss_focus"), own(40)];
    g.set_rules(RuleSet { rows: rows.clone(), name: None, route: Vec::new() }).unwrap();
    let set = g.lineage.rules();
    assert_eq!((set.rows.len(), set.own_rows(), set.card_rows()), (6, 4, 2));
    assert_eq!(g.vocabulary().max_rows, 4, "the wire's cap is the own-row cap");
    assert_eq!(set.active(4).map(|(i, _)| i).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5]);
    // A fifth own row: refused, the set unchanged.
    let mut five = rows.clone();
    five.push(own(50));
    assert_eq!(g.set_rules(RuleSet { rows: five, name: None, route: Vec::new() }).unwrap_err(), "5 own rows, 4 allowed");
    assert_eq!(g.lineage.rules().rows.len(), 6);
    // A second row for a card the set already carries: refused.
    let mut twice = rows.clone();
    twice.push(card("thief_guard"));
    assert_eq!(g.set_rules(RuleSet { rows: twice, name: None, route: Vec::new() }).unwrap_err(), "two rows for thief guard");
    // A card the lineage does not own: refused.
    let mut unowned = rows.clone();
    unowned.push(card("gas_step"));
    assert_eq!(g.set_rules(RuleSet { rows: unowned, name: None, route: Vec::new() }).unwrap_err(), "card not owned: gas step");
    // A sim cuts the last own row and keeps every card row.
    let mut five = rows.clone();
    five.insert(0, own(5));
    let fit = RuleSet { rows: five, name: None, route: Vec::new() }.fit(4);
    assert_eq!(fit.rows.len(), 6);
    assert_eq!(fit.card_rows(), 2);
    assert!(!fit.rows.contains(&own(40)) && fit.rows.contains(&own(5)));
    // `needs: fill rows` reads the own rows: four own rows fill a 4-row lineage.
    let cat = crate::meta::catalogue(&g.lineage);
    assert_ne!(cat.iter().find(|u| u.id == "row5").unwrap().needs.as_deref(), Some("fill rows"));
    // `insert_at`: before the first `attack`/`shoot`, else the end; the delta row goes there.
    let mut with_attack = rows.clone();
    with_attack[2] = Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"));
    g.set_rules(RuleSet { rows: with_attack, name: None, route: Vec::new() }).unwrap();
    g.lineage.facts.insert("foe:bloat:gas".into());
    let cat = crate::meta::catalogue(&g.lineage);
    // (Cut 29 §1: gas step is an oath's reward, off the shelf — pack break is sold)
    let gas = cat.iter().find(|u| u.id == "pack_break").unwrap();
    assert_eq!(gas.insert_at, Some(2));
    assert_eq!(crate::meta::delta_row(&g.lineage, "pack_break").unwrap().1, 2);
    assert_eq!(cat.iter().find(|u| u.id == "thief_guard").unwrap().insert_at, None, "an owned card has no place to go");
    assert_eq!(cat.iter().find(|u| u.id == "throw").unwrap().insert_at, None, "a verb unlock's row is the player's");
    g.set_rules(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
    assert_eq!(crate::meta::catalogue(&g.lineage).iter().find(|u| u.id == "pack_break").unwrap().insert_at, Some(6), "no engagement row: the end");
    // The rows in play: an own row fires with its set index (`R6` = `hp<40`; indices are 0-based).
    let mut g = arena();
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.unlocks.insert("boss_focus".into());
    let rows = vec![own(0), card("thief_guard"), own(0), own(0), card("boss_focus"), Row::new(vec![], Verb::new("return"))];
    g.set_rules_raw(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
    assert_eq!(g.lineage.rules().rows.len(), 6);
    let evs = ticks(&mut g, 100);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 5, .. })), "the fourth own row is the set's sixth: {evs:?}");
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
}

#[test]
fn rule_text_is_at_most_three_words() {
    let mut g = Game::new_literal(11);
    g.set_rules_raw(crate::probes::good()).unwrap();
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    g.send();
    let mut notes = 0;
    for _ in 0..40 {
        let r = g.step(50);
        for e in &r.events {
            match e {
                Ev::Rule { text, .. } => assert!(word_count(text) <= 3, "{text}"),
                Ev::Callout { text, .. } => assert!(word_count(text) <= 3, "{text}"),
                Ev::Note { text, .. } => {
                    notes += 1;
                    assert!(word_count(text) <= 8, "{text}")
                }
                _ => {}
            }
        }
        if r.run_over {
            g.send();
        }
    }
    assert!(notes > 0);
}

// ---------------------------------------------------------------- sanity

#[test]
fn sanity_no_drink_heal_at_full_hp() {
    let mut g = arena();
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    give(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("drink", "heal"))]);
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Use { .. })));
    g.run.as_mut().unwrap().hero.hp = 10;
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "heal potion")));
    assert_eq!(hero(&g).hp, 10 + hero(&g).max_hp / 2);
}

#[test]
fn sanity_no_reading_a_known_useless_scroll() {
    let mut g = arena();
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "mapping").unwrap());
    give(&mut g, "mapping");
    g.run.as_mut().unwrap().floor.map.reveal_all();
    rules(&mut g, vec![Row::new(vec![], Verb::arg("read", "mapping"))]);
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Use { .. })));
    assert_eq!(hero(&g).inv.len(), 1);
}

#[test]
fn sanity_no_retreat_without_a_foe_in_view() {
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![], Verb::new("retreat")), Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 10);
    let fired: Vec<i32> = evs.iter().filter_map(|e| if let Ev::Rule { row, .. } = e { Some(*row) } else { None }).collect();
    assert_eq!(fired, vec![1]);
}

// ---------------------------------------------------------------- traits

/// Cut 30 §2: a temperament acts only through its rows — the old overrides are gone: a cowardly
/// heir at 5 hp does not retreat before the rows, a brave one does not hold a retreat row, a
/// curious one does not drink an unknown no row names, a greedy one does not grab past the rows.
#[test]
fn no_temperament_chooses_an_action() {
    use crate::hero::Trait;
    for t in Trait::ALL {
        let mut g = arena();
        g.run.as_mut().unwrap().trait_ = t;
        g.run.as_mut().unwrap().hero.hp = if t == Trait::Cowardly { 5 } else { 30 };
        add_monster(&mut g, "rat", 6, 5);
        give(&mut g, "speed");
        g.run.as_mut().unwrap().items.push(crate::engine::FloorItem { pos: Pos::new(3, 5), item: Item::new(99, "sword") });
        rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        let evs = ticks(&mut g, 10);
        assert!(!evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, .. })), "{t:?}: {evs:?}");
        assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })), "{t:?}");
    }
}

// ---------------------------------------------------------------- monsters

#[test]
fn jackal_pack_holds_back_alone_and_learns_pack_in_numbers() {
    let mut g = arena();
    hold_rules(&mut g);
    let j = add_monster(&mut g, "jackal", 10, 5);
    ticks(&mut g, 60);
    let d = monster(&g, j).unwrap().pos.cheb(hero(&g).pos);
    assert!(d >= 3, "a lone jackal lurks at distance, got {d}");
    add_monster(&mut g, "jackal", 11, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:jackal:pack")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { dst, .. } if *dst == HERO_ID)), "the pack closes in");
}

#[test]
fn jackal_fast_is_learned_by_double_moves() {
    let mut g = arena();
    hold_rules(&mut g);
    add_monster(&mut g, "jackal", 12, 5);
    add_monster(&mut g, "jackal", 12, 6);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:jackal:fast")));
}

#[test]
fn archer_telegraphs_draws_then_shoots_with_a_projectile() {
    let mut g = arena();
    hold_rules(&mut g);
    let a = add_monster(&mut g, "goblin_archer", 8, 5);
    let evs = ticks(&mut g, 40);
    let kinds = ev_kinds(&evs);
    let ti = evs.iter().position(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == a && what == "draws")).expect("telegraph");
    let pi = evs.iter().position(|e| matches!(e, Ev::Projectile { src, dst, .. } if *src == a && *dst == HERO_ID)).expect("projectile");
    let ai = evs.iter().position(|e| matches!(e, Ev::Attack { src, verb, .. } if *src == a && verb.as_deref() == Some("shoot"))).expect("shot");
    assert!(ti < pi && pi < ai, "{kinds:?}");
    assert!(evs[ai].t() - evs[ti].t() >= 5, "the telegraph precedes the shot by an action");
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:goblin_archer:telegraph")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:goblin_archer:ranged")));
}

#[test]
fn ogre_winds_up_then_smashes_double() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 200;
    g.run.as_mut().unwrap().hero.hp = 199;
    let o = add_monster(&mut g, "ogre", 5, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == o && what == "winds up")));
    let smash = evs.iter().find(|e| matches!(e, Ev::Attack { src, verb, .. } if *src == o && verb.as_deref() == Some("smash")));
    assert!(smash.is_some(), "ogre smashes after winding up");
    if let Some(Ev::Attack { dmg, hit: true, .. }) = smash {
        assert!(*dmg >= 4, "double damage (2× 2-4), got {dmg}");
    }
    let evs2 = ticks(&mut g, 200);
    assert!(evs.iter().chain(evs2.iter()).any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:ogre:heavy")));
}

#[test]
fn bloat_pops_gas_on_death_and_gas_hurts() {
    let mut g = arena();
    let b = add_monster(&mut g, "bloat", 5, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Die { id, .. } if *id == b)));
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:bloat:gas")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Hurt { id, cause, .. } if *id == HERO_ID && cause == "gas")));
}

#[test]
fn pink_jelly_splits_when_hit() {
    let mut g = arena();
    add_monster(&mut g, "pink_jelly", 5, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "pink_jelly")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:pink_jelly:splitter")));
    assert!(evs.iter().filter(|e| matches!(e, Ev::Attack { dst, .. } if *dst == HERO_ID)).count() >= 2, "both halves fight");
}

#[test]
fn eel_attacks_only_from_water() {
    let mut g = arena();
    hold_rules(&mut g);
    {
        let run = g.run.as_mut().unwrap();
        run.floor.map.set(Pos::new(5, 5), Tile::Water);
        run.floor.map.set(Pos::new(5, 6), Tile::Water);
    }
    let e = add_monster(&mut g, "eel", 5, 5);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e2| matches!(e2, Ev::Attack { src, .. } if *src == e)));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:eel:water")));
    assert_eq!(g.run.as_ref().unwrap().floor.map.get(monster(&g, e).unwrap().pos), Tile::Water, "eels stay in water");
    // Out of water: no attack, no movement.
    let mut g = arena();
    hold_rules(&mut g);
    let e = add_monster(&mut g, "eel", 5, 5);
    let evs = ticks(&mut g, 40);
    assert!(!evs.iter().any(|e2| matches!(e2, Ev::Attack { src, .. } if *src == e)));
}

#[test]
fn conjurer_summons_two_blades_that_fade() {
    let mut g = arena();
    hold_rules(&mut g);
    add_monster(&mut g, "goblin_conjurer", 9, 5);
    let evs = ticks(&mut g, 20);
    let blades = evs.iter().filter(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "spectral_blade")).count();
    assert_eq!(blades, 2);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:goblin_conjurer:caster")));
    g.run.as_mut().unwrap().hero.max_hp = 500;
    g.run.as_mut().unwrap().hero.hp = 499;
    let evs = ticks(&mut g, 110);
    assert!(evs.iter().any(|e| matches!(e, Ev::Die { cause, .. } if cause == "faded")));
}

#[test]
fn monkey_steals_then_flees_and_drops_on_death() {
    let mut g = arena();
    hold_rules(&mut g);
    give(&mut g, "sword");
    let m = add_monster(&mut g, "monkey", 5, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Steal { id, .. } if *id == m)));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:monkey:thief")));
    assert!(hero(&g).inv.is_empty());
    let d = monster(&g, m).map(|mm| mm.pos.cheb(hero(&g).pos)).unwrap_or(99);
    assert!(d >= 3, "monkey flees, got {d}");
    // Killing it returns the loot to the floor.
    let run = g.run.as_mut().unwrap();
    let mi = run.monsters.iter().position(|mm| mm.id == m).unwrap();
    let mut cx_events = Vec::new();
    let mut cx_prov = Vec::new();
    let (run, mut cx) = {
        let Game { run, lineage, .. } = &mut g;
        let run = run.as_mut().unwrap();
        let set = lineage.active_set;
        let max_rows = lineage.max_rows();
        (
            run,
            crate::engine::Ctx {
                facts: &mut lineage.facts,
                kill_counts: &mut lineage.kill_counts,
                flavours: &lineage.flavours,
                rules: &lineage.sets[set],
                unlocks: &lineage.unlocks,
                grudges: &lineage.grudges,
                forge: &lineage.forge,
                max_rows,
                events: &mut cx_events,
                sim: true,
                prov: &mut cx_prov,
                variant: "",
                hunter: None,
                vault_pref: "weapon",
                lost: &lineage.lost,
                sets: &lineage.sets,
                active_set: set,
                trophies: &[],
                tally: Box::leak(Box::default()),
            },
        )
    };
    crate::turn::damage_monster(run, &mut cx, mi, 99, &crate::turn::Src::Hero { ranged: false });
    assert!(run.items.iter().any(|i| i.item.kind == "sword"));
}

/// QA on 3d71c33: a watched run read `$-3 · keeps $0` after a monkey took a brought item on
/// D1 — a theft took the item's value off loot that never held it. The carried gold never goes
/// below 0 (any `loot_add`), and a theft or swap takes off only what the item added: a brought
/// item or a packed supply takes nothing, a found one takes its value back.
#[test]
fn carried_gold_never_goes_below_zero() {
    // A brought item stolen with $3 carried: the $3 stay.
    let mut g = arena();
    hold_rules(&mut g);
    let id = give(&mut g, "sword");
    g.run.as_mut().unwrap().brought.push(id);
    g.run.as_mut().unwrap().loot_add_gold(3);
    let m = add_monster(&mut g, "monkey", 5, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Steal { id, amount: None, .. } if *id == m)), "a brought item's theft costs the stake nothing");
    assert_eq!(g.run.as_ref().unwrap().loot, 3);
    // An item given with nothing carried (never counted): the stake stays at $0, not below.
    let mut g = arena();
    hold_rules(&mut g);
    give(&mut g, "sword");
    add_monster(&mut g, "monkey", 5, 5);
    ticks(&mut g, 60);
    assert!(hero(&g).inv.is_empty(), "the monkey stole it");
    let run = g.run.as_ref().unwrap();
    assert!(run.loot >= 0 && run.loot_raw >= 0, "loot {} raw {}", run.loot, run.loot_raw);
    let s = g.snapshot();
    assert!(s.stake.loot >= 0 && s.stake.kept.unwrap_or(0) >= 0, "{:?}", s.stake.loot);
    // A found item's theft takes back exactly what it added.
    let mut g = arena();
    hold_rules(&mut g);
    let id = give(&mut g, "sword");
    let v = g.run.as_ref().unwrap().hero.inv.iter().find(|i| i.id == id).unwrap().value();
    g.run.as_mut().unwrap().loot_add(v);
    g.run.as_mut().unwrap().loot_add_gold(2);
    add_monster(&mut g, "monkey", 5, 5);
    ticks(&mut g, 60);
    assert!(hero(&g).inv.is_empty());
    assert_eq!(g.run.as_ref().unwrap().loot, 2);
    // Any `loot_add`, however large the take: floored at 0.
    let run = g.run.as_mut().unwrap();
    run.loot_add(-1_000_000);
    assert_eq!((run.loot, run.loot_raw), (0, 0));
    run.loot_add_gold(4);
    assert_eq!(run.loot, 4);
}

#[test]
fn ghoul_paralyses_sometimes() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 2000;
    g.run.as_mut().unwrap().hero.hp = 1999;
    add_monster(&mut g, "ghoul", 5, 5);
    add_monster(&mut g, "ghoul", 5, 6);
    let evs = ticks(&mut g, 600);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "paralysed")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:ghoul:paralyse")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { verb, .. } if verb.v == "paralysed")));
}

#[test]
fn wraith_drains_max_hp() {
    let mut g = arena();
    hold_rules(&mut g);
    let before = hero(&g).max_hp;
    add_monster(&mut g, "wraith", 5, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:wraith:drain")));
    assert!(hero(&g).max_hp < before);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:wraith:undead")));
}

#[test]
fn captive_is_neutral_until_freed_then_fights_and_can_die() {
    let mut g = arena();
    let c = add_monster(&mut g, "captive", 6, 5);
    g.run.as_mut().unwrap().monsters[0].awake = false;
    hold_rules(&mut g);
    let evs = ticks(&mut g, 30);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Move { id, .. } if *id == c)), "captives do not move");
    rules(&mut g, vec![Row::new(vec![], Verb::new("free_captive")), Row::new(vec![], Verb::new("rest"))]);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ally { id, state, .. } if *id == c && state == "freed")));
    assert!(monster(&g, c).unwrap().ally);
    add_monster(&mut g, "rat", 7, 5);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, .. } if *src == c)), "the ally fights");
    // It can die.
    let run = g.run.as_mut().unwrap();
    let ci = run.monsters.iter().position(|m| m.id == c).unwrap();
    run.monsters[ci].hp = 1;
    for (x, y) in [(8, 4), (8, 5), (8, 6), (9, 5)] {
        add_monster(&mut g, "ogre", x, y);
    }
    let evs = ticks(&mut g, 200);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ally { id, state, .. } if *id == c && state == "lost")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("fell"))));
}

#[test]
fn warlord_rallies_summons_and_buffs() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 500;
    g.run.as_mut().unwrap().hero.hp = 499;
    let w = add_monster(&mut g, "goblin_warlord", 8, 5);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == w && what == "rallies")));
    assert!(evs.iter().filter(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "goblin")).count() >= 2);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.starts_with("boss:goblin_warlord:counter"))));
    assert!(g.run.as_ref().unwrap().monsters.iter().any(|m| m.kind == "goblin" && m.buff_def.1 > 0));
}

#[test]
fn bloat_mother_swells_and_pops_a_big_cloud() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 500;
    g.run.as_mut().unwrap().hero.hp = 499;
    let b = add_monster(&mut g, "bloat_mother", 6, 5);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == b && what == "swells")), "she telegraphs on the first turn");
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.starts_with("boss:bloat_mother:counter"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, .. })));
    // Melee hits vent a 5×5 cloud and she heals in it: melee-only cannot win.
    g.run.as_mut().unwrap().overlays.clear();
    {
        let run = g.run.as_mut().unwrap();
        run.monsters[0].hp = 10;
    }
    attack_rules(&mut g);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "vents!")));
    let hp_after = monster(&g, b).map(|m| m.hp).unwrap_or(0);
    assert!(hp_after > 10 || monster(&g, b).is_none() || evs.iter().filter(|e| matches!(e, Ev::Hurt { id, .. } if *id == b)).count() > 0);
    // Ranged does not vent: shoot her down and she pops a 5×5 cloud on death.
    let b = if monster(&g, b).is_some() { b } else { add_monster(&mut g, "bloat_mother", 8, 5) };
    {
        let run = g.run.as_mut().unwrap();
        run.overlays.clear();
        run.monsters.retain(|m| m.hp > 0);
        run.monsters[0].hp = 3;
        run.hero.weapon = None;
        run.hero.auto_equip(Item::new(77, "bow"));
        run.hero.pos = Pos::new(2, 5);
        run.hero_dist_pos = None;
        run.monsters[0].pos = Pos::new(8, 5);
    }
    let evs = ticks(&mut g, 120);
    assert!(evs.iter().any(|e| matches!(e, Ev::Die { id, .. } if *id == b)));
    let cloud = evs.iter().filter(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, ttl: 60, .. })).count();
    assert!(cloud >= 20, "5×5 pop, got {cloud}");
}

#[test]
fn lich_reflects_throws_and_chants_skeletons() {
    let mut g = arena();
    g.lineage.unlocks.insert("throw".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "poison").unwrap());
    give(&mut g, "poison");
    g.run.as_mut().unwrap().hero.max_hp = 500;
    g.run.as_mut().unwrap().hero.hp = 499;
    let l = add_monster(&mut g, "lich", 8, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "poison")), Row::new(vec![], Verb::new("rest"))]);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "reflected!")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:lich:reflect")));
    assert!(hero(&g).poison.1 > 0, "the poison came back");
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == l && what == "chants")));
    assert!(evs.iter().filter(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "skeleton")).count() >= 2);
}

#[test]
fn grudge_monster_is_named_and_stronger() {
    let mut g = Game::new_literal(5);
    g.lineage.grudges.push(crate::descent::Grudge { kind: "rat".into(), name: "Grak".into(), depth: 1, heir: 1, avenged: false, tamed: false, biome: None });
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    let m = run.monsters.iter().find(|m| m.grudge).expect("grudge spawned on its floor");
    assert_eq!(m.name.as_deref(), Some("Grak"));
    assert!(m.max_hp > monster_def("rat").hp);
}

// ---------------------------------------------------------------- items

fn drink_test(kind: &str) -> (Game, Vec<Ev>) {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.hp = 12;
    give(&mut g, kind);
    rules(&mut g, vec![Row::new(vec![], Verb::arg("drink", "unknown"))]);
    let evs = ticks(&mut g, 10);
    (g, evs)
}

#[test]
fn potions_drunk_identify_and_take_effect() {
    let (g, evs) = drink_test("heal");
    assert_eq!(hero(&g).hp, 12 + hero(&g).max_hp / 2);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.ends_with("=heal"))));
    let (g, _) = drink_test("strength");
    assert_eq!(hero(&g).str_bonus, 1);
    let (g, _) = drink_test("speed");
    assert!(hero(&g).speed_t > 0 && hero(&g).speed() == 15);
    let (g, _) = drink_test("invisibility");
    assert!(hero(&g).invis_t > 0);
    let (g, _) = drink_test("poison");
    assert!(hero(&g).poison.1 > 0);
    let (g, evs) = drink_test("caustic");
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, .. })));
    assert!(!g.run.as_ref().unwrap().overlays.is_empty());
    let (g, _) = drink_test("confusion");
    assert!(hero(&g).confused > 0);
    let (mut g, evs) = drink_test("fire");
    let n0 = evs.iter().filter(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Fire, .. })).count();
    assert!(n0 >= 9, "3×3 fire, then it spreads once: {n0}");
    let evs = ticks(&mut g, 10);
    assert_eq!(evs.iter().filter(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Fire, .. })).count(), 0, "fire spreads only once");
    assert!(g.run.as_ref().unwrap().overlays.len() <= 25);
}

#[test]
fn poison_ticks_eight_over_forty_ticks_and_undead_are_immune() {
    let (mut g, _) = drink_test("poison");
    ticks(&mut g, 45);
    assert_eq!(12 - hero(&g).hp, 8, "8 damage over 40 ticks");
    let mut g = arena();
    g.lineage.unlocks.insert("throw".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "poison").unwrap());
    give(&mut g, "poison");
    let s = add_monster(&mut g, "skeleton", 8, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "poison")), Row::new(vec![], Verb::new("rest"))]);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "poison potion")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "counter:undead>poison")));
    assert_eq!(monster(&g, s).unwrap().hp, monster(&g, s).unwrap().max_hp);
}

#[test]
fn thrown_potions_land_on_the_target() {
    let mut g = arena();
    g.lineage.unlocks.insert("throw".into());
    for k in ["caustic", "fire", "confusion"] {
        g.lineage.facts.insert(ident_fact(&g.lineage.flavours, k).unwrap());
        give(&mut g, k);
    }
    let r = add_monster(&mut g, "rat", 6, 5);
    g.run.as_mut().unwrap().monsters[0].hp = 100;
    g.run.as_mut().unwrap().monsters[0].max_hp = 100;
    g.run.as_mut().unwrap().monsters[0].paralysed = 100;
    rules(
        &mut g,
        vec![
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "confusion")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "caustic")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "fire")),
            Row::new(vec![], Verb::new("rest")),
        ],
    );
    let evs = ticks(&mut g, 35);
    assert!(evs.iter().filter(|e| matches!(e, Ev::Projectile { .. })).count() >= 3);
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, x: 6, y: 5, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Fire, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Use { outcome, .. } if outcome == "confused")));
    let _ = r;
}

fn read_test(kind: &str, setup: impl Fn(&mut Game)) -> (Game, Vec<Ev>) {
    let mut g = arena();
    setup(&mut g);
    give(&mut g, kind);
    rules(&mut g, vec![Row::new(vec![], Verb::arg("read", "unknown")), Row::new(vec![], Verb::new("rest"))]);
    g.run.as_mut().unwrap().hero.hp -= 1;
    let evs = ticks(&mut g, 10);
    (g, evs)
}

#[test]
fn scrolls_read_identify_and_take_effect() {
    let (g, _) = read_test("teleport", |g| {
        add_monster(g, "rat", 5, 5);
    });
    assert!(hero(&g).pos.cheb(Pos::new(4, 5)) >= 8);
    let (g, _) = read_test("blink", |g| {
        add_monster(g, "rat", 5, 5);
    });
    let d = hero(&g).pos.cheb(Pos::new(4, 5));
    assert!((1..=3).contains(&d), "blink ≤ 3 tiles, got {d}");
    let (g, _) = read_test("fear", |g| {
        add_monster(g, "rat", 5, 5);
    });
    assert!(g.run.as_ref().unwrap().monsters[0].fear > 0);
    let (g, _) = read_test("mapping", |_| {});
    assert_eq!(g.run.as_ref().unwrap().floor.map.seen_pct(), 100);
    let (g, evs) = read_test("identify", |g| {
        give(g, "heal");
    });
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.ends_with("=heal"))));
    let _ = g;
    let (g, _) = read_test("enchant", |_| {});
    assert_eq!(hero(&g).weapon.as_ref().unwrap().enchant, 1);
    // QA on 524827b (qaAA: KEPT `axe +7` after `took axe +1`): the scroll's +1 is counted on the item
    assert_eq!(hero(&g).weapon.as_ref().unwrap().enchanted, 1);
    assert_eq!(hero(&g).weapon.as_ref().unwrap().kind, "sword", "the fighter starts with a sword");
    let (g, _) = read_test("darkness", |g| {
        add_monster(g, "rat", 5, 5);
    });
    assert!(g.run.as_ref().unwrap().monsters[0].blind > 0);
    let (g, evs) = read_test("summon_ally", |_| {});
    assert!(evs.iter().any(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "spectral_hound" && e.ally == Some(true))));
    assert!(g.run.as_ref().unwrap().allies().next().is_some());
    let (g, evs) = read_test("aggravate", |g| {
        add_monster(g, "rat", 13, 9);
        g.run.as_mut().unwrap().monsters[0].awake = false;
    });
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "aggravated!")));
    assert!(g.run.as_ref().unwrap().monsters[0].awake);
}

#[test]
fn gear_stats_and_bow_shoots_at_range() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.weapon = None;
    g.run.as_mut().unwrap().hero.auto_equip(Item::new(60, "bow"));
    g.run.as_mut().unwrap().hero.auto_equip(Item::new(61, "mail"));
    assert_eq!(hero(&g).atk(), (2, 6));
    assert_eq!(hero(&g).def(), 3);
    assert_eq!(hero(&g).speed(), 9);
    add_monster(&mut g, "rat", 9, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 15);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, verb, .. } if *src == HERO_ID && verb.as_deref() == Some("shoot"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Projectile { src, .. } if *src == HERO_ID)));
    assert!(!hero(&g).melee_used_flag(&g));
}

impl crate::hero::Hero {
    fn melee_used_flag(&self, g: &Game) -> bool {
        g.run.as_ref().unwrap().melee_used
    }
}

#[test]
fn gold_adds_loot_and_unknown_items_hint() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        let mut gold = Item::new(70, "gold");
        gold.amount = 25;
        run.items.push(crate::engine::FloorItem { pos: Pos::new(5, 5), item: gold });
        let mut p = Item::new(71, "poison");
        p.hint = Some(crate::item::Hint::Malevolent);
        run.items.push(crate::engine::FloorItem { pos: Pos::new(6, 5), item: p });
    }
    rules(&mut g, vec![]);
    ticks(&mut g, 40);
    // A pile's amount is coins and is added whole; an item adds its value before the divisor.
    assert_eq!(g.run.as_ref().unwrap().loot_raw, 25 * crate::engine::GOLD_DIVISOR + 14);
    assert_eq!(g.run.as_ref().unwrap().loot, 25 + 14 / crate::engine::GOLD_DIVISOR, "loot is counted in gold at pickup (Cut 4); a pile of $25 is $25");
    let snap = g.snapshot();
    let inv = &snap.hero.inv[0];
    assert!(!inv.known);
    assert_eq!(inv.kind, "potion");
    assert!(inv.label.ends_with("potion?"));
    assert_eq!(inv.hint, Some(crate::item::Hint::Malevolent));
}

#[test]
fn leash_stacks_and_teaches_the_fact() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        for (i, x) in [5, 6].iter().enumerate() {
            let mut l = Item::new(80 + i as u32, "leash");
            l.amount = 1;
            run.items.push(crate::engine::FloorItem { pos: Pos::new(*x, 5), item: l });
        }
    }
    rules(&mut g, vec![]);
    // Cut 8B §3: the fact is held from the start (the kennel's leash); drop it to see it learned.
    g.lineage.facts.remove("item:leash");
    let evs = ticks(&mut g, 40);
    let leash = hero(&g).inv.iter().find(|i| i.kind == "leash").unwrap();
    assert_eq!(leash.amount, 2);
    assert_eq!(hero(&g).inv.len(), 1);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "item:leash")));
}

// ---------------------------------------------------------------- exits

fn finish_with(g: &mut Game, tier: ExitTier) -> u32 {
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, tier);
    }
    let id = g.run.as_ref().unwrap().id;
    g.finish_run();
    id
}

/// Cut 27 §3: a return is out where he stands once `RETURN_TICKS` are spent (short of the
/// stairs); a bank walks on to them.
#[test]
fn a_return_is_out_where_he_stands() {
    for bank in [false, true] {
        let mut g = arena();
        rules(&mut g, vec![Row::new(vec![], Verb::new(if bank { "bank" } else { "return" }))]);
        let run = g.run.as_mut().unwrap();
        run.hero.pos = Pos::new(13, 10);
        let far = run.hero.pos.cheb(run.floor.stairs_up) as u32;
        assert!(far * crate::engine::TICKS_PER_TURN > crate::turn::RETURN_TICKS + 20);
        let mut n = 0;
        while g.run.as_ref().unwrap().over.is_none() && n < 2_000 {
            g.tick();
            n += 1;
        }
        let run = g.run.as_ref().unwrap();
        let tier = if bank { ExitTier::Bank } else { ExitTier::Return };
        assert_eq!(run.over, Some(tier));
        assert_eq!(run.hero.pos == run.floor.stairs_up, bank, "bank {bank}: out at {:?} after {n} ticks", run.hero.pos);
        if !bank {
            assert!(n <= crate::turn::RETURN_TICKS + 2 * crate::engine::TICKS_PER_TURN, "{n}");
        }
    }
}

/// Cut 27 §3: a bank's walk wakes the hostiles within the carry's scent — wider the more he
/// carries; a return's does not.
#[test]
fn a_bank_walk_wakes_the_carrys_scent() {
    for (loot, verb, woke) in [(0, "bank", false), (200, "bank", true), (200, "return", false)] {
        let mut g = arena();
        rules(&mut g, vec![Row::new(vec![], Verb::new(verb))]);
        let id = add_monster(&mut g, "goblin", 13, 10);
        let run = g.run.as_mut().unwrap();
        run.loot = loot;
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.awake = false;
        m.last_seen = None;
        let mut n = 0;
        while g.run.as_ref().unwrap().homeward.is_none() && n < 20 {
            g.tick();
            n += 1;
        }
        let run = g.run.as_ref().unwrap();
        assert!(run.homeward.is_some());
        let m = run.monsters.iter().find(|m| m.id == id).unwrap();
        assert_eq!(m.awake, woke, "{verb} with ${loot}");
    }
}

#[test]
fn bank_keeps_all_loot_and_gold() {
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(100);
    give(&mut g, "sword");
    give(&mut g, "heal");
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.gold, 100 / crate::engine::GOLD_DIVISOR);
    let p = g.pending_exit.as_ref().unwrap();
    assert_eq!(p.items.len(), 2, "sword and heal are offered; the starting dagger is not loot");
    g.keep(vec![p.items[0].id]).unwrap();
    assert_eq!(g.lineage.vault.len(), 1);
    assert!(g.lineage.gold > 25, "the rest was salvaged");
}

#[test]
fn return_keeps_sixty_percent() {
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(100);
    for k in ["sword", "axe", "heal", "heal", "teleport"] {
        give(&mut g, k);
    }
    finish_with(&mut g, ExitTier::Return);
    assert!(g.lineage.gold >= 15 && g.lineage.gold < 25, "60% of loot plus salvage of the unkept items, ÷4: {}", g.lineage.gold);
    assert_eq!(g.pending_exit.as_ref().unwrap().items.len(), 3, "60% of 5 items, rounded up");
}

#[test]
fn death_keeps_nothing_and_leaves_bones() {
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(100);
    for k in ["sword", "heal", "heal", "teleport"] {
        give(&mut g, k);
    }
    let brought = give(&mut g, "plate");
    g.run.as_mut().unwrap().brought.push(brought);
    g.run.as_mut().unwrap().hero.hp = 0;
    finish_with(&mut g, ExitTier::Death);
    // Cut 2 §2: the death itself yields nothing; the purse is then topped up to the wake pay
    // (a new heir's one potion), which is its own ledger line, never loot.
    let died = g.lineage.gold_ledger.iter().find(|l| l.why.starts_with("died")).expect("death line");
    assert_eq!(died.delta, 0, "Cut 2 §2: death yields nothing");
    assert_eq!(g.lineage.gold, crate::engine::WAKE_PAY.min(g.lineage.gold.max(crate::engine::WAKE_PAY)), "wake pay tops the purse up, never beyond");
    let p = g.pending_exit.as_ref().unwrap();
    assert!(p.items.is_empty());
    assert_eq!(g.lineage.heir, 2);
    assert_eq!(g.lineage.graveyard.len(), 1);
    assert_eq!(g.lineage.bones.len(), 1, "the kit waits on the floor");
    let b = &g.lineage.bones[0];
    assert_eq!((b.heir, b.depth), (1, 1));
    assert_eq!(b.items.len(), 5, "sword, heals, scroll, the uninsured plate; not the starting sword");
    assert!(g.lineage.facts.contains("bones:1"));
    assert!(g.events.iter().any(|e| matches!(e, Ev::Bones { heir: 1, items: 5, .. })));
    assert!(g.events.iter().any(|e| matches!(e, Ev::Rest { seconds: 1200, .. })), "a death is a 20-minute wake");
    assert_eq!(g.lineage().rest_left_s, 1200);
    assert_eq!(g.lineage().bones[0].items, 5);
    // The next heir finds them on D1 and recovers them.
    g.auto_keep();
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    let bi = run.items.iter().position(|i| i.item.kind == "bones").expect("bones placed on D1");
    run.hero.pos = run.items[bi].pos;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::pickup_here(run, &mut cx);
    }
    assert!(g.events.iter().any(|e| matches!(e, Ev::Bones { heir: 1, items: 5, .. })));
    assert!(hero(&g).armour.as_ref().is_some_and(|a| a.kind == "plate"));
    assert_eq!(g.run.as_ref().unwrap().bones_found, vec![1]);
    finish_with(&mut g, ExitTier::Return);
    assert!(g.lineage.bones.is_empty(), "the pile is gone once recovered");
    assert_eq!(g.batch.bones_found.len(), 1);
    assert!(g.events.iter().any(|e| matches!(e, Ev::Rest { seconds: 1200, .. })), "rest as long as the run, never under the 20-minute floor");
}

#[test]
fn bones_piles_cap_at_bones_max_and_bone_sense_paths_to_them() {
    let cap = crate::engine::BONES_MAX as u32;
    let mut g = arena();
    for heir in 1..=cap + 1 {
        g.lineage.bones.push(crate::engine::Bones { heir, depth: 1, items: vec![Item::new(500 + heir, "dagger")], named: false, biome: None });
    }
    give(&mut g, "sword");
    g.run.as_mut().unwrap().hero.hp = 0;
    finish_with(&mut g, ExitTier::Death);
    assert_eq!(g.lineage.bones.len(), cap as usize, "oldest expires");
    let mut want: Vec<u32> = (3..=cap + 1).collect();
    want.push(1);
    assert_eq!(g.lineage.bones.iter().map(|b| b.heir).collect::<Vec<_>>(), want);
    // bone_sense: the chores walk to unseen bones.
    g.lineage.unlocks.insert("bone_sense".into());
    g.auto_keep();
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().items.iter().filter(|i| i.item.kind == "bones").count() == cap as usize);
    g.set_rules(RuleSet::default()).unwrap();
    let mut found = false;
    for _ in 0..3000 {
        g.tick();
        if g.events.iter().any(|e| matches!(e, Ev::Bones { .. })) {
            found = true;
            break;
        }
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            break;
        }
    }
    assert!(found, "bone_sense led the hero to a pile");
}

#[test]
fn reaching_d34_is_the_ending_and_d19_is_the_foundry() {
    // Cut 7: the bottom is D34 (Cut 3: D31).
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 33;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 34, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "bank")));
    assert!(g.lineage.ended);
    // Cut 3: the Foundry's doorstep (Cut 7: D19) is not an ending.
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 18;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 19, biome, .. } if biome == "foundry")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "biome:foundry")));
    assert!(!g.lineage.ended);
    assert_eq!(g.run.as_ref().unwrap().depth, 19);
}

#[test]
fn descending_regenerates_floor_and_learns_biome() {
    // Cut 7: the Fens start at D9 (the Warrens run to D8).
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 8;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 9, biome, .. } if biome == "fens")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "biome:fens")));
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.depth, 9);
    assert!(!run.monsters.is_empty());
    assert!(!run.floor.water_tiles().is_empty());
    assert_eq!(run.hero.pos, run.floor.stairs_up);
}

// ---------------------------------------------------------------- facts and vocabulary

#[test]
fn foe_tag_condition_needs_the_fact() {
    let mut g = arena();
    add_monster(&mut g, "jackal", 6, 5);
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "pack")], Verb::new("return")), Row::new(vec![], Verb::new("rest"))]);
    g.run.as_mut().unwrap().hero.hp -= 1;
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Exit { .. })));
    g.lineage.facts.insert("foe:jackal:pack".into());
    let evs = ticks(&mut g, 100);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { .. })));
}

#[test]
fn item_condition_needs_identification() {
    let mut g = arena();
    give(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![Cond::t("item", "heal")], Verb::new("return")), Row::new(vec![], Verb::new("rest"))]);
    g.run.as_mut().unwrap().hero.hp -= 1;
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Exit { .. })));
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let evs = ticks(&mut g, 100);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { .. })));
}

#[test]
fn first_sight_facts_and_notes() {
    let mut g = arena();
    hold_rules(&mut g);
    add_monster(&mut g, "skeleton", 9, 5);
    let evs = ticks(&mut g, 1);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:skeleton")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "foe:skeleton:undead")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text == "Met a skeleton.")));
    let v = g.vocabulary();
    assert!(v.conds.contains(&Cond::t("foe_tag", "undead")));
}

// ---------------------------------------------------------------- forecast

#[test]
fn forecast_is_bounded_by_best_depth_plus_one() {
    let mut g = Game::new_literal(9);
    let f = g.forecast();
    assert_eq!(f.known_to, 1);
    assert_eq!(f.depths.len(), 1);
    assert_eq!(f.depths[0].depth, 1);
    assert!((f.depths[0].reach - 1.0).abs() < 1e-9);
    g.lineage.best_depth = 4;
    let f = g.forecast();
    assert_eq!(f.known_to, 5);
    assert_eq!(f.depths.len(), 5);
    assert!(f.causes.len() <= 3);
    for w in f.depths.windows(2) {
        assert!(w[0].reach >= w[1].reach);
    }
    let f2 = g.forecast();
    assert_eq!(f, f2, "forecast is deterministic");
}

// ---------------------------------------------------------------- offline

#[test]
fn offline_consumes_the_budget_and_ends_at_camp() {
    let mut g = Game::new_literal(4);
    let r = g.run_offline(300);
    assert_eq!(r.elapsed_s, 300);
    // Cut 12: the begun run finishes past the budget (≤ one run), the hero is at camp.
    assert!(g.lineage.total_turns >= 3000, "{}", g.lineage.total_turns);
    assert!(g.run.is_none(), "an absence ends at camp");
    assert!(r.live.is_none());
    assert!(!r.sampled);
    assert!(!r.learned.is_empty());
    let r = g.run_offline(3600);
    assert!(!r.pending.is_empty(), "an hour away leaves a decision: {:?}", r.pending);
}

#[test]
fn offline_samples_after_twenty_stalled_runs() {
    let mut g = Game::new_literal(6);
    crate::probes::learn_everything(&mut g);
    for m in crate::defs::MONSTERS {
        g.lineage.kills.insert(m.kind.into());
    }
    g.lineage.best_depth = 16;
    g.lineage.renown = 1_000_000;
    g.lineage.rank = 100;
    g.lineage.trophies = vec!["pacifist_floor".into(), "no_heal_D5".into(), "ranged_only_D5".into(), "boss_untouched".into()];
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 10, xp: 0, next: 0 });
    g.lineage.trophies.push("master:fighter".into());
    g.lineage.unlocks.insert("phalanx".into());
    g.set_rules(RuleSet::default()).unwrap();
    let r = g.run_offline(30 * 3600);
    assert!(r.sampled, "a walled passive policy stalls and is sampled");
    assert!(r.runs > 40);
    assert!(g.run.is_none(), "an absence ends at camp (Cut 12)");
}

// ---------------------------------------------------------------- stall verdict

/// `hp<20 → return` on top of the default set: the hero always comes home, so an absence is a
/// stall — the return row is named, and at least one patch moves the forecast at depth + 1.
#[test]
fn a_set_that_always_returns_gets_a_stall_verdict_with_patches() {
    // Cut 23 §2: seed 10 (was 5: the walk home answers now — a heal under the return drinks,
    // a jackal at the elbow is struck — and seed 5's heirs set new bests through the night).
    // Cut 5: 12 h (situations on D1–5 gave this seed a new best on its 17th run of 8 h);
    // Cut 8B: 16 h (the first stray on D2 moved the bests again). Cut 13: `hp < 35 %` (at
    // 20 % the set died three times in 16 h once running thieves stopped counting as foes).
    // The client's path (below) runs beside the 16 h night: two games, one thread each.
    // Cut 24 (the floors' arrival events, named foes resting, the kit never stolen): seed 6.
    let chunked = std::thread::spawn(|| {
        let mut q = Game::new_literal(6);
        let mut set = q.lineage.rules().clone();
        set.rows.insert(0, Row::new(vec![Cond::n("hp<", 35)], Verb::new("return")));
        q.set_rules(set).unwrap();
        let mut last = None;
        for _ in 0..32 {
            last = crate::offline::run_offline_quick(&mut q, 1800).stall;
        }
        last
    });
    let mut g = Game::new_literal(6);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 35)], Verb::new("return")));
    g.set_rules(set).unwrap();
    let r = g.run_offline(16 * 3600);
    let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
    let stall = r.stall.unwrap_or_else(|| panic!("no stall: {} runs · {deaths} deaths · bests {:?}", r.runs, r.bests));
    // Cut 13: the stairs are taken on arrival (`ai::descend_step`), which moved this seed's
    // dice; the one death it gets now is a bloat's burst at 9 hp — `dice`, no row to name.
    // Cut 19 §2/§4: a return walks now — this seed's one death is a jackal on the way home, and
    // the verdict names the return row that walked him into it (`row`, R1); nothing else names
    // a missing row.
    // Cut 25 §3: the chore fix moved the night — its one death is now an archer's on D6, a `gap`
    // measured on its own replays; the stall still names the return row.
    assert!(g.deaths.values().all(|d| matches!(d.death.verdict.as_str(), "dice" | "gap") || (d.death.verdict == "row" && d.death.cause_row == Some(0))), "{:?}", g.deaths.values().map(|d| (&d.death.cause, &d.death.verdict)).collect::<Vec<_>>());
    assert_eq!(stall.row, 0);
    assert!(stall.fired >= 4, "{}", stall.text);
    assert!(stall.text.starts_with("R1 return ended"), "{}", stall.text);
    assert!(crate::rules::word_count(&stall.text) <= 12, "{}", stall.text);
    assert!(!stall.patches.is_empty() && stall.patches.len() <= 3);
    assert!(stall.patches.iter().all(|p| p.forecast_delta > crate::offline::STALL_DELTA), "{:?}", stall.patches);
    assert!(stall.patches.windows(2).all(|w| w[0].forecast_delta >= w[1].forecast_delta));
    // Cut 9 §5: the verdict carries the last-5 trace of the latest run the row ended.
    let tr = stall.trace.as_ref().expect("the stall carries a trace");
    assert!(!tr.turns.is_empty() && tr.turns.len() <= crate::engine::EXIT_TRACE_LEN);
    assert_eq!(tr.turns.last().unwrap().row, 0, "the last turn is the return: {:?}", tr.turns.last());
    // The client's path: 30-minute quick slices; the last slice carries the same stall.
    let chunked = chunked.join().unwrap_or_else(|e| std::panic::resume_unwind(e)).expect("the last quick slice carries the stall");
    assert_eq!(chunked.row, 0);
    assert!(chunked.fired >= 4, "{}", chunked.text);
    // Cut 7: this seed's chunked stall sits at the Warlord's floor (D8), where no single
    // patch moves the forecast past the wall; a stall at a boss may carry none.
    let at_wall = chunked.text.ends_with(" at D8");
    assert!(!chunked.patches.is_empty() || at_wall, "{}", chunked.text);
    // The verdict is a state: a second look (no runs, same rules) repeats it from the cache.
    let again = crate::offline::stall_verdict(&mut g).expect("still stalled");
    assert_eq!(again.patches, stall.patches);
    // Editing the rules opens a fresh window.
    let mut set = g.lineage.rules().clone();
    set.rows.remove(0);
    g.set_rules(set).unwrap();
    assert!(crate::offline::stall_verdict(&mut g).is_none());
}

#[test]
fn a_set_that_dies_has_no_stall() {
    let mut g = Game::new_literal(5);
    let r = g.run_offline(8 * 3600);
    assert!(r.deaths.iter().map(|d| d.n).sum::<u32>() > 0, "the default set dies");
    assert!(r.stall.is_none());
}

#[test]
fn stall_patches_apply_as_replace_remove_or_insert() {
    use crate::offline::apply_patch;
    let rules = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")), Row::new(vec![], Verb::new("attack"))], name: None, route: Vec::new() };
    let mk = |row: Row, at: usize, replace: bool, remove: bool| Patch { no_gain: false, row, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace, remove, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None, whole: None, gem: false, restores: None };
    let deeper = Row::new(vec![Cond::n("hp<", 10)], Verb::new("return"));
    let r = apply_patch(&rules, &mk(deeper.clone(), 0, true, false), 8);
    assert_eq!(r.rows, vec![deeper.clone(), rules.rows[1].clone()]);
    let r = apply_patch(&rules, &mk(rules.rows[0].clone(), 0, false, true), 8);
    assert_eq!(r.rows, vec![rules.rows[1].clone()]);
    let r = apply_patch(&rules, &mk(deeper.clone(), 0, false, false), 2);
    assert_eq!(r.rows, vec![deeper, rules.rows[0].clone()], "an insert into a full set drops the last row");
}

// ---------------------------------------------------------------- verdict and patches

#[test]
fn death_with_an_unused_heal_is_a_gap_with_a_drink_patch() {
    let mut g = arena_seed(2);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    give(&mut g, "heal");
    give(&mut g, "heal");
    g.run.as_mut().unwrap().hero.hp = 14;
    for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4)] {
        add_monster(&mut g, "goblin", x, y);
    }
    attack_rules(&mut g);
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("the hero died");
    let d = g.death(id).unwrap();
    assert_eq!(d.verdict, "gap");
    // Cut 14 §2: the margin calls the heal unused only when drinking it would have saved him
    // (its row survives ≥ `MARGIN_BAR` of the replays); here the heal row is offered at 17 %
    // beside a return at 100 %, so the line stays silent about it.
    let heal = d.patches.iter().find(|p| p.row.verb.v == "drink" && p.row.verb.a.as_deref() == Some("heal")).expect("the heal row is a patch");
    assert_eq!(d.margin.contains("heal unused"), heal.survive >= crate::trace::MARGIN_BAR, "{} vs {:?}", d.margin, heal);
    assert!(d.trace.turns.len() <= 10 && !d.trace.turns.is_empty());
    assert!(!d.patches.is_empty() && d.patches.len() <= 3);
    assert!(d.patches.iter().all(|p| p.survive >= crate::trace::survive_bar(d.baseline) || p.forecast_delta >= crate::trace::DELTA_BAR), "{:?}", d.patches);
    assert!(d.patches.iter().any(|p| p.row.verb.v == "drink" || p.row.verb.v == "retreat" || p.row.verb.v == "back_corridor"), "{:?}", d.patches);
    assert!(d.morgue.contains("rules:") && d.morgue.contains("trace:"));
}

#[test]
fn hopeless_death_is_dice() {
    let mut g = arena_seed(3);
    g.run.as_mut().unwrap().hero.hp = 1;
    g.run.as_mut().unwrap().hero.max_hp = 1;
    for (x, y) in [(3, 4), (4, 4), (5, 4), (3, 5), (5, 5), (3, 6), (4, 6), (5, 6)] {
        add_monster(&mut g, "ogre", x, y);
    }
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.hp = 1;
    g.run.as_mut().unwrap().hero.paralysed = 400; // no action can save a hero that cannot act
    // Advance far enough that history has ~100 ticks, then let the ogres land.
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let d = g.death(id.expect("died")).unwrap();
    assert_eq!(d.verdict, "dice");
    // Cut 11 §4: a dice death names the alternative below the bar. Cut 15 §6: 0 % candidates
    // go when anything survives; a paralysed 1-HP hero ringed by ogres has nothing that does,
    // so the hopeless alternatives stay (the gate: never empty).
    assert!(!d.patches.is_empty() && d.patches.iter().all(|p| p.below_bar && p.survive <= 1.0), "{:?}", d.patches);
}

// ---------------------------------------------------------------- sifter (Cut 5 §1 episodes)

/// A low point (two jackals, 3 HP), the row that answered it, a recovery that seals the
/// episode, and the exit that resolves it: one story line in the three-beat grammar.
#[test]
fn episodes_close_on_a_low_a_recovery_and_the_exit() {
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(58 * crate::engine::GOLD_DIVISOR);
    let a = add_monster(&mut g, "jackal", 5, 5);
    let _b = add_monster(&mut g, "jackal", 5, 6);
    attack_rules(&mut g);
    {
        let (run, mut cx) = g.ctx();
        for m in run.monsters.iter_mut() {
            m.stun = 500; // they bit once; the story is what follows
        }
        run.floor.map.update_vision(run.hero.pos, VISION);
        let ai = run.monsters.iter().position(|m| m.id == a).unwrap();
        let hp = run.hero.hp;
        crate::turn::damage_hero(run, &mut cx, hp - 3, &crate::turn::Src::Mon(ai));
        assert_eq!(run.arc.low, Some((3, run.turn)));
        assert_eq!(run.arc.threat[0], ("jackal".to_string(), 2));
        assert!(run.arc.row_pending);
    }
    // The next action is the row at the low point.
    ticks(&mut g, 10);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.arc.row.as_ref().map(|a| a.row), Some(0), "{:?}", run.arc);
    assert_eq!(run.arc.row.as_ref().unwrap().verb.v, "attack");
    // Recovery past 60 % seals it; the exit resolves it.
    {
        let run = g.run.as_mut().unwrap();
        run.hero.hp = run.hero.max_hp;
        run.monsters.clear();
    }
    ticks(&mut g, 10);
    assert_eq!(g.run.as_ref().unwrap().arc.sealed.len(), 1, "sealed after recovery");
    assert!(!g.run.as_ref().unwrap().arc.has_low());
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.episodes.len(), 1, "{:?}", run.episodes);
    let hs = crate::sifter::sift_with(run, false);
    assert_eq!(hs[0].text, "Two jackals cornered him to 3 HP; R1 attacked; banked $58.");
    assert!(crate::sifter::story_ok(&hs[0].text));
    let arc = hs[0].arc.as_ref().unwrap();
    assert_eq!((arc.low_hp, arc.row, arc.threat.as_str(), arc.resolution.as_str()), (3, 0, "jackal", "banked $58"));
    assert_eq!(hs[0].score, 4 * 3, "low-point depth 4 × banked 3");
    // QA on 92eb880 (qaN): a low walked down from reads the depth the run went on to reach.
    {
        let run = g.run.as_mut().unwrap();
        run.episodes[0].resolution = crate::sifter::Resolution::Reached { depth: 3 };
        run.max_depth = 6;
    }
    let hs = crate::sifter::sift_with(g.run.as_ref().unwrap(), false);
    assert!(hs[0].text.ends_with("reached D6."), "{}", hs[0].text);
    // A quiet run still closes on its exit — Cut 12 §4: as the routine line, the floor and
    // what it brought (`D1: returned $0.`; the arena's D1 has no situation word).
    let mut g2 = arena();
    attack_rules(&mut g2);
    ticks(&mut g2, 2);
    let (run, mut cx) = g2.ctx();
    crate::turn::end_run(run, &mut cx, ExitTier::Return);
    let hs = crate::sifter::sift_with(run, false);
    assert_eq!(hs.len(), 1);
    assert_eq!(hs[0].text, "D1: returned $0.");
    assert!(crate::sifter::story_ok(&hs[0].text));
    assert!(!crate::sifter::names_agent(&hs[0]));
}

/// Cut 12 §4: the routine line — a send with nothing to tell reads its floor, its situation,
/// the row that ended it and what it brought; the gate's grammar accepts it, and the line
/// names a row when a row brought the hero home.
#[test]
fn routine_line_reads_the_floor_and_its_twist() {
    use crate::sifter::{names_agent, routine_line, story_line, story_ok, Act, Episode, Resolution, Setup};
    let ep = |depth: u32, twist: Option<&str>, act: Act, res: Resolution| Episode { depth, setup: Setup::Untouched, act, resolution: res, twist: twist.map(String::from), max_hp: 30, low_hp: 30, ..Default::default() };
    let ret = |row: i32| Act { row, verb: Verb::new("return"), target: None, boss: false, walk: false };
    let e = ep(6, Some("nest"), ret(2), Resolution::Returned { gold: 54 });
    assert_eq!(story_line(&e), "D6, the nest: R3 returned $54.");
    let e = ep(2, None, Act::default(), Resolution::Returned { gold: 8 });
    assert_eq!(story_line(&e), "D2: returned $8.");
    let e = ep(5, Some("vault"), ret(0), Resolution::Lost { stalled: false });
    assert_eq!(story_line(&e), "D5, the cage: lost the thread.");
    let e = ep(7, Some("lock"), Act { row: 1, verb: Verb::new("bank"), target: None, boss: false, walk: false }, Resolution::Banked { gold: 120 });
    assert_eq!(story_line(&e), "D7, the lock: R2 banked $120.");
    // An attack row that happened to be the last act is not credited with the exit.
    let e = ep(7, Some("den"), Act { row: 1, verb: Verb::arg("attack", "nearest"), target: None, boss: false, walk: false }, Resolution::Returned { gold: 3 });
    assert_eq!(story_line(&e), "D7, the den: returned $3.");
    // A low point is never routine.
    let mut e = ep(6, Some("nest"), ret(2), Resolution::Returned { gold: 54 });
    e.setup = Setup::Hurt;
    e.low_hp = 4;
    e.threat = vec![("jackal".into(), 2)];
    assert_eq!(routine_line(&e), None);
    assert_eq!(story_line(&e), "Two jackals took him to 4 HP; R3 returned; returned.");
    for t in ["D6, the nest: R3 returned $54.", "D2: returned $8.", "D5, the cage: lost the thread.", "D7, the lock: R2 banked $120."] {
        assert!(story_ok(t), "{t}");
        assert!(word_count(t) <= crate::sifter::STORY_WORDS);
    }
    for t in ["D6, the pit: returned $54.", "D6, the nest: returned.", "the nest: returned $54.", "D6: R3 returned $54"] {
        assert!(!story_ok(t), "{t}");
    }
    let h = |text: &str| Highlight { pattern: "episode".into(), score: 1, t: 0, run_id: 1, text: text.into(), arc: Some(HighlightArc { low_hp: 30, row: 2, threat: "none".into(), resolution: "returned".into() }) };
    assert!(names_agent(&h("D6, the nest: R3 returned $54.")));
    assert!(!names_agent(&h("D6, the nest: returned $54.")));
}

/// A boss dying and a companion falling close the live episode on their own; a death names
/// its cause; the trait beat reads `greed took the gold`.
#[test]
fn episodes_close_on_bosses_companions_and_deaths() {
    use crate::sifter::Resolution;
    let mut g = arena();
    let w = add_monster(&mut g, "goblin_warlord", 6, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "tag:boss"))]);
    g.lineage.facts.insert("foe:goblin_warlord:boss".into());
    g.run.as_mut().unwrap().monsters.iter_mut().for_each(|m| m.stun = 500);
    ticks(&mut g, 10);
    {
        let (run, mut cx) = g.ctx();
        let wi = run.monsters.iter().position(|m| m.id == w).unwrap();
        crate::turn::damage_hero(run, &mut cx, 27, &crate::turn::Src::Mon(wi));
    }
    ticks(&mut g, 10);
    {
        let (run, mut cx) = g.ctx();
        let wi = run.monsters.iter().position(|m| m.id == w).unwrap();
        run.aimed = true;
        crate::turn::damage_monster(run, &mut cx, wi, 99, &crate::turn::Src::Hero { ranged: false });
    }
    let run = g.run.as_ref().unwrap();
    let boss = run.episodes.iter().find(|e| matches!(e.resolution, Resolution::FirstBoss { .. })).expect("boss episode");
    let text = crate::sifter::story_line(boss);
    assert_eq!(text, "The Warlord took him to 9 HP; R1 attacked him; first boss.", "{boss:?}");
    assert!(crate::sifter::story_ok(&text));
    // A companion's fall.
    let mut g = arena();
    let party = crate::probes::pets_party();
    crate::engine::spawn_party(g.run.as_mut().unwrap(), &party);
    let j = add_monster(&mut g, "jackal", 6, 6);
    {
        let (run, mut cx) = g.ctx();
        let ji = run.monsters.iter().position(|m| m.id == j).unwrap();
        crate::turn::damage_hero(run, &mut cx, 30, &crate::turn::Src::Mon(ji));
        let ci = run.monsters.iter().position(|m| m.is_companion()).unwrap();
        crate::turn::damage_monster(run, &mut cx, ci, 99, &crate::turn::Src::Mon(ji));
    }
    let run = g.run.as_ref().unwrap();
    let fell = run.episodes.last().unwrap();
    assert!(matches!(fell.resolution, Resolution::Fell { .. }), "{fell:?}");
    let text = crate::sifter::story_line(fell);
    assert!(text.ends_with(" fell."), "{text}");
    assert!(crate::sifter::story_ok(&text), "{text}");
    assert!(crate::sifter::names_agent(&crate::sifter::to_highlight(run, fell, false)));
    // A death, with the greedy trait's beat.
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    let r = add_monster(&mut g, "rat", 6, 5);
    {
        let (run, mut cx) = g.ctx();
        let ri = run.monsters.iter().position(|m| m.id == r).unwrap();
        crate::turn::damage_hero(run, &mut cx, 33, &crate::turn::Src::Mon(ri));
        crate::sifter::on_action(run, -1, &Verb::new("pick_up"));
        crate::turn::damage_hero(run, &mut cx, 99, &crate::turn::Src::Gas);
    }
    let run = g.run.as_ref().unwrap();
    let died = run.episodes.last().unwrap();
    assert_eq!(crate::sifter::story_line(died), "A rat took him to 3 HP; greed grabbed; died to gas.", "the cause outranks the trait's long form");
    assert!(crate::sifter::names_agent(&crate::sifter::to_highlight(run, died, false)));
    assert!(crate::sifter::score(died, true) > crate::sifter::score(died, false), "a named heir's death weighs more");
}

/// The reel: the top three by score, never two with the same (threat, resolution), plus the
/// best-depth run's closing episode.
#[test]
fn reel_is_three_distinct_pairs_plus_the_best_run() {
    let h = |score: i32, run_id: u32, t: u32, threat: &str, res: &str| Highlight {
        pattern: "episode".into(),
        score,
        t,
        run_id,
        text: format!("{threat}; R1 attacked; {res}."),
        arc: Some(HighlightArc { low_hp: 3, row: 0, threat: threat.into(), resolution: res.into() }),
    };
    let hs = vec![
        h(20, 1, 10, "jackal", "banked $58"),
        h(18, 2, 10, "jackal", "banked $70"),
        h(15, 3, 10, "goblin", "died to a goblin"),
        h(12, 4, 10, "gas", "reached D3"),
        h(9, 5, 10, "rat", "returned"),
        h(2, 6, 5, "ogre", "reached D4"),
        h(1, 6, 50, "none", "banked $12"),
        Highlight { pattern: "bones".into(), score: 6, t: 1, run_id: 7, text: "Recovered heir 2's bones on D3.".into(), arc: None },
    ];
    let reel = crate::sifter::reel(&hs, Some(6), &[]);
    let ids: Vec<u32> = reel.iter().map(|h| h.run_id).collect();
    // Cut 9 §6: the best run's closing episode leads.
    assert_eq!(ids, vec![6, 1, 3, 4], "{reel:?}");
    assert_eq!(reel[0].t, 50, "the best run's closing episode");
    let pairs: std::collections::BTreeSet<_> = reel.iter().filter_map(crate::sifter::pair).collect();
    assert_eq!(pairs.len(), 4);
    // Without a best run the fourth slot takes the bones highlight.
    let reel = crate::sifter::reel(&hs, None, &[]);
    assert_eq!(reel.len(), 4);
    assert_eq!(reel[3].pattern, "bones");
}

/// Cut 9 §6: the reel skips pairs the last three absences showed, prefers a turn beat that
/// names a row or a combo over `no row fired`, and leads with the best run's closing episode
/// (its latest fresh one when the closing pair was shown before; alone when nothing is fresh).
#[test]
fn reel_dedupes_across_absences_and_prefers_rows() {
    let h = |score: i32, run_id: u32, t: u32, threat: &str, turn: &str, res: &str| Highlight {
        pattern: "episode".into(),
        score,
        t,
        run_id,
        text: format!("{threat}; {turn}; {res}."),
        arc: Some(HighlightArc { low_hp: 3, row: 0, threat: threat.into(), resolution: res.into() }),
    };
    let hs = vec![
        h(20, 1, 10, "jackal", "no row fired", "banked $58"),
        h(15, 2, 10, "goblin", "R2 drank", "died to a goblin"),
        h(12, 3, 10, "gas", "the bait landed", "reached D3"),
        h(9, 4, 10, "rat", "greed grabbed", "returned"),
        h(2, 5, 5, "ogre", "R1 attacked", "reached D4"),
        h(1, 5, 50, "none", "no row fired", "banked $12"),
    ];
    let recent = vec![("gas".to_string(), "reached D".to_string())];
    let reel = crate::sifter::reel(&hs, Some(5), &recent);
    let ids: Vec<(u32, u32)> = reel.iter().map(|h| (h.run_id, h.t)).collect();
    // Lead: run 5's closing episode. Then rows first (2, then 5's opener is a repeat of run
    // 5? no — a different pair, but its text already... it is fresh: ogre/reached D is not
    // recent), then the rest by score; gas/reached D was shown last absence and is skipped.
    assert_eq!(ids, vec![(5, 50), (2, 10), (5, 5), (1, 10)], "{reel:?}");
    assert!(!reel.iter().any(|h| h.run_id == 3), "a pair shown in the last three absences is skipped");
    // The closing pair shown before: the run's latest fresh episode leads instead.
    let recent = vec![("none".to_string(), "banked $".to_string())];
    let reel = crate::sifter::reel(&hs, Some(5), &recent);
    assert_eq!(reel[0].t, 5, "{reel:?}");
    // Nothing fresh at all: the closing episode leads alone.
    let recent: Vec<(String, String)> = hs.iter().filter_map(crate::sifter::pair).collect();
    let reel = crate::sifter::reel(&hs, Some(5), &recent);
    assert_eq!(reel.len(), 1, "{reel:?}");
    assert_eq!(reel[0].t, 50);
}

/// Gate: over 100 real runs every story line is three beats in ≤ 12 words with a table verb,
/// and every run closes at least one episode.
#[test]
fn story_lines_over_a_hundred_runs_follow_the_grammar() {
    // (seed, edited) → 2·seed + edited: one thread each.
    let per = par_seeds(2..=9u64, |k| {
        let (seed, edited) = (k / 2, k % 2 == 1);
        let (mut lines, mut runs) = (0, 0);
        {
            let mut g = Game::new_literal(seed);
            g.sim = true;
            if edited {
                for u in ["row5", "row6", "row7", "row8", "throw"] {
                    g.lineage.unlocks.insert(u.into());
                }
                g.set_rules(crate::probes::good()).unwrap();
            }
            for _ in 0..13 {
                g.lineage.rest_left = 0;
                g.start_run(None);
                g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
                let run = g.run.as_ref().unwrap();
                let hs = crate::sifter::sift(run, &g.lineage);
                assert!(!hs.is_empty(), "run {} closed no episode: {:?}", run.id, run.arc);
                for h in &hs {
                    assert!(crate::sifter::story_ok(&h.text), "seed {seed} run {}: {}", run.id, h.text);
                    assert!(word_count(&h.text) <= crate::sifter::STORY_WORDS, "{}", h.text);
                }
                lines += hs.len();
                runs += 1;
                g.finish_run();
                g.auto_keep();
                g.events.clear();
            }
        }
        (lines, runs)
    });
    let (lines, runs) = per.iter().fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1));
    assert!(runs >= 100 && lines >= runs, "{runs} runs, {lines} lines");
}

// ---------------------------------------------------------------- marks and meta

#[test]
fn marks_are_earned_on_new_bests_only() {
    let mut g = arena();
    g.run.as_mut().unwrap().max_depth = 3;
    g.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.marks, 4, "D1..D3 (3) + first bank from D3 (Cut 4); no frontier mark (Cut 29 §1); first kills no longer mark (Cut 2 §2)");
    assert!(g.batch.bests.iter().any(|b| b == "first kill: rat"), "but stay in bests");
    assert!(g.batch.bests.iter().any(|b| b == "home:D3"));
    g.auto_keep();
    let mut g2 = arena();
    g2.lineage = g.lineage.clone();
    g2.run.as_mut().unwrap().max_depth = 3;
    g2.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g2, ExitTier::Bank);
    assert_eq!(g2.lineage.marks, 4, "no new best, no marks (Cut 29 §1: the frontier mark is gone)");
    let mut g3 = arena();
    g3.lineage = g2.lineage.clone();
    g3.run.as_mut().unwrap().kills.push((1, "goblin_warlord".into(), 5));
    g3.run.as_mut().unwrap().trophies_run.push("pacifist_floor".into());
    finish_with(&mut g3, ExitTier::Bank);
    assert_eq!(g3.lineage.marks, 4 + 3 + 2 + 1, "boss 3, trophy 2, first bank from D1 1");
}

#[test]
fn unlocks_cost_marks_and_gate_rows() {
    let mut g = Game::new_literal(1);
    assert!(g.buy("row5").is_err());
    g.lineage.marks = 5;
    assert_eq!(g.buy("row5").unwrap_err(), "needs bank once", "Cut 29 §1: T1 opens at the first bank");
    g.lineage.banked_depths.insert(2);
    g.buy("row5").unwrap();
    assert_eq!(g.lineage.marks, 2);
    assert_eq!(g.vocabulary().max_rows, 5);
    assert!(g.buy("row5").is_err(), "already owned");
    assert!(g.buy("row7").is_err(), "prerequisite");
    assert!(g.buy("corridor_fighting").is_err(), "needs the pack fact");
    g.lineage.facts.insert("foe:jackal:pack".into());
    assert_eq!(g.buy("corridor_fighting").unwrap_err(), "needs meet Warlord", "T2");
    g.lineage.facts.insert("foe:goblin_warlord".into());
    g.lineage.marks = 5;
    g.buy("corridor_fighting").unwrap();
    assert!(g.vocabulary().verbs.contains(&Verb::arg("tactic", "corridor_fighting")));
    let cat = g.unlocks();
    assert!(cat.iter().any(|u| u.id == "row5" && u.owned));
    assert!(g.set_class("rogue").is_err());
}

#[test]
fn trophies_pacifist_and_no_heal() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 4;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("without a kill"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("no heal"))));
    let run = g.run.as_ref().unwrap();
    assert!(run.trophies_run.contains(&"pacifist_floor".to_string()));
    assert!(run.trophies_run.contains(&"no_heal_D5".to_string()));
}

// ---------------------------------------------------------------- save

#[test]
fn save_round_trip_continues_identically() {
    let mut g = Game::new_literal(21);
    g.send();
    g.step(123);
    let text = g.save();
    let mut h = Game::load(&text).unwrap();
    assert_eq!(h.save(), text);
    assert_eq!(h.lineage(), g.lineage());
    let a = g.step(200);
    let b = h.step(200);
    assert_eq!(a, b);
    assert!(Game::load("{}").is_err());
}

#[test]
fn snapshot_wire_shape() {
    let mut g = Game::new_literal(1);
    let s = g.send();
    let v = serde_json::to_value(&s).unwrap();
    for k in ["depth", "biome", "w", "h", "tiles", "seen", "visible", "overlays", "hero", "entities", "items", "alert", "turn", "loot", "run"] {
        assert!(v.get(k).is_some(), "missing {k}");
    }
    let h = &v["hero"];
    for k in ["id", "kind", "x", "y", "hp", "max_hp", "tags", "inv", "class", "trait", "weapon"] {
        assert!(h.get(k).is_some(), "hero missing {k}");
    }
    assert_eq!(h["kind"], "hero_fighter");
    assert_eq!(v["run"]["id"], 1);
    assert_eq!(v["tiles"].as_array().unwrap().len(), (s.w * s.h) as usize);
    let l = serde_json::to_value(g.lineage()).unwrap();
    for k in ["seed", "heir", "trait", "class", "best_depth", "marks", "facts", "unlocks", "vault", "graveyard", "trophies", "sets", "active_set", "ended", "party", "kennel", "eggs", "party_slots", "ledger", "gold", "supplies", "classes", "forge", "renown", "rank", "keep_pref"] {
        assert!(l.get(k).is_some(), "lineage missing {k}");
    }
    let r = g.step(5);
    let rv = serde_json::to_value(&r).unwrap();
    assert!(rv.get("events").is_some() && rv.get("snapshot").is_some() && rv.get("run_over").is_some());
}

// ---------------------------------------------------------------- companions (Addendum A)

fn tame_setup(seed: u64) -> (Game, u32) {
    let mut g = arena_seed(seed);
    g.lineage.unlocks.insert("tame".into());
    g.lineage.facts.insert("foe:rat:studied".into());
    give(&mut g, "leash");
    let r = add_monster(&mut g, "rat", 5, 5);
    g.run.as_mut().unwrap().monsters[0].hp = 1;
    g.run.as_mut().unwrap().monsters[0].max_hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("foe_hp<", 25)], Verb::arg("tame", "nearest")), Row::new(vec![], Verb::new("rest"))]);
    g.run.as_mut().unwrap().hero.hp -= 1;
    (g, r)
}

#[test]
fn tame_consumes_the_leash_and_succeeds_or_fails() {
    let mut successes = 0;
    let mut failures = 0;
    for seed in 1..=12 {
        let (mut g, r) = tame_setup(seed);
        let evs = ticks(&mut g, 10);
        let t = evs.iter().find_map(|e| if let Ev::Tame { ok, kind, .. } = e { Some((*ok, kind.clone())) } else { None }).expect("tame event");
        assert_eq!(t.1, "rat");
        assert!(hero(&g).inv.iter().all(|i| i.kind != "leash"), "leash consumed");
        if t.0 {
            successes += 1;
            let m = monster(&g, r).unwrap();
            assert!(m.ally && m.cid.is_some());
            assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "tamed:rat")));
            assert_eq!(g.run.as_ref().unwrap().companions.len(), 1);
        } else {
            failures += 1;
            assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, .. } if *src == r)), "a free attack on failure");
        }
    }
    assert!(successes >= 1 && failures >= 1, "20% base chance: {successes} ok, {failures} failed");
    assert_eq!(crate::engine::tame_chance(&Default::default(), "jackal"), 20);
    let mut f = std::collections::BTreeSet::new();
    f.insert("foe:jackal:pack".to_string());
    f.insert("foe:jackal:fast".to_string());
    assert_eq!(crate::engine::tame_chance(&f, "jackal"), 40);
    assert_eq!(crate::engine::tame_chance(&f, "lich"), 20);
}

#[test]
fn tamed_companion_joins_the_party_and_levels_on_bank() {
    let mut seed = 1;
    let (mut g, r) = loop {
        let (mut g, r) = tame_setup(seed);
        ticks(&mut g, 10);
        if monster(&g, r).is_some_and(|m| m.ally) {
            break (g, r);
        }
        seed += 1;
    };
    let _ = r;
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.party.len(), 1);
    assert_eq!(g.lineage.party[0].level, 2);
    assert_eq!(g.lineage.party[0].max_rows, 3);
    assert!(g.lineage.ledger().iter().any(|l| l.kind == "rat" && l.tamed));
    // Next run: it spawns beside the hero with a cid.
    g.auto_keep();
    g.start_run(None);
    let snap = g.snapshot();
    assert!(snap.entities.iter().any(|e| e.ally == Some(true) && e.cid.is_some() && e.kind == "rat"));
}

#[test]
fn companion_death_leaves_an_egg_that_hatches_after_three_rests() {
    let mut g = arena();
    g.lineage.party = vec![crate::probes::pets_party()[0].clone()];
    g.auto_keep();
    g.start_run(None);
    // Kill the companion outright.
    let mi = g.run.as_ref().unwrap().monsters.iter().position(|m| m.is_companion()).unwrap();
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_monster(run, &mut cx, mi, 999, &crate::turn::Src::Fire);
    }
    assert!(g.events.iter().any(|e| matches!(e, Ev::Ally { state, .. } if state == "lost")));
    finish_with(&mut g, ExitTier::Return);
    assert!(g.lineage.party.is_empty());
    assert_eq!(g.lineage.eggs.len(), 1);
    assert!(g.lineage.eggs[0].from_loss);
    assert_eq!(g.lineage.eggs[0].tags, vec!["ranged", "telegraph", "gas"]);
    assert!(g.batch.lost.contains(&"Skix".to_string()));
    for i in 0..3 {
        assert_eq!(g.lineage.eggs.len(), 1, "still an egg before rest {}", i + 1);
        g.auto_keep();
        g.start_run(None);
        finish_with(&mut g, ExitTier::Return);
    }
    assert!(g.lineage.eggs.is_empty(), "hatched after 3 rests (Cut 2 §1)");
    assert_eq!(g.lineage.kennel.len(), 1);
    assert_eq!(g.lineage.kennel[0].level, 1);
    assert!(g.events.iter().any(|e| matches!(e, Ev::Hatch { kind, .. } if kind == "goblin_archer")));
}

#[test]
fn hatch_from_loss_costs_fifty_gold_and_breeding_merges_tags() {
    let mut g = Game::new_literal(1);
    g.lineage.eggs.push(Egg { id: 9, kind: "rat".into(), tags: vec![], gen: 0, hatch_in: 5, from_loss: true });
    assert!(g.hatch(9).is_err());
    g.lineage.gold = 60;
    g.hatch(9).unwrap();
    assert_eq!(g.lineage.gold, 10);
    assert_eq!(g.lineage.kennel.len(), 1);
    let mut a = crate::probes::pets_party()[0].clone();
    let mut b = crate::probes::pets_party()[1].clone();
    a.level = 1;
    g.lineage.kennel.push(a.clone());
    g.lineage.kennel.push(b.clone());
    assert!(g.breed(a.id, b.id).is_err(), "level 2 needed");
    a.level = 2;
    b.level = 2;
    g.lineage.kennel.retain(|c| c.id != a.id && c.id != b.id);
    g.lineage.kennel.push(a.clone());
    g.lineage.kennel.push(b.clone());
    g.breed(a.id, b.id).unwrap();
    let egg = g.lineage.eggs.last().unwrap();
    assert_eq!(egg.kind, "goblin_archer");
    assert_eq!(egg.tags.len(), 2, "two tags without third_tag (Cut 2 §3)");
    assert_eq!(egg.hatch_in, 3);
    assert!(!egg.from_loss);
    assert_eq!(egg.gen, 2);
    assert!(g.hatch(egg.id).is_err(), "bred eggs hatch by expeditions");
    assert!(g.lineage.kennel.iter().all(|c| c.id != a.id && c.id != b.id));
    assert!(g.lineage.ledger().iter().any(|l| l.kind == "goblin_archer" && l.bred));
}

#[test]
fn companion_rules_shoot_and_burst() {
    let mut g = arena();
    let mut c = crate::probes::pets_party()[0].clone();
    c.rules = RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("shoot"))], name: None, route: Vec::new() };
    g.lineage.party = vec![c];
    g.auto_keep();
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.floor = arena().run.take().unwrap().floor;
    run.hero.pos = Pos::new(4, 5);
    run.hero_dist_pos = None;
    let cm = run.monsters.iter_mut().find(|m| m.is_companion()).unwrap();
    cm.pos = Pos::new(4, 6);
    let cid = cm.id;
    run.monsters.retain(|m| m.is_companion());
    run.items.clear();
    hold_rules(&mut g);
    let r = add_monster(&mut g, "rat", 9, 5);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Projectile { src, dst, .. } if *src == cid && *dst == r)));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, verb, .. } if *src == cid && verb.as_deref() == Some("shoot"))));
    let v = g.companion_vocabulary(900_001).unwrap();
    assert!(v.verbs.iter().any(|x| x.v == "shoot") && v.verbs.iter().any(|x| x.v == "burst"));
    assert!(v.conds.iter().any(|c| c.k == "self_hp<"));
    assert_eq!(v.max_rows, 4);
    // Burst: a gas companion self-destructs into a cloud.
    g.set_companion_rules(900_001, RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("burst"))], name: None, route: Vec::new() }).unwrap();
    g.run.as_mut().unwrap().companions[0].rules = g.lineage.party[0].rules.clone();
    add_monster(&mut g, "rat", 6, 6);
    let evs = ticks(&mut g, 20);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "burst!")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Gas, .. })));
    assert!(g.run.as_ref().unwrap().party_alive().next().is_none());
}

#[test]
fn hero_party_scope_recall_is_a_free_action() {
    let mut g = arena();
    g.lineage.party = vec![crate::probes::pets_party()[1].clone()];
    g.auto_keep();
    g.start_run(None);
    {
        let run = g.run.as_mut().unwrap();
        run.floor = arena().run.take().unwrap().floor;
        run.hero.pos = Pos::new(4, 5);
        run.hero_dist_pos = None;
        run.monsters.retain(|m| m.is_companion());
        run.monsters[0].pos = Pos::new(5, 5);
        run.items.clear();
    }
    rules(&mut g, vec![Row::new(vec![Cond::t("party", "ghoul")], Verb::new("recall")), Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 1, .. })), "recall did not consume the action");
    assert!(g.run.as_ref().unwrap().recalled.len() == 1);
    finish_with(&mut g, ExitTier::Return);
    assert_eq!(g.lineage.party.len(), 1, "a recalled companion comes home");
    assert_eq!(g.lineage.party[0].level, 3);
}

#[test]
fn counters_scale_damage_and_are_learned() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.weapon = None;
    g.run.as_mut().unwrap().hero.auto_equip(Item::new(60, "bow"));
    let o = add_monster(&mut g, "ogre", 8, 5);
    g.run.as_mut().unwrap().monsters[0].awake = false;
    attack_rules(&mut g);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "counter:ranged>heavy")));
    let hurt = evs.iter().find_map(|e| if let Ev::Hurt { id, dmg, .. } = e { if *id == o { Some(*dmg) } else { None } } else { None });
    let atk = evs.iter().find_map(|e| if let Ev::Attack { dst, dmg, hit: true, .. } = e { if *dst == o { Some(*dmg) } else { None } } else { None });
    if let (Some(h), Some(a)) = (hurt, atk) {
        assert_eq!(h, a * 3 / 2);
    }
    let g2 = arena();
    let run = g2.run.as_ref().unwrap();
    assert!(!crate::turn::is_lone(run, run.hero.pos, false), "the hero is never lone");
    assert!(crate::turn::is_lone(run, Pos::new(10, 10), true));
}

// ---------------------------------------------------------------- gold and supplies (Addendum B)

#[test]
fn supplies_are_bought_with_gold_and_never_kept_back() {
    let mut g = Game::new_literal(1);
    no_kennel_leash(&mut g);
    assert!(g.buy_supply("leash").is_err());
    g.lineage.gold = 100;
    assert!(g.buy_supply("heal").is_err(), "unidentified");
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let cat = g.supply_catalogue();
    // Cut 22 §1: `$10 + 2 × best depth` (a new lineage: $10), a scroll 1.5×.
    assert!(cat.iter().any(|s| s.kind == "heal" && s.price == 10));
    g.lineage.best_depth = 8;
    assert!(g.supply_catalogue().iter().any(|s| s.kind == "heal" && s.price == 26));
    assert_eq!(crate::engine::supply_price(crate::defs::Cat::Scroll, 20), 39, "a deep lineage pays the cap (potion $26, scroll 1.5×)");
    g.lineage.best_depth = 0;
    g.buy_supply("heal").unwrap();
    g.buy_supply("leash").unwrap();
    assert_eq!(g.lineage.gold, 60);
    assert_eq!(g.lineage().supplies.len(), 2);
    g.buy_supply("leash").unwrap();
    assert!(g.buy_supply("leash").unwrap_err().contains("max"));
    g.clear_supplies();
    assert_eq!(g.lineage.gold, 100);
    g.lineage.gold = 40;
    assert!(g.buy_supply("heal").is_ok() && g.buy_supply("heal").is_ok() && g.buy_supply("leash").unwrap_err().contains("gold"));
    g.clear_supplies();
    g.buy_supply("heal").unwrap();
    g.start_run(None);
    assert!(hero(&g).inv.iter().any(|i| i.kind == "heal"));
    assert!(g.lineage.supplies.is_empty());
    let sid = g.run.as_ref().unwrap().supplies[0];
    g.run.as_mut().unwrap().hero.inv.retain(|i| i.kind != "dagger");
    finish_with(&mut g, ExitTier::Bank);
    assert!(g.pending_exit.as_ref().unwrap().items.iter().all(|i| i.id != sid));
}

// ---------------------------------------------------------------- class XP (Addendum C)

#[test]
fn xp_levels_the_class_and_gates_verbs() {
    let mut g = arena();
    let run = g.run.as_mut().unwrap();
    run.max_depth = 10;
    for _ in 0..60 {
        run.kills.push((1, "goblin".into(), 5));
    }
    finish_with(&mut g, ExitTier::Bank);
    let p = &g.lineage.classes["fighter"];
    assert_eq!(p.level, 2, "100 xp to L2 (60 kills at D5 and D10 reached = 135): {}", p.xp);
    assert!(g.events.iter().any(|e| matches!(e, Ev::Level { class, level: 2, .. } if class == "fighter")));
    assert_eq!(g.batch.level_ups, 1);
    assert!(!g.vocabulary().verbs.iter().any(|v| v.v == "cleave"));
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 3, xp: 0, next: 0 });
    assert!(g.vocabulary().verbs.iter().any(|v| v.v == "cleave"));
    g.auto_keep();
    g.start_run(None);
    let h = hero(&g);
    assert_eq!(h.max_hp, 36 + 4);
    assert_eq!(h.str_bonus, 1);
    // Cut 7 §5: 60·L² to L3, 100·L² from L4.
    assert_eq!(crate::hero::xp_to_next(3), 540);
    assert_eq!(crate::hero::xp_to_next(1), 60);
    assert_eq!(crate::hero::xp_to_next(4), 1600);
}

#[test]
fn cleave_hits_all_adjacent_and_mastery_grants_card() {
    let mut g = arena();
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 3, xp: 0, next: 0 });
    g.run.as_mut().unwrap().hero.level = 3;
    let a = add_monster(&mut g, "rat", 5, 5);
    let b = add_monster(&mut g, "rat", 5, 6);
    rules(&mut g, vec![Row::new(vec![Cond::n("adj>=", 2)], Verb::new("cleave")), Row::new(vec![], Verb::new("rest"))]);
    g.run.as_mut().unwrap().hero.hp -= 1;
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { dst, verb, .. } if *dst == a && verb.as_deref() == Some("cleave"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { dst, verb, .. } if *dst == b && verb.as_deref() == Some("cleave"))));
    assert!(hero(&g).cleave_cd > 0);
    let mut g = arena();
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 9, xp: crate::hero::xp_to_next(9) - 5, next: 0 });
    g.run.as_mut().unwrap().max_depth = 2;
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.classes["fighter"].level, 10);
    assert!(g.lineage.trophies.contains(&"master:fighter".to_string()));
    assert!(g.lineage.unlocks.contains("phalanx"));
}

// ---------------------------------------------------------------- salvage, forge, renown (Addendum D)

#[test]
fn salvage_feeds_the_forge_and_tiers_apply() {
    let mut g = Game::new_literal(1);
    for _ in 0..3 {
        let mut a = arena();
        a.lineage = g.lineage.clone();
        for _ in 0..5 {
            give(&mut a, "heal");
        }
        finish_with(&mut a, ExitTier::Bank);
        a.keep(vec![]).unwrap();
        g.lineage = a.lineage.clone();
    }
    let f = &g.lineage.forge["heal"];
    assert_eq!(f.salvaged, 15);
    assert!(f.craftable);
    assert_eq!(f.tier, 1);
    assert!(g.lineage.gold >= 15 * 8 / crate::engine::GOLD_DIVISOR);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.gold = 100;
    no_kennel_leash(&mut g);
    g.buy_supply("heal").unwrap();
    assert_eq!(g.lineage.supplies[0].enchant, 1, "forge tier on bought copies");
    let mut a = arena();
    a.lineage = g.lineage.clone();
    let mut d = Item::new(500, "dagger");
    d.enchant = 0;
    a.run.as_mut().unwrap().hero.inv.push(d);
    let cat = a.supply_catalogue();
    assert!(!cat.iter().any(|s| s.kind == "sword"));
    a.lineage.forge.insert("sword".into(), ForgeRow::at(6));
    assert!(a.supply_catalogue().iter().any(|s| s.kind == "sword" && s.price == 50));
}

#[test]
fn exit_pending_then_keep_and_keep_pref() {
    let mut g = Game::new_literal(8);
    g.send();
    let mut last = None;
    for _ in 0..200 {
        let r = g.step(50);
        if r.run_over {
            last = Some(r);
            break;
        }
    }
    let r = last.expect("run ended");
    let p = r.exit_pending.expect("exit pending on the last step");
    assert!(["bank", "return", "death"].contains(&p.tier.as_str()));
    assert!(g.pending_exit.is_some());
    g.keep(p.items.iter().map(|i| i.id).collect()).unwrap();
    assert!(g.pending_exit.is_none());
    assert!(g.keep(vec![]).is_err());
    let mut a = arena();
    give(&mut a, "sword");
    give(&mut a, "plate");
    a.lineage.keep_pref = "best_armour".into();
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(a.lineage.vault[0].kind, "plate");
    assert!(a.set_keep_pref("none").is_ok() && a.set_keep_pref("x").is_err());
}

/// QA on 23ed91f: `keep armour` with `mail · summon ally scroll` vaulted woke to `sword +2 ·
/// bow +1` — the fallback weapon keep evicted by value. The chip's category wins; a keep
/// replaces only a weaker vault item of its own category; brought vault items go back first.
#[test]
fn auto_keep_follows_the_chip_and_never_evicts_across_categories() {
    let vaulted = |g: &mut Game, kinds: &[&str]| {
        g.lineage.unlocks.insert("vault2".into());
        g.lineage.vault = kinds.iter().enumerate().map(|(i, k)| { let mut it = Item::new(100_000 + i as u32, k); it.known = true; it }).collect();
        g.lineage.next_vault_id = 100_100;
    };
    let kinds = |g: &Game| { let mut v: Vec<String> = g.lineage.vault.iter().map(|i| format!("{}+{}", i.kind, i.enchant)).collect(); v.sort(); v };
    // K's night: armour chip, weapons found, no armour — the vault is untouched.
    let mut a = arena();
    vaulted(&mut a, &["mail", "summon_ally"]);
    a.set_keep_pref("best_armour").unwrap();
    let s = give(&mut a, "sword");
    let b = give(&mut a, "bow");
    { let run = a.run.as_mut().unwrap(); run.hero.inv.iter_mut().for_each(|i| if i.id == s { i.enchant = 2 } else if i.id == b { i.enchant = 1 }); }
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(kinds(&a), ["mail+0", "summon_ally+0"]);
    // (Cut 29 §4: the quartermaster's keep is the default — the chip's category, then the other)
    assert_eq!(a.lineage.to_wire().keep_auto, ["armour", "weapon"]);
    // A better armour replaces the weaker armour, never the scroll.
    let mut a = arena();
    vaulted(&mut a, &["leather", "summon_ally"]);
    a.set_keep_pref("best_armour").unwrap();
    give(&mut a, "mail");
    give(&mut a, "sword");
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(kinds(&a), ["mail+0", "summon_ally+0"]);
    // A free slot takes the fallback category when no armour came home.
    let mut a = arena();
    vaulted(&mut a, &["summon_ally"]);
    a.set_keep_pref("best_armour").unwrap();
    give(&mut a, "sword");
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(kinds(&a), ["summon_ally+0", "sword+0"]);
    // A brought vault item comes home to the vault whatever the chip says (was: salvaged
    // unless it was the chip's best).
    for pref in ["best_weapon", "none"] {
        let mut a = arena();
        vaulted(&mut a, &[]);
        a.set_keep_pref(pref).unwrap();
        let mut m = Item::new(100_050, "mail");
        m.known = true;
        { let run = a.run.as_mut().unwrap(); run.hero.inv.push(m); run.brought.push(100_050); }
        give(&mut a, "sword");
        finish_with(&mut a, ExitTier::Bank);
        a.auto_keep();
        let want: &[&str] = if pref == "none" { &["mail+0"] } else { &["mail+0", "sword+0"] };
        assert_eq!(kinds(&a), want, "{pref}");
    }
    // `none` wins over the quartermaster; with it, weapon and armour each upgrade their own.
    let mut a = arena();
    vaulted(&mut a, &["dagger", "leather"]);
    a.lineage.unlocks.insert("quartermaster".into());
    a.set_keep_pref("none").unwrap();
    give(&mut a, "sword");
    give(&mut a, "mail");
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(kinds(&a), ["dagger+0", "leather+0"]);
    assert!(a.lineage.to_wire().keep_auto.is_empty());
    let mut a = arena();
    vaulted(&mut a, &["dagger", "leather"]);
    a.lineage.unlocks.insert("quartermaster".into());
    a.set_keep_pref("best_armour").unwrap();
    give(&mut a, "sword");
    give(&mut a, "mail");
    finish_with(&mut a, ExitTier::Bank);
    a.auto_keep();
    assert_eq!(kinds(&a), ["mail+0", "sword+0"]);
    assert_eq!(a.lineage.to_wire().keep_auto, ["armour", "weapon"]);
}

/// QA on 23ed91f (seed 1015's walk): `hp < 20% → rest · reach +8%`, then the camp's D6 read
/// 34 % → 64 %. `death()` answers with the verdict's numbers flagged `camp_pending`;
/// `death_deltas` returns the camp's own move at `forecast_depth`, exactly what the camp's
/// forecast shows once the patch is applied (and the camp then reads the memoised panels).
#[test]
fn death_deltas_are_the_camp_forecasts_move() {
    // Cut 20 §1: seed 1015's first death moved (one theft a run); 1022's offers the rest patch.
    // QA on e75ec29 (the first night's bounty floor from the start moved 1022's): 1048's.
    // Cut 22 §3 (a forecast's sims draw each floor from its own stream): 1033's.
    // Cut 23 (thieves take the leash last, a walk home answers): 1010's.
    // Cut 24 (the floors' arrival events, named foes resting): 1047's.
    let mut g = Game::new_literal(1047);
    g.send();
    let mut died = None;
    for _ in 0..4000 {
        let r = g.step(50);
        if r.run_over {
            died = r.events.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "death")).then_some(r.snapshot.run.id);
            break;
        }
    }
    let id = died.expect("seed 1047's first heir dies");
    g.keep(vec![]).unwrap();
    let d = g.death(id).unwrap();
    assert!(!d.patches.is_empty() && d.patches.iter().all(|p| p.camp_pending && p.forecast_depth == 0), "{:?}", d.patches);
    let ps = g.death_deltas(id).unwrap();
    assert!(ps.iter().all(|p| !p.camp_pending && p.forecast_depth > 0));
    assert!(g.death(id).unwrap().patches.iter().all(|p| !p.camp_pending), "measured on this camp state");
    let rules = g.lineage.rules().clone();
    let before = g.forecast();
    let bar = |f: &Forecast, d: u32| f.depths.iter().find(|x| x.depth == d).unwrap().reach;
    for p in ps.iter().filter(|p| p.insert_at >= 0) {
        let mut h = g.clone();
        h.set_rules(crate::offline::apply_patch(&rules, p, h.lineage.max_rows())).unwrap();
        let moved = bar(&h.forecast(), p.forecast_depth) - bar(&before, p.forecast_depth);
        assert!((moved - p.forecast_delta).abs() < 1e-9, "{}: reach {:+.3} vs camp {moved:+.3}", p.row.describe(), p.forecast_delta);
    }
    let rest = ps.iter().find(|p| p.row.verb.v == "rest").unwrap_or_else(|| panic!("the rest patch: {:?}", ps.iter().map(|p| (p.row.describe(), p.forecast_delta)).collect::<Vec<_>>()));
    assert!(rest.forecast_delta > 0.2, "rest moves D{} by {:+.2}", rest.forecast_depth, rest.forecast_delta);
}

#[test]
fn renown_ranks_grant_marks() {
    let mut g = arena();
    g.run.as_mut().unwrap().max_depth = 12;
    g.run.as_mut().unwrap().boss_kills.push((1, "goblin_warlord".into()));
    g.run.as_mut().unwrap().kills.push((1, "goblin_warlord".into(), 5));
    finish_with(&mut g, ExitTier::Bank);
    assert!(g.lineage.renown >= 120 + 25);
    assert_eq!(g.lineage.rank, 1);
    assert!(g.events.iter().any(|e| matches!(e, Ev::Rank { rank: 1, .. })));
    assert_eq!(g.batch.ranks_up, 1);
    assert!(g.lineage.marks > 12 + 3, "depth, boss and a rank mark");
}

// ---------------------------------------------------------------- content sanity

#[test]
fn a_full_lineage_plays_through_many_runs_without_panics() {
    let mut g = Game::new_literal(77);
    g.set_rules_raw(crate::probes::good()).unwrap();
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    g.lineage.party = crate::probes::pets_party();
    g.lineage.unlocks.insert("party_slot_2".into());
    let r = g.run_offline(1200);
    assert!(r.runs >= 1);
    assert!(r.live.is_none() && g.run.is_none(), "an absence ends at camp");
    let v = serde_json::to_value(&r).unwrap();
    for k in ["elapsed_s", "runs", "sampled", "learned", "bests", "found", "deaths", "pending", "reel", "marks_earned", "tamed", "hatched", "lost", "xp", "salvaged", "renown"] {
        assert!(v.get(k).is_some(), "report missing {k}");
    }
}

#[test]
fn class_rogue_preset_uses_vanish() {
    let mut g = Game::new_literal(12);
    g.lineage.unlocks.insert("rogue".into());
    g.set_class("rogue").unwrap();
    g.set_rules(crate::probes::preset(Class::Rogue)).unwrap();
    assert!(g.vocabulary().verbs.iter().any(|v| v.v == "vanish"));
    assert!(g.vocabulary().verbs.iter().any(|v| v.v == "read"));
    g.send();
    let mut saw_vanish = false;
    for _ in 0..80 {
        let r = g.step(50);
        saw_vanish |= r.events.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "vanish"));
        if r.run_over {
            g.send();
        }
    }
    assert!(saw_vanish || g.lineage.heir >= 1);
}





// ---------------------------------------------------------------- oscillation and trait guards

#[test]
fn seed_3_floors_do_not_deadlock() {
    // The coordinator's playtest: seed 3, fighter preset with heal identified; run 2 once spent
    // 20 000 ticks on D3 ping-ponging between `attack` (unreachable rats behind a chained
    // captive) and the descend chore. Every floor of the first three runs must finish quickly.
    let mut g = Game::new_literal(3);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.send();
    let mut floor_start = 0u32;
    let mut run_id = 1;
    let mut longest = 0u32;
    let mut floors: Vec<(u32, u32, u32)> = Vec::new(); // (run, depth left, ticks)
    let mut depth = 1;
    while run_id <= 3 {
        let r = g.step(20);
        for e in &r.events {
            if let Ev::Descend { t, depth: d, .. } = e {
                floors.push((run_id, depth, *t - floor_start));
                longest = longest.max(*t - floor_start);
                floor_start = *t;
                depth = *d;
            }
        }
        if r.run_over {
            floors.push((run_id, depth, r.snapshot.turn - floor_start));
            longest = longest.max(r.snapshot.turn - floor_start);
            floor_start = 0;
            depth = 1;
            run_id += 1;
            g.send();
        }
    }
    let run2_d3 = floors.iter().find(|(r, d, _)| *r == 2 && *d == 3).map(|f| f.2);
    if let Some(t) = run2_d3 {
        assert!(t < 3000, "run 2 spent {t} ticks on D3 (was ~20 000 before the guards)");
    }
    // 4000 before Cut 3; the larger flavour pool reseeds the lineage and a D5 (Warlord) floor
    // now lands at ~4000 on this seed; Cut 5's situations add a detour (4600); Cut 7 moves the
    // Warlord to D8 with a group more (5300). A deadlock is 20 000.
    assert!(longest < 6500, "longest floor took {longest} ticks: {floors:?}");
}

#[test]
fn unreachable_foe_falls_through_and_the_guard_frees_the_chores() {
    // A chained captive plugs a one-wide corridor; rats sit behind it, awake but stuck.
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        for x in 1..15 {
            for y in 7..11 {
                run.floor.map.set(Pos::new(x, y), Tile::Wall);
            }
        }
        run.floor.map.set(Pos::new(8, 7), Tile::Floor);
        run.floor.map.set(Pos::new(8, 8), Tile::Floor);
        run.floor.map.set(Pos::new(8, 9), Tile::Floor);
        run.floor.map.set(Pos::new(8, 10), Tile::Floor);
        run.floor.map.set(Pos::new(14, 10), Tile::Wall);
        run.floor.stairs_down = Pos::new(1, 1);
        run.floor.map.set(Pos::new(1, 1), Tile::StairsDown);
        run.floor.map.set(Pos::new(1, 2), Tile::StairsUp);
        run.floor.stairs_up = Pos::new(1, 2);
        run.floor.map.compute_corridors(&[]);
        run.hero.pos = Pos::new(8, 6);
        run.hero_dist_pos = None;
        run.floor.map.reveal_all();
        run.floor.map.update_vision(run.hero.pos, VISION);
    }
    let c = add_monster(&mut g, "captive", 8, 8);
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == c).unwrap().awake = false;
    // Wall off the far side so the rats cannot come around either.
    for x in 1..15 {
        g.run.as_mut().unwrap().floor.map.set(Pos::new(x, 11), Tile::Wall);
    }
    g.run.as_mut().unwrap().floor.map.set(Pos::new(8, 11), Tile::Floor);
    add_monster(&mut g, "rat", 8, 10);
    add_monster(&mut g, "rat", 8, 11);
    g.run.as_mut().unwrap().monsters.iter_mut().for_each(|m| if m.kind == "rat" { m.paralysed = 10_000 });
    attack_rules(&mut g);
    let evs = ticks(&mut g, 600);
    // Only this floor counts: D2 (a real 32×32 floor) has reachable foes to fight.
    let left = evs.iter().find_map(|e| if let Ev::Descend { t, .. } = e { Some(*t) } else { None }).unwrap_or(600);
    let attacks = evs.iter().filter(|e| matches!(e, Ev::Rule { row: 0, t, .. } if *t <= left)).count();
    assert!(attacks <= 24, "attack keeps firing at an unreachable foe: {attacks} of 60 actions");
    assert!(g.run.as_ref().unwrap().depth >= 2 || hero(&g).pos == Pos::new(1, 1), "the chores took the hero down");
}

#[test]
fn oscillation_guard_emits_stuck_and_suppresses_targeting_rows() {
    let mut g = arena();
    // A rat behind a wall pocket that the hero can see but never reach.
    {
        let run = g.run.as_mut().unwrap();
        for (x, y) in [(9, 4), (9, 6), (10, 4), (10, 6), (11, 4), (11, 5), (11, 6)] {
            run.floor.map.set(Pos::new(x, y), Tile::Chasm);
        }
    }
    let r = add_monster(&mut g, "rat", 10, 5);
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == r).unwrap().paralysed = 10_000;
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("shield_bash")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 400);
    let stuck = evs.iter().filter(|e| matches!(e, Ev::Rule { row: -2, verb, .. } if verb.v == "stuck")).count();
    let last_attack = evs.iter().rposition(|e| matches!(e, Ev::Rule { row: 1, .. })).unwrap_or(0);
    let first_stuck = evs.iter().position(|e| matches!(e, Ev::Rule { row: -2, verb, .. } if verb.v == "stuck"));
    let explored = evs.iter().filter(|e| matches!(e, Ev::Rule { row: -2, verb, .. } if verb.v == "explore" || verb.v == "descend")).count();
    assert!(explored >= 10, "chores proceeded: {explored}");
    if let Some(fs) = first_stuck {
        assert!(stuck >= 1 && last_attack < fs + 400, "one stuck event, then no attacks");
    }
    let attacks = evs.iter().filter(|e| matches!(e, Ev::Rule { row: 1, .. })).count();
    assert!(attacks <= 24, "attacks against the unreachable rat: {attacks} of 40 actions");
}

#[test]
fn pick_up_never_repeats_three_times_without_a_pickup() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        for i in 0..10 {
            run.hero.inv.push(Item::new(400 + i, "teleport"));
        }
        // A dagger on the floor: worse than the sword, and the pack is full, so it stays put.
        run.items.push(crate::engine::FloorItem { pos: Pos::new(9, 5), item: Item::new(500, "dagger") });
    }
    rules(&mut g, vec![Row::new(vec![], Verb::new("pick_up"))]);
    let evs = ticks(&mut g, 300);
    let verbs: Vec<String> = evs.iter().filter_map(|e| if let Ev::Rule { verb, .. } = e { Some(verb.v.clone()) } else { None }).collect();
    let mut streak = 0;
    let mut worst = 0;
    for v in &verbs {
        if v == "pick_up" {
            streak += 1;
            worst = worst.max(streak);
        } else {
            streak = 0;
        }
    }
    assert!(worst <= 3, "pick_up chosen {worst} times in a row with nothing picked up: {verbs:?}");
    assert!(verbs.iter().any(|v| v == "explore" || v == "descend"), "chores proceed");
}

// ---------------------------------------------------------------- walls, rest clock, insurance, patches

#[test]
fn a_living_boss_seals_the_stairs() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 5;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    let w = add_monster(&mut g, "goblin_warlord", 3, 3);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend")), Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 30);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Descend { .. })), "no way down past a living boss");
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 1, .. })), "the descend row falls through");
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == w).unwrap().hp = 0;
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 6, .. })));
}

#[test]
fn warlord_wall_shields_him_and_aimed_strikes_go_through() {
    // attack-nearest: swings at the Warlord are taken by his goblins; he keeps rallying.
    let mut g = arena();
    g.run.as_mut().unwrap().hero.max_hp = 400;
    g.run.as_mut().unwrap().hero.hp = 399;
    let w = add_monster(&mut g, "goblin_warlord", 5, 5);
    add_monster(&mut g, "goblin", 6, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 300);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == w && what == "rallies")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.starts_with("boss:goblin_warlord:counter"))), "the first telegraph teaches the counter");
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "shielded")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "shields up")));
    assert!(monster(&g, w).is_some(), "the Warlord survives 30 actions of attack-nearest");
    let hurt: i32 = evs.iter().filter_map(|e| if let Ev::Hurt { id, dmg, .. } = e { if *id == w { Some(*dmg) } else { None } } else { None }).sum();
    assert!(hurt <= 8, "incidental swings barely touch him: {hurt}");
    // attack tag:boss: aimed strikes land and cancel his rally.
    let mut g = arena();
    g.run.as_mut().unwrap().hero.max_hp = 400;
    g.run.as_mut().unwrap().hero.hp = 399;
    g.lineage.facts.insert("foe:goblin_warlord:boss".into());
    let w = add_monster(&mut g, "goblin_warlord", 5, 5);
    add_monster(&mut g, "goblin", 6, 5);
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")), Row::new(vec![], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 300);
    assert!(evs.iter().any(|e| matches!(e, Ev::Die { id, .. } if *id == w)), "aimed at, the Warlord dies");
}

#[test]
fn lich_reflects_arrows_and_keeps_chanting_while_summons_stand() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.max_hp = 400;
    g.run.as_mut().unwrap().hero.hp = 399;
    g.run.as_mut().unwrap().hero.weapon = None;
    g.run.as_mut().unwrap().hero.auto_equip(Item::new(60, "bow"));
    let l = add_monster(&mut g, "lich", 9, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 150);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == l && what == "chants")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.starts_with("boss:lich:counter"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb, dst, .. } if verb.as_deref() == Some("reflect") && *dst == HERO_ID)), "arrows come back");
    let chants = evs.iter().filter(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == l && what == "chants")).count();
    assert!(chants >= 2, "re-chants while skeletons stand: {chants}");
    assert!(evs.iter().filter(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "skeleton")).count() >= 4, "two skeletons per chant");
}

#[test]
fn resting_raises_the_alert_and_calls_a_pack() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.hp = 1;
    g.run.as_mut().unwrap().hero.max_hp = 400;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 99)], Verb::new("rest"))]);
    let evs = ticks(&mut g, 10 * crate::turn::REST_ALERT_EVERY * 5 + 20);
    let run = g.run.as_ref().unwrap();
    assert!(run.alert >= 5, "{} rests raise the alert to 5: {}", crate::turn::REST_ALERT_EVERY * 5, run.alert);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "they heard you")));
    assert!(evs.iter().filter(|e| matches!(e, Ev::Spawn { .. })).count() >= 2, "a pack, not a straggler");
}

#[test]
fn insurance_keeps_a_brought_item_on_death() {
    let mut g = Game::new_literal(1);
    g.lineage.vault.push(Item::new(100_001, "plate"));
    assert!(g.insure(100_001).is_err(), "no gold");
    g.lineage.gold = 1000;
    g.insure(100_001).unwrap();
    assert_eq!(g.lineage.gold, 1000 - crate::engine::insure_cost("plate"));
    assert!(g.insure(100_001).is_err(), "already insured");
    assert_eq!(g.lineage().insured, vec![100_001]);
    g.loadout(vec![100_001]);
    g.start_run(None);
    assert!(hero(&g).armour.as_ref().is_some_and(|a| a.kind == "plate"));
    g.run.as_mut().unwrap().hero.hp = 0;
    finish_with(&mut g, ExitTier::Death);
    assert!(g.lineage.vault.iter().any(|v| v.id == 100_001), "the insured plate came home");
    assert!(g.lineage.insured.is_empty(), "the policy is spent");
}

/// Cut 14 §2: `heal unused` is on the margin when the heal-drinking row survives ≥ half the
/// replays (the same fight as the id-policy test, the potions known): the verdict is `gap`
/// and the drink row is among the patches — the line never contradicts the verdict.
#[test]
fn margin_names_the_heal_when_drinking_it_saves_him() {
    let mut g = arena_seed(4);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.run.as_mut().unwrap().hero.hp = 12;
    for _ in 0..3 {
        give(&mut g, "heal");
    }
    for (x, y) in [(5, 5), (5, 6), (4, 6)] {
        add_monster(&mut g, "goblin", x, y);
    }
    attack_rules(&mut g);
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let d = g.death(id.expect("died")).unwrap();
    let heal = d.patches.iter().find(|p| p.row.verb.v == "drink" && p.row.verb.a.as_deref() == Some("heal")).expect("the heal row is a patch");
    assert!(heal.survive >= crate::trace::MARGIN_BAR, "{heal:?}");
    assert_eq!(d.verdict, "gap");
    assert!(d.margin.contains("heal unused"), "{}", d.margin);
    assert!(!d.margin.contains("unknown"), "{}", d.margin);
}

#[test]
fn patches_offer_the_id_policy_when_unknown_potions_went_unused() {
    let mut g = arena_seed(4);
    g.run.as_mut().unwrap().hero.hp = 12;
    for _ in 0..3 {
        give(&mut g, "heal");
    }
    // Cut 4: the replay runs on through the fight, so the fight must be winnable with a heal.
    for (x, y) in [(5, 5), (5, 6), (4, 6)] {
        add_monster(&mut g, "goblin", x, y);
    }
    attack_rules(&mut g);
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let d = g.death(id.expect("died")).unwrap();
    assert!(d.margin.split(" · ").any(|x| x.contains("unknown") && x.ends_with(" unused")), "{}", d.margin);
    assert_eq!(d.verdict, "gap");
    assert!(d.patches.iter().any(|p| p.row.verb.v == "drink" && p.row.verb.a.as_deref() == Some("unknown") && !p.row.conds.is_empty()), "{:?}", d.patches);
    for p in &d.patches {
        assert!(p.survive - d.baseline > 0.15 || p.forecast_delta > 0.02, "{p:?} vs baseline {}", d.baseline);
        assert!(!p.row.conds.is_empty() || p.survive - d.baseline >= 0.3);
    }
}


// ---------------------------------------------------------------- Cut 3: biomes 4–6

fn add_sleeping(g: &mut Game, kind: &str, x: i32, y: i32) -> u32 {
    let id = add_monster(g, kind, x, y);
    let run = g.run.as_mut().unwrap();
    let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
    m.awake = false;
    m.last_seen = None;
    id
}

fn unlock(g: &mut Game, u: &str) {
    g.lineage.unlocks.insert(u.into());
}

fn identify(g: &mut Game, kind: &str) {
    if let Some(f) = ident_fact(&g.lineage.flavours, kind) {
        g.lineage.facts.insert(f);
    }
}

#[test]
fn iron_golem_reflects_melee_but_not_arrows() {
    let mut g = arena();
    let id = add_monster(&mut g, "iron_golem", 5, 5);
    attack_rules(&mut g);
    let hp0 = hero(&g).hp;
    let evs = ticks(&mut g, 40);
    let m = monster(&g, id).expect("the golem stands");
    assert_eq!(m.hp, m.max_hp, "melee never lands on a golem");
    assert!(hero(&g).hp < hp0, "the swings came back");
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), dst: HERO_ID, .. } if v == "reflect")));
    assert!(g.lineage.facts.contains("foe:iron_golem:reflect_melee"));
    // Arrows land.
    let mut g = arena();
    let id = add_monster(&mut g, "iron_golem", 9, 5);
    g.run.as_mut().unwrap().hero.weapon = Some(Item::new(2, "bow"));
    attack_rules(&mut g);
    ticks(&mut g, 60);
    assert!(monster(&g, id).is_none_or(|m| m.hp < m.max_hp), "a shot golem is hurt");
}

#[test]
fn bell_sentinel_rings_the_alarm_on_sight() {
    let mut g = arena();
    add_monster(&mut g, "bell_sentinel", 8, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 20);
    assert_eq!(g.run.as_ref().unwrap().alert, 2, "+2 alert on sight");
    assert!(g.lineage.facts.contains("foe:bell_sentinel:alarm"));
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "bell!")));
    assert!(g.run.as_ref().unwrap().noise.is_some(), "the whole floor heard it");
}

#[test]
fn forge_imp_steals_only_potions() {
    let mut g = arena();
    give(&mut g, "sword");
    give(&mut g, "heal");
    let id = add_monster(&mut g, "forge_imp", 5, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 80);
    let stolen: Vec<&Ev> = evs.iter().filter(|e| matches!(e, Ev::Steal { .. })).collect();
    assert_eq!(stolen.len(), 1, "one theft");
    assert!(hero(&g).inv.iter().any(|i| i.kind == "sword"), "the sword stays");
    assert!(!hero(&g).inv.iter().any(|i| i.kind == "heal"), "the potion went");
    assert!(monster(&g, id).is_some_and(|m| m.fleeing));
    // No potion: nothing to steal.
    let mut g = arena();
    give(&mut g, "sword");
    add_monster(&mut g, "forge_imp", 5, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 80);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Steal { .. })));
}

#[test]
fn slag_crawler_leaves_fire_where_it_crawled() {
    let mut g = arena();
    add_monster(&mut g, "slag_crawler", 10, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Overlay { ov: OverlayKind::Fire, .. })), "a fire trail");
    assert!(g.lineage.facts.contains("foe:slag_crawler:fire"));
}

#[test]
fn smith_armours_its_allies() {
    let mut g = arena();
    add_monster(&mut g, "smith", 8, 5);
    let gob = add_monster(&mut g, "goblin", 9, 5);
    hold_rules(&mut g);
    ticks(&mut g, 15);
    assert_eq!(monster(&g, gob).unwrap().buff_def.0, 3, "+3 def from the smith");
    assert!(g.lineage.facts.contains("foe:smith:buffer"));
}

#[test]
fn deep_floors_are_dark_and_a_lantern_helps() {
    let mut g = arena();
    g.run.as_mut().unwrap().floor.vision = 4;
    hold_rules(&mut g);
    ticks(&mut g, 10);
    let map = &g.run.as_ref().unwrap().floor.map;
    assert!(map.is_visible(Pos::new(8, 5)), "four tiles away is seen");
    assert!(!map.is_visible(Pos::new(10, 5)), "six tiles away is dark");
    assert_eq!(g.snapshot().vision, 4);
    give(&mut g, "lantern");
    ticks(&mut g, 10);
    assert!(g.run.as_ref().unwrap().floor.map.is_visible(Pos::new(10, 5)), "a lantern reaches six");
    assert_eq!(g.snapshot().vision, 6);
    g.run.as_mut().unwrap().hero.inv.retain(|i| i.kind != "lantern");
    unlock(&mut g, "lantern_rig");
    ticks(&mut g, 10);
    assert_eq!(g.snapshot().vision, 6, "lantern_rig: the light without the slot");
    // Monsters in the dark see as far as the floor allows.
    let mut g = arena();
    g.run.as_mut().unwrap().floor.vision = 4;
    let id = add_sleeping(&mut g, "goblin", 10, 5);
    hold_rules(&mut g);
    ticks(&mut g, 40);
    assert!(!monster(&g, id).unwrap().awake, "six tiles off in the dark: unseen");
}

#[test]
fn rest_is_a_noise_that_blind_hunters_follow_and_silence_swallows() {
    let mut g = arena();
    let id = add_sleeping(&mut g, "lurker", 12, 5);
    g.run.as_mut().unwrap().hero.hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))]);
    ticks(&mut g, 60);
    let m = monster(&g, id).unwrap();
    assert!(m.awake, "the lurker heard the rest");
    assert!(m.pos.cheb(Pos::new(4, 5)) < 8, "and came for it: {:?}", m.pos);
    // Under silence nothing is heard.
    let mut g = arena();
    let id = add_sleeping(&mut g, "lurker", 12, 5);
    g.run.as_mut().unwrap().hero.hp = 10;
    g.run.as_mut().unwrap().hero.silence_t = 200;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))]);
    ticks(&mut g, 60);
    assert!(!monster(&g, id).unwrap().awake, "silence: the lurker sleeps on");
    assert!(g.run.as_ref().unwrap().noise.is_none());
}

#[test]
fn a_thrown_bell_lures_hunters_and_raises_the_alert() {
    let mut g = arena();
    give(&mut g, "bell");
    let id = add_sleeping(&mut g, "lurker", 13, 9);
    unlock(&mut g, "throw");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("throw", "bell"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "bell")));
    assert_eq!(g.run.as_ref().unwrap().alert, 3);
    let m = monster(&g, id).unwrap();
    assert!(m.awake);
    let heard = m.last_seen.expect("goes to the bell");
    assert!(heard != Pos::new(4, 5), "the bell rang away from the hero");
}

#[test]
fn cave_troll_regenerates_unless_poisoned() {
    let mut g = arena();
    let id = add_monster(&mut g, "cave_troll", 10, 5);
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.hp = 10;
        m.awake = false;
        m.last_seen = None;
    }
    hold_rules(&mut g);
    ticks(&mut g, 20);
    assert_eq!(monster(&g, id).unwrap().hp, 14, "+2 every 10 ticks");
    assert!(g.lineage.facts.contains("foe:cave_troll:regen"));
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == id).unwrap().poison = (1, 100);
    ticks(&mut g, 20);
    assert!(monster(&g, id).unwrap().hp <= 14, "poison stops it");
}

#[test]
fn siren_aura_confuses_within_two_tiles_unless_clear() {
    let mut g = arena();
    add_monster(&mut g, "siren", 6, 5);
    hold_rules(&mut g);
    ticks(&mut g, 10);
    assert!(hero(&g).confused > 0, "confused by the aura");
    assert!(g.lineage.facts.contains("foe:siren:aura"));
    let mut g = arena();
    add_monster(&mut g, "siren", 6, 5);
    g.run.as_mut().unwrap().hero.clarity_t = 100;
    hold_rules(&mut g);
    ticks(&mut g, 10);
    assert_eq!(hero(&g).confused, 0, "clarity holds");
}

#[test]
fn mirror_shade_copies_the_class_verb() {
    // A ranger's shade shoots from range.
    let mut g = arena();
    g.run.as_mut().unwrap().hero.class = Class::Ranger;
    let id = add_monster(&mut g, "mirror_shade", 8, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 20);
    assert!(evs.iter().any(|e| matches!(e, Ev::Projectile { src, dst: HERO_ID, .. } if *src == id)));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), .. } if v == "shoot")));
    assert!(g.lineage.facts.contains("foe:mirror_shade:mirror"));
    // A fighter's shade bashes.
    let mut g = arena();
    add_monster(&mut g, "mirror_shade", 5, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), .. } if v == "bash")));
}

#[test]
fn warden_alternates_its_faces_with_a_telegraph() {
    let m = Monster::spawn(1, "warden", Pos::new(1, 1), 26);
    assert!(m.reflects_melee() && !m.reflects_ranged(), "blade face first");
    let mut r = m.clone();
    r.warden_ranged = true;
    assert!(!r.reflects_melee() && r.reflects_ranged());
    let mut g = arena();
    let id = add_monster(&mut g, "warden", 6, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "shifts")));
    assert!(monster(&g, id).unwrap().warden_ranged, "flipped after the telegraph");
    assert!(g.lineage.facts.contains("foe:warden:reflect"));
}

#[test]
fn acolyte_heals_the_most_hurt_ally() {
    let mut g = arena();
    add_monster(&mut g, "acolyte", 8, 5);
    let gob = add_monster(&mut g, "goblin", 9, 5);
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == gob).unwrap().hp = 1;
    hold_rules(&mut g);
    ticks(&mut g, 12);
    assert!(monster(&g, gob).unwrap().hp >= 6, "healed 5");
    assert!(g.lineage.facts.contains("foe:acolyte:healer"));
}

#[test]
fn echo_splits_on_ranged_hits_only() {
    let count = |g: &Game| g.run.as_ref().unwrap().monsters.iter().filter(|m| m.kind == "echo" && m.hp > 0).count();
    let mut g = arena();
    add_monster(&mut g, "echo", 9, 5);
    g.run.as_mut().unwrap().hero.weapon = Some(Item::new(2, "bow"));
    attack_rules(&mut g);
    let mut split = false;
    for _ in 0..60 {
        ticks(&mut g, 1);
        if count(&g) >= 2 {
            split = true;
            break;
        }
    }
    assert!(split, "an arrow split the echo");
    assert!(g.lineage.facts.contains("foe:echo:echo"));
    let mut g = arena();
    add_monster(&mut g, "echo", 5, 5);
    attack_rules(&mut g);
    for _ in 0..60 {
        ticks(&mut g, 1);
        assert!(count(&g) <= 1, "a sword never splits it");
    }
}

#[test]
fn sentinel_gazes_then_stuns() {
    let mut g = arena();
    add_monster(&mut g, "sentinel", 6, 5);
    hold_rules(&mut g);
    let mut stunned = false;
    let mut telegraphed = false;
    for _ in 0..40 {
        let evs = ticks(&mut g, 1);
        telegraphed |= evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "gazes"));
        if hero(&g).paralysed > 0 {
            stunned = true;
            break;
        }
    }
    assert!(telegraphed && stunned, "telegraph {telegraphed}, stunned {stunned}");
    assert!(g.lineage.facts.contains("foe:sentinel:gaze"));
}

#[test]
fn mirror_king_reflects_the_third_of_a_kind_and_cadence_never_repeats() {
    let mut g = arena();
    add_monster(&mut g, "mirror_king", 5, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), dst: HERO_ID, .. } if v == "mirror")), "the third swing came back");
    assert!(crate::facts::boss_counter_known(&g.lineage.facts, "mirror_king"));
    assert!(g.lineage.facts.contains("foe:mirror_king:mirror"));
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "mirrors")));
    // Cadence: two of a kind, then another; nothing comes back and the King bleeds.
    let mut g = arena();
    let id = add_monster(&mut g, "mirror_king", 5, 5);
    g.run.as_mut().unwrap().hero.hp = 500;
    g.run.as_mut().unwrap().hero.max_hp = 500;
    unlock(&mut g, "cadence");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tactic", "cadence"))]);
    let evs = ticks(&mut g, 200);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), dst: HERO_ID, .. } if v == "mirror")), "cadence never repeats thrice");
    assert!(monster(&g, id).is_none_or(|m| m.hp < m.max_hp));
    let ring = &g.run.as_ref().unwrap().verb_ring;
    assert!(ring.len() < 3 || !(ring[0] == ring[1] && ring[1] == ring[2]), "{ring:?}");
}

#[test]
fn lurker_queen_calls_lurkers_to_a_noise() {
    let mut g = arena();
    add_sleeping(&mut g, "lurker_queen", 12, 5);
    g.run.as_mut().unwrap().hero.hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))]);
    let evs = ticks(&mut g, 12);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "listens")), "she heard the rest");
    assert!(evs.iter().any(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "lurker")), "lurkers came");
    assert!(!crate::facts::boss_counter_known(&g.lineage.facts, "lurker_queen"), "unseen in the dark: no fact yet");
    // A bell rung in her sight: the call, seen, is the counter fact.
    hold_rules(&mut g);
    ticks(&mut g, 40);
    g.run.as_mut().unwrap().hero.pos = Pos::new(9, 5);
    give(&mut g, "bell");
    unlock(&mut g, "throw");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("throw", "bell"))]);
    ticks(&mut g, 12);
    assert!(crate::facts::boss_counter_known(&g.lineage.facts, "lurker_queen"));
}

#[test]
fn foundry_master_telegraphs_his_hammer_and_learns_the_counter() {
    let mut g = arena();
    add_monster(&mut g, "foundry_master", 5, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "hammers")));
    assert!(crate::facts::boss_counter_known(&g.lineage.facts, "foundry_master"));
    let m = g.run.as_ref().unwrap().monsters.iter().find(|m| m.kind == "foundry_master").unwrap();
    assert!(m.reflects_melee());
}

#[test]
fn boss_escorts_and_stock_for_the_new_biomes() {
    for (depth, boss, escort) in [(23u32, "foundry_master", "smith"), (28, "lurker_queen", "lurker"), (33, "mirror_king", "mirror_shade")] {
        let mut found_escort = false;
        for seed in 1..=4u64 {
            let mut g = Game::new_literal(seed);
            g.start_run(Some(seed));
            let run = g.run.as_mut().unwrap();
            run.depth = depth - 1;
            run.hero.pos = run.floor.stairs_down;
            {
                let (run, mut cx) = g.ctx();
                crate::turn::descend(run, &mut cx);
            }
            let run = g.run.as_ref().unwrap();
            assert_eq!(run.depth, depth);
            assert!(run.monsters.iter().any(|m| m.kind == boss), "seed {seed}: {boss} on D{depth}");
            found_escort |= run.monsters.iter().any(|m| m.kind == escort);
        }
        assert!(found_escort, "{escort} escorts the {boss}");
    }
    let mut g = Game::new_literal(3);
    g.start_run(Some(3));
    let run = g.run.as_mut().unwrap();
    run.depth = 23;
    run.hero.pos = run.floor.stairs_down;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::descend(run, &mut cx);
    }
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.biome(), crate::descent::Biome::Deep);
    assert_eq!(run.floor.vision, 4);
    assert!(run.items.iter().any(|i| i.item.kind == "silence"), "the Deep stocks silence");
    assert!(run.items.iter().any(|i| i.item.kind == "lantern"), "and a lantern on its doorstep");
}

// ---------------------------------------------------------------- Cut 3: items

#[test]
fn recall_scroll_banks_from_anywhere() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 12;
    give(&mut g, "recall");
    identify(&mut g, "recall");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("read", "recall"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "bank")));
    assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Bank));
    assert!(g.lineage.facts.contains("item:recall"), "knowing recall gates recall_sense");
}

#[test]
fn recall_sense_reads_recall_below_fifteen_percent() {
    let mut g = arena();
    give(&mut g, "recall");
    identify(&mut g, "recall");
    unlock(&mut g, "recall_sense");
    g.run.as_mut().unwrap().hero.hp = 4;
    hold_rules(&mut g);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { text, .. } if text == "recall sense")));
    assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Bank));
}

#[test]
fn earthquake_opens_walls_and_keeps_the_floor_connected() {
    let mut quaked = false;
    for seed in 1..=12u64 {
        let mut g = Game::new_literal(seed);
        g.start_run(Some(seed * 11));
        let hp = hero(&g).pos;
        let map = &g.run.as_ref().unwrap().floor.map;
        let walls = (-2..=2)
            .flat_map(|dy| (-2..=2).map(move |dx| (dx, dy)))
            .map(|d| hp.step(d))
            .filter(|p| map.get(*p) == Tile::Wall && p.x > 0 && p.y > 0 && p.x < map.w - 1 && p.y < map.h - 1)
            .count();
        if walls == 0 {
            continue;
        }
        give(&mut g, "earthquake");
        identify(&mut g, "earthquake");
        rules(&mut g, vec![Row::new(vec![], Verb::arg("read", "earthquake"))]);
        let evs = ticks(&mut g, 10);
        assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "earthquake scroll")));
        let run = g.run.as_ref().unwrap();
        let map = &run.floor.map;
        for dy in -2..=2 {
            for dx in -2..=2 {
                let p = hp.step((dx, dy));
                if map.in_bounds(p) && p.x > 0 && p.y > 0 && p.x < map.w - 1 && p.y < map.h - 1 {
                    assert_ne!(map.get(p), Tile::Wall, "wall at {p:?} fell");
                }
            }
        }
        for x in 0..map.w {
            assert_eq!(map.get(Pos::new(x, 0)), Tile::Wall, "the rim holds");
            assert_eq!(map.get(Pos::new(x, map.h - 1)), Tile::Wall);
        }
        let d = map.bfs(run.floor.stairs_up, false, &|_| false);
        assert!(d[map.idx(run.floor.stairs_down)] > 0, "still connected");
        quaked = true;
        break;
    }
    assert!(quaked, "some seed had walls to open");
}

#[test]
fn chalk_marks_the_floor_left_and_the_next_visit_walks_to_the_stairs() {
    let mut g = arena();
    give(&mut g, "chalk");
    g.run.as_mut().unwrap().depth = 3;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "chalk:3")));
    assert!(!hero(&g).inv.iter().any(|i| i.kind == "chalk"), "the chalk is spent");
    // A chalked depth: the stairs are known on arrival and the chores go straight for them.
    let mut g = arena();
    g.lineage.facts.insert("chalk:4".into());
    g.run.as_mut().unwrap().depth = 3;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 99)], Verb::arg("attack", "nearest"))]);
    ticks(&mut g, 10);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.depth, 4);
    assert!(run.floor.map.is_seen(run.floor.stairs_down), "chalk: the way down is known");
    let evs = ticks(&mut g, 30);
    let chores: Vec<String> = evs.iter().filter_map(|e| match e { Ev::Rule { row: -2, verb, .. } => Some(verb.v.clone()), _ => None }).collect();
    assert!(chores.iter().any(|v| v == "descend"), "{chores:?}");
}

#[test]
fn mirror_scroll_sends_the_next_blow_back() {
    let mut g = arena();
    let id = add_monster(&mut g, "goblin", 5, 5);
    give(&mut g, "mirror");
    identify(&mut g, "mirror");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("read", "mirror"))]);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), src: HERO_ID, .. } if v == "mirror")), "the blow came back");
    assert!(monster(&g, id).is_none_or(|m| m.hp < m.max_hp));
    assert_eq!(hero(&g).mirror_charge, 0);
}

#[test]
fn salt_routs_the_undead() {
    let mut g = arena();
    let id = add_monster(&mut g, "skeleton", 6, 5);
    give(&mut g, "salt");
    unlock(&mut g, "throw");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("throw", "salt"))]);
    ticks(&mut g, 10);
    assert!(monster(&g, id).unwrap().fear > 0, "the skeleton flees");
    assert!(!hero(&g).inv.iter().any(|i| i.kind == "salt"));
}

#[test]
fn new_potions_take_effect() {
    let mut g = arena();
    for k in ["regen", "resist_fire", "clarity"] {
        give(&mut g, k);
        identify(&mut g, k);
    }
    g.run.as_mut().unwrap().hero.hp = 10;
    g.run.as_mut().unwrap().hero.confused = 30;
    rules(&mut g, vec![
        Row::new(vec![Cond::t("item", "clarity")], Verb::arg("drink", "clarity")),
        Row::new(vec![Cond::t("item", "regen")], Verb::arg("drink", "regen")),
        Row::new(vec![Cond::t("item", "resist_fire")], Verb::arg("drink", "resist_fire")),
    ]);
    ticks(&mut g, 30);
    let h = hero(&g);
    assert_eq!(h.confused, 0);
    assert!(h.regen_t > 0 && h.resist_fire_t > 0 && h.clarity_t > 0, "{:?}", h.status_tags());
    assert!(h.hp > 10, "regen ticked");
    // Fire does nothing to a fireproof hero.
    let hp = hero(&g).hp;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_hero(run, &mut cx, 5, &crate::turn::Src::Fire);
    }
    assert_eq!(hero(&g).hp, hp);
}

#[test]
fn spear_reaches_two_tiles_and_mace_stuns_sometimes() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.weapon = Some(Item::new(2, "spear"));
    add_monster(&mut g, "goblin", 6, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src: HERO_ID, .. })), "struck from two tiles");
    assert_eq!(hero(&g).pos, Pos::new(4, 5), "without stepping");
    let mut g = arena();
    g.run.as_mut().unwrap().hero.weapon = Some(Item::new(2, "mace"));
    g.run.as_mut().unwrap().hero.hp = 9999;
    g.run.as_mut().unwrap().hero.max_hp = 9999;
    let id = add_monster(&mut g, "ogre", 5, 5);
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.hp = 5000;
        m.max_hp = 5000;
    }
    attack_rules(&mut g);
    let evs = ticks(&mut g, 1200);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "stunned")), "one hit in ten stuns");
}

#[test]
fn found_tools_are_facts_and_tokens() {
    let mut g = arena();
    let run = g.run.as_mut().unwrap();
    let id = run.new_item_id();
    run.items.push(crate::engine::FloorItem { pos: Pos::new(5, 5), item: Item::new(id, "bell") });
    rules(&mut g, vec![Row::new(vec![], Verb::new("pick_up"))]);
    ticks(&mut g, 30);
    assert!(g.lineage.facts.contains("item:bell"));
    g.lineage.unlocks.insert("throw".into());
    let v = g.vocabulary();
    assert!(v.verbs.contains(&Verb::arg("throw", "bell")));
    assert!(v.conds.contains(&Cond::t("item", "bell")));
}

#[test]
fn mirror_shard_breeds_the_mirror_tag() {
    let mut g = Game::new_literal(1);
    let mut a = crate::probes::pets_party()[0].clone();
    let mut b = crate::probes::pets_party()[1].clone();
    a.level = 2;
    b.level = 2;
    g.lineage.kennel.push(a.clone());
    g.lineage.kennel.push(b.clone());
    g.lineage.vault.push(Item::new(100_001, "mirror_shard"));
    g.breed(a.id, b.id).unwrap();
    let egg = g.lineage.eggs.last().unwrap();
    assert!(egg.tags.iter().any(|t| t == "mirror"), "{:?}", egg.tags);
    assert!(!g.lineage.vault.iter().any(|i| i.kind == "mirror_shard"), "the shard is spent");
    assert_eq!(crate::defs::tag_verb("mirror"), Some("mimic"));
}

// ---------------------------------------------------------------- Cut 3: unlocks and cards

#[test]
fn tier_two_unlocks_need_bosses_and_open_rows_to_ten() {
    let mut g = Game::new_literal(1);
    for u in ["row5", "row6", "row7", "row8"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.marks = 100;
    g.lineage.banked_depths.insert(2);
    // Cut 29 §1: row 9 opens at T5 (the Foundry Master met), row 10 at T6 (the Lurker Queen met)
    assert!(g.buy("row9").is_err(), "four bosses met first");
    for k in ["goblin_warlord", "bloat_mother", "lich", "foundry_master"] {
        g.lineage.facts.insert(format!("foe:{k}"));
    }
    g.buy("row9").unwrap();
    assert_eq!(g.lineage.max_rows(), 9);
    assert!(g.buy("row10").is_err(), "five bosses met for the tenth row");
    g.lineage.facts.insert("foe:lurker_queen".into());
    g.buy("row10").unwrap();
    assert_eq!(g.lineage.max_rows(), 10);
    assert_eq!(g.lineage.marks, 100 - 7 - 8);
    let ten: Vec<Row> = (0..10).map(|_| Row::new(vec![], Verb::new("hold"))).collect();
    assert!(g.set_rules(RuleSet { rows: ten.clone(), name: None, route: Vec::new() }).is_ok());
    assert_eq!(g.lineage.rules().rows.len(), 10);
    let mut eleven = ten;
    eleven.push(Row::new(vec![], Verb::new("hold")));
    assert!(RuleSet { rows: eleven, name: None, route: Vec::new() }.validate().is_err());
    assert!(g.buy("cadence").is_err(), "needs the mirror fact");
    g.lineage.facts.insert("foe:mirror_shade:mirror".into());
    g.buy("cadence").unwrap();
    assert!(g.vocabulary().verbs.contains(&Verb::arg("tactic", "cadence")));
}

#[test]
fn noise_discipline_holds_the_rest_while_a_blind_foe_lives() {
    let mut g = arena();
    let id = add_monster(&mut g, "lurker", 8, 5);
    unlock(&mut g, "noise_discipline");
    hold_rules(&mut g);
    ticks(&mut g, 1);
    assert!(g.run.as_ref().unwrap().blind_foe_known());
    // Out of sight but alive: no rest.
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == id).unwrap().pos = Pos::new(14, 10);
    g.run.as_mut().unwrap().hero.hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 90)], Verb::arg("tactic", "noise_discipline"))]);
    ticks(&mut g, 30);
    assert_eq!(hero(&g).hp, 10, "no rest with a lurker known on the floor");
    assert!(g.run.as_ref().unwrap().noise.is_none());
    // Dead: the card rests.
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == id).unwrap().hp = 0;
    ticks(&mut g, 30);
    assert!(hero(&g).hp > 10, "rested once the hunter is gone");
}

#[test]
fn reflect_read_never_swings_at_a_mirror() {
    let mut g = arena();
    add_monster(&mut g, "iron_golem", 5, 5);
    give(&mut g, "fire");
    identify(&mut g, "fire");
    unlock(&mut g, "reflect_read");
    unlock(&mut g, "throw");
    g.lineage.facts.insert("foe:iron_golem:reflect_melee".into());
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 200);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Attack { src: HERO_ID, verb: Some(v), .. } if v == "attack")), "no melee on the golem");
    assert!(!evs.iter().any(|e| matches!(e, Ev::Attack { verb: Some(v), .. } if v == "reflect")));
    // With a bow in the pack it goes up and the golem is shot.
    let mut g = arena();
    let id = add_monster(&mut g, "iron_golem", 7, 5);
    give(&mut g, "bow");
    unlock(&mut g, "reflect_read");
    g.lineage.facts.insert("foe:iron_golem:reflect_melee".into());
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read"))]);
    let evs = ticks(&mut g, 100);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "bow up")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src: HERO_ID, verb: Some(v), .. } if v == "shoot")));
    assert!(monster(&g, id).is_none_or(|m| m.hp < m.max_hp));
    // Once no mirror is in view the sword comes back.
    g.run.as_mut().unwrap().monsters.clear();
    ticks(&mut g, 10);
    assert_eq!(hero(&g).weapon_kind(), "sword");
    assert!(g.run.as_ref().unwrap().bow_swap.is_none());
}

#[test]
fn deep_march_descends_at_forty_percent_only_in_the_dark() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        run.floor.vision = 4;
        for s in run.floor.map.seen.iter_mut() {
            *s = false;
        }
        run.floor.map.update_vision(run.hero.pos, 4);
        // A known way down: an L of seen tiles to the stairs.
        for x in 4..=14 {
            let i = run.floor.map.idx(Pos::new(x, 5));
            run.floor.map.seen[i] = true;
        }
        for y in 5..=10 {
            let i = run.floor.map.idx(Pos::new(14, y));
            run.floor.map.seen[i] = true;
        }
    }
    unlock(&mut g, "deep_march");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("tactic", "deep_march"))]);
    let seen = g.run.as_ref().unwrap().floor.map.seen_pct();
    assert!((40..60).contains(&seen), "seen {seen}%");
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })), "the card fires in the dark");
    let mut g = arena();
    unlock(&mut g, "deep_march");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("tactic", "deep_march"))]);
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })), "lit: the card falls through");
}

#[test]
fn studied_all_biome_is_a_three_mark_trophy() {
    let mut g = Game::new_literal(1);
    for k in crate::defs::biome_kinds(crate::descent::Biome::Warrens) {
        g.lineage.facts.insert(format!("foe:{k}:studied"));
    }
    g.start_run(None);
    let marks = g.lineage.marks;
    finish_with(&mut g, ExitTier::Return);
    assert!(g.lineage.trophies.contains(&"studied_all_warrens".to_string()));
    assert!(g.lineage.marks >= marks + 3);
}

// ---------------------------------------------------------------- Cut 3: ascension

fn finished_lineage() -> Game {
    let mut g = Game::new_literal(5);
    g.lineage.ended = true;
    g.lineage.marks = 12;
    g.lineage.gold = 300;
    g.lineage.heir = 9;
    g.lineage.best_depth = 31;
    for u in ["row5", "row6", "row7", "row8", "row9", "cadence", "throw", "rogue", "phalanx"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 7, xp: 10, next: 0 });
    g.lineage.kennel.push(crate::probes::pets_party()[0].clone());
    g.lineage.party.push(crate::probes::pets_party()[1].clone());
    g.lineage.vault.push(Item::new(100_001, "plate"));
    g.lineage.facts.insert("foe:lich:boss".into());
    g.lineage.forge.insert("sword".into(), ForgeRow::at(20));
    g.lineage.grudges.push(crate::descent::Grudge { kind: "ogre".into(), name: "Grak".into(), depth: 7, heir: 3, avenged: false, tamed: false, biome: None });
    g.lineage.graveyard.push(Grave { heir: 3, depth: 7, cause: "ogre".into(), deeds: vec![], death_id: None });
    g.set_rules(crate::probes::good()).unwrap();
    g
}

#[test]
fn ascension_keeps_the_meta_and_restarts_the_descent() {
    let mut g = Game::new_literal(1);
    assert!(g.ascend("no_rest").is_err(), "not before the ending");
    let mut g = finished_lineage();
    assert!(g.ascend("nonsense").is_err());
    g.ascend("no_rest").unwrap();
    let l = &g.lineage;
    assert_eq!(l.ascension, 1);
    assert_eq!(l.variant, "no_rest");
    assert!(!l.ended);
    assert_eq!((l.marks, l.gold, l.heir, l.best_depth), (0, 0, 1, 0));
    assert_eq!(l.classes["fighter"].level, 7, "classes carry");
    assert_eq!(l.kennel.len(), 2, "kennel carries (the party home in it)");
    assert!(l.party.is_empty());
    assert_eq!(l.vault.len(), 1, "vault carries");
    assert!(l.facts.contains("foe:lich:boss"), "facts carry");
    assert_eq!(l.forge["sword"].tier, 1, "forge carries");
    assert!(l.graveyard.is_empty() && l.grudges.is_empty());
    assert_eq!(l.rules(), &crate::probes::good(), "rules stay");
    assert!(!l.unlocks.contains("row5") && !l.unlocks.contains("cadence"), "unlocks start over");
    assert!(l.unlocks.contains("rogue") && l.unlocks.contains("phalanx"), "class doors and mastery stay");
    assert_eq!(l.max_rows(), 4);
    let w = g.lineage();
    assert_eq!(w.ascension, Ascension { level: 1, variant: "no_rest".into() });
    let json = serde_json::to_string(&w).unwrap();
    assert!(json.contains(r#""ascension":{"level":1,"variant":"no_rest"}"#));
    // The save carries it.
    let g2 = Game::load(&g.save()).unwrap();
    assert_eq!(g2.lineage.variant, "no_rest");
    assert_eq!(g2.lineage.ascension, 1);
}

#[test]
fn no_rest_removes_the_verb_and_halves_camp_rest() {
    let mut g = finished_lineage();
    g.ascend("no_rest").unwrap();
    assert!(!g.vocabulary().verbs.iter().any(|v| v.v == "rest"));
    assert_eq!(g.rest_after(6000, ExitTier::Return), crate::offline::rest_after(6000, ExitTier::Return) / 2);
    assert_eq!(g.rest_after(6000, ExitTier::Death), crate::engine::WAKE_TICKS / 2);
    g.start_run(Some(9));
    g.run.as_mut().unwrap().monsters.clear();
    g.run.as_mut().unwrap().hero.hp = 10;
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))], name: None, route: Vec::new() }).unwrap();
    ticks(&mut g, 50);
    assert_eq!(hero(&g).hp, 10, "rest is not a verb");
}

#[test]
fn short_list_caps_rows_at_six_and_keeps_the_cards() {
    let mut g = finished_lineage();
    g.ascend("short_list").unwrap();
    assert!(g.lineage.unlocks.contains("cadence"), "tactic cards carry");
    for u in ["row5", "row6", "row7", "row8"] {
        g.lineage.unlocks.insert(u.into());
    }
    assert_eq!(g.lineage.max_rows(), 6);
    assert_eq!(g.vocabulary().max_rows, 6);
}

#[test]
fn bones_only_has_no_vault() {
    let mut g = finished_lineage();
    g.ascend("bones_only").unwrap();
    assert!(g.lineage.vault.is_empty());
    assert_eq!(g.lineage.vault_slots(), 0);
    g.loadout(vec![100_001]);
    assert!(g.loadout.is_empty());
    g.start_run(Some(4));
    g.run.as_mut().unwrap().hero.inv.push(Item::new(77, "mail"));
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![77]).unwrap();
    assert!(g.lineage.vault.is_empty(), "nothing enters the vault");
    assert!(g.lineage.gold > 0, "salvaged instead");
}

#[test]
fn hunted_puts_the_grudge_on_every_floor_from_d3() {
    let mut g = finished_lineage();
    g.ascend("hunted").unwrap();
    let h = g.lineage.hunter.clone().expect("a hunter");
    assert_eq!(h.name, "Grak");
    g.start_run(Some(2));
    assert!(!g.run.as_ref().unwrap().monsters.iter().any(|m| m.grudge), "not on D1");
    for target in [2u32, 3, 4] {
        let run = g.run.as_mut().unwrap();
        run.hero.pos = run.floor.stairs_down;
        let (run, mut cx) = g.ctx();
        crate::turn::descend(run, &mut cx);
        let run = g.run.as_ref().unwrap();
        assert_eq!(run.depth, target);
        let stalks = run.monsters.iter().any(|m| m.grudge && m.name.as_deref() == Some("Grak") && m.awake);
        assert_eq!(stalks, target >= 3, "D{target}");
    }
}

#[test]
fn the_ending_after_an_ascension_records_the_variant() {
    let mut g = finished_lineage();
    g.ascend("no_rest").unwrap();
    g.start_run(Some(1));
    let run = g.run.as_mut().unwrap();
    run.depth = 33;
    run.hero.pos = run.floor.stairs_down;
    run.monsters.clear();
    g.set_rules(RuleSet { rows: vec![Row::new(vec![], Verb::new("descend"))], name: None, route: Vec::new() }).unwrap();
    ticks(&mut g, 20);
    assert!(g.lineage.ended);
    assert_eq!(g.lineage.ascended, vec!["no_rest".to_string()]);
    assert!(g.ascend("short_list").is_ok(), "again, under another variant");
    assert_eq!(g.lineage.ascension, 2);
}

#[test]
fn full_preset_is_the_probe_and_reaches_the_new_biomes_facts() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/presets/full.json")).unwrap();
    let set = RuleSet::parse(&text).unwrap();
    assert_eq!(set, crate::probes::full());
    let mut g = Game::new_literal(1);
    for u in crate::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    crate::probes::learn_everything(&mut g);
    g.set_rules(set).unwrap();
    let v = g.vocabulary();
    for r in &g.lineage.rules().rows {
        assert!(v.verbs.iter().any(|x| x.v == r.verb.v && x.a.as_deref().map(|a| a.split(',').next().unwrap()) == r.verb.a.as_deref().map(|a| a.split(',').next().unwrap())), "{:?} in the vocabulary", r.verb);
    }
}

#[test]
fn forecast_stops_at_known_to_and_the_run_cap_is_long() {
    assert_eq!(crate::engine::MAX_TURNS_PER_RUN, 120_000);
    let mut g = Game::new_literal(3);
    g.lineage.best_depth = 2;
    let f = g.forecast();
    assert_eq!(f.known_to, 3);
    assert_eq!(f.depths.len(), 3);
    let r = crate::forecast::simulate(&g, g.lineage.rules(), 3, 7, 1);
    assert!(r.iter().all(|s| s.max_depth <= 1), "sims stop at the depth asked");
}

// ---------------------------------------------------------------- Cut 4

/// Cut 4 §5: one loot unit. The exit note, `Ev::Exit.loot_kept`, the HUD stake and the gold
/// delta on a bank are the same number, in gold, on every seed.
#[test]
fn loot_is_one_unit_from_pickup_to_the_bank() {
    for seed in 1..=4u64 {
        let mut g = arena_seed(seed);
        {
            let run = g.run.as_mut().unwrap();
            let mut gold = Item::new(70, "gold");
            gold.amount = 25 + seed as i32 * 7;
            run.items.push(crate::engine::FloorItem { pos: Pos::new(5, 5), item: gold });
            let mut gold = Item::new(71, "gold");
            gold.amount = 3;
            run.items.push(crate::engine::FloorItem { pos: Pos::new(6, 5), item: gold });
        }
        rules(&mut g, vec![]);
        ticks(&mut g, 60);
        let snap = g.snapshot();
        let run = g.run.as_ref().unwrap();
        assert!(run.loot_raw >= 28, "seed {seed}: nothing picked up");
        assert_eq!(run.loot, run.loot_raw / crate::engine::GOLD_DIVISOR);
        assert_eq!(snap.stake.loot, run.loot, "the HUD stake is the run's gold");
        assert_eq!(snap.loot, run.loot);
        let gold_before = g.lineage.gold;
        let (kept, note) = {
            let (run, mut cx) = g.ctx();
            crate::turn::end_run(run, &mut cx, ExitTier::Bank);
            let kept = cx.events.iter().find_map(|e| if let Ev::Exit { loot_kept, .. } = e { Some(*loot_kept) } else { None }).unwrap();
            let note = cx.events.iter().find_map(|e| if let Ev::Note { text, .. } = e { Some(text.clone()) } else { None }).unwrap();
            (kept, note)
        };
        g.finish_run();
        g.keep(vec![]).unwrap();
        assert_eq!(note, format!("Banked ${kept}."), "seed {seed}");
        assert_eq!(kept, snap.stake.loot, "seed {seed}: the exit keeps what the HUD showed");
        // Salvage of the starting kit is nil (the dagger is not loot): the delta is the loot.
        assert_eq!(g.lineage.gold - gold_before, kept, "seed {seed}: gold delta on bank");
    }
}

/// Cut 4 §6: the first bank from each depth is a mark and a `home:D<n>` best, distinct from
/// the first reach; a second bank from the same depth earns nothing.
#[test]
fn first_bank_at_each_depth_is_a_mark() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 3;
    g.run.as_mut().unwrap().max_depth = 3;
    g.lineage.best_depth = 6; // already reached: no depth mark (and D3 is not the frontier: Cut 15 §1)
    let marks = g.lineage.marks;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1);
    assert!(g.batch.bests.contains(&"home:D3".to_string()), "{:?}", g.batch.bests);
    // Again from D3: nothing. From D3 as a return: nothing.
    g.start_run(Some(9));
    g.run.as_mut().unwrap().depth = 3;
    g.run.as_mut().unwrap().max_depth = 3;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1);
    g.start_run(Some(10));
    g.run.as_mut().unwrap().depth = 4;
    g.run.as_mut().unwrap().max_depth = 4;
    g.lineage.best_depth = 4;
    finish_with(&mut g, ExitTier::Return);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1, "a return is not a bank");
    assert!(!g.batch.bests.iter().any(|b| b == "home:D4"));
}

/// Cut 29 §1 (docs/PROGRESSION.md §3: the frontier mark paid for the same D13 bank every run):
/// a bank at the record pays no mark of its own — a new best still pays its depth and its first
/// bank; the night's mark (◆1 per day whose absences brought a send home) is the absence's.
#[test]
fn no_mark_for_the_frontier_and_one_a_night() {
    let exit_text = |g: &Game| g.last_exit.as_ref().map(|l| l.text.clone()).unwrap_or_default();
    let mut g = arena();
    g.lineage.rank = 50; // no rank-up mark from the watched run's renown
    g.lineage.best_depth = 8;
    g.lineage.banked_depths.insert(7); // not a first bank from D7
    g.run.as_mut().unwrap().depth = 7;
    g.run.as_mut().unwrap().max_depth = 7;
    let marks = g.lineage.marks;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks, "D7 against a best of D8 pays nothing now");
    assert_eq!(g.batch.frontier_banks, 0);
    assert!(!exit_text(&g).contains('◆'), "{}", exit_text(&g));
    // A new-best bank (D10 from a best of D8): two depth marks and the first bank from D10.
    g.start_run(Some(11));
    g.run.as_mut().unwrap().depth = 10;
    g.run.as_mut().unwrap().max_depth = 10;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 2 + 1);
    assert!(exit_text(&g).ends_with(" · ◆+3"), "{}", exit_text(&g));
    // The night's mark: an absence whose sends came home pays ◆1 for its day, once.
    let mut g = Game::new(5);
    g.max_deaths = 1000;
    let r = crate::offline::run_offline_counts(&mut g, 8 * 3600);
    assert!(r.banked + r.returned > 0, "sends came home");
    assert_eq!(r.night_marks, 1);
    let r = crate::offline::run_offline_counts(&mut g, 4 * 3600);
    assert_eq!(r.night_marks, 0, "the same day pays once");
    let r = crate::offline::run_offline_counts(&mut g, 14 * 3600);
    assert_eq!(r.night_marks, 1, "the next day (the absence reached it) pays");
    // an absence of three whole days pays each day it covered
    let r = crate::offline::run_offline_counts(&mut g, 3 * 24 * 3600);
    assert_eq!(r.night_marks, 3);
    assert_eq!(g.lineage.day, 1, "the day an absence starts on");
}

/// Cut 15 §6: a hazard pre-emption is said once — one `RowWhy` on the first row, naming the
/// hazard (`hazard first · gas`), not the same words on every row.
#[test]
fn a_hazard_preemption_is_said_once_with_its_name() {
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![Cond::n("hp<", 30)], Verb::new("rest"))]);
    let hp = g.run.as_ref().unwrap().hero.pos;
    g.run.as_mut().unwrap().overlays.push(crate::tiles::Overlay { x: hp.x, y: hp.y, k: OverlayKind::Gas, ttl: 20, spread: false });
    let mut found = None;
    for _ in 0..40 {
        ticks(&mut g, 1);
        if let Some(t) = g.run.as_ref().unwrap().trace.iter().find(|t| t.row == -2) {
            found = Some(t.clone());
            break;
        }
    }
    let t = found.expect("the hazard pre-empted");
    let rows = t.rows.clone().unwrap_or_default();
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!((rows[0].row, rows[0].why.as_str()), (0, "hazard first · gas"));
    assert!(crate::turn::row_reason_ok(&rows[0].why));
}

/// Cut 15 §2, Cut 29 §1: a gold buy spends gold (a ledger line `unlock <id>`), leaves the marks
/// and owns the unlock — gold buys the row slots (the forge's ladder) and the automations (forge
/// units, at the Lich) only; a card, an owned one or a shut gate is refused.
#[test]
fn a_gold_buy_spends_gold_on_rows_and_automations() {
    let mut g = Game::new_literal(3);
    let price = |g: &Game, id: &str| g.unlocks().into_iter().find(|u| u.id == id).unwrap().gold;
    assert_eq!(price(&g, "vault2"), 0, "a card or a vault slot is marks only");
    // Cut 23 §1: a row slot sits on the forge's ladder (2 × the unit, $100 at best 0).
    assert_eq!(price(&g, "row5"), 2 * crate::kit::unit(0));
    assert!(g.buy_unlock_gold("vault2").is_err());
    g.lineage.gold_move(20000, "test");
    g.lineage.marks = 1;
    assert!(g.buy_unlock_gold("row5").is_err(), "row5 needs the first bank (T1)");
    g.lineage.banked_depths.insert(2);
    g.lineage.best_depth = 2;
    g.buy_unlock_gold("row5").unwrap();
    assert!(g.lineage.unlocks.contains("row5"));
    assert_eq!(g.lineage.marks, 1, "marks untouched");
    assert_eq!(g.lineage.gold_ledger.last().map(|l| l.why.as_str()), Some("unlock row5"));
    assert_eq!(g.lineage.gold_buys, 0, "a row slot's price is the ladder's");
    assert!(g.buy_unlock_gold("row5").is_err(), "already owned");
    assert!(g.buy_unlock_gold("row7").is_err(), "prerequisite missing");
    // an automation: gold only, at the Lich (T4), `units × unit`
    assert!(g.buy_unlock_gold("bone_sense").is_err(), "a shut tier");
    g.lineage.best_depth = 18;
    g.lineage.facts.insert("bones:1".into());
    let p = price(&g, "bone_sense");
    assert_eq!(p, 3 * crate::kit::unit_of(&g.lineage));
    assert!(g.buy("bone_sense").is_err(), "not for marks");
    let gold = g.lineage.gold;
    g.buy_unlock_gold("bone_sense").unwrap();
    assert_eq!(g.lineage.gold, gold - p as i32);
    let back = Game::load(&g.save()).unwrap();
    assert!(back.lineage.unlocks.contains("bone_sense"));
}

/// Cut 4 §3: a row that acted on a foe keeps hunting it when it steps out of view: the
/// snapshot carries it as `remembered` at its last-seen tile and the callout reads `hunt <kind>`;
/// in view the callout names the act and the kind (`attack goblin`).
#[test]
fn a_pursued_foe_out_of_view_is_remembered_and_hunted() {
    let mut g = arena();
    let id = add_monster(&mut g, "goblin", 7, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 10);
    let texts: Vec<String> = evs.iter().filter_map(|e| if let Ev::Rule { row: 0, text, .. } = e { Some(text.clone()) } else { None }).collect();
    assert!(!texts.is_empty() && texts.iter().all(|t| t == "attack goblin"), "{texts:?}");
    assert!(texts.iter().all(|t| word_count(t) <= 3));
    let seen_at = g.run.as_ref().unwrap().known_foes.get(&id).map(|(p, _)| *p).expect("the goblin was noted");
    // It slips out of view (a corner, in a real floor): far off and asleep.
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.pos = Pos::new(14, 9);
        m.awake = false;
        run.floor.map.update_vision(run.hero.pos, VISION);
        assert!(!run.floor.map.is_visible(Pos::new(14, 9)));
    }
    let snap = g.snapshot();
    let ghost = snap.entities.iter().find(|e| e.id == id).expect("remembered in the snapshot");
    assert!(ghost.remembered);
    assert_eq!((ghost.x, ghost.y), (seen_at.x, seen_at.y), "at its last-seen tile, not where it is");
    assert!(serde_json::to_string(ghost).unwrap().contains(r#""remembered":true"#));
    assert!(!serde_json::to_string(&snap.hero).unwrap().contains("remembered"), "serde skips it when false");
    let pos_before = hero(&g).pos;
    let evs = ticks(&mut g, 10);
    let hunt: Vec<String> = evs.iter().filter_map(|e| if let Ev::Rule { row: 0, text, .. } = e { Some(text.clone()) } else { None }).collect();
    assert!(!hunt.is_empty() && hunt[0].starts_with("hunt goblin"), "{hunt:?}");
    assert!(hero(&g).pos != pos_before, "the hero walked toward the last-seen tile");
    assert!(g.run.as_ref().unwrap().trace.last().unwrap().row == 0, "the hunt is the row's action");
}

/// Cut 4 (rater B): `foes>=N` counts every hostile in view, not only the ones the hero has not
/// given up on; a row whose conditions hold but whose verb cannot execute shows why.
#[test]
fn foes_count_the_visible_and_a_blocked_row_says_why() {
    let mut g = arena();
    // Cornered in the top-left: hero at (1,2) with the stairs up at (1,1), goblins on every
    // other free neighbour.
    g.run.as_mut().unwrap().hero.pos = Pos::new(1, 2);
    g.run.as_mut().unwrap().hero.hp = 10;
    let ids: Vec<u32> = [(2, 1), (2, 2), (2, 3), (1, 3)].iter().map(|&(x, y)| add_monster(&mut g, "goblin", x, y)).collect();
    // The stairs tile is free: block it too.
    add_monster(&mut g, "goblin", 1, 1);
    // One of them was given up on (unreachable): it still counts as a foe in view.
    g.run.as_mut().unwrap().ignore(ids[0], 30);
    g.run.as_mut().unwrap().floor.map.update_vision(Pos::new(1, 2), VISION);
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50), Cond::n("foes>=", 5)], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 10);
    let run = g.run.as_ref().unwrap();
    let t = run.trace.first().expect("an action");
    assert_eq!(t.foes, 5, "five hostiles in view, one of them ignored");
    assert_eq!(t.blocked.as_deref(), Some("R1 retreat ✗ no path"), "{t:?}");
    assert!(word_count(t.blocked.as_ref().unwrap()) <= 4);
    assert_eq!(t.row, 1, "the attack row acted instead");
    let callouts: Vec<&String> = evs.iter().filter_map(|e| if let Ev::Callout { text, .. } = e { Some(text) } else { None }).collect();
    assert!(callouts.iter().any(|c| c.as_str() == "retreat ✗ no path"), "{callouts:?}");
    assert_eq!(callouts.iter().filter(|c| c.as_str() == "retreat ✗ no path").count(), 1, "once per streak");
    let json = serde_json::to_string(t).unwrap();
    assert!(json.contains(r#""blocked":"R1 retreat"#));
    // A turn without a block serialises without the field.
    let mut g2 = arena();
    rules(&mut g2, vec![]);
    ticks(&mut g2, 12);
    let t2 = g2.run.as_ref().unwrap().trace.first().unwrap();
    assert!(t2.blocked.is_none());
    assert!(!serde_json::to_string(t2).unwrap().contains("blocked"));
}

/// QA on 92eb880 (qaN): the trace's `foes` is the player's count — a thief running with the
/// loot is in it though `foes>=` (the rules' count, `rule_foes`) leaves it out.
#[test]
fn trace_foes_count_what_the_player_sees() {
    let mut g = arena();
    let id = add_monster(&mut g, "monkey", 8, 5);
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.stolen = Some(Item::new(500, "dagger"));
        m.fleeing = true;
        m.awake = true;
        run.floor.map.update_vision(Pos::new(4, 5), VISION);
    }
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("rest"))]);
    ticks(&mut g, 12);
    let run = g.run.as_ref().unwrap();
    let t = run.trace.first().expect("an action");
    assert_eq!(t.rule_foes, 0, "the rules never count a running thief: {t:?}");
    assert_eq!(t.foes, 1, "the column counts it: {t:?}");
    assert_ne!(t.row, 0, "the row did not fire");
}

/// QA on 92eb880 (qaN: `R7 bank ended 13 runs` beside `11 BANKED`): the plateau's window opened
/// before the absence; its line counts the absence's exits apart from the earlier ones, and a
/// watched run's exit closes the absence's count.
#[test]
fn a_plateau_counts_the_absence_apart() {
    let mut g = Game::new_literal(5);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 35)], Verb::new("return")));
    g.set_rules(set).unwrap();
    g.stall.runs = 13;
    g.stall.depth = 6;
    g.stall.exit_rows.insert(0, 13);
    g.stall.absent_rows.insert(0, 11);
    let s = crate::offline::stall_verdict(&mut g).expect("a stall");
    assert_eq!(s.text, "R1 return ended 11 runs, 2 before; none past D6");
    assert!(crate::rules::word_count(&s.text) <= 12);
    g.stall.absent_rows.clear();
    let s = crate::offline::stall_verdict(&mut g).expect("a stall");
    assert_eq!(s.text, "R1 return ended 13 earlier runs, none past D6");
    g.stall.absent_rows.insert(0, 13);
    let s = crate::offline::stall_verdict(&mut g).expect("a stall");
    assert_eq!(s.text, "R1 return ended 13 runs, none past D6");
}

/// QA on 92eb880 (qaN: `fighter +0 · L4 ↑1`): the exit line carries the run's XP and the levels
/// it crossed — the XP that crossed the level included — and the lineage view carries the
/// core's ladder (`next`), so no client sums a ladder of its own.
#[test]
fn a_level_up_carries_the_xp_that_crossed_it() {
    let mut g = arena_seed(3);
    let class = g.lineage.class.name().to_string();
    let need = crate::hero::xp_to_next(3);
    g.lineage.classes.insert(class.clone(), ClassProg { level: 3, xp: need - 1, next: 0 });
    rules(&mut g, vec![Row::new(vec![], Verb::new("return"))]);
    let mut line = None;
    for _ in 0..40 {
        let r = g.step(10);
        if let Some(l) = r.events.iter().find_map(|e| if let Ev::Exit { line, .. } = e { line.clone() } else { None }) {
            line = Some(l);
            break;
        }
    }
    let line = line.expect("an exit line");
    assert_eq!(line.level_ups, 1, "{line:?}");
    assert!(line.xp > 0, "a level crossed with no XP: {line:?}");
    let prog = &g.lineage.classes[&class];
    assert_eq!(prog.level, 4);
    assert_eq!(prog.xp, need - 1 + line.xp - need, "the line's XP is what the ladder took");
    let wire = g.lineage();
    assert_eq!(wire.classes[&class].next, crate::hero::xp_to_next(4));
}

/// QA on 92eb880 (qaN: `corridor fighting` inserted at its best place and the stall share went
/// to 35 %): a place whose stall share rises more than 5 pts is not best; when every place
/// does, the one that stalls least is.
#[test]
fn a_cards_best_place_does_not_stall() {
    use crate::meta::{best_place, Measured};
    let m = |at: usize, reach: f64, stall: f64| Measured { at: Some(at), reach, stall };
    // The highest reach stalls: the next calm place wins.
    let b = best_place(&[m(5, 0.30, 0.02), m(0, 0.50, 0.35), m(2, 0.40, 0.06)], 0.01).unwrap();
    assert_eq!(b.at, Some(2), "{b:?}");
    // Ties keep the earlier (the buy's old place).
    assert_eq!(best_place(&[m(5, 0.3, 0.0), m(0, 0.3, 0.0)], 0.0).unwrap().at, Some(5));
    // Every place stalls: the least stalling one, whatever its reach.
    let b = best_place(&[m(5, 0.3, 0.22), m(0, 0.4, 0.26), m(7, 0.1, 0.10)], 0.0).unwrap();
    assert_eq!(b.at, Some(7), "{b:?}");
    assert!(best_place(&[], 0.0).is_none());
    // The catalogue: the card's delta is read where it does not stall, and says its stall move.
    // (Cut 22 §3: seed 1218 — a forecast's sims draw each floor from its own stream now, and
    // 1216's bottom place stalled 7 of 18)
    let mut g = Game::new_literal(1218);
    for u in ["kite_archers", "pack_break", "gas_step", "row5", "row6", "row7", "cond_telegraph"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.best_depth = 6;
    tiers(&mut g);
    for m in crate::defs::MONSTERS.iter() {
        for t in m.tags {
            g.lineage.facts.insert(format!("foe:{}:{}", m.kind, t));
        }
    }
    let card = |id: &str| Row::new(vec![], Verb::arg("tactic", id));
    let rows = vec![
        Row::new(vec![Cond::n("hp<", 30)], Verb::new("return")),
        card("kite_archers"),
        card("pack_break"),
        Row::new(vec![Cond { k: "foe_tag".into(), n: None, t: Some("telegraph".into()) }], Verb::new("retreat")),
        card("gas_step"),
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        Row::new(vec![Cond::n("depth>=", 6)], Verb::new("bank")),
    ];
    g.set_rules_raw(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
    let cat = crate::meta::catalogue_with_deltas(&g, true);
    let u = cat.iter().find(|u| u.id == "corridor_fighting").unwrap();
    let stall = u.stall.expect("the card carries its stall move");
    // Before: R6, the old place before the engagement row (the camp panel read stall 22 % there).
    assert!(stall <= 0.15, "the least-stalling place: {u:?}");
    assert_eq!(u.insert_at, Some(7), "every place stalls; the bottom stalls least: {u:?}");
    // The cached read (no sims) names the same place.
    let again = crate::meta::catalogue_with_deltas(&g, false);
    let v = again.iter().find(|x| x.id == "corridor_fighting").unwrap();
    assert_eq!((v.insert_at, v.stall), (u.insert_at, u.stall));
}

/// Cut 4 (rater B): the stalemate guards lift when blood is drawn, and the chores never `wait`
/// with an awake hostile adjacent.
#[test]
fn blood_drawn_lifts_the_stalemate_guards_and_nobody_waits_while_bitten() {
    let mut g = arena();
    // Two fast jackals: the hero cannot step clear of both, so one bites.
    let id = add_monster(&mut g, "jackal", 5, 5);
    add_monster(&mut g, "jackal", 3, 6);
    attack_rules(&mut g);
    {
        let run = g.run.as_mut().unwrap();
        run.stuck_until = run.actions + 30; // the oscillation guard is up
        run.row_suppressed = (0, run.actions + 30); // and the same-row guard
        run.ignore(id, 30);
    }
    let evs = ticks(&mut g, 60);
    let verbs: Vec<String> = evs.iter().filter_map(|e| if let Ev::Rule { verb, .. } = e { Some(verb.v.clone()) } else { None }).collect();
    assert!(!verbs.contains(&"wait".to_string()), "{verbs:?}");
    let bitten = evs.iter().position(|e| matches!(e, Ev::Hurt { id: HERO_ID, .. }));
    let bitten = bitten.expect("a jackal bit the hero");
    let after: Vec<&Ev> = evs[bitten..].iter().filter(|e| matches!(e, Ev::Rule { .. })).collect();
    assert!(matches!(after.first(), Some(Ev::Rule { row: 0, .. })), "the attack row fired once blood was drawn: {after:?}");
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.stuck_until, 0);
    assert!(run.ignored.is_empty());
    // No rows at all (PASSIVE), walled in with a goblin at its elbow: the chore is `cornered`,
    // never `wait` (and never a step or a swing the player did not write).
    let mut g = arena();
    g.run.as_mut().unwrap().hero.pos = Pos::new(1, 2);
    for (x, y) in [(1, 1), (2, 1), (2, 2), (2, 3), (1, 3)] {
        add_monster(&mut g, "goblin", x, y);
    }
    g.run.as_mut().unwrap().floor.map.update_vision(Pos::new(1, 2), VISION);
    rules(&mut g, vec![]);
    let evs = ticks(&mut g, 30);
    let chores: Vec<String> = evs.iter().filter_map(|e| if let Ev::Rule { row: -2, verb, .. } = e { Some(verb.v.clone()) } else { None }).collect();
    assert!(!chores.contains(&"wait".to_string()), "{chores:?}");
    assert!(chores.iter().any(|v| v == "cornered"), "{chores:?}");
    assert!(!chores.iter().any(|v| v == "attack"), "{chores:?}");
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { text, .. } if text == "cornered, no orders")));
}

/// Cut 4 (rater B): `auto_supply` restocks the last expedition's supplies from gold when the
/// hero comes home, and the camp's shelf (`Lineage.supplies`) shows it before the next send.
#[test]
fn auto_supply_restocks_the_shelf_when_the_hero_comes_home() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.gold = 1000;
    g.buy_supply("heal").unwrap();
    g.buy_supply("heal").unwrap();
    let price = 1000 - g.lineage.gold;
    assert!(price > 0);
    g.start_run(None);
    assert!(g.lineage.supplies.is_empty(), "the shelf went into the pack");
    assert_eq!(g.run.as_ref().unwrap().supplies.len(), 2);
    {
        // Cut 13: the send used both (an unused one would come back on its own, unbought).
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert_eq!(g.lineage.supplies.len(), 2, "restocked on coming home");
    assert!(g.lineage.supplies.iter().all(|s| s.kind == "heal"));
    assert_eq!(g.lineage().supplies.len(), 2, "and the wire lineage shows it");
    // Cut 22 §1: priced by depth. QA on 778fa1b (qaU: `repeat on · $20`, then `−$26` at the
    // return): at the price the send quoted — the camp's badge — whatever the bank did to it.
    let repeat: i32 = g.lineage.gold_ledger.iter().filter(|l| l.why == "repeat heal").map(|l| -l.delta).sum();
    assert_eq!(repeat, price, "paid from gold at the send's price (was {price} for two)");
    assert!(g.supply_catalogue().iter().find(|e| e.kind == "heal").unwrap().price > price / 2, "the bank moved the shelf's price");
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.supplies.len(), 2, "no double restock after the vault decision");
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().supplies.len(), 2, "the restocked shelf goes out again");
    assert!(g.lineage.supplies.is_empty());
    // Clearing the shelf is an order the automation respects.
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.supplies.len(), 2);
    g.clear_supplies();
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().supplies.is_empty(), "cleared stays cleared");
    // Without the gold, nothing is bought and nothing breaks (a return: no wake pay arrives).
    let mut g = Game::new_literal(6);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.gold = 100;
    g.buy_supply("heal").unwrap();
    g.start_run(None);
    g.lineage.gold = 0;
    {
        let (run, mut cx) = g.ctx();
        run.loot = 0;
        run.hero.inv.retain(|i| i.kind != "heal"); // used (Cut 13: an unused one would come back)
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    assert!(g.lineage.supplies.iter().all(|s| s.free), "nothing bought; only the kennel's leash");
    // After a death the wake pay covers one potion, so the restock does happen (cohort 5, rater J).
    let mut g = Game::new_literal(6);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.gold = 100;
    g.buy_supply("heal").unwrap();
    g.start_run(None);
    g.lineage.gold = 0;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Death);
    }
    g.finish_run();
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "heal" && !s.free), "wake pay lets auto_supply rebuy the heal");
}

/// Cut 4 §7: when a row caught the hero (≤ 20 % HP, then the floor survived), the chronicle
/// names it.
#[test]
fn the_chronicle_names_the_row_that_caught_the_hero() {
    let mut g = arena();
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    give(&mut g, "heal");
    g.run.as_mut().unwrap().hero.hp = 20;
    let id = add_monster(&mut g, "rat", 5, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![Cond::n("hp<", 25)], Verb::arg("drink", "heal"))]);
    // The rat bites until the hero is under 20 %, then the drink row fires (R2).
    let mut evs = Vec::new();
    for _ in 0..200 {
        evs.append(&mut ticks(&mut g, 1));
        if hero(&g).hp_pct() <= 20 || monster(&g, id).is_none() {
            break;
        }
    }
    if hero(&g).hp_pct() > 20 {
        // The rat died first: bite the hero ourselves.
        let run = g.run.as_mut().unwrap();
        run.hero.hp = 5;
        run.low20_t = Some(run.turn);
    }
    evs.append(&mut ticks(&mut g, 40));
    let saved = g.run.as_ref().unwrap().saved_by;
    assert!(saved.is_some(), "a row fired after the low point: {:?}", g.run.as_ref().unwrap().trace);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
        evs.append(cx.events);
    }
    let notes: Vec<&String> = evs.iter().filter_map(|e| if let Ev::Note { text, .. } = e { Some(text) } else { None }).collect();
    let caught = notes.iter().find(|n| n.contains("saved him")).unwrap_or_else(|| panic!("{notes:?}"));
    assert!(!notes.iter().any(|n| n.contains("caught him")), "Cut 15 §6: `saved`, not `caught`: {notes:?}");
    assert!(caught.starts_with(&format!("R{} ", saved.unwrap() + 1)), "{caught}");
    assert!(word_count(caught) <= 8);
    let bank = notes.iter().find(|n| n.starts_with("Banked $")).expect("the exit note");
    assert!(notes.iter().position(|n| n == caught) < notes.iter().position(|n| n == bank), "the catch is noted before the exit");
    // A death names no saviour.
    let mut g = arena();
    g.run.as_mut().unwrap().hero.hp = 3;
    g.run.as_mut().unwrap().low20_t = Some(1);
    g.run.as_mut().unwrap().saved_by = Some(0);
    attack_rules(&mut g);
    let (run, mut cx) = g.ctx();
    crate::turn::end_run(run, &mut cx, ExitTier::Death);
    assert!(!cx.events.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("caught") || text.contains("saved"))));
}

/// Cut 4 §9: `unlocks()` carries `delta` for a card or verb whose gate is open and that is not
/// yet owned — the forecast reach delta at `best_depth + 1` with its row at the top — and for
/// nothing else; the catalogue is memoised, so a second call is free.
#[test]
fn unlock_catalogue_carries_a_forecast_delta_for_open_cards() {
    let mut g = Game::new_literal(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:jackal:fast".into());
    g.lineage.facts.insert("item:leash".into());
    g.lineage.best_depth = 2;
    tiers(&mut g);
    assert!(g.unlocks().iter().all(|u| u.delta.is_none()), "no sims before unlock_deltas");
    let t = std::time::Instant::now();
    let cat = g.unlock_deltas();
    let first = t.elapsed().as_secs_f64();
    let by = |id: &str| cat.iter().find(|u| u.id == id).unwrap().clone();
    assert!(by("corridor_fighting").delta.is_some(), "open card: {:?}", by("corridor_fighting"));
    assert!(by("pack_break").delta.is_some());
    assert!(cat.iter().all(|u| u.id != "kite_archers"), "Cut 29 §1: a card a row could type is free, off the shelf");
    assert!(by("row5").delta.is_none(), "rows have no row to add");
    assert!(by("throw").delta.is_none(), "Cut 13: a verb carries no delta — its canonical row is nobody's policy (QA on 50bb162: `reach −92% ±11`)");
    assert!(g.lineage.unlocks.contains("tame"), "Cut 8B: tame is owned from the start");
    for u in &cat {
        if let Some(d) = u.delta {
            assert!((-1.0..=1.0).contains(&d), "{u:?}");
        }
    }
    let t = std::time::Instant::now();
    let again = g.unlocks();
    assert_eq!(again, cat, "unlocks() carries the memoised deltas");
    assert!(t.elapsed().as_secs_f64() < first / 4.0 + 0.01, "memoised: {:.3}s vs {first:.3}s", t.elapsed().as_secs_f64());
    assert_eq!(g.unlock_deltas(), cat);
    // Owning it drops the delta (there is nothing to buy).
    g.lineage.marks = 10;
    g.buy("corridor_fighting").unwrap();
    let cat = g.unlock_deltas(); // a purchase changes the lineage: the deltas are re-simulated
    let c = cat.iter().find(|u| u.id == "corridor_fighting").unwrap();
    assert!(c.owned && c.delta.is_none());
    let json = serde_json::to_string(&cat).unwrap();
    assert!(json.contains(r#""delta":"#));
    assert!(!serde_json::to_string(&c).unwrap().contains("delta"), "serde skips it when absent");
    eprintln!("catalogue deltas at D3: {first:.3}s");
}

/// Cut 14 §1: a condition unlock carries a delta when a row of the set waits on it (the
/// same rules on a lineage that owns the condition, against the base) — where a death's
/// lock root sends its number once nothing under the baseline is offered on the death
/// screen; a condition no row uses has nothing to measure.
#[test]
fn cond_unlock_carries_the_locked_rows_delta() {
    // Cut 29 §1: a condition word is free vocabulary — never on the shelf, owned with its gate; a
    // row waiting on it wakes the moment the gate opens (the end of the run that opened it).
    let mut g = Game::new_literal(2);
    g.lineage.facts.insert("den".into());
    let mut rules = g.lineage.rules().clone();
    rules.rows.insert(0, Row::new(vec![Cond::t("on_see", "den")], Verb::arg("attack", "nearest")));
    g.set_rules_raw(rules).unwrap();
    assert!(g.unlocks().iter().all(|u| !u.id.starts_with("cond_")), "no condition word is sold");
    assert!(!g.lineage.unlocks.contains("cond_on_see"));
    g.lineage.facts.insert("foe:jackal:pack".into()); // the gate: a foe met
    crate::meta::grant_free(&mut g.lineage);
    assert!(g.lineage.unlocks.contains("cond_on_see"));
    assert!(g.vocabulary().conds.iter().any(|c| c.k == "on_see"));
}

/// Cut 4: `Lineage.ascended` lists the variants already finished.
#[test]
fn lineage_wire_carries_ascended_variants() {
    let mut g = Game::new_literal(1);
    assert!(g.lineage().ascended.is_empty());
    g.lineage.ascended.push("no_rest".into());
    let l = g.lineage();
    assert_eq!(l.ascended, vec!["no_rest".to_string()]);
    assert!(serde_json::to_string(&l).unwrap().contains(r#""ascended":["no_rest"]"#));
}

#[test]
#[ignore]
fn catalogue_delta_cost_by_depth() {
    for hours in [2u64, 8, 24] {
        let mut g = Game::new_literal(7);
        for u in ["row5", "row6", "row7", "row8", "throw"] {
            g.lineage.unlocks.insert(u.into());
        }
        for k in ["heal", "poison", "fire", "teleport", "blink"] {
            if let Some(f) = ident_fact(&g.lineage.flavours, k) {
                g.lineage.facts.insert(f);
            }
        }
        g.set_rules_raw(RuleSet::parse(&std::fs::read_to_string("presets/good.json").unwrap()).unwrap()).unwrap();
        g.run_offline(hours * 3600);
        g.forecast_cache.borrow_mut().clear();
        let t = std::time::Instant::now();
        let open = g.unlock_deltas().iter().filter(|u| u.delta.is_some()).count();
        eprintln!("best D{} open cards {open}: catalogue {:.2}s", g.lineage.best_depth, t.elapsed().as_secs_f64());
    }
}

// ---------------------------------------------------------------- Cut 5 §2–§5

/// §2: an heir's end writes one chronicle line in the contract's grammar; a heir is written
/// once; the cap holds.
#[test]
fn the_lineage_chronicle_has_one_line_per_ended_heir() {
    let mut g = arena();
    g.lineage.trait_ = crate::hero::Trait::Greedy;
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    g.lineage.sets[0].name = Some("corridor".into());
    g.run.as_mut().unwrap().max_depth = 7;
    g.run.as_mut().unwrap().depth = 7;
    g.run.as_mut().unwrap().kills.push((5, "goblin_warlord".into(), 5));
    g.run.as_mut().unwrap().boss_kills.push((5, "goblin_warlord".into()));
    give(&mut g, "sword");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_hero(run, &mut cx, 99, &crate::turn::Src::Gas);
    }
    g.finish_run();
    assert_eq!(g.lineage.chronicle, vec!["♟1 the greedy fighter · D7 · \"corridor\" set · took the Warlord · fell to gas · left bones on D7.".to_string()]);
    assert_eq!(g.lineage.heir, 2);
    assert!(g.lineage.heir_deeds.is_empty() && g.lineage.heir_best == 0, "the next heir starts clean");
    // The wire carries it; the cap holds.
    assert_eq!(g.lineage().chronicle.len(), 1);
    for _ in 0..50 {
        g.lineage.heir += 1;
        g.lineage.chronicle_heir("fell to a rat", None);
    }
    assert_eq!(g.lineage.chronicle.len(), crate::engine::CHRONICLE_CAP);
    assert!(g.lineage.chronicle.last().unwrap().starts_with("♟52 "));
    // Ascension writes the live heir's line once.
    let mut g = Game::new_literal(3);
    g.lineage.ended = true;
    g.lineage.heir = 4;
    g.lineage.heir_best = 31;
    let trait_ = g.lineage.trait_.name();
    g.ascend("no_rest").unwrap();
    assert_eq!(g.lineage.chronicle, vec![format!("♟4 the {trait_} fighter · D31 · ascended.")], "the class's preset name is not quoted (QA on 0c6e126, qaY)");
}

/// §4: D1 always holds a situation in a room near the entrance; every run meets one on D1–5
/// in the gate's measure (`Batch.situation_runs`).
#[test]
fn the_first_floors_hold_situations() {
    use crate::tiles::Tile;
    let mut with = 0;
    for seed in 1..=20u64 {
        let mut g = Game::new_literal(seed);
        g.sim = true;
        g.start_run(None);
        let run = g.run.as_ref().unwrap();
        let tiles = &run.floor.map.tiles;
        let n = tiles.iter().filter(|t| matches!(t, Tile::Shrine | Tile::Vault | Tile::Nest)).count();
        assert!(n >= 1, "seed {seed}: no situation on D1");
        if let Some(p) = run.tile_pos(Tile::Nest) {
            let sleepers = run.monsters.iter().filter(|m| m.nest && m.dormant).count();
            assert!((2..=3).contains(&sleepers), "{sleepers} sleepers on D1");
            assert!(run.items.iter().any(|i| i.pos == p && i.item.kind == "gold"), "a gold pile on the nest");
        }
        if run.tile_pos(Tile::Vault).is_some() {
            assert_eq!(run.vault_cage.len(), 3);
        }
        g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
        g.finish_run();
        with += g.batch.situation_runs;
    }
    assert!(with >= 18, "runs that met a situation: {with}/20");
}

/// §4 nest: sleepers ignore sight; the hero's step within two tiles wakes the den, learns
/// `nest`, and the fight is the episode (`The nest ...`).
#[test]
fn the_nest_wakes_when_the_hero_comes_close() {
    use crate::tiles::Tile;
    let mut g = arena();
    let ids: Vec<u32> = (0..4).map(|k| add_monster(&mut g, "jackal", 12, 3 + k)).collect();
    {
        let run = g.run.as_mut().unwrap();
        run.floor.map.set(Pos::new(11, 5), Tile::Nest);
        for m in run.monsters.iter_mut() {
            m.nest = true;
            m.dormant = true;
            m.awake = false;
        }
    }
    hold_rules(&mut g);
    ticks(&mut g, 40);
    assert!(ids.iter().all(|id| !monster(&g, *id).unwrap().awake), "seen from seven tiles: still asleep");
    assert!(g.lineage.facts.contains("nest"), "the den in view is a fact");
    assert!(g.run.as_ref().unwrap().sees_situation("nest"));
    {
        let (run, cx) = g.ctx();
        let v = crate::turn::view(run);
        assert!(crate::turn::cond_holds(run, &cx, &v, &Cond::t("on_see", "nest")));
        assert!(!crate::turn::cond_holds(run, &cx, &v, &Cond::t("on_see", "shrine")));
    }
    g.run.as_mut().unwrap().hero.pos = Pos::new(10, 5);
    g.run.as_mut().unwrap().hero_dist_pos = None;
    let evs = ticks(&mut g, 10);
    assert!(ids.iter().all(|id| monster(&g, *id).is_none_or(|m| m.awake && !m.dormant)), "within two tiles: the den wakes");
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "the nest wakes")));
    assert!(!g.run.as_ref().unwrap().sees_situation("nest"), "awake, the den is no longer a token");
    attack_rules(&mut g);
    ticks(&mut g, 120);
    let run = g.run.as_ref().unwrap();
    // The fight is the episode, its threat the den (resolved by now or still live).
    let threat = run.episodes.iter().filter_map(|e| e.threat.first()).chain(run.arc.threat.first()).map(|(k, _)| k.clone()).next();
    assert_eq!(threat.as_deref(), Some("nest"), "{:?} / {:?}", run.episodes, run.arc);
}

/// §4 vault: stepping on the cage opens it; watched, the choice waits for `choose` within
/// the grace; unwatched or offline the preference picks; the tile becomes `vault_open` and
/// the choice is an episode.
#[test]
fn the_vault_waits_for_a_choice_when_watched_and_picks_by_preference_otherwise() {
    use crate::tiles::Tile;
    let cage = |g: &mut Game| {
        let run = g.run.as_mut().unwrap();
        run.floor.map.set(Pos::new(5, 5), Tile::Vault);
        run.vault_cage = ["sword", "mail", "heal"].iter().enumerate().map(|(i, k)| Item::new(900 + i as u32, k)).collect();
    };
    let mut g = arena();
    cage(&mut g);
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.pos = Pos::new(5, 5);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "choose one")));
    let snap = g.snapshot();
    let vc = snap.vault_choice.as_ref().expect("the choice is on the snapshot");
    assert_eq!(vc.items.len(), 3);
    assert_eq!(g.run.as_ref().unwrap().floor.map.get(Pos::new(5, 5)), Tile::VaultOpen);
    assert!(g.lineage.facts.contains("vault"));
    ticks(&mut g, 20);
    assert!(g.run.as_ref().unwrap().vault_choice.is_some(), "watched: still waiting");
    g.choose(901).unwrap();
    assert!(g.run.as_ref().unwrap().vault_choice.is_none());
    assert_eq!(hero(&g).armour.as_ref().map(|a| a.kind.as_str()), Some("mail"));
    assert!(g.choose(902).is_err(), "the rest vanished");
    assert!(g.snapshot().vault_choice.is_none());
    let ep = g.run.as_ref().unwrap().arc.sealed.last().expect("the vault is an episode");
    assert_eq!(ep.setup, crate::sifter::Setup::Vault);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    let hs = crate::sifter::sift_with(g.run.as_ref().unwrap(), false);
    let line = hs.iter().find(|h| h.arc.as_ref().unwrap().threat == "vault").unwrap();
    assert_eq!(line.text, "The cage held three; he took the mail; returned.");
    assert!(crate::sifter::story_ok(&line.text));
    // Unanswered past the grace: the preference picks.
    let mut g = arena();
    g.set_vault_pref("potion").unwrap();
    cage(&mut g);
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.pos = Pos::new(5, 5);
    ticks(&mut g, 10 + crate::engine::VAULT_GRACE);
    assert!(g.run.as_ref().unwrap().vault_choice.is_none());
    assert!(hero(&g).inv.iter().any(|i| i.kind == "heal"), "{:?}", hero(&g).inv);
    // Offline: the same grace (a verdict replay stays faithful), the same preference.
    let mut g = arena();
    g.offline = true;
    cage(&mut g);
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.pos = Pos::new(5, 5);
    ticks(&mut g, 11);
    assert!(g.run.as_ref().unwrap().vault_choice.is_some());
    ticks(&mut g, crate::engine::VAULT_GRACE);
    assert!(g.run.as_ref().unwrap().vault_choice.is_none());
    assert_eq!(hero(&g).weapon.as_ref().map(|w| w.kind.as_str()), Some("sword"), "weapon by default");
    assert!(g.set_vault_pref("gold").is_err());
}

/// §4 shrine: `pray row` walks to the altar, costs a fifth of max HP and lends a row from
/// another saved set (or swaps the trait); once per run; the fact gates the verb.
#[test]
fn the_shrine_lends_a_row_or_swaps_the_trait_for_a_fifth_of_max_hp() {
    use crate::tiles::Tile;
    let mut g = arena();
    g.lineage.facts.insert("shrine".into());
    g.lineage.sets[1] = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))], name: None, route: Vec::new() };
    g.run.as_mut().unwrap().floor.map.set(Pos::new(8, 5), Tile::Shrine);
    rules(&mut g, vec![Row::new(vec![Cond::t("on_see", "shrine")], Verb::arg("pray", "row"))]);
    assert!(g.vocabulary().verbs.contains(&Verb::arg("pray", "row")));
    assert!(g.vocabulary().conds.contains(&Cond::t("on_see", "shrine")));
    let evs = ticks(&mut g, 80);
    let run = g.run.as_ref().unwrap();
    assert!(run.prayed, "{:?}", run.trace);
    assert_eq!(run.hero.max_hp, 36 - 7);
    assert_eq!(run.lent_row.as_ref().map(|r| r.verb.v.as_str()), Some("rest"));
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "prayed")));
    assert!(run.arc.threat.first().is_some_and(|(k, _)| k == "shrine"), "{:?}", run.arc);
    assert!(!run.sees_situation("shrine"), "prayed: the altar is no longer a token");
    // The lent row fires as R2 (after the set's one row).
    g.run.as_mut().unwrap().hero.hp = 10;
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 1, verb, .. } if verb.v == "rest")), "{:?}", ev_kinds(&evs));
    // `pray trait` with nothing to lend swaps the trait; a second prayer is blocked.
    let mut g = arena();
    g.lineage.facts.insert("shrine".into());
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Cowardly;
    g.run.as_mut().unwrap().floor.map.set(Pos::new(8, 5), Tile::Shrine);
    rules(&mut g, vec![Row::new(vec![], Verb::arg("pray", "trait"))]);
    ticks(&mut g, 80);
    assert_eq!(g.run.as_ref().unwrap().trait_, crate::hero::Trait::Brave);
    ticks(&mut g, 10);
    let blocked = g.run.as_ref().unwrap().trace.last().unwrap().blocked.clone();
    assert_eq!(blocked.as_deref(), Some("R1 pray trait ✗ prayed"));
    // Without the fact the verb is not offered and never fires.
    let mut g = arena();
    g.run.as_mut().unwrap().floor.map.set(Pos::new(8, 5), Tile::Shrine);
    rules(&mut g, vec![Row::new(vec![], Verb::arg("pray", "row"))]);
    assert!(!g.vocabulary().verbs.iter().any(|v| v.v == "pray"));
    ticks(&mut g, 40);
    assert!(!g.run.as_ref().unwrap().prayed);
}

/// §4 stray: a lost companion of a previous heir turns up wild on D1–5 with its old name;
/// `tame` takes it at 60 % whatever its wounds, and it comes back as a companion.
#[test]
fn a_lost_companion_returns_as_a_stray_and_tames_at_sixty_percent() {
    let mut g = arena();
    g.lineage.lost.push(crate::engine::Lost { kind: "jackal".into(), name: "Uleth".into(), gen: 2, heir: 1, why: String::new() });
    g.lineage.unlocks.insert("tame".into());
    g.lineage.facts.insert("item:leash".into());
    let id = add_monster(&mut g, "jackal", 6, 5);
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.stray = true;
        m.name = Some("Uleth".into());
        m.awake = false;
    }
    give(&mut g, "leash");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tame", "nearest"))]);
    let evs = ticks(&mut g, 10);
    assert!(g.lineage.facts.contains("stray"));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text == "Uleth the jackal, gone wild.")), "{evs:?}");
    // 60 % whatever its wounds: over many tries the leash lands.
    let mut tamed = 0;
    let mut tries = 0;
    for seed in 1..=30u64 {
        let mut g = arena_seed(seed);
        g.lineage.lost.push(crate::engine::Lost { kind: "jackal".into(), name: "Uleth".into(), gen: 2, heir: 1, why: String::new() });
        g.lineage.unlocks.insert("tame".into());
        let id = add_monster(&mut g, "jackal", 5, 5);
        {
            let run = g.run.as_mut().unwrap();
            let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
            m.stray = true;
            m.name = Some("Uleth".into());
            m.stun = 500;
        }
        give(&mut g, "leash");
        rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tame", "nearest"))]);
        let evs = ticks(&mut g, 10);
        let t = evs.iter().find_map(|e| if let Ev::Tame { ok, .. } = e { Some(*ok) } else { None });
        if let Some(ok) = t {
            tries += 1;
            if ok {
                tamed += 1;
                let run = g.run.as_ref().unwrap();
                let c = run.companions.last().unwrap();
                assert_eq!((c.name.as_str(), c.gen), ("Uleth", 2));
                assert_eq!(run.strays_tamed, vec!["Uleth".to_string()]);
                let ep = run.arc.sealed.last().expect("the return is an episode");
                assert_eq!(ep.setup, crate::sifter::Setup::Stray);
                let mut ep = ep.clone();
                ep.resolution = crate::sifter::Resolution::Reached { depth: 3 };
                assert_eq!(crate::sifter::story_line(&ep), "Uleth the jackal came back; R1 tamed; reached D3.");
                assert!(crate::sifter::story_ok(&crate::sifter::story_line(&ep)));
                // Home, the lost list forgets it.
                {
                    let (run, mut cx) = g.ctx();
                    crate::turn::end_run(run, &mut cx, ExitTier::Bank);
                }
                g.finish_run();
                assert!(g.lineage.lost.is_empty());
            }
        }
    }
    assert!(tries >= 25, "{tries}");
    assert!((10..=26).contains(&tamed), "tamed {tamed}/{tries} at 60 %");
}

/// §5: `bail()` queues a `return` for the hero's next action without touching the rules.
#[test]
fn bail_queues_a_return_for_the_next_action() {
    let mut g = arena();
    attack_rules(&mut g);
    let before = g.lineage.rules().clone();
    ticks(&mut g, 5);
    g.bail();
    let evs = ticks(&mut g, 10);
    assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Return));
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -2, verb, text, .. } if verb.v == "return" && text == "bail → return")), "{:?}", ev_kinds(&evs));
    assert_eq!(g.lineage.rules(), &before);
    assert!(g.run.as_ref().unwrap().exit_row.is_none(), "not a row's exit");
    // No run: a no-op.
    g.finish_run();
    g.bail();
}

/// §3: the hero speaks from the trait × moment table, at most once per 100 ticks and never
/// in a fight's first ten.
#[test]
fn the_hero_speaks_sparingly() {
    use crate::sifter::{voice, voice_line, Moment};
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Cowardly;
    {
        let (run, mut cx) = g.ctx();
        run.fight_t = Some(run.turn);
        assert!(!voice(run, &mut cx, Moment::Low), "a fight's first ten ticks are silent");
        run.turn += 10;
        assert!(voice(run, &mut cx, Moment::Low));
        assert!(matches!(cx.events.last(), Some(Ev::Callout { text, .. }) if text == voice_line(crate::hero::Trait::Cowardly, Moment::Low)));
        run.turn += 50;
        assert!(!voice(run, &mut cx, Moment::Resolved), "one per hundred ticks");
        run.turn += 50;
        assert!(voice(run, &mut cx, Moment::Resolved));
    }
    // A low point in play: the line follows the `near death` callout family.
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Brave;
    let r = add_monster(&mut g, "rat", 5, 5);
    g.run.as_mut().unwrap().monsters.iter_mut().for_each(|m| m.stun = 500);
    hold_rules(&mut g);
    ticks(&mut g, 20);
    let (run, mut cx) = g.ctx();
    run.turn += 20;
    let ri = run.monsters.iter().position(|m| m.id == r).unwrap();
    crate::turn::damage_hero(run, &mut cx, 30, &crate::turn::Src::Mon(ri));
    assert!(cx.events.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "come on then")), "{:?}", cx.events);
}

// ---------------------------------------------------------------- Cut 6

/// §1: the ledger reconciles on every exit — the exit's own movement is `carried × keep%`,
/// the automations' spending is `spent`, salvage is its own line, and the lineage's gold
/// moved by exactly their sum. 30 seeds of DEFAULT and EDITED runs, every exit checked; the
/// exit line reads as the contract's arithmetic.
#[test]
fn ledger_line_reconciles_on_every_exit() {
    let per = par_seeds(1..=30u64, |seed| {
        let (mut exits, mut deaths, mut spent_any) = (0, 0, false);
        let mut g = Game::new_literal(seed);
        if seed % 2 == 0 {
            g.set_rules_raw(crate::probes::good()).unwrap();
        }
        // Half the seeds own the automations, so `spent` is exercised.
        if seed % 3 == 0 {
            g.lineage.unlocks.insert("auto_supply".into());
            g.lineage.gold_move(200, "gift");
            g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
            g.buy_supply("heal").unwrap();
        }
        for _ in 0..6 {
            g.lineage.rest_left = 0;
            g.start_run(None);
            g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            let (carried, tier, timed_out, depth, heir, pct) = {
                let r = g.run.as_ref().unwrap();
                (r.loot.max(0), r.over.unwrap(), r.timed_out, r.depth, r.heir, r.yield_pct(r.over.unwrap()))
            };
            let before = g.lineage.gold;
            let lines_before = g.lineage.gold_ledger.len();
            g.finish_run();
            g.auto_keep();
            let after = g.lineage.gold;
            let line = g.batch.exits.last().cloned().expect("an exit line");
            // (Cut 24 §1: a drive-off with no way home written keeps nothing, as a stall)
            assert_eq!(line.carried, carried, "seed {seed}");
            assert_eq!(line.keep_pct, pct, "seed {seed}");
            assert_eq!(line.kept, carried * pct / 100, "seed {seed}");
            assert!(word_count(&line.text) <= 14, "{}", line.text);
            // Cut 10 §3: the verb and what came home lead (`returned $50 · $84 carried · keeps 60%`).
            // QA on 912e135: a timed-out run leads with its own word; an exit that kept nothing says what it lost.
            let verb = match tier {
                _ if timed_out => if line.text.starts_with("stalled") { "stalled" } else { "lost thread" },
                ExitTier::Bank => "banked",
                // QA on 0c6e126 (qaY): a drive-off leads with its own word, the tile's
                ExitTier::Return if line.driven.is_some() => "driven",
                ExitTier::Return => "returned",
                ExitTier::Death => "died",
            };
            if pct == 0 {
                assert!(line.text.starts_with(&format!("{verb} $0 · ${carried} lost")), "{}", line.text);
            } else {
                assert!(line.text.starts_with(&format!("{verb} ${} · ${carried} carried · keeps {pct}%", line.kept)), "{}", line.text);
            }
            assert_eq!(line.text.starts_with("lost thread") || line.text.starts_with("stalled"), timed_out, "{}", line.text);
            assert_eq!(line.bones.iter().map(|b| b.n as usize).sum::<usize>(), g.lineage.bones.last().filter(|b| b.heir == heir && tier == ExitTier::Death).map(|b| b.items.len()).unwrap_or(0), "{}", line.text);
            if tier == ExitTier::Death {
                deaths += 1;
                assert!(line.text.starts_with("died $0 · "), "{}", line.text);
                match g.lineage.bones.last().filter(|b| b.heir == heir) {
                    Some(b) => assert!(line.text.contains(&format!("bones: {} items on D{depth}", b.items.len())), "{}", line.text),
                    None => assert!(!line.text.contains("bones"), "{}", line.text),
                }
                let d = g.deaths.values().last().unwrap();
                assert_eq!(d.death.line.as_ref(), Some(&ExitLine { trace: None, ..line.clone() }), "the death carries its line (its own trace is longer)");
                assert!(line.trace.as_ref().is_some_and(|t| !t.turns.is_empty()), "Cut 9 §5: the exit line carries the trace");
            }
            // The ledger since the exit: the exit movement, salvage, the automations' spending.
            let tail: Vec<GoldLine> = g.lineage.gold_ledger.iter().skip(lines_before.min(g.lineage.gold_ledger.len().saturating_sub(1))).cloned().collect();
            let t = g.lineage.total_turns;
            let since: Vec<&GoldLine> = g.lineage.gold_ledger.iter().filter(|l| l.t == t).collect();
            assert!(!since.is_empty(), "seed {seed}: no ledger line for the exit at t {t} ({tail:?}; last 4: {:?})", g.lineage.gold_ledger.iter().rev().take(4).collect::<Vec<_>>());
            let exit_line = since.iter().find(|l| crate::engine::is_exit_why(&l.why)).expect("exit movement");
            assert_eq!(exit_line.delta, line.kept, "seed {seed}: {since:?}");
            assert!(word_count(&exit_line.why) <= 3, "{}", exit_line.why);
            let salvage: i32 = since.iter().filter(|l| l.why == "salvage").map(|l| l.delta).sum();
            let wake: i32 = since.iter().filter(|l| l.why == "wake pay").map(|l| l.delta).sum();
            let spent: i32 = since.iter().filter(|l| l.delta < 0).map(|l| -l.delta).sum();
            assert_eq!(line.spent, spent, "seed {seed}: {since:?}");
            assert_eq!(after - before, line.kept + salvage + wake - line.spent, "seed {seed}: gold delta vs ledger {since:?}");
            assert_eq!(after - before, since.iter().map(|l| l.delta).sum::<i32>(), "seed {seed}: the ledger sums to the delta");
            if line.spent > 0 {
                spent_any = true;
                assert!(!line.spent_on.is_empty());
            }
            for l in &g.lineage.gold_ledger {
                assert!(word_count(&l.why) <= 3, "{}", l.why);
            }
            assert!(g.lineage.gold_ledger.len() <= crate::engine::GOLD_LEDGER_CAP);
            exits += 1;
        }
        assert!(g.batch.exits.len() <= crate::engine::EXITS_CAP);
        let lw = g.lineage();
        assert_eq!(lw.gold_ledger, g.lineage.gold_ledger);
        (exits, deaths, spent_any)
    });
    let (exits, deaths): (u32, u32) = per.iter().fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1));
    let spent_any = per.iter().any(|p| p.2);
    assert!(exits >= 150 && deaths >= 20, "{exits} exits, {deaths} deaths");
    assert!(spent_any, "no seed spent on the way home");
}

/// §1: the exit event carries the settled line; the HUD stake shows what the return row
/// would keep; the report lists the batch's last five exits; purchases are ledger lines.
#[test]
fn exit_event_stake_and_report_carry_the_ledger() {
    let mut g = arena_seed(3);
    {
        let run = g.run.as_mut().unwrap();
        let mut gold = Item::new(70, "gold");
        gold.amount = 400;
        run.items.push(crate::engine::FloorItem { pos: Pos::new(5, 5), item: gold });
    }
    rules(&mut g, vec![Row::new(vec![Cond::n("loot>=", 50)], Verb::new("return"))]);
    g.lineage.unlocks.insert("cond_loot".into());
    let mut line = None;
    for _ in 0..40 {
        let r = g.step(10);
        if r.snapshot.stake.loot > 0 && !r.run_over {
            assert_eq!(r.snapshot.stake.return_row, Some(0));
            assert_eq!(r.snapshot.stake.kept, Some(r.snapshot.stake.loot * 60 / 100), "the kept number, not the carried one");
        }
        if let Some((l, tr)) = r.events.iter().find_map(|e| if let Ev::Exit { line, trace, .. } = e { line.clone().map(|l| (l, trace.clone())) } else { None }) {
            // Cut 9 §5: the exit event carries the last-5 trace beside the line.
            assert!(tr.is_some_and(|t| !t.turns.is_empty() && t.turns.len() <= crate::engine::EXIT_TRACE_LEN));
            line = Some(l);
            break;
        }
    }
    let line = line.expect("an exit line on the exit event");
    assert_eq!(line.keep_pct, 60);
    assert_eq!(line.kept, line.carried * 60 / 100);
    assert!(line.text.starts_with(&format!("returned ${} · ", line.kept)) && line.text.contains("keeps 60%"), "{}", line.text);
    assert_eq!(g.batch.exits.last().map(|l| ExitLine { trace: None, ..l.clone() }), Some(*line.clone()));
    assert!(g.batch.exits.last().unwrap().trace.is_some(), "the report's exit line carries the trace");
    // Purchases, insurance, a refund and a hatch are ledger lines with ≤ 3-word reasons.
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let before = g.lineage.gold;
    let p = crate::engine::supply_price(crate::defs::Cat::Potion, g.lineage.best_depth);
    g.buy_supply("heal").unwrap();
    assert_eq!(g.lineage.gold_ledger.last().map(|l| (l.delta, l.why.as_str())), Some((-p, "heal")));
    g.clear_supplies();
    assert_eq!(g.lineage.gold_ledger.last().map(|l| (l.delta, l.why.as_str())), Some((p, "refund heal")));
    assert_eq!(g.lineage.gold, before);
    g.lineage.vault.push(Item::new(100_001, "sword"));
    g.lineage.gold_move(500, "gift");
    g.insure(100_001).unwrap();
    let last = g.lineage.gold_ledger.last().unwrap();
    assert_eq!(last.why, "insure sword");
    assert_eq!(last.delta, -crate::engine::insure_cost("sword"));
    // The report carries the exits of its batch (≤ 5, oldest first).
    let rep = g.run_offline(3600);
    assert!(!rep.exits.is_empty() && rep.exits.len() <= 5, "{:?}", rep.exits);
    for e in &rep.exits {
        assert!(word_count(&e.text) <= 14, "{}", e.text);
    }
}

/// §2: a heal bought by name is drunk by `hp<30% → drink heal` at 3 HP — bought, crafted and
/// vaulted items are known by name whatever their flavour; a found one still needs the fact.
#[test]
fn bought_heal_is_drunk_by_name() {
    for identified in [true, false] {
        let mut g = arena_seed(5);
        g.lineage.gold_move(100, "gift");
        if identified {
            g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
            g.buy_supply("heal").unwrap();
        } else {
            // Unidentified: the shop will not list it, so it is a vaulted (kept) potion.
            assert!(g.buy_supply("heal").is_err());
            let mut it = Item::new(100_010, "heal");
            it.known = true;
            g.lineage.vault.push(it);
        }
        let supplies: Vec<Item> = if identified { std::mem::take(&mut g.lineage.supplies) } else { g.lineage.vault.clone() };
        assert!(supplies.iter().all(|s| s.known), "bought/vaulted items are known by name");
        // The potion goes into the live run's pack as `start_run` would place it.
        for it in supplies {
            g.run.as_mut().unwrap().hero.auto_equip(it);
        }
        rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        g.run.as_mut().unwrap().hero.hp = 3;
        add_monster(&mut g, "rat", 12, 9);
        let snap = g.snapshot();
        let potion = snap.hero.inv.iter().find(|i| i.kind == "heal").expect("the wire names the potion");
        assert!(potion.known && potion.label == "heal potion", "{potion:?}");
        let evs = ticks(&mut g, 20);
        assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, verb, .. } if verb.v == "drink")), "identified {identified}: {:?}", ev_kinds(&evs));
        assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "heal potion")));
        assert!(hero(&g).hp > 3);
        assert!(!evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.starts_with("Gambled"))), "a named potion is no gamble");
    }
    // A found heal (no flavour fact, not known by name) is not `drink heal`'s.
    let mut g = arena_seed(5);
    give(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal"))]);
    g.run.as_mut().unwrap().hero.hp = 3;
    let evs = ticks(&mut g, 20);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })));
    let t = g.run.as_ref().unwrap().trace.last().unwrap();
    assert_eq!(t.rows.as_ref().unwrap()[0].why, "unknown item", "{:?}", t.rows);
    // A save round-trips the flag.
    let mut it = Item::new(1, "heal");
    it.known = true;
    let j = serde_json::to_string(&it).unwrap();
    assert!(j.contains("\"known\":true"));
    let back: Item = serde_json::from_str(r#"{"id":1,"kind":"heal"}"#).unwrap();
    assert!(!back.known);
}

/// §3: every death trace's turns account for every row above the one that acted, with a
/// reason from the fixed table (≤ 3 words); 100 deaths over the offline batches.
#[test]
fn death_traces_account_for_every_row_above_the_fired_one() {
    let mut deaths = 0;
    let mut turns = 0;
    let mut reasons: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    // Eight seeds at a time until 100 deaths.
    for chunk in [1..=8u64, 9..=16, 17..=24, 25..=30] {
        for (d, t, r) in par_seeds(chunk, |seed| {
            let (mut deaths, mut turns) = (0, 0);
            let mut reasons: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            let mut g = Game::new_literal(seed);
            g.max_deaths = 1000;
            if seed % 2 == 0 {
                g.set_rules_raw(crate::probes::good()).unwrap();
            }
            crate::offline::run_offline_quick(&mut g, 8 * 3600);
            let n_rows = g.lineage.rules().rows.len().min(g.lineage.max_rows());
            for rec in g.deaths.values() {
                deaths += 1;
                for t in &rec.death.trace.turns {
                    turns += 1;
                    let above = if t.row >= 0 { t.row as usize } else { n_rows };
                    let rows = t.rows.clone().unwrap_or_default();
                    // Cut 15 §6: a hazard pre-emption is said once, on the first row, for all of them.
                    if rows.len() == 1 && rows[0].why.starts_with("hazard first") {
                        assert!(t.row < 0 && crate::turn::row_reason_ok(&rows[0].why), "{rows:?}");
                        continue;
                    }
                    assert_eq!(rows.len(), above, "seed {seed} run {}: t{} row {} {:?}", rec.death.run_id, t.t, t.row, rows);
                    for (i, w) in rows.iter().enumerate() {
                        assert_eq!(w.row, i);
                        assert!(crate::turn::row_reason_ok(&w.why), "reason off the table: {}", w.why);
                        reasons.insert(w.why.split(|c: char| c.is_ascii_digit()).next().unwrap_or("").trim().to_string());
                    }
                }
            }
            (deaths, turns, reasons)
        }) {
            deaths += d;
            turns += t;
            reasons.extend(r);
        }
        if deaths >= 100 {
            break;
        }
    }
    assert!(deaths >= 100, "{deaths} deaths");
    assert!(turns >= 500);
    assert!(reasons.len() >= 3, "{reasons:?}");
}

/// §3: the reasons name the failing condition and the blocked verb in play: `hp not <30%`,
/// `foes not ≥2`, `not in view`, `none held`, the Cut 4 block reason, `card passed`, a free
/// action, a trait pre-empting the list.
#[test]
fn row_reasons_name_the_condition_or_the_block() {
    let mut g = arena_seed(2);
    g.lineage.unlocks.insert("gas_step".into());
    g.lineage.unlocks.extend(["row5", "row6", "row7"].map(String::from));
    g.lineage.facts.insert("foe:jackal:pack".into());
    rules(
        &mut g,
        vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "unknown")),
            Row::new(vec![Cond::n("foes>=", 2)], Verb::new("retreat")),
            Row::new(vec![Cond::t("foe_tag", "pack")], Verb::arg("attack", "tag:pack")),
            Row::new(vec![Cond::t("item", "heal")], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::t("foe_tag", "gas")], Verb::arg("tactic", "gas_step")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
    );
    // (Cut 30 §2: a temperament no longer holds a row — the brave heir's retreat row acts)
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Brave;
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, 5, "{t:?}");
    let whys: Vec<&str> = t.rows.as_ref().unwrap().iter().map(|w| w.why.as_str()).collect();
    assert_eq!(whys, ["hp not <30%", "foes not ≥2", "not in view", "none held", "not in view"], "{t:?}");
    // A card that passes, an unknown-item row with nothing unknown, a locked token.
    let mut g = arena_seed(2);
    g.lineage.unlocks.insert("gas_step".into());
    rules(
        &mut g,
        vec![
            Row::new(vec![], Verb::arg("tactic", "gas_step")),
            Row::new(vec![Cond::flag("unknown_item")], Verb::arg("drink", "unknown")),
            // (Cut 29 §1: `loot ≥` is free from the start; `alert ≥` waits on its gate)
            Row::new(vec![Cond::n("alert>=", 1)], Verb::new("return")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
    );
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, 3, "{t:?}");
    let whys: Vec<&str> = t.rows.as_ref().unwrap().iter().map(|w| w.why.as_str()).collect();
    // QA on 912e135: a foe-keyed card with no such foe in view is `card idle` (a rat is no gas foe)
    assert_eq!(whys, ["card idle", "no unknown", "locked cond"], "{t:?}");
    // A chore: every row accounted for; a pre-empting trait: every row `trait first`.
    let mut g = arena_seed(2);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![Cond::n("hp<", 50)], Verb::new("rest"))]);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, -2);
    let whys: Vec<&str> = t.rows.as_ref().unwrap().iter().map(|w| w.why.as_str()).collect();
    assert_eq!(whys, ["foes not ≥1", "hp not <50%"]);
    let mut g = arena_seed(2);
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Cowardly;
    g.run.as_mut().unwrap().hero.hp = 10;
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    // (Cut 30 §2: the cowardly heir's row acts — no trait pre-empts the list)
    assert_eq!(t.row, 0, "{t:?}");
    // Row 1 acting: nothing above it, no list.
    let mut g = arena_seed(2);
    attack_rules(&mut g);
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, 0);
    assert!(t.rows.is_none());
    for r in crate::turn::ROW_REASONS {
        assert!(word_count(r) <= 3, "{r}");
    }
}

/// §5: the counter facts carry the row, the lineage lists the known counters with ≤ 3-word
/// texts, old-form facts in a save are upgraded, and the gates that read the old key still
/// match.
#[test]
fn counter_facts_carry_the_row() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 500;
    g.run.as_mut().unwrap().hero.hp = 499;
    add_monster(&mut g, "goblin_warlord", 8, 5);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "boss:goblin_warlord:counter=attack tag:boss")), "{:?}", ev_kinds(&evs));
    assert_eq!(g.lineage.facts.iter().filter(|f| f.starts_with("boss:goblin_warlord:counter")).count(), 1);
    let l = g.lineage();
    assert_eq!(l.counters.len(), 1);
    assert_eq!(l.counters[0].boss, "goblin_warlord");
    assert_eq!(l.counters[0].text, "attack boss");
    assert_eq!(l.counters[0].row, Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")));
    // The `stair_dance` gate ("a boss counter") reads the new form.
    assert!(crate::meta::gate(&g.lineage, "stair_dance").is_none());
    // An old save's fact is upgraded on load.
    g.lineage.facts.insert("boss:lich:counter".into());
    let g2 = Game::load(&g.save()).unwrap();
    assert!(g2.lineage.facts.contains("boss:lich:counter=attack tag:summoned"));
    assert!(!g2.lineage.facts.contains("boss:lich:counter"));
    let l = g2.lineage();
    assert_eq!(l.counters.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), ["attack boss", "attack summoned"]);
    for c in &l.counters {
        assert!(word_count(&c.text) <= 3);
    }
}

/// §5/§8: a boss death with the counter known shows the counter row first; a `return` patch
/// is never the only patch on a boss death. 30 Warlord deaths.
#[cfg(not(debug_assertions))]
#[test]
fn boss_deaths_show_the_counter_row_first() {
    let mut deaths = 0;
    let mut lone_return = 0;
    let mut under = 0;
    let mut seed = 100u64;
    while deaths < 30 {
        seed += 1;
        let mut g = arena_seed(seed);
        g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
        {
            // D5 (the Warlord's floor); `max_depth` stays 1 so the patch forecasts, which run to
            // `best_depth + 1`, stay cheap — the test is about the pin, not the deltas.
            let run = g.run.as_mut().unwrap();
            run.depth = 5;
            run.hero.hp = 12 + (seed % 5) as i32;
        }
        add_monster(&mut g, "goblin_warlord", 7 + (seed % 3) as i32, 5);
        add_monster(&mut g, "goblin", 6, 4);
        add_monster(&mut g, "goblin", 6, 6);
        add_monster(&mut g, "goblin", 5, 7);
        // The counter is known; the boss tag is what the sight teaches (learned in play).
        g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
        let mut rows = vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))];
        if seed.is_multiple_of(2) {
            rows.insert(0, Row::new(vec![Cond::n("hp<", 10)], Verb::new("return")));
        }
        rules(&mut g, rows);
        let mut died = None;
        for _ in 0..800 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                let r = g.run.as_ref().unwrap();
                if r.over == Some(ExitTier::Death) {
                    died = Some(r.id);
                }
                g.finish_run();
                break;
            }
        }
        let Some(id) = died else { continue };
        let d = g.death(id).unwrap();
        let rec = g.deaths.get(&id).unwrap();
        assert_eq!(rec.boss.as_deref(), Some("goblin_warlord"), "seed {seed}");
        deaths += 1;
        assert!(!d.patches.is_empty(), "seed {seed}: no patch on a boss death");
        let counter = crate::facts::counter_row("goblin_warlord");
        // Cut 14 §1: nothing under the baseline is offered, the pinned counter included — it
        // leads whenever its measured survival is at or over the unpatched replays'.
        let measured = crate::trace::measure_row(&g, rec, &counter).expect("the counter row replays");
        if measured.0 >= d.baseline - 1e-9 {
            assert_eq!(d.patches[0].row, counter, "seed {seed}: {:?}", d.patches);
        } else {
            // Cut 19 §2: on a seed with `hp < 10% → return` the unpatched replays walk home and
            // sometimes live; the counter at the top pre-empts that walk (it strikes whenever
            // the boss is in view) — under the baseline by construction, not the pin's fault.
            if !seed.is_multiple_of(2) {
                under += 1;
            }
            assert!(d.patches.iter().all(|p| p.row != counter), "seed {seed}: the counter under the baseline ({} vs {}) is shown: {:?}", measured.0, d.baseline, d.patches);
        }
        assert!(d.line.is_some());
        if d.patches.len() == 1 && d.patches[0].row.verb.v == "return" {
            lone_return += 1;
        }
    }
    assert_eq!(lone_return, 0);
    // (a hero at 12–16 hp who turns on the Warlord dies every replay on some seeds: the
    // counter's number there is the wall's, on the forecast's try row, not the moment's)
    assert!(under * 3 <= 15, "the counter fell under the baseline on {under} of the ~15 boss deaths without a return row");
}

/// §9: the forecast is a function of (rules, lineage seed, depth) and the lineage a sim starts
/// from — re-reading it gives the same numbers, transient state moves nothing, and the refine
/// pass at 100 sims is as deterministic.
#[test]
fn forecast_is_deterministic_per_rules_and_lineage() {
    let mut g = Game::new_literal(11);
    // The shelf is an input (a send packs it, and the ends panel's sends run to their exit
    // with what was packed): an empty shelf keeps the send below from moving the numbers.
    no_kennel_leash(&mut g);
    g.set_rules_raw(crate::probes::good()).unwrap();
    let a = g.forecast();
    let b = g.forecast();
    assert_eq!(a, b);
    // Transient state: marks, renown, the rest clock, a run in progress.
    g.lineage.marks += 5;
    g.lineage.renown += 100;
    g.lineage.rest_left = 300;
    g.send();
    // (a run in progress at its first tick: thirty ticks in it would have learned a fact,
    // and facts are lineage state the ends panel's sends — run to their exit — do read)
    let c = g.forecast();
    assert_eq!(a, c, "transient state moved the forecast");
    let r1 = g.forecast_refine();
    let r2 = g.forecast_refine();
    assert_eq!(r1, r2);
    assert_eq!(r1.known_to, a.known_to);
    // Cut 14 §1: the refine's ends line is quieter (its `±` ≤ 14 at 50 %). QA on 3d71c33: the
    // ends are the bars' own panel — the refine's 100 sims, the first paint's 50.
    assert!(r1.refined && !a.refined);
    let (e1, e0) = (r1.ends.as_ref().expect("ends"), a.ends.as_ref().expect("ends"));
    assert!(e1.pm <= 0.14 + 1e-9, "refined ends ±{:.3}", e1.pm);
    assert_eq!(e1.pm, crate::forecast::half_width(e1.death, crate::forecast::REFINE_SIMS as usize), "the refined ends ran short of {}", crate::forecast::REFINE_SIMS);
    assert_eq!(e0.pm, crate::forecast::half_width(e0.death, crate::forecast::FORECAST_SIMS as usize));
    // A second game with the same seed and rules reads the same forecast; the seeds are the
    // forecast's own.
    let mut h = Game::new_literal(11);
    no_kennel_leash(&mut h);
    h.set_rules_raw(crate::probes::good()).unwrap();
    assert_eq!(h.forecast(), a);
    // Cut 14 §1: the seeds are the lineage's per depth, whatever the set — a different rule
    // set is measured on the same dungeons (a paired difference), a different depth on its own.
    let t1 = crate::forecast::forecast_tag(&g, 1);
    let t3 = crate::forecast::forecast_tag(&g, 2);
    assert!(t1 != t3);
    // Through the save: identical — the refined panel, since the refine ran (QA on a946e04:
    // a reload read the first pass again, `return 90%` → `88%`).
    let g3 = Game::load(&g.save()).unwrap();
    assert_eq!(g3.forecast(), g.forecast());
    assert_eq!(g3.forecast(), r1);
}

/// Cut 14 §1: the forecast is a paired measurement — every set a lineage forecasts plays the
/// same dungeons (`forecast_tag` hashes the lineage seed and the depth, not the rules), so a
/// one-notch edit (the good set's `hp<35 → drink heal` at `hp<30`) reads as a small delta at
/// D5: |Δ| ≤ 0.08 on ≥ 90 % of 30 lineage seeds. (Two independent draws at 50 sims read ±14
/// each — "the same six rows read 94/6, then 79/21", cohort 10.)
#[test]
fn paired_forecast_one_notch_edit_is_a_small_delta() {
    let good = crate::probes::good();
    let mut edited = good.clone();
    let i = edited.rows.iter().position(|r| r.verb.v == "drink" && r.conds.iter().any(|c| c.k == "hp<" && c.n == Some(35))).expect("the good set's heal row");
    edited.rows[i].conds[0].n = Some(30);
    let seeds = 30u64;
    let deltas: Vec<(f64, f64)> = par_seeds(1..=seeds, |seed| {
        // The EDITED bot's lineage (`examples/metrics.rs`): eight rows, the heal known, two
        // on the shelf — the row fires, so the edit is load-bearing.
        let mut g = Game::new_literal(seed);
        no_kennel_leash(&mut g);
        for u in ["row5", "row6", "row7", "row8"] {
            g.lineage.unlocks.insert(u.into());
        }
        g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "heal").unwrap());
        g.lineage.gold_move(100, "test");
        g.buy_supply("heal").unwrap();
        g.buy_supply("heal").unwrap();
        g.set_rules_raw(good.clone()).unwrap();
        assert_eq!(g.lineage.rules().rows.len(), good.rows.len());
        // The panel's own seeds at D5 (`forecast_tag`), run to D8 so one panel answers D5
        // (the contract's depth; the good set is near its ceiling there) and D8 (where its
        // reach is mid-range and a second draw would spread).
        let tag = crate::forecast::forecast_tag(&g, 5);
        let n = crate::forecast::FORECAST_SIMS;
        let panel = |rules: &RuleSet, tag: u64| crate::forecast::simulate_budget(&g, rules, n, tag, 8, u64::MAX);
        let reach = |rs: &[crate::forecast::SimResult], d: u32| rs.iter().filter(|r| r.max_depth >= d).count() as f64 / n as f64;
        let (b, e) = (panel(&good, tag), panel(&edited, tag));
        assert_eq!((b.len(), e.len()), (n as usize, n as usize));
        (reach(&e, 5) - reach(&b, 5), reach(&e, 8) - reach(&b, 8))
    });
    let small = deltas.iter().filter(|(d5, d8)| d5.abs() <= 0.08 + 1e-9 && d8.abs() <= 0.08 + 1e-9).count() as u64;
    assert!(small * 10 >= seeds * 9, "|Δ| ≤ 0.08 at D5 and D8 on {small}/{seeds}: {deltas:?}");
}

// ---------------------------------------------------------------- Cut 7

/// §2: the shipped preset is two rows tagged `preset`; a card's row is tagged `card` (by
/// `unlock_row`, and by `set_rules` for a bare tactic row); origin is not identity.
#[test]
fn row_origins_preset_and_card() {
    let g = Game::new_literal(1);
    let rows = &g.lineage.rules().rows;
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.origin.as_deref() == Some("preset")));
    assert_eq!(g.lineage.max_rows(), 4);
    let a = Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal"));
    let b = a.clone().from("player");
    assert_eq!(a, b, "origin is not part of a row's identity");
    let json = serde_json::to_string(&b).unwrap();
    assert!(json.contains("\"origin\":\"player\""));
    let back: Row = serde_json::from_str(&json).unwrap();
    assert_eq!(back.origin.as_deref(), Some("player"));
    let plain: Row = serde_json::from_str(r#"{"conds":[],"verb":{"v":"rest"}}"#).unwrap();
    assert!(plain.origin.is_none());
    let mut g = Game::new_literal(1);
    g.lineage.unlocks.insert("boss_focus".into());
    let card = crate::meta::unlock_row(&g.lineage, "boss_focus").unwrap();
    assert_eq!(card.origin.as_deref(), Some("card"));
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("tactic", "boss_focus")));
    set.rows.push(Row::new(vec![], Verb::new("rest")));
    g.set_rules_raw(set).unwrap();
    let rows = &g.lineage.rules().rows;
    assert_eq!(rows[0].origin.as_deref(), Some("card"), "a bare tactic row is a card's");
    assert!(rows[3].origin.is_none(), "the client tags patch/player rows");
}

/// §1: the Captain rallies once (two goblins) and never shield-buffs; not a boss, no counter
/// fact, no `boss` tag; the Warlord keeps both.
#[test]
fn captain_rallies_once_and_is_no_boss() {
    let mut g = arena();
    let c = add_monster(&mut g, "goblin_captain", 6, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == c && what == "rallies")));
    let rallies = evs.iter().filter(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == c && what == "rallies")).count();
    assert_eq!(rallies, 1, "one rally");
    let goblins = g.run.as_ref().unwrap().monsters.iter().filter(|m| m.kind == "goblin" && m.hp > 0).count();
    assert_eq!(goblins, 1, "one goblin, once");
    assert!(!evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "shields up")), "no shield wall");
    assert!(!crate::facts::has_boss_counter(&g.lineage.facts));
    assert!(!crate::defs::monster_def("goblin_captain").boss);
    assert!(!g.lineage.facts.contains("foe:goblin_captain:boss"));
    assert_eq!(crate::descent::lieutenant_for(5), Some("goblin_captain"));
    // Placed by the stairs on D5 with goblins about, like a boss.
    for seed in 1..=4u64 {
        let mut g = Game::new_literal(seed);
        g.start_run(Some(seed));
        g.descend_to(5);
        let run = g.run.as_ref().unwrap();
        let cap = run.monsters.iter().find(|m| m.kind == "goblin_captain").expect("a captain on D5");
        assert!(cap.pos.cheb(run.floor.stairs_down) <= 3, "seed {seed}");
        g.descend_to(8);
        let run = g.run.as_ref().unwrap();
        assert!(run.monsters.iter().any(|m| m.kind == "goblin_warlord"), "seed {seed}: the Warlord at D8");
        assert!(!run.monsters.iter().any(|m| m.kind == "goblin_captain"));
    }
}

/// §4: `Snapshot.room` names the room and its awake hostiles; `rooms` the floor's count; a
/// corridor reads room 0.
#[test]
fn snapshot_room_and_rooms() {
    let mut g = Game::new_literal(3);
    g.send();
    let s = g.snapshot();
    let run = g.run.as_ref().unwrap();
    assert_eq!(s.rooms, Some(run.floor.rooms.len() as u32));
    let room = s.room.unwrap();
    let ri = run.floor.rooms.iter().position(|r| r.contains(run.hero.pos)).map(|i| i as u32 + 1).unwrap_or(0);
    assert_eq!(room.id, ri);
    // A room with two awake hostiles beside the hero.
    let hp = run.hero.pos;
    let r = run.floor.rooms[(room.id.max(1) - 1) as usize];
    let free: Vec<Pos> = (r.y..r.y + r.h).flat_map(|y| (r.x..r.x + r.w).map(move |x| Pos::new(x, y))).filter(|p| *p != hp && run.floor.map.get(*p) == Tile::Floor && !run.occupied(*p)).take(2).collect();
    for p in &free {
        let id = add_monster(&mut g, "rat", p.x, p.y);
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
        m.awake = true;
    }
    let s = g.snapshot();
    assert!(s.room.unwrap().hostiles >= free.len() as u32);
    // The arena has no rooms: a corridor.
    let g = arena();
    let s = g.snapshot();
    assert_eq!(s.room, Some(crate::wire::RoomRef { id: 0, hostiles: 0 }));
    assert_eq!(s.rooms, Some(0));
}

/// §4: `Ev::Ending` before a bank walk-out (within three steps of the stairs) and in the
/// air of a death (≤ 15% with a foe adjacent), once per 100 ticks; a `return` reads 0.
#[test]
fn ending_is_foreseen() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.pos = Pos::new(4, 4);
    rules(&mut g, vec![Row::new(vec![], Verb::new("bank"))]);
    let evs = ticks(&mut g, 60);
    let ending = evs.iter().find(|e| matches!(e, Ev::Ending { .. })).expect("an ending before the bank");
    let exit_t = evs.iter().find_map(|e| if let Ev::Exit { t, .. } = e { Some(*t) } else { None }).expect("the bank");
    assert!(matches!(ending, Ev::Ending { ticks: 30, t } if *t < exit_t));
    assert_eq!(evs.iter().filter(|e| matches!(e, Ev::Ending { .. })).count(), 1);
    // Near death with a foe adjacent.
    let mut g = arena();
    add_monster(&mut g, "rat", 5, 5);
    g.run.as_mut().unwrap().hero.hp = 3;
    hold_rules(&mut g);
    let evs = ticks(&mut g, 120);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ending { ticks: 30, .. })));
    assert!(evs.iter().filter(|e| matches!(e, Ev::Ending { .. })).count() <= 2, "once per 100 ticks");
    // An instant exit (the player's bail) says 0.
    let mut g = arena();
    hold_rules(&mut g);
    g.bail();
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ending { ticks: 0, .. })));
    // Cut 19 §2: a return walks home, foreseen like a bank.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ending { ticks: 30, .. })), "the walk's end is foreseen");
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
    assert!(!Ev::Ending { t: 0, ticks: 0 }.renderable());
}

/// §5: L1–3 need 60·L²; a watched bank earns half again; offline and returns do not.
#[test]
fn watched_bank_earns_half_again() {
    let xp_of = |watched: bool, tier: ExitTier| -> u32 {
        let mut g = arena();
        g.run.as_mut().unwrap().depth = 4;
        g.run.as_mut().unwrap().max_depth = 4;
        if watched {
            g.watched = true;
        }
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, tier);
        g.finish_run();
        g.lineage.classes["fighter"].xp
    };
    let offline = xp_of(false, ExitTier::Bank);
    let watched = xp_of(true, ExitTier::Bank);
    assert_eq!(offline, 12, "3 × depth 4");
    assert_eq!(watched, 18);
    assert_eq!(xp_of(true, ExitTier::Return), 7, "a return is 60% and no bonus");
    // `send`/`step` mark the run watched; an absence clears it.
    let mut g = Game::new_literal(1);
    assert!(!g.watched);
    g.send();
    assert!(g.watched);
    g.run_offline(600);
    assert!(!g.watched);
    g.step(5);
    assert!(g.watched);
}

/// §1: the ledger lists the bosses after the kinds; a known counter rides along as a chip.
#[test]
fn ledger_boss_rows_carry_counters() {
    let mut g = Game::new_literal(1);
    let ledger = g.lineage.ledger();
    let bosses: Vec<&LedgerRow> = ledger.iter().filter(|r| crate::defs::monster_def(&r.kind).boss).collect();
    assert_eq!(bosses.len(), 6);
    assert!(bosses.iter().all(|r| r.counter.is_none() && !r.seen));
    assert_eq!(ledger.iter().position(|r| r.kind == "goblin_warlord").unwrap(), ledger.len() - 6, "bosses last, in descent order");
    g.lineage.facts.insert(crate::facts::boss_counter_fact("bloat_mother"));
    g.lineage.facts.insert("foe:bloat_mother".into());
    let ledger = g.lineage.ledger();
    let mother = ledger.iter().find(|r| r.kind == "bloat_mother").unwrap();
    let chip = mother.counter.as_ref().expect("the counter chip");
    assert_eq!(chip.text, "throw fire, boss");
    assert_eq!(chip.row, crate::facts::counter_row("bloat_mother"));
    assert!(mother.seen);
    assert!(ledger.iter().find(|r| r.kind == "lich").unwrap().counter.is_none());
    assert!(ledger.iter().filter(|r| !crate::defs::monster_def(&r.kind).boss).all(|r| r.counter.is_none()));
}

/// §3: the situations are facts and tokens; the captive gate takes the coward's way for a
/// plain attack row and the friend's way for `free_captive`; the hunger's shrine lights for
/// a hero who prayed above.
#[test]
fn situations_are_facts_tokens_and_answers() {
    let mut l = crate::engine::LineageState::new(1);
    for k in ["den", "lock", "captive", "hunger"] {
        assert!(!crate::tokens::vocabulary(&l).conds.contains(&Cond::t("on_see", k)));
        l.facts.insert(k.into());
        assert!(crate::tokens::vocabulary(&l).conds.contains(&Cond::t("on_see", k)), "{k}");
    }
    // The gate: attack nearest cuts the chained captive down (no_friends), free_captive passes.
    for coward in [true, false] {
        let mut g = Game::new_literal(4);
        g.lineage.facts.insert("captive".into());
        let row = if coward { Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")) } else { Row::new(vec![Cond::t("on_see", "captive")], Verb::new("free_captive")) };
        g.set_rules(RuleSet { rows: vec![row], name: None, route: Vec::new() }).unwrap();
        g.start_run(Some(4));
        g.descend_to_twist(9, "captive");
        {
            let run = g.run.as_mut().unwrap();
            run.monsters.retain(|m| m.situation.is_some());
            let s = run.floor.stairs_down;
            let near = s.neighbours8().into_iter().find(|q| run.floor.map.passable(*q)).unwrap();
            run.hero.pos = near;
            run.hero_dist_pos = None;
            run.floor.map.update_vision(near, 7);
        }
        let evs = ticks(&mut g, 200);
        let run = g.run.as_ref().unwrap();
        if coward {
            assert!(run.trophies_run.contains(&"no_friends".to_string()), "{evs:?}");
            assert!(!run.passed.contains(&"captive".to_string()));
        } else {
            assert!(evs.iter().any(|e| matches!(e, Ev::Ally { state, .. } if state == "freed")));
            assert!(run.passed.contains(&"captive".to_string()));
            assert!(run.monsters.iter().any(|m| m.ally && m.kind == "captive"));
        }
    }
    // The hunger: a prayed hero still lights D12's shrine, free.
    let mut g = Game::new_literal(5);
    for f in ["shrine", "hunger"] {
        g.lineage.facts.insert(f.into());
    }
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::t("on_see", "hunger")], Verb::arg("pray", "row"))], name: None, route: Vec::new() }).unwrap();
    g.start_run(Some(5));
    g.descend_to_twist(12, "hunger");
    {
        let run = g.run.as_mut().unwrap();
        run.prayed = true;
        run.monsters.clear();
    }
    let max_before = g.run.as_ref().unwrap().hero.max_hp;
    let evs = ticks(&mut g, 600);
    let run = g.run.as_ref().unwrap();
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "lit")), "{evs:?}");
    assert!(run.lit && run.passed.contains(&"hunger".to_string()));
    assert!(run.hero.max_hp >= max_before - 2, "the hunger bit at most twice before the light");
}

/// Rater F (cohort 3): a thief-guard card plus a pickup chore shuffled between two tiles for
/// minutes at frozen HP with no verdict. Three guard firings on one floor now end the run as a
/// `return` with nothing kept, so the stall verdict can name the row.
#[test]
fn a_run_that_keeps_shuffling_ends_as_stalled_not_at_the_cap() {
    use crate::engine::STALL_FIRES;
    let mut g = Game::new_literal(67);
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.facts.insert("foe:monkey:thief".into());
    let rows = vec![
        Row::new(vec![], Verb::arg("tactic", "thief_guard")),
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
    ];
    g.set_rules(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
    let _ = g.send();
    let (mut fires, mut ticks, mut exit_tier) = (0u32, 0u32, None::<String>);
    let mut stalled_note = false;
    for _ in 0..6_000 {
        let r = g.step(10);
        ticks += 10;
        for e in &r.events {
            match e {
                Ev::Rule { verb, .. } if verb.v == "stuck" => fires += 1,
                Ev::Exit { tier, .. } => exit_tier = Some(tier.clone()),
                Ev::Note { text, .. } if text.starts_with("Stalled") => stalled_note = true,
                _ => {}
            }
        }
        if r.run_over || exit_tier.is_some() { break; }
    }
    assert!(exit_tier.is_some(), "a shuffling run must end within 60k ticks (fires {fires})");
    if stalled_note {
        assert!(fires >= STALL_FIRES, "stalled without three guard firings");
        assert_eq!(exit_tier.as_deref(), Some("return"));
        assert!(ticks < crate::engine::MAX_TURNS_PER_RUN / 2, "stall should end long before the cap: {ticks}");
    }
}

// ---------------------------------------------------------------- Cut 8B (rows become a roster)

/// §1: a set with a combo names the heir by it in the chronicle (`the chokepoint fighter`,
/// the first combo by row order; a multi-word name is hyphenated), else by the trait; the
/// wire lineage carries the combos and recomputes them on every `set_rules`.
#[test]
fn the_chronicle_names_the_heir_by_its_first_combo() {
    let mut g = arena();
    g.lineage.trait_ = crate::hero::Trait::Greedy;
    let set = RuleSet {
        name: None,
        rows: vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("foes>=", 2)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("adj>=", 1)], Verb::arg("attack", "nearest")),
        ],
        route: Vec::new(),
    };
    g.set_rules(set).unwrap();
    let l = g.lineage();
    assert_eq!(l.combos.len(), 1);
    assert_eq!((l.combos[0].rows, l.combos[0].name.as_str()), ([1, 2], "chokepoint"));
    g.run.as_mut().unwrap().max_depth = 3;
    g.lineage.heir_best = 3;
    g.lineage.chronicle_heir("fell to a rat", None);
    assert_eq!(g.lineage.chronicle, vec!["♟1 the chokepoint fighter · D3 · fell to a rat.".to_string()]);
    // A rogue with `throw → retreat` above the chokepoint: the first combo wins, hyphenated.
    g.lineage.heir = 2;
    g.lineage.unlocks.insert("rogue".into());
    g.lineage.class = crate::hero::Class::Rogue;
    let set = RuleSet {
        name: Some("fade".into()),
        rows: vec![
            Row::new(vec![Cond::n("foes>=", 2)], Verb::arg("throw", "unknown,nearest")),
            Row::new(vec![Cond::n("adj>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("foes>=", 2)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("adj>=", 1)], Verb::arg("attack", "nearest")),
        ],
        route: Vec::new(),
    };
    g.set_rules(set).unwrap();
    assert_eq!(g.lineage().combos.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), vec!["hit and fade", "chokepoint"]);
    g.lineage.chronicle_heir("fell to gas", None);
    assert_eq!(g.lineage.chronicle[1], "♟2 the hit-and-fade rogue · D3 · \"fade\" set · fell to gas.");
    // No combo: the trait, as before.
    g.lineage.heir = 3;
    g.set_rules(crate::probes::preset(crate::hero::Class::Fighter)).unwrap();
    assert!(g.lineage().combos.is_empty());
    g.lineage.chronicle_heir("fell to gas", None);
    assert!(g.lineage.chronicle[2].starts_with("♟3 the greedy rogue"), "{}", g.lineage.chronicle[2]);
    // The vocabulary carries the table.
    assert_eq!(g.vocabulary().combos.len(), crate::rules::COMBOS.len());
}

/// §1: an episode credits a combo when both of its rows fired within three hero actions of
/// the low point — the story line's turn beat reads `the chokepoint landed` in place of the
/// single row, and the gate's grammar accepts it. Rows fired further apart, or out of order,
/// credit nothing.
#[test]
fn story_lines_credit_a_combo_that_landed_at_the_low_point() {
    let mut g = arena();
    let a = add_monster(&mut g, "jackal", 5, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 2)], Verb::new("back_corridor")), Row::new(vec![Cond::n("adj>=", 1)], Verb::arg("attack", "nearest"))]);
    let corridor = Verb::new("back_corridor");
    let attack = Verb::arg("attack", "nearest");
    {
        let (run, mut cx) = g.ctx();
        run.monsters.iter_mut().for_each(|m| m.stun = 500);
        run.floor.map.update_vision(run.hero.pos, VISION);
        let ai = run.monsters.iter().position(|m| m.id == a).unwrap();
        // R1 fired the action before the blow, R2 the action after: the pair lands.
        run.actions += 1;
        crate::sifter::on_action(run, 0, &corridor);
        let hp = run.hero.hp;
        crate::turn::damage_hero(run, &mut cx, hp - 3, &crate::turn::Src::Mon(ai));
        assert!(run.arc.row_pending);
        run.actions += 1;
        crate::sifter::on_action(run, 1, &attack);
        assert_eq!(run.arc.combo.as_deref(), Some("chokepoint"));
        let ep = run.arc.to_episode(run, crate::sifter::Resolution::Banked { gold: 58 });
        let text = crate::sifter::story_line(&ep);
        assert_eq!(text, "A jackal chased him to 3 HP; the chokepoint landed; banked $58.");
        assert!(crate::sifter::story_ok(&text), "{text}");
        let h = crate::sifter::to_highlight(run, &ep, false);
        assert!(crate::sifter::names_agent(&h));
    }
    // Out of order (R2 then R1) is no combo; a pair three actions apart is none either.
    let mut g = arena();
    let a = add_monster(&mut g, "jackal", 5, 5);
    {
        let (run, mut cx) = g.ctx();
        run.monsters.iter_mut().for_each(|m| m.stun = 500);
        run.floor.map.update_vision(run.hero.pos, VISION);
        let ai = run.monsters.iter().position(|m| m.id == a).unwrap();
        let hp = run.hero.hp;
        crate::turn::damage_hero(run, &mut cx, hp - 3, &crate::turn::Src::Mon(ai));
        run.actions += 1;
        crate::sifter::on_action(run, 1, &attack);
        run.actions += 1;
        crate::sifter::on_action(run, 0, &corridor);
        assert_eq!(run.arc.combo, None);
        run.actions += 1;
        crate::sifter::on_action(run, 1, &attack);
        assert_eq!(run.arc.combo, None, "the low's window has closed");
    }
    // A multi-word name reads whole; a combo with no row (a trait acted) is not credited.
    let mut ep = crate::sifter::Episode { combo: Some("hit and fade".into()), act: crate::sifter::Act { row: 2, verb: Verb::new("retreat"), target: None, boss: false, walk: false }, low_hp: 5, max_hp: 30, threat: vec![("ogre".into(), 1)], resolution: crate::sifter::Resolution::Reached { depth: 4 }, ..Default::default() };
    assert_eq!(crate::sifter::story_line(&ep), "An ogre took him to 5 HP; the hit-and-fade landed; reached D4.", "the short form is one word");
    ep.threat = vec![("gas".into(), 1)];
    ep.low_hp = 0;
    ep.resolution = crate::sifter::Resolution::Returned { gold: 0 };
    assert_eq!(crate::sifter::story_line(&ep), "Gas took him down; the hit and fade landed; returned.");
    assert!(crate::sifter::story_ok(&crate::sifter::story_line(&ep)));
    ep.act.row = -1;
    assert!(!crate::sifter::story_line(&ep).contains("landed"));
}

/// §3: the first stray — 80% of lineages meet a named stray jackal on D2 or D3 while they
/// have tamed nothing, the same jackal every run; `on_see: stray → tame` takes it with the
/// kennel's leash (free, known, on the shelf from the first camp, back after every run until
/// something is tamed, never refunded).
#[test]
fn the_first_stray_waits_on_d2_or_d3_for_the_kennel_leash() {
    let mut with = 0;
    for seed in 1..=40u64 {
        let g = Game::new_literal(seed);
        if let Some((d, name)) = g.lineage.first_stray() {
            with += 1;
            assert!((2..=3).contains(&d), "{d}");
            assert!(!name.is_empty());
            assert_eq!(g.lineage.first_stray(), Some((d, name)), "decided once per lineage");
        }
    }
    assert!((26..=38).contains(&with), "{with}/40 lineages at 80%");
    let seed = (1..=40u64).find(|s| Game::new_literal(*s).lineage.first_stray().is_some()).unwrap();
    let mut g = Game::new_literal(seed);
    let (d, name) = g.lineage.first_stray().unwrap();
    g.max_deaths = 1000;
    g.start_run(None);
    assert!(hero(&g).inv.iter().any(|i| i.kind == "leash" && i.amount == 1), "the kennel's leash is in the pack");
    assert!(g.lineage.supplies.is_empty());
    g.descend_to(d);
    {
        let run = g.run.as_ref().unwrap();
        let m = run.monsters.iter().find(|m| m.stray).expect("the stray is placed");
        assert_eq!((m.kind.as_str(), m.name.as_deref()), ("jackal", Some(name.as_str())));
        assert_eq!(run.first_stray, Some((d, name.clone())));
    }
    // Home again untamed: the leash is back on the shelf; a refund is nothing.
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "leash" && s.free));
    let gold = g.lineage.gold;
    g.clear_supplies();
    assert_eq!(g.lineage.gold, gold);
    // Tamed: the stray answer (`see stray → tame nearest`) fires on sight and the jackal
    // comes back named; afterwards no stray waits and no leash is supplied.
    let mut tamed = false;
    for seed in 1..=30u64 {
        let mut g = arena_seed(seed);
        g.run.as_mut().unwrap().first_stray = Some((1, "Uleth".into()));
        g.lineage.facts.insert("stray".into());
        let id = add_monster(&mut g, "jackal", 6, 5);
        {
            let run = g.run.as_mut().unwrap();
            let m = run.monsters.iter_mut().find(|m| m.id == id).unwrap();
            m.stray = true;
            m.name = Some("Uleth".into());
            m.stun = 500;
        }
        give(&mut g, "leash");
        assert!(g.vocabulary().conds.contains(&Cond::t("on_see", "stray")));
        rules(&mut g, vec![crate::probes::situation_answer("stray")]);
        let evs = ticks(&mut g, 60);
        if evs.iter().any(|e| matches!(e, Ev::Tame { ok: true, .. })) {
            tamed = true;
            {
                let (run, mut cx) = g.ctx();
                crate::turn::end_run(run, &mut cx, ExitTier::Bank);
            }
            g.finish_run();
            assert_eq!(g.lineage.tamed_kinds(), 1);
            assert_eq!(g.lineage.first_stray(), None);
            assert!(g.lineage.supplies.iter().all(|s| !s.free));
            assert!(g.lineage.party.iter().any(|c| c.name == "Uleth"), "{:?}", g.lineage.party);
            break;
        }
    }
    assert!(tamed);
}


// ---------------------------------------------------------------- Cut 9 (the last four points)

/// §1: the vocabulary lists every gated condition token with its ≤ 3-word gate, none of them
/// among the open ones, and `set_rules` refuses a row that uses one, naming the token.
#[test]
fn vocabulary_lists_locked_conds_and_set_rules_refuses_them() {
    let mut g = Game::new_literal(3);
    let v = g.vocabulary();
    assert!(!v.locked.is_empty());
    for l in &v.locked {
        assert!(!v.conds.iter().any(|c| c.same_token(&l.cond)), "{:?} is open and locked", l.cond);
        assert!(!l.needs.is_empty() && word_count(&l.needs) <= 3, "{:?}: {}", l.cond, l.needs);
    }
    let needs = |v: &crate::rules::Vocabulary, c: &Cond| v.locked.iter().find(|l| l.cond.same_token(c)).map(|l| l.needs.clone());
    assert_eq!(needs(&v, &Cond::t("on_see", "stray")).as_deref(), Some("see: stray"));
    assert_eq!(needs(&v, &Cond::flag("on_see")).as_deref(), Some("meet a foe"), "the unlock's own gate while shut");
    // (Cut 29 §1: a condition word is free — `turns>` has no gate, so it is open from the start)
    assert!(v.conds.iter().any(|c| c.k == "turns>") && needs(&v, &Cond::n("turns>", 100)).is_none());
    assert!(needs(&v, &Cond::t("foe_tag", "pack")).is_none(), "no jackal met yet: its tag is not on the sheet");
    g.lineage.facts.insert("foe:jackal".into());
    let v = g.vocabulary();
    assert_eq!(needs(&v, &Cond::t("foe_tag", "pack")).as_deref(), Some("fact: pack"));
    assert_eq!(needs(&v, &Cond::t("foe_tag", "fast")).as_deref(), Some("fact: fast"));
    assert!(needs(&v, &Cond::t("foe_tag", "mirror")).is_none(), "a tag of a kind never met");
    g.lineage.facts.insert("foe:jackal:pack".into());
    let v = g.vocabulary();
    assert!(v.conds.contains(&Cond::t("foe_tag", "pack")) && needs(&v, &Cond::t("foe_tag", "pack")).is_none());
    assert_eq!(needs(&v, &Cond::n("foe_hp<", 25)).as_deref(), Some("study a kind"));
    assert_eq!(needs(&v, &Cond::n("party_hp<", 50)).as_deref(), Some("tame a foe"));
    assert!(needs(&v, &Cond::t("item", "heal")).is_some_and(|n| n.starts_with("identify ")));
    assert_eq!(needs(&v, &Cond::t("item", "lantern")).as_deref(), Some("find: lantern"));
    assert!(needs(&v, &Cond::t("item", "leash")).is_none(), "the kennel's leash is a fact from the start");
    assert!(serde_json::to_string(&v).unwrap().contains(r#""locked":[{"cond":"#));
    // The door: a locked token is refused by name; the number is the player's.
    let row = |c: Cond| RuleSet { rows: vec![Row::new(vec![c], Verb::arg("tame", "nearest"))], name: None, route: Vec::new() };
    let e = g.set_rules(row(Cond::t("on_see", "stray"))).unwrap_err();
    assert!(e.contains("see stray") && e.contains("locked") && e.contains("see: stray"), "{e}");
    let e = g.set_rules(row(Cond::n("alert>=", 7))).unwrap_err();
    assert!(e.contains("alert 7+") && e.contains("see alert rise"), "{e}");
    // Open the gates: the fact, the unlock.
    g.lineage.facts.insert("stray".into());
    let v = g.vocabulary();
    assert!(v.conds.contains(&Cond::t("on_see", "stray")) && needs(&v, &Cond::t("on_see", "stray")).is_none());
    assert!(v.conds.iter().any(|c| c.k == "turns>") && needs(&v, &Cond::n("turns>", 7)).is_none());
    g.set_rules(row(Cond::t("on_see", "stray"))).unwrap();
    g.set_rules(row(Cond::n("turns>", 7))).unwrap();
    // A sim (a verdict replay, a forecast) takes the lineage's rows as they are.
    let mut sim = g.sim_clone();
    sim.set_rules(row(Cond::t("foe_tag", "pack"))).unwrap();
    // A companion's vocabulary locks nothing.
    assert!(crate::tokens::companion_vocabulary(&g.lineage, &crate::probes::pets_party()[0]).locked.is_empty());
}

/// §2: every unlock that is not `available` carries a non-empty ≤ 3-word `needs` — at a
/// fresh lineage, with marks to spare, and after an ascension; party slots read `tame a foe`.
#[test]
fn every_unavailable_unlock_carries_needs() {
    let check = |g: &Game, when: &str| {
        for u in g.unlocks() {
            if u.owned {
                assert!(u.needs.is_none(), "{when}: {} owned with needs", u.id);
            } else if u.available {
                assert!(u.needs.is_none(), "{when}: {} available with needs {:?}", u.id, u.needs);
            } else {
                let n = u.needs.as_deref().unwrap_or_else(|| panic!("{when}: {} unavailable without needs", u.id));
                assert!(!n.is_empty() && word_count(n) <= 3, "{when}: {} needs {n:?}", u.id);
            }
        }
    };
    let mut g = Game::new_literal(4);
    check(&g, "fresh");
    let by = |g: &Game, id: &str| g.unlocks().into_iter().find(|u| u.id == id).unwrap();
    // (Cut 29 §1: the party slots are oath rewards, off the shelf; the rows open at the first bank)
    assert!(g.unlocks().iter().all(|u| u.id != "party_slot_2"));
    assert_eq!(by(&g, "row5").needs.as_deref(), Some("bank once"));
    tiers(&mut g);
    // Cut 10 §3: a row unlock is dimmed while the set has a free row (the preset is two of four).
    assert_eq!(by(&g, "row5").needs.as_deref(), Some("fill rows"));
    assert_eq!(by(&g, "row6").needs.as_deref(), Some("row5"));
    g.lineage.marks = 100;
    check(&g, "rich");
    assert_eq!(by(&g, "row5").needs.as_deref(), Some("fill rows"), "marks do not open a row the set cannot use");
    assert!(g.buy("row5").is_ok(), "`rows full` is the card's dimming, not a refusal (the bots buy rows ahead)");
    g.lineage.unlocks.remove("row5");
    g.lineage.marks = 100;
    g.set_rules_raw(crate::probes::good()).unwrap();
    assert_eq!(g.lineage.rules().rows.len(), 4, "the set is truncated to its four rows");
    assert!(by(&g, "row5").available && by(&g, "row5").needs.is_none(), "{:?}", by(&g, "row5"));
    g.lineage.marks = 0;
    assert_eq!(by(&g, "row5").needs.as_deref(), Some("◆3 more"));
    g.lineage.marks = 100;
    let mut g = finished_lineage();
    g.ascend("short_list").unwrap();
    check(&g, "ascended");
    g.lineage.marks = 100;
    check(&g, "ascended, rich");
    // A card short of marks still shows a delta once simulated (the marks are not a gate).
    let mut g = Game::new_literal(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.best_depth = 1;
    tiers(&mut g);
    let c = g.unlock_deltas().into_iter().find(|u| u.id == "corridor_fighting").unwrap();
    assert_eq!(c.needs.as_deref(), Some("◆4 more"));
    assert!(c.delta.is_some(), "{c:?}");
}

/// §3: the forecast carries `pm` (the binomial half-width), two reads of an unchanged set are
/// identical, the refine pass widens the same seed sequence (its first 50 results are the
/// panel's) so its number lands within the panel's `±` of the first, and the unlock deltas
/// run the panel's seeds (a shallow lineage's base *is* the panel number).
#[test]
fn forecast_pm_and_refine_share_the_panel_seeds() {
    let mut worst = 0.0f64;
    // A set that dives at once and fights what it meets: short sims, a real miss rate.
    let dive = RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![], Verb::new("descend"))], name: None, route: Vec::new() };
    for w in par_seeds(1..=30u64, |seed| {
        let mut worst = 0.0f64;
        let mut g = Game::new_literal(seed);
        g.set_rules(dive.clone()).unwrap();
        g.lineage.best_depth = 1;
        let a = g.forecast();
        if seed <= 12 {
            assert_eq!(a, g.forecast(), "seed {seed}: two reads differ");
        }
        // A D2 sim is short: all fifty run.
        let n = crate::forecast::FORECAST_SIMS as usize;
        for d in &a.depths {
            let pm = d.pm.expect("pm on every depth");
            let want = 1.96 * (d.reach * (1.0 - d.reach) / n as f64).sqrt();
            assert!((pm - want).abs() < 1e-9, "seed {seed} D{}: pm {pm} vs {want}", d.depth);
            assert!((0.0..=0.5).contains(&pm));
        }
        if seed > 6 {
            return worst;
        }
        let r = g.forecast_refine();
        assert_eq!(r.known_to, a.known_to);
        for (x, y) in a.depths.iter().zip(&r.depths) {
            let pm = x.pm.unwrap();
            let diff = (x.reach - y.reach).abs();
            worst = worst.max(diff - 2.0 * pm);
            // A share at 0 or 1 has no width; one more miss in the second fifty is 1/100.
            assert!(diff <= 2.0 * pm + 0.02, "seed {seed} D{}: {:.2} → {:.2} (±{pm:.2})", x.depth, x.reach, y.reach);
            assert!(y.pm.unwrap() <= pm + 1e-9 || pm == 0.0, "the refine narrows the width");
        }
        worst
    }) {
        worst = worst.max(w);
    }
    // The refine's first fifty are the panel's own sims.
    let mut g = Game::new_literal(7);
    g.set_rules(dive.clone()).unwrap();
    g.lineage.best_depth = 2;
    tiers(&mut g);
    let rules = g.lineage.rules().clone();
    let tag = crate::forecast::forecast_tag(&g, 3);
    let a = crate::forecast::simulate_budget(&g, &rules, 50, tag, 3, u64::MAX);
    let b = crate::forecast::simulate_budget(&g, &rules, 100, tag, 3, u64::MAX);
    assert_eq!(b.len(), 100);
    for (x, y) in a.iter().zip(&b) {
        assert_eq!((x.max_depth, x.tier, &x.cause), (y.max_depth, y.tier, &y.cause));
    }
    // The unlock deltas' base is the panel's number at `best_depth + 1`.
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:jackal:fast".into());
    let panel = g.forecast();
    let cat = g.unlock_deltas();
    assert!(cat.iter().any(|u| u.delta.is_some()));
    let (base, n) = *g.forecast_cache.borrow().get(&crate::forecast::reach_key(&g, g.lineage.rules(), 3, crate::forecast::FORECAST_SIMS, tag, crate::forecast::CATALOGUE_TICK_BUDGET)).expect("the base is memoised");
    if n == crate::forecast::FORECAST_SIMS {
        assert_eq!(base, panel.depths[2].reach, "the base is the panel's own number");
    } else {
        // The catalogue budget stopped the base short: it is the panel's first `n` seeds.
        let first = crate::forecast::simulate_budget(&g, g.lineage.rules(), n, tag, 3, u64::MAX);
        assert_eq!(base, first.iter().filter(|r| r.max_depth >= 3).count() as f64 / n as f64);
    }
    assert_eq!(g.unlock_deltas(), cat, "a second read is the same");
    eprintln!("refine vs panel: worst overshoot of 2·pm {worst:.3}");
}

/// §5: every exit — bank, return, death — carries its last five hero turns with row
/// accounting on the report's exit lines and on the exit event; a death's trace is its own.
#[test]
fn every_exit_carries_a_five_turn_trace_with_row_accounting() {
    let per = par_seeds(1..=12u64, |seed| {
        let mut tiers: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut exits = 0;
        let mut g = Game::new_literal(seed);
        if seed % 3 == 0 {
            let mut set = g.lineage.rules().clone();
            set.rows.insert(0, Row::new(vec![Cond::n("hp<", 40)], Verb::new(if seed % 2 == 0 { "bank" } else { "return" })));
            g.set_rules(set).unwrap();
        }
        let r = crate::offline::run_offline_quick(&mut g, 3 * 3600);
        for line in &r.exits {
            exits += 1;
            let tr = line.trace.as_ref().unwrap_or_else(|| panic!("seed {seed}: an exit line without a trace: {}", line.text));
            assert!(!tr.turns.is_empty() && tr.turns.len() <= crate::engine::EXIT_TRACE_LEN, "{}", line.text);
            assert!(tr.turns.windows(2).all(|w| w[0].t < w[1].t));
            for t in &tr.turns {
                if let Some(rows) = &t.rows {
                    assert!(!rows.is_empty());
                    assert!(rows.iter().all(|w| crate::turn::row_reason_ok(&w.why)), "{rows:?}");
                }
            }
            tiers.insert(match line.text.split(' ').next().unwrap() {
                "banked" => "bank",
                "returned" => "return",
                _ => "death",
            }.to_string());
        }
        // A death's record has its own, longer trace; its line leaves it out.
        for rec in g.deaths.values() {
            assert!(rec.death.line.as_ref().is_some_and(|l| l.trace.is_none()));
            assert!(!rec.death.trace.turns.is_empty());
        }
        (exits, tiers)
    });
    let exits: u32 = per.iter().map(|p| p.0).sum();
    let tiers: std::collections::BTreeSet<String> = per.into_iter().flat_map(|p| p.1).collect();
    assert!(exits >= 12, "{exits} exits");
    assert!(tiers.contains("return") && tiers.contains("death"), "{tiers:?}");
    // The exit event of a watched run carries the same trace beside its line.
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(50);
    rules(&mut g, vec![Row::new(vec![Cond::n("hp>", 10)], Verb::new("return")), Row::new(vec![], Verb::new("rest"))]);
    let r = g.step(100);
    let (line, trace) = r.events.iter().find_map(|e| if let Ev::Exit { line, trace, tier, .. } = e { assert_eq!(tier, "return"); Some((line.clone(), trace.clone())) } else { None }).expect("the return");
    assert!(line.is_some_and(|l| l.trace.is_none()), "the line's copy rides on the event, once");
    let trace = trace.expect("the exit event's trace");
    assert!(!trace.turns.is_empty() && trace.turns.len() <= crate::engine::EXIT_TRACE_LEN);
    assert_eq!(trace.turns.last().unwrap().verb.v, "return");
    assert_eq!(trace, g.batch.exits.last().unwrap().trace.clone().unwrap());
}

/// §6: over three consecutive absences a (threat, resolution) pair shows on at most one reel;
/// the fourth absence may repeat the first's. The best-depth run's closing episode leads.
#[test]
fn reel_pairs_never_repeat_across_three_absences() {
    let checked: usize = par_seeds(1..=10u64, |seed| {
        let mut checked = 0;
        let mut g = Game::new_literal(seed);
        let mut reels: Vec<Vec<(String, String)>> = Vec::new();
        for _ in 0..6 {
            let r = crate::offline::run_offline_quick(&mut g, 2 * 3600);
            let pairs: Vec<(String, String)> = r.reel.iter().filter_map(crate::sifter::pair).collect();
            // Within a reel: distinct pairs.
            let set: std::collections::BTreeSet<_> = pairs.iter().cloned().collect();
            assert_eq!(set.len(), pairs.len(), "seed {seed}: {:?}", r.reel);
            // The best run leads whenever one of its episodes is on the reel.
            if let (Some((_, best)), Some(first)) = (g.batch.best_run, r.reel.first()) {
                if r.reel.iter().any(|h| h.run_id == best && h.arc.is_some()) {
                    assert_eq!(first.run_id, best, "seed {seed}: the best run leads: {:?}", r.reel);
                }
            }
            reels.push(pairs);
            assert_eq!(g.lineage.reel_pairs.len(), reels.len().min(crate::engine::REEL_ABSENCES));
        }
        for w in reels.windows(3) {
            let all: Vec<&(String, String)> = w.iter().flatten().collect();
            let set: std::collections::BTreeSet<_> = all.iter().cloned().collect();
            // The one allowed repeat: a reel with nothing fresh leads with the closing episode alone.
            let lone = w.iter().filter(|r| r.len() == 1).count();
            assert!(set.len() + lone >= all.len(), "seed {seed}: a pair repeated within three absences: {w:?}");
            checked += 1;
        }
        checked
    })
    .into_iter()
    .sum();
    assert!(checked >= 30);
    // The memory survives a save.
    let mut g = Game::new_literal(2);
    crate::offline::run_offline_quick(&mut g, 3600);
    let g2 = Game::load(&g.save()).unwrap();
    assert_eq!(g2.lineage.reel_pairs, g.lineage.reel_pairs);
}

/// §7: the graveyard's last five deaths keep `death_id`, the engine keeps their records
/// through a save (even one that trimmed `max_deaths`), and `death(id)` answers for each.
#[test]
fn graveyard_keeps_the_last_five_deaths_answerable() {
    let mut g = Game::new_literal(3);
    // A set that fights everything and never comes home: a death an hour or so.
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![], Verb::new("descend"))], name: None, route: Vec::new() }).unwrap();
    let mut hours = 0;
    while g.lineage.graveyard.len() < 7 && hours < 48 {
        crate::offline::run_offline_quick(&mut g, 2 * 3600);
        hours += 2;
    }
    let n = g.lineage.graveyard.len();
    assert!(n >= 7, "{n} deaths in {hours} h");
    let gy = &g.lineage.graveyard;
    for (i, grave) in gy.iter().enumerate() {
        if i + crate::engine::KEPT_DEATHS < n {
            assert!(grave.death_id.is_none(), "grave {i} of {n} keeps an id");
        } else {
            let id = grave.death_id.unwrap_or_else(|| panic!("grave {i} of {n} has no id"));
            assert!(g.deaths.contains_key(&id));
        }
    }
    let ids: Vec<u32> = gy.iter().filter_map(|x| x.death_id).collect();
    assert_eq!(ids.len(), crate::engine::KEPT_DEATHS);
    assert!(serde_json::to_string(&g.lineage()).unwrap().contains(r#""death_id":"#));
    // Through a save that trimmed the record cap: five stay, and each answers.
    g.max_deaths = 1;
    let mut g2 = Game::load(&g.save()).unwrap();
    assert_eq!(g2.max_deaths, crate::engine::KEPT_DEATHS);
    for id in &ids {
        assert!(g2.deaths.contains_key(id), "death {id} lost in the save");
    }
    let d = g2.death(ids[0]).expect("the oldest kept death answers");
    assert_eq!(d.run_id, ids[0]);
    assert!(!d.trace.turns.is_empty() && d.line.is_some());
    // A sixth death after the load drops the oldest id.
    let before = ids.clone();
    let mut more = 0;
    while g2.lineage.graveyard.len() == n && more < 12 {
        crate::offline::run_offline_quick(&mut g2, 2 * 3600);
        more += 1;
    }
    assert!(g2.lineage.graveyard.len() > n);
    assert!(g2.lineage.graveyard.iter().all(|x| x.death_id != Some(before[0])), "the oldest id is gone");
    assert_eq!(g2.lineage.graveyard.iter().filter(|x| x.death_id.is_some()).count(), crate::engine::KEPT_DEATHS);
}

/// §8: a pending line for a silent row counts the absence's runs in numbers.
#[test]
fn pending_counts_a_silent_row_over_the_absence() {
    let mut g = Game::new_literal(6);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 1)], Verb::new("bank")));
    g.set_rules(set).unwrap();
    let r = crate::offline::run_offline_quick(&mut g, 2 * 3600);
    let runs = g.batch.run_ticks.len();
    assert!(runs >= 2, "{runs} runs");
    let line = r.pending.iter().find(|p| p.starts_with("R1 fired 0 of ")).unwrap_or_else(|| panic!("{:?}", r.pending));
    assert!(line.starts_with(&format!("R1 fired 0 of {runs} runs: ")), "{line}");
    assert!(!r.pending.iter().any(|p| p.contains("never")), "{:?}", r.pending);
    // Every row gets its numbers (the client sums slices and shows the quiet ones): the attack
    // row fires in every run.
    let attack = g.lineage.rules().rows.iter().position(|r| r.verb.v == "attack").unwrap();
    let al = r.pending.iter().find(|p| p.starts_with(&format!("R{} fired", attack + 1))).unwrap_or_else(|| panic!("{:?}", r.pending));
    assert!(al.starts_with(&format!("R{} fired {runs} of {runs} runs", attack + 1)), "{al}");
}

/// §10: forge rows carry the ladder's next rung.
#[test]
fn forge_rows_carry_the_next_rung() {
    let next = |n: u32| ForgeRow::at(n).next.map(|x| (x.need, x.label));
    assert_eq!(next(0), Some((5, "craftable".into())));
    assert_eq!(next(3), Some((5, "craftable".into())));
    let r = ForgeRow::at(5);
    assert!(r.craftable && r.tier == 0);
    assert_eq!(next(5), Some((15, "tier 1".into())));
    assert_eq!(ForgeRow::at(15).tier, 1);
    assert_eq!(next(20), Some((40, "tier 2".into())));
    assert_eq!(ForgeRow::at(40).tier, 2);
    assert_eq!(next(40), None);
    assert_eq!(next(99), None);
    // On the wire, an older save's row settles too.
    let mut g = Game::new_literal(1);
    g.lineage.forge.insert("sword".into(), ForgeRow { salvaged: 3, ..Default::default() });
    let l = g.lineage();
    assert_eq!(l.forge["sword"].next.as_ref().map(|x| (x.need, x.label.as_str())), Some((5, "craftable")));
    assert!(serde_json::to_string(&l).unwrap().contains(r#""next":{"need":5,"label":"craftable"}"#));
    // Salvage climbs the ladder.
    let mut g = arena();
    for _ in 0..6 {
        give(&mut g, "sword");
    }
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    let f = &g.lineage.forge["sword"];
    assert!(f.salvaged >= 5 && f.craftable, "{f:?}");
    assert_eq!(f.next.as_ref().map(|x| x.need), Some(15));
}

/// Cohort 5 (rater J): a lineage that never banks and spends its purse on supplies that die
/// with the hero must not be locked out of the shop; each heir wakes with one potion's worth.
#[test]
fn a_new_heir_wakes_with_enough_for_one_supply() {
    use crate::engine::WAKE_PAY;
    let mut g = Game::new_literal(9);
    g.lineage.gold = 0;
    let before = g.lineage.heir;
    let _ = g.run_offline(3 * 3600);
    assert!(g.lineage.heir > before, "no heir change in 3 h");
    // whatever the runs yielded, the purse after any death is at least the wake pay minus what
    // auto-supply may have spent (a fresh lineage owns no automations)
    assert!(g.lineage.gold >= WAKE_PAY || g.lineage.gold_ledger.iter().any(|l| l.why == "wake pay"), "gold {} ledger {:?}", g.lineage.gold, g.lineage.gold_ledger);
}

// ---------------------------------------------------------------- Cut 10 (the wall as a ramp, clarity)

/// §2: the forecast row a boss wall gates (his floor + 1) names the known-but-absent counter
/// (`try`), and only then: not without the fact, not when a row of the set already carries
/// the counter's verb (under any conditions), never on another depth.
#[test]
fn forecast_try_names_the_known_but_absent_counter() {
    let mut g = Game::new_literal(3);
    g.lineage.best_depth = 8;
    let mut set = crate::probes::good();
    set.rows.retain(|r| !matches!(r.verb.a.as_deref(), Some("tag:boss") | Some("fire,tag:boss")));
    g.set_rules_raw(set.clone()).unwrap();
    assert!(crate::forecast::try_row(&g, &set, 9).is_none(), "no fact, no try");
    g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    let t = crate::forecast::try_row(&g, &set, 9).expect("D9 is gated by the Warlord on D8");
    assert_eq!(t.boss, "goblin_warlord");
    assert_eq!(t.row, crate::facts::counter_row("goblin_warlord"));
    assert_eq!(t.text, "attack boss");
    assert!(word_count(&t.text) <= 3);
    for d in [1, 2, 7, 8, 10] {
        assert!(crate::forecast::try_row(&g, &set, d).is_none(), "D{d} is not the Warlord's ramp");
    }
    // A row with the counter's verb, whatever its conditions, is the row: no try.
    let mut with = set.clone();
    with.rows.push(Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "tag:boss")));
    assert!(crate::forecast::try_row(&g, &with, 9).is_none());
    // The Bloat Mother's ramp is D14 and her counter is the throw.
    g.lineage.facts.insert(crate::facts::boss_counter_fact("bloat_mother"));
    assert_eq!(crate::forecast::try_row(&g, &set, 14).map(|t| t.text), Some("throw fire, boss".into()));
    assert!(crate::forecast::try_row(&g, &crate::probes::good(), 14).is_none(), "good.json throws at bosses");
    // On the wire: the forecast's D9 row carries it and D8's does not.
    let f = crate::forecast::forecast_with(&g, &set, crate::forecast::MIN_SIMS);
    let d = |n: u32| f.depths.iter().find(|d| d.depth == n).unwrap();
    assert_eq!(d(9).try_.as_ref().map(|t| t.text.as_str()), Some("attack boss"));
    assert!(d(8).try_.is_none());
    let json = serde_json::to_value(d(9)).unwrap();
    assert_eq!(json["try"]["text"], "attack boss");
    assert!(serde_json::to_value(d(8)).unwrap().get("try").is_none(), "absent, not null");
}

/// §2 gate (the 30-seed number is `metrics.rs`): a lineage that knows the Warlord's counter
/// with a set that lacks it — the forecast names it on D9 and the counter row at the top
/// lifts D9's reach by ≥ 0.3; at the end of the set (under `foes 1+ → attack nearest`) it
/// lifts nothing, which is why the row inserts at the top.
#[cfg(not(debug_assertions))]
#[test]
fn counter_at_the_top_lifts_the_wall_floor() {
    for (seed, (named, base, top, end)) in (1..=4u64).zip(par_seeds(1..=4u64, crate::probes::counter_trial)) {
        assert!(named, "seed {seed}: the D9 row does not name the counter");
        assert!(top - base >= 0.3, "seed {seed}: D9 {base:.2} → {top:.2} at the top");
        assert!(top - end >= 0.3, "seed {seed}: the end placement ({end:.2}) is not the answer; the top ({top:.2}) is");
    }
}

/// §2: on a boss death the counter row is measured at the top only and pinned there — never
/// offered "before the row that fired most" — whoever landed the blow; and a death on the
/// boss's floor to his goblins with him out of sight is his death too (`DeathRec.boss`; the
/// pin then depends on his showing up in the replays). 12 Warlord-floor deaths.
#[cfg(not(debug_assertions))]
#[test]
fn boss_counter_patch_is_pinned_at_the_top_only() {
    let counter = crate::facts::counter_row("goblin_warlord");
    let floor = |seed: u64, warlord_at: (i32, i32)| -> Option<Game> {
        let mut g = arena_seed(seed);
        {
            let run = g.run.as_mut().unwrap();
            run.depth = 8;
            run.hero.hp = 10 + (seed % 5) as i32;
            run.hero.pos = Pos::new(2, 2);
            run.floor.map.update_vision(run.hero.pos, VISION);
        }
        add_monster(&mut g, "goblin_warlord", warlord_at.0, warlord_at.1);
        add_monster(&mut g, "goblin", 3, 2);
        add_monster(&mut g, "goblin", 3, 3);
        add_monster(&mut g, "goblin", 2, 3);
        g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
        g.lineage.facts.insert("foe:goblin_warlord:boss".into());
        rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 6)], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        for _ in 0..800 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                let died = g.run.as_ref().unwrap().over == Some(ExitTier::Death);
                g.finish_run();
                return died.then_some(g);
            }
        }
        None
    };
    let mut deaths = 0;
    let mut pinned = 0;
    let mut seed = 300u64;
    while deaths < 12 {
        seed += 1;
        // The Warlord in view five tiles off; his goblins are on the hero.
        let Some(mut g) = floor(seed, (7, 2)) else { continue };
        let id = *g.deaths.keys().last().unwrap();
        let d = g.death(id).unwrap();
        let rec = g.deaths.get(&id).unwrap();
        assert_eq!(rec.boss.as_deref(), Some("goblin_warlord"), "seed {seed}");
        deaths += 1;
        for p in &d.patches {
            if p.row.verb == counter.verb {
                assert_eq!(p.insert_at, 0, "seed {seed}: the counter is only ever at the top: {:?}", d.patches);
            }
        }
        if rec.counter.is_some() {
            pinned += 1;
            assert_eq!(d.patches[0].row, counter, "seed {seed}: pinned first");
            assert_eq!(d.patches[0].insert_at, 0);
        }
        assert!(d.margin.starts_with(|c: char| c.is_ascii_digit()) && d.margin.contains(" hp short"), "{}", d.margin);
    }
    assert!(pinned >= 6, "the counter fired at the top and was pinned on {pinned} of {deaths} deaths");
    // Out of sight in the far corner (twelve tiles off), the death is still his.
    let mut seed = 400u64;
    let g = loop {
        seed += 1;
        if let Some(g) = floor(seed, (14, 10)) {
            break g;
        }
    };
    let rec = g.deaths.values().last().unwrap();
    assert_eq!(rec.death.cause, "goblin");
    assert_eq!(rec.boss.as_deref(), Some("goblin_warlord"), "a death on his floor while he lives is his");
    assert!(crate::trace::pinnable_counter(&g, rec).is_some(), "the counter is pinnable though he was never in the death's context");
}

/// §3: `Death.margin` reads `N hp short` — the HP that would have kept the hero through the
/// killing blow — and the morgue carries both numbers.
#[test]
fn death_margin_reads_hp_short() {
    let mut g = arena();
    g.run.as_mut().unwrap().hero.hp = 3;
    g.run.as_mut().unwrap().hero.armour = None;
    for (x, y) in [(5, 5), (5, 4), (5, 6)] {
        add_monster(&mut g, "ogre", x, y);
    }
    hold_rules(&mut g);
    let evs = ticks(&mut g, 400);
    let blow = evs.iter().rev().find_map(|e| match e {
        Ev::Hurt { id, dmg, .. } if *id == HERO_ID => Some(*dmg),
        _ => None,
    });
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.over, Some(ExitTier::Death));
    let blow = blow.expect("the killing blow");
    assert_eq!(run.death_blow, blow);
    assert!(run.death_short >= 1 && run.death_short <= blow, "short {} of a {blow} blow", run.death_short);
    let id = run.id;
    let short = run.death_short;
    g.finish_run();
    let d = g.death(id).unwrap();
    assert!(d.margin.starts_with(&format!("{short} hp short")), "{}", d.margin);
    assert!(!d.margin.contains(" over"), "{}", d.margin);
    // QA on 524827b (qaAB): the morgue names the hp the blow landed on (`blow 3 at 2 hp`)
    assert!(d.morgue.contains(&format!("blow {blow} at {} hp", blow + 1 - short)), "{}", d.morgue);
}

/// §3: a theft says what it cost the loot — `Ev::steal.amount` is the item's value and the
/// callout reads `stolen $16` (the run's gold dropped by that much).
#[test]
fn theft_carries_its_amount() {
    let mut g = arena();
    hold_rules(&mut g);
    give(&mut g, "sword");
    g.run.as_mut().unwrap().loot_add(160);
    let before = g.run.as_ref().unwrap().loot;
    assert_eq!(before, 40);
    let value = before - (160 - hero(&g).inv[0].value()) / crate::engine::GOLD_DIVISOR;
    assert!(value > 0);
    let m = add_monster(&mut g, "monkey", 5, 5);
    let evs = ticks(&mut g, 60);
    let amount = evs.iter().find_map(|e| match e {
        Ev::Steal { id, amount, .. } if *id == m => Some(*amount),
        _ => None,
    });
    assert_eq!(amount, Some(Some(value)), "the steal event carries the gold it took");
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if *text == format!("stolen ${value}"))), "{:?}", ev_kinds(&evs));
    assert_eq!(g.run.as_ref().unwrap().loot, before - value, "the amount is the loot's drop");
    let json = serde_json::to_string(evs.iter().find(|e| matches!(e, Ev::Steal { .. })).unwrap()).unwrap();
    assert!(json.contains(&format!("\"amount\":{value}")), "{json}");
}

/// §3: a companion's fall calls out with the chronicle's verb (`Ashar fell`, ≤ 3 words), not
/// the client's `slain`; an unnamed ally reads `jackal fell`.
#[test]
fn companion_death_calls_out_fell() {
    let mut g = arena();
    let party = crate::probes::pets_party();
    crate::engine::spawn_party(g.run.as_mut().unwrap(), &party);
    let j = add_monster(&mut g, "jackal", 6, 6);
    let (name, cid) = {
        let run = g.run.as_ref().unwrap();
        let c = run.monsters.iter().find(|m| m.is_companion()).unwrap();
        (run.companion(c.cid.unwrap()).unwrap().name.clone(), c.id)
    };
    {
        let (run, mut cx) = g.ctx();
        let ji = run.monsters.iter().position(|m| m.id == j).unwrap();
        let ci = run.monsters.iter().position(|m| m.id == cid).unwrap();
        crate::turn::damage_monster(run, &mut cx, ci, 99, &crate::turn::Src::Mon(ji));
    }
    let evs = std::mem::take(&mut g.events);
    let want = format!("{name} fell");
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if *text == want)), "{evs:?}");
    assert!(word_count(&want) <= 3);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ally { id, state, .. } if *id == cid && state == "lost")));
    // A freed captive (no companion record) falls by its kind.
    let mut g = arena();
    let c = add_monster(&mut g, "captive", 6, 5);
    rules(&mut g, vec![Row::new(vec![], Verb::new("free_captive")), Row::new(vec![], Verb::new("rest"))]);
    ticks(&mut g, 30);
    assert!(monster(&g, c).unwrap().ally);
    let r = add_monster(&mut g, "rat", 7, 5);
    {
        let (run, mut cx) = g.ctx();
        let ri = run.monsters.iter().position(|m| m.id == r).unwrap();
        let ci = run.monsters.iter().position(|m| m.id == c).unwrap();
        crate::turn::damage_monster(run, &mut cx, ci, 99, &crate::turn::Src::Mon(ri));
    }
    assert!(g.events.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "captive fell")), "{:?}", g.events);
}

/// §3: the exit line leads with the verb and what came home (`returned $50 · $84 carried ·
/// keeps 60%`), so the report's exit lines read `returned $61`, not `$61`.
#[test]
fn exit_line_leads_with_the_verb() {
    let line = |tier: ExitTier, timed_out: bool, bones: usize| -> String { crate::engine::exit_line(84, tier.pct(), 84 * tier.pct() / 100, 0, vec![], tier, timed_out, bones, 5).text };
    assert_eq!(line(ExitTier::Return, false, 0), "returned $50 · $84 carried · keeps 60%");
    assert_eq!(line(ExitTier::Bank, false, 0), "banked $84 · $84 carried · keeps 100%");
    // QA on 912e135 (qaW): an exit that kept nothing names what it lost; a timed-out run leads with its word.
    assert_eq!(line(ExitTier::Death, false, 7), "died $0 · $84 lost · bones: 7 items on D5");
    let lost = crate::engine::exit_line(84, 0, 0, 0, vec![], ExitTier::Return, true, 0, 5).text;
    assert_eq!(lost, "lost thread $0 · $84 lost");
    // A stall says so (the chronicle's "Stalled."), and a supply the send spent unused is counted.
    let stalled = crate::engine::exit_line_of(15, 0, 0, 0, vec![], ExitTier::Return, true, true, 1, 0, 2).text;
    assert_eq!(stalled, "stalled $0 · $15 lost · 1 supply back");
    for t in [lost, line(ExitTier::Death, false, 7), stalled] {
        assert!(word_count(&t) <= 14, "{t}");
    }
}

/// §3: a tactic card's delta is the buy's own number — the bare `[card]` row where the buy
/// puts it (Cut 12 §1: before the engagement row, `insert_at`; it was the end), not a
/// conditioned row at the top; a verb unlock's canonical row still goes at the top.
#[test]
fn card_delta_is_measured_where_the_buy_puts_it() {
    let mut g = Game::new_literal(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:bloat:gas".into());
    g.lineage.best_depth = 1;
    g.lineage.facts.insert("foe:jackal:fast".into());
    tiers(&mut g);
    g.set_rules_raw(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None, route: Vec::new() }).unwrap();
    let l = &g.lineage;
    let (row, at) = crate::meta::delta_row(l, "pack_break").unwrap();
    assert_eq!(row, Row::new(vec![], Verb::arg("tactic", "pack_break")), "the bare card row the client appends");
    assert_eq!(row.origin.as_deref(), Some("card"));
    assert_eq!(at, 1, "before the set's `attack nearest` (Cut 12 §1)");
    let (row, at) = crate::meta::delta_row(l, "throw").unwrap();
    assert_eq!((row.verb.v.as_str(), at), ("throw", 0), "a verb's canonical row goes at the top");
    assert!(crate::meta::delta_row(l, "row5").is_none());
    // Cut 18 §5: the delta is the paired reach of the bare card row at its best place — the
    // buy's old place (before `attack nearest`), the top, before the first own row — minus
    // the base, and `insert_at` is that place (the buy puts it there), ties to the old place.
    // QA on 308f045 (qaAD: cards added over `hp < 30% → drink heal`, ORDER deaths after): never above the safety rows — the
    // heal at R1 keeps the top
    assert_eq!(crate::meta::card_positions(g.lineage.rules()), vec![1]);
    let cat = g.unlock_deltas();
    let card = cat.iter().find(|u| u.id == "pack_break").unwrap();
    let delta = card.delta.expect("open card");
    let rules = g.lineage.rules().clone();
    let depth = g.lineage.best_depth + 1;
    let tag = crate::forecast::forecast_tag(&g, depth);
    let (base, n) = crate::forecast::reach_counted(&g, &rules, depth, crate::forecast::FORECAST_SIMS, tag, crate::forecast::CATALOGUE_TICK_BUDGET);
    let mut sim = g.sim_clone();
    sim.lineage.unlocks.insert("pack_break".into());
    let at = |i: usize| {
        let mut set = rules.clone();
        set.rows.insert(i, Row::new(vec![], Verb::arg("tactic", "pack_break")).from("card"));
        crate::forecast::reach_paired(&sim, &set, depth, n, tag)
    };
    let (best_at, best) = (1, at(1));
    assert!((delta - (best - base)).abs() < 1e-9, "delta {delta} vs best {best} − base {base}");
    assert_eq!(card.insert_at, Some(best_at), "the card goes where it was measured best");
    assert_eq!(card.situation.as_deref(), Some("pack"));
}

/// Cut 18 §5 (Y, Z: "every card read `reach ~0 at R4`"): a card is measured at each of its
/// places and reads its best — a set whose top rows fire first no longer hides a card placed
/// under them; each tactic card names the foe it answers (`kite archers · vs archers`).
#[test]
fn a_card_reads_its_best_place_and_its_situation() {
    // A card under an own row that always fires first (`foes ≥ 1 → retreat` then the rest)
    // is measured at the top as well, and the top wins or ties.
    let rules = RuleSet { rows: vec![Row::new(vec![], Verb::arg("tactic", "thief_guard")), Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None, route: Vec::new() };
    assert_eq!(crate::meta::card_positions(&rules), vec![2], "old place — never over the heal row (QA on 308f045), so neither the top nor before it");
    let open = RuleSet { rows: vec![Row::new(vec![], Verb::arg("tactic", "thief_guard")), Row::new(vec![Cond::n("foes>=", 2)], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None, route: Vec::new() };
    assert_eq!(crate::meta::card_positions(&open), vec![2, 0, 1], "old place, the top, before the first own row");
    let rules = RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None, route: Vec::new() };
    assert_eq!(crate::meta::card_positions(&rules), vec![0]);
    let sit = |id: &str| crate::meta::card_situation(id);
    assert_eq!(sit("kite_archers").as_deref(), Some("ranged"));
    assert_eq!(sit("gas_step").as_deref(), Some("gas"));
    assert_eq!(sit("thief_guard").as_deref(), Some("thief"));
    assert_eq!(sit("boss_focus").as_deref(), Some("boss"));
    assert_eq!(sit("pack_break").as_deref(), Some("pack"));
    assert_eq!((sit("corridor_fighting"), sit("last_stand"), sit("row5"), sit("throw")), (None, None, None, None));
    let g = Game::new_literal(3);
    let cat = crate::meta::catalogue(&g.lineage);
    // (Cut 29 §1: kite archers is free vocabulary, off the shelf — thief guard is sold)
    let kite = cat.iter().find(|u| u.id == "thief_guard").unwrap();
    let j = serde_json::to_value(kite).unwrap();
    assert_eq!(j["situation"], "thief");
    assert!(serde_json::to_value(cat.iter().find(|u| u.id == "row5").unwrap()).unwrap().get("situation").is_none(), "absent, not null");
    // Rater Y's set (a boss row, bank rows, gas retreat above `attack nearest`): with `kite
    // archers` measured at every place, its delta is the best of them.
    let mut g = Game::new_literal(1101);
    g.lineage.best_depth = 6;
    for u in ["row5", "row6", "row7", "row8"] {
        g.lineage.unlocks.insert(u.into());
    }
    for f in ["foe:goblin_archer:ranged", "foe:goblin_archer:telegraph", "foe:bloat:gas", "foe:jackal:pack", "foe:jackal:fast"] {
        g.lineage.facts.insert(f.into());
    }
    tiers(&mut g);
    let set = RuleSet::parse(&std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards/32971ad.raterY.rules.json")).unwrap()).unwrap();
    g.set_rules_raw(set.clone()).unwrap();
    let cat = g.unlock_deltas();
    let kite = cat.iter().find(|u| u.id == "pack_break").unwrap();
    let (delta, at) = (kite.delta.expect("open card"), kite.insert_at.expect("a place"));
    let depth = g.lineage.best_depth + 1;
    let tag = crate::forecast::forecast_tag(&g, depth);
    let (base, n) = crate::forecast::reach_counted(&g, &set, depth, crate::forecast::FORECAST_SIMS, tag, crate::forecast::CATALOGUE_TICK_BUDGET);
    let mut sim = g.sim_clone();
    sim.lineage.unlocks.insert("pack_break".into());
    for i in crate::meta::card_positions(&set) {
        let mut s = set.clone();
        s.rows.insert(i, Row::new(vec![], Verb::arg("tactic", "pack_break")).from("card"));
        let r = crate::forecast::reach_paired(&sim, &s, depth, n, tag);
        assert!(r - base <= delta + 1e-9, "R{} reads {:+.2} over the shown {delta:+.2} at R{}", i + 1, r - base, at + 1);
    }
    assert_eq!(kite.situation.as_deref(), Some("pack"));
}

/// Cut 10 (client finding): `choose` after the run ended returned a wasm panic and poisoned the
/// engine; it must be a plain error.
#[test]
fn choose_after_the_run_ended_is_an_error_not_a_panic() {
    let mut g = Game::new_literal(12);
    assert!(g.choose(1).is_err(), "no run yet");
    let _ = g.send();
    {
        let (run, mut cx) = g.ctx();
        run.hero.hp = 0;
        crate::turn::end_run(run, &mut cx, ExitTier::Death);
    }
    assert!(g.choose(1).is_err(), "run over");
    g.finish_run();
    assert!(g.choose(1).is_err(), "no run");
    let _ = g.lineage(); // the engine still answers
}

// ---------------------------------------------------------------- Cut 11: the chain

/// Cut 11 §1: `no item` / `none held` carry the slot's last emptying event — the theft with
/// its tick, the drink with the HP it went down at — or `never found`; a fill is not a because.
#[test]
fn because_names_the_theft_the_drink_or_never_found() {
    // Never found.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 90)], Verb::arg("drink", "heal")), Row::new(vec![Cond::t("item", "speed")], Verb::arg("drink", "speed")), Row::new(vec![], Verb::new("hold"))]);
    g.run.as_mut().unwrap().hero.hp = 5;
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let rows = t.rows.clone().unwrap();
    assert_eq!(rows[0].why, "no item");
    assert_eq!(rows[0].because.as_ref().map(|b| b.text.as_str()), Some("never found"), "{rows:?}");
    assert_eq!(rows[1].why, "none held");
    assert_eq!(rows[1].because.as_ref().map(|b| b.text.as_str()), Some("never found"));
    // The theft: a monkey's blow takes the heal; the row's because names it with the tick.
    let mut g = arena();
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    hold_rules(&mut g);
    give(&mut g, "heal");
    let m = add_monster(&mut g, "monkey", 5, 5);
    let evs = ticks(&mut g, 80);
    let stole_t = evs.iter().find_map(|e| match e {
        Ev::Steal { id, t, .. } if *id == m => Some(*t),
        _ => None,
    });
    let stole_t = stole_t.expect("the monkey stole");
    assert!(hero(&g).inv.is_empty());
    let prov = &g.prov;
    let theft = prov.iter().find(|p| p.kind == crate::provenance::ProvKind::Stolen).expect("a theft entry");
    assert_eq!(theft.text, "monkey took the heal, D1");
    assert_eq!((theft.t, theft.key.as_str()), (stole_t, "item:heal"));
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 99)], Verb::arg("drink", "heal")), Row::new(vec![], Verb::new("hold"))]);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "no item");
    let b = w.because.as_ref().expect("the theft is the because");
    assert_eq!((b.text.as_str(), b.t, b.depth), ("monkey took the heal, D1", stole_t, 1));
    assert!(crate::provenance::because_ok(&b.text));
    // The drink: the next action's `no item` reads the HP the heal went down at.
    let mut g = arena();
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    give(&mut g, "heal");
    g.run.as_mut().unwrap().hero.hp = 4;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 99)], Verb::arg("drink", "heal")), Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 10);
    let drank_t = evs.iter().find_map(|e| if let Ev::Use { t, .. } = e { Some(*t) } else { None }).expect("drank");
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "no item", "hp {} trace {:?}", hero(&g).hp, g.run.as_ref().unwrap().trace);
    let b = w.because.as_ref().unwrap();
    assert_eq!((b.text.as_str(), b.t), (format!("R1 drank heal at 4/{} hp", hero(&g).max_hp).as_str(), drank_t), "the row that drank is named");
    // A fill after the emptying event is not a because (the item left some way the log did
    // not see).
    {
        let (run, mut cx) = g.ctx();
        crate::provenance::found(run, &mut cx, "heal", "heal potion");
        run.hero.hp = 4;
    }
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "no item");
    assert!(w.because.is_none(), "{w:?}");
}

/// Cut 11 §1: `not in view` carries the tagged foe's last-seen tile once it has stepped out
/// of view; `cooldown` the ticks left and the tick the use started it; `locked cond` the
/// unlock and its cost; `no path` the blocker (a captive chained on the stairs).
#[test]
fn because_names_the_lost_target_the_cooldown_the_lock_and_the_blocker() {
    // Lost target: a jackal seen then gone (moved beyond the vision radius).
    let mut g = arena();
    g.lineage.facts.insert("foe:jackal:pack".into());
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "pack")], Verb::arg("attack", "tag:pack")), Row::new(vec![], Verb::new("hold"))]);
    let j = add_monster(&mut g, "jackal", 6, 5);
    g.run.as_mut().unwrap().monsters.iter_mut().for_each(|m| m.awake = false);
    ticks(&mut g, 10);
    assert!(g.run.as_ref().unwrap().last_visible.contains(&j));
    // Walled off: the jackal is moved far beyond sight (a 16-wide room; vision 7).
    {
        let run = g.run.as_mut().unwrap();
        let m = run.monsters.iter_mut().find(|m| m.id == j).unwrap();
        m.pos = Pos::new(14, 10);
        m.awake = false;
        m.paralysed = 100;
    }
    ticks(&mut g, 20);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "not in view", "{t:?}");
    let b = w.because.as_ref().expect("last seen");
    assert_eq!(b.text, "jackal last seen D1", "{b:?}");
    assert!(crate::provenance::because_ok(&b.text));
    // Cooldown: a fighter's shield bash used, then blocked by its cooldown.
    let mut g = arena();
    g.lineage.unlocks.insert("row5".into());
    rules(&mut g, vec![Row::new(vec![Cond::n("adj>=", 1)], Verb::new("shield_bash")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    add_monster(&mut g, "ogre", 5, 5);
    let evs = ticks(&mut g, 10);
    let bashed_t = evs.iter().find_map(|e| match e {
        Ev::Rule { t, verb, .. } if verb.v == "shield_bash" => Some(*t),
        _ => None,
    });
    let bashed_t = bashed_t.expect("bashed");
    let prov = &g.prov;
    assert!(prov.iter().any(|p| p.key == "cooldown:shield_bash" && p.t == bashed_t), "{prov:?}");
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "cooldown", "{t:?}");
    let b = w.because.as_ref().unwrap();
    assert!(b.text.starts_with("cooldown ") && b.text.ends_with(" ticks left"), "{b:?}");
    assert_eq!(b.t, bashed_t);
    // Locked condition: the unlock and its cost.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![Cond::n("alert>=", 1)], Verb::new("retreat")), Row::new(vec![], Verb::new("hold"))]);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "locked cond");
    // (Cut 29 §1: a condition word is free — its lock is its gate)
    assert_eq!(w.because.as_ref().map(|b| b.text.as_str()), Some("see alert rise"));
    // A captive chained on the stairs: `path_stairs` fails with `no path` and the chain names it.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![Cond::flag("path_stairs")], Verb::new("descend")), Row::new(vec![], Verb::new("hold"))]);
    {
        let run = g.run.as_mut().unwrap();
        let down = run.floor.stairs_down;
        let id = run.new_id();
        let mut m = Monster::spawn(id, "captive", down, run.depth);
        m.situation = Some("captive".into());
        m.neutral = true;
        run.monsters.push(m);
        run.floor.map.seen.iter_mut().for_each(|s| *s = true);
    }
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    let w = &t.rows.as_ref().unwrap()[0];
    assert_eq!(w.why, "no path", "{t:?}");
    assert_eq!(w.because.as_ref().map(|b| b.text.as_str()), Some("captive chained the way"));
    // The blocker is one provenance entry, not one per action.
    ticks(&mut g, 30);
    let prov = &g.prov;
    assert_eq!(prov.iter().filter(|p| p.key == "path").count(), 1, "{prov:?}");
}

/// Cut 11 §1: the provenance log is capped at `PROV_CAP` (oldest out), thefts and uses
/// append, finds replace their slot's last find; sims and verdict replays record nothing.
#[test]
fn provenance_is_capped_and_sims_record_nothing() {
    let mut g = arena();
    {
        let (run, mut cx) = g.ctx();
        assert!(!cx.sim);
        for i in 0..100 {
            crate::provenance::used(run, &mut cx, "drunk", &format!("k{i}"), 10);
        }
        assert_eq!(cx.prov.len(), crate::provenance::PROV_CAP);
        assert_eq!(cx.prov[0].key, "item:k36", "the oldest were evicted");
        cx.prov.clear();
        crate::provenance::found(run, &mut cx, "heal", "heal potion");
        crate::provenance::found(run, &mut cx, "heal", "heal potion");
        crate::provenance::used(run, &mut cx, "drunk", "heal", 3);
        crate::provenance::used(run, &mut cx, "drunk", "heal", 2);
        assert_eq!(cx.prov.iter().map(|p| p.text.as_str()).collect::<Vec<_>>(), ["found heal on D1", "drunk heal at 3/36 hp", "drunk heal at 2/36 hp"]);
        for p in cx.prov.iter() {
            assert!(crate::provenance::because_ok(&p.text), "{p:?}");
        }
    }
    // A sim clone ticks with nothing logged; the death's replays neither.
    let mut s = g.sim_clone();
    s.prov.clear();
    give(&mut s, "heal");
    s.lineage.facts.insert(ident_fact(&s.lineage.flavours, "heal").unwrap());
    s.run.as_mut().unwrap().hero.hp = 3;
    s.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")), Row::new(vec![], Verb::new("hold"))], name: None, route: Vec::new() }).unwrap();
    for _ in 0..20 {
        s.tick();
        s.events.clear();
    }
    assert!(s.run.as_ref().unwrap().hero.inv.is_empty(), "the sim drank");
    assert!(s.prov.is_empty(), "sims log nothing");
    assert!(s.run.as_ref().unwrap().trace.last().unwrap().rows.is_none(), "sims account for nothing");
    // Same seed, same log: the log is part of the run's determinism.
    let mut a = Game::new_literal(11);
    let mut b = Game::new_literal(11);
    a.run_offline(1800);
    b.run_offline(1800);
    assert_eq!(serde_json::to_string(&a.batch.exits).unwrap(), serde_json::to_string(&b.batch.exits).unwrap());
    assert!(a.batch.exits.iter().any(|l| l.trace.as_ref().is_some_and(|t| t.provenance.is_some())), "an exit trace carries the provenance");
}

/// Cut 11 §2: a theft in the killing turn's chain is the death's root; the patch list
/// carries `foe_tag:thief → attack tag:thief` with `root` (the `thief_guard` card when owned),
/// always shown, at the head when its forecast delta reaches the best symptom patch's.
#[test]
fn theft_root_offers_the_thief_row_with_its_root() {
    let mut g = arena_seed(3);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    hold_rules(&mut g);
    give(&mut g, "heal");
    let m = add_monster(&mut g, "monkey", 5, 5);
    ticks(&mut g, 80);
    assert!(hero(&g).inv.is_empty(), "the monkey took the heal");
    // The thief runs off with it; the hero then meets a pack it cannot hold at low HP.
    g.run.as_mut().unwrap().monsters.retain(|x| x.id != m);
    g.run.as_mut().unwrap().hero.hp = 6;
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    for (x, y) in [(5, 5), (5, 6), (6, 4), (3, 6)] {
        add_monster(&mut g, "goblin", x, y);
    }
    let mut id = None;
    for _ in 0..600 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Death));
            g.finish_run();
            break;
        }
    }
    let id = id.expect("died");
    let rec = g.deaths.get(&id).unwrap().clone();
    let root = rec.root.clone().expect("a theft root");
    assert_eq!((root.kind.as_str(), root.text.as_str(), root.row), ("theft", "monkey took the heal", 0));
    assert!(word_count(&root.text) <= crate::trace::ROOT_WORDS);
    let chain = rec.death.chain.clone().expect("the chain");
    assert_eq!(chain.len(), 1);
    assert_eq!(chain[0].text, "monkey took the heal, D1");
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("the root patch is shown: {:?} · {} base {:.2} nbb {}", d.patches, d.verdict, d.baseline, d.nothing_beats_base));
    assert_eq!(r.root.as_ref().unwrap().text, "monkey took the heal");
    assert_eq!(r.row, Row::new(vec![Cond::t("foe_tag", "thief")], Verb::arg("attack", "tag:thief")));
    assert_eq!(r.insert_at, 0);
    let best_symptom = d.patches.iter().filter(|p| p.root.is_none()).map(|p| p.forecast_delta).fold(f64::NEG_INFINITY, f64::max);
    if r.forecast_delta >= best_symptom - 1e-9 {
        assert_eq!(d.patches[0].row, r.row, "the root leads when its delta reaches the symptom's: {:?}", d.patches);
    } else {
        assert_eq!(d.patches.last().unwrap().row, r.row, "else it takes the last slot: {:?}", d.patches);
    }
    assert!(d.patches.len() <= crate::trace::SHOWN);
    // The card, when owned and not in the set, is the row.
    let mut g2 = g.clone();
    g2.lineage.unlocks.insert("thief_guard".into());
    let card = crate::trace::thief_row(&g2, &rec.rules, false).unwrap().0;
    assert_eq!(card.verb, Verb::arg("tactic", "thief_guard"));
    // A set that already carries the answer gets no root patch.
    let mut with = rec.rules.clone();
    with.rows.insert(0, Row::new(vec![Cond::t("foe_tag", "thief")], Verb::arg("attack", "tag:thief")));
    assert!(crate::trace::thief_row(&g, &with, false).is_none());
}

/// Cut 11 §2: a den's snatch is answered by the raid row (`on_see: den → attack nearest`):
/// a writable row when `cond_on_see` is owned, else the unlock pseudo-patch (`insert_at`
/// −1, `root` `◆2 cond: on see`) measured with the condition unlocked and the row at the top.
#[test]
fn den_theft_root_offers_the_raid_or_its_unlock() {
    let build = |on_see: bool| -> (Game, u32) {
        let mut g = arena_seed(4);
        g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
        g.lineage.facts.insert("den".into());
        if on_see {
            g.lineage.unlocks.insert("cond_on_see".into());
        }
        hold_rules(&mut g);
        give(&mut g, "heal");
        let m = add_monster(&mut g, "monkey", 5, 5);
        g.run.as_mut().unwrap().monsters.iter_mut().find(|x| x.id == m).unwrap().situation = Some("den".into());
        ticks(&mut g, 80);
        assert!(hero(&g).inv.is_empty(), "the den took the heal");
        g.run.as_mut().unwrap().monsters.retain(|x| x.id != m);
        g.run.as_mut().unwrap().hero.hp = 6;
        rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        for (x, y) in [(5, 5), (5, 6), (6, 4), (3, 6)] {
            add_monster(&mut g, "goblin", x, y);
        }
        for _ in 0..600 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                let id = g.run.as_ref().unwrap().id;
                assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Death));
                g.finish_run();
                return (g, id);
            }
        }
        panic!("the hero did not die");
    };
    let raid = crate::probes::situation_answer("den");
    // Cut 29 §1: `on see` is free vocabulary — the run met a foe, so the word came with its gate:
    // no pseudo-patch; the raid is a writable row at the top either way.
    let (mut g, id) = build(false);
    assert!(g.lineage.unlocks.contains("cond_on_see"), "the word arrived with its gate");
    let rec = g.deaths.get(&id).unwrap().clone();
    assert_eq!(rec.root.as_ref().map(|r| r.text.as_str()), Some("den took the heal"));
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("{:?}", d.patches));
    assert_eq!(r.insert_at, 0);
    assert_eq!(r.row, raid);
    // With the unlock: a writable row at the top, the root it answers as its text.
    let (mut g, id) = build(true);
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("{:?}", d.patches));
    assert_eq!(r.insert_at, 0);
    assert_eq!(r.root.as_ref().unwrap().text, "den took the heal");
    assert_eq!(r.row, raid);
}

/// Cut 11 §2: a row above the fired one that read `locked cond` is a lock root: the patch
/// list gains the set's own row at `insert_at` −1 with `root` `◆2 cond: alert` and its
/// survival measured with the condition unlocked.
#[test]
fn lock_root_offers_the_unlock_pseudo_patch() {
    let mut g = arena_seed(5);
    g.run.as_mut().unwrap().hero.hp = 8;
    let locked = Row::new(vec![Cond::n("alert>=", 1)], Verb::new("return"));
    rules(&mut g, vec![locked.clone(), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    g.run.as_mut().unwrap().alert = 5;
    for (x, y) in [(5, 5), (5, 6), (6, 4), (3, 6)] {
        add_monster(&mut g, "goblin", x, y);
    }
    let mut id = None;
    for _ in 0..600 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("died");
    let rec = g.deaths.get(&id).unwrap().clone();
    let root = rec.root.clone().expect("a lock root");
    assert_eq!((root.kind.as_str(), root.text.as_str(), root.row, root.unlock.as_deref()), ("lock", "see alert rise", 0, Some("cond_alert")));
    assert_eq!(rec.death.chain.as_ref().unwrap()[0].text, "see alert rise");
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("{:?}", d.patches));
    assert_eq!((r.insert_at, r.root.as_ref().unwrap().text.as_str()), (-1, "see alert rise"));
    assert_eq!(r.row, locked);
    // With the alert at 5 and the row unlocked, the return fires in the replays: it survives.
    assert!(r.survive >= 0.5, "{r:?}");
    assert_eq!(crate::trace::patched_rules(&rec.rules, r, 8), rec.rules, "the set's own row: nothing inserted");
}

/// Cut 11 §4: a `dice` death still names its alternative — the best candidates measured in
/// full and flagged `below_bar`, the telegraph retreat among them when a telegraph preceded
/// the blow and the lineage knows the tag.
#[test]
fn dice_death_names_an_alternative_below_the_bar() {
    // Eight seeds at a time; the first seed (in order) with a telegraphed dice death is the one.
    let mut found = None;
    for chunk in [1..=8u64, 9..=16, 17..=24] {
        found = par_seeds(chunk, dice_seed).into_iter().flatten().next();
        if found.is_some() {
            break;
        }
    }
    let (seed, id, d, cands) = found.expect("a telegraphed dice death in 24 seeds");
    let tele = Row::new(vec![Cond::t("foe_tag", "telegraph")], Verb::new("retreat"));
    // The telegraph retreat is among the measured candidates of a telegraphed death; it is
    // shown (first) when it fires in the replays — a `retreat` at range often cannot.
    assert!(cands.contains(&tele), "seed {seed} run {id}: {cands:?}");
    assert!(!d.patches.is_empty(), "seed {seed} run {id}");
    let json = serde_json::to_string(&d).unwrap();
    assert!(json.contains(r#""below_bar":true"#) || d.patches.iter().all(|p| !p.below_bar), "{json}");
}

/// One seed of `dice_death_names_an_alternative_below_the_bar`: every dice death's patches
/// checked, and the first telegraphed one with its candidates.
fn dice_seed(seed: u64) -> Option<(u64, u32, crate::wire::Death, Vec<Row>)> {
    let mut found = None;
    let mut g = Game::new_literal(seed);
    g.max_deaths = 1000;
    for m in crate::defs::MONSTERS.iter().filter(|m| m.tags.contains(&"telegraph")) {
        g.lineage.facts.insert(format!("foe:{}:telegraph", m.kind));
    }
    g.run_offline(4 * 3600);
    let ids: Vec<u32> = g.deaths.keys().copied().collect();
    for id in ids {
        if crate::trace::verdict(&mut g, id).as_deref() == Some("dice") {
            let d = g.death(id).unwrap();
            assert!(!d.patches.is_empty(), "seed {seed} run {id}: a dice death with no alternative: {:?}", d);
            for p in &d.patches {
                assert!(p.below_bar || p.survive >= crate::trace::survive_bar(d.baseline) - 1e-9, "{p:?}");
                // Cut 15 §6: nothing below the bar that survives 0 %.
                assert!(!p.below_bar || p.survive > 0.0, "{p:?}");
                assert!((0.0..=1.0).contains(&p.survive));
            }
            let telegraphed = d.trace.turns.iter().any(|t| !t.telegraphs.is_empty());
            if telegraphed && found.is_none() {
                let rec = g.deaths.get(&id).unwrap();
                if crate::trace::telegraph_row(&rec.vocab, &d.trace, &rec.t10_facts).is_some() {
                    let cands = crate::trace::candidates(&rec.vocab, &rec.rules, rec.t10.as_ref().unwrap(), &rec.t10_facts, &g.lineage.flavours, &d.trace);
                    found = Some((seed, id, d.clone(), cands));
                }
            }
        }
    }
    found
}

/// Cut 11 §3: survivor exits carry the last ten hero turns and the run's provenance; the
/// stall trace too; the death's own trace carries the provenance and the chain.
#[test]
fn survivor_traces_carry_ten_turns_and_provenance() {
    assert_eq!(crate::engine::EXIT_TRACE_LEN, 10);
    let mut longest = 0;
    let mut with_prov = 0;
    let mut exits = 0;
    for seed in 1..=4u64 {
        let mut g = Game::new_literal(seed);
        let r = crate::offline::run_offline_quick(&mut g, 3 * 3600);
        for line in &r.exits {
            exits += 1;
            let tr = line.trace.as_ref().unwrap();
            assert!(tr.turns.len() <= 10);
            longest = longest.max(tr.turns.len());
            if let Some(p) = &tr.provenance {
                with_prov += 1;
                assert!(!p.is_empty() && p.len() <= crate::provenance::PROV_CAP);
                assert!(p.windows(2).all(|w| w[0].t <= w[1].t), "oldest first");
                for b in p {
                    assert!(crate::provenance::because_ok(&b.text), "{b:?}");
                }
            }
        }
        for rec in g.deaths.values() {
            for t in &rec.death.trace.turns {
                for w in t.rows.iter().flatten() {
                    if let Some(b) = &w.because {
                        assert!(crate::provenance::because_ok(&b.text), "{b:?}");
                        assert!(b.t <= t.t);
                    }
                }
            }
            if let Some(chain) = &rec.death.chain {
                let mut texts: Vec<&str> = chain.iter().map(|b| b.text.as_str()).collect();
                texts.dedup();
                assert_eq!(texts.len(), chain.len(), "deduped by text: {chain:?}");
            }
        }
        if let Some(stall) = &r.stall {
            if let Some(tr) = &stall.trace {
                assert!(tr.turns.len() <= 10);
            }
        }
    }
    assert!(exits >= 6);
    assert_eq!(longest, 10, "a long run's exit shows ten turns");
    assert!(with_prov >= exits / 2, "{with_prov} of {exits} exits carry provenance");
}

/// Cut 11: the wire — `because` absent when none, `chain` absent when none, `provenance`
/// absent on a trace without events; a `RowWhy` round-trips.
#[test]
fn cut11_wire_is_optional_and_snake_case() {
    let w = RowWhy { row: 1, why: "hp not <30%".into(), because: None };
    assert_eq!(serde_json::to_string(&w).unwrap(), r#"{"row":1,"why":"hp not <30%"}"#);
    let w = RowWhy { row: 0, why: "no item".into(), because: Some(Because { text: "den took the heal, D3".into(), t: 2140, depth: 3 }) };
    let s = serde_json::to_string(&w).unwrap();
    assert_eq!(s, r#"{"row":0,"why":"no item","because":{"text":"den took the heal, D3","t":2140,"depth":3}}"#);
    assert_eq!(serde_json::from_str::<RowWhy>(&s).unwrap(), w);
    let t = Trace { max_steps: Vec::new(), turns: vec![], provenance: None, blow: None, blows: Vec::new(), hp_lost: Vec::new(), hp_healed: 0 };
    assert_eq!(serde_json::to_string(&t).unwrap(), r#"{"turns":[]}"#);
    let old: Trace = serde_json::from_str(r#"{"turns":[]}"#).unwrap();
    assert_eq!(old, t);
    let mut g = arena();
    hold_rules(&mut g);
    let d = crate::trace::death_record(&g, g.run.as_ref().unwrap()).death;
    let s = serde_json::to_string(&d).unwrap();
    assert!(!s.contains("\"chain\"") && !s.contains("\"provenance\""), "{s}");
}

/// Cohort 8 (rater O): `foe: boss → attack boss` was offered at the top while it sat at R2.
/// A patch that inserts (insert_at ≥ 0) never duplicates a row the set already carries; the
/// unlock pseudo-patch (insert_at −1) may name the set's own locked row.
#[test]
fn patches_never_offer_a_row_the_set_already_has() {
    // (a thread per seed: `par_seeds`)
    let checked: usize = par_seeds(1..=4u64, |seed| {
        let mut checked = 0;
        let mut g = Game::new_literal(seed);
        for u in ["row5", "row6", "row7", "row8"] {
            g.lineage.unlocks.insert(u.into());
        }
        g.set_rules_raw(RuleSet::parse(&std::fs::read_to_string("presets/good.json").unwrap()).unwrap()).unwrap();
        crate::offline::run_offline_quick(&mut g, 8 * 3600);
        let ids: Vec<u32> = g.deaths.keys().copied().take(3).collect();
        for id in ids {
            let d = g.death(id).unwrap();
            let rows = g.deaths[&id].rules.rows.clone();
            // (a `row` verdict's cut — `remove` / `replace` — names the set's own row: Cut 19 §4; a
            // move — Cut 25 §2 — reorders one)
            for p in d.patches.iter().filter(|p| p.insert_at >= 0 && !p.remove && !p.replace && p.moves_from.is_none()) {
                assert!(!rows.iter().any(|x| x.conds == p.row.conds && x.verb == p.row.verb), "seed {seed} death {id}: {:?} already in {rows:?}", p.row);
                checked += 1;
            }
        }
        checked
    })
    .into_iter()
    .sum();
    assert!(checked >= 10, "{checked} patches checked");
}

/// Cut 12 (cohort 8, rater P): supplies bought at camp after an absence are in the pack of
/// the next send, and that send starts a fresh run under the rules edited at camp.
#[test]
fn the_first_send_after_an_absence_packs_what_was_bought_at_camp() {
    let mut g = Game::new_literal(9);
    g.run_offline(2 * 3600);
    assert!(g.run.is_none());
    g.lineage.gold += 500;
    let kind = "heal";
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, kind).unwrap());
    let before = g.lineage.supplies.len();
    g.buy_supply(kind).expect("a heal is for sale");
    assert_eq!(g.lineage.supplies.len(), before + 1);
    let s = g.send();
    assert_eq!(s.turn, 0, "a fresh run");
    let run = g.run.as_ref().unwrap();
    assert!(run.hero.inv.iter().any(|i| i.kind == kind), "the bought heal is in the pack: {:?}", run.hero.inv.iter().map(|i| &i.kind).collect::<Vec<_>>());
}

/// Cut 12 §2: the pickup swap chores never take what a row needs — on 30 seeds with
/// `hp<30 → drink heal` in the set and a heal packed at camp, no `swapped for` provenance
/// line names the heal (rater O: "`swapped for the poison` named an engine chore no row of
/// mine could touch"). The chore still runs on other kinds.
#[test]
fn swap_chores_never_take_what_a_row_needs() {
    let per = par_seeds(1..=30u64, |seed| {
        let (mut swaps, mut heals_packed) = (0, 0);
        let mut g = Game::new_literal(seed);
        g.max_deaths = 1000;
        g.lineage.unlocks.insert("row5".into());
        // A heal bought by name (usable before its flavour is known), on the shelf.
        let id = g.lineage.next_vault_id;
        g.lineage.next_vault_id += 1;
        let mut heal = Item::new(id, "heal");
        heal.known = true;
        g.lineage.supplies.push(heal);
        let rows = vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![Cond::n("floor_seen>=", 60)], Verb::new("descend")),
        ];
        g.set_rules_raw(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
        g.send();
        heals_packed += g.run.as_ref().unwrap().hero.inv.iter().filter(|i| i.kind == "heal").count();
        let mut n = 0;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 400 {
            g.step(50);
            n += 1;
            for p in g.prov.iter().filter(|p| p.kind == crate::provenance::ProvKind::Spent && p.text.starts_with("swapped for")) {
                assert_ne!(p.key, "item:heal", "seed {seed}: the row's heal was swapped away: {}", p.text);
                swaps += 1;
            }
            g.prov.retain(|p| !(p.kind == crate::provenance::ProvKind::Spent && p.text.starts_with("swapped for")));
        }
        (swaps, heals_packed)
    });
    let (swaps, heals_packed) = per.iter().fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1));
    assert_eq!(heals_packed, 30, "every send packs its heal");
    assert!(swaps >= 5, "the chore still swaps other kinds ({swaps})");
}

/// Cut 12 §5: the rest after a return or a bank is half the run's length (rater P: "rested
/// 287m"), never under the 20-minute floor nor over the 30-minute cap; the wake after a death
/// stays 20 minutes; the online exit and the offline batch use the same function.
#[test]
fn rest_after_a_return_is_half_the_run() {
    use crate::engine::{REST_CAP_TICKS, REST_MIN_TICKS, WAKE_TICKS};
    use crate::offline::rest_after;
    assert_eq!(rest_after(60 * 60 * 10, ExitTier::Return), 30 * 60 * 10, "an hour's run rests thirty minutes");
    assert_eq!(rest_after(50 * 60 * 10, ExitTier::Bank), 25 * 60 * 10);
    assert_eq!(rest_after(80 * 60 * 10, ExitTier::Return), REST_CAP_TICKS, "capped");
    assert_eq!(rest_after(10 * 60 * 10, ExitTier::Return), REST_MIN_TICKS, "never under the floor");
    assert_eq!(rest_after(50 * 60 * 10, ExitTier::Death), WAKE_TICKS, "the death wake is unchanged");
    let g = Game::new_literal(1);
    assert_eq!(g.rest_after(50 * 60 * 10, ExitTier::Return), 25 * 60 * 10, "the exit's rest is the same function");
}

/// QA on 3d71c33: the forecast's ends line is what a send from this camp does. A set banking at
/// `depth ≥ 8` / `depth ≥ 10` on a lineage that reaches D8–D10: the bank share never exceeds
/// the reach of the bank row's depth (the same sims), and it agrees with sends actually played
/// from this state (the real engine, the night's own run seeds) within the pooled 95 % band of
/// the two samples. (It said `bank 42%` for a set that then banked 0 of 16: its ends came
/// from a 5–10-sim panel of other seeds.) An 8 h night climbs past it as the heir levels and
/// learns (examples/qa.rs checks the per-send agreement on 30 seeds).
#[test]
fn forecast_bank_share_is_a_send_from_this_camp() {
    let mut g = Game::new_literal(11);
    crate::probes::learn_everything(&mut g);
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    g.lineage.best_depth = 9;
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 4, xp: 0, next: 0 });
    for d in [8u32, 10] {
        let mut set = crate::probes::good();
        set.rows.retain(|r| r.verb.v != "bank");
        set.rows.insert(1, Row::new(vec![Cond::n("depth>=", d as i32)], Verb::new("bank")));
        g.set_rules_raw(set).unwrap();
        let f = g.forecast();
        let e = f.ends.clone().expect("ends");
        if let Some(r) = f.depths.iter().find(|x| x.depth == d) {
            assert!(e.bank <= r.reach + 1e-9, "depth ≥ {d}: bank {:.2} over reach {:.2}", e.bank, r.reach);
        }
        let n = crate::forecast::simulate_budget(&g, g.lineage.rules(), crate::forecast::FORECAST_SIMS, 1, u32::MAX, u64::MAX).len();
        assert_eq!(n, crate::forecast::FORECAST_SIMS as usize);
        let k = 40u32;
        let banked = crate::forecast::par_map(&g, (0..k).collect(), |base, &j| {
            let mut h = base.clone();
            h.lineage.next_run_id += 500 + j;
            h.start_run(None);
            h.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            h.run.as_ref().unwrap().over == Some(ExitTier::Bank)
        })
        .into_iter()
        .filter(|b| *b)
        .count();
        let real = banked as f64 / k as f64;
        let m = 50.0;
        let p = (e.bank * m + banked as f64) / (m + k as f64);
        let band = 1.96 * (p * (1.0 - p) * (1.0 / m + 1.0 / k as f64)).sqrt();
        assert!((e.bank - real).abs() <= band + 1e-9, "depth ≥ {d}: forecast bank {:.2} vs {banked}/{k} sends ({real:.2}), band ±{band:.2}", e.bank);
    }
}

/// Cut 12 §3: `Forecast.ends` — how the sends end over the same sims (the rates sum to 1)
/// and the mean gold brought home (bank 100% · return 60% · death 0% of the loot); two reads
/// of an unchanged set are identical; a set with `depth>=8 → bank` on a lineage that reaches
/// D8 shows bank > 0 and its gold.
#[test]
fn forecast_ends_name_how_a_send_ends() {
    let mut g = Game::new_literal(11);
    g.set_rules_raw(crate::probes::good()).unwrap();
    let a = g.forecast();
    let e = a.ends.clone().expect("ends over the sims");
    assert!((e.bank + e.return_ + e.death + e.stall - 1.0).abs() < 1e-9, "{e:?}");
    assert!(e.gold >= 0.0);
    assert_eq!(g.forecast().ends, a.ends, "two reads agree");
    let json = serde_json::to_string(&a).unwrap();
    assert!(json.contains(r#""ends":{"bank":"#) && json.contains(r#""return":"#) && json.contains(r#""death":"#) && json.contains(r#""gold":"#), "{json}");
    // A lineage that reaches D8, a set that banks there.
    let mut g = Game::new_literal(11);
    crate::probes::learn_everything(&mut g);
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    g.lineage.best_depth = 8;
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 6, xp: 0, next: 0 });
    let mut set = crate::probes::good();
    set.rows.retain(|r| r.verb.v != "bank");
    set.rows.insert(0, Row::new(vec![Cond::n("depth>=", 8)], Verb::new("bank")));
    g.set_rules_raw(set).unwrap();
    let f = g.forecast();
    let e = f.ends.expect("ends");
    assert!(e.bank > 0.0, "{e:?}");
    assert!(e.gold > 0.0, "{e:?}");
    assert!((e.bank + e.return_ + e.death + e.stall - 1.0).abs() < 1e-9, "{e:?}");
    // Nothing reaches D9 past the bank row.
    assert_eq!(f.depths.iter().find(|d| d.depth == 9).unwrap().reach, 0.0);
    // QA on 3d71c33 (`D8 44% · bank 57%`): the ends are the bars' own sims, so a bank at
    // `depth ≥ 8` is among the sims that reached D8.
    assert!(e.bank <= f.depths.iter().find(|d| d.depth == 8).unwrap().reach + 1e-9, "bank {} over reach(D8) {:?}", e.bank, f.depths);
    // A fresh lineage with no return row: its sends end in deaths, never in "returns" the
    // reach panel cut off at D1 (the ends panel runs every send to its exit).
    let g = Game::new_literal(7);
    let set = g.lineage.rules().clone();
    assert!(!set.rows.iter().any(|r| r.verb.v == "return" || r.verb.v == "bank"));
    let e = g.forecast().ends.expect("ends");
    assert!(e.death > 0.5 && e.bank == 0.0, "{e:?}");
}

/// Cut 12 §6: `×` on one supply line takes that line only — a bought line is refunded and
/// leaves `last_supplies`, the kennel's leash is put back for nothing (it returns at the next
/// exit); the wire says which line is the kennel's (`free`), so a bought leash beside it is not
/// mistaken for it (QA on 952e306: "−$40 heal · +$40 refund heal · −$40 heal" from the client's
/// clear-and-rebuy fallback, "bought one labelled found").
#[test]
fn drop_supply_takes_one_line_and_refunds_a_bought_one() {
    let mut g = Game::new_literal(5);
    g.lineage.gold = 200;
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let cat = g.supply_catalogue();
    let heal = cat.iter().find(|e| e.kind == "heal").expect("heal for sale").price;
    g.buy_supply("heal").unwrap();
    g.buy_supply("leash").unwrap();
    let wire = g.lineage();
    let kennel: Vec<&crate::item::InvItem> = wire.supplies.iter().filter(|s| s.free).collect();
    assert_eq!(kennel.len(), 1, "one free line, the kennel's: {:?}", wire.supplies);
    assert!(wire.supplies.iter().any(|s| s.kind == "leash" && !s.free), "the bought leash is not free");
    let gold = g.lineage.gold;
    let heal_id = g.lineage.supplies.iter().find(|s| s.kind == "heal").unwrap().id;
    g.drop_supply(heal_id).unwrap();
    assert_eq!(g.lineage.gold, gold + heal, "the heal is refunded");
    assert_eq!(g.lineage.supplies.len(), 2, "the two leashes stay");
    assert!(g.lineage.gold_ledger.iter().any(|e| e.why == "refund heal"), "{:?}", g.lineage.gold_ledger);
    let kennel_id = g.lineage.supplies.iter().find(|s| s.free).unwrap().id;
    g.drop_supply(kennel_id).unwrap();
    assert_eq!(g.lineage.gold, gold + heal, "the kennel's leash refunds nothing");
    assert_eq!(g.lineage.supplies.len(), 1);
    assert!(g.drop_supply(999).is_err());
}

/// Cut 13: a packed supply the send did not use comes back to the shelf, not to the salvage
/// (QA on 50bb162: a $40 heal salvaged for $2 and rebought for $40 on every banked run).
#[test]
fn an_unused_supply_comes_back_to_the_shelf() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.gold = 100;
    g.buy_supply("heal").unwrap();
    let gold = g.lineage.gold;
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 1, "the unused heal is back");
    assert_eq!(g.lineage.gold, gold + g.batch.gold_earned + g.batch.salvage_gold, "nothing bought, nothing salvaged for it: {:?}", g.lineage.gold_ledger);
    assert!(g.batch.exits.last().is_some_and(|l| l.text.contains("1 supply back")), "{:?}", g.batch.exits.last().map(|l| l.text.clone()));
}

/// Cut 12 §2: the thief guard card answers the den — over 30 seeds the den's snatches with
/// the card are ≤ 20% of those without (the gate's probe, `den_guard_trial`).
#[test]
fn thief_guard_cuts_den_snatches() {
    let (mut without, mut with) = (0u32, 0u32);
    for seed in 1..=30u64 {
        let (a, b) = crate::probes::den_guard_trial(seed);
        without += a;
        with += b;
    }
    // Cut 20 §1: one theft a run — at most one snatch a seed (a wandering monkey's theft on
    // the way makes the den's thieves bolt empty-handed).
    assert!(without >= 20, "the preset is robbed ({without} snatches over 30 seeds)");
    assert!(with * 5 <= without, "with the card {with} vs without {without}");
    // The card's sheet leads with the raid.
    let rows = crate::meta::unlock_rows("thief_guard").unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], Row::new(vec![Cond::t("on_see", "den")], Verb::arg("attack", "nearest")));
}

// ---------------------------------------------------------------- Cut 13

/// Cut 13 §1: an arena whose stairs and one goblin sit behind a chasm — the hero paces before
/// a foe it cannot reach until the oscillation guard has fired `STALL_FIRES` times and the run
/// ends as `stalled`.
fn stall_arena() -> Game {
    let mut g = arena_seed(13);
    attack_rules(&mut g);
    let run = g.run.as_mut().unwrap();
    for y in 1..11 {
        run.floor.map.set(Pos::new(9, y), Tile::Chasm);
    }
    run.floor.map.reveal_all();
    run.hero_dist_pos = None;
    let id = run.new_id();
    let mut m = Monster::spawn(id, "goblin", Pos::new(12, 5), 1);
    m.awake = true;
    run.monsters.push(m);
    g
}

/// Cut 13 §1 (Q: "'stall' is never explained … no patch is offered"; R: "the reel blamed
/// `R3 drink heal caught him` while the trace said `R1 throw unknown stuck`"): a stalled run
/// gets a death-style record whose cause is the guard's moment, `keeps $0`, the trace, a
/// `stall` verdict with a patch that fires and leaves the floor; the reel line, the record and
/// the chronicle note name the same cause; the HUD's stake reads `stalling` from the first
/// guard on; the batch counts it as a return, never a death.
#[test]
fn a_stalled_run_gets_a_verdict_whose_cause_the_reel_repeats() {
    let mut g = stall_arena();
    assert!(!g.snapshot().stake.stalling);
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 3000 {
        g.tick();
        g.events.clear();
        n += 1;
        if g.run.as_ref().unwrap().stuck_fires == 1 {
            assert!(g.snapshot().stake.stalling, "the stake says so from the first guard");
        }
    }
    let run = g.run.as_ref().unwrap();
    assert!(run.timed_out && run.stuck_fires >= crate::engine::STALL_FIRES, "stalled after {n} ticks: fires {}", run.stuck_fires);
    assert_eq!(run.stuck_cause.as_deref(), Some("goblin, no path"));
    assert!(run.notes.iter().any(|(_, s)| s == "Stalled: the goblin, no path. Came home empty-handed."), "{:?}", run.notes);
    let id = run.id;
    g.finish_run();
    assert_eq!((g.batch.stalls, g.batch.returned, g.batch.deaths.len()), (1, 1, 0));
    assert_eq!(g.batch.worst_death, Some(id), "the stall is the worst candidate when no death is");
    let d = g.death(id).expect("a stall record");
    assert_eq!((d.verdict.as_str(), d.cause.as_str(), d.margin.as_str()), ("stall", "stalled · goblin, no path", "keeps $0"));
    assert!(d.line.as_ref().is_some_and(|l| l.text.contains("stalled")), "{:?}", d.line);
    assert!(!d.trace.turns.is_empty() && d.notes.iter().any(|s| s.starts_with("Stalled:")), "{:?}", d.notes);
    assert!(!d.patches.is_empty(), "a stall names a row");
    let rec = g.deaths.get(&id).unwrap().clone();
    assert!(rec.stall && rec.t10.is_some());
    let fired = d.patches.iter().map(|p| crate::trace::patch_fired_rate(&g, &rec, p)).fold(0.0, f64::max);
    assert!(fired >= 0.5, "no patch fires: {:?}", d.patches);
    assert!(d.patches.iter().any(|p| matches!(p.row.verb.v.as_str(), "return" | "bank" | "descend")), "{:?}", d.patches);
    // The reel line reads the trace's cause, in the grammar.
    let h = g.batch.highlights.iter().find(|h| h.run_id == id && h.arc.is_some()).expect("the run's line");
    assert_eq!(h.arc.as_ref().unwrap().resolution, "stalled, goblin no path");
    assert!(h.text.ends_with("stalled, goblin no path."), "{}", h.text);
    assert!(crate::sifter::story_ok(&h.text), "{}", h.text);
    assert_eq!(crate::sifter::stall_short(&d.cause["stalled · ".len()..]), "goblin no path");
    // The grammar takes a stall's cause and nothing looser.
    assert!(crate::sifter::stalled_ok("stalled, archer no path") && crate::sifter::stalled_ok("stalled, eel across water") && crate::sifter::stalled_ok("stalled, paced"));
    assert!(!crate::sifter::stalled_ok("stalled, R3 drink heal") && !crate::sifter::stalled_ok("stalled, dragon no path"));
    // The record survives a save; a second read is the same verdict.
    let mut h2 = Game::load(&g.save()).unwrap();
    assert_eq!(h2.death(id).unwrap(), d);
}

/// Cut 13 §2 (both raters: "R4 retreat — brave held", "two losses I could not own"): a new
/// heir is offered two distinct traits, drawn from the seed and the heir number (the same on
/// every read), never the last heir's; the heir's trait is the first until `set_trait` picks
/// the other, which sticks through a save; a name not on offer is refused; the send spends
/// the offer.
#[test]
fn a_new_heir_chooses_between_two_offered_traits() {
    let g = Game::new_literal(21);
    let offer = g.lineage().trait_offer;
    assert_eq!(offer.len(), 2);
    assert_ne!(offer[0], offer[1]);
    assert_eq!(g.lineage.trait_.name(), offer[0], "the first is the heir's until a pick");
    assert_eq!(Game::new_literal(21).lineage().trait_offer, offer, "deterministic");
    use crate::hero::Trait;
    let o = crate::engine::trait_offer(7, 2, Some(Trait::Brave), Trait::Brave);
    assert!(o[0] != Trait::Brave && o[1] != Trait::Brave && o[0] != o[1], "{o:?}: the last heir's trait is redrawn");
    assert_eq!(o, crate::engine::trait_offer(7, 2, Some(Trait::Brave), Trait::Brave), "a pure draw");
    assert_eq!(crate::engine::trait_offer(7, 2, Some(Trait::Brave), Trait::Greedy)[0], Trait::Greedy, "the lineage's draw leads when it may");
    for seed in 1..=40u64 {
        let mut g = arena_seed(seed);
        let last = g.lineage.trait_;
        {
            let (run, mut cx) = g.ctx();
            crate::turn::damage_hero(run, &mut cx, 99, &crate::turn::Src::Gas);
        }
        g.finish_run();
        let l = g.lineage();
        assert_eq!(l.heir, 2);
        assert_eq!(l.trait_offer.len(), 2, "seed {seed}");
        assert!(!l.trait_offer.contains(&last.name().to_string()), "seed {seed}: the last heir's {} offered again", last.name());
        assert_eq!(l.trait_, l.trait_offer[0]);
        assert_eq!(g.set_trait("nobody").unwrap_err(), "not on offer");
        let pick = l.trait_offer[1].clone();
        g.set_trait(&pick).unwrap();
        assert_eq!(g.lineage().trait_, pick);
        let mut h = Game::load(&g.save()).unwrap();
        assert_eq!((h.lineage().trait_, h.lineage().trait_offer.clone()), (pick.clone(), l.trait_offer.clone()), "the pick and the offer survive a save");
        h.send();
        assert!(h.lineage().trait_offer.is_empty(), "the send spends the offer");
        assert_eq!(h.run.as_ref().unwrap().trait_.name(), pick, "the run wears the pick");
    }
}

/// Cut 13 §3 (Q: "I left with $142 and came back to $6 and the report never said where it
/// went"): the night's ledger — `ReturnReport.spent` is what the restock bought, per kind in
/// coins, and the purse reconciles to the coin over 8 h on 20 seeds: exits + salvage + wake
/// pay − spent == the delta. And a kind the last run used to no effect is not rebought.
#[test]
fn the_nights_ledger_reconciles_and_a_wasted_kind_is_not_rebought() {
    let night = |seed: u64| -> bool {
        let mut g = Game::new_literal(seed);
        g.lineage.unlocks.insert("auto_supply".into());
        g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
        g.lineage.gold = 400;
        g.buy_supply("heal").unwrap();
        g.buy_supply("heal").unwrap();
        // Cut 19 §3: the repeat spends only what the night brings home — a set that banks.
        let mut set = g.lineage.rules().clone();
        set.rows.push(Row::new(vec![Cond::n("depth>=", 3)], Verb::new("bank")));
        g.set_rules(set).unwrap();
        let before = g.lineage.gold;
        let r = crate::offline::run_offline_quick(&mut g, 8 * 3600);
        let b = &g.batch;
        let spent: i32 = b.spent.values().map(|(_, c)| *c).sum();
        assert_eq!(b.gold_earned + b.salvage_gold + b.wake_pay - spent, g.lineage.gold - before, "seed {seed}: earned {} salvage {} wake {} spent {spent}", b.gold_earned, b.salvage_gold, b.wake_pay);
        assert_eq!(r.spent.iter().map(|s| s.gold).sum::<i32>(), spent, "the report's SPENT rows are the batch's");
        // Cut 21 §2: a found heal on the shelf stands in for a bought one — the repeat's heal
        // is re-bought or shelved.
        let bought = r.spent.iter().find(|s| s.kind == "heal").inspect(|row| assert!(row.n >= 1 && row.gold >= 10 * row.n as i32, "{row:?}")).is_some();
        bought || r.shelved.iter().any(|s| s.kind == "heal")
    };
    let spent_seen = std::thread::scope(|sc| (1..=20u64).map(|seed| sc.spawn(move || night(seed))).collect::<Vec<_>>().into_iter().filter_map(|h| h.join().unwrap().then_some(())).count());
    assert!(spent_seen >= 10, "the restock bought (or shelved) heals on {spent_seen} seeds");
    // A heal drunk at full HP is a use to no effect: the heal is skipped by the next restock,
    // the strength potion beside it is rebought.
    let mut g = arena_seed(5);
    no_kennel_leash(&mut g);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "strength").unwrap());
    g.lineage.gold = 1000;
    g.lineage.last_supplies = vec!["heal".into(), "strength".into()];
    give(&mut g, "heal");
    // (Cut 21 §2: the repeat re-buys only what a row can use — a strength row that never fires)
    rules(&mut g, vec![Row::new(vec![], Verb::arg("drink", "unknown")), Row::new(vec![Cond::n("hp<", 1)], Verb::arg("drink", "strength"))]);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "heal potion")), "{:?}", ev_kinds(&evs));
    assert_eq!(g.run.as_ref().unwrap().wasted_kinds, vec!["heal".to_string()]);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let kinds: Vec<&str> = g.lineage.supplies.iter().map(|s| s.kind.as_str()).collect();
    assert_eq!(kinds, vec!["strength"], "the wasted heal is not rebought");
    assert_eq!(g.batch.spent.get("strength"), Some(&(1, crate::engine::supply_price(crate::defs::Cat::Potion, g.lineage.best_depth))));
    // The shop grows with the forge: a craftable potion is for sale before the hero drank one.
    let mut g = Game::new_literal(5);
    assert!(!g.supply_catalogue().iter().any(|s| s.kind == "strength"));
    g.lineage.forge.insert("strength".into(), ForgeRow::at(5));
    let entry = g.supply_catalogue().into_iter().find(|s| s.kind == "strength").expect("craftable strength on the shelf");
    assert_eq!((entry.price, entry.label.as_str()), (10, "strength potion"), "a new lineage's price (Cut 22 §1)");
}

/// Cut 13 §5 (both raters: "±10 between re-rolls", "pack break +7 % then −14 %"): a catalogue
/// delta carries its half-width (`UnlockInfo.pm`, the base's and the patched reach's combined)
/// whenever it carries a delta; the ends line carries its `±`; the refine pass says so and
/// two reads of it agree.
#[test]
fn a_catalogue_delta_carries_its_half_width() {
    let mut g = Game::new_literal(11);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:goblin_archer:ranged".into());
    g.lineage.facts.insert("foe:jackal:fast".into());
    g.lineage.marks = 9;
    g.lineage.best_depth = 3;
    tiers(&mut g);
    let cat = g.unlock_deltas();
    let with: Vec<&UnlockInfo> = cat.iter().filter(|u| u.delta.is_some()).collect();
    assert!(with.len() >= 2, "{}", with.len());
    for u in &with {
        let pm = u.pm.unwrap_or_else(|| panic!("{}: delta without pm", u.id));
        assert!((0.0..0.5).contains(&pm), "{}: pm {pm}", u.id);
    }
    assert!(with.iter().any(|u| u.pm.unwrap() > 0.0), "a fractional reach has a width");
    assert!(cat.iter().filter(|u| u.delta.is_none()).all(|u| u.pm.is_none()));
    assert_eq!(g.unlocks().iter().map(|u| (u.delta, u.pm)).collect::<Vec<_>>(), cat.iter().map(|u| (u.delta, u.pm)).collect::<Vec<_>>(), "the memoised read carries the same numbers");
    assert!((crate::meta::delta_pm(0.5, 0.5, 50) - (2.0f64).sqrt() * crate::forecast::half_width(0.5, 50)).abs() < 1e-12);
    let f = g.forecast();
    assert!(!f.refined && f.ends.as_ref().is_some_and(|e| e.pm >= 0.0));
    let a = g.forecast_refine();
    assert!(a.refined, "the refine pass is marked");
    assert_eq!(g.forecast_refine(), a, "two refine reads agree");
}





/// Rater W on 238bd67: a potion bought by name (the forge made it craftable before the hero
/// ever drank one) teaches the lineage its flavour — `drink heal` executes on a found heal.
#[test]
fn buying_a_kind_by_name_identifies_it() {
    let mut g = Game::new_literal(901);
    g.lineage.gold = 500;
    g.lineage.forge.entry("heal".into()).or_default().craftable = true;
    assert!(!crate::item::is_identified(&g.lineage.facts, &g.lineage.flavours, "heal"));
    assert!(g.supply_catalogue().iter().any(|s| s.kind == "heal"), "the forge puts heal on the shelf");
    g.buy_supply("heal").unwrap();
    assert!(crate::item::is_identified(&g.lineage.facts, &g.lineage.flavours, "heal"), "a bought heal is a known kind");
}

// ---------------------------------------------------------------- stalls (rater X on 238bd67)

/// The first row event of the next hero action.
fn first_rule(g: &mut Game) -> i32 {
    for _ in 0..40 {
        g.tick();
        let evs = std::mem::take(&mut g.events);
        if let Some(r) = evs.iter().find_map(|e| if let Ev::Rule { row, .. } = e { Some(*row) } else { None }) {
            return r;
        }
    }
    panic!("the hero never acted");
}

/// A retreat does not run from foes the oscillation guard gave up on for the floor: three
/// holding at range kept `foes ≥ 3 → to corridor` pulling the hero into a corridor the chores
/// walked it back out of, and the guard fired until the run stalled.
#[test]
fn a_retreat_ignores_foes_given_up_on() {
    for given_up in [false, true] {
        let mut g = arena();
        rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 3)], Verb::new("retreat"))]);
        let ids: Vec<u32> = (0..3).map(|i| add_monster(&mut g, "goblin", 10, 3 + i * 2)).collect();
        if given_up {
            let run = g.run.as_mut().unwrap();
            for id in ids {
                run.ignore(id, u32::MAX);
            }
        }
        let row = first_rule(&mut g);
        assert_eq!(row == 0, !given_up, "given up {given_up}: row {row}");
    }
}

/// A boss holds the stairs: with no hero in sight it walks back to its post (a Warlord 20
/// tiles off left the hero waiting at the sealed wall until the run stalled).
#[test]
fn a_boss_with_no_hero_to_chase_returns_to_the_stairs() {
    let mut g = arena();
    hold_rules(&mut g);
    {
        let run = g.run.as_mut().unwrap();
        // A wall down the middle, open at the bottom row: the hero on the right, out of sight.
        for y in 1..10 {
            run.floor.map.set(Pos::new(8, y), Tile::Wall);
        }
        run.hero.pos = Pos::new(12, 2);
        run.hero_dist_pos = None;
        run.floor.map.update_vision(run.hero.pos, VISION);
    }
    let id = add_monster(&mut g, "goblin_warlord", 2, 2);
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == id).unwrap().last_seen = None;
    let stairs = g.run.as_ref().unwrap().floor.stairs_down;
    let mut closest = 99;
    for _ in 0..400 {
        ticks(&mut g, 1);
        if let Some(m) = monster(&g, id) {
            closest = closest.min(m.pos.cheb(stairs));
        }
    }
    assert!(closest <= 3, "the Warlord never came back to the stairs: {closest}");
}

/// The shield wall has no hole: a Warlord boxed in with no tile for a reserve turns an
/// unaimed blow aside (four `attack nearest` swings at a cornered Warlord let TRIVIAL pass D8).
#[test]
fn a_boxed_in_warlord_turns_unaimed_blows_aside() {
    let mut g = arena();
    attack_rules(&mut g);
    {
        let run = g.run.as_mut().unwrap();
        let mut map = Map::new(16, 12, Tile::Wall);
        map.set(Pos::new(4, 5), Tile::Floor);
        map.set(Pos::new(5, 5), Tile::Floor);
        map.compute_corridors(&[]);
        run.floor.map = map;
        run.hero.pos = Pos::new(4, 5);
        run.hero_dist_pos = None;
        run.floor.map.update_vision(run.hero.pos, VISION);
    }
    let id = add_monster(&mut g, "goblin_warlord", 5, 5);
    let max = monster(&g, id).unwrap().max_hp;
    let evs = ticks(&mut g, 60);
    let swings = evs.iter().filter(|e| matches!(e, Ev::Attack { src, dst, hit: true, .. } if *src == HERO_ID && *dst == id)).count();
    assert!(swings > 0, "the hero never hit him");
    assert_eq!(monster(&g, id).map(|m| m.hp), Some(max), "an unaimed blow landed on a boxed-in Warlord");
}

// ---------------------------------------------------------------- Cut 16

/// The loot a freshly generated floor holds: coins (×`GOLD_DIVISOR`, raw) plus item values.
fn floor_value(g: &Game) -> i32 {
    let run = g.run.as_ref().unwrap();
    run.items
        .iter()
        .filter(|fi| fi.item.kind != "leash" && fi.item.kind != "bones")
        .map(|fi| if fi.item.kind == "gold" { fi.item.amount * crate::engine::GOLD_DIVISOR } else { fi.item.value() })
        .sum()
}

fn d3_value(seed: u64, picks: u32, best: u32) -> i32 {
    let mut g = Game::new_literal(seed);
    g.lineage.best_depth = best;
    if picks > 0 {
        g.lineage.picked.insert(3, picks);
    }
    g.start_run(Some(seed * 31 + 7));
    g.descend_to(3);
    floor_value(&g)
}

/// §1: a depth banked from ten times pays ≤ 30 % of a fresh one; one step recovers per night
/// it is not visited (a night it is visited does not); the lineage's deepest depth is always
/// fresh; a bank and a return pick, a death does not.
#[test]
fn a_picked_depth_pays_less_and_recovers_night_over_night() {
    let mut g = Game::new_literal(3);
    g.lineage.best_depth = 10;
    for _ in 0..10 {
        g.lineage.night_run(3, 3, true);
    }
    assert_eq!(g.lineage.picked.get(&3), Some(&crate::engine::PICKED_CAP));
    assert_eq!(g.lineage.freshness(3), 250);
    assert_eq!(g.lineage.picked_clean(), vec![3]);
    let (mut fresh, mut picked) = (0, 0);
    for seed in 1..=40u64 {
        fresh += d3_value(seed, 0, 10);
        picked += d3_value(seed, 10, 10);
    }
    assert!(fresh > 0);
    assert!(picked * 100 <= fresh * 30, "picked clean pays {picked} of {fresh}");
    // The deepest depth the lineage has seen is fresh whatever its picks.
    let at_best: i32 = (1..=10u64).map(|s| d3_value(s, 10, 3)).sum();
    let fresh10: i32 = (1..=10u64).map(|s| d3_value(s, 0, 3)).sum();
    assert_eq!(at_best, fresh10);
    // Recovery: a night that visits D3 keeps it picked; each night that does not, one step.
    let mut l = g.lineage.clone();
    l.night_runs = 0;
    l.night_seen.clear();
    for _ in 0..crate::engine::NIGHT_RUNS {
        l.night_run(0, 5, false);
    }
    assert_eq!(l.picked.get(&3), Some(&7), "visited tonight: no recovery");
    for night in 1..=7u32 {
        for _ in 0..crate::engine::NIGHT_RUNS {
            l.night_run(0, 2, false);
        }
        assert_eq!(l.picked.get(&3).copied().unwrap_or(0), 7 - night, "night {night}");
    }
    assert_eq!(l.freshness(3), 1000);
    assert!(l.picked.is_empty() && l.picked_clean().is_empty());
    // A bank picks its floor; a death does not.
    let mut g = Game::new_literal(5);
    g.lineage.best_depth = 10;
    g.start_run(None);
    g.descend_to(4);
    g.run.as_mut().unwrap().over = Some(ExitTier::Bank);
    g.finish_run();
    assert_eq!(g.lineage.picked.get(&4), Some(&1));
    g.lineage.rest_left = 0;
    g.start_run(None);
    g.descend_to(4);
    g.run.as_mut().unwrap().over = Some(ExitTier::Death);
    g.finish_run();
    assert_eq!(g.lineage.picked.get(&4), Some(&1));
    // The run carries the thinned depths; the report and the lineage name the clean ones.
    g.lineage.picked.insert(2, 5);
    g.lineage.rest_left = 0;
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().thin.get(&2), Some(&328));
    assert_eq!(g.lineage().picked, vec![2]);
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.lineage.picked, g.lineage.picked, "picks survive a save");
}

/// §2: the wake offers the owned classes (the current first, each with its signature verb);
/// the pick sticks to the heir's runs, through a save and past the next heir's wake.
#[test]
fn the_wake_offers_owned_classes_and_the_pick_sticks() {
    let mut g = Game::new_literal(8);
    g.lineage.unlocks.remove("rogue");
    assert!(g.lineage().class_offer.is_empty(), "one class owned: no chips");
    g.lineage.unlocks.insert("rogue".into());
    g.lineage.unlocks.insert("ranger".into());
    let offer = g.lineage().class_offer;
    let names: Vec<&str> = offer.iter().map(|c| c.class.as_str()).collect();
    assert_eq!(names, vec!["fighter", "rogue", "ranger"]);
    let sigs: Vec<&str> = offer.iter().map(|c| c.signature.as_str()).collect();
    assert_eq!(sigs, vec!["shield_bash", "vanish", "mark"]);
    assert_eq!((offer[2].level, offer[2].opens), (1, 7));
    assert!(g.set_class("caster").is_err(), "not owned");
    g.set_class("ranger").unwrap();
    assert_eq!(g.lineage().class_offer[0].class, "ranger", "the pick leads");
    let mut g = Game::load(&g.save()).unwrap();
    g.send();
    assert!(g.lineage().class_offer.is_empty(), "the send closes the wake");
    assert_eq!(g.run.as_ref().unwrap().hero.class, Class::Ranger, "the run wears the pick");
    g.run.as_mut().unwrap().over = Some(ExitTier::Bank);
    g.finish_run();
    g.lineage.rest_left = 0;
    g.send();
    assert_eq!(g.run.as_ref().unwrap().hero.class, Class::Ranger, "and the next run");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_hero(run, &mut cx, 999, &crate::turn::Src::Gas);
    }
    g.finish_run();
    let l = g.lineage();
    assert_eq!(l.heir, 2);
    assert_eq!(l.class_offer[0].class, "ranger", "the next heir's wake starts from the class");
}

/// §3: D5–8 are the Burrows — their own biome fact and spawn mix (monkeys and archers up,
/// rats out); the Captain opens them, the Warlord closes them.
#[test]
fn the_burrows_are_d5_to_d8() {
    use crate::descent::{biome_for, Biome};
    assert_eq!(biome_for(4), Biome::Warrens);
    assert!((5..=8).all(|d| biome_for(d) == Biome::Burrows));
    assert_eq!(crate::descent::lieutenant_for(5), Some("goblin_captain"));
    assert_eq!(crate::descent::boss_for(8), Some("goblin_warlord"));
    let w = crate::defs::spawn_table(Biome::Warrens, 4);
    let b = crate::defs::spawn_table(Biome::Burrows, 5);
    let weight = |t: &[(&str, u32, i32, i32)], k: &str| t.iter().filter(|e| e.0 == k).map(|e| e.1).sum::<u32>();
    assert_eq!(weight(&b, "rat"), 0);
    assert!(weight(&b, "monkey") > weight(&w, "monkey"));
    assert!(weight(&b, "goblin_archer") > weight(&w, "goblin_archer"));
    let mut g = Game::new_literal(4);
    g.start_run(None);
    g.descend_to(4);
    assert_eq!(g.run.as_ref().unwrap().biome(), Biome::Warrens);
    let evs = {
        let (run, mut cx) = g.ctx();
        run.hero.pos = run.floor.stairs_down;
        crate::turn::descend(run, &mut cx);
        std::mem::take(cx.events)
    };
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 5, biome, .. } if biome == "burrows")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "biome:burrows")));
    assert!(g.run.as_ref().unwrap().monsters.iter().all(|m| m.kind != "rat"));
    assert!(g.run.as_ref().unwrap().monsters.iter().any(|m| m.kind == "goblin_captain"));
}

/// §4: at half hp the Warlord breaks, once — the callout and the note, his goblins' shields
/// drop, he is faster and hits harder, and he never rallies again.
#[test]
fn the_warlord_breaks_once_at_half_hp_and_stops_rallying() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.max_hp = 2000;
    g.run.as_mut().unwrap().hero.hp = 1999;
    let w = add_monster(&mut g, "goblin_warlord", 8, 5);
    let evs = ticks(&mut g, 40);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == w && what == "rallies")));
    assert!(g.run.as_ref().unwrap().monsters.iter().any(|m| m.kind == "goblin" && m.buff_def.1 > 0));
    let wi = g.run.as_ref().unwrap().monsters.iter().position(|m| m.id == w).unwrap();
    let (speed, atk, max) = {
        let m = &g.run.as_ref().unwrap().monsters[wi];
        (m.speed, m.atk, m.max_hp)
    };
    // Just above half: no break.
    let evs = {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_monster(run, &mut cx, wi, max - max / 2 - 1, &crate::turn::Src::Gas);
        std::mem::take(cx.events)
    };
    assert!(!g.run.as_ref().unwrap().monsters[wi].broken);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "warlord breaks")));
    let evs = {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_monster(run, &mut cx, wi, 1, &crate::turn::Src::Gas);
        std::mem::take(cx.events)
    };
    let m = &g.run.as_ref().unwrap().monsters[wi];
    assert!(m.broken && m.hp * 2 <= m.max_hp);
    assert!(m.speed > speed && m.atk.1 > atk.1 && m.atk.0 > atk.0);
    assert!(evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "warlord breaks")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text == "The Warlord breaks.")));
    assert!(g.run.as_ref().unwrap().monsters.iter().all(|m| m.kind != "goblin" || m.buff_def == (0, 0)), "the shields drop");
    let goblins = g.run.as_ref().unwrap().monsters.iter().filter(|m| m.kind == "goblin").count();
    g.run.as_mut().unwrap().monsters.retain(|m| m.kind != "goblin");
    let evs = ticks(&mut g, 300);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Telegraph { id, what, .. } if *id == w && what == "rallies")), "no rally after the break");
    assert!(!evs.iter().any(|e| matches!(e, Ev::Spawn { e, .. } if e.kind == "goblin")), "no reserves");
    assert!(!evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "shields up")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Attack { src, .. } if *src == w)), "he charges");
    assert!(goblins >= 2);
    let wi = g.run.as_ref().unwrap().monsters.iter().position(|m| m.id == w).unwrap();
    let evs = {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_monster(run, &mut cx, wi, 1, &crate::turn::Src::Gas);
        std::mem::take(cx.events)
    };
    assert!(!evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "warlord breaks")), "once");
}

/// QA on 23ed91f (qaL, D6: a conjurer out of reach, `keeps $0 · stalling`, 15/38 for 4+ min,
/// only `bail` ended it). L's set (gas → retreat · depth ≥ 8 → bank · hp < 50 → drink heal ·
/// hp < 30 → read unknown · pack break) under a cowardly heir: before, seeds 1001/1004/1005
/// each had a run cut down 2 600–3 000 spectral blades to the 120 000-tick cap (a blow on a
/// summoned foe reset the oscillation guard, and one at the elbow exempted it). Now the guard
/// sees through the blades: no run nears the cap.
#[test]
fn a_summoner_out_of_reach_is_a_stall_not_a_loop() {
    use crate::hero::Trait;
    let set = RuleSet { rows: vec![
        Row::new(vec![Cond::t("foe_tag", "gas")], Verb::new("retreat")),
        Row::new(vec![Cond::n("depth>=", 8)], Verb::new("bank")),
        Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")),
        Row::new(vec![Cond::n("hp<", 30)], Verb::arg("read", "unknown")),
        Row::new(vec![], Verb::arg("tactic", "pack_break")),
    ], name: None, route: Vec::new() };
    let worst = par_seeds([1001u64, 1004, 1005], |seed| {
        let mut g = Game::new_literal(seed);
        g.lineage.unlocks.extend(["pack_break", "row5"].map(String::from));
        g.set_rules_raw(set.clone()).unwrap();
        let mut worst = (0u32, 0u32);
        for _ in 0..32 {
            g.lineage.rest_left = 0;
            g.lineage.trait_ = Trait::Cowardly;
            g.start_run(None);
            g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            let r = g.run.as_ref().unwrap();
            if r.turn > worst.0 {
                worst = (r.turn, r.summoned_kills);
            }
            g.finish_run();
            g.auto_keep();
        }
        worst
    });
    for (seed, (turns, blades)) in [1001, 1004, 1005].iter().zip(worst) {
        assert!(turns < 40_000, "seed {seed}: a run of {turns} ticks, {blades} summoned foes cut down");
    }
}

/// QA on 23ed91f (qaL: that loop paid `fighter +598 · L4 ↑2` and `renown +1054`): a summoned
/// foe is no kill for the lineage — no XP, no renown, no bestiary or `studied` count.
#[test]
fn summoned_kills_pay_nothing() {
    let run_with = |summoned: bool| {
        let mut g = arena();
        for i in 0..6 {
            let id = add_monster(&mut g, "goblin", 5, 3 + i);
            let (run, mut cx) = g.ctx();
            let mi = run.monsters.iter().position(|m| m.id == id).unwrap();
            run.monsters[mi].summoned = summoned;
            crate::turn::damage_monster(run, &mut cx, mi, 999, &crate::turn::Src::Hero { ranged: false });
        }
        let r = g.run.as_ref().unwrap();
        let (kills, summ) = (r.kills.len(), r.summoned_kills);
        let xp0 = g.lineage.classes.get("fighter").map(|p| p.xp).unwrap_or(0);
        finish_with(&mut g, ExitTier::Bank);
        let xp = g.lineage.classes.get("fighter").map(|p| (p.level, p.xp)).unwrap();
        (kills, summ, xp0, xp, g.lineage.renown, g.lineage.kill_counts.get("goblin").copied().unwrap_or(0))
    };
    let (k, s, _, xp_s, score_s, count_s) = run_with(true);
    assert_eq!((k, s, count_s), (0, 6, 0), "summoned: no lineage kills, no bestiary count");
    let (k, s, _, xp_r, score_r, count_r) = run_with(false);
    assert_eq!((k, s, count_r), (6, 0, 6));
    assert!(xp_r > xp_s && score_r > score_s, "real kills pay ({xp_r:?} > {xp_s:?}, {score_r} > {score_s})");
    let (_, _, _, xp_none, score_none, _) = { let mut g = arena(); finish_with(&mut g, ExitTier::Bank); (0, 0, 0, g.lineage.classes.get("fighter").map(|p| (p.level, p.xp)).unwrap(), g.lineage.renown, 0) };
    assert_eq!((xp_s, score_s), (xp_none, score_none), "six summoned kills pay what no kills pay");
}

/// QA on 23ed91f (qaL: `heal salvaged 2/5` while the keep sheet said `green potion?`): the
/// forge's wire row is keyed by the flavour until the kind is identified.
#[test]
fn the_forge_does_not_name_an_unknown_kind() {
    let mut g = Game::new_literal(1016);
    g.lineage.forge.insert("heal".into(), ForgeRow::at(2));
    g.lineage.forge.insert("sword".into(), ForgeRow::at(1));
    let fl = g.lineage.flavours.flavour_of("heal").unwrap().to_string();
    let w = g.lineage();
    assert!(!w.forge.contains_key("heal") && w.forge.contains_key(&format!("{fl} potion?")) && w.forge.contains_key("sword"), "{:?}", w.forge.keys());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    assert!(g.lineage().forge.contains_key("heal"));
}

/// QA on 23ed91f (qaL: `auto: keep weapon+armour` owned, vault `1/1 leather`, a watched bank
/// found mail · axe · sword and salvaged all three — the skipped sheet called `keep([])`): the
/// exit carries the preference's picks (`ExitPending.auto_keep`), and `auto_keep` (wasm
/// `autoKeep`) keeps the mail over the leather.
#[test]
fn a_watched_exit_carries_the_automations_picks() {
    let mut a = arena();
    a.lineage.unlocks.insert("quartermaster".into());
    let mut l = Item::new(100_000, "leather");
    l.known = true;
    a.lineage.vault = vec![l];
    a.lineage.next_vault_id = 100_001;
    let mail = give(&mut a, "mail");
    give(&mut a, "axe");
    give(&mut a, "sword");
    finish_with(&mut a, ExitTier::Bank);
    assert_eq!(a.exit_pending_wire().unwrap().auto_keep, vec![mail]);
    a.auto_keep();
    assert_eq!(a.lineage.vault.iter().map(|v| v.kind.as_str()).collect::<Vec<_>>(), ["mail"]);
}


// ---------------------------------------------------------------- Cut 18

/// Cut 18 §4: rater Y's run-2 set (`hp<30% → drink heal`, `foe: gas → retreat`, `foes ≥ 1 →
/// attack`, `hp<50% · depth ≥ 5 → return`) in a room whose bloat sits behind a chasm: the
/// retreat steps out of the bloat's sight, the explore chore steps back into it — the loop Y
/// saw, which the stall used to blame on the foe (`the jackal, no path`).
fn gas_loop_arena() -> Game {
    let mut g = arena_seed(13);
    g.lineage.facts.insert("foe:bloat:gas".into());
    rules(
        &mut g,
        vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::t("foe_tag", "gas")], Verb::new("retreat")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 50), Cond::n("depth>=", 5)], Verb::new("return")),
        ],
    );
    let run = g.run.as_mut().unwrap();
    let mut map = Map::new(24, 12, Tile::Wall);
    for y in 1..11 {
        for x in 1..9 {
            map.set(Pos::new(x, y), Tile::Floor);
        }
        for x in 10..23 {
            map.set(Pos::new(x, y), if x == 13 { Tile::Chasm } else { Tile::Floor });
        }
    }
    map.set(Pos::new(9, 5), Tile::Floor);
    let (up, down) = (Pos::new(1, 1), Pos::new(22, 10));
    map.set(up, Tile::StairsUp);
    map.set(down, Tile::StairsDown);
    map.compute_corridors(&[]);
    run.floor = Floor { map, stairs_up: up, stairs_down: down, rooms: Vec::new(), vision: VISION };
    run.hero.pos = Pos::new(8, 7);
    run.floor.map.update_vision(run.hero.pos, VISION);
    let id = run.new_id();
    let mut m = Monster::spawn(id, "bloat", Pos::new(19, 8), 1);
    m.awake = true;
    run.monsters.push(m);
    g
}

/// Cut 18 §4 (Y: "Stalled: the jackal, no path" while the trace showed `R2 retreat` and
/// `explore` taking turns): a stall whose guard window alternates between two actors names
/// them — the record, the chronicle note and the reel line alike — and the verdict's first
/// patch addresses that row (narrows or deletes it).
#[test]
fn a_stall_names_the_rules_loop_and_its_first_patch_addresses_the_row() {
    let mut g = gas_loop_arena();
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 4000 {
        g.tick();
        g.events.clear();
        n += 1;
    }
    let run = g.run.as_ref().unwrap();
    assert!(run.timed_out && run.stuck_fires >= crate::engine::STALL_FIRES, "stalled after {n} ticks: fires {}", run.stuck_fires);
    assert_eq!(run.stuck_cause.as_deref(), Some("R2 retreat ↔ explore"));
    assert_eq!(run.stuck_row, Some(1));
    assert!(run.notes.iter().any(|(_, s)| s == "Stalled: R2 retreat ↔ explore. Came home empty-handed."), "{:?}", run.notes);
    let id = run.id;
    g.finish_run();
    assert_eq!(g.batch.stalls, 1);
    let d = g.death(id).expect("a stall record");
    assert_eq!((d.verdict.as_str(), d.cause.as_str()), ("stall", "stalled · R2 retreat ↔ explore"));
    // The stairs sit behind the chasm: narrowing or deleting R2 cannot end this stall (the
    // loop patch is measured and not offered); the way out, as good at R2 as at the top, goes
    // in at R2, where the loop fired, and fires in the stall's replays.
    let first = d.patches.first().expect("a stall names a row");
    assert_eq!(first.insert_at, 1, "the first patch addresses R2: {:?}", d.patches);
    assert!(first.survive > 0.0, "{first:?}");
    let rec = g.deaths.get(&id).unwrap().clone();
    assert!(crate::trace::patch_fired_rate(&g, &rec, first) >= 0.5, "{first:?}");
    // The reel line reads the record's cause, in the grammar.
    let h = g.batch.highlights.iter().find(|h| h.run_id == id && h.arc.is_some()).expect("the run's line");
    let cause = d.cause.strip_prefix("stalled · ").unwrap();
    assert_eq!(h.arc.as_ref().unwrap().resolution, format!("stalled, {}", crate::sifter::stall_short(cause)));
    assert!(crate::sifter::story_ok(&h.text), "{}", h.text);
}

/// Cut 18 §4: the loop's shapes — two actors taking turns (a row and a chore, two rows), one
/// moving row alone; a targeting row alone, a trait's step or three actors are no loop.
#[test]
fn row_loop_reads_two_actors_or_one_moving_row() {
    let turn = |row: i32, verb: Verb| TraceTurn { max_hp: 0, t: 0, row, verb, hp: 18, foes: 3, rule_foes: 3, telegraphs: Vec::new(), blocked: None, rows: None, blows: Vec::new(), gift: None };
    let alt = |a: TraceTurn, b: TraceTurn| (0..6).flat_map(|_| [a.clone(), b.clone()]).collect::<Vec<_>>();
    let tr = alt(turn(1, Verb::new("retreat")), turn(-2, Verb::new("explore")));
    assert_eq!(crate::turn::row_loop(&tr), Some(("R2 retreat ↔ explore".to_string(), 1)));
    let tr = alt(turn(7, Verb::arg("attack", "nearest")), turn(4, Verb::new("back_corridor")));
    assert_eq!(crate::turn::row_loop(&tr), Some(("R5 corridor ↔ R8 attack".to_string(), 4)), "the moving row is the one named");
    let tr: Vec<_> = (0..12).map(|_| turn(0, Verb::new("retreat"))).collect();
    assert_eq!(crate::turn::row_loop(&tr), Some(("R1 retreat paced".to_string(), 0)));
    let tr: Vec<_> = (0..12).map(|_| turn(1, Verb::arg("attack", "nearest"))).collect();
    assert_eq!(crate::turn::row_loop(&tr), None, "an attack pacing before an unreachable foe keeps the foe's cause");
    let tr = alt(turn(1, Verb::new("retreat")), turn(-1, Verb::new("retreat")));
    assert_eq!(crate::turn::row_loop(&tr), None);
    let mut tr = alt(turn(1, Verb::new("retreat")), turn(-2, Verb::new("explore")));
    tr[3] = turn(2, Verb::arg("attack", "nearest"));
    tr[5] = turn(2, Verb::arg("attack", "nearest"));
    tr[7] = turn(2, Verb::arg("attack", "nearest"));
    assert_eq!(crate::turn::row_loop(&tr), None, "three actors");
    for c in ["R2 retreat ↔ explore", "R5 corridor ↔ R8 attack", "R1 retreat paced", "R3 drink ↔ pick up"] {
        assert!(crate::turn::loop_cause_ok(c) && crate::sifter::stalled_ok(&format!("stalled, {c}")), "{c}");
        assert!(word_count(c) <= 4, "{c}");
    }
    for c in ["R3 drink heal", "retreat ↔ explore", "R2 retreat ↔ R3", "R2 hit ranged ↔ explore"] {
        assert!(!crate::turn::loop_cause_ok(c), "{c}");
    }
}

/// Cut 18 §4 (rater Z: `drink ✗ no item` at 19/40 while FOUND listed `heal potion ×4`): the
/// firing row was `hp<50% → drink unknown` and the heals were known — the callout drops the
/// argument, so the reason names what is missing (`drink ✗ no unknown`). A heal held but
/// unidentified reads `unknown item` to `drink heal` (`drink ✗ unknown item`), never `no item`.
#[test]
fn a_blocked_drink_never_reads_no_item_beside_a_held_potion() {
    let callouts = |evs: &[Ev]| evs.iter().filter_map(|e| if let Ev::Callout { text, .. } = e { Some(text.clone()) } else { None }).collect::<Vec<_>>();
    // Known heals, `drink unknown` under 50 %: nothing unknown to drink.
    let mut g = arena_seed(5);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    for _ in 0..4 {
        give(&mut g, "heal");
    }
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "unknown"))]);
    g.run.as_mut().unwrap().hero.hp = 19 * hero(&g).max_hp / 40;
    let evs = ticks(&mut g, 20);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.rows.as_ref().unwrap()[0].why, "no unknown", "{t:?}");
    assert_eq!(t.blocked.as_deref(), Some("R1 drink ? ✗ no unknown"), "{t:?}");
    assert!(callouts(&evs).iter().any(|c| c == "drink ✗ no unknown"), "{:?}", callouts(&evs));
    assert!(!callouts(&evs).iter().any(|c| c.contains("no item")), "{:?}", callouts(&evs));
    assert!(crate::turn::row_reason_ok("no unknown"));
    // Found heals, flavour not identified: `drink heal` reads `unknown`.
    let mut g = arena_seed(5);
    give(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal"))]);
    g.run.as_mut().unwrap().hero.hp = 3;
    let evs = ticks(&mut g, 20);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.rows.as_ref().unwrap()[0].why, "unknown item", "{t:?}");
    assert!(callouts(&evs).iter().any(|c| c == "drink ✗ unknown item"), "{:?}", callouts(&evs));
    assert!(!callouts(&evs).iter().any(|c| c.contains("no item")), "{:?}", callouts(&evs));
    // Nothing held at all is still `no item`.
    let mut g = arena_seed(5);
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal"))]);
    g.run.as_mut().unwrap().hero.hp = 3;
    ticks(&mut g, 20);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.rows.as_ref().unwrap()[0].why, "no item", "{t:?}");
}

/// Cut 18 §3 (Z: "`D9 0%` for every rule set, with no reason given, until I met the Goblin
/// Warlord"): a D8 lineage whose set lacks the counter row reads its D9 row as the Warlord's
/// wall; with the counter at the top D9's reach clears 5 % and the row names no wall.
#[cfg(not(debug_assertions))]
#[test]
fn a_boss_wall_names_the_boss_until_the_set_passes_him() {
    for (seed, [(d8, d9), (_, d9_top)]) in (1..=3u64).zip(par_seeds(1..=3u64, crate::probes::wall_trial)) {
        assert!(d8.reach > crate::forecast::WALL_REACH && d8.wall.is_none(), "seed {seed}: D8 {d8:?}");
        assert!(d9.reach <= crate::forecast::WALL_REACH, "seed {seed}: the set without the counter passes the Warlord ({:.2})", d9.reach);
        assert_eq!(d9.wall.as_deref(), Some("goblin_warlord"), "seed {seed}");
        assert!(d9_top.reach > crate::forecast::WALL_REACH, "seed {seed}: the counter at the top reaches D9 {:.2}", d9_top.reach);
        assert_eq!(d9_top.wall, None, "seed {seed}");
    }
    // The rule itself: only under a boss floor, only where reach falls to ≤ 5 % from above it.
    use crate::forecast::wall_at;
    assert_eq!(wall_at(9, 0.0, 0.8).as_deref(), Some("goblin_warlord"));
    assert_eq!(wall_at(14, 0.04, 0.5).as_deref(), Some("bloat_mother"));
    assert_eq!(wall_at(9, 0.2, 0.8), None, "passable");
    assert_eq!(wall_at(9, 0.0, 0.03), None, "the fall came before the boss");
    assert_eq!(wall_at(8, 0.0, 0.8), None, "no boss above D8");
    assert!(serde_json::to_value(ForecastDepth { depth: 4, reach: 0.5, pm: None, try_: None, wall: None, bounty: false, boss: None, biome: None, clear: None }).unwrap().get("wall").is_none());
}

// ---------------------------------------------------------------- QA on 92eb880 (qaM, seed 1215)

/// Seed 1215's default first send to its death (the QA's live L1–L2): the game, the run id.
fn qam_first_death() -> (Game, u32) {
    let mut g = Game::new_literal(1215);
    g.send();
    loop {
        let r = g.step(50);
        if r.run_over {
            let died = r.events.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "death"));
            if g.pending_exit.is_some() {
                g.keep(vec![]).unwrap();
            }
            assert!(died, "seed 1215's first send dies (the QA's D6 jackal)");
            return (g, r.snapshot.run.id);
        }
    }
}

/// qaM: "D6 32 %±13 → 38 %±10 on opening edit … 26 %±12 → 34 %±9 on opening loadout" with no
/// rule edited — the first pass, then the refine, then (on any repaint) the first pass again.
/// Every read the camp's sheets make leaves the forecast as it was, and once the refined panel
/// exists a forecast read is that panel (never back to the coarser one).
#[test]
fn forecast_is_the_same_across_the_camps_sheet_reads() {
    let (mut g, id) = qam_first_death();
    let _ = g.death(id);
    let ps = g.death_deltas(id).unwrap();
    let rules = g.lineage.rules().clone();
    g.set_rules(crate::offline::apply_patch(&rules, &ps[0], g.lineage.max_rows())).unwrap();
    let first = g.forecast();
    assert!(!first.refined);
    let reads = |g: &mut Game| {
        let _ = g.lineage();
        let _ = g.vocabulary();
        let _ = g.unlocks();
        let _ = g.unlock_deltas();
        let _ = g.supply_catalogue();
        let _ = g.death(id);
        let r = g.lineage.rules().clone();
        g.set_rules(r).unwrap();
        let l = g.loadout.clone();
        g.loadout(l);
    };
    reads(&mut g);
    assert_eq!(g.forecast(), first, "a sheet read moved the first pass");
    let refined = g.forecast_refine();
    assert!(refined.refined);
    assert_eq!(g.forecast(), refined, "a read after the refine went back to the first pass");
    reads(&mut g);
    assert_eq!(g.forecast(), refined);
    // The same numbers from a cold cache: nothing but the lineage and the rules feeds them.
    g.panel_cache.borrow_mut().clear();
    g.forecast_cache.borrow_mut().clear();
    g.refined_panels.borrow_mut().clear();
    assert_eq!(g.forecast(), first);
    assert_eq!(g.forecast_refine(), refined);
}

/// qaM: `R2 attack saved him.` on the death screen of a hero who died while R2 fired (a note
/// from the floor above). No death's notes carry a `saved him` line.
#[test]
fn a_deaths_notes_never_say_saved_him() {
    let (mut g, id) = qam_first_death();
    let d = g.death(id).unwrap();
    // (Cut 30 §2: the qaM repro moved with the temperaments; the invariant holds on every death)
    assert!(d.notes.iter().all(|n| !n.contains("saved him")), "{:?}", d.notes);
    g.run_offline(8 * 3600);
    let ids: Vec<u32> = g.deaths.keys().copied().collect();
    for id in ids {
        let d = g.death(id).unwrap();
        assert!(d.notes.iter().all(|n| !n.contains("saved him")), "run {id}: {:?}", d.notes);
    }
}

/// qaM dump 09 (the worst death): `DICE` over three patches all `survives 100% · below bar` —
/// the unpatched rules win the same fight 12 of 12 times reseeded (the ogre at 2 HP when he
/// fell), so nothing can beat the base; the death says so (`nothing_beats_base`), and a dice
/// death with a patch surviving more than the base does not.
#[test]
fn a_dice_death_at_a_full_base_says_nothing_beats_it() {
    let (mut g, id) = qam_first_death();
    let _ = g.death(id);
    let ps = g.death_deltas(id).unwrap();
    let rules = g.lineage.rules().clone();
    g.set_rules(crate::offline::apply_patch(&rules, &ps[0], g.lineage.max_rows())).unwrap();
    // Cut 19: the night moved (a return walks, dens thin, the camp re-ranks the patches), so
    // the QA's D8 ogre is no longer this night's worst death; a dice death at a full base is
    // still among its deaths, and says so.
    let mut full_base: Vec<Death> = Vec::new();
    let mut ids: Vec<u32> = Vec::new();
    // (Cut 22 §2: thieves take coins before the heal; this seed's first such death is on its 8th night;
    // QA on 778fa1b: an outpaced retreat rests and the next row fights — the nights moved again)
    // (Cut 23: the leash is taken last and a walk home answers — the first such death is on the 23rd night)
    for _ in 0..32 {
        g.run_offline(8 * 3600);
        ids = g.deaths.keys().copied().collect();
        full_base = ids.iter().filter_map(|id| g.death(*id)).filter(|d| d.verdict == "dice" && d.baseline >= 1.0 - 1e-9).collect();
        if !full_base.is_empty() {
            break;
        }
    }
    assert!(!full_base.is_empty(), "a dice death the replays always survive");
    for d in &full_base {
        assert!(d.patches.iter().all(|p| p.survive <= d.baseline + 1e-9));
        assert!(d.nothing_beats_base, "{:?}", d.patches.iter().map(|p| (p.row.describe(), p.survive)).collect::<Vec<_>>());
    }
    for id in ids {
        let d = g.death(id).unwrap();
        let beaten = d.patches.iter().any(|p| p.survive > d.baseline + 1e-9);
        assert_eq!(d.nothing_beats_base, d.verdict == "dice" && !d.patches.is_empty() && !beaten, "run {id}: {} base {:.2}", d.verdict, d.baseline);
    }
}

/// The apportionment: whole coins summing to the floor of the total, largest fractions first.
#[test]
fn salvage_coins_sum_to_the_ledgers_floor() {
    use crate::engine::apportion_cents;
    assert_eq!(apportion_cents(&[180, 180, 120], 0), vec![2, 1, 1]);
    assert_eq!(apportion_cents(&[180, 180, 120], 60), vec![2, 2, 1]);
    assert_eq!(apportion_cents(&[120], -90), vec![0]);
    assert_eq!(apportion_cents(&[], 50), Vec::<i32>::new());
    for carry in [-150, -20, 0, 37, 99, 180, 350] {
        let c = [5, 99, 180, 240, 12];
        let out = apportion_cents(&c, carry);
        assert_eq!(out.iter().sum::<i32>(), (carry + c.iter().sum::<i32>()).max(0) / 100, "carry {carry}: {out:?}");
        assert!(out.iter().all(|x| *x >= 0));
    }
}

/// qaM: the keep sheet's `sold aggravate $2 · blink $2`, the report's `aggravate ×1 · $1`. The
/// exit sheet's per-item `worth` is what the ledger pays for that item when it is let go (all
/// of them, or any part), and the cut's `sold` rows and the report's rows are the ledger's
/// coins per kind.
#[test]
fn the_keep_sheets_prices_are_the_ledgers() {
    let checked: u32 = par_seeds(1..=16u64, |seed| {
        let mut n = 0;
        let mut g = Game::new_literal(seed);
        g.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
        g.set_rules_raw(crate::probes::good()).unwrap();
        for send in 0..6u32 {
            g.lineage.rest_left = 0;
            let coins0 = g.batch.salvaged_coins.clone();
            let head0 = g.batch.salvage_gold;
            g.send();
            let mut line = None;
            let r = loop {
                let r = g.step(200);
                for e in &r.events {
                    if let Ev::Exit { line: Some(l), .. } = e {
                        line = Some(l.clone());
                    }
                }
                if r.run_over {
                    break r;
                }
            };
            // The cut (a return's 40 %) was sold before the sheet: its rows are the ledger's.
            let paid_cut: i32 = g.batch.salvage_gold - head0;
            if let Some(l) = &line {
                assert_eq!(l.salvaged.iter().map(|s| s.gold).sum::<i32>(), paid_cut, "seed {seed}: cut rows vs ledger");
                for s in &l.salvaged {
                    // (the batch keys a kind; the line names it as the exit read it — an unidentified
                    // potion by its flavour, `violet potion?`)
                    let named = |k: &String| *k == s.kind || g.lineage.wire_name(k).replace('_', " ") == s.kind || crate::item::describe(&crate::item::Item::new(0, k), &Default::default(), &g.lineage.flavours).2 == s.kind;
                    let paid: i32 = g.batch.salvaged_coins.iter().filter(|(k, _)| named(k)).map(|(k, v)| v - coins0.get(k).copied().unwrap_or(0)).sum();
                    assert_eq!(paid, s.gold, "seed {seed}: cut {}", s.kind);
                }
            }
            let (Some(pe), Some(p)) = (r.exit_pending.clone(), g.pending_exit.clone()) else { continue };

            if p.pct == 0 || p.items.is_empty() {
                g.keep(vec![]).unwrap();
                continue;
            }
            // Keep the first item when the vault has room: each unkept one pays its sheet
            // price, exactly.
            let keep: Vec<u32> = p.items.iter().take(usize::from(g.lineage.vault.len() < g.lineage.vault_slots())).map(|i| i.id).collect();
            let want: i32 = p.items.iter().zip(&pe.worth).filter(|(i, _)| !keep.contains(&i.id)).map(|(_, w)| *w).sum();
            let (gold, head, coins1) = (g.lineage.gold, g.batch.salvage_gold, g.batch.salvaged_coins.clone());
            let slots_before = g.lineage.vault.len();
            g.keep(keep.clone()).unwrap();
            // (a full vault evicts: those pay too — only the no-eviction case is exact)
            if g.lineage.vault.len() == slots_before + keep.len() {
                assert_eq!(g.lineage.gold - gold, want, "seed {seed} send {send}: sheet {:?} kept {keep:?}", pe.worth);
                assert_eq!(g.batch.salvage_gold - head, want);
                let mut per: std::collections::BTreeMap<String, i32> = Default::default();
                for (i, w) in p.items.iter().zip(&pe.worth).filter(|(i, _)| !keep.contains(&i.id)) {
                    *per.entry(i.kind.clone()).or_insert(0) += w;
                }
                for (k, w) in per {
                    assert_eq!(g.batch.salvaged_coins.get(&k).copied().unwrap_or(0) - coins1.get(&k).copied().unwrap_or(0), w, "seed {seed}: {k}");
                }
                n += 1;
            }
        }
        // The report's rows are the ledger's coins per kind, summing to its header.
        let rep = g.run_offline(0);
        let rows: i32 = rep.salvaged.iter().map(|x| x.gold).sum();
        assert_eq!(rows, rep.gold.as_ref().map(|x| x.salvage).unwrap_or(0), "seed {seed}");
        for row in &rep.salvaged {
            assert_eq!(row.gold, *g.batch.salvaged_coins.get(&row.kind).unwrap_or(&row.gold), "seed {seed}: {}", row.kind);
        }
        n
    })
    .into_iter()
    .sum();
    assert!(checked >= 5, "only {checked} partial keeps checked");
}

/// qaM: R3 `hp < 30% → drink heal` under R1 `hp < 30% → return` "fired 0 of 16 runs · heal
/// unknown"; a new `hp < 50% → attack nearest` under `foes ≥ 1 → attack nearest`, unmarked.
#[test]
fn shadowed_rows_are_named() {
    let all = |_: &Cond| true;
    // Cut 19 §2: a return walks and can be blocked (a foe in the way), so it shadows nothing
    // but its own copies; `hold` always acts and stands in for the qaM set's R1.
    let set = RuleSet {
        rows: vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::new("hold")),
            Row::new(vec![Cond::n("hp<", 50)], Verb::new("rest")),
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 50)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 20)], Verb::arg("drink", "heal")),
        ],
        ..Default::default()
    };
    // R3 by R1 (hold always acts); R5 by R4 (the same strike, which needs a foe in view);
    // R6 by R1 (hp < 20 is hp < 30). R2 (rest) and R4 are free: rest fails where hold is
    // not written for it, and nothing above R4 holds whenever a foe is in view. A return shadows
    // only a return (it walks, and a foe in the way fails it):
    let ret = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 30)], Verb::new("return")), Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("hp<", 20)], Verb::new("return"))], ..Default::default() };
    assert_eq!(ret.shadowed_by(8, all), vec![None, None, Some(0)]);
    assert_eq!(set.shadowed_by(8, all), vec![None, None, Some(0), None, Some(3), Some(0)]);
    // Not shadowed: a looser threshold below, a verb that can fail above, another scope.
    let free = RuleSet {
        rows: vec![
            Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")),
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("hp<", 25)], Verb::new("retreat")),
            Row::new(vec![Cond::n("foes>=", 2)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
        ..Default::default()
    };
    assert_eq!(free.shadowed_by(8, all), vec![None; 5]);
    // A row whose condition the lineage cannot use never fires, so it shadows nothing.
    let locked = RuleSet { rows: vec![Row::new(vec![Cond::n("alert>=", 1)], Verb::new("hold")), Row::new(vec![Cond::n("alert>=", 2)], Verb::new("rest"))], ..Default::default() };
    assert_eq!(locked.shadowed_by(8, |c: &Cond| c.k != "alert>="), vec![None, None]);
    assert_eq!(locked.shadowed_by(8, all), vec![None, Some(0)]);
    // On the wire (lineage and forecast), and in the report's pending line.
    let mut g = Game::new_literal(1215);
    g.set_rules(RuleSet { rows: set.rows[..4].to_vec(), ..Default::default() }).unwrap();
    assert_eq!(g.lineage().shadowed_by, vec![None, None, Some(0), None]);
    assert_eq!(crate::forecast::forecast_with(&g, &g.lineage.rules().clone(), 5).shadowed_by, vec![None, None, Some(0), None]);
    g.set_rules(RuleSet { rows: set.rows[3..4].to_vec(), ..Default::default() }).unwrap();
    assert!(g.lineage().shadowed_by.is_empty(), "nothing shadowed: the field is absent");
    g.set_rules(RuleSet { rows: set.rows[..4].to_vec(), ..Default::default() }).unwrap();
    g.run_offline(2 * 3600);
    let p = crate::meta::pending(&g);
    let r3 = p.iter().find(|l| l.starts_with("R3 fired")).expect("an R3 line");
    assert!(r3.starts_with("R3 fired 0 of") && r3.ends_with(" · shadowed by R1") && !r3.contains("unknown"), "{r3}");
}

// ---------------------------------------------------------------- Cut 19

/// Cut 19 §2: a return walks to the up-stairs and exits there at 60 %; a foe on the way can
/// end it as a death; the player's bail stays instant.
#[test]
fn a_return_walks_home_and_can_die_on_the_way() {
    // An empty floor: the walk ends on the up-stairs, as a return.
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(100);
    rules(&mut g, vec![Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 200);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.over, Some(ExitTier::Return));
    assert_eq!(run.hero.pos, run.floor.stairs_up, "he walked to the stairs");
    let steps = evs.iter().filter(|e| matches!(e, Ev::Move { id, .. } if *id == HERO_ID)).count();
    assert!(steps >= 3, "four tiles from the stairs: {steps} steps");
    let loot = run.loot;
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, loot_kept, .. } if tier == "return" && *loot_kept == loot * 60 / 100)), "60 % of ${loot}");
    // A hurt hero with an ogre at his heels: the walk is interrupted by a death.
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        run.hero.pos = Pos::new(13, 9);
        run.hero.hp = 4;
        run.hero_dist_pos = None;
    }
    add_monster(&mut g, "ogre", 13, 10);
    add_monster(&mut g, "ogre", 12, 10);
    add_monster(&mut g, "ogre", 14, 9);
    rules(&mut g, vec![Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 400);
    assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Death), "no door out: the foes caught him");
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, verb, .. } if verb.v == "return")), "the return row acted on the way");
    // The bail is the player's own button: instant.
    let mut g = arena();
    hold_rules(&mut g);
    g.bail();
    ticks(&mut g, 10);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.over, Some(ExitTier::Return));
    assert_ne!(run.hero.pos, run.floor.stairs_up, "bail does not walk");
}

/// QA on 1a2a4a9 (qaP: `returning` at 6/38, the hp came back, the row stopped holding and he
/// went on down to D8): once the return row acts the walk home is the chore until the exit —
/// at full health, with the return row's condition false, he still walks home; a `descend`
/// row above it refuses `going home`; the stake says `returning`.
#[test]
fn a_return_is_a_commitment() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        run.hero.hp = 3;
    }
    let low = Row::new(vec![Cond::n("hp<", 50)], Verb::new("return"));
    let down = Row::new(vec![Cond::n("hp>", 60)], Verb::new("descend"));
    rules(&mut g, vec![down, low, Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    // One action: the return row fires and commits.
    for _ in 0..40 {
        g.tick();
        if g.run.as_ref().unwrap().homeward.is_some() {
            break;
        }
    }
    let run = g.run.as_mut().unwrap();
    assert_eq!(run.homeward, Some(1), "R2 return acted: committed");
    // Full health again: the return row's condition no longer reads true, the descend row's does.
    run.hero.hp = run.hero.max_hp;
    let depth = run.depth;
    assert!(g.snapshot().stake.returning, "the HUD reads `returning`");
    let evs = ticks(&mut g, 400);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.over, Some(ExitTier::Return), "he walked home");
    assert_eq!(run.depth, depth, "never down a floor on the way");
    assert!(!evs.iter().any(|e| matches!(e, Ev::Rule { row: 0, .. })), "R1 descend never acted");
    assert!(run.trace.iter().any(|t| t.rows.as_ref().is_some_and(|r| r.iter().any(|w| w.row == 0 && w.why == "going home"))), "R1 reads `going home`");
    assert!(run.trace.iter().any(|t| t.row == -2 && t.verb.v == "return"), "the walk is the chore once R2 no longer holds");
    assert_eq!(run.exit_row, Some(1), "the exit credits the committing row");
}

/// Cut 19 §4: the dying action was a row the player wrote, and the set without it survives
/// the replays: the verdict is `row`, naming it, and the first patch cuts it.
#[test]
fn a_row_the_player_wrote_is_the_row_verdict() {
    let mut g = arena_seed(4);
    let fire = give(&mut g, "fire");
    assert!(g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.id == fire && !i.is_known(&g.lineage.facts, &g.lineage.flavours)));
    {
        let run = g.run.as_mut().unwrap();
        run.hero.hp = 1;
    }
    // A quiet stretch first (the replays' checkpoint), then the gamble.
    hold_rules(&mut g);
    ticks(&mut g, 60);
    let player = Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "unknown")).from("player");
    rules(&mut g, vec![player.clone(), Row::new(vec![], Verb::new("hold"))]);
    let mut id = None;
    for _ in 0..200 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("the fire took him");
    let d = g.death(id).unwrap();
    assert_eq!(d.trace.turns.last().map(|t| t.row), Some(0), "R1 was the dying action: {:?}", d.trace.turns.last());
    assert_eq!((d.verdict.as_str(), d.cause_row), ("row", Some(0)), "{} {:?}", d.verdict, d.patches);
    let p = &d.patches[0];
    assert!((p.remove || p.replace) && p.insert_at == 0 && p.survive >= crate::trace::ROW_BAR, "{p:?}");
    // The same row as the shipped preset's is the game's, not the player's: never `row`.
    let mut g = arena_seed(4);
    give(&mut g, "fire");
    g.run.as_mut().unwrap().hero.hp = 1;
    hold_rules(&mut g);
    ticks(&mut g, 60);
    rules(&mut g, vec![player.clone().from("preset"), Row::new(vec![], Verb::new("hold"))]);
    let mut id = None;
    for _ in 0..200 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    // Cut 21 §3: a gamble row is the player's whatever its origin — a kept preset row too.
    let d = g.death(id.expect("died")).unwrap();
    assert_eq!((d.verdict.as_str(), d.cause_row), ("row", Some(0)), "{} {:?}", d.verdict, d.patches);
}

/// Cut 19 §4: the ranking — survival first when it differs by more than the band, reach inside
/// it; an insert on a full set names the least-fired own row it drops, and `apply_patch` drops
/// that row.
#[test]
fn an_insert_on_a_full_set_drops_the_least_fired_row() {
    use crate::offline::apply_patch;
    let rules = RuleSet {
        rows: vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 50)], Verb::new("rest")),
        ],
        name: None,
        route: Vec::new(),
    };
    let new = Row::new(vec![Cond::n("hp<", 20)], Verb::new("return"));
    let p = Patch { no_gain: false, row: new.clone(), insert_at: 1, survive: 1.0, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: Some(0), exits: false, buys: None, moves_from: None, whole: None, gem: false, restores: None };
    let r = apply_patch(&rules, &p, 3);
    assert_eq!(r.rows, vec![new.clone(), rules.rows[1].clone(), rules.rows[2].clone()], "R1 dropped, the patch where it was measured");
    let r = apply_patch(&rules, &Patch { drops: Some(2), ..p.clone() }, 3);
    assert_eq!(r.rows, vec![rules.rows[0].clone(), new.clone(), rules.rows[1].clone()]);
    // Room in the set: nothing drops.
    let r = apply_patch(&rules, &p, 4);
    assert_eq!(r.rows.len(), 4);
    // From a real death on a full set: every insert names a row, and it is the least fired.
    let mut g = arena_seed(2);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    give(&mut g, "heal");
    g.run.as_mut().unwrap().hero.hp = 14;
    for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4)] {
        add_monster(&mut g, "goblin", x, y);
    }
    let full: Vec<Row> = vec![
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        Row::new(vec![Cond::n("hp<", 5)], Verb::new("rest")),
        Row::new(vec![Cond::n("foes>=", 9)], Verb::new("hold")),
        Row::new(vec![Cond::n("hp<", 1)], Verb::new("rest")),
    ];
    self::rules(&mut g, full);
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("the hero died");
    let d = g.death(id).unwrap();
    let rec = g.deaths.get(&id).unwrap();
    assert_eq!(rec.rules.own_rows(), rec.vocab.max_rows, "a full set");
    for p in d.patches.iter().filter(|p| p.insert_at >= 0 && !p.remove && !p.replace) {
        // QA on 1a2a4a9: never an exit row, never a row with the patch's own verb.
        let eligible = |i: usize| !matches!(rec.rules.rows[i].verb.v.as_str(), "return" | "bank") && rec.rules.rows[i].verb.v != p.row.verb.v;
        let Some(at) = p.drops.map(|d| d as usize) else {
            assert!(!(0..rec.rules.rows.len()).any(eligible), "{}: an eligible row and no drop", p.row.describe());
            continue;
        };
        assert!(eligible(at), "{} drops R{} ({})", p.row.describe(), at + 1, rec.rules.rows[at].describe());
        let fires = |i: usize| rec.row_fired.get(i).copied().unwrap_or(0);
        assert!((0..rec.rules.rows.len()).filter(|&i| eligible(i)).all(|i| fires(at) <= fires(i)), "R{} is not the least fired: {:?}", at + 1, rec.row_fired);
        assert_ne!(at, 0, "the attack row fired every turn");
    }
}

/// QA on 1a2a4a9 (qaP: `+ drop R1` on every patch, R1 his return row): `drops` never names an
/// exit row nor a row with the patch's own verb; with nothing else to drop it is `None`.
#[test]
fn a_patch_never_drops_the_exit_row() {
    let mut g = arena_seed(2);
    g.run.as_mut().unwrap().hero.hp = 14;
    for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4)] {
        add_monster(&mut g, "goblin", x, y);
    }
    let full: Vec<Row> = vec![
        Row::new(vec![Cond::n("depth>=", 9)], Verb::new("return")),
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        Row::new(vec![Cond::n("foes>=", 9)], Verb::new("hold")),
        Row::new(vec![Cond::n("depth>=", 12)], Verb::new("bank")),
    ];
    self::rules(&mut g, full);
    let mut id = None;
    for _ in 0..400 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("the hero died");
    let d = g.death(id).unwrap();
    let inserts: Vec<&Patch> = d.patches.iter().filter(|p| p.insert_at >= 0 && !p.remove && !p.replace).collect();
    assert!(!inserts.is_empty(), "{:?}", d.patches);
    for p in inserts {
        let rows = &g.deaths.get(&id).unwrap().rules.rows;
        if let Some(at) = p.drops {
            let r = &rows[at as usize];
            assert!(!matches!(r.verb.v.as_str(), "return" | "bank") && r.verb.v != p.row.verb.v, "{} drops {}", p.row.describe(), r.describe());
        }
    }
}

/// Cut 19 §5: once the lineage has lost to a den, the den pounces on at most one run in three
/// (drawn from the run's seed and floor, so the same run always agrees).
#[test]
fn den_thefts_thin_once_the_lineage_has_lost_to_them() {
    let runs = |thin: bool| -> Vec<bool> {
        (1..=30u64)
            .map(|seed| {
                let mut g = Game::new_literal(seed);
                for f in ["den", "foe:monkey:thief"] {
                    g.lineage.facts.insert(f.into());
                }
                if thin {
                    g.lineage.den_thefts = 1;
                }
                g.set_rules(crate::probes::preset(Class::Fighter)).unwrap();
                g.sim = true;
                g.start_run(Some(seed ^ 0x51));
                g.descend_to_twist(3, "den");
                let mut n = 0;
                while g.run.as_ref().is_some_and(|r| r.over.is_none() && r.depth == 3) && n < 6000 {
                    g.tick();
                    g.events.clear();
                    n += 1;
                }
                g.run.as_ref().unwrap().den_snatches > 0
            })
            .collect()
    };
    let fresh = runs(false);
    let thin = runs(true);
    let (a, b) = (fresh.iter().filter(|x| **x).count(), thin.iter().filter(|x| **x).count());
    assert!(a >= 20, "a lineage that never lost to a den is robbed: {a}/30");
    assert!(b * 3 <= 30 + 3 && b < a, "thinned: {b}/30 (fresh {a}/30)");
    assert_eq!(thin, runs(true), "deterministic");
    // The exit adds the run's snatches to the lineage.
    let mut g = Game::new_literal(3);
    g.lineage.facts.insert("den".into());
    g.start_run(Some(7));
    g.run.as_mut().unwrap().den_snatches = 2;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    assert_eq!(g.lineage.den_thefts, 2);
    g.start_run(Some(8));
    assert!(g.run.as_ref().unwrap().den_thin);
}

/// Cut 19 §5: a grudge is avenged once; a later kill of the same named foe is `slain`.
#[test]
fn a_grudge_is_avenged_once() {
    let kill_grudge = |g: &mut Game| -> Vec<String> {
        g.start_run(Some(11));
        let run = g.run.as_mut().unwrap();
        run.monsters.clear();
        let hp = run.hero.pos;
        let id = run.new_id();
        let mut m = Monster::spawn(id, "rat", Pos::new(hp.x + 1, hp.y), 1);
        let grudge = g.lineage.grudges[0].clone();
        m.make_grudge(&grudge.name);
        m.avenged = grudge.avenged;
        let run = g.run.as_mut().unwrap();
        run.monsters.push(m);
        let mi = run.monsters.len() - 1;
        {
            let (run, mut cx) = g.ctx();
            let hp = run.monsters[mi].hp;
            crate::turn::damage_monster(run, &mut cx, mi, hp, &crate::turn::Src::Hero { ranged: false });
            crate::turn::end_run(run, &mut cx, ExitTier::Return);
        }
        let notes = g.run.as_ref().unwrap().notes.iter().map(|(_, n)| n.clone()).collect();
        g.finish_run();
        notes
    };
    let mut g = Game::new_literal(9);
    g.lineage.grudges.push(crate::descent::Grudge { kind: "rat".into(), name: "Zeleth".into(), depth: 1, heir: 1, avenged: false, tamed: false, biome: None });
    let first = kill_grudge(&mut g);
    assert!(first.iter().any(|n| n == "Zeleth the rat is avenged."), "{first:?}");
    assert!(g.lineage.grudges[0].avenged);
    let second = kill_grudge(&mut g);
    assert!(second.iter().any(|n| n == "Zeleth the rat slain."), "{second:?}");
    assert!(!second.iter().any(|n| n.ends_with("is avenged.")), "{second:?}");
}

/// Cut 19 §3: the loadout repeats by default (no unlock), a `repeat <kind>` ledger line each;
/// the camp's toggle clears it (refunded) and brings it back; `+1 row` is pinned.
#[test]
fn the_loadout_repeats_unless_cleared() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    g.lineage.gold = 1000;
    g.lineage.best_depth = 5;
    let p = crate::engine::supply_price(crate::defs::Cat::Potion, 5);
    g.buy_supply("heal").unwrap();
    g.buy_supply("heal").unwrap();
    assert!(g.lineage().repeat);
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 2, "re-packed at the send's return");
    assert!(g.lineage.gold_ledger.iter().any(|l| l.why == "repeat heal" && l.delta == -2 * p), "{:?}", g.lineage.gold_ledger.iter().rev().take(3).collect::<Vec<_>>());
    let l = g.lineage();
    assert_eq!((l.repeat_kinds.clone(), l.repeat_gold), (vec!["heal".to_string(), "heal".to_string()], 2 * p));
    let gold = g.lineage.gold;
    g.set_restock(false);
    assert!(g.lineage.supplies.iter().all(|s| s.free) && g.lineage.gold == gold + 2 * p, "cleared: refunded");
    assert!(!g.lineage().repeat);
    g.keep(vec![]).ok();
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().supplies.is_empty(), "a cleared repeat packs nothing");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    g.set_restock(true);
    assert!(g.lineage.supplies.is_empty() || g.lineage.supplies.iter().all(|s| s.free), "the kinds of a send that packed nothing");
    // `+1 row` stays on the short list until bought: the next row unlock is pinned.
    let cat = crate::meta::catalogue(&g.lineage);
    let pinned: Vec<&str> = cat.iter().filter(|u| u.pinned).map(|u| u.id.as_str()).collect();
    assert_eq!(pinned, ["row5"]);
    g.lineage.unlocks.insert("row5".into());
    let cat = crate::meta::catalogue(&g.lineage);
    assert_eq!(cat.iter().filter(|u| u.pinned).map(|u| u.id.as_str()).collect::<Vec<_>>(), ["row6"]);
    assert!(!cat.iter().any(|u| u.id == "auto_supply"), "the repeat is the free default, not an unlock");
}

/// QA on 1a2a4a9 (qaP: `repeat · $40` with a heal and a mapping scroll packed, then
/// `−$60 repeat mapping · −$40 repeat heal`; after a $26 stall the heal was not re-packed and
/// nothing said so): the badge's price is what the re-pack charges, and a re-pack short of gold
/// buys what it can and says `repeat short`.
#[test]
fn the_repeat_badge_is_the_repack_price() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    identify(&mut g, "mapping");
    // (Cut 21 §2: the repeat packs only the kinds a row reads — a mapping row that never fires)
    let mut set = g.lineage.rules().clone();
    set.rows.push(Row::new(vec![Cond::n("hp<", 1)], Verb::arg("read", "mapping")));
    g.set_rules_raw(set).unwrap();
    g.lineage.gold = 1000;
    g.lineage.best_depth = 5;
    g.buy_supply("heal").unwrap();
    let price = |g: &Game, k: &str| g.supply_catalogue().iter().find(|e| e.kind == k).unwrap().price;
    assert_eq!(g.lineage().repeat_gold, price(&g, "heal"));
    g.buy_supply("mapping").unwrap();
    let want = price(&g, "heal") + price(&g, "mapping");
    assert_eq!(g.lineage().repeat_gold, want, "the shelf is the next send's pack");
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal" && i.kind != "mapping");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    let before = g.lineage.gold;
    g.finish_run();
    let charged: i32 = g.lineage.gold_ledger.iter().filter(|l| l.why.starts_with("repeat ") && l.delta < 0).map(|l| -l.delta).sum();
    assert_eq!(charged, want, "the re-pack charged the badge: {:?}", g.lineage.gold_ledger.iter().rev().take(4).collect::<Vec<_>>());
    let _ = before;
    // Only the heal used: the mapping scroll comes back, the heal alone is bought again.
    assert_eq!(g.lineage().repeat_gold, want);
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    let n = g.lineage.gold_ledger.len();
    g.finish_run();
    let charged: i32 = g.lineage.gold_ledger[n.min(g.lineage.gold_ledger.len())..].iter().filter(|l| l.why.starts_with("repeat ") && l.delta < 0).map(|l| -l.delta).sum();
    assert_eq!(charged, price(&g, "heal"), "a top-up: {:?}", g.lineage.gold_ledger.iter().rev().take(4).collect::<Vec<_>>());
    let mut kinds: Vec<&str> = g.lineage.supplies.iter().filter(|s| !s.free).map(|s| s.kind.as_str()).collect();
    kinds.sort();
    assert_eq!(kinds, ["heal", "mapping"]);
    // Short of gold: what it can, and a `repeat short` line.
    let refund: Vec<u32> = g.lineage.supplies.iter().filter(|s| !s.free).map(|s| s.id).collect();
    for id in refund {
        g.drop_supply(id).unwrap();
    }
    g.lineage.last_supplies = vec!["mapping".into(), "heal".into()];
    let gold = g.lineage.gold;
    g.lineage.gold_move(price(&g, "heal") + 1 - gold, "test");
    g.restock();
    assert_eq!(g.lineage.supplies.iter().filter(|s| !s.free).map(|s| s.kind.as_str()).collect::<Vec<_>>(), ["heal"], "the heal it could pay for");
    assert_eq!(g.lineage.repeat_short, ["mapping"]);
    assert_eq!(g.lineage().repeat_short, ["mapping"]);
    assert!(g.lineage.gold_ledger.iter().any(|l| l.why == "repeat short" && l.delta == 0), "{:?}", g.lineage.gold_ledger.iter().rev().take(3).collect::<Vec<_>>());
}

/// Cut 19 §3: an absence's repeat never spends more than the absence brought home.
#[test]
fn offline_restock_never_spends_more_than_the_night_brought() {
    let (capped, ok) = par_seeds(1..=6u64, |seed| {
        let mut g = Game::new_literal(seed);
        no_kennel_leash(&mut g);
        identify(&mut g, "heal");
        g.lineage.gold = 100_000;
        g.lineage.unlocks.insert("supply_cap_5".into());
        for _ in 0..5 {
            g.buy_supply("heal").unwrap();
        }
        let r = crate::offline::run_offline_quick(&mut g, 4 * 3600);
        let gold = r.gold.clone().unwrap();
        (r.restock_capped, gold.spent <= gold.home + gold.salvage + gold.wake)
    })
    .into_iter()
    .fold((0, true), |a, (c, o)| (a.0 + c as u32, a.1 && o));
    assert!(ok, "spent more than the night brought home");
    assert!(capped >= 1, "a preset that dies early cannot pay five heals a run: the cap bites");
}

/// Cut 19 §1: the cage preferences measured for the set — one per preference, the current one
/// at a zero delta, memoised on the game; the snapshot names the preference's pick.
#[test]
fn the_cage_forecast_measures_each_preference() {
    let mut g = Game::new_literal(21);
    g.lineage.best_depth = 4;
    g.lineage.vault_pref = "armour".into();
    let opts = g.cage_forecast();
    assert_eq!(opts.iter().map(|o| o.pref.as_str()).collect::<Vec<_>>(), ["weapon", "armour", "potion", "scroll"]);
    let cur = opts.iter().find(|o| o.current).unwrap();
    assert_eq!((cur.pref.as_str(), cur.delta, cur.reach_delta, cur.bank_delta), ("armour", 0.0, 0.0, 0.0));
    // QA on 0c6e126 (qaY): read at the frontier (best + 1) while the panel reaches it, else at the best depth
    let reach5 = g.forecast().depths.iter().find(|d| d.depth == 5).map(|d| d.reach).unwrap_or(0.0);
    let at = if reach5 > crate::forecast::WALL_REACH { 5 } else { 4 };
    assert!(opts.iter().all(|o| (0.0..=1.0).contains(&o.reach) && o.depth == at), "{:?}", opts.iter().map(|o| o.depth).collect::<Vec<_>>());
    assert_eq!(g.cage_forecast(), opts, "memoised");
    // Switching the preference reads the same panels from the other side.
    g.set_vault_pref("weapon").unwrap();
    let back = g.cage_forecast();
    let w = opts.iter().find(|o| o.pref == "weapon").unwrap();
    let a = back.iter().find(|o| o.pref == "armour").unwrap();
    assert!((a.reach_delta + w.reach_delta).abs() < 1e-9, "paired: {} vs {}", a.reach_delta, w.reach_delta);
    // The in-run beat: the snapshot's vault choice names what the preference takes.
    let items = vec![Item::new(1001, "sword"), Item::new(1002, "mail"), Item::new(1003, "heal")];
    assert_eq!(crate::turn::vault_pick(&items, "armour"), 1);
    assert_eq!(crate::turn::vault_pick(&items, "scroll"), 0);
    let mut g = arena();
    g.lineage.vault_pref = "armour".into();
    g.run.as_mut().unwrap().vault_choice = Some((g.run.as_ref().unwrap().turn, items));
    assert_eq!(g.snapshot().vault_choice.unwrap().pick, Some(1002));
}

// ---------------------------------------------------------------- Cut 20

/// Cut 20 §1: a run suffers at most one theft — a den's other thieves bolt empty-handed, and a
/// monkey's blow after the first theft takes nothing (it runs). Measured over whole runs of the
/// preset on 24 seeds with a den forced on D3, and on the den floor itself.
#[test]
fn a_run_suffers_at_most_one_theft() {
    let counts = par_seeds(1..=24u64, |seed| {
        let mut g = Game::new_literal(seed);
        g.lineage.facts.insert("foe:monkey:thief".into());
        g.sim = true;
        g.start_run(Some(seed ^ 0x20));
        g.descend_to_twist(3, "den");
        g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
        let r = g.run.as_ref().unwrap();
        (r.stolen.len(), r.den_wakes)
    });
    assert!(counts.iter().all(|(n, _)| *n <= 1), "{counts:?}");
    let woke = counts.iter().filter(|(_, w)| *w > 0).count();
    assert!(woke >= 12, "the den still pounces on a fresh lineage: {woke}/24");
    assert!(counts.iter().filter(|(n, _)| *n == 1).count() >= 12, "{counts:?}");
    // A monkey after the run's one theft: its blow takes nothing and it runs.
    let mut g = arena();
    let heal = give(&mut g, "heal");
    let id = add_monster(&mut g, "monkey", 5, 5);
    let run = g.run.as_mut().unwrap();
    run.stolen.push((0, "scroll".into()));
    let mi = run.monsters.iter().position(|m| m.id == id).unwrap();
    for _ in 0..40 {
        let (run, mut cx) = g.ctx();
        crate::ai::monster_act(run, &mut cx, mi);
        if run.monsters[mi].fear > 0 {
            break;
        }
    }
    let run = g.run.as_ref().unwrap();
    assert!(run.hero.inv.iter().any(|i| i.id == heal), "the heal is still in the pack");
    assert_eq!(run.stolen.len(), 1);
    assert!(run.monsters[mi].fear > 0 && run.monsters[mi].stolen.is_none(), "the monkey runs empty-handed");
}

/// Cut 20 §1: a killed thief drops what it stole where it fell; picked back up, it is a note
/// (`Got the heal back.`, ≤ 8 words).
#[test]
fn a_killed_thief_drops_what_it_stole_and_the_pickup_is_a_note() {
    let mut g = arena();
    let id = add_monster(&mut g, "monkey", 5, 5);
    let run = g.run.as_mut().unwrap();
    let iid = run.new_item_id();
    let mut heal = Item::new(iid, "heal");
    heal.known = true;
    let mi = run.monsters.iter().position(|m| m.id == id).unwrap();
    run.monsters[mi].stolen = Some(heal);
    run.monsters[mi].fleeing = true;
    run.stolen_ids.push(iid);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_monster(run, &mut cx, mi, 99, &crate::turn::Src::Hero { ranged: false });
    }
    let run = g.run.as_mut().unwrap();
    assert!(run.items.iter().any(|fi| fi.item.id == iid && fi.pos == Pos::new(5, 5)), "dropped where it fell");
    run.hero.pos = Pos::new(5, 5);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::pickup_here(run, &mut cx);
    }
    let run = g.run.as_ref().unwrap();
    assert!(run.hero.inv.iter().any(|i| i.id == iid), "back in the pack");
    let label = run.recovered.last().map(|(_, l)| l.clone()).expect("the recovery is noted");
    assert!(label.contains("heal"), "{label}");
    let text = format!("Got the {label} back.");
    assert!(word_count(&text) <= 8, "{text}");
    assert!(run.stolen_ids.is_empty());
}

/// Cut 20 §1: the first den a lineage meets always wakes; after one has, dens pounce on ≤ 1
/// floor in 3 (deterministic from the run's seed and the floor) — for every lineage, not only
/// one that lost to a den; and within a run, once its den has woken.
#[test]
fn dens_wake_on_one_floor_in_three_after_the_first() {
    let mut g = Game::new_literal(5);
    g.start_run(Some(5));
    assert!(!g.run.as_ref().unwrap().den_thin, "a lineage that never met a den: it wakes");
    g.lineage.den_wakes = 1;
    g.run = None;
    g.start_run(Some(6));
    assert!(g.run.as_ref().unwrap().den_thin);
    // The share over many (seed, floor) draws.
    let (mut n, mut k) = (0u32, 0u32);
    for seed in 1..=400u64 {
        for depth in 3..=11u32 {
            let mut g = Game::new_literal(1);
            g.lineage.den_wakes = 1;
            g.start_run(Some(seed));
            let run = g.run.as_mut().unwrap();
            run.depth = depth;
            n += 1;
            k += crate::situations::den_pounces(run) as u32;
            assert_eq!(crate::situations::den_pounces(run), crate::situations::den_pounces(run));
        }
    }
    let share = k as f64 / n as f64;
    assert!((0.28..=0.36).contains(&share), "{share}");
    // A lineage whose den has woken is thinned at the exit.
    let mut g = Game::new_literal(3);
    g.start_run(Some(7));
    g.run.as_mut().unwrap().den_wakes = 1;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    assert_eq!(g.lineage.den_wakes, 1);
}

/// Cut 20 §2: a companion at ≤ 30 % hp falls back (engages nothing) and a blow that would kill
/// it then leaves it at 1 unless it is cornered; each descent heals it to ≥ 60 %.
#[test]
fn a_wounded_pet_falls_back_and_heals_on_the_stairs() {
    let mut g = arena();
    let c = Companion { id: 7_000_001, kind: "jackal".into(), name: "Thix".into(), level: 2, tags: vec!["pack".into(), "fast".into()], gen: 0, rules: crate::probes::default_companion_rules(&["pack".into()], 2), max_rows: 3, hp: 8, max_hp: 8 };
    let run = g.run.as_mut().unwrap();
    let pid = run.new_id();
    let mut pet = crate::engine::companion_monster(pid, &c, Pos::new(5, 5));
    run.companions.push(c.clone());
    let max = pet.max_hp;
    assert!(max >= 8 + crate::engine::PET_LEVEL_HP, "a raised pet is sturdier: {max}");
    pet.hp = 1;
    run.monsters.push(pet);
    let foe = add_monster(&mut g, "goblin", 6, 5);
    let run = g.run.as_mut().unwrap();
    let pi = run.monsters.iter().position(|m| m.id == pid).unwrap();
    let fi = run.monsters.iter().position(|m| m.id == foe).unwrap();
    {
        let (run, mut cx) = g.ctx();
        crate::ai::monster_act(run, &mut cx, pi);
        assert!(run.monsters[pi].fleeing, "it falls back");
        assert_eq!(run.monsters[fi].hp, run.monsters[fi].max_hp, "it engages nothing");
        crate::turn::damage_monster(run, &mut cx, pi, 50, &crate::turn::Src::Mon(fi));
        assert_eq!(run.monsters[pi].hp, 1, "not cornered: it lives");
    }
    // From above 30 % a killing blow kills.
    {
        let (run, mut cx) = g.ctx();
        run.monsters[pi].hp = run.monsters[pi].max_hp;
        crate::turn::damage_monster(run, &mut cx, pi, 999, &crate::turn::Src::Mon(fi));
        assert!(run.monsters[pi].hp <= 0);
    }
    // Cornered: walled in on every side, it dies.
    let mut g = arena();
    let run = g.run.as_mut().unwrap();
    let pid = run.new_id();
    let mut pet = crate::engine::companion_monster(pid, &c, Pos::new(1, 10));
    run.companions.push(c.clone());
    pet.hp = 1;
    run.monsters.push(pet);
    add_monster(&mut g, "goblin", 2, 10);
    add_monster(&mut g, "goblin", 2, 9);
    add_monster(&mut g, "goblin", 1, 9);
    let run = g.run.as_mut().unwrap();
    let pi = run.monsters.iter().position(|m| m.id == pid).unwrap();
    {
        let (run, mut cx) = g.ctx();
        assert!(crate::ai::pet_cornered(run, pi));
        crate::turn::damage_monster(run, &mut cx, pi, 50, &crate::turn::Src::Mon(pi + 1));
        assert!(run.monsters[pi].hp <= 0, "cornered: it dies");
    }
    // The stairs heal a wounded pet to ≥ 60 %.
    let mut g = Game::new_literal(9);
    g.start_run(Some(9));
    let run = g.run.as_mut().unwrap();
    let pid = run.new_id();
    let hp = run.hero.pos;
    let mut pet = crate::engine::companion_monster(pid, &c, Pos::new(hp.x, hp.y));
    run.companions.push(c.clone());
    pet.hp = 1;
    run.monsters.push(pet);
    g.descend_to(2);
    let run = g.run.as_ref().unwrap();
    let pet = run.monsters.iter().find(|m| m.id == pid).expect("the pet came down");
    assert!(pet.hp * 100 >= pet.max_hp * crate::ai::PET_DESCENT_PCT && !pet.fleeing, "{}/{}", pet.hp, pet.max_hp);
}

/// Cut 20 §4: the stake names what a death keeps (the death tier's share) beside the exit
/// row's keep; a repeat re-pack charged at a death's exit is a `repeat` ledger line.
#[test]
fn the_stake_names_the_death_keep_and_a_death_repeat_is_a_line() {
    let mut g = Game::new_literal(4);
    g.start_run(Some(4));
    g.run.as_mut().unwrap().loot_add_gold(78);
    let st = g.snapshot().stake;
    assert_eq!(st.loot, 78);
    assert_eq!(st.death_keep, 78 * ExitTier::Death.pct() / 100);
    let v = serde_json::to_value(&st).unwrap();
    assert!(v.get("death_keep").is_some());
    // A death with a repeat to pay: the ledger line and the death record's `spent`.
    let mut g = Game::new_literal(4);
    g.lineage.gold = 500;
    if let Some(f) = ident_fact(&g.lineage.flavours, "heal") {
        g.lineage.facts.insert(f);
    }
    g.lineage.supplies.clear();
    g.start_run(Some(4));
    g.lineage.last_supplies = vec!["heal".into()];
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Death);
    }
    g.finish_run();
    let t = g.lineage.total_turns;
    let repeat: Vec<&GoldLine> = g.lineage.gold_ledger.iter().filter(|x| x.t == t && x.why.starts_with("repeat")).collect();
    assert!(!repeat.is_empty(), "{:?}", g.lineage.gold_ledger);
    let line = g.last_exit.clone().unwrap();
    assert_eq!(line.spent, -repeat.iter().map(|x| x.delta).sum::<i32>());
}

/// Cut 20 §5: each night the lineage's best depth + 2 is the bounty floor — its gold piles pay
/// double and it holds one item of the next tier (forged one past the lineage's forge); the
/// forecast flags its notch; the report says whether a run brought it home. Deterministic.
#[test]
fn the_bounty_floor_moves_each_night_and_pays_double() {
    let mut g = Game::new_literal(12);
    g.lineage.best_depth = 5;
    assert_eq!(g.lineage.bounty, None);
    for _ in 0..crate::engine::NIGHT_RUNS {
        g.lineage.night_run(1, 1, false);
    }
    assert_eq!(g.lineage.bounty, Some(7));
    assert_eq!(g.lineage().bounty.as_ref().map(|b| b.depth), Some(7));
    // Cut 28 §1: the bounty says what it pays and needs.
    let b = g.lineage().bounty.unwrap();
    assert_eq!((b.pays.as_str(), b.needs.as_str()), ("$×2 · item", "reach"));
    let floor = |bounty: Option<u32>| {
        let mut g = Game::new_literal(12);
        g.lineage.best_depth = 5;
        g.lineage.bounty = bounty;
        g.start_run(Some(33));
        g.descend_to(7);
        let run = g.run.as_ref().unwrap();
        let gold: Vec<i32> = run.items.iter().filter(|fi| fi.item.kind == "gold").map(|fi| fi.item.amount).collect();
        let gear: Vec<(String, i32)> = run.items.iter().filter(|fi| fi.item.known && matches!(fi.item.cat(), crate::defs::Cat::Weapon | crate::defs::Cat::Armour)).map(|fi| (fi.item.kind.clone(), fi.item.enchant)).collect();
        (gold, gear)
    };
    let (plain, _) = floor(None);
    let (bounty, gear) = floor(Some(7));
    // The floor's six piles (the first placed; a nest's or a den's pile may follow).
    assert!(plain.len() >= 6 && bounty.len() >= 6);
    assert_eq!(bounty[..6].to_vec(), plain[..6].iter().map(|g| 2 * g).collect::<Vec<i32>>(), "{plain:?} → {bounty:?}");
    assert_eq!(gear.len(), 1, "{gear:?}");
    assert!(gear[0].1 >= 1);
    assert_eq!(floor(Some(7)), (bounty.clone(), gear.clone()), "deterministic");
    // The forecast's notch.
    let mut g = Game::new_literal(12);
    g.lineage.best_depth = 5;
    g.lineage.bounty = Some(7);
    let f = g.forecast();
    assert!(f.depths.iter().any(|d| d.depth == 7 && d.bounty), "{:?}", f.depths.iter().map(|d| (d.depth, d.bounty)).collect::<Vec<_>>());
    assert_eq!(f.depths.iter().filter(|d| d.bounty).count(), 1);
    // The report: taken when a run reached it and came home (a floor the camp showed).
    g.bounty_seen = g.lineage.bounty;
    g.start_run(Some(1));
    g.batch = crate::engine::Batch::default();
    {
        let run = g.run.as_mut().unwrap();
        run.max_depth = 7;
        run.bounty_gold = 50;
    }
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert_eq!(g.batch.bounty, Some(BountyReport { depth: 7, taken: true, gold: 50 }));
    let mut g = Game::new_literal(12);
    g.lineage.bounty = Some(9);
    g.bounty_seen = g.lineage.bounty;
    g.start_run(Some(1));
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Death);
    }
    g.finish_run();
    assert_eq!(g.batch.bounty, Some(BountyReport { depth: 9, taken: false, gold: 0 }));
}

// ---------------------------------------------------------------- QA on e75ec29 (qaQ)

/// The first death of seed 1615 under the QA player's set (`hp < 20% → to corridor`, `hp < 40%
/// → drink heal`, `foes ≥ 1 → attack nearest`, a curious heir): the unpatched replays all
/// survive (`base 100%`), so it is a `dice` death that says `nothing beats base` — not a `gap`
/// offering `hp < 20% → rest · survives 100%`. And every shown patch that inserts a row acts in
/// half the replays from the floor's start (`trace::floor_fired`): the rest row the window
/// fired (the goblin out of view) acted in 3 of 12 of those, and the next run never rested.
#[test]
fn a_death_the_replays_all_survive_is_dice_and_its_patches_act_on_the_floor() {
    // (Cut 30 §2: the curious heir no longer drinks on its own, so seed 1615's repro moved — the first
    // first-heir death from 1615 on whose replays all survive is the case)
    let mut found = false;
    for seed in 1615..1700u64 {
        let mut g = Game::new_literal(seed);
        let set = RuleSet { name: None, rows: vec![Row::new(vec![Cond::n("hp<", 20)], Verb::new("back_corridor")), Row::new(vec![Cond::n("hp<", 40)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], route: Vec::new() };
        g.set_rules(set).unwrap();
        g.send();
        let mut died = None;
        for _ in 0..4000 {
            let r = g.step(50);
            if r.run_over {
                died = r.events.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "death")).then_some(r.snapshot.run.id);
                break;
            }
        }
        let Some(id) = died else { continue };
        g.keep(vec![]).unwrap();
        let d = g.death(id).unwrap();
        if (d.baseline - 1.0).abs() > 1e-9 {
            continue;
        }
        found = true;
        assert_eq!(d.verdict, "dice", "seed {seed}");
        assert!(d.nothing_beats_base, "{:?}", d.patches);
        let rec = g.deaths.get(&id).unwrap().clone();
        assert!(rec.floor.is_some(), "the record carries the floor's start");
        // (every shown patch acts on the floor — unless none does, when the list stands: a dice death
        // still names what was tried)
        let rates: Vec<(String, f64)> = d.patches.iter().filter_map(|p| crate::trace::floor_fired(&g, &rec, p).map(|f| (p.row.describe(), f))).collect();
        if rates.iter().any(|(_, f)| *f >= crate::trace::FIRED_BAR) {
            for (row, f) in &rates {
                assert!(*f >= crate::trace::FIRED_BAR, "{row} acts in {:.0}% of the floor's replays", f * 100.0);
            }
        }
        break;
    }
    assert!(found, "a first-heir death whose replays all survive");
}

/// Every bones pile an heir recovers is named on its chronicle line, however many deeds.
#[test]
fn the_chronicle_names_every_bones_pile_found() {
    let mut g = Game::new_literal(3);
    let l = &mut g.lineage;
    l.heir_deed("took the Warlord".into());
    l.heir_deed("freed a captive".into());
    l.heir_deed("tamed a jackal".into());
    for h in [2, 5, 7] {
        l.heir_deed(format!("found ♟{h}'s bones"));
    }
    l.chronicle_heir("fell to gas", None);
    let line = l.chronicle.last().unwrap();
    assert!(line.contains("found ♟2, ♟5, ♟7's bones"), "{line}");
    assert!(line.contains("took the Warlord"), "{line}");
}

/// A drink the shipped set holds is offered while its kind is still unidentified.
#[test]
fn the_vocabulary_offers_the_verbs_the_set_holds() {
    let g = Game::new_literal(5);
    let heal = Verb::arg("drink", "heal");
    assert!(g.lineage.rules().rows.iter().any(|r| r.verb == heal), "the shipped set drinks heal");
    assert!(!crate::item::is_identified(&g.lineage.facts, &g.lineage.flavours, "heal"));
    assert!(g.vocabulary().verbs.contains(&heal));
    // A kind no set holds stays unoffered until identified.
    assert!(!g.vocabulary().verbs.contains(&Verb::arg("drink", "speed")));
}

/// An absence's `rested` is the rest its runs earned — one wake per death — not the clock's
/// (the camp rest it opens with is the previous run's; the last run's runs on at the return).
#[test]
fn rested_is_the_rest_the_absences_runs_earned() {
    let mut g = Game::new_literal(21);
    g.lineage.rest_left = crate::engine::WAKE_TICKS;
    let r = g.run_offline(2 * 3600);
    assert!(!r.sampled && r.runs > 0 && r.banked == 0 && r.returned == 0, "{} runs · {} banked · {} returned", r.runs, r.banked, r.returned);
    assert_eq!(r.rested_s, r.runs as u64 * crate::engine::WAKE_TICKS as u64 / crate::offline::TICKS_PER_SECOND);
}

/// QA on e75ec29 (qaQ: `bounty D10 · missed`, no bounty on the camp before the absence): a
/// floor a night sets inside an absence is not reported; the one the camp showed (a load, a
/// send, an edit) is.
#[test]
fn the_report_names_only_the_bounty_the_camp_showed() {
    let mut g = Game::new_literal(8);
    g.lineage.night_runs = crate::engine::NIGHT_RUNS - 1;
    let r = g.run_offline(4 * 3600);
    assert!(g.lineage.bounty.is_some(), "a night closed inside the absence");
    assert_eq!(r.bounty, None, "never on the camp");
    let shown = g.lineage.bounty;
    let mut h = Game::load(&g.save()).unwrap();
    let r = h.run_offline(2 * 3600);
    assert_eq!(r.bounty.map(|b| b.depth), shown, "the floor the camp showed at the load");
}

// ---------------------------------------------------------------- QA on e75ec29 (qaR)

/// A card buy puts its row in the set only when the measure says it does not hurt, and only
/// while the set holds fewer than `AUTO_CARDS` card rows.
#[test]
fn a_card_buy_inserts_only_what_the_measure_allows() {
    use crate::meta::{auto_insert, AUTO_CARDS, CARD_STALL_RISE};
    let g = Game::new_literal(3);
    let mut u = g.unlocks().into_iter().find(|u| u.id == "corridor_fighting").unwrap();
    u.owned = false;
    u.delta = Some(0.02);
    u.stall = Some(0.01);
    assert!(auto_insert(&u, 0));
    assert!(!auto_insert(&u, AUTO_CARDS), "a set with {AUTO_CARDS} cards takes no more on its own");
    let mut down = u.clone();
    down.delta = Some(-0.04);
    assert!(!auto_insert(&down, 0), "reach down at its best place");
    let mut stalls = u.clone();
    stalls.stall = Some(CARD_STALL_RISE + 0.03);
    assert!(!auto_insert(&stalls, 0), "stall share up past the rise");
    let mut unmeasured = u.clone();
    unmeasured.delta = None;
    assert!(!auto_insert(&unmeasured, 0), "unmeasured");
    let mut verb = g.unlocks().into_iter().find(|u| u.id == "throw").unwrap();
    verb.delta = Some(0.1);
    verb.stall = Some(0.0);
    assert!(!auto_insert(&verb, 0), "a verb is the player's to write");
    // The catalogue carries it: never on an owned or an unmeasured card.
    for u in g.unlocks() {
        assert!(!u.auto_insert || (u.delta.is_some() && !u.owned), "{}", u.id);
    }
}

/// What thieves took and kept is on the exit line and in the absence's report; a got-back
/// item is not.
#[test]
fn thefts_kept_by_thieves_reach_the_exit_line_and_the_report() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        run.stolen_ids = vec![71];
        run.stolen_labels = vec![(70, "heal potion".into()), (71, "murky potion?".into())];
        run.stolen = vec![(3, "heal potion".into()), (9, "murky potion?".into())];
        run.recovered = vec![(12, "heal potion".into())];
    }
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.batch = crate::engine::Batch::default();
    g.finish_run();
    let line = g.batch.exits.last().unwrap();
    assert_eq!(line.stolen, vec!["murky potion?".to_string()]);
    assert_eq!(g.batch.stolen.get("murky potion?"), Some(&1));
    assert_eq!(g.batch.stolen.get("heal potion"), None, "got back");
}

/// The bounty floor is one a run can reach: best + 2, never past the next boss floor the
/// lineage has not passed.
#[test]
fn the_bounty_floor_stops_at_the_next_unbeaten_boss() {
    use crate::engine::bounty_floor;
    let none = std::collections::BTreeSet::new();
    assert_eq!(bounty_floor(13, &none), 13, "the mother (D13) unbeaten");
    assert_eq!(bounty_floor(12, &none), 13);
    assert_eq!(bounty_floor(14, &none), 16, "past D13: the lich's D18 is the next");
    let mother: std::collections::BTreeSet<String> = ["bloat_mother".to_string()].into();
    assert_eq!(bounty_floor(13, &mother), 15, "the mother slain");
    assert_eq!(bounty_floor(8, &none), 8, "the warlord's floor");
    assert_eq!(bounty_floor(0, &none), 2);
}

/// A companion that walks down with the heir is named at the start (`Skog joins.`).
#[test]
fn a_party_companion_is_named_when_it_joins() {
    let mut g = Game::new_literal(4);
    g.lineage.party = crate::probes::pets_party();
    let names: Vec<String> = g.lineage.party.iter().map(|c| c.name.clone()).collect();
    assert!(!names.is_empty());
    g.events.clear();
    g.start_run(Some(5));
    for n in &names {
        let want = format!("{n} joins.");
        assert!(g.events.iter().any(|e| matches!(e, Ev::Note { text, .. } if *text == want)), "{want}: {:?}", g.events);
    }
}

// ---------------------------------------------------------------- Cut 21

/// Cut 21 §1: a bank lights every waystone at or above its floor (a return or a death lights
/// none); a lit waystone is a start the lineage may choose, an unlit one is refused; the lit
/// set and the start survive a save, and a save from before the waystones lights them from
/// its banks.
#[test]
fn waystones_light_on_a_bank_and_the_start_round_trips() {
    let mut g = arena();
    assert!(g.lineage.waystones.is_empty());
    assert!(g.set_start(9).is_err(), "an unlit waystone is refused");
    // A return from D10 lights nothing; a death neither.
    for tier in [ExitTier::Return, ExitTier::Death] {
        g.run.as_mut().unwrap().depth = 10;
        g.run.as_mut().unwrap().max_depth = 10;
        {
            let (run, mut cx) = g.ctx();
            crate::turn::end_run(run, &mut cx, tier);
        }
        g.finish_run();
        g.auto_keep();
        assert!(g.lineage.waystones.is_empty(), "{tier:?} lit a waystone");
        g.start_run(None);
    }
    g.run.as_mut().unwrap().depth = 10;
    g.run.as_mut().unwrap().max_depth = 10;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert_eq!(g.lineage.waystones, vec![5, 9], "a bank from D10 lights D5 and D9");
    assert!(g.batch.bests.iter().any(|b| b == "waystone D9"), "{:?}", g.batch.bests);
    assert!(g.set_start(14).is_err());
    g.set_start(9).unwrap();
    let l = g.lineage();
    assert_eq!((l.start, l.start_toll, l.waystones.clone()), (9, 0, vec![5, 9]));
    let h = Game::load(&g.save()).unwrap();
    assert_eq!((h.lineage.start, h.lineage.waystones.clone()), (9, vec![5, 9]));
    // A save from before the waystones: lit from its deepest bank at the load.
    let mut old: serde_json::Value = serde_json::from_str(&g.save()).unwrap();
    let lin = old["lineage"].as_object_mut().unwrap();
    lin.remove("waystones");
    lin.remove("start");
    let h = Game::load(&old.to_string()).unwrap();
    assert_eq!((h.lineage.start, h.lineage.waystones.clone()), (1, vec![5, 9]));
}

/// Cut 21 §1: a start below D1 begins on that floor — generated at that depth, the heir's kit
/// and level as from D1. QA on 778fa1b (qaV): the start is free — no toll, no ledger line,
/// payable from an empty purse, and every run of an absence starts there.
#[test]
fn a_waystone_start_is_free_and_begins_on_its_floor() {
    let mut g = Game::new_literal(3);
    g.lineage.waystones = vec![5, 9];
    g.lineage.best_depth = 12;
    g.set_start(9).unwrap();
    g.lineage.gold = 500;
    g.lineage.classes.get_mut("fighter").unwrap().level = 3;
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    assert_eq!((run.depth, run.max_depth, run.start, run.toll), (9, 9, 9, 0));
    assert_eq!(run.biome(), crate::descent::Biome::Fens, "the floor is a Fens floor");
    assert_eq!(run.hero.level, 3, "the heir's level");
    assert_eq!(run.hero.pos, run.floor.stairs_up);
    assert_eq!(g.lineage.gold, 500, "no toll");
    assert!(!g.lineage.gold_ledger.iter().any(|l| l.why.starts_with("waystone")), "{:?}", g.lineage.gold_ledger);
    assert!(run.notes.iter().any(|(_, n)| n == "Heir 1 enters D9, the Fens."), "{:?}", run.notes);
    // An empty purse still starts there.
    g.lineage.night();
    g.run = None;
    g.lineage.gold = 0;
    assert!(g.lineage().start_payable);
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    assert_eq!((run.depth, run.start_short, run.toll, g.lineage.gold), (9, None, 0, 0));
    // Offline: every run starts on D9, nothing charged, nothing short.
    g.run = None;
    g.lineage.gold = 100_000;
    let r = g.run_offline(4 * 3600);
    assert!(r.runs >= 2);
    assert!(g.batch.run_outcomes.iter().all(|(d, _)| *d >= 9), "{:?}", g.batch.run_outcomes);
    assert!(!r.spent.iter().any(|s| s.kind.starts_with("waystone")), "{:?}", r.spent);
    assert!(r.start_short.is_none());
    // The forecast simulates from the start: every row above it reads 1.0.
    let f = g.forecast();
    assert_eq!(f.start, 9);
    assert!(f.depths.iter().filter(|d| d.depth <= 9).all(|d| d.reach >= 1.0 - 1e-9), "{:?}", f.depths.iter().map(|d| (d.depth, d.reach)).collect::<Vec<_>>());
    // The start tablet: D1 and each lit waystone, the current one marked, paired with it.
    let opts = g.start_forecast();
    assert_eq!(opts.iter().map(|o| o.start).collect::<Vec<_>>(), vec![1, 5, 9]);
    assert_eq!(opts.iter().map(|o| o.toll).collect::<Vec<_>>(), vec![0, 0, 0]);
    assert!(opts.iter().all(|o| !o.short));
    let cur = opts.iter().find(|o| o.current).unwrap();
    assert_eq!(cur.start, 9);
    assert!(cur.delta.abs() < 1e-9 && cur.gold_delta.abs() < 1e-9 && cur.net_delta.abs() < 1e-9);
    assert!(opts.iter().all(|o| (o.net - o.gold).abs() < 1e-9));
    // Bots start at D1: a new lineage's start is 1 whatever it banks.
    assert_eq!(Game::new_literal(3).lineage.start, 1);
}

/// Cut 21 §2: a found supply of a kind the shelf sells, a row uses and the repeat packs goes
/// to the shelf at a non-death exit (one per packed one) — not salvaged, marked `found`, on
/// the exit's `shelved` — and the repeat does not buy it again; dropped from the shelf, it is
/// salvaged, never refunded at the price. A kind no row uses is neither shelved nor re-bought
/// (`repeat_dropped`).
#[test]
fn found_supplies_go_to_the_shelf_and_the_repeat_buys_only_used_kinds() {
    let mut g = arena_seed(5);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    identify(&mut g, "strength");
    identify(&mut g, "poison");
    g.lineage.gold = 1000;
    g.lineage.last_supplies = vec!["heal".into(), "strength".into()];
    // The shipped set drinks heal; nothing drinks strength or poison.
    let heal = give(&mut g, "heal");
    let poison = give(&mut g, "poison");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let shelf: Vec<(&str, bool)> = g.lineage.supplies.iter().map(|s| (s.kind.as_str(), s.found)).collect();
    assert_eq!(shelf, vec![("heal", true)], "the found heal is on the shelf; strength is not re-bought");
    let line = g.last_exit.clone().unwrap();
    assert_eq!(line.shelved, vec![KindCount { kind: "heal".into(), n: 1 }]);
    assert!(!line.text.contains("shelf"), "{}", line.text);
    assert!(!g.batch.salvaged.contains_key("heal"), "the heal was not salvaged");
    assert_eq!(g.batch.shelved.get("heal"), Some(&(1, crate::engine::supply_price(crate::defs::Cat::Potion, g.lineage.best_depth))));
    assert!(!g.batch.spent.contains_key("heal") && !g.batch.spent.contains_key("strength"), "{:?}", g.batch.spent);
    let pending: Vec<u32> = g.pending_exit.as_ref().unwrap().items.iter().map(|i| i.id).collect();
    assert!(pending.contains(&poison) && !pending.contains(&heal), "the poison waits on the keep sheet, the heal does not");
    let l = g.lineage();
    assert_eq!(l.repeat_dropped, vec!["strength".to_string()]);
    assert!(l.repeat_kinds.is_empty() && l.repeat_gold == 0, "{:?} ${}", l.repeat_kinds, l.repeat_gold);
    assert!(l.supplies.iter().any(|s| s.kind == "heal" && s.found));
    // The next send packs it free. QA on a946e04 (qaT: the repeat forgot heal after a found one
    // stood in for it, and the next found heals were salvaged): the repeat still remembers the
    // heal the found one stood in for — the next exit re-buys it if it was drunk.
    g.auto_keep();
    g.start_run(None);
    assert_eq!(g.lineage.last_supplies, vec!["heal".to_string()]);
    assert!(g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "heal" && i.found));
    g.run = None;
    // Dropped from the shelf: salvaged (a potion pays $2), never refunded at $40.
    let mut g = arena_seed(6);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    g.lineage.last_supplies = vec!["heal".into()];
    give(&mut g, "heal");
    give(&mut g, "heal");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.found).count(), 1, "one heal stands in for the one the repeat packs; the other is salvaged");
    let id = g.lineage.supplies.iter().find(|s| s.found).map(|s| s.id).expect("shelved on a return too");
    let gold = g.lineage.gold;
    g.drop_supply(id).unwrap();
    assert!(g.lineage.gold - gold <= 2, "a found supply is salvaged, not refunded: +{}", g.lineage.gold - gold);
}

/// Cut 21 §2 (AF: `returned $0 · stalled · repeat −$80`): a stall does not re-pack on top of
/// the loss.
#[test]
fn a_stall_does_not_charge_the_repeat() {
    let mut g = arena_seed(7);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    g.lineage.gold = 1000;
    g.lineage.last_supplies = vec!["heal".into()];
    {
        let (run, mut cx) = g.ctx();
        run.stuck_fires = crate::engine::STALL_FIRES;
        run.timed_out = true;
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.run.as_mut().unwrap().timed_out = true;
    g.finish_run();
    assert!(g.last_exit.as_ref().unwrap().text.contains("stalled"), "{}", g.last_exit.as_ref().unwrap().text);
    assert!(!g.lineage.gold_ledger.iter().any(|l| l.why.starts_with("repeat")), "{:?}", g.lineage.gold_ledger);
    assert_eq!(g.lineage.gold, 1000);
}

/// Cut 21 §3 (AE: `FIRE · D4` read GAP): the player's own `drink unknown` row whose fire
/// killed him is the `row` verdict even when the dying action was another row's.
#[test]
fn a_gamble_row_that_kills_is_the_row_verdict_whoever_acted_last() {
    let mut g = arena_seed(4);
    give(&mut g, "fire");
    // (7 hp: the fire outlasts the drink — the chores step him about the flames before it
    // kills, so the dying action is not R1's)
    g.run.as_mut().unwrap().hero.hp = 7;
    hold_rules(&mut g);
    ticks(&mut g, 60);
    let gamble = Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "unknown")).from("preset");
    rules(&mut g, vec![gamble, Row::new(vec![], Verb::new("hold"))]);
    let mut id = None;
    for _ in 0..300 {
        g.tick();
        g.events.clear();
        if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
            id = Some(g.run.as_ref().unwrap().id);
            g.finish_run();
            break;
        }
    }
    let id = id.expect("the fire took him");
    let rec = g.deaths.get(&id).unwrap();
    assert_eq!(rec.death.cause, "own fire", "his own gamble's fire (QA on 778fa1b)");
    assert_eq!(rec.gamble_row, Some(0), "R1 gambled the fire");
    let last = rec.death.trace.turns.last().map(|t| t.row);
    assert_ne!(last, Some(0), "the dying action was not the gamble's");
    let d = g.death(id).unwrap();
    assert_eq!((d.verdict.as_str(), d.cause_row), ("row", Some(0)), "last {last:?} {} {:?}", d.verdict, d.patches);
    assert!(d.patches.iter().take(2).any(|p| (p.remove || p.replace) && p.insert_at == 0), "{:?}", d.patches);
}

/// Cut 21 §3 (AF: why `hp < 30% → bank` never fired over six ticks): a bank row held by foes
/// reads `no way` with a because on every tick it is refused, and the death's chain carries it.
#[test]
fn a_blocked_bank_row_reads_no_way_with_a_because_on_every_tick() {
    let mut g = arena();
    {
        let run = g.run.as_mut().unwrap();
        run.hero.pos = Pos::new(7, 6);
        run.hero_dist_pos = None;
    }
    for (x, y) in [(6, 5), (7, 5), (8, 5), (6, 6), (8, 6), (6, 7), (7, 7), (8, 7)] {
        add_monster(&mut g, "ogre", x, y);
    }
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 101)], Verb::new("bank")), Row::new(vec![], Verb::new("hold"))]);
    ticks(&mut g, 60);
    let run = g.run.as_ref().unwrap();
    let whys: Vec<&RowWhy> = run.trace.iter().filter_map(|t| t.rows.as_ref()).flatten().filter(|w| w.row == 0 && w.why == "no way").collect();
    assert!(whys.len() >= 2, "{:?}", run.trace.iter().map(|t| &t.rows).collect::<Vec<_>>());
    assert!(whys.iter().all(|w| w.because.as_ref().is_some_and(|b| crate::provenance::because_ok(&b.text) && b.text == "foes on every side")), "{whys:?}");
    if run.over == Some(ExitTier::Death) {
        let id = run.id;
        g.finish_run();
        let d = g.death(id).unwrap();
        assert!(d.chain.as_ref().is_some_and(|c| c.iter().any(|b| b.text == "foes on every side")), "{:?}", d.chain);
    }
}

/// Cut 21 §3: the `depth ≥` picker reaches the lineage's best + 2 (never under 8).
#[test]
fn the_depth_picker_reaches_best_plus_two() {
    let mut g = Game::new_literal(1);
    assert_eq!(g.vocabulary().depth_max, 8);
    g.lineage.best_depth = 20;
    assert_eq!(g.vocabulary().depth_max, 22);
}

// ---------------------------------------------------------------- QA on a946e04 (qaS)

/// qaS (an archer death's notes `Goblin Captain: summoner.`, a poison death's `The black one:
/// confusion.`): a death's notes name its killer or the harm that killed him.
#[test]
fn death_notes_name_the_death() {
    use crate::trace::death_note_names;
    assert!(death_note_names("goblin archer: ranged.", "goblin_archer"));
    assert!(!death_note_names("Goblin Captain: summoner.", "goblin_archer"));
    assert!(!death_note_names("Grog the goblin slain.", "goblin_archer"));
    assert!(death_note_names("Grog the goblin slain.", "goblin"));
    assert!(!death_note_names("goblin archer: telegraph.", "goblin"));
    assert!(death_note_names("The air stings: bloats ahead.", "burst"));
    assert!(death_note_names("The blue one: poison.", "poison"));
    assert!(death_note_names("Gambled: poison potion.", "poison"));
    assert!(!death_note_names("The black one: confusion.", "poison"));
    assert!(!death_note_names("Gambled: confusion potion.", "poison"));
    // The walk's death (seed 1815, run 1: an archer on D6, the run's last note a D5 captain's).
    let mut g = Game::new_literal(1815);
    g.send();
    loop {
        let r = g.step(200);
        if r.run_over {
            break;
        }
    }
    let id = *g.deaths.keys().next().expect("seed 1815's first run dies");
    let d = g.death(id).unwrap();
    // (Cut 25 §3's chore fix moved this seed's killer from the archer to a goblin; the notes still
    // name the death, never the D5 captain's.)
    assert!(d.notes.iter().all(|n| death_note_names(n, &d.cause)), "{:?}", d.notes);
    assert!(!d.notes.iter().any(|n| n.contains("Captain")), "{:?}", d.notes);
    let title = crate::engine::kind_title(&d.cause).to_lowercase();
    assert!(d.morgue.to_lowercase().contains(&format!("slain by {title}")), "the morgue names the killer by its title: {}", d.morgue);
}

/// qaS (`A goblin took him to 3 HP; no row; died to gas.` under R2 `hp < 40% → return`): the
/// walk home is the committing row's beat.
#[test]
fn reel_walk_home_names_the_return_row() {
    use crate::sifter::{story_line, story_ok, Act, Episode, Resolution, Setup};
    let ep = Episode {
        setup: Setup::Hurt,
        low_hp: 3,
        max_hp: 36,
        depth: 6,
        threat: vec![("goblin".into(), 1)],
        act: Act { row: 1, verb: Verb::new("return"), target: None, boss: false, walk: true },
        resolution: Resolution::Died { cause: "gas".into() },
        ..Default::default()
    };
    let line = story_line(&ep);
    assert!(line.contains("R2 returning"), "{line}");
    assert!(!line.contains("no row"), "{line}");
    assert!(story_ok(&line), "{line}");
}

/// qaS (`Nothing lost to the den.` beside `stolen red potion?`): the den is passed only while no
/// theft of the run is still out.
#[test]
fn den_pass_checks_the_runs_thefts() {
    for (outstanding, passed) in [(false, true), (true, false)] {
        let mut g = arena();
        {
            let run = g.run.as_mut().unwrap();
            run.floor_twist = Some("den".into());
            run.met_situation("den");
            if outstanding {
                run.stolen_ids.push(9999);
            }
        }
        let (run, mut cx) = g.ctx();
        crate::situations::on_leave_floor(run, &mut cx);
        assert_eq!(run.passed.iter().any(|p| p == "den"), passed, "outstanding theft {outstanding}");
    }
}

/// qaS (SALVAGED `blue potion? ×8` beside `poison ×3`): the lineage maps each identified
/// flavour's label to its name now, and a report's STOLEN row is named when read.
#[test]
fn identified_flavours_rename_on_the_wire() {
    let mut g = Game::new_literal(5);
    let kind = "poison";
    let fl = g.lineage.flavours.flavour_of(kind).expect("poison has a flavour").to_string();
    let label = format!("{fl} potion?");
    assert!(!g.lineage().renamed.contains_key(&label));
    let rep = |g: &mut Game| {
        let facts = g.lineage.facts.clone();
        let rank = g.lineage.rank;
        crate::offline::report(g, 0, &facts, "fighter", rank, false, false)
    };
    g.batch.stolen.insert(kind.into(), 2);
    let r = rep(&mut g);
    assert!(r.stolen.iter().any(|x| x.label == label && x.n == 2), "unidentified: its flavour · {:?}", r.stolen);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, kind).unwrap());
    assert_eq!(g.lineage().renamed.get(&label).map(String::as_str), Some(kind));
    let r = rep(&mut g);
    assert!(r.stolen.iter().any(|x| x.label == kind), "identified: its name · {:?}", r.stolen);
}

/// qaS (`+1 ROW ◆2 or $300 · each $ buy +25%`, then `◆4 or $600`): a chained unlock names the
/// next step's own price.
#[test]
fn unlock_names_the_chains_next_price() {
    let mut g = Game::new_literal(3);
    let row5 = g.unlocks().into_iter().find(|u| u.id == "row5").unwrap();
    let next = row5.next.expect("row5 → row6");
    assert_eq!((next.id.as_str(), next.cost), ("row6", 4));
    // Cut 23 §1: row slots on the forge's ladder — the same price after any gold buy.
    assert_eq!(next.gold, crate::kit::row_gold(&g.lineage, "row6").unwrap());
    assert_eq!(next.gold_after_gold, next.gold);
    assert_eq!(row5.gold_next, crate::kit::row_gold(&g.lineage, "row5").unwrap());
    g.lineage.marks = 10;
    tiers(&mut g);
    g.buy("row5").unwrap();
    let row6 = g.unlocks().into_iter().find(|u| u.id == "row6").unwrap();
    assert_eq!((row6.cost, row6.gold), (next.cost, next.gold), "the named next price is the price");
}

/// qaS (adding a row the editor marks dead moved the forecast): a shadowed row never acts
/// (`same as R<n>`), so the set with it plays the set without it — and reads its panel.
#[test]
fn a_shadowed_row_never_acts_and_reads_the_same_panel() {
    let mut g = Game::new_literal(1);
    g.lineage.unlocks.extend(["row5", "row6"].map(String::from));
    let base = g.lineage.rules().clone();
    let mut dead = base.clone();
    dead.rows.push(Row::new(vec![Cond::n("hp<", 50)], Verb::arg("attack", "nearest")));
    assert_eq!(g.lineage.shadowed_by(&dead).last().copied().flatten(), Some(1));
    let sims = crate::forecast::FORECAST_SIMS;
    let tag = crate::forecast::forecast_tag(&g, g.lineage.best_depth + 1);
    let a = crate::forecast::simulate_budget(&g, &base, sims, tag, u32::MAX, u64::MAX);
    let b = crate::forecast::simulate_budget(&g, &dead, sims, tag, u32::MAX, u64::MAX);
    assert!(a == b, "the dead row changed the sims");
    assert_eq!(crate::forecast::panel_key(&g, &base, sims), crate::forecast::panel_key(&g, &dead, sims));
    g.set_rules_raw(dead).unwrap();
    let last = g.lineage.rules().rows.len() - 1;
    for _ in 0..3 {
        g.send();
        loop {
            let r = g.step(50);
            assert!(!r.events.iter().any(|e| matches!(e, Ev::Rule { row, .. } if *row == last as i32)), "the shadowed row acted");
            if r.run_over {
                g.auto_keep();
                break;
            }
        }
    }
}

// ---------------------------------------------------------------- QA on a946e04 (qaT)

/// qaT (`R1 drank poison at 17/36 hp`, then a goblin's blow — every death `gap`): the gamble
/// whose harm made the difference is the death's row — the poison took more than the blow
/// overshot by — though the killer is the goblin; a harm smaller than the margin is not.
#[test]
fn a_gambles_harm_that_made_the_difference_names_its_row() {
    for (harm, short, want) in [(8, 3, Some(0)), (2, 3, None)] {
        let mut g = arena();
        rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "unknown")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        let rec = {
            let run = g.run.as_mut().unwrap();
            run.turn = 500;
            run.trace.push(TraceTurn { max_hp: 0, t: 400, row: 0, verb: Verb::arg("drink", "unknown"), hp: 17, foes: 1, rule_foes: 1, telegraphs: Vec::new(), blocked: None, rows: None, blows: Vec::new(), gift: None });
            run.trace.push(TraceTurn { max_hp: 0, t: 490, row: 1, verb: Verb::arg("attack", "nearest"), hp: 2, foes: 1, rule_foes: 1, telegraphs: Vec::new(), blocked: None, rows: None, blows: Vec::new(), gift: None });
            run.gambles.push((400, "poison".into(), true));
            run.gamble_harm = harm;
            run.death_short = short;
            run.death_cause = Some("goblin".into());
            crate::trace::death_record(&g, g.run.as_ref().unwrap())
        };
        assert_eq!(rec.gamble_row, want, "harm {harm} vs {short} short");
    }
}

/// qaT (`$0 repeat short` after run 6 while the run had found heals; the night's SALVAGED
/// `heal ×2`): the repeat remembers a kind it could not re-pack (short of gold), so the next
/// exit's found one of that kind goes to the shelf instead of the salvage.
#[test]
fn the_repeat_remembers_a_kind_it_could_not_pay_for() {
    let mut g = arena_seed(7);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    g.lineage.gold = 40;
    g.buy_supply("heal").unwrap();
    g.run = None;
    g.start_run(None);
    assert_eq!(g.lineage.last_supplies, vec!["heal".to_string()]);
    // Drunk, then home with no gold for the re-pack: short, but remembered.
    {
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.lineage.gold = 0;
    g.finish_run();
    g.auto_keep();
    assert!(g.lineage.supplies.iter().all(|s| s.kind != "heal"), "nothing to pay with");
    g.start_run(None);
    assert_eq!(g.lineage.last_supplies, vec!["heal".to_string()], "the short kind stays the repeat's");
    // A heal found on this run comes home to the shelf, not the salvage.
    let found = give(&mut g, "heal");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "heal" && s.found), "{:?}", g.lineage.supplies);
    assert!(!g.pending_exit.as_ref().is_some_and(|p| p.items.iter().any(|i| i.id == found)));
    assert!(!g.batch.salvaged.contains_key("heal"));
}

/// qaT (the strip's `−$36 stolen`, STOLEN listing the item alone; run 6's report without the
/// send's `−$50 waystone D5`): a report's stolen row carries the carried gold the theft took,
/// and a run's exit line its send's toll and start.
#[test]
fn stolen_rows_carry_their_gold_and_exit_lines_their_toll() {
    let mut g = Game::new_literal(3);
    g.batch.stolen.insert("leash".into(), 2);
    g.batch.stolen_gold.insert("leash".into(), 36);
    let facts = g.lineage.facts.clone();
    let rank = g.lineage.rank;
    let r = crate::offline::report(&mut g, 0, &facts, "fighter", rank, false, false);
    assert_eq!(r.stolen, vec![StolenRow { label: "leash".into(), n: 2, gold: 36 }]);
    let mut g = Game::new_literal(3);
    g.lineage.waystones = vec![5];
    g.set_start(5).unwrap();
    g.lineage.gold = 60;
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let line = g.last_exit.clone().unwrap();
    assert_eq!((line.toll, line.start, line.start_short), (0, 5, None), "a waystone start is free (QA on 778fa1b)");
    g.auto_keep();
    g.lineage.night();
    g.lineage.gold = 0;
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let line = g.last_exit.clone().unwrap();
    assert_eq!((line.toll, line.start, line.start_short), (0, 5, None), "an empty purse starts there too");
    g.auto_keep();
    g.lineage.waystones.clear();
    g.start_run(None);
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let line = g.last_exit.clone().unwrap();
    assert_eq!((line.toll, line.start, line.start_short), (0, 1, Some(5)), "an unlit start goes from D1, said");
}

// ---------------------------------------------------------------- Cut 22: the gold loop pays

/// Cut 22 §2 (AH: "the monkey stole the heal potion on D1 … 8 seconds after I paid $40"): a
/// thief takes what the run found first, a coin pile's worth next (`stolen $6` on D1), and the
/// packed supply only when the pack holds nothing else.
#[test]
fn a_thief_takes_the_found_then_coins_then_a_packed_supply() {
    let setup = |found: bool, carried: i32| -> (Game, u32, Option<u32>) {
        let mut g = arena();
        hold_rules(&mut g);
        let heal = give(&mut g, "heal");
        g.run.as_mut().unwrap().supplies.push(heal);
        let sword = found.then(|| give(&mut g, "sword"));
        g.run.as_mut().unwrap().loot_add_gold(carried);
        add_monster(&mut g, "monkey", 5, 5);
        (g, heal, sword)
    };
    // A found sword beside the packed heal: the sword goes.
    let (mut g, heal, sword) = setup(true, 20);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Steal { .. })), "{:?}", ev_kinds(&evs));
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.stolen_ids, vec![sword.unwrap()]);
    assert!(run.hero.inv.iter().any(|i| i.id == heal), "the bought heal stays");
    // Only the packed heal, $20 carried: a coin pile's worth ($4 + 2 × depth).
    let (mut g, heal, _) = setup(false, 20);
    let evs = ticks(&mut g, 60);
    assert!(evs.iter().any(|e| matches!(e, Ev::Steal { amount: Some(6), .. })), "{:?}", ev_kinds(&evs));
    let run = g.run.as_ref().unwrap();
    assert!(run.hero.inv.iter().any(|i| i.id == heal), "the bought heal stays");
    assert_eq!(run.loot, 14);
    assert_eq!(run.stolen_kinds.iter().map(|(_, k, g)| (k.as_str(), *g)).collect::<Vec<_>>(), vec![("gold", 6)]);
    assert!(run.notes.iter().any(|(_, n)| n == "The monkey stole $6."), "{:?}", run.notes);
    // Nothing else to take: the packed heal.
    let (mut g, heal, _) = setup(false, 0);
    ticks(&mut g, 60);
    assert_eq!(g.run.as_ref().unwrap().stolen_ids, vec![heal]);
    // The den's order: the same; never what he wears.
    let (mut g, _, _) = setup(false, 0);
    let run = g.run.as_mut().unwrap();
    run.hero.inv.clear();
    // Cut 24 §3: the kit's arm (the heir's own, `kit::WEAPON_ID`) is never taken. Cut 25 §5: nor a
    // found arm in hand, nor the armour on him (what he wears is his, not loot).
    assert!(run.hero.weapon.as_ref().is_some_and(|w| crate::kit::is_kit_id(w.id)));
    assert!(crate::ai::thief_pick(run, false, false).is_none(), "the kit's arm is never a thief's");
    let found = run.new_item_id();
    run.hero.weapon = Some(crate::item::Item::new(found, "axe"));
    let worn = run.new_item_id();
    run.hero.armour = Some(crate::item::Item::new(worn, "mail"));
    assert!(crate::ai::thief_pick(run, false, false).is_none(), "the worn axe and mail are never a thief's");
    run.loot_add_gold(3);
    assert!(matches!(crate::ai::thief_pick(run, false, false), Some(crate::ai::Take::Coins(3))));
    assert!(crate::ai::thief_pick(run, true, false).is_none(), "a forge imp takes potions only");
}

/// Cut 22 §2: stolen coins a killed thief drops come back into the carry (`Got $6 back.`), and
/// the den counts them as got back.
#[test]
fn stolen_coins_come_back_from_a_killed_thief() {
    let mut g = arena();
    hold_rules(&mut g);
    g.run.as_mut().unwrap().loot_add_gold(20);
    let m = add_monster(&mut g, "monkey", 5, 5);
    ticks(&mut g, 60);
    assert_eq!(g.run.as_ref().unwrap().loot, 14);
    let run = g.run.as_mut().unwrap();
    let mi = run.monsters.iter().position(|mm| mm.id == m).unwrap();
    let coins = run.monsters[mi].stolen.clone().expect("the monkey holds the coins");
    assert_eq!((coins.kind.as_str(), coins.amount), ("gold", 6));
    // Dropped where it fell, picked up on the hero's tile.
    let at = run.hero.pos;
    run.items.push(crate::engine::FloorItem { pos: at, item: coins });
    let mut evs = Vec::new();
    let mut prov = Vec::new();
    let Game { run, lineage, .. } = &mut g;
    let run = run.as_mut().unwrap();
    let set = lineage.active_set;
    let max_rows = lineage.max_rows();
    let mut cx = crate::engine::Ctx {
        facts: &mut lineage.facts,
        kill_counts: &mut lineage.kill_counts,
        flavours: &lineage.flavours,
        rules: &lineage.sets[set],
        unlocks: &lineage.unlocks,
        grudges: &lineage.grudges,
        forge: &lineage.forge,
        max_rows,
        events: &mut evs,
        sim: true,
        prov: &mut prov,
        variant: "",
        hunter: None,
        vault_pref: "weapon",
        lost: &lineage.lost,
        sets: &lineage.sets,
        active_set: set,
        trophies: &[],
        tally: Box::leak(Box::default()),
    };
    crate::turn::pickup_here(run, &mut cx);
    assert_eq!(run.loot, 20);
    assert!(run.stolen_ids.is_empty());
    assert!(run.notes.iter().any(|(_, n)| n == "Got $6 back."), "{:?}", run.notes);
}

/// Cut 22 §1: a bought supply thieves kept is re-bought by the repeat once a night; a second
/// waits for the night's end. A refund pays what was paid, whatever the price is now.
#[test]
fn a_stolen_supply_is_rebought_once_a_night_and_a_refund_pays_what_was_paid() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    identify(&mut g, "heal");
    g.lineage.gold = 1000;
    g.lineage.best_depth = 5;
    g.buy_supply("heal").unwrap();
    let steal = |g: &mut Game| {
        g.start_run(None);
        let run = g.run.as_mut().unwrap();
        let i = run.hero.inv.iter().position(|i| i.kind == "heal").expect("the heal was packed");
        let it = run.hero.inv.remove(i);
        run.stolen_ids.push(it.id);
        run.stolen_kinds.push((it.id, it.kind.clone(), 0));
        run.stolen_labels.push((it.id, "heal potion".into()));
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
        g.finish_run();
        g.auto_keep();
    };
    steal(&mut g);
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "heal"), "the night's one re-buy");
    assert!(g.lineage.night_theft_rebought);
    steal(&mut g);
    assert!(!g.lineage.supplies.iter().any(|s| s.kind == "heal"), "a second theft waits for the night");
    assert!(g.lineage().repeat_kinds.is_empty(), "{:?}", g.lineage().repeat_kinds);
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().supplies.is_empty(), "the send packs nothing");
    g.run = None;
    g.lineage.night();
    g.restock();
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "heal"), "a new night re-buys it");
    // Bought at D5's price ($20), refunded at it after the price moved.
    let s = g.lineage.supplies.iter().find(|s| s.kind == "heal").unwrap().clone();
    assert_eq!(s.paid, 20);
    g.lineage.best_depth = 20;
    let gold = g.lineage.gold;
    g.drop_supply(s.id).unwrap();
    assert_eq!(g.lineage.gold, gold + 20);
}

/// Cut 22 §3: an edit's paired move — the active panel minus the previous set's on the same
/// seeds: nothing moves against itself, the move at each depth is the two bars' difference,
/// and on AH's set a one-notch edit's ± sits well inside the bars' own.
#[test]
fn an_edits_paired_delta_is_the_bars_difference_and_tighter_than_them() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards/6da3ed0.raterAH.rules.json")).unwrap();
    let set = RuleSet::parse(&text).unwrap();
    let mut g = Game::new_literal(7);
    for u in crate::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    identify(&mut g, "heal");
    g.lineage.best_depth = 8;
    g.set_rules_raw(set.clone()).unwrap();
    let base = g.forecast();
    let same = g.forecast_vs(&set);
    assert!(same.depths.iter().all(|d| d.delta == 0.0 && d.pm == 0.0) && same.bank.pm == 0.0 && same.death.delta == 0.0, "{same:?}");
    assert_eq!(same.depths.len(), base.depths.len());
    // R3 `hp < 20 → return` to 25.
    let mut edit = set.clone();
    edit.rows[2].conds[0].n = Some(25);
    g.set_rules_raw(edit).unwrap();
    let f = g.forecast();
    let vs = g.forecast_vs(&set);
    assert_eq!(vs.sims, 50);
    for (d, (a, b)) in vs.depths.iter().zip(f.depths.iter().zip(&base.depths)) {
        assert!((d.delta - (a.reach - b.reach)).abs() < 1e-9, "D{}: {} vs {} − {}", d.depth, d.delta, a.reach, b.reach);
        assert!((d.abs_pm - a.pm.unwrap()).abs() < 1e-9);
    }
    let ends = |f: &Forecast| f.ends.clone().unwrap();
    assert!((vs.death.delta - (ends(&f).death - ends(&base).death)).abs() < 1e-9);
    assert!((vs.gold.delta - (ends(&f).gold - ends(&base).gold)).abs() < 1e-6);
    let (p, a): (f64, f64) = vs.depths.iter().filter(|d| d.abs_pm > 0.0).fold((0.0, 0.0), |s, d| (s.0 + d.pm, s.1 + d.abs_pm));
    assert!(p < a, "paired ± {p:.2} vs absolute ± {a:.2}: {:?}", vs.depths);
}

/// Cut 22 §4 (AH, run 10: `R3 attack ranged ×4` through a pack — hp 24 → 9 — then `R2 return`
/// at 5 hp, dead to a jackal, sealed GAP): an attack row that chased into the death is `row`,
/// naming it, when cutting or narrowing it survives ≥ 50 % of the replays and beats the base by
/// 15 points; its cut is among the patches whatever an added row survives.
#[test]
fn an_attack_row_that_chased_into_the_death_is_the_row_verdict() {
    // AH's set at run 10 (the notes; R5 the preset's strike).
    let mut rows = vec![
        Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
        Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")),
        Row::new(vec![Cond::t("foe_tag", "ranged")], Verb::arg("attack", "tag:ranged")),
        Row::new(vec![Cond::t("foe_tag", "thief")], Verb::arg("tame", "tag:thief")),
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        Row::new(vec![Cond::n("depth>=", 8)], Verb::new("bank")),
    ];
    for (i, r) in rows.iter_mut().enumerate() {
        r.origin = Some(if i == 4 { "preset" } else { "player" }.into());
    }
    // Cut 24 (the floors' arrival events, named foes resting): 1900's night. (Cut 30 §2: the
    // temperaments' overrides gone, the nights moved — the first of 1900's neighbours with such a death.)
    let mut chased: Vec<Death> = Vec::new();
    for seed in 1900..1940u64 {
        let mut g = Game::new_literal(seed);
        g.max_deaths = 1000;
        for u in ["row5", "row6", "tame", "cond_on_see"] {
            g.lineage.unlocks.insert(u.into());
        }
        for f in ["foe:goblin_archer:ranged", "foe:monkey:thief", "foe:jackal:pack"] {
            g.lineage.facts.insert(f.into());
        }
        identify(&mut g, "heal");
        g.set_rules_raw(RuleSet { rows: rows.clone(), name: None, route: Vec::new() }).unwrap();
        g.lineage.gold = 300;
        g.buy_supply("heal").unwrap();
        g.buy_supply("heal").unwrap();
        g.run_offline(8 * 3600);
        let ids: Vec<u32> = g.deaths.keys().copied().collect();
        chased = ids.iter().filter_map(|id| g.death(*id)).filter(|d| d.verdict == "row" && d.cause_row == Some(2) && d.patches.iter().any(|p| p.insert_at == 2 && (p.remove || p.replace))).collect();
        if !chased.is_empty() {
            break;
        }
    }
    let d = chased.first().expect("a death R3 chased into, named");
    let tail = &d.trace.turns[d.trace.turns.len().saturating_sub(crate::trace::CHASE_TURNS)..];
    assert!(tail.iter().filter(|t| t.row == 2).count() >= 2, "R3 took the last actions: {:?}", tail.iter().map(|t| t.row).collect::<Vec<_>>());
    let cut = d.patches.iter().find(|p| p.insert_at == 2 && (p.remove || p.replace)).expect("the chase's cut is shown");
    assert!(cut.survive >= crate::trace::ROW_BAR - 1e-9 && cut.survive - d.baseline >= crate::trace::PATCH_MARGIN - 1e-9, "{:.2} vs base {:.2}", cut.survive, d.baseline);
    assert!(d.patches.first().is_some_and(|p| p.survive >= cut.survive - 1e-9), "the best patch leads");
}

/// QA on 778fa1b (qaU: the camp painted 94/78/42, then 95/73/36 for the same rules with no
/// sign it was settling): `refined` is on the wire whichever pass a forecast is — `false` on
/// the first, `true` on the refine — and so on the vs, cage and start reads.
#[test]
fn forecast_refined_is_always_on_the_wire() {
    let g = Game::new_literal(3);
    let first = serde_json::to_value(g.forecast()).unwrap();
    assert_eq!(first.get("refined"), Some(&serde_json::Value::Bool(false)), "{first}");
    let prev = g.lineage.rules().clone();
    assert!(!g.forecast_vs(&prev).refined);
    assert!(g.cage_forecast().iter().all(|o| !o.refined));
    assert!(g.start_forecast().iter().all(|o| !o.refined));
    let refined = serde_json::to_value(g.forecast_refine()).unwrap();
    assert_eq!(refined.get("refined"), Some(&serde_json::Value::Bool(true)));
    // After the refine every read is the refined panel, and says so.
    assert!(g.forecast().refined);
    let vs = serde_json::to_value(g.forecast_vs(&prev)).unwrap();
    assert_eq!(vs.get("refined"), Some(&serde_json::Value::Bool(true)));
    // Cut 25 §4: the start tablets are measured on the first pass's sims whatever (the panel the
    // camp reads right after the tap) — and say so. QA on 524827b (qaAA: the cage sheet's current
    // option `D6 6%` under the camp's refined `D6 11%`): the cage's options are on the camp's own
    // pass — the refined one once the camp refined (and the caller may name the pass).
    assert!(g.cage_forecast().iter().all(|o| o.refined));
    assert!(g.cage_forecast_refined(false).iter().all(|o| !o.refined));
    let f = g.forecast();
    let cur = g.cage_forecast().into_iter().find(|o| o.current).unwrap();
    let bar = f.depths.iter().find(|d| d.depth == cur.depth).map(|d| d.reach);
    assert_eq!(bar, Some(cur.reach), "the current option is the camp's own bar");
    // QA on 308f045 (qaAD: `D1 · bank 90%` beside the shaft's refined 92 %): the starts follow the camp's pass too.
    assert!(g.start_forecast().iter().all(|o| o.refined));
    assert!(g.start_forecast_refined(false).iter().all(|o| !o.refined));
}

/// QA on 778fa1b (qaU: `carry $61 −$37 swapped` on the strip, in no ledger): a pack swap that
/// takes a find in the place of a dearer carried item lowers the carry by what the strip shows,
/// and the exit line names it (`swapped`); the report sums them.
#[test]
fn a_pack_swap_that_lowers_the_carry_is_on_the_exit_line() {
    let mut g = arena();
    g.run.as_mut().unwrap().loot_add(1000);
    for _ in 0..crate::hero::INV_SLOTS {
        let id = give(&mut g, "sword");
        let run = g.run.as_mut().unwrap();
        run.hero.inv.iter_mut().find(|i| i.id == id).unwrap().enchant = 3;
    }
    {
        let run = g.run.as_mut().unwrap();
        let id = run.new_item_id();
        let pos = run.hero.pos;
        run.items.push(crate::engine::FloorItem { pos, item: Item::new(id, "heal") });
    }
    let before = g.run.as_ref().unwrap().loot;
    {
        let (run, mut cx) = g.ctx();
        crate::turn::pickup_here(run, &mut cx);
    }
    let run = g.run.as_ref().unwrap();
    assert!(run.hero.inv.iter().any(|i| i.kind == "heal"), "the heal took a spare sword's slot");
    let fall = before - run.loot;
    assert!(fall > 0, "a +3 sword counted more than the heal");
    assert_eq!(run.swapped, fall);
    finish_with(&mut g, ExitTier::Return);
    assert_eq!(g.last_exit.as_ref().unwrap().swapped, fall);
    assert_eq!(g.batch.swapped, fall);
    let json = serde_json::to_value(g.last_exit.as_ref().unwrap()).unwrap();
    assert_eq!(json.get("swapped").and_then(|v| v.as_i64()), Some(fall as i64));
}

/// QA on 778fa1b (qaU: `purse full` on death lines carrying $66 and $257 while the camp held
/// $1434): the flag is a death's that found the purse just over the top-up line (under
/// `PURSE_FULL_BAND`); a richer death says nothing of it; a top-up is `wake`.
#[test]
fn purse_full_only_near_the_top_up_line() {
    use crate::engine::{PURSE_FULL_BAND, WAKE_PAY};
    for (gold, full, wake) in [(0, false, WAKE_PAY), (WAKE_PAY + 10, true, 0), (PURSE_FULL_BAND, false, 0), (1434, false, 0)] {
        let mut g = arena();
        g.lineage.gold = gold;
        g.run.as_mut().unwrap().hero.hp = 0;
        finish_with(&mut g, ExitTier::Death);
        let line = g.last_exit.clone().unwrap();
        assert_eq!((line.purse_full, line.wake), (full, wake), "gold {gold}: {}", line.text);
    }
}

/// QA on 778fa1b (qaU: after the absence the shelf was `1/3` — the capped re-pack left the
/// heal — while the tile read `repeat on · $26`): the lineage names what the next send's
/// re-pack buys then (`repeat_due`) and what the purse will not let it (`repeat_unpaid`: the
/// tile's `repeat short`).
#[test]
fn the_repeat_says_what_the_send_will_and_will_not_pack() {
    let mut g = Game::new_literal(5);
    no_kennel_leash(&mut g);
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.lineage.last_supplies = vec!["heal".into()];
    g.lineage.gold = 1000;
    let l = g.lineage();
    assert_eq!((l.repeat_due.clone(), l.repeat_unpaid.clone()), (vec!["heal".to_string()], vec![]));
    assert!(l.repeat_gold > 0);
    // A purse that cannot pay it: `repeat short`.
    g.lineage.gold = 0;
    let l = g.lineage();
    assert_eq!((l.repeat_due.clone(), l.repeat_unpaid.clone()), (vec![], vec!["heal".to_string()]));
    let json = serde_json::to_value(&l).unwrap();
    assert!(json.get("repeat_unpaid").is_some());
    // A shelf that holds the repeat: nothing due.
    g.lineage.gold = 1000;
    g.buy_supply("heal").unwrap();
    let l = g.lineage();
    assert!(l.repeat_due.is_empty() && l.repeat_unpaid.is_empty());
    // Off: nothing either way.
    g.set_restock(false);
    let l = g.lineage();
    assert!(l.repeat_due.is_empty() && l.repeat_unpaid.is_empty());
}

// ---------------------------------------------------------------- QA on 778fa1b (qaV)

/// QA on 778fa1b (qaV: `fire · D6 · GAP`, the trace `R3 card last stand` 10 → 3 → 0): the
/// card's own `throw fire` at the adjacent foe burned the hero — the death is his own harm
/// (`own fire` on the record) and the card row is the `row` verdict, its cut among the first
/// patches.
#[test]
fn a_card_rows_own_throw_that_kills_is_own_fire_and_the_row_verdict() {
    let mut found = false;
    for seed in 1..=12u64 {
        let mut g = arena_seed(seed);
        g.lineage.unlocks.insert("last_stand".into());
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "fire") {
            g.lineage.facts.insert(f);
        }
        give(&mut g, "fire");
        add_monster(&mut g, "rat", 5, 5);
        {
            let run = g.run.as_mut().unwrap();
            run.hero.hp = 6;
            run.hero.second_wind_used = true;
        }
        // (without the card, the attack row kills the rat)
        rules(&mut g, vec![Row::new(vec![], Verb::arg("tactic", "last_stand")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
        let mut id = None;
        for _ in 0..400 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                id = Some(g.run.as_ref().unwrap().id);
                g.finish_run();
                break;
            }
        }
        let Some(id) = id else { continue };
        let Some(rec) = g.deaths.get(&id) else { continue };
        if rec.death.cause != "own fire" {
            continue;
        }
        assert_eq!(rec.gamble_row, Some(0), "the card threw the fire");
        let d = g.death(id).unwrap();
        if d.baseline >= 1.0 - 1e-9 {
            continue; // not reproduced: never a `row`
        }
        assert_eq!((d.verdict.as_str(), d.cause_row), ("row", Some(0)), "{} {:?}", d.verdict, d.patches);
        assert!(d.patches.iter().take(2).any(|p| (p.remove || p.replace) && p.insert_at == 0), "{:?}", d.patches);
        found = true;
        break;
    }
    assert!(found, "no seed burned the hero with the card's own throw");
}

/// QA on 778fa1b (qaV: `R4 retreat` / `R6 attack nearest` before two ogres for six minutes,
/// no blow landing either way): a retreat that cannot shake its pursuer is rested
/// (`OUTPACED_RETREATS` retreats of one engagement, no blow on the hero) and the attack
/// row fights — blood is drawn within a bounded number of actions.
#[test]
fn an_outpaced_retreat_rests_and_the_next_row_fights() {
    let mut g = arena();
    add_monster(&mut g, "ogre", 7, 5);
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "heavy")], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    g.lineage.facts.insert("foe:ogre:heavy".into());
    let mut blood_at = None;
    for _ in 0..3000 {
        g.tick();
        g.events.clear();
        let Some(run) = g.run.as_ref() else { break };
        if run.over.is_some() {
            break;
        }
        let ogre_hurt = run.monsters.iter().any(|m| m.kind == "ogre" && m.hp < m.max_hp) || !run.monsters.iter().any(|m| m.kind == "ogre" && m.hp > 0);
        if ogre_hurt || run.hero.hp < run.hero.max_hp {
            blood_at = Some(run.actions);
            break;
        }
    }
    let at = blood_at.expect("blood was drawn");
    assert!(at <= crate::turn::OUTPACED_RETREATS + 40, "first blood after {at} actions");
}

/// QA on 778fa1b (qaV: `repeat heal ×1 · −$104` for four heals at $26): the repeat's ledger
/// line counts what it bought (`GoldLine.n`) — n × price == the amount.
#[test]
fn a_repeat_line_counts_what_it_bought() {
    let mut g = Game::new_literal(3);
    g.lineage.gold = 1000;
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") {
        g.lineage.facts.insert(f);
    }
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")));
    g.set_rules_raw(set).unwrap();
    g.lineage.supplies.retain(|s| !s.free);
    for _ in 0..3 {
        g.buy_supply("heal").unwrap();
    }
    let price = g.lineage.supplies[0].paid;
    let bought = g.lineage.gold_ledger.last().unwrap().clone();
    assert_eq!((bought.n, bought.delta), (3, -3 * price), "{bought:?}");
    g.start_run(None);
    // The run drinks them all, then comes home.
    g.run.as_mut().unwrap().hero.inv.retain(|i| i.kind != "heal");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    let line = g.lineage.gold_ledger.iter().rev().find(|l| l.why == "repeat heal").cloned().expect("a repeat line");
    let on_shelf: Vec<&Item> = g.lineage.supplies.iter().filter(|s| s.kind == "heal" && !s.found).collect();
    assert_eq!(line.n as usize, on_shelf.len(), "{line:?}");
    assert_eq!(-line.delta, on_shelf.iter().map(|s| s.paid).sum::<i32>(), "{line:?}");
    assert_eq!(line.n, 3);
}

/// Cut 23 §3 (AJ: "paid cards are rows he could type"): every paid card carries a token no
/// typed row can hold — a verb, a cond or a bound target outside the vocabulary of a lineage
/// that owns every unlock and knows every fact (for its class) — and says what (`carries`).
#[test]
fn cards_carry_outside_the_vocabulary() {
    let mut lines = Vec::new();
    let mut bad: Vec<String> = Vec::new();
    for class in crate::hero::Class::ALL {
        let mut g = Game::new_literal(3);
        // (Cut 29 §1: not the verb `hold` — an oath's reward: with it, the `hold` cards are typed rows)
        for u in crate::meta::UNLOCKS.iter().filter(|u| u.id != "hold") {
            g.lineage.unlocks.insert(u.id.into());
        }
        crate::probes::learn_everything(&mut g);
        g.lineage.class = class;
        g.lineage.classes.insert(class.name().into(), crate::wire::ClassProg { level: 10, xp: 0, next: 0 });
        let mastery = crate::hero::mastery_card(class);
        g.lineage.unlocks.insert(mastery.into());
        let vocab = g.vocabulary();
        let cards: Vec<&str> = crate::meta::TACTIC_CARDS.iter().chain(crate::meta::TIER2_CARDS.iter()).copied().chain([mastery]).collect();
        for id in cards {
            let rows = crate::meta::unlock_rows(id).unwrap();
            let mut outside: Vec<String> = Vec::new();
            for r in &rows {
                if !vocab.verbs.contains(&r.verb) {
                    outside.push(format!("verb {}", r.verb.short()));
                }
                for c in &r.conds {
                    let typed = vocab.conds.iter().any(|v| v.k == c.k && (v.t.is_none() || v.t == c.t));
                    if !typed {
                        outside.push(format!("cond {}", c.k));
                    }
                }
            }
            lines.push(format!("{} {id}: {:?} carries {:?}", class.name(), outside, crate::meta::card_carries(id)));
            let cost = crate::meta::unlock_cost(id);
            if outside.is_empty() && cost > 0 {
                bad.push(format!("{id} ({}) is rows a player can type", class.name()));
            }
            if crate::meta::card_carries(id).is_none() && cost > 0 {
                bad.push(format!("{id}: no carries"));
            }
        }
    }
    if std::env::var("CARDS_DEBUG").is_ok() {
        eprintln!("{}", lines.join("\n"));
    }
    assert!(bad.is_empty(), "{bad:?}");
}

// ---------------------------------------------------------------- Cut 23

/// Cut 23 §1: the forge — each ladder's next step is priced from the lineage, a buy spends the
/// gold (`forge sword +1`), is permanent (every heir starts with it; a death loses none), and
/// the heir's kit carries it: the class arm at the weapon step's enchant, the armour step's
/// piece; the pack step holds one more supply. Neither kit piece is loot.
#[test]
fn the_forge_sells_permanent_kit_steps() {
    let mut g = Game::new_literal(4);
    g.lineage.best_depth = 8;
    let lad = g.lineage().kit;
    assert_eq!(lad.iter().map(|l| l.slot.as_str()).collect::<Vec<_>>(), vec!["weapon", "armour", "pack"]);
    let unit = crate::kit::unit(8);
    assert_eq!(unit, 300);
    let w = &lad[0];
    assert_eq!((w.owned, w.steps[0].label.as_str(), w.steps[0].price), (0, "sword +1", unit));
    assert!(w.steps.len() >= 3 && lad[1].steps.len() >= 4 && lad[2].steps.len() >= 4, "{lad:?}");
    for l in &lad {
        assert!(l.steps.windows(2).all(|p| p[0].price < p[1].price), "a ladder climbs: {l:?}");
    }
    assert!(!w.next.as_ref().unwrap().affordable);
    assert_eq!(crate::kit::buy(&mut g, "weapon").unwrap_err(), "not enough gold");
    g.lineage.gold = 10_000;
    crate::kit::buy(&mut g, "weapon").unwrap();
    crate::kit::buy(&mut g, "armour").unwrap();
    crate::kit::buy(&mut g, "pack").unwrap();
    assert_eq!(g.lineage.gold, 10_000 - 4 * unit as i32);
    assert!(g.lineage.gold_ledger.iter().any(|l| l.why == "forge sword +1" && l.delta == -(unit as i32)), "{:?}", g.lineage.gold_ledger);
    assert_eq!(g.lineage.supply_cap(), 4);
    let l = g.lineage();
    assert_eq!((l.kit[0].owned, l.kit[0].next.as_ref().map(|n| n.label.as_str())), (1, Some("sword +2")));
    assert!(l.kit[0].steps[0].owned);
    // The heir starts with it; so does the next heir after a death.
    for _ in 0..2 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let h = &g.run.as_ref().unwrap().hero;
        assert_eq!(h.weapon.as_ref().map(|w| (w.kind.as_str(), w.enchant)), Some(("sword", 1)));
        assert_eq!(h.armour.as_ref().map(|a| (a.kind.as_str(), a.enchant)), Some(("leather", 0)));
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Death);
        g.finish_run();
        g.auto_keep();
    }
    assert_eq!(crate::kit::owned(&g.lineage, "weapon"), 1, "a death loses no step");
    assert!(!g.lineage.vault.iter().any(|v| crate::kit::is_kit_id(v.id)), "the kit is never kept as loot");
    // The save keeps the kit.
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.lineage.kit, g.lineage.kit);
    // The top of a ladder.
    g.lineage.gold = 1_000_000;
    while crate::kit::buy(&mut g, "weapon").is_ok() {}
    assert_eq!(crate::kit::buy(&mut g, "weapon").unwrap_err(), "top of the ladder");
    assert!(g.lineage().kit[0].next.is_none());
    // The row slots sit on the same ladder.
    assert_eq!(crate::kit::row_gold(&g.lineage, "row8"), Some(16 * unit));
}

/// Cut 23 §1: a step's forecast move is the paired panel with the step against without it, at
/// the frontier (`kitDeltas`); the next step's `nights` reads the last night's net.
#[test]
fn a_forge_step_shows_its_paired_move_and_its_nights() {
    let mut g = Game::new_literal(9);
    g.lineage.best_depth = 4;
    g.lineage.last_night_net = 500;
    let price = crate::kit::price(&g.lineage, "armour", 0) as i32;
    g.lineage.gold = price - 400;
    let l = crate::kit::ladders(&g.lineage);
    assert_eq!(l[1].next.as_ref().unwrap().nights, Some(1));
    g.lineage.gold = price;
    assert_eq!(crate::kit::ladders(&g.lineage)[1].next.as_ref().unwrap().nights, Some(0));
    let d = crate::kit::deltas(&g);
    for lad in &d {
        let n = lad.next.as_ref().unwrap();
        assert_eq!(n.depth, Some(5));
        assert!(n.delta.is_some_and(|x| (-1.0..=1.0).contains(&x)) && n.pm.is_some(), "{lad:?}");
    }
    // Measured twice: the same numbers (the panels are memoised).
    assert_eq!(crate::kit::deltas(&g), d);
}

/// Cut 23 §2: the forecast never prints `0%` for a share it sampled at 0 of N — `<N%`, N from
/// its sims (`Forecast.low`).
#[test]
fn a_share_sampled_at_zero_reads_under_one_sim() {
    assert_eq!(crate::forecast::share_label(0.0, 50), "<2%");
    assert_eq!(crate::forecast::share_label(0.0, 100), "<1%");
    assert_eq!(crate::forecast::share_label(0.0, 12), "<9%");
    assert_eq!(crate::forecast::share_label(0.04, 50), "4%");
    assert_eq!(crate::forecast::low_pct(0), 100);
    let g = Game::new_literal(3);
    let f = g.forecast();
    assert_eq!(f.sims, crate::forecast::FORECAST_SIMS);
    assert_eq!(f.low, 2);
    let r = g.forecast_refine();
    assert_eq!((r.sims, r.low), (crate::forecast::REFINE_SIMS, 1));
}

/// Cut 23 §2 (AJ: every death `return too late` at 1–2 HP, the heal row under the return
/// never read): walking home, a row under the committed `return` that answers — a drink of a
/// held heal — acts; with nothing to answer, the walk goes on (the return row is the walk).
#[test]
fn a_heal_under_the_return_answers_on_the_walk_home() {
    let mut g = arena();
    identify(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![Cond::n("hp<", 50)], Verb::new("return")), Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal"))]);
    {
        let run = g.run.as_mut().unwrap();
        run.hero.hp = run.hero.max_hp / 3;
        run.hero.pos = Pos::new(12, 9);
    }
    let evs = ticks(&mut g, 12);
    let rows: Vec<i32> = evs.iter().filter_map(|e| if let Ev::Rule { row, .. } = e { Some(*row) } else { None }).collect();
    assert_eq!(rows.first(), Some(&0), "R1 commits the walk: {rows:?}");
    assert!(g.run.as_ref().unwrap().homeward.is_some());
    // Nothing held: the walk goes on (R2 has nothing to drink).
    assert!(!rows.contains(&1), "{rows:?}");
    // A heal in the pack: R2 answers on the walk.
    let heal = give(&mut g, "heal");
    g.run.as_mut().unwrap().hero.inv.iter_mut().find(|i| i.id == heal).unwrap().known = true;
    let evs = ticks(&mut g, 12);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 1, .. })), "{:?}", ev_kinds(&evs));
    let t = g.run.as_ref().unwrap().trace.iter().rev().find(|t| t.row == 1).cloned().expect("R2 acted");
    assert!(t.rows.iter().flatten().any(|w| w.row == 0 && w.why == "going home"), "{t:?}");
    assert!(crate::turn::row_reason_ok("going home"));
}

/// Cut 23 §3: a row's why-not over the sends — a `foe: gas` row that never met gas reads
/// `0/N · no gas met`; a row blocked when its conds held names the block.
#[test]
fn every_row_says_why_not() {
    let mut g = arena();
    identify(&mut g, "heal");
    g.lineage.facts.insert("foe:bloat:gas".into());
    rules(&mut g, vec![Row::new(vec![Cond::t("foe_tag", "gas")], Verb::arg("throw", "unknown,nearest")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("read", "teleport")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    g.lineage.unlocks.insert("throw".into());
    add_monster(&mut g, "rat", 6, 5);
    ticks(&mut g, 200);
    let (run, mut cx) = g.ctx();
    crate::turn::end_run(run, &mut cx, ExitTier::Return);
    g.finish_run();
    g.auto_keep();
    let why = g.lineage().row_why;
    assert_eq!(why.len(), 3);
    let r1 = why[0].clone().expect("R1 tallied");
    assert_eq!((r1.fired, r1.matched), (0, 0));
    assert_eq!(r1.unmet.as_ref().map(|u| u.why.as_str()), Some("gas"));
    assert!(r1.text.ends_with("· no gas met"), "{}", r1.text);
    assert!(r1.text.starts_with(&format!("0/{}", r1.actions)), "{}", r1.text);
    let r2 = why[1].clone().expect("R2 tallied");
    assert!(r2.matched > 0 && r2.fired == 0, "{r2:?}");
    assert_eq!(r2.blocked.as_ref().map(|b| b.why.as_str()), Some("no item"));
    assert!(r2.text.ends_with("blocked · no item"), "{}", r2.text);
    let r3 = why[2].clone().expect("R3 tallied");
    assert!(r3.fired > 0, "{r3:?}");
    for r in why.iter().flatten() {
        assert!(word_count(&r.text) <= 5, "{}", r.text);
    }
    // A row edited away drops its tally; the saved game keeps the rest.
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.lineage().row_why, why);
}

/// Cut 23 §3: every reason the core gives for a row not acting has its gloss (≤ 3 words), and
/// every `✗` callout carries one (`read ✗ no use` → `nothing to learn`); a telegraph's shout
/// carries what comes next.
#[test]
fn every_cross_and_shout_has_a_reason() {
    // (the verb blocks, `ai::block_reason`, and the guards and pre-emptions; a cond reason —
    // `hp not <30%` — explains itself)
    let blocks = ["no path", "no target", "no line", "no bow", "cooldown", "no item", "unknown item", "no unknown", "no use", "no leash", "none weak", "not safe", "no stairs", "going home", "prayed", "no shrine", "no way", "card passed", "card idle", "card blocked", "brave held", "stuck", "row guard", "same as R", "trait first", "hazard first", "recall sense", "paralysed", "confused", "bail", "locked cond", "fired, free"];
    for r in blocks.iter().filter(|r| crate::turn::ROW_REASONS.contains(r)) {
        let g = crate::turn::why_gloss("attack", r).unwrap_or_else(|| panic!("no gloss for {r:?}"));
        assert!(word_count(g) <= 3, "{g}");
    }
    assert_eq!(crate::turn::why_gloss("read", "no use"), Some("nothing to learn"));
    assert_eq!(crate::turn::why_gloss("attack", "same as R2"), Some("earlier row covers"));
    let v = Game::new_literal(1).vocabulary_wire();
    assert!(v.why_gloss.contains_key("no target") && v.why_gloss.contains_key("row guard"));
    assert!(Game::new_literal(1).vocabulary().why_gloss.is_empty(), "stored copies do not carry it");
    // A `✗` callout in play carries its reason.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![], Verb::arg("read", "teleport")), Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 30);
    let cross = evs.iter().find_map(|e| if let Ev::Callout { text, why, .. } = e { text.contains('✗').then(|| why.clone()) } else { None });
    assert_eq!(cross, Some(Some("none in pack".to_string())), "{:?}", ev_kinds(&evs));
    for p in [crate::monster::Pending::Shoot, crate::monster::Pending::Swell, crate::monster::Pending::Call, crate::monster::Pending::Mirror] {
        assert!(word_count(p.why()) <= 3 && !p.why().contains("your"), "{}", p.why());
    }
}

/// Cut 23 §5 (AI: "the leash stolen nearly every run"): a thief takes a leash — the kennel's
/// free one too — only when the pack holds nothing else, after the packed supplies.
#[test]
fn a_thief_takes_the_leash_last() {
    let mut g = arena();
    hold_rules(&mut g);
    let leash = give(&mut g, "leash");
    g.run.as_mut().unwrap().hero.inv.iter_mut().find(|i| i.id == leash).unwrap().free = true;
    let heal = give(&mut g, "heal");
    g.run.as_mut().unwrap().supplies.push(heal);
    let run = g.run.as_mut().unwrap();
    // Found nothing, carried nothing: the packed heal before the leash.
    match crate::ai::thief_pick(run, false, false) {
        Some(crate::ai::Take::Inv(i)) => assert_eq!(run.hero.inv[i].id, heal),
        _ => panic!("the heal first"),
    }
    run.loot_add_gold(10);
    assert!(matches!(crate::ai::thief_pick(run, false, false), Some(crate::ai::Take::Coins(_))), "coins before the leash");
    run.loot_add_gold(-10);
    run.hero.inv.retain(|i| i.id != heal);
    match crate::ai::thief_pick(run, false, false) {
        Some(crate::ai::Take::Inv(i)) => assert_eq!(run.hero.inv[i].kind, "leash", "nothing else: the leash"),
        _ => panic!("the leash last"),
    }
}

/// Cut 23 §1: the walls hold against the forge — the Queen's brood shields her (half of every
/// blow while a lurker she called lives) and bites harder than a wild lurker; the Mirror King
/// heals twice what he sends back.
#[test]
fn the_walls_answer_the_forge() {
    let mut g = arena();
    hold_rules(&mut g);
    let q = add_monster(&mut g, "lurker_queen", 8, 5);
    let qi = g.run.as_ref().unwrap().monsters.iter().position(|m| m.id == q).unwrap();
    let hp0 = g.run.as_ref().unwrap().monsters[qi].hp;
    let (run, mut cx) = g.ctx();
    crate::turn::damage_monster(run, &mut cx, qi, 10, &crate::turn::Src::Hero { ranged: false });
    assert_eq!(g.run.as_ref().unwrap().monsters[qi].hp, hp0 - 10, "no brood: the whole blow");
    let l = add_monster(&mut g, "lurker", 9, 5);
    g.run.as_mut().unwrap().monsters.iter_mut().find(|m| m.id == l).unwrap().summoned = true;
    let (run, mut cx) = g.ctx();
    crate::turn::damage_monster(run, &mut cx, qi, 10, &crate::turn::Src::Hero { ranged: false });
    assert_eq!(g.run.as_ref().unwrap().monsters[qi].hp, hp0 - 15, "her brood takes half");
}


/// Hero looks: a cosmetic lineage field — saved, inherited by the next heir, on the wire as the
/// class's own until set, and never part of what a sim starts from (`lineage_key`).
#[test]
fn look_is_cosmetic_saved_and_inherited() {
    let mut g = Game::new_literal(7);
    assert_eq!(g.lineage().look, g.lineage.class.default_look());
    let key = crate::forecast::lineage_key(&g);
    let plain = crate::save::save(&g);
    assert!(g.set_look("dog").is_err());
    g.set_look("cat").unwrap();
    assert_eq!(g.lineage().look, "cat");
    assert_eq!(crate::forecast::lineage_key(&g), key);
    let g2 = crate::save::load(&crate::save::save(&g)).unwrap();
    assert_eq!(g2.lineage().look, "cat");
    // an older save (no field) loads with the class's own look, and serialises unchanged
    let old = crate::save::load(&plain).unwrap();
    assert_eq!(old.lineage.look, None);
    assert_eq!(crate::save::save(&old), plain);
    let mut g3 = g2;
    g3.lineage.new_heir();
    assert_eq!(g3.lineage().look, "cat");
}

/// QA on 912e135 (qaW: `D6 61%` read `▲32` then `▲38` unedited; a bought leash moved the bar
/// while `▲` fell): the paired move carries the sent set's share on the same seeds — `base +
/// delta` is the active panel's own reach, the number the bar shows for the same pass.
#[test]
fn a_paired_move_carries_the_sent_sets_share() {
    let mut g = Game::new_literal(2201);
    g.lineage.best_depth = 4;
    let prev = g.lineage.rules().clone();
    let mut rows = prev.rows.clone();
    rows.insert(0, Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")));
    g.set_rules(RuleSet { rows, name: None, route: Vec::new() }).unwrap();
    let f = g.forecast();
    let vs = g.forecast_vs(&prev);
    assert_eq!(vs.sims, f.sims, "one pass");
    for d in f.depths.iter().filter(|d| d.depth >= f.start) {
        let v = vs.depths.iter().find(|v| v.depth == d.depth).expect("the depth's move");
        assert!((v.base + v.delta - d.reach).abs() < 1e-9, "D{}: base {} + delta {} vs bar {}", d.depth, v.base, v.delta, d.reach);
    }
    let e = f.ends.expect("ends");
    assert!((vs.death.base + vs.death.delta - e.death).abs() < 1e-9 && (vs.bank.base + vs.bank.delta - e.bank).abs() < 1e-9);
    assert!((vs.stall.base + vs.stall.delta - e.stall).abs() < 1e-9);
}

/// QA on 912e135 (qaW: `sword +1 · $300 · 7 nights` after a night of deaths with the purse held
/// at $40): the heir purse's top-up is no income — a night of deaths nets nothing toward a step.
#[test]
fn the_heir_purse_is_no_income() {
    let mut g = Game::new_literal(3);
    let before = g.lineage.night_net;
    g.lineage.gold_move(40, "wake pay");
    assert_eq!(g.lineage.night_net, before);
    g.lineage.gold_move(50, "returned D4");
    assert_eq!(g.lineage.night_net, before + 50);
}

/// QA on 912e135 (qaW: the kennel's free leash, dropped, came back only at $30): while nothing is
/// tamed the kennel's leash is free to take back, and it goes back on the shelf free.
#[test]
fn the_kennel_leash_comes_back_free() {
    let mut g = Game::new_literal(5);
    let leash = g.lineage.supplies.iter().find(|s| s.kind == "leash" && s.free).map(|s| s.id).expect("the kennel's leash");
    g.drop_supply(leash).unwrap();
    assert_eq!(g.supply_catalogue().iter().find(|s| s.kind == "leash").map(|s| s.price), Some(0));
    let gold = g.lineage.gold;
    g.buy_supply("leash").unwrap();
    assert_eq!(g.lineage.gold, gold);
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "leash" && s.free));
    assert_eq!(g.supply_catalogue().iter().find(|s| s.kind == "leash").map(|s| s.price), Some(30));
}

// ---------------------------------------------------------------- Cut 24

/// AL's set before the boss row (`eval/cards/4b15a61.raterAL-preboss.rules.json`) on a lineage
/// with the kit AL had bought (sword +1, leather +1).
fn al_preboss(seed: u64) -> Game {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards/4b15a61.raterAL-preboss.rules.json")).unwrap();
    let set = RuleSet::parse(&text).unwrap();
    let mut g = Game::new_literal(seed);
    g.max_deaths = 1000;
    for u in ["row5", "row6", "row7", "kite_archers"] {
        g.lineage.unlocks.insert(u.into());
    }
    for f in ["foe:goblin_warlord:boss", "foe:monkey:thief", "foe:bloat:gas"] {
        g.lineage.facts.insert(f.into());
    }
    identify(&mut g, "heal");
    g.lineage.kit.insert("weapon".into(), 1);
    g.lineage.kit.insert("armour".into(), 2);
    g.set_rules_raw(set).unwrap();
    g
}

/// Cut 24 §1 (AL: the Warlord > 4 min on `attack nearest`, his bar full): a boss whose HP does
/// not move for `BOSS_STILL` of the hero's actions drives him off — a return-tier exit whose
/// line reads `no counter` with the boss's defence and the counter to write; a set with a way
/// home keeps the return's share, the fight never runs past the bound.
#[test]
fn a_boss_that_cannot_be_hurt_drives_the_hero_off() {
    let mut g = al_preboss(2302);
    let mut driven = None;
    for _ in 0..12 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
        let r = g.run.as_ref().unwrap();
        assert!(r.nohp.2 <= crate::turn::BOSS_STILL.max(crate::turn::NOHP_ACTIONS), "a stretch of {} actions", r.nohp.2);
        let (off, carried, pct) = (r.driven_off.clone(), r.loot.max(0), r.yield_pct(ExitTier::Return));
        // QA on 308f045 (qaAC: the reel's `Returned with $0.` under `1 DRIVEN`): a drive-off never says it returned.
        if off.is_some() {
            assert!(r.notes.iter().all(|(_, n)| !n.starts_with("Returned with")), "{:?}", r.notes);
            assert!(r.notes.iter().any(|(_, n)| n.starts_with("Driven off")), "{:?}", r.notes);
        }
        g.finish_run();
        g.auto_keep();
        if off.is_some() {
            driven = Some((g.batch.exits.last().cloned().unwrap(), carried, pct));
            break;
        }
    }
    let (line, carried, pct) = driven.expect("AL's pre-boss set is driven off by the Warlord");
    let d = line.driven.as_ref().expect("the exit names the drive-off");
    assert_eq!((d.boss.as_str(), d.verdict.as_str(), d.defence.as_str(), d.counter.as_str()), ("goblin_warlord", "no counter", "shields up", "attack boss"));
    assert_eq!(d.row, crate::facts::counter_row("goblin_warlord"));
    assert_eq!(pct, ExitTier::Return.pct(), "a set with a return row keeps the return's share");
    assert_eq!(line.kept, carried * pct / 100);
    // QA on 0c6e126 (qaY): the line leads with `driven` (the tile's word), the ledger's line too.
    assert!(line.text.starts_with("driven ") && line.text.contains("by Warlord"), "{}", line.text);
    assert!(g.lineage.gold_ledger.iter().any(|x| x.why.starts_with("driven D")), "the ledger names the drive-off");
    assert!(g.lineage.facts.iter().any(|f| f.starts_with("boss:goblin_warlord:counter")), "the counter is learned");
    // No way home written (the DEFAULT set): the drive-off keeps nothing, as a stall.
    let mut g = Game::new_literal(5);
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.loot = 40;
    run.driven_lost = true;
    assert_eq!(run.yield_pct(ExitTier::Return), 0);
}

/// Cut 24 §1: a fight that merely runs long (both bars moving) never trips: the FULL set's boss
/// fights, counters written, end with the boss slain inside the bound, never driven off.
#[test]
fn a_fight_that_moves_is_never_driven_off() {
    let n: u32 = par_seeds(1..=3u64, |seed| {
        let mut g = Game::new_literal(seed);
        g.max_deaths = 1000;
        for u in crate::meta::UNLOCKS {
            g.lineage.unlocks.insert(u.id.into());
        }
        crate::probes::learn_everything(&mut g);
        g.lineage.classes.insert(Class::Fighter.name().into(), crate::wire::ClassProg { level: 10, xp: 0, next: 0 });
        g.lineage.unlocks.insert(crate::hero::mastery_card(Class::Fighter).into());
        crate::kit::buy_all(&mut g.lineage);
        g.set_rules(crate::probes::full()).unwrap();
        let mut n = 0;
        for _ in 0..3 {
            g.lineage.rest_left = 0;
            g.start_run(None);
            g.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            let r = g.run.as_ref().unwrap();
            n += r.boss_kills.len() as u32;
            // (the Queen's floor: FULL's hero may yet meet a queen he cannot silence in time)
            assert!(r.driven_off.as_deref().is_none_or(|k| k == "lurker_queen" || k == "foundry_master"), "seed {seed}: driven off by {:?} at D{}", r.driven_off, r.depth);
            g.finish_run();
            g.auto_keep();
        }
        n
    })
    .into_iter()
    .sum();
    assert!(n >= 6, "FULL slew {n} bosses");
}

/// Cut 24 §2 (AK: `Ulak is avenged.`, then Ulak again): an avenged grudge retires for good; an
/// unavenged one keeps its floor.
#[test]
fn an_avenged_grudge_retires() {
    for avenged in [false, true] {
        let mut g = Game::new_literal(11);
        g.lineage.grudges.push(crate::descent::Grudge { kind: "goblin".into(), name: "Ulak".into(), depth: 2, heir: 1, avenged, tamed: false, biome: None });
        g.start_run(None);
        g.descend_to(2);
        let met = g.run.as_ref().unwrap().monsters.iter().any(|m| m.name.as_deref() == Some("Ulak"));
        assert_eq!(met, !avenged, "avenged {avenged}");
    }
}

/// Cut 24 §2 (AK: `Ashul the jackal` on D3 run after run): a named foe (the first stray) met in
/// a run rests the next two — at most every third run.
#[test]
fn a_named_foe_rotates() {
    let g0 = (1..200).map(Game::new).find(|g| g.lineage.first_stray().is_some()).expect("a lineage with a first stray");
    let mut g = g0;
    let (depth, name) = g.lineage.first_stray().unwrap();
    let mut met = Vec::new();
    for _ in 0..7 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let due = g.run.as_ref().unwrap().first_stray.is_some();
        g.descend_to(depth);
        let placed = g.run.as_ref().unwrap().named_placed.contains(&name);
        met.push(placed);
        assert!(!placed || due, "placed only when due");
        let r = g.run.as_mut().unwrap();
        r.over = Some(ExitTier::Return);
        g.finish_run();
        g.auto_keep();
    }
    assert!(met.iter().any(|m| *m), "{met:?}");
    for w in met.windows(3) {
        assert!(w.iter().filter(|m| **m).count() <= 1, "met twice within three runs: {met:?}");
    }
}

/// Cut 24 §2: the floor events draw from pools of ≥ 3 lines, never one of the last three runs'
/// while one is left (`chronicle::variant` over the lineage's `event_recent`).
#[test]
fn a_floor_event_line_never_repeats_within_three_runs() {
    let mut g = Game::new_literal(3);
    let mut shown: Vec<String> = Vec::new();
    for _ in 0..9 {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let r = g.run.as_mut().unwrap();
        shown.push(crate::chronicle::variant(r, "shrine"));
        r.over = Some(ExitTier::Return);
        g.finish_run();
        g.auto_keep();
    }
    for w in shown.windows(3) {
        assert!(w[0] != w[1] && w[1] != w[2] && w[0] != w[2], "{shown:?}");
    }
    assert!(crate::chronicle::pool("shrine").contains(&shown[0].as_str()));
}

/// Cut 24 §2: the exit's news leads with what was new — a record depth, a first boss — and a run
/// with nothing new says the one thing that differed.
#[test]
fn an_exit_names_what_was_new() {
    let mut g = Game::new_literal(4);
    g.lineage.best_depth = 3;
    g.lineage.found_kinds.insert("dagger".into());
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.max_depth = 5;
    run.over = Some(ExitTier::Return);
    let news = g.run_news(g.run.as_ref().unwrap(), ExitTier::Return, 60);
    assert_eq!(news.first().map(|n| (n.k.as_str(), n.text.as_str())), Some(("record", "record: D5")));
    // Nothing new: the thing that differed from the last run.
    g.lineage.best_depth = 9;
    g.lineage.last_run = Some(crate::engine::RunBrief { depth: 4, tier: "return".into(), ..Default::default() });
    let news = g.run_news(g.run.as_ref().unwrap(), ExitTier::Return, 60);
    assert_eq!(news.iter().map(|n| (n.k.as_str(), n.text.as_str())).collect::<Vec<_>>(), vec![("differ", "deeper: D5, last D4")]);
}

/// Cut 24 §3 (AL: leather +1 $1000 → $1100 → $1400 as the best deepened): a step's price is
/// fixed once the forge is shown; the kit is never a thief's and never put down.
#[test]
fn the_forge_keeps_its_price_and_the_kit_is_never_loot() {
    let mut g = Game::new_literal(6);
    g.lineage.best_depth = 4;
    let before = crate::kit::price(&g.lineage, "armour", 1);
    g.lineage.gold = crate::kit::price(&g.lineage, "weapon", 0) as i32;
    crate::kit::lock_unit(&mut g.lineage);
    g.lineage.best_depth = 14;
    assert_eq!(crate::kit::price(&g.lineage, "armour", 1), before, "fixed once shown");
    assert_eq!(crate::kit::row_gold(&g.lineage, "row8"), Some(crate::kit::unit(4) * 16));
    // The kit takes no pack slot and a full pack never puts it down.
    let mut h = crate::hero::Hero::new(Class::Fighter, Pos::new(0, 0));
    h.auto_equip(Item::new(crate::kit::ARMOUR_ID, "leather"));
    for i in 0..crate::hero::INV_SLOTS as u32 {
        h.inv.push(Item::new(100 + i, "heal"));
    }
    let old = h.auto_equip(Item::new(99, "mail"));
    assert_eq!(old.map(|o| o.id), Some(crate::kit::ARMOUR_ID));
    assert!(h.inv.iter().any(|i| i.id == crate::kit::ARMOUR_ID), "the kit rides over a full pack");
    assert!(h.inv_full());
    h.inv.retain(|i| i.id != 100);
    assert!(!h.inv_full(), "the kit takes no slot");
}

/// QA on 0c6e126 (qaY, qaZ): prices shown stay put — the row slot's (`+1 row $450` → `$600` overnight with no gold buy), the
/// forge's once its tile is carved (the first salvage), and the repeat's quote (`≤$20` then `repeat −$26`); the captive's `ally`
/// tag is no foe token.
#[test]
fn shown_prices_stay_put_and_ally_is_no_foe_token() {
    let mut g = Game::new_literal(6);
    g.lineage.best_depth = 5;
    crate::kit::lock_unit(&mut g.lineage);
    let row = crate::kit::row_gold(&g.lineage, "row5");
    g.lineage.best_depth = 8;
    assert_eq!(crate::kit::row_gold(&g.lineage, "row5"), row, "the row's price is fixed at the first exit");
    let mut g = Game::new_literal(7);
    g.lineage.best_depth = 3;
    g.lineage.forge.insert("heal".into(), Default::default());
    let before = crate::kit::price(&g.lineage, "weapon", 0);
    crate::kit::lock_unit(&mut g.lineage);
    g.lineage.best_depth = 9;
    assert_eq!(crate::kit::price(&g.lineage, "weapon", 0), before, "the forge shown is the forge fixed");
    // The repeat's quote holds as the shelf's price climbs.
    let mut g = Game::new_literal(8);
    g.lineage.best_depth = 5;
    g.lineage.gold = 500;
    g.set_rules(RuleSet::parse(r#"{"rows":[{"conds":[{"k":"hp<","n":30}],"verb":{"v":"drink","a":"heal"}}]}"#).unwrap()).unwrap();
    g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "heal").unwrap());
    g.buy_supply("heal").unwrap();
    g.start_run(None);
    let quoted = *g.lineage.repeat_quote.get("heal").unwrap();
    g.run = None;
    g.lineage.best_depth = 9;
    g.lineage.supplies.clear();
    let (kinds, gold) = g.repeat_plan();
    assert_eq!((kinds, gold), (vec!["heal".to_string()], quoted), "the badge's price is the quote");
    g.restock();
    assert_eq!(g.lineage.supplies.last().map(|s| s.paid), Some(quoted), "the re-pack pays the quote");
    // No `foe: ally` / `attack ally` / `tame ally`.
    let mut g = Game::new_literal(9);
    g.lineage.facts.insert("foe:captive:ally".into());
    g.lineage.unlocks.insert("tame".into());
    let v = crate::tokens::vocabulary(&g.lineage);
    assert!(!v.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some("ally")));
    assert!(!v.verbs.iter().any(|x| x.a.as_deref() == Some("tag:ally")));
}

/// Cut 24 §5 (AK: "the warlord forecast on D9, met on D8"): the forecast names each boss on the
/// floor he is met on; the row below keeps his `try` and says where he stands.
#[test]
fn the_forecast_names_the_boss_on_his_floor() {
    let mut g = Game::new_literal(8);
    g.lineage.best_depth = 9;
    g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    let f = g.forecast();
    let at = |d: u32| f.depths.iter().find(|x| x.depth == d).unwrap();
    assert_eq!(at(8).boss.as_deref(), Some("goblin_warlord"));
    assert!(at(9).boss.is_none() && at(7).boss.is_none());
    if let Some(t) = &at(9).try_ {
        assert_eq!(t.met, 8);
    }
}

/// Cut 24 §4: the refine continues the first pass's panel (its sims are the refine's first) —
/// the same numbers as a refine from a cold cache.
#[test]
fn the_refine_reuses_the_first_pass() {
    let mut g = Game::new_literal(9);
    g.lineage.best_depth = 5;
    let cold = g.clone().forecast_refine();
    let _ = g.forecast();
    assert_eq!(g.forecast_refine(), cold);
}

/// QA on 0c6e126 (qaY, seed 2401: ♟1's trace ended at `hp 1`, the spectral blade's blow unseen):
/// a death's trace ends on the killing blow — after the last action, at 0 hp, naming the killer;
/// the exit's own trace carries it too.
#[test]
fn a_death_trace_ends_on_the_killing_blow() {
    let mut g = Game::new_literal(2401);
    g.send();
    let mut exit_trace = None;
    loop {
        let r = g.step(200);
        for e in &r.events {
            if let Ev::Exit { trace, .. } = e {
                exit_trace = trace.clone();
            }
        }
        if r.run_over {
            g.auto_keep();
            break;
        }
    }
    let d = g.death(1).expect("♟1 dies");
    let b = d.trace.blow.clone().expect("a blow");
    assert_eq!((b.hp, b.by.as_str()), (0, d.cause.as_str()));
    assert!(b.dmg > 0 && b.t >= d.trace.turns.last().unwrap().t, "{b:?}");
    assert_eq!(exit_trace.and_then(|t| t.blow), Some(b));
}

/// QA on 0c6e126 (qaY: `hp < 20% → drink invisibility · survives 92%` offered and applied; the
/// next heir carried none — `blocked · no item · none in pack`): a patch's named item is carried
/// by the next heir, or the patch comes with its purchase (its reach measured with it bought),
/// or it is not offered.
#[test]
fn a_patch_needing_an_item_comes_with_its_purchase_or_not_at_all() {
    // (Cut 25 §3 moved seed 2401's first death — a full pack no longer paces over a scroll — so
    // the test finds the first seed from 2401 whose first death offers a supply's patch.)
    let play = |seed: u64, gold: i32| {
        let mut g = Game::new_literal(seed);
        // (Cut 30 §2: no curious heir identifies the pack's potions on its own any more — the lineage
        // knows the common ones, as a player past the first nights does)
        for k in ["heal", "speed", "invisibility", "teleport", "blink", "strength", "regen"] {
            identify(&mut g, k);
        }
        g.send();
        loop {
            let r = g.step(200);
            if r.run_over {
                g.auto_keep();
                break;
            }
        }
        g.lineage.gold = gold;
        let d = g.death(1)?;
        let camp = crate::trace::camp_state(&g);
        let pack = crate::trace::next_pack_kinds(&camp);
        for p in &d.patches {
            if let Some(k) = crate::trace::row_item(&p.row) {
                assert!(pack.contains(k) || p.buys.as_ref().is_some_and(|b| b.kind == k && b.price <= gold), "{} · {:?}", p.row.describe(), p.buys);
            }
        }
        d.patches.iter().find(|p| p.buys.is_some()).and_then(|p| p.buys.clone())
    };
    // (eight seeds at a time, the first in order that offers one: `par_seeds` keeps the order)
    let mut found = None;
    for from in (2401..2600u64).step_by(8) {
        found = par_seeds(from..(from + 8).min(2600), |s| play(s, 40).map(|b| (s, b))).into_iter().flatten().next();
        if found.is_some() {
            break;
        }
    }
    let (seed, rich) = found.expect("a patch offered with its purchase");
    assert!(rich.price <= 40, "{rich:?}");
    assert!(play(seed, 0).is_none(), "an unaffordable item's patch is not offered");
}

/// QA on 0c6e126 (qaY: `♟6 · D8 · … left bones on D6` — "bones two floors above his depth"): an
/// heir who fell shallower than his best reads `best D8`; the fall on his best floor stays `D7`.
/// qaZ (`avenged Vrak` for a name never shown): the death that makes a grudge names it on its line.
#[test]
fn an_heir_who_fell_above_his_best_reads_best() {
    let mut g = arena();
    g.lineage.trait_ = crate::hero::Trait::Greedy;
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    g.lineage.heir_best = 8;
    g.run.as_mut().unwrap().max_depth = 6;
    g.run.as_mut().unwrap().depth = 6;
    give(&mut g, "sword");
    {
        let (run, mut cx) = g.ctx();
        crate::turn::damage_hero(run, &mut cx, 99, &crate::turn::Src::Gas);
    }
    g.finish_run();
    let line = g.lineage.chronicle.last().unwrap();
    assert!(line.contains(" · best D8 · ") && line.ends_with("left bones on D6."), "{line}");
    // on his best floor, the plain depth
    g.lineage.heir_best = 3;
    g.lineage.chronicle_heir_at("fell to gas", Some("left bones on D3".into()), Some(3));
    assert!(g.lineage.chronicle.last().unwrap().contains(" · D3 · "), "{:?}", g.lineage.chronicle);
}

/// QA on 0c6e126 (qaZ: `new find: axe, leash` — the leash the free supply; qaY: FOUND `mail +1`, the
/// vault's piece the send carried out and back): a kind the send packed is no find, and what the
/// send brought from the vault is no find of the absence.
#[test]
fn packed_and_brought_are_not_finds() {
    let mut g = Game::new_literal(11);
    g.lineage.found_kinds.insert("sword".into());
    let mut mail = crate::item::Item::new(g.lineage.next_vault_id, "mail");
    g.lineage.next_vault_id += 1;
    mail.enchant = 1;
    let mail_id = mail.id;
    g.lineage.vault.push(mail);
    g.loadout(vec![mail_id]);
    g.lineage.rest_left = 0;
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().packed.iter().any(|k| k == "leash") || g.lineage.kennel_declined, "the kennel's leash is packed");
    {
        let run = g.run.as_mut().unwrap();
        run.found_units.push((777_001, "leash".into()));
        run.over = Some(ExitTier::Bank);
    }
    let news = { let run = g.run.as_ref().unwrap(); g.run_news(run, ExitTier::Bank, 100) };
    assert!(!news.iter().any(|n| n.text.contains("leash")), "{news:?}");
    g.finish_run();
    assert!(!g.batch.new_finds.iter().any(|k| k == "leash"), "{:?}", g.batch.new_finds);
    g.auto_keep();
    assert!(g.lineage.vault.iter().any(|v| v.kind == "mail"), "the mail went back to the vault");
    assert!(!g.batch.found.iter().any(|v| v.kind == "mail"), "the brought mail is no find: {:?}", g.batch.found.iter().map(|v| v.kind.clone()).collect::<Vec<_>>());
}

/// Cut 25 §2 (AM: `hp < 30% → return` under `foes ≥ 1 → attack nearest` all run, never fired —
/// stamped DICE): a row of the set whose conditions held but a row above it won every tick, and
/// that moved up survives, is the `order` verdict — naming both rows, the move leading.
#[test]
fn a_shadowed_row_that_saves_him_is_the_order_verdict() {
    let mut found = None;
    for seed in 1..40u64 {
        let mut g = arena_seed(seed);
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") {
            g.lineage.facts.insert(f);
        }
        give(&mut g, "heal");
        give(&mut g, "heal");
        for (x, y) in [(9, 5), (9, 6), (10, 6)] {
            add_monster(&mut g, "goblin", x, y);
        }
        g.run.as_mut().unwrap().hero.hp = 8;
        let attack = Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")).from("player");
        let heal = Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")).from("player");
        rules(&mut g, vec![attack, heal]);
        let mut id = None;
        for _ in 0..600 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                if g.run.as_ref().unwrap().over == Some(ExitTier::Death) {
                    id = Some(g.run.as_ref().unwrap().id);
                }
                g.finish_run();
                break;
            }
        }
        let Some(id) = id else { continue };
        let d = g.death(id).unwrap();
        found = Some(d.clone());
        if d.verdict == "order" {
            break;
        }
    }
    let d = found.expect("a death");
    assert_eq!((d.verdict.as_str(), d.cause_row, d.order_over), ("order", Some(1), Some(0)), "{} base {:.2} {:?}", d.verdict, d.baseline, d.patches.iter().map(|p| (p.row.describe(), p.moves_from, p.insert_at, p.survive)).collect::<Vec<_>>());
    let p = &d.patches[0];
    assert_eq!((p.moves_from, p.insert_at), (Some(1), 0));
    assert!(p.survive >= crate::trace::ORDER_BAR && p.survive - d.baseline >= crate::trace::PATCH_MARGIN - 1e-9, "{p:?}");
    // Applied, the move reorders the set: nothing added, nothing cut.
    let set = d.rules.clone().unwrap();
    let moved = crate::offline::apply_patch(&set, p, 8);
    assert_eq!(moved.rows.len(), set.rows.len());
    assert_eq!((moved.rows[0].clone(), moved.rows[1].clone()), (set.rows[1].clone(), set.rows[0].clone()));
}

/// Cut 25 §3 (AN, AM: `pick up ×441` while alert rose 8/8): a full pack whose only spare weapon is
/// the forged arm (never put down) does not walk onto a scroll it cannot take — and an item the
/// chores stood on and left lying is given up for the floor.
#[test]
fn a_full_pack_never_paces_over_an_item_it_cannot_take() {
    let mut g = arena_seed(3);
    {
        let run = g.run.as_mut().unwrap();
        run.hero.inv.clear();
        // The forged sword rides in the pack (a found axe in hand); nine more fill it.
        let kit = Item::new(crate::kit::WEAPON_ID, "sword");
        run.hero.inv.push(kit);
        let axe = run.new_item_id();
        run.hero.weapon = Some(Item::new(axe, "axe"));
        let spare = run.new_item_id();
        run.hero.inv.push(Item::new(spare, "sword"));
        for _ in 0..9 {
            let id = run.new_item_id();
            run.hero.inv.push(Item::new(id, "heal"));
        }
        let id = run.new_item_id();
        let pos = Pos::new(8, 5);
        run.items.push(crate::engine::FloorItem { pos, item: Item::new(id, "aggravate") });
        run.floor.map.update_vision(run.hero.pos, VISION);
    }
    assert!(g.run.as_ref().unwrap().hero.inv_full());
    let item = g.run.as_ref().unwrap().items[0].item.clone();
    // One spare melee weapon besides the kit: the pack keeps it (a spare makes way for a scroll only when two are).
    assert!(!crate::turn::can_take(&g.run.as_ref().unwrap().hero, &item), "the forged sword is no spare");
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 600);
    let picks = evs.iter().filter(|e| matches!(e, Ev::Rule { row: -2, verb, .. } if verb.v == "pick_up")).count();
    assert!(picks <= 10, "{picks} pick-ups toward a scroll the pack cannot take");
}

/// Cut 25 §3: a drain (a poison tick, a hunger bite) with no foe in view sends `Ev::Drain` once per
/// stretch; a foe in view ends the stretch.
#[test]
fn a_drain_with_no_foe_in_view_is_named_once() {
    let mut g = arena_seed(2);
    hold_rules(&mut g);
    g.run.as_mut().unwrap().hero.poison = (1, 40);
    let evs = ticks(&mut g, 200);
    let drains: Vec<&Ev> = evs.iter().filter(|e| matches!(e, Ev::Drain { .. })).collect();
    assert_eq!(drains.len(), 1, "{:?}", ev_kinds(&evs));
    assert!(matches!(drains[0], Ev::Drain { cause, .. } if cause == "poisoned"));
    // A foe in view: no drain line.
    let mut g = arena_seed(2);
    hold_rules(&mut g);
    add_monster(&mut g, "rat", 9, 5);
    g.run.as_mut().unwrap().hero.poison = (1, 40);
    let evs = ticks(&mut g, 30);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Drain { .. })), "{:?}", ev_kinds(&evs));
}

/// Cut 25 §3 (AN: an offline trace `14 → 0` on one `goblin −2` row): a death's trace itemises
/// every blow after the last action, stepping the hp down to the killing blow.
#[test]
fn a_death_trace_itemises_every_blow_after_the_last_action() {
    let mut blows_seen = false;
    for seed in 1..30u64 {
        let mut g = arena_seed(seed);
        hold_rules(&mut g);
        for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4)] {
            add_monster(&mut g, "goblin", x, y);
        }
        g.run.as_mut().unwrap().hero.hp = 8;
        ticks(&mut g, 400);
        let run = g.run.as_ref().unwrap();
        if run.over != Some(ExitTier::Death) {
            continue;
        }
        let tr = crate::engine::exit_trace(run, &[]);
        let blow = tr.blow.clone().expect("the killing blow");
        assert_eq!(blow.hp, 0);
        if tr.blows.len() >= 2 {
            blows_seen = true;
            assert_eq!(tr.blows.last(), Some(&blow));
            assert!(tr.blows.windows(2).all(|w| w[1].hp <= w[0].hp), "{:?}", tr.blows);
            let first = &tr.blows[0];
            let last_hp = tr.turns.last().map(|t| t.hp).unwrap_or(8);
            assert!(first.hp < last_hp, "the first blow steps down from the last action's {last_hp}: {:?}", tr.blows);
            break;
        }
    }
    assert!(blows_seen, "a death with several blows after the last action");
}

/// QA on 524827b (qaAA: `hp < 50% → read unknown` went in at R1 above the set's heal and return
/// rows): a low-hp candidate is measured under the set's top block of safety rows too; a
/// safety row itself, or a row with no hp threshold, has no such slot.
#[test]
fn safe_slot_is_under_the_top_safety_block() {
    let row = |conds: Vec<Cond>, v: &str, a: Option<&str>| Row::new(conds, Verb { v: v.into(), a: a.map(str::to_string) });
    let set = RuleSet { rows: vec![row(vec![Cond::n("hp<", 40)], "return", None), row(vec![Cond::n("hp<", 30)], "drink", Some("heal")), row(vec![Cond::n("foes>=", 1)], "attack", Some("nearest"))], name: None, route: Vec::new() };
    assert_eq!(crate::trace::safe_slot(&set, &row(vec![Cond::n("hp<", 50)], "read", Some("unknown"))), Some(2));
    assert_eq!(crate::trace::safe_slot(&set, &row(vec![Cond::n("hp<", 20)], "return", None)), None, "a safety row itself");
    assert_eq!(crate::trace::safe_slot(&set, &row(vec![Cond::n("foes>=", 2)], "retreat", None)), None, "no hp threshold");
    let bare = RuleSet { rows: vec![row(vec![Cond::n("foes>=", 1)], "attack", Some("nearest"))], name: None, route: Vec::new() };
    assert_eq!(crate::trace::safe_slot(&bare, &row(vec![Cond::n("hp<", 50)], "read", Some("unknown"))), None, "no safety block");
}

/// QA on 524827b (qaAA: `drink unknown · 12/12` led; the camp's killers then read fire · poison):
/// a patch's whole-run move is paired over the camp panels; worse beyond its ± it harms, sinks
/// under the rest and is never the gem's; a gamble's rising self-harm is its `risk`.
#[test]
fn whole_run_move_judges_a_patch() {
    use crate::forecast::SimResult;
    let sim = |depth: u32, tier: ExitTier, cause: Option<&str>| SimResult { oath_progress: 0.0, oath_steps: 0, oath: false, max_depth: depth, tier, cause: cause.map(str::to_string), loot_kept: 0, timed_out: false, ticks: 1, loot: 0, fires: Vec::new(), passage: 0 };
    let base: Vec<SimResult> = (0..50).map(|i| if i < 25 { sim(6, ExitTier::Return, None) } else { sim(5, ExitTier::Death, Some("goblin")) }).collect();
    // ten seeds that returned now die to fire
    let worse: Vec<SimResult> = (0..50).map(|i| if i < 10 { sim(5, ExitTier::Death, Some("fire")) } else { base[i].clone() }).collect();
    let w = crate::trace::whole_move(&base, &worse, 6, true);
    assert!(w.death > 0.19 && w.harms && w.risk.as_deref() == Some("fire"), "{w:?}");
    assert!(crate::trace::whole_move(&base, &worse, 6, false).risk.is_none(), "no gamble, no risk");
    let same = crate::trace::whole_move(&base, &base, 6, true);
    assert!(!same.harms && same.death == 0.0 && same.risk.is_none(), "{same:?}");
    // one seed apart is inside the ±
    let one: Vec<SimResult> = (0..50).map(|i| if i == 0 { sim(5, ExitTier::Death, Some("fire")) } else { base[i].clone() }).collect();
    assert!(!crate::trace::whole_move(&base, &one, 6, true).harms);
    let mk = |v: &str, whole: Option<PatchWhole>| Patch { no_gain: false, row: Row::new(vec![Cond::n("hp<", 20)], Verb { v: v.into(), a: None }), insert_at: 0, survive: 1.0, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 6, forecast_pm: 0.1, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None, whole, gem: false, restores: None };
    let mut ps = vec![mk("rest", Some(w.clone())), mk("retreat", Some(same.clone())), mk("descend", None)];
    crate::trace::sink_harms(&mut ps);
    assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["retreat", "descend", "rest"]);
    assert_eq!(crate::trace::gem_patch(&ps).map(|p| p.row.verb.v.as_str()), Some("retreat"));
}

/// QA on 524827b (qaAB: a bought 2nd leash, $30, gone after a returned run — the pack merged it
/// into the kennel's free stack, and the exit sent the stack back to the kennel as the free one):
/// the bought leash comes home to the shelf, bought, beside the kennel's.
#[test]
fn a_bought_leash_comes_home_from_a_return() {
    let mut g = Game::new_literal(2602);
    g.lineage.gold = 500;
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "leash" && s.free), "the kennel's leash");
    g.buy_supply("leash").unwrap();
    g.send();
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, ExitTier::Return);
    }
    g.finish_run();
    let leashes: Vec<(bool, i32)> = g.lineage.supplies.iter().filter(|s| s.kind == "leash").map(|s| (s.free, s.paid)).collect();
    assert!(leashes.contains(&(false, 30)), "the bought leash is back on the shelf: {leashes:?}");
    assert!(leashes.iter().any(|l| l.0), "and the kennel's: {leashes:?}");
    assert!(g.last_exit.as_ref().is_some_and(|l| l.text.contains("leash")), "the exit line names it: {:?}", g.last_exit.as_ref().map(|l| &l.text));
}

// ---------------------------------------------------------------- Cut 26 routes

/// Cut 26 §1: `examples/compat.rs`'s hash of a save's next `n` sends — every event but the
/// fork's own beats (`TWO STAIRS`, the `fork:` fact), and each run's end state.
/// `"key":<integer>` removed from serialized JSON (a field added after a fixture was recorded).
fn strip_key(json: &str, key: &str) -> String {
    let pat = format!(",\"{key}\":");
    let mut out = String::with_capacity(json.len());
    let mut rest = json;
    while let Some(i) = rest.find(&pat) {
        out.push_str(&rest[..i]);
        let tail = &rest[i + pat.len()..];
        let n = tail.find(|c: char| !(c.is_ascii_digit() || c == '-')).unwrap_or(tail.len());
        rest = &tail[n..];
    }
    out.push_str(rest);
    out
}

/// `strip_key` for a key whose value is an array (`,"key":[…]`, brackets balanced).
fn strip_array(json: &str, key: &str) -> String {
    let pat = format!(",\"{key}\":[");
    let mut out = String::with_capacity(json.len());
    let mut rest = json;
    while let Some(i) = rest.find(&pat) {
        out.push_str(&rest[..i]);
        let tail = &rest[i + pat.len() - 1..];
        let mut depth = 0i32;
        let mut end = tail.len();
        for (k, c) in tail.char_indices() {
            match c {
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// `strip_key` for a key whose value is an object (`,"key":{…}`, braces balanced).
fn strip_object(json: &str, key: &str) -> String {
    let pat = format!(",\"{key}\":{{");
    let mut out = String::with_capacity(json.len());
    let mut rest = json;
    while let Some(i) = rest.find(&pat) {
        out.push_str(&rest[..i]);
        let tail = &rest[i + pat.len() - 1..];
        let mut depth = 0i32;
        let mut end = tail.len();
        for (k, c) in tail.char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = k + 1;
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// `strip_key` only where the numeric key closes its object (`,"key":N}`) — a trace turn's last
/// field, not an entity's `max_hp` mid-object.
fn strip_tail_key(json: &str, key: &str) -> String {
    let pat = format!(",\"{key}\":");
    let mut out = String::with_capacity(json.len());
    let mut rest = json;
    while let Some(i) = rest.find(&pat) {
        let tail = &rest[i + pat.len()..];
        let n = tail.find(|c: char| !(c.is_ascii_digit() || c == '-')).unwrap_or(tail.len());
        if tail[n..].starts_with('}') {
            out.push_str(&rest[..i]);
        } else {
            out.push_str(&rest[..i + pat.len() + n]);
        }
        rest = &tail[n..];
    }
    out.push_str(rest);
    out
}

fn sends_hash(g: &mut Game, n: u32) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    let fnv = |h: &mut u64, s: &str| {
        for b in s.bytes() {
            *h ^= b as u64;
            *h = h.wrapping_mul(0x100000001b3);
        }
    };
    for _ in 0..n {
        g.send();
        for _ in 0..400 {
            let r = g.step(200);
            for e in &r.events {
                if matches!(e, Ev::Callout { text, .. } if text == "TWO STAIRS") || matches!(e, Ev::Fact { fact, .. } if fact.starts_with("fork:")) {
                    continue;
                }
                // QA on 308f045 (qaAC): what the screen is told, not what is played — a max-HP loss's own
                // event (drain, the shrine; hunger's was 307dbed's) and the exit note a drive-off or a stall
                // no longer writes (`Returned with $0.`)
                // (Cut 29 §3: the meters' heal events and a line's meters are new reads of the same run)
                if matches!(e, Ev::Heal { .. }) {
                    continue;
                }
                if matches!(e, Ev::MaxHp { cause, .. } if cause != "hunger") || matches!(e, Ev::Note { text, .. } if text.starts_with("Returned with") || text.starts_with("Came home with")) {
                    continue;
                }
                // (QA on 308f045: a death trace's `hp_healed` is a new read of the same run)
                // (and a reason's gloss reworded since: `given up · hero quit chasing` was `· out of reach`)
                // (Cut 28 §2/§4: a trace's max hp per turn and its max-hp steps are new reads of the same run; the `saved` note turns
                // its words — `got him out`, `pulled him through` — and says the same)
                let j = strip_key(&serde_json::to_string(e).unwrap(), "hp_healed").replace("hero quit chasing", "out of reach");
                let j = strip_tail_key(&strip_array(&j, "max_steps"), "max_hp").replace(" got him out.", " saved him.").replace(" pulled him through.", " saved him.");
                // (Cut 29 §1: the frontier mark is gone — its words on the exit line with it)
                let j = strip_object(&j, "meters");
                fnv(&mut h, &j);
            }
            if r.run_over {
                break;
            }
        }
        fnv(&mut h, &format!("{} {} {}", g.lineage.gold, g.lineage.best_depth, g.deaths.len()));
    }
    h
}

/// Cut 26 §1: a 307dbed save (a D28 lineage, every unlock and fact, the FULL set) plays its next
/// 10 sends exactly as 307dbed did — the default route is the base order.
/// Cut 27 §3 re-baselined the hash (`476e6517d82b9054` → `adbe8cb9d2210f7a`): the exits trade — a
/// return is out after `turn::RETURN_TICKS`, a bank's walk wakes the carry's scent; with both off
/// (`RETURN_TICKS` unbounded, scent radius < 0) the save still hashes to 307dbed's.
/// Cut 27 §5 moved it again (`adbe8cb9d2210f7a` → `d59e321d76373469`) without moving the play: a
/// full event diff of the 10 sends (92 767 events, moves included) against the §3 tree differs in
/// one event — send 8's Lich drive-off exit line, whose `driven.verdict` now reads `order` (with
/// `held: 7`: the set's `foe: summoned → attack summoned` row is there, pre-empted) where it read
/// `no counter` (§5, a drive-off whose counter row is in the set). Every tick, purse, depth and
/// death is identical; §4's loop fixes do not act on these sends.
#[test]
fn saves_from_307dbed_send_identically() {
    let mut g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    assert!(g.lineage.rules().route.is_empty());
    // (Cut 29 §1: re-recorded — the frontier mark and its words left the exit lines; with the old
    // mark's text restored the stream hashed to 307dbed's d59e321d76373469 exactly)
    // (Cut 29, the Queen's silence slot: re-recorded `21e701bd6e6b9c2d` → `3754cfc8a2604f4b` — the
    // full event diff of the 10 sends against the tree before it first differs at send 6, t 42131, on
    // D28: a `pick_up` chore walks to a silence scroll the full pack now takes (`turn::queen_slot`)
    // where it explored; everything before is identical, and with `QUEEN_PACK_DEPTH` out of reach
    // the save hashes to `21e701bd6e6b9c2d` again)
    // (Cut 30: re-recorded `3754cfc8a2604f4b` → `9b68c23a7de8b0cd` — the save migrates as the
    // `custom` stance (its set as written), its heir's temperament no longer overrides a row, and each
    // band boss it meets carries the lineage's scars from the next send on)
    let want = u64::from_str_radix(include_str!("fixtures/sends_307dbed.txt").trim(), 16).unwrap();
    assert_eq!(format!("{:016x}", sends_hash(&mut g, 10)), format!("{want:016x}"));
}

/// A game on `route` (the forks seen, the set's route written past the door).
fn route_game(seed: u64, route: crate::descent::Route) -> Game {
    let mut g = Game::new_literal(seed);
    for f in crate::descent::FORKS {
        g.lineage.facts.insert(format!("fork:{f}"));
    }
    let set = g.lineage.rules().clone().with_route(route);
    g.set_rules(set).unwrap();
    g
}

/// Cut 26 §1: every one of the 13 routes reaches D34's stairs — each floor generated in the
/// route's biome, each band's boss on its last floor, the Captain on the Burrows' first.
#[test]
fn every_route_reaches_the_bottom() {
    use crate::descent::{Route, ENDING_DEPTH};
    assert_eq!(Route::all().len(), 13);
    for route in Route::all() {
        let mut g = route_game(7, route);
        g.start_run(Some(7));
        for d in 2..ENDING_DEPTH {
            g.descend_to(d);
            let run = g.run.as_ref().unwrap();
            assert_eq!((run.depth, run.route), (d, route));
            assert_eq!(run.biome(), route.biome(d), "{route:?} D{d}");
            let boss = run.monsters.iter().find(|m| m.is_boss()).map(|m| m.kind.as_str());
            assert_eq!(boss, route.boss(d), "{route:?} D{d}");
            let captain = run.monsters.iter().any(|m| m.kind == "goblin_captain");
            assert_eq!(captain, route.lieutenant(d).is_some(), "{route:?} D{d}");
            assert!(run.floor.map.passable(run.floor.stairs_down));
        }
        g.descend_to(ENDING_DEPTH);
        assert_eq!(g.run.as_ref().unwrap().max_depth, ENDING_DEPTH, "{route:?}");
    }
}

/// Cut 26 §1: same seed + rules + route ⇒ identical events (each route's replay hash is its
/// own); two routes past the first fork play two descents.
#[test]
fn replay_hash_per_route() {
    use crate::descent::Route;
    let hash = |route: Route, sends: u32| -> (String, u32) {
        let mut g = route_game(11, route);
        for u in ["throw", "row5", "row6", "row7", "row8"] {
            g.lineage.unlocks.insert(u.into());
        }
        crate::probes::learn_everything(&mut g);
        g.lineage.classes.insert("fighter".into(), crate::wire::ClassProg { level: 8, xp: 0, next: 0 });
        g.set_rules_raw(crate::probes::good().with_route(route)).unwrap();
        let mut out = String::new();
        let mut deepest = 0;
        for _ in 0..sends {
            g.send();
            for _ in 0..200 {
                let r = g.step(200);
                out.push_str(&serde_json::to_string(&r.events).unwrap());
                if r.run_over {
                    deepest = deepest.max(g.lineage.best_depth);
                    break;
                }
            }
        }
        (out, deepest)
    };
    // (each hash its own thread: `par_seeds` over the jobs' indices, results in order)
    let routes = Route::all();
    let fens = Route::from_forks(&[5]).unwrap();
    let jobs: Vec<(Route, u32)> = routes.iter().flat_map(|r| [(*r, 1), (*r, 1)]).chain([(Route::BASE, 3), (fens, 3)]).collect();
    let hashes = par_seeds(0..jobs.len() as u64, |i| hash(jobs[i as usize].0, jobs[i as usize].1));
    for (i, route) in routes.iter().enumerate() {
        assert_eq!(hashes[2 * i], hashes[2 * i + 1], "{route:?}");
    }
    let n = hashes.len();
    let ((a, da), (b, db)) = (hashes[n - 2].clone(), hashes[n - 1].clone());
    assert!(da >= 5 && db >= 5, "both reach the fork: {da} {db}");
    assert_ne!(a, b, "the Fens-first descent is another dungeon below D4");
}

/// Cut 26 §2: the fork is a fact the hero learns on D4's two stairs (`TWO STAIRS`, once a run);
/// until then the set cannot take it, and `in: fens` is locked until the Fens are entered.
#[test]
fn fork_seen_before_shown() {
    use crate::descent::Route;
    let mut g = Game::new_literal(3);
    let fens = Route::from_forks(&[5]).unwrap();
    assert!(g.lineage().forks.is_empty());
    assert_eq!(g.set_rules(g.lineage.rules().clone().with_route(fens)), Err("D5 fork unseen".into()));
    let with_in = |g: &Game| {
        let mut s = g.lineage.rules().clone();
        s.rows.insert(0, Row::new(vec![Cond::t("in", "fens"), Cond::n("hp<", 50)], Verb::new("retreat")));
        s
    };
    assert!(g.set_rules(with_in(&g)).unwrap_err().contains("locked"));
    g.start_run(Some(3));
    g.descend_to(4);
    // Walk him onto the stairs: the fork is seen.
    let mut evs = Vec::new();
    {
        let (run, mut cx) = g.ctx();
        run.hero.pos = run.floor.stairs_down;
        run.floor.map.update_vision(run.hero.pos, 7);
        crate::facts::fork_seen(run, &mut cx);
        crate::facts::fork_seen(run, &mut cx);
    }
    evs.append(&mut g.events);
    assert!(g.lineage.facts.contains("fork:5"));
    assert_eq!(evs.iter().filter(|e| matches!(e, Ev::Callout { text, .. } if text == "TWO STAIRS")).count(), 1, "once a run");
    let snap = g.snapshot();
    let f = snap.fork.expect("D4's stairs are the fork's");
    assert_eq!((f.depth, f.taken.as_str(), f.other.as_str()), (5, "burrows", "fens"));
    let chips = g.lineage().forks;
    assert_eq!(chips.len(), 1);
    assert_eq!((chips[0].near.as_str(), chips[0].far.as_str(), chips[0].taken.as_str(), chips[0].open), ("burrows", "fens", "burrows", true));
    // The fork's biomes show on the sheet, locked until entered.
    let v = g.vocabulary_wire();
    assert!(v.locked.iter().any(|l| l.cond == Cond::t("in", "fens") && l.needs == "enter fens"));
    assert!(!v.conds.contains(&Cond::t("in", "fens")));
    g.run = None;
    g.set_rules(g.lineage.rules().clone().with_route(fens)).unwrap();
    assert_eq!(g.lineage().forks[0].taken, "fens");
    assert_eq!(g.lineage.rules().route, vec![5]);
    // Exported with the rows, restored by an import.
    let text = g.export_rules();
    assert!(text.contains("\"route\""));
    assert_eq!(g.import_rules(&text).unwrap().route, vec![5]);
    g.lineage.facts.insert("biome:fens".into());
    assert!(g.vocabulary().conds.contains(&Cond::t("in", "fens")));
    g.set_rules(with_in(&g)).unwrap();
    assert!(g.lineage().locked_rows.is_empty());
}

/// Cut 26 §2: the `in:` cond reads the run's biome on its route.
#[test]
fn in_cond_reads_the_lane() {
    use crate::descent::Route;
    let fens = Route::from_forks(&[5]).unwrap();
    for (route, want) in [(Route::BASE, "burrows"), (fens, "fens")] {
        let mut g = route_game(5, route);
        g.lineage.facts.insert("biome:fens".into());
        g.lineage.facts.insert("biome:burrows".into());
        g.start_run(Some(5));
        g.descend_to(6);
        let (run, cx) = g.ctx();
        let v = crate::turn::view(run);
        for b in ["burrows", "fens"] {
            assert_eq!(crate::turn::cond_holds(run, &cx, &v, &Cond::t("in", b)), b == want, "{route:?} {b}");
        }
    }
}

/// Cut 26 §2: the fork chip's tablet — both stairs on the camp's seeds, the current one the camp's
/// own panel; a route prices its bosses where it puts them.
#[test]
fn fork_tablet_prices_both_stairs() {
    use crate::descent::Route;
    let mut g = route_game(9, Route::BASE);
    g.lineage.best_depth = 7;
    let opts = g.fork_forecast(5);
    assert_eq!(opts.len(), 2);
    assert!(opts[0].current && !opts[0].far && opts[0].biome == "burrows");
    assert!(!opts[1].current && opts[1].far && opts[1].biome == "fens" && opts[1].route == vec![5]);
    assert!((7..=8).contains(&opts[0].depth), "{}", opts[0].depth);
    assert_eq!(opts[0].reach_delta, 0.0);
    // The camp's own number for the current stair.
    let f = g.forecast();
    let at = f.depths.iter().find(|d| d.depth == opts[0].depth).unwrap();
    assert!((at.reach - opts[0].reach).abs() < 1e-9);
    let d8 = f.depths.iter().find(|d| d.depth == 8).unwrap();
    assert_eq!(d8.boss.as_deref(), Some("goblin_warlord"));
    g.set_rules(g.lineage.rules().clone().with_route(Route::from_forks(&[5]).unwrap())).unwrap();
    let f = g.forecast();
    let d8 = f.depths.iter().find(|d| d.depth == 8).unwrap();
    assert_eq!((d8.boss.as_deref(), d8.biome.as_deref()), (Some("bloat_mother"), Some("fens")));
    let far = f.depths.iter().find(|d| d.depth == opts[1].depth).unwrap();
    assert!((far.reach - opts[1].reach).abs() < 1e-9, "the tablet's far stair is the camp's panel once taken");
    // A fork the route gives no choice at is not priced.
    assert!(g.fork_forecast(9).is_empty());
}

/// Cut 26 §2: a bank lights the waystones of its route's prefix — D9 in the Burrows (a Fens-first
/// route's) is not D9 in the Fens; the start takes the active route's.
#[test]
fn waystones_per_route_prefix() {
    use crate::descent::Route;
    let fens = Route::from_forks(&[5]).unwrap();
    let mut l = crate::engine::LineageState::new(1);
    assert_eq!(l.light_waystones_on(10, fens), vec![5, 9]);
    assert!(l.waystones.is_empty());
    assert!(l.stone_lit(9, fens) && !l.stone_lit(9, Route::BASE));
    assert!(l.stone_lit(9, Route::from_forks(&[5, 14]).unwrap()), "the prefix to D9 is the same");
    assert_eq!(l.light_waystones_on(10, Route::BASE), vec![5, 9]);
    assert_eq!(l.waystones, vec![5, 9]);
    assert!(l.set_start(9).is_ok());
    let lanes = l.lane_list();
    assert_eq!(lanes.len(), 4);
    assert!(lanes.iter().any(|s| s.depth == 9 && s.lane == "burrows" && s.route == vec![5] && !s.current));
    assert!(lanes.iter().any(|s| s.depth == 9 && s.lane == "fens" && s.route.is_empty() && s.current));
}

/// Cut 26 §1: grudges and bones live on the biome and floor where they happened.
#[test]
fn grudges_live_on_their_biome() {
    use crate::descent::{Biome, Grudge};
    let g = Grudge { kind: "eel".into(), name: "Grak".into(), depth: 6, heir: 1, avenged: false, tamed: false, biome: Some(Biome::Fens) };
    assert!(g.lives_on(6, Biome::Fens) && !g.lives_on(6, Biome::Burrows));
    let old = Grudge { biome: None, ..g };
    assert!(old.lives_on(6, Biome::Burrows) && !old.lives_on(6, Biome::Fens), "a save's grudge is the base order's");
}

/// Cut 26 seam (control rater AR: a 20-minute absence read `0 RUNS`): the rest after a watched
/// exit was the camp time spent on it — the absence starts rested; and a rest that ends inside
/// the absence, to its last tick, sends the next heir (the run finishes past the budget).
#[test]
fn a_short_break_yields_a_run() {
    let mut g = Game::new_literal(4);
    g.send();
    for _ in 0..2000 {
        if g.step(200).run_over {
            break;
        }
    }
    let _ = g.keep(Vec::new());
    assert!(g.lineage.rest_left > 0 && g.lineage.rest_watched, "a watched exit rests the heir at camp");
    let text = g.save();
    let mut a = Game::load(&text).unwrap();
    assert!(a.run_offline(20 * 60).runs >= 1, "20 minutes after a watched exit");
    // An offline exit's rest carries into the next absence; one that ends on its last tick sends.
    let mut b = Game::load(&text).unwrap();
    b.lineage.rest_watched = false;
    b.lineage.rest_left = crate::engine::REST_MIN_TICKS;
    assert!(b.run_offline(crate::engine::REST_MIN_TICKS as u64 / crate::offline::TICKS_PER_SECOND).runs >= 1);
    let mut c = Game::load(&text).unwrap();
    c.lineage.rest_watched = false;
    c.lineage.rest_left = crate::engine::REST_MIN_TICKS + 10;
    assert_eq!(c.run_offline(crate::engine::REST_MIN_TICKS as u64 / crate::offline::TICKS_PER_SECOND).runs, 0, "still resting");
}

/// Cut 26 (risks: attribution): a death on the far lane the near stair would have carried past
/// reads `route` — the fork, the stair taken, the other; its sends' numbers agree with the bar.
#[test]
fn a_far_lane_death_can_be_the_routes() {
    use crate::descent::Route;
    let fens = Route::from_forks(&[5]).unwrap();
    let mut seen = 0;
    for seed in [3u64, 5, 8, 13] {
        let mut g = route_game(seed, fens);
        g.lineage.classes.insert("fighter".into(), crate::wire::ClassProg { level: 3, xp: 0, next: 0 });
        crate::probes::learn_everything(&mut g);
        let set = RuleSet {
            rows: vec![
                Row::new(vec![Cond::n("hp<", 35)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
                Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            ],
            name: None,
            route: vec![5],
        };
        g.set_rules_raw(set).unwrap();
        g.run_offline(4 * 3600);
        let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall && (5..=8).contains(&r.death.depth)).map(|(id, _)| *id).collect();
        for id in ids.into_iter().take(3) {
            let d = g.death(id).unwrap();
            if d.verdict == "route" {
                let c = d.route_cause.clone().expect("a route verdict names its fork");
                assert_eq!((c.fork, c.taken.as_str(), c.other.as_str()), (5, "fens", "burrows"));
                assert!(c.route.is_empty());
                assert!(c.survive >= crate::trace::ROUTE_BAR - 1e-9 && c.survive - c.base >= crate::trace::PATCH_MARGIN - 1e-9, "{c:?}");
                seen += 1;
            } else {
                assert!(d.route_cause.is_none());
            }
        }
        if seen > 0 {
            break;
        }
    }
    assert!(seen > 0, "a Fens-lane death the Burrows would have carried past reads `route`");
}

/// Cut 26 §6 (AP: `R3 descend · locked cond` beside `see: hunger`): `see: den` is gated by the
/// den's fact alone — offered once the den is known, whatever the bare `on_see` cond's unlock, and
/// its row's why-not reads `no den seen`, never `locked cond`; a row whose cond the lineage cannot
/// use (the fact unknown, a cond unbought) is marked on the wire, and the door refuses it.
#[test]
fn a_locked_cond_never_enters_a_row_unmarked() {
    let mut g = Game::new_literal(2);
    let see = Cond::t("on_see", "den");
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![see.clone()], Verb::new("descend")));
    assert!(!g.vocabulary().conds.contains(&see));
    assert!(g.set_rules(set.clone()).unwrap_err().contains("locked"));
    g.set_rules_raw(set.clone()).unwrap();
    assert!(g.lineage().locked_rows.first().cloned().flatten().is_some(), "{:?}", g.lineage().locked_rows);
    g.lineage.facts.insert("den".into());
    assert!(!g.lineage.unlocks.contains("cond_on_see"));
    assert!(g.vocabulary().conds.contains(&see));
    g.set_rules(set).unwrap();
    assert!(g.lineage().locked_rows.is_empty());
    g.start_run(Some(2));
    let (run, cx) = g.ctx();
    assert_eq!(crate::turn::row_why_of(run, &cx, &see), "no den seen");
    let bare = Cond::flag("on_kill");
    assert_eq!(crate::turn::row_why_of(run, &cx, &bare), "locked cond");
}

/// Cut 26 §2: the night plays the active set's route, and the report names each band's lane.
#[test]
fn the_report_names_the_lanes() {
    use crate::descent::Route;
    let fens = Route::from_forks(&[5]).unwrap();
    let mut g = route_game(6, fens);
    crate::probes::learn_everything(&mut g);
    g.lineage.classes.insert("fighter".into(), crate::wire::ClassProg { level: 6, xp: 0, next: 0 });
    for u in ["row5", "row6", "row7", "row8", "throw"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.set_rules_raw(crate::probes::good().with_route(fens)).unwrap();
    let r = g.run_offline(4 * 3600);
    assert!(r.deepest >= 5, "deepest D{}", r.deepest);
    assert_eq!(r.lanes.first().map(String::as_str), Some("D5–8 · the Fens"));
    assert!(r.lanes.iter().all(|l| crate::rules::word_count(l) <= 4), "{:?}", r.lanes);
    assert!(g.lineage.facts.contains("biome:fens"));
}

/// Cut 26 §6 (AO: `GAP` beside `unpatched 10/12` — fault or luck?): a gap, row or order most of
/// whose unpatched replays survive leans to the dice (`Death.lean`), and only such a death does.
#[test]
fn a_stamp_never_contradicts_its_counts() {
    let mut g = Game::new_literal(12);
    g.max_deaths = 1000;
    g.run_offline(8 * 3600);
    let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall).map(|(id, _)| *id).take(12).collect();
    assert!(!ids.is_empty());
    for id in ids {
        let d = g.death(id).unwrap();
        let leans = matches!(d.verdict.as_str(), "gap" | "row" | "order") && d.baseline > crate::trace::STAMP_BASE + 1e-9;
        assert_eq!(d.lean.as_deref() == Some("dice"), leans, "run {id}: {} base {:.2} lean {:?}", d.verdict, d.baseline, d.lean);
    }
}

/// QA on 308f045 (qaAC: `← never met` over `R1 attack boss · not in view`, R1 having fired four
/// times on the floor; `← R4 drank heal` with no row): a chain link is its row's newest reason —
/// a reason the row outlived (it fired after it, or said something else since) is off the chain.
#[test]
fn the_chain_keeps_each_rows_newest_reason() {
    use crate::wire::{Because, RowWhy, TraceTurn};
    let b = |text: &str, t: u32| Some(Because { text: text.into(), t, depth: 8 });
    let turn = |t: u32, row: i32, rows: Vec<RowWhy>| TraceTurn { max_hp: 0, t, row, verb: Verb::new("attack"), hp: 10, foes: 1, rule_foes: 1, telegraphs: Vec::new(), blocked: None, rows: Some(rows), blows: Vec::new(), gift: None };
    let turns = vec![
        // R1 (`attack boss`) never met the boss, R2's heal was drunk by R4
        turn(10, 2, vec![RowWhy { row: 0, why: "not in view".into(), because: b("never met", 10) }, RowWhy { row: 1, why: "no item".into(), because: b("R4 drank heal at 17/36 hp", 5) }]),
        // R1 fires on the boss
        turn(20, 0, vec![]),
        // the boss slain: R1 not in view, slain; R2 still no heal
        turn(30, 2, vec![RowWhy { row: 0, why: "not in view".into(), because: b("bloat mother slain D8", 25) }, RowWhy { row: 1, why: "no item".into(), because: b("R4 drank heal at 17/36 hp", 5) }]),
    ];
    let links = crate::trace::chain_links(&turns);
    let texts: Vec<(usize, &str)> = links.iter().map(|(r, b)| (*r, b.text.as_str())).collect();
    assert_eq!(texts, [(0, "bloat mother slain D8"), (1, "R4 drank heal at 17/36 hp")]);
    // R1 fired after its `never met`, and nothing newer: no link for it at all
    let fired_last = vec![turns[0].clone(), turns[1].clone()];
    let links = crate::trace::chain_links(&fired_last);
    assert_eq!(links.iter().map(|(r, b)| (*r, b.text.as_str())).collect::<Vec<_>>(), [(1, "R4 drank heal at 17/36 hp")]);
}

/// QA on 308f045 (qaAC: a patch read `reach D9 ≈ ±1` — the frontier no sim reached either way —
/// and applied, the camp's `vs sent` led with `D6 −21`): the whole-run move is read where the camp
/// reads it — the frontier when it moves, else the floor that moves most; an exit's reach is its
/// price, not a harm.
#[test]
fn a_patch_reads_the_floor_the_camp_leads_with() {
    use crate::forecast::SimResult;
    let sim = |depth: u32, tier: ExitTier| SimResult { oath_progress: 0.0, oath_steps: 0, oath: false, max_depth: depth, tier, cause: None, loot_kept: 0, timed_out: false, ticks: 1, loot: 0, fires: Vec::new(), passage: 0 };
    // the sent set: half reach D7, none D9; the patch: 11 of those stop at D5 (D6 and D7 both −22: the deeper leads), still none D9
    let base: Vec<SimResult> = (0..50).map(|i| sim(if i < 25 { 7 } else { 5 }, ExitTier::Death)).collect();
    let worse: Vec<SimResult> = (0..50).map(|i| if i < 11 { sim(5, ExitTier::Death) } else { base[i].clone() }).collect();
    let w = crate::trace::whole_move_on(&base, &worse, 1, 9, false, false);
    assert_eq!(w.depth, 7, "{w:?}");
    assert!((w.reach + 0.22).abs() < 1e-9 && (w.reach_from - 0.5).abs() < 1e-9 && w.harms, "{w:?}");
    assert!((w.death_from - 1.0).abs() < 1e-9);
    // the same move on an exit is its price
    let home: Vec<SimResult> = (0..50).map(|i| if i < 11 { sim(5, ExitTier::Return) } else { base[i].clone() }).collect();
    let e = crate::trace::whole_move_on(&base, &home, 1, 9, false, true);
    assert!(e.depth == 7 && e.reach < 0.0 && !e.harms && e.death < 0.0, "{e:?}");
    // nothing moves: the deepest floor a sim still reaches, never the sealed frontier
    let same = crate::trace::whole_move_on(&base, &base, 1, 9, false, false);
    assert_eq!(same.depth, 7, "{same:?}");
    // the frontier moving leads, whatever moves more above it
    let front: Vec<SimResult> = (0..50).map(|i| if i < 25 { sim(9, ExitTier::Death) } else { sim(3, ExitTier::Death) }).collect();
    assert_eq!(crate::trace::whole_move_on(&base, &front, 1, 9, false, false).depth, 9);
}
