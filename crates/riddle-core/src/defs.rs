//! Static content tables: monsters, items, spawn tables. Difficulty scales by mix and traits,
//! with only small linear stat growth.
use crate::descent::Biome;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonsterDef {
    pub kind: &'static str,
    pub title: &'static str,
    pub hp: i32,
    pub atk: (i32, i32),
    pub def: i32,
    pub speed: i32,
    pub tags: &'static [&'static str],
    pub boss: bool,
}

pub const MONSTERS: &[MonsterDef] = &[
    MonsterDef { kind: "rat", title: "rat", hp: 4, atk: (1, 2), def: 0, speed: 10, tags: &[], boss: false },
    MonsterDef { kind: "jackal", title: "jackal", hp: 5, atk: (1, 3), def: 0, speed: 15, tags: &["pack", "fast"], boss: false },
    MonsterDef { kind: "goblin", title: "goblin", hp: 9, atk: (2, 4), def: 0, speed: 10, tags: &[], boss: false },
    MonsterDef { kind: "goblin_archer", title: "goblin archer", hp: 7, atk: (2, 4), def: 0, speed: 10, tags: &["ranged", "telegraph"], boss: false },
    MonsterDef { kind: "goblin_conjurer", title: "goblin conjurer", hp: 8, atk: (1, 2), def: 0, speed: 10, tags: &["caster", "summoner"], boss: false },
    MonsterDef { kind: "monkey", title: "monkey", hp: 6, atk: (1, 2), def: 0, speed: 15, tags: &["thief", "fast"], boss: false },
    MonsterDef { kind: "ogre", title: "ogre", hp: 26, atk: (3, 7), def: 0, speed: 10, tags: &["heavy", "telegraph"], boss: false },
    MonsterDef { kind: "bloat", title: "bloat", hp: 4, atk: (0, 0), def: 0, speed: 7, tags: &["gas"], boss: false },
    MonsterDef { kind: "pink_jelly", title: "pink jelly", hp: 20, atk: (1, 3), def: 0, speed: 10, tags: &["splitter"], boss: false },
    MonsterDef { kind: "eel", title: "eel", hp: 12, atk: (3, 6), def: 0, speed: 12, tags: &["water"], boss: false },
    MonsterDef { kind: "skeleton", title: "skeleton", hp: 14, atk: (2, 5), def: 1, speed: 10, tags: &["undead"], boss: false },
    MonsterDef { kind: "ghoul", title: "ghoul", hp: 12, atk: (2, 4), def: 0, speed: 10, tags: &["undead", "pack", "paralyse"], boss: false },
    MonsterDef { kind: "wraith", title: "wraith", hp: 10, atk: (1, 4), def: 0, speed: 12, tags: &["undead", "drain"], boss: false },
    MonsterDef { kind: "captive", title: "captive", hp: 14, atk: (2, 4), def: 0, speed: 10, tags: &["ally"], boss: false },
    MonsterDef { kind: "spectral_blade", title: "spectral blade", hp: 3, atk: (1, 3), def: 0, speed: 15, tags: &["summoned"], boss: false },
    MonsterDef { kind: "spectral_hound", title: "spectral hound", hp: 8, atk: (2, 4), def: 0, speed: 15, tags: &["summoned"], boss: false },
    MonsterDef { kind: "goblin_warlord", title: "Goblin Warlord", hp: 42, atk: (3, 7), def: 1, speed: 10, tags: &["boss", "summoner", "buffer", "telegraph"], boss: true },
    MonsterDef { kind: "bloat_mother", title: "Bloat Mother", hp: 55, atk: (2, 4), def: 0, speed: 7, tags: &["boss", "gas", "telegraph"], boss: true },
    MonsterDef { kind: "lich", title: "Lich", hp: 50, atk: (3, 6), def: 2, speed: 10, tags: &["boss", "undead", "reflect", "summoner", "telegraph"], boss: true },
];

pub fn monster_def(kind: &str) -> &'static MonsterDef {
    MONSTERS.iter().find(|m| m.kind == kind).unwrap_or(&MONSTERS[0])
}

/// The bestiary's observable tags (the 15 monsters + bosses), for vocabulary and tests.
pub fn all_tags() -> Vec<&'static str> {
    let mut v: Vec<&str> = Vec::new();
    for m in MONSTERS {
        for t in m.tags {
            if !v.contains(t) {
                v.push(t);
            }
        }
    }
    v
}

/// (kind, weight, group min, group max) per biome and depth.
pub fn spawn_table(biome: Biome, depth: u32) -> Vec<(&'static str, u32, i32, i32)> {
    let d = depth as i32;
    let mut t: Vec<(&str, u32, i32, i32)> = Vec::new();
    match biome {
        Biome::Warrens => {
            if d <= 3 {
                t.push(("rat", 30, 1, 2));
            }
            t.push(("jackal", 22, 2, 3));
            t.push(("monkey", 10, 1, 1));
            if d >= 2 {
                t.push(("goblin", 20, 1, 2));
            }
            if d >= 3 {
                t.push(("goblin_archer", 12, 1, 1));
            }
            if d >= 4 {
                t.push(("goblin_conjurer", 8, 1, 1));
                t.push(("ogre", 6, 1, 1));
            }
            if d >= 2 {
                t.push(("captive", 5, 1, 1));
            }
        }
        Biome::Fens => {
            t.push(("bloat", 22, 1, 2));
            t.push(("pink_jelly", 14, 1, 1));
            t.push(("eel", 14, 1, 1));
            t.push(("jackal", 10, 2, 3));
            t.push(("goblin", 10, 1, 2));
            t.push(("ogre", 12, 1, 1));
            t.push(("goblin_archer", 8, 1, 1));
            t.push(("captive", 5, 1, 1));
        }
        Biome::Crypt => {
            t.push(("skeleton", 28, 1, 2));
            t.push(("ghoul", 24, 2, 3));
            t.push(("wraith", 16, 1, 1));
            t.push(("ogre", 10, 1, 1));
            t.push(("goblin_conjurer", 6, 1, 1));
            t.push(("bloat", 6, 1, 1));
            t.push(("captive", 5, 1, 1));
        }
    }
    t
}

/// Number of spawn groups on a floor.
pub fn group_budget(depth: u32) -> i32 {
    3 + (depth as i32) / 2
}

/// Small linear stat growth with depth: +1 hp per 2 floors, +1 max atk per 5 floors.
pub fn depth_hp_bonus(depth: u32) -> i32 {
    (depth as i32) / 2
}
pub fn depth_atk_bonus(depth: u32) -> i32 {
    (depth as i32) / 5
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    Weapon,
    Armour,
    Potion,
    Scroll,
    Gold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDef {
    pub kind: &'static str,
    pub cat: Cat,
    /// weapon: (min,max) damage; armour: (def,0); potion/scroll: unused.
    pub a: (i32, i32),
    pub speed: i32,
    pub ranged: bool,
    pub value: i32,
    pub benevolent: bool,
    pub weight: u32,
}

pub const ITEMS: &[ItemDef] = &[
    ItemDef { kind: "dagger", cat: Cat::Weapon, a: (2, 4), speed: 0, ranged: false, value: 10, benevolent: true, weight: 4 },
    ItemDef { kind: "sword", cat: Cat::Weapon, a: (3, 7), speed: 0, ranged: false, value: 25, benevolent: true, weight: 3 },
    ItemDef { kind: "axe", cat: Cat::Weapon, a: (4, 9), speed: -1, ranged: false, value: 30, benevolent: true, weight: 2 },
    ItemDef { kind: "bow", cat: Cat::Weapon, a: (2, 6), speed: 0, ranged: true, value: 25, benevolent: true, weight: 2 },
    ItemDef { kind: "leather", cat: Cat::Armour, a: (1, 0), speed: 0, ranged: false, value: 15, benevolent: true, weight: 4 },
    ItemDef { kind: "mail", cat: Cat::Armour, a: (3, 0), speed: -1, ranged: false, value: 30, benevolent: true, weight: 2 },
    ItemDef { kind: "plate", cat: Cat::Armour, a: (5, 0), speed: -2, ranged: false, value: 45, benevolent: true, weight: 1 },
    ItemDef { kind: "heal", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 12, benevolent: true, weight: 9 },
    ItemDef { kind: "strength", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 20, benevolent: true, weight: 2 },
    ItemDef { kind: "speed", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 3 },
    ItemDef { kind: "invisibility", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 2 },
    ItemDef { kind: "poison", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: false, weight: 4 },
    ItemDef { kind: "caustic", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: false, weight: 3 },
    ItemDef { kind: "confusion", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: false, weight: 3 },
    ItemDef { kind: "fire", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: false, weight: 2 },
    ItemDef { kind: "teleport", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 12, benevolent: true, weight: 4 },
    ItemDef { kind: "blink", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 4 },
    ItemDef { kind: "fear", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 3 },
    ItemDef { kind: "mapping", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 3 },
    ItemDef { kind: "identify", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 3 },
    ItemDef { kind: "enchant", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 20, benevolent: true, weight: 2 },
    ItemDef { kind: "darkness", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 2 },
    ItemDef { kind: "summon_ally", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 15, benevolent: true, weight: 2 },
    ItemDef { kind: "aggravate", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 2, benevolent: false, weight: 3 },
    ItemDef { kind: "gold", cat: Cat::Gold, a: (0, 0), speed: 0, ranged: false, value: 1, benevolent: true, weight: 0 },
];

pub fn item_def(kind: &str) -> &'static ItemDef {
    ITEMS.iter().find(|i| i.kind == kind).unwrap_or(&ITEMS[0])
}

pub const POTION_FLAVOURS: [&str; 8] = ["blue", "murky", "red", "amber", "green", "violet", "black", "clear"];
pub const SCROLL_FLAVOURS: [&str; 9] = ["runed", "torn", "sealed", "ashen", "crimson", "folded", "mossy", "silver", "burnt"];

pub fn potion_kinds() -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == Cat::Potion).map(|i| i.kind).collect()
}
pub fn scroll_kinds() -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == Cat::Scroll).map(|i| i.kind).collect()
}

/// Items per floor.
pub fn item_budget(depth: u32) -> i32 {
    4 + (depth as i32) / 4
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_counts() {
        assert_eq!(MONSTERS.iter().filter(|m| !m.boss && !m.tags.contains(&"summoned")).count(), 14);
        assert_eq!(MONSTERS.iter().filter(|m| m.boss).count(), 3);
        assert_eq!(ITEMS.len(), 25);
        assert_eq!(potion_kinds().len(), 8);
        assert_eq!(scroll_kinds().len(), 9);
    }
    #[test]
    fn every_biome_has_spawns() {
        for d in 1..=15 {
            let t = spawn_table(crate::descent::biome_for(d), d);
            assert!(!t.is_empty());
            for (k, ..) in t {
                assert_eq!(monster_def(k).kind, k);
            }
        }
    }
}
