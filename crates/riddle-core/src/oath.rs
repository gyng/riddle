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

/// Cut 29 §1 (docs/PROGRESSION.md §4–5): the oath rewards are a pool of their own, never sold — so
/// the board never empties when the marks catalogue is bought out. Per kind, in order, each opened by
/// its tier (`meta::UnlockDef::tier`): `bold` the verb `hold` then the D9 route; `lean` gas step,
/// last stand, then the heir pick (three traits, not two); `tamer` the party slots. A kind whose
/// pool is spent gives a chronicle title (`bold`/`lean`: the floor sworn to — no stat).
pub const BOLD_REWARDS: [&str; 2] = ["hold", "route2"];
pub const LEAN_REWARDS: [&str; 3] = ["gas_step", "last_stand", "heir_pick"];
pub const SLOT_REWARDS: [&str; 3] = ["party_slot_2", "party_slot_3", "party_slot_4"];

/// The next reward of a pool the lineage can be given now (its tier open, its prerequisite owned).
fn next_of(l: &LineageState, pool: &[&'static str]) -> Option<&'static str> {
    let t = crate::meta::tier(l);
    pool.iter().copied().find(|id| !owned(l, id) && crate::meta::def(id).is_none_or(|d| d.tier <= t && d.prereq.is_none_or(|p| owned(l, p))))
}

/// The kind of a pool reward (`OathReward.kind`): `verb` · `route` · `heir` · `slot` · `card`.
fn reward_kind(id: &str) -> &'static str {
    match id {
        "hold" => "verb",
        "route2" => "route",
        "heir_pick" => "heir",
        c if c.starts_with("party_slot") => "slot",
        _ => "card",
    }
}

/// The label of a pool reward (≤ 3 words).
fn reward_label(id: &str) -> String {
    match id {
        "hold" => "verb: hold".into(),
        "route2" => "route: D9 fork".into(),
        "heir_pick" => "heir pick: 3".into(),
        c if c.starts_with("party_slot") => "+1 party".into(),
        c => format!("card: {}", c.replace('_', " ")),
    }
}

/// The title a `bold` or `lean` oath gives once its pool is spent: the floor it swore to.
fn depth_title(kind: &str, depth: u32) -> String {
    match kind {
        "lean" => format!("Lean at D{depth}"),
        _ => format!("Bold at D{depth}"),
    }
}

/// The chronicle title a `fire` oath earns (one per boss burned).
fn fire_title(boss: &str) -> String {
    match boss {
        "goblin_warlord" => "Warlord-burner".into(),
        "bloat_mother" => "Firebrand".into(),
        "lich" => "Pyre of the Lich".into(),
        _ => format!("{}-burner", crate::sifter::boss_short(boss)),
    }
}

/// The oath's price: the lineage's income decides it — Cut 29 §5: a quarter of the last whole day's
/// net per slot (`LineageState::last_day_net`: fixed through a day), or half the last full night's
/// while no day has closed (Cut 28), never under the forge's unit (`kit::unit`: $100 + $25 a floor of
/// the best depth), in tens.
pub fn price(l: &LineageState) -> i32 {
    let unit = crate::kit::unit(l.best_depth) as i32;
    let income = if l.day > 0 { l.last_day_net.max(0) / 4 } else { l.last_night_net.max(0) / 2 };
    (unit.max(income) + 5) / 10 * 10
}

/// Cut 29 §1: the oaths a lineage may hold sworn at once (1, `oath_slot_2`, `oath_slot_3`).
pub fn slots(l: &LineageState) -> usize {
    1 + l.unlocks.contains("oath_slot_2") as usize + l.unlocks.contains("oath_slot_3") as usize
}

/// Every sworn oath's id, the first slot's first.
pub fn sworn_ids(l: &LineageState) -> Vec<String> {
    l.oath_sworn.iter().chain(l.oath_extra.iter()).cloned().collect()
}

/// Cut 29 §1: a new day on the lineage's clock (`offline`): the last day's net closes (the oaths'
/// price), and every oath sworn on an earlier day lapses unkept — no refund (a bet the player chose).
pub fn new_day(l: &mut LineageState, day: u32) {
    if day <= l.day {
        return;
    }
    l.last_day_net = std::mem::take(&mut l.day_net);
    l.day = day;
    let stale = |id: &String, l: &LineageState| l.oath_days.get(id).is_some_and(|d| *d < day);
    if l.oath_sworn.as_ref().is_some_and(|id| stale(id, l)) {
        l.oath_sworn = None;
    }
    let extra: Vec<String> = l.oath_extra.iter().filter(|id| !stale(id, l)).cloned().collect();
    l.oath_extra = extra;
    let live = sworn_ids(l);
    l.oath_days.retain(|id, _| live.contains(id));
    refresh(l);
}

/// Cut 29 §1: an oath draw — ◆`meta::OATH_DRAW_COST`, from tier `meta::OATH_DRAW_TIER`: the first
/// unsworn standing oath is replaced by a fresh one of a kind not on the board (the board grows by
/// one when every standing oath is sworn). The new oath's id.
pub fn draw(game: &mut Game) -> Result<String, String> {
    let l = &mut game.lineage;
    if crate::meta::tier(l) < crate::meta::OATH_DRAW_TIER {
        return Err(format!("needs {}", crate::meta::tier_need(crate::meta::OATH_DRAW_TIER)));
    }
    if l.marks < crate::meta::OATH_DRAW_COST {
        return Err("not enough marks".into());
    }
    refresh(l);
    let sworn = sworn_ids(l);
    let mut rng = Rng::derive(l.seed, hash_str("oath draw") ^ l.oath_drawn as u64);
    let mut kinds: Vec<&str> = KINDS.to_vec();
    // a seeded order; kinds not on the board first
    let off = rng.below(kinds.len() as u32) as usize;
    kinds.rotate_left(off);
    kinds.sort_by_key(|k| l.oaths.iter().any(|o| o.kind == *k));
    let replace = l.oaths.iter().position(|o| !sworn.contains(&o.id));
    // (the replaced oath's own kind last: a fresh one of it only when it gives something else — AP
    // s1 at D33: every other kind's reward already on the board, the draw failed and marks piled)
    let own = replace.map(|i| l.oaths[i].kind.clone());
    kinds.sort_by_key(|k| own.as_deref() == Some(*k));
    for kind in kinds {
        l.oath_drawn += 1;
        if let Some(o) = draw_kind(l, kind, l.oath_drawn) {
            if l.oaths.iter().any(|x| x.kind == o.kind && x.reward == o.reward) {
                continue;
            }
            let id = o.id.clone();
            match replace {
                Some(i) => l.oaths[i] = o,
                None => l.oaths.push(o),
            }
            l.marks -= crate::meta::OATH_DRAW_COST;
            return Ok(id);
        }
    }
    Err("no oath to draw".into())
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

/// Cut 29 core (marks at the deepest wall: once the catalogue was bought and every title of the
/// pool owned, draws failed and marks piled — rater AP s1 ◆25): a title is earned again, numbered
/// (`Bold at D34`, `Bold at D34 II`, …) — the first of its line neither owned nor offered by another
/// oath on the board (`except`: the oath asking). Titles are the chronicle's: no stat, no card.
fn next_title(l: &LineageState, base: &str, except: Option<&str>) -> String {
    const ROMAN: [&str; 9] = ["II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
    let taken = |t: &str| l.titles.iter().any(|x| x == t) || l.oaths.iter().any(|o| Some(o.id.as_str()) != except && o.reward.kind == "title" && o.reward.id == t);
    (1..)
        .map(|n: usize| match n {
            1 => base.to_string(),
            n if n <= ROMAN.len() + 1 => format!("{base} {}", ROMAN[n - 2]),
            n => format!("{base} {n}"),
        })
        .find(|t| !taken(t))
        .unwrap_or_else(|| base.to_string())
}

/// A numbered title's line (`Bold at D34 III` → `Bold at D34`).
fn title_line(t: &str) -> &str {
    match t.rsplit_once(' ') {
        Some((head, n)) if n.parse::<u32>().is_ok() || (!n.is_empty() && n.chars().all(|c| matches!(c, 'I' | 'V' | 'X'))) => head,
        _ => t,
    }
}

/// The reward a kind would give now, if one is left to give (`except`: the board's oath asking —
/// its own title is not taken from it).
fn reward_for(l: &LineageState, kind: &str, boss: Option<&str>, except: Option<&str>) -> Option<OathReward> {
    let pool = |pool: &[&'static str], _k: &str| next_of(l, pool).map(|id| OathReward { kind: reward_kind(id).into(), id: id.into(), label: reward_label(id) });
    let title = |base: String| {
        let t = next_title(l, &base, except);
        Some(OathReward { kind: "title".into(), label: format!("title: {t}"), id: t })
    };
    match kind {
        "bold" => pool(&BOLD_REWARDS, "verb").or_else(|| title(depth_title("bold", goal(l, "bold")))),
        "lean" => pool(&LEAN_REWARDS, "card").or_else(|| title(depth_title("lean", goal(l, "lean")))),
        "tamer" => pool(&SLOT_REWARDS, "slot"),
        "fire" => title(fire_title(boss?)),
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
    let reward = reward_for(l, kind, boss.as_deref(), None)?;
    Some(OathState { id: format!("{kind}:{n}"), kind: kind.into(), depth, boss, seen: if kind == "tamer" { tamed(l) } else { Vec::new() }, n: 0, reward, price: price(l) })
}

/// Whether a standing oath still stands (its reward still to give; a `slayer`'s boss unslain).
fn stands(l: &LineageState, o: &OathState) -> bool {
    if o.kind == "slayer" && o.boss.as_deref().is_some_and(|b| l.kills.contains(b)) {
        return false;
    }
    // (a numbered title stands while its line is the kind's line now and it is not yet owned)
    let now = reward_for(l, &o.kind, o.boss.as_deref(), Some(&o.id));
    match now {
        Some(r) if r.kind == "title" && o.reward.kind == "title" => title_line(&r.id) == title_line(&o.reward.id) && !l.titles.contains(&o.reward.id),
        Some(r) => r.id == o.reward.id,
        None => false,
    }
}

/// Keep the board full: drop the oaths that no longer stand (never the sworn one while it stands),
/// re-price the unsworn to the lineage's income now, and draw new ones from the kinds not on it —
/// in a seeded order, so a lineage's board is a function of its seed and its history.
pub fn refresh(l: &mut LineageState) {
    let sworn = sworn_ids(l);
    let keep: Vec<OathState> = l.oaths.iter().filter(|o| stands(l, o) || sworn.contains(&o.id)).cloned().collect();
    l.oaths = keep;
    if l.oath_sworn.as_ref().is_some_and(|s| !l.oaths.iter().any(|o| &o.id == s)) {
        l.oath_sworn = None;
    }
    let ids: Vec<String> = l.oaths.iter().map(|o| o.id.clone()).collect();
    l.oath_extra.retain(|s| ids.contains(s));
    // (an extra slot's oath moves up when the first slot is free)
    if l.oath_sworn.is_none() && !l.oath_extra.is_empty() {
        l.oath_sworn = Some(l.oath_extra.remove(0));
    }
    let p = price(l);
    let sworn = sworn_ids(l);
    for o in l.oaths.iter_mut().filter(|o| !sworn.contains(&o.id)) {
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
    if sworn_ids(l).iter().any(|s| s == id) {
        return Ok(());
    }
    // Cut 29 §1: a free slot takes it beside the sworn one (`oath_slot_2`, `oath_slot_3`).
    let day = l.day;
    if l.oath_sworn.is_some() && sworn_ids(l).len() < slots(l) {
        if l.gold < o.price {
            return Err("not enough gold".into());
        }
        l.gold_move(-o.price, &format!("oath {}", o.kind));
        l.oath_extra.push(o.id.clone());
        l.oath_days.insert(o.id.clone(), day);
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
    l.oath_days.insert(o.id.clone(), day);
    Ok(())
}

/// Cut 29 §1: forswear the sworn oath `id` (either slot): half its price back; it stays on the board.
pub fn forswear_id(game: &mut Game, id: &str) -> Result<(), String> {
    if game.lineage.oath_sworn.as_deref() == Some(id) {
        return forswear(game);
    }
    let l = &mut game.lineage;
    if !l.oath_extra.iter().any(|x| x == id) {
        return Err("not sworn".into());
    }
    let o = l.oaths.iter().find(|o| o.id == id).cloned().ok_or("no such oath")?;
    l.gold_move(o.price * REFUND_PCT / 100, &format!("forswear {}", o.kind));
    l.oath_extra.retain(|x| x != id);
    l.oath_days.remove(id);
    refresh(l);
    Ok(())
}

/// Forswear the sworn oath: half its price back; it stays on the board.
pub fn forswear(game: &mut Game) -> Result<(), String> {
    let l = &mut game.lineage;
    let o = sworn(l).cloned().ok_or("no oath sworn")?;
    l.gold_move(o.price * REFUND_PCT / 100, &format!("forswear {}", o.kind));
    l.oath_sworn = None;
    l.oath_days.remove(&o.id);
    refresh(l);
    Ok(())
}

/// Did this run keep the oath?
pub fn kept(o: &OathState, run: &Run) -> bool {
    let tier = run.over.unwrap_or(ExitTier::Return);
    let boss_slain = |b: &str| run.boss_kills.iter().any(|(_, k)| k == b);
    match o.kind.as_str() {
        // (Cut 28b: a `return` a row committed to breaks it there — the walk home that dies on the way keeps nothing)
        "bold" => run.max_depth >= o.depth && (tier != ExitTier::Return || run.over.is_none()) && !run.timed_out && !run.home_return,
        "lean" => run.max_depth >= o.depth && !run.rested,
        "tamer" => run.tamed.iter().any(|(_, k)| !o.seen.contains(k)),
        "fire" => o.boss.as_deref().is_some_and(|b| boss_slain(b) && run.burned.iter().any(|k| k == b)),
        "slayer" => o.boss.as_deref().is_some_and(boss_slain),
        _ => false,
    }
}

/// Cut 28b: a sworn oath's fate is said once in the run, the moment it is decided (`Ev::Oath`):
/// kept the moment a lasting condition is met (a new kind tamed, the boss slain — or slain burning)
/// or, for a depth oath, when the run ends past the floor with its rule held; broken the moment the
/// forbidden tool is used (`no rest`: a rest; `no return`: a `return` committed, or the run
/// returned — stalled, driven off); missed when the run ends short of it. `row` is the row whose
/// verb did it (−1: a chore, a trait, the run's end). The run's notes carry it into the chronicle;
/// the settle (`engine`) carries the cause into the exit line and the report.
pub fn beat(run: &mut Run, cx: &mut crate::engine::Ctx, row: i32) {
    let Some(o) = run.oath.as_ref() else { return };
    if run.oath_said.is_some() {
        return;
    }
    let ended = run.over.is_some();
    let lasting = matches!(o.kind.as_str(), "tamer" | "fire" | "slayer");
    let broke = match o.kind.as_str() {
        "lean" => run.rested.then_some("rest"),
        "bold" if run.home_return => Some("return"),
        "bold" if run.over == Some(ExitTier::Return) => Some(if run.timed_out { "stalled" } else if run.driven_off.is_some() { "driven" } else { "return" }),
        _ => None,
    };
    let (ok, cause) = if kept(o, run) && (ended || lasting) {
        (true, "")
    } else if let Some(c) = broke {
        (false, c)
    } else if ended {
        (false, "")
    } else {
        return;
    };
    let row = if matches!(cause, "stalled" | "driven") { -1 } else { row };
    let verb = (row >= 0).then(|| cx.rules.rows.get(row as usize).map(|r| r.verb.short())).flatten();
    // the kept oath's row is the verb that did it (`R3 tame`); a broken one's the tool it forbade (`R2 return`)
    let word = if ok { verb.clone().unwrap_or_default() } else { cause.to_string() };
    let text = match (row >= 0 && !word.is_empty(), word.is_empty()) {
        (true, _) => format!("R{} {word}", row + 1),
        (false, false) => word.clone(),
        _ => String::new(),
    };
    cx.events.push(crate::wire::Ev::Oath { t: run.turn, kept: ok, row, cause: word });
    if ok {
        crate::chronicle::note(run, cx, "Kept the oath.".into());
    } else if !cause.is_empty() {
        crate::chronicle::note(run, cx, format!("Broke the oath: {text}."));
    }
    run.oath_said = Some((ok, if ok || !cause.is_empty() { text } else { String::new() }));
}

/// Cut 28b (AW: a boss oath at `9% ±8` with no visible lever): the steps a send took toward the
/// oath, as bits — `STEP_FLOOR` its floor reached (the goal floor, or the boss's), `STEP_MET` the
/// boss met, `STEP_BURNED` the boss burned. The panel reads each as a share (`OathShare.steps`).
pub const STEP_FLOOR: u8 = 1;
pub const STEP_MET: u8 = 2;
pub const STEP_BURNED: u8 = 4;
pub fn steps(o: &OathState, run: &Run) -> u8 {
    let mut s = 0;
    if o.depth > 0 && run.max_depth >= o.depth {
        s |= STEP_FLOOR;
    }
    if let Some(b) = o.boss.as_deref() {
        if run.boss_seen_t.is_some() && run.max_depth >= o.depth || run.boss_kills.iter().any(|(_, k)| k == b) || run.burned.iter().any(|k| k == b) {
            s |= STEP_MET;
        }
        if run.burned.iter().any(|k| k == b) {
            s |= STEP_BURNED;
        }
    }
    s
}

/// The steps a kind's panel names, in order (≤ 2 words each): a depth oath its floor (`D9`, the
/// share that reached it — the rest is what its rule cost), a boss oath the floor and the meeting,
/// and a `fire` oath the burning too.
pub fn step_names(o: &OathState) -> Vec<(String, u8)> {
    let d = format!("D{}", o.depth);
    match o.kind.as_str() {
        "bold" | "lean" => vec![(d, STEP_FLOOR)],
        "slayer" => vec![(d, STEP_FLOOR), ("met".into(), STEP_MET)],
        "fire" => vec![(d, STEP_FLOOR), ("met".into(), STEP_MET), ("burned".into(), STEP_BURNED)],
        _ => Vec::new(),
    }
}

/// Cut 28b: the board has a use — a band boss seen, a plateau met (`LineageState::oath_open`), or an
/// oath sworn or kept before (an old save), or a band boss slain or his counter known.
pub fn open(l: &LineageState) -> bool {
    l.oath_open
        || l.oath_sworn.is_some()
        || !l.titles.is_empty()
        || l.oaths_kept > 0
        || crate::descent::BOSS_DEPTHS.iter().any(|(k, _)| l.kills.contains(*k) || crate::facts::boss_counter_known(&l.facts, k))
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
        "card" | "verb" | "row" | "slot" | "route" | "heir" => {
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
    for id in BOLD_REWARDS.iter().chain(LEAN_REWARDS.iter()).chain(SLOT_REWARDS.iter()) {
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
    l.oath_days.remove(&o.id);
    l.oaths.retain(|x| x.id != o.id);
    l.oaths_kept += 1;
    refresh(l);
    Some((o, true))
}

/// Cut 29 §1: the oaths sworn in the extra slots, settled against a finished real run — the ones
/// kept (granted, off the board). They are read at the run's end (the forecast and the run's
/// beats follow the first slot's).
pub fn settle_extra(l: &mut LineageState, run: &Run) -> Vec<OathState> {
    let mut out = Vec::new();
    for id in l.oath_extra.clone() {
        let Some(o) = l.oaths.iter().find(|o| o.id == id).cloned() else { continue };
        if kept(&o, run) {
            grant(l, &o);
            l.oath_extra.retain(|x| *x != id);
            l.oath_days.remove(&id);
            l.oaths.retain(|x| x.id != id);
            l.oaths_kept += 1;
            out.push(o);
        }
    }
    if !out.is_empty() {
        refresh(l);
    }
    out
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
                sworn: sworn_ids(l).contains(&o.id),
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
        // (Cut 28b, AX: "never understood what `D3 · no return` required" — the floor is a goal: `reach D3`)
        "bold" => vec![format!("reach D{}", o.depth), "no return".into()],
        "lean" => vec![format!("reach D{}", o.depth), "no rest".into()],
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
    let at = |bit: u8| ended.iter().filter(|r| r.oath_steps & bit != 0).count() as f64 / n.max(1) as f64;
    let steps = step_names(o).into_iter().map(|(k, bit)| crate::wire::OathStep { k, share: at(bit) }).collect();
    Some(OathShare { id: o.id.clone(), text: text(o), share: p, pm: crate::forecast::half_width(p, n), night: night(p), steps })
}

/// The chance a night of `NIGHT` sends keeps an oath each send keeps with `p`.
pub fn night(p: f64) -> f64 {
    1.0 - (1.0 - p.clamp(0.0, 1.0)).powi(NIGHT)
}

/// Cut 29 §1: the oath draw on the wire.
pub fn draw_wire(l: &LineageState) -> crate::wire::OathDraw {
    let cost = crate::meta::OATH_DRAW_COST;
    let needs = if crate::meta::tier(l) < crate::meta::OATH_DRAW_TIER {
        Some(crate::meta::tier_need(crate::meta::OATH_DRAW_TIER).to_string())
    } else if l.marks < cost {
        Some(format!("◆{} more", cost - l.marks))
    } else {
        None
    };
    crate::wire::OathDraw { cost, available: needs.is_none(), needs }
}
