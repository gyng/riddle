//! Fourteen simulated days of a plausible player: check in N times a day, read the worst death,
//! apply its best patch when it clearly beats the baseline, buy the cheapest affordable unlock,
//! field the kennel. Prints a per-day table and the pacing/progression probes from
//! docs/FUN_EVAL_IDLE.md §5 (unlock cadence, empty check-ins, wall stalls).
//!   cargo run -q --profile fast -p riddle-core --example dayplayer -- [--seeds 3] [--days 14] [--checkins 3]
use riddle_core::Game;

struct Day { best: u32, unlocks: u32, edits: u32, empty: u32, checkins: u32, runs: u32, deaths: u32, marks: u32, gold: i32, facts: usize, level: u32, rank: u32, rows: usize, kennel: usize }

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 3);
    let days = get("--days", 14) as usize;
    let checkins = get("--checkins", 3) as u64;
    let interval = 24 * 3600 / checkins;
    let mut all: Vec<Vec<Day>> = Vec::new();
    for seed in 1..=seeds {
        let mut g = Game::new(seed);
        let mut table: Vec<Day> = Vec::new();
        for day in 0..days {
            let mut d = Day { best: 0, unlocks: 0, edits: 0, empty: 0, checkins: 0, runs: 0, deaths: 0, marks: 0, gold: 0, facts: 0, level: 0, rank: 0, rows: 0, kennel: 0 };
            for _ in 0..checkins {
                let rep = riddle_core::offline::run_offline_quick(&mut g, interval);
                d.checkins += 1;
                d.runs += rep.runs;
                d.deaths += rep.deaths.iter().map(|x| x.n).sum::<u32>();
                let mut decided = false;
                // 1. the worst death: patch if the top candidate clearly beats the baseline
                if let Some(id) = rep.worst_death_id {
                    if let Some(death) = g.death(id) {
                        if death.verdict == "gap" {
                            if let Some(p) = death.patches.first() {
                                if p.survive > death.baseline + 0.15 {
                                    let max_rows = g.vocabulary().max_rows;
                                    let mut rules = g.lineage.rules().clone();
                                    if rules.rows.len() >= max_rows { rules.rows.pop(); }
                                    let at = p.insert_at.min(rules.rows.len());
                                    rules.rows.insert(at, p.row.clone());
                                    if g.set_rules(rules).is_ok() { d.edits += 1; decided = true; }
                                }
                            }
                        }
                    }
                }
                // 2. cheapest affordable unlock
                let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.available && u.cost <= g.lineage.marks).collect();
                opts.sort_by_key(|u| u.cost);
                if let Some(u) = opts.first() { if g.buy(&u.id).is_ok() { d.unlocks += 1; decided = true; } }
                // 3. field the kennel
                let slots = g.lineage.party_slots() as usize;
                if g.lineage.party.len() < slots && !g.lineage.kennel.is_empty() {
                    let mut ids: Vec<u32> = g.lineage.party.iter().map(|c| c.id).collect();
                    let mut k: Vec<_> = g.lineage.kennel.iter().filter(|c| !ids.contains(&c.id)).collect();
                    k.sort_by_key(|c| std::cmp::Reverse(c.level));
                    for c in k.iter().take(slots - ids.len()) { ids.push(c.id); }
                    if g.set_party(ids).is_ok() { decided = true; }
                }
                if !decided && rep.learned.is_empty() && rep.pending.is_empty() { d.empty += 1; }
            }
            d.best = g.lineage.best_depth; d.marks = g.lineage.marks; d.gold = g.lineage.gold; d.facts = g.lineage.facts.len();
            d.level = g.lineage.classes.get(g.lineage.class.name()).map(|c| c.level).unwrap_or(1); d.rank = g.lineage.rank;
            d.rows = g.lineage.rules().rows.len(); d.kennel = g.lineage.kennel.len() + g.lineage.party.len();
            let _ = day;
            table.push(d);
        }
        println!("seed {seed}: rules now = {}", g.export_rules().replace('\n', " | "));
        println!("day  best  unl  edit  empty/ci  runs  deaths  marks  gold  facts  L  rank  rows  pets");
        for (i, d) in table.iter().enumerate() {
            println!("{:>3}  {:>4}  {:>3}  {:>4}  {:>4}/{:<3}  {:>4}  {:>6}  {:>5}  {:>4}  {:>5}  {:>1}  {:>4}  {:>4}  {:>4}", i + 1, d.best, d.unlocks, d.edits, d.empty, d.checkins, d.runs, d.deaths, d.marks, d.gold, d.facts, d.level, d.rank, d.rows, d.kennel);
        }
        all.push(table);
    }
    // Probes
    let n = all.len() as f64;
    let days_with_unlock = all.iter().map(|t| t.iter().filter(|d| d.unlocks > 0).count()).sum::<usize>() as f64 / n;
    let empty = all.iter().map(|t| t.iter().map(|d| d.empty).sum::<u32>()).sum::<u32>() as f64;
    let cis = all.iter().map(|t| t.iter().map(|d| d.checkins).sum::<u32>()).sum::<u32>() as f64;
    let longest_stall = all.iter().map(|t| { let mut best = 0; let mut cur = 0; let mut last = 0; for d in t { if d.best > last { cur = 0; last = d.best; } else { cur += 1; } best = best.max(cur); } best }).max().unwrap_or(0);
    let final_best = all.iter().map(|t| t.last().map(|d| d.best).unwrap_or(0)).collect::<Vec<_>>();
    println!("\nprobes over {seeds} seeds × {days} days × {checkins}/day");
    println!("  days with ≥1 unlock (mean)      {days_with_unlock:.1} / {days}   bar: ≥ {}", (days as f64 * 0.7).ceil());
    println!("  empty check-ins                 {:.0}%   bar: ≤ 15%", 100.0 * empty / cis);
    println!("  longest best-depth stall (days) {longest_stall}   bar: ≤ 3");
    println!("  final best depth per seed       {final_best:?}   ending at 16");
}
