//! Forecast: N deterministic sims on fresh floor seeds, reported only as deep as the hero has facts.
use crate::engine::{ExitTier, Game};
use crate::rng::splitmix;
use crate::rules::RuleSet;
use crate::wire::{Forecast, ForecastCause, ForecastDepth, ForecastEnds, ForecastTry};
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

#[derive(Clone, Debug, PartialEq)]
pub struct SimResult {
    pub max_depth: u32,
    pub tier: ExitTier,
    pub cause: Option<String>,
    /// Cut 12 §3: the gold this send brings home — the loot by the exit's share (`ExitTier::pct`;
    /// a run that timed out keeps nothing, as the exit's own maths has it).
    pub loot_kept: i32,
    /// The send hit the turn cap or stalled: a return by nothing in the rules (its own share).
    pub timed_out: bool,
    /// Cut 24 §4: the ticks the sim ran (its share of the panel's budget) — the refine pass
    /// continues the first pass's panel from its last sim (`camp_panel`).
    pub ticks: u32,
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
    let from = (prefix.len() as u32).min(sims);
    let spent0: u64 = prefix.iter().map(|r| r.ticks as u64).sum();
    let mut ran: Vec<(u32, SimResult)> = prefix.into_iter().take(sims as usize).map(|r| (r.ticks, r)).collect();
    // (the prefix already ends the panel: nothing more to run)
    let done = from >= MIN_SIMS && spent0 >= budget;
    // Every sim is a pure function of (lineage, rules, tag, i); the tick budget only decides
    // how many of them count, in order. Natively they run on all cores and the budget is
    // applied to the ordered results afterwards, so the answer is the sequential one exactly.
    if done {
    } else if parallel_sims() && sims > from + 1 {
        ran.extend(par_map(game, (from..sims).collect(), |base, &i| simulate_one(base, rules, tag, stop_depth, i)));
    } else {
        let mut spent: u64 = spent0;
        for i in from..sims {
            if i >= MIN_SIMS && spent >= budget {
                break;
            }
            let r = simulate_one(game, rules, tag, stop_depth, i);
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

/// One fresh expedition (`i`-th of the panel) under `rules`, to `stop_depth` or its end:
/// (ticks spent, result).
fn simulate_one(game: &Game, rules: &RuleSet, tag: u64, stop_depth: u32, i: u32) -> (u32, SimResult) {
    let mut g = game.sim_clone();
    g.run = None;
    g.pending_exit = None;
    g.history.clear();
    let _ = g.set_rules(rules.clone());
    let seed = splitmix(game.lineage.seed ^ splitmix(tag ^ (i as u64 + 1).wrapping_mul(0xA24B_AED4_963E_E407)));
    // Cut 24 §2: a named foe rests two runs in three (`LineageState::named_resting`) — the panel
    // plays the sends from here, so its sims turn through the three.
    g.lineage.next_run_id += i % 3;
    g.start_run(Some(seed));
    // Cut 22 §3: the panel's sims share their floors seed by seed (`Run.floor_streams`).
    g.run.as_mut().unwrap().floor_streams = true;
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none() && r.max_depth < stop_depth) && n < SIM_MAX_TICKS {
        g.tick();
        n += 1;
    }
    let run = g.run.as_ref().unwrap();
    let tier = run.over.unwrap_or(ExitTier::Return);
    let loot_kept = run.loot.max(0) * run.yield_pct(tier) / 100;
    (n, SimResult { max_depth: run.max_depth, tier, cause: run.death_cause.clone(), loot_kept, timed_out: run.timed_out, ticks: n })
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
    let threads = max_threads().min(jobs.len());
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
    PARALLEL_SIMS.load(std::sync::atomic::Ordering::Relaxed) && !IN_WORKER.with(|w| w.get())
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
        let rules = game.lineage.rules();
        let keep = [panel_key(game, rules, FORECAST_SIMS), panel_key(game, rules, REFINE_SIMS)];
        game.panel_cache.borrow_mut().retain(|k, _| keep.contains(k));
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
    let known_to = game.lineage.best_depth + 1;
    let tag = forecast_tag(game, known_to);
    let budget = panel_budget(sims);
    let key = panel_key(game, rules, sims);
    if let Some(v) = game.panel_cache.borrow().get(&key) {
        return v.clone();
    }
    // Cut 24 §4 (AK: "edits wait 3–7 s to settle"): the refine's first sims are the first
    // pass's own — continued from its panel when it is cached, not run again.
    let prefix = if sims > FORECAST_SIMS { game.panel_cache.borrow().get(&panel_key(game, rules, FORECAST_SIMS)).cloned().unwrap_or_default() } else { Vec::new() };
    let ended = simulate_budget_from(game, rules, sims, tag, u32::MAX, budget, prefix);
    if sims >= REFINE_SIMS {
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
            ForecastDepth { depth: d, reach, pm: Some(half_width(reach, ended.len())), try_, wall, bounty: game.lineage.bounty == Some(d), boss: route.boss(d).map(str::to_string), biome: (!route.is_base()).then(|| route.biome(d).name().to_string()) }
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
    let ends = (!ended.is_empty()).then(|| ForecastEnds { bank: share(ExitTier::Bank), return_: share(ExitTier::Return), death, stall, gold, pm: half_width(death, ended.len()) });
    let n_sims = ended.len() as u32;
    Forecast { depths, causes, known_to, ends, refined: sims > FORECAST_SIMS, shadowed_by: game.lineage.shadowed_by(rules), start, sims: n_sims, low: low_pct(n_sims) }
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
        out.push(crate::wire::StartOption {
            start: d,
            current: d == current,
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
    if !route.fork_open(fork) || !crate::descent::OPEN_FORKS.contains(&fork) {
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
    let rules = game.lineage.rules().clone();
    let sims = camp_sims(game, &rules);
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
        depths,
        bank: paired(a, b, tier(ExitTier::Bank)),
        death: paired(a, b, |r| ind(r.tier == ExitTier::Death)),
        return_: paired(a, b, tier(ExitTier::Return)),
        gold: paired(a, b, |r| r.loot_kept as f64),
        stall: paired(a, b, |r| ind(r.timed_out)),
        sims: n as u32,
        refined: sims > FORECAST_SIMS,
    }
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
    feed(l.class.name());
    feed(&l.class_level().to_string());
    feed(&format!("{:?}", l.facts));
    feed(&format!("{:?}", l.unlocks));
    feed(&serde_json::to_string(&l.vault).unwrap_or_default());
    // (the send sorts and de-duplicates the loadout: `[5, 3]` and `[3, 5]` pack the same)
    let mut loadout = game.loadout.clone();
    loadout.sort();
    loadout.dedup();
    feed(&format!("{loadout:?}"));
    feed(&serde_json::to_string(&l.party).unwrap_or_default());
    feed(&serde_json::to_string(&l.supplies).unwrap_or_default());
    feed(&format!("{:?}", l.last_supplies));
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
    h
}
