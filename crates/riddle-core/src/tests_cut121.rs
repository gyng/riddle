//! Cut 121 (docs/CUT121_PACE_AND_TRUST.md, core): the death's killer told as one story, the report's income apart
//! from the workers' spending, the forecast's bands, the Legacy curve, Ascension's start.
use crate::engine::{ExitTier, Game};
use crate::feats;

fn camp(seed: u64) -> Game {
    let mut g = Game::new_resident(seed);
    crate::tree::grant(&mut g.lineage, &["porter", "scout"]);
    g.lineage.best_depth = 14;
    g.lineage.waystones = vec![5, 9];
    g
}

/// A death on `depth` by `cause`: `summoned` puts a summoned foe of that kind beside the boss and makes it the
/// killer; `boss_in_view` sets the floor boss's tile visible (or not) at the blow.
fn die_by(g: &mut Game, depth: u32, cause: &str, summoned: bool, boss_in_view: Option<bool>) -> u32 {
    g.start_run(None);
    g.descend_to(depth);
    let run = g.run.as_mut().unwrap();
    let boss = run.monsters.iter().position(|m| m.is_boss()).expect("a band boss on the floor");
    if let Some(seen) = boss_in_view {
        let p = run.monsters[boss].pos;
        let map = &mut run.floor.map;
        let i = (p.y as usize) * map.w as usize + p.x as usize;
        map.visible[i] = seen;
        run.death_source = crate::turn::hazard_source(run);
    }
    if summoned {
        let mut m = run.monsters[boss].clone();
        m.kind = cause.into();
        m.summoned = true;
        m.id = 9_999;
        run.monsters.push(m);
        let i = run.monsters.len() - 1;
        run.death_summoner = crate::turn::summoner_of(run, i);
    }
    run.death_cause = Some(cause.into());
    run.hero.hp = 0;
    run.over = Some(ExitTier::Death);
    let id = run.id;
    g.finish_run();
    id
}

/// §2 (blind 1a7d834 B: `goblin · D8` beside `fell to the Warlord`): the epitaph names the killing blow's source, as
/// the header does — a goblin's blow on the Warlord's floor is the goblin's; his summons say so.
#[test]
fn the_epitaph_names_the_killing_blow() {
    let mut g = camp(31);
    let id = die_by(&mut g, 8, "goblin", false, None);
    let d = g.death(id).expect("the death");
    let m = d.memorial.expect("memorial");
    assert_eq!(m.epitaph, "fell to goblin, D8", "a floor goblin's blow is no Warlord's");
    assert_eq!(d.cause, "goblin");
    assert!(d.summoned_by.is_none());
    // his reserve: the header and the stone both say who called it
    let id = die_by(&mut g, 8, "goblin", true, None);
    let d = g.death(id).expect("the death");
    assert_eq!(d.summoned_by.as_deref(), Some("Warlord"));
    assert_eq!(d.memorial.unwrap().epitaph, "fell to goblin, summoned by the Warlord, D8");
    // the boss's own blow
    let id = die_by(&mut g, 8, "goblin_warlord", false, None);
    assert_eq!(g.death(id).unwrap().memorial.unwrap().epitaph, "fell to the Warlord, D8");
}

/// §2 (B: `fell to the Mother` while the trace said `not in view`): gas names the Mother only when she was in view.
#[test]
fn a_hazard_names_its_boss_only_in_view() {
    let mut g = camp(32);
    let id = die_by(&mut g, 13, "gas", false, Some(false));
    let d = g.death(id).expect("the death");
    assert!(d.source.is_none());
    assert_eq!(d.memorial.unwrap().epitaph, "fell to gas, D13");
    let id = die_by(&mut g, 13, "gas", false, Some(true));
    let d = g.death(id).expect("the death");
    assert_eq!(d.source.as_deref(), Some("Mother"));
    assert_eq!(d.memorial.unwrap().epitaph, "fell to the Mother's gas, D13");
}

#[test]
fn the_killer_phrase_reads_one_story() {
    assert_eq!(feats::killer_phrase("lurker", Some("lurker_queen"), None), "lurker, summoned by the Queen");
    assert_eq!(feats::killer_phrase("goblin", Some("goblin_captain"), None), "goblin, summoned by goblin captain");
    assert_eq!(feats::killer_phrase("poison", None, Some("bloat_mother")), "the Mother's poison");
    assert_eq!(feats::killer_phrase("lurker_queen", None, None), "the Queen");
    assert_eq!(feats::killer_phrase("cave_troll", None, None), "cave troll");
}

/// §1 (blind 1a7d834: Ascension restarted at D1 rats): a numbered descent starts a band above the deepest lit stone
/// (never above D9), the stones to it lit, the record at it; its modifiers are on the wire.
#[test]
fn ascension_starts_at_the_deep_stones_with_its_modifiers_named() {
    let mut g = Game::new_resident(3);
    g.lineage.ended = true;
    g.lineage.best_depth = 34;
    g.lineage.waystones = vec![5, 9, 14, 19, 24, 29];
    crate::packages::recompile(&mut g.lineage);
    assert_eq!(crate::endgame::ascent_start(&g.lineage), 14);
    assert!(g.lineage().descent.is_none(), "no descent before one is begun");
    g.begin_descent(1).unwrap();
    assert_eq!(g.lineage.start, 14);
    assert_eq!(g.lineage.waystones, vec![5, 9, 14]);
    assert_eq!(g.lineage.best_depth, 13);
    let w = g.lineage().descent.expect("the descent on the wire");
    assert_eq!((w.tier, w.start), (1, 14));
    assert!(w.hp_pct > 0 && w.atk_pct > 0);
    assert!(w.modifiers.iter().any(|m| m.id == "armoured"), "tier 1's affix: {:?}", w.modifiers.iter().map(|m| &m.id).collect::<Vec<_>>());
    assert!(!w.modifiers.iter().any(|m| m.id == "swift"), "tier 1 has no swift affix");
    // the first send goes from the start
    g.send();
    assert_eq!(g.run.as_ref().unwrap().depth, 14);
    // a shallow line: never above D9; none lit at all, D1
    let mut l = Game::new_resident(4).lineage;
    l.waystones = vec![5, 9, 14];
    assert_eq!(crate::endgame::ascent_start(&l), 9);
    l.waystones = vec![5];
    assert_eq!(crate::endgame::ascent_start(&l), 5);
    l.waystones.clear();
    assert_eq!(crate::endgame::ascent_start(&l), 1);
}

/// §2 (blind 1a7d834 B: `−$2807 GOLD CHANGE` was the apprentice's spending): the report's gold carries the income and
/// the workers' spend apart from the net.
#[test]
fn the_report_separates_income_from_the_workers_spend() {
    let mut g = camp(33);
    g.lineage.gold_move(2_000, "test income");
    let before = g.lineage.gold_tally.clone();
    let acts = g.lineage.tree.acts.clone();
    g.lineage.gold_move(900, "banked D9");
    g.lineage.gold_move(-1_500, "forge sword +1");
    crate::tree::add_act(&mut g.lineage, crate::tree::APPRENTICE_SPENT, 1_500);
    g.lineage.gold_move(-300, "supply ration");
    g.lineage.gold_move(-200, "tithe");
    let mut r = crate::wire::ReturnReport { gold: Some(crate::wire::GoldSummary { net: Some(900 - 2_000), ..Default::default() }), ..Default::default() };
    crate::offline::set_terms(&mut r, &g.lineage, Some(&before), &acts);
    let gold = r.gold.unwrap();
    assert_eq!(gold.net, Some(-1_100), "the net reads a loss");
    assert_eq!(gold.spent_by_workers, Some(2_000), "the apprentice's step and his sinks");
    assert!(gold.earned.unwrap() >= 900, "the haul is income: {:?}", gold.earned);
}

/// §2 (blind 1a7d834 A: `death 75%` on an easy clear): the ends carry bands — a Wilson interval on the death share
/// (never 0–0 on a few sims) and a band on the mean gold.
#[test]
fn the_forecast_ends_carry_bands() {
    let (lo, hi) = crate::forecast::wilson(0.0, 12);
    assert!(lo == 0.0 && hi > 0.15, "no deaths in 12 is not `death 0`: {hi}");
    let (lo, hi) = crate::forecast::wilson(0.75, 8);
    assert!(lo < 0.45 && hi > 0.9, "6 of 8 is a wide band: {lo}–{hi}");
    let g = camp(34);
    let f = g.forecast();
    let e = f.ends.expect("ends");
    assert!(e.death_lo <= e.death && e.death <= e.death_hi, "{e:?}");
    assert!(e.gold_lo <= e.gold && e.gold <= e.gold_hi, "{e:?}");
    assert!(e.death_hi > e.death_lo);
}

/// §2 (A: `compare: same` on a pick that then stalled): a wall whose paired split leans past the noise is no `same`.
#[test]
fn compare_same_needs_the_walls_to_agree() {
    use crate::packages::{wall_differs, WallRead};
    let w = |better, worse| WallRead { depth: 28, boss: "Queen".into(), n: 8, better, worse };
    assert!(!wall_differs(&w(0, 0)), "every send alike");
    assert!(!wall_differs(&w(2, 2)), "both ways");
    assert!(wall_differs(&w(0, 3)), "every differing send worse");
    assert!(wall_differs(&w(1, 6)));
    assert!(!wall_differs(&w(1, 0)), "one send is noise");
}

/// §4 (blind 1a7d834 B: a boss rule misfired on a siren unsaid): a foe-tag row's why names the kinds it fired on, a
/// boss flagged — the tablet's `also matches: siren`.
#[test]
fn a_foe_tag_rows_why_names_the_kinds_it_fired_on() {
    let row = crate::rules::Row::new(vec![crate::rules::Cond { k: "foe_tag".into(), t: Some("boss".into()), ..Default::default() }], crate::rules::Verb::new("attack"));
    let mut t = crate::engine::RowTally { actions: 10, fired: 4, matched: 4, ..Default::default() };
    t.fired_on.insert("siren".into(), 3);
    t.fired_on.insert("lurker_queen".into(), 1);
    let s = crate::turn::row_stat(&row, &t);
    assert_eq!(s.fired_on, vec![crate::wire::FiredOn { kind: "lurker_queen".into(), boss: true, n: 1 }, crate::wire::FiredOn { kind: "siren".into(), boss: false, n: 3 }]);
    // real sends: every kind a foe-tag row fired on is a kind of the game
    let mut g = camp(35);
    for _ in 0..4 {
        g.send();
        let _ = crate::offline::run_offline_counts(&mut g, 1);
    }
    for s in g.lineage.row_why().into_iter().flatten() {
        assert!(s.fired_on.iter().all(|f| crate::defs::MONSTERS.iter().any(|d| d.kind == f.kind) && f.n > 0), "{:?}", s.fired_on);
    }
}
