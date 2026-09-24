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
    MonsterDef { kind: "jackal", title: "jackal", hp: 4, atk: (1, 2), def: 0, speed: 15, tags: &["pack", "fast"], boss: false },
    MonsterDef { kind: "goblin", title: "goblin", hp: 7, atk: (1, 3), def: 0, speed: 10, tags: &[], boss: false },
    MonsterDef { kind: "goblin_archer", title: "goblin archer", hp: 7, atk: (1, 3), def: 0, speed: 10, tags: &["ranged", "telegraph"], boss: false },
    MonsterDef { kind: "goblin_conjurer", title: "goblin conjurer", hp: 8, atk: (1, 2), def: 0, speed: 10, tags: &["caster", "summoner"], boss: false },
    MonsterDef { kind: "monkey", title: "monkey", hp: 6, atk: (1, 2), def: 0, speed: 15, tags: &["thief", "fast"], boss: false },
    MonsterDef { kind: "ogre", title: "ogre", hp: 30, atk: (2, 4), def: 0, speed: 7, tags: &["heavy", "telegraph"], boss: false },
    MonsterDef { kind: "bloat", title: "bloat", hp: 4, atk: (0, 0), def: 0, speed: 7, tags: &["gas"], boss: false },
    MonsterDef { kind: "pink_jelly", title: "pink jelly", hp: 20, atk: (1, 3), def: 0, speed: 10, tags: &["splitter"], boss: false },
    MonsterDef { kind: "eel", title: "eel", hp: 12, atk: (3, 6), def: 0, speed: 12, tags: &["water"], boss: false },
    MonsterDef { kind: "skeleton", title: "skeleton", hp: 14, atk: (2, 4), def: 1, speed: 10, tags: &["undead"], boss: false },
    MonsterDef { kind: "ghoul", title: "ghoul", hp: 12, atk: (2, 3), def: 0, speed: 10, tags: &["undead", "pack", "paralyse"], boss: false },
    MonsterDef { kind: "wraith", title: "wraith", hp: 10, atk: (1, 3), def: 0, speed: 12, tags: &["undead", "drain"], boss: false },
    MonsterDef { kind: "captive", title: "captive", hp: 14, atk: (2, 4), def: 0, speed: 10, tags: &["ally"], boss: false },
    MonsterDef { kind: "spectral_blade", title: "spectral blade", hp: 3, atk: (1, 2), def: 0, speed: 12, tags: &["summoned"], boss: false },
    MonsterDef { kind: "spectral_hound", title: "spectral hound", hp: 8, atk: (2, 4), def: 0, speed: 15, tags: &["summoned"], boss: false },
    // Cut 7 §1: the lieutenant at D5 — rallies once, no shield wall; the preset beats him about
    // half the time. Not a boss: no counter fact, no marks, the ledger lists him like any kind.
    MonsterDef { kind: "goblin_captain", title: "Goblin Captain", hp: 12, atk: (2, 4), def: 0, speed: 10, tags: &["summoner", "telegraph"], boss: false },
    MonsterDef { kind: "goblin_warlord", title: "Goblin Warlord", hp: 24, atk: (2, 5), def: 0, speed: 10, tags: &["boss", "summoner", "buffer", "telegraph"], boss: true },
    MonsterDef { kind: "bloat_mother", title: "Bloat Mother", hp: 20, atk: (2, 3), def: 0, speed: 7, tags: &["boss", "gas", "telegraph"], boss: true },
    MonsterDef { kind: "lich", title: "Lich", hp: 50, atk: (3, 5), def: 2, speed: 10, tags: &["boss", "undead", "reflect", "summoner", "telegraph"], boss: true },
    // Cut 3 — the Foundry (Cut 7: D19–23): melee is reflected, bells raise the clock.
    MonsterDef { kind: "iron_golem", title: "iron golem", hp: 24, atk: (3, 6), def: 1, speed: 4, tags: &["reflect_melee"], boss: false },
    MonsterDef { kind: "forge_imp", title: "forge imp", hp: 9, atk: (1, 3), def: 0, speed: 13, tags: &["fire", "thief"], boss: false },
    MonsterDef { kind: "bell_sentinel", title: "bell sentinel", hp: 10, atk: (1, 2), def: 0, speed: 10, tags: &["alarm"], boss: false },
    MonsterDef { kind: "slag_crawler", title: "slag crawler", hp: 26, atk: (2, 5), def: 0, speed: 7, tags: &["heavy", "fire", "telegraph"], boss: false },
    MonsterDef { kind: "smith", title: "smith", hp: 12, atk: (2, 4), def: 0, speed: 10, tags: &["buffer"], boss: false },
    // The Deep (Cut 7: D24–28): dark, hunters track noise, regen.
    MonsterDef { kind: "lurker", title: "lurker", hp: 10, atk: (2, 5), def: 0, speed: 12, tags: &["blind"], boss: false },
    MonsterDef { kind: "deep_eel", title: "deep eel", hp: 16, atk: (4, 7), def: 0, speed: 12, tags: &["water"], boss: false },
    MonsterDef { kind: "cave_troll", title: "cave troll", hp: 34, atk: (3, 6), def: 0, speed: 8, tags: &["regen"], boss: false },
    MonsterDef { kind: "siren", title: "siren", hp: 14, atk: (1, 3), def: 0, speed: 10, tags: &["aura"], boss: false },
    MonsterDef { kind: "mirror_shade", title: "mirror shade", hp: 14, atk: (2, 4), def: 0, speed: 10, tags: &["mirror"], boss: false },
    // The Sanctum (Cut 7: D29–33): variety is enforced.
    MonsterDef { kind: "warden", title: "warden", hp: 30, atk: (3, 6), def: 1, speed: 9, tags: &["reflect_melee", "reflect", "telegraph"], boss: false },
    MonsterDef { kind: "acolyte", title: "acolyte", hp: 12, atk: (1, 3), def: 0, speed: 10, tags: &["healer"], boss: false },
    MonsterDef { kind: "echo", title: "echo", hp: 16, atk: (2, 5), def: 0, speed: 10, tags: &["echo"], boss: false },
    MonsterDef { kind: "sentinel", title: "sentinel", hp: 26, atk: (3, 6), def: 1, speed: 8, tags: &["gaze", "telegraph"], boss: false },
    // Cut 3 bosses.
    MonsterDef { kind: "foundry_master", title: "Foundry Master", hp: 42, atk: (3, 6), def: 1, speed: 8, tags: &["boss", "reflect_melee", "buffer", "telegraph"], boss: true },
    MonsterDef { kind: "lurker_queen", title: "Lurker Queen", hp: 60, atk: (4, 7), def: 0, speed: 10, tags: &["boss", "blind", "summoner", "telegraph"], boss: true },
    MonsterDef { kind: "mirror_king", title: "Mirror King", hp: 80, atk: (3, 6), def: 1, speed: 10, tags: &["boss", "mirror", "telegraph"], boss: true },
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
        // Cut 16 §3: the Burrows (D5–8) are the Warrens' old deep end with their own mix —
        // monkeys and archers up, jackals thinner (the rats stay on D1–3).
        Biome::Burrows => {
            t.push(("jackal", 14, 2, 3));
            t.push(("monkey", 16, 1, 1));
            t.push(("goblin", 20, 1, if d >= 6 { 3 } else { 2 }));
            t.push(("goblin_archer", if d >= 6 { 18 } else { 14 }, 1, if d >= 6 { 2 } else { 1 }));
            t.push(("goblin_conjurer", if d >= 6 { 8 } else { 6 }, 1, 1));
            t.push(("ogre", if d >= 6 { 10 } else { 6 }, 1, 1));
            t.push(("captive", 5, 1, 1));
        }
        Biome::Warrens => {
            // D1 is the doorstep: rats, monkeys and lone goblins. Packs from D2, archers from D4.
            // Cut 7: D6–8 are the Warrens' deep end — goblin bands, archers in pairs, ogres;
            // the first bloats are the D6 gas lock's (`situations`).
            if d <= 3 {
                t.push(("rat", 30, 1, 2));
            }
            if d >= 2 {
                t.push(("jackal", 22, 2, 3));
            }
            t.push(("monkey", 10, 1, 1));
            t.push(("goblin", if d >= 2 { 20 } else { 12 }, 1, if d >= 6 { 3 } else if d >= 2 { 2 } else { 1 }));
            if d >= 4 {
                t.push(("goblin_archer", if d >= 6 { 16 } else { 12 }, 1, if d >= 6 { 2 } else { 1 }));
            }
            if d >= 4 {
                t.push(("goblin_conjurer", if d >= 6 { 8 } else { 6 }, 1, 1));
                t.push(("ogre", if d >= 6 { 10 } else { 6 }, 1, 1));
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
        // Cut 3.
        Biome::Foundry => {
            t.push(("iron_golem", 10, 1, 1));
            t.push(("forge_imp", 18, 1, 2));
            t.push(("bell_sentinel", 12, 1, 1));
            t.push(("slag_crawler", 12, 1, 1));
            t.push(("smith", 14, 1, 2));
            t.push(("skeleton", 8, 1, 2));
            t.push(("captive", 5, 1, 1));
        }
        Biome::Deep => {
            t.push(("lurker", 24, 1, 2));
            t.push(("deep_eel", 12, 1, 1));
            t.push(("cave_troll", 12, 1, 1));
            t.push(("siren", 12, 1, 1));
            t.push(("mirror_shade", 14, 1, 2));
            t.push(("captive", 5, 1, 1));
        }
        Biome::Sanctum => {
            t.push(("warden", 16, 1, 1));
            t.push(("acolyte", 14, 1, 2));
            t.push(("echo", 18, 1, 2));
            t.push(("sentinel", 14, 1, 1));
            t.push(("mirror_shade", 10, 1, 1));
            t.push(("captive", 5, 1, 1));
        }
    }
    t
}

/// Number of spawn groups on a floor (32×32 floors, Cut 2 §1). Cut 3: the count stops
/// growing past the Crypt (nine groups); the deep biomes' kinds carry the difficulty, not the
/// crowd. Cut 7: keyed on the content depth (`descent::tier_depth`); the Warrens' tail D6–8
/// grows by one group.
pub fn group_budget(depth: u32) -> i32 {
    let d = crate::descent::tier_depth(depth) + (6..=8).contains(&depth) as u32;
    1 + (d.min(15) as i32 + 1) / 2
}

/// Small linear stat growth with depth: +1 hp per 2 floors, +1 max atk per 6 floors, both
/// frozen from the Foundry (Cut 3: the new kinds are tuned on their own numbers). Cut 7: on
/// the content depth.
pub fn depth_hp_bonus(depth: u32) -> i32 {
    (crate::descent::tier_depth(depth).min(16) as i32) / 2
}
pub fn depth_atk_bonus(depth: u32) -> i32 {
    (crate::descent::tier_depth(depth).min(16) as i32) / 6
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cat {
    Weapon,
    Armour,
    Potion,
    Scroll,
    Gold,
    Misc,
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
    ItemDef { kind: "heal", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 12, benevolent: true, weight: 6 },
    ItemDef { kind: "strength", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 20, benevolent: true, weight: 2 },
    ItemDef { kind: "speed", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 3 },
    ItemDef { kind: "invisibility", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 2 },
    ItemDef { kind: "poison", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 14, benevolent: false, weight: 4 },
    ItemDef { kind: "caustic", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: false, weight: 3 },
    ItemDef { kind: "confusion", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: false, weight: 3 },
    ItemDef { kind: "fire", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 14, benevolent: false, weight: 2 },
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
    ItemDef { kind: "leash", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 6, benevolent: true, weight: 0 },
    // Cut 2: a dead heir's kit on the floor (§2) and the ranger's trap (§4). Never loot.
    ItemDef { kind: "bones", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 0, benevolent: true, weight: 0 },
    ItemDef { kind: "trap", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 0, benevolent: true, weight: 0 },
    // Cut 3: 15 dual-use items. Gear appears from the Crypt on (`populate_floor` gates it).
    ItemDef { kind: "spear", cat: Cat::Weapon, a: (3, 6), speed: 0, ranged: false, value: 28, benevolent: true, weight: 2 },
    ItemDef { kind: "mace", cat: Cat::Weapon, a: (4, 8), speed: 0, ranged: false, value: 32, benevolent: true, weight: 2 },
    ItemDef { kind: "scale", cat: Cat::Armour, a: (4, 0), speed: -1, ranged: false, value: 38, benevolent: true, weight: 2 },
    ItemDef { kind: "regen", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 14, benevolent: true, weight: 2 },
    ItemDef { kind: "resist_fire", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 2 },
    ItemDef { kind: "clarity", cat: Cat::Potion, a: (0, 0), speed: 0, ranged: false, value: 10, benevolent: true, weight: 2 },
    ItemDef { kind: "recall", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 20, benevolent: true, weight: 2 },
    ItemDef { kind: "silence", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 18, benevolent: true, weight: 2 },
    ItemDef { kind: "earthquake", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 12, benevolent: true, weight: 1 },
    ItemDef { kind: "mirror", cat: Cat::Scroll, a: (0, 0), speed: 0, ranged: false, value: 14, benevolent: true, weight: 2 },
    ItemDef { kind: "lantern", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 12, benevolent: true, weight: 2 },
    ItemDef { kind: "bell", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: true, weight: 2 },
    ItemDef { kind: "salt", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 8, benevolent: true, weight: 2 },
    ItemDef { kind: "chalk", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 6, benevolent: true, weight: 2 },
    ItemDef { kind: "mirror_shard", cat: Cat::Misc, a: (0, 0), speed: 0, ranged: false, value: 16, benevolent: true, weight: 1 },
];

/// Cut 3: misc items the hero can throw (`throw bell` lures hunters, `throw salt` routs undead).
pub const THROWABLE_MISC: [&str; 2] = ["bell", "salt"];
/// Cut 3: misc items whose pickup is a fact (`item:<kind>`), gating unlocks and tokens.
pub const FACT_MISC: [&str; 5] = ["lantern", "bell", "salt", "chalk", "mirror_shard"];

/// The content depth (`descent::tier_depth`) from which a Cut 3 item may appear (0 = anywhere).
pub fn item_min_depth(kind: &str) -> u32 {
    match kind {
        "axe" | "bow" | "mail" => 4,
        "plate" => 8,
        "spear" => 10,
        "mace" | "scale" | "salt" | "chalk" => 12,
        "lantern" | "bell" => 14,
        "mirror_shard" => 18,
        "regen" | "resist_fire" | "clarity" | "recall" | "silence" | "earthquake" | "mirror" => 6,
        _ => 0,
    }
}

/// Counters (Addendum A): (winner, loser, immune). For the first three the winner is the
/// attacker's tag; for the last two the winner is the defender's tag against the attack's tag.
pub const COUNTERS: &[(&str, &str, bool)] = &[
    ("ranged", "heavy", false),
    ("pack", "lone", false),
    ("gas", "pack", false),
    ("water", "fire", true),
    ("undead", "poison", true),
];

pub fn counter_fact(a: &str, b: &str) -> String {
    format!("counter:{a}>{b}")
}

/// Companion verb granted by a tag.
pub fn tag_verb(tag: &str) -> Option<&'static str> {
    match tag {
        "ranged" => Some("shoot"),
        "gas" => Some("burst"),
        "thief" => Some("steal"),
        "splitter" => Some("split"),
        "pack" => Some("flank"),
        "undead" => Some("drain"),
        // Cut 3: a bred `mirror` tag copies the hero's class verb.
        "mirror" => Some("mimic"),
        _ => None,
    }
}

/// Kinds a biome's ledger needs tamed (tameable spawns only).
pub fn biome_kinds(biome: Biome) -> Vec<&'static str> {
    let mut v: Vec<&str> = Vec::new();
    for d in 1..crate::descent::ENDING_DEPTH {
        if crate::descent::biome_for(d) != biome {
            continue;
        }
        for (k, ..) in spawn_table(biome, d) {
            if k != "captive" && !v.contains(&k) {
                v.push(k);
            }
        }
    }
    v
}

pub fn item_def(kind: &str) -> &'static ItemDef {
    ITEMS.iter().find(|i| i.kind == kind).unwrap_or(&ITEMS[0])
}

pub const POTION_FLAVOURS: [&str; 11] = ["blue", "murky", "red", "amber", "green", "violet", "black", "clear", "smoky", "pearly", "oily"];
pub const SCROLL_FLAVOURS: [&str; 13] = ["runed", "torn", "sealed", "ashen", "crimson", "folded", "mossy", "silver", "burnt", "waxen", "inked", "brittle", "gilded"];

pub fn potion_kinds() -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == Cat::Potion).map(|i| i.kind).collect()
}
pub fn scroll_kinds() -> Vec<&'static str> {
    ITEMS.iter().filter(|i| i.cat == Cat::Scroll).map(|i| i.kind).collect()
}

/// Items per floor (32×32 floors, Cut 2 §1); Cut 7: on the content depth.
pub fn item_budget(depth: u32) -> i32 {
    7 + (crate::descent::tier_depth(depth) as i32) / 3
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn content_counts() {
        assert_eq!(MONSTERS.iter().filter(|m| !m.boss && !m.tags.contains(&"summoned")).count(), 29, "14 + the Cut 3 biomes' 14 + the Cut 7 captain");
        assert_eq!(MONSTERS.iter().filter(|m| m.boss).count(), 6);
        assert_eq!(ITEMS.len(), 43, "25 items + the leash (Addendum A) + bones and trap (Cut 2) + 15 (Cut 3)");
        assert_eq!(potion_kinds().len(), 11);
        assert_eq!(scroll_kinds().len(), 13);
        assert!(POTION_FLAVOURS.len() >= potion_kinds().len());
        assert!(SCROLL_FLAVOURS.len() >= scroll_kinds().len());
        for k in ["iron_golem", "forge_imp", "bell_sentinel", "slag_crawler", "smith", "lurker", "deep_eel", "cave_troll", "siren", "mirror_shade", "warden", "acolyte", "echo", "sentinel", "foundry_master", "lurker_queen", "mirror_king", "goblin_captain"] {
            assert_eq!(monster_def(k).kind, k, "{k} is a sprite key");
        }
    }
    #[test]
    fn every_biome_has_spawns() {
        for d in 1..crate::descent::ENDING_DEPTH {
            let t = spawn_table(crate::descent::biome_for(d), d);
            assert!(!t.is_empty());
            for (k, ..) in t {
                assert_eq!(monster_def(k).kind, k);
            }
        }
    }
}
