//! Cut 28 §1 (P3: "goals the player picks, priced by the forecast"): oaths. A lineage keeps a board
//! of `BOARD` standing oaths drawn from a pool — each a constraint on one send and a reward that is
//! never a stat (a card, a party slot, a row, a verb, a waystone, a chronicle title), so policy stays
//! the lever (the Cut 25 lever row holds with every reward owned). Swearing one costs gold priced
//! from the lineage's income (`price`) — the late game's gold sink; a send that keeps it spends the
//! price into the reward (`grant`), forswearing refunds half. One oath is sworn at a time; the camp's
//! forecast prices it on its own panel (`Forecast.oath`: the share of the sims that keep it), an edit's
//! paired move carries its move (`ForecastVs.oath`), and the divergence's ends say whether each
//! branch kept it. Nothing swears on its own: the bots (and an absence) never do.
use crate::engine::{ExitTier, Game, LineageState, Run};
use crate::rng::{hash_str, Rng};
use crate::wire::{BossWall, Bounty, Oath, OathReward, OathShare};
use serde::{Deserialize, Serialize};

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Standing oaths per lineage.
pub const BOARD: usize = 3;
/// The pool, in draw order of preference (`draw`): each a constraint of one send.
/// - `bold` — reach one past the best and never take the `return` exit (a bank is fine) → a verb
/// - `lean` — reach the best depth without resting → a card
/// - `tamer` — tame a kind the lineage has not → a party slot
/// - `fire` — slay `{boss}` with fire on him (thrown, or a burning floor) → a title
/// - `slayer` — slay `{boss}`, the next band boss unslain → the waystone past him
///
/// (Cut 28 tuning: `dry` — D{best} · no drink → a card — and `quiet` — ≤ N kills — were cut: on
/// the oath sets `dry`'s best set was the bank-optimal set less its drink row (1 row apart on
/// raterAV's) and no writable set kept `quiet` or a potion-and-rest-free `dry` in a night.)
pub const KINDS: [&str; 5] = ["bold", "tamer", "fire", "slayer", "lean"];
/// Forswearing refunds this share of the price (percent).
pub const REFUND_PCT: i32 = 50;
/// The sends of a night the camp's `night` share is read over (`engine::NIGHT_RUNS`).
pub const NIGHT: i32 = crate::engine::NIGHT_RUNS as i32;

/// Cut 28 §1: one standing oath as the lineage keeps it (`LineageState::oaths`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OathState {
    pub id: String,
    pub kind: String,
    /// The floor to reach (`bold` · `lean`), or the boss's floor (`fire` · `slayer`).
    #[serde(default)]
    pub depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
    /// `tamer`: the kinds the lineage had tamed when it was drawn.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub seen: Vec<String>,
    /// A count the oath allows (none of the pool's uses one now; kept for saves).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub n: u32,
    pub reward: OathReward,
    pub price: i32,
}

/// The paid tactic cards an oath may give, in order (a free card is the player's to take).
const CARD_REWARDS: [&str; 6] = ["gas_step", "pack_break", "thief_guard", "boss_focus", "last_stand", "corridor_fighting"];
/// The verbs and condition words an oath may give.
const VERB_REWARDS: [&str; 7] = ["throw", "cond_on_kill", "cond_on_see", "cond_alert", "cond_turns", "cond_loot", "cond_party_hp"];
const SLOT_REWARDS: [&str; 3] = ["party_slot_2", "party_slot_3", "party_slot_4"];
const ROW_REWARDS: [&str; 6] = ["row5", "row6", "row7", "row8", "row9", "row10"];

/// The chronicle title a `fire` oath earns (one per boss burned).
fn fire_title(boss: &str) -> String {
    match boss {
        "goblin_warlord" => "Warlord-burner".into(),
        "bloat_mother" => "Firebrand".into(),
        "lich" => "Pyre of the Lich".into(),
        _ => format!("{}-burner", crate::sifter::boss_short(boss)),
    }
}

/// The oath's price: the lineage's income decides it — half the last full night's net
/// (`LineageState::last_night_net`: fixed through a night, so the board does not re-price as the
/// purse moves), never under the forge's unit (`kit::unit`: $100 + $25 a floor of the best depth), in tens.
pub fn price(l: &LineageState) -> i32 {
    let unit = crate::kit::unit(l.best_depth) as i32;
    let night = l.last_night_net.max(0) / 2;
    (unit.max(night) + 5) / 10 * 10
}

/// The floor a depth oath asks for: one past the lineage's best for `bold` (a push), the best itself
/// for an oath that takes a tool away (`lean`: the record without rest);
/// at least D3.
fn goal(l: &LineageState, kind: &str) -> u32 {
    let push = (kind == "bold") as u32;
    (l.best_depth + push).max(3)
}

fn tamed(l: &LineageState) -> Vec<String> {
    l.facts.iter().filter_map(|f| f.strip_prefix("tamed:")).map(str::to_string).collect()
}

/// The next unslain band boss (base order) within reach — his floor at most two past the best depth.
fn next_boss(l: &LineageState) -> Option<(&'static str, u32)> {
    crate::descent::BOSS_DEPTHS.iter().copied().find(|(k, d)| !l.kills.contains(*k) && *d <= l.best_depth + 2)
}

/// The boss a `fire` oath names: the deepest band boss whose floor the lineage has reached (or is
/// one short of), else the Warlord.
fn fire_boss(l: &LineageState) -> (&'static str, u32) {
    crate::descent::BOSS_DEPTHS.iter().copied().rfind(|(_, d)| *d <= l.best_depth + 1).unwrap_or(crate::descent::BOSS_DEPTHS[0])
}

fn owned(l: &LineageState, id: &str) -> bool {
    l.unlocks.contains(id)
}

/// The reward a kind would give now, if one is left to give.
fn reward_for(l: &LineageState, kind: &str, boss: Option<&str>) -> Option<OathReward> {
    let r = |k: &str, id: String, label: String| Some(OathReward { kind: k.into(), id, label });
    match kind {
        "bold" => VERB_REWARDS.iter().find(|c| !owned(l, c)).and_then(|c| r("verb", c.to_string(), format!("verb: {}", c.trim_start_matches("cond_").replace('_', " ")))),
        "lean" => CARD_REWARDS.iter().find(|c| !owned(l, c)).and_then(|c| r("card", c.to_string(), format!("card: {}", c.replace('_', " ")))),
        "tamer" => SLOT_REWARDS.iter().find(|c| !owned(l, c)).and_then(|c| r("slot", c.to_string(), "+1 party".into())),
        "fire" => {
            let t = fire_title(boss?);
            (!l.titles.contains(&t)).then(|| OathReward { kind: "title".into(), label: format!("title: {t}"), id: t })
        }
        "slayer" => {
            let d = crate::descent::BOSS_DEPTHS.iter().find(|(k, _)| Some(*k) == boss)?.1 + 1;
            (crate::engine::WAYSTONES.contains(&d) && !l.waystones.contains(&d)).then(|| OathReward { kind: "waystone".into(), id: d.to_string(), label: format!("waystone D{d}") })
        }
        _ => None,
    }
}

/// A fresh oath of `kind` for the lineage as it stands, if the kind has something to give and a
/// goal it can name.
pub fn draw_kind(l: &LineageState, kind: &str, n: u32) -> Option<OathState> {
    let (depth, boss) = match kind {
        "bold" | "lean" => (goal(l, kind), None),
        "tamer" => {
            if !l.unlocks.contains("tame") {
                return None;
            }
            (0, None)
        }
        "fire" => {
            let (k, d) = fire_boss(l);
            (d, Some(k.to_string()))
        }
        "slayer" => {
            let (k, d) = next_boss(l)?;
            (d, Some(k.to_string()))
        }
        _ => return None,
    };
    let reward = reward_for(l, kind, boss.as_deref())?;
    Some(OathState { id: format!("{kind}:{n}"), kind: kind.into(), depth, boss, seen: if kind == "tamer" { tamed(l) } else { Vec::new() }, n: 0, reward, price: price(l) })
}

/// Whether a standing oath still stands (its reward still to give; a `slayer`'s boss unslain).
fn stands(l: &LineageState, o: &OathState) -> bool {
    if o.kind == "slayer" && o.boss.as_deref().is_some_and(|b| l.kills.contains(b)) {
        return false;
    }
    reward_for(l, &o.kind, o.boss.as_deref()).is_some_and(|r| r.id == o.reward.id)
}

/// Keep the board full: drop the oaths that no longer stand (never the sworn one while it stands),
/// re-price the unsworn to the lineage's income now, and draw new ones from the kinds not on it —
/// in a seeded order, so a lineage's board is a function of its seed and its history.
pub fn refresh(l: &mut LineageState) {
    let sworn = l.oath_sworn.clone();
    let keep: Vec<OathState> = l.oaths.iter().filter(|o| stands(l, o) || Some(&o.id) == sworn.as_ref()).cloned().collect();
    l.oaths = keep;
    if sworn.as_ref().is_some_and(|s| !l.oaths.iter().any(|o| &o.id == s)) {
        l.oath_sworn = None;
    }
    let p = price(l);
    let sworn = l.oath_sworn.clone();
    for o in l.oaths.iter_mut().filter(|o| Some(&o.id) != sworn.as_ref()) {
        o.price = p;
    }
    let mut tries = 0;
    while l.oaths.len() < BOARD && tries < 2 * KINDS.len() {
        tries += 1;
        let mut rng = Rng::derive(l.seed, hash_str("oath") ^ l.oath_drawn as u64);
        let open: Vec<&str> = KINDS.iter().copied().filter(|k| !l.oaths.iter().any(|o| o.kind == *k)).collect();
        if open.is_empty() {
            break;
        }
        let kind = open[rng.below(open.len() as u32) as usize];
        l.oath_drawn += 1;
        if let Some(o) = draw_kind(l, kind, l.oath_drawn) {
            l.oaths.push(o);
        }
    }
    // (a kind drawn and refused leaves its slot to the next; the board holds what can be given)
    for kind in KINDS {
        if l.oaths.len() >= BOARD {
            break;
        }
        if !l.oaths.iter().any(|o| o.kind == kind) {
            l.oath_drawn += 1;
            if let Some(o) = draw_kind(l, kind, l.oath_drawn) {
                l.oaths.push(o);
            }
        }
    }
}

/// The sworn oath, if one is.
pub fn sworn(l: &LineageState) -> Option<&OathState> {
    let id = l.oath_sworn.as_ref()?;
    l.oaths.iter().find(|o| &o.id == id)
}

/// Swear the oath `id` (paying its price; one at a time — another sworn is forsworn first).
pub fn swear(game: &mut Game, id: &str) -> Result<(), String> {
    let l = &mut game.lineage;
    refresh(l);
    let o = l.oaths.iter().find(|o| o.id == id).cloned().ok_or("no such oath")?;
    if l.oath_sworn.as_deref() == Some(id) {
        return Ok(());
    }
    let refund = sworn(l).map(|s| s.price * REFUND_PCT / 100).unwrap_or(0);
    if l.gold + refund < o.price {
        return Err("not enough gold".into());
    }
    if l.oath_sworn.is_some() {
        forswear(game)?;
    }
    let l = &mut game.lineage;
    l.gold_move(-o.price, &format!("oath {}", o.kind));
    l.oath_sworn = Some(o.id.clone());
    Ok(())
}

/// Forswear the sworn oath: half its price back; it stays on the board.
pub fn forswear(game: &mut Game) -> Result<(), String> {
    let l = &mut game.lineage;
    let o = sworn(l).cloned().ok_or("no oath sworn")?;
    l.gold_move(o.price * REFUND_PCT / 100, &format!("forswear {}", o.kind));
    l.oath_sworn = None;
    refresh(l);
    Ok(())
}

/// Did this run keep the oath?
pub fn kept(o: &OathState, run: &Run) -> bool {
    let tier = run.over.unwrap_or(ExitTier::Return);
    let boss_slain = |b: &str| run.boss_kills.iter().any(|(_, k)| k == b);
    match o.kind.as_str() {
        "bold" => run.max_depth >= o.depth && (tier != ExitTier::Return || run.over.is_none()) && !run.timed_out,
        "lean" => run.max_depth >= o.depth && !run.rested,
        "tamer" => run.tamed.iter().any(|(_, k)| !o.seen.contains(k)),
        "fire" => o.boss.as_deref().is_some_and(|b| boss_slain(b) && run.burned.iter().any(|k| k == b)),
        "slayer" => o.boss.as_deref().is_some_and(boss_slain),
        _ => false,
    }
}

/// How close a send came to keeping the oath (0..1; 1 when kept): the floor reached toward the
/// goal while the constraint held (a depth oath), the boss met, hurt by fire, slain (a boss oath), a
/// tame tried or made. The metrics' search reads it past a plateau of zero (`examples/oath_lib`).
pub fn progress(o: &OathState, run: &Run) -> f64 {
    if kept(o, run) {
        return 1.0;
    }
    let toward = |d: u32| (run.max_depth.min(d) as f64 / d.max(1) as f64).powi(2) * 0.9;
    let boss_met = |b: &str| run.boss_seen_t.is_some() && run.max_depth >= crate::descent::BOSS_DEPTHS.iter().find(|(k, _)| *k == b).map_or(99, |x| x.1);
    match o.kind.as_str() {
        "lean" => if !run.rested { toward(o.depth) } else { 0.0 },
        "bold" => if run.over != Some(ExitTier::Return) { toward(o.depth) } else { toward(o.depth) * 0.3 },
        "tamer" => if run.tamed.is_empty() { 0.0 } else { 0.5 },
        "fire" | "slayer" => {
            let Some(b) = o.boss.as_deref() else { return 0.0 };
            let mut p = toward(crate::descent::BOSS_DEPTHS.iter().find(|(k, _)| *k == b).map_or(1, |x| x.1)) * 0.5;
            if boss_met(b) {
                p += 0.2;
            }
            if run.burned.iter().any(|k| k == b) {
                p += 0.2;
            }
            if o.kind == "fire" && run.boss_kills.iter().any(|(_, k)| k == b) {
                p += 0.05;
            }
            p.min(0.95)
        }
        _ => 0.0,
    }
}

/// Give the oath's reward (its price was spent into it at the swearing).
pub fn grant(l: &mut LineageState, o: &OathState) {
    match o.reward.kind.as_str() {
        "card" | "verb" | "row" | "slot" => {
            l.unlocks.insert(o.reward.id.clone());
        }
        "title" => {
            if !l.titles.contains(&o.reward.id) {
                l.titles.push(o.reward.id.clone());
            }
            let trophy = format!("oath:{}", o.reward.id);
            if !l.trophies.contains(&trophy) {
                l.trophies.push(trophy);
            }
        }
        "waystone" => {
            if let Ok(d) = o.reward.id.parse::<u32>() {
                if !l.waystones.contains(&d) {
                    l.waystones.push(d);
                    l.waystones.sort_unstable();
                }
            }
        }
        _ => {}
    }
}

/// Every reward the pool can give, owned (the metrics' lever row with every oath reward: policy
/// stays the lever).
pub fn grant_all(l: &mut LineageState) {
    for id in CARD_REWARDS.iter().chain(VERB_REWARDS.iter()).chain(SLOT_REWARDS.iter()).chain(ROW_REWARDS.iter()) {
        l.unlocks.insert(id.to_string());
    }
    for (boss, depth) in crate::descent::BOSS_DEPTHS {
        let t = fire_title(boss);
        if !l.titles.contains(&t) {
            l.titles.push(t.clone());
            l.trophies.push(format!("oath:{t}"));
        }
        if crate::engine::WAYSTONES.contains(&(depth + 1)) && !l.waystones.contains(&(depth + 1)) {
            l.waystones.push(depth + 1);
        }
    }
    l.waystones.sort_unstable();
    refresh(l);
}

/// Settle a finished real run against the sworn oath: `Some(true)` kept (the reward granted, the
/// oath off the board, a new one drawn), `Some(false)` sworn and not kept, `None` none sworn.
pub fn settle(l: &mut LineageState, run: &Run) -> Option<(OathState, bool)> {
    let o = sworn(l)?.clone();
    if !kept(&o, run) {
        return Some((o, false));
    }
    grant(l, &o);
    l.oath_sworn = None;
    l.oaths.retain(|x| x.id != o.id);
    l.oaths_kept += 1;
    refresh(l);
    Some((o, true))
}

/// The board on the wire.
pub fn wire(l: &LineageState) -> Vec<Oath> {
    l.oaths
        .iter()
        .map(|o| {
            let chips = chips(o);
            Oath {
                id: o.id.clone(),
                kind: o.kind.clone(),
                text: chips.join(" · "),
                chips,
                reward: o.reward.clone(),
                price: o.price,
                sworn: l.oath_sworn.as_ref() == Some(&o.id),
                boss: o.boss.clone(),
                depth: (o.depth > 0).then_some(o.depth),
                counter: o.boss.as_deref().map(|b| counter_fact(l, b)),
            }
        })
        .collect()
}

/// An oath's constraint as chips (≤ 3 words each).
pub fn chips(o: &OathState) -> Vec<String> {
    let boss = o.boss.as_deref().map(crate::sifter::boss_short).unwrap_or("boss");
    match o.kind.as_str() {
        "bold" => vec![format!("D{}", o.depth), "no return".into()],
        "lean" => vec![format!("D{}", o.depth), "no rest".into()],
        "tamer" => vec!["tame".into(), "a new kind".into()],
        "fire" => vec![boss.into(), "fire".into()],
        "slayer" => vec!["slay".into(), boss.into()],
        k => vec![k.into()],
    }
}

/// The text of an oath (its chips joined).
pub fn text(o: &OathState) -> String {
    chips(o).join(" · ")
}

/// A boss's counter as the lineage knows it: `mother: fire`, or `mother: ?` while unknown.
pub fn counter_fact(l: &LineageState, boss: &str) -> String {
    let short = crate::sifter::boss_short(boss).to_lowercase();
    if crate::facts::boss_counter_known(&l.facts, boss) {
        format!("{short}: {}", counter_word(boss))
    } else {
        format!("{short}: ?")
    }
}

/// A boss's counter in one or two words (`fire`, `aim`, `summons first`).
pub fn counter_word(boss: &str) -> &'static str {
    match boss {
        "goblin_warlord" => "aim",
        "bloat_mother" => "fire",
        "lich" => "summons first",
        "foundry_master" => "reflect read",
        "lurker_queen" => "silence",
        "mirror_king" => "cadence",
        _ => "aim",
    }
}

/// The band bosses as walls: every one up to the best depth, and the first unslain past it.
pub fn walls(l: &LineageState) -> Vec<BossWall> {
    let mut out = Vec::new();
    for (kind, depth) in crate::descent::BOSS_DEPTHS {
        let past = depth > l.best_depth;
        let slain = l.kills.contains(kind);
        let known = crate::facts::boss_counter_known(&l.facts, kind);
        let row = crate::facts::boss_counter_row(&l.facts, kind);
        out.push(BossWall {
            boss: kind.into(),
            title: crate::sifter::boss_short(kind).into(),
            depth,
            slain,
            known,
            fact: counter_fact(l, kind),
            learn: (!known).then(|| if kind == "bloat_mother" || kind == "lurker_queen" { "meet her" } else { "meet him" }.into()),
            counter: row.as_ref().map(crate::facts::counter_text),
            row,
        });
        if past && !slain {
            break;
        }
    }
    out
}

/// The bounty on the wire (`Lineage.bounty`): what it pays and needs, and the boss on its floor.
pub fn bounty_wire(l: &LineageState, depth: u32) -> Bounty {
    let boss = l.rules().route().boss(depth).map(str::to_string);
    let fact = boss.as_deref().map(|b| counter_fact(l, b));
    Bounty { depth, pays: format!("$×{} · item", crate::engine::BOUNTY_GOLD_MULT), needs: "reach".into(), boss, fact }
}

/// The sworn oath priced on a panel's sims (`SimResult.oath`).
pub fn share(l: &LineageState, ended: &[crate::forecast::SimResult]) -> Option<OathShare> {
    let o = sworn(l)?;
    let n = ended.len();
    let p = ended.iter().filter(|r| r.oath).count() as f64 / n.max(1) as f64;
    Some(OathShare { id: o.id.clone(), text: text(o), share: p, pm: crate::forecast::half_width(p, n), night: night(p) })
}

/// The chance a night of `NIGHT` sends keeps an oath each send keeps with `p`.
pub fn night(p: f64) -> f64 {
    1.0 - (1.0 - p.clamp(0.0, 1.0)).powi(NIGHT)
}
