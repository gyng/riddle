//! Runtime monster state.
use crate::defs::{depth_atk_bonus, depth_hp_bonus, monster_def, MonsterDef};
use crate::geom::Pos;
use crate::item::Item;
use serde::{Deserialize, Serialize};

/// What a telegraphing monster does next turn.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Pending {
    Shoot,
    HeavyHit,
    Rally,
    Swell,
    Chant,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Monster {
    pub id: u32,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub pos: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub atk: (i32, i32),
    pub def: i32,
    pub speed: i32,
    pub energy: i32,
    pub ally: bool,
    /// Captive not yet freed.
    pub neutral: bool,
    pub awake: bool,
    pub last_seen: Option<Pos>,
    pub telegraph: Option<String>,
    pub pending: Option<Pending>,
    pub stun: i32,
    pub paralysed: i32,
    pub confused: i32,
    pub fear: i32,
    pub blind: i32,
    pub poison: (i32, i32),
    pub buff_def: (i32, i32),
    pub cooldown: i32,
    pub ttl: Option<i32>,
    pub stolen: Option<Item>,
    pub fleeing: bool,
    pub grudge: bool,
    pub summoned: bool,
}

impl Monster {
    pub fn spawn(id: u32, kind: &str, pos: Pos, depth: u32) -> Monster {
        let d: &MonsterDef = monster_def(kind);
        let bonus_hp = if d.boss || d.tags.contains(&"summoned") || d.hp == 0 { 0 } else { depth_hp_bonus(depth) };
        let bonus_atk = if d.atk.1 == 0 { 0 } else { depth_atk_bonus(depth) };
        let hp = d.hp + bonus_hp;
        Monster {
            id,
            kind: kind.into(),
            name: None,
            pos,
            hp,
            max_hp: hp,
            atk: (d.atk.0, d.atk.1 + bonus_atk),
            def: d.def,
            speed: d.speed,
            energy: 0,
            ally: false,
            neutral: kind == "captive",
            awake: false,
            last_seen: None,
            telegraph: None,
            pending: None,
            stun: 0,
            paralysed: 0,
            confused: 0,
            fear: 0,
            blind: 0,
            poison: (0, 0),
            buff_def: (0, 0),
            cooldown: 0,
            ttl: None,
            stolen: None,
            fleeing: false,
            grudge: false,
            summoned: d.tags.contains(&"summoned"),
        }
    }
    pub fn def(&self) -> &'static MonsterDef {
        monster_def(&self.kind)
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.def().tags.contains(&tag)
    }
    pub fn is_boss(&self) -> bool {
        self.def().boss
    }
    pub fn title(&self) -> String {
        match &self.name {
            Some(n) => format!("{} the {}", n, self.def().title),
            None => self.def().title.to_string(),
        }
    }
    /// Apply the grudge bonus: +10% stats and a name.
    pub fn make_grudge(&mut self, name: &str) {
        self.name = Some(name.into());
        self.grudge = true;
        self.max_hp = (self.max_hp * 11).div_ceil(10);
        self.hp = self.max_hp;
        self.atk = (self.atk.0, (self.atk.1 * 11).div_ceil(10));
    }
    pub fn effective_def(&self) -> i32 {
        self.def + if self.buff_def.1 > 0 { self.buff_def.0 } else { 0 }
    }
    pub fn hostile(&self) -> bool {
        !self.ally && !self.neutral
    }
    pub fn tick_statuses(&mut self) {
        if self.stun > 0 {
            self.stun -= 1;
        }
        if self.paralysed > 0 {
            self.paralysed -= 1;
        }
        if self.confused > 0 {
            self.confused -= 1;
        }
        if self.fear > 0 {
            self.fear -= 1;
        }
        if self.blind > 0 {
            self.blind -= 1;
        }
        if self.buff_def.1 > 0 {
            self.buff_def.1 -= 1;
        }
        if self.cooldown > 0 {
            self.cooldown -= 1;
        }
        if let Some(t) = self.ttl.as_mut() {
            *t -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grudge_bonus() {
        let mut m = Monster::spawn(1, "jackal", Pos::new(1, 1), 1);
        let hp = m.max_hp;
        m.make_grudge("Grak");
        assert!(m.max_hp > hp);
        assert_eq!(m.title(), "Grak the jackal");
        assert!(m.grudge);
    }
    #[test]
    fn depth_growth_is_small() {
        let a = Monster::spawn(1, "goblin", Pos::new(0, 0), 2);
        let b = Monster::spawn(1, "goblin", Pos::new(0, 0), 10);
        assert!(b.max_hp - a.max_hp <= 5);
        assert!(b.atk.1 - a.atk.1 <= 2);
    }
}
