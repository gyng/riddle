//! Wire types mirroring `web/src/engine/types.ts` (snake_case JSON).
use crate::item::{FloorItemWire, InvItem};
use crate::rules::{Row, RuleSet, Verb};
use crate::tiles::{Overlay, OverlayKind, Tile};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    pub id: u32,
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ally: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegraph: Option<String>,
    /// Companion id (Addendum A).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeroSnap {
    #[serde(flatten)]
    pub entity: Entity,
    pub inv: Vec<InvItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armour: Option<String>,
    pub class: String,
    #[serde(rename = "trait")]
    pub trait_: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunRef {
    pub id: u32,
    pub heir: u32,
    pub started_turn: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub depth: u32,
    pub biome: String,
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub seen: Vec<bool>,
    pub visible: Vec<bool>,
    pub overlays: Vec<Overlay>,
    pub hero: HeroSnap,
    pub entities: Vec<Entity>,
    pub items: Vec<FloorItemWire>,
    pub alert: i32,
    pub turn: u32,
    pub loot: i32,
    pub run: RunRef,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Ev {
    Move { t: u32, id: u32, x: i32, y: i32 },
    Attack { t: u32, src: u32, dst: u32, dmg: i32, hit: bool, #[serde(default, skip_serializing_if = "Option::is_none")] verb: Option<String> },
    Hurt { t: u32, id: u32, dmg: i32, hp: i32, cause: String },
    Die { t: u32, id: u32, cause: String },
    Rule { t: u32, row: i32, verb: Verb, text: String },
    Telegraph { t: u32, id: u32, what: String },
    Pickup { t: u32, id: u32, item: String },
    Use { t: u32, item: String, outcome: String },
    Fact { t: u32, fact: String },
    Overlay { t: u32, x: i32, y: i32, ov: OverlayKind, ttl: i32 },
    Spawn { t: u32, e: Entity },
    Steal { t: u32, id: u32, item: String },
    Ally { t: u32, id: u32, state: String },
    Descend { t: u32, depth: u32, biome: String },
    Exit { t: u32, tier: String, loot_kept: i32 },
    Note { t: u32, text: String },
    Callout { t: u32, text: String },
    Tame { t: u32, id: u32, kind: String, ok: bool },
    Hatch { t: u32, kind: String },
    Level { t: u32, class: String, level: u32 },
    Rank { t: u32, rank: u32 },
    Projectile { t: u32, src: u32, dst: u32, path: Vec<[i32; 2]> },
}

impl Ev {
    pub fn t(&self) -> u32 {
        match self {
            Ev::Move { t, .. }
            | Ev::Attack { t, .. }
            | Ev::Hurt { t, .. }
            | Ev::Die { t, .. }
            | Ev::Rule { t, .. }
            | Ev::Telegraph { t, .. }
            | Ev::Pickup { t, .. }
            | Ev::Use { t, .. }
            | Ev::Fact { t, .. }
            | Ev::Overlay { t, .. }
            | Ev::Spawn { t, .. }
            | Ev::Steal { t, .. }
            | Ev::Ally { t, .. }
            | Ev::Descend { t, .. }
            | Ev::Exit { t, .. }
            | Ev::Note { t, .. }
            | Ev::Callout { t, .. }
            | Ev::Tame { t, .. }
            | Ev::Hatch { t, .. }
            | Ev::Level { t, .. }
            | Ev::Rank { t, .. }
            | Ev::Projectile { t, .. } => *t,
        }
    }
    /// Renderable, non-movement events (the "events per 60 turns" gate).
    pub fn renderable(&self) -> bool {
        !matches!(self, Ev::Move { .. } | Ev::Rule { .. } | Ev::Fact { .. } | Ev::Note { .. })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExitPending {
    pub items: Vec<InvItem>,
    pub tier: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepResult {
    pub events: Vec<Ev>,
    pub snapshot: Snapshot,
    pub run_over: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_pending: Option<ExitPending>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastDepth {
    pub depth: u32,
    pub reach: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastCause {
    pub cause: String,
    pub share: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    pub depths: Vec<ForecastDepth>,
    pub causes: Vec<ForecastCause>,
    pub known_to: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceTurn {
    pub t: u32,
    pub row: i32,
    pub verb: Verb,
    pub hp: i32,
    pub foes: i32,
    pub telegraphs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Trace {
    pub turns: Vec<TraceTurn>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Patch {
    pub row: Row,
    pub insert_at: usize,
    pub survive: f64,
    pub forecast_delta: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Death {
    pub run_id: u32,
    pub depth: u32,
    pub cause: String,
    pub margin: String,
    pub verdict: String,
    /// Survival of the unpatched rules over the reseeded replays (addition; 0..1).
    #[serde(default)]
    pub baseline: f64,
    pub trace: Trace,
    pub patches: Vec<Patch>,
    pub morgue: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Highlight {
    pub pattern: String,
    pub score: i32,
    pub t: u32,
    pub run_id: u32,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeathCount {
    pub cause: String,
    pub n: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReturnReport {
    pub elapsed_s: u64,
    pub runs: u32,
    pub sampled: bool,
    pub learned: Vec<String>,
    pub bests: Vec<String>,
    pub found: Vec<InvItem>,
    pub deaths: Vec<DeathCount>,
    pub pending: Vec<String>,
    pub reel: Vec<Highlight>,
    pub marks_earned: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worst_death: Option<Death>,
    pub live: Snapshot,
    pub tamed: Vec<String>,
    pub hatched: Vec<String>,
    pub lost: Vec<String>,
    pub xp: XpReport,
    pub salvaged: Vec<SalvageRow>,
    pub renown: RenownReport,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct XpReport {
    pub class: String,
    pub gained: u32,
    pub level_ups: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SalvageRow {
    pub kind: String,
    pub n: u32,
    pub gold: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RenownReport {
    pub gained: u32,
    pub rank: u32,
    pub ranks_up: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ClassProg {
    pub level: u32,
    pub xp: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ForgeRow {
    pub salvaged: u32,
    pub craftable: bool,
    pub tier: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Companion {
    pub id: u32,
    pub kind: String,
    pub name: String,
    pub level: u32,
    pub tags: Vec<String>,
    pub gen: u32,
    pub rules: RuleSet,
    pub max_rows: usize,
    pub hp: i32,
    pub max_hp: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Egg {
    pub id: u32,
    pub kind: String,
    pub tags: Vec<String>,
    pub gen: u32,
    pub hatch_in: u32,
    pub from_loss: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LedgerRow {
    pub kind: String,
    pub seen: bool,
    pub known: bool,
    pub tamed: bool,
    pub bred: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grave {
    pub heir: u32,
    pub depth: u32,
    pub cause: String,
    pub deeds: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lineage {
    pub seed: u64,
    pub heir: u32,
    #[serde(rename = "trait")]
    pub trait_: String,
    pub class: String,
    pub best_depth: u32,
    pub marks: u32,
    pub facts: Vec<String>,
    pub unlocks: Vec<String>,
    pub vault: Vec<InvItem>,
    pub graveyard: Vec<Grave>,
    pub trophies: Vec<String>,
    pub sets: Vec<RuleSet>,
    pub active_set: usize,
    pub ended: bool,
    pub party: Vec<Companion>,
    pub kennel: Vec<Companion>,
    pub eggs: Vec<Egg>,
    pub party_slots: u32,
    pub ledger: Vec<LedgerRow>,
    pub gold: i32,
    pub supplies: Vec<InvItem>,
    pub classes: std::collections::BTreeMap<String, ClassProg>,
    pub forge: std::collections::BTreeMap<String, ForgeRow>,
    pub renown: u32,
    pub rank: u32,
    pub keep_pref: String,
    pub insured: Vec<u32>,
}

/// Supply catalogue entry (Addendum B).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupplyInfo {
    pub kind: String,
    pub price: i32,
    pub label: String,
}

/// Unlock catalogue entry (addition to the contract; see README).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnlockInfo {
    pub id: String,
    pub cost: u32,
    pub owned: bool,
    pub available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_json_is_tagged_with_k() {
        let e = Ev::Attack { t: 3, src: 1, dst: 2, dmg: 4, hit: true, verb: None };
        assert_eq!(serde_json::to_string(&e).unwrap(), r#"{"k":"attack","t":3,"src":1,"dst":2,"dmg":4,"hit":true}"#);
        let e = Ev::Overlay { t: 1, x: 2, y: 3, ov: OverlayKind::Gas, ttl: 4 };
        assert!(serde_json::to_string(&e).unwrap().contains(r#""ov":"gas""#));
        let e = Ev::Rule { t: 1, row: -1, verb: Verb::new("retreat"), text: "cowardly → retreat".into() };
        assert!(serde_json::to_string(&e).unwrap().contains(r#""row":-1"#));
    }
    #[test]
    fn hero_snapshot_flattens_entity_and_trait() {
        let h = HeroSnap {
            entity: Entity { id: 1, kind: "hero_fighter".into(), name: None, x: 1, y: 2, hp: 3, max_hp: 4, tags: vec![], ally: None, telegraph: None, cid: None },
            inv: vec![],
            weapon: Some("dagger".into()),
            armour: None,
            class: "fighter".into(),
            trait_: "brave".into(),
        };
        let s = serde_json::to_string(&h).unwrap();
        assert!(s.contains(r#""trait":"brave""#));
        assert!(s.contains(r#""kind":"hero_fighter""#));
        assert!(!s.contains("armour"));
    }
}
