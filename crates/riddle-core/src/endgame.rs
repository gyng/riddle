//! Numbered descent progression. Historical challenge restarts are a separate axis.
//! Encounter modifiers are snapshotted on each foe; no gameplay RNG or extra mobs.
use serde::{Deserialize, Serialize};
use crate::Game;

pub const ARMOURED: u8 = 1;
pub const SWIFT: u8 = 2;
pub const REGENERATING: u8 = 4;
pub const STAT_CAP: i32 = 1_000_000;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Elite { Shielded, Frenzied }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Modifiers {
    pub tier: u32,
    pub affixes: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elite: Option<Elite>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tight_mirror: bool,
}
impl Modifiers {
    pub fn has(self, affix: u8) -> bool { self.affixes & affix != 0 }
}

/// Shared tooltip vocabulary, emitted once per snapshot rather than per foe.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModifierInfo {
    pub id: String,
    pub mask: u8,
    pub name: String,
    pub effect: String,
    pub counter: String,
}
pub fn catalogue(tier: u32) -> Vec<ModifierInfo> {
    if tier == 0 { return Vec::new(); }
    [
        ("armoured", ARMOURED, "Armoured", "+1 armour", "Poison or fire"),
        ("swift", SWIFT, "Swift", "+2 speed", "Slow"),
        ("regenerating", REGENERATING, "Regenerating", "+1 HP each second while awake and unpoisoned", "Poison or sustained damage"),
        ("shielded", 0, "Shielded", "+2 armour unless stunned or paralysed", "Stun or paralyse"),
        ("frenzied", 0, "Frenzied", "+2 damage and +3 speed at half HP or lower", "Slow or burst damage"),
        ("tight_mirror", 0, "Quick mirror", "Second repeated attack reflects", "Alternate attacks"),
    ].into_iter().map(|(id,mask,name,effect,counter)| ModifierInfo {id:id.into(),mask,name:name.into(),effect:effect.into(),counter:counter.into()}).collect()
}

/// Fixed for a tier, not rerolled on a send; at most two mechanical modifiers.
pub fn affixes(tier: u32) -> u8 {
    if tier == 0 { return 0; }
    let first = 1 << ((tier - 1) % 3);
    first | if tier >= 3 { 1 << (tier % 3) } else { 0 }
}
fn scaled(value: i32, tier: u32, percent: u64) -> i32 {
    let multiplier = 100 + u64::from(tier) * percent;
    ((value.max(0) as u64).saturating_mul(multiplier).saturating_add(99) / 100)
        .min(STAT_CAP as u64) as i32
}
fn apply(m: &mut crate::monster::Monster, mods: Modifiers) {
    m.max_hp = scaled(m.max_hp, mods.tier, 8);
    m.hp = m.max_hp;
    m.atk = (scaled(m.atk.0, mods.tier, 4), scaled(m.atk.1, mods.tier, 4));
    if mods.has(ARMOURED) { m.def = m.def.saturating_add(1); }
    if mods.has(SWIFT) { m.speed = m.speed.saturating_add(2); }
    m.modifiers = Some(mods);
}
/// All calls are at birth: script overrides are applied afterwards. Companion,
/// captive, nest and stray births deliberately use ordinary Monster::spawn.
pub fn spawn(run: &crate::engine::Run, id: u32, kind: &str, pos: crate::geom::Pos, depth: u32, elites: bool) -> crate::monster::Monster {
    let mut m = crate::monster::Monster::spawn(id, kind, pos, depth);
    if run.difficulty == 0 || !m.hostile() || m.max_hp <= 0 { return m; }
    let hash = crate::rng::splitmix(run.seed ^ (u64::from(depth) << 32) ^ u64::from(id) ^ 0x6173_6365_6e64_6564);
    let elite = if elites && !m.is_boss() && !m.summoned && hash.is_multiple_of(8) {
        Some(if hash & 256 == 0 { Elite::Shielded } else { Elite::Frenzied })
    } else { None };
    apply(&mut m, Modifiers { tier:run.difficulty, affixes:affixes(run.difficulty), elite, tight_mirror:kind == "mirror_king" });
    m
}
/// A split inherits its parent's birth modifiers; never multiply modified stats
/// again, and never reroll an elite on its newly assigned id.
pub fn spawn_split(id: u32, kind: &str, pos: crate::geom::Pos, depth: u32, inherited: Option<Modifiers>) -> crate::monster::Monster {
    let mut m = crate::monster::Monster::spawn(id, kind, pos, depth);
    if let Some(mods) = inherited { apply(&mut m, mods); }
    m
}
/// Taming retains wounds but removes dungeon difficulty from a persistent pet.
pub fn normalise_tamed(m: &mut crate::monster::Monster, depth: u32) {
    if m.modifiers.take().is_none() { return; }
    let base = crate::monster::Monster::spawn(m.id, &m.kind, m.pos, depth);
    let hp = (i64::from(m.hp.max(0)) * i64::from(base.max_hp) + i64::from(m.max_hp.max(1)) - 1) / i64::from(m.max_hp.max(1));
    m.hp = (hp as i32).clamp(1, base.max_hp.max(1));
    m.max_hp = base.max_hp;
    m.atk = base.atk;
    m.def = base.def;
    m.speed = base.speed;
}
/// Existing cadence and the monster use the same saved rhythm rule.
pub fn repeated(ring: &[String], verb: &str, tight: bool) -> bool {
    let count = if tight { 1 } else { 2 };
    ring.len() >= count && ring[ring.len()-count..].iter().all(|v| v == verb)
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub tier: u32,
    pub unlocked: u32,
    pub cleared: Option<u32>,
}
impl Progress {
    pub fn validate(&self) -> Result<(), String> {
        if self.tier > self.unlocked || self.cleared.is_some_and(|n| n >= self.unlocked)
            || (self.cleared.is_none() && self.unlocked != 0) {
            return Err("invalid difficulty progress".into());
        }
        Ok(())
    }
    /// Idempotent, constant-size and atomic even at the integer boundary.
    pub fn complete(&mut self, tier: u32) -> Result<(), String> {
        if tier > self.unlocked { return Err("difficulty locked".into()); }
        let next = tier.checked_add(1).ok_or("difficulty limit reached")?;
        self.unlocked = self.unlocked.max(next);
        self.cleared = Some(self.cleared.map_or(tier, |n| n.max(tier)));
        Ok(())
    }
}
impl Game {
    /// Eligibility without mutating old saves or inventing harder clears from
    /// their old ascension count. Not exposed by the client bridge yet.
    pub fn descent_progress(&self) -> Progress {
        let mut p = self.lineage.endgame.clone().unwrap_or_default();
        if self.lineage.endgame.is_none() && self.lineage.ended {
            // Old endings prove the unscaled dungeon only.
            let _ = p.complete(0);
        }
        p
    }

    /// Begin a separately numbered descent after an ending. Keeps the survivor,
    /// the town wallet, equipment and all other bloodlines. Encounter modifiers
    /// must be complete before exposing this API in the app.
    pub fn begin_descent(&mut self, tier: u32) -> Result<(), String> {
        if !self.lineage.ended { return Err("clear the dungeon first".into()); }
        if self.pending_exit.is_some() || self.offline_absence.is_some() || self.offline {
            return Err("finish the return first".into());
        }
        if self.run.as_ref().is_some_and(|r| r.over.is_none()) {
            return Err("hero still away".into());
        }
        let mut p = self.descent_progress();
        p.validate()?;
        if tier > p.unlocked { return Err("difficulty locked".into()); }
        p.tier = tier;
        // All validation precedes the first mutation.
        self.lineage.endgame = Some(p);
        self.lineage.ended = false;
        self.lineage.best_depth = 0;
        self.lineage.heir_best = 0;
        self.lineage.start = 1;
        self.lineage.banked_depths.clear();
        self.lineage.waystones.clear();
        self.lineage.lane_stones.clear();
        self.lineage.picked.clear();
        self.lineage.night_passes.clear();
        self.lineage.night_short = None;
        self.lineage.bounty = None;
        self.lineage.wall_day = None;
        self.lineage.wall_offer = None;
        self.lineage.gold_carry = 0;
        self.run = None;
        self.pending_exit = None;
        self.history.clear();
        self.floor_start = None;
        self.sent_state = None;
        self.capsules.0.clear();
        if let Some(tap) = &mut self.tap { tap.clear(); }
        self.deaths.clear();
        self.reel.clear();
        self.events.clear();
        self.prov.clear();
        self.last_snapshot = None;
        self.last_exit = None;
        self.batch = Default::default();
        self.stall = Default::default();
        self.stall_runs = 0;
        self.stall_cache = None;
        self.row_tally.clear();
        self.forecast_cache.borrow_mut().clear();
        self.panel_cache.borrow_mut().clear();
        self.refined_panels.borrow_mut().clear();
        self.bounty_seen = None;
        self.passage = None;
        self.fold_plan = None;
        // Generated thresholds follow the new record immediately, not only after load.
        crate::packages::recompile(&mut self.lineage);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bloodlines::Session;

    #[test]
    fn five_clears_unlock_once_and_replays_never_increase_difficulty() {
        let mut p = Progress::default();
        for tier in 0..5 {
            p.tier = tier;
            p.complete(tier).unwrap();
            assert_eq!(p.unlocked, tier + 1);
            let once = p.clone();
            p.complete(tier).unwrap();
            assert_eq!(p, once);
        }
        p.complete(0).unwrap();
        assert_eq!((p.unlocked, p.cleared), (5, Some(4)));
        let before = p.clone();
        assert!(p.complete(6).is_err());
        assert_eq!(p, before);
        let mut limit = Progress { tier:u32::MAX, unlocked:u32::MAX, cleared:Some(u32::MAX-1) };
        let before = limit.clone();
        assert!(limit.complete(u32::MAX).is_err());
        assert_eq!(limit, before);
    }

    #[test]
    fn old_challenge_counts_are_not_harder_clears_and_refusals_are_exact() {
        let mut g = Game::new_resident(3);
        g.lineage.ascension = 37;
        assert_eq!(g.descent_progress(), Progress::default());
        let before = g.save();
        assert!(g.begin_descent(0).is_err());
        assert_eq!(g.save(), before);
        g.lineage.ended = true;
        g.lineage.best_depth = 34;
        crate::packages::recompile(&mut g.lineage);
        assert_eq!(g.descent_progress(), Progress { tier:0, unlocked:1, cleared:Some(0) });
        let before = g.save();
        assert!(g.begin_descent(2).is_err());
        assert_eq!(g.save(), before);
        g.begin_descent(1).unwrap();
        assert_eq!(g.lineage.ascension, 37);
        assert_eq!(g.lineage.endgame.as_ref().unwrap().tier, 1);
        assert!(!g.lineage.ended);
        assert_eq!(g.lineage.rules(), &crate::packages::compile(&g.lineage));
        let save=g.save();
        assert!(Game::load(&save).unwrap().save()==save, "compiled thresholds round-trip immediately");
    }

    #[test]
    fn numbered_descent_preserves_shared_town_and_other_live_bloodlines() {
        for selected in 1..=3 {
            let mut s = Session::new(3);
            s.active.lineage.town.home = Some(true);
            s.active.lineage.gold = 2000;
            s.add_bloodline().unwrap();s.add_bloodline().unwrap();
            for id in 1..=3 {
                s.select_bloodline(id).unwrap();
                if id != selected { s.send(); }
            }
            s.select_bloodline(selected).unwrap();
            let l = &mut s.active.lineage;
            l.ended = true;l.best_depth = 34;l.heir_best = 34;l.start = 13;
            l.banked_depths.insert(13);l.waystones.push(13);l.picked.insert(13,2);
            l.kit.insert("sword".into(),2);
            l.supplies.push(crate::item::Item::new(100_001,"heal"));
            l.bloodline.as_mut().unwrap().points=42;
            let before = serde_json::to_value(&s.active.lineage).unwrap();
            let others = s.others.clone();
            let loadout = s.active.loadout.clone();
            s.begin_descent(1).unwrap();
            let after = serde_json::to_value(&s.active.lineage).unwrap();
            for field in ["gold","gold_ledger","town","tree","bloodline_id","bloodline", "hero_legacy", "heir", "class", "classes", "kit", "supplies", "facts", "sets", "pkg", "forge", "unlocks", "clock_s", "rng", "vault", "party", "kennel"] {
                assert_eq!(after.get(field),before.get(field),"slot {selected}: {field}");
            }
            assert_eq!(s.others,others);
            assert_eq!(s.active.loadout,loadout);
            assert_eq!((s.active.lineage.best_depth,s.active.lineage.start),(0,1));
            assert!(s.active.lineage.picked.is_empty());
            let save = s.save();let loaded = Session::load(&save).unwrap();
            assert_eq!(loaded.active.lineage.endgame,s.active.lineage.endgame);
            for (id,old) in others {
                s.select_bloodline(id).unwrap();
                assert_eq!(s.active.lineage.gold,old.lineage.gold,"wallet unchanged after switching to {id}");
                assert!(s.active.run == old.run,"live run unchanged after switching to {id}");
                assert!(s.active.run.is_some());
            }
        }
    }

    #[test]
    fn run_difficulty_is_saved_and_forecast_keys_follow_selected_tier() {
        let mut g = Game::new_resident(3);
        g.lineage.endgame=Some(Progress { tier:2, unlocked:2, cleared:Some(1) });
        let key = crate::forecast::lineage_key(&g);
        g.send();
        assert_eq!(g.run.as_ref().unwrap().difficulty,2);
        g.step(20);
        let loaded = Game::load(&g.save()).unwrap();
        assert!(serde_json::to_string(&loaded.run).unwrap() == serde_json::to_string(&g.run).unwrap(), "all saved run fields round-trip");
        g.lineage.endgame.as_mut().unwrap().tier=1;
        assert_ne!(crate::forecast::lineage_key(&g),key);
        assert_eq!(g.run.as_ref().unwrap().difficulty,2);
    }

    #[test]
    fn numbered_zero_difficulty_matches_existing_gameplay() {
        let mut old = Game::new_resident(7);
        let mut numbered = old.clone();
        numbered.lineage.endgame=Some(Progress::default());
        assert_eq!(crate::forecast::lineage_key(&old),crate::forecast::lineage_key(&numbered));
        assert!(old.send() == numbered.send());
        for _ in 0..5 {
            assert!(old.step(100) == numbered.step(100), "identical events and snapshots");
            assert!(old.run == numbered.run, "identical runtime and RNG");
        }
    }

    #[test]
    fn ending_tick_records_the_played_tier_not_later_camp_state() {
        // Synthetic boundary fixture tests the engine hook, not an earned King victory.
        let mut g = Game::new_resident(3);
        g.lineage.endgame=Some(Progress { tier:1, unlocked:1, cleared:Some(0) });
        g.send();
        g.lineage.endgame.as_mut().unwrap().tier=0;
        g.run.as_mut().unwrap().depth=crate::descent::ENDING_DEPTH;
        g.tick();
        assert!(g.lineage.ended);
        let p = g.lineage.endgame.as_ref().unwrap();
        assert_eq!((p.tier,p.unlocked,p.cleared),(0,2,Some(1)));
        let before = p.clone();
        g.tick();
        assert_eq!(g.lineage.endgame.as_ref().unwrap(),&before);
    }

    #[test]
    fn live_unsettled_absence_and_invalid_state_requests_preserve_exact_saves() {
        let mut live = Game::new_resident(3);live.send();live.lineage.ended=true;
        let mut pending = Game::new_resident(3);pending.send();
        pending.run.as_mut().unwrap().over=Some(crate::engine::ExitTier::Bank);
        pending.finish_run();pending.lineage.ended=true;
        assert!(pending.pending_exit.is_some());
        let mut absence = Game::new_resident(3);
        crate::offline::run_offline_slice(&mut absence,1,false);
        assert!(absence.offline_absence.is_some());absence.lineage.ended=true;
        let mut invalid = Game::new_resident(3);invalid.lineage.ended=true;
        invalid.lineage.endgame=Some(Progress {tier:3, unlocked:1, cleared:Some(0)});
        assert!(Game::load(&invalid.save()).is_err());
        for mut g in [live,pending,absence,invalid] {
            let before=g.save();
            assert!(g.begin_descent(1).is_err());
            assert!(g.save()==before, "refusal preserves every saved field");
        }
    }

    #[test]
    fn old_tier_zero_save_keeps_rng_run_and_hash_without_new_fields() {
        let mut old = Game::new_resident(3);old.send();old.step(30);
        let text = old.save();
        assert!(!text.contains("\"endgame\""));
        assert!(!text.contains("\"difficulty\""));
        let mut loaded = Game::load(&text).unwrap();
        assert!(serde_json::to_string(&loaded.run).unwrap() == serde_json::to_string(&old.run).unwrap(), "all saved tier-zero run fields match");
        assert_eq!(loaded.lineage.rng,old.lineage.rng);
        assert_eq!(crate::forecast::lineage_key(&loaded),crate::forecast::lineage_key(&old));
        assert!(loaded.step(100) == old.step(100), "tier-zero continuation events and snapshots match");
        assert!(serde_json::to_string(&loaded.run).unwrap() == serde_json::to_string(&old.run).unwrap(), "all saved tier-zero run fields match");
    }
}

#[cfg(test)]
#[path = "endgame_tests.rs"]
mod encounter_tests;
