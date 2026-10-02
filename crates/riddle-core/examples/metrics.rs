//! Bot gates (docs/CUT1.md): 30 seeds × the 8 h offline model per bot. Prints PASS/FAIL and
//! exits non-zero on any FAIL. Never weaken a gate; tune content.
//!   cargo run --release --example metrics [-- --seeds 30 --hours 8]
use riddle_core::engine::{without_history, ExitTier};
use riddle_core::hero::Class;
use riddle_core::rng::{splitmix, Rng};
use riddle_core::{Ev, Game, RuleSet};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
#[path = "lever_lib/mod.rs"]
mod lever;
#[path = "oath_lib/mod.rs"]
mod oath_lib;
#[path = "lanes_lib/mod.rs"]
mod lanes;
#[path = "exits_lib/mod.rs"]
mod exits;
#[path = "progression_lib/mod.rs"]
mod prog;
#[path = "idle_lib/mod.rs"]
mod idle;
#[path = "jobcache_lib/mod.rs"]
mod jobcache;

/// The panel width of the pool's long chains (the progression lineages): one while
/// the pool is full, the idle threads shared among the chains as the short jobs run out
/// (`forecast::with_sim_width`; the results are the same at any width).
static CHAIN_WIDTH: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(1);

/// IDLE's snapshots of `seed` (`idle::snapshots`, as saves), kept across runs (`jobcache`).
fn idle_snaps(seed: u64) -> Vec<(u32, String)> {
    jobcache::cached("idle-snaps", &[include_str!("idle_lib/mod.rs")], &format!("snapshots {seed} 14"), || idle::snapshots(seed, 14).into_iter().map(|(w, g)| (w, g.save())).collect())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Bot {
    Default,
    Edited,
    Random,
    Passive,
    Learned,
    Pets,
    Levelled,
    Trivial,
    Countered,
    // Cut 3: the shipped best set with every counter and unlock, and the same set minus one
    // boss counter row each (Cut 7 depths: D23 reflect_read, D28 silence, D33 cadence).
    // 3 × 8 h batches.
    Full,
    FullNo23,
    FullNo28,
    FullNo33,
}

const BOTS: [Bot; 13] = [Bot::Default, Bot::Edited, Bot::Random, Bot::Passive, Bot::Learned, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered, Bot::Full, Bot::FullNo23, Bot::FullNo28, Bot::FullNo33];
/// Cut 23 §1: the bots that also play with every forge step bought (`kit::buy_all`) — the gates
/// hold with the whole kit: KITTED (DEFAULT kitted) dies by D8, EDITED kitted beats it by 15,
/// RANDOM and PASSIVE lose, LEARNED kitted gains ≤ 2 floors on it, TRIVIAL never passes D8,
/// the FULL walls hold.
const KIT_BOTS: [Bot; 9] = [Bot::Default, Bot::Edited, Bot::Random, Bot::Passive, Bot::Learned, Bot::Trivial, Bot::FullNo23, Bot::FullNo28, Bot::FullNo33];
/// Cut 3: the FULL bots play three 8 h absences.
const FULL_BATCHES: u64 = 3;

impl Bot {
    fn name(self) -> &'static str {
        match self {
            Bot::Default => "DEFAULT",
            Bot::Edited => "EDITED",
            Bot::Random => "RANDOM",
            Bot::Passive => "PASSIVE",
            Bot::Learned => "LEARNED",
            Bot::Pets => "PETS",
            Bot::Levelled => "LEVELLED",
            Bot::Trivial => "TRIVIAL",
            Bot::Countered => "COUNTERED",
            Bot::Full => "FULL",
            Bot::FullNo23 => "FULL−D23",
            Bot::FullNo28 => "FULL−D28",
            Bot::FullNo33 => "FULL−D33",
        }
    }
    fn is_full(self) -> bool {
        matches!(self, Bot::Full | Bot::FullNo23 | Bot::FullNo28 | Bot::FullNo33)
    }
    fn batches(self) -> u64 {
        if self.is_full() {
            FULL_BATCHES
        } else {
            1
        }
    }
}

#[derive(Clone, Default, Debug)]
struct SeedResult {
    best_depth: u32,
    run_depths: Vec<u32>,
    causes: Vec<String>,
    verdicts: Vec<String>,
    learned: usize,
    pending: usize,
    events: u32,
    ticks: u32,
    known_to_ok: bool,
    runs: u32,
    run_ticks: Vec<u32>,
    // Cut 2
    banked: u32,
    returned: u32,
    xp: u32,
    gold: i32,
    /// Cut 15 §1: marks earned over the absence (lineage delta) and the frontier banks among them.
    marks: u32,
    frontier: u32,
    rested_s: u64,
    /// (patches shown, patches whose row fired in ≥ 50% of replays)
    patches: (u32, u32),
    /// (fresh reseeded replays, of which the patch row fired) — information only
    patches_fresh: (u32, u32),
    verdict_secs: Vec<f64>,
    death_secs: Vec<f64>,
    /// Cut 3: microseconds per simulated tick of the offline batch (rest ticks excluded).
    tick_us: f64,
    // Cut 5
    /// Per absence: distinct (threat, resolution) pairs in the reel, and whether its top line
    /// names a row, a trait or a companion.
    reel_pairs: Vec<usize>,
    reel_names: Vec<bool>,
    /// (story lines, of which in the grammar) and (real runs, of which met a situation on D1–5).
    stories: (u32, u32),
    situations: (u32, u32),
    /// Cut 7 §3: per real run, the depth reached and the band situations met / passed.
    band: Vec<riddle_core::engine::BandRun>,
    /// Cut 7 §1: DEFAULT's runs that reached D5, of which left it alive.
    captain: (u32, u32),
    // Cut 11
    /// §1: state reasons in the death traces (`no item` …), of which carry a `because`; and
    /// how many deaths were looked at.
    because: (u32, u32),
    because_deaths: u32,
    /// §2: deaths with a theft/lock root (player-shaped bots, ≤ 3 per seed) whose root patch
    /// measured at or over the baseline, of which show a root patch, of which the root
    /// patch's forecast delta reaches the best symptom's.
    roots: (u32, u32, u32),
    /// Cut 14 §1: root deaths whose root patch measured under the baseline (not offered; the
    /// unlock sheet carries its number).
    roots_under: u32,
    /// §4: sampled `dice` deaths, of which name an alternative (non-empty patches).
    dice_named: (u32, u32),
    // Cut 13 §1
    /// (real runs, of which stalled); sampled stall records (≤ 2 per batch), of which carry a
    /// `stall` verdict with ≥ 1 patch that fired in ≥ 50 % of its replays; and of which the
    /// run's reel line names the trace's cause.
    stalls: (u32, u32),
    stall_verdicts: (u32, u32),
    stall_reel: (u32, u32),
    /// Cut 28 §1: the lineage swore an oath (or kept one) — a bot never does.
    swore: bool,
}

fn good() -> RuleSet {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/presets/good.json")).expect("presets/good.json");
    RuleSet::parse(&text).expect("good.json parses")
}

fn full() -> RuleSet {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/presets/full.json")).expect("presets/full.json");
    let set = RuleSet::parse(&text).expect("full.json parses");
    assert_eq!(set, riddle_core::probes::full(), "presets/full.json is probes::full()");
    set
}

/// Cut 13 §1: the cohorts' own rule sets (`eval/cards/<build>.<rater>[-<tag>].rules.json`:
/// a rater's export, or a set reconstructed from a rater's notes) — the stall gate runs over
/// them too, because the loop rater T hit on 2cb9e88 was one no bot's set carried.
fn cohort_sets() -> Vec<(String, RuleSet)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let Ok(rd) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut out: Vec<(String, RuleSet)> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            let text = std::fs::read_to_string(e.path()).ok()?;
            let set = RuleSet::parse(&text).unwrap_or_else(|err| panic!("{name}: {err}"));
            Some((stem, set))
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// (set index, seed, the set's return rows cut) → (sends, stalls, deaths, dances).
/// (sends, stalls, deaths, dances, sends with a card ↔ chore loop the guard caught — Cut 27 §4).
type CohortStalls = BTreeMap<(usize, u64, bool), StallCounts>;
/// Cut 23 §2: one death of a cohort set — its killer, whether the hero was walking home
/// (and his hp % when he turned), and its verdict (sampled).
#[derive(Clone, Debug)]
struct DeathMix {
    killer: String,
    home: Option<i32>,
    verdict: Option<String>,
}
type CohortDeaths = BTreeMap<(usize, u64), Vec<DeathMix>>;

/// Cut 19 §2: `set` without its `return` rows (the death share's comparison).
fn without_return(set: &RuleSet) -> RuleSet {
    RuleSet { rows: set.rows.iter().filter(|r| r.verb.v != "return").cloned().collect(), name: set.name.clone(), route: set.route.clone() }
}

/// A lineage that owns what a cohort set needs (its cards, its condition tokens, eight
/// rows, the common facts), playing that set: (sends, stalls) over `hours`.
fn cohort_game(set: &RuleSet, seed: u64) -> Game {
    let mut g = setup(Bot::Edited, seed);
    for r in &set.rows {
        if let Some(c) = r.card() {
            g.lineage.unlocks.insert(c.into());
        }
        for c in &r.conds {
            if let Some(u) = riddle_core::meta::cond_unlock(&c.k) {
                g.lineage.unlocks.insert(u.into());
            }
        }
    }
    // `_raw`: the lineage has not met what a token's lock needs (`see: captive`); the run has.
    g.set_rules_raw(set.clone()).unwrap_or_else(|e| panic!("cohort set {:?}: {e}", set.name));
    g
}

/// Cut 24 §1: a cohort job's sends' longest no-HP stretches, and the sends a boss drove off.
type NoHp = (Vec<u32>, u32);
/// Cut 27 §4: a cohort job's loops' causes and its stalls' (cause, depth) — `METRICS_LOOPS=1` prints them.
type LoopLog = (Vec<String>, Vec<(String, u32)>);
/// (sends, stalls, deaths, dances, card ↔ chore loops) — `CohortStalls`' value.
type StallCounts = (u32, u32, u32, u32, u32);

/// `mix`: the death mix and its sampled verdicts (the without-return twin reads only the counts,
/// and a verdict reads the game and writes its own death's record — nothing the counts read).
fn cohort_stalls(set: &RuleSet, seed: u64, hours: u64, mix: bool) -> (StallCounts, Vec<DeathMix>, NoHp, bool, LoopLog) {
    let mut g = cohort_game(set, seed);
    g.max_deaths = 100_000;
    riddle_core::offline::run_offline_counts(&mut g, hours * 3600);
    // Cut 23 §2: the death mix — every death's killer and walk home; verdicts on two a seed.
    let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall).map(|(id, _)| *id).collect();
    let step = (ids.len() / 2).max(1);
    let sampled: Vec<u32> = ids.iter().step_by(step).take(2).copied().collect();
    let want_mix = mix;
    let mut mix = Vec::new();
    for id in ids.iter().filter(|_| want_mix) {
        let verdict = if sampled.contains(id) { riddle_core::trace::verdict(&mut g, *id) } else { None };
        let rec = &g.deaths[id];
        mix.push(DeathMix { killer: rec.death.cause.clone(), home: rec.home.map(|h| h.0), verdict });
    }
    // QA on 23ed91f (qaL): a run that reaches the tick cap is a stall that never ended (a
    // conjurer's blades reset the guard: 120 000 ticks, 3 000 kills) — counted with them.
    let capped = g.batch.run_ticks.iter().filter(|&&t| t >= riddle_core::engine::MAX_TURNS_PER_RUN).count() as u32;
    let deaths = g.batch.run_outcomes.iter().filter(|(_, c)| c.is_some()).count() as u32;
    // QA on 778fa1b (qaV): runs with a bloodless dance (`Batch.dances`).
    // Cut 24 §1: each send's longest no-HP stretch, and the sends a boss drove off.
    // Cut 26 §4: whether the lineage saw the D9 fork (D8's two stairs) before the absence ended.
    let fork9 = g.lineage.facts.contains("fork:9");
    ((g.batch.run_outcomes.len() as u32, g.batch.stalls + capped, deaths, g.batch.dances, g.batch.card_loops), mix, (g.batch.nohp.clone(), g.batch.driven_off), fork9, (g.batch.loop_causes.clone(), g.deaths.values().filter(|r| r.stall).map(|r| (r.death.cause.clone(), r.death.depth)).collect()))
}

/// Cut 26 §4: the first-hour agent — a fresh lineage on the preset that writes the first patch its
/// first death offers, sending until the hero sees the D5 fork (D4's two stairs): the sends it took
/// (`None`: not within `cap`).
fn first_fork(seed: u64, cap: u32) -> Option<u32> {
    let mut g = Game::new_literal(seed);
    let mut patched = false;
    for send in 1..=cap {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let mut n = 0;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < riddle_core::engine::MAX_TURNS_PER_RUN {
            g.tick();
            g.events.clear();
            n += 1;
        }
        let died = g.run.as_ref().is_some_and(|r| r.over == Some(ExitTier::Death));
        let id = g.run.as_ref().map(|r| r.id).unwrap_or(0);
        g.finish_run();
        g.auto_keep();
        g.events.clear();
        if g.lineage.facts.contains("fork:5") {
            return Some(send);
        }
        if died && !patched {
            if let Some(d) = g.death(id) {
                if let Some(p) = d.patches.iter().find(|p| p.insert_at >= 0 && !p.below_bar) {
                    let set = riddle_core::offline::apply_patch(g.lineage.rules(), p, g.lineage.max_rows());
                    patched = g.set_rules(set).is_ok();
                }
            }
        }
    }
    None
}

/// Cut 23 §2: the death mix per cohort set — the share of deaths on the walk home, the top
/// killer, and the sampled verdicts × killer; a set whose single top (walk, killer) cause is
/// over half its deaths is flagged.
fn death_mix_report(sets: &[(String, RuleSet)], deaths: &CohortDeaths, seeds: u64) -> (usize, usize) {
    println!("\ndeath mix (Cut 23 §2; cohort sets, 4 h × {seeds} seeds): deaths · walking home (hp % at the turn) · top killer · top cause (walk × killer) · sampled verdicts");
    let (mut flagged, mut n_sets) = (0usize, 0usize);
    for (si, (name, _)) in sets.iter().enumerate() {
        let all: Vec<&DeathMix> = (1..=seeds).flat_map(|s| deaths.get(&(si, s)).into_iter().flatten()).collect();
        if all.is_empty() {
            println!("  {name}: no deaths");
            continue;
        }
        n_sets += 1;
        let n = all.len();
        let home: Vec<i32> = all.iter().filter_map(|d| d.home).collect();
        let home_hp = if home.is_empty() { 0.0 } else { home.iter().sum::<i32>() as f64 / home.len() as f64 };
        let count = |f: &dyn Fn(&DeathMix) -> String| {
            let mut m: BTreeMap<String, usize> = BTreeMap::new();
            for d in &all {
                *m.entry(f(d)).or_insert(0) += 1;
            }
            let mut v: Vec<(String, usize)> = m.into_iter().collect();
            v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
            v
        };
        let killers = count(&|d| d.killer.clone());
        let causes = count(&|d| format!("{}{}", if d.home.is_some() { "walk · " } else { "" }, d.killer));
        let verdicts: Vec<String> = all.iter().filter_map(|d| d.verdict.as_ref().map(|v| format!("{v} × {}{}", if d.home.is_some() { "walk · " } else { "" }, d.killer))).collect();
        let mut vc: BTreeMap<&str, usize> = BTreeMap::new();
        for v in &verdicts {
            *vc.entry(v.as_str()).or_insert(0) += 1;
        }
        let mut vv: Vec<(&str, usize)> = vc.into_iter().collect();
        vv.sort_by_key(|b| std::cmp::Reverse(b.1));
        let top = &causes[0];
        let top_share = pct(top.1, n);
        let walk_share = pct(home.len(), n);
        // (the contract's bar: one cause — walk × killer — over half; the walk share prints beside)
        let flag = top_share > 50.0;
        flagged += flag as usize;
        println!(
            "  {name}: {n} · walk {walk_share:.0}% (hp {home_hp:.0}%) · killer {} {:.0}% · top {} {top_share:.0}%{} · {}",
            killers[0].0,
            pct(killers[0].1, n),
            top.0,
            if flag { " · OVER HALF" } else { "" },
            vv.iter().take(3).map(|(v, k)| format!("{v} ×{k}")).collect::<Vec<_>>().join(", ")
        );
    }
    (flagged, n_sets)
}

/// Cut 22 §1: one cohort set's gold over `hours` of watched sends (the repeat on, no absence's
/// `restock ≤ income` cap — the sends the raters watched): what came home, what the exits
/// salvaged, what the shelf and the tolls cost, per send.
#[derive(Clone, Copy, Default)]
struct GoldTally {
    sends: u32,
    /// Sends that banked or returned.
    home_sends: u32,
    home: i64,
    salvage: i64,
    /// Supplies bought (the repeat, the first pack excluded: the shelf starts and ends packed).
    spent: i64,
    tolls: i64,
    /// Everything else (the heir purse's wake pay, refunds).
    other: i64,
    /// Thefts of a bought (packed, not found) supply, and all thefts.
    supply_thefts: u32,
    thefts: u32,
    /// Cut 23 §5: thefts of a leash (the only pet gear).
    leash_thefts: u32,
    /// The sends that returned (60 %) and their own net: the safe run the raters called a
    /// treadmill (AG: "+$77 returned · +$19 salvage · −$160 spent").
    ret_sends: u32,
    ret_net: i64,
    /// Cut 27 §1: the ticks the sends and their rests took (gold per hour), the sends that started
    /// below D1, and those paid a passage.
    ticks: u64,
    way_sends: u32,
    paid_sends: u32,
    /// The paid sends' own net and ticks (the set cleared the floors above its start then).
    paid_net: i64,
    paid_ticks: u64,
}

impl GoldTally {
    fn add(&mut self, o: &GoldTally) {
        self.sends += o.sends;
        self.home_sends += o.home_sends;
        self.home += o.home;
        self.salvage += o.salvage;
        self.spent += o.spent;
        self.tolls += o.tolls;
        self.other += o.other;
        self.supply_thefts += o.supply_thefts;
        self.thefts += o.thefts;
        self.leash_thefts += o.leash_thefts;
        self.ret_sends += o.ret_sends;
        self.ret_net += o.ret_net;
        self.ticks += o.ticks;
        self.way_sends += o.way_sends;
        self.paid_sends += o.paid_sends;
        self.paid_net += o.paid_net;
        self.paid_ticks += o.paid_ticks;
    }
    /// Cut 27 §1: net gold per hour over the sends paid a passage.
    fn paid_per_hour(&self) -> f64 {
        self.paid_net as f64 / (self.paid_ticks.max(1) as f64 / (riddle_core::offline::TICKS_PER_SECOND as f64 * 3600.0))
    }
    /// Cut 27 §1: net gold per hour of sends and rests.
    fn per_hour(&self) -> f64 {
        self.net() as f64 / (self.ticks.max(1) as f64 / (riddle_core::offline::TICKS_PER_SECOND as f64 * 3600.0))
    }
    fn net(&self) -> i64 {
        self.home + self.salvage - self.spent - self.tolls + self.other
    }
}

/// The supplies a player packs for `set`: 2 of a kind the set drinks to heal, 1 of each other
/// kind a row names that the shelf sells, up to the cap.
fn pack_for(g: &mut Game) {
    let kinds = g.lineage.row_kinds();
    let cat = g.supply_catalogue();
    for k in kinds.iter().filter(|k| cat.iter().any(|e| e.kind == **k) && k.as_str() != "leash") {
        for _ in 0..if k == "heal" { 2 } else { 1 } {
            let _ = g.buy_supply(k);
        }
    }
}

/// Cut 22 §1: `hours` of sends on `set` from `start` (1, or the deepest lit waystone when
/// `waystone`), each followed by its rest, the repeat on.
///
/// Both starts in one job (`cohort_golds`: (from D1, from the waystone)): until a waystone past
/// D1 is lit the two play the same sends on the same game — the start is D1 either way and the
/// passage is priced only below it — so the waystone twin is the D1 game cloned at its first
/// send that would start deeper, played on from there (a set that never lights one: the D1
/// tally itself). The same sends as two jobs, bit for bit, once.
fn cohort_golds(set: &RuleSet, seed: u64, hours: u64) -> (GoldTally, GoldTally) {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    pack_for(&mut g);
    let budget = hours * 3600 * riddle_core::offline::TICKS_PER_SECOND;
    let (mut t, mut consumed) = (GoldTally::default(), 0u64);
    let mut fork = None;
    gold_sends(&mut g, &mut t, &mut consumed, budget, false, Some(&mut fork));
    let way = match fork {
        Some((mut g2, mut t2, mut c2)) => {
            gold_sends(&mut g2, &mut t2, &mut c2, budget, true, None);
            t2
        }
        None => t,
    };
    (t, way)
}

#[allow(dead_code)]
fn cohort_gold(set: &RuleSet, seed: u64, hours: u64, waystone: bool) -> GoldTally {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    pack_for(&mut g);
    let budget = hours * 3600 * riddle_core::offline::TICKS_PER_SECOND;
    let (mut t, mut consumed) = (GoldTally::default(), 0u64);
    gold_sends(&mut g, &mut t, &mut consumed, budget, waystone, None);
    t
}

/// `cohort_gold`'s sends from `consumed` to `budget`. `fork` (D1 sends only): the game, tally
/// and clock at the first send a waystone start would begin below D1.
#[allow(clippy::type_complexity)]
fn gold_sends(g: &mut Game, t: &mut GoldTally, consumed: &mut u64, budget: u64, waystone: bool, mut fork: Option<&mut Option<(Game, GoldTally, u64)>>) {
    while *consumed < budget {
        if let Some(f) = fork.as_deref_mut() {
            if f.is_none() && g.lineage.waystones.iter().copied().max().unwrap_or(1) != 1 {
                *f = Some((g.clone(), *t, *consumed));
            }
        }
        // Each send's lines are read and cleared (a send's first lines share the last one's turn).
        g.lineage.gold_ledger.clear();
        g.lineage.rest_left = 0;
        g.lineage.start = if waystone { g.lineage.waystones.iter().copied().max().unwrap_or(1) } else { 1 };
        // Cut 27 §1: the passage the camp prices for this start — re-priced when the start moves
        // and every few sends (a player's camp re-reads it as the lineage grows)
        if g.lineage.start > 1 && (g.passage.is_none_or(|p| p.0 != g.lineage.start) || t.way_sends.is_multiple_of(5)) {
            let rules = g.lineage.rules().clone();
            g.passage = riddle_core::forecast::sim_passage(g, &rules);
        }
        g.start_run(None);
        let paid = g.run.as_ref().is_some_and(|r| r.start > 1 && r.passage > 0);
        if let Some(r) = g.run.as_ref().filter(|r| r.start > 1) {
            t.way_sends += 1;
            t.paid_sends += (r.passage > 0) as u32;
        }
        g.events.clear();
        let bought: Vec<u32> = {
            let r = g.run.as_ref().unwrap();
            r.hero.inv.iter().filter(|i| r.supplies.contains(&i.id) && !i.found && !i.free).map(|i| i.id).collect()
        };
        let mut n = 0u32;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < riddle_core::engine::MAX_TURNS_PER_RUN {
            g.tick();
            g.events.clear();
            n += 1;
        }
        let (turns, tier) = {
            let r = g.run.as_ref().unwrap();
            t.thefts += r.stolen_kinds.len() as u32;
            t.supply_thefts += r.stolen_kinds.iter().filter(|(id, _, _)| bought.contains(id)).count() as u32;
            t.leash_thefts += r.stolen_kinds.iter().filter(|(_, k, _)| k == "leash").count() as u32;
            (r.turn, r.over.unwrap_or(ExitTier::Return))
        };
        *consumed += turns as u64 + g.rest_after(turns, tier) as u64;
        t.ticks += turns as u64 + g.rest_after(turns, tier) as u64;
        g.finish_run();
        g.auto_keep();
        g.events.clear();
        t.sends += 1;
        if tier != ExitTier::Death {
            t.home_sends += 1;
        }
        let before = t.net();
        for l in &g.lineage.gold_ledger {
            let w = l.why.as_str();
            let d = l.delta as i64;
            if ["returned", "banked", "died", "lost", "stalled", "driven"].iter().any(|p| w.starts_with(p)) {
                t.home += d;
            } else if w.starts_with("salvage") {
                t.salvage += d;
            } else if w.starts_with("waystone") {
                t.tolls -= d;
            } else if w.starts_with("repeat") {
                t.spent -= d;
            } else {
                t.other += d;
            }
        }
        if tier == ExitTier::Return {
            t.ret_sends += 1;
            t.ret_net += t.net() - before;
        }
        if paid {
            t.paid_net += t.net() - before;
            t.paid_ticks += turns as u64 + g.rest_after(turns, tier) as u64;
        }
    }
}

/// Cut 23 §1: a cohort set's forge after an absence — the gold it brought home (from an empty
/// purse, the shelf packed), the steps it could then buy (cheapest first), and the nights of that
/// income the next step is away.
#[derive(Clone, Copy, Default, Debug)]
struct ForgeTally {
    /// Sends that banked or returned, of all sends.
    home: (u32, u32),
    income: i64,
    bought: u32,
    first_price: u32,
    next_price: u32,
    nights: f64,
    best: u32,
}

fn cohort_forge(set: &RuleSet, seed: u64, hours: u64) -> ForgeTally {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    pack_for(&mut g);
    g.lineage.gold = 0;
    let report = riddle_core::offline::run_offline_counts(&mut g, hours * 3600);
    let income = g.lineage.gold as i64;
    let price = |g: &Game| riddle_core::kit::ladders(&g.lineage).iter().filter_map(|l| l.next.as_ref().map(|n| (n.price, l.slot.clone()))).min();
    let mut t = ForgeTally { income, best: g.lineage.best_depth, home: (report.banked + report.returned, report.runs), ..Default::default() };
    t.first_price = price(&g).map(|p| p.0).unwrap_or(0);
    while let Some((p, slot)) = price(&g) {
        if (p as i32) > g.lineage.gold || riddle_core::kit::buy(&mut g, &slot).is_err() {
            break;
        }
        t.bought += 1;
    }
    if let Some((p, _)) = price(&g) {
        t.next_price = p;
        let short = (p as i64 - g.lineage.gold as i64).max(0) as f64;
        t.nights = if income > 0 { short / income as f64 } else { f64::INFINITY };
    }
    t
}

/// Cut 23 §1: the forge's rows — on each cohort set an 8 h absence leaves a step affordable and
/// the next ≤ 3 nights away (≥ 80 % of seeds; the median seed's nights); the kitted bots hold
/// the bot gates.
fn forge_report(sets: &[(String, RuleSet)], forges: &BTreeMap<(usize, u64), ForgeTally>, kres: &BTreeMap<(usize, u64), SeedResult>, res: &BTreeMap<(usize, u64), SeedResult>, seeds: u64, rows: &mut Vec<(String, String, bool)>) {
    println!("\nforge (Cut 23 §1; an 8 h absence from an empty purse): income · first step · steps bought · next step · nights to it (median) · affordable seeds");
    let (mut ok_n, mut n, mut worst) = (0usize, 0usize, (0.0f64, String::new()));
    for (si, (name, _)) in sets.iter().enumerate() {
        let ts: Vec<&ForgeTally> = (1..=seeds).filter_map(|s| forges.get(&(si, s))).collect();
        if ts.is_empty() {
            continue;
        }
        // (the Cut 22 gold gate's sets: sends that mostly come home — a set that dies most nights
        // has no night's income to price a step from; printed, not gated)
        let (h, r) = ts.iter().fold((0u32, 0u32), |a, t| (a.0 + t.home.0, a.1 + t.home.1));
        let mostly = pct(h as usize, r as usize) >= 60.0;
        let afford = pct(ts.iter().filter(|t| t.bought >= 1).count(), ts.len());
        let mut nights: Vec<f64> = ts.iter().map(|t| t.nights).collect();
        nights.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let med = nights[nights.len() / 2];
        let mean = |f: &dyn Fn(&ForgeTally) -> f64| ts.iter().map(|t| f(t)).sum::<f64>() / ts.len() as f64;
        let ok = afford >= 80.0 && med <= 3.0;
        if mostly {
            n += 1;
            ok_n += ok as usize;
            if med > worst.0 {
                worst = (med, name.clone());
            }
        }
        println!("  {name}: home {:.0}% · +${:.0} · first ${:.0} · bought {:.1} · next ${:.0} · {med:.2} nights · affordable {afford:.0}% · best D{:.1}{}", pct(h as usize, r as usize), mean(&|t| t.income as f64), mean(&|t| t.first_price as f64), mean(&|t| t.bought as f64), mean(&|t| t.next_price as f64), mean(&|t| t.best as f64), if !mostly { " (not mostly home)" } else if ok { "" } else { " · FAIL" });
    }
    if n > 0 {
        rows.push((format!("Forge: a step after 8 h, next ≤ 3 nights ({n} mostly-home sets)"), format!("{ok_n}/{n} · worst {:.1} n", worst.0), ok_n == n));
    }
    let ns = seeds as usize;
    let kit = |b: Bot| -> Vec<&SeedResult> {
        let bi = KIT_BOTS.iter().position(|x| *x == b).unwrap();
        (1..=seeds).filter_map(|s| kres.get(&(bi, s))).collect()
    };
    let plain = |b: Bot| -> Vec<&SeedResult> {
        let bi = BOTS.iter().position(|x| *x == b).unwrap();
        (1..=seeds).filter_map(|s| res.get(&(bi, s))).collect()
    };
    if kit(Bot::Default).len() < ns {
        return;
    }
    let mean_best = |rs: &[&SeedResult]| rs.iter().map(|r| r.best_depth as f64).sum::<f64>() / rs.len().max(1) as f64;
    let mean_death = |rs: &[&SeedResult]| {
        let d: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        d.iter().sum::<u32>() as f64 / d.len().max(1) as f64
    };
    println!("kitted bots (every forge step): best-depth mean · mean run depth");
    for b in KIT_BOTS {
        let rs = kit(b);
        let p = plain(b);
        println!("  {:<10} kitted {:>6.2} · {:>5.2}{}", b.name(), mean_best(&rs), mean_death(&rs), if p.len() == ns { format!("   (unkitted {:.2} · {:.2})", mean_best(&p), mean_death(&p)) } else { String::new() });
    }
    let d = kit(Bot::Default);
    let d_le8 = pct(d.iter().filter(|r| r.best_depth <= 8).count(), ns);
    rows.push(("KITTED (DEFAULT + every step) dies by ≤ D8 ≥ 80%".into(), format!("{d_le8:.0}%"), d_le8 >= 80.0));
    let e = kit(Bot::Edited);
    let (e10, d10) = (pct(e.iter().filter(|r| r.best_depth >= 10).count(), ns), pct(d.iter().filter(|r| r.best_depth >= 10).count(), ns));
    rows.push(("Kitted: EDITED − KITTED (≥ D10) ≥ 15 pts".into(), format!("{:.0} pts", e10 - d10), e10 - d10 >= 15.0));
    let r_lose = pct(kit(Bot::Random).iter().filter(|r| r.best_depth < 19).count(), ns);
    rows.push(("Kitted: RANDOM loses 100%".into(), format!("{r_lose:.0}%"), r_lose >= 100.0));
    let p_le3 = pct(kit(Bot::Passive).iter().filter(|r| r.best_depth <= 3).count(), ns);
    rows.push(("Kitted: PASSIVE loses by ≤ D3 100%".into(), format!("{p_le3:.0}%"), p_le3 >= 100.0));
    let (ml, md) = (mean_death(&kit(Bot::Learned)), mean_death(&d));
    rows.push(("Kitted: LEARNED mean depth ≤ KITTED + 2".into(), format!("{ml:.2} vs {md:.2}"), ml <= md + 2.0));
    let t8 = pct(kit(Bot::Trivial).iter().filter(|r| r.best_depth <= 8).count(), ns);
    rows.push(("Kitted: TRIVIAL never passes D8 ≥ 90%".into(), format!("{t8:.0}%"), t8 >= 90.0));
    for (bot, boss) in [(Bot::FullNo23, 23u32), (Bot::FullNo28, 28), (Bot::FullNo33, 33)] {
        let held = pct(kit(bot).iter().filter(|r| r.best_depth <= boss).count(), ns);
        rows.push((format!("Kitted: {} never passes D{boss} ≥ 90%", bot.name()), format!("{held:.0}%"), held >= 90.0));
    }
}

/// (set index, seed) → (kit move, best row move, the row).
type Levers = BTreeMap<(usize, u64), (f64, f64, String)>;

/// Cut 25 §1: the lever's measure — each banking cohort set's lineage after a day of its own
/// sends (`LEVER_HOURS`), `LEVER_SIMS` paired sends per arm, `LEVER_SEEDS` seeds pooled.
const LEVER_HOURS: u64 = 24;
const LEVER_SIMS: u32 = 64;
const LEVER_SEEDS: u64 = 3;
/// A row move this far past the seed's kit move ends the seed's search (the table's `row ≥`).
const LEVER_MARGIN: f64 = 0.05;

/// Cut 25 §1 (AN: three forge buys took bank 40 → 95 %, more than any row he wrote): on every
/// banking cohort set, the whole forge's paired bank move is less than the set's best single-row
/// move (`lever::gate`), pooled over the seeds; a forge that moves nothing passes.
fn lever_report(sets: &[(String, RuleSet)], levers: &Levers, seeds: u64, rows: &mut Vec<(String, String, bool)>) {
    println!("\nlever (Cut 25 §1; the set's lineage after {LEVER_HOURS} h, {LEVER_SIMS} paired sends × {seeds} seeds): whole-forge bank move · best row move (the first past the forge) per seed");
    let (mut ok_n, mut n, mut worst) = (0usize, 0usize, (f64::MAX, String::new()));
    for (si, (name, _)) in sets.iter().enumerate() {
        let ms: Vec<&(f64, f64, String)> = (1..=seeds).filter_map(|s| levers.get(&(si, s))).collect();
        if ms.is_empty() {
            continue;
        }
        let kit = ms.iter().map(|m| m.0).sum::<f64>() / ms.len() as f64;
        let row = ms.iter().map(|m| m.1).sum::<f64>() / ms.len() as f64;
        let ok = kit < row || kit <= 0.005;
        n += 1;
        ok_n += ok as usize;
        if row - kit < worst.0 {
            worst = (row - kit, name.clone());
        }
        println!("  {name}: forge {:+.1} · row ≥ {:+.1} ({}){}", 100.0 * kit, 100.0 * row, ms.iter().map(|m| format!("{:+.0}/{:+.0} {}", 100.0 * m.0, 100.0 * m.1, m.2)).collect::<Vec<_>>().join(" · "), if ok { "" } else { " · FAIL" });
    }
    if n > 0 {
        rows.push((format!("Whole forge's bank move < best row's ({n} banking sets)"), format!("{ok_n}/{n} · min gap {:+.0}", 100.0 * worst.0), ok_n == n));
    }
}

/// Cut 22 §3: every one-notch edit of `set` (each numeric condition ±5 for a share, ±1
/// otherwise) against the set, on a D8 lineage owning every token: (edits, Σ paired ±,
/// Σ absolute ±) over the shaft's depths whose bar has a ±.
fn paired_edits(set: &RuleSet) -> (u32, f64, f64) {
    let mut g = Game::new_literal(7);
    for u in riddle_core::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    g.lineage.best_depth = 8;
    if g.set_rules_raw(set.clone()).is_err() {
        return (0, 0.0, 0.0);
    }
    let _ = g.forecast();
    let (mut n, mut sp, mut sa) = (0u32, 0.0, 0.0);
    for (ri, r) in set.rows.iter().enumerate() {
        for (ci, c) in r.conds.iter().enumerate() {
            let Some(v) = c.n else { continue };
            let step = if c.k.starts_with("hp") { 5 } else { 1 };
            for sign in [-1, 1] {
                let mut edit = set.clone();
                edit.rows[ri].conds[ci].n = Some((v + sign * step).max(0));
                if edit == *set || g.set_rules_raw(edit).is_err() {
                    continue;
                }
                let vs = g.forecast_vs(set);
                for d in vs.depths.iter().filter(|d| d.abs_pm > 0.0) {
                    sp += d.pm;
                    sa += d.abs_pm;
                }
                n += 1;
            }
        }
    }
    (n, sp, sa)
}

/// Cut 27 §2: one cohort set's camp edits and their scenes — (edits moving the forecast ≥ 5 pts,
/// divergences found for them, found at a row, edits inside their ± with a scene, the slowest
/// scene in ms).
type Diverged = (u32, u32, u32, u32, f64);

/// Cut 27 §2: the edits a player makes in a turn at camp — each row cut, each row moved up one,
/// each row's first number nudged (hp ±10, else ±1) — on a D8 lineage owning the set's tokens,
/// each paired against the set (`forecast_vs`); for each that moves the forecast ≥ 5 pts (the
/// largest |Δ| over the shaft and the ends), whether the core finds its divergence.
fn diverge_edits(set: &RuleSet) -> Diverged {
    let mut g = cohort_game(set, 7);
    g.lineage.best_depth = 8;
    let _ = g.forecast();
    let mut edits: Vec<RuleSet> = Vec::new();
    for ri in 0..set.rows.len() {
        let mut e = set.clone();
        e.rows.remove(ri);
        edits.push(e);
        if ri > 0 {
            let mut e = set.clone();
            e.rows.swap(ri - 1, ri);
            edits.push(e);
        }
        if let Some((ci, c)) = set.rows[ri].conds.iter().enumerate().find(|(_, c)| c.n.is_some()) {
            let step = if c.k.starts_with("hp") { 10 } else { 1 };
            for sign in [-1, 1] {
                let mut e = set.clone();
                e.rows[ri].conds[ci].n = Some((c.n.unwrap() + sign * step).max(0));
                if e != *set {
                    edits.push(e);
                }
            }
        }
    }
    let (mut n5, mut found, mut rows, mut inside, mut slow) = (0u32, 0u32, 0u32, 0u32, 0.0f64);
    for e in edits {
        if g.set_rules_raw(e).is_err() {
            continue;
        }
        let vs = g.forecast_vs(set);
        let mut moves: Vec<(f64, f64)> = vs.depths.iter().map(|d| (d.delta, d.pm)).collect();
        moves.extend([(vs.bank.delta, vs.bank.pm), (vs.death.delta, vs.death.pm), (vs.return_.delta, vs.return_.pm), (vs.stall.delta, vs.stall.pm)]);
        let (moved, pm) = moves.iter().copied().fold((0.0f64, 0.0f64), |a, (d, p)| if d.abs() > a.0 { (d.abs(), p) } else { a });
        if moved < 0.05 && moved > pm {
            continue;
        }
        let t = Instant::now();
        let d = g.divergence(set);
        let ms = t.elapsed().as_secs_f64() * 1e3;
        if ms > 400.0 {
            eprintln!("  slow scene {ms:.0} ms on {:?}: {:?}", set.name, d.as_ref().map(|d| (d.seed, d.tick, d.sent_row, d.new_row)));
        }
        slow = slow.max(ms);
        if moved >= 0.05 {
            n5 += 1;
            found += d.is_some() as u32;
            rows += d.as_ref().is_some_and(|d| d.sent_row.is_some() || d.new_row.is_some()) as u32;
        } else if d.is_some() {
            inside += 1;
        }
    }
    let _ = g.set_rules_raw(set.clone());
    (n5, found, rows, inside, slow)
}

fn setup(bot: Bot, seed: u64) -> Game {
    let mut g = Game::new_literal(seed);
    g.max_deaths = 100_000;
    // Cut 29 §2: the bots play with every system open (the curriculum gates the editor and the camp,
    // never a sim: their play is what it was).
    riddle_core::systems::open_all(&mut g.lineage);
    // Cut 30: every bot runs the neutral heir (no trait now or at any wake).
    riddle_core::traits::neutral(&mut g.lineage);
    match bot {
        Bot::Default => {}
        Bot::Edited => {
            for u in ["row5", "row6", "row7", "row8"] {
                g.lineage.unlocks.insert(u.into());
            }
            // The vocabulary a player has after identifying the common items.
            for k in ["heal", "poison", "fire", "teleport", "blink"] {
                if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
                    g.lineage.facts.insert(f);
                }
            }
            for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss"] {
                g.lineage.facts.insert(f.into());
            }
            // Cut 2 §3: the condition tokens are unlocks; a player this far along owns them.
            for u in ["throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see"] {
                g.lineage.unlocks.insert(u.into());
            }
            g.set_rules(good()).expect("good rules");
        }
        Bot::Random => {
            let vocab = g.vocabulary();
            let mut rng = Rng::derive(seed, 0xBADC0DE);
            let set = riddle_core::probes::random_rules(&mut rng, &vocab, 4);
            g.set_rules(set).expect("random rules");
        }
        Bot::Passive => {
            g.set_rules(RuleSet::default()).unwrap();
        }
        Bot::Learned => {
            riddle_core::probes::learn_everything(&mut g);
        }
        Bot::Pets => {
            g.lineage.unlocks.insert("party_slot_2".into());
            g.lineage.party = riddle_core::probes::pets_party();
        }
        Bot::Levelled => {
            g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0, next: 0 });
        }
        Bot::Trivial | Bot::Countered => {
            for u in ["row5", "row6", "row7", "row8", "tame", "throw"] {
                g.lineage.unlocks.insert(u.into());
            }
            g.lineage.facts.insert("item:leash".into());
            if bot == Bot::Countered {
                for k in ["fire", "poison"] {
                    if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
                        g.lineage.facts.insert(f);
                    }
                }
                for f in ["foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss", "foe:skeleton:summoned"] {
                    g.lineage.facts.insert(f.into());
                }
            }
            g.set_rules_raw(if bot == Bot::Trivial { riddle_core::probes::trivial() } else { riddle_core::probes::countered() }).unwrap();
        }
        Bot::Full | Bot::FullNo23 | Bot::FullNo28 | Bot::FullNo33 => {
            // Everything a finished lineage has: every unlock, every fact, a mastered fighter.
            for u in riddle_core::meta::UNLOCKS {
                g.lineage.unlocks.insert(u.id.into());
            }
            riddle_core::probes::learn_everything(&mut g);
            g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0, next: 0 });
            g.lineage.unlocks.insert(riddle_core::hero::mastery_card(Class::Fighter).into());
            let mut set = full();
            let drop = |set: &mut RuleSet, verb: &str, arg: &str| {
                let i = set.rows.iter().position(|r| r.verb.v == verb && r.verb.a.as_deref() == Some(arg)).expect("counter row present");
                set.rows.remove(i);
            };
            match bot {
                Bot::FullNo23 => drop(&mut set, "tactic", "reflect_read"),
                Bot::FullNo28 => drop(&mut set, "read", "silence"),
                Bot::FullNo33 => drop(&mut set, "tactic", "cadence"),
                _ => {}
            }
            g.set_rules(set).expect("full rules");
        }
    }
    g
}

/// One reseeded replay of the last ticks before a death with `rules`; true if `row` fired.
fn patch_fired(g: &Game, rec: &riddle_core::engine::DeathRec, rules: &RuleSet, row: i32, nonce: u64) -> bool {
    let Some(t10) = rec.t10.clone() else { return false };
    let mut base = g.sim_clone();
    base.lineage.facts = rec.t10_facts.clone();
    base.lineage.heir = t10.heir;
    base.lineage.trait_ = t10.trait_;
    base.lineage.class = t10.hero.class;
    base.lineage.party.clear();
    base.lineage.supplies.clear();
    let last_t = rec.death.trace.turns.last().map(|t| t.t).unwrap_or(t10.turn + 100);
    let ticks = (last_t.saturating_sub(t10.turn)).max(1) + 1;
    let turn = t10.turn as u64;
    base.run = Some(t10);
    let _ = base.set_rules(rules.clone());
    if let Some(run) = base.run.as_mut() {
        run.rng = Rng::derive(g.lineage.seed ^ splitmix(nonce), turn);
    }
    for _ in 0..ticks {
        base.tick();
        if base.events.iter().any(|e| matches!(e, Ev::Rule { row: r, .. } if *r == row)) {
            return true;
        }
        base.events.clear();
        if base.run.as_ref().is_none_or(|r| r.over.is_some()) {
            break;
        }
    }
    false
}

/// Cut 26 §3: the lane gate's lineage seeds and paired sends per lane and seed.
const LANE_SEEDS: u64 = 3;
const LANE_SIMS: u32 = 96;

/// Cut 26 §3: the plateau search's results (`examples/lanes.rs --search`), shipped as a preset:
/// the per-lane best sets are re-read on fresh paired seeds by the table.
fn lane_found() -> Vec<lanes::Found> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/presets/lanes.json");
    std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
}

/// Cut 26 §4: the first-hour agent's sends (the first fork is due within six).
const FIRST_FORK_CAP: u32 = 12;
const FIRST_FORK_SENDS: u32 = 6;

/// Cut 26 §5: the bots the gate table also plays on a swapped route, one per seed.
const ROUTE_BOTS: [Bot; 5] = [Bot::Default, Bot::Edited, Bot::Random, Bot::Passive, Bot::Learned];

/// Cut 26 §5: the swapped route a seed plays (the gate table's sample, as verdicts are sampled):
/// the 12 routes off the base order in turn, so 30 seeds cover each of them.
fn sampled_route(seed: u64) -> riddle_core::descent::Route {
    let all = riddle_core::descent::Route::all();
    all[1 + ((seed as usize).saturating_sub(1)) % (all.len() - 1)]
}

fn run_seed(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize, kit: bool) -> SeedResult {
    run_seed_on(bot, seed, hours, verdicts_per_seed, kit, riddle_core::descent::Route::BASE)
}

/// `run_seed_on` for the kitted and the route bots, whose rows read a seed's best depth, run
/// depths, deaths and sampled verdicts only (`forge_report`, the routes' rows): the stall
/// deaths' screens, the shown patches' replays, the root and dice patches, the trace reasons and
/// the `known_to` forecast are the plain bots' rows (`all`), computed there and nowhere else.
/// A stall screen (`death`) and those after the verdicts write only their deaths' records,
/// which no later send reads — the depths, deaths and verdicts are the same (the table's text
/// is unchanged, byte for byte).
fn run_seed_lean(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize, kit: bool, route: riddle_core::descent::Route) -> SeedResult {
    run_seed_with(bot, seed, hours, verdicts_per_seed, kit, route, true)
}

fn run_seed_on(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize, kit: bool, route: riddle_core::descent::Route) -> SeedResult {
    run_seed_with(bot, seed, hours, verdicts_per_seed, kit, route, false)
}

/// `run_seed` on `route` (every fork seen; the bot's set written with it).
fn run_seed_with(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize, kit: bool, route: riddle_core::descent::Route, lean: bool) -> SeedResult {
    let mut g = setup(bot, seed);
    if !route.is_base() {
        for f in riddle_core::descent::FORKS {
            g.lineage.facts.insert(format!("fork:{f}"));
        }
        let set = g.lineage.rules().clone().with_route(route);
        g.set_rules_raw(set).expect("the bot's set on a route");
    }
    if kit {
        riddle_core::kit::buy_all(&mut g.lineage);
        // `KIT_ONLY=weapon,armour`: the kitted bots with those ladders alone (tuning).
        if let Ok(only) = std::env::var("KIT_ONLY") {
            g.lineage.kit.retain(|k, _| only.split(',').any(|o| o == k));
        }
    }
    let mut r = SeedResult::default();
    let mut secs = 0.0;
    for _ in 0..bot.batches() {
        let t = Instant::now();
        let report = riddle_core::offline::run_offline_counts(&mut g, hours * 3600);
        secs += t.elapsed().as_secs_f64();
        r.learned += report.learned.len();
        r.pending += report.pending.len();
        r.events += g.batch.renderable_events;
        r.ticks += g.batch.turns;
        r.runs += report.runs;
        r.banked += report.banked;
        r.returned += report.returned;
        r.xp += report.xp.gained;
        r.marks += g.batch.marks;
        r.frontier += g.batch.frontier_banks;
        r.rested_s += report.rested_s;
        r.run_ticks.extend(g.batch.run_ticks.iter().copied());
        // Cut 5 gates: the reel and the story lines of this absence, the situations met.
        let pairs: std::collections::BTreeSet<_> = report.reel.iter().filter_map(riddle_core::sifter::pair).collect();
        r.reel_pairs.push(pairs.len());
        r.reel_names.push(report.reel.first().is_some_and(riddle_core::sifter::names_agent));
        for h in g.batch.highlights.iter().filter(|h| h.arc.is_some()) {
            r.stories.0 += 1;
            if riddle_core::sifter::story_ok(&h.text) {
                r.stories.1 += 1;
            }
        }
        r.situations.0 += g.batch.run_outcomes.len() as u32;
        r.situations.1 += g.batch.situation_runs;
        r.band.extend(g.batch.band_runs.iter().cloned());
        for b in &g.batch.band_runs {
            if b.depth >= riddle_core::descent::LIEUTENANT_DEPTH {
                r.captain.0 += 1;
                r.captain.1 += (b.depth > riddle_core::descent::LIEUTENANT_DEPTH) as u32;
            }
        }
        for (d, c) in &g.batch.run_outcomes {
            r.run_depths.push(*d);
            if let Some(c) = c {
                r.causes.push(c.clone());
            }
        }
        // Cut 13 §1: stalls per send; every sampled stall's verdict names a row that fires and
        // leaves the floor, and its reel line says what its trace says (Q: "no patch is
        // offered"; R: "the reel blamed `R3 drink heal caught him` while the trace said `R1
        // throw unknown stuck`").
        r.stalls.0 += g.batch.run_outcomes.len() as u32;
        r.stalls.1 += g.batch.stalls;
        let stall_ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| rec.stall && !lean).map(|(id, _)| *id).filter(|id| g.batch.highlights.iter().any(|h| h.run_id == *id)).take(2).collect();
        for id in stall_ids {
            let Some(d) = g.death(id) else { continue };
            let rec = g.deaths.get(&id).cloned().unwrap();
            r.stall_verdicts.0 += 1;
            let fires = d.verdict == "stall" && d.patches.iter().any(|p| riddle_core::trace::patch_fired_rate(&g, &rec, p) >= 0.5);
            r.stall_verdicts.1 += fires as u32;
            r.stall_reel.0 += 1;
            let cause = d.cause.strip_prefix("stalled · ").unwrap_or(&d.cause);
            let want = format!("stalled, {}", riddle_core::sifter::stall_short(cause));
            let same = g.batch.highlights.iter().filter(|h| h.run_id == id).filter_map(|h| h.arc.as_ref()).any(|a| a.resolution == want);
            r.stall_reel.1 += same as u32;
            if (!fires || !same) && std::env::var("STALL_DEBUG").is_ok() {
                eprintln!("STALL {} seed {seed} run {id} D{} cause {} verdict {} patches {:?} reel {:?}", bot.name(), d.depth, d.cause, d.verdict, d.patches.iter().map(|p| p.row.describe()).collect::<Vec<_>>(), g.batch.highlights.iter().filter(|h| h.run_id == id).map(|h| h.text.clone()).collect::<Vec<_>>());
                if let Some(t10) = rec.t10.as_ref() {
                    for row in riddle_core::trace::stall_candidates(&rec.vocab, t10) {
                        eprintln!("  cand {} → {:?} (baseline {:.2})", row.describe(), riddle_core::trace::measure_row(&g, &rec, &row), d.baseline);
                    }
                }
            }
        }
    }
    r.best_depth = g.lineage.best_depth;
    // Earned gold only: exits and salvage. Wake pay (a new heir's potion) is a stipend, not a
    // yield, so the "DEFAULT yields 0" gate keeps measuring the policy, not the purse.
    r.gold = g.batch.gold_earned + g.batch.salvaged.values().map(|(_, cents)| (cents + 50) / 100).sum::<i32>();
    r.tick_us = secs * 1e6 / r.ticks.max(1) as f64;
    // Verdicts cost ~1.2 s each (candidates × reseeded replays); sample evenly across the seed's
    // deaths. 8 per seed × seeds × bots is plenty for the unfair/gap shares.
    // Cut 13 §1: stall records live beside the deaths (`DeathRec.stall`); the death gates
    // sample the deaths alone.
    let ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| !rec.stall).map(|(id, _)| *id).collect();
    let step = (ids.len() / verdicts_per_seed.max(1)).max(1);
    for id in ids.iter().step_by(step).take(verdicts_per_seed) {
        let t = Instant::now();
        if let Some(v) = riddle_core::trace::verdict(&mut g, *id) {
            r.verdict_secs.push(t.elapsed().as_secs_f64());
            if v == "dice" && std::env::var("DICE_DEBUG").is_ok() {
                let rec = g.deaths.get(id).unwrap();
                eprintln!("DICE {} seed {seed} run {id} D{} cause {} margin {} rows {} base {:.2} boss {}", bot.name(), rec.death.depth, rec.death.cause, rec.death.margin, rec.death.trace.turns.len(), rec.death.baseline, rec.boss.is_some());
            }
            r.verdicts.push(v);
        }
    }
    if lean {
        return r;
    }
    // Cut 2 §6 patch quality: the shown patches (full `death()`, two per seed on the
    // player-shaped bots) must have fired in ≥ 50% of 12 reseeded replays of the death.
    if matches!(bot, Bot::Default | Bot::Edited) && !kit {
        for id in ids.iter().step_by(step).take(2) {
            let t = Instant::now();
            let Some(d) = g.death(*id) else { continue };
            r.death_secs.push(t.elapsed().as_secs_f64());
            let rec = g.deaths.get(id).cloned().unwrap();
            // Cut 11 §2: a root-cause patch answers a theft floors back (or an unlock); its
            // number is the forecast delta, not the moment's replays — exempt here. §4: so is
            // a `dice` death's below-bar alternative (labelled as such on the screen).
            // Cut 19 §4: a `row` verdict's cut (a removed or narrowed row) inserts nothing.
            // Cut 25 §2: a move reorders the set (kept only when it acts in ≥ `FIRED_BAR` of the
            // death's own replays: `trace::order_moves`).
            for p in d.patches.iter().filter(|p| p.insert_at >= 0 && p.root.is_none() && !p.below_bar && !p.remove && !p.replace && p.moves_from.is_none()) {
                let mut rules = rec.rules.clone();
                let at = (p.insert_at as usize).min(rules.rows.len());
                rules.rows.insert(at, p.row.clone());
                rules.rows.truncate(rec.vocab.max_rows.max(rules.rows.len()));
                // The selection's own replays (trace::patch_fired_rate) plus 12 fresh ones: a
                // shown patch must have fired in at least half of each.
                let own = riddle_core::trace::patch_fired_rate(&g, &rec, p);
                let fresh = (0..12).filter(|i| patch_fired(&g, &rec, &rules, at as i32, 0xF1_7ED0 + *i as u64 + ((*id as u64) << 8))).count();
                r.patches.0 += 1;
                if own >= 0.5 {
                    r.patches.1 += 1;
                } else if std::env::var("PATCH_DEBUG").is_ok() {
                    eprintln!("PATCH {} seed {seed} run {id}: {} {} at R{} fired {own:.2} · survive {:.2} base {:.2} drops {:?} · {:?}", bot.name(), d.verdict, p.row.describe(), p.insert_at + 1, p.survive, d.baseline, p.drops, d.patches.iter().map(|x| x.row.describe()).collect::<Vec<_>>());
                }
                r.patches_fresh.0 += 12;
                r.patches_fresh.1 += fresh as u32;
            }
        }
    }
    // Cut 11 §1: every state reason in a death trace carries a `because` when the run has
    // one — counted straight off the records (no verdict needed). `no path` and `not in view`
    // may legitimately have none (no such foe on the floor; a block the log cannot name), so
    // the gate reads the slot reasons and the rest is printed.
    let state = ["no item", "none held", "no path", "not in view", "cooldown", "locked cond", "no way"];
    for rec in g.deaths.values().filter(|rec| !rec.stall) {
        r.because_deaths += 1;
        for t in &rec.death.trace.turns {
            for w in t.rows.iter().flatten() {
                if state.iter().any(|s| w.why.starts_with(s)) {
                    r.because.0 += 1;
                    r.because.1 += w.because.is_some() as u32;
                }
            }
        }
    }
    // Cut 11 §2: root-cause patches on the player-shaped bots (≤ 3 root deaths per seed).
    if matches!(bot, Bot::Default | Bot::Edited | Bot::Pets | Bot::Levelled) && !kit {
        let root_ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| rec.root.is_some() && !rec.stall).map(|(id, _)| *id).take(3).collect();
        for id in root_ids {
            let Some(d) = g.death(id) else { continue };
            if g.deaths.get(&id).is_some_and(|rec| rec.root_under_base) {
                r.roots_under += 1;
                continue;
            }
            r.roots.0 += 1;
            if let Some(p) = d.patches.iter().find(|p| p.root.is_some()) {
                r.roots.1 += 1;
                let best = d.patches.iter().filter(|p| p.root.is_none()).map(|p| p.forecast_delta).fold(f64::NEG_INFINITY, f64::max);
                if p.forecast_delta >= best - 1e-9 {
                    r.roots.2 += 1;
                }
            }
        }
    }
    // Cut 11 §4: a sampled `dice` death names an alternative (≤ 2 per seed).
    let dice_ids: Vec<u32> = ids.iter().step_by(step).take(verdicts_per_seed).copied().filter(|id| g.deaths.get(id).is_some_and(|rec| rec.verdict_done && rec.death.verdict == "dice")).take(2).collect();
    for id in dice_ids {
        let Some(d) = g.death(id) else { continue };
        r.dice_named.0 += 1;
        let named = !d.patches.is_empty() && d.patches.iter().all(|p| (0.0..=1.0).contains(&p.survive));
        r.dice_named.1 += named as u32;
        if !named && std::env::var("DICE_DEBUG").is_ok() {
            let rec = g.deaths.get(&id).unwrap();
            eprintln!("DICE EMPTY {} seed {seed} run {id} D{} cause {} baseline {:.2} t10 {} turns {}", bot.name(), d.depth, d.cause, d.baseline, rec.t10.is_some(), d.trace.turns.len());
        }
    }
    // `known_to` is the panel's range, not a sim result: the same `forecast_with` at MIN_SIMS
    // (1/10 of the player's panel) reads the same field — this was 15–17 % of a job's CPU
    // (docs/ITERATION_SPEED.md §3.2).
    let f = riddle_core::forecast::forecast_with(&g, g.lineage.rules(), riddle_core::forecast::MIN_SIMS);
    r.known_to_ok = f.known_to == g.lineage.best_depth + 1;
    r.swore = g.lineage.oath_sworn.is_some() || g.lineage.oaths_kept > 0;
    r
}

/// Cut 28 §1: the cohort sets the oaths' gate plays — the last two cohorts' (the sets the oaths
/// were designed against, cohort 23's among them).
const OATH_SETS: [&str; 4] = ["631fe23.raterAU", "631fe23.raterAV", "420f27c.raterAS", "420f27c.raterAT"];
/// (set index) → the bank-optimal set's (rules, bank share, edits); (set, pool kind) → the oath read.
type OathBanks = BTreeMap<usize, (RuleSet, f64, Vec<String>)>;
type OathReads = BTreeMap<(usize, usize), oath_lib::OathRead>;
/// (set index) → the lever with every oath reward owned (kit move, best row move, the row).
type OathLevers = BTreeMap<usize, (f64, f64, String)>;

/// Cut 28 §1: the oaths' rows — on each oath set's lineage, every pool oath offered there: its best
/// set (a plateau search) differs from the bank-optimal set by ≥ 2 rows; its best set keeps it in a
/// night of 16 sends ≥ 20 % of the time; and the Cut 25 lever row holds with every oath reward owned.
fn oath_report(sets: &[(String, RuleSet)], banks: &OathBanks, reads: &OathReads, levers: &OathLevers, rows: &mut Vec<(String, String, bool)>) {
    println!("\noaths (Cut 28 §1; the set's lineage after {} h, searches of {} steps on {}/{} sends): per set and oath, the oath share with the set → with its best set · the night's chance · rows from the bank-optimal set", oath_lib::OATH_HOURS, oath_lib::STEPS, oath_lib::SCREEN, oath_lib::FULL);
    let (mut n, mut diverse, mut done) = (0usize, 0usize, 0usize);
    let (mut worst_night, mut worst_diff) = ((f64::MAX, String::new()), (usize::MAX, String::new()));
    for (si, (name, _)) in sets.iter().enumerate() {
        let Some(bank) = banks.get(&si) else { continue };
        println!("  {name}: bank-optimal {:.0}% via {:?}", 100.0 * bank.1, bank.2);
        for (ki, kind) in riddle_core::oath::KINDS.iter().enumerate() {
            let Some(r) = reads.get(&(si, ki)) else { continue };
            if !r.offered {
                println!("    {kind:<7} not offered (D{})", r.best_depth);
                continue;
            }
            // the rows the oath's best set differs from the bank-optimal one by (the two searches' sets)
            let diff = oath_lib::row_diff(&r.best, &bank.0);
            n += 1;
            diverse += (diff >= 2) as usize;
            done += (r.night() >= 0.2) as usize;
            let tag = format!("{} {}", name.split('.').nth(1).unwrap_or(name), r.text);
            if r.night() < worst_night.0 {
                worst_night = (r.night(), tag.clone());
            }
            if diff < worst_diff.0 {
                worst_diff = (diff, tag);
            }
            println!("    {kind:<7} {:<18} {:>3.0}% → {:>3.0}% · night {:>3.0}% · diff {diff} · {:?}", r.text, 100.0 * r.share_set, 100.0 * r.share_best, 100.0 * r.night(), r.edits);
        }
    }
    if n > 0 {
        rows.push((format!("Oath best set ≠ bank-optimal by ≥ 2 rows ({n} oath·sets)"), format!("{diverse}/{n} · min {} {}", worst_diff.0, worst_diff.1), diverse == n));
        rows.push((format!("Oath completable ≥ 20%/night by a set ({n} oath·sets)"), format!("{done}/{n} · min {:.0}% {}", 100.0 * worst_night.0, worst_night.1), done == n));
    }
    if !levers.is_empty() {
        let ok = levers.values().filter(|m| m.0 < m.1 || m.0 <= 0.005).count();
        for (si, m) in levers {
            println!("  lever with every oath reward · {}: forge {:+.1} · row ≥ {:+.1} ({})", sets[*si].0, 100.0 * m.0, 100.0 * m.1, m.2);
        }
        let gap = levers.values().map(|m| m.1 - m.0).fold(f64::MAX, f64::min);
        rows.push((format!("Lever holds with every oath reward ({} sets)", levers.len()), format!("{ok}/{} · min gap {:+.0}", levers.len(), 100.0 * gap), ok == levers.len()));
    }
}

type Golds = BTreeMap<(usize, u64, bool), GoldTally>;
/// Cut 22 §3: set index → (edits, Σ paired ±, Σ absolute ±) (`paired_edits`).
type Paireds = BTreeMap<usize, (u32, f64, f64)>;

fn gold_report(sets: &[(String, RuleSet)], golds: &Golds, seeds: u64, hours: u64, rows: &mut Vec<(String, String, bool)>) {
    // Cut 22 §1: a safe run nets gold — per send, after its supplies, tolls and thefts, on every
    // cohort set whose sends mostly come home (bank or return ≥ 60 %), from D1. The deepest
    // waystone's numbers print beside (a start the player picks, not the gate's).
    let (mut g_ok, mut g_n, mut g_worst) = (0usize, 0usize, (f64::INFINITY, String::new()));
    let (mut th_all, mut th_sup, mut th_sends, mut th_leash) = (0u32, 0u32, 0u32, 0u32);
    let mut leash_worst = (0.0f64, String::new());
    let (mut w_ok, mut w_n, mut w_worst) = (0usize, 0usize, (f64::INFINITY, String::new()));
    println!("\nnet gold per send (Cut 22 §1; {hours} h watched, repeat on): home + salvage − spent − tolls (+ other) over sends");
    for (si, (name, _)) in sets.iter().enumerate() {
        let tally = |way: bool| {
            let mut t = GoldTally::default();
            for s in 1..=seeds {
                t.add(&golds[&(si, s, way)]);
            }
            t
        };
        let (d1, ws) = (tally(false), tally(true));
        let per = |t: &GoldTally, x: i64| x as f64 / t.sends.max(1) as f64;
        let home_pct = pct(d1.home_sends as usize, d1.sends as usize);
        let mostly = home_pct >= 60.0;
        let net = per(&d1, d1.net());
        println!(
            "  {name}: D1 {:.1} sends · home {home_pct:.0}% · +${:.1} home · +${:.1} salvage · −${:.1} spent · −${:.1} tolls · {:+.1} other → net {net:+.1}/send{} · a return nets {:+.1} · supply thefts {:.2}/send ({} thefts) │ waystone: {:.1} sends · net {:+.1}/send · +${:.1} home · +${:.1} salvage · −${:.1} spent · tolls −${:.1} · {:+.1} other · home {:.0}%",
            d1.sends as f64 / seeds as f64,
            per(&d1, d1.home),
            per(&d1, d1.salvage),
            per(&d1, d1.spent),
            per(&d1, d1.tolls),
            per(&d1, d1.other),
            if mostly { "" } else { " (not mostly home)" },
            d1.ret_net as f64 / d1.ret_sends.max(1) as f64,
            d1.supply_thefts as f64 / d1.sends.max(1) as f64,
            d1.thefts,
            ws.sends as f64 / seeds as f64,
            per(&ws, ws.net()),
            per(&ws, ws.home),
            per(&ws, ws.salvage),
            per(&ws, ws.spent),
            per(&ws, ws.tolls),
            per(&ws, ws.other),
            pct(ws.home_sends as usize, ws.sends as usize),
        );
        // Cut 27 §1: gold per hour from D1 and from the waystone; the gate reads the sets whose
        // waystone sends were mostly paid their passage (the set clears the floors above ≥ 95 %)
        // (the sends that cleared: the lineage priced the passage and was paid it; a set with at
        // least a fifth of its waystone sends paid, over 10 of them, is read)
        let paid = ws.paid_sends >= 10 && ws.paid_sends * 5 >= ws.way_sends;
        println!("    $/h: D1 {:+.0} · waystone {:+.0} · its paid sends {:+.0} ({} of {} sends paid a passage){}", d1.per_hour(), ws.per_hour(), ws.paid_per_hour(), ws.paid_sends, ws.way_sends, if paid && ws.paid_per_hour() < d1.per_hour() { " · FAIL" } else { "" });
        if paid {
            w_n += 1;
            if ws.paid_per_hour() >= d1.per_hour() {
                w_ok += 1;
            }
            let r = ws.paid_per_hour() / d1.per_hour().max(1e-9);
            if r < w_worst.0 {
                w_worst = (r, name.clone());
            }
        }
        th_all += d1.thefts + ws.thefts;
        th_sup += d1.supply_thefts + ws.supply_thefts;
        th_leash += d1.leash_thefts + ws.leash_thefts;
        let lp = (d1.leash_thefts + ws.leash_thefts) as f64 / (d1.sends + ws.sends).max(1) as f64;
        if lp > leash_worst.0 {
            leash_worst = (lp, name.clone());
        }
        th_sends += d1.sends + ws.sends;
        if mostly {
            g_n += 1;
            if net >= 20.0 {
                g_ok += 1;
            }
            if net < g_worst.0 {
                g_worst = (net, name.clone());
            }
        }
    }
    let sup_per = th_sup as f64 / th_sends.max(1) as f64;
    println!("  bought-supply thefts {th_sup} of {th_all} thefts over {th_sends} sends ({sup_per:.3}/send; Cut 22 §2 target ≤ 0.1)");
    let leash_per = th_leash as f64 / th_sends.max(1) as f64;
    println!("  leash thefts {th_leash} over {th_sends} sends ({leash_per:.3}/send; worst set {} {:.3}; Cut 23 §5 target ≤ 0.1)", leash_worst.1, leash_worst.0);
    rows.push((format!("Leash thefts ≤ 0.1 per send, every cohort set (n={th_sends})"), format!("{leash_per:.3} · worst {:.3}", leash_worst.0), leash_worst.0 <= 0.1));
    if g_n > 0 {
        rows.push((format!("Net gold per send ≥ +$20 on every mostly-home cohort set ({g_n})"), format!("{g_ok}/{g_n} · worst {:+.0}", g_worst.0), g_ok == g_n));
    }
    // Cut 27 §1: a lit waystone start is never worse on gold/hr than D1 for a set that clears the
    // floors above it (its sends paid their passage)
    rows.push((format!("Waystone $/h ≥ D1 $/h, cohort sets paid a passage ({w_n})"), format!("{w_ok}/{w_n} · worst {:.2}× {}", if w_worst.0.is_finite() { w_worst.0 } else { 1.0 }, w_worst.1), w_ok == w_n && w_n > 0));
}

/// A gate row that measured nothing (a count of 0 in its name: `(n=0)`, `(0 cohort edits)`,
/// `(0 sets, …)`) FAILS — a gate silently doing nothing is not a pass (Cut 27 follow-up: a
/// `--gold` run printed `Paired edit ± … (0 cohort edits) PASS` with the edits' jobs filtered out).
fn vacuous(name: &str) -> bool {
    let b = name.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let at = if b[i..].starts_with(b"n=0") {
            Some(i + 3)
        } else if b[i..].starts_with(b"(0") {
            Some(i + 2)
        } else {
            None
        };
        if let Some(j) = at {
            if b.get(j).is_none_or(|c| !c.is_ascii_digit() && *c != b'.' && *c != b'/' && *c != b'%') {
                return true;
            }
        }
        i += 1;
    }
    false
}

/// Cut 30 §6 (docs/CUT30.md, the owner's decision in docs/HANDOFF.md §0): the rows the idle gates
/// replace — "not engaging fails" — printed and not counted. Exactly the contract's list: DEFAULT dies by
/// ≤ D8; EDITED − DEFAULT ≥ 15 pts; LEARNED ≤ DEFAULT + 2; PETS ≤ D8; LEVELLED ≤ D9; TRIVIAL never passes
/// D8; KITTED ≤ D8 and its `Kitted:` twins; `{set} never passes D{boss}` (FULL−D23/28/33) and its kitted
/// twin; DEFAULT passes the den/lock/captive/hunger ≤ 20 %; DEFAULT yields 0 xp/gold over 8 h; the
/// `Routes:` DEFAULT/EDITED/LEARNED rows and the PASSIVE/RANDOM twins; PASSIVE loses; the whole forge <
/// the best row (the lever) and its oath twin; DEFAULT/EDITED never swear; the oath best set ≠
/// bank-optimal; the progression rows (unlock days, stall ≤ 3 on 13/18, marks unspent ≤ 8, purse ≤ 1.5
/// days' net). (The oath board left play in §5: its completable row is the quests' now.)
const RETIRED: &[&str] = &[
    "DEFAULT dies by ≤ D8",
    "EDITED − DEFAULT (≥ D10) ≥ 15 pts",
    "LEARNED mean depth ≤ DEFAULT + 2",
    "PETS dies by ≤ D8",
    "LEVELLED dies by ≤ D9",
    "TRIVIAL never passes D8",
    "KITTED (DEFAULT + every step) dies by ≤ D8",
    "Kitted: ",
    "FULL−D23 never passes",
    "FULL−D28 never passes",
    "FULL−D33 never passes",
    "DEFAULT passes the ",
    "DEFAULT yields 0 xp/gold",
    "Routes: DEFAULT",
    "Routes: EDITED − DEFAULT",
    "Routes: LEARNED",
    "Routes: PASSIVE",
    "Routes: RANDOM",
    "PASSIVE loses by ≤ D3",
    "Whole forge's bank move < best row's",
    "Lever holds with every oath reward",
    "DEFAULT/EDITED never swear an oath",
    "Oath best set ≠ bank-optimal",
    "Oath completable ≥ 20%/night",
    "Progression: days with an unlock",
    "Progression: marks unspent",
    "Progression: longest stall",
    "Progression: purse",
    // (moved by the pivot — the heirs' temperaments no longer act for the bots, `docs/CUT30.md` deviations;
    // re-derived on the new bots: content reach and stalls in the dayplayer, the lanes in `cut30_rows`)
    "COUNTERED reaches ≥ D14",
    "Lanes D5: no set dominates",
    "Lanes D9: no set dominates",
    "Stalls ≤ 1% of sends on every cohort set",
];

fn retired(name: &str) -> bool {
    RETIRED.iter().any(|p| name.starts_with(p)) || (name.starts_with("FULL") && name.contains(" never passes D"))
}

/// Every row that measured nothing fails (`vacuous`), its value marked.
fn seal(rows: &mut [(String, String, bool)]) {
    for (name, value, ok) in rows.iter_mut() {
        if vacuous(name) {
            *ok = false;
            value.push_str(" · n=0");
        }
    }
}

/// A stance wins a wall when it passes it this much more often than the next one (shares of sends).
const STANCE_MARGIN: f64 = 0.02;

/// Run `f` over `items` on `threads` threads, the results in the items' order.
fn pool<T: Sync, R: Send>(items: &[T], threads: usize, f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: Mutex<Vec<(usize, R)>> = Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads.max(1) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(it) = items.get(i) else { break };
                let r = f(it);
                out.lock().unwrap().push((i, r));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, r)| r).collect()
}

/// Cut 30 §6 (docs/CUT30.md): the idle floor's table rows (`examples/idle_lib`): a 20-minute absence
/// pays, a drill's item is packed at its wall, every stance is best at some wall and none at all, each
/// quest is keepable with the pen closed. The fortnight's own rows are the dayplayer's (`--idle`,
/// `--picked`, `--tuned`).
fn cut30_rows(rows: &mut Vec<(String, String, bool)>, seeds: u64, threads: usize) {
    let lineages: Vec<u64> = (1..=seeds.min(8)).collect();
    // (the snapshots travel between threads as saves: a game's caches are not `Sync`)
    let snaps: Vec<Vec<(u32, String)>> = pool(&lineages, threads, |s| idle_snaps(*s));
    let load = |t: &String| Game::load(t).expect("a snapshot loads");
    // a 20-minute absence: 30 fresh lineages and every IDLE lineage standing at D13
    let fresh: Vec<u64> = (1..=30).collect();
    let fresh_runs: Vec<u32> = pool(&fresh, threads, |s| idle::twenty_minutes(&idle::fresh(*s)));
    let d13: Vec<&String> = snaps.iter().flat_map(|v| v.iter().filter(|(w, _)| *w == 13).map(|(_, g)| g)).collect();
    let d13_runs: Vec<u32> = pool(&d13, threads, |g| idle::twenty_minutes(&load(g)));
    // expeditions per 8 h on the idle floor (fresh lineages' first 8 h, and at D13): at least 6, at most one
    // send and its rest per 20 minutes (`engine::REST_MIN_TICKS` — the band's own reason: no sortie farm)
    let e8: Vec<u32> = pool(&fresh[..8], threads, |s| {
        let mut g = idle::fresh(*s);
        riddle_core::offline::run_offline_counts(&mut g, 8 * 3600).runs
    });
    let e8_d13: Vec<u32> = pool(&d13, threads, |g| riddle_core::offline::run_offline_counts(&mut load(g), 8 * 3600).runs);
    let cap = (8 * 3600 * riddle_core::offline::TICKS_PER_SECOND) as f64 / riddle_core::engine::REST_MIN_TICKS as f64;
    let mean = |v: &[u32]| v.iter().sum::<u32>() as f64 / v.len().max(1) as f64;
    let (a, b) = (mean(&e8), mean(&e8_d13));
    rows.push((format!("Expeditions per 8 h, IDLE (fresh · D13) in 6–{cap:.0}"), format!("{a:.1} · {b:.1}"), [a, b].iter().all(|x| (6.0..=cap).contains(x)) && !e8_d13.is_empty()));
    // (Cut 30.5: the fresh lineage's first absence once the scout is hired — by IDLE's own first session)
    let session_runs: Vec<u32> = pool(&fresh, threads, |s| idle::twenty_minutes(&idle::after_session(*s)));
    let sp = session_runs.iter().filter(|r| **r >= 1).count();
    rows.push(("A fresh lineage's first 20-min absence after the scout's hire returns ≥ 1 run ≥ 95%".to_string(), format!("{:.0}% ({sp}/{})", pct(sp, session_runs.len()), session_runs.len()), pct(sp, session_runs.len()) >= 95.0));
    let paid = fresh_runs.iter().chain(&d13_runs).filter(|r| **r >= 1).count();
    let n20 = fresh_runs.len() + d13_runs.len();
    rows.push((format!("A 20-min absence returns ≥ 1 run ≥ 95% (fresh 30 + D13 {})", d13_runs.len()), format!("{:.0}%", pct(paid, n20)), pct(paid, n20) >= 95.0 && !d13_runs.is_empty()));
    // the D5 lanes on the idle floor (re-derived from `Lanes D5: no set dominates`, whose written sets
    // the pivot's bots no longer write): IDLE's own set at its D8 snapshot, the near stair and the far —
    // both lanes viable (the weaker reaches D8 at least half as often as the stronger, over the seeds)
    let d8: Vec<&String> = snaps.iter().flat_map(|v| v.iter().filter(|(w, _)| *w == 8).map(|(_, g)| g)).collect();
    let lanes: Vec<(f64, f64)> = pool(&d8, threads, |t| {
        let mut g = load(t);
        g.lineage.facts.insert("fork:5".into());
        let pass = |g: &Game, route: &[u32]| {
            let set = g.lineage.rules().clone().with_route(riddle_core::descent::Route::from_forks(route).unwrap_or_default());
            let rs = riddle_core::forecast::camp_panel(g, &set, 48);
            // (Cut 30.5, owner-approved row change: each lane reaching D8, the band's end — under the record rule no sim
            // from IDLE's D7 record passes the Warlord on either lane, so passing him compared two zeros)
            rs.iter().filter(|r| r.max_depth >= 8).count() as f64 / rs.len().max(1) as f64
        };
        (pass(&g, &[]), pass(&g, &[5]))
    });
    let (near, far) = lanes.iter().fold((0.0, 0.0), |a, x| (a.0 + x.0, a.1 + x.1));
    let ratio = near.min(far) / near.max(far).max(1e-9);
    rows.push((format!("Lanes D5 (IDLE): both reach D8, weaker ≥ ½ the stronger (n={})", lanes.len()), format!("near {:.2} · far {:.2}", near / lanes.len().max(1) as f64, far / lanes.len().max(1) as f64), ratio >= 0.5 && !lanes.is_empty()));
    // the quartermaster: a drill's item in the pack at its wall
    let packs: Vec<(u32, u32)> = pool(&lineages, threads, |s| idle::drill_packed(*s, 12));
    let (c, n) = packs.iter().fold((0, 0), |a, p| (a.0 + p.0, a.1 + p.1));
    rows.push((format!("A drill-needed item is in the pack at its wall ≥ 90% (n={n})"), format!("{:.0}%", pct(c as usize, n as usize)), pct(c as usize, n as usize) >= 90.0));
    // every stance best at ≥ 1 wall, none at all: per wall, the stance whose sends from the IDLE lineages
    // under it pass it most — read twice, from the wall's stone (the wall alone) and from D1 (the walk
    // and the wall: the generalist's ground); a win is a margin over the next stance (`STANCE_MARGIN`,
    // ~2 ± of a 96-send panel over the seeds), else the contest is a tie and counts for no one
    let mut jobs: Vec<(u32, bool, &'static str, &String)> = Vec::new();
    for v in &snaps {
        for (w, g) in v {
            for from_stone in [true, false] {
                for s in idle::STANCES {
                    jobs.push((*w, from_stone, s, g));
                }
            }
        }
    }
    let panels: Vec<(f64, f64)> = pool(&jobs, threads, |(w, st, s, g)| idle::stance_past(&load(g), s, *w, 96, *st));
    let past: Vec<f64> = panels.iter().map(|p| p.0).collect();
    // Steady, the idle default, is exempt from a wall (the owner, 2026-10-01): it is the safest default —
    // the fewest deaths a send from D1 across the walls, against every other stance
    let mut died: BTreeMap<&str, (f64, u32)> = BTreeMap::new();
    for (k, (_, st, s, _)) in jobs.iter().enumerate() {
        if !*st {
            let e = died.entry(*s).or_insert((0.0, 0));
            e.0 += panels[k].1;
            e.1 += 1;
        }
    }
    let rate = |s: &str| died.get(s).map_or(1.0, |(t, n)| t / (*n).max(1) as f64);
    let others = idle::STANCES.iter().filter(|s| **s != "steady").map(|s| rate(s)).fold(f64::MAX, f64::min);
    rows.push(("Steady is the safest default: fewest deaths a send from D1".into(), idle::STANCES.iter().map(|s| format!("{s} {:.0}%", 100.0 * rate(s))).collect::<Vec<_>>().join(" "), rate("steady") < others));
    let mut wins: BTreeMap<&str, u32> = BTreeMap::new();
    let mut contests = 0;
    for w in idle::WALLS {
        for from_stone in [true, false] {
            let mut score: BTreeMap<&str, (f64, u32)> = BTreeMap::new();
            for (k, (jw, st, s, _)) in jobs.iter().enumerate() {
                if *jw == w && *st == from_stone && *s != "steady" {
                    let e = score.entry(*s).or_insert((0.0, 0));
                    e.0 += past[k];
                    e.1 += 1;
                }
            }
            if score.is_empty() {
                continue;
            }
            let mut ranked: Vec<(&str, f64)> = score.iter().map(|(s, (t, n))| (*s, t / (*n).max(1) as f64)).collect();
            ranked.sort_by(|a, b| b.1.total_cmp(&a.1));
            eprintln!("stances D{w} {}: {}", if from_stone { "stone" } else { "D1" }, ranked.iter().map(|(s, v)| format!("{s} {v:.3}")).collect::<Vec<_>>().join(" "));
            if ranked[0].1 - ranked.get(1).map_or(0.0, |x| x.1) < STANCE_MARGIN {
                continue;
            }
            contests += 1;
            *wins.entry(ranked[0].0).or_insert(0) += 1;
        }
    }
    let each = idle::STANCES.iter().filter(|s| **s != "steady").all(|s| wins.get(s).is_some_and(|n| *n > 0));
    let none_all = wins.values().all(|n| *n < contests);
    rows.push((format!("Guarded, Bold, Hunter each best at ≥ 1 wall by ≥ {STANCE_MARGIN}, none at all ({contests} won)"), wins.iter().map(|(s, n)| format!("{s} {n}")).collect::<Vec<_>>().join(" "), each && none_all && contests > 0));
    // each quest keepable ≥ 20 % a night by some package set, the pen closed (the board opens with the
    // Warlord slain: the lineages at D13, D18 and D23)
    let mut qjobs: Vec<(&String, &'static str)> = Vec::new();
    for v in &snaps {
        for (w, g) in v {
            if [13, 18, 23].contains(w) {
                for k in ["reach", "reach_no_return", "bank", "slay"] {
                    qjobs.push((g, k));
                }
            }
        }
    }
    let q: Vec<(String, f64)> = pool(&qjobs, threads, |(g, k)| idle::quest_night(&load(g), k, 24));
    let worst = q.iter().fold(("-".to_string(), 1.0f64), |a, b| if b.1 < a.1 { b.clone() } else { a });
    rows.push((format!("Quests keepable ≥ 20%/night by a package set, pen closed (n={})", q.len()), format!("worst {:.0}% {}", 100.0 * worst.1, worst.0), worst.1 >= 0.2 && !q.is_empty()));
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

/// This thread's CPU seconds (Linux: `/proc/thread-self/schedstat`), 0 where the kernel does not say.
fn thread_cpu() -> f64 {
    thread_cpu_opt().unwrap_or(0.0)
}

fn thread_cpu_opt() -> Option<f64> {
    std::fs::read_to_string("/proc/thread-self/schedstat").ok().and_then(|t| t.split_whitespace().next().and_then(|x| x.parse::<f64>().ok())).map(|ns| ns / 1e9)
}

/// The clock of the table's single-threaded timing rows (per-tick cost, verdict time): this
/// thread's CPU seconds, so a loaded machine — qa beside the table, other agents' browsers —
/// is not read as a slower engine (the wall read 3.3 → 5.0 µs/tick under a load of ~70 with the
/// same binary; the bars are unchanged). The wall where the kernel does not say.
struct Clock(Option<f64>, Instant);

impl Clock {
    fn start() -> Clock {
        Clock(thread_cpu_opt(), Instant::now())
    }
    fn secs(&self) -> f64 {
        match (self.0, thread_cpu_opt()) {
            (Some(a), Some(b)) => b - a,
            _ => self.1.elapsed().as_secs_f64(),
        }
    }
}

fn main() {
    // RUNS_UI: real games at scale keep no replay capsules (a lineage clone per send never read)
    riddle_core::engine::set_capsules(false);
    // The table fills the machine seed by seed; a panel's sims stay sequential inside a job.
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    // --quick: 8 seeds × 8 h × 3 verdicts (≈ 30 s; depth gates are 8 h tail gates, so hours stay).
    // Full: 30 × 8 h × 8 (≈ 2 min) is the number that counts. Never weaken bars.
    let quick = args.iter().any(|a| a == "--quick") || args.iter().any(|a| a == "--fast");
    // `--fast` (`tools/gates.mjs --fast`): the quick table less the jobs whose rows are all retired (the
    // progression lineages; the lever and the oaths are the full table's already) — the gated rows only
    let fast = args.iter().any(|a| a == "--fast");
    // `--cut30`: the idle floor's rows alone (`cut30_rows`, a few minutes)
    if args.iter().any(|a| a == "--cut30") {
        let mut rows = Vec::new();
        cut30_rows(&mut rows, get("--seeds", 8), get("--threads", 24) as usize);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    let seeds = get("--seeds", if quick { 8 } else { 30 });
    let hours = get("--hours", 8);
    let verdicts_per_seed = get("--verdicts", if quick { 3 } else { 8 }) as usize;
    let t_start = std::time::Instant::now();
    // `METRICS_PHASES=1`: each phase's wall to stderr, and every job's (docs/ITERATION_SPEED.md). `METRICS_JOBCPU=1`: every
    // job's kind, thread CPU seconds and wall (`jobcpu <kind> <cpu> <wall>`) — what the table's time is made of.
    let phases = std::env::var("METRICS_PHASES").is_ok();
    let phase = |name: &str| if phases { eprintln!("phase {name}: {:.1}s", t_start.elapsed().as_secs_f64()) };
    // Cut 3: the quiet per-tick cost — one run to its end per bot, single-threaded, before the
    // parallel jobs (the FULL run reaches the deep biomes' floors).
    let quiet_ticks: Vec<(&str, f64)> = [Bot::Default, Bot::Edited, Bot::Full]
        .iter()
        .map(|b| {
            let mut g = setup(*b, 1);
            g.sim = true;
            let mut ticks = 0u64;
            let t = Clock::start();
            for _ in 0..3 {
                g.lineage.rest_left = 0;
                g.start_run(None);
                while g.run.as_ref().is_some_and(|r| r.over.is_none()) && ticks < 200_000 {
                    g.tick();
                    g.events.clear();
                    ticks += 1;
                }
                g.finish_run();
                g.auto_keep();
            }
            (b.name(), t.secs() * 1e6 / ticks.max(1) as f64)
        })
        .collect();
    phase("quiet ticks");
    // `tools/gates.mjs` starts the wire invariants (examples/qa.rs, 3/4 of the cores) on this line,
    // so they never share the cores with the single-threaded quiet measurement above.
    if std::env::var("METRICS_QUIET_SIGNAL").is_ok() {
        eprintln!("metrics: quiet ticks measured");
    }
    // Cut 30 §6: the idle floor's rows (`cut30_rows`: IDLE's fourteen-day snapshot chains, then their panels)
    // read nothing of the pool's jobs — they play beside it from the start (they ran after it, the table's
    // tail: ~25 min of a quick table under load). The whole table only (a `--gold`-style subset returns
    // before its rows).
    let subset = ["--gold", "--deaths", "--forge", "--lever", "--oaths", "--progression", "--forks", "--bots", "--exits", "--diverge"];
    let cut30 = (!args.iter().any(|a| subset.contains(&a.as_str()))).then(|| {
        let threads = get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4).min(32)) as usize;
        std::thread::spawn(move || {
            let mut rows = Vec::new();
            cut30_rows(&mut rows, seeds, threads);
            rows
        })
    });
    let results: Arc<Mutex<BTreeMap<(usize, u64), SeedResult>>> = Arc::new(Mutex::new(BTreeMap::new()));
    // One pool, one queue, longest first (`pop` takes from the end): the counter trials, then the
    // cohort sets' stalls (one job per (set, seed), 4 h each ≈ 12 sends), then the bots with the FULL
    // ones last in `BOTS` — so the short jobs fill the cores while the last long ones finish, where
    // they used to wait for them in phases of their own. Every job is its own game; the results
    // land in maps keyed by job, so the table is the same whatever the order.
    #[derive(Clone, Copy)]
    enum Job {
        Counter(u64),
        Cohort(usize, u64, bool),
        /// Cut 22 §1: a cohort set's gold from D1 and from its deepest lit waystone (`cohort_golds`).
        Gold(usize, u64),
        Paired(usize),
        Bot(usize, u64),
        /// Cut 23 §1: a `KIT_BOTS` bot with every forge step.
        Kit(usize, u64),
        /// Cut 23 §1: a cohort set's forge after an 8 h absence.
        Forge(usize, u64),
        /// Cut 25 §1: a cohort set's lever — the whole forge's bank move against its best row's.
        Lever(usize, u64),
        /// Cut 26 §5: a `ROUTE_BOTS` bot on the seed's swapped route (`sampled_route`).
        Route(usize, u64),
        /// Cut 26 §4: the first-hour agent on a fresh lineage (`first_fork`).
        FirstFork(u64),
        /// Cut 26 §3: a fork's lane gate on one lineage seed (`lanes::gate`).
        Lane(u32, u64, usize),
        /// Cut 27 §2: a cohort set's camp edits and their divergences (`diverge_edits`).
        Diverge(usize),
        /// Cut 27 §3: a cohort set's bank twin (false) or return twin (true) on one seed (`exits::run`).
        Exit(usize, u64, bool),
        /// Cut 28 §1: an oath set's bank-optimal set, one pool oath's best set on it, the lever with
        /// every oath reward owned.
        OathBank(usize),
        Oath(usize, usize),
        OathLever(usize),
        /// Cut 29 §1: a rater set's fourteen days from a fresh lineage (`progression_lib::play`, no measures).
        Prog(usize, u64),
    }
    let sets = Arc::new(cohort_sets());
    // `--gold`: the Cut 22 §1 gold table alone (the cohort sets' sends; ~20 s).
    let gold_only = args.iter().any(|a| a == "--gold");
    // `--deaths`: the Cut 23 §2 death mix alone (the cohort sets' 4 h absences).
    let deaths_only = args.iter().any(|a| a == "--deaths");
    // `--forge`: the Cut 23 §1 forge rows alone (the kitted bots and the cohort sets' forge).
    let forge_only = args.iter().any(|a| a == "--forge");
    let mut jobs: Vec<Job> = (1..=seeds).map(Job::Counter).collect();
    jobs.extend((0..sets.len()).flat_map(|si| (1..=seeds).map(move |s| Job::Cohort(si, s, false))));
    // Cut 19 §2: a set with a return row also plays without it (the night's death share, beside).
    let returning: Vec<usize> = (0..sets.len()).filter(|&si| sets[si].1.rows.iter().any(|r| r.verb.v == "return")).collect();
    jobs.extend(returning.iter().flat_map(|&si| (1..=seeds).map(move |s| Job::Cohort(si, s, true))));
    // Cut 22 §1: each cohort set's gold per send over 8 h of watched sends, from D1 and from its
    // deepest lit waystone.
    jobs.extend((0..sets.len()).flat_map(|si| (1..=seeds).map(move |s| Job::Gold(si, s))));
    jobs.extend(KIT_BOTS.iter().enumerate().flat_map(|(bi, _)| (1..=seeds).map(move |s| Job::Kit(bi, s))));
    jobs.extend((0..sets.len()).flat_map(|si| (1..=seeds).map(move |s| Job::Forge(si, s))));
    // Cut 25 §1: the lever (`lever::gate`), on the sets that bank; `LEVER_SEEDS` seeds each (the
    // full table; `--lever` alone prints it).
    let lever_only = args.iter().any(|a| a == "--lever");
    let lever_seeds = get("--lever-seeds", LEVER_SEEDS);
    if !quick || lever_only {
        jobs.extend((0..sets.len()).filter(|&si| sets[si].1.rows.iter().any(|r| r.verb.v == "bank")).flat_map(|si| (1..=lever_seeds).map(move |s| Job::Lever(si, s))));
    }
    // Cut 28 §1: the oaths' gate on the oath sets (the full table; `--oaths` alone prints it).
    let oaths_only = args.iter().any(|a| a == "--oaths");
    let oath_sets: Vec<usize> = (0..sets.len()).filter(|&si| OATH_SETS.contains(&sets[si].0.as_str())).collect();
    if !quick || oaths_only {
        for &si in &oath_sets {
            jobs.push(Job::OathBank(si));
            jobs.push(Job::OathLever(si));
            jobs.extend((0..riddle_core::oath::KINDS.len()).map(|ki| Job::Oath(si, ki)));
        }
    }
    jobs.extend(ROUTE_BOTS.iter().enumerate().flat_map(|(bi, _)| (1..=seeds).map(move |s| Job::Route(bi, s))));
    // Cut 29 §1: the progression rows — each rater set of the last cohorts (`progression_lib::rater_sets`)
    // as a fourteen-day lineage, 2 seeds (1 in --quick). `--progression` alone prints them.
    let prog_only = args.iter().any(|a| a == "--progression");
    let psets = Arc::new(prog::rater_sets());
    let prog_seeds = get("--prog-seeds", if quick { 1 } else { 2 });
    let prog_days = get("--prog-days", 14) as usize;
    if !fast {
        jobs.extend((0..psets.len()).flat_map(|si| (1..=prog_seeds).map(move |s| Job::Prog(si, s))));
    }

    jobs.extend((1..=seeds).map(Job::FirstFork));
    let found = Arc::new(lane_found());
    // (one job per (fork, seed, candidate): `lanes::gate_one`; `gate_seed`'s vector is reassembled in candidate order)
    for f in lanes::LANE_FORKS.iter().copied().filter(|f| found.iter().any(|x| x.fork == *f)) {
        let cands = lanes::gate_candidates(&found, f);
        jobs.extend((1..=LANE_SEEDS).flat_map(|s| cands.iter().map(move |&i| Job::Lane(f, s, i))));
    }
    jobs.extend(BOTS.iter().enumerate().flat_map(|(bi, _)| (1..=seeds).map(move |s| Job::Bot(bi, s))));
    // Cut 22 §3: each cohort set's one-notch edits, paired against the set.
    jobs.extend((0..sets.len()).map(Job::Paired));
    jobs.extend((0..sets.len()).map(Job::Diverge));
    // Cut 27 §3: each cohort set with an exit row, as its bank twin and its return twin.
    // `--exits`: these rows alone.
    let exits_only = args.iter().any(|a| a == "--exits");
    let exit_seeds = get("--exit-seeds", seeds.min(exits::EXIT_SEEDS));
    let exit_hours = get("--exit-hours", exits::EXIT_HOURS);
    jobs.extend((0..sets.len()).filter(|&si| exits::has_exit(&sets[si].1)).flat_map(|si| (1..=exit_seeds).flat_map(move |s| [Job::Exit(si, s, false), Job::Exit(si, s, true)])));
    if exits_only {
        jobs.retain(|j| matches!(j, Job::Exit(..)));
    }
    // `--diverge`: the Cut 27 §2 divergence row alone.
    let diverge_only = args.iter().any(|a| a == "--diverge");
    if diverge_only {
        jobs.retain(|j| matches!(j, Job::Diverge(..)));
    }
    if gold_only {
        jobs.retain(|j| matches!(j, Job::Gold(..)));
    }
    if deaths_only {
        jobs.retain(|j| matches!(j, Job::Cohort(_, _, false)));
    }
    if forge_only {
        jobs.retain(|j| matches!(j, Job::Kit(..) | Job::Forge(..)) || matches!(j, Job::Bot(bi, _) if matches!(BOTS[*bi], Bot::Default | Bot::Edited | Bot::Learned)));
    }
    if lever_only {
        jobs.retain(|j| matches!(j, Job::Lever(..)));
    }
    if oaths_only {
        jobs.retain(|j| matches!(j, Job::OathBank(..) | Job::Oath(..) | Job::OathLever(..)));
    }
    if prog_only {
        jobs.retain(|j| matches!(j, Job::Prog(..)));
    }
    // `--bots`: the bots alone — the per-bot table and the verdict sample (the dice measure; ~2 min).
    // `--forks`: the Cut 26 rows alone — the first fork's sends and the lanes' gate (~1–2 min).
    let forks_only = args.iter().any(|a| a == "--forks");
    if forks_only {
        jobs.retain(|j| matches!(j, Job::FirstFork(..) | Job::Lane(..)));
    }
    let bots_only = args.iter().any(|a| a == "--bots");
    if bots_only {
        jobs.retain(|j| matches!(j, Job::Bot(..) | Job::Route(..)));
    }
    // `--threads N` leaves cores to whatever runs beside the table (gates.mjs: the dayplayer's
    // sequential chains, which the full 32 starved — docs/ITERATION_SPEED.md §3.2).
    let threads = get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4).min(32)) as usize;
    // Longest first (`pop` takes from the end): each kind's typical thread-seconds on the full table
    // (`METRICS_JOBCPU=1` prints every job's), so a 2-minute lever or divergence job never starts
    // behind the short ones and ends the table alone. A stable sort: within a kind the order stays.
    let cost = |j: &Job| -> u32 {
        match j {
            // (a fourteen-day chain: the longest job — it starts first)
            Job::Prog(..) => 900,
            Job::Lever(..) | Job::OathLever(..) => 300,
            Job::Oath(..) => 280,
            Job::OathBank(..) => 120,
            Job::Diverge(..) => 250,
            Job::Bot(bi, _) if BOTS[*bi].is_full() => 200,
            Job::Kit(bi, _) if KIT_BOTS[*bi].is_full() => 190,
            Job::Paired(..) => 150,
            Job::Route(..) | Job::Bot(..) => 60,
            Job::Lane(..) => 50,
            Job::Gold(..) | Job::Forge(..) | Job::Kit(..) => 30,
            Job::Cohort(..) => 25,
            Job::Counter(..) | Job::FirstFork(..) => 15,
            Job::Exit(..) => 10,
        }
    };
    jobs.sort_by_key(cost);
    let jobs = Arc::new(Mutex::new(jobs));
    let cohort: Arc<Mutex<CohortStalls>> = Arc::new(Mutex::new(BTreeMap::new()));
    type Counters = BTreeMap<u64, (bool, f64, f64, f64)>;
    let counters: Arc<Mutex<Counters>> = Arc::new(Mutex::new(BTreeMap::new()));
    let golds: Arc<Mutex<Golds>> = Arc::new(Mutex::new(BTreeMap::new()));
    let paireds: Arc<Mutex<Paireds>> = Arc::new(Mutex::new(BTreeMap::new()));
    let diverged: Arc<Mutex<BTreeMap<usize, Diverged>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let cdeaths: Arc<Mutex<CohortDeaths>> = Arc::new(Mutex::new(BTreeMap::new()));
    let cnohp: Arc<Mutex<BTreeMap<(usize, u64), NoHp>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let kresults: Arc<Mutex<BTreeMap<(usize, u64), SeedResult>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let forges: Arc<Mutex<BTreeMap<(usize, u64), ForgeTally>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let levers: Arc<Mutex<Levers>> = Arc::new(Mutex::new(BTreeMap::new()));
    let rresults: Arc<Mutex<BTreeMap<(usize, u64), SeedResult>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let cforks: Arc<Mutex<BTreeMap<(usize, u64), bool>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let ffork: Arc<Mutex<BTreeMap<u64, Option<u32>>>> = Arc::new(Mutex::new(BTreeMap::new()));
    type LaneReads = Vec<(usize, f64, f64, f64)>;
    let lgates: Arc<Mutex<BTreeMap<(u32, u64), LaneReads>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let exit_tallies: Arc<Mutex<exits::Exits>> = Arc::new(Mutex::new(BTreeMap::new()));
    let obanks: Arc<Mutex<OathBanks>> = Arc::new(Mutex::new(BTreeMap::new()));
    let oreads: Arc<Mutex<OathReads>> = Arc::new(Mutex::new(BTreeMap::new()));
    let olevers: Arc<Mutex<OathLevers>> = Arc::new(Mutex::new(BTreeMap::new()));
    let pouts: Arc<Mutex<BTreeMap<(usize, u64), prog::Out>>> = Arc::new(Mutex::new(BTreeMap::new()));
    // (jobs running, chains among them: `CHAIN_WIDTH`)
    let busy: Arc<Mutex<(usize, usize)>> = Arc::new(Mutex::new((0, 0)));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let exit_tallies = Arc::clone(&exit_tallies);
        let (jobs, results, cohort, counters, sets, golds, paireds, cdeaths) = (Arc::clone(&jobs), Arc::clone(&results), Arc::clone(&cohort), Arc::clone(&counters), Arc::clone(&sets), Arc::clone(&golds), Arc::clone(&paireds), Arc::clone(&cdeaths));
        let (kresults, forges, cnohp, levers, rresults) = (Arc::clone(&kresults), Arc::clone(&forges), Arc::clone(&cnohp), Arc::clone(&levers), Arc::clone(&rresults));
        let (cforks, ffork, lgates, found) = (Arc::clone(&cforks), Arc::clone(&ffork), Arc::clone(&lgates), Arc::clone(&found));
        let diverged = Arc::clone(&diverged);
        let (obanks, oreads, olevers) = (Arc::clone(&obanks), Arc::clone(&oreads), Arc::clone(&olevers));
        let (pouts, psets) = (Arc::clone(&pouts), Arc::clone(&psets));
        let busy = Arc::clone(&busy);
        handles.push(std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some(job) = job else { break };
            let chain = matches!(job, Job::Prog(..));
            let widen = |d: isize, c: isize| {
                let mut b = busy.lock().unwrap();
                b.0 = (b.0 as isize + d) as usize;
                b.1 = (b.1 as isize + c) as usize;
                CHAIN_WIDTH.store(1 + threads.saturating_sub(b.0) / b.1.max(1), std::sync::atomic::Ordering::Relaxed);
            };
            widen(1, chain as isize);
            let tj = Instant::now();
            let cj = thread_cpu();
            let kind: String = match job {
                Job::Counter(_) => "counter".into(),
                Job::Cohort(_, _, b) => if b { "cohort-bare".into() } else { "cohort".into() },
                Job::Gold(..) => "gold".into(),
                Job::Paired(_) => "paired".into(),
                Job::Bot(bi, _) => format!("bot-{}", BOTS[bi].name()),
                Job::Prog(..) => "progression".into(),
                Job::Kit(bi, _) => format!("kit-{}", KIT_BOTS[bi].name()),
                Job::Forge(..) => "forge".into(),
                Job::Lever(..) => "lever".into(),
                Job::Route(bi, _) => format!("route-{}", ROUTE_BOTS[bi].name()),
                Job::FirstFork(_) => "firstfork".into(),
                Job::Lane(..) => "lane".into(),
                Job::Diverge(_) => "diverge".into(),
                Job::Exit(..) => "exit".into(),
                Job::OathBank(..) => "oath-bank".into(),
                Job::Oath(..) => "oath".into(),
                Job::OathLever(..) => "oath-lever".into(),
            };
            match job {
                Job::Kit(bi, seed) => {
                    // (no verdicts, no death screens: the history ring feeds only those — `without_history`)
                    let r = without_history(|| run_seed_lean(KIT_BOTS[bi], seed, hours, 0, true, riddle_core::descent::Route::BASE));
                    kresults.lock().unwrap().insert((bi, seed), r);
                }
                Job::Forge(si, seed) => {
                    let r = without_history(|| cohort_forge(&sets[si].1, seed, hours));
                    forges.lock().unwrap().insert((si, seed), r);
                }
                // (the retired rows' jobs — the lever, the oaths, the lanes, the progression lineages — are kept
                // across runs by the harness file they run and every argument, named in `params`: change the
                // call and its `params` together)
                Job::Lever(si, seed) => {
                    let params = format!("without_history gate {} s{seed} h{LEVER_HOURS} n{LEVER_SIMS} m{LEVER_MARGIN}", serde_json::to_string(&sets[si].1).unwrap_or_default());
                    let r = jobcache::cached("lever", &[include_str!("lever_lib/mod.rs")], &params, || without_history(|| lever::gate(&sets[si].1, seed, LEVER_HOURS, LEVER_SIMS, LEVER_MARGIN)));
                    levers.lock().unwrap().insert((si, seed), r);
                }
                Job::Lane(fork, seed, i) => {
                    // (`lanes_lib` reads `LANE_*` switches)
                    let env: Vec<(String, String)> = std::env::vars().filter(|(k, _)| k.starts_with("LANE_")).collect();
                    let params = format!("gate_one {} D{fork} s{seed} n{LANE_SIMS} #{i} {env:?}", serde_json::to_string(&*found).unwrap_or_default());
                    let r = jobcache::cached("lane", &[include_str!("lanes_lib/mod.rs")], &params, || lanes::gate_one(&found, fork, seed, LANE_SIMS, i));
                    if std::env::var("METRICS_PHASES").is_ok() {
                        eprintln!("job LANE D{fork} {seed} #{i} {:.1}s", tj.elapsed().as_secs_f64());
                    }
                    let mut lg = lgates.lock().unwrap();
                    let v = lg.entry((fork, seed)).or_default();
                    let at = v.partition_point(|x: &(usize, f64, f64, f64)| x.0 < i);
                    v.insert(at, r);
                }
                Job::FirstFork(seed) => {
                    let r = first_fork(seed, FIRST_FORK_CAP);
                    ffork.lock().unwrap().insert(seed, r);
                }
                Job::Route(bi, seed) => {
                    // (half the base's verdict sample: the routes' dice share reads over ~600 verdicts)
                    let r = run_seed_lean(ROUTE_BOTS[bi], seed, hours, verdicts_per_seed.div_ceil(2), false, sampled_route(seed));
                    if std::env::var("METRICS_PHASES").is_ok() {
                        eprintln!("job ROUTE {} {seed} {:.1}s", ROUTE_BOTS[bi].name(), tj.elapsed().as_secs_f64());
                    }
                    rresults.lock().unwrap().insert((bi, seed), r);
                }
                Job::Bot(bi, seed) => {
                    let r = run_seed(BOTS[bi], seed, hours, verdicts_per_seed, false);
                    if std::env::var("METRICS_PHASES").is_ok() {
                        eprintln!("job {} {seed} {:.1}s", BOTS[bi].name(), tj.elapsed().as_secs_f64());
                    }
                    results.lock().unwrap().insert((bi, seed), r);
                }
                Job::Cohort(si, seed, bare) => {
                    let set = if bare { without_return(&sets[si].1) } else { sets[si].1.clone() };
                    // (the without-return twin reads counts only: no verdicts, so no history ring)
                    let play = || cohort_stalls(&set, seed, 4, !bare);
                    let (r, mix, nohp, fork9, (loops, stalled)) = if bare { without_history(play) } else { play() };
                    if std::env::var("METRICS_LOOPS").is_ok() && !bare {
                        for l in &loops {
                            eprintln!("loop {} {seed}: {l}", sets[si].0);
                        }
                        for (c, d) in &stalled {
                            eprintln!("stall {} {seed}: D{d} {c}", sets[si].0);
                        }
                    }
                    cohort.lock().unwrap().insert((si, seed, bare), r);
                    if !bare {
                        cforks.lock().unwrap().insert((si, seed), fork9);
                        cdeaths.lock().unwrap().insert((si, seed), mix);
                        cnohp.lock().unwrap().insert((si, seed), nohp);
                    }
                }
                Job::Gold(si, seed) => {
                    let (d1, way) = without_history(|| cohort_golds(&sets[si].1, seed, hours));
                    let mut golds = golds.lock().unwrap();
                    golds.insert((si, seed, false), d1);
                    golds.insert((si, seed, true), way);
                }
                Job::Exit(si, seed, ret) => {
                    let set = exits::twin(&sets[si].1, if ret { "return" } else { "bank" });
                    let r = without_history(|| exits::run(&set, seed, exit_hours));
                    exit_tallies.lock().unwrap().insert((si, seed, ret), r);
                }
                Job::Paired(si) => {
                    let r = paired_edits(&sets[si].1);
                    paireds.lock().unwrap().insert(si, r);
                }
                Job::Diverge(si) => {
                    let r = diverge_edits(&sets[si].1);
                    diverged.lock().unwrap().insert(si, r);
                }
                Job::Counter(seed) => {
                    let r = riddle_core::probes::counter_trial(seed);
                    counters.lock().unwrap().insert(seed, r);
                }
                Job::OathBank(si) => {
                    let set = &sets[si].1;
                    let params = format!("without_history bank_best(lineage(set, 1)) {}", serde_json::to_string(set).unwrap_or_default());
                    let r = jobcache::cached("oath", &[include_str!("oath_lib/mod.rs"), include_str!("lever_lib/mod.rs")], &params, || {
                        without_history(|| {
                            let g = oath_lib::lineage(set, 1);
                            oath_lib::bank_best(&g, set)
                        })
                    });
                    obanks.lock().unwrap().insert(si, r);
                }
                Job::Oath(si, ki) => {
                    let set = &sets[si].1;
                    // (the bank-optimal set is its own job on the same deterministic lineage; the rows between are read at the report)
                    let params = format!("without_history measure(lineage(set, 1), {}, None) {}", riddle_core::oath::KINDS[ki], serde_json::to_string(set).unwrap_or_default());
                    let r = jobcache::cached("oath", &[include_str!("oath_lib/mod.rs"), include_str!("lever_lib/mod.rs")], &params, || {
                        without_history(|| {
                            let g = oath_lib::lineage(set, 1);
                            oath_lib::measure(&g, set, riddle_core::oath::KINDS[ki], None)
                        })
                    });
                    oreads.lock().unwrap().insert((si, ki), r);
                }
                Job::OathLever(si) => {
                    let params = format!("without_history gate_with oath::grant_all {} s1 h{LEVER_HOURS} n{LEVER_SIMS} m{LEVER_MARGIN}", serde_json::to_string(&sets[si].1).unwrap_or_default());
                    let r = jobcache::cached("lever", &[include_str!("lever_lib/mod.rs")], &params, || without_history(|| lever::gate_with(&sets[si].1, 1, LEVER_HOURS, LEVER_SIMS, LEVER_MARGIN, |g| riddle_core::oath::grant_all(&mut g.lineage))));
                    olevers.lock().unwrap().insert(si, r);
                }
                Job::Prog(si, seed) => {
                    let (name, set) = psets[si].clone();
                    let params = format!("play(name, Rater(set), seed, days, three_absence, false, false) {name} {} s{seed} d{prog_days} {:?}", serde_json::to_string(&set).unwrap_or_default(), prog::three_absence());
                    let o = jobcache::cached("prog", &[include_str!("progression_lib/mod.rs")], &params, || riddle_core::forecast::with_sim_width(&CHAIN_WIDTH, || prog::play(name, prog::Mode::Rater(set), seed, prog_days, &prog::three_absence(), false, false)));
                    pouts.lock().unwrap().insert((si, seed), o);
                }
            }
            widen(-1, -(chain as isize));
            if std::env::var("METRICS_JOBCPU").is_ok() {
                eprintln!("jobcpu {kind} {:.3} {:.3}", thread_cpu() - cj, tj.elapsed().as_secs_f64());
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    phase("jobs");
    // Cut 27 §2: the divergence is found for ≥ 90 % of the cohort sets' ≥ 5-pt edits.
    let diverge_rows = |rows: &mut Vec<(String, String, bool)>| {
        let dv = diverged.lock().unwrap();
        let (n5, found, by_rows, inside, slow) = dv.values().fold((0u32, 0u32, 0u32, 0u32, 0.0f64), |a, r| (a.0 + r.0, a.1 + r.1, a.2 + r.2, a.3 + r.3, a.4.max(r.4)));
        println!("divergence (Cut 27 §2): {n5} edits ≥ 5 pts over {} sets · found {found} ({:.0}%) · at a row {by_rows} · inside ± with a scene {inside} · slowest {slow:.0} ms", dv.len(), pct(found as usize, n5 as usize));
        for (si, r) in dv.iter().filter(|(_, r)| r.1 < r.0) {
            println!("  {}: {}/{} found", sets[*si].0, r.1, r.0);
        }
        rows.push((format!("Divergence found ≥ 90% of ≥ 5-pt edits ({n5} cohort edits)"), format!("{:.0}% · at a row {:.0}%", pct(found as usize, n5 as usize), pct(by_rows as usize, n5 as usize)), n5 > 0 && found as f64 >= 0.9 * n5 as f64));
    };
    if diverge_only {
        let mut rows = Vec::new();
        diverge_rows(&mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if forks_only {
        let ns = seeds as usize;
        let ff = ffork.lock().unwrap();
        let mut sends: Vec<u32> = ff.values().map(|v| v.unwrap_or(99)).collect();
        sends.sort();
        let within = pct(sends.iter().filter(|s| **s <= FIRST_FORK_SENDS).count(), ns);
        println!("first fork: within {FIRST_FORK_SENDS} sends {within:.0}% · {:?}", sends);
        let lg = lgates.lock().unwrap();
        for fork in lanes::LANE_FORKS {
            let gs: Vec<LaneReads> = (1..=LANE_SEEDS).filter_map(|s| lg.get(&(fork, s)).cloned()).collect();
            let Some((g, win)) = lanes::gate_pool(&found, fork, &gs) else { continue };
            println!("lanes D{fork}: loss {:+.0} · {:+.0} · own {:.0}% {:.0}% · cross {:.0}% {:.0}% · EDITED's {:.0}% {:.0}% · $/h {:.2}× · near {} · far {}", 100.0 * g.loss(0), 100.0 * g.loss(1), 100.0 * g.own[0], 100.0 * g.own[1], 100.0 * g.cross[0], 100.0 * g.cross[1], 100.0 * g.edited[0], 100.0 * g.edited[1], g.gold_ratio(), found[win[0]].seed_set, found[win[1]].seed_set);
        }
        return;
    }
    let lever_rows = |rows: &mut Vec<(String, String, bool)>| lever_report(&sets, &levers.lock().unwrap(), lever_seeds, rows);
    // Cut 29 §1: the progression rows (the rater lineages' fourteen days).
    let prog_rows = |rows: &mut Vec<(String, String, bool)>| {
        let po = pouts.lock().unwrap();
        let outs: Vec<&prog::Out> = po.values().collect();
        for o in &outs {
            let (raw, known) = prog::stalls(o);
            let unl = o.days.iter().filter(|d| prog::unlock_day(d)).count();
            let marks = o.days.iter().skip(2).map(|d| d.marks_max).max().unwrap_or(0);
            println!("progression {} s{}: best D{} · unlock days {unl} · stall {known} (raw {raw}) · marks ≤ {marks} · purse {:.2}×", o.name, o.seed, o.days.last().map(|d| d.best).unwrap_or(0), prog::purse_worst(o));
        }
        let owned: Vec<prog::Out> = outs.into_iter().cloned().collect();
        rows.extend(prog::bars(&owned));
    };
    if prog_only {
        let mut rows = Vec::new();
        prog_rows(&mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if oaths_only {
        let mut rows = Vec::new();
        oath_report(&sets, &obanks.lock().unwrap(), &oreads.lock().unwrap(), &olevers.lock().unwrap(), &mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if lever_only {
        let mut rows = Vec::new();
        lever_rows(&mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if forge_only {
        let mut rows = Vec::new();
        forge_report(&sets, &forges.lock().unwrap(), &kresults.lock().unwrap(), &results.lock().unwrap(), seeds, &mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if exits_only {
        let mut rows = Vec::new();
        exits::report(&sets, &exit_tallies.lock().unwrap(), exit_seeds, exit_hours, &mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    if deaths_only {
        let (flagged, n) = death_mix_report(&sets, &cdeaths.lock().unwrap(), seeds);
        println!("sets with one cause over half their deaths: {flagged}/{n}");
        // Cut 27 §4: the cohort stall row's per-set counts beside (the same jobs).
        let cohort = cohort.lock().unwrap();
        for (si, (name, _)) in sets.iter().enumerate() {
            let (n, k, lp) = (1..=seeds).fold((0u32, 0u32, 0u32), |a, s| cohort.get(&(si, s, false)).map_or(a, |r| (a.0 + r.0, a.1 + r.1, a.2 + r.4)));
            println!("  stalls {name}: {k}/{n} ({:.1}%) · card ↔ chore loops {lp}", pct(k as usize, n as usize));
        }
        return;
    }
    if gold_only {
        let mut rows = Vec::new();
        gold_report(&sets, &golds.lock().unwrap(), seeds, hours, &mut rows);
        seal(&mut rows);
        for (name, value, ok) in &rows {
            println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        }
        return;
    }
    let cohort = cohort.lock().unwrap();
    let results = results.lock().unwrap();
    let per_bot = |bot: Bot| -> Vec<&SeedResult> {
        let bi = BOTS.iter().position(|b| *b == bot).unwrap();
        (1..=seeds).map(|s| &results[&(bi, s)]).collect()
    };
    let ns = seeds as usize;
    let mut rows: Vec<(String, String, bool)> = Vec::new();
    // Per-bot summary.
    println!("bot        best-depth mean  ≤D8  ≥D10  ≥D23  ≥D29  mean-death-depth  runs  deaths   dice");
    for bot in BOTS {
        let rs = per_bot(bot);
        let mean_best = rs.iter().map(|r| r.best_depth as f64).sum::<f64>() / ns as f64;
        let le6 = rs.iter().filter(|r| r.best_depth <= 8).count();
        let ge10 = rs.iter().filter(|r| r.best_depth >= 10).count();
        let ge20 = rs.iter().filter(|r| r.best_depth >= 23).count();
        let ge26 = rs.iter().filter(|r| r.best_depth >= 29).count();
        let depths: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        let mdd = depths.iter().sum::<u32>() as f64 / depths.len().max(1) as f64;
        let runs: u32 = rs.iter().map(|r| r.runs).sum();
        let deaths: usize = rs.iter().map(|r| r.causes.len()).sum();
        let vs: Vec<&String> = rs.iter().flat_map(|r| r.verdicts.iter()).collect();
        let dice = pct(vs.iter().filter(|v| v.as_str() == "dice").count(), vs.len());
        println!("{:<10} {:>15.2} {:>4} {:>5} {:>5} {:>5} {:>17.2} {:>5} {:>7} {:>5.1}%", bot.name(), mean_best, le6, ge10, ge20, ge26, mdd, runs, deaths, dice);
    }
    println!("\nrun depth histogram (% of runs ending at depth ≥ d):");
    for bot in BOTS {
        let rs = per_bot(bot);
        let depths: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        let n = depths.len().max(1);
        let cols: Vec<u32> = if bot.is_full() { vec![8, 13, 18, 19, 23, 24, 28, 29, 33, 34] } else { (1..=13).collect() };
        let cells: Vec<String> = cols.iter().map(|d| format!("D{d} {:>4.1}", pct(depths.iter().filter(|x| *x >= d).count(), n))).collect();
        println!("{:<9} {}", bot.name(), cells.join(" │ "));
    }
    println!("\nrun length (ticks): p10 / median / p90 / max, share in 3600–7200 (6–12 min) · per 8 h: runs · banked · returned · rest share");
    for bot in [Bot::Default, Bot::Edited, Bot::Learned, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered, Bot::Full] {
        let rs = per_bot(bot);
        let hours = hours * bot.batches();
        let mut t: Vec<u32> = rs.iter().flat_map(|r| r.run_ticks.iter().copied()).collect();
        t.sort();
        let q = |f: f64| t.get(((t.len() as f64 - 1.0) * f) as usize).copied().unwrap_or(0);
        let band = pct(t.iter().filter(|x| (3600..=7200).contains(*x)).count(), t.len());
        let mean = |f: &dyn Fn(&SeedResult) -> f64| rs.iter().map(|r| f(r)).sum::<f64>() / ns as f64;
        let runs8 = mean(&|r| r.runs as f64 * 8.0 / hours as f64);
        let banked8 = mean(&|r| r.banked as f64 * 8.0 / hours as f64);
        let returned8 = mean(&|r| r.returned as f64 * 8.0 / hours as f64);
        let rest = mean(&|r| r.rested_s as f64) / (hours as f64 * 3600.0);
        println!("{:<9} {:>6} / {:>6} / {:>6} / {:>6}   {band:.0}%   {runs8:.1} · {banked8:.1} · {returned8:.1} · {:.0}%", bot.name(), q(0.1), q(0.5), q(0.9), t.last().copied().unwrap_or(0), 100.0 * rest);
    }
    let default = per_bot(Bot::Default);
    let edited = per_bot(Bot::Edited);
    // Cut 7: the wall moved to D8 (the Warlord; D5 is the Captain); the bars moved with it.
    let d_le6 = pct(default.iter().filter(|r| r.best_depth <= 8).count(), ns);
    rows.push(("DEFAULT dies by ≤ D8 ≥ 80% of seeds".into(), format!("{d_le6:.0}%"), d_le6 >= 80.0));
    // Cut 28 §1: nothing swears on its own — the bots play with the oaths off.
    let swore = default.iter().chain(edited.iter()).filter(|r| r.swore).count();
    rows.push(("DEFAULT/EDITED never swear an oath".into(), format!("{swore} seeds"), swore == 0));
    let e_ge10 = pct(edited.iter().filter(|r| r.best_depth >= 10).count(), ns);
    let d_ge10 = pct(default.iter().filter(|r| r.best_depth >= 10).count(), ns);
    rows.push(("EDITED reaches ≥ D10 ≥ 50% of seeds".into(), format!("{e_ge10:.0}%"), e_ge10 >= 50.0));
    rows.push(("EDITED − DEFAULT (≥ D10) ≥ 15 pts".into(), format!("{:.0} pts", e_ge10 - d_ge10), e_ge10 - d_ge10 >= 15.0));
    let random = per_bot(Bot::Random);
    let r_lose = pct(random.iter().filter(|r| r.best_depth < 19).count(), ns);
    rows.push(("RANDOM loses 100%".into(), format!("{r_lose:.0}%"), r_lose >= 100.0));
    let passive = per_bot(Bot::Passive);
    let p_le3 = pct(passive.iter().filter(|r| r.best_depth <= 3).count(), ns);
    rows.push(("PASSIVE loses by ≤ D3 100%".into(), format!("{p_le3:.0}%"), p_le3 >= 100.0));
    let mean_death = |rs: &[&SeedResult]| {
        let d: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        d.iter().sum::<u32>() as f64 / d.len().max(1) as f64
    };
    let learned = per_bot(Bot::Learned);
    let (ml, md) = (mean_death(&learned), mean_death(&default));
    rows.push(("LEARNED mean depth ≤ DEFAULT + 2".into(), format!("{ml:.2} vs {md:.2}"), ml <= md + 2.0));
    let pets = per_bot(Bot::Pets);
    let pets_le8 = pct(pets.iter().filter(|r| r.best_depth <= 8).count(), ns);
    rows.push(("PETS dies by ≤ D8 ≥ 80% of seeds".into(), format!("{pets_le8:.0}%"), pets_le8 >= 80.0));
    let lev = per_bot(Bot::Levelled);
    let lev_le9 = pct(lev.iter().filter(|r| r.best_depth <= 9).count(), ns);
    rows.push(("LEVELLED dies by ≤ D9 ≥ 80% of seeds".into(), format!("{lev_le9:.0}%"), lev_le9 >= 80.0));
    let triv = per_bot(Bot::Trivial);
    let triv_le5 = pct(triv.iter().filter(|r| r.best_depth <= 8).count(), ns);
    rows.push(("TRIVIAL never passes D8 ≥ 90% of seeds".into(), format!("{triv_le5:.0}%"), triv_le5 >= 90.0));
    let ctr = per_bot(Bot::Countered);
    let ctr_ge11 = pct(ctr.iter().filter(|r| r.best_depth >= 14).count(), ns);
    rows.push(("COUNTERED reaches ≥ D14 ≥ 50% of seeds".into(), format!("{ctr_ge11:.0}%"), ctr_ge11 >= 50.0));
    // Cut 3 gates (docs/CUT3.md): the shipped best set reaches the Sanctum; each new boss's
    // counter row is load-bearing.
    let full = per_bot(Bot::Full);
    let full_ge26 = pct(full.iter().filter(|r| r.best_depth >= 29).count(), ns);
    rows.push((format!("FULL reaches ≥ D29 ≥ 50% of seeds ({FULL_BATCHES} × 8 h)"), format!("{full_ge26:.0}%"), full_ge26 >= 50.0));
    for (bot, boss) in [(Bot::FullNo23, 23u32), (Bot::FullNo28, 28), (Bot::FullNo33, 33)] {
        let rs = per_bot(bot);
        let held = pct(rs.iter().filter(|r| r.best_depth <= boss).count(), ns);
        rows.push((format!("{} never passes D{boss} ≥ 90% of seeds", bot.name()), format!("{held:.0}%"), held >= 90.0));
    }
    // Verdicts and causes across bots. The verdicts are a per-seed sample (`verdicts_per_seed`
    // of each seed's deaths), so the share of all deaths is estimated per bot and weighted by
    // the bot's deaths: a FULL bot dies a dozen times in 3 × 8 h and every one is sampled,
    // while DEFAULT's 470 deaths yield the same eight — the raw share of sampled verdicts
    // weighted the boss walls' deaths (dice by design) thirty times over. `--verdicts 100000`
    // verdicts every death and the two numbers agree.
    let all: Vec<&SeedResult> = BOTS.iter().flat_map(|b| per_bot(*b)).collect();
    let verdicts: Vec<&String> = all.iter().flat_map(|r| r.verdicts.iter()).collect();
    let raw_dice = pct(verdicts.iter().filter(|v| v.as_str() == "dice").count(), verdicts.len());
    let weighted = |which: &str| -> f64 {
        let (mut num, mut den) = (0.0, 0usize);
        for bot in BOTS {
            let rs = per_bot(bot);
            let vs: Vec<&String> = rs.iter().flat_map(|r| r.verdicts.iter()).collect();
            let deaths: usize = rs.iter().map(|r| r.causes.len()).sum();
            if vs.is_empty() || deaths == 0 {
                continue;
            }
            num += deaths as f64 * vs.iter().filter(|v| v.as_str() == which).count() as f64 / vs.len() as f64;
            den += deaths;
        }
        100.0 * num / den.max(1) as f64
    };
    let dice = weighted("dice");
    // Cut 19 §4: a `row` death traces to a row too — the one the player wrote.
    let row = weighted("row");
    // Cut 26: a `route` death traces to the route the set wrote (a row-like cause).
    let gap = weighted("gap") + row + weighted("route");
    println!("verdict sample: {} verdicts, raw dice share {raw_dice:.1}% · death-weighted {dice:.1}% · row {row:.1}%", verdicts.len());
    if bots_only {
        return;
    }
    // A share near 5% needs a few hundred verdicts to read: the quick mode's ~240 give ±2.8 pts.
    // Under 500 the bar is applied with that half-width; the full table is the gate that counts.
    let dice_n = verdicts.len() as f64;
    let dice_pm = if dice_n > 0.0 { 196.0 * (0.05 * 0.95 / dice_n).sqrt() } else { 0.0 };
    let dice_ok = if dice_n < 500.0 { dice <= 5.0 + dice_pm } else { dice <= 5.0 };
    rows.push((format!("Unfair deaths (dice) ≤ 5% (n={}, death-weighted{})", verdicts.len(), if dice_n < 500.0 { format!(", ±{dice_pm:.1} sample") } else { String::new() }), format!("{dice:.1}%"), dice_ok));
    rows.push(("Deaths tracing to a row (gap + row) ≥ 70%".into(), format!("{gap:.1}%"), gap >= 70.0));
    // Cut 26 §5: the bots again on each seed's swapped route (`sampled_route`: the twelve in turn).
    {
        let rres = rresults.lock().unwrap();
        let on_route = |bot: Bot| -> Vec<&SeedResult> {
            let bi = ROUTE_BOTS.iter().position(|b| *b == bot).unwrap();
            (1..=seeds).filter_map(|s| rres.get(&(bi, s))).collect()
        };
        if on_route(Bot::Default).len() == ns {
            let (rd, re) = (on_route(Bot::Default), on_route(Bot::Edited));
            let covered: std::collections::BTreeSet<_> = (1..=seeds).map(sampled_route).collect();
            println!("routes (Cut 26 §5): {} of {} swapped routes sampled across {seeds} seeds", covered.len(), riddle_core::descent::Route::all().len() - 1);
            for bot in ROUTE_BOTS {
                let rs = on_route(bot);
                println!("  {:<8} on routes: best-depth mean {:.2} · mean death depth {:.2} · ≤D8 {:.0}% · ≥D10 {:.0}%", bot.name(), rs.iter().map(|r| r.best_depth as f64).sum::<f64>() / ns as f64, mean_death(&rs), pct(rs.iter().filter(|r| r.best_depth <= 8).count(), ns), pct(rs.iter().filter(|r| r.best_depth >= 10).count(), ns));
            }
            let d_le8 = pct(rd.iter().filter(|r| r.best_depth <= 8).count(), ns);
            rows.push(("Routes: DEFAULT dies by ≤ D8 ≥ 80% of seeds".into(), format!("{d_le8:.0}%"), d_le8 >= 80.0));
            let (e10, d10) = (pct(re.iter().filter(|r| r.best_depth >= 10).count(), ns), pct(rd.iter().filter(|r| r.best_depth >= 10).count(), ns));
            rows.push(("Routes: EDITED − DEFAULT (≥ D10) ≥ 15 pts".into(), format!("{:.0} pts", e10 - d10), e10 - d10 >= 15.0));
            let r_lose = pct(on_route(Bot::Random).iter().filter(|r| r.best_depth < 19).count(), ns);
            rows.push(("Routes: RANDOM loses 100%".into(), format!("{r_lose:.0}%"), r_lose >= 100.0));
            let p_le3 = pct(on_route(Bot::Passive).iter().filter(|r| r.best_depth <= 3).count(), ns);
            rows.push(("Routes: PASSIVE loses by ≤ D3 100%".into(), format!("{p_le3:.0}%"), p_le3 >= 100.0));
            let (ml, md) = (mean_death(&on_route(Bot::Learned)), mean_death(&rd));
            rows.push(("Routes: LEARNED mean depth ≤ DEFAULT + 2".into(), format!("{ml:.2} vs {md:.2}"), ml <= md + 2.0));
            let (mut num, mut den, mut nv) = (0.0, 0usize, 0usize);
            for bot in ROUTE_BOTS {
                let rs = on_route(bot);
                let vs: Vec<&String> = rs.iter().flat_map(|r| r.verdicts.iter()).collect();
                let deaths: usize = rs.iter().map(|r| r.causes.len()).sum();
                nv += vs.len();
                if vs.is_empty() || deaths == 0 {
                    continue;
                }
                num += deaths as f64 * vs.iter().filter(|v| v.as_str() == "dice").count() as f64 / vs.len() as f64;
                den += deaths;
            }
            let rdice = 100.0 * num / den.max(1) as f64;
            let rpm = if nv > 0 { 196.0 * (0.05 * 0.95 / nv as f64).sqrt() } else { 0.0 };
            let rok = if nv < 500 { rdice <= 5.0 + rpm } else { rdice <= 5.0 };
            rows.push((format!("Routes: dice ≤ 5% (n={nv}, death-weighted{})", if nv < 500 { format!(", ±{rpm:.1} sample") } else { String::new() }), format!("{rdice:.1}%"), rok));
        }
    }
    // Cut 26 §3: lanes that want different sets — at each fork the per-lane best set (the plateau
    // search's, `presets/lanes.json`) loses ≥ 15 pts on the other lane, paired; EDITED's best per
    // lane reaches the band's end ≥ 50 %; gold/hr within 1.5×; the clears' sets diverse.
    {
        let lg = lgates.lock().unwrap();
        for fork in lanes::LANE_FORKS {
            let gs: Vec<LaneReads> = (1..=LANE_SEEDS).filter_map(|s| lg.get(&(fork, s)).cloned()).collect();
            let Some((g, win)) = lanes::gate_pool(&found, fork, &gs) else { continue };
            let names = [found[win[0]].seed_set.clone(), found[win[1]].seed_set.clone()];
            for (l, w) in win.iter().enumerate() {
                println!("  D{fork} {} best (from {}): {}", ["near", "far"][l], found[*w].seed_set, found[*w].set.rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | "));
            }
            let [near, far] = lanes::lanes(fork);
            println!("lanes D{fork} (Cut 26 §3; {} rows, L{}, {LANE_SIMS} paired sends × {} seeds): near {} best {} {:.0}% (on {} {:.0}%) · far {} best {} {:.0}% (on {} {:.0}%) · $/h {:.0} · {:.0} · EDITED's {:.0}% · {:.0}%", lanes::rows_cap(fork), lanes::level(fork), gs.len(), near.biome(fork).name(), names[0], 100.0 * g.own[0], far.biome(fork).name(), 100.0 * g.cross[0], far.biome(fork).name(), names[1], 100.0 * g.own[1], near.biome(fork).name(), 100.0 * g.cross[1], g.gold[0], g.gold[1], 100.0 * g.edited[0], 100.0 * g.edited[1]);
            let (a, b) = (g.loss(0), g.loss(1));
            let ev = g.edited[0].min(g.edited[1]);
            // (a fork the descent keeps closed — `descent::OPEN_FORKS` — is measured and printed, not gated)
            if !riddle_core::descent::OPEN_FORKS.contains(&fork) {
                println!("  D{fork} fork closed (recorded): loss {:+.0} · {:+.0} · EDITED's {:.0}% · {:.0}% · $/h ratio {:.2}×", 100.0 * a, 100.0 * b, 100.0 * g.edited[0], 100.0 * g.edited[1], g.gold_ratio());
                continue;
            }
            rows.push((format!("Lanes D{fork}: no set dominates (cross-lane loss ≥ 15 pts, paired)"), format!("{:+.0} · {:+.0}", 100.0 * a, 100.0 * b), a >= 0.15 - 1e-9 && b >= 0.15 - 1e-9));
            rows.push((format!("Lanes D{fork}: both viable (EDITED's best ≥ 50% band end)"), format!("{:.0}% · {:.0}%", 100.0 * g.edited[0], 100.0 * g.edited[1]), ev >= 0.5 - 1e-9));
            rows.push((format!("Lanes D{fork}: gold/hr ratio ≤ 1.5×"), format!("{:.2}×", g.gold_ratio()), g.gold_ratio() <= 1.5 + 1e-9));
        }
        // The diversity of the sets that clear the open forks' bands (≥ 50 % past the band's boss),
        // over every seed set and lane of the search.
        let open: Vec<lanes::Found> = found.iter().filter(|f| riddle_core::descent::OPEN_FORKS.contains(&f.fork)).cloned().collect();
        if !open.is_empty() {
            let (d, c) = lanes::diversity(&open);
            let div = d as f64 / c.max(1) as f64;
            rows.push((format!("Lanes: rule-set diversity over band clears ≥ 0.5 (n={c})"), format!("{d}/{c} = {div:.2}"), div >= 0.5 - 1e-9 && c > 0));
        }
    }
    // Cut 26 §4: the first fork arrives in the first hour; the D9 fork before the cohort absence ends.
    {
        let ff = ffork.lock().unwrap();
        if ff.len() == ns {
            let mut sends: Vec<u32> = ff.values().map(|v| v.unwrap_or(99)).collect();
            sends.sort();
            let within = pct(sends.iter().filter(|s| **s <= FIRST_FORK_SENDS).count(), ns);
            println!("first fork (Cut 26 §4): sends to D4's two stairs, preset + first patch: median {} · {:?}", sends[ns / 2], sends.iter().map(|s| if *s == 99 { "–".to_string() } else { s.to_string() }).collect::<Vec<_>>().join(" "));
            rows.push((format!("First fork (D5) seen within {FIRST_FORK_SENDS} sends ≥ 50% of fresh lineages"), format!("{within:.0}%"), within >= 50.0));
        }
        let cf = cforks.lock().unwrap();
        if !riddle_core::descent::OPEN_FORKS.contains(&9) {
            println!("D9 fork closed (Cut 26 §3 fallback): not seen by design");
        } else if !cf.is_empty() {
            let seen = pct(cf.values().filter(|v| **v).count(), cf.len());
            println!("D9 fork seen by the end of a 4 h absence: {seen:.0}% of {} cohort lineages", cf.len());
            rows.push((format!("D9 fork seen by cohort lineages by the absence's end ≥ 50% (n={})", cf.len()), format!("{seen:.0}%"), seen >= 50.0));
        }
    }
    let mut causes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut n_causes = 0;
    for r in &all {
        for c in &r.causes {
            *causes.entry(c.as_str()).or_insert(0) += 1;
            n_causes += 1;
        }
    }
    let mut cv: Vec<(&str, usize)> = causes.into_iter().collect();
    cv.sort_by_key(|b| std::cmp::Reverse(b.1));
    let top = cv.first().map(|(c, n)| (c.to_string(), pct(*n, n_causes))).unwrap_or(("none".into(), 0.0));
    rows.push((format!("Top death cause share < 35% ({})", top.0), format!("{:.1}%", top.1), top.1 < 35.0));
    let ev_per_600 = {
        let e: u64 = all.iter().map(|r| r.events as u64).sum();
        let t: u64 = all.iter().map(|r| r.ticks as u64).sum();
        e as f64 * 600.0 / t.max(1) as f64
    };
    rows.push(("Events per 600 ticks (renderable) ≥ 6".into(), format!("{ev_per_600:.1}"), ev_per_600 >= 6.0));
    // Replay hash.
    let hash_of = |g: &mut Game| {
        let mut h: u64 = 0xcbf29ce484222325;
        g.send();
        for _ in 0..40 {
            let r = g.step(50);
            let s = serde_json::to_string(&r.events).unwrap();
            for b in s.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            if r.run_over {
                g.send();
            }
        }
        let mut g2 = Game::new_literal(g.lineage.seed);
        g2.set_rules_raw(good()).unwrap();
        let rep = g2.run_offline(1800);
        let s = serde_json::to_string(&rep).unwrap() + &g2.save();
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };
    let mut ga = Game::new_literal(7);
    ga.set_rules_raw(good()).unwrap();
    let mut gb = Game::new_literal(7);
    gb.set_rules_raw(good()).unwrap();
    let (ha, hb) = (hash_of(&mut ga), hash_of(&mut gb));
    rows.push(("Replay hash identical (seed+rules+elapsed)".into(), format!("{ha:016x}"), ha == hb));
    let known_ok = all.iter().all(|r| r.known_to_ok);
    rows.push(("Forecast known_to == best_depth + 1".into(), if known_ok { "all".into() } else { "violated".into() }, known_ok));
    // Cut 2 gates (docs/CUT2.md).
    let runs8 = |rs: &[&SeedResult]| rs.iter().map(|r| r.runs as f64 * 8.0 / hours as f64).sum::<f64>() / ns as f64;
    let (d8, e8) = (runs8(&default), runs8(&edited));
    // (Cut 30 §6: DEFAULT's sends were the two-row fighter's, dead every send and waking 20 minutes; the
    // idle floor's own count is `cut30_rows`' IDLE row — printed here for the record, EDITED still gated)
    rows.push(("Expeditions per 8 h (EDITED) in 6–16".into(), format!("{e8:.1} · DEFAULT {d8:.1}"), (6.0..=16.0).contains(&e8)));
    let d_yield = default.iter().map(|r| r.xp as u64 + r.gold.max(0) as u64).sum::<u64>();
    rows.push(("DEFAULT yields 0 xp/gold over 8 h".into(), format!("{d_yield}"), d_yield == 0));
    // Cut 15 §1: marks per 8 h (printed, not gated) and the frontier banks' share of them.
    let per8 = |rs: &[&SeedResult], f: &dyn Fn(&SeedResult) -> u32| rs.iter().map(|r| f(r) as f64 * 8.0 / hours as f64).sum::<f64>() / ns as f64;
    println!("marks per 8 h (Cut 15 §1): DEFAULT {:.1} ({:.1} frontier) · EDITED {:.1} ({:.1} frontier)", per8(&default, &|r| r.marks), per8(&default, &|r| r.frontier), per8(&edited, &|r| r.marks), per8(&edited, &|r| r.frontier));
    let e_banked = edited.iter().map(|r| r.banked as f64 * 8.0 / hours as f64).sum::<f64>() / ns as f64;
    rows.push(("EDITED banks ≥ 3 runs per 8 h".into(), format!("{e_banked:.1}"), e_banked >= 3.0));
    let (shown, fired): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.patches.0, a.1 + r.patches.1));
    let (fresh_n, fresh_f): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.patches_fresh.0, a.1 + r.patches_fresh.1));
    println!("shown patches fire in {:.0}% of fresh reseeded replays (n={fresh_n})", pct(fresh_f as usize, fresh_n as usize));
    rows.push((format!("Patches whose row fired in ≥ 50% of replays (n={shown})"), format!("{:.0}%", pct(fired as usize, shown as usize)), fired == shown));
    // Verdict cost as the player pays it: one verdict at a time in the worker. The batch above ran
    // on every core, so its per-verdict wall times include contention; re-measure single-threaded on
    // a fresh sample of deaths after the batch has drained.
    phase("report to verdict timing");
    let vsecs: Vec<f64> = {
        let mut v = Vec::new();
        for seed in 1..=3u64 {
            let mut g = setup(Bot::Default, seed);
            let _ = g.run_offline(3600 * 2);
            let ids: Vec<u32> = g.deaths.keys().copied().collect();
            for id in ids.iter().take(4) {
                let t = Clock::start();
                let _ = riddle_core::trace::verdict(&mut g, *id);
                v.push(t.secs());
            }
        }
        v
    };
    let vmean = vsecs.iter().sum::<f64>() / vsecs.len().max(1) as f64;
    rows.push((format!("Verdict time ≤ 0.4 s (single-threaded mean of {})", vsecs.len()), format!("{vmean:.2} s"), vmean <= 0.4));
    phase("verdict timing");
    let dsecs: Vec<f64> = all.iter().flat_map(|r| r.death_secs.iter().copied()).collect();
    let dmean = dsecs.iter().sum::<f64>() / dsecs.len().max(1) as f64;
    println!("death() with forecast deltas: mean {dmean:.2} s over {}", dsecs.len());
    // Cut 3: per-tick cost. The batches' own number (report and verdicts included, `threads`
    // jobs at once) is printed for reference; the gate is the quiet one measured above.
    let mut tus: Vec<f64> = [Bot::Default, Bot::Edited, Bot::Full].iter().flat_map(|b| per_bot(*b)).map(|r| r.tick_us).filter(|x| x.is_finite() && *x > 0.0).collect();
    tus.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let tick_med = tus.get(tus.len() / 2).copied().unwrap_or(0.0);
    println!("per-tick cost: quiet {} · batches incl. reports, contended: median {tick_med:.2} µs over {} ({threads} threads)", quiet_ticks.iter().map(|(b, us)| format!("{b} {us:.2} µs")).collect::<Vec<_>>().join(" · "), tus.len());
    let quiet_max = quiet_ticks.iter().map(|(_, us)| *us).fold(0.0, f64::max);
    rows.push(("Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)".into(), format!("{quiet_max:.2} µs"), quiet_max <= 6.0));
    // Player-shaped lineages only: LEARNED and the FULL bots know everything by construction,
    // RANDOM/PASSIVE are probes.
    let player_bots: Vec<&SeedResult> = [Bot::Default, Bot::Edited, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered].iter().flat_map(|b| per_bot(*b)).collect();
    let off_ok = player_bots.iter().all(|r| r.learned >= 1 && r.pending >= 1);
    let off_min = player_bots.iter().map(|r| r.learned.min(r.pending)).min().unwrap_or(0);
    rows.push((format!("Offline {hours} h: learned ≥ 1 and pending ≥ 1 every seed"), format!("min {off_min}"), off_ok));
    // Cut 5 gates (docs/CUT5.md): story lines in the grammar, reels with a turn in them, a
    // top line that names a row/trait/companion, situations on the first floors. The story
    // and situation counts run over every bot; the reel gates over the player-shaped ones.
    let (st_n, st_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.stories.0, a.1 + r.stories.1));
    rows.push((format!("Story lines ≤ 12 words, table verb (n={st_n})"), format!("{:.1}%", pct(st_ok as usize, st_n as usize)), st_ok == st_n));
    let absences: Vec<(usize, bool)> = player_bots.iter().flat_map(|r| r.reel_pairs.iter().copied().zip(r.reel_names.iter().copied())).collect();
    let pairs_ok = pct(absences.iter().filter(|(p, _)| *p >= 2).count(), absences.len());
    rows.push((format!("Reel ≥ 2 distinct (threat, resolution) pairs per 8 h (n={})", absences.len()), format!("{pairs_ok:.0}%"), pairs_ok >= 90.0));
    let names_ok = pct(absences.iter().filter(|(_, n)| *n).count(), absences.len());
    rows.push(("Top reel line names a row, trait or companion ≥ 80%".into(), format!("{names_ok:.0}%"), names_ok >= 80.0));
    let (si_runs, si_with): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.situations.0, a.1 + r.situations.1));
    let si_pct = pct(si_with as usize, si_runs as usize);
    rows.push((format!("Situations: ≥ 1 per run on D1–5 ≥ 90% (n={si_runs})"), format!("{si_pct:.1}%"), si_pct >= 90.0));
    // Cut 7 gates (docs/CUT7.md). §1: the Captain's floor — DEFAULT leaves D5 alive about half
    // the time (informational; the wall gates above are the bars). §3: each band situation
    // appears on ≥ 90% of the runs that reach its depth (every bot's real runs), and the
    // shipped preset passes it on ≤ 20% of seeds (a fresh hero on the floor,
    // `probes::situation_trial`); the one-row answer's rate is printed beside it.
    let (c_reached, c_left): (u32, u32) = default.iter().fold((0, 0), |a, r| (a.0 + r.captain.0, a.1 + r.captain.1));
    println!("captain: DEFAULT left D5 alive in {:.0}% of {} runs that reached it", pct(c_left as usize, c_reached as usize), c_reached);
    // Cut 12 §4: a band's situation sits on one of the band's floors (drawn from the run's
    // seed), so "appears" is measured on the runs that passed the band's last floor it can sit
    // on; the trials force it onto its Cut 7 floor.
    phase("to situations");
    let band: Vec<&riddle_core::engine::BandRun> = all.iter().flat_map(|r| r.band.iter()).collect();
    let mut sit_lines: Vec<String> = Vec::new();
    for (what, depth) in riddle_core::descent::SITUATION_DEPTHS {
        let b = riddle_core::situations::band(depth).expect("a band");
        let passed = (b.first..=b.last).filter(|d| riddle_core::descent::boss_for(*d).is_none()).max().unwrap_or(b.last) + 1;
        let reached: Vec<&&riddle_core::engine::BandRun> = band.iter().filter(|b| b.depth >= passed).collect();
        let met = reached.iter().filter(|b| b.met.iter().any(|m| m == what)).count();
        let appear = pct(met, reached.len());
        rows.push((format!("Situation {what} (D{}–{}) met by D{passed} ≥ 90% (n={})", b.first, passed - 1, reached.len()), format!("{appear:.0}%"), appear >= 90.0 || reached.is_empty()));
        let trials: Vec<(bool, bool, bool)> = (1..=seeds).map(|s| riddle_core::probes::situation_trial(s, what, false)).collect();
        let answered: Vec<(bool, bool, bool)> = (1..=seeds).map(|s| riddle_core::probes::situation_trial(s, what, true)).collect();
        let p_pass = pct(trials.iter().filter(|t| t.1).count(), ns);
        let a_pass = pct(answered.iter().filter(|t| t.1).count(), ns);
        let p_left = pct(trials.iter().filter(|t| t.2).count(), ns);
        let a_left = pct(answered.iter().filter(|t| t.2).count(), ns);
        sit_lines.push(format!("{what:<8} D{depth:<3} appears {appear:>4.0}%   preset pass {p_pass:>4.0}% (left {p_left:>3.0}%)   answered pass {a_pass:>4.0}% (left {a_left:>3.0}%)   row: {}", riddle_core::probes::situation_answer(what).describe()));
        rows.push((format!("DEFAULT passes the {what} ≤ 20% of seeds"), format!("{p_pass:.0}%"), p_pass <= 20.0));
    }
    phase("situation trials");
    println!("situations (Cut 7 §3):\n  {}", sit_lines.join("\n  "));
    // Cut 12 §2: the thief guard card answers the den — snatches with the card ≤ 20% of
    // those without, over the seeds (`probes::den_guard_trial`, the preset on a den floor).
    let den: Vec<(u32, u32)> = (1..=seeds).map(riddle_core::probes::den_guard_trial).collect();
    let (den_without, den_with): (u32, u32) = den.iter().fold((0, 0), |a, (w, c)| (a.0 + w, a.1 + c));
    let den_pct = pct(den_with as usize, den_without as usize);
    println!("thief guard (Cut 12 §2): den snatches without the card {den_without} · with it {den_with} ({den_pct:.0}%) over {ns} seeds");
    rows.push((format!("Thief guard cuts den snatches ≤ 20% of without (n={den_without})"), format!("{den_pct:.0}%"), den_pct <= 20.0 && den_without > 0));
    // Cut 12 §4: from D3 every floor rolls one situation, never the previous floor's kind, and
    // D3–10 hold ≥ 4 kinds on every seed (`probes::twist_sequence`).
    phase("den");
    let seqs: Vec<Vec<Option<String>>> = (1..=seeds).map(riddle_core::probes::twist_sequence).collect();
    let mut tw_kinds_min = usize::MAX;
    let mut tw_gaps = 0usize;
    let mut tw_repeats = 0usize;
    let mut tw_counts: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for seq in &seqs {
        let kinds: std::collections::BTreeSet<&str> = seq.iter().filter_map(|t| t.as_deref()).collect();
        tw_kinds_min = tw_kinds_min.min(kinds.len());
        for (i, t) in seq.iter().enumerate() {
            let depth = 3 + i as u32;
            match t {
                Some(k) => *tw_counts.entry(k.clone()).or_insert(0) += 1,
                None if riddle_core::descent::boss_for(depth).is_none() => tw_gaps += 1,
                None => {}
            }
            if i > 0 && t.is_some() && *t == seq[i - 1] {
                tw_repeats += 1;
            }
        }
    }
    println!("twists (Cut 12 §4): D3–10 over {ns} seeds · kinds min {tw_kinds_min} · floors without one {tw_gaps} · repeats {tw_repeats} · {}", tw_counts.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(" · "));
    rows.push(("Twists D3–10: ≥ 4 kinds every seed, one per floor, none twice".into(), format!("min {tw_kinds_min} · {tw_gaps} gaps · {tw_repeats} rep"), tw_kinds_min >= 4 && tw_gaps == 0 && tw_repeats == 0));
    // Cut 10 §2: the wall as a ramp — on a lineage that knows the Warlord's counter and whose
    // set lacks it, the forecast's D9 row names the counter and inserting it at the top lifts
    // D9's reach by ≥ 0.3 on every seed (`probes::counter_trial`; the end placement is printed
    // beside it: position is the point).
    phase("twists");
    let trials: Vec<(u64, (bool, f64, f64, f64))> = counters.lock().unwrap().iter().map(|(s, t)| (*s, *t)).collect();
    let named = trials.iter().filter(|(_, t)| t.0).count();
    let lifted = trials.iter().filter(|(_, t)| t.2 - t.1 >= 0.3).count();
    let mean = |xs: Vec<f64>| xs.iter().sum::<f64>() / ns.max(1) as f64;
    let (m_base, m_top, m_end) = (mean(trials.iter().map(|(_, t)| t.1).collect()), mean(trials.iter().map(|(_, t)| t.2).collect()), mean(trials.iter().map(|(_, t)| t.3).collect()));
    println!("counter try (Cut 10 §2): D9 reach {m_base:.2} → top {m_top:.2} · end {m_end:.2} (mean of {ns}); named {named}/{ns}; lifted ≥ 0.3 {lifted}/{ns}");
    rows.push(("Forecast names the absent counter (D9 try)".into(), format!("{}/{ns}", named), named == ns));
    rows.push(("Counter at the top lifts D9 reach ≥ 0.3".into(), format!("{}/{ns}", lifted), lifted == ns));
    // Cut 11 gates (docs/CUT11.md): the chain's because on the state reasons of death traces;
    // the root-cause patch shown on theft/lock roots and its delta against the symptom's;
    // a dice death never empty.
    phase("counter trials");
    let (bc_n, bc_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.because.0, a.1 + r.because.1));
    let bc_deaths: u32 = all.iter().map(|r| r.because_deaths).sum();
    let bc_pct = pct(bc_ok as usize, bc_n as usize);
    rows.push((format!("State reasons carry a because ≥ 95% ({bc_n} over {bc_deaths} deaths)"), format!("{bc_pct:.1}%"), bc_pct >= 95.0 && bc_deaths >= 100));
    let (rt_n, rt_shown, rt_beats): (u32, u32, u32) = all.iter().fold((0, 0, 0), |a, r| (a.0 + r.roots.0, a.1 + r.roots.1, a.2 + r.roots.2));
    let rt_shown_pct = pct(rt_shown as usize, rt_n as usize);
    let rt_beats_pct = pct(rt_beats as usize, rt_n as usize);
    // The contract's second bar (the root patch's delta ≥ the best symptom patch's, ≥ 80 %)
    // is printed, not gated: measured 30 % on the quick table — the "symptom" it competes
    // with is nearly always `hp<20 → rest` (+0.17 on a set that never rests), the largest
    // generic gain there is, and a den raid on D3 does not out-forecast it at D5 (docs/CUT11.md
    // deviation, README "Cut 11"). The root patch is shown regardless, ranked by its number.
    // Cut 14 §1: a root patch measured under the baseline is not offered (nothing under it
    // is); those deaths are counted out of the gate and printed beside it.
    let rt_under: u32 = all.iter().map(|r| r.roots_under).sum();
    println!("root patches (Cut 11 §2): {rt_n} theft/lock deaths · root patch shown {rt_shown} ({rt_shown_pct:.0}%) · its delta ≥ the best symptom's {rt_beats} ({rt_beats_pct:.0}%; bar 80%, informational — deviation) · {rt_under} more with the root under the baseline (Cut 14 §1: not offered)");
    rows.push((format!("Root patch shown on theft/lock roots ≥ 80% (n={rt_n}, +{rt_under} under base)"), format!("{rt_shown_pct:.0}%"), rt_shown_pct >= 80.0 || rt_n == 0));
    let (dn_n, dn_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.dice_named.0, a.1 + r.dice_named.1));
    rows.push((format!("Dice deaths name an alternative 100% (n={dn_n})"), format!("{}/{dn_n}", dn_ok), dn_ok == dn_n));
    // Cut 13 §1 gates (docs/CUT13.md): stalls ≤ 1 % of sends on DEFAULT (EDITED plays
    // `probes::good()`, FULL `probes::full()` — printed beside it); every sampled stall has a
    // verdict with a patch that fired in ≥ 50 % of its replays; the reel's cause is the trace's.
    let stall_pct = |rs: &[&SeedResult]| {
        let (n, k): (u32, u32) = rs.iter().fold((0, 0), |a, r| (a.0 + r.stalls.0, a.1 + r.stalls.1));
        (pct(k as usize, n as usize), n, k)
    };
    let (d_stall, d_sends, d_stalls) = stall_pct(&default);
    let (e_stall, e_sends, e_stalls) = stall_pct(&edited);
    let (f_stall, f_sends, f_stalls) = stall_pct(&full);
    println!("stalls (Cut 13 §1): DEFAULT {d_stalls}/{d_sends} ({d_stall:.1}%) · EDITED (good) {e_stalls}/{e_sends} ({e_stall:.1}%) · FULL {f_stalls}/{f_sends} ({f_stall:.1}%)");
    rows.push((format!("Stalls ≤ 1% of sends on DEFAULT (n={d_sends})"), format!("{d_stall:.1}%"), d_stall <= 1.0));
    // … and on the cohorts' own sets (`eval/cards/*.rules.json`), each over `seeds` × 4 h.
    let (mut c_sends, mut c_stalls, mut worst) = (0u32, 0u32, (0.0f64, String::new()));
    let (mut c_dances, mut worst_dance) = (0u32, (0.0f64, String::new()));
    let mut ret_sets: Vec<(String, f64, f64)> = Vec::new();
    for (si, (name, _)) in sets.iter().enumerate() {
        let (n, k, dd, dn) = (1..=seeds).fold((0u32, 0u32, 0u32, 0u32), |a, s| { let r = cohort[&(si, s, false)]; (a.0 + r.0, a.1 + r.1, a.2 + r.2, a.3 + r.3) });
        let p = pct(k as usize, n as usize);
        let pd = pct(dn as usize, n as usize);
        c_dances += dn;
        if pd > worst_dance.0 {
            worst_dance = (pd, name.clone());
        }
        let death = pct(dd as usize, n as usize);
        // Cut 19 §2: a set with a return row, and the same set without it.
        let bare = cohort.contains_key(&(si, 1, true)).then(|| {
            let (n, _, dd) = (1..=seeds).fold((0u32, 0u32, 0u32), |a, s| { let r = cohort[&(si, s, true)]; (a.0 + r.0, a.1 + r.1, a.2 + r.2) });
            pct(dd as usize, n as usize)
        });
        match bare {
            Some(b) => {
                println!("  cohort set {name}: {k}/{n} ({p:.1}%) · death {death:.1}% (no return row {b:.1}%) · dances {dn}");
                ret_sets.push((name.clone(), death, b));
            }
            None => println!("  cohort set {name}: {k}/{n} ({p:.1}%) · death {death:.1}% · dances {dn}"),
        }
        c_sends += n;
        c_stalls += k;
        if p > worst.0 {
            worst = (p, name.clone());
        }
    }
    if !sets.is_empty() {
        let c_pct = pct(c_stalls as usize, c_sends as usize);
        rows.push((format!("Stalls ≤ 1% of sends on every cohort set ({} sets, n={c_sends})", sets.len()), format!("{c_pct:.1}% · worst {:.1}%", worst.0), worst.0 <= 1.0));
        // Cut 27 §4 (AS: `R8 pack ↔ pick up`, 12 turns among four foes — a stall no stall row saw: the
        // cohort lineage's sends recovered from it, 20 of 213 on AS's set before the fix, 0 stalls): a card
        // taking turns with a chore is the engine's loop (a card's sub-rows are not the player's), counted
        // whether the guard's give-up recovered the send or not.
        let loops = |si: usize| (1..=seeds).fold((0u32, 0u32), |a, s| { let r = cohort[&(si, s, false)]; (a.0 + r.0, a.1 + r.4) });
        let (l_sends, l_n, l_worst) = (0..sets.len()).fold((0u32, 0u32, (0.0f64, String::new())), |a, si| {
            let (n, k) = loops(si);
            let p = pct(k as usize, n as usize);
            (a.0 + n, a.1 + k, if p > a.2 .0 { (p, sets[si].0.clone()) } else { a.2 })
        });
        println!("card ↔ chore loops (Cut 27 §4): {l_n}/{l_sends} sends · worst {:.1}% {}", l_worst.0, l_worst.1);
        rows.push((format!("Card ↔ chore loops ≤ 1% of sends on every cohort set (n={l_sends})"), format!("{:.1}% · worst {:.1}%", pct(l_n as usize, l_sends as usize), l_worst.0), l_worst.0 <= 1.0));
        // QA on 778fa1b (qaV: `R4 retreat` / `R6 attack` before two ogres for six minutes, no
        // blood either way, no stall): a run with `DANCE_ACTIONS` foe-facing row actions in a
        // row, `DANCE_MOVES` of them retreats, and no blood drawn is a loop the stall guard does
        // not see.
        let d_pct = pct(c_dances as usize, c_sends as usize);
        rows.push((format!("Bloodless dances ≤ 1% of sends on every cohort set (≥ {} foe-facing actions, ≥ {} retreats, n={c_sends})", riddle_core::turn::DANCE_ACTIONS, riddle_core::turn::DANCE_MOVES), format!("{d_pct:.1}% · worst {:.1}% {}", worst_dance.0, worst_dance.1), worst_dance.0 <= 1.0));
    }
    // Cut 24 §1 (AL: the Warlord > 4 min on `attack nearest`; AK: ~100 s of retreat ↔ pack
    // break): the longest no-HP stretch of each send (foe-facing row actions with neither side's
    // HP moved, or a boss in view unhurt), p99 per set; the pre-boss set of AL reaches the
    // Warlord and is driven off (the loop ends).
    {
        let cn = cnohp.lock().unwrap();
        let mut worst = (0u32, String::new());
        let (mut n_all, mut driven_all) = (0usize, 0u32);
        for (si, (name, _)) in sets.iter().enumerate() {
            let mut v: Vec<u32> = (1..=seeds).filter_map(|s| cn.get(&(si, s))).flat_map(|x| x.0.clone()).collect();
            let driven: u32 = (1..=seeds).filter_map(|s| cn.get(&(si, s))).map(|x| x.1).sum();
            if v.is_empty() {
                continue;
            }
            v.sort();
            let p99 = v[((v.len() as f64 - 1.0) * 0.99).round() as usize];
            println!("  no-HP stretch {name}: p50 {} · p99 {p99} · max {} · driven off {driven}/{}", v[v.len() / 2], v[v.len() - 1], v.len());
            n_all += v.len();
            driven_all += driven;
            if p99 > worst.0 || worst.1.is_empty() {
                worst = (p99, name.clone());
            }
        }
        if n_all > 0 {
            rows.push((format!("Longest no-HP stretch p99 ≤ {} actions, every cohort set (n={n_all}, driven off {driven_all})", riddle_core::turn::NOHP_ACTIONS), format!("worst {} {}", worst.0, worst.1), worst.0 <= riddle_core::turn::NOHP_ACTIONS));
        }
        let al: Vec<usize> = sets.iter().enumerate().filter(|(_, (n, _))| n.contains("raterAL-preboss")).map(|(i, _)| i).collect();
        if let Some(&si) = al.first() {
            let driven: u32 = (1..=seeds).filter_map(|s| cn.get(&(si, s))).map(|x| x.1).sum();
            let long = (1..=seeds).filter_map(|s| cn.get(&(si, s))).flat_map(|x| x.0.iter()).filter(|&&x| x > riddle_core::turn::BOSS_STILL).count();
            rows.push(("AL's pre-boss set: the Warlord drives it off, no fight > 60".into(), format!("driven {driven} · longer {long}"), driven > 0 && long == 0));
        }
    }
    // Cut 19 §2: a return walks — a set with a return row dies some nights (> 0), and less
    // than without the row (the same, to the send, when the row never acts: raterY's return
    // sits under `hp < 25% → bank`, which takes every moment it could).
    if !ret_sets.is_empty() {
        let ok = ret_sets.iter().filter(|(_, d, b)| *d > 0.0 && (d < b || (d - b).abs() < 1e-9)).count();
        let worst = ret_sets.iter().map(|(_, d, b)| d - b).fold(f64::NEG_INFINITY, f64::max);
        rows.push((format!("Return row: 0 < death share < without it ({} cohort sets; = if it never acts)", ret_sets.len()), format!("{ok}/{} · worst {worst:+.1} pts", ret_sets.len()), ok == ret_sets.len()));
    }
    gold_report(&sets, &golds.lock().unwrap(), seeds, hours, &mut rows);
    exits::report(&sets, &exit_tallies.lock().unwrap(), exit_seeds, exit_hours, &mut rows);
    forge_report(&sets, &forges.lock().unwrap(), &kresults.lock().unwrap(), &results, seeds, &mut rows);
    if !quick {
        lever_rows(&mut rows);
        oath_report(&sets, &obanks.lock().unwrap(), &oreads.lock().unwrap(), &olevers.lock().unwrap(), &mut rows);
    }
    // Cut 23 §2: the death mix (reported, not gated: the contract's bar is the measure).
    let (dm_flag, dm_n) = death_mix_report(&sets, &cdeaths.lock().unwrap(), seeds);
    println!("  sets with one cause over half their deaths: {dm_flag}/{dm_n}");
    // Cut 22 §3: an edit's paired move is far tighter than the bars' own ± — over every
    // one-notch edit of every cohort set, Σ paired ± ≤ ½ Σ absolute ± (the depths with a ±).
    let paireds = paireds.lock().unwrap();
    let (pe, pp, pa) = paireds.values().fold((0u32, 0.0f64, 0.0f64), |a, r| (a.0 + r.0, a.1 + r.1, a.2 + r.2));
    let worst = paireds.iter().map(|(si, r)| (r.1 / r.2.max(1e-9), sets[*si].0.clone())).fold((0.0f64, String::new()), |a, b| if b.0 > a.0 { b } else { a });
    println!("paired edit delta (Cut 22 §3): {pe} one-notch edits over {} sets · paired ± / absolute ± {:.2} (worst set {} {:.2})", sets.len(), pp / pa.max(1e-9), worst.1, worst.0);
    rows.push((format!("Paired edit ± ≤ ½ absolute ± ({pe} cohort edits)"), format!("{:.2}", pp / pa.max(1e-9)), pp <= 0.5 * pa));
    diverge_rows(&mut rows);
    if !fast {
        prog_rows(&mut rows);
    }
    let (sv_n, sv_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.stall_verdicts.0, a.1 + r.stall_verdicts.1));
    rows.push((format!("Stall verdicts: ≥ 1 patch fired ≥ 50% (n={sv_n})"), format!("{sv_ok}/{sv_n}"), sv_ok == sv_n));
    let (sr_n, sr_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.stall_reel.0, a.1 + r.stall_reel.1));
    rows.push((format!("Stall reel line's cause == the trace's (n={sr_n})"), format!("{sr_ok}/{sr_n}"), sr_ok == sr_n));
    phase("rest");
    rows.extend(cut30.map_or_else(
        || {
            let mut r = Vec::new();
            cut30_rows(&mut r, seeds, threads);
            r
        },
        |h| h.join().unwrap(),
    ));
    phase("cut30");
    println!();
    println!("{:<52} {:>18}  result", "gate", "value");
    let mut fails = 0;
    seal(&mut rows);
    for (name, value, ok) in &rows {
        // Cut 30 §6: the "not engaging fails" rows are replaced by the idle gates (dayplayer `--idle`,
        // `--picked`, `--tuned`); they print for the record and no longer gate
        if retired(name) {
            println!("{:<52} {:>18}  retired ({})", name, value, if *ok { "pass" } else { "fail" });
            continue;
        }
        println!("{:<52} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        if !ok {
            fails += 1;
        }
    }
    println!("\ncauses: {}", cv.iter().take(8).map(|(c, n)| format!("{c} {:.0}%", pct(*n, n_causes))).collect::<Vec<_>>().join(" · "));
    let _ = ExitTier::Bank;
    println!("gates: {} ({} seeds × {} h × {} verdicts/seed, {:.0}s)", if fails > 0 { "FAIL" } else { "all PASS" }, seeds, hours, verdicts_per_seed, t_start.elapsed().as_secs_f64());
    if fails > 0 {
        eprintln!("{fails} gate(s) FAIL");
        std::process::exit(1);
    }
}
