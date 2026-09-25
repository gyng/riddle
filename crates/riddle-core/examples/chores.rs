//! Cut 25 §3: the chore loop — per floor of every send, the `pick up` chores (row −2) and the
//! trait's (`greedy → pick up`, row −1); prints the worst floors and, with `--dump`, the worst
//! floor's pick-ups with the items around the hero.
//!   cargo run -q --profile fast -p riddle-core --example chores -- [--seeds 4] [--runs 12] [filter] [--dump]
#[path = "lever_lib/mod.rs"]
mod lever;
use riddle_core::{Ev, RuleSet};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 4);
    let runs = get("--runs", 12);
    let dump = args.iter().any(|a| a == "--dump");
    let filter: Option<&String> = args.iter().skip(1).find(|a| !a.starts_with("--") && a.parse::<u64>().is_err());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .filter(|(n, _)| filter.is_none_or(|f| n.contains(f.as_str())))
        .collect();
    sets.sort_by(|a, b| a.0.cmp(&b.0));
    let mut worst_all: Vec<(u32, String)> = Vec::new();
    for (name, set) in &sets {
        let mut hist = [0u32; 6];
        let mut worst = (0u32, String::new());
        let mut dry_worst = 0u32;
        for seed in 1..=seeds {
            let mut g = lever::cohort_game(set, seed);
            g.lineage.gold = 400;
            lever::fill_shelf(&mut g);
            for run in 0..runs {
                g.lineage.rest_left = 0;
                g.start_run(None);
                let mut per: u32 = 0;
                let mut dry: u32 = 0;
                let mut log: Vec<String> = Vec::new();
                let mut depth = 1;
                let mut n = 0;
                let flush = |per: u32, depth: u32, log: &mut Vec<String>, hist: &mut [u32; 6], worst: &mut (u32, String)| {
                    let b = match per {
                        0..=10 => 0,
                        11..=25 => 1,
                        26..=50 => 2,
                        51..=100 => 3,
                        101..=200 => 4,
                        _ => 5,
                    };
                    hist[b] += 1;
                    if per > worst.0 && std::env::var("DRY").is_err() {
                        *worst = (per, format!("seed {seed} run {run} D{depth}: {per}\n{}", log.join("\n")));
                    }
                    log.clear();
                };
                while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 200_000 {
                    g.tick();
                    n += 1;
                    let r = g.run.as_ref().unwrap();
                    for e in &g.events {
                        match e {
                            Ev::Descend { depth: d, .. } => {
                                flush(per, depth, &mut log, &mut hist, &mut worst);
                                per = 0;
                                dry = 0;
                                depth = *d;
                            }
                            Ev::Pickup { id: 1, .. } => dry = 0,
                            Ev::Rule { t, row, verb, text } if verb.v == "pick_up" && *row < 0 => {
                                per += 1;
                                dry += 1;
                                if dry > dry_worst && std::env::var("DRY").is_ok() {
                                    worst = (dry, format!("seed {seed} run {run} D{depth} dry {dry}\n{}", log.join("\n")));
                                }
                                dry_worst = dry_worst.max(dry);
                                if dump {
                                    let hp = r.hero.pos;
                                    let near: Vec<String> = r.items.iter().filter(|i| i.pos.cheb(hp) <= 6).map(|i| format!("{}@{},{}", i.item.kind, i.pos.x - hp.x, i.pos.y - hp.y)).collect();
                                    log.push(format!("{t:>6} R{row} {text} hero ({},{}) hp {} alert {} inv {} {:?} near {:?} ignored {}", hp.x, hp.y, r.hero.hp, r.alert, r.hero.inv.len(), r.hero.inv.iter().map(|i| i.kind.as_str()).collect::<Vec<_>>(), near, r.items_ignored()));
                                }
                            }
                            Ev::Rule { t, row, text, .. } if dump && std::env::var("ALL").is_ok() => log.push(format!("{t:>6} R{row} {text} hero ({},{}) hp {}", r.hero.pos.x, r.hero.pos.y, r.hero.hp)),
                            Ev::Hurt { t, id: 1, dmg, cause, .. } if dump && std::env::var("ALL").is_ok() => log.push(format!("{t:>6}   hurt {dmg} {cause}")),
                            _ => {}
                        }
                    }
                    g.events.clear();
                }
                flush(per, depth, &mut log, &mut hist, &mut worst);
                g.events.clear();
            }
        }
        println!("{name}: floors by pick-ups ≤10 {} · ≤25 {} · ≤50 {} · ≤100 {} · ≤200 {} · >200 {} · worst {} · longest dry streak {dry_worst}", hist[0], hist[1], hist[2], hist[3], hist[4], hist[5], worst.0);
        worst_all.push((worst.0, format!("{name} {}", worst.1)));
    }
    if dump {
        worst_all.sort_by_key(|b| std::cmp::Reverse(b.0));
        if let Some((_, w)) = worst_all.first() {
            println!("{w}");
        }
    }
}
