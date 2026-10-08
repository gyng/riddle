//! `dayplayer --rows <ids or substrings> [--fail-fast]` (docs/ITERATION_SPEED.md §0h): the tuning loop's
//! targeted check. Each bar declares what it reads (`ROWS`: its bots, how far into the fortnight — a
//! milestone depth, a day, the whole fortnight — and so which seeds), and the dayplayer plays only those
//! configurations and seeds, each game stopped once every row reading it has what it needs. The
//! shared-prefix machinery (`Group`, `Ask`, `Play`) is used as it is; a game stopped early is never kept
//! in the job cache (a kept fortnight is read back whole). A row run to completion prints the full run's
//! value exactly (the bar code in `main` computes it, on games that are prefixes of the full run's).
//!
//! `--fail-fast`: seeds play lowest first, and each row's verdict is settled as early as it can be — an
//! every-seed row fails on its first failing seed; a median or count row settles once the seeds left cannot
//! change it (the median of n values with k known and the rest at ±∞, or absent where the bar drops a
//! seed); a row that can no longer fail is passed early and the games only it needed stop. The run aborts on
//! the first row settled FAIL, naming the seed and the values that settled it. At the end every targeted
//! row's settle rule is checked against the bar code (a bar edited in `main` without this table prints a
//! mismatch, and the targeted run fails).
//!
//! This file sits outside the job cache's key (`dayplayer.rs` less `main`): it decides which games run and
//! where they stop, never what a game does.
use super::{hours_or, median, Bot, Cfg, Group, Play, Pool, SeedOut, MILESTONES, SYSTEMS};
use std::collections::HashMap;
use std::sync::Mutex;

/// Where a configuration's game may stop for a row (besides the day limit).
#[derive(Clone, Copy, Debug)]
pub enum Until {
    /// only at the row's day limit
    Never,
    /// the record reached this depth (its milestone hours are written at that check-in)
    Depth(u32),
    /// the day ended with the record at D23 or deeper (the stall row stops counting there)
    Reached23,
    /// the day ended with the worn stance at this level
    StanceDayEnd(u32),
}

pub enum Verdict {
    Open,
    Pass(String),
    Fail(String),
}

type Settle = fn(&Ctx) -> Verdict;

pub struct RowDef {
    pub id: &'static str,
    /// a phrase of the bar's name in `main` (the row's printed line is the bar whose name holds it)
    pub key: &'static str,
    /// the configurations the row reads ("LOO": TUNED's six leave-one-outs)
    pub bots: &'static [&'static str],
    /// read only for being there (`main` computes the row only when it has a result — the PICKED-stages row
    /// sits under `!picked.is_empty() && !idle.is_empty()`): seed 1, no days
    pub present: &'static [&'static str],
    /// the days read (`None`: the whole fortnight) and the earlier stop
    pub days: Option<usize>,
    pub until: Until,
    pub settle: Settle,
}

/// Every bar of `main`, what it reads and how its verdict settles early. Keep in step with the bars: a row
/// whose bar is renamed matches nothing (the run says so); a bar whose rule changes without its settle rule
/// prints a mismatch at the end of every targeted run.
pub const ROWS: &[RowDef] = &[
    RowDef { id: "idle-d8", key: "IDLE reaches D8 by the end of day 1", bots: &["IDLE"], present: &[], days: Some(1), until: Until::Depth(8), settle: |c| c.every(&["IDLE"], |o| if o[0].hours[0].is_some_and(|h| h <= 24.0) { Ok(()) } else { Err(format!("D8 at {:?} h", o[0].hours[0])) }) },
    RowDef { id: "idle-d13", key: "IDLE reaches D13 by day 4", bots: &["IDLE"], present: &[], days: None, until: Until::Depth(13), settle: |c| c.median(&["IDLE"], "median day", false, |o, c| Some(hours_or(o[0], 1, c.cap) / 24.0), |m| m <= 4.0) },
    RowDef {
        id: "idle-d23",
        key: "IDLE reaches D23 by day 12",
        bots: &["IDLE"],
        present: &[],
        days: None,
        until: Until::Depth(23),
        settle: |c| and(vec![c.count(&["IDLE"], "seeds by day 12", |o, c| hours_or(o[0], 3, c.cap) / 24.0 <= 12.0, |k, n| k * 8 >= 6 * n), c.median(&["IDLE"], "median day", false, |o, c| Some(hours_or(o[0], 3, c.cap) / 24.0), |m| m <= 12.0)]),
    },
    RowDef { id: "idle-stall", key: "IDLE longest best-depth stall", bots: &["IDLE"], present: &[], days: None, until: Until::Reached23, settle: |c| c.every(&["IDLE"], |o| if o[0].stall <= 4 { Ok(()) } else { Err(format!("stall {} d", o[0].stall)) }) },
    RowDef {
        id: "idle-gold",
        key: "IDLE net gold > 0 every day",
        bots: &["IDLE"],
        present: &[],
        days: None,
        until: Until::Never,
        settle: |c| {
            let days = c.days;
            c.every(&["IDLE"], move |o| {
                let n = o[0].gold_day.iter().filter(|g| **g > 0).count();
                if n == days {
                    Ok(())
                } else {
                    Err(format!("{n}/{days} days"))
                }
            })
        },
    },
    RowDef { id: "idle-king", key: "IDLE does not slay the Mirror King", bots: &["IDLE"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["IDLE"], |o| if o[0].king_day.is_none() { Ok(()) } else { Err(format!("king on day {:?}", o[0].king_day)) }) },
    RowDef {
        id: "stance",
        key: "Stance L3 by day 2, L5 by day 7",
        bots: &["IDLE"],
        present: &[],
        days: None,
        until: Until::StanceDayEnd(5),
        settle: |c| {
            let at = |o: &SeedOut, l: u32| o.stance_level_day.iter().position(|x| *x >= l).map_or(99.0, |d| d as f64 + 1.0);
            and(vec![c.median(&["IDLE"], "L3 median day", false, move |o, _| Some(at(o[0], 3)), |m| m <= 2.0), c.median(&["IDLE"], "L5 median day", false, move |o, _| Some(at(o[0], 5)), |m| m <= 7.0)])
        },
    },
    RowDef { id: "return-pick", key: "Every IDLE check-in offers a return pick", bots: &["IDLE"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["IDLE"], |o| if o[0].picks == o[0].checkins { Ok(()) } else { Err(format!("{}/{} check-ins offered", o[0].picks, o[0].checkins)) }) },
    RowDef { id: "grew", key: "Every IDLE check-in grows", bots: &["IDLE"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["IDLE"], |o| if o[0].grew == o[0].checkins { Ok(()) } else { Err(format!("{}/{} check-ins grew", o[0].grew, o[0].checkins)) }) },
    RowDef { id: "idle-stages", key: "Days with a stage opened: IDLE", bots: &["IDLE"], present: &[], days: None, until: Until::Never, settle: |c| c.median(&["IDLE"], "median days", false, |o, _| Some(o[0].stage_days as f64), |m| m >= 8.0) },
    RowDef {
        id: "picked-idle",
        key: "PICKED ≥ 1.5× IDLE at D13, D18, D23",
        bots: &["IDLE", "PICKED"],
        present: &[],
        days: None,
        until: Until::Depth(23),
        settle: |c| and([1, 2, 3].iter().map(|&i| c.median(&["IDLE", "PICKED"], ["", "D13", "D18", "D23"][i], true, move |o, _| seed_ratio(o[0], o[1], i), |r| r >= 1.5)).collect()),
    },
    RowDef { id: "outpace", key: "IDLE never out-paces PICKED", bots: &["IDLE", "PICKED"], present: &[], days: None, until: Until::Depth(33), settle: outpace },
    RowDef { id: "picked-stages", key: "Days with a stage opened: PICKED", bots: &["PICKED"], present: &["IDLE"], days: None, until: Until::Never, settle: |c| c.median(&["PICKED"], "median days", false, |o, _| Some(o[0].stage_days as f64), |m| m >= 10.0) },
    RowDef { id: "reach", key: "Content reach: PICKED ≥ D14 by day 2", bots: &["PICKED"], present: &[], days: Some(2), until: Until::Never, settle: |c| c.count(&["PICKED"], "seeds ≥ D14 on day 2", |o, _| o[0].best_day.get(1).is_some_and(|b| *b >= 14), |k, n| k * 2 >= n) },
    RowDef { id: "stalls-idle", key: "Stalls ≤ 1 % of sends, every IDLE seed", bots: &["IDLE"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["IDLE"], stalls) },
    RowDef { id: "stalls-picked", key: "Stalls ≤ 1 % of sends, every PICKED seed", bots: &["PICKED"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["PICKED"], stalls) },
    RowDef { id: "stalls-tuned", key: "Stalls ≤ 1 % of sends, every TUNED seed", bots: &["TUNED"], present: &[], days: None, until: Until::Never, settle: |c| c.every(&["TUNED"], stalls) },
    RowDef { id: "tuned-picked", key: "TUNED beats PICKED by ≥ 15 % at D33", bots: &["PICKED", "TUNED"], present: &[], days: None, until: Until::Depth(33), settle: |c| c.median(&["PICKED", "TUNED"], "D33", true, |o, _| seed_ratio(o[0], o[1], 6), |r| r >= 1.15) },
    // (Cut 30.5, the owner: the median seed and ≥ 14/16 — settled at the end, by the bar)
    RowDef { id: "random-picked", key: "RANDOM slower than PICKED at D13, D23", bots: &["RANDOM", "PICKED"], present: &[], days: None, until: Until::Depth(23), settle: |_| Verdict::Open },
    RowDef {
        id: "nothing-required",
        key: "Nothing required: TUNED − S",
        bots: &["LOO", "IDLE"],
        present: &["TUNED"],
        days: None,
        until: Until::Depth(33),
        settle: |c| {
            // each system × milestone: the median lag behind IDLE ≤ a check-in (seeds where either reached it)
            // and no seed more than `NOTHING_REQ_MAX_LAG` behind from the pen's opening
            let (cap, step) = (c.cap, (24 / c.checkins) as f64);
            let lag = move |o: &[&SeedOut], k: usize, i: usize| {
                let (x, y) = (o[k], o[SYSTEMS.len()]);
                (x.hours[i].is_some() || y.hours[i].is_some()).then(|| hours_or(x, i, cap) - hours_or(y, i, cap))
            };
            // (the worst seed's lag counts from the pen's opening: each side's hours clamped up to `PEN_DAY_H`;
            // the cap and the day read from `main`'s own constants, so the row follows the bar)
            let (max_lag, pen_h) = (main_const("NOTHING_REQ_MAX_LAG"), main_const("PEN_DAY_H"));
            let mut vs = vec![c.every(&["LOO", "IDLE"], move |o| {
                for (k, s) in SYSTEMS.iter().enumerate() {
                    let (x, y) = (o[k], o[SYSTEMS.len()]);
                    for (i, m) in MILESTONES.iter().enumerate() {
                        if x.hours[i].is_none() && y.hours[i].is_none() {
                            continue;
                        }
                        let l = hours_or(x, i, cap).max(pen_h) - hours_or(y, i, cap).max(pen_h);
                        if l > max_lag {
                            return Err(format!("{s} D{m}: {l:+.0} h behind IDLE from day 5 (cap {max_lag} h)"));
                        }
                    }
                }
                Ok(())
            })];
            for (k, s) in SYSTEMS.iter().enumerate() {
                for (i, m) in MILESTONES.iter().enumerate() {
                    vs.push(c.median(&["LOO", "IDLE"], &format!("{s} D{m} median lag"), true, move |o, _| lag(o, k, i), move |l| l.is_nan() || l <= step));
                    // (a system × milestone no seed reaches is skipped by the bar: an empty median, NaN, passes)
                }
            }
            and(vs)
        },
    },
    // Cut 30.5: the works tree's rows (settled at the end, by the bar)
    RowDef { id: "porter", key: "Porter bought ≤ min 12", bots: &["PICKED"], present: &[], days: Some(1), until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "scout", key: "Scout bought ≤ min 15", bots: &["IDLE", "PICKED", "HANDS", "RANDOM"], present: &[], days: Some(1), until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "nodes-lit", key: "Every node whose chore exists lit by 48 h", bots: &["PICKED", "HANDS"], present: &[], days: Some(2), until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "nodes-bought", key: "Every node whose chore exists bought by 72 h", bots: &["PICKED"], present: &[], days: Some(3), until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "conserved", key: "Gold conserved", bots: &["IDLE", "PICKED", "HANDS"], present: &[], days: None, until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "hands-idle", key: "HANDS never slower than IDLE", bots: &["HANDS", "IDLE"], present: &[], days: None, until: Until::Depth(23), settle: |_| Verdict::Open },
    RowDef { id: "automation-pays", key: "Workers pay the away player", bots: &["AWAY", "AWAY-nodes"], present: &[], days: None, until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "workers-daily", key: "Workers never cost the daily player", bots: &["DAILY", "DAILY-nodes"], present: &[], days: None, until: Until::Never, settle: |_| Verdict::Open },
    RowDef { id: "nodes-required", key: "Nothing required, S = nodes", bots: &["PICKED-nodes", "IDLE"], present: &["PICKED"], days: None, until: Until::Depth(33), settle: |_| Verdict::Open },
    RowDef { id: "idle-delta", key: "IDLE within a bounded delta", bots: &["IDLE", "IDLE30"], present: &[], days: None, until: Until::Depth(13), settle: |_| Verdict::Open },
    // (means over the seeds: settled only at the end, by the bar itself)
    RowDef { id: "each-system", key: "Each system adds value by its own output", bots: &["TUNED", "LOO"], present: &["IDLE"], days: None, until: Until::Never, settle: |_| Verdict::Open },
];

fn stalls(o: &[&SeedOut]) -> Result<(), String> {
    let r = o[0].stalled as f64 / o[0].sends.max(1) as f64;
    if r <= 0.01 {
        Ok(())
    } else {
        Err(format!("{:.2}% ({}/{})", 100.0 * r, o[0].stalled, o[0].sends))
    }
}

/// One seed's slower ÷ faster at milestone `i` as `ratio` in `main` reads it (`None`: the seed drops out).
fn seed_ratio(slow: &SeedOut, fast: &SeedOut, i: usize) -> Option<f64> {
    match (slow.hours[i], fast.hours[i]) {
        (_, None) => None,
        (None, Some(_)) => Some(f64::INFINITY),
        (Some(a), Some(b)) => Some(a / b.max(1.0)),
    }
}

/// IDLE never out-paces PICKED: PICKED's hours ≤ IDLE's on ≥ 90 % of the seed × milestone pairs either
/// reached. A seed adds 0–7 pairs; the seeds left at worst add 7 lost pairs each, at best 7 kept.
fn outpace(c: &Ctx) -> Verdict {
    let (known, left) = c.known(&["IDLE", "PICKED"]);
    let (mut ok, mut pairs) = (0usize, 0usize);
    for (_, o) in &known {
        let (s, p) = (o[0], o[1]);
        for (i, _) in MILESTONES.iter().enumerate() {
            if p.hours[i].is_none() && s.hours[i].is_none() {
                continue;
            }
            pairs += 1;
            ok += (hours_or(p, i, c.cap) <= hours_or(s, i, c.cap)) as usize;
        }
    }
    let pct = |ok: usize, pairs: usize| 100.0 * ok as f64 / pairs.max(1) as f64;
    let m = MILESTONES.len() * left;
    let (lo, hi) = (pct(ok, pairs + m), pct(ok + m, pairs + m));
    let say = format!("{lo:.0}–{hi:.0}% ({ok}/{pairs} pairs, {left} seeds left)");
    settle_by(lo >= 90.0, hi >= 90.0, say)
}

/// Both extremes pass: passed; both fail: failed (the bar is a threshold, so every value between them does
/// the same).
fn settle_by(worst: bool, best: bool, say: String) -> Verdict {
    match (worst, best) {
        (true, true) => Verdict::Pass(say),
        (false, false) => Verdict::Fail(say),
        _ => Verdict::Open,
    }
}

fn and(vs: Vec<Verdict>) -> Verdict {
    let mut says = Vec::new();
    let mut open = false;
    for v in &vs {
        if let Verdict::Fail(s) = v {
            return Verdict::Fail(s.clone());
        }
    }
    for v in vs {
        match v {
            Verdict::Pass(s) => says.push(s),
            _ => open = true,
        }
    }
    if open {
        Verdict::Open
    } else if says.len() > 3 {
        Verdict::Pass(format!("{} clauses pass ({} …)", says.len(), says[..2].join(" · ")))
    } else {
        Verdict::Pass(says.join(" · "))
    }
}

/// The median of `vals` with `lo` more values at −∞ (a proxy below every real one) or `hi` at +∞.
fn med_with(vals: &[f64], extra: usize, x: f64) -> f64 {
    let mut v = vals.to_vec();
    v.extend(std::iter::repeat_n(x, extra));
    median(v)
}

/// What a row's settle rule reads: the results so far of the configurations the row reads, by seed.
pub struct Ctx<'a> {
    pub got: HashMap<(String, u64), &'a SeedOut>,
    /// the row's seeds (1..=n)
    pub n: u64,
    pub days: usize,
    pub checkins: u64,
    pub cap: f64,
}

fn expand(labels: &[&str]) -> Vec<String> {
    labels.iter().flat_map(|l| if *l == "LOO" { SYSTEMS.iter().map(|s| format!("TUNED-{s}")).collect() } else { vec![l.to_string()] }).collect()
}

impl<'a> Ctx<'a> {
    /// The seeds with every configuration's result (in seed order, the results in `labels`' order) and the
    /// number of seeds left.
    fn known(&self, labels: &[&str]) -> (Vec<(u64, Vec<&'a SeedOut>)>, usize) {
        let ls = expand(labels);
        let mut known = Vec::new();
        for s in 1..=self.n {
            let os: Option<Vec<&SeedOut>> = ls.iter().map(|l| self.got.get(&(l.clone(), s)).copied()).collect();
            if let Some(os) = os {
                known.push((s, os));
            }
        }
        let left = self.n as usize - known.len();
        (known, left)
    }

    fn every(&self, labels: &[&str], f: impl Fn(&[&SeedOut]) -> Result<(), String>) -> Verdict {
        let (known, left) = self.known(labels);
        for (s, o) in &known {
            if let Err(e) = f(o) {
                return Verdict::Fail(format!("seed {s}: {e}"));
            }
        }
        if left == 0 {
            Verdict::Pass(format!("{}/{} seeds", known.len(), self.n))
        } else {
            Verdict::Open
        }
    }

    fn count(&self, labels: &[&str], what: &str, f: impl Fn(&[&SeedOut], &Ctx) -> bool, pass: impl Fn(usize, usize) -> bool) -> Verdict {
        let (known, left) = self.known(labels);
        let k = known.iter().filter(|(_, o)| f(o, self)).count();
        let n = self.n as usize;
        let fails: Vec<String> = known.iter().filter(|(_, o)| !f(o, self)).map(|(s, _)| format!("s{s}")).collect();
        settle_by(pass(k, n), pass(k + left, n), format!("{what} {k}–{}/{n} (not: {})", k + left, fails.join(" ")))
    }

    /// The median over seeds of `f` (`None`: the seed drops out, as `ratio` in `main` drops a seed the faster
    /// bot never reaches the milestone on — `may_drop`: a seed left may drop out too).
    fn median(&self, labels: &[&str], what: &str, may_drop: bool, f: impl Fn(&[&SeedOut], &Ctx) -> Option<f64>, pass: impl Fn(f64) -> bool) -> Verdict {
        let (known, left) = self.known(labels);
        let vals: Vec<f64> = known.iter().filter_map(|(_, o)| f(o, self)).collect();
        let mut meds = Vec::new();
        for dropped in 0..=if may_drop { left } else { 0 } {
            let here = left - dropped;
            meds.push(med_with(&vals, here, -1e300));
            meds.push(med_with(&vals, here, f64::INFINITY));
        }
        let all = meds.iter().all(|m| pass(*m));
        let none = meds.iter().all(|m| !pass(*m));
        let lo = meds.iter().copied().filter(|m| !m.is_nan()).fold(f64::INFINITY, f64::min);
        let hi = meds.iter().copied().filter(|m| !m.is_nan()).fold(f64::NEG_INFINITY, f64::max);
        let shown: Vec<String> = known.iter().map(|(s, o)| format!("s{s} {}", f(o, self).map_or("–".into(), |v| format!("{v:.2}")))).collect();
        let range = if left == 0 { format!("{lo:.2}") } else { format!("{}…{} with {left} seeds left", if lo < -1e299 { "−∞".into() } else { format!("{lo:.2}") }, if hi.is_infinite() { "∞".into() } else { format!("{hi:.2}") }) };
        // (the seeds' values only when it fails: a pass is one line)
        match settle_by(all, !none, String::new()) {
            Verdict::Pass(_) => Verdict::Pass(format!("{what} {range}")),
            Verdict::Fail(_) => Verdict::Fail(format!("{what} {range} ({})", shown.join(", "))),
            v => v,
        }
    }
}

/// A configuration's needs at a seed: the row, the days it reads and its earlier stop.
#[derive(Clone, Copy)]
struct Need {
    row: usize,
    days: usize,
    until: Until,
}

pub struct Plan {
    pub rows: Vec<usize>,
    pub fail_fast: bool,
    /// the seeds of each row (1..=n)
    n_of: Vec<u64>,
    needs: HashMap<(String, u64), Vec<Need>>,
    /// each targeted row's verdict once settled (fail-fast)
    settled: Mutex<HashMap<usize, Verdict>>,
    days: usize,
    checkins: u64,
}

fn label(c: &Cfg) -> String {
    c.label()
}

impl Plan {
    /// The rows named by `arg` (ids, or substrings of the bars' names, comma-separated; the whole argument is
    /// tried as one substring first, since a bar's name may hold a comma), and the configurations they read
    /// in the full run's order (IDLE, PICKED, TUNED, RANDOM, then the leave-one-outs).
    #[allow(clippy::too_many_arguments)]
    pub fn new(arg: &str, fail_fast: bool, cfgs: &mut Vec<Cfg>, seeds: u64, tuned_seeds: u64, loo_seeds: u64, days: usize, checkins: u64) -> Plan {
        let hits = |t: &str| -> Vec<usize> {
            let t = t.trim().to_lowercase();
            if t.is_empty() {
                return Vec::new();
            }
            let by_id: Vec<usize> = (0..ROWS.len()).filter(|i| ROWS[*i].id == t).collect();
            if !by_id.is_empty() {
                return by_id;
            }
            (0..ROWS.len()).filter(|i| ROWS[*i].key.to_lowercase().contains(&t) || ROWS[*i].id.contains(&t)).collect()
        };
        let mut rows = hits(arg);
        if rows.is_empty() {
            for t in arg.split(',') {
                let h = hits(t);
                if h.is_empty() {
                    eprintln!("dayplayer --rows: `{}` names no row; the rows:\n{}", t.trim(), ROWS.iter().map(|r| format!("  {:<18} {}", r.id, r.key)).collect::<Vec<_>>().join("\n"));
                    std::process::exit(2);
                }
                rows.extend(h);
            }
        }
        rows.sort();
        rows.dedup();
        let all: Vec<Cfg> = [Bot::Idle, Bot::Picked, Bot::Tuned, Bot::Random, Bot::Hands, Bot::Idle30, Bot::Daily, Bot::Away].into_iter().map(|bot| Cfg { bot, without: None }).chain([Cfg { bot: Bot::Picked, without: Some("nodes") }, Cfg { bot: Bot::Daily, without: Some("nodes") }, Cfg { bot: Bot::Away, without: Some("nodes") }]).chain(SYSTEMS.iter().map(|s| Cfg { bot: Bot::Tuned, without: Some(s) })).collect();
        let seeds_of = |l: &str| -> u64 {
            if l.starts_with("AWAY") || l.starts_with("DAILY") {
                seeds
            } else if l.starts_with("TUNED-") || l == "PICKED-nodes" {
                loo_seeds.min(seeds)
            } else if l == "TUNED" {
                tuned_seeds.min(seeds)
            } else {
                seeds
            }
        };
        let mut needs: HashMap<(String, u64), Vec<Need>> = HashMap::new();
        let mut n_of = vec![0; ROWS.len()];
        for &r in &rows {
            let d = &ROWS[r];
            let ls = expand(d.bots);
            let n = ls.iter().map(|l| seeds_of(l)).min().unwrap_or(0);
            n_of[r] = n;
            for l in &ls {
                for s in 1..=n {
                    needs.entry((l.clone(), s)).or_default().push(Need { row: r, days: d.days.unwrap_or(days).min(days), until: d.until });
                }
            }
            for l in expand(d.present) {
                if seeds_of(&l) >= 1 {
                    needs.entry((l, 1)).or_default().push(Need { row: r, days: 0, until: Until::Never });
                }
            }
        }
        *cfgs = all.into_iter().filter(|c| needs.keys().any(|(l, _)| *l == label(c))).collect();
        Plan { rows, fail_fast, n_of, needs, settled: Mutex::new(HashMap::new()), days, checkins }
    }

    pub fn say(&self) -> String {
        self.rows.iter().map(|r| ROWS[*r].id).collect::<Vec<_>>().join(",")
    }

    /// Cross-runtime scheduling hints only. They never supply a game or verdict.
    pub fn priority_seeds(&self) -> Vec<u64> {
        let mut seeds = Vec::new();
        if !self.fail_fast || std::env::var_os("RIDDLE_SEED_ORDER_NUMERIC").is_some() { return seeds; }
        for &r in &self.rows {
            let path = format!("target/gates/failure-hints/{}.json", ROWS[r].id);
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(s) = serde_json::from_str::<u64>(&text) {
                    if (1..=1_000_000).contains(&s) && !seeds.contains(&s) { seeds.push(s); }
                }
            }
        }
        seeds
    }

    /// Whether configuration `c` plays seed `s` at all.
    pub fn wants(&self, c: &Cfg, s: u64) -> bool {
        self.needs.contains_key(&(label(c), s))
    }

    /// Whether every member has what each (unsettled) row reading it needs: the game may stop.
    pub fn enough(&self, p: &Play, members: &[usize], cfgs: &[Cfg]) -> bool {
        let settled = self.settled.lock().unwrap();
        members.iter().all(|m| {
            self.needs.get(&(label(&cfgs[*m]), p.seed)).is_none_or(|ns| {
                ns.iter().filter(|n| !settled.contains_key(&n.row)).all(|n| {
                    p.day >= n.days
                        || match n.until {
                            Until::Never => false,
                            Until::Depth(d) => p.g.lineage.best_depth >= d,
                            Until::Reached23 => p.reached23,
                            Until::StanceDayEnd(l) => p.ci == 0 && p.out.stance_level_day.last().is_some_and(|x| *x >= l),
                        }
                })
            })
        })
    }

    fn ctx<'a>(&self, r: usize, res: &'a [(usize, SeedOut)], cfgs: &[Cfg], cap: f64) -> Ctx<'a> {
        let got = res.iter().map(|(c, o)| ((label(&cfgs[*c]), o.seed), o)).collect();
        Ctx { got, n: self.n_of[r], days: self.days, checkins: self.checkins, cap }
    }

    /// The settle rules on the results so far (fail-fast; called as each result lands): a row settled FAIL
    /// aborts the run, a row settled PASS frees the games only it needed, every row settled ends the run.
    pub fn check(&self, res: &[(usize, SeedOut)], cfgs: &[Cfg], cap: f64, last: &str, t0: std::time::Instant) {
        if !self.fail_fast {
            return;
        }
        let mut settled = self.settled.lock().unwrap();
        for &r in &self.rows {
            if settled.contains_key(&r) {
                continue;
            }
            match (ROWS[r].settle)(&self.ctx(r, res, cfgs, cap)) {
                Verdict::Open => {}
                Verdict::Pass(s) => {
                    eprintln!("dayplayer --fail-fast: {} settled PASS after {last} ({:.0}s): {s}", ROWS[r].id, t0.elapsed().as_secs_f64());
                    settled.insert(r, Verdict::Pass(s));
                }
                Verdict::Fail(s) => {
                    // Remember the completing seed, independently of acceptance caches.
                    if let Some(seed) = last.split_whitespace().find_map(|t| t.strip_prefix('s').and_then(|s| s.parse::<u64>().ok())) {
                        let dir = std::path::Path::new("target/gates/failure-hints");
                        let _ = std::fs::create_dir_all(dir);
                        let path = dir.join(format!("{}.json", ROWS[r].id));
                        let tmp = path.with_extension(format!("tmp{}", std::process::id()));
                        if std::fs::write(&tmp, seed.to_string()).is_ok() { let _ = std::fs::rename(tmp, path); }
                    }
                    println!("\nfail-fast: `{}` settled FAIL after {last} ({:.0}s)\n  {}\n  {s}", ROWS[r].id, t0.elapsed().as_secs_f64(), ROWS[r].key);
                    settled.insert(r, Verdict::Fail(s));
                    self.print_settled(&settled, t0);
                    std::process::exit(1);
                }
            }
        }
        if self.rows.iter().all(|r| settled.contains_key(r)) {
            println!("\nfail-fast: every targeted row settled after {last}");
            self.print_settled(&settled, t0);
            std::process::exit(0);
        }
    }

    fn print_settled(&self, settled: &HashMap<usize, Verdict>, t0: std::time::Instant) {
        println!("\n{:<64} {:>18}  result", "bar (targeted, fail-fast)", "value");
        let mut fail = false;
        for &r in &self.rows {
            let (v, say) = match settled.get(&r) {
                Some(Verdict::Pass(s)) => ("PASS", s.as_str()),
                Some(Verdict::Fail(s)) => {
                    fail = true;
                    ("FAIL", s.as_str())
                }
                _ => ("open", ""),
            };
            println!("{:<64} {:>18}  {v}\n    {say}", ROWS[r].key, "settled");
        }
        println!("dayplayer (targeted: {}, fail-fast, {:.0}s): {} — not a gate pass", self.say(), t0.elapsed().as_secs_f64(), if fail { "FAIL" } else { "targeted rows PASS" });
    }

    /// The end of a targeted run: the bars of the targeted rows (rows settled early print their settled
    /// verdict), each checked against its settle rule on the whole results. Returns the lines' failures.
    pub fn finish(&self, bars: &[(String, String, bool)], res: &[(usize, SeedOut)], cfgs: &[Cfg], cap: f64, t0: std::time::Instant) -> usize {
        let settled = self.settled.lock().unwrap();
        println!();
        println!("{:<64} {:>18}  result", "bar (targeted)", "value");
        let mut fails = 0;
        for &r in &self.rows {
            if let Some(v) = settled.get(&r) {
                let (ok, say) = match v {
                    Verdict::Pass(s) => (true, s.clone()),
                    Verdict::Fail(s) => (false, s.clone()),
                    Verdict::Open => (true, String::new()),
                };
                fails += !ok as usize;
                println!("{:<64} {:>18}  {}\n    settled early: {say}", ROWS[r].key, "settled", if ok { "PASS" } else { "FAIL" });
                continue;
            }
            let mine: Vec<&(String, String, bool)> = bars.iter().filter(|b| b.0.contains(ROWS[r].key)).collect();
            if mine.is_empty() {
                println!("{:<64} {:>18}  MISSING (no bar holds `{}`: renamed? update dayplayer_rows ROWS)", ROWS[r].id, "-", ROWS[r].key);
                fails += 1;
                continue;
            }
            for (name, value, ok) in mine {
                println!("{:<64} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
                fails += !ok as usize;
                // the settle rule on the whole results must give the bar's verdict
                let agree = match (ROWS[r].settle)(&self.ctx(r, res, cfgs, cap)) {
                    Verdict::Open => true,
                    Verdict::Pass(_) => *ok,
                    Verdict::Fail(_) => !*ok,
                };
                if !agree {
                    println!("    MISMATCH: the settle rule for `{}` disagrees with the bar (update dayplayer_rows ROWS)", ROWS[r].id);
                    fails += 1;
                }
            }
        }
        println!("dayplayer (targeted: {}{}, {:.0}s): {} — not a gate pass", self.say(), if self.fail_fast { ", fail-fast" } else { "" }, t0.elapsed().as_secs_f64(), if fails > 0 { "FAIL" } else { "targeted rows PASS" });
        fails
    }
}

/// The pool's push with the lower seeds first (fail-fast: a seed's verdicts land as early as they can),
/// then the pool's own order within a seed. Scheduling only: the same games, the same results.
pub fn push_seed_first(pool: &Pool, g: Group, cfgs: &[Cfg], days: usize, checkins: u64, priority: &[u64]) {
    let weight = g.members.iter().map(|c| match cfgs[*c].bot {
        Bot::Tuned => 3,
        Bot::Picked => 2,
        _ => 1,
    });
    let left = g.state.as_ref().map_or(days as u64 * checkins, |p| (days - p.day.min(days)) as u64 * checkins - p.ci);
    let rank = priority.iter().position(|s| *s == g.seed).map_or(priority.len() as u64 + g.seed, |i| i as u64);
    let prio = ((1u64 << 20) - rank.min((1 << 20) - 1)) << 40 | (weight.max().unwrap_or(1) * left).min((1 << 40) - 1);
    let mut q = pool.q.lock().unwrap();
    q.1 += 1;
    q.2 += 1;
    let seq = q.1;
    q.0.push((prio, u64::MAX - seq, g));
    pool.cv.notify_one();
}

/// A `const NAME: f64 = v;` of `dayplayer.rs` (a bar's threshold declared in `main`), read from the source
/// the binary was built from — a row's settle rule follows the bar when the bar's constant moves.
fn main_const(name: &str) -> f64 {
    let src = include_str!("../dayplayer.rs");
    let pat = format!("const {name}: f64 = ");
    src.find(&pat).and_then(|i| src[i + pat.len()..].split(';').next()).and_then(|v| v.trim().parse().ok()).unwrap_or_else(|| panic!("dayplayer_rows: `{pat}…;` not found in dayplayer.rs (update the row table)"))
}
