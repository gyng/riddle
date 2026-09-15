//! Forecast: N deterministic sims on fresh floor seeds, reported only as deep as the hero has facts.
use crate::engine::{ExitTier, Game};
use crate::rng::splitmix;
use crate::rules::RuleSet;
use crate::wire::{Forecast, ForecastCause, ForecastDepth};
use std::collections::BTreeMap;

pub const FORECAST_SIMS: u32 = 50;
/// Sims per candidate when computing patch forecast deltas (four forecasts per death).
pub const DELTA_SIMS: u32 = 20;
pub const SIM_MAX_TICKS: u32 = 30_000;

pub struct SimResult {
    pub max_depth: u32,
    pub tier: ExitTier,
    pub cause: Option<String>,
}

/// Simulate `sims` fresh expeditions from the current lineage with `rules`.
pub fn simulate(game: &Game, rules: &RuleSet, sims: u32, tag: u64) -> Vec<SimResult> {
    let mut out = Vec::with_capacity(sims as usize);
    for i in 0..sims {
        let mut g = game.sim_clone();
        g.run = None;
        g.pending_exit = None;
        g.history.clear();
        let _ = g.set_rules(rules.clone());
        let seed = splitmix(game.lineage.seed ^ splitmix(tag ^ (i as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407)));
        g.start_run(Some(seed));
        g.run_to_end(SIM_MAX_TICKS);
        let run = g.run.as_ref().unwrap();
        out.push(SimResult { max_depth: run.max_depth, tier: run.over.unwrap_or(ExitTier::Return), cause: run.death_cause.clone() });
    }
    out
}

pub fn forecast(game: &Game) -> Forecast {
    forecast_with(game, game.lineage.rules(), FORECAST_SIMS)
}

pub fn forecast_with(game: &Game, rules: &RuleSet, sims: u32) -> Forecast {
    let known_to = game.lineage.best_depth + 1;
    let results = simulate(game, rules, sims, 0x5EED_F0C4);
    let n = results.len().max(1) as f64;
    let depths = (1..=known_to)
        .map(|d| ForecastDepth { depth: d, reach: results.iter().filter(|r| r.max_depth >= d).count() as f64 / n })
        .collect();
    let mut causes: BTreeMap<String, u32> = BTreeMap::new();
    let mut deaths = 0u32;
    for r in &results {
        if r.tier == ExitTier::Death {
            deaths += 1;
            if let Some(c) = &r.cause {
                *causes.entry(c.clone()).or_insert(0) += 1;
            }
        }
    }
    let mut cv: Vec<(String, u32)> = causes.into_iter().collect();
    cv.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let causes = cv.into_iter().take(3).map(|(c, k)| ForecastCause { cause: c, share: k as f64 / deaths.max(1) as f64 }).collect();
    Forecast { depths, causes, known_to }
}

/// Fraction of sims reaching `depth` with `rules`.
pub fn reach_with(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64) -> f64 {
    let results = simulate(game, rules, sims, tag);
    results.iter().filter(|r| r.max_depth >= depth).count() as f64 / results.len().max(1) as f64
}
