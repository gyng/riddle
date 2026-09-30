//! Cut 5 probe: story lines, reels and chronicle lines from real runs.
//!   cargo run -q --profile fast -p riddle-core --example stories -- [seeds] [hours]
use riddle_core::Game;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(3);
    let hours: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let good = riddle_core::probes::good();
    let mut sit: std::collections::BTreeMap<String, u32> = Default::default();
    let mut runs_with = 0u32;
    let mut runs = 0u32;
    for seed in 1..=seeds {
        for edited in [false, true] {
            let mut g = Game::new_literal(seed);
            if edited {
                for u in ["row5", "row6", "row7", "row8", "throw", "tame"] {
                    g.lineage.unlocks.insert(u.into());
                }
                g.set_rules_raw(good.clone()).unwrap();
            }
            let r = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
            println!("== seed {seed} {} · {} runs · best D{}", if edited { "EDITED" } else { "DEFAULT" }, r.runs, g.lineage.best_depth);
            for h in &r.reel {
                println!("  reel {:>2}  {}   {:?}", h.score, h.text, h.arc.as_ref().map(|a| (a.threat.as_str(), a.resolution.as_str())));
            }
            for l in &g.lineage.chronicle {
                println!("  chron  {l}");
            }
            let all: Vec<&riddle_core::Highlight> = g.batch.highlights.iter().collect();
            for h in all.iter().take(12) {
                println!("  ep {:>2}  {}", h.score, h.text);
            }
            runs_with += g.batch.situation_runs;
            runs += g.batch.run_outcomes.len() as u32;
            for h in &g.batch.highlights {
                if h.arc.is_some() && !riddle_core::sifter::story_ok(&h.text) {
                    println!("  BAD   {}", h.text);
                }
                if let Some(a) = &h.arc {
                    if ["shrine", "vault", "nest", "stray"].contains(&a.threat.as_str()) {
                        *sit.entry(a.threat.clone()).or_default() += 1;
                    }
                }
            }
        }
    }
    println!("situations: {sit:?} · runs with a situation {runs_with}/{runs}");
}
