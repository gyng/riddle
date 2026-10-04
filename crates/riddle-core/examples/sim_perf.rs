//! Single-thread work accounting and exact-output fingerprint for a saved camp.
use riddle_core::{forecast, Game};
use std::time::Instant;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let g = Game::load(&std::fs::read_to_string(a.get(1).expect("sim_perf SAVE [wall|packages|offline|clone|setup]")).unwrap()).unwrap();
    let mode = a.get(2).map(String::as_str).unwrap_or("packages");
    forecast::set_parallel_sims(false);
    let t = Instant::now();
    let (out, counts) = forecast::measure_work(|| match mode {
        "clone" | "setup" => {
            let iterations = 4096u32;
            let tag = forecast::forecast_tag(&g, g.lineage.best_depth + 1);
            for i in 0..iterations {
                let copy = if mode == "clone" { g.sim_clone() } else { forecast::sim_game(&g, g.lineage.rules(), tag, i % forecast::FORECAST_SIMS, None) };
                std::hint::black_box(copy);
            }
            let sample = if mode == "clone" { g.sim_clone() } else { forecast::sim_game(&g, g.lineage.rules(), tag, (iterations - 1) % forecast::FORECAST_SIMS, None) };
            serde_json::json!({ "iterations": iterations, "sample": sample.save() }).to_string()
        }
        "wall" => serde_json::to_string(&riddle_core::wall::search(&g)).unwrap(),
        "packages" => serde_json::to_string(&riddle_core::packages::options(&g, forecast::FORECAST_SIMS)).unwrap(),
        "offline" => { let mut g = g; let r = g.run_offline(8 * 3600); serde_json::to_string(&(r, g.save())).unwrap() },
        _ => panic!("unknown workload: {mode}"),
    });
    let elapsed = t.elapsed().as_secs_f64();
    if let Some(path) = a.get(3) { std::fs::write(path, &out).unwrap(); }
    let hash = out.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    println!("{}", serde_json::json!({"mode": mode, "seconds": elapsed, "output": format!("{hash:016x}"), "work": counts}));
}
