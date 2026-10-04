//! Run-clear (the owner, 2026-10-02: "include item rarity colours + icons, each run should have the clear screen"):
//! item rarity read off what an item already is, on every wire item; the exit line's end kind, floor, record and
//! finds for the run-clear card.
use crate::item::{rarity, to_inv, Flavours, Item, Rarity};
use crate::rng::Rng;
use std::collections::BTreeSet;

fn gear(kind: &str, e: i32) -> Item {
    let mut it = Item::new(9, kind);
    it.enchant = e;
    it
}

#[test]
fn rarity_reads_the_item_band_and_its_plus() {
    // gear: the kind's depth band + its `+N`
    assert_eq!(rarity(&gear("dagger", 0), true), Rarity::Common);
    assert_eq!(rarity(&gear("sword", 0), true), Rarity::Common);
    assert_eq!(rarity(&gear("leather", 0), true), Rarity::Common);
    assert_eq!(rarity(&gear("sword", 1), true), Rarity::Uncommon, "a cage's sword +1");
    assert_eq!(rarity(&gear("axe", 0), true), Rarity::Uncommon, "a D4 kind");
    assert_eq!(rarity(&gear("mail", 1), true), Rarity::Uncommon);
    assert_eq!(rarity(&gear("plate", 1), true), Rarity::Rare);
    assert_eq!(rarity(&gear("sword", 3), true), Rarity::Rare);
    assert_eq!(rarity(&gear("mace", 2), true), Rarity::Epic);
    assert_eq!(rarity(&gear("sword", 6), true), Rarity::Epic, "the forge's top step");
    assert_eq!(rarity(&gear("axe", 7), true), Rarity::Legendary, "scrolls stacked an axe to +7");
    assert_eq!(rarity(&gear("mace", 5), true), Rarity::Legendary);
    // consumables and trinkets: worth and depth, never above rare
    assert_eq!(rarity(&Item::new(1, "heal"), true), Rarity::Common);
    assert_eq!(rarity(&Item::new(1, "fire"), true), Rarity::Uncommon);
    assert_eq!(rarity(&Item::new(1, "regen"), true), Rarity::Uncommon);
    assert_eq!(rarity(&Item::new(1, "strength"), true), Rarity::Rare);
    assert_eq!(rarity(&Item::new(1, "enchant"), true), Rarity::Rare);
    assert_eq!(rarity(&Item::new(1, "mirror_shard"), true), Rarity::Rare);
    assert_eq!(rarity(&Item::new(1, "leash"), true), Rarity::Common);
    assert_eq!(rarity(&Item::new(1, "gold"), true), Rarity::Common);
    for d in crate::defs::ITEMS.iter().filter(|d| !matches!(d.cat, crate::defs::Cat::Weapon | crate::defs::Cat::Armour)) {
        assert!(rarity(&Item::new(1, d.kind), true) <= Rarity::Rare, "{}", d.kind);
    }
    // an unidentified flavour never names its kind by its rim
    assert_eq!(rarity(&Item::new(1, "strength"), false), Rarity::Common);
    // the order is the tiers' order
    assert!(Rarity::Common < Rarity::Uncommon && Rarity::Uncommon < Rarity::Rare && Rarity::Rare < Rarity::Epic && Rarity::Epic < Rarity::Legendary);
}

#[test]
fn rarity_rides_the_wire_and_hides_an_unknown_flavour() {
    let f = Flavours::roll(&mut Rng::new(4));
    let mut facts = BTreeSet::new();
    let w = to_inv(&gear("plate", 1), &facts, &f);
    assert_eq!(w.rarity, Rarity::Rare);
    let j = serde_json::to_value(&w).unwrap();
    assert_eq!(j["rarity"], "rare");
    // common is the default: absent on the wire
    let c = serde_json::to_value(to_inv(&gear("dagger", 0), &facts, &f)).unwrap();
    assert!(c.get("rarity").is_none());
    let s = Item::new(3, "strength");
    assert_eq!(to_inv(&s, &facts, &f).rarity, Rarity::Common, "unknown");
    facts.insert(crate::item::ident_fact(&f, "strength").unwrap());
    assert_eq!(to_inv(&s, &facts, &f).rarity, Rarity::Rare, "identified");
    // an old wire without the field reads common
    let back: crate::item::InvItem = serde_json::from_value(c).unwrap();
    assert_eq!(back.rarity, Rarity::Common);
}

#[test]
fn the_forge_steps_carry_their_rarity() {
    let g = crate::Game::new_resident(7);
    let lad = crate::kit::ladders(&g.lineage);
    let weapon = lad.iter().find(|l| l.slot == "weapon").unwrap();
    assert!(weapon.steps.iter().all(|s| s.kind.is_some()));
    // rising with the step, never falling
    assert!(weapon.steps.windows(2).all(|w| w[0].rarity <= w[1].rarity));
    assert!(weapon.steps.last().unwrap().rarity >= Rarity::Epic);
    let pack = lad.iter().find(|l| l.slot == "pack").unwrap();
    assert!(pack.steps.iter().all(|s| s.kind.is_none() && s.rarity == Rarity::Common));
}

/// Every exit line says its kind, its floor and whether it set a record; its finds are the items the run brought home (or left
/// in its bones), rarest first, ≤ FINDS_SHOWN, each with its rarity — and none of it moves the run.
#[test]
fn every_exit_line_carries_the_card() {
    let mut seen_finds = 0;
    let mut records = 0;
    for seed in [3u64, 1001, 1004] {
        let mut g = crate::Game::new_resident(seed);
        let mut h = crate::Game::new_resident(seed);
        for _ in 0..5 {
            let best0 = g.lineage.best_depth;
            g.send();
            h.send();
            let mut line = None;
            for _ in 0..400 {
                let r = g.step(200);
                let q = h.step(200);
                // the same events, the card's fields and all (determinism)
                assert_eq!(serde_json::to_string(&r.events).unwrap(), serde_json::to_string(&q.events).unwrap());
                for e in &r.events {
                    if let crate::Ev::Exit { line: Some(l), tier, .. } = e {
                        line = Some((l.clone(), tier.clone()));
                    }
                }
                if r.run_over {
                    break;
                }
            }
            g.auto_keep();
            h.auto_keep();
            let (l, tier) = line.expect("an exit line");
            assert_eq!(l.end, tier, "the line's kind is the event's");
            assert!(l.reached >= 1);
            assert_eq!(l.new_best, l.reached > best0, "seed {seed}: D{} vs best D{best0}", l.reached);
            records += l.new_best as u32;
            assert!(l.finds.len() <= crate::wire::FINDS_SHOWN);
            assert!(l.finds.windows(2).all(|w| w[0].rarity >= w[1].rarity), "rarest first");
            assert!(l.finds.iter().all(|f| f.kind != "gold"));
            seen_finds += l.finds.len();
        }
    }
    assert!(records >= 3, "each fresh lineage's first run sets a record");
    assert!(seen_finds > 0, "some run brings a find home");
}
