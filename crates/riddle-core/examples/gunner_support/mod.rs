//! Paid home preparation shared by earned Gunner diagnostics.
use riddle_core::{bloodlines::Session,kit,legacy,tree,town};
pub fn camp(s:&mut Session,branches:bool)->Vec<String> {
    assert!(s.active.run.is_none(),"player choices require home");
    let mut actions=vec![];
    for (id,_) in town::BUILDINGS {
        if !town::built(&s.active.lineage,id)&&s.build_town(id).is_ok(){actions.push(format!("build {id}"));}
    }
    for _ in 0..tree::NODES.len() {
        let Some(n)=tree::lit(&s.active.lineage) else {break};
        if s.hire(n.id).is_err(){break;}actions.push(format!("hire {}",n.id));
    }
    let ids=if branches {vec!["health","damage","armour","restoration","clear_lungs","renewal","brace"]}
        else {legacy::IDS.to_vec()};
    for id in ids {
        while legacy::offers(&s.active.lineage,false).iter().any(|o|o.id==id&&o.affordable) {
            s.upgrade_hero(id).expect("pay earned offered Legacy");actions.push(format!("Legacy {id}"));
        }
    }
    // Preserve the same three-unit supplies reserve as the daily-player harness.
    loop {
        let next=kit::ladders(&s.active.lineage).into_iter().filter(|l|kit::KIT_SLOTS.contains(&l.slot.as_str())).filter_map(|l|l.next.map(|n|(l.slot,n.price))).min_by_key(|n|n.1);
        let Some((id,price))=next else {break};
        let reserve=3*kit::unit_of(&s.active.lineage);
        if i64::from(s.active.lineage.gold)<i64::from(price)+i64::from(reserve){break;}
        kit::buy(&mut s.active,&id).expect("pay offered forge price");actions.push(format!("forge {id}"));
    }
    actions
}
