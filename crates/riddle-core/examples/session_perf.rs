//! Actual multihero quick catch-up. Session, not selected Game only.
//! cargo run --profile fast --example session_perf -- SAVE [HOURS] [REPEATS] [OUTPUT]
//! RIDDLE_SHIPPING_WORDS=1 mirrors the native app/WASM event wording.
use riddle_core::bloodlines::Session;
use std::time::Instant;
fn main() {
    let args:Vec<_>=std::env::args().collect();
    if std::env::var("RIDDLE_SHIPPING_WORDS").as_deref()==Ok("1") {
        riddle_core::chronicle::use_shipping_words();
    }
    let input=std::fs::read_to_string(args.get(1).expect("session_perf SAVE [HOURS] [REPEATS] [OUTPUT]")).unwrap();
    let source=Session::load(&input).unwrap();
    let hours=args.get(2).map(|s|s.parse::<u64>().unwrap()).unwrap_or(8);
    let repeat=args.get(3).map(|s|s.parse::<usize>().unwrap()).unwrap_or(5);
    assert!(hours>0&&repeat>0);
    riddle_core::forecast::set_parallel_sims(false);
    let mut expected=None;
    for n in 0..=repeat {
        // Loading/cloning and save serialization aren't part of the workload timer.
        let mut game=source.clone();let started=Instant::now();
        let report=game.run_offline_mode(hours.checked_mul(3600).unwrap(),false,true);
        let wire=serde_json::to_string(&report).unwrap();
        let seconds=started.elapsed().as_secs_f64();
        let output=serde_json::json!({"report":wire,"save":game.save()}).to_string();
        let hash=output.bytes().fold(0xcbf2_9ce4_8422_2325u64,|h,b|(h^u64::from(b)).wrapping_mul(0x0100_0000_01b3));
        if let Some(ref first)=expected{assert_eq!(&output,first,"fresh repetitions must be identical");}
        else {if let Some(path)=args.get(4){std::fs::write(path,&output).unwrap();}expected=Some(output);}
        if n==0{continue;} // One warm-up outside the recorded series.
        let mut ticks:Vec<_>=game.others.iter().map(|(id,g)|(*id,g.batch.turns)).collect();
        ticks.push((game.selected,game.active.batch.turns));ticks.sort_unstable();
        println!("{}",serde_json::json!({"repeat":n,"hours":hours,"slots":ticks.len(),"seconds":seconds,"runs":report.runs,"ticks":ticks,"output":format!("{hash:016x}")}));
    }
}
