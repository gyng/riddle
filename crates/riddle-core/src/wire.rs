//! Wire types mirroring `web/src/engine/types.ts` (snake_case JSON).
use crate::item::{FloorItemWire, InvItem};
use crate::rules::{Row, RuleSet, Verb};
use crate::tiles::{Overlay, OverlayKind, Tile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<crate::endgame::Modifiers>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gun: Option<GunSnap>,
    #[serde(flatten)]
    pub entity: Entity,
    pub inv: Vec<InvItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armour: Option<String>,
    pub class: String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub specialization:Option<crate::specialization::Style>,
    #[serde(rename = "trait")]
    pub trait_: String,
}

/// The actual weapon/chambers at this snapshot, independent of later camp gear.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GunSnap {
    pub item: u32,
    pub kind: String,
    pub loaded: u8,
    pub capacity: u8,
    pub range: i32,
    pub damage: (i32, i32),
    pub armour_piercing: i32,
    pub reload_ticks: u32,
    pub reload_left: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reload_until: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub aiming: bool,
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
    /// Cut 27 §1: the passage this run was paid at its waystone start (`+$84 passage`: the
    /// skipped floors' gold, into the purse at the send — ledger `passage D9` — when the set
    /// clears them ≥ 95 %);
    /// 0 from D1 or when the set does not clear the floors above its start.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub passage: i32,
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    /// Cut 118 §5: this floor is swift (its band boss cleared `feats::SWIFT_CLEARS` times): the watch plays it as one beat.
    #[serde(default, skip_serializing_if = "is_false")]
    pub swift: bool,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub difficulty: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifier_catalogue: Vec<crate::endgame::ModifierInfo>,
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
    /// Take control: the player has the watched run (`manual`), and the world waits for his action (`awaiting`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub manual: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub awaiting: bool,
    /// Blind b58b431: in hand control, how the last order resolved (`moved`, `hit`, `can't · wall`, `paralysed · 3`,
    /// `foe down`, `hp under 30%`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order: Option<String>,
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
    /// Cut 26 §2: this floor's down stairs are a fork's (D4's, on the base order): the stair the
    /// run's route takes is the floor's own down stairs (`taken`, its biome); the other stair
    /// (`other`) is drawn at (`x`, `y`) beside it. The hero never takes it — the set's route
    /// decides; the callout is `TWO STAIRS`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fork: Option<SnapFork>,
    /// Cut 29 §3: the watch's compact meter — the run so far and the fight in progress (or the
    /// last), `fighting` while one is. Absent before the first tick.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meters: Option<SnapMeters>,
}

/// Cut 26 §2: a fork on the floor (`Snapshot.fork`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SnapFork {
    pub depth: u32,
    pub taken: String,
    pub other: String,
    pub x: i32,
    pub y: i32,
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
fn first_bloodline()->u32 {1}
fn solo_bloodline(id:&u32)->bool{*id<=1}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExitLine {
    #[serde(default="first_bloodline", skip_serializing_if="solo_bloodline")]
    pub bloodline_id:u32,
    /// Cut 30.5 (the owner: a new record is a checkpoint): of `carried`, the gold this run's checkpoints secured
    /// (kept whole at any exit, a death's included); `keep_pct` is the share of the rest.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub secured: i32,
    pub carried: i32,
    pub keep_pct: i32,
    pub kept: i32,
    /// This run's bounded progression beats, also carried by watched exits.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
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
    pub legacy_earned: u32,
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
    /// c30-legible (the owner, a new player: "I didn't understand … why the run ended early at like D3"):
    /// why the run ended, ≤ 3 words, for the watch's end beat, the report and the town's returning party
    /// (`banks every record`, `hurt · went home`, `slain · jackal`, `stuck · went home`, `repelled · Warlord`;
    /// `engine::exit_reason`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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
    /// Cut 29 §3: the run metered (`meters::MeterWire`); absent on a sim's line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meters: Option<Box<crate::meters::MeterWire>>,
    /// Run-clear (the owner, 2026-10-02: "each run should have the clear screen"): the end's kind — `bank`,
    /// `return` or `death` (the exit event's `tier`, on the line so a report's last run can be stamped).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub end: String,
    /// Run-clear: the deepest floor the run reached.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub reached: u32,
    /// Run-clear: the run went past the lineage's record (a `new best` badge on the card).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub new_best: bool,
    /// Run-clear: the items this run found that it brought home (or, on a death, left in its bones),
    /// rarest first, ≤ `FINDS_SHOWN` — each with its rarity (`item::rarity`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finds: Vec<InvItem>,
}

/// Run-clear: how many finds an exit line carries (the card shows them all, a row of six).
pub const FINDS_SHOWN: usize = 6;

/// Cut 24 §2: one line of `ExitLine.news` — its kind (`first` · `record` · `named` · `find`
/// · `situation` · `driven` · `learned` · `differ`) and its text (≤ 6 words, lower case).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct News {
    pub k: String,
    pub text: String,
    /// Cut 118 §8: the lineage day (1-based) it happened on.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub day: Option<u32>,
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
    /// Cut 26 §6 (AP: a drive-off at 25/36 hp, `driven $0 · $297 lost`, no verdict screen): the
    /// run, so a report's drive-off opens its verdict (`ReturnReport.drives`) as a death does;
    /// `hp`/`max_hp` the hero's at the drive-off, `lost` the carry it did not keep.
    #[serde(default)]
    pub run_id: u32,
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default)]
    pub lost: i32,
    /// Cut 27 §5 (AT: `D8 · counter unwritten` offered `foe: boss → attack boss at R7` — his own
    /// R7, pre-empted by the archers' rows above it): the set's row (0-based) that already
    /// carries the counter's verb; `verdict` then reads `order` (not `no counter`) and the fix is a
    /// move, not a new row. `None` when the counter is unwritten.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub held: Option<u32>,
    /// Cut 27 §5: on an `order` drive-off, the row above `held` that acted most in the boss fight
    /// (the run's last actions) — the move puts `held` above it (`R7 under R2` → move R7 above R2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub over: Option<u32>,
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
    #[serde(default="first_bloodline", skip_serializing_if="solo_bloodline")]
    pub bloodline_id:u32,
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
    /// Cut 122 §6: whether the row passes the set's door now (`owned`; else `refusal`, the door's words, and
    /// `wear`, the package whose wearing owns its card).
    #[serde(default = "yes")]
    pub owned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear: Option<String>,
}

/// Cut 122 §6 (blind 9621b19 A: an added `cadence` counter shown as added, refused by `setRules` with `card not
/// owned: cadence`): whether a row offered for the set passes `setRules`' door now. `owned` false: `refusal` is the
/// door's own words (`card not owned: cadence`) and `wear` the package that owns its card once worn (`cadence`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ownership {
    pub owned: bool,
    pub refusal: Option<String>,
    pub wear: Option<String>,
}
impl Default for Ownership {
    fn default() -> Self { Ownership { owned: true, refusal: None, wear: None } }
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

/// Bounded hostile recovery labels; avoids allocating a String per hit.
#[derive(Clone,Copy,Debug,Serialize,Deserialize,PartialEq,Eq)]
#[serde(rename_all="snake_case")]
pub enum RecoverySource { Leeching }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Ev {
    /// Played equipment/chambers only when changed; null means the gun was stowed/lost.
    Gun { t:u32, state:Option<Box<GunSnap>> },
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
    Spawn { t: u32, e: Box<Entity> },
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
        trace: Option<Box<Trace>>,
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
    /// Cut 122 §7 (blind 9621b19 B: `Heading home` ~2 min of watch): the walk home committed (a `return` or `bank`
    /// row acted) — `ticks` the walk's length foreseen (a bank's path to the up-stairs at his step; a return's, at most
    /// its `RETURN_TICKS`). The watch may fold a long walk into one summary beat (`walk home ·
    /// 14 s`) and resume at the next fight or the exit.
    Homeward { t: u32, ticks: u32, bank: bool },
    /// QA on 1a2a4a9 (qaP: `12/38 → 6/16 → 3/16 → 0/15` on D12 with no line until the death):
    /// the hero's max HP moved — `delta` (−1 per hunger bite on an unlit hunger floor), `max`
    /// after it, `cause` (`hunger`). The callout beside it reads `hunger −1 max`.
    MaxHp { t: u32, id: u32, max: i32, delta: i32, cause: String },
    /// Cut 25 §3 (AN: ~55 s of max hp draining 41 → 17 on D10 with only numbers moving): a drain
    /// stretch starts — the hero's hp or max hp falls with no foe in view (a hunger bite, poison).
    /// `cause` is one word (`starving`, `poisoned`, `drained`), sent once per stretch; until a foe
    /// comes into view, the stairs or the run's end, the hero's `hurt` / `max_hp` events with no
    /// foe in view are the drain's: dead time (the watch plays them at the travel rate).
    Drain { t: u32, cause: String },
    /// Cut 28b (owner: "it's not clear what oaths do"): the sworn oath's fate in this send, said
    /// once, as it happens — `kept` (a new kind tamed, the boss slain, the floor reached and the
    /// run ended by a bank or a death), or not: broken (`cause` the tool it forbade: `return`,
    /// `rest`; or `stalled` · `driven`) or missed at the end (`cause` empty: the run ended short of
    /// it). `row` is the row whose verb did it (−1: a chore, a trait, the run's end). The watch's
    /// beat reads `OATH KEPT` / `OATH BROKEN · R2 return`; the run's notes and exit line say the same.
    Oath { t: u32, kept: bool, row: i32, cause: String },
    /// Cut 29 §3: hp the hero (`id` 0) or a pet regained this tick, by source (`potion` · `rest` ·
    /// `regen` · `skill`); read off the hp around the tick for the meters (`meters.rs`) — not
    /// renderable, no input to anything.
    Heal { t: u32, id: u32, amount: i32, src: String },
    /// Actual hostile recovery, distinct from hero/pet healing meters.
    Recover { t:u32, id:u32, amount:i32, hp:i32, src:RecoverySource },
}

impl Ev {
    pub fn t(&self) -> u32 {
        match self {
            Ev::Gun { t, .. }
            | Ev::Move { t, .. }
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
            | Ev::Ending { t, .. }
            | Ev::Homeward { t, .. }
            | Ev::Drain { t, .. }
            | Ev::Oath { t, .. }
            | Ev::Heal { t, .. }
            | Ev::Recover { t, .. } => *t,
        }
    }
    /// Renderable, non-movement events (the "events per 60 turns" gate).
    pub fn renderable(&self) -> bool {
        !matches!(self, Ev::Gun { .. } | Ev::Move { .. } | Ev::Rule { .. } | Ev::Fact { .. } | Ev::Note { .. } | Ev::Rest { .. } | Ev::Ending { .. } | Ev::Homeward { .. } | Ev::Oath { .. } | Ev::Heal { .. })
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
    /// Cut 29 §4: the sheet is a decision — a find beats something in the full vault (or finds
    /// outnumber its free slots). False: the client settles it (`autoKeep()`) and shows `note`.
    #[serde(default)]
    pub decide: bool,
    /// Cut 29 §4: the settled exit's one line (`kept leather +1`); absent when nothing new is kept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StepResult {
    pub events: Vec<Ev>,
    pub snapshot: Snapshot,
    pub run_over: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_pending: Option<ExitPending>,
    /// Cut 28 §3 (AU: ~50 s of `pick up ×N` at 1×): the step's calm stretches — `[from, to]` run
    /// ticks (inclusive, `Ev::t`) with no decision and no threat (`fold::calm_tick`): the watch plays
    /// them at the travel rate in every mode. Empty when the step had none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calm: Vec<[u32; 2]>,
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
    /// Cut 116 §1: the boss's affix for the live heir (`armoured`), beside `boss` (`D8 · warlord · armoured`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affix: Option<String>,
    /// Cut 26 §2: on a set that writes a route, the biome this floor sits in on it (`fens` at D5
    /// for `route: [5]`); absent on the base order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<String>,
    /// Cut 27 §1: the share of the sims on this floor that got through it (reached the next
    /// floor) — a floor at ≥ `forecast::FOLD_CLEAR` (95 %) from the start down is folded on the
    /// watch (`Forecast.fold_to`). Absent where no sim stood on the floor.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear: Option<f64>,
}

/// Cut 116 §1: a band boss's affix for the live heir.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BossAffix {
    pub boss: String,
    pub depth: u32,
    pub affix: String,
    pub effect: String,
    pub counter: String,
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
    /// Cut 122 §6: as `Counter.owned` / `refusal` / `wear` (the row, inserted as the pen inserts it).
    #[serde(default = "yes")]
    pub owned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear: Option<String>,
    /// Cut 122 §2: the try priced at its wall (the boss's floor) by the paired forecast; `trade_off` when it is worse
    /// there past the noise. Absent when not measured (a wall no sim reaches).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<WallPrice>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub trade_off: bool,
}

/// Cut 122 §5: the hunger priced on a forecast's panel (`Forecast.hunger`); means per send, in max hp.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct HungerForecast {
    pub lost: f64,
    pub unfed: f64,
    pub fed: f64,
    pub kept: f64,
    pub packed: bool,
    pub price: i32,
}

/// Cut 122 §2 (blind 9621b19 A: `Mirror rhythm` took the forecast's D33 50→0%, `gas step · wade in` D18 88→38%):
/// a suggestion priced at the wall it answers — paired sims of the camp with and without it, the share of sends
/// past the wall (`past_*`, reach of `depth + 1`) and the death share (`death_*`), each with its 95 % half-width (0..1).
/// `trade_off`: worse past the noise on either (`past` down, or `death` up, beyond its ±) — show the numbers, never
/// as a fix.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct WallPrice {
    pub depth: u32,
    pub past_from: f64,
    pub past_to: f64,
    pub past_pm: f64,
    pub death_from: f64,
    pub death_to: f64,
    pub death_pm: f64,
    pub sims: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub trade_off: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastCause {
    pub cause: String,
    pub share: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Forecast {
    /// Cut 117 §1 (blind 8cf9050 B: `D33 88%`, then the scout banked the sends at D30/D32): the scout's order
    /// at a wall rides with the next send — the floor it stops before, and the send's ends under it. The
    /// depths' `reach` stays the reach *if pushed* (what the rules can do); `hold.ends` is what the next
    /// send will do. Absent with no scout, no wall or the order `push`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hold: Option<ForecastHold>,
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
    /// Cut 27 §1: the last floor the watch folds for this set — every floor from `start` to it
    /// clears ≥ 95 % on this panel (`ForecastDepth.clear`) and is known (≤ the best depth).
    /// Absent when the start floor itself is below the bar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fold_to: Option<u32>,
    /// Cut 28 §1: the sworn oath priced on this panel (`oath · D10 no drink · 34%`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<OathShare>,
    /// Cut 122 §5 (blind 9621b19: `starving −36` with no answer): the hunger on this panel — the max hp a send loses to
    /// it as the camp stands (`lost`), unfed and with a ration (`unfed`, `fed`: mean per send), the hp a ration keeps
    /// (`kept` = unfed − fed) and whether the next send already packs one (`packed`). Absent when no sim met the hunger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hunger: Option<HungerForecast>,
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
    /// Cut 28 §1: the sworn oath's share moved by the edit (the same paired seeds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<VsMove>,
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
    /// Cut 27 §1: the passage a send from this start is paid (`+$84 passage`, into the purse at
    /// the send; 0 from D1, or when the set does not clear the floors above it ≥ 95 %). `gold`
    /// and `net` count it (a send's gold is what it brings home and what its start paid).
    #[serde(default)]
    pub passage: i32,
}

/// Cut 12 §3: how a send ends — `bank` / `return` / `death` as shares of a panel of sends run
/// to their exit (the bars' own panel — QA on 3d71c33; a run at the cap is a stall) and
/// `gold`, the mean loot kept per send by the exit's own share (bank 100% · return 60% · death 0%).
/// Cut 117 §1: the scout's wall order on the forecast (`Forecast.hold`): the wall's floor `depth`, the order
/// (`bank`: the send banks at D`stop`'s stairs, `stop` = `depth − 1`; `carry`: its haul is carried home from
/// there and it goes on), `share` the sends that get to those stairs, and the panel's ends with the order
/// applied (a `bank` send that reached the stairs banks there, its carry then; a `carry` keeps at least it).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastHold {
    pub depth: u32,
    pub stop: u32,
    pub order: String,
    pub share: f64,
    pub ends: ForecastEnds,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
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
    /// Cut 29 §6 (AX: `$81` banked under `~$260`): of `gold`, the waystone passage paid at the send
    /// (the exit line's `BANKED $N` is `gold − passage`); 0 from D1.
    #[serde(default)]
    pub passage: f64,
    /// Cut 121 §2 (blind 1a7d834 A: `death 75%` on an easy clear; per-run `$` that swung on one threshold): the
    /// bands the panel's numbers lie in — the death share's 95 % Wilson interval (never `0` or `1` on a few sims)
    /// and the mean gold per send's 95 % interval (mean ± 1.96 · sd / √n, never below 0). The client shows ranges.
    #[serde(default)]
    pub death_lo: f64,
    #[serde(default)]
    pub death_hi: f64,
    #[serde(default)]
    pub gold_lo: f64,
    #[serde(default)]
    pub gold_hi: f64,
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
    /// QA on 524827b (qaAB: `8002 R1 return 12 · 8013 R1 return 6` — hp fell with no row): the
    /// blows on the hero since the previous action, before this one (oldest first, the hp after
    /// each) — the table's rows between two actions. Blows during this action's tick roll to the
    /// next action's (the last action's are `Trace.blows`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blows: Vec<TraceBlow>,
    /// Cut 28 §2 (AV: `R1 hp not <30%` at 6 hp — his max drained 44 → 29, not shown): the hero's
    /// max hp at this action.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub max_hp: i32,
    /// Cut 30 §4: the heir's gift that acted at this action (`fury +1`, `mend +1 · sure 50%`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gift: Option<String>,
}

/// Cut 28 §2: one step of the hero's max hp (`Trace.max_steps`): the tick, the max after it, the
/// move and its cause (`hunger`, `drain`, `shrine`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaxStep {
    pub t: u32,
    pub max: i32,
    pub delta: i32,
    pub cause: String,
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
    /// Cut 25 §3 (AN: `14 → 0` on one `goblin −2` row — the tick's other blows not itemised): a
    /// death's every blow after the last action, oldest first, each with the hp after it; the last
    /// is `blow`. Empty when `blow` was the only one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blows: Vec<TraceBlow>,
    /// QA on 524827b (qaAA: the trace starts at 1 hp exploring — "where 36 hp went takes a
    /// replay"): a death's hp lost since the hero was last at full hp, per cause, most first
    /// (`jackal −24 · monkey −8`). Empty on any other exit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hp_lost: Vec<HpLoss>,
    /// QA on 308f045 (qaAC: `hp_lost` summing to twice his max): the hp healed over the same
    /// stretch (heals, rest, regeneration) — `Σ hp_lost − hp_healed` is the hp he began it with.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub hp_healed: i32,
    /// Cut 28 §2: the hero's max-hp steps from the trace's first turn to its end, oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub max_steps: Vec<MaxStep>,
}

/// QA on 524827b: one cause's share of a death's hp lost since full (`Trace.hp_lost`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct HpLoss {
    pub by: String,
    pub dmg: i32,
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
    /// Cut 25 §2 (AM: his `hp < 30% → return` sat under `foes ≥ 1 → attack nearest` all run and
    /// never fired — stamped DICE): a move — the set's own row at `moves_from` (0-based) goes
    /// above the row at `insert_at` (`row` echoes it; nothing is added or cut). The client reads
    /// `move R5 above R2`; `offline::apply_patch` moves it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moves_from: Option<i32>,
    /// QA on 524827b (qaAA: `hp < 20% → drink unknown · survives 12/12` led, and the camp's
    /// killers then read fire 28 % · poison 26 %; `read unknown · 11/12` went in as R1 and the
    /// camp read `death +6`): the patch judged on whole runs — the camp panel's paired move
    /// once applied (`trace::camp_deltas`, set with the camp's numbers; `None` while
    /// `camp_pending`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whole: Option<PatchWhole>,
    /// Cut 27 §4: the gem's patch — set by `death_deltas` (never while `camp_pending`) on the
    /// best whole-run patch that does not harm (`trace::pick_gem`), which is then the first
    /// shown; on no patch when none may be the gem (the gem reads `edit`).
    #[serde(default, skip_serializing_if = "is_false")]
    pub gem: bool,
    /// Cut 27 §5 (AT: the bloat row deleted, a gas death stamped DICE): the patch puts back a row
    /// the set sent before this one held, at its old place (0-based; `insert_at` is the same) —
    /// the client reads `restore R4`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restores: Option<u32>,
    /// Cut 28 §2 (AU: `survives 0/12 · no gain` beside a green `reach D14 0→24%`): the patch
    /// survives no more of the death's replays than the unpatched rules — the client prints no
    /// green move beside it.
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_gain: bool,
}

/// QA on 524827b: a patch's whole-run move on the camp's panel (the same seeds, the same
/// lineage state as the camp after the tap): the reach at `Patch.forecast_depth` and the
/// death share, each a paired mean difference with its 95 % half-width (`forecast::paired`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PatchWhole {
    pub reach: f64,
    pub reach_pm: f64,
    /// Cut 26 §6 (AP: `survives 12/12 · reach D5 −76`, unreadable): the reach at the bar before
    /// and after the patch (0..1) — the screen prints the move as from→to (`reach D5 90→14%`).
    #[serde(default)]
    pub reach_from: f64,
    #[serde(default)]
    pub reach_to: f64,
    pub death: f64,
    pub death_pm: f64,
    /// Worse than the set as it is beyond its ± — death up past `death_pm` or reach down past
    /// `reach_pm` (`trace::whole_harms`): never the lead, never the gem's default.
    #[serde(default, skip_serializing_if = "is_false")]
    pub harms: bool,
    /// The self-dealt harm (`fire`, `poison`, `gas`) whose deaths the patch raises by two sims
    /// or more on the panel — the gamble the moment's replays did not show (`risk fire`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    /// QA on 308f045 (qaAC: `reach D9 ≈ ±1` on the patch, `vs sent · D6 −21` on the camp after
    /// it): the floor the reach is read at — the camp's `vs sent` head (`trace::whole_move_on`);
    /// `Patch.forecast_depth` is the same floor.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub depth: u32,
    /// QA on 308f045 (qaAC: `death −100 ±1` — "points? percent of runs?"): the death share
    /// before the patch (0..1) — the screen prints the move as from→to (`death 100→0%`).
    #[serde(default)]
    pub death_from: f64,
    /// Cut 122 §2 (blind 9621b19: fixes that made the game's own forecast worse): the patch priced at the wall it
    /// answers (the death's floor, the stall's) — `trade_off` when it is worse there past the noise (sends past the
    /// wall down, or deaths up, beyond the ±): never a fix, never the gem; show the numbers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<WallPrice>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub trade_off: bool,
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

/// Identity captured with the run's verdict, independent of the current heir.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeathHero {
    pub name: String,
    pub bloodline_id: u32,
    pub heir: u32,
    pub class: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Death {
    /// Cut 118 (owner amendment): the death screen's lead — the siege's progress (`the Queen · try 4 · +16%`), the
    /// epitaph, the grave's gold, the Legacy his deeds paid (`feats::memorial`; filled by `Game::death`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memorial: Option<crate::feats::Memorial>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub difficulty: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub modifier_catalogue: Vec<crate::endgame::ModifierInfo>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modifiers: Option<crate::endgame::Modifiers>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hero: Option<DeathHero>,
    /// Cut 30 §2: the package row that last acted (the verdict's row when it names one), as
    /// `package · row` (`Steady · HP<20% → return`, `drill · Warlord · boss → hit boss`); absent when the
    /// last row to act was the pen's or a chore.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Cut 30 §2: before the pen opens, the death screen's one cheapest lever — `spend` (a blacksmith
    /// step the purse pays: `sword +2`), `package` (a stance that answers the killer: `Hunter`) or `wait`
    /// (the drill or the scars will come: `Warlord · scarred ×3`). Absent once the pen is open.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lever: Option<Lever>,
    /// Cut 115 §4: the tactic (and variant) that answers this death, when one has arrived — offered before a
    /// raw row (`try: gas step · burn`); taken, its rows are credited `taught`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<Lever>,
    /// Cut 115 §1: who chose the deciding row (`picked` · `taught` · `default` · `chores`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub credit: Option<String>,
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
    /// Cut 25 §2: on an `order` verdict, the row (0-based) that won every tick the cause row
    /// (`cause_row`) would have acted on — the verdict reads `R5 under R2` (`cause_row` 4,
    /// `order_over` 1), and the lead patch moves R5 above R2. `None` on every other verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_over: Option<u32>,
    /// Cut 26 (risks: attribution): on a `route` verdict, the far stair the set's route took and
    /// the near one whose sends survive the death's floor (`D5 fens · route`, the fix `take
    /// burrows`: `route` is the set's route with that stair). `None` on every other verdict.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_cause: Option<RouteCause>,
    /// Cut 26 §6 (AO: `GAP` beside `unpatched 10/12` — "fault or luck?"): `dice` on a `gap`, `row`
    /// or `order` whose unpatched replays mostly survive (over `trace::STAMP_BASE`) — the stamp
    /// reads it beside the counts (`GAP · dice-leaning`), so it never contradicts them. `None`
    /// otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lean: Option<String>,
    /// Cut 28 §2 (AU/AV: `7–11/12 live unpatched` banners read as blame): on a death most replays
    /// survive (`baseline` over `trace::STAMP_BASE`), the rare event that killed him and its odds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub luck: Option<DeathLuck>,
    /// Blind 3ab97ea (A: 48→0 in 10 s to the Mirror King's mirrored blows read `bad luck · 1 in 6`): the
    /// boss this death was fought under. A boss's fall is his wall — never luck, nor dice-leaning: his
    /// counter, his floor's fixes answer it, however many reseeded replays live.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
    /// Cut 29 §3: the fight he died in, metered (`meters::MeterWire`: dps dealt and taken by side,
    /// hps by source, hits taken, the rows that fired); absent for a stall or a sim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fight: Option<crate::meters::MeterWire>,
    /// Cut 117 §1 (blind 8cf9050 A: a King death's header `hero at 18 hp` over a trace ending `0/50`):
    /// the hp the hero stood at when the killing blow landed — the trace's row before that blow (the
    /// previous blow's hp, else the last action's). The client read `blow.hp + blow.dmg`, which counts
    /// an overkill (an 18 blow on 5 hp read `at 18 hp`). Absent on a stall.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moment_hp: Option<i32>,
    /// Cut 117 §1: the hero's max hp at that moment (the trace's `hp/max`).
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub moment_max_hp: i32,
    /// Cut 121 §2 (blind 1a7d834 B: `goblin · D8` beside `fell to the Warlord`): the summoner of the foe whose blow
    /// killed him (`Warlord`, `goblin captain`): the header reads `goblin · summoned by Warlord`. Absent when the
    /// killer was nobody's summons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summoned_by: Option<String>,
    /// Cut 121 §2 (B: `fell to the Mother` while the trace said `not in view`): a hazard's (gas, fire, poison,
    /// burst) source boss, only when he was in view at the killing blow (`Mother`); absent otherwise — the hazard
    /// is the cause.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Cut 28 §2: a luck-leaning death's event (`Death.luck`): `text` ≤ 6 words (`two blows at 6 hp`,
/// `a max hit at 4 hp`, `goblin −6 at 6 hp`), `t` its tick, `odds` the share of the reseeded
/// replays that died too (0..1) and `one_in` = round(1 / odds) — `REPLAYS` when none died (rarer
/// than 1 in 12).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct DeathLuck {
    pub text: String,
    pub t: u32,
    pub odds: f64,
    pub one_in: u32,
}

/// Cut 26: a `route` verdict's cause (`Death.route_cause`): at `fork` the route took `taken`;
/// the set on the route with `other` instead (`route`) gets past the death's floor in `survive`
/// of the sends (≥ `trace::ROUTE_BAR`, and `trace::PATCH_MARGIN` over the route it took,
/// `base`), on paired seeds.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct RouteCause {
    pub fork: u32,
    pub taken: String,
    pub other: String,
    pub route: Vec<u32>,
    pub survive: f64,
    pub base: f64,
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

/// Learned details captured from the bloodline that earned a first victory.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BossKnowledge {
    pub boss: String,
    pub facts: Vec<String>,
    pub ledger: Option<LedgerRow>,
    pub wall: Option<BossWall>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct BloodlineReturn {
    #[serde(default, skip_serializing_if = "is_zero")]
    pub legacy_earned: u32,
    pub id:u32, pub name:String, pub runs:u32, pub deepest:u32, pub gold:i32,
    /// This slot's class gains; arrays allow clients to merge class changes
    /// across report slices without assigning everything to the latest class.
    #[serde(default)]
    pub xp: Vec<XpReport>,
    /// Bounded training/unlock beats from this slot's own report (empty is known empty).
    #[serde(default)]
    pub packages: Vec<String>,
    /// This slot's new records and first kills; empty is known empty.
    #[serde(default)]
    pub bests: Vec<String>,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub boss_knowledge: Vec<BossKnowledge>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct ReturnReport {
    /// Cut 113 §3: the return's pick (one of three, sized by the absence), as it waits at camp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pick: Option<ReturnPick>,
    /// Actual inherited points earned by completed runs in this report.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub legacy_earned: u32,
    /// Transport acknowledgement only. The final call returns the whole absence once.
    #[serde(default, skip_serializing_if = "is_false")]
    pub slice_pending: bool,
    #[serde(default, skip_serializing_if="Vec::is_empty")]
    pub bloodlines: Vec<BloodlineReturn>,
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
    /// Cut 29 §1: of `marks_earned`, the night's mark (◆1 per day whose absences brought a send
    /// home; the frontier mark is gone).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub night_marks: u32,
    /// Cut 29 §6: the companions that fell in the absence, named (the report's line: `Greth · ogre L5
    /// · fell D12 to lurker`); `lost` keeps the names as before.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallen: Vec<Fallen>,
    /// Cut 29 §3: the absence's real runs metered, summed (the report's per-night meter).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub meters: Option<crate::meters::MeterWire>,
    /// Cut 29 §2: the systems the absence opened, in order (the reveal's glint).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems_opened: Vec<String>,
    /// Cut 29 §1: the oaths the extra slots kept (`id`, `kind`, `label` its text; `oath` is the
    /// first slot's).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oaths_kept: Vec<OathReward>,
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
    /// Cut 117 §4 (blind 8cf9050 B: `Supplies limited · $0 budget` after the apprentice spent): the absence's
    /// supply budget when the repeat was limited — `income` (what the absence brought home: exits, salvage, the
    /// heir purse), `spent` (what the repeat and the drills bought), `left`, and `reason`: `no_income` (nothing
    /// came home: the repeat spends only away income, never the purse), `income_spent` (the repeat used it up),
    /// `purse_short` (the purse itself could not pay — a forge order or a purchase took it). Absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supply_budget: Option<SupplyBudget>,
    /// Cut 20 §5: the absence's bounty floor and whether a run brought it home (`bounty D12 ·
    /// missed` / `taken $412`); absent when the lineage had no bounty during the absence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<BountyReport>,
    /// Cut 26 §6: every drive-off of the absence (the last `EXITS_CAP`), each opening its verdict
    /// (`no counter`, the boss's defence, the counter row to write) by its `run_id`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drives: Vec<DrivenOff>,
    /// Cut 26 §2: each band the absence's sends reached and the lane the route played there
    /// (`D5–8 · the Fens`), shallowest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lanes: Vec<String>,
    /// Cut 21 §2: found supplies the absence's exits put on the shelf instead of salvaging,
    /// per kind — `gold` is their price on the shelf (what the repeat did not have to pay;
    /// `found heal ×12 → shelf`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shelved: Vec<SalvageRow>,
    /// QA on 912e135 (qaW: `♟18` over a report of ♟2–♟17, read as the heir who ran): the first
    /// and the last heir who ran these runs (`[2, 17]`; one heir `[5, 5]`); empty with no run.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heirs: Vec<u32>,
    /// Cut 28 §2 (both raters: reports opened with salvage walls): the report's first screen,
    /// decisions first (≤ `offline::LEAD_MAX`): the oath, the plateau, a counter learned, a record,
    /// the worst death's verdict, a drive-off, the bounty, the first pending decision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lead: Vec<ReportLead>,
    /// Cut 28 §1: the sworn oath over the absence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<OathReport>,
    /// Cut 30 §4: what grew on each track over the absence (the report leads with it): xp, a
    /// level, a package level, gold, a building, a best, a stage opened.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub grew: Vec<GrewLine>,
    /// Cut 30 §1–2: the packages' beats over the absence — `STEADY L3`, `DRILLED · Warlord`,
    /// `+Guarded`, `the pen`, `QUEST DONE · reach D10`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<String>,
    /// Cut 30.5: the workers' acts over the absence (`apprentice · +2 steps`; `first` the first ever).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workers: Vec<crate::tree::WorkerAct>,
    /// Cut 30.5: the haul gold this absence left in the chest (before the porter).
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub chest: i32,
    /// Cut 118 §3: the finds sealed this absence, opened now (one reveal, best first, and the count).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finds: Option<crate::feats::FindsReveal>,
    /// Cut 118 §8: the notable acts since the last report — `trial` cleared, a find `set` completed, `swift`
    /// floors, a `seek`, a `wish` granted — each with its lineage day.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub feats: Vec<crate::feats::FeatNews>,
}

/// Cut 30 §2: a death's cheapest lever (`kind` spend · package · wait; `text` ≤ 3 words).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Lever {
    pub kind: String,
    pub text: String,
    /// Cut 115 §4: a `tactic` lever's package id and variant (`take_fix`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// Cut 122 §2 (blind 9621b19 A: `TRY Mirror rhythm`, then the forecast's D33 50→0%): a `tactic` pick priced at
    /// the death's wall on the camp's panel (with `deathDeltas`; absent before, `pending` true) — `trade_off` when it
    /// is worse there past the noise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<WallPrice>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub trade_off: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub pending: bool,
}

/// Cut 115 §1: the build the player wore (`Packages.build`): its name (`Bulwark`, `Guarded skirmisher`), the synergy
/// its pair forms and that synergy's effect (≤ 6 words), and the picks it is made of (`corridor fighting · at two`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BuildWire {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synergy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect: Option<String>,
    #[serde(default)]
    pub picks: Vec<String>,
    /// Cut 119 §5: the pet synergy formed (role + build), its effect (`Falconer`, `scout sees further`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pet_synergy: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pet_effect: Option<String>,
}

/// Cut 115 §1: a meter's rule fires by who chose the row (`picked` · `taught` · `default` · `chores`), and their
/// share of all fires (0..1).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct CreditShare {
    pub credit: String,
    pub fires: u32,
    pub share: f64,
}
impl Eq for CreditShare {}

/// Cut 30 §4: one thing that grew on a track (`character` · `L7`, `scale` · `best D14`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GrewLine {
    pub track: String,
    pub what: String,
}

/// Cut 30 §4: a track on the tracks panel — its stage (≤ 2 words), stages reached, the next stage
/// and its trigger (`next · kennel · first tame`), progress toward a numeric trigger (0..1).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct TrackWire {
    pub id: String,
    pub stage: String,
    pub stages: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<f64>,
}

/// Cut 30 §3: a building (`blacksmith`, look 1–3, the day it was built).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BuildingWire {
    pub id: String,
    pub level: u32,
    pub day: u32,
    /// c30-legible: what raised it (`first gold home`, `town::BUILDINGS`) — its arrival's beat names it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub trigger: String,
}

/// Cut 30 §3: the town — its buildings, the next plot and its trigger, the bank.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct TownWire {
    #[serde(default = "crate::town::home_default")]
    pub home: bool,
    #[serde(default)]
    pub auto_collect: bool,
    pub buildings: Vec<BuildingWire>,
    #[serde(default)]
    pub next_ready: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_trigger: Option<String>,
    pub bank: i32,
    pub bank_cap: i32,
    pub interest: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quest: Option<QuestWire>,
    #[serde(default)]
    pub quests_done: u32,
    /// Cut 30.5: the workers at their posts (hired; the lit node's greyed with its price).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workers: Vec<crate::tree::WorkerPost>,
}

/// Each hero's lifetime achievements, independent of XP and package marks.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HeroLegacy {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(default)]
    pub spent: u32,
    #[serde(default)]
    pub upgrades: std::collections::BTreeMap<String, u32>,
    pub heir: u32,
    pub points: u32,
    pub runs: u32,
    pub best_depth: u32,
    pub class: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BloodlineLegacy {
    pub points: u32,
    pub spent: u32,
    pub upgrades: std::collections::BTreeMap<String, u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HeroSlot {
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub specialization:Option<crate::specialization::Style>,
    #[serde(default)]
    pub look: String,
    #[serde(default)]
    pub hero_name: String,
    /// Cut 115 §1: the build the bloodline's picks make (`Bulwark`, `Guarded skirmisher`).
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub build: Option<String>,
    pub id:u32, pub name:String, pub heir:u32, pub class:String,
    pub level:u32, pub xp:u32, pub next:Option<u32>,
    pub state:String, pub live:Option<LiveRun>, pub rest_s:f64,
    pub legacy:BloodlineLegacy, pub notice:bool, pub chronicle:Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LegacyUpgrade {
    pub id: String,
    pub rank: u32,
    pub cap: u32,
    pub price: u32,
    pub effect: String,
    pub affordable: bool,
    /// Blind b58b431: the hero is away and the points buy it — `upgradeHeroNext` buys it for the next run.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub next_run: bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub name:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub branch:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub parent:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub min_depth:Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub blocked:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub owned_effect:Option<String>,
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct LegacyRespec {
    pub refund:u32,
    pub available:bool,
    pub points_after:Option<u32>,
    pub blocked:Option<String>,
}

/// Cut 30 §5: the quest on the board — one plain goal (≤ 5 words), the reward's picture, progress
/// (0..1), kept or not, a free swap left today.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct QuestWire {
    pub goal: String,
    pub reward: String,
    pub progress: f64,
    pub done: bool,
    pub swap: bool,
}

/// Cut 30 §2: a package on the wire (`Guarded L3`): its kind, level, runs toward the next level,
/// equipped (the slot) or not, and its rows at its level (tagged as compiled).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PackageWire {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    pub kind: String,
    pub level: u32,
    pub runs: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_at: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<u32>,
    pub owned: bool,
    /// The stage that brings it (≤ 3 words) while it has not arrived.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub trigger: String,
    /// Marks the next level costs (a level spend), when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level_price: Option<u32>,
    /// Cut 111: a tactic's two L3 row variants (names ≤ 3 words), and the one worn (from L3).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<u32>,
    /// Blind 77030eb: the rows the next level brings or changes to (`packages::level_adds`; empty: none).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub level_adds: Vec<crate::rules::Row>,
}

/// Cut 30 §1: a drilled counter on the stance (`drill · attack boss`), revocable.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct DrillWire {
    pub boss: String,
    pub rows: Vec<Row>,
    pub revoked: bool,
    /// The boss's scar for the lineage now, in percent (`scarred ×3` = 15).
    pub scar: u32,
}

/// Cut 30 §2: the lineage's packages — every package (owned or its trigger), the slots, the drills,
/// the scars, the pen, the wake's temperament cards, the rows each compiled row shadows behind.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PackagesWire {
    pub all: Vec<PackageWire>,
    pub stance: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tactics: Vec<String>,
    #[serde(default)]
    pub tactic_slots: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperament: Option<String>,
    #[serde(default)]
    pub temperament_open: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub offer: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drills: Vec<DrillWire>,
    /// Band boss → meetings (the scars: 5 % each, 30 % at most, gone once slain).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scars: Vec<(String, u32)>,
    pub pen_open: bool,
    /// Unmet custom-rule requirements, authored by the core; empty when open.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pen_needs: Vec<String>,
    /// Per row of the compiled set: its package label (`Steady`, `drill · Warlord`, empty for a pen
    /// row) and the row that always pre-empts it when one does (`Guarded wins`), by index.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rows: Vec<RowSource>,
    /// Literal: a harness's lineage (no packages).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub literal: bool,
    /// Cut 115 §1: the build the picks make (absent while nothing is the player's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildWire>,
}

/// Cut 30 §2: where a compiled row came from and whether a same-role row above always wins.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RowSource {
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadowed_by: Option<u32>,
}

/// Cut 28 §2: one line of a report's first screen (`ReturnReport.lead`): `k` oath · plateau ·
/// counter · record · death · driven · bounty · pending; `text` ≤ 6 words.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReportLead {
    pub k: String,
    pub text: String,
}

/// Cut 20 §5: the bounty floor (`Lineage.bounty`): each night the lineage's best depth + 2 —
/// its gold ×2 and one item of the lineage's next tier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Bounty {
    pub depth: u32,
    /// Cut 28 §1 (both raters: `bounty D13 · missed` never said what it pays or needs): what the
    /// floor pays (`$×2 · item`) and needs (`reach`, or `slay mother` on a boss floor the run must
    /// win to come home from), and on a boss floor the boss and its counter fact (`mother: ?`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub pays: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub needs: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fact: Option<String>,
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
    /// QA on 524827b (qaAA: `$936 lost` on a return report with 0 deaths): the part of `lost`
    /// the exits that kept something left unkept (a return's 40 %) — `not kept`, not `lost`.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub unkept: i32,
    /// Blind ad71e72 (A: `$8 GOLD EARNED` while the purse rose ~$1000 on a D19 start): the
    /// waystone passages paid into the purse at the sends — income beside `home`.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub passage: i32,
    /// Cut 118 §2 (gate fix): the graves' packs the absence's runs brought home — income beside `home`.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub recovered: i32,
    /// Cut 119 (gate fix): the packs the absence's pets fetched home from deaths — income beside `home`.
    #[serde(default, skip_serializing_if = "is_zero_i")]
    pub fetched: i32,
    /// The purse's actual change over the absence (`home + salvage + passage + wake − spent`
    /// and every other movement: forge steps the apprentice bought, hires, bank moves).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub net: Option<i32>,
    /// Cut 117 §1 (blind 8cf9050 B: `$0 GOLD EARNED · purse −$6789 · forge −$6750 · lost $2620`): the absence's
    /// ledger (`GoldLedger`), its terms summing exactly to `net`. Absent for an absence saved before it existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ledger: Option<GoldLedger>,
    /// Cut 121 §2 (blind 1a7d834 B: `−$2807 GOLD CHANGE` led a return — the apprentice's spending read as a loss): the
    /// absence's income — the carry kept, the heir purse, passages, graves recovered, packs fetched (the ledger's
    /// income terms) — the headline's `+$X earned`. Absent before the ledger.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub earned: Option<i32>,
    /// Cut 121 §2: what the workers spent of it (a positive number): the apprentice's forge steps and sinks
    /// (rations, tithes), the ranks the ranks order bought — the headline's `−$Y spent by workers`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spent_by_workers: Option<i32>,
}

/// Cut 117 §4: an absence's supply budget when the repeat was limited (`ReturnReport.supply_budget`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SupplyBudget {
    pub income: i32,
    pub spent: i32,
    pub left: i32,
    pub reason: String,
}

/// Cut 117 §1: one term of an absence's gold ledger (`GoldLedger.terms`): its label and signed amount.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoldTerm {
    pub label: String,
    pub amount: i32,
}

/// Cut 117 §1: an absence's gold ledger (`GoldSummary.ledger`): `earned` the sum of its positive terms, `spent` the
/// sum of its negative ones (a positive number), `net` = `earned − spent` = the purse's change. `terms`, each signed
/// and named, zero terms left out: `carried` (what the exits, salvage and passages brought to the door) and `lost`
/// (−, the carry the exits did not keep: `carried + lost` is the income kept), `heir` (the heir purse's top-ups),
/// `apprentice` (−, his forge steps), `forge` (−, steps bought by hand), `works`, `supplies`, `tolls`, `hires`,
/// `bank` (deposits −, withdrawals +), `other` (unlocks, oaths, hatching…).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GoldLedger {
    pub earned: i32,
    pub spent: i32,
    pub net: i32,
    pub terms: Vec<GoldTerm>,
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
    /// Cut 119: the pet's life past its kind — role, xp, lamed runs, falls, heirs served, grudge, bred, fetched.
    #[serde(default, skip_serializing_if = "crate::pets::PetLife::is_default")]
    pub life: crate::pets::PetLife,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Egg {
    pub id: u32,
    pub kind: String,
    pub tags: Vec<String>,
    pub gen: u32,
    pub hatch_in: u32,
    pub from_loss: bool,
    /// Cut 119 §4: a bred egg's sire (its name, the generation after it: `Rook II`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sire: String,
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
    /// Cut 122 §6: as `Counter.owned` / `refusal` / `wear`.
    #[serde(default = "yes")]
    pub owned: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wear: Option<String>,
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

/// RUNS_UI: one run in the runs log (`LineageState::run_log`, oldest first, cap `RUN_LOG_CAP`). `via`
/// `away` (an absence's batch) · `town` (unwatched while the app was open: `Game::advance`) ·
/// `watched`. A record with `sampled` is no run: the absence's runs extrapolated past its stall.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RunRec {
    pub id: u32,
    pub heir: u32,
    pub via: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub absence: Option<u32>,
    pub clock_s: u64,
    pub start: u32,
    pub depth: u32,
    pub tier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub gold: i32,
    pub found: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kept: Vec<String>,
    pub turns: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub best: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sampled: Option<u32>,
    /// RUNS_UI × run-clear: the run's finds (its exit line's, rarest first, ≤ 6) — the log's rarity marks and the entry's card.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub finds: Vec<InvItem>,
    /// RUNS_UI × Cut 30.5: of `gold`, what the run's checkpoints secured (kept whole at any exit; a death keeps it alone).
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub secured: i32,
}

/// RUNS_UI: the run under way (`Lineage.live`; absent at home).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LiveRun {
    /// Presentation only; older wire data has no activity.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub activity: String,
    pub run_id: u32,
    pub heir: u32,
    pub depth: u32,
    pub start: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub turn: u32,
}

/// RUNS_UI: `Game::advance` — the runs it finished and the run under way after it.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Advance {
    pub ended: Vec<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<LiveRun>,
}

/// RUNS_UI: a past run re-simulated from its send (`Game::replay`): each floor's first snapshot
/// (later entities, items and seen tiles folded in) and its events; `hash` = `events_hash`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReplayFloor {
    pub snapshot: Snapshot,
    pub events: Vec<Ev>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Replay {
    pub run_id: u32,
    pub floors: Vec<ReplayFloor>,
    pub hash: String,
    pub ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Lineage {
    /// Cut 118: boss tokens, trials, the find log, swift floors, the sinks, the hero's wish (`feats::wire`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feats: Option<crate::feats::FeatsWire>,
    /// Cut 118 §8: the hours until the King may fall at the current pace (`feats::king_eta_h`; 0 slain, none
    /// before a pace), and the record's share of the way to him (percent).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub king_eta_h: Option<u32>,
    #[serde(default)]
    pub king_pct: u32,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub class_styles:Option<crate::specialization::Choice>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endgame: Option<crate::endgame::Progress>,
    /// Cut 121 §1: the numbered descent under way — its tier, the floor it started on, the hp and attack its foes
    /// gain (percent) and the modifiers at work (`endgame::ascension_wire`). Absent at tier 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub descent: Option<AscensionWire>,
    #[serde(default)]
    pub selected_loadout: Vec<u32>,
    #[serde(default)]
    pub hero_slots: Vec<HeroSlot>,
    #[serde(default)]
    pub selected_bloodline:u32,
    #[serde(default)]
    pub bloodline_price:i32,
    #[serde(default)]
    pub bloodline_cap:u32,
    #[serde(default)]
    pub bloodline: BloodlineLegacy,
    #[serde(default)]
    pub hero_legacy: Vec<HeroLegacy>,
    #[serde(default)]
    pub legacy_upgrades: Vec<LegacyUpgrade>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub legacy_respec:Option<LegacyRespec>,
    /// Cut 113 §3: the return's pick waiting at camp (`returns::wire`; `takeReturnPick(id)`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_pick: Option<ReturnPick>,
    /// RUNS_UI: the runs log, the run under way, the run ids a replay is held for, the lineage
    /// clock and the absences counted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<RunRec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub live: Option<LiveRun>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replays: Vec<u32>,
    #[serde(default)]
    pub clock_s: u64,
    #[serde(default)]
    pub absences: u32,
    /// Cut 29 §1 (E1): the wall's edit on offer (`Game::wall_edit`, cached a day) while the best depth holds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall: Option<WallEdit>,
    /// Cut 29 §3: the meters — the last two runs (oldest first: the camp's two-run comparison),
    /// this night's runs so far and the last full night's (a night is `NIGHT_RUNS` runs).
    #[serde(default)]
    pub meters: LineageMeters,
    /// Cut 29 §2: the system curriculum — every system in order (`systems::SYSTEMS`), open or not,
    /// its trigger (≤ 3 words), `new` when opened since the camp last looked (`seenSystems()`
    /// clears it). The client gates the editor's vocabulary and the camp's tiles by `open`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems: Vec<SystemInfo>,
    /// Cut 29 §1: the catalogue's tier now (0–6), the oath slots (1–3), every sworn oath's id (the
    /// first slot's first; `oath` is that one), the oath draw, the works commissioned and the next
    /// commission's price.
    #[serde(default)]
    pub tier: u32,
    #[serde(default = "one")]
    pub oath_slots: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sworn: Vec<String>,
    #[serde(default)]
    pub oath_draw: OathDraw,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub works: Vec<String>,
    #[serde(default)]
    pub commission: Commission,
    /// Cut 29 §4: the kinds a player's `throw` rows name that the repeat lacks — offered (a tap buys one).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_added: Vec<RepeatAdd>,
    /// Cut 29 §4: the standing orders in one place (`setOrders`).
    #[serde(default)]
    pub orders: StandingOrders,
    /// Cut 29 §6 (AX: `pack 4` bought, the shelf still read 3/3 — the client computed the cap
    /// from `supply_cap_5` alone): the shelf's cap, the core's (`LineageState::supply_cap`).
    #[serde(default)]
    pub supply_cap: u32,
    pub seed: u64,
    pub heir: u32,
    #[serde(rename = "trait")]
    pub trait_: String,
    /// Cut 13 §2: the two traits a new heir may wake with (drawn from the lineage seed, never
    /// the last heir's); empty once chosen or sent. `set_trait(name)` picks one; a send without
    /// a pick keeps the first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trait_offer: Vec<String>,
    /// Cut 30: the heir's traits — blood and born slots, the wake's three cards (chip, formula
    /// with `?` for an unlearned gift, tier, source), the blood/bloodline gates, the last fade.
    /// Absent for the neutral heir before traits arrive (heir 3 / a death past D5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heir_traits: Option<crate::traits::HeirTraitsWire>,
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
    /// Cut 119: the companions' camp read (the tame row, the keeper's order, synergies, fetched, old hounds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pets: Option<crate::pets::PetsWire>,
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
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub guns:Vec<GunOffer>,
    /// Cut 23 §3: per row of the active set (by index), its why-not over the recent sends
    /// (`LineageState::row_stats`); `null` before any send under that row.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_why: Vec<Option<RowStat>>,
    /// Hero looks: the heir's cosmetic look (`male | female | cat`; the class's own until
    /// `setLook`); sprites and portraits are `hero_<class>_<look>`.
    #[serde(default)]
    pub look: String,
    /// Cut 26 §2: the forks the hero has seen (a `fork:<d>` fact each), shallowest first — the
    /// route chip line above the rows (`⑂ D5 fens · D14 crypt`); empty until D4's two stairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forks: Vec<ForkChip>,
    /// Cut 116 §1: the live heir's band-boss affixes (`Warlord · armoured`), descent order, each with
    /// what it does and what answers it — the camp names them before a send.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affixes: Vec<BossAffix>,
    /// Cut 26 §2: every lit waystone as a (lane, depth) pair, ascending by depth — the start
    /// sheet lists them; `current` marks the ones lit for the active set's route (the ones a send
    /// can start on: `waystones`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lanes: Vec<LaneStone>,
    /// Cut 26 §6 (AP: `R3 descend · locked cond` found only in a trace): per row of the active set,
    /// the gate of its first cond this lineage cannot use (`see: den`, `enter fens`, `◆2`), `null`
    /// for a row whose conds are all open; empty when none is locked.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locked_rows: Vec<Option<String>>,
    /// Cut 28 §1: the oath board — the lineage's standing oaths (≤ `oath::BOARD`), the sworn one
    /// flagged; `oath` the sworn one's id; `titles` the chronicle titles oaths earned.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oaths: Vec<Oath>,
    #[serde(default)]
    pub oath: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub titles: Vec<String>,
    /// Cut 28b: the oath board has something to answer — the lineage has met its first band boss
    /// (a wall seen) or its first plateau (an absence that stalled): the reveal ladder carves the
    /// board then, not when the purse first covers a price.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub oath_open: bool,
    /// Cut 28 §1: the band bosses from the Warlord to the first unslain one past the best depth,
    /// each counter as a fact the lineage knows or can learn (`mother: fire` / `mother: ?`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub walls: Vec<BossWall>,
    /// Cut 30 §2: the packages (slots, levels, drills, scars, the pen).
    #[serde(default)]
    pub packages: PackagesWire,
    /// Cut 30 §3: the town (buildings, the next plot, the bank, the quest board).
    #[serde(default)]
    pub town: TownWire,
    /// Cut 30 §4: the four tracks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tracks: Vec<TrackWire>,
    /// Cut 30.5: the works tree (workers, the tracks' stages as its branches, the chest, the pill);
    /// `gold` is the purse (collected), the chest beside it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tree: Option<crate::tree::WorksWire>,
    /// Cut 30 (PROGRESSION_V2 §4): the lineage's age (hours, offline included); the systems ready and
    /// waiting their turn (one opens a report), in order; the next system to come — its trigger, and the
    /// hours of age it still waits once triggered (`next · tactics · 3 h`).
    #[serde(default)]
    pub age_h: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reveal_queue: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reveal_next: Option<RevealNext>,
    /// PROGRESSION_V2 §2, reserved (Cut 31: the expedition and the era): glory, expeditions, the era's gate.
    #[serde(default)]
    pub glory: u32,
    #[serde(default)]
    pub expeditions: u32,
    #[serde(default)]
    pub era_gate: u32,
}

/// Cut 30: the next system to open — its id, its trigger (≤ 3 words), whether the trigger has come,
/// and the hours of age it still waits.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RevealNext {
    pub id: String,
    pub trigger: String,
    pub triggered: bool,
    pub wait_h: u32,
}

/// Cut 28 §1: an oath's reward — never a stat: `card` · `slot` · `row` · `title` · `waystone` ·
/// `verb`; `id` the unlock (or the title, or the waystone's depth), `label` ≤ 3 words.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OathReward {
    pub kind: String,
    pub id: String,
    pub label: String,
}

/// Cut 28 §1: one standing oath (`oath::board`): its constraint as chips (≤ 3 words each) and
/// `text` (the chips joined ` · `), its reward, its price in gold; `boss` / `depth` / `counter`
/// when it points at a band boss (`counter`: `mother: fire`, or `mother: ?` unknown).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Oath {
    pub id: String,
    pub kind: String,
    pub chips: Vec<String>,
    pub text: String,
    pub reward: OathReward,
    pub price: i32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub sworn: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter: Option<String>,
}

/// Cut 28 §1: the sworn oath on a panel (`Forecast.oath`): the share of the sends that keep it,
/// its 95 % half-width, and `night` — the chance a night of `NIGHT_RUNS` sends keeps it once.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OathShare {
    pub id: String,
    pub text: String,
    pub share: f64,
    pub pm: f64,
    pub night: f64,
    /// Cut 28b (AW): the steps toward it, each the share of the sends that got that far
    /// (`D13` reached · `met` · `burned`), so a lever that moves a step reads even while the
    /// kept share sits in its noise (a second fire potion: `burned 18% → 31%`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<OathStep>,
}

/// Cut 28b: one step toward the sworn oath on a panel (`OathShare.steps`): `k` ≤ 2 words.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct OathStep {
    pub k: String,
    pub share: f64,
}

/// Cut 28 §1: the sworn oath over an absence (`ReturnReport.oath`): the sends while it was sworn,
/// those that met it, whether it was kept (the reward granted).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OathReport {
    pub id: String,
    pub chips: Vec<String>,
    pub text: String,
    pub runs: u32,
    pub kept: u32,
    pub done: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reward: Option<OathReward>,
    #[serde(default)]
    pub price: i32,
    /// Cut 28b: the sends that broke it (a `return` taken, a rest) and the cause most of them
    /// share (`R2 return`); 0 / absent when none did.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub broken: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
}

/// Cut 28 §1: a band boss as the wall ahead (`Lineage.walls`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BossWall {
    pub boss: String,
    pub title: String,
    pub depth: u32,
    pub slain: bool,
    pub known: bool,
    pub fact: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learn: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub counter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<Row>,
    /// Cut 118 (the roster preview): the live heir's affix on him, its answers; his guard (the band's wandering
    /// champion, else his escort) and its answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affix: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affix_counter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard_counter: Option<String>,
}

/// Cut 28 §2: one part of an attributed forecast move (`ForecastMove.parts`): `kind` party · kit ·
/// purse · start · facts · heir (state since the send) or route · rows (the set's edit); `text`
/// ≤ 3 words; `move` the paired move this part alone made.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MovePart {
    pub kind: String,
    pub text: String,
    #[serde(rename = "move")]
    pub move_: ForecastVs,
}

/// Cut 28 §2 (AV: the scene said `R2 now → dies` when both pets had died): the camp's move
/// against the set sent, attributed (`forecast::forecast_move`): `whole` the active set on today's
/// lineage less the sent set on the lineage at its send, on paired seeds; `parts` in order (state
/// first, then route, then rows) sum to it; `lead` the part with the largest headline move; `rows`
/// the rows differ (the divergence scene runs only then); `state` a state part is present.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForecastMove {
    pub whole: ForecastVs,
    pub parts: Vec<MovePart>,
    pub lead: String,
    pub rows: bool,
    pub state: bool,
    pub sims: u32,
    pub refined: bool,
}

/// Cut 26 §2: a seen fork — the stairs into the band at `depth`: `near` the band's own biome,
/// `far` the next one early; `taken` the biome the active set's route takes there. `open` false:
/// the fork above took its far stair, so this band is the deferred biome's (`taken`), no choice.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ForkChip {
    pub depth: u32,
    pub near: String,
    pub far: String,
    pub taken: String,
    #[serde(default = "yes")]
    pub open: bool,
}

/// Cut 26 §2: a lit waystone on a lane — its floor, the biome there (`lane`), the route's far
/// stairs above it (`route`, the prefix it was lit on; empty on the base order) and whether it is
/// lit for the active set's route (`current`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LaneStone {
    pub depth: u32,
    pub lane: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route: Vec<u32>,
    #[serde(default)]
    pub current: bool,
}

/// Cut 26 §2: one stair of a fork on the fork chip's option tablet (`Game::fork_forecast`): the
/// active set with the route taking `biome` at `fork` (`far`: the far stair), measured on the
/// camp's seeds (the first pass, paired with the current route's panel) like `StartOption`.
/// `depth` is the bar: the band's last floor (`fens D8 61% · burrows D8 34%`); `delta` the
/// headline — `bank_delta` when either panel banks, else `reach_delta`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ForkOption {
    pub fork: u32,
    pub biome: String,
    pub far: bool,
    pub current: bool,
    /// The route the option plays (fork depths of its far stairs).
    #[serde(default)]
    pub route: Vec<u32>,
    pub depth: u32,
    pub reach: f64,
    pub reach_delta: f64,
    pub bank: f64,
    pub bank_delta: f64,
    pub gold: f64,
    pub gold_delta: f64,
    #[serde(default)]
    pub death: f64,
    #[serde(default)]
    pub death_delta: f64,
    pub delta: f64,
    pub pm: f64,
    #[serde(default)]
    pub refined: bool,
    #[serde(default)]
    pub low: u32,
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
    /// Cut 29 §1: the tier that opens it (0–6: T1 the first bank, T2 the Warlord met … T6 the
    /// Lurker Queen met); a card of a tier not yet open reads its gate in `needs` (`meet Mother`).
    #[serde(default)]
    pub tier: u32,
    /// Cut 29 §1: an automation — gold only (`gold` its price, `needs: $N more` when short).
    #[serde(default, skip_serializing_if = "is_false")]
    pub gold_only: bool,
}

#[derive(Clone,Debug,Serialize,Deserialize,PartialEq,Eq)]
pub struct GunOffer {
    pub kind:String, pub selected:bool, pub owned:bool, pub price:u32,
    pub available:bool, pub blocked:Option<String>, pub capacity:u8,
    pub range:i32, pub damage:(i32,i32), pub armour_piercing:i32, pub reload_ticks:u32,
}

/// Cut 23 §1: one step of a forge ladder (`sword +1`, `mail`, `pack 4`) and its gold price.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitStep {
    pub label: String,
    pub price: u32,
    pub owned: bool,
    /// Run-clear: the item kind the step forges (`sword`, `mail`); absent on the pack's steps.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Run-clear: the forged piece's rarity (`item::rarity`); absent = common.
    #[serde(default, skip_serializing_if = "crate::item::Rarity::is_common")]
    pub rarity: crate::item::Rarity,
    /// Cut 113 §2: an owned tier's branch (`aim · edge`, `plate · pace`); absent on the pack's.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Cut 113 §3: one offer of a return's pick — `drill` (runs on a worn package: `title` its name),
/// `legacy` (points), `forge` (the next step of `target`'s ladder at `price`, discounted), `marks`
/// (◆, when the drill or the forge has nothing left). `available`: takeable now (a forge offer the purse
/// cannot pay waits like the rest).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReturnOffer {
    pub id: String,
    pub title: String,
    pub line: String,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub price: i32,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Cut 113 §3: the return's pick — the absence minutes behind it, its size (1–5) and its offers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ReturnPick {
    pub minutes: u64,
    pub size: u32,
    pub offers: Vec<ReturnOffer>,
    /// Cut 118 §6: the offer `collect & send` takes when the player does not choose (its id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Cut 113 §2: one of the next tier's two steps (`buyKit("weapon:edge")`), priced as the tier:
/// `label` its tile line (`aim +4%`, `edge +2`, `mail +1`, `pace +1`); `default` the ladder's own
/// order; `lean` the branch the apprentice follows (the player's last off-default pick).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct KitBranch {
    pub id: String,
    pub label: String,
    pub default: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lean: bool,
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
    /// Optional kit choices can be packed/stowed without losing ownership.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    pub owned: u32,
    pub steps: Vec<KitStep>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<KitNext>,
    /// Cut 113 §2: the next tier's two steps (weapon, armour); empty on the pack's and at the top.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub branches: Vec<KitBranch>,
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
    /// Cut 121 §4 (blind 1a7d834 B: a boss rule misfired on a siren): a foe-tag row's fires by the foe kind it acted on
    /// over the recent sends, and whether each is a band boss — the tablet's `also matches: siren`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fired_on: Vec<FiredOn>,
}

/// Cut 121 §4: a foe kind a row fired on (`RowStat.fired_on`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FiredOn {
    pub kind: String,
    pub boss: bool,
    pub n: u32,
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

/// Cut 27 §1: one state-changing beat on a folded floor — a chip on the fold line. `kind`:
/// `theft` (a foe took from the hero), `find` (the hero picked up an item), `gold` (a gold
/// pile), `use` (a potion / scroll / bell spent), `fact` (a fact learned), `dip` (the floor's
/// lowest hp when ≤ 30 % of max: death-adjacent), `max_hp` (max hp moved), `pet` (a companion
/// tamed, freed, lost, fallen or stealing), `boss` (a boss slain), `bones` (bones found),
/// `level`, `hatch`. `text` ≤ 3 words (`stolen heal`, `hp 9/40`); `t` the run tick.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FoldBeat {
    pub depth: u32,
    pub t: u32,
    pub kind: String,
    pub text: String,
}

/// Cut 27 §1: one folded floor as the sim played it — its snapshot at the floor's first tick
/// (with the entities and items first seen later on it folded in, and every tile seen on it:
/// what `runlog.floorSnapshot` builds) and its events, so a tap on the fold line can play it on
/// the renderer (`load(snapshot)`, `apply(events)`); `clear` the forecast's share through it,
/// `gold` the loot it added (coins), `beats` its state changes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FoldFloor {
    pub depth: u32,
    pub clear: f64,
    pub gold: i32,
    pub snapshot: Snapshot,
    pub events: Vec<Ev>,
    pub beats: Vec<FoldBeat>,
}

/// Cut 27 §1: the watch's fold (`Game::fold`, called right after `send`): the live run played
/// through the floors the set clears ≥ 95 % (`Forecast.fold_to` at the send), one line
/// `D{from}–{to} · {clear} · +${gold} · chips`. `clear` is the forecast's share through all of
/// them (the product of the floors' clears); `gold` the loot added over them; `beats` every
/// state change on them (`FoldBeat`), and `chips` the line's words (≤ 3 words each, by kind:
/// `stolen heal`, `2 finds`, `hp 9/40`). `step` is the fold as one `step()` — every event, the
/// snapshot on the first unfolded floor, `run_over` / `exit_pending` when the run ended inside
/// the fold (the watch then shows the exit as after any step). Nothing folded (`to < from`):
/// no tick ran, `floors` is empty and `step` is the current snapshot.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FoldLine {
    pub from: u32,
    pub to: u32,
    pub clear: f64,
    pub gold: i32,
    pub beats: Vec<FoldBeat>,
    pub chips: Vec<String>,
    pub floors: Vec<FoldFloor>,
    pub step: StepResult,
    /// Cut 28 §4 (AV: "`send skips rest` sent a 9/40 heir" — the fold handed off a hurt hero): his hp
    /// and max hp where the fold hands the run to the watch; `low` at or under half his max (the
    /// line says `hp 9/40`: a chip in `chips` too).
    #[serde(default)]
    pub hp: i32,
    #[serde(default)]
    pub max_hp: i32,
    #[serde(default, skip_serializing_if = "is_false")]
    pub low: bool,
}

/// Cut 27 §2: how one branch of a divergence's paired sim ended (the panel's own result for
/// that seed): `tier` `bank` / `return` / `death` / `stall`, `depth` the deepest floor, `cause`
/// the killer on a death, `gold` kept.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DivergenceEnd {
    pub tier: String,
    pub depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<String>,
    pub gold: i32,
    /// Cut 28 §1: with an oath sworn, whether this branch's whole run kept it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<bool>,
}

/// Cut 27 §2: one branch's seconds from the divergence — `snapshot` a few ticks before `tick`
/// (`DIVERGENCE_LEAD`: both branches' are the same state unless the sets packed differently),
/// `events` from it to `DIVERGENCE_AFTER` ticks past `tick` (ending early at the run's end or a
/// descend), for the renderer (`load(snapshot)`, `apply(events)`); `end_snapshot` the last frame.
/// `row` the row (0-based, in its own set) that fired at `tick` (`None`: no row — a chore, or
/// the hero only moved), `text` its words (`R5 bank`, `—`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DivergenceBranch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    pub text: String,
    pub snapshot: Snapshot,
    pub events: Vec<Ev>,
    pub end_snapshot: Snapshot,
}

/// Cut 27 §2: a row's mean fires per send in each set on the paired panel (matched by the
/// row's conditions and verb; `sent_row` / `new_row` its index in each set, absent where the
/// set lacks it) — `≈ ±6 · R3 fires 4× more`. The rows whose fires moved most, first.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RowFires {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_row: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_row: Option<u32>,
    pub text: String,
    pub sent: f64,
    pub new: f64,
}

/// Cut 27 §2: the edit as a scene (`Game::divergence(prev)`): on the paired panel seed `seed`
/// (the sim index; both sets played the same dungeon) the first `tick` at which the active set
/// acts differently from `prev` (the sent set), on floor `depth`: `sent_row` / `new_row` what
/// each fired there, the two branches' next seconds (`sent` / `new`), and each branch's whole
/// run's end (`sent_end` / `new_end`: `lives · D9` vs `dies · D7`). The seed is the first whose
/// ends differ most (a death against none, then the deepest floor, then the exit), else the
/// first where the sets act differently at all. `moved` is the paired move's headline (the
/// largest |Δ| over the shaft and the ends, 0..1) and `inside` whether it sat within its ±.
/// `fires` the rows whose fire counts moved (`R3 fires 4× more`). None: the sets play alike
/// on every seed tried.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Divergence {
    pub seed: u32,
    pub tick: u32,
    pub depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_row: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_row: Option<u32>,
    pub sent_end: DivergenceEnd,
    pub new_end: DivergenceEnd,
    pub sent: DivergenceBranch,
    pub new: DivergenceBranch,
    pub moved: f64,
    pub inside: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fires: Vec<RowFires>,
    pub sims: u32,
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
            gun: None,
            entity: Entity { modifiers: None, id: 1, kind: "hero_fighter".into(), name: None, x: 1, y: 2, hp: 3, max_hp: 4, tags: vec![], ally: None, telegraph: None, cid: None, remembered: false },
            inv: vec![],
            weapon: Some("dagger".into()),
            armour: None,
            class: "fighter".into(),
            specialization:None,
            trait_: "brave".into(),
        };
        let s = serde_json::to_string(&h).unwrap();
        assert!(s.contains(r#""trait":"brave""#));
        assert!(s.contains(r#""kind":"hero_fighter""#));
        assert!(!s.contains("armour"));
    }
}

/// Cut 29 §4: the standing orders' switches kept on the lineage (`LineageState::orders`): `insure`
/// — the brought vault items are insured at each send the purse covers (on by default).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandingSwitches {
    #[serde(default = "yes")]
    pub insure: bool,
    /// Blind 1fb7786 (A, B: `forge −$20925 · purse +$823`, "losses I didn't choose"): what the apprentice may
    /// forge with (`FORGE_ORDERS`): `all` the spare purse, `half` half of each haul home (the rest is the
    /// player's to decide with), `off` nothing.
    #[serde(default = "forge_half")]
    pub forge: String,
    /// Cut 114 §3 (blind 77030eb, A: "the 8h absence came back with $1 496, four dead heirs and the best
    /// unchanged"): the scout's order at a wall that killed the last heirs (`WALL_ORDERS`, `tree::wall_hold`) —
    /// `bank` the heir home at its stairs until he is stronger (a retry every `WALL_RETRY` sends, its haul
    /// carried), `carry` the haul home from its stairs (secured, as a record's is) while the heir goes on,
    /// `push` nothing.
    #[serde(default = "wall_bank")]
    pub wall: String,
    /// Cut 118 §4: the apprentice's sinks after Kit complete (`feats::SINK_ORDERS`: `both` a ration a send and
    /// the tithe on the hour, `ration`, `tithe`, `off`). Adds only, never upkeep.
    #[serde(default = "sink_both")]
    pub sink: String,
    /// Cut 118 (owner amendment 2): the heir order — which offered heir succeeds a death (`feats::HEIR_ORDERS`:
    /// `answer` the killer, `strongest`, `surprise`). Set once; nobody is prompted.
    #[serde(default = "heir_answer")]
    pub heir: String,
    /// Cut 119 §4: the kennel keeper's order (`pets::KENNEL_ORDERS`: `breed` for the wall, `best`, `off`).
    #[serde(default = "kennel_breed")]
    pub kennel: String,
    /// Cut 120 §1: the Legacy order (`tree::LEGACY_ORDERS`); `off` for a save and a new lineage alike (`tree::NEW_LEGACY`, owner option B).
    #[serde(default = "order_off")]
    pub legacy: String,
    /// Cut 120 §2: the ranks order (`tree::RANKS_ORDERS`: `auto` · `off`); `off` for a save and a new lineage alike (`tree::NEW_RANKS`).
    #[serde(default = "order_off")]
    pub ranks: String,
    /// Cut 120 §4: the ascend order (`tree::ASCEND_ORDERS`: `off` · `on`), carried by the herald after the clear.
    #[serde(default = "order_off")]
    pub ascend: String,
    /// Cut 120 §3: the orders set `same for all` (`tree::SHAREABLE` keys): a later change to one follows on every
    /// bloodline (`Session::set_orders`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub shared: BTreeSet<String>,
    /// Cut 120: the orders already announced (`order:value`, once each, as a drill is).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub announced: BTreeSet<String>,
}

fn order_off() -> String {
    "off".into()
}

fn kennel_breed() -> String {
    crate::pets::KENNEL_ORDERS[0].into()
}

fn heir_answer() -> String {
    crate::feats::HEIR_ORDERS[0].into()
}

fn sink_both() -> String {
    crate::feats::SINK_ORDERS[0].into()
}

/// The apprentice's orders (`StandingSwitches::forge`), the default first.
pub const FORGE_ORDERS: [&str; 3] = ["half", "all", "off"];
/// Cut 114 §3: the scout's orders at a wall (`StandingSwitches::wall`), the default first.
pub const WALL_ORDERS: [&str; 3] = ["bank", "carry", "push"];

fn wall_bank() -> String {
    WALL_ORDERS[0].into()
}

fn forge_half() -> String {
    FORGE_ORDERS[0].into()
}

impl Default for StandingSwitches {
    fn default() -> Self {
        StandingSwitches { insure: true, forge: forge_half(), wall: wall_bank(), sink: sink_both(), heir: heir_answer(), kennel: kennel_breed(), legacy: order_off(), ranks: order_off(), ascend: order_off(), shared: BTreeSet::new(), announced: BTreeSet::new() }
    }
}

/// Cut 29 §2: one system of the curriculum (`Lineage.systems`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemInfo {
    pub id: String,
    pub open: bool,
    /// What opens it (≤ 3 words: `first death`, `meet Warlord`); empty for the day-0 systems.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub trigger: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub new: bool,
    /// Cut 118 §6: what lit it — its trigger (`slay Warlord`), `time` (the age fallback) or a feat (`feat: trial:
    /// Mother`) that lit it before its age.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lit_by: Option<String>,
}

/// Cut 29 §1: the oath draw (`drawOath()`): ◆`cost` for a fresh standing oath, repeatable;
/// `needs` the gate while shut (`meet Warlord`, `◆1 more`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct OathDraw {
    pub cost: u32,
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
}

/// Cut 29 §1: the next commission (`commission()`): a work of the lineage bought with gold — the
/// chronicle's monuments, the camp — policy-neutral (no sim reads it); `price` 10 forge units ×
/// 1.25ⁿ, `label` the work it builds.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Commission {
    pub price: i32,
    pub label: String,
    pub available: bool,
}

/// Cut 29 §4: the standing orders (`Lineage.orders`, `setOrders`): what an exit keeps (`keep`:
/// `best_weapon | best_armour | none`), what an unwatched cage takes (`cage`: `weapon | armour |
/// potion | scroll`), the floor a send starts on, the loadout's repeat, insuring the brought
/// items at each send the purse covers, and the apprentice's forge (`all · half · off`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StandingOrders {
    pub keep: String,
    pub cage: String,
    pub start: u32,
    pub repeat: bool,
    pub insure: bool,
    /// Blind 1fb7786: the apprentice's forge order (`all · half · off`; `FORGE_ORDERS`).
    #[serde(default = "forge_half")]
    pub forge: String,
    /// Cut 114 §3: the scout's order at a wall (`bank · carry · push`; `WALL_ORDERS`); absent on a set from a client
    /// that does not know it (the order stands).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall: Option<String>,
    /// Cut 118 §4: the apprentice's sink order (`both · ration · tithe · off`; `feats::SINK_ORDERS`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sink: Option<String>,
    /// Cut 118 (owner amendment 2): the heir order (`answer · strongest · surprise`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heir: Option<String>,
    /// Cut 119 §4: the kennel keeper's order (`breed · best · off`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kennel: Option<String>,
    /// Cut 120 §1: the Legacy order (`balanced · health · damage · armour · off`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy: Option<String>,
    /// Cut 120 §2: the ranks order (`auto · off`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranks: Option<String>,
    /// Cut 120 §4: the ascend order after the clear (`off · on`); absent: it stands.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ascend: Option<String>,
    /// Cut 120 §3: the orders set `same for all` (keys of `tree::SHAREABLE`); a set copies each named order to every
    /// bloodline now and on every later change. Absent: it stands; `[]` shares none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared: Option<Vec<String>>,
}

impl Default for StandingOrders {
    fn default() -> Self {
        StandingOrders { keep: "best_armour".into(), cage: "weapon".into(), start: 1, repeat: true, insure: true, forge: forge_half(), wall: Some(wall_bank()), sink: Some(sink_both()), heir: Some(heir_answer()), kennel: Some(kennel_breed()), legacy: Some(order_off()), ranks: Some(order_off()), ascend: Some(order_off()), shared: None }
    }
}

/// Cut 29 §3: the lineage's meters (`Lineage.meters`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct LineageMeters {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runs: Vec<crate::meters::MeterWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub night: Option<crate::meters::MeterWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_night: Option<crate::meters::MeterWire>,
}

/// Cut 29 §3: the live meters on a snapshot (`Snapshot.meters`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapMeters {
    pub run: crate::meters::MeterWire,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fight: Option<crate::meters::MeterWire>,
    #[serde(default)]
    pub fighting: bool,
}

/// Cut 29 §1 (E1): a wall's edit (`ReturnReport.wall`, `Lineage.wall`) — at a best depth held two
/// days, the plateau search's best one-row edit (up to two steps) from the lineage's own vocabulary:
/// `edits` their labels (`drop R6`, `R1 → hp < 90% → rest`), `rules` the set with them, `before` /
/// `after` the share of `sims` panel sends past the wall's floor (`depth`: the record, or the floor a
/// lucky record's set meets its wall on — `wall::wall_floor`). `start`: the lit waystone the
/// offer was measured from when it moves the sends' start there (`start D24`, the first edit; the
/// apply sets it) — a record reached once on a D1 send is measured where the sends meet the wall.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WallEdit {
    pub depth: u32,
    pub edits: Vec<String>,
    pub rules: RuleSet,
    pub before: f64,
    pub after: f64,
    pub sims: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,
}
impl Eq for WallEdit {}

/// Cut 29 §4: a kind a player's `throw` row names that the repeat lacks — offered on the repeat tile (`Lineage.repeat_added`,
/// the tile's `+ fire · for throw fire`): `row` the row's verb in short (`throw fire`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepeatAdd {
    pub kind: String,
    pub row: String,
}

/// Cut 29 §6: a companion that fell (`ReturnReport.fallen`): its name, kind, level, the floor, the
/// cause (`fell D12 to lurker`), the heir it served.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Fallen {
    pub name: String,
    pub kind: String,
    pub level: u32,
    pub depth: u32,
    pub why: String,
    pub heir: u32,
    /// Cut 119: lamed (sits out `pets::LAME_RUNS` runs, its level kept), not gone.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lamed: bool,
}

/// Cut 121 §1 (blind 1a7d834: Ascension restarted at D1 rats with nothing said of what changed): a numbered descent as
/// the camp names it (`Lineage.descent`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct AscensionWire {
    pub tier: u32,
    pub start: u32,
    pub hp_pct: u32,
    pub atk_pct: u32,
    pub modifiers: Vec<crate::endgame::ModifierInfo>,
}
