//! Controlled fixtures test real purchases/verbs/damage, not earned campaigns.
use super::*;
use crate::{monster::Monster,geom::Pos,rules::Verb,turn::Src,hero::Class,rng::Rng};
fn rich()->Game {
    let mut g=Game::new_resident(3);ensure(&mut g.lineage);
    g.lineage.bloodline.as_mut().unwrap().points=814;g.lineage.best_depth=34;
    crate::packages::recompile(&mut g.lineage);crate::oath::refresh(&mut g.lineage);g
}
fn purchase(g:&mut Game,id:&str) {
    let n=NODES.iter().find(|n|n.id==id).unwrap();
    if let Some(parent)=n.parent {if current(&g.lineage).unwrap().upgrades.get(parent).is_none_or(|r|*r==0){purchase(g,parent);}}
    buy(g,id).unwrap();
}
fn arena(ids:&[&str])->Game {
    let mut g=rich();for id in ids {purchase(&mut g,id);}g.send();g.sim=true;
    let r=g.run.as_mut().unwrap();r.hero.hp=1;r.hero.max_hp=40;r.hero.max_hp_base=40;
    r.hero.class=Class::Fighter;r.hero.level=10;r.hero.gift=Default::default();r.hero.inv.clear();
    r.monsters.clear();r.rng=Rng::new(5);g.events.clear();g
}
fn foe(g:&mut Game)->usize {
    let r=g.run.as_ref().unwrap();
    let at=r.hero.pos.neighbours8().into_iter().find(|p|r.floor.map.passable(*p)).unwrap();
    let mut m=Monster::spawn(100,"goblin",at,1);m.awake=true;m.hp=20;m.max_hp=20;
    let r=g.run.as_mut().unwrap();let i=r.monsters.len();r.monsters.push(m);i
}
fn act(g:&mut Game,verb:&str,arg:Option<&str>)->bool {
    let(r,mut cx)=g.ctx();let v=crate::turn::view(r);
    crate::ai::try_verb(r,&mut cx,&Verb{v:verb.into(),a:arg.map(str::to_owned)},&v)
}
fn same_save(g:&Game,expected:&str) {
    let actual=g.save();if actual==expected {return;}
    fn differences(a:&serde_json::Value,b:&serde_json::Value,path:&str,out:&mut Vec<String>) {
        if a==b||out.len()>20{return;}
        match(a,b) {
            (serde_json::Value::Object(a),serde_json::Value::Object(b))=>{
                for key in a.keys().chain(b.keys()).collect::<std::collections::BTreeSet<_>>() {
                    differences(a.get(key).unwrap_or(&serde_json::Value::Null),b.get(key).unwrap_or(&serde_json::Value::Null),&format!("{path}/{key}"),out);
                }
            }
            (serde_json::Value::Array(a),serde_json::Value::Array(b)) if a.len()==b.len()=>{
                for(i,(a,b))in a.iter().zip(b).enumerate(){differences(a,b,&format!("{path}/{i}"),out);}
            }
            _=>out.push(path.into()),
        }
    }
    let mut paths=Vec::new();differences(&serde_json::from_str(expected).unwrap(),&serde_json::from_str(&actual).unwrap(),"",&mut paths);
    panic!("complete save mismatch: {paths:?}");
}
#[test]
fn twelve_offers_keep_old_rank_prices_and_require_earned_depth_and_parents() {
    let mut g=rich();let o=offers(&g.lineage,false);assert_eq!(o.len(),12);
    assert_eq!(o.iter().filter(|u|u.cap==3).count(),3);
    for id in IDS {for price in [3,6,9] {
        assert_eq!(offers(&g.lineage,false).iter().find(|u|u.id==id).unwrap().price,price);buy(&mut g,id).unwrap();
    }}
    assert_eq!((current(&g.lineage).unwrap().points,current(&g.lineage).unwrap().spent),(760,54));
    let before=g.save();assert!(buy(&mut g,"health").is_err());assert_eq!(g.save(),before);
    let mut g=rich();let before=g.save();assert!(buy(&mut g,"restoration").is_err());assert_eq!(g.save(),before);
    buy(&mut g,"health").unwrap();g.lineage.best_depth=7;
    let before=g.save();assert!(buy(&mut g,"restoration").is_err());assert_eq!(g.save(),before);
    g.lineage.best_depth=8;buy(&mut g,"restoration").unwrap();
    let before=g.save();assert!(buy(&mut g,"mending").is_err());assert_eq!(g.save(),before);
    g.lineage.hero_legacy.last_mut().unwrap().best_depth=18;g.lineage.best_depth=1;
    buy(&mut g,"mending").unwrap();assert_eq!(deepest(&g.lineage),18);
}
#[test]
fn all_six_fork_conflicts_refuse_without_saved_mutation() {
    for (a,b) in [("mending","renewal"),("venom","debilitate"),("fireward","brace")] {
        for (chosen,other) in [(a,b),(b,a)] {
            let mut g=rich();purchase(&mut g,chosen);let before=g.save();
            let offer=offers(&g.lineage,false).into_iter().find(|u|u.id==other).unwrap();
            assert!(!offer.affordable&&offer.blocked.unwrap().starts_with("Chosen "));
            assert!(buy(&mut g,other).is_err());assert_eq!(g.save(),before);
        }
    }
}
#[test]
fn respec_preview_uses_the_same_checked_eligibility_and_actual_refund_as_purchase() {
    let mut g=rich();
    let before=g.save();let offer=respec_offer(&g.lineage,false);
    assert!(!offer.available);assert_eq!(offer.points_after,None);same_save(&g,&before);
    purchase(&mut g,"mending");
    let before=g.save();let offer=respec_offer(&g.lineage,false);
    assert_eq!((offer.refund,offer.points_after,offer.available),(57,Some(814),true));
    assert_eq!(g.lineage().legacy_respec,Some(offer.clone()));same_save(&g,&before);
    assert!(!respec_offer(&g.lineage,true).available);
    g.lineage.bloodline.as_mut().unwrap().points=u32::MAX;
    let before=g.save();let offer=respec_offer(&g.lineage,false);
    assert_eq!((offer.available,offer.points_after),(false,None));
    assert_eq!(respec(&mut g),Err(offer.blocked.unwrap()));same_save(&g,&before);
}
#[test]
fn free_respec_refunds_actual_spent_and_rebuilds_exactly_without_other_changes() {
    let mut g=rich();for id in IDS {for _ in 0..CAP {buy(&mut g,id).unwrap();}}
    purchase(&mut g,"renewal");purchase(&mut g,"venom");purchase(&mut g,"brace");
    let allocated=g.save();let mut expected=g.clone();let h=expected.lineage.bloodline.as_mut().unwrap();
    h.points+=h.spent;h.spent=0;h.upgrades.clear();respec(&mut g).unwrap();assert_eq!(g.save(),expected.save());
    for id in IDS {for _ in 0..CAP {buy(&mut g,id).unwrap();}}
    purchase(&mut g,"renewal");purchase(&mut g,"venom");purchase(&mut g,"brace");assert_eq!(g.save(),allocated);
}
#[test]
fn refusal_for_unknown_poor_away_houseless_or_overflow_keeps_whole_save() {
    let mut g=rich();let before=g.save();assert!(buy(&mut g,"unknown").is_err());assert_eq!(g.save(),before);
    g.lineage.bloodline.as_mut().unwrap().points=0;
    let before=g.save();assert!(buy(&mut g,"health").is_err());assert!(respec(&mut g).is_err());assert_eq!(g.save(),before);
    let mut g=rich();buy(&mut g,"health").unwrap();g.lineage.town.home=Some(false);
    let before=g.save();assert!(buy(&mut g,"damage").is_err());assert!(respec(&mut g).is_err());assert_eq!(g.save(),before);
    g.lineage.town.home=Some(true);g.send();let before=g.save();
    assert!(buy(&mut g,"damage").is_err());assert!(respec(&mut g).is_err());assert_eq!(g.save(),before);
    let mut g=rich();buy(&mut g,"health").unwrap();g.lineage.bloodline.as_mut().unwrap().points=u32::MAX;
    let before=g.save();assert!(respec(&mut g).is_err());assert_eq!(g.save(),before);
    g.lineage.bloodline.as_mut().unwrap().spent=u32::MAX;
    let before=g.save();assert!(buy(&mut g,"damage").is_err());assert_eq!(g.save(),before);
}
#[test]
fn selected_slot_respec_does_not_change_shared_gold_or_any_other_game() {
    let mut s=crate::bloodlines::Session::new(3);s.active=rich();s.active.lineage.gold_move(10000,"fixture income");
    s.add_bloodline().unwrap();s.add_bloodline().unwrap();
    for id in [1,2,3] {
        s.select_bloodline(id).unwrap();ensure(&mut s.active.lineage);s.active.lineage.best_depth=34;
        s.active.lineage.bloodline.as_mut().unwrap().points=814;purchase(&mut s.active,"fireward");
        let others=serde_json::to_value(&s.others).unwrap();let town=s.active.lineage.town.clone();let gold=s.active.lineage.gold;
        s.respec_legacy().unwrap();assert_eq!(serde_json::to_value(&s.others).unwrap(),others);
        assert_eq!(s.active.lineage.town,town);assert_eq!(s.active.lineage.gold,gold);
    }
}
#[test]
fn old_nine_ranks_and_migration_keep_effects_and_omit_new_live_metadata() {
    let mut g=rich();for id in IDS {for _ in 0..CAP {buy(&mut g,id).unwrap();}}
    let raw=g.save();let loaded=Game::load(&raw).unwrap();same_save(&loaded,&raw);
    let mut h=Hero::new(Class::Fighter,Pos::new(0,0));let base=h.clone();apply(&g.lineage,&mut h);
    assert_eq!((h.hp-base.hp,h.str_bonus-base.str_bonus,h.legacy_armour),(9,3,3));assert_eq!(h.legacy_effects,0);
    assert!(!serde_json::to_string(&h).unwrap().contains("legacy_effects"));
    let b=current(&g.lineage).unwrap().clone();g.lineage.bloodline=None;
    let last=g.lineage.hero_legacy.last_mut().unwrap();last.points=b.points;last.spent=b.spent;last.upgrades=b.upgrades.clone();
    let loaded=Game::load(&g.save()).unwrap();assert_eq!(current(&loaded.lineage),Some(&b));
}
#[test]
fn restoration_and_mending_change_real_rest_and_potion_healing() {
    for (ids,expected) in [(&[][..],4),(&["restoration"][..],5)] {
        let mut g=arena(ids);assert!(act(&mut g,"rest",None));assert_eq!(g.run.as_ref().unwrap().hero.hp-1,expected);
    }
    for (ids,expected) in [(&[][..],20),(&["mending"][..],25)] {
        let mut g=arena(ids);let mut potion=crate::item::Item::new(900,"heal");potion.known=true;
        g.run.as_mut().unwrap().hero.inv.push(potion);assert!(act(&mut g,"drink",Some("heal")));
        assert_eq!(g.run.as_ref().unwrap().hero.hp-1,expected);
    }
}
#[test]
fn renewal_heals_only_natural_hostile_hero_kills_once_and_caps_at_max_hp() {
    for (ally,neutral,summoned,source,expected) in [(false,false,false,Src::Hero{ranged:false},1),
        (true,false,false,Src::Hero{ranged:false},0),(false,true,false,Src::Hero{ranged:false},0),
        (false,false,true,Src::Hero{ranged:false},0),(false,false,false,Src::Fire,0)] {
        let mut g=arena(&["renewal"]);let i=foe(&mut g);
        let m=&mut g.run.as_mut().unwrap().monsters[i];m.ally=ally;m.neutral=neutral;m.summoned=summoned;
        {let(r,mut cx)=g.ctx();assert!(crate::turn::damage_monster(r,&mut cx,i,100,&source));}
        assert_eq!(g.run.as_ref().unwrap().hero.hp,1+expected);
        {let(r,mut cx)=g.ctx();assert!(!crate::turn::damage_monster(r,&mut cx,i,100,&source));}
        assert_eq!(g.run.as_ref().unwrap().hero.hp,1+expected);
    }
    let mut g=arena(&["renewal"]);foe(&mut g);g.run.as_mut().unwrap().hero.hp=40;
    let(r,mut cx)=g.ctx();crate::turn::damage_monster(r,&mut cx,0,100,&Src::Hero{ranged:false});assert_eq!(r.hero.hp,40);
}
#[test]
fn control_debilitate_and_venom_modify_owned_verbs_without_granting_them() {
    for (ids,ticks) in [(&[][..],10),(&["control"][..],15)] {
        let mut g=arena(ids);foe(&mut g);assert!(act(&mut g,"shield_bash",None));
        assert_eq!(g.run.as_ref().unwrap().monsters[0].stun,ticks);
    }
    for (ids,ticks) in [(&[][..],30),(&["control"][..],35),(&["debilitate"][..],50)] {
        let mut g=arena(ids);foe(&mut g);assert!(!act(&mut g,"slow",None));
        let h=&mut g.run.as_mut().unwrap().hero;h.class=Class::Caster;h.level=5;
        assert!(act(&mut g,"slow",None));assert_eq!(g.run.as_ref().unwrap().monsters[0].slow_t,ticks);
    }
    for (ids,dmg) in [(&[][..],2),(&["venom"][..],3)] {
        let mut g=arena(ids);foe(&mut g);let mut poison=crate::item::Item::new(900,"poison");poison.known=true;
        g.run.as_mut().unwrap().hero.inv.push(poison);assert!(!act(&mut g,"throw",Some("poison,nearest")));
        g.lineage.unlocks.insert("throw".into());assert!(act(&mut g,"throw",Some("poison,nearest")));
        assert_eq!(g.run.as_ref().unwrap().monsters[0].poison,(dmg,40));
    }
}
#[test]
fn warding_reductions_change_actual_damage_with_zero_floor_and_hp_threshold() {
    for (ids,src,hp,dmg,expected) in [(&[][..],Src::Gas,40,10,10),(&["clear_lungs"][..],Src::Gas,40,10,8),
        (&["clear_lungs"][..],Src::Poison,40,1,0),(&["clear_lungs"][..],Src::Burst,40,10,8),(&["fireward"][..],Src::Fire,40,10,5),
        (&["brace"][..],Src::Fire,9,10,8),(&["brace"][..],Src::Fire,10,10,10)] {
        let mut g=arena(ids);g.run.as_mut().unwrap().hero.hp=hp;
        let(r,mut cx)=g.ctx();crate::turn::damage_hero(r,&mut cx,dmg,&src);assert_eq!(hp-r.hero.hp,expected);
    }
}
#[test]
fn new_effects_keep_whole_sliced_and_reloaded_eight_hour_state_and_reports_exact() {
    for slots in [1,3] {
        let mut base=crate::bloodlines::Session::new(3);base.active=rich();
        base.active.lineage.gold_move(10000,"fixture income");
        for _ in 1..slots {base.add_bloodline().unwrap();}
        crate::tree::grant(&mut base.active.lineage,&["porter","scout"]);
        for id in 1..=slots {
            base.select_bloodline(id).unwrap();ensure(&mut base.active.lineage);
            base.active.lineage.best_depth=34;base.active.lineage.bloodline.as_mut().unwrap().points=814;
            for leaf in ["mending","venom","brace"] {purchase(&mut base.active,leaf);}
            crate::packages::recompile(&mut base.active.lineage);crate::oath::refresh(&mut base.active.lineage);
            base.send();
            assert_ne!(base.active.run.as_ref().unwrap().hero.legacy_effects,0);
        }
        base.select_bloodline(1).unwrap();
        base=crate::bloodlines::Session::load(&base.save()).unwrap();
        let mut whole=base.clone();let expected=whole.run_offline_mode(28800,false,true);
        for (widths,reload) in [(&[1800][..],false),(&[1,719,1280][..],false),(&[1,719,1280][..],true)] {
            let mut sliced=base.clone();let mut left=28800;let mut i=0;
            let report=loop {
                let seconds=left.min(widths[i%widths.len()]);let last=seconds==left;
                let report=sliced.run_offline_slice(seconds,last);
                if last {break report;}
                assert!(report.slice_pending);left-=seconds;i+=1;
                if reload {sliced=crate::bloodlines::Session::load(&sliced.save()).unwrap();}
            };
            assert!(sliced.save()==whole.save(),"complete u64-safe state slots{slots}/reload{reload}");
            assert_eq!(report,expected,"complete report slots{slots}/reload{reload}");
        }
    }
}
#[test]
fn sent_perks_survive_save_and_history_without_reading_later_camp_allocations() {
    let mut g=arena(&["mending","venom","brace"]);g.sim=false;let mask=g.run.as_ref().unwrap().hero.legacy_effects;
    assert_eq!(mask,RESTORATION|MENDING|CONTROL|VENOM|CLEAR_LUNGS|BRACE);
    let raw=g.save();let mut loaded=Game::load(&raw).unwrap();same_save(&loaded,&raw);
    g.lineage.bloodline.as_mut().unwrap().upgrades.clear();loaded.lineage.bloodline.as_mut().unwrap().upgrades.clear();
    assert!(act(&mut g,"rest",None));assert!(act(&mut loaded,"rest",None));
    assert_eq!(g.run.as_ref().unwrap().hero.hp,6);same_save(&loaded,&g.save());
    g.step(11);assert!(g.history.iter().all(|(r,_)|r.hero.legacy_effects==mask));
}

#[test]
fn earned_legacy_exit_and_batch_match_points_for_each_exit_and_skip_simulations() {
    use crate::engine::ExitTier;
    for tier in [ExitTier::Bank, ExitTier::Return, ExitTier::Death] {
        for sim in [false,true] {
            let mut g=Game::new_resident(7);
            let before=current(&g.lineage).unwrap().points;
            g.send();g.sim=sim;
            let run=g.run.as_mut().unwrap();run.max_depth=8;run.over=Some(tier);
            g.finish_run().unwrap();
            let gain=current(&g.lineage).unwrap().points-before;
            assert_eq!(gain,if sim {0}else{9});
            assert_eq!(g.last_exit.as_ref().unwrap().legacy_earned,gain);
            assert_eq!(g.batch.legacy_earned,gain);
            assert!(g.finish_run().is_none());
            assert_eq!(current(&g.lineage).unwrap().points,before+gain);
        }
    }
}

/// Blind ad71e72 (B: Legacy piled to 80, "could not buy from it"): the camp rest's clock readies
/// the next run at its first tick while the hero rests at home; a purchase then is no absence, and
/// the readied hero carries it. A send (the rest skipped) is away.
#[test]
fn a_resting_hero_buys_legacy_and_the_readied_run_carries_it() {
    let mut g=rich();g.start_run(Some(7));g.finish_run();
    assert!(g.lineage.rest_left>0,"home from a run, resting");
    g.step(1);
    let run=g.run.as_ref().expect("the rest clock readied a run");assert_eq!(run.turn,0);
    let base=run.hero.clone();
    assert!(!away(&g));assert!(offers(&g.lineage,away(&g)).iter().any(|u|u.id=="health"&&u.affordable));
    buy(&mut g,"health").unwrap();buy(&mut g,"damage").unwrap();buy(&mut g,"armour").unwrap();
    let hero=&g.run.as_ref().unwrap().hero;
    assert_eq!((hero.max_hp,hero.max_hp_base,hero.hp),(base.max_hp+3,base.max_hp_base+3,base.hp+3));
    assert_eq!(hero.atk(),(base.atk().0+1,base.atk().1+1));assert_eq!(hero.legacy_armour,base.legacy_armour+1);
    let sent=g.sent_state.as_ref().unwrap().lineage.bloodline.as_ref().unwrap();assert_eq!(sent.upgrades["health"],1);
    respec(&mut g).unwrap();
    let hero=&g.run.as_ref().unwrap().hero;assert_eq!((hero.max_hp,hero.atk(),hero.legacy_armour),(base.max_hp,base.atk(),base.legacy_armour),"a respec refits too");
    g.send();assert!(away(&g),"a send skips the rest: away");let before=g.save();assert!(buy(&mut g,"health").is_err());assert_eq!(before,g.save());
}

/// Blind b58b431 (A: Legacy 146 unspendable — the scout kept the hero away, and a purchase waited for a rest the player never saw):
/// away, an upgrade the points buy is offered for the next run; `buy` still refuses, `buy_next` takes it — the run under way keeps
/// the hero it sent, the next send carries it.
#[test]
fn away_a_legacy_upgrade_is_bought_for_the_next_run() {
    let mut g=rich();g.send();assert!(away(&g));
    let health=offers(&g.lineage,true).into_iter().find(|u|u.id=="health").unwrap();
    assert!(health.next_run&&!health.affordable&&health.blocked.as_deref()==Some("Hero away"),"{health:?}");
    assert!(offers(&g.lineage,true).iter().all(|u|!u.next_run||u.rank<u.cap),"a complete upgrade is never offered");
    assert!(offers(&g.lineage,false).iter().all(|u|!u.next_run),"home, nothing waits for a next run");
    let before=g.save();assert!(buy(&mut g,"health").is_err());assert_eq!(before,g.save());
    let hero=g.run.as_ref().unwrap().hero.clone();let sent=g.sent_state.as_ref().map(|s|s.lineage.bloodline.clone());
    let points=current(&g.lineage).unwrap().points;
    buy_next(&mut g,"health").unwrap();
    assert_eq!(current(&g.lineage).unwrap().points,points-3);assert_eq!(current(&g.lineage).unwrap().upgrades["health"],1);
    let now=&g.run.as_ref().unwrap().hero;
    assert_eq!((now.max_hp,now.hp,now.str_bonus),(hero.max_hp,hero.hp,hero.str_bonus),"the run under way keeps the hero it sent");
    assert_eq!(g.sent_state.as_ref().map(|s|s.lineage.bloodline.clone()),sent,"its replays keep the camp it left");
    // a refusal (an unbuyable fork) keeps the save whole
    let before=g.save();assert!(buy_next(&mut g,"mending").is_err());assert_eq!(before,g.save());
    // the next send carries it
    g.run.as_mut().unwrap().over=Some(crate::engine::ExitTier::Bank);g.finish_run();
    let base={let mut f=rich();f.send();f.run.as_ref().unwrap().hero.max_hp_base};
    g.send();assert_eq!(g.run.as_ref().unwrap().hero.max_hp_base,base+3,"the next heir's send carries the upgrade");
}
