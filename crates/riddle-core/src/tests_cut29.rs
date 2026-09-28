//! Cut 29: the curve keeps opening; a tap is a decision (docs/CUT29.md).
use crate::engine::Game;
use crate::rules::{Cond, Row, Verb};
use crate::wire::Ev;

/// §2: a fresh lineage opens the day-0 systems only; each trigger opens its system (sticky), the
/// wire lists every system with its trigger and the new ones glint once; a bot's `open_all` opens all.
#[test]
fn systems_open_one_at_a_time() {
    let mut g = Game::new(3);
    let open = |g: &Game| g.lineage().systems.iter().filter(|s| s.open).map(|s| s.id.clone()).collect::<Vec<_>>();
    assert_eq!(open(&g), vec!["send", "dial", "headline"]);
    let w = g.lineage();
    assert_eq!(w.systems.len(), crate::systems::SYSTEMS.len());
    assert!(w.systems.iter().all(|s| s.open || !s.trigger.is_empty()), "every closed system says what opens it");
    assert!(w.systems.iter().all(|s| !s.new), "day 0 does not glint");
    // the first death opens the editor and the death screen
    g.lineage.graveyard.push(crate::wire::Grave { heir: 1, depth: 3, cause: "rat".into(), deeds: Vec::new(), death_id: None });
    let opened = crate::systems::update(&mut g.lineage, false);
    assert_eq!(opened, vec!["edit", "death"]);
    assert!(g.lineage().systems.iter().any(|s| s.id == "edit" && s.open && s.new));
    g.seen_systems();
    assert!(g.lineage().systems.iter().all(|s| !s.new));
    // the first plateau opens the order and the vs line (and the oaths); the Warlord met, the walls
    let opened = crate::systems::update(&mut g.lineage, true);
    assert!(opened.contains(&"reorder".to_string()) && opened.contains(&"vs".to_string()) && opened.contains(&"oaths".to_string()), "{opened:?}");
    g.lineage.best_depth = 8;
    let opened = crate::systems::update(&mut g.lineage, false);
    assert!(opened.contains(&"walls".to_string()) && opened.contains(&"divergence".to_string()) && !opened.contains(&"forge".to_string()), "{opened:?}");
    // sticky: a system stays open
    g.lineage.best_depth = 2;
    crate::systems::update(&mut g.lineage, false);
    assert!(crate::systems::is_open(&g.lineage, "walls"));
    // the bots: every system
    let mut b = Game::new(4);
    crate::systems::open_all(&mut b.lineage);
    assert!(b.lineage().systems.iter().all(|s| s.open && !s.new));
    // an old save (no system recorded) opens what it has used
    let mut old = Game::new(5);
    old.lineage.systems.clear();
    old.lineage.kills.insert("goblin_warlord".into());
    old.lineage.best_depth = 9;
    let back = Game::load(&old.save()).unwrap();
    assert!(crate::systems::is_open(&back.lineage, "forge") && crate::systems::is_open(&back.lineage, "walls"));
    assert!(back.lineage.systems_new.is_empty(), "an old save does not glint what it already used");
}

/// §3: a watched run's meter is the event stream's sums — damage by side, healing, fires,
/// supplies, gold — and its time split covers its ticks; the death screen carries the fight.
#[test]
fn meters_are_the_event_streams_sums() {
    let mut g = Game::new(8);
    g.max_deaths = 100;
    g.send();
    let mut evs: Vec<Ev> = Vec::new();
    let mut line = None;
    for _ in 0..4000 {
        let r = g.step(50);
        for e in &r.events {
            if let Ev::Exit { line: Some(l), .. } = e {
                line = Some((**l).clone());
            }
        }
        if let Some(m) = r.snapshot.meters.as_ref() {
            assert!(m.run.seconds >= 0.0);
        }
        evs.extend(r.events);
        if r.run_over {
            break;
        }
    }
    let m = line.expect("an exit line").meters.expect("the run's meter");
    let (dealt, taken, healed, fires, used, gold) = crate::meters::stream_sums(&evs);
    assert_eq!(m.dealt.total(), dealt);
    assert_eq!(m.taken.total(), taken);
    assert_eq!(m.healed.iter().map(|h| h.total).sum::<i64>(), healed);
    assert_eq!(m.rows.iter().map(|r| r.fires).sum::<u32>(), fires);
    assert_eq!(m.supplies.values().sum::<u32>(), used);
    assert_eq!(m.gold, gold);
    assert_eq!((m.time.fight + m.time.travel + m.time.chores + m.time.rest) as f64, m.seconds * 10.0);
    assert!(dealt > 0 && taken > 0, "a run with fights");
    assert!(evs.iter().all(|e| !matches!(e, Ev::Heal { .. }) || !e.renderable()));
    // the lineage keeps the last two runs' meters, and the night's
    assert_eq!(g.lineage().meters.runs.len(), 1);
    // a sim carries none (the meters read the watched stream; the forecast is untouched)
    let mut s = g.sim_clone();
    s.start_run(None);
    s.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
    assert!(s.run.as_ref().unwrap().meters.is_empty());
}

/// §1: oath slots, draws and the day's lapse — a second slot swears beside the first, a draw
/// (◆2, from T2) replaces an unsworn standing oath, a new day lapses the sworn ones unkept.
#[test]
fn oath_slots_draws_and_the_days_lapse() {
    let mut g = Game::new(9);
    g.lineage.best_depth = 9;
    g.lineage.banked_depths.insert(9);
    g.lineage.oath_open = true;
    g.lineage.gold = 100_000;
    crate::oath::refresh(&mut g.lineage);
    assert_eq!(crate::oath::slots(&g.lineage), 1);
    let ids: Vec<String> = g.lineage.oaths.iter().map(|o| o.id.clone()).collect();
    g.swear_oath(&ids[0]).unwrap();
    // one slot: another oath forswears the first
    g.swear_oath(&ids[1]).unwrap();
    assert_eq!(crate::oath::sworn_ids(&g.lineage), vec![ids[1].clone()]);
    // a second slot swears beside it
    g.lineage.unlocks.insert("oath_slot_2".into());
    g.swear_oath(&ids[0]).unwrap();
    assert_eq!(crate::oath::sworn_ids(&g.lineage).len(), 2);
    assert_eq!(g.lineage().sworn.len(), 2);
    assert_eq!(g.lineage().oaths.iter().filter(|o| o.sworn).count(), 2);
    // a draw: ◆2, a fresh oath where an unsworn one stood
    assert!(g.draw_oath().is_err(), "no marks");
    g.lineage.marks = 5;
    let before: Vec<String> = g.lineage.oaths.iter().map(|o| o.id.clone()).collect();
    let id = g.draw_oath().unwrap();
    assert_eq!(g.lineage.marks, 3);
    assert!(!before.contains(&id));
    assert_eq!(crate::oath::sworn_ids(&g.lineage).len(), 2, "a draw never replaces a sworn oath");
    // the next day: both lapse unkept, no refund
    let gold = g.lineage.gold;
    crate::oath::new_day(&mut g.lineage, 1);
    assert!(crate::oath::sworn_ids(&g.lineage).is_empty());
    assert_eq!(g.lineage.gold, gold);
    // a lineage short of T2 draws nothing
    let mut f = Game::new(10);
    f.lineage.marks = 9;
    assert_eq!(f.draw_oath().unwrap_err(), "needs meet Warlord");
}

/// §1: the oaths' reward pool is their own — `bold` gives the verb `hold` (then the D9 route),
/// `lean` gas step at T3, `tamer` the party slots; the verb joins the vocabulary; a spent pool
/// gives a title. The price is a quarter of the last day's net once a day has closed.
#[test]
fn the_oath_pool_is_its_own() {
    let mut g = Game::new(11);
    g.lineage.best_depth = 9;
    g.lineage.banked_depths.insert(9);
    let bold = crate::oath::draw_kind(&g.lineage, "bold", 1).unwrap();
    assert_eq!((bold.reward.kind.as_str(), bold.reward.id.as_str()), ("verb", "hold"));
    crate::oath::grant(&mut g.lineage, &bold);
    assert!(g.vocabulary().verbs.contains(&Verb::new("hold")));
    let bold = crate::oath::draw_kind(&g.lineage, "bold", 2).unwrap();
    assert_eq!(bold.reward.kind, "title", "route 2 waits for T4 (the Lich met)");
    g.lineage.facts.insert("foe:lich".into());
    g.lineage.facts.insert("foe:bloat_mother".into());
    let bold = crate::oath::draw_kind(&g.lineage, "bold", 3).unwrap();
    assert_eq!((bold.reward.kind.as_str(), bold.reward.id.as_str()), ("route", "route2"));
    let tamer = crate::oath::draw_kind(&g.lineage, "tamer", 4).unwrap();
    assert_eq!(tamer.reward.id, "party_slot_2");
    // the price: ¼ of the last whole day's net (never under the forge's unit)
    g.lineage.day = 1;
    g.lineage.last_day_net = 40_000;
    assert_eq!(crate::oath::price(&g.lineage), 10_000);
    // route 2 opens the D9 fork: a run sees it
    crate::oath::grant(&mut g.lineage, &bold);
    assert!(crate::descent::fork_open_for(true, 9) && !crate::descent::fork_open_for(false, 9));
}

/// §4: the keep sheet is a decision only when a find beats something in the full vault; else it
/// settles by the standing order with one line. The repeat adds a kind a player's row names.
#[test]
fn a_tap_is_a_decision() {
    use crate::item::Item;
    let pending = |items: Vec<Item>| crate::engine::PendingExit { run_id: 1, tier: crate::engine::ExitTier::Bank, items, pct: 100, brought: Vec::new(), found: Vec::new(), stalled: false };
    let it = |id: u32, kind: &str, e: i32| {
        let mut i = Item::new(id, kind);
        i.enchant = e;
        i.known = true;
        i
    };
    // a one-slot vault holding leather; a leather +1 found beats it: a decision
    let vault = vec![it(1, "leather", 0)];
    assert!(crate::engine::keep_is_a_decision(&pending(vec![it(2, "leather", 1)]), &vault, 1));
    // a worse find: no decision
    assert!(!crate::engine::keep_is_a_decision(&pending(vec![it(3, "dagger", 0)]), &vault, 1));
    // an empty vault with room for the one find: no decision; the note names the keep
    let l = crate::engine::LineageState::new(1);
    let p = pending(vec![it(4, "leather", 1)]);
    assert!(!crate::engine::keep_is_a_decision(&p, &[], 1));
    let plan = crate::engine::auto_keep_plan(&p, &[], 1, "best_armour", true).0;
    assert_eq!(crate::engine::keep_note(&p, &l, &plan).as_deref(), Some("kept leather +1"));
    // the standing orders: one struct, each order through its own rules; insure on by default
    let mut g = Game::new(12);
    let mut o = g.lineage().orders;
    assert!(o.insure && o.repeat);
    o.keep = "best_weapon".into();
    o.insure = false;
    g.set_orders(&o).unwrap();
    assert_eq!(g.lineage().orders, o);
    o.keep = "junk".into();
    assert!(g.set_orders(&o).is_err());
    // the repeat adds a kind a player's row names (`throw fire`), once it is sold
    let mut g = Game::new(13);
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "fire") {
        g.lineage.facts.insert(f);
    }
    g.lineage.forge.insert("fire".into(), crate::wire::ForgeRow::at(5));
    g.lineage.unlocks.insert("throw".into());
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("throw", "fire,tag:boss")).from("player"));
    g.lineage.facts.insert("foe:goblin_warlord:boss".into());
    g.set_rules(set).unwrap();
    let adds = g.lineage().repeat_added;
    assert_eq!(adds.iter().map(|a| (a.kind.as_str(), a.row.as_str())).collect::<Vec<_>>(), vec![("fire", "throw fire")]);
    // a preset row adds nothing (the bots' sets pack what they pack)
    let g = Game::new(14);
    assert!(g.lineage().repeat_added.is_empty());
}

/// §6: a companion's fall is named on the report; a tamed grudge closes as tamed; the forecast's
/// gold splits out the passage; the shelf's cap is the core's.
#[test]
fn seams_of_cohort_24() {
    let g = Game::new(15);
    assert_eq!(g.lineage().supply_cap, 3);
    let mut g = Game::new(15);
    g.lineage.kit.insert("pack".into(), 1);
    assert_eq!(g.lineage().supply_cap, 4, "pack 4 bought → 4 slots (AX saw 3/3)");
    let f = g.forecast();
    assert_eq!(f.ends.as_ref().map(|e| e.passage), Some(0.0), "no passage from D1");
    // a grudge tamed closes as tamed (never avenged; it lives on no floor)
    let mut g = Game::new(16);
    g.lineage.grudges.push(crate::descent::Grudge { kind: "rat".into(), name: "Greth".into(), depth: 1, heir: 1, avenged: false, tamed: false, biome: None });
    g.start_run(Some(3));
    g.run.as_mut().unwrap().tamed_grudges.push("Greth".into());
    {
        let (run, mut cx) = g.ctx();
        crate::turn::end_run(run, &mut cx, crate::engine::ExitTier::Bank);
    }
    g.finish_run();
    assert!(g.lineage.grudges[0].tamed && !g.lineage.grudges[0].avenged);
}

/// §1 (E1): the wall's search is lazy — never inside an offline run (it was 20–60 s native at a
/// wall, minutes in wasm); `wall_edit` searches once a day at a wall and answers the cached offer
/// after; off a wall it answers nothing.
#[test]
fn wall_edit_is_lazy_and_cached() {
    let mut g = Game::new(21);
    assert_eq!(g.wall_edit(), None, "no best depth: no wall");
    g.lineage.best_depth = 9;
    g.lineage.best_day = 3;
    g.lineage.day = 4;
    assert_eq!(g.wall_edit(), None, "held one day: not a wall yet");
    assert_eq!(g.lineage.wall_day, None, "off a wall nothing is searched");
    // at the wall, searched today already: the cached offer, no search
    g.lineage.day = 5;
    g.lineage.wall_day = Some(5);
    let offer = crate::wire::WallEdit { depth: 9, edits: vec!["drop R6".into()], rules: g.lineage.rules().clone(), before: 0.0, after: 0.2, sims: 48 };
    g.lineage.wall_offer = Some(offer.clone());
    assert_eq!(g.wall_edit(), Some(offer.clone()));
    assert_eq!(g.lineage().wall, Some(offer));
    // an offline run leaves the day's search alone (and its report carries no wall)
    let day = g.lineage.day;
    let rep = serde_json::to_value(crate::offline::run_offline_quick(&mut g, 60)).unwrap();
    assert!(rep.get("wall").is_none());
    assert!(g.lineage.wall_day == Some(5) || g.lineage.day != day);
}
