//! Fixed owned builds; synthetic Tier6/seed eligibility, never a progression proof.
use riddle_core::{bloodlines::Session,endgame::Progress,wire::Ev};
use std::{path::Path,collections::BTreeMap};
fn main(){
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();let source=Path::new(args.get(1).expect("earned preparations"));
    let dir=Path::new(args.get(2).expect("new output directory"));std::fs::create_dir(dir).expect("new directory");
    let mut rows=Vec::new();let mut failed=false;
    for style in ["sentinel","hexbinder"] {for seed in [1,3,5] {
        let mut s=Session::load(&std::fs::read_to_string(source.join(format!("{style}-start.json"))).unwrap()).unwrap();
        assert!(s.active.run.is_none()&&s.others.is_empty());s.active.lineage.seed=seed;
        s.active.lineage.endgame=Some(Progress{tier:6,unlocked:6,cleared:Some(5)});
        let stem=format!("{style}-seed{seed}");std::fs::write(dir.join(format!("{stem}-start.json")),s.save()).unwrap();
        // A normal stepped encounter branch for an actual positive-hit checkpoint.
        // No arena grants or encounter overrides; only seed/tier eligibility above.
        if style=="sentinel"&&seed==1 {
            let mut played=s.clone();let mut found=false;
            for _ in 0..5000 {
                if played.active.lineage.ended {break;}
                if played.active.run.is_none(){played.send();}
                let before=played.save();let step=played.step(20);
                if step.events.iter().any(|e|matches!(e,Ev::Recover{..})) {
                    std::fs::write(dir.join("played-before.json"),before).unwrap();
                    std::fs::write(dir.join("played-step.json"),serde_json::to_string(&step).unwrap()).unwrap();
                    std::fs::write(dir.join("played-after.json"),played.save()).unwrap();found=true;break;
                }
            }
            assert!(found,"normal encounter must demonstrate actual Leeching recovery");
        }
        let mut checkins=0;let mut recovered=0;let mut heals=0;let mut runs=0;
        for n in 1..=6 {
            s.active.tap=Some(BTreeMap::new());let report=s.run_offline_mode(28800,false,true);checkins=n;runs+=report.runs;
            for events in s.active.tap.as_ref().unwrap().values(){for ev in events {
                if let Ev::Recover{amount,..}=ev {recovered+=amount;heals+=1;}
            }}s.active.tap=None;
            if s.active.lineage.ended {break;}
        }
        let clear=s.active.lineage.ended;failed|=!clear;
        std::fs::write(dir.join(format!("{stem}-after.json")),s.save()).unwrap();
        let row=serde_json::json!({"style":style,"seed":seed,"tier":6,"hours":checkins*8,"runs":runs,"cleared":clear,"best":s.active.lineage.best_depth,"recoveries":heals,"enemy_healed":recovered});println!("{row}");rows.push(row);
    }}
    std::fs::write(dir.join("balance.json"),serde_json::to_string_pretty(&serde_json::json!({"source":source,"synthetic_seed_tier_eligibility":true,"grants":false,"hours_limit":48,"pass":!failed,"cases":rows})).unwrap()).unwrap();
    if failed {std::process::exit(1);}
}
