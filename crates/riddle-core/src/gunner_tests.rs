//! Controlled class/progression/skill checks, not earned campaign results.
use crate::{Game,hero::Class,item::Item,geom::Pos,rules::{Verb,Row,Cond},wire::Ev};
fn home()->Game {
    let mut g=Game::new_resident(3);g.lineage.marks=8;
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=13;g
}
fn unlocked()->Game {let mut g=home();g.buy("gunner").unwrap();g.set_class("gunner").unwrap();g}
fn arena(kind:&str,level:u32)->Game {
    let mut g=crate::tests::arena();g.sim=true;
    crate::tests::rules(&mut g,vec![Row::new(vec![],Verb::new("hold"))]);
    let r=g.run.as_mut().unwrap();r.hero.class=Class::Gunner;r.hero.level=level;
    r.hero.weapon=Some(Item::new(900,kind)).into();r.hero.armour=None.into();r.hero.inv.clear();
    r.hero.str_bonus=0;r.hero.gift=Default::default();r.gun_skills=Some(Default::default());
    r.floor.map.update_vision(r.hero.pos,8);g.events.clear();g
}
fn foe(g:&mut Game,x:i32,y:i32) {
    crate::tests::add_monster(g,"goblin",x,y);let m=g.run.as_mut().unwrap().monsters.last_mut().unwrap();
    m.hp=100;m.max_hp=100;m.stun=1000;
}
fn act(g:&mut Game,verb:&str)->bool {let(r,mut cx)=g.ctx();let v=crate::turn::view(r);crate::ai::try_verb(r,&mut cx,&Verb::new(verb),&v)}
fn cadence(g:&mut Game)->bool {let(r,mut cx)=g.ctx();let v=crate::turn::view(r);crate::ai::try_verb(r,&mut cx,&Verb::arg("tactic","cadence"),&v)}
#[test]
fn known_ranged_reflection_draws_sidearm_and_restores_saved_gun() {
    let mut g=arena("long_gun",3);crate::tests::add_monster(&mut g,"lich",6,5);
    g.run.as_mut().unwrap().monsters[0].stun=1000;
    g.run.as_mut().unwrap().hero.inv.push(Item::new(901,"sword"));
    g.lineage.facts.insert("foe:lich:reflect".into());
    assert!(act(&mut g,"gunner_tactic"));
    assert_eq!(g.run.as_ref().unwrap().hero.weapon_kind(),"sword");
    assert_eq!(g.run.as_ref().unwrap().bow_swap.as_ref().unwrap().firearm.unwrap().loaded,1);
    assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{..})));
    let save=g.save();let mut g=Game::load(&save).unwrap();assert_eq!(g.save(),save);
    assert!(act(&mut g,"gunner_tactic"));assert!(act(&mut g,"gunner_tactic"));
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{src:crate::engine::HERO_ID,verb:Some(v),..} if v=="attack")));
    assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="reflect")));
    g.run.as_mut().unwrap().monsters[0].hp=0;for _ in 0..10 {g.tick();}
    assert_eq!(g.run.as_ref().unwrap().hero.weapon_kind(),"long_gun");
    assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded,1);
    assert!(g.events.iter().any(|e|matches!(e,Ev::Callout{text,..} if text=="gun ready")));
    let mut g=arena("long_gun",3);crate::tests::add_monster(&mut g,"lich",6,5);
    g.run.as_mut().unwrap().monsters[0].stun=1000;
    g.run.as_mut().unwrap().hero.inv.push(Item::new(901,"sword"));
    assert!(act(&mut g,"fire"));assert!(act(&mut g,"reload"));assert!(act(&mut g,"attack"));
    assert_eq!(g.run.as_ref().unwrap().hero.weapon_kind(),"sword");
    let mut g=Game::load(&g.save()).unwrap();for _ in 0..19 {g.tick();}
    assert!(g.run.as_ref().unwrap().gun_reload.is_some());g.tick();
    assert!(g.run.as_ref().unwrap().gun_reload.is_none());
    assert_eq!(g.run.as_ref().unwrap().bow_swap.as_ref().unwrap().firearm.unwrap().loaded,1);
    let mut g=arena("long_gun",3);crate::tests::add_monster(&mut g,"lich",6,5);
    g.run.as_mut().unwrap().hero.inv.push(Item::new(901,"sword"));
    assert!(act(&mut g,"gunner_tactic"));assert!(g.run.as_ref().unwrap().bow_swap.is_none());
    g.lineage.facts.insert("foe:lich:reflect".into());assert!(act(&mut g,"fire"));
    assert!(g.run.as_ref().unwrap().bow_swap.is_none(),"explicit fire retains the player's reflection risk");
}
#[test]
fn automatic_gun_choices_skip_unneeded_aim_and_illegal_or_weak_spread() {
    let mut g=arena("long_gun",3);foe(&mut g,6,5);g.run.as_mut().unwrap().monsters[0].hp=1;
    assert!(act(&mut g,"gunner_tactic"));
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="fire")));
    assert!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().aim.is_none());
    let mut g=arena("long_gun",3);foe(&mut g,6,5);
    assert!(act(&mut g,"gunner_tactic"));assert!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().aim.is_some());
    assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{..})));
    for (second_x,weak,expected,left) in [(10,true,"fire",1),(7,true,"fire",1),(7,false,"close_burst",0),(10,false,"close_burst",0)] {
        let mut g=arena("short_gun",3);foe(&mut g,6,5);foe(&mut g,second_x,5);
        if weak {for m in g.run.as_mut().unwrap().monsters.iter_mut() {m.hp=1;}}
        assert!(act(&mut g,"gunner_tactic"));
        assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v==expected)),"{second_x}/{weak}");
        assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded,left);
    }
}
#[test]
fn authored_cadence_uses_real_aim_burst_and_reload_commitments() {
    for kind in ["long_gun","short_gun"] {
        let mut g=arena(kind,3);foe(&mut g,6,5);
        g.lineage.unlocks.insert("cadence".into());
        assert!(act(&mut g,"fire"));
        g.run.as_mut().unwrap().verb_ring=vec!["fire".into()];
        assert!(cadence(&mut g));
        let at=g.run.as_ref().unwrap().gun_reload.unwrap().at;
        assert!(!act(&mut g,"fire"));
        while g.run.as_ref().unwrap().turn<at {g.tick();}
        assert!(g.run.as_ref().unwrap().gun_reload.is_none());
        g.events.clear();
        if kind=="long_gun" {
            assert!(cadence(&mut g));
            assert!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().aim.is_some());
            assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{..})));
        }
        assert!(cadence(&mut g));
        let expected=if kind=="long_gun" {"aimed_shot"}else{"close_burst"};
        assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v==expected)));
        assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded,0);
        assert!(cadence(&mut g));assert!(g.run.as_ref().unwrap().gun_reload.is_some());
    }
}
#[test]
fn paid_historical_unlock_keeps_missing_xp_absent_and_refuses_exactly() {
    let mut g=home();assert!(!g.lineage.classes.contains_key("gunner"));
    let before=g.save();assert!(!g.unlocks().iter().any(|u|u.id=="gunner"));assert_eq!(g.save(),before);
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=12;let before=g.save();assert!(g.buy("gunner").is_err());assert_eq!(g.save(),before);
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=13;g.lineage.marks=7;let before=g.save();assert!(g.buy("gunner").is_err());assert_eq!(g.save(),before);
    g.lineage.marks=8;g.buy("gunner").unwrap();assert_eq!(g.lineage.marks,0);assert_eq!(g.lineage.class,Class::Fighter);
    assert_eq!((g.lineage.classes["gunner"].level,g.lineage.classes["gunner"].xp),(1,0));
    let before=g.save();assert!(g.buy("gunner").is_err());assert_eq!(g.save(),before);
    let mut g=home();g.ensure_run();let before=g.save();assert!(g.buy("gunner").is_err());assert_eq!(g.save(),before);
}
#[test]
fn owned_short_gun_costs_once_switches_freely_and_inherits_forge_steps() {
    let mut g=unlocked();g.lineage.gold=1000;g.lineage.kit_unit=Some(100);g.lineage.kit.insert("weapon".into(),4);
    crate::kit::buy(&mut g,"short_gun").unwrap();assert_eq!(g.lineage.gold,900);
    assert_eq!(crate::firearm::starting_kind(&g.lineage),"short_gun");
    assert_eq!(g.lineage().guns[1].price,0);assert!(g.lineage().guns[1].selected);
    crate::kit::buy(&mut g,"long_gun").unwrap();crate::kit::buy(&mut g,"short_gun").unwrap();assert_eq!(g.lineage.gold,900);
    let save=g.save();let mut g=Game::load(&save).unwrap();assert_eq!(g.save(),save);
    g.lineage.bloodline.as_mut().unwrap().upgrades.insert("damage".into(),3);
    g.lineage.classes.get_mut("gunner").unwrap().level=3;
    let damage=g.lineage().guns[1].damage;
    let snap=g.send();let gun=snap.hero.gun.unwrap();assert_eq!((gun.kind.as_str(),gun.capacity,gun.loaded),("short_gun",2,2));
    assert_eq!(damage,gun.damage,"forge includes inherited damage and actual class level bonus");
    assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().enchant,4);
    let before=g.save();assert!(crate::kit::buy(&mut g,"long_gun").is_err());assert!(g.set_class("fighter").is_err());assert_eq!(g.save(),before);
    assert!(g.lineage().guns.iter().all(|o|!o.available&&o.blocked.as_deref()==Some("hero away")));
}
#[test]
fn insufficient_gun_purchase_and_wrong_class_are_read_only() {
    let mut g=unlocked();g.lineage.gold=0;let before=g.save();assert!(crate::kit::buy(&mut g,"short_gun").is_err());assert_eq!(g.save(),before);
    g.set_class("fighter").unwrap();let before=g.save();assert!(crate::kit::buy(&mut g,"short_gun").is_err());assert_eq!(g.save(),before);
}
#[test]
fn automatic_row_is_named_below_authored_rules_and_literal_stays_explicit() {
    let mut g=unlocked();let own=Row::new(vec![Cond::n("foes>=",1)],Verb::new("hold")).from("player");
    g.lineage.pkg.pen_open=true;g.lineage.pkg.pen=vec![own.clone()];crate::packages::recompile(&mut g.lineage);
    let rows=&g.lineage.rules().rows;let i=rows.iter().position(|r|r.origin.as_deref()==Some("class:gunner")).unwrap();
    assert_eq!(rows[0],own);assert!(i>0);assert!(rows[i].is_pkg());assert_eq!(crate::packages::row_label(&rows[i]).as_deref(),Some("Gunner"));
    crate::packages::make_literal(&mut g.lineage);let before=g.lineage.rules().clone();g.set_class("fighter").unwrap();g.set_class("gunner").unwrap();assert_eq!(*g.lineage.rules(),before);
    let mut g=unlocked();g.send();let mut fire=false;let mut reload=false;
    for _ in 0..1000 {let step=g.step(10);fire|=step.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="fire"));reload|=step.events.iter().any(|e|matches!(e,Ev::Callout{text,..} if text=="loaded"));if fire&&reload||step.run_over {break;}}
    assert!(fire&&reload,"normal generated first Gunner send must fire/reload without Pen edits");
}
#[test]
fn aim_costs_preparation_then_accurate_harder_shot_and_movement_cancels() {
    let mut g=arena("long_gun",3);foe(&mut g,6,5);assert!(act(&mut g,"aimed_shot"));
    assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded,1);
    assert!(!g.events.iter().any(|e|matches!(e,Ev::Attack{..})));
    let before=g.save();assert!(!act(&mut g,"aimed_shot"));assert_eq!(g.save(),before);
    let mut g=Game::load(&g.save()).unwrap();assert!(act(&mut g,"fire"));
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{hit:true,dmg:9..=15,verb:Some(v),..} if v=="aimed_shot")));
    assert!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().aim.is_none());
    let mut g=arena("long_gun",3);foe(&mut g,6,5);assert!(act(&mut g,"aimed_shot"));
    let(r,mut cx)=g.ctx();crate::ai::move_hero(r,&mut cx,Pos::new(4,4));assert!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().aim.is_none());
    let mut g=arena("long_gun",2);foe(&mut g,6,5);let before=g.save();assert!(!act(&mut g,"aimed_shot"));assert_eq!(g.save(),before);
}
#[test]
fn smoke_retreat_blinds_nearby_hostiles_with_exact_cooldown() {
    let mut g=arena("long_gun",5);foe(&mut g,5,5);let from=g.run.as_ref().unwrap().hero.pos;
    assert!(act(&mut g,"smoke_retreat"));let r=g.run.as_ref().unwrap();assert_ne!(r.hero.pos,from);
    assert_eq!(r.monsters[0].blind,30);assert_eq!(r.gun_skills.as_ref().unwrap().smoke_ready,90);
    let before=g.save();assert!(!act(&mut g,"smoke_retreat"));assert_eq!(g.save(),before);
}
#[test]
fn fast_reload_and_finisher_use_real_level_timer_ammo_and_hp_gates() {
    for (kind,ticks) in [("long_gun",10),("short_gun",8)] {
        let mut g=arena(kind,7);foe(&mut g,6,5);assert!(act(&mut g,"fire"));assert!(act(&mut g,"fast_reload"));
        assert_eq!(g.run.as_ref().unwrap().gun_reload.unwrap().at,ticks);assert_eq!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().fast_ready,100);
        for _ in 0..ticks-1 {g.tick();}assert!(g.run.as_ref().unwrap().gun_reload.is_some());g.tick();assert!(g.run.as_ref().unwrap().gun_reload.is_none());
        assert!(act(&mut g,"fire"));let before=g.save();assert!(!act(&mut g,"fast_reload"));assert_eq!(g.save(),before);
    }
    let mut g=arena("long_gun",9);foe(&mut g,6,5);let before=g.save();assert!(!act(&mut g,"finishing_shot"));assert_eq!(g.save(),before);
    g.run.as_mut().unwrap().monsters[0].hp=25;assert!(act(&mut g,"finishing_shot"));
    assert_eq!(g.run.as_ref().unwrap().gun_skills.as_ref().unwrap().finish_ready,60);
    assert_eq!(g.run.as_ref().unwrap().hero.weapon.as_ref().unwrap().firearm.unwrap().loaded,0);
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="finishing_shot")));
}
