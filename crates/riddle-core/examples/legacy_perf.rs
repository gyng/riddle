//! Equal-work tick/status and gas-damage microbenchmark, not complete catch-up timings.
use riddle_core::{Game,engine::HERO_ID,geom::Pos,turn::Src};
use std::time::Instant;
fn arena(tier:u32,perks:bool)->Game {
    let mut g=Game::new_literal(7);g.sim=true;
    g.lineage.endgame=Some(riddle_core::endgame::Progress{tier,unlocked:tier.max(1),cleared:Some(tier.saturating_sub(1))});
    if perks {
        riddle_core::legacy::ensure(&mut g.lineage);
        let b=g.lineage.bloodline.as_mut().unwrap();
        for id in ["restoration","mending","control","venom","clear_lungs","brace"] {b.upgrades.insert(id.into(),1);}
    }
    g.ensure_run();let r=g.run.as_mut().unwrap();r.monsters.clear();
    r.hero.hp=1000000;r.hero.max_hp=1000000;r.hero.energy= -1000000;
    for n in 0..32 {
        let mut m=riddle_core::endgame::spawn(r,HERO_ID+1+n,"goblin",Pos{x:3+(n as i32%8),y:3+n as i32/8},1,true);
        m.stun=1000000;m.awake=true;m.speed=10;r.monsters.push(m);
    }
    g
}
fn main(){
    for (tier,perks) in [(0,false),(5,false),(5,true)] {
        let g=arena(tier,perks);let mut samples=Vec::new();
        for _ in 0..101 {
            let mut h=g.clone();let at=Instant::now();
            // Stay before natural pressure reinforcements: fixed enemy work.
            for _ in 0..500 {
                h.tick();let(r,mut cx)=h.ctx();riddle_core::turn::damage_hero(r,&mut cx,10,&Src::Gas);
            }
            samples.push(at.elapsed().as_nanos() as u64);
            assert!(h.run.as_ref().unwrap().over.is_none());
            assert_eq!(h.run.as_ref().unwrap().monsters.len(),32);std::hint::black_box(h);
        }
        samples.sort();println!("{{\"tier\":{tier},\"perks\":{perks},\"ticks\":500,\"samples\":101,\"median_ns\":{},\"ns_tick\":{}}}",samples[50],samples[50] as f64/500.0);
    }
}
