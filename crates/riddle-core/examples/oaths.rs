//! Cut 28 §1: the oaths' gate on chosen cohort sets (`oath_lib`): per set and pool oath, the oath's
//! share of a send with the set as written and with its best set, the night's chance, the rows the
//! best set differs from the bank-optimal one by, and the edits each search took.
//! `cargo run --profile fast -p riddle-core --example oaths -- eval/cards/631fe23.raterAV.rules.json [seed] [kinds]`
#[path = "lever_lib/mod.rs"]
mod lever;
#[path = "oath_lib/mod.rs"]
mod oath_lib;
use riddle_core::RuleSet;

fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let paths: Vec<String> = args.get(1).map(|s| s.split(',').map(str::to_string).collect()).unwrap_or_default();
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let kinds: Vec<String> = args.get(3).map(|s| s.split(',').map(str::to_string).collect()).unwrap_or_else(|| riddle_core::oath::KINDS.iter().map(|k| k.to_string()).collect());
    let threads: usize = std::env::var("OATH_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8);
    for p in paths {
        let set = RuleSet::parse(&std::fs::read_to_string(&p).unwrap()).unwrap();
        let t = std::time::Instant::now();
        let g = oath_lib::lineage(&set, seed);
        let bank = oath_lib::bank_best(&g, &set);
        println!("{p} seed {seed}: best D{} · bank-optimal {:.0}% via {:?} ({:.0}s)", g.lineage.best_depth, 100.0 * bank.1, bank.2, t.elapsed().as_secs_f64());
        let next = std::sync::atomic::AtomicUsize::new(0);
        let out = std::sync::Mutex::new(Vec::new());
        let clones: Vec<riddle_core::Game> = (0..threads.min(kinds.len())).map(|_| g.clone()).collect();
        std::thread::scope(|sc| {
            for gc in clones {
                let (next, out, kinds, set, bank) = (&next, &out, &kinds, &set, &bank);
                sc.spawn(move || loop {
                    let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let Some(k) = kinds.get(i) else { break };
                    let r = oath_lib::measure(&gc, set, k, Some(bank));
                    out.lock().unwrap().push(r);
                });
            }
        });
        let mut out = out.into_inner().unwrap();
        out.sort_by(|a, b| a.kind.cmp(&b.kind));
        for r in out {
            if !r.offered {
                println!("  {:<7} not offered", r.kind);
                continue;
            }
            println!("  {:<7} {:<18} set {:>3.0}% → best {:>3.0}% · night {:>3.0}% · bank {:.0}% · diff {} · {:?}", r.kind, r.text, 100.0 * r.share_set, 100.0 * r.share_best, 100.0 * r.night(), 100.0 * r.bank_oath, r.diff, r.edits);
        }
        println!("  ({:.0}s)", t.elapsed().as_secs_f64());
    }
}
