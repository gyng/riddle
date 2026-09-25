//! Cut 25 §1: kit vs rules — on a cohort set's lineage after an absence, the whole forge's bank
//! move against the set's best single-row edit, paired seeds (the same sims, the same floors).
//!   cargo run -q --profile fast -p riddle-core --example lever -- [--seeds 2] [--sims 96] [--hours 4] [filter]
#[path = "lever_lib/mod.rs"]
mod lever;
use riddle_core::RuleSet;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 2);
    let sims = get("--sims", 96) as u32;
    let hours = get("--hours", 4);
    let filter: Option<&String> = args.iter().skip(1).find(|a| !a.starts_with("--") && a.parse::<u64>().is_err());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .filter(|(n, _)| filter.is_none_or(|f| n.contains(f.as_str())))
        .collect();
    sets.sort_by(|a, b| a.0.cmp(&b.0));
    // `--gate`: the metrics row's own measure (`lever::gate`), pooled over the seeds.
    if args.iter().any(|a| a == "--gate") {
        let (mut pass, mut n) = (0, 0);
        for (name, set) in &sets {
            if !set.rows.iter().any(|r| r.verb.v == "bank") {
                continue;
            }
            let t = std::time::Instant::now();
            let ms: Vec<(f64, f64, String)> = (1..=seeds).map(|s| lever::gate(set, s, hours, sims, 0.05)).collect();
            let kit = ms.iter().map(|m| m.0).sum::<f64>() / ms.len() as f64;
            let row = ms.iter().map(|m| m.1).sum::<f64>() / ms.len() as f64;
            let ok = kit < row || kit <= 0.005;
            n += 1;
            pass += ok as u32;
            println!("{name}: kit {:+.1} · row ≥ {:+.1} ({}) {}{:.0}s", 100.0 * kit, 100.0 * row, ms.iter().map(|m| format!("{:+.0}/{:+.0} {}", 100.0 * m.0, 100.0 * m.1, m.2)).collect::<Vec<_>>().join(" · "), if ok { "" } else { "FAIL " }, t.elapsed().as_secs_f64());
        }
        println!("kit < row on {pass}/{n} banking sets");
        return;
    }
    let (mut pass, mut n) = (0, 0);
    for (name, set) in &sets {
        let banks = set.rows.iter().any(|r| r.verb.v == "bank");
        let (mut kit, mut row, mut base, mut sl) = (0.0, 0.0, 0.0, [0.0f64; 3]);
        let t = std::time::Instant::now();
        let mut notes = Vec::new();
        for seed in 1..=seeds {
            let m = lever::measure(set, seed, hours, sims);
            kit += m.kit / seeds as f64;
            base += m.base_bank / seeds as f64;
            for (x, k) in sl.iter_mut().zip(m.kit_slots) {
                *x += k / seeds as f64;
            }
            let r = m.row.max(m.drop.0).max(m.drop_kitted.0);
            row += r / seeds as f64;
            notes.push(format!("s{seed} D{} L{} {:.0}%→kit {:+.0} ({:.0}%) · edit {:+.0} `{}` · drop {:+.0} `{}` · kitted drop {:+.0} `{}`", m.best_depth, m.level, 100.0 * m.base_bank, 100.0 * m.kit, 100.0 * m.kit_bank, 100.0 * m.row, m.row_label, 100.0 * m.drop.0, m.drop.1, 100.0 * m.drop_kitted.0, m.drop_kitted.1));
        }
        // (a set the forge does not move at all passes: nothing for a row to beat)
        let ok = kit < row || kit <= 0.005;
        if banks {
            n += 1;
            pass += ok as u32;
        }
        println!("{name}: base {:.0}% · kit {:+.1} (w {:+.1} a {:+.1} p {:+.1}) · row {:+.1} · {}{:.0}s", 100.0 * base, 100.0 * kit, 100.0 * sl[0], 100.0 * sl[1], 100.0 * sl[2], 100.0 * row, if !banks { "(no bank row) " } else if ok { "" } else { "FAIL " }, t.elapsed().as_secs_f64());
        for x in notes {
            println!("    {x}");
        }
    }
    println!("kit < row on {pass}/{n} banking sets");
}
