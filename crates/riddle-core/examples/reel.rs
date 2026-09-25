//! Cut 25 §5: the overnight reel's distinctness — an 8 h absence in the client's 30-minute
//! slices (`runOfflineQuick`), the slices' reels merged as the client merged them (top five by
//! score: `old`) and as it merges them now (one line per shape, `app.ts mergeReel`: `new`).
//!   cargo run -q --profile fast -p riddle-core --example reel -- [--seeds 6] [filter]
#[path = "lever_lib/mod.rs"]
mod lever;
use riddle_core::{Highlight, RuleSet};
use std::collections::BTreeMap;

fn shape(text: &str) -> String {
    let mut out = String::new();
    let mut digit = false;
    for c in text.chars() {
        if c.is_ascii_digit() {
            if !digit {
                out.push('N');
            }
            digit = true;
        } else {
            digit = false;
            out.push(c);
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.iter().position(|a| a == "--seeds").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(6);
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
    let (mut old_worst, mut new_worst, mut lines_old, mut shapes_old, mut lines_new) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for (name, set) in &sets {
        for seed in 1..=seeds {
            let mut g = lever::cohort_game(set, seed);
            g.lineage.gold = 400;
            lever::fill_shelf(&mut g);
            let mut all: Vec<Highlight> = Vec::new();
            let mut old: Vec<Highlight> = Vec::new();
            for _ in 0..16 {
                let r = riddle_core::offline::run_offline_quick(&mut g, 1800);
                all.extend(r.reel.iter().cloned());
                old.extend(r.reel.iter().cloned());
                old.sort_by_key(|b| std::cmp::Reverse(b.score));
                old.truncate(5);
            }
            let mut by: BTreeMap<String, usize> = BTreeMap::new();
            for h in &old {
                *by.entry(shape(&h.text)).or_insert(0) += 1;
            }
            let rep = by.values().copied().max().unwrap_or(0);
            old_worst = old_worst.max(rep);
            lines_old += old.len();
            shapes_old += by.len();
            // new: one per shape, top five by score
            let mut best: BTreeMap<String, &Highlight> = BTreeMap::new();
            for h in &all {
                let k = shape(&h.text);
                if best.get(&k).is_none_or(|b| h.score > b.score) {
                    best.insert(k, h);
                }
            }
            let n = best.len().min(5);
            lines_new += n;
            new_worst = new_worst.max(1);
            if rep > 2 {
                println!("{name} s{seed}: old reel repeats a shape ×{rep}: {:?}", old.iter().map(|h| h.text.as_str()).collect::<Vec<_>>());
            }
        }
    }
    println!("old merge: {lines_old} lines, {shapes_old} shapes ({:.0}% distinct), worst repeat ×{old_worst}; new merge: {lines_new} lines, all distinct shapes (repeat ×{new_worst})", 100.0 * shapes_old as f64 / lines_old.max(1) as f64);
}
