//! Cut 30 §1–2 (docs/CUT30.md, docs/IDLE_FIRST.md §4 D): packages — pre-written rule bundles that
//! level from runs — and the idle floor they make (the `Steady` school stance, drills, scars).
//!
//! A lineage equips a stance (never empty), a tactic (a second from the Lich met) and a temperament
//! (from heir 3). The equipped packages **compile to a plain `RuleSet`** in a fixed order — the pen's
//! rows (once open) › drills › the stance's guard rows › tactics › the temperament › the stance's
//! fallback — each row tagged with its origin (`stance:steady`, `drill:goblin_warlord`,
//! `tactic:boss_focus`, `temper:skittish`; the pen's rows keep the editor's `player` / `patch`), so the
//! sim, the trace, the replay hash and the forecast read what they always read. Package rows sit
//! outside the player's row cap (`Row::is_pkg`).
//!
//! Levels are the idle engine: a package gains a run for every run one of its rows fired (offline
//! included); L2 · L3 · L4 · L5 at `LEVEL_RUNS`. Each level adds or tunes a row, written here.
//!
//! Drills: a band boss's counter the lineage knows enters the set as a named row at the boss's
//! second meeting (`drill:<boss>`), announced once (`DRILLED`), revocable. Scars: every meeting
//! leaves the boss −`SCAR_PCT` % max hp for the lineage (cap `SCAR_CAP`), cleared when he is slain.
//!
//! A `literal` lineage (the harnesses' bots: `Game::new_literal`) compiles nothing: its set is what
//! `set_rules` wrote, as before Cut 30. An old save migrates as one `custom` stance holding its set,
//! the pen open (`migrate`): its rules keep working, drills added above them.
use crate::engine::LineageState;
use crate::rules::{Cond, Row, RuleSet, Verb};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A package's kind — its slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Stance,
    Tactic,
    Temperament,
}

impl Kind {
    pub fn word(self) -> &'static str {
        match self {
            Kind::Stance => "stance",
            Kind::Tactic => "tactic",
            Kind::Temperament => "temperament",
        }
    }
    /// The origin prefix of this kind's rows.
    pub fn origin(self) -> &'static str {
        match self {
            Kind::Stance => "stance",
            Kind::Tactic => "tactic",
            Kind::Temperament => "temper",
        }
    }
}

/// A package: its id, kind, name (≤ 2 words), the card it plays (a tactic), the stage that brings
/// it (≤ 3 words), and a temperament's gift and cost (the trait engine's mapped shape).
pub struct PackageDef {
    pub id: &'static str,
    pub kind: Kind,
    pub name: &'static str,
    pub card: Option<&'static str>,
    pub trigger: &'static str,
    /// Temperaments: the old temperament it maps (`traits::upgrade`).
    pub temperament: Option<crate::hero::Trait>,
}

const fn stance(id: &'static str, name: &'static str, trigger: &'static str) -> PackageDef {
    PackageDef { id, kind: Kind::Stance, name, card: None, trigger, temperament: None }
}
const fn tactic(id: &'static str, name: &'static str) -> PackageDef {
    PackageDef { id, kind: Kind::Tactic, name, card: Some(id), trigger: "slay Warlord", temperament: None }
}
const fn temper(id: &'static str, name: &'static str, t: crate::hero::Trait) -> PackageDef {
    PackageDef { id, kind: Kind::Temperament, name, card: None, trigger: "heir 3", temperament: Some(t) }
}

/// The set v1 (docs/CUT30.md §2): 4 stances, 6 tactics (today's cards), 4 temperaments.
pub const PACKAGES: &[PackageDef] = &[
    stance("steady", "Steady", ""),
    stance("guarded", "Guarded", "meet Warlord"),
    stance("bold", "Bold", "a day on"),
    stance("hunter", "Hunter", "a day on"),
    tactic("boss_focus", "boss focus"),
    tactic("corridor_fighting", "corridor fighting"),
    tactic("kite_archers", "kite archers"),
    tactic("thief_guard", "thief guard"),
    tactic("gas_step", "gas step"),
    tactic("pack_break", "pack break"),
    temper("skittish", "skittish", crate::hero::Trait::Cowardly),
    temper("unbowed", "unbowed", crate::hero::Trait::Brave),
    temper("light_hands", "light hands", crate::hero::Trait::Greedy),
    temper("iron_gut", "iron gut", crate::hero::Trait::Curious),
];

/// The custom stance: an old save's set (or a pen-written whole set), as written.
pub const CUSTOM: &str = "custom";

pub fn def(id: &str) -> Option<&'static PackageDef> {
    PACKAGES.iter().find(|p| p.id == id)
}

/// The display name (`Steady`, `boss focus`, `custom`).
pub fn name(id: &str) -> &str {
    def(id).map(|d| d.name).unwrap_or(if id == CUSTOM { "custom" } else { id })
}

/// Runs a package needs for L2 · L3 · L4 · L5 (tunable; the gate is the felt pace).
pub const LEVEL_RUNS: [u32; 4] = [10, 40, 120, 220];
pub const MAX_LEVEL: u32 = 5;

pub fn level_of(runs: u32) -> u32 {
    1 + LEVEL_RUNS.iter().filter(|&&n| runs >= n).count() as u32
}

/// Runs to the next level, if any.
pub fn next_at(runs: u32) -> Option<u32> {
    LEVEL_RUNS.iter().copied().find(|&n| runs < n)
}

/// Each meeting's scar on a band boss, in percent of his max hp, and the cap.
pub const SCAR_PCT: u32 = 5;
pub const SCAR_CAP: u32 = 30;
/// A drill enters at this meeting (the second).
pub const DRILL_MEETING: u32 = 2;
/// Every band boss past the Warlord is drilled at this many days met (the idle path is patient; a
/// package that answers him is the quicker one).
pub const DRILL_DAYS: u32 = 3;
/// The deep walls (D28 on: the Queen, the King) are drilled only at this many days met — the pen's counter
/// row is the quicker way past them (the owner, 2026-10-01).
pub const DEEP_DRILL_DAYS: u32 = 6;
pub const DEEP_FROM: u32 = 28;
/// The Foundry's first floor: its golems are the wall before its master (`on_run_end`).
pub const FOUNDRY_WALL: u32 = 19;


/// A drilled counter: the boss, the rows, revoked (one tap) or not, announced (`DRILLED`) or not.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Drill {
    pub boss: String,
    pub rows: Vec<Row>,
    #[serde(default)]
    pub revoked: bool,
    #[serde(default)]
    pub announced: bool,
}

/// The lineage's packages (`LineageState::pkg`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PkgState {
    /// A harness's lineage: nothing compiles, the set is `set_rules`'s (pre-Cut 30 behaviour).
    #[serde(default)]
    pub literal: bool,
    pub stance: String,
    #[serde(default)]
    pub tactics: Vec<String>,
    #[serde(default)]
    pub temperament: Option<String>,
    /// Package id → runs one of its rows fired in (the level's count).
    #[serde(default)]
    pub runs: BTreeMap<String, u32>,
    /// Packages that have arrived (stances, tactics, temperaments offered and taken).
    #[serde(default)]
    pub owned: BTreeSet<String>,
    #[serde(default)]
    pub drills: Vec<Drill>,
    /// Band boss → meetings (runs that saw him), for the drill and the scars.
    #[serde(default)]
    pub meets: BTreeMap<String, u32>,
    /// Band boss → the day of the clock his last meeting was counted on (one a day).
    #[serde(default)]
    pub met_day: BTreeMap<String, u32>,
    /// Band boss → runs that met him (the drill comes at the second).
    #[serde(default)]
    pub met_runs: BTreeMap<String, u32>,
    /// Package id (and `tactics`, the first tactic's) → the day of the clock it arrived.
    #[serde(default)]
    pub arrived: BTreeMap<String, u32>,
    /// The pen: open or not, and the rows the player wrote (above every package).
    #[serde(default)]
    pub pen_open: bool,
    #[serde(default)]
    pub pen: Vec<Row>,
    /// The custom stance's rows (an old save's set).
    #[serde(default)]
    pub custom: Vec<Row>,
    /// The wake's temperament cards (heir 3 on); a send without a pick takes card 1.
    #[serde(default)]
    pub offer: Vec<String>,
    /// Levels reached and drills made since the report last looked (the report's lines).
    #[serde(default)]
    pub news: Vec<String>,
}

impl Default for PkgState {
    fn default() -> Self {
        PkgState {
            literal: false,
            stance: "steady".into(),
            tactics: Vec::new(),
            temperament: None,
            runs: BTreeMap::new(),
            owned: ["steady".to_string()].into_iter().collect(),
            drills: Vec::new(),
            meets: BTreeMap::new(),
            met_day: BTreeMap::new(),
            met_runs: BTreeMap::new(),
            arrived: BTreeMap::new(),
            pen_open: false,
            pen: Vec::new(),
            custom: Vec::new(),
            offer: Vec::new(),
            news: Vec::new(),
        }
    }
}

impl PkgState {
    pub fn level(&self, id: &str) -> u32 {
        level_of(self.runs.get(id).copied().unwrap_or(0))
    }
    /// The equipped packages, stance first.
    pub fn equipped(&self) -> Vec<String> {
        let mut v = vec![self.stance.clone()];
        v.extend(self.tactics.iter().cloned());
        v.extend(self.temperament.iter().cloned());
        v
    }
    /// The scar on `boss` now, in percent (0 once he is slain).
    pub fn scar(&self, boss: &str, kills: &BTreeSet<String>) -> u32 {
        if kills.contains(boss) {
            return 0;
        }
        (self.meets.get(boss).copied().unwrap_or(0) * SCAR_PCT).min(SCAR_CAP)
    }
}

// ---------------------------------------------------------------- rows

fn r(conds: Vec<Cond>, verb: Verb) -> Row {
    Row::new(conds, verb)
}
fn n(k: &str, v: i32) -> Cond {
    Cond::n(k, v)
}
fn tag(t: &str) -> Cond {
    Cond::t("foe_tag", t)
}

/// The depth a stance banks at: the first floor past the record (`at record`), `extra` more.
fn bank_at(best: u32, extra: u32) -> i32 {
    (best + 1 + extra).max(2) as i32
}

/// Steady rests only when well hurt (`Guarded` rests at 80 %): L2 under this, L3 under the next.
pub const STEADY_REST: i32 = 40;
pub const STEADY_REST_L3: i32 = 50;
/// Guarded's way home: earlier than Steady's, never so early it cannot reach the record.
pub const GUARDED_RETURN: i32 = 25;

/// The heal threshold of a stance at a level (a drill above the guard rows yields to it).
pub fn heal_pct(id: &str, level: u32) -> i32 {
    match id {
        "steady" => {
            if level >= 2 {
                35
            } else {
                30
            }
        }
        "hunter" => {
            if level >= 2 {
                35
            } else {
                30
            }
        }
        "guarded" => {
            if level >= 2 {
                45
            } else {
                40
            }
        }
        "bold" => 25,
        _ => 30,
    }
}

/// A stance's rows at a level: (guard rows, fallback rows). `best` is the lineage's record.
pub fn stance_rows(id: &str, level: u32, best: u32) -> (Vec<Row>, Vec<Row>) {
    let heal = heal_pct(id, level);
    let drink = r(vec![n("hp<", heal)], Verb::arg("drink", "heal"));
    let attack = r(vec![n("foes>=", 1)], Verb::arg("attack", "nearest"));
    match id {
        // heal 30 %, return 20 %, bank at the record, attack nearest. L2 heals at 35 % and rests when
        // well hurt (under 40 %), L3 rests under 50 %, L4 banks a floor further when whole (at the
        // record when hurt), L5 steps off a telegraph when hurt. The long rest is `Guarded`'s.
        "steady" => {
            // (the safest default: from L3 it walks home at a quarter of its hp)
            let mut g = vec![drink, r(vec![n("hp<", if level >= 3 { 25 } else { 20 })], Verb::new("return"))];
            if level >= 4 {
                g.push(r(vec![n("hp<", 60), n("depth>=", bank_at(best, 0))], Verb::new("bank")));
                g.push(r(vec![n("depth>=", bank_at(best, 1))], Verb::new("bank")));
            } else {
                g.push(r(vec![n("depth>=", bank_at(best, 0))], Verb::new("bank")));
            }
            if level >= 5 {
                g.push(r(vec![tag("telegraph"), n("hp<", 35)], Verb::new("retreat")));
            }
            let mut f = vec![attack];
            if level >= 2 {
                f.push(r(vec![n("hp<", if level >= 3 { STEADY_REST_L3 } else { STEADY_REST })], Verb::new("rest")));
            }
            (g, f)
        }
        // heal 40 %, return 25 %, bank at the record, rest between fights (under 80 %). L2 heals at
        // 45 %, L3 steps off a telegraphed blow, L4 backs into a corridor against a crowd, L5 banks a
        // floor further when whole.
        "guarded" => {
            let mut g = vec![drink, r(vec![n("hp<", GUARDED_RETURN)], Verb::new("return"))];
            if level >= 5 {
                g.push(r(vec![n("hp<", 60), n("depth>=", bank_at(best, 0))], Verb::new("bank")));
                g.push(r(vec![n("depth>=", bank_at(best, 1))], Verb::new("bank")));
            } else {
                g.push(r(vec![n("depth>=", bank_at(best, 0))], Verb::new("bank")));
            }
            if level >= 3 {
                // (a telegraphed blow is met with a drink, not a step back: `telegraph → retreat` looped
                // retreat ↔ explore on ~3 % of sends — the stall row)
                g.push(r(vec![tag("telegraph"), n("hp<", 60)], Verb::arg("drink", "heal")));
            }
            if level >= 4 {
                g.push(r(vec![n("foes>=", 3), n("hp<", 60)], Verb::new("back_corridor")));
            }
            (g, vec![attack, r(vec![n("hp<", 80)], Verb::new("rest"))])
        }
        // heal 25 %, no return, bank two floors past the record, the boss first and to the end (no heal
        // stops the blow), attack the weakest. L2 rests under 50 %, L5 banks one floor further.
        "bold" => {
            let extra = if level >= 5 { 2 } else { 1 };
            // (the bold go down, not home: hurt with the stairs in reach, he dives past the floor)
            let g = vec![
                r(vec![tag("boss"), Cond::flag("on_hurt")], Verb::arg("attack", "tag:boss")),
                drink,
                r(vec![n("depth>=", bank_at(best, extra))], Verb::new("bank")),
                r(vec![tag("boss")], Verb::arg("attack", "tag:boss")),
                r(vec![n("hp<", 40), Cond::flag("path_stairs")], Verb::new("descend")),
            ];
            let mut f = vec![r(vec![n("foes>=", 1)], Verb::arg("attack", "lowest"))];
            if level >= 2 {
                f.push(r(vec![n("hp<", 50)], Verb::new("rest")));
            }
            (g, f)
        }
        // heal 30 %, return 20 %, bank at the record, the summoned and the boss first. L2 heals at
        // 35 % and rests under 50 %, L4 the casters first, L5 banks a floor further when whole.
        "hunter" => {
            let mut g = vec![drink, r(vec![n("hp<", 20)], Verb::new("return"))];
            if level >= 5 {
                g.push(r(vec![n("hp<", 60), n("depth>=", bank_at(best, 0))], Verb::new("bank")));
                g.push(r(vec![n("depth>=", bank_at(best, 1))], Verb::new("bank")));
            } else {
                g.push(r(vec![n("depth>=", bank_at(best, 0))], Verb::new("bank")));
            }
            // the hunter goes for what strikes from afar or raises others: archers, casters, the
            // summoned, the boss — before the nearest
            // (and never meleés a mirror of blows: from afar, fire, or a step away — the Foundry's card)
            g.push(r(vec![tag("reflect_melee")], Verb::arg("tactic", "reflect_read")));
            g.push(r(vec![tag("ranged"), n("hp>", heal)], Verb::arg("attack", "tag:ranged")));
            g.push(r(vec![tag("summoned"), n("hp>", heal)], Verb::arg("attack", "tag:summoned")));
            g.push(r(vec![tag("boss"), n("hp>", heal)], Verb::arg("attack", "tag:boss")));
            if level >= 3 {
                g.push(r(vec![tag("caster"), n("hp>", heal)], Verb::arg("attack", "tag:caster")));
            }
            let mut f = vec![attack];
            if level >= 2 {
                f.push(r(vec![n("hp<", 50)], Verb::new("rest")));
            }
            (g, f)
        }
        _ => (Vec::new(), vec![attack]),
    }
}

/// A tactic's rows at a level: its card (the card's rows play at the row), and from L3 a row the
/// card's situation wants.
pub fn tactic_rows(id: &str, level: u32) -> Vec<Row> {
    let mut v = vec![r(vec![], Verb::arg("tactic", id))];
    if level >= 3 {
        let extra = match id {
            "boss_focus" => Some(r(vec![tag("summoned")], Verb::arg("attack", "tag:summoned"))),
            "kite_archers" => Some(r(vec![tag("ranged"), n("adj>=", 1)], Verb::arg("attack", "tag:ranged"))),
            "thief_guard" => Some(r(vec![tag("thief")], Verb::arg("attack", "tag:thief"))),
            "gas_step" => Some(r(vec![tag("gas"), n("adj>=", 1)], Verb::new("retreat"))),
            "pack_break" => Some(r(vec![tag("pack"), n("adj>=", 2)], Verb::new("back_corridor"))),
            "corridor_fighting" => Some(r(vec![n("foes>=", 3)], Verb::new("back_corridor"))),
            _ => None,
        };
        v.extend(extra);
    }
    v
}

/// A temperament's rows: what the name promises (1–2 rows).
pub fn temperament_rows(id: &str, level: u32) -> Vec<Row> {
    match id {
        "skittish" => {
            let mut v = vec![r(vec![n("hp<", 35), n("adj>=", 2)], Verb::new("retreat"))];
            if level >= 3 {
                v.push(r(vec![n("hp<", 25), n("foes>=", 1)], Verb::new("back_corridor")));
            }
            v
        }
        "unbowed" => vec![r(vec![tag("boss"), n("hp>", 40)], Verb::arg("attack", "tag:boss"))],
        // (the kill's drop, grabbed at once — `loot ≥ 0 → pick up` looped pick up ↔ explore)
        "light_hands" => vec![r(vec![Cond::flag("on_kill"), n("hp>", 50)], Verb::new("pick_up"))],
        "iron_gut" => vec![r(vec![Cond::flag("unknown_item"), n("hp>", 60)], Verb::arg("drink", "unknown"))],
        _ => Vec::new(),
    }
}

/// The counter row a drill writes for `boss` (the fact's row; the Lich's two), yielding to the
/// stance's heal (`hp > heal`): a drill above the guard rows must not stop the hero drinking.
pub fn drill_rows(boss: &str, heal: i32) -> Vec<Row> {
    let guard = |mut row: Row| {
        if row.conds.len() < 2 {
            row.conds.push(n("hp>", heal));
        }
        row
    };
    let mut rows = vec![guard(crate::facts::counter_row(boss))];
    if boss == "lich" {
        rows.push(guard(r(vec![tag("boss")], Verb::arg("attack", "tag:boss"))));
    }
    if boss == "foundry_master" {
        rows.push(guard(r(vec![tag("buffer"), n("depth>=", 19)], Verb::arg("attack", "tag:buffer"))));
    }
    rows
}

/// The item a drill's rows need packed (`fire` at the Mother, `silence` at the Queen).
pub fn drill_item(d: &Drill) -> Option<String> {
    d.rows.iter().find_map(|row| match row.verb.v.as_str() {
        "throw" | "read" | "drink" => row.verb.a.as_deref().and_then(|a| a.split(',').next()).filter(|k| !k.is_empty() && *k != "unknown").map(String::from),
        _ => None,
    })
}

fn tagged(rows: Vec<Row>, origin: &str) -> Vec<Row> {
    rows.into_iter().map(|row| row.from(origin)).collect()
}

/// The compiled set of a lineage (see the module docs for the order).
pub fn compile(l: &LineageState) -> RuleSet {
    let p = &l.pkg;
    let best = l.best_depth;
    let held = l.rules();
    let mut rows: Vec<Row> = Vec::new();
    if p.pen_open {
        rows.extend(p.pen.iter().cloned());
    }
    let level = p.level(&p.stance);
    let heal = heal_pct(&p.stance, level);
    for d in p.drills.iter().filter(|d| !d.revoked) {
        let origin = format!("drill:{}", d.boss);
        rows.extend(d.rows.iter().cloned().map(|row| row.from(&origin)));
    }
    let origin = format!("stance:{}", p.stance);
    let (guard, fallback) = if p.stance == CUSTOM { (p.custom.clone(), Vec::new()) } else { stance_rows(&p.stance, level, best) };
    rows.extend(tagged(guard, &origin));
    for t in &p.tactics {
        rows.extend(tagged(tactic_rows(t, p.level(t)), &format!("tactic:{t}")));
    }
    if let Some(t) = &p.temperament {
        rows.extend(tagged(temperament_rows(t, p.level(t)), &format!("temper:{t}")));
    }
    rows.extend(tagged(fallback, &origin));
    let _ = heal;
    // one row per card (the pen's own card row wins), and never the same row twice
    let mut out: Vec<Row> = Vec::new();
    let mut cards: Vec<String> = Vec::new();
    for row in rows {
        if let Some(c) = row.card() {
            if cards.iter().any(|x| x == c) {
                continue;
            }
            cards.push(c.to_string());
        }
        if out.contains(&row) {
            continue;
        }
        out.push(row);
    }
    out.truncate(crate::engine::ROWS_TOTAL);
    RuleSet { rows: out, name: held.name.clone(), route: held.route.clone() }
}

/// Write the compiled set into the active slot (nothing for a literal lineage).
pub fn recompile(l: &mut LineageState) -> bool {
    if l.pkg.literal {
        return false;
    }
    let set = compile(l);
    // a package's card plays with the package (the lineage owns it from then on)
    for c in set.rows.iter().filter(|r| r.is_pkg()).filter_map(|r| r.card()) {
        if !l.unlocks.contains(c) {
            l.unlocks.insert(c.to_string());
        }
    }
    let i = l.active_set.min(l.sets.len() - 1);
    if l.sets[i] == set && l.sets[i].rows.iter().zip(&set.rows).all(|(a, b)| a.origin == b.origin) {
        return false;
    }
    l.sets[i] = set;
    true
}

/// The pen's rows (or the custom stance's) from a set the editor wrote: every row that is not a
/// package's as compiled now — a package row the player edited is the player's.
pub fn absorb(l: &mut LineageState, set: &RuleSet) {
    let compiled = compile(l);
    let is_ours = |row: &Row| row.is_pkg() && compiled.rows.iter().any(|c| c == row && c.origin == row.origin);
    if l.pkg.stance == CUSTOM {
        // the custom stance is the pen's own set: every row but the drills
        l.pkg.custom = set.rows.iter().filter(|row| !row.origin.as_deref().is_some_and(|o| o.starts_with("drill:"))).cloned().map(|mut row| {
            if row.origin.as_deref().is_some_and(|o| o.starts_with("stance:")) {
                row.origin = None;
            }
            row
        }).collect();
        l.pkg.pen.clear();
    } else {
        l.pkg.pen = set.rows.iter().filter(|row| !is_ours(row)).cloned().map(|mut row| {
            if row.is_pkg() {
                row.origin = Some("player".into());
            }
            row
        }).collect();
    }
    let i = l.active_set.min(l.sets.len() - 1);
    l.sets[i].route = set.route.clone();
    l.sets[i].name = set.name.clone();
}

/// A fresh lineage's packages: `Steady`, compiled.
pub fn init(l: &mut LineageState) {
    l.pkg = PkgState::default();
    recompile(l);
}

/// An old save (no packages yet): its set becomes the `custom` stance, the pen open.
pub fn migrate(l: &mut LineageState) {
    let set = l.rules().clone();
    let mut p = PkgState { stance: CUSTOM.into(), pen_open: true, ..PkgState::default() };
    p.custom = set.rows.clone();
    p.owned.insert(CUSTOM.into());
    l.pkg = p;
    arrive(l);
    recompile(l);
}

/// The harness's lineage: nothing compiles (bots and tests that write their own sets).
pub fn make_literal(l: &mut LineageState) {
    l.pkg.literal = true;
    let i = l.active_set.min(l.sets.len() - 1);
    l.sets[i] = crate::probes::preset(l.class);
}

// ---------------------------------------------------------------- arrivals, equip, levels

/// Packages whose stage has come (stances by bosses met or slain, tactics with the Warlord slain,
/// temperaments offered from heir 3); the ids that arrived now.
pub fn arrive(l: &mut LineageState) -> Vec<String> {
    // Cut 30 (PROGRESSION_V2 §4): a drip, not a dump — `Guarded` with the Warlord met, `Bold` a day
    // after, `Hunter` a day after that; the tactics one per band boss slain or per day since the Warlord
    // fell (a card the lineage already owned arrives with the first). A day is the lineage's clock.
    let met = |k: &str| l.pkg.meets.get(k).copied().unwrap_or(0) > 0 || l.facts.contains(&format!("foe:{k}")) || l.kills.contains(k);
    let day = l.day;
    let since = |id: &str| l.pkg.arrived.get(id).map(|d| day > *d).unwrap_or(false);
    let slain = crate::descent::BOSS_DEPTHS.iter().filter(|(k, _)| l.kills.contains(*k)).count() as u32;
    let tactics_owned = l.pkg.owned.iter().filter(|id| def(id).is_some_and(|d| d.kind == Kind::Tactic)).count() as u32;
    let tactic_day = l.pkg.arrived.get("tactics").copied();
    let tactics_due = match tactic_day {
        Some(d0) => slain.max(1) + day.saturating_sub(d0),
        None if l.kills.contains("goblin_warlord") => 1,
        None => 0,
    };
    let mut new = Vec::new();
    let mut tactics_new = 0;
    for d in PACKAGES {
        if l.pkg.owned.contains(d.id) {
            continue;
        }
        let ok = match (d.kind, d.id) {
            (Kind::Stance, "steady") => true,
            (Kind::Stance, "guarded") => met("goblin_warlord"),
            (Kind::Stance, "bold") => since("guarded"),
            (Kind::Stance, "hunter") => since("bold"),
            (Kind::Tactic, _) => {
                let owned_card = d.card.is_some_and(|c| l.unlocks.contains(c));
                let due = tactics_owned + tactics_new < tactics_due;
                (owned_card && l.kills.contains("goblin_warlord")) || due
            }
            _ => false,
        };
        if ok {
            if d.kind == Kind::Tactic {
                tactics_new += 1;
            }
            new.push(d.id.to_string());
        }
    }
    if tactics_new > 0 && tactic_day.is_none() {
        l.pkg.arrived.insert("tactics".into(), day);
    }
    for id in &new {
        l.pkg.owned.insert(id.clone());
        l.pkg.arrived.insert(id.clone(), day);
    }
    new
}

/// Tactic slots: one from the Warlord slain, two from the Lich met.
pub fn tactic_slots(l: &LineageState) -> usize {
    let lich = l.pkg.meets.get("lich").copied().unwrap_or(0) > 0 || l.best_depth >= 18;
    if !l.kills.contains("goblin_warlord") && !l.pkg.owned.iter().any(|id| def(id).is_some_and(|d| d.kind == Kind::Tactic)) {
        0
    } else if lich {
        2
    } else {
        1
    }
}

/// Whether the temperament slot is open (heir 3).
pub fn temperament_open(l: &LineageState) -> bool {
    l.heir >= 3
}

/// Equip a package in its slot (a tactic in `slot` 0/1). Free and instant; refused when it has not
/// arrived or its slot is closed.
pub fn equip(l: &mut LineageState, id: &str, slot: usize) -> Result<(), String> {
    if l.pkg.literal {
        return Err("no packages".into());
    }
    let d = def(id).ok_or("unknown package")?;
    if !l.pkg.owned.contains(id) {
        return Err("not yet".into());
    }
    match d.kind {
        Kind::Stance => l.pkg.stance = id.into(),
        Kind::Tactic => {
            let slots = tactic_slots(l);
            if slot >= slots.max(1) || slots == 0 {
                return Err("slot closed".into());
            }
            l.pkg.tactics.retain(|t| t != id);
            if slot < l.pkg.tactics.len() {
                l.pkg.tactics[slot] = id.into();
            } else {
                l.pkg.tactics.push(id.into());
            }
            l.pkg.tactics.truncate(slots);
            if let Some(c) = d.card {
                l.unlocks.insert(c.into());
            }
        }
        Kind::Temperament => {
            if !temperament_open(l) {
                return Err("slot closed".into());
            }
            wear(l, Some(id));
        }
    }
    recompile(l);
    Ok(())
}

/// Empty a tactic or temperament slot (the stance is never empty).
pub fn unequip(l: &mut LineageState, id: &str) -> Result<(), String> {
    let d = def(id).ok_or("unknown package")?;
    match d.kind {
        Kind::Stance => return Err("stance never empty".into()),
        Kind::Tactic => l.pkg.tactics.retain(|t| t != id),
        Kind::Temperament => wear(l, None),
    }
    recompile(l);
    Ok(())
}

/// The temperament worn: its package and the heir's gift and cost (the trait engine's mapped shape).
pub fn wear(l: &mut LineageState, id: Option<&str>) {
    l.pkg.temperament = id.map(String::from);
    if let Some(t) = id.and_then(def).and_then(|d| d.temperament) {
        l.pkg.owned.insert(id.unwrap_or_default().to_string());
        if !l.heirs.neutral {
            l.heirs.born = Some(crate::traits::legacy(t));
        }
    } else if !l.heirs.neutral {
        l.heirs.born = None;
    }
}

/// The wake's temperament cards (heir 3 on): three of the four, seeded by the lineage and the heir,
/// the first the heir's until a pick (a send without one keeps card 1).
pub fn wake(l: &mut LineageState) {
    if l.pkg.literal || !temperament_open(l) {
        l.pkg.offer.clear();
        return;
    }
    let ids: Vec<&str> = PACKAGES.iter().filter(|d| d.kind == Kind::Temperament).map(|d| d.id).collect();
    let mut rng = crate::rng::Rng::derive(l.seed, crate::rng::hash_str("temperament_cards") ^ (l.heir + 1000 * l.ascension) as u64);
    let skip = rng.below(ids.len() as u32) as usize;
    let mut cards: Vec<String> = ids.iter().enumerate().filter(|(i, _)| *i != skip).map(|(_, s)| s.to_string()).collect();
    let k = rng.below(cards.len() as u32) as usize;
    cards.rotate_left(k);
    l.pkg.offer = cards;
    let first = l.pkg.offer.first().cloned();
    wear(l, first.as_deref());
    recompile(l);
}

/// Pick a wake card (the temperament slot).
pub fn pick(l: &mut LineageState, id: &str) -> Result<(), String> {
    if !l.pkg.offer.iter().any(|c| c == id) {
        return Err("not on offer".into());
    }
    wear(l, Some(id));
    recompile(l);
    Ok(())
}

/// Marks a level spend costs (the next level's number): ◆2 for L2 … ◆5 for L5.
pub fn level_price(l: &LineageState, id: &str) -> Option<u32> {
    let lv = l.pkg.level(id);
    (lv < MAX_LEVEL).then_some(lv + 1)
}

/// Spend marks on a package's next level (its runs set to the level's count).
pub fn spend_level(l: &mut LineageState, id: &str) -> Result<u32, String> {
    if !l.pkg.owned.contains(id) || def(id).is_none() {
        return Err("not yet".into());
    }
    let price = level_price(l, id).ok_or("top level")?;
    if l.marks < price {
        return Err("not enough marks".into());
    }
    l.marks -= price;
    let runs = l.pkg.runs.entry(id.to_string()).or_insert(0);
    *runs = next_at(*runs).unwrap_or(*runs);
    let lv = level_of(*runs);
    l.pkg.news.push(format!("{} L{lv}", name(id)));
    recompile(l);
    Ok(lv)
}

/// Revoke (or restore) a drill — one tap; it stays revoked.
pub fn revoke(l: &mut LineageState, boss: &str, revoked: bool) -> Result<(), String> {
    let d = l.pkg.drills.iter_mut().find(|d| d.boss == boss).ok_or("no drill")?;
    d.revoked = revoked;
    recompile(l);
    Ok(())
}

/// Open the pen (the Mother met, or a stall of `PEN_STALL_DAYS` days); true when it opened now.
/// (PROGRESSION_V2 §4: the Mother met and an age of 72 h, or 5 days whatever the climb — the system
/// curriculum's `pen`, one reveal a report; `systems::update_with` opens it.)
pub fn update_pen(l: &mut LineageState) -> bool {
    if l.pkg.pen_open || !l.systems.contains("pen") {
        return false;
    }
    l.pkg.pen_open = true;
    true
}

/// The Mother met (the pen's trigger).
pub fn mother_met(l: &LineageState) -> bool {
    l.pkg.meets.get("bloat_mother").copied().unwrap_or(0) > 0 || l.kills.contains("bloat_mother")
}

/// A run ended: its meetings (drills at the second, scars), its packages' runs (levels), the stages
/// that came, the pen; recompiled. Returns the lines for the report (`STEADY L3`, `DRILLED …`).
pub fn on_run_end(l: &mut LineageState, bosses_met: &[String], max_depth: u32, fired: &[u32], rules: &RuleSet) -> Vec<String> {
    if l.pkg.literal {
        return Vec::new();
    }
    let mut lines = Vec::new();
    // The Foundry's wall stands before its master: the iron golems at D19–22 turn melee back. A run
    // that met them with the tag known is a meeting of the wall (its drill is the master's counter,
    // `reflect read`) — no fact-counter of his is needed.
    let mut met: Vec<String> = bosses_met.to_vec();
    if max_depth >= FOUNDRY_WALL && crate::facts::tag_known(&l.facts, "iron_golem", "reflect_melee") && !met.iter().any(|b| b == "foundry_master") {
        met.push("foundry_master".into());
    }
    for b in &met {
        if crate::descent::boss_depth(b).is_none() {
            continue;
        }
        // a meeting is a day of the lineage's clock that saw him (the runs of one night are one
        // meeting): the drill comes the next day he is met, a scar each day
        // (the drill counts runs: the second run that meets him; the scars count days)
        let day = l.day;
        let runs = l.pkg.met_runs.entry(b.clone()).or_insert(0);
        *runs += 1;
        let met_runs = *runs;
        if l.pkg.met_day.get(b) != Some(&day) {
            l.pkg.met_day.insert(b.clone(), day);
            *l.pkg.meets.entry(b.clone()).or_insert(0) += 1;
        }
        // (the Warlord is drilled at the second run that meets him — the first wall teaches the drill;
        // every later boss at the second *day* he is met: a package that answers him — Hunter, boss
        // focus — passes him sooner, the drill is the idle path)
        let meets = if b == "goblin_warlord" { met_runs } else { l.pkg.meets.get(b).copied().unwrap_or(0) };
        let known = crate::facts::boss_counter_known(&l.facts, b) || (b == "foundry_master" && crate::facts::tag_known(&l.facts, "iron_golem", "reflect_melee"));
        // (the Foundry is a wall of golems, not one boss: its drill wants a third day)
        // (the deep walls drill late: the counter written in the pen breaks them days sooner — deep on
        // the lineage's own route, where the forks may have moved the boss up or down a band)
        let deep = l.rules().route().boss_depth(b).is_some_and(|d| d >= DEEP_FROM);
        let need = match b.as_str() {
            "goblin_warlord" => DRILL_MEETING,
            _ if deep => DEEP_DRILL_DAYS,
            "foundry_master" => DRILL_DAYS + 1,
            _ => DRILL_DAYS,
        };
        if meets >= need && known && !l.pkg.drills.iter().any(|d| d.boss == *b) {
            let heal = heal_pct(&l.pkg.stance, l.pkg.level(&l.pkg.stance));
            let rows = drill_rows(b, heal);
            for row in &rows {
                if let Some(c) = row.card() {
                    l.unlocks.insert(c.to_string());
                }
            }
            l.pkg.drills.push(Drill { boss: b.clone(), rows, revoked: false, announced: false });
            lines.push(format!("DRILLED · {}", crate::sifter::boss_short(b)));
        }
    }
    // levels: every package with a row that fired this run
    let mut ran: BTreeSet<String> = BTreeSet::new();
    for (i, row) in rules.rows.iter().enumerate() {
        if fired.get(i).copied().unwrap_or(0) == 0 {
            continue;
        }
        if let Some(o) = row.origin.as_deref() {
            if let Some((kind, id)) = o.split_once(':') {
                if matches!(kind, "stance" | "tactic" | "temper") {
                    ran.insert(id.to_string());
                }
            }
        }
    }
    for id in ran {
        let before = l.pkg.level(&id);
        *l.pkg.runs.entry(id.clone()).or_insert(0) += 1;
        let after = l.pkg.level(&id);
        if after > before {
            lines.push(format!("{} L{after}", name(&id).to_uppercase()));
        }
    }
    for id in arrive(l) {
        lines.push(format!("+{}", name(&id)));
    }
    if update_pen(l) {
        lines.push("the pen".into());
    }
    recompile(l);
    l.pkg.news.extend(lines.iter().cloned());
    while l.pkg.news.len() > 24 {
        l.pkg.news.remove(0);
    }
    lines
}

/// The drill items a send should carry now: the drills whose boss the send may meet (his floor
/// within reach of the record) and whose item the hero can name.
pub fn quartermaster(l: &LineageState) -> Vec<String> {
    let mut out = Vec::new();
    for d in l.pkg.drills.iter().filter(|d| !d.revoked) {
        let Some(depth) = crate::descent::boss_depth(&d.boss) else { continue };
        if l.best_depth + 2 < depth || l.kills.contains(&d.boss) && l.best_depth > depth + 1 {
            continue;
        }
        if let Some(k) = drill_item(d) {
            if crate::item::is_identified(&l.facts, &l.flavours, &k) && !out.contains(&k) {
                out.push(k);
            }
        }
    }
    out
}

/// Cut 30 (PROGRESSION_V2 §1): a report announces at most `BEATS` beats, the rest as `+N more` (a
/// package's levels climbed in one absence read as its last).
pub const BEATS: usize = 5;
pub fn beats(lines: &[String]) -> Vec<String> {
    beats_n(lines, BEATS)
}

/// `beats` with room for `cap` of them.
pub fn beats_n(lines: &[String], cap: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for l in lines {
        // (`STEADY L2` then `STEADY L3`: the last level stands)
        if let Some((head, _)) = l.rsplit_once(" L").filter(|(_, n)| n.parse::<u32>().is_ok()) {
            if let Some(i) = out.iter().position(|o| o.rsplit_once(" L").is_some_and(|(h, n)| h == head && n.parse::<u32>().is_ok())) {
                out.remove(i);
            }
        }
        out.push(l.clone());
    }
    if out.len() > cap.max(1) {
        let more = out.len() - (cap.max(1) - 1);
        out.truncate(cap.max(1) - 1);
        out.push(format!("+{more} more"));
    }
    out
}

/// The idle floor's pack keeps this many of the stance's kinds (the heal), bought at the send.
pub const PACK_FILL: usize = 2;

/// The kinds the packages' own rows drink or read (the stance's heal): the idle floor's pack is
/// kept full of them (`Game::restock_at`), the ones the hero can name and the shelf sells. A
/// literal lineage (a harness's) packs as it always did.
pub fn pack_kinds(l: &LineageState) -> Vec<String> {
    // (a custom stance is the player's own set: its repeat packs as it always did)
    if l.pkg.literal || l.pkg.stance == CUSTOM {
        return Vec::new();
    }
    let mut out: Vec<String> = Vec::new();
    // the stance's and the temperament's rows, and what a tactic's card throws (boss focus: fire at the
    // Mother) — the card's rows as it plays them
    let mut rows: Vec<Row> = Vec::new();
    for row in l.rules().rows.iter().filter(|r| r.origin.as_deref().is_some_and(|o| o.starts_with("stance:") || o.starts_with("temper:") || o.starts_with("tactic:"))) {
        match row.card() {
            Some(c) => rows.extend(crate::meta::unlock_rows(c).unwrap_or_default().into_iter().filter(|r| r.verb.v == "throw")),
            None => rows.push(row.clone()),
        }
    }
    for row in &rows {
        if !matches!(row.verb.v.as_str(), "drink" | "read" | "throw") {
            continue;
        }
        let Some(k) = row.verb.a.as_deref().and_then(|a| a.split(',').next()).filter(|k| !k.is_empty() && *k != "unknown") else { continue };
        if crate::item::is_identified(&l.facts, &l.flavours, k) && !out.iter().any(|x| x == k) {
            out.push(k.to_string());
        }
    }
    out
}

/// Cut 30 §2: a death's one cheapest lever before the pen opens (`None` once it is open, or on a
/// harness's lineage): a blacksmith step the purse pays; else, the killer a band boss not yet drilled,
/// `Hunter` when it has arrived and is not worn; else wait — the drill or the scars will come.
pub fn lever(l: &LineageState, cause: &str) -> Option<crate::wire::Lever> {
    if l.pkg.pen_open || l.pkg.literal {
        return None;
    }
    let step = crate::kit::ladders(l).into_iter().filter_map(|lad| lad.next.map(|n| (n.price, n.label))).min_by_key(|x| x.0);
    if let Some((price, label)) = step.filter(|(p, _)| *p as i32 <= l.gold) {
        let _ = price;
        return Some(crate::wire::Lever { kind: "spend".into(), text: label });
    }
    let boss = crate::descent::BOSS_DEPTHS.iter().map(|(k, _)| *k).find(|k| *k == cause);
    if let Some(b) = boss {
        if !l.pkg.drills.iter().any(|d| d.boss == b) && l.pkg.owned.contains("hunter") && l.pkg.stance != "hunter" {
            return Some(crate::wire::Lever { kind: "package".into(), text: "Hunter".into() });
        }
        let scar = l.pkg.scar(b, &l.kills) / SCAR_PCT;
        return Some(crate::wire::Lever { kind: "wait".into(), text: if scar > 0 { format!("scarred ×{scar}") } else { "drill next".into() } });
    }
    Some(crate::wire::Lever { kind: "wait".into(), text: format!("{} L{}", name(&l.pkg.stance), (l.pkg.level(&l.pkg.stance) + 1).min(MAX_LEVEL)) })
}

/// A row's package label for the verdict and the trace (`Steady`, `drill · Warlord`), if it is one.
pub fn row_label(row: &Row) -> Option<String> {
    let o = row.origin.as_deref()?;
    let (kind, id) = o.split_once(':')?;
    match kind {
        "stance" | "tactic" | "temper" => Some(name(id).to_string()),
        "drill" => Some(format!("drill · {}", crate::sifter::boss_short(id))),
        _ => None,
    }
}

// ---------------------------------------------------------------- the wire

/// The lineage's packages on the wire (`Lineage.packages`).
pub fn wire(l: &LineageState) -> crate::wire::PackagesWire {
    let p = &l.pkg;
    let slot_of = |id: &str| -> Option<u32> {
        if p.stance == id || p.temperament.as_deref() == Some(id) {
            Some(0)
        } else {
            p.tactics.iter().position(|t| t == id).map(|i| i as u32)
        }
    };
    let mut all: Vec<crate::wire::PackageWire> = PACKAGES
        .iter()
        .filter(|d| d.kind != Kind::Temperament || p.owned.contains(d.id) || p.offer.iter().any(|c| c == d.id))
        .map(|d| {
            let runs = p.runs.get(d.id).copied().unwrap_or(0);
            crate::wire::PackageWire {
                id: d.id.into(),
                name: d.name.into(),
                kind: d.kind.word().into(),
                level: level_of(runs),
                runs,
                next_at: next_at(runs),
                slot: slot_of(d.id),
                owned: p.owned.contains(d.id),
                trigger: if p.owned.contains(d.id) { String::new() } else { d.trigger.into() },
                level_price: p.owned.contains(d.id).then(|| level_price(l, d.id)).flatten(),
            }
        })
        .collect();
    if p.stance == CUSTOM {
        all.insert(0, crate::wire::PackageWire { id: CUSTOM.into(), name: "custom".into(), kind: "stance".into(), level: 1, slot: Some(0), owned: true, ..Default::default() });
    }
    let set = l.rules();
    let usable = |_: &Cond| true;
    let shadow = set.shadowed_by(l.max_rows(), usable);
    let rows = set.rows.iter().zip(&shadow).map(|(row, s)| crate::wire::RowSource { label: row_label(row).unwrap_or_default(), shadowed_by: s.map(|i| i as u32) }).collect();
    crate::wire::PackagesWire {
        all,
        stance: p.stance.clone(),
        tactics: p.tactics.clone(),
        tactic_slots: tactic_slots(l) as u32,
        temperament: p.temperament.clone(),
        temperament_open: temperament_open(l),
        offer: p.offer.clone(),
        drills: p.drills.iter().map(|d| crate::wire::DrillWire { boss: d.boss.clone(), rows: d.rows.clone(), revoked: d.revoked, scar: p.scar(&d.boss, &l.kills) }).collect(),
        scars: p.meets.iter().filter(|(k, _)| crate::descent::boss_depth(k).is_some()).map(|(k, _)| (k.clone(), p.scar(k, &l.kills))).filter(|(_, s)| *s > 0).collect(),
        pen_open: p.pen_open,
        rows,
        literal: p.literal,
    }
}

// ---------------------------------------------------------------- the camp's prices

/// A package move the camp can make, priced on the paired panel against the set as it stands
/// (`Guarded · death −8`): the shares of the sends that pass the record, bank and die, and their
/// moves.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PkgOption {
    pub id: String,
    /// `equip` (a stance, a tactic in `slot`, a wake card) or `level` (a mark spend).
    pub action: String,
    #[serde(default)]
    pub slot: usize,
    /// Marks a level spend costs (0 for an equip).
    #[serde(default)]
    pub price: u32,
    pub past: f64,
    pub bank: f64,
    pub death: f64,
    /// The share of the sends that reach the record's floor.
    #[serde(default)]
    pub reach: f64,
    /// The sends' mean deepest floor, and its move.
    #[serde(default)]
    pub mean: f64,
    #[serde(default)]
    pub d_mean: f64,
    /// The move in the share past the record read at the wall (from the deepest lit waystone at or
    /// above it); 0 with no stone.
    #[serde(default)]
    pub d_wall: f64,
    pub d_past: f64,
    pub d_bank: f64,
    pub d_death: f64,
    #[serde(default)]
    pub d_reach: f64,
}

/// Every move the camp offers now: each arrived stance not worn, each arrived tactic into each open
/// slot, each wake card not worn, each worn package's next level the marks pay for.
pub fn candidates(l: &LineageState) -> Vec<(String, String, usize)> {
    let p = &l.pkg;
    if p.literal {
        return Vec::new();
    }
    let mut out = Vec::new();
    for d in PACKAGES {
        if !p.owned.contains(d.id) {
            continue;
        }
        match d.kind {
            Kind::Stance if p.stance != d.id => out.push((d.id.to_string(), "equip".to_string(), 0)),
            Kind::Tactic => {
                for slot in 0..tactic_slots(l) {
                    if !p.tactics.iter().any(|t| t == d.id) {
                        out.push((d.id.to_string(), "equip".to_string(), slot));
                    }
                }
            }
            _ => {}
        }
    }
    for c in &p.offer {
        if p.temperament.as_deref() != Some(c.as_str()) {
            out.push((c.clone(), "equip".to_string(), 0));
        }
    }
    for id in p.equipped() {
        if def(&id).is_some() && level_price(l, &id).is_some_and(|m| m <= l.marks) {
            out.push((id, "level".to_string(), 0));
        }
    }
    out
}

/// Make a move (`equip` / `level`).
pub fn apply(l: &mut LineageState, id: &str, action: &str, slot: usize) -> Result<(), String> {
    match action {
        "level" => spend_level(l, id).map(|_| ()),
        _ if l.pkg.offer.iter().any(|c| c == id) => pick(l, id),
        _ => equip(l, id, slot),
    }
}

fn shares(g: &crate::engine::Game, set: &RuleSet, sims: u32) -> (f64, f64, f64, f64, f64) {
    read_shares(g, &crate::forecast::camp_panel(g, set, sims))
}

fn read_shares(g: &crate::engine::Game, rs: &[crate::forecast::SimResult]) -> (f64, f64, f64, f64, f64) {
    let k = rs.len().max(1) as f64;
    let best = g.lineage.best_depth;
    let past = rs.iter().filter(|r| r.max_depth > best).count() as f64 / k;
    let bank = rs.iter().filter(|r| r.tier == crate::engine::ExitTier::Bank).count() as f64 / k;
    let death = rs.iter().filter(|r| r.tier == crate::engine::ExitTier::Death).count() as f64 / k;
    let reach = rs.iter().filter(|r| r.max_depth >= best).count() as f64 / k;
    let mean = rs.iter().map(|r| r.max_depth as f64).sum::<f64>() / k;
    (past, bank, death, reach, mean)
}

/// A game's panel and the wall's, their passages from D1 shared: the wall's sends start at `stone`
/// and are paid the gold of the floors above it (`forecast::passage_for`: sims from D1, each stopped
/// at `stone`), and the game's own sends either start at D1 — the same sims, run on (its camp panel,
/// when the panel is run here, not read from the memo) — or at a shallower start whose passage is the
/// wall's sims cut there. Priced from those sims (`forecast::passage_from`), each passage is the one
/// `passage_for` runs; the shares are `shares`' and the wall's past share `at_wall`'s.
fn panels(g: &crate::engine::Game, set: &RuleSet, sims: u32, stone: Option<u32>) -> ((f64, f64, f64, f64, f64), Option<f64>) {
    use crate::forecast::{camp_panel, panel_key, passage_from, passage_run, sim_start, FORECAST_SIMS};
    let wall = stone.map(|s| {
        let mut w = g.sim_clone();
        w.lineage.start = s;
        w
    });
    let wall_start = wall.as_ref().map_or(1, sim_start);
    let start = sim_start(g);
    // (a panel read from the memo, or continued from a memoised first pass, was run here on a set that
    // plays alike — `played_key` — not on `set` itself: its passages are left to it)
    let memo = g.panel_cache.borrow();
    let cached = memo.contains_key(&panel_key(g, set, sims)) || (sims > FORECAST_SIMS && memo.contains_key(&panel_key(g, set, FORECAST_SIMS)));
    drop(memo);
    let own = if wall_start > 1 && start > 1 && start < wall_start && !cached {
        // (the deeper passage first: the shallower one is its sims cut, then the panel reads it)
        let w = wall.as_ref().expect("a wall start");
        let d1 = passage_run(w, set, wall_start);
        passage_from(w, set, wall_start, &d1);
        passage_from(g, set, start, &d1);
        read_shares(g, &camp_panel(g, set, sims))
    } else {
        let panel = camp_panel(g, set, sims);
        if let Some(w) = wall.as_ref().filter(|_| wall_start > 1 && g.lineage.start == 1 && !cached) {
            passage_from(w, set, wall_start, &panel);
        }
        read_shares(g, &panel)
    };
    (own, wall.map(|w| shares(&w, set, sims).0))
}

/// The camp's package prices (`sims` sends each, on the camp's seeds), best move first — read twice:
/// from where the sends start (the walk and the night), and at the wall (`d_wall`: the share past the
/// record from the deepest lit waystone at or above it, the wall's own panel as `wall::search` reads it).
/// A move that answers the wall reads there, under the noise of the floors above it; one that only helps
/// at the frontier while the walk to it suffers reads on the first panel. (Read from the stone alone, a
/// stance that walks home early looked best.)
pub fn options(g: &crate::engine::Game, sims: u32) -> Vec<PkgOption> {
    let best = g.lineage.best_depth;
    let stone = g.lineage.stones().into_iter().filter(|w| *w <= best && *w > g.lineage.start.max(1)).max();
    let (base, wall_base) = panels(g, g.lineage.rules(), sims, stone);
    let mut out: Vec<PkgOption> = Vec::new();
    for (id, action, slot) in candidates(&g.lineage) {
        let mut c = g.sim_clone();
        if apply(&mut c.lineage, &id, &action, slot).is_err() {
            continue;
        }
        let set = compile(&c.lineage);
        let ((past, bank, death, reach, mean), wall) = panels(&c, &set, sims, stone);
        let d_wall = match (wall_base, wall) {
            (Some(a), Some(b)) => b - a,
            _ => 0.0,
        };
        let price = if action == "level" { level_price(&g.lineage, &id).unwrap_or(0) } else { 0 };
        out.push(PkgOption { id, action, slot, price, past, bank, death, reach, mean, d_past: past - base.0, d_bank: bank - base.1, d_death: death - base.2, d_reach: reach - base.3, d_mean: mean - base.4, d_wall });
    }
    out.sort_by(|a, b| score(b).total_cmp(&score(a)));
    out
}

/// A move's worth: the sends past the record, then those reaching it and banked, the floors the
/// sends reach (a tenth a floor: the long walk from D1 the record sits under), less those that die.
pub fn score(o: &PkgOption) -> f64 {
    o.d_past + 0.3 * o.d_reach + 0.2 * o.d_bank - 0.2 * o.d_death + 0.1 * o.d_mean + o.d_wall
}
