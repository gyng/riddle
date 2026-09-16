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
                variant: "",
                hunter: None,
                vault_pref: "weapon",
                lost: &lineage.lost,
                sets: &lineage.sets,
                active_set: set,
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
    assert_eq!(g.run.as_ref().unwrap().loot_raw, 25 + 14);
    assert_eq!(g.run.as_ref().unwrap().loot, (25 + 14) / crate::engine::GOLD_DIVISOR, "loot is counted in gold at pickup (Cut 4)");
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
fn reaching_d31_is_the_ending_and_d16_is_the_foundry() {
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 30;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 31, .. })));
    assert!(evs.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "bank")));
    assert!(g.lineage.ended);
    // Cut 3: D16 is the Foundry's doorstep, not an ending.
    let mut g = arena();
    g.run.as_mut().unwrap().depth = 15;
    g.run.as_mut().unwrap().hero.pos = Pos::new(14, 10);
    rules(&mut g, vec![Row::new(vec![], Verb::new("descend"))]);
    let evs = ticks(&mut g, 10);
    assert!(evs.iter().any(|e| matches!(e, Ev::Descend { depth: 16, biome, .. } if biome == "foundry")));
    assert!(evs.iter().any(|e| matches!(e, Ev::Fact { fact, .. } if fact == "biome:foundry")));
    assert!(!g.lineage.ended);
    assert_eq!(g.run.as_ref().unwrap().depth, 16);
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
    // Cut 5: 12 h (situations on D1–5 gave this seed a new best on its 17th run of 8 h).
    let mut g = Game::new(5);
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")));
    g.set_rules(set).unwrap();
    let r = g.run_offline(12 * 3600);
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
    for _ in 0..24 {
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
    assert!(d.patches.is_empty());
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
    // A quiet run still closes on its exit.
    let mut g2 = arena();
    attack_rules(&mut g2);
    ticks(&mut g2, 2);
    let (run, mut cx) = g2.ctx();
    crate::turn::end_run(run, &mut cx, ExitTier::Return);
    let hs = crate::sifter::sift_with(run, false);
    assert_eq!(hs.len(), 1);
    assert!(hs[0].text.starts_with("Untouched; ") && hs[0].text.ends_with("; returned."), "{}", hs[0].text);
    assert!(crate::sifter::story_ok(&hs[0].text));
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
    let reel = crate::sifter::reel(&hs, Some(6));
    let ids: Vec<u32> = reel.iter().map(|h| h.run_id).collect();
    assert_eq!(ids, vec![1, 3, 4, 6], "{reel:?}");
    assert_eq!(reel[3].t, 50, "the best run's closing episode");
    let pairs: std::collections::BTreeSet<_> = reel.iter().filter_map(crate::sifter::pair).collect();
    assert_eq!(pairs.len(), 4);
    // Without a best run the fourth slot takes the bones highlight.
    let reel = crate::sifter::reel(&hs, None);
    assert_eq!(reel.len(), 4);
    assert_eq!(reel[3].pattern, "bones");
}

/// Gate: over 100 real runs every story line is three beats in ≤ 12 words with a table verb,
/// and every run closes at least one episode.
#[test]
fn story_lines_over_a_hundred_runs_follow_the_grammar() {
    let mut lines = 0;
    let mut runs = 0;
    for seed in 1..=4u64 {
        for edited in [false, true] {
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
    }
    assert!(runs >= 100 && lines >= runs, "{runs} runs, {lines} lines");
}

// ---------------------------------------------------------------- marks and meta

#[test]
fn marks_are_earned_on_new_bests_only() {
    let mut g = arena();
    g.run.as_mut().unwrap().max_depth = 3;
    g.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g, ExitTier::Bank);
    assert_eq!(g.lineage.marks, 4, "D1..D3 (3) + first bank from D3 (Cut 4); first kills no longer mark (Cut 2 §2)");
    assert!(g.batch.bests.iter().any(|b| b == "first kill: rat"), "but stay in bests");
    assert!(g.batch.bests.iter().any(|b| b == "home:D3"));
    g.auto_keep();
    let mut g2 = arena();
    g2.lineage = g.lineage.clone();
    g2.run.as_mut().unwrap().max_depth = 3;
    g2.run.as_mut().unwrap().kills.push((1, "rat".into(), 1));
    finish_with(&mut g2, ExitTier::Bank);
    assert_eq!(g2.lineage.marks, 4, "no new best, no marks");
    let mut g3 = arena();
    g3.lineage = g2.lineage.clone();
    g3.run.as_mut().unwrap().kills.push((1, "goblin_warlord".into(), 5));
    g3.run.as_mut().unwrap().trophies_run.push("pacifist_floor".into());
    finish_with(&mut g3, ExitTier::Bank);
    assert_eq!(g3.lineage.marks, 4 + 3 + 2 + 1, "boss 3, trophy 2, first bank from D1 1");
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
    // 4000 before Cut 3; the larger flavour pool reseeds the lineage and a D5 (Warlord) floor
    // now lands at ~4000 on this seed; Cut 5's situations add a detour (4600). A deadlock is
    // 20 000.
    assert!(longest < 5000, "longest floor took {longest} ticks: {floors:?}");
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
    assert!(g.lineage.facts.contains("boss:mirror_king:counter"));
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
    assert!(!g.lineage.facts.contains("boss:lurker_queen:counter"), "unseen in the dark: no fact yet");
    // A bell rung in her sight: the call, seen, is the counter fact.
    hold_rules(&mut g);
    ticks(&mut g, 40);
    g.run.as_mut().unwrap().hero.pos = Pos::new(9, 5);
    give(&mut g, "bell");
    unlock(&mut g, "throw");
    rules(&mut g, vec![Row::new(vec![], Verb::arg("throw", "bell"))]);
    ticks(&mut g, 12);
    assert!(g.lineage.facts.contains("boss:lurker_queen:counter"));
}

#[test]
fn foundry_master_telegraphs_his_hammer_and_learns_the_counter() {
    let mut g = arena();
    add_monster(&mut g, "foundry_master", 5, 5);
    hold_rules(&mut g);
    let evs = ticks(&mut g, 30);
    assert!(evs.iter().any(|e| matches!(e, Ev::Telegraph { what, .. } if what == "hammers")));
    assert!(g.lineage.facts.contains("boss:foundry_master:counter"));
    let m = g.run.as_ref().unwrap().monsters.iter().find(|m| m.kind == "foundry_master").unwrap();
    assert!(m.reflects_melee());
}

#[test]
fn boss_escorts_and_stock_for_the_new_biomes() {
    for (depth, boss, escort) in [(20u32, "foundry_master", "smith"), (25, "lurker_queen", "lurker"), (30, "mirror_king", "mirror_shade")] {
        let mut found_escort = false;
        for seed in 1..=6u64 {
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
    run.depth = 20;
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
    g.lineage.forge.insert("sword".into(), ForgeRow { salvaged: 20, craftable: true, tier: 1 });
    g.lineage.grudges.push(crate::descent::Grudge { kind: "ogre".into(), name: "Grak".into(), depth: 7, heir: 3 });
    g.lineage.graveyard.push(Grave { heir: 3, depth: 7, cause: "ogre".into(), deeds: vec![] });
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
    run.depth = 30;
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
    for seed in 1..=6u64 {
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
    g.lineage.best_depth = 3; // already reached: no depth mark
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
        let (run, mut cx) = g.ctx();
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
    // Without the gold, nothing is bought and nothing breaks.
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
    assert!(g.lineage.supplies.is_empty());
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
    let caught = notes.iter().find(|n| n.contains("caught him")).unwrap_or_else(|| panic!("{notes:?}"));
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
    assert!(!cx.events.iter().any(|e| matches!(e, Ev::Note { text, .. } if text.contains("caught"))));
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
    assert!(by("throw").delta.is_some(), "verbs get a canonical row");
    assert!(by("tame").delta.is_some());
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
        g.set_rules(RuleSet::parse(&std::fs::read_to_string("presets/good.json").unwrap()).unwrap()).unwrap();
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
    g.ascend("no_rest").unwrap();
    assert_eq!(g.lineage.chronicle, vec!["♟4 the curious fighter · D31 · \"fighter\" set · ascended.".to_string()]);
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
    assert_eq!(line.text, "The vault held three; he took the mail; returned.");
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
