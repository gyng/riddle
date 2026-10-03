//! Cut 23 tuning probe: FULL−D28 per seed (best depth over 3 × 8 h), with thefts and pets.
use riddle_core::hero::Class;
use riddle_core::{Game, RuleSet};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(30);
    let drop_verb = args.get(2).cloned().unwrap_or_else(|| "read".into());
    let drop_arg = args.get(3).cloned().unwrap_or_else(|| "silence".into());
    let wall: u32 = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(28);
    riddle_core::forecast::set_parallel_sims(false);
    let out: Vec<(u64, u32, u32, u32, u32)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=seeds)
            .map(|seed| {
                let (dv, da) = (drop_verb.clone(), drop_arg.clone());
                sc.spawn(move || {
                    let mut g = Game::new_literal(seed);
                    g.max_deaths = 100_000;
                    for u in riddle_core::meta::UNLOCKS {
                        g.lineage.unlocks.insert(u.id.into());
                    }
                    riddle_core::probes::learn_everything(&mut g);
                    g.lineage.classes.insert(Class::Fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0, next: 0 });
                    g.lineage.unlocks.insert(riddle_core::hero::mastery_card(Class::Fighter).into());
                    if std::env::var("KIT").is_ok() {
                        riddle_core::kit::buy_all(&mut g.lineage);
                        if let Ok(lv) = std::env::var("KIT_LEVELS") {
                            for kv in lv.split(',') {
                                let (k, v) = kv.split_once('=').unwrap();
                                g.lineage.kit.insert(k.into(), v.parse().unwrap());
                            }
                        }
                        if let Ok(only) = std::env::var("KIT_ONLY") {
                            g.lineage.kit.retain(|k, _| only.split(',').any(|o| o == k));
                        }
                    }
                    let mut set: RuleSet = riddle_core::probes::full();
                    let i = set.rows.iter().position(|r| r.verb.v == dv && r.verb.a.as_deref() == Some(da.as_str())).unwrap();
                    set.rows.remove(i);
                    g.set_rules(set).unwrap();
                    let (mut leash, mut thefts, mut past) = (0u32, 0u32, 0u32);
                    for _ in 0..3 {
                        riddle_core::offline::run_offline_quick(&mut g, 8 * 3600);
                        past += g.batch.run_outcomes.iter().filter(|(d, _)| *d > wall).count() as u32;
                        for (k, n) in &g.batch.stolen {
                            thefts += n;
                            if k == "leash" {
                                leash += n;
                            }
                        }
                    }
                    (seed, g.lineage.best_depth, leash, thefts, past)
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let held = out.iter().filter(|o| o.1 <= wall).count();
    for o in &out {
        println!("seed {:>2} best D{:<2} leash stolen {} · thefts {} · runs past {}", o.0, o.1, o.2, o.3, o.4);
    }
    println!("held {held}/{seeds}");
}
