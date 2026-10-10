//! Cut 30.5 (docs/CUT30_5.md, docs/AUTOMATION_TREE.md): the works tree — start manual, automate. Each
//! chore the town asks of the player is done by hand a few times, then a worker is hired for it (the
//! first, the porter, free; the rest for gold in forge units). The owner's answer (3): **manual send
//! first** — until the scout is hired a send is one run (the hero comes home and waits). The haul waits
//! in a chest until the porter: it never caps, never decays, and the engine's own needs draw on it.
//! The four Cut 30 tracks are the tree's branches (their stages are its `stage` nodes).
use crate::engine::{Game, LineageState};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A node of the tree (a worker).
pub struct NodeDef {
    pub id: &'static str,
    /// ≤ 2 words.
    pub name: &'static str,
    /// trunk · character · items · scale · town.
    pub branch: &'static str,
    /// The act by hand that counts toward it (`""`: given).
    pub chore: &'static str,
    /// Chores by hand that light it.
    pub need: u32,
    /// Its price in tenths of a forge unit (0: free).
    pub price_tenths: u32,
    /// The lineage age (h) that lights it without the count, once its chore exists (0: none).
    pub fallback_h: u32,
    /// The system (systems.rs) its chore needs (`""`: from the start).
    pub gate: &'static str,
    /// Where its worker stands on the town scene.
    pub post: &'static str,
    /// The hire's beat (≤ 2 words, caps) and the tip (≤ 10 words).
    pub beat: &'static str,
    pub tip: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn node(id: &'static str, name: &'static str, branch: &'static str, chore: &'static str, need: u32, price_tenths: u32, fallback_h: u32, gate: &'static str, post: &'static str, beat: &'static str, tip: &'static str) -> NodeDef {
    NodeDef { id, name, branch, chore, need, price_tenths, fallback_h, gate, post, beat, tip }
}

/// The tree in its order (the trunk, then the branches): one node lit at a time, the first ready one.
pub const NODES: &[NodeDef] = &[
    node("quartermaster", "quartermaster", "trunk", "", 0, 0, 0, "", "crate", "", "packs heal · drill item"),
    node("porter", "porter", "trunk", "chest", 3, 0, 0, "", "mouth", "AUTO HAUL", "hauls home · while away"),
    node("scout", "scout", "trunk", "send", 3, 5, 0, "", "fire", "AUTO SEND", "sends him · each rest"),
    node("armourer", "armourer", "trunk", "wear", 2, 10, 24, "storehouse", "storehouse", "AUTO EQUIP", "wears better finds"),
    node("apprentice", "apprentice", "trunk", "forge", 3, 30, 30, "forge", "blacksmith", "AUTO FORGE", "buys forge steps"),
    node("keeper", "keeper", "items", "keep", 2, 20, 30, "storehouse", "storehouse", "AUTO KEEP", "sorts finds · never asks"),
    node("clerk", "clerk", "town", "deposit", 3, 40, 36, "bank", "bank", "AUTO BANK", "banks spare gold"),
    node("drillmaster", "drillmaster", "character", "level", 2, 30, 36, "", "tent", "AUTO LEVEL", "levels the stance"),
    node("kennel_hand", "kennel-hand", "town", "field", 2, 30, 40, "kennel", "kennel", "AUTO PETS", "fields best pets"),
    node("herald", "herald", "town", "swap", 2, 20, 40, "quests", "board", "AUTO QUEST", "swaps stale quests"),
    node("guide", "guide", "scale", "start", 3, 40, 44, "start", "mouth", "AUTO START", "starts deeper"),
    // Cut 120 §5: the kennel keeper carries the Cut 119 breeding order (`StandingSwitches.kennel`); before his hire the
    // breeding is by hand (`Game::breed`), the order waits
    node("kennel_keeper", "kennel keeper", "town", "breed", 2, 30, 48, "kennel", "kennel", "AUTO BREED", "breeds pets · by order"),
];

/// The apprentice and the clerk keep this many forge units in the purse (the shelf's money).
pub const RESERVE_UNITS: i32 = 3;
/// The guide starts sends at the deepest lit stone this many floors under the record.
pub const GUIDE_GAP: u32 = 4;
/// Cut 30.5, week 2 (the owner, 2026-10-02: later worker upgrades): a worker's ranks II and III come these many
/// days of service after its hire, each bought with gold — the worker's look (and its post's) on the town scene.
pub const RANK_DAYS: [u32; 3] = [4, 8, 12];
pub const MAX_RANK: u32 = 4;
/// The chores of a save from before the tree: its workers up to the scout are hired (no player regresses).
pub const LEGACY: [&str; 3] = ["quartermaster", "porter", "scout"];

pub fn def(id: &str) -> Option<&'static NodeDef> {
    NODES.iter().find(|n| n.id == id)
}

/// The lineage's tree (`LineageState::tree`). `v` 0 is a save from before it (`save::load` maps it).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tree {
    #[serde(default)]
    pub v: u32,
    /// Workers hired (id, the day).
    #[serde(default)]
    pub hired: crate::shared::Shared<Vec<(String, u32)>>,
    /// Chores done by hand, per node.
    #[serde(default, skip_serializing_if = "crate::shared::map_is_empty")]
    pub done: crate::shared::Shared<BTreeMap<String, u32>>,
    /// Hired workers switched off.
    #[serde(default, skip_serializing_if = "crate::shared::set_is_empty")]
    pub paused: crate::shared::Shared<BTreeSet<String>>,
    /// Cut 111: the wall tactics the drillmaster has put on (once each: a tactic the player takes off stays off).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub walls_worn: BTreeSet<String>,
    /// The haul not yet collected (part of `LineageState::gold`; the purse is the rest).
    #[serde(default)]
    pub chest: i32,
    /// A send by hand is under way (before the scout).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub sent: bool,
    /// Vault items worn by hand (each counts once toward the armourer).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub worn: Vec<u32>,
    /// Every gold movement summed (interest in, bank moves out): purse + chest + bank = ledger.
    #[serde(default)]
    pub ledger: i64,
    /// Each worker's acts in all (the report's `first`).
    #[serde(default, skip_serializing_if = "crate::shared::map_is_empty")]
    pub acts: crate::shared::Shared<BTreeMap<String, u32>>,
    /// Week 2: each promoted worker's rank (2–3; a hired worker absent here is rank 1).
    #[serde(default, skip_serializing_if = "crate::shared::map_is_empty")]
    pub ranks: crate::shared::Shared<BTreeMap<String, u32>>,
    /// The guide's ledger: per waystone started from, the recent sends and those that brought gold home
    /// (halved past `STONE_MEMORY` sends); and the start last picked by hand.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub stones: BTreeMap<u32, (u32, u32)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_by_hand: Option<(u32, u32)>,
    /// The record the stones' ledger was kept under (a new record forgets it: a stronger hero tries again).
    #[serde(default)]
    pub stones_best: u32,
    /// The armourer's ledger: the recent sends and those that died (halved past `STONE_MEMORY`).
    #[serde(default)]
    pub sends: (u32, u32),
    /// Blind 1fb7786: under the `half` forge order, the purse the apprentice may still spend — half of each
    /// haul home since he was hired, less what he spent (never above the purse).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub forge_budget: i32,
    /// Cut 114 §3: the scout's wall ledger (`note_wall`) — the floor the last heirs died on near the record.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wall: Option<WallLedger>,
    /// Cut 114 §3: the floor the scout last banked before (the report's `banked before Queen ×4`).
    #[serde(default, skip_serializing_if = "is_zero_u")]
    pub held_at: u32,
    /// Cut 120 §2 (Cut 114 §5): each worker's rank-II perk chosen (`PERKS`; absent: the first, the old bonus).
    #[serde(default, skip_serializing_if = "crate::shared::map_is_empty")]
    pub perks: crate::shared::Shared<BTreeMap<String, String>>,
}

fn is_zero_u(n: &u32) -> bool {
    *n == 0
}

/// Cut 114 §3 (blind 77030eb, A: "long absences paid less than short ones … $1 496, four dead heirs and the
/// best depth unchanged"): the floor the sends die on — deaths there in a row (a send past it forgets it), the
/// hero's strength at the last (`strength`), and the sends the scout has banked before it since.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WallLedger {
    pub depth: u32,
    pub deaths: u32,
    pub strength: u64,
    pub held: u32,
}

/// Deaths on one floor before the scout banks the sends at its stairs (order `bank`).
pub const WALL_DEATHS: u32 = 2;
/// A death this many floors above the record or nearer is a wall's (deeper on the walk is not).
pub const WALL_NEAR: u32 = 2;
/// Sends banked before an unchanged wall before the scout lets one try it again.
pub const WALL_RETRY: u32 = 3;
/// The `acts` keys of the sends the scout banked before a wall, of those whose haul he carried home from its
/// stairs, and of that haul in all (never a node's).
pub const SCOUT_HELD: &str = "scout:held";
pub const SCOUT_CARRIED: &str = "scout:carried";
pub const SCOUT_CARRIED_GOLD: &str = "scout:carried$";

/// What the hero brings to a wall that a camp can change: the forge's steps, the class level, the Legacy, the
/// worn packages and their levels, the rules. A wall is tried again when any of it moved.
pub fn strength(l: &LineageState) -> u64 {
    let kit: Vec<u32> = crate::kit::KIT_SLOTS.iter().map(|s| crate::kit::owned(l, s)).collect();
    let legacy = crate::legacy::current(l).map(|b| format!("{:?}", b.upgrades)).unwrap_or_default();
    let pkgs: Vec<(String, u32)> = std::iter::once(l.pkg.stance.clone()).chain(l.pkg.tactics.iter().cloned()).chain(l.pkg.temperament.iter().cloned()).map(|id| { let n = l.pkg.level(&id); (id, n) }).collect();
    crate::rng::hash_str(&format!("{kit:?}|{}|{legacy}|{pkgs:?}|{}", l.class_level(), crate::forecast::rules_key(l.rules())))
}

/// A wall's name on the report and the exit (`Queen`; `D27` on a floor with no boss).
pub fn wall_name(route: crate::descent::Route, depth: u32) -> String {
    route.boss(depth).map(|k| crate::sifter::boss_short(k).to_string()).unwrap_or_else(|| format!("D{depth}"))
}

/// The scout's order for his next send at a wall — `WALL_DEATHS` heirs dead on one floor in a row: (the floor,
/// banked at its stairs). `carry`: the haul carried home from the stairs, every send. `bank`: the heir banked
/// there while he is no stronger than at the last death and fewer than `WALL_RETRY` sends were banked before it
/// since — the next tries it again, its haul carried. `push`: none.
/// Cut 118, owner amendment: the scout's wall order is retired (`wall_hold` is always `None`).
pub const WALL_ORDER_RETIRED: bool = true;

pub fn wall_hold(l: &LineageState) -> Option<(u32, bool)> {
    // Cut 118, owner amendment ("heroes yolo"): the wall order is retired — every send pushes (the record's
    // checkpoints still secure the carry). The ledger below stays for the siege's reads and old saves.
    if WALL_ORDER_RETIRED {
        return None;
    }
    if l.pkg.literal || !on(l, "scout") || l.orders.wall == "push" {
        return None;
    }
    let w = l.tree.wall.as_ref().filter(|w| w.deaths >= WALL_DEATHS && w.depth > l.start.max(1))?;
    // (`bank`: a send that tries the wall again has its haul carried home from the stairs as `carry`'s)
    Some((w.depth, l.orders.wall == "bank" && w.held < WALL_RETRY && w.strength == strength(l)))
}

/// A real run home (never a sim's, never a harness's literal lineage): the scout's wall ledger.
pub fn note_wall(game: &mut Game, run: &crate::engine::Run, tier: crate::engine::ExitTier) {
    if game.sim || game.lineage.pkg.literal {
        return;
    }
    if run.wall_held {
        if let Some(w) = game.lineage.tree.wall.as_mut() {
            w.held += 1;
        }
        game.lineage.tree.held_at = run.depth + 1;
        *game.lineage.tree.acts.entry(SCOUT_HELD.to_string()).or_insert(0) += 1;
        return;
    }
    if run.wall_carried > 0 {
        game.lineage.tree.held_at = run.wall_hold.map_or(run.depth, |(d, _)| d);
        let acts = &mut game.lineage.tree.acts;
        *acts.entry(SCOUT_CARRIED.to_string()).or_insert(0) += 1;
        let g = acts.entry(SCOUT_CARRIED_GOLD.to_string()).or_insert(0);
        *g = g.saturating_add(run.wall_carried as u32);
    }
    let l = &mut game.lineage;
    if tier == crate::engine::ExitTier::Death {
        let d = run.depth;
        if d + WALL_NEAR < l.best_depth {
            return;
        }
        let s = strength(l);
        match l.tree.wall.as_mut() {
            Some(w) if w.depth == d => {
                w.deaths += 1;
                w.strength = s;
                w.held = 0;
            }
            _ => l.tree.wall = Some(WallLedger { depth: d, deaths: 1, strength: s, held: 0 }),
        }
    } else if l.tree.wall.as_ref().is_some_and(|w| run.max_depth > w.depth) {
        l.tree.wall = None;
    }
}

/// Blind 1fb7786: the share of a haul the apprentice's order lets him forge with, in percent (`all` spends the
/// whole spare purse, not a share — `None`).
pub fn forge_share(l: &LineageState) -> Option<i32> {
    match l.orders.forge.as_str() {
        "all" => None,
        "off" => Some(0),
        _ => Some(50),
    }
}

/// The sends a stone's record remembers before it halves.
pub const STONE_MEMORY: u32 = 12;

/// A send from a waystone came home (non-sim): its start's record — whether it brought gold home.
pub fn note_start(l: &mut LineageState, start: u32, paid: bool, died: bool) {
    let s = &mut l.tree.sends;
    s.0 += 1;
    s.1 += died as u32;
    if s.0 > STONE_MEMORY {
        s.0 /= 2;
        s.1 /= 2;
    }
    if l.tree.stones_best != l.best_depth {
        l.tree.stones.clear();
        l.tree.stones_best = l.best_depth;
    }
    if start <= 1 {
        return;
    }
    let e = l.tree.stones.entry(start).or_insert((0, 0));
    e.0 += 1;
    e.1 += paid as u32;
    if e.0 > STONE_MEMORY {
        e.0 /= 2;
        e.1 /= 2;
    }
}

/// A stone pays: untried under this record, too few sends to say, or at least half of its recent sends brought
/// gold home.
pub fn stone_pays(l: &LineageState, s: u32) -> bool {
    l.tree.stones_best != l.best_depth || l.tree.stones.get(&s).is_none_or(|(n, p)| *n < 3 || 2 * p >= *n)
}

/// The guide's start: the deepest lit stone a band (`GUIDE_GAP`) under the record that pays (D1 when none) —
/// a start picked by hand stands while it pays and the record holds (`by_hand`: the hand's own pick, the same rule).
pub fn guide_pick(l: &LineageState, by_hand: bool) -> u32 {
    if let Some((h, _)) = l.tree.start_by_hand.filter(|(h, best)| !by_hand && *h == l.start.max(1) && *best == l.best_depth && stone_pays(l, *h)) {
        return h;
    }
    l.stones().into_iter().filter(|s| s + GUIDE_GAP <= l.best_depth && stone_pays(l, *s)).max().unwrap_or(1)
}

impl Tree {
    /// A new lineage's tree: the quartermaster given, nothing else.
    pub fn fresh() -> Tree {
        Tree { v: 1, hired: vec![("quartermaster".into(), 0)].into(), ..Default::default() }
    }
}

/// Map a save from before the tree: the workers up to the scout hired, the ledger opened.
pub fn upgrade(l: &mut LineageState) {
    if l.tree.v != 0 {
        return;
    }
    let ledger = l.gold as i64 + l.town.bank as i64;
    l.tree = Tree { v: 1, hired: LEGACY.iter().map(|id| (id.to_string(), l.day)).collect(), ledger, ..Default::default() };
}

/// A harness's grant: these workers hired, no trigger or price (tests, the metrics' fresh lineages).
pub fn grant(l: &mut LineageState, ids: &[&str]) {
    for id in ids {
        if !hired(l, id) {
            l.tree.hired.push((id.to_string(), l.day));
        }
        if *id == "porter" {
            l.tree.chest = 0;
        }
    }
}

pub fn hired(l: &LineageState, id: &str) -> bool {
    l.tree.hired.iter().any(|(h, _)| h == id)
}

/// A worker at work: hired and not switched off (a harness's literal lineage has every worker's
/// old behaviour: sends itself, no chest — and no worker acts on it).
pub fn on(l: &LineageState, id: &str) -> bool {
    hired(l, id) && !l.tree.paused.contains(id)
}

/// The hero goes down on his own after his rest (the scout, or a harness's lineage).
pub fn auto_send(l: &LineageState) -> bool {
    l.pkg.literal || on(l, "scout")
}

/// Hauls land in the purse (the porter, or a harness's lineage).
pub fn porter(l: &LineageState) -> bool {
    l.pkg.literal || on(l, "porter")
}

/// The purse: collected gold, what a tap can spend.
pub fn purse(l: &LineageState) -> i32 {
    l.gold - l.tree.chest
}

/// A ledger reason that is a haul (an exit's pay, salvage, passage coins).
pub fn is_haul(why: &str) -> bool {
    crate::engine::is_exit_why(why) || why.starts_with("salvage") || why.starts_with("passage")
}

/// Week 2 (the owner, 2026-10-02: ranks give small real bonuses): a worker at work's rank above I (0 off or at I).
pub fn bonus_rank(l: &LineageState, id: &str) -> u32 {
    if l.pkg.literal || !on(l, id) {
        return 0;
    }
    rank(l, id).saturating_sub(1)
}

/// The porter's haul bonus per rank above I, in percent (II +2 %, III +4 %, IV +6 %).
pub const PORTER_BONUS_PCT: i32 = 2;
/// The apprentice's discount on the steps it buys, per rank above I, in percent.
pub const APPRENTICE_OFF_PCT: u32 = 5;
/// The clerk's extra interest per rank above I, in tenths of a percent a night (2 % → 2.25 · 2.5 · 2.75 %).
pub const CLERK_BONUS_PERMILLE: i32 = 25;
/// The scout's shorter rest per rank above I, in percent.
pub const SCOUT_REST_OFF_PCT: u32 = 5;

/// The rest after a run, the scout's rank taken off (the dayplayer's harnesses: rank I or none, as before) — under
/// his first perk (`shorter rest`) only.
pub fn rest_scaled(l: &LineageState, ticks: u32) -> u32 {
    let r = if perk(l, "scout") == PERKS[1].1[0] { bonus_rank(l, "scout") } else { 0 };
    ticks - ticks * SCOUT_REST_OFF_PCT * r / 100
}

/// Cut 120 §2 (Cut 114 §5, "ranks you choose"): the workers whose rank II offers one of two perks, the first the old
/// fixed bonus (the default): apprentice `cheaper steps` (−5 % steps a rank) or `keeps a reserve` (a forge unit
/// more kept a rank); scout `shorter rest` (−5 % rest a rank) or `safer start` (+5 % max hp at the send a rank).
pub const PERKS: [(&str, [&str; 2]); 2] = [("apprentice", ["cheaper steps", "keeps a reserve"]), ("scout", ["shorter rest", "safer start"])];
/// The scout's `safer start`: max hp at the send per rank above I, in percent.
pub const SAFER_START_PCT: i32 = 5;

/// A worker's perk (its first when unchosen; `""` for a worker without perks).
pub fn perk<'a>(l: &'a LineageState, id: &str) -> &'a str {
    let Some((_, two)) = PERKS.iter().find(|(w, _)| *w == id) else { return "" };
    l.tree.perks.get(id).map(String::as_str).filter(|p| two.contains(p)).unwrap_or(two[0])
}

/// Choose a worker's perk (the chip; it takes effect from rank II).
pub fn set_perk(l: &mut LineageState, id: &str, p: &str) -> Result<(), String> {
    let (_, two) = PERKS.iter().find(|(w, _)| *w == id).ok_or("no perk for this worker")?;
    if !two.contains(&p) {
        return Err("unknown perk".into());
    }
    if !hired(l, id) {
        return Err("not hired".into());
    }
    if p == two[0] {
        l.tree.perks.remove(id);
    } else {
        l.tree.perks.insert(id.to_string(), p.to_string());
    }
    Ok(())
}

/// The scout's `safer start` at a send: the hero's max hp (never on a harness's lineage; rank I none).
pub fn safer_start(l: &LineageState, hero: &mut crate::hero::Hero) {
    if perk(l, "scout") != PERKS[1].1[1] {
        return;
    }
    let r = bonus_rank(l, "scout") as i32;
    let add = hero.max_hp * SAFER_START_PCT * r / 100;
    if add > 0 {
        hero.max_hp += add;
        hero.max_hp_base += add;
        hero.hp += add;
    }
}

/// The apprentice's discount on a step, in percent (his rank, under `cheaper steps`).
pub fn apprentice_off(l: &LineageState) -> u32 {
    if perk(l, "apprentice") == PERKS[0].1[0] { APPRENTICE_OFF_PCT * bonus_rank(l, "apprentice") } else { 0 }
}

/// The purse the apprentice and the clerk keep (`RESERVE_UNITS`, a unit more a rank under `keeps a reserve`).
pub fn reserve(l: &LineageState) -> i32 {
    let extra = if perk(l, "apprentice") == PERKS[0].1[1] { bonus_rank(l, "apprentice") as i32 } else { 0 };
    (RESERVE_UNITS + extra) * crate::kit::unit(l.best_depth) as i32
}

/// A waystone's toll, the guide's rank taken off (II half, III on free).
pub fn toll_scaled(l: &LineageState, toll: i32) -> i32 {
    match bonus_rank(l, "guide") {
        0 => toll,
        1 => toll / 2,
        _ => 0,
    }
}

/// Called by `gold_move` after `gold` moved: the chest takes the haul (before the porter) and gives
/// way to spending after the purse; the ledger sums the movement.
pub fn on_gold(l: &mut LineageState, delta: i32, why: &str) {
    if !why.starts_with("bank ") {
        l.tree.ledger += delta as i64;
    }
    if delta > 0 && is_haul(why) {
        // (the apprentice's share of the haul, under the `half` order)
        if on(l, "apprentice") && !l.pkg.literal {
            if let Some(pct) = forge_share(l) {
                l.tree.forge_budget = l.tree.forge_budget.saturating_add(delta * pct / 100);
            }
        }
        if porter(l) {
            if !l.pkg.literal {
                *l.tree.acts.entry("porter".into()).or_insert(0) += delta as u32;
            }
        } else if !l.town.auto_collect {
            l.tree.chest += delta;
        }
    }
    l.tree.chest = l.tree.chest.min(l.gold.max(0)).max(0);
    // (week 2: the porter's rank adds to every haul he carries)
    let r = bonus_rank(l, "porter") as i32;
    if delta > 0 && r > 0 && is_haul(why) {
        let bonus = delta * PORTER_BONUS_PCT * r / 100;
        if bonus > 0 {
            l.gold_move(bonus, "porter bonus");
        }
    }
}

/// Whether `n`'s chore exists for this lineage (its system open).
pub fn chore_open(l: &LineageState, n: &NodeDef) -> bool {
    n.gate.is_empty() || crate::systems::is_open(l, n.gate) || match n.gate {
        "storehouse" => crate::town::built(l, "storehouse"),
        "forge" => crate::town::built(l, "blacksmith"),
        "bank" => crate::town::built(l, "bank"),
        "kennel" => crate::town::built(l, "kennel"),
        _ => false,
    }
}

pub fn count(l: &LineageState, id: &str) -> u32 {
    l.tree.done.get(id).copied().unwrap_or(0)
}

/// Its trigger is met: the count, or the fallback age with its chore open.
pub fn ready(l: &LineageState, n: &NodeDef) -> bool {
    if n.chore.is_empty() {
        return true;
    }
    chore_open(l, n) && (count(l, n.id) >= n.need || (n.fallback_h > 0 && l.age_h() >= n.fallback_h))
}

/// The one node to buy: the first ready, unhired node in the tree's order.
pub fn lit(l: &LineageState) -> Option<&'static NodeDef> {
    if l.pkg.literal {
        return None;
    }
    NODES.iter().find(|n| !hired(l, n.id) && ready(l, n))
}

/// A hired worker's rank (1–3; 0 unhired).
pub fn rank(l: &LineageState, id: &str) -> u32 {
    if !hired(l, id) {
        return 0;
    }
    l.tree.ranks.get(id).copied().unwrap_or(1)
}

/// The day a hired worker was hired.
fn hire_day(l: &LineageState, id: &str) -> Option<u32> {
    l.tree.hired.iter().find(|(h, _)| h == id).map(|(_, d)| *d)
}

/// The days until a hired worker's next rank comes (0: it is on offer; `None`: at the top, or not hired, or given).
pub fn rank_wait(l: &LineageState, n: &NodeDef) -> Option<u32> {
    let r = rank(l, n.id);
    if r == 0 || r >= MAX_RANK || n.chore.is_empty() {
        return None;
    }
    let due = hire_day(l, n.id)? + RANK_DAYS[(r - 1) as usize];
    Some(due.saturating_sub(l.day))
}

/// A rank's price: a forge unit × the rank less one (a look, not a second hire: week 2's purse barely notices).
pub fn rank_price(l: &LineageState, n: &NodeDef) -> i32 {
    let next = rank(l, n.id) + 1;
    crate::kit::unit_of(l) as i32 * (next as i32 - 1)
}

/// The one rank on offer: the first hired worker (in the tree's order) whose next rank has come — only when no
/// hire is lit (one BUY at a time).
pub fn lit_rank(l: &LineageState) -> Option<&'static NodeDef> {
    if l.pkg.literal || lit(l).is_some() {
        return None;
    }
    NODES.iter().find(|n| rank_wait(l, n) == Some(0))
}

/// Promote the worker whose rank is on offer: its price from the purse, then the chest.
pub fn promote(l: &mut LineageState, id: &str) -> Result<u32, String> {
    let n = def(id).ok_or("unknown worker")?;
    if lit_rank(l).map(|x| x.id) != Some(id) {
        return Err("not on offer".into());
    }
    let p = rank_price(l, n);
    if l.gold < p {
        return Err("not enough gold".into());
    }
    l.gold_move(-p, &format!("hire {id} rank"));
    let r = rank(l, id) + 1;
    l.tree.ranks.insert(id.to_string(), r);
    Ok(r)
}

/// A rank's edge (≤ 3 words) at rank `r` (week 2: small real bonuses; none at I, none for the keeper, the
/// kennel-hand or the herald — their ranks are looks).
pub fn rank_bonus(id: &str, r: u32) -> Option<String> {
    let k = r.saturating_sub(1);
    if k == 0 {
        return None;
    }
    Some(match id {
        "porter" => format!("+{}% hauls", PORTER_BONUS_PCT as u32 * k),
        "scout" => format!("−{}% rest", SCOUT_REST_OFF_PCT * k),
        "apprentice" => format!("−{}% steps", APPRENTICE_OFF_PCT * k),
        "clerk" => format!("{}‰ interest", BANK_PCT_PERMILLE + CLERK_BONUS_PERMILLE as u32 * k),
        "guide" => if k == 1 { "half toll".into() } else { "no toll".into() },
        "drillmaster" => "levels −◆1".into(),
        "armourer" => "insures its finds".into(),
        _ => return None,
    })
}

const BANK_PCT_PERMILLE: u32 = crate::town::BANK_PCT as u32 * 10;

/// A rank's numeral (`II`, `III`).
pub fn numeral(r: u32) -> &'static str {
    match r {
        2 => "II",
        3 => "III",
        4 => "IV",
        _ => "",
    }
}

/// A node's price in gold (forge units, fixed with the forge's).
pub fn price(l: &LineageState, n: &NodeDef) -> i32 {
    ((crate::kit::unit_of(l) * n.price_tenths + 5) / 10) as i32
}

/// A chore done by hand: its node's count (until its worker is hired).
pub fn did(l: &mut LineageState, chore: &str) {
    if l.pkg.literal {
        return;
    }
    if let Some(n) = NODES.iter().find(|n| n.chore == chore) {
        if !hired(l, n.id) && chore_open(l, n) {
            *l.tree.done.entry(n.id.to_string()).or_insert(0) += 1;
        }
    }
}

/// Hire the lit node's worker: its price from the purse, then the chest.
pub fn hire(l: &mut LineageState, id: &str) -> Result<(), String> {
    let n = def(id).ok_or("unknown worker")?;
    if hired(l, id) {
        return Err("hired already".into());
    }
    if lit(l).map(|x| x.id) != Some(id) {
        return Err("not lit".into());
    }
    let p = price(l, n);
    if l.gold < p {
        return Err("not enough gold".into());
    }
    if p > 0 {
        l.gold_move(-p, &format!("hire {id}"));
    }
    let day = l.day;
    l.tree.hired.push((id.to_string(), day));
    if id == "porter" {
        l.tree.chest = 0;
    }
    Ok(())
}

/// The chest into the purse; the gold it held.
pub fn open_chest(l: &mut LineageState) -> Result<i32, String> {
    let g = l.tree.chest;
    if g <= 0 {
        return Err("chest empty".into());
    }
    did(l, "chest");
    l.tree.chest = 0;
    Ok(g)
}

/// Switch a hired worker off (its chore by hand again) or on.
pub fn set_worker(l: &mut LineageState, id: &str, on: bool) -> Result<(), String> {
    if !hired(l, id) {
        return Err("not hired".into());
    }
    if on {
        l.tree.paused.remove(id);
    } else {
        l.tree.paused.insert(id.to_string());
    }
    Ok(())
}

fn act(game: &mut Game, id: &str, n: u32) {
    if n == 0 {
        return;
    }
    *game.lineage.tree.acts.entry(id.to_string()).or_insert(0) += n;
}

/// A new run started by the automatic clock, not by a Send button or a preview.
pub(crate) fn scout_sent(game: &mut Game) {
    if !game.sim && !game.lineage.pkg.literal && on(&game.lineage, "scout") {
        act(game, "scout", 1);
        // Cut 114 §3: his order at a wall rides with the send
        let hold = wall_hold(&game.lineage);
        // (Cut 118: a sought boss or a trial pushes to its floor)
        let hold = hold.filter(|_| game.run.as_ref().is_none_or(|r| crate::feats::goal_of(&game.lineage, r.id).is_none()));
        if let Some(run) = game.run.as_mut().filter(|r| r.turn == 0) {
            run.wall_hold = hold;
        }
    }
}

/// A weapon's worth a blow: its mean hit with its aim (a forged arm's steps), at its pace.
fn blow_worth(w: &crate::item::Item) -> i64 {
    let (lo, hi) = w.atk();
    let hit = if crate::kit::is_kit_id(w.id) { crate::kit::hit_pct(w) as i64 } else { 80 };
    (lo + hi) as i64 * hit * (10 + w.def().speed) as i64
}

/// The workers act continuously (the owner, 2026-10-02): at each hour of an absence, between the sends, as at
/// a send — the armourer's pack is a send's alone.
pub fn at_hour(game: &mut Game) {
    let previous_start = game.lineage.start.max(1);
    workers_act(game, false);
    // An hourly guide move precedes start_run's worker pass. Refresh its quote here
    // so that the next send cannot retain the preceding stone's passage.
    if game.lineage.start.max(1) != previous_start {
        let rules = game.lineage.rules().clone();
        game.passage = crate::forecast::sim_passage(game, &rules);
    }
}

/// The workers' standing orders, at a real send (before the run begins; never in a sim, never on a
/// harness's literal lineage).
pub fn at_send(game: &mut Game) {
    workers_act(game, true);
}

fn workers_act(game: &mut Game, send: bool) {
    if game.sim || game.lineage.pkg.literal {
        return;
    }
    // Cut 120 §4: after the clear, the herald carries the next ascension's dungeon (the scout's sends, the order on)
    if send {
        announce_orders(game);
        ascend_act(game);
    }
    // the herald: a quest drawn on an earlier day and still unkept gives way (the day's free swap)
    if on(&game.lineage, "herald") && crate::town::quests_open(&game.lineage) {
        let l = &game.lineage;
        let today = crate::town::today(l);
        if l.town.quest.as_ref().is_some_and(|q| !q.done && q.day < today) && l.town.swap_day != Some(l.day) && crate::town::swap(&mut game.lineage).is_ok() {
            act(game, "herald", 1);
        }
    }
    // the drillmaster: marks into the worn stance's next level, then the tactics'
    if on(&game.lineage, "drillmaster") {
        let mut n = 0;
        let ids: Vec<String> = std::iter::once(game.lineage.pkg.stance.clone()).chain(game.lineage.pkg.tactics.iter().cloned()).collect();
        for id in ids {
            while crate::packages::level_price(&game.lineage, &id).is_some_and(|m| m <= game.lineage.marks) {
                let price = crate::packages::level_price(&game.lineage, &id).unwrap_or(0);
                if crate::packages::spend_level(&mut game.lineage, &id).is_err() {
                    break;
                }
                // (week 2: from rank II the drillmaster's level costs a mark less, never under one)
                if bonus_rank(&game.lineage, "drillmaster") > 0 && price > 1 {
                    game.lineage.marks += 1;
                }
                n += 1;
            }
        }
        // Cut 111: a wall's counter arrived while the slots stand open — the drillmaster puts it on (never over a
        // tactic the player wears; revocable as any tactic; counted as his act on the return)
        let slots = crate::packages::tactic_slots(&game.lineage);
        for id in crate::packages::WALL_TACTICS {
            let l = &game.lineage;
            if l.pkg.tactics.len() >= slots || !l.pkg.owned.contains(id) || l.tree.walls_worn.contains(id) || l.pkg.tactics.iter().any(|t| t == id) {
                continue;
            }
            let slot = l.pkg.tactics.len();
            if crate::packages::equip(&mut game.lineage, id, slot).is_ok() {
                game.lineage.tree.walls_worn.insert(id.to_string());
                n += 1;
            }
        }
        act(game, "drillmaster", n);
    }
    // the apprentice: the cheapest next step the purse pays with the reserve kept — under his forge order
    // (blind 1fb7786: `all` the spare purse; `half` half of each haul home, the rest the player's; `off` none)
    if on(&game.lineage, "apprentice") && forge_share(&game.lineage) != Some(0) {
        let mut n = 0;
        let share = forge_share(&game.lineage);
        game.lineage.tree.forge_budget = game.lineage.tree.forge_budget.min(purse(&game.lineage).max(0));
        loop {
            let l = &game.lineage;
            let reserve = reserve(l);
            let Some((slot, p)) = crate::kit::ladders(l).iter().filter(|x|crate::kit::KIT_SLOTS.contains(&x.slot.as_str())).filter_map(|x| x.next.as_ref().map(|s| (x.slot.clone(), s.price as i32))).min_by_key(|x| x.1) else { break };
            let off = apprentice_off(l);
            let before = game.lineage.gold;
            let price = p - p * off as i32 / 100;
            if purse(l) < price + reserve || share.is_some() && l.tree.forge_budget < price || crate::kit::buy_step_off(&mut game.lineage, &slot, off).is_err() {
                break;
            }
            // blind c4705f9 (A, B: `purse −$12562` with no word of what went): each step's slot and its price, for the report
            let paid = (before - game.lineage.gold).max(0) as u32;
            if share.is_some() {
                game.lineage.tree.forge_budget -= paid as i32;
            }
            let acts = &mut game.lineage.tree.acts;
            let spent = acts.entry(APPRENTICE_SPENT.to_string()).or_insert(0);
            *spent = spent.saturating_add(paid);
            *acts.entry(format!("{APPRENTICE_SLOT}{slot}")).or_insert(0) += 1;
            n += 1;
        }
        // Cut 118 §4: after Kit complete, his sinks under their order — a ration a send, the tithe on the hour
        n += crate::feats::apprentice_sinks(game, send);
        act(game, "apprentice", n);
    }
    // the clerk: the purse above the next forge step and the reserve into the bank
    if on(&game.lineage, "clerk") && crate::town::built(&game.lineage, "bank") {
        let l = &game.lineage;
        let next = crate::kit::ladders(l).iter().filter(|x|crate::kit::KIT_SLOTS.contains(&x.slot.as_str())).filter_map(|x| x.next.as_ref().map(|s| s.price as i32)).min().unwrap_or(0);
        let spare = purse(l) - next - reserve(l);
        if spare > 0 {
            if let Ok(d) = crate::town::deposit(&mut game.lineage, spare) {
                act(game, "clerk", d as u32);
            }
        }
    }
    // the kennel-hand: empty party slots filled with the kennel's best
    if on(&game.lineage, "kennel_hand") {
        let l = &game.lineage;
        let slots = l.party_slots() as usize;
        if l.party.len() < slots && !l.kennel.is_empty() {
            let mut ids: Vec<u32> = l.party.iter().map(|c| c.id).collect();
            let mut k: Vec<_> = l.kennel.iter().filter(|c| !ids.contains(&c.id)).collect();
            // (Cut 119: a kind the party already holds last, lamed pets never — the line's pets stay a mix)
            if l.pets.off {
                k.sort_by_key(|c| (std::cmp::Reverse(c.level), c.id));
            } else {
                k.retain(|c| c.life.lame == 0);
                k.sort_by_key(|c| (l.party.iter().any(|p| p.kind == c.kind), std::cmp::Reverse(c.level), c.id));
            }
            let add: Vec<u32> = k.iter().take(slots - ids.len()).map(|c| c.id).collect();
            let n = add.len() as u32;
            ids.extend(add);
            if game.set_party_by(ids, false).is_ok() {
                act(game, "kennel_hand", n);
            }
        }
    }
    // Cut 120 §1–2, at a send (the camp step before the run, where the clerk banks; the run takes both): each worker's
    // rank as it comes, the reserve kept (`ranks: auto`), and the Legacy under its order. (Never on the hour: a sliced
    // absence settles a run's end and the last hour's pass in another order than a whole one — the purchase would move.)
    if send {
        ranks_act(game);
        legacy_act(game);
    }
    // the guide: the deepest lit stone a band under the record whose sends bring gold home (a stone that stops
    // paying is given up for the next one up; a start picked by hand stands while it pays)
    if on(&game.lineage, "guide") {
        let pick = guide_pick(&game.lineage, false);
        if pick != game.lineage.start.max(1) && game.lineage.set_start(pick).is_ok() {
            act(game, "guide", 1);
        }
    }
    // the armourer: the vault's best weapon and armour go with each send — each only when it beats the forged kit
    // the hero would wear (a weaker one rides in the pack and takes a find's slot)
    // (and only while the sends come home: at a wall, where half of them die, the find stays safe in the storehouse —
    // a death's insurance is paid again at every send, and an uninsured find is lost to the bones)
    // (week 2: from rank II the armourer insures what it brings itself — it brings it at a wall too)
    let (n, died) = game.lineage.tree.sends;
    if send && on(&game.lineage, "armourer") && !game.lineage.vault.is_empty() && (4 * died < n.max(1) || bonus_rank(&game.lineage, "armourer") > 0) {
        let mut add = Vec::new();
        let mut kit = crate::hero::Hero::new(game.lineage.class, crate::geom::Pos::new(0, 0));
        crate::kit::equip(&game.lineage, &mut kit);
        for cat in [crate::defs::Cat::Weapon, crate::defs::Cat::Armour] {
            if let Some(it) = game.lineage.vault.iter().filter(|v| v.cat() == cat).max_by_key(|v| (v.value(), std::cmp::Reverse(v.id))) {
                // (truly better: a blow's worth with its aim and its pace, or more armour at no drag — the
                // pack's own choice weighs the blow alone, and a found arm over a forged one lost the aim)
                let better = match cat {
                    crate::defs::Cat::Weapon => kit.weapon.as_ref().is_none_or(|k| blow_worth(it) > blow_worth(k)),
                    _ => it.def().speed >= 0 && kit.armour.as_ref().is_none_or(|k| it.def_bonus() > k.def_bonus()),
                };
                if better && !game.loadout.contains(&it.id) {
                    add.push(it.id);
                }
            }
        }
        if !add.is_empty() && !game.lineage.variant_is("bones_only") {
            if bonus_rank(&game.lineage, "armourer") > 0 {
                for id in &add {
                    if !game.lineage.insured.contains(id) {
                        game.lineage.insured.push(*id);
                    }
                }
            }
            act(game, "armourer", add.len() as u32);
            game.loadout.extend(add);
        }
    }
}

// ---------------------------------------------------------------- Cut 120: the orders that fill the automation

/// Cut 120 §1: the Legacy order (`StandingSwitches.legacy`): `balanced` (health → damage → armour in turn, then the
/// effects in the offer's order) · a focus (`health` · `damage` · `armour`: that one to its cap, then balanced) · `off`.
/// A save's default is `off`; a new lineage's `NEW_LEGACY`.
pub const LEGACY_ORDERS: [&str; 5] = ["balanced", "health", "damage", "armour", "off"];
/// Cut 120 §2: the ranks order (`StandingSwitches.ranks`): `auto` buys each worker's rank when it comes and the purse
/// pays it with the reserve (and a lit hire's price) kept · `off`. A save's default is `off`; a new lineage's `NEW_RANKS`.
pub const RANKS_ORDERS: [&str; 2] = ["auto", "off"];
/// Cut 120 §4: the ascend order (`StandingSwitches.ascend`): `off` (the default) · `on` — after the clear the herald
/// takes the scout's next send into the next ascension's dungeon (`Game::carry_on`). Never without the order.
pub const ASCEND_ORDERS: [&str; 2] = ["off", "on"];
/// Owner option B (2026-10-10): a new lineage's Legacy and ranks orders are `off`, as a save's and the harnesses'
/// (`Session::new`, a new bloodline, `Game::new_resident` all alike), so the bars measure what a new player gets; the
/// client offers each order once when its rung lights. (Option C — `balanced` · `auto` for new lineages — let IDLE
/// slay the King by day 12 on 16/16 seeds and compressed PICKED/TUNED/RANDOM; docs/CUT120_AUTOMATION_FILL.md.)
pub const NEW_LEGACY: &str = "off";
pub const NEW_RANKS: &str = "off";
/// The Legacy order a new lineage starts with (`NEW_LEGACY`).
pub fn new_legacy() -> &'static str {
    NEW_LEGACY
}
/// The orders `same for all` copies to every bloodline (`StandingSwitches.shared`).
pub const SHAREABLE: [&str; 9] = ["insure", "forge", "wall", "sink", "heir", "kennel", "legacy", "ranks", "ascend"];

/// The `acts` keys of the order lines (never a node's): the ranks bought (`ranks`, per worker `ranks:scout`, their
/// gold `ranks$`); the Legacy bought (`legacy`, per upgrade `legacy:health`, the points `legacy◆`); the herald's
/// ascensions (`herald:ascend`); the kennel keeper's eggs and releases (`kennel_keeper:bred`, `kennel_keeper:freed`);
/// the bank the apprentice's tithe drew (`apprentice:drew$`).
pub const RANKS: &str = "ranks";
pub const RANKS_GOLD: &str = "ranks$";
pub const LEGACY_ACT: &str = "legacy";
pub const LEGACY_POINTS: &str = "legacy◆";
pub const HERALD_ASCEND: &str = "herald:ascend";
pub const KEEPER_BRED: &str = "kennel_keeper:bred";
pub const KEEPER_FREED: &str = "kennel_keeper:freed";
pub const APPRENTICE_DREW: &str = "apprentice:drew$";
/// The apprentice's sinks' gold (rations, tithes), beside his forge steps' (`APPRENTICE_SPENT`).
pub const APPRENTICE_SINKS: &str = "apprentice:sinks$";

pub(crate) fn add_act(l: &mut LineageState, key: &str, n: u32) {
    if n == 0 {
        return;
    }
    let e = l.tree.acts.entry(key.to_string()).or_insert(0);
    *e = e.saturating_add(n);
}

/// Each order announced once, as a drill is, at the first worker pass it stands at (`LEGACY BALANCED`): a beat of
/// the report and a line of the news.
fn announce_orders(game: &mut Game) {
    let l = &game.lineage;
    let mut say: Vec<(String, String)> = Vec::new();
    if l.orders.legacy != "off" {
        say.push((format!("legacy:{}", l.orders.legacy), format!("legacy {}", l.orders.legacy)));
    }
    if l.orders.ranks == "auto" && NODES.iter().any(|n| hired(l, n.id) && !n.chore.is_empty()) {
        say.push(("ranks:auto".into(), "ranks auto".into()));
    }
    if l.orders.ascend == "on" && l.ended {
        say.push(("ascend:on".into(), "herald ascends".into()));
    }
    let day = crate::feats::day_of(game.lineage.clock_s);
    for (key, text) in say {
        if game.lineage.orders.announced.insert(key) {
            game.batch.pkg_lines.push(text.to_uppercase());
            crate::feats::news(&mut game.lineage, "order", text, day);
        }
    }
}

/// The ranks order: each hired worker at work whose rank has come (in the tree's order), bought from the purse while
/// it keeps the reserve and the lit hire's price. The ranks bought.
fn ranks_act(game: &mut Game) -> u32 {
    if game.lineage.orders.ranks != "auto" {
        return 0;
    }
    let mut n = 0;
    for node in NODES {
        let l = &game.lineage;
        if !on(l, node.id) || rank_wait(l, node) != Some(0) {
            continue;
        }
        let p = rank_price(l, node);
        let keep = reserve(l) + lit(l).map_or(0, |h| price(l, h));
        if p <= 0 || purse(l) < p + keep {
            continue;
        }
        game.lineage.gold_move(-p, &format!("hire {} rank", node.id));
        let r = rank(&game.lineage, node.id) + 1;
        game.lineage.tree.ranks.insert(node.id.to_string(), r);
        let l = &mut game.lineage;
        add_act(l, RANKS, 1);
        add_act(l, RANKS_GOLD, p as u32);
        add_act(l, &format!("{RANKS}:{}", node.id), 1);
        n += 1;
    }
    n
}

/// The Legacy upgrade the order buys next (`None`: none it waits for is affordable yet).
pub fn legacy_pick(l: &LineageState) -> Option<String> {
    let order = l.orders.legacy.as_str();
    if order == "off" {
        return None;
    }
    let offers = crate::legacy::offers(l, false);
    let open = |u: &&crate::wire::LegacyUpgrade| u.rank < u.cap;
    let get = |id: &str| offers.iter().find(|u| u.id == id);
    // a focus: that upgrade to its cap first
    if crate::legacy::IDS.contains(&order) {
        if let Some(u) = get(order).filter(open) {
            return u.affordable.then(|| u.id.clone());
        }
    }
    // balanced: the next of the three in turn (the lowest rank, in their order) or the next effect in the offer's order
    // (a fork's first branch; one shut by depth, a parent or the other branch is passed), whichever costs less — on
    // the Legacy curve (Cut 121 §1) the roots' upper ranks cost more than the effects
    let root = crate::legacy::IDS.iter().filter_map(|id| get(id)).filter(open).min_by_key(|u| u.rank);
    let effect = offers.iter().filter(|u| !crate::legacy::IDS.contains(&u.id.as_str()) && u.rank < u.cap).find(|u| u.affordable || u.blocked.as_deref() == Some("More Legacy needed"));
    let u = match (root, effect) {
        (Some(r), Some(e)) => if e.price < r.price { e } else { r },
        (r, e) => r.or(e)?,
    };
    u.affordable.then(|| u.id.clone())
}

/// A Legacy order key for one bloodline (`legacy◆@2`): the town's acts sum every bloodline's buys, a bloodline's
/// ledger line reads its own.
pub fn legacy_key(key: &str, bloodline: u32) -> String {
    format!("{key}@{bloodline}")
}

/// The Legacy order at the bank step: every upgrade it buys while the points pay (for the next run while away).
fn legacy_act(game: &mut Game) -> u32 {
    let mut n = 0;
    while let Some(id) = legacy_pick(&game.lineage) {
        let before = crate::legacy::current(&game.lineage).map_or(0, |b| b.points);
        if crate::legacy::buy_next(game, &id).is_err() {
            break;
        }
        let after = crate::legacy::current(&game.lineage).map_or(0, |b| b.points);
        let l = &mut game.lineage;
        let spent = before.saturating_sub(after);
        add_act(l, LEGACY_ACT, 1);
        add_act(l, LEGACY_POINTS, spent);
        // (the acts are the town's, summed over every bloodline; Legacy is a bloodline's own, so its line reads
        // these per-bloodline keys and reconciles with that bloodline's `spent`)
        let b = l.bloodline_id;
        add_act(l, &legacy_key(LEGACY_ACT, b), 1);
        add_act(l, &legacy_key(LEGACY_POINTS, b), spent);
        add_act(l, &legacy_key(&format!("{LEGACY_ACT}:{id}"), b), 1);
        n += 1;
        if n > 64 {
            break;
        }
    }
    n
}

/// The ascend order at a send: the King fallen, the scout and the herald at work — the next ascension's dungeon.
fn ascend_act(game: &mut Game) {
    let l = &game.lineage;
    if !l.ended || l.orders.ascend != "on" || !on(l, "scout") || !on(l, "herald") {
        return;
    }
    if let Ok(tier) = game.carry_on() {
        let l = &mut game.lineage;
        add_act(l, "herald", 1);
        add_act(l, HERALD_ASCEND, 1);
        let day = crate::feats::day_of(l.clock_s);
        crate::feats::news(l, "ascend", format!("herald · descent {tier}"), day);
    }
}

// ---------------------------------------------------------------- the wire

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct WorkNodeWire {
    pub id: String,
    pub kind: String,
    pub branch: String,
    pub name: String,
    pub state: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chore: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub affordable: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_h: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub paused: bool,
    /// Week 2: a hired worker's rank (1–3), and its next rank — the price, the days it still waits (0: on offer).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_price: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_wait_d: Option<u32>,
    /// Week 2: the rank's edge now and the next rank's (≤ 3 words: `+4% hauls`, `half toll`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bonus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank_adds: Option<String>,
    /// Cut 120 §7: an unhired worker's chores by hand toward it (`2/3`) and the hours until its fallback lights it
    /// without them (`eta_h`; absent once lit, or with no fallback) — the post's `forge 2/3 · or 30h`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eta_h: Option<u32>,
    /// Cut 120 §2: a worker's rank-II perk chip — the two perks (`perks`, the first the default) and the one worn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perks: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perk: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct NextPill {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub have: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub need: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct WorksWire {
    pub nodes: Vec<WorkNodeWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lit: Option<String>,
    /// Week 2: the one worker rank on offer (`promote(id)`), when no hire is lit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lit_rank: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NextPill>,
    pub chest: i32,
    pub waits: bool,
    pub sent: bool,
    pub auto_send: bool,
    pub ledger: i64,
    /// Cut 120 §7: the next worker the town works toward — the lit one, else the first whose chore counts (its node
    /// carries `progress`, `fallback_h`, `eta_h`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_worker: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkerPost {
    pub id: String,
    pub post: String,
    /// Week 2: the worker's rank (1–3): its look.
    #[serde(default)]
    pub rank: u32,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lit: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<i32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub paused: bool,
    /// Cut 120 §7: on the lit worker's post, its count (`2/3`), its fallback and the hours to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback_h: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eta_h: Option<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkerAct {
    pub id: String,
    pub what: String,
    pub n: u32,
    pub first: bool,
    /// Blind c4705f9 (A, B: `purse −$12562` unexplained): what the worker bought, each slot's step it
    /// reached (`sword +3`, `mail +2`), and the purse it spent on them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub spent: i32,
    /// Cut 117 §4: on the apprentice's line, why the absence's supplies were limited (the report's
    /// `SupplyBudget.reason`: `no_income` · `income_spent` · `purse_short`); absent when they were not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Cut 120 §6: the purse this line moved, signed — the porter's hauls in, the clerk's deposits out, the apprentice's
    /// steps and sinks out (less the bank his tithe drew), the ranks bought out. The ledger's terms reconcile with it.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub gold: i32,
}

fn is_zero(n: &i32) -> bool {
    *n == 0
}

fn gate_text(gate: &str) -> String {
    match gate {
        "forge" => "blacksmith built".into(),
        "quests" => "quest board".into(),
        "start" => "waystones".into(),
        g => format!("{g} built"),
    }
}

/// The tree on the wire; `waits` is the game's (no run in flight, before the scout).
pub fn wire(l: &LineageState, waits: bool) -> WorksWire {
    let lit_id = lit(l).map(|n| n.id);
    let gold = l.gold;
    let mut nodes: Vec<WorkNodeWire> = NODES
        .iter()
        .map(|n| {
            let done = hired(l, n.id);
            let open = chore_open(l, n);
            let state = if done {
                "done"
            } else if Some(n.id) == lit_id {
                "lit"
            } else if ready(l, n) {
                "ready"
            } else if open {
                "open"
            } else {
                "shut"
            };
            let p = price(l, n);
            let has_chore = !n.chore.is_empty();
            WorkNodeWire {
                id: n.id.into(),
                kind: "worker".into(),
                branch: n.branch.into(),
                name: n.name.into(),
                state: state.into(),
                chore: has_chore.then(|| if n.id == "porter" && l.town.auto_collect { "hauls".into() } else { n.chore.into() }),
                count: has_chore.then(|| count(l, n.id)),
                need: has_chore.then_some(n.need),
                price: has_chore.then_some(p),
                affordable: has_chore.then_some(gold >= p),
                fallback_h: (n.fallback_h > 0).then_some(n.fallback_h),
                trigger: (!open).then(|| gate_text(n.gate)),
                tip: Some(n.tip.into()),
                beat: (!n.beat.is_empty()).then(|| n.beat.into()),
                post: Some(n.post.into()),
                paused: l.tree.paused.contains(n.id),
                rank: done.then(|| rank(l, n.id)).filter(|_| has_chore),
                rank_price: rank_wait(l, n).map(|_| rank_price(l, n)),
                rank_wait_d: rank_wait(l, n),
                bonus: done.then(|| rank_bonus_for(l, n.id, rank(l, n.id))).flatten(),
                rank_adds: rank_wait(l, n).and_then(|_| rank_bonus_for(l, n.id, rank(l, n.id) + 1)),
                progress: (!done && has_chore).then(|| format!("{}/{}", count(l, n.id).min(n.need), n.need)),
                eta_h: eta_h(l, n),
                perks: if done { PERKS.iter().find(|(w, _)| *w == n.id).map(|(_, two)| two.iter().map(|p| p.to_string()).collect()).unwrap_or_default() } else { Vec::new() },
                perk: (done && !perk(l, n.id).is_empty()).then(|| perk(l, n.id).to_string()),
            }
        })
        .collect();
    // the four tracks as branches: their stages
    for t in crate::town::TRACKS {
        let mut next_seen = false;
        for (s, trig) in crate::town::stages(t) {
            let done = crate::town::reached(l, t, s);
            let state = if done {
                "done"
            } else if !next_seen {
                next_seen = true;
                "next"
            } else {
                "later"
            };
            nodes.push(WorkNodeWire { id: format!("{t}:{s}"), kind: "stage".into(), branch: t.into(), name: s.to_string(), state: state.into(), trigger: (!trig.is_empty()).then(|| trig.to_string()), ..Default::default() });
        }
    }
    WorksWire { next: Some(next_pill(l, waits)), nodes, lit: lit_id.map(String::from), lit_rank: lit_rank(l).map(|n| n.id.to_string()), chest: l.tree.chest, waits, sent: l.tree.sent, auto_send: auto_send(l), ledger: l.tree.ledger, next_worker: next_worker(l).map(|n| n.id.to_string()) }
}

/// Cut 120 §7: the hours until an unhired worker's fallback lights it (its chore open, its count short; `None` when
/// it is ready or hired, or has no fallback).
pub fn eta_h(l: &LineageState, n: &NodeDef) -> Option<u32> {
    (!hired(l, n.id) && !n.chore.is_empty() && n.fallback_h > 0 && chore_open(l, n) && !ready(l, n)).then(|| n.fallback_h.saturating_sub(l.age_h()))
}

/// Cut 120 §7: the next worker — the lit one, else the first unhired whose chore is open (`None` on a harness's lineage).
pub fn next_worker(l: &LineageState) -> Option<&'static NodeDef> {
    if l.pkg.literal {
        return None;
    }
    lit(l).or_else(|| NODES.iter().find(|n| !hired(l, n.id) && !n.chore.is_empty() && chore_open(l, n)))
}

/// A rank's edge under the worker's perk (`rank_bonus` for the first perk and the workers without one).
pub fn rank_bonus_for(l: &LineageState, id: &str, r: u32) -> Option<String> {
    let k = r.saturating_sub(1);
    match (id, perk(l, id)) {
        _ if k == 0 => None,
        ("apprentice", p) if p == PERKS[0].1[1] => Some(format!("+{k} unit kept")),
        ("scout", p) if p == PERKS[1].1[1] => Some(format!("+{}% hp", SAFER_START_PCT as u32 * k)),
        _ => rank_bonus(id, r),
    }
}

/// The single next goal (docs/CUT30_5.md §3).
pub fn next_pill(l: &LineageState, waits: bool) -> NextPill {
    let pill = |kind: &str, node: Option<&str>, text: String, have: Option<i64>, need: Option<i64>| NextPill { kind: kind.into(), node: node.map(String::from), text, have, need };
    if let Some(n) = lit(l) {
        let p = price(l, n);
        if l.gold >= p {
            return pill("buy", Some(n.id), format!("{} · {}", n.name, if p == 0 { "free".into() } else { format!("${p}") }), None, None);
        }
    }
    if l.tree.chest > 0 && !porter(l) {
        return pill("chest", Some("porter"), "open chest".into(), Some(l.tree.chest as i64), None);
    }
    if waits {
        return pill("send", Some("scout"), "send".into(), None, None);
    }
    if let Some(n) = lit(l) {
        let p = price(l, n);
        return pill("gold", Some(n.id), format!("{} · ${}/${p}", n.name, l.gold.max(0)), Some(l.gold.max(0) as i64), Some(p as i64));
    }
    if let Some(n) = lit_rank(l) {
        let (p, r) = (rank_price(l, n), numeral(rank(l, n.id) + 1));
        return if l.gold >= p { pill("buy", Some(n.id), format!("{} {r} · ${p}", n.name), None, None) } else { pill("gold", Some(n.id), format!("{} {r} · ${}/${p}", n.name, l.gold.max(0)), Some(l.gold.max(0) as i64), Some(p as i64)) };
    }
    if let Some(n) = NODES.iter().find(|n| !hired(l, n.id) && !n.chore.is_empty() && chore_open(l, n)) {
        let c = count(l, n.id).min(n.need);
        return pill("count", Some(n.id), format!("{} · {c}/{}", n.name, n.need), Some(c as i64), Some(n.need as i64));
    }
    if let Some(r) = crate::systems::next(l) {
        let text = if r.wait_h > 0 { format!("{} · in {}h", r.id, r.wait_h) } else { format!("{} · {}", r.id, r.trigger) };
        return pill("system", None, text, None, Some(r.wait_h as i64));
    }
    pill("none", None, String::new(), None, None)
}

/// The workers at their posts (hired; the lit node's greyed with its price).
pub fn posts(l: &LineageState) -> Vec<WorkerPost> {
    let mut v: Vec<WorkerPost> = NODES.iter().filter(|n| hired(l, n.id)).map(|n| WorkerPost { id: n.id.into(), post: n.post.into(), paused: l.tree.paused.contains(n.id), rank: rank(l, n.id), ..Default::default() }).collect();
    if let Some(n) = lit(l) {
        let progress = (!n.chore.is_empty()).then(|| format!("{}/{}", count(l, n.id).min(n.need), n.need));
        v.push(WorkerPost { id: n.id.into(), post: n.post.into(), lit: true, price: Some(price(l, n)), paused: false, rank: 0, progress, fallback_h: (n.fallback_h > 0).then_some(n.fallback_h), eta_h: eta_h(l, n) });
    }
    v
}

/// The apprentice's purse spent on forge steps in all (an `acts` key beside the workers' ids; never a node's).
pub const APPRENTICE_SPENT: &str = "apprentice$";
/// The apprentice's steps bought per forge slot (`apprentice:weapon`; an `acts` key, never a node's).
pub const APPRENTICE_SLOT: &str = "apprentice:";
/// The apprentice's report item under the `half` order (his gold: half of each haul since his hire).
pub const APPRENTICE_HALF: &str = "half of hauls";

/// The report's worker lines: what each did between two looks at its acts (`before`, `after`); `l` the
/// lineage after (the forge steps the apprentice reached, by name: `sword +3`).
pub fn report_acts(l: &LineageState, before: &BTreeMap<String, u32>, after: &BTreeMap<String, u32>) -> Vec<WorkerAct> {
    let acts_before = before;
    let moved = |key: &str| after.get(key).copied().unwrap_or(0).saturating_sub(acts_before.get(key).copied().unwrap_or(0));
    let was = |key: &str| acts_before.get(key).copied().unwrap_or(0);
    let plural = |k: u32, w: &str| format!("{k} {w}{}", if k == 1 { "" } else { "s" });
    let mut out: Vec<WorkerAct> = NODES
        .iter()
        .filter_map(|n| {
            let k = after.get(n.id).copied().unwrap_or(0).saturating_sub(acts_before.get(n.id).copied().unwrap_or(0));
            (k > 0).then(|| {
                let ascended = if n.id == "herald" { moved(HERALD_ASCEND) } else { 0 };
                let what = match n.id {
                    "porter" => format!("hauled ${k}"),
                    "scout" => format!("sent {k}"),
                    "apprentice" => format!("+{k} step{}", if k == 1 { "" } else { "s" }),
                    "clerk" => format!("banked ${k}"),
                    "drillmaster" => format!("+{k} level{}", if k == 1 { "" } else { "s" }),
                    "kennel_hand" => format!("fielded {k}"),
                    "herald" if ascended > 0 => "ascended".to_string(),
                    "herald" => "swapped".to_string(),
                    "guide" => "deeper start".to_string(),
                    "armourer" => format!("wore {k}"),
                    "kennel_keeper" => format!("sorted {k}"),
                    _ => format!("{k}"),
                };
                let (held, carried) = if n.id == "scout" { (moved(SCOUT_HELD), moved(SCOUT_CARRIED)) } else { (0, 0) };
                let mut gold = 0;
                let (items, spent) = if n.id == "apprentice" {
                    let mut items: Vec<String> = crate::kit::KIT_SLOTS.iter().filter(|s| moved(&format!("{APPRENTICE_SLOT}{s}")) > 0)
                        .filter_map(|s| crate::kit::owned(l, s).checked_sub(1).map(|i| crate::kit::step_label(l, s, i as usize))).collect();
                    // Blind b8dd77c (B: `purse −$11784` over a 4 h of two deaths): under `half` he forges with
                    // his half of every haul since his hire — hauls of earlier absences he saved toward a dear
                    // step included — and the line says whose gold it was
                    if forge_share(l) == Some(50) && moved(APPRENTICE_SPENT) > 0 {
                        items.push(APPRENTICE_HALF.into());
                    }
                    // (Cut 120 §6: his sinks — rations, tithes — and the bank they drew on)
                    if moved(APPRENTICE_SINKS) > 0 {
                        items.push(format!("sinks ${}", moved(APPRENTICE_SINKS)));
                    }
                    gold = moved(APPRENTICE_DREW) as i32 - moved(APPRENTICE_SPENT) as i32 - moved(APPRENTICE_SINKS) as i32;
                    (items, moved(APPRENTICE_SPENT) as i32)
                } else if held + carried > 0 {
                    // Cut 114 §3: the sends he banked before the wall (`banked before Queen ×4`), the haul he carried
                    // home from its stairs (`carried $5400 · Queen`)
                    let at = wall_name(l.rules().route(), l.tree.held_at);
                    let mut v = Vec::new();
                    if carried > 0 {
                        v.push(format!("carried ${} · {at}", moved(SCOUT_CARRIED_GOLD)));
                    }
                    if held > 0 {
                        v.push(format!("banked before {at} ×{held}"));
                    }
                    (v, 0)
                } else if ascended > 0 {
                    let tier = l.endgame.as_ref().map_or(0, |p| p.tier);
                    (vec![format!("descent {tier}")], 0)
                } else if n.id == "kennel_keeper" {
                    let mut v = Vec::new();
                    if moved(KEEPER_BRED) > 0 {
                        v.push(format!("bred {}", moved(KEEPER_BRED)));
                    }
                    if moved(KEEPER_FREED) > 0 {
                        v.push(format!("freed {}", moved(KEEPER_FREED)));
                    }
                    (v, 0)
                } else {
                    (Vec::new(), 0)
                };
                if n.id == "porter" {
                    gold = k as i32;
                } else if n.id == "clerk" {
                    gold = -(k as i32);
                }
                // (his order's first act at a wall is announced as a hire's first act is; the herald's first ascension too)
                let first = was(n.id) == 0 || (held > 0 && was(SCOUT_HELD) == 0) || (carried > 0 && was(SCOUT_CARRIED) == 0) || (ascended > 0 && was(HERALD_ASCEND) == 0);
                WorkerAct { id: n.id.into(), what, n: k, first, items, spent, reason: None, gold }
            })
        })
        .collect();
    // Cut 120 §6: the order lines — the ranks the order bought (`ranks · scout II · −$X`), the Legacy it bought
    // (`legacy · health 2/3 · damage 1/3`)
    let k = moved(RANKS);
    if k > 0 {
        let items = NODES.iter().filter(|n| moved(&format!("{RANKS}:{}", n.id)) > 0).map(|n| format!("{} {}", n.name, numeral(rank(l, n.id)))).collect();
        let spent = moved(RANKS_GOLD) as i32;
        out.push(WorkerAct { id: RANKS.into(), what: format!("+{}", plural(k, "rank")), n: k, first: was(RANKS) == 0, items, spent, reason: None, gold: -spent });
    }
    // (the shown bloodline's own buys: the town's acts sum every bloodline's, `legacy_key`)
    let own = |key: &str| moved(&legacy_key(key, l.bloodline_id));
    let k = own(LEGACY_ACT);
    if k > 0 {
        let b = crate::legacy::current(l);
        let items = crate::legacy::offers(l, false).iter().filter(|u| own(&format!("{LEGACY_ACT}:{}", u.id)) > 0).map(|u| {
            let r = b.and_then(|b| b.upgrades.get(&u.id)).copied().unwrap_or(0);
            let name = u.name.clone().unwrap_or_else(|| u.id.clone()).to_lowercase();
            if u.cap > 1 { format!("{name} {r}/{}", u.cap) } else { name }
        }).collect();
        out.push(WorkerAct { id: LEGACY_ACT.into(), what: format!("+{} · ◆{}", plural(k, "upgrade"), own(LEGACY_POINTS)), n: k, first: was(LEGACY_ACT) == 0, items, spent: 0, reason: None, gold: 0 });
    }
    out
}
