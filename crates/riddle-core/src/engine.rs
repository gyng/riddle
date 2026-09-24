//! The game: lineage state, the live run, and the public API mirrored by the wasm bridge.
use crate::defs::{spawn_table, Cat};
use crate::descent::{biome_for, boss_for, Biome, Grudge, ENDING_DEPTH};
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
/// Cut 4: a hostile that stepped out of view is remembered (snapshot `remembered`, the hunt)
/// for this many hero actions after it was last seen.
pub const REMEMBER_ACTIONS: u32 = 10;
/// A run that cannot finish in this many ticks (≈ 3 h 20 min at 1×) comes home empty-handed
/// (tier `return`, yield ×0: a stalemate is not a policy; the DEFAULT set must yield nothing
/// over 8 h, and a stall that kept the return's share paid it). Cut 3: a D30 run needs ~75 000.
pub const MAX_TURNS_PER_RUN: u32 = 120_000;
/// Oscillation-guard firings on one floor before the run ends as `stalled` and gets a verdict.
pub const STALL_FIRES: u32 = 3;
/// A new heir's purse is topped up to this (one potion) so the camp is never at $0 after a death.
pub const WAKE_PAY: i32 = 40;
/// Cut 3: rule rows with every row unlock (`row5`–`row10`).
pub const MAX_ROWS: usize = 10;
/// Cut 12 §1: card rows sit outside the player's cap — one per owned card (8 tactic, 4 tier 2,
/// 4 mastery); the per-row tallies are sized for both.
pub const MAX_CARD_ROWS: usize = 16;
pub const ROWS_TOTAL: usize = MAX_ROWS + MAX_CARD_ROWS;
/// Cut 3: the ascension variants, in the order they are offered.
pub const VARIANTS: [&str; 4] = ["no_rest", "short_list", "bones_only", "hunted"];
/// Energy needed to act; actors gain `speed` per tick.
pub const ACT_ENERGY: i32 = 100;
pub const TICKS_PER_TURN: u32 = 10;
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FloorItem {
    pub pos: Pos,
    pub item: Item,
}

/// Everything about one expedition. Cloned per turn into the history ring (for verdicts).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Run {
    pub id: u32,
    pub heir: u32,
    pub started_turn: u64,
    pub rng: Rng,
    pub depth: u32,
    pub floor: Floor,
    pub hero: Hero,
    pub trait_: Trait,
    pub monsters: Vec<Monster>,
    pub items: Vec<FloorItem>,
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
    pub kills: Vec<(u32, String, u32)>,
    /// QA on 23ed91f: summoned foes cut down this run (no XP, renown or bestiary count).
    #[serde(default)]
    pub summoned_kills: u32,
    pub brought: Vec<u32>,
    pub trace: Vec<TraceTurn>,
    pub notes: Vec<(u32, String)>,
    pub over: Option<ExitTier>,
    pub death_cause: Option<String>,
    pub death_blow: i32,
    /// Cut 10 §3: the HP the hero was short of taking the killing blow (`Death.margin`: `3 hp
    /// short` — the blow overshot by two, one more would have held).
    #[serde(default)]
    pub death_short: i32,
    pub hurt_since_action: bool,
    pub hurt_last: bool,
    pub kill_since_action: bool,
    pub kill_last: bool,
    pub seen_ids: BTreeSet<u32>,
    pub new_seen: bool,
    pub telegraphs_now: Vec<String>,
    pub low10_t: Option<u32>,
    pub low20_t: Option<u32>,
    pub near_deaths: Vec<u32>,
    pub gambles: Vec<(u32, String, bool)>,
    pub gambles_survived: Vec<(u32, String)>,
    pub stolen: Vec<(u32, String)>,
    pub ally_lost: Vec<(u32, String)>,
    pub ally_freed: Vec<u32>,
    pub boss_kills: Vec<(u32, String)>,
    pub drank_heal: bool,
    pub melee_used: bool,
    pub boss_seen_t: Option<u32>,
    pub hurt_since_boss: bool,
    pub row_fired: Vec<u32>,
    pub renderable_events: u32,
    pub ended: bool,
    pub max_depth: u32,
    pub trophies_run: Vec<String>,
    /// Companion records active in this run (party members and new tames).
    pub companions: Vec<Companion>,
    pub recalled: Vec<u32>,
    pub tamed: Vec<(u32, String)>,
    pub lost_companions: Vec<(u32, String)>,
    /// Item ids of camp supplies (never kept back).
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
    /// Cached BFS field from the hero (recomputed when the hero moves).
    #[serde(skip)]
    pub hero_dist: Vec<i32>,
    #[serde(skip)]
    pub hero_dist_pos: Option<Pos>,
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
    /// Where hostiles were last seen (id → position, action), so pathing does not flip
    /// between "blocked" and "open" as a corridor foe drifts in and out of view.
    #[serde(default)]
    pub known_foes: BTreeMap<u32, (Pos, u32)>,
    /// Same-row loop guard: (row, consecutive firings) and the row suppressed until an action.
    #[serde(default)]
    pub row_streak: (i32, u32),
    #[serde(default)]
    pub row_suppressed: (i32, u32),
    /// Rests taken on this floor (the rest clock).
    #[serde(default)]
    pub rests: u32,
    /// The current hero strike is aimed (`attack tag:boss` / bash): goes through shield walls.
    #[serde(default)]
    pub aimed: bool,
    /// Cut 2 §2: the lineage's bones piles (copied at start) and the heirs recovered this run.
    #[serde(default)]
    pub bones: Vec<Bones>,
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
    pub episodes: Vec<Episode>,
    /// §3: the hero's last line and the tick the current fight began (no lines in its first ten).
    #[serde(default)]
    pub voice_t: Option<u32>,
    #[serde(default)]
    pub fight_t: Option<u32>,
    /// §4: situations met this run (`shrine | vault | nest | stray`, with the tick).
    #[serde(default)]
    pub situations: Vec<(u32, String)>,
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
    pub passed: Vec<String>,
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
    pub floor_twist: Option<String>,
    #[serde(default)]
    pub last_twist: Option<String>,
    #[serde(default)]
    pub seed: u64,
    #[serde(skip)]
    pub next_twist: Option<String>,
    // Cut 16
    /// §1: the freshness of the floors this run may generate, in permille by depth (only
    /// thinned depths; `LineageState::thin_map` at the send) — a floor's gold piles and item
    /// budget are kept at this rate (`populate_floor`).
    #[serde(default)]
    pub thin: BTreeMap<u32, u32>,
}

impl Run {
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
        let seen = map.tiles.iter().enumerate().any(|(i, x)| *x == tile && map.visible[i]);
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
        if it.id == 1 || self.brought.contains(&it.id) || self.supplies.contains(&it.id) {
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
        biome_for(self.depth)
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
        self.floor.vision + if lantern { 2 } else { 0 }
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
pub struct Ctx<'a> {
    pub facts: &'a mut BTreeSet<String>,
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
    pub max_rows: usize,
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
}

/// Cut 5 §4: a companion that died on an expedition (its kennel entry is gone); a later run
/// may meet it gone wild on D1–5, tameable at `STRAY_TAME`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Lost {
    pub kind: String,
    pub name: String,
    pub gen: u32,
    pub heir: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageState {
    pub seed: u64,
    pub heir: u32,
    pub trait_: Trait,
    pub class: Class,
    pub best_depth: u32,
    pub marks: u32,
    pub facts: BTreeSet<String>,
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
    /// Bones piles (§2), oldest first.
    #[serde(default)]
    pub bones: Vec<Bones>,
    /// Kills per kind (§5 studied).
    #[serde(default)]
    pub kill_counts: BTreeMap<String, u32>,
    /// Supplies of the last expedition, for the `auto_supply` automation.
    #[serde(default)]
    pub last_supplies: Vec<String>,
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
    /// QA on 56f2a1d: the kennel's free leash, dropped from the shelf, came back after every
    /// run. A drop declines the kennel until a leash is bought or a kind is tamed.
    #[serde(default)]
    pub kennel_declined: bool,
    /// §3: kinds the last run used to no effect — `restock` skips them once.
    #[serde(default)]
    pub last_wasted: Vec<String>,
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
}

/// Cut 16 §1: a night of runs (the ledger's "a night of 16 runs").
pub const NIGHT_RUNS: u32 = 16;
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
            classes.insert(c.name().to_string(), ClassProg { level: 1, xp: 0, next: 0 });
        }
        let mut l = LineageState {
            seed,
            heir: 1,
            trait_,
            class: Class::Fighter,
            best_depth: 0,
            marks: 0,
            facts: BTreeSet::new(),
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
            forge: BTreeMap::new(),
            renown: 0,
            rank: 0,
            keep_pref: "best_weapon".into(),
            insured: Vec::new(),
            rest_left: 0,
            bones: Vec::new(),
            kill_counts: BTreeMap::new(),
            last_supplies: Vec::new(),
            eggs_laid: 0,
            runs_banked: 0,
            runs_returned: 0,
            runs_died: 0,
            ascension: 0,
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
            kennel_declined: false,
            last_wasted: Vec::new(),
            gold_buys: 0,
            picked: BTreeMap::new(),
            night_runs: 0,
            night_seen: BTreeSet::new(),
        };
        // Cut 8B §3: `tame` is owned from the start and the kennel's leash is on the shelf (its
        // fact with it), so the first stray is a companion in the first hour.
        l.unlocks.insert("tame".into());
        l.facts.insert("item:leash".into());
        l.kennel_leash();
        l
    }
    /// Cut 8B §3: the kennel's leash — while the lineage has never tamed, a free, known leash
    /// sits on the shelf (never refunded, never rebought; one at a time).
    pub fn kennel_leash(&mut self) {
        if self.kennel_declined || self.tamed_kinds() > 0 || self.supplies.iter().any(|s| s.kind == "leash") || self.supplies.len() >= self.supply_cap() {
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
        self.gold += delta;
        let exit = why.starts_with("returned") || why.starts_with("banked") || why.starts_with("died") || why.starts_with("lost") || why.starts_with("stalled");
        if delta == 0 && !exit {
            return;
        }
        let t = self.total_turns;
        if let Some(last) = self.gold_ledger.last_mut() {
            if last.t == t && last.why == why && !exit {
                last.delta += delta;
                return;
            }
        }
        self.gold_ledger.push(GoldLine { t, delta, why: why.into() });
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
        let mut parts = vec![format!("♟{} the {} {}", self.heir, epithet, self.class.name()), format!("D{}", self.heir_best)];
        if let Some(name) = self.rules().name.as_deref().filter(|n| !n.trim().is_empty()) {
            parts.push(format!("\"{}\" set", name.trim()));
        }
        // Bosses first, then the rest, two at most.
        let mut deeds: Vec<String> = self.heir_deeds.iter().filter(|d| d.starts_with("took the")).cloned().collect();
        deeds.extend(self.heir_deeds.iter().filter(|d| !d.starts_with("took the")).cloned());
        parts.extend(deeds.into_iter().take(2));
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
        if !self.heir_deeds.contains(&deed) && self.heir_deeds.len() < 6 {
            self.heir_deeds.push(deed);
        }
    }
    pub fn variant_is(&self, v: &str) -> bool {
        self.variant == v
    }
    pub fn rules(&self) -> &RuleSet {
        &self.sets[self.active_set.min(self.sets.len() - 1)]
    }
    pub fn class_level(&self) -> u32 {
        self.classes.get(self.class.name()).map(|c| c.level).unwrap_or(1)
    }
    pub fn to_wire(&self) -> Lineage {
        Lineage {
            seed: self.seed,
            heir: self.heir,
            trait_: self.trait_.name().into(),
            trait_offer: self.trait_offer.iter().map(|t| t.name().to_string()).collect(),
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
            gold: self.gold,
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
            ascended: self.ascended.clone(),
            chronicle: self.chronicle.clone(),
            vault_pref: self.vault_pref.clone(),
            gold_ledger: self.gold_ledger.clone(),
            counters: self.counters(),
            combos: self.rules().combos(),
            picked: self.picked_clean(),
            class_offer: self.class_offer(),
            shadowed_by: self.shadowed_by(self.rules()),
        }
    }
    /// QA on 92eb880: `rules`' shadowed rows for this lineage (a row whose condition it does not
    /// own never fires, so it shadows nothing); empty when none is shadowed.
    pub fn shadowed_by(&self, rules: &RuleSet) -> Vec<Option<u32>> {
        let v = rules.shadowed_by(self.max_rows(), |c| crate::meta::cond_unlock(&c.k).is_none_or(|u| self.unlocks.contains(u)) && (c.k != "on_see" || c.t.as_deref().is_none_or(|t| t.is_empty() || self.facts.contains(t))));
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
        let mut owned: Vec<Class> = Class::ALL.iter().copied().filter(|c| c.unlock().is_none_or(|u| self.unlocks.contains(u))).collect();
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
    /// The categories an unwatched exit keeps, in order (`Game::auto_keep`).
    pub fn keep_auto(&self) -> Vec<String> {
        let qm = self.unlocks.contains("quartermaster");
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
        // One descent to the `row…` ids, not six lookups: this runs on every tick (`ctx`).
        use std::ops::Bound::{Excluded, Included};
        let n = 4 + self.unlocks.range::<str, _>((Included("row"), Excluded("rox"))).filter(|u| ["row5", "row6", "row7", "row8", "row9", "row10"].contains(&u.as_str())).count();
        if self.variant_is("short_list") {
            n.min(6)
        } else {
            n
        }
    }
    pub fn party_slots(&self) -> u32 {
        1 + self.unlocks.contains("party_slot_2") as u32 + self.unlocks.contains("party_slot_3") as u32 + self.unlocks.contains("party_slot_4") as u32
    }
    /// Supplies per expedition (Cut 2 §3 `supply_cap_5`).
    pub fn supply_cap(&self) -> usize {
        if self.unlocks.contains("supply_cap_5") {
            5
        } else {
            3
        }
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
        self.heir_deeds.clear();
        self.heir_best = 0;
        let first = Trait::ALL[self.rng.below(4) as usize];
        let offer = trait_offer(self.seed, self.heir + 1000 * self.ascension, Some(last), first);
        self.trait_ = offer[0];
        self.trait_offer = offer.to_vec();
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
    }

    /// Cut 13 §2: pick one of the offered traits (the chip beside `♟3`); refused when it is
    /// not on offer (a send without a pick keeps the first).
    pub fn set_trait(&mut self, name: &str) -> Result<(), String> {
        let t = self.trait_offer.iter().copied().find(|t| t.name() == name).ok_or_else(|| "not on offer".to_string())?;
        self.trait_ = t;
        Ok(())
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
    pub t10: Option<Run>,
    pub t10_facts: BTreeSet<String>,
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
}

fn is_zero_u64(n: &u64) -> bool {
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
}

fn default_pct() -> i32 {
    60
}

/// Accumulated outcomes since the last return report.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Batch {
    pub runs: u32,
    /// Cut 15 §1: banks from depth ≥ the lineage's best − 1 this batch (each paid ◆1).
    #[serde(default)]
    pub frontier_banks: u32,
    pub bests: Vec<String>,
    pub found: Vec<Item>,
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
    /// §3: what the automations bought this batch, per kind → (n, coins); the exact coins of
    /// salvage and of wake pay, so the night's gold reconciles to the coin
    /// (`gold_earned + salvage_gold + wake_pay − spent == the purse's delta`).
    #[serde(default)]
    pub spent: BTreeMap<String, (u32, i32)>,
    #[serde(default)]
    pub salvage_gold: i32,
    #[serde(default)]
    pub wake_pay: i32,
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
    pub history: VecDeque<(Run, BTreeSet<String>)>,
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
    /// Cut 6 §1: the ledger line of the last settled exit (`step` attaches it to `Ev::Exit`).
    #[serde(default)]
    pub last_exit: Option<ExitLine>,
    /// Cut 7 §5: the live run is being watched (`send()` / `step()` set it, the offline batch
    /// clears it): a watched bank grants +50% class XP.
    #[serde(default)]
    pub watched: bool,
}

fn default_max_deaths() -> usize {
    40
}

impl Game {
    pub fn new(seed: u64) -> Game {
        Game {
            version: SAVE_VERSION,
            lineage: LineageState::new(seed),
            run: None,
            deaths: BTreeMap::new(),
            loadout: Vec::new(),
            sim: false,
            history: VecDeque::new(),
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
            last_exit: None,
            watched: false,
            stall: StallTally::default(),
            stall_cache: None,
            forecast_cache: Default::default(),
            panel_cache: Default::default(),
        }
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
            stall: StallTally::default(),
            stall_cache: None,
            forecast_cache: Default::default(),
            panel_cache: Default::default(),
            last_exit: None,
            watched: false,
        }
    }

    pub fn lineage(&self) -> Lineage {
        self.lineage.to_wire()
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
        let i = self.lineage.active_set.min(self.lineage.sets.len() - 1);
        if self.lineage.sets[i] != set {
            self.stall = StallTally::default();
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
        if let Some(u) = c.unlock() {
            if !self.lineage.unlocks.contains(u) {
                return Err(format!("{u} not unlocked"));
            }
        }
        self.lineage.class = c;
        Ok(())
    }

    pub fn set_keep_pref(&mut self, pref: &str) -> Result<(), String> {
        if !["best_weapon", "best_armour", "none"].contains(&pref) {
            return Err("unknown keep_pref".into());
        }
        self.lineage.keep_pref = pref.into();
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

    /// Cut 5 §5: bail — a `return` fires on the hero's next action, the rules untouched.
    pub fn bail(&mut self) {
        if let Some(run) = self.run.as_mut() {
            if run.over.is_none() {
                run.bail = true;
            }
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
        crate::turn::vault_take(run, &mut cx, Some(item_id));
        Ok(())
    }

    pub fn loadout(&mut self, ids: Vec<u32>) {
        if self.lineage.variant_is("bones_only") {
            self.loadout.clear();
            return;
        }
        self.loadout = ids.into_iter().filter(|id| self.lineage.vault.iter().any(|v| v.id == *id)).collect();
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
        l.renown = 0;
        l.rank = 0;
        l.graveyard.clear();
        l.bones.clear();
        l.insured.clear();
        l.supplies.clear();
        l.last_supplies.clear();
        l.rest_left = 0;
        l.runs_banked = 0;
        l.runs_returned = 0;
        l.runs_died = 0;
        // Cut 13 §2: the ascended lineage's first heir is a new heir — two traits on offer.
        let first = Trait::ALL[l.rng.below(4) as usize];
        let offer = trait_offer(l.seed, l.heir + 1000 * l.ascension, Some(l.trait_), first);
        l.trait_ = offer[0];
        l.trait_offer = offer.to_vec();
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
        let r = crate::offline::rest_after(turns, tier);
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
    pub fn send(&mut self) -> Snapshot {
        self.lineage.rest_left = 0;
        self.watched = true;
        self.ensure_run()
    }

    /// Cut 13 §2: pick the heir's trait from the camp's offer (`Lineage.trait_offer`).
    pub fn set_trait(&mut self, name: &str) -> Result<(), String> {
        self.lineage.set_trait(name)
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
        self.auto_keep();
        self.prov.clear();
        // Cut 13 §2: the send settles the heir's trait; the offer is spent.
        self.lineage.trait_offer.clear();
        let id = self.lineage.next_run_id;
        self.lineage.next_run_id += 1;
        let seed = seed_override.unwrap_or_else(|| self.run_seed(id));
        let mut rng = Rng::new(seed);
        let floor = generate(&mut rng, biome_for(1), 1);
        let mut hero = Hero::new(self.lineage.class, floor.stairs_up);
        hero.apply_level(self.lineage.class_level());
        // Starting arms by class (id 1 is never loot).
        hero.auto_equip(Item::new(1, self.lineage.class.starting_weapon()));
        let mut brought = Vec::new();
        let mut loadout = std::mem::take(&mut self.loadout);
        loadout.sort();
        loadout.dedup();
        // Automations (Cut 2 §3): auto_insure covers the brought items when gold allows;
        // auto_supply restocks the last expedition's supplies.
        if self.lineage.unlocks.contains("auto_insure") {
            for id in loadout.clone() {
                let gold = self.lineage.gold;
                let kind = self.lineage.vault.iter().find(|v| v.id == id).map(|v| v.kind.replace('_', " "));
                if self.insure(id).is_ok() {
                    // Cut 13 §3: the night's ledger counts the automation's premiums.
                    let e = self.batch.spent.entry(format!("insure {}", kind.unwrap_or_default())).or_insert((0, 0));
                    e.0 += 1;
                    e.1 += gold - self.lineage.gold;
                }
            }
        }
        self.restock();
        self.lineage.last_supplies = self.lineage.supplies.iter().filter(|i| !i.free).map(|i| i.kind.clone()).collect();
        for id in loadout {
            if let Some(i) = self.lineage.vault.iter().position(|v| v.id == id) {
                let it = self.lineage.vault.remove(i);
                brought.push(it.id);
                hero.auto_equip(it);
            }
        }
        let mut run = Run {
            id,
            heir: self.lineage.heir,
            started_turn: self.lineage.total_turns,
            rng,
            depth: 1,
            floor,
            hero,
            trait_: self.lineage.trait_,
            monsters: Vec::new(),
            items: Vec::new(),
            overlays: Vec::new(),
            turn: 0,
            floor_turn: 0,
            alert: 0,
            loot: 0,
            next_id: HERO_ID,
            next_item_id: 10,
            kills_floor: 0,
            kills: Vec::new(),
            summoned_kills: 0,
            brought,
            trace: Vec::new(),
            notes: Vec::new(),
            over: None,
            death_cause: None,
            death_blow: 0,
            death_short: 0,
            hurt_since_action: false,
            hurt_last: false,
            kill_since_action: false,
            kill_last: false,
            seen_ids: BTreeSet::new(),
            new_seen: false,
            telegraphs_now: Vec::new(),
            low10_t: None,
            low20_t: None,
            near_deaths: Vec::new(),
            gambles: Vec::new(),
            gambles_survived: Vec::new(),
            stolen: Vec::new(),
            ally_lost: Vec::new(),
            ally_freed: Vec::new(),
            boss_kills: Vec::new(),
            drank_heal: false,
            melee_used: false,
            boss_seen_t: None,
            hurt_since_boss: false,
            row_fired: vec![0; ROWS_TOTAL],
            renderable_events: 0,
            ended: false,
            max_depth: 1,
            trophies_run: Vec::new(),
            companions: Vec::new(),
            recalled: Vec::new(),
            tamed: Vec::new(),
            lost_companions: Vec::new(),
            supplies: Vec::new(),
            taunt_t: 0,
            kited: None,
            pack_go: 0,
            hero_dist: Vec::new(),
            hero_dist_pos: None,
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
            trait_last: None,
            trait_floor: 0,
            wasted_kinds: Vec::new(),
            pickup_streak: 0,
            pickup_inv: 0,
            items_until: 0,
            known_foes: BTreeMap::new(),
            loot_raw: 0,
            low_hp: i32::MAX,
            saved_by: None,
            last_target: None,
            hunt: None,
            blocked_now: None,
            blocked_last: None,
            row_streak: (-9, 0),
            row_suppressed: (-9, 0),
            rests: 0,
            aimed: false,
            bones: self.lineage.bones.clone(),
            bones_found: Vec::new(),
            kills_counted: 0,
            timed_out: false,
            exit_row: None,
            noise: None,
            verb_ring: Vec::new(),
            last_hit_verb: None,
            blind_seen: Vec::new(),
            bow_swap: None,
            mirrors: Vec::new(),
            arc: Arc::default(),
            episodes: Vec::new(),
            voice_t: None,
            fight_t: None,
            situations: Vec::new(),
            vault_cage: Vec::new(),
            vault_choice: None,
            prayed: false,
            lent_row: None,
            stray_placed: false,
            first_stray: self.lineage.first_stray(),
            strays_tamed: Vec::new(),
            bail: false,
            dens: Vec::new(),
            tempted: false,
            rows_why: Vec::new(),
            ending_t: None,
            den_stolen: Vec::new(),
            gas_dmg_floor: 0,
            lock_last_pop: 0,
            lit: false,
            passed: Vec::new(),
            lock_tiles: Vec::new(),
            raiding: false,
            acting_row: -1,
            floor_twist: None,
            last_twist: None,
            seed,
            next_twist: None,
            sleepers: Vec::new(),
            thin: self.lineage.thin_map(),
        };
        for s in std::mem::take(&mut self.lineage.supplies) {
            let mut it = s;
            it.id = run.new_item_id() + 5000;
            run.supplies.push(it.id);
            if it.kind == "leash" {
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
        let vision = run.vision(&self.lineage.unlocks);
        run.floor.map.update_vision(run.hero.pos, vision);
        self.history.clear();
        self.facts_at_run_start = self.lineage.facts.len();
        self.run = Some(run);
        let mut cx = self.ctx();
        let run = cx.0;
        crate::facts::on_vision(run, &mut cx.1);
        crate::chronicle::note(run, &mut cx.1, format!("Heir {} enters D1, {}.", run.heir, biome_for(1).title()));
    }

    /// Borrow the run and a context together.
    pub fn ctx(&mut self) -> (&mut Run, Ctx<'_>) {
        let Game { run, lineage, events, sim, prov, .. } = self;
        let run = run.as_mut().expect("no live run");
        let set = lineage.active_set.min(lineage.sets.len() - 1);
        let max_rows = lineage.max_rows();
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
            max_rows,
            events,
            sim: *sim,
            prov,
            variant: &lineage.variant,
            hunter: lineage.hunter.as_ref(),
        };
        (run, cx)
    }

    /// Advance the live view by `turns`, returning the events and the new snapshot.
    pub fn step(&mut self, turns: u32) -> StepResult {
        let mut events = Vec::new();
        let mut run_over = false;
        self.watched = true;
        // Camp rest runs on the same clock online (Cut 2 §1); `send` skips it.
        if self.lineage.rest_left > 0 && self.run.as_ref().is_none_or(|r| r.turn == 0) {
            let used = self.rest_tick(turns);
            if self.lineage.rest_left > 0 || used == turns {
                let snapshot = self.ensure_run();
                let exit_pending = self.exit_pending_wire();
                return StepResult { events, snapshot, run_over: false, exit_pending };
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
            return StepResult { events, snapshot, run_over: true, exit_pending };
        }
        for _ in 0..turns {
            self.tick();
            events.append(&mut self.events);
            if self.run.as_ref().is_none_or(|r| r.over.is_some()) {
                run_over = true;
                break;
            }
        }
        let snapshot = self.snapshot();
        if run_over {
            self.finish_run();
            events.append(&mut self.events);
            // Cut 6 §1: the exit event carries the settled ledger line.
            if let Some(line) = self.last_exit.take() {
                if let Some(Ev::Exit { line: l, trace, .. }) = events.iter_mut().rev().find(|e| matches!(e, Ev::Exit { .. })) {
                    // Cut 9 §5: the exit event carries the last-5 trace beside the line.
                    *trace = line.trace.clone();
                    *l = Some(ExitLine { trace: None, ..line });
                }
            }
        }
        self.last_snapshot = Some(snapshot.clone());
        let exit_pending = if run_over { self.exit_pending_wire() } else { None };
        StepResult { events, snapshot, run_over, exit_pending }
    }

    pub fn exit_pending_wire(&self) -> Option<ExitPending> {
        self.pending_exit.as_ref().map(|p| ExitPending {
            items: p.items.iter().map(|i| to_inv(i, &self.lineage.facts, &self.lineage.flavours)).collect(),
            tier: p.tier.name().into(),
            worth: salvage_coins(&p.items, p.pct, self.lineage.gold_carry),
            auto_keep: auto_keep_plan(p, &self.lineage.vault, self.lineage.vault_slots(), &self.lineage.keep_pref, self.lineage.unlocks.contains("quartermaster")).0,
        })
    }

    /// One turn of the live run. Records history for verdicts when not simulating.
    pub fn tick(&mut self) {
        if self.run.as_ref().is_none_or(|r| r.over.is_some()) {
            return;
        }
        if !self.sim && self.run.as_ref().unwrap().turn.is_multiple_of(HISTORY_STRIDE) {
            let r = self.run.as_ref().unwrap();
            self.history.push_back((r.clone(), self.lineage.facts.clone()));
            while self.history.len() > HISTORY_TURNS + 1 {
                self.history.pop_front();
            }
        }
        let (run, mut cx) = self.ctx();
        let before = cx.events.len();
        crate::turn::tick(run, &mut cx);
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

    /// Run the live expedition to its end (used by forecasts and the offline batch).
    pub fn run_to_end(&mut self, max_turns: u32) {
        let mut n = 0;
        while self.run.as_ref().is_some_and(|r| r.over.is_none()) && n < max_turns {
            self.tick();
            n += 1;
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let l = &self.lineage;
        let run = self.run.as_ref().expect("no live run");
        let m = &run.floor.map;
        let h = &run.hero;
        let hero = HeroSnap {
            entity: Entity {
                id: HERO_ID,
                kind: format!("hero_{}", h.class.name()),
                name: None,
                x: h.pos.x,
                y: h.pos.y,
                hp: h.hp,
                max_hp: h.max_hp,
                tags: h.status_tags(),
                ally: None,
                telegraph: None,
                cid: None,
                remembered: false,
            },
            inv: h.inv.iter().map(|i| to_inv(i, &l.facts, &l.flavours)).collect(),
            weapon: h.weapon.as_ref().map(|w| w.kind.clone()),
            armour: h.armour.as_ref().map(|a| a.kind.clone()),
            class: h.class.name().into(),
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
            run.loot.max(0) * tier.pct() / 100
        });
        Snapshot {
            depth: run.depth,
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
            loot: run.loot,
            run: RunRef { id: run.id, heir: run.heir, started_turn: run.started_turn },
            stake: Stake { loot: run.loot, brought, return_row, kept, stalling: run.stuck_fires > 0 },
            vision: run.vision(&l.unlocks),
            vault_choice: run.vault_choice.as_ref().map(|(t0, items)| VaultChoice { items: items.iter().map(|i| to_inv(i, &l.facts, &l.flavours)).collect(), left: (t0 + VAULT_GRACE).saturating_sub(run.turn) }),
            room: Some(run.room_ref()),
            rooms: Some(run.floor.rooms.len() as u32),
            floor_twist: run.floor_twist.as_deref().map(|t| crate::situations::twist_word(t).to_string()),
        }
    }

    /// Bank the run's outcome into the lineage: marks, xp, renown, companions, deaths, rest.
    /// The vault decision (Addendum D) is left pending; `keep` or `auto_keep` finalises it.
    /// Cut 2 §2: yield follows the exit — bank 1.0 / return 0.6 / death 0.0 for gold, salvage,
    /// class XP and renown; a dead heir's kit stays on the floor as bones.
    pub fn finish_run(&mut self) -> Option<RunOutcome> {
        let run = self.run.take()?;
        let tier = run.over.unwrap_or(ExitTier::Return);
        // Yield follows the exit (Cut 2 §2); a timed-out run yields nothing.
        let pct: i32 = if run.timed_out { 0 } else { tier.pct() };
        // Cut 13 §1: a stall (the guard fired `STALL_FIRES` times on one floor) is a run the
        // player can read: it gets a death-style record below.
        let stalled = run.timed_out && run.stuck_fires >= STALL_FIRES;
        let t = run.turn;
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
        self.batch.stalls += stalled as u32;
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
            passed: run.passed.clone(),
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
        for (i, n) in run.row_fired.iter().enumerate() {
            if i < MAX_ROWS {
                self.batch.row_fired[i] += n;
                if *n > 0 {
                    self.batch.row_runs[i] += 1;
                }
            }
        }
        // Camp rest (Cut 2 §1): as long as the expedition, capped; a death is a fixed wake.
        let rest = self.rest_after(run.turn, tier);
        self.lineage.rest_left = rest;
        self.events.push(Ev::Rest { t, seconds: rest.div_ceil(crate::offline::TICKS_PER_SECOND as u32) });
        // Marks (Cut 2 §2): new depth, boss, trophy, rank. First kills stay in bests and the ledger.
        let mut marks = 0;
        let mut wake_top = 0;
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
        // Cut 15 §1: a bank near the frontier pays a mark — from depth ≥ the lineage's best
        // *before this run* − 1 (so a new-best bank qualifies, on top of its depth marks). A
        // return, a shallower bank and a stall (timed out) pay nothing.
        let frontier = tier == ExitTier::Bank && !run.timed_out && run.max_depth > 0 && run.max_depth + 1 >= self.lineage.best_depth;
        if frontier {
            marks += 1;
            self.batch.frontier_banks += 1;
        }
        if run.max_depth > self.lineage.best_depth {
            marks += run.max_depth - self.lineage.best_depth;
            self.lineage.best_depth = run.max_depth;
            bests.push(format!("D{}", run.max_depth));
        }
        // Cut 4: the first bank from each depth is a mark (a distinct best from first reach).
        if tier == ExitTier::Bank && !run.timed_out && self.lineage.banked_depths.insert(run.max_depth) {
            marks += 1;
            bests.push(format!("home:D{}", run.max_depth));
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
                self.lineage.lost.push(Lost { kind: rec.kind.clone(), name: rec.name.clone(), gen: rec.gen, heir });
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
        let loot_kept = run.loot.max(0) * pct / 100;
        let gold_before = self.lineage.gold;
        let exit_why = match tier {
            _ if run.timed_out && run.stuck_fires >= STALL_FIRES => format!("stalled D{}", run.max_depth),
            _ if run.timed_out => format!("lost thread D{}", run.max_depth),
            ExitTier::Bank => format!("banked D{}", run.max_depth),
            ExitTier::Return => format!("returned D{}", run.max_depth),
            ExitTier::Death => format!("died D{}", run.depth),
        };
        self.lineage.gold_move(loot_kept, &exit_why);
        self.batch.gold_earned += loot_kept;
        let mut all: Vec<Item> = run.hero.inv.clone();
        if let Some(w) = &run.hero.weapon {
            all.push(w.clone());
        }
        if let Some(a) = &run.hero.armour {
            all.push(a.clone());
        }
        all.retain(|i| i.cat() != Cat::Gold && i.id != 1 && !matches!(i.kind.as_str(), "bones" | "trap"));
        if tier == ExitTier::Death {
            // Insured brought items come home (Melvor insurance); everything else stays on the
            // floor as a bones pile for a later heir.
            let (insured, rest): (Vec<Item>, Vec<Item>) = all.into_iter().partition(|i| run.brought.contains(&i.id) && self.lineage.insured.contains(&i.id));
            for it in insured {
                self.lineage.insured.retain(|id| *id != it.id);
                self.lineage.vault.push(it);
            }
            all = Vec::new();
            if !rest.is_empty() {
                let named = !self.batch.bests.is_empty() || !run.boss_kills.is_empty();
                self.lineage.bones.push(Bones { heir: run.heir, depth: run.depth, items: rest.clone(), named });
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
            all = rest.into_iter().filter(|i| !(i.kind == "leash" && i.free && run.supplies.contains(&i.id))).collect();
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
        }
        // A return keeps the dearest 60 %; what the vault sent along is the player's already
        // and comes first, whatever it is worth (QA on 952e306: a vaulted poison potion,
        // cheapest in the pack, was cut on a bail — `VAULT 1/1 → 0/1` with no line).
        all.sort_by(|a, b| run.brought.contains(&b.id).cmp(&run.brought.contains(&a.id)).then(b.value().cmp(&a.value())).then(a.id.cmp(&b.id)));
        let n_keep = match tier {
            ExitTier::Bank => all.len(),
            ExitTier::Return => (all.len() * 60).div_ceil(100),
            ExitTier::Death => 0,
        };
        let eligible: Vec<Item> = all.iter().take(n_keep).cloned().collect();
        let rest_items: Vec<Item> = all.into_iter().skip(n_keep).collect();
        let cut_coins = self.salvage(&rest_items, pct);
        // Per kind, the coins the cut brought — the ledger's own (QA on 92eb880: `sold
        // aggravate $2` on the keep sheet, `aggravate ×1 · $1` in the report).
        let mut cut: std::collections::BTreeMap<String, (u32, i32)> = std::collections::BTreeMap::new();
        for (it, c) in rest_items.iter().zip(&cut_coins) {
            let e = cut.entry(it.kind.clone()).or_insert((0, 0));
            e.0 += 1;
            e.1 += c;
        }
        let cut_rows: Vec<crate::wire::SalvageRow> = cut.into_iter().map(|(k, (n, c))| crate::wire::SalvageRow { kind: k, n, gold: c }).filter(|r| r.gold > 0).collect();
        let brought: Vec<u32> = eligible.iter().filter(|i| run.brought.contains(&i.id)).map(|i| i.id).collect();
        self.pending_exit = Some(PendingExit { run_id: run.id, tier, items: eligible, pct, brought });
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
                let name = crate::descent::grudge_name(&mut self.lineage.rng);
                self.lineage.grudges.push(Grudge { kind: cause.clone(), name, depth: run.depth, heir: run.heir });
            }
            // Cut 5 §2: the heir's line in the lineage chronicle.
            let bones_left = self.lineage.bones.last().is_some_and(|b| b.heir == run.heir);
            let tail = bones_left.then(|| format!("left bones on D{}", run.depth));
            self.lineage.chronicle_heir(&format!("fell to {}", crate::sifter::cause_phrase(&cause)), tail);
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
                wake_top = top;
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
        let spent_on = self.restock();
        // Cut 8B §3: the kennel's leash is back on the shelf while nothing has been tamed.
        self.lineage.kennel_leash();
        // Cut 6 §1: the ledger line — carried × keep% → kept, what the automations spent on
        // coming home, and where the kit went on a death.
        let spent: i32 = self.lineage.gold_ledger.iter().rev().take_while(|g| g.t == self.lineage.total_turns).filter(|g| g.delta < 0 && !g.why.starts_with("salvage")).map(|g| -g.delta).sum();
        let bones_n = if tier == ExitTier::Death { self.lineage.bones.last().filter(|b| b.heir == run.heir).map(|b| b.items.len()).unwrap_or(0) } else { 0 };
        let unused = run.hero.inv.iter().filter(|i| run.supplies.contains(&i.id) && i.kind != "leash").count();
        let mut line = exit_line_of(run.loot.max(0), pct, loot_kept, spent, spent_on, tier, run.timed_out, run.stuck_fires >= STALL_FIRES, unused, bones_n, run.depth);
        // What the run earned besides gold, on its own line (QA on 50bb162: `◆7` and `$40` at
        // the camp after a death with no source on the death screen).
        // Cut 15 §1: a frontier bank's mark is part of the total and named once: `◆+1
        // frontier` alone, `◆+3 (1 frontier)` beside a new depth's marks.
        if marks > 0 {
            match (frontier, marks) {
                (true, 1) => line.text.push_str(" · ◆+1 frontier"),
                (true, _) => line.text.push_str(&format!(" · ◆+{marks} (1 frontier)")),
                _ => line.text.push_str(&format!(" · ◆+{marks}")),
            }
        }
        if wake_top > 0 {
            line.text.push_str(&format!(" · +${wake_top} wake"));
        }
        // Cut 9 §5: every exit carries its last five hero turns (read off the run's own trace
        // ring: nothing more per tick).
        line.trace = Some(exit_trace(&run, &self.prov));
        line.salvaged = cut_rows;
        line.run_id = run.id;
        line.xp = xp;
        line.level_ups = level_ups;
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
        self.last_exit = Some(line);
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
        let Some(p) = self.pending_exit.take() else { return Err("nothing to keep".into()) };
        let slots = self.lineage.vault_slots();
        // QA on 92eb880 (qaM: the sheet's `aggravate $2`, the report's `aggravate ×1 · $1`): an
        // unkept item pays exactly the coins the sheet showed for it (`ExitPending.worth`).
        let worth = salvage_coins(&p.items, p.pct, self.lineage.gold_carry);
        let mut paid: Vec<(Item, i32)> = Vec::new();
        let mut salvage: Vec<Item> = Vec::new();
        for (it, coin) in p.items.into_iter().zip(worth) {
            // `bones_only`: nothing enters the vault; only bones piles carry gear.
            if !ids.contains(&it.id) || slots == 0 {
                paid.push((it, coin));
                continue;
            }
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
                        paid.push((d, coin));
                        continue;
                    }
                    salvage.push(d);
                }
            }
            self.batch.found.push(v);
        }
        let (items, coins): (Vec<Item>, Vec<i32>) = paid.into_iter().unzip();
        self.salvage_paid(&items, p.pct, &coins);
        self.salvage(&salvage, p.pct);
        self.restock();
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
        let (ids, evict) = auto_keep_plan(p, &self.lineage.vault, self.lineage.vault_slots(), &self.lineage.keep_pref, self.lineage.unlocks.contains("quartermaster"));
        if !evict.is_empty() {
            // The replaced vault items are salvaged with the exit's unkept finds.
            let out: Vec<Item> = self.lineage.vault.iter().filter(|v| evict.contains(&v.id)).cloned().collect();
            self.lineage.vault.retain(|v| !evict.contains(&v.id));
            if let Some(p) = self.pending_exit.as_mut() {
                p.items.extend(out);
            }
        }
        let _ = self.keep(ids);
    }

    pub fn vocabulary(&self) -> Vocabulary {
        crate::tokens::vocabulary(&self.lineage)
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
        let e = self.lineage.eggs.remove(i);
        self.hatch_egg(e);
        Ok(())
    }

    pub fn companion_vocabulary(&self, id: u32) -> Result<Vocabulary, String> {
        let c = self.lineage.all_companions().find(|c| c.id == id).ok_or("no such companion")?;
        Ok(crate::tokens::companion_vocabulary(&self.lineage, c))
    }

    // ---- Gold and supplies (Addendum B, forge Addendum D)

    pub fn supply_catalogue(&self) -> Vec<SupplyInfo> {
        let mut out = vec![SupplyInfo { kind: "leash".into(), price: 30, label: "leash".into() }];
        for d in crate::defs::ITEMS {
            let base = match d.cat {
                Cat::Potion => Some(40),
                Cat::Scroll => Some(60),
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
        if self.lineage.supplies.len() >= self.lineage.supply_cap() {
            return Err(format!("{} supplies max", self.lineage.supply_cap()));
        }
        let entry = self.supply_catalogue().into_iter().find(|s| s.kind == kind).ok_or("not for sale")?;
        if self.lineage.gold < entry.price {
            return Err("not enough gold".into());
        }
        self.lineage.gold_move(-entry.price, &kind.replace('_', " "));
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
            self.lineage.kennel_declined = false;
        }
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
        let why = format!("insure {}", it.kind.replace('_', " "));
        self.lineage.gold_move(-cost, &why);
        self.lineage.insured.push(id);
        Ok(())
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
        if !s.free {
            if let Some(e) = self.supply_catalogue().iter().find(|e| e.kind == s.kind) {
                let why = format!("refund {}", s.kind.replace('_', " "));
                self.lineage.gold_move(e.price, &why);
            }
            if let Some(k) = self.lineage.last_supplies.iter().position(|k| *k == s.kind) {
                self.lineage.last_supplies.remove(k);
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
            if let Some(e) = cat.iter().find(|e| e.kind == s.kind) {
                let why = format!("refund {}", s.kind.replace('_', " "));
                self.lineage.gold_move(e.price, &why);
            }
        }
        // Cut 4: clearing the shelf is an order; the automation does not undo it.
        self.lineage.last_supplies.clear();
    }

    /// `auto_supply` (Cut 2 §3): an empty shelf is restocked with the last expedition's kinds
    /// as far as gold allows. Cut 4: runs when the hero comes home (`finish_run`, and again
    /// after the vault decision brought the salvage in), so the camp's shelf shows the restock
    /// before the next send — restocking only at `start_run` moved the supplies straight into
    /// the pack and the shelf never showed them.
    pub fn restock(&mut self) -> Vec<String> {
        if !self.lineage.unlocks.contains("auto_supply") || self.lineage.supplies.iter().any(|s| !s.free) {
            return Vec::new();
        }
        let mut bought = Vec::new();
        // Cut 13 §3: a kind the last run used to no effect is not rebought (rater R: "the
        // strength potion the trait drinks at full HP is rebought sixteen times").
        let wasted = self.lineage.last_wasted.clone();
        for kind in self.lineage.last_supplies.clone() {
            if wasted.contains(&kind) {
                continue;
            }
            let gold = self.lineage.gold;
            if self.buy_supply(&kind).is_ok() {
                bought.push(kind.replace('_', " "));
                let e = self.batch.spent.entry(kind.clone()).or_insert((0, 0));
                e.0 += 1;
                e.1 += gold - self.lineage.gold;
            }
        }
        bought
    }
}

/// Cut 6 §1: the exit's ledger line, ≤ 14 words (`$84 carried · return keeps 60% → $50`;
/// death: `$144 carried · death keeps 0% → $0 · bones: 7 items on D5`; a run that hit the cap:
/// `lost thread keeps 0%`).
/// Cut 9 §5: the last `EXIT_TRACE_LEN` hero turns of a run, from its trace ring. Cut 11 §3:
/// plus the run's provenance log (every `because` event), when it has one.
pub fn exit_trace(run: &Run, prov: &[crate::provenance::Prov]) -> Trace {
    Trace { turns: run.trace.iter().rev().take(EXIT_TRACE_LEN).rev().cloned().collect(), provenance: crate::provenance::all(prov) }
}

#[allow(clippy::too_many_arguments)]
pub fn exit_line(carried: i32, keep_pct: i32, kept: i32, spent: i32, spent_on: Vec<String>, tier: ExitTier, timed_out: bool, bones: usize, depth: u32) -> ExitLine {
    exit_line_of(carried, keep_pct, kept, spent, spent_on, tier, timed_out, false, 0, bones, depth)
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
    let verb = match tier {
        ExitTier::Bank => "banked",
        ExitTier::Return => "returned",
        ExitTier::Death => "died",
    };
    let mut text = format!("{verb} ${kept} · ${carried} carried · keeps {keep_pct}%");
    if timed_out {
        text.push_str(if stalled { " · stalled" } else { " · lost thread" });
    }
    if bones > 0 {
        text.push_str(&format!(" · bones: {bones} items on D{depth}"));
    }
    if unused > 0 && tier != ExitTier::Death {
        text.push_str(&format!(" · {unused} {} back", if unused == 1 { "supply" } else { "supplies" }));
    }
    ExitLine { carried, keep_pct, kept, spent, spent_on, text, trace: None, salvaged: Vec::new(), run_id: 0, xp: 0, level_ups: 0 }
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
            run.monsters.push(Monster::spawn(id, kind, pos, depth));
        }
    }
    // Boss with escorts, near the down stairs. Cut 7 §1: the D5 lieutenant is placed the same
    // way (he holds the stairs), with goblins round him.
    if let Some(boss) = boss_for(depth).or_else(|| crate::descent::lieutenant_for(depth)) {
        let near: Vec<Pos> = run
            .floor
            .open_tiles()
            .into_iter()
            .filter(|p| p.cheb(run.floor.stairs_down) <= 3 && !run.occupied(*p) && p.cheb(hero) > 6)
            .collect();
        let pos = if near.is_empty() { take(&mut run.rng, &open, &mut cursor) } else { *run.rng.pick(&near) };
        let id = run.new_id();
        run.monsters.push(Monster::spawn(id, boss, pos, depth));
        let escort = match boss {
            "goblin_warlord" | "goblin_captain" => "goblin",
            "bloat_mother" => "bloat",
            "foundry_master" => "smith",
            "lurker_queen" => "lurker",
            "mirror_king" => "mirror_shade",
            _ => "skeleton",
        };
        // The Captain keeps a thinner guard than a boss.
        let guard = if crate::defs::monster_def(boss).boss { 40 } else { 15 };
        for q in pos.neighbours8() {
            if run.floor.map.passable(q) && !run.occupied(q) && run.rng.chance(guard) {
                let id = run.new_id();
                run.monsters.push(Monster::spawn(id, escort, q, depth));
            }
        }
    }
    // Grudge monsters live on the floor they killed on; the `hunted` stalker is on every floor
    // from D3, awake and already on the hero's trail.
    let hunted = hunter.filter(|_| depth >= 3);
    for g in grudges.iter().filter(|g| g.depth == depth).chain(hunted) {
        let pos = take(&mut run.rng, &open, &mut cursor);
        if run.occupied(pos) {
            continue;
        }
        let id = run.new_id();
        let mut m = Monster::spawn(id, &g.kind, pos, depth);
        m.make_grudge(&g.name);
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
            if depth == crate::descent::biome_first(Biome::Foundry) {
                vec!["fire", "poison", "caustic", "bow"]
            } else if boss_for(depth).is_some() {
                // The Master's own stockpile, for whoever reaches him.
                vec!["fire", "fire", "poison", "caustic", "bow"]
            } else {
                vec!["fire", "poison", if depth.is_multiple_of(2) { "caustic" } else { "poison" }]
            }
        }
        Biome::Deep => {
            if depth == crate::descent::biome_first(Biome::Deep) {
                vec!["silence", "silence", "lantern", "regen"]
            } else if boss_for(depth).is_none() {
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
        if keep() {
            place(run, it);
        }
    }
}

pub fn monster_entity(mo: &Monster, facts: &BTreeSet<String>) -> Entity {
    let mut tags: Vec<String> = mo.tags().into_iter().filter(|t| mo.ally || facts.contains(&format!("foe:{}:{}", mo.kind, t))).collect();
    if mo.neutral {
        tags.push("captive".into());
    }
    if mo.stun > 0 {
        tags.push("stunned".into());
    }
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
    }
}

/// Spawn the party's companions next to the hero at the start of a run.
pub fn spawn_party(run: &mut Run, party: &[Companion]) {
    let hp = run.hero.pos;
    for c in party {
        let free = hp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
        let Some(q) = free else { continue };
        let id = run.new_id();
        let m = companion_monster(id, c, q);
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
    m.max_hp = c.max_hp.max(1);
    m.hp = m.max_hp;
    let base: Vec<&str> = m.def().tags.to_vec();
    m.extra_tags = c.tags.iter().filter(|t| !base.contains(&t.as_str())).cloned().collect();
    m
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
    let heirs: Vec<u32> = run.bones.iter().filter(|b| b.depth == depth && !run.bones_found.contains(&b.heir)).map(|b| b.heir).collect();
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
    run.last_twist = run.floor_twist.take();
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

/// `auto_keep`'s decision: the pending ids that go to the vault and the vault ids they replace
/// (see `Game::auto_keep` for the precedence). Never plans past `slots`.
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
