//! The progression timeline (docs/PROGRESSION.md): a fresh lineage played over 14 days by
//! (a) the dayplayer's policy (verbatim, its own 3 × 8 h schedule), (a') the same policy with the
//! gold sinks a player of this build uses (forge steps, oaths), and (b) each recent cohort's rater
//! set replayed as the goal a player grows into (the rows the lineage can write, in the set's order,
//! under its row cap; the unlocks the set needs bought first) over the three-absence day (8 h ·
//! 20 m · 4 h · 4 h · 7 h 40 m). Per day: marks earned by source, spent and on what, unspent, gold
//! earned by source and spent by sink, best depth, the reveal ladder, unlocks bought and each one's
//! measured move (leave-one-out at the day's end: the lineage's camp panel with and without it,
//! paired seeds), and the affordable unlocks left unbought with their measured move.
//!   RIDDLE_THREADS=12 cargo run -q --profile fast -p riddle-core --example progression -- \
//!     [--days 14] [--seeds 2] [--dp-seeds 3] [--jobs 12] [--filter AU] [--out docs/progression/timeline.json]
//! `PROG_WALLS_ONLY=1` skips the unlock measures and keeps the wall probe (docs/progression/walls.json);
//! `docs/progression/project.py` projects the proposal onto the output (docs/PROGRESSION.md).
#[path = "progression_lib/mod.rs"]
mod prog;
use prog::*;
use serde_json::json;

fn main() {
    // RUNS_UI: real games at scale keep no replay capsules (a lineage clone per send never read)
    riddle_core::engine::set_capsules(false);
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let gets = |k: &str| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).cloned();
    let days = get("--days", 14) as usize;
    let seeds = get("--seeds", 2);
    let dp_seeds = get("--dp-seeds", 3);
    let jobs = get("--jobs", 12) as usize;
    let verbose = a.iter().any(|x| x == "--verbose");
    // `--bars`: the lineages alone (no leave-one-out, no wall probe) and the Cut 29 bars (as metrics prints them)
    let measure = !a.iter().any(|x| x == "--bars");
    let filter = gets("--filter");
    let out_path = gets("--out");
    // one lineage per worker; the panels inside run single-threaded
    riddle_core::forecast::set_parallel_sims(false);
    let sets = rater_sets();
    let day3x8: Vec<u64> = vec![8 * 3600; 3];
    let three_absence: Vec<u64> = prog::three_absence();
    let mut jobs_list: Vec<(String, Mode, u64, Vec<u64>)> = Vec::new();
    for s in 1..=dp_seeds {
        jobs_list.push(("dayplayer".into(), Mode::Day { sinks: false }, s, day3x8.clone()));
        jobs_list.push(("dayplayer+sinks".into(), Mode::Day { sinks: true }, s, day3x8.clone()));
    }
    for (name, set) in &sets {
        for s in 1..=seeds {
            jobs_list.push((name.clone(), Mode::Rater(set.clone()), s, three_absence.clone()));
        }
    }
    if let Some(f) = &filter {
        jobs_list.retain(|j| j.0.contains(f.as_str()));
    }
    let t0 = std::time::Instant::now();
    let n = jobs_list.len();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results = std::sync::Mutex::new(Vec::<(usize, Out)>::new());
    std::thread::scope(|sc| {
        for _ in 0..jobs.min(n) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if i >= n {
                    break;
                }
                let (name, mode, seed, sched) = jobs_list[i].clone();
                let t = std::time::Instant::now();
                let o = play(name, mode, seed, days, &sched, verbose, measure);
                eprintln!("done {} s{} ({:.0}s) final D{}", o.name, o.seed, t.elapsed().as_secs_f64(), o.days.last().map(|d| d.best).unwrap_or(0));
                results.lock().unwrap().push((i, o));
            });
        }
    });
    let mut outs = results.into_inner().unwrap();
    outs.sort_by_key(|x| x.0);
    let outs: Vec<Out> = outs.into_iter().map(|x| x.1).collect();
    for o in &outs {
        println!("\n{} ({}) seed {} · rules now = {}", o.name, o.mode, o.seed, o.rules);
        println!("day best  +◆  spent  ◆end ◆max  gold$  $max  buys");
        for (i, d) in o.days.iter().enumerate() {
            let src: Vec<String> = d.marks_src.iter().map(|(k, v)| format!("{k}{v}")).collect();
            println!(
                "{:>3} {:>4}{} {:>3} {:>5} {:>5} {:>4} {:>6} {:>5}  {} | src {} | no-move {}",
                i + 1,
                d.best,
                if d.new_best { "*" } else { " " },
                d.marks_src.values().sum::<i64>(),
                d.marks_spent,
                d.marks_end,
                d.marks_max,
                d.gold_end,
                d.gold_max,
                d.buys.iter().map(|b| if b.via == "marks" { format!("{}◆{}", b.id, b.cost) } else { format!("{}${}", b.id, b.gold) }).collect::<Vec<_>>().join(" "),
                src.join(" "),
                d.measured.iter().filter(|m| m["meaningful"] == json!(false)).map(|m| m["id"].as_str().unwrap_or("").to_string()).collect::<Vec<_>>().join(" ")
            );
        }
        println!("{}", summary(o));
    }
    println!("\n({} lineages, {:.0}s)", outs.len(), t0.elapsed().as_secs_f64());
    for (name, value, ok) in bars(&outs) {
        println!("{:<64} {:>14}  {}", name, value, if ok { "PASS" } else { "FAIL" });
    }
    if let Some(p) = out_path {
        let doc = json!({
            "build": option_env!("GIT_HEAD").unwrap_or("working tree"),
            "days": days,
            "schedules": {"dayplayer": "3 × 8 h", "rater": "8 h · 20 m · 4 h · 4 h · 7 h 40 m"},
            "measure": "leave-one-out at the day's end: the camp panel (50 sims, paired seeds) with the unlock vs without; meaningful = bank or reach@half-floor +≥3 pts past its ±, or mean depth +≥0.25 past its ±",
            "lineages": outs.iter().map(|o| json!({"summary": summary(o), "days": o.days.iter().enumerate().map(|(i, d)| day_json(i, d)).collect::<Vec<_>>()})).collect::<Vec<_>>(),
        });
        std::fs::write(&p, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
        eprintln!("wrote {p}");
    }
}
