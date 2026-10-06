//! Cut 29 §3: the diagnostics meters — per fight, per run, per night: damage dealt and taken per
//! second by the hero, his pets and the foes; healing per second by source; the time split
//! (fighting · travel · chores · rest); each row's fires and share of rule activations; the
//! supplies used; gold per minute; the hits taken by the hero and by his pets. Pure reads of the
//! event stream (`fold`, one tick's events at a time): nothing here feeds the sim, so a run's
//! outcome is the same with the meters or without (the replay hash holds). The one event the
//! meters add, `Ev::Heal`, is read off the hero's and the pets' hp around each tick
//! (`Game::tick`), after the tick — it is not an input to anything.
use crate::engine::HERO_ID;
use crate::offline::TICKS_PER_SECOND;
use crate::wire::Ev;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Ticks without a blow between the hero's side and a foe that end a fight (3 s).
pub const FIGHT_GAP: u32 = 30;

/// Damage by side.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Sides {
    pub hero: i64,
    pub pets: i64,
    pub foes: i64,
}

impl Sides {
    pub fn add(&mut self, o: &Sides) {
        self.hero += o.hero;
        self.pets += o.pets;
        self.foes += o.foes;
    }
    pub fn total(&self) -> i64 {
        self.hero + self.pets + self.foes
    }
}

/// Ticks by activity.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Split {
    pub fight: u32,
    pub travel: u32,
    pub chores: u32,
    pub rest: u32,
}

/// One meter's totals (a fight, a run, a night).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Meter {
    pub ticks: u32,
    /// Damage dealt by side (`Ev::Attack` hits, and a foe's harm by hazard or burst to the hero's
    /// side counts as the foes'), and taken by side (`Ev::Hurt`).
    pub dealt: Sides,
    pub taken: Sides,
    /// Healing by source (`potion`, `rest`, `regen`, `skill`, `pet`).
    pub healed: BTreeMap<String, i64>,
    pub time: Split,
    /// Fires by row (−1: a trait's or a card's own step). One action may emit multiple rules.
    pub rows: BTreeMap<i32, u32>,
    /// Ticks containing a rule activation or pickup; retained separately from rule fires.
    pub actions: u32,
    /// Supplies used by kind (`Ev::Use`).
    pub supplies: BTreeMap<String, u32>,
    /// Gold brought home (`Ev::Exit.loot_kept`).
    pub gold: i64,
    /// Blows landed on the hero and on his pets (`Ev::Hurt` with damage).
    pub hits_hero: u32,
    pub hits_pets: u32,
    /// Fights begun (a run's, a night's).
    pub fights: u32,
}

impl Meter {
    pub fn is_empty(&self) -> bool {
        self.ticks == 0
    }
    pub fn add(&mut self, o: &Meter) {
        self.ticks += o.ticks;
        self.dealt.add(&o.dealt);
        self.taken.add(&o.taken);
        for (k, v) in &o.healed {
            *self.healed.entry(k.clone()).or_insert(0) += v;
        }
        self.time.fight += o.time.fight;
        self.time.travel += o.time.travel;
        self.time.chores += o.time.chores;
        self.time.rest += o.time.rest;
        for (k, v) in &o.rows {
            *self.rows.entry(*k).or_insert(0) += v;
        }
        self.actions += o.actions;
        for (k, v) in &o.supplies {
            *self.supplies.entry(k.clone()).or_insert(0) += v;
        }
        self.gold += o.gold;
        self.hits_hero += o.hits_hero;
        self.hits_pets += o.hits_pets;
        self.fights += o.fights;
    }
}

/// A run's meters as the engine keeps them (`Run.meters`): the run so far, the fight in progress
/// (or the last one), and the pets' ids (a pet that died is still a pet).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunMeters {
    pub run: Meter,
    pub fight: Meter,
    /// Ticks since the fight's last blow (≥ `FIGHT_GAP`: no fight in progress).
    pub quiet: u32,
    pub pets: BTreeSet<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Side {
    Hero,
    Pet,
    Foe,
}

fn side(pets: &BTreeSet<u32>, id: u32) -> Side {
    if id == HERO_ID {
        Side::Hero
    } else if pets.contains(&id) {
        Side::Pet
    } else {
        Side::Foe
    }
}

fn put(s: &mut Sides, side: Side, v: i64) {
    match side {
        Side::Hero => s.hero += v,
        Side::Pet => s.pets += v,
        Side::Foe => s.foes += v,
    }
}

/// One tick's events into a meter: (a blow passed between the hero's side and a foe, the hero did
/// a chore — picked something up).
fn fold_into(m: &mut Meter, pets: &BTreeSet<u32>, evs: &[Ev]) -> (bool, bool) {
    m.ticks += 1;
    let mut blow = false;
    let mut acted = false;
    let mut chore = false;
    for e in evs {
        match e {
            Ev::Attack { src, dst, dmg, hit, .. } => {
                let (a, b) = (side(pets, *src), side(pets, *dst));
                if *hit && *dmg > 0 {
                    put(&mut m.dealt, a, *dmg as i64);
                }
                if (a == Side::Foe) != (b == Side::Foe) {
                    blow = true;
                }
            }
            Ev::Hurt { id, dmg, cause, .. } => {
                let s = side(pets, *id);
                if *dmg > 0 {
                    put(&mut m.taken, s, *dmg as i64);
                    match s {
                        Side::Hero => m.hits_hero += 1,
                        Side::Pet => m.hits_pets += 1,
                        Side::Foe => {}
                    }
                    if s != Side::Foe && !matches!(cause.as_str(), "hunger" | "poison" | "starving") {
                        blow = true;
                    }
                }
            }
            Ev::Heal { src, amount, .. } => {
                *m.healed.entry(src.clone()).or_insert(0) += *amount as i64;
            }
            Ev::Rule { row, .. } => {
                *m.rows.entry(*row).or_insert(0) += 1;
                acted = true;
            }
            Ev::Use { item, .. } => {
                *m.supplies.entry(item.clone()).or_insert(0) += 1;
            }
            Ev::Pickup { id, .. } if *id == HERO_ID => chore = true,
            Ev::Exit { loot_kept, .. } => m.gold += *loot_kept as i64,
            _ => {}
        }
    }
    if acted || chore {
        m.actions += 1;
    }
    (blow, chore)
}

impl RunMeters {
    /// Fold one tick's events (`Game::tick`, after the tick): the run's meter always, the fight's
    /// while a blow passed between the hero's side and a foe within `FIGHT_GAP` ticks. `allies`:
    /// the pets alive now (added to `pets`).
    pub fn tick(&mut self, evs: &[Ev], allies: impl Iterator<Item = u32>) {
        self.pets.extend(allies);
        let (blow, chore) = fold_into(&mut self.run, &self.pets, evs);
        let rest = evs.iter().any(|e| matches!(e, Ev::Rule { verb, .. } if verb.v == "rest") || matches!(e, Ev::Hurt { id, cause, .. } if *id == HERO_ID && cause == "rest"));
        let chores = !rest && chore;
        if blow {
            if self.quiet >= FIGHT_GAP || self.run.fights == 0 {
                self.fight = Meter::default();
                self.run.fights += 1;
                self.fight.fights = 1;
            }
            self.quiet = 0;
        } else {
            self.quiet = self.quiet.saturating_add(1);
        }
        let fighting = self.quiet < FIGHT_GAP && self.run.fights > 0;
        if fighting {
            fold_into(&mut self.fight, &self.pets, evs);
            self.fight.time.fight += 1;
            self.run.time.fight += 1;
        } else if rest {
            self.run.time.rest += 1;
        } else if chores {
            self.run.time.chores += 1;
        } else {
            self.run.time.travel += 1;
        }
    }
}

impl RunMeters {
    /// Consecutive ticks without events; exactly the same accounting as `tick(&[], ..)`.
    pub(crate) fn quiet_ticks(&mut self, ticks: u32) {
        let fighting = if self.run.fights > 0 { FIGHT_GAP.saturating_sub(self.quiet.saturating_add(1)).min(ticks) } else { 0 };
        self.quiet = self.quiet.saturating_add(ticks);
        self.run.ticks += ticks;
        self.run.time.fight += fighting;
        self.run.time.travel += ticks - fighting;
        self.fight.ticks += fighting;
        self.fight.time.fight += fighting;
    }
    pub fn is_empty(&self) -> bool {
        self.run.ticks == 0
    }
}

/// Cut 29 §3: the healing of one tick as `Ev::Heal` events — each of the hero's side whose hp rose
/// past the tick's harm to it (`hp0`: the hp before the tick), by the tick's own evidence: a heal
/// or regen drunk (`potion`), a rest (`rest`), a second wind or a drain (`skill`), else `regen`.
pub fn heals(run: &crate::engine::Run, cx: &mut crate::engine::Ctx, from: usize, hp0: impl Iterator<Item = (u32, i32)>) {
    let evs = &cx.events[from..];
    let hurt = |id: u32| evs.iter().filter_map(|e| if let Ev::Hurt { id: i, dmg, .. } = e { (*i == id).then_some((*dmg).max(0)) } else { None }).sum::<i32>();
    let src_hero = if evs.iter().any(|e| matches!(e, Ev::Use { item, .. } if item == "heal" || item == "regen")) {
        "potion"
    } else if evs.iter().any(|e| matches!(e, Ev::Rule { verb, .. } if verb.v == "rest") || matches!(e, Ev::Hurt { id, cause, .. } if *id == HERO_ID && cause == "rest")) {
        "rest"
    } else if evs.iter().any(|e| matches!(e, Ev::Callout { text, .. } if text == "second wind" || text == "drain")) {
        "skill"
    } else {
        "regen"
    };
    let mut out = Vec::new();
    for (id, before) in hp0 {
        let now = if id == HERO_ID { Some(run.hero.hp) } else { run.monsters.iter().find(|m| m.id == id).map(|m| m.hp) };
        let Some(now) = now.filter(|n| *n > 0) else { continue };
        let gained = now - before + hurt(id);
        if gained > 0 {
            out.push(Ev::Heal { t: run.turn, id, amount: gained, src: if id == HERO_ID { src_hero.into() } else { "pet".into() } });
        }
    }
    cx.events.extend(out);
}

// ---- on the wire ----

/// Per-second rates by side.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Rates {
    pub hero: f64,
    pub pets: f64,
    pub foes: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct HealRate {
    pub src: String,
    pub total: i64,
    pub per_s: f64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct RowShare {
    /// The row (0-based; −1 a trait's or a card's own step).
    pub row: i32,
    pub fires: u32,
    /// Of all recorded rule activations (0..1). Several may belong to one action.
    pub share: f64,
}

/// Cut 29 §3: a meter on the wire — totals, and the rates over its game seconds (`seconds`: ticks
/// ÷ 10). `dealt`/`taken` per second by side; `time` in seconds.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct MeterWire {
    pub seconds: f64,
    pub dealt: Sides,
    pub taken: Sides,
    pub dps_dealt: Rates,
    pub dps_taken: Rates,
    pub healed: Vec<HealRate>,
    pub hps: f64,
    pub time: Split,
    pub time_s: SplitSeconds,
    pub rows: Vec<RowShare>,
    pub actions: u32,
    pub supplies: BTreeMap<String, u32>,
    pub gold: i64,
    pub gold_per_min: f64,
    pub hits_hero: u32,
    pub hits_pets: u32,
    pub fights: u32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SplitSeconds {
    pub fight: f64,
    pub travel: f64,
    pub chores: f64,
    pub rest: f64,
}

fn secs(t: u32) -> f64 {
    t as f64 / TICKS_PER_SECOND as f64
}

/// A rate to three decimals (a wire line round-trips its JSON exactly).
fn r3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn wire(m: &Meter) -> MeterWire {
    let s = secs(m.ticks).max(0.1);
    let rate = |x: &Sides| Rates { hero: r3(x.hero as f64 / s), pets: r3(x.pets as f64 / s), foes: r3(x.foes as f64 / s) };
    let healed: Vec<HealRate> = m.healed.iter().map(|(k, v)| HealRate { src: k.clone(), total: *v, per_s: r3(*v as f64 / s) }).collect();
    let fires = m.rows.values().map(|n|u64::from(*n)).sum::<u64>().max(1) as f64;
    MeterWire {
        seconds: secs(m.ticks),
        dealt: m.dealt.clone(),
        taken: m.taken.clone(),
        dps_dealt: rate(&m.dealt),
        dps_taken: rate(&m.taken),
        hps: r3(healed.iter().map(|h| h.total).sum::<i64>() as f64 / s),
        healed,
        time: m.time.clone(),
        time_s: SplitSeconds { fight: secs(m.time.fight), travel: secs(m.time.travel), chores: secs(m.time.chores), rest: secs(m.time.rest) },
        rows: m.rows.iter().map(|(r, n)| RowShare { row: *r, fires: *n, share: r3(*n as f64 / fires) }).collect(),
        actions: m.actions,
        supplies: m.supplies.clone(),
        gold: m.gold,
        gold_per_min: r3(m.gold as f64 / (s / 60.0)),
        hits_hero: m.hits_hero,
        hits_pets: m.hits_pets,
        fights: m.fights,
    }
}

/// The sums an event stream holds, for the invariant `qa.rs` checks (meter totals equal them):
/// (damage dealt by all hits, damage taken by all, healing, rule fires, supplies used, gold).
pub fn stream_sums(evs: &[Ev]) -> (i64, i64, i64, u32, u32, i64) {
    let mut out = (0, 0, 0, 0, 0, 0);
    for e in evs {
        match e {
            Ev::Attack { dmg, hit: true, .. } if *dmg > 0 => out.0 += *dmg as i64,
            Ev::Hurt { dmg, .. } if *dmg > 0 => out.1 += *dmg as i64,
            Ev::Heal { amount, .. } => out.2 += *amount as i64,
            Ev::Rule { .. } => out.3 += 1,
            Ev::Use { .. } => out.4 += 1,
            Ev::Exit { loot_kept, .. } => out.5 += *loot_kept as i64,
            _ => {}
        }
    }
    out
}

// (a wire line derives `Eq`; the rates are finite by construction — `seconds` ≥ 0.1)
impl Eq for MeterWire {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rule_shares_use_activations_when_one_action_emits_multiple_rules() {
        let mut m=RunMeters::default();
        let rule=|row|Ev::Rule{t:10,row,verb:crate::rules::Verb::new("explore"),text:"chore".into()};
        m.tick(&[rule(-2),rule(-2),rule(0)],std::iter::empty());
        assert_eq!(m.run.actions,1);assert_eq!(m.run.rows.get(&-2),Some(&2));
        let w=wire(&m.run);
        assert_eq!(w.rows[0].share,0.667);assert_eq!(w.rows[1].share,0.333);
        // Historical saved totals with more fires than actions derive the same
        // proportions; no lost counts, save mutation or presentation clamp.
        let mut restored:Meter=serde_json::from_str(&serde_json::to_string(&m.run).unwrap()).unwrap();
        restored.actions=0;assert_eq!(wire(&restored).rows,w.rows);
        assert!(wire(&Meter::default()).rows.is_empty());
    }
}
