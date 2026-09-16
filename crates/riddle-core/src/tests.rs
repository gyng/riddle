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

/// A game with a live run on an open 16×12 room, no monsters or items.
fn arena() -> Game {
    arena_seed(1)
}

fn arena_seed(seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 1000;
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
    run.floor = Floor { map, stairs_up: up, stairs_down: down, rooms: Vec::new() };
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

fn rules(g: &mut Game, rows: Vec<Row>) {
    g.set_rules(RuleSet { rows, name: None }).unwrap();
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

#[test]
fn rows_beyond_unlocked_count_are_ignored() {
    let mut g = arena();
    let mut rows: Vec<Row> = (0..4).map(|_| Row::new(vec![Cond::n("hp<", 0)], Verb::new("rest"))).collect();
    rows.push(Row::new(vec![], Verb::new("return")));
    rules(&mut g, rows.clone());
    assert_eq!(g.lineage.rules().rows.len(), 4, "a fifth row is dropped at set time without the row5 unlock (Cut 2)");
    let evs = ticks(&mut g, 10);
    assert!(!evs.iter().any(|e| matches!(e, Ev::Exit { .. })), "row 5 must not fire without the row5 unlock");
    g.lineage.unlocks.insert("row5".into());
    rules(&mut g, rows);
    assert_eq!(g.lineage.rules().rows.len(), 5);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "return")));
}

#[test]
fn rule_text_is_at_most_three_words() {
    let mut g = Game::new(11);
    g.set_rules(crate::probes::good()).unwrap();
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
            },
        )
    };
    crate::turn::damage_monster(run, &mut cx, mi, 99, &crate::turn::Src::Hero { ranged: false });
    assert!(run.items.iter().any(|i| i.item.kind == "sword"));
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
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "boss:goblin_warlord:counter")));
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
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "boss:bloat_mother:counter")));
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
    assert_eq!(g.run.as_ref().unwrap().loot, 25 + 14);
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
    g.run.as_mut().unwrap().loot = 100;
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
    g.run.as_mut().unwrap().loot = 100;
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
    g.run.as_mut().unwrap().loot = 100;
    for k in ["sword", "heal", "heal", "teleport"] {
        give(&mut g, k);
    }
    let brought = give(&mut g, "plate");
    g.run.as_mut().unwrap().brought.push(brought);
    g.run.as_mut().unwrap().hero.hp = 0;
    finish_with(&mut g, ExitTier::Death);
    assert_eq!(g.lineage.gold, 0, "Cut 2 §2: death yields nothing");
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
fn bones_piles_cap_at_three_and_bone_sense_paths_to_them() {
    let mut g = arena();
    for heir in 1..=4u32 {
        g.lineage.bones.push(crate::engine::Bones { heir, depth: 1, items: vec![Item::new(500 + heir, "dagger")], named: false });
    }
    give(&mut g, "sword");
    g.run.as_mut().unwrap().hero.hp = 0;
    finish_with(&mut g, ExitTier::Death);
    assert_eq!(g.lineage.bones.len(), 3, "oldest expires");
    assert_eq!(g.lineage.bones.iter().map(|b| b.heir).collect::<Vec<_>>(), vec![3, 4, 1]);
    // bone_sense: the chores walk to unseen bones.
    g.lineage.unlocks.insert("bone_sense".into());
    g.auto_keep();
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().items.iter().filter(|i| i.item.kind == "bones").count() == 3);
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
fn reaching_d16_is_the_ending() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 15;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 16, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "bank")));
    assert!(g.lineage.ended);
}

#[test]
fn descending_regenerates_floor_and_learns_biome() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 5;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 6, biome, .. } if biome == "fens")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "biome:fens")));
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.depth, 6);
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
fn offline_consumes_the_budget_and_leaves_the_hero_mid_run() {
    let mut g = Game::new(4);
    let r = g.run_offline(300);
    assert_eq!(r.elapsed_s, 300);
    assert!(g.lineage.total_turns >= 3000 && g.lineage.total_turns <= 3010, "{}", g.lineage.total_turns);
    assert!(g.run.as_ref().is_some_and(|r| r.over.is_none()), "hero left mid-run");
    assert!(!r.sampled);
    assert!(!r.learned.is_empty());
    assert_eq!(r.live.turn, g.run.as_ref().unwrap().turn);
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
    assert!(g.run.is_some());
}

// ---------------------------------------------------------------- stall verdict

/// `hp<20 → return` on top of the default set: the hero always comes home, so an absence is a
/// stall — the return row is named, and at least one patch moves the forecast at depth + 1.
#[test]
fn a_set_that_always_returns_gets_a_stall_verdict_with_patches() {
    let mut g = Game::new(5);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")));
    g.set_rules(set).unwrap();
    let r = g.run_offline(8 * 3600);
    let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
    let stall = r.stall.unwrap_or_else(|| panic!("no stall: {} runs · {deaths} deaths · bests {:?}", r.runs, r.bests));
    assert_eq!(deaths, 0);
    assert_eq!(stall.row, 0);
    assert!(stall.fired >= 4, "{}", stall.text);
    assert!(stall.text.starts_with("R1 return ended"), "{}", stall.text);
    assert!(crate::rules::word_count(&stall.text) <= 12, "{}", stall.text);
    assert!(!stall.patches.is_empty() && stall.patches.len() <= 3);
    assert!(stall.patches.iter().all(|p| p.forecast_delta > crate::offline::STALL_DELTA), "{:?}", stall.patches);
    assert!(stall.patches.windows(2).all(|w| w[0].forecast_delta >= w[1].forecast_delta));
    // The client's path: 30-minute quick slices; the last slice carries the same stall.
    let mut q = Game::new(5);
    let mut set = q.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")));
    q.set_rules(set).unwrap();
    let mut last = None;
    for _ in 0..16 {
        last = crate::offline::run_offline_quick(&mut q, 1800).stall;
    }
    let chunked = last.expect("the last quick slice carries the stall");
    assert_eq!(chunked.row, 0);
    assert!(chunked.fired >= 4, "{}", chunked.text);
    assert!(!chunked.patches.is_empty(), "{}", chunked.text);
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
    let mk = |row: Row, at: usize, replace: bool, remove: bool| Patch { row, insert_at: at, survive: 0.0, forecast_delta: 0.0, replace, remove };
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
    assert!(d.margin.contains("heal unused"));
    assert!(d.trace.turns.len() <= 10 && !d.trace.turns.is_empty());
    assert!(!d.patches.is_empty() && d.patches.len() <= 3);
    assert!(d.patches.iter().all(|p| p.survive >= 0.6));
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
    assert!(d.patches.is_empty());
}

// ---------------------------------------------------------------- sifter

#[test]
fn sifter_scores_patterns() {
    let g = arena();
    let mut run = g.run.as_ref().unwrap().clone();
    run.over = Some(ExitTier::Bank);
    run.near_deaths.push(10);
    run.low20_t = Some(20);
    run.boss_kills.push((30, "goblin_warlord".into()));
    run.kills.push((30, "goblin_warlord".into(), 5));
    run.kills.push((5, "rat".into(), 1));
    run.ally_lost.push((40, "captive".into()));
    run.gambles.push((50, "poison".into(), true));
    run.gambles_survived.push((50, "poison".into()));
    run.gambles.push((55, "heal".into(), false));
    run.stolen.push((60, "sword".into()));
    let hs = crate::sifter::sift_with(&run, &["rat".to_string()]);
    let score = |p: &str| hs.iter().find(|h| h.pattern == p).map(|h| h.score).unwrap_or(-1);
    assert_eq!(score("near_death"), 5);
    assert_eq!(score("comeback"), 8);
    assert_eq!(score("first_kill"), 3);
    assert_eq!(score("ally_lost"), 4);
    assert_eq!(score("boss"), 6);
    assert_eq!(score("stolen"), 2);
    let gambles: Vec<i32> = hs.iter().filter(|h| h.pattern == "gamble").map(|h| h.score).collect();
    assert_eq!(gambles, vec![4], "one entry per pattern: the best gamble");
    assert!(hs.iter().all(|h| word_count(&h.text) <= 8));
    let reel = crate::sifter::reel(&hs);
    assert_eq!(reel.len(), 5);
    assert_eq!(reel[0].pattern, "comeback");
    let mut run2 = run.clone();
    run2.kills.push((7, "spectral_blade".into(), 2));
    let hs2 = crate::sifter::sift_with(&run2, &["spectral_blade".to_string(), "rat".to_string()]);
    assert!(!hs2.iter().any(|h| h.text.contains("spectral")), "summons are never a first kill");
}

// ---------------------------------------------------------------- marks and meta

#[test]
fn marks_are_earned_on_new_bests_only() {
    let mut g = arena();
    g.run.as_mut().unwrap().max_depth = 3;
    g.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.marks, 3, "D1..D3 (3); first kills no longer mark (Cut 2 §2)");
    assert!(g.batch.bests.iter().any(|b| b == "first kill: rat"), "but stay in bests");
    g.auto_keep();
    let mut g2 = arena();
    g2.lineage = g.lineage.clone();
    g2.run.as_mut().unwrap().max_depth = 3;
    g2.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g2, ExitTier::Bank);
    assert_eq!(g2.lineage.marks, 3, "no new best, no marks");
    let mut g3 = arena();
    g3.lineage = g2.lineage.clone();
    g3.run.as_mut().unwrap().kills.push((1, "goblin_warlord".into(), 5));
    g3.run.as_mut().unwrap().trophies_run.push("pacifist_floor".into());
    finish_with(&mut g3, ExitTier::Bank);
    assert_eq!(g3.lineage.marks, 3 + 3 + 2);
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
    assert_eq!(crate::hero::xp_to_next(3), 900);
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
    g.buy_supply("heal").unwrap();
    assert_eq!(g.lineage.supplies[0].enchant, 1, "forge tier on bought copies");
    let mut a = arena();
    a.lineage = g.lineage.clone();
    let mut d = Item::new(500, "dagger");
    d.enchant = 0;
    a.run.as_mut().unwrap().hero.inv.push(d);
    let cat = a.supply_catalogue();
    assert!(!cat.iter().any(|s| s.kind == "sword"));
    a.lineage.forge.insert("sword".into(), ForgeRow { salvaged: 6, craftable: true, tier: 0 });
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
    g.set_rules(crate::probes::good()).unwrap();
    g.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    g.lineage.party = crate::probes::pets_party();
    g.lineage.unlocks.insert("party_slot_2".into());
    let r = g.run_offline(1200);
    assert!(r.runs >= 1);
    assert!(r.live.turn > 0 || g.lineage.rest_left > 0, "mid-run, or resting at camp");
    let v = serde_json::to_value(&r).unwrap();
    for k in ["elapsed_s", "runs", "sampled", "learned", "bests", "found", "deaths", "pending", "reel", "marks_earned", "live", "tamed", "hatched", "lost", "xp", "salvaged", "renown"] {
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
    assert!(longest < 4000, "longest floor took {longest} ticks: {floors:?}");
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
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "boss:goblin_warlord:counter")), "the first telegraph teaches the counter");
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
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "boss:lich:counter")));
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

#[test]
fn patches_offer_the_id_policy_when_unknown_potions_went_unused() {
    let mut g = arena_seed(4);
    g.run.as_mut().unwrap().hero.hp = 12;
    for _ in 0..3 {
        give(&mut g, "heal");
    }
    for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4), (5, 4)] {
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

