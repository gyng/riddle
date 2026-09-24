//! A determinism fingerprint for performance-only engine changes (docs/ITERATION_SPEED.md):
//! per seed, a DEFAULT lineage's night and a finished lineage's (every unlock and fact, the
//! FULL set — the deep floors, the chores, the stall sims), then three full `death()`s and
//! the forecast; everything the player can see is serialised and hashed (FNV-1a). Same
//! printout before and after ⇒ the change is bit-identical on these paths. ~10 s on 32 threads
//! (the metrics table's replay hash is the gate; this is the inner-loop check).
//!   cargo run -q --profile fast -p riddle-core --example fingerprint [-- --seeds 8 --hours 8]
use riddle_core::Game;

fn fnv(h: &mut u64, s: &str) {
    for b in s.bytes() {
        *h ^= b as u64;
        *h = h.wrapping_mul(0x100000001b3);
    }
}

fn full(seed: u64) -> Game {
    let mut g = Game::new(seed);
    for u in riddle_core::meta::UNLOCKS {
        g.lineage.unlocks.insert(u.id.into());
    }
    riddle_core::probes::learn_everything(&mut g);
    let fighter = riddle_core::hero::Class::Fighter;
    g.lineage.classes.insert(fighter.name().into(), riddle_core::wire::ClassProg { level: 10, xp: 0, next: 0 });
    g.lineage.unlocks.insert(riddle_core::hero::mastery_card(fighter).into());
    g.set_rules(riddle_core::probes::full()).expect("full rules");
    g
}

fn night(mut g: Game, hours: u64) -> (u64, u32) {
    let mut h = 0xcbf29ce484222325u64;
    let rep = g.run_offline(hours * 3600);
    fnv(&mut h, &serde_json::to_string(&rep).unwrap());
    fnv(&mut h, &serde_json::to_string(&g.lineage).unwrap());
    let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.stall).map(|(id, _)| *id).collect();
    let step = (ids.len() / 3).max(1);
    for id in ids.iter().step_by(step).take(3) {
        if let Some(d) = g.death(*id) {
            fnv(&mut h, &serde_json::to_string(&d).unwrap());
        }
    }
    fnv(&mut h, &serde_json::to_string(&g.forecast()).unwrap());
    (h, g.batch.turns)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let (seeds, hours) = (get("--seeds", 8), get("--hours", 8));
    riddle_core::forecast::set_parallel_sims(false);
    let t = std::time::Instant::now();
    let out: Vec<(u64, &str, u64, u32)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=seeds)
            .flat_map(|s| [(s, "default"), (s, "full")])
            .map(|(s, which)| sc.spawn(move || {
                let g = if which == "full" { full(s) } else { Game::new(s) };
                let (h, turns) = night(g, hours);
                (s, which, h, turns)
            }))
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut all = 0xcbf29ce484222325u64;
    for (s, which, h, turns) in &out {
        println!("seed {s:>2} {which:<7} turns {turns:>7} hash {h:016x}");
        fnv(&mut all, &format!("{h:016x}"));
    }
    println!("fingerprint {all:016x} ({seeds} seeds × {hours} h, {:.1}s)", t.elapsed().as_secs_f64());
}
