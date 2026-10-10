//! Offline batch: consumes ticks at 10/s across consecutive expeditions and the camp rests
//! between them (Cut 2 §1). Historical literal tools retain stall sampling;
//! current-game transport slices simulate exact ticks and settle one report.
use crate::engine::{Batch, ExitTier, Game, StallTally, REST_CAP_TICKS, REST_MIN_TICKS, WAKE_TICKS};
use crate::item::to_inv;
use crate::rules::{Cond, Row, RuleSet, Verb};
use crate::wire::*;
use std::collections::BTreeMap;
use serde::{Serialize, Deserialize};

/// Saved continuation and report baselines for one absence. The single-hero
/// clock is committed at the final boundary; run timestamps use consumed ticks.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Absence {
    pub elapsed_s: u64,
    pub consumed: u64,
    pub hour: u64,
    pub day0: u32,
    pub facts_before: crate::shared::Shared<std::collections::BTreeSet<String>>,
    pub grew_before: crate::town::Snap,
    pub class: String,
    pub rank_before: u32,
    pub acts_before: crate::shared::Shared<BTreeMap<String, u32>>,
    pub chest_before: i32,
    /// Blind ad71e72 (A: `$6712 GOLD EARNED`, the purse up $78): the purse at the absence's
    /// start, so the report states the purse's actual change (`GoldSummary.net`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gold_before: Option<i32>,
    /// Cut 117 §1: the lineage's gold tally at the absence's start (`LineageState::gold_tally`), so the
    /// report's ledger terms (`GoldSummary.ledger`) are the absence's movements alone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tally_before: Option<BTreeMap<String, i64>>,
    /// Current guide quote, refreshed at transport boundaries for exact reloads.
    pub passage: Option<(u32, i32)>,
    pub bounty_seen: Option<u32>,
    /// A transport boundary interrupted a camp rest. Workers already acted at
    /// its entrance; resuming the same rest must not add another hourly pass.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub resting: bool,
}

pub(crate) fn begin_absence(game: &mut Game) -> Absence {
    game.settle_renown(0);
    game.lineage.reveal_left = 1;
    game.batch = Batch::default();
    game.events.clear();
    game.offline = true;
    if !game.lineage.in_absence {
        game.lineage.absences += 1;
        game.lineage.in_absence = true;
    }
    game.watched = false;
    let rules = game.lineage.rules().clone();
    game.passage = crate::forecast::sim_passage(game, &rules);
    let day0 = (game.lineage.clock_s / crate::engine::DAY_S) as u32;
    crate::oath::new_day(&mut game.lineage, day0);
    if game.lineage.rest_watched {
        game.lineage.rest_left = 0;
        game.lineage.rest_watched = false;
    }
    if !game.lineage.pkg.literal {
        game.lineage.rest_left = game.lineage.rest_left.min(REST_CARRY_TICKS);
    }
    Absence {
        elapsed_s: 0, consumed: 0, hour: game.lineage.clock_s / 3600, day0,
        facts_before: game.lineage.facts.clone(), grew_before: crate::town::snap(&game.lineage),
        class: game.lineage.class.name().into(), rank_before: game.lineage.rank,
        acts_before: game.lineage.tree.acts.clone(), chest_before: game.lineage.tree.chest, gold_before: Some(game.lineage.gold),
        tally_before: Some(game.lineage.gold_tally.clone()),
        passage: game.passage, bounty_seen: game.bounty_seen, resting: false,
    }
}

pub fn run_offline_slice(game: &mut Game, elapsed_s: u64, last: bool) -> ReturnReport {
    run_offline_partition(game, elapsed_s, false, last, last, true)
}

pub const TICKS_PER_SECOND: u64 = 10;
pub const STALL_RUNS: u32 = 20;
pub const SAMPLE_RUNS: u32 = 20;
/// Stall verdict: this many runs home in a row with no death and no new depth.
pub const STALL_MIN_RUNS: u32 = 4;
/// A stall patch must move the forecast at the stall depth + 1 by more than this.
pub const STALL_DELTA: f64 = 0.02;
pub const STALL_SHOWN: usize = 3;
/// Quick reports estimate optional plateau changes on a bounded paired sample.
/// Full reports and direct analysis keep FORECAST_SIMS.
pub const QUICK_STALL_SIMS: u32 = 12;
/// Cut 30 §1: the most rest an idle-floor hero carries into a new absence (ticks: 10 minutes).
pub const REST_CARRY_TICKS: u32 = 10 * 60 * 10;

pub fn run_offline(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, true, true)
}

/// Same batch, but the worst death is returned by id only (no verdict or patch forecasts, which
/// cost ~3 s); the client calls `death(id)` once at the end of a chunked absence.
pub fn run_offline_quick(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, false, true)
}

/// `run_offline_quick` for tools that read the batch and the report's counts, never its
/// `stall` (the stall verdict's patch forecasts were 50–80 % of a gate job's CPU — the plateau
/// the player reads, not a count; docs/ITERATION_SPEED.md round 3): `stall` is `None`, and
/// nothing else differs — the verdict only reads the game (its memo, `stall_cache`, is read by
/// nothing but the verdict).
pub fn run_offline_counts(game: &mut Game, elapsed_s: u64) -> ReturnReport {
    run_offline_with(game, elapsed_s, false, false)
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

fn run_offline_with(game: &mut Game, elapsed_s: u64, full: bool, with_stall: bool) -> ReturnReport {
    run_offline_partition(game, elapsed_s, full, with_stall, true, false)
}

fn run_offline_partition(game: &mut Game, elapsed_s: u64, full: bool, with_stall: bool, last: bool, transport: bool) -> ReturnReport {
    let mut absence = game.offline_absence.take().unwrap_or_else(|| begin_absence(game));
    absence.elapsed_s = absence.elapsed_s.saturating_add(elapsed_s);
    let budget = absence.elapsed_s.saturating_mul(TICKS_PER_SECOND);
    let mut consumed = absence.consumed;
    let mut hour = absence.hour;
    let mut stall = game.stall_runs;
    let mut sampled = false;
    while consumed < budget {
        let now = (game.lineage.clock_s + consumed / TICKS_PER_SECOND) / 3600;
        if !absence.resting && now > hour && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            hour = now;
            crate::tree::at_hour(game);
        }
        // Cut 30.5 (the owner: manual send first): before the scout the hero home waits — an absence
        // yields at most the run in flight (a send by hand under way)
        if !game.may_go() && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            break;
        }
        // Camp rest first (the heir at camp: no run, or a run not yet begun).
        if game.lineage.rest_left > 0 && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            // QA on e75ec29 (qaQ: `rested 333m` beside 16 runs × `rest 20m`): the report's
            // `rested` is the rest each of its runs earned (counted at the run's end, below),
            // not the clock's — the camp rest a report begins with is the previous run's, and
            // the last run's is still running at the return.
            let used = game.rest_tick((budget - consumed).min(u32::MAX as u64) as u32);
            consumed += used as u64;
            if game.lineage.rest_left > 0 {
                absence.resting = true;
                break;
            }
        }
        absence.resting = false;
        if game.run.is_none() {
            game.start_run(None);
            crate::tree::scout_sent(game);
            game.events.clear();
        }
        let mut settled = false;
        while game.run.as_ref().is_some_and(|r| r.over.is_none()) && consumed < budget {
            let ticks = game.tick_batch((budget - consumed).min(u64::from(u32::MAX)) as u32, &mut settled);
            game.events.clear();
            consumed += u64::from(ticks);
        }
        // Cut 12: a begun run finishes past the budget (≤ one run), so an absence always ends
        // at camp and the next send packs what the player bought and runs the rules they
        // edited (cohort 8: a return's first run resumed a night-old run from D2 with an
        // empty pack, and its trace read `no item ← never found` beside 5/5 supplies).
        // Cut 26 (seam): a rest that ends inside the absence — to its last tick — sends the next
        // heir, whose run finishes past the budget like any begun run (a 20-minute break after a
        // 20-minute rest yields its run).
        if last && game.run.as_ref().is_some_and(|r| r.over.is_none() && (r.turn > 0 || consumed >= budget)) {
            let before = game.run.as_ref().unwrap().turn;
            game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            game.events.clear();
            consumed += (game.run.as_ref().unwrap().turn - before) as u64;
        }
        if game.run.as_ref().is_some_and(|r| r.over.is_some()) {
            settle_completed_run(game, consumed, &mut stall);
            if !transport && game.lineage.pkg.literal && stall >= STALL_RUNS && consumed < budget {
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
                    crate::tree::scout_sent(game);
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
                    game.lineage.town.today = ((game.lineage.clock_s + (consumed + ticks) / TICKS_PER_SECOND) / crate::engine::DAY_S) as u32;
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
                    if !game.sim { crate::legacy::ensure(&mut game.lineage);game.lineage.bloodline.as_mut().expect("bloodline").points+=extra as u32; }
                    // RUNS_UI: the extrapolated runs, one record in the log (`+N`)
                    if extra > 0 {
                        let rec = RunRec { via: "away".into(), absence: Some(game.lineage.absences), clock_s: game.lineage.clock_s + budget / TICKS_PER_SECOND, tier: "return".into(), sampled: Some(extra as u32), ..Default::default() };
                        game.lineage.push_run(rec);
                    }
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
                            *game.stall.absent_rows.entry(*r).or_insert(0) += share(*k);
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
    // A final zero-length transport call can close a run left by the previous slice.
    if last && absence.elapsed_s > 0 && game.run.as_ref().is_some_and(|r| r.over.is_none()) {
        let before = game.run.as_ref().unwrap().turn;
        game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
        consumed += u64::from(game.run.as_ref().unwrap().turn - before);
        game.events.clear();
        settle_completed_run(game, consumed, &mut stall);
    }
    game.stall_runs = stall;
    if !last {
        absence.consumed = consumed;
        absence.hour = hour;
        absence.passage = game.passage;
        game.offline_absence = Some(absence);
        return ReturnReport { slice_pending: true, ..Default::default() };
    }
    let elapsed_s = absence.elapsed_s;
    let day0 = absence.day0;
    // (the hours the absence ran past its last send: the workers' last acts, before the report)
    if (game.lineage.clock_s + budget / TICKS_PER_SECOND) / 3600 > hour {
        crate::tree::at_hour(game);
    }
    game.offline = false;
    // Cut 29 §1: the night's mark — ◆1 for each day this absence covered whose sends came home (a
    // bank or a return), each day once.
    game.lineage.clock_s += elapsed_s;
    // Cut 30 §5: the board at the return shows the day's quest (a quest kept yesterday gives way to
    // today's draw, from the record the absence reached).
    game.lineage.town.today = (game.lineage.clock_s / crate::engine::DAY_S) as u32;
    crate::town::roll(&mut game.lineage);
    if elapsed_s > 0 && game.batch.banked + game.batch.returned > 0 {
        let last = ((game.lineage.clock_s - 1) / crate::engine::DAY_S) as u32;
        let first = day0.max(game.lineage.mark_day);
        if last >= first {
            let n = last - first + 1;
            game.lineage.marks += n;
            game.lineage.mark_day = last + 1;
            game.batch.night_marks += n;
            game.batch.marks += n;
        }
    }
    let mut r = report_with(game, elapsed_s, &absence.facts_before, &absence.class, absence.rank_before, sampled, full, with_stall);
    r.grew = crate::town::grew(&absence.grew_before, &crate::town::snap(&game.lineage));
    r.workers = crate::tree::report_acts(&game.lineage, &absence.acts_before, &game.lineage.tree.acts);
    supply_reason(&mut r);
    r.chest = (game.lineage.tree.chest - absence.chest_before).max(0);
    set_net(&mut r, absence.gold_before, game.lineage.gold);
    set_terms(&mut r, &game.lineage, absence.tally_before.as_ref(), &absence.acts_before);
    // Cut 113 §3: the return carries a pick (or grows the one waiting)
    crate::returns::on_return(&mut game.lineage, elapsed_s);
    r.pick = crate::returns::wire(&game.lineage);
    // Cut 118: the finds opened, the notable acts since the last report
    crate::feats::on_report(&mut game.lineage, &mut r);
    crate::pets::on_report(&game.lineage, &mut r);
    // (a system's reveal is a beat of the five)
    if !r.systems_opened.is_empty() {
        r.packages = crate::packages::beats_n(&game.batch.pkg_lines, crate::packages::BEATS - 1);
    }
    r
}

/// The purse's change over the absence on the report's gold (`GoldSummary.net`); none for an
/// absence saved before the baseline existed.
pub(crate) fn set_net(r: &mut ReturnReport, before: Option<i32>, after: i32) {
    if let (Some(g), Some(b)) = (r.gold.as_mut(), before) {
        g.net = Some(after - b);
    }
}

/// Cut 117 §4 (blind 8cf9050 B: `Supplies limited · $0 budget` after the apprentice's `−$6750`): the
/// apprentice's line says why the supplies were limited (`WorkerAct.reason`, the report's
/// `SupplyBudget.reason`) — the repeat spends only the absence's income, never his forge purse.
pub(crate) fn supply_reason(r: &mut ReturnReport) {
    let Some(why) = r.supply_budget.as_ref().map(|b| b.reason.clone()) else { return };
    if let Some(a) = r.workers.iter_mut().find(|w| w.id == "apprentice") {
        a.reason = Some(why);
    }
}

/// Cut 117 §1: the absence's ledger (`GoldSummary.ledger`) — the lineage's tally since `before`, by term, with
/// the exits' unkept carry split out (`carried` + `lost`) and the apprentice's forge steps named apart from
/// the hand's. The terms sum to `net` exactly: any movement that bypassed `gold_move` lands in `other`.
pub(crate) fn set_terms(r: &mut ReturnReport, l: &crate::engine::LineageState, before: Option<&BTreeMap<String, i64>>, acts_before: &BTreeMap<String, u32>) {
    let (Some(g), Some(before)) = (r.gold.as_mut(), before) else { return };
    let Some(net) = g.net else { return };
    let d = |k: &str| -> i64 { l.gold_tally.get(k).copied().unwrap_or(0) - before.get(k).copied().unwrap_or(0) };
    let key = crate::tree::APPRENTICE_SPENT;
    let appr = i64::from(l.tree.acts.get(key).copied().unwrap_or(0).saturating_sub(acts_before.get(key).copied().unwrap_or(0)));
    let forge = d("forge");
    let appr = appr.min((-forge).max(0));
    let lost = i64::from(g.lost.max(0));
    let mut terms: Vec<(&str, i64)> = vec![
        ("carried", d("earned") + lost),
        ("lost", -lost),
        ("heir", d("heir")),
        ("apprentice", -appr),
        ("forge", forge + appr),
        ("works", d("works")),
        ("supplies", d("supplies")),
        ("tolls", d("tolls")),
        ("hires", d("hires")),
        ("bank", d("bank")),
        ("sinks", d("sinks")),
        ("recovered", d("recovered")),
        ("fetched", d("fetched")),
        ("other", d("other")),
    ];
    let sum: i64 = terms.iter().map(|t| t.1).sum();
    if sum != i64::from(net) {
        terms.last_mut().unwrap().1 += i64::from(net) - sum;
    }
    let terms: Vec<crate::wire::GoldTerm> = terms.into_iter().filter(|t| t.1 != 0).map(|(k, v)| crate::wire::GoldTerm { label: k.into(), amount: v.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32 }).collect();
    let earned: i32 = terms.iter().filter(|t| t.amount > 0).map(|t| t.amount).sum();
    let spent: i32 = -terms.iter().filter(|t| t.amount < 0).map(|t| t.amount).sum::<i32>();
    g.ledger = Some(crate::wire::GoldLedger { earned, spent, net: earned - spent, terms });
}

fn settle_completed_run(game: &mut Game, consumed: u64, stall: &mut u32) {
    game.lineage.town.today = ((game.lineage.clock_s + consumed / TICKS_PER_SECOND) / crate::engine::DAY_S) as u32;
    game.clock_at = Some(game.lineage.clock_s + consumed / TICKS_PER_SECOND);
    let outcome = game.finish_run().unwrap_or_default();
    game.clock_at = None;
    game.batch.rested += u64::from(game.lineage.rest_left);
    game.auto_keep();
    game.events.clear();
    if outcome.new_facts == 0 && !outcome.new_best { *stall += 1; } else { *stall = 0; }
}

/// At a multi-hero return, finish each outstanding run once, after the shared
/// wall-clock budget. Session supplies stable slot order and the current wallet.
pub(crate) fn finish_return_run(game: &mut Game) {
    if game.run.as_ref().is_none_or(|r| r.over.is_some()) { return; }
    let before = game.run.as_ref().unwrap().turn;
    game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
    let extra = u64::from(game.run.as_ref().unwrap().turn - before);
    let mut stall = game.stall_runs;
    settle_completed_run(game, extra, &mut stall);
    game.stall_runs = stall;
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

#[cfg(test)]
pub(crate) fn report(game: &mut Game, elapsed_s: u64, facts_before: &std::collections::BTreeSet<String>, class: &str, rank_before: u32, sampled: bool, full: bool) -> ReturnReport {
    report_with(game, elapsed_s, facts_before, class, rank_before, sampled, full, true)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn report_with(game: &mut Game, elapsed_s: u64, facts_before: &std::collections::BTreeSet<String>, class: &str, rank_before: u32, sampled: bool, full: bool, with_stall: bool) -> ReturnReport {
    let t = game.run.as_ref().map(|r| r.turn).unwrap_or(0);
    game.settle_renown(t);
    let learned: Vec<String> = game.lineage.facts.difference(facts_before).cloned().collect();
    let worst_death_id = game.batch.worst_death;
    let worst_death = if full { worst_death_id.and_then(|id| crate::trace::death(game, id)) } else { None };
    let pending = crate::meta::pending(game);
    let stall = if with_stall { stall_verdict_with(game, if full { crate::forecast::FORECAST_SIMS } else { QUICK_STALL_SIMS }) } else { None };
    // Cut 29 §1 (E1): the wall's edit is not searched here (20–60 s native at a wall, minutes in
    // wasm, inside an offline slice): the client asks `Game::wall_edit` on the report.
    // Cut 28b: the lineage's first plateau (the stall's window, verdict or not) opens the oath board
    game.lineage.oath_open |= game.stall.runs >= STALL_MIN_RUNS;
    // Cut 29 §2: the first plateau opens the order and the vs line (and the oaths).
    let opened = crate::systems::update(&mut game.lineage, game.stall.runs >= STALL_MIN_RUNS);
    game.batch.systems_opened.extend(opened);
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
    // QA on 92eb880: the coins the ledger paid per kind, when the batch recorded them (a save
    // from before carries cents only, apportioned below) — the exit sheet's own numbers.
    let paid = b.salvaged.keys().all(|k| b.salvaged_coins.contains_key(k));
    // QA on 1a2a4a9: an unidentified kind reads as its flavour (`LineageState::wire_name`).
    let mut salvaged: Vec<SalvageRow> = b.salvaged.iter().map(|(k, (n, g))| SalvageRow { kind: game.lineage.wire_name(k), n: *n, gold: if paid { b.salvaged_coins[k] } else { g / 100 } }).collect();
    let mut order: Vec<(i32, usize)> = if paid { Vec::new() } else { b.salvaged.values().enumerate().map(|(i, (_, g))| (g % 100, i)).collect() };
    order.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let short = total - salvaged.iter().map(|r| r.gold).sum::<i32>();
    for &(_, i) in order.iter().take(short.max(0) as usize) {
        salvaged[i].gold += 1;
    }
    for &(_, i) in order.iter().rev().filter(|&&(_, i)| salvaged[i].gold > 0).take((-short).max(0) as usize).collect::<Vec<_>>() {
        salvaged[i].gold -= 1;
    }
    let oath = b.oath.as_ref().map(|(o, runs, kept, done)| crate::wire::OathReport { id: o.id.clone(), chips: crate::oath::chips(o), text: crate::oath::text(o), runs: *runs, kept: *kept, done: *done, reward: Some(o.reward.clone()), price: o.price,
        broken: b.oath_breaks.values().sum(), cause: b.oath_breaks.iter().max_by_key(|(c, n)| (**n, std::cmp::Reverse(c.len()))).map(|(c, _)| c.clone()) });
    let mut r = ReturnReport { finds: None, feats: Vec::new(), pick: None, legacy_earned:b.legacy_earned,slice_pending:false,bloodlines:vec![],lead: Vec::new(), oath, grew: Vec::new(), workers: Vec::new(), chest: 0, packages: crate::packages::beats(&b.pkg_lines),
        elapsed_s,
        runs: b.runs,
        sampled,
        learned,
        bests: b.bests.clone(),
        found: b.found.iter().map(|i| to_inv(i, &game.lineage.facts, &game.lineage.flavours)).collect(),
        new_finds: b.new_finds.clone(),
        deaths,
        pending,
        reel,
        marks_earned: b.marks,
        night_marks: b.night_marks,
        systems_opened: b.systems_opened.clone(),
        meters: (!b.meters.is_empty()).then(|| crate::meters::wire_for(&b.meters, Some(&game.lineage.pkg))),
        fallen: b.fallen.clone(),
        oaths_kept: b.oaths_kept.iter().map(|o| crate::wire::OathReward { kind: o.kind.clone(), id: o.id.clone(), label: crate::oath::text(o) }).collect(),
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
        stolen: {
            // QA on a946e04: named now, by the lineage's facts now (`blue potion?` → `poison`),
            // and a kind twice (two labels of one kind) is one row.
            let mut v: Vec<crate::wire::StolenRow> = Vec::new();
            for (k, n) in &b.stolen {
                let label = game.lineage.wire_name(k).replace('_', " ");
                let gold = b.stolen_gold.get(k).copied().unwrap_or(0);
                match v.iter_mut().find(|r| r.label == label) {
                    Some(r) => {
                        r.n += n;
                        r.gold += gold;
                    }
                    None => v.push(crate::wire::StolenRow { label, n: *n, gold }),
                }
            }
            v.sort_by(|a, b| b.n.cmp(&a.n).then(a.label.cmp(&b.label)));
            v
        },
        stall,
        start_short: b.start_short.map(|(depth, toll, runs)| crate::wire::StartShort { depth, toll, runs }),
        stolen_gold: b.stolen_gold.values().sum(),
        swapped: b.swapped,
        deepest: b.run_outcomes.iter().map(|(d, _)| *d).max().unwrap_or(0),
        // Cut 13 §3: the night's ledger — what the automations bought, per kind in coins.
        stalled: b.stalls,
        driven: b.driven_off,
        spent: b.spent.iter().map(|(k, (n, g))| SalvageRow { kind: game.lineage.wire_name(k).replace('_', " "), n: *n, gold: *g }).filter(|r| r.gold > 0).collect(),
        gold: Some(crate::wire::GoldSummary { home: b.gold_earned, salvage: b.salvage_gold, wake: b.wake_pay, spent: b.spent.values().map(|(_, g)| *g).sum(), wake_cap: crate::engine::WAKE_PAY, wake_n: b.wake_n, lost: b.gold_lost, unkept: b.gold_unkept, passage: b.passage, recovered: b.recovered_gold, fetched: b.fetched_gold, net: None, ledger: None }),
        exits: b.exits.clone(),
        picked: game.lineage.picked_clean(),
        restock_capped: b.restock_capped,
        bounty: b.bounty.clone(),
        drives: b.drives.clone(),
        lanes: {
            let deepest = b.run_outcomes.iter().map(|(d, _)| *d).max().unwrap_or(0);
            game.lineage.rules().route().lanes().into_iter().filter(|(a, _, _)| deepest >= *a).map(|(a, z, biome)| format!("D{a}–{z} · {}", biome.title())).collect()
        },
        repeat_short: b.repeat_short,
        supply_budget: (b.restock_capped || b.repeat_short).then(|| {
            let (income, spent) = (b.income(), b.spent_total());
            let reason = if b.repeat_short { "purse_short" } else if income <= 0 { "no_income" } else { "income_spent" };
            crate::wire::SupplyBudget { income, spent, left: (income - spent).max(0), reason: reason.into() }
        }),
        shelved: b.shelved.iter().map(|(k, (n, g))| SalvageRow { kind: game.lineage.wire_name(k).replace('_', " "), n: *n, gold: *g }).collect(),
        heirs: b.heirs.map(|(lo, hi)| vec![lo, hi]).unwrap_or_default(),
    };
    r.lead = lead_of(game, &r);
    r
}

/// Cut 28 §2: the lines of a report's first screen (`ReturnReport.lead`).
pub const LEAD_MAX: usize = 4;

/// Cut 28 §2 (both cohort-23 raters: reports opened with salvage walls): what changed and what to do,
/// decisions first — the oath (kept, or its count), the plateau, a counter learned, a record, the
/// worst death's verdict, a drive-off, the bounty, the first pending decision; ≤ `LEAD_MAX`, each
/// ≤ 6 words. Salvage, bones and spending are the details under it.
pub fn lead_of(game: &Game, r: &ReturnReport) -> Vec<crate::wire::ReportLead> {
    let mut out: Vec<crate::wire::ReportLead> = Vec::new();
    let mut push = |k: &str, text: String| out.push(crate::wire::ReportLead { k: k.into(), text });
    if let Some(o) = &r.oath {
        // Cut 28b: a send that broke it says what did (`oath broken: R2 return ×11`)
        push("oath", match (&o.cause, o.done) {
            (_, true) => format!("oath kept: {}", o.text),
            (Some(c), false) if o.broken > 0 => format!("oath broken: {c} ×{}", o.broken),
            _ => format!("oath {} · {}/{}", o.text, o.kept, o.runs),
        });
    }
    if r.stall.is_some() {
        push("plateau", format!("plateau: none past D{}", game.lineage.best_depth));
    }
    for f in r.learned.iter().filter(|f| f.starts_with("boss:") && f.contains(":counter")) {
        let kind = f.trim_start_matches("boss:").split(":counter").next().unwrap_or("");
        push("counter", format!("{} learned", crate::oath::counter_fact(&game.lineage, kind)));
    }
    if let Some(best) = r.bests.iter().find(|b| b.starts_with('D') || b.starts_with("boss:")) {
        let text = match best.strip_prefix("boss: ") {
            Some(kind) => format!("first: {} slain", crate::sifter::boss_short(kind)),
            None => format!("record {best}"),
        };
        push("record", text);
    }
    if let Some(d) = &r.worst_death {
        push("death", format!("D{} death · {}", d.depth, d.lean.as_deref().filter(|_| d.verdict != "dice").map(|l| format!("{} · {l}", d.verdict)).unwrap_or_else(|| d.verdict.clone())));
    } else if let Some(c) = r.deaths.first() {
        push("death", format!("{} {} · {}", c.n, if c.n == 1 { "death" } else { "deaths" }, c.cause.replace('_', " ")));
    }
    if let Some(d) = r.drives.last() {
        push("driven", format!("driven off: {} · {}", d.title, crate::oath::counter_fact(&game.lineage, &d.boss)));
    }
    if let Some(b) = &r.bounty {
        push("bounty", if b.taken { format!("bounty D{} taken ${}", b.depth, b.gold) } else { format!("bounty D{} · missed · reach", b.depth) });
    }
    if let Some(p) = r.pending.iter().find(|p| p.starts_with("unlock ") || p.starts_with("forge ") || p.starts_with("patch ")) {
        push("pending", crate::chronicle::clamp_words(p, 6));
    }
    out.truncate(LEAD_MAX);
    out
}

// ---------------------------------------------------------------- stall verdict

/// The stall verdict: when the last ≥ 4 runs all came home (no death, no new depth) the row that
/// ended most of them is named and up to three patches are forecast at the stall depth + 1.
/// The patches are cached per (row, depth, rules, vocabulary): a chunked absence asks every slice.
pub fn stall_verdict(game: &mut Game) -> Option<Stall> {
    stall_verdict_with(game, crate::forecast::FORECAST_SIMS)
}

fn stall_verdict_with(game: &mut Game, sims: u32) -> Option<Stall> {
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
    // QA on 92eb880 (qaN: `R7 bank ended 13 runs` beside `11 BANKED`): the window can open
    // before the absence; the runs the report's tiles count are said apart from the earlier ones.
    let here = t.absent_rows.get(&row).copied().unwrap_or(0).min(fired);
    let verb = ending.verb.short();
    let text = match fired - here {
        0 => format!("R{} {verb} ended {fired} runs, none past D{depth}", row + 1),
        before if here == 0 => format!("R{} {verb} ended {before} earlier runs, none past D{depth}", row + 1),
        before => format!("R{} {verb} ended {here} runs, {before} before; none past D{depth}", row + 1),
    };
    let vocab = game.vocabulary();
    let key = format!("{row}:{depth}:{sims}:{}:{}:{}", serde_json::to_string(&rules).unwrap_or_default(), vocab.conds.len(), vocab.verbs.len());
    let patches = match &game.stall_cache {
        Some((k, p)) if *k == key => p.clone(),
        _ => {
            let p = stall_patches(game, &rules, row as usize, &ending, depth, sims);
            game.stall_cache = Some((key, p.clone()));
            p
        }
    };
    let trace = game.stall.traces.get(&row).cloned();
    let mut patches = patches;
    crate::trace::mark_exits(&mut patches);
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
    // Cut 25 §2: a move — the set's own row goes above the row at `insert_at`.
    if let Some(from) = p.moves_from.and_then(|f| usize::try_from(f).ok()) {
        if from < r.rows.len() && at < from {
            let row = r.rows.remove(from);
            r.rows.insert(at, row);
        }
        return r;
    }
    if p.remove {
        if at < r.rows.len() {
            r.rows.remove(at);
        }
    } else if p.replace && at < r.rows.len() {
        r.rows[at] = p.row.clone();
    } else {
        let mut at = at.min(r.rows.len());
        // Cut 19 §4: on a full set the patch's `drops` row makes the room (`+ drop R5`: the
        // least-fired own row), not the last one.
        if let Some(d) = p.drops.and_then(|d| usize::try_from(d).ok()) {
            if r.own_rows() >= max_rows && d < r.rows.len() && !r.rows[d].is_card() {
                r.rows.remove(d);
                if d < at {
                    at -= 1;
                }
            }
        }
        r.rows.insert(at, p.row.clone());
        // Cut 12 §1: the cap is on the player's own rows; a card row never falls off.
        r = r.fit(max_rows.max(1));
    }
    r
}

/// Candidates: the ending row pushed 10 points deeper, that row removed, the boss counter when
/// the stall floor is a boss floor and the boss is known, `hp<90 → rest` when absent. Kept
/// when the forecast at depth + 1 moves by more than `STALL_DELTA`, ranked by delta.
fn stall_patches(game: &Game, rules: &RuleSet, row: usize, ending: &Row, depth: u32, sims: u32) -> Vec<Patch> {
    let target = depth + 1;
    let max_rows = game.lineage.max_rows();
    let vocab = game.vocabulary();
    let has_verb = |v: &Verb| vocab.verbs.contains(v);
    let has_cond = |k: &str, t: Option<&str>| vocab.conds.iter().any(|c| c.k == k && (t.is_none() || c.t.as_deref() == t));
    let present = |r: &Row| rules.rows.contains(r);
    let patch = |row: Row, at: usize, replace: bool, remove: bool| Patch { no_gain: false, row, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace, remove, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None, whole: None, gem: false, restores: None };
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
    if let Some(kind) = rules.route().boss(depth) {
        let facts = &game.lineage.facts;
        let known = crate::facts::boss_counter_known(facts, kind) || facts.contains(&format!("foe:{kind}"));
        if known && has_cond("foe_tag", Some("boss")) {
            let boss = Cond::t("foe_tag", "boss");
            let conds = boss_attack_conds(game, rules, depth, kind, has_cond);
            if let Some(conds) = conds {
                let attack = Row::new(conds, Verb::arg("attack", "tag:boss"));
                if has_verb(&attack.verb) && !present(&attack) {
                    cands.push(patch(attack, 0, false, false));
                }
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
    // QA on 0c6e126 (qaZ: PLATEAU `reach D7 17% · base 8%` between the camp's `D7 16%` and `21%` for the same set): the base
    // is the camp's own measure — its first pass's seeds (`forecast_tag` at the lineage's frontier) and sims — and every
    // candidate replays exactly those, so the plateau's base is the camp's number and a patch's reach its paired move.
    // QA on 524827b (qaAB: PLATEAU `foe: boss → attack boss · drops one` while a death's patch read
    // `drops R6 · 11/1026 fires`): an insert onto a full set names the row it drops — the own row
    // the absence's sends fired least in (`Batch.row_runs`; ties: the lowest in the list), never a
    // card, an exit, the stall's own row or a row of the patch's verb — as `apply_patch` drops it (set before the measure: the numbers are that set's).
    if rules.own_rows() >= max_rows {
        for p in cands.iter_mut().filter(|p| !p.replace && !p.remove) {
            p.drops = rules
                .rows
                .iter()
                .enumerate()
                .filter(|(i, r)| *i != row && !r.is_card() && !matches!(r.verb.v.as_str(), "return" | "bank") && r.verb.v != p.row.verb.v)
                .map(|(i, _)| (game.batch.row_runs.get(i).copied().unwrap_or(0), i))
                .fold(None, |best: Option<(u32, usize)>, x| if best.is_none_or(|b| x.0 <= b.0) { Some(x) } else { best })
                .map(|(_, i)| i as i32);
        }
    }
    let tag = crate::forecast::forecast_tag(game, game.lineage.best_depth + 1);
    let budget = crate::forecast::CAMP_TICK_BUDGET;
    let (base, n) = crate::forecast::reach_counted(game, rules, target, sims, tag, budget);
    for p in cands.iter_mut() {
        let edited = crate::forecast::edited_game(game, &apply_patch(rules, p, max_rows));
        let r = crate::forecast::reach_paired(&edited, edited.lineage.rules(), target, n, tag);
        p.survive = r;
        p.forecast_delta = r - base;
    }
    cands.retain(|p| p.forecast_delta > STALL_DELTA);
    cands.sort_by(|a, b| b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap());
    cands.truncate(STALL_SHOWN);
    cands
}

/// Blind 3ab97ea (A: `Target the boss · reach D29 +38` at the Queen's stall, applied — then the Mirror King:
/// `attack boss 100% · DEALT 0 dps`): a stall's `foe: boss → attack boss` is every boss's row. When a boss
/// deeper on the route that the lineage has met turns blows back (the King's mirror, the Master's
/// reflection), the suggestion is scoped to this boss's biome (`foe: boss · in: deep`); without that
/// condition it is not made (None).
pub(crate) fn boss_attack_conds(game: &Game, rules: &RuleSet, depth: u32, kind: &str, has_cond: impl Fn(&str, Option<&str>) -> bool) -> Option<Vec<Cond>> {
    let facts = &game.lineage.facts;
    let boss = Cond::t("foe_tag", "boss");
    let biome = rules.route().biome(depth).name();
    let turns_blows = |k: &str| crate::defs::MONSTERS.iter().find(|m| m.kind == k).is_some_and(|m| m.tags.iter().any(|t| matches!(*t, "mirror" | "reflect_melee")));
    let met = |k: &str| crate::facts::boss_counter_known(facts, k) || facts.contains(&format!("foe:{k}"));
    let deeper = crate::descent::BOSS_DEPTHS.iter().any(|&(k, d)| d > depth && k != kind && turns_blows(k) && met(k));
    if !deeper {
        Some(vec![boss])
    } else if has_cond("in", Some(biome)) {
        Some(vec![boss, Cond::t("in", biome)])
    } else {
        None
    }
}

/// RUNS_UI: the open app's clock on the lineage — `elapsed_ms` of rest, then the next run, unwatched,
/// as an absence plays them, except that a run in flight at the budget's end stays in flight (the
/// lane shows it live; the watch can take it up), nothing is sampled, the run settles its renown as a
/// watched one does (`offline` stays false: the log's `town`), no reveal is reset and no rest is
/// capped. Cheap when nothing ends: no report, no forecast (the passage is priced at a send only).
pub fn advance(game: &mut Game, elapsed_ms: u64) -> Advance {
    let ms = game.advance_rem.0 + elapsed_ms;
    game.advance_rem.0 = ms % 100;
    let budget = ms / 100;
    game.lineage.in_absence = false;
    game.watched = false;
    let mut out = Advance::default();
    if budget == 0 {
        out.live = game.live_run();
        return out;
    }
    let clock0 = game.lineage.clock_s;
    let day0 = (clock0 / crate::engine::DAY_S) as u32;
    crate::oath::new_day(&mut game.lineage, day0);
    let mut hour = clock0 / 3600;
    let mut consumed: u64 = 0;
    let mut home = 0u32;
    while consumed < budget {
        let now = (clock0 + consumed / TICKS_PER_SECOND) / 3600;
        if now > hour && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            hour = now;
            crate::tree::at_hour(game);
        }
        // (before the scout the hero home waits: only a send by hand under way runs)
        if !game.may_go() && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            break;
        }
        if game.lineage.rest_left > 0 && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            consumed += game.rest_tick((budget - consumed).min(u32::MAX as u64) as u32) as u64;
            if game.lineage.rest_left > 0 {
                break;
            }
        }
        if consumed >= budget {
            break;
        }
        if game.run.is_none() {
            let rules = game.lineage.rules().clone();
            game.passage = crate::forecast::sim_passage(game, &rules);
            game.start_run(None);
            crate::tree::scout_sent(game);
            game.events.clear();
        }
        let mut settled = false;
        while game.run.as_ref().is_some_and(|r| r.over.is_none()) && consumed < budget {
            let ticks = game.tick_batch((budget - consumed).min(u64::from(u32::MAX)) as u32, &mut settled);
            game.events.clear();
            consumed += u64::from(ticks);
        }
        if game.run.as_ref().is_some_and(|r| r.over.is_some()) {
            game.lineage.town.today = ((clock0 + consumed / TICKS_PER_SECOND) / crate::engine::DAY_S) as u32;
            game.clock_at = Some(clock0 + consumed / TICKS_PER_SECOND);
            let id = game.run.as_ref().map_or(0, |r| r.id);
            let outcome = game.finish_run().unwrap_or_default();
            game.clock_at = None;
            game.auto_keep();
            game.events.clear();
            out.ended.push(id);
            if matches!(outcome.tier, Some(ExitTier::Bank) | Some(ExitTier::Return)) {
                home += 1;
            }
            if outcome.new_facts == 0 && !outcome.new_best {
                game.stall_runs += 1;
            } else {
                game.stall_runs = 0;
            }
        }
    }
    // the clock moves by the whole budget (a hero who waits still lets the hours pass)
    let ticks = game.advance_rem.1 + budget;
    game.advance_rem.1 = ticks % TICKS_PER_SECOND;
    game.lineage.clock_s += ticks / TICKS_PER_SECOND;
    let clock = game.lineage.clock_s;
    if clock / 3600 > hour && game.run.as_ref().is_none_or(|r| r.turn == 0) {
        crate::tree::at_hour(game);
    }
    if clock / crate::engine::DAY_S != clock0 / crate::engine::DAY_S || !out.ended.is_empty() {
        game.lineage.town.today = (clock / crate::engine::DAY_S) as u32;
        crate::town::roll(&mut game.lineage);
    }
    // Cut 29 §1's night mark, as an absence pays it: ◆1 a day whose sends came home
    if home > 0 {
        let last = (clock.saturating_sub(1).max(clock0) / crate::engine::DAY_S) as u32;
        let first = day0.max(game.lineage.mark_day);
        if last >= first {
            game.lineage.marks += last - first + 1;
            if game.offline { game.batch.night_marks += last - first + 1; }
            game.lineage.mark_day = last + 1;
        }
    }
    out.live = game.live_run();
    out
}
#[cfg(test)]
mod slice_tests {
    use super::*;

    fn camp(seed:u64)->Game {
        let mut g=Game::new_resident(seed);
        crate::tree::grant(&mut g.lineage,&["porter","scout"]);
        g
    }
    fn partition(g:&Game, total:u64, widths:&[u64], reload:bool)->(Game,ReturnReport) {
        let mut g=g.clone();let mut left=total;let mut i=0;
        loop {
            let seconds=left.min(widths[i%widths.len()]);let last=seconds==left;
            let r=run_offline_slice(&mut g,seconds,last);
            if last { return (g,r); }
            assert!(r.slice_pending && r.elapsed_s==0 && r.runs==0);
            assert!(g.offline && g.offline_absence.is_some());
            left-=seconds;i+=1;
            if reload { g=Game::load(&g.save()).unwrap(); }
        }
    }
    fn equal(a:&Game,b:&Game,label:&str) {
        assert!(serde_json::to_value(a).unwrap()==serde_json::to_value(b).unwrap(),"complete saved state differs: {label}");
    }
    #[test]
    fn offline_slice_partition_preserves_complete_state_and_report() {
        for seed in [1,3,5] {
            let mut base=camp(seed);run_offline_counts(&mut base,3600);
            let mut whole=base.clone();let expected=run_offline_quick(&mut whole,8*3600);
            // (Cut 120 §1: the points the Legacy order spent count as earned)
            let spent=|g:&Game|g.lineage.tree.acts.get(crate::tree::LEGACY_POINTS).copied().unwrap_or(0);
            assert_eq!(expected.legacy_earned,crate::legacy::current(&whole.lineage).unwrap().points+spent(&whole)-crate::legacy::current(&base.lineage).unwrap().points-spent(&base));
            assert!(expected.legacy_earned>0);
            for (widths,reload) in [(&[1800][..],false),(&[1,1799,3601,719][..],false),(&[1800][..],true)] {
                let (actual,report)=partition(&base,8*3600,widths,reload);
                equal(&whole,&actual,&format!("seed{seed}, {widths:?}, reload{reload}"));
                assert_eq!(report,expected,"one final report includes the whole absence exactly once");
            }
        }
    }
    #[test]
    fn offline_slice_manual_home_and_inflight_are_bounded_and_reloadable() {
        for sent in [false,true] {
            let mut base=Game::new_resident(3);if sent { base.send(); }
            let mut whole=base.clone();let expected=run_offline_quick(&mut whole,86401);
            let (actual,r)=partition(&base,86401,&[1,600,1799],true);
            equal(&whole,&actual,"manual hero across day boundary");
            assert_eq!(r,expected);assert_eq!(r.runs,u32::from(sent));assert!(actual.run.is_none());
        }
    }
    #[test]
    fn offline_slice_days_workers_quests_and_reveals_are_partition_independent() {
        let mut base=camp(3);base.lineage.clock_s=crate::engine::DAY_S-601;
        run_offline_counts(&mut base,3600);
        let mut whole=base.clone();let expected=run_offline_quick(&mut whole,2*crate::engine::DAY_S+1);
        let (actual,report)=partition(&base,2*crate::engine::DAY_S+1,&[1800],true);
        equal(&whole,&actual,"multiple days, workers, quests and reveals");
        assert_eq!(report,expected);
        let reveal_units:std::collections::BTreeSet<_>=report.systems_opened.iter()
            // (blind 1fb7786: the pen's group, the Mother met, opens outside the budget as the day-0 systems do)
            .filter(|id|!crate::systems::DAY0.contains(&id.as_str())&&!crate::systems::pen_due(&actual.lineage,id))
            .map(|id|crate::systems::SYSTEMS.iter().find(|s|s.id==id).unwrap())
            .map(|s|(s.trigger,s.min_age_h)).collect();
        assert!(reveal_units.len()<=1,"the curriculum budgets trigger groups, such as forge/loadout/exits, as one reveal");
        assert!(!report.slice_pending && !actual.offline && actual.offline_absence.is_none());
    }
    #[test]
    fn offline_slice_preserves_rest_and_final_zero_finishes_only_once() {
        let mut g=camp(3);g.lineage.rest_left=REST_CARRY_TICKS+900;
        let initial_clock=g.lineage.clock_s;
        assert!(run_offline_slice(&mut g,1,false).slice_pending);
        assert_eq!(g.lineage.rest_left,REST_CARRY_TICKS-10);
        assert!(run_offline_slice(&mut g,2,false).slice_pending);
        assert_eq!(g.lineage.rest_left,REST_CARRY_TICKS-30);
        assert_eq!(g.lineage.clock_s,initial_clock,"clock is committed at the real return");
        let mut whole=camp(3);whole.lineage.rest_left=REST_CARRY_TICKS+900;
        let expected=run_offline_quick(&mut whole,601);
        let mut sliced=camp(3);sliced.lineage.rest_left=REST_CARRY_TICKS+900;
        assert!(run_offline_slice(&mut sliced,601,false).slice_pending);
        assert!(sliced.run.as_ref().is_some_and(|r|r.over.is_none()));
        sliced=Game::load(&sliced.save()).unwrap();
        assert_eq!(run_offline_slice(&mut sliced,0,true),expected);
        equal(&whole,&sliced,"zero-second final boundary");
        let mut a=Game::new(3);let (b,r)=partition(&a,0,&[1],false);
        assert_eq!(run_offline_quick(&mut a,0),r);equal(&a,&b,"empty zero absence");
    }
    /// Cut 120: the new orders (Legacy, ranks) act at the same moments in a sliced absence as in a whole one.
    #[test]
    fn the_cut120_orders_are_slice_stable() {
        // (the interrupted rest of `interrupted_rest_does_not_insert_an_hourly_worker_action`, the orders on)
        let mut base=camp(3);
        crate::tree::grant(&mut base.lineage,&["clerk"]);
        base.lineage.town.built.push(("bank".into(),0));
        base.lineage.orders.legacy="balanced".into();base.lineage.orders.ranks="auto".into();
        base.lineage.day=crate::tree::RANK_DAYS[0];
        crate::legacy::ensure(&mut base.lineage);base.lineage.bloodline.as_mut().unwrap().points=40;
        base.lineage.gold_move(10000,"fixture income");
        base.lineage.clock_s=3590;base.lineage.rest_left=6000;
        let mut whole=base.clone();let expected=run_offline_quick(&mut whole,4000);
        assert!(whole.lineage.tree.acts.get(crate::tree::LEGACY_ACT).is_some(),"the order bought");
        for reload in [false,true] {
            let(actual,report)=partition(&base,4000,&[60,1000],reload);
            equal(&whole,&actual,"the Cut 120 orders across slices");
            assert_eq!(report,expected);
        }
    }
    #[test]
    fn interrupted_rest_does_not_insert_an_hourly_worker_action() {
        let mut base=camp(3);
        crate::tree::grant(&mut base.lineage,&["clerk"]);
        base.lineage.town.built.push(("bank".into(),0));
        base.lineage.gold_move(10000,"fixture income");
        base.lineage.last_night_net=1000;
        base.lineage.clock_s=3590;base.lineage.rest_left=6000;
        let mut whole=base.clone();let expected=run_offline_quick(&mut whole,601);
        assert!(whole.lineage.gold_ledger.iter().any(|m|m.why=="bank deposit"));
        for reload in [false,true] {
            let(actual,report)=partition(&base,601,&[60],reload);
            equal(&whole,&actual,"worker timestamp across a partially consumed rest");
            assert_eq!(report,expected);
        }
    }
    #[test]
    fn changed_guide_quote_is_saved_with_the_absence_continuation() {
        let mut g=camp(3);
        crate::tree::grant(&mut g.lineage,&["guide"]);
        g.lineage.gold_move(10000,"fixture income");
        g.lineage.best_depth=12;g.lineage.light_waystones(9);
        assert!(run_offline_slice(&mut g,0,false).slice_pending);
        assert_eq!(g.passage,None);
        crate::tree::at_hour(&mut g);
        assert!(g.lineage.start>1);
        let quote=g.passage;assert!(quote.is_some());
        assert!(run_offline_slice(&mut g,0,false).slice_pending);
        assert_eq!(Game::load(&g.save()).unwrap().passage,quote);
    }
    #[test]
    fn live_clock_keeps_partial_milliseconds_and_seconds_after_reload() {
        let mut whole=camp(3);whole.send();let mut interrupted=whole.clone();
        whole.advance(1000);
        interrupted.advance(175);
        assert_eq!(interrupted.advance_rem,(75,1));
        let mut restored=Game::load(&interrupted.save()).unwrap();
        assert_eq!(restored.advance_rem,(75,1));
        interrupted.advance(825);restored.advance(825);
        equal(&whole,&interrupted,"continuous fractional scheduler step");
        equal(&whole,&restored,"saved fractional scheduler step");
    }
}

#[cfg(test)]
mod plateau_tests {
    use super::*;
    #[test]
    fn quick_plateau_is_bounded_and_full_analysis_has_a_separate_cache() {
        static WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);
        crate::forecast::with_sim_width(&WIDTH, || {
            let mut g=Game::new_literal(1);
            let mut rules=g.lineage.rules().clone();
            rules.rows.insert(0,Row::new(vec![Cond::n("hp<",35)],Verb::new("return")));
            g.set_rules(rules).unwrap();g.stall.runs=4;g.stall.depth=5;
            g.stall.exit_rows.insert(0,4);g.stall.absent_rows.insert(0,4);
            let saved=g.save();
            let (quick,qwork)=crate::forecast::measure_work(||stall_verdict_with(&mut g,QUICK_STALL_SIMS).unwrap());
            // Base + deeper threshold + removal + rest (no boss on D5).
            assert!(qwork.simulations>0&&qwork.simulations<=u64::from(QUICK_STALL_SIMS)*4,"{qwork:?}");
            assert_eq!(g.save(),saved,"estimates cannot alter progression");
            assert!(quick.patches.iter().all(|p|p.forecast_delta>STALL_DELTA));
            assert!(quick.patches.windows(2).all(|w|w[0].forecast_delta>=w[1].forecast_delta));
            let (again,work)=crate::forecast::measure_work(||stall_verdict_with(&mut g,QUICK_STALL_SIMS).unwrap());
            assert_eq!(again,quick);assert_eq!(work.simulations,0);
            let (full,fwork)=crate::forecast::measure_work(||stall_verdict(&mut g).unwrap());
            assert!(fwork.simulations>qwork.simulations,"full analysis must not reuse the quick sample: {fwork:?}");
            let mut reference=Game::load(&saved).unwrap();
            assert_eq!(full,stall_verdict(&mut reference).unwrap(),"full analysis remains the fresh full estimate");
            let (quick_after_full,work)=crate::forecast::measure_work(||stall_verdict_with(&mut g,QUICK_STALL_SIMS).unwrap());
            assert_eq!(quick_after_full,quick);
            // One result slot: switching modes recomputes candidates, reusing the base memo.
            assert!(work.simulations<=qwork.simulations);
            assert_eq!(g.save(),saved);
        });
    }
}
