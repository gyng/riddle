//! The hero: stats, inventory, equipment, statuses. No XP; power comes from gear and potions.
use crate::defs::Cat;
use crate::geom::Pos;
use crate::item::Item;
use serde::{Deserialize, Serialize};

pub const INV_SLOTS: usize = 10;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Fighter,
    Rogue,
    Ranger,
    Caster,
}

impl Class {
    pub const ALL: [Class; 4] = [Class::Fighter, Class::Rogue, Class::Ranger, Class::Caster];
    pub fn name(self) -> &'static str {
        match self {
            Class::Fighter => "fighter",
            Class::Rogue => "rogue",
            Class::Ranger => "ranger",
            Class::Caster => "caster",
        }
    }
    pub fn parse(s: &str) -> Option<Class> {
        match s {
            "fighter" => Some(Class::Fighter),
            "rogue" => Some(Class::Rogue),
            "ranger" => Some(Class::Ranger),
            "caster" => Some(Class::Caster),
            _ => None,
        }
    }
    /// The unlock that opens the class (the fighter is free).
    pub fn unlock(self) -> Option<&'static str> {
        match self {
            Class::Fighter => None,
            Class::Rogue => Some("rogue"),
            Class::Ranger => Some("ranger"),
            Class::Caster => Some("caster"),
        }
    }
    /// Starting arm (item id 1 is never loot).
    pub fn starting_weapon(self) -> &'static str {
        match self {
            Class::Fighter => "sword",
            Class::Rogue | Class::Caster => "dagger",
            Class::Ranger => "bow",
        }
    }
    pub fn base_hp(self) -> i32 {
        match self {
            Class::Fighter => 36,
            Class::Rogue => 28,
            Class::Ranger => 30,
            Class::Caster => 24,
        }
    }
}

/// Class verb ladder (Addendum C, Cut 2 §4): (verb, level).
pub fn class_ladder(class: Class) -> &'static [(&'static str, u32)] {
    match class {
        Class::Fighter => &[("shield_bash", 1), ("cleave", 3), ("taunt", 5), ("second_wind", 7), ("bulwark", 9)],
        Class::Rogue => &[("vanish", 1), ("throw", 1), ("backstab", 3), ("smoke", 5), ("ambush", 7), ("shadowstep", 9)],
        Class::Ranger => &[("shoot", 1), ("kite", 1), ("volley", 3), ("trap", 5), ("mark", 7), ("double_shot", 9)],
        Class::Caster => &[("bolt", 1), ("ward", 1), ("blink", 3), ("slow", 5), ("nova", 7), ("drain", 9)],
    }
}

pub fn class_has_verb(class: Class, level: u32, verb: &str) -> bool {
    class_ladder(class).iter().any(|(v, l)| *v == verb && level >= *l)
}

/// Cut 2 §2: `100 × level²`; XP only from banked and returned runs.
pub fn xp_to_next(level: u32) -> u32 {
    100 * level * level
}

pub fn mastery_card(class: Class) -> &'static str {
    match class {
        Class::Fighter => "phalanx",
        Class::Rogue => "hit_and_fade",
        Class::Ranger => "hawkeye",
        Class::Caster => "archmage",
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Trait {
    Greedy,
    Cowardly,
    Curious,
    #[default]
    Brave,
}

impl Trait {
    pub const ALL: [Trait; 4] = [Trait::Greedy, Trait::Cowardly, Trait::Curious, Trait::Brave];
    pub fn name(self) -> &'static str {
        match self {
            Trait::Greedy => "greedy",
            Trait::Cowardly => "cowardly",
            Trait::Curious => "curious",
            Trait::Brave => "brave",
        }
    }
    /// Cut 5 §4: the shrine's trait swap (`pray trait`): greed ↔ curiosity, cowardice ↔ bravery.
    pub fn swap(self) -> Trait {
        match self {
            Trait::Greedy => Trait::Curious,
            Trait::Curious => Trait::Greedy,
            Trait::Cowardly => Trait::Brave,
            Trait::Brave => Trait::Cowardly,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hero {
    pub pos: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub base_atk: (i32, i32),
    pub str_bonus: i32,
    pub class: Class,
    pub inv: Vec<Item>,
    pub weapon: Option<Item>,
    pub armour: Option<Item>,
    pub energy: i32,
    pub speed_t: i32,
    pub invis_t: i32,
    pub vanish_t: i32,
    pub paralysed: i32,
    pub confused: i32,
    pub poison: (i32, i32),
    pub bash_cd: i32,
    pub vanish_cd: i32,
    pub resting: bool,
    /// Class level (Addendum C) and its verb cooldowns/states.
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub cleave_cd: i32,
    #[serde(default)]
    pub bulwark_t: i32,
    #[serde(default)]
    pub bulwark_cd: i32,
    #[serde(default)]
    pub second_wind_used: bool,
    // Cut 2 §4: ranger and caster cooldowns and buffs (ticks).
    #[serde(default)]
    pub volley_cd: i32,
    #[serde(default)]
    pub double_cd: i32,
    #[serde(default)]
    pub ward_t: i32,
    #[serde(default)]
    pub ward_cd: i32,
    #[serde(default)]
    pub blink_cd: i32,
    #[serde(default)]
    pub nova_cd: i32,
    // Cut 3: silence (no noise), regen (+2 per 10 ticks), fire resistance, clarity (immune to
    // confusion), and the mirror scroll's charge (the next hit taken comes back), in ticks.
    #[serde(default)]
    pub silence_t: i32,
    #[serde(default)]
    pub regen_t: i32,
    #[serde(default)]
    pub resist_fire_t: i32,
    #[serde(default)]
    pub clarity_t: i32,
    #[serde(default)]
    pub mirror_charge: i32,
    /// Cut 3: max HP before any drain (the level's), recovered a floor at a time.
    #[serde(default)]
    pub max_hp_base: i32,
}

impl Hero {
    pub fn new(class: Class, pos: Pos) -> Hero {
        let max_hp = class.base_hp();
        Hero {
            pos,
            hp: max_hp,
            max_hp,
            base_atk: (1, 2),
            str_bonus: 0,
            class,
            inv: Vec::new(),
            weapon: None,
            armour: None,
            energy: 0,
            speed_t: 0,
            invis_t: 0,
            vanish_t: 0,
            paralysed: 0,
            confused: 0,
            poison: (0, 0),
            bash_cd: 0,
            vanish_cd: 0,
            resting: false,
            level: 1,
            cleave_cd: 0,
            bulwark_t: 0,
            bulwark_cd: 0,
            second_wind_used: false,
            volley_cd: 0,
            double_cd: 0,
            ward_t: 0,
            ward_cd: 0,
            blink_cd: 0,
            nova_cd: 0,
            silence_t: 0,
            regen_t: 0,
            resist_fire_t: 0,
            clarity_t: 0,
            mirror_charge: 0,
            max_hp_base: max_hp,
        }
    }
    /// Apply a class level: +2 max_hp per level past 1, +1 atk at L3/L6/L9.
    pub fn apply_level(&mut self, level: u32) {
        self.level = level.clamp(1, 10);
        let extra = 2 * (self.level as i32 - 1);
        self.max_hp += extra;
        self.max_hp_base = self.max_hp;
        self.hp = self.max_hp;
        self.str_bonus += [3, 6, 9].iter().filter(|l| self.level >= **l).count() as i32;
    }
    pub fn atk(&self) -> (i32, i32) {
        let (lo, hi) = self.weapon.as_ref().map(|w| w.atk()).unwrap_or(self.base_atk);
        (lo + self.str_bonus, hi + self.str_bonus)
    }
    pub fn def(&self) -> i32 {
        self.armour.as_ref().map(|a| a.def_bonus()).unwrap_or(0) + if self.bulwark_t > 0 { 3 } else { 0 } + if self.ward_t > 0 { 2 } else { 0 }
    }
    pub fn speed(&self) -> i32 {
        let mut s = 10;
        if let Some(w) = &self.weapon {
            s += w.def().speed;
        }
        if let Some(a) = &self.armour {
            s += a.def().speed;
        }
        if self.speed_t > 0 {
            s += 5;
        }
        s
    }
    pub fn ranged(&self) -> bool {
        self.weapon.as_ref().is_some_and(|w| w.def().ranged)
    }
    pub fn hp_pct(&self) -> i32 {
        if self.max_hp <= 0 {
            0
        } else {
            (self.hp * 100 / self.max_hp).clamp(0, 100)
        }
    }
    pub fn untargetable(&self) -> bool {
        self.vanish_t > 0
    }
    pub fn inv_full(&self) -> bool {
        self.inv.len() >= INV_SLOTS
    }
    pub fn has_kind(&self, kind: &str) -> bool {
        self.inv.iter().any(|i| i.kind == kind)
            || self.weapon.as_ref().is_some_and(|w| w.kind == kind)
            || self.armour.as_ref().is_some_and(|a| a.kind == kind)
    }
    pub fn take_kind(&mut self, kind: &str) -> Option<Item> {
        let i = self.inv.iter().position(|i| i.kind == kind)?;
        Some(self.inv.remove(i))
    }
    /// Equip if better than the current piece; returns the replaced item (kept in inventory).
    pub fn auto_equip(&mut self, item: Item) -> Option<Item> {
        match item.cat() {
            Cat::Weapon => {
                let cur = self.weapon.as_ref().map(|w| w.atk().0 + w.atk().1).unwrap_or(0);
                let new = item.atk().0 + item.atk().1;
                if new > cur {
                    let old = self.weapon.replace(item);
                    if let Some(o) = old {
                        if !self.inv_full() {
                            self.inv.push(o.clone());
                        }
                        return Some(o);
                    }
                } else if !self.inv_full() {
                    self.inv.push(item);
                }
            }
            Cat::Armour => {
                let cur = self.armour.as_ref().map(|a| a.def_bonus()).unwrap_or(0);
                if item.def_bonus() > cur {
                    let old = self.armour.replace(item);
                    if let Some(o) = old {
                        if !self.inv_full() {
                            self.inv.push(o.clone());
                        }
                        return Some(o);
                    }
                } else if !self.inv_full() {
                    self.inv.push(item);
                }
            }
            _ => {
                if !self.inv_full() {
                    self.inv.push(item);
                }
            }
        }
        None
    }
    pub fn tick_statuses(&mut self) {
        if self.speed_t > 0 {
            self.speed_t -= 1;
        }
        if self.invis_t > 0 {
            self.invis_t -= 1;
        }
        if self.vanish_t > 0 {
            self.vanish_t -= 1;
        }
        if self.paralysed > 0 {
            self.paralysed -= 1;
        }
        if self.confused > 0 {
            self.confused -= 1;
        }
        if self.bash_cd > 0 {
            self.bash_cd -= 1;
        }
        if self.vanish_cd > 0 {
            self.vanish_cd -= 1;
        }
        if self.cleave_cd > 0 {
            self.cleave_cd -= 1;
        }
        if self.bulwark_t > 0 {
            self.bulwark_t -= 1;
        }
        if self.bulwark_cd > 0 {
            self.bulwark_cd -= 1;
        }
        for c in [
            &mut self.volley_cd,
            &mut self.double_cd,
            &mut self.ward_t,
            &mut self.ward_cd,
            &mut self.blink_cd,
            &mut self.nova_cd,
            &mut self.silence_t,
            &mut self.regen_t,
            &mut self.resist_fire_t,
            &mut self.clarity_t,
        ] {
            if *c > 0 {
                *c -= 1;
            }
        }
    }
    pub fn status_tags(&self) -> Vec<String> {
        let mut t = Vec::new();
        if self.speed_t > 0 {
            t.push("fast".into());
        }
        if self.invis_t > 0 {
            t.push("invisible".into());
        }
        if self.vanish_t > 0 {
            t.push("vanished".into());
        }
        if self.paralysed > 0 {
            t.push("paralysed".into());
        }
        if self.confused > 0 {
            t.push("confused".into());
        }
        if self.poison.1 > 0 {
            t.push("poisoned".into());
        }
        if self.bulwark_t > 0 {
            t.push("bulwark".into());
        }
        if self.ward_t > 0 {
            t.push("warded".into());
        }
        if self.silence_t > 0 {
            t.push("silent".into());
        }
        if self.regen_t > 0 {
            t.push("regen".into());
        }
        if self.resist_fire_t > 0 {
            t.push("fireproof".into());
        }
        if self.clarity_t > 0 {
            t.push("clear".into());
        }
        if self.mirror_charge > 0 {
            t.push("mirrored".into());
        }
        t
    }
    /// Cut 3: the wielded weapon's kind.
    pub fn weapon_kind(&self) -> &str {
        self.weapon.as_ref().map(|w| w.kind.as_str()).unwrap_or("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn auto_equip_prefers_better() {
        let mut h = Hero::new(Class::Fighter, Pos::new(0, 0));
        h.auto_equip(Item::new(1, "dagger"));
        assert_eq!(h.weapon.as_ref().unwrap().kind, "dagger");
        h.auto_equip(Item::new(2, "sword"));
        assert_eq!(h.weapon.as_ref().unwrap().kind, "sword");
        assert_eq!(h.inv.len(), 1, "old dagger goes to the pack");
        h.auto_equip(Item::new(3, "dagger"));
        assert_eq!(h.weapon.as_ref().unwrap().kind, "sword");
        h.auto_equip(Item::new(4, "plate"));
        assert_eq!(h.def(), 5);
        assert_eq!(h.speed(), 8);
        h.speed_t = 5;
        assert_eq!(h.speed(), 13);
    }
}
