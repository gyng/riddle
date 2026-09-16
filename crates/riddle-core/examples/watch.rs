//! Watch one FULL-bot run from a given depth: prints the rule, hurt, callout, use, descend and
//! exit events tick by tick, and the quiet per-tick cost of the whole run.
//!   cargo run --profile fast --example watch -- [seed] [from_depth] [presets/full.json]
use riddle_core::hero::Class;
use riddle_core::{Ev, Game, RuleSet};
use std::time::Instant;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let from: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(16);
    let path = args.iter().skip(3).find(|a| !a.starts_with("--")).cloned().unwrap_or("crates/riddle-core/presets/full.json".into());
    let quiet = args.iter().any(|a| a == "--quiet");
    let survey = args.iter().any(|a| a == "--survey");
    let set = RuleSet::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut g = Game::new(seed);
    g.sim = true;
    for u in riddle_core::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    riddle_core::probes::learn_everything(&mut g);
    g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0 });
    g.lineage.unlocks.insert(riddle_core::hero::mastery_card(Class::Fighter).into());
    g.set_rules(set).unwrap();
    // Runs until one reaches `from`, then watch it. `--survey`: every run's last floor summarised.
    let attempts = if survey { from as usize } else { 40 };
    for attempt in 0..attempts {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let t = Instant::now();
        let mut n = 0u64;
        let mut watching = false;
        let mut dmg: std::collections::BTreeMap<String, i32> = std::collections::BTreeMap::new();
        let mut rows: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut uses: Vec<String> = Vec::new();
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) {
            g.tick();
            n += 1;
            if survey {
                for e in &g.events {
                    match e {
                        Ev::Descend { .. } => {
                            dmg.clear();
                            rows.clear();
                            uses.clear();
                        }
                        Ev::Hurt { id: 1, dmg: d, cause, .. } => *dmg.entry(cause.clone()).or_insert(0) += d,
                        Ev::Rule { row, verb, .. } => *rows.entry(format!("R{row} {}{}", verb.v, verb.a.as_ref().map(|a| format!(" {a}")).unwrap_or_default())).or_insert(0) += 1,
                        Ev::Use { item, .. } => uses.push(item.clone()),
                        _ => {}
                    }
                }
                g.events.clear();
                continue;
            }
            let r = g.run.as_ref().unwrap();
            if r.depth >= from && !watching {
                watching = true;
                println!("--- attempt {attempt}: reached D{} at tick {} (hero {}/{} hp, {:?}/{:?}, inv {:?})", r.depth, r.turn, r.hero.hp, r.hero.max_hp, r.hero.weapon.as_ref().map(|w| &w.kind), r.hero.armour.as_ref().map(|a| &a.kind), r.hero.inv.iter().map(|i| i.kind.as_str()).collect::<Vec<_>>());
            }
            if watching && !quiet {
                for e in &g.events {
                    match e {
                        Ev::Rule { t, row, verb, text } => {
                            println!("{t:>6} R{row} {}{} · {text}", verb.v, verb.a.as_ref().map(|a| format!(" {a}")).unwrap_or_default());
                            if args.iter().any(|a| a == "--items") && verb.v == "pick_up" {
                                let hp = r.hero.pos;
                                let near: Vec<String> = r.items.iter().filter(|i| i.pos.cheb(hp) <= 1).map(|i| format!("{}@({},{})", i.item.kind, i.pos.x, i.pos.y)).collect();
                                println!("        inv {:?} near {:?}", r.hero.inv.iter().map(|i| i.kind.as_str()).collect::<Vec<_>>(), near);
                            }
                        }
                        Ev::Hurt { t, id, dmg, hp, cause } => println!("{t:>6}   hurt #{id} -{dmg} → {hp} ({cause})"),
                        Ev::Callout { t, text } => println!("{t:>6}   [{text}]"),
                        Ev::Use { t, item, outcome } => println!("{t:>6}   use {item}: {outcome}"),
                        Ev::Descend { t, depth, biome } => println!("{t:>6} === D{depth} {biome}"),
                        Ev::Exit { t, tier, loot_kept, .. } => println!("{t:>6} === exit {tier} loot {loot_kept}"),
                        Ev::Spawn { t, e } => println!("{t:>6}   spawn {} #{}", e.kind, e.id),
                        Ev::Telegraph { t, id, what } => println!("{t:>6}   #{id} telegraphs {what}"),
                        Ev::Move { t, id, x, y } if *id == 1 || args.iter().any(|a| a == "--moves") => println!("{t:>6}   move #{id} → ({x},{y})"),
                        Ev::Attack { t, src, dst, dmg, hit, verb } => println!("{t:>6}   #{src} → #{dst} {} {dmg}{}", verb.as_deref().unwrap_or("?"), if *hit { "" } else { " miss" }),
                        _ => {}
                    }
                }
            }
            g.events.clear();
        }
        let dt = t.elapsed();
        let r = g.run.as_ref().unwrap();
        if survey {
            let mut dv: Vec<(&String, &i32)> = dmg.iter().collect();
            dv.sort_by(|a, b| b.1.cmp(a.1));
            let mut rv: Vec<(&String, &u32)> = rows.iter().collect();
            rv.sort_by(|a, b| b.1.cmp(a.1));
            println!(
                "run {attempt:>2}: D{:<2} {:<7} {:<16} hp {:>2}/{:<2} ticks {:>6} · dmg {} · rows {} · used {}",
                r.max_depth,
                format!("{:?}", r.over.unwrap()),
                r.death_cause.clone().unwrap_or_default(),
                r.hero.hp,
                r.hero.max_hp,
                r.turn,
                dv.iter().take(4).map(|(c, d)| format!("{c} {d}")).collect::<Vec<_>>().join(", "),
                rv.iter().take(5).map(|(c, d)| format!("{c}×{d}")).collect::<Vec<_>>().join(", "),
                uses.join(",")
            );
            g.finish_run();
            g.auto_keep();
            continue;
        }
        println!("attempt {attempt}: D{} {:?} cause {:?} ticks {} ({:.2} µs/tick)", r.max_depth, r.over, r.death_cause, r.turn, dt.as_secs_f64() * 1e6 / n.max(1) as f64);
        g.finish_run();
        g.auto_keep();
        if watching {
            break;
        }
    }
}
