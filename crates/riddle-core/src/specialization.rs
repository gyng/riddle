//! Explicit inherited class choices, with send-snapshotted abilities.
use crate::{engine::{Game,LineageState},hero::{Class,Hero},rules::{Row,Cond,Verb}};
use serde::{Serialize,Deserialize};
#[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(rename_all="snake_case")]
pub enum Style {Sentinel,Hexbinder}
impl Style {
    pub fn id(self)->&'static str {match self {Self::Sentinel=>"sentinel",Self::Hexbinder=>"hexbinder"}}
    pub fn name(self)->&'static str {match self {Self::Sentinel=>"Sentinel",Self::Hexbinder=>"Hexbinder"}}
    pub fn parent(self)->Class {match self {Self::Sentinel=>Class::Fighter,Self::Hexbinder=>Class::Caster}}
    pub fn verb(self)->&'static str {match self {Self::Sentinel=>"riposte",Self::Hexbinder=>"hex"}}
    pub fn parse(id:&str)->Option<Self> {match id {"sentinel"=>Some(Self::Sentinel),"hexbinder"=>Some(Self::Hexbinder),_=>None}}
}
pub const COOLDOWN:i32=60;
pub const RIPOSTE_DURATION:i32=20;
pub const HEX_DURATION:i32=40;
pub const COUNTER_DAMAGE:i32=8;
pub const HEX_REDUCTION:i32=2;
pub const REQUIRED_LEVEL:u32=10;
pub const REQUIRED_DEPTH:u32=23;

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct Offer {
    pub id:Style, pub name:String, pub parent:String,
    pub level:u32, pub xp:u32, pub next:u32,
    pub required_level:u32, pub required_depth:u32, pub deepest:u32,
    pub selected:bool, pub available:bool, pub blocked:Option<String>,
    pub cooldown_ticks:i32, pub duration_ticks:i32, pub effect:String,
    pub tactic:Row,
}
#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct Choice {
    pub selected:Option<Style>, pub offers:Vec<Offer>,
    pub remove_available:bool, pub remove_blocked:Option<String>,
    pub automatic_row:bool, pub player_overrides:bool,
}
fn home_block(l:&LineageState,away:bool)->Option<String> {
    if away {Some("hero away".into())}else if !l.town.home.unwrap_or(true) {Some("build a house".into())}else {None}
}
fn blocked(l:&LineageState,away:bool,style:Style)->Option<String> {
    home_block(l,away).or_else(||if style.parent()!=l.class {Some(format!("choose {}",style.parent().name()))}
        else if l.class_level()<REQUIRED_LEVEL {Some(format!("reach class level{REQUIRED_LEVEL}"))}
        else if crate::legacy::deepest(l)<REQUIRED_DEPTH {Some(format!("reach D{REQUIRED_DEPTH}"))}else{None})
}
fn default_row(style:Style)->Row {
    Row{conds:vec![Cond::n(if style==Style::Sentinel {"adj>="}else{"foes>="},1)],
        verb:if style==Style::Sentinel {Verb::new("riposte")}else{Verb::arg("hex","nearest")},
        origin:Some(format!("style:{}",style.id()))}
}
pub fn offers(l:&LineageState,away:bool)->Choice {
    let selected=current(l);let home=home_block(l,away);
    Choice{selected,remove_available:l.specializations.contains_key(l.class.name())&&home.is_none(),remove_blocked:home,
        automatic_row:!l.pkg.literal,player_overrides:!l.pkg.pen.is_empty(),
        offers:[Style::Sentinel,Style::Hexbinder].into_iter().map(|style| {
            let p=l.classes.get(style.parent().name());let level=p.map_or(1,|p|p.level);
            let blocked=blocked(l,away,style);
            Offer{id:style,name:style.name().into(),parent:style.parent().name().into(),level,xp:p.map_or(0,|p|p.xp),
                next:if level>=crate::engine::MAX_LEVEL {0}else{crate::hero::xp_to_next(level)},
                required_level:REQUIRED_LEVEL,required_depth:REQUIRED_DEPTH,deepest:crate::legacy::deepest(l),
                selected:l.specializations.get(style.parent().name())==Some(&style),available:blocked.is_none(),blocked,
                cooldown_ticks:COOLDOWN,duration_ticks:if style==Style::Sentinel {RIPOSTE_DURATION}else{HEX_DURATION},
                effect:if style==Style::Sentinel {format!("Halve one melee hit · counter {COUNTER_DAMAGE}")}else{format!("Enemy attacks deal {HEX_REDUCTION} less damage")},tactic:default_row(style)}
        }).collect()}
}
pub fn zero(n:&i32)->bool {*n==0}
pub fn current(l:&LineageState)->Option<Style> {
    l.specializations.get(l.class.name()).copied().filter(|s|s.parent()==l.class&&l.class_level()>=REQUIRED_LEVEL&&crate::legacy::deepest(l)>=REQUIRED_DEPTH)
}
pub fn has(h:&Hero,style:Style)->bool {h.specialization==Some(style)&&h.class==style.parent()&&h.level>=REQUIRED_LEVEL}
pub fn choose(g:&mut Game,id:&str)->Result<(),String> {
    if let Some(reason)=home_block(&g.lineage,g.run.is_some()) {return Err(reason);}
    if id=="none" {g.lineage.specializations.remove(g.lineage.class.name());}
    else {
        let style=Style::parse(id).ok_or("unknown specialization")?;
        if let Some(reason)=blocked(&g.lineage,false,style) {return Err(reason);}
        g.lineage.specializations.insert(g.lineage.class.name().into(),style);
    }
    crate::packages::recompile(&mut g.lineage);Ok(())
}
pub fn row(l:&LineageState)->Option<Row> {
    current(l).map(default_row)
}

#[cfg(test)]
#[path="specialization_tests.rs"]
mod tests;
