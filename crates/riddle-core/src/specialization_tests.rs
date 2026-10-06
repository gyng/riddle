//! Controlled fixtures exercise real choice, actions, damage and transport.
use super::*;
use crate::{wire::{ClassProg,Ev},monster::Monster,geom::Pos,turn::Src,rng::Rng};
fn qualified(class:Class)->Game {
    let mut g=Game::new_resident(3);g.lineage.class=class;
    crate::legacy::ensure(&mut g.lineage);
    g.lineage.unlocks.extend(["caster".into(),"rogue".into()]);
    for c in [Class::Fighter,Class::Caster] {g.lineage.classes.insert(c.name().into(),ClassProg{level:10,xp:0,next:0});}
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=23;
    crate::packages::recompile(&mut g.lineage);crate::oath::refresh(&mut g.lineage);g
}
fn arena(style:Style)->Game {
    let mut g=qualified(style.parent());choose(&mut g,style.id()).unwrap();g.send();g.sim=true;
    let r=g.run.as_mut().unwrap();r.hero.hp=40;r.hero.max_hp=40;r.hero.max_hp_base=40;
    r.hero.inv.clear();r.hero.armour=None.into();r.hero.gift=Default::default();r.monsters.clear();r.rng=Rng::new(5);g.events.clear();g
}
fn foe(g:&mut Game,kind:&str)->usize {
    let r=g.run.as_ref().unwrap();let pos=r.hero.pos.neighbours8().into_iter().find(|p|r.floor.map.passable(*p)).unwrap();
    let mut m=Monster::spawn(100+g.run.as_ref().unwrap().monsters.len() as u32,kind,pos,1);
    m.hp=100;m.max_hp=100;m.awake=true;
    let r=g.run.as_mut().unwrap();r.floor.map.update_vision(r.hero.pos,8);let i=r.monsters.len();r.monsters.push(m);i
}
fn act(g:&mut Game,verb:&str)->bool {
    let(r,mut cx)=g.ctx();let view=crate::turn::view(r);
    crate::ai::try_verb(r,&mut cx,&Verb::new(verb),&view)
}
fn hit(g:&mut Game,damage:i32,src:Src) {let(r,mut cx)=g.ctx();crate::turn::damage_hero(r,&mut cx,damage,&src);}
#[test]
fn choice_gates_refuse_exactly_and_none_preserves_xp_gold_legacy_and_class_memory() {
    let mut g=qualified(Class::Fighter);
    for id in ["bogus","hexbinder"] {let before=g.save();assert!(choose(&mut g,id).is_err());assert_eq!(g.save(),before);}
    g.lineage.classes.get_mut("fighter").unwrap().level=9;
    let before=g.save();assert!(choose(&mut g,"sentinel").is_err());assert_eq!(g.save(),before);
    g.lineage.classes.get_mut("fighter").unwrap().level=10;
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=22;
    let before=g.save();assert!(choose(&mut g,"sentinel").is_err());assert_eq!(g.save(),before);
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=23;
    g.lineage.town.home=Some(false);let before=g.save();assert!(choose(&mut g,"sentinel").is_err());assert_eq!(g.save(),before);
    g.lineage.town.home=Some(true);let classes=g.lineage.classes.clone();let gold=g.lineage.gold;let legacy=g.lineage.bloodline.clone();
    choose(&mut g,"sentinel").unwrap();assert_eq!(current(&g.lineage),Some(Style::Sentinel));
    g.set_class("caster").unwrap();choose(&mut g,"hexbinder").unwrap();
    g.set_class("fighter").unwrap();assert_eq!(current(&g.lineage),Some(Style::Sentinel));
    assert!(g.lineage.rules().rows.iter().any(|r|r.origin.as_deref()==Some("style:sentinel")));
    choose(&mut g,"none").unwrap();assert_eq!(current(&g.lineage),None);
    assert!(!g.lineage.rules().rows.iter().any(|r|r.origin.as_deref().is_some_and(|o|o.starts_with("style:"))));
    g.set_class("caster").unwrap();assert_eq!(current(&g.lineage),Some(Style::Hexbinder));
    assert_eq!((g.lineage.classes,g.lineage.gold,g.lineage.bloodline),(classes,gold,legacy));
}
#[test]
fn away_choice_refusal_and_saved_style_snapshot_are_exact() {
    let mut g=arena(Style::Sentinel);g.sim=false;let before=g.save();
    assert!(choose(&mut g,"none").is_err());assert_eq!(g.save(),before);
    let mut loaded=Game::load(&before).unwrap();assert!(loaded.save()==before,"complete live reload");
    g.lineage.specializations.clear();loaded.lineage.specializations.clear();
    foe(&mut g,"goblin");foe(&mut loaded,"goblin");
    assert!(act(&mut g,"riposte"));assert!(act(&mut loaded,"riposte"));
    assert_eq!(g.save(),loaded.save());assert_eq!(g.snapshot().hero.specialization,Some(Style::Sentinel));
    assert!(g.snapshot().hero.entity.tags.contains(&"riposte ready".into()));
}
#[test]
fn riposte_halves_one_positive_melee_hit_and_counters_once_with_named_event() {
    for (damage,taken) in [(10,5),(5,3)] {
        let mut g=arena(Style::Sentinel);foe(&mut g,"goblin");assert!(act(&mut g,"riposte"));
        assert_eq!((g.run.as_ref().unwrap().hero.riposte_t,g.run.as_ref().unwrap().hero.special_cd),(20,60));
        hit(&mut g,damage,Src::Mon(0));let r=g.run.as_ref().unwrap();
        assert_eq!((r.hero.hp,r.monsters[0].hp,r.hero.riposte_t),(40-taken,92,0));
        assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),dmg:8,..} if v=="riposte")));
        hit(&mut g,damage,Src::Mon(0));assert_eq!(g.run.as_ref().unwrap().monsters[0].hp,92);
        assert_eq!(g.run.as_ref().unwrap().hero.hp,40-taken-damage);assert!(!act(&mut g,"riposte"));
    }
}
#[test]
fn riposte_excludes_hazards_ranged_reflection_allies_zero_and_expiration() {
    for (kind,src,ally,damage,taken) in [("goblin",Src::Gas,false,10,10),("goblin_archer",Src::Mon(0),false,10,10),
        ("goblin",Src::Reflect(0),false,10,10),("goblin",Src::Mon(0),true,10,10),("goblin",Src::Mon(0),false,0,0)] {
        let mut g=arena(Style::Sentinel);foe(&mut g,kind);assert!(act(&mut g,"riposte"));g.run.as_mut().unwrap().monsters[0].ally=ally;
        hit(&mut g,damage,src);let r=g.run.as_ref().unwrap();assert_eq!((r.hero.hp,r.monsters[0].hp,r.hero.riposte_t),(40-taken,100,20));
    }
    let mut g=arena(Style::Sentinel);foe(&mut g,"goblin");assert!(act(&mut g,"riposte"));
    let h=&mut g.run.as_mut().unwrap().hero;h.tick_statuses_by(20);assert_eq!((h.riposte_t,h.special_cd),(0,40));
    hit(&mut g,10,Src::Mon(0));assert_eq!(g.run.as_ref().unwrap().hero.hp,30);
}
#[test]
fn mirror_has_priority_and_fatal_parries_do_not_counter() {
    let mut g=arena(Style::Sentinel);foe(&mut g,"goblin");assert!(act(&mut g,"riposte"));
    g.run.as_mut().unwrap().hero.mirror_charge=1;hit(&mut g,10,Src::Mon(0));
    let r=g.run.as_ref().unwrap();assert_eq!((r.hero.hp,r.monsters[0].hp,r.hero.riposte_t),(40,90,20));
    let mut g=arena(Style::Sentinel);foe(&mut g,"goblin");assert!(act(&mut g,"riposte"));
    g.run.as_mut().unwrap().hero.hp=5;hit(&mut g,10,Src::Mon(0));
    let r=g.run.as_ref().unwrap();assert_eq!((r.hero.hp,r.monsters[0].hp),(0,100));assert!(r.over.is_some());
}
#[test]
fn hex_reduces_only_its_target_attacks_and_uses_exact_timers_and_snapshot_tag() {
    for damage in [10,1] {
        let mut g=arena(Style::Hexbinder);foe(&mut g,"goblin");foe(&mut g,"goblin");assert!(act(&mut g,"hex"));
        assert_eq!((g.run.as_ref().unwrap().monsters[0].hex_t,g.run.as_ref().unwrap().hero.special_cd),(40,60));
        assert!(g.snapshot().entities.iter().any(|e|e.id==100&&e.tags.contains(&"hexed".into())));
        hit(&mut g,damage,Src::Mon(0));assert_eq!(g.run.as_ref().unwrap().hero.hp,40-(damage-2).max(0));
        assert!(!act(&mut g,"hex"));
        hit(&mut g,10,Src::Mon(1));hit(&mut g,10,Src::Gas);
        assert_eq!(g.run.as_ref().unwrap().hero.hp,20-(damage-2).max(0));
        g.run.as_mut().unwrap().monsters[0].tick_statuses_by(40);
        assert_eq!(g.run.as_ref().unwrap().monsters[0].hex_t,0);
        g.run.as_mut().unwrap().hero.tick_statuses_by(60);assert!(act(&mut g,"hex"));
    }
}
#[test]
fn wrong_style_class_level_and_targets_cannot_mutate_actions() {
    for style in [Style::Sentinel,Style::Hexbinder] {
        let mut g=arena(style);foe(&mut g,"goblin");let wrong=if style==Style::Sentinel {"hex"}else{"riposte"};
        let before=g.save();assert!(!act(&mut g,wrong));assert_eq!(g.save(),before);
        g.run.as_mut().unwrap().hero.level=9;let before=g.save();assert!(!act(&mut g,style.verb()));assert_eq!(g.save(),before);
        g.run.as_mut().unwrap().hero.level=10;g.run.as_mut().unwrap().hero.class=Class::Rogue;
        let before=g.save();assert!(!act(&mut g,style.verb()));assert_eq!(g.save(),before);
    }
    let mut g=arena(Style::Hexbinder);foe(&mut g,"goblin");g.run.as_mut().unwrap().monsters[0].pos=Pos::new(100,100);
    let before=g.save();assert!(!act(&mut g,"hex"));assert_eq!(g.save(),before);
    for distance in [2,9] {
        let mut g=arena(Style::Hexbinder);foe(&mut g,"goblin");
        let r=g.run.as_mut().unwrap();let p=r.hero.pos;
        let sign=if p.x+distance<r.floor.map.w {1}else{-1};
        let target=Pos::new(p.x+sign*distance,p.y);assert!(r.floor.map.in_bounds(target));
        for n in 1..=distance {r.floor.map.set(Pos::new(p.x+sign*n,p.y),crate::tiles::Tile::Floor);}
        r.monsters[0].pos=target;let i=r.floor.map.idx(target);r.floor.map.visible[i]=true;r.floor.map.seen[i]=true;
        if distance==2 {r.floor.map.set(Pos::new(p.x+sign,p.y),crate::tiles::Tile::Wall);}
        assert_eq!(r.floor.map.los(p,target),distance==9);
        assert!(!crate::turn::view(r).foes.is_empty(),"controlled visible target");
        let (r,cx)=g.ctx();let v=crate::turn::view(r);
        assert_eq!(crate::ai::block_reason(r,&cx,&Verb::new("hex"),&v),if distance==2 {"no sight"}else{"too far"});
        let before=g.save();assert!(!act(&mut g,"hex"));assert_eq!(g.save(),before);
    }
}
#[test]
fn selected_styles_leave_every_other_complete_game_and_shared_town_resources_unchanged() {
    let mut s=crate::bloodlines::Session::new(3);s.active=qualified(Class::Fighter);
    s.active.lineage.gold_move(10000,"fixture income");s.add_bloodline().unwrap();s.add_bloodline().unwrap();
    for id in 1..=3 {
        s.select_bloodline(id).unwrap();crate::legacy::ensure(&mut s.active.lineage);
        s.active.lineage.classes.insert("fighter".into(),ClassProg{level:10,xp:0,next:0});
        s.active.lineage.hero_legacy.last_mut().unwrap().best_depth=23;
        let others=serde_json::to_string(&s.others).unwrap();let town=s.active.lineage.town.clone();let gold=s.active.lineage.gold;
        s.set_specialization("sentinel").unwrap();
        assert_eq!(serde_json::to_string(&s.others).unwrap(),others);assert_eq!(s.active.lineage.town,town);assert_eq!(s.active.lineage.gold,gold);
        s.set_specialization("none").unwrap();assert_eq!(serde_json::to_string(&s.others).unwrap(),others);
    }
}
#[test]
fn reflecting_foes_counter_riposte_without_recursion_and_unused_style_memory_keeps_cache_key() {
    let mut g=arena(Style::Sentinel);foe(&mut g,"iron_golem");assert!(act(&mut g,"riposte"));
    hit(&mut g,10,Src::Mon(0));let r=g.run.as_ref().unwrap();
    assert_eq!((r.hero.hp,r.monsters[0].hp,r.hero.riposte_t),(27,100,0));
    assert_eq!(g.events.iter().filter(|e|matches!(e,Ev::Attack{verb:Some(v),..} if v=="riposte")).count(),1);
    let mut g=qualified(Class::Fighter);let key=crate::forecast::lineage_key(&g);
    g.lineage.specializations.insert("caster".into(),Style::Hexbinder);assert_eq!(crate::forecast::lineage_key(&g),key);
    let h=Hero::new(Class::Fighter,Pos::new(1,1));let encoded=serde_json::to_string(&h).unwrap();
    assert!(!encoded.contains("specialization")&&!encoded.contains("special_cd")&&!encoded.contains("riposte_t"));
}
#[test]
fn king_mirror_reflects_the_reactive_hit_and_does_not_contaminate_next_action() {
    let mut g=arena(Style::Sentinel);foe(&mut g,"mirror_king");assert!(act(&mut g,"riposte"));
    let r=g.run.as_mut().unwrap();r.monsters[0].hp=80;r.verb_ring=vec!["riposte".into(),"riposte".into()];
    hit(&mut g,10,Src::Mon(0));let r=g.run.as_ref().unwrap();
    assert_eq!((r.hero.hp,r.monsters[0].hp,r.hero.riposte_t),(27,96,0));
    assert!(r.melee_used);assert_eq!(r.last_hit_verb,None);
    assert!(g.events.iter().any(|e|matches!(e,Ev::Attack{verb:Some(v),dmg:8,..} if v=="mirror")));
}
#[test]
fn styled_eight_hour_absences_match_whole_sliced_reloaded_for_one_and_three_slots() {
    for class in [Class::Fighter,Class::Caster] {for slots in [1,3] {
        let mut base=crate::bloodlines::Session::new(3);base.active=qualified(class);
        base.active.lineage.gold_move(10000,"fixture income");
        for _ in 1..slots {base.add_bloodline().unwrap();}
        crate::tree::grant(&mut base.active.lineage,&["porter","scout"]);
        for id in 1..=slots {
            base.select_bloodline(id).unwrap();crate::legacy::ensure(&mut base.active.lineage);
            let parent=if id%2==1 {class}else{Class::Caster};
            base.active.lineage.class=parent;
            base.active.lineage.classes.insert(parent.name().into(),ClassProg{level:10,xp:0,next:0});
            base.active.lineage.hero_legacy.last_mut().unwrap().best_depth=23;
            let style=if parent==Class::Fighter {Style::Sentinel}else{Style::Hexbinder};
            choose(&mut base.active,style.id()).unwrap();crate::oath::refresh(&mut base.active.lineage);base.send();
            base.active.run.as_mut().unwrap().hero.special_cd=7;
            if let Some(m)=base.active.run.as_mut().unwrap().monsters.first_mut(){m.hex_t=3;}
        }
        base.select_bloodline(1).unwrap();base=crate::bloodlines::Session::load(&base.save()).unwrap();
        let mut whole=base.clone();let expected=whole.run_offline_mode(28800,false,true);
        for reload in [false,true] {
            let mut sliced=base.clone();let mut left=28800;let widths=[1,719,1280];let mut n=0;
            let report=loop {
                let seconds=left.min(widths[n%widths.len()]);let last=left==seconds;
                let report=sliced.run_offline_slice(seconds,last);if last {break report;}
                left-=seconds;n+=1;if reload {sliced=crate::bloodlines::Session::load(&sliced.save()).unwrap();}
            };
            assert!(sliced.save()==whole.save(),"complete state {class:?}/{slots}/reload{reload}");
            assert_eq!(report,expected,"complete report {class:?}/{slots}/reload{reload}");
        }
    }}
}
#[test]
fn automatic_named_row_fires_and_active_style_changes_forecast_key_and_vocabulary() {
    for style in [Style::Sentinel,Style::Hexbinder] {
        let mut g=qualified(style.parent());let key=crate::forecast::lineage_key(&g);
        assert!(!crate::tokens::vocabulary(&g.lineage).verbs.iter().any(|v|v.v==style.verb()));
        choose(&mut g,style.id()).unwrap();assert_ne!(crate::forecast::lineage_key(&g),key);
        assert!(crate::tokens::vocabulary(&g.lineage).verbs.iter().any(|v|v.v==style.verb()));
        let row=g.lineage.rules().rows.iter().find(|r|r.verb.v==style.verb()).unwrap();
        assert!(row.is_pkg());assert_eq!(crate::packages::row_label(row),Some(style.name().into()));
        choose(&mut g,"none").unwrap();assert_eq!(crate::forecast::lineage_key(&g),key);
        let mut g=arena(style);foe(&mut g,"goblin");g.sim=false;g.run.as_mut().unwrap().hero.energy=100;
        let result=g.step(1);assert!(result.events.iter().any(|e|matches!(e,Ev::Rule{verb,..} if verb.v==style.verb())),"style {style:?}, events {:?}, rows {:?}",result.events,g.lineage.rules().rows);
    }
}

#[test]
fn wire_choices_match_refusals_parent_xp_history_and_actual_default_rows() {
    let mut g=qualified(Class::Fighter);
    g.lineage.classes.get_mut("fighter").unwrap().xp=123;
    let base=g.save();let c=g.lineage().class_styles.unwrap();assert_eq!(g.save(),base);
    assert_eq!(c.selected,None);assert!(!c.remove_available);assert!(c.automatic_row);
    for o in c.offers {
        let before=g.save();let result=g.set_specialization(o.id.id());
        assert_eq!(result.is_ok(),o.available);
        if let Err(reason)=result {assert_eq!(Some(reason),o.blocked);assert_eq!(g.save(),before);}
        else {assert_eq!(o.parent,"fighter");assert_eq!(o.level,10);assert_eq!(o.xp,123);
            assert_eq!(o.deepest,23);assert_eq!(o.next,if o.level>=crate::engine::MAX_LEVEL {0}else{crate::hero::xp_to_next(o.level)});
            assert_eq!(o.tactic,row(&g.lineage).unwrap());assert_eq!(o.duration_ticks,20);assert_eq!(o.cooldown_ticks,60);
            g.set_specialization("none").unwrap();}
    }
    assert_eq!(g.save(),base);
    g.lineage.classes.get_mut("fighter").unwrap().level=9;
    let o=offers(&g.lineage,false).offers.remove(0);assert_eq!(o.required_level,10);assert_eq!(o.blocked.as_deref(),Some("reach class level10"));
    g.lineage.classes.get_mut("fighter").unwrap().level=10;g.lineage.hero_legacy.last_mut().unwrap().best_depth=22;
    let o=offers(&g.lineage,false).offers.remove(0);assert_eq!(o.required_depth,23);assert_eq!(o.blocked.as_deref(),Some("reach D23"));
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=23;g.set_specialization("sentinel").unwrap();g.send();
    let c=g.lineage().class_styles.unwrap();assert_eq!(c.selected,Some(Style::Sentinel));assert!(!c.remove_available);
    for o in c.offers {assert!(!o.available);assert_eq!(o.blocked.as_deref(),Some("hero away"));}
    assert_eq!(c.remove_blocked.as_deref(),Some("hero away"));
}
#[test]
fn live_roster_uses_snapshot_style_and_literal_policy_is_reported_without_claiming_auto_row() {
    let mut s=crate::bloodlines::Session::new(1);s.active=qualified(Class::Fighter);
    s.active.set_specialization("sentinel").unwrap();s.active.send();
    s.active.lineage.specializations.clear();
    assert_eq!(s.lineage().hero_slots[0].specialization,Some(Style::Sentinel));
    let mut g=qualified(Class::Caster);crate::packages::make_literal(&mut g.lineage);
    let before=g.lineage.rules().clone();g.set_specialization("hexbinder").unwrap();
    let c=g.lineage().class_styles.unwrap();assert_eq!(c.selected,Some(Style::Hexbinder));assert!(!c.automatic_row);
    assert_eq!(g.lineage.rules(),&before);
}

#[test]
fn removal_offer_allows_clearing_unqualified_saved_choice_without_granting_ability() {
    let mut g=qualified(Class::Fighter);g.set_specialization("sentinel").unwrap();
    g.lineage.classes.get_mut("fighter").unwrap().level=9;
    let c=offers(&g.lineage,false);assert_eq!(c.selected,None);assert!(c.offers[0].selected);assert!(c.remove_available);
    assert!(!c.offers[0].available);let before=g.lineage.classes.clone();
    g.set_specialization("none").unwrap();assert!(g.lineage.specializations.is_empty());assert_eq!(g.lineage.classes,before);
}
