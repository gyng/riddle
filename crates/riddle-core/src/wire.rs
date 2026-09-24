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
    /// Cut 4: a hostile the hero remembers but cannot see, at its last-seen tile.
    #[serde(default, skip_serializing_if = "is_false")]
    pub remembered: bool,
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
    /// Cut 5 §4: an opened vault waiting for `choose(itemId)` (watched runs only; offline
    /// and unwatched runs pick by `Lineage.vault_pref`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_choice: Option<VaultChoice>,
    /// Cut 7 §4: the room the hero stands in (`id` 0 = a corridor or a cave; rooms are
    /// numbered from 1 in the floor's room list) and the awake hostiles in it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<RoomRef>,
    /// Cut 7 §4: the floor's room count (0 on a cave), for the `D3 · 4 rooms` ambient.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rooms: Option<u32>,
    /// Cut 12 §4: the floor's one situation, one word (`nest`), for `D4 · 9 rooms · a nest`;
    /// absent on a floor without one (D1–2, a boss's cave floor, the bottom).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_twist: Option<String>,
}

/// Cut 7 §4: a room as a scene — the viewer holds 1× while a room with ≥ 2 hostiles is not
/// yet clear or left.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RoomRef {
    pub id: u32,
    pub hostiles: u32,
}

/// Cut 5 §4: the three items of an opened vault; `choose(id)` takes one, the rest vanish.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultChoice {
    pub items: Vec<InvItem>,
    /// Cut 14 (QA on 56f2a1d: the sheet "closes by itself with no timer"): ticks of the grace
    /// left at this snapshot; the sheet shows them as a shrinking bar.
    #[serde(default)]
    pub left: u32,
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
    /// Cut 6 §1: the gold the `return_row` would bring home right now (`$84 · keeps $50`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kept: Option<i32>,
    /// Cut 13 §1: the oscillation guard has fired this floor (`run.stuck_fires > 0`): the
    /// run is stalling and a stall pays nothing — the HUD reads `keeps $0 · stalling`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stalling: bool,
}

/// Cut 6 §1: the ledger line of an exit — one arithmetic line the player can check.
/// `text` ≤ 14 words: `$84 carried · return keeps 60% → $50` (death: `$144 carried · death
/// keeps 0% → $0 · bones: 7 items on D5`). `spent` / `spent_on` are the automations' purchases
/// on coming home (`auto_supply`); salvage is its own ledger movement (`Lineage.gold_ledger`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExitLine {
    pub carried: i32,
    pub keep_pct: i32,
    pub kept: i32,
    pub spent: i32,
    pub spent_on: Vec<String>,
    pub text: String,
    /// Cut 9 §5: the exit's last `EXIT_TRACE_LEN` hero turns with row accounting (every tier;
    /// a death's own `Death.trace` is longer, so its line leaves this out).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Trace>,
    /// What the exit salvaged before the keep sheet (the 40 % a return cuts), per kind in
    /// coins — the watched report's `salvaged` lines are these plus what the sheet let go
    /// (QA on e0f87e7: `SALVAGED $6` beside the ledger's `+$12 salvage`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub salvaged: Vec<SalvageRow>,
    /// The run, so a report's trace links can open its replay (the return sheet's could; a
    /// report's could not — QA on e0f87e7).
    #[serde(default)]
    pub run_id: u32,
}

/// Cut 6 §1: one gold movement (`+50 returned D5`, `−40 heal`, `−8 insure sword`, `+3
/// salvage`, `−50 hatch`); `why` ≤ 3 words; `t` is the lineage tick.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldLine {
    pub t: u64,
    pub delta: i32,
    pub why: String,
}

/// Cut 6 §5: a boss whose counter row is known (`Lineage.counters`): the row and its ≤ 3-word
/// text (`attack boss`, `throw fire, boss`, `read silence`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Counter {
    pub boss: String,
    pub row: Row,
    pub text: String,
}

/// Cut 6 §3: why a row above the fired one did not fire this action (≤ 3 words from
/// `turn::ROW_REASONS`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowWhy {
    pub row: usize,
    pub why: String,
    /// Cut 11 §1: the event that put the state there (`none held` → `den took the heal, D3`;
    /// `no path` → `gas cloud, this room`; `not in view` → `last seen D6 (17,3)`; `cooldown`
    /// → `cooldown 12 ticks left`; `locked cond` → `◆2 cond: alert`), ≤ 8 words, with the
    /// tick to scrub the replay to. Absent for a condition reason (`hp not <30%`) and when
    /// the run has no such event.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub because: Option<Because>,
}

/// Cut 11 §1: a cause's cause — `text` ≤ 8 words, `t` the run tick it happened at, `depth` the
/// floor. On `RowWhy.because`, `Trace.provenance`, `Death.chain`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Because {
    pub text: String,
    pub t: u32,
    pub depth: u32,
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
    /// Cut 10 §3: `amount` is the gold the theft took off the run's loot (`$26 → $10`; the
    /// callout reads `stolen $16`) — or, on a companion's theft from a foe, the gold it
    /// brought; absent when the loot did not move.
    Steal { t: u32, id: u32, item: String, #[serde(default, skip_serializing_if = "Option::is_none")] amount: Option<i32> },
    Ally { t: u32, id: u32, state: String },
    Descend { t: u32, depth: u32, biome: String },
    Exit {
        t: u32,
        tier: String,
        loot_kept: i32,
        /// Cut 6 §1: the ledger line (filled once the exit is settled).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        line: Option<ExitLine>,
        /// Cut 9 §5: the exit's last-5 trace (bank and return; a death has `Death.trace`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace: Option<Trace>,
    },
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
    /// Cut 7 §4: an exit the engine can foresee, `ticks` ahead — a bank walk-out or the
    /// bottom's stairs within three steps (30 ticks), or death in the air (hp ≤ 15% with a
    /// hostile adjacent; once per 100 ticks). An instant exit (`return`, recall) says 0.
    Ending { t: u32, ticks: u32 },
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
            | Ev::Bones { t, .. }
            | Ev::Ending { t, .. } => *t,
        }
    }
    /// Renderable, non-movement events (the "events per 60 turns" gate).
    pub fn renderable(&self) -> bool {
        !matches!(self, Ev::Move { .. } | Ev::Rule { .. } | Ev::Fact { .. } | Ev::Note { .. } | Ev::Rest { .. } | Ev::Ending { .. })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExitPending {
    pub items: Vec<InvItem>,
    pub tier: String,
    /// What each item (by position) salvages at this exit, in coins — the engine's own
    /// arithmetic (`salvage_value × pct ÷ GOLD_DIVISOR`; the client's old table read 4× it).
    #[serde(default)]
    pub worth: Vec<i32>,
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
    /// Cut 9 §3: the binomial half-width of `reach` over the sims that ran (`1.96·√(p(1−p)/n)`,
    /// 0..1), so a wobble inside it reads as noise (`D4 71% ±6`). The refine pass narrows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pm: Option<f64>,
    /// Cut 10 §2: on the row a boss wall gates (the boss's floor + 1) whose counter fact is
    /// known and whose verb no row of the set carries: the counter row and its ≤ 3-word text
    /// (`D9 0% · warlord · try: attack boss`); the client inserts it at the **top**.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "try")]
    pub try_: Option<ForecastTry>,
}

/// Cut 10 §2: a forecast row's `try` — the known-but-absent counter of the boss whose floor
/// sits just above this depth (`boss` is the kind, `text` from `facts::counter_text`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastTry {
    pub boss: String,
    pub row: Row,
    pub text: String,
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
    /// Cut 12 §3: how a send ends over the same sims (rates summing to 1) and the mean gold
    /// brought home per send (`bank 40% · return 35% · death 25% · ~$54`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ends: Option<ForecastEnds>,
    /// Cut 13 §5: this forecast is the refine pass (100 sims); a first paint (50) is marked
    /// `…` by the client so a re-read does not look like a re-roll.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub refined: bool,
}

/// Cut 12 §3: how a send ends — `bank` / `return` / `death` as shares of a panel of sends run
/// to their exit (the bars' own panel — QA on 3d71c33; a run at the cap is a stall) and
/// `gold`, the mean loot kept per send by the exit's own share (bank 100% · return 60% · death 0%).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastEnds {
    pub bank: f64,
    #[serde(rename = "return")]
    pub return_: f64,
    pub death: f64,
    /// Sends that came home by the turn cap or a stall — nothing in the rules returned them.
    #[serde(default)]
    pub stall: f64,
    pub gold: f64,
    /// Cut 13 §5: the half-width of the death share over the ends panel (`death 5% ±4`).
    #[serde(default)]
    pub pm: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceTurn {
    pub t: u32,
    pub row: i32,
    pub verb: Verb,
    pub hp: i32,
    pub foes: i32,
    pub telegraphs: Vec<String>,
    /// Cut 4: the first row this action whose conditions held but whose verb could not
    /// execute (`R1 retreat ✗ no path`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<String>,
    /// Cut 6 §3: every row above the fired one (all rows when a trait or a chore acted) with
    /// the reason it did not fire (`none held` · `no path` · `not in view` · `hp not <30%`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<RowWhy>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Trace {
    pub turns: Vec<TraceTurn>,
    /// Cut 11 §3: the run's provenance log — every `because` event of the run (thefts, uses,
    /// finds, targets lost, path blocks, cooldown starts), oldest first, ≤ `PROV_CAP`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Vec<Because>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Patch {
    pub row: Row,
    /// Cut 11 §2: `-1` on the lock pseudo-patch (`root` = `◆2 cond: alert`): nothing is
    /// inserted; `row` is the set's own locked row and `survive` its measured survival with
    /// the condition unlocked. The client renders it as an unlock button.
    pub insert_at: i32,
    pub survive: f64,
    pub forecast_delta: f64,
    /// Stall patches (addition): `replace` swaps the row at `insert_at` for `row`; `remove`
    /// deletes the row at `insert_at` (`row` echoes it). Absent = insert before `insert_at`.
    #[serde(default, skip_serializing_if = "is_false")]
    pub replace: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub remove: bool,
    /// Cut 11 §2: the candidate addresses a `because`-root of the killing turn (`den took the
    /// heal` → `foe_tag:thief → attack tag:thief`; `◆2 cond: alert` → the unlock), ≤ 6 words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<PatchRoot>,
    /// Cut 11 §4: on a `dice` death, a candidate kept with its measured survival although it
    /// is under the bar (`survives 40% · dice`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub below_bar: bool,
}

/// Cut 11 §2: what a root-cause patch answers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchRoot {
    pub text: String,
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
    /// Cut 9 §5: the last-5 trace of the latest run the named row ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trace: Option<Trace>,
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
    /// Cut 6 §1: the death's ledger line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<ExitLine>,
    /// Cut 11 §2: the `because` entries of the killing turn's rows, in row order, deduped by
    /// text — the chain under the trace. Absent when no row had one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain: Option<Vec<Because>>,
    /// The rules the run died under, so the trace's row accounting is labelled with the run's
    /// own rows in the editor's words (an old death from the chronicle keeps its rules;
    /// before this the client parsed the morgue's short forms — `R1 drink ?`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rules: Option<RuleSet>,
    /// Cut 13 §4: the run's last two chronicle notes (≤ 8 words each: `The green one: fire.
    /// Gambled: fire potion.`), under the headline — the beat the morgue alone carried.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Highlight {
    pub pattern: String,
    pub score: i32,
    pub t: u32,
    pub run_id: u32,
    pub text: String,
    /// Cut 5 §1: the episode behind a story line (absent on `bones`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arc: Option<HighlightArc>,
}

/// Cut 5 §1: an episode's shape — the low-water HP, the row that fired at it (−1 trait, −2
/// chores), the threat key (`jackal`, `goblin_warlord`, `gas`, `nest`, `shrine`, `vault`,
/// `stray`, `none`) and the resolution text (`banked $58`, `reached D6`, `first boss`,
/// `boss slain`, `returned`, `lost the thread`, `died to gas`, `jackal Uleth fell`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighlightArc {
    pub low_hp: u32,
    pub row: i32,
    pub threat: String,
    pub resolution: String,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<Snapshot>,
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
    /// Cut 13 §1: sends that stalled (they are among `returned`, keeping nothing): the tiles
    /// count them apart — `1 RETURNED` for a stall read as a return to two QA players.
    #[serde(default)]
    pub stalled: u32,
    /// Cut 13 §3: what the automations bought during the absence, per kind in coins
    /// (`heal ×16 · −$640`), the report's SPENT section; `banked + returned + salvage − spent`
    /// is the header's delta.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spent: Vec<SalvageRow>,
    /// Cut 13 §3: the absence's gold movements to the coin — what the exits brought home
    /// (banked + returned), the salvage, the heirs' wake pay — so the report's gold line
    /// reconciles the header on a long absence too (the exit lines are capped per slice).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gold: Option<GoldSummary>,
    /// The deepest floor any run of this absence reached (the report's `deepest` tile: a
    /// delta like the tiles beside it; the lineage best is the header's — QA on 952e306:
    /// "`1 RUNS · D4 BEST` for a run that peaked at D2").
    #[serde(default)]
    pub deepest: u32,
    /// Cut 6 §1: the ledger lines of the absence's last five exits, oldest first.
    #[serde(default)]
    pub exits: Vec<ExitLine>,
    /// Cut 16 §1: depths picked clean at the report (≥ 3 banks/returns from them, shallower
    /// than the lineage best; ascending) — the report's `D3 · picked clean` line.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub picked: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct XpReport {
    pub class: String,
    pub gained: u32,
    pub level_ups: u32,
}

/// Cut 13 §3: `home + salvage + wake − spent` is the purse's delta over the absence.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoldSummary {
    pub home: i32,
    pub salvage: i32,
    pub wake: i32,
    pub spent: i32,
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
    /// Cut 9 §10: the ladder's next rung (`salvaged 3/5 → craftable`); absent at the top.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<ForgeNext>,
}

/// Cut 9 §10: the salvage count the next forge rung needs and its label (`craftable`, `tier
/// 1`, `tier 2`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ForgeNext {
    pub need: u32,
    pub label: String,
}

/// Cut 9 §10: the forge ladder — (salvaged needed, rung label), in order.
pub const FORGE_LADDER: [(u32, &str); 3] = [(5, "craftable"), (15, "tier 1"), (40, "tier 2")];

impl ForgeRow {
    /// The row for a salvage count, with its rung and its next rung.
    pub fn at(salvaged: u32) -> ForgeRow {
        let mut r = ForgeRow { salvaged, ..Default::default() };
        r.settle();
        r
    }
    /// Cut 9 §10: recompute `craftable`, `tier` and `next` from `salvaged`.
    pub fn settle(&mut self) {
        self.craftable = self.salvaged >= FORGE_LADDER[0].0;
        self.tier = if self.salvaged >= FORGE_LADDER[2].0 {
            2
        } else if self.salvaged >= FORGE_LADDER[1].0 {
            1
        } else {
            0
        };
        self.next = FORGE_LADDER.iter().find(|(need, _)| self.salvaged < *need).map(|(need, label)| ForgeNext { need: *need, label: (*label).into() });
    }
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
    /// Cut 7 §1: a boss kind whose counter is known — the row the bestiary card offers as a
    /// chip and its ≤ 3-word text (`Lineage.counters` holds the same for the camp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter: Option<CounterChip>,
}

/// Cut 7 §1: the counter row on a bestiary card.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterChip {
    pub row: Row,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Grave {
    pub heir: u32,
    pub depth: u32,
    pub cause: String,
    pub deeds: Vec<String>,
    /// Cut 9 §7: the death's run id while the engine still keeps its record (the last
    /// `KEPT_DEATHS` deaths; `death(id)` answers for these, through a save).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_id: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lineage {
    pub seed: u64,
    pub heir: u32,
    #[serde(rename = "trait")]
    pub trait_: String,
    /// Cut 13 §2: the two traits a new heir may wake with (drawn from the lineage seed, never
    /// the last heir's); empty once chosen or sent. `set_trait(name)` picks one; a send without
    /// a pick keeps the first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trait_offer: Vec<String>,
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
    /// Cut 4: variants the lineage has already finished the dungeon with.
    #[serde(default)]
    pub ascended: Vec<String>,
    /// Cut 5 §2: one line per ended heir, oldest first (cap 40).
    #[serde(default)]
    pub chronicle: Vec<String>,
    /// Cut 5 §4: what an unwatched vault choice takes (`weapon | armour | potion | scroll`).
    #[serde(default)]
    pub vault_pref: String,
    /// Cut 6 §1: the last 20 gold movements, oldest first (`ledger` is the bestiary).
    #[serde(default)]
    pub gold_ledger: Vec<GoldLine>,
    /// Cut 6 §5: bosses whose counter row is known, with the row and its ≤ 3-word text.
    #[serde(default)]
    pub counters: Vec<Counter>,
    /// Cut 8B §1: the active set's combos (adjacent rows the engine resolves as one move), in
    /// row order; recomputed on every `set_rules`.
    #[serde(default)]
    pub combos: Vec<crate::rules::ComboHit>,
    /// Cut 16 §1: depths picked clean now (as `ReturnReport.picked`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub picked: Vec<u32>,
    /// Cut 16 §2: the classes the new heir may wake as — the owned ones, the current first —
    /// while the wake is open (`trait_offer` non-empty) and ≥ 2 are owned; `set_class(name)`
    /// picks (it sticks to every run until changed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_offer: Vec<ClassChip>,
}

/// Cut 16 §2: a class chip at the wake (`rogue · vanish`). `signature` is a verb id
/// (`shield_bash | vanish | mark | slow`); `level` the class's level; `opens` the level the
/// signature opens at (the chip can read `mark L7` while `level < opens`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClassChip {
    pub class: String,
    pub signature: String,
    pub level: u32,
    pub opens: u32,
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UnlockInfo {
    pub id: String,
    pub cost: u32,
    pub owned: bool,
    pub available: bool,
    /// Cut 2 §3: the human-readable gate (fact/trophy) still missing, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
    /// Cut 4 §9: for a tactic card or a verb not yet owned whose gate is open, the forecast
    /// reach delta at `best_depth + 1` if it were owned and its row added (0..1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    /// Cut 6 §6: a tactic card's rows (what the card does, as rows), or an automation's effect
    /// as one row-like entry (`{conds: [], verb: {v: "auto", a: "keeps best weapon+armour"}}`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<Vec<Row>>,
    /// Cut 12 §1: where a bought card's row goes — before the set's engagement row (the first
    /// `attack` / `shoot`), else the end; the card's `delta` is measured there. Tactic cards only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insert_at: Option<usize>,
    /// Cut 13 §5: the half-width of `delta` (the paired panel's); a delta within it reads
    /// `reach ~0` on the client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pm: Option<f64>,
    /// Cut 15 §2: the price in gold today (`meta::gold_price`: `150 × cost × (4 + gold_buys) / 4`);
    /// 0 for a free unlock (not gold-buyable). The gates in `needs` other than `◆N more` hold
    /// for a gold buy too (`buyUnlockGold`).
    #[serde(default)]
    pub gold: u32,
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
            entity: Entity { id: 1, kind: "hero_fighter".into(), name: None, x: 1, y: 2, hp: 3, max_hp: 4, tags: vec![], ally: None, telegraph: None, cid: None, remembered: false },
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
