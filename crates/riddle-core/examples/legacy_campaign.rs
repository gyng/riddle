//! Earned post-clear Legacy comparisons; diagnostic, not a progression gate.
//! legacy_campaign EARNED_CLEAR_SAVE NEW_OUT_DIR [--build inherited|LEAF-LEAF-LEAF] [--tactic OWNED_ID] [--stance OWNED_ID] [--temper OWNED_ID]
use riddle_core::{bloodlines::Session,legacy};
#[path="gunner_support/mod.rs"] mod support;
use std::{path::Path,time::Instant};
fn main(){
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();
    let source=args.get(1).expect("earned clear save");
    let out=Path::new(args.get(2).expect("new output directory"));
    let filter=|name:&str|args.iter().position(|a|a==name).map(|i|args.get(i+1).expect("filter value").as_str());
    let build=filter("--build");let tactic=filter("--tactic");let stance=filter("--stance");let temper=filter("--temper");
    assert!(tactic.is_none_or(|id|riddle_core::packages::def(id).is_some_and(|p|p.kind==riddle_core::packages::Kind::Tactic)),"unknown tactic");
    for (id,kind) in [(stance,riddle_core::packages::Kind::Stance),(temper,riddle_core::packages::Kind::Temperament)] {
        assert!(id.is_none_or(|id|riddle_core::packages::def(id).is_some_and(|p|p.kind==kind)),"wrong package kind");
    }
    let input=Session::load(&std::fs::read_to_string(source).expect("earned source")).expect("valid source");
    let cases=[(format!("{}{}",input.active.lineage.class.name(),input.active.lineage.seed),source.as_str())];
    let mut builds=vec![("inherited".to_owned(),None)];
    for recovery in ["mending","renewal"] {for control in ["venom","debilitate"] {for ward in ["fireward","brace"] {
        builds.push((format!("{recovery}-{control}-{ward}"),Some([recovery,control,ward])));
    }}}
    assert!(build.is_none_or(|id|builds.iter().any(|(name,_)|name==id)),"unknown build");
    std::fs::create_dir(out).expect("new directory preserves previous evidence");let mut results=vec![];
    for (case,source_path) in cases {
        let raw=std::fs::read_to_string(source_path).expect("earned source save");
        let mut prepared=Session::load(&raw).expect("valid earned source");
        assert!(prepared.others.is_empty()&&prepared.active.run.is_none()&&prepared.active.lineage.ended);
        let tier=prepared.active.descent_progress().unlocked;assert_eq!(tier,1,"these sources prove the first clear");
        let mut common_actions=support::camp(&mut prepared,false);
        for id in [stance,temper,tactic].into_iter().flatten() {prepared.equip_package(id,0).expect("choose actually owned package");common_actions.push(format!("equip {id}"));}
        std::fs::write(out.join(format!("{case}-source.json")),&raw).unwrap();
        std::fs::write(out.join(format!("{case}-prepared.json")),prepared.save()).unwrap();
        for (name,leaves) in builds.iter().filter(|(name,_)|build.is_none_or(|id|name==id)) {
            let mut s=prepared.clone();let gold=s.active.lineage.gold;
            let points_before=legacy::current(&s.active.lineage).unwrap().points;
            let mut actions=common_actions.clone();
            if let Some(leaves)=leaves {
                s.respec_legacy().expect("public free home refund");actions.push("respec Legacy".into());
                for id in legacy::IDS {for _ in 0..legacy::CAP {s.upgrade_hero(id).expect("paid roots");actions.push(format!("Legacy {id}"));}}
                for id in ["restoration","control","clear_lungs"].into_iter().chain(leaves.iter().copied()) {
                    s.upgrade_hero(id).expect("paid legal branch");actions.push(format!("Legacy {id}"));
                }
                assert_eq!(legacy::current(&s.active.lineage).unwrap().spent,216);
            }
            assert_eq!(s.active.lineage.gold,gold,"Legacy cannot spend or grant gold");
            s.begin_descent(tier).expect("explicit earned harder descent");actions.push(format!("begin descent {tier}"));
            let prefix=format!("{case}-{name}");std::fs::write(out.join(format!("{prefix}-start.json")),s.save()).unwrap();
            let timer=Instant::now();let mut checks=vec![];let mut clear=None;let mut total_runs=0;
            for n in 1..=6 {
                let upkeep=support::camp(&mut s,false);let r=s.run_offline_mode(28800,false,false);total_runs+=r.runs;
                let owned=legacy::current(&s.active.lineage).unwrap();
                let row=serde_json::json!({"hours":8*n,"played_depth":r.deepest,"record":s.active.lineage.best_depth,"runs":r.runs,"deaths":r.deaths,"stalled":r.stalled,"gold_home":r.gold,"meters":r.meters,"legacy_earned":r.legacy_earned,"legacy_points":owned.points,"legacy_spent":owned.spent,"upkeep":upkeep,"cleared":s.active.lineage.ended});
                println!("{case} {name} {row}");checks.push(row);
                if s.active.lineage.ended {clear=Some(8*n);break;}
            }
            std::fs::write(out.join(format!("{prefix}-after.json")),s.save()).unwrap();
            let result=serde_json::json!({"case":case,"build":name,"source":source_path,"explicit_tactic":tactic,"explicit_stance":stance,"explicit_temper":temper,"tier":tier,"actions":actions,"points_before":points_before,"first_clear_hours":clear,"runs":total_runs,"seconds":timer.elapsed().as_secs_f64(),"checks":checks});
            std::fs::write(out.join(format!("{prefix}-result.json")),serde_json::to_string_pretty(&result).unwrap()).unwrap();results.push(result);
        }
        assert_eq!(std::fs::read_to_string(source_path).unwrap(),raw,"source must stay unchanged");
    }
    std::fs::write(out.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"diagnostic":true,"no_grants":true,"checkin_hours":8,"max_hours":48,"results":results})).unwrap()).unwrap();
}
