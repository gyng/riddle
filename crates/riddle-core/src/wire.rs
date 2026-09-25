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
    /// QA on a946e04 (qaT: a short purse sent the run from D1 unsaid): the floor this run
    /// started on (1, or the waystone it paid for).
    #[serde(default = "one")]
    pub start: u32,
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
    /// Cut 19 §1: the item the vault preference takes when the grace runs out (the beat's
    /// `took mail` before it lands); `choose(id)` overrides it while `left > 0`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<u32>,
}

/// Cut 19 §1: one cage preference measured for the active set (`Game::cage_forecast`) — the
/// camp panel's sims with `vault_pref` set to `pref`, against the current preference's panel
/// (paired: the same seeds). `depth` is the bar the reach is read at (the set's bank row's
/// depth when it has one, else the lineage's best depth); `delta` is the picker's headline —
/// the bank share's move when either panel banks, else the reach's.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CageOption {
    pub pref: String,
    pub current: bool,
    pub depth: u32,
    pub reach: f64,
    pub reach_delta: f64,
    pub bank: f64,
    pub bank_delta: f64,
    pub gold: f64,
    pub gold_delta: f64,
    pub delta: f64,
    /// The 95 % half-width of `reach` (a delta inside it reads `~0`).
    pub pm: f64,
    /// QA on 778fa1b: measured on the refined panels (the forecast's own sims once refined).
    #[serde(default)]
    pub refined: bool,
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
    /// QA on 1a2a4a9: a return (or bank) row has acted and the hero is walking home — the walk
    /// replaces the chores until the exit (`Run.homeward`); the HUD's `returning`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub returning: bool,
    /// Cut 20 §4 (AC: `carry $78 · keeps $78 · bank R4`, then died with $0): what a death
    /// would keep of the carried gold right now (the death tier's share), beside `kept`, so
    /// the HUD reads `carry $78 · bank keeps $78 · death $0`.
    #[serde(default)]
    pub death_keep: i32,
    /// QA on 912e135 (qaW: the strip's `−$8 swap` twice against the death line's `−$2
    /// swapped`): the carry this run's pack swaps have taken so far (`Run.swapped`, the exit
    /// line's `swapped` at the end) — the strip names a fall `swap` by this counter's rise.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub swapped: i32,
    /// QA on 0c6e126 (qaY: `−$5 swap → waxen scroll?` read as a price): what the last costly
    /// swap left on the floor (`axe`) — the strip's fall reads `−$5 left axe`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swap_left: Option<String>,
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
    /// QA on 92eb880 (qaN: `fighter +0 · L4 ↑1` — the client's own ladder, 40·L², was not the
    /// core's): the XP this run earned, the part that crossed a level included, and the
    /// levels it crossed. The watched report reads these, never a difference of levels.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub xp: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub level_ups: u32,
    /// QA on e75ec29 (qaR: a packed heal stolen on D1, nothing on the exit): the labels of what
    /// thieves took this run and it never got back (`· stolen heal`; flavour-named while
    /// unidentified).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stolen: Vec<String>,
    /// QA on a946e04 (qaT: run 6's `+$71 banked · −$40 spent` beside `$51 → $32` — the send's
    /// `−$50 waystone D5` left out): the toll this run's send paid (0 from D1 or on the night's
    /// pass), the floor it started on, and the waystone it wanted and did not start on (a toll
    /// the purse could not pay, or unlit: `from D1 · toll short`).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub toll: i32,
    #[serde(default = "one")]
    pub start: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_short: Option<u32>,
    /// QA on a946e04 (qaT: the strip's `−$36 stolen`): the carried gold the thefts this run
    /// never got back took off it (each stolen item's worth).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub stolen_gold: i32,
    /// QA on 778fa1b (qaU: `carry $61 −$37 swapped`, `$106 −$9 swapped` on the strip and in
    /// no ledger): the carried gold this run's pack swaps took off (coins; a find taken in the
    /// place of a dearer carried item — the strip's falls, summed). `carried` is after them:
    /// what the run picked up less `stolen_gold` less this.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub swapped: i32,
    /// QA on 0c6e126 (qaY: the report's death lines listed items and buried the killer): a
    /// death's killer as it reads after `died to` (`a goblin archer`, `gas`) — the report's
    /// lines lead with it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    /// QA on 0c6e126 (qaY: `−$5 swapped` naming no item): what the costly swaps left on the
    /// floor, per label (`axe`) — the line's `−$5 swapped` names them (`−$5 left axe`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub swap_left: Vec<KindCount>,
    /// QA on 778fa1b: the heir purse's top-up this death paid (the `+$N wake` of `text`), 0 when
    /// none.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub wake: i32,
    /// QA on 778fa1b (qaV): every item the run found and where it ended (`FoundRow`), and how
    /// many it found (units: a leash stack counts each) — Σ `found[].n` == `found_n`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub found: Vec<FoundRow>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub found_n: u32,
    /// QA on e75ec29 (qaR): a death whose heir purse was already at `engine::WAKE_PAY` — no
    /// top-up (`purse full`); a top-up reads `+$N wake` in `text` (and `wake`). QA on 778fa1b
    /// (qaU: `purse full` beside $1434 read as a cap): only while the purse is under
    /// `engine::PURSE_FULL_BAND` ($80) — a richer death has nothing to say of the purse.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub purse_full: bool,
    /// Cut 21 §2: the found supplies this exit put on the shelf, per kind (the client's `found
    /// heal → shelf`; `text` does not carry it).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shelved: Vec<KindCount>,
    /// QA on 912e135 (qaW: `bones: 12 items on D6` beside a list of 14 — `leash ×3` counted
    /// charges; `11 items on D6` beside 10 — the list held only the finds): the death's whole
    /// pile per kind, `n` the items (a stack is one) — Σ `n` is the `bones: N items` of `text`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bones: Vec<KindCount>,
    /// Cut 24 §1 (AL: the Warlord > 4 min on `attack nearest`, his bar full): a boss whose HP
    /// did not move for `turn::BOSS_STILL` of the hero's actions drove him off his floor — a
    /// `return`-tier exit (keeps 60 %) whose verdict is `no counter`. Absent on every other exit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driven: Option<DrivenOff>,
    /// Cut 24 §2 (AK: identical end-of-run summaries run after run): what was new this run,
    /// most telling first, ≤ 3 — a first (`first: Warlord slain`), a record (`record: D10`),
    /// a named kill (`avenged Ulak`), a find kind never found (`new find: mail`), a first
    /// situation (`first: the captive`), a drive-off (`driven off: Warlord`), facts learned
    /// (`learned 3`); a run with nothing new has one line, the thing that differed from the last
    /// run (`k` `differ`: `deeper: D9, last D8` · `banked, last returned` · `+$23 on last`).
    /// The report leads with these, before the counts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub news: Vec<News>,
}

/// Cut 24 §2: one line of `ExitLine.news` — its kind (`first` · `record` · `named` · `find`
/// · `situation` · `driven` · `learned` · `differ`) and its text (≤ 6 words, lower case).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct News {
    pub k: String,
    pub text: String,
}

/// Cut 24 §1: the `no counter` exit — the boss (`goblin_warlord`, `Warlord`), the floor, the
/// verdict word, the defence that shrugged every blow (`shield wall`, ≤ 3 words), and the
/// counter to write, as a row the editor can insert (`foe: boss → attack boss`) and in words
/// (`attack boss`). The client renders `Warlord · no counter · shield wall · try: attack boss`
/// and offers `row` like a death's patch.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DrivenOff {
    pub boss: String,
    pub title: String,
    pub depth: u32,
    pub verdict: String,
    pub defence: String,
    pub counter: String,
    pub row: crate::rules::Row,
}

/// Cut 21 §2: a kind and how many (`ExitLine.shelved`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct KindCount {
    pub kind: String,
    pub n: u32,
}

/// QA on 778fa1b (qaV: FOUND `leather +1`, `dagger ×2`, `aggravate scroll ×2` beside SALVAGED
/// `dagger ×1` — neither kept, salvaged nor shelved): where the run's finds ended, per kind and
/// place (`ExitLine.found`). `fate`: `kept` (the vault), `salvaged`, `shelved` (the supply
/// shelf), `used` (drunk, read, thrown, spent), `left` (swapped out or put down on a floor),
/// `stolen` (a thief kept it), `bones` (a death's pile), `sheet` (on the keep sheet, not yet
/// decided — `keep` moves it to `kept` / `salvaged`), `lost` (nowhere: a bug; qa.rs holds it 0).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FoundRow {
    pub kind: String,
    pub n: u32,
    pub fate: String,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Cut 6 §1: one gold movement (`+50 returned D5`, `−40 heal`, `−8 insure sword`, `+3
/// salvage`, `−50 hatch`); `why` ≤ 3 words; `t` is the lineage tick.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GoldLine {
    pub t: u64,
    pub delta: i32,
    pub why: String,
    /// QA on 778fa1b (qaV: `repeat heal ×1 · −$104` for four heals at $26): the supplies a
    /// line bought or refunded (`repeat heal`, `bought heal`, `refund heal` — merged lines add
    /// up); 0 on any other line.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub n: u32,
    /// QA on 912e135 (qaW: sixteen `$0 died D6` rows and none of the ~$1,900 the runs carried):
    /// on an exit's line, the carried gold the exit did not keep (`$0 died D6 · $157 lost`).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub lost: i32,
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
        /// Cut 6 §1: the ledger line (filled once the exit is settled). (Boxed: the line is the
        /// largest payload of any event.)
        #[serde(default, skip_serializing_if = "Option::is_none")]
        line: Option<Box<ExitLine>>,
        /// Cut 9 §5: the exit's last-5 trace (bank and return; a death has `Death.trace`).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trace: Option<Trace>,
    },
    Note { t: u32, text: String },
    /// Cut 23 §3: `why` — the reason on tap, ≤ 3 words: a `✗` callout's block gloss (`read ✗
    /// no use` → `nothing to learn`), a telegraph's foe trait (`bloat swells` → `gas burst next`).
    Callout { t: u32, text: String, #[serde(default, skip_serializing_if = "Option::is_none")] why: Option<String> },
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
    /// hostile adjacent; once per 100 ticks). An instant exit (`bail`, recall) says 0; a `return` walks to the up-stairs (Cut 19 §2).
    Ending { t: u32, ticks: u32 },
    /// QA on 1a2a4a9 (qaP: `12/38 → 6/16 → 3/16 → 0/15` on D12 with no line until the death):
    /// the hero's max HP moved — `delta` (−1 per hunger bite on an unlit hunger floor), `max`
    /// after it, `cause` (`hunger`). The callout beside it reads `hunger −1 max`.
    MaxHp { t: u32, id: u32, max: i32, delta: i32, cause: String },
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
            | Ev::MaxHp { t, .. }
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
    /// QA on 23ed91f (qaL: `auto: keep weapon+armour` owned, a full vault, mail · axe · sword
    /// all salvaged): the ids the preference and owned automations keep (`Game::auto_keep`'s
    /// plan, brought vault items first) — what `autoKeep()` does, and the sheet's pre-ticks.
    #[serde(default)]
    pub auto_keep: Vec<u32>,
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
    /// Cut 18 §3: the wall — when `reach` falls to ≤ 5 % here (from over 5 % on the floor
    /// above) and the floor above is a boss's, whose living boss seals its stairs
    /// (`ai::stairs_sealed`): the boss's kind (`goblin_warlord`), so the row reads `D9 0% ·
    /// warlord wall` before the player has met him.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall: Option<String>,
    /// Cut 20 §5: this notch is the lineage's bounty floor (`D12 ×2`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub bounty: bool,
    /// Cut 24 §5 (AK: "the warlord forecast on D9, met on D8"): on the floor a boss stands on —
    /// the floor he is met on (`descent::BOSS_DEPTHS`: the Warlord D8) — his kind. `try` and
    /// `wall` stay on the floor below it (reaching D9 is passing him); the client names the boss
    /// on this row (`D8 · warlord`) and reads the next row's `try` / `wall` as his.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
}

/// Cut 10 §2: a forecast row's `try` — the known-but-absent counter of the boss whose floor
/// sits just above this depth (`boss` is the kind, `text` from `facts::counter_text`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastTry {
    pub boss: String,
    pub row: Row,
    pub text: String,
    /// Cut 24 §5: the floor the boss is met on (this row's depth − 1).
    #[serde(default)]
    pub met: u32,
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
    /// `…` by the client so a re-read does not look like a re-roll. QA on 778fa1b (qaU: the
    /// camp painted 94/78/42 then 95/73/36 for the same rules with no sign it was settling —
    /// `false` was skipped on the wire, so the client could not tell a first pass from an old
    /// core): always serialised, `false` on the first pass.
    #[serde(default)]
    pub refined: bool,
    /// QA on 92eb880: per row of the forecast's set (by index), the earlier row that takes
    /// every moment it could fire (`RuleSet::shadowed_by`) — the editor marks it; `null` for a
    /// row nothing shadows. Empty when no row is shadowed.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shadowed_by: Vec<Option<u32>>,
    /// Cut 21 §1: the floor the sims started on (`Lineage.start` when lit and the toll is
    /// payable, else 1). Rows above it read reach 1.0 (every send is there from its first
    /// tick); the shaft shows from here.
    #[serde(default = "one")]
    pub start: u32,
    /// Cut 23 §2 (AJ: `death 0%`, then the next run died): the sims the shares were drawn from,
    /// and the smallest share one sim makes in whole percent (`⌈100 / sims⌉`) — a share sampled
    /// at 0 of `sims` prints `<{low}%` (`death <2%`), never `0%` (`forecast::share_label`).
    #[serde(default)]
    pub sims: u32,
    #[serde(default)]
    pub low: u32,
}

/// Cut 22 §3 (AH: "most edits moved the forecast less than its ±10–13 error, so I couldn't tell
/// good from bad"): one paired move — the active set's panel minus the previous set's on the
/// same seeds (`forecast::forecast_vs`): `delta` the mean per-seed difference (a 0..1 share,
/// or coins for `gold`), `pm` its own 95 % half-width (1.96 · sd of the per-seed differences
/// / √n) — far tighter than either absolute bar's ± when the two sets mostly play alike. A
/// move inside its `pm` reads `≈`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct VsMove {
    pub delta: f64,
    pub pm: f64,
    /// QA on 912e135 (qaW: `D6 61%` read `▲32`, then `▲38` unedited; a bought leash took D6 63 →
    /// 79 % while `▲` fell): the sent set's own share on the same seeds, under today's lineage
    /// (the kit, the facts) — `base + delta` is the active panel's share, the number the shaft
    /// shows when the forecast painted is the same pass (`sims`).
    #[serde(default)]
    pub base: f64,
}

/// Cut 22 §3: a depth's paired move (`D8 +6`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VsDepth {
    pub depth: u32,
    pub delta: f64,
    pub pm: f64,
    /// The absolute bar's own half-width at this depth (the active panel's `ForecastDepth.pm`).
    pub abs_pm: f64,
    /// QA on 912e135: the sent set's reach here on the same seeds (`VsMove.base`).
    #[serde(default)]
    pub base: f64,
}

/// Cut 22 §3: an edit's paired move against the previous set (`Game::forecast_vs`): per depth
/// of the shaft and on the ends (`vs last · D8 +6 · bank +4`), over `sims` paired seeds.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastVs {
    pub depths: Vec<VsDepth>,
    pub bank: VsMove,
    pub death: VsMove,
    #[serde(rename = "return")]
    pub return_: VsMove,
    pub gold: VsMove,
    /// QA on 912e135 (qaX: `pack break` added — `death −11` in green while `stall 11%` came up
    /// and D8 fell): the stall share's move (a timed-out send), so a death traded for a stall reads.
    #[serde(default)]
    pub stall: VsMove,
    pub sims: u32,
    /// QA on 778fa1b (qaU: `VS LAST · death −10` stayed from the first paint while the refined
    /// panel beside it read 22 → 27 %): the panels paired are the refined ones (the active
    /// set's camp panel was refined) — a vs read before the refine is `false`; ask again after.
    #[serde(default)]
    pub refined: bool,
}

/// Cut 21 §1: one start the camp's tablet offers (`Game::start_forecast`): D1 or a lit
/// waystone, measured for the active set on the camp's seeds (paired with the current
/// start's panel), like `CageOption`. `start` is the floor, `biome` its biome, `toll` what the
/// send pays for it (`$10 × start`, 0 at D1); `depth` the bar `reach` is read at (the set's
/// bank row's depth, else the best depth; a start at or past it reads 1.0); `gold` the mean
/// kept per send and `gold_delta` its move; `net` = `gold − toll` and `net_delta` its move;
/// `delta` the headline — `bank_delta` when either panel banks, else `reach_delta`. `short`:
/// the purse cannot pay the toll now — that send would start on D1 (the numbers are D1's).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StartOption {
    pub start: u32,
    pub current: bool,
    pub toll: i32,
    pub biome: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub short: bool,
    /// QA on a946e04: tonight's pass for this start is paid — the next send from it is free
    /// (`toll` is the next night's).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pass: bool,
    pub depth: u32,
    pub reach: f64,
    pub reach_delta: f64,
    pub bank: f64,
    pub bank_delta: f64,
    pub gold: f64,
    pub gold_delta: f64,
    pub net: f64,
    pub net_delta: f64,
    pub delta: f64,
    /// The 95 % half-width of `reach`.
    pub pm: f64,
    /// Cut 22 §4 (AG: `D9 · bank +3%` beside a 61 % death): the death share of a send from
    /// this start (0..1), and its move against the current start.
    #[serde(default)]
    pub death: f64,
    #[serde(default)]
    pub death_delta: f64,
    /// QA on 778fa1b: measured on the refined panels (the forecast's own sims once refined).
    #[serde(default)]
    pub refined: bool,
    /// Cut 23 §2: as `Forecast.low` — a `death` of 0 prints `<{low}%`.
    #[serde(default)]
    pub low: u32,
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
    /// The player's count: every hostile the hero saw from this action to the next (running
    /// thieves, foes given up on and the killer included; a sleeping den not). QA on 92eb880.
    pub foes: i32,
    /// What `foes>=` counted at this action (the rules' count; the patch candidates' packs).
    #[serde(default)]
    pub rule_foes: i32,
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
    /// QA on 0c6e126 (qaY: "the last row is never the killing blow — 05 ends at `hp 1`, 08 at
    /// five rows of `hp 3`"): the turns are the hero's actions, and the blow lands between two
    /// of them — a death's trace ends with it (the tick, what hit, the damage, `hp 0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blow: Option<TraceBlow>,
}

/// QA on 0c6e126: a death trace's last row — the blow that killed (`Trace.blow`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceBlow {
    pub t: u32,
    /// The killer's kind or the hazard (`goblin_archer`, `gas`, `poison`) — `Run.death_cause`.
    pub by: String,
    pub dmg: i32,
    /// Always 0: the hero's hp after the blow.
    pub hp: i32,
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
    /// QA on 23ed91f: on a death's shown patches, the depth whose camp bar `forecast_delta`
    /// moves (the death's depth + 1, at most the forecast's last) and that bar's 95 %
    /// half-width after the patch — the delta is the camp forecast's own (`trace::camp_deltas`).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub forecast_depth: u32,
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub forecast_pm: f64,
    /// QA on 23ed91f: `death()` answers before the camp's numbers are measured — while this
    /// is set, `forecast_delta` is the verdict's own 12-sim ranking estimate (not a camp
    /// number: do not show it as reach) and `forecast_depth`/`forecast_pm` are unset;
    /// `death_deltas(id)` returns the patches with the camp's numbers and this cleared.
    #[serde(default, skip_serializing_if = "is_false")]
    pub camp_pending: bool,
    /// Cut 19 §4: an insert onto a full set (own rows at `max_rows`) drops a row — this one
    /// (the set's index): the own row that fired least in the dead run (ties: the lowest in the
    /// list), so the screen reads `+ drop R5` and the drop sheet opens on it. `None` when the
    /// set has room, or the patch swaps or removes a row. The measured numbers are the
    /// inserted set's; `offline::apply_patch` drops this row.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drops: Option<i32>,
    /// QA on 778fa1b (qaU: `hp < 40% → return · survives 100%` ranked first; applied, `vs last
    /// · D5 −54 · death −94`): the patch's row ends the run (`return` / `bank`) — it survives
    /// the moment by going home, so its cost is floors: the client names it (`return early`)
    /// beside its reach. Set on every shown patch (`trace::mark_exits`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub exits: bool,
    /// QA on 0c6e126 (qaY: `hp < 20% → drink invisibility · survives 92%` applied, and the next
    /// heir carried none — `blocked · no item · none in pack`): a patch whose row uses a named
    /// item the next heir will not carry is offered with its purchase (the supply, its price;
    /// its reach measured with it bought — the tap buys it, then applies the row), or not at
    /// all (`trace::pack_need`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buys: Option<PatchBuy>,
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

fn is_zero_f64(n: &f64) -> bool {
    *n == 0.0
}

/// QA on 0c6e126: the supply a patch's row needs and the next heir will not carry (`Patch.buys`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PatchBuy {
    pub kind: String,
    pub label: String,
    pub price: i32,
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
    /// QA on 0c6e126 (qaY: `unpatched 58%` on two unrelated deaths — "a shared cache?"): the
    /// reseeded replays `survive` and `baseline` are shares of (`trace::REPLAYS`): 58 % is 7 of
    /// 12, a number many deaths share. The client prints the shares as counts (`7/12`).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub replays: u32,
    pub trace: Trace,
    pub patches: Vec<Patch>,
    pub morgue: String,
    /// Cut 6 §1: the death's ledger line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<ExitLine>,
    /// Cut 11 §2: the `because` entries of the killing turn's rows, in row order, deduped by
    /// text — the chain under the trace. Absent when no row had one. Cut 21 §3 (AF: why `hp <
    /// 30% → bank` never fired over six ticks): the killing turn's first, then the earlier
    /// turns' (newest first) — every tick's row reasons carry their own `because`
    /// (`TraceTurn.rows[].because`).
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
    /// QA on 92eb880 (qaM, the worst death: `DICE` above three patches all `survives 100% ·
    /// below bar`): a `dice` death whose unpatched rules already survive the replays as well as
    /// any candidate does — no shown patch survives more than `baseline` (a hero who won the
    /// same fight 12 of 12 times reseeded died to the rolls). The screen says nothing beats
    /// base (`base 100%`) rather than calling equal rows "below bar".
    #[serde(default)]
    pub nothing_beats_base: bool,
    /// Cut 19 §4: on a `row` verdict, the set's row (0-based) whose action was the dying one
    /// and whose removal survives the death's replays (`trace::ROW_BAR`): the verdict reads
    /// `row` and names it (`R2`). `None` on every other verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause_row: Option<u32>,
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
    /// QA on 0c6e126 (qaY): the kinds this absence found for the first time (wire names, in
    /// order) — the header's `new find` lines, all of them; FOUND lists them beside the vaulted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub new_finds: Vec<String>,
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
    /// QA on e75ec29 (qaR: a packed heal stolen on D1 in three runs, no report line): what
    /// thieves took this absence and no run got back, per label (the item's label at the
    /// theft: a flavour name while unidentified — `murky potion?`), most first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stolen: Vec<StolenRow>,
    /// QA on a946e04 (qaT): the waystone the absence could not pay the night's pass for, and
    /// the sends that went from D1 instead. Absent when every send started where chosen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_short: Option<StartShort>,
    /// QA on a946e04 (qaT): the carried gold the absence's kept thefts took (Σ `stolen[].gold`).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub stolen_gold: i32,
    /// QA on 778fa1b: the carried gold the absence's pack swaps took (Σ `exits[].swapped`).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub swapped: i32,
    /// Stall verdict (addition): present when the last ≥ 4 runs all came home with no new depth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stall: Option<Stall>,
    /// Cut 13 §1: sends that stalled (they are among `returned`, keeping nothing): the tiles
    /// count them apart — `1 RETURNED` for a stall read as a return to two QA players.
    #[serde(default)]
    pub stalled: u32,
    /// Cut 24 §1: sends a boss drove off (`no counter`; they are among `returned`) — the tiles
    /// can count them apart as they do the stalls.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub driven: u32,
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
    /// Cut 19 §3: the repeat skipped a supply because the absence's spending had reached what
    /// it brought home (`restock capped`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub restock_capped: bool,
    /// QA on 1a2a4a9: a re-pack of the absence ran short of gold and bought only what it could
    /// (`repeat short`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub repeat_short: bool,
    /// Cut 20 §5: the absence's bounty floor and whether a run brought it home (`bounty D12 ·
    /// missed` / `taken $412`); absent when the lineage had no bounty during the absence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<BountyReport>,
    /// Cut 21 §2: found supplies the absence's exits put on the shelf instead of salvaging,
    /// per kind — `gold` is their price on the shelf (what the repeat did not have to pay;
    /// `found heal ×12 → shelf`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shelved: Vec<SalvageRow>,
    /// QA on 912e135 (qaW: `♟18` over a report of ♟2–♟17, read as the heir who ran): the first
    /// and the last heir who ran these runs (`[2, 17]`; one heir `[5, 5]`); empty with no run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heirs: Vec<u32>,
}

/// Cut 20 §5: the bounty floor (`Lineage.bounty`): each night the lineage's best depth + 2 —
/// its gold ×2 and one item of the lineage's next tier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Bounty {
    pub depth: u32,
}

/// Cut 20 §5: the report's bounty — `taken` when a run reached the floor and came home
/// (bank or return); `gold` the kept share of the coins picked up on it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BountyReport {
    pub depth: u32,
    pub taken: bool,
    pub gold: i32,
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
    /// QA on e75ec29 (qaR: `+$40 heir purse` after the first death, nothing after three
    /// later ones): the heir purse's rule — each death tops the purse up to this much
    /// (`engine::WAKE_PAY`), never past it; a death with the purse at or over it pays
    /// nothing (its exit line reads `purse full`). `wake` is the absence's sum of top-ups.
    #[serde(default)]
    pub wake_cap: i32,
    /// How many of the absence's deaths topped the purse up (the rest found it full).
    #[serde(default)]
    pub wake_n: u32,
    /// QA on 912e135 (qaW: `STALLED $224 lost` and sixteen `$N carried` lines while the gold
    /// sheet named none of it): the carried gold the absence's exits did not keep (a death's,
    /// a stall's whole carry; a return's 40 %) — no movement of the purse, what it could have held.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub lost: i32,
}

/// QA on e75ec29: a kind thieves took and kept (`ReturnReport.stolen`): the label, how many.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StolenRow {
    pub label: String,
    pub n: u32,
    /// QA on a946e04 (qaT: the strip's `−$36 stolen`, never in STOLEN): the carried gold these
    /// thefts took off the runs (a stolen item's worth leaves the carry with it).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub gold: i32,
}

fn is_zero_i(n: &i32) -> bool {
    *n == 0
}

/// QA on a946e04 (qaT: `start → D5 · $50` with $32 — the absence's runs went from D1 unsaid):
/// the waystone whose night pass the purse could not pay, its toll, and the sends that went
/// from D1 instead (`ReturnReport.start_short`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartShort {
    pub depth: u32,
    pub toll: i32,
    pub runs: u32,
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
    /// QA on 92eb880: the XP the next level costs (`hero::xp_to_next`; 0 at the top), filled on
    /// the lineage the client reads so its bar and its sums use the core's ladder.
    #[serde(default)]
    pub next: u32,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
    /// What an unwatched exit keeps, in precedence order after the brought vault items (QA
    /// on 23ed91f): `["armour"]` for `best_armour`, `["armour", "weapon"]` with `quartermaster`,
    /// `[]` for `none`. Each keep may replace only a weaker vault item of its own category.
    #[serde(default)]
    pub keep_auto: Vec<String>,
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
    /// QA on 92eb880: the active set's shadowed rows, as `Forecast.shadowed_by` (per row by
    /// index, the earlier row that takes all its moments; empty when none is shadowed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shadowed_by: Vec<Option<u32>>,
    /// Cut 16 §2: the classes the new heir may wake as — the owned ones, the current first —
    /// while the wake is open (`trait_offer` non-empty) and ≥ 2 are owned; `set_class(name)`
    /// picks (it sticks to every run until changed).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub class_offer: Vec<ClassChip>,
    /// Cut 19 §3: the loadout repeats — `true` unless the player cleared it
    /// (`setRestock(false)`); the kinds the next send re-packs (the last send's, less the ones
    /// it wasted) and their price on the shelf today (`repeat · $120`).
    #[serde(default)]
    pub repeat: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_kinds: Vec<String>,
    #[serde(default)]
    pub repeat_gold: i32,
    /// QA on 1a2a4a9: the kinds the last re-pack could not pay for (`repeat short`; the ledger
    /// has a `$0 repeat short` line at that exit). Empty once a re-pack paid for everything.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_short: Vec<String>,
    /// QA on 778fa1b (qaU: the tile's `repeat on · $26` over a `1/3` shelf the absence's
    /// capped re-pack left empty): of the repeat's kinds, the ones the shelf lacks that the
    /// next send's re-pack buys at the send (`+heal at send`), and the ones it will not — the
    /// purse (after the start's toll) or the shelf's cap will not let it: the tile reads
    /// `repeat short` (`Game::repeat_at_send`). Both empty when the shelf holds the repeat.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_due: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_unpaid: Vec<String>,
    /// Cut 20 §5: tonight's bounty floor (set at each night's end: best depth + 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<Bounty>,
    /// Cut 21 §1: the lit waystones (biome first floors: D5 D9 D14 D19 D24 D29), ascending —
    /// each lit by a bank from a floor at or past it; the start tablet offers D1 and these.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waystones: Vec<u32>,
    /// Cut 21 §1: the floor the next send starts on (`setStart`; 1 by default) and the toll it
    /// pays at the send (0: QA on 778fa1b made a waystone start free; the field stays).
    #[serde(default = "one")]
    pub start: u32,
    #[serde(default)]
    pub start_toll: i32,
    /// Cut 21 §2: kinds the last send packed that no row of the active set can use — the
    /// repeat does not buy them again (`strength · no row`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_dropped: Vec<String>,
    /// QA on a946e04 (qaS: `cowardly · flees under 50%` never fled): each trait's real rule,
    /// by name (`Trait::rule`: `cowardly` → `backs off once a floor under 50%`) — the chip's
    /// words; a trait overrides a row at most once per floor.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub trait_rules: std::collections::BTreeMap<String, String>,
    /// QA on a946e04 (qaS: SALVAGED `blue potion? ×8` beside `poison ×3` after the night had
    /// identified poison): each identified flavoured kind's unidentified label (`blue potion?`)
    /// → its name now (`poison`, as `LineageState::wire_name`). A label computed before the
    /// kind was known (an earlier slice's report, a stored line) reads through this map.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub renamed: std::collections::BTreeMap<String, String>,
    /// QA on a946e04 (qaT: `start → D5 · $50` read as the start with $32 in the purse; the send
    /// went from D1): whether the next send starts on `start` — D1, or a lit waystone whose
    /// night pass is held (`start_pass`) or the purse pays now (`start_toll`). False: the send
    /// goes from D1 (`ExitLine.start_short` says so after).
    #[serde(default = "yes")]
    pub start_payable: bool,
    /// QA on a946e04: tonight's pass for `start` is paid — the toll is not charged again
    /// until the night ends (a night is `NIGHT_RUNS` runs).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub start_pass: bool,
    /// Cut 23 §1: the forge — the heir's starting kit, a ladder per slot (`meta::kit_ladders`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kit: Vec<KitLadder>,
    /// Cut 23 §3: per row of the active set (by index), its why-not over the recent sends
    /// (`LineageState::row_stats`); `null` before any send under that row.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_why: Vec<Option<RowStat>>,
    /// Hero looks: the heir's cosmetic look (`male | female | cat`; the class's own until
    /// `setLook`); sprites and portraits are `hero_<class>_<look>`.
    #[serde(default)]
    pub look: String,
}

fn yes() -> bool {
    true
}

fn one() -> u32 {
    1
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
    /// Cut 18 §5: a tactic card's situation — the foe tag its row answers (`kite_archers` →
    /// `ranged`, `gas_step` → `gas`; `meta::card_situation`), so a `reach ~0` card still says
    /// when it matters (`vs archers`). Absent for cards keyed on no foe and other unlocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub situation: Option<String>,
    /// QA on 92eb880 (qaN: a bought card raised the stall share to 35 %): the stall share's move
    /// at the card's best place, from the same sims as `delta` (0..1, signed) — the stall risk
    /// shown before buying. A best place never raises it more than `meta::CARD_STALL_RISE`
    /// unless every place does.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stall: Option<f64>,
    /// Cut 19 §3 (rater AA: "no `+1 row` offered any more"): the next `+1 row` — the row
    /// unlock whose prerequisite is owned — stays on the camp's short list until bought
    /// (the client's top-three cut had dropped it behind cheaper cards).
    #[serde(default, skip_serializing_if = "is_false")]
    pub pinned: bool,
    /// QA on 1a2a4a9: on the short list (`meta::SHORT_LIST` cards, the pinned one included) —
    /// the core's choice from the lineage alone, so two opens of an unchanged camp, the
    /// report's PENDING and the camp's UNLOCKS show the same cards (`meta::mark_short`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub short: bool,
    /// QA on e75ec29 (qaR: eight cards bought, eight rows inserted — 14 rows, stall 40 %, D7
    /// 76 % → 44 %): a tactic card's buy puts its row in the set at `insert_at` only when this
    /// is set — measured (`delta`, `stall`), its best place's reach not down (`delta ≥ 0`), its
    /// stall share up by ≤ `meta::CARD_STALL_RISE`, and fewer than `meta::AUTO_CARDS` card rows
    /// in the set already (`meta::auto_insert`). Otherwise the card is owned, not in the set
    /// (the client offers `add` on the card).
    #[serde(default, skip_serializing_if = "is_false")]
    pub auto_insert: bool,
    /// QA on a946e04 (qaS: `+1 ROW ◆2 or $300 … each $ buy +25%`, then `◆4 or $600` after a ◆
    /// buy): the chain's next unlock once this one is owned (`row5` → `row6`) — its own price,
    /// not this one's raised — with its marks, its gold price after a ◆ buy of this one (the
    /// gold buys unchanged) and after a $ buy (one more gold buy: `meta::gold_price`). This
    /// card's own gold price after any other $ buy is `gold_next`. Absent at a chain's end.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NextUnlock>,
    /// This card's gold price once one more gold buy has been made (`× (5 + gold_buys) / 4`);
    /// 0 when not gold-buyable.
    #[serde(default)]
    pub gold_next: u32,
    /// Cut 23 §3 (AJ: "paid cards are rows he could type"): what a paid card holds that no
    /// typed row can (`meta::card_carries`: `four rows, one slot`, `den raid first`), ≤ 4
    /// words. Absent on every other unlock.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub carries: Option<String>,
}

/// Cut 23 §1: one step of a forge ladder (`sword +1`, `mail`, `pack 4`) and its gold price.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitStep {
    pub label: String,
    pub price: u32,
    pub owned: bool,
}

/// Cut 23 §1: a ladder's next step — its price, whether the purse pays it now, the nights of
/// income until it does (`None` without an income to go on), and with `kitDeltas` its paired
/// forecast move at `depth` (`delta` ± `pm`, 0..1) and the ends' moves.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct KitNext {
    pub label: String,
    pub price: u32,
    pub affordable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nights: Option<u32>,
    /// QA on 912e135 (qaW: `7 nights` with no income on screen): the night's net the estimate
    /// divides by (`kit::nights`), shown beside it (`7 nights · $44/night`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub per_night: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pm: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bank: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death: Option<f64>,
}

/// Cut 23 §1: the forge — one slot of the heir's starting kit (`weapon | armour | pack`), its
/// ladder, the steps owned, and the next step.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct KitLadder {
    pub slot: String,
    pub owned: u32,
    pub steps: Vec<KitStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<KitNext>,
}

/// Cut 23 §3: a count behind a reason (`RowStat.blocked` / `unmet`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WhyCount {
    pub why: String,
    pub n: u32,
}

/// Cut 23 §3 (AJ: `foe: gas → throw unknown` fired 0/164, no word): a row's why-not over the
/// recent sends it sat in (`LineageState::row_stats`): the sends and the hero's actions over
/// them, the actions it took, the actions its conds held; the most common reason it did not
/// act when reached with its conds holding (`blocked`), and its most common failing cond
/// (`unmet`: `gas` for `foe: gas` never in view). `text` is the tablet's line.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RowStat {
    pub sends: u32,
    pub actions: u32,
    pub fired: u32,
    pub matched: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked: Option<WhyCount>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unmet: Option<WhyCount>,
    pub text: String,
}

/// QA on a946e04: the chain's next unlock and its prices (`UnlockInfo.next`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NextUnlock {
    pub id: String,
    pub cost: u32,
    /// Its gold price after this one is bought with marks.
    pub gold: u32,
    /// Its gold price after this one is bought with gold.
    pub gold_after_gold: u32,
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
