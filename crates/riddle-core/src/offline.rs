//! Offline batch: consumes ticks at 10/s across consecutive expeditions and the camp rests
//! between them (Cut 2 §1), samples when stalled.
use crate::engine::{Batch, ExitTier, Game, REST_CAP_TICKS, REST_MIN_TICKS, WAKE_TICKS};
use crate::item::to_inv;
use crate::wire::*;
use std::collections::BTreeMap;

pub const TICKS_PER_SECOND: u64 = 10;
pub const STALL_RUNS: u32 = 20;
pub const SAMPLE_RUNS: u32 = 20;

pub fn run_offline(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, true)
}

/// Same batch, but the worst death is returned by id only (no verdict or patch forecasts, which
/// cost ~3 s); the client calls `death(id)` once at the end of a chunked absence.
pub fn run_offline_quick(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, false)
}

/// Rest that follows a run of `turns` ticks ending in `tier` (Cut 2 §1).
pub fn rest_after(turns: u32, tier: ExitTier) -> u32 {
    match tier {
        ExitTier::Death => WAKE_TICKS,
        _ => turns.clamp(REST_MIN_TICKS, REST_CAP_TICKS),
    }
}

fn run_offline_with(game: &mut Game, elapsed_s: u64, full: bool) -> ReturnReport {
    let budget: u64 = elapsed_s * TICKS_PER_SECOND;
    // Renown of runs watched since the last report settles as its own absence.
    game.settle_renown(0);
    game.batch = Batch::default();
    game.events.clear();
    game.offline = true;
    let facts_before = game.lineage.facts.clone();
    let class = game.lineage.class.name().to_string();
    let rank_before = game.lineage.rank;
    let mut consumed: u64 = 0;
    let mut stall = game.stall_runs;
    let mut sampled = false;
    while consumed < budget {
        // Camp rest first (the heir at camp: no run, or a run not yet begun).
        if game.lineage.rest_left > 0 && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            let used = game.rest_tick((budget - consumed).min(u32::MAX as u64) as u32);
            consumed += used as u64;
            game.batch.rested += used as u64;
            if game.lineage.rest_left > 0 {
                break;
            }
        }
        if game.run.is_none() {
            game.start_run(None);
            game.events.clear();
        }
        while game.run.as_ref().is_some_and(|r| r.over.is_none()) && consumed < budget {
            game.tick();
            game.events.clear();
            consumed += 1;
        }
        if game.run.as_ref().is_some_and(|r| r.over.is_some()) {
            let outcome = game.finish_run().unwrap_or_default();
            game.auto_keep();
            game.events.clear();
            if outcome.new_facts == 0 && !outcome.new_best {
                stall += 1;
            } else {
                stall = 0;
            }
            if stall >= STALL_RUNS && consumed < budget {
                // Statistically identical runs: sample and extrapolate the rest of the budget.
                let mut ticks = 0u64;
                let mut rested = 0u64;
                let mut deaths: BTreeMap<String, u32> = BTreeMap::new();
                let mut banked = 0u32;
                let mut returned = 0u32;
                for _ in 0..SAMPLE_RUNS {
                    game.lineage.rest_left = 0;
                    game.start_run(None);
                    game.run_to_end(crate::forecast::SIM_MAX_TICKS);
                    let (turns, tier, cause) = {
                        let r = game.run.as_ref().unwrap();
                        (r.turn, r.over.unwrap_or(ExitTier::Return), if r.over == Some(ExitTier::Death) { r.death_cause.clone() } else { None })
                    };
                    ticks += turns as u64;
                    rested += rest_after(turns, tier) as u64;
                    match tier {
                        ExitTier::Bank => banked += 1,
                        ExitTier::Return => returned += 1,
                        ExitTier::Death => {}
                    }
                    if let Some(c) = cause {
                        *deaths.entry(c).or_insert(0) += 1;
                    }
                    game.finish_run();
                    game.auto_keep();
                    game.events.clear();
                }
                consumed += ticks + rested;
                game.batch.rested += rested;
                let mean = ((ticks + rested) / SAMPLE_RUNS as u64).max(1);
                if consumed < budget {
                    let remaining = budget - consumed;
                    let extra = remaining / mean;
                    game.batch.runs += extra as u32;
                    let share = |k: u32| (extra as f64 * k as f64 / SAMPLE_RUNS as f64).round() as u32;
                    for (c, k) in &deaths {
                        *game.batch.deaths.entry(c.clone()).or_insert(0) += share(*k);
                    }
                    game.batch.banked += share(banked);
                    game.batch.returned += share(returned);
                    game.batch.rested += extra * (rested / SAMPLE_RUNS as u64);
                    consumed = budget - remaining % mean;
                    game.lineage.total_turns += extra * mean;
                }
                sampled = true;
                // Leave the hero mid-run where the budget ran out.
                if consumed < budget {
                    game.lineage.rest_left = 0;
                    game.start_run(None);
                    while game.run.as_ref().is_some_and(|r| r.over.is_none()) && consumed < budget {
                        game.tick();
                        game.events.clear();
                        consumed += 1;
                    }
                }
                break;
            }
        }
    }
    game.stall_runs = stall;
    game.offline = false;
    report(game, elapsed_s, &facts_before, &class, rank_before, sampled, full)
}

fn report(game: &mut Game, elapsed_s: u64, facts_before: &std::collections::BTreeSet<String>, class: &str, rank_before: u32, sampled: bool, full: bool) -> ReturnReport {
    let t = game.run.as_ref().map(|r| r.turn).unwrap_or(0);
    game.settle_renown(t);
    let learned: Vec<String> = game.lineage.facts.difference(facts_before).cloned().collect();
    let worst_death_id = game.batch.worst_death;
    let worst_death = if full { worst_death_id.and_then(|id| crate::trace::death(game, id)) } else { None };
    let pending = crate::meta::pending(game);
    let live = game.ensure_run();
    game.events.clear();
    let b = &game.batch;
    let mut deaths: Vec<DeathCount> = b.deaths.iter().map(|(c, n)| DeathCount { cause: c.clone(), n: *n }).collect();
    deaths.sort_by(|a, b| b.n.cmp(&a.n).then(a.cause.cmp(&b.cause)));
    let salvaged = b.salvaged.iter().map(|(k, (n, g))| SalvageRow { kind: k.clone(), n: *n, gold: (*g + 50) / 100 }).collect();
    ReturnReport {
        elapsed_s,
        runs: b.runs,
        sampled,
        learned,
        bests: b.bests.clone(),
        found: b.found.iter().map(|i| to_inv(i, &game.lineage.facts, &game.lineage.flavours)).collect(),
        deaths,
        pending,
        reel: crate::sifter::reel(&b.highlights),
        marks_earned: b.marks,
        worst_death,
        worst_death_id,
        live,
        tamed: b.tamed.clone(),
        hatched: b.hatched.clone(),
        lost: b.lost.clone(),
        xp: XpReport { class: class.into(), gained: b.xp_gained, level_ups: b.level_ups },
        salvaged,
        renown: RenownReport { gained: b.renown_gained, rank: game.lineage.rank, ranks_up: game.lineage.rank - rank_before },
        rested_s: b.rested / TICKS_PER_SECOND,
        banked: b.banked,
        returned: b.returned,
        bones_found: b.bones_found.clone(),
    }
}
