//! Vocabulary: which tokens the editor may offer, gated by facts (conds), class and unlocks (verbs).
use crate::defs::{all_tags, item_def, Cat, ITEMS};
use crate::engine::LineageState;
use crate::facts::has_tag_fact;
use crate::item::is_identified;
use crate::rules::{Cond, Verb, Vocabulary};
use crate::wire::Companion;

pub use crate::meta::{MASTERY_CARDS, TACTIC_CARDS, TIER2_CARDS};

pub fn known_tags(l: &LineageState) -> Vec<&'static str> {
    all_tags().into_iter().filter(|t| has_tag_fact(&l.facts, t)).collect()
}

pub fn identified_kinds(l: &LineageState, cat: Cat) -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == cat && is_identified(&l.facts, &l.flavours, i.kind)).map(|i| i.kind).collect()
}

pub fn vocabulary(l: &LineageState) -> Vocabulary {
    // Cut 2 §3: `alert>= turns> loot>= on_kill on_see party_hp<` are unlocks; §5: `foe_hp<`
    // opens once a kind is studied.
    let owned = |k: &str| crate::meta::cond_unlock(k).is_none_or(|u| l.unlocks.contains(u));
    let mut conds = vec![
        Cond::n("hp<", 30),
        Cond::n("hp>", 70),
        Cond::n("foes>=", 2),
        Cond::n("adj>=", 2),
        Cond::flag("unknown_item"),
        Cond::n("floor_seen>=", 60),
        Cond::n("depth>=", 2),
        Cond::flag("in_corridor"),
        Cond::flag("path_stairs"),
        Cond::flag("ally"),
        Cond::flag("on_hurt"),
    ];
    if l.facts.iter().any(|f| f.starts_with("foe:") && f.ends_with(":studied")) {
        conds.push(Cond::n("foe_hp<", 25));
    }
    for (k, n) in [("alert>=", 3), ("loot>=", 20), ("turns>", 100)] {
        if owned(k) {
            conds.push(Cond::n(k, n));
        }
    }
    for k in ["on_kill", "on_see"] {
        if owned(k) {
            conds.push(Cond::flag(k));
        }
    }
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
    // Cut 3: misc items are tokens once found.
    for k in crate::defs::FACT_MISC {
        if l.facts.contains(&format!("item:{k}")) {
            conds.push(Cond::t("item", k));
        }
    }
    // Cut 5 §4: situations seen are tokens (`on_see: nest`), gated by their fact alone.
    // Cut 7 §3: the band situations are tokens the same way.
    // Cut 8B §3: the stray too (`on_see: stray` → `tame`), once one has been seen.
    for k in ["nest", "shrine", "vault", "den", "lock", "captive", "hunger", "stray"] {
        if l.facts.contains(k) {
            conds.push(Cond::t("on_see", k));
        }
    }
    if l.all_companions().next().is_some() && owned("party_hp<") {
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
    if crate::hero::class_has_verb(l.class, l.class_level(), "throw") || l.unlocks.contains("throw") {
        verbs.push(Verb::arg("throw", "unknown"));
        for k in identified_kinds(l, Cat::Potion) {
            // Cut 3: clarity is thrown to clear confusion around a foe.
            if !item_def(k).benevolent || k == "clarity" {
                verbs.push(Verb::arg("throw", k));
            }
        }
        for k in crate::defs::THROWABLE_MISC {
            if l.facts.contains(&format!("item:{k}")) {
                verbs.push(Verb::arg("throw", k));
            }
        }
    }
    for v in ["descend", "bank", "return", "rest", "pick_up", "free_captive"] {
        // Cut 3 `no_rest`: rest is not a verb.
        if v == "rest" && l.variant_is("no_rest") {
            continue;
        }
        verbs.push(Verb::new(v));
    }
    let level = l.class_level();
    for (verb, lvl) in crate::hero::class_ladder(l.class) {
        if level < *lvl || *verb == "throw" {
            continue;
        }
        match *verb {
            // Targeted ranged verbs take the attack selectors.
            "shoot" | "bolt" | "mark" | "slow" | "double_shot" => {
                verbs.push(Verb::arg(verb, "nearest"));
                verbs.push(Verb::arg(verb, "lowest"));
                for t in known_tags(l) {
                    verbs.push(Verb::arg(verb, &format!("tag:{t}")));
                }
            }
            _ => verbs.push(Verb::new(verb)),
        }
    }
    if l.facts.contains("shrine") {
        verbs.push(Verb::arg("pray", "row"));
        verbs.push(Verb::arg("pray", "trait"));
    }
    for card in TACTIC_CARDS.iter().copied().chain(MASTERY_CARDS).chain(TIER2_CARDS) {
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
    Vocabulary { conds, verbs, max_rows: l.max_rows(), combos: crate::rules::combo_table() }
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
    Vocabulary { conds, verbs, max_rows: c.max_rows, combos: Vec::new() }
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
        // Cut 2: gated condition tokens and the studied tier.
        for k in ["alert>=", "turns>", "loot>=", "on_kill", "on_see", "foe_hp<"] {
            assert!(!v.conds.iter().any(|c| c.k == k), "{k} is gated");
        }
        l.unlocks.insert("cond_alert".into());
        l.facts.insert("foe:rat:studied".into());
        let v = vocabulary(&l);
        assert!(v.conds.iter().any(|c| c.k == "alert>="));
        assert!(v.conds.iter().any(|c| c.k == "foe_hp<"));
        assert!(!v.conds.iter().any(|c| c.k == "turns>"));
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

#[cfg(test)]
mod class_tests {
    use super::*;
    use crate::hero::Class;
    #[test]
    fn ranger_and_caster_ladders_are_level_gated() {
        let mut l = LineageState::new(3);
        l.class = Class::Ranger;
        let v = vocabulary(&l);
        assert!(v.verbs.contains(&Verb::arg("shoot", "nearest")));
        assert!(v.verbs.contains(&Verb::new("kite")));
        assert!(!v.verbs.iter().any(|x| x.v == "volley"));
        l.classes.insert("ranger".into(), crate::wire::ClassProg { level: 9, xp: 0 });
        let v = vocabulary(&l);
        for x in ["volley", "trap", "mark", "double_shot"] {
            assert!(v.verbs.iter().any(|y| y.v == x), "{x}");
        }
        l.class = Class::Caster;
        let v = vocabulary(&l);
        assert!(v.verbs.contains(&Verb::arg("bolt", "nearest")));
        assert!(v.verbs.contains(&Verb::new("ward")));
        assert!(!v.verbs.iter().any(|x| x.v == "nova"));
        assert!(!v.verbs.iter().any(|x| x.v == "throw"), "casters need the throw unlock");
    }
}
