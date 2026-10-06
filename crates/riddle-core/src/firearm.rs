//! Firearm profiles and per-item commitment. Combat/scheduler integration is
//! separate: these primitives never select targets, roll RNG or deal damage.
use crate::geom::Pos;
use serde::{Deserialize, Serialize};

pub const LONG_GUN: crate::defs::ItemDef = crate::defs::ItemDef {
    kind: "long_gun", cat: crate::defs::Cat::Weapon, a: (6, 10), speed: 0,
    ranged: true, value: 35, benevolent: true, weight: 0,
};
pub const SHORT_GUN: crate::defs::ItemDef = crate::defs::ItemDef {
    kind: "short_gun", cat: crate::defs::Cat::Weapon, a: (4, 7), speed: 0,
    ranged: true, value: 35, benevolent: true, weight: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Profile {
    pub capacity: u8,
    pub range: i32,
    pub damage: (i32, i32),
    pub armour_piercing: i32,
    pub reload_ticks: u32,
}

impl Profile {
    pub fn of(kind: &str) -> Option<Self> {
        match kind {
            "long_gun" => Some(Self { capacity: 1, range: 8, damage: (6, 10), armour_piercing: 2, reload_ticks: 20 }),
            "short_gun" => Some(Self { capacity: 2, range: 3, damage: (4, 7), armour_piercing: 0, reload_ticks: 15 }),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Chambers {
    pub loaded: u8,
    /// Absolute run tick. Save/load and slice boundaries cannot shorten it;
    /// scheduler integration must stop batches at this deadline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reload_at: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal { Reloading, Empty, Full, InvalidTarget, Range, Sight, Burst, Clock }

/// Already observed target facts. No remembered-target mutation occurs until
/// this validation and chamber commitment both succeed in the action caller.
#[derive(Clone, Copy, Debug)]
pub struct ShotTarget {
    pub from: Pos,
    pub to: Pos,
    pub hostile_alive: bool,
    pub visible: bool,
    pub los: bool,
}

impl Chambers {
    pub fn loaded(profile: Profile) -> Self { Self { loaded: profile.capacity, reload_at: None } }

    pub fn reload(&mut self, profile: Profile, now: u32) -> Result<u32, Refusal> {
        if self.reload_at.is_some() { return Err(Refusal::Reloading); }
        if self.loaded >= profile.capacity { return Err(Refusal::Full); }
        let at = now.checked_add(profile.reload_ticks).ok_or(Refusal::Clock)?;
        self.reload_at = Some(at);
        Ok(at)
    }

    /// True exactly once, at/after the deadline, for the completion event.
    pub fn complete(&mut self, profile: Profile, now: u32) -> bool {
        if self.reload_at.is_some_and(|at| now >= at) {
            self.loaded = profile.capacity;
            self.reload_at = None;
            return true;
        }
        false
    }

    /// Reserve chambers before any hit/miss/reflection RNG. An invalid request
    /// leaves all state exact; a valid miss still spends its reserved chamber.
    pub fn fire(&mut self, profile: Profile, target: ShotTarget, burst: bool) -> Result<(), Refusal> {
        if !target.hostile_alive || target.from == target.to { return Err(Refusal::InvalidTarget); }
        if target.from.cheb(target.to) > profile.range { return Err(Refusal::Range); }
        if !target.visible || !target.los { return Err(Refusal::Sight); }
        if self.reload_at.is_some() { return Err(Refusal::Reloading); }
        if burst && profile.capacity != 2 { return Err(Refusal::Burst); }
        let cost = if burst { 2 } else { 1 };
        if self.loaded < cost { return Err(Refusal::Empty); }
        self.loaded -= cost;
        Ok(())
    }
}

/// Cheap geometric filter for short-gun secondary targets. The caller supplies
/// only live, hostile, visible, LOS candidates, and caps the total at three.
/// A 90-degree forward cone centred on the actual primary shot, within range,
/// with secondary targets adjacent to the primary (not the shooter).
pub fn in_spread(from: Pos, primary: Pos, candidate: Pos) -> bool {
    if from == primary || from == candidate || candidate == primary ||
        from.cheb(candidate) > 3 || !primary.adjacent(candidate) { return false; }
    let ax = i64::from(primary.x) - i64::from(from.x);
    let ay = i64::from(primary.y) - i64::from(from.y);
    let bx = i64::from(candidate.x) - i64::from(from.x);
    let by = i64::from(candidate.y) - i64::from(from.y);
    let dot = ax * bx + ay * by;
    let cross = ax * by - ay * bx;
    dot > 0 && cross.abs() <= dot
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Item;
    fn target(distance: i32) -> ShotTarget {
        ShotTarget { from: Pos::new(0, 0), to: Pos::new(distance, 0), hostile_alive: true, visible: true, los: true }
    }
    #[test]
    fn exact_ranges_and_failed_shots_preserve_chambers() {
        for (kind, range) in [("long_gun", 8), ("short_gun", 3)] {
            let p = Profile::of(kind).unwrap();
            let mut c = Chambers::loaded(p);
            let original = c;
            assert_eq!(c.fire(p, target(range + 1), false), Err(Refusal::Range));
            for invalid in [ShotTarget { los: false, ..target(range) }, ShotTarget { visible: false, ..target(range) }, ShotTarget { hostile_alive: false, ..target(range) }, target(0)] {
                assert!(c.fire(p, invalid, false).is_err());
                assert_eq!(c, original);
            }
            assert_eq!(c.fire(p, target(range), false), Ok(()));
            assert_eq!(c.loaded, p.capacity - 1);
        }
    }
    #[test]
    fn exact_reload_boundaries_refuse_fire_and_repeated_reload() {
        for kind in ["long_gun", "short_gun"] {
            let p = Profile::of(kind).unwrap();
            let mut c = Chambers::loaded(p);
            assert_eq!(c.reload(p, 40), Err(Refusal::Full));
            c.fire(p, target(1), false).unwrap();
            let deadline = c.reload(p, 40).unwrap();
            let pending = c;
            assert_eq!(deadline, 40 + p.reload_ticks);
            assert_eq!(c.reload(p, 41), Err(Refusal::Reloading));
            assert_eq!(c.fire(p, target(1), false), Err(Refusal::Reloading));
            assert!(!c.complete(p, deadline - 1));
            assert_eq!(c, pending);
            assert!(c.complete(p, deadline));
            assert_eq!(c.loaded, p.capacity);
            assert!(!c.complete(p, deadline + 1));
        }
    }
    #[test]
    fn weapon_instances_keep_independent_chambers_across_swap_and_save() {
        let mut long = Item::new(101, "long_gun");
        let mut short = Item::new(102, "short_gun");
        let lp = Profile::of(&long.kind).unwrap();
        let sp = Profile::of(&short.kind).unwrap();
        long.firearm.as_mut().unwrap().fire(lp, target(1), false).unwrap();
        short.firearm.as_mut().unwrap().fire(sp, target(1), false).unwrap();
        long.firearm.as_mut().unwrap().reload(lp, 100).unwrap();
        let mut worn = Some(long);
        let stowed = worn.replace(short).unwrap();
        let text = serde_json::to_string(&(worn, stowed)).unwrap();
        let (mut worn, stowed): (Option<Item>, Item) = serde_json::from_str(&text).unwrap();
        let short = worn.replace(stowed).unwrap();
        assert_eq!(short.firearm.unwrap().loaded, 1);
        let c = worn.as_mut().unwrap().firearm.as_mut().unwrap();
        assert_eq!(c.loaded, 0);
        assert_eq!(c.reload_at, Some(120));
        assert!(!c.complete(lp, 119));
        assert!(c.complete(lp, 120));
    }
    #[test]
    fn slice_and_serialization_boundaries_do_not_shorten_reload() {
        let p = Profile::of("short_gun").unwrap();
        let mut whole = Chambers::loaded(p);
        whole.fire(p, target(1), true).unwrap();
        whole.reload(p, 80).unwrap();
        let mut sliced = whole;
        for now in [81, 84, 90, 94] {
            assert!(!sliced.complete(p, now));
            sliced = serde_json::from_str(&serde_json::to_string(&sliced).unwrap()).unwrap();
        }
        assert!(whole.complete(p, 95));
        assert!(sliced.complete(p, 95));
        assert_eq!(sliced, whole);
    }
    #[test]
    fn burst_requires_both_short_chambers_and_clock_overflow_is_atomic() {
        let lp = Profile::of("long_gun").unwrap();
        let sp = Profile::of("short_gun").unwrap();
        let mut long = Chambers::loaded(lp);
        assert_eq!(long.fire(lp, target(1), true), Err(Refusal::Burst));
        assert_eq!(long.loaded, 1);
        let mut short = Chambers::loaded(sp);
        short.fire(sp, target(1), false).unwrap();
        let original = short;
        assert_eq!(short.fire(sp, target(1), true), Err(Refusal::Empty));
        assert_eq!(short.reload(sp, u32::MAX), Err(Refusal::Clock));
        assert_eq!(short, original);
    }
    #[test]
    fn spread_is_forward_local_and_bounded_by_short_range() {
        let from = Pos::new(0, 0);
        let primary = Pos::new(2, 0);
        for pos in [Pos::new(2, 1), Pos::new(3, -1), Pos::new(1, 1)] { assert!(in_spread(from, primary, pos)); }
        for pos in [from, primary, Pos::new(-1, 0), Pos::new(4, 0), Pos::new(1, 2)] { assert!(!in_spread(from, primary, pos)); }
        assert!(!in_spread(from, from, Pos::new(1, 0)));
    }
    #[test]
    fn ordinary_item_roundtrip_omits_firearm_and_bows_are_not_guns() {
        assert!(Profile::of("bow").is_none());
        let item = Item::new(9, "bow");
        let text = serde_json::to_string(&item).unwrap();
        assert!(!text.contains("firearm"));
        let old: Item = serde_json::from_str(&text).unwrap();
        assert_eq!(old, item);
        assert_eq!(serde_json::to_string(&old).unwrap(), text);
        for kind in ["long_gun", "short_gun"] {
            let gun = Item::new(10, kind);
            let profile = Profile::of(kind).unwrap();
            assert_eq!(gun.def().kind, kind);
            assert_eq!(gun.cat(), crate::defs::Cat::Weapon);
            assert_eq!(gun.atk(), profile.damage);
            assert_eq!(gun.def().weight, 0);
            assert!(!crate::defs::ITEMS.iter().any(|d| d.kind == kind));
        }
    }
}
