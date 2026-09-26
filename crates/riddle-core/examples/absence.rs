//! The runs a short absence yields (the 307dbed control rater AR: a 20-minute absence read `0 RUNS`):
//! a fresh-ish lineage (the preset, or the good set) watches one run to its exit, then is away
//! 20 m, 4 h or 8 h (each from the same save). Prints the runs each absence yields.
//!   cargo run -q --profile fast -p riddle-core --example absence [-- --seeds 6]
use riddle_core::Game;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.iter().position(|a| a == "--seeds").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(6);
    for (label, good) in [("preset", false), ("good", true)] {
        for (away, secs) in [("20m", 20 * 60), ("25m", 25 * 60), ("4h", 4 * 3600), ("8h", 8 * 3600)] {
            let mut runs = Vec::new();
            for seed in 1..=seeds {
                let mut g = Game::new(seed);
                if good {
                    for u in ["row5", "row6", "row7", "row8", "throw"] {
                        g.lineage.unlocks.insert(u.into());
                    }
                    g.set_rules_raw(riddle_core::probes::good()).unwrap();
                }
                // One watched run to its exit (the player saw it), its keep sheet settled.
                g.send();
                for _ in 0..2000 {
                    if g.step(200).run_over {
                        break;
                    }
                }
                let _ = g.keep(Vec::new());
                let text = g.save();
                let mut h = Game::load(&text).unwrap();
                runs.push(h.run_offline(secs).runs);
            }
            println!("{label:<6} away {away:<4} runs {:?} (min {})", runs, runs.iter().min().unwrap());
        }
    }
}
