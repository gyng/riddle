//! Compare owned Tier5 camps with one explicit paid backup purchase, never tier grants.
//! gunner_fixed EARNED_DIR NEW_OUT_DIR [--sidearm] [--seed 1|3|5] [--gun long_gun|short_gun]
use riddle_core::{bloodlines::Session,firearm,hero::Class,kit,Ev};
use std::path::Path;
fn main() {
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let a:Vec<_>=std::env::args().collect();let source=Path::new(a.get(1).expect("EARNED_DIR"));
    let dir=Path::new(a.get(2).expect("NEW_OUT_DIR"));std::fs::create_dir(dir).expect("preserve prior failures");
    let sidearm=a.iter().any(|x|x=="--sidearm");
    let selected=a.iter().position(|x|x=="--seed").map(|i|a[i+1].parse::<u64>().unwrap());assert!(selected.is_none_or(|s|[1,3,5].contains(&s)));
    let gun=a.iter().position(|x|x=="--gun").map(|i|a[i+1].as_str());assert!(gun.is_none_or(|g|["long_gun","short_gun"].contains(&g)));
    let mut results=vec![];
    for seed in [1,3,5].into_iter().filter(|s|selected.is_none_or(|n|n==*s)) {for kind in ["long_gun","short_gun"].into_iter().filter(|g|gun.is_none_or(|n|n==*g)) {
        let prefix=format!("seed{seed}-{kind}");
        let mut s=Session::load(&std::fs::read_to_string(source.join(format!("{prefix}-tier5-start.json"))).unwrap()).unwrap();
        assert!(s.others.is_empty()&&s.active.run.is_none());assert_eq!(s.active.lineage.seed,seed);assert_eq!(s.active.lineage.class,Class::Gunner);
        let p=s.active.descent_progress();assert_eq!(p.tier,5);assert!(p.unlocked>=5&&p.cleared.is_some_and(|t|t>=4));
        assert_eq!(firearm::starting_kind(&s.active.lineage),kind);
        std::fs::write(dir.join(format!("{prefix}-source.json")),s.save()).unwrap();
        let gold=s.active.lineage.gold;
        if sidearm {kit::buy(&mut s.active,firearm::SIDEARM_SLOT).expect("one explicit purchase with earned gold");}
        let paid=gold-s.active.lineage.gold;
        std::fs::write(dir.join(format!("{prefix}-start.json")),s.save()).unwrap();
        let mut proof=s.clone();let snap=proof.try_send().unwrap();assert_eq!(snap.hero.gun.as_ref().unwrap().kind,kind);
        if sidearm {assert!(proof.active.run.as_ref().unwrap().hero.inv.iter().any(|i|i.id==kit::SIDEARM_ID));}
        let step=proof.step(100000);
        let shots=step.events.iter().filter(|e|matches!(e,Ev::Attack{src:1,verb:Some(v),..} if matches!(v.as_str(),"fire"|"aimed_shot"|"close_burst"|"finishing_shot"))).count();
        let reloads=step.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count();
        let swaps=step.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="melee backup")).count();
        std::fs::write(dir.join(format!("{prefix}-first-send.json")),serde_json::to_string(&snap).unwrap()).unwrap();
        std::fs::write(dir.join(format!("{prefix}-first-step.json")),serde_json::to_string(&step).unwrap()).unwrap();
        std::fs::write(dir.join(format!("{prefix}-first-after.json")),proof.save()).unwrap();
        assert!(shots>0&&reloads>0,"actual gun firing/reloading proof retained");
        let mut checks=vec![];let mut clear=None;
        for n in 1..=6 {
            let report=s.run_offline_mode(28800,false,false);
            let row=serde_json::json!({"seed":seed,"gun":kind,"hours":n*8,"deepest":report.deepest,"runs":report.runs,"deaths":report.deaths,"stalled":report.stalled,"cleared":s.active.lineage.ended});
            println!("{row}");checks.push(row);if s.active.lineage.ended {clear=Some(n*8);break;}
        }
        std::fs::write(dir.join(format!("{prefix}-after.json")),s.save()).unwrap();
        results.push(serde_json::json!({"seed":seed,"gun":kind,"paid_backup":paid,"clear_hours":clear,"pass":clear.is_some(),"first_proof_budget_ticks":100000,"first_shots":shots,"first_reload_completions":reloads,"first_sidearm_swaps":swaps,"checks":checks}));
    }}
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"no_grants":true,"sidearm":sidearm,"source":source,"fixed_build":true,"results":results})).unwrap()).unwrap();
}
