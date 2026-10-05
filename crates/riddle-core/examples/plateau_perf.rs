//! Profile the selected plateau forecast separately from actual catch-up.
//! cargo run -q --profile fast --example plateau_perf -- SAVE [HOURS]
use riddle_core::{bloodlines::Session,forecast,offline};
fn main() {
    let args:Vec<_>=std::env::args().collect();
    if args.get(1).is_some_and(|a|a=="--fixtures") {
        let dir=args.get(2).expect("plateau_perf --fixtures DIR");std::fs::create_dir_all(dir).unwrap();
        let seeds=args.get(3).map(|s|s.parse::<u64>().unwrap()).unwrap_or(16);assert!(seeds>0);
        for seed in 1..=seeds {
            let mut g=riddle_core::engine::Game::new_literal(seed);
            let mut rules=g.lineage.rules().clone();
            rules.rows.insert(0,riddle_core::rules::Row::new(vec![riddle_core::rules::Cond::n("hp<",35)],riddle_core::rules::Verb::new("return")));
            g.set_rules(rules).unwrap();offline::run_offline_counts(&mut g,16*3600);
            std::fs::write(format!("{dir}/literal-{seed}.json"),g.save()).unwrap();
        }
        return;
    }
    let input=std::fs::read_to_string(args.get(1).expect("plateau_perf SAVE [HOURS]")).unwrap();
    let mut game=Session::load(&input).unwrap();
    let hours=args.get(2).map(|s|s.parse::<u64>().unwrap()).unwrap_or(8);
    assert!(hours>0);forecast::set_parallel_sims(false);
    let (report,work)=forecast::measure_work(||game.run_offline_mode(hours.checked_mul(3600).unwrap(),false,true));
    let g=&game.active;let rules=g.lineage.rules();let depth=g.stall.depth.max(1)+1;
    let base=forecast::reach_cached(g,rules,depth,offline::QUICK_STALL_SIMS,forecast::forecast_tag(g,g.lineage.best_depth+1),forecast::CAMP_TICK_BUDGET);
    println!("{}",serde_json::json!({"hours":hours,"runs":report.runs,"stall":report.stall.as_ref().map(|s|(&s.text,&s.patches)),"base":base,"work":work,"threshold":offline::STALL_DELTA}));
}
