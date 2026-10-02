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
    let budget: u64 = elapsed_s * TICKS_PER_SECOND;
    // Renown of runs watched since the last report settles as its own absence.
    game.settle_renown(0);
    // Cut 30 (PROGRESSION_V2 §4): one new system a report
    game.lineage.reveal_left = 1;
    game.batch = Batch::default();
    game.events.clear();
    game.offline = true;
    // Cut 7 §5: nothing in an absence is watched (a run left mid-watch finishes unwatched).
    game.watched = false;
    // Cut 27 §1: the absence's sends from a waystone are paid the passage the camp priced for the
    // set as it stands (once: the rules do not change in an absence).
    let rules = game.lineage.rules().clone();
    game.passage = crate::forecast::sim_passage(game, &rules);
    let facts_before = game.lineage.facts.clone();
    // Cut 30 §4: what grows over the absence, track by track (`ReturnReport.grew`).
    let grew_before = crate::town::snap(&game.lineage);
    let class = game.lineage.class.name().to_string();
    let rank_before = game.lineage.rank;
    // Cut 29 §1: the lineage's clock — a new day closes the last one's net and lapses the oaths
    // sworn on an earlier day.
    let day0 = (game.lineage.clock_s / crate::engine::DAY_S) as u32;
    crate::oath::new_day(&mut game.lineage, day0);
    let mut consumed: u64 = 0;
    let mut stall = game.stall_runs;
    let mut sampled = false;
    // Cut 26 (seam, control rater AR: `0 RUNS` after a 20-minute break): the rest after a run the
    // player watched was the camp time he spent on its exit; the absence starts rested.
    if game.lineage.rest_watched {
        game.lineage.rest_left = 0;
        game.lineage.rest_watched = false;
    }
    // Cut 30 §1 (short absences pay): on the idle floor the camp time before a new absence is rest —
    // the rest the hero owes carries at most `REST_CARRY_TICKS` in (a 20-minute absence after an 8-hour
    // one read `0 runs`: the last run's 20–30 minutes of rest). A harness's literal lineage waits it out.
    if !game.lineage.pkg.literal {
        game.lineage.rest_left = game.lineage.rest_left.min(REST_CARRY_TICKS);
    }
    while consumed < budget {
        // Camp rest first (the heir at camp: no run, or a run not yet begun).
        if game.lineage.rest_left > 0 && game.run.as_ref().is_none_or(|r| r.turn == 0) {
            // QA on e75ec29 (qaQ: `rested 333m` beside 16 runs × `rest 20m`): the report's
            // `rested` is the rest each of its runs earned (counted at the run's end, below),
            // not the clock's — the camp rest a report begins with is the previous run's, and
            // the last run's is still running at the return.
            let used = game.rest_tick((budget - consumed).min(u32::MAX as u64) as u32);
            consumed += used as u64;
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
        // Cut 26 (seam): a rest that ends inside the absence — to its last tick — sends the next
        // heir, whose run finishes past the budget like any begun run (a 20-minute break after a
        // 20-minute rest yields its run).
        if game.run.as_ref().is_some_and(|r| r.over.is_none() && (r.turn > 0 || consumed >= budget)) {
            let before = game.run.as_ref().unwrap().turn;
            game.run_to_end(crate::engine::MAX_TURNS_PER_RUN);
            game.events.clear();
            consumed += (game.run.as_ref().unwrap().turn - before) as u64;
        }
        if game.run.as_ref().is_some_and(|r| r.over.is_some()) {
            // (the board's day: the day this send came home on, inside the absence)
            game.lineage.town.today = ((game.lineage.clock_s + consumed / TICKS_PER_SECOND) / crate::engine::DAY_S) as u32;
            let outcome = game.finish_run().unwrap_or_default();
            game.batch.rested += game.lineage.rest_left as u64;
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
    game.stall_runs = stall;
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
    let mut r = report_with(game, elapsed_s, &facts_before, &class, rank_before, sampled, full, with_stall);
    r.grew = crate::town::grew(&grew_before, &crate::town::snap(&game.lineage));
    // (a system's reveal is a beat of the five)
    if !r.systems_opened.is_empty() {
        r.packages = crate::packages::beats_n(&game.batch.pkg_lines, crate::packages::BEATS - 1);
    }
    r
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
fn report_with(game: &mut Game, elapsed_s: u64, facts_before: &std::collections::BTreeSet<String>, class: &str, rank_before: u32, sampled: bool, full: bool, with_stall: bool) -> ReturnReport {
    let t = game.run.as_ref().map(|r| r.turn).unwrap_or(0);
    game.settle_renown(t);
    let learned: Vec<String> = game.lineage.facts.difference(facts_before).cloned().collect();
    let worst_death_id = game.batch.worst_death;
    let worst_death = if full { worst_death_id.and_then(|id| crate::trace::death(game, id)) } else { None };
    let pending = crate::meta::pending(game);
    let stall = if with_stall { stall_verdict(game) } else { None };
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
    let mut r = ReturnReport { lead: Vec::new(), oath, grew: Vec::new(), packages: crate::packages::beats(&b.pkg_lines),
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
        meters: (!b.meters.is_empty()).then(|| crate::meters::wire(&b.meters)),
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
        gold: Some(crate::wire::GoldSummary { home: b.gold_earned, salvage: b.salvage_gold, wake: b.wake_pay, spent: b.spent.values().map(|(_, g)| *g).sum(), wake_cap: crate::engine::WAKE_PAY, wake_n: b.wake_n, lost: b.gold_lost, unkept: b.gold_unkept }),
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
fn stall_patches(game: &Game, rules: &RuleSet, row: usize, ending: &Row, depth: u32) -> Vec<Patch> {
    let target = depth + 1;
    let sims = crate::forecast::FORECAST_SIMS;
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
        let r = crate::forecast::reach_paired(game, &apply_patch(rules, p, max_rows), target, n, tag);
        p.survive = r;
        p.forecast_delta = r - base;
    }
    cands.retain(|p| p.forecast_delta > STALL_DELTA);
    cands.sort_by(|a, b| b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap());
    cands.truncate(STALL_SHOWN);
    cands
}
