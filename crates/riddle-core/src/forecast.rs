//! Forecast: N deterministic sims on fresh floor seeds, reported only as deep as the hero has facts.
use crate::engine::{ExitTier, Game};
use crate::rng::splitmix;
use crate::rules::RuleSet;
use crate::wire::{Forecast, ForecastCause, ForecastDepth};
use std::collections::BTreeMap;

pub const FORECAST_SIMS: u32 = 50;
/// Sims per candidate when computing patch forecast deltas (paired seeds; base and patched
/// runs share them). Cut 4: 12, the replay count — a delta of `DELTA_BAR` is "one seed
/// improved net" at 12 as it was at 20, and the verdict now waits on these (up to three
/// candidates when no patch has a survival edge).
pub const DELTA_SIMS: u32 = 12;
/// Cut 3: a forecast stops launching sims once this many ticks have been simulated (a deep
/// lineage's sims run to D20+, ~40 000 ticks each); at least `MIN_SIMS` always run. A shallow
/// lineage's 50 × ~6 000 ticks stay under it, so the Cut 1/2 numbers are unchanged.
pub const FORECAST_TICK_BUDGET: u64 = 400_000;
pub const DELTA_TICK_BUDGET: u64 = 150_000;
pub const MIN_SIMS: u32 = 5;
/// Cut 3: a sim runs until the run ends or `stop_depth` is reached (the forecast only reports to
/// `best_depth + 1`, so a deep lineage's sims stay cheap), never past the run cap.
pub const SIM_MAX_TICKS: u32 = crate::engine::MAX_TURNS_PER_RUN;

pub struct SimResult {
    pub max_depth: u32,
    pub tier: ExitTier,
    pub cause: Option<String>,
}

/// Simulate `sims` fresh expeditions from the current lineage with `rules`, each stopping once
/// it reaches `stop_depth` (the depth the caller asks about) or ends.
pub fn simulate(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32) -> Vec<SimResult> {
    let budget = if sims >= FORECAST_SIMS { FORECAST_TICK_BUDGET } else { DELTA_TICK_BUDGET };
    simulate_budget(game, rules, sims, tag, stop_depth, budget)
}

pub fn simulate_budget(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32, budget: u64) -> Vec<SimResult> {
    let mut out = Vec::with_capacity(sims as usize);
    let mut spent: u64 = 0;
    for i in 0..sims {
        if i >= MIN_SIMS && spent >= budget {
            break;
        }
        let mut g = game.sim_clone();
        g.run = None;
        g.pending_exit = None;
        g.history.clear();
        let _ = g.set_rules(rules.clone());
        let seed = splitmix(game.lineage.seed ^ splitmix(tag ^ (i as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407)));
        g.start_run(Some(seed));
        let mut n = 0;
        while g.run.as_ref().is_some_and(|r| r.over.is_none() && r.max_depth < stop_depth) && n < SIM_MAX_TICKS {
            g.tick();
            n += 1;
        }
        spent += n as u64;
        let run = g.run.as_ref().unwrap();
        out.push(SimResult { max_depth: run.max_depth, tier: run.over.unwrap_or(ExitTier::Return), cause: run.death_cause.clone() });
    }
    out
}

pub fn forecast(game: &Game) -> Forecast {
    forecast_with(game, game.lineage.rules(), FORECAST_SIMS)
}

/// Cut 6 §9: the same forecast at `REFINE_SIMS` sims — the same seeds first, then as many
/// again (a second, quieter pass the client runs once the rule set has been still for 2 s).
pub fn forecast_refine(game: &Game) -> Forecast {
    forecast_with(game, game.lineage.rules(), REFINE_SIMS)
}

pub const REFINE_SIMS: u32 = 2 * FORECAST_SIMS;

/// Cut 6 §9: the forecast's seeds are a function of (rules, lineage seed, depth) — re-reading
/// the same set gives the same number, and nothing transient (marks, renown, the rest clock,
/// the run in progress) moves it. The lineage state a sim starts from (facts, gold, class…)
/// still does: `reach_with`'s fingerprint is the same idea.
pub fn forecast_tag(game: &Game, rules: &RuleSet, depth: u32) -> u64 {
    let mut h: u64 = 0x5EED_F0C4;
    for b in serde_json::to_string(rules).unwrap_or_default().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    splitmix(h ^ game.lineage.seed.rotate_left(17) ^ ((depth as u64) << 40))
}

pub fn forecast_with(game: &Game, rules: &RuleSet, sims: u32) -> Forecast {
    let known_to = game.lineage.best_depth + 1;
    let tag = forecast_tag(game, rules, known_to);
    // The refine pass runs twice the sims under twice the tick budget (the same seeds first).
    let budget = FORECAST_TICK_BUDGET * (sims as u64).div_ceil(FORECAST_SIMS as u64).max(1);
    let results = simulate_budget(game, rules, sims, tag, known_to, budget);
    let n = results.len().max(1) as f64;
    let depths = (1..=known_to)
        .map(|d| {
            let reach = results.iter().filter(|r| r.max_depth >= d).count() as f64 / n;
            ForecastDepth { depth: d, reach, pm: Some(half_width(reach, results.len())) }
        })
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

/// Cut 9 §3: the 95 % binomial half-width of a share `p` over `n` sims (`1.96·√(p(1−p)/n)`).
pub fn half_width(p: f64, n: usize) -> f64 {
    if n == 0 {
        return 0.0;
    }
    1.96 * (p * (1.0 - p) / n as f64).sqrt()
}

/// Fraction of sims reaching `depth` with `rules`. Cut 4: memoised on the game per (lineage,
/// rules, depth, sims, tag) — a verdict, the stall verdict and the unlock deltas all ask for
/// the same unpatched base at the same depth, and a batch of verdicts asks for it per death.
pub fn reach_with(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64) -> f64 {
    let budget = if sims >= FORECAST_SIMS { FORECAST_TICK_BUDGET } else { DELTA_TICK_BUDGET };
    reach_budget(game, rules, depth, sims, tag, budget)
}

/// Cut 4 §9: the unlock catalogue's deltas run under a tighter tick budget per forecast
/// (`MIN_SIMS` always), so a deep lineage's camp visit pays seconds, not tens of seconds.
/// Cut 9 §3: 100 000 — a D1–5 lineage's 50 panel seeds fit (the base then *is* the panel
/// number); deeper, the base runs as many seeds as fit and every candidate replays exactly
/// those (`reach_paired`), so a delta is always a paired difference over the same seeds.
pub const CATALOGUE_TICK_BUDGET: u64 = 100_000;

pub fn reach_budget(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64, budget: u64) -> f64 {
    reach_counted(game, rules, depth, sims, tag, budget).0
}

/// `reach_budget` with the number of sims that ran (the budget may have stopped it short).
pub fn reach_counted(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64, budget: u64) -> (f64, u32) {
    let key = reach_key(game, rules, depth, sims, tag, budget);
    if let Some(v) = game.forecast_cache.borrow().get(&key) {
        return *v;
    }
    let results = simulate_budget(game, rules, sims, tag, depth, budget);
    let v = results.iter().filter(|r| r.max_depth >= depth).count() as f64 / results.len().max(1) as f64;
    let mut cache = game.forecast_cache.borrow_mut();
    if cache.len() >= FORECAST_CACHE_MAX {
        cache.clear();
    }
    cache.insert(key, (v, results.len() as u32));
    (v, results.len() as u32)
}

/// Cut 9 §3: the reach of `rules` over exactly the first `n` seeds of `tag` (no tick budget),
/// to pair a candidate with a base that ran `n` (`reach_counted`).
pub fn reach_paired(game: &Game, rules: &RuleSet, depth: u32, n: u32, tag: u64) -> f64 {
    reach_counted(game, rules, depth, n.max(1), tag, u64::MAX).0
}

pub const FORECAST_CACHE_MAX: usize = 256;

/// The memoised reach, if this game already computed it (no sims).
pub fn reach_cached(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64, budget: u64) -> Option<f64> {
    game.forecast_cache.borrow().get(&reach_key(game, rules, depth, sims, tag, budget)).map(|v| v.0)
}

pub fn reach_key(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64, budget: u64) -> String {
    format!("{}:{depth}:{sims}:{tag}:{budget}:{}", lineage_key(game), serde_json::to_string(rules).unwrap_or_default())
}

/// A fingerprint of what a sim starts from: the lineage fields a fresh run reads (facts,
/// unlocks, class and level, vault and loadout, party, supplies, gold, forge, grudges, bones,
/// insurance, keep preference, trait, heir, variant, hunter) — not marks, renown or the rest
/// clock, so a purchase or a rank does not spill the cache.
fn lineage_key(game: &Game) -> u64 {
    let l = &game.lineage;
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |s: &str| {
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h ^= 0xff;
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    feed(&l.seed.to_string());
    feed(&l.heir.to_string());
    feed(l.trait_.name());
    feed(l.class.name());
    feed(&l.class_level().to_string());
    feed(&format!("{:?}", l.facts));
    feed(&format!("{:?}", l.unlocks));
    feed(&serde_json::to_string(&l.vault).unwrap_or_default());
    feed(&format!("{:?}", game.loadout));
    feed(&serde_json::to_string(&l.party).unwrap_or_default());
    feed(&serde_json::to_string(&l.supplies).unwrap_or_default());
    feed(&format!("{:?}", l.last_supplies));
    feed(&l.gold.to_string());
    feed(&serde_json::to_string(&l.forge).unwrap_or_default());
    feed(&serde_json::to_string(&l.grudges).unwrap_or_default());
    feed(&serde_json::to_string(&l.bones).unwrap_or_default());
    feed(&format!("{:?}", l.insured));
    feed(&l.keep_pref);
    feed(&l.variant);
    feed(&serde_json::to_string(&l.hunter).unwrap_or_default());
    feed(&format!("{:?}", l.kill_counts));
    feed(&l.ended.to_string());
    h
}
