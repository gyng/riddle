//! Cut 26 §1: the 307dbed compatibility fixture — a lineage played on 307dbed (every unlock and
//! fact, the FULL set, an hour away), saved, then its next 10 sends hashed (`riddle_core::tests::sends_hash`
//! mirrors `hash_sends` here). `--make DIR` writes `save_307dbed.json` and `sends_307dbed.txt`;
//! without it, prints the hash of the fixture's next 10 sends on this build.
//!   cargo run -q --profile fast -p riddle-core --example compat [-- --make crates/riddle-core/src/fixtures]
use riddle_core::{Ev, Game};

fn fnv(h: &mut u64, s: &str) {
    for b in s.bytes() {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x100000001b3);
    }
}

/// The next `n` sends: every event but the Cut 26 fork beats, and each run's end state.
fn hash_sends(g: &mut Game, n: u32) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for _ in 0..n {
        g.send();
        for _ in 0..400 {
            let r = g.step(200);
            for e in &r.events {
                if matches!(e, Ev::Callout { text, .. } if text == "TWO STAIRS") || matches!(e, Ev::Fact { fact, .. } if fact.starts_with("fork:")) {
                    continue;
                }
                fnv(&mut h, &serde_json::to_string(e).unwrap());
            }
            if r.run_over {
                break;
            }
        }
        fnv(&mut h, &format!("{} {} {}", g.lineage.gold, g.lineage.best_depth, g.deaths.len()));
    }
    h
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let make = args.iter().position(|a| a == "--make").and_then(|i| args.get(i + 1)).cloned();
    if let Some(dir) = make {
        // Two lineages in one fixture would double it; one deep one: every unlock and fact,
        // the FULL set (as `fingerprint`), an hour away — it walls past D8.
        let mut g = Game::new_literal(307);
        for u in riddle_core::meta::UNLOCKS {
            g.lineage.unlocks.insert(u.id.into());
        }
        riddle_core::probes::learn_everything(&mut g);
        g.set_rules(riddle_core::probes::full()).expect("full rules");
        g.run_offline(3600);
        let text = g.save();
        std::fs::write(format!("{dir}/save_307dbed.json"), &text).unwrap();
        let mut h = Game::load(&text).unwrap();
        let hash = hash_sends(&mut h, 10);
        std::fs::write(format!("{dir}/sends_307dbed.txt"), format!("{hash:016x}\n")).unwrap();
        println!("wrote {dir}: {} bytes, best D{}, hash {hash:016x}", text.len(), h.lineage.best_depth);
    } else {
        let mut g = Game::load(&std::fs::read_to_string("crates/riddle-core/src/fixtures/save_307dbed.json").unwrap()).unwrap();
        println!("{:016x}", hash_sends(&mut g, 10));
    }
}
