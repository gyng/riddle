//! The persistent descent: biome order, bosses, grudge monsters. Floors regenerate per run.
use serde::{Deserialize, Serialize};

pub const ENDING_DEPTH: u32 = 16;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Biome {
    Warrens,
    Fens,
    Crypt,
}

impl Biome {
    pub fn name(self) -> &'static str {
        match self {
            Biome::Warrens => "warrens",
            Biome::Fens => "fens",
            Biome::Crypt => "crypt",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Biome::Warrens => "the Warrens",
            Biome::Fens => "the Fens",
            Biome::Crypt => "the Crypt",
        }
    }
}

pub fn biome_for(depth: u32) -> Biome {
    match depth {
        0..=5 => Biome::Warrens,
        6..=10 => Biome::Fens,
        _ => Biome::Crypt,
    }
}

pub fn boss_for(depth: u32) -> Option<&'static str> {
    match depth {
        5 => Some("goblin_warlord"),
        10 => Some("bloat_mother"),
        15 => Some("lich"),
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
    }
}
