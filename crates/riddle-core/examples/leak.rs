//! Find runs of a rule set that pass a given depth and print the boss floor's rule/callout tail.
use riddle_core::{Ev, Game, RuleSet};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap();
    let pass: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(5);
    let set = RuleSet::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for seed in 1..=40u64 {
        let mut g = Game::new_literal(seed);
        for u in ["row5", "row6", "row7", "row8", "tame", "throw"] {
            g.lineage.unlocks.insert(u.into());
        }
        g.lineage.facts.insert("item:leash".into());
        for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss"] {
            g.lineage.facts.insert(f.into());
        }
        for k in ["heal", "poison", "fire", "teleport", "blink"] {
            if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
                g.lineage.facts.insert(f);
            }
        }
        g.set_rules_raw(set.clone()).unwrap();
        g.send();
        let mut events: Vec<Ev> = Vec::new();
        let mut runs = 0;
        while runs < 30 {
            let r = g.step(100);
            events.extend(r.events);
            if r.run_over {
                runs += 1;
                let died_at = events.iter().any(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass)) && !events.iter().any(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass + 1)) && std::env::var("LEAK_DEATH").is_ok();
                if died_at {
                    println!("seed {seed} run {runs} died on D{pass}");
                    let start = events.iter().position(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass)).unwrap_or(0);
                    for e in &events[start..] {
                        match e {
                            Ev::Rule { t, row, text, .. } if *row != -2 => println!("  t{t} R{row} {text}"),
                            Ev::Callout { t, text, .. } => println!("  t{t} ! {text}"),
                            Ev::Note { t, text } => println!("  t{t} {text}"),
                            Ev::Hurt { t, id, dmg, hp, cause } => println!("  t{t} #{id} -{dmg} → {hp} ({cause})"),
                            Ev::Use { t, item, outcome } => println!("  t{t} use {item}: {outcome}"),
                            Ev::Pickup { t, item, .. } => println!("  t{t} + {item}"),
                            _ => {}
                        }
                    }
                    return;
                }
                if events.iter().any(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass + 1)) && std::env::var("LEAK_DEATH").is_err() {
                    println!("seed {seed} run {runs} passed D{pass}");
                    let start = events.iter().position(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass)).unwrap_or(0);
                    let end = events.iter().position(|e| matches!(e, Ev::Descend { depth, .. } if *depth == pass + 1)).unwrap();
                    for e in &events[start..=end] {
                        match e {
                            Ev::Rule { t, row, text, .. } if *row != -2 || text.contains("stuck") => println!("  t{t} R{row} {text}"),
                            Ev::Callout { t, text, .. } => println!("  t{t} ! {text}"),
                            Ev::Note { t, text } => println!("  t{t} {text}"),
                            Ev::Die { t, id, cause } => println!("  t{t} † #{id} {cause}"),
                            Ev::Hurt { t, id, dmg, hp, cause } if *id != 1 => println!("  t{t} #{id} -{dmg} → {hp} ({cause})"),
                            Ev::Descend { t, depth, .. } => println!("  t{t} ↓ D{depth}"),
                            Ev::Use { t, item, outcome } => println!("  t{t} use {item}: {outcome}"),
                            _ => {}
                        }
                    }
                    return;
                }
                events.clear();
                g.send();
            }
        }
    }
    println!("no leak in 40 seeds × 30 runs");
}
