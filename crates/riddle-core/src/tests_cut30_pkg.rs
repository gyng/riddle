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
    let g = Game::new(3);
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
    let mut g = Game::new(5);
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
    let mut g = Game::new(7);
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
    let mut h = Game::new(8);
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
    let mut g = Game::new(9);
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
    let mut g = Game::new(9);
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
    let mut g = Game::new(11);
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
    let mut g = Game::new(13);
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
    let mut g = Game::new(17);
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
        let mut g = Game::new(seed);
    crate::tree::grant(&mut g.lineage, &crate::tree::LEGACY);
        let r = g.run_offline(20 * 60);
        assert!(r.runs >= 1, "seed {seed}: {} runs", r.runs);
    }
}

/// §3: the bank takes deposits once built, caps them, pays a night's interest (never negative).
#[test]
fn the_bank_is_capped_and_pays_interest() {
    let mut g = Game::new(19);
    g.lineage.gold = 1000;
    assert!(g.bank_deposit(100).is_err(), "no bank yet");
    g.lineage.last_night_net = 200;
    crate::town::update(&mut g.lineage);
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
    let mut g = Game::new(23);
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
    let mut g = Game::new(29);
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
    let mut g = Game::new(23);
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
    let mut g = Game::new(3101);
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
    let mut g = Game::new(3101);
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
