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
pub fn zero(n:&i32)->bool {*n==0}
pub fn current(l:&LineageState)->Option<Style> {
    l.specializations.get(l.class.name()).copied().filter(|s|s.parent()==l.class&&l.class_level()>=10&&crate::legacy::deepest(l)>=23)
}
pub fn has(h:&Hero,style:Style)->bool {h.specialization==Some(style)&&h.class==style.parent()&&h.level>=10}
pub fn choose(g:&mut Game,id:&str)->Result<(),String> {
    if g.run.is_some() {return Err("hero away".into());}
    if !g.lineage.town.home.unwrap_or(true) {return Err("build a house".into());}
    if id=="none" {g.lineage.specializations.remove(g.lineage.class.name());}
    else {
        let style=Style::parse(id).ok_or("unknown specialization")?;
        if style.parent()!=g.lineage.class {return Err(format!("choose {}",style.parent().name()));}
        if g.lineage.class_level()<10 {return Err("reach class level10".into());}
        if crate::legacy::deepest(&g.lineage)<23 {return Err("reach D23".into());}
        g.lineage.specializations.insert(g.lineage.class.name().into(),style);
    }
    crate::packages::recompile(&mut g.lineage);Ok(())
}
pub fn row(l:&LineageState)->Option<Row> {
    current(l).map(|style|Row{conds:vec![Cond::n(if style==Style::Sentinel {"adj>="}else{"foes>="},1)],
        verb:if style==Style::Sentinel {Verb::new("riposte")}else{Verb::arg("hex","nearest")},
        origin:Some(format!("style:{}",style.id()))})
}

#[cfg(test)]
#[path="specialization_tests.rs"]
mod tests;
