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
    /// Cut 6 §2: bought, crafted or vaulted by name — usable by its kind (`drink heal`) even
    /// while the flavour is unidentified. Found items are known only through their flavour.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub known: bool,
    /// Cut 8B §3: the kennel's leash — on the shelf for nothing while the lineage has never
    /// tamed; clearing the shelf refunds nothing for it and `auto_supply` never rebuys it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub free: bool,
    /// Cut 21 §2: a supply the hero found and brought home to the shelf (`found heal → shelf`)
    /// — packed like a bought one, but never the repeat's (the repeat buys the kinds the
    /// player packed) and a drop salvages it rather than refunding a price never paid.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub found: bool,
    /// Cut 22 §1: what the shelf charged for a bought supply — the price moves with the
    /// lineage's best depth, and a refund pays back what was paid (0: not bought, or bought
    /// before the price moved; the refund then pays today's price).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub paid: i32,
    /// QA on 524827b (qaAA: KEPT `axe +7 → vault` after the cage's `took axe +1`): how many of
    /// its `enchant` came from enchant scrolls the heirs read on it (`InvItem.enchanted`).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub enchanted: i32,
}

fn is_zero(x: &i32) -> bool {
    *x == 0
}

impl Item {
    pub fn new(id: u32, kind: &str) -> Item {
        Item { id, kind: kind.into(), hint: None, amount: 0, enchant: 0, known: false, free: false, found: false, paid: 0, enchanted: 0 }
    }
    /// Cut 6 §2: known by name (bought, crafted, vaulted) or by an identified flavour.
    pub fn is_known(&self, facts: &BTreeSet<String>, flavours: &Flavours) -> bool {
        self.known || is_identified(facts, flavours, &self.kind)
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
        // Cut 25 §1: the forged arm's steps are aim (`Hero::hit_pct`), not a harder blow.
        // Cut 30: the forge's steps past its aim are a harder blow.
        if crate::kit::is_kit_id(self.id) {
            let more = crate::kit::DMG_PER_STEP * (self.enchant - crate::kit::AIM_STEPS).max(0);
            return (a.0 + more, a.1 + more);
        }
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
    // (`ident_fact`'s text, written into a reused buffer: this is read on the tick's hot paths)
    thread_local! {
        static BUF: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    }
    match flavours.flavour_of(kind) {
        Some(f) => BUF.with(|b| {
            let mut b = b.borrow_mut();
            b.clear();
            b.push_str("item:");
            b.push_str(f);
            b.push('=');
            b.push_str(kind);
            facts.contains(b.as_str())
        }),
        None => true,
    }
}

/// Run-clear (the owner, 2026-10-02: "include item rarity colours + icons"): an item's rarity, read off what
/// it already is — never a roll of its own, so it changes nothing in play. Gear: its kind's depth band
/// (`gear_band`: the floors it is found from) plus its `+N` (the forge's tier, a bounty's or a cage's step,
/// enchant scrolls read on it); `power` 0 common · 1–2 uncommon · 3–4 rare · 5–7 epic · ≥ 8 legendary (a
/// mace +5, an axe the scrolls stacked to +7: the few things that truly are). Consumables and trinkets: their
/// worth and depth — value ≥ 20 or found only from D18 is rare (strength, enchant, recall, the mirror shard),
/// value ≥ 14 or found from D6 uncommon, the rest common; never above rare. An unidentified flavour reads
/// common: its rarity would name its kind (facts are learned, never leaked by a rim colour).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn is_common(&self) -> bool {
        *self == Rarity::Common
    }
    pub fn name(&self) -> &'static str {
        match self {
            Rarity::Common => "common",
            Rarity::Uncommon => "uncommon",
            Rarity::Rare => "rare",
            Rarity::Epic => "epic",
            Rarity::Legendary => "legendary",
        }
    }
}

/// Run-clear: a weapon or armour kind's depth band — 0 from D1 (dagger, sword, leather), 1 from D4
/// (axe, bow, mail), 2 from D8–D10 (plate, spear), 3 from D12 (mace, scale) — `defs::item_min_depth`.
pub fn gear_band(kind: &str) -> i32 {
    match crate::defs::item_min_depth(kind) {
        0 => 0,
        1..=4 => 1,
        5..=10 => 2,
        _ => 3,
    }
}

/// Run-clear: the rarity of `item` as the player knows it (`known`: its kind identified).
pub fn rarity(item: &Item, known: bool) -> Rarity {
    let d = item.def();
    match d.cat {
        Cat::Gold => Rarity::Common,
        Cat::Weapon | Cat::Armour => match gear_band(&item.kind) + item.enchant.max(0) {
            ..=0 => Rarity::Common,
            1..=2 => Rarity::Uncommon,
            3..=4 => Rarity::Rare,
            5..=7 => Rarity::Epic,
            _ => Rarity::Legendary,
        },
        Cat::Potion | Cat::Scroll | Cat::Misc => {
            if !known || matches!(item.kind.as_str(), "bones" | "trap") {
                return Rarity::Common;
            }
            let depth = crate::defs::item_min_depth(&item.kind);
            if d.value >= 20 || depth >= 18 {
                Rarity::Rare
            } else if d.value >= 14 || depth >= 6 {
                Rarity::Uncommon
            } else {
                Rarity::Common
            }
        }
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
    /// Cut 12 §6: a supply the camp gave (the kennel's leash; `leash · kennel`), never bought —
    /// always on the wire so a bought leash on the same shelf is not mistaken for it.
    #[serde(default)]
    pub free: bool,
    /// Cut 21 §2: found on a run and put on the shelf for free (`found`); a drop salvages it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub found: bool,
    /// QA on 524827b (qaAA): the `+N` of `enchant` that enchant scrolls read on it added
    /// (`axe +7 → vault · enchanted ×6`); absent when none.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub enchanted: i32,
    /// Run-clear: the item's rarity (`rarity`); absent = common.
    #[serde(default, skip_serializing_if = "Rarity::is_common")]
    pub rarity: Rarity,
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
        Cat::Gold => (true, "gold".into(), format!("gold ${}", item.amount)),
        Cat::Misc => (true, item.kind.clone(), if item.amount > 1 { format!("{} ({})", item.kind, item.amount) } else { item.kind.clone() }),
        Cat::Weapon | Cat::Armour => {
            let label = if item.enchant > 0 { format!("{} +{}", item.kind, item.enchant) } else { item.kind.clone() };
            (true, item.kind.clone(), label)
        }
        Cat::Potion | Cat::Scroll => {
            let cat = if d.cat == Cat::Potion { "potion" } else { "scroll" };
            if item.is_known(facts, flavours) {
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
    let rarity = rarity(item, known);
    InvItem { id: item.id, kind, known, label, hint: if known { None } else { item.hint }, free: item.free, found: item.found, enchanted: item.enchanted, rarity }
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
