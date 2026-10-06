//! Equal-work tick/status/damage benchmark compatible with pre-Leeching e2e74c7.
//! JSON fixture fields are ignored by pre-style serde; no generation/parsing is timed.
//! Hero resets every60 ticks in both builds to cover repeated reactive parries.
use riddle_core::{Game,hero::{Hero,Class},engine::HERO_ID,geom::Pos,turn::Src,wire::Ev};
use std::time::Instant;
fn arena(tier:u32,mode:&str)->(Game,Hero) {
    let mut g=Game::new_literal(7);g.sim=true;
    g.lineage.endgame=Some(riddle_core::endgame::Progress{tier,unlocked:tier.max(1),cleared:Some(tier.saturating_sub(1))});
    g.ensure_run();let r=g.run.as_mut().unwrap();r.monsters.clear();
    r.hero.hp=1000000;r.hero.max_hp=1000000;r.hero.max_hp_base=1000000;r.hero.energy= -1000000;
    r.hero.level=10;r.hero.armour=None.into();r.hero.inv.clear();r.hero.gift=Default::default();
    if mode=="hex" {r.hero.class=Class::Caster;}
    let adjacent=r.hero.pos.neighbours8().into_iter().find(|p|r.floor.map.passable(*p)).unwrap();
    for n in 0..32 {
        let pos=if n==0 {adjacent}else{Pos{x:3+(n as i32%8),y:3+n as i32/8}};
        let mut m=riddle_core::endgame::spawn(r,HERO_ID+1+n,"goblin",pos,1,true);
        m.hp=500000;m.max_hp=1000000;m.stun=1000000;m.awake=true;m.speed=10;
        // The old build rejects only the new enum variant; explicitly retain None
        // for its baseline. All other stats/workload are equal and parsed outside timing.
        if let Some(mods)=m.modifiers.as_mut(){mods.elite=None;}
        if mode=="leeching"&&n==0 {
            let mut v=serde_json::to_value(&m).unwrap();v["modifiers"]["elite"]=serde_json::json!("leeching");
            if let Ok(positive)=serde_json::from_value(v){m=positive;}
        }
        if mode=="hex"&&n==0 {
            let mut v=serde_json::to_value(&m).unwrap();v["hex_t"]=serde_json::json!(1000000);
            m=serde_json::from_value(v).unwrap();
        }
        r.monsters.push(m);
    }
    if matches!(mode,"riposte"|"hex") {
        let mut v=serde_json::to_value(&r.hero).unwrap();
        v["specialization"]=serde_json::json!(if mode=="riposte"{"sentinel"}else{"hexbinder"});
        v["special_cd"]=serde_json::json!(1000000);
        if mode=="riposte" {v["riposte_t"]=serde_json::json!(1000000);}
        r.hero=serde_json::from_value(v).unwrap();
    }
    let hero=r.hero.clone();g.events.clear();(g,hero)
}
fn main(){
    let event_bytes=std::mem::size_of::<Ev>();
    for (tier,mode) in [(0,"none"),(5,"none"),(6,"none"),(6,"leeching")] {
        let (g,hero)=arena(tier,mode);let mut samples=Vec::new();let mut observed=None;
        for _ in 0..101 {
            let mut h=g.clone();let at=Instant::now();
            for n in 0..500 {
                if n%60==0 {h.run.as_mut().unwrap().hero=hero.clone();}
                h.tick();let(r,mut cx)=h.ctx();riddle_core::turn::damage_hero(r,&mut cx,10,&Src::Mon(0));
            }
            samples.push(at.elapsed().as_nanos() as u64);
            let r=h.run.as_ref().unwrap();assert!(r.over.is_none());assert_eq!(r.monsters.len(),32);
            let counters=h.events.iter().filter(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="riposte")).count();
            let result=(r.hero.hp,r.monsters[0].hp,counters);
            if let Some(prev)=observed {assert_eq!(prev,result);}else{observed=Some(result);}
            std::hint::black_box(h);
        }
        samples.sort();let (hp,foe_hp,counters)=observed.unwrap();
        println!("{{\"event_bytes\":{event_bytes},\"tier\":{tier},\"mode\":\"{mode}\",\"ticks\":500,\"foes\":32,\"samples\":101,\"hero_resets\":9,\"median_ns\":{},\"ns_tick\":{},\"final_hp\":{hp},\"foe_hp\":{foe_hp},\"riposte_counters\":{counters}}}",samples[50],samples[50] as f64/500.0);
    }
}
