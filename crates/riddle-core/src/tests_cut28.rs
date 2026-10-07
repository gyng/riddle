//! Cut 28 (core): oaths, the move that names its cause, calm stretches, the defects.
use crate::engine::{ExitTier, Game, HERO_ID};
use crate::geom::Pos;
use crate::monster::Monster;
use crate::rules::{Cond, Row, RuleSet, Verb};
use crate::wire::*;

/// The metrics' EDITED lineage (as `tests_cut27::edited`): the common items known, eight rows,
/// the good set, D`best` reached.
fn edited(seed: u64, best: u32) -> Game {
    let mut g = Game::new_literal(seed);
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink"] {
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss"] {
        g.lineage.facts.insert(f.into());
    }
    g.set_rules(crate::probes::good()).expect("good rules");
    g.lineage.best_depth = best;
    crate::oath::refresh(&mut g.lineage);
    g
}

fn jackal(g: &mut Game, name: &str) -> Companion {
    let id = g.lineage.new_comp_id();
    crate::engine::new_companion(id, &Monster::spawn(0, "jackal", Pos::new(0, 0), 3), name.into())
}

/// §1: a lineage stands three oaths, each a constraint and a reward that is never a stat, priced
/// from its income; swearing pays, one at a time; forswearing refunds half; the board survives a save.
#[test]
fn the_oath_board_stands_three_and_swearing_is_a_gold_sink() {
    let mut g = edited(3, 9);
    let l = g.lineage();
    assert_eq!(l.oaths.len(), crate::oath::BOARD, "{:?}", l.oaths);
    let kinds: std::collections::BTreeSet<&str> = l.oaths.iter().map(|o| o.kind.as_str()).collect();
    assert_eq!(kinds.len(), crate::oath::BOARD, "three kinds");
    for o in &l.oaths {
        assert!(matches!(o.reward.kind.as_str(), "card" | "slot" | "row" | "title" | "waystone" | "verb"), "{o:?}");
        assert!(o.chips.iter().all(|c| crate::rules::word_count(c) <= 3) && crate::rules::word_count(&o.reward.label) <= 3, "{o:?}");
        assert_eq!(o.price, crate::oath::price(&g.lineage));
        assert!(!o.sworn);
    }
    // $100 + $25 a floor of the best depth, or half a night's net when that is more
    assert_eq!(crate::oath::price(&g.lineage), 330);
    g.lineage.last_night_net = 2000;
    assert_eq!(crate::oath::price(&g.lineage), 1000);
    g.lineage.last_night_net = 0;
    let id = l.oaths[0].id.clone();
    g.lineage.gold = 100;
    assert_eq!(g.swear_oath(&id).unwrap_err(), "not enough gold");
    g.lineage.gold = 1000;
    g.swear_oath(&id).unwrap();
    assert_eq!(g.lineage.gold, 1000 - 330);
    let l = g.lineage();
    assert_eq!(l.oath.as_deref(), Some(id.as_str()));
    assert!(l.oaths.iter().any(|o| o.id == id && o.sworn));
    // the price is the player's purchase: no income in the night's net
    assert_eq!(g.lineage.night_net, 0);
    // another oath forswears the first (half back) and pays its own
    let other = l.oaths[1].id.clone();
    g.swear_oath(&other).unwrap();
    assert_eq!(g.lineage.gold, 1000 - 330 + 165 - 330);
    g.forswear_oath().unwrap();
    assert_eq!(g.lineage.gold, 1000 - 330 + 165 - 330 + 165);
    assert_eq!(g.lineage.oath_sworn, None);
    assert_eq!(g.forswear_oath().unwrap_err(), "no oath sworn");
    // the board rides the save
    g.swear_oath(&id).unwrap();
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.lineage().oaths, g.lineage().oaths);
    assert_eq!(back.lineage.oath_sworn.as_deref(), Some(id.as_str()));
}

/// §1: each pool oath reads its send — the floor reached and what the run did — and a kept oath
/// grants its reward, leaves the board and draws the next; nothing swears on its own.
#[test]
fn an_oath_kept_grants_its_reward_and_the_board_refills() {
    let mut g = edited(5, 9);
    g.lineage.gold = 5000;
    // (Cut 29 §1: `lean`'s first card, gas step, opens at T3 — the Mother met)
    g.lineage.facts.insert("foe:bloat_mother".into());
    let o = crate::oath::draw_kind(&g.lineage, "lean", 77).unwrap();
    assert_eq!(o.depth, 9, "the record, without rest");
    assert_eq!(o.reward.kind, "card");
    assert_eq!(crate::oath::chips(&o), vec!["reach D9".to_string(), "no rest".to_string()]);
    g.lineage.oaths = vec![o.clone()];
    crate::oath::refresh(&mut g.lineage);
    assert_eq!(g.lineage.oaths.len(), 3);
    g.swear_oath(&o.id).unwrap();
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.max_depth = 10;
    // a rest breaks it
    run.rested = true;
    assert!(!crate::oath::kept(&o, run));
    run.rested = false;
    run.over = Some(ExitTier::Return);
    assert!(crate::oath::kept(&o, run));
    let card = o.reward.id.clone();
    assert!(!g.lineage.unlocks.contains(&card));
    let _ = g.finish_run();
    assert!(g.lineage.unlocks.contains(&card), "the card is the lineage's");
    assert_eq!(g.lineage.oath_sworn, None);
    assert!(!g.lineage.oaths.iter().any(|x| x.id == o.id));
    assert_eq!(g.lineage.oaths.len(), 3, "a new oath drawn");
    assert_eq!(g.lineage.oaths_kept, 1);
    let (_, runs, kept, done) = g.batch.oath.clone().unwrap();
    assert_eq!((runs, kept, done), (1, 1, true));
    // the other kinds
    let run = |f: &dyn Fn(&mut crate::engine::Run)| {
        let mut h = edited(5, 9);
        h.start_run(None);
        let r = h.run.as_mut().unwrap();
        f(r);
        h
    };
    let d9 = edited(5, 9);
    let bold = crate::oath::draw_kind(&d9.lineage, "bold", 1).unwrap();
    assert_eq!(bold.depth, 10);
    let h = run(&|r| {
        r.max_depth = 10;
        r.over = Some(ExitTier::Return);
    });
    assert!(!crate::oath::kept(&bold, h.run.as_ref().unwrap()), "a return breaks `no return`");
    let h = run(&|r| {
        r.max_depth = 10;
        r.over = Some(ExitTier::Bank);
    });
    assert!(crate::oath::kept(&bold, h.run.as_ref().unwrap()));
    let lean = crate::oath::draw_kind(&d9.lineage, "lean", 1).unwrap();
    let h = run(&|r| {
        r.max_depth = 10;
        r.rested = true;
    });
    assert!(!crate::oath::kept(&lean, h.run.as_ref().unwrap()));
    let fire = crate::oath::draw_kind(&d9.lineage, "fire", 1).unwrap();
    assert_eq!(fire.boss.as_deref(), Some("goblin_warlord"));
    let h = run(&|r| r.boss_kills.push((5, "goblin_warlord".into())));
    assert!(!crate::oath::kept(&fire, h.run.as_ref().unwrap()), "slain without fire");
    let h = run(&|r| {
        r.boss_kills.push((5, "goblin_warlord".into()));
        r.burned.push("goblin_warlord".into());
    });
    assert!(crate::oath::kept(&fire, h.run.as_ref().unwrap()));
    // a slayer names the next band boss within reach (the Mother for a D12 lineage) and points at her counter
    let mut deep = edited(5, 12);
    deep.lineage.kills.insert("goblin_warlord".into());
    let slay = crate::oath::draw_kind(&deep.lineage, "slayer", 1).unwrap();
    assert_eq!((slay.boss.as_deref(), slay.reward.label.as_str()), (Some("bloat_mother"), "waystone D14"));
    assert_eq!(crate::oath::counter_fact(&deep.lineage, "bloat_mother"), "mother: ?");
    deep.lineage.facts.insert(crate::facts::boss_counter_fact("bloat_mother"));
    assert_eq!(crate::oath::counter_fact(&deep.lineage, "bloat_mother"), "mother: fire");
    let tamer = crate::oath::draw_kind(&deep.lineage, "tamer", 1).unwrap();
    let h = run(&|r| r.tamed.push((3, "jackal".into())));
    assert!(crate::oath::kept(&tamer, h.run.as_ref().unwrap()));
    // an absence never swears
    let mut idle = edited(6, 9);
    idle.lineage.gold = 9999;
    let _ = crate::offline::run_offline_counts(&mut idle, 2 * 3600);
    assert_eq!(idle.lineage.oath_sworn, None);
    assert_eq!(idle.lineage.oaths_kept, 0);
}

/// §1: the forecast prices the sworn oath on its own panel, the edit's paired move carries its
/// move, and the walls and the bounty say what the deep asks.
#[test]
fn the_forecast_prices_the_sworn_oath() {
    let mut g = edited(7, 6);
    assert!(g.forecast().oath.is_none());
    g.lineage.gold = 5000;
    let o = crate::oath::draw_kind(&g.lineage, "lean", 1).unwrap();
    g.lineage.oaths.insert(0, o.clone());
    g.swear_oath(&o.id).unwrap();
    let f = g.forecast();
    let s = f.oath.clone().expect("the sworn oath priced");
    assert_eq!(s.id, o.id);
    assert!((0.0..=1.0).contains(&s.share));
    assert!((s.night - crate::oath::night(s.share)).abs() < 1e-9);
    // the share is the panel's: the sims that reached the record without a rest
    let panel = crate::forecast::camp_panel(&g, g.lineage.rules(), crate::forecast::camp_sims(&g, g.lineage.rules()));
    let reach = panel.iter().filter(|r| r.max_depth >= o.depth).count();
    assert!(panel.iter().filter(|r| r.oath).count() <= reach);
    // an edit's move carries the oath's
    let prev = g.lineage.rules().clone();
    let dry = RuleSet { rows: prev.rows.iter().filter(|r| r.verb.v != "drink").cloned().collect(), ..prev.clone() };
    g.set_rules(dry).unwrap();
    let vs = g.forecast_vs(&prev);
    assert!(vs.oath.is_some());
    // walls: from the Warlord to the first unslain past the best
    let l = g.lineage();
    assert_eq!(l.walls.first().map(|w| w.boss.as_str()), Some("goblin_warlord"));
    assert!(l.walls.iter().all(|w| !w.fact.is_empty()));
}

/// §2: the camp's move against the set sent splits into the state's part (a pet lost since the
/// send) and the rows' part; the parts sum to the whole on every term; a state-only change has no
/// rows part and the divergence has nothing to run on.
#[test]
fn the_forecast_move_names_its_cause() {
    let mut g = edited(11, 6);
    let d1 = jackal(&mut g, "Drix");
    let d2 = jackal(&mut g, "Theth");
    g.lineage.party = vec![d1, d2];
    g.lineage.unlocks.insert("party_slot_2".into());
    let sent = g.lineage.rules().clone();
    assert!(g.forecast_move(&sent).is_none(), "no send recorded");
    g.send();
    g.run = None;
    // the camp as it was at the send (the run abandoned: its pack not re-bought), and both pets fell
    let at = g.sent_state.as_ref().unwrap().clone();
    g.lineage = at.lineage.clone();
    g.loadout = at.loadout.clone();
    g.lineage.party.clear();
    let m = g.forecast_move(&sent).expect("a send recorded");
    assert!(!m.rows && m.state);
    assert_eq!(m.parts.iter().map(|p| p.kind.as_str()).collect::<Vec<_>>(), vec!["party"]);
    assert_eq!(m.parts[0].text, "party −2 jackals");
    assert_eq!(m.lead, "party");
    // an edit on top: the rows' part joins, and the parts sum to the whole
    let mut edit = sent.clone();
    let last = edit.rows.len() - 1;
    edit.rows.remove(last);
    edit.rows.insert(0, Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")));
    g.set_rules(edit).unwrap();
    let m = g.forecast_move(&sent).unwrap();
    assert!(m.rows && m.state);
    assert_eq!(m.parts.iter().map(|p| p.kind.as_str()).collect::<Vec<_>>(), vec!["party", "rows"]);
    let sum = |f: &dyn Fn(&ForecastVs) -> f64| m.parts.iter().map(|p| f(&p.move_)).sum::<f64>();
    for (name, f) in [("bank", &(|v: &ForecastVs| v.bank.delta) as &dyn Fn(&ForecastVs) -> f64), ("death", &|v: &ForecastVs| v.death.delta), ("return", &|v: &ForecastVs| v.return_.delta), ("gold", &|v: &ForecastVs| v.gold.delta), ("stall", &|v: &ForecastVs| v.stall.delta)] {
        assert!((sum(f) - f(&m.whole)).abs() < 1e-9, "{name}: {} vs {}", sum(f), f(&m.whole));
    }
    for (i, d) in m.whole.depths.iter().enumerate() {
        let s: f64 = m.parts.iter().map(|p| p.move_.depths[i].delta).sum();
        assert!((s - d.delta).abs() < 1e-9, "D{}", d.depth);
    }
    // the rows' part is the edit's paired move on today's lineage (`forecast_vs`)
    let vs = g.forecast_vs(&sent);
    let rows = &m.parts[1].move_;
    assert!((rows.bank.delta - vs.bank.delta).abs() < 1e-9 && (rows.death.delta - vs.death.delta).abs() < 1e-9);
    // the move survives a save (a mirror lane loads it)
    let back = Game::load(&g.save()).unwrap();
    assert_eq!(back.forecast_move(&sent).map(|m| m.parts.len()), Some(2));
}

/// §2: a trace carries the hero's max hp at each action and the max-hp steps of its window.
#[test]
fn traces_carry_max_hp_and_its_steps() {
    let mut g = edited(12, 3);
    g.send();
    for _ in 0..40 {
        g.step(10);
    }
    let run = g.run.as_mut().unwrap();
    let max = run.hero.max_hp;
    assert!(run.trace.iter().all(|t| t.max_hp > 0), "every action carries his max");
    let mut cx_run = run.clone();
    let _ = &mut cx_run;
    run.hero.max_hp -= 3;
    run.max_steps.push(MaxStep { t: run.turn, max: max - 3, delta: -3, cause: "drain".into() });
    let tr = crate::engine::exit_trace(run, &[]);
    assert_eq!(tr.max_steps.last().map(|s| (s.max, s.delta)), Some((max - 3, -3)));
}

/// §2: a death most replays survive leads with the rare event and its odds; one they mostly lose
/// does not; a patch no better than the base is `no_gain`.
#[test]
fn luck_deaths_name_the_event() {
    let blow = |t, by: &str, dmg, hp| TraceBlow { t, by: by.into(), dmg, hp };
    let mut d = Death { difficulty: 0, modifier_catalogue: Vec::new(), modifiers: None, hero: None, package: None, lever: None, run_id: 1, depth: 6, cause: "goblin".into(), margin: "2 hp short".into(), verdict: "dice".into(), baseline: 10.0 / 12.0, replays: 12, trace: Trace::default(), patches: Vec::new(), morgue: String::new(), line: None, chain: None, rules: None, notes: Vec::new(), nothing_beats_base: false, cause_row: None, order_over: None, route_cause: None, lean: None, luck: None, fight: None };
    d.trace.turns.push(TraceTurn { max_hp: 36, t: 100, row: 0, verb: Verb::new("attack"), hp: 6, foes: 2, rule_foes: 2, telegraphs: Vec::new(), blocked: None, rows: None, blows: Vec::new(), gift: None });
    d.trace.blow = Some(blow(105, "goblin", 6, 0));
    let l = crate::trace::luck_of(&d).expect("most replays live");
    assert_eq!((l.one_in, l.t), (6, 105));
    assert!(l.text.contains("6 hp"), "{}", l.text);
    d.trace.blows = vec![blow(104, "goblin", 3, 3), blow(105, "goblin", 3, 0)];
    assert_eq!(crate::trace::luck_of(&d).unwrap().text, "two blows at 6 hp");
    d.baseline = 5.0 / 12.0;
    assert!(crate::trace::luck_of(&d).is_none(), "a death most replays share is no luck");
    let p = |survive: f64| Patch { row: Row::new(vec![Cond::n("hp<", 30)], Verb::new("return")), insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None, whole: None, gem: false, restores: None, no_gain: false };
    d.patches = vec![p(5.0 / 12.0), p(0.0), p(0.9)];
    crate::trace::mark_no_gain(&mut d);
    assert_eq!(d.patches.iter().map(|p| p.no_gain).collect::<Vec<_>>(), vec![true, true, false]);
}

/// §3: calm ticks — no decision, no threat — come back as stretches with the step.
#[test]
fn calm_stretches_come_with_the_step() {
    assert_eq!(crate::fold::calm_spans(&[(1, true), (2, true), (3, false), (4, true), (5, true), (6, true)]), vec![[1, 2], [4, 6]]);
    assert!(crate::fold::calm_spans(&[(1, false)]).is_empty());
    let mut g = edited(13, 2);
    g.send();
    let mut calm = 0u32;
    let mut loud_in_calm = 0;
    for _ in 0..60 {
        let r = g.step(20);
        for s in &r.calm {
            calm += s[1] - s[0] + 1;
            // no row of the set acts inside a calm stretch
            loud_in_calm += r.events.iter().filter(|e| matches!(e, Ev::Rule { t, row, verb, .. } if *t >= s[0] && *t <= s[1] && *row >= 0 && verb.v != "pick_up" && verb.v != "rest")).count();
            loud_in_calm += r.events.iter().filter(|e| matches!(e, Ev::Attack { t, .. } | Ev::Descend { t, .. } if *t >= s[0] && *t <= s[1])).count();
        }
        if r.run_over {
            break;
        }
    }
    assert!(calm > 0, "a run has calm stretches");
    assert_eq!(loud_in_calm, 0);
}

/// §4: the fold hands a hurt hero to the watch saying so (`hp 9/40`).
#[test]
fn a_fold_that_hands_off_a_hurt_hero_says_so() {
    for seed in 1..40 {
        let mut g = edited(seed, 6);
        if g.forecast().fold_to.is_none_or(|t| t < 2) {
            continue;
        }
        g.send();
        let f = g.fold();
        if f.floors.is_empty() || f.step.run_over {
            continue;
        }
        let hero = &f.step.snapshot.hero.entity;
        assert_eq!((f.hp, f.max_hp), (hero.hp, hero.max_hp));
        assert_eq!(f.low, hero.hp * 2 <= hero.max_hp);
        if f.low {
            assert!(f.chips.contains(&format!("hp {}/{}", f.hp, f.max_hp)), "{:?}", f.chips);
            return;
        }
    }
}

/// §4: a grudge never takes a companion's name.
#[test]
fn a_grudge_never_takes_a_pets_name() {
    let mut g = edited(21, 4);
    // every name the generator can draw next is a pet's but one
    let mut rng = g.lineage.rng.clone();
    let first = crate::descent::grudge_name(&mut rng);
    let pet = jackal(&mut g, &first);
    g.lineage.party = vec![pet];
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.over = Some(ExitTier::Death);
    run.death_cause = Some("jackal".into());
    let _ = g.finish_run();
    let grudge = g.lineage.grudges.last().expect("a grudge");
    assert_ne!(grudge.name, first, "the pet's name is not the killer's");
}

/// §4: the reel holds a line's shape at most twice; a story's low is a low.
#[test]
fn the_reel_never_repeats_a_shape_thrice() {
    assert_eq!(crate::sifter::shape("A goblin took him to 12 HP; R2 returned; returned $40."), "A goblin took him to N HP; RN returned; returned $N.");
    let h = |i: u32, text: &str, threat: &str| Highlight { pattern: "episode".into(), score: 10, t: i, run_id: i, text: text.into(), arc: Some(HighlightArc { low_hp: 3, row: 1, threat: threat.into(), resolution: format!("r{i}") }) };
    let hs: Vec<Highlight> = (1..=5).map(|i| h(i, &format!("A goblin took him to {i} HP; R2 returned; returned ${i}0."), &format!("t{i}"))).collect();
    let reel = crate::sifter::reel(&hs, None, &[]);
    assert!(reel.len() <= crate::sifter::SHAPE_MAX, "{:?}", reel.iter().map(|x| &x.text).collect::<Vec<_>>());
    // the saved note turns its words: never the same three runs running
    assert!(crate::chronicle::is_saved_note("R2 return saved him.") && crate::chronicle::is_saved_note("R2 return got him out."));
}

/// §4: a caged item reaches the keep sheet on a return whatever the cut (a one-slot vault offers
/// the keep choice).
#[test]
fn a_caged_item_reaches_the_keep_sheet() {
    let mut g = edited(31, 4);
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    // a cheap caged dagger among dearer finds
    let mut items = Vec::new();
    for (i, k) in ["mail", "sword", "axe", "leather"].iter().enumerate() {
        let mut it = crate::item::Item::new(900 + i as u32, k);
        it.known = true;
        items.push(it);
    }
    let mut dagger = crate::item::Item::new(950, "dagger");
    dagger.known = true;
    items.push(dagger);
    run.hero.inv.extend(items.iter().cloned());
    for it in &items {
        run.note_found(it.id, &it.kind, 1);
    }
    run.caged.push(950);
    run.over = Some(ExitTier::Return);
    let _ = g.finish_run();
    let p = g.pending_exit.as_ref().expect("a keep sheet");
    assert!(p.items.iter().any(|i| i.id == 950), "the caged dagger is on the sheet: {:?}", p.items.iter().map(|i| &i.kind).collect::<Vec<_>>());
    let _ = HERO_ID;
}

/// §4 (AU: the `kite archers` card left the forecast on `…` until dropped): the card at every place
/// of a set, bare or with a condition, is taken and priced — both passes, the edit's move and its
/// scene come back, every panel's sims inside the run cap. (Not reproduced natively or in wasm over
/// 3 cohort sets × 4 seeds × 3 ages × every place and six conditions, pets or none; this pins it.)
#[test]
fn the_kite_archers_card_is_priced_at_every_place() {
    let mut g = edited(41, 6);
    g.lineage.unlocks.insert("kite_archers".into());
    g.lineage.facts.insert("foe:goblin_archer:ranged".into());
    let sent = g.lineage.rules().clone();
    for at in [0, 2, sent.rows.len()] {
        for conds in [vec![], vec![Cond::t("foe_tag", "ranged")], vec![Cond::n("hp<", 30)]] {
            let mut s = sent.clone();
            s.rows.insert(at, Row::new(conds.clone(), Verb::arg("tactic", "kite_archers")).from("card"));
            let mut h = g.clone();
            h.set_rules(s).unwrap_or_else(|e| panic!("R{} {conds:?}: refused {e}", at + 1));
            let f = h.forecast_refine();
            assert!(f.sims >= crate::forecast::MIN_SIMS && f.refined);
            let v = h.forecast_vs(&sent);
            assert_eq!(v.sims, f.sims, "the move pairs the bars' own sims");
            let _ = h.divergence(&sent);
            let panel = crate::forecast::camp_panel(&h, h.lineage.rules(), crate::forecast::REFINE_SIMS);
            assert!(panel.iter().all(|r| r.ticks <= crate::forecast::SIM_MAX_TICKS));
        }
    }
}

/// Cut 28b (owner: "it's not clear what oaths do"): a sworn oath's fate is said once in every
/// send, as it happens (`Ev::Oath`) — kept the moment its condition is met, broken at the row that
/// used the tool it forbids (`R2 return`, `rest`), missed at an end short of it — and always agrees
/// with the settle: the batch's tally, the exit line's news (`oath broken: R2 return`), the run's
/// notes (`Broke the oath: R2 return.`), the report's line.
#[test]
fn a_sworn_oath_says_its_fate_once_and_the_settle_agrees() {
    let mut seen: std::collections::BTreeMap<(String, &str), u32> = Default::default();
    for kind in ["lean", "bold", "tamer"] {
        for seed in 1..13u64 {
            let mut g = edited(seed, 3);
            g.lineage.gold = 5000;
            let Some(o) = crate::oath::draw_kind(&g.lineage, kind, 900 + seed as u32) else { continue };
            g.lineage.oaths = vec![o.clone()];
            crate::oath::refresh(&mut g.lineage);
            g.swear_oath(&o.id).unwrap();
            g.send();
            let mut evs = Vec::new();
            for _ in 0..4000 {
                let r = g.step(40);
                evs.extend(r.events);
                if r.run_over {
                    break;
                }
            }
            let said: Vec<(bool, i32, String)> = evs.iter().filter_map(|e| match e { Ev::Oath { kept, row, cause, .. } => Some((*kept, *row, cause.clone())), _ => None }).collect();
            assert_eq!(said.len(), 1, "{kind} seed {seed}: said once ({said:?})");
            let (ok, row, cause) = said[0].clone();
            // the event comes before the exit's
            let at = evs.iter().position(|e| matches!(e, Ev::Oath { .. })).unwrap();
            assert!(evs[at..].iter().any(|e| matches!(e, Ev::Exit { .. })), "{kind} seed {seed}: the oath before the exit");
            let (_, runs, kept, _) = g.batch.oath.clone().unwrap();
            assert_eq!((runs, kept), (1, ok as u32), "{kind} seed {seed}: the settle agrees ({said:?})");
            let news: Vec<String> = evs.iter().filter_map(|e| match e { Ev::Exit { line: Some(l), .. } => Some(l.news.iter().map(|n| n.text.clone()).collect::<Vec<_>>()), _ => None }).flatten().collect();
            let notes: Vec<String> = evs.iter().filter_map(|e| match e { Ev::Note { text, .. } => Some(text.clone()), _ => None }).collect();
            let state = if ok {
                assert!(news.iter().any(|n| n.starts_with("oath kept")), "{news:?}");
                assert!(notes.iter().any(|n| n == "Kept the oath."), "{notes:?}");
                "kept"
            } else if !cause.is_empty() {
                let text = if row >= 0 { format!("R{} {cause}", row + 1) } else { cause.clone() };
                assert!(matches!(cause.as_str(), "rest" | "return" | "stalled" | "driven"), "{cause}");
                assert!(news.contains(&format!("oath broken: {text}")), "{kind} seed {seed}: {news:?}");
                assert!(notes.contains(&format!("Broke the oath: {text}.")), "{notes:?}");
                assert_eq!(g.batch.oath_breaks.get(&text), Some(&1));
                // the chronicle keeps it (the heir's deeds, or his line once he fell)
                let chron = serde_json::to_string(&g.lineage.chronicle).unwrap();
                assert!(g.lineage.heir_deeds.iter().any(|d| d.starts_with("broke the oath")) || chron.contains("broke the oath"), "{:?} {chron}", g.lineage.heir_deeds);
                assert!(crate::rules::word_count(&text) <= 3);
                "broken"
            } else {
                assert!(!news.iter().any(|n| n.starts_with("oath")) && !notes.iter().any(|n| n.contains("oath")), "a miss is quiet");
                "missed"
            };
            // a kept oath is off the board, a broken or missed one still sworn (the stake rides)
            assert_eq!(g.lineage.oath_sworn.is_some(), !ok, "{kind} seed {seed}");
            *seen.entry((kind.to_string(), state)).or_insert(0) += 1;
        }
    }
    assert!(seen.keys().any(|(k, s)| k == "lean" && *s == "broken"), "a rest breaks `no rest` somewhere: {seen:?}");
    assert!(seen.keys().any(|(_, s)| *s == "kept"), "some send keeps one: {seen:?}");
}

/// Cut 28b: the oath board opens when it has something to answer — a band boss seen or a plateau
/// met — not when the purse first covers a price; an old save with an oath sworn or kept stays open.
#[test]
fn the_oath_board_opens_at_a_wall_or_a_plateau() {
    let mut g = edited(4, 2);
    assert!(!g.lineage().oath_open, "a young lineage");
    g.lineage.gold = 100_000;
    assert!(!g.lineage().oath_open, "gold alone opens nothing");
    g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    assert!(g.lineage().oath_open, "a wall known");
    let mut g = edited(4, 2);
    g.lineage.oath_open = true;
    let back = Game::load(&g.save()).unwrap();
    assert!(back.lineage().oath_open, "rides the save");
    // a run that sees a band boss opens it
    let mut g = edited(6, 7);
    g.send();
    g.descend_to(8);
    for _ in 0..3000 {
        if g.step(20).run_over {
            break;
        }
    }
    assert!(g.lineage.oath_open, "a run at the Warlord's floor opens the board");
    // a plateau: the stall window's runs
    let mut g = edited(4, 2);
    g.stall.runs = crate::offline::STALL_MIN_RUNS;
    let _ = crate::offline::run_offline_quick(&mut g, 60);
    assert!(g.lineage.oath_open, "a plateau opens it");
}

/// Cut 28b (AW: "the Mother oath sat at 9% ±8 with no lever I could find"; AX: "never understood
/// what `D3 · no return` required"): a depth oath's floor reads as a goal (`reach D3`), and the
/// sworn oath's panel names its steps — the floor reached, the boss met, the boss burned — each a
/// share of the sends, ordered as a funnel.
#[test]
fn a_sworn_oath_panel_names_its_steps() {
    let mut g = edited(8, 12);
    let bold = crate::oath::draw_kind(&g.lineage, "bold", 1).unwrap();
    assert_eq!(crate::oath::chips(&bold)[0], "reach D13");
    let mut o = crate::oath::draw_kind(&g.lineage, "fire", 2).unwrap();
    o.boss = Some("bloat_mother".into());
    o.depth = 13;
    assert_eq!(crate::oath::step_names(&o).iter().map(|s| s.0.as_str()).collect::<Vec<_>>(), vec!["D13", "met", "burned"]);
    g.lineage.oaths = vec![o.clone()];
    g.lineage.oath_sworn = Some(o.id.clone());
    let f = crate::forecast::forecast(&g);
    let s = f.oath.expect("the sworn oath's share");
    assert_eq!(s.steps.len(), 3);
    assert!(s.steps.windows(2).all(|w| w[0].share + 1e-9 >= w[1].share), "a funnel: {:?}", s.steps);
    assert!(s.steps[2].share + 1e-9 >= s.share, "kept needs burned: {s:?}");
    // a run's bits
    g.start_run(None);
    let run = g.run.as_mut().unwrap();
    run.max_depth = 13;
    run.burned.push("bloat_mother".into());
    assert_eq!(crate::oath::steps(&o, run), crate::oath::STEP_FLOOR | crate::oath::STEP_MET | crate::oath::STEP_BURNED);
}

/// Floor recovery already changes gameplay; its exact gain must also survive in
/// the event and trace, so a later log never totals drain losses without gains.
#[test]
fn descent_recovery_is_recorded_and_capped() {
    for (before, expected) in [(31, 36), (38, 40), (40, 40)] {
        let mut g = Game::new_literal(4202);
        g.set_rules_raw(RuleSet { rows: vec![Row::new(Vec::new(), Verb::new("descend"))], ..Default::default() }).unwrap();
        g.send();
        let r = g.run.as_mut().unwrap();
        r.monsters.clear();
        r.hero.max_hp_base = 40;
        r.hero.max_hp = before;
        r.hero.hp = before;
        r.hero.pos = r.floor.stairs_down;
        r.hero.energy = 100;
        let step = g.step(1);
        let r = g.run.as_ref().unwrap();
        assert_eq!(r.depth, 2, "the test must actually descend");
        assert_eq!(r.hero.max_hp, expected);
        assert_eq!(step.snapshot.hero.entity.max_hp, expected);
        let evs: Vec<_> = step.events.iter().filter_map(|e| match e {
            crate::wire::Ev::MaxHp { t, id, max, delta, cause } if *id == HERO_ID && cause == "recovery" => Some((*t, *max, *delta)),
            _ => None,
        }).collect();
        let recorded: Vec<_> = r.max_steps.iter().filter(|s| s.cause == "recovery").map(|s| (s.t, s.max, s.delta)).collect();
        let trace = crate::engine::exit_trace(r, &[]);
        let traced: Vec<_> = trace.max_steps.iter().filter(|s| s.cause == "recovery").map(|s| (s.t, s.max, s.delta)).collect();
        assert_eq!(evs, recorded);
        assert_eq!(traced, recorded);
        if expected > before { assert_eq!(evs, vec![(r.turn, expected, expected - before)]); }
        else { assert!(evs.is_empty(), "no health event at the cap"); }
    }
}
