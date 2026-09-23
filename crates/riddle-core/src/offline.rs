//! Offline batch: consumes ticks at 10/s across consecutive expeditions and the camp rests
//! between them (Cut 2 §1), samples when stalled.
use crate::engine::{Batch, ExitTier, Game, StallTally, REST_CAP_TICKS, REST_MIN_TICKS, WAKE_TICKS};
use crate::item::to_inv;
use crate::rules::{Cond, Row, RuleSet, Verb};
use crate::wire::*;
use std::collections::BTreeMap;

pub const TICKS_PER_SECOND: u64 = 10;
pub const STALL_RUNS: u32 = 20;
pub const SAMPLE_RUNS: u32 = 20;
/// Stall verdict: this many runs home in a row with no death and no new depth.
pub const STALL_MIN_RUNS: u32 = 4;
/// A stall patch must move the forecast at the stall depth + 1 by more than this.
pub const STALL_DELTA: f64 = 0.02;
pub const STALL_SHOWN: usize = 3;

pub fn run_offline(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, true)
}

/// Same batch, but the worst death is returned by id only (no verdict or patch forecasts, which
/// cost ~3 s); the client calls `death(id)` once at the end of a chunked absence.
pub fn run_offline_quick(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, false)
}

/// Rest that follows a run of `turns` ticks ending in `tier` (Cut 2 §1). Cut 12 §5: after a
/// return or a bank, **half** the run's length (was: the run's; rater P: "`rested 287m` — the
/// hero idled for five of the eight hours"), still never under `REST_MIN_TICKS` nor over the
/// cap; the wake after a death stays.
pub fn rest_after(turns: u32, tier: ExitTier) -> u32 {
    match tier {
        ExitTier::Death => WAKE_TICKS,
        _ => (turns / 2).clamp(REST_MIN_TICKS, REST_CAP_TICKS),
    }
}

fn run_offline_with(game: &mut Game, elapsed_s: u64, full: bool) -> ReturnReport {
    let budget: u64 = elapsed_s * TICKS_PER_SECOND;
    // Renown of runs watched since the last report settles as its own absence.
    game.settle_renown(0);
    game.batch = Batch::default();
    game.events.clear();
    game.offline = true;
    // Cut 7 §5: nothing in an absence is watched (a run left mid-watch finishes unwatched).
    game.watched = false;
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
        // Cut 12: a begun run finishes past the budget (≤ one run), so an absence always ends
        // at camp and the next send packs what the player bought and runs the rules they
        // edited (cohort 8: a return's first run resumed a night-old run from D2 with an
        // empty pack, and its trace read `no item ← never found` beside 5/5 supplies).
        if game.run.as_ref().is_some_and(|r| r.over.is_none() && r.turn > 0) {
            let before = game.run.as_ref().unwrap().turn;
            game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            game.events.clear();
            consumed += (game.run.as_ref().unwrap().turn - before) as u64;
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
                let exits_before = game.batch.exit_rows.clone();
                for _ in 0..SAMPLE_RUNS {
                    game.lineage.rest_left = 0;
                    game.start_run(None);
                    game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
                    let (turns, tier, cause) = {
                        let r = game.run.as_ref().unwrap();
                        (r.turn, r.over.unwrap_or(ExitTier::Return), if r.over == Some(ExitTier::Death) { r.death_cause.clone() } else { None })
                    };
                    ticks += turns as u64;
                    rested += game.rest_after(turns, tier) as u64;
                    match tier {
                        ExitTier::Bank => banked += 1,
                        ExitTier::Return => returned += 1,
                        ExitTier::Death => {}
                    }
                    if tier == ExitTier::Death {
                        *deaths.entry(cause.unwrap_or_else(|| "unknown".into())).or_insert(0) += 1;
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
                    // Cut 13 §6: the extrapolated runs are apportioned so `runs == deaths +
                    // banked + returned` holds to the run (largest remainder over the sample's
                    // exits; rounding each share alone left the report a run short).
                    let mut cats: Vec<(String, u32)> = deaths.iter().map(|(c, k)| (c.clone(), *k)).collect();
                    cats.push(("\0bank".into(), banked));
                    cats.push(("\0return".into(), returned));
                    let shares = apportion(extra as u32, &cats.iter().map(|(_, k)| *k).collect::<Vec<u32>>());
                    for ((c, _), n) in cats.iter().zip(shares) {
                        match c.as_str() {
                            "\0bank" => game.batch.banked += n,
                            "\0return" => game.batch.returned += n,
                            _ => *game.batch.deaths.entry(c.clone()).or_insert(0) += n,
                        }
                    }
                    let share = |k: u32| (extra as f64 * k as f64 / SAMPLE_RUNS as f64).round() as u32;
                    // The extrapolated runs end the way the sample did: same exit rows, same stall.
                    let exits: Vec<(i32, u32)> = game.batch.exit_rows.iter().map(|(r, n)| (*r, n - exits_before.get(r).copied().unwrap_or(0))).filter(|(_, n)| *n > 0).collect();
                    for (r, k) in &exits {
                        *game.batch.exit_rows.entry(*r).or_insert(0) += share(*k);
                    }
                    if deaths.is_empty() {
                        game.stall.runs += extra as u32;
                        for (r, k) in &exits {
                            *game.stall.exit_rows.entry(*r).or_insert(0) += share(*k);
                        }
                    } else {
                        game.stall = StallTally::default();
                    }
                    game.batch.rested += extra * (rested / SAMPLE_RUNS as u64);
                    game.lineage.total_turns += extra * mean;
                }
                sampled = true;
                // The extrapolation covered the rest of the budget; the hero is at camp.
                break;
            }
        }
    }
    game.stall_runs = stall;
    game.offline = false;
    report(game, elapsed_s, &facts_before, &class, rank_before, sampled, full)
}

/// Cut 13 §6: `total` split in proportion to `weights` (largest remainder), summing to
/// `total` exactly; all zeros when the weights are.
pub fn apportion(total: u32, weights: &[u32]) -> Vec<u32> {
    let sum: u32 = weights.iter().sum();
    if sum == 0 || weights.is_empty() {
        return vec![0; weights.len()];
    }
    let mut out: Vec<u32> = weights.iter().map(|w| (total as u64 * *w as u64 / sum as u64) as u32).collect();
    let mut rem: Vec<(u64, usize)> = weights.iter().enumerate().map(|(i, w)| ((total as u64 * *w as u64) % sum as u64, i)).collect();
    rem.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let short = total - out.iter().sum::<u32>();
    for (_, i) in rem.into_iter().take(short as usize) {
        out[i] += 1;
    }
    out
}

fn report(game: &mut Game, elapsed_s: u64, facts_before: &std::collections::BTreeSet<String>, class: &str, rank_before: u32, sampled: bool, full: bool) -> ReturnReport {
    let t = game.run.as_ref().map(|r| r.turn).unwrap_or(0);
    game.settle_renown(t);
    let learned: Vec<String> = game.lineage.facts.difference(facts_before).cloned().collect();
    let worst_death_id = game.batch.worst_death;
    let worst_death = if full { worst_death_id.and_then(|id| crate::trace::death(game, id)) } else { None };
    let pending = crate::meta::pending(game);
    let stall = stall_verdict(game);
    // Cut 12: no run is started here — an idle run at turn 0 would have packed the supplies
    // before the player bought them at camp (`start_run` packs). `live` is the run in
    // progress only when one exists (never after an absence; kept on the wire as optional).
    let live = game.run.as_ref().map(|_| game.snapshot());
    game.events.clear();
    // Cut 9 §6: the reel skips the pairs of the last three absences' reels and remembers its
    // own.
    let recent: Vec<(String, String)> = game.lineage.reel_pairs.iter().flatten().cloned().collect();
    let reel = crate::sifter::reel(&game.batch.highlights, game.batch.best_run.map(|(_, id)| id), &recent);
    game.lineage.reel_pairs.push(reel.iter().filter_map(crate::sifter::pair).collect());
    while game.lineage.reel_pairs.len() > crate::engine::REEL_ABSENCES {
        game.lineage.reel_pairs.remove(0);
    }
    let b = &game.batch;
    // The reel says `died to gas` where the tally said `burst ×1`: one word per cause, the
    // reel's (`cause_phrase` merges a bloat's burst into gas).
    let mut tally: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
    for (c, n) in &b.deaths {
        *tally.entry(crate::sifter::cause_key(c)).or_insert(0) += n;
    }
    let mut deaths: Vec<DeathCount> = tally.into_iter().map(|(cause, n)| DeathCount { cause, n }).collect();
    deaths.sort_by(|a, b| b.n.cmp(&a.n).then(a.cause.cmp(&b.cause)));
    // The rows add up to the header (QA on 3d71c33: rows rounded one by one read $275 under
    // `+$263 salvage`): each row floors its cents, the rounded total's remainder goes to the
    // largest fractions.
    let total = b.salvage_gold;   // the header's coins (the ledger's per-exit rounding)
    let mut salvaged: Vec<SalvageRow> = b.salvaged.iter().map(|(k, (n, g))| SalvageRow { kind: k.clone(), n: *n, gold: g / 100 }).collect();
    let mut order: Vec<(i32, usize)> = b.salvaged.values().enumerate().map(|(i, (_, g))| (g % 100, i)).collect();
    order.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let short = total - salvaged.iter().map(|r| r.gold).sum::<i32>();
    for &(_, i) in order.iter().take(short.max(0) as usize) {
        salvaged[i].gold += 1;
    }
    for &(_, i) in order.iter().rev().filter(|&&(_, i)| salvaged[i].gold > 0).take((-short).max(0) as usize).collect::<Vec<_>>() {
        salvaged[i].gold -= 1;
    }
    ReturnReport {
        elapsed_s,
        runs: b.runs,
        sampled,
        learned,
        bests: b.bests.clone(),
        found: b.found.iter().map(|i| to_inv(i, &game.lineage.facts, &game.lineage.flavours)).collect(),
        deaths,
        pending,
        reel,
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
        stall,
        deepest: b.run_outcomes.iter().map(|(d, _)| *d).max().unwrap_or(0),
        // Cut 13 §3: the night's ledger — what the automations bought, per kind in coins.
        stalled: b.stalls,
        spent: b.spent.iter().map(|(k, (n, g))| SalvageRow { kind: k.replace('_', " "), n: *n, gold: *g }).filter(|r| r.gold > 0).collect(),
        gold: Some(crate::wire::GoldSummary { home: b.gold_earned, salvage: b.salvage_gold, wake: b.wake_pay, spent: b.spent.values().map(|(_, g)| *g).sum() }),
        exits: b.exits.clone(),
    }
}

// ---------------------------------------------------------------- stall verdict

/// The stall verdict: when the last ≥ 4 runs all came home (no death, no new depth) the row that
/// ended most of them is named and up to three patches are forecast at the stall depth + 1.
/// The patches are cached per (row, depth, rules, vocabulary): a chunked absence asks every slice.
pub fn stall_verdict(game: &mut Game) -> Option<Stall> {
    let t = &game.stall;
    if t.runs < STALL_MIN_RUNS {
        return None;
    }
    let (&row, &fired) = t.exit_rows.iter().filter(|(r, _)| **r >= 0).max_by_key(|(r, n)| (**n, std::cmp::Reverse(**r)))?;
    let rules = game.lineage.rules().clone();
    let ending = rules.rows.get(row as usize)?.clone();
    let depth = t.depth.max(1);
    // `depth` is the deepest those runs reached: "at D12" read as where they ended (QA on
    // e0f87e7: "nothing ended at D12"); the stall is that none got past it.
    let text = format!("R{} {} ended {} runs, none past D{}", row + 1, ending.verb.short(), fired, depth);
    let vocab = game.vocabulary();
    let key = format!("{row}:{depth}:{}:{}:{}", serde_json::to_string(&rules).unwrap_or_default(), vocab.conds.len(), vocab.verbs.len());
    let patches = match &game.stall_cache {
        Some((k, p)) if *k == key => p.clone(),
        _ => {
            let p = stall_patches(game, &rules, row as usize, &ending, depth);
            game.stall_cache = Some((key, p.clone()));
            p
        }
    };
    let trace = game.stall.traces.get(&row).cloned();
    Some(Stall { row: row as usize, fired, text, patches, trace })
}

/// `rules` with a stall patch applied (replace / remove / insert), cut to the row cap like the editor.
pub fn apply_patch(rules: &RuleSet, p: &Patch, max_rows: usize) -> RuleSet {
    let mut r = rules.clone();
    // Cut 11 §2: the lock pseudo-patch (`insert_at` −1) changes no row — it is an unlock.
    if p.insert_at < 0 {
        return r;
    }
    let at = p.insert_at as usize;
    if p.remove {
        if at < r.rows.len() {
            r.rows.remove(at);
        }
    } else if p.replace && at < r.rows.len() {
        r.rows[at] = p.row.clone();
    } else {
        r.rows.insert(at.min(r.rows.len()), p.row.clone());
        // Cut 12 §1: the cap is on the player's own rows; a card row never falls off.
        r = r.fit(max_rows.max(1));
    }
    r
}

/// Candidates: the ending row pushed 10 points deeper, that row removed, the boss counter when
/// the stall floor is a boss floor and the boss is known, `hp<90 → rest` when absent. Kept
/// when the forecast at depth + 1 moves by more than `STALL_DELTA`, ranked by delta.
fn stall_patches(game: &Game, rules: &RuleSet, row: usize, ending: &Row, depth: u32) -> Vec<Patch> {
    let target = depth + 1;
    let sims = crate::forecast::DELTA_SIMS;
    let max_rows = game.lineage.max_rows();
    let vocab = game.vocabulary();
    let has_verb = |v: &Verb| vocab.verbs.contains(v);
    let has_cond = |k: &str, t: Option<&str>| vocab.conds.iter().any(|c| c.k == k && (t.is_none() || c.t.as_deref() == t));
    let present = |r: &Row| rules.rows.contains(r);
    let patch = |row: Row, at: usize, replace: bool, remove: bool| Patch { row, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace, remove, root: None, below_bar: false };
    let mut cands: Vec<Patch> = Vec::new();
    // (a) the ending row, its threshold pushed deeper.
    let mut deeper = ending.clone();
    let mut changed = false;
    for c in deeper.conds.iter_mut() {
        match (c.k.as_str(), c.n) {
            ("hp<", Some(n)) if n > 10 => {
                c.n = Some(n - 10);
                changed = true;
            }
            ("depth>=", Some(n)) => {
                c.n = Some(n + 1);
                changed = true;
            }
            ("loot>=", Some(n)) if n > 0 => {
                c.n = Some(n * 2);
                changed = true;
            }
            _ => {}
        }
    }
    if changed && !present(&deeper) {
        cands.push(patch(deeper, row, true, false));
    }
    // (b) the ending row removed.
    cands.push(patch(ending.clone(), row, false, true));
    // (c) the boss counter on a boss floor the hero has met.
    if let Some(kind) = crate::descent::boss_for(depth) {
        let facts = &game.lineage.facts;
        let known = crate::facts::boss_counter_known(facts, kind) || facts.contains(&format!("foe:{kind}"));
        if known && has_cond("foe_tag", Some("boss")) {
            let boss = Cond::t("foe_tag", "boss");
            let attack = Row::new(vec![boss.clone()], Verb::arg("attack", "tag:boss"));
            if has_verb(&attack.verb) && !present(&attack) {
                cands.push(patch(attack, 0, false, false));
            }
            if let Some(k) = vocab.verbs.iter().filter(|v| v.v == "throw").filter_map(|v| v.a.as_deref()).find(|a| *a != "unknown") {
                let throw = Row::new(vec![boss], Verb::arg("throw", &format!("{k},tag:boss")));
                if !present(&throw) {
                    cands.push(patch(throw, 0, false, false));
                }
            }
        }
    }
    // (d) rest when the set never rests.
    let rest = Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest"));
    if !rules.rows.iter().any(|r| r.verb.v == "rest") && has_verb(&rest.verb) && has_cond("hp<", None) {
        cands.push(patch(rest, 0, false, false));
    }
    let base = crate::forecast::reach_with(game, rules, target, sims, 0x57A11);
    for p in cands.iter_mut() {
        let r = crate::forecast::reach_with(game, &apply_patch(rules, p, max_rows), target, sims, 0x57A11);
        p.survive = r;
        p.forecast_delta = r - base;
    }
    cands.retain(|p| p.forecast_delta > STALL_DELTA);
    cands.sort_by(|a, b| b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap());
    cands.truncate(STALL_SHOWN);
    cands
}
