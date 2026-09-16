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
    // Cut 3: the sentinel's stun gaze, the warden's reflect flip, the Lurker Queen's call, the
    // Mirror King's mirror, the Foundry Master's hammer.
    Gaze,
    Flip,
    Call,
    Mirror,
    Hammer,
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
    /// Companion id when this ally is a party member (Addendum A).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<u32>,
    /// Tags beyond the kind's own (bred companions).
    #[serde(default)]
    pub extra_tags: Vec<String>,
    #[serde(default)]
    pub level: u32,
    /// Companion ordered to engage (hero verb `send`).
    #[serde(default)]
    pub sent: bool,
    /// Foes already pickpocketed by a thief companion.
    #[serde(default)]
    pub stole_from: Vec<u32>,
    #[serde(default)]
    pub hurt_since_action: bool,
    /// Actions taken since the hero last acted (for the `fast` fact).
    #[serde(default)]
    pub acts_since_hero: u32,
    /// Cut 2 §4: ranger `mark` (×1.5 damage taken) and caster `slow` (speed −5), in ticks.
    #[serde(default)]
    pub marked: i32,
    #[serde(default)]
    pub slow_t: i32,
    /// Cut 3: the warden's current face — reflecting ranged (true) or melee (false).
    #[serde(default)]
    pub warden_ranged: bool,
    /// Cut 3: a boss has shown its opening telegraph.
    #[serde(default)]
    pub introduced: bool,
    /// Cut 4: the companion verb last announced (`<kind>: flank`, once per streak).
    #[serde(default)]
    pub last_verb: String,
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
            cid: None,
            extra_tags: Vec::new(),
            level: 0,
            sent: false,
            stole_from: Vec::new(),
            hurt_since_action: false,
            acts_since_hero: 0,
            marked: 0,
            slow_t: 0,
            warden_ranged: false,
            introduced: false,
            last_verb: String::new(),
        }
    }
    pub fn def(&self) -> &'static MonsterDef {
        monster_def(&self.kind)
    }
    pub fn has_tag(&self, tag: &str) -> bool {
        self.def().tags.contains(&tag) || self.extra_tags.iter().any(|t| t == tag)
    }
    /// All tags: the kind's plus bred extras.
    pub fn tags(&self) -> Vec<String> {
        let mut v: Vec<String> = self.def().tags.iter().map(|t| t.to_string()).collect();
        for t in &self.extra_tags {
            if !v.contains(t) {
                v.push(t.clone());
            }
        }
        v
    }
    pub fn is_companion(&self) -> bool {
        self.cid.is_some() && self.ally
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
        self.max_hp = (self.max_hp * 11 + 9) / 10;
        self.hp = self.max_hp;
        self.atk = (self.atk.0, (self.atk.1 * 11 + 9) / 10);
    }
    /// Speed after the caster's `slow` (never below 1).
    pub fn effective_speed(&self) -> i32 {
        if self.slow_t > 0 {
            (self.speed - 5).max(1)
        } else {
            self.speed
        }
    }
    pub fn effective_def(&self) -> i32 {
        self.def + if self.buff_def.1 > 0 { self.buff_def.0 } else { 0 }
    }
    pub fn hostile(&self) -> bool {
        !self.ally && !self.neutral
    }
    /// Cut 3: melee hits on this monster land on the attacker instead (the warden only on its
    /// blade face).
    pub fn reflects_melee(&self) -> bool {
        self.has_tag("reflect_melee") && !(self.kind == "warden" && self.warden_ranged)
    }
    /// Arrows and thrown potions come back (the Lich; the warden on its arrow face).
    pub fn reflects_ranged(&self) -> bool {
        self.has_tag("reflect") && (self.kind != "warden" || self.warden_ranged)
    }
    /// Cut 3: hunts by noise, never by sight.
    pub fn is_blind(&self) -> bool {
        self.has_tag("blind")
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
        if self.marked > 0 {
            self.marked -= 1;
        }
        if self.slow_t > 0 {
            self.slow_t -= 1;
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
