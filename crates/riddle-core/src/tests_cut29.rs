//! Cut 29: the curve keeps opening; a tap is a decision (docs/CUT29.md).
use crate::engine::Game;
use crate::rules::{Cond, Row, Verb};
use crate::wire::Ev;

/// §2: a fresh lineage opens the day-0 systems only; each trigger opens its system (sticky), the
/// wire lists every system with its trigger and the new ones glint once; a bot's `open_all` opens all.
#[test]
fn systems_open_one_at_a_time() {
    // (Cut 30 Reveal: day 0 is the camp; the pen's group — the editor, the dial, the order — waits
    // for the Mother met or a 3-day stall)
    let mut g = Game::new_resident(3);
    g.lineage.clock_s = 2 * 3600;
    let open = |g: &Game| g.lineage().systems.iter().filter(|s| s.open).map(|s| s.id.clone()).collect::<Vec<_>>();
    assert_eq!(open(&g), vec!["send", "headline"]);
    let w = g.lineage();
    assert_eq!(w.systems.len(), crate::systems::SYSTEMS.len());
    assert!(w.systems.iter().all(|s| s.open || !s.trigger.is_empty()), "every closed system says what opens it");
    assert!(w.systems.iter().all(|s| !s.new), "day 0 does not glint");
    // the first death opens the death screen, not the editor
    g.lineage.graveyard.push(crate::wire::Grave { heir: 1, depth: 3, cause: "rat".into(), deeds: Vec::new(), death_id: None });
    g.lineage.reveal_left = 1;
    let opened = crate::systems::update(&mut g.lineage, false);
    assert_eq!(opened, vec!["death"]);
    assert!(g.lineage().systems.iter().any(|s| s.id == "death" && s.open && s.new));
    g.seen_systems();
    assert!(g.lineage().systems.iter().all(|s| !s.new));
    // the Warlord met: the stances; the pen still closed
    g.lineage.best_depth = 8;
    g.lineage.facts.insert("foe:goblin_warlord".into());
    g.lineage.clock_s = 2 * 3600;
    g.lineage.reveal_left = 1;
    let opened = crate::systems::update(&mut g.lineage, true);
    assert!(opened.contains(&"stances".to_string()) && !opened.contains(&"edit".to_string()) && !opened.contains(&"reorder".to_string()), "{opened:?}");
    // the pen opens its group
    g.lineage.pkg.pen_open = true;
    g.lineage.reveal_left = 1;
    let opened = crate::systems::update(&mut g.lineage, false);
    for id in ["pen", "edit", "dial", "reorder", "walls"] {
        assert!(opened.contains(&id.to_string()), "{id}: {opened:?}");
    }
    // sticky: a system stays open
    g.lineage.best_depth = 2;
    crate::systems::update(&mut g.lineage, false);
    assert!(crate::systems::is_open(&g.lineage, "walls"));
    // the bots: every system
    let mut b = Game::new_literal(4);
    crate::systems::open_all(&mut b.lineage);
    assert!(b.lineage().systems.iter().all(|s| s.open && !s.new));
    // an old save (no system recorded) opens what it has used — its set is the pen's
    let mut old = Game::new_literal(5);
    old.lineage.systems.clear();
    old.lineage.kills.insert("goblin_warlord".into());
    old.lineage.best_depth = 9;
    old.lineage.pkg_v = 0;
    let back = Game::load(&old.save()).unwrap();
    assert!(crate::systems::is_open(&back.lineage, "tactics") && crate::systems::is_open(&back.lineage, "walls"));
    assert!(back.lineage.systems_new.is_empty(), "an old save does not glint what it already used");
}

/// §3: a watched run's meter is the event stream's sums — damage by side, healing, fires,
/// supplies, gold — and its time split covers its ticks; the death screen carries the fight.
#[test]
fn meters_are_the_event_streams_sums() {
    let mut g = Game::new_literal(8);
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
    let mut g = Game::new_literal(9);
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
    let mut f = Game::new_literal(10);
    f.lineage.marks = 9;
    assert_eq!(f.draw_oath().unwrap_err(), "needs meet Warlord");
}

/// §1: the oaths' reward pool is their own — `bold` gives the verb `hold` (then the D9 route),
/// `lean` gas step at T3, `tamer` the party slots; the verb joins the vocabulary; a spent pool
/// gives a title. The price is a quarter of the last day's net once a day has closed.
#[test]
fn the_oath_pool_is_its_own() {
    let mut g = Game::new_literal(11);
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
    let mut g = Game::new_literal(12);
    let mut o = g.lineage().orders;
    assert!(o.insure && o.repeat);
    o.keep = "best_weapon".into();
    o.insure = false;
    g.set_orders(&o).unwrap();
    assert_eq!(g.lineage().orders, o);
    o.keep = "junk".into();
    assert!(g.set_orders(&o).is_err());
    // the repeat adds a kind a player's row names (`throw fire`), once it is sold
    let mut g = Game::new_literal(13);
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
    let g = Game::new_literal(14);
    assert!(g.lineage().repeat_added.is_empty());
}

/// §6: a companion's fall is named on the report; a tamed grudge closes as tamed; the forecast's
/// gold splits out the passage; the shelf's cap is the core's.
#[test]
fn seams_of_cohort_24() {
    let g = Game::new_literal(15);
    assert_eq!(g.lineage().supply_cap, 3);
    let mut g = Game::new_literal(15);
    g.lineage.kit.insert("pack".into(), 1);
    assert_eq!(g.lineage().supply_cap, 4, "pack 4 bought → 4 slots (AX saw 3/3)");
    let f = g.forecast();
    assert_eq!(f.ends.as_ref().map(|e| e.passage), Some(0.0), "no passage from D1");
    // a grudge tamed closes as tamed (never avenged; it lives on no floor)
    let mut g = Game::new_literal(16);
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
    let mut g = Game::new_literal(21);
    assert_eq!(g.wall_edit(), None, "no best depth: no wall");
    g.lineage.best_depth = 9;
    g.lineage.best_day = 3;
    g.lineage.day = 4;
    assert_eq!(g.wall_edit(), None, "held one day: not a wall yet");
    assert_eq!(g.lineage.wall_day, None, "off a wall nothing is searched");
    // at the wall, searched today already: the cached offer, no search
    g.lineage.day = 5;
    g.lineage.wall_day = Some(5);
    let offer = crate::wire::WallEdit { depth: 9, edits: vec!["drop R6".into()], rules: g.lineage.rules().clone(), before: 0.0, after: 0.2, sims: 48, start: None };
    g.lineage.wall_offer = Some(offer.clone());
    assert_eq!(g.wall_edit(), Some(offer.clone()));
    assert_eq!(g.lineage().wall, Some(offer));
    // an offline run leaves the day's search alone (and its report carries no wall)
    let day = g.lineage.day;
    let rep = serde_json::to_value(crate::offline::run_offline_quick(&mut g, 60)).unwrap();
    assert!(rep.get("wall").is_none());
    assert!(g.lineage.wall_day == Some(5) || g.lineage.day != day);
}

/// §1 (E1, record spikes): the wall's edits never take the set's way home — its only exit row is
/// neither dropped nor written over (rater AO s1: nine days without a send home after a wall edit
/// replaced its bank) — while a second exit may go; an offer saved before `start` reads none.
#[test]
fn the_wall_keeps_the_last_way_home() {
    let g = Game::new_literal(5);
    let bank = Row::new(vec![Cond::n("hp<", 30)], Verb::new("bank"));
    let push = Row::new(vec![Cond::n("depth>=", 6)], Verb::new("bank"));
    let heal = Row::new(vec![Cond::n("hp<", 40)], Verb::arg("drink", "heal"));
    let fight = Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"));
    let exit = |r: &Row| matches!(r.verb.v.as_str(), "bank" | "return");
    let one = crate::rules::RuleSet { rows: vec![heal.clone(), bank.clone(), fight.clone()], ..Default::default() };
    let cands = crate::wall::edits(&g, &one, 5);
    assert!(!cands.is_empty());
    assert!(cands.iter().all(|(_, s)| s.rows.iter().any(exit)), "every edit keeps an exit: {:?}", cands.iter().map(|c| &c.0).collect::<Vec<_>>());
    assert!(!cands.iter().any(|(l, _)| l == "drop R2" || l.starts_with("R2 → ")), "the only exit is neither dropped nor written over");
    let two = crate::rules::RuleSet { rows: vec![heal, push, bank, fight], ..Default::default() };
    let cands = crate::wall::edits(&g, &two, 5);
    assert!(cands.iter().any(|(l, _)| l == "drop R2"), "a second exit may go");
    assert!(cands.iter().all(|(_, s)| s.rows.iter().any(exit)));
    let old = r#"{"depth":9,"edits":["drop R6"],"rules":{"rows":[]},"before":0.0,"after":0.2,"sims":48}"#;
    let w: crate::wire::WallEdit = serde_json::from_str(old).unwrap();
    assert_eq!(w.start, None);
}

/// §5 (Cut 29 core 4): a commission is priced against income — the climb (10 units × 1.25ⁿ) never
/// above half a day's net, never under `COMMISSION_FLOOR` units.
#[test]
fn a_commission_costs_at_most_a_days_net() {
    let mut g = Game::new_literal(5);
    g.lineage.kit_unit = Some(100);
    let price = |g: &Game| crate::kit::commission_price(&g.lineage);
    g.lineage.last_day_net = 5000;
    assert_eq!(price(&g), 1000, "the climb's first step under a big day");
    g.lineage.works = vec!["a".into(), "b".into(), "c".into()];
    assert_eq!(price(&g), 1950);
    g.lineage.last_day_net = 2400;
    assert_eq!(price(&g), 1200, "half a day's net caps it");
    g.lineage.last_day_net = 0;
    assert_eq!(price(&g), 100 * crate::kit::COMMISSION_FLOOR as i32, "the floor");
}

/// Cut 29 core (item 3, marks at the deepest wall): with the pool spent and every title of the day
/// owned, a draw still draws — the titles come again, numbered (`Lean at D33 II`) — so the ◆2 draw
/// is a sink that never dries; a numbered title stands on the board until it is owned.
#[test]
fn titles_come_again_numbered_so_draws_never_dry() {
    let mut g = Game::new_literal(11);
    g.lineage.best_depth = 33;
    g.lineage.banked_depths.insert(33);
    for (boss, _) in crate::descent::BOSS_DEPTHS {
        g.lineage.facts.insert(format!("foe:{boss}"));
    }
    crate::oath::grant_all(&mut g.lineage);
    for kind in ["bold", "lean", "fire"] {
        let o = crate::oath::draw_kind(&g.lineage, kind, 50).unwrap();
        assert_eq!(o.reward.kind, "title", "{kind}");
        crate::oath::grant(&mut g.lineage, &o);
        let again = crate::oath::draw_kind(&g.lineage, kind, 51).unwrap();
        let line = o.reward.id.trim_end_matches(['I', 'V', 'X', ' ']);
        assert!(again.reward.id.starts_with(line) && again.reward.id != o.reward.id && !g.lineage.titles.contains(&again.reward.id), "{kind}: the line again, numbered ({} → {})", o.reward.id, again.reward.id);
    }
    crate::oath::refresh(&mut g.lineage);
    g.lineage.marks = 40;
    for i in 0..15 {
        g.draw_oath().unwrap_or_else(|e| panic!("draw {i}: {e}"));
    }
    assert_eq!(g.lineage.marks, 10);
    let titles: Vec<&str> = g.lineage.oaths.iter().filter(|o| o.reward.kind == "title").map(|o| o.reward.id.as_str()).collect();
    let mut dedup = titles.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(dedup.len(), titles.len(), "no title offered twice: {titles:?}");
}
