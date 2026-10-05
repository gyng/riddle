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
    if dest.gold_ledger.len()!=source.gold_ledger.len()||dest.gold_ledger.last()!=source.gold_ledger.last(){dest.gold_ledger=source.gold_ledger.clone();}
    if dest.town!=source.town {dest.town=source.town.clone();}
    if dest.tree.hired!=source.tree.hired{dest.tree.hired=source.tree.hired.clone();}
    if dest.tree.done!=source.tree.done{dest.tree.done=source.tree.done.clone();}
    if dest.tree.paused!=source.tree.paused{dest.tree.paused=source.tree.paused.clone();}
    dest.tree.chest=source.tree.chest; dest.tree.ledger=source.tree.ledger;
    if dest.tree.acts!=source.tree.acts{dest.tree.acts=source.tree.acts.clone();}
    if dest.tree.ranks!=source.tree.ranks{dest.tree.ranks=source.tree.ranks.clone();}
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
            let live=g.run.as_ref().filter(|r|r.over.is_none());
            let xp=g.lineage.classes.get(g.lineage.class.name());
            HeroSlot { hero_name:crate::legacy::hero_name(g.lineage.seed,g.lineage.heir).into(), id, name:format!("Bloodline {id}"), heir:g.lineage.heir, class:g.lineage.class.name().into(),
                level:xp.map_or(1,|x|x.level), xp:xp.map_or(0,|x|x.xp), next:xp.map(|x|if x.level>=crate::engine::MAX_LEVEL {0}else{crate::hero::xp_to_next(x.level)}),
                state:if live.is_some(){"live"}else if g.waits(){"waits"}else{"rests"}.into(),
                live:g.live_run(), rest_s:g.lineage.rest_left as f64 /10.0,
                legacy:g.lineage.bloodline.clone().unwrap_or_default(),
                chronicle:g.lineage.chronicle.clone(),
                notice:crate::legacy::offers(&g.lineage,g.run.is_some()).iter().any(|u|u.affordable),
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
    pub fn run_offline_mode(&mut self,seconds:u64,full:bool,last:bool)->ReturnReport {
        if self.others.is_empty(){return if full {self.active.run_offline(seconds)}else if last{crate::offline::run_offline_quick(&mut self.active,seconds)}else{crate::offline::run_offline_counts(&mut self.active,seconds)};}
        let begin=|g:&mut Game|{
            g.settle_renown(0);g.lineage.reveal_left=1;g.batch=Default::default();g.events.clear();g.offline=true;g.watched=false;
            if !g.lineage.in_absence {g.lineage.absences+=1;g.lineage.in_absence=true;}
            if g.lineage.rest_watched{g.lineage.rest_left=0;g.lineage.rest_watched=false;}
            if !g.lineage.pkg.literal{g.lineage.rest_left=g.lineage.rest_left.min(crate::offline::REST_CARRY_TICKS);}
            let rules=g.lineage.rules().clone();g.passage=crate::forecast::sim_passage(g,&rules);
            (g.lineage.facts.clone(),g.lineage.class.name().to_string(),g.lineage.rank)
        };
        let active_before=begin(&mut self.active);
        let others_before:BTreeMap<_,_>=self.others.iter_mut().map(|(id,g)|{town_from(&self.active.lineage,&mut g.lineage);(*id,begin(g))}).collect();
        self.advance(seconds.saturating_mul(1000));
        let finish=|g:&mut Game,before:&(crate::shared::Shared<std::collections::BTreeSet<String>>,String,u32),full:bool,last:bool|{
            g.offline=false;g.lineage.in_absence=true;
            crate::offline::report_with(g,seconds,&before.0,&before.1,before.2,false,full,last)
        };
        let mut r=finish(&mut self.active,&active_before,full,last);
        let summary=|id:u32,r:&ReturnReport|crate::wire::BloodlineReturn{id,name:format!("Bloodline {id}"),runs:r.runs,deepest:r.deepest,gold:r.gold.as_ref().map_or(0,|g|g.home+g.salvage+g.wake-g.spent)};
        if !self.others.is_empty(){r.bloodlines.push(summary(self.selected,&r));}
        for (id,g) in &mut self.others {
            town_from(&self.active.lineage,&mut g.lineage);
            let other=finish(g,&others_before[id],false,false);
            r.bloodlines.push(summary(*id,&other));
            r.runs+=other.runs; r.banked+=other.banked; r.returned+=other.returned;r.deepest=r.deepest.max(other.deepest);
            for death in other.deaths {if let Some(d)=r.deaths.iter_mut().find(|d|d.cause==death.cause){d.n+=death.n;}else{r.deaths.push(death);}}
            if let Some(b)=other.gold {let a=r.gold.get_or_insert_with(Default::default);a.home+=b.home;a.salvage+=b.salvage;a.spent+=b.spent;a.wake+=b.wake;a.wake_n+=b.wake_n;a.wake_cap=a.wake_cap.max(b.wake_cap);a.lost+=b.lost;a.unkept+=b.unkept;}
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
