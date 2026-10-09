//! Cut 115 §1–2 probe (docs/CUT115_BUILDS_FROM_TACTICS.md): at IDLE's walls (`idle_lib::snapshots`: the lineage the first
//! time it stands under each wall; sends from the deepest lit stone at or above it, the record at the wall, drills
//! revoked — a pick is weighed before the drill comes), on the paired panel (both sides play the same seeds):
//!
//! - **variants**: each tactic worn at L1 in its first variant and its second — past (sends past the wall), bank and
//!   death shares, mean over the seeds. Gate: the two differ by ≥ 10 points in one of past · bank · death at ≥ 1 wall,
//!   and neither dominates every wall (each is ahead on `past − DW·death` at some wall).
//! - **synergies**: each synergy's pair worn, its effect on and off (`packages::SYNERGIES_OFF`). Gate: it pays ≥ 3
//!   points of the value at ≥ 1 wall; and no synergy pays the most at every wall (the builds' totals print beside it).
//!
//! Diagnostic, never the gate table.
//!   cargo run -q --profile fast -p riddle-core --example builds -- [SEEDS=16] [SIMS=24] [variants|synergies|all]
use riddle_core::engine::ExitTier;
use riddle_core::packages;
use riddle_core::Game;
#[path = "idle_lib/mod.rs"]
#[allow(dead_code)]
mod idle;
#[path = "jobcache_lib/mod.rs"]
mod jobcache;

fn snaps(seed: u64) -> Vec<(u32, String)> {
    jobcache::cached("idle-snaps", &[include_str!("idle_lib/mod.rs")], &format!("snapshots {seed} 14"), || idle::snapshots(seed, 14).into_iter().map(|(w, g)| (w, g.save())).collect())
}

/// What a lineage wears for a job: tactic + variant, or a synergy's pair.
#[derive(Clone)]
enum Wear {
    Variant(&'static str, u8),
    Pair(&'static str, bool),
}

fn dw() -> f64 {
    std::env::var("DW").ok().and_then(|v| v.parse().ok()).unwrap_or(0.2)
}

/// (past, bank, death, reach, mean depth) of the sends from the wall's stone.
fn shares(g: &Game, wall: u32, wear: &Wear, sims: u32) -> [f64; 5] {
    let mut c = g.sim_clone();
    c.lineage.start = 1;
    // (from the deepest lit stone at or above the wall, as `idle_lib::stance_past` and the camp's `d_wall` read a wall;
    // `FROM=d1`: from D1, where the Warlord's floor gates every later wall)
    if let Some(s) = c.lineage.stones().into_iter().filter(|s| *s <= wall).max().filter(|_| std::env::var("FROM").map_or(true, |v| v != "d1")) {
        c.lineage.start = s;
    }
    c.lineage.best_depth = c.lineage.best_depth.max(wall);
    for d in c.lineage.pkg.drills.iter_mut() {
        d.revoked = true;
    }
    let p = &mut c.lineage.pkg;
    p.tactics.clear();
    p.variants.clear();
    let mut off = false;
    match wear {
        Wear::Variant("none", _) => {}
        Wear::Variant(t, v) => {
            p.owned.insert(t.to_string());
            p.tactics.push(t.to_string());
            if *v > 0 {
                p.variants.insert(t.to_string(), *v);
            }
        }
        Wear::Pair(id, on) => {
            off = !on;
            let s = packages::SYNERGIES.iter().find(|s| s.id == *id).unwrap();
            for pick in s.pair {
                let d = packages::def(pick).unwrap();
                p.owned.insert(pick.to_string());
                match d.kind {
                    packages::Kind::Stance => p.stance = pick.to_string(),
                    packages::Kind::Tactic => p.tactics.push(pick.to_string()),
                    packages::Kind::Temperament => {
                        p.temperament = Some(pick.to_string());
                        p.temperament_chosen = true;
                    }
                }
            }
        }
    }
    // (a worn tactic's card plays with it: `recompile` unlocks it, as `equip` does)
    packages::recompile(&mut c.lineage);
    let set = packages::compile(&c.lineage);
    packages::SYNERGIES_OFF.with(|x| x.set(off));
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    packages::SYNERGIES_OFF.with(|x| x.set(false));
    let k = rs.len().max(1) as f64;
    let past = rs.iter().filter(|r| r.max_depth > wall).count() as f64 / k;
    let bank = rs.iter().filter(|r| r.tier == ExitTier::Bank).count() as f64 / k;
    let death = rs.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / k;
    let reach = rs.iter().filter(|r| r.max_depth >= wall).count() as f64 / k;
    let mean = rs.iter().map(|r| r.max_depth as f64).sum::<f64>() / k;
    [past, bank, death, reach, mean]
}

fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let seeds: u64 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(16);
    let sims: u32 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(24);
    let mode = args.get(3).cloned().unwrap_or_else(|| "all".into());
    let games: Vec<(u64, u32, String)> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=seeds).map(|s| sc.spawn(move || snaps(s).into_iter().map(|(w, save)| (s, w, save)).collect::<Vec<_>>())).collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let mut wears: Vec<Wear> = Vec::new();
    if mode == "none" {
        wears.push(Wear::Variant("none", 0));
    }
    if mode != "synergies" && mode != "none" {
        for d in packages::PACKAGES.iter().filter(|d| packages::variants(d.id).is_some()) {
            wears.push(Wear::Variant(d.id, 0));
            wears.push(Wear::Variant(d.id, 1));
        }
    }
    if mode != "variants" && mode != "none" {
        for s in packages::SYNERGIES.iter() {
            wears.push(Wear::Pair(s.id, true));
            wears.push(Wear::Pair(s.id, false));
        }
    }
    let jobs: Vec<(usize, usize)> = (0..wears.len()).flat_map(|w| (0..games.len()).map(move |g| (w, g))).collect();
    let threads = std::thread::available_parallelism().map_or(8, |n| n.get());
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let j = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let Some(&(wi, gi)) = jobs.get(j) else { break };
                let (_, wall, save) = &games[gi];
                let g = Game::load(save).unwrap();
                let v = shares(&g, *wall, &wears[wi], sims);
                out.lock().unwrap().push((wi, *wall, v));
            });
        }
    });
    let out = out.into_inner().unwrap();
    let walls: Vec<u32> = idle::WALLS.iter().copied().filter(|w| games.iter().any(|g| g.1 == *w)).collect();
    let mean = |wi: usize, w: u32| -> [f64; 5] {
        let v: Vec<[f64; 5]> = out.iter().filter(|x| x.0 == wi && x.1 == w).map(|x| x.2).collect();
        let n = v.len().max(1) as f64;
        std::array::from_fn(|k| v.iter().map(|x| x[k]).sum::<f64>() / n)
    };
    // a pick's worth at a wall, as the camp's paired panel weighs a move (`packages::score`): the sends past it, those
    // reaching it and banking, a tenth a floor of mean depth, less those that die (`DW=` the death weight)
    let score = |m: [f64; 5]| m[0] + 0.3 * m[3] + 0.2 * m[1] - dw() * m[2] + 0.1 * m[4];
    let mut fails = 0;
    println!("walls: {}", walls.iter().map(|w| format!("D{w} ({} seeds)", games.iter().filter(|g| g.1 == *w).count())).collect::<Vec<_>>().join(" · "));
    if mode == "none" {
        for w in &walls {
            let m = mean(0, *w);
            println!("  none D{w}: {:.0}/{:.0}/{:.0} reach {:.0} mean {:.1}", m[0] * 100.0, m[1] * 100.0, m[2] * 100.0, m[3] * 100.0, m[4]);
        }
        return;
    }
    if mode != "synergies" {
        println!("\nvariants — Δ = second − first (points): past · bank · death; value = past + 0.3·reach + 0.2·bank − {}·death + 0.1·mean", dw());
        for d in packages::PACKAGES.iter().filter(|d| packages::variants(d.id).is_some()) {
            let names = packages::variants(d.id).unwrap();
            let a = wears.iter().position(|w| matches!(w, Wear::Variant(t, 0) if *t == d.id)).unwrap();
            let b = a + 1;
            let mut line = format!("{:<18} {} vs {}:", d.name, names[0], names[1]);
            // (dominance is Pareto: ahead on one of past · bank · death by more than `TOL=` points (2) at some wall —
            // a variant that only trades one term for another is a choice, not a mistake; `value` prints the camp score's read)
            let tol = std::env::var("TOL").ok().and_then(|v| v.parse().ok()).unwrap_or(2.0);
            let (mut differ, mut a_ahead, mut b_ahead) = (false, false, false);
            let mut value_line = String::new();
            for w in &walls {
                let (ma, mb) = (mean(a, *w), mean(b, *w));
                let dp = (mb[0] - ma[0]) * 100.0;
                let db = (mb[1] - ma[1]) * 100.0;
                let dd = (mb[2] - ma[2]) * 100.0;
                differ |= dp.abs() >= 10.0 || db.abs() >= 10.0 || dd.abs() >= 10.0;
                b_ahead |= dp > tol || db > tol || dd < -tol;
                a_ahead |= dp < -tol || db < -tol || dd > tol;
                value_line += &format!(" D{w} {:+.1}", (score(mb) - score(ma)) * 100.0);
                line += &format!("  D{w} {dp:+.0}·{db:+.0}·{dd:+.0}");
            }
            line += &format!("  | value Δ{value_line}");
            if std::env::var("VERBOSE").is_ok() {
                for w in &walls {
                    let (ma, mb) = (mean(a, *w), mean(b, *w));
                    println!("    D{w} first {:.0}/{:.0}/{:.0} reach {:.0} mean {:.1} · second {:.0}/{:.0}/{:.0} reach {:.0} mean {:.1}", ma[0] * 100.0, ma[1] * 100.0, ma[2] * 100.0, ma[3] * 100.0, ma[4], mb[0] * 100.0, mb[1] * 100.0, mb[2] * 100.0, mb[3] * 100.0, mb[4]);
                }
            }
            let ok = differ && a_ahead && b_ahead;
            fails += usize::from(!ok);
            println!("{line}  → {}{}", if ok { "PASS" } else { "FAIL" }, if !differ { " (<10 pts everywhere)" } else if !(a_ahead && b_ahead) { if a_ahead { " (first dominates)" } else { " (second dominates)" } } else { "" });
        }
    }
    if mode != "variants" {
        println!("\nsynergies — effect on − off (points of the value, death weight {}); the build's value per wall", dw());
        let mut best_at: Vec<(u32, &str, f64)> = Vec::new();
        let mut totals: Vec<(u32, &str)> = Vec::new();
        for s in packages::SYNERGIES.iter() {
            let on = wears.iter().position(|w| matches!(w, Wear::Pair(id, true) if *id == s.id)).unwrap();
            let off = on + 1;
            let mut line = format!("{:<11} ({} + {}):", s.name, s.pair[0], s.pair[1]);
            let mut pays = 0.0f64;
            for w in &walls {
                let d = (score(mean(on, *w)) - score(mean(off, *w))) * 100.0;
                pays = pays.max(d);
                line += &format!("  D{w} {d:+.1} ({:.0})", score(mean(on, *w)) * 100.0);
            }
            let ok = pays >= 3.0;
            fails += usize::from(!ok);
            println!("{line}  → {}", if ok { "PASS" } else { "FAIL (pays < 3 pts)" });
        }
        // (none dominates: no synergy pays the most at every wall — its effect's worth, on − off; the builds' totals
        // print beside it, led by their stance: Guarded is the strongest stance from the stones)
        let pay = |s: &packages::Synergy, w: u32| {
            let on = wears.iter().position(|x| matches!(x, Wear::Pair(id, true) if *id == s.id)).unwrap();
            (score(mean(on, w)) - score(mean(on + 1, w)), score(mean(on, w)))
        };
        for w in &walls {
            let top = packages::SYNERGIES.iter().map(|s| (s.name, pay(s, *w).0)).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
            let total = packages::SYNERGIES.iter().map(|s| (s.name, pay(s, *w).1)).max_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
            best_at.push((*w, top.0, top.1));
            totals.push((*w, total.0));
        }
        let first = best_at.first().map(|b| b.1);
        let dominated = best_at.len() > 1 && best_at.iter().all(|b| Some(b.1) == first);
        fails += usize::from(dominated);
        println!("pays most per wall: {}  → {}", best_at.iter().map(|b| format!("D{} {} {:+.1}", b.0, b.1, b.2 * 100.0)).collect::<Vec<_>>().join(" · "), if dominated { "FAIL (one synergy pays most everywhere)" } else { "PASS" });
        println!("best build total per wall: {}", totals.iter().map(|b| format!("D{} {}", b.0, b.1)).collect::<Vec<_>>().join(" · "));
    }
    println!("\n{} fail(s)", fails);
    if fails > 0 {
        std::process::exit(1);
    }
}
