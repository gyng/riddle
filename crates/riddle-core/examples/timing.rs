//! Where does a gate job spend its time? `cargo run --profile fast --example timing -- [seed] [hours]`
use std::time::Instant;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: u64 = a.get(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let hours: u64 = a.get(2).and_then(|s| s.parse().ok()).unwrap_or(8);
    // A third argument `seq` runs the panels one sim after another (the wasm cost).
    if a.get(3).is_some_and(|s| s == "seq") {
        riddle_core::forecast::set_parallel_sims(false);
    }
    let mut g = riddle_core::Game::new_literal(seed);
    let t = Instant::now();
    let rep = g.run_offline(hours * 3600);
    println!("run_offline({hours}h): {:.2}s  runs {}  deaths {}  ticks {}", t.elapsed().as_secs_f64(), rep.runs, g.deaths.len(), g.batch.turns);
    let ids: Vec<u32> = g.deaths.keys().copied().collect();
    let t = Instant::now();
    let mut n = 0;
    for id in &ids { if riddle_core::trace::verdict(&mut g, *id).is_some() { n += 1; } }
    println!("verdicts x{n}: {:.2}s ({:.3}s each)", t.elapsed().as_secs_f64(), t.elapsed().as_secs_f64() / n.max(1) as f64);
    let t = Instant::now();
    let _ = g.forecast();
    println!("forecast: {:.2}s", t.elapsed().as_secs_f64());
    let t = Instant::now();
    let f = g.forecast_refine();
    println!("forecast_refine: {:.2}s  (ends pm ±{:.0})", t.elapsed().as_secs_f64(), f.ends.as_ref().map(|e| e.pm * 100.0).unwrap_or(0.0));
    if let Some(id) = ids.last() { let t = Instant::now(); let _ = riddle_core::trace::death(&mut g, *id); println!("death() with deltas: {:.2}s", t.elapsed().as_secs_f64()); }
}
