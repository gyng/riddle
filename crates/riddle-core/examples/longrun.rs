//! Find runs longer than N ticks and print what the hero was doing.
use riddle_core::{Ev, Game};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let limit: u32 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(20000);
    let good = args.get(2).map(|p| riddle_core::RuleSet::parse(&std::fs::read_to_string(p).unwrap()).unwrap());
    for seed in 1..=30u64 {
        let mut g = Game::new(seed);
        if let Some(set) = &good {
            for u in ["row5", "row6", "row7", "row8"] {
                g.lineage.unlocks.insert(u.into());
            }
            for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged"] {
                g.lineage.facts.insert(f.into());
            }
            g.set_rules(set.clone()).unwrap();
        }
        g.send();
        let mut events: Vec<Ev> = Vec::new();
        let mut runs = 0;
        let mut kept: Option<riddle_core::engine::Run> = None;
        while runs < 60 {
            let r = g.step(100);
            if let Some(run) = g.run.as_ref() {
                if run.turn > limit {
                    kept = Some(run.clone());
                }
            }
            events.extend(r.events);
            if r.run_over {
                runs += 1;
                let ticks = r.snapshot.turn;
                if ticks > limit {
                    println!("seed {seed} run {} depth {} ticks {ticks} trait {}", runs, r.snapshot.depth, r.snapshot.hero.trait_);
                    let rules: Vec<&Ev> = events.iter().filter(|e| matches!(e, Ev::Rule { .. })).collect();
                    let n = rules.len();
                    let mut counts = std::collections::BTreeMap::new();
                    for e in &rules[n.saturating_sub(400)..] {
                        if let Ev::Rule { text, .. } = e {
                            *counts.entry(text.clone()).or_insert(0) += 1;
                        }
                    }
                    println!("  last 400 actions: {counts:?}");
                    for e in events.iter().rev().take(60).collect::<Vec<_>>().into_iter().rev() {
                        if !matches!(e, Ev::Rule { .. } | Ev::Move { id: 1, .. }) {
                            continue;
                        }
                        println!("    {}", serde_json::to_string(e).unwrap());
                    }
                    // Map dump around the hero.
                    let snap = &r.snapshot;
                    let hx = snap.hero.entity.x;
                    let hy = snap.hero.entity.y;
                    for y in (hy - 6).max(0)..(hy + 7).min(snap.h) {
                        let mut line = String::new();
                        for x in (hx - 10).max(0)..(hx + 11).min(snap.w) {
                            let i = (y * snap.w + x) as usize;
                            let c = if x == hx && y == hy {
                                '@'
                            } else if let Some(e) = snap.entities.iter().find(|e| e.x == x && e.y == y) {
                                e.kind.chars().next().unwrap().to_ascii_uppercase()
                            } else if !snap.seen[i] {
                                '?'
                            } else {
                                match snap.tiles[i] {
                                    riddle_core::tiles::Tile::Wall => '#',
                                    riddle_core::tiles::Tile::Floor => '.',
                                    riddle_core::tiles::Tile::Door => '+',
                                    riddle_core::tiles::Tile::Water => '~',
                                    riddle_core::tiles::Tile::Chasm => 'v',
                                    riddle_core::tiles::Tile::StairsDown => '>',
                                    riddle_core::tiles::Tile::StairsUp => '<',
                                }
                            };
                            line.push(c);
                        }
                        println!("  {line}");
                    }
                    println!("  entities: {:?}", snap.entities.iter().map(|e| format!("{}@{},{} {:?}", e.kind, e.x, e.y, e.tags)).collect::<Vec<_>>());
                    if let Some(run) = kept.as_ref() {
                        println!("  frontier from {:?}: {:?}", run.hero.pos, riddle_core::ai::frontier_target(run));
                        println!("  overlays {:?} tick {}", run.overlays, run.turn);
                        let mut alt = run.clone();
                        alt.hero.pos = riddle_core::geom::Pos::new(run.hero.pos.x - 1, run.hero.pos.y);
                        println!("  frontier from {:?}: {:?}", alt.hero.pos, riddle_core::ai::frontier_target(&alt));
                        let goal = riddle_core::ai::frontier_target(&alt).map(|(g, _)| g);
                        if let Some(gl) = goal {
                            let m = &run.floor.map;
                            let around: Vec<String> = gl.neighbours8().iter().map(|q| format!("{:?}:{}{}", q, if m.is_seen(*q) { "s" } else { "u" }, if m.passable(*q) { "p" } else { "w" })).collect();
                            println!("  goal {:?} seen={} passable={} occupied={:?} around {:?}", gl, m.is_seen(gl), m.passable(gl), run.monster_at(gl).map(|i| run.monsters[i].kind.clone()), around);
                        }
                    }
                    if let Some(run) = kept.as_ref() {
                        let mut sim = g.sim_clone();
                        sim.run = Some(run.clone());
                        for _ in 0..40 {
                            let r = sim.run.as_ref().unwrap();
                            if r.turn.is_multiple_of(10) {
                                let items: Vec<String> = r.items.iter().map(|i| format!("{}@{:?}", i.item.kind, i.pos)).collect();
                                println!("  t{} hero {:?} frontier {:?} items_ignored {} items {:?} stairs {:?}", r.turn, r.hero.pos, riddle_core::ai::frontier_target(r), r.items_ignored(), items, r.floor.stairs_down);
                                let v = riddle_core::turn::view(r);
                                let foes: Vec<String> = v.foes.iter().map(|&i| { let m = &r.monsters[i]; format!("{}@{:?} flee={} fear={} stun={} par={} ign={} awake={} energy={} conf={} blind={} stolen={} hero_invis={} vanish={}", m.kind, m.pos, m.fleeing, m.fear, m.stun, m.paralysed, r.is_ignored(m.id), m.awake, m.energy, m.confused, m.blind, m.stolen.is_some(), r.hero.invis_t, r.hero.vanish_t) }).collect();
                                println!("     foes {:?} stuck_until {} actions {} hp {}/{}", foes, r.stuck_until, r.actions, r.hero.hp, r.hero.max_hp);
                            }
                            sim.tick();
                            let evs: Vec<String> = sim.events.iter().filter(|e| matches!(e, Ev::Rule { .. } | Ev::Move { id: 1, .. })).map(|e| serde_json::to_string(e).unwrap()).collect();
                            if !evs.is_empty() {
                                println!("     {}", evs.join(" "));
                            }
                            sim.events.clear();
                        }
                    }
                    println!("  seen {}%", snap.seen.iter().zip(snap.tiles.iter()).filter(|(s, t)| **s && t.passable()).count() * 100 / snap.tiles.iter().filter(|t| t.passable()).count());
                    return;
                }
                events.clear();
                g.send();
            }
        }
    }
}
