//! Explicit upgrades belonging to the current hero, independent of class XP and gold.
use crate::{engine::{Game, LineageState}, hero::Hero, wire::{HeroLegacy, LegacyUpgrade, BloodlineLegacy}};

/// Stable cosmetic identity; never consumes the game's random stream.
pub fn hero_name(seed: u64, heir: u32) -> &'static str {
    const NAMES: [&str; 24] = ["Alden", "Bryn", "Corin", "Dara", "Elian", "Fenn", "Galen", "Hale",
        "Iris", "Jora", "Kael", "Lark", "Maren", "Niall", "Orin", "Petra", "Quill", "Rook",
        "Sable", "Toren", "Una", "Vale", "Wren", "Yara"];
    let first = crate::rng::splitmix(seed) % NAMES.len() as u64;
    NAMES[((first + u64::from(heir.saturating_sub(1))) % NAMES.len() as u64) as usize]
}

/// The family belongs to the persistent slot; given names keep their old derivation.
/// Distinct complete names across active slots without depending on other heirs.
pub fn hero_identity(seed: u64, heir: u32, bloodline_id: u32) -> String {
    let family = match bloodline_id {1 => "Ash".into(), 2 => "Thorn".into(), 3 => "Flint".into(),
        id => format!("Wayfarer{id}")};
    format!("{} {family}", hero_name(seed, heir))
}

pub const CAP: u32 = 3;
pub const IDS: [&str; 3] = ["health", "damage", "armour"];

pub fn current(l: &LineageState) -> Option<&BloodlineLegacy> { l.bloodline.as_ref() }
pub fn ensure(l: &mut LineageState) {
    if l.bloodline.is_none() {
        let mut b = BloodlineLegacy::default();
        for h in l.hero_legacy.iter() {
            b.points += h.points; b.spent += h.spent;
            for (id, rank) in &h.upgrades { let r = b.upgrades.entry(id.clone()).or_default(); *r = (*r).max(*rank).min(CAP); }
        }
        l.bloodline = Some(b);
    }
    if l.hero_legacy.last().is_none_or(|h| h.heir != l.heir) {
        l.hero_legacy.push(HeroLegacy { heir: l.heir, best_depth: l.heir_best, class: l.class.name().into(), ..Default::default() });
    }
}
pub fn offers(l: &LineageState, away: bool) -> Vec<LegacyUpgrade> {
    let h = current(l);
    IDS.iter().map(|id| {
        let rank = h.and_then(|h| h.upgrades.get(*id)).copied().unwrap_or(0);
        let price = 3 * (rank + 1);
        LegacyUpgrade { id: (*id).into(), rank, cap: CAP, price, effect: match *id { "health" => "+3 HP", "damage" => "+1 damage", _ => "+1 armour" }.into(), affordable: !away && l.town.home.unwrap_or(true) && rank < CAP && h.is_some_and(|h| h.points >= price) }
    }).collect()
}
pub fn buy(g: &mut Game, id: &str) -> Result<(), String> {
    if g.run.is_some() || !g.lineage.town.home.unwrap_or(true) { return Err("hero away".into()); }
    let offer = offers(&g.lineage, false).into_iter().find(|u| u.id == id).ok_or("unknown upgrade")?;
    if offer.rank >= CAP { return Err("upgrade complete".into()); }
    if !offer.affordable { return Err("more Legacy needed".into()); }
    ensure(&mut g.lineage);
    let h = g.lineage.bloodline.as_mut().expect("bloodline");
    h.points -= offer.price;
    h.spent += offer.price;
    h.upgrades.insert(id.into(), offer.rank + 1);
    Ok(())
}
pub fn apply(l: &LineageState, hero: &mut Hero) {
    if let Some(h) = current(l) {
        let rank = |id: &str| h.upgrades.get(id).copied().unwrap_or(0).min(CAP) as i32;
        let hp = 3 * rank("health");
        hero.max_hp += hp; hero.max_hp_base += hp; hero.hp += hp;
        hero.str_bonus += rank("damage");
        hero.legacy_armour = rank("armour");
    }
}
