//! Death-cause survey for one rule set: which causes end runs at each depth, which rows end
//! the runs that come home, and the quiet (single-threaded) per-tick cost.
//!   cargo run --release --example probe -- presets/good.json 8 6 [--full]
//! `--full` sets the lineage up like the FULL bot (every unlock and fact, a mastered fighter).
use riddle_core::hero::Class;
use riddle_core::{Game, RuleSet};
use std::collections::BTreeMap;
use std::time::Instant;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let full = args.iter().any(|a| a == "--full");
    let pos: Vec<&String> = args.iter().skip(1).filter(|a| !a.starts_with("--")).collect();
    let path = pos.first().map(|s| s.to_string()).unwrap_or("presets/good.json".into());
    let hours: u64 = pos.get(1).and_then(|s| s.parse().ok()).unwrap_or(4);
    let seeds: u64 = pos.get(2).and_then(|s| s.parse().ok()).unwrap_or(6);
    let set = RuleSet::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut by_depth: BTreeMap<u32, BTreeMap<String, u32>> = BTreeMap::new();
    let mut exit_rows: BTreeMap<i32, u32> = BTreeMap::new();
    let mut runs = 0;
    let mut ticks: u64 = 0;
    let mut secs = 0.0;
    let mut best: Vec<u32> = Vec::new();
    for seed in 1..=seeds {
        let mut g = Game::new(seed);
        if full {
            for u in riddle_core::meta::UNLOCKS {
                g.lineage.unlocks.insert(u.id.into());
            }
            riddle_core::probes::learn_everything(&mut g);
            g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0 });
            g.lineage.unlocks.insert(riddle_core::hero::mastery_card(Class::Fighter).into());
        } else {
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
        }
        g.set_rules_raw(set.clone()).unwrap();
        let t = Instant::now();
        riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        secs += t.elapsed().as_secs_f64();
        ticks += g.batch.turns as u64;
        for (d, c) in &g.batch.run_outcomes {
            runs += 1;
            *by_depth.entry(*d).or_default().entry(c.clone().unwrap_or("home".into())).or_insert(0) += 1;
        }
        for (r, n) in &g.batch.exit_rows {
            *exit_rows.entry(*r).or_insert(0) += n;
        }
        best.push(g.lineage.best_depth);
    }
    println!("{runs} runs · best depth per seed {best:?} · {:.2} µs/tick quiet ({ticks} ticks)", secs * 1e6 / ticks.max(1) as f64);
    println!("exit rows (row → runs ended by it): {exit_rows:?}");
    for (d, causes) in by_depth {
        let mut cv: Vec<(&String, &u32)> = causes.iter().collect();
        cv.sort_by(|a, b| b.1.cmp(a.1));
        let total: u32 = causes.values().sum();
        let s: Vec<String> = cv.iter().take(6).map(|(c, n)| format!("{c} {n}")).collect();
        println!("D{d:<2} {total:>4}: {}", s.join(", "));
    }
}
