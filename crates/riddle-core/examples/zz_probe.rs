use riddle_core::Game;
fn main() {
    for seed in 1..=12u64 {
        let mut g = Game::new(seed);
        riddle_core::tree::grant(&mut g.lineage, &riddle_core::tree::LEGACY);
        let mut line = format!("s{seed:2}:");
        for _ in 0..6 {
            g.lineage.rest_left = 0;
            g.start_run(None);
            g.run_to_end(riddle_core::engine::MAX_TURNS_PER_RUN);
            let r = g.run.as_ref().unwrap();
            let (d, md, t, hp, mx, sec, loot) = (r.depth, r.max_depth, r.over, r.hero.hp, r.hero.max_hp, r.secured, r.loot);
            g.finish_run();
            g.auto_keep();
            line += &format!(" [{:?} D{d}/{md} hp{hp}/{mx} s{sec} l{loot} best{}]", t.unwrap(), g.lineage.best_depth);
        }
        println!("{line}");
    }
}
