//! Encounter integration tests; fixtures are diagnostic, not earned clears.
use super::*;
use crate::{engine::{Run, HERO_ID}, geom::Pos, monster::Monster, wire::Ev};
fn game(tier: u32) -> Game {
    let mut g=Game::new_literal(7);g.sim=true;
    g.lineage.endgame=Some(Progress{tier,unlocked:tier.max(1),cleared:Some(tier.saturating_sub(1))});
    g.ensure_run();g
}
fn near(r: &Run) -> Pos {
    r.hero.pos.neighbours8().into_iter().find(|p| r.floor.map.passable(*p)).unwrap()
}
fn mods(tier:u32,elite:Option<Elite>) -> Modifiers {
    Modifiers{tier,affixes:affixes(tier),elite,tight_mirror:false}
}
#[test]
fn affixes_rotate_with_two_max_and_stats_are_bounded_at_extreme_tiers() {
    assert_eq!((affixes(0),affixes(1),affixes(2),affixes(3)),(0,ARMOURED,SWIFT,REGENERATING|ARMOURED));
    for tier in (1..=1000).chain([u32::MAX]) {
        assert!((1..=2).contains(&affixes(tier).count_ones()));
        let g=game(tier);let r=g.run.as_ref().unwrap();
        let m=spawn(r,100,"goblin",near(r),1,true);
        assert!(m.max_hp>0&&m.max_hp<=STAT_CAP);
        assert!(m.atk.0>=0&&m.atk.0<=m.atk.1&&m.atk.1<=STAT_CAP);
        assert_eq!(m.modifiers.unwrap().tier,tier);
    }
    assert_eq!(scaled(i32::MAX,u32::MAX,8),STAT_CAP);
    assert_eq!(scaled(0,u32::MAX,4),0);
}
#[test]
fn births_preserve_rng_and_zero_tier_is_the_ordinary_monster() {
    for tier in [0,1,3,5] {
        let g=game(tier);let r=g.run.as_ref().unwrap();let rng=r.rng.clone();
        let m=spawn(r,100,"goblin",near(r),3,true);
        assert_eq!(r.rng,rng);
        assert_eq!(m,spawn(r,100,"goblin",near(r),3,true));
        if tier==0 {assert_eq!(m,Monster::spawn(100,"goblin",near(r),3));}
        else {assert!(m.max_hp>Monster::spawn(100,"goblin",near(r),3).max_hp);}
    }
}
#[test]
fn elites_are_bounded_and_bosses_summons_captives_and_pets_are_distinct() {
    let g=game(5);let r=g.run.as_ref().unwrap();let p=near(r);
    let elite_count=(100..356).filter(|id|spawn(r,*id,"goblin",p,1,true).modifiers.unwrap().elite.is_some()).count();
    assert!((10..=60).contains(&elite_count));
    let kinds=(100..356).filter_map(|id|spawn(r,id,"goblin",p,1,true).modifiers.unwrap().elite).collect::<Vec<_>>();
    assert!(kinds.contains(&Elite::Shielded)&&kinds.contains(&Elite::Frenzied));
    for id in 100..356 {
        assert!(spawn(r,id,"goblin",p,1,false).modifiers.unwrap().elite.is_none());
        let boss=spawn(r,id,"mirror_king",p,33,true);
        assert!(boss.modifiers.unwrap().elite.is_none());assert!(boss.modifiers.unwrap().tight_mirror);
    }
    assert!(spawn(r,99,"captive",p,1,true).modifiers.is_none());
    assert!(Monster::spawn(99,"jackal",p,1).modifiers.is_none(),"companion/nest/stray constructor unchanged");
}
#[test]
fn shield_and_frenzy_have_actual_control_counters_and_do_not_buff_allies() {
    let mut m=Monster::spawn(100,"goblin",Pos{x:1,y:1},1);
    m.modifiers=Some(mods(1,Some(Elite::Shielded)));
    let def=m.effective_def();m.stun=1;assert_eq!(m.effective_def(),def-2);
    m.stun=0;m.paralysed=1;assert_eq!(m.effective_def(),def-2);
    m.paralysed=0;m.modifiers=Some(mods(1,Some(Elite::Frenzied)));
    let speed=m.effective_speed();let atk=m.effective_atk();
    m.hp=m.max_hp/2;assert_eq!(m.effective_speed(),speed+3);assert_eq!(m.effective_atk(),(atk.0,atk.1+2));
    m.slow_t=10;assert_eq!(m.effective_speed(),(speed+3-5).max(1));
    m.ally=true;assert_eq!(m.effective_atk(),atk);assert_eq!(m.effective_speed(),(speed-5).max(1));
}
#[test]
fn regeneration_ticks_are_poison_countered_and_never_learned_as_kind_traits() {
    for (awake,poison,expected_delta) in [(true,false,1),(true,true,-1),(false,false,0)] {
        let mut g=game(3);let r=g.run.as_ref().unwrap();let mut m=spawn(r,100,"goblin",near(r),1,false);
        m.hp=3;m.speed=0;m.energy= -100;m.awake=awake;m.stun=1000;
        if poison {m.poison=(1,100);}
        let r=g.run.as_mut().unwrap();r.hero.energy= -1000;r.monsters=vec![m];
        for _ in 0..10 {g.tick();}
        assert_eq!(g.run.as_ref().unwrap().monsters[0].hp,3+expected_delta);
        assert!(!g.lineage.facts.contains("foe:goblin:regen"));
    }
}
#[test]
fn split_and_tame_do_not_double_scale_or_persist_elite_power() {
    let g=game(5);let r=g.run.as_ref().unwrap();let mut parent=spawn(r,100,"pink_jelly",near(r),9,true);
    parent.modifiers.as_mut().unwrap().elite=Some(Elite::Frenzied);
    let child=spawn_split(101,"pink_jelly",near(r),9,parent.modifiers);
    assert_eq!((child.max_hp,child.atk,child.def,child.speed),(parent.max_hp,parent.atk,parent.def,parent.speed));
    assert_eq!(child.modifiers,parent.modifiers);
    parent.hp=(parent.max_hp/4).max(1);let fraction=parent.hp as f64/parent.max_hp as f64;
    normalise_tamed(&mut parent,9);
    let ordinary=Monster::spawn(100,"pink_jelly",near(r),9);
    assert!(parent.modifiers.is_none());assert_eq!((parent.max_hp,parent.atk,parent.def,parent.speed),(ordinary.max_hp,ordinary.atk,ordinary.def,ordinary.speed));
    assert!((parent.hp as f64/parent.max_hp as f64-fraction).abs()<=1.0/parent.max_hp as f64);
    let mut old=ordinary.clone();old.hp=2;let before=old.clone();normalise_tamed(&mut old,9);assert_eq!(old,before);
}
#[test]
fn actual_monster_attack_uses_frenzy_and_exact_killer_metadata() {
    let mut stronger_hits=0;
    for seed in 1..=32 {
        let mut g=game(1);g.run.as_mut().unwrap().rng=crate::rng::Rng::new(seed);
        let r=g.run.as_ref().unwrap();let mut m=spawn(r,100,"goblin",near(r),1,false);
        m.modifiers.as_mut().unwrap().elite=Some(Elite::Frenzied);
        m.atk=(10,10);m.awake=true;m.hp=m.max_hp/2;
        let r=g.run.as_mut().unwrap();r.monsters=vec![m];r.hero.hp=1000;r.hero.max_hp=1000;
        let (r,mut cx)=g.ctx();crate::ai::monster_act(r,&mut cx,0);
        if g.events.iter().any(|e|matches!(e,Ev::Attack{src:100,dst:HERO_ID,dmg,..} if *dmg>10)){stronger_hits+=1;}
    }
    assert!(stronger_hits>0,"real attack path must use the increased top roll");
    let mut g=game(1);let r=g.run.as_ref().unwrap();
    let mut first=spawn(r,100,"goblin",near(r),1,false);first.modifiers.as_mut().unwrap().elite=Some(Elite::Shielded);
    let mut killer=first.clone();killer.id=101;killer.modifiers.as_mut().unwrap().elite=Some(Elite::Frenzied);
    let expected=killer.modifiers;g.run.as_mut().unwrap().monsters=vec![first,killer];
    g.run.as_mut().unwrap().hero.hp=1;
    {let(r,mut cx)=g.ctx();crate::turn::damage_hero(r,&mut cx,50,&crate::turn::Src::Mon(1));}
    assert_eq!(g.run.as_ref().unwrap().death_modifiers,expected);
    g.sim=false;let rec=crate::trace::death_record(&g,g.run.as_ref().unwrap());
    assert_eq!(rec.death.modifiers,expected);assert_eq!(rec.death.difficulty,1);
    assert_eq!(rec.death.cause,"goblin");
}
#[test]
fn environmental_death_does_not_guess_an_elite_and_metadata_is_snapshotted() {
    let mut g=game(5);let r=g.run.as_ref().unwrap();let m=spawn(r,100,"goblin",near(r),1,true);
    let expected=m.modifiers;g.run.as_mut().unwrap().monsters=vec![m];
    let pos=g.run.as_ref().unwrap().hero.pos;
    g.run.as_mut().unwrap().floor.map.update_vision(pos,20);
    let snap=g.snapshot();assert_eq!(snap.difficulty,5);assert_eq!(snap.modifier_catalogue.len(),6);
    assert_eq!(snap.entities.iter().find(|e|e.id==100).unwrap().modifiers,expected);
    g.lineage.endgame.as_mut().unwrap().tier=0;
    assert_eq!(g.snapshot().difficulty,5);
    g.run.as_mut().unwrap().hero.hp=1;
    {let(r,mut cx)=g.ctx();crate::turn::damage_hero(r,&mut cx,50,&crate::turn::Src::Fire);}
    assert!(g.run.as_ref().unwrap().death_modifiers.is_none());
    let rec=crate::trace::death_record(&g,g.run.as_ref().unwrap());assert!(rec.death.modifiers.is_none());
    assert_eq!(rec.death.difficulty,5);
}
#[test]
fn modified_run_save_replay_and_zero_tier_wire_are_faithful() {
    let mut g=game(3);g.sim=false;g.step(60);
    let before=g.save();let mut loaded=Game::load(&before).unwrap();
    assert!(serde_json::to_string(&g.run).unwrap()==serde_json::to_string(&loaded.run).unwrap());
    assert!(g.step(100)==loaded.step(100),"saved affixes preserve actual events and snapshots");
    let old=game(0);let text=serde_json::to_string(&old.snapshot()).unwrap();
    assert!(!text.contains("modifiers")&&!text.contains("modifier_catalogue")&&!text.contains("difficulty"));
}
#[test]
fn tighter_mirror_reflects_second_attack_and_cadence_actually_alternates() {
    for tier in [0,1] {
        let mut g=game(tier);let r=g.run.as_ref().unwrap();
        let mut m=spawn(r,100,"mirror_king",near(r),33,false);m.hp=1000;m.max_hp=1000;m.awake=true;
        let r=g.run.as_mut().unwrap();r.hero.hp=1000;r.hero.max_hp=1000;r.hero.level=10;r.monsters=vec![m];
        r.verb_ring=vec!["attack".into()];
        {let(r,mut cx)=g.ctx();crate::ai::hero_attack(r,&mut cx,0,"attack",false);}
        let mirrored=g.events.iter().any(|e|matches!(e,Ev::Attack{src:100,dst:HERO_ID,verb:Some(v),..} if v=="mirror"));
        assert_eq!(mirrored,tier>0);
        g.events.clear();g.lineage.unlocks.insert("cadence".into());
        g.run.as_mut().unwrap().verb_ring=vec!["attack".into()];
        let(r,mut cx)=g.ctx();let v=crate::turn::view(r);
        assert!(crate::ai::try_verb(r,&mut cx,&crate::Verb{v:"tactic".into(),a:Some("cadence".into())},&v));
        let basic=g.events.iter().any(|e|matches!(e,Ev::Attack{src:HERO_ID,verb:Some(v),..} if v=="attack"));
        assert_eq!(basic,tier==0,"cadence must change rhythm before a stronger mirror");
        assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="mirror")));
    }
}
#[test]
fn actual_split_and_boss_summon_paths_keep_birth_modifiers() {
    let mut g=game(5);let r=g.run.as_ref().unwrap();
    let mut jelly=spawn(r,100,"pink_jelly",near(r),1,true);jelly.modifiers.as_mut().unwrap().elite=Some(Elite::Shielded);
    let inherited=jelly.modifiers;let max=jelly.max_hp;let atk=jelly.atk;
    g.run.as_mut().unwrap().monsters=vec![jelly];
    {let(r,mut cx)=g.ctx();crate::turn::damage_monster(r,&mut cx,0,1,&crate::turn::Src::Hero{ranged:false});}
    let r=g.run.as_ref().unwrap();assert_eq!(r.monsters.len(),2);
    assert!(r.monsters.iter().all(|m|m.modifiers==inherited&&m.max_hp==max&&m.atk==atk));
    let mut g=game(5);let r=g.run.as_ref().unwrap();
    let mut boss=spawn(r,100,"goblin_warlord",near(r),8,false);boss.awake=true;boss.pending=Some(crate::monster::Pending::Rally);
    g.run.as_mut().unwrap().monsters=vec![boss];
    {let(r,mut cx)=g.ctx();crate::ai::monster_act(r,&mut cx,0);}
    let r=g.run.as_ref().unwrap();assert!(r.monsters.len()>1);
    for m in &r.monsters[1..]{assert!(m.summoned);let mods=m.modifiers.unwrap();assert_eq!((mods.tier,mods.affixes,mods.elite),(5,affixes(5),None));}
}
#[test]
fn actual_taming_path_removes_modifiers_before_companion_record_is_written() {
    let mut tamed=0;
    for seed in 1..=32 {
        let mut g=game(5);let r=g.run.as_ref().unwrap();let mut m=spawn(r,100,"goblin",near(r),1,true);
        m.modifiers.as_mut().unwrap().elite=Some(Elite::Frenzied);m.hp=1;m.awake=true;
        let r=g.run.as_mut().unwrap();r.rng=crate::rng::Rng::new(seed);r.monsters=vec![m];
        let mut leash=crate::item::Item::new(9999,"leash");leash.amount=1;r.hero.inv.push(leash);
        {let(r,mut cx)=g.ctx();let v=crate::turn::view(r);assert!(crate::ai::try_verb(r,&mut cx,&crate::Verb{v:"tame".into(),a:Some("nearest".into())},&v));}
        let r=g.run.as_ref().unwrap();
        if r.monsters[0].ally {
            tamed+=1;let m=&r.monsters[0];assert!(m.modifiers.is_none());
            let ordinary=Monster::spawn(100,"goblin",m.pos,1);
            assert_eq!((m.max_hp,m.atk,m.def,m.speed),(ordinary.max_hp,ordinary.atk,ordinary.def,ordinary.speed));
            assert_eq!(r.companions.last().unwrap().max_hp,ordinary.max_hp);
        }
    }
    assert!(tamed>0);
}
#[test]
fn ally_mirror_uses_the_saved_boss_rhythm_too() {
    for tier in [0,1] {
        let mut g=game(tier);let r=g.run.as_ref().unwrap();let p=near(r);
        let mut king=spawn(r,100,"mirror_king",p,33,false);king.hp=1000;king.max_hp=1000;king.awake=true;
        let q=p.neighbours8().into_iter().find(|q|r.floor.map.passable(*q)&&*q!=r.hero.pos).unwrap();
        let mut ally=Monster::spawn(101,"goblin",q,1);ally.ally=true;ally.awake=true;ally.verb_ring=vec!["attack".into()];
        g.run.as_mut().unwrap().monsters=vec![king,ally];
        {let(r,mut cx)=g.ctx();crate::ai::monster_act(r,&mut cx,1);}
        let reflect=g.events.iter().any(|e|matches!(e,Ev::Attack{src:100,dst:101,verb:Some(v),..} if v=="mirror"));
        assert_eq!(reflect,tier>0);
    }
}

#[test]
fn preview_is_read_only_matches_birth_and_removes_old_challenge_on_new_path() {
    let mut g=game(5);g.run=None;g.lineage.ended=true;g.lineage.variant="hunted".into();
    let before=g.save();
    for tier in 0..=5 {
        let offer=g.descent_offer(tier).unwrap();
        assert_eq!(offer.hp_bonus_percent,u64::from(tier)*8);
        assert_eq!(offer.attack_bonus_percent,u64::from(tier)*4);
        assert_eq!(offer.affixes.iter().fold(0,|mask,m|mask|m.mask),affixes(tier));
        assert_eq!(offer.elites.len(),if tier>0 {2}else{0});
        assert_eq!(offer.boss.is_some(),tier>0);
    }
    assert!(g.descent_offer(6).is_err());assert_eq!(g.save(),before);
    g.begin_descent(5).unwrap();assert!(g.lineage.variant.is_empty());
    let wire=g.lineage();assert_eq!(wire.endgame.unwrap().tier,5);
    assert!(g.descent_offer(0).is_err());
}
#[test]
fn old_ending_offers_numbered_progress_without_save_mutation() {
    let mut g=Game::new_literal(2);let before=g.save();
    assert!(g.lineage().endgame.is_none());assert_eq!(g.save(),before);
    g.lineage.ended=true;g.lineage.ascension=37;let before=g.save();
    let wire=g.lineage();assert_eq!(wire.endgame.unwrap(),Progress{tier:0,unlocked:1,cleared:Some(0)});
    assert_eq!(g.descent_offer(1).unwrap().affixes.len(),1);
    assert!(g.descent_offer(2).is_err());assert_eq!(g.save(),before);
}

#[test]
fn difficulty_stat_cap_still_holds_after_grudge_bonus() {
    let g=game(u32::MAX);let r=g.run.as_ref().unwrap();
    let mut m=spawn(r,100,"goblin",near(r),33,false);m.make_grudge("Morog");
    assert_eq!(m.max_hp,STAT_CAP);assert_eq!(m.hp,STAT_CAP);
    assert!(m.atk.1<=STAT_CAP);assert!(m.atk.0<=m.atk.1);
}
