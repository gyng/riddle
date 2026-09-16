//! The persistent descent: biome order, bosses, grudge monsters. Floors regenerate per run.
use serde::{Deserialize, Serialize};

/// Cut 7: reaching D34's stairs is the ending (Cut 3: D31; D16 was v1's placeholder). The
/// Warrens run to D8 so the wall arrives when the player has a policy, not a preset.
pub const ENDING_DEPTH: u32 = 34;
/// Cut 7 §1: the lieutenant's floor — the Goblin Captain at D5 (rallies once, no shield wall).
pub const LIEUTENANT_DEPTH: u32 = 5;
/// Cut 7 §3: the band situations (thief's den, gas lock, captive gate, crypt's hunger).
pub const SITUATION_DEPTHS: [(&str, u32); 4] = [("den", 3), ("lock", 6), ("captive", 9), ("hunger", 12)];
/// Cut 3: the hero's sight radius in a lit biome; the Deep is dark (`vision_for`).
pub const VISION_LIT: i32 = 7;
pub const VISION_DARK: i32 = 4;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Biome {
    Warrens,
    Fens,
    Crypt,
    // Cut 3: biomes 4–6, each breaking the program that cleared the last one.
    Foundry,
    Deep,
    Sanctum,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Warrens => "warrens",
            Biome::Fens => "fens",
            Biome::Crypt => "crypt",
            Biome::Foundry => "foundry",
            Biome::Deep => "deep",
            Biome::Sanctum => "sanctum",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Biome::Warrens => "the Warrens",
            Biome::Fens => "the Fens",
            Biome::Crypt => "the Crypt",
            Biome::Foundry => "the Foundry",
            Biome::Deep => "the Deep",
            Biome::Sanctum => "the Sanctum",
        }
    }
    pub const ALL: [Biome; 6] = [Biome::Warrens, Biome::Fens, Biome::Crypt, Biome::Foundry, Biome::Deep, Biome::Sanctum];
    /// Cave floors (cellular, with water): the Fens and the Deep.
    pub fn is_cave(self) -> bool {
        matches!(self, Biome::Fens | Biome::Deep)
    }
    /// Sight radius on this biome's floors (Cut 3: the Deep is dark).
    pub fn vision(self) -> i32 {
        match self {
            Biome::Deep => VISION_DARK,
            _ => VISION_LIT,
        }
    }
}

/// Cut 7: Warrens D1–8 (captain D5, Warlord D8), Fens D9–13, Crypt D14–18, Foundry D19–23,
/// Deep D24–28, Sanctum D29–33, the bottom at D34.
pub fn biome_for(depth: u32) -> Biome {
    match depth {
        0..=8 => Biome::Warrens,
        9..=13 => Biome::Fens,
        14..=18 => Biome::Crypt,
        19..=23 => Biome::Foundry,
        24..=28 => Biome::Deep,
        _ => Biome::Sanctum,
    }
}

/// The first floor of a biome.
pub fn biome_first(biome: Biome) -> u32 {
    match biome {
        Biome::Warrens => 1,
        Biome::Fens => 9,
        Biome::Crypt => 14,
        Biome::Foundry => 19,
        Biome::Deep => 24,
        Biome::Sanctum => 29,
    }
}

/// The boss floors in order (Cut 6 §5: `Lineage.counters` walks them).
pub const BOSS_DEPTHS: [(&str, u32); 6] = [("goblin_warlord", 8), ("bloat_mother", 13), ("lich", 18), ("foundry_master", 23), ("lurker_queen", 28), ("mirror_king", 33)];

pub fn boss_for(depth: u32) -> Option<&'static str> {
    BOSS_DEPTHS.iter().find(|(_, d)| *d == depth).map(|(k, _)| *k)
}

/// The depth of a boss kind.
pub fn boss_depth(kind: &str) -> Option<u32> {
    BOSS_DEPTHS.iter().find(|(k, _)| *k == kind).map(|(_, d)| *d)
}

/// Cut 7 §1: the lieutenant on a floor (the Goblin Captain at D5), placed like a boss but
/// no boss: no counter fact, no marks, a bestiary entry like any other kind.
pub fn lieutenant_for(depth: u32) -> Option<&'static str> {
    (depth == LIEUTENANT_DEPTH).then_some("goblin_captain")
}

/// Cut 7: the content depth — the Cut 3–6 tuning (spawn groups, stat growth, item budgets,
/// item depths) was written against Warrens D1–5 / Fens D6–10 / …; the Warrens' three new
/// floors sit at the old D5's numbers and everything below keeps its old tuning depth.
pub fn tier_depth(depth: u32) -> u32 {
    match depth {
        0..=5 => depth,
        6..=8 => 5,
        _ => depth - 3,
    }
}

/// A named monster that killed an heir; lives on the floor it killed on, +10% stats.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grudge {
    pub kind: String,
    pub name: String,
    pub depth: u32,
    pub heir: u32,
}

const SYL_A: [&str; 10] = ["Gr", "Sk", "Vr", "Th", "Mor", "Ash", "Ul", "Kr", "Zel", "Dr"];
const SYL_B: [&str; 8] = ["ak", "ix", "ul", "eth", "og", "ar", "im", "usk"];

pub fn grudge_name(rng: &mut crate::rng::Rng) -> String {
    format!("{}{}", rng.pick(&SYL_A), rng.pick(&SYL_B))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn biome_order_fixed() {
        assert_eq!(biome_for(1), Biome::Warrens);
        assert_eq!(biome_for(8), Biome::Warrens);
        assert_eq!(biome_for(9), Biome::Fens);
        assert_eq!(biome_for(14), Biome::Crypt);
        assert_eq!(boss_for(8), Some("goblin_warlord"));
        assert_eq!(boss_for(13), Some("bloat_mother"));
        assert_eq!(boss_for(18), Some("lich"));
        assert_eq!(boss_for(5), None);
        assert_eq!(lieutenant_for(5), Some("goblin_captain"));
        assert_eq!(lieutenant_for(8), None);
        assert_eq!(biome_for(19), Biome::Foundry);
        assert_eq!(biome_for(24), Biome::Deep);
        assert_eq!(biome_for(33), Biome::Sanctum);
        assert_eq!(boss_for(23), Some("foundry_master"));
        assert_eq!(boss_for(28), Some("lurker_queen"));
        assert_eq!(boss_for(33), Some("mirror_king"));
        assert_eq!(boss_depth("lich"), Some(18));
        assert_eq!(ENDING_DEPTH, 34);
        for b in Biome::ALL {
            assert_eq!(biome_for(biome_first(b)), b);
            assert!(biome_first(b) == 1 || biome_for(biome_first(b) - 1) != b);
        }
        for (k, d) in BOSS_DEPTHS {
            assert_eq!(boss_for(d), Some(k));
            assert!(d + 1 == ENDING_DEPTH || biome_for(d + 1) != biome_for(d), "{k} guards its biome's last floor");
        }
        assert_eq!((tier_depth(5), tier_depth(6), tier_depth(8), tier_depth(9), tier_depth(33)), (5, 5, 5, 6, 30));
        assert_eq!(Biome::Deep.vision(), 4);
        assert_eq!(Biome::Sanctum.vision(), 7);
    }
}
