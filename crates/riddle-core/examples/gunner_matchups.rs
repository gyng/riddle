//! Controlled paired gun/bow matchups, never earned progression or the routine gate.
//! gunner_matchups NEW_OUT_DIR [--seeds N] [--level 1|3|5|10]
use riddle_core::{Game,Ev,engine::HERO_ID,geom::Pos,hero::{Class,Hero},item::Item,monster::Monster,gen::Floor,tiles::{Map,Tile},rules::{Cond,Row,RuleSet,Verb}};
use std::path::Path;
fn fight(seed:u64,kind:&str,scenario:&str,level:u32,dir:&Path)->serde_json::Value {
    let class=if kind.starts_with("bow") {Class::Ranger}else{Class::Gunner};
    let mut g=Game::new_literal(seed);g.sim=true;g.lineage.class=class;
    g.lineage.supplies.clear();g.start_run(Some(seed.wrapping_mul(7)+3));
    g.lineage.unlocks.insert("lantern_rig".into());
    let r=g.run.as_mut().unwrap();let mut map=Map::new(24,14,Tile::Wall);
    for y in 1..13 {for x in 1..23 {map.set(Pos::new(x,y),Tile::Floor);}}
    let up=Pos::new(1,1);let down=Pos::new(22,12);
    map.set(up,Tile::StairsUp);map.set(down,Tile::StairsDown);map.compute_corridors(&[]);
    r.floor=Floor{map,stairs_up:up,stairs_down:down,rooms:Vec::new(),vision:7};
    r.hero=Hero::new(class,Pos::new(4,6));r.hero.apply_level(level);
    r.hero.weapon=Some(Item::new(900,if class==Class::Ranger {"bow"}else{kind})).into();r.monsters.clear();r.items.clear();r.overlays.clear();
    r.hero_dist_pos=None;r.last_visible=vec![u32::MAX];r.gun_reload=None;
    r.gun_skills=if class==Class::Gunner {Some(Default::default())}else{None};
    let foes:Vec<(&str,i32,i32,i32,i32)>=match scenario {
        "armoured_gap"=>vec![("iron_golem",12,6,36,6)],
        "close_pack"=>vec![("goblin",5,5,12,0),("goblin",5,6,12,0),("goblin",5,7,12,0)],
        "crossfire"=>vec![("goblin_archer",10,3,12,0),("goblin_archer",10,9,12,0)],
        _=>unreachable!(),
    };
    for (kind,x,y,hp,def) in foes {let id=r.new_id();let mut m=Monster::spawn(id,kind,Pos::new(x,y),1);m.hp=hp;m.max_hp=hp;m.def=def;m.awake=true;m.last_seen=Some(r.hero.pos);r.monsters.push(m);}
    r.floor.map.update_vision(r.hero.pos,r.vision(&g.lineage.unlocks));
    // Explicit fixed policies, each using real class-level actions. No card/stance tuning.
    let rows=if class==Class::Gunner {vec![Row::new(vec![],Verb::new("gunner_tactic"))]}else{
        let mut rows=Vec::new();
        if kind=="bow_kite" {rows.push(Row::new(vec![Cond::n("adj>=",1)],Verb::new("kite")));}
        rows.extend([
            Row::new(vec![Cond::n("foes>=",2)],Verb::new("volley")),
            Row::new(vec![Cond::n("foes>=",1)],Verb::new("double_shot")),
            Row::new(vec![Cond::n("foes>=",1)],Verb::arg("shoot","nearest")),
        ]);rows
    };
    g.set_rules_raw(RuleSet{rows,name:Some(format!("fixture:{kind}")),route:Vec::new()}).unwrap();g.events.clear();g.history.clear();
    if seed==1 {std::fs::write(dir.join(format!("{scenario}-{kind}-start.json")),g.save()).unwrap();}
    let mut clear=false;
    for _ in 0..1200 {
        g.tick();let r=g.run.as_ref().unwrap();
        if r.over.is_some()||r.depth!=1 {break;}
        if r.monsters.iter().all(|m|m.hp<=0) {clear=true;break;}
    }
    let r=g.run.as_ref().unwrap();
    let damage_taken:i32=g.events.iter().filter_map(|e|if let Ev::Hurt{id,dmg,..}=e {(*id==HERO_ID).then_some(*dmg)}else{None}).sum();
    let shots=g.events.iter().filter(|e|matches!(e,Ev::Attack{src,..} if *src==HERO_ID)).count();
    let reloads=g.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count();
    let aims=g.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="aim steady")).count();
    let row=serde_json::json!({"seed":seed,"weapon":kind,"scenario":scenario,"level":level,"clear":clear,"ticks":r.turn,"hp":r.hero.hp,"damage_taken":damage_taken,"shots":shots,"reload_completions":reloads,"aims":aims});
    if seed==1 {std::fs::write(dir.join(format!("{scenario}-{kind}-after.json")),g.save()).unwrap();std::fs::write(dir.join(format!("{scenario}-{kind}-events.json")),serde_json::to_string(&g.events).unwrap()).unwrap();}
    row
}
fn median(mut values:Vec<u64>)->Option<u64>{if values.is_empty(){return None;}values.sort_unstable();Some(values[values.len()/2])}
fn main(){
    riddle_core::chronicle::use_shipping_words();let args:Vec<_>=std::env::args().collect();
    let dir=Path::new(args.get(1).expect("NEW_OUT_DIR"));std::fs::create_dir(dir).expect("new directory preserves failures");
    let seeds=args.iter().position(|a|a=="--seeds").map_or(32,|i|args[i+1].parse::<u64>().expect("seed count"));assert!((1..=256).contains(&seeds));
    let level=args.iter().position(|a|a=="--level").map_or(3,|i|args[i+1].parse::<u32>().expect("level"));assert!([1,3,5,10].contains(&level));
    let mut samples=vec![];let mut summaries=vec![];
    for scenario in ["armoured_gap","close_pack","crossfire"] {for kind in ["long_gun","short_gun","bow","bow_kite"] {
        let rows:Vec<_>=(1..=seeds).map(|seed|fight(seed,kind,scenario,level,dir)).collect();
        let clear=rows.iter().filter(|r|r["clear"]==true).collect::<Vec<_>>();
        let summary=serde_json::json!({"scenario":scenario,"weapon":kind,"level":level,"clears":clear.len(),"seeds":seeds,
            "median_clear_ticks":median(clear.iter().map(|r|r["ticks"].as_u64().unwrap()).collect()),
            "median_clear_damage_taken":median(clear.iter().map(|r|r["damage_taken"].as_u64().unwrap()).collect()),
            "median_all_damage_taken":median(rows.iter().map(|r|r["damage_taken"].as_u64().unwrap()).collect())});
        println!("{summary}");summaries.push(summary);samples.extend(rows);
    }}
    // Compare paired seeds: clear beats failure, then faster clear, then less harm.
    // Never treat a fast death as a fast clear.
    let mut paired=Vec::new();
    for scenario in ["armoured_gap","close_pack","crossfire"] {
        for (a,b) in [("long_gun","short_gun"),("long_gun","bow"),("short_gun","bow"),("bow","bow_kite")] {
            let mut wins=0;let mut ties=0;let mut losses=0;
            for seed in 1..=seeds {
                let find=|kind:&str|samples.iter().find(|r|r["scenario"]==scenario&&r["weapon"]==kind&&r["seed"]==seed).unwrap();
                let rank=|r:&serde_json::Value|(!r["clear"].as_bool().unwrap(),if r["clear"]==true {r["ticks"].as_u64().unwrap()}else{1200},if r["clear"]==true {r["damage_taken"].as_u64().unwrap()}else{0});
                match rank(find(a)).cmp(&rank(find(b))) {
                    std::cmp::Ordering::Less=>wins+=1,
                    std::cmp::Ordering::Equal=>ties+=1,
                    std::cmp::Ordering::Greater=>losses+=1,
                }
            }
            paired.push(serde_json::json!({"scenario":scenario,"a":a,"b":b,"wins":wins,"ties":ties,"losses":losses}));
        }
    }
    std::fs::write(dir.join("results.json"),serde_json::to_string_pretty(&serde_json::json!({"diagnostic":true,"controlled_fixtures":true,"not_earned_campaign":true,"fixture":"open24x14; starting weapons; class-level HP/attack; no armour/heals/Legacy; equal lantern rig; specified foe HP/armour; unstunned enemies; timeout1200ticks; paired initial RNG; fixed actual policies: automatic guns; sustained bow; kite-first bow","paired_ranking":"clear first, then fewer clear ticks, then less total harm; all failures tie, never faster failure","paired":paired,"summaries":summaries,"samples":samples})).unwrap()).unwrap();
}
