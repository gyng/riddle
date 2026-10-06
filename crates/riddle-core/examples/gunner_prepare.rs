//! Earn Gunner progression from an existing camp, with paid player actions only.
//! cargo run -q --profile fast -p riddle-core --example gunner_prepare -- SAVE NEW_OUT_DIR
use riddle_core::bloodlines::Session;
use std::path::Path;
fn main() {
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let args:Vec<_>=std::env::args().collect();
    let source=args.get(1).expect("SAVE");
    let dir=Path::new(args.get(2).expect("NEW_OUT_DIR"));
    std::fs::create_dir(dir).expect("new output directory");
    let original=Session::load(&std::fs::read_to_string(source).unwrap()).unwrap();
    assert!(original.active.run.is_none()&&original.others.is_empty());
    assert!(!original.active.lineage.classes.contains_key("gunner"));
    let mut results=vec![];
    for kind in ["long_gun","short_gun"] {
        let mut s=original.clone();
        s.buy("gunner").expect("legal historical depth/earned marks unlock");
        s.set_class("gunner").expect("select unlocked class at home");
        if kind=="short_gun" {riddle_core::kit::buy(&mut s.active,kind).expect("pay existing gold for short gun");}
        std::fs::write(dir.join(format!("{kind}-start.json")),s.save()).unwrap();
        let mut checks=vec![];let mut d13=None;let mut mastery=None;
        for n in 1..=21 {
            if s.active.lineage.ended {
                let tier=s.active.lineage.endgame.as_ref().map_or(1,|p|p.tier.max(1));
                s.begin_descent(tier).expect("legally repeat owned descent");
            }
            let report=s.run_offline_mode(28800,false,true);
            if d13.is_none()&&s.active.lineage.best_depth>=13 {d13=Some(n*8);}
            if mastery.is_none()&&s.active.lineage.class_level()>=10 {mastery=Some(n*8);}
            let row=serde_json::json!({"weapon":kind,"hours":n*8,"runs":report.runs,
                "level":s.active.lineage.class_level(),"xp":s.active.lineage.classes["gunner"].xp,
                "best":s.active.lineage.best_depth,"cleared":s.active.lineage.ended});
            println!("{row}");checks.push(row);
            if n>=6&&mastery.is_some() {break;}
        }
        std::fs::write(dir.join(format!("{kind}-after.json")),s.save()).unwrap();
        results.push(serde_json::json!({"weapon":kind,"no_grants":true,
            "authored_policy_preserved":true,"d13_hours":d13,"mastery_hours":mastery,"checks":checks}));
    }
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({
        "source":source,"scope":"one earned source camp, not three-seed acceptance",
        "results":results})).unwrap()).unwrap();
}
