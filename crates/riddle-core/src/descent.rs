//! The persistent descent: biome order, bosses, grudge monsters. Floors regenerate per run.
use serde::{Deserialize, Serialize};

/// Cut 3: reaching D31's stairs is the ending (D16 was v1's placeholder).
pub const ENDING_DEPTH: u32 = 31;
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

pub fn biome_for(depth: u32) -> Biome {
    match depth {
        0..=5 => Biome::Warrens,
        6..=10 => Biome::Fens,
        11..=15 => Biome::Crypt,
        16..=20 => Biome::Foundry,
        21..=25 => Biome::Deep,
        _ => Biome::Sanctum,
    }
}

/// The boss floors in order (Cut 6 §5: `Lineage.counters` walks them).
pub const BOSS_DEPTHS: [(&str, u32); 6] = [("goblin_warlord", 5), ("bloat_mother", 10), ("lich", 15), ("foundry_master", 20), ("lurker_queen", 25), ("mirror_king", 30)];

pub fn boss_for(depth: u32) -> Option<&'static str> {
    match depth {
        5 => Some("goblin_warlord"),
        10 => Some("bloat_mother"),
        15 => Some("lich"),
        20 => Some("foundry_master"),
        25 => Some("lurker_queen"),
        30 => Some("mirror_king"),
        _ => None,
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
        assert_eq!(biome_for(5), Biome::Warrens);
        assert_eq!(biome_for(6), Biome::Fens);
        assert_eq!(biome_for(11), Biome::Crypt);
        assert_eq!(boss_for(5), Some("goblin_warlord"));
        assert_eq!(boss_for(10), Some("bloat_mother"));
        assert_eq!(boss_for(15), Some("lich"));
        assert_eq!(boss_for(7), None);
        assert_eq!(biome_for(16), Biome::Foundry);
        assert_eq!(biome_for(21), Biome::Deep);
        assert_eq!(biome_for(30), Biome::Sanctum);
        assert_eq!(boss_for(20), Some("foundry_master"));
        assert_eq!(boss_for(25), Some("lurker_queen"));
        assert_eq!(boss_for(30), Some("mirror_king"));
        assert_eq!(ENDING_DEPTH, 31);
        assert_eq!(Biome::Deep.vision(), 4);
        assert_eq!(Biome::Sanctum.vision(), 7);
    }
}
