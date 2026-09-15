//! Items: runtime instances, flavours (per lineage), labels and identification.
use crate::defs::{item_def, potion_kinds, scroll_kinds, Cat, ItemDef, POTION_FLAVOURS, SCROLL_FLAVOURS};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Hint {
    Benevolent,
    Malevolent,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Item {
    pub id: u32,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Hint>,
    #[serde(default)]
    pub amount: i32,
    #[serde(default)]
    pub enchant: i32,
}

impl Item {
    pub fn new(id: u32, kind: &str) -> Item {
        Item { id, kind: kind.into(), hint: None, amount: 0, enchant: 0 }
    }
    pub fn def(&self) -> &'static ItemDef {
        item_def(&self.kind)
    }
    pub fn cat(&self) -> Cat {
        self.def().cat
    }
    pub fn value(&self) -> i32 {
        match self.cat() {
            Cat::Gold => self.amount,
            Cat::Misc => self.def().value * self.amount.max(1),
            _ => self.def().value + self.enchant * 10,
        }
    }
    pub fn is_consumable(&self) -> bool {
        matches!(self.cat(), Cat::Potion | Cat::Scroll)
    }
    pub fn atk(&self) -> (i32, i32) {
        let a = self.def().a;
        (a.0 + self.enchant, a.1 + self.enchant)
    }
    pub fn def_bonus(&self) -> i32 {
        self.def().a.0 + self.enchant
    }
}

/// Potion and scroll flavours, randomised per lineage.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Flavours {
    pub potion: BTreeMap<String, String>,
    pub scroll: BTreeMap<String, String>,
}

impl Flavours {
    pub fn roll(rng: &mut Rng) -> Flavours {
        let mut pf: Vec<&str> = POTION_FLAVOURS.to_vec();
        let mut sf: Vec<&str> = SCROLL_FLAVOURS.to_vec();
        rng.shuffle(&mut pf);
        rng.shuffle(&mut sf);
        let mut f = Flavours::default();
        for (i, k) in potion_kinds().iter().enumerate() {
            f.potion.insert(k.to_string(), pf[i % pf.len()].to_string());
        }
        for (i, k) in scroll_kinds().iter().enumerate() {
            f.scroll.insert(k.to_string(), sf[i % sf.len()].to_string());
        }
        f
    }
    pub fn flavour_of(&self, kind: &str) -> Option<&str> {
        self.potion.get(kind).or_else(|| self.scroll.get(kind)).map(|s| s.as_str())
    }
}

pub fn ident_fact(flavours: &Flavours, kind: &str) -> Option<String> {
    flavours.flavour_of(kind).map(|f| format!("item:{f}={kind}"))
}

pub fn is_identified(facts: &BTreeSet<String>, flavours: &Flavours, kind: &str) -> bool {
    match ident_fact(flavours, kind) {
        Some(f) => facts.contains(&f),
        None => true,
    }
}

/// Wire: inventory item.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvItem {
    pub id: u32,
    pub kind: String,
    pub known: bool,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<Hint>,
}

/// Wire: item on the floor.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FloorItemWire {
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub kind: String,
    pub known: bool,
    pub label: String,
}

/// (known, wire kind, label). Unknown consumables hide their kind behind the category.
pub fn describe(item: &Item, facts: &BTreeSet<String>, flavours: &Flavours) -> (bool, String, String) {
    let d = item.def();
    match d.cat {
        Cat::Gold => (true, "gold".into(), format!("gold ({})", item.amount)),
        Cat::Misc => (true, item.kind.clone(), if item.amount > 1 { format!("{} ({})", item.kind, item.amount) } else { item.kind.clone() }),
        Cat::Weapon | Cat::Armour => {
            let label = if item.enchant > 0 { format!("{} +{}", item.kind, item.enchant) } else { item.kind.clone() };
            (true, item.kind.clone(), label)
        }
        Cat::Potion | Cat::Scroll => {
            let cat = if d.cat == Cat::Potion { "potion" } else { "scroll" };
            if is_identified(facts, flavours, &item.kind) {
                (true, item.kind.clone(), format!("{} {}", item.kind.replace('_', " "), cat))
            } else {
                let fl = flavours.flavour_of(&item.kind).unwrap_or("odd");
                (false, cat.to_string(), format!("{fl} {cat}?"))
            }
        }
    }
}

pub fn to_inv(item: &Item, facts: &BTreeSet<String>, flavours: &Flavours) -> InvItem {
    let (known, kind, label) = describe(item, facts, flavours);
    InvItem { id: item.id, kind, known, label, hint: if known { None } else { item.hint } }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flavours_bijective_per_lineage() {
        let f = Flavours::roll(&mut Rng::new(1));
        let vals: BTreeSet<&String> = f.potion.values().collect();
        assert_eq!(vals.len(), f.potion.len());
        let g = Flavours::roll(&mut Rng::new(1));
        assert_eq!(f, g);
    }
    #[test]
    fn unknown_items_hide_kind() {
        let f = Flavours::roll(&mut Rng::new(2));
        let mut facts = BTreeSet::new();
        let mut it = Item::new(1, "heal");
        it.hint = Some(Hint::Benevolent);
        let w = to_inv(&it, &facts, &f);
        assert!(!w.known);
        assert_eq!(w.kind, "potion");
        assert!(w.label.ends_with("potion?"));
        assert_eq!(w.hint, Some(Hint::Benevolent));
        facts.insert(ident_fact(&f, "heal").unwrap());
        let w2 = to_inv(&it, &facts, &f);
        assert!(w2.known);
        assert_eq!(w2.kind, "heal");
        assert_eq!(w2.label, "heal potion");
        assert_eq!(w2.hint, None);
    }
}
