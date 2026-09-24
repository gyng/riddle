//! Cut 20 measures on the cohort sets (`eval/cards/<build>.<rater>.rules.json`): thefts and
//! den wakes per run (§1), pet deaths per run on a lineage with two tamed jackals (§2), and
//! the bounty floor's take (§5).
//!   cargo run -q --profile fast -p riddle-core --example cut20 -- [seeds] [runs] [sets…]
use riddle_core::{Game, RuleSet};

fn lineage_for(set: &RuleSet, seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see", "tame", "party_slot_2"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink", "strength", "invisibility"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:monkey:thief"] {
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
    g.set_rules_raw(set.clone()).unwrap_or_else(|e| panic!("{:?}: {e}", set.name));
    g
}

fn jackals() -> Vec<riddle_core::wire::Companion> {
    let def = riddle_core::defs::monster_def("jackal");
    let tags = vec!["pack".to_string(), "fast".to_string()];
    let mk = |id: u32, name: &str| riddle_core::wire::Companion {
        id,
        kind: "jackal".into(),
        name: name.into(),
        level: 2,
        tags: tags.clone(),
        gen: 0,
        rules: riddle_core::probes::default_companion_rules(&tags, 2),
        max_rows: 3,
        hp: def.hp,
        max_hp: def.hp,
    };
    vec![mk(800_001, "Thix"), mk(800_002, "Skog")]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(12);
    let runs: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(16);
    let wanted: Vec<String> = args.iter().skip(3).cloned().collect();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            if !wanted.is_empty() && !wanted.iter().any(|w| stem.contains(w.as_str())) {
                return None;
            }
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .collect();
    sets.sort_by(|a, b| a.0.cmp(&b.0));
    for (name, set) in &sets {
        let (mut n, mut thefts, mut wakes, mut max_thefts, mut pet_n, mut pet_deaths, mut pet_runs_lost) = (0u32, 0u32, 0u32, 0usize, 0u32, 0u32, 0u32);
        let (mut depth_sum, mut deaths) = (0u32, 0u32);
        for seed in 1..=seeds {
            // §1: thefts per run on the set as played.
            let mut g = lineage_for(set, seed);
            for _ in 0..runs {
                g.lineage.rest_left = 0;
                g.start_run(None);
                g.run_to_end(riddle_core::engine::MAX_TURNS_PER_RUN);
                let r = g.run.as_ref().unwrap();
                max_thefts = max_thefts.max(r.stolen.len());
                depth_sum += r.depth;
                deaths += (r.over == Some(riddle_core::engine::ExitTier::Death)) as u32;
                g.finish_run();
                g.auto_keep();
                g.events.clear();
                n += 1;
            }
            thefts += g.batch.thefts;
            wakes += g.batch.den_wakes;
            // §2: two tamed jackals at every send.
            let mut g = lineage_for(set, seed ^ 0x7E7);
            for _ in 0..runs {
                g.lineage.rest_left = 0;
                g.lineage.party = jackals();
                g.start_run(None);
                g.run_to_end(riddle_core::engine::MAX_TURNS_PER_RUN);
                let lost = g.run.as_ref().unwrap().lost_companions.len() as u32;
                pet_deaths += lost;
                pet_runs_lost += (lost > 0) as u32;
                g.finish_run();
                g.auto_keep();
                g.events.clear();
                pet_n += 1;
            }
        }
        println!(
            "{name}: {n} runs · thefts/run {:.2} (max {max_thefts}) · den wakes/run {:.2} · mean depth {:.1} · deaths {:.0}% | pets: {pet_n} runs · pet deaths/run {:.2} · runs losing a pet {:.0}%",
            thefts as f64 / n as f64,
            wakes as f64 / n as f64,
            depth_sum as f64 / n as f64,
            100.0 * deaths as f64 / n as f64,
            pet_deaths as f64 / pet_n as f64,
            100.0 * pet_runs_lost as f64 / pet_n as f64
        );
    }
}
