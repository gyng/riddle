//! Replay one actual earned camp without changing its build; retain full events.
//! gunner_trace SAVE NEW_OUT_DIR [RUNS]
use riddle_core::bloodlines::Session;
use std::collections::BTreeMap;
fn main() {
    riddle_core::chronicle::use_shipping_words();
    let args:Vec<_>=std::env::args().collect();
    let mut s=Session::load(&std::fs::read_to_string(args.get(1).expect("SAVE")).unwrap()).unwrap();
    let dir=std::path::Path::new(args.get(2).expect("NEW_OUT_DIR"));
    std::fs::create_dir(dir).expect("preserve earlier evidence");
    let runs=args.get(3).map_or(3,|s|s.parse::<u32>().unwrap());assert!((1..=12).contains(&runs));
    for n in 1..=runs {
        assert!(s.active.run.is_none());
        std::fs::write(dir.join(format!("run{n}-before.json")),s.save()).unwrap();
        let snap=s.try_send().expect("actual home send");
        std::fs::write(dir.join(format!("run{n}-send.json")),serde_json::to_string(&snap).unwrap()).unwrap();
        let mut events=vec![];let mut ended=false;
        for _ in 0..2000 {
            let step=s.step(100);events.extend(step.events);
            if step.run_over {ended=true;break;}
        }
        std::fs::write(dir.join(format!("run{n}-events.json")),serde_json::to_string(&events).unwrap()).unwrap();
        std::fs::write(dir.join(format!("run{n}-after.json")),s.save()).unwrap();
        let mut counts=BTreeMap::<String,u32>::new();
        for e in &events {
            let v=serde_json::to_value(e).unwrap();
            let key=format!("{}:{}",v.get("k").unwrap_or(&serde_json::Value::Null),v.get("verb").or_else(||v.get("text")).unwrap_or(&serde_json::Value::Null));
            *counts.entry(key).or_default()+=1;
        }
        println!("{}",serde_json::json!({"run":n,"ended":ended,"last_run":s.active.lineage.last_run,"events":events.len(),"counts":counts}));
        if !ended||s.active.pending_exit.is_some()||s.active.lineage.ended {break;}
    }
}
