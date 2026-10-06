//! Actually earn normal Gunner unlock camps, then measure played depth (not inherited records).
//! gunner_campaign NEW_OUT_DIR [--unlock-only] [--from EARNED_CAMP_DIR] [--upkeep] [--hours 48|168] [--seed 1|3|5] [--gun long_gun|short_gun] [--tactic OWNED_ID] [--temperament OWNED_ID|none]
//! Public paid player actions only. No seed/XP/currency/depth/kit/fact grants or pen edits.
use riddle_core::{bloodlines::Session,kit,legacy,tree,town};
use std::path::Path;
fn camp(s:&mut Session,branches:bool)->Vec<String> {
    assert!(s.active.run.is_none(),"player choices require home");
    let mut actions=vec![];
    for (id,_) in town::BUILDINGS {
        if !town::built(&s.active.lineage,id)&&s.build_town(id).is_ok(){actions.push(format!("build {id}"));}
    }
    for _ in 0..tree::NODES.len() {
        let Some(n)=tree::lit(&s.active.lineage) else {break};
        if s.hire(n.id).is_err(){break;}actions.push(format!("hire {}",n.id));
    }
    let ids=if branches {vec!["health","damage","armour","restoration","clear_lungs","renewal","brace"]}
        else {legacy::IDS.to_vec()};
    for id in ids {
        while legacy::offers(&s.active.lineage,false).iter().any(|o|o.id==id&&o.affordable) {
            s.upgrade_hero(id).expect("pay earned offered Legacy");actions.push(format!("Legacy {id}"));
        }
    }
    // Preserve the same three-unit supplies reserve as the daily-player harness.
    loop {
        let next=kit::ladders(&s.active.lineage).into_iter().filter_map(|l|l.next.map(|n|(l.slot,n.price))).min_by_key(|n|n.1);
        let Some((id,price))=next else {break};
        let reserve=3*kit::unit_of(&s.active.lineage);
        if i64::from(s.active.lineage.gold)<i64::from(price)+i64::from(reserve){break;}
        kit::buy(&mut s.active,&id).expect("pay offered forge price");actions.push(format!("forge {id}"));
    }
    actions
}
fn earn(seed:u64,dir:&Path)->Option<Session> {
    let mut s=Session::new(seed);s.build_town("house").expect("manual free opening house");
    let mut checks=vec![];
    // First sends stay manual until a genuinely affordable scout is hired.
    for n in 1..=30 {
        let actions=camp(&mut s,false);
        if tree::auto_send(&s.active.lineage){break;}
        s.try_send().expect("legal manual first send");
        let report=s.run_offline_mode(1,false,false);
        checks.push(serde_json::json!({"phase":"manual","send":n,"actions":actions,"runs":report.runs,"deepest":report.deepest,"best":s.active.lineage.best_depth}));
    }
    for n in 0..=42 {
        if s.active.lineage.best_depth>=13&&s.active.lineage.marks>=8&&s.active.run.is_none(){
            assert!(!s.active.lineage.ended&&s.active.lineage.endgame.as_ref().is_none_or(|p|p.tier==0));
            std::fs::write(dir.join(format!("seed{seed}-camp.json")),s.save()).unwrap();
            std::fs::write(dir.join(format!("seed{seed}-earning.json")),serde_json::to_string_pretty(&serde_json::json!({"seed":seed,"new_town":true,"manual_house":true,"no_grants":true,"offline_checkin_hours":n*8,"checks":checks})).unwrap()).unwrap();
            println!("{}",serde_json::json!({"phase":"unlock-camp","seed":seed,"hours":n*8,"best":s.active.lineage.best_depth,"marks":s.active.lineage.marks,"level":s.active.lineage.class_level(),"kit":s.active.lineage.kit}));
            return Some(s);
        }
        if n==42||s.active.lineage.ended{break;}
        let actions=camp(&mut s,false);
        if !tree::auto_send(&s.active.lineage){s.try_send().expect("manual send while scout unavailable");}
        let report=s.run_offline_mode(28800,false,false);
        checks.push(serde_json::json!({"phase":"earning","hours":(n+1)*8,"actions":actions,"runs":report.runs,"deepest":report.deepest,"best":s.active.lineage.best_depth}));
    }
    std::fs::write(dir.join(format!("seed{seed}-unlock-failed.json")),s.save()).unwrap();
    std::fs::write(dir.join(format!("seed{seed}-earning.json")),serde_json::to_string_pretty(&checks).unwrap()).unwrap();
    None
}
fn play(original:&Session,kind:&str,dir:&Path,upkeep:bool,hours:u32,tactic:Option<&str>,temperament:Option<&str>)->serde_json::Value {
    let seed=original.active.lineage.seed;
    let mut s=original.clone();s.buy("gunner").expect("earned D13/eight marks unlock");s.set_class("gunner").unwrap();
    if kind=="short_gun" {kit::buy(&mut s.active,kind).expect("existing gold pays one short-gun unit");}
    let ids=s.active.loadout.iter().copied().filter(|id|!s.active.lineage.vault.iter().any(|i|i.id==*id&&i.cat()==riddle_core::defs::Cat::Weapon)).collect();
    s.loadout(ids);
    if let Some(id)=tactic {s.equip_package(id,0).expect("explicit owned tactic choice at home");}
    if let Some(id)=temperament {
        if id=="none" {if let Some(worn)=s.active.lineage.pkg.temperament.clone() {s.unequip_package(&worn).expect("explicitly remove the worn temperament");}}
        else {s.equip_package(id,0).expect("explicit owned temperament choice at home");}
    }
    let prefix=format!("seed{seed}-{kind}");
    std::fs::write(dir.join(format!("{prefix}-start.json")),s.save()).unwrap();
    let mut proof=s.clone();let sent=proof.try_send().unwrap();let gun=sent.hero.gun.as_ref().expect("actual gun at send");assert_eq!(gun.kind,kind);
    std::fs::write(dir.join(format!("{prefix}-first-send.json")),serde_json::to_string(&sent).unwrap()).unwrap();
    let first=proof.step(10000);
    let shots=first.events.iter().filter(|e|matches!(e,riddle_core::Ev::Attack{verb:Some(v),..} if v=="fire"||v=="aimed_shot"||v=="close_burst")).count();
    let reloads=first.events.iter().filter(|e|matches!(e,riddle_core::Ev::Callout{text,..} if text=="loaded")).count();
    std::fs::write(dir.join(format!("{prefix}-first-step.json")),serde_json::to_string(&first).unwrap()).unwrap();
    std::fs::write(dir.join(format!("{prefix}-first-after.json")),proof.save()).unwrap();
    assert!(shots>0&&reloads>0,"first 10000-tick sample must actually shoot/reload; evidence preserved");
    let mut d13=None;let mut mastery=None;let mut deepest=0;let mut checks=vec![];
    for n in 1..=hours/8 {
        let mut actions=vec![];
        if s.active.lineage.ended {
            let tier=s.active.lineage.endgame.as_ref().map_or(1,|p|p.tier.max(1));
            s.begin_descent(tier).expect("explicitly repeat an owned descent after a clear");
            actions.push(format!("begin descent {tier}"));
        }
        if upkeep {actions.extend(camp(&mut s,true));}
        let report=s.run_offline_mode(28800,false,false);
        deepest=deepest.max(report.deepest);
        if d13.is_none()&&report.deepest>=13 {d13=Some(n*8);}
        if mastery.is_none()&&s.active.lineage.class_level()>=10 {mastery=Some(n*8);}
        let row=serde_json::json!({"seed":seed,"weapon":kind,"hours":n*8,"runs":report.runs,"played_deepest":report.deepest,"inherited_record":original.active.lineage.best_depth,
            "best":s.active.lineage.best_depth,"level":s.active.lineage.class_level(),"xp":s.active.lineage.classes["gunner"].xp,"actions":actions,
            "deaths":report.deaths,"stalled":report.stalled,"cleared":s.active.lineage.ended});
        println!("{row}");checks.push(row);
        if n>=6&&mastery.is_some(){break;}
    }
    std::fs::write(dir.join(format!("{prefix}-after.json")),s.save()).unwrap();
    serde_json::json!({"seed":seed,"weapon":kind,"no_grants":true,"upkeep":upkeep,"explicit_tactic":tactic,"explicit_temperament":temperament,"horizon_hours":hours,"mastery_complete_horizon":hours>=168||mastery.is_some(),"first_proof_ticks":10000,"first_send_shots":shots,"first_send_reload_completions":reloads,
        "d13_played_hours":d13,"d13_pass":d13.is_some_and(|h|h<=48),"mastery_hours":mastery,"mastery_pass":mastery.is_some_and(|h|h<=168),"played_deepest":deepest,"checks":checks})
}
fn main() {
    riddle_core::chronicle::use_shipping_words();riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();let dir=Path::new(args.get(1).expect("NEW_OUT_DIR"));std::fs::create_dir(dir).expect("new directory preserves failures");
    let from=args.iter().position(|a|a=="--from").map(|i|Path::new(args.get(i+1).expect("earned camp directory")));
    let hours=args.iter().position(|a|a=="--hours").map_or(168,|i|args.get(i+1).expect("hours").parse::<u32>().expect("integer hours"));
    assert!((48..=336).contains(&hours)&&hours.is_multiple_of(8),"48–336h in eight-hour check-ins");
    let selected_seed=args.iter().position(|a|a=="--seed").map(|i|args.get(i+1).expect("seed").parse::<u64>().expect("integer seed"));
    assert!(selected_seed.is_none_or(|s|[1,3,5].contains(&s)),"normal accepted seeds1/3/5");
    let selected_gun=args.iter().position(|a|a=="--gun").map(|i|args.get(i+1).expect("gun").as_str());
    assert!(selected_gun.is_none_or(|g|["long_gun","short_gun"].contains(&g)),"known gun choice");
    let tactic=args.iter().position(|a|a=="--tactic").map(|i|args.get(i+1).expect("owned tactic id").as_str());
    assert!(tactic.is_none_or(|id|riddle_core::packages::def(id).is_some_and(|p|p.kind==riddle_core::packages::Kind::Tactic)),"a named tactic, not a stance or temperament");
    let temperament=args.iter().position(|a|a=="--temperament").map(|i|args.get(i+1).expect("owned temperament id or none").as_str());
    assert!(temperament.is_none_or(|id|id=="none"||riddle_core::packages::def(id).is_some_and(|p|p.kind==riddle_core::packages::Kind::Temperament)),"a named temperament or none");
    let seeds:Vec<_>=[1,3,5].into_iter().filter(|s|selected_seed.is_none_or(|selected|selected==*s)).collect();
    let mut results=vec![];let mut unlock_failed=vec![];
    for &seed in &seeds {
        let original=if let Some(from)=from {Some(Session::load(&std::fs::read_to_string(from.join(format!("seed{seed}-camp.json"))).unwrap()).unwrap())}else{earn(seed,dir)};
        let Some(original)=original else {unlock_failed.push(seed);continue};
        assert_eq!(original.active.lineage.seed,seed);assert!(original.active.run.is_none()&&original.others.is_empty());
        assert!(!original.active.lineage.ended&&original.active.lineage.endgame.as_ref().is_none_or(|p|p.tier==0));
        assert!(!original.active.lineage.classes.contains_key("gunner"));
        if !args.iter().any(|a|a=="--unlock-only") {for kind in ["long_gun","short_gun"].into_iter().filter(|g|selected_gun.is_none_or(|selected|selected==*g)) {results.push(play(&original,kind,dir,args.iter().any(|a|a=="--upkeep"),hours,tactic,temperament));}}
    }
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"diagnostic":true,"normal_seeds":seeds,"selected_gun":selected_gun,"explicit_tactic":tactic,"explicit_temperament":temperament,"targeted":selected_seed.is_some()||selected_gun.is_some(),"no_grants":true,"from":from,"unlock_failed":unlock_failed,"results":results})).unwrap()).unwrap();
}
