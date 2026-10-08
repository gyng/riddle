//! Cut 113 §4 diagnostic: an ended save begins numbered descent N and plays 8 h check-ins
//! until the King falls again (≤ 14 days). Runs and hours to the next clear, per seed × tier.
//! Diagnostic only, never a gate (one earned build).
//! cargo run -q --profile fast -p riddle-core --example descent_check -- SAVE [TIERS=1,3] [--kit weapon=N,armour=N]
use riddle_core::{bloodlines::Session, endgame::Progress};
fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let raw = std::fs::read_to_string(args.get(1).expect("SAVE")).expect("read save");
    let tiers: Vec<u32> = args.get(2).map_or(vec![1, 3], |s| s.split(',').filter_map(|t| t.parse().ok()).collect());
    let kit: Vec<(String, u32)> = args.iter().position(|a| a == "--kit").and_then(|i| args.get(i + 1)).map_or(vec![], |s| {
        s.split(',').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (k.to_string(), v.parse().unwrap())).collect()
    });
    let mut base = Session::load(&raw).expect("valid save");
    // `--rules FILE`: a cohort rater's exported set in place of the save's (diagnostic)
    if let Some(f) = args.iter().position(|a| a == "--rules").and_then(|i| args.get(i + 1)) {
        let set: riddle_core::RuleSet = serde_json::from_str(&std::fs::read_to_string(f).unwrap()).expect("rule set");
        base.active.set_rules_raw(set).expect("rules apply");
    }
    let graves0 = base.active.lineage.graveyard.len();
    assert!(base.active.lineage.ended, "an ended save");
    let jobs: Vec<(u64, u32)> = [1u64, 3, 5, 7].iter().flat_map(|s| tiers.iter().map(move |t| (*s, *t))).collect();
    std::thread::scope(|sc| {
        let hs: Vec<_> = jobs
            .iter()
            .map(|&(seed, tier)| {
                let mut s = base.clone();
                let kit = kit.clone();
                sc.spawn(move || {
                    s.active.lineage.seed = seed;
                    s.active.lineage.endgame = Some(Progress { tier: 0, unlocked: tier.max(1), cleared: Some(tier.max(1) - 1) });
                    for (k, v) in kit {
                        s.active.lineage.kit.insert(k, v);
                    }
                    s.active.begin_descent(tier).expect("begin descent");
                    let (mut runs, mut checkins) = (0, 0);
                    for n in 1..=42 {
                        let r = s.run_offline_mode(28800, false, true);
                        runs += r.runs;
                        checkins = n;
                        if s.active.lineage.ended {
                            break;
                        }
                    }
                    let start = graves0.min(s.active.lineage.graveyard.len());
                    let mut deaths: std::collections::BTreeMap<String, u32> = Default::default();
                    for g in &s.active.lineage.graveyard[start..] {
                        if g.depth >= 24 { *deaths.entry(format!("D{} {}", g.depth, g.cause)).or_default() += 1; }
                    }
                    format!("seed {seed} tier {tier}: cleared {} · {} h · {} runs · best D{} · deaths ≥D24 {:?}", s.active.lineage.ended, 8 * checkins, runs, s.active.lineage.best_depth, deaths)
                })
            })
            .collect();
        for h in hs {
            println!("{}", h.join().unwrap());
        }
    });
}
