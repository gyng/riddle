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

impl Pending {
    /// Cut 23 §3: what a telegraph announces, ≤ 3 words — the reason on tap of its shout
    /// (`bloat swells` → `gas burst next`).
    pub fn why(self) -> &'static str {
        match self {
            Pending::Shoot => "arrow next",
            Pending::HeavyHit => "heavy blow next",
            Pending::Rally => "calls more goblins",
            Pending::Swell => "gas burst next",
            Pending::Chant => "raises the dead",
            Pending::Gaze => "stun gaze next",
            Pending::Flip => "reflects melee now",
            Pending::Call => "calls for help",
            Pending::Mirror => "mirrors the rhythm",
            Pending::Hammer => "hammer blow next",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Monster {
    pub id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<crate::endgame::Modifiers>,
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
    /// Cut 19 §5: this grudge was avenged by an earlier heir (its kill reads `slain`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub avenged: bool,
    pub summoned: bool,
    /// Cut 24 §1: a Warlord's shield-wall reserve (stepped in to take an unaimed blow): its
    /// blood is the boss shrugging the blow, not the fight moving (`Run.boss_still`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub reserve: bool,
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
    /// Cut 16: rallies a boss has made — the Warlord's reserves are finite (`WARLORD_RESERVES`):
    /// an endless wall with heal potions cycling was a 7-minute loop (rater X on 238bd67).
    #[serde(default)]
    pub rallies: u32,
    /// Cut 16 §4: the Warlord has broken (at half hp, once): no rallies, no wall, faster and
    /// harder-hitting (`ai::warlord_break`).
    #[serde(default)]
    pub broken: bool,
    /// Cut 4: the companion verb last announced (`<kind>: flank`, once per streak).
    #[serde(default)]
    pub last_verb: String,
    /// Cut 5 §4: a nest's sleeper (never wakes by sight; the hero within two tiles wakes the
    /// den) and a lost heir's stray companion (tameable at 60 % whatever its wounds).
    #[serde(default)]
    pub nest: bool,
    #[serde(default)]
    pub stray: bool,
    #[serde(default)]
    pub dormant: bool,
    /// Cut 5: an ally's last verbs on the Mirror King (he mirrors the pack as he mirrors the
    /// hero: the third of a kind comes back).
    #[serde(default)]
    pub verb_ring: Vec<String>,
    /// Cut 7 §3: the band situation this monster belongs to — `den` (a thief that snatches
    /// only from a flanked hero), `lock` (a bloat that swells and bursts on sight), `captive`
    /// (chained across the stairs: no place-swapping, a foe once adjacent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub situation: Option<String>,
}

impl Monster {
    pub fn spawn(id: u32, kind: &str, pos: Pos, depth: u32) -> Monster {
        let d: &MonsterDef = monster_def(kind);
        let bonus_hp = if d.boss || d.tags.contains(&"summoned") || d.hp == 0 { 0 } else { depth_hp_bonus(depth) };
        let bonus_atk = if d.atk.1 == 0 { 0 } else { depth_atk_bonus(depth) };
        let hp = d.hp + bonus_hp;
        Monster {
            id,
            modifiers: None,
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
            avenged: false,
            summoned: d.tags.contains(&"summoned"),
            reserve: false,
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
            rallies: 0,
            broken: false,
            last_verb: String::new(),
            nest: false,
            stray: false,
            dormant: false,
            verb_ring: Vec::new(),
            situation: None,
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
        if self.modifiers.is_some() {
            self.max_hp = self.max_hp.min(crate::endgame::STAT_CAP);
            self.hp = self.max_hp;
            self.atk.1 = self.atk.1.min(crate::endgame::STAT_CAP);
        }
    }
    /// Speed after the caster's `slow` (never below 1).
    pub fn frenzied(&self) -> bool {
        self.modifiers.is_some_and(|m| m.elite == Some(crate::endgame::Elite::Frenzied))
            && self.hostile() && self.hp > 0 && i64::from(self.hp) * 2 <= i64::from(self.max_hp)
    }
    pub fn effective_atk(&self) -> (i32, i32) {
        (self.atk.0, self.atk.1 + if self.frenzied() { 2 } else { 0 })
    }
    pub fn effective_speed(&self) -> i32 {
        let speed = self.speed + if self.frenzied() { 3 } else { 0 };
        if self.slow_t > 0 {
            (speed - 5).max(1)
        } else {
            speed
        }
    }
    pub fn effective_def(&self) -> i32 {
        self.def + if self.buff_def.1 > 0 { self.buff_def.0 } else { 0 }
            + if self.modifiers.is_some_and(|m| m.elite == Some(crate::endgame::Elite::Shielded))
                && self.hostile() && self.stun == 0 && self.paralysed == 0 { 2 } else { 0 }
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
        self.tick_statuses_by(1);
    }
    pub(crate) fn tick_statuses_by(&mut self, ticks: i32) {
        if self.stun > 0 {
            self.stun = (self.stun - ticks).max(0);
        }
        if self.paralysed > 0 {
            self.paralysed = (self.paralysed - ticks).max(0);
        }
        if self.confused > 0 {
            self.confused = (self.confused - ticks).max(0);
        }
        if self.fear > 0 {
            self.fear = (self.fear - ticks).max(0);
        }
        if self.blind > 0 {
            self.blind = (self.blind - ticks).max(0);
        }
        if self.buff_def.1 > 0 {
            self.buff_def.1 = (self.buff_def.1 - ticks).max(0);
        }
        if self.cooldown > 0 {
            self.cooldown = (self.cooldown - ticks).max(0);
        }
        if self.marked > 0 {
            self.marked = (self.marked - ticks).max(0);
        }
        if self.slow_t > 0 {
            self.slow_t = (self.slow_t - ticks).max(0);
        }
        if let Some(t) = self.ttl.as_mut() {
            *t -= ticks;
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
