//! Fixed earned preparation, explicit synthetic seed/tier eligibility, actual paid builds.
//! cargo run -q --profile fast -p riddle-core --example legacy_check -- SAVE NEW_OUT_DIR
use riddle_core::{bloodlines::Session,endgame::Progress,legacy};
use std::{path::Path,time::Instant};
fn main(){
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();
    let source=args.get(1).expect("SAVE required");
    let dir=Path::new(args.get(2).expect("NEW_OUT_DIR required"));
    std::fs::create_dir(dir).expect("new artifact directory");
    let prepared=Session::load(&std::fs::read_to_string(source).expect("source save")).expect("valid earned preparation");
    assert!(prepared.others.is_empty()&&prepared.active.run.is_none()&&!prepared.active.lineage.ended);
    let mut rows=Vec::new();let mut failed=false;
    for seed in [1,3,5] {for (branch,leaf) in [("restoration","mending"),("control","venom"),("clear_lungs","fireward")] {
        let mut s=prepared.clone();
        // Diagnostic controls, not independently earned Tier5 campaigns.
        s.active.lineage.seed=seed;
        s.active.lineage.endgame=Some(Progress{tier:5,unlocked:5,cleared:Some(4)});
        let initial=legacy::current(&s.active.lineage).unwrap().points;
        for id in legacy::IDS {for _ in 0..legacy::CAP {s.upgrade_hero(id).expect("owned root purchase");}}
        s.upgrade_hero(branch).expect("owned follow-up purchase");
        s.upgrade_hero(leaf).expect("owned leaf purchase");
        assert_eq!(initial-legacy::current(&s.active.lineage).unwrap().points,108);
        let stem=format!("seed{seed}-{leaf}");std::fs::write(dir.join(format!("{stem}-start.json")),s.save()).unwrap();
        let at=Instant::now();let mut runs=0;let mut checkins=0;
        for n in 1..=6 {
            let report=s.run_offline_mode(28800,false,true);runs+=report.runs;checkins=n;
            if s.active.lineage.ended {break;}
        }
        let cleared=s.active.lineage.ended;failed|=!cleared;
        if cleared {assert_eq!(s.active.descent_progress().unlocked,6);}
        std::fs::write(dir.join(format!("{stem}-after.json")),s.save()).unwrap();
        let row=serde_json::json!({"seed":seed,"tier":5,"build":leaf,"hours":8*checkins,"runs":runs,"best":s.active.lineage.best_depth,"cleared":cleared,"seconds":at.elapsed().as_secs_f64()});
        println!("{row}");rows.push(row);
    }}
    std::fs::write(dir.join("balance.json"),serde_json::to_string_pretty(&serde_json::json!({
        "scope":"fixed earned preparation; synthetic seed/tier eligibility; actual purchases; no grants",
        "source":source,"hours_limit":48,"pass":!failed,"cases":rows})).unwrap()).unwrap();
    if failed {std::process::exit(1);}
}
