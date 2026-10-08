//! The game: lineage state, the live run, and the public API mirrored by the wasm bridge.
use crate::defs::{spawn_table, Cat};
use crate::descent::{Biome, Grudge, Route, ENDING_DEPTH};
use crate::gen::{generate, Floor};
use crate::geom::Pos;
use crate::hero::{mastery_card, xp_to_next, Class, Hero, Trait};
use crate::item::{describe, to_inv, Flavours, FloorItemWire, InvItem, Item};
use crate::monster::Monster;
use crate::rng::{hash_str, Rng};
use crate::rules::{Row, RuleSet, Vocabulary};
use crate::sifter::{Arc, Episode};
use crate::tiles::Overlay;
use crate::wire::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const SAVE_VERSION: u32 = 1;
pub const HERO_ID: u32 = 1;
/// History entries kept for verdict replays (one per 10 ticks ⇒ ≈ 100 ticks back).
/// Hero turns kept for verdicts. The replay starts at the last checkpoint where the hero still had
/// half its HP (up to this many turns back), so slow bleeds are attributable to a row, not to dice.
pub const HISTORY_TURNS: usize = 30;
pub const HISTORY_STRIDE: u32 = 10;

thread_local! {
    /// `without_history`: this thread's games keep no history ring.
    static NO_HISTORY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// `f` with the history ring off on this thread: no snapshot every `HISTORY_STRIDE` ticks, so a
/// death recorded meanwhile has no checkpoint (`DeathRec.t10` is `None`: no verdict, no patch can
/// be measured). For tools that play sends and never judge a death (the gate table's gold, forge,
/// exit and kitted jobs): the ring only ever feeds a death's checkpoint, and cloning the run every
/// ten ticks was a fifth of their time. Everything else a send does is the same, bit for bit.
pub fn without_history<R>(f: impl FnOnce() -> R) -> R {
    let was = NO_HISTORY.with(|c| c.replace(true));
    let r = f();
    NO_HISTORY.with(|c| c.set(was));
    r
}

/// RUNS_UI: the runs log's cap (`LineageState::run_log`) and the replay capsules kept in memory.
pub const RUN_LOG_CAP: usize = 60;
pub const CAPSULES: usize = 40;
static CAPSULES_OFF: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// RUNS_UI: harnesses that play real games at scale (the dayplayer, the gate table) keep no replay
/// capsules — a lineage clone per send they never read. Process-wide.
pub fn set_capsules(on: bool) {
    CAPSULES_OFF.store(!on, std::sync::atomic::Ordering::Relaxed);
}

/// RUNS_UI: a player's input inside a live run, replayed at its tick (`Game::replay`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    Choose(u32),
    Bail,
    /// Take control: on or off.
    Control(bool),
    /// Take control: the hero's next action.
    Act(Manual),
}

/// Take control: one hand-chosen action — a step (dx, dy: a foe there is attacked), a verb as the rules write
/// it (`drink heal`, `throw fire`, `descend`, `return`, `attack lowest` …), or a wait.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Manual {
    Step { dx: i32, dy: i32 },
    Verb { verb: crate::rules::Verb },
    Wait,
}
/// RUNS_UI: a real run's send, kept to re-simulate it — the game as `start_run` left it, the events
/// it left pending, and the inputs the run took.
#[derive(Clone, Debug)]
pub struct Capsule {
    pub id: u32,
    pub game: Box<Game>,
    pub pre: Vec<Ev>,
    pub inputs: Vec<(u32, Input)>,
}
/// The capsules (newest last). Never saved and never compared: a loaded game equals the one saved.
#[derive(Clone, Debug, Default)]
pub struct Capsules(pub VecDeque<Capsule>);
impl PartialEq for Capsules {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// RUNS_UI: what a later snapshot of the same floor showed, folded into its first (a replay viewer
/// rebuilds from the first): entities and items first seen since, at their first sighting, and the
/// tiles seen (web/src/ui/runlog.ts `floorSnapshot`).
fn fold_into(first: &mut Snapshot, later: &Snapshot) {
    if later.depth != first.depth {
        return;
    }
    for e in &later.entities {
        if e.id != first.hero.entity.id && !first.entities.iter().any(|x| x.id == e.id) {
            first.entities.push(e.clone());
        }
    }
    for i in &later.items {
        if !first.items.iter().any(|x| x.id == i.id) {
            first.items.push(i.clone());
        }
    }
    if later.seen.len() == first.seen.len() {
        for (a, b) in first.seen.iter_mut().zip(&later.seen) {
            *a |= *b;
        }
    }
}

/// RUNS_UI: FNV-1a 64 over each event's JSON, an exit's `line` and `trace` left out (a replay
/// never settles the exit), hex.
pub fn events_hash(events: &[Ev]) -> String {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for e in events {
        let j = match e {
            Ev::Exit { t, tier, loot_kept, .. } => serde_json::to_string(&Ev::Exit { t: *t, tier: tier.clone(), loot_kept: *loot_kept, line: None, trace: None }),
            _ => serde_json::to_string(e),
        }
        .unwrap_or_default();
        for b in j.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{h:016x}")
}
/// Cut 4: a hostile that stepped out of view is remembered (snapshot `remembered`, the hunt)
/// for this many hero actions after it was last seen.
pub const REMEMBER_ACTIONS: u32 = 10;
/// A run that cannot finish in this many ticks (≈ 3 h 20 min at 1×) comes home empty-handed
/// (tier `return`, yield ×0: a stalemate is not a policy; the DEFAULT set must yield nothing
/// over 8 h, and a stall that kept the return's share paid it). Cut 3: a D30 run needs ~75 000.
pub const MAX_TURNS_PER_RUN: u32 = 120_000;
/// Oscillation-guard firings on one floor before the run ends as `stalled` and gets a verdict.
pub const STALL_FIRES: u32 = 3;
/// QA on 1a2a4a9: the ledger's zero line when the re-pack ran short of gold.
pub const REPEAT_SHORT: &str = "repeat short";
/// A new heir's purse is topped up to this (one potion) so the camp is never at $0 after a death.
pub const WAKE_PAY: i32 = 40;
/// QA on 778fa1b: a death's `purse_full` is set only while the purse holds under this (twice
/// `WAKE_PAY`): just over the top-up line, where the missing `+$N wake` needs a word.
pub const PURSE_FULL_BAND: i32 = 2 * WAKE_PAY;
/// Cut 3: rule rows with every row unlock (`row5`–`row10`).
pub const MAX_ROWS: usize = 10;
/// Cut 12 §1: card rows sit outside the player's cap — one per owned card (8 tactic, 4 tier 2,
/// 4 mastery); the per-row tallies are sized for both.
pub const MAX_CARD_ROWS: usize = 16;
/// Cut 30 §2: a compiled set's package rows (drills, a stance, tactics, a temperament) sit outside
/// the cap too.
pub const MAX_PKG_ROWS: usize = 24;
pub const ROWS_TOTAL: usize = MAX_ROWS + MAX_CARD_ROWS + MAX_PKG_ROWS;
/// Cut 3: the ascension variants, in the order they are offered.
pub const VARIANTS: [&str; 4] = ["no_rest", "short_list", "bones_only", "hunted"];
/// Energy needed to act; actors gain `speed` per tick.
pub const ACT_ENERGY: i32 = 100;
pub const TICKS_PER_TURN: u32 = 10;
fn default_bloodline_id()->u32 {1}
pub const MAX_LEVEL: u32 = 10;
/// Gold from loot and salvage is divided by this (Addendum B/D economy pass).
pub const GOLD_DIVISOR: i32 = 4;
/// Cut 2 §1: camp rest after an expedition lasts half as long as it did (Cut 12 §5; Cut 2: as
/// long), capped at 30 min and never shorter than the wake (a two-minute `hp<50 → return`
/// sortie would otherwise farm hundreds of runs a day); a death is followed by a fixed
/// 20-minute wake. Ticks (10/s).
pub const REST_CAP_TICKS: u32 = 30 * 60 * 10;
pub const REST_MIN_TICKS: u32 = 20 * 60 * 10;
pub const WAKE_TICKS: u32 = 20 * 60 * 10;
/// Cut 2 §1: an egg hatches after this many rests (1 with the incubator).
pub const EGG_RESTS: u32 = 3;
/// Cut 2 §2: bones piles kept per lineage (oldest expires). Cut 13: eight — three read as
/// `bones: 3 on the floor` under a chronicle of seventeen heirs who each left some.
pub const BONES_MAX: usize = 8;
/// Cut 2 §5: kills of a kind before it counts as studied.
pub const STUDIED_KILLS: u32 = 5;
/// Cut 5 §2: lineage chronicle lines kept (one per ended heir).
pub const CHRONICLE_CAP: usize = 40;
/// Cut 5 §4: a watched run's opened vault waits this many ticks for `choose` before the
/// preference picks; offline and simulated runs pick at once.
pub const VAULT_GRACE: u32 = 50;
/// Cut 8B §3: share of lineages whose first stray waits on D2 or D3.
pub const FIRST_STRAY_PCT: u32 = 80;
/// Cut 9 §5: hero turns on an exit's trace (bank / return / death lines alike). Cut 11 §3:
/// 10 (was 5; "survivor runs keep only five turns").
pub const EXIT_TRACE_LEN: usize = 10;
/// Cut 9 §7: deaths whose record the graveyard still points at (`Grave.death_id`); the
/// engine keeps at least this many records through a save.
pub const KEPT_DEATHS: usize = 5;
/// Cut 9 §6: absences whose reel pairs (threat, resolution) are remembered for the dedupe.
pub const REEL_ABSENCES: usize = 3;
/// Cut 5 §4: a stray (a lost heir's companion gone wild) tames at this chance.
pub const STRAY_TAME: u32 = 60;
/// Cut 5 §4: the shrine's price — a fifth of max HP for the run.
pub const PRAY_COST_PCT: i32 = 20;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExitTier {
    Bank,
    Return,
    Death,
}

impl ExitTier {
    pub fn name(self) -> &'static str {
        match self {
            ExitTier::Bank => "bank",
            ExitTier::Return => "return",
            ExitTier::Death => "death",
        }
    }
    pub fn pct(self) -> i32 {
        match self {
            ExitTier::Bank => 100,
            ExitTier::Return => 60,
            ExitTier::Death => 0,
        }
    }
}

/// Cut 2 §2: a dead heir's inventory and kit, waiting on the floor it died on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Bones {
    pub heir: u32,
    pub depth: u32,
    pub items: Vec<Item>,
    /// The grave had deeds: recovering it is a highlight.
    #[serde(default)]
    pub named: bool,
    /// Cut 26 §1: the biome of the floor the heir fell on (`None`: the base order's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<Biome>,
}

impl Bones {
    /// Cut 26 §1: whether the pile lies on this floor (its depth, its biome).
    pub fn lies_on(&self, depth: u32, biome: Biome) -> bool {
        self.depth == depth && self.biome.unwrap_or(crate::descent::biome_for(self.depth)) == biome
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FloorItem {
    pub pos: Pos,
    pub item: Item,
}

/// Everything about one expedition. Cloned per turn into the history ring (for verdicts).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Run {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gun_reload: Option<crate::firearm::Reload>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gun_skills: Option<crate::firearm::Skills>,
    pub id: u32,
    pub heir: u32,
    pub started_turn: u64,
    /// Numbered difficulty as played, independent of later camp selections.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub difficulty: u32,
    pub rng: Rng,
    pub depth: u32,
    /// Cut 21 §1: the floor the send started on (1, or the lit waystone it paid for).
    #[serde(default = "default_start")]
    pub start: u32,
    /// QA on a946e04 (qaT: run 6's report `+$71 banked · −$40 spent` left out the send's
    /// `−$50 waystone D5`): the toll this send paid (0 from D1 or on the night's pass).
    #[serde(default)]
    pub toll: i32,
    /// Cut 27 §1: the passage paid at this run's waystone start (coins, into the purse at the
    /// send): the skipped floors' gold when the set clears them ≥ 95 % (`forecast::passage_for`).
    #[serde(default)]
    pub passage: i32,
    /// QA on a946e04 (qaT: a short purse sent the run from D1 unsaid): the waystone the send
    /// wanted and did not start on (the toll short, or unlit) — `ExitLine.start_short`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_short: Option<u32>,
    pub floor: Floor,
    pub hero: Hero,
    pub trait_: Trait,
    /// Cut 30: the heir's traits on this run and the gift's state (`traits::Worn`, `GiftRun`).
    #[serde(default, skip_serializing_if = "crate::traits::Worn::is_empty")]
    pub worn: crate::traits::Worn,
    #[serde(default)]
    pub gift: crate::traits::GiftRun,
    pub monsters: Vec<Monster>,
    pub items: crate::shared::Shared<Vec<FloorItem>>,
    pub overlays: Vec<Overlay>,
    pub turn: u32,
    pub floor_turn: u32,
    pub alert: i32,
    pub loot: i32,
    pub next_id: u32,
    pub next_item_id: u32,
    pub kills_floor: u32,
    /// (turn, kind, depth) — the lineage's kills (XP, renown, the bestiary); summoned foes
    /// are counted apart (`summoned_kills`).
    pub kills: crate::shared::Shared<Vec<(u32, String, u32)>>,
    /// QA on 23ed91f: summoned foes cut down this run (no XP, renown or bestiary count).
    #[serde(default)]
    pub summoned_kills: u32,
    pub brought: crate::shared::Shared<Vec<u32>>,
    pub trace: Vec<crate::shared::Shared<TraceTurn>>,
    pub notes: crate::shared::Shared<Vec<(u32, String)>>,
    pub over: Option<ExitTier>,
    pub death_cause: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_modifiers: Option<crate::endgame::Modifiers>,
    pub death_blow: i32,
    /// QA on 0c6e126: the tick of the killing blow (`Trace.blow`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub death_t: Option<u32>,
    /// Cut 10 §3: the HP the hero was short of taking the killing blow (`Death.margin`: `3 hp
    /// short` — the blow overshot by two, one more would have held).
    #[serde(default)]
    pub death_short: i32,
    pub hurt_since_action: bool,
    pub hurt_last: bool,
    pub kill_since_action: bool,
    pub kill_last: bool,
    pub seen_ids: crate::shared::Shared<BTreeSet<u32>>,
    pub new_seen: bool,
    pub telegraphs_now: Vec<String>,
    pub low10_t: Option<u32>,
    pub low20_t: Option<u32>,
    pub near_deaths: Vec<u32>,
    pub gambles: Vec<(u32, String, bool)>,
    pub gambles_survived: Vec<(u32, String)>,
    /// QA on a946e04 (qaT): the HP the last gamble's harm (poison, its gas, its fire) has taken
    /// inside `trace::GAMBLE_WINDOW` (`turn::damage_hero`; reset at each gamble).
    #[serde(default)]
    pub gamble_harm: i32,
    /// QA on 778fa1b (qaV: `fire · D6 · GAP`, the trace's `R3 card last stand` 10 → 3 → 0 — the
    /// card's own `throw fire` at an adjacent foe burned him): the hero's last throw whose harm
    /// reached him (a fire or caustic blast over his tile, a poison thrown back) — (tick, kind)
    /// — and the HP that harm has taken inside `trace::GAMBLE_WINDOW` (`turn::damage_hero`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub own_throw: Option<(u32, String)>,
    #[serde(default)]
    pub own_throw_harm: i32,
    /// QA on 778fa1b (qaV: finds that ended in no named place): every unit the run found —
    /// (item id, kind), one entry per unit (a leash merged into a stack is the stack's id) —
    /// and the units that left the pack on the way, with where (`used` / `left` / `stolen`).
    /// The exit reads them into `ExitLine.found` (`Game::found_rows`).
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub found_units: crate::shared::Shared<Vec<(u32, String)>>,
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub found_gone: crate::shared::Shared<Vec<(u32, String, String)>>,
    /// Cut 22 §3: a forecast's sim (`forecast::simulate_one`): each floor below the first draws
    /// from a stream of its own, (run seed, depth) — `turn::descend`. A send, and any replay of
    /// one, goes on drawing from the run's stream.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub floor_streams: bool,
    /// Cut 26 §1: the route this send descends — the active set's at the send (`RuleSet.route`).
    #[serde(default, skip_serializing_if = "Route::is_base")]
    pub route: Route,
    /// Cut 26 §2: the deepest fork whose two stairs this run has seen (`facts::fork_seen`).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub fork_seen: u32,
    pub stolen: Vec<(u32, String)>,
    /// Cut 20 §1: the ids of the items thieves took this run, and what was taken back from a
    /// killed thief's drop (turn, label).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stolen_ids: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recovered: Vec<(u32, String)>,
    /// QA on e75ec29: each theft's item id and its label at the theft — the ones still in
    /// `stolen_ids` at the exit are what the run lost to thieves (`Batch.stolen`, the exit
    /// line's `· stolen heal`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stolen_labels: Vec<(u32, String)>,
    /// QA on a946e04 (qaS: STOLEN `black potion?` while LEARNED said `confusion (black)`;
    /// `leash ×4` beside `leash (2)`): each theft's item id and kind — the report names what
    /// thieves kept by kind, labelled when it is read (`LineageState::wire_name`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stolen_kinds: Vec<(u32, String, i32)>,
    pub ally_lost: Vec<(u32, String)>,
    pub ally_freed: Vec<u32>,
    pub boss_kills: crate::shared::Shared<Vec<(u32, String)>>,
    /// Cut 28 §1: what an oath reads of a run — the potions the hero drank (any kind, a trait's
    /// sip included), whether he rested, the bosses fire hurt (their kinds).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub drinks: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rested: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub burned: Vec<String>,
    /// Cut 28b: the oath sworn at the send (its terms, read as the run goes), what the run said of
    /// it (`Ev::Oath`, once: kept, and the cause — `R2 return` broke it, empty when missed), the
    /// `return` a row committed the walk home to (a `no return` oath is broken there), and a band
    /// boss seen this run (the board opens: `LineageState::oath_open`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<crate::oath::OathState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath_said: Option<(bool, String)>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub home_return: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub wall_seen: bool,
    /// Cut 29 §3: the run's meters (the run, the fight in progress; `meters.rs`) — real runs only.
    #[serde(default, skip_serializing_if = "crate::meters::RunMeters::is_empty")]
    pub meters: crate::meters::RunMeters,
    /// Cut 29 §6: the grudges this run tamed (by name) — closed as tamed at the exit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tamed_grudges: Vec<String>,
    /// Cut 29 §1: the lineage owns `route2` (an oath's reward) — the D9 fork is seen.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub route2: bool,
    /// Cut 28 §4 (AU: a one-slot vault salvaged a caged sword +1, no choice): the items taken from
    /// a cage this run — a return's cut never takes them; they reach the keep sheet.
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub caged: crate::shared::Shared<Vec<u32>>,
    /// Cut 28 §1: (floor, tick) each time the run went deeper than it had been (the `swift` oath).
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub depth_t: crate::shared::Shared<Vec<(u32, u32)>>,
    /// Cut 28 §2: the hero's max-hp steps this run, oldest first (`Trace.max_steps`).
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub max_steps: crate::shared::Shared<Vec<crate::wire::MaxStep>>,
    pub drank_heal: bool,
    pub melee_used: bool,
    pub boss_seen_t: Option<u32>,
    /// Cut 30 §1: the band bosses this run saw (a meeting: drills, scars), and the scars the send
    /// carried (boss → percent of max hp off).
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub bosses_met: crate::shared::Shared<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scars: Vec<(String, u32)>,
    pub hurt_since_boss: bool,
    pub row_fired: Vec<u32>,
    pub renderable_events: u32,
    pub ended: bool,
    pub max_depth: u32,
    /// Run-clear: the lineage's record when this run was sent — the exit line's `new_best` reads against it
    /// (a record the run itself secured on its way down is still this run's).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_at_send: Option<u32>,
    pub trophies_run: crate::shared::Shared<Vec<String>>,
    /// Companion records active in this run (party members and new tames).
    pub companions: Vec<Companion>,
    pub recalled: Vec<u32>,
    pub tamed: Vec<(u32, String)>,
    pub lost_companions: Vec<(u32, String)>,
    /// Cut 27 §5: each companion that fell this run and how (`fell D7 to ogre`) — `Lost.why`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fell_why: Vec<(String, String)>,
    /// Item ids of camp supplies (never kept back).
    /// QA on 0c6e126 (qaZ: `new find: leash`, the free supply back from bones): the kinds the
    /// send packed (its supplies), never a find of the run.
    #[serde(default)]
    pub packed: crate::shared::Shared<Vec<String>>,
    /// QA on 524827b (qaAB: a bought 2nd leash, $30, merged into the kennel's free stack at the
    /// pack and went back to the kennel as the free one at a return — gone, run after run): the
    /// price paid for each bought leash packed into the stack (the stack's units carry no mark).
    #[serde(default)]
    pub bought_leashes: Vec<i32>,
    pub supplies: Vec<u32>,
    /// Taunt: foes ignore companions for this many ticks.
    pub taunt_t: i32,
    /// Cut 13: the archer the `kite archers` card last broke line of sight from, and when —
    /// the card charges it the second time inside `KITE_WINDOW` (it held its ground).
    #[serde(default)]
    pub kited: Option<(u32, u32)>,
    /// QA on 23ed91f (qaL run 5): the `pack break` card went for a pack that would not come
    /// (it hung back past the hold, or shoots) — until this action the card does not fall back
    /// to a corridor, which would undo the step it just took.
    #[serde(default)]
    pub pack_go: u32,
    /// Cut 27 §4: the tile a card (`pack break`, `thief guard`) last fell back to a corridor from —
    /// back on it (a chore walked the hero out again), the card does not fall back again.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_fell: Option<Pos>,
    /// Cut 27 §4 (AS: `R5 free ↔ descend` stalled a D9 send): the captive a `free captive` row
    /// set out for (its id) — the row's `on see: captive` holds on the way while it lives
    /// chained, though a corner hides it (the step toward it broke the sight line, the `descend`
    /// chore stepped back, the row stepped in again).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freeing: Option<u32>,
    /// Cached BFS field from the hero (recomputed when the hero moves).
    #[serde(skip)]
    pub hero_dist: Vec<i32>,
    #[serde(skip)]
    pub hero_dist_pos: Option<Pos>,
    /// `hero_dist`'s flood (FIFO queue and head) while it is still partial: a monster's step
    /// reads the field only as far as its own tile (`turn::hero_dist_to`), the rest is flooded
    /// on demand — the values are the full flood's (a tile's distance is final when found).
    #[serde(skip)]
    pub hero_flood: Vec<u32>,
    #[serde(skip)]
    pub hero_flood_head: usize,
    /// Ids visible at the last vision pass (to skip unchanged passes).
    #[serde(skip)]
    pub last_visible: Vec<u32>,
    #[serde(default)]
    pub idle_actions: u32,
    /// Consecutive cowardly retreats (the trait yields after three).
    #[serde(default)]
    pub cowardly_streak: u32,
    /// Corridor hold: distance of the nearest foe at the last hold, and how long it stayed put.
    #[serde(default)]
    pub hold_dist: i32,
    #[serde(default)]
    pub hold_streak: u32,
    /// Hero actions taken this run (the clock for the guards below).
    #[serde(default)]
    pub actions: u32,
    /// Foes the hero has given up on: id → action at which they may be engaged again.
    #[serde(default)]
    pub ignored: BTreeMap<u32, u32>,
    /// Chase progress: (target id, last distance, attempts without closing).
    #[serde(default)]
    pub chase: Option<(u32, i32, u32)>,
    /// Last 12 hero positions and the action of the last damage dealt or taken.
    #[serde(default)]
    pub recent_pos: Vec<Pos>,
    #[serde(default)]
    pub last_damage_action: u32,
    /// Times the oscillation guard fired on this floor; three ends the run as `stalled` (Cut 7 fix).
    #[serde(default)]
    pub stuck_fires: u32,
    /// Oscillation guard: rows that target foes are suppressed until this action.
    #[serde(default)]
    pub stuck_until: u32,
    /// Cut 13 §1: the tick of the first guard on this floor (the stall verdict's checkpoint)
    /// and the guard's moment (`goblin archer, no path`) — the stall's cause, on the record,
    /// the reel line and the chronicle alike.
    #[serde(default)]
    pub stuck_first_t: Option<u32>,
    #[serde(default)]
    pub stuck_cause: Option<String>,
    /// Cut 18 §4: the row a rules' loop named as the stall's cause (`R2 retreat ↔ explore` →
    /// 1), for the stall verdict's first patch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stuck_row: Option<i32>,
    /// Cut 27 §4: the oscillation guards this run whose window was a card taking turns with a
    /// chore (`R8 pack ↔ pick up`: the card's sub-rows are the engine's) — recovered or not.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub card_loops: u32,
    /// (the loops' causes, for the metrics' breakdown; not saved)
    #[serde(skip)]
    pub loop_causes: Vec<String>,
    /// QA on 1a2a4a9 (qaP: `returning` at 6/38, then hp came back, the row stopped holding
    /// and he went on down to D8): a return is a commitment. The row (set index) whose
    /// `return` / `bank` acted: from then on the walk home replaces the chores — when no row
    /// acts, the hero steps toward the up-stairs and exits there at that row's tier
    /// (`ai::chore`) — while every row still acts as written (a heal, a fight on the way; the
    /// return row itself walks when it holds), and nothing takes him down a floor (a
    /// `descend` refuses `going home`; the chores never descend). Until he exits or dies.
    /// Sims and verdict replays play the same rule (it is the run's own state).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homeward: Option<i32>,
    /// Cut 23 §2: when the walk home was committed — (tick, hp %, hp) — for the death mix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home_at: Option<(u32, i32, i32)>,
    /// The committing row was a `bank` (the walk exits at 100 %), else a `return`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub homeward_bank: bool,
    /// QA on 1a2a4a9: the kinds the send's re-pack could not pay for (`LineageState::
    /// repeat_short` at the send) — a row that reaches for one reads `repeat short`, not
    /// `never found`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_short: Vec<String>,
    /// Trait pre-emption clock: the action of the last trait deviation.
    #[serde(default)]
    pub trait_last: Option<u32>,
    /// Cut 13 §2: trait deviations on this floor (a trait overrides a row at most once per
    /// floor; reset at the stairs).
    #[serde(default)]
    pub trait_floor: u32,
    /// Cut 13 §3: kinds this run used to no effect (`Ev::Use` outcome `nothing`, a heal drunk
    /// at full HP): `restock` does not rebuy them for the next run.
    #[serde(default)]
    pub wasted_kinds: Vec<String>,
    /// Consecutive pick_up choices and the inventory size when they started.
    #[serde(default)]
    pub pickup_streak: u32,
    #[serde(default)]
    pub pickup_inv: usize,
    #[serde(default)]
    pub items_until: u32,
    /// Cut 25 §3: floor items the chores give up on this floor (a `pick up` that stood on one and
    /// left it lying: the pack would not take it). Cleared at the stairs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skip_items: Vec<u32>,
    /// QA on 524827b (seed 30's D5: `pick up` chores alternating with an attack row, 60 on one
    /// floor with nothing taken): the floor's `pick up` chores since one last took something.
    /// Cleared at the stairs.
    #[serde(default)]
    pub pickup_dry: u32,
    /// Cut 25 §3: the drain stretch under way (`Ev::Drain`'s word) — `None` once a foe is in view
    /// or at the stairs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drain_on: Option<String>,
    /// Cut 25 §3: the blows on the hero since his last action (`Trace.blows`), oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blows: Vec<crate::wire::TraceBlow>,
    /// QA on 524827b: the hp lost since the hero was last at full hp, per cause (`Trace.hp_lost`).
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub hp_lost: crate::shared::Shared<Vec<(String, i32)>>,
    /// QA on 308f045 (qaAC: `since full hp` summing to 72 on a 36-hp hero): the hp he had when
    /// the count began (`hp_lost`'s first blow) — what the losses beyond it were healed from.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub hp_lost_from: i32,
    /// Where hostiles were last seen (id → position, action), so pathing does not flip
    /// between "blocked" and "open" as a corridor foe drifts in and out of view.
    #[serde(default)]
    pub known_foes: BTreeMap<u32, (Pos, u32)>,
    /// Same-row loop guard: (row, consecutive firings) and the row suppressed until an action.
    #[serde(default)]
    pub row_streak: (i32, u32),
    #[serde(default)]
    pub row_suppressed: (i32, u32),
    /// QA on 778fa1b (qaV: `R4 retreat` / `R6 attack` taking turns before two ogres for
    /// minutes at 21/42): the policy retreats of the current engagement — how many, and the
    /// nearest foe after the last (`turn::outpaced_guard`).
    #[serde(default)]
    pub retreats: (u32, i32),
    /// QA on 778fa1b (qaV): foe-facing row actions since blood was last drawn either way, the
    /// retreats among them, and the run's longest such stretch with `turn::DANCE_MOVES`
    /// retreats (`Batch.dances`: runs whose stretch reached `turn::DANCE_ACTIONS` — the cohort
    /// sets' dance gate).
    #[serde(default)]
    pub bloodless: (u32, u32, u32),
    /// Cut 24 §1 (AL: the Warlord > 4 min on `attack nearest`, the boss bar full; AK: ~100 s of
    /// retreat ↔ pack break before archers): the hero's actions with a foe in view since the
    /// fight last moved — a real foe (not a summon) lost HP, or the hero lost HP to anything but
    /// a summon — the actions since a foe was last in view, and the run's longest stretch (this
    /// or a boss's `boss_still`; `turn::NOHP_ACTIONS` ends one: `turn::nohp_guard`).
    #[serde(default)]
    pub nohp: (u32, u32, u32),
    /// Cut 24 §1: the boss in view — (id, its HP, the hero's actions since that HP last moved).
    /// `turn::BOSS_STILL` of them and the boss wins its fight: the hero is driven off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss_still: Option<(u32, i32, u32)>,
    /// Cut 24 §1: the boss that drove the hero off (its kind), for the exit's `no counter`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub driven_off: Option<String>,
    /// Cut 24 §1: driven off with no way home written (no `return` / `bank` row in play): the
    /// carry is dropped — the game's exit pays nothing, as a stall's does (DEFAULT yields 0).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub driven_lost: bool,
    /// Cut 30.5 (the owner, 2026-10-02: a new record never ends the run): the gold secured at this run's
    /// checkpoints (each new deepest floor past the record secures the carry so far, as a waystone would — safe
    /// whatever the exit, a death's included), the record the run left with, and its deepest checkpoint.
    #[serde(default)]
    pub secured: i32,
    #[serde(default)]
    pub record0: u32,
    #[serde(default)]
    pub record_mark: u32,
    /// Cut 24 §1: the rows a dance rested (`turn::nohp_guard`) and the action they rest until.
    #[serde(default)]
    pub rows_rested: (Vec<i32>, u32),
    /// Rests taken on this floor (the rest clock).
    #[serde(default)]
    pub rests: u32,
    /// The current hero strike is aimed (`attack tag:boss` / bash): goes through shield walls.
    #[serde(default)]
    pub aimed: bool,
    /// Cut 2 §2: the lineage's bones piles (copied at start) and the heirs recovered this run.
    #[serde(default)]
    pub bones: crate::shared::Shared<Vec<Bones>>,
    #[serde(default)]
    pub bones_found: Vec<u32>,
    /// Cut 2 §5: kills per kind this run were counted into the lineage's ledger up to here.
    #[serde(default)]
    pub kills_counted: usize,
    /// Hit the run cap: comes home as `return` with no yield.
    #[serde(default)]
    pub timed_out: bool,
    /// The rule row whose verb ended the run (`return` / `bank`), for the stall verdict.
    #[serde(default)]
    pub exit_row: Option<i32>,
    // Cut 3
    /// The last noise on the floor (tile, tick): rests, melee and shouts. Blind hunters go there.
    #[serde(default)]
    pub noise: Option<(Pos, u32)>,
    /// The hero's last three verbs while the Mirror King watched (his `mirror`).
    #[serde(default)]
    pub verb_ring: Vec<String>,
    /// The verb of the hit the current action landed, if any (fills the ring).
    #[serde(default)]
    pub last_hit_verb: Option<String>,
    /// Blind foes seen on this floor (ids), for `noise_discipline`.
    #[serde(default)]
    pub blind_seen: Vec<u32>,
    /// `reflect_read` put the pack's bow up: the melee weapon set aside, back once no mirror
    /// is in view.
    #[serde(default)]
    pub bow_swap: Option<Item>,
    /// QA on 778fa1b (qaU: `carry $61 −$37 swapped` on the strip, in no ledger): the carried
    /// gold (coins) this run's pack swaps took off — a find taken in the place of a dearer
    /// carried one (`turn::pickup_here`): the exit line's `swapped`.
    #[serde(default)]
    pub swapped: i32,
    /// QA on 0c6e126 (qaY: `carry $135 −$5 swap → waxen scroll?` read as a price; the death
    /// line's `−$5 swapped` named no item): what each costly swap left on the floor, by its
    /// label at the time (`axe`), in order — the strip's and the exit line's `left axe`.
    #[serde(default)]
    pub swap_left: Vec<String>,
    /// Visible hostiles whose `reflect_melee` tag is known: the chores path around them
    /// (never within a tile), as they path around water. Refreshed each hero action.
    #[serde(default)]
    pub mirrors: Vec<Pos>,
    // Cut 4
    /// Item value picked up this run, before `GOLD_DIVISOR`; `loot` is gold (`loot_add`).
    #[serde(default)]
    pub loot_raw: i32,
    /// The lowest HP the hero fell to this run (the reel's setup).
    #[serde(default)]
    pub low_hp: i32,
    /// The first rule row that fired after the hero fell to ≤ 20 % on this floor (the row
    /// the chronicle credits when the floor is survived).
    #[serde(default)]
    pub saved_by: Option<i32>,
    /// Cut 24 §5: the HP at which `saved_by`'s row fired (the lowest a row fired at).
    #[serde(default)]
    pub saved_low: i32,
    /// The foe the last foe-targeting row acted on, and that row: the hunt continues to its
    /// last-seen tile when it steps out of view.
    #[serde(default)]
    pub last_target: Option<u32>,
    #[serde(default)]
    pub hunt: Option<(u32, i32)>,
    /// The first row this action whose conditions held but whose verb could not execute
    /// (`R1 retreat ✗ no path`), and the callout last shown for it (once per streak).
    #[serde(skip)]
    pub blocked_now: Option<String>,
    #[serde(default)]
    pub blocked_last: Option<String>,
    // Cut 5
    /// §1: the live episode and the closed ones.
    #[serde(default)]
    pub arc: Arc,
    #[serde(default)]
    pub episodes: crate::shared::Shared<Vec<Episode>>,
    /// §3: the hero's last line and the tick the current fight began (no lines in its first ten).
    #[serde(default)]
    pub voice_t: Option<u32>,
    #[serde(default)]
    pub fight_t: Option<u32>,
    /// §4: situations met this run (`shrine | vault | nest | stray`, with the tick).
    #[serde(default)]
    pub situations: crate::shared::Shared<Vec<(u32, String)>>,
    /// §4: this floor's vault cage (three items) and, once opened, the choice waiting
    /// (tick opened, items) for `choose` or the preference.
    #[serde(default)]
    pub vault_cage: Vec<Item>,
    #[serde(default)]
    pub vault_choice: Option<(u32, Vec<Item>)>,
    /// §4: the shrine was used this run; the row it lent (`pray row`).
    #[serde(default)]
    pub prayed: bool,
    #[serde(default)]
    pub lent_row: Option<Row>,
    /// §4: a stray was placed this run; strays tamed (their names, back from the lost list).
    #[serde(default)]
    pub stray_placed: bool,
    /// Cut 8B §3: the lineage's first stray, (depth, name), while nothing has been tamed.
    #[serde(default)]
    pub first_stray: Option<(u32, String)>,
    #[serde(default)]
    pub strays_tamed: Vec<String>,
    /// §5: `bail()` queued a `return` for the next hero action.
    #[serde(default)]
    pub bail: bool,
    /// Take control (owner 2026-10-08, a secondary mode): the player chooses the hero's actions in the watched run —
    /// the world waits at each of his turns for `act` (`awaiting`), the rules rest until `take_control(false)`.
    /// Never in a sim or an absence (the rules take it back there: `Game::tick_inner`).
    #[serde(default)]
    pub manual: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manual_act: Option<Manual>,
    #[serde(default)]
    pub awaiting: bool,
    /// §4: sleeping dens the hero has seen (the chores keep two tiles clear of them, and
    /// their gold is not the chores' to fetch) — unless greed has been tempted this floor.
    #[serde(skip)]
    pub dens: Vec<Pos>,
    #[serde(default)]
    pub tempted: bool,
    /// Cut 6 §3: why each row above the one that acted did not fire this action (moved onto
    /// the trace turn by `hero_action`).
    #[serde(default)]
    pub rows_why: Vec<RowWhy>,
    // Cut 7
    /// §4: the tick `Ev::Ending` was last emitted (once per 100 ticks).
    #[serde(default)]
    pub ending_t: Option<u32>,
    /// §3: the band situations' bookkeeping — den thefts outstanding (item ids), gas damage
    /// taken on this floor, the lit shrine, and the situations passed this run
    /// (`den | lock | captive | hunger`).
    #[serde(default)]
    pub den_stolen: Vec<u32>,
    #[serde(default)]
    pub gas_dmg_floor: i32,
    /// §3: the tick the last lock bloat burst (gas within 40 ticks of it is the lock's).
    #[serde(default)]
    pub lock_last_pop: u32,
    #[serde(default)]
    pub lit: bool,
    #[serde(default)]
    pub passed: crate::shared::Shared<Vec<String>>,
    /// §3: the gas lock's tiles (bloats swell on sight), for the `on_see: lock` token.
    #[serde(default)]
    pub lock_tiles: Vec<Pos>,
    /// §3: the row acting now names the den (`on_see: den` / `foe_tag thief`): its melee may
    /// raid the sleeping thieves. Any other row walks past them.
    #[serde(skip)]
    pub raiding: bool,
    /// The row whose verb is acting right now (−1 between rows: a chore, a trait), so a
    /// provenance entry can name it (`R4 drank heal at 33/36 hp`).
    #[serde(skip)]
    pub acting_row: i32,
    /// §3: the den's sleepers (tiles the chores walk round; a blow on one is a raid).
    #[serde(skip)]
    pub sleepers: Vec<Pos>,
    // Cut 12
    /// §4: the floor's one situation from D3 (`nest` · `den` · `lock` · `captive` · `shrine`
    /// · `vault` · `stray` · `hunger`), the previous floor's (never rolled twice running),
    /// the run's seed (each band's flagship floor is drawn from it), and a kind forced on
    /// the next floor (trials and probes; never saved).
    #[serde(default)]
    pub floor_twist: crate::shared::Shared<Option<String>>,
    #[serde(default)]
    pub last_twist: crate::shared::Shared<Option<String>>,
    #[serde(default)]
    pub seed: u64,
    #[serde(skip)]
    pub next_twist: Option<String>,
    // Cut 19
    /// §5: the lineage has lost to a den before (`LineageState::den_thefts` and the `den`
    /// fact, at the send): this run's dens pounce on ≤ 1 in 3 floors; and the den snatches
    /// this run suffered (added to the lineage at the exit).
    #[serde(default)]
    pub den_thin: bool,
    #[serde(default)]
    pub den_snatches: u32,
    /// Cut 20 §1: dens that woke (pounced) this run.
    #[serde(default)]
    pub den_wakes: u32,
    /// Cut 20 §1: this floor's den woke after the run's one theft and bolted empty-handed —
    /// not a pass (nothing answered it).
    #[serde(default)]
    pub den_bolted: bool,
    /// Cut 20 §5: the lineage's bounty floor at the send (gold ×2, one next-tier item), and
    /// the coins picked up on it this run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<u32>,
    #[serde(default)]
    pub bounty_gold: i32,
    /// §5: the grudges (by name) this run avenged — the lineage marks them at the exit, and a
    /// later kill of the same named foe reads `slain`.
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub avenged: crate::shared::Shared<Vec<String>>,
    // Cut 24 §2
    /// Named foes (a stray, the first jackal) met in the last two runs (`LineageState::
    /// named_met`): they rest this run — a named foe appears at most every third run unless it
    /// holds a grudge. And the names this run placed.
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub named_rest: crate::shared::Shared<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub named_placed: Vec<String>,
    /// The floor events' lines (by kind: `shrine`, `lock`, `omen:fens` …) shown in the last two
    /// runs and this one, oldest first (`chronicle::variant`), and this run's picks.
    #[serde(default, skip_serializing_if = "crate::shared::map_is_empty")]
    pub event_recent: crate::shared::Shared<BTreeMap<String, Vec<u8>>>,
    #[serde(default, skip_serializing_if = "crate::shared::vec_is_empty")]
    pub event_used: crate::shared::Shared<Vec<(String, u8)>>,
    /// The facts this run learned, in order (`facts::learn`; `ExitLine.news`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub learned: Vec<String>,
    // Cut 16
    /// §1: the freshness of the floors this run may generate, in permille by depth (only
    /// thinned depths; `LineageState::thin_map` at the send) — a floor's gold piles and item
    /// budget are kept at this rate (`populate_floor`).
    #[serde(default)]
    pub thin: crate::shared::Shared<BTreeMap<u32, u32>>,
}

impl Run {
    /// Cut 26 §2: the fork at this floor's down stairs, if its band offers two (`Snapshot.fork`):
    /// the other stair beside the real one (the first open neighbour, top row first).
    pub fn fork_snap(&self) -> Option<crate::wire::SnapFork> {
        let next = self.depth + 1;
        if !crate::descent::fork_open_for(self.route2, next) || !self.route.fork_open(next) {
            return None;
        }
        let i = crate::descent::FORKS.iter().position(|f| *f == next)?;
        let taken = self.route.biome(next);
        let (near, far) = (crate::descent::BASE_ORDER[i], crate::descent::BASE_ORDER[i + 1]);
        let other = if taken == near { far } else { near };
        let s = self.floor.stairs_down;
        let mut cands: Vec<Pos> = s.neighbours8().into_iter().filter(|q| self.floor.map.passable(*q) && *q != self.floor.stairs_up).collect();
        cands.sort_by_key(|q| (q.y, q.x));
        let at = cands.first().copied().unwrap_or(s);
        Some(crate::wire::SnapFork { depth: next, taken: taken.name().into(), other: other.name().into(), x: at.x, y: at.y })
    }
    /// Cut 5 §4: a tile the chores keep out of — within two of a sleeping den — while the
    /// hero is not tempted.
    pub fn in_den_zone(&self, p: Pos) -> bool {
        !self.tempted && self.dens.iter().any(|d| d.cheb(p) <= 2)
    }
    /// Cut 7 §4: the room the hero stands in (1-based index into `floor.rooms`; 0 for a
    /// corridor, a door or a cave) and the awake hostiles standing in it.
    pub fn room_ref(&self) -> crate::wire::RoomRef {
        let hp = self.hero.pos;
        let Some(ri) = self.floor.rooms.iter().position(|r| r.contains(hp)) else { return crate::wire::RoomRef { id: 0, hostiles: 0 } };
        let r = self.floor.rooms[ri];
        let hostiles = self.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && !m.dormant && m.awake && r.contains(m.pos)).count() as u32;
        crate::wire::RoomRef { id: ri as u32 + 1, hostiles }
    }
    /// Cut 5 §4: the first tile of a kind on this floor.
    pub fn tile_pos(&self, t: crate::tiles::Tile) -> Option<Pos> {
        let map = &self.floor.map;
        map.tiles.iter().position(|x| *x == t).map(|i| map.pos(i))
    }
    /// Cut 5 §4: a situation tile of that kind is in view (`on_see: nest | shrine | vault`):
    /// a den still asleep, an altar not yet prayed at, a cage not yet opened.
    pub fn sees_situation(&self, what: &str) -> bool {
        use crate::tiles::Tile;
        let map = &self.floor.map;
        let tile = match what {
            "nest" => Tile::Nest,
            "shrine" => Tile::Shrine,
            "vault" => Tile::Vault,
            // Cut 7 §3: the band situations.
            "den" | "lock" | "captive" | "hunger" => return crate::situations::sees(self, what),
            // Cut 8B §3: a stray in view (`on_see: stray → tame`).
            "stray" => return self.monsters.iter().any(|m| m.stray && m.hp > 0 && map.is_visible(m.pos)),
            _ => return false,
        };
        // (no tile outside the vision's last square is visible: `Map::visible_rows`)
        let seen = match map.visible_rows() {
            Some(mut rows) => rows.any(|r| r.into_iter().any(|i| map.tiles[i] == tile && map.visible[i])),
            None => map.tiles.iter().enumerate().any(|(i, x)| *x == tile && map.visible[i]),
        };
        seen && match what {
            "nest" => self.monsters.iter().any(|m| m.nest && m.dormant && m.hp > 0),
            // Cut 7 §3: the hunger's shrine can be lit once, whatever was prayed above.
            "shrine" => !self.prayed || (crate::situations::hunger_floor(self) && !self.lit),
            _ => true,
        }
    }
    /// Cut 5 §4: note a situation met this run (once each per floor).
    pub fn met_situation(&mut self, what: &str) -> bool {
        if self.situations.iter().any(|(t, s)| s == what && *t >= self.turn.saturating_sub(self.floor_turn)) {
            return false;
        }
        self.situations.push((self.turn, what.into()));
        true
    }
    /// Cut 4: loot is counted in gold at pickup. `raw` is the item value (gold pile amount,
    /// item value); `loot` is `loot_raw / GOLD_DIVISOR`, so the HUD stake, the exit note,
    /// `Ev::Exit.loot_kept` and the lineage's gold all agree.
    /// QA on 3d71c33: the carried gold never goes below 0 (`$-3 · keeps $0` after a monkey
    /// took a brought item on D1): whatever is taken off, the stake floors at nothing.
    pub fn loot_add(&mut self, raw: i32) {
        self.loot_raw = (self.loot_raw + raw).max(0);
        self.loot = self.loot_raw / GOLD_DIVISOR;
    }
    /// What an item in the pack counts for in the loot (raw, before `GOLD_DIVISOR`): its value
    /// when it was found on this run, nothing when it never counted — the starting arms, a
    /// brought vault item, a packed supply. A theft or a swap takes off only what was added.
    pub fn loot_value(&self, it: &Item) -> i32 {
        if crate::kit::is_kit_id(it.id) || self.brought.contains(&it.id) || self.supplies.contains(&it.id) {
            0
        } else {
            it.value()
        }
    }
    /// A gold pile's amount is coins (what the label says, what the stake rises by): it is
    /// added whole. Items add their value before `GOLD_DIVISOR` (`loot_add`).
    pub fn loot_add_gold(&mut self, coins: i32) {
        self.loot_add(coins * GOLD_DIVISOR);
    }
    pub fn monster_at(&self, p: Pos) -> Option<usize> {
        self.monsters.iter().position(|m| m.hp > 0 && m.pos == p)
    }
    pub fn occupied(&self, p: Pos) -> bool {
        self.hero.pos == p || self.monster_at(p).is_some()
    }
    pub fn item_at(&self, p: Pos) -> Option<usize> {
        self.items.iter().position(|i| i.pos == p)
    }
    pub fn new_id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }
    pub fn new_item_id(&mut self) -> u32 {
        self.next_item_id += 1;
        self.next_item_id
    }
    pub fn biome(&self) -> Biome {
        self.route.biome(self.depth)
    }
    pub fn allies(&self) -> impl Iterator<Item = &Monster> {
        self.monsters.iter().filter(|m| m.ally && m.hp > 0)
    }
    pub fn companion(&self, cid: u32) -> Option<&Companion> {
        self.companions.iter().find(|c| c.id == cid)
    }
    /// Living companions on the floor.
    pub fn party_alive(&self) -> impl Iterator<Item = &Monster> {
        self.monsters.iter().filter(|m| m.is_companion() && m.hp > 0)
    }
    /// A foe the hero has given up on for now (unreachable or not closing).
    pub fn is_ignored(&self, id: u32) -> bool {
        self.ignored.get(&id).is_some_and(|until| *until > self.actions)
    }
    /// The oscillation guard gave up on this foe for the floor (`ignore(id, u32::MAX)`).
    pub fn given_up(&self, id: u32) -> bool {
        self.ignored.get(&id).is_some_and(|until| *until == u32::MAX)
    }
    /// QA on 778fa1b (qaV): `units` of `kind` found into the pack as item `id` — unless the
    /// item is coming back (a thief's take got back, a find put down and picked up again): then
    /// the way it left is undone instead.
    pub fn note_found(&mut self, id: u32, kind: &str, units: i32) {
        let back = if self.stolen_ids.contains(&id) { "stolen" } else { "left" };
        if self.found_gone.iter().any(|(g, k, f)| *g == id && k == kind && f == back) {
            // (every unit that went that way is back: a stack comes back whole)
            for _ in 0..units.max(1) {
                match self.found_gone.iter().position(|(g, k, f)| *g == id && k == kind && f == back) {
                    Some(i) => {
                        self.found_gone.remove(i);
                    }
                    None => break,
                }
            }
            return;
        }
        // (id 1 is the class's own arms, picked up again)
        if crate::kit::is_kit_id(id) || self.supplies.contains(&id) || self.brought.contains(&id) {
            return;
        }
        if self.found_units.iter().any(|(g, k)| *g == id && k == kind) && kind != "leash" {
            return;
        }
        for _ in 0..units.max(1) {
            self.found_units.push((id, kind.to_string()));
        }
    }
    /// QA on 778fa1b (qaV): item `id` (of `kind`: a bones pile's finds keep their old run's
    /// ids, so an id alone may name two items) left the pack (`used` / `left` / `stolen`) —
    /// its found units still in it (up to `units`; a stack's) are gone there.
    pub fn note_gone(&mut self, id: u32, kind: &str, fate: &str, units: i32) {
        let left = self.found_left(id, kind);
        for _ in 0..left.min(units.max(1) as usize) {
            self.found_gone.push((id, kind.to_string(), fate.to_string()));
        }
    }
    /// The found units of item `id` of `kind` still in the pack.
    pub fn found_left(&self, id: u32, kind: &str) -> usize {
        let found = self.found_units.iter().filter(|(g, k)| *g == id && k == kind).count();
        let gone = self.found_gone.iter().filter(|(g, k, _)| *g == id && k == kind).count();
        found.saturating_sub(gone)
    }
    /// The share of the carry (and of the salvage, XP and renown) an exit at `tier` keeps: a
    /// timed-out run and a drive-off with no way home written keep nothing (Cut 2 §2, Cut 24 §1).
    /// What the exit brings home: the secured gold whole, and the carry since the last checkpoint at the
    /// tier's share (bank 100 % · return 60 % · death 30 %; a stall or a drive-off with no way home nothing).
    pub fn kept(&self, tier: ExitTier) -> i32 {
        self.secured + self.loot.max(0) * self.yield_pct(tier) / 100
    }
    /// All the run carried: the secured gold and the carry since.
    pub fn carried(&self) -> i32 {
        self.secured + self.loot.max(0)
    }
    pub fn yield_pct(&self, tier: ExitTier) -> i32 {
        if self.timed_out || self.driven_lost {
            0
        } else {
            tier.pct()
        }
    }
    /// Cut 24 §1: HP moved on either side of the fight (`turn::nohp_guard`).
    pub fn fight_moved(&mut self) {
        self.nohp.0 = 0;
    }
    pub fn ignore(&mut self, id: u32, actions: u32) {
        let until = self.actions.saturating_add(actions);
        self.ignored.insert(id, until);
    }
    /// Cut 4: blood drawn on the hero ends the stalemate guards and every ignore — except the
    /// oscillation guard's (`u32::MAX`: a foe the hero paced in front of and could not reach
    /// stays ignored until the floor changes; an arrow from it lifting the ignore was the
    /// loop that ended runs as stalls — cohort 9: "the most expensive outcome in the game").
    pub fn unstick(&mut self) {
        self.stuck_until = 0;
        self.row_suppressed = (-9, 0);
        self.rows_rested.1 = 0;
        self.ignored.retain(|_, until| *until == u32::MAX);
    }
    /// Cut 4: hostiles the hero remembers but cannot see — alive, out of view, seen within
    /// `REMEMBER_ACTIONS` — with the tile they were last seen on.
    pub fn remembered_foes(&self) -> Vec<(usize, Pos)> {
        let map = &self.floor.map;
        self.monsters
            .iter()
            .enumerate()
            .filter(|(_, m)| m.hp > 0 && m.hostile() && !map.is_visible(m.pos))
            .filter_map(|(i, m)| {
                let (p, at) = self.known_foes.get(&m.id)?;
                (self.actions.saturating_sub(*at) <= REMEMBER_ACTIONS).then_some((i, *p))
            })
            .collect()
    }
    /// Items are ignored by chores until this action (after fruitless pick_up loops).
    pub fn items_ignored(&self) -> bool {
        self.items_until > self.actions
    }
    /// A tile with a visible hostile, or where one was seen recently (diagnostics; pathing
    /// goes through monsters).
    pub fn foe_blocks(&self, p: Pos) -> bool {
        if let Some(mi) = self.monster_at(p) {
            let m = &self.monsters[mi];
            if m.hostile() && self.floor.map.is_visible(m.pos) {
                return true;
            }
        }
        self.known_foes.values().any(|(q, at)| *q == p && self.actions.saturating_sub(*at) < 30)
    }
    /// Cut 3: the hero's sight radius here — the floor's, +2 with a lantern in the pack or the
    /// `lantern_rig` automation.
    pub fn vision(&self, unlocks: &BTreeSet<String>) -> i32 {
        let lantern = self.hero.inv.iter().any(|i| i.kind == "lantern") || unlocks.contains("lantern_rig");
        self.floor.vision + if lantern { 2 } else { 0 } - crate::traits::dim(self)
    }
    /// A blind foe seen on this floor is still alive (`noise_discipline` holds the rest).
    pub fn blind_foe_known(&self) -> bool {
        self.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.is_blind() && self.blind_seen.contains(&m.id))
    }
    /// Remember where the visible hostiles are (called at each hero action).
    pub fn note_foes(&mut self) {
        let actions = self.actions;
        let map = &self.floor.map;
        let seen: Vec<(u32, Pos)> = self.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && map.is_visible(m.pos)).map(|m| (m.id, m.pos)).collect();
        for (id, p) in seen {
            self.known_foes.insert(id, (p, actions));
        }
        self.known_foes.retain(|_, (_, at)| actions.saturating_sub(*at) < 30);
    }
}

/// Per-turn context borrowed from the game.
/// The own rows a set may play (`LineageState::max_rows`): 4, one per `row5`…`row10` unlock, at most 6
/// under the `short_list` variant.
pub fn max_rows_of(unlocks: &BTreeSet<String>, variant: &str) -> usize {
    // One descent to the `row…` ids, not six lookups.
    use std::ops::Bound::{Excluded, Included};
    let n = 4 + unlocks.range::<str, _>((Included("row"), Excluded("rox"))).filter(|u| ["row5", "row6", "row7", "row8", "row9", "row10"].contains(&u.as_str())).count();
    if variant == "short_list" {
        n.min(6)
    } else {
        n
    }
}

pub struct Ctx<'a> {
    pub facts: &'a mut crate::shared::Shared<BTreeSet<String>>,
    /// The lineage's trophies: a trophy's note is said the first time only (QA on 56f2a1d:
    /// `Trophy: no heal to D5.` in the reel of every D5 run).
    pub trophies: &'a [String],
    /// Cut 2 §5: lineage kills per kind (the `studied` tier).
    pub kill_counts: &'a mut BTreeMap<String, u32>,
    pub flavours: &'a Flavours,
    pub rules: &'a RuleSet,
    pub unlocks: &'a BTreeSet<String>,
    pub grudges: &'a [Grudge],
    pub forge: &'a BTreeMap<String, ForgeRow>,
    /// The own rows in play (`max_rows_of` the unlocks and the variant, which the context holds
    /// unchanged): read when a row is chosen, not on every tick — `Ctx::max_rows`. `Some` fixes it.
    pub max_rows: std::cell::Cell<Option<usize>>,
    pub events: &'a mut Vec<Ev>,
    pub sim: bool,
    /// Cut 11 §1: the live run's provenance log (`Game.prov`; sims never write it).
    pub prov: &'a mut Vec<crate::provenance::Prov>,
    /// Cut 3: the ascension variant ("" at level 0) and the `hunted` stalker.
    pub variant: &'a str,
    pub hunter: Option<&'a Grudge>,
    /// Cut 5 §4: the vault preference; the lineage's lost companions (strays); the saved
    /// sets (a shrine lends a row).
    pub vault_pref: &'a str,
    pub lost: &'a [Lost],
    pub sets: &'a [RuleSet],
    pub active_set: usize,
    /// Cut 23 §3: the live run's why-not tally per row of the set (`Game.row_tally`; sims
    /// never write it).
    pub tally: &'a mut Vec<RowTally>,
}

impl Ctx<'_> {
    /// The own rows in play (`max_rows_of`), worked out once per context.
    pub fn max_rows(&self) -> usize {
        match self.max_rows.get() {
            Some(n) => n,
            None => {
                let n = max_rows_of(self.unlocks, self.variant);
                self.max_rows.set(Some(n));
                n
            }
        }
    }
}

/// Cut 5 §4: a companion that died on an expedition (its kennel entry is gone); a later run
/// may meet it gone wild on D1–5, tameable at `STRAY_TAME`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lost {
    pub kind: String,
    pub name: String,
    pub gen: u32,
    pub heir: u32,
    /// Cut 27 §5 (AT: `Krak the jackal, gone wild` — "is that my hatched jackal?"): how it was
    /// lost (`fell D7 to ogre`), for the stray's line when a later heir meets it.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageState {
    #[serde(default="default_bloodline_id")]
    pub bloodline_id:u32,
    #[serde(default)]
    pub bloodline: Option<crate::wire::BloodlineLegacy>,
    #[serde(default)]
    pub hero_legacy: crate::shared::Shared<Vec<crate::wire::HeroLegacy>>,
    pub seed: u64,
    pub heir: u32,
    pub trait_: Trait,
    pub class: Class,
    pub best_depth: u32,
    pub marks: u32,
    pub facts: crate::shared::Shared<BTreeSet<String>>,
    pub unlocks: BTreeSet<String>,
    pub vault: Vec<Item>,
    pub graveyard: Vec<Grave>,
    pub trophies: Vec<String>,
    pub sets: Vec<RuleSet>,
    pub active_set: usize,
    pub ended: bool,
    pub kills: BTreeSet<String>,
    pub flavours: Flavours,
    pub grudges: Vec<Grudge>,
    pub total_turns: u64,
    pub next_run_id: u32,
    pub next_vault_id: u32,
    pub rng: Rng,
    // Addendum A
    pub party: Vec<Companion>,
    pub kennel: Vec<Companion>,
    pub eggs: Vec<Egg>,
    pub bred: BTreeSet<String>,
    pub next_comp_id: u32,
    // Addendum B
    pub gold: i32,
    /// Sub-gold remainder from salvage, in hundredths (cheap items salvage fractions of a coin).
    #[serde(default)]
    pub gold_carry: i32,
    pub supplies: Vec<Item>,
    // Addendum C
    pub classes: BTreeMap<String, ClassProg>,
    #[serde(default,skip_serializing_if="BTreeMap::is_empty")]
    pub specializations:BTreeMap<String,crate::specialization::Style>,
    // Addendum D
    pub forge: BTreeMap<String, ForgeRow>,
    pub renown: u32,
    pub rank: u32,
    pub keep_pref: String,
    /// Vault item ids insured against loss on death.
    #[serde(default)]
    pub insured: Vec<u32>,
    // Cut 2
    /// Camp rest (or wake) left before the next expedition, in ticks. `send` skips it.
    #[serde(default)]
    pub rest_left: u32,
    /// Cut 26 (seam, control rater AR: a 20-minute absence read `0 RUNS`): the rest left is a
    /// watched run's — the player saw that exit, and the camp time since was its rest; the next
    /// absence does not wait it out again (`offline::run_offline_with`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rest_watched: bool,
    /// Bones piles (§2), oldest first.
    #[serde(default)]
    pub bones: Vec<Bones>,
    /// Kills per kind (§5 studied).
    #[serde(default)]
    pub kill_counts: BTreeMap<String, u32>,
    /// Supplies of the last expedition, which the next send re-packs (Cut 19 §3: the loadout
    /// repeats by default; was the `auto_supply` automation).
    #[serde(default)]
    pub last_supplies: Vec<String>,
    /// Origins parallel to `last_supplies`; absent old saves keep every repeat protected.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub last_supply_origins: Vec<(String, bool)>,
    /// Eggs ever laid (the incubator gate).
    #[serde(default)]
    pub eggs_laid: u32,
    /// Runs banked / returned / died, lifetime.
    #[serde(default)]
    pub runs_banked: u32,
    #[serde(default)]
    pub runs_returned: u32,
    #[serde(default)]
    pub runs_died: u32,
    // Cut 3: ascension.
    /// Times the lineage has ascended, and the variant it plays under ("" at level 0).
    #[serde(default)]
    pub ascension: u32,
    /// Separate from historical challenge restart count; absent saves play tier zero.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endgame: Option<crate::endgame::Progress>,
    #[serde(default)]
    pub variant: String,
    /// Variants the lineage has finished the dungeon with.
    #[serde(default)]
    pub ascended: Vec<String>,
    /// `hunted`: the grudge that stalks every floor from D3.
    #[serde(default)]
    pub hunter: Option<Grudge>,
    /// Cut 4: depths banked from at least once (the first bank at a depth is a mark).
    #[serde(default)]
    pub banked_depths: BTreeSet<u32>,
    // Cut 5
    /// §2: one line per ended heir (cap `CHRONICLE_CAP`); the heir last written; the live
    /// heir's deeds and best depth.
    #[serde(default)]
    pub chronicle: Vec<String>,
    #[serde(default)]
    pub chronicled: u32,
    #[serde(default)]
    pub heir_deeds: Vec<String>,
    #[serde(default)]
    pub heir_best: u32,
    /// §4: companions lost on expeditions (strays), newest last; the vault preference.
    #[serde(default)]
    pub lost: Vec<Lost>,
    #[serde(default = "default_vault_pref")]
    pub vault_pref: String,
    // Cut 6
    /// §1: the last `GOLD_LEDGER_CAP` gold movements, oldest first.
    #[serde(default)]
    pub gold_ledger: Vec<GoldLine>,
    // Cut 9
    /// §6: the (threat, resolution) pairs of the last `REEL_ABSENCES` reels, oldest first —
    /// the next reel skips them.
    #[serde(default)]
    pub reel_pairs: Vec<Vec<(String, String)>>,
    // Cut 13
    /// §2: the two traits the new heir may wake with (drawn from the seed and the heir
    /// number, never the last heir's); `trait_` is the first until `set_trait` picks; a send
    /// empties it.
    #[serde(default)]
    pub trait_offer: Vec<Trait>,
    /// Cut 30: the heir's traits (blood and born), the wake's cards, the family's lean
    /// (`traits::HeirTraits`); the temperament above is kept only to map an older save.
    #[serde(default)]
    pub heirs: crate::traits::HeirTraits,
    /// QA on 56f2a1d: the kennel's free leash, dropped from the shelf, came back after every
    /// run. A drop declines the kennel until a leash is bought or a kind is tamed.
    #[serde(default)]
    pub kennel_declined: bool,
    /// §3: kinds the last run used to no effect — `restock` skips them once.
    #[serde(default)]
    pub last_wasted: Vec<String>,
    /// QA on 1a2a4a9 (qaP: `repeat · $100`, $26 in the purse, the heal not re-packed and no
    /// word): the kinds the last re-pack could not pay for (it buys what it can, in order).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub repeat_short: Vec<String>,
    /// QA on 778fa1b (qaU: `repeat on · $20` at the camp, then `repeat heal ×1 · −$26` at the
    /// return — the run set a new best, and the shelf's price rose with it before the re-pack):
    /// the price of each kind the last send's repeat re-buys, quoted at the send (the camp's
    /// badge then). The re-pack at that send's exit pays these (`Game::restock_at`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub repeat_quote: BTreeMap<String, i32>,
    /// Cut 15 §2: unlocks bought with gold so far; each raises the next gold price by a quarter
    /// of the first (`meta::gold_price`).
    #[serde(default)]
    pub gold_buys: u32,
    // Cut 16
    /// §1: banks and returns from each depth (capped at `PICKED_CAP`): the floor's gold and
    /// items pay `0.8^picked` (never under a quarter; `freshness`). One step recovers per
    /// night the depth is not visited.
    #[serde(default)]
    pub picked: BTreeMap<u32, u32>,
    /// §1: a night is `NIGHT_RUNS` finished runs (offline or live, so it does not hang on how
    /// the client slices an absence); the runs so far and the depths they visited.
    #[serde(default)]
    pub night_runs: u32,
    #[serde(default)]
    pub night_seen: BTreeSet<u32>,
    // Cut 19
    /// §5: den snatches the lineage has suffered — once it has lost to a den (and holds the
    /// `den` fact), the den's thieves pounce on ≤ 1 in 3 of the runs that meet them
    /// (`Run.den_thin`, `situations::den_pounces`).
    #[serde(default)]
    pub den_thefts: u32,
    // Cut 20
    /// §1: dens that have woken on the lineage's heirs — after the first (the lesson), every
    /// den pounces on ≤ 1 floor in 3 (`Run.den_thin`).
    #[serde(default)]
    pub den_wakes: u32,
    /// §5: the bounty floor — at each night's end the lineage's best depth + 2 (`night`): its
    /// gold pays double and it holds one item of the lineage's next tier (`bounty_item`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<u32>,
    /// §3: the loadout repeats by default — a send re-packs the last send's supplies at the
    /// shelf's price (`Game::restock`); `set_restock(false)` (the camp's tap) stops it.
    #[serde(default)]
    pub restock_off: bool,
    /// Hero looks: the heirs' cosmetic look (`male | female | cat`), inherited by every heir;
    /// `None` = the class's own (`Class::default_look`). No run, sim or key reads it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub look: Option<String>,
    // Cut 21
    /// §1: the lit waystones (`WAYSTONES`, ascending) — a biome's first floor, lit once the
    /// lineage has banked from a floor at or past it (`finish_run`; a save from before is
    /// lit from `banked_depths` at the load).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub waystones: Vec<u32>,
    /// Cut 26 §2: waystones lit on a lane off the base order — (depth, the route's prefix to it:
    /// `Route::prefix`), ascending. A waystone is a place: D9 in the Burrows (a Fens-first
    /// route's) is not D9 in the Fens. The base order's stay in `waystones`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lane_stones: Vec<(u32, u8)>,
    /// §1: the floor the next send starts on — 1, or a lit waystone (`Game::set_start`). A
    /// start below D1 pays `WAYSTONE_TOLL × depth` at the send (`start_run`).
    #[serde(default = "default_start")]
    pub start: u32,
    /// QA on a946e04 (qaT: `waystone D5 ×16 · −$800` over one absence, $121 → $40, the repeat
    /// starved): the waystone passes paid this night — a start's toll is paid once per night
    /// (`NIGHT_RUNS` runs, offline or live) and the night's later sends from it go free; the
    /// night's end clears them (`night`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub night_passes: BTreeSet<u32>,
    /// Cut 27 §5: the last `SENT_SETS` distinct rule sets sent (live or offline), oldest first —
    /// a death's verdict offers back a row the player took out since (`DeathRec.removed`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sent_sets: Vec<RuleSet>,
    /// QA on a946e04: the waystone whose pass an absence's purse could not pay this night —
    /// the night's later offline sends start on D1 without asking again (the report's
    /// `start_short`); cleared at the night's end. A watched send asks each time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub night_short: Option<u32>,
    /// Cut 22 §1 (AH: "the monkey stole the heal potion … 8 seconds after I paid $40"): the
    /// repeat re-buys a supply thieves kept once a night — this night's one is spent — and
    /// the kinds a later theft took wait for the night's end (`theft_skip`; `night` clears both).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub night_theft_rebought: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub theft_skip: Vec<String>,
    // Cut 23
    /// §1: the forge — steps owned per ladder (`kit::KIT_SLOTS`), permanent for the lineage.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kit: BTreeMap<String, u32>,
    /// §1: this night's net gold (every movement but the player's purchases: unlocks, the
    /// forge, insurance, hatching) and the last full night's (`kit::nights`).
    #[serde(default)]
    pub night_net: i32,
    #[serde(default)]
    pub last_night_net: i32,
    /// §3: per row the recent sends sat under, its why-not tally (`RowTally`), keyed by the
    /// row's conds and verb; rows no set holds any more are dropped.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_stats: Vec<(crate::rules::Row, RowTally)>,
    // Cut 24 §2
    /// A named foe (not a grudge) → the run that last met it (`Run.named_rest`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub named_met: BTreeMap<String, u32>,
    /// A floor event's kind → the (run, line) it showed in the last two runs (`chronicle::variant`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub event_recent: BTreeMap<String, Vec<(u32, u8)>>,
    /// The item kinds any run has found (`ExitLine.news`: `new find`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub found_kinds: BTreeSet<String>,
    /// The last run in brief — for the news of a run with nothing new (`ExitLine.news`: the one
    /// thing that differed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<RunBrief>,
    /// Cut 24 §3: the forge's unit, fixed the first time the forge was shown (`kit::unit_of`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kit_unit: Option<u32>,
    /// Cut 113 §2: the forge tiers that took the other branch (bit i: tier i; `kit::branch_at`),
    /// per ladder — weapon aim · edge, armour plate · pace. Empty: the ladder's defaults.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kit_alt: BTreeMap<String, u32>,
    /// Cut 113 §2: the branch the player last chose off a ladder's default (the apprentice
    /// follows it; a default chosen by hand clears it).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub kit_lean: BTreeMap<String, String>,
    /// Cut 113 §3: the return's pick, waiting at camp until taken (`returns`); never lost —
    /// a later return adds its minutes to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_pick: Option<crate::returns::Pending>,
    /// QA on 0c6e126 (qaY: `+1 row ◆2 or $450` became `$600` overnight with no gold buy): the row slots' unit, fixed at the first
    /// exit (the unlock shelf shows the row's price from the first camp) — `kit::row_gold`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub row_unit: Option<u32>,
    // Cut 28
    /// §1: the oath board (`oath::refresh`), the sworn one's id, the draws so far (the board's
    /// seeded order), the oaths kept, the chronicle titles they earned.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oaths: Vec<crate::oath::OathState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath_sworn: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub oath_drawn: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub oaths_kept: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub titles: Vec<String>,
    /// Cut 28b: a band boss seen or a plateau met (`oath::open`) — the board has a use now.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub oath_open: bool,
    // Cut 29
    /// §1: the lineage's clock — the seconds of every absence so far (`offline::run_offline`); a
    /// day is `DAY_S` of it. `mark_day` is the first day whose night's mark is unpaid (◆1 per day
    /// whose absences brought a send home); `day_net` this day's net gold, `last_day_net` the
    /// last whole day's (an oath's price, `oath::price`), `day` the day `day_net` is for.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub clock_s: u64,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub mark_day: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub day: u32,
    #[serde(default)]
    pub day_net: i32,
    #[serde(default)]
    pub last_day_net: i32,
    /// §1: the oaths sworn in the extra slots (`oath_slot_2`, `oath_slot_3`), and the day each
    /// sworn oath was sworn on (an oath lapses unkept at its day's end: `oath::lapse`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oath_extra: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oath_days: BTreeMap<String, u32>,
    /// §1: the lineage's works commissioned with gold (`meta::commission`), oldest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub works: Vec<String>,
    /// §2: the systems open to this lineage (`systems::update`), and the ones opened since the
    /// camp last looked (the reveal's glint; `Game::seen_systems` clears them).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub systems: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems_new: Vec<String>,
    /// §2: the times each fork has been seen (the D5 route opens at the second).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub forks_seen: BTreeMap<u32, u32>,
    /// §1 (E1): the day of the clock the best depth last rose on, the day the wall's edit was last
    /// searched, and the edit on offer (`wall::search`) — cleared by a new best.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub best_day: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_day: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall_offer: Option<crate::wire::WallEdit>,
    /// §3: the meters of this night's runs, the last full night's, and the last two runs'.
    #[serde(default, skip_serializing_if = "crate::meters::Meter::is_empty")]
    pub night_meter: crate::meters::Meter,
    #[serde(default, skip_serializing_if = "crate::meters::Meter::is_empty")]
    pub last_night_meter: crate::meters::Meter,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub meters_recent: Vec<crate::meters::Meter>,
    /// §4: the standing orders' switches that have no field of their own (`insure`).
    #[serde(default)]
    pub orders: crate::wire::StandingSwitches,
    /// Cut 30 §2: the packages (`packages::PkgState`: slots, levels, drills, meetings, the pen);
    /// `pkg_v` 0 is a save from before them (`packages::migrate` at the load).
    #[serde(default)]
    pub pkg: crate::packages::PkgState,
    #[serde(default)]
    pub pkg_v: u32,
    /// Cut 30 §3–5: the town (buildings, the bank, the quest board).
    #[serde(default)]
    pub town: crate::shared::Shared<crate::town::Town>,
    /// Cut 30 (PROGRESSION_V2 §4): the systems that may still open this report (one a report; set at
    /// an absence's start and a send), and the ready ones waiting their turn, in curriculum order.
    #[serde(default)]
    pub reveal_left: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reveal_queue: Vec<String>,
    /// PROGRESSION_V2 §2 (reserved for Cut 31: the expedition and the era; saved, unread): glory, the
    /// expeditions taken, the era's gate.
    #[serde(default)]
    pub glory: u32,
    #[serde(default)]
    pub expeditions: u32,
    #[serde(default)]
    pub era_gate: u32,
    /// Cut 30.5: the works tree (workers, chores by hand, the haul chest, the gold ledger); `v` 0 a
    /// save from before it (`tree::upgrade` at the load).
    #[serde(default)]
    pub tree: crate::tree::Tree,
    /// RUNS_UI: the runs log (oldest first, cap `RUN_LOG_CAP`), the absences counted and whether an
    /// absence's batch is the latest thing that ran (a send, a step or `advance` ends it).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub run_log: Vec<crate::wire::RunRec>,
    #[serde(default)]
    pub absences: u32,
    #[serde(default)]
    pub in_absence: bool,
}

/// Cut 29 §1: a day of the lineage's clock.
pub const DAY_S: u64 = 24 * 3600;

/// Cut 24 §2: a finished run in brief (`LineageState::last_run`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RunBrief {
    pub depth: u32,
    pub tier: String,
    #[serde(default)]
    pub cause: Option<String>,
    #[serde(default)]
    pub twists: Vec<String>,
    #[serde(default)]
    pub kept: i32,
    #[serde(default)]
    pub turns: u32,
    #[serde(default)]
    pub kills: u32,
}

/// Cut 23 §3: one row's why-not over the recent sends (`LineageState::row_stats`). Halved
/// once `actions` passes `ROW_TALLY_CAP`, so it reads the recent nights.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RowTally {
    pub sends: u32,
    pub actions: u32,
    pub fired: u32,
    pub matched: u32,
    /// Reached with its conds holding and did not act: reason → count.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub blocked: BTreeMap<String, u32>,
    /// Its first failing cond (the cond's own word: `gas`, `hp<`) → count.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub unmet: BTreeMap<String, u32>,
}

/// Cut 23 §3: a row's tally is halved past this many actions (≈ two nights of sends).
pub const ROW_TALLY_CAP: u32 = 20_000;

fn default_start() -> u32 {
    1
}

/// Cut 21 §1: the waystones — each biome's first floor below the Warrens (Burrows D5, Fens D9,
/// Crypt D14, Foundry D19, Deep D24, Sanctum D29).
pub const WAYSTONES: [u32; 6] = [5, 9, 14, 19, 24, 29];
/// Cut 21 §1: a start below D1 costs this many coins per floor (Cut 22 §1: $10 → $5).
/// QA on 778fa1b (qaV: a D5 start expected ~$25 against a $25 toll and reached D6 less often
/// than D1): a waystone start is free — the skipped shallow loot and the added death risk are
/// its cost. The pass / night bookkeeping stays and charges nothing.
pub const WAYSTONE_TOLL: i32 = 0;
/// Cut 27 §5: how many distinct sent sets the lineage remembers (`LineageState::sent_sets`).
pub const SENT_SETS: usize = 3;

/// Cut 16 §1: a night of runs (the ledger's "a night of 16 runs").
pub const NIGHT_RUNS: u32 = 16;
/// Cut 20 §5: the night's bounty floor — the best depth + `BOUNTY_BELOW`, and (QA on e75ec29,
/// qaR: `D15 ×2 · 0%` behind the unbeaten D13 mother) never past the next boss floor the
/// lineage has not passed (its boss unslain and the best depth not below it): a bounty is a
/// floor a run can reach. Set once at the night's end; fixed for the night.
pub fn bounty_floor(best_depth: u32, kills: &BTreeSet<String>) -> u32 {
    bounty_floor_on(best_depth, kills, Route::BASE)
}

/// `bounty_floor` on a route (Cut 26 §1: the bosses where the route puts them).
pub fn bounty_floor_on(best_depth: u32, kills: &BTreeSet<String>, route: Route) -> u32 {
    let want = best_depth + BOUNTY_BELOW;
    let wall = route.bosses().into_iter().filter(|(k, d)| *d >= best_depth && !kills.contains(*k)).map(|(_, d)| d).min();
    want.min(wall.unwrap_or(u32::MAX)).clamp(1, crate::descent::ENDING_DEPTH - 1)
}

/// Cut 20 §5: the bounty floor sits this far below the lineage's best depth.
pub const BOUNTY_BELOW: u32 = 2;
/// Cut 20 §5: the bounty floor's gold piles pay this many times their coins.
pub const BOUNTY_GOLD_MULT: i32 = 2;
/// Cut 16 §1: picks past this pay the floor (`0.8^7` < a quarter).
pub const PICKED_CAP: u32 = 7;
/// Cut 16 §1: freshness by picks, in permille (`0.8^n`, never under 250).
pub const FRESHNESS: [u32; 8] = [1000, 800, 640, 512, 410, 328, 262, 250];
/// Cut 16 §1: a depth this picked reads `D3 · picked clean` in the report.
pub const PICKED_SHOWN: u32 = 3;

/// Cut 6 §1: gold movements kept on the lineage.
pub const GOLD_LEDGER_CAP: usize = 80;   // a night of 16 runs writes ~50 lines; 20 could not be reconciled (QA on 952e306)
/// Cut 6 §1: exit lines kept per absence (`ReturnReport.exits`).
pub const EXITS_CAP: usize = 5;

fn default_vault_pref() -> String {
    "weapon".into()
}

impl LineageState {
    pub fn new(seed: u64) -> LineageState {
        let mut rng = Rng::derive(seed, hash_str("lineage"));
        let flavours = Flavours::roll(&mut rng);
        let offer = trait_offer(seed, 1, None, Trait::ALL[rng.below(4) as usize]);
        let trait_ = offer[0];
        let mut classes = BTreeMap::new();
        for c in Class::ALL {
            // Keep old zero-XP class maps exact until Gunner is chosen/unlocked.
            if c == Class::Gunner { continue; }
            classes.insert(c.name().to_string(), ClassProg { level: 1, xp: 0, next: 0 });
        }
        let mut l = LineageState {
            bloodline_id:1, bloodline: Some(Default::default()),
            hero_legacy: vec![crate::wire::HeroLegacy { heir: 1, class: "fighter".into(), ..Default::default() }].into(),
            seed,
            heir: 1,
            trait_,
            class: Class::Fighter,
            best_depth: 0,
            marks: 0,
            facts: BTreeSet::new().into(),
            unlocks: BTreeSet::new(),
            vault: Vec::new(),
            graveyard: Vec::new(),
            trophies: Vec::new(),
            sets: vec![crate::probes::preset(Class::Fighter), RuleSet::default(), RuleSet::default()],
            active_set: 0,
            ended: false,
            kills: BTreeSet::new(),
            flavours,
            grudges: Vec::new(),
            total_turns: 0,
            next_run_id: 1,
            next_vault_id: 100_000,
            rng,
            party: Vec::new(),
            kennel: Vec::new(),
            eggs: Vec::new(),
            bred: BTreeSet::new(),
            next_comp_id: 1,
            gold: 0,
            gold_carry: 0,
            supplies: Vec::new(),
            classes,
            specializations:BTreeMap::new(),
            forge: BTreeMap::new(),
            renown: 0,
            rank: 0,
            keep_pref: "best_weapon".into(),
            insured: Vec::new(),
            rest_left: 0,
            rest_watched: false,
            bones: Vec::new(),
            kill_counts: BTreeMap::new(),
            last_supplies: Vec::new(),
            last_supply_origins: Vec::new(),
            eggs_laid: 0,
            runs_banked: 0,
            runs_returned: 0,
            runs_died: 0,
            ascension: 0,
            endgame: None,
            variant: String::new(),
            ascended: Vec::new(),
            banked_depths: BTreeSet::new(),
            hunter: None,
            chronicle: Vec::new(),
            chronicled: 0,
            heir_deeds: Vec::new(),
            heir_best: 0,
            lost: Vec::new(),
            vault_pref: default_vault_pref(),
            gold_ledger: Vec::new(),
            reel_pairs: Vec::new(),
            trait_offer: offer.to_vec(),
            heirs: crate::traits::fresh(),
            kennel_declined: false,
            last_wasted: Vec::new(),
            repeat_short: Vec::new(),
            repeat_quote: BTreeMap::new(),
            gold_buys: 0,
            picked: BTreeMap::new(),
            night_runs: 0,
            night_seen: BTreeSet::new(),
            night_passes: BTreeSet::new(),
            sent_sets: Vec::new(),
            night_short: None,
            night_theft_rebought: false,
            theft_skip: Vec::new(),
            den_thefts: 0,
            den_wakes: 0,
            bounty: None,
            restock_off: false,
            look: None,
            waystones: Vec::new(),
            lane_stones: Vec::new(),
            start: 1,
            kit: BTreeMap::new(),
            night_net: 0,
            last_night_net: 0,
            row_stats: Vec::new(),
            named_met: BTreeMap::new(),
            event_recent: BTreeMap::new(),
            found_kinds: BTreeSet::new(),
            last_run: None,
            kit_unit: None,
            kit_alt: BTreeMap::new(),
            kit_lean: BTreeMap::new(),
            return_pick: None,
            row_unit: None,
            oaths: Vec::new(),
            oath_sworn: None,
            oath_drawn: 0,
            oaths_kept: 0,
            oath_open: false,
            titles: Vec::new(),
            clock_s: 0,
            mark_day: 0,
            day: 0,
            day_net: 0,
            last_day_net: 0,
            oath_extra: Vec::new(),
            oath_days: BTreeMap::new(),
            works: Vec::new(),
            systems: BTreeSet::new(),
            systems_new: Vec::new(),
            forks_seen: BTreeMap::new(),
            orders: Default::default(),
            pkg: Default::default(),
            pkg_v: 1,
            town: crate::town::Town { manual: true, home: Some(false), auto_collect: true, gold_v: 1, ..Default::default() }.into(),
            reveal_left: 0,
            reveal_queue: Vec::new(),
            glory: 0,
            expeditions: 0,
            era_gate: 0,
            tree: crate::tree::Tree::fresh(),
            best_day: 0,
            wall_day: None,
            wall_offer: None,
            night_meter: Default::default(),
            last_night_meter: Default::default(),
            meters_recent: Vec::new(),
            run_log: Vec::new(),
            absences: 0,
            in_absence: false,
        };
        // Cut 8B §3: `tame` is owned from the start and the kennel's leash is on the shelf (its
        // fact with it), so the first stray is a companion in the first hour.
        l.unlocks.insert("tame".into());
        l.facts.insert("item:leash".into());
        l.kennel_leash();
        l
    }
    /// RUNS_UI: a record onto the runs log (cap `RUN_LOG_CAP`, oldest dropped).
    pub fn push_run(&mut self, rec: crate::wire::RunRec) {
        self.run_log.push(rec);
        if self.run_log.len() > RUN_LOG_CAP {
            let k = self.run_log.len() - RUN_LOG_CAP;
            self.run_log.drain(..k);
        }
    }
    /// Cut 8B §3: the kennel's leash — while the lineage has never tamed, a free, known leash
    /// sits on the shelf (never refunded, never rebought; one at a time).
    pub fn kennel_leash(&mut self) {
        // (a bought leash on the shelf is the player's, not the kennel's: QA on 524827b)
        if self.kennel_declined || self.tamed_kinds() > 0 || self.supplies.iter().any(|s| s.kind == "leash" && s.free) || self.supplies.len() >= self.supply_cap() {
            return;
        }
        let id = self.next_vault_id;
        self.next_vault_id += 1;
        let mut it = Item::new(id, "leash");
        it.amount = 1;
        it.known = true;
        it.free = true;
        self.supplies.push(it);
    }
    /// Cut 24 §2 (AK: `Ashul the jackal` on D3 run after run): a named foe met in one of the two
    /// runs before `run` rests — it appears at most every third run (a grudge excepted).
    pub fn named_resting(&self, name: &str, run: u32) -> bool {
        self.named_met.get(name).is_some_and(|&m| run < m + 3)
    }
    /// Cut 8B §3: the first stray — a lineage that has never tamed (and has lost no companion
    /// to go wild) meets a stray jackal on D2 or D3 on 80% of seeds, the same jackal every run
    /// until it is tamed. Returns (depth, name).
    pub fn first_stray(&self) -> Option<(u32, String)> {
        if self.tamed_kinds() > 0 || !self.lost.is_empty() {
            return None;
        }
        let mut rng = Rng::derive(self.seed, hash_str("first_stray"));
        if !rng.chance(FIRST_STRAY_PCT) {
            return None;
        }
        let depth = 2 + rng.below(2);
        Some((depth, crate::descent::grudge_name(&mut rng)))
    }
    /// Cut 6 §1: every gold movement goes through here — the amount and a ≤ 3-word reason
    /// land in the ledger (a movement with the same tick and reason as the last line merges
    /// into it: an exit's salvage is one line). A zero movement is kept only for an exit
    /// (`why` starting with the tier word), so `+$0 died D5` explains what a death yields.
    pub fn gold_move(&mut self, delta: i32, why: &str) {
        self.gold_move_n(delta, why, 0);
    }
    /// `gold_move` for `n` supplies bought or refunded at once — the line counts them
    /// (`GoldLine.n`; QA on 778fa1b: `repeat heal ×1 · −$104` for four heals).
    pub fn gold_move_n(&mut self, delta: i32, why: &str, n: u32) {
        self.gold += delta;
        // Cut 30.5: the haul chest and the gold ledger
        crate::tree::on_gold(self, delta, why);
        // Cut 23 §1: the night's net is income less upkeep — the player's own purchases are not in it.
        // QA on 912e135 (qaW: `sword +1 · $300 · 7 nights` after a night of deaths — the purse held at $40 by the heir purse): the
        // heir purse's top-up refills to a floor and is no income; a night of deaths nets nothing toward a step.
        // (Cut 28 §1: an oath's price and a forswearing's refund are the player's purchase too)
        if !["unlock ", "forge ", "insure ", "hatch", "egg", "ascended", "wake pay", "oath ", "forswear ", "bank ", "hire "].iter().any(|p| why.starts_with(p)) {
            self.night_net += delta;
            self.day_net += delta;
        }
        let exit = is_exit_why(why);
        // QA on 1a2a4a9: a re-pack the purse could not pay says so (`$0 repeat short`).
        if delta == 0 && !exit && why != REPEAT_SHORT {
            return;
        }
        let t = self.total_turns;
        if let Some(last) = self.gold_ledger.last_mut() {
            if last.t == t && last.why == why && last.bloodline_id==self.bloodline_id && !exit {
                last.delta += delta;
                last.n += n;
                return;
            }
        }
        self.gold_ledger.push(GoldLine { bloodline_id:self.bloodline_id,t, delta, why: why.into(), n, lost: 0 });
        while self.gold_ledger.len() > GOLD_LEDGER_CAP {
            self.gold_ledger.remove(0);
        }
    }
    /// Cut 6 §5: bosses whose counter is known, each with its counter row and text.
    pub fn counters(&self) -> Vec<Counter> {
        crate::descent::BOSS_DEPTHS
            .iter()
            .filter_map(|(kind, _)| crate::facts::boss_counter_row(&self.facts, kind).map(|row| Counter { boss: kind.to_string(), text: crate::facts::counter_text(&row), row }))
            .collect()
    }
    /// Cut 5 §2: the heir's chronicle line, written once when the heir ends (`end` = `fell to
    /// gas` · `retired at rank 3` · `ascended`; `tail` = `left bones on D7`).
    pub fn chronicle_heir(&mut self, end: &str, tail: Option<String>) {
        self.chronicle_heir_at(end, tail, None)
    }
    /// `chronicle_heir` for an heir who ended on `at`: QA on 0c6e126 (qaY: `♟6 · D8 · … left
    /// bones on D6` — "bones two floors above his depth"), an heir whose best is deeper than the
    /// floor he fell on reads `best D8`, so the bones' floor is not read against it.
    pub fn chronicle_heir_at(&mut self, end: &str, tail: Option<String>, at: Option<u32>) {
        if self.chronicled == self.heir {
            return;
        }
        self.chronicled = self.heir;
        // Cut 8B §1: a set with a combo names the heir by it (`the bait fighter`, `the
        // hit-and-fade rogue`; the first combo by row order), else by the trait.
        let epithet = match self.rules().combos().first() {
            Some(c) => crate::rules::combo_slug(&c.name),
            None => self.trait_.name().to_string(),
        };
        let best = if at.is_some_and(|d| d < self.heir_best) { format!("best D{}", self.heir_best) } else { format!("D{}", self.heir_best) };
        let mut parts = vec![format!("♟{} the {} {}", self.heir, epithet, self.class.name()), best];
        // QA on 0c6e126 (qaY: every heir's line quoted `"fighter" set`): the class's own preset name is no name — a set the player
        // named is quoted
        let class_name = self.class.name();
        if let Some(name) = self.rules().name.as_deref().filter(|n| !n.trim().is_empty() && n.trim() != class_name) {
            parts.push(format!("\"{}\" set", name.trim()));
        }
        // Bosses first, then the rest, two at most. QA on e75ec29 (qaQ: the report's BONES
        // listed 12 finds, the chronicle's lines 8 — a third find was cut by the two): the
        // bones found are one deed of their own, every pile named (`found ♟4, ♟9's bones`).
        let bones = |d: &String| d.starts_with("found ♟") && d.ends_with("'s bones");
        // (Cut 28b: the oath kept or broken first — the player's own goal — then the bosses)
        let oath = |d: &String| d.starts_with("kept the oath") || d.starts_with("broke the oath");
        let mut deeds: Vec<String> = self.heir_deeds.iter().filter(|d| oath(d)).cloned().collect();
        deeds.extend(self.heir_deeds.iter().filter(|d| d.starts_with("took the")).cloned());
        deeds.extend(self.heir_deeds.iter().filter(|d| !d.starts_with("took the") && !oath(d) && !bones(d)).cloned());
        parts.extend(deeds.into_iter().take(2));
        let found: Vec<&str> = self.heir_deeds.iter().filter(|d| bones(d)).filter_map(|d| d.strip_prefix("found ").and_then(|d| d.strip_suffix("'s bones"))).collect();
        if !found.is_empty() {
            parts.push(format!("found {}'s bones", found.join(", ")));
        }
        parts.push(end.to_string());
        if let Some(t) = tail {
            parts.push(t);
        }
        self.chronicle.push(format!("{}.", parts.join(" · ")));
        while self.chronicle.len() > CHRONICLE_CAP {
            self.chronicle.remove(0);
        }
    }
    /// A deed of the live heir (first boss kills, captives freed, tames, bones found).
    pub fn heir_deed(&mut self, deed: String) {
        // The bones found are all kept (the chronicle names every pile, the report counts them).
        let bones = deed.starts_with("found ♟") && deed.ends_with("'s bones");
        if !self.heir_deeds.contains(&deed) && (bones || self.heir_deeds.iter().filter(|d| !(d.starts_with("found ♟") && d.ends_with("'s bones"))).count() < 6) {
            self.heir_deeds.push(deed);
        }
    }
    /// QA on 0c6e126 (qaZ: `new find: leash` — the free supply, back from a pile of bones): a kind
    /// the lineage packs or keeps (the shelf, the last send's supplies, the vault) is no find.
    pub fn owned_kind(&self, kind: &str) -> bool {
        self.supplies.iter().any(|s| s.kind == kind) || self.last_supplies.iter().any(|k| k == kind) || self.vault.iter().any(|v| v.kind == kind) || (kind == "leash" && self.kennel_declined)
    }
    pub fn variant_is(&self, v: &str) -> bool {
        self.variant == v
    }
    /// Cut 30 (PROGRESSION_V2 §4): the lineage's age in hours — the clock of its absences, or the ticks
    /// it has lived (watched play included), whichever is more.
    pub fn age_h(&self) -> u32 {
        (self.clock_s.max(self.total_turns / 10) / 3600) as u32
    }
    pub fn rules(&self) -> &RuleSet {
        &self.sets[self.active_set.min(self.sets.len() - 1)]
    }
    pub fn class_level(&self) -> u32 {
        self.classes.get(self.class.name()).map(|c| c.level).unwrap_or(1)
    }
    /// Cut 23 §3: fold a run's why-not tally (indexed by the active set's rows) into
    /// `row_stats`, keyed by each row's conds and verb; rows no set holds are dropped.
    pub fn fold_row_tally(&mut self, tally: &[RowTally]) {
        let set = self.rules().clone();
        for (i, r) in set.rows.iter().enumerate() {
            let Some(t) = tally.get(i).filter(|t| t.actions > 0) else { continue };
            let pos = self.row_stats.iter().position(|(x, _)| x.conds == r.conds && x.verb == r.verb);
            let e = match pos {
                Some(p) => &mut self.row_stats[p].1,
                None => {
                    self.row_stats.push((crate::rules::Row::new(r.conds.clone(), r.verb.clone()), RowTally::default()));
                    &mut self.row_stats.last_mut().expect("pushed").1
                }
            };
            e.sends += 1;
            e.actions += t.actions;
            e.fired += t.fired;
            e.matched += t.matched;
            for (k, n) in &t.blocked {
                *e.blocked.entry(k.clone()).or_insert(0) += n;
            }
            for (k, n) in &t.unmet {
                *e.unmet.entry(k.clone()).or_insert(0) += n;
            }
            if e.actions > ROW_TALLY_CAP {
                // (QA on 524827b, qaAB: `R1 3→11 · R2 11→17` over one 16-run absence — the sends are a count, never
                // halved with the window: every row that sat in a send counts it, so rows that ran together agree)
                e.actions /= 2;
                e.fired /= 2;
                e.matched /= 2;
                for n in e.blocked.values_mut().chain(e.unmet.values_mut()) {
                    *n /= 2;
                }
                e.blocked.retain(|_, n| *n > 0);
                e.unmet.retain(|_, n| *n > 0);
            }
        }
        let sets = self.sets.clone();
        self.row_stats.retain(|(r, _)| sets.iter().any(|s| s.rows.iter().any(|x| x.conds == r.conds && x.verb == r.verb)));
    }
    /// Cut 23 §3: the active set's rows' why-not (`Lineage.row_why`).
    pub fn row_why(&self) -> Vec<Option<crate::wire::RowStat>> {
        if self.row_stats.is_empty() {
            return Vec::new();
        }
        self.rules().rows.iter().map(|r| self.row_stats.iter().find(|(x, _)| x.conds == r.conds && x.verb == r.verb).map(|(_, t)| crate::turn::row_stat(r, t))).collect()
    }
    pub fn to_wire(&self) -> Lineage {
        Lineage { class_styles:Some(crate::specialization::offers(self,false)), legacy_respec:None, return_pick:None, selected_loadout:vec![], hero_slots:vec![], selected_bloodline:1, bloodline_price:crate::bloodlines::SLOT_PRICE, bloodline_cap:crate::bloodlines::SLOT_CAP as u32, bloodline: self.bloodline.clone().unwrap_or_default(), legacy_upgrades: crate::legacy::offers(self, false), hero_legacy: self.hero_legacy.iter().cloned().map(|mut h| { if h.name.is_empty() { h.name = crate::legacy::hero_identity(self.seed, h.heir, self.bloodline_id); } h }).collect(), runs: self.run_log.clone(), live: None, replays: Vec::new(), clock_s: self.clock_s, absences: self.absences, age_h: self.age_h(), reveal_queue: self.reveal_queue.clone(), reveal_next: crate::systems::next(self), glory: self.glory, expeditions: self.expeditions, era_gate: self.era_gate, packages: crate::packages::wire(self), town: crate::town::wire(self), tracks: crate::town::tracks(self), tree: None, repeat_added: Vec::new(), wall: self.wall_offer.clone(), meters: crate::wire::LineageMeters { runs: self.meters_recent.iter().map(crate::meters::wire).collect(), night: (!self.night_meter.is_empty()).then(|| crate::meters::wire(&self.night_meter)), last_night: (!self.last_night_meter.is_empty()).then(|| crate::meters::wire(&self.last_night_meter)) }, systems: crate::systems::wire(self), oath_slots: crate::oath::slots(self) as u32, sworn: crate::oath::sworn_ids(self), tier: crate::meta::tier(self), oath_draw: crate::oath::draw_wire(self), works: self.works.clone(), commission: crate::kit::commission_wire(self), orders: self.standing_orders(), supply_cap: self.supply_cap() as u32, oaths: crate::oath::wire(self), oath: self.oath_sworn.clone(), titles: self.titles.clone(), walls: crate::oath::walls(self), oath_open: crate::oath::open(self),
            seed: self.seed,
            heir: self.heir,
            trait_: self.trait_.name().into(),
            trait_offer: self.trait_offer.iter().map(|t| t.name().to_string()).collect(),
            heir_traits: crate::traits::wire(self),
            class: self.class.name().into(),
            best_depth: self.best_depth,
            marks: self.marks,
            facts: self.facts.iter().cloned().collect(),
            unlocks: self.unlocks.iter().cloned().collect(),
            vault: self.vault.iter().map(|i| to_inv(i, &self.facts, &self.flavours)).collect(),
            graveyard: self.graveyard.clone(),
            trophies: self.trophies.clone(),
            sets: self.sets.clone(),
            active_set: self.active_set,
            ended: self.ended,
            party: self.party.clone(),
            kennel: self.kennel.clone(),
            eggs: self.eggs.clone(),
            party_slots: self.party_slots(),
            ledger: self.ledger(),
            gold: crate::tree::purse(self),
            supplies: self.supplies.iter().map(|i| to_inv(i, &self.facts, &self.flavours)).collect(),
            classes: self.classes.iter().map(|(k, c)| (k.clone(), ClassProg { next: if c.level >= MAX_LEVEL { 0 } else { xp_to_next(c.level) }, ..c.clone() })).collect(),
            // Cut 9 §10: every row carries its next rung (an older save's rows too).
            // QA on 23ed91f (qaL: `heal salvaged 2/5` while the game still said `green potion?`):
            // an unidentified potion or scroll's row is keyed by what the hero calls it.
            forge: self.forge.iter().map(|(k, f)| (self.forge_name(k), { let mut f = f.clone(); f.settle(); f })).collect(),
            renown: self.renown,
            rank: self.rank,
            keep_pref: self.keep_pref.clone(),
            keep_auto: self.keep_auto(),
            insured: self.insured.clone(),
            rest_left_s: self.rest_left.div_ceil(crate::offline::TICKS_PER_SECOND as u32),
            bones: self.bones.iter().map(|b| BonesPile { depth: b.depth, heir: b.heir, items: b.items.len() as u32 }).collect(),
            ascension: Ascension { level: self.ascension, variant: self.variant.clone() },
            endgame: self.endgame.clone(),
            ascended: self.ascended.clone(),
            chronicle: self.chronicle.clone(),
            vault_pref: self.vault_pref.clone(),
            gold_ledger: self.gold_ledger.clone(),
            counters: self.counters(),
            combos: self.rules().combos(),
            picked: self.picked_clean(),
            class_offer: self.class_offer(),
            shadowed_by: self.shadowed_by(self.rules()),
            repeat: !self.restock_off,
            look: self.look.clone().unwrap_or_else(|| self.class.default_look().into()),
            repeat_kinds: self.last_supplies.iter().filter(|k| !self.last_wasted.contains(k)).cloned().collect(),
            repeat_short: self.repeat_short.clone(),
            repeat_due: Vec::new(),
            repeat_unpaid: Vec::new(),
            bounty: self.bounty.map(|depth| crate::oath::bounty_wire(self, depth)),
            // (priced on the shelf by `Game::lineage`)
            repeat_gold: 0,
            waystones: self.stones(),
            start: self.start.max(1),
            start_toll: LineageState::start_toll(self.start),
            repeat_dropped: {
                let used = self.row_kinds();
                let mut v: Vec<String> = self.last_supplies.iter().filter(|k| !used.contains(*k)).cloned().collect();
                v.dedup();
                v
            },
            trait_rules: Trait::ALL.iter().map(|t| (t.name().to_string(), t.rule().to_string())).collect(),
            start_payable: self.start_payable(self.start.max(1)),
            start_pass: self.night_passes.contains(&self.start.max(1)),
            renamed: self.renamed(),
            kit: crate::kit::ladders(self),
            guns:Vec::new(),
            row_why: self.row_why(),
            forks: self.fork_chips(),
            lanes: self.lane_list(),
            locked_rows: self.locked_rows(self.rules()),
        }
    }
    /// Cut 26 §2: the seen forks, each with the active set's stair (`Lineage.forks`).
    pub fn fork_chips(&self) -> Vec<crate::wire::ForkChip> {
        let route = self.rules().route();
        crate::descent::FORKS
            .iter()
            .enumerate()
            .filter(|(_, f)| self.facts.contains(&format!("fork:{f}")))
            .map(|(i, f)| crate::wire::ForkChip {
                depth: *f,
                near: crate::descent::BASE_ORDER[i].name().into(),
                far: crate::descent::BASE_ORDER[i + 1].name().into(),
                taken: route.biome(*f).name().into(),
                open: route.fork_open(*f),
            })
            .collect()
    }
    /// Cut 26 §2: every lit (lane, depth) pair (`Lineage.lanes`).
    pub fn lane_list(&self) -> Vec<crate::wire::LaneStone> {
        let route = self.rules().route();
        let mut v: Vec<(u32, Route)> = self.waystones.iter().map(|d| (*d, Route::BASE)).chain(self.lane_stones.iter().map(|(d, p)| (*d, Route(*p)))).collect();
        v.sort();
        v.into_iter().map(|(d, r)| crate::wire::LaneStone { depth: d, lane: r.biome(d).name().into(), route: r.forks(), current: route.prefix(d) == r }).collect()
    }
    /// Cut 26 §6: per row of `rules`, the gate of its first cond this lineage cannot use
    /// (`Lineage.locked_rows`); empty when every row's conds are open.
    pub fn locked_rows(&self, rules: &RuleSet) -> Vec<Option<String>> {
        let vocab = crate::tokens::vocabulary(self);
        let locked = crate::tokens::locked_conds(self, &vocab.conds);
        let open = |c: &crate::rules::Cond| vocab.conds.iter().any(|o| o.same_token(c)) || c.k == "party" || matches!(c.k.as_str(), "self_hp<" | "self_hp>");
        let v: Vec<Option<String>> = rules
            .rows
            .iter()
            .map(|r| {
                r.conds.iter().find(|c| !open(c)).map(|c| {
                    if let Some(l) = locked.iter().find(|l| l.cond.same_token(c)) {
                        l.needs.clone()
                    } else if c.k == "in" {
                        format!("enter {}", c.t.as_deref().unwrap_or(""))
                    } else {
                        "locked".into()
                    }
                })
            })
            .collect();
        if v.iter().all(|x| x.is_none()) {
            return Vec::new();
        }
        v
    }
    /// QA on a946e04: every identified flavoured kind's flavour label → its wire name
    /// (`Lineage.renamed`).
    pub fn renamed(&self) -> BTreeMap<String, String> {
        let f = &self.flavours;
        f.potion
            .keys()
            .chain(f.scroll.keys())
            .filter(|k| crate::item::is_identified(&self.facts, f, k))
            .map(|k| {
                let (_, _, label) = crate::item::describe(&Item::new(0, k), &BTreeSet::new(), f);
                (label, self.wire_name(k))
            })
            .collect()
    }
    /// QA on 92eb880: `rules`' shadowed rows for this lineage (a row whose condition it does not
    /// own never fires, so it shadows nothing); empty when none is shadowed.
    pub fn shadowed_by(&self, rules: &RuleSet) -> Vec<Option<u32>> {
        let v = rules.shadowed_by(self.max_rows(), |c| if c.k == "on_see" && c.t.as_deref().is_some_and(|t| !t.is_empty()) { c.t.as_deref().is_some_and(|t| self.facts.contains(t)) } else { crate::meta::cond_unlock(&c.k).is_none_or(|u| self.unlocks.contains(u)) });
        if v.iter().all(|x| x.is_none()) {
            return Vec::new();
        }
        v.into_iter().map(|x| x.map(|i| i as u32)).collect()
    }
    /// Cut 16 §2: the wake's class chips (see `Lineage.class_offer`).
    pub fn class_offer(&self) -> Vec<crate::wire::ClassChip> {
        if self.trait_offer.is_empty() {
            return Vec::new();
        }
        let mut owned: Vec<Class> = Class::ALL.iter().copied().filter(|c|(*c!=Class::Gunner||crate::firearm::UI_READY)&&c.unlock().is_none_or(|u| self.unlocks.contains(u))).collect();
        if owned.len() < 2 {
            return Vec::new();
        }
        owned.sort_by_key(|c| *c != self.class);
        owned
            .into_iter()
            .map(|c| crate::wire::ClassChip {
                class: c.name().into(),
                signature: c.signature().into(),
                level: self.classes.get(c.name()).map(|p| p.level).unwrap_or(1),
                opens: c.signature_level(),
            })
            .collect()
    }
    /// A forge row's name on the wire: the kind once known, else its flavour (`green potion?`).
    pub fn forge_name(&self, kind: &str) -> String {
        let (known, _, label) = describe(&Item::new(0, kind), &self.facts, &self.flavours);
        if known { kind.to_string() } else { label }
    }
    /// QA on 1a2a4a9 (qaO: the keep sheet's `sold blink $2 · fear ×2 $3`, the report's
    /// SALVAGED `teleport ×9 · identify ×4` while the forge and the cond list still said
    /// `brittle scroll?`): a kind on a wire line (salvage rows, the cut's `sold`, the spent
    /// rows) is its flavour (`brittle scroll?`) until the lineage identifies it; any other
    /// kind (gear, a leash, an `insure sword` line) passes through.
    pub fn wire_name(&self, kind: &str) -> String {
        if self.flavours.flavour_of(kind).is_some() && !crate::item::is_identified(&self.facts, &self.flavours, kind) {
            self.forge_name(kind)
        } else {
            kind.to_string()
        }
    }
    /// Cut 29 §4: the standing orders, read off the lineage.
    pub fn standing_orders(&self) -> crate::wire::StandingOrders {
        crate::wire::StandingOrders { keep: self.keep_pref.clone(), cage: self.vault_pref.clone(), start: self.start.max(1), repeat: !self.restock_off, insure: self.orders.insure, forge: self.orders.forge.clone() }
    }
    /// The categories an unwatched exit keeps, in order (`Game::auto_keep`).
    pub fn keep_auto(&self) -> Vec<String> {
        // Cut 29 §4: the quartermaster's keep (both categories) is the default now.
        let qm = true;
        let order: &[&str] = match self.keep_pref.as_str() {
            "best_weapon" if qm => &["weapon", "armour"],
            "best_weapon" => &["weapon"],
            "best_armour" if qm => &["armour", "weapon"],
            "best_armour" => &["armour"],
            _ => &[],
        };
        order.iter().map(|s| s.to_string()).collect()
    }
    pub fn vault_slots(&self) -> usize {
        if self.variant_is("bones_only") {
            return 0;
        }
        1 + ["vault2", "vault3", "vault4", "vault5"].iter().filter(|u| self.unlocks.contains(**u)).count()
    }
    /// Rows the rules may use: 4 plus the row unlocks (cap 10); `short_list` caps at 6.
    pub fn max_rows(&self) -> usize {
        max_rows_of(&self.unlocks, &self.variant)
    }
    pub fn party_slots(&self) -> u32 {
        1 + self.unlocks.contains("party_slot_2") as u32 + self.unlocks.contains("party_slot_3") as u32 + self.unlocks.contains("party_slot_4") as u32
    }
    /// Supplies per expedition (Cut 2 §3 `supply_cap_5`).
    pub fn supply_cap(&self) -> usize {
        // Cut 23 §1: each pack step of the forge holds one more.
        (crate::kit::base_cap(self) + crate::kit::owned(self, "pack") as usize).min(crate::kit::SUPPLY_CAP_MAX)
    }
    /// Rests an egg needs (Cut 2 §1; `incubator`).
    pub fn egg_rests(&self) -> u32 {
        if self.unlocks.contains("incubator") {
            1
        } else {
            EGG_RESTS
        }
    }
    /// Kinds tamed so far (facts `tamed:<kind>`).
    pub fn tamed_kinds(&self) -> usize {
        self.facts.iter().filter(|f| f.starts_with("tamed:")).count()
    }
    /// Distinct bosses slain.
    pub fn bosses_slain(&self) -> usize {
        self.kills.iter().filter(|k| crate::defs::monster_def(k).boss).count()
    }
    pub fn studied(&self, kind: &str) -> bool {
        self.facts.contains(&format!("foe:{kind}:studied"))
    }
    pub fn new_comp_id(&mut self) -> u32 {
        let id = self.next_comp_id;
        self.next_comp_id += 1;
        id
    }
    /// Bestiary ledger derived from facts and breeding. Cut 7 §1: the bosses follow the kinds,
    /// each carrying its counter row as a chip once the counter fact is held.
    pub fn ledger(&self) -> Vec<LedgerRow> {
        let row = |m: &crate::defs::MonsterDef| {
            let seen = self.facts.contains(&format!("foe:{}", m.kind));
            let known = seen && m.tags.iter().all(|t| self.facts.contains(&format!("foe:{}:{}", m.kind, t)));
            let counter = if m.boss {
                crate::facts::boss_counter_row(&self.facts, m.kind).map(|row| crate::wire::CounterChip { text: crate::facts::counter_text(&row), row })
            } else {
                None
            };
            LedgerRow {
                kind: m.kind.to_string(),
                seen,
                known,
                tamed: self.facts.contains(&format!("tamed:{}", m.kind)),
                bred: self.bred.contains(m.kind),
                studied: self.studied(m.kind),
                counter,
            }
        };
        let kinds = crate::defs::MONSTERS.iter().filter(|m| !m.boss && !m.tags.contains(&"summoned")).map(row);
        let bosses = crate::descent::BOSS_DEPTHS.iter().map(|(k, _)| row(crate::defs::monster_def(k)));
        kinds.chain(bosses).collect()
    }
    pub fn all_companions(&self) -> impl Iterator<Item = &Companion> {
        self.party.iter().chain(self.kennel.iter())
    }
    /// Forge tier of a kind (Addendum D): +tier to every future copy.
    pub fn forge_tier(&self, kind: &str) -> i32 {
        self.forge.get(kind).map(|f| f.tier as i32).unwrap_or(0)
    }
    /// Cut 13 §2: the next heir wakes — the heir number moves on, the deeds clear, and the
    /// camp offers two traits (`trait_offer`); the first is the heir's until `set_trait`.
    pub fn new_heir(&mut self) {
        let last = self.trait_;
        self.heir += 1;
        self.hero_legacy.push(crate::wire::HeroLegacy { heir: self.heir, class: self.class.name().into(), ..Default::default() });
        self.heir_deeds.clear();
        self.heir_best = 0;
        let first = Trait::ALL[self.rng.below(4) as usize];
        let offer = trait_offer(self.seed, self.heir + 1000 * self.ascension, Some(last), first);
        self.trait_ = offer[0];
        self.trait_offer = offer.to_vec();
        // Cut 29 §1: the heir pick (an oath's reward) offers a third trait — its own draw, so the
        // first two are the same with it or without.
        if self.unlocks.contains("heir_pick") {
            let mut r = Rng::derive(self.seed, hash_str("trait_offer3") ^ self.heir as u64);
            let rest: Vec<Trait> = Trait::ALL.iter().copied().filter(|t| Some(*t) != Some(last) && !offer.contains(t)).collect();
            if !rest.is_empty() {
                self.trait_offer.push(rest[r.below(rest.len() as u32) as usize]);
            }
        }
        // Cut 30 §3: the wake offers trait cards and passes the blood (`traits::wake`).
        crate::traits::wake(self);
        // Cut 30 §2: from heir 3 the wake deals three temperament cards (card 1 worn until a pick).
        crate::packages::wake(self);
    }
    /// Cut 16 §1: a floor's freshness at `depth`, in permille. The deepest depth the lineage
    /// has reached (and anything below it) is always fresh.
    pub fn freshness(&self, depth: u32) -> u32 {
        if depth >= self.best_depth {
            return 1000;
        }
        FRESHNESS[self.picked.get(&depth).copied().unwrap_or(0).min(PICKED_CAP) as usize]
    }

    /// Cut 16 §1: the thinned depths a run carries (`Run.thin`).
    pub fn thin_map(&self) -> BTreeMap<u32, u32> {
        self.picked.keys().map(|&d| (d, self.freshness(d))).filter(|(_, f)| *f < 1000).collect()
    }

    /// Cut 16 §1: the depths the report names `picked clean` (≥ `PICKED_SHOWN` picks, not fresh).
    pub fn picked_clean(&self) -> Vec<u32> {
        self.picked.iter().filter(|(d, p)| **p >= PICKED_SHOWN && self.freshness(**d) < 1000).map(|(d, _)| *d).collect()
    }

    /// Cut 16 §1: a run ended — a bank or a return from `exit_depth` picks it; every depth the
    /// run visited is seen tonight; the `NIGHT_RUNS`th run closes the night.
    pub fn night_run(&mut self, exit_depth: u32, max_depth: u32, picks: bool) {
        if picks && exit_depth > 0 {
            let p = self.picked.entry(exit_depth).or_insert(0);
            *p = (*p + 1).min(PICKED_CAP);
        }
        self.night_seen.extend(1..=max_depth);
        if let Some(n)=&mut self.town.shared_night_runs {
            *n+=1;
            if *n>=NIGHT_RUNS { *n=0; crate::town::night(self); }
        }
        self.night_runs += 1;
        if self.night_runs >= NIGHT_RUNS {
            self.night();
        }
    }

    /// Cut 16 §1: the night ends — each picked depth it did not visit recovers one step.
    pub fn night(&mut self) {
        let seen = std::mem::take(&mut self.night_seen);
        for (d, p) in self.picked.iter_mut() {
            if !seen.contains(d) {
                *p = p.saturating_sub(1);
            }
        }
        self.picked.retain(|_, p| *p > 0);
        self.night_runs = 0;
        // Cut 23 §1: the night's net, for the forge's `nights`.
        self.last_night_net = std::mem::take(&mut self.night_net);
        // Cut 29 §3: the night's meters.
        self.last_night_meter = std::mem::take(&mut self.night_meter);
        // Cut 30 §3: the bank pays its night's interest.
        if self.town.shared_night_runs.is_none() { crate::town::night(self); }
        // QA on a946e04: a new night buys a new waystone pass.
        self.night_passes.clear();
        self.night_short = None;
        self.night_theft_rebought = false;
        self.theft_skip.clear();
        // Cut 20 §5 (AC: "after the absence 15 of 16 runs banked, so the second half had
        // little at stake"): the deep calls — the next night's bounty floor.
        self.bounty = Some(bounty_floor_on(self.best_depth, &self.kills, self.rules().route()));
    }

    /// Cut 13 §2: pick one of the offered traits (the chip beside `♟3`); refused when it is
    /// not on offer (a send without a pick keeps the first).
    pub fn set_trait(&mut self, name: &str) -> Result<(), String> {
        // Cut 30 §3: a trait card by chip or head; else (until the temperaments go) the old offer's.
        if crate::traits::pick(self, name).is_ok() {
            return Ok(());
        }
        let t = self.trait_offer.iter().copied().find(|t| t.name() == name).ok_or_else(|| "not on offer".to_string())?;
        self.trait_ = t;
        Ok(())
    }

    /// Cut 21 §1: a bank from `depth` lights every waystone at or above it; the ones it lit.
    pub fn light_waystones(&mut self, depth: u32) -> Vec<u32> {
        self.light_waystones_on(depth, Route::BASE)
    }
    /// Cut 26 §2: a bank on `route` lights each waystone at or above its floor for the route's
    /// prefix to it (the base order's in `waystones`, a lane's in `lane_stones`); the ones it lit.
    pub fn light_waystones_on(&mut self, depth: u32, route: Route) -> Vec<u32> {
        let mut lit = Vec::new();
        for w in WAYSTONES.iter().copied().filter(|w| *w <= depth) {
            if self.stone_lit(w, route) {
                continue;
            }
            let p = route.prefix(w);
            if p.is_base() {
                self.waystones.push(w);
                self.waystones.sort_unstable();
            } else {
                self.lane_stones.push((w, p.0));
                self.lane_stones.sort_unstable();
            }
            lit.push(w);
        }
        lit
    }
    /// Cut 26 §2: whether the waystone at `depth` is lit for `route` (its prefix to the floor).
    pub fn stone_lit(&self, depth: u32, route: Route) -> bool {
        let p = route.prefix(depth);
        if p.is_base() {
            self.waystones.contains(&depth)
        } else {
            self.lane_stones.contains(&(depth, p.0))
        }
    }
    /// The waystones lit for the active set's route, ascending.
    pub fn stones(&self) -> Vec<u32> {
        let route = self.rules().route();
        WAYSTONES.iter().copied().filter(|w| self.stone_lit(*w, route)).collect()
    }

    /// Cut 21 §1: the send's start floor — 1, or a lit waystone (`set_start`). Cut 26 §2: lit
    /// for the active set's route.
    pub fn set_start(&mut self, depth: u32) -> Result<(), String> {
        if depth != 1 && !self.stone_lit(depth, self.rules().route()) {
            return Err(format!("D{depth} not lit"));
        }
        self.start = depth;
        Ok(())
    }

    /// QA on a946e04 (qaT: `start → D5 · $50` with $32, the send went from D1 unsaid): whether
    /// a send from `depth` starts there — D1, or a lit waystone whose pass is held tonight or
    /// that the purse pays now.
    pub fn start_payable(&self, depth: u32) -> bool {
        depth <= 1 || (self.stone_lit(depth, self.rules().route()) && (self.night_passes.contains(&depth) || self.gold >= LineageState::start_toll(depth)))
    }
    /// Cut 21 §1: the toll a start at `depth` pays at the send (`$0` from D1).
    pub fn start_toll(depth: u32) -> i32 {
        if depth <= 1 {
            0
        } else {
            WAYSTONE_TOLL * depth as i32
        }
    }

    /// Cut 21 §2: the supply kinds a row of the active set can use — a `drink` / `read` / `throw`
    /// naming the kind (a card's rows included), a `tame` row's leash. `unknown` names no kind:
    /// the shelf only holds kinds known by name, so the trait's and the gamble rows' unknowns
    /// never ask the repeat for anything.
    pub fn row_kinds(&self) -> BTreeSet<String> {
        self.row_kinds_of(self.rules())
    }
    /// `row_kinds` for any set (the repeat's kinds had `rules` been the active set).
    pub fn row_kinds_of(&self, rules: &RuleSet) -> BTreeSet<String> {
        let mut rows: Vec<Row> = Vec::new();
        for (_, r) in rules.active(self.max_rows()) {
            match r.card() {
                Some(c) => rows.extend(crate::meta::unlock_rows(c).unwrap_or_default()),
                None => rows.push(r.clone()),
            }
        }
        let mut out = BTreeSet::new();
        for r in rows {
            match r.verb.v.as_str() {
                "drink" | "read" | "throw" => {
                    let k = r.verb.a.as_deref().unwrap_or("").split(',').next().unwrap_or("");
                    if !k.is_empty() && k != "unknown" {
                        out.insert(k.to_string());
                    }
                }
                "tame" => {
                    out.insert("leash".to_string());
                }
                _ => {}
            }
        }
        out
    }
}

/// Cut 13 §2: the two traits offered to heir `heir` of lineage `seed`. `first` is the
/// lineage rng's draw (the trait the heir wore before this cut, so a seed's heirs keep their
/// traits) — unless it is the last heir's, when the offer's own rng (derived from the seed and
/// the heir number; an ascended lineage's heirs count from 1000 × its ascension) redraws it;
/// the second is drawn from that rng, distinct from both. The same offer on every device and
/// after a load.
pub fn trait_offer(seed: u64, heir: u32, last: Option<Trait>, first: Trait) -> [Trait; 2] {
    let mut rng = Rng::derive(seed, hash_str("trait_offer") ^ heir as u64);
    let pool: Vec<Trait> = Trait::ALL.iter().copied().filter(|t| Some(*t) != last).collect();
    let a = if Some(first) == last { pool[rng.below(pool.len() as u32) as usize] } else { first };
    let rest: Vec<Trait> = pool.into_iter().filter(|t| *t != a).collect();
    let b = rest[rng.below(rest.len() as u32) as usize];
    [a, b]
}

/// A recorded death: the wire `Death` plus what is needed to compute its verdict lazily.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DeathRec {
    pub death: Death,
    /// Cut 23 §2: the run was walking home when it died — (hp % at the commit, ticks walked).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub home: Option<(i32, u32)>,
    pub t10: Option<Run>,
    pub t10_facts: crate::shared::Shared<BTreeSet<String>>,
    pub rules: RuleSet,
    pub vocab: Vocabulary,
    pub verdict_done: bool,
    pub deltas_done: bool,
    /// Cut 4: leading patches (by survival edge) whose forecast delta is simulated, and
    /// whether the shown list has been cut and ranked (`compute_deltas`).
    #[serde(default)]
    pub deltas_n: usize,
    #[serde(default)]
    pub shaped: bool,
    /// Cut 4: the tick the killing blow landed (the replay window's end).
    #[serde(default)]
    pub death_tick: u32,
    /// Cut 6 §5: the boss this death was fought under (in view, awake and near, or the cause),
    /// and its counter row when the fact is known and the row was executable in the replays
    /// (pinned first among the patches).
    #[serde(default)]
    pub boss: Option<String>,
    #[serde(default)]
    pub counter: Option<Row>,
    /// Cut 11 §2: the killing turn's root cause when it is a theft or a locked condition —
    /// what the root-cause patch answers (`trace::root_of`).
    #[serde(default)]
    pub root: Option<Root>,
    /// Cut 13 §1: this record is a stall's (`Death.verdict` `stall`): its replays count a
    /// survival when the hero leaves the floor or the guard stays quiet, not when it lives.
    #[serde(default)]
    pub stall: bool,
    /// Cut 13: the kill counts at the checkpoint (the death-time counts less the run's kills
    /// after it), so a replay learns `studied` on the same tick the run did — the counts were
    /// the death's, and a fifth kill inside the window learned the fact a tick early in the
    /// replay, spoke a note, and spent the rng (the faithful-replay test caught it once the
    /// content moved). Absent on older records: the death-time counts stand in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t10_kill_counts: Option<BTreeMap<String, u32>>,
    /// Cut 13: the lineage state a floor generated inside the replay window reads
    /// (`populate_floor`: grudges, forge, the hunter; `place_situations`: the lost; the
    /// vision and the vault preference), as it stood at the death — a verdict read after the
    /// next runs (or a purchase) generated a different D4 in the window and the replay no
    /// longer reproduced the death. Absent on older records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t10_lineage: Option<CheckpointLineage>,
    /// Cut 14 §2: what the pack held at the death — a known heal, and how many unknown
    /// consumables — for the margin's `heal unused` / `N unknown unused`, which the verdict
    /// writes only when the candidate that uses it survived the replays (`trace::margin_lines`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub heal_held: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub unknown_held: u32,
    /// QA on 1a2a4a9: how many of `unknown_held` are scrolls (the rest are potions) — the
    /// margin counts the kind a `drink unknown` / `read unknown` row would have used.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub unknown_scrolls: u32,
    /// Cut 14 §1: the root patch measured under the baseline — not offered on the death
    /// screen (its number is the unlock sheet's delta); the gate table counts it out.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub root_under_base: bool,
    /// QA on 23ed91f: the camp state (`forecast::lineage_key`) the shown patches'
    /// `forecast_delta`s were measured on (`trace::camp_deltas`); a `death()` read on another
    /// state measures them again. 0 = not yet.
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub camp_key: u64,
    /// Cut 18 §4: a stall whose cause is the rules' loop (`R2 retreat ↔ explore`): the row it
    /// names (`Run.stuck_row`); the verdict's first patch addresses it (`trace::loop_patch`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_row: Option<usize>,
    /// Cut 19 §4: how often each row of the set acted over the dead run (`Run.row_fired`) —
    /// the least-fired own row is the one an insert on a full set drops (`Patch.drops`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_fired: Vec<u32>,
    /// A `dice` death's fallback alternatives that fired in under `FIRED_BAR` of the replays
    /// (the moment barely reached them): shown, always `below_bar` (QA on the Cut 20 gate: a
    /// 17 %-fired fallback surviving 17 % over a 0 % base was relabelled advice).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub low_fired: Vec<Row>,
    /// QA on e75ec29: the run on arriving at the death's floor and the facts then
    /// (`Game.floor_start`) — `trace::floor_fired` replays the shown patches from it. Not
    /// saved (a record read after a load skips that check).
    #[serde(skip)]
    pub floor: Option<(Run, BTreeSet<String>)>,
    /// QA on e75ec29: the verdict was measured on the floor's replays (`trace::floor_verdict`):
    /// the moment's replays did not reproduce the death, the floor's did — the baseline and the
    /// patches' survival are the floor's.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub floor_window: bool,
    /// Cut 21 §3 (AE: `FIRE · D4` read GAP after his own `drink unknown` row drank the fire):
    /// the set's own row (not a card's) whose `drink unknown` / `read unknown` gamble dealt
    /// the death — the cause is the gambled item's harm (fire, poison, caustic gas) and the
    /// gamble was this floor's, shortly before. Whatever the row's origin, the verdict is `row`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gamble_row: Option<usize>,
    /// Cut 22 §4: the set's own attack row that chased into the death (`trace::chase_cause`),
    /// set by the verdict: its `row` stands whatever an added row would survive.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chase_row: Option<usize>,
    /// Cut 25 §2: the verdict's moves (`trace::order_moves`) — every own row shadowed while its
    /// conditions held, moved above the rows that won; joined to the shown patches last.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub moves: Vec<crate::wire::Patch>,
    /// Cut 27 §5 (AT: a gas death stamped DICE after he had deleted the bloat row): the rows of
    /// the set sent before this one that this set no longer holds, each with its index there —
    /// the verdict's `restore` candidates (`LineageState::sent_sets`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<(crate::rules::Row, usize)>,
}

fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Cut 13: what a replay's floor generation reads off the lineage (see `DeathRec.t10_lineage`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CheckpointLineage {
    pub grudges: Vec<Grudge>,
    pub forge: BTreeMap<String, ForgeRow>,
    pub hunter: Option<Grudge>,
    pub lost: Vec<Lost>,
    pub unlocks: BTreeSet<String>,
    pub vault_pref: String,
}

impl CheckpointLineage {
    pub fn of(l: &LineageState) -> CheckpointLineage {
        CheckpointLineage { grudges: l.grudges.clone(), forge: l.forge.clone(), hunter: l.hunter.clone(), lost: l.lost.clone(), unlocks: l.unlocks.clone(), vault_pref: l.vault_pref.clone() }
    }
    pub fn apply(&self, l: &mut LineageState) {
        l.grudges = self.grudges.clone();
        l.forge = self.forge.clone();
        l.hunter = self.hunter.clone();
        l.lost = self.lost.clone();
        l.unlocks = self.unlocks.clone();
        l.vault_pref = self.vault_pref.clone();
    }
}

/// Cut 11 §2: a death's root — `theft` (a `because` of the killing turn is a theft: the patch
/// is the den's raid row, the `thief_guard` card or `foe_tag:thief → attack tag:thief`) or
/// `lock` (a row above the fired one read `locked cond`: the patch is the unlock, `row` the
/// locked row's index). `unlock` is the unlock id the root patch needs (the lock's condition;
/// `cond_on_see` for a den raid the lineage cannot write yet), set when the patch is built.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Root {
    pub kind: String,
    /// ≤ 6 words (`Patch.root.text`).
    pub text: String,
    pub row: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlock: Option<String>,
}

/// The vault decision waiting at an exit (Addendum D).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingExit {
    pub run_id: u32,
    pub tier: ExitTier,
    pub items: Vec<Item>,
    /// Salvage share of the unkept items (0 for a timed-out run).
    #[serde(default = "default_pct")]
    pub pct: i32,
    /// Vault items the send brought along that came home: the player's already, so the
    /// night's `auto_keep` puts them back before it keeps anything new (QA on 23ed91f).
    #[serde(default)]
    pub brought: Vec<u32>,
    /// QA on 778fa1b (qaV): the items the run found among `items` — `keep` settles their
    /// `sheet` rows on the exit line as `kept` or `salvaged`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub found: Vec<(u32, String)>,
    /// Cut 24: the exit was a stall — `keep` does not re-pack on top of the loss either (Cut
    /// 21 §2: the exit's own re-pack skips a stall; the keep's topped the shelf up after it).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stalled: bool,
}

fn default_pct() -> i32 {
    60
}

/// Accumulated outcomes since the last return report.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Batch {
    pub runs: u32,
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub legacy_earned: u32,
    /// Cut 30 §2: the packages' lines this batch (`STEADY L3`, `DRILLED · Warlord`, `+Guarded`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pkg_lines: Vec<String>,
    /// Cut 30 §1: drill items the quartermaster packed this batch.
    #[serde(default)]
    pub drill_packs: u32,
    /// Cut 28 §1: the sworn oath over the batch — the oath, the sends while it was sworn, those
    /// that kept it, and whether it was kept (then granted).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oath: Option<(crate::oath::OathState, u32, u32, bool)>,
    /// Cut 28b: the sends that broke the sworn oath, by cause (`R2 return` → 11).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub oath_breaks: BTreeMap<String, u32>,
    /// Cut 15 §1: banks from depth ≥ the lineage's best − 1 this batch (each paid ◆1). Cut 29 §1:
    /// the frontier mark is gone — always 0 (kept for saves).
    #[serde(default)]
    pub frontier_banks: u32,
    /// Cut 29 §6: the companions that fell this batch, named.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallen: Vec<crate::wire::Fallen>,
    /// Cut 29 §3: the absence's real runs metered (summed).
    #[serde(default, skip_serializing_if = "crate::meters::Meter::is_empty")]
    pub meters: crate::meters::Meter,
    /// Cut 29 §2: the systems this batch opened, in order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub systems_opened: Vec<String>,
    /// Cut 29 §1: the extra slots' oaths kept this batch (the first slot's is `oath`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub oaths_kept: Vec<crate::oath::OathState>,
    /// Cut 29 §1: the night's marks this batch paid (◆1 per day whose absences brought a send home).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub night_marks: u32,
    pub bests: Vec<String>,
    pub found: Vec<Item>,
    /// QA on 0c6e126 (qaY: the header's `new find: bow, leather` beside FOUND `mail +1`): the
    /// kinds found for the first time in this absence (the exit lines' `new find` news, every
    /// one, in order) — the report's FOUND lists them.
    #[serde(default)]
    pub new_finds: Vec<String>,
    pub deaths: BTreeMap<String, u32>,
    pub marks: u32,
    pub highlights: Vec<Highlight>,
    pub tamed: Vec<String>,
    pub hatched: Vec<String>,
    pub lost: Vec<String>,
    pub xp_gained: u32,
    pub level_ups: u32,
    pub salvaged: BTreeMap<String, (u32, i32)>,
    /// QA on 92eb880: per kind, the coins the ledger paid for `salvaged` (the report's rows).
    #[serde(default)]
    pub salvaged_coins: BTreeMap<String, i32>,
    /// Gold earned this batch by exits (kept loot) — the policy's yield, exclusive of stipends.
    pub gold_earned: i32,
    /// QA on 912e135: the carried gold this batch's exits did not keep (`GoldSummary.lost`).
    #[serde(default)]
    pub gold_lost: i32,
    /// QA on 524827b (qaAA: `$936 lost` on a report with 0 deaths): the part of `gold_lost` that
    /// exits which kept something did not keep (a return's 40 %) — `GoldSummary.unkept`.
    #[serde(default)]
    pub gold_unkept: i32,
    /// QA on 912e135: the first and last heir whose runs this batch holds (`ReturnReport.heirs`).
    #[serde(default)]
    pub heirs: Option<(u32, u32)>,
    pub renown_gained: u32,
    pub ranks_up: u32,
    pub worst_death: Option<u32>,
    pub worst_depth: u32,
    // Cut 2
    /// Runs banked / returned, rest and wake ticks consumed, the best single run's score
    /// (renown per absence), bones recovered.
    pub banked: u32,
    pub returned: u32,
    pub rested: u64,
    pub best_score: u32,
    pub bones_found: Vec<String>,
    /// QA on e75ec29: the thefts no run got back, per kind (QA on a946e04; a save from before
    /// keyed them by label) (`ReturnReport.stolen`), and the
    /// deaths that topped the heir purse up (`GoldSummary.wake_n`).
    #[serde(default)]
    pub stolen: BTreeMap<String, u32>,
    #[serde(default)]
    pub wake_n: u32,
    /// The stolen items runs got back (`Run.recovered`), beside `thefts`.
    #[serde(default)]
    pub recovered: u32,
    pub row_fired: Vec<u32>,
    /// Runs in which each row fired at least once (Cut 9 §8: `R1 fired n of m runs`).
    pub row_runs: Vec<u32>,
    pub renderable_events: u32,
    pub turns: u32,
    /// Real (simulated) runs: (final depth, death cause).
    pub run_outcomes: Vec<(u32, Option<String>)>,
    /// Ticks per real run.
    pub run_ticks: Vec<u32>,
    /// Runs ended by each rule row (`return` / `bank`), row index → count.
    pub exit_rows: BTreeMap<i32, u32>,
    /// Cut 5: the run that reached the best depth this absence (depth, run id), and the
    /// real runs that met a situation on D1–5 (§4 gate).
    pub best_run: Option<(u32, u32)>,
    pub situation_runs: u32,
    /// Cut 7 §3: per real run — the depth reached, the band situations met and passed.
    #[serde(default)]
    pub band_runs: Vec<BandRun>,
    /// Cut 6 §1: the ledger lines of the last `EXITS_CAP` exits, oldest first.
    #[serde(default)]
    pub exits: Vec<ExitLine>,
    // Cut 13
    /// §1: real runs that ended as `stalled`, and whether `worst_death` points at a stall
    /// record (a death at the same depth takes its place).
    #[serde(default)]
    pub stalls: u32,
    #[serde(default)]
    pub worst_stall: bool,
    /// QA on 778fa1b (qaV): runs with a bloodless dance — `turn::DANCE_ACTIONS` foe-facing row
    /// actions in a row with no blood drawn either way, `turn::DANCE_MOVES` of them retreats
    /// (`Run.bloodless`).
    #[serde(default)]
    pub dances: u32,
    /// Cut 27 §4: real runs with a card ↔ chore loop the oscillation guard caught (`Run.card_loops`).
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub card_loops: u32,
    /// (the loops' causes, for the metrics' breakdown; not saved)
    #[serde(skip)]
    pub loop_causes: Vec<String>,
    /// Cut 24 §1: each real run's longest no-HP stretch in hero actions (`Run.nohp`'s longest,
    /// or a boss's still stretch) — the metrics' p99 ≤ 60 row — and the runs a boss drove off.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nohp: Vec<u32>,
    #[serde(default)]
    pub driven_off: u32,
    /// Cut 26 §6: the batch's drive-offs (the last `EXITS_CAP`), for `ReturnReport.drives`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub drives: Vec<crate::wire::DrivenOff>,
    /// §3: what the automations bought this batch, per kind → (n, coins); the exact coins of
    /// salvage and of wake pay, so the night's gold reconciles to the coin
    /// (`gold_earned + salvage_gold + wake_pay − spent == the purse's delta`).
    #[serde(default)]
    pub spent: BTreeMap<String, (u32, i32)>,
    #[serde(default)]
    pub salvage_gold: i32,
    /// The waystone passages paid into the purse at this batch's sends (`GoldSummary.passage`).
    #[serde(default)]
    pub passage: i32,
    #[serde(default)]
    pub wake_pay: i32,
    /// Cut 19 §3: the repeat skipped a kind this batch because the batch's spending had
    /// reached its income (`Game::restock`).
    #[serde(default)]
    pub restock_capped: bool,
    /// QA on 1a2a4a9: a re-pack of this absence ran short of gold (`repeat short`).
    #[serde(default)]
    pub repeat_short: bool,
    /// Cut 21 §2: found supplies put on the shelf at the exits, per kind — (count, their
    /// price on the shelf).
    #[serde(default)]
    pub shelved: BTreeMap<String, (u32, i32)>,
    // Cut 20
    /// §1: thefts suffered this batch (every thief's, the den's included) and the dens that
    /// woke (pounced) — the measures behind "a thief steals once per run".
    #[serde(default)]
    pub thefts: u32,
    #[serde(default)]
    pub den_wakes: u32,
    /// §5: the bounty the absence played for (`ReturnReport.bounty`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounty: Option<crate::wire::BountyReport>,
    /// QA on a946e04 (qaT): the waystone the batch's sends could not pay the pass for — (depth,
    /// toll, sends that went from D1 instead) (`ReturnReport.start_short`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_short: Option<(u32, i32, u32)>,
    /// QA on a946e04 (qaT: `−$36 stolen` in the strip, STOLEN listing the item only): the
    /// carried gold each kept theft took off the run (the item's worth), per kind.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub stolen_gold: BTreeMap<String, i32>,
    /// QA on 778fa1b: the carried gold the batch's pack swaps took off (Σ `ExitLine.swapped`).
    #[serde(default)]
    pub swapped: i32,
}

impl Batch {
    /// Cut 19 §3: what the batch brought in — the exits' kept gold, the salvage, the wake pay.
    pub fn income(&self) -> i32 {
        self.gold_earned + self.salvage_gold + self.wake_pay
    }
    /// What the automations (the repeat, the insurance) spent this batch.
    pub fn spent_total(&self) -> i32 {
        self.spent.values().map(|(_, g)| *g).sum()
    }
}

/// Cut 7 §3: what a run met and answered of the band situations.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BandRun {
    pub depth: u32,
    pub met: Vec<String>,
    pub passed: Vec<String>,
}

/// The stall verdict's window: runs since the last death, new best depth or rule edit, the
/// deepest floor they reached and the rows that ended them. Lives on the game (not the batch)
/// so a chunked absence (30-minute slices, ~1 run each) still adds up.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StallTally {
    pub runs: u32,
    pub depth: u32,
    pub exit_rows: BTreeMap<i32, u32>,
    /// Cut 9 §5: the last-5 trace of the latest run each row ended.
    #[serde(default)]
    pub traces: BTreeMap<i32, Trace>,
    /// QA on 92eb880 (qaN: `R7 bank ended 13 runs` beside `11 BANKED`): the window's exits
    /// since the last watched run — the absence the report describes; the rest came before.
    #[serde(default)]
    pub absent_rows: BTreeMap<i32, u32>,
}

/// What a finished run contributed (for offline accounting).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RunOutcome {
    pub run_id: u32,
    pub depth: u32,
    pub tier: Option<ExitTier>,
    pub cause: Option<String>,
    pub new_facts: u32,
    pub new_best: bool,
    pub turns: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub version: u32,
    pub lineage: LineageState,
    pub run: Option<Run>,
    pub deaths: BTreeMap<u32, DeathRec>,
    pub loadout: Vec<u32>,
    pub sim: bool,
    pub history: VecDeque<(Run, crate::shared::Shared<BTreeSet<String>>)>,
    /// QA on e75ec29: the live run as it stood on arriving at its current floor, with the
    /// facts then — a death's patches are replayed from here as well (`trace::floor_fired`):
    /// a row the death's short window fires but the floor never reaches is not the fix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor_start: Option<(Run, crate::shared::Shared<BTreeSet<String>>)>,
    /// Cut 11 §1: the live run's provenance log (`provenance.rs`; cap `PROV_CAP`) — the
    /// events a row reason's `because` points at. Cleared at `start_run`; off the run so the
    /// history ring's clones do not carry it; empty on sims.
    #[serde(default)]
    pub prov: Vec<crate::provenance::Prov>,
    pub reel: Vec<Highlight>,
    pub last_snapshot: Option<Snapshot>,
    pub events: Vec<Ev>,
    pub stall_runs: u32,
    pub pending_exit: Option<PendingExit>,
    pub batch: Batch,
    pub facts_at_run_start: usize,
    /// Death records kept (the metrics raise this to keep every death of a batch).
    #[serde(default = "default_max_deaths")]
    pub max_deaths: usize,
    /// Inside an offline batch: renown settles once per absence (Cut 2 §2).
    #[serde(default)]
    pub offline: bool,
    /// One resumable absence; internal transport calls do not settle reports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offline_absence: Option<crate::offline::Absence>,
    /// Runs since the last death / new best / rule edit (the stall verdict's window).
    #[serde(default)]
    pub stall: StallTally,
    /// The stall patches last forecast, keyed by (row, depth, rules, vocabulary); the text is cheap.
    #[serde(skip)]
    pub stall_cache: Option<(String, Vec<Patch>)>,
    /// Cut 4: `forecast::reach_with` results keyed by (lineage, rules, depth, sims, tag).
    #[serde(skip)]
    pub forecast_cache: std::cell::RefCell<BTreeMap<String, (f64, u32)>>,
    /// QA on 23ed91f: the camp's forecast panels (`forecast::camp_panel`) keyed by (lineage,
    /// rules) — a death screen measures its patches on the panels the camp then reads, so the
    /// camp after a tap is a lookup.
    #[serde(skip)]
    pub panel_cache: std::cell::RefCell<BTreeMap<String, Vec<crate::forecast::SimResult>>>,
    /// QA on 1a2a4a9: the panel keys refined at `REFINE_SIMS` (`forecast::camp_sims`) — a
    /// refined panel evicted from `panel_cache` is recomputed refined, never read coarser.
    /// QA on a946e04 (qaS: `return 90% death 10%` → `88% · 12%` across a reload, nothing
    /// edited): saved — the save keeps the keys of the lineage as it stands
    /// (`save::save`), so a load's first read of a refined set is the refined panel again.
    #[serde(default, skip_serializing_if = "refined_empty")]
    pub refined_panels: std::cell::RefCell<std::collections::BTreeSet<String>>,
    /// Cut 6 §1: the ledger line of the last settled exit (`step` attaches it to `Ev::Exit`).
    #[serde(default)]
    pub last_exit: Option<ExitLine>,
    /// QA on e75ec29: the bounty floor the camp last showed — the lineage's at a load, a new
    /// game, a send or a camp edit (rules, loadout); an absence's batches (`run_offline`, in
    /// slices) never move it. The report names a bounty only when it is this one.
    #[serde(skip)]
    pub bounty_seen: Option<u32>,
    /// Cut 7 §5: the live run is being watched (`send()` / `step()` set it, the offline batch
    /// clears it): a watched bank grants +50% class XP.
    #[serde(default)]
    pub watched: bool,
    /// Cut 23 §3: the live run's why-not tally per row of the set (`turn::tally_rows`), off the
    /// run so the history ring's clones do not carry it; folded into
    /// `LineageState::row_stats` at the run's end.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub row_tally: Vec<RowTally>,
    /// Cut 27 §1: the passage a send from a waystone is paid — (start, coins), set by `send` and
    /// at an absence's start (`forecast::sim_passage`), read by `start_run` when the run starts
    /// on that floor; the sims carry their own (`forecast::simulate_one`).
    #[serde(skip)]
    pub passage: Option<(u32, i32)>,
    /// Cut 27 §1: the floors the watch folds for the send in flight — (last folded floor, each
    /// floor's clear from the start), from the camp's panel at `send` (`Game::fold`).
    #[serde(skip)]
    pub fold_plan: Option<(u32, Vec<(u32, f64)>)>,
    /// Cut 28 §2: the lineage (and loadout) at the last real send — what the camp's forecast then
    /// stood on, so a move since can be split into what the state did and what the rows did
    /// (`forecast::forecast_move`). Saved (a mirror lane loads the save); never on a sim clone.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_state: Option<Box<SentState>>,
    /// RUNS_UI: the last `CAPSULES` real sends, to replay (`replay`); memory only.
    #[serde(skip)]
    pub capsules: Capsules,
    /// RUNS_UI (tests): every real run's events by run id (its send's pending events, then each tick's).
    #[serde(skip)]
    pub tap: Option<BTreeMap<u32, Vec<Ev>>>,
    /// RUNS_UI: the lineage clock at the end of the run settling now (an absence's or `advance`'s
    /// clock inside its budget); None reads `LineageState::clock_s`.
    #[serde(skip)]
    pub clock_at: Option<u64>,
    /// RUNS_UI: `advance`'s remainders — ms short of a tick, ticks short of a second.
    // Milliseconds and fractional clock ticks are continuation state, including
    // background bloodlines saved between 100 ms scheduler steps.
    #[serde(default, skip_serializing_if = "advance_remainder_empty")]
    pub advance_rem: (u64, u64),
}

fn advance_remainder_empty(value: &(u64,u64)) -> bool { *value == (0,0) }

/// Cut 28 §2: the camp state a send left from (`Game::sent_state`) — the lineage less its history
/// (the chronicle, the ledger, the row tallies: nothing a sim reads) and the loadout.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SentState {
    pub lineage: LineageState,
    #[serde(default)]
    pub loadout: Vec<u32>,
}

impl SentState {
    pub fn of(game: &Game) -> SentState {
        let mut l = game.lineage.clone();
        l.chronicle.clear();
        l.gold_ledger.clear();
        l.row_stats.clear();
        l.reel_pairs.clear();
        l.event_recent.clear();
        l.graveyard.clear();
        SentState { lineage: l, loadout: game.loadout.clone() }
    }
}

fn refined_empty(r: &std::cell::RefCell<std::collections::BTreeSet<String>>) -> bool {
    r.borrow().is_empty()
}

fn default_max_deaths() -> usize {
    40
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            version: SAVE_VERSION,
            lineage: LineageState::new(seed),
            run: None,
            deaths: BTreeMap::new(),
            loadout: Vec::new(),
            sim: false,
            history: VecDeque::new(),
            floor_start: None,
            prov: Vec::new(),
            reel: Vec::new(),
            last_snapshot: None,
            events: Vec::new(),
            stall_runs: 0,
            pending_exit: None,
            batch: Batch::default(),
            facts_at_run_start: 0,
            max_deaths: 40,
            offline: false,
            offline_absence: None,
            last_exit: None,
            watched: false,
            bounty_seen: None,
            stall: StallTally::default(),
            stall_cache: None,
            forecast_cache: Default::default(),
            panel_cache: Default::default(),
            refined_panels: Default::default(),
            row_tally: Vec::new(),
            passage: None,
            fold_plan: None,
            sent_state: None,
            capsules: Capsules::default(),
            tap: None,
            clock_at: None,
            advance_rem: (0, 0),
        };
        // Cut 28 §1: the oath board is drawn with the lineage.
        crate::oath::refresh(&mut g.lineage);
        // Cut 29: the free vocabulary its gates open, and the day-0 systems.
        crate::meta::grant_free(&mut g.lineage);
        crate::systems::update(&mut g.lineage, false);
        // Cut 30 §1: every new lineage climbs on the school stance (`Steady`, compiled).
        crate::packages::init(&mut g.lineage);
        g
    }

    /// A harness's lineage (bots, tests): the class preset as written, nothing compiled — the
    /// pre-Cut 30 behaviour (`packages::make_literal`).
    pub fn new_literal(seed: u64) -> Game {
        let mut g = Game::new_resident(seed);
        crate::packages::make_literal(&mut g.lineage);
        g
    }

    /// Gameplay harnesses explicitly complete the free opening construction.
    pub fn new_resident(seed: u64) -> Game {
        let mut g = Game::new(seed);
        g.build_town("house").expect("free first home");
        g
    }

    /// A lightweight copy for simulations: same lineage and run, no records.
    pub fn sim_clone(&self) -> Game {
        Game {
            version: self.version,
            lineage: self.lineage.clone(),
            run: self.run.clone(),
            deaths: BTreeMap::new(),
            loadout: self.loadout.clone(),
            sim: true,
            history: VecDeque::new(),
            floor_start: None,
            prov: Vec::new(),
            reel: Vec::new(),
            last_snapshot: None,
            events: Vec::new(),
            stall_runs: 0,
            pending_exit: None,
            batch: Batch::default(),
            facts_at_run_start: self.lineage.facts.len(),
            max_deaths: 0,
            offline: false,
            offline_absence: None,
            stall: StallTally::default(),
            stall_cache: None,
            forecast_cache: Default::default(),
            panel_cache: Default::default(),
            refined_panels: Default::default(),
            row_tally: Vec::new(),
            last_exit: None,
            watched: false,
            bounty_seen: self.bounty_seen,
            passage: self.passage,
            fold_plan: None,
            sent_state: None,
            capsules: Capsules::default(),
            tap: None,
            clock_at: None,
            advance_rem: (0, 0),
        }
    }

    pub fn lineage(&self) -> Lineage {
        let mut l = self.lineage.to_wire();
        if self.lineage.endgame.is_some() || self.lineage.ended { l.endgame = Some(self.descent_progress()); }
        l.return_pick = crate::returns::wire(&self.lineage);
        let (kinds, gold) = self.repeat_plan();
        l.repeat_kinds = kinds;
        l.repeat_gold = gold;
        let (due, unpaid) = self.repeat_at_send();
        l.repeat_due = due;
        l.repeat_unpaid = unpaid;
        l.repeat_added = self.repeat_adds();
        // Cut 30.5: the works tree (the hero's wait is the game's: no run under way, before the scout)
        if !self.lineage.pkg.literal {
            l.tree = Some(crate::tree::wire(&self.lineage, self.waits()));
        }
        // RUNS_UI: the run under way and the runs a replay is held for
        l.live = self.live_run();
        l.guns=crate::firearm::offers(&self.lineage,self.run.is_some());
        l.class_styles=Some(crate::specialization::offers(&self.lineage,self.run.is_some()));
        l.legacy_upgrades = crate::legacy::offers(&self.lineage, crate::legacy::away(self));
        l.legacy_respec = Some(crate::legacy::respec_offer(&self.lineage,crate::legacy::away(self)));
        l.replays = self.capsules.0.iter().map(|c| c.id).collect();
        l
    }

    /// RUNS_UI: the run under way (begun, not over).
    pub fn live_run(&self) -> Option<crate::wire::LiveRun> {
        self.run.as_ref().filter(|r| r.turn > 0 && r.over.is_none()).map(|r| crate::wire::LiveRun { activity: if r.homeward.is_some() { "returning" } else if r.meters.quiet < crate::meters::FIGHT_GAP && r.meters.run.fights > 0 { "combat" } else { "exploring" }.into(), run_id: r.id, heir: r.heir, depth: r.depth, start: r.start.max(1), hp: r.hero.hp, max_hp: r.hero.max_hp, turn: r.turn })
    }

    /// RUNS_UI: the open app's clock (`offline::advance`).
    pub fn advance(&mut self, elapsed_ms: u64) -> crate::wire::Advance {
        crate::offline::advance(self, elapsed_ms)
    }

    /// RUNS_UI: a held run re-simulated from its send — floor by floor (each floor's first snapshot
    /// with what it later showed folded in, as the watch's run log keeps it) and its events, the
    /// inputs it took applied at their ticks. The exit is not settled (no verdict's cost).
    pub fn replay(&self, run_id: u32) -> Option<crate::wire::Replay> {
        let cap = self.capsules.0.iter().find(|c| c.id == run_id)?;
        without_history(|| {
            let mut g = (*cap.game).clone();
            g.events = cap.pre.clone();
            let mut all: Vec<Ev> = Vec::new();
            let mut floors: Vec<crate::wire::ReplayFloor> = Vec::new();
            let mut cur = crate::wire::ReplayFloor { snapshot: g.snapshot(), events: Vec::new() };
            let mut inputs = cap.inputs.iter().peekable();
            let mut n = 0u32;
            while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < MAX_TURNS_PER_RUN {
                let turn = g.run.as_ref().map_or(0, |r| r.turn);
                while let Some((_, inp)) = inputs.next_if(|(t, _)| *t <= turn) {
                    match inp {
                        Input::Choose(id) => {
                            let _ = g.choose(*id);
                        }
                        Input::Control(on) => { let _ = g.take_control(*on); }
                        Input::Act(a) => { let _ = g.act(a.clone()); }
                        Input::Bail => g.bail(),
                    }
                }
                g.tick();
                n += 1;
                let evs = std::mem::take(&mut g.events);
                all.extend(evs.iter().cloned());
                match evs.iter().position(|e| matches!(e, Ev::Descend { .. })) {
                    Some(k) if g.run.as_ref().is_some_and(|r| r.over.is_none()) => {
                        cur.events.extend(evs[..=k].iter().cloned());
                        floors.push(std::mem::replace(&mut cur, crate::wire::ReplayFloor { snapshot: g.snapshot(), events: evs[k + 1..].to_vec() }));
                    }
                    _ => {
                        cur.events.extend(evs);
                        if n.is_multiple_of(20) && g.run.as_ref().is_some_and(|r| r.over.is_none()) {
                            fold_into(&mut cur.snapshot, &g.snapshot());
                        }
                    }
                }
            }
            if g.run.is_some() {
                fold_into(&mut cur.snapshot, &g.snapshot());
            }
            floors.push(cur);
            Some(crate::wire::Replay { run_id, hash: events_hash(&all), floors, ticks: g.run.as_ref().map_or(n, |r| r.turn) })
        })
    }

    /// QA on 778fa1b (qaU: after the absence the loadout read `1/3` — the heal not re-packed
    /// after the last death, the absence's re-packs capped at its income — while the tile said
    /// `repeat on · $26`): what the next send's re-pack (`restock` at `start_run`, after the
    /// toll) does with the shelf as it stands — the kinds it buys then (`due`) and the kinds
    /// the purse (or the shelf's cap) will not let it (`unpaid`: the tile's `repeat short`).
    /// Both empty when the shelf already holds the repeat, or the repeat is off.
    pub fn repeat_at_send(&self) -> (Vec<String>, Vec<String>) {
        let l = &self.lineage;
        if l.restock_off {
            return (Vec::new(), Vec::new());
        }
        let cat = self.supply_catalogue();
        let mut on_shelf: Vec<String> = l.supplies.iter().filter(|s| !s.free).map(|s| s.kind.clone()).collect();
        let used = l.row_kinds();
        let mut skip = l.theft_skip.clone();
        let want = l.start.max(1);
        let toll = if want > 1 && l.stone_lit(want, l.rules().route()) && !l.night_passes.contains(&want) && l.gold >= LineageState::start_toll(want) { LineageState::start_toll(want) } else { 0 };
        let mut gold = l.gold - toll;
        let mut n = l.supplies.len();
        let (mut due, mut unpaid) = (Vec::new(), Vec::new());
        for kind in &l.last_supplies {
            if let Some(i) = on_shelf.iter().position(|k| k == kind) {
                on_shelf.remove(i);
                continue;
            }
            if l.last_wasted.contains(kind) || !used.contains(kind) {
                continue;
            }
            if let Some(i) = skip.iter().position(|k| k == kind) {
                skip.remove(i);
                continue;
            }
            let Some(price) = self.repeat_price(kind, &cat) else { continue };
            if n >= l.supply_cap() || gold < price {
                unpaid.push(kind.clone());
            } else {
                gold -= price;
                n += 1;
                due.push(kind.clone());
            }
        }
        (due, unpaid)
    }

    /// Cut 12 §1: a player's set is validated at the door — at most `max_rows` own rows (a
    /// card's row is the card's, outside the cap), one row per card, every card owned — and
    /// refused with a ≤ 6-word error otherwise (it was truncated). A set the lineage already
    /// holds over the cap (the rules stay through an ascension, which resets the row unlocks)
    /// may be edited but never grown: the rows in play are `RuleSet::active`. A sim's set (a
    /// patch replay, a forecast candidate) is cut like the editor cuts it: the last own row
    /// falls off (`RuleSet::fit`).
    pub fn set_rules(&mut self, set: RuleSet) -> Result<(), String> {
        let mut set = set;
        let max_rows = self.lineage.max_rows();
        if self.sim {
            set = set.fit(max_rows);
        } else {
            let held = self.lineage.rules();
            let cap = max_rows.max(held.own_rows());
            let mut owned = self.lineage.unlocks.clone();
            owned.extend(held.rows.iter().filter_map(|r| r.card().map(String::from)));
            set.check_rows(cap, &owned)?;
        }
        set.validate()?;
        // Cut 9 §1: a locked token is refused at the door (the sheet never offers it; a sim
        // replays what the lineage already holds, locks and all).
        if !self.sim {
            let locked = crate::tokens::locked_conds(&self.lineage, &crate::tokens::vocabulary(&self.lineage).conds);
            for (i, r) in set.rows.iter().enumerate() {
                for c in &r.conds {
                    if let Some(l) = locked.iter().find(|l| l.cond.same_token(c)) {
                        return Err(format!("row {}: {} is locked ({})", i + 1, c.short(), l.needs));
                    }
                    // Cut 26 §2: a biome never entered is no cond yet (seen at a fork or not).
                    if c.k == "in" && !self.lineage.facts.contains(&format!("biome:{}", c.t.as_deref().unwrap_or(""))) {
                        return Err(format!("row {}: {} is locked (enter {})", i + 1, c.short(), c.t.as_deref().unwrap_or("")));
                    }
                }
            }
            // Cut 26 §2: a route takes only forks the hero has seen.
            for f in &set.route {
                if !self.lineage.facts.contains(&format!("fork:{f}")) {
                    return Err(format!("D{f} fork unseen"));
                }
            }
        }
        // Cut 7 §2: a card's row is the card's wherever it came from; the client tags
        // patch/player rows itself.
        for r in set.rows.iter_mut() {
            if r.origin.is_none() && r.verb.v == "tactic" {
                r.origin = Some("card".into());
            }
        }
        if !self.sim {
            self.bounty_seen = self.lineage.bounty;
        }
        let i = self.lineage.active_set.min(self.lineage.sets.len() - 1);
        if self.lineage.sets[i] != set {
            self.stall = StallTally::default();
        }
        // Cut 30 §2: a lineage on packages keeps the rows the pen wrote (above every package) and
        // recompiles; a sim's set, and a harness's literal lineage, are the set as written.
        if !self.sim && !self.lineage.pkg.literal {
            crate::packages::absorb(&mut self.lineage, &set, true);
            crate::packages::recompile(&mut self.lineage);
            return Ok(());
        }
        self.lineage.sets[i] = set;
        Ok(())
    }

    /// `set_rules` without the Cut 9 lock check: tests and tools that write rows ahead of
    /// the facts (a row with a locked token exists in play too — after an ascension, or when
    /// a companion is lost — and reads `locked cond` in the trace).
    pub fn set_rules_raw(&mut self, set: RuleSet) -> Result<(), String> {
        let sim = std::mem::replace(&mut self.sim, true);
        let r = self.set_rules(set);
        self.sim = sim;
        r
    }

    pub fn select_set(&mut self, i: usize) {
        if self.lineage.active_set != i.min(self.lineage.sets.len() - 1) {
            self.stall = StallTally::default();
        }
        self.lineage.active_set = i.min(self.lineage.sets.len() - 1);
    }

    pub fn set_class(&mut self, class: &str) -> Result<(), String> {
        let c = Class::parse(class).ok_or("unknown class")?;
        if (c==Class::Gunner||self.lineage.class==Class::Gunner)&&self.run.is_some() {
            return Err("hero away".into());
        }
        if let Some(u) = c.unlock() {
            if !self.lineage.unlocks.contains(u) {
                return Err(format!("{u} not unlocked"));
            }
        }
        let was_gunner=self.lineage.class==Class::Gunner;
        self.lineage.class = c;
        if c==Class::Gunner {self.lineage.classes.entry("gunner".into()).or_insert(ClassProg{level:1,xp:0,next:0});}
        if was_gunner||c==Class::Gunner||!self.lineage.specializations.is_empty() {crate::packages::recompile(&mut self.lineage);}
        Ok(())
    }

    pub fn set_keep_pref(&mut self, pref: &str) -> Result<(), String> {
        if !["best_weapon", "best_armour", "none"].contains(&pref) {
            return Err("unknown keep_pref".into());
        }
        self.lineage.keep_pref = pref.into();
        Ok(())
    }

    /// Hero looks: the heirs' cosmetic look (`Class::LOOKS`); inherited, never read by a run.
    pub fn set_look(&mut self, look: &str) -> Result<(), String> {
        if !crate::hero::Class::LOOKS.contains(&look) {
            return Err("unknown look".into());
        }
        self.lineage.look = Some(look.into());
        Ok(())
    }

    /// Cut 5 §4: what an unwatched vault choice takes.
    pub fn set_vault_pref(&mut self, pref: &str) -> Result<(), String> {
        if !["weapon", "armour", "potion", "scroll"].contains(&pref) {
            return Err("unknown vault_pref".into());
        }
        self.lineage.vault_pref = pref.into();
        Ok(())
    }

    /// Cut 19 §3: the loadout's repeat (`repeat · $120` on the camp's tile). Off: the shelf's
    /// re-packed supplies are refunded at their price and no send re-packs until it is on
    /// again (the kinds are kept); on: the shelf is re-packed now if it is empty.
    pub fn set_restock(&mut self, on: bool) {
        // QA on 1a2a4a9: the same value again is no order — it neither re-packs an emptied
        // shelf nor refunds what the player bought by hand (the camp's forecast inputs stay).
        if self.lineage.restock_off != on {
            return;
        }
        self.lineage.restock_off = !on;
        if on {
            self.restock_at(false);
            return;
        }
        let cat = self.supply_catalogue();
        let (bought, kept): (Vec<Item>, Vec<Item>) = std::mem::take(&mut self.lineage.supplies).into_iter().partition(|s| !s.free && !s.found);
        self.lineage.supplies = kept;
        for s in bought {
            if let Some(price) = refund_of(&s, &cat) {
                let why = format!("refund {}", crate::item::kind_name(&s.kind));
                self.lineage.gold_move_n(price, &why, 1);
            }
        }
    }

    /// Cut 19 §3: what the repeat packs at the next send — the last send's kinds less the
    /// ones it wasted — and their price on the shelf today.
    ///
    /// QA on 1a2a4a9 (qaP: `repeat · $40` with a heal and a mapping scroll on the shelf, then
    /// `−$60 repeat mapping · −$40 repeat heal`): a shelf holding bought supplies is what the
    /// send packs, and so what the re-pack after it buys again (`start_run` makes it the last
    /// send's); an empty shelf re-packs the last send's kinds at the send.
    ///
    /// Cut 21 §2: only the kinds a row can use (`LineageState::row_kinds`); a found supply on
    /// the shelf is packed free and is not the repeat's — on an empty shelf of bought ones it
    /// stands in for one of the kind the re-pack would buy.
    /// QA on 0c6e126: what the repeat pays for `kind` — its quote (`LineageState::repeat_quote`), never above the shelf's price
    /// today; None when the shelf does not sell it.
    pub fn repeat_price(&self, kind: &str, cat: &[SupplyInfo]) -> Option<i32> {
        let shelf = cat.iter().find(|e| e.kind == kind)?.price;
        Some(self.lineage.repeat_quote.get(kind).map_or(shelf, |q| (*q).min(shelf)))
    }

    pub fn repeat_plan(&self) -> (Vec<String>, i32) {
        let cat = self.supply_catalogue();
        let used = self.lineage.row_kinds();
        let shelf: Vec<String> = self.lineage.supplies.iter().filter(|s| !s.free && !s.found).map(|s| s.kind.clone()).collect();
        let kinds: Vec<String> = if !shelf.is_empty() {
            shelf
        } else {
            let mut found: Vec<&str> = self.lineage.supplies.iter().filter(|s| s.found).map(|s| s.kind.as_str()).collect();
            self.lineage
                .last_supplies
                .iter()
                .filter(|k| !self.lineage.last_wasted.contains(k))
                .filter(|k| match found.iter().position(|f| f == k) {
                    Some(i) => {
                        found.remove(i);
                        false
                    }
                    None => true,
                })
                .cloned()
                .collect()
        };
        let mut skip = self.lineage.theft_skip.clone();
        let kinds: Vec<String> = kinds
            .into_iter()
            .filter(|k| used.contains(k))
            // Cut 22 §1: a kind thieves kept after the night's one re-buy waits for the night's end.
            .filter(|k| match skip.iter().position(|s| s == k) {
                Some(i) if self.lineage.supplies.iter().all(|s| s.kind != *k) => {
                    skip.remove(i);
                    false
                }
                _ => true,
            })
            .collect();
        let gold = kinds.iter().filter_map(|k| self.repeat_price(k, &cat)).sum();
        (kinds, gold)
    }

    /// Cut 5 §5: bail — a `return` fires on the hero's next action, the rules untouched.
    pub fn bail(&mut self) {
        if let Some(run) = self.run.as_mut() {
            if run.over.is_none() {
                run.bail = true;
                let (id, t) = (run.id, run.turn);
                self.input(id, t, Input::Bail);
            }
        }
    }

    /// Take control (a secondary mode): the player chooses the watched run's hero actions until released.
    pub fn take_control(&mut self, on: bool) -> Result<(), String> {
        let run = self.run.as_mut().filter(|r| r.over.is_none()).ok_or("no run")?;
        run.manual = on;
        if !on {
            run.manual_act = None;
            run.awaiting = false;
        }
        let (id, t) = (run.id, run.turn);
        self.input(id, t, Input::Control(on));
        Ok(())
    }

    /// Take control: the hero's next action (the world waits for it at his turn).
    pub fn act(&mut self, a: Manual) -> Result<(), String> {
        let run = self.run.as_mut().filter(|r| r.over.is_none() && r.manual).ok_or("not in control")?;
        if let Manual::Step { dx, dy } = a {
            if dx.abs() > 1 || dy.abs() > 1 || (dx == 0 && dy == 0) {
                return Err("one step".into());
            }
        }
        run.manual_act = Some(a.clone());
        let (id, t) = (run.id, run.turn);
        self.input(id, t, Input::Act(a));
        Ok(())
    }

    /// RUNS_UI: an input the live run took, on its capsule (replayed at its tick).
    fn input(&mut self, id: u32, t: u32, inp: Input) {
        if let Some(c) = self.capsules.0.iter_mut().rev().find(|c| c.id == id) {
            c.inputs.push((t, inp));
        }
    }

    /// Cut 5 §4: take one item of the opened vault (`Snapshot.vault_choice`); the rest vanish.
    pub fn choose(&mut self, item_id: u32) -> Result<(), String> {
        // A choice after the run ended (the hero died on the vault tile) must be an error, not a
        // panic: `ctx()` expects a live run and a wasm panic poisons the whole engine object.
        if self.run.as_ref().is_none_or(|r| r.over.is_some()) {
            return Err("no run".into());
        }
        let (run, mut cx) = self.ctx();
        if run.vault_choice.is_none() {
            return Err("no vault open".into());
        }
        if !run.vault_choice.as_ref().unwrap().1.iter().any(|i| i.id == item_id) {
            return Err("not in the vault".into());
        }
        let n0 = cx.events.len();
        crate::turn::vault_take(run, &mut cx, Some(item_id));
        let (id, t) = (run.id, run.turn);
        self.input(id, t, Input::Choose(item_id));
        if let Some(tap) = self.tap.as_mut() {
            tap.entry(id).or_default().extend(self.events[n0..].iter().cloned());
        }
        Ok(())
    }

    pub fn loadout(&mut self, ids: Vec<u32>) {
        self.bounty_seen = self.lineage.bounty;
        if self.lineage.variant_is("bones_only") {
            self.loadout.clear();
            return;
        }
        self.loadout = ids.into_iter().filter(|id| self.lineage.vault.iter().any(|v| v.id == *id)).collect();
        // Cut 30.5: a find worn by hand (each vault item once) counts toward the armourer
        for id in self.loadout.clone() {
            if !self.lineage.tree.worn.contains(&id) && !crate::tree::hired(&self.lineage, "armourer") {
                self.lineage.tree.worn.push(id);
                crate::tree::did(&mut self.lineage, "wear");
            }
        }
    }

    /// Cut 3: a new lineage after the ending that keeps classes (levels), kennel, vault, ledger,
    /// forge and facts; marks, gold, heirs, the descent and the unlocks start over (the second
    /// act re-buys them; class unlocks and mastery cards stay with the classes). The rules stay.
    /// Variants add a system each: `no_rest` (no rest verb, camp rest halved), `short_list`
    /// (six rows; tactic cards carry), `bones_only` (no vault), `hunted` (a grudge from the last
    /// lineage stalks every floor from D3).
    pub fn ascend(&mut self, variant: &str) -> Result<(), String> {
        if !self.lineage.ended {
            return Err("the dungeon has a bottom: reach it first".into());
        }
        if !VARIANTS.contains(&variant) {
            return Err(format!("unknown variant {variant}"));
        }
        self.run = None;
        self.pending_exit = None;
        self.history.clear();
        self.deaths.clear();
        self.reel.clear();
        self.batch = Batch::default();
        self.stall = StallTally::default();
        self.stall_cache = None;
        self.last_snapshot = None;
        self.loadout.clear();
        let l = &mut self.lineage;
        // Cut 5 §2: the ascending heir's line.
        l.chronicle_heir("ascended", None);
        l.heir_deeds.clear();
        l.heir_best = 0;
        l.chronicled = 0;
        l.lost.clear();
        l.ascension += 1;
        if let Some(p) = &mut l.endgame { p.tier = 0; }
        l.variant = variant.into();
        l.ended = false;
        l.heir = 1;
        l.marks = 0;
        let gold = l.gold;
        l.gold_ledger.clear();
        l.gold_move(-gold, "ascended");
        l.gold_carry = 0;
        l.best_depth = 0;
        l.banked_depths.clear();
        l.waystones.clear();
        l.lane_stones.clear();
        l.start = 1;
        // Cut 23 §1: the forge starts over with the gold.
        l.kit.clear();
        l.night_net = 0;
        l.last_night_net = 0;
        l.renown = 0;
        l.rank = 0;
        l.graveyard.clear();
        l.bones.clear();
        l.insured.clear();
        l.supplies.clear();
        l.last_supplies.clear();
        l.last_supply_origins.clear();
        l.rest_left = 0;
        l.runs_banked = 0;
        l.runs_returned = 0;
        l.runs_died = 0;
        // Cut 13 §2: the ascended lineage's first heir is a new heir — two traits on offer.
        let first = Trait::ALL[l.rng.below(4) as usize];
        let offer = trait_offer(l.seed, l.heir + 1000 * l.ascension, Some(l.trait_), first);
        l.trait_ = offer[0];
        l.trait_offer = offer.to_vec();
        crate::traits::wake(l);
        // The hunter: the deepest grudge of the last lineage (or its last killer).
        l.hunter = if variant == "hunted" { l.grudges.iter().max_by_key(|g| (g.depth, g.heir)).cloned() } else { None };
        l.grudges.clear();
        // Unlocks start over; the classes keep their doors and their mastery cards, and the
        // short list keeps its tactic cards.
        let keep: BTreeSet<String> = l
            .unlocks
            .iter()
            .filter(|u| {
                Class::ALL.iter().any(|c| c.unlock() == Some(u.as_str()))
                    || crate::meta::MASTERY_CARDS.contains(&u.as_str())
                    || (variant == "short_list" && crate::meta::TACTIC_CARDS.contains(&u.as_str()))
                    || (variant == "short_list" && crate::meta::TIER2_CARDS.contains(&u.as_str()))
            })
            .cloned()
            .collect();
        l.unlocks = keep;
        // Cut 8B §3: `tame` is never bought; the kennel's leash is back if nothing was tamed.
        l.unlocks.insert("tame".into());
        l.kennel_leash();
        if variant == "bones_only" {
            l.vault.clear();
        }
        // Party members come home to the kennel; the player fields them again.
        let mut party = std::mem::take(&mut l.party);
        l.kennel.append(&mut party);
        Ok(())
    }

    /// Cut 3: camp rest after a run (`no_rest` halves it).
    pub fn rest_after(&self, turns: u32, tier: ExitTier) -> u32 {
        let r = crate::tree::rest_scaled(&self.lineage, crate::offline::rest_after(turns, tier));
        if self.lineage.variant_is("no_rest") {
            r / 2
        } else {
            r
        }
    }

    pub fn run_seed(&self, run_id: u32) -> u64 {
        crate::rng::splitmix(self.lineage.seed ^ (run_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    /// Start (or resume) an expedition. The player chose to go: any camp rest left is skipped.
    pub fn try_send(&mut self) -> Result<Snapshot, String> {
        if !self.lineage.town.home.unwrap_or(true) { return Err("build a house first".into()); }
        Ok(self.send())
    }

    pub fn send(&mut self) -> Snapshot {
        assert!(self.lineage.town.home.unwrap_or(true) || self.sim, "build a house first");
        self.bounty_seen = self.lineage.bounty;
        // Cut 30.5 (the owner: manual send first): before the scout a send by hand is one run — counted
        // toward him when it sends a hero who was home
        if !crate::tree::auto_send(&self.lineage) && !self.lineage.tree.sent && self.run.as_ref().is_none_or(|r| r.turn == 0) {
            crate::tree::did(&mut self.lineage, "send");
            self.lineage.tree.sent = true;
        }
        // Cut 30: a watched send is a check-in of its own — one system may open by its report
        self.lineage.reveal_left = 1;
        self.lineage.in_absence = false;
        self.lineage.rest_left = 0;
        self.watched = true;
        // Cut 27 §1: a run begun here is priced as the camp priced it — its passage and the floors
        // the watch folds come from the camp's panels for the lineage as it stands (cached from the
        // forecast the camp painted).
        // (a run the camp's rest clock already began — `step` while resting — is still at its
        // first tick: it folds too)
        let rules = self.lineage.rules().clone();
        if self.run.is_none() {
            self.passage = crate::forecast::sim_passage(self, &rules);
        }
        self.fold_plan = if self.run.as_ref().is_none_or(|r| r.turn == 0 && r.over.is_none()) { crate::forecast::fold_plan(self, &rules) } else { None };
        self.ensure_run()
    }

    /// Cut 13 §2: pick the heir's trait from the camp's offer (`Lineage.trait_offer`).
    pub fn set_trait(&mut self, name: &str) -> Result<(), String> {
        self.lineage.set_trait(name)
    }

    /// Cut 21 §1: the floor the next sends start on — 1, or a lit waystone (refused otherwise).
    pub fn set_start(&mut self, depth: u32) -> Result<(), String> {
        self.lineage.set_start(depth)?;
        self.lineage.tree.start_by_hand = Some((depth.max(1), self.lineage.best_depth));
        crate::tree::did(&mut self.lineage, "start");
        Ok(())
    }

    /// Cut 30.5: a run may begin — the scout sends him, or a send by hand is under way.
    pub fn may_go(&self) -> bool {
        self.lineage.town.home.unwrap_or(true) && (crate::tree::auto_send(&self.lineage) || self.lineage.tree.sent)
    }

    /// Cut 30.5: the hero is home and waits for a send (before the scout, no run under way).
    pub fn waits(&self) -> bool {
        !self.may_go() && self.run.as_ref().is_none_or(|r| r.turn == 0 || r.over.is_some())
    }

    /// A live run for the snapshot without touching the rest clock (reports, replays).
    pub fn ensure_run(&mut self) -> Snapshot {
        self.auto_keep();
        if self.run.is_none() {
            self.start_run(None);
        }
        let s = self.snapshot();
        self.last_snapshot = Some(s.clone());
        s
    }

    /// Consume up to `ticks` of camp rest; returns how many were used.
    pub fn rest_tick(&mut self, ticks: u32) -> u32 {
        let used = ticks.min(self.lineage.rest_left);
        self.lineage.rest_left -= used;
        self.lineage.total_turns += used as u64;
        used
    }

    pub fn start_run(&mut self, seed_override: Option<u64>) {
        assert!(self.lineage.town.home.unwrap_or(true) || self.sim, "build a house first");
        let ev0 = self.events.len();
        self.auto_keep();
        // Cut 30.5: the workers' standing orders, between real runs
        let previous_start = self.lineage.start.max(1);
        crate::tree::at_send(self);
        // The guide may pick a different stone during an absence. Its send needs that
        // stone's passage on the camp now, rather than the tuple priced before the workers.
        // Unchanged starts keep their quote; sims never run workers or recursively price it.
        if !self.sim && self.lineage.start.max(1) != previous_start {
            let rules = self.lineage.rules().clone();
            self.passage = crate::forecast::sim_passage(self, &rules);
        }
        // Cut 28 §2: the camp this send left (a sim's send is no send).
        if !self.sim {
            self.sent_state = Some(Box::new(SentState::of(self)));
        }
        self.prov.clear();
        // Cut 13 §2: the send settles the heir's trait; the offer is spent.
        self.lineage.trait_offer.clear();
        crate::traits::on_send(&mut self.lineage);
        let id = self.lineage.next_run_id;
        self.lineage.next_run_id += 1;
        let seed = seed_override.unwrap_or_else(|| self.run_seed(id));
        // Cut 21 §1: the send starts on the lineage's chosen floor — a lit waystone pays its
        // toll here (`waystone D9 −$90`); a toll the purse cannot pay (or a start no longer
        // lit) starts on D1 and says so.
        let (start, start_note, toll) = self.pay_start();
        // Cut 27 §5: the set this send plays, remembered while it is one of the last few sent.
        if !self.sim {
            let played = crate::forecast::rules_key(self.lineage.rules());
            if self.lineage.sent_sets.last().is_none_or(|s| crate::forecast::rules_key(s) != played) {
                let set = self.lineage.rules().clone();
                self.lineage.sent_sets.push(set);
                while self.lineage.sent_sets.len() > SENT_SETS {
                    self.lineage.sent_sets.remove(0);
                }
            }
        }
        let wanted = self.lineage.start.max(1);
        let mut rng = Rng::new(seed);
        let route = self.lineage.rules().route();
        let floor = generate(&mut rng, route.biome(start), start);
        let mut hero = Hero::new(self.lineage.class, floor.stairs_up);
        hero.apply_level(self.lineage.class_level());
        crate::legacy::apply(&self.lineage, &mut hero);
        hero.specialization=crate::specialization::current(&self.lineage);
        // Starting arms by class, at the forge's steps (Cut 23 §1; the kit's ids are never loot).
        crate::kit::equip(&self.lineage, &mut hero);
        let mut brought = Vec::new();
        let mut loadout = std::mem::take(&mut self.loadout);
        loadout.sort();
        loadout.dedup();
        // Automations (Cut 2 §3): auto_insure covers the brought items when gold allows.
        // Cut 19 §3: the loadout repeats — the last expedition's supplies are re-packed.
        // Cut 29 §4: insuring is a standing order (on by default), when the purse covers it.
        if self.lineage.orders.insure {
            for id in loadout.clone() {
                let gold = self.lineage.gold;
                let kind = self.lineage.vault.iter().find(|v| v.id == id).map(|v| crate::item::kind_name(&v.kind));
                if self.insure(id).is_ok() {
                    // Cut 13 §3: the night's ledger counts the automation's premiums.
                    let e = self.batch.spent.entry(format!("insure {}", kind.unwrap_or_default())).or_insert((0, 0));
                    e.0 += 1;
                    e.1 += gold - self.lineage.gold;
                }
            }
        }
        self.lineage.repeat_short.clear();
        // QA on a946e04 (qaT: an absence's tolls starved the repeat — `restock ≤ income` — and
        // from the next send on the repeat had forgotten heal and caustic: the found heals were
        // salvaged `heal ×2 · $4`, the badge gone): the repeat's kinds are remembered through a
        // send that could not re-pack them (capped, short) or that packed a found one in their
        // place — only the player's drop (`drop_supply`, `clear_supplies`), a wasted kind or a
        // kind no row uses leaves the list.
        // (a send with the repeat off is the player's pack alone: its kinds are the next's)
        // (Cut 29 §4: the kinds a player's `throw` row names are offered on the camp's repeat tile —
        // `Lineage.repeat_added`, one tap buys it and the repeat keeps it — never added unasked: added at
        // the send they moved the oath gate's raterAT sets to 1 row from bank-optimal)
        let plan: Vec<(String, bool)> = if self.lineage.restock_off {
            Vec::new()
        } else {
            let used = self.lineage.row_kinds();
            self.lineage.last_supplies.iter().enumerate().filter(|(_, k)| !self.lineage.last_wasted.contains(k) && used.contains(*k)).map(|(i, k)| (k.clone(), self.lineage.last_supply_origins.get(i).is_some_and(|(kind, auto)| kind == k && *auto))).collect()
        };
        let plan = current_repeat_plan(&self.lineage, plan);
        self.restock();
        // (a found supply packs free and is not the repeat's — Cut 21 §2)
        let mut origins: Vec<(String, bool)> = self.lineage.supplies.iter().filter(|i| !i.free && !i.found).map(|i| (i.kind.clone(), i.auto_packed)).collect();
        let mut kinds: Vec<String> = origins.iter().map(|(k, _)| k.clone()).collect();
        let assignments = supplied_slot_assignments(&plan, &origins);
        let shelf_indices: Vec<usize> = self.lineage.supplies.iter().enumerate().filter(|(_, i)| !i.free && !i.found).map(|(i, _)| i).collect();
        // A current package refill may satisfy an older authored slot. Its ownership stays
        // authored, so a later package change cannot prune that repeated purchase.
        for ((_, automatic), assigned) in plan.iter().zip(&assignments) {
            if !automatic {
                if let Some(i) = assigned {
                    origins[*i].1 = false;
                    self.lineage.supplies[shelf_indices[*i]].auto_packed = false;
                }
            }
        }
        for ((k, auto), assigned) in plan.into_iter().zip(assignments) {
            let present = assigned.is_some();
            if !present { origins.push((k.clone(), auto)); kinds.push(k); }
        }
        // QA on 778fa1b: the price the camp's badge showed is the price this send's exit re-packs at.
        // QA on 0c6e126 (qaY: `repeat on · ≤$20`, later `≤$24`, then `repeat −$26` overnight as the best floor climbed): a kind's
        // repeat price stays put once quoted — the lower of its old quote and the shelf's today — while the kind stays in the repeat.
        let cat = self.supply_catalogue();
        let was = std::mem::take(&mut self.lineage.repeat_quote);
        self.lineage.repeat_quote = kinds.iter().filter_map(|k| cat.iter().find(|e| e.kind == *k).map(|e| (k.clone(), was.get(k).map_or(e.price, |q| (*q).min(e.price))))).collect();
        self.lineage.last_supplies = kinds;
        self.lineage.last_supply_origins = if origins.iter().any(|(_, auto)| *auto) { origins } else { Vec::new() };
        for id in loadout {
            if let Some(i) = self.lineage.vault.iter().position(|v| v.id == id) {
                let it = self.lineage.vault.remove(i);
                brought.push(it.id);
                if hero.class==Class::Gunner&&it.cat()==Cat::Weapon {
                    if let Some(old)=hero.weapon.replace(it) {hero.inv.push(old);}
                }else {hero.auto_equip(it);}
            }
        }
        // A fresh send starts guns loaded; live saves and ordinary swaps retain
        // their own chamber state. Kept equipment may come from an earlier run.
        if hero.class==Class::Gunner {
            for item in hero.inv.iter_mut().chain(hero.weapon.iter_mut()) {
                if let Some(p)=crate::firearm::Profile::of(&item.kind) {item.firearm=Some(crate::firearm::Chambers::loaded(p));}
            }
        }
        let mut run = Run {
            gun_reload: None,
            gun_skills: (hero.class==Class::Gunner).then(Default::default),
            id,
            heir: self.lineage.heir,
            started_turn: self.lineage.total_turns,
            difficulty: self.lineage.endgame.as_ref().map_or(0, |p| p.tier),
            rng,
            depth: start,
            start,
            toll,
            passage: 0,
            start_short: (start != wanted).then_some(wanted),
            floor,
            hero,
            trait_: self.lineage.trait_,
            worn: crate::traits::Worn::default(),
            gift: crate::traits::GiftRun::default(),
            monsters: Vec::new(),
            items: Vec::new().into(),
            overlays: Vec::new(),
            turn: 0,
            floor_turn: 0,
            alert: 0,
            loot: 0,
            next_id: HERO_ID,
            next_item_id: 10,
            kills_floor: 0,
            kills: Vec::new().into(),
            summoned_kills: 0,
            brought: brought.into(),
            trace: Vec::new(),
            notes: Vec::new().into(),
            over: None,
            death_cause: None,
            death_modifiers: None,
            death_blow: 0,
            death_t: None,
            death_short: 0,
            hurt_since_action: false,
            hurt_last: false,
            kill_since_action: false,
            kill_last: false,
            seen_ids: BTreeSet::new().into(),
            new_seen: false,
            telegraphs_now: Vec::new(),
            low10_t: None,
            low20_t: None,
            near_deaths: Vec::new(),
            gambles: Vec::new(),
            floor_streams: false,
            route,
            fork_seen: 0,
            gambles_survived: Vec::new(),
            gamble_harm: 0,
            own_throw: None,
            own_throw_harm: 0,
            found_units: Vec::new().into(),
            found_gone: Vec::new().into(),
            stolen: Vec::new(),
            stolen_ids: Vec::new(),
            recovered: Vec::new(),
            stolen_labels: Vec::new(),
            stolen_kinds: Vec::new(),
            ally_lost: Vec::new(),
            ally_freed: Vec::new(),
            boss_kills: Vec::new().into(),
            drinks: 0,
            rested: false,
            oath: crate::oath::sworn(&self.lineage).cloned(),
            oath_said: None,
            home_return: false,
            wall_seen: false,
            route2: self.lineage.unlocks.contains("route2"),
            tamed_grudges: Vec::new(),
            meters: Default::default(),
            burned: Vec::new(),
            caged: Vec::new().into(),
            depth_t: Vec::new().into(),
            max_steps: Vec::new().into(),
            drank_heal: false,
            melee_used: false,
            boss_seen_t: None,
            bosses_met: Vec::new().into(),
            scars: crate::descent::BOSS_DEPTHS.iter().map(|(k, _)| (k.to_string(), self.lineage.pkg.scar(k, &self.lineage.kills))).filter(|(_, p)| *p > 0).collect(),
            hurt_since_boss: false,
            row_fired: vec![0; ROWS_TOTAL],
            renderable_events: 0,
            ended: false,
            max_depth: start,
            secured: 0,
            record0: self.lineage.best_depth,
            record_mark: self.lineage.best_depth,
            best_at_send: Some(self.lineage.best_depth),
            trophies_run: Vec::new().into(),
            companions: Vec::new(),
            recalled: Vec::new(),
            tamed: Vec::new(),
            lost_companions: Vec::new(),
            fell_why: Vec::new(),
            supplies: Vec::new(),
            taunt_t: 0,
            kited: None,
            pack_go: 0,
            card_fell: None,
            freeing: None,
            hero_dist: Vec::new(),
            hero_dist_pos: None,
            hero_flood: Vec::new(),
            hero_flood_head: 0,
            last_visible: vec![u32::MAX],
            idle_actions: 0,
            cowardly_streak: 0,
            hold_dist: -1,
            hold_streak: 0,
            actions: 0,
            ignored: BTreeMap::new(),
            chase: None,
            recent_pos: Vec::new(),
            last_damage_action: 0,
            stuck_fires: 0,
            stuck_until: 0,
            stuck_first_t: None,
            stuck_cause: None,
            stuck_row: None,
            card_loops: 0,
            loop_causes: Vec::new(),
            homeward: None,
            home_at: None,
            homeward_bank: false,
            repeat_short: self.lineage.repeat_short.clone(),
            trait_last: None,
            trait_floor: 0,
            wasted_kinds: Vec::new(),
            pickup_streak: 0,
            pickup_inv: 0,
            items_until: 0,
            skip_items: Vec::new(),
            pickup_dry: 0,
            drain_on: None,
            blows: Vec::new(),
            hp_lost: Vec::new().into(),
            hp_lost_from: 0,
            known_foes: BTreeMap::new(),
            loot_raw: 0,
            low_hp: i32::MAX,
            saved_by: None,
            saved_low: 0,
            last_target: None,
            hunt: None,
            blocked_now: None,
            blocked_last: None,
            row_streak: (-9, 0),
            row_suppressed: (-9, 0),
            retreats: (0, 0),
            bloodless: (0, 0, 0),
            nohp: (0, 0, 0),
            boss_still: None,
            driven_off: None,
            driven_lost: false,
            rows_rested: (Vec::new(), 0),
            rests: 0,
            aimed: false,
            bones: self.lineage.bones.clone().into(),
            bones_found: Vec::new(),
            kills_counted: 0,
            timed_out: false,
            exit_row: None,
            noise: None,
            verb_ring: Vec::new(),
            last_hit_verb: None,
            blind_seen: Vec::new(),
            bow_swap: None,
            swapped: 0,
            swap_left: Vec::new(),
            mirrors: Vec::new(),
            arc: Arc::default(),
            episodes: Vec::new().into(),
            voice_t: None,
            fight_t: None,
            situations: Vec::new().into(),
            vault_cage: Vec::new(),
            vault_choice: None,
            prayed: false,
            lent_row: None,
            stray_placed: false,
            // The same untamed stray waits at the entrance when a waystone skips its original floor.
            first_stray: self.lineage.first_stray().map(|(d, n)| (d.max(start), n)).filter(|(_, n)| !self.lineage.named_resting(n, id)),
            strays_tamed: Vec::new(),
            bail: false,
            manual: false,
            manual_act: None,
            awaiting: false,
            dens: Vec::new(),
            tempted: false,
            rows_why: Vec::new(),
            ending_t: None,
            den_stolen: Vec::new(),
            den_thin: (self.lineage.den_thefts > 0 && self.lineage.facts.contains("den")) || self.lineage.den_wakes > 0,
            den_snatches: 0,
            den_wakes: 0,
            den_bolted: false,
            bounty: self.lineage.bounty,
            bounty_gold: 0,
            avenged: Vec::new().into(),
            named_rest: self.lineage.named_met.keys().filter(|n| self.lineage.named_resting(n, id)).cloned().collect(),
            named_placed: Vec::new(),
            event_recent: self.lineage.event_recent.iter().map(|(k, v)| (k.clone(), v.iter().filter(|(r, _)| r + 2 >= id).map(|(_, i)| *i).collect::<Vec<u8>>())).filter(|(_, v)| !v.is_empty()).collect(),
            event_used: Vec::new().into(),
            learned: Vec::new(),
            gas_dmg_floor: 0,
            lock_last_pop: 0,
            lit: false,
            passed: Vec::new().into(),
            lock_tiles: Vec::new(),
            raiding: false,
            acting_row: -1,
            floor_twist: None.into(),
            last_twist: None.into(),
            seed,
            next_twist: None,
            sleepers: Vec::new(),
            thin: self.lineage.thin_map().into(),
            packed: Vec::new().into(),
            bought_leashes: Vec::new(),
        };
        // Cut 30 §1: the heir's traits go on the run (`frail` takes its max hp here).
        crate::traits::wear(&mut run, &self.lineage);
        for s in std::mem::take(&mut self.lineage.supplies) {
            let mut it = s;
            it.id = run.new_item_id() + 5000;
            run.supplies.push(it.id);
            run.packed.push(it.kind.clone());
            if it.kind == "leash" {
                if !it.free {
                    run.bought_leashes.push(it.paid);
                }
                match run.hero.inv.iter_mut().find(|i| i.kind == "leash") {
                    Some(l) => l.amount += 1,
                    None => run.hero.inv.push(it),
                }
            } else {
                run.hero.auto_equip(it);
            }
        }
        populate_floor(&mut run, &self.lineage.grudges, &self.lineage.forge, self.lineage.hunter.as_ref());
        place_situations(&mut run, &self.lineage.lost);
        place_bones(&mut run);
        spawn_party(&mut run, &self.lineage.party);
        // QA on e75ec29 (qaR: the pet is never in the text): each companion that walks down
        // with the heir is named at the start (`Skog joins.`).
        for m in run.monsters.iter().filter(|m| m.ally && m.cid.is_some()) {
            let text = format!("{} joins.", m.name.clone().unwrap_or_default());
            self.events.push(Ev::Note { t: run.turn, text: text.clone() });
            run.notes.push((run.turn, text));
        }
        let vision = run.vision(&self.lineage.unlocks);
        run.floor.map.update_vision(run.hero.pos, vision);
        self.history.clear();
        self.row_tally.clear();
        self.facts_at_run_start = self.lineage.facts.len();
        // Cut 27 §1: a waystone start the set would have reached at ≥ 95 % a floor is paid the
        // skipped floors' gold at the send, into the purse (`passage D9 +$84`): the floors above it
        // would have come home with the send nineteen times in twenty.
        let paid = self.passage.filter(|(at, n)| *at == run.depth && run.depth > 1 && *n > 0).map(|p| p.1);
        run.passage = paid.unwrap_or(0);
        self.run = Some(run);
        if let Some(coins) = paid {
            let d = self.run.as_ref().map_or(1, |r| r.depth);
            self.lineage.gold_move(coins, &format!("passage D{d}"));
            self.batch.passage += coins;
            self.events.push(Ev::Callout { t: 0, text: format!("passage +${coins}"), why: None });
        }
        let mut cx = self.ctx();
        let run = cx.0;
        crate::facts::on_vision(run, &mut cx.1);
        if let Some(text) = start_note {
            crate::chronicle::note(run, &mut cx.1, text);
        }
        let (d, biome) = (run.depth, run.biome());
        crate::chronicle::note(run, &mut cx.1, format!("Heir {} enters D{d}, {}.", run.heir, biome.title()));
        if d > 1 {
            crate::facts::learn(run, &mut cx.1, format!("biome:{}", biome.name()));
        }
        // RUNS_UI: the send kept to replay (real sends; not a harness's literal lineage, nor a
        // thread or process that keeps no history / no capsules)
        if !self.sim {
            let id = self.run.as_ref().map_or(0, |r| r.id);
            let pre: Vec<Ev> = self.events[ev0.min(self.events.len())..].to_vec();
            if let Some(tap) = self.tap.as_mut() {
                tap.insert(id, pre.clone());
            }
            if !self.lineage.pkg.literal && !NO_HISTORY.with(|c| c.get()) && !CAPSULES_OFF.load(std::sync::atomic::Ordering::Relaxed) {
                let mut g = self.sim_clone();
                g.sim = false;
                g.watched = self.watched;
                g.offline = self.offline;
                g.prov = self.prov.clone();
                g.row_tally = self.row_tally.clone();
                g.facts_at_run_start = self.facts_at_run_start;
                self.capsules.0.push_back(Capsule { id, game: Box::new(g), pre, inputs: Vec::new() });
                while self.capsules.0.len() > CAPSULES {
                    self.capsules.0.pop_front();
                }
            }
        }
    }

    /// Cut 21 §1: the floor this send starts on, the toll paid for it (a gold line
    /// `waystone D9`, counted with the absence's spending), and the note when the chosen
    /// start could not be taken — `(1, Some(..))` when the purse is short of the toll or the
    /// waystone is not lit.
    ///
    /// QA on a946e04 (qaT: `waystone D5 ×16 · −$800` in one absence): the toll buys the
    /// night's pass (`LineageState::night_passes`) — the night's later sends from that
    /// waystone pay nothing. An absence whose purse cannot pay the pass sends the rest of the
    /// night's runs from D1 without asking again (`night_short`; the report's `start_short`).
    /// Returns (start, note, toll paid).
    fn pay_start(&mut self) -> (u32, Option<String>, i32) {
        let want = self.lineage.start.max(1);
        if want == 1 {
            return (1, None, 0);
        }
        if !self.lineage.stone_lit(want, self.lineage.rules().route()) {
            return (1, Some(format!("D{want} unlit: from D1.")), 0);
        }
        if self.lineage.night_passes.contains(&want) {
            return (want, None, 0);
        }
        let toll = crate::tree::toll_scaled(&self.lineage, LineageState::start_toll(want));
        let short = toll > 0 && (self.lineage.gold < toll || (self.offline && self.lineage.night_short == Some(want)));
        if short {
            if self.offline {
                self.lineage.night_short = Some(want);
            }
            self.batch.start_short.get_or_insert((want, toll, 0)).2 += 1;
            return (1, Some(format!("Toll ${toll} short: from D1.")), 0);
        }
        if toll > 0 {
            let why = format!("waystone D{want}");
            self.lineage.gold_move(-toll, &why);
            let e = self.batch.spent.entry(why).or_insert((0, 0));
            e.0 += 1;
            e.1 += toll;
        }
        self.lineage.night_passes.insert(want);
        (want, None, toll)
    }

    /// Borrow the run and a context together.
    pub fn ctx(&mut self) -> (&mut Run, Ctx<'_>) {
        let Game { run, lineage, events, sim, prov, row_tally, .. } = self;
        let run = run.as_mut().expect("no live run");
        let set = lineage.active_set.min(lineage.sets.len() - 1);
        let cx = Ctx {
            vault_pref: &lineage.vault_pref,
            lost: &lineage.lost,
            sets: &lineage.sets,
            active_set: set,
            trophies: &lineage.trophies,
            facts: &mut lineage.facts,
            kill_counts: &mut lineage.kill_counts,
            flavours: &lineage.flavours,
            rules: &lineage.sets[set],
            unlocks: &lineage.unlocks,
            grudges: &lineage.grudges,
            forge: &lineage.forge,
            max_rows: std::cell::Cell::new(None),
            events,
            sim: *sim,
            prov,
            variant: &lineage.variant,
            hunter: lineage.hunter.as_ref(),
            tally: row_tally,
        };
        (run, cx)
    }

    /// Advance the live view by `turns`, returning the events and the new snapshot.
    pub fn step(&mut self, turns: u32) -> StepResult {
        let mut events = Vec::new();
        let mut run_over = false;
        self.watched = true;
        self.lineage.in_absence = false;
        // Cut 30.5: before the scout the hero home waits for a send (no run begins by itself)
        if !self.may_go() && self.run.as_ref().is_none_or(|r| r.turn == 0) {
            let snapshot = match self.last_snapshot.clone() {
                Some(s) => s,
                None => self.ensure_run(),
            };
            let exit_pending = self.exit_pending_wire();
            return StepResult { calm: Vec::new(), events, snapshot, run_over: true, exit_pending };
        }
        // Camp rest runs on the same clock online (Cut 2 §1); `send` skips it.
        if self.lineage.rest_left > 0 && self.run.as_ref().is_none_or(|r| r.turn == 0) {
            let used = self.rest_tick(turns);
            if self.lineage.rest_left > 0 || used == turns {
                let snapshot = self.ensure_run();
                let exit_pending = self.exit_pending_wire();
                return StepResult { calm: Vec::new(), events, snapshot, run_over: false, exit_pending };
            }
        }
        if self.run.is_none() {
            let snapshot = match self.last_snapshot.clone() {
                Some(s) => s,
                None => {
                    self.start_run(None);
                    let s = self.snapshot();
                    self.run = None;
                    s
                }
            };
            let exit_pending = self.exit_pending_wire();
            return StepResult { calm: Vec::new(), events, snapshot, run_over: true, exit_pending };
        }
        // Cut 28 §3: each tick's calm (no decision, no threat) — the step's calm stretches.
        let mut calm: Vec<(u32, bool)> = Vec::new();
        for _ in 0..turns {
            self.tick();
            if let Some(r) = self.run.as_ref() {
                calm.push((r.turn, crate::fold::calm_tick(r, &self.lineage.found_kinds, &self.events)));
            }
            events.append(&mut self.events);
            if self.run.as_ref().is_none_or(|r| r.over.is_some()) {
                run_over = true;
                break;
            }
            // take control: the world waits for the player's action
            if self.run.as_ref().is_some_and(|r| r.awaiting) {
                break;
            }
        }
        let mut out = self.step_result(events, run_over);
        out.calm = crate::fold::calm_spans(&calm);
        out
    }

    /// The end of a `step` (and of a `fold`): the snapshot, and when the run ended the settled
    /// exit (its ledger line and trace on the exit event, the keep sheet pending).
    pub fn step_result(&mut self, mut events: Vec<Ev>, run_over: bool) -> StepResult {
        let snapshot = self.snapshot();
        if run_over {
            self.finish_run();
            events.append(&mut self.events);
            // Cut 6 §1: the exit event carries the settled ledger line.
            if let Some(line) = self.last_exit.take() {
                if let Some(Ev::Exit { line: l, trace, .. }) = events.iter_mut().rev().find(|e| matches!(e, Ev::Exit { .. })) {
                    // Cut 9 §5: the exit event carries the last-5 trace beside the line.
                    *trace = line.trace.clone().map(Box::new);
                    *l = Some(Box::new(ExitLine { trace: None, ..line }));
                }
            }
        }
        self.last_snapshot = Some(snapshot.clone());
        let exit_pending = if run_over { self.exit_pending_wire() } else { None };
        StepResult { events, snapshot, run_over, exit_pending, calm: Vec::new() }
    }

    pub fn exit_pending_wire(&self) -> Option<ExitPending> {
        self.pending_exit.as_ref().map(|p| ExitPending {
            items: p.items.iter().map(|i| to_inv(i, &self.lineage.facts, &self.lineage.flavours)).collect(),
            tier: p.tier.name().into(),
            worth: salvage_coins(&p.items, p.pct, self.lineage.gold_carry),
            auto_keep: auto_keep_plan(p, &self.lineage.vault, self.lineage.vault_slots(), &self.lineage.keep_pref, true).0,
            decide: keep_is_a_decision(p, &self.lineage.vault, self.lineage.vault_slots()) && !crate::tree::on(&self.lineage, "keeper"),
            note: keep_note(p, &self.lineage, &auto_keep_plan(p, &self.lineage.vault, self.lineage.vault_slots(), &self.lineage.keep_pref, true).0),
        })
    }

    /// One turn of the live run. Records history for verdicts when not simulating.
    pub fn tick(&mut self) {
        if self.tap.is_none() {
            return self.tick_inner();
        }
        // RUNS_UI (tests): the tick's events, by run
        let (n0, id) = (self.events.len(), self.run.as_ref().map_or(0, |r| r.id));
        self.tick_inner();
        let evs: Vec<Ev> = self.events[n0.min(self.events.len())..].to_vec();
        if let Some(tap) = self.tap.as_mut() {
            tap.entry(id).or_default().extend(evs);
        }
    }

    fn tick_inner(&mut self) {
        if self.run.as_ref().is_none_or(|r| r.over.is_some()) {
            return;
        }
        // Take control is the watched run's alone: a sim (a forecast, a verdict's replay) or an absence plays the rules
        if (self.sim || self.lineage.in_absence) && self.run.as_ref().is_some_and(|r| r.manual) {
            let r = self.run.as_mut().unwrap();
            r.manual = false;
            r.manual_act = None;
            r.awaiting = false;
        }
        if !self.sim && self.run.as_ref().unwrap().turn.is_multiple_of(HISTORY_STRIDE) && !NO_HISTORY.with(|c| c.get()) {
            let r = self.run.as_ref().unwrap();
            self.history.push_back((r.clone(), self.lineage.facts.clone()));
            while self.history.len() > HISTORY_TURNS + 1 {
                self.history.pop_front();
            }
        }
        if !self.sim {
            let r = self.run.as_ref().unwrap();
            if self.floor_start.as_ref().is_none_or(|(f, _)| f.id != r.id || f.depth != r.depth) {
                self.floor_start = Some((r.clone(), self.lineage.facts.clone()));
            }
        }
        let (run, mut cx) = self.ctx();
        let before = cx.events.len();
        // Cut 29 §3: the hero's and the pets' hp before the tick (the meters' healing).
        let hero_hp0 = run.hero.hp;
        let pets_hp0: Vec<(u32, i32)> = if cx.sim { Vec::new() } else { run.monsters.iter().filter(|m| m.ally && m.hp > 0).map(|m| (m.id, m.hp)).collect() };
        let gun_before = crate::firearm::ObservedGun::of(run);
        crate::turn::tick(run, &mut cx);
        let gun_after = crate::firearm::ObservedGun::of(run);
        if gun_before != gun_after {
            cx.events.push(Ev::Gun { t:run.turn, state:gun_after.map(|g|Box::new(g.snapshot(run.turn))) });
        }
        if !cx.sim {
            crate::meters::heals(run, &mut cx, before, std::iter::once((HERO_ID, hero_hp0)).chain(pets_hp0));
            run.meters.tick(&cx.events[before..], run.monsters.iter().filter(|m| m.ally).map(|m| m.id));
        }
        let n = cx.events[before..].iter().filter(|e| e.renderable()).count() as u32;
        run.renderable_events += n;
        if run.turn >= MAX_TURNS_PER_RUN && run.over.is_none() {
            run.timed_out = true;
            crate::chronicle::note(run, &mut cx, "Lost the thread. Came home empty-handed.".into());
            crate::turn::end_run(run, &mut cx, ExitTier::Return);
        }
        // A run that keeps shuffling on one floor is a policy failure, not a wait: end it as a
        // return with nothing kept so the stall verdict names the row (rater F: minutes of
        // `to corridor` ↔ `pick up` at frozen HP with no trace and no patch). Keeping the
        // return's share was tried (QA on 952e306: four stalls in an hour, $72–$145 forfeited)
        // and let the DEFAULT set profit over 8 h, a gated invariant; the line says `stalled`
        // and counts the unused supplies instead.
        // Cut 24 §1: a floor stalled before a boss met there (his fight could not progress: a
        // corridor of his goblins, a dance before him, his stairs sealed) is his win — driven
        // off, not a stall.
        // Cut 30 §1: on the idle floor (a set of packages — ours, not the player's) a floor the guard has
        // stopped twice is given up: the hero walks home with what he carries (a queued return, `bail`)
        // instead of pacing to the stall — a foe across a gap or a mirror he will not strike is no reason
        // to lose the send
        if run.stuck_fires + 1 >= STALL_FIRES && run.over.is_none() && !run.bail && cx.rules.rows.iter().any(|r| r.is_pkg()) && run.boss_still.is_none() {
            run.bail = true;
        }
        if run.stuck_fires >= STALL_FIRES && run.over.is_none() {
            // (met on this floor — `boss_still` is the floor's — and alive: his stairs are sealed)
            let met = run.boss_still.map(|b| b.0);
            if let Some(bi) = run.monsters.iter().position(|m| m.hp > 0 && m.hostile() && m.is_boss() && (run.floor.map.is_visible(m.pos) || Some(m.id) == met)) {
                crate::turn::driven_off(run, &mut cx, bi);
            }
        }
        if run.stuck_fires >= STALL_FIRES && run.over.is_none() {
            run.timed_out = true;
            // Cut 13 §1: the note names the guard's moment, as the record and the reel do
            // (`Stalled: the archer, no path. Came home empty-handed.`).
            let cause = crate::sifter::stall_note_cause(run.stuck_cause.as_deref().unwrap_or("paced"));
            crate::chronicle::note(run, &mut cx, format!("Stalled: {cause}. Came home empty-handed."));
            crate::turn::end_run(run, &mut cx, ExitTier::Return);
        }
        self.lineage.total_turns += 1;
        if self.run.as_ref().unwrap().depth >= ENDING_DEPTH {
            self.lineage.ended = true;
            if let Some(p) = &mut self.lineage.endgame {
                let _ = p.complete(self.run.as_ref().unwrap().difficulty);
            }
            let v = self.lineage.variant.clone();
            if !v.is_empty() && !self.lineage.ascended.contains(&v) {
                self.lineage.ascended.push(v);
            }
        }
    }

    /// Cut 7: walk the live run straight down to `depth` (fresh floors, the hero untouched),
    /// for probes and tests that start at a band's floor.
    pub fn descend_to(&mut self, depth: u32) {
        while self.run.as_ref().is_some_and(|r| r.over.is_none() && r.depth < depth) {
            let (run, mut cx) = self.ctx();
            run.hero.pos = run.floor.stairs_down;
            crate::turn::descend(run, &mut cx);
        }
        self.events.clear();
    }

    /// Cut 12 §4: `descend_to` with the floor's situation chosen (trials, probes, tests):
    /// the floor above is reached first, then `depth` rolls `kind` (a kind the floor cannot
    /// hold falls through to the band's roll).
    pub fn descend_to_twist(&mut self, depth: u32, kind: &str) {
        self.descend_to(depth.saturating_sub(1));
        if let Some(run) = self.run.as_mut() {
            run.next_twist = Some(kind.into());
        }
        self.descend_to(depth);
    }

    /// An ordinary tick or a span before the next possible effect or action. `settled`
    /// belongs to this uninterrupted loop: its first tick observes the state normally.
    /// Single-step/replay callers continue using `tick`; taps must observe each tick.
    pub(crate) fn tick_batch(&mut self, limit: u32, settled: &mut bool) -> u32 {
        if limit == 0 || self.run.as_ref().is_none_or(|r| r.over.is_some()) {
            return 0;
        }
        if !*settled || self.tap.is_some() {
            self.tick();
            *settled = true;
            return 1;
        }
        let run = self.run.as_mut().unwrap();
        if !self.sim && self.floor_start.as_ref().is_none_or(|(floor, _)| floor.id != run.id || floor.depth != run.depth) {
            self.tick();
            return 1;
        }
        if run.stuck_fires >= STALL_FIRES {
            self.tick();
            return 1;
        }
        let mut quiet = limit
            .min(TICKS_PER_TURN - 1 - run.turn % TICKS_PER_TURN)
            .min(TICKS_PER_TURN - 1 - run.floor_turn % TICKS_PER_TURN)
            .min(MAX_TURNS_PER_RUN.saturating_sub(run.turn + 1));
        if !self.sim && !NO_HISTORY.with(|c| c.get()) {
            let rem = run.turn % HISTORY_STRIDE;
            quiet = quiet.min(if rem == 0 { 0 } else { HISTORY_STRIDE - rem });
        }
        if let Some((t0, _)) = run.vault_choice.as_ref() {
            quiet = quiet.min((t0 + VAULT_GRACE).saturating_sub(run.turn + 1));
        }
        if let Some(reload) = run.gun_reload {
            quiet = quiet.min(reload.at.saturating_sub(run.turn.saturating_add(1)));
        }
        if quiet == 0 { self.tick(); return 1; }
        let before_action = |bound: u32, energy: i32, speed: i32| -> u32 {
            if speed <= 0 || energy >= ACT_ENERGY - speed { return 0; }
            let distance = ACT_ENERGY.saturating_sub(energy).saturating_sub(1);
            if distance <= speed.saturating_mul(bound as i32) { bound.min((distance / speed) as u32) } else { bound }
        };
        let hero_speed = run.hero.speed();
        quiet = before_action(quiet, run.hero.energy, hero_speed);
        if run.hero.speed_t > 0 { quiet = quiet.min(run.hero.speed_t as u32); }
        if run.hero.special_cd>0 {quiet=quiet.min(run.hero.special_cd as u32);}
        if run.hero.riposte_t>0 {quiet=quiet.min(run.hero.riposte_t as u32);}
        if quiet == 0 { self.tick(); return 1; }
        for m in &run.monsters {
            quiet = before_action(quiet, m.energy, m.effective_speed());
            if m.slow_t > 0 { quiet = quiet.min(m.slow_t as u32); }
            if m.hex_t>0 {quiet=quiet.min(m.hex_t as u32);}
            if let Some(ttl) = m.ttl { quiet = quiet.min(ttl.saturating_sub(1).max(0) as u32); }
            if quiet == 0 { break; }
        }
        if quiet == 0 { self.tick(); return 1; }
        let ticks = quiet as i32;
        run.turn += quiet;
        run.floor_turn += quiet;
        run.hero.energy += hero_speed * ticks;
        if run.hero.poison.1 > 0 { run.hero.poison.1 = (run.hero.poison.1 - ticks).max(0); }
        run.hero.tick_statuses_by(ticks);
        if run.taunt_t > 0 { run.taunt_t = (run.taunt_t - ticks).max(0); }
        for m in &mut run.monsters {
            m.energy += m.effective_speed() * ticks;
            if m.poison.1 > 0 { m.poison.1 = (m.poison.1 - ticks).max(0); }
            m.tick_statuses_by(ticks);
        }
        if !self.sim { run.meters.quiet_ticks(quiet); }
        self.lineage.total_turns += u64::from(quiet);
        quiet
    }

    /// Run the live expedition to its end (used by forecasts and the offline batch).
    pub fn run_to_end(&mut self, max_turns: u32) {
        let mut n = 0;
        let mut settled = false;
        while self.run.as_ref().is_some_and(|r| r.over.is_none()) && n < max_turns {
            n += self.tick_batch(max_turns - n, &mut settled);
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let l = &self.lineage;
        let run = self.run.as_ref().expect("no live run");
        let m = &run.floor.map;
        let h = &run.hero;
        let hero = HeroSnap {
            gun: crate::firearm::ObservedGun::of(run).map(|g|g.snapshot(run.turn)),
            entity: Entity {
                id: HERO_ID,
                kind: format!("hero_{}", h.class.name()),
                name: Some(crate::legacy::hero_identity(l.seed, run.heir, l.bloodline_id)),
                x: h.pos.x,
                y: h.pos.y,
                hp: h.hp,
                max_hp: h.max_hp,
                tags: h.status_tags(),
                ally: None,
                telegraph: None,
                cid: None,
                remembered: false,
                modifiers: None,
            },
            inv: h.inv.iter().map(|i| to_inv(i, &l.facts, &l.flavours)).collect(),
            weapon: h.weapon.as_ref().map(|w| w.kind.clone()),
            armour: h.armour.as_ref().map(|a| a.kind.clone()),
            class: h.class.name().into(),
            specialization:h.specialization,
            trait_: run.trait_.name().into(),
        };
        let mut entities: Vec<Entity> = run
            .monsters
            .iter()
            .filter(|mo| mo.hp > 0 && (m.is_visible(mo.pos) || mo.ally))
            .map(|mo| monster_entity(mo, &l.facts))
            .collect();
        // Cut 4: pursued-but-unseen hostiles, at the tile they were last seen on.
        for (i, p) in run.remembered_foes() {
            let mut e = monster_entity(&run.monsters[i], &l.facts);
            e.x = p.x;
            e.y = p.y;
            e.remembered = true;
            entities.push(e);
        }
        let items = run
            .items
            .iter()
            .filter(|fi| m.is_seen(fi.pos))
            .map(|fi| {
                let (known, kind, label) = describe(&fi.item, &l.facts, &l.flavours);
                FloorItemWire { id: fi.item.id, x: fi.pos.x, y: fi.pos.y, kind, known, label }
            })
            .collect();
        let mut brought: Vec<StakeItem> = Vec::new();
        for id in &run.brought {
            let it = h.inv.iter().chain(h.weapon.iter()).chain(h.armour.iter()).find(|i| i.id == *id);
            if let Some(it) = it {
                let (_, _, label) = describe(it, &l.facts, &l.flavours);
                brought.push(StakeItem { label, insured: l.insured.contains(id) });
            }
        }
        let return_row = l.rules().active(l.max_rows()).find(|(_, r)| matches!(r.verb.v.as_str(), "return" | "bank")).map(|(i, _)| i);
        // Cut 6 §1: what that row would bring home now (the kept number, not the carried one).
        let kept = return_row.map(|i| {
            let tier = if l.rules().rows[i].verb.v == "bank" { ExitTier::Bank } else { ExitTier::Return };
            run.secured + run.loot.max(0) * tier.pct() / 100
        });
        Snapshot {
            depth: run.depth,
            difficulty: run.difficulty,
            modifier_catalogue: crate::endgame::catalogue(run.difficulty),
            biome: run.biome().name().into(),
            w: m.w,
            h: m.h,
            tiles: m.tiles.clone(),
            seen: m.seen.clone(),
            visible: m.visible.clone(),
            overlays: run.overlays.clone(),
            hero,
            entities,
            items,
            alert: run.alert,
            turn: run.turn,
            // (Cut 30.5: the gold carried — the checkpoints' secured gold with the carry since; the stake reads the carry at risk)
            loot: run.carried(),
            run: RunRef { id: run.id, heir: run.heir, started_turn: run.started_turn, start: run.start.max(1), passage: run.passage },
            stake: Stake { loot: run.loot, brought, return_row, kept, stalling: run.stuck_fires > 0, returning: run.homeward.is_some(), death_keep: run.kept(ExitTier::Death), swapped: run.swapped, swap_left: run.swap_left.last().cloned() },
            vision: run.vision(&l.unlocks),
            manual: run.manual,
            awaiting: run.awaiting,
            vault_choice: run.vault_choice.as_ref().map(|(t0, items)| VaultChoice {
                items: items.iter().map(|i| to_inv(i, &l.facts, &l.flavours)).collect(),
                left: (t0 + VAULT_GRACE).saturating_sub(run.turn),
                pick: items.get(crate::turn::vault_pick(items, &l.vault_pref)).map(|i| i.id),
            }),
            room: Some(run.room_ref()),
            rooms: Some(run.floor.rooms.len() as u32),
            meters: (!run.meters.is_empty()).then(|| crate::wire::SnapMeters { run: crate::meters::wire(&run.meters.run), fight: (run.meters.fight.ticks > 0).then(|| crate::meters::wire(&run.meters.fight)), fighting: run.meters.quiet < crate::meters::FIGHT_GAP && run.meters.run.fights > 0 }),
            floor_twist: run.floor_twist.as_deref().map(|t| crate::situations::twist_word(t).to_string()),
            fork: run.fork_snap(),
        }
    }

    /// Cut 24 §2: what was new in `run` (`ExitLine.news`), read against the lineage as it
    /// stood before the run: ≤ 3 lines, most telling first; none new → the one thing that
    /// differed from the last run.
    pub fn run_news(&self, run: &Run, tier: ExitTier, _pct: i32) -> Vec<crate::wire::News> {
        let l = &self.lineage;
        let mut out: Vec<crate::wire::News> = Vec::new();
        let push = |out: &mut Vec<crate::wire::News>, k: &str, text: String| out.push(crate::wire::News { k: k.into(), text });
        for e in &run.episodes {
            if let crate::sifter::Resolution::FirstBoss { kind } = &e.resolution {
                push(&mut out, "first", format!("first: {} slain", crate::sifter::boss_short(kind)));
            }
        }
        if run.max_depth > l.best_depth && l.best_depth > 0 {
            push(&mut out, "record", format!("record: D{}", run.max_depth));
        }
        for name in &run.avenged {
            push(&mut out, "named", format!("avenged {name}"));
        }
        if let Some(kind) = &run.driven_off {
            push(&mut out, "driven", format!("driven off: {}", crate::sifter::boss_short(kind)));
        }
        let mut finds: Vec<String> = Vec::new();
        for (_, k) in &run.found_units {
            if k != "gold" && !l.found_kinds.contains(k) && !l.owned_kind(k) && !run.packed.contains(k) && !finds.contains(k) {
                finds.push(k.clone());
            }
        }
        // (a lineage from before Cut 24 has found things it never recorded: its first run says none)
        if !l.found_kinds.is_empty() || l.next_run_id <= 2 {
            for k in finds.iter().take(2) {
                push(&mut out, "find", format!("new find: {}", l.wire_name(k).replace('_', " ")));
            }
        }
        for what in &run.learned {
            if crate::situations::TWISTS.contains(&what.as_str()) {
                push(&mut out, "situation", format!("first: the {}", crate::situations::twist_word(what)));
            }
        }
        let learned = run.learned.len();
        if out.is_empty() && learned > 0 {
            push(&mut out, "learned", format!("learned {learned}"));
        }
        out.dedup();
        out.truncate(3);
        if !out.is_empty() {
            return out;
        }
        // Nothing new: the one thing that differed from the last run.
        let kept = run.kept(tier);
        let Some(last) = &l.last_run else { return vec![crate::wire::News { k: "differ".into(), text: format!("D{} · {}", run.max_depth, tier.name()) }] };
        let twists: Vec<String> = run.situations.iter().map(|(_, s)| s.clone()).filter(|s| crate::situations::TWISTS.contains(&s.as_str()) && !last.twists.contains(s)).collect();
        let text = if run.max_depth > last.depth {
            format!("deeper: D{}, last D{}", run.max_depth, last.depth)
        } else if run.max_depth < last.depth {
            format!("shallower: D{}, last D{}", run.max_depth, last.depth)
        } else if tier.name() != last.tier {
            format!("{}, last {}", past(tier.name()), past(&last.tier))
        } else if tier == ExitTier::Death && run.death_cause != last.cause {
            format!("died to {}, last {}", kind_title(run.death_cause.as_deref().unwrap_or("?")).to_lowercase(), kind_title(last.cause.as_deref().unwrap_or("?")).to_lowercase())
        } else if let Some(t) = twists.first() {
            format!("this time: the {}", crate::situations::twist_word(t))
        } else if kept != last.kept {
            format!("{}${} on last", if kept > last.kept { "+" } else { "−" }, (kept - last.kept).abs())
        } else if run.kills.len() as u32 != last.kills {
            format!("{} kills, last {}", run.kills.len(), last.kills)
        } else if run.turn < last.turns {
            "quicker than last".into()
        } else {
            "slower than last".into()
        };
        vec![crate::wire::News { k: "differ".into(), text }]
    }

    /// Cut 24 §2: the lineage takes in what `run` met — the named foes it placed, the floor
    /// events' lines it showed, the item kinds it found, and the run in brief.
    fn settle_novelty(&mut self, run: &Run, tier: ExitTier, _pct: i32) {
        let l = &mut self.lineage;
        for n in &run.named_placed {
            l.named_met.insert(n.clone(), run.id);
        }
        for (k, i) in &run.event_used {
            l.event_recent.entry(k.clone()).or_default().push((run.id, *i));
        }
        for v in l.event_recent.values_mut() {
            v.retain(|(r, _)| r + 1 >= run.id);
        }
        l.event_recent.retain(|_, v| !v.is_empty());
        for (_, k) in &run.found_units {
            if k != "gold" {
                l.found_kinds.insert(k.clone());
            }
        }
        // (a kind the lineage has packed is known to it: a later pile's leash is no find)
        for k in &run.packed {
            if !l.found_kinds.is_empty() {
                l.found_kinds.insert(k.clone());
            }
        }
        let twists = run.situations.iter().map(|(_, s)| s.clone()).collect();
        l.last_run = Some(RunBrief { depth: run.max_depth, tier: tier.name().into(), cause: run.death_cause.clone(), twists, kept: run.kept(tier), turns: run.turn, kills: run.kills.len() as u32 });
    }

    /// Bank the run's outcome into the lineage: marks, xp, renown, companions, deaths, rest.
    /// The vault decision (Addendum D) is left pending; `keep` or `auto_keep` finalises it.
    /// Cut 2 §2: yield follows the exit — bank 1.0 / return 0.6 / death 0.0 for gold, salvage,
    /// class XP and renown; a dead heir's kit stays on the floor as bones.
    pub fn finish_run(&mut self) -> Option<RunOutcome> {
        let run = self.run.take()?;
        let tier = run.over.unwrap_or(ExitTier::Return);
        let legacy_earned = if !self.sim {
            crate::legacy::ensure(&mut self.lineage);
            let legacy = self.lineage.hero_legacy.last_mut().expect("hero legacy");
            let earned = 1 + run.max_depth.saturating_sub(legacy.best_depth);
            legacy.points += earned;
            self.lineage.bloodline.as_mut().expect("bloodline").points += earned;
            legacy.best_depth = legacy.best_depth.max(run.max_depth);
            legacy.runs += 1;
            legacy.class = run.hero.class.name().into();
            self.batch.legacy_earned += earned;
            earned
        } else { 0 };
        // Cut 30.5: a send by hand is spent — the hero is home and waits (before the scout); the guide's record of
        // the stone it started from
        if !self.sim {
            self.lineage.tree.sent = false;
            // Passage was already paid at the send; a checkpoint can pay even on death.
            // Count actual income, not survival or a carry that rounds down to nothing.
            crate::tree::note_start(&mut self.lineage, run.start, run.passage > 0 || run.kept(tier) > 0, tier == ExitTier::Death);
        }
        // (c30-legible: the record before this run, for the end's reason)
        let best0 = self.lineage.best_depth;
        // Yield follows the exit (Cut 2 §2); a timed-out run yields nothing.
        let pct: i32 = run.yield_pct(tier);
        // Cut 13 §1: a stall (the guard fired `STALL_FIRES` times on one floor) is a run the
        // player can read: it gets a death-style record below.
        let stalled = run.timed_out && run.stuck_fires >= STALL_FIRES;
        let t = run.turn;
        // Cut 24 §2: what was new (against the lineage before this run settles into it).
        let news = if self.sim { Vec::new() } else { self.run_news(&run, tier, pct) };
        if !self.sim && (!self.lineage.found_kinds.is_empty() || self.lineage.next_run_id <= 2) {
            for (_, k) in &run.found_units {
                let name = self.lineage.wire_name(k).replace('_', " ");
                if k != "gold" && !self.lineage.found_kinds.contains(k) && !self.lineage.owned_kind(k) && !run.packed.contains(k) && !self.batch.new_finds.contains(&name) {
                    self.batch.new_finds.push(name);
                }
            }
        }
        self.settle_novelty(&run, tier, pct);
        let mut outcome = RunOutcome {
            run_id: run.id,
            depth: run.depth,
            tier: Some(tier),
            cause: run.death_cause.clone(),
            new_facts: (self.lineage.facts.len().saturating_sub(self.facts_at_run_start)) as u32,
            turns: run.turn,
            ..Default::default()
        };
        self.batch.runs += 1;
        self.batch.turns += run.turn;
        // Cut 19 §5: the den's thefts and the grudges avenged are the lineage's from now on.
        self.lineage.den_thefts += run.den_snatches;
        self.lineage.den_wakes += run.den_wakes;
        self.batch.thefts += run.stolen.len() as u32;
        self.batch.recovered += run.recovered.len() as u32;
        // QA on a946e04: keyed by kind (`Run.stolen_kinds`; a save from before by its label),
        // named at the report (`wire_name`).
        // QA on 0c6e126 (qaZ: the vault's `leather +1`, carried out and stolen, read `stolen leather`): a piece of gear goes by its
        // label, enchant and all (`leather +1`); a potion or scroll by its kind (named at the report).
        let gear = |kind: &str, label: &str| matches!(crate::defs::item_def(kind).cat, Cat::Weapon | Cat::Armour) && label.starts_with(kind) && label != kind;
        let kept: Vec<(String, i32)> = run.stolen_labels.iter().filter(|(id, _)| run.stolen_ids.contains(id)).map(|(id, l)| run.stolen_kinds.iter().find(|(k, _, _)| k == id).map(|(_, kind, g)| (if gear(kind, l) { l.clone() } else { kind.clone() }, *g)).unwrap_or_else(|| (l.clone(), 0))).collect();
        for (k, g) in &kept {
            *self.batch.stolen.entry(k.clone()).or_insert(0) += 1;
            *self.batch.stolen_gold.entry(k.clone()).or_insert(0) += g;
        }
        let kept_gold: i32 = kept.iter().map(|(_, g)| *g).sum();
        let kept_kinds: Vec<String> = kept.into_iter().map(|(k, _)| k).collect();
        // (Cut 22 §2: stolen coins are the line's `−$6 stolen`, not an item it names)
        let kept_by_thieves: Vec<String> = kept_kinds.iter().filter(|k| k.as_str() != "gold").map(|k| self.lineage.wire_name(k).replace('_', " ")).collect();
        self.batch.den_wakes += run.den_wakes;
        // Cut 20 §5: the bounty floor — taken when the run reached it and came home. QA on
        // e75ec29 (qaQ: `bounty D10 · missed` after the first absence, no bounty on the camp
        // before it): the report names only the floor the camp showed (`bounty_seen`) — a
        // night that closes inside an absence moves the floor for the runs after it, and the
        // camp shows the new one on the return.
        if let Some(bd) = run.bounty.filter(|b| Some(*b) == self.bounty_seen) {
            let taken = tier != ExitTier::Death && run.max_depth >= bd;
            let gold = if taken { run.bounty_gold.max(0) * run.yield_pct(tier) / 100 } else { 0 };
            match self.batch.bounty.as_mut() {
                Some(b) if b.depth == bd => {
                    b.taken |= taken;
                    b.gold += gold;
                }
                Some(b) if b.taken => {}
                _ => self.batch.bounty = Some(crate::wire::BountyReport { depth: bd, taken, gold }),
            }
        }
        for name in &run.avenged {
            for g in self.lineage.grudges.iter_mut().filter(|g| g.name == *name) {
                g.avenged = true;
            }
        }
        // Cut 29 §6: a grudge tamed closes as tamed (the report's `tamed Greth`, never `avenged`).
        for name in &run.tamed_grudges {
            for g in self.lineage.grudges.iter_mut().filter(|g| g.name == *name && !g.avenged) {
                g.tamed = true;
            }
        }
        // Cut 28 §1: the sworn oath — kept, its reward granted and a new oath drawn; the batch
        // counts the sends it was sworn over (the report's `oath · 2/16`).
        let mut oath_news: Option<String> = None;
        if !self.sim {
            // Cut 28b: a band boss seen opens the board
            self.lineage.oath_open |= run.wall_seen;
            if let Some((o, ok)) = crate::oath::settle(&mut self.lineage, &run) {
                let e = self.batch.oath.get_or_insert_with(|| (o.clone(), 0, 0, false));
                if e.0.id != o.id {
                    *e = (o.clone(), 0, 0, false);
                    self.batch.oath_breaks.clear();
                }
                e.1 += 1;
                e.2 += ok as u32;
                if ok {
                    e.3 = true;
                    oath_news = Some(format!("oath kept: {}", crate::oath::text(&o)));
                    self.lineage.heir_deed(format!("kept the oath {}", crate::oath::text(&o)));
                } else if let Some(c) = run.oath_said.as_ref().map(|(_, c)| c).filter(|c| !c.is_empty()) {
                    // Cut 28b: the exit line says what broke it (`oath broken: R2 return`)
                    *self.batch.oath_breaks.entry(c.clone()).or_insert(0) += 1;
                    oath_news = Some(format!("oath broken: {c}"));
                    // (the chronicle keeps it, once an heir: `broke the oath D3 · no rest`)
                    self.lineage.heir_deed(format!("broke the oath {}", crate::oath::text(&o)));
                }
            }
            // Cut 29 §2: a fork's sightings (the D5 route opens at the second).
            for f in crate::descent::FORKS.iter().copied().filter(|f| *f <= run.fork_seen && crate::descent::fork_open_for(run.route2, *f)) {
                *self.lineage.forks_seen.entry(f).or_insert(0) += 1;
            }
            // Cut 29 §1: the oaths in the extra slots, read at the run's end.
            for o in crate::oath::settle_extra(&mut self.lineage, &run) {
                self.batch.oaths_kept.push(o.clone());
                oath_news.get_or_insert_with(|| format!("oath kept: {}", crate::oath::text(&o)));
                self.lineage.heir_deed(format!("kept the oath {}", crate::oath::text(&o)));
            }
        }
        // Cut 29: the free vocabulary its gates opened; the systems the run's end triggered.
        crate::meta::grant_free(&mut self.lineage);
        if !self.sim {
            let opened = crate::systems::update(&mut self.lineage, false);
            self.batch.systems_opened.extend(opened);
        }
        self.batch.stalls += stalled as u32;
        self.batch.dances += (run.bloodless.2 >= crate::turn::DANCE_ACTIONS) as u32;
        self.batch.card_loops += (run.card_loops > 0) as u32;
        self.batch.loop_causes.extend(run.loop_causes.iter().cloned());
        self.batch.nohp.push(run.nohp.2);
        self.batch.driven_off += run.driven_off.is_some() as u32;
        self.batch.run_outcomes.push((run.depth, if tier == ExitTier::Death { run.death_cause.clone() } else { None }));
        self.batch.run_ticks.push(run.turn);
        self.batch.renderable_events += run.renderable_events;
        // Cut 5: the best-depth run anchors the reel; situations met on D1–5 are the §4 gate.
        if self.batch.best_run.is_none_or(|(d, _)| run.max_depth > d) {
            self.batch.best_run = Some((run.max_depth, run.id));
        }
        if !run.situations.is_empty() {
            self.batch.situation_runs += 1;
        }
        let band: Vec<&str> = crate::descent::SITUATION_DEPTHS.iter().map(|(k, _)| *k).collect();
        self.batch.band_runs.push(BandRun {
            depth: run.max_depth,
            met: run.situations.iter().map(|(_, s)| s.clone()).filter(|s| band.contains(&s.as_str())).collect(),
            passed: (*run.passed).clone(),
        });
        self.lineage.heir_best = self.lineage.heir_best.max(run.max_depth);
        if !run.ally_freed.is_empty() {
            self.lineage.heir_deed("freed a captive".into());
        }
        for name in &run.strays_tamed {
            self.lineage.lost.retain(|l| l.name != *name);
        }
        if let Some((_, k)) = run.tamed.first() {
            self.lineage.heir_deed(format!("tamed a {}", kind_title(k)));
        }
        // Cut 30 §3: a gift that acted in a banked run keeps its tier at the next wake.
        crate::traits::on_run_end(&mut self.lineage, &run, tier == ExitTier::Bank);
        match tier {
            ExitTier::Bank => {
                self.batch.banked += 1;
                self.lineage.runs_banked += 1;
            }
            ExitTier::Return => {
                self.batch.returned += 1;
                self.lineage.runs_returned += 1;
            }
            ExitTier::Death => self.lineage.runs_died += 1,
        }
        // Cut 16 §1: a bank or a return picks its floor; a timed-out run took nothing home.
        self.lineage.night_run(run.depth, run.max_depth, tier != ExitTier::Death && !run.timed_out);
        if self.batch.row_fired.len() < ROWS_TOTAL {
            self.batch.row_fired = vec![0; ROWS_TOTAL];
        }
        if self.batch.row_runs.len() < ROWS_TOTAL {
            self.batch.row_runs = vec![0; ROWS_TOTAL];
        }
        // (every compiled row's fires — the package rows past the pen's ten too: a later `attack nearest` read 'fired in 0 of N')
        for (i, n) in run.row_fired.iter().enumerate() {
            if i < ROWS_TOTAL {
                self.batch.row_fired[i] += n;
                if *n > 0 {
                    self.batch.row_runs[i] += 1;
                }
            }
        }
        // Cut 23 §3: the run's why-not tally joins the lineage's, row by row.
        let tally = std::mem::take(&mut self.row_tally);
        self.lineage.fold_row_tally(&tally);
        // Camp rest (Cut 2 §1): as long as the expedition, capped; a death is a fixed wake.
        let rest = self.rest_after(run.turn, tier);
        self.lineage.rest_left = rest;
        self.lineage.rest_watched = self.watched && !self.sim && !self.offline;
        self.events.push(Ev::Rest { t, seconds: rest.div_ceil(crate::offline::TICKS_PER_SECOND as u32) });
        // Marks (Cut 2 §2): new depth, boss, trophy, rank. First kills stay in bests and the ledger.
        let mut marks = 0;
        let mut wake_top = 0;
        let mut purse_full = false;
        // QA on 0c6e126 (qaZ: `avenged Vrak` for a name no screen had shown): a grudge is named where it is made — the death's line
        let mut new_grudge: Option<String> = None;
        let mut bests: Vec<String> = Vec::new();
        // Stall verdict window: a death or a new depth closes it; an exit row extends it.
        if let Some(r) = run.exit_row {
            *self.batch.exit_rows.entry(r).or_insert(0) += 1;
        }
        if tier == ExitTier::Death || run.max_depth > self.lineage.best_depth {
            self.stall = StallTally::default();
        } else {
            self.stall.runs += 1;
            self.stall.depth = self.stall.depth.max(run.max_depth);
            if let Some(r) = run.exit_row {
                *self.stall.exit_rows.entry(r).or_insert(0) += 1;
                self.stall.traces.insert(r, exit_trace(&run, &self.prov));
            }
            if !self.offline {
                self.stall.absent_rows.clear();
            } else if let Some(r) = run.exit_row {
                *self.stall.absent_rows.entry(r).or_insert(0) += 1;
            }
        }
        // Cut 29 §1 (docs/PROGRESSION.md §3: the frontier mark was 78 % of a rater's marks and paid
        // for the same D13 bank every run): no mark for a bank at the record; the night's mark
        // (`LineageState::clock_s`, paid by the absence) pays for coming home once a day.
        if run.max_depth > self.lineage.best_depth {
            marks += run.max_depth - self.lineage.best_depth;
            self.lineage.best_depth = run.max_depth;
            // Cut 29 §1 (E1): a new best ends the wall; its edit is off the table.
            self.lineage.best_day = self.lineage.day;
            self.lineage.wall_offer = None;
            bests.push(format!("D{}", run.max_depth));
        }
        // Cut 4: the first bank from each depth is a mark (a distinct best from first reach).
        if tier == ExitTier::Bank && !run.timed_out && self.lineage.banked_depths.insert(run.max_depth) {
            marks += 1;
            bests.push(format!("home:D{}", run.max_depth));
        }
        // Cut 21 §1: a bank lights the waystones at or above its floor (`waystone D9`).
        if tier == ExitTier::Bank && !run.timed_out {
            for d in self.lineage.light_waystones_on(run.depth, run.route) {
                bests.push(format!("waystone D{d}"));
                self.events.push(Ev::Note { t, text: format!("Waystone D{d} lit.") });
            }
        }
        for (_, kind, _) in &run.kills {
            if kind.starts_with("spectral_") {
                continue; // summons are not bests
            }
            if self.lineage.kills.insert(kind.clone()) {
                let boss = crate::defs::monster_def(kind).boss;
                if boss {
                    marks += 3;
                    self.lineage.heir_deed(format!("took the {}", crate::sifter::boss_short(kind)));
                }
                bests.push(if boss { format!("boss: {kind}") } else { format!("first kill: {kind}") });
            }
        }
        for tr in &run.trophies_run {
            if !self.lineage.trophies.contains(tr) {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {}", trophy_label(tr)));
            }
        }
        // Class XP (Addendum C, Cut 2 §2): only banked and returned runs feed it. Cut 7 §5: a
        // watched bank is worth half again (presence, inside the 1.5–3× band; offline unchanged).
        // Cut 7: XP is keyed on the content depth (`descent::tier_depth`), so a Fens floor is
        // worth what it was before the Warrens grew three floors.
        let raw: u32 = run.kills.iter().map(|(_, _, d)| 2 + crate::descent::tier_depth(*d)).sum::<u32>() / 4 + 3 * crate::descent::tier_depth(run.max_depth);
        let mut xp = raw * pct as u32 / 100;
        if self.watched && !self.sim && tier == ExitTier::Bank {
            xp += xp / 2;
        }
        let class = self.lineage.class;
        let prog = self.lineage.classes.entry(class.name().into()).or_insert(ClassProg { level: 1, xp: 0, next: 0 });
        prog.xp += xp;
        let mut level_ups = 0;
        while prog.level < MAX_LEVEL && prog.xp >= xp_to_next(prog.level) {
            prog.xp -= xp_to_next(prog.level);
            prog.level += 1;
            level_ups += 1;
            let lv = prog.level;
            self.events.push(Ev::Level { t, class: class.name().into(), level: lv });
            bests.push(format!("{} L{}", class.name(), lv));
        }
        let level_now = prog.level;
        if level_now >= MAX_LEVEL {
            let tr = format!("master:{}", class.name());
            if !self.lineage.trophies.contains(&tr) {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {}", trophy_label(&tr)));
                self.lineage.unlocks.insert(mastery_card(class).into());
            }
        }
        self.batch.xp_gained += xp;
        self.batch.level_ups += level_ups;
        // Companions (Addendum A): survivors return, level on bank; the dead are eggs.
        let eggs_before: Vec<u32> = self.lineage.eggs.iter().map(|e| e.id).collect();
        let alive_cids: Vec<u32> = run.party_alive().filter_map(|m| m.cid).collect();
        let egg_rests = self.lineage.egg_rests();
        for rec in &run.companions {
            let alive = alive_cids.contains(&rec.id);
            let recalled = run.recalled.contains(&rec.id);
            let in_party = self.lineage.party.iter().position(|c| c.id == rec.id);
            if alive || recalled {
                let mut c = rec.clone();
                c.hp = c.max_hp;
                if tier == ExitTier::Bank && alive && c.level < 5 {
                    c.level += 1;
                    c.max_rows = 1 + c.level as usize;
                    bests.push(format!("{} L{}", c.name, c.level));
                    // QA on e75ec29 (qaR: a pet levelled L1 → L3, never named until it fell).
                    self.events.push(Ev::Note { t: run.turn, text: format!("{}: level {}.", c.name, c.level) });
                }
                match in_party {
                    Some(i) => self.lineage.party[i] = c,
                    None => {
                        if (self.lineage.party.len() as u32) < self.lineage.party_slots() {
                            self.lineage.party.push(c);
                        } else {
                            self.lineage.kennel.push(c);
                        }
                    }
                }
            } else {
                if let Some(i) = in_party {
                    self.lineage.party.remove(i);
                }
                let eid = self.lineage.new_comp_id();
                self.lineage.eggs.push(Egg { id: eid, kind: rec.kind.clone(), tags: rec.tags.clone(), gen: rec.gen, hatch_in: egg_rests, from_loss: true });
                self.lineage.eggs_laid += 1;
                // Cut 5 §4: the lost companion may be met again, gone wild, by a later heir.
                let heir = run.heir;
                self.lineage.lost.retain(|l| l.name != rec.name);
                let why = run.fell_why.iter().rev().find(|(n, _)| *n == rec.name).map(|(_, w)| w.clone()).unwrap_or_default();
                // Cut 29 §6 (AX: Greth the tamed ogre, L5, gone with only `party −1 ogre`): the fall is
                // named on the report (`Fallen`) and in the chronicle's deeds.
                if !self.sim {
                    self.batch.fallen.push(crate::wire::Fallen { name: rec.name.clone(), kind: rec.kind.clone(), level: rec.level, depth: run.depth, why: why.clone(), heir });
                    self.lineage.heir_deed(format!("lost {} the {}", rec.name, kind_title(&rec.kind).to_lowercase()));
                }
                self.lineage.lost.push(Lost { kind: rec.kind.clone(), name: rec.name.clone(), gen: rec.gen, heir, why });
                while self.lineage.lost.len() > 6 {
                    self.lineage.lost.remove(0);
                }
                if !run.lost_companions.iter().any(|(_, n)| *n == rec.name) {
                    self.batch.lost.push(rec.name.clone());
                }
            }
        }
        for (_, k) in &run.tamed {
            self.batch.tamed.push(k.clone());
        }
        for (_, n) in &run.lost_companions {
            self.batch.lost.push(n.clone());
        }
        // Eggs incubate through the camp rest that follows every expedition (Cut 2 §1): three
        // rests (one with the incubator), not counting the run that laid them.
        for e in self.lineage.eggs.iter_mut() {
            if e.hatch_in > 0 && eggs_before.contains(&e.id) {
                e.hatch_in -= 1;
            }
        }
        let eggs = std::mem::take(&mut self.lineage.eggs);
        for e in eggs {
            if e.hatch_in == 0 {
                let kind = e.kind.clone();
                self.hatch_egg(e);
                self.events.push(Ev::Hatch { t, kind: kind.clone() });
                self.batch.hatched.push(kind);
            } else {
                self.lineage.eggs.push(e);
            }
        }
        // Lifetime trophies (Cut 2): the ledger's studied column, banking, bones, kills. Each is
        // a best worth two marks, spread across the arc so the camp has something to buy.
        let studied = self.lineage.facts.iter().filter(|f| f.starts_with("foe:") && f.ends_with(":studied")).count();
        let slain: u32 = self.lineage.kill_counts.values().sum();
        let homed = self.lineage.runs_banked + self.lineage.runs_returned;
        let lifetime: Vec<(bool, String)> = vec![
            (studied >= 5, "studied:5".into()),
            (studied >= 10, "studied:10".into()),
            (studied >= 14, "studied:14".into()),
            (homed >= 10, "home:10".into()),
            (homed >= 50, "home:50".into()),
            (homed >= 100, "home:100".into()),
            (homed >= 200, "home:200".into()),
            (!run.bones_found.is_empty(), "bones:1".into()),
            (slain >= 100, "slain:100".into()),
            (slain >= 500, "slain:500".into()),
            (slain >= 1000, "slain:1000".into()),
        ];
        for (ok, tr) in lifetime {
            if ok && !self.lineage.trophies.contains(&tr) {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {}", trophy_label(&tr)));
            }
        }
        // Cut 16 §3: the Burrows' kinds are the Warrens' (less the rats): their ledger is the
        // Warrens' trophy, not a second one.
        for biome in Biome::ALL.into_iter().filter(|b| *b != Biome::Burrows) {
            let tr = format!("ledger:{}", biome.name());
            if !self.lineage.trophies.contains(&tr)
                && crate::defs::biome_kinds(biome).iter().all(|k| self.lineage.facts.contains(&format!("tamed:{k}")))
            {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {}", trophy_label(&tr)));
            }
            // Cut 3: every kind of a biome studied (five kills each) — three marks.
            let tr = format!("studied_all_{}", biome.name());
            if !self.lineage.trophies.contains(&tr) && crate::defs::biome_kinds(biome).iter().all(|k| self.lineage.studied(k)) {
                self.lineage.trophies.push(tr.clone());
                marks += 3;
                bests.push(format!("trophy: {}", trophy_label(&tr)));
            }
        }
        // Bones recovered this run (Cut 2 §2): the piles leave the lineage; their items are now
        // the hero's and follow the exit like anything else.
        let mut highlights = crate::sifter::sift(&run, &self.lineage);
        for heir in &run.bones_found {
            if let Some(i) = self.lineage.bones.iter().position(|b| b.heir == *heir) {
                let b = self.lineage.bones.remove(i);
                self.batch.bones_found.push(format!("heir {} · D{} · {} items", b.heir, b.depth, b.items.len()));
                self.lineage.heir_deed(format!("found ♟{}'s bones", b.heir));
                if b.named {
                    let text = format!("Recovered heir {}'s bones on D{}.", b.heir, b.depth);
                    highlights.push(Highlight { pattern: "bones".into(), score: crate::sifter::BONES, t, run_id: run.id, text, arc: None });
                }
            }
        }
        // Highlights and renown (Addendum D; Cut 2 §2: renown per absence = the best run's
        // score, and only banked or returned runs score).
        let kill_value: u32 = run.kills.iter().map(|(_, k, _)| kill_value(k)).sum();
        let raw_score = 10 * run.max_depth + kill_value + 25 * run.boss_kills.len() as u32 + highlights.iter().map(|h| h.score as u32).sum::<u32>();
        let score = raw_score * pct as u32 / 100;
        self.batch.best_score = self.batch.best_score.max(score);
        for h in &highlights {
            self.reel.push(h.clone());
            self.batch.highlights.push(h.clone());
        }
        self.reel.sort_by(|a, b| b.score.cmp(&a.score).then(a.run_id.cmp(&b.run_id)).then(a.t.cmp(&b.t)));
        self.reel.truncate(50);
        self.lineage.marks += marks;
        self.batch.marks += marks;
        outcome.new_best = !bests.is_empty();
        self.batch.bests.extend(bests);
        // Loot, gold and the vault (Addendum B/D; Cut 2 §2 death keeps nothing).
        // Cut 4: `run.loot` is already gold (÷ 4 at pickup); the same number the exit note
        // and `Ev::Exit.loot_kept` carried.
        let loot_kept = run.kept(tier);
        let gold_before = self.lineage.gold;
        let exit_why = match tier {
            _ if run.timed_out && run.stuck_fires >= STALL_FIRES => format!("stalled D{}", run.max_depth),
            _ if run.timed_out => format!("lost thread D{}", run.max_depth),
            // QA on 0c6e126 (qaY: `0 RETURNED · 3 DRIVEN` over three `returned D8 · $268 lost`): a
            // drive-off is its own word in the ledger, as on its line and its tile.
            ExitTier::Return if run.driven_off.is_some() => format!("driven D{}", run.max_depth),
            ExitTier::Bank => format!("banked D{}", run.max_depth),
            ExitTier::Return => format!("returned D{}", run.max_depth),
            ExitTier::Death => format!("died D{}", run.depth),
        };
        self.lineage.gold_move(loot_kept, &exit_why);
        if self.lineage.town.auto_collect && loot_kept > 0 && !crate::tree::hired(&self.lineage, "porter") {
            crate::tree::did(&mut self.lineage, "chest");
        }
        // QA on 912e135 (qaW): the exit's ledger line names what it did not keep.
        if let Some(g) = self.lineage.gold_ledger.last_mut().filter(|g| g.why == exit_why) {
            g.lost = (run.carried() - loot_kept).max(0);
        }
        self.batch.gold_lost += (run.carried() - loot_kept).max(0);
        if loot_kept > 0 {
            self.batch.gold_unkept += (run.carried() - loot_kept).max(0);
        }
        self.batch.heirs = Some(self.batch.heirs.map_or((run.heir, run.heir), |(lo, hi)| (lo.min(run.heir), hi.max(run.heir))));
        self.batch.gold_earned += loot_kept;
        let mut all: Vec<Item> = (*run.hero.inv).clone();
        if let Some(w) = &*run.hero.weapon {
            all.push(w.clone());
        }
        if let Some(a) = &*run.hero.armour {
            all.push(a.clone());
        }
        // QA on 778fa1b (qaV): the melee weapon parked while the bow is up comes home too.
        if let Some(w) = &run.bow_swap {
            all.push(w.clone());
        }
        all.retain(|i| i.cat() != Cat::Gold && !crate::kit::is_kit_id(i.id) && !matches!(i.kind.as_str(), "bones" | "trap"));
        // QA on 778fa1b (qaV: finds in no named place): where each found item in the pack ends
        // (`found_rows`), by id.
        let mut pack: BTreeMap<(u32, String), i32> = BTreeMap::new();
        for i in &all {
            *pack.entry((i.id, i.kind.clone())).or_insert(0) += i.amount.max(1);
        }
        // Run-clear: the pack's finds as they came home (before the shelf, the sheet or the bones take them)
        let found_items: Vec<Item> = if self.sim { Vec::new() } else { all.iter().filter(|i| run.found_units.iter().any(|(id, k)| *id == i.id && *k == i.kind)).cloned().collect() };
        let mut exit_fate: BTreeMap<(u32, String), &'static str> = BTreeMap::new();
        let mut shelved: Vec<String> = Vec::new();
        let mut leash_back = 0usize;
        if tier == ExitTier::Death {
            // Insured brought items come home (Melvor insurance); everything else stays on the
            // floor as a bones pile for a later heir.
            let (insured, rest): (Vec<Item>, Vec<Item>) = all.into_iter().partition(|i| run.brought.contains(&i.id) && self.lineage.insured.contains(&i.id));
            for it in insured {
                self.lineage.insured.retain(|id| *id != it.id);
                self.lineage.vault.push(it);
            }
            all = Vec::new();
            for it in &rest {
                exit_fate.insert((it.id, it.kind.clone()), "bones");
            }
            if !rest.is_empty() {
                let named = !self.batch.bests.is_empty() || !run.boss_kills.is_empty();
                self.lineage.bones.push(Bones { heir: run.heir, depth: run.depth, items: rest.clone(), named, biome: Some(run.biome()) });
                while self.lineage.bones.len() > BONES_MAX {
                    self.lineage.bones.remove(0);
                }
                self.events.push(Ev::Bones { t, heir: run.heir, items: rest.len() as u32 });
            }
            let f = format!("bones:{}", run.depth);
            if self.lineage.facts.insert(f.clone()) {
                self.events.push(Ev::Fact { t, fact: f });
            }
        } else {
            // A packed supply the send did not use goes back on the shelf (never into the
            // vault, never salvaged): a $40 heal salvaged for $2 and rebought for $40 on
            // every banked run read as the shop robbing the player (QA on 50bb162, both
            // players). The kennel's leash is the kennel's (`kennel_leash` puts it back).
            let (back, rest): (Vec<Item>, Vec<Item>) = all.into_iter().partition(|i| run.supplies.contains(&i.id) && i.kind != "leash");
            // The kennel's own leash goes back to the kennel, not to salvage (QA on 56f2a1d:
            // `leash ×1 · $1` salvaged, `leash 1/5` at the forge, the leash still on the shelf).
            // QA on 524827b (qaAB): the packed leash stack whatever the unit that opened it (a
            // bought one first made it a bought stack, which went to salvage).
            let kennel: Vec<Item> = rest.iter().filter(|i| i.kind == "leash" && run.supplies.contains(&i.id)).cloned().collect();
            all = rest.into_iter().filter(|i| !(i.kind == "leash" && run.supplies.contains(&i.id))).collect();
            // QA on 778fa1b (qaV): leashes found and stacked on the kennel's go home as a find
            // of their own (the kennel's stack goes back to the kennel; they went with it).
            for k in kennel {
                let found = run.found_left(k.id, &k.kind) as i32;
                // (packed units are spent first: what is left of the stack is found first)
                let n = found.min(k.amount);
                if n > 0 {
                    let mut it = k.clone();
                    it.amount = n;
                    it.free = false;
                    all.push(it);
                }
                // QA on 524827b (qaAB: `leash $30` bought, a returned run, the loadout back to the
                // free one alone): the packed units left — the free one spent first on a tame —
                // that were bought go back on the shelf, bought, as any unused supply.
                let packed_left = (k.amount - n).max(0) as usize;
                let bought_back = packed_left.min(run.bought_leashes.len());
                for paid in run.bought_leashes.iter().take(bought_back) {
                    if self.lineage.supplies.len() >= self.lineage.supply_cap() {
                        break;
                    }
                    let id = self.lineage.next_vault_id;
                    self.lineage.next_vault_id += 1;
                    let mut it = Item::new(id, "leash");
                    it.amount = 1;
                    it.known = true;
                    it.paid = *paid;
                    self.lineage.supplies.push(it);
                    leash_back += 1;
                }
            }
            let cap = self.lineage.supply_cap();
            for mut it in back {
                if self.lineage.supplies.len() >= cap {
                    break;
                }
                it.free = false;
                if !self.lineage.supplies.iter().any(|s| s.id == it.id) {
                    self.lineage.supplies.push(it);
                }
            }
            // Cut 21 §2 (AE: "sells heal potions he finds for $2 while I pay $40"): a found
            // supply of a kind the shelf sells and a row can use goes onto the shelf, up to
            // the cap, not to salvage — the next send packs it free. The kinds the repeat
            // re-packs are served first (a found heal stands in for a bought one); another
            // row's kind takes only a slot the repeat does not need.
            let before: Vec<(u32, String)> = all.iter().map(|i| (i.id, i.kind.clone())).collect();
            let (to_shelf, rest) = self.shelve_found(all, &run);
            for key in before.into_iter().filter(|(id, k)| !rest.iter().any(|i| i.id == *id && i.kind == *k)) {
                exit_fate.insert(key, "shelved");
            }
            all = rest;
            shelved = to_shelf;
        }
        // A return keeps the dearest 60 %; what the vault sent along is the player's already
        // and comes first, whatever it is worth (QA on 952e306: a vaulted poison potion,
        // cheapest in the pack, was cut on a bail — `VAULT 1/1 → 0/1` with no line).
        // Cut 28 §4: a caged item is the player's choice as much — it comes next.
        all.sort_by(|a, b| run.brought.contains(&b.id).cmp(&run.brought.contains(&a.id)).then(run.caged.contains(&b.id).cmp(&run.caged.contains(&a.id))).then(b.value().cmp(&a.value())).then(a.id.cmp(&b.id)));
        let n_keep = match tier {
            ExitTier::Bank => all.len(),
            ExitTier::Return => ((all.len() * 60).div_ceil(100)).max(all.iter().filter(|i| run.brought.contains(&i.id) || run.caged.contains(&i.id)).count()),
            ExitTier::Death => 0,
        };
        let eligible: Vec<Item> = all.iter().take(n_keep).cloned().collect();
        let rest_items: Vec<Item> = all.into_iter().skip(n_keep).collect();
        for it in &eligible {
            exit_fate.insert((it.id, it.kind.clone()), "sheet");
        }
        for it in &rest_items {
            exit_fate.insert((it.id, it.kind.clone()), "salvaged");
        }
        // (one entry a unit: a stack of found leashes settles each)
        // (the finds on the sheet, one entry a unit — as `found_rows` counts them)
        let found_ids: Vec<(u32, String)> = exit_fate
            .iter()
            .filter(|(_, f)| **f == "sheet")
            .flat_map(|(key, _)| std::iter::repeat_n(key.clone(), found_in_pack(&run, &pack, key) as usize))
            .collect();
        let cut_coins = self.salvage(&rest_items, pct);
        // Per kind, the coins the cut brought — the ledger's own (QA on 92eb880: `sold
        // aggravate $2` on the keep sheet, `aggravate ×1 · $1` in the report).
        let mut cut: std::collections::BTreeMap<String, (u32, i32)> = std::collections::BTreeMap::new();
        for (it, c) in rest_items.iter().zip(&cut_coins) {
            let e = cut.entry(it.kind.clone()).or_insert((0, 0));
            e.0 += 1;
            e.1 += c;
        }
        // QA on 912e135 (qaX: the forge's `leash · salvaged 1/5` beside a SALVAGED list without it — a $0 coin was dropped): a
        // return's cut names every item it salvaged, a $0 one too (the forge counts them all); a death salvages nothing.
        let cut_rows: Vec<crate::wire::SalvageRow> = cut.into_iter().map(|(k, (n, c))| crate::wire::SalvageRow { kind: self.lineage.wire_name(&k), n, gold: c }).filter(|_| pct > 0).collect();
        let brought: Vec<u32> = eligible.iter().filter(|i| run.brought.contains(&i.id)).map(|i| i.id).collect();
        self.pending_exit = Some(PendingExit { run_id: run.id, tier, items: eligible, pct, brought, found: found_ids, stalled });
        // Death: graveyard, grudge, heir, record.
        if tier == ExitTier::Death {
            let cause = run.death_cause.clone().unwrap_or_else(|| "unknown".into());
            *self.batch.deaths.entry(cause.clone()).or_insert(0) += 1;
            // The worst death: the deepest, the latest on a tie; Cut 13 §1: a stall at the
            // same depth yields to it (`worst_stall`).
            if run.depth >= self.batch.worst_depth {
                self.batch.worst_depth = run.depth;
                self.batch.worst_death = Some(run.id);
                self.batch.worst_stall = false;
            }
            let mut deeds: Vec<String> = self.batch.bests.iter().rev().take(3).cloned().collect();
            for (_, k) in &run.boss_kills {
                let d = format!("slew the {}", crate::defs::monster_def(k).title);
                if !deeds.contains(&d) {
                    deeds.push(d);
                }
            }
            deeds.truncate(3);
            // The record is taken before this death's grudge joins the lineage: a floor the
            // verdict's replay generates inside its window would otherwise hold a grudge the
            // run never met (Cut 13: `DeathRec.t10_lineage`).
            let rec = (!self.sim).then(|| crate::trace::death_record(self, &run));
            // Cut 9 §7: the grave points at its death record while the engine keeps it.
            self.lineage.graveyard.push(Grave { heir: run.heir, depth: run.depth, cause: cause.clone(), deeds, death_id: (!self.sim).then_some(run.id) });
            if crate::defs::MONSTERS.iter().any(|m| m.kind == cause && !m.boss && !m.tags.contains(&"summoned"))
                && !self.lineage.grudges.iter().any(|g| g.kind == cause && g.depth == run.depth)
            {
                // QA on 524827b (qaAB: `grudge: Morog the goblin archer`, `avenged Morog`, then `grudge: Morog the
                // ogre`): a name is one foe for the lineage — never one a grudge (open or avenged) already holds
                // Cut 28 §4 (AV: `grudge: Drix the jackal` — Drix was his own jackal): nor a companion's name
                // (the party, the kennel, the lost)
                let taken = |l: &LineageState, name: &str| l.grudges.iter().any(|g| g.name == name) || l.all_companions().any(|c| c.name == name) || l.lost.iter().any(|c| c.name == name) || run.companions.iter().any(|c| c.name == name);
                let mut name = crate::descent::grudge_name(&mut self.lineage.rng);
                for _ in 0..32 {
                    if !taken(&self.lineage, &name) {
                        break;
                    }
                    name = crate::descent::grudge_name(&mut self.lineage.rng);
                }
                new_grudge = Some(format!("grudge: {name} the {}", kind_title(&cause).to_lowercase()));
                self.lineage.grudges.push(Grudge { kind: cause.clone(), name, depth: run.depth, heir: run.heir, avenged: false, tamed: false, biome: Some(run.biome()) });
            }
            // Cut 5 §2: the heir's line in the lineage chronicle.
            let bones_left = self.lineage.bones.last().is_some_and(|b| b.heir == run.heir);
            let tail = bones_left.then(|| format!("left bones on D{}", run.depth));
            self.lineage.chronicle_heir_at(&format!("fell to {}", crate::sifter::cause_phrase(&cause)), tail, Some(run.depth));
            // Cut 13 §2: the next heir wakes with two traits on offer.
            self.lineage.new_heir();
            // Wake pay: a new heir arrives with enough for one cheap supply, so a lineage that has
            // never banked is not gold-locked out of the shop after a death (cohort 5, rater J:
            // "$0 after death, seven identical deaths overnight"). Bounded: tops the purse up to
            // WAKE_PAY, never adds on top of it.
            if self.lineage.gold < WAKE_PAY {
                let top = WAKE_PAY - self.lineage.gold;
                self.lineage.gold_move(top, "wake pay");
                self.batch.wake_pay += top;
                self.batch.wake_n += 1;
                wake_top = top;
            } else {
                // QA on 778fa1b (qaU: `purse full` on death lines carrying $66 and $257 with
                // $1434 in the camp — read as a cap): the flag explains a missing top-up only
                // where one was to be expected, a purse just over the line (under
                // `PURSE_FULL_BAND`); a richer lineage's death says nothing of the purse.
                purse_full = self.lineage.gold < PURSE_FULL_BAND;
            }
            if let Some(rec) = rec {
                self.deaths.insert(run.id, rec);
                while self.deaths.len() > self.max_deaths.max(KEPT_DEATHS) {
                    let k = *self.deaths.keys().next().unwrap();
                    self.deaths.remove(&k);
                }
                self.prune_graves();
            }
        }
        // Cut 13 §1: the stall's record — the guard's moment as the cause, the trace, the
        // patches measured from the first guard (`trace::stall_record`); `death(id)` returns
        // it. It is a worst candidate below a death at its depth or deeper: the deepest stall
        // leads the report only when no death reached its floor.
        if stalled && !self.sim {
            if run.depth > self.batch.worst_depth || (run.depth == self.batch.worst_depth && (self.batch.worst_stall || self.batch.worst_death.is_none())) {
                self.batch.worst_depth = run.depth;
                self.batch.worst_death = Some(run.id);
                self.batch.worst_stall = true;
            }
            let rec = crate::trace::stall_record(self, &run);
            self.deaths.insert(run.id, rec);
            while self.deaths.len() > self.max_deaths.max(KEPT_DEATHS) {
                let k = *self.deaths.keys().next().unwrap();
                self.deaths.remove(&k);
            }
            self.prune_graves();
        }
        if run.ended && tier != ExitTier::Death {
            // Cut 5 §2: the heir who reached the bottom retires from the chronicle at its rank.
            let rank = self.lineage.rank;
            self.lineage.chronicle_heir(&format!("retired at rank {rank}"), None);
        }
        self.history.clear();
        outcome.new_facts = (self.lineage.facts.len().saturating_sub(self.facts_at_run_start)) as u32;
        if !self.offline {
            self.settle_renown(t);
        }
        if self.sim {
            self.auto_keep();
        }
        // Cut 13 §3: what this run used to no effect is not rebought for the next.
        self.lineage.last_wasted = run.wasted_kinds.clone();
        // Cut 22 §1: a bought supply thieves kept is re-bought once a night; a later one waits
        // for the night's end.
        for (id, kind, _) in &run.stolen_kinds {
            if !run.supplies.contains(id) || !run.stolen_ids.contains(id) || kind == "leash" {
                continue;
            }
            if self.lineage.night_theft_rebought {
                self.lineage.theft_skip.push(kind.clone());
            } else {
                self.lineage.night_theft_rebought = true;
            }
        }
        // Cut 21 §2 (AF: `returned $0 · stalled · repeat −$80`): a stall does not re-pack on
        // top of the loss — the next send's re-pack buys what the shelf lacks then.
        let spent_on = if stalled { Vec::new() } else { self.restock_at(true) };
        // Cut 8B §3: the kennel's leash is back on the shelf while nothing has been tamed.
        self.lineage.kennel_leash();
        // Cut 6 §1: the ledger line — carried × keep% → kept, what the automations spent on
        // coming home, and where the kit went on a death.
        let spent: i32 = self.lineage.gold_ledger.iter().rev().take_while(|g| g.t == self.lineage.total_turns).filter(|g| g.delta < 0 && !g.why.starts_with("salvage")).map(|g| -g.delta).sum();
        let pile = if tier == ExitTier::Death { self.lineage.bones.last().filter(|b| b.heir == run.heir).map(|b| b.items.clone()).unwrap_or_default() } else { Vec::new() };
        let bones_n = pile.len();
        let unused = run.hero.inv.iter().filter(|i| run.supplies.contains(&i.id) && i.kind != "leash").count() + leash_back;
        let mut line = exit_line_of(run.carried(), pct, loot_kept, spent, spent_on, tier, run.timed_out, run.stuck_fires >= STALL_FIRES, unused, bones_n, run.depth);
        line.bloodline_id=self.lineage.bloodline_id;
        // Cut 30.5: what the checkpoints secured leads the arithmetic (`banked $120 · $80 secured + 100% of $40`)
        if run.secured > 0 {
            line.secured = run.secured;
            line.text = secured_text(&line.text, loot_kept, run.secured, run.loot.max(0), pct);
        }
        // QA on 0c6e126 (qaZ: `1 supply back` never named which — read beside `sold … invisibility $1` as the bought potion sold): the
        // supplies that came back unused, named when they are of one kind (`1 supply back: invisibility`)
        let mut back: BTreeSet<String> = run.hero.inv.iter().filter(|i| run.supplies.contains(&i.id) && i.kind != "leash").map(|i| self.lineage.wire_name(&i.kind).replace('_', " ")).collect();
        if leash_back > 0 {
            back.insert("leash".into());
        }
        if unused > 0 && tier != ExitTier::Death && back.len() == 1 {
            let seg = format!(" · {unused} {} back", if unused == 1 { "supply" } else { "supplies" });
            if let Some(k) = back.first() {
                line.text = line.text.replacen(&seg, &format!("{seg}: {k}"), 1);
            }
        }
        line.news = news;
        if let Some(t) = oath_news {
            line.news.insert(0, crate::wire::News { k: "oath".into(), text: t });
            line.news.truncate(3);
        }
        if let Some(g) = new_grudge.filter(|_| !self.sim) {
            line.news.insert(0, crate::wire::News { k: "named".into(), text: g });
            line.news.truncate(3);
        }
        // Cut 24 §1: a boss that could not be hurt drove him off — `no counter` with its defence
        // and the counter row to write (`ExitLine.driven`); the text ends `· no counter`.
        // QA on 0c6e126 (qaY: `0 RETURNED · 3 DRIVEN` beside lines `returned $0 · $388 lost · no counter`):
        // the line leads with its own word, `driven` (the tile's and the ledger's), and names who
        // drove him off (`· by Warlord`); `no counter` is the COUNTER tablet's verdict, beside its `try:`.
        if let Some(kind) = &run.driven_off {
            let row = crate::facts::counter_row(kind);
            // Cut 27 §5: the counter already in the set (a row of its verb, or a card carrying one) is
            // an `order` drive-off — the rows above it that acted in the fight kept it from its turn.
            let (held, over) = crate::trace::driven_order_in(self.lineage.rules(), &row, &run.trace);
            let verdict = if held.is_some() { "order" } else { "no counter" };
            let d = crate::wire::DrivenOff { boss: kind.clone(), title: crate::sifter::boss_short(kind).into(), depth: run.depth, verdict: verdict.into(), defence: crate::facts::boss_trait(kind).into(), counter: crate::packages::counter_offer(&self.lineage, kind, &row).0, row, run_id: run.id, hp: run.hero.hp, max_hp: run.hero.max_hp, lost: line.carried - line.kept, held, over };
            if !self.sim {
                self.batch.drives.push(d.clone());
                while self.batch.drives.len() > EXITS_CAP {
                    self.batch.drives.remove(0);
                }
            }
            if let Some(rest) = line.text.strip_prefix("returned ") {
                line.text = format!("driven {rest}");
            }
            line.text.push_str(&format!(" · by {}", d.title));
            line.driven = Some(d);
        }
        // What the run earned besides gold, on its own line (QA on 50bb162: `◆7` and `$40` at
        // the camp after a death with no source on the death screen).
        // Cut 15 §1: a frontier bank's mark is part of the total and named once: `◆+1
        // frontier` alone, `◆+3 (1 frontier)` beside a new depth's marks.
        if marks > 0 {
            line.text.push_str(&format!(" · ◆+{marks}"));
        }
        if wake_top > 0 {
            // QA on 912e135 (qaW: `heir purse +$40` in the walk, `+$30` live — "no screen says the
            // purse tops up to $40"): the top-up names the purse it fills to.
            // QA on 524827b (qaAB: `+$40 heir purse` after ♟1's death, nothing after the rest — "no rule"): the top-up always
            // names the line it fills to (`+$40 wake → $40`), so a later death with a fuller purse reads as the same rule
            line.text.push_str(&format!(" · +${wake_top} wake → ${WAKE_PAY}"));
        }
        // QA on e75ec29 (qaR: the heir purse paid after one death and not three others; packed
        // heals stolen on D1 and nothing on the exit): a death that found the purse full at
        // `WAKE_PAY` says so, and what thieves took and kept rides the line (`· stolen heal`,
        // the client's; the text stays ≤ 14 words).
        line.purse_full = purse_full;
        line.wake = wake_top;
        line.stolen = kept_by_thieves.clone();
        // QA on a946e04 (qaT): the send's toll and where it started, on its exit's line.
        line.toll = run.toll;
        line.start = run.start.max(1);
        line.start_short = run.start_short;
        line.stolen_gold = kept_gold;
        // QA on 778fa1b: what the pack's swaps took off the carry, on the line beside the thefts.
        line.swapped = run.swapped;
        if tier == ExitTier::Death {
            line.cause = run.death_cause.as_deref().map(crate::sifter::cause_phrase);
        }
        line.reason = Some(exit_reason(&run, tier, best0, self.lineage.rules()));
        for l in &run.swap_left {
            match line.swap_left.iter_mut().find(|c| c.kind == *l) {
                Some(c) => c.n += 1,
                None => line.swap_left.push(crate::wire::KindCount { kind: l.clone(), n: 1 }),
            }
        }
        self.batch.swapped += run.swapped;
        // QA on 912e135 (qaW): the pile the `bones: N items` counts, per kind, one an item.
        for it in &pile {
            let kind = self.lineage.wire_name(&it.kind).replace('_', " ");
            match line.bones.iter_mut().find(|c| c.kind == kind) {
                Some(c) => c.n += 1,
                None => line.bones.push(crate::wire::KindCount { kind, n: 1 }),
            }
        }
        // Cut 21 §2: the found supplies this exit shelved, per kind (the client's `found heal →
        // shelf`; not in `text`).
        for k in &shelved {
            let kind = self.lineage.wire_name(k).replace('_', " ");
            match line.shelved.iter_mut().find(|c| c.kind == kind) {
                Some(c) => c.n += 1,
                None => line.shelved.push(crate::wire::KindCount { kind, n: 1 }),
            }
        }
        // Cut 9 §5: every exit carries its last five hero turns (read off the run's own trace
        // ring: nothing more per tick).
        line.found = found_rows(&run, &pack, &exit_fate, |k| self.lineage.wire_name(k).replace('_', " "));
        line.found_n = run.found_units.len() as u32;
        // Run-clear: the end's kind, its floor, a record, and what the run found and kept (or left), rarest first
        line.end = match tier {
            ExitTier::Bank => "bank",
            ExitTier::Return => "return",
            ExitTier::Death => "death",
        }
        .into();
        line.reached = run.max_depth.max(run.depth);
        line.new_best = run.max_depth > run.best_at_send.unwrap_or(best0);
        line.finds = exit_finds(&found_items, &exit_fate, &self.lineage.facts, &self.lineage.flavours);
        line.trace = Some(exit_trace(&run, &self.prov));
        line.salvaged = cut_rows;
        line.run_id = run.id;
        line.legacy_earned = legacy_earned;
        line.xp = xp;
        line.level_ups = level_ups;
        // Cut 29 §3: the run metered — on its line, the absence's sum, the night's, the last two runs'.
        if !self.sim && !run.meters.is_empty() {
            line.meters = Some(Box::new(crate::meters::wire(&run.meters.run)));
            self.batch.meters.add(&run.meters.run);
            self.lineage.night_meter.add(&run.meters.run);
            self.lineage.meters_recent.push(run.meters.run.clone());
            while self.lineage.meters_recent.len() > 2 {
                self.lineage.meters_recent.remove(0);
            }
        }
        debug_assert!(self.lineage.gold - gold_before == loot_kept - spent + self.lineage.gold_ledger.iter().rev().take_while(|g| g.t == self.lineage.total_turns).filter(|g| g.why.starts_with("salvage")).map(|g| g.delta).sum::<i32>());
        if tier == ExitTier::Death || stalled {
            if let Some(rec) = self.deaths.get_mut(&run.id) {
                // The death's (or the stall's) own trace is longer; its line does not repeat it.
                rec.death.line = Some(ExitLine { trace: None, ..line.clone() });
            }
        }
        self.batch.exits.push(line.clone());
        while self.batch.exits.len() > EXITS_CAP {
            self.batch.exits.remove(0);
        }
        // RUNS_UI: the runs log
        if !self.sim {
            let rec = crate::wire::RunRec {
                id: run.id,
                heir: run.heir,
                via: if self.offline { "away" } else if self.watched { "watched" } else { "town" }.into(),
                absence: self.offline.then_some(self.lineage.absences),
                clock_s: self.clock_at.unwrap_or(self.lineage.clock_s),
                start: run.start.max(1),
                depth: run.depth,
                tier: tier.name().into(),
                reason: line.reason.clone(),
                gold: line.kept,
                found: line.found_n,
                kept: line.found.iter().filter(|f| f.fate == "kept").map(|f| f.kind.clone()).collect(),
                turns: run.turn,
                best: outcome.new_best,
                death_id: (matches!(tier, ExitTier::Death) || stalled).then_some(run.id).filter(|id| self.deaths.contains_key(id)),
                sampled: None,
                finds: line.finds.clone(),
                secured: line.secured,
            };
            self.lineage.push_run(rec);
        }
        self.last_exit = Some(line);
        // Cut 24 §3: the forge's prices are fixed the first time it is shown.
        crate::kit::lock_unit(&mut self.lineage);
        // Cut 28 §1: the board stands on the lineage as this run left it (a boss slain, a waystone lit,
        // the best depth, the night's income)
        if !self.sim {
            crate::oath::refresh(&mut self.lineage);
            // Cut 30 §1–2: the run's meetings (drills, scars), its packages' runs (levels), the stages
            // that came, the pen — the set recompiled for the next send.
            let set = self.lineage.rules().clone();
            let lines = crate::packages::on_run_end(&mut self.lineage, &run.bosses_met, run.max_depth, &run.row_fired, &set);
            let beats = crate::packages::beats(&lines);
            if let Some(line) = self.last_exit.as_mut() { line.packages = beats.clone(); }
            if let Some(line) = self.batch.exits.last_mut() { line.packages = beats.clone(); }
            if let Some(line) = self.deaths.get_mut(&run.id).and_then(|r| r.death.line.as_mut()) { line.packages = beats; }
            self.batch.pkg_lines.extend(lines);
            // Cut 30 §3, §5: the town builds what its triggers brought; the quest board reads the run.
            for b in crate::town::update(&mut self.lineage) {
                self.batch.pkg_lines.push(format!("built {b}"));
            }
            let slew: Vec<String> = run.boss_kills.iter().map(|(_, k)| k.clone()).collect();
            if let Some(line) = crate::town::on_run(&mut self.lineage, run.max_depth, tier, run.home_return, run.depth, &slew) {
                self.batch.pkg_lines.push(line);
            }
        }
        Some(outcome)
    }

    /// Cut 9 §7: only the last `KEPT_DEATHS` graves keep their `death_id`, and only while
    /// the record is still held.
    pub fn prune_graves(&mut self) {
        let n = self.lineage.graveyard.len();
        for (i, g) in self.lineage.graveyard.iter_mut().enumerate() {
            if i + KEPT_DEATHS < n || g.death_id.is_some_and(|id| !self.deaths.contains_key(&id)) {
                g.death_id = None;
            }
        }
    }

    /// Renown per absence (Cut 2 §2): the best single run's score, settled once per report
    /// offline and per run when watched. Ranks at `100 r²` grant a mark each.
    pub fn settle_renown(&mut self, t: u32) {
        let score = std::mem::take(&mut self.batch.best_score);
        if score == 0 {
            return;
        }
        self.lineage.renown += score;
        self.batch.renown_gained += score;
        while self.lineage.renown >= 100 * (self.lineage.rank + 1) * (self.lineage.rank + 1) {
            self.lineage.rank += 1;
            self.lineage.marks += 1;
            self.batch.marks += 1;
            self.batch.ranks_up += 1;
            let r = self.lineage.rank;
            self.events.push(Ev::Rank { t, rank: r });
            self.batch.bests.push(format!("rank {r}"));
        }
    }

    /// Salvage items: gold by tier, forge ledger by full count (Addendum D).
    /// Salvage `items` at `pct`: each pays its coins by `salvage_coins` (the exit sheet's
    /// `worth`, the cut's `sold` rows and the report's rows are these very coins). Returns them.
    /// Cut 21 §2: the found supplies of `all` that go onto the shelf at a non-death exit
    /// (`finish_run`) — potions and scrolls known by name or flavour, of a kind the shelf
    /// sells and a row of the active set uses (`LineageState::row_kinds`), not brought from
    /// the vault or packed at the send — each standing in for one the repeat would buy (the
    /// last send's kinds, less what the shelf holds), up to the cap. Returns the kinds shelved
    /// (one per item) and the items left.
    ///
    /// Deviation from the contract's "up to the cap": a found kind the player did not pack is
    /// salvaged as before. Shelving every row-used kind carried free consumables from run to
    /// run for any set rich in item rows — the FULL−D28 bot passed the D28 wall on 8 of 30
    /// seeds (bar ≤ 3) and COUNTERED's found potions crowded the kennel's leash off the
    /// shelf (D14 reach 57 % → 47 %, bar 50 %). Bounded by the pack, a found supply saves a
    /// purchase (the economy's leak) and never adds power the player did not buy.
    fn shelve_found(&mut self, all: Vec<Item>, run: &Run) -> (Vec<String>, Vec<Item>) {
        let used = self.lineage.row_kinds();
        let cat = self.supply_catalogue();
        let cap = self.lineage.supply_cap();
        let mut reserved: Vec<String> = self.lineage.last_supplies.iter().filter(|k| used.contains(*k) && !run.wasted_kinds.contains(k)).cloned().collect();
        for s in self.lineage.supplies.iter().filter(|s| !s.free) {
            if let Some(i) = reserved.iter().position(|k| *k == s.kind) {
                reserved.remove(i);
            }
        }
        let (facts, flavours) = (&self.lineage.facts, &self.lineage.flavours);
        let fits = |it: &Item| it.is_consumable() && !run.supplies.contains(&it.id) && !run.brought.contains(&it.id) && it.is_known(facts, flavours) && used.contains(&it.kind) && cat.iter().any(|e| e.kind == it.kind);
        let (cands, mut rest): (Vec<Item>, Vec<Item>) = all.into_iter().partition(|it| fits(it));
        let mut shelved = Vec::new();
        for mut it in cands {
            let take = match reserved.iter().position(|k| *k == it.kind) {
                Some(i) if self.lineage.supplies.len() < cap => {
                    reserved.remove(i);
                    true
                }
                _ => false,
            };
            if !take {
                rest.push(it);
                continue;
            }
            it.id = self.lineage.next_vault_id;
            self.lineage.next_vault_id += 1;
            it.found = true;
            it.known = true;
            it.free = false;
            let price = cat.iter().find(|e| e.kind == it.kind).map(|e| e.price).unwrap_or(0);
            let e = self.batch.shelved.entry(it.kind.clone()).or_insert((0, 0));
            e.0 += 1;
            e.1 += price;
            shelved.push(it.kind.clone());
            self.lineage.supplies.push(it);
        }
        rest.sort_by_key(|it| it.id);
        (shelved, rest)
    }

    fn salvage(&mut self, items: &[Item], pct: i32) -> Vec<i32> {
        let coins = salvage_coins(items, pct, self.lineage.gold_carry);
        self.salvage_paid(items, pct, &coins);
        coins
    }

    /// Salvage `items`, item `i` paying `coins[i]`; the hundredths it leaves (or overdraws) stay
    /// in the carry, so the lineage's salvage over time is the cents' sum whatever the split.
    fn salvage_paid(&mut self, items: &[Item], pct: i32, coins: &[i32]) {
        for (it, &gold) in items.iter().zip(coins) {
            // hundredths of a coin: value × tier% ÷ divisor, carried so cheap items still add up
            let cents = salvage_value(&it.kind) * pct / GOLD_DIVISOR;
            self.lineage.gold_carry += cents - gold * 100;
            self.lineage.gold_move(gold, "salvage");
            self.batch.salvage_gold += gold;
            let f = self.lineage.forge.entry(it.kind.clone()).or_default();
            f.salvaged += it.amount.max(1) as u32;
            f.settle();
            // The report's `salvaged` lines: what came home as gold. A death salvages at 0 %
            // (the forge still counts the kit; the bones hold it), so it has no line — `sword
            // ×2 · $0` beside `bones: 12 items on D7` read as a kit sold for nothing.
            if pct > 0 {
                let e = self.batch.salvaged.entry(it.kind.clone()).or_insert((0, 0));
                e.0 += 1;
                e.1 += cents;
                *self.batch.salvaged_coins.entry(it.kind.clone()).or_insert(0) += gold;
            }
        }
    }

    /// Finalise the exit's vault choice: chosen ids go to the vault, the rest are salvaged.
    pub fn keep(&mut self, ids: Vec<u32>) -> Result<(), String> {
        // Cut 30.5: a keep sheet answered by hand counts toward the keeper
        if self.pending_exit.as_ref().is_some_and(|p| keep_is_a_decision(p, &self.lineage.vault, self.lineage.vault_slots())) {
            crate::tree::did(&mut self.lineage, "keep");
        }
        self.keep_settle(ids)
    }

    fn keep_settle(&mut self, ids: Vec<u32>) -> Result<(), String> {
        let Some(p) = self.pending_exit.take() else { return Err("nothing to keep".into()) };
        let slots = self.lineage.vault_slots();
        // QA on 92eb880 (qaM: the sheet's `aggravate $2`, the report's `aggravate ×1 · $1`): an
        // unkept item pays exactly the coins the sheet showed for it (`ExitPending.worth`).
        let worth = salvage_coins(&p.items, p.pct, self.lineage.gold_carry);
        let mut paid: Vec<(Item, i32)> = Vec::new();
        let mut salvage: Vec<Item> = Vec::new();
        let mut settled: Vec<(String, &'static str)> = Vec::new();
        let mut done: Vec<(u32, String)> = Vec::new();
        for (it, coin) in p.items.into_iter().zip(worth) {
            let key = (it.id, it.kind.clone());
            let was_found = if done.contains(&key) { 0 } else { p.found.iter().filter(|f| **f == key).count() };
            done.push(key);
            // `bones_only`: nothing enters the vault; only bones piles carry gear.
            if !ids.contains(&it.id) || slots == 0 {
                for _ in 0..was_found {
                    settled.push((it.kind.clone(), "salvaged"));
                }
                paid.push((it, coin));
                continue;
            }
            let was_brought = p.brought.contains(&it.id);
            let mut v = it;
            if v.id < 100_000 {
                v.id = self.lineage.next_vault_id;
                self.lineage.next_vault_id += 1;
            }
            // Cut 6 §2: a vaulted item is known by name from here on — and its flavour with it
            // (QA on e0f87e7: `⊘ has: strength · identify strength` beside VAULT `strength potion`).
            v.known = true;
            if let Some(f) = crate::item::ident_fact(&self.lineage.flavours, &v.kind) {
                self.lineage.facts.insert(f);
            }
            self.lineage.vault.push(v.clone());
            self.lineage.vault.sort_by(|a, b| b.value().cmp(&a.value()).then(a.id.cmp(&b.id)));
            if self.lineage.vault.len() > slots {
                if let Some(d) = self.lineage.vault.pop() {
                    if d.id == v.id {
                        for _ in 0..was_found {
                            settled.push((d.kind.clone(), "salvaged"));
                        }
                        paid.push((d, coin));
                        continue;
                    }
                    salvage.push(d);
                }
            }
            for _ in 0..was_found {
                settled.push((v.kind.clone(), "kept"));
            }
            // QA on 0c6e126 (qaY: FOUND `mail +1`, which the send had brought from the vault): what
            // the send brought goes back to the vault — it is no find of this absence.
            if !was_brought {
                self.batch.found.push(v);
            }
        }
        // QA on 778fa1b (qaV): the exit line's `sheet` finds, settled.
        let settle = |line: &mut ExitLine, wire: &dyn Fn(&str) -> String| {
            for (kind, fate) in &settled {
                let k = wire(kind);
                if let Some(r) = line.found.iter_mut().find(|r| r.kind == k && r.fate == "sheet" && r.n > 0) {
                    r.n -= 1;
                }
                match line.found.iter_mut().find(|r| r.kind == k && r.fate == *fate) {
                    Some(r) => r.n += 1,
                    None => line.found.push(crate::wire::FoundRow { kind: k, n: 1, fate: fate.to_string() }),
                }
            }
            line.found.retain(|r| r.n > 0);
        };
        let wire = |k: &str| self.lineage.wire_name(k).replace('_', " ");
        if let Some(line) = self.last_exit.as_mut().filter(|l| l.run_id == p.run_id) {
            settle(line, &wire);
        }
        if let Some(line) = self.batch.exits.iter_mut().rev().find(|l| l.run_id == p.run_id) {
            settle(line, &wire);
        }
        let (items, coins): (Vec<Item>, Vec<i32>) = paid.into_iter().unzip();
        self.salvage_paid(&items, p.pct, &coins);
        self.salvage(&salvage, p.pct);
        if !p.stalled {
            self.restock_at(true);
        }
        self.lineage.kennel_leash();
        Ok(())
    }

    /// Resolve a pending exit by `keep_pref` (offline runs, or when the client moves on).
    /// Precedence (QA on 23ed91f: `keep armour` with a mail in the vault woke to `sword +2 ·
    /// bow +1`, the mail evicted by value and the vaulted scroll salvaged for a fallback sword):
    /// 1. what the send brought from the vault and came home goes back first, whatever the chip;
    /// 2. the chip's category (`best_weapon` / `best_armour`): its best find, which may replace
    ///    only a weaker vault item **of the same category**; `none` keeps no new find, the
    ///    `quartermaster` automation included;
    /// 3. the other category — with `quartermaster` like the chip's (same-category upgrades);
    ///    without it, only when no find of the chip's category came home, and only into a free
    ///    slot. Nothing the player vaulted is ever evicted for another category.
    pub fn auto_keep(&mut self) {
        let Some(p) = self.pending_exit.as_ref() else { return };
        let (ids, evict) = auto_keep_plan(p, &self.lineage.vault, self.lineage.vault_slots(), &self.lineage.keep_pref, true);
        if !evict.is_empty() {
            // The replaced vault items are salvaged with the exit's unkept finds.
            let out: Vec<Item> = self.lineage.vault.iter().filter(|v| evict.contains(&v.id)).cloned().collect();
            self.lineage.vault.retain(|v| !evict.contains(&v.id));
            if let Some(p) = self.pending_exit.as_mut() {
                p.items.extend(out);
            }
        }
        let _ = self.keep_settle(ids);
    }

    pub fn vocabulary(&self) -> Vocabulary {
        crate::tokens::vocabulary(&self.lineage)
    }

    /// Cut 23 §3: the vocabulary as the wire sends it — with the reasons' glosses
    /// (`Vocabulary.why_gloss`), which stored copies (a death's) do not carry.
    pub fn vocabulary_wire(&self) -> Vocabulary {
        let mut v = self.vocabulary();
        v.why_gloss = crate::turn::WHY_GLOSS.iter().map(|(k, g)| (k.to_string(), g.to_string())).collect();
        v
    }

    // ---- Companions (Addendum A)

    fn hatch_egg(&mut self, e: Egg) {
        let id = self.lineage.new_comp_id();
        let name = crate::descent::grudge_name(&mut self.lineage.rng);
        let def = crate::defs::monster_def(&e.kind);
        let rules = crate::probes::default_companion_rules(&e.tags, 1);
        self.lineage.kennel.push(Companion {
            id,
            kind: e.kind,
            name,
            level: 1,
            tags: e.tags,
            gen: e.gen,
            rules,
            max_rows: 2,
            hp: def.hp,
            max_hp: def.hp,
        });
    }

    /// Choose which owned companions go on the next expedition.
    pub fn set_party(&mut self, ids: Vec<u32>) -> Result<(), String> {
        self.set_party_by(ids, true)
    }

    /// `set_party`; `by_hand` counts toward the kennel-hand (a worker's fielding does not).
    pub fn set_party_by(&mut self, ids: Vec<u32>, by_hand: bool) -> Result<(), String> {
        if by_hand && !ids.is_empty() {
            crate::tree::did(&mut self.lineage, "field");
        }
        let slots = self.lineage.party_slots() as usize;
        let mut all: Vec<Companion> = std::mem::take(&mut self.lineage.party);
        all.append(&mut self.lineage.kennel);
        let mut party = Vec::new();
        for id in ids {
            if party.len() >= slots {
                break;
            }
            if let Some(i) = all.iter().position(|c| c.id == id) {
                party.push(all.remove(i));
            }
        }
        self.lineage.party = party;
        self.lineage.kennel = all;
        Ok(())
    }

    pub fn set_companion_rules(&mut self, id: u32, set: RuleSet) -> Result<(), String> {
        set.validate()?;
        let c = self
            .lineage
            .party
            .iter_mut()
            .chain(self.lineage.kennel.iter_mut())
            .find(|c| c.id == id)
            .ok_or("no such companion")?;
        let mut set = set;
        set.rows.truncate(c.max_rows);
        c.rules = set;
        Ok(())
    }

    /// Two level ≥ 2 companions become one egg: A's kind, A's tags plus one of B's.
    pub fn breed(&mut self, a: u32, b: u32) -> Result<(), String> {
        if a == b {
            return Err("same companion".into());
        }
        let find = |l: &LineageState, id: u32| l.all_companions().find(|c| c.id == id).cloned();
        let ca = find(&self.lineage, a).ok_or("no such companion")?;
        let cb = find(&self.lineage, b).ok_or("no such companion")?;
        if ca.level < 2 || cb.level < 2 {
            return Err("both must be level 2".into());
        }
        // Cut 2 §3: two tags carry unless `third_tag` is owned.
        let cap = if self.lineage.unlocks.contains("third_tag") { 3 } else { 2 };
        let mut tags = ca.tags.clone();
        tags.truncate(cap);
        if tags.len() < cap {
            if let Some(t) = cb.tags.iter().find(|t| !tags.contains(t)) {
                tags.push(t.clone());
            }
        }
        // Cut 3: a mirror shard in the vault breeds the `mirror` tag into the egg.
        if !tags.iter().any(|t| t == "mirror") {
            if let Some(i) = self.lineage.vault.iter().position(|v| v.kind == "mirror_shard") {
                self.lineage.vault.remove(i);
                tags.push("mirror".into());
            }
        }
        for id in [a, b] {
            self.lineage.party.retain(|c| c.id != id);
            self.lineage.kennel.retain(|c| c.id != id);
        }
        let eid = self.lineage.new_comp_id();
        let hatch_in = self.lineage.egg_rests();
        self.lineage.eggs.push(Egg { id: eid, kind: ca.kind.clone(), tags, gen: ca.gen.max(cb.gen) + 1, hatch_in, from_loss: false });
        self.lineage.eggs_laid += 1;
        self.lineage.bred.insert(ca.kind);
        Ok(())
    }

    /// Hatch a lost companion's egg now for 50 gold (Addendum B).
    pub fn hatch(&mut self, egg_id: u32) -> Result<(), String> {
        let i = self.lineage.eggs.iter().position(|e| e.id == egg_id).ok_or("no such egg")?;
        if !self.lineage.eggs[i].from_loss {
            return Err("bred eggs hatch after 3 rests".into());
        }
        if self.lineage.gold < 50 {
            return Err("50 gold needed".into());
        }
        self.lineage.gold_move(-50, "hatch");
        crate::tree::did(&mut self.lineage, "field");
        let e = self.lineage.eggs.remove(i);
        self.hatch_egg(e);
        Ok(())
    }

    pub fn companion_vocabulary(&self, id: u32) -> Result<Vocabulary, String> {
        let c = self.lineage.all_companions().find(|c| c.id == id).ok_or("no such companion")?;
        Ok(crate::tokens::companion_vocabulary(&self.lineage, c))
    }

    // ---- Gold and supplies (Addendum B, forge Addendum D)

    /// Cut 29 §4 (AW: a `throw fire` row with no fire in the pack, send after send): the kinds a
    /// `throw` row the player wrote (or a patch) names that the repeat does not carry — the shelf sells
    /// them, the last send did not waste them — each with the row that wants it (`throw fire`).
    /// The camp's tile offers them (`+ fire · for throw fire`, `Lineage.repeat_added`): a tap buys one
    /// (`buy_supply`) and the repeat keeps it from then on. Preset rows add nothing (the shipped sets pack what the player buys).
    pub fn repeat_adds(&self) -> Vec<crate::wire::RepeatAdd> {
        let l = &self.lineage;
        if l.restock_off {
            return Vec::new();
        }
        let cat = self.supply_catalogue();
        let mut out: Vec<crate::wire::RepeatAdd> = Vec::new();
        for (_, r) in l.rules().active(l.max_rows()) {
            // (a throw's kind only: a drink or a read the player packs by hand — the gate table's cohort
            // sets with a `drink heal` row packed heals every send and stopped dying, the return row's bar)
            if !matches!(r.origin.as_deref(), Some("player") | Some("patch")) || r.verb.v != "throw" {
                continue;
            }
            let k = r.verb.a.as_deref().unwrap_or("").split(',').next().unwrap_or("").to_string();
            if k.is_empty() || k == "unknown" || l.last_supplies.contains(&k) || l.last_wasted.contains(&k) || out.iter().any(|a| a.kind == k) || !cat.iter().any(|e| e.kind == k) {
                continue;
            }
            out.push(crate::wire::RepeatAdd { row: r.verb.short(), kind: k });
        }
        out
    }

    pub fn supply_catalogue(&self) -> Vec<SupplyInfo> {
        // QA on 912e135 (qaW: the kennel's free leash, dropped once, came back only at $30): while nothing is tamed the kennel's
        // leash is free to take back off the shelf.
        let kennel = self.lineage.kennel_declined && self.lineage.tamed_kinds() == 0;
        let mut out = vec![SupplyInfo { kind: "leash".into(), price: if kennel { 0 } else { 30 }, label: "leash".into() }];
        for d in crate::defs::ITEMS {
            let base = match d.cat {
                Cat::Potion | Cat::Scroll => Some(supply_price(d.cat, self.lineage.best_depth)),
                _ => None,
            };
            let identified = crate::item::is_identified(&self.lineage.facts, &self.lineage.flavours, d.kind);
            let craftable = self.lineage.forge.get(d.kind).is_some_and(|f| f.craftable);
            // Cut 13 §3: the shop grows with the night — a potion or scroll the forge can craft
            // is for sale at its base price whether or not the hero has drunk one (rater R:
            // "$2131 after the night had almost nothing to buy"); it comes at the forge's tier.
            let price = match (base, craftable, identified) {
                (Some(p), _, true) | (Some(p), true, _) => p,
                (None, true, _) if d.cat != Cat::Gold && d.kind != "leash" => 2 * salvage_value(d.kind),
                _ => continue,
            };
            let label = match d.cat {
                Cat::Potion => format!("{} potion", d.kind.replace('_', " ")),
                Cat::Scroll => format!("{} scroll", d.kind.replace('_', " ")),
                _ => d.kind.to_string(),
            };
            out.push(SupplyInfo { kind: d.kind.into(), price, label });
        }
        out
    }

    pub fn buy_supply(&mut self, kind: &str) -> Result<(), String> {
        self.buy_supply_as(kind, &kind.replace('_', " "))
    }

    /// Cut 19 §3: `buy_supply` with the ledger line's text (`repeat heal` for the repeat).
    pub fn buy_supply_as(&mut self, kind: &str, why: &str) -> Result<(), String> {
        self.buy_supply_priced(kind, why, None)
    }

    /// `buy_supply_as` at `price` (a repeat's quote, never above the shelf's price today) when
    /// given, else the shelf's.
    pub fn buy_supply_priced(&mut self, kind: &str, why: &str, price: Option<i32>) -> Result<(), String> {
        if self.lineage.supplies.len() >= self.lineage.supply_cap() {
            return Err(format!("{} supplies max", self.lineage.supply_cap()));
        }
        let mut entry = self.supply_catalogue().into_iter().find(|s| s.kind == kind).ok_or("not for sale")?;
        if let Some(p) = price {
            entry.price = entry.price.min(p);
        }
        if self.lineage.gold < entry.price {
            return Err("not enough gold".into());
        }
        self.lineage.gold_move_n(-entry.price, why, 1);
        let id = self.lineage.next_vault_id;
        self.lineage.next_vault_id += 1;
        let mut it = Item::new(id, kind);
        it.enchant = self.lineage.forge_tier(kind);
        // Cut 6 §2: bought (or forge-crafted) by name: usable as such. And the lineage now
        // knows the kind (rater W on 238bd67: the shop sold `heal potion` off the forge while
        // `drink heal` read `unknown item` and 28 found heals were salvaged unknown).
        it.known = true;
        if let Some(f) = crate::item::ident_fact(&self.lineage.flavours, kind) {
            if self.lineage.facts.insert(f.clone()) {
                self.events.push(Ev::Fact { t: 0, fact: f });
            }
        }
        if kind == "leash" {
            it.amount = 1;
            // (the kennel's leash taken back: free, as it was)
            it.free = entry.price == 0 && self.lineage.kennel_declined;
            self.lineage.kennel_declined = false;
        }
        it.paid = entry.price;
        self.lineage.supplies.push(it);
        Ok(())
    }

    /// Insure a vault item against loss on death: 25% of its salvage value ×10.
    pub fn insure(&mut self, id: u32) -> Result<(), String> {
        let it = self.lineage.vault.iter().find(|v| v.id == id).ok_or("not in the vault")?;
        if self.lineage.insured.contains(&id) {
            return Err("already insured".into());
        }
        let cost = insure_cost(&it.kind);
        if self.lineage.gold < cost {
            return Err("not enough gold".into());
        }
        let why = format!("insure {}", crate::item::kind_name(&it.kind));
        self.lineage.gold_move(-cost, &why);
        self.lineage.insured.push(id);
        Ok(())
    }

    /// QA on 308f045 (qaAC: `home armour` answered `vault full · axe stays` and the vault offered
    /// no way to take the axe out): an item out of the vault — salvaged at a bank's share, as a
    /// shelved find is (`salvage <kind>` on the ledger); off the loadout and its insurance too.
    pub fn sell_vault(&mut self, id: u32) -> Result<i32, String> {
        let i = self.lineage.vault.iter().position(|v| v.id == id).ok_or("not in the vault")?;
        let it = self.lineage.vault.remove(i);
        self.lineage.insured.retain(|x| *x != id);
        self.loadout.retain(|x| *x != id);
        let coins = self.salvage(std::slice::from_ref(&it), 100);
        crate::tree::did(&mut self.lineage, "keep");
        Ok(coins.iter().sum())
    }

    /// Cut 12 §6: one line off the shelf (rater O: the header's `×` cleared the whole list —
    /// "I lost the leash"). A bought supply is refunded; the kennel's leash (free) is simply
    /// put back — it returns at the next exit while the lineage has never tamed.
    pub fn drop_supply(&mut self, id: u32) -> Result<(), String> {
        let i = self.lineage.supplies.iter().position(|s| s.id == id).ok_or("not on the shelf")?;
        let s = self.lineage.supplies.remove(i);
        if s.free && s.kind == "leash" {
            self.lineage.kennel_declined = true;
        }
        if s.found {
            // Cut 21 §2: a found supply off the shelf is salvaged, as the exit would have.
            self.salvage(std::slice::from_ref(&s), 100);
        } else if !s.free {
            if let Some(price) = refund_of(&s, &self.supply_catalogue()) {
                let why = format!("refund {}", crate::item::kind_name(&s.kind));
                self.lineage.gold_move_n(price, &why, 1);
            }
            let slot = self.lineage.last_supplies.iter().enumerate().find(|(i, k)| **k == s.kind && self.lineage.last_supply_origins.get(*i).is_some_and(|(kind, auto)| kind == *k && *auto == s.auto_packed)).map(|(i, _)| i)
                .or_else(|| self.lineage.last_supplies.iter().position(|k| *k == s.kind));
            if let Some(k) = slot {
                self.lineage.last_supplies.remove(k);
                if k < self.lineage.last_supply_origins.len() { self.lineage.last_supply_origins.remove(k); }
            }
        }
        Ok(())
    }

    pub fn clear_supplies(&mut self) {
        let cat = self.supply_catalogue();
        for s in std::mem::take(&mut self.lineage.supplies) {
            if s.free {
                continue;
            }
            if s.found {
                self.salvage(std::slice::from_ref(&s), 100);
                continue;
            }
            if let Some(price) = refund_of(&s, &cat) {
                let why = format!("refund {}", crate::item::kind_name(&s.kind));
                self.lineage.gold_move_n(price, &why, 1);
            }
        }
        // Cut 4: clearing the shelf is an order; the automation does not undo it.
        self.lineage.last_supplies.clear();
        self.lineage.last_supply_origins.clear();
    }

    /// The loadout repeats (Cut 19 §3; was the `auto_supply` automation, Cut 2 §3): an empty
    /// shelf is re-packed with the last expedition's kinds at the shelf's price (a `repeat
    /// <kind>` ledger line each) as far as gold allows, unless the player cleared the repeat
    /// (`set_restock(false)`). Cut 4: runs when the hero comes home (`finish_run`, and again
    /// after the vault decision brought the salvage in), so the camp's shelf shows the restock
    /// before the next send — restocking only at `start_run` moved the supplies straight into
    /// the pack and the shelf never showed them. Cut 19 §3 (rater AB: `−$1280 spent` by the
    /// restock overnight): an absence's restock never spends more than the absence brought
    /// home (`Batch::income`); a kind it could not afford under that cap sets
    /// `Batch.restock_capped` (the report's `restock capped`).
    ///
    /// QA on 1a2a4a9 (qaP): the re-pack tops the shelf up to the last send's kinds — what came
    /// back unused stays and is not bought again, what was used is — so it never charges more
    /// than the badge (`repeat_plan`, the whole pack's price).
    pub fn restock(&mut self) -> Vec<String> {
        self.restock_at(false)
    }

    /// `restock` — at an exit (`quoted`) each kind at the price its send quoted
    /// (`LineageState::repeat_quote`: the badge the camp showed; QA on 778fa1b), else at the
    /// shelf's price today (the send's own re-pack, the camp's `set_restock(true)`).
    pub fn restock_at(&mut self, quoted: bool) -> Vec<String> {
        let mut bought = Vec::new();
        // Cut 30 §1: the quartermaster packs a drill's item first (`throw fire` at the Mother, the
        // silence scroll at the Queen) — its slot is reserved before any other supply, the repeat on
        // or off.
        let qm = crate::packages::quartermaster(&self.lineage);
        let cap = self.lineage.supply_cap();
        let want = automatic_pack_plan(&self.lineage);
        for (i, kind) in want.iter().enumerate() {
            let drill = i < qm.len();
            if drill && self.lineage.supplies.iter().any(|s| s.kind == *kind) {
                continue;
            }
            if !drill && self.lineage.supplies.iter().filter(|s| s.kind == *kind).count() >= want[qm.len()..].iter().filter(|k| *k == kind).count() {
                continue;
            }
            // (an absence's fill spends no more than the absence brought home, as the repeat)
            if !drill && self.offline {
                let price = self.supply_catalogue().iter().find(|e| e.kind == *kind).map(|e| e.price).unwrap_or(0);
                if price > self.batch.income() - self.batch.spent_total() {
                    self.batch.restock_capped = true;
                    continue;
                }
            }
            if !drill && self.lineage.supplies.len() >= cap {
                // A former package's unused automatic supplies must not crowd out the current
                // stance's sustain. Player purchases and found supplies retain their places.
                // Check the replacement can be bought before giving up an existing supply.
                let Some(price) = self.supply_catalogue().iter().find(|e| e.kind == *kind).map(|e| e.price) else { continue };
                if self.lineage.gold < price { continue; }
                let surplus = self.lineage.supplies.iter().rposition(|s| {
                    s.auto_packed && !s.free && !s.found
                        && self.lineage.supplies.iter().filter(|t| t.kind == s.kind).count()
                            > want.iter().filter(|k| **k == s.kind).count()
                });
                let Some(i) = surplus else { continue };
                self.lineage.supplies.remove(i);
            }
            if self.lineage.supplies.len() >= self.lineage.supply_cap() {
                if let Some(i) = self.lineage.supplies.iter().rposition(|s| !qm.contains(&s.kind)) {
                    self.lineage.supplies.remove(i);
                }
            }
            let gold = self.lineage.gold;
            let why = format!("{} {}", if drill { "drill" } else { "pack" }, kind.replace('_', " "));
            if self.buy_supply_as(kind, &why).is_ok() {
                if let Some(item) = self.lineage.supplies.last_mut() { item.auto_packed = true; }
                bought.push(kind.replace('_', " "));
                let e = self.batch.spent.entry(kind.clone()).or_insert((0, 0));
                e.0 += 1;
                e.1 += gold - self.lineage.gold;
                self.batch.drill_packs += drill as u32;
            }
        }
        if self.lineage.restock_off {
            return bought;
        }
        let plan: Vec<(String, bool)> = self.lineage.last_supplies.iter().enumerate().map(|(i, kind)| (kind.clone(), self.lineage.last_supply_origins.get(i).is_some_and(|(k, auto)| k == kind && *auto))).collect();
        let plan = current_repeat_plan(&self.lineage, plan);
        let on_shelf: Vec<(String, bool)> = self.lineage.supplies.iter().filter(|s| !s.free).map(|s| (s.kind.clone(), s.auto_packed)).collect();
        let supplied = supplied_slots(&plan, &on_shelf);
        let mut short = Vec::new();
        // Cut 13 §3: a kind the last run used to no effect is not rebought (rater R: "the
        // strength potion the trait drinks at full HP is rebought sixteen times").
        let wasted = self.lineage.last_wasted.clone();
        // Cut 21 §2 (AE: a strength potion no row drinks re-bought sixteen times): only the
        // kinds a row of the active set can use are re-bought (`LineageState::row_kinds`).
        let used = self.lineage.row_kinds();
        let mut skip = self.lineage.theft_skip.clone();
        for (slot, (kind, automatic)) in plan.iter().cloned().enumerate() {
            if supplied[slot] { continue; }
            if wasted.contains(&kind) || !used.contains(&kind) {
                continue;
            }
            if let Some(i) = skip.iter().position(|k| *k == kind) {
                skip.remove(i);
                continue;
            }
            // (QA on 0c6e126: the send's own re-pack pays the quote too — the badge's `≤$N` is the most any re-pack charges)
            let _ = quoted;
            let quote = self.lineage.repeat_quote.get(&kind).copied();
            if self.offline {
                let price = quote.or_else(|| self.supply_catalogue().iter().find(|e| e.kind == kind).map(|e| e.price)).unwrap_or(0);
                if price > self.batch.income() - self.batch.spent_total() {
                    self.batch.restock_capped = true;
                    continue;
                }
            }
            let gold = self.lineage.gold;
            match self.buy_supply_priced(&kind, &format!("repeat {}", kind.replace('_', " ")), quote) {
                Ok(()) => {
                    if let Some(item) = self.lineage.supplies.last_mut() { item.auto_packed = automatic; }
                    bought.push(kind.replace('_', " "));
                    let e = self.batch.spent.entry(kind.clone()).or_insert((0, 0));
                    e.0 += 1;
                    e.1 += gold - self.lineage.gold;
                }
                // QA on 1a2a4a9: short of gold — the rest is still bought, and it says so.
                Err(e) if e == "not enough gold" => short.push(kind.clone()),
                Err(_) => {}
            }
        }
        if !short.is_empty() {
            self.lineage.gold_move(0, REPEAT_SHORT);
            self.batch.repeat_short = true;
        }
        self.lineage.repeat_short = short;
        bought
    }
}

/// QA on 778fa1b (qaV: FOUND `dagger ×2` beside SALVAGED `dagger ×1`, a `leather +1` in no
/// named place): where each unit the run found ended — the ways it left the pack on the way
/// (`Run.found_gone`: used, left, stolen), then its item's place at the exit (`fate`, by id:
/// kept on the sheet, salvaged, shelved, bones); a stack's units not in it at the exit were
/// spent (`used`: a leash on a tame, chalk on a floor); any other unit is `lost` (none should be).
/// Run-clear: the finds an exit line shows — kept, on the keep sheet or shelved on a bank or a return, left
/// in the bones on a death (never what was salvaged) — rarest first, then dearest, ≤ `wire::FINDS_SHOWN`.
pub fn exit_finds(found: &[Item], fate: &BTreeMap<(u32, String), &'static str>, facts: &BTreeSet<String>, flavours: &Flavours) -> Vec<InvItem> {
    let mut out: Vec<(crate::item::Rarity, i32, InvItem)> = found
        .iter()
        .filter(|i| matches!(fate.get(&(i.id, i.kind.clone())).copied(), Some("sheet" | "shelved" | "bones")))
        .map(|i| {
            let w = to_inv(i, facts, flavours);
            (w.rarity, i.value(), w)
        })
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)).then(a.2.id.cmp(&b.2.id)));
    out.into_iter().take(crate::wire::FINDS_SHOWN).map(|x| x.2).collect()
}

pub fn found_rows(run: &Run, pack: &BTreeMap<(u32, String), i32>, fate: &BTreeMap<(u32, String), &'static str>, wire: impl Fn(&str) -> String) -> Vec<crate::wire::FoundRow> {
    let mut rows: Vec<crate::wire::FoundRow> = Vec::new();
    let mut add = |kind: &str, fate: &str, n: u32| {
        if n == 0 {
            return;
        }
        let k = wire(kind);
        match rows.iter_mut().find(|r| r.kind == k && r.fate == fate) {
            Some(r) => r.n += n,
            None => rows.push(crate::wire::FoundRow { kind: k, n, fate: fate.into() }),
        }
    };
    let mut keys: Vec<(u32, String)> = Vec::new();
    for key in &run.found_units {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    for key in keys {
        let (id, kind) = (key.0, key.1.as_str());
        let found = run.found_units.iter().filter(|x| **x == key).count() as u32;
        let mut gone = 0u32;
        for (_, _, f) in run.found_gone.iter().filter(|(g, k, _)| *g == id && k == kind) {
            add(kind, f, 1);
            gone += 1;
        }
        let left = found.saturating_sub(gone);
        let stack = crate::defs::item_def(kind).cat == Cat::Misc;
        let in_pack = match fate.get(&key) {
            Some(f) => {
                let n = found_in_pack(run, pack, &key);
                add(kind, f, n);
                n
            }
            None => 0,
        };
        add(kind, if stack { "used" } else { "lost" }, left - in_pack);
    }
    rows
}

/// The found units of `key` (id, kind) in the pack at the exit: a stack's up to its amount
/// (packed units are spent first), an item's one (items sharing a bones find's id and kind
/// count once).
pub fn found_in_pack(run: &Run, pack: &BTreeMap<(u32, String), i32>, key: &(u32, String)) -> u32 {
    let Some(a) = pack.get(key) else { return 0 };
    let left = run.found_left(key.0, &key.1) as u32;
    if crate::defs::item_def(&key.1).cat == Cat::Misc {
        left.min((*a).max(1) as u32)
    } else {
        left.min(1)
    }
}

/// Cut 6 §1: the exit's ledger line, ≤ 14 words (`$84 carried · return keeps 60% → $50`;
/// death: `$144 carried · death keeps 0% → $0 · bones: 7 items on D5`; a run that hit the cap:
/// `lost thread keeps 0%`).
/// Cut 9 §5: the last `EXIT_TRACE_LEN` hero turns of a run, from its trace ring. Cut 11 §3:
/// plus the run's provenance log (every `because` event), when it has one.
pub fn exit_trace(run: &Run, prov: &[crate::provenance::Prov]) -> Trace {
    let turns: Vec<crate::wire::TraceTurn> = run.trace.iter().rev().take(EXIT_TRACE_LEN).rev().map(|t| (**t).clone()).collect();
    Trace { max_steps: max_steps_in(run, &turns), turns, provenance: crate::provenance::all(prov), blow: death_blow(run), blows: death_blows(run), hp_lost: hp_lost(run), hp_healed: hp_healed(run) }
}

/// Cut 28 §2 (AV: `R1 hp not <30%` at 6 hp, his max drained unseen): the run's max-hp steps from the
/// trace's first turn on — every step when the whole run's are few (≤ 6), so a drain that began
/// floors ago still reads (`44 → 29`).
pub fn max_steps_in(run: &Run, turns: &[crate::wire::TraceTurn]) -> Vec<crate::wire::MaxStep> {
    if run.max_steps.len() <= 6 {
        return (*run.max_steps).clone();
    }
    let from = turns.first().map_or(0, |t| t.t);
    let mut out: Vec<crate::wire::MaxStep> = run.max_steps.iter().filter(|s| s.t >= from).cloned().collect();
    // (the drain before the window, folded into one step: where the max stood at its first turn)
    let before: Vec<&crate::wire::MaxStep> = run.max_steps.iter().filter(|s| s.t < from).collect();
    if let Some(last) = before.last() {
        let delta: i32 = before.iter().map(|s| s.delta).sum();
        out.insert(0, crate::wire::MaxStep { t: last.t, max: last.max, delta, cause: last.cause.clone() });
    }
    out
}

/// QA on 524827b (qaAA): a death's hp lost since full, per cause, most first (`Trace.hp_lost`);
/// empty on any other exit.
pub fn hp_lost(run: &Run) -> Vec<crate::wire::HpLoss> {
    if run.over != Some(ExitTier::Death) {
        return Vec::new();
    }
    let mut v: Vec<crate::wire::HpLoss> = run.hp_lost.iter().map(|(by, dmg)| crate::wire::HpLoss { by: by.clone(), dmg: *dmg }).collect();
    v.sort_by(|a, b| b.dmg.cmp(&a.dmg).then(a.by.cmp(&b.by)));
    v
}

/// QA on 308f045 (qaAC: `since full hp · warlord −26 · fire −10 · … · others −18` = 72 on a 36-hp
/// hero): the hp healed since the count began — the losses beyond the hp he started it with
/// (`Trace.hp_healed`); 0 on any other exit.
pub fn hp_healed(run: &Run) -> i32 {
    if run.over != Some(ExitTier::Death) || run.hp_lost.is_empty() {
        return 0;
    }
    let lost: i32 = run.hp_lost.iter().map(|(_, d)| *d).sum();
    (lost - run.hp_lost_from.max(0)).max(0)
}

/// QA on 0c6e126 (qaY): a death's killing blow, the trace's last row (`Trace.blow`); none on
/// any other exit.
pub fn death_blow(run: &Run) -> Option<crate::wire::TraceBlow> {
    if run.over != Some(ExitTier::Death) {
        return None;
    }
    let by = run.death_cause.clone()?;
    Some(crate::wire::TraceBlow { t: run.death_t.unwrap_or(run.turn), by, dmg: run.death_blow.max(1), hp: 0 })
}

/// Cut 25 §3: a death's blows after the last action, oldest first, the killing one last (`Trace.blows`);
/// empty when there was only the one (`Trace.blow` says it).
pub fn death_blows(run: &Run) -> Vec<crate::wire::TraceBlow> {
    if run.over != Some(ExitTier::Death) || run.blows.len() < 2 {
        return Vec::new();
    }
    let mut out = run.blows.clone();
    if let (Some(last), Some(b)) = (out.last_mut(), death_blow(run)) {
        *last = b;
    }
    out
}

#[allow(clippy::too_many_arguments)]
pub fn exit_line(carried: i32, keep_pct: i32, kept: i32, spent: i32, spent_on: Vec<String>, tier: ExitTier, timed_out: bool, bones: usize, depth: u32) -> ExitLine {
    exit_line_of(carried, keep_pct, kept, spent, spent_on, tier, timed_out, false, 0, bones, depth)
}

/// c30-legible: why a run ended, ≤ 3 words (`ExitLine.reason`) — the committing row's reason, not its
/// words: a bank at the record (Steady's `depth ≥ best → bank`, which ends a fresh lineage's first runs
/// at D2, D3, D4 …) reads `banks every record`; a hurt row `hurt · went home`; a death `slain · jackal`.
pub fn exit_reason(run: &Run, tier: ExitTier, best0: u32, rules: &RuleSet) -> String {
    let bare = |s: String| -> String { s.trim_start_matches("the ").trim_start_matches("a ").trim_start_matches("an ").to_string() };
    if tier == ExitTier::Death {
        let c = run.death_cause.as_deref().unwrap_or("");
        if c == "hunger" {
            return "starved".into();
        }
        let who = bare(crate::sifter::cause_phrase(c));
        return if who.is_empty() { "slain".into() } else { format!("slain · {who}") };
    }
    if run.timed_out && run.stuck_fires >= STALL_FIRES {
        return "stuck · gave up".into();
    }
    if run.timed_out {
        return "lost thread".into();
    }
    if let Some(k) = &run.driven_off {
        return format!("repelled · {}", crate::sifter::boss_short(k));
    }
    let row = run.exit_row.and_then(|i| usize::try_from(i).ok()).and_then(|i| rules.rows.get(i));
    let Some(row) = row else {
        return match tier {
            ExitTier::Bank => "recalled".into(),
            _ if run.stuck_fires > 0 => "stuck · went home".into(),
            _ => "bailed".into(),
        };
    };
    let has = |k: &str| row.conds.iter().any(|c| c.k == k);
    let bank = tier == ExitTier::Bank;
    // (Cut 30.5: out of supplies — the heals spent — is its own reason)
    if has("lacks") {
        return if bank { "no heals · banked".into() } else { "no heals · went home".into() };
    }
    if has("hp<") || has("party_hp<") {
        return if bank { "hurt · banked".into() } else { "hurt · went home".into() };
    }
    if has("depth>=") {
        // (a stance's row banks at the record: `depth ≥ best + 1`, a floor further when whole)
        let at = row.conds.iter().find(|c| c.k == "depth>=").and_then(|c| c.n).unwrap_or(0);
        // (Cut 30.5: a row a floor further than the record's next — the whole hero's — reads `new best · banked`)
        return if run.max_depth > best0 && at as u32 <= (best0 + 1).max(2) { "banks every record".into() } else if run.max_depth > best0 { "new best · banked".into() } else { format!("reached D{}", run.max_depth) };
    }
    if has("foes>=") || has("adj>=") {
        return "outnumbered".into();
    }
    if has("loot>=") {
        return "loot full".into();
    }
    if has("turns>") {
        return "long run".into();
    }
    format!("{} rule", if bank { "bank" } else { "return" })
}

/// A ledger line an exit wrote (`returned D5`, `banked D8`, `died D3`, `lost thread D3`,
/// `stalled D2`, `driven D8`): the word it leads with.
pub fn is_exit_why(why: &str) -> bool {
    ["returned", "banked", "died", "lost", "stalled", "driven"].iter().any(|w| why.starts_with(w))
}

/// `exit_line` with the two things a stalled or supplied run has to say: `stalled` (the run
/// shuffled on one floor; `lost thread` is the turn cap) and `N supplies unused` (a supply
/// packed at camp is spent by the send whether it was used or not — QA on 952e306: "the $40
/// heal potion is gone (never drunk)").
#[allow(clippy::too_many_arguments)]
pub fn exit_line_of(carried: i32, keep_pct: i32, kept: i32, spent: i32, spent_on: Vec<String>, tier: ExitTier, timed_out: bool, stalled: bool, unused: usize, bones: usize, depth: u32) -> ExitLine {
    // Cut 10 §3: the line leads with the verb and what came home (`returned $50 · $84
    // carried · keeps 60%`; the report's exit lines read `returned $61`, not `$61`), the
    // arithmetic after it; a run that timed out says so at the end.
    // QA on 912e135 (qaW: `returned $0 · $224 carried · keeps 0% · stalled` under the tiles `0
    // RETURNED · 1 STALLED $224 lost`): a timed-out run leads with its own word — `stalled` or
    // `lost thread`, the ledger's — never `returned`; and an exit that kept nothing says what it
    // lost (`died $0 · $157 lost`: `$157 carried · keeps 0%` put two gold figures on one line
    // with no verb for the second, and `keeps 0%` on every death line explained nothing).
    let verb = match tier {
        _ if timed_out && stalled => "stalled",
        _ if timed_out => "lost thread",
        ExitTier::Bank => "banked",
        ExitTier::Return => "returned",
        ExitTier::Death => "died",
    };
    let mut text = if kept <= 0 && keep_pct <= 0 { format!("{verb} $0 · ${carried} lost") } else { format!("{verb} ${kept} · ${carried} carried · keeps {keep_pct}%") };
    // (Cut 30.5: the checkpoints' secured gold — the caller re-words the line: `secured_text`)
    if bones > 0 {
        text.push_str(&format!(" · bones: {bones} items on D{depth}"));
    }
    if unused > 0 && tier != ExitTier::Death {
        text.push_str(&format!(" · {unused} {} back", if unused == 1 { "supply" } else { "supplies" }));
    }
    ExitLine { packages:Vec::new(),bloodline_id:1,secured: 0, carried, keep_pct, kept, spent, spent_on, text, trace: None, salvaged: Vec::new(), run_id: 0, legacy_earned: 0, xp: 0, level_ups: 0, stolen: Vec::new(), purse_full: false, shelved: Vec::new(), toll: 0, start: 1, start_short: None, stolen_gold: 0, swapped: 0, cause: None, reason: None, swap_left: Vec::new(), wake: 0, found: Vec::new(), found_n: 0, bones: Vec::new(), driven: None, news: Vec::new(), meters: None, end: String::new(), reached: 0, new_best: false, finds: Vec::new() }
}

/// Cut 30.5: an exit line's head re-worded with the secured gold — `banked $120 · $80 secured + 100% of $40`, `died
/// $80 · $41 lost` (a death keeps the secured gold alone) — its tail (bones, supplies back) kept.
pub fn secured_text(text: &str, kept: i32, secured: i32, rest: i32, pct: i32) -> String {
    let verb = ["stalled", "lost thread", "banked", "returned", "died"].iter().find(|w| text.starts_with(**w)).copied().unwrap_or("returned");
    let tail: Vec<&str> = text.split(" · ").filter(|s| s.starts_with("bones: ") || s.ends_with(" back")).collect();
    let head = if pct <= 0 { format!("{verb} ${kept} · ${rest} lost") } else { format!("{verb} ${kept} · ${secured} secured + {pct}% of ${rest}") };
    std::iter::once(head).chain(tail.into_iter().map(String::from)).collect::<Vec<_>>().join(" · ")
}

/// A trophy's id as the report reads it (QA on 952e306: "`trophy: home:10`, `trophy:
/// studied:5` — no screen explains trophies"): `studied:5` → `5 studied`, `home:10` → `10
/// homecomings`, `slain:100` → `100 slain`, `bones:1` → `first bones`, `ledger:warrens` →
/// `warrens ledger`, a run trophy's snake case as words (`no heal D5`).
pub fn trophy_label(id: &str) -> String {
    match id.split_once(':') {
        Some(("studied", n)) => format!("{n} studied"),
        Some(("home", n)) => format!("{n} homecomings"),
        Some(("slain", n)) => format!("{n} slain"),
        Some(("bones", _)) => "first bones".into(),
        Some(("ledger", biome)) => format!("{biome} ledger"),
        Some((a, b)) => format!("{a} {b}").replace('_', " "),
        // QA on 524827b (qaAB: `Trophy: pacifist floor` — "unexplained"): what it is for
        None if id == "pacifist_floor" => "no-kill floor".into(),
        None => id.replace('_', " "),
    }
}

/// Salvage value per kind (Addendum D).
/// QA on 92eb880: the coins each of `items` pays when they are salvaged together at `pct` with
/// `carry` hundredths already banked — the whole's floor (`(carry + Σ cents) ÷ 100`, as the
/// ledger always paid) apportioned by largest remainder (ties to the earlier item). The exit
/// sheet's `worth`, the ledger and the report's rows all read these.
pub fn salvage_coins(items: &[Item], pct: i32, carry: i32) -> Vec<i32> {
    // A death's kit salvages at 0 %: no coins, whatever the carry holds.
    if pct <= 0 {
        return vec![0; items.len()];
    }
    let cents: Vec<i32> = items.iter().map(|i| salvage_value(&i.kind) * pct / GOLD_DIVISOR).collect();
    apportion_cents(&cents, carry)
}

/// `cents` (hundredths) to whole coins summing to `(carry + Σ cents) ÷ 100` (never below 0):
/// each floors, the remainder goes to the largest fractions (ties to the earlier), an
/// overdrawn carry takes from the smallest.
pub fn apportion_cents(cents: &[i32], carry: i32) -> Vec<i32> {
    let total = (carry + cents.iter().sum::<i32>()).max(0) / 100;
    let mut out: Vec<i32> = cents.iter().map(|c| c.max(&0) / 100).collect();
    if out.is_empty() {
        return out;
    }
    let mut order: Vec<usize> = (0..cents.len()).collect();
    order.sort_by(|&a, &b| (cents[b].max(0) % 100).cmp(&(cents[a].max(0) % 100)).then(a.cmp(&b)));
    let mut short = total - out.iter().sum::<i32>();
    while short > 0 {
        for &i in &order {
            if short == 0 {
                break;
            }
            out[i] += 1;
            short -= 1;
        }
    }
    while short < 0 {
        let before = short;
        for &i in order.iter().rev() {
            if short == 0 {
                break;
            }
            if out[i] > 0 {
                out[i] -= 1;
                short += 1;
            }
        }
        if short == before {
            break;
        }
    }
    out
}

pub fn salvage_value(kind: &str) -> i32 {
    let d = crate::defs::item_def(kind);
    match d.cat {
        Cat::Weapon | Cat::Armour => d.value,
        Cat::Potion => 8,
        Cat::Scroll => 12,
        Cat::Misc => 5,
        Cat::Gold => 0,
    }
}

/// Cut 22 §1 (AG: "+$77 returned · −$160 spent"; AH: "a treadmill"): a potion on the shelf
/// costs `$10 + 2 × best depth` (D8 $26, D20 $50) and a scroll 1.5× that — a D8 lineage's heal
/// is not a D20 lineage's (was a flat $40 / $60).
/// The most a potion costs (a scroll 1.5×).
pub const SUPPLY_PRICE_CAP: i32 = 26;

pub fn supply_price(cat: Cat, best_depth: u32) -> i32 {
    // capped: a deep lineage starting at a waystone skips the shallow income, so a price that
    // kept climbing with depth made its sends lose money (the waystone column, Cut 22 §1)
    let potion = (10 + 2 * best_depth as i32).min(SUPPLY_PRICE_CAP);
    match cat {
        Cat::Scroll => potion * 3 / 2,
        _ => potion,
    }
}

/// Cut 22 §1: a bought supply's refund — what was paid for it (`Item.paid`), else the shelf's
/// price today (a save from before the price moved); None when the shelf does not sell it.
fn refund_of(s: &Item, cat: &[SupplyInfo]) -> Option<i32> {
    if s.paid > 0 {
        return Some(s.paid);
    }
    cat.iter().find(|e| e.kind == s.kind).map(|e| e.price)
}

/// Insurance premium: 25% of salvage value ×10 (Melvor style).
pub fn insure_cost(kind: &str) -> i32 {
    salvage_value(kind) * 10 / 4
}

/// Renown value of a kill (Addendum D): scaled by the monster's toughness.
pub fn kill_value(kind: &str) -> u32 {
    let d = crate::defs::monster_def(kind);
    (d.hp as u32) / 3 + 1
}

/// Fill a freshly generated floor with monsters and items for the run's depth.
pub fn populate_floor(run: &mut Run, grudges: &[Grudge], forge: &BTreeMap<String, ForgeRow>, hunter: Option<&Grudge>) {
    let depth = run.depth;
    let biome = run.biome();
    let mut open = run.floor.open_tiles();
    let hero = run.hero.pos;
    open.retain(|p| p.cheb(hero) > 6);
    if open.is_empty() {
        open = run.floor.open_tiles();
    }
    run.rng.shuffle(&mut open);
    let mut cursor = 0usize;
    let take = |rng: &mut Rng, open: &Vec<Pos>, cursor: &mut usize| -> Pos {
        if *cursor < open.len() {
            let p = open[*cursor];
            *cursor += 1;
            p
        } else {
            *rng.pick(open)
        }
    };
    let table = spawn_table(biome, depth);
    let weights: Vec<u32> = table.iter().map(|t| t.1).collect();
    let groups = crate::defs::group_budget(depth);
    let water = run.floor.water_tiles();
    let mut captive_placed = false;
    for _ in 0..groups {
        let (kind, _, gmin, gmax) = table[run.rng.weighted(&weights)];
        if kind == "captive" {
            if captive_placed {
                continue;
            }
            captive_placed = true;
        }
        let n = run.rng.range(gmin, gmax);
        let anchor = if crate::defs::monster_def(kind).tags.contains(&"water") && !water.is_empty() { *run.rng.pick(&water) } else { take(&mut run.rng, &open, &mut cursor) };
        for k in 0..n {
            let pos = if k == 0 {
                anchor
            } else {
                let mut cands: Vec<Pos> =
                    anchor.neighbours8().into_iter().filter(|q| run.floor.map.passable(*q) && !run.occupied(*q)).collect();
                if cands.is_empty() {
                    cands.push(take(&mut run.rng, &open, &mut cursor));
                }
                *run.rng.pick(&cands)
            };
            if run.occupied(pos) {
                continue;
            }
            let id = run.new_id();
            run.monsters.push(crate::endgame::spawn(run, id, kind, pos, depth, true));
        }
    }
    // Boss with escorts, near the down stairs. Cut 7 §1: the D5 lieutenant is placed the same
    // way (he holds the stairs), with goblins round him.
    // Cut 26 §1: the route's — a biome's boss on the last floor of the band it sits in, the
    // Captain on the Burrows' first floor.
    if let Some(boss) = run.route.boss(depth).or_else(|| run.route.lieutenant(depth)) {
        let near: Vec<Pos> = run
            .floor
            .open_tiles()
            .into_iter()
            .filter(|p| p.cheb(run.floor.stairs_down) <= 3 && !run.occupied(*p) && p.cheb(hero) > 6)
            .collect();
        let pos = if near.is_empty() { take(&mut run.rng, &open, &mut cursor) } else { *run.rng.pick(&near) };
        let id = run.new_id();
        let mut m = crate::endgame::spawn(run, id, boss, pos, depth, false);
        // Cut 30 §1: the lineage's scars on him (every meeting −5 %, cap −30 %, gone once slain).
        if let Some((_, pct)) = run.scars.iter().find(|(k, _)| k == boss) {
            m.max_hp = (m.max_hp * (100 - *pct as i32) / 100).max(1);
            m.hp = m.max_hp;
        }
        run.monsters.push(m);
        let escort = boss_escort(boss);
        // The Captain keeps a thinner guard than a boss.
        let guard = if crate::defs::monster_def(boss).boss { 40 } else { 15 };
        for q in pos.neighbours8() {
            if run.floor.map.passable(q) && !run.occupied(q) && run.rng.chance(guard) {
                let id = run.new_id();
                run.monsters.push(crate::endgame::spawn(run, id, escort, q, depth, true));
            }
        }
    }
    // Grudge monsters live on the floor they killed on; the `hunted` stalker is on every floor
    // from D3, awake and already on the hero's trail.
    // Cut 24 §2 (AK: `Ulak is avenged.`, then Ulak again): an avenged grudge retires for good.
    let hunted = hunter.filter(|h| depth >= 3 && !h.avenged && !h.tamed);
    for g in grudges.iter().filter(|g| g.lives_on(depth, biome) && !g.avenged && !g.tamed).chain(hunted) {
        let pos = take(&mut run.rng, &open, &mut cursor);
        if run.occupied(pos) {
            continue;
        }
        let id = run.new_id();
        let mut m = crate::endgame::spawn(run, id, &g.kind, pos, depth, false);
        m.make_grudge(&g.name);
        m.avenged = g.avenged;
        if hunted.is_some_and(|h| std::ptr::eq(h, g)) {
            m.awake = true;
            m.last_seen = Some(hero);
        }
        run.monsters.push(m);
    }
    // Cut 16 §1: a picked floor keeps each gold pile and each budget item at its freshness,
    // drawn on the floor's own rng so the rest of the floor (monsters, twists, stock) is the
    // floor a fresh lineage would see.
    let fresh = run.thin.get(&depth).copied().unwrap_or(1000);
    let mut thin_rng = Rng::derive(run.seed ^ depth as u64, hash_str("picked"));
    let mut keep = move || fresh >= 1000 || thin_rng.below(1000) < fresh;
    // Items.
    let budget = crate::defs::item_budget(depth);
    let kinds: Vec<&crate::defs::ItemDef> = crate::defs::ITEMS.iter().filter(|i| i.weight > 0).collect();
    let iw: Vec<u32> = kinds.iter().map(|i| if crate::descent::tier_depth(depth) < crate::defs::item_min_depth(i.kind) { 0 } else { i.weight }).collect();
    let mut open_items = run.floor.open_tiles();
    run.rng.shuffle(&mut open_items);
    let mut ic = 0usize;
    let mut place = |run: &mut Run, it: Item| {
        let pos = if ic < open_items.len() { open_items[ic] } else { *run.rng.pick(&open_items) };
        ic += 1;
        run.items.push(FloorItem { pos, item: it });
    };
    for _ in 0..budget {
        let d = kinds[run.rng.weighted(&iw)];
        let iid = run.new_item_id();
        let mut it = Item::new(iid, d.kind);
        it.enchant = forge.get(d.kind).map(|f| f.tier as i32).unwrap_or(0);
        if (d.cat == Cat::Potion || d.cat == Cat::Scroll) && run.rng.chance(50) {
            it.hint = Some(if d.benevolent { crate::item::Hint::Benevolent } else { crate::item::Hint::Malevolent });
        }
        if keep() {
            place(run, it);
        }
    }
    // The Fens stock the Bloat Mother's answer: three throwables per floor. Cut 3: so does the
    // Foundry (melee is reflected there); the Deep stocks a silence scroll per floor and a
    // lantern on its doorstep; the Sanctum a mirror scroll per floor.
    let stock: Vec<&str> = match biome {
        Biome::Fens => vec!["fire", "poison", if depth.is_multiple_of(2) { "fire" } else { "poison" }],
        Biome::Foundry => {
            // The smiths' rack: a bow on the doorstep (melee is reflected here).
            if depth == run.route.first(Biome::Foundry) {
                vec!["fire", "poison", "caustic", "bow"]
            } else if run.route.boss(depth).is_some() {
                // The Master's own stockpile, for whoever reaches him.
                vec!["fire", "fire", "poison", "caustic", "bow"]
            } else {
                vec!["fire", "poison", if depth.is_multiple_of(2) { "caustic" } else { "poison" }]
            }
        }
        Biome::Deep => {
            if depth == run.route.first(Biome::Deep) {
                vec!["silence", "silence", "lantern", "regen"]
            } else if run.route.boss(depth).is_none() {
                vec!["silence", "silence", "regen"]
            } else {
                vec!["silence", "regen"]
            }
        }
        Biome::Sanctum => vec!["mirror", "fire"],
        _ => Vec::new(),
    };
    for k in stock {
        let iid = run.new_item_id();
        let mut it = Item::new(iid, k);
        it.enchant = forge.get(k).map(|f| f.tier as i32).unwrap_or(0);
        place(run, it);
    }
    if depth >= 2 && run.rng.chance(50) {
        let iid = run.new_item_id();
        let mut it = Item::new(iid, "leash");
        it.amount = 1;
        place(run, it);
    }
    for _ in 0..6 {
        let iid = run.new_item_id();
        let mut it = Item::new(iid, "gold");
        // Coins (the pile reads `gold $2` and the stake rises by 2): the raw draw of
        // 3–6 × depth over `GOLD_DIVISOR`, rounded, never empty.
        it.amount = ((run.rng.range(3, 7) * depth as i32 + GOLD_DIVISOR / 2) / GOLD_DIVISOR).max(1);
        // Cut 20 §5: the bounty floor pays double.
        if run.bounty == Some(depth) {
            it.amount *= BOUNTY_GOLD_MULT;
        }
        if keep() {
            place(run, it);
        }
    }
    // Cut 20 §5: and holds one item of the lineage's next tier.
    if run.bounty == Some(depth) {
        let iid = run.new_item_id();
        let it = bounty_item(run.seed, depth, iid, forge);
        place(run, it);
    }
}

/// Cut 20 §5: the bounty floor's item — the newest tier of gear the floor's depth allows (the
/// weapon or armour kind with the deepest `item_min_depth` at or above the floor's content
/// depth, drawn from the run's seed and the floor among ties), forged one tier past the
/// lineage's forge, and known.
pub fn bounty_item(seed: u64, depth: u32, id: u32, forge: &BTreeMap<String, ForgeRow>) -> Item {
    let td = crate::descent::tier_depth(depth);
    let gear: Vec<&crate::defs::ItemDef> = crate::defs::ITEMS.iter().filter(|i| i.weight > 0 && matches!(i.cat, Cat::Weapon | Cat::Armour) && crate::defs::item_min_depth(i.kind) <= td).collect();
    let top = gear.iter().map(|i| crate::defs::item_min_depth(i.kind)).max().unwrap_or(0);
    let cands: Vec<&str> = gear.iter().filter(|i| crate::defs::item_min_depth(i.kind) == top).map(|i| i.kind).collect();
    let mut rng = Rng::derive(seed ^ depth as u64, hash_str("bounty"));
    let kind = if cands.is_empty() { "sword" } else { cands[rng.below(cands.len() as u32) as usize] };
    let mut it = Item::new(id, kind);
    it.enchant = forge.get(kind).map(|f| f.tier as i32).unwrap_or(0) + 1;
    it.known = true;
    it
}

pub fn monster_entity(mo: &Monster, facts: &BTreeSet<String>) -> Entity {
    let mut tags: Vec<String> = mo.tags().into_iter().filter(|t| mo.ally || facts.contains(&format!("foe:{}:{}", mo.kind, t))).collect();
    if mo.neutral {
        tags.push("captive".into());
    }
    if mo.stun > 0 {
        tags.push("stunned".into());
    }
    if mo.hex_t>0 {tags.push("hexed".into());}
    if mo.paralysed > 0 {
        tags.push("paralysed".into());
    }
    if mo.confused > 0 {
        tags.push("confused".into());
    }
    if mo.fear > 0 || mo.fleeing {
        tags.push("fleeing".into());
    }
    if mo.grudge {
        tags.push("grudge".into());
    }
    Entity {
        id: mo.id,
        kind: mo.kind.clone(),
        name: mo.name.clone(),
        x: mo.pos.x,
        y: mo.pos.y,
        hp: mo.hp,
        max_hp: mo.max_hp,
        tags,
        ally: if mo.ally { Some(true) } else { None },
        telegraph: mo.telegraph.clone(),
        cid: mo.cid,
        remembered: false,
        modifiers: mo.modifiers,
    }
}

/// Spawn the party's companions next to the hero at the start of a run.
pub fn spawn_party(run: &mut Run, party: &[Companion]) {
    let hp = run.hero.pos;
    for c in party {
        let free = hp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
        let Some(q) = free else { continue };
        let id = run.new_id();
        let mut m = companion_monster(id, c, q);
        // A waystone enters this floor directly; match the health normal descent gives here.
        m.max_hp = pet_max_hp(c, run.depth);
        m.hp = m.max_hp;
        run.monsters.push(m);
        let mut rec = c.clone();
        rec.hp = rec.max_hp;
        run.companions.push(rec);
    }
}

pub fn companion_monster(id: u32, c: &Companion, pos: Pos) -> Monster {
    let mut m = Monster::spawn(id, &c.kind, pos, 1);
    m.ally = true;
    m.awake = true;
    m.neutral = false;
    m.cid = Some(c.id);
    m.name = Some(c.name.clone());
    m.level = c.level;
    m.max_hp = pet_max_hp(c, 1);
    m.hp = m.max_hp;
    let base: Vec<&str> = m.def().tags.to_vec();
    m.extra_tags = c.tags.iter().filter(|t| !base.contains(&t.as_str())).cloned().collect();
    m
}

/// Cut 20 §2: hp a raised companion adds per level past the first.
pub const PET_LEVEL_HP: i32 = 2;

/// Cut 20 §2: a companion's max hp on `depth` — its record's, or its kind's at that depth
/// (the floors' foes grow; a pet keeps pace), plus `PET_LEVEL_HP` per level past the first.
pub fn pet_max_hp(c: &Companion, depth: u32) -> i32 {
    let d = crate::defs::monster_def(&c.kind);
    c.max_hp.max(d.hp + crate::defs::depth_hp_bonus(depth)).max(1) + PET_LEVEL_HP * (c.level.max(1) as i32 - 1)
}

/// A companion record for a freshly tamed monster.
pub fn new_companion(id: u32, m: &Monster, name: String) -> Companion {
    let tags = m.tags();
    let rules = crate::probes::default_companion_rules(&tags, 1);
    Companion { id, kind: m.kind.clone(), name, level: 1, tags, gen: 0, rules, max_rows: 2, hp: m.max_hp, max_hp: m.max_hp }
}

/// Monster tags known well enough to tame: 20% + 10% per known tag, cap 60%; a studied kind
/// (Cut 2 §5) adds 20 points.
pub fn tame_chance(facts: &BTreeSet<String>, kind: &str) -> u32 {
    let known = crate::defs::monster_def(kind).tags.iter().filter(|t| facts.contains(&format!("foe:{kind}:{t}"))).count() as u32;
    let studied = facts.contains(&format!("foe:{kind}:studied")) as u32;
    (20 + 10 * known).min(60) + 20 * studied
}

/// Cut 2 §2: a bones pile for this depth lies somewhere on the floor (item kind `bones`; the
/// item id is the dead heir's number).
pub fn place_bones(run: &mut Run) {
    let depth = run.depth;
    let biome = run.biome();
    let heirs: Vec<u32> = run.bones.iter().filter(|b| b.lies_on(depth, biome) && !run.bones_found.contains(&b.heir)).map(|b| b.heir).collect();
    if heirs.is_empty() {
        return;
    }
    let hero = run.hero.pos;
    let mut open: Vec<Pos> = run.floor.open_tiles().into_iter().filter(|p| p.cheb(hero) > 6 && run.item_at(*p).is_none()).collect();
    if open.is_empty() {
        open = run.floor.open_tiles();
    }
    for heir in heirs {
        let pos = *run.rng.pick(&open);
        let mut it = Item::new(1_000_000 + heir, "bones");
        it.amount = heir as i32;
        run.items.push(FloorItem { pos, item: it });
    }
}

/// Cut 5 §4: the doorstep (D1–2) holds situations at a rate — a shrine (`pray`), a vault (a
/// three-item cage), a nest (four sleeping jackals round a gold pile) and, when a previous
/// heir lost a companion, a stray (that companion gone wild). D1 always has one, in a room near
/// the entrance; D2 has one (80 %) or two (30 %). Rooms only (the Warrens).
///
/// Cut 12 §4: from D3 every floor rolls **one** situation of its band (`situations::place_twist`:
/// den, lock, captive, nest, shrine, vault, stray, hunger), never the previous floor's kind;
/// `Run.floor_twist` names it for the interstitial and the reel.
pub fn place_situations(run: &mut Run, lost: &[Lost]) {
    run.last_twist = run.floor_twist.take().into();
    // Cut 24 §2: a lost companion met in the last two runs rests (`Run.named_rest`).
    let lost: Vec<Lost> = lost.iter().filter(|l| !run.named_rest.contains(&l.name)).cloned().collect();
    let lost = &lost[..];
    if run.depth >= 3 {
        crate::situations::place_twist(run, lost);
        return;
    }
    if run.floor.rooms.is_empty() {
        return;
    }
    let depth = run.depth;
    // A side stream: the floor's own rng is untouched by what the situations draw.
    let mut rng = run.rng.side(hash_str("situations") ^ depth as u64);
    let mut n = if depth == 1 { 1 } else { rng.chance(80) as usize };
    if depth >= 2 && rng.chance(30) {
        n += 1;
    }
    let mut kinds: Vec<&str> = Vec::new();
    // The doorstep leans to the cage and the altar; dens are the D2+ surprise.
    let weights = |k: &str| match k {
        "nest" => if depth == 1 { 2u32 } else { 3 },
        "vault" => if depth == 1 { 4 } else { 3 },
        "shrine" => if depth == 1 { 4 } else { 3 },
        _ => 0,
    };
    let pool = ["nest", "vault", "shrine"];
    for _ in 0..n {
        let w: Vec<u32> = pool.iter().map(|k| if kinds.contains(k) { 0 } else { weights(k) }).collect();
        if w.iter().all(|x| *x == 0) {
            break;
        }
        kinds.push(pool[rng.weighted(&w)]);
    }
    let stray = !run.stray_placed && !lost.is_empty() && rng.chance(50);
    // Cut 8B §3: the first stray waits on its floor (a jackal, named once per lineage).
    let first = run.first_stray.clone().filter(|(d, _)| *d == depth && !run.stray_placed && !stray);
    let mut used: Vec<usize> = Vec::new();
    for (k, kind) in kinds.iter().enumerate() {
        let near = depth == 1 && k == 0;
        place_room_kind(run, &mut rng, kind, near, &mut used);
    }
    let wild = if stray { lost.last().map(|l| (l.kind.clone(), l.name.clone())) } else { first.map(|(_, name)| ("jackal".to_string(), name)) };
    if let Some((kind, name)) = wild {
        place_stray(run, &mut rng, &kind, &name, &mut used);
    }
}

/// Candidate rooms for a situation: not the stairs rooms, ordered by path distance from the
/// entrance.
fn situation_rooms(run: &Run) -> Vec<(i32, usize)> {
    let hero = run.hero.pos;
    let dist = run.floor.map.bfs(hero, false, &|_| false);
    let mut rooms: Vec<(i32, usize)> = run
        .floor
        .rooms
        .iter()
        .enumerate()
        .filter(|(_, r)| !r.contains(run.floor.stairs_up) && !r.contains(run.floor.stairs_down))
        .map(|(i, r)| (dist[run.floor.map.idx(r.centre())], i))
        .filter(|(d, _)| *d > 0)
        .collect();
    rooms.sort();
    rooms
}

/// A free candidate room: one of the three closest to the entrance when `near`, else any.
fn pick_room(run: &Run, rng: &mut Rng, near: bool, used: &mut Vec<usize>) -> Option<usize> {
    let rooms = situation_rooms(run);
    let free: Vec<usize> = rooms.iter().filter(|(_, i)| !used.contains(i)).map(|(_, i)| *i).collect();
    if free.is_empty() {
        return None;
    }
    let i = if near { free[rng.below(free.len().min(3) as u32) as usize] } else { free[rng.below(free.len() as u32) as usize] };
    used.push(i);
    Some(i)
}

/// Cut 12 §4: a spot for a situation — a free room's interior, or on a cave (no rooms) an
/// open tile at least six steps from the entrance and two from the stairs with room to
/// breathe (five open neighbours), drawn from the floor's side stream.
fn situation_spot(run: &Run, rng: &mut Rng, near: bool, used: &mut Vec<usize>) -> Option<Pos> {
    use crate::tiles::Tile;
    if !run.floor.rooms.is_empty() {
        let ri = pick_room(run, rng, near, used)?;
        return room_interior(run, ri);
    }
    let map = &run.floor.map;
    let dist = map.bfs(run.hero.pos, false, &|_| false);
    let mut cands: Vec<Pos> = (0..map.tiles.len())
        .filter(|&i| map.tiles[i] == Tile::Floor && dist[i] >= 6)
        .map(|i| map.pos(i))
        .filter(|p| !run.occupied(*p) && run.item_at(*p).is_none() && p.cheb(run.floor.stairs_down) > 2 && p.neighbours8().into_iter().filter(|q| map.in_bounds(*q) && map.get(*q) == Tile::Floor && !run.occupied(*q)).count() >= 5)
        .collect();
    cands.sort_by_key(|p| (p.y, p.x));
    if cands.is_empty() {
        return None;
    }
    Some(cands[rng.below(cands.len() as u32) as usize])
}

/// An interior floor tile of a room with nothing on it (the centre when the room is tiny).
fn room_interior(run: &Run, ri: usize) -> Option<Pos> {
    use crate::tiles::Tile;
    let r = run.floor.rooms[ri];
    let mut cands: Vec<Pos> = Vec::new();
    for y in r.y..r.y + r.h {
        for x in r.x..r.x + r.w {
            let p = Pos::new(x, y);
            let inner = x > r.x && y > r.y && x < r.x + r.w - 1 && y < r.y + r.h - 1;
            if run.floor.map.get(p) == Tile::Floor && !run.occupied(p) && run.item_at(p).is_none() && (inner || r.w <= 2 || r.h <= 2) {
                cands.push(p);
            }
        }
    }
    if cands.is_empty() {
        let c = r.centre();
        return (run.floor.map.get(c) == Tile::Floor).then_some(c);
    }
    Some(cands[(cands.len() / 2).min(cands.len() - 1)])
}

/// Cut 5 §4 / Cut 12 §4: a room situation — `shrine`, `vault` or `nest` — in a free room.
/// Returns whether it was placed (a floor without a free room places nothing).
pub fn place_room_kind(run: &mut Run, rng: &mut Rng, kind: &str, near: bool, used: &mut Vec<usize>) -> bool {
    use crate::tiles::Tile;
    let depth = run.depth;
    let Some(p) = situation_spot(run, rng, near, used) else { return false };
    // Nothing else stands on the tile (monsters and items may have landed there).
    run.items.retain(|fi| fi.pos != p);
    run.monsters.retain(|m| m.pos != p);
    match kind {
        "shrine" => run.floor.map.set(p, Tile::Shrine),
        "vault" => {
            run.floor.map.set(p, Tile::Vault);
            run.vault_cage = vault_cage(run, rng, depth);
        }
        _ => {
            run.floor.map.set(p, Tile::Nest);
            let mut gold = Item::new(run.new_item_id(), "gold");
            gold.amount = (12 * depth as i32 + rng.range(4, 12) + GOLD_DIVISOR / 2) / GOLD_DIVISOR;
            // Cut 16 §1: a picked floor's nest holds less.
            let fresh = run.thin.get(&depth).copied().unwrap_or(1000) as i32;
            gold.amount = ((gold.amount * fresh + 500) / 1000).max(1);
            run.items.push(FloorItem { pos: p, item: gold });
            let mut placed = 0;
            let mut spots: Vec<Pos> = p.neighbours8().into_iter().filter(|q| run.floor.map.get(*q) == Tile::Floor && !run.occupied(*q)).collect();
            rng.shuffle(&mut spots);
            // Three sleepers to D3, four below; a den's jackals are D1 jackals (no depth
            // bonus): the surprise is the shape — four at once round the gold — not the
            // numbers (DEFAULT must still last ten minutes a run).
            let sleepers = if depth <= 3 { 3 } else { 4 };
            for q in spots.into_iter().take(sleepers) {
                let id = run.new_id();
                let mut m = Monster::spawn(id, "jackal", q, 1);
                m.nest = true;
                m.dormant = true;
                run.monsters.push(m);
                placed += 1;
            }
            if placed == 0 {
                run.floor.map.set(p, Tile::Floor);
                return false;
            }
        }
    }
    true
}

/// Cut 5 §4: a stray — a lost companion gone wild (or Cut 8B §3's first jackal) — in a free
/// room. Returns whether it was placed.
pub fn place_stray(run: &mut Run, rng: &mut Rng, kind: &str, name: &str, used: &mut Vec<usize>) -> bool {
    let Some(p) = situation_spot(run, rng, false, used) else { return false };
    let id = run.new_id();
    let mut m = Monster::spawn(id, kind, p, run.depth);
    m.stray = true;
    m.name = Some(name.to_string());
    m.level = 1;
    run.monsters.push(m);
    run.stray_placed = true;
    run.named_placed.push(name.to_string());
    true
}

/// Cut 12 §4: the stray a twist floor would place — the lineage's first jackal on its floor,
/// else the last companion lost — while none has been placed this run.
pub fn wild_for(run: &Run, lost: &[Lost]) -> Option<(String, String)> {
    if run.stray_placed {
        return None;
    }
    if let Some((d, name)) = &run.first_stray {
        if *d == run.depth {
            return Some(("jackal".into(), name.clone()));
        }
    }
    lost.last().map(|l| (l.kind.clone(), l.name.clone()))
}

/// The vault's three items: a weapon, an armour and a consumable, a cut above the floor.
/// Never an enchant scroll: the quartermaster keeps the best weapon for life, and a scroll a
/// run stacked a FULL fighter's sword to +12 by the Sanctum (the Mirror King's wall fell).
fn vault_cage(run: &mut Run, rng: &mut Rng, depth: u32) -> Vec<Item> {
    let weapon = if depth >= 3 && rng.chance(40) { "axe" } else if rng.chance(30) { "bow" } else { "sword" };
    let armour = if depth >= 3 && rng.chance(50) { "mail" } else { "leather" };
    let consumable = if rng.chance(50) { "heal" } else if rng.chance(50) { "strength" } else { "teleport" };
    let mut out = Vec::new();
    for k in [weapon, armour, consumable] {
        let mut it = Item::new(run.new_item_id(), k);
        if it.cat() == Cat::Weapon || it.cat() == Cat::Armour {
            it.enchant = 1;
        }
        out.push(it);
    }
    out
}

pub fn item_wire(item: &Item, facts: &BTreeSet<String>, flavours: &Flavours) -> InvItem {
    to_inv(item, facts, flavours)
}

pub fn kind_title(kind: &str) -> String {
    if crate::defs::MONSTERS.iter().any(|m| m.kind == kind) {
        crate::defs::monster_def(kind).title.to_string()
    } else {
        kind.replace('_', " ")
    }
}

/// A boss's escort and summons (`populate_floor`; Cut 24 §1: the summons whose fall moves his
/// fight, `turn::nohp_guard`).
pub fn boss_escort(boss: &str) -> &'static str {
    match boss {
        "goblin_warlord" | "goblin_captain" => "goblin",
        "bloat_mother" => "bloat",
        "foundry_master" => "smith",
        "lurker_queen" => "lurker",
        "mirror_king" => "mirror_shade",
        _ => "skeleton",
    }
}

/// A tier's word in the past (`bank` → `banked`), for `Game::run_news`.
fn past(tier: &str) -> String {
    match tier {
        "bank" => "banked".into(),
        "return" => "returned".into(),
        "death" => "died".into(),
        t => t.to_string(),
    }
}

/// `auto_keep`'s decision: the pending ids that go to the vault and the vault ids they replace
/// (see `Game::auto_keep` for the precedence). Never plans past `slots`.
/// Cut 29 §4 (AW: "one ▲ per tap"; the keep sheet asked on every exit): the keep sheet is a
/// decision only when a find beats something in a full vault (a find of its category worth more),
/// or a caged item came home (Cut 28 §4: the player's choice). Otherwise the exit settles by the
/// standing order (`auto_keep`) and says so in one line (`keep_note`).
pub fn keep_is_a_decision(p: &PendingExit, vault: &[Item], slots: usize) -> bool {
    if slots == 0 {
        return false;
    }
    let finds: Vec<&Item> = p.items.iter().filter(|i| !p.brought.contains(&i.id) && matches!(i.cat(), Cat::Weapon | Cat::Armour)).collect();
    let full = vault.len() >= slots;
    let beats = finds.iter().any(|f| vault.iter().any(|v| v.cat() == f.cat() && f.value() > v.value()));
    let room_contest = !full && finds.len() > slots - vault.len();
    (full && beats) || room_contest
}

/// The one line a settled exit says (`kept leather +1`, `kept sword +1 · mail`); `None` when it
/// keeps nothing new.
pub fn keep_note(p: &PendingExit, l: &LineageState, plan: &[u32]) -> Option<String> {
    let names: Vec<String> = p.items.iter().filter(|i| plan.contains(&i.id) && !p.brought.contains(&i.id)).map(|i| {
        let k = l.wire_name(&i.kind).replace('_', " ");
        if i.enchant > 0 { format!("{k} +{}", i.enchant) } else { k }
    }).collect();
    (!names.is_empty()).then(|| format!("kept {}", names.join(" · ")))
}

pub fn auto_keep_plan(p: &PendingExit, vault: &[Item], slots: usize, keep_pref: &str, quartermaster: bool) -> (Vec<u32>, Vec<u32>) {
    let mut ids: Vec<u32> = Vec::new();
    let mut evict: Vec<u32> = Vec::new();
    if slots == 0 {
        return (ids, evict);
    }
    // (id, cat, value) of what the vault will hold.
    let mut held: Vec<(u32, Cat, i32)> = vault.iter().map(|v| (v.id, v.cat(), v.value())).collect();
    for it in p.items.iter().filter(|i| p.brought.contains(&i.id)) {
        if held.len() < slots {
            held.push((it.id, it.cat(), it.value()));
            ids.push(it.id);
        }
    }
    let (first, second) = match keep_pref {
        "best_weapon" => (Cat::Weapon, Cat::Armour),
        "best_armour" => (Cat::Armour, Cat::Weapon),
        _ => return (ids, evict),
    };
    let best = |cat: Cat| p.items.iter().filter(|i| i.cat() == cat && !p.brought.contains(&i.id)).max_by_key(|i| (i.value(), i.id));
    let mut place = |it: &Item, upgrade: bool, ids: &mut Vec<u32>, evict: &mut Vec<u32>| {
        if held.len() < slots {
            held.push((it.id, it.cat(), it.value()));
            ids.push(it.id);
            return;
        }
        if !upgrade {
            return;
        }
        // The weakest vault item of the same category that this find beats; never one kept
        // on this exit.
        let weakest = held.iter().enumerate().filter(|(_, (id, c, v))| *c == it.cat() && *v < it.value() && !ids.contains(id)).min_by_key(|(_, (id, _, v))| (*v, *id)).map(|(i, _)| i);
        if let Some(i) = weakest {
            let (old, _, _) = held.remove(i);
            evict.push(old);
            held.push((it.id, it.cat(), it.value()));
            ids.push(it.id);
        }
    };
    let a = best(first);
    if let Some(it) = a {
        place(it, true, &mut ids, &mut evict);
    }
    if let Some(it) = best(second) {
        if quartermaster {
            place(it, true, &mut ids, &mut evict);
        } else if a.is_none() {
            place(it, false, &mut ids, &mut evict);
        }
    }
    (ids, evict)
}

fn is_zero_u32(n: &u32) -> bool {
    *n == 0
}

/// The automatic pack reserves each active counter once, then fills sustain slots with other kinds.
fn automatic_pack_plan(l: &LineageState) -> Vec<String> {
    let qm = crate::packages::quartermaster(l);
    let fill: Vec<String> = crate::packages::pack_kinds(l).into_iter().filter(|k| !qm.contains(k)).collect();
    let mut want = qm;
    want.extend(fill.iter().cycle().take(if fill.is_empty() { 0 } else { l.supply_cap().min(crate::packages::PACK_FILL) }).cloned());
    want
}

/// Old automatic slots track the current package pack; authored and unknown-origin repeats stay exact.
fn current_repeat_plan(l: &LineageState, plan: Vec<(String, bool)>) -> Vec<(String, bool)> {
    if l.pkg.literal || l.pkg.stance == crate::packages::CUSTOM { return plan; }
    let want = automatic_pack_plan(l);
    let mut kept: BTreeMap<String, usize> = BTreeMap::new();
    plan.into_iter().filter(|(kind, automatic)| {
        if !automatic { return true; }
        let n = kept.entry(kind.clone()).or_default();
        *n += 1;
        *n <= want.iter().filter(|k| *k == kind).count()
    }).collect()
}

/// Assign surviving supplies to their repeat slots. Reserve exact origins across the whole
/// plan before matching kinds alone, so a manual duplicate cannot consume an automatic slot.
fn supplied_slots(plan: &[(String, bool)], supplies: &[(String, bool)]) -> Vec<bool> {
    supplied_slot_assignments(plan, supplies).into_iter().map(|i| i.is_some()).collect()
}

fn supplied_slot_assignments(plan: &[(String, bool)], supplies: &[(String, bool)]) -> Vec<Option<usize>> {
    let mut available: Vec<(usize, &(String, bool))> = supplies.iter().enumerate().collect();
    let mut supplied = vec![None; plan.len()];
    for exact in [true, false] {
        for (slot, (kind, automatic)) in plan.iter().enumerate() {
            if supplied[slot].is_some() { continue; }
            if let Some(i) = available.iter().position(|(_, (k, a))| k == kind && (!exact || a == automatic)) {
                supplied[slot] = Some(available.remove(i).0);
            }
        }
    }
    supplied
}
