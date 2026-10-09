//! Cut 118 (docs/CUT118_IDLE_LESSONS.md, Round 2): boss tokens, trials, away finds, sinks after Kit complete, swift
//! floors, feats that light the ladder, the hero's wish, and the wire fields the client asked for.
use crate::engine::{ExitTier, Game};
use crate::feats::{self, Rule};

/// A camp past the Mother: both bosses slain, the D5 and D9 stones lit, the scout hired.
fn deep_camp(seed: u64) -> Game {
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

/// A run sent now and ended at once: `slain` band bosses on its kills, `tier` its exit.
fn end_run(g: &mut Game, slain: &[&str], tier: ExitTier) -> u32 {
    if g.run.is_none() {
        g.start_run(None);
    }
    let run = g.run.as_mut().unwrap();
    for k in slain {
        run.boss_kills.push((1, k.to_string()));
        *g.lineage.kill_counts.entry(k.to_string()).or_insert(0) += 1;
    }
    let run = g.run.as_mut().unwrap();
    run.over = Some(tier);
    let id = run.id;
    g.finish_run();
    id
}

/// §1: each band boss slain banks one token (kept across heirs, capped); spending one makes the next send start at
/// the deepest lit stone above him with no wall hold (it pushes to him); the token is gone; a slay is a feat.
#[test]
fn a_token_seeks_its_boss_from_his_stone() {
    let mut g = deep_camp(3);
    assert!(g.seek_boss("bloat_mother").is_err(), "no token yet");
    for _ in 0..5 {
        end_run(&mut g, &["bloat_mother"], ExitTier::Bank);
    }
    assert_eq!(g.lineage.feats.tokens.get("bloat_mother"), Some(&feats::TOKEN_CAP), "capped");
    g.lineage.new_heir();
    assert_eq!(g.lineage.feats.tokens.get("bloat_mother"), Some(&feats::TOKEN_CAP), "kept across heirs");
    assert!(g.seek_boss("lich").is_err());
    assert!(g.seek_boss("goblin").is_err());
    g.seek_boss("bloat_mother").unwrap();
    let w = g.lineage().feats.unwrap();
    assert_eq!(w.seek.as_deref(), Some("bloat_mother"));
    assert_eq!(w.tokens[0].stone, 9);
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    assert_eq!(run.start, 9, "the deepest lit stone at or above D13");
    assert_eq!(run.wall_hold, None);
    assert_eq!(g.lineage.feats.tokens.get("bloat_mother"), Some(&(feats::TOKEN_CAP - 1)));
    assert!(g.lineage.feats.seek.is_none(), "one send");
    let feats0 = g.lineage.feats.feats;
    end_run(&mut g, &["bloat_mother"], ExitTier::Bank);
    assert_eq!(g.lineage.feats.feats, feats0 + 1, "a sought boss slain is a feat");
    assert!(g.lineage.feats.news.iter().any(|n| n.k == "seek" && n.day >= 1));
    // the next send is in order again, from the lineage's own start
    g.start_run(None);
    assert_eq!(g.run.as_ref().unwrap().start, 1);
}

/// §1: the forecast prices each banked token's seek (reach and pass his floor, from his stone).
#[test]
fn the_forecast_prices_a_seek() {
    let mut g = deep_camp(5);
    end_run(&mut g, &["bloat_mother"], ExitTier::Bank);
    let opts = g.seek_forecast();
    assert_eq!(opts.len(), 1);
    let o = &opts[0];
    assert_eq!((o.boss.as_str(), o.depth, o.stone, o.tokens), ("bloat_mother", 13, 9, 1));
    assert!(o.sims > 0 && (0.0..=1.0).contains(&o.reach) && o.past <= o.reach);
}

/// §2: one trial a week from (week, seed); the last four open; every boss × restriction answerable by packages
/// that arrive (the probe); a clear pays a capped Legacy and a mark; breaking the bar fails it.
#[test]
fn trials_are_weekly_answerable_and_pay_capped() {
    // the probe: every restriction of every boss a trial can draw is answered by arrived-able packages
    for (boss, _) in &crate::descent::BOSS_DEPTHS[..5] {
        for rule in feats::rules_for(boss) {
            let ans = feats::answers(boss, &rule);
            assert!(!ans.is_empty(), "{boss} {rule:?}");
            for a in &ans {
                assert!(crate::packages::def(a).is_some(), "{a} is a package");
            }
            if let Rule::Affixes(a, b) = rule {
                assert_ne!(a, b);
                for x in [a, b] {
                    assert!(crate::descent::affix_breakers(boss, x).iter().any(|p| crate::packages::def(p).is_some()), "{boss} {x:?} has a breaker");
                }
            }
            if let Rule::Barred(t) = &rule {
                assert!(!ans.contains(&t.as_str()));
                assert!(ans.iter().any(|a| crate::packages::def(a).is_some_and(|d| d.kind == crate::packages::Kind::Stance)), "a stance answers");
            }
        }
    }
    let mut g = deep_camp(7);
    g.lineage.clock_s = 5 * feats::WEEK_S + 3600;
    assert_eq!(feats::open_weeks(&g.lineage), vec![2, 3, 4, 5]);
    // a pure draw
    assert_eq!(feats::trial_of(&g.lineage, 4), feats::trial_of(&g.lineage, 4));
    assert!(feats::set_trial(&mut g.lineage, Some(1)).is_err(), "closed");
    // find an open week whose boss the camp has slain
    let week = (2..=5).find(|w| feats::trial_of(&g.lineage, *w).is_some_and(|t| t.open)).or_else(|| {
        // (else every boss open: the lineage deeper)
        g.lineage.best_depth = 29;
        for (k, _) in crate::descent::BOSS_DEPTHS {
            g.lineage.kills.insert(k.into());
        }
        (2..=5).find(|w| feats::trial_of(&g.lineage, *w).is_some_and(|t| t.open))
    });
    let week = week.expect("an open trial");
    let t = feats::trial_of(&g.lineage, week).unwrap();
    let wire = g.lineage().feats.unwrap();
    assert_eq!(wire.trials.len(), 4);
    let tw = wire.trials.iter().find(|x| x.week == week).unwrap();
    assert!(tw.open && !tw.cleared && tw.legacy == feats::TRIAL_LEGACY);
    // capped: ≤ the return pick's largest, ≤ a day's Legacy
    assert!(feats::TRIAL_LEGACY <= crate::returns::LEGACY_POINTS[4]);
    // opted in, a send at camp does not play it; one while away does
    g.set_trial(Some(week)).unwrap();
    g.start_run(None);
    assert!(feats::goal_of(&g.lineage, g.run.as_ref().unwrap().id).is_none(), "a trial plays while away");
    end_run(&mut g, &[], ExitTier::Bank);
    assert_eq!(g.lineage.feats.trial, Some(week));
    g.offline = true;
    // (a barred tactic: the set sent wears none of it)
    if let Rule::Barred(x) = &t.rule {
        assert!(!crate::packages::wears(g.lineage.rules(), x));
    }
    g.start_run(None);
    let run = g.run.as_ref().unwrap();
    let goal = feats::goal_of(&g.lineage, run.id).cloned().expect("the trial's send");
    assert_eq!(goal.week, Some(week));
    assert_eq!(run.start, feats::stone_for(&g.lineage, t.depth, run.route));
    if let Rule::Affixes(a, b) = t.rule {
        let on: Vec<_> = run.affixes.iter().filter(|(k, _)| *k == t.boss).map(|(_, x)| *x).collect();
        assert_eq!(on, vec![a, b], "the trial's two affixes");
    }
    let legacy0 = crate::legacy::current(&g.lineage).map_or(0, |b| b.points);
    end_run(&mut g, &[t.boss.as_str()], ExitTier::Bank);
    assert!(g.lineage.feats.trials.contains(&week));
    assert_eq!(g.lineage.feats.trial_marks, 1);
    let legacy1 = crate::legacy::current(&g.lineage).unwrap().points;
    assert!(legacy1 >= legacy0 + feats::TRIAL_LEGACY);
    assert!(g.lineage.feats.news.iter().any(|n| n.k == "trial"));
    assert!(g.set_trial(Some(week)).is_err(), "cleared");
    let mut r = crate::wire::ReturnReport::default();
    feats::on_report(&mut g.lineage, &mut r);
    assert!(r.feats.iter().any(|n| n.k == "trial" && n.day >= 1), "the report names the trial cleared");
}

/// §2: a trial whose bar the set sent breaks is no clear.
#[test]
fn a_broken_bar_fails_the_trial() {
    let mut g = deep_camp(11);
    g.lineage.best_depth = 29;
    for (k, _) in crate::descent::BOSS_DEPTHS {
        g.lineage.kills.insert(k.into());
    }
    let (week, t) = (0..400u64)
        .find_map(|w| {
            g.lineage.clock_s = w * feats::WEEK_S;
            let t = feats::trial_of(&g.lineage, w as u32)?;
            matches!(t.rule, Rule::Barred(_)).then_some((w as u32, t))
        })
        .expect("a barred trial");
    let Rule::Barred(x) = t.rule.clone() else { unreachable!() };
    g.lineage.pkg.owned.insert(x.clone());
    crate::packages::equip(&mut g.lineage, &x, 0).unwrap();
    assert!(crate::packages::wears(g.lineage.rules(), &x));
    g.set_trial(Some(week)).unwrap();
    g.offline = true;
    g.start_run(None);
    end_run(&mut g, &[t.boss.as_str()], ExitTier::Bank);
    assert!(!g.lineage.feats.trials.contains(&week));
    assert!(g.lineage.feats.news.iter().any(|n| n.k == "trial_failed" && n.text.contains("rule broken")));
}

fn absence_finds(save: &str, hours: u64) -> (u32, crate::feats::FindsReveal) {
    let mut g = Game::load(save).unwrap();
    let r = crate::offline::run_offline_counts(&mut g, hours * 3600);
    (r.runs, r.finds.unwrap_or_default())
}

/// §3: each completed run while away seals one find; the second table opens after 6 h; one reveal, best first, and
/// the count. Gate: on the median seed, 8 h brings ≥ 1.8× the finds of 4 h.
#[test]
fn away_finds_scale_with_time() {
    let mut ratios = Vec::new();
    for seed in [1u64, 2, 3, 4, 5] {
        let mut g = Game::new_resident(seed);
        crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
        crate::offline::run_offline_counts(&mut g, 3600);
        let save = g.save();
        let (runs4, f4) = absence_finds(&save, 4);
        let (runs8, f8) = absence_finds(&save, 8);
        for (runs, f) in [(runs4, &f4), (runs8, &f8)] {
            assert!(f.sealed >= runs.saturating_sub(1) && f.sealed <= runs, "one a completed run: {} of {runs}", f.sealed);
            assert_eq!(f.finds.len() as u32, f.sealed);
            assert_eq!(f.best.as_ref(), f.finds.first());
            assert!(f.finds.windows(2).all(|w| w[0].rank >= w[1].rank), "best first");
        }
        assert!(f4.finds.iter().all(|x| x.table == 1), "no second table before 6 h");
        assert!(f8.finds.iter().any(|x| x.table == 2), "the second table after 6 h");
        ratios.push(f8.sealed as f64 / f4.sealed.max(1) as f64);
    }
    ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = ratios[ratios.len() / 2];
    eprintln!("finds 8 h / 4 h by seed: {ratios:?}");
    assert!(median >= 1.8, "8 h vs 4 h finds, median {median:.2} ({ratios:?})");
}

/// §3: the find log completes sets; each set's bonus is ≤ 2 % (rest, xp) or +1 Legacy a return; the sealed count
/// rides the lineage for the away screen.
#[test]
fn find_sets_give_small_bonuses() {
    let mut g = Game::new_resident(2);
    assert_eq!(feats::bonus_pct(&g.lineage, "rest"), 0);
    let rest0 = g.rest_after(4000, ExitTier::Bank);
    let all: Vec<feats::Find> = feats::SETS.iter().flat_map(|(id, _, pieces, _)| pieces.iter().map(move |p| feats::Find { id: (*p).into(), kind: "piece".into(), name: (*p).into(), rank: 3, set: Some((*id).into()), table: 2 })).collect();
    g.lineage.feats.sealed = all;
    assert_eq!(g.lineage().feats.unwrap().sealed, 9, "the sealed count on the away screen");
    let mut r = crate::wire::ReturnReport::default();
    feats::on_report(&mut g.lineage, &mut r);
    let f = r.finds.unwrap();
    assert_eq!(f.sets.len(), 3);
    assert_eq!(r.feats.iter().filter(|n| n.k == "set").count(), 3);
    for what in ["rest", "xp"] {
        assert!(feats::bonus_pct(&g.lineage, what) <= 2 && feats::bonus_pct(&g.lineage, what) > 0);
    }
    let rest1 = g.rest_after(4000, ExitTier::Bank);
    assert!(rest1 < rest0 && rest1 * 100 >= rest0 * 98, "{rest0} → {rest1}");
    assert!(g.lineage().feats.unwrap().sets.iter().all(|s| s.done));
    g.lineage.feats.off = true;
    assert_eq!(g.rest_after(4000, ExitTier::Bank), rest0, "pinned off");
}

/// A camp whose kit is complete, the apprentice hired, a deep purse.
fn kitted(seed: u64) -> Game {
    let mut g = deep_camp(seed);
    crate::tree::grant(&mut g.lineage, &["apprentice"]);
    for s in crate::kit::KIT_SLOTS {
        g.lineage.kit.insert(s.into(), crate::kit::mults(s).len() as u32);
    }
    g.lineage.gold_move(60_000, "test income");
    g
}

/// §4: after Kit complete the apprentice's sinks (a ration a send, the tithe on the hour) — adds only: each takes
/// gold and returns none; the tithe's rate falls and shows; the order `off` and an incomplete kit buy nothing.
#[test]
fn sinks_add_never_return_gold() {
    let mut g = kitted(4);
    assert!(feats::kit_complete(&g.lineage));
    let (gold0, legacy0) = (g.lineage.gold, crate::legacy::current(&g.lineage).map_or(0, |b| b.points));
    let p0 = feats::tithe_price(&g.lineage);
    crate::tree::at_hour(&mut g);
    let bought = g.lineage.feats.tithed;
    assert!(bought > 0 && bought <= feats::TITHE_PER_HOUR);
    assert!(g.lineage.gold < gold0, "the tithe takes gold");
    assert_eq!(crate::legacy::current(&g.lineage).unwrap().points, legacy0 + bought);
    assert!(feats::tithe_price(&g.lineage) > p0, "the rate falls");
    assert_eq!(g.lineage.gold_tally.get("sinks").copied().unwrap_or(0), -(gold0 - g.lineage.gold) as i64);
    let w = g.lineage().feats.unwrap();
    assert!(w.sinks.iter().any(|s| s.id == "tithe" && s.line.starts_with("1 Legacy / $")));
    // a ration a send: gold out, max hp in, for that run alone
    let gold1 = g.lineage.gold;
    let base = {
        let mut h = Game::load(&g.save()).unwrap();
        h.lineage.orders.sink = "off".into();
        h.start_run(None);
        h.run.as_ref().unwrap().hero.max_hp
    };
    g.start_run(None);
    assert!(g.lineage.gold < gold1);
    let hp = g.run.as_ref().unwrap().hero.max_hp;
    assert!(hp > base && hp <= base + (base * feats::RATION_PCT / 100).max(feats::RATION_MIN_HP), "{base} → {hp}");
    assert!(!g.lineage.feats.ration, "eaten");
    // off: nothing
    let mut h = kitted(4);
    h.lineage.orders.sink = "off".into();
    let gold = h.lineage.gold;
    crate::tree::at_hour(&mut h);
    assert_eq!((h.lineage.feats.tithed, h.lineage.gold), (0, gold));
    // the kit incomplete: nothing
    let mut h = kitted(4);
    h.lineage.kit.insert("pack".into(), 0);
    crate::tree::at_hour(&mut h);
    assert_eq!(h.lineage.feats.tithed, 0);
    // by hand, and the survey work
    let mut h = kitted(4);
    let gold = h.lineage.gold;
    assert_eq!(h.tithe(2).unwrap(), 2);
    assert!(h.lineage.gold < gold);
    assert!(!h.lineage.unlocks.contains("route2"));
    h.buy_survey().unwrap();
    assert!(h.lineage.unlocks.contains("route2") && h.buy_survey().is_err());
    // the order is a standing order on the wire
    let mut o = h.lineage.standing_orders();
    assert_eq!(o.sink.as_deref(), Some("both"));
    o.sink = Some("tithe".into());
    h.set_orders(&o).unwrap();
    assert_eq!(h.lineage.orders.sink, "tithe");
    o.sink = Some("hoard".into());
    assert!(h.set_orders(&o).is_err());
}

/// §5: three clears of a band boss make the floors above him swift — announced once, shown on the snapshot.
#[test]
fn three_clears_make_the_floors_above_swift() {
    let mut g = deep_camp(6);
    g.lineage.kill_counts.clear();
    for i in 0..4 {
        end_run(&mut g, &["goblin_warlord"], ExitTier::Bank);
        let swift = feats::swift_to(&g.lineage, crate::descent::Route::BASE);
        assert_eq!(swift, if i >= 2 { 8 } else { 0 }, "after {} clears", i + 1);
    }
    assert_eq!(g.lineage.feats.news.iter().filter(|n| n.k == "swift").count(), 1, "announced once");
    assert!(g.lineage.feats.news.iter().any(|n| n.text == "D1–7 swift"));
    g.start_run(None);
    assert!(g.snapshot().swift, "D1 is swift");
    g.descend_to(8);
    assert!(!g.snapshot().swift, "the boss's floor is not");
}

/// §6: a feat lights a system whose trigger has come before its age; the time fallback stands; `lit_by` names it.
#[test]
fn a_feat_lights_the_ladder_early() {
    let mut g = Game::new_resident(8);
    g.lineage.kills.insert("goblin_warlord".into());
    g.lineage.reveal_left = 5;
    crate::systems::update(&mut g.lineage, false);
    assert!(!g.lineage.systems.contains("tactics"), "the tactics wait their 4 h");
    let before = g.lineage.systems.clone();
    g.lineage.feats.feats = 1;
    g.lineage.feats.last_feat = "trial: Warlord".into();
    crate::systems::update(&mut g.lineage, false);
    let lit: Vec<String> = g.lineage.systems.difference(&before).cloned().collect();
    assert!(!lit.is_empty(), "the feat lit the next waiting rung");
    assert_eq!(g.lineage.feats.feats_used, 1);
    for id in &lit {
        let info = crate::systems::wire(&g.lineage).into_iter().find(|s| s.id == *id).unwrap();
        assert_eq!(info.lit_by.as_deref(), Some("feat: trial: Warlord"), "{id}");
        assert!(!info.trigger.is_empty());
    }
    // one feat, one rung: the next waits its age (or the next feat)
    let before = g.lineage.systems.clone();
    crate::systems::update(&mut g.lineage, false);
    assert_eq!(g.lineage.systems, before);
    g.lineage.feats.feats = 2;
    g.lineage.feats.last_feat = "slay Mother".into();
    crate::systems::update(&mut g.lineage, false);
    assert!(g.lineage.systems.len() > before.len());
    // the trigger's own systems name their trigger
    let send = crate::systems::wire(&g.lineage).into_iter().find(|s| s.id == "death");
    assert!(send.is_some());
}

/// §7: one wish at a time, upside only — it waits (never decays), granting pays a small Legacy for a small price,
/// the next comes a day on. IDLE (no grant) loses nothing.
#[test]
fn the_heros_wish_waits_and_pays() {
    let mut g = deep_camp(9);
    assert!(feats::wish(&g.lineage).is_none(), "none before the first return");
    let r = crate::offline::run_offline_counts(&mut g, 3 * 3600);
    let _ = r;
    let w = g.lineage().feats.unwrap().wish.expect("a wish after a return");
    // it waits: another long absence leaves it as it was
    crate::offline::run_offline_counts(&mut g, 30 * 3600);
    let again = g.lineage().feats.unwrap().wish.unwrap();
    assert_eq!((w.id.clone(), w.legacy), (again.id.clone(), again.legacy));
    let (gold, legacy) = (g.lineage.gold, crate::legacy::current(&g.lineage).unwrap().points);
    g.grant_wish().unwrap();
    assert_eq!(g.lineage.gold, gold - again.price);
    assert_eq!(crate::legacy::current(&g.lineage).unwrap().points, legacy + again.legacy);
    assert!(feats::wish(&g.lineage).is_none(), "the next a day on");
    assert!(g.grant_wish().is_err());
    g.lineage.clock_s += feats::WISH_EVERY_S;
    assert!(feats::wish(&g.lineage).is_some());
}

/// §8: the wire fields — the King's horizon, the pick's default, the wall's roster preview, a news line's day.
#[test]
fn the_wire_fields_the_client_asked_for() {
    let mut g = deep_camp(12);
    crate::offline::run_offline_counts(&mut g, 3600);
    let r = crate::offline::run_offline_quick(&mut g, 4 * 3600);
    let l = g.lineage();
    let eta = l.king_eta_h.expect("a pace");
    assert!(eta > 0);
    assert!(l.king_pct > 0 && l.king_pct < 100);
    let pick = r.pick.expect("a pick");
    assert_eq!(pick.default.as_deref(), Some("legacy"));
    let walls = l.walls;
    assert!(walls.iter().all(|w| w.guard.is_some() && w.guard_counter.is_some()));
    for w in walls.iter().filter(|w| w.boss != "mirror_king") {
        assert!(w.affix.is_some() && w.affix_counter.is_some(), "{}", w.boss);
    }
    assert!(r.exits.iter().flat_map(|e| e.news.iter()).all(|n| n.day.is_some_and(|d| d >= 1)));
    assert!(r.exits.iter().any(|e| !e.news.is_empty()));
    // a slain King: no horizon left
    g.lineage.kills.insert("mirror_king".into());
    assert_eq!(feats::king_eta_h(&g.lineage), Some(0));
}

/// IDLE never spends a token, opts into a trial or grants a wish: a fortnight's absences leave the order, the trial
/// and the wishes untouched (tokens only bank).
#[test]
fn idle_never_spends() {
    let mut g = Game::new_resident(13);
    crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
    for _ in 0..6 {
        crate::offline::run_offline_counts(&mut g, 8 * 3600);
    }
    let f = &g.lineage.feats;
    assert!(f.seek.is_none() && f.trial.is_none() && f.goal.is_none());
    assert_eq!((f.wishes, f.tithed, f.trials.len()), (0, 0, 0));
    assert!(f.log.values().sum::<u32>() > 0, "finds opened");
}

/// The 307dbed send hash: with every Cut 118 system pinned off it is Cut 117's aa808dca72425139 exactly (the graves
/// moved the recorded one, `fixtures/sends_307dbed.txt`).
#[test]
fn the_307dbed_hash_holds_with_the_systems_pinned_off() {
    let want = 0xaa80_8dca_7242_5139u64;
    let mut g = Game::load(include_str!("fixtures/save_307dbed.json")).unwrap();
    g.lineage.feats.off = true;
    assert_eq!(format!("{:016x}", crate::tests::sends_hash(&mut g, 10)), format!("{want:016x}"));
}

// ---------------------------------------------------------------- owner amendments: heroes yolo; death is progress

/// Amendment 1: the scout's wall order is retired — every send pushes, the forecast holds nothing.
#[test]
fn the_wall_order_is_retired() {
    let mut g = deep_camp(21);
    g.lineage.orders.wall = "bank".into();
    assert_eq!(crate::tree::wall_hold(&g.lineage), None);
    assert!(g.forecast().hold.is_none());
}

/// A death at the band boss on `depth` with `carry` lost: the boss on the floor left at `hp_pct`.
fn die_at(g: &mut Game, depth: u32, carry: i32, hp_pct: i32) -> u32 {
    g.start_run(None);
    g.descend_to(depth);
    let run = g.run.as_mut().unwrap();
    run.loot = carry;
    if let Some(m) = run.monsters.iter_mut().find(|m| m.is_boss()) {
        m.hp = m.max_hp * hp_pct / 100;
    }
    run.death_cause = Some(run.route.boss(depth).unwrap_or("goblin").into());
    run.over = Some(ExitTier::Death);
    let id = run.id;
    g.finish_run();
    id
}

/// Amendment 2–4 and amendment 2's deeds and titles: each death at a band boss adds a siege try (a capped edge on
/// him alone, the next heir's run carries it), a grave of the lost carry, a stone with an epitaph, a deed's Legacy;
/// the boss slain clears the siege, names the heirs and the line's title.
#[test]
fn death_is_progress_siege_grave_and_stone() {
    let mut g = deep_camp(22);
    let mut lost = 0;
    let legacy0 = crate::legacy::current(&g.lineage).map_or(0, |b| b.points);
    for i in 0..7 {
        let id = die_at(&mut g, 13, 300, 60 - i * 5);
        lost += g.last_exit.as_ref().map_or(0, |x| x.carried - x.kept);
        let s = &g.lineage.feats.siege["bloat_mother"];
        assert_eq!(s.tries, i as u32 + 1);
        assert_eq!(feats::siege_edge(&g.lineage, "bloat_mother"), ((i as u32 + 1) * feats::SIEGE_PCT).min(feats::SIEGE_CAP));
        let d = g.death(id).expect("the death");
        let m = d.memorial.expect("the memorial");
        assert!(m.lead.contains(&format!("Mother try {}", i + 1)) && m.lead.starts_with('+'), "{}", m.lead);
        assert!(m.epitaph.contains("Mother") && m.epitaph.contains("D13"));
    }
    let s = &g.lineage.feats.siege["bloat_mother"];
    assert!(s.best_pct <= 30, "the best hp he was left on: {}", s.best_pct);
    assert_eq!(feats::siege_edge(&g.lineage, "lich"), 0, "against that boss alone");
    assert_eq!(g.lineage.feats.stones.len(), 7);
    assert!(crate::legacy::current(&g.lineage).unwrap().points >= legacy0 + 7 * feats::DEED_LEGACY);
    // one grave a floor, the lost carry kept whole; no gold made
    assert_eq!(g.lineage.feats.graves.len(), 1);
    let grave = g.lineage.feats.graves[0].gold;
    assert!(grave > 0);
    // (each heir walked D13 and brought the grave before his home: what lies there is the last carry lost)
    let back = g.lineage.gold_tally.get("recovered").copied().unwrap_or(0);
    assert_eq!(i64::from(grave) + back, i64::from(lost), "no gold made: the graves hold the lost carry exactly");
    assert_eq!(grave, 300);
    let w = g.lineage().feats.unwrap();
    assert_eq!(w.siege[0].tries, 7);
    assert_eq!(w.graves[0].gold, grave);
    assert_eq!(w.graveyard.len(), 7);
    // the next heir's run carries the edge
    g.start_run(None);
    assert!(g.run.as_ref().unwrap().siege.iter().any(|(k, p)| k == "bloat_mother" && *p == feats::SIEGE_CAP));
    // he reaches the floor: the grave comes home (`recovered`), exactly its gold; the Mother slain: the siege won
    g.descend_to(13);
    let tally0 = g.lineage.gold_tally.get("recovered").copied().unwrap_or(0);
    let gold1 = g.lineage.gold;
    let run = g.run.as_mut().unwrap();
    run.loot = 0;
    run.boss_kills.push((1, "bloat_mother".into()));
    run.over = Some(ExitTier::Return);
    g.finish_run();
    let recovered = g.lineage.gold_tally.get("recovered").copied().unwrap_or(0) - tally0;
    assert_eq!(recovered, i64::from(grave));
    assert!(g.lineage.gold >= gold1 + grave);
    assert!(g.lineage.feats.graves.is_empty());
    assert!(!g.lineage.feats.siege.contains_key("bloat_mother"));
    assert_eq!(g.lineage.feats.worn["bloat_mother"].len(), 7);
    assert!(g.lineage.titles.contains(&"Motherbane".to_string()));
    assert_eq!(feats::siege_edges(&g.lineage), vec![("bloat_mother".to_string(), feats::TITLE_PCT)], "a title's lasting edge");
    let mut r = crate::wire::ReturnReport::default();
    feats::on_report(&mut g.lineage, &mut r);
    for k in ["siege", "recovered", "siege_won", "title"] {
        assert!(r.feats.iter().any(|n| n.k == k), "{k}: {:?}", r.feats);
    }
}

/// Amendment 2: the heir order picks among three offered heirs with no prompt — one always answers the killer; the
/// order is a standing order; the swap is the offer's `set_trait` before the next send.
#[test]
fn the_heir_order_picks_without_a_prompt() {
    let mut g = Game::new_resident(23);
    g.lineage.heir = 4;
    // (no shape ships today — `traits.json` is empty — so the offer is drawn from the generator's table here)
    let pool: Vec<(crate::traits::Shape, bool)> = crate::traits::generated().into_iter().map(|s| (s, true)).collect();
    for order in feats::HEIR_ORDERS {
        let mut h = Game::load(&g.save()).unwrap();
        h.lineage.orders.heir = order.into();
        h.lineage.graveyard.push(crate::wire::Grave { heir: 4, depth: 13, cause: "bloat_mother".into(), deeds: Vec::new(), death_id: None });
        h.lineage.heir = 5;
        crate::traits::wake_from(&mut h.lineage, &pool);
        let offer = h.lineage.heirs.offer.clone();
        assert_eq!(offer.len(), 3, "{order}");
        assert!(offer.iter().any(|c| c.shape.gift == crate::traits::Gift::Guard), "one answers the boss: {order}");
        let born = h.lineage.heirs.born.unwrap();
        if order == "answer" {
            assert_eq!(born.gift, crate::traits::Gift::Guard);
            let line = feats::heir_line(&h.lineage).unwrap();
            assert!(line.contains("answers the Mother"), "{line}");
        }
        if order == "strongest" {
            assert_eq!(born.tier, offer.iter().map(|c| c.shape.tier).max().unwrap());
        }
        // the swap: another offered heir before the send
        let other = offer.iter().find(|c| c.shape != born).unwrap().shape;
        h.set_trait(&other.chip()).unwrap();
        assert_eq!(h.lineage.heirs.born, Some(other));
    }
    let mut o = g.lineage.standing_orders();
    assert_eq!(o.heir.as_deref(), Some("answer"));
    o.heir = Some("strongest".into());
    g.set_orders(&o).unwrap();
    o.heir = Some("eldest".into());
    assert!(g.set_orders(&o).is_err());
    assert!(crate::systems::SYSTEMS.iter().any(|s| s.id == "heirs" && s.trigger == "first death"));
}

/// Owner amendment 2 (probe): deeds redistribute Legacy, not inflate it — over an IDLE fortnight the siege tries'
/// deed Legacy is ≤ 10 % of all Legacy earned. Run with `--ignored --nocapture` for the numbers.
#[test]
#[ignore = "probe: a fortnight per seed"]
fn deeds_stay_within_ten_percent_of_legacy() {
    for seed in [1u64, 2, 3, 4] {
        let mut g = Game::new_resident(seed);
        crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
        for _ in 0..42 {
            crate::offline::run_offline_counts(&mut g, 8 * 3600);
        }
        let b = crate::legacy::current(&g.lineage).unwrap();
        let total = b.points + b.spent;
        let deeds = g.lineage.feats.deed_legacy;
        eprintln!("seed {seed}: legacy {total} · deeds {deeds} ({:.1}%) · best D{} · tries {:?} · titles {:?} · graves ${} recovered ${}", 100.0 * deeds as f64 / total.max(1) as f64, g.lineage.best_depth, g.lineage.feats.siege.iter().map(|(k, s)| (k.clone(), s.tries)).collect::<Vec<_>>(), g.lineage.feats.titles, g.lineage.feats.graves.iter().map(|x| x.gold).sum::<i32>(), g.lineage.gold_tally.get("recovered").copied().unwrap_or(0));
        assert!(deeds * 10 <= total);
    }
}
