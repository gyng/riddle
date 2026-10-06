//! Earn Caster XP by normal runs from a real save; paid existing unlocks/upgrades only.
//! cargo run -q --profile fast -p riddle-core --example style_prepare -- SAVE NEW_OUT_DIR [--recovery-warding] [--boss-counters] [--reflection-counter] [--style-defaults]
use riddle_core::{bloodlines::Session,hero::Class,specialization::Style,legacy,Row,Cond,Verb};
use std::{path::Path,collections::BTreeMap};
fn write(dir:&Path,name:&str,s:&Session){std::fs::write(dir.join(name),s.save()).unwrap();}
fn main(){
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();let source=args.get(1).expect("SAVE");let dir=Path::new(args.get(2).expect("NEW_OUT_DIR"));
    std::fs::create_dir(dir).expect("new output directory");
    let mut prepared=Session::load(&std::fs::read_to_string(source).unwrap()).unwrap();
    assert!(prepared.active.run.is_none()&&prepared.others.is_empty()&&!prepared.active.lineage.ended);
    for id in legacy::IDS {
        while legacy::current(&prepared.active.lineage).unwrap().upgrades.get(id).copied().unwrap_or(0)<legacy::CAP {
            prepared.upgrade_hero(id).expect("existing earned Legacy buys roots");
        }
    }
    let recovery_warding=args.iter().any(|a|a=="--recovery-warding");
    if recovery_warding {
        for id in ["restoration","mending","clear_lungs","fireward"] {prepared.upgrade_hero(id).expect("pay owned Legacy for an explicit recovery/warding build");}
    }
    let boss_counters=args.iter().any(|a|a=="--boss-counters");
    if boss_counters {prepared.equip_package("boss_focus",0).expect("equip already-owned boss tactic");}
    let mut sentinel=prepared.clone();sentinel.set_specialization("sentinel").expect("earned Fighter10/D23");
    write(dir,"sentinel-start.json",&sentinel);
    let mut caster=prepared;
    if !caster.active.lineage.unlocks.contains("caster") {caster.buy("caster").expect("pay existing marks for Caster");}
    caster.set_class("caster").unwrap();
    // Explicit owned pen choice for an actual Caster build, no XP/verb grant.
    assert!(caster.active.lineage.pkg.pen_open);
    let reflection_counter=args.iter().any(|a|a=="--reflection-counter");
    if reflection_counter {
        caster.active.lineage.pkg.pen.push(Row{conds:vec![Cond::t("foe_tag","reflect_melee"),Cond::n("hp>",45)],verb:Verb::arg("bolt","tag:reflect_melee"),origin:Some("player".into())});
    }
    if boss_counters {
        caster.active.lineage.pkg.pen.push(Row{conds:vec![Cond::t("foe_tag","boss"),Cond::n("hp>",45)],verb:Verb::arg("tactic","boss_focus"),origin:Some("player".into())});
    }
    caster.active.lineage.pkg.pen.push(Row{conds:vec![Cond::n("foes>=",1),Cond::n("hp>",45)],verb:Verb::arg("bolt","nearest"),origin:Some("player".into())});
    riddle_core::packages::recompile(&mut caster.active.lineage);
    let rules=caster.active.lineage.rules().clone();caster.set_rules(rules).expect("training policy uses only owned vocabulary and row capacity");
    write(dir,"caster-training-start.json",&caster);let mut checks=Vec::new();
    for n in 1..=42 {
        if caster.active.lineage.ended {caster.begin_descent(1).expect("legally replay an unlocked descent");}
        let report=caster.run_offline_mode(28800,false,true);
        let row=serde_json::json!({"checkin":n,"hours":8*n,"runs":report.runs,"class_level":caster.active.lineage.class_level(),"class_xp":caster.active.lineage.classes["caster"].xp,"best":caster.active.lineage.best_depth,"cleared":caster.active.lineage.ended});
        println!("{row}");checks.push(row);
        if caster.active.lineage.class_level()>=10 {break;}
    }
    write(dir,"caster-training-after.json",&caster);
    let qualified=caster.active.lineage.class_level()>=10;
    std::fs::write(dir.join("training.json"),serde_json::to_string_pretty(&serde_json::json!({"source":source,"no_xp_or_currency_grants":true,"recovery_warding":recovery_warding,"boss_counters":boss_counters,"reflection_counter":reflection_counter,"qualified":qualified,"checks":checks})).unwrap()).unwrap();
    if !qualified {std::process::exit(1);}
    if caster.active.lineage.ended {caster.begin_descent(1).unwrap();}
    // Explicit post-training policy choice: retain learned boss counters, let the
    // selected automatic style handle ordinary encounters instead of the training override.
    let style_defaults=args.iter().any(|a|a=="--style-defaults");
    if style_defaults {
        caster.active.lineage.pkg.pen.retain(|r|r.verb!=Verb::arg("bolt","nearest"));
        riddle_core::packages::recompile(&mut caster.active.lineage);
        let rules=caster.active.lineage.rules().clone();caster.set_rules(rules).unwrap();
    }
    assert_eq!(caster.active.lineage.class,Class::Caster);caster.set_specialization("hexbinder").unwrap();write(dir,"hexbinder-start.json",&caster);
    for (style,mut s) in [(Style::Sentinel,sentinel),(Style::Hexbinder,caster)] {
        s.active.tap=Some(BTreeMap::new());let report=s.run_offline_mode(28800,false,true);
        let mut verbs=0;let mut counters=0;
        for events in s.active.tap.as_ref().unwrap().values(){for event in events {
            if matches!(event,riddle_core::Ev::Rule{verb,..} if verb.v==style.verb()){verbs+=1;}
            if matches!(event,riddle_core::Ev::Attack{verb:Some(v),..} if v=="riposte"){counters+=1;}
        }}
        if let Some((id,events))=s.active.tap.as_ref().unwrap().iter().rev().find(|(_,events)|events.iter().any(|event|matches!(event,riddle_core::Ev::Rule{verb,..} if verb.v==style.verb()))) {
            std::fs::write(dir.join(format!("{}-earned-events.json",style.id())),serde_json::to_string(&serde_json::json!({"run_id":id,"style":style.id(),"events":events})).unwrap()).unwrap();
        }
        let row=serde_json::json!({"style":style.id(),"runs":report.runs,"exclusive_actions":verbs,"riposte_counters":counters,"best":s.active.lineage.best_depth,"cleared":s.active.lineage.ended});
        std::fs::write(dir.join(format!("{}-actions.json",style.id())),serde_json::to_string_pretty(&row).unwrap()).unwrap();println!("{row}");
        write(dir,&format!("{}-after.json",style.id()),&s);
    }
}
