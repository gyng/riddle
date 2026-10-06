//! Forecast: N deterministic sims on fresh floor seeds, reported only as deep as the hero has facts.
use crate::engine::{ExitTier, Game};
use crate::rng::splitmix;
use crate::rules::RuleSet;
use crate::wire::{Forecast, ForecastCause, ForecastDepth, ForecastEnds, ForecastTry};
use std::collections::BTreeMap;

pub const FORECAST_SIMS: u32 = 50;

/// Optional single-thread diagnostic accounting; not game state or saved data.
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct SimulationWork {
    pub simulations: u64,
    pub ticks: u64,
    pub prefix_simulations: u64,
    pub panel_hits: u64,
    pub panel_misses: u64,
    pub evicted_panels: u64,
}
thread_local! {
    static WORK: std::cell::RefCell<Option<SimulationWork>> = const { std::cell::RefCell::new(None) };
}
fn work(f: impl FnOnce(&mut SimulationWork)) {
    WORK.with(|w| { if let Some(w) = w.borrow_mut().as_mut() { f(w); } });
}
/// Counts this thread only: disable parallel sims for complete diagnostic jobs.
pub fn measure_work<R>(f: impl FnOnce() -> R) -> (R, SimulationWork) {
    struct Restore(Option<SimulationWork>);
    impl Drop for Restore { fn drop(&mut self) { WORK.with(|w| *w.borrow_mut() = self.0.take()); } }
    let _restore = Restore(WORK.with(|w| w.replace(Some(SimulationWork::default()))));
    let r = f();
    let counts = WORK.with(|w| w.borrow_mut().take().unwrap());
    (r, counts)
}
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

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SimResult {
    pub max_depth: u32,
    pub tier: ExitTier,
    pub cause: Option<String>,
    /// Cut 12 §3: the gold this send brings home — the loot by the exit's share (`ExitTier::pct`;
    /// a run that timed out keeps nothing, as the exit's own maths has it). Cut 27 §1: and the
    /// passage its waystone start was paid.
    pub loot_kept: i32,
    /// The send hit the turn cap or stalled: a return by nothing in the rules (its own share).
    pub timed_out: bool,
    /// Cut 24 §4: the ticks the sim ran (its share of the panel's budget) — the refine pass
    /// continues the first pass's panel from its last sim (`camp_panel`).
    pub ticks: u32,
    /// Cut 27 §1: the loot carried when the sim stopped (coins) — at `stop_depth`, the gold the
    /// floors above it brought (`passage_for`).
    pub loot: i32,
    /// Cut 27 §2: how many times each row of the sim's set fired — (`row_key` of the row, fires),
    /// the rows that fired, by key (a row that never fired is absent: a dead row reads the panel
    /// of the set without it, `played_key`).
    pub fires: Vec<(u64, u32)>,
    /// Cut 28 §1: the lineage's sworn oath was kept by this send (false with none sworn), and how
    /// close it came (`oath::progress`, 0..1).
    pub oath: bool,
    pub oath_progress: f64,
    /// Cut 28b (AW: "the Mother oath sat at 9% ±8 with no lever I could find"): the oath's steps
    /// this send passed (`oath::steps` bits: the floor reached, the boss met, the boss burned).
    pub oath_steps: u8,
    /// Cut 29 §6: the waystone passage paid at the send (in `loot_kept`).
    pub passage: i32,
    /// Each new deepest floor as the sim reached it — (max depth, ticks, loot carried) after the
    /// tick that took it there (the start floor at tick 0): the sim stopped at any shallower
    /// `stop_depth` is this one cut there (`cut_at`), so a passage priced from D1 to a floor reads
    /// a deeper run's sims instead of running them again (`passage_from`).
    pub arrive: Vec<(u32, u32, i32)>,
}

impl SimResult {
    /// This sim as the same sim stopped on arriving at `stop` would have ended — the fields a
    /// passage reads (`max_depth`, `loot`, `ticks`); a sim that ended above `stop` is itself.
    fn cut_at(&self, stop: u32) -> SimResult {
        match self.arrive.iter().find(|a| a.0 >= stop) {
            Some(&(depth, ticks, loot)) if self.max_depth >= stop => SimResult { max_depth: depth, tier: ExitTier::Return, cause: None, loot_kept: 0, timed_out: false, ticks, loot, fires: Vec::new(), oath: false, oath_progress: 0.0, oath_steps: 0, passage: 0, arrive: Vec::new() },
            _ => self.clone(),
        }
    }
}

/// Simulate `sims` fresh expeditions from the current lineage with `rules`, each stopping once
/// it reaches `stop_depth` (the depth the caller asks about) or ends.
pub fn simulate(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32) -> Vec<SimResult> {
    let budget = if sims >= FORECAST_SIMS { FORECAST_TICK_BUDGET } else { DELTA_TICK_BUDGET };
    simulate_budget(game, rules, sims, tag, stop_depth, budget)
}

pub fn simulate_budget(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32, budget: u64) -> Vec<SimResult> {
    simulate_budget_from(game, rules, sims, tag, stop_depth, budget, Vec::new())
}

/// `simulate_budget` continuing from `prefix` — the panel's first sims already run (a pure
/// function of their index, so a shorter panel on the same seeds is this one's prefix: Cut 24
/// §4, the refine pass reuses the first pass's sims instead of running them again).
pub fn simulate_budget_from(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32, budget: u64, prefix: Vec<SimResult>) -> Vec<SimResult> {
    simulate_budget_from_priced(game, rules, sims, tag, stop_depth, budget, prefix, true)
}

#[allow(clippy::too_many_arguments)]
fn simulate_budget_from_priced(game: &Game, rules: &RuleSet, sims: u32, tag: u64, stop_depth: u32, budget: u64, prefix: Vec<SimResult>, price_passage: bool) -> Vec<SimResult> {
    let from = (prefix.len() as u32).min(sims);
    work(|w| w.prefix_simulations += u64::from(from));
    let spent0: u64 = prefix.iter().map(|r| r.ticks as u64).sum();
    let mut ran: Vec<(u32, SimResult)> = prefix.into_iter().take(sims as usize).map(|r| (r.ticks, r)).collect();
    // (the prefix already ends the panel: nothing more to run)
    let done = from >= MIN_SIMS && spent0 >= budget;
    // Every sim is a pure function of (lineage, rules, tag, i); the tick budget only decides
    // how many of them count, in order. Natively they run on all cores and the budget is
    // applied to the ordered results afterwards, so the answer is the sequential one exactly.
    // Cut 27 §1: a waystone start's passage, priced once for the panel (every sim is paid alike).
    let passage = if done || !price_passage { None } else { sim_passage(game, rules) };
    if done {
    } else if parallel_sims() && sims > from + 1 {
        ran.extend(par_sims(game, from, sims, spent0, budget, |base, i| simulate_one(base, rules, tag, stop_depth, i, passage)));
    } else {
        let mut spent: u64 = spent0;
        for i in from..sims {
            if i >= MIN_SIMS && spent >= budget {
                break;
            }
            let r = simulate_one(game, rules, tag, stop_depth, i, passage);
            spent += r.0 as u64;
            ran.push(r);
        }
    }
    let mut out = Vec::with_capacity(ran.len());
    let mut spent: u64 = 0;
    for (i, (n, r)) in ran.into_iter().enumerate() {
        if i as u32 >= MIN_SIMS && spent >= budget {
            break;
        }
        spent += n as u64;
        out.push(r);
    }
    out
}

/// A prospective camp edit, composed as the public editor composes it before any sims.
/// Raw panels also compare historical sets, so projection belongs at edit callers only.
pub fn edited_game(game: &Game, rules: &RuleSet) -> Game {
    let mut g = game.sim_clone();
    g.lineage = crate::packages::project_edit(&game.lineage, rules);
    g.panel_cache = game.panel_cache.clone();
    g.forecast_cache = game.forecast_cache.clone();
    g.refined_panels = game.refined_panels.clone();
    g
}

/// The `i`-th sim of a panel under `rules`, at its first tick (the run started on its seed,
/// its passage paid): what `simulate_one` plays, and what `divergence` plays twice in step.
pub fn sim_game(game: &Game, rules: &RuleSet, tag: u64, i: u32, passage: Option<(u32, i32)>) -> Game {
    let mut g = game.sim_clone();
    g.run = None;
    g.pending_exit = None;
    g.history.clear();
    g.passage = passage;
    let _ = g.set_rules(rules.clone());
    let seed = splitmix(game.lineage.seed ^ splitmix(tag ^ (i as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407)));
    // Cut 24 §2: a named foe rests two runs in three (`LineageState::named_resting`) — the panel
    // plays the sends from here, so its sims turn through the three.
    g.lineage.next_run_id += i % 3;
    g.start_run(Some(seed));
    // Cut 22 §3: the panel's sims share their floors seed by seed (`Run.floor_streams`).
    g.run.as_mut().unwrap().floor_streams = true;
    g
}

/// One fresh expedition (`i`-th of the panel) under `rules`, to `stop_depth` or its end:
/// (ticks spent, result).
fn simulate_one(game: &Game, rules: &RuleSet, tag: u64, stop_depth: u32, i: u32, passage: Option<(u32, i32)>) -> (u32, SimResult) {
    let mut g = sim_game(game, rules, tag, i, passage);
    let mut n = 0;
    let mut settled = false;
    let mut fires = vec![0u32; rules.rows.len()];
    let mut arrive: Vec<(u32, u32, i32)> = g.run.as_ref().map(|r| vec![(r.max_depth, 0, r.carried())]).unwrap_or_default();
    while g.run.as_ref().is_some_and(|r| r.over.is_none() && r.max_depth < stop_depth) && n < SIM_MAX_TICKS {
        let ticks = g.tick_batch(SIM_MAX_TICKS - n, &mut settled);
        count_fires(&mut fires, &g.events);
        g.events.clear();
        n += ticks;
        if let Some(r) = g.run.as_ref() {
            if arrive.last().is_none_or(|a| r.max_depth > a.0) {
                // (Cut 30.5: the gold carried — what the checkpoints secured with the carry since)
                arrive.push((r.max_depth, n, r.carried()));
            }
        }
    }
    let run = g.run.as_ref().unwrap();
    work(|w| { w.simulations += 1; w.ticks += u64::from(n); });
    let mut keyed: Vec<(u64, u32)> = Vec::new();
    for (r, f) in rules.rows.iter().zip(&fires).filter(|(_, f)| **f > 0) {
        let k = row_key(r);
        match keyed.iter_mut().find(|(x, _)| *x == k) {
            Some(e) => e.1 += f,
            None => keyed.push((k, *f)),
        }
    }
    keyed.sort();
    let sworn = crate::oath::sworn(&g.lineage);
    let oath = sworn.is_some_and(|o| crate::oath::kept(o, run));
    let oath_progress = sworn.map_or(0.0, |o| crate::oath::progress(o, run));
    let oath_steps = sworn.map_or(0, |o| crate::oath::steps(o, run));
    (n, SimResult { oath, oath_progress, oath_steps, arrive, ..sim_result(run, n, keyed) })
}

/// Cut 27 §2: a row's key in `SimResult.fires` — its conditions and verb (not its origin).
pub fn row_key(row: &crate::rules::Row) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in format!("{:?}|{:?}", row.conds, row.verb).bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Cut 27 §2: the rows a tick's events say fired (`Ev::Rule` of a row of the set).
fn count_fires(fires: &mut [u32], events: &[crate::wire::Ev]) {
    for e in events {
        if let crate::wire::Ev::Rule { row, .. } = e {
            if let Some(f) = usize::try_from(*row).ok().and_then(|r| fires.get_mut(r)) {
                *f += 1;
            }
        }
    }
}

/// A finished (or stopped) sim's result.
fn sim_result(run: &crate::engine::Run, ticks: u32, fires: Vec<(u64, u32)>) -> SimResult {
    let tier = run.over.unwrap_or(ExitTier::Return);
    // (Cut 27 §1: a waystone start's passage is the send's gold too — paid at the send)
    let loot_kept = run.kept(tier) + run.passage;
    SimResult { max_depth: run.max_depth, tier, cause: run.death_cause.clone(), loot_kept, timed_out: run.timed_out, ticks, loot: run.carried(), fires, oath: false, oath_progress: 0.0, oath_steps: 0, passage: run.passage, arrive: Vec::new() }
}

/// The sims `from..sims` of a panel on the cores, in index order: a worker takes the next index
/// unless the finished prefix already spends the budget before it (the sequential loop's stop, which
/// the caller applies to the ordered results) — so the sims past the budget are not run, but for the
/// few already started. The results are a contiguous run of indices from `from`, each the sim the
/// sequential loop runs.
fn par_sims(game: &Game, from: u32, sims: u32, spent0: u64, budget: u64, run: impl Fn(&Game, u32) -> (u32, SimResult) + Sync) -> Vec<(u32, SimResult)> {
    let n = (sims - from) as usize;
    let threads = sim_threads().min(n);
    let next = std::sync::atomic::AtomicUsize::new(0);
    // (the slots, the finished prefix's length, its ticks, the first index not needed)
    let st = std::sync::Mutex::new((Vec::<Option<(u32, SimResult)>>::from_iter((0..n).map(|_| None)), 0usize, spent0, n));
    let bases: Vec<Game> = (0..threads).map(|_| game.sim_clone()).collect();
    let (next_r, st_r, run) = (&next, &st, &run);
    std::thread::scope(|sc| {
        for base in bases {
            let (next, st) = (next_r, st_r);
            sc.spawn(move || {
                IN_WORKER.with(|w| w.set(true));
                loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if i >= n || i >= st.lock().unwrap().3 {
                        break;
                    }
                    let r = run(&base, from + i as u32);
                    let mut g = st.lock().unwrap();
                    g.0[i] = Some(r);
                    while g.1 < g.3 {
                        let k = g.1;
                        let Some(t) = g.0[k].as_ref().map(|r| r.0 as u64) else { break };
                        if from + k as u32 >= MIN_SIMS && g.2 >= budget {
                            g.3 = k;
                            break;
                        }
                        g.2 += t;
                        g.1 += 1;
                    }
                }
            });
        }
    });
    let (slots, ..) = st.into_inner().unwrap();
    slots.into_iter().map_while(|r| r).collect()
}

/// `f` over every job, results in job order. Natively (and unless `set_parallel_sims(false)`)
/// the jobs are spread over the cores, each worker holding its own `sim_clone` of `game` —
/// `Game` is not `Sync`, and a sim reads only what a `sim_clone` carries (lineage, run,
/// loadout), so `f` sees the same inputs on every path. On wasm, one after another.
pub fn par_map<T: Send + Sync, R: Send>(game: &Game, jobs: Vec<T>, f: impl Fn(&Game, &T) -> R + Sync) -> Vec<R> {
    if !parallel_sims() || jobs.len() <= 1 {
        return jobs.iter().map(|j| f(game, j)).collect();
    }
    par_map_threads(game, jobs, f)
}

thread_local! {
    /// Set on a worker: a job that itself forecasts (a patch's delta) runs its sims in place —
    /// one level of threads, never threads of threads.
    static IN_WORKER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn par_map_threads<T: Send + Sync, R: Send>(game: &Game, jobs: Vec<T>, f: impl Fn(&Game, &T) -> R + Sync) -> Vec<R> {
    let threads = sim_threads().min(jobs.len());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let mut slots: Vec<Option<R>> = (0..jobs.len()).map(|_| None).collect();
    let slots_ref = std::sync::Mutex::new(&mut slots);
    let bases: Vec<Game> = (0..threads).map(|_| game.sim_clone()).collect();
    let (next, slots_ref, jobs, f) = (&next, &slots_ref, &jobs, &f);
    std::thread::scope(|sc| {
        for base in bases {
            sc.spawn(move || loop {
                IN_WORKER.with(|w| w.set(true));
                let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if i >= jobs.len() {
                    break;
                }
                let r = f(&base, &jobs[i]);
                slots_ref.lock().unwrap()[i] = Some(r);
            });
        }
    });
    slots.into_iter().map(|r| r.expect("every job ran")).collect()
}

/// The worker count of `par_map`: every core, or `RIDDLE_THREADS` when set (a tool sharing the
/// machine keeps its load moderate).
pub fn max_threads() -> usize {
    static N: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *N.get_or_init(|| std::env::var("RIDDLE_THREADS").ok().and_then(|v| v.parse().ok()).filter(|n: &usize| *n > 0).unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)))
}

/// Whether a panel's sims run on all cores (native default) or one after another (wasm, and
/// callers that already fill the machine seed by seed — `examples/metrics.rs`).
static PARALLEL_SIMS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(!cfg!(target_arch = "wasm32"));

pub fn parallel_sims() -> bool {
    !IN_WORKER.with(|w| w.get())
        && match width() {
            Some(n) => n > 1 && !cfg!(target_arch = "wasm32"),
            None => PARALLEL_SIMS.load(std::sync::atomic::Ordering::Relaxed),
        }
}

thread_local! {
    /// `with_sim_width`: this thread's panel width, read at each panel (`None`: the process-wide
    /// switch and `max_threads`).
    static WIDTH: std::cell::Cell<Option<&'static std::sync::atomic::AtomicUsize>> = const { std::cell::Cell::new(None) };
}

/// `f` with this thread's panels (`par_map`) on `width` threads, read at each panel — a harness that
/// fills the machine job by job (the gate table) widens a long chain's panels as its other jobs end,
/// whatever `set_parallel_sims` says (1: one after another). The results are the same at any width.
pub fn with_sim_width<R>(width: &'static std::sync::atomic::AtomicUsize, f: impl FnOnce() -> R) -> R {
    let was = WIDTH.with(|w| w.replace(Some(width)));
    let r = f();
    WIDTH.with(|w| w.set(was));
    r
}

fn width() -> Option<usize> {
    WIDTH.with(|w| w.get()).map(|a| a.load(std::sync::atomic::Ordering::Relaxed))
}

/// The worker count of a panel now: this thread's width, else `max_threads`.
fn sim_threads() -> usize {
    width().unwrap_or_else(max_threads).max(1)
}

/// The threads a panel read now would run on (1 when its sims run one after another).
pub fn sim_width() -> usize {
    if parallel_sims() {
        sim_threads()
    } else {
        1
    }
}

pub fn set_parallel_sims(on: bool) {
    PARALLEL_SIMS.store(on && !cfg!(target_arch = "wasm32"), std::sync::atomic::Ordering::Relaxed);
}

/// QA on 92eb880 (qaM: "D6 32 %±13 → 38 %±10 on opening edit, 42 → 50 on opening the cond
/// sheet" — no rule edited): the camp's two passes (`FORECAST_SIMS`, then `REFINE_SIMS` on the
/// same seeds first) were the only thing moving, and any repaint after the refine (a sheet's
/// lineage refresh re-forecasts) went back to the first pass, then forward again. Once the
/// refined panel for this (lineage, rules) exists it *is* the forecast: a read never goes back
/// to the coarser number (`refined` says which one it is).
pub fn forecast(game: &Game) -> Forecast {
    let rules = game.lineage.rules();
    forecast_with(game, rules, camp_sims(game, rules))
}

/// Small UI preview; real progress and the normal/refined API keep their quality.
pub const PREVIEW_SIMS: u32 = 8;
pub fn forecast_estimate(game: &Game) -> Forecast {
    forecast_with(game, game.lineage.rules(), PREVIEW_SIMS)
}

/// The camp's sims for `rules` now: `REFINE_SIMS` once that panel was refined for this
/// (lineage, rules) — whether or not the panel is still in the cache (QA on 1a2a4a9, qaO:
/// the cage picker's three panels filled the cache, its clear-all dropped the refined panel,
/// and the next read went back to the first pass: `D4 68%±13` ↔ `67%±9` on opening the
/// vault) — else `FORECAST_SIMS`.
pub fn camp_sims(game: &Game, rules: &RuleSet) -> u32 {
    let key = panel_key(game, rules, REFINE_SIMS);
    if game.panel_cache.borrow().contains_key(&key) || game.refined_panels.borrow().contains(&key) {
        REFINE_SIMS
    } else {
        FORECAST_SIMS
    }
}

/// Cut 25 §4: the sims an option tablet (forge, cage, start) measures with — the first pass's.
pub fn option_sims(_game: &Game, _rules: &RuleSet) -> u32 {
    FORECAST_SIMS
}

/// Memoise a camp panel: a full cache keeps the active set's own panels (both passes) and
/// drops the rest — the panel the camp shows is never the one evicted.
pub fn panel_insert(game: &Game, key: String, v: Vec<SimResult>) {
    let full = game.panel_cache.borrow().len() >= PANEL_CACHE_MAX;
    if full {
        let before = game.panel_cache.borrow().len();
        let rules = game.lineage.rules();
        let keep = [panel_key(game, rules, FORECAST_SIMS), panel_key(game, rules, REFINE_SIMS)];
        game.panel_cache.borrow_mut().retain(|k, _| keep.contains(k));
        work(|w| w.evicted_panels += (before - game.panel_cache.borrow().len()) as u64);
    }
    game.panel_cache.borrow_mut().insert(key, v);
}

/// Cut 6 §9: the same forecast at `REFINE_SIMS` sims — the same seeds first, then as many
/// again (a second, quieter pass the client runs once the rule set has been still for 2 s).
pub fn forecast_refine(game: &Game) -> Forecast {
    forecast_with(game, game.lineage.rules(), REFINE_SIMS)
}

pub const REFINE_SIMS: u32 = 2 * FORECAST_SIMS;

/// Cut 6 §9: the forecast's seeds are a function of (lineage seed, depth) — re-reading the
/// same set gives the same number, and nothing transient (marks, renown, the rest clock, the
/// run in progress) moves it. The lineage state a sim starts from (facts, gold, class…)
/// still does: `reach_with`'s fingerprint is the same idea. Cut 14 §1: the rules are not in
/// the hash — every set a lineage forecasts plays the same dungeons (seed × depth × sim
/// index), so an edit's delta is a paired difference, not two draws (cohort 10: "the same
/// six rows read 94/6, then 79/21"; "a 10-point edit is unreadable").
pub fn forecast_tag(game: &Game, depth: u32) -> u64 {
    splitmix(0x5EED_F0C4 ^ game.lineage.seed.rotate_left(17) ^ ((depth as u64) << 40))
}

/// The tick budget of the camp's panel (`forecast_with` at `FORECAST_SIMS`). Cut 25 §4 (AM: 5–9 s
/// of `…` after an edit, ~8 s for the forge, on a D11 lineage after an absence — sims there run
/// ~15 000 ticks, so a panel spends its whole budget; the forge and the cage are three panels
/// each): 450 000 (was 550 000) — a shallow lineage's 50 sims stay under it, a D11 panel runs
/// ~30 sims first and the refine twice that.
pub const CAMP_TICK_BUDGET: u64 = 450_000;

/// QA on 23ed91f: the camp's panel for `rules` — every sim to its exit on the camp's seeds
/// (`forecast_tag` at `known_to`) under the camp's budget, `sims` of them — memoised on the
/// game by (lineage, rules): the death screen's patch deltas and the camp that follows read
/// the same panels.
pub fn camp_panel(game: &Game, rules: &RuleSet, sims: u32) -> Vec<SimResult> {
    camp_panel_from(game, rules, sims, Vec::new())
}

/// Continue a screening panel for exactly this game/rules/tag, in index order.
/// The wall search owns its prefix so cache eviction cannot discard its work.
pub(crate) fn camp_panel_from(game: &Game, rules: &RuleSet, sims: u32, prefix: Vec<SimResult>) -> Vec<SimResult> {
    camp_panel_priced(game, rules, sims, prefix, true)
}

/// Outcome-only forecast: skipped-floor gold cannot affect a single simulation's
/// combat/depth/exit, and these callers do not read its gold totals. Never reuse
/// these incomplete results in a gold forecast or an actual send.
pub(crate) fn camp_panel_outcomes(game: &Game, rules: &RuleSet, sims: u32) -> Vec<SimResult> {
    camp_panel_outcomes_from(game, rules, sims, Vec::new())
}

/// Continue an owned screening prefix without pricing unused skipped-floor gold.
pub(crate) fn camp_panel_outcomes_from(game: &Game, rules: &RuleSet, sims: u32, prefix: Vec<SimResult>) -> Vec<SimResult> {
    camp_panel_priced(game, rules, sims, prefix, false)
}

fn camp_panel_priced(game: &Game, rules: &RuleSet, sims: u32, mut prefix: Vec<SimResult>, price_passage: bool) -> Vec<SimResult> {
    let known_to = game.lineage.best_depth + 1;
    let tag = forecast_tag(game, known_to);
    let budget = panel_budget(sims);
    let normal_key = panel_key(game, rules, sims);
    // From D1 there is no skipped-floor gold, so ordinary outcome results
    // are already complete and may also cover later gold reads. Keep explicit
    // refinement separate unless its normal quality metadata was requested.
    let normal_cache = price_passage || (sims < REFINE_SIMS && sim_start(game) <= 1);
    let namespace = if normal_cache { "" } else { "outcomes:" };
    let key = format!("{namespace}{normal_key}");
    if let Some(v) = game.panel_cache.borrow().get(&key) {
        work(|w| w.panel_hits += 1);
        return v.clone();
    }
    work(|w| w.panel_misses += 1);
    // Every size/budget runs the same indexed sends. Reuse any exact-input prefix,
    // including wall12→48, and truncate it under this request's ordered budget.
    let parts: Vec<_> = normal_key.splitn(5, ':').collect();
    for (k, v) in game.panel_cache.borrow().iter() {
        // Complete priced results contain every outcome field. Reuse that work
        // in this direction only; normal reads cannot match outcomes: keys.
        let k = if price_passage { k.as_str() } else { k.strip_prefix("outcomes:").unwrap_or(k) };
        let p: Vec<_> = k.splitn(5, ':').collect();
        if p.len() == 5 && p[0] == parts[0] && p[2] == parts[2] && p[4] == parts[4] && v.len().min(sims as usize) > prefix.len().min(sims as usize) {
            prefix = v.clone();
        }
    }
    prefix.truncate(sims as usize);
    let mut spent = 0u64;
    let mut used = 0;
    for r in &prefix {
        if used >= MIN_SIMS as usize && spent >= budget { break; }
        spent += u64::from(r.ticks);
        used += 1;
    }
    prefix.truncate(used);
    let ended = simulate_budget_from_priced(game, rules, sims, tag, u32::MAX, budget, prefix, price_passage);
    if price_passage && sims >= REFINE_SIMS {
        let mut r = game.refined_panels.borrow_mut();
        if r.len() >= REFINED_MAX {
            r.clear();
        }
        r.insert(key.clone());
    }
    panel_insert(game, key, ended.clone());
    ended
}

/// How many refined (lineage, rules) keys a game remembers (`camp_sims`).
pub const REFINED_MAX: usize = 256;

/// Cut 19 §1: 32 (was 16) — the cage picker adds three panels per rule set.
pub const PANEL_CACHE_MAX: usize = 32;

/// Cut 19 §1 (rater AA: the vault's `cage → armour` "raised bank-at-D7 from 54 % to 90 %, more
/// than all my rule edits", found at 25 min): each cage preference measured for the active set
/// — the camp's panel (the forecast's own sims count: the refined panel once it exists) with
/// `vault_pref` set to the option, against the current preference's panel. Memoised like the
/// camp's panels (`Game.panel_cache`: the options' panels land there, keyed by the lineage
/// fingerprint, which carries the preference).
pub fn cage_forecast(game: &Game) -> Vec<crate::wire::CageOption> {
    let rules = game.lineage.rules().clone();
    cage_forecast_at(game, camp_sims(game, &rules))
}

/// QA on 524827b (qaAA: the cage sheet's current choice read `weapon D6 6%` while the camp above
/// it read `D6 11%` — the sheet on the first pass's 50 sims, the camp on its refined 100; `armour
/// D6 60% ▲54`, and picked the camp read `57% · cage +46`): the options are measured on the
/// sims the camp shows (`sims`: the caller's — a measure lane's mirror does not know whether
/// the camp refined), so the current option *is* the camp's number, and the option picked is
/// the camp's next panel at the same pass. Cut 25 §4 kept options on the first pass for speed;
/// a refined camp now costs the sheet its refined panels (the prefix of each is reused).
pub fn cage_forecast_at(game: &Game, sims: u32) -> Vec<crate::wire::CageOption> {
    let rules = game.lineage.rules().clone();
    let sims = if sims > FORECAST_SIMS { REFINE_SIMS } else { FORECAST_SIMS };
    // The bar the reach is read at: the set's bank row's depth (`depth ≥ d → bank`), else the
    // lineage's best depth.
    let bank_depth = rules.rows.iter().filter(|r| r.verb.v == "bank").filter_map(|r| r.conds.iter().find(|c| c.k == "depth>=").and_then(|c| c.n)).map(|n| n.max(1) as u32).min();
    // QA on 0c6e126 (qaY: `armour D5 +10` on the sheet, then the camp's D6 25 → 72 % on the tap): with no bank row the reach is read
    // at the frontier the camp leads with (best + 1, `vsLine`'s head) while the current panel still reaches it (over `WALL_REACH`),
    // else at the best depth.
    let base_panel = camp_panel(game, &rules, sims);
    let frontier = game.lineage.best_depth + 1;
    let at_frontier = base_panel.iter().filter(|r| r.max_depth >= frontier).count() as f64 / base_panel.len().max(1) as f64;
    let open = if at_frontier > WALL_REACH + 1e-9 { frontier } else { game.lineage.best_depth };
    let depth = bank_depth.unwrap_or(open).clamp(1, game.lineage.best_depth + 1);
    /// (reach at `depth`, bank share, gold per send, sims).
    type Read = (f64, f64, f64, usize);
    let read = |ended: &[SimResult]| -> Read {
        let n = ended.len().max(1) as f64;
        let reach = ended.iter().filter(|r| r.max_depth >= depth).count() as f64 / n;
        let bank = ended.iter().filter(|r| r.tier == ExitTier::Bank && !r.timed_out).count() as f64 / n;
        let gold = ended.iter().map(|r| r.loot_kept as f64).sum::<f64>() / n;
        (reach, bank, gold, ended.len())
    };
    let current = game.lineage.vault_pref.clone();
    let base = read(&base_panel);
    const PREFS: [&str; 4] = ["weapon", "armour", "potion", "scroll"];
    let others: Vec<&str> = PREFS.iter().copied().filter(|p| *p != current).collect();
    let measured: Vec<(Read, BTreeMap<String, Vec<SimResult>>)> = others
        .iter()
        .map(|pref| {
            let mut g = game.sim_clone();
            g.lineage.vault_pref = (*pref).into();
            // The clone starts with an empty cache: seed it with the game's, so an option already
            // measured (this or an earlier open of the cage tablet) is a lookup, not a re-simulation.
            *g.panel_cache.borrow_mut() = game.panel_cache.borrow().clone();
            let r = read(&camp_panel(&g, &rules, sims));
            (r, g.panel_cache.into_inner().into_iter().collect())
        })
        .collect();
    let mut out = Vec::with_capacity(PREFS.len());
    let mut m = measured.into_iter();
    for pref in PREFS {
        let (reach, bank, gold, n) = if pref == current {
            base
        } else {
            let (r, panels) = m.next().expect("one panel per other preference");
            for (k, v) in panels {
                panel_insert(game, k, v);
            }
            r
        };
        let banks = bank > 0.0 || base.1 > 0.0;
        let (reach_delta, bank_delta, gold_delta) = (reach - base.0, bank - base.1, gold - base.2);
        out.push(crate::wire::CageOption {
            pref: pref.into(),
            current: pref == current,
            depth,
            reach,
            reach_delta,
            bank,
            bank_delta,
            gold,
            gold_delta,
            delta: if banks { bank_delta } else { reach_delta },
            pm: half_width(reach, n),
            refined: sims > FORECAST_SIMS,
        });
    }
    out
}

fn panel_budget(sims: u32) -> u64 {
    CAMP_TICK_BUDGET * (sims as u64).div_ceil(FORECAST_SIMS as u64).max(1)
}

/// The memo key of the camp's panel for `rules` at `sims` (`camp_panel`).
pub fn panel_key(game: &Game, rules: &RuleSet, sims: u32) -> String {
    let tag = forecast_tag(game, game.lineage.best_depth + 1);
    format!("{}:{sims}:{tag}:{}:{}", lineage_key(game), panel_budget(sims), played_key(game, rules))
}

/// QA on a946e04 (qaS: adding a row the editor marks dead — `↑ R3` — moved the shaft D5 72 →
/// 74 %, D6 56 → 61 %): what of a set a sim can play — the set less its shadowed rows (a
/// shadowed row never acts: `turn::choose_and_act` skips it `same as R<n>`), with the kinds
/// the send's repeat re-buys for the whole set (`LineageState::row_kinds_of`: a shadowed
/// `drink heal` still packs a heal). A shadowed row stays in the key when another saved set
/// holds it (a shrine lends only a row the active set lacks) or when the set is over its row
/// cap (the send refuses it). So a dead row reads the panel of the set without it — the
/// refined one when that was refined — and the forecast does not move.
pub fn played_key(game: &Game, rules: &RuleSet) -> String {
    let l = &game.lineage;
    let kinds = l.row_kinds_of(rules);
    let shadowed = l.shadowed_by(rules);
    if shadowed.is_empty() || rules.own_rows() > l.max_rows() {
        return format!("{}|{kinds:?}", rules_key(rules));
    }
    let active = l.active_set.min(l.sets.len().saturating_sub(1));
    let lent = |r: &crate::rules::Row| l.sets.iter().enumerate().any(|(i, s)| i != active && s.rows.contains(r));
    let rows: Vec<crate::rules::Row> = rules.rows.iter().enumerate().filter(|(i, r)| shadowed.get(*i).copied().flatten().is_none() || lent(r)).map(|(_, r)| r.clone()).collect();
    let played = RuleSet { rows, name: rules.name.clone(), route: rules.route.clone() };
    format!("{}|{kinds:?}", rules_key(&played))
}

/// QA on 1a2a4a9: what of a set a sim plays — each row's conditions and verb. A row's
/// `origin` (the client re-tags rows `player` / `patch` as the editor opens and edits) and
/// the set's name are not played, so a `setRules` of the same rows re-tagged is the same
/// panel, not a new first pass.
pub fn rules_key(rules: &RuleSet) -> String {
    let rows: Vec<(&Vec<crate::rules::Cond>, &crate::rules::Verb)> = rules.rows.iter().map(|r| (&r.conds, &r.verb)).collect();
    let key = serde_json::to_string(&rows).unwrap_or_default();
    // Cut 26 §2: the route is played (the base order keys as before).
    if rules.route.is_empty() {
        key
    } else {
        format!("{key}|route {:?}", rules.route)
    }
}

/// The camp bar at `depth` for `rules` (`camp_panel` at `FORECAST_SIMS`): (reach, sims).
pub fn camp_reach(game: &Game, rules: &RuleSet, depth: u32) -> (f64, u32) {
    let ended = camp_panel(game, rules, FORECAST_SIMS);
    let n = ended.len().max(1);
    (ended.iter().filter(|r| r.max_depth >= depth).count() as f64 / n as f64, n as u32)
}

pub fn forecast_with(game: &Game, rules: &RuleSet, sims: u32) -> Forecast {
    let known_to = game.lineage.best_depth + 1;
    // QA on 3d71c33: one panel for the bars and the ends. The ends used to come from their own
    // small panel (20 sims on other seeds, cut by the delta budget to ~5–10 on a D8+ lineage)
    // while the bars stopped each sim at `known_to`: `D8 44% · bank 57%` under
    // `depth ≥ 8 → bank`, and `bank 42%` for a set that then banked 0 of 16 overnight. Every
    // sim now runs to its exit — its reach at any depth ≤ `known_to` is the one a sim stopped
    // there reads, and a sim that stops short of `known_to` costs the same — so a bank at
    // `depth ≥ d` is counted among the sims that reached `d` (bank ≤ reach(d), exactly), and
    // the killers are the bars' own. The budget is the two old panels' together (the reach
    // panel's and the ends'); the refine pass runs twice the sims under twice that (the same
    // seeds first). A sim cut off at D(best+1) no longer reads as a "return" (Cut 12 §3).
    let ended = camp_panel(game, rules, sims);
    let n = ended.len().max(1) as f64;
    let reach_at = |d: u32| ended.iter().filter(|r| r.max_depth >= d).count() as f64 / n;
    // Cut 20 §5: the bounty floor is a notch of its own, below `known_to` when it lies deeper.
    let last = known_to.max(game.lineage.bounty.unwrap_or(0));
    // Cut 21 §1: a row at or above the start floor is passed by every send (reach 1.0): no
    // boss stands between, so no `try`.
    let start = sim_start(game);
    // Cut 26 §2: the forecast prices the set's route (its bosses where the route puts them).
    let route = rules.route();
    let depths = (1..=last)
        .map(|d| {
            let reach = reach_at(d);
            let wall = wall_on(route, d, reach, reach_at(d.saturating_sub(1)));
            let try_ = if d > start { try_row(game, rules, d) } else { None };
            ForecastDepth { depth: d, reach, pm: Some(half_width(reach, ended.len())), try_, wall, bounty: game.lineage.bounty == Some(d), boss: route.boss(d).map(str::to_string), biome: (!route.is_base()).then(|| route.biome(d).name().to_string()), clear: None }
        })
        .collect();
    let mut causes: BTreeMap<String, u32> = BTreeMap::new();
    let mut deaths = 0u32;
    for r in &ended {
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
    // A stall (the run cap, a floor shuffled) came home by nothing in the rules: its own
    // share, not a return's (QA on 952e306: "`return 70%` — no return verb in my rules").
    let share = |t: ExitTier| ended.iter().filter(|r| r.tier == t && !r.timed_out).count() as f64 / n;
    let stall = ended.iter().filter(|r| r.timed_out).count() as f64 / n;
    let gold = ended.iter().map(|r| r.loot_kept as f64).sum::<f64>() / n;
    let death = share(ExitTier::Death);
    // Cut 29 §6 (AX: a D12 bank of $81 under a ~$260 forecast — the forecast counts the waystone's
    // passage, paid at the send, the exit line does not): the passage's share of `gold`, its own.
    let passage = ended.iter().map(|r| r.passage as f64).sum::<f64>() / n;
    let ends = (!ended.is_empty()).then(|| ForecastEnds { bank: share(ExitTier::Bank), return_: share(ExitTier::Return), death, stall, gold, pm: half_width(death, ended.len()), passage });
    let n_sims = ended.len() as u32;
    let mut depths: Vec<ForecastDepth> = depths;
    for d in depths.iter_mut().filter(|d| d.depth >= start) {
        d.clear = floor_clear(&ended, d.depth);
    }
    let fold_to = fold_to(game, &ended, start);
    let oath = crate::oath::share(&game.lineage, &ended);
    Forecast { oath, depths, causes, known_to, ends, refined: sims > FORECAST_SIMS, shadowed_by: game.lineage.shadowed_by(rules), start, sims: n_sims, low: low_pct(n_sims), fold_to }
}

/// Cut 27 §1: a floor the watch folds — the share of the sims on it that got through it is at
/// least this (`ForecastDepth.clear`).
pub const FOLD_CLEAR: f64 = 0.95;

/// Cut 27 §1: the share of `ended`'s sims that stood on floor `d` and got through it (reached
/// `d + 1`); None when none stood on it. A sim that ended on `d` — a death, a stall, a bank or a
/// return there — did not clear it.
pub fn floor_clear(ended: &[SimResult], d: u32) -> Option<f64> {
    let on = ended.iter().filter(|r| r.max_depth >= d).count();
    (on > 0).then(|| ended.iter().filter(|r| r.max_depth > d).count() as f64 / on as f64)
}

/// Cut 27 §1: the last floor of the fold — every floor from `start` to it clears ≥ `FOLD_CLEAR`
/// on `ended` and is known (≤ the lineage's best depth: a floor never reached is never folded).
pub fn fold_to(game: &Game, ended: &[SimResult], start: u32) -> Option<u32> {
    let mut to = None;
    for d in start.max(1)..=game.lineage.best_depth {
        match floor_clear(ended, d) {
            Some(c) if c >= FOLD_CLEAR - 1e-9 => to = Some(d),
            _ => break,
        }
    }
    to
}

/// Cut 27 §1: the send's fold (`Game::fold_plan`) — the last folded floor and each folded
/// floor's clear, on the camp's panel for `rules` (the pass the camp painted: cached).
pub fn fold_plan(game: &Game, rules: &RuleSet) -> Option<(u32, Vec<(u32, f64)>)> {
    let ended = camp_panel(game, rules, camp_sims(game, rules));
    let start = sim_start(game);
    let to = fold_to(game, &ended, start)?;
    Some((to, (start..=to).map(|d| (d, floor_clear(&ended, d).unwrap_or(1.0))).collect()))
}

/// Cut 27 §1: the sims a passage is priced on (the camp's seeds, from D1, each stopped on
/// arriving at the start floor).
pub const PASSAGE_SIMS: u32 = 20;

/// Cut 27 §1: the passage of a send on `rules` from the lineage's start — (start, coins) when
/// the sends start below D1 (`sim_start`), priced by `passage_for`; None from D1.
pub fn sim_passage(game: &Game, rules: &RuleSet) -> Option<(u32, i32)> {
    let start = sim_start(game);
    (start > 1).then(|| (start, passage_for(game, rules, start)))
}

/// Cut 27 §1 (P2: "a lit waystone start is never dominated on gold/hr by D1 for a set that clears
/// the band ≥ 95 %"): the gold the floors above `start` would have brought a send on `rules` —
/// `PASSAGE_SIMS` sends from D1 on the camp's seeds, each stopped on arriving at `start`: the mean gold
/// they bring from the floors above (an arrival's carry, an earlier end's kept gold — Cut 30.5). Memoised
/// per (lineage, rules).
pub fn passage_for(game: &Game, rules: &RuleSet, start: u32) -> i32 {
    passage_from(game, rules, start, &[])
}

/// `passage_for` reading `have` for its first sims: sims `0..k` of a panel from D1 on this
/// lineage (its start at 1, no passage) and `rules`, on the camp's seeds, each run at least to
/// `start` or its end — a camp panel of the lineage at D1, or the sims of a deeper passage.
/// Each is cut where it arrived at `start` (`SimResult::cut_at`): the sim the passage runs
/// stops there and has played the same ticks, so the panel is the one `passage_for` runs —
/// the sims past `have` run as before, the budget applied to the whole in order.
pub fn passage_from(game: &Game, rules: &RuleSet, start: u32, have: &[SimResult]) -> i32 {
    if start <= 1 {
        return 0;
    }
    let mut g = game.sim_clone();
    g.lineage.start = 1;
    g.passage = None;
    let key = format!("passage:{}:{start}:{}", lineage_key(&g), rules_key(rules));
    if let Some(v) = game.forecast_cache.borrow().get(&key) {
        return v.0 as i32;
    }
    let ended = passage_sims(&g, rules, start, have);
    // Cut 30.5 (the owner, 2026-10-02: a stone's passage counts as a checkpoint): the gold a send from D1 brings
    // home from the floors above — the carry of each sim that arrived (secured at the stone), and what each one
    // that ended above it kept — whatever share clears them (was: the arrivals' mean carry, and nothing unless every
    // floor above cleared ≥ `FOLD_CLEAR`: under the record rule a run goes home hurt in the D5 band and few sets
    // cleared, so a stone paid nothing on four sends in five and fell under D1's gold an hour)
    let coins = if ended.is_empty() { 0 } else { (ended.iter().map(|r| if r.max_depth >= start { r.loot } else { r.loot_kept } as f64).sum::<f64>() / ended.len() as f64).round() as i32 };
    let mut cache = game.forecast_cache.borrow_mut();
    if cache.len() + 1 >= FORECAST_CACHE_MAX {
        cache.clear();
    }
    cache.insert(key, (coins as f64, ended.len() as u32));
    coins
}

/// The sims a passage to `start` is priced on, for `g` already at D1 with no passage
/// (`passage_from`'s clone), `have` read first.
fn passage_sims(g: &Game, rules: &RuleSet, start: u32, have: &[SimResult]) -> Vec<SimResult> {
    let tag = forecast_tag(g, g.lineage.best_depth + 1);
    let prefix: Vec<SimResult> = have.iter().take(PASSAGE_SIMS as usize).map(|r| r.cut_at(start)).collect();
    simulate_budget_from(g, rules, PASSAGE_SIMS, tag, start, DELTA_TICK_BUDGET, prefix)
}

/// The sims of `game`'s passage to `start` on `rules` (`passage_for`'s, not memoised): for a
/// caller that prices passages to shallower floors of the same lineage and set from them
/// (`passage_from`).
pub fn passage_run(game: &Game, rules: &RuleSet, start: u32) -> Vec<SimResult> {
    let mut g = game.sim_clone();
    g.lineage.start = 1;
    g.passage = None;
    passage_sims(&g, rules, start, &[])
}

/// Cut 23 §2: the smallest share one of `sims` sims makes, in whole percent (`⌈100 / sims⌉`;
/// 100 with no sims) — a share sampled at 0 reads `<{low}%`.
pub fn low_pct(sims: u32) -> u32 {
    if sims == 0 {
        100
    } else {
        100u32.div_ceil(sims)
    }
}

/// Cut 23 §2 (AJ: `death 0%`, then death): a sampled share as the camp prints it — whole
/// percent, and `<N%` (N = `low_pct`) for a share no sim of `sims` showed: a panel of 50 that
/// saw no death says `death <2%`, never `0%`.
pub fn share_label(share: f64, sims: u32) -> String {
    if share <= 0.0 {
        format!("<{}%", low_pct(sims))
    } else {
        format!("{}%", (share * 100.0).round() as i64)
    }
}

/// Cut 21 §1: the floor the camp's sims start on — the lineage's start when it is lit and the
/// purse pays its toll (`Game::pay_start`), else 1. The forecast's rows above it read 1.0
/// (every sim is there from its first tick: `max_depth ≥ start`); the client folds them.
pub fn sim_start(game: &Game) -> u32 {
    let l = &game.lineage;
    let d = l.start.max(1);
    if l.start_payable(d) {
        d
    } else {
        1
    }
}

/// Cut 21 §1: the start tablet — each start the lineage can choose (D1 and every lit
/// waystone) measured for the active set: the camp's panel with `start` set to the option
/// (the forecast's own sims: the refined panel once it exists, the same seeds), against the
/// current start's panel. `depth` is the bar the reach is read at (the set's bank row's
/// depth, else the lineage's best depth — never above the option's own floor, where it
/// reads 1.0). `gold` is what a send brings home; `net` less the option's toll; the deltas
/// are the net's and the bank share's. A start the purse cannot pay (`toll > gold`) is
/// measured as it would play — from D1 — and marked `short`. Memoised like the cage tablet
/// (the options' panels land in `Game.panel_cache`, keyed by the lineage fingerprint, which
/// carries the start).
pub fn start_forecast(game: &Game) -> Vec<crate::wire::StartOption> {
    let rules = game.lineage.rules().clone();
    start_forecast_at(game, camp_sims(game, &rules))
}

/// QA on 308f045 (qaAD: the start sheet's `D1 · bank 90% · death 10%` beside the shaft's `bank
/// 92% · death 8%`): the starts measured on the pass the camp shows (`sims`: the caller's, as the
/// cage and fork tablets'), so the start the camp plays reads the camp's own numbers. Cut 25 §4
/// kept options on the first pass for speed; a refined camp reuses each panel's first pass.
pub fn start_forecast_at(game: &Game, sims: u32) -> Vec<crate::wire::StartOption> {
    let rules = game.lineage.rules().clone();
    let sims = if sims > FORECAST_SIMS { REFINE_SIMS } else { FORECAST_SIMS };
    let bank_depth = rules.rows.iter().filter(|r| r.verb.v == "bank").filter_map(|r| r.conds.iter().find(|c| c.k == "depth>=").and_then(|c| c.n)).map(|n| n.max(1) as u32).min();
    let bar = bank_depth.unwrap_or(game.lineage.best_depth).clamp(1, game.lineage.best_depth + 1);
    let current = game.lineage.start.max(1);
    let mut options: Vec<u32> = vec![1];
    options.extend(game.lineage.stones());
    /// (reach at `bar`, bank share, gold per send, sims, death share).
    type Read = (f64, f64, f64, usize, f64);
    let read = |ended: &[SimResult]| -> Read {
        let n = ended.len().max(1) as f64;
        let reach = ended.iter().filter(|r| r.max_depth >= bar).count() as f64 / n;
        let bank = ended.iter().filter(|r| r.tier == ExitTier::Bank && !r.timed_out).count() as f64 / n;
        let gold = ended.iter().map(|r| r.loot_kept as f64).sum::<f64>() / n;
        let death = ended.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / n;
        (reach, bank, gold, ended.len(), death)
    };
    let base = read(&camp_panel(game, &rules, sims));
    let base_toll = if sim_start(game) > 1 && !game.lineage.night_passes.contains(&current) { crate::engine::LineageState::start_toll(current) } else { 0 };
    let others: Vec<u32> = options.iter().copied().filter(|d| *d != current).collect();
    let measured: Vec<(Read, BTreeMap<String, Vec<SimResult>>)> = others
        .iter()
        .map(|&d| {
            let mut g = game.sim_clone();
            g.lineage.start = d;
            *g.panel_cache.borrow_mut() = game.panel_cache.borrow().clone();
            let r = read(&camp_panel(&g, &rules, sims));
            (r, g.panel_cache.into_inner().into_iter().collect())
        })
        .collect();
    let mut m = measured.into_iter();
    let mut out = Vec::with_capacity(options.len());
    for d in options {
        let toll = crate::engine::LineageState::start_toll(d);
        // QA on a946e04: tonight's pass is paid — the next send from it pays nothing.
        let pass = game.lineage.night_passes.contains(&d);
        let short = !game.lineage.start_payable(d);
        let (reach, bank, gold, n, death) = if d == current {
            base
        } else {
            let (r, panels) = m.next().expect("one panel per other start");
            for (k, v) in panels {
                panel_insert(game, k, v);
            }
            r
        };
        let paid = if short || pass { 0 } else { toll };
        let net = gold - paid as f64;
        let banks = bank > 0.0 || base.1 > 0.0;
        // Cut 27 §1: the passage a send from here is paid (in `gold`; the option's panel paid it).
        let passage = if short || d <= 1 { 0 } else { passage_for(game, &rules, d) };
        out.push(crate::wire::StartOption {
            start: d,
            current: d == current,
            passage,
            toll,
            biome: rules.route().biome(d).name().into(),
            short,
            pass,
            depth: bar,
            reach,
            reach_delta: reach - base.0,
            bank,
            bank_delta: bank - base.1,
            gold,
            gold_delta: gold - base.2,
            net,
            net_delta: net - (base.2 - base_toll as f64),
            delta: if banks { bank - base.1 } else { reach - base.0 },
            pm: half_width(reach, n),
            death,
            death_delta: death - base.4,
            refined: sims > FORECAST_SIMS,
            low: low_pct(n as u32),
        });
    }
    out
}

/// Cut 26 §2: the fork chip's tablet — both stairs of the fork at `fork` for the active set:
/// the camp's panel for the set's own route and for the route taking the other stair (a far
/// stair drops the neighbouring far stairs it overlaps: `Route::with`), on the camp's seeds (the
/// first pass: one more panel), read at the band's last floor (`fens D8 61% · burrows D8 34%`).
/// Memoised like the start tablet. Empty for a fork the route gives no choice at, or unseen.
pub fn fork_forecast(game: &Game, fork: u32) -> Vec<crate::wire::ForkOption> {
    let rules = game.lineage.rules().clone();
    fork_forecast_at(game, fork, camp_sims(game, &rules))
}

/// QA on 308f045 (qaAC: the sheet's `fens D8 12%`, picked: the shaft's `D8 16%`; `burrows D8
/// 57%` → `50%`): both stairs are measured on the pass the camp shows (`sims`: the caller's, as
/// the cage tablet's — the refined 100 once the camp refined), so the stair taken reads the
/// shaft's own number and the stair picked is the camp's next panel at the same pass.
pub fn fork_forecast_at(game: &Game, fork: u32, sims: u32) -> Vec<crate::wire::ForkOption> {
    use crate::descent::{Route, BANDS, BASE_ORDER, FORKS};
    let rules = game.lineage.rules().clone();
    let sims = if sims > FORECAST_SIMS { REFINE_SIMS } else { FORECAST_SIMS };
    let Some(i) = FORKS.iter().position(|f| *f == fork) else { return Vec::new() };
    let route = rules.route();
    if !route.fork_open(fork) || !crate::descent::fork_open_for(game.lineage.unlocks.contains("route2"), fork) {
        return Vec::new();
    }
    // QA on 308f045 (qaAD: `burrows D8 <2% · fens D8 <2%` before the absence, the shaft differing at D6 27 % vs 14 %): the
    // lanes are compared where they have signal — the band's last floor while a sim of either lane reaches it (over
    // `WALL_REACH`), else the deepest floor of the band one does, else the fork's own floor.
    let panels: Vec<(Route, Vec<SimResult>)> = [false, true]
        .into_iter()
        .map(|far| {
            let r: Route = route.with(fork, far);
            let p = if r == route { camp_panel(game, &rules, sims) } else { camp_panel(game, &rules.clone().with_route(r), sims) };
            (r, p)
        })
        .collect();
    let share_at = |p: &[SimResult], d: u32| p.iter().filter(|r| r.max_depth >= d).count() as f64 / p.len().max(1) as f64;
    let bar = (fork..=BANDS[i].1).rev().find(|d| panels.iter().any(|(_, p)| share_at(p, *d) > WALL_REACH)).unwrap_or(fork);
    type Read = (f64, f64, f64, usize, f64);
    let read = |ended: &[SimResult]| -> Read {
        let n = ended.len().max(1) as f64;
        let reach = ended.iter().filter(|r| r.max_depth >= bar).count() as f64 / n;
        let bank = ended.iter().filter(|r| r.tier == ExitTier::Bank && !r.timed_out).count() as f64 / n;
        let gold = ended.iter().map(|r| r.loot_kept as f64).sum::<f64>() / n;
        let death = ended.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / n;
        (reach, bank, gold, ended.len(), death)
    };
    let base = read(&panels.iter().find(|(r, _)| *r == route).map(|(_, p)| p.clone()).unwrap_or_default());
    let mut out = Vec::with_capacity(2);
    for (far, (r, panel)) in [false, true].into_iter().zip(panels.iter()) {
        let r: Route = *r;
        let current = r == route;
        let (reach, bank, gold, n, death) = if current { base } else { read(panel) };
        let banks = bank > 0.0 || base.1 > 0.0;
        out.push(crate::wire::ForkOption {
            fork,
            biome: BASE_ORDER[i + far as usize].name().into(),
            far,
            current,
            route: r.forks(),
            depth: bar,
            reach,
            reach_delta: reach - base.0,
            bank,
            bank_delta: bank - base.1,
            gold,
            gold_delta: gold - base.2,
            death,
            death_delta: death - base.4,
            delta: if banks { bank - base.1 } else { reach - base.0 },
            pm: half_width(reach, n),
            refined: sims > FORECAST_SIMS,
            low: low_pct(n as u32),
        });
    }
    out
}

/// Cut 22 §3: the paired move of the active set against `prev` — both camp panels at the
/// active set's sims count (the same seeds: `forecast_tag` at `known_to`, the rules not in
/// it), memoised like the forecast (the previous set's panel is usually still cached from its
/// own paint). Per depth of the shaft and on the ends: the mean per-seed difference and its
/// 95 % half-width from the per-seed differences (`paired`), over the seeds both panels ran.
pub fn forecast_vs(game: &Game, prev: &RuleSet) -> crate::wire::ForecastVs {
    forecast_vs_with(game, prev, camp_sims(game, game.lineage.rules()))
}

pub fn forecast_vs_estimate(game: &Game, prev: &RuleSet) -> crate::wire::ForecastVs {
    forecast_vs_with(game, prev, PREVIEW_SIMS)
}

fn forecast_vs_with(game: &Game, prev: &RuleSet, sims: u32) -> crate::wire::ForecastVs {
    let rules = game.lineage.rules().clone();
    let a = camp_panel(game, &rules, sims);
    let mut b = camp_panel(game, prev, sims);
    // QA on 524827b (qaAA: `vs sent…` held > 25 s after a cage change): under the tick budget the
    // two panels may run different counts of sims, and a move paired over fewer than the bars'
    // own never matched the forecast painted (`ForecastVs.sims` ≠ `Forecast.sims`) — the sent
    // set's panel runs on to the active set's count (the same seeds: a pure function of the index).
    if b.len() < a.len() {
        let tag = forecast_tag(game, game.lineage.best_depth + 1);
        b = simulate_budget_from(game, prev, a.len() as u32, tag, u32::MAX, u64::MAX, b);
    }
    vs_between(game, &a, &b, sims > FORECAST_SIMS)
}

/// Cut 22 §3: the paired move `a − b` (two panels on the same seeds, over the sims both ran) per
/// depth of the camp's shaft and on the ends.
pub fn vs_between(game: &Game, a: &[SimResult], b: &[SimResult], refined: bool) -> crate::wire::ForecastVs {
    let n = a.len().min(b.len());
    let (a, b) = (&a[..n], &b[..n]);
    let known_to = game.lineage.best_depth + 1;
    let last = known_to.max(game.lineage.bounty.unwrap_or(0));
    let ind = |x: bool| if x { 1.0 } else { 0.0 };
    let depths = (1..=last)
        .map(|d| {
            let m = paired(a, b, |r| ind(r.max_depth >= d));
            let reach = a.iter().filter(|r| r.max_depth >= d).count() as f64 / n.max(1) as f64;
            crate::wire::VsDepth { depth: d, delta: m.delta, pm: m.pm, abs_pm: half_width(reach, n), base: m.base }
        })
        .collect();
    let tier = |t: ExitTier| move |r: &SimResult| ind(r.tier == t && !r.timed_out);
    crate::wire::ForecastVs {
        oath: crate::oath::sworn(&game.lineage).map(|_| paired(a, b, |r| ind(r.oath))),
        depths,
        bank: paired(a, b, tier(ExitTier::Bank)),
        death: paired(a, b, |r| ind(r.tier == ExitTier::Death)),
        return_: paired(a, b, tier(ExitTier::Return)),
        gold: paired(a, b, |r| r.loot_kept as f64),
        stall: paired(a, b, |r| ind(r.timed_out)),
        sims: n as u32,
        refined,
    }
}

/// Cut 28 §2: the state since a send, in the order the attribution applies it — each a part of
/// the move (`MovePart.kind`) and what it moves of the lineage (`take` copies today's onto the
/// send's).
const STATE_PARTS: [&str; 6] = ["party", "kit", "purse", "start", "facts", "heir"];

/// Copy the fields of state part `kind` from `now` onto `l` (the lineage at the send, being moved
/// to today's a part at a time); `heir` takes the whole of today's lineage (all that is left).
fn take_part(kind: &str, l: &mut crate::engine::LineageState, now: &crate::engine::LineageState) {
    match kind {
        "party" => {
            l.party = now.party.clone();
            l.kennel = now.kennel.clone();
            l.eggs = now.eggs.clone();
            l.bred = now.bred.clone();
            l.lost = now.lost.clone();
            l.next_comp_id = now.next_comp_id;
        }
        "kit" => {
            l.vault = now.vault.clone();
            l.supplies = now.supplies.clone();
            l.last_supplies = now.last_supplies.clone();
            l.last_supply_origins = now.last_supply_origins.clone();
            l.forge = now.forge.clone();
            l.kit = now.kit.clone();
            l.insured = now.insured.clone();
            l.keep_pref = now.keep_pref.clone();
            l.vault_pref = now.vault_pref.clone();
            l.unlocks = now.unlocks.clone();
            l.restock_off = now.restock_off;
            l.repeat_quote = now.repeat_quote.clone();
            l.repeat_short = now.repeat_short.clone();
            l.last_wasted = now.last_wasted.clone();
            l.theft_skip = now.theft_skip.clone();
            l.night_theft_rebought = now.night_theft_rebought;
            l.next_vault_id = now.next_vault_id;
        }
        "purse" => {
            l.gold = now.gold;
            l.gold_carry = now.gold_carry;
        }
        "start" => {
            l.start = now.start;
            l.waystones = now.waystones.clone();
            l.lane_stones = now.lane_stones.clone();
            l.night_passes = now.night_passes.clone();
            l.night_short = now.night_short;
        }
        "facts" => {
            l.facts = now.facts.clone();
            l.flavours = now.flavours.clone();
            l.kill_counts = now.kill_counts.clone();
            l.kills = now.kills.clone();
            l.trophies = now.trophies.clone();
            l.grudges = now.grudges.clone();
            l.bones = now.bones.clone();
        }
        _ => *l = now.clone(),
    }
}

/// Cut 28 §2: the words of a state part (≤ 3): `party −2 jackals`, `forge · pack`, `purse $640→$1874`,
/// `start D5→D1`, `learned 3`, `new heir`.
fn part_text(kind: &str, was: &crate::engine::LineageState, now: &crate::engine::LineageState) -> String {
    match kind {
        "party" => {
            let kinds = |p: &[crate::wire::Companion]| -> Vec<String> {
                let mut v: Vec<String> = p.iter().map(|c| c.kind.clone()).collect();
                v.sort();
                v
            };
            let (a, b) = (kinds(&was.party), kinds(&now.party));
            let mut lost = a.clone();
            for k in &b {
                if let Some(i) = lost.iter().position(|x| x == k) {
                    lost.remove(i);
                }
            }
            let mut got = b.clone();
            for k in &a {
                if let Some(i) = got.iter().position(|x| x == k) {
                    got.remove(i);
                }
            }
            let name = |v: &[String]| -> String {
                let mut u = v.to_vec();
                u.dedup();
                if u.len() == 1 {
                    let k = crate::engine::kind_title(&u[0]).to_lowercase();
                    if v.len() == 1 { format!(" {k}") } else { format!(" {k}s") }
                } else {
                    String::new()
                }
            };
            match (lost.len(), got.len()) {
                (l, 0) if l > 0 => format!("party −{l}{}", name(&lost)),
                (0, g) if g > 0 => format!("party +{g}{}", name(&got)),
                (0, 0) => "party".into(),
                _ => format!("party {:+}", b.len() as i32 - a.len() as i32),
            }
        }
        "kit" => {
            let mut w: Vec<&str> = Vec::new();
            if was.kit != now.kit || was.forge != now.forge {
                w.push("forge");
            }
            if was.vault != now.vault {
                w.push("vault");
            }
            if was.supplies != now.supplies || was.last_supplies != now.last_supplies || was.last_supply_origins != now.last_supply_origins {
                w.push("pack");
            }
            if was.unlocks != now.unlocks {
                w.push("unlocks");
            }
            if w.is_empty() {
                w.push("kit");
            }
            w.truncate(2);
            w.join(" · ")
        }
        "purse" => format!("purse ${}→${}", was.gold, now.gold),
        "start" => {
            if was.start != now.start {
                format!("start D{}→D{}", was.start.max(1), now.start.max(1))
            } else {
                "waystones".into()
            }
        }
        "facts" => {
            let n = now.facts.difference(&was.facts).count();
            if n > 0 {
                format!("learned {n}")
            } else {
                "bestiary".into()
            }
        }
        _ => {
            if was.heir != now.heir {
                "new heir".into()
            } else if crate::traits::key(was) != crate::traits::key(now) {
                // Cut 30 §4: a trait's move is the heir's (`heir wrathful · frail`)
                let c = crate::traits::chip(now);
                if c.is_empty() { "heir neutral".into() } else { format!("heir {c}") }
            } else if was.trait_ != now.trait_ || was.class != now.class {
                format!("heir {}", now.trait_.name())
            } else if was.class_level() != now.class_level() {
                format!("level {}", now.class_level())
            } else {
                "the floors".into()
            }
        }
    }
}

/// The headline of a paired move: its largest |Δ| over the ends and the shaft.
pub fn headline(v: &crate::wire::ForecastVs) -> f64 {
    let ends = [v.bank.delta, v.death.delta, v.return_.delta, v.stall.delta].into_iter().map(f64::abs).fold(0.0, f64::max);
    v.depths.iter().map(|d| d.delta.abs()).fold(ends, f64::max)
}

/// Cut 28 §2 (AV: `death 14 → 36%` after a gas death, the scene saying `R2 now → dies` — both pets
/// had died): the camp's move against the set sent, split into what the state did since the send
/// and what the edit did. On one set of seeds (the camp's, at today's `known_to`) the sent set is
/// played on the lineage as it was at the send (`Game::sent_state`), then with each state part
/// moved to today's in turn (`STATE_PARTS`, only those that changed), then with the active set's
/// route, then as the active set: each step's paired move against the one before is a part, so the
/// parts sum to the whole exactly (the whole is the active set on today's lineage less the sent set
/// on the lineage at the send). `None` when no send was recorded.
pub fn forecast_move(game: &Game, prev: &RuleSet) -> Option<crate::wire::ForecastMove> {
    let sent = game.sent_state.as_ref()?;
    let now = &game.lineage;
    let rules = now.rules().clone();
    let sims = camp_sims(game, &rules);
    let a = camp_panel(game, &rules, sims);
    let n = a.len() as u32;
    let tag = forecast_tag(game, now.best_depth + 1);
    // today's lineage, the sent set (`forecast_vs`'s base), run on to the active panel's count
    let mut today_prev = camp_panel(game, prev, sims);
    if today_prev.len() < a.len() {
        today_prev = simulate_budget_from(game, prev, n, tag, u32::MAX, u64::MAX, today_prev);
    }
    let today_prev: Vec<SimResult> = today_prev.into_iter().take(a.len()).collect();
    // the chain from the send's lineage to today's (the saved sets are today's throughout: the rules
    // are the parts `route` and `rows`, never the state)
    let mut was = sent.lineage.clone();
    was.sets = now.sets.clone();
    was.active_set = now.active_set;
    let mut g = game.sim_clone();
    g.lineage = was.clone();
    g.loadout = sent.loadout.clone();
    let panel_of = |g: &Game, set: &RuleSet| -> Vec<SimResult> {
        let key = format!("move:{}:{n}:{tag}:{}", lineage_key(g), played_key(g, set));
        if let Some(v) = game.panel_cache.borrow().get(&key) {
            return v.clone();
        }
        let v = simulate_budget_from(g, set, n, tag, u32::MAX, u64::MAX, Vec::new());
        panel_insert(game, key, v.clone());
        v
    };
    let mut parts: Vec<crate::wire::MovePart> = Vec::new();
    let mut last = if lineage_key(&g) == lineage_key(game) { today_prev.clone() } else { panel_of(&g, prev) };
    let base = last.clone();
    let refined = sims > FORECAST_SIMS;
    for kind in STATE_PARTS {
        // (a part that moves nothing a sim reads — the lineage's key unchanged — is taken silently: no line)
        let key0 = lineage_key(&g);
        let before = g.lineage.clone();
        take_part(kind, &mut g.lineage, now);
        if kind == "kit" || kind == "heir" {
            g.loadout = game.loadout.clone();
        }
        if lineage_key(&g) == key0 {
            continue;
        }
        let next = if lineage_key(&g) == lineage_key(game) { today_prev.clone() } else { panel_of(&g, prev) };
        parts.push(crate::wire::MovePart { kind: kind.into(), text: part_text(kind, &before, now), move_: vs_between(game, &next, &last, refined) });
        last = next;
    }
    // (any state the parts did not name — the lineage is today's from here on)
    if lineage_key(&g) != lineage_key(game) {
        let before = g.lineage.clone();
        g.lineage = now.clone();
        g.loadout = game.loadout.clone();
        let next = today_prev.clone();
        parts.push(crate::wire::MovePart { kind: "heir".into(), text: part_text("heir", &before, now), move_: vs_between(game, &next, &last, refined) });
        last = next;
    }
    let state = !parts.is_empty();
    // the route, then the rows
    let rows = rules_key(&RuleSet { rows: rules.rows.clone(), name: None, route: Vec::new() }) != rules_key(&RuleSet { rows: prev.rows.clone(), name: None, route: Vec::new() });
    if rules.route != prev.route {
        let routed = prev.clone().with_route(rules.route());
        let next = if rows { panel_of(game, &routed) } else { a.clone() };
        parts.push(crate::wire::MovePart { kind: "route".into(), text: "route".into(), move_: vs_between(game, &next, &last, refined) });
        last = next;
    }
    if rows {
        // Cut 30 §2: the packages' rows first (a swap, a level, a drill: today's package rows under the
        // sent set's own rows), then the pen's — the parts still sum to the whole
        let pkg_of = |s: &RuleSet| rules_key(&RuleSet { rows: s.rows.iter().filter(|r| r.is_pkg()).cloned().collect(), name: None, route: Vec::new() });
        if pkg_of(&rules) != pkg_of(prev) {
            let mut mid = prev.clone().with_route(rules.route());
            mid.rows.retain(|r| !r.is_pkg());
            mid.rows.extend(rules.rows.iter().filter(|r| r.is_pkg()).cloned());
            let pen_same = rules_key(&RuleSet { rows: mid.rows.clone(), name: None, route: Vec::new() }) == rules_key(&RuleSet { rows: rules.rows.clone(), name: None, route: Vec::new() });
            let next = if pen_same { a.clone() } else { panel_of(game, &mid) };
            let name = rules.rows.iter().find(|r| r.origin.as_deref().is_some_and(|o| o.starts_with("stance:"))).and_then(crate::packages::row_label).unwrap_or_else(|| "package".into());
            parts.push(crate::wire::MovePart { kind: "package".into(), text: name, move_: vs_between(game, &next, &last, refined) });
            last = next;
        }
        if last != a {
            parts.push(crate::wire::MovePart { kind: "rows".into(), text: "rows".into(), move_: vs_between(game, &a, &last, refined) });
        }
    }
    let whole = vs_between(game, &a, &base, refined);
    let lead = parts.iter().max_by(|x, y| headline(&x.move_).total_cmp(&headline(&y.move_))).map(|p| p.kind.clone()).unwrap_or_else(|| "rows".into());
    Some(crate::wire::ForecastMove { whole, parts, lead, rows, state, sims: n, refined })
}


/// Cut 22 §3: the mean of `f(a_i) − f(b_i)` over paired sims and its 95 % half-width
/// (`1.96 · s / √n`, `s` the sample deviation of the differences; 0 when every seed agrees).
pub fn paired(a: &[SimResult], b: &[SimResult], f: impl Fn(&SimResult) -> f64) -> crate::wire::VsMove {
    let n = a.len().min(b.len());
    if n == 0 {
        return crate::wire::VsMove::default();
    }
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| f(x) - f(y)).collect();
    let mean = d.iter().sum::<f64>() / n as f64;
    let var = if n > 1 { d.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1) as f64 } else { 0.0 };
    let base = b[..n].iter().map(&f).sum::<f64>() / n as f64;
    crate::wire::VsMove { delta: mean, pm: 1.96 * (var / n as f64).sqrt(), base }
}

/// Cut 10 §2: the `try` of a forecast row — reaching `depth` means passing the boss on the
/// floor above it; when that boss's counter fact is known and no row of `rules` carries the
/// counter's verb, the row names it (`D9 0% · warlord · try: attack boss`). Position is the
/// point: the client inserts it at the top, and the gate measures that placement.
pub fn try_row(game: &Game, rules: &RuleSet, depth: u32) -> Option<ForecastTry> {
    let kind = rules.route().boss(depth.checked_sub(1)?)?;
    let row = crate::facts::boss_counter_row(&game.lineage.facts, kind)?;
    if crate::trace::has_counter_verb(rules, &row) {
        return None;
    }
    Some(ForecastTry { boss: kind.to_string(), text: crate::facts::counter_text(&row), row, met: depth - 1 })
}

/// Cut 18 §3: a forecast row's reach at or under this is a wall when a boss seals the stairs
/// of the floor above.
pub const WALL_REACH: f64 = 0.05;

/// Cut 18 §3: the wall at `depth` — reach falls to ≤ `WALL_REACH` there from over it on the
/// floor above, and that floor is a boss's (`descent::boss_for`: a living boss seals its
/// stairs, `ai::stairs_sealed`): the boss's kind. Rater Z: "`D9 0%` for every rule set, with
/// no reason given, until I met the Goblin Warlord".
pub fn wall_at(depth: u32, reach: f64, reach_above: f64) -> Option<String> {
    wall_on(crate::descent::Route::BASE, depth, reach, reach_above)
}

/// `wall_at` on a route (Cut 26 §1).
pub fn wall_on(route: crate::descent::Route, depth: u32, reach: f64, reach_above: f64) -> Option<String> {
    let kind = route.boss(depth.checked_sub(1)?)?;
    (reach <= WALL_REACH + 1e-9 && reach_above > WALL_REACH + 1e-9).then(|| kind.to_string())
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
    let stall = results.iter().filter(|r| r.timed_out).count() as f64 / results.len().max(1) as f64;
    let mut cache = game.forecast_cache.borrow_mut();
    if cache.len() + 1 >= FORECAST_CACHE_MAX {
        cache.clear();
    }
    // QA on 92eb880: the same sims' stall share beside the reach (`stall_cached`), so a
    // card's best place is never one that stalls.
    cache.insert(format!("stall:{key}"), (stall, results.len() as u32));
    cache.insert(key, (v, results.len() as u32));
    (v, results.len() as u32)
}

/// The stall share (sims that timed out or shuffled a floor, 0..1) of the sims that measured
/// `reach_counted` with these arguments, if this game ran them.
pub fn stall_cached(game: &Game, rules: &RuleSet, depth: u32, sims: u32, tag: u64, budget: u64) -> Option<f64> {
    game.forecast_cache.borrow().get(&format!("stall:{}", reach_key(game, rules, depth, sims, tag, budget))).map(|v| v.0)
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
    format!("{}:{depth}:{sims}:{tag}:{budget}:{}", lineage_key(game), rules_key(rules))
}

/// A fingerprint of what a sim starts from: the lineage fields a fresh run reads (facts,
/// unlocks, class and level, vault and loadout, party, supplies, gold, forge, grudges, bones,
/// insurance, keep preference, trait, heir, variant, hunter, freshness, the other sets, trophies) — not marks, renown or the rest
/// clock, so a purchase or a rank does not spill the cache.
pub fn lineage_key(game: &Game) -> u64 {
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
    feed(&crate::traits::key(l));
    feed(l.class.name());
    feed(&l.class_level().to_string());
    if let Some(hero) = crate::legacy::current(l).filter(|hero| !hero.upgrades.is_empty()) { feed(&format!("legacy {:?}", hero.upgrades)); }
    feed(&format!("{:?}", l.facts));
    feed(&format!("{:?}", l.unlocks));
    // Cut 30 §1: the scars the sends carry (a boss met is weaker until he falls)
    feed(&format!("{:?}", crate::descent::BOSS_DEPTHS.iter().map(|(k, _)| l.pkg.scar(k, &l.kills)).collect::<Vec<_>>()));
    feed(&serde_json::to_string(&l.vault).unwrap_or_default());
    // (the send sorts and de-duplicates the loadout: `[5, 3]` and `[3, 5]` pack the same)
    let mut loadout = game.loadout.clone();
    loadout.sort();
    loadout.dedup();
    feed(&format!("{loadout:?}"));
    feed(&serde_json::to_string(&l.party).unwrap_or_default());
    feed(&serde_json::to_string(&l.supplies).unwrap_or_default());
    // Automatic supplies read the effective package and pen as well as the compiled rows.
    feed(&format!("{:?}", crate::packages::quartermaster(l)));
    feed(&format!("{:?}", crate::packages::pack_kinds(l)));
    feed(&format!("{:?}", l.last_supplies));
    if !l.last_supply_origins.is_empty() { feed(&format!("{:?}", l.last_supply_origins)); }
    feed(&l.gold.to_string());
    feed(&serde_json::to_string(&l.forge).unwrap_or_default());
    feed(&serde_json::to_string(&l.grudges).unwrap_or_default());
    feed(&serde_json::to_string(&l.bones).unwrap_or_default());
    feed(&format!("{:?}", l.insured));
    feed(&l.keep_pref);
    // What a floor generated in a sim reads besides (`CheckpointLineage`).
    feed(&l.vault_pref);
    feed(&serde_json::to_string(&l.lost).unwrap_or_default());
    feed(&l.variant);
    if let Some(p) = &l.endgame { if p.tier > 0 { feed(&format!("difficulty {}", p.tier)); } }
    feed(&serde_json::to_string(&l.hunter).unwrap_or_default());
    feed(&format!("{:?}", l.kill_counts));
    feed(&l.ended.to_string());
    // QA on 92eb880: what else a sim's run reads — the floors' freshness (`picked`, Cut 16
    // §1), the other sets (a shrine lends one of their rows), the trophies (the turn's
    // context) — so a memoised panel is never a stale one.
    feed(&format!("{:?}", l.picked));
    // (the active set is the sims' own rules, keyed apart: a patch measured on the death screen
    // and the camp that applies it read one panel)
    let active = l.active_set.min(l.sets.len().saturating_sub(1));
    for (i, set) in l.sets.iter().enumerate().filter(|(i, _)| *i != active) {
        feed(&format!("{i}:{}", rules_key(set)));
    }
    feed(&format!("{:?}", l.trophies));
    // Cut 21 §1: where the sends start (and whether that waystone is lit).
    feed(&format!("start {} {:?}", l.start, l.waystones));
    if !l.lane_stones.is_empty() {
        feed(&format!("lanes {:?}", l.lane_stones));
    }
    // QA on a946e04: the night's waystone passes (a sim's send from one pays no toll).
    feed(&format!("passes {:?}", l.night_passes));
    // Cut 23 §1: the forge's steps (the heir's starting kit).
    feed(&format!("kit {:?}", l.kit));
    // Cut 28 §1: the sworn oath (a panel's sims say whether each kept it).
    if let Some(o) = crate::oath::sworn(l) {
        feed(&format!("oath {} {} {:?} {:?}", o.kind, o.depth, o.boss, o.seen));
    }
    h
}
