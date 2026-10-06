//! Earn harder descents from normal seven-day Gunner saves. Never grants tiers/builds.
//! gunner_descents EARNED_DIR NEW_OUT_DIR [--seed 1|3|5] [--gun long_gun|short_gun] [--climb-hours 336] [--venom]
use riddle_core::{bloodlines::Session,firearm,hero::Class,legacy};
use std::path::Path;
#[path="gunner_support/mod.rs"]
mod gunner_support;
fn prepare(s:&mut Session,venom:bool)->Vec<String> {
    let mut actions=gunner_support::camp(s,true);
    if venom {for id in ["control","venom"] {
        if legacy::offers(&s.active.lineage,false).iter().any(|o|o.id==id&&o.affordable){s.upgrade_hero(id).unwrap();actions.push(format!("Legacy {id}"));}
    }}actions
}
fn play(source:&Path,dir:&Path,seed:u64,kind:&str,climb_hours:u32,venom:bool)->serde_json::Value {
    let prefix=format!("seed{seed}-{kind}");
    let mut s=Session::load(&std::fs::read_to_string(source.join(format!("{prefix}-after.json"))).unwrap()).unwrap();
    assert!(s.others.is_empty()&&s.active.run.is_none());assert_eq!(s.active.lineage.seed,seed);
    assert_eq!(s.active.lineage.class,Class::Gunner);assert_eq!(firearm::starting_kind(&s.active.lineage),kind);
    let initial=s.active.descent_progress();assert!(initial.unlocked>=1&&initial.tier==1);
    std::fs::write(dir.join(format!("{prefix}-source.json")),s.save()).unwrap();
    let mut checks=Vec::new();let mut hours=0;let mut owned5=false;
    while hours<=climb_hours {
        if s.active.lineage.ended {
            let tier=s.active.descent_progress().unlocked.min(5);
            s.begin_descent(tier).expect("select only the next actually owned descent");
            checks.push(serde_json::json!({"phase":"select","hours":hours,"tier":tier,"progress":s.active.descent_progress()}));
            std::fs::write(dir.join(format!("{prefix}-tier{tier}-began.json")),s.save()).unwrap();
        }
        if s.active.descent_progress().tier==5 {owned5=true;break;}
        if hours==climb_hours {break;}
        let actions=prepare(&mut s,venom);let tier=s.active.descent_progress().tier;
        let report=s.run_offline_mode(28800,false,false);hours+=8;
        let row=serde_json::json!({"phase":"climb","seed":seed,"weapon":kind,"hours":hours,"tier":tier,"actions":actions,"runs":report.runs,"played_deepest":report.deepest,"deaths":report.deaths,"stalled":report.stalled,"level":s.active.lineage.class_level(),"cleared":s.active.lineage.ended,"progress":s.active.descent_progress()});
        println!("{row}");checks.push(row);
        if s.active.lineage.ended {std::fs::write(dir.join(format!("{prefix}-tier{tier}-cleared.json")),s.save()).unwrap();}
    }
    if !owned5 {std::fs::write(dir.join(format!("{prefix}-climb-failed.json")),s.save()).unwrap();return serde_json::json!({"seed":seed,"weapon":kind,"no_grants":true,"source_tier1_already_in_progress":true,"climb_hours":hours,"owned_tier5":false,"tier5_pass":false,"checks":checks});}
    assert!(s.active.descent_progress().cleared.is_some_and(|t|t>=4)&&s.active.descent_progress().unlocked>=5);
    let actions=prepare(&mut s,venom);
    std::fs::write(dir.join(format!("{prefix}-tier5-start.json")),s.save()).unwrap();
    let mut proof=s.clone();let snap=proof.try_send().unwrap();let gun=snap.hero.gun.as_ref().unwrap();assert_eq!(gun.kind,kind);
    let first=proof.step(10000);let shots=first.events.iter().filter(|e|matches!(e,riddle_core::Ev::Attack{verb:Some(v),..} if matches!(v.as_str(),"fire"|"aimed_shot"|"close_burst"|"finishing_shot"))).count();
    let reloads=first.events.iter().filter(|e|matches!(e,riddle_core::Ev::Callout{text,..} if text=="loaded")).count();
    std::fs::write(dir.join(format!("{prefix}-tier5-first-send.json")),serde_json::to_string(&snap).unwrap()).unwrap();
    std::fs::write(dir.join(format!("{prefix}-tier5-first-step.json")),serde_json::to_string(&first).unwrap()).unwrap();
    std::fs::write(dir.join(format!("{prefix}-tier5-first-after.json")),proof.save()).unwrap();
    assert!(shots>0&&reloads>0,"actual Tier5 firing/reload sample, evidence saved");
    let mut clear_hours=None;
    for n in 1..=6 {
        let report=s.run_offline_mode(28800,false,false);
        let row=serde_json::json!({"phase":"fixed-tier5","seed":seed,"weapon":kind,"hours":n*8,"runs":report.runs,"played_deepest":report.deepest,"deaths":report.deaths,"stalled":report.stalled,"cleared":s.active.lineage.ended,"progress":s.active.descent_progress()});
        println!("{row}");checks.push(row);if s.active.lineage.ended {clear_hours=Some(n*8);break;}
    }
    std::fs::write(dir.join(format!("{prefix}-tier5-after.json")),s.save()).unwrap();
    serde_json::json!({"seed":seed,"weapon":kind,"no_grants":true,"source_tier1_already_in_progress":true,"climb_hours":hours,"owned_tier5":true,"tier5_preparation":actions,"tier5_fixed_build":true,"tier5_pass":clear_hours.is_some(),"tier5_clear_hours":clear_hours,"first_proof_ticks":10000,"first_shots":shots,"first_reload_completions":reloads,"checks":checks})
}
fn main(){
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();let source=Path::new(args.get(1).expect("EARNED_DIR"));let dir=Path::new(args.get(2).expect("NEW_OUT_DIR"));std::fs::create_dir(dir).expect("new output directory preserves failures");
    let selected=args.iter().position(|a|a=="--seed").map(|i|args[i+1].parse::<u64>().expect("seed"));assert!(selected.is_none_or(|s|[1,3,5].contains(&s)));
    let gun=args.iter().position(|a|a=="--gun").map(|i|args[i+1].as_str());assert!(gun.is_none_or(|g|["long_gun","short_gun"].contains(&g)));
    let hours=args.iter().position(|a|a=="--climb-hours").map_or(336,|i|args[i+1].parse::<u32>().unwrap());assert!((8..=672).contains(&hours)&&hours.is_multiple_of(8));
    let venom=args.iter().any(|a|a=="--venom");let mut results=Vec::new();
    for seed in [1,3,5].into_iter().filter(|s|selected.is_none_or(|n|n==*s)) {for kind in ["long_gun","short_gun"].into_iter().filter(|g|gun.is_none_or(|k|k==*g)){results.push(play(source,dir,seed,kind,hours,venom));}}
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"diagnostic":true,"source":source,"no_grants":true,"climb_budget_hours":hours,"venom":venom,"results":results})).unwrap()).unwrap();
}
