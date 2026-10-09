//! Cut 118 (docs/CUT118_IDLE_LESSONS.md, research/IDLE_STEAM_2026-10_ROUND2.md): lessons from the idle genre, the
//! core's half. Every piece is deterministic, drawn from the lineage seed and the run or week it belongs to, never
//! from a game's dice, so a lineage that never touches them plays as before:
//! - **boss tokens** (a band boss slain banks one, kept across heirs; spending one makes the next send seek him),
//! - **trials** (one optional restricted wall a week of the absence clock, the last four open, cleared offline),
//! - **away finds** (each run home while away seals one; a second table after 6 h; sets give ≤ 2 % bonuses),
//! - **gold sinks after Kit complete** (a ration a send, the tithe, the survey work: adds only, never upkeep),
//! - **swift floors** (three clears of a band boss: the floors above him play as one beat in the watch),
//! - **feats** (a trial or a sought boss cleared lights the next waiting system early; `systems::update_with`),
//! - **the hero's wish** (one at a time, upside only, never decaying).
//!
//! IDLE never spends a token, never opts into a trial, never grants a wish and hires no apprentice, so none of
//! those touch its games; its finds feed only the small set bonuses (rest −2 %, class xp +2 %, +1 Legacy a return).
use crate::descent::{Affix, Route, BOSS_DEPTHS};
use crate::engine::{ExitTier, Game, LineageState, Run, DAY_S, WAYSTONES};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Tokens banked per boss at most.
pub const TOKEN_CAP: u32 = 3;
/// Clears of a band boss that make the floors above him swift.
pub const SWIFT_CLEARS: u32 = 3;
/// A trial's week of the lineage's clock.
pub const WEEK_S: u64 = 7 * DAY_S;
/// Trials open at once: this week's and the three before.
pub const TRIAL_WEEKS_OPEN: u32 = 4;
/// A trial's clear pays this much Legacy (≤ the return pick's largest, `returns::LEGACY_POINTS`, and well under a
/// day's runs' Legacy: one point a run at least).
pub const TRIAL_LEGACY: u32 = 10;
/// The finds' second table opens this far into an absence.
pub const SECOND_TABLE_S: u64 = 6 * 3600;
/// A set's bonus, percent (never wall-deciding).
pub const SET_PCT: u32 = 2;
/// The next wish waits this long after the last one granted.
pub const WISH_EVERY_S: u64 = DAY_S;
/// The tithe: the first Legacy point's price in forge units, and how fast the price climbs (one unit more per
/// `TITHE_STEP` points bought: the rate falls).
pub const TITHE_UNITS: i32 = 1;
pub const TITHE_STEP: u32 = 6;
/// The apprentice's tithe leaves this many units of the town's wealth (purse and bank) untouched.
pub const TITHE_RESERVE_UNITS: i32 = 20;
/// The apprentice's tithe takes at most this many points an hour.
pub const TITHE_PER_HOUR: u32 = 3;
/// A ration: its price in forge units, the max hp it adds for one run (percent, at least `RATION_MIN_HP`).
pub const RATION_UNITS: i32 = 1;
pub const RATION_PCT: i32 = 10;
pub const RATION_MIN_HP: i32 = 2;
/// The survey (a work that opens the deep forks early): its price in forge units.
pub const SURVEY_UNITS: i32 = 20;
/// The siege: a band boss's edge per heir who died at him (percent of the hero's blows), and its cap.
pub const SIEGE_PCT: u32 = 2;
pub const SIEGE_CAP: u32 = 10;
/// Owner amendment 2: a siege try's deed Legacy, and a title's lasting edge on its boss (percent).
pub const DEED_LEGACY: u32 = 1;
pub const TITLE_PCT: u32 = 3;
/// Stones kept in the graveyard (`Feats::stones`).
pub const STONES_CAP: usize = 40;
/// The heir order (`StandingSwitches::heir`): which offered heir succeeds a death, the default first —
/// `answer` the killer, `strongest`, `surprise` me.
pub const HEIR_ORDERS: [&str; 3] = ["answer", "strongest", "surprise"];
/// The apprentice's sink orders (`StandingSwitches::sink`), the default first.
pub const SINK_ORDERS: [&str; 4] = ["both", "ration", "tithe", "off"];

/// A sealed find.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Find {
    pub id: String,
    /// `cosmetic` · `shard` · `piece`.
    pub kind: String,
    pub name: String,
    /// 1 (plain) .. 4 (rare): the reveal shows the best first.
    pub rank: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub set: Option<String>,
    /// The table it came from (1, or 2 after `SECOND_TABLE_S` away).
    pub table: u8,
}

/// A notable act for the report (`ReturnReport.feats`) and the crier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FeatNews {
    /// `token` · `seek` · `trial` · `trial_failed` · `set` · `swift` · `wish` · `lit`.
    pub k: String,
    pub text: String,
    /// The lineage day it happened on (1-based).
    pub day: u32,
}

/// What a send is after: a sought boss (a token spent) or a trial.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Goal {
    pub run: u32,
    pub boss: String,
    pub depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub week: Option<u32>,
    /// Trial runs: the set sent wore the barred tactic.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub broke_rule: bool,
}

/// The amendment's siege on one band boss: the heirs who died at him, the lowest hp share he was left on.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Siege {
    pub tries: u32,
    /// The lowest hp he was left on at a death (percent of his max; 100 untouched).
    pub best_pct: u32,
    /// The heirs who wore him down (their names).
    pub heirs: Vec<String>,
}

/// The amendment's grave: a dead heir's lost carry where he fell, until a later heir reaches the floor.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Grave {
    pub depth: u32,
    pub heir: u32,
    pub name: String,
    pub gold: i32,
    /// The run that left it (only a later run recovers it).
    pub run: u32,
    pub day: u32,
}

/// The amendment's memorial: one stone per fallen heir.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Stone {
    pub heir: u32,
    pub name: String,
    pub depth: u32,
    pub cause: String,
    /// `fell to the Queen, D28`.
    pub epitaph: String,
    pub day: u32,
    pub run: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boss: Option<String>,
    /// The siege's try this death made (at a band boss), and the edge it leaves the next heir.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub try_n: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub edge_pct: u32,
    /// The carry left in his grave.
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub grave_gold: i32,
    /// The Legacy his run's deeds paid (the run's own and a siege try's).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub legacy: u32,
}

fn is_zero_i32(n: &i32) -> bool {
    *n == 0
}

/// The lineage's Cut 118 state (`LineageState::feats`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Feats {
    /// Boss → tokens banked (cap `TOKEN_CAP`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tokens: BTreeMap<String, u32>,
    /// The order: the boss the next send seeks (a token spent at the send). `None`: in order (the default).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seek: Option<String>,
    /// The send under way's goal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<Goal>,
    /// The trial opted into (its week), played by the next send while away.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial: Option<u32>,
    /// Weeks whose trial is cleared; the marks they earned.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub trials: BTreeSet<u32>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub trial_marks: u32,
    /// Finds sealed this absence (opened on the return).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sealed: Vec<Find>,
    /// The find log: piece/cosmetic id → times found.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub log: BTreeMap<String, u32>,
    /// Sets completed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub sets: BTreeSet<String>,
    /// Bosses whose swift floors were announced.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub swift: BTreeSet<String>,
    /// Tithe points bought so far (the rate falls with them).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub tithed: u32,
    /// A ration bought for the next send (the apprentice's, at the send).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub ration: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rations: u32,
    /// Wishes granted; the clock the next wish opens at.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub wishes: u32,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub wish_at: u64,
    /// Feats earned (a trial cleared, a sought boss slain) and spent lighting a system early.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub feats: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub feats_used: u32,
    /// System → what lit it (`slay Warlord` the trigger, `time` the fallback, `feat: Mother` a feat).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lit_by: BTreeMap<String, String>,
    /// The last feat's label (what a feat-lit system names).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub last_feat: String,
    /// Notable acts since the last report.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub news: Vec<FeatNews>,
    /// The amendment: the siege per band boss, the graves, the graveyard's stones, and the heirs who wore down a
    /// boss now fallen (boss → names).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub siege: BTreeMap<String, Siege>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub graves: Vec<Grave>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stones: Vec<Stone>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub worn: BTreeMap<String, Vec<String>>,
    /// Owner amendment 2: the bosses whose siege was won (each a title, `TITLE_PCT` on him), and the Legacy deeds paid.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub titles: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub deed_legacy: u32,
    /// A probe's or a test's pin: every Cut 118 system off (no tokens, finds, sinks, swift, bonuses).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}
fn is_zero_u64(n: &u64) -> bool {
    *n == 0
}

impl Feats {
    pub fn is_empty(&self) -> bool {
        *self == Feats::default()
    }
}

fn mix(seed: u64, a: u64, salt: u64) -> u64 {
    crate::rng::splitmix(seed ^ a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ salt.wrapping_mul(0xC2B2_AE3D_27D4_EB4F))
}

/// The lineage day (1-based) at clock `clock_s`.
pub fn day_of(clock_s: u64) -> u32 {
    (clock_s / DAY_S) as u32 + 1
}

/// The clock now: inside an absence, the absence's moment.
pub fn clock_now(g: &Game) -> u64 {
    g.clock_at.unwrap_or(g.lineage.clock_s)
}

/// The seconds into the absence under way (0 at camp).
pub fn away_s(g: &Game) -> u64 {
    if let Some(c) = g.clock_at {
        return c.saturating_sub(g.lineage.clock_s);
    }
    g.offline_absence.as_ref().map_or(0, |a| a.elapsed_s)
}

/// Whether a run ending now ends inside an absence (single hero: `offline`; a session's: its absence).
pub fn away(g: &Game) -> bool {
    g.offline || g.offline_absence.is_some()
}

fn live(g: &Game) -> bool {
    !g.sim && !g.lineage.feats.off
}

pub(crate) fn news(l: &mut LineageState, k: &str, text: String, day: u32) {
    l.feats.news.push(FeatNews { k: k.into(), text, day });
    let n = l.feats.news.len();
    if n > 24 {
        l.feats.news.drain(..n - 24);
    }
}

// ---------------------------------------------------------------- the goal: tokens and trials

/// The deepest lit stone at or above `depth` on `route` (1 when none).
pub fn stone_for(l: &LineageState, depth: u32, route: Route) -> u32 {
    WAYSTONES.iter().copied().filter(|w| *w <= depth && l.stone_lit(*w, route)).max().unwrap_or(1)
}

/// A band boss's depth on `route` (the base order's when the route has none for him).
fn boss_at(kind: &str, route: Route) -> Option<u32> {
    route.boss_depth(kind).or_else(|| crate::descent::boss_depth(kind))
}

/// The order: seek `boss` with the next send (a token spent then), or `""` for in order.
pub fn set_seek(l: &mut LineageState, boss: &str) -> Result<(), String> {
    if boss.is_empty() {
        l.feats.seek = None;
        return Ok(());
    }
    if !BOSS_DEPTHS.iter().any(|(k, _)| *k == boss) {
        return Err("not a band boss".into());
    }
    if l.feats.tokens.get(boss).copied().unwrap_or(0) == 0 {
        return Err("no token".into());
    }
    l.feats.seek = Some(boss.into());
    Ok(())
}

/// At a real send (before the start is paid): the goal this send carries — a trial opted into (while away) or a
/// sought boss (its token spent) — and the stone it starts from. `None`: the lineage's own start.
pub fn at_send(g: &mut Game, run_id: u32) -> Option<u32> {
    g.lineage.feats.goal = None;
    if !live(g) || g.lineage.pkg.literal {
        return None;
    }
    let route = g.lineage.rules().route();
    if let Some(week) = g.lineage.feats.trial.filter(|_| away(g)) {
        g.lineage.feats.trial = None;
        if let Some(t) = trial_of(&g.lineage, week).filter(|t| t.open) {
            let depth = boss_at(&t.boss, route).unwrap_or(t.depth);
            let broke = matches!(&t.rule, Rule::Barred(x) if crate::packages::wears(g.lineage.rules(), x));
            g.lineage.feats.goal = Some(Goal { run: run_id, boss: t.boss.clone(), depth, week: Some(week), broke_rule: broke });
            return Some(stone_for(&g.lineage, depth, route));
        }
    }
    let boss = g.lineage.feats.seek.take()?;
    let n = g.lineage.feats.tokens.get(&boss).copied().unwrap_or(0);
    let depth = boss_at(&boss, route)?;
    if n == 0 {
        return None;
    }
    if n == 1 {
        g.lineage.feats.tokens.remove(&boss);
    } else {
        g.lineage.feats.tokens.insert(boss.clone(), n - 1);
    }
    g.lineage.feats.goal = Some(Goal { run: run_id, boss, depth, week: None, broke_rule: false });
    Some(stone_for(&g.lineage, depth, route))
}

/// The goal of run `id`, if it has one.
pub fn goal_of(l: &LineageState, id: u32) -> Option<&Goal> {
    l.feats.goal.as_ref().filter(|g| g.run == id)
}

/// The trial run's affixes on its boss (the rule's two in place of the heir's draw).
pub fn goal_affixes(l: &LineageState, id: u32, affixes: &mut Vec<(String, Affix)>) {
    let Some(g) = goal_of(l, id) else { return };
    let Some(week) = g.week else { return };
    let Some(t) = trial_of(l, week) else { return };
    if let Rule::Affixes(a, b) = t.rule {
        affixes.retain(|(k, _)| *k != t.boss);
        affixes.push((t.boss.clone(), a));
        affixes.push((t.boss.clone(), b));
    }
}

// ---------------------------------------------------------------- trials

/// A trial's restriction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Rule {
    /// The boss cleared without this tactic worn.
    Barred(String),
    /// The boss wears both affixes.
    Affixes(Affix, Affix),
}

/// One week's trial.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Trial {
    pub week: u32,
    pub boss: String,
    pub depth: u32,
    pub rule: Rule,
    /// Reachable now (the boss's floor at or above the best depth, the boss slain once).
    pub open: bool,
}

/// The tactics a trial may bar (each leaves the stances and the other tactics to answer).
pub const BARRED: [&str; 3] = ["boss_focus", "kite_archers", "corridor_fighting"];

/// The restrictions a boss's trial draws from: a tactic barred, or two of his affixes.
pub fn rules_for(boss: &str) -> Vec<Rule> {
    let mut out: Vec<Rule> = BARRED.iter().map(|t| Rule::Barred((*t).into())).collect();
    let pool = crate::descent::affix_pool(boss);
    for pair in pool.chunks(2) {
        if let [a, b] = pair {
            out.push(Rule::Affixes(*a, *b));
        }
    }
    out
}

/// The packages that answer `rule` on `boss`: the affixes' breakers, or every stance and tactic but the barred one.
pub fn answers(boss: &str, rule: &Rule) -> Vec<&'static str> {
    match rule {
        Rule::Affixes(a, b) => {
            let mut v: Vec<&'static str> = crate::descent::affix_breakers(boss, *a).to_vec();
            for x in crate::descent::affix_breakers(boss, *b) {
                if !v.contains(x) {
                    v.push(x);
                }
            }
            v
        }
        Rule::Barred(t) => crate::packages::PACKAGES
            .iter()
            .filter(|p| matches!(p.kind, crate::packages::Kind::Stance | crate::packages::Kind::Tactic) && p.id != t && p.id != "custom")
            .map(|p| p.id)
            .collect(),
    }
}

/// The trial of `week` (a pure draw from the week and the lineage seed). The King draws none.
pub fn trial_of(l: &LineageState, week: u32) -> Option<Trial> {
    let bosses = &BOSS_DEPTHS[..BOSS_DEPTHS.len() - 1];
    let h = mix(l.seed, u64::from(week), 0x71A15);
    let (boss, depth) = bosses[(h % bosses.len() as u64) as usize];
    let pool = rules_for(boss);
    let rule = pool[((h >> 16) % pool.len() as u64) as usize].clone();
    let open = l.kills.contains(boss) && depth <= l.best_depth.max(1);
    Some(Trial { week, boss: boss.into(), depth, rule, open })
}

/// The lineage's week now.
pub fn week_now(l: &LineageState) -> u32 {
    (l.clock_s / WEEK_S) as u32
}

/// The open weeks: this one and the three before (never before week 0).
pub fn open_weeks(l: &LineageState) -> Vec<u32> {
    let w = week_now(l);
    (w.saturating_sub(TRIAL_WEEKS_OPEN - 1)..=w).collect()
}

/// Opt into week `week`'s trial (`None` opts out): the next send while away plays it.
pub fn set_trial(l: &mut LineageState, week: Option<u32>) -> Result<(), String> {
    let Some(w) = week else {
        l.feats.trial = None;
        return Ok(());
    };
    if !open_weeks(l).contains(&w) {
        return Err("trial closed".into());
    }
    if l.feats.trials.contains(&w) {
        return Err("trial cleared".into());
    }
    let t = trial_of(l, w).ok_or("no trial")?;
    if !t.open {
        return Err(format!("slay {}", crate::sifter::boss_short(&t.boss)));
    }
    l.feats.trial = Some(w);
    Ok(())
}

fn rule_label(rule: &Rule) -> String {
    match rule {
        Rule::Barred(t) => format!("no {}", crate::packages::name(t)),
        Rule::Affixes(a, b) => format!("{} + {}", a.word(), b.word()),
    }
}

// ---------------------------------------------------------------- finds

struct FindDef {
    id: &'static str,
    kind: &'static str,
    name: &'static str,
    rank: u8,
    set: Option<&'static str>,
}

const fn fd(id: &'static str, kind: &'static str, name: &'static str, rank: u8, set: Option<&'static str>) -> FindDef {
    FindDef { id, kind, name, rank, set }
}

/// The first table (any absence).
const TABLE1: [FindDef; 8] = [
    fd("pebble", "cosmetic", "painted pebble", 1, None),
    fd("moth", "cosmetic", "moth lantern", 1, None),
    fd("banner", "cosmetic", "crooked banner", 1, None),
    fd("shard", "shard", "legacy shard", 2, None),
    fd("pillow", "piece", "sleeper's pillow", 3, Some("sleeper")),
    fd("quill", "piece", "scholar's quill", 3, Some("scholar")),
    fd("stone", "piece", "hearth stone", 3, Some("hearth")),
    fd("blanket", "piece", "sleeper's blanket", 3, Some("sleeper")),
];

/// The second table (after `SECOND_TABLE_S` away): richer — each set's last piece is here.
const TABLE2: [FindDef; 8] = [
    fd("gilded", "cosmetic", "gilded banner", 2, None),
    fd("bright", "shard", "bright shard", 3, None),
    fd("candle", "piece", "sleeper's candle", 4, Some("sleeper")),
    fd("lens", "piece", "scholar's lens", 4, Some("scholar")),
    fd("tome", "piece", "scholar's tome", 4, Some("scholar")),
    fd("kettle", "piece", "hearth kettle", 4, Some("hearth")),
    fd("rug", "piece", "hearth rug", 4, Some("hearth")),
    fd("quill", "piece", "scholar's quill", 3, Some("scholar")),
];

/// The sets: id, title, pieces, bonus (what it gives, ≤ `SET_PCT` %).
pub const SETS: [(&str, &str, [&str; 3], &str); 3] = [
    ("sleeper", "Sleeper's set", ["pillow", "blanket", "candle"], "rest −2%"),
    ("scholar", "Scholar's set", ["quill", "lens", "tome"], "class xp +2%"),
    ("hearth", "Hearth set", ["stone", "kettle", "rug"], "+1 Legacy a return"),
];

/// A set bonus in percent (`rest`, `xp`), 0 without the set.
pub fn bonus_pct(l: &LineageState, what: &str) -> u32 {
    if l.feats.off {
        return 0;
    }
    let set = match what {
        "rest" => "sleeper",
        "xp" => "scholar",
        _ => return 0,
    };
    if l.feats.sets.contains(set) {
        SET_PCT
    } else {
        0
    }
}

fn draw_find(seed: u64, run: u32, table: u8) -> Find {
    let h = mix(seed, u64::from(run), 0xF14D5);
    let t: &[FindDef] = if table >= 2 { &TABLE2 } else { &TABLE1 };
    let d = &t[(h % t.len() as u64) as usize];
    Find { id: d.id.into(), kind: d.kind.into(), name: d.name.into(), rank: d.rank, set: d.set.map(str::to_string), table }
}

/// The finds' reveal on the return (`ReturnReport.finds`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FindsReveal {
    /// How many were sealed this absence.
    pub sealed: u32,
    /// The best one (the reveal shows it first).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best: Option<Find>,
    /// Every find, best first.
    pub finds: Vec<Find>,
    /// Legacy the shards gave.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub legacy: u32,
    /// Sets completed by these finds (their titles).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sets: Vec<String>,
}

// ---------------------------------------------------------------- the run's end

/// A real run home: tokens, the goal's outcome, a find sealed (away), the swift floors.
pub fn on_run_end(g: &mut Game, run: &Run, tier: ExitTier, legacy_earned: u32) {
    if !live(g) {
        return;
    }
    let day = day_of(clock_now(g));
    let route = run.route;
    let slew: Vec<String> = run.boss_kills.iter().map(|(_, k)| k.clone()).filter(|k| BOSS_DEPTHS.iter().any(|(b, _)| b == k)).collect();
    // tokens: one per band boss slain (kept across heirs; capped)
    for k in &slew {
        let n = g.lineage.feats.tokens.entry(k.clone()).or_insert(0);
        if *n < TOKEN_CAP {
            *n += 1;
        }
    }
    // the goal
    if let Some(goal) = g.lineage.feats.goal.take().filter(|x| x.run == run.id) {
        let short = crate::sifter::boss_short(&goal.boss);
        let slain = slew.contains(&goal.boss);
        match goal.week {
            Some(week) if slain && !goal.broke_rule => {
                g.batch.legacy_earned += TRIAL_LEGACY;
                let l = &mut g.lineage;
                l.feats.trials.insert(week);
                l.feats.trial_marks += 1;
                crate::legacy::ensure(l);
                l.bloodline.as_mut().expect("bloodline").points += TRIAL_LEGACY;
                l.feats.feats += 1;
                l.feats.last_feat = format!("trial: {short}");
                news(l, "trial", format!("trial cleared · {short} · +{TRIAL_LEGACY} Legacy"), day);
            }
            Some(_) => news(&mut g.lineage, "trial_failed", format!("trial · {short} · {}", if goal.broke_rule { "rule broken" } else { tier.name() }), day),
            None if slain => {
                g.lineage.feats.feats += 1;
                g.lineage.feats.last_feat = format!("slay {short}");
                news(&mut g.lineage, "seek", format!("{short} sought · slain"), day);
            }
            None => news(&mut g.lineage, "seek", format!("{short} sought · D{}", run.max_depth), day),
        }
    }
    // the amendment: graves recovered, a death's grave and stone, the siege
    recover_graves(g, run, day);
    if tier == ExitTier::Death {
        fallen(g, run, day, legacy_earned);
    }
    for k in &slew {
        if let Some(s) = g.lineage.feats.siege.remove(k) {
            let short = crate::sifter::boss_short(k);
            news(&mut g.lineage, "siege_won", format!("{short} fell · {} tries", s.tries + 1), day);
            g.lineage.feats.worn.insert(k.clone(), s.heirs);
            // (owner amendment 2: a siege won names the line — one title a boss, a small lasting edge on him)
            if g.lineage.feats.titles.insert(k.clone()) {
                let t = title_of(k);
                if !g.lineage.titles.contains(&t) {
                    g.lineage.titles.push(t.clone());
                }
                news(&mut g.lineage, "title", format!("the line is {t}"), day);
            }
        }
    }
    // a find sealed for the return
    if away(g) {
        let table = if away_s(g) >= SECOND_TABLE_S { 2 } else { 1 };
        let f = draw_find(g.lineage.seed, run.id, table);
        g.lineage.feats.sealed.push(f);
    }
    // swift floors
    for k in &slew {
        let clears = g.lineage.kill_counts.get(k).copied().unwrap_or(0);
        if clears >= SWIFT_CLEARS && g.lineage.feats.swift.insert(k.clone()) {
            let d = boss_at(k, route).unwrap_or(1);
            news(&mut g.lineage, "swift", format!("D1–{} swift", d.saturating_sub(1)), day);
        }
    }
}

/// The floors above this depth are swift (one beat in the watch): the deepest band boss cleared `SWIFT_CLEARS`
/// times. 0 when none.
pub fn swift_to(l: &LineageState, route: Route) -> u32 {
    if l.feats.off {
        return 0;
    }
    BOSS_DEPTHS.iter().filter(|(k, _)| l.kill_counts.get(*k).copied().unwrap_or(0) >= SWIFT_CLEARS).filter_map(|(k, _)| boss_at(k, route)).max().unwrap_or(0)
}

/// At the return: open the sealed finds (shards pay, pieces log, sets complete), the hearth's Legacy, the wish's
/// arrival; the report's reveal and the notable acts since the last report.
pub fn on_report(l: &mut LineageState, r: &mut crate::wire::ReturnReport) {
    let day = day_of(l.clock_s);
    let sealed = std::mem::take(&mut l.feats.sealed);
    if !sealed.is_empty() {
        let mut legacy = 0;
        let mut sets = Vec::new();
        for f in &sealed {
            *l.feats.log.entry(f.id.clone()).or_insert(0) += 1;
            if f.kind == "shard" {
                legacy += if f.table >= 2 { 2 } else { 1 };
            }
        }
        for (id, title, pieces, bonus) in SETS {
            if !l.feats.sets.contains(id) && pieces.iter().all(|p| l.feats.log.contains_key(*p)) {
                l.feats.sets.insert(id.into());
                sets.push(title.to_string());
                news(l, "set", format!("{title} · {bonus}"), day);
            }
        }
        if l.feats.sets.contains("hearth") {
            legacy += 1;
        }
        if legacy > 0 {
            crate::legacy::ensure(l);
            l.bloodline.as_mut().expect("bloodline").points += legacy;
            r.legacy_earned += legacy;
        }
        let mut finds = sealed;
        finds.sort_by(|a, b| b.rank.cmp(&a.rank).then(b.table.cmp(&a.table)).then(a.id.cmp(&b.id)));
        r.finds = Some(FindsReveal { sealed: finds.len() as u32, best: finds.first().cloned(), finds, legacy, sets });
    }
    if l.feats.wish_at == 0 && !l.feats.off {
        l.feats.wish_at = l.clock_s.max(1);
    }
    r.feats = std::mem::take(&mut l.feats.news);
}

// ---------------------------------------------------------------- sinks

/// Every forge ladder at its top.
pub fn kit_complete(l: &LineageState) -> bool {
    crate::kit::KIT_SLOTS.iter().all(|s| crate::kit::owned(l, s) as usize >= crate::kit::mults(s).len())
}

fn unit(l: &LineageState) -> i32 {
    crate::kit::unit(l.best_depth) as i32
}

/// The next tithe point's price (the rate falls as the points are bought).
pub fn tithe_price(l: &LineageState) -> i32 {
    TITHE_UNITS * unit(l) * (TITHE_STEP + l.feats.tithed) as i32 / TITHE_STEP as i32
}

/// Tithe `n` points by hand (gold → Legacy at the falling rate). The points bought.
pub fn tithe(l: &mut LineageState, n: u32) -> Result<u32, String> {
    if !kit_complete(l) {
        return Err("kit first".into());
    }
    let mut got = 0;
    for _ in 0..n.min(50) {
        let p = tithe_price(l);
        if crate::tree::purse(l) < p {
            break;
        }
        l.gold_move(-p, "tithe");
        l.feats.tithed += 1;
        crate::legacy::ensure(l);
        l.bloodline.as_mut().expect("bloodline").points += 1;
        got += 1;
    }
    if got == 0 {
        return Err("not enough gold".into());
    }
    Ok(got)
}

/// A ration's price.
pub fn ration_price(l: &LineageState) -> i32 {
    RATION_UNITS * unit(l)
}

/// The apprentice's sinks after Kit complete, under his order: a ration for the send (`send`), the tithe on the
/// hour. Only the purse above his reserve; never the chest. The points or rations bought.
pub fn apprentice_sinks(g: &mut Game, send: bool) -> u32 {
    let l = &g.lineage;
    if l.feats.off || l.orders.sink == "off" || !kit_complete(l) {
        return 0;
    }
    let order = l.orders.sink.clone();
    let reserve = crate::tree::RESERVE_UNITS * unit(l);
    let mut n = 0;
    if send && matches!(order.as_str(), "both" | "ration") && !g.lineage.feats.ration {
        let p = ration_price(&g.lineage);
        if crate::tree::purse(&g.lineage) >= p + reserve {
            g.lineage.gold_move(-p, "supply ration");
            g.lineage.feats.ration = true;
            g.lineage.feats.rations += 1;
            n += 1;
        }
    }
    if !send && matches!(order.as_str(), "both" | "tithe") {
        // (the tithe reads the town's whole wealth — purse and bank — above `TITHE_RESERVE_UNITS`: the bank's
        // savings are what a complete kit leaves to pile; the purse keeps the forge's reserve)
        let keep = TITHE_RESERVE_UNITS * unit(&g.lineage);
        let floor = crate::tree::RESERVE_UNITS * unit(&g.lineage);
        for _ in 0..TITHE_PER_HOUR {
            let p = tithe_price(&g.lineage);
            let purse = crate::tree::purse(&g.lineage);
            if purse + g.lineage.town.bank < p + keep {
                break;
            }
            let short = p - (purse - floor).max(0);
            if short > 0 && crate::town::withdraw(&mut g.lineage, short).is_err() {
                break;
            }
            if crate::tree::purse(&g.lineage) < p {
                break;
            }
            g.lineage.gold_move(-p, "tithe");
            g.lineage.feats.tithed += 1;
            crate::legacy::ensure(&mut g.lineage);
            g.lineage.bloodline.as_mut().expect("bloodline").points += 1;
            n += 1;
        }
    }
    n
}

/// At the run's start: the ration bought for it (its max hp).
pub fn eat_ration(l: &mut LineageState, hero: &mut crate::hero::Hero) {
    if !std::mem::take(&mut l.feats.ration) {
        return;
    }
    let add = (hero.max_hp * RATION_PCT / 100).max(RATION_MIN_HP);
    hero.max_hp += add;
    hero.hp += add;
}

/// The survey's price, and whether it is on offer (Kit complete, the deep forks still shut).
pub fn survey_offer(l: &LineageState) -> Option<i32> {
    (kit_complete(l) && !l.unlocks.contains("route2")).then(|| SURVEY_UNITS * unit(l))
}

/// Commission the survey: the deep forks (D19, D24) open early.
pub fn buy_survey(l: &mut LineageState) -> Result<(), String> {
    let p = survey_offer(l).ok_or("not on offer")?;
    if crate::tree::purse(l) < p {
        return Err("not enough gold".into());
    }
    l.gold_move(-p, "works survey");
    l.unlocks.insert("route2".into());
    l.works.push("survey".into());
    Ok(())
}

// ---------------------------------------------------------------- the hero's wish

/// The wishes: id, words, Legacy it pays.
pub const WISHES: [(&str, &str, u32); 5] = [("lantern", "a lantern", 2), ("pet", "a pet", 2), ("rest", "a rest", 2), ("song", "a song", 2), ("map", "a map", 3)];

/// The wish waiting (id, words, price, Legacy), when one has come.
pub fn wish(l: &LineageState) -> Option<(&'static str, &'static str, i32, u32)> {
    if l.feats.off || l.feats.wish_at == 0 || l.clock_s < l.feats.wish_at || l.heir == 0 {
        return None;
    }
    let (id, text, legacy) = WISHES[(mix(l.seed, u64::from(l.feats.wishes), 0x4154) % WISHES.len() as u64) as usize];
    Some((id, text, unit(l) / 2, legacy))
}

/// Grant the waiting wish: its small price, its Legacy; the next one comes a day on.
pub fn grant_wish(l: &mut LineageState) -> Result<String, String> {
    let (_, text, price, legacy) = wish(l).ok_or("no wish")?;
    if crate::tree::purse(l) < price {
        return Err("not enough gold".into());
    }
    l.gold_move(-price, "supply wish");
    crate::legacy::ensure(l);
    l.bloodline.as_mut().expect("bloodline").points += legacy;
    l.feats.wishes += 1;
    l.feats.wish_at = l.clock_s + WISH_EVERY_S;
    let day = day_of(l.clock_s);
    news(l, "wish", format!("granted {text} · +{legacy} Legacy"), day);
    Ok(format!("{text} +{legacy}"))
}

// ---------------------------------------------------------------- the King's horizon

/// The hours until the King may fall at the current pace (`None` slain, or no pace yet): the lineage's hours
/// per floor so far, the floors left weighted deeper (a floor at the bottom takes twice the walk's mean).
pub fn king_eta_h(l: &LineageState) -> Option<u32> {
    if l.kills.contains("mirror_king") {
        return Some(0);
    }
    let best = l.best_depth;
    let age = l.clock_s as f64 / 3600.0;
    if best < 5 || age <= 0.0 {
        return None;
    }
    let pace = age / f64::from(best);
    let end = f64::from(crate::descent::ENDING_DEPTH - 1);
    let left: f64 = ((best + 1)..=(crate::descent::ENDING_DEPTH - 1)).map(|d| 1.0 + f64::from(d) / end).sum();
    Some((pace * left).ceil() as u32)
}

/// The record as a share of the way to the King (percent).
pub fn king_pct(l: &LineageState) -> u32 {
    if l.kills.contains("mirror_king") {
        return 100;
    }
    (l.best_depth * 100 / (crate::descent::ENDING_DEPTH - 1)).min(99)
}

// ---------------------------------------------------------------- the wire

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BossToken {
    pub boss: String,
    pub title: String,
    pub depth: u32,
    pub n: u32,
    pub cap: u32,
    /// The stone a seek starts from.
    pub stone: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TrialWire {
    pub week: u32,
    pub boss: String,
    pub title: String,
    pub depth: u32,
    /// `barred` · `affixes`.
    pub rule: String,
    /// ≤ 3 words: `no boss focus`, `brood + swift`.
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub barred: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affixes: Vec<String>,
    /// The packages that answer it, and those of them the lineage owns.
    pub answers: Vec<String>,
    pub owned: Vec<String>,
    pub open: bool,
    pub cleared: bool,
    pub opted: bool,
    /// Weeks left open (0: this is the oldest).
    pub weeks_left: u32,
    pub legacy: u32,
    /// Why it cannot be opted into now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub needs: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FindSet {
    pub id: String,
    pub title: String,
    pub pieces: Vec<String>,
    pub have: Vec<bool>,
    pub done: bool,
    pub bonus: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Sink {
    /// `tithe` · `ration` · `survey`.
    pub id: String,
    pub price: i32,
    /// The rate shown (`1 Legacy / $900`, `+10% hp · 1 run`, `forks D19 · D24`).
    pub line: String,
    pub available: bool,
    /// Bought so far.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub n: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct WishWire {
    pub id: String,
    pub text: String,
    pub price: i32,
    pub legacy: u32,
    pub available: bool,
}

/// `Lineage.feats`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct FeatsWire {
    pub tokens: Vec<BossToken>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seek: Option<String>,
    pub trials: Vec<TrialWire>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trial: Option<u32>,
    pub trial_marks: u32,
    /// Finds sealed in the absence under way (the away screen's count).
    pub sealed: u32,
    pub sets: Vec<FindSet>,
    /// Floors above this are swift (0: none).
    pub swift_to: u32,
    /// The sinks on offer once the kit is complete (empty before).
    pub sinks: Vec<Sink>,
    pub sink_order: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wish: Option<WishWire>,
    pub feats: u32,
    /// The amendment: the siege per band boss (`Queen · 4 tries · best 22%`), the graves waiting on their floors,
    /// the graveyard's stones (newest last), and the heirs who wore down each fallen boss.
    pub siege: Vec<SiegeWire>,
    pub graves: Vec<GraveWire>,
    pub graveyard: Vec<Stone>,
    pub worn: BTreeMap<String, Vec<String>>,
    /// Owner amendment 2: the heir order (`answer · strongest · surprise`), the line's titles (`Queensbane`, each
    /// +`TITLE_PCT`% on its boss), the Legacy deeds have paid.
    pub heir_order: String,
    pub titles: Vec<String>,
    pub deed_legacy: u32,
}

pub fn wire(l: &LineageState) -> FeatsWire {
    let route = l.rules().route();
    let tokens = BOSS_DEPTHS
        .iter()
        .filter_map(|(k, _)| {
            let n = l.feats.tokens.get(*k).copied().unwrap_or(0);
            let depth = boss_at(k, route)?;
            (n > 0).then(|| BossToken { boss: (*k).into(), title: crate::sifter::boss_short(k).into(), depth, n, cap: TOKEN_CAP, stone: stone_for(l, depth, route) })
        })
        .collect();
    let weeks = open_weeks(l);
    let trials = weeks
        .iter()
        .rev()
        .filter_map(|w| trial_of(l, *w))
        .map(|t| {
            let ans = answers(&t.boss, &t.rule);
            let owned: Vec<String> = ans.iter().filter(|a| l.pkg.owned.contains(**a)).map(|a| a.to_string()).collect();
            let (rule, barred, affixes) = match &t.rule {
                Rule::Barred(x) => ("barred", Some(x.clone()), Vec::new()),
                Rule::Affixes(a, b) => ("affixes", None, vec![a.word().to_string(), b.word().to_string()]),
            };
            let cleared = l.feats.trials.contains(&t.week);
            TrialWire {
                week: t.week,
                title: crate::sifter::boss_short(&t.boss).into(),
                depth: t.depth,
                rule: rule.into(),
                label: rule_label(&t.rule),
                barred,
                affixes,
                answers: ans.iter().map(|a| a.to_string()).collect(),
                owned,
                open: t.open && !cleared,
                cleared,
                opted: l.feats.trial == Some(t.week),
                weeks_left: t.week + TRIAL_WEEKS_OPEN - 1 - week_now(l).min(t.week + TRIAL_WEEKS_OPEN - 1),
                legacy: TRIAL_LEGACY,
                needs: (!t.open && !cleared).then(|| format!("slay {}", crate::sifter::boss_short(&t.boss))),
                boss: t.boss,
            }
        })
        .collect();
    let sets = SETS
        .iter()
        .map(|(id, title, pieces, bonus)| FindSet { id: (*id).into(), title: (*title).into(), pieces: pieces.iter().map(|p| p.to_string()).collect(), have: pieces.iter().map(|p| l.feats.log.contains_key(*p)).collect(), done: l.feats.sets.contains(*id), bonus: (*bonus).into() })
        .collect();
    let mut sinks = Vec::new();
    if kit_complete(l) {
        let p = tithe_price(l);
        sinks.push(Sink { id: "tithe".into(), price: p, line: format!("1 Legacy / ${p}"), available: crate::tree::purse(l) >= p, n: l.feats.tithed });
        let r = ration_price(l);
        sinks.push(Sink { id: "ration".into(), price: r, line: format!("+{RATION_PCT}% hp · 1 run"), available: crate::tree::on(l, "apprentice"), n: l.feats.rations });
        if let Some(s) = survey_offer(l) {
            sinks.push(Sink { id: "survey".into(), price: s, line: "forks D19 · D24".into(), available: crate::tree::purse(l) >= s, n: 0 });
        }
    }
    FeatsWire {
        tokens,
        seek: l.feats.seek.clone(),
        trials,
        trial: l.feats.trial,
        trial_marks: l.feats.trial_marks,
        sealed: l.feats.sealed.len() as u32,
        sets,
        swift_to: swift_to(l, route),
        sinks,
        sink_order: l.orders.sink.clone(),
        wish: wish(l).map(|(id, text, price, legacy)| WishWire { id: id.into(), text: text.into(), price, legacy, available: crate::tree::purse(l) >= price }),
        feats: l.feats.feats.saturating_sub(l.feats.feats_used),
        siege: siege_wire(l),
        graves: l.feats.graves.iter().map(|x| GraveWire { depth: x.depth, heir: x.heir, name: x.name.clone(), gold: x.gold, day: x.day }).collect(),
        graveyard: l.feats.stones.clone(),
        worn: l.feats.worn.clone(),
        heir_order: l.orders.heir.clone(),
        titles: l.feats.titles.iter().map(|b| title_of(b)).collect(),
        deed_legacy: l.feats.deed_legacy,
    }
}

/// A sought boss priced (`Game::seek_forecast`): the stone it starts from, the share of sends that reach his
/// floor and that pass it, on the camp's panel from that stone.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct SeekOption {
    pub boss: String,
    pub title: String,
    pub depth: u32,
    pub stone: u32,
    pub tokens: u32,
    pub reach: f64,
    pub past: f64,
    pub death: f64,
    pub sims: u32,
    pub current: bool,
}

/// Each boss a token is banked for, priced on the camp's panel from his stone.
pub fn seek_forecast(g: &Game) -> Vec<SeekOption> {
    let route = g.lineage.rules().route();
    let rules = g.lineage.rules().clone();
    let mut out = Vec::new();
    for (k, n) in &g.lineage.feats.tokens {
        let Some(depth) = boss_at(k, route) else { continue };
        let stone = stone_for(&g.lineage, depth, route);
        let mut s = g.sim_clone();
        s.lineage.start = stone;
        let ended = crate::forecast::camp_panel(&s, &rules, crate::forecast::FORECAST_SIMS);
        let m = ended.len().max(1) as f64;
        out.push(SeekOption {
            boss: k.clone(),
            title: crate::sifter::boss_short(k).into(),
            depth,
            stone,
            tokens: *n,
            reach: ended.iter().filter(|r| r.max_depth >= depth).count() as f64 / m,
            past: ended.iter().filter(|r| r.max_depth > depth).count() as f64 / m,
            death: ended.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / m,
            sims: ended.len() as u32,
            current: g.lineage.feats.seek.as_deref() == Some(k.as_str()),
        });
    }
    out
}

// ---------------------------------------------------------------- the amendment: death is progress

/// The siege's edge against `boss` (percent; 0 none).
pub fn siege_edge(l: &LineageState, boss: &str) -> u32 {
    if l.feats.off {
        return 0;
    }
    l.feats.siege.get(boss).map_or(0, |s| (s.tries * SIEGE_PCT).min(SIEGE_CAP))
}

/// The run's siege edges at its send (boss → percent).
pub fn siege_edges(l: &LineageState) -> Vec<(String, u32)> {
    if l.feats.off {
        return Vec::new();
    }
    BOSS_DEPTHS
        .iter()
        .map(|(k, _)| (k.to_string(), siege_edge(l, k) + if l.feats.titles.contains(*k) { TITLE_PCT } else { 0 }))
        .filter(|(_, p)| *p > 0)
        .collect()
}

/// A title's name (`Queensbane`).
pub fn title_of(boss: &str) -> String {
    format!("{}bane", crate::sifter::boss_short(boss))
}

/// Graves on floors this run reached (left by an earlier run) come home: the purse takes their gold (`recovered`).
fn recover_graves(g: &mut Game, run: &Run, day: u32) {
    let (take, keep): (Vec<Grave>, Vec<Grave>) = std::mem::take(&mut g.lineage.feats.graves).into_iter().partition(|x| x.run < run.id && x.depth <= run.max_depth);
    g.lineage.feats.graves = keep;
    for x in take {
        if x.gold > 0 {
            g.lineage.gold_move(x.gold, &format!("recovered {}'s pack", x.name));
            news(&mut g.lineage, "recovered", format!("{}'s pack · ${} · D{}", x.name, x.gold, x.depth), day);
        }
    }
}

/// A death: the grave of his lost carry, his stone, and the siege's try when he fell at a band boss.
fn fallen(g: &mut Game, run: &Run, day: u32, legacy_earned: u32) {
    let name = crate::legacy::hero_name(g.lineage.seed, run.heir).to_string();
    let lost = g.last_exit.as_ref().filter(|x| x.run_id == run.id).map_or(0, |x| (x.carried - x.kept).max(0));
    let mut grave_gold = 0;
    if lost > 0 {
        // one grave a floor: the newest keeps the older's carry with its own
        let older: i32 = g.lineage.feats.graves.iter().filter(|x| x.depth == run.depth).map(|x| x.gold).sum();
        g.lineage.feats.graves.retain(|x| x.depth != run.depth);
        grave_gold = lost + older;
        g.lineage.feats.graves.push(Grave { depth: run.depth, heir: run.heir, name: name.clone(), gold: grave_gold, run: run.id, day });
    }
    let boss = run.route.boss(run.depth).filter(|b| !run.boss_kills.iter().any(|(_, k)| k == b)).map(str::to_string);
    let (mut try_n, mut edge) = (0, 0);
    if let Some(b) = &boss {
        let pct = run.monsters.iter().find(|m| m.kind == *b).map_or(100, |m| (m.hp.max(0) * 100 / m.max_hp.max(1)) as u32);
        let s = g.lineage.feats.siege.entry(b.clone()).or_insert(Siege { tries: 0, best_pct: 100, heirs: Vec::new() });
        s.tries += 1;
        s.best_pct = s.best_pct.min(pct);
        s.heirs.push(name.clone());
        try_n = s.tries;
        edge = siege_edge(&g.lineage, b);
        // (owner amendment 2: a try at the wall is a deed — `DEED_LEGACY` more for the heir who made it)
        crate::legacy::ensure(&mut g.lineage);
        g.lineage.bloodline.as_mut().expect("bloodline").points += DEED_LEGACY;
        if let Some(h) = g.lineage.hero_legacy.iter_mut().rev().find(|h| h.heir == run.heir) {
            h.points += DEED_LEGACY;
        }
        g.batch.legacy_earned += DEED_LEGACY;
        g.lineage.feats.deed_legacy += DEED_LEGACY;
        let short = crate::sifter::boss_short(b);
        news(&mut g.lineage, "siege", format!("{short} · try {try_n} · +{edge}%"), day);
    }
    let cause = run.death_cause.clone().unwrap_or_else(|| "unknown".into());
    let foe = boss.as_deref().map(crate::sifter::boss_short).map(|s| format!("the {s}")).unwrap_or_else(|| cause.replace('_', " "));
    let epitaph = format!("fell to {foe}, D{}", run.depth);
    let l = &mut g.lineage;
    let legacy = legacy_earned + if try_n > 0 { DEED_LEGACY } else { 0 };
    l.feats.stones.push(Stone { heir: run.heir, name, depth: run.depth, cause, epitaph, day, run: run.id, boss, try_n, edge_pct: edge, grave_gold, legacy });
    let n = l.feats.stones.len();
    if n > STONES_CAP {
        l.feats.stones.drain(..n - STONES_CAP);
    }
}

/// The death screen's memorial (`Death.memorial`): the lead line of progress, the epitaph, the grave.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Memorial {
    /// `the Queen · try 4 · +16%` at a band boss, else the epitaph.
    pub lead: String,
    pub epitaph: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub siege: Option<SiegeWire>,
    #[serde(default, skip_serializing_if = "is_zero_i32")]
    pub grave_gold: i32,
    /// The Legacy the death's deeds paid.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub legacy: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SiegeWire {
    pub boss: String,
    pub title: String,
    pub depth: u32,
    pub tries: u32,
    /// The lowest hp he was left on (percent).
    pub best_pct: u32,
    /// The next heir's edge against him (percent of his blows).
    pub edge_pct: u32,
    pub heirs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct GraveWire {
    pub depth: u32,
    pub heir: u32,
    pub name: String,
    pub gold: i32,
    pub day: u32,
}

/// The memorial of the death of run `id`, when its stone stands.
pub fn memorial(l: &LineageState, id: u32) -> Option<Memorial> {
    let s = l.feats.stones.iter().rev().find(|s| s.run == id)?;
    let siege = s.boss.as_ref().map(|b| {
        let now = l.feats.siege.get(b);
        SiegeWire {
            boss: b.clone(),
            title: crate::sifter::boss_short(b).into(),
            depth: s.depth,
            tries: now.map_or(s.try_n, |x| x.tries),
            best_pct: now.map_or(100, |x| x.best_pct),
            edge_pct: siege_edge(l, b),
            heirs: now.map_or_else(Vec::new, |x| x.heirs.clone()),
        }
    });
    let lead = match &s.boss {
        Some(b) => format!("+{} Legacy · {} try {} · +{}%", s.legacy, crate::sifter::boss_short(b), s.try_n, s.edge_pct),
        None if s.legacy > 0 => format!("+{} Legacy · {}", s.legacy, s.epitaph),
        None => s.epitaph.clone(),
    };
    Some(Memorial { lead, epitaph: s.epitaph.clone(), name: s.name.clone(), siege, grave_gold: s.grave_gold, legacy: s.legacy })
}

fn siege_wire(l: &LineageState) -> Vec<SiegeWire> {
    let route = l.rules().route();
    l.feats
        .siege
        .iter()
        .map(|(b, s)| SiegeWire { boss: b.clone(), title: crate::sifter::boss_short(b).into(), depth: boss_at(b, route).unwrap_or(0), tries: s.tries, best_pct: s.best_pct, edge_pct: siege_edge(l, b), heirs: s.heirs.clone() })
        .collect()
}

// ---------------------------------------------------------------- owner amendment 2: the heir order

/// The card the heir order picks among the wake's `cards` (`answer`: the card whose gift answers the last killer,
/// else card 1; `strongest`: the highest tier, card order breaking ties; `surprise`: a draw from the seed and heir).
pub fn heir_choice(l: &LineageState, cards: &[crate::traits::Card], answer: Option<crate::traits::Gift>) -> usize {
    match l.orders.heir.as_str() {
        "strongest" => cards.iter().enumerate().max_by_key(|(i, c)| (c.shape.tier, std::cmp::Reverse(*i))).map_or(0, |(i, _)| i),
        "surprise" => (mix(l.seed, u64::from(l.heir), 0x5E1F) % cards.len().max(1) as u64) as usize,
        _ => answer.and_then(|g| cards.iter().position(|c| c.shape.gift == g)).unwrap_or(0),
    }
}

/// The report's line for the heir the order chose (`Bram · Guard on bosses · answers the Queen`).
pub fn heir_line(l: &LineageState) -> Option<String> {
    let born = l.heirs.born?;
    let killer = l.graveyard.last()?.cause.clone();
    let name = crate::legacy::hero_name(l.seed, l.heir);
    let answers = crate::traits::answer_gift(&killer).is_some_and(|(g, _)| g == born.gift);
    let foe = crate::defs::MONSTERS.iter().find(|m| m.kind == killer).map_or(killer.replace('_', " "), |m| if m.boss { crate::sifter::boss_short(m.kind).to_string() } else { m.title.to_string() });
    Some(if answers { format!("{name} · {} · answers the {foe}", born.chip()) } else { format!("{name} · {}", born.chip()) })
}

