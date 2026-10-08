//! Cut 113 §2 diagnostic: the forge's two branches a tier (weapon aim · edge, armour plate · pace),
//! paired at IDLE's walls (`idle_lib::snapshots`: the lineage the first time it stands under each wall),
//! the record at the wall, sends from the deepest lit stone at or above it. For each ladder and each
//! tier count k, the lineage owns k tiers on the ladder's defaults below the last, and the last tier takes
//! the first branch or the second (the per-tier choice); and k tiers all one way (the whole lean). The
//! value: the share of sends past the wall, less a quarter of the deaths (`DW=`), the same seeds both ways.
//! Diagnostic, never the gate.
//!   cargo run -q --profile fast -p riddle-core --example forge_branch -- [SEEDS=8] [SIMS=96]
use riddle_core::engine::ExitTier;
use riddle_core::Game;
#[path = "idle_lib/mod.rs"]
#[allow(dead_code)]
mod idle;
#[path = "jobcache_lib/mod.rs"]
mod jobcache;

fn snaps(seed: u64) -> Vec<(u32, String)> {
    jobcache::cached("idle-snaps", &[include_str!("idle_lib/mod.rs")], &format!("snapshots {seed} 14"), || idle::snapshots(seed, 14).into_iter().map(|(w, g)| (w, g.save())).collect())
}

fn value(g: &Game, wall: u32, slot: &str, k: u32, alt_mask: u32, sims: u32) -> f64 {
    let mut c = g.sim_clone();
    c.lineage.start = 1;
    if let Some(s) = c.lineage.stones().into_iter().filter(|s| *s <= wall).max() {
        c.lineage.start = s;
    }
    c.lineage.best_depth = c.lineage.best_depth.max(wall);
    c.lineage.kit.insert(slot.into(), k);
    if alt_mask != 0 {
        c.lineage.kit_alt.insert(slot.into(), alt_mask);
    } else {
        c.lineage.kit_alt.remove(slot);
    }
    let set = c.lineage.rules().clone();
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    let n = rs.len().max(1) as f64;
    let past = rs.iter().filter(|r| r.max_depth > wall).count() as f64 / n;
    let died = rs.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / n;
    past - std::env::var("DW").ok().and_then(|v| v.parse().ok()).unwrap_or(0.25) * died
}

fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8);
    let sims: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(96);
    let games: Vec<(u64, u32, String)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=seeds).map(|s| sc.spawn(move || snaps(s).into_iter().map(|(w, save)| (s, w, save)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    // jobs: (slot, k, mode 0 = last tier, 1 = whole lean), per (seed, wall): (A, B)
    let mut jobs = Vec::new();
    for slot in ["weapon", "armour"] {
        let tiers = riddle_core::kit::mults(slot).len() as u32;
        for k in 1..=tiers {
            for mode in 0..2u32 {
                for (i, _) in games.iter().enumerate() {
                    jobs.push((slot, k, mode, i));
                }
            }
        }
    }
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&(slot, k, mode, i)) = jobs.get(j) else { break };
                let (_, wall, save) = &games[i];
                let g = &Game::load(save).unwrap();
                // the default branches below tier k − 1; the last tier flipped (mode 0) or every tier
                // on the second branch (mode 1: B) against every tier on the first (A)
                let first_all: u32 = (0..k).filter(|&t| riddle_core::kit::default_branch(slot, t as usize) != riddle_core::kit::branches(slot).map(|b| b[0])).fold(0, |m, t| m | 1 << t);
                let second_all: u32 = (0..k).filter(|&t| riddle_core::kit::default_branch(slot, t as usize) != riddle_core::kit::branches(slot).map(|b| b[1])).fold(0, |m, t| m | 1 << t);
                let (ma, mb) = if mode == 0 {
                    let last = k - 1;
                    let flip = 1u32 << last;
                    let d_is_first = riddle_core::kit::default_branch(slot, last as usize) == riddle_core::kit::branches(slot).map(|b| b[0]);
                    if d_is_first { (0, flip) } else { (flip, 0) }
                } else {
                    (first_all, second_all)
                };
                let a = value(g, *wall, slot, k, ma, sims);
                let b = value(g, *wall, slot, k, mb, sims);
                out.lock().unwrap().push((slot, k, mode, *wall, b - a));
            });
        }
    });
    let out = out.into_inner().unwrap();
    for slot in ["weapon", "armour"] {
        let names = riddle_core::kit::branches(slot).unwrap();
        for mode in 0..2u32 {
            println!("{slot} · {} · Δ = {} − {} (median over seeds; + favours {})", if mode == 0 { "last tier" } else { "whole lean" }, names[1], names[0], names[1]);
            let tiers = riddle_core::kit::mults(slot).len() as u32;
            for k in 1..=tiers {
                let mut line = format!("  k={k}:");
                for w in idle::WALLS {
                    let mut v: Vec<f64> = out.iter().filter(|x| x.0 == slot && x.1 == k && x.2 == mode && x.3 == w).map(|x| x.4).collect();
                    v.sort_by(f64::total_cmp);
                    let m = if v.is_empty() { f64::NAN } else { v[v.len() / 2] };
                    line += &format!("  D{w} {:+.3} ({}+/{}−)", m, v.iter().filter(|x| **x > 0.0).count(), v.iter().filter(|x| **x < 0.0).count());
                }
                println!("{line}");
            }
        }
    }
}
