//! Death-cause survey for one rule set: which causes end runs at each depth.
//!   cargo run --release --example probe -- presets/good.json 8 6
use riddle_core::{Game, RuleSet};
use std::collections::BTreeMap;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap_or("presets/good.json".into());
    let hours: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(4);
    let seeds: u64 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(6);
    let set = RuleSet::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut by_depth: BTreeMap<u32, BTreeMap<String, u32>> = BTreeMap::new();
    let mut runs = 0;
    for seed in 1..=seeds {
        let mut g = Game::new(seed);
        for u in ["row5", "row6", "row7", "row8"] {
            g.lineage.unlocks.insert(u.into());
        }
        for k in ["heal", "poison", "fire", "teleport", "blink"] {
            if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
                g.lineage.facts.insert(f);
            }
        }
        for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss"] {
            g.lineage.facts.insert(f.into());
        }
        g.lineage.unlocks.insert("throw".into());
        g.set_rules(set.clone()).unwrap();
        g.run_offline(hours * 3600);
        for (d, c) in &g.batch.run_outcomes {
            runs += 1;
            *by_depth.entry(*d).or_default().entry(c.clone().unwrap_or("survived".into())).or_insert(0) += 1;
        }
    }
    println!("{runs} runs");
    for (d, causes) in by_depth {
        let mut cv: Vec<(&String, &u32)> = causes.iter().collect();
        cv.sort_by(|a, b| b.1.cmp(a.1));
        let total: u32 = causes.values().sum();
        let s: Vec<String> = cv.iter().take(6).map(|(c, n)| format!("{c} {n}")).collect();
        println!("D{d:<2} {total:>4}: {}", s.join(", "));
    }
}
