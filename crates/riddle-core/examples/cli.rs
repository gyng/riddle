//! Text playthrough: run a rule set for N runs, print chronicle, exits, the death trace,
//! verdict, patches and the forecast.
//!   cargo run --release --example cli -- --seed 1 --rules presets/good.json --runs 3
use riddle_core::engine::ExitTier;
use riddle_core::{Ev, Game, RuleSet};

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1).cloned())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seed: u64 = arg(&args, "--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let runs: u32 = arg(&args, "--runs").and_then(|s| s.parse().ok()).unwrap_or(3);
    let verbose = args.iter().any(|a| a == "--verbose");
    let mut game = Game::new(seed);
    if let Some(path) = arg(&args, "--rules") {
        let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let set = RuleSet::parse(&text).unwrap_or_else(|e| panic!("{path}: {e}"));
        // Playtest tool: grant the rows and the common facts the set needs.
        for (i, u) in ["row5", "row6", "row7", "row8"].iter().enumerate() {
            if set.rows.len() > 4 + i {
                game.lineage.unlocks.insert(u.to_string());
            }
        }
        for r in &set.rows {
            for c in &r.conds {
                if c.k == "foe_tag" {
                    if let Some(t) = &c.t {
                        for m in riddle_core::defs::MONSTERS.iter().filter(|m| m.tags.contains(&t.as_str())) {
                            game.lineage.facts.insert(format!("foe:{}:{}", m.kind, t));
                        }
                        if t == "summoned" {
                            game.lineage.facts.insert("foe:skeleton:summoned".into());
                            game.lineage.facts.insert("foe:goblin:summoned".into());
                        }
                    }
                }
            }
            if r.verb.v == "throw" {
                game.lineage.unlocks.insert("throw".into());
            }
            if r.verb.v == "tame" {
                game.lineage.unlocks.insert("tame".into());
            }
            if let Some(a) = &r.verb.a {
                if let Some(f) = riddle_core::item::ident_fact(&game.lineage.flavours, a) {
                    game.lineage.facts.insert(f);
                }
            }
        }
        game.set_rules(set).expect("rules");
    }
    if arg(&args, "--class").as_deref() == Some("rogue") {
        game.lineage.unlocks.insert("rogue".into());
        game.set_class("rogue").unwrap();
        game.set_rules(riddle_core::probes::preset(riddle_core::hero::Class::Rogue)).unwrap();
    }
    println!("seed {seed} · trait {} · class {}", game.lineage.trait_.name(), game.lineage.class.name());
    for (i, r) in game.lineage.rules().rows.iter().enumerate() {
        println!("  R{} {}", i + 1, r.describe());
    }
    let mut last_death: Option<u32> = None;
    let mut deaths: Vec<u32> = Vec::new();
    for _ in 0..runs {
        game.send();
        let mut events: Vec<Ev> = Vec::new();
        let mut last_hero: Option<riddle_core::HeroSnap>;
        loop {
            let r = game.step(50);
            events.extend(r.events);
            last_hero = Some(r.snapshot.hero.clone());
            if r.run_over {
                if let Some(p) = &r.exit_pending {
                    println!("  exit {}: {} items offered", p.tier, p.items.len());
                }
                game.auto_keep();
                break;
            }
        }
        let run_id = events.iter().find_map(|e| if let Ev::Exit { .. } = e { Some(()) } else { None });
        let _ = run_id;
        println!("\n== run {} (heir {})", game.lineage.next_run_id - 1, game.lineage.heir);
        for e in &events {
            match e {
                Ev::Note { t, text } => println!("  t{t:<5} {text}"),
                Ev::Descend { t, depth, biome } => println!("  t{t:<5} ↓ D{depth} {biome}"),
                Ev::Exit { t, tier, loot_kept, line, .. } => println!("  t{t:<5} exit {tier} · loot {loot_kept}{}", line.as_ref().map(|l| format!(" · {}", l.text)).unwrap_or_default()),
                Ev::Rule { t, row, text, .. } if verbose => {
                    let label = match row {
                        -1 => "trait".to_string(),
                        -2 => "chore".to_string(),
                        r => format!("R{}", r + 1),
                    };
                    println!("  t{t:<5} {label:<5} {text}")
                }
                Ev::Callout { t, text } if verbose => println!("  t{t:<5} ! {text}"),
                Ev::Die { t, id, cause } if verbose && *id != 1 => println!("  t{t:<5} † #{id} by {cause}"),
                Ev::Hurt { t, id, dmg, hp, cause } if verbose && *id != 1 && *hp > 0 => println!("  t{t:<5} · #{id} -{dmg} → {hp} by {cause}"),
                _ => {}
            }
        }
        if let Some(h) = &last_hero {
            let inv: Vec<String> = h.inv.iter().map(|i| i.label.clone()).collect();
            println!("  hero: {} · hp {}/{} · {} · {} · inv [{}]", h.trait_, h.entity.hp, h.entity.max_hp, h.weapon.clone().unwrap_or_default(), h.armour.clone().unwrap_or("no armour".into()), inv.join(", "));
        }
        let renderable = events.iter().filter(|e| e.renderable()).count();
        let ticks = events.last().map(|e| e.t()).unwrap_or(1).max(1);
        println!("  events {renderable} renderable over {ticks} ticks ({:.1}/600)", renderable as f64 * 600.0 / ticks as f64);
        let died = events.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "death"));
        if died {
            last_death = Some(game.lineage.next_run_id - 1);
            deaths.push(game.lineage.next_run_id - 1);
        }
        let l = game.lineage();
        println!("  lineage: best D{} · marks {} · gold {} · facts {} · vault {} · renown {} (rank {})", l.best_depth, l.marks, l.gold, l.facts.len(), l.vault.len(), l.renown, l.rank);
    }
    let all = args.iter().any(|a| a == "--all-deaths");
    let to_show: Vec<u32> = if all { deaths } else { last_death.into_iter().collect() };
    for id in to_show {
        if let Some(d) = game.death(id) {
            println!("\n== death run {} · D{} · {} · {} · verdict {}", d.run_id, d.depth, d.cause, d.margin, d.verdict);
            println!("  trace (last {}):", d.trace.turns.len());
            for t in &d.trace.turns {
                let row = match t.row {
                    -1 => "trait".to_string(),
                    -2 => "chore".to_string(),
                    r => format!("R{}", r + 1),
                };
                // Cut 6 §3: the rows above the one that acted, with their reasons.
                let rows = t.rows.as_ref().map(|r| r.iter().map(|w| format!("R{} {}", w.row + 1, w.why)).collect::<Vec<_>>().join(" · ")).unwrap_or_default();
                println!("    t{:<5} {:<6} {:<14} hp {:<3} foes {} {} {}", t.t, row, t.verb.short(), t.hp, t.foes, t.telegraphs.join(", "), rows);
            }
            if let Some(l) = &d.line {
                println!("  {}", l.text);
            }
            for p in &d.patches {
                println!("  patch @{}: {}  survive {:.0}%  forecast Δ {:+.0}%", p.insert_at, p.row.describe(), p.survive * 100.0, p.forecast_delta * 100.0);
            }
            if verbose {
                println!("{}", d.morgue);
            }
        }
    }
    let f = game.forecast();
    let depths: Vec<String> = f.depths.iter().map(|d| format!("D{} {:.0}%", d.depth, d.reach * 100.0)).collect();
    let causes: Vec<String> = f.causes.iter().map(|c| format!("{} {:.0}%", c.cause, c.share * 100.0)).collect();
    println!("\n== forecast (known to D{}): {} · deaths: {} · D{}+ ?", f.known_to, depths.join(" · "), causes.join(", "), f.known_to + 1);
    let _ = ExitTier::Bank;
}
