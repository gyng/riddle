//! Timing probe: one 8 h offline batch, one forecast, one death verdict.
use riddle_core::Game;
use std::time::Instant;
fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut g = Game::new(seed);
    let t = Instant::now();
    let r = g.run_offline(8 * 3600);
    let dt = t.elapsed();
    println!("offline 8h: {:?} · runs {} sampled {} · ticks {} ({:.2} µs/tick) · deaths {} · learned {} pending {}",
        dt, r.runs, r.sampled, g.batch.turns, dt.as_secs_f64() * 1e6 / g.batch.turns.max(1) as f64, g.deaths.len(), r.learned.len(), r.pending.len());
    let t = Instant::now();
    let f = g.forecast();
    println!("forecast: {:?} known_to {}", t.elapsed(), f.known_to);
    if let Some(id) = g.deaths.keys().next().copied() {
        let t = Instant::now();
        let v = riddle_core::trace::verdict(&mut g, id);
        println!("verdict: {:?} {:?}", t.elapsed(), v);
    }
}
