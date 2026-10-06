//! Real firearm actions on controlled arenas; these are not earned campaigns.
use crate::{Game, item::Item, geom::Pos, rules::{Row,Verb}, rng::Rng, wire::Ev, engine::HERO_ID};
fn arena(kind: &str) -> Game {
    let mut g=crate::tests::arena();g.sim=true;
    crate::tests::rules(&mut g,vec![Row::new(vec![],Verb::new("hold"))]);
    let r=g.run.as_mut().unwrap();r.hero.weapon=Some(Item::new(900,kind)).into();
    r.hero.armour=None.into();r.hero.str_bonus=0;r.hero.gift=Default::default();
    r.hero.level=3;r.hero.hp=1000;r.hero.max_hp=1000;r.hero.max_hp_base=1000;
    r.hero.class=crate::hero::Class::Gunner;r.gun_skills=Some(Default::default());
    r.floor.map.update_vision(r.hero.pos,8);g.events.clear();g
}
fn foe(g:&mut Game,kind:&str,p:Pos)->u32 {
    let id=crate::tests::add_monster(g,kind,p.x,p.y);
    let m=g.run.as_mut().unwrap().monsters.last_mut().unwrap();m.hp=1000;m.max_hp=1000;m.stun=1000;id
}
fn act(g:&mut Game,verb:&str)->bool {
    let (r,mut cx)=g.ctx();let v=crate::turn::view(r);
    crate::ai::try_verb(r,&mut cx,&Verb::new(verb),&v)
}
fn loaded(g:&Game)->u8 {g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded}
fn shot(g:&Game)->(bool,i32) {
    g.events.iter().find_map(|e|if let Ev::Attack{src,hit,dmg,..}=e {(*src==HERO_ID).then_some((*hit,*dmg))}else{None}).unwrap()
}
#[test]
fn actual_range_sight_and_target_refusals_preserve_whole_save_and_events() {
    for (kind,range) in [("long_gun",8),("short_gun",3)] {
        let mut g=arena(kind);foe(&mut g,"goblin",Pos::new(4+range,5));
        assert!(act(&mut g,"fire"));assert_eq!(loaded(&g),if kind=="long_gun" {0}else{1});
        for mode in 0..4 {
            let mut g=arena(kind);foe(&mut g,"goblin",Pos::new(4+range,5));
            let r=g.run.as_mut().unwrap();r.last_target=Some(777);
            match mode {
                0=>r.monsters[0].pos=Pos::new(4+range+1,5),
                1=>r.floor.map.set(Pos::new(5,5),crate::tiles::Tile::Wall),
                2=>r.monsters[0].ally=true,
                _=>r.floor.map.update_vision(r.hero.pos,1),
            }
            let before=g.save();let events=g.events.clone();
            assert!(!act(&mut g,"fire"),"{kind} mode {mode}");
            assert_eq!(g.save(),before);assert_eq!(g.events,events);
        }
    }
}
#[test]
fn real_rolls_pierce_only_long_gun_armour_and_misses_spend_ammo() {
    let mut misses=0;
    for kind in ["long_gun","short_gun"] {
        for seed in 0..20 {
            let mut g=arena(kind);foe(&mut g,"iron_golem",Pos::new(6,5));
            let r=g.run.as_mut().unwrap();r.rng=Rng::new(seed);
            let def=r.monsters[0].effective_def();let atk=r.hero.atk();let mut rng=r.rng.clone();
            let p=super::Profile::of(kind).unwrap();
            let expected=crate::ai::roll_hit_pct(&mut rng,atk,(def-p.armour_piercing).max(0),r.hero.hit_pct());
            assert!(act(&mut g,"fire"));assert_eq!(shot(&g),expected);
            assert_eq!(g.run.as_ref().unwrap().rng,rng);
            assert_eq!(g.run.as_ref().unwrap().monsters[0].hp,1000-expected.1);
            assert_eq!(loaded(&g),p.capacity-1);
            if !expected.0 {misses+=1;}
        }
    }
    assert!(misses>0);
}
#[test]
fn short_spread_caps_three_targets_and_excludes_allies_rear_and_walls() {
    let mut g=arena("short_gun");
    let primary=foe(&mut g,"goblin",Pos::new(6,5));
    let second=foe(&mut g,"goblin",Pos::new(6,4));
    let third=foe(&mut g,"goblin",Pos::new(6,6));
    let fourth=foe(&mut g,"goblin",Pos::new(7,5));
    let ally=foe(&mut g,"goblin",Pos::new(5,4));g.run.as_mut().unwrap().monsters.last_mut().unwrap().ally=true;
    let rear=foe(&mut g,"goblin",Pos::new(2,5));
    // Give the rear foe fear so the actual nearest targeting chooses the front.
    g.run.as_mut().unwrap().monsters.last_mut().unwrap().fear=100;
    assert!(act(&mut g,"fire"));
    let targets:Vec<u32>=g.events.iter().filter_map(|e|if let Ev::Attack{src,dst,..}=e {(*src==HERO_ID).then_some(*dst)}else{None}).collect();
    assert_eq!(targets,vec![primary,second,third]);
    assert!(!targets.contains(&fourth)&&!targets.contains(&ally)&&!targets.contains(&rear));
    assert_eq!(loaded(&g),1);
    let mut g=arena("short_gun");let primary=foe(&mut g,"goblin",Pos::new(6,5));
    let behind=foe(&mut g,"goblin",Pos::new(6,4));
    g.run.as_mut().unwrap().floor.map.set(Pos::new(5,4),crate::tiles::Tile::Wall);
    assert!(act(&mut g,"fire"));
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{dst,..} if *dst==primary)));
    assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{dst,..} if *dst==behind)));
}
#[test]
fn reflected_shots_consume_chambers_and_preserve_ranged_source() {
    let mut g=arena("long_gun");let id=foe(&mut g,"lich",Pos::new(6,5));
    assert!(act(&mut g,"fire"));assert_eq!(loaded(&g),0);
    assert_eq!(g.run.as_ref().unwrap().monsters[0].hp,1000);
    assert!(!g.run.as_ref().unwrap().melee_used);
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{src,dst,verb:Some(v),..} if *src==id&&*dst==HERO_ID&&v=="reflect")));
}
#[test]
fn generic_attack_and_direct_shoot_cannot_bypass_empty_chambers() {
    let mut g=arena("long_gun");foe(&mut g,"goblin",Pos::new(5,5));
    assert!(act(&mut g,"attack"));assert_eq!(loaded(&g),0);
    let before=g.save();let events=g.events.clone();
    assert!(!act(&mut g,"attack"));assert_eq!(g.save(),before);assert_eq!(g.events,events);
    let(r,mut cx)=g.ctx();crate::ai::hero_attack(r,&mut cx,0,"shoot",false);
    assert_eq!(g.save(),before);assert_eq!(g.events,events);
}
#[test]
fn actual_reload_deadline_precedes_action_and_survives_swapping_and_save() {
    for kind in ["long_gun","short_gun"] {
        let mut g=arena(kind);foe(&mut g,"goblin",Pos::new(6,5));assert!(act(&mut g,"fire"));
        assert!(act(&mut g,"reload"));let deadline=g.run.as_ref().unwrap().gun_reload.unwrap().at;
        let before=g.save();assert!(!act(&mut g,"fire"));assert!(!act(&mut g,"reload"));assert_eq!(g.save(),before);
        let r=g.run.as_mut().unwrap();let stowed=r.hero.weapon.replace(Item::new(901,"short_gun")).unwrap();r.hero.inv.push(stowed);
        let mut g=Game::load(&g.save()).unwrap();
        for _ in 0..deadline-1 {g.tick();}
        assert_eq!(g.run.as_ref().unwrap().hero.inv.iter().find(|w|w.id==900).unwrap().firearm.unwrap().reload_at,Some(deadline));
        g.tick();let r=g.run.as_ref().unwrap();assert!(r.gun_reload.is_none());
        let c=r.hero.inv.iter().find(|w|w.id==900).unwrap().firearm.unwrap();
        assert_eq!(c.loaded,super::Profile::of(kind).unwrap().capacity);assert_eq!(c.reload_at,None);
        assert_eq!(g.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count(),1);
        g.tick();assert_eq!(g.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count(),1);
    }
}
#[test]
fn reload_boundary_batched_single_and_reloaded_execution_are_exact() {
    let mut skipped=0;
    for kind in ["long_gun","short_gun"] {
        let mut batch=arena(kind);foe(&mut batch,"goblin",Pos::new(6,5));assert!(act(&mut batch,"fire"));assert!(act(&mut batch,"reload"));
        let mut reference=batch.clone();let mut settled=false;
        for limit in [2,4,8,1,3,5,17] {
            let mut used=0;
            while used<limit {
                let n=batch.tick_batch(limit-used,&mut settled);assert!(n>0&&n<=limit-used);skipped+=n.saturating_sub(1);
                for _ in 0..n {reference.tick();}
                used+=n;assert_eq!(batch.events,reference.events);assert_eq!(batch.save(),reference.save());
            }
            let loaded=Game::load(&batch.save()).unwrap();assert_eq!(loaded.save(),batch.save());
        }
        assert!(batch.run.as_ref().unwrap().gun_reload.is_none());
        assert_eq!(batch.events.iter().filter(|e|matches!(e,Ev::Callout{text,..} if text=="loaded")).count(),1);
    }
    assert!(skipped>0,"must exercise the actual quiet batch optimization");
}
#[test]
fn actual_close_burst_requires_two_chambers_and_both_hits_are_one_commitment() {
    let mut g=arena("short_gun");foe(&mut g,"goblin",Pos::new(6,5));
    assert!(act(&mut g,"close_burst"));assert_eq!(loaded(&g),0);
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="close_burst")));
    let mut g=arena("short_gun");foe(&mut g,"goblin",Pos::new(6,5));assert!(act(&mut g,"fire"));
    let before=g.save();assert!(!act(&mut g,"close_burst"));assert_eq!(g.save(),before);
    let mut g=arena("long_gun");foe(&mut g,"goblin",Pos::new(6,5));let before=g.save();
    assert!(!act(&mut g,"close_burst"));assert_eq!(g.save(),before);
}

#[test]
fn snapshot_reports_played_gun_reload_and_camp_changes_cannot_rewrite_it() {
    let mut g=arena("long_gun");foe(&mut g,"goblin",Pos::new(6,5));assert!(act(&mut g,"fire"));assert!(act(&mut g,"reload"));
    let snap=g.snapshot();let gun=snap.hero.gun.unwrap();
    assert_eq!((gun.item,gun.kind.as_str(),gun.loaded,gun.capacity,gun.range),(900,"long_gun",0,1,8));
    assert_eq!((gun.damage,gun.armour_piercing,gun.reload_ticks,gun.reload_left),((6,10),2,20,20));
    g.lineage.class=crate::hero::Class::Ranger;
    assert_eq!(g.snapshot().hero.gun.unwrap(),gun);
    for _ in 0..7 {g.tick();}
    assert_eq!(g.snapshot().hero.gun.unwrap().reload_left,13);
    let g=Game::load(&g.save()).unwrap();assert_eq!(g.snapshot().hero.gun.unwrap().reload_left,13);
}

#[test]
fn aimed_primary_does_not_bypass_secondary_warlord_shield_wall() {
    let mut intercepted=false;
    for seed in 0..12 {
        let mut g=arena("short_gun");foe(&mut g,"lich",Pos::new(6,5));
        foe(&mut g,"goblin_warlord",Pos::new(6,6));foe(&mut g,"goblin",Pos::new(6,7));
        g.run.as_mut().unwrap().rng=Rng::new(seed);
        let(r,mut cx)=g.ctx();let v=crate::turn::view(r);
        assert!(crate::ai::fire_gun(r,&mut cx,"tag:boss",&v,false));
        let r=g.run.as_ref().unwrap();assert_eq!(r.monsters[1].hp,1000);
        assert!(!r.aimed);assert_eq!(loaded(&g),1);
        if r.monsters[2].hp<1000 {intercepted=true;}
    }
    assert!(intercepted,"must include a positive ordinary shield interception");
}
