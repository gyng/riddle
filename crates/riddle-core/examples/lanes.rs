//! Cut 26 §3: the lanes of each fork (D5, D9).
//!   read:   every seed set (EDITED and the cohort sets) on both lanes, paired
//!   search: `--search FILE` — the plateau search per (fork, seed set, lane), written as it goes
//!   gate:   `--gate FILE` — the per-lane best sets' cross-lane loss, viability, gold/hr ratio and
//!           the diversity of the clears, on fresh paired seeds (the metrics row's measure)
//!   cargo run -q --profile fast -p riddle-core --example lanes -- [--sims 64] [--seeds 1] [--fork 5] [--steps 5] [--search F | --gate F] [filter]
#[path = "lanes_lib/mod.rs"]
mod lanes;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
    let path = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let sims = get("--sims", 64) as u32;
    let seeds = get("--seeds", 1);
    let steps = get("--steps", 5) as u32;
    let only_fork = get("--fork", 0) as u32;
    let filter: Option<&String> = args.iter().skip(1).find(|a| !a.starts_with("--") && a.parse::<u64>().is_err() && !a.ends_with(".json"));
    let mut sets = lanes::seed_sets();
    sets.retain(|(n, _)| filter.is_none_or(|f| f.split(',').any(|p| n.contains(p))));
    let forks: Vec<u32> = lanes::LANE_FORKS.into_iter().filter(|f| only_fork == 0 || *f == only_fork).collect();
    if let Some(file) = path("--gate") {
        let found: Vec<lanes::Found> = serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        let seeds: Vec<u64> = (1..=seeds.max(3)).collect();
        for fork in forks {
            let reads: Vec<Vec<(usize, f64, f64, f64)>> = seeds.iter().map(|s| lanes::gate_seed(&found, fork, *s, sims.max(96))).collect();
            let Some((g, win)) = lanes::gate_pool(&found, fork, &reads) else { continue };
            let [near, far] = lanes::lanes(fork);
            println!("fork D{fork} ({} · {}):", near.biome(fork).name(), far.biome(fork).name());
            for l in 0..2 {
                println!("  {} best {} — own {:.0}% · other lane {:.0}% · loss {:+.0} · $/h {:.0} · EDITED's best {:.0}%", ["near", "far"][l], found[win[l]].seed_set, 100.0 * g.own[l], 100.0 * g.cross[l], 100.0 * g.loss(l), g.gold[l], 100.0 * g.edited[l]);
                println!("    {}", found[win[l]].set.rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | "));
            }
            println!("  gold/hr ratio {:.2}", g.gold_ratio());
            let (d, c) = lanes::diversity(&found.iter().filter(|f| f.fork == fork).cloned().collect::<Vec<_>>());
            println!("  diversity {d}/{c} = {:.2}", d as f64 / c.max(1) as f64);
        }
        return;
    }
    if let Some(file) = path("--search") {
        let mut found: Vec<lanes::Found> = std::fs::read_to_string(&file).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        for &fork in &forks {
            for (name, set) in &sets {
                for (l, route) in lanes::lanes(fork).into_iter().enumerate() {
                    if found.iter().any(|f| f.fork == fork && f.lane == l && f.seed_set == *name) {
                        continue;
                    }
                    let t = std::time::Instant::now();
                    let g = lanes::lane_game(set, 1, fork);
                    let seed_set = g.lineage.rules().clone();
                    let (best, r) = lanes::climb(&g, &seed_set, route, fork, sims, steps, 0.03);
                    println!("D{fork} {} {name}: {:.0}% ({:.0}s) · {}", ["near", "far"][l], 100.0 * r.reach, t.elapsed().as_secs_f64(), best.rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | "));
                    found.push(lanes::Found { fork, lane: l, seed_set: name.clone(), set: best, reach: r.reach });
                    std::fs::write(&file, serde_json::to_string_pretty(&found).unwrap()).unwrap();
                }
            }
        }
        return;
    }
    for fork in forks {
        let [near, far] = lanes::lanes(fork);
        println!("fork D{fork}: near {} · far {} · bar D{} · start D{} · L{} · {} rows", near.biome(fork).name(), far.biome(fork).name(), lanes::bar(fork), lanes::start(fork), lanes::level(fork), lanes::rows_cap(fork));
        for seed in 1..=seeds {
            for (name, set) in &sets {
                let g = lanes::lane_game(set, seed, fork);
                let set = g.lineage.rules().clone();
                let a = lanes::read(&lanes::sends(&g, &set, near, fork, sims), fork);
                let b = lanes::read(&lanes::sends(&g, &set, far, fork, sims), fork);
                println!("  s{seed} {name:<24} near {:>3.0}% · far {:>3.0}% · $/h {:>5.0} {:>5.0}", 100.0 * a.reach, 100.0 * b.reach, a.gold_hr, b.gold_hr);
            }
        }
    }
}

