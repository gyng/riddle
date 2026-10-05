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

/// The rest after a run, the scout's rank taken off (the dayplayer's harnesses: rank I or none, as before).
pub fn rest_scaled(l: &LineageState, ticks: u32) -> u32 {
    let r = bonus_rank(l, "scout");
    ticks - ticks * SCOUT_REST_OFF_PCT * r / 100
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
    }
}

/// A weapon's worth a blow: its mean hit with its aim (a forged arm's steps), at its pace.
fn blow_worth(w: &crate::item::Item) -> i64 {
    let (lo, hi) = w.atk();
    let hit = if crate::kit::is_kit_id(w.id) { 80 + crate::kit::AIM_PER_STEP as i64 * w.enchant.clamp(0, crate::kit::AIM_STEPS) as i64 } else { 80 };
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
        act(game, "drillmaster", n);
    }
    // the apprentice: the cheapest next step the purse pays with the reserve kept
    if on(&game.lineage, "apprentice") {
        let mut n = 0;
        loop {
            let l = &game.lineage;
            let reserve = RESERVE_UNITS * crate::kit::unit(l.best_depth) as i32;
            let Some((slot, p)) = crate::kit::ladders(l).iter().filter_map(|x| x.next.as_ref().map(|s| (x.slot.clone(), s.price as i32))).min_by_key(|x| x.1) else { break };
            let off = APPRENTICE_OFF_PCT * bonus_rank(l, "apprentice");
            if purse(l) < p - p * off as i32 / 100 + reserve || crate::kit::buy_step_off(&mut game.lineage, &slot, off).is_err() {
                break;
            }
            n += 1;
        }
        act(game, "apprentice", n);
    }
    // the clerk: the purse above the next forge step and the reserve into the bank
    if on(&game.lineage, "clerk") && crate::town::built(&game.lineage, "bank") {
        let l = &game.lineage;
        let next = crate::kit::ladders(l).iter().filter_map(|x| x.next.as_ref().map(|s| s.price as i32)).min().unwrap_or(0);
        let spare = purse(l) - next - RESERVE_UNITS * crate::kit::unit(l.best_depth) as i32;
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
            k.sort_by_key(|c| (std::cmp::Reverse(c.level), c.id));
            let add: Vec<u32> = k.iter().take(slots - ids.len()).map(|c| c.id).collect();
            let n = add.len() as u32;
            ids.extend(add);
            if game.set_party_by(ids, false).is_ok() {
                act(game, "kennel_hand", n);
            }
        }
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
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WorkerAct {
    pub id: String,
    pub what: String,
    pub n: u32,
    pub first: bool,
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
                bonus: done.then(|| rank_bonus(n.id, rank(l, n.id))).flatten(),
                rank_adds: rank_wait(l, n).and_then(|_| rank_bonus(n.id, rank(l, n.id) + 1)),
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
    WorksWire { next: Some(next_pill(l, waits)), nodes, lit: lit_id.map(String::from), lit_rank: lit_rank(l).map(|n| n.id.to_string()), chest: l.tree.chest, waits, sent: l.tree.sent, auto_send: auto_send(l), ledger: l.tree.ledger }
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
        v.push(WorkerPost { id: n.id.into(), post: n.post.into(), lit: true, price: Some(price(l, n)), paused: false, rank: 0 });
    }
    v
}

/// The report's worker lines: what each did between two looks at its acts (`before`, `after`).
pub fn report_acts(before: &BTreeMap<String, u32>, after: &BTreeMap<String, u32>) -> Vec<WorkerAct> {
    let acts_before = before;
    NODES
        .iter()
        .filter_map(|n| {
            let k = after.get(n.id).copied().unwrap_or(0) - acts_before.get(n.id).copied().unwrap_or(0);
            (k > 0).then(|| {
                let what = match n.id {
                    "porter" => format!("hauled ${k}"),
                    "scout" => format!("sent {k}"),
                    "apprentice" => format!("+{k} step{}", if k == 1 { "" } else { "s" }),
                    "clerk" => format!("banked ${k}"),
                    "drillmaster" => format!("+{k} level{}", if k == 1 { "" } else { "s" }),
                    "kennel_hand" => format!("fielded {k}"),
                    "herald" => "swapped".to_string(),
                    "guide" => "deeper start".to_string(),
                    "armourer" => format!("wore {k}"),
                    _ => format!("{k}"),
                };
                WorkerAct { id: n.id.into(), what, n: k, first: acts_before.get(n.id).copied().unwrap_or(0) == 0 }
            })
        })
        .collect()
}
