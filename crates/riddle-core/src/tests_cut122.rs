//! Cut 122 (docs/CUT122_SMOOTH_AND_HONEST.md, core): a smooth Legacy curve, advice priced at its wall, one truth
//! about the King, credit before any rule, hunger with an answer, no silent refusal, the walk home foreseen.
use crate::engine::{ExitTier, Game};
use crate::wire::Ev;

/// Play a resident (`prep` applied) to its first death within `max` runs; the game and the run id.
fn first_death(seed: u64, max: u32, prep: impl Fn(&mut Game)) -> Option<(Game, u32)> {
    let mut g = Game::new_resident(seed);
    prep(&mut g);
    for _ in 0..max {
        g.start_run(None);
        g.run_to_end(200_000);
        let out = g.finish_run()?;
        if out.tier == Some(ExitTier::Death) {
            return Some((g, out.run_id));
        }
    }
    None
}

/// §1: each rank costs 2–3× the last; a rank's effect grows with it; the next rank is never a cliff.
#[test]
fn the_legacy_curve_is_smooth() {
    let p = crate::legacy::ROOT_PRICES;
    for w in p.windows(2) {
        let r = w[1] as f64 / w[0] as f64;
        assert!((2.0..=3.0).contains(&r), "rank {} → {}: ×{r:.2}", w[0], w[1]);
    }
    for id in crate::legacy::IDS {
        for r in 1..=crate::legacy::CAP {
            assert!(crate::legacy::root_effect(id, r) > crate::legacy::root_effect(id, r - 1), "{id} rank {r} adds");
        }
    }
}

/// §4 (blind 9621b19 A: `Guarded · attack nearest · own rule` before any rule written): the school's rows are
/// `default`, a package the player wore is his pick (`picked`), only a row he wrote is `written` (the client's `own rule`).
#[test]
fn credit_before_any_rule_is_never_written() {
    for seed in 1..=4 {
        let Some((mut g, id)) = first_death(seed, 40, |_| {}) else { continue };
        let d = g.death(id).unwrap();
        assert_ne!(d.credit.as_deref(), Some("written"), "s{seed}: the first death, no rule written");
    }
    let p = crate::packages::PkgState::default();
    assert_eq!(crate::packages::credit("stance:steady", &p), "default");
    assert_eq!(crate::packages::credit("stance:guarded", &p), "picked");
    assert_eq!(crate::packages::credit("tactic:kite_archers", &p), "picked");
    assert_eq!(crate::packages::credit("player", &p), "written");
    assert_eq!(crate::packages::credit(crate::pets::TAME_ORIGIN, &p), "default", "the default tame row is no lesson");
}

/// §6 (blind 9621b19 A: the guide's `cadence` counter shown as added, refused `card not owned: cadence`): a counter
/// whose card the lineage does not own says so, in the door's own words, and names the package that owns it.
#[test]
fn an_unowned_counter_says_so_before_the_door_refuses_it() {
    let mut g = Game::new_resident(5);
    g.lineage.kills.insert("goblin_warlord".into());
    crate::packages::recompile(&mut g.lineage);
    g.lineage.facts.insert("foe:mirror_king".into());
    g.lineage.facts.insert("foe:mirror_king:mirror".into());
    let row = crate::rules::Row::new(vec![crate::rules::Cond::t("foe_tag", "boss")], crate::rules::Verb::arg("tactic", "cadence"));
    let own = crate::packages::row_owned(&g.lineage, &row);
    assert!(!own.owned);
    assert_eq!(own.refusal.as_deref(), Some("card not owned: cadence"));
    assert_eq!(own.wear.as_deref(), Some("cadence"), "the King met: Mirror rhythm can be worn");
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, row.clone().from("player"));
    assert_eq!(g.set_rules(set).unwrap_err(), "card not owned: cadence", "the door's words are the wire's");
    for c in g.lineage.counters().iter().filter(|c| c.row.card() == Some("cadence")) {
        assert!(!c.owned);
        assert_eq!(c.refusal.as_deref(), Some("card not owned: cadence"));
    }
    // worn, the card is owned and the counter passes
    crate::packages::equip(&mut g.lineage, "cadence", 0).unwrap();
    assert!(crate::packages::row_owned(&g.lineage, &row).owned);
}

/// §3 (blind 9621b19: `King · slain` beside `died to the King`): in a numbered descent the walls and the King's
/// horizon read the bosses slain in this descent, not the lineage's old kills.
#[test]
fn the_king_is_slain_only_in_this_descent() {
    let mut g = Game::new_resident(7);
    g.lineage.kills.insert("mirror_king".into());
    g.lineage.ended = true;
    g.lineage.best_depth = 34;
    assert_eq!(crate::feats::king_eta_h(&g.lineage), Some(0));
    g.begin_descent(1).unwrap();
    let walls = crate::oath::walls(&g.lineage);
    assert!(walls.iter().all(|w| !w.slain), "a new descent: no boss has fallen in it yet");
    assert_ne!(crate::feats::king_eta_h(&g.lineage), Some(0));
    assert!(crate::feats::king_pct(&g.lineage) < 100);
}

/// §5 (blind 9621b19: `starving −36` with no answer): a ration in the pack — the forecast prices the hp it keeps, and a
/// ration-carrying deep camp loses at most a third of the max hp the hunger takes without one.
#[test]
fn a_ration_keeps_two_thirds_of_the_hunger() {
    let (mut unfed, mut fed, mut met) = (0u32, 0u32, 0u32);
    for seed in 1..=16u64 {
        for ration in [false, true] {
            let mut g = Game::new_resident(seed);
            g.lineage.feats.ration = ration;
            g.start_run(None);
            assert_eq!(g.run.as_ref().unwrap().hero.fed, ration, "the ration is eaten at the send");
            g.descend_to_twist(12, "hunger");
            {
                let h = &mut g.run.as_mut().unwrap().hero;
                h.max_hp = 400;
                h.hp = 400;
            }
            for _ in 0..4000 {
                if g.run.as_ref().is_none_or(|r| r.over.is_some() || r.depth != 12) {
                    break;
                }
                g.tick();
            }
            let r = g.run.as_ref().unwrap();
            if ration { fed += r.hunger_lost } else { unfed += r.hunger_lost; met += u32::from(r.hunger_periods > 0) }
        }
    }
    eprintln!("hunger: unfed −{unfed} · fed −{fed} · {met}/16 sends met it");
    assert!(met >= 8, "the hunger floor bites ({met}/16)");
    assert!(fed * 3 <= unfed, "a ration keeps ≥ 2/3: fed −{fed}, unfed −{unfed}");
    // the forecast names the hunger and what a ration keeps
    let mut g = Game::new_resident(3);
    crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
    g.lineage.best_depth = 24;
    g.lineage.waystones = vec![5, 9, 13];
    g.lineage.start = 13;
    let f = g.forecast();
    if let Some(h) = f.hunger {
        assert!(h.kept > 0.0 && h.fed < h.unfed, "{h:?}");
        assert!(h.fed * 3.0 <= h.unfed + 1e-9, "{h:?}");
    }
    // the player buys one by hand
    g.lineage.gold = 10_000;
    let p = crate::feats::buy_ration(&mut g.lineage).unwrap();
    assert!(p > 0 && g.lineage.feats.ration);
    assert_eq!(crate::feats::buy_ration(&mut g.lineage).unwrap_err(), "ration packed");
}

/// §7: a committed walk home is foreseen once, with its length.
#[test]
fn the_walk_home_is_foreseen() {
    let mut seen = 0;
    for seed in 1..=6u64 {
        let mut g = Game::new_resident(seed);
        g.start_run(None);
        for _ in 0..200_000 {
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                break;
            }
            g.tick();
        }
        let evs = g.events.clone();
        let walks: Vec<_> = evs.iter().filter_map(|e| if let Ev::Homeward { ticks, .. } = e { Some(*ticks) } else { None }).collect();
        eprintln!("s{seed} walks {walks:?} over {:?}", g.run.as_ref().and_then(|r| r.over));
        assert!(walks.len() <= 1, "once a run: {walks:?}");
        seen += walks.len();
    }
    assert!(seen > 0, "some run walked home");
}

/// §2 (blind 9621b19: fixes and TRYs that made the forecast worse): over 16 seeds of death screens, no fix offered as
/// one — a patch, the gem, the tactic pick — is worse at the death's wall than the set as it is, past the noise,
/// measured again here on the camp's panels.
#[test]
fn advice_never_lies_at_its_wall() {
    use crate::forecast::{camp_panel, FORECAST_SIMS};
    let (mut offered, mut traded, mut screens) = (0, 0, 0);
    for seed in 1..=16u64 {
        let Some((mut g, id)) = first_death(seed, 60, |g| {
            crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
            g.lineage.pkg.pen_open = true;
        }) else {
            continue;
        };
        screens += 1;
        let _ = g.death(id).unwrap();
        let patches = g.death_deltas(id).unwrap();
        let d = g.death(id).unwrap();
        let camp = crate::trace::camp_state(&g);
        let rules = camp.lineage.rules().clone();
        let base = camp_panel(&camp, &rules, FORECAST_SIMS);
        let max_rows = camp.lineage.max_rows();
        let check = |with: &[crate::forecast::SimResult], what: &str| {
            let p = crate::trace::wall_price(&base, with, d.depth.max(1));
            let past = p.past_to - p.past_from;
            let death = p.death_to - p.death_from;
            assert!(past >= -p.past_pm - 1e-3, "s{seed} {what}: past D{} {:.2}→{:.2} ±{:.2}", d.depth, p.past_from, p.past_to, p.past_pm);
            assert!(death <= p.death_pm + 1e-3, "s{seed} {what}: death {:.2}→{:.2} ±{:.2}", p.death_from, p.death_to, p.death_pm);
        };
        for p in patches.iter().filter(|p| p.insert_at >= 0 && p.buys.is_none()) {
            let w = p.whole.as_ref().expect("measured");
            assert!(w.price.is_some(), "s{seed}: every measured patch is priced at its wall");
            if w.trade_off {
                traded += 1;
                assert!(!p.gem, "s{seed}: a trade-off is never the gem");
                continue;
            }
            offered += 1;
            let set = crate::offline::apply_patch(&rules, p, max_rows);
            let edited = crate::forecast::edited_game(&camp, &set);
            let with = camp_panel(&edited, edited.lineage.rules(), FORECAST_SIMS);
            check(&with, &format!("patch {}", p.row.describe()));
        }
        if let Some(pick) = d.pick.as_ref().filter(|l| l.kind == "tactic") {
            assert!(!pick.pending && pick.price.is_some(), "s{seed}: the pick is priced once deltas land");
            if pick.trade_off {
                traded += 1;
            } else {
                offered += 1;
                let mut worn = camp.sim_clone();
                crate::packages::take_fix(&mut worn.lineage, pick.id.as_deref().unwrap(), pick.variant.unwrap_or(0) as u8).unwrap();
                crate::packages::recompile(&mut worn.lineage);
                let set = worn.lineage.rules().clone();
                check(&camp_panel(&worn, &set, FORECAST_SIMS), &format!("pick {}", pick.text));
            }
        }
    }
    eprintln!("advice: {screens} death screens · {offered} offered · {traded} trade-offs");
    assert!(screens >= 12, "16 seeds of death screens ({screens})");
}
