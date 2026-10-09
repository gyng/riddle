//! A town session holds independent active bloodlines. The selected game is the
//! authoritative town wallet; other slots never spend against a stale copy.
use crate::{Game, engine::LineageState, wire::{Lineage, HeroSlot, StepResult, Advance, ReturnReport}};
use serde::Serialize;
use std::{collections::BTreeMap, ops::{Deref,DerefMut}};
pub const SLOT_CAP: usize = 3;
pub const SLOT_PRICE: i32 = 250;
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Session {
    pub bloodlines_v: u32,
    pub selected: u32,
    pub next_id: u32,
    #[serde(default)]
    pub elapsed_remainder_ms:u64,
    #[serde(flatten)]
    pub active: Game,
    pub others: BTreeMap<u32, Game>,
}
impl Deref for Session { type Target=Game; fn deref(&self)->&Game { &self.active } }
impl DerefMut for Session { fn deref_mut(&mut self)->&mut Game { &mut self.active } }
/// These are town resources. Hero policy, facts, XP, gear, clocks and progression
/// are deliberately not copied when changing bloodlines.
fn town_from(source: &LineageState, dest: &mut LineageState) {
    dest.gold=source.gold;
    dest.gold_tally.clone_from(&source.gold_tally);
    if dest.gold_ledger.len()!=source.gold_ledger.len()||dest.gold_ledger.last()!=source.gold_ledger.last(){dest.gold_ledger=source.gold_ledger.clone();}
    dest.town.clone_from(&source.town);
    dest.tree.hired.clone_from(&source.tree.hired);
    dest.tree.done.clone_from(&source.tree.done);
    dest.tree.paused.clone_from(&source.tree.paused);
    dest.tree.chest=source.tree.chest; dest.tree.ledger=source.tree.ledger;
    dest.tree.acts.clone_from(&source.tree.acts);
    dest.tree.ranks.clone_from(&source.tree.ranks);
}
impl Session {
    pub fn new(seed:u64)->Self { Self { bloodlines_v:1, selected:1, next_id:2, elapsed_remainder_ms:0, active:Game::new(seed), others:BTreeMap::new() } }
    pub fn load(text:&str)->Result<Self,String> {
        let v:serde_json::Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
        if v.get("bloodlines_v").is_none() { return Ok(Self { active:Game::load(text)?, ..Self::new(0) }); }
        let selected=v["selected"].as_u64().and_then(|n|u32::try_from(n).ok()).ok_or("invalid selection")?;
        let next_id=v["next_id"].as_u64().and_then(|n|u32::try_from(n).ok()).ok_or("invalid next bloodline")?;
        if v["bloodlines_v"]!=1 || selected==0 {return Err("invalid bloodlines".into());}
        let mut others=BTreeMap::new();
        for (id,value) in v["others"].as_object().ok_or("invalid bloodlines")? {
            let id:u32=id.parse().map_err(|_|"invalid bloodline")?;
            if id==0 || id==selected {return Err("invalid bloodline".into());}
            others.insert(id,Game::load(&value.to_string())?);
        }
        if others.len()>=SLOT_CAP || next_id<=selected || others.keys().any(|id|*id>=next_id) {return Err("invalid bloodlines".into());}
        let mut s=Self {bloodlines_v:1,selected,next_id,elapsed_remainder_ms:v["elapsed_remainder_ms"].as_u64().unwrap_or(0),active:Game::load(text)?,others};
        s.active.lineage.bloodline_id=selected;for (id,g) in &mut s.others {g.lineage.bloodline_id= *id;}
        Ok(s)
    }
    pub fn save(&self)->String { serde_json::to_string(self).unwrap_or_default() }
    pub fn add_bloodline(&mut self)->Result<(),String> {
        if self.active.lineage.town.home==Some(false) { return Err("build a house first".into()); }
        if self.others.len()+1>=SLOT_CAP { return Err("bloodlines full".into()); }
        if self.active.lineage.gold<SLOT_PRICE { return Err("more gold needed".into()); }
        let id=self.next_id;
        if self.active.lineage.town.shared_night_runs.is_none() { self.active.lineage.town.shared_night_runs=Some(self.active.lineage.night_runs); }
        let mut g=Game::new_resident(self.active.lineage.seed ^ (id as u64).wrapping_mul(0x9e3779b97f4a7c15));
        g.lineage.bloodline_id=id;g.lineage.clock_s=self.active.lineage.clock_s;
        // A new resident uses an unworn cosmetic. Existing/saved looks never change.
        let worn:Vec<_>=std::iter::once(&self.active).chain(self.others.values())
            .map(|g|g.lineage.look.as_deref().unwrap_or_else(||g.lineage.class.default_look())).collect();
        g.lineage.look=crate::hero::Class::LOOKS.iter().find(|look|!worn.contains(look)).map(|look|(*look).into());
        self.active.lineage.gold_move(-SLOT_PRICE,&format!("bloodline {id}"));
        town_from(&self.active.lineage,&mut g.lineage);
        self.others.insert(id,g); self.next_id+=1;
        Ok(())
    }
    pub fn select_bloodline(&mut self,id:u32)->Result<(),String> {
        if id==self.selected { return Ok(()); }
        let mut next=self.others.remove(&id).ok_or("unknown bloodline")?;
        town_from(&self.active.lineage,&mut next.lineage);
        let prev=std::mem::replace(&mut self.active,next);
        self.others.insert(self.selected,prev); self.selected=id;
        Ok(())
    }
    pub fn lineage(&self)->Lineage {
        let mut l=self.active.lineage();
        l.selected_loadout=self.active.loadout.clone();
        let slot=|id:u32,g:&Game| {
            // (a run the rest clock readied but has not begun is no delve: the hero still rests)
            let live=g.run.as_ref().filter(|r|r.over.is_none()&&(r.turn>0||g.lineage.rest_left==0));
            let xp=g.lineage.classes.get(g.lineage.class.name());
            HeroSlot { specialization:live.map_or_else(||crate::specialization::current(&g.lineage),|r|r.hero.specialization), look:g.lineage.look.clone().unwrap_or_else(||g.lineage.class.default_look().into()), hero_name:crate::legacy::hero_identity(g.lineage.seed,g.lineage.heir,id), build:crate::packages::build_name(&g.lineage), id, name:format!("Bloodline {id}"), heir:g.lineage.heir, class:g.lineage.class.name().into(),
                level:xp.map_or(1,|x|x.level), xp:xp.map_or(0,|x|x.xp), next:xp.map(|x|if x.level>=crate::engine::MAX_LEVEL {0}else{crate::hero::xp_to_next(x.level)}),
                state:if live.is_some(){"live"}else if g.waits(){"waits"}else{"rests"}.into(),
                live:g.live_run(), rest_s:g.lineage.rest_left as f64 /10.0,
                legacy:g.lineage.bloodline.clone().unwrap_or_default(),
                chronicle:g.lineage.chronicle.clone(),
                notice:crate::legacy::offers(&g.lineage,crate::legacy::away(g)).iter().any(|u|u.affordable),
            }
        };
        l.hero_slots=self.others.iter().map(|(id,g)|slot(*id,g)).chain(std::iter::once(slot(self.selected,&self.active))).collect();
        l.hero_slots.sort_by_key(|s|s.id); if self.active.lineage.town.home==Some(false) { l.hero_slots.clear(); } l.selected_bloodline=self.selected;
        l.bloodline_price=SLOT_PRICE; l.bloodline_cap=SLOT_CAP as u32;
        l
    }
    fn background(&mut self,ms:u64) {
        for g in self.others.values_mut() {
            town_from(&self.active.lineage,&mut g.lineage);
            g.advance(ms);
            town_from(&g.lineage,&mut self.active.lineage);
        }
    }
    pub fn step(&mut self,turns:u32)->StepResult {
        let before=self.active.run.as_ref().map_or(0,|r|r.turn);
        let r=self.active.step(turns);
        let ticks=r.snapshot.turn.saturating_sub(before) as u64;
        if !self.others.is_empty(){
            // A watched tick advances every active hero on the same town clock.
            // The original single-hero watch keeps its historical paused clock.
            let clock_ticks=self.active.advance_rem.1+ticks;
            self.active.advance_rem.1=clock_ticks%10;self.active.lineage.clock_s+=clock_ticks/10;
            self.background(ticks*100);
        }
        r
    }
    pub fn advance(&mut self,ms:u64)->Advance {
        if self.others.is_empty(){return self.active.advance(ms);}
        let ms=ms+self.elapsed_remainder_ms;self.elapsed_remainder_ms=ms%100;
        let mut remaining=ms/100*100;
        let mut out=Advance::default();
        let mut ids:Vec<_>=self.others.keys().copied().chain(std::iter::once(self.selected)).collect();ids.sort_unstable();
        while remaining>0 {
            // Once every manual hero is home, skip idle time in one operation.
            let waiting=self.active.waits()&&self.others.values().all(Game::waits);
            let quantum=if waiting {remaining}else{100};
            for id in &ids {
                if *id==self.selected {let r=self.active.advance(quantum);out.ended.extend(r.ended);}
                else {let g=self.others.get_mut(id).expect("slot");town_from(&self.active.lineage,&mut g.lineage);g.advance(quantum);town_from(&g.lineage,&mut self.active.lineage);}
            }
            remaining-=quantum;
        }
        out.live=self.active.live_run();out
    }
    fn victory_knowledge(g:&Game,bests:&[String])->Vec<crate::wire::BossKnowledge> {
        bests.iter().filter_map(|best|best.strip_prefix("boss:").map(str::trim)).map(|boss| {
            let prefix=format!("foe:{boss}:");
            crate::wire::BossKnowledge { boss:boss.into(),
                facts:g.lineage.facts.iter().filter(|f|f.starts_with(&prefix)).cloned().collect(),
                ledger:g.lineage.ledger().into_iter().find(|r|r.kind==boss),
                wall:crate::oath::walls(&g.lineage).into_iter().find(|w|w.boss==boss),
            }
        }).collect()
    }
    pub fn run_offline_mode(&mut self,seconds:u64,full:bool,last:bool)->ReturnReport {
        if self.others.is_empty(){return if full {self.active.run_offline(seconds)}else if last{crate::offline::run_offline_quick(&mut self.active,seconds)}else{crate::offline::run_offline_counts(&mut self.active,seconds)};}
        self.run_offline_multi(seconds,full,true,last)
    }
    pub fn run_offline_slice(&mut self, seconds:u64, last:bool)->ReturnReport {
        if self.others.is_empty() { return crate::offline::run_offline_slice(&mut self.active,seconds,last); }
        self.run_offline_multi(seconds,false,last,last)
    }
    fn run_offline_multi(&mut self,seconds:u64,full:bool,final_slice:bool,with_stall:bool)->ReturnReport {
        let begin=|g:&mut Game|{
            if g.offline_absence.is_none() { g.offline_absence=Some(crate::offline::begin_absence(g)); }
            let a=g.offline_absence.as_mut().unwrap();a.elapsed_s=a.elapsed_s.saturating_add(seconds);
        };
        begin(&mut self.active);
        for g in self.others.values_mut() { town_from(&self.active.lineage,&mut g.lineage);begin(g); }
        self.advance(seconds.saturating_mul(1000));
        // Guide moves during an absence replace the quote captured at its start.
        // Keep the saved continuation current for every independently running slot.
        self.active.offline_absence.as_mut().unwrap().passage = self.active.passage;
        for g in self.others.values_mut() { g.offline_absence.as_mut().unwrap().passage = g.passage; }
        self.active.lineage.in_absence=true;
        for g in self.others.values_mut() { g.lineage.in_absence=true; }
        if !final_slice { return ReturnReport { slice_pending:true,..Default::default() }; }
        if self.active.offline_absence.as_ref().is_some_and(|a|a.elapsed_s>0) {
            let mut ids:Vec<_>=self.others.keys().copied().chain(std::iter::once(self.selected)).collect();ids.sort_unstable();
            for id in ids {
                if id==self.selected { crate::offline::finish_return_run(&mut self.active); }
                else {
                    let g=self.others.get_mut(&id).unwrap();town_from(&self.active.lineage,&mut g.lineage);
                    crate::offline::finish_return_run(g);town_from(&g.lineage,&mut self.active.lineage);
                }
            }
        }
        let finish=|g:&mut Game,full:bool,with_stall:bool|{
            let before=g.offline_absence.take().expect("absence initialized");
            g.offline=false;
            let mut r=crate::offline::report_with(g,before.elapsed_s,&before.facts_before,&before.class,before.rank_before,false,full,with_stall);
            r.grew=crate::town::grew(&before.grew_before,&crate::town::snap(&g.lineage));
            r.workers=crate::tree::report_acts(&g.lineage,&before.acts_before,&g.lineage.tree.acts);
            crate::offline::supply_reason(&mut r);
            r.chest=(g.lineage.tree.chest-before.chest_before).max(0);
            // (the purse is the town's: every slot's sees the whole session's change)
            crate::offline::set_net(&mut r,before.gold_before,g.lineage.gold);
            crate::offline::set_terms(&mut r,&g.lineage,before.tally_before.as_ref(),&before.acts_before);
            r
        };
        let away=self.active.offline_absence.as_ref().map_or(0,|a|a.elapsed_s);
        let mut r=finish(&mut self.active,full,with_stall);
        // Cut 113 §3: the shown bloodline's pick (one a return, sized by the absence)
        crate::returns::on_return(&mut self.active.lineage,away);r.pick=crate::returns::wire(&self.active.lineage);
        let summary=|id:u32,r:&ReturnReport,g:&Game|crate::wire::BloodlineReturn{legacy_earned:r.legacy_earned,id,name:format!("Bloodline {id}"),xp:vec![r.xp.clone()],packages:r.packages.clone(),bests:r.bests.clone(),boss_knowledge:Self::victory_knowledge(g,&r.bests),runs:r.runs,deepest:r.deepest,gold:r.gold.as_ref().map_or(0,|g|g.home+g.salvage+g.passage+g.wake-g.spent)};
        if !self.others.is_empty(){r.bloodlines.push(summary(self.selected,&r,&self.active));}
        for (id,g) in &mut self.others {
            town_from(&self.active.lineage,&mut g.lineage);
            let other=finish(g,false,false);
            r.bloodlines.push(summary(*id,&other,g));
            r.legacy_earned+=other.legacy_earned; r.runs+=other.runs; r.banked+=other.banked; r.returned+=other.returned;r.deepest=r.deepest.max(other.deepest);
            for death in other.deaths {if let Some(d)=r.deaths.iter_mut().find(|d|d.cause==death.cause){d.n+=death.n;}else{r.deaths.push(death);}}
            if let Some(b)=other.gold {let a=r.gold.get_or_insert_with(Default::default);a.home+=b.home;a.salvage+=b.salvage;a.spent+=b.spent;a.wake+=b.wake;a.wake_n+=b.wake_n;a.wake_cap=a.wake_cap.max(b.wake_cap);a.lost+=b.lost;a.unkept+=b.unkept;a.passage+=b.passage;}
            for (target,rows) in [(&mut r.salvaged,other.salvaged),(&mut r.spent,other.spent)] {
                for row in rows {if let Some(a)=target.iter_mut().find(|a|a.kind==row.kind){a.n+=row.n;a.gold+=row.gold;}else{target.push(row);}}
            }
        }
        r
    }
    pub fn run_offline(&mut self,s:u64)->ReturnReport {self.run_offline_mode(s,true,true)}
}

#[cfg(test)]
mod tests {
    use super::*;
    fn resident()->Session { let mut s=Session::new(1);s.active.build_town("house").unwrap();s.active.lineage.gold_move(1000,"test income");s }
    #[test]
    fn ascension_preserves_bloodlines_and_town_but_resets_shared_wallet() {
        for variant in crate::engine::VARIANTS {
            for selected in 1..=3 {
                let mut s=resident();s.add_bloodline().unwrap();s.add_bloodline().unwrap();
                s.active.lineage.town.bank=123;
                s.active.lineage.tree.ledger+=123;
                crate::tree::grant(&mut s.active.lineage,&["porter","scout"]);
                for id in 1..=3 {
                    s.select_bloodline(id).unwrap();
                    let l=&mut s.active.lineage;
                    crate::legacy::ensure(l);
                    let legacy=l.bloodline.as_mut().unwrap();
                    legacy.points=40+id;legacy.upgrades.insert("health".into(),1);
                    l.best_depth=10+id;l.heir_best=10+id;
                    l.clock_s=86400+u64::from(id);
                    l.facts.insert("foe:lich:boss".into());
                    l.pkg.runs.insert("steady".into(),12);
                    l.classes.insert("fighter".into(),crate::wire::ClassProg{level:7,xp:10,next:0});
                    l.forge.insert("sword".into(),crate::wire::ForgeRow::at(20));
                    l.kit.insert("sword".into(),2);
                    l.vault.push(crate::item::Item::new(100_001,"plate"));
                    l.unlocks.insert("cadence".into());
                    if id!=selected {s.send();}
                }
                s.select_bloodline(selected).unwrap();
                s.active.lineage.ended=true;
                let before=serde_json::to_value(&s.active.lineage).unwrap();
                let others=s.others.clone();
                s.ascend(variant).unwrap();
                let after=serde_json::to_value(&s.active.lineage).unwrap();
                for field in ["bloodline_id","bloodline","hero_legacy","classes","facts","pkg","forge","town","sets","clock_s"] {
                    assert!(before.get(field).is_some(),"fixture has {field}");
                    assert_eq!(after[field],before[field],"{variant}, slot {selected}, carry {field}");
                }
                let mut expected_tree=before["tree"].clone();
                expected_tree["ledger"]=serde_json::json!(before["tree"]["ledger"].as_i64().unwrap()-before["gold"].as_i64().unwrap());
                assert_eq!(after["tree"],expected_tree,"only cleared gold changes worker bookkeeping");
                assert_eq!(s.selected,selected);
                assert_eq!(s.others,others,"unselected games, including live runs, untouched");
                assert_eq!((s.active.lineage.gold,s.active.lineage.best_depth,s.active.lineage.heir),(0,0,1));
                assert!(s.active.lineage.kit.is_empty());
                assert_eq!(s.active.lineage.vault.is_empty(),variant=="bones_only");
                assert_eq!(s.active.lineage.unlocks.contains("cadence"),variant=="short_list");
                for (id,old) in others {
                    s.select_bloodline(id).unwrap();
                    assert_eq!(s.active.lineage.gold,0,"switch cannot restore old shared gold");
                    assert_eq!(s.active.lineage.town.bank,123,"Savings survives");
                    let mut expected=old;
                    town_from(&s.active.lineage,&mut expected.lineage);
                    assert_eq!(s.active,expected,"only shared town resources synchronize");
                }
            }
        }
    }
    #[test]
    fn ascension_refusal_preserves_complete_multibloodline_save() {
        let mut s=resident();s.add_bloodline().unwrap();s.add_bloodline().unwrap();
        for id in 1..=3 {
            s.select_bloodline(id).unwrap();s.send();
            let before=s.save();
            assert!(s.ascend("hunted").is_err());
            assert_eq!(s.save(),before,"premature refusal, slot {id}");
            s.active.lineage.ended=true;
            let before=s.save();
            assert!(s.ascend("unknown").is_err());
            assert_eq!(s.save(),before,"unknown variant, slot {id}");
            s.active.lineage.ended=false;
        }
    }
    #[test]
    fn offline_slice_multi_preserves_wallet_each_hero_and_complete_report() {
        for automated in [false,true] {
            let mut base=resident();base.add_bloodline().unwrap();base.add_bloodline().unwrap();
            if automated {crate::tree::grant(&mut base.active.lineage,&["porter","scout"]);}
            base.active.lineage.clock_s=crate::engine::DAY_S-60;
            for id in 1..=3 {base.select_bloodline(id).unwrap();base.active.lineage.clock_s=crate::engine::DAY_S-60;base.send();}
            base.select_bloodline(1).unwrap();
            let mut whole=base.clone();let expected=whole.run_offline_mode(7200,false,true);
            assert_eq!(expected.legacy_earned,expected.bloodlines.iter().map(|s|s.legacy_earned).sum::<u32>());
            for row in &expected.bloodlines {
                let prior=if row.id==base.selected {&base.active}else{&base.others[&row.id]};
                let now=if row.id==whole.selected {&whole.active}else{&whole.others[&row.id]};
                assert_eq!(row.legacy_earned,crate::legacy::current(&now.lineage).unwrap().points-crate::legacy::current(&prior.lineage).unwrap().points);
                assert!(row.legacy_earned>0);
            }
            for reload in [false,true] {
                let mut sliced=base.clone();let widths=[1,719,1280];let mut left=7200;let mut i=0;
                let report=loop {
                    let seconds=left.min(widths[i%widths.len()]);let last=left==seconds;
                    let r=sliced.run_offline_slice(seconds,last);
                    if last {break r;}
                    assert!(r.slice_pending && r.elapsed_s==0 && r.runs==0);
                    left-=seconds;i+=1;
                    if reload {sliced=Session::load(&sliced.save()).unwrap();}
                };
                assert!(serde_json::to_value(&sliced).unwrap()==serde_json::to_value(&whole).unwrap(),"all saved fields: automated{automated}, reload{reload}");
                assert_eq!(report,expected,"whole absence report, with each slot's XP and shared gold");
                assert!(sliced.active.run.is_none() && sliced.others.values().all(|g|g.run.is_none()),"real return ends at camp, with at most one completion per slot");
                assert_eq!(sliced.active.lineage.gold as i64+sliced.active.lineage.town.bank as i64,sliced.active.lineage.tree.ledger);
            }
        }
    }
    #[test]
    fn worker_maps_detach_before_writes_and_synchronize_in_slot_order() {
        let mut s=resident();s.add_bloodline().unwrap();s.add_bloodline().unwrap();
        s.active.lineage.tree.done.insert("send".into(),3);
        s.active.lineage.tree.acts.insert("scout".into(),7);
        s.active.lineage.tree.ranks.insert("scout".into(),2);
        for g in s.others.values_mut(){town_from(&s.active.lineage,&mut g.lineage);}
        let snapshot=s.clone();let before=snapshot.save();
        let second=&mut s.others.get_mut(&2).unwrap().lineage;
        second.town.bank=100;second.tree.ledger+=100;
        second.tree.hired.push(("porter".into(),0));second.tree.paused.insert("porter".into());
        *second.tree.done.get_mut("send").unwrap()+=1;
        *second.tree.acts.get_mut("scout").unwrap()+=1;
        *second.tree.ranks.get_mut("scout").unwrap()+=1;
        assert_eq!(s.active.lineage.town.bank,0);
        assert_eq!(s.others[&3].lineage.town.bank,0);
        assert!(!s.active.lineage.tree.paused.contains("porter"));
        assert_eq!(s.active.lineage.tree.done["send"],3);
        assert_eq!(s.others[&3].lineage.tree.acts["scout"],7);
        assert_eq!(snapshot.save(),before,"writes cannot mutate an earlier simulation snapshot");
        town_from(&s.others[&2].lineage,&mut s.active.lineage);
        town_from(&s.active.lineage,&mut s.others.get_mut(&3).unwrap().lineage);
        assert_eq!(s.active.lineage.town.bank,100);
        assert!(s.others[&3].lineage.tree.paused.contains("porter"));
        assert!(s.others[&3].lineage.tree.hired.iter().any(|(id,_)|id=="porter"));
        assert_eq!(s.active.lineage.tree.done["send"],4);
        assert_eq!(s.others[&3].lineage.tree.acts["scout"],8);
        assert_eq!(s.others[&3].lineage.tree.ranks["scout"],3);
        let saved=s.save();assert_eq!(Session::load(&saved).unwrap().save(),saved);
        assert_eq!(snapshot.save(),before);
    }
    #[test]
    fn new_residents_use_unworn_looks_without_affecting_runs_or_saved_choices() {
        for chosen in [None,Some("cat")] {
            let mut s=resident();if let Some(look)=chosen{s.set_look(look).unwrap();}
            s.add_bloodline().unwrap();s.add_bloodline().unwrap();
            let looks:Vec<_>=s.lineage().hero_slots.iter().map(|h|h.look.clone()).collect();
            let expected=if chosen.is_some(){vec!["cat","male","female"]}else{vec!["male","female","cat"]};
            assert_eq!(looks,expected);
            let before=s.save();assert!(s.add_bloodline().is_err());assert_eq!(s.save(),before);
            let mut loaded=Session::load(&before).unwrap();
            loaded.select_bloodline(3).unwrap();loaded.select_bloodline(1).unwrap();
            assert_eq!(loaded.lineage().hero_slots,s.lineage().hero_slots);
            for g in loaded.others.values_mut(){let look=g.lineage.look.clone();g.lineage.new_heir();assert_eq!(g.lineage.look,look);}
            for id in 1..=3 {s.select_bloodline(id).unwrap();s.send();}
            let mut plain=s.clone();plain.active.lineage.look=None;for g in plain.others.values_mut(){g.lineage.look=None;}
            assert_eq!(s.run_offline_mode(3600,false,true),plain.run_offline_mode(3600,false,true));
            let normalize=|session:&Session|{let mut v=serde_json::to_value(session).unwrap();v["lineage"].as_object_mut().unwrap().remove("look");
                for g in v["others"].as_object_mut().unwrap().values_mut(){g["lineage"].as_object_mut().unwrap().remove("look");}v};
            assert_eq!(normalize(&s),normalize(&plain),"cosmetics cannot affect game state or RNG");
        }
        // An existing class default counts even without an explicit saved choice.
        let mut s=resident();s.active.lineage.class=crate::hero::Class::Rogue;
        s.add_bloodline().unwrap();assert_eq!(s.others[&2].lineage.look.as_deref(),Some("male"));
        let mut old=serde_json::to_value(&s).unwrap();old["others"]["2"]["lineage"].as_object_mut().unwrap().remove("look");
        let loaded=Session::load(&old.to_string()).unwrap();assert_eq!(loaded.others[&2].lineage.look,None);
        assert_eq!(loaded.lineage().hero_slots[1].look,"male","old saves keep their old default");
    }
    #[test]
    fn away_class_xp_belongs_to_each_reported_slot() {
        for selected_runs in [false,true] {
        // Both heroes must come home for class XP (deaths feed none). Cut 109's floors kill
        // seed 1's fighter to a goblin; seed 2 returns both, as seed 1 did before.
        let mut s=Session::new(2);s.active.build_town("house").unwrap();s.active.lineage.gold_move(1000,"test income");
        s.add_bloodline().unwrap();
        s.select_bloodline(2).unwrap();s.lineage.class=crate::hero::Class::Rogue;s.send();
        s.select_bloodline(1).unwrap();if selected_runs {s.send();}
        let r=s.run_offline_mode(3600,false,true);
        let selected=r.bloodlines.iter().find(|b|b.id==1).unwrap();
        let other=r.bloodlines.iter().find(|b|b.id==2).unwrap();
        assert_eq!(selected.xp,vec![r.xp.clone()]);
        assert_eq!(selected.xp[0].class,"fighter");assert_eq!(other.xp[0].class,"rogue");
        assert_eq!(selected.xp[0].gained>0,selected_runs);assert!(other.xp[0].gained>0);
        assert_eq!(other.xp[0].gained,s.others[&2].lineage.classes["rogue"].xp);
        let wire=serde_json::to_value(&r).unwrap();
        let restored:ReturnReport=serde_json::from_value(wire).unwrap();assert_eq!(restored.bloodlines,r.bloodlines);
        let mut old=serde_json::to_value(other).unwrap();old.as_object_mut().unwrap().remove("xp");
        assert!(serde_json::from_value::<crate::wire::BloodlineReturn>(old).unwrap().xp.is_empty());
        let later=s.run_offline_mode(3600,false,true);
        assert!(later.bloodlines.iter().flat_map(|b|&b.xp).all(|x|x.gained==0&&x.level_ups==0));
        }
    }
    #[test]
    fn away_training_retains_each_slot_even_when_selected_hero_waits() {
        for both in [false,true] {
            let mut s=resident();s.add_bloodline().unwrap();
            s.select_bloodline(2).unwrap();s.lineage.pkg.runs.insert("steady".into(),39);s.send();
            s.select_bloodline(1).unwrap();
            if both {s.lineage.pkg.runs.insert("steady".into(),9);s.send();}
            let r=s.run_offline_mode(3600,false,true);
            let other=r.bloodlines.iter().find(|b|b.id==2).unwrap();
            assert!(other.packages.contains(&"STEADY L3".into()), "unselected training lost");
            let selected=r.bloodlines.iter().find(|b|b.id==1).unwrap();
            assert_eq!(selected.packages.contains(&"STEADY L2".into()),both);
            assert_eq!(selected.packages,r.packages,"top-level beats remain selected-slot compatible");
            assert_eq!(r.bloodlines.iter().map(|b|b.runs).sum::<u32>(),r.runs);
            let gold=r.gold.as_ref().unwrap();
            assert_eq!(r.bloodlines.iter().map(|b|b.gold).sum::<i32>(),gold.home+gold.salvage+gold.wake-gold.spent);
            let wire=serde_json::to_value(&r).unwrap();
            let restored:ReturnReport=serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(restored.bloodlines,r.bloodlines);
            let mut old=serde_json::to_value(other).unwrap();old.as_object_mut().unwrap().remove("packages");
            assert!(serde_json::from_value::<crate::wire::BloodlineReturn>(old).unwrap().packages.is_empty());
            let saved=s.save();let mut restored=Session::load(&saved).unwrap();
            assert_eq!(restored.active.lineage.pkg.runs,s.active.lineage.pkg.runs);
            assert_eq!(restored.others[&2].lineage.pkg.runs,s.others[&2].lineage.pkg.runs);
            let later=restored.run_offline_mode(3600,false,true);
            assert!(later.bloodlines.iter().all(|b|b.packages.is_empty()),"later no-gain report must not repeat training");
        }
    }
    #[test]
    fn away_bests_belong_to_each_slot_even_when_selected_hero_waits() {
        for both in [false,true] {
            let mut s=resident();s.add_bloodline().unwrap();
            s.select_bloodline(2).unwrap();s.send();
            s.select_bloodline(1).unwrap();if both {s.send();}
            let r=s.run_offline_mode(3600,false,true);
            let other=r.bloodlines.iter().find(|b|b.id==2).unwrap();
            assert!(!other.bests.is_empty(),"unselected completed-run records lost");
            assert!(other.bests.contains(&format!("D{}",s.others[&2].lineage.best_depth)));
            let selected=r.bloodlines.iter().find(|b|b.id==1).unwrap();
            assert_eq!(!selected.bests.is_empty(),both);
            assert_eq!(selected.bests,r.bests,"top-level records keep selected-slot compatibility");
            let wire=serde_json::to_value(&r).unwrap();
            let restored:ReturnReport=serde_json::from_value(wire).unwrap();
            assert_eq!(restored.bloodlines,r.bloodlines);
            let mut old=serde_json::to_value(other).unwrap();old.as_object_mut().unwrap().remove("bests");
            assert!(serde_json::from_value::<crate::wire::BloodlineReturn>(old).unwrap().bests.is_empty());
            let mut restored=Session::load(&s.save()).unwrap();
            let later=restored.run_offline_mode(3600,false,true);
            assert!(later.bloodlines.iter().all(|b|b.bests.is_empty()),"no later run must not repeat old records");
        }
    }
    #[test]
    fn slots_isolate_policy_xp_and_legacy_and_switch_without_restarting_runs() {
        let mut s=resident();s.active.lineage.bloodline.as_mut().unwrap().points=9;
        s.active.upgrade_hero("health").unwrap();s.active.lineage.classes.get_mut("fighter").unwrap().xp=17;
        let rules=s.active.lineage.rules().clone();s.add_bloodline().unwrap();
        assert_eq!(s.active.lineage.gold,750);s.active.send();let first=s.active.run.clone();
        s.select_bloodline(2).unwrap();assert_eq!(s.active.lineage.bloodline.as_ref().unwrap().points,0);
        assert_eq!(s.active.lineage.classes["fighter"].xp,0);
        s.active.send();let second=s.active.run.clone();s.select_bloodline(1).unwrap();
        assert_eq!(s.active.run,first);assert_eq!(s.active.lineage.rules(),&rules);
        assert_eq!(s.active.lineage.classes["fighter"].xp,17);assert_eq!(s.active.lineage.bloodline.as_ref().unwrap().upgrades["health"],1);
        s.select_bloodline(2).unwrap();assert_eq!(s.active.run,second);
        s.step(20);assert_eq!(s.active.lineage.clock_s,s.others[&1].lineage.clock_s,"watch and background use the same town clock");
        let save=s.save();let restored=Session::load(&save).unwrap();assert_eq!(restored.save(),save);
    }
    #[test]
    fn simultaneous_runs_advance_while_selected_hero_waits_and_wallet_is_conserved() {
        let mut s=resident();s.add_bloodline().unwrap();s.select_bloodline(2).unwrap();s.active.send();
        s.select_bloodline(1).unwrap();let before=s.others[&2].run.as_ref().unwrap().turn;s.advance(1000);
        assert!(s.others[&2].run.as_ref().unwrap().turn>before);assert!(s.active.run.is_none());
        s.active.send();let r=s.run_offline_mode(3600,false,true);assert_eq!(r.runs,2);
        let gold=r.gold.as_ref().unwrap();assert_eq!(r.bloodlines.iter().map(|b|b.gold).sum::<i32>(),gold.home+gold.salvage+gold.wake-gold.spent);
        assert_eq!(r.salvaged.iter().map(|row|row.gold).sum::<i32>(),gold.salvage,"all bloodlines' salvage rows explain the total");
        for g in std::iter::once(&s.active).chain(s.others.values()) {assert!(g.run.is_none());assert!(g.lineage.bloodline.as_ref().unwrap().points>0);}
        assert_eq!(s.active.lineage.gold as i64+s.active.lineage.town.bank as i64,s.active.lineage.tree.ledger);
        assert_eq!(s.lineage().hero_slots.len(),2);
        let a=Session::load(&s.save()).unwrap();let mut b=a.clone();let mut c=a;
        b.run_offline_mode(86400,false,true);c.run_offline_mode(86400,false,true);assert_eq!(b.save(),c.save());
    }
    #[test]
    fn failed_creation_selection_and_cap_leave_complete_session_unchanged() {
        let mut s=Session::new(1);let before=s.save();assert!(s.add_bloodline().is_err());assert_eq!(s.save(),before);
        s.active.build_town("house").unwrap();let before=s.save();assert!(s.add_bloodline().is_err());assert_eq!(s.save(),before);
        s.active.lineage.gold_move(1000,"test income");s.add_bloodline().unwrap();s.add_bloodline().unwrap();
        let before=s.save();assert!(s.add_bloodline().is_err());assert!(s.select_bloodline(99).is_err());assert_eq!(s.save(),before);
    }
    #[test]
    fn clock_is_chunk_independent_and_observing_another_slot_changes_no_gameplay() {
        let mut s=resident();s.add_bloodline().unwrap();s.active.send();s.select_bloodline(2).unwrap();s.active.send();
        let mut a=s.clone();let mut b=s.clone();let mut c=s;
        a.advance(180_000);
        for _ in 0..1800 {b.advance(100);}
        assert_eq!(a,b,"chunking the same elapsed time must preserve every game state");
        c.select_bloodline(1).unwrap();c.advance(180_000);c.select_bloodline(2).unwrap();
        // Unselected town mirrors are synchronised lazily; compare the authoritative
        // wallet and each hero after selecting it, rather than stale mirrors.
        for id in [1,2] {a.select_bloodline(id).unwrap();c.select_bloodline(id).unwrap();assert_eq!(a.active,c.active,"selection changed bloodline {id}");}
    }
    #[test]
    fn old_per_heir_legacy_migrates_once_and_a_successor_keeps_bloodline_upgrades() {
        let mut g=Game::new_resident(2);g.lineage.hero_legacy[0].points=11;g.lineage.bloodline=None;
        g.lineage.hero_legacy.push(crate::wire::HeroLegacy {heir:2,points:7,spent:3,upgrades:BTreeMap::from([("health".into(),1)]),..Default::default()});
        let s=Session::load(&g.save()).unwrap();let b=s.active.lineage.bloodline.as_ref().unwrap();assert_eq!((b.points,b.spent),(18,3));assert_eq!(b.upgrades["health"],1);
        assert_eq!(Session::load(&s.save()).unwrap().save(),s.save());
        let mut s=resident();s.add_bloodline().unwrap();s.active.lineage.town.bank=1000;s.active.lineage.tree.ledger+=1000;
        s.active.lineage.town.shared_night_runs=Some(crate::engine::NIGHT_RUNS-1);
        s.active.send();s.select_bloodline(2).unwrap();s.active.send();
        s.run_offline_mode(3600,false,true);assert_eq!(s.active.lineage.town.interest,20,"shared Savings pays once, not once per bloodline");
    }
}

#[cfg(test)]
mod victory_knowledge_tests {
    use super::*;
    #[test]
    fn victory_details_are_owned_read_only_and_default_on_old_wire() {
        let mut a=Game::new(1);let mut b=Game::new(2);
        a.lineage.facts.insert("foe:bloat_mother:gas".into());
        b.lineage.facts.insert("foe:bloat_mother:telegraph".into());
        let before=(a.save(),b.save());
        let bests=vec!["D13".into(),"boss: bloat_mother".into()];
        let first=Session::victory_knowledge(&a,&bests);
        let second=Session::victory_knowledge(&b,&bests);
        assert_eq!(first.len(),1);assert_eq!(first[0].facts,vec!["foe:bloat_mother:gas"]);
        assert_eq!(second[0].facts,vec!["foe:bloat_mother:telegraph"]);
        assert_eq!((a.save(),b.save()),before);
        assert!(Session::victory_knowledge(&a,&[]).is_empty());
        let old:crate::wire::BloodlineReturn=serde_json::from_str(r#"{"id":1,"name":"One","runs":1,"deepest":13,"gold":0}"#).unwrap();
        assert!(old.boss_knowledge.is_empty());
    }
}
