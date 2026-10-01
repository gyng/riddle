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
/// A death weighed against a send past the wall: nothing — the idle gates are time to a milestone, and a
/// death's 20-minute wake costs the time a bank's rest does (`WAKE_TICKS` = `REST_MIN_TICKS`); `DW=` probes others.
pub const DEATH_WEIGHT: f64 = 0.0;
/// From D1 a death on the walk costs the carry (70 % of it) and the heir: a quarter of a pass.
pub const WALK_DEATH: f64 = 0.25;

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

/// The shares of `stance`'s sends from `g` that pass `wall` (a sim panel of `sims`): every stance at
/// the worn stance's level, the record at the wall's floor (each stance's bank row then asks for the
/// floor past it — a stance is weighed on passing the wall, not on banking under it).
pub fn stance_past(g: &Game, stance: &str, wall: u32, sims: u32, from_stone: bool) -> f64 {
    let mut c = g.sim_clone();
    // (from the deepest lit waystone at or above the wall: the wall weighed, not the walk to it — or from
    // D1, the walk and the wall)
    c.lineage.start = 1;
    if let Some(s) = c.lineage.stones().into_iter().filter(|s| *s <= wall).max().filter(|_| from_stone) {
        c.lineage.start = s;
    }
    c.lineage.pkg.owned.insert(stance.to_string());
    let worn = c.lineage.pkg.stance.clone();
    let runs = c.lineage.pkg.runs.get(&worn).copied().unwrap_or(0);
    c.lineage.pkg.runs.insert(stance.to_string(), runs);
    c.lineage.best_depth = c.lineage.best_depth.max(wall);
    // (the stances are weighed at the wall before it is drilled: the drill is every stance's, the answer is the stance's own)
    for d in c.lineage.pkg.drills.iter_mut() {
        d.revoked = true;
    }
    if packages::equip(&mut c.lineage, stance, 0).is_err() {
        return 0.0;
    }
    let set = packages::compile(&c.lineage);
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    let k = rs.len().max(1) as f64;
    // a stance's worth at a wall: the sends past it, less half those that die (a death costs the heir
    // and 70 % of the carry — passing by dying more is not the better stance)
    let past = rs.iter().filter(|r| r.max_depth > wall).count() as f64 / k;
    let died = rs.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / k;
    // (from D1 the walk is weighed too: a send that dies on the way loses the heir's carry — `WALK_DEATH`)
    let dw = if from_stone { DEATH_WEIGHT } else { WALK_DEATH };
    past - std::env::var("DW").ok().and_then(|v| v.parse().ok()).unwrap_or(dw) * died
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
    let next_boss = riddle_core::descent::BOSS_DEPTHS.iter().find(|(k, d)| !g.lineage.kills.contains(*k) && *d <= best + 2 && *k != "foundry_master" && g.lineage.pkg.drills.iter().any(|x| x.boss == *k && !x.revoked)).map(|(k, d)| (k.to_string(), *d));
    let depth = match kind {
        "reach" => best.saturating_sub(3).max(2),
        "reach_no_return" => best.saturating_sub(5).max(2),
        "bank" => g.lineage.stones().into_iter().filter(|s| *s <= best).max().unwrap_or(best.saturating_sub(5)).max(2),
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
                "bank" => r.max_depth >= depth && r.tier != ExitTier::Death,
                _ => r.max_depth > depth,
            })
            .count() as f64
            / rs.len().max(1) as f64;
        top = top.max(1.0 - (1.0 - kept).powi(16));
    }
    (format!("{kind} D{depth}"), top)
}
