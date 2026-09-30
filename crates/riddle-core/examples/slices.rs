//! Chunked offline must equal one call, including after a watched death + save/load round trip
//! (the client's exact sequence: send → step to death → keep → save → reload → 16 × 30 min).
fn main() {
    let seed = 5;
    let mut g = riddle_core::Game::new_literal(seed);
    let _ = g.send();
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < 100_000 { let _ = g.step(10); n += 10; }
    println!("watched run: over {:?} after {n} ticks, pending_exit {}", g.run.as_ref().and_then(|r| r.over), g.pending_exit.is_some());
    let _ = g.keep(vec![]);
    let save = g.save();
    let mut b = riddle_core::Game::load(&save).expect("load");
    println!("after load: run {:?} rest_left {}", b.run.as_ref().map(|r| (r.turn, r.over)), b.lineage.rest_left);
    let (mut runs, mut banked, mut returned, mut deaths) = (0, 0, 0, 0);
    let mut worst = None;
    for i in 0..16 {
        let r = riddle_core::offline::run_offline_quick(&mut b, 1800);
        runs += r.runs; banked += r.banked; returned += r.returned; deaths += r.deaths.iter().map(|d| d.n).sum::<u32>();
        if r.worst_death_id.is_some() { worst = r.worst_death_id; }
        if i < 3 { println!("  slice {i}: runs {} banked {} returned {} deaths {} worst {:?} rested {} run_now {:?}", r.runs, r.banked, r.returned, r.deaths.iter().map(|d| d.n).sum::<u32>(), r.worst_death_id, r.rested_s, b.run.as_ref().map(|r| (r.turn, r.over, r.depth))); }
    }
    println!("16 slices after load: runs {runs} banked {banked} returned {returned} deaths {deaths} worst {worst:?}");
}
