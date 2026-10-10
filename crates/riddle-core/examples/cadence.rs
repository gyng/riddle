//! Cut 121 §1 (blind 1a7d834 A: `+$10 823` at an 8 h return vs `+$14 723` at a 4 h one): the return cadence probe.
//! Each camp save (a dayplayer `DP_SAVE_DIR` camp) is played twice from the same state: a 4 h absence and an 8 h one.
//! Prints per camp the income (`GoldSummary.earned`), the workers' spend (`spent_by_workers`), the purse's net, runs and
//! deaths, and the 8 h ÷ 4 h ratios; flags a camp whose 8 h income is below its 4 h one. Diagnostic, never a gate.
//!   cargo run -q --profile fast -p riddle-core --example cadence -- DIR [--match d2-c0] [--hours 4,8]
use riddle_core::Game;

struct Out {
    earned: i64,
    workers: i64,
    net: i64,
    runs: u32,
    deaths: u32,
    legacy: u32,
}

fn absence(save: &str, hours: u64) -> Out {
    absence_with(save, hours, None)
}

/// `sink`: the apprentice's sink order for the absence (`--sinks`: `both` against `off`).
fn absence_with(save: &str, hours: u64, sink: Option<&str>) -> Out {
    let mut g = Game::load(save).expect("camp save");
    if let Some(s) = sink {
        g.lineage.orders.sink = s.into();
    }
    let r = g.run_offline(hours * 3600);
    let gold = r.gold.clone().unwrap_or_default();
    Out {
        earned: i64::from(gold.earned.unwrap_or(0)),
        workers: i64::from(gold.spent_by_workers.unwrap_or(0)),
        net: i64::from(gold.net.unwrap_or(0)),
        runs: r.runs,
        deaths: r.deaths.iter().map(|d| d.n).sum(),
        legacy: r.legacy_earned,
    }
}

fn main() {
    riddle_core::engine::set_capsules(false);
    let a: Vec<String> = std::env::args().collect();
    let dir = a.get(1).expect("DIR of camp saves");
    let pat = a.iter().position(|x| x == "--match").and_then(|i| a.get(i + 1)).cloned().unwrap_or_default();
    let hours: Vec<u64> = a.iter().position(|x| x == "--hours").and_then(|i| a.get(i + 1)).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_else(|| vec![4, 8]);
    let (short, long) = (hours[0], hours[1]);
    // `--sinks`: does a sink return more than it costs? 8 h with the sinks `both` against `off` from each camp —
    // the income gained beside the gold the sinks took
    if a.iter().any(|x| x == "--sinks") {
        let mut files: Vec<_> = std::fs::read_dir(dir).expect("dir").filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.to_string_lossy().contains(&pat)).collect();
        files.sort();
        let (mut more, mut n) = (0, 0);
        for f in files {
            let save = std::fs::read_to_string(&f).expect("read");
            let (on, off) = (absence_with(&save, long, Some("both")), absence_with(&save, long, Some("off")));
            let gain = on.earned - off.earned;
            let cost = on.workers - off.workers;
            n += 1;
            let flag = if cost > 0 && gain > cost { more += 1; "RETURNS MORE" } else { "" };
            println!("{:<34} income on {:>7} off {:>7} gain {:>7} · sinks {:>7} · deaths {}/{} · ◆{}/{} {flag}", f.file_name().unwrap().to_string_lossy(), on.earned, off.earned, gain, cost, on.deaths, off.deaths, on.legacy, off.legacy);
        }
        println!("\n{n} camps: a sink's income gain above its cost on {more}");
        return;
    }
    let mut files: Vec<_> = std::fs::read_dir(dir).expect("dir").filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json") && p.to_string_lossy().contains(&pat)).collect();
    files.sort();
    let (mut worse_earned, mut worse_net, mut n) = (0, 0, 0);
    let mut ratios = Vec::new();
    println!("{:<34} {:>8} {:>8} {:>7} {:>7} {:>8} {:>8} {:>9} {:>9} {:>5} {:>5}", "camp", "earn4", "earn8", "work4", "work8", "net4", "net8", "runs4/8", "death4/8", "×earn", "flag");
    for f in files {
        let save = std::fs::read_to_string(&f).expect("read");
        let (s, l) = (absence(&save, short), absence(&save, long));
        n += 1;
        let r = l.earned as f64 / s.earned.max(1) as f64;
        ratios.push(r);
        let flag = if l.earned < s.earned { worse_earned += 1; "EARN" } else if l.net < s.net { worse_net += 1; "net" } else { "" };
        println!("{:<34} {:>8} {:>8} {:>7} {:>7} {:>8} {:>8} {:>4}/{:<4} {:>4}/{:<4} {:>5.2} {:>5}  ◆{}/{}", f.file_name().unwrap().to_string_lossy(), s.earned, l.earned, s.workers, l.workers, s.net, l.net, s.runs, l.runs, s.deaths, l.deaths, r, flag, s.legacy, l.legacy);
    }
    ratios.sort_by(|a, b| a.total_cmp(b));
    let med = ratios.get(ratios.len() / 2).copied().unwrap_or(0.0);
    println!("\n{n} camps: {long} h income below {short} h on {worse_earned}; net below (income not) on {worse_net}; median income ratio {med:.2}, least {:.2}", ratios.first().copied().unwrap_or(0.0));
}
