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
    pub hired: Vec<(String, u32)>,
    /// Chores done by hand, per node.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub done: BTreeMap<String, u32>,
    /// Hired workers switched off.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub paused: BTreeSet<String>,
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub acts: BTreeMap<String, u32>,
}

impl Tree {
    /// A new lineage's tree: the quartermaster given, nothing else.
    pub fn fresh() -> Tree {
        Tree { v: 1, hired: vec![("quartermaster".into(), 0)], ..Default::default() }
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
        } else {
            l.tree.chest += delta;
        }
    }
    l.tree.chest = l.tree.chest.min(l.gold.max(0)).max(0);
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

/// The workers' standing orders, at a real send (before the run begins; never in a sim, never on a
/// harness's literal lineage).
pub fn at_send(game: &mut Game) {
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
        let ids: Vec<String> = std::iter::once(game.lineage.pkg.stance.clone()).chain(game.lineage.pkg.tactics.iter().cloned().filter(|_| std::env::var_os("PROBE_STANCE_ONLY").is_none())).collect();
        for id in ids {
            while crate::packages::level_price(&game.lineage, &id).is_some_and(|m| m <= game.lineage.marks) {
                if crate::packages::spend_level(&mut game.lineage, &id).is_err() {
                    break;
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
            let reserve = if std::env::var_os("PROBE_NIGHT_RESERVE").is_some() { (RESERVE_UNITS * crate::kit::unit(l.best_depth) as i32).max(crate::kit::per_night(l)) } else { RESERVE_UNITS * crate::kit::unit(l.best_depth) as i32 };
            let Some((slot, p)) = crate::kit::ladders(l).iter().filter_map(|x| x.next.as_ref().map(|s| (x.slot.clone(), s.price as i32))).min_by_key(|x| x.1) else { break };
            if purse(l) < p + reserve || crate::kit::buy_step(&mut game.lineage, &slot).is_err() {
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
    // the guide: the deepest lit stone a band under the record, never shallower than the start set
    if on(&game.lineage, "guide") {
        let l = &game.lineage;
        let gap = std::env::var("RIDDLE_GUIDE_GAP").ok().and_then(|v| v.parse().ok()).unwrap_or(GUIDE_GAP);
        let pick = l.stones().into_iter().filter(|s| s + gap <= l.best_depth).max();
        if let Some(s) = pick.filter(|s| *s > l.start.max(1)) {
            if game.lineage.set_start(s).is_ok() {
                act(game, "guide", 1);
            }
        }
    }
    // the armourer: the best vault weapon and armour go with each send
    if on(&game.lineage, "armourer") && !game.lineage.vault.is_empty() {
        let mut add = Vec::new();
        for cat in [crate::defs::Cat::Weapon, crate::defs::Cat::Armour] {
            if let Some(it) = game.lineage.vault.iter().filter(|v| v.cat() == cat).max_by_key(|v| (v.value(), std::cmp::Reverse(v.id))) {
                if !game.loadout.contains(&it.id) {
                    add.push(it.id);
                }
            }
        }
        if !add.is_empty() && !game.lineage.variant_is("bones_only") {
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
                chore: has_chore.then(|| n.chore.into()),
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
    WorksWire { next: Some(next_pill(l, waits)), nodes, lit: lit_id.map(String::from), chest: l.tree.chest, waits, sent: l.tree.sent, auto_send: auto_send(l), ledger: l.tree.ledger }
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
    let mut v: Vec<WorkerPost> = NODES.iter().filter(|n| hired(l, n.id)).map(|n| WorkerPost { id: n.id.into(), post: n.post.into(), paused: l.tree.paused.contains(n.id), ..Default::default() }).collect();
    if let Some(n) = lit(l) {
        v.push(WorkerPost { id: n.id.into(), post: n.post.into(), lit: true, price: Some(price(l, n)), paused: false });
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
