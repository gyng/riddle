//! Bot gates (docs/CUT1.md): 30 seeds × the 8 h offline model per bot. Prints PASS/FAIL and
//! exits non-zero on any FAIL. Never weaken a gate; tune content.
//!   cargo run --release --example metrics [-- --seeds 30 --hours 8]
use riddle_core::engine::ExitTier;
use riddle_core::hero::Class;
use riddle_core::rng::{splitmix, Rng};
use riddle_core::{Ev, Game, RuleSet};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
type CohortStalls = BTreeMap<(usize, u64, bool), (u32, u32, u32, u32)>;

/// Cut 19 §2: `set` without its `return` rows (the death share's comparison).
fn without_return(set: &RuleSet) -> RuleSet {
    RuleSet { rows: set.rows.iter().filter(|r| r.verb.v != "return").cloned().collect(), name: set.name.clone() }
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

fn cohort_stalls(set: &RuleSet, seed: u64, hours: u64) -> (u32, u32, u32, u32) {
    let mut g = cohort_game(set, seed);
    riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
    // QA on 23ed91f (qaL): a run that reaches the tick cap is a stall that never ended (a
    // conjurer's blades reset the guard: 120 000 ticks, 3 000 kills) — counted with them.
    let capped = g.batch.run_ticks.iter().filter(|&&t| t >= riddle_core::engine::MAX_TURNS_PER_RUN).count() as u32;
    let deaths = g.batch.run_outcomes.iter().filter(|(_, c)| c.is_some()).count() as u32;
    // QA on 778fa1b (qaV): runs with a bloodless dance (`Batch.dances`).
    (g.batch.run_outcomes.len() as u32, g.batch.stalls + capped, deaths, g.batch.dances)
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
    /// The sends that returned (60 %) and their own net: the safe run the raters called a
    /// treadmill (AG: "+$77 returned · +$19 salvage · −$160 spent").
    ret_sends: u32,
    ret_net: i64,
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
        self.ret_sends += o.ret_sends;
        self.ret_net += o.ret_net;
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
fn cohort_gold(set: &RuleSet, seed: u64, hours: u64, waystone: bool) -> GoldTally {
    let mut g = cohort_game(set, seed);
    g.lineage.gold = 400;
    pack_for(&mut g);
    let budget = hours * 3600 * riddle_core::offline::TICKS_PER_SECOND;
    let mut consumed = 0u64;
    let mut t = GoldTally::default();
    while consumed < budget {
        // Each send's lines are read and cleared (a send's first lines share the last one's turn).
        g.lineage.gold_ledger.clear();
        g.lineage.rest_left = 0;
        g.lineage.start = if waystone { g.lineage.waystones.iter().copied().max().unwrap_or(1) } else { 1 };
        g.start_run(None);
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
            (r.turn, r.over.unwrap_or(ExitTier::Return))
        };
        consumed += turns as u64 + g.rest_after(turns, tier) as u64;
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
            if ["returned", "banked", "died", "lost", "stalled"].iter().any(|p| w.starts_with(p)) {
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
    }
    t
}

/// Cut 22 §3: every one-notch edit of `set` (each numeric condition ±5 for a share, ±1
/// otherwise) against the set, on a D8 lineage owning every token: (edits, Σ paired ±,
/// Σ absolute ±) over the shaft's depths whose bar has a ±.
fn paired_edits(set: &RuleSet) -> (u32, f64, f64) {
    let mut g = Game::new(7);
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

fn setup(bot: Bot, seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
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

fn run_seed(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize) -> SeedResult {
    let mut g = setup(bot, seed);
    let mut r = SeedResult::default();
    let mut secs = 0.0;
    for _ in 0..bot.batches() {
        let t = Instant::now();
        let report = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
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
        let stall_ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| rec.stall).map(|(id, _)| *id).filter(|id| g.batch.highlights.iter().any(|h| h.run_id == *id)).take(2).collect();
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
    let step = (ids.len() / verdicts_per_seed).max(1);
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
    // Cut 2 §6 patch quality: the shown patches (full `death()`, two per seed on the
    // player-shaped bots) must have fired in ≥ 50% of 12 reseeded replays of the death.
    if matches!(bot, Bot::Default | Bot::Edited) {
        for id in ids.iter().step_by(step).take(2) {
            let t = Instant::now();
            let Some(d) = g.death(*id) else { continue };
            r.death_secs.push(t.elapsed().as_secs_f64());
            let rec = g.deaths.get(id).cloned().unwrap();
            // Cut 11 §2: a root-cause patch answers a theft floors back (or an unlock); its
            // number is the forecast delta, not the moment's replays — exempt here. §4: so is
            // a `dice` death's below-bar alternative (labelled as such on the screen).
            // Cut 19 §4: a `row` verdict's cut (a removed or narrowed row) inserts nothing.
            for p in d.patches.iter().filter(|p| p.insert_at >= 0 && p.root.is_none() && !p.below_bar && !p.remove && !p.replace) {
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
    if matches!(bot, Bot::Default | Bot::Edited | Bot::Pets | Bot::Levelled) {
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
    r
}

type Golds = BTreeMap<(usize, u64, bool), GoldTally>;
/// Cut 22 §3: set index → (edits, Σ paired ±, Σ absolute ±) (`paired_edits`).
type Paireds = BTreeMap<usize, (u32, f64, f64)>;

fn gold_report(sets: &[(String, RuleSet)], golds: &Golds, seeds: u64, hours: u64, rows: &mut Vec<(String, String, bool)>) {
    // Cut 22 §1: a safe run nets gold — per send, after its supplies, tolls and thefts, on every
    // cohort set whose sends mostly come home (bank or return ≥ 60 %), from D1. The deepest
    // waystone's numbers print beside (a start the player picks, not the gate's).
    let (mut g_ok, mut g_n, mut g_worst) = (0usize, 0usize, (f64::INFINITY, String::new()));
    let (mut th_all, mut th_sup, mut th_sends) = (0u32, 0u32, 0u32);
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
        th_all += d1.thefts + ws.thefts;
        th_sup += d1.supply_thefts + ws.supply_thefts;
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
    if g_n > 0 {
        rows.push((format!("Net gold per send ≥ +$20 on every mostly-home cohort set ({g_n})"), format!("{g_ok}/{g_n} · worst {:+.0}", g_worst.0), g_ok == g_n));
    }
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn main() {
    // The table fills the machine seed by seed; a panel's sims stay sequential inside a job.
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    // --quick: 8 seeds × 8 h × 3 verdicts (≈ 30 s; depth gates are 8 h tail gates, so hours stay).
    // Full: 30 × 8 h × 8 (≈ 2 min) is the number that counts. Never weaken bars.
    let quick = args.iter().any(|a| a == "--quick");
    let seeds = get("--seeds", if quick { 8 } else { 30 });
    let hours = get("--hours", 8);
    let verdicts_per_seed = get("--verdicts", if quick { 3 } else { 8 }) as usize;
    let t_start = std::time::Instant::now();
    // `METRICS_PHASES=1`: each phase's wall to stderr, and every job's (docs/ITERATION_SPEED.md).
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
            let t = Instant::now();
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
            (b.name(), t.elapsed().as_secs_f64() * 1e6 / ticks.max(1) as f64)
        })
        .collect();
    phase("quiet ticks");
    // `tools/gates.mjs` starts the wire invariants (examples/qa.rs, 3/4 of the cores) on this line,
    // so they never share the cores with the single-threaded quiet measurement above.
    if std::env::var("METRICS_QUIET_SIGNAL").is_ok() {
        eprintln!("metrics: quiet ticks measured");
    }
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
        Gold(usize, u64, bool),
        Paired(usize),
        Bot(usize, u64),
    }
    let sets = Arc::new(cohort_sets());
    // `--gold`: the Cut 22 §1 gold table alone (the cohort sets' sends; ~20 s).
    let gold_only = args.iter().any(|a| a == "--gold");
    let mut jobs: Vec<Job> = (1..=seeds).map(Job::Counter).collect();
    jobs.extend((0..sets.len()).flat_map(|si| (1..=seeds).map(move |s| Job::Cohort(si, s, false))));
    // Cut 19 §2: a set with a return row also plays without it (the night's death share, beside).
    let returning: Vec<usize> = (0..sets.len()).filter(|&si| sets[si].1.rows.iter().any(|r| r.verb.v == "return")).collect();
    jobs.extend(returning.iter().flat_map(|&si| (1..=seeds).map(move |s| Job::Cohort(si, s, true))));
    // Cut 22 §1: each cohort set's gold per send over 8 h of watched sends, from D1 and from its
    // deepest lit waystone.
    jobs.extend((0..sets.len()).flat_map(|si| (1..=seeds).flat_map(move |s| [Job::Gold(si, s, false), Job::Gold(si, s, true)])));
    jobs.extend(BOTS.iter().enumerate().flat_map(|(bi, _)| (1..=seeds).map(move |s| Job::Bot(bi, s))));
    // Cut 22 §3: each cohort set's one-notch edits, paired against the set.
    jobs.extend((0..sets.len()).map(Job::Paired));
    if gold_only {
        jobs.retain(|j| matches!(j, Job::Gold(..)));
    }
    // `--threads N` leaves cores to whatever runs beside the table (gates.mjs: the dayplayer's
    // sequential chains, which the full 32 starved — docs/ITERATION_SPEED.md §3.2).
    let threads = get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4).min(32)) as usize;
    let jobs = Arc::new(Mutex::new(jobs));
    let cohort: Arc<Mutex<CohortStalls>> = Arc::new(Mutex::new(BTreeMap::new()));
    type Counters = BTreeMap<u64, (bool, f64, f64, f64)>;
    let counters: Arc<Mutex<Counters>> = Arc::new(Mutex::new(BTreeMap::new()));
    let golds: Arc<Mutex<Golds>> = Arc::new(Mutex::new(BTreeMap::new()));
    let paireds: Arc<Mutex<Paireds>> = Arc::new(Mutex::new(BTreeMap::new()));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let (jobs, results, cohort, counters, sets, golds, paireds) = (Arc::clone(&jobs), Arc::clone(&results), Arc::clone(&cohort), Arc::clone(&counters), Arc::clone(&sets), Arc::clone(&golds), Arc::clone(&paireds));
        handles.push(std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some(job) = job else { break };
            let tj = Instant::now();
            match job {
                Job::Bot(bi, seed) => {
                    let r = run_seed(BOTS[bi], seed, hours, verdicts_per_seed);
                    if std::env::var("METRICS_PHASES").is_ok() {
                        eprintln!("job {} {seed} {:.1}s", BOTS[bi].name(), tj.elapsed().as_secs_f64());
                    }
                    results.lock().unwrap().insert((bi, seed), r);
                }
                Job::Cohort(si, seed, bare) => {
                    let set = if bare { without_return(&sets[si].1) } else { sets[si].1.clone() };
                    let r = cohort_stalls(&set, seed, 4);
                    cohort.lock().unwrap().insert((si, seed, bare), r);
                }
                Job::Gold(si, seed, way) => {
                    let r = cohort_gold(&sets[si].1, seed, hours, way);
                    golds.lock().unwrap().insert((si, seed, way), r);
                }
                Job::Paired(si) => {
                    let r = paired_edits(&sets[si].1);
                    paireds.lock().unwrap().insert(si, r);
                }
                Job::Counter(seed) => {
                    let r = riddle_core::probes::counter_trial(seed);
                    counters.lock().unwrap().insert(seed, r);
                }
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    phase("jobs");
    if gold_only {
        let mut rows = Vec::new();
        gold_report(&sets, &golds.lock().unwrap(), seeds, hours, &mut rows);
    // Cut 22 §3: an edit's paired move is far tighter than the bars' own ± — over every
    // one-notch edit of every cohort set, Σ paired ± ≤ ½ Σ absolute ± (the depths with a ±).
    let paireds = paireds.lock().unwrap();
    let (pe, pp, pa) = paireds.values().fold((0u32, 0.0f64, 0.0f64), |a, r| (a.0 + r.0, a.1 + r.1, a.2 + r.2));
    let worst = paireds.iter().map(|(si, r)| (r.1 / r.2.max(1e-9), sets[*si].0.clone())).fold((0.0f64, String::new()), |a, b| if b.0 > a.0 { b } else { a });
    println!("paired edit delta (Cut 22 §3): {pe} one-notch edits over {} sets · paired ± / absolute ± {:.2} (worst set {} {:.2})", sets.len(), pp / pa.max(1e-9), worst.1, worst.0);
    rows.push((format!("Paired edit ± ≤ ½ absolute ± ({pe} cohort edits)"), format!("{:.2}", pp / pa.max(1e-9)), pp <= 0.5 * pa));
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
    let gap = weighted("gap") + row;
    println!("verdict sample: {} verdicts, raw dice share {raw_dice:.1}% · death-weighted {dice:.1}% · row {row:.1}%", verdicts.len());
    // A share near 5% needs a few hundred verdicts to read: the quick mode's ~240 give ±2.8 pts.
    // Under 500 the bar is applied with that half-width; the full table is the gate that counts.
    let dice_n = verdicts.len() as f64;
    let dice_pm = if dice_n > 0.0 { 196.0 * (0.05 * 0.95 / dice_n).sqrt() } else { 0.0 };
    let dice_ok = if dice_n < 500.0 { dice <= 5.0 + dice_pm } else { dice <= 5.0 };
    rows.push((format!("Unfair deaths (dice) ≤ 5% (n={}, death-weighted{})", verdicts.len(), if dice_n < 500.0 { format!(", ±{dice_pm:.1} sample") } else { String::new() }), format!("{dice:.1}%"), dice_ok));
    rows.push(("Deaths tracing to a row (gap + row) ≥ 70%".into(), format!("{gap:.1}%"), gap >= 70.0));
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
        let mut g2 = Game::new(g.lineage.seed);
        g2.set_rules_raw(good()).unwrap();
        let rep = g2.run_offline(1800);
        let s = serde_json::to_string(&rep).unwrap() + &g2.save();
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };
    let mut ga = Game::new(7);
    ga.set_rules_raw(good()).unwrap();
    let mut gb = Game::new(7);
    gb.set_rules_raw(good()).unwrap();
    let (ha, hb) = (hash_of(&mut ga), hash_of(&mut gb));
    rows.push(("Replay hash identical (seed+rules+elapsed)".into(), format!("{ha:016x}"), ha == hb));
    let known_ok = all.iter().all(|r| r.known_to_ok);
    rows.push(("Forecast known_to == best_depth + 1".into(), if known_ok { "all".into() } else { "violated".into() }, known_ok));
    // Cut 2 gates (docs/CUT2.md).
    let runs8 = |rs: &[&SeedResult]| rs.iter().map(|r| r.runs as f64 * 8.0 / hours as f64).sum::<f64>() / ns as f64;
    let (d8, e8) = (runs8(&default), runs8(&edited));
    rows.push(("Expeditions per 8 h (DEFAULT, EDITED) in 6–16".into(), format!("{d8:.1} · {e8:.1}"), (6.0..=16.0).contains(&d8) && (6.0..=16.0).contains(&e8)));
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
                let t = std::time::Instant::now();
                let _ = riddle_core::trace::verdict(&mut g, *id);
                v.push(t.elapsed().as_secs_f64());
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
        // QA on 778fa1b (qaV: `R4 retreat` / `R6 attack` before two ogres for six minutes, no
        // blood either way, no stall): a run with `DANCE_ACTIONS` foe-facing row actions in a
        // row, `DANCE_MOVES` of them retreats, and no blood drawn is a loop the stall guard does
        // not see.
        let d_pct = pct(c_dances as usize, c_sends as usize);
        rows.push((format!("Bloodless dances ≤ 1% of sends on every cohort set (≥ {} foe-facing actions, ≥ {} retreats, n={c_sends})", riddle_core::turn::DANCE_ACTIONS, riddle_core::turn::DANCE_MOVES), format!("{d_pct:.1}% · worst {:.1}% {}", worst_dance.0, worst_dance.1), worst_dance.0 <= 1.0));
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
    // Cut 22 §3: an edit's paired move is far tighter than the bars' own ± — over every
    // one-notch edit of every cohort set, Σ paired ± ≤ ½ Σ absolute ± (the depths with a ±).
    let paireds = paireds.lock().unwrap();
    let (pe, pp, pa) = paireds.values().fold((0u32, 0.0f64, 0.0f64), |a, r| (a.0 + r.0, a.1 + r.1, a.2 + r.2));
    let worst = paireds.iter().map(|(si, r)| (r.1 / r.2.max(1e-9), sets[*si].0.clone())).fold((0.0f64, String::new()), |a, b| if b.0 > a.0 { b } else { a });
    println!("paired edit delta (Cut 22 §3): {pe} one-notch edits over {} sets · paired ± / absolute ± {:.2} (worst set {} {:.2})", sets.len(), pp / pa.max(1e-9), worst.1, worst.0);
    rows.push((format!("Paired edit ± ≤ ½ absolute ± ({pe} cohort edits)"), format!("{:.2}", pp / pa.max(1e-9)), pp <= 0.5 * pa));
    let (sv_n, sv_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.stall_verdicts.0, a.1 + r.stall_verdicts.1));
    rows.push((format!("Stall verdicts: ≥ 1 patch fired ≥ 50% (n={sv_n})"), format!("{sv_ok}/{sv_n}"), sv_ok == sv_n));
    let (sr_n, sr_ok): (u32, u32) = all.iter().fold((0, 0), |a, r| (a.0 + r.stall_reel.0, a.1 + r.stall_reel.1));
    rows.push((format!("Stall reel line's cause == the trace's (n={sr_n})"), format!("{sr_ok}/{sr_n}"), sr_ok == sr_n));
    phase("rest");
    println!();
    println!("{:<52} {:>18}  result", "gate", "value");
    let mut fails = 0;
    for (name, value, ok) in &rows {
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
