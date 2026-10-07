//! Paid class pacing from existing earned camps; diagnostic, never a balance gate.
//! class_campaign EARNED_CAMP_DIR NEW_OUT_DIR [--seed 1|3|5] [--class fighter|rogue|ranger|caster] [--tactic OWNED_ID]
use riddle_core::{bloodlines::Session, hero::Class};
#[path="gunner_support/mod.rs"]
mod support;
use std::path::Path;
fn main() {
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();
    let from=Path::new(args.get(1).expect("earned camp directory"));
    let out=Path::new(args.get(2).expect("new output directory"));
    std::fs::create_dir(out).expect("new directory preserves previous evidence");
    let selected=args.iter().position(|a|a=="--seed").map(|i|args[i+1].parse::<u64>().expect("integer seed"));
    assert!(selected.is_none_or(|s|[1,3,5].contains(&s)));
    let chosen=args.iter().position(|a|a=="--class").map(|i|Class::parse(&args[i+1]).expect("known class"));
    assert!(chosen!=Some(Class::Gunner),"use gunner_campaign for firearm choices");
    let tactic=args.iter().position(|a|a=="--tactic").map(|i|args[i+1].as_str());
    assert!(tactic.is_none_or(|id|riddle_core::packages::def(id).is_some_and(|p|p.kind==riddle_core::packages::Kind::Tactic)));
    let mut results=vec![];
    for seed in [1,3,5].into_iter().filter(|s|selected.is_none_or(|n|*s==n)) {
        let source=std::fs::read_to_string(from.join(format!("seed{seed}-camp.json"))).unwrap();
        let original=Session::load(&source).unwrap();
        assert_eq!(original.active.lineage.seed,seed);
        assert!(original.active.run.is_none()&&original.others.is_empty()&&!original.active.lineage.ended);
        std::fs::write(out.join(format!("seed{seed}-source.json")),&source).unwrap();
        for class in [Class::Fighter,Class::Rogue,Class::Ranger,Class::Caster].into_iter().filter(|c|chosen.is_none_or(|n|*c==n)) {
            let mut s=original.clone();
            let mut unlock_checks=vec![];
            if let Some(id)=class.unlock() {
                for n in 0..=21 {
                    if s.active.lineage.unlocks.contains(id){break;}
                    match s.buy(id) {
                        Ok(_)=>break,
                        Err(reason)=>{
                            unlock_checks.push(serde_json::json!({"hours":n*8,"blocked":reason,"best":s.active.lineage.best_depth}));
                            if n==21||s.active.lineage.ended{break;}
                            support::camp(&mut s,true);
                            s.run_offline_mode(28800,false,false);
                        }
                    }
                }
                if !s.active.lineage.unlocks.contains(id) {
                    results.push(serde_json::json!({"seed":seed,"class":class.name(),"unlock_unavailable":true,"unlock_checks":unlock_checks}));
                    continue;
                }
            }
            s.set_class(class.name()).expect("choose unlocked class at home");
            if let Some(id)=tactic {s.equip_package(id,0).expect("explicitly choose owned tactic");}
            // Start with the chosen class's real weapon, not an inherited Fighter sword.
            let loadout=s.active.loadout.iter().copied().filter(|id|!s.active.lineage.vault.iter().any(|i|i.id==*id&&i.cat()==riddle_core::defs::Cat::Weapon)).collect();
            s.loadout(loadout);
            let prefix=format!("seed{seed}-{}",class.name());
            std::fs::write(out.join(format!("{prefix}-start.json")),s.save()).unwrap();
            let mut d13=None;let mut clear=None;let mut mastery=None;let mut checks=vec![];
            for n in 1..=21 {
                let mut actions=vec![];
                if s.active.lineage.ended {
                    let tier=s.active.lineage.endgame.as_ref().map_or(1,|p|p.tier.max(1));
                    s.begin_descent(tier).expect("explicit repeat of owned descent");
                    actions.push(format!("repeat descent {tier}"));
                }
                actions.extend(support::camp(&mut s,true));
                let r=s.run_offline_mode(28800,false,false);
                if r.deepest>=13&&d13.is_none(){d13=Some(n*8);}
                if s.active.lineage.ended&&clear.is_none(){clear=Some(n*8);}
                if s.active.lineage.class_level()>=10&&mastery.is_none(){mastery=Some(n*8);}
                let b=s.active.lineage.bloodline.as_ref().unwrap();
                let row=serde_json::json!({"hours":n*8,"runs":r.runs,"played_depth":r.deepest,"deaths":r.deaths,"stalled":r.stalled,"best":s.active.lineage.best_depth,"level":s.active.lineage.class_level(),"legacy_points":b.points,"legacy_spent":b.spent,"actions":actions,"cleared":s.active.lineage.ended});
                println!("seed{seed} {} {row}",class.name());checks.push(row);
                if clear.is_some()&&mastery.is_some(){break;}
            }
            std::fs::write(out.join(format!("{prefix}-after.json")),s.save()).unwrap();
            let result=serde_json::json!({"seed":seed,"class":class.name(),"explicit_tactic":tactic,"unlock_checks":unlock_checks,"inherited_record":original.active.lineage.best_depth,"played_d13_hours":d13,"first_clear_hours":clear,"mastery_hours":mastery,"checks":checks});
            std::fs::write(out.join(format!("{prefix}-result.json")),serde_json::to_string_pretty(&result).unwrap()).unwrap();
            results.push(result);
        }
        assert_eq!(std::fs::read_to_string(from.join(format!("seed{seed}-camp.json"))).unwrap(),source);
    }
    std::fs::write(out.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"diagnostic":true,"no_grants":true,"checkin_hours":8,"max_hours":168,"results":results})).unwrap()).unwrap();
}
