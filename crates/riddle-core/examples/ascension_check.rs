//! Seeded diagnostic from one fixed earned preparation; not independently earned campaigns.
//! cargo run -q --profile fast -p riddle-core --example ascension_check -- SAVE NEW_OUT_DIR [--spend-legacy]
use riddle_core::{bloodlines::Session,endgame::Progress};
use std::{path::Path,time::Instant};
fn main(){
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();
    let source=args.get(1).expect("SAVE required");let dir=Path::new(args.get(2).expect("NEW_OUT_DIR required"));
    std::fs::create_dir(dir).expect("new artifact directory");
    let raw=std::fs::read_to_string(source).expect("source save");
    let prepared=Session::load(&raw).expect("valid preparation");
    assert!(prepared.others.is_empty()&&prepared.active.run.is_none()&&!prepared.active.lineage.ended,"single clean pre-descent fixture");
    let mut rows=Vec::new();let mut failed=false;
    for seed in [1,3,5] {for tier in [1,3,5] {
        let mut s=prepared.clone();
        // Explicit diagnostic controls: common preparation, altered RNG and tier eligibility.
        s.active.lineage.seed=seed;
        s.active.lineage.endgame=Some(Progress{tier,unlocked:tier,cleared:Some(tier-1)});
        // An explicit player action on existing earned currency, never a stat grant.
        let spend_legacy=args.iter().any(|arg|arg=="--spend-legacy");
        if spend_legacy {
            for id in riddle_core::legacy::IDS {for _ in 0..riddle_core::legacy::CAP {
                riddle_core::legacy::buy(&mut s.active,id).expect("earned Legacy can buy this build");
            }}
        }
        let stem=format!("seed{seed}-tier{tier}");std::fs::write(dir.join(format!("{stem}-start.json")),s.save()).unwrap();
        let at=Instant::now();let mut runs=0;let mut checkins=0;
        for n in 1..=42 {
            let report=s.run_offline_mode(28800,false,true);runs+=report.runs;checkins=n;
            if s.active.lineage.ended {break;}
        }
        let cleared=s.active.lineage.ended;
        if cleared {assert_eq!(s.active.descent_progress().unlocked,tier+1);}
        failed|=!cleared;
        std::fs::write(dir.join(format!("{stem}-after.json")),s.save()).unwrap();
        let row=serde_json::json!({"seed":seed,"tier":tier,"checkins":checkins,"hours":8*checkins,"runs":runs,"best":s.active.lineage.best_depth,"cleared":cleared,"seconds":at.elapsed().as_secs_f64(),"spend_earned_legacy":spend_legacy,"synthetic_rng_and_eligibility":true});
        println!("{row}");rows.push(row);
    }}
    let result=serde_json::json!({"scope":"seeded fixed-preparation diagnostic, not earned campaign or statistical gate","source":source,"days_limit":14,"pass":!failed,"cases":rows});
    std::fs::write(dir.join("balance.json"),serde_json::to_string_pretty(&result).unwrap()).unwrap();
    if failed {std::process::exit(1);}
}
