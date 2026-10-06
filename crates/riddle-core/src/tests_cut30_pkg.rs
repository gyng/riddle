//! Cut 30 (the idle-first pivot, docs/CUT30.md): the idle floor (`Steady`, drills, scars, the
//! quartermaster, short absences) and packages v1 (compile order, provenance, levels, the pen, the
//! save mapping), the town's bank and the quest board.
use crate::engine::{ExitTier, Game};
use crate::packages;
use crate::rules::{Cond, Row, RuleSet, Verb};

fn origins(g: &Game) -> Vec<String> {
    g.lineage.rules().rows.iter().map(|r| r.origin.clone().unwrap_or_default()).collect()
}

/// §1: a new lineage's set is the school stance, compiled and tagged — it banks and returns on its own.
#[test]
fn a_new_lineage_climbs_on_steady() {
    let g = Game::new_resident(3);
    let set = g.lineage.rules();
    assert!(set.rows.iter().all(|r| r.origin.as_deref() == Some("stance:steady")), "{:?}", origins(&g));
    let verbs: Vec<&str> = set.rows.iter().map(|r| r.verb.v.as_str()).collect();
    // (Cut 30.5, the owner: a new record never ends the run — home hurt (a bank past the record, a return before
    // it) or out of heals)
    assert_eq!(verbs, ["drink", "bank", "return", "bank", "attack"], "{:?}", set.rows);
    assert_eq!(set.rows[1].conds.iter().map(|c| c.k.as_str()).collect::<Vec<_>>(), ["hp<", "depth>="], "the hurt bank first");
    assert_eq!(set.rows[3].conds.iter().map(|c| c.k.as_str()).collect::<Vec<_>>(), ["lacks", "hp<"], "out of heals");
    assert_eq!(set.own_rows(), 0, "package rows sit outside the cap");
    assert!(set.validate().is_ok());
    let w = g.lineage();
    assert_eq!(w.packages.stance, "steady");
    assert!(!w.packages.pen_open);
    // a harness's lineage is the class preset as written
    let l = Game::new_literal(3);
    assert!(l.lineage.pkg.literal && l.lineage.rules().rows.len() == 2);
}

/// §2: the compile order is fixed by kind — pen › drills › stance guard › tactic › temperament ›
/// stance fallback — each row tagged with its package (provenance); a same-verb row above wins.
#[test]
fn packages_compile_in_their_fixed_order() {
    let mut g = Game::new_resident(5);
    let l = &mut g.lineage;
    l.kills.insert("goblin_warlord".into());
    l.heir = 3;
    packages::arrive(l);
    packages::equip(l, "boss_focus", 0).unwrap();
    packages::wear(l, Some("skittish"));
    l.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    l.pkg.drills.push(packages::Drill { boss: "goblin_warlord".into(), rows: packages::drill_rows("goblin_warlord", 30), revoked: false, announced: false });
    l.pkg.pen_open = true;
    packages::recompile(l);
    let pen = Row::new(vec![Cond::n("hp<", 50)], Verb::new("rest")).from("player");
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, pen.clone());
    g.set_rules(set).unwrap();
    let o = origins(&g);
    let kinds: Vec<&str> = o.iter().map(|s| s.split(':').next().unwrap_or("")).collect();
    let first = |k: &str| kinds.iter().position(|x| *x == k).unwrap();
    let last = |k: &str| kinds.iter().rposition(|x| *x == k).unwrap();
    assert_eq!(kinds[0], "player", "{o:?}");
    assert!(first("drill") < first("stance") && first("stance") < first("tactic") && last("tactic") < first("temper") && first("temper") < last("stance"), "{o:?}");
    assert_eq!(g.lineage.rules().rows.last().unwrap().verb.v, "attack", "the stance's fallback closes the set");
    assert_eq!(g.lineage.rules().own_rows(), 1, "only the pen's row counts against the cap");
    // provenance on the wire: each row's package label
    let w = g.lineage();
    assert_eq!(w.packages.rows[0].label, "");
    assert!(w.packages.rows.iter().any(|r| r.label == "drill · Warlord"), "{:?}", w.packages.rows);
    assert!(w.packages.rows.iter().any(|r| r.label == "Steady"));
    assert!(w.packages.rows.iter().any(|r| r.label == "boss focus"));
    // the pen's row survives a recompile (a level, a drill)
    packages::recompile(&mut g.lineage);
    assert_eq!(g.lineage.rules().rows[0], pen);
}

/// §2: the pen is late — before it opens, a written row waits in the pen, uncompiled.
#[test]
fn the_pen_writes_nothing_until_it_opens() {
    let mut g = Game::new_resident(7);
    let before = g.lineage.rules().clone();
    let mut set = before.clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 60)], Verb::new("rest")));
    g.set_rules(set).unwrap();
    assert_eq!(*g.lineage.rules(), before, "closed pen: the stance plays");
    // the Mother met is not enough before 72 h of age; at 72 h it opens (one reveal); at 5 days without her
    g.lineage.pkg.meets.insert("bloat_mother".into(), 1);
    g.lineage.clock_s = 24 * 3600;
    g.lineage.reveal_left = 1;
    crate::systems::update(&mut g.lineage, false);
    assert!(!g.lineage.pkg.pen_open, "a day old: the pen waits");
    g.lineage.clock_s = crate::systems::PEN_AGE_H as u64 * 3600;
    // (one reveal a report: the pen waits its turn behind what came before it)
    for _ in 0..crate::systems::SYSTEMS.len() {
        g.lineage.reveal_left = 1;
        crate::systems::update(&mut g.lineage, false);
    }
    assert!(g.lineage.pkg.pen_open, "the Mother met and 72 h");
    assert_eq!(g.lineage.rules().rows[0].verb.v, "rest");
    let mut h = Game::new_resident(8);
    h.lineage.clock_s = crate::systems::PEN_FALLBACK_H as u64 * 3600;
    for _ in 0..crate::systems::SYSTEMS.len() {
        h.lineage.reveal_left = 1;
        crate::systems::update(&mut h.lineage, false);
    }
    assert!(h.lineage.pkg.pen_open, "five days: the pen opens whatever the climb");
}

/// PROGRESSION_V2 §4: one new system a report — the rest wait in the queue, shown as the next stage;
/// a report's beats are ≤ 5 (`+N more`).
#[test]
fn one_system_a_report_and_five_beats() {
    let mut g = Game::new_resident(9);
    g.lineage.clock_s = 10 * 24 * 3600;
    g.lineage.graveyard.push(crate::wire::Grave { heir: 1, depth: 3, cause: "rat".into(), deeds: Vec::new(), death_id: None });
    g.lineage.facts.insert("stray".into());
    g.lineage.facts.insert("foe:goblin_warlord".into());
    g.lineage.reveal_left = 1;
    let opened = crate::systems::update(&mut g.lineage, false);
    assert_eq!(opened, vec!["death"]);
    assert!(g.lineage.reveal_queue.len() >= 2, "{:?}", g.lineage.reveal_queue);
    assert_eq!(g.lineage().reveal_next.map(|n| n.id), g.lineage.reveal_queue.first().cloned());
    let lines: Vec<String> = ["STEADY L2", "STEADY L3", "+Guarded", "DRILLED · Warlord", "built bank", "the pen", "QUEST DONE · reach D9"].iter().map(|s| s.to_string()).collect();
    let b = packages::beats(&lines);
    assert_eq!(b.len(), packages::BEATS);
    assert!(b.contains(&"STEADY L3".to_string()) && !b.contains(&"STEADY L2".to_string()));
    assert_eq!(b.last().unwrap(), "+2 more");
}

/// §1: a known counter enters as a drill at the boss's second meeting (the second run that sees him;
/// the scars count days), announced once; revoking it persists through a save; a revoked drill is not
/// compiled.
#[test]
fn a_drill_comes_at_the_second_meeting_and_stays_revoked() {
    let mut g = Game::new_resident(9);
    g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    let met = vec!["goblin_warlord".to_string()];
    let set = g.lineage.rules().clone();
    let lines = packages::on_run_end(&mut g.lineage, &met, 8, &[], &set);
    assert!(g.lineage.pkg.drills.is_empty() && lines.iter().all(|l| !l.starts_with("DRILLED")));
    let lines = packages::on_run_end(&mut g.lineage, &met, 8, &[], &set);
    assert_eq!(g.lineage.pkg.meets["goblin_warlord"], 1, "one scar a day");
    assert!(lines.iter().any(|l| l == "DRILLED · Warlord"), "the second run that meets him: {lines:?}");
    assert!(origins(&g).iter().any(|o| o == "drill:goblin_warlord"));
    let drill = g.lineage.rules().rows.iter().find(|r| r.origin.as_deref() == Some("drill:goblin_warlord")).unwrap().clone();
    assert_eq!(drill.verb, Verb::arg("attack", "tag:boss"));
    assert!(drill.conds.iter().any(|c| c.k == "hp>"), "a drill yields to the stance's heal");
    g.revoke_drill("goblin_warlord", true).unwrap();
    assert!(!origins(&g).iter().any(|o| o.starts_with("drill:")));
    let mut h = Game::load(&g.save()).unwrap();
    assert!(h.lineage.pkg.drills[0].revoked);
    h.lineage.day = 2;
    let set = h.lineage.rules().clone();
    packages::on_run_end(&mut h.lineage, &met, 8, &[], &set);
    assert!(!origins(&h).iter().any(|o| o.starts_with("drill:")), "a revoked drill stays out");
    assert_eq!(h.lineage().packages.drills.len(), 1);
}

/// §1: each meeting scars the boss −5 % max hp for the lineage (cap −30 %), cleared once slain; a
/// send carries the scars into its run, and the boss spawns scarred.
#[test]
fn scars_are_capped_cleared_and_carried() {
    let mut g = Game::new_resident(11);
    for day in 0..9u32 {
        g.lineage.day = day;
        let set = g.lineage.rules().clone();
        packages::on_run_end(&mut g.lineage, &["goblin_warlord".to_string()], 8, &[], &set);
        let want = ((day + 1) * packages::SCAR_PCT).min(packages::SCAR_CAP);
        assert_eq!(g.lineage.pkg.scar("goblin_warlord", &g.lineage.kills), want);
    }
    g.start_run(Some(3));
    assert!(g.run.as_ref().unwrap().scars.contains(&("goblin_warlord".to_string(), packages::SCAR_CAP)));
    // the spawn: a run on the boss floor meets him at 70 % of his hp
    let full = crate::defs::monster_def("goblin_warlord").hp;
    let mut run = g.run.take().unwrap();
    run.depth = 8;
    crate::engine::populate_floor(&mut run, &[], &Default::default(), None);
    let boss = run.monsters.iter().find(|m| m.kind == "goblin_warlord").expect("the Warlord on D8");
    assert_eq!(boss.max_hp, full * 70 / 100);
    g.lineage.kills.insert("goblin_warlord".into());
    assert_eq!(g.lineage.pkg.scar("goblin_warlord", &g.lineage.kills), 0, "slain: the scars are gone");
}

/// §1: the quartermaster packs a drill's item before any other supply (the slot reserved).
#[test]
fn the_quartermaster_reserves_a_drills_item() {
    let mut g = Game::new_resident(13);
    for k in ["heal", "fire"] {
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    g.lineage.best_depth = 12;
    g.lineage.gold = 5000;
    g.lineage.pkg.drills.push(packages::Drill { boss: "bloat_mother".into(), rows: packages::drill_rows("bloat_mother", 30), revoked: false, announced: false });
    packages::recompile(&mut g.lineage);
    let cap = g.lineage.supply_cap();
    while g.lineage.supplies.len() < cap {
        g.buy_supply("heal").unwrap();
    }
    g.restock();
    assert!(g.lineage.supplies.iter().any(|s| s.kind == "fire"), "{:?}", g.lineage.supplies.iter().map(|s| &s.kind).collect::<Vec<_>>());
    assert!(g.lineage.supplies.len() <= cap);
    assert_eq!(packages::quartermaster(&g.lineage), vec!["fire".to_string()]);
}

/// §2: levels come from runs one of the package's rows fired in — offline included — and a level is a
/// report line; a mark spend buys the next one.
#[test]
fn packages_level_from_offline_runs() {
    let mut g = Game::new_resident(17);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    let r = g.run_offline(8 * 3600);
    assert!(r.runs >= 10, "{} runs", r.runs);
    let runs = g.lineage.pkg.runs.get("steady").copied().unwrap_or(0);
    assert!(runs > 0 && runs <= r.runs, "steady ran {runs} of {}", r.runs);
    assert!(g.lineage.pkg.level("steady") >= 2);
    // (a level is a beat — shown, or folded into the report's `+N more` when the beats are full)
    assert!(r.packages.iter().any(|l| l.starts_with("STEADY L")) || r.packages.last().is_some_and(|l| l.starts_with('+') && l.ends_with(" more")), "{:?}", r.packages);
    g.lineage.marks = 10;
    let lv = g.lineage.pkg.level("steady");
    assert_eq!(g.spend_level("steady").unwrap(), lv + 1);
    assert!(g.lineage.marks < 10);
}

/// §2: an old save (before the packages) keeps its set as the `custom` stance, the pen open; its
/// rows play as written and the editor still writes them.
#[test]
fn an_old_save_keeps_its_rules_as_a_custom_stance() {
    let mut g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    assert_eq!(g.lineage.pkg.stance, packages::CUSTOM);
    assert!(g.lineage.pkg.pen_open && !g.lineage.pkg.literal);
    let rows = g.lineage.pkg.custom.clone();
    assert_eq!(g.lineage.rules().rows, rows, "the set plays as written");
    let mut set = g.lineage.rules().clone();
    set.rows.remove(set.rows.len() - 1);
    g.set_rules_raw(set.clone()).unwrap();
    g.lineage.pkg.literal = false;
    let mut edited = set.clone();
    edited.rows.swap(0, 1);
    g.set_rules(edited.clone()).unwrap();
    assert_eq!(g.lineage.rules().rows, edited.rows, "the pen edits the custom stance in place");
}

/// §1: a short absence pays — 20 minutes on a fresh lineage returns a run.
#[test]
fn a_twenty_minute_absence_returns_a_run() {
    for seed in 1..=4u64 {
        let mut g = Game::new_resident(seed);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
        let r = g.run_offline(20 * 60);
        assert!(r.runs >= 1, "seed {seed}: {} runs", r.runs);
    }
}

/// §3: the bank takes deposits once built, caps them, pays a night's interest (never negative).
#[test]
fn the_bank_is_capped_and_pays_interest() {
    let mut g = Game::new_resident(19);
    g.lineage.gold = 1000;
    assert!(g.bank_deposit(100).is_err(), "no bank yet");
    g.lineage.last_night_net = 200;
    g.build_town("bank").unwrap();
    assert!(crate::town::built(&g.lineage, "bank"));
    let moved = g.bank_deposit(5000).unwrap();
    assert_eq!(moved, crate::town::bank_cap(&g.lineage));
    assert_eq!(g.lineage.gold, 1000 - moved);
    let i = crate::town::night(&mut g.lineage);
    assert_eq!(i, moved * crate::town::BANK_PCT / 100);
    assert!(i >= 0);
    let before = g.lineage.gold;
    g.bank_withdraw(50).unwrap();
    assert_eq!(g.lineage.gold, before + 50);
}

/// §5: the quest board — one quest from the Warlord slain, progress, a reward kept once, one free swap a day.
#[test]
fn the_quest_board_keeps_one_goal() {
    let mut g = Game::new_resident(23);
    assert!(g.swap_quest().is_err(), "no board before the Warlord falls");
    g.lineage.kills.insert("goblin_warlord".into());
    g.lineage.best_depth = 9;
    assert!(crate::town::on_run(&mut g.lineage, 5, ExitTier::Return, true, 5, &[]).is_none());
    let q = g.lineage.town.quest.clone().expect("a quest drawn");
    assert!(crate::rules::word_count(&crate::town::goal_text(&q)) <= 5);
    g.swap_quest().unwrap();
    assert!(g.swap_quest().is_err(), "one free swap a day");
    let q = g.lineage.town.quest.clone().unwrap();
    let line = crate::town::on_run(&mut g.lineage, 40, ExitTier::Bank, false, 40, &["bloat_mother".into(), "goblin_warlord".into(), "lich".into()]);
    assert!(line.is_some_and(|l| l.starts_with("QUEST DONE")), "{q:?}");
    assert_eq!(g.lineage.town.quests_done, 1);
    assert!(crate::town::on_run(&mut g.lineage, 40, ExitTier::Bank, false, 40, &[]).is_none(), "kept once");
    let w = g.lineage();
    assert!(w.town.quest.as_ref().is_some_and(|q| q.done && q.progress >= 1.0));
}

/// §2: equipping is free and instant, only what has arrived, and the stance is never empty.
#[test]
fn equipping_is_free_and_gated_by_arrival() {
    let mut g = Game::new_resident(29);
    assert!(g.equip_package("guarded", 0).is_err(), "not yet");
    g.lineage.facts.insert("foe:goblin_warlord".into());
    packages::arrive(&mut g.lineage);
    let gold = g.lineage.gold;
    g.equip_package("guarded", 0).unwrap();
    assert_eq!(g.lineage.gold, gold);
    assert!(origins(&g).iter().all(|o| o == "stance:guarded"));
    assert!(g.unequip_package("guarded").is_err(), "the stance is never empty");
    assert!(g.equip_package("boss_focus", 0).is_err(), "the tactic slot opens with the Warlord slain");
    g.lineage.kills.insert("goblin_warlord".into());
    packages::arrive(&mut g.lineage);
    g.equip_package("boss_focus", 0).unwrap();
    assert!(g.lineage.rules().rows.iter().any(|r| r.card() == Some("boss_focus")));
    let set: RuleSet = g.lineage.rules().clone();
    assert!(set.validate().is_ok());
}

/// §5 (owner check, 2026-10-02: `reach D6 · QUEST DONE` beside best D20 after a 48 h absence): a kept
/// quest gives way to the next day's draw at each day an absence crosses, and a draw follows the
/// record as it stands.
#[test]
fn a_kept_quest_is_redrawn_each_day_of_an_absence() {
    let mut g = Game::new_resident(23);
    g.lineage.kills.insert("goblin_warlord".into());
    g.lineage.best_depth = 9;
    // kept on day 0
    crate::town::draw(&mut g.lineage);
    assert!(crate::town::on_run(&mut g.lineage, 9, ExitTier::Bank, false, 9, &[]).is_some());
    // the same day: the board keeps it
    assert!(crate::town::on_run(&mut g.lineage, 9, ExitTier::Bank, false, 9, &[]).is_none());
    assert!(g.lineage.town.quest.as_ref().is_some_and(|q| q.done && q.day == 0));
    // the record climbs, a day passes inside the absence (`LineageState::day` still 0): the day's draw, from the new record
    g.lineage.best_depth = 20;
    g.lineage.town.today = 1;
    crate::town::roll(&mut g.lineage);
    let q = g.lineage.town.quest.clone().unwrap();
    assert!(!q.done && q.day == 1, "{q:?}");
    let floor = match q.goal.as_str() {
        "reach" => 17,
        "reach_no_return" => 15,
        "slay" => 2,
        _ => 2,
    };
    assert!(q.depth >= floor, "a draw follows the record: {q:?}");
    // drawn once a day
    crate::town::roll(&mut g.lineage);
    assert_eq!(g.lineage.town.quest.as_ref().unwrap().id, q.id);

    // a whole absence in one call: the board at the return holds the return day's quest, from the record then
    let mut g = Game::new_resident(3101);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    let _ = crate::offline::run_offline_counts(&mut g, 72 * 3600);
    let l = &g.lineage;
    let q = l.town.quest.clone().expect("the board is open by the third day");
    let day = (l.clock_s / crate::engine::DAY_S) as u32;
    assert!(l.town.quests_done >= 2, "a quest a day: {} kept", l.town.quests_done);
    assert!(q.day == day || !q.done, "a kept quest is from the return's day: {q:?} on day {day}");
    if q.goal == "reach" {
        assert_eq!(q.depth, l.best_depth.saturating_sub(3).max(2), "{q:?} best D{}", l.best_depth);
    }
}

/// §4 (owner check, 2026-10-02: the hero's line read `xp · package level · opened a class · L3 · opened
/// +Guarded · …`): what grew names each fact once, without `opened`, the stages first, xp last.
#[test]
fn grew_names_each_fact_once_stages_first() {
    let mut g = Game::new_resident(3101);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
    let a = crate::town::snap(&g.lineage);
    let _ = crate::offline::run_offline_counts(&mut g, 8 * 3600);
    let grew = crate::town::grew(&a, &crate::town::snap(&g.lineage));
    assert!(!grew.is_empty());
    assert!(grew.iter().all(|x| !x.what.starts_with("opened") && x.what != "a building" && x.what != "package level"), "{grew:?}");
    let mut seen = std::collections::BTreeSet::new();
    for x in &grew {
        assert!(seen.insert((x.track.clone(), x.what.clone())), "once: {grew:?}");
    }
    let hero: Vec<&str> = grew.iter().filter(|x| x.track == "character").map(|x| x.what.as_str()).collect();
    if let Some(i) = hero.iter().position(|w| *w == "xp") {
        assert_eq!(i, hero.len() - 1, "xp last: {hero:?}");
    }
    // a building is named once, on the town's row
    assert!(!grew.iter().any(|x| x.track == "items" && x.what == "storehouse"), "{grew:?}");
}

#[test]
fn every_package_level_produces_valid_rules() {
    for p in packages::PACKAGES {
        for level in 1..=packages::MAX_LEVEL {
            let rows = match p.kind {
                packages::Kind::Stance => { let (mut guard, fallback) = packages::stance_rows(p.id, level, 34); guard.extend(fallback); guard },
                packages::Kind::Tactic => packages::tactic_rows(p.id, level),
                packages::Kind::Temperament => packages::temperament_rows(p.id, level),
            };
            let set = RuleSet { rows: rows.into_iter().map(|r| r.from(&format!("{}:{}", p.kind.origin(), p.id))).collect(), name: None, route: Vec::new() };
            assert!(set.validate().is_ok(), "{} L{}: {:?}", p.id, level, set.validate());
        }
    }
}

#[test]
fn automatic_surplus_supplies_make_room_for_sustain_without_touching_owned_items() {
    let setup = || {
        let mut g = Game::new_resident(13);
        g.lineage.supplies.clear();
        g.lineage.gold_move(10_000 - g.lineage.gold, "test");
        for kind in ["heal", "fire", "silence"] {
            if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, kind) { g.lineage.facts.insert(f); }
        }
        for kind in ["fire", "fire", "silence"] {
            g.buy_supply(kind).unwrap();
            g.lineage.supplies.last_mut().unwrap().auto_packed = true;
        }
        assert_eq!(g.lineage.supplies.len(), g.lineage.supply_cap());
        g
    };
    let mut g = setup();
    let gold = g.lineage.gold;
    let ledger = g.lineage.tree.ledger;
    g.restock();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 2);
    assert!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").all(|s| s.auto_packed));
    assert_eq!(g.lineage.tree.ledger - ledger, (g.lineage.gold - gold) as i64);
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().hero.inv.iter().filter(|i| i.kind == "heal").count(), 2, "sustain actually reaches the hero");

    // The observed shelf: an obsolete automatic fire, a found fire, and the active
    // Queen counter. Only the obsolete supply yields its slot to the stance's heal.
    let mut mixed = setup();
    mixed.lineage.best_depth = crate::descent::boss_depth("lurker_queen").unwrap();
    mixed.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows: packages::drill_rows("lurker_queen", 30), revoked: false, announced: false });
    packages::recompile(&mut mixed.lineage);
    mixed.lineage.supplies[1].found = true;
    let found_id = mixed.lineage.supplies[1].id;
    let counter_id = mixed.lineage.supplies[2].id;
    assert_eq!(packages::quartermaster(&mixed.lineage), ["silence"]);
    mixed.restock();
    assert!(mixed.lineage.supplies.iter().any(|s| s.id == found_id && s.found));
    assert!(mixed.lineage.supplies.iter().any(|s| s.id == counter_id && s.kind == "silence"));
    mixed.start_run(None);
    let inv = &mixed.run.as_ref().unwrap().hero.inv;
    assert!(inv.iter().any(|s| s.kind == "heal"), "sustain reaches the hero beside the counter and find");
    assert!(inv.iter().any(|s| s.kind == "silence"));
    assert!(inv.iter().any(|s| s.kind == "fire" && s.found));

    let mut protected = setup();
    protected.lineage.supplies[0].auto_packed = false;
    protected.lineage.supplies[1].found = true;
    protected.lineage.supplies[2].free = true;
    let before = protected.lineage.supplies.clone();
    protected.restock();
    assert_eq!(protected.lineage.supplies, before, "manual, found and free supplies keep their slots");

    for offline in [false, true] {
        let mut poor = setup();
        if offline { poor.offline = true; } else { poor.lineage.gold_move(-poor.lineage.gold, "test"); }
        let before = poor.lineage.supplies.clone();
        let gold = poor.lineage.gold;
        poor.restock();
        assert_eq!(poor.lineage.supplies, before, "unaffordable replacement discards nothing");
        assert_eq!(poor.lineage.gold, gold);
    }
    let mut legacy: serde_json::Value = serde_json::from_str(&setup().save()).unwrap();
    for item in legacy["lineage"]["supplies"].as_array_mut().unwrap() { item.as_object_mut().unwrap().remove("auto_packed"); }
    let mut old = Game::load(&legacy.to_string()).unwrap();
    assert!(old.lineage.supplies.iter().all(|s| !s.auto_packed));
    let before = old.lineage.supplies.clone();
    old.restock();
    assert_eq!(old.lineage.supplies, before, "old saves do not invent permission to replace supplies");
}

#[test]
fn repeat_preserves_each_supply_origin_through_send_and_save() {
    let mut g = Game::new_literal(13);
    g.lineage.supplies.clear();
    g.lineage.gold_move(10_000 - g.lineage.gold, "test");
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(f); }
    g.buy_supply("heal").unwrap();
    g.lineage.supplies.last_mut().unwrap().auto_packed = true;
    g.buy_supply("heal").unwrap(); // same kind, explicitly packed by the player
    g.start_run(None);
    assert_eq!(g.lineage.last_supply_origins, [("heal".into(), true), ("heal".into(), false)]);
    g = Game::load(&g.save()).unwrap();
    {
        let (run, mut cx) = g.ctx();
        run.hero.inv.retain(|i| i.kind != "heal");
        crate::turn::end_run(run, &mut cx, ExitTier::Bank);
    }
    g.finish_run();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").map(|s| s.auto_packed).collect::<Vec<_>>(), [true, false], "repeat preserves individual origins, not just kinds");
    let manual = g.lineage.supplies.iter().find(|s| s.kind == "heal" && !s.auto_packed).unwrap().id;
    g.drop_supply(manual).unwrap();
    assert_eq!(g.lineage.last_supplies, ["heal"]);
    assert_eq!(g.lineage.last_supply_origins, [("heal".into(), true)], "dropping the manual duplicate leaves the automatic slot");
    g.clear_supplies();
    assert!(g.lineage.last_supply_origins.is_empty());

    // Only one duplicate was used: the surviving slot is reserved for its own origin
    // before the missing opposite origin is repurchased, in either direction.
    for consumed_automatic in [true, false] {
        let mut partial = Game::new_literal(13);
        partial.lineage.supplies.clear();
        partial.lineage.gold_move(10_000 - partial.lineage.gold, "test");
        if let Some(f) = crate::item::ident_fact(&partial.lineage.flavours, "heal") { partial.lineage.facts.insert(f); }
        for automatic in [true, false] {
            partial.buy_supply("heal").unwrap();
            partial.lineage.supplies.last_mut().unwrap().auto_packed = automatic;
        }
        partial.start_run(None);
        {
            let (run, mut cx) = partial.ctx();
            run.hero.inv.retain(|i| i.kind != "heal" || i.auto_packed != consumed_automatic);
            crate::turn::end_run(run, &mut cx, ExitTier::Bank);
        }
        partial.finish_run();
        let flags: Vec<bool> = partial.lineage.supplies.iter().filter(|s| s.kind == "heal").map(|s| s.auto_packed).collect();
        assert_eq!(flags.len(), 2);
        assert_eq!(flags.iter().filter(|v| **v).count(), 1, "one automatic and one manual slot survive partial consumption: {flags:?}");
        partial = Game::load(&partial.save()).unwrap();
        partial.start_run(None);
        assert_eq!(partial.lineage.last_supply_origins.iter().filter(|(_, auto)| *auto).count(), 1, "the next send keeps the same origins");
    }

    let mut legacy: serde_json::Value = serde_json::from_str(&g.save()).unwrap();
    legacy["lineage"]["last_supplies"] = serde_json::json!(["heal"]);
    legacy["lineage"].as_object_mut().unwrap().remove("last_supply_origins");
    let mut old = Game::load(&legacy.to_string()).unwrap();
    old.restock();
    assert!(old.lineage.supplies.iter().all(|s| !s.auto_packed), "legacy repeat slots remain protected");
}

#[test]
fn learned_drill_yields_to_the_current_stances_heal() {
    let mut g = crate::tests::arena();
    g.lineage.pkg.literal = false;
    g.lineage.pkg.stance = "guarded".into();
    g.lineage.pkg.runs.insert("guarded".into(), 220);
    let learned = packages::drill_rows("goblin_warlord", 35);
    g.lineage.pkg.drills.push(packages::Drill { boss: "goblin_warlord".into(), rows: learned.clone(), revoked: false, announced: false });
    packages::recompile(&mut g.lineage);
    let compiled = g.lineage.rules().clone();
    assert!(compiled.validate().is_ok());
    assert_eq!(g.lineage.pkg.drills[0].rows, learned, "the stored learned rows are preserved");
    assert_eq!(compiled.rows[0].conds.iter().find(|c| c.k == "hp>").unwrap().n, Some(45));
    assert_eq!(g.lineage().packages.drills[0].rows[0].conds.iter().find(|c| c.k == "hp>").unwrap().n, Some(45), "the drill panel shows the effective sustain guard");
    crate::tests::add_monster(&mut g, "goblin_warlord", 5, 5);
    crate::tests::give(&mut g, "heal");
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(f); }
    let r = g.run.as_mut().unwrap(); r.hero.max_hp = 100; r.hero.hp = 40; r.hero.energy = 100;
    let events = crate::tests::ticks(&mut g, 1);
    let row = events.iter().find_map(|e| if let crate::wire::Ev::Rule { row, .. } = e { Some(*row) } else { None }).expect("a rule fires");
    assert_eq!(compiled.rows[row as usize].verb, Verb::arg("drink", "heal"), "the current stance drinks before the old drill counter: {events:?}");
}

#[test]
fn newer_trained_reflection_counter_runs_before_old_generic_boss_attacks() {
    for with_king in [false, true] {
        let mut g = crate::tests::arena();
        g.lineage.pkg.literal = false;
        // These are legacy learned rows: the King still carries the old generic boss condition.
        let mut bosses = vec!["goblin_warlord", "foundry_master"];
        if with_king { bosses.push("mirror_king"); }
        for boss in bosses {
            let row = crate::facts::counter_row(boss);
            g.lineage.pkg.drills.push(packages::Drill { boss: boss.into(), rows: vec![Row::new([row.conds, vec![Cond::n("hp>", 35)]].concat(), row.verb)], revoked: false, announced: false });
        }
        let learned = g.lineage.pkg.drills.clone();
        packages::recompile(&mut g.lineage);
        assert_eq!(g.lineage.pkg.drills, learned, "legacy stored rows are never rewritten");
        assert!(g.lineage.rules().validate().is_ok());
        assert!(g.lineage.rules().rows.iter().all(|r| r.conds.len() <= 2));
        for fact in ["foe:foundry_master", "foe:foundry_master:boss", "foe:foundry_master:reflect_melee"] { g.lineage.facts.insert(fact.into()); }
        g.lineage.unlocks.insert("reflect_read".into());
        crate::tests::add_monster(&mut g, "foundry_master", 5, 5);
        crate::tests::give(&mut g, "bow");
        let r = g.run.as_mut().unwrap(); r.hero.max_hp = 100; r.hero.hp = 100; r.hero.energy = 100;
        let events = crate::tests::ticks(&mut g, 1);
        let fired = events.iter().find_map(|e| if let crate::wire::Ev::Rule { row, .. } = e { Some(*row) } else { None }).expect("the trained counter fires");
        assert_eq!(g.lineage.rules().rows[fired as usize].verb, Verb::arg("tactic", "reflect_read"), "neither Warlord's attack nor King's cadence intercepts the Master: {events:?}");
        assert_eq!(g.run.as_ref().unwrap().hero.weapon_kind(), "bow", "the real counter raises the bow");
        assert!(!events.iter().any(|e| matches!(e, crate::wire::Ev::Attack { verb: Some(v), .. } if v == "reflect")));
    }
}

#[test]
fn generated_counter_scopes_match_the_wire_and_preserve_explicit_player_priority() {
    let mut g = Game::new_resident(3);
    g.lineage.pkg.pen_open = true;
    let player = Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("tactic", "cadence")).from("player");
    g.lineage.pkg.pen.push(player.clone());
    for (boss, tag) in [("bloat_mother", "gas"), ("lich", "undead"), ("lurker_queen", "brood"), ("mirror_king", "mirror")] {
        let canonical = packages::drill_rows(boss, 35);
        let scoped = canonical.iter().find(|r| r.conds.iter().any(|c| c.t.as_deref() == Some(tag))).expect("the copy template scopes the generic boss row");
        assert!(scoped.conds.len() <= 2);
        let generic = Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("hp>", 35)], scoped.verb.clone());
        g.lineage.pkg.drills.push(packages::Drill { boss: boss.into(), rows: vec![generic.clone()], revoked: false, announced: false });
        packages::recompile(&mut g.lineage);
        assert_eq!(g.lineage.rules().rows[0], player, "an explicitly authored generic boss row still wins");
        assert_eq!(g.lineage.pkg.drills.last().unwrap().rows[0], generic, "save bodies retain the original conditions");
        let wire = g.lineage().packages.drills;
        assert!(wire.last().unwrap().rows[0].conds.iter().any(|c| c.t.as_deref() == Some(tag)), "the panel shows the effective scope");
        assert!(g.lineage.rules().validate().is_ok());
    }
    assert_eq!(crate::facts::counter_row("mirror_king").conds[0], Cond::t("foe_tag", "boss"), "counter-fact fingerprints stay stable");
}

#[test]
fn legacy_camp_drills_refresh_before_editing_without_rewriting_live_replay_rows() {
    for custom in [false, true] {
        let mut g = Game::new_resident(12);
        g.lineage.pkg.pen_open = true;
        let player = Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")).from("player");
        let own = Row::new(vec![Cond::n("hp<", 50)], Verb::new("rest"));
        if custom { g.lineage.pkg.stance = packages::CUSTOM.into(); g.lineage.pkg.custom.push(own.clone()); }
        g.lineage.pkg.pen.push(player.clone());
        for boss in ["goblin_warlord", "foundry_master", "mirror_king"] {
            let row = crate::facts::counter_row(boss);
            g.lineage.pkg.drills.push(packages::Drill { boss: boss.into(), rows: vec![Row::new([row.conds, vec![Cond::n("hp>", 35)]].concat(), row.verb)], revoked: false, announced: false });
        }
        for kind in ["goblin_warlord", "foundry_master", "mirror_king"] {
            g.lineage.facts.insert(format!("foe:{kind}"));
            for tag in crate::defs::monster_def(kind).tags { g.lineage.facts.insert(format!("foe:{kind}:{tag}")); }
        }
        g.lineage.unlocks.insert("cadence".into());
        g.lineage.unlocks.insert("reflect_read".into());
        let stored = g.lineage.pkg.drills.clone();
        // The serialized set is the old compiler's learned order and generic King scope.
        let mut legacy = vec![player.clone()];
        for d in &stored { legacy.extend(d.rows.iter().cloned().map(|r| r.from(&format!("drill:{}", d.boss)))); }
        if custom { legacy.push(own.clone().from("stance:custom")); }
        g.lineage.sets[g.lineage.active_set].rows = legacy.clone();
        let mut loaded = Game::load(&g.save()).unwrap();
        assert_eq!(loaded.lineage.pkg.drills, stored);
        assert_eq!(loaded.lineage.pkg.pen.as_slice(), std::slice::from_ref(&player));
        assert_eq!(loaded.lineage.rules().rows[1].origin.as_deref(), Some("drill:mirror_king"));
        assert!(loaded.lineage.rules().rows[1].conds.iter().any(|c| c.t.as_deref() == Some("mirror")));
        let mut edited = loaded.lineage.rules().clone();
        edited.rows.insert(0, Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")).from("player"));
        loaded.set_rules(edited).unwrap();
        if custom {
            assert_eq!(loaded.lineage.pkg.custom.len(), 3, "only the custom set's three authored rows remain");
            assert!(loaded.lineage.pkg.custom.contains(&own));
            assert!(loaded.lineage.pkg.custom.contains(&player));
        } else {
            assert_eq!(loaded.lineage.pkg.pen.len(), 2, "legacy generated rows do not leak into the pen");
            assert!(loaded.lineage.pkg.pen.contains(&player), "the authored generic boss row remains authored");
        }
        // A live replay is immutable across load even when its compiled set predates the fix.
        g.start_run(Some(83));
        g.lineage.sets[g.lineage.active_set].rows = legacy.clone();
        let live = Game::load(&g.save()).unwrap();
        assert_eq!(live.lineage.rules().rows, legacy);
        assert_eq!(live.run.as_ref().unwrap().row_fired, g.run.as_ref().unwrap().row_fired);
        assert_eq!(live.history.len(), g.history.len());
    }
    let literal = Game::new_literal(12);
    assert_eq!(Game::load(&literal.save()).unwrap().lineage.rules(), literal.lineage.rules());
}

#[test]
fn master_secondary_new_and_legacy_drills_yield_to_healing_but_keep_the_buffer_attack() {
    for legacy in [false, true] {
        let mut g = crate::tests::arena();
        for fact in ["foe:foundry_master", "foe:foundry_master:buffer", "foe:foundry_master:reflect_melee", "foe:smith", "foe:smith:buffer"] { g.lineage.facts.insert(fact.into()); }
        g.lineage.unlocks.insert("reflect_read".into());
        g.lineage.pkg.literal = false;
        g.lineage.pkg.stance = "guarded".into();
        g.lineage.pkg.runs.insert("guarded".into(), 220);
        let mut learned = packages::drill_rows("foundry_master", 35);
        if legacy { learned[1].conds = vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 19)]; }
        g.lineage.pkg.drills.push(packages::Drill { boss: "foundry_master".into(), rows: learned.clone(), ..Default::default() });
        packages::recompile(&mut g.lineage);
        assert_eq!(g.lineage.pkg.drills[0].rows, learned, "stored learned rows are not rewritten");
        let wire = packages::wire(&g.lineage);
        let effective = &wire.drills[0].rows[1];
        assert_eq!(effective.conds, vec![Cond::n("depth>=", 19), Cond::n("hp>", 45)]);
        assert_eq!(g.lineage.rules().rows[1].conds, effective.conds, "the wire shows the actual secondary guard");
        assert!(g.lineage.rules().validate().is_ok());
        assert!(g.lineage.rules().rows.iter().all(|r| r.conds.len() <= 2));
        g.run.as_mut().unwrap().depth = 23;
        crate::tests::add_monster(&mut g, "foundry_master", 5, 5);
        crate::tests::give(&mut g, "heal");
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(f); }
        let run = g.run.as_mut().unwrap(); run.hero.max_hp = 100; run.hero.hp = 40; run.hero.energy = 100;
        let mut healthy = g.clone();
        let events = crate::tests::ticks(&mut g, 1);
        let fired = events.iter().find_map(|e| if let crate::wire::Ev::Rule { verb, .. } = e { Some(verb) } else { None }).unwrap();
        assert_eq!(*fired, Verb::arg("drink", "heal"), "both generated Master rows yield to a held heal: {events:?}");

        // With no reflecting foe, a healthy hero still attacks the Foundry's buffer.
        let run = healthy.run.as_mut().unwrap(); run.monsters.clear(); run.hero.hp = 100; run.hero.energy = 100;
        let smith = crate::tests::add_monster(&mut healthy, "smith", 5, 5);
        let events = crate::tests::ticks(&mut healthy, 1);
        assert!(events.iter().any(|e| matches!(e, crate::wire::Ev::Rule { verb, .. } if *verb == Verb::arg("attack", "tag:buffer"))));
        assert!(events.iter().any(|e| matches!(e, crate::wire::Ev::Attack { dst, .. } if *dst == smith)));
    }
}

#[test]
fn an_authored_legacy_buffer_row_keeps_its_priority_above_the_generated_heal() {
    let mut g = crate::tests::arena();
    for fact in ["foe:foundry_master", "foe:foundry_master:buffer", "foe:foundry_master:reflect_melee"] { g.lineage.facts.insert(fact.into()); }
    g.lineage.pkg.literal = false;
    g.lineage.pkg.stance = "guarded".into();
    g.lineage.pkg.runs.insert("guarded".into(), 220);
    g.lineage.pkg.pen_open = true;
    let own = Row::new(vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 19)], Verb::arg("attack", "tag:buffer")).from("player");
    g.lineage.pkg.pen = vec![own.clone()];
    g.lineage.pkg.drills.push(packages::Drill { boss: "foundry_master".into(), rows: packages::drill_rows("foundry_master", 35), ..Default::default() });
    packages::recompile(&mut g.lineage);
    assert_eq!(g.lineage.rules().rows[0], own);
    assert_eq!(g.lineage.pkg.pen.as_slice(), std::slice::from_ref(&own));
    g.run.as_mut().unwrap().depth = 23;
    crate::tests::add_monster(&mut g, "foundry_master", 5, 5);
    crate::tests::give(&mut g, "heal");
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(f); }
    let run = g.run.as_mut().unwrap(); run.hero.max_hp = 100; run.hero.hp = 40; run.hero.energy = 100;
    let events = crate::tests::ticks(&mut g, 1);
    assert!(events.iter().any(|e| matches!(e, crate::wire::Ev::Rule { row: 0, verb, .. } if *verb == own.verb)), "the explicit player row remains above sustain: {events:?}");
    assert!(g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "heal"));
}

fn shared_fire_counter_pack() -> Game {
    let mut g = Game::new_resident(13);
    g.lineage.supplies.clear();
    g.lineage.gold_move(10_000 - g.lineage.gold, "test");
    g.lineage.best_depth = 12;
    g.lineage.pkg.stance = "guarded".into();
    g.lineage.pkg.runs.insert("guarded".into(), 220);
    g.lineage.pkg.tactics.push("boss_focus".into());
    g.lineage.pkg.drills.push(packages::Drill { boss: "bloat_mother".into(), rows: packages::drill_rows("bloat_mother", 45), revoked: false, announced: false });
    for kind in ["heal", "fire"] { if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, kind) { g.lineage.facts.insert(f); } }
    packages::recompile(&mut g.lineage);
    assert_eq!(packages::quartermaster(&g.lineage), ["fire"]);
    assert_eq!(packages::pack_kinds(&g.lineage), ["heal", "fire"]);
    for _ in 0..3 { g.buy_supply("fire").unwrap(); g.lineage.supplies.last_mut().unwrap().auto_packed = true; }
    g
}

#[test]
fn shared_counter_fire_does_not_take_the_stances_second_heal_slot() {
    let mut g = shared_fire_counter_pack();
    let gold = g.lineage.gold;
    let heal_price = g.supply_catalogue().iter().find(|s| s.kind == "heal").unwrap().price;
    g.restock();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 2);
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "fire").count(), 1, "the active counter keeps one slot");
    assert_eq!(gold - g.lineage.gold, 2 * heal_price);
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().hero.inv.iter().filter(|s| s.kind == "heal").count(), 2);
    let r = g.run.as_mut().unwrap(); r.monsters.clear(); r.hero.max_hp = 100; r.hero.hp = 40; r.hero.energy = 100;
    g.tick();
    assert!(g.run.as_ref().unwrap().hero.hp > 40, "the packed sustain actually heals the hero");
    { let (r, mut cx) = g.ctx(); r.hero.inv.retain(|i| !["heal", "fire"].contains(&i.kind.as_str())); crate::turn::end_run(r, &mut cx, ExitTier::Bank); }
    g.finish_run();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 2, "repeat restores the two sustain slots");
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "fire").count(), 1);
}

#[test]
fn shared_counter_surplus_replacement_respects_budget_and_protected_supplies() {
    let mut g = shared_fire_counter_pack();
    g.offline = true;
    let price = g.supply_catalogue().iter().find(|s| s.kind == "heal").unwrap().price;
    let before = g.lineage.supplies.clone(); let gold = g.lineage.gold;
    g.restock(); assert_eq!(g.lineage.supplies, before, "no absence income authorizes no replacement"); assert_eq!(g.lineage.gold, gold);
    g.batch.gold_earned = price;
    g.restock();
    assert_eq!(g.lineage.supplies.iter().filter(|s| s.kind == "heal").count(), 1, "only one heal fits the earned budget");
    assert_eq!(g.batch.spent_total(), price);
    assert!(g.batch.spent_total() <= g.batch.income());
    let mut protected = shared_fire_counter_pack();
    protected.lineage.supplies[0].auto_packed = false;
    protected.lineage.supplies[1].found = true;
    protected.lineage.supplies[2].free = true;
    let held = protected.lineage.supplies.clone(); protected.restock();
    assert_eq!(protected.lineage.supplies, held, "manual, found and free counter supplies keep every slot");
}

#[test]
fn obsolete_automatic_repeat_slots_follow_the_pack_but_manual_duplicates_remain() {
    let mut g = shared_fire_counter_pack();
    g.lineage.supplies.clear();
    g.lineage.last_supplies = vec!["fire".into(); 5];
    g.lineage.last_supply_origins = (0..5).map(|i| ("fire".into(), i < 3)).collect();
    g.buy_supply("fire").unwrap(); // an actual manual duplicate remains protected
    let manual = g.lineage.supplies[0].id;
    g.restock();
    assert!(g.lineage.supplies.iter().any(|s| s.id == manual && !s.auto_packed));
    g.start_run(None);
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && *a).count(), 1, "only the active automatic counter repeats");
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2, "every authored duplicate is remembered");
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "heal" && *a).count(), 2);
    let mut loaded = Game::load(&g.save()).unwrap();
    assert_eq!(loaded.lineage.last_supply_origins, g.lineage.last_supply_origins, "save/load retains exact origins");
    { let (r, mut cx) = loaded.ctx(); r.hero.inv.retain(|i| i.kind != "heal"); crate::turn::end_run(r, &mut cx, ExitTier::Bank); }
    loaded.finish_run();
    loaded.lineage.supplies.clear();
    loaded.lineage.kills.insert("bloat_mother".into()); loaded.lineage.best_depth = 20;
    loaded.lineage.pkg.tactics.clear(); packages::recompile(&mut loaded.lineage);
    loaded.lineage.light_waystones(20);
    loaded.set_start(14).unwrap(); // this send actually skips the Mother's floor
    loaded.start_run(None);
    assert_eq!(loaded.run.as_ref().unwrap().start, 14);
    assert_eq!(loaded.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && *a).count(), 0, "the former package's automatic fire stops repeating");
    assert_eq!(loaded.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2);
}

#[test]
fn lifecycle_pack_caps_leave_literal_custom_and_unknown_origin_quotes_unchanged() {
    for custom in [false, true] {
        let mut g = Game::new_literal(13);
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "fire") { g.lineage.facts.insert(f); }
        g.lineage.gold_move(10_000 - g.lineage.gold, "test");
        g.lineage.supplies.clear();
        let row = Row::new(vec![], Verb::arg("throw", "fire,nearest"));
        g.set_rules_raw(RuleSet { rows: vec![row.clone()], ..Default::default() }).unwrap();
        if custom { g.lineage.pkg.literal = false; g.lineage.pkg.stance = packages::CUSTOM.into(); g.lineage.pkg.custom = vec![row]; packages::recompile(&mut g.lineage); }
        let quote = g.supply_catalogue().iter().find(|s| s.kind == "fire").unwrap().price - 1;
        g.lineage.repeat_quote.insert("fire".into(), quote);
        g.lineage.last_supplies = vec!["fire".into(); 2];
        g.lineage.last_supply_origins = vec![("fire".into(), true); 2];
        let gold = g.lineage.gold;
        g.restock();
        assert_eq!(gold - g.lineage.gold, 2 * quote);
        assert_eq!(g.lineage.supplies.len(), 2, "literal/custom repeats retain their automatic quantities");
        g = Game::load(&g.save()).unwrap();
        let id = g.lineage.supplies[0].id; g.drop_supply(id).unwrap();
        assert_eq!(g.lineage.last_supplies, ["fire"]);
        assert_eq!(g.lineage.last_supply_origins, [("fire".into(), true)]);
    }
    let mut g = shared_fire_counter_pack();
    g.lineage.supplies.clear();
    g.lineage.last_supplies = vec!["fire".into(); 2];
    g.lineage.last_supply_origins.clear(); // old saves lack provenance: never invent automatic ownership
    g.buy_supply("fire").unwrap(); g.buy_supply("fire").unwrap(); // surviving legacy/manual slots
    g.start_run(None);
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2);
}


#[test]
fn empty_legacy_repeat_refills_keep_authored_ownership_across_two_sends_and_package_changes() {
    let mut g = shared_fire_counter_pack();
    g.lineage.supplies.clear();
    g.lineage.last_supplies = vec!["fire".into(); 2];
    g.lineage.last_supply_origins.clear();
    for send in 0..2 {
        g.start_run(None);
        assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2, "both legacy purchases stay authored at send{send}");
        assert!(g.run.as_ref().unwrap().hero.inv.iter().filter(|i| i.kind == "fire").all(|i| !i.auto_packed), "a package refill matched to the authored order is owned by the player");
        g = Game::load(&g.save()).unwrap();
        assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2);
        { let (r, mut cx) = g.ctx(); r.hero.inv.retain(|i| i.kind != "fire"); crate::turn::end_run(r, &mut cx, ExitTier::Bank); }
        g.finish_run();
        g.lineage.kills.insert("bloat_mother".into()); g.lineage.best_depth = 20;
        g.lineage.pkg.tactics.clear(); packages::recompile(&mut g.lineage);
    }
    g.start_run(None);
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 2, "the authored quantity survives the old counter becoming obsolete");
    { let (r, mut cx) = g.ctx(); crate::turn::end_run(r, &mut cx, ExitTier::Bank); }
    g.finish_run();
    g.lineage.supplies.clear(); g.buy_supply("fire").unwrap();
    let id = g.lineage.supplies.last().unwrap().id;
    g.drop_supply(id).unwrap();
    assert_eq!(g.lineage.last_supply_origins.iter().filter(|(k,a)| k == "fire" && !a).count(), 1, "only an explicit player drop reduces the authored repeat");
}

fn projection_arena() -> Game {
    let mut g = crate::tests::arena();
    g.lineage.pkg.literal = false;
    g.lineage.pkg.stance = "guarded".into();
    g.lineage.pkg.runs.insert("guarded".into(), 220);
    g.lineage.pkg.pen_open = true;
    g.lineage.pkg.pen.clear();
    packages::recompile(&mut g.lineage);
    crate::tests::add_monster(&mut g, "jackal", 5, 5);
    crate::tests::give(&mut g, "heal");
    crate::tests::give(&mut g, "silence");
    if let Some(fact) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(fact); }
    let run = g.run.as_mut().unwrap();
    run.hero.max_hp = 100;
    run.hero.hp = 40;
    run.hero.energy = 100;
    g
}

#[test]
fn prospective_projection_matches_public_unknown_scroll_priority_and_held_heal() {
    let g = projection_arena();
    let mut edit = g.lineage.rules().clone();
    let heal = edit.rows.iter().position(|r| r.verb == Verb::arg("drink", "heal")).unwrap();
    edit.rows.insert(heal + 1, Row::new(vec![Cond::n("hp<", 50)], Verb::arg("read", "unknown")));
    let old = g.lineage.clone();
    let mut projected = crate::forecast::edited_game(&g, &edit);
    let mut public = g.clone();
    public.set_rules(edit.clone()).unwrap();
    assert_eq!(projected.lineage.pkg, public.lineage.pkg);
    assert_eq!(projected.lineage.rules().rows, public.lineage.rules().rows);
    assert_eq!(projected.lineage.rules().rows[0].verb, Verb::arg("read", "unknown"));
    assert_eq!(g.lineage, old, "projection never edits the live lineage");
    for h in [&mut projected, &mut public] {
        let events = crate::tests::ticks(h, 1);
        let fired = events.iter().find_map(|e| if let crate::wire::Ev::Rule { verb, .. } = e { Some(verb) } else { None }).unwrap();
        assert_eq!(*fired, Verb::arg("read", "unknown"), "both paths model the authored pen above package healing");
        assert!(h.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "heal"));
    }
    // A raw historical panel still plays the archived placement literally.
    let raw = crate::forecast::sim_game(&g, &edit, 33, 0, None);
    assert_eq!(raw.lineage.rules().rows, edit.rows);
    assert_eq!(g.lineage.rules().rows, old.rules().rows);
}

#[test]
fn prospective_projection_restores_generated_moves_and_wall_edits_are_real_public_sets() {
    let g = projection_arena();
    let before = g.lineage.rules().clone();
    let mut moved = before.clone();
    let index = moved.rows.len() - 1;
    let row = moved.rows.remove(index);
    moved.rows.insert(0, row);
    let projected = packages::project_edit(&g.lineage, &moved);
    assert_eq!(projected.rules().rows, before.rows, "moving an untouched generated row changes no public policy");
    let mut public = g.clone();
    public.set_rules(moved).unwrap();
    assert_eq!(public.lineage.rules().rows, projected.rules().rows);
    for (_, edit) in crate::wall::edits(&g, &before, 8) {
        let mut applied = g.clone();
        applied.set_rules(edit.clone()).unwrap();
        assert_eq!(edit.rows, applied.lineage.rules().rows, "offered wall rows are already public compositions");
        assert_ne!(edit.rows, before.rows);
    }
}

#[test]
fn prospective_projection_preserves_literal_and_custom_rules_without_rewriting_archives() {
    let mut g = projection_arena();
    let archived = g.run.clone();
    let mut edit = g.lineage.rules().clone();
    edit.rows.insert(0, Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")));
    g.lineage.pkg.literal = true;
    assert_eq!(packages::project_edit(&g.lineage, &edit).rules().rows, edit.rows);
    g.lineage.pkg.literal = false;
    g.lineage.pkg.stance = packages::CUSTOM.into();
    g.lineage.pkg.custom = vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))];
    packages::recompile(&mut g.lineage);
    let mut custom = g.lineage.rules().clone();
    custom.rows.insert(0, Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal")));
    let projected = packages::project_edit(&g.lineage, &custom);
    let mut public = g.clone();
    public.set_rules(custom).unwrap();
    assert_eq!(projected.pkg, public.lineage.pkg);
    assert_eq!(projected.rules().rows, public.lineage.rules().rows);
    assert_eq!(g.run, archived, "projecting a future camp policy does not rewrite the live run");
}

#[test]
fn prospective_projection_updates_counter_supply_requests_and_cache_dependencies() {
    let mut g = Game::new_resident(55);
    g.lineage.best_depth = 28;
    g.lineage.pkg.pen_open = true;
    g.lineage.gold = 10_000;
    g.lineage.facts.insert("foe:lurker_queen".into());
    g.lineage.facts.insert("foe:lurker_queen:blind".into());
    if let Some(fact) = crate::item::ident_fact(&g.lineage.flavours, "silence") { g.lineage.facts.insert(fact); }
    if let Some(fact) = crate::item::ident_fact(&g.lineage.flavours, "heal") { g.lineage.facts.insert(fact); }
    packages::recompile(&mut g.lineage);
    let before = crate::forecast::lineage_key(&g);
    let mut edit = g.lineage.rules().clone();
    edit.rows.insert(0, Row::new(vec![Cond::t("foe_tag", "blind")], Verb::arg("read", "silence")));
    let projected = crate::forecast::edited_game(&g, &edit);
    assert!(packages::quartermaster(&projected.lineage).contains(&"silence".into()));
    assert!(!packages::quartermaster(&g.lineage).contains(&"silence".into()));
    assert!(packages::pack_kinds(&projected.lineage).contains(&"heal".into()));
    assert_ne!(crate::forecast::lineage_key(&projected), before, "the next-send counter pack is a simulation input");
    let mut public = g.clone();
    public.set_rules(edit).unwrap();
    assert_eq!(crate::forecast::lineage_key(&projected), crate::forecast::lineage_key(&public));
    let mut projected = projected;
    projected.start_run(Some(55));
    public.start_run(Some(55));
    for game in [&projected, &public] {
        let pack = &game.run.as_ref().unwrap().hero.inv;
        assert!(pack.iter().any(|item| item.kind == "silence"), "the prospective counter is actually packed");
        assert!(pack.iter().any(|item| item.kind == "heal"), "the sustain pack remains available");
    }
}

#[test]
fn prospective_projection_death_deltas_measure_current_indices_and_recache_after_edits() {
    // An actual archived death supplies the replay state; current camp rules can then change.
    let mut g = Game::new_literal(1047);
    g.send();
    let mut id = None;
    for _ in 0..4000 {
        let step = g.step(50);
        if step.run_over {
            id = step.events.iter().any(|e| matches!(e, crate::wire::Ev::Exit { tier, .. } if tier == "death")).then_some(step.snapshot.run.id);
            break;
        }
    }
    let id = id.expect("the first heir dies");
    g.keep(vec![]).unwrap();
    let recorded = g.deaths[&id].rules.clone();
    let archived = g.deaths[&id].t10.clone();
    let current = RuleSet { rows: vec![Row::new(vec![], Verb::new("bank")), Row::new(vec![], Verb::new("return"))], ..Default::default() };
    g.set_rules(current.clone()).unwrap();
    let patch: crate::wire::Patch = serde_json::from_value(serde_json::json!({
        "row": recorded.rows[0], "insert_at": 0, "moves_from": 1,
        "survive": 1.0, "forecast_delta": 0.0, "camp_pending": true
    })).unwrap();
    {
        let rec = g.deaths.get_mut(&id).unwrap();
        rec.verdict_done = true;
        rec.deltas_done = true;
        rec.shaped = true;
        rec.death.patches = vec![patch.clone()];
        rec.camp_key = 0;
    }
    let edited = crate::offline::apply_patch(&current, &patch, g.lineage.max_rows());
    let projected = crate::forecast::edited_game(&g, &edited);
    let mut public = g.clone();
    public.set_rules(edited).unwrap();
    assert_eq!(projected.lineage.rules().rows, public.lineage.rules().rows);
    assert_eq!(projected.lineage.rules().rows[0].verb.v, "return", "current-index move uses today's row, not the archived row echoed in the patch");
    let measured = g.death_deltas(id).unwrap();
    assert_eq!(measured.len(), 1);
    assert!(!measured[0].camp_pending);
    assert_eq!(measured[0].whole.as_ref().unwrap().death_from, 0.0, "the current bank policy supplies the camp baseline");
    let first_key = g.deaths[&id].camp_key;
    let before = crate::forecast::lineage_key(&g);
    let mut next = current;
    next.rows[1] = Row::new(vec![Cond::n("hp<", 50)], Verb::new("return"));
    g.set_rules(next).unwrap();
    assert_eq!(before, crate::forecast::lineage_key(&g), "this row edit does not alter any next-send supplies or state");
    g.death_deltas(id).unwrap();
    assert_ne!(g.deaths[&id].camp_key, first_key, "active rules invalidate prospective patch measures independently of state");
    assert_eq!(g.deaths[&id].rules.rows, recorded.rows, "historical survival still reads the archived set");
    assert_eq!(g.deaths[&id].t10, archived, "the saved fight remains unchanged");
}

#[test]
fn master_secondary_keeps_its_deep_scope_for_new_and_both_legacy_templates() {
    for template in 0..3 {
        let mut base = crate::tests::arena();
        for fact in ["foe:goblin_warlord", "foe:goblin_warlord:boss", "foe:goblin_warlord:buffer", "foe:smith", "foe:smith:buffer", "foe:goblin"] {
            base.lineage.facts.insert(fact.into());
        }
        base.lineage.pkg.literal = false;
        base.lineage.pkg.stance = "guarded".into();
        base.lineage.pkg.runs.insert("guarded".into(), 220);
        let mut learned = packages::drill_rows("foundry_master", 35);
        match template {
            1 => learned[1].conds = vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 19)],
            2 => learned[1].conds = vec![Cond::t("foe_tag", "buffer"), Cond::n("hp>", 35)],
            _ => {},
        }
        base.lineage.pkg.drills.push(packages::Drill { boss: "foundry_master".into(), rows: learned.clone(), ..Default::default() });
        packages::recompile(&mut base.lineage);
        assert_eq!(base.lineage.pkg.drills[0].rows, learned, "saved generated templates remain intact");
        let effective = &packages::wire(&base.lineage).drills[0].rows[1];
        assert_eq!(effective.conds, vec![Cond::n("depth>=", 19), Cond::n("hp>", 45)]);
        assert_eq!(effective.conds, base.lineage.rules().rows[1].conds);
        assert!(base.lineage.rules().validate().is_ok());
        assert_eq!(Game::load(&base.save()).unwrap().lineage.pkg.drills[0].rows, learned);

        for (depth, kind, hp, should_fire) in [(18, "goblin_warlord", 100, false), (19, "smith", 100, true), (23, "smith", 40, false), (23, "goblin", 100, false)] {
            let mut g = base.clone();
            g.run.as_mut().unwrap().depth = depth;
            crate::tests::add_monster(&mut g, kind, 5, 5);
            crate::tests::give(&mut g, "heal");
            g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "heal").unwrap());
            let run = g.run.as_mut().unwrap();
            run.hero.max_hp = 100; run.hero.hp = hp; run.hero.energy = 100;
            let events = crate::tests::ticks(&mut g, 1);
            let (fired, verb) = events.iter().find_map(|e| if let crate::wire::Ev::Rule { row, verb, .. } = e { Some((*row, verb)) } else { None }).expect("another policy row acts when secondary is unavailable");
            assert_eq!(fired == 1, should_fire, "template {template}, D{depth}, {kind}, hp {hp}: {events:?}");
            if should_fire { assert_eq!(*verb, Verb::arg("attack", "tag:buffer")); }
            if hp == 40 { assert_eq!(*verb, Verb::arg("drink", "heal"), "Master secondary must yield to sustain"); }
        }
    }
}

#[test]
fn quartermaster_keeps_the_queens_counter_on_a_route_below_the_record() {
    for pen in [false, true] {
        let mut g = Game::new_resident(55);
        g.lineage.best_depth = 33;
        g.lineage.gold = 100_000;
        g.lineage.light_waystones(33);
        g.set_start(24).unwrap();
        for boss in ["goblin_warlord", "bloat_mother", "lich", "foundry_master", "lurker_queen"] {
            g.lineage.kills.insert(boss.into());
        }
        for kind in ["silence", "heal"] {
            g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, kind).unwrap());
        }
        if pen {
            g.lineage.pkg.pen_open = true;
            g.lineage.pkg.pen.push(Row::new(vec![Cond::t("foe_tag", "blind")], Verb::arg("read", "silence")));
        } else {
            g.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows: packages::drill_rows("lurker_queen", 30), revoked: false, announced: false });
        }
        packages::recompile(&mut g.lineage);
        let saved = g.save();
        let mut loaded = Game::load(&saved).unwrap();
        loaded.restock();
        loaded.start_run(Some(55));
        let run = loaded.run.as_ref().unwrap();
        assert_eq!(run.start, 24);
        assert!(run.hero.inv.iter().any(|item| item.kind == "silence"), "actual D24 send still crosses D28; pen={pen}");
        assert!(run.hero.inv.iter().any(|item| item.kind == "heal"), "the counter leaves sustain slots; pen={pen}");
    }
}

#[test]
fn quartermaster_retires_the_queens_counter_only_when_the_send_skips_her() {
    let mut g = Game::new_resident(55);
    g.lineage.best_depth = 33;
    g.lineage.light_waystones(33);
    g.set_start(29).unwrap();
    g.lineage.kills.insert("lurker_queen".into());
    g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "silence").unwrap());
    g.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows: packages::drill_rows("lurker_queen", 30), revoked: false, announced: false });
    packages::recompile(&mut g.lineage);
    assert!(packages::quartermaster(&g.lineage).is_empty(), "D29 skips the Queen");
    g.lineage.start = 24;
    assert_eq!(packages::quartermaster(&g.lineage), ["silence"], "choosing D24 restores its counter even after the record passes D28");
    g.lineage.pkg.drills[0].revoked = true;
    assert!(packages::quartermaster(&g.lineage).is_empty(), "revoked policy stays revoked");
}


#[test]
fn queens_generated_counter_keeps_silence_for_the_boss_instead_of_ordinary_lurkers() {
    for legacy in [false, true] {
        let mut g = crate::tests::arena();
        g.lineage.pkg.literal = false;
        let rows = if legacy {
            vec![Row::new(vec![Cond::t("foe_tag", "blind"), Cond::n("hp>", 35)], Verb::arg("read", "silence"))]
        } else { vec![crate::facts::counter_row("lurker_queen")] };
        let saved_rows = rows.clone();
        g.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows, revoked: false, announced: false });
        packages::recompile(&mut g.lineage);
        crate::tests::give(&mut g, "silence");
        g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "silence").unwrap());
        g.lineage.facts.insert("foe:lurker:blind".into());
        crate::tests::add_monster(&mut g, "lurker", 5, 5);
        g.run.as_mut().unwrap().hero.energy = 100;
        let events = crate::tests::ticks(&mut g, 1);
        assert!(g.run.as_ref().unwrap().hero.inv.iter().any(|i| i.kind == "silence"), "ordinary lurker must not consume the Queen's generated counter; legacy={legacy}, events={events:?}");
        assert!(!events.iter().any(|e| matches!(e, crate::wire::Ev::Use { item, .. } if item == "silence scroll")));
        assert_eq!(g.lineage.pkg.drills[0].rows, saved_rows, "saved drill bodies remain intact");
    }
}

#[test]
fn queens_generated_counter_is_learned_in_sight_and_yields_to_healing() {
    for hurt in [false, true] {
        let mut g = crate::tests::arena();
        g.lineage.pkg.literal = false;
        g.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows: packages::drill_rows("lurker_queen", 35), revoked: false, announced: false });
        packages::recompile(&mut g.lineage);
        for kind in ["silence", "heal"] {
            crate::tests::give(&mut g, kind);
            g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, kind).unwrap());
        }
        crate::tests::add_monster(&mut g, "lurker_queen", 5, 5);
        let run = g.run.as_mut().unwrap(); run.hero.max_hp = 100; run.hero.hp = if hurt { 20 } else { 100 }; run.hero.energy = 100;
        assert!(!g.lineage.facts.contains("foe:lurker_queen:brood"));
        let mut events = crate::tests::ticks(&mut g, 1);
        assert!(g.lineage.facts.contains("foe:lurker_queen:brood"), "Queen sight teaches the scope");
        assert!(g.vocabulary().conds.contains(&Cond::t("foe_tag", "brood")));
        // Vision learns a newly met foe after the first action. Use the next actual action.
        g.run.as_mut().unwrap().hero.energy = 100;
        events.extend(crate::tests::ticks(&mut g, 1));
        let kind = if hurt { "heal potion" } else { "silence scroll" };
        assert!(events.iter().any(|e| matches!(e, crate::wire::Ev::Use { item, .. } if item == kind)), "the real Queen counter must yield to healing: {events:?}");
        assert!(g.lineage.rules().validate().is_ok());
        assert!(g.lineage.rules().rows.iter().all(|r| r.conds.len() <= 2));
    }
}


#[test]
fn queens_counter_scope_preserves_an_explicit_blind_pen_row() {
    let mut g = crate::tests::arena();
    g.lineage.pkg.literal = false;
    g.lineage.pkg.pen_open = true;
    let authored = Row::new(vec![Cond::t("foe_tag", "blind")], Verb::arg("read", "silence")).from("player");
    g.lineage.pkg.pen.push(authored.clone());
    g.lineage.pkg.drills.push(packages::Drill { boss: "lurker_queen".into(), rows: packages::drill_rows("lurker_queen", 35), revoked: false, announced: false });
    packages::recompile(&mut g.lineage);
    g.lineage.facts.insert("foe:lurker:blind".into());
    g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "silence").unwrap());
    crate::tests::give(&mut g, "silence");
    crate::tests::add_monster(&mut g, "lurker", 5, 5);
    g.run.as_mut().unwrap().hero.energy = 100;
    let events = crate::tests::ticks(&mut g, 1);
    assert_eq!(g.lineage.pkg.pen[0], authored);
    assert_eq!(g.lineage.rules().rows[0], authored);
    assert!(events.iter().any(|e| matches!(e, crate::wire::Ev::Use { item, .. } if item == "silence scroll")), "the explicit policy still reads at ordinary lurkers");
    assert_eq!(crate::facts::counter_row("lurker_queen").conds[0], Cond::t("foe_tag", "boss"), "learned counter fingerprints remain stable");
}

#[test]
fn buildings_require_a_manual_action_and_survive_save_load() {
    let mut g = Game::new_resident(21);
    assert!(g.lineage.town.built.is_empty());
    assert!(g.build_town("blacksmith").is_err());
    g.lineage.banked_depths.insert(1);
    assert!(crate::town::update(&mut g.lineage).is_empty());
    assert!(!crate::town::built(&g.lineage, "blacksmith"));
    assert!(g.lineage().town.next_ready);
    g.build_town("blacksmith").unwrap();
    assert!(g.build_town("blacksmith").is_err());
    let restored = Game::load(&g.save()).unwrap();
    assert!(crate::town::built(&restored.lineage, "blacksmith"));
    assert!(restored.lineage.town.manual);
}

#[test]
fn equivalent_package_prices_keep_each_slot_and_purchase() {
    let mut g = Game::new_resident(7);
    g.lineage.pkg.owned.insert("boss_focus".into());
    g.lineage.pkg.meets.insert("lich".into(), 1);
    g.lineage.marks = 10;
    packages::wear(&mut g.lineage, Some("unbowed"));
    packages::recompile(&mut g.lineage);
    static WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
    crate::forecast::with_sim_width(&WIDTH, || {
        let before = g.save();
        let options = packages::options(&g, 2);
        let slots: Vec<_> = options.iter().filter(|o| o.id == "boss_focus").collect();
        assert_eq!(slots.len(), 2, "both empty tactic slots remain choices");
        let mut first = slots[0].clone();
        first.slot = slots[1].slot;
        assert_eq!(&first, slots[1], "equivalent slots retain identical prices");
        let level = options.iter().find(|o| o.id == "unbowed" && o.action == "level").unwrap();
        assert_eq!(level.price, 2, "a forecast-neutral level still has its own purchase price");
        assert_eq!((level.d_past, level.d_bank, level.d_death, level.d_reach, level.d_mean, level.d_wall), (0.0, 0.0, 0.0, 0.0, 0.0, 0.0));
        assert_eq!(g.save(), before, "pricing must not spend or equip");
        assert_eq!(packages::options(&g, 2), options, "a warm base cache preserves every choice and its order");
    });
}

#[test]
fn package_query_key_tracks_prices_and_simulation_inputs() {
    let g = Game::new_resident(7);
    let saved = g.save();
    let key = packages::options_key(&g, 24);
    assert_eq!(g.save(), saved, "key calculation is read-only");
    let mut c = g.sim_clone();
    c.set_look("cat").unwrap();
    c.lineage.rest_left += 10;
    c.lineage.bloodline.as_mut().unwrap().points += 100;
    c.lineage.sets[0].name = Some("renamed".into());
    assert_eq!(packages::options_key(&c, 24), key, "cosmetics, clock and unspent Legacy do not change prices");
    assert_eq!(packages::options(&g, 2), packages::options(&c, 2));
    assert_ne!(packages::options_key(&g, 25), key, "simulation count belongs to the query");
    let changes: [fn(&mut Game); 8] = [
        |c| c.lineage.seed += 1,
        |c| c.lineage.heir += 1,
        |c| c.lineage.gold += 1,
        |c| c.lineage.best_depth += 1,
        |c| { c.lineage.bloodline.as_mut().unwrap().upgrades.insert("health".into(), 1); },
        |c| { c.lineage.pkg.owned.insert("guarded".into()); },
        |c| c.lineage.marks = 100,
        |c| c.lineage.sets[0].rows.clear(),
    ];
    for (i, change) in changes.into_iter().enumerate() {
        let mut c = g.sim_clone(); change(&mut c);
        assert_ne!(packages::options_key(&c, 24), key, "changed query input {i}");
    }
    let mut c = g.sim_clone(); c.loadout.push(123);
    assert_ne!(packages::options_key(&c, 24), key, "loadout belongs to the query");
}

#[test]
fn selected_package_prices_match_full_choices() {
    for seed in [7, 21] {
        let mut g = Game::new_resident(seed);
        for id in ["guarded", "boss_focus", "pack_break"] { g.lineage.pkg.owned.insert(id.into()); }
        g.lineage.pkg.meets.insert("lich".into(), 1);
        g.lineage.marks = 10;
        packages::wear(&mut g.lineage, Some("unbowed"));
        packages::recompile(&mut g.lineage);
        let saved = g.save();
        let full = packages::options(&g, 8);
        for choices in [vec![("guarded".into(), 0)], vec![("boss_focus".into(), 1)], vec![("guarded".into(), 0), ("pack_break".into(), 0)]] {
            let fresh = Game::load(&saved).unwrap();
            let expected: Vec<_> = full.iter().filter(|o| o.action == "equip" && choices.contains(&(o.id.clone(), o.slot))).cloned().collect();
            assert_eq!(packages::options_for(&fresh, 8, &choices), expected);
            assert_eq!(fresh.save(), saved);
            let key = packages::options_for_key(&fresh, 8, &choices);
            let mut duplicate = choices.clone(); duplicate.extend(choices.clone()); duplicate.reverse();
            assert_eq!(packages::options_for_key(&fresh, 8, &duplicate), key);
        }
        let empty = Game::load(&saved).unwrap();
        assert!(packages::options_for(&empty, 8, &[]).is_empty());
        assert!(empty.panel_cache.borrow().is_empty(), "empty selection must run no panels");
        assert!(packages::options_for(&empty, 8, &[("invalid".into(), 0)]).is_empty());
        assert_eq!(empty.save(), saved);
    }
}

#[test]
fn automatic_pack_reports_away_budget_limit_without_changing_purchases() {
    let mut g=Game::new_resident(17);
    crate::tree::grant(&mut g.lineage,&crate::tree::LEGACY);
    g.lineage.supplies.clear();g.lineage.gold=1000;g.lineage.restock_off=true;
    g.lineage.forge.entry("heal".into()).or_default().craftable=true;
    if let Some(f)=crate::item::ident_fact(&g.lineage.flavours,"heal") {g.lineage.facts.insert(f); }
    g.offline=true;g.batch=Default::default();
    assert!(g.restock().is_empty());assert!(g.batch.restock_capped);
    assert_eq!(g.lineage.gold,1000);assert!(g.lineage.supplies.is_empty());
    g.batch=Default::default();g.batch.gold_earned=1000;
    assert!(!g.restock().is_empty());assert!(!g.batch.restock_capped);
    let price=g.batch.spent_total();assert!(price>0);assert_eq!(g.lineage.gold,1000-price);
    g.lineage.supplies.clear();g.batch=Default::default();g.offline=false;
    assert!(!g.restock().is_empty());assert!(!g.batch.restock_capped);
}
