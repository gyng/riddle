//! Paired fixed earned builds; synthetic seed/Tier5 eligibility only, no grants.
//! cargo run -q --profile fast -p riddle-core --example style_check -- PREP_DIR NEW_OUT_DIR
use riddle_core::{bloodlines::Session,endgame::Progress,specialization::Style,wire::Ev};
use std::{path::Path,collections::BTreeMap,time::Instant};
fn main(){
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();let source=Path::new(args.get(1).expect("earned preparation directory"));
    let dir=Path::new(args.get(2).expect("new output directory"));std::fs::create_dir(dir).expect("new artifact directory");
    let mut rows=Vec::new();let mut failed=false;
    for style in [Style::Sentinel,Style::Hexbinder] {
        let prepared=Session::load(&std::fs::read_to_string(source.join(format!("{}-start.json",style.id()))).unwrap()).unwrap();
        assert!(prepared.others.is_empty()&&prepared.active.run.is_none()&&!prepared.active.lineage.ended);
        assert_eq!(prepared.active.lineage.class,style.parent());assert!(prepared.active.lineage.class_level()>=10);
        assert_eq!(riddle_core::specialization::current(&prepared.active.lineage),Some(style));
        for seed in [1,3,5] {for selected in [false,true] {
            let mut s=prepared.clone();s.active.lineage.seed=seed;
            // Diagnostic controls, not separately earned ascension eligibility.
            s.active.lineage.endgame=Some(Progress{tier:5,unlocked:5,cleared:Some(4)});
            if !selected {s.set_specialization("none").unwrap();}
            let stem=format!("{}-seed{seed}-{}",style.id(),if selected{"style"}else{"base"});
            std::fs::write(dir.join(format!("{stem}-start.json")),s.save()).unwrap();
            let at=Instant::now();let mut runs=0;let mut checkins=0;let mut actions=0;let mut counters=0;
            for n in 1..=6 {
                s.active.tap=Some(BTreeMap::new());let report=s.run_offline_mode(28800,false,true);runs+=report.runs;checkins=n;
                for events in s.active.tap.as_ref().unwrap().values(){for event in events {
                    if matches!(event,Ev::Rule{verb,..} if verb.v==style.verb()){actions+=1;}
                    if matches!(event,Ev::Attack{verb:Some(v),..} if v=="riposte"){counters+=1;}
                }}s.active.tap=None;
                if s.active.lineage.ended {break;}
            }
            let cleared=s.active.lineage.ended;if selected {failed|=!cleared;}
            if cleared {assert_eq!(s.active.descent_progress().unlocked,6);}
            std::fs::write(dir.join(format!("{stem}-after.json")),s.save()).unwrap();
            let row=serde_json::json!({"style":style.id(),"selected":selected,"seed":seed,"tier":5,"hours":8*checkins,"runs":runs,"best":s.active.lineage.best_depth,"cleared":cleared,"exclusive_actions":actions,"riposte_counters":counters,"seconds":at.elapsed().as_secs_f64()});
            println!("{row}");rows.push(row);
        }}
    }
    std::fs::write(dir.join("balance.json"),serde_json::to_string_pretty(&serde_json::json!({"source":source,"scope":"paired same earned parent XP/paid Legacy/gear/owned policy; synthetic seed and Tier5 eligibility; no grants or in-campaign policy edits; fixed base is not an optimised policy search","hours_limit":48,"styles_pass":!failed,"cases":rows})).unwrap()).unwrap();
    if failed {std::process::exit(1);}
}
