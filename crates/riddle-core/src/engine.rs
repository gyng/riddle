//! The game: lineage state, the live run, and the public API mirrored by the wasm bridge.
use crate::defs::{spawn_table, Cat};
use crate::descent::{biome_for, boss_for, Biome, Grudge, ENDING_DEPTH};
use crate::gen::{generate, Floor};
use crate::geom::Pos;
use crate::hero::{mastery_card, xp_to_next, Class, Hero, Trait};
use crate::item::{describe, to_inv, Flavours, FloorItemWire, InvItem, Item};
use crate::monster::Monster;
use crate::rng::{hash_str, Rng};
use crate::rules::{RuleSet, Vocabulary};
use crate::tiles::{Overlay, VISION};
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
/// A run that cannot finish in this many ticks (≈ 67 min at 1×) comes home empty-handed
/// (tier `return`, yield ×0: a stalemate is not a policy).
pub const MAX_TURNS_PER_RUN: u32 = 40_000;
/// Energy needed to act; actors gain `speed` per tick.
pub const ACT_ENERGY: i32 = 100;
pub const TICKS_PER_TURN: u32 = 10;
pub const MAX_LEVEL: u32 = 10;
/// Gold from loot and salvage is divided by this (Addendum B/D economy pass).
pub const GOLD_DIVISOR: i32 = 4;
/// Cut 2 §1: camp rest after an expedition lasts as long as it did, capped at 30 min and never
/// shorter than the wake (a two-minute `hp<50 → return` sortie would otherwise farm hundreds
/// of runs a day); a death is followed by a fixed 20-minute wake. Ticks (10/s).
pub const REST_CAP_TICKS: u32 = 30 * 60 * 10;
pub const REST_MIN_TICKS: u32 = 20 * 60 * 10;
pub const WAKE_TICKS: u32 = 20 * 60 * 10;
/// Cut 2 §1: an egg hatches after this many rests (1 with the incubator).
pub const EGG_RESTS: u32 = 3;
/// Cut 2 §2: bones piles kept per lineage (oldest expires).
pub const BONES_MAX: usize = 3;
/// Cut 2 §5: kills of a kind before it counts as studied.
pub const STUDIED_KILLS: u32 = 5;

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
    /// (turn, kind, depth)
    pub kills: Vec<(u32, String, u32)>,
    pub brought: Vec<u32>,
    pub trace: Vec<TraceTurn>,
    pub notes: Vec<(u32, String)>,
    pub over: Option<ExitTier>,
    pub death_cause: Option<String>,
    pub death_blow: i32,
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
    /// Oscillation guard: rows that target foes are suppressed until this action.
    #[serde(default)]
    pub stuck_until: u32,
    /// Trait pre-emption clock: the action of the last trait deviation.
    #[serde(default)]
    pub trait_last: Option<u32>,
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
}

impl Run {
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
    pub fn ignore(&mut self, id: u32, actions: u32) {
        let until = self.actions + actions;
        self.ignored.insert(id, until);
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
}

impl LineageState {
    pub fn new(seed: u64) -> LineageState {
        let mut rng = Rng::derive(seed, hash_str("lineage"));
        let flavours = Flavours::roll(&mut rng);
        let trait_ = Trait::ALL[rng.below(4) as usize];
        let mut classes = BTreeMap::new();
        for c in Class::ALL {
            classes.insert(c.name().to_string(), ClassProg { level: 1, xp: 0 });
        }
        LineageState {
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
        }
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
            classes: self.classes.clone(),
            forge: self.forge.clone(),
            renown: self.renown,
            rank: self.rank,
            keep_pref: self.keep_pref.clone(),
            insured: self.insured.clone(),
            rest_left_s: self.rest_left.div_ceil(crate::offline::TICKS_PER_SECOND as u32),
            bones: self.bones.iter().map(|b| BonesPile { depth: b.depth, heir: b.heir, items: b.items.len() as u32 }).collect(),
        }
    }
    pub fn vault_slots(&self) -> usize {
        1 + ["vault2", "vault3", "vault4"].iter().filter(|u| self.unlocks.contains(**u)).count()
    }
    pub fn max_rows(&self) -> usize {
        4 + ["row5", "row6", "row7", "row8"].iter().filter(|u| self.unlocks.contains(**u)).count()
    }
    pub fn party_slots(&self) -> u32 {
        1 + self.unlocks.contains("party_slot_2") as u32 + self.unlocks.contains("party_slot_3") as u32
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
    /// Bestiary ledger derived from facts and breeding.
    pub fn ledger(&self) -> Vec<LedgerRow> {
        crate::defs::MONSTERS
            .iter()
            .filter(|m| !m.boss && !m.tags.contains(&"summoned"))
            .map(|m| {
                let seen = self.facts.contains(&format!("foe:{}", m.kind));
                let known = seen && m.tags.iter().all(|t| self.facts.contains(&format!("foe:{}:{}", m.kind, t)));
                LedgerRow {
                    kind: m.kind.to_string(),
                    seen,
                    known,
                    tamed: self.facts.contains(&format!("tamed:{}", m.kind)),
                    bred: self.bred.contains(m.kind),
                    studied: self.studied(m.kind),
                }
            })
            .collect()
    }
    pub fn all_companions(&self) -> impl Iterator<Item = &Companion> {
        self.party.iter().chain(self.kennel.iter())
    }
    /// Forge tier of a kind (Addendum D): +tier to every future copy.
    pub fn forge_tier(&self, kind: &str) -> i32 {
        self.forge.get(kind).map(|f| f.tier as i32).unwrap_or(0)
    }
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
}

fn default_pct() -> i32 {
    60
}

/// Accumulated outcomes since the last return report.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Batch {
    pub runs: u32,
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
    pub renderable_events: u32,
    pub turns: u32,
    /// Real (simulated) runs: (final depth, death cause).
    pub run_outcomes: Vec<(u32, Option<String>)>,
    /// Ticks per real run.
    pub run_ticks: Vec<u32>,
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
            reel: Vec::new(),
            last_snapshot: None,
            events: Vec::new(),
            stall_runs: 0,
            pending_exit: None,
            batch: Batch::default(),
            facts_at_run_start: 0,
            max_deaths: 40,
            offline: false,
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
            reel: Vec::new(),
            last_snapshot: None,
            events: Vec::new(),
            stall_runs: 0,
            pending_exit: None,
            batch: Batch::default(),
            facts_at_run_start: self.lineage.facts.len(),
            max_deaths: 0,
            offline: false,
        }
    }

    pub fn lineage(&self) -> Lineage {
        self.lineage.to_wire()
    }

    /// Rows beyond the unlocked count are dropped (a patch inserted into a full set pushes
    /// the last row out, which is what the player would do); the rest is validated.
    pub fn set_rules(&mut self, set: RuleSet) -> Result<(), String> {
        let mut set = set;
        set.rows.truncate(self.lineage.max_rows());
        set.validate()?;
        let i = self.lineage.active_set.min(self.lineage.sets.len() - 1);
        self.lineage.sets[i] = set;
        Ok(())
    }

    pub fn select_set(&mut self, i: usize) {
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

    pub fn loadout(&mut self, ids: Vec<u32>) {
        self.loadout = ids.into_iter().filter(|id| self.lineage.vault.iter().any(|v| v.id == *id)).collect();
    }

    pub fn run_seed(&self, run_id: u32) -> u64 {
        crate::rng::splitmix(self.lineage.seed ^ (run_id as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    /// Start (or resume) an expedition. The player chose to go: any camp rest left is skipped.
    pub fn send(&mut self) -> Snapshot {
        self.lineage.rest_left = 0;
        self.ensure_run()
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
                let _ = self.insure(id);
            }
        }
        if self.lineage.unlocks.contains("auto_supply") && self.lineage.supplies.is_empty() {
            for kind in self.lineage.last_supplies.clone() {
                let _ = self.buy_supply(&kind);
            }
        }
        self.lineage.last_supplies = self.lineage.supplies.iter().map(|i| i.kind.clone()).collect();
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
            brought,
            trace: Vec::new(),
            notes: Vec::new(),
            over: None,
            death_cause: None,
            death_blow: 0,
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
            row_fired: vec![0; 8],
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
            stuck_until: 0,
            trait_last: None,
            pickup_streak: 0,
            pickup_inv: 0,
            items_until: 0,
            known_foes: BTreeMap::new(),
            row_streak: (-9, 0),
            row_suppressed: (-9, 0),
            rests: 0,
            aimed: false,
            bones: self.lineage.bones.clone(),
            bones_found: Vec::new(),
            kills_counted: 0,
            timed_out: false,
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
        populate_floor(&mut run, &self.lineage.grudges, &self.lineage.forge);
        place_bones(&mut run);
        spawn_party(&mut run, &self.lineage.party);
        run.floor.map.update_vision(run.hero.pos, VISION);
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
        let Game { run, lineage, events, sim, .. } = self;
        let run = run.as_mut().expect("no live run");
        let set = lineage.active_set.min(lineage.sets.len() - 1);
        let max_rows = lineage.max_rows();
        let cx = Ctx {
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
        };
        (run, cx)
    }

    /// Advance the live view by `turns`, returning the events and the new snapshot.
    pub fn step(&mut self, turns: u32) -> StepResult {
        let mut events = Vec::new();
        let mut run_over = false;
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
        }
        self.last_snapshot = Some(snapshot.clone());
        let exit_pending = if run_over { self.exit_pending_wire() } else { None };
        StepResult { events, snapshot, run_over, exit_pending }
    }

    fn exit_pending_wire(&self) -> Option<ExitPending> {
        self.pending_exit.as_ref().map(|p| ExitPending {
            items: p.items.iter().map(|i| to_inv(i, &self.lineage.facts, &self.lineage.flavours)).collect(),
            tier: p.tier.name().into(),
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
        self.lineage.total_turns += 1;
        if self.run.as_ref().unwrap().depth >= ENDING_DEPTH {
            self.lineage.ended = true;
        }
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
            },
            inv: h.inv.iter().map(|i| to_inv(i, &l.facts, &l.flavours)).collect(),
            weapon: h.weapon.as_ref().map(|w| w.kind.clone()),
            armour: h.armour.as_ref().map(|a| a.kind.clone()),
            class: h.class.name().into(),
            trait_: run.trait_.name().into(),
        };
        let entities = run
            .monsters
            .iter()
            .filter(|mo| mo.hp > 0 && (m.is_visible(mo.pos) || mo.ally))
            .map(|mo| monster_entity(mo, &l.facts))
            .collect();
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
        let return_row = l.rules().rows.iter().take(l.max_rows()).position(|r| matches!(r.verb.v.as_str(), "return" | "bank"));
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
            stake: Stake { loot: run.loot, brought, return_row },
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
        self.batch.run_outcomes.push((run.depth, if tier == ExitTier::Death { run.death_cause.clone() } else { None }));
        self.batch.run_ticks.push(run.turn);
        self.batch.renderable_events += run.renderable_events;
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
        if self.batch.row_fired.len() < 8 {
            self.batch.row_fired = vec![0; 8];
        }
        for (i, n) in run.row_fired.iter().enumerate() {
            if i < 8 {
                self.batch.row_fired[i] += n;
            }
        }
        // Camp rest (Cut 2 §1): as long as the expedition, capped; a death is a fixed wake.
        let rest = crate::offline::rest_after(run.turn, tier);
        self.lineage.rest_left = rest;
        self.events.push(Ev::Rest { t, seconds: rest.div_ceil(crate::offline::TICKS_PER_SECOND as u32) });
        // Marks (Cut 2 §2): new depth, boss, trophy, rank. First kills stay in bests and the ledger.
        let mut marks = 0;
        let mut bests: Vec<String> = Vec::new();
        if run.max_depth > self.lineage.best_depth {
            marks += run.max_depth - self.lineage.best_depth;
            self.lineage.best_depth = run.max_depth;
            bests.push(format!("D{}", run.max_depth));
        }
        for (_, kind, _) in &run.kills {
            if kind.starts_with("spectral_") {
                continue; // summons are not bests
            }
            if self.lineage.kills.insert(kind.clone()) {
                let boss = crate::defs::monster_def(kind).boss;
                if boss {
                    marks += 3;
                }
                bests.push(if boss { format!("boss: {kind}") } else { format!("first kill: {kind}") });
            }
        }
        for tr in &run.trophies_run {
            if !self.lineage.trophies.contains(tr) {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {tr}"));
            }
        }
        // Class XP (Addendum C, Cut 2 §2): only banked and returned runs feed it.
        let raw: u32 = run.kills.iter().map(|(_, _, d)| 2 + d).sum::<u32>() / 4 + 3 * run.max_depth;
        let xp = raw * pct as u32 / 100;
        let class = self.lineage.class;
        let prog = self.lineage.classes.entry(class.name().into()).or_insert(ClassProg { level: 1, xp: 0 });
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
                bests.push(format!("trophy: {tr}"));
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
                bests.push(format!("trophy: {tr}"));
            }
        }
        for biome in [Biome::Warrens, Biome::Fens, Biome::Crypt] {
            let tr = format!("ledger:{}", biome.name());
            if !self.lineage.trophies.contains(&tr)
                && crate::defs::biome_kinds(biome).iter().all(|k| self.lineage.facts.contains(&format!("tamed:{k}")))
            {
                self.lineage.trophies.push(tr.clone());
                marks += 2;
                bests.push(format!("trophy: {tr}"));
            }
        }
        // Bones recovered this run (Cut 2 §2): the piles leave the lineage; their items are now
        // the hero's and follow the exit like anything else.
        let mut highlights = crate::sifter::sift(&run, &self.lineage);
        for heir in &run.bones_found {
            if let Some(i) = self.lineage.bones.iter().position(|b| b.heir == *heir) {
                let b = self.lineage.bones.remove(i);
                self.batch.bones_found.push(format!("heir {} · D{} · {} items", b.heir, b.depth, b.items.len()));
                if b.named {
                    let text = format!("Recovered heir {}'s bones on D{}.", b.heir, b.depth);
                    highlights.push(Highlight { pattern: "bones".into(), score: crate::sifter::BONES, t, run_id: run.id, text });
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
        let loot_kept = run.loot.max(0) * pct / 100;
        self.lineage.gold += loot_kept / GOLD_DIVISOR;
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
            all.retain(|i| !run.supplies.contains(&i.id));
        }
        all.sort_by(|a, b| b.value().cmp(&a.value()).then(a.id.cmp(&b.id)));
        let n_keep = match tier {
            ExitTier::Bank => all.len(),
            ExitTier::Return => (all.len() * 60).div_ceil(100),
            ExitTier::Death => 0,
        };
        let eligible: Vec<Item> = all.iter().take(n_keep).cloned().collect();
        let rest_items: Vec<Item> = all.into_iter().skip(n_keep).collect();
        self.salvage(&rest_items, pct);
        self.pending_exit = Some(PendingExit { run_id: run.id, tier, items: eligible, pct });
        // Death: graveyard, grudge, heir, record.
        if tier == ExitTier::Death {
            let cause = run.death_cause.clone().unwrap_or_else(|| "unknown".into());
            *self.batch.deaths.entry(cause.clone()).or_insert(0) += 1;
            if run.depth >= self.batch.worst_depth {
                self.batch.worst_depth = run.depth;
                self.batch.worst_death = Some(run.id);
            }
            let mut deeds: Vec<String> = self.batch.bests.iter().rev().take(3).cloned().collect();
            for (_, k) in &run.boss_kills {
                let d = format!("slew the {}", crate::defs::monster_def(k).title);
                if !deeds.contains(&d) {
                    deeds.push(d);
                }
            }
            deeds.truncate(3);
            self.lineage.graveyard.push(Grave { heir: run.heir, depth: run.depth, cause: cause.clone(), deeds });
            if crate::defs::MONSTERS.iter().any(|m| m.kind == cause && !m.boss && !m.tags.contains(&"summoned"))
                && !self.lineage.grudges.iter().any(|g| g.kind == cause && g.depth == run.depth)
            {
                let name = crate::descent::grudge_name(&mut self.lineage.rng);
                self.lineage.grudges.push(Grudge { kind: cause.clone(), name, depth: run.depth, heir: run.heir });
            }
            self.lineage.heir += 1;
            self.lineage.trait_ = Trait::ALL[self.lineage.rng.below(4) as usize];
            if !self.sim {
                let rec = crate::trace::death_record(self, &run);
                self.deaths.insert(run.id, rec);
                while self.deaths.len() > self.max_deaths {
                    let k = *self.deaths.keys().next().unwrap();
                    self.deaths.remove(&k);
                }
            }
        }
        self.history.clear();
        outcome.new_facts = (self.lineage.facts.len().saturating_sub(self.facts_at_run_start)) as u32;
        if !self.offline {
            self.settle_renown(t);
        }
        if self.sim {
            self.auto_keep();
        }
        Some(outcome)
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
    fn salvage(&mut self, items: &[Item], pct: i32) {
        for it in items {
            // hundredths of a coin: value × tier% ÷ divisor, carried so cheap items still add up
            let cents = salvage_value(&it.kind) * pct / GOLD_DIVISOR;
            self.lineage.gold_carry += cents;
            let gold = self.lineage.gold_carry / 100;
            self.lineage.gold_carry %= 100;
            self.lineage.gold += gold;
            let f = self.lineage.forge.entry(it.kind.clone()).or_default();
            f.salvaged += it.amount.max(1) as u32;
            f.craftable = f.salvaged >= 5;
            f.tier = if f.salvaged >= 40 {
                2
            } else if f.salvaged >= 15 {
                1
            } else {
                0
            };
            let e = self.batch.salvaged.entry(it.kind.clone()).or_insert((0, 0));
            e.0 += 1;
            e.1 += cents;
        }
    }

    /// Finalise the exit's vault choice: chosen ids go to the vault, the rest are salvaged.
    pub fn keep(&mut self, ids: Vec<u32>) -> Result<(), String> {
        let Some(p) = self.pending_exit.take() else { return Err("nothing to keep".into()) };
        let slots = self.lineage.vault_slots();
        let mut salvage: Vec<Item> = Vec::new();
        for it in p.items {
            if !ids.contains(&it.id) {
                salvage.push(it);
                continue;
            }
            let mut v = it;
            if v.id < 100_000 {
                v.id = self.lineage.next_vault_id;
                self.lineage.next_vault_id += 1;
            }
            self.lineage.vault.push(v.clone());
            self.lineage.vault.sort_by(|a, b| b.value().cmp(&a.value()).then(a.id.cmp(&b.id)));
            if self.lineage.vault.len() > slots {
                if let Some(d) = self.lineage.vault.pop() {
                    if d.id == v.id {
                        salvage.push(d);
                        continue;
                    }
                    salvage.push(d);
                }
            }
            self.batch.found.push(v);
        }
        self.salvage(&salvage, p.pct);
        Ok(())
    }

    /// Resolve a pending exit by `keep_pref` (offline runs, or when the client moves on).
    /// The `quartermaster` automation (Cut 2 §3) keeps the best weapon *and* armour.
    pub fn auto_keep(&mut self) {
        let Some(p) = self.pending_exit.as_ref() else { return };
        let pick = |cat: Cat| p.items.iter().filter(|i| i.cat() == cat).max_by_key(|i| (i.value(), i.id)).map(|i| i.id);
        let ids: Vec<u32> = if self.lineage.unlocks.contains("quartermaster") {
            pick(Cat::Weapon).into_iter().chain(pick(Cat::Armour)).collect()
        } else {
            match self.lineage.keep_pref.as_str() {
                "best_weapon" => pick(Cat::Weapon).or_else(|| pick(Cat::Armour)).into_iter().collect(),
                "best_armour" => pick(Cat::Armour).or_else(|| pick(Cat::Weapon)).into_iter().collect(),
                _ => Vec::new(),
            }
        };
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
        self.lineage.gold -= 50;
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
            let price = match (base, craftable, identified) {
                (Some(p), _, true) => p,
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
        self.lineage.gold -= entry.price;
        let id = self.lineage.next_vault_id;
        self.lineage.next_vault_id += 1;
        let mut it = Item::new(id, kind);
        it.enchant = self.lineage.forge_tier(kind);
        if kind == "leash" {
            it.amount = 1;
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
        self.lineage.gold -= cost;
        self.lineage.insured.push(id);
        Ok(())
    }

    pub fn clear_supplies(&mut self) {
        let cat = self.supply_catalogue();
        for s in std::mem::take(&mut self.lineage.supplies) {
            if let Some(e) = cat.iter().find(|e| e.kind == s.kind) {
                self.lineage.gold += e.price;
            }
        }
    }
}

/// Salvage value per kind (Addendum D).
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
pub fn populate_floor(run: &mut Run, grudges: &[Grudge], forge: &BTreeMap<String, ForgeRow>) {
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
        let anchor = if kind == "eel" && !water.is_empty() { *run.rng.pick(&water) } else { take(&mut run.rng, &open, &mut cursor) };
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
    // Boss with escorts, near the down stairs.
    if let Some(boss) = boss_for(depth) {
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
            "goblin_warlord" => "goblin",
            "bloat_mother" => "bloat",
            _ => "skeleton",
        };
        for q in pos.neighbours8() {
            if run.floor.map.passable(q) && !run.occupied(q) && run.rng.chance(40) {
                let id = run.new_id();
                run.monsters.push(Monster::spawn(id, escort, q, depth));
            }
        }
    }
    // Grudge monsters live on the floor they killed on.
    for g in grudges.iter().filter(|g| g.depth == depth) {
        let pos = take(&mut run.rng, &open, &mut cursor);
        if run.occupied(pos) {
            continue;
        }
        let id = run.new_id();
        let mut m = Monster::spawn(id, &g.kind, pos, depth);
        m.make_grudge(&g.name);
        run.monsters.push(m);
    }
    // Items.
    let budget = crate::defs::item_budget(depth);
    let kinds: Vec<&crate::defs::ItemDef> = crate::defs::ITEMS.iter().filter(|i| i.weight > 0).collect();
    let iw: Vec<u32> = kinds
        .iter()
        .map(|i| {
            let mut w = i.weight;
            match i.kind {
                "axe" | "bow" | "mail" if depth < 4 => w = 0,
                "plate" if depth < 8 => w = 0,
                _ => {}
            }
            w
        })
        .collect();
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
        place(run, it);
    }
    // The Fens stock the Bloat Mother's answer: three throwables per floor.
    if biome == Biome::Fens {
        for k in ["fire", "poison", if depth.is_multiple_of(2) { "fire" } else { "poison" }] {
            let iid = run.new_item_id();
            let mut it = Item::new(iid, k);
            it.enchant = forge.get(k).map(|f| f.tier as i32).unwrap_or(0);
            place(run, it);
        }
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
        it.amount = run.rng.range(3, 7) * depth as i32;
        place(run, it);
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
