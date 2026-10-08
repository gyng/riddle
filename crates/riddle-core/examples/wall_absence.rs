//! Cut 114 §3 probe (blind 77030eb, A: "the 20m absence came back with a D28 run and $10 949; the 8h absence
//! came back with $1 496, four dead heirs and the best depth unchanged"): from each saved camp, a 20-minute and
//! an 8-hour absence under each of the scout's wall orders (`carry · push · bank`) — the haul (purse + chest + bank),
//! the deaths, the sends banked before the wall (`held`), the record. The camps come from the dayplayer
//! (`DP_SAVE_DIR=dir dayplayer --bots away …`); a camp is *walled* when its scout's ledger holds
//! `tree::WALL_DEATHS` deaths on one floor.
//!   cargo run -q --profile fast -p riddle-core --example wall_absence -- SAVE... [--walled]
use riddle_core::Game;

struct Out {
    haul: i64,
    deaths: u32,
    runs: u32,
    held: u32,
    best: u32,
}

fn absence(text: &str, order: &str, secs: u64) -> Out {
    let mut g = Game::load(text).expect("save");
    g.lineage.orders.wall = order.into();
    let w0 = g.lineage.gold as i64 + g.lineage.town.bank as i64;
    let h0 = g.lineage.tree.acts.get(riddle_core::tree::SCOUT_HELD).copied().unwrap_or(0);
    let rep = riddle_core::offline::run_offline_counts(&mut g, secs);
    Out {
        haul: g.lineage.gold as i64 + g.lineage.town.bank as i64 - w0,
        deaths: rep.deaths.iter().map(|d| d.n).sum(),
        runs: rep.runs,
        held: g.lineage.tree.acts.get(riddle_core::tree::SCOUT_HELD).copied().unwrap_or(0) - h0,
        best: g.lineage.best_depth,
    }
}

fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().skip(1).collect();
    let walled_only = args.iter().any(|a| a == "--walled");
    let saves: Vec<&String> = args.iter().filter(|a| !a.starts_with("--")).collect();
    let cells = [("carry", 20 * 60u64), ("carry", 8 * 3600), ("push", 20 * 60), ("push", 8 * 3600), ("bank", 20 * 60), ("bank", 8 * 3600)];
    let mut sum = [(0i64, 0u32, 0u32, 0u32, 0u32); 6];
    let mut n = 0;
    let mut short_beats_long = [0u32; 3];
    for path in saves {
        let text = std::fs::read_to_string(path).expect("read");
        let g = Game::load(&text).expect("save");
        let walled = g.lineage.tree.wall.as_ref().is_some_and(|w| w.deaths >= riddle_core::tree::WALL_DEATHS);
        if walled_only && !walled {
            continue;
        }
        if !riddle_core::tree::on(&g.lineage, "scout") {
            continue;
        }
        let best0 = g.lineage.best_depth;
        let wall = g.lineage.tree.wall.as_ref().map(|w| format!("D{}×{}", w.depth, w.deaths)).unwrap_or_else(|| "-".into());
        let outs: Vec<Out> = std::thread::scope(|s| {
            let hs: Vec<_> = cells.iter().map(|(o, secs)| { let t = &text; s.spawn(move || absence(t, o, *secs)) }).collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        n += 1;
        let name = std::path::Path::new(path).file_stem().unwrap().to_string_lossy();
        print!("{name:<28} D{best0:<2} wall {wall:<7}");
        for (i, ((o, secs), x)) in cells.iter().zip(&outs).enumerate() {
            print!(" | {o} {:>3}m ${:>6} {}r {}d {}h D{}", secs / 60, x.haul, x.runs, x.deaths, x.held, x.best);
            sum[i].0 += x.haul;
            sum[i].1 += x.deaths;
            sum[i].2 += x.runs;
            sum[i].3 += x.held;
            sum[i].4 += x.best - best0;
        }
        println!();
        short_beats_long[0] += (outs[0].haul > outs[1].haul) as u32;
        short_beats_long[1] += (outs[2].haul > outs[3].haul) as u32;
        short_beats_long[2] += (outs[4].haul > outs[5].haul) as u32;
    }
    if n == 0 {
        println!("no camps");
        return;
    }
    println!("camps {n}{}", if walled_only { " (walled)" } else { "" });
    for (i, (o, secs)) in cells.iter().enumerate() {
        let (h, d, r, held, b) = sum[i];
        println!("  {o} {:>3}m: mean haul ${:.0} · deaths {:.2} · runs {:.1} · held {:.2} · record +{:.2}", secs / 60, h as f64 / n as f64, d as f64 / n as f64, r as f64 / n as f64, held as f64 / n as f64, b as f64 / n as f64);
    }
    println!("  20m out-paid 8h: carry {}/{n} · push {}/{n} · bank {}/{n}", short_beats_long[0], short_beats_long[1], short_beats_long[2]);
}
