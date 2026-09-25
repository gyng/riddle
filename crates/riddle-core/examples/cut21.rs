//! Cut 21 measures on the cohort sets (`eval/cards/<build>.<rater>.rules.json`):
//!   supplies  — an 8 h night per seed with a packed shelf (3 heal + strength): what the repeat
//!               spent, what the exits salvaged, and what went back on the shelf (§2);
//!   waystone  — ticks from the send to the first blow (either side) from a D1 start and from
//!               a lit D9 waystone on a D12+ lineage (§1).
//!   cargo run -q --profile fast -p riddle-core --example cut21 -- supplies|waystone [seeds] [sets…]
use riddle_core::{Ev, Game, RuleSet};

fn lineage_for(set: &RuleSet, seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see", "tame", "supply_cap_5"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink", "strength", "invisibility"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:monkey:thief", "foe:bloat_mother:boss"] {
        g.lineage.facts.insert(f.into());
    }
    for r in &set.rows {
        if let Some(c) = r.card() {
            g.lineage.unlocks.insert(c.into());
        }
        for c in &r.conds {
            if let Some(u) = riddle_core::meta::cond_unlock(&c.k) {
                g.lineage.unlocks.insert(u.into());
            }
        }
    }
    // A cohort-16 lineage at the return: fighter L4.
    if let Some(p) = g.lineage.classes.get_mut("fighter") {
        p.level = 4;
    }
    g.set_rules_raw(set.clone()).unwrap_or_else(|e| panic!("{:?}: {e}", set.name));
    g
}

fn sets(wanted: &[String]) -> Vec<(String, RuleSet)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            let pick = if wanted.is_empty() { stem.starts_with("44193ac") } else { wanted.iter().any(|w| stem.contains(w.as_str())) };
            if !pick {
                return None;
            }
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .collect();
    sets.sort_by(|a, b| a.0.cmp(&b.0));
    sets
}

fn supplies(seeds: u64, sets: &[(String, RuleSet)]) {
    for (name, set) in sets {
        let mut by_kind: std::collections::BTreeMap<String, (u32, i64)> = Default::default();
        let (mut heal_salv_n, mut stall_n) = (0u32, 0u32);
        let (mut spent, mut salvage, mut salvage_supply, mut home, mut runs, mut shelved, mut shelved_n) = (0i64, 0i64, 0i64, 0i64, 0u32, 0i64, 0u32);
        for seed in 1..=seeds {
            let mut g = lineage_for(set, seed);
            g.lineage.gold = 600;
            for k in ["heal", "heal", "heal", "strength"] {
                g.buy_supply(k).unwrap();
            }
            let r = g.run_offline(8 * 3600);
            runs += r.runs;
            stall_n += r.stalled;
            for x in &r.spent {
                let e = by_kind.entry(x.kind.clone()).or_default();
                e.0 += x.n;
                e.1 += x.gold as i64;
            }
            heal_salv_n += r.salvaged.iter().filter(|x| x.kind == "heal").map(|x| x.n).sum::<u32>();
            spent += r.spent.iter().map(|s| s.gold as i64).sum::<i64>();
            salvage += r.gold.as_ref().map(|s| s.salvage as i64).unwrap_or(0);
            home += r.gold.as_ref().map(|s| s.home as i64).unwrap_or(0);
            salvage_supply += r.salvaged.iter().filter(|s| s.kind.ends_with("heal") || s.kind.ends_with("strength")).map(|s| s.gold as i64).sum::<i64>();
            let v = serde_json::to_value(&r).unwrap();
            if let Some(a) = v.get("shelved").and_then(|a| a.as_array()) {
                for s in a {
                    shelved += s.get("gold").and_then(|x| x.as_i64()).unwrap_or(0);
                    shelved_n += s.get("n").and_then(|x| x.as_u64()).unwrap_or(0) as u32;
                }
            }
        }
        let s = seeds as f64;
        println!(
            "{name}: {seeds} nights · {:.1} runs/night · home ${:.0} · spent ${:.0} · salvage ${:.0} (heal+strength ${:.0}) · shelved {:.1} (${:.0} at the shelf's price) per night",
            runs as f64 / s,
            home as f64 / s,
            spent as f64 / s,
            salvage as f64 / s,
            salvage_supply as f64 / s,
            shelved_n as f64 / s,
            shelved as f64 / s
        );
        let kinds: Vec<String> = by_kind.iter().map(|(k, (n, g))| format!("{k} ×{:.1} ${:.0}", *n as f64 / s, *g as f64 / s)).collect();
        println!("    spent per night: {} · heals salvaged {:.1}/night · stalls {stall_n}", kinds.join(" · "), heal_salv_n as f64 / s);
    }
}

/// Ticks from the send to the first blow on the hero or by it (`Ev::Hurt` on either side) on a
/// floor at or below `from`.
fn first_fight(g: &mut Game, from: u32) -> Option<u32> {
    g.lineage.rest_left = 0;
    g.start_run(None);
    g.events.clear();
    let mut n = 0u32;
    while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 60_000 {
        g.tick();
        n += 1;
        if g.run.as_ref().is_some_and(|r| r.depth >= from) && g.events.iter().any(|e| matches!(e, Ev::Hurt { .. })) {
            g.run = None;
            g.events.clear();
            return Some(n);
        }
        g.events.clear();
    }
    g.run = None;
    None
}

fn waystone(seeds: u64, sets: &[(String, RuleSet)]) {
    for (name, set) in sets {
        let (mut d1, mut d9, mut n1, mut n9) = (0u64, 0u64, 0u32, 0u32);
        let (mut d1_9, mut n1_9) = (0u64, 0u32);
        for seed in 1..=seeds {
            let mut g = lineage_for(set, seed);
            g.lineage.best_depth = 12;
            g.lineage.banked_depths.insert(12);
            g.lineage.waystones = vec![5, 9];
            g.lineage.gold = 10_000;
            for _ in 0..4 {
                g.lineage.start = 1;
                if let Some(t) = first_fight(&mut g, 1) {
                    d1 += t as u64;
                    n1 += 1;
                }
                if let Some(t) = first_fight(&mut g, 9) {
                    d1_9 += t as u64;
                    n1_9 += 1;
                }
                g.lineage.start = 9;
                if let Some(t) = first_fight(&mut g, 9) {
                    d9 += t as u64;
                    n9 += 1;
                }
            }
        }
        println!(
            "{name}: first blow from a D1 start {:.0} ticks ({n1} runs); first blow on D9+ from a D1 start {:.0} ticks ({n1_9} runs reached it) · from a D9 start {:.0} ticks ({n9} runs)",
            d1 as f64 / n1.max(1) as f64,
            d1_9 as f64 / n1_9.max(1) as f64,
            d9 as f64 / n9.max(1) as f64
        );
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).cloned().unwrap_or_else(|| "supplies".into());
    let seeds: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    let wanted: Vec<String> = args.iter().skip(3).cloned().collect();
    let sets = sets(&wanted);
    match mode.as_str() {
        "supplies" => supplies(seeds, &sets),
        "waystone" => waystone(seeds, &sets),
        m => panic!("unknown mode {m}"),
    }
}
