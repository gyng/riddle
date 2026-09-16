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
    /// Cut 2 §7: what is on the line right now.
    #[serde(default)]
    pub stake: Stake,
    /// Cut 3 (addition): the hero's sight radius on this floor (the Deep is dark).
    #[serde(default = "default_vision")]
    pub vision: i32,
}

fn default_vision() -> i32 {
    crate::descent::VISION_LIT
}

/// Cut 2 §7: loot on the hero, brought vault items (insured = kept on death), and the row
/// that would bank or return if any.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Stake {
    pub loot: i32,
    pub brought: Vec<StakeItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_row: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StakeItem {
    pub label: String,
    pub insured: bool,
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
    /// Cut 2 §1: emitted at exit; camp rest (or wake) in seconds.
    Rest { t: u32, seconds: u32 },
    /// Cut 2 §2: a bones pile left on death, or recovered by a later heir.
    Bones { t: u32, heir: u32, items: u32 },
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
            | Ev::Projectile { t, .. }
            | Ev::Rest { t, .. }
            | Ev::Bones { t, .. } => *t,
        }
    }
    /// Renderable, non-movement events (the "events per 60 turns" gate).
    pub fn renderable(&self) -> bool {
        !matches!(self, Ev::Move { .. } | Ev::Rule { .. } | Ev::Fact { .. } | Ev::Note { .. } | Ev::Rest { .. })
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
    /// Stall patches (addition): `replace` swaps the row at `insert_at` for `row`; `remove`
    /// deletes the row at `insert_at` (`row` echoes it). Absent = insert before `insert_at`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub replace: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub remove: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Stall verdict (addition): a batch of ≥ 4 runs with no death and no new depth names the
/// row that ended most of them and offers patches with forecast deltas at the stall depth + 1
/// (`survive` on these is the patched reach at that depth).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Stall {
    pub row: usize,
    pub fired: u32,
    pub text: String,
    pub patches: Vec<Patch>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worst_death_id: Option<u32>,
    pub live: Snapshot,
    pub tamed: Vec<String>,
    pub hatched: Vec<String>,
    pub lost: Vec<String>,
    pub xp: XpReport,
    pub salvaged: Vec<SalvageRow>,
    pub renown: RenownReport,
    /// Cut 2 §1: camp rest and wake consumed by this absence (seconds).
    #[serde(default)]
    pub rested_s: u64,
    #[serde(default)]
    pub banked: u32,
    #[serde(default)]
    pub returned: u32,
    /// Cut 2 §2: bones piles recovered this absence ("heir 3 · D4 · 5 items").
    #[serde(default)]
    pub bones_found: Vec<String>,
    /// Stall verdict (addition): present when the last ≥ 4 runs all came home with no new depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stall: Option<Stall>,
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
    /// Cut 2 §5: five kills of the kind.
    #[serde(default)]
    pub studied: bool,
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
    /// Cut 2 §1: camp rest remaining (seconds) before the next expedition; `send` skips it.
    #[serde(default)]
    pub rest_left_s: u32,
    /// Cut 2 §2: dead heirs' kit waiting on the floor (max 3, oldest expires).
    #[serde(default)]
    pub bones: Vec<BonesPile>,
    /// Cut 3: ascension level and variant ("" at level 0).
    #[serde(default)]
    pub ascension: Ascension,
}

/// Cut 3: `{level, variant}`; `variant` is one of `no_rest short_list bones_only hunted`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Ascension {
    pub level: u32,
    pub variant: String,
}

/// Cut 2 §2: a bones pile on the wire.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BonesPile {
    pub depth: u32,
    pub heir: u32,
    pub items: u32,
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
    /// Cut 2 §3: the human-readable gate (fact/trophy) still missing, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
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
