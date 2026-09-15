//! Vocabulary: which tokens the editor may offer, gated by facts (conds), class and unlocks (verbs).
use crate::defs::{all_tags, item_def, Cat, ITEMS};
use crate::engine::LineageState;
use crate::facts::has_tag_fact;
use crate::hero::Class;
use crate::item::is_identified;
use crate::rules::{Cond, Verb, Vocabulary};
use crate::wire::Companion;

pub const TACTIC_CARDS: [&str; 3] = ["corridor_fighting", "kite_archers", "stair_dance"];

pub fn known_tags(l: &LineageState) -> Vec<&'static str> {
    all_tags().into_iter().filter(|t| has_tag_fact(&l.facts, t)).collect()
}

pub fn identified_kinds(l: &LineageState, cat: Cat) -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == cat && is_identified(&l.facts, &l.flavours, i.kind)).map(|i| i.kind).collect()
}

pub fn vocabulary(l: &LineageState) -> Vocabulary {
    let mut conds = vec![
        Cond::n("hp<", 30),
        Cond::n("hp>", 70),
        Cond::n("foes>=", 2),
        Cond::n("adj>=", 2),
        Cond::n("foe_hp<", 25),
        Cond::flag("unknown_item"),
        Cond::n("floor_seen>=", 60),
        Cond::n("depth>=", 2),
        Cond::n("alert>=", 3),
        Cond::flag("in_corridor"),
        Cond::flag("path_stairs"),
        Cond::flag("ally"),
        Cond::n("loot>=", 50),
        Cond::n("turns>", 100),
        Cond::flag("on_hurt"),
        Cond::flag("on_kill"),
        Cond::flag("on_see"),
    ];
    for t in known_tags(l) {
        conds.push(Cond::t("foe_tag", t));
    }
    for i in ITEMS {
        match i.cat {
            Cat::Weapon | Cat::Armour => conds.push(Cond::t("item", i.kind)),
            Cat::Potion | Cat::Scroll if is_identified(&l.facts, &l.flavours, i.kind) => conds.push(Cond::t("item", i.kind)),
            _ => {}
        }
    }
    if l.facts.contains("item:leash") {
        conds.push(Cond::t("item", "leash"));
    }
    if l.all_companions().next().is_some() {
        conds.push(Cond::n("party_hp<", 50));
    }
    for c in &l.party {
        if !conds.iter().any(|x| x.k == "party" && x.t.as_deref() == Some(&c.kind)) {
            conds.push(Cond::t("party", &c.kind));
        }
    }
    let mut verbs = vec![Verb::arg("attack", "nearest"), Verb::arg("attack", "lowest")];
    for t in known_tags(l) {
        verbs.push(Verb::arg("attack", &format!("tag:{t}")));
    }
    verbs.push(Verb::new("retreat"));
    verbs.push(Verb::new("back_corridor"));
    verbs.push(Verb::arg("drink", "unknown"));
    for k in identified_kinds(l, Cat::Potion) {
        verbs.push(Verb::arg("drink", k));
    }
    verbs.push(Verb::arg("read", "unknown"));
    for k in identified_kinds(l, Cat::Scroll) {
        if k != "aggravate" {
            verbs.push(Verb::arg("read", k));
        }
    }
    if l.class == Class::Rogue || l.unlocks.contains("throw") {
        for k in identified_kinds(l, Cat::Potion) {
            if !item_def(k).benevolent {
                verbs.push(Verb::arg("throw", k));
            }
        }
    }
    for v in ["descend", "bank", "return", "rest", "pick_up", "free_captive"] {
        verbs.push(Verb::new(v));
    }
    match l.class {
        Class::Fighter => verbs.push(Verb::new("shield_bash")),
        Class::Rogue => verbs.push(Verb::new("vanish")),
    }
    for card in TACTIC_CARDS {
        if l.unlocks.contains(card) {
            verbs.push(Verb::arg("tactic", card));
        }
    }
    if l.unlocks.contains("tame") {
        verbs.push(Verb::arg("tame", "nearest"));
        for t in known_tags(l) {
            verbs.push(Verb::arg("tame", &format!("tag:{t}")));
        }
    }
    if !l.party.is_empty() {
        verbs.push(Verb::new("recall"));
        verbs.push(Verb::new("send"));
    }
    Vocabulary { conds, verbs, max_rows: l.max_rows() }
}

/// A companion's editor vocabulary: its tags are its verbs.
pub fn companion_vocabulary(l: &LineageState, c: &Companion) -> Vocabulary {
    let mut conds = vec![
        Cond::n("self_hp<", 30),
        Cond::n("self_hp>", 70),
        Cond::n("hp<", 30),
        Cond::n("foes>=", 2),
        Cond::n("adj>=", 1),
        Cond::n("foe_hp<", 25),
        Cond::flag("in_corridor"),
        Cond::flag("on_hurt"),
        Cond::flag("on_see"),
        Cond::n("depth>=", 2),
        Cond::n("alert>=", 3),
        Cond::n("turns>", 100),
    ];
    for t in known_tags(l) {
        conds.push(Cond::t("foe_tag", t));
    }
    let mut verbs = vec![Verb::new("attack")];
    for t in &c.tags {
        if let Some(v) = crate::defs::tag_verb(t) {
            verbs.push(Verb::new(v));
        }
    }
    verbs.push(Verb::new("follow"));
    verbs.push(Verb::new("recall"));
    Vocabulary { conds, verbs, max_rows: c.max_rows }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn facts_gate_conds_and_unlocks_gate_verbs() {
        let mut l = LineageState::new(1);
        let v = vocabulary(&l);
        assert!(!v.conds.iter().any(|c| c.k == "foe_tag"));
        assert!(!v.verbs.iter().any(|x| x.v == "throw"));
        assert!(v.verbs.iter().any(|x| x.v == "shield_bash"));
        assert_eq!(v.max_rows, 4);
        l.facts.insert("foe:jackal:pack".into());
        let f = crate::item::ident_fact(&l.flavours, "poison").unwrap();
        l.facts.insert(f);
        l.unlocks.insert("throw".into());
        l.unlocks.insert("row5".into());
        let v = vocabulary(&l);
        assert!(v.conds.contains(&Cond::t("foe_tag", "pack")));
        assert!(!v.conds.contains(&Cond::t("foe_tag", "fast")));
        assert!(v.conds.contains(&Cond::t("item", "poison")));
        assert!(v.verbs.contains(&Verb::arg("throw", "poison")));
        assert!(v.verbs.contains(&Verb::arg("drink", "poison")));
        assert!(v.verbs.contains(&Verb::arg("attack", "tag:pack")));
        assert_eq!(v.max_rows, 5);
    }
}
