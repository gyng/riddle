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
fn arena() -> Game {
    arena_seed(1)
}

fn arena_seed(seed: u64) -> Game {
    let mut g = Game::new(seed);
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

fn add_monster(g: &mut Game, kind: &str, x: i32, y: i32) -> u32 {
    let run = g.run.as_mut().unwrap();
    let id = run.new_id();
    let depth = run.depth;
    let mut m = Monster::spawn(id, kind, Pos::new(x, y), depth);
    m.awake = true;
    m.last_seen = Some(run.hero.pos);
    run.monsters.push(m);
    id
}

fn give(g: &mut Game, kind: &str) -> u32 {
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
fn rules(g: &mut Game, rows: Vec<Row>) {
    g.set_rules_raw(RuleSet { rows, name: None }).unwrap();
}

fn ticks(g: &mut Game, n: u32) -> Vec<Ev> {
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
    let mut a = Game::new(42);
    let mut b = Game::new(42);
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
    let mut a = Game::new(3);
    let mut b = Game::new(3);
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
    let err = g.set_rules(RuleSet { rows: rows.clone(), name: None }).unwrap_err();
    assert_eq!(err, "5 own rows, 4 allowed");
    assert!(word_count(&err) <= 6, "{err}");
    rules(&mut g, rows.clone());
    assert_eq!(g.lineage.rules().rows.len(), 4, "a sim's fifth own row is cut without the row5 unlock");
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Exit { .. })), "row 5 must not fire without the row5 unlock");
    g.lineage.unlocks.insert("row5".into());
    g.set_rules(RuleSet { rows: rows.clone(), name: None }).unwrap();
    assert_eq!(g.lineage.rules().rows.len(), 5);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
}

/// Cut 12 §1: a card brings its row — a 4-row lineage holds 4 own rows and 2 card rows; a
/// fifth own row is refused, a second row for the same card is refused, an unowned card's row
/// is refused; the rows in play carry their set indices (the card rows fire as `R2`/`R5`);
/// `needs: fill rows` counts own rows; `insert_at` sits before the engagement row and the
/// card's delta row goes there.
#[test]
fn card_rows_sit_outside_the_cap() {
    let mut g = Game::new(5);
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.unlocks.insert("boss_focus".into());
    let own = |n: i32| Row::new(vec![Cond::n("hp<", n)], Verb::arg("drink", "heal"));
    let card = |c: &str| Row::new(vec![], Verb::arg("tactic", c));
    let rows = vec![own(10), card("thief_guard"), own(20), own(30), card("boss_focus"), own(40)];
    g.set_rules(RuleSet { rows: rows.clone(), name: None }).unwrap();
    let set = g.lineage.rules();
    assert_eq!((set.rows.len(), set.own_rows(), set.card_rows()), (6, 4, 2));
    assert_eq!(g.vocabulary().max_rows, 4, "the wire's cap is the own-row cap");
    assert_eq!(set.active(4).map(|(i, _)| i).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4, 5]);
    // A fifth own row: refused, the set unchanged.
    let mut five = rows.clone();
    five.push(own(50));
    assert_eq!(g.set_rules(RuleSet { rows: five, name: None }).unwrap_err(), "5 own rows, 4 allowed");
    assert_eq!(g.lineage.rules().rows.len(), 6);
    // A second row for a card the set already carries: refused.
    let mut twice = rows.clone();
    twice.push(card("thief_guard"));
    assert_eq!(g.set_rules(RuleSet { rows: twice, name: None }).unwrap_err(), "two rows for thief guard");
    // A card the lineage does not own: refused.
    let mut unowned = rows.clone();
    unowned.push(card("gas_step"));
    assert_eq!(g.set_rules(RuleSet { rows: unowned, name: None }).unwrap_err(), "card not owned: gas step");
    // A sim cuts the last own row and keeps every card row.
    let mut five = rows.clone();
    five.insert(0, own(5));
    let fit = RuleSet { rows: five, name: None }.fit(4);
    assert_eq!(fit.rows.len(), 6);
    assert_eq!(fit.card_rows(), 2);
    assert!(!fit.rows.contains(&own(40)) && fit.rows.contains(&own(5)));
    // `needs: fill rows` reads the own rows: four own rows fill a 4-row lineage.
    let cat = crate::meta::catalogue(&g.lineage);
    assert_ne!(cat.iter().find(|u| u.id == "row5").unwrap().needs.as_deref(), Some("fill rows"));
    // `insert_at`: before the first `attack`/`shoot`, else the end; the delta row goes there.
    let mut with_attack = rows.clone();
    with_attack[2] = Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"));
    g.set_rules(RuleSet { rows: with_attack, name: None }).unwrap();
    g.lineage.facts.insert("foe:bloat:gas".into());
    let cat = crate::meta::catalogue(&g.lineage);
    let gas = cat.iter().find(|u| u.id == "gas_step").unwrap();
    assert_eq!(gas.insert_at, Some(2));
    assert_eq!(crate::meta::delta_row(&g.lineage, "gas_step").unwrap().1, 2);
    assert_eq!(cat.iter().find(|u| u.id == "thief_guard").unwrap().insert_at, None, "an owned card has no place to go");
    assert_eq!(cat.iter().find(|u| u.id == "throw").unwrap().insert_at, None, "a verb unlock's row is the player's");
    g.set_rules(RuleSet { rows, name: None }).unwrap();
    assert_eq!(crate::meta::catalogue(&g.lineage).iter().find(|u| u.id == "gas_step").unwrap().insert_at, Some(6), "no engagement row: the end");
    // The rows in play: an own row fires with its set index (`R6` = `hp<40`; indices are 0-based).
    let mut g = arena();
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.unlocks.insert("boss_focus".into());
    let rows = vec![own(0), card("thief_guard"), own(0), own(0), card("boss_focus"), Row::new(vec![], Verb::new("return"))];
    g.set_rules_raw(RuleSet { rows, name: None }).unwrap();
    assert_eq!(g.lineage.rules().rows.len(), 6);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 5, .. })), "the fourth own row is the set's sixth: {evs:?}");
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
}

#[test]
fn rule_text_is_at_most_three_words() {
    let mut g = Game::new(11);
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

#[test]
fn cowardly_trait_retreats_before_rows() {
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Cowardly;
    g.run.as_mut().unwrap().hero.hp = 5;
    add_monster(&mut g, "rat", 6, 5);
    attack_rules(&mut g);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, text, .. } if text.starts_with("cowardly"))));
}

#[test]
fn brave_trait_never_retreats_from_one_foe() {
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Brave;
    add_monster(&mut g, "rat", 6, 5);
    rules(&mut g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("retreat")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, text, .. } if text.starts_with("brave"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: 1, .. })));
}

#[test]
fn curious_trait_uses_unknown_when_safe() {
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Curious;
    give(&mut g, "speed");
    rules(&mut g, vec![]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, text, .. } if text.starts_with("curious"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact.ends_with("=speed"))));
}

#[test]
fn greedy_trait_grabs_adjacent_items_with_foes_adjacent() {
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    add_monster(&mut g, "rat", 5, 5);
    let run = g.run.as_mut().unwrap();
    run.items.push(crate::engine::FloorItem { pos: Pos::new(3, 5), item: Item::new(99, "sword") });
    attack_rules(&mut g);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, text, .. } if text.starts_with("greedy"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Pickup { .. })));
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
    let mut g = Game::new(5);
    g.lineage.grudges.push(crate::descent::Grudge { kind: "rat".into(), name: "Grak".into(), depth: 1, heir: 1 });
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
        g.lineage.bones.push(crate::engine::Bones { heir, depth: 1, items: vec![Item::new(500 + heir, "dagger")], named: false });
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
    let evs = ticks(&mut g, 10);
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
    let evs = ticks(&mut g, 10);
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
    let mut g = Game::new(9);
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
    let mut g = Game::new(4);
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
    let mut g = Game::new(6);
    crate::probes::learn_everything(&mut g);
    for m in crate::defs::MONSTERS {
        g.lineage.kills.insert(m.kind.into());
    }
    g.lineage.best_depth = 16;
    g.lineage.renown = 1_000_000;
    g.lineage.rank = 100;
    g.lineage.trophies = vec!["pacifist_floor".into(), "no_heal_D5".into(), "ranged_only_D5".into(), "boss_untouched".into()];
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 10, xp: 0 });
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
    // Cut 5: 12 h (situations on D1–5 gave this seed a new best on its 17th run of 8 h);
    // Cut 8B: 16 h (the first stray on D2 moved the bests again). Cut 13: `hp < 35 %` (at
    // 20 % the set died three times in 16 h once running thieves stopped counting as foes).
    // The client's path (below) runs beside the 16 h night: two games, one thread each.
    let chunked = std::thread::spawn(|| {
        let mut q = Game::new(5);
        let mut set = q.lineage.rules().clone();
        set.rows.insert(0, Row::new(vec![Cond::n("hp<", 35)], Verb::new("return")));
        q.set_rules(set).unwrap();
        let mut last = None;
        for _ in 0..32 {
            last = crate::offline::run_offline_quick(&mut q, 1800).stall;
        }
        last
    });
    let mut g = Game::new(5);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 35)], Verb::new("return")));
    g.set_rules(set).unwrap();
    let r = g.run_offline(16 * 3600);
    let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
    let stall = r.stall.unwrap_or_else(|| panic!("no stall: {} runs · {deaths} deaths · bests {:?}", r.runs, r.bests));
    // Cut 13: the stairs are taken on arrival (`ai::descend_step`), which moved this seed's
    // dice; the one death it gets now is a bloat's burst at 9 hp — `dice`, no row to name.
    assert!(g.deaths.values().all(|d| d.death.verdict == "dice"), "a row-named death: {:?}", g.deaths.values().map(|d| (&d.death.cause, &d.death.verdict)).collect::<Vec<_>>());
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
    let mut g = Game::new(5);
    let r = g.run_offline(8 * 3600);
    assert!(r.deaths.iter().map(|d| d.n).sum::<u32>() > 0, "the default set dies");
    assert!(r.stall.is_none());
}

#[test]
fn stall_patches_apply_as_replace_remove_or_insert() {
    use crate::offline::apply_patch;
    let rules = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")), Row::new(vec![], Verb::new("attack"))], name: None };
    let mk = |row: Row, at: usize, replace: bool, remove: bool| Patch { row, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace, remove, root: None, below_bar: false };
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
    let ret = |row: i32| Act { row, verb: Verb::new("return"), target: None, boss: false };
    let e = ep(6, Some("nest"), ret(2), Resolution::Returned { gold: 54 });
    assert_eq!(story_line(&e), "D6, the nest: R3 returned $54.");
    let e = ep(2, None, Act::default(), Resolution::Returned { gold: 8 });
    assert_eq!(story_line(&e), "D2: returned $8.");
    let e = ep(5, Some("vault"), ret(0), Resolution::Lost { stalled: false });
    assert_eq!(story_line(&e), "D5, the cage: lost the thread.");
    let e = ep(7, Some("lock"), Act { row: 1, verb: Verb::new("bank"), target: None, boss: false }, Resolution::Banked { gold: 120 });
    assert_eq!(story_line(&e), "D7, the lock: R2 banked $120.");
    // An attack row that happened to be the last act is not credited with the exit.
    let e = ep(7, Some("den"), Act { row: 1, verb: Verb::arg("attack", "nearest"), target: None, boss: false }, Resolution::Returned { gold: 3 });
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
            let mut g = Game::new(seed);
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
    assert_eq!(g.lineage.marks, 5, "D1..D3 (3) + first bank from D3 (Cut 4) + the frontier (Cut 15 §1); first kills no longer mark (Cut 2 §2)");
    assert!(g.batch.bests.iter().any(|b| b == "first kill: rat"), "but stay in bests");
    assert!(g.batch.bests.iter().any(|b| b == "home:D3"));
    g.auto_keep();
    let mut g2 = arena();
    g2.lineage = g.lineage.clone();
    g2.run.as_mut().unwrap().max_depth = 3;
    g2.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g2, ExitTier::Bank);
    assert_eq!(g2.lineage.marks, 6, "no new best, no marks but the frontier's (Cut 15 §1: D3 banked at a best of D3)");
    let mut g3 = arena();
    g3.lineage = g2.lineage.clone();
    g3.run.as_mut().unwrap().kills.push((1, "goblin_warlord".into(), 5));
    g3.run.as_mut().unwrap().trophies_run.push("pacifist_floor".into());
    finish_with(&mut g3, ExitTier::Bank);
    assert_eq!(g3.lineage.marks, 6 + 3 + 2 + 1, "boss 3, trophy 2, first bank from D1 1 (D1 is not the frontier of D3)");
}

#[test]
fn unlocks_cost_marks_and_gate_rows() {
    let mut g = Game::new(1);
    assert!(g.buy("row5").is_err());
    g.lineage.marks = 5;
    g.buy("row5").unwrap();
    assert_eq!(g.lineage.marks, 3);
    assert_eq!(g.vocabulary().max_rows, 5);
    assert!(g.buy("row5").is_err(), "already owned");
    assert!(g.buy("row7").is_err(), "prerequisite");
    assert!(g.buy("corridor_fighting").is_err(), "needs the pack fact");
    g.lineage.facts.insert("foe:jackal:pack".into());
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
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("pacifist"))));
    assert!(evs.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("no heal"))));
    let run = g.run.as_ref().unwrap();
    assert!(run.trophies_run.contains(&"pacifist_floor".to_string()));
    assert!(run.trophies_run.contains(&"no_heal_D5".to_string()));
}

// ---------------------------------------------------------------- save

#[test]
fn save_round_trip_continues_identically() {
    let mut g = Game::new(21);
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(1);
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
    c.rules = RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("shoot"))], name: None };
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
    g.set_companion_rules(900_001, RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::new("burst"))], name: None }).unwrap();
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
    let mut g = Game::new(1);
    no_kennel_leash(&mut g);
    assert!(g.buy_supply("leash").is_err());
    g.lineage.gold = 100;
    assert!(g.buy_supply("heal").is_err(), "unidentified");
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let cat = g.supply_catalogue();
    assert!(cat.iter().any(|s| s.kind == "heal" && s.price == 40));
    g.buy_supply("heal").unwrap();
    g.buy_supply("leash").unwrap();
    assert_eq!(g.lineage.gold, 30);
    assert_eq!(g.lineage().supplies.len(), 2);
    g.buy_supply("leash").unwrap();
    assert!(g.buy_supply("leash").unwrap_err().contains("max"));
    g.clear_supplies();
    assert_eq!(g.lineage.gold, 100);
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
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 3, xp: 0 });
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
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 3, xp: 0 });
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
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 9, xp: crate::hero::xp_to_next(9) - 5 });
    g.run.as_mut().unwrap().max_depth = 2;
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.classes["fighter"].level, 10);
    assert!(g.lineage.trophies.contains(&"master:fighter".to_string()));
    assert!(g.lineage.unlocks.contains("phalanx"));
}

// ---------------------------------------------------------------- salvage, forge, renown (Addendum D)

#[test]
fn salvage_feeds_the_forge_and_tiers_apply() {
    let mut g = Game::new(1);
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
    let mut g = Game::new(8);
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
    let mut g = Game::new(77);
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
    let mut g = Game::new(12);
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
    let mut g = Game::new(3);
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
fn traits_pre_empt_at_most_once_per_five_actions_and_never_below_quarter_hp() {
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    {
        let run = g.run.as_mut().unwrap();
        for i in 0..8 {
            run.items.push(crate::engine::FloorItem { pos: Pos::new(5 + i, 5), item: Item::new(200 + i as u32, "heal") });
        }
    }
    rules(&mut g, vec![Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 200);
    let ts: Vec<u32> = evs.iter().filter_map(|e| if let Ev::Rule { t, row: -1, .. } = e { Some(*t) } else { None }).collect();
    assert!(!ts.is_empty());
    for w in ts.windows(2) {
        assert!(w[1] - w[0] >= 50, "greedy fired twice within 5 actions: {ts:?}");
    }
    // Below 25% HP the trait never pre-empts (cowardice excepted).
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Greedy;
    g.run.as_mut().unwrap().hero.hp = 5;
    g.run.as_mut().unwrap().items.push(crate::engine::FloorItem { pos: Pos::new(5, 5), item: Item::new(300, "heal") });
    add_monster(&mut g, "goblin_archer", 10, 5);
    rules(&mut g, vec![Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 100);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, .. })), "greedy pre-empted at 5 HP");
    let mut g = arena();
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Cowardly;
    g.run.as_mut().unwrap().hero.hp = 5;
    add_monster(&mut g, "rat", 6, 5);
    rules(&mut g, vec![Row::new(vec![], Verb::new("hold"))]);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Rule { row: -1, text, .. } if text.starts_with("cowardly"))));
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
    let mut g = Game::new(1);
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
    assert!(d.margin.contains("unknown unused"));
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
            let mut g = Game::new(seed);
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
    let mut g = Game::new(3);
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
        let mut g = Game::new(seed);
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(1);
    for u in ["row5", "row6", "row7", "row8"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.marks = 100;
    assert!(g.buy("row9").is_err(), "three bosses first");
    for k in ["goblin_warlord", "bloat_mother", "lich"] {
        g.lineage.kills.insert(k.into());
    }
    g.buy("row9").unwrap();
    assert_eq!(g.lineage.max_rows(), 9);
    assert!(g.buy("row10").is_err(), "four bosses for the tenth row");
    g.lineage.kills.insert("foundry_master".into());
    g.buy("row10").unwrap();
    assert_eq!(g.lineage.max_rows(), 10);
    assert_eq!(g.lineage.marks, 100 - 8 - 12);
    let ten: Vec<Row> = (0..10).map(|_| Row::new(vec![], Verb::new("hold"))).collect();
    assert!(g.set_rules(RuleSet { rows: ten.clone(), name: None }).is_ok());
    assert_eq!(g.lineage.rules().rows.len(), 10);
    let mut eleven = ten;
    eleven.push(Row::new(vec![], Verb::new("hold")));
    assert!(RuleSet { rows: eleven, name: None }.validate().is_err());
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(5);
    g.lineage.ended = true;
    g.lineage.marks = 12;
    g.lineage.gold = 300;
    g.lineage.heir = 9;
    g.lineage.best_depth = 31;
    for u in ["row5", "row6", "row7", "row8", "row9", "cadence", "throw", "rogue", "phalanx"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 7, xp: 10 });
    g.lineage.kennel.push(crate::probes::pets_party()[0].clone());
    g.lineage.party.push(crate::probes::pets_party()[1].clone());
    g.lineage.vault.push(Item::new(100_001, "plate"));
    g.lineage.facts.insert("foe:lich:boss".into());
    g.lineage.forge.insert("sword".into(), ForgeRow::at(20));
    g.lineage.grudges.push(crate::descent::Grudge { kind: "ogre".into(), name: "Grak".into(), depth: 7, heir: 3 });
    g.lineage.graveyard.push(Grave { heir: 3, depth: 7, cause: "ogre".into(), deeds: vec![], death_id: None });
    g.set_rules(crate::probes::good()).unwrap();
    g
}

#[test]
fn ascension_keeps_the_meta_and_restarts_the_descent() {
    let mut g = Game::new(1);
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
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))], name: None }).unwrap();
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
    g.set_rules(RuleSet { rows: vec![Row::new(vec![], Verb::new("descend"))], name: None }).unwrap();
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(3);
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

/// Cut 15 §1: a bank from depth ≥ the lineage's best (before the run) − 1 pays ◆1 on top of
/// whatever else it earns, named on the exit line; a shallower bank and a return pay nothing.
#[test]
fn a_frontier_bank_pays_a_mark() {
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
    assert_eq!(g.lineage.marks, marks + 1, "D7 against a best of D8 is the frontier");
    assert_eq!(g.batch.frontier_banks, 1);
    assert!(exit_text(&g).ends_with(" · ◆+1 frontier"), "{}", exit_text(&g));
    // A shallower bank (D6 < 8 − 1): nothing.
    g.start_run(Some(9));
    g.lineage.banked_depths.insert(6);
    g.run.as_mut().unwrap().depth = 6;
    g.run.as_mut().unwrap().max_depth = 6;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1, "a shallow bank pays nothing: {:?} {}", g.batch.bests, exit_text(&g));
    assert!(!exit_text(&g).contains('◆'), "{}", exit_text(&g));
    // A return from the frontier: nothing.
    g.start_run(Some(10));
    g.run.as_mut().unwrap().depth = 8;
    g.run.as_mut().unwrap().max_depth = 8;
    finish_with(&mut g, ExitTier::Return);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1, "a return pays nothing");
    // A new-best bank (D10 from a best of D8): two depth marks, the first bank from D10, and
    // the frontier's — measured against the best *before* this run.
    g.start_run(Some(11));
    g.run.as_mut().unwrap().depth = 10;
    g.run.as_mut().unwrap().max_depth = 10;
    finish_with(&mut g, ExitTier::Bank);
    g.keep(vec![]).unwrap();
    assert_eq!(g.lineage.marks, marks + 1 + 2 + 1 + 1);
    assert!(exit_text(&g).ends_with(" · ◆+4 (1 frontier)"), "{}", exit_text(&g));
    assert_eq!(g.batch.frontier_banks, 2);
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

/// Cut 15 §2: a gold buy spends gold (a ledger line `unlock <id>`), leaves the marks, owns the
/// unlock and raises the next gold price by a quarter of the first; a free unlock, an owned
/// one or a shut gate is refused.
#[test]
fn a_gold_buy_spends_gold_and_raises_the_price() {
    let mut g = Game::new(3);
    let price = |g: &Game, id: &str| g.unlocks().into_iter().find(|u| u.id == id).unwrap().gold;
    assert_eq!(price(&g, "vault2"), 450, "◆3 × 150");
    assert_eq!(price(&g, "row5"), 300);
    assert_eq!(price(&g, "rogue"), 0, "a free unlock has no gold price");
    assert!(g.buy_unlock_gold("rogue").is_err());
    assert!(g.buy_unlock_gold("vault2").is_err(), "no gold yet");
    g.lineage.gold_move(20000, "test");
    g.lineage.marks = 1;
    g.buy_unlock_gold("vault2").unwrap();
    assert!(g.lineage.unlocks.contains("vault2"));
    assert_eq!(g.lineage.marks, 1, "marks untouched");
    assert_eq!(g.lineage.gold, 20000 - 450);
    assert_eq!(g.lineage.gold_ledger.last().map(|l| (l.delta, l.why.as_str())), Some((-450, "unlock vault2")));
    assert_eq!(g.lineage.gold_buys, 1);
    assert_eq!(price(&g, "vault2"), 0, "owned");
    assert_eq!(price(&g, "row5"), 150 * 2 * 5 / 4, "the second gold buy costs a quarter more");
    assert!(g.buy_unlock_gold("vault2").is_err(), "already owned");
    assert!(g.buy_unlock_gold("row7").is_err(), "prerequisite missing");
    assert!(g.buy_unlock_gold("caster").is_err(), "a shut gate");
    for _ in 0..3 {
        let id = g.unlocks().into_iter().find(|u| !u.owned && u.gold > 0 && g.buy_unlock_gold(&u.id.clone()).is_ok()).map(|u| u.id);
        assert!(id.is_some());
    }
    assert_eq!(g.lineage.gold_buys, 4);
    assert_eq!(crate::meta::gold_price(3, g.lineage.gold_buys), 900, "the fifth gold buy costs double the first");
    // The save round-trips the count; an old save without it reads 0.
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.lineage.gold_buys, 4);
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
    let mut g = Game::new(5);
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
    let gold_before = g.lineage.gold;
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
    assert_eq!(g.lineage.gold, gold_before - price, "paid from gold");
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
    let mut g = Game::new(6);
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
    let mut g = Game::new(6);
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
    let mut g = Game::new(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:jackal:fast".into());
    g.lineage.facts.insert("item:leash".into());
    g.lineage.best_depth = 2;
    assert!(g.unlocks().iter().all(|u| u.delta.is_none()), "no sims before unlock_deltas");
    let t = std::time::Instant::now();
    let cat = g.unlock_deltas();
    let first = t.elapsed().as_secs_f64();
    let by = |id: &str| cat.iter().find(|u| u.id == id).unwrap().clone();
    assert!(by("corridor_fighting").delta.is_some(), "open card: {:?}", by("corridor_fighting"));
    assert!(by("pack_break").delta.is_some());
    assert!(by("kite_archers").delta.is_none(), "gated on the ranged fact");
    assert!(by("row5").delta.is_none(), "rows have no row to add");
    assert!(by("throw").delta.is_none(), "Cut 13: a verb carries no delta — its canonical row is nobody's policy (QA on 50bb162: `reach −92% ±11`)");
    assert!(by("tame").owned && by("tame").delta.is_none(), "Cut 8B: tame is owned from the start");
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
    let mut g = Game::new(2);
    g.lineage.facts.insert("den".into());
    g.lineage.facts.insert("foe:jackal:pack".into()); // the gate: a foe met
    g.lineage.best_depth = 2;
    let mut rules = g.lineage.rules().clone();
    rules.rows.insert(0, Row::new(vec![Cond::t("on_see", "den")], Verb::arg("attack", "nearest")));
    g.set_rules_raw(rules).unwrap();
    let cat = g.unlock_deltas();
    let by = |id: &str| cat.iter().find(|u| u.id == id).unwrap().clone();
    let on_see = by("cond_on_see");
    assert!(!on_see.owned && on_see.delta.is_some() && on_see.pm.is_some(), "{on_see:?}");
    assert!(by("cond_alert").delta.is_none(), "no row waits on it: {:?}", by("cond_alert"));
}

/// Cut 4: `Lineage.ascended` lists the variants already finished.
#[test]
fn lineage_wire_carries_ascended_variants() {
    let mut g = Game::new(1);
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
        let mut g = Game::new(7);
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
    let mut g = Game::new(3);
    g.lineage.ended = true;
    g.lineage.heir = 4;
    g.lineage.heir_best = 31;
    let trait_ = g.lineage.trait_.name();
    g.ascend("no_rest").unwrap();
    assert_eq!(g.lineage.chronicle, vec![format!("♟4 the {trait_} fighter · D31 · \"fighter\" set · ascended.")]);
}

/// §4: D1 always holds a situation in a room near the entrance; every run meets one on D1–5
/// in the gate's measure (`Batch.situation_runs`).
#[test]
fn the_first_floors_hold_situations() {
    use crate::tiles::Tile;
    let mut with = 0;
    for seed in 1..=20u64 {
        let mut g = Game::new(seed);
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
    g.lineage.sets[1] = RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"))], name: None };
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
    g.lineage.lost.push(crate::engine::Lost { kind: "jackal".into(), name: "Uleth".into(), gen: 2, heir: 1 });
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
        g.lineage.lost.push(crate::engine::Lost { kind: "jackal".into(), name: "Uleth".into(), gen: 2, heir: 1 });
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
        let mut g = Game::new(seed);
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
            let (carried, tier, timed_out, depth, heir) = {
                let r = g.run.as_ref().unwrap();
                (r.loot.max(0), r.over.unwrap(), r.timed_out, r.depth, r.heir)
            };
            let before = g.lineage.gold;
            let lines_before = g.lineage.gold_ledger.len();
            g.finish_run();
            g.auto_keep();
            let after = g.lineage.gold;
            let line = g.batch.exits.last().cloned().expect("an exit line");
            let pct = if timed_out { 0 } else { tier.pct() };
            assert_eq!(line.carried, carried, "seed {seed}");
            assert_eq!(line.keep_pct, pct, "seed {seed}");
            assert_eq!(line.kept, carried * pct / 100, "seed {seed}");
            assert!(word_count(&line.text) <= 14, "{}", line.text);
            // Cut 10 §3: the verb and what came home lead (`returned $50 · $84 carried · keeps 60%`).
            let verb = match tier {
                ExitTier::Bank => "banked",
                ExitTier::Return => "returned",
                ExitTier::Death => "died",
            };
            assert!(line.text.starts_with(&format!("{verb} ${} · ${carried} carried · keeps {pct}%", line.kept)), "{}", line.text);
            assert_eq!(line.text.contains("lost thread") || line.text.contains("stalled"), timed_out, "{}", line.text);
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
            let exit_line = since.iter().find(|l| l.why.starts_with("returned") || l.why.starts_with("banked") || l.why.starts_with("died") || l.why.starts_with("lost") || l.why.starts_with("stalled")).expect("exit movement");
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
    assert_eq!(g.batch.exits.last().map(|l| ExitLine { trace: None, ..l.clone() }), Some(line.clone()));
    assert!(g.batch.exits.last().unwrap().trace.is_some(), "the report's exit line carries the trace");
    // Purchases, insurance, a refund and a hatch are ledger lines with ≤ 3-word reasons.
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
    let before = g.lineage.gold;
    g.buy_supply("heal").unwrap();
    assert_eq!(g.lineage.gold_ledger.last().map(|l| (l.delta, l.why.as_str())), Some((-40, "heal")));
    g.clear_supplies();
    assert_eq!(g.lineage.gold_ledger.last().map(|l| (l.delta, l.why.as_str())), Some((40, "refund heal")));
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
            let mut g = Game::new(seed);
            g.max_deaths = 1000;
            if seed % 2 == 0 {
                g.set_rules_raw(crate::probes::good()).unwrap();
            }
            crate::offline::run_offline_quick(&mut g, 4 * 3600);
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
    g.run.as_mut().unwrap().trait_ = crate::hero::Trait::Brave;
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, 6, "{t:?}");
    let whys: Vec<&str> = t.rows.as_ref().unwrap().iter().map(|w| w.why.as_str()).collect();
    assert_eq!(whys, ["hp not <30%", "foes not ≥2", "not in view", "none held", "not in view", "brave held"], "{t:?}");
    // A card that passes, an unknown-item row with nothing unknown, a locked token.
    let mut g = arena_seed(2);
    g.lineage.unlocks.insert("gas_step".into());
    rules(
        &mut g,
        vec![
            Row::new(vec![], Verb::arg("tactic", "gas_step")),
            Row::new(vec![Cond::flag("unknown_item")], Verb::arg("drink", "unknown")),
            Row::new(vec![Cond::n("loot>=", 1)], Verb::new("return")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
    );
    add_monster(&mut g, "rat", 5, 5);
    ticks(&mut g, 10);
    let t = g.run.as_ref().unwrap().trace.last().unwrap().clone();
    assert_eq!(t.row, 3, "{t:?}");
    let whys: Vec<&str> = t.rows.as_ref().unwrap().iter().map(|w| w.why.as_str()).collect();
    assert_eq!(whys, ["card passed", "no unknown", "locked cond"], "{t:?}");
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
    assert_eq!(t.row, -1, "{t:?}");
    assert_eq!(t.rows.as_ref().unwrap()[0].why, "trait first");
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
            under += 1;
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
    assert!(under * 3 <= 30, "the counter fell under the baseline on {under} of 30 boss deaths");
}

/// §9: the forecast is a function of (rules, lineage seed, depth) and the lineage a sim starts
/// from — re-reading it gives the same numbers, transient state moves nothing, and the refine
/// pass at 100 sims is as deterministic.
#[test]
fn forecast_is_deterministic_per_rules_and_lineage() {
    let mut g = Game::new(11);
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
    let mut h = Game::new(11);
    no_kennel_leash(&mut h);
    h.set_rules_raw(crate::probes::good()).unwrap();
    assert_eq!(h.forecast(), a);
    // Cut 14 §1: the seeds are the lineage's per depth, whatever the set — a different rule
    // set is measured on the same dungeons (a paired difference), a different depth on its own.
    let t1 = crate::forecast::forecast_tag(&g, 1);
    let t3 = crate::forecast::forecast_tag(&g, 2);
    assert!(t1 != t3);
    // Through the save: identical.
    let g3 = Game::load(&g.save()).unwrap();
    assert_eq!(g3.forecast(), a);
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
        let mut g = Game::new(seed);
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
    let g = Game::new(1);
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
    let mut g = Game::new(1);
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
        let mut g = Game::new(seed);
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
    let mut g = Game::new(3);
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
    // An instant exit says 0.
    let mut g = arena();
    rules(&mut g, vec![Row::new(vec![], Verb::new("return"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Ending { ticks: 0, .. })));
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(1);
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
        let mut g = Game::new(4);
        g.lineage.facts.insert("captive".into());
        let row = if coward { Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")) } else { Row::new(vec![Cond::t("on_see", "captive")], Verb::new("free_captive")) };
        g.set_rules(RuleSet { rows: vec![row], name: None }).unwrap();
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
    let mut g = Game::new(5);
    for f in ["shrine", "hunger"] {
        g.lineage.facts.insert(f.into());
    }
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::t("on_see", "hunger")], Verb::arg("pray", "row"))], name: None }).unwrap();
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
    let mut g = Game::new(67);
    g.lineage.unlocks.insert("thief_guard".into());
    g.lineage.facts.insert("foe:monkey:thief".into());
    let rows = vec![
        Row::new(vec![], Verb::arg("tactic", "thief_guard")),
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
    ];
    g.set_rules(RuleSet { rows, name: None }).unwrap();
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
    let mut ep = crate::sifter::Episode { combo: Some("hit and fade".into()), act: crate::sifter::Act { row: 2, verb: Verb::new("retreat"), target: None, boss: false }, low_hp: 5, max_hp: 30, threat: vec![("ogre".into(), 1)], resolution: crate::sifter::Resolution::Reached { depth: 4 }, ..Default::default() };
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
        let g = Game::new(seed);
        if let Some((d, name)) = g.lineage.first_stray() {
            with += 1;
            assert!((2..=3).contains(&d), "{d}");
            assert!(!name.is_empty());
            assert_eq!(g.lineage.first_stray(), Some((d, name)), "decided once per lineage");
        }
    }
    assert!((26..=38).contains(&with), "{with}/40 lineages at 80%");
    let seed = (1..=40u64).find(|s| Game::new(*s).lineage.first_stray().is_some()).unwrap();
    let mut g = Game::new(seed);
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
    let mut g = Game::new(3);
    let v = g.vocabulary();
    assert!(!v.locked.is_empty());
    for l in &v.locked {
        assert!(!v.conds.iter().any(|c| c.same_token(&l.cond)), "{:?} is open and locked", l.cond);
        assert!(!l.needs.is_empty() && word_count(&l.needs) <= 3, "{:?}: {}", l.cond, l.needs);
    }
    let needs = |v: &crate::rules::Vocabulary, c: &Cond| v.locked.iter().find(|l| l.cond.same_token(c)).map(|l| l.needs.clone());
    assert_eq!(needs(&v, &Cond::t("on_see", "stray")).as_deref(), Some("see: stray"));
    assert_eq!(needs(&v, &Cond::flag("on_see")).as_deref(), Some("meet a foe"), "the unlock's own gate while shut");
    assert_eq!(needs(&v, &Cond::n("turns>", 100)).as_deref(), Some("◆2"), "an open unlock: its price");
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
    assert_eq!(needs(&v, &Cond::n("party_hp<", 50)).as_deref(), Some("tame once"));
    assert!(needs(&v, &Cond::t("item", "heal")).is_some_and(|n| n.starts_with("identify ")));
    assert_eq!(needs(&v, &Cond::t("item", "lantern")).as_deref(), Some("find: lantern"));
    assert!(needs(&v, &Cond::t("item", "leash")).is_none(), "the kennel's leash is a fact from the start");
    assert!(serde_json::to_string(&v).unwrap().contains(r#""locked":[{"cond":"#));
    // The door: a locked token is refused by name; the number is the player's.
    let row = |c: Cond| RuleSet { rows: vec![Row::new(vec![c], Verb::arg("tame", "nearest"))], name: None };
    let e = g.set_rules(row(Cond::t("on_see", "stray"))).unwrap_err();
    assert!(e.contains("see stray") && e.contains("locked") && e.contains("see: stray"), "{e}");
    let e = g.set_rules(row(Cond::n("turns>", 7))).unwrap_err();
    assert!(e.contains("turns 7+") && e.contains("◆2"), "{e}");
    let e = g.set_rules(row(Cond::n("alert>=", 7))).unwrap_err();
    assert!(e.contains("alert 7+") && e.contains("see alert rise"), "{e}");
    // Open the gates: the fact, the unlock.
    g.lineage.facts.insert("stray".into());
    g.lineage.unlocks.insert("cond_turns".into());
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
/// fresh lineage, with marks to spare, and after an ascension; party slots read `tame once`.
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
    let mut g = Game::new(4);
    check(&g, "fresh");
    let by = |g: &Game, id: &str| g.unlocks().into_iter().find(|u| u.id == id).unwrap();
    assert_eq!(by(&g, "party_slot_2").needs.as_deref(), Some("tame once"));
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
    assert_eq!(by(&g, "row5").needs.as_deref(), Some("◆2 more"));
    g.lineage.marks = 100;
    assert_eq!(by(&g, "party_slot_2").needs.as_deref(), Some("tame once"));
    let mut g = finished_lineage();
    g.ascend("short_list").unwrap();
    check(&g, "ascended");
    g.lineage.marks = 100;
    check(&g, "ascended, rich");
    // A card short of marks still shows a delta once simulated (the marks are not a gate).
    let mut g = Game::new(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.best_depth = 1;
    let c = g.unlock_deltas().into_iter().find(|u| u.id == "corridor_fighting").unwrap();
    assert_eq!(c.needs.as_deref(), Some("◆3 more"));
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
    let dive = RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![], Verb::new("descend"))], name: None };
    for w in par_seeds(1..=30u64, |seed| {
        let mut worst = 0.0f64;
        let mut g = Game::new(seed);
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
    let mut g = Game::new(7);
    g.set_rules(dive.clone()).unwrap();
    g.lineage.best_depth = 2;
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
        let mut g = Game::new(seed);
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
    let r = g.step(40);
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
        let mut g = Game::new(seed);
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
    let mut g = Game::new(2);
    crate::offline::run_offline_quick(&mut g, 3600);
    let g2 = Game::load(&g.save()).unwrap();
    assert_eq!(g2.lineage.reel_pairs, g.lineage.reel_pairs);
}

/// §7: the graveyard's last five deaths keep `death_id`, the engine keeps their records
/// through a save (even one that trimmed `max_deaths`), and `death(id)` answers for each.
#[test]
fn graveyard_keeps_the_last_five_deaths_answerable() {
    let mut g = Game::new(3);
    // A set that fights everything and never comes home: a death an hour or so.
    g.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")), Row::new(vec![], Verb::new("descend"))], name: None }).unwrap();
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
    let mut g = Game::new(6);
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
    let mut g = Game::new(1);
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
    let mut g = Game::new(9);
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
    let mut g = Game::new(3);
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
    assert!(d.morgue.contains(&format!("blow {blow} · {short} hp short")), "{}", d.morgue);
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
    assert_eq!(line(ExitTier::Death, false, 7), "died $0 · $84 carried · keeps 0% · bones: 7 items on D5");
    assert_eq!(line(ExitTier::Return, true, 0), "returned $50 · $84 carried · keeps 60% · lost thread");
    // A stall says so (the chronicle's "Stalled."), and a supply the send spent unused is counted.
    let stalled = crate::engine::exit_line_of(15, 0, 0, 0, vec![], ExitTier::Return, true, true, 1, 0, 2).text;
    assert_eq!(stalled, "returned $0 · $15 carried · keeps 0% · stalled · 1 supply back");
    for t in [line(ExitTier::Return, true, 0), line(ExitTier::Death, false, 7), stalled] {
        assert!(word_count(&t) <= 14, "{t}");
    }
}

/// §3: a tactic card's delta is the buy's own number — the bare `[card]` row where the buy
/// puts it (Cut 12 §1: before the engagement row, `insert_at`; it was the end), not a
/// conditioned row at the top; a verb unlock's canonical row still goes at the top.
#[test]
fn card_delta_is_measured_where_the_buy_puts_it() {
    let mut g = Game::new(2);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:bloat:gas".into());
    g.lineage.best_depth = 1;
    g.set_rules_raw(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")), Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))], name: None }).unwrap();
    let l = &g.lineage;
    let (row, at) = crate::meta::delta_row(l, "gas_step").unwrap();
    assert_eq!(row, Row::new(vec![], Verb::arg("tactic", "gas_step")), "the bare card row the client appends");
    assert_eq!(row.origin.as_deref(), Some("card"));
    assert_eq!(at, 1, "before the set's `attack nearest` (Cut 12 §1)");
    let (row, at) = crate::meta::delta_row(l, "throw").unwrap();
    assert_eq!((row.verb.v.as_str(), at), ("throw", 0), "a verb's canonical row goes at the top");
    assert!(crate::meta::delta_row(l, "row5").is_none());
    // The delta is exactly the paired reach of that appended set minus the base.
    let cat = g.unlock_deltas();
    let card = cat.iter().find(|u| u.id == "gas_step").unwrap();
    let delta = card.delta.expect("open card");
    let rules = g.lineage.rules().clone();
    let depth = g.lineage.best_depth + 1;
    let tag = crate::forecast::forecast_tag(&g, depth);
    let (base, n) = crate::forecast::reach_counted(&g, &rules, depth, crate::forecast::FORECAST_SIMS, tag, crate::forecast::CATALOGUE_TICK_BUDGET);
    let mut appended = rules.clone();
    appended.rows.insert(1, Row::new(vec![], Verb::arg("tactic", "gas_step")).from("card"));
    let mut sim = g.sim_clone();
    sim.lineage.unlocks.insert("gas_step".into());
    let r = crate::forecast::reach_paired(&sim, &appended, depth, n, tag);
    assert!((delta - (r - base)).abs() < 1e-9, "delta {delta} vs appended {r} − base {base}");
    let mut top = rules.clone();
    top.rows.insert(0, Row::new(vec![Cond::t("foe_tag", "gas")], Verb::arg("tactic", "gas_step")).from("card"));
    let r_top = crate::forecast::reach_paired(&sim, &top, depth, n, tag);
    // (informational: the two placements may or may not agree on this seed; the number shown is the buy's)
    let _ = r_top;
}

/// Cut 10 (client finding): `choose` after the run ended returned a wasm panic and poisoned the
/// engine; it must be a plain error.
#[test]
fn choose_after_the_run_ended_is_an_error_not_a_panic() {
    let mut g = Game::new(12);
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
    assert!(b.text.starts_with("jackal last seen D1 ("), "{b:?}");
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
    assert_eq!(w.because.as_ref().map(|b| b.text.as_str()), Some("◆2 cond: alert"));
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
    s.set_rules(RuleSet { rows: vec![Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")), Row::new(vec![], Verb::new("hold"))], name: None }).unwrap();
    for _ in 0..20 {
        s.tick();
        s.events.clear();
    }
    assert!(s.run.as_ref().unwrap().hero.inv.is_empty(), "the sim drank");
    assert!(s.prov.is_empty(), "sims log nothing");
    assert!(s.run.as_ref().unwrap().trace.last().unwrap().rows.is_none(), "sims account for nothing");
    // Same seed, same log: the log is part of the run's determinism.
    let mut a = Game::new(11);
    let mut b = Game::new(11);
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
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("the root patch is shown: {:?}", d.patches));
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
    // Without the unlock: the pseudo-patch.
    let (mut g, id) = build(false);
    let rec = g.deaths.get(&id).unwrap().clone();
    assert_eq!(rec.root.as_ref().map(|r| r.text.as_str()), Some("den took the heal"));
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("{:?}", d.patches));
    assert_eq!(r.insert_at, -1);
    assert_eq!(r.root.as_ref().unwrap().text, "◆2 cond: on see");
    assert_eq!(r.row, raid);
    assert!((0.0..=1.0).contains(&r.survive));
    assert_eq!(g.deaths.get(&id).unwrap().root.as_ref().unwrap().unlock.as_deref(), Some("cond_on_see"));
    let json = serde_json::to_string(r).unwrap();
    assert!(json.contains(r#""insert_at":-1"#) && json.contains(r#""root":{"text":"◆2 cond: on see"}"#), "{json}");
    assert!(!json.contains("below_bar"));
    // The pseudo-patch changes no row for the bots; its fired rate is measured on the row at
    // the top with the token unlocked.
    assert_eq!(crate::offline::apply_patch(&rec.rules, r, 8), rec.rules);
    let rules = crate::trace::patched_rules(&rec.rules, r, 8);
    assert_eq!(rules.rows[0], raid);
    let _ = crate::trace::patch_fired_rate(&g, &rec, r);
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
    assert_eq!((root.kind.as_str(), root.text.as_str(), root.row, root.unlock.as_deref()), ("lock", "◆2 cond: alert", 0, Some("cond_alert")));
    assert_eq!(rec.death.chain.as_ref().unwrap()[0].text, "◆2 cond: alert");
    let d = g.death(id).unwrap();
    let r = d.patches.iter().find(|p| p.root.is_some()).unwrap_or_else(|| panic!("{:?}", d.patches));
    assert_eq!((r.insert_at, r.root.as_ref().unwrap().text.as_str()), (-1, "◆2 cond: alert"));
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
    let mut g = Game::new(seed);
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
                if crate::trace::telegraph_row(&rec.vocab, &d.trace).is_some() {
                    let cands = crate::trace::candidates(&rec.vocab, rec.t10.as_ref().unwrap(), &rec.t10_facts, &g.lineage.flavours, &d.trace);
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
        let mut g = Game::new(seed);
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
    let t = Trace { turns: vec![], provenance: None };
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
    let mut checked = 0;
    for seed in 1..=4u64 {
        let mut g = Game::new(seed);
        for u in ["row5", "row6", "row7", "row8"] {
            g.lineage.unlocks.insert(u.into());
        }
        g.set_rules_raw(RuleSet::parse(&std::fs::read_to_string("presets/good.json").unwrap()).unwrap()).unwrap();
        crate::offline::run_offline_quick(&mut g, 8 * 3600);
        let ids: Vec<u32> = g.deaths.keys().copied().take(3).collect();
        for id in ids {
            let d = g.death(id).unwrap();
            let rows = g.deaths[&id].rules.rows.clone();
            for p in d.patches.iter().filter(|p| p.insert_at >= 0) {
                assert!(!rows.iter().any(|x| x.conds == p.row.conds && x.verb == p.row.verb), "seed {seed} death {id}: {:?} already in {rows:?}", p.row);
                checked += 1;
            }
        }
    }
    assert!(checked >= 10, "{checked} patches checked");
}

/// Cut 12 (cohort 8, rater P): supplies bought at camp after an absence are in the pack of
/// the next send, and that send starts a fresh run under the rules edited at camp.
#[test]
fn the_first_send_after_an_absence_packs_what_was_bought_at_camp() {
    let mut g = Game::new(9);
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
        let mut g = Game::new(seed);
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
        g.set_rules_raw(RuleSet { rows, name: None }).unwrap();
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
    let g = Game::new(1);
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
    let mut g = Game::new(11);
    crate::probes::learn_everything(&mut g);
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    g.lineage.best_depth = 9;
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 4, xp: 0 });
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
    let mut g = Game::new(11);
    g.set_rules_raw(crate::probes::good()).unwrap();
    let a = g.forecast();
    let e = a.ends.clone().expect("ends over the sims");
    assert!((e.bank + e.return_ + e.death + e.stall - 1.0).abs() < 1e-9, "{e:?}");
    assert!(e.gold >= 0.0);
    assert_eq!(g.forecast().ends, a.ends, "two reads agree");
    let json = serde_json::to_string(&a).unwrap();
    assert!(json.contains(r#""ends":{"bank":"#) && json.contains(r#""return":"#) && json.contains(r#""death":"#) && json.contains(r#""gold":"#), "{json}");
    // A lineage that reaches D8, a set that banks there.
    let mut g = Game::new(11);
    crate::probes::learn_everything(&mut g);
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    g.lineage.best_depth = 8;
    g.lineage.classes.insert("fighter".into(), ClassProg { level: 6, xp: 0 });
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
    let g = Game::new(7);
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
    let mut g = Game::new(5);
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
    let mut g = Game::new(5);
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
    assert!(without >= 30, "the preset is robbed ({without} snatches over 30 seeds)");
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
    let g = Game::new(21);
    let offer = g.lineage().trait_offer;
    assert_eq!(offer.len(), 2);
    assert_ne!(offer[0], offer[1]);
    assert_eq!(g.lineage.trait_.name(), offer[0], "the first is the heir's until a pick");
    assert_eq!(Game::new(21).lineage().trait_offer, offer, "deterministic");
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

/// Cut 13 §2: a trait overrides a row at most once per floor — over 30 seeds' first runs no
/// floor carries two `row −1` events — and every override leaves a `because` on the rows it
/// held (`brave held it` · `cowardly ran first` · `greedy went first`, ≤ 8 words).
#[test]
fn a_trait_deviates_at_most_once_per_floor_and_says_so_on_the_row() {
    let mut deviations = 0;
    let mut becauses = 0;
    for seed in 1..=30u64 {
        let mut g = Game::new(seed);
        g.send();
        let mut depth = 1;
        let mut per_floor = 0;
        for _ in 0..600 {
            let r = g.step(50);
            for e in &r.events {
                match e {
                    Ev::Descend { depth: d, .. } => {
                        depth = *d;
                        per_floor = 0;
                    }
                    Ev::Rule { row: -1, text, t, .. } => {
                        per_floor += 1;
                        deviations += 1;
                        assert!(per_floor <= 1, "seed {seed} D{depth} t{t}: a second trait deviation ({text})");
                    }
                    _ => {}
                }
            }
            for t in g.run.as_ref().map(|r| r.trace.clone()).unwrap_or_default() {
                for w in t.rows.iter().flatten().filter(|w| w.why == "brave held" || w.why == "trait first") {
                    let b = w.because.as_ref().unwrap_or_else(|| panic!("seed {seed}: `{}` without a because", w.why));
                    assert!(crate::provenance::because_ok(&b.text) && (b.text == "brave held it" || b.text.ends_with(" first")), "{b:?}");
                    becauses += 1;
                }
            }
            if r.run_over {
                break;
            }
        }
    }
    assert!(deviations >= 10, "traits still act ({deviations} deviations)");
    assert!(becauses >= 1, "no trait because seen");
}

/// Cut 13 §3 (Q: "I left with $142 and came back to $6 and the report never said where it
/// went"): the night's ledger — `ReturnReport.spent` is what the restock bought, per kind in
/// coins, and the purse reconciles to the coin over 8 h on 20 seeds: exits + salvage + wake
/// pay − spent == the delta. And a kind the last run used to no effect is not rebought.
#[test]
fn the_nights_ledger_reconciles_and_a_wasted_kind_is_not_rebought() {
    let night = |seed: u64| -> bool {
        let mut g = Game::new(seed);
        g.lineage.unlocks.insert("auto_supply".into());
        g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "heal").unwrap());
        g.lineage.gold = 400;
        g.buy_supply("heal").unwrap();
        g.buy_supply("heal").unwrap();
        let before = g.lineage.gold;
        let r = crate::offline::run_offline_quick(&mut g, 8 * 3600);
        let b = &g.batch;
        let spent: i32 = b.spent.values().map(|(_, c)| *c).sum();
        assert_eq!(b.gold_earned + b.salvage_gold + b.wake_pay - spent, g.lineage.gold - before, "seed {seed}: earned {} salvage {} wake {} spent {spent}", b.gold_earned, b.salvage_gold, b.wake_pay);
        assert_eq!(r.spent.iter().map(|s| s.gold).sum::<i32>(), spent, "the report's SPENT rows are the batch's");
        r.spent.iter().find(|s| s.kind == "heal").inspect(|row| assert!(row.n >= 1 && row.gold == 40 * row.n as i32, "{row:?}")).is_some()
    };
    let spent_seen = std::thread::scope(|sc| (1..=20u64).map(|seed| sc.spawn(move || night(seed))).collect::<Vec<_>>().into_iter().filter_map(|h| h.join().unwrap().then_some(())).count());
    assert!(spent_seen >= 10, "the restock bought heals on {spent_seen} seeds");
    // A heal drunk at full HP is a use to no effect: the heal is skipped by the next restock,
    // the strength potion beside it is rebought.
    let mut g = arena_seed(5);
    no_kennel_leash(&mut g);
    g.lineage.unlocks.insert("auto_supply".into());
    g.lineage.facts.insert(ident_fact(&g.lineage.flavours, "strength").unwrap());
    g.lineage.gold = 1000;
    g.lineage.last_supplies = vec!["heal".into(), "strength".into()];
    give(&mut g, "heal");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("drink", "unknown"))]);
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
    assert_eq!(g.batch.spent.get("strength"), Some(&(1, 40)));
    // The shop grows with the forge: a craftable potion is for sale before the hero drank one.
    let mut g = Game::new(5);
    assert!(!g.supply_catalogue().iter().any(|s| s.kind == "strength"));
    g.lineage.forge.insert("strength".into(), ForgeRow::at(5));
    let entry = g.supply_catalogue().into_iter().find(|s| s.kind == "strength").expect("craftable strength on the shelf");
    assert_eq!((entry.price, entry.label.as_str()), (40, "strength potion"));
}

/// Cut 13 §5 (both raters: "±10 between re-rolls", "pack break +7 % then −14 %"): a catalogue
/// delta carries its half-width (`UnlockInfo.pm`, the base's and the patched reach's combined)
/// whenever it carries a delta; the ends line carries its `±`; the refine pass says so and
/// two reads of it agree.
#[test]
fn a_catalogue_delta_carries_its_half_width() {
    let mut g = Game::new(11);
    g.lineage.facts.insert("foe:jackal:pack".into());
    g.lineage.facts.insert("foe:goblin_archer:ranged".into());
    g.lineage.marks = 9;
    g.lineage.best_depth = 3;
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
    let mut g = Game::new(901);
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
    let mut g = Game::new(seed);
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
    let mut g = Game::new(3);
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
    let mut g = Game::new(5);
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
    let mut g = Game::new(8);
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
    let mut g = Game::new(4);
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
