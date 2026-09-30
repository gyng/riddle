//! Cut 30 §6 (docs/CUT30.md): the idle floor's table rows — a 20-minute absence pays (fresh and D13
//! lineages), a drill's item is in the pack at its wall, every stance is best at some wall and none at
//! all, each quest is keepable with the pen closed. The lineages are IDLE's own: a fresh lineage sent at
//! three check-ins a day, snapshot the first time it stands under each wall.
use riddle_core::engine::ExitTier;
use riddle_core::packages;
use riddle_core::Game;

/// The walls the stances are weighed at (a band boss's floor).
pub const WALLS: [u32; 5] = [8, 13, 18, 23, 28];
pub const STANCES: [&str; 4] = ["steady", "guarded", "bold", "hunter"];

/// IDLE's lineage of `seed` over `days`, snapshot at each wall (the first check-in it stands at
/// `wall − 1` or deeper, before passing it), and the first check-in at D13 or deeper.
pub fn snapshots(seed: u64, days: usize) -> Vec<(u32, Game)> {
    let mut g = Game::new(seed);
    let mut out: Vec<(u32, Game)> = Vec::new();
    for _ in 0..days * 3 {
        riddle_core::offline::run_offline_counts(&mut g, 8 * 3600);
        for w in WALLS {
            if g.lineage.best_depth + 1 >= w && g.lineage.best_depth <= w && !out.iter().any(|(x, _)| *x == w) {
                out.push((w, g.clone()));
            }
        }
        if out.len() == WALLS.len() {
            break;
        }
    }
    out
}

/// A 20-minute absence's runs.
pub fn twenty_minutes(g: &Game) -> u32 {
    let mut g = g.clone();
    riddle_core::offline::run_offline_counts(&mut g, 20 * 60).runs
}

/// The shares of `stance`'s sends from `g` that pass `wall` (a sim panel of `sims`), with the
/// stance worn at its lineage's level.
pub fn stance_past(g: &Game, stance: &str, wall: u32, sims: u32) -> f64 {
    let mut c = g.sim_clone();
    c.lineage.pkg.owned.insert(stance.to_string());
    if packages::equip(&mut c.lineage, stance, 0).is_err() {
        return 0.0;
    }
    let set = packages::compile(&c.lineage);
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    rs.iter().filter(|r| r.max_depth > wall).count() as f64 / rs.len().max(1) as f64
}

/// Of `sends` sends from a lineage whose Mother drill wants fire (fire named, the purse full), the share
/// that carry it into the run (the quartermaster's reserved slot).
pub fn drill_packed(seed: u64, sends: u32) -> (u32, u32) {
    let mut g = Game::new(seed);
    for k in ["heal", "fire"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    g.lineage.best_depth = 12;
    g.lineage.gold = 20_000;
    g.lineage.pkg.drills.push(packages::Drill { boss: "bloat_mother".into(), rows: packages::drill_rows("bloat_mother", 30), revoked: false, announced: false });
    packages::recompile(&mut g.lineage);
    let mut carried = 0;
    for _ in 0..sends {
        g.lineage.rest_left = 0;
        g.start_run(None);
        carried += g.run.as_ref().is_some_and(|r| r.hero.inv.iter().any(|i| i.kind == "fire")) as u32;
        g.run_to_end(riddle_core::engine::MAX_TURNS_PER_RUN);
        g.finish_run();
        g.auto_keep();
        g.events.clear();
        g.lineage.gold = g.lineage.gold.max(20_000);
        g.lineage.best_depth = 12;
    }
    (carried, sends)
}

/// A quest of `kind` for the lineage as it stands (its goal as the board draws it), and the best
/// stance's chance a night (16 sends) keeps it — the pen closed, the packages alone.
pub fn quest_night(g: &Game, kind: &str, sims: u32) -> (String, f64) {
    let best = g.lineage.best_depth.max(1);
    let next_boss = riddle_core::descent::BOSS_DEPTHS.iter().find(|(k, d)| !g.lineage.kills.contains(*k) && *d <= best + 2).map(|(k, d)| (k.to_string(), *d));
    let depth = match kind {
        "reach" => best + 1,
        "reach_no_return" => best.saturating_sub(2).max(2),
        "bank" => best.saturating_sub(1).max(2),
        _ => match &next_boss {
            Some((_, d)) => *d,
            None => return (kind.to_string(), 1.0),
        },
    };
    let mut top = 0.0f64;
    for s in STANCES {
        let mut c = g.sim_clone();
        c.lineage.pkg.owned.insert(s.to_string());
        if packages::equip(&mut c.lineage, s, 0).is_err() {
            continue;
        }
        c.lineage.pkg.pen_open = false;
        let set = packages::compile(&c.lineage);
        let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
        let kept = rs
            .iter()
            .filter(|r| match kind {
                "reach" => r.max_depth >= depth,
                "reach_no_return" => r.max_depth >= depth && r.tier != ExitTier::Return,
                "bank" => r.max_depth >= depth && r.tier == ExitTier::Bank,
                _ => r.max_depth > depth,
            })
            .count() as f64
            / rs.len().max(1) as f64;
        top = top.max(1.0 - (1.0 - kept).powi(16));
    }
    (format!("{kind} D{depth}"), top)
}
