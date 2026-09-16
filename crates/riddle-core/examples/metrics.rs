//! Bot gates (docs/CUT1.md): 30 seeds × the 8 h offline model per bot. Prints PASS/FAIL and
//! exits non-zero on any FAIL. Never weaken a gate; tune content.
//!   cargo run --release --example metrics [-- --seeds 30 --hours 8]
use riddle_core::engine::ExitTier;
use riddle_core::hero::Class;
use riddle_core::rng::Rng;
use riddle_core::{Game, RuleSet};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

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
}

const BOTS: [Bot; 9] = [Bot::Default, Bot::Edited, Bot::Random, Bot::Passive, Bot::Learned, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered];

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
}

fn good() -> RuleSet {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/presets/good.json")).expect("presets/good.json");
    RuleSet::parse(&text).expect("good.json parses")
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
            g.lineage.unlocks.insert("throw".into());
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
            g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0 });
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
            g.set_rules(if bot == Bot::Trivial { riddle_core::probes::trivial() } else { riddle_core::probes::countered() }).unwrap();
        }
    }
    g
}

fn run_seed(bot: Bot, seed: u64, hours: u64, verdicts_per_seed: usize) -> SeedResult {
    let mut g = setup(bot, seed);
    let report = g.run_offline(hours * 3600);
    let mut r = SeedResult {
        best_depth: g.lineage.best_depth,
        learned: report.learned.len(),
        pending: report.pending.len(),
        events: g.batch.renderable_events,
        ticks: g.batch.turns,
        runs: report.runs,
        ..Default::default()
    };
    r.run_ticks = g.batch.run_ticks.clone();
    for (d, c) in &g.batch.run_outcomes {
        r.run_depths.push(*d);
        if let Some(c) = c {
            r.causes.push(c.clone());
        }
    }
    // Verdicts cost ~1.2 s each (candidates × reseeded replays); sample evenly across the seed's
    // deaths. 8 per seed × seeds × bots is plenty for the unfair/gap shares.
    let ids: Vec<u32> = g.deaths.keys().copied().collect();
    let step = (ids.len() / verdicts_per_seed).max(1);
    for id in ids.iter().step_by(step).take(verdicts_per_seed) {
        if let Some(v) = riddle_core::trace::verdict(&mut g, *id) {
            r.verdicts.push(v);
        }
    }
    let f = g.forecast();
    r.known_to_ok = f.known_to == g.lineage.best_depth + 1;
    r
}

fn pct(n: usize, d: usize) -> f64 {
    if d == 0 {
        0.0
    } else {
        100.0 * n as f64 / d as f64
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    // --quick: 8 seeds × 8 h × 3 verdicts (≈ 30 s; depth gates are 8 h tail gates, so hours stay).
    // Full: 30 × 8 h × 8 (≈ 2 min) is the number that counts. Never weaken bars.
    let quick = args.iter().any(|a| a == "--quick");
    let seeds = get("--seeds", if quick { 8 } else { 30 });
    let hours = get("--hours", 8);
    let verdicts_per_seed = get("--verdicts", if quick { 3 } else { 8 }) as usize;
    let t_start = std::time::Instant::now();
    let results: Arc<Mutex<BTreeMap<(usize, u64), SeedResult>>> = Arc::new(Mutex::new(BTreeMap::new()));
    let jobs: Vec<(usize, u64)> = BOTS.iter().enumerate().flat_map(|(bi, _)| (1..=seeds).map(move |s| (bi, s))).collect();
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).min(32);
    let jobs = Arc::new(Mutex::new(jobs));
    let mut handles = Vec::new();
    for _ in 0..threads {
        let jobs = Arc::clone(&jobs);
        let results = Arc::clone(&results);
        handles.push(std::thread::spawn(move || loop {
            let job = jobs.lock().unwrap().pop();
            let Some((bi, seed)) = job else { break };
            let r = run_seed(BOTS[bi], seed, hours, verdicts_per_seed);
            results.lock().unwrap().insert((bi, seed), r);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let results = results.lock().unwrap();
    let per_bot = |bot: Bot| -> Vec<&SeedResult> {
        let bi = BOTS.iter().position(|b| *b == bot).unwrap();
        (1..=seeds).map(|s| &results[&(bi, s)]).collect()
    };
    let ns = seeds as usize;
    let mut rows: Vec<(String, String, bool)> = Vec::new();
    // Per-bot summary.
    println!("bot        best-depth mean  ≤D6  ≥D10  mean-death-depth  runs  deaths");
    for bot in BOTS {
        let rs = per_bot(bot);
        let mean_best = rs.iter().map(|r| r.best_depth as f64).sum::<f64>() / ns as f64;
        let le6 = rs.iter().filter(|r| r.best_depth <= 6).count();
        let ge10 = rs.iter().filter(|r| r.best_depth >= 10).count();
        let depths: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        let mdd = depths.iter().sum::<u32>() as f64 / depths.len().max(1) as f64;
        let runs: u32 = rs.iter().map(|r| r.runs).sum();
        let deaths: usize = rs.iter().map(|r| r.causes.len()).sum();
        println!("{:<10} {:>15.2} {:>4} {:>5} {:>17.2} {:>5} {:>7}", bot.name(), mean_best, le6, ge10, mdd, runs, deaths);
    }
    println!("\nrun depth histogram (% of runs ending at depth ≥ d):");
    for bot in BOTS {
        let rs = per_bot(bot);
        let depths: Vec<u32> = rs.iter().flat_map(|r| r.run_depths.iter().copied()).collect();
        let n = depths.len().max(1);
        let cells: Vec<String> = (1..=12).map(|d| format!("D{d} {:>4.1}", pct(depths.iter().filter(|x| **x >= d).count(), n))).collect();
        println!("{:<9} {}", bot.name(), cells.join(" │ "));
    }
    println!("\nrun length (ticks): p10 / median / p90 / max, share in 1800–4800");
    for bot in [Bot::Default, Bot::Edited, Bot::Learned, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered] {
        let mut t: Vec<u32> = per_bot(bot).iter().flat_map(|r| r.run_ticks.iter().copied()).collect();
        t.sort();
        let q = |f: f64| t.get(((t.len() as f64 - 1.0) * f) as usize).copied().unwrap_or(0);
        let band = pct(t.iter().filter(|x| (1800..=4800).contains(*x)).count(), t.len());
        println!("{:<9} {:>6} / {:>6} / {:>6} / {:>6}   {band:.0}%", bot.name(), q(0.1), q(0.5), q(0.9), t.last().copied().unwrap_or(0));
    }
    let default = per_bot(Bot::Default);
    let edited = per_bot(Bot::Edited);
    let d_le6 = pct(default.iter().filter(|r| r.best_depth <= 6).count(), ns);
    rows.push(("DEFAULT dies by ≤ D6 ≥ 80% of seeds".into(), format!("{d_le6:.0}%"), d_le6 >= 80.0));
    let e_ge10 = pct(edited.iter().filter(|r| r.best_depth >= 10).count(), ns);
    let d_ge10 = pct(default.iter().filter(|r| r.best_depth >= 10).count(), ns);
    rows.push(("EDITED reaches ≥ D10 ≥ 50% of seeds".into(), format!("{e_ge10:.0}%"), e_ge10 >= 50.0));
    rows.push(("EDITED − DEFAULT (≥ D10) ≥ 15 pts".into(), format!("{:.0} pts", e_ge10 - d_ge10), e_ge10 - d_ge10 >= 15.0));
    let random = per_bot(Bot::Random);
    let r_lose = pct(random.iter().filter(|r| r.best_depth < 16).count(), ns);
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
    let triv_le5 = pct(triv.iter().filter(|r| r.best_depth <= 5).count(), ns);
    rows.push(("TRIVIAL never passes D5 ≥ 90% of seeds".into(), format!("{triv_le5:.0}%"), triv_le5 >= 90.0));
    let ctr = per_bot(Bot::Countered);
    let ctr_ge11 = pct(ctr.iter().filter(|r| r.best_depth >= 11).count(), ns);
    rows.push(("COUNTERED reaches ≥ D11 ≥ 50% of seeds".into(), format!("{ctr_ge11:.0}%"), ctr_ge11 >= 50.0));
    // Verdicts and causes across bots.
    let all: Vec<&SeedResult> = BOTS.iter().flat_map(|b| per_bot(*b)).collect();
    let verdicts: Vec<&String> = all.iter().flat_map(|r| r.verdicts.iter()).collect();
    let dice = pct(verdicts.iter().filter(|v| v.as_str() == "dice").count(), verdicts.len());
    let gap = pct(verdicts.iter().filter(|v| v.as_str() == "gap").count(), verdicts.len());
    rows.push((format!("Unfair deaths (dice) ≤ 5% (n={})", verdicts.len()), format!("{dice:.1}%"), dice <= 5.0));
    rows.push(("Deaths tracing to a row (gap) ≥ 70%".into(), format!("{gap:.1}%"), gap >= 70.0));
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
        g2.set_rules(good()).unwrap();
        let rep = g2.run_offline(1800);
        let s = serde_json::to_string(&rep).unwrap() + &g2.save();
        for b in s.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    };
    let mut ga = Game::new(7);
    ga.set_rules(good()).unwrap();
    let mut gb = Game::new(7);
    gb.set_rules(good()).unwrap();
    let (ha, hb) = (hash_of(&mut ga), hash_of(&mut gb));
    rows.push(("Replay hash identical (seed+rules+elapsed)".into(), format!("{ha:016x}"), ha == hb));
    let known_ok = all.iter().all(|r| r.known_to_ok);
    rows.push(("Forecast known_to == best_depth + 1".into(), if known_ok { "all".into() } else { "violated".into() }, known_ok));
    // Player-shaped lineages only: LEARNED knows everything by construction, RANDOM/PASSIVE are probes.
    let player_bots: Vec<&SeedResult> = [Bot::Default, Bot::Edited, Bot::Pets, Bot::Levelled, Bot::Trivial, Bot::Countered].iter().flat_map(|b| per_bot(*b)).collect();
    let off_ok = player_bots.iter().all(|r| r.learned >= 1 && r.pending >= 1);
    let off_min = player_bots.iter().map(|r| r.learned.min(r.pending)).min().unwrap_or(0);
    rows.push((format!("Offline {hours} h: learned ≥ 1 and pending ≥ 1 every seed"), format!("min {off_min}"), off_ok));
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
