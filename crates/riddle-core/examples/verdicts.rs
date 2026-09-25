//! Cut 25 §2: a set's deaths and their verdicts — `order`, `row`, `gap`, `dice` — over absences on
//! several seeds (the set's lineage as the gate table builds it); for each `dice`, the recent own
//! rows' cuts (`trace::recent_own_cuts`) and the moves (`DeathRec.moves`).
//!   cargo run -q --profile fast -p riddle-core --example verdicts -- set.json [--seeds 6] [--hours 8] [--level 3]
#[path = "lever_lib/mod.rs"]
mod lever;
use riddle_core::RuleSet;
use std::collections::BTreeMap;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
    let path = args.iter().skip(1).find(|a| a.ends_with(".json")).expect("a set");
    let set = RuleSet::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    let (seeds, hours, level) = (get("--seeds", 6), get("--hours", 8), get("--level", 3) as u32);
    let mut tally: BTreeMap<String, u32> = BTreeMap::new();
    for seed in 1..=seeds {
        let mut g = lever::cohort_game(&set, seed);
        g.lineage.classes.insert(g.lineage.class.name().into(), riddle_core::wire::ClassProg { level, xp: 0, next: 0 });
        g.lineage.gold = 400;
        lever::fill_shelf(&mut g);
        let _ = riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall).map(|(id, _)| *id).collect();
        for id in ids.iter().rev().take(8) {
            let Some(d) = g.death(*id) else { continue };
            *tally.entry(d.verdict.clone()).or_insert(0) += 1;
            let rec = g.deaths.get(id).cloned().unwrap();
            let moves: Vec<String> = rec.moves.iter().map(|p| format!("R{}→above R{} {:.2}", p.moves_from.unwrap_or(0) + 1, p.insert_at + 1, p.survive)).collect();
            let extra = if d.verdict == "dice" { format!(" · cuts {:?}", riddle_core::trace::recent_own_cuts(&g, &rec).iter().map(|(r, s)| format!("R{} {s:.2}", r + 1)).collect::<Vec<_>>()) } else { String::new() };
            println!("s{seed} run {id} D{} {} · {} base {:.2} · cause R{:?} over R{:?} · last rows {:?} · moves {:?}{extra}", d.depth, d.cause, d.verdict, d.baseline, d.cause_row.map(|r| r + 1), d.order_over.map(|r| r + 1), d.trace.turns.iter().rev().take(5).map(|t| t.row + 1).collect::<Vec<_>>(), moves);
        }
    }
    println!("{tally:?}");
}
