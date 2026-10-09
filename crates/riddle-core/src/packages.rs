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
    stance("guarded", "Guarded", "meet Captain"),
    stance("bold", "Bold", "tomorrow"),
    stance("hunter", "Hunter", "tomorrow"),
    tactic("boss_focus", "boss focus"),
    tactic("corridor_fighting", "corridor fighting"),
    tactic("kite_archers", "kite archers"),
    tactic("thief_guard", "thief guard"),
    tactic("gas_step", "gas step"),
    tactic("pack_break", "pack break"),
    PackageDef { id: "cadence", kind: Kind::Tactic, name: "Mirror rhythm", card: Some("cadence"), trigger: "meet Mirror King", temperament: None },
    // Cut 110 (cohort ad71e72: "nothing addressed the golem wall"; the owner: tactics, not the pen, are the
    // player's tuning): a wall's counter is a tactic that arrives when the wall is met, beside the drip
    PackageDef { id: "reflect_read", kind: Kind::Tactic, name: "mirror read", card: Some("reflect_read"), trigger: "meet a reflector", temperament: None },
    PackageDef { id: "noise_discipline", kind: Kind::Tactic, name: "quiet steps", card: Some("noise_discipline"), trigger: "meet a blinder", temperament: None },
    PackageDef { id: "deep_march", kind: Kind::Tactic, name: "deep march", card: Some("deep_march"), trigger: "enter the Deep", temperament: None },
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

/// Plain-language purpose; detailed conditions remain in the compiled rules.
pub fn description(id: &str) -> &'static str {
    match id {
        "steady" => "Heal when hurt · fight nearest",
        "guarded" => "Heal early · recover between fights",
        "bold" => "Push deeper · fight while hurt",
        "hunter" => "Prioritise ranged enemies and bosses",
        "boss_focus" => "Prioritise bosses and their summons",
        "corridor_fighting" => "Fight groups from narrow corridors",
        "kite_archers" => "Dodge archer shots · target archers",
        "thief_guard" => "Prioritise thieves · protect supplies",
        "gas_step" => "Keep gas enemies at range",
        "pack_break" => "Split groups · finish weak foes",
        "cadence" => "Alternate attacks against mirrors",
        "reflect_read" => "Shoot or burn reflectors · never melee them",
        "noise_discipline" => "Rest to full · slip past blinders",
        "deep_march" => "Press on through explored floors",
        "skittish" => "Retreat when hurt and surrounded",
        "unbowed" => "Face bosses while healthy",
        "light_hands" => "Collect loot after kills",
        "iron_gut" => "Try unknown potions while healthy",
        CUSTOM => "Written hero rules",
        _ => "",
    }
}

/// Runs a package needs for L2 · L3 · L4 · L5 (tunable; the gate is the felt pace).
pub const LEVEL_RUNS: [u32; 4] = [10, 40, 120, 220];
pub const MAX_LEVEL: u32 = 5;

pub fn level_of(runs: u32) -> u32 {
    1 + crate::balance::get().level_runs.iter().filter(|&&n| runs >= n).count() as u32
}

/// Runs to the next level, if any.
pub fn next_at(runs: u32) -> Option<u32> {
    crate::balance::get().level_runs.iter().copied().find(|&n| runs < n)
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
    /// The worn temperament is the player's pick (`pick`), not the wake's draw: a new heir keeps it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub temperament_chosen: bool,
    /// Cut 111: tactic id → the variant of its L3 row the player chose (0 the first, 1 the second).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub variants: BTreeMap<String, u8>,
    /// Cut 115 §4: tactics a death's fix put on (`take_fix`) — their rows are credited `taught` until the
    /// player equips the tactic or sets its variant himself.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub taught: BTreeSet<String>,
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
            variants: BTreeMap::new(),
            taught: BTreeSet::new(),
            temperament_chosen: false,
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
        (self.meets.get(boss).copied().unwrap_or(0) * crate::balance::get().scar_pct).min(crate::balance::get().scar_cap)
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

/// Cut 30.5 (the owner, 2026-10-02: a new record never ends the run): the hp under which a stance goes home —
/// banking past the record, returning before it.
/// (tuned on the idle floor's rows: home at 40 % held IDLE under D8 on day 1 on every seed; 25 % is Cut 30's pace)
pub const STEADY_HOME: i32 = 25;
/// Out of heals, Steady goes home under this.
pub const STEADY_DRY: i32 = 40;
pub const GUARDED_HOME: i32 = 30;
pub const HUNTER_HOME: i32 = 20;

/// A walking stance's way home: hurt under `hurt` % — a bank once past the record (the checkpoints secured the carry
/// before it; this banks the rest), a return before it — or out of heals under `dry` %, a bank (out of supplies).
fn home_rows(best: u32, hurt: i32, dry: i32) -> Vec<Row> {
    let past = bank_at(best, 0);
    vec![
        r(vec![n("hp<", hurt), n("depth>=", past)], Verb::new("bank")),
        r(vec![n("hp<", hurt)], Verb::new("return")),
        r(vec![Cond::t("lacks", "heal"), n("hp<", dry)], Verb::new("bank")),
    ]
}
/// Steady rests only when well hurt (`Guarded` rests at 80 %): L2 under this, L3 under the next.
pub const STEADY_REST: i32 = 40;
pub const STEADY_REST_L3: i32 = 50;
/// Guarded's way home: earlier than Steady's, never so early it cannot reach the record.
pub const GUARDED_RETURN: i32 = 25;

/// The heal threshold of a stance at a level (a drill above the guard rows yields to it).
pub fn heal_pct(id: &str, level: u32) -> i32 {
    let b = crate::balance::get();
    let upgraded = usize::from(level >= 2);
    match id {
        "steady" => b.steady_heal[upgraded],
        "hunter" => b.hunter_heal[upgraded],
        "guarded" => b.guarded_heal[upgraded],
        "bold" => b.bold_heal,
        _ => 30,
    }
}

/// A stance's rows at a level: (guard rows, fallback rows). `best` is the lineage's record.
pub fn stance_rows(id: &str, level: u32, best: u32) -> (Vec<Row>, Vec<Row>) {
    let heal = heal_pct(id, level);
    let drink = r(vec![n("hp<", heal)], Verb::arg("drink", "heal"));
    let attack = r(vec![n("foes>=", 1)], Verb::arg("attack", "nearest"));
    match id {
        // Cut 30.5 (the owner, 2026-10-02: a new record never ends the run — it is a checkpoint that secures the
        // carry): a stance goes home only hurt or out of heals — banking (everything) once past the record, else
        // returning (60 %). The stances keep their characters: Steady goes home earliest of the walkers (40 %),
        // Guarded earlier still and rests between fights, Hunter late (30 %) with the boss and the summoned first,
        // Bold never turns back before the record and goes on hurt.
        // heal 30 %, home hurt under 25 % or out of heals under 40 %, attack nearest. L2 heals at 35 % and rests
        // when well hurt (under 40 %), L3 rests under 50 % (60 % after D18), L4 goes on without heals to 35 %, L5 steps off a
        // telegraph when hurt. The long rest is `Guarded`'s.
        "steady" => {
            let mut g = vec![drink];
            g.extend(home_rows(best, crate::balance::get().steady_home, crate::balance::get().steady_dry[usize::from(level >= 4)]));
            if level >= 5 {
                g.push(r(vec![tag("telegraph"), n("hp<", 35)], Verb::new("retreat")));
            }
            let mut f = vec![attack];
            if level >= 2 {
                // Preserve the first-session run/training cadence; deeper floors need more recovery.
                let rest = crate::balance::get().steady_rest[usize::from(level >= 3)];
                let rest = if best < 18 { rest.min(STEADY_REST_L3) } else { rest };
                f.push(r(vec![n("hp<", rest)], Verb::new("rest")));
            }
            (g, f)
        }
        // heal 40 %, home hurt under 30 %, bank out of heals under 45 % when recovery is unsafe, and recover fully
        // between fights. L2 heals at 45 %, L3 meets a telegraphed blow with a drink. L4 backs into a corridor
        // against a crowd; L5 goes on hurt to 25 %. An empty pack alone never ends a recoverable walk.
        "guarded" => {
            let mut g = vec![drink];
            // Rest succeeds only without foes, poison, or a hazardous tile. Recover before the dry exit;
            // the hurt exits still come first, and a guard unable to rest banks out of supplies.
            let mut home = home_rows(best, crate::balance::get().guarded_home[usize::from(level >= 5)], crate::balance::get().guarded_dry[usize::from(level >= 5)]);
            home.insert(2, r(vec![n("hp<", 100)], Verb::new("rest")));
            g.extend(home);
            if level >= 3 {
                // (a telegraphed blow is met with a drink, not a step back: `telegraph → retreat` looped
                // retreat ↔ explore on ~3 % of sends — the stall row)
                g.push(r(vec![tag("telegraph"), n("hp<", 60)], Verb::arg("drink", "heal")));
            }
            if level >= 4 {
                g.push(r(vec![n("foes>=", 3), n("hp<", 60)], Verb::new("back_corridor")));
            }
            (g, vec![attack, r(vec![n("hp<", 100)], Verb::new("rest"))])
        }
        // heal 25 %, never home before the record — past it, banks under 25 % (L5: 20 %) — the boss first and to the
        // end (no heal stops the blow), attack the weakest; hurt with the stairs in reach he dives. L2 rests under 50 %.
        "bold" => {
            let g = vec![
                r(vec![tag("boss"), Cond::flag("on_hurt")], Verb::arg("attack", "tag:boss")),
                drink,
                r(vec![n("hp<", if level >= 5 { 20 } else { 25 }), n("depth>=", bank_at(best, 0))], Verb::new("bank")),
                r(vec![tag("boss")], Verb::arg("attack", "tag:boss")),
                r(vec![n("hp<", 40), Cond::flag("path_stairs")], Verb::new("descend")),
            ];
            let mut f = vec![r(vec![n("foes>=", 1)], Verb::arg("attack", "lowest"))];
            if level >= 2 {
                f.push(r(vec![n("hp<", 50)], Verb::new("rest")));
            }
            (g, f)
        }
        // heal 30 %, home hurt under 20 % or out of heals under 35 %, the summoned and the boss first. L2 heals at
        // 35 % and rests under 50 %, L4 the casters first, L5 home at 15 %.
        "hunter" => {
            let mut g = vec![drink];
            g.extend(home_rows(best, if level >= 5 { 15 } else { HUNTER_HOME }, 35));
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

/// Cut 110: the wall tactics — each arrives when its wall is met (not on the drip) and plays only in its
/// situation (`situation_conds`), so it never costs a slot's worth of play elsewhere.
pub const WALL_TACTICS: [&str; 3] = ["reflect_read", "noise_discipline", "deep_march"];
fn drip(id: &str) -> bool {
    id != "cadence" && !WALL_TACTICS.contains(&id)
}
fn wall_met(l: &LineageState, id: &str) -> bool {
    match id {
        "reflect_read" => crate::facts::has_tag_fact(&l.facts, "reflect_melee"),
        "noise_discipline" => crate::facts::has_tag_fact(&l.facts, "blind"),
        "deep_march" => l.facts.contains("biome:deep"),
        _ => false,
    }
}

/// Cut 111 (owner: tactics, not the pen, are the player's tuning; cohort c4705f9: "Steady + boss focus — little
/// felt mine"): a tactic is worn one of two ways, each a different trade. Cut 115 §2: the pick is open from L1
/// and the two differ in the tactic's **main** row — its first: the first variant leads with the card itself (the
/// card's own read of the situation), the second with its own row ahead of the card (`boss first`: the boss before
/// anything the card would do); the first variant's L3 row follows the card.
pub fn variants(id: &str) -> Option<[&'static str; 2]> {
    Some(match id {
        "boss_focus" => ["summons first", "boss first"],
        "kite_archers" => ["hunt", "fall back"],
        "thief_guard" => ["chase", "head home"],
        "gas_step" => ["step away", "wade in"],
        "pack_break" => ["to corridor", "stand"],
        "corridor_fighting" => ["at three", "at two"],
        _ => return None,
    })
}

/// A tactic's rows at a level: its card (the card's rows play at the row), and from L3 a row the
/// card's situation wants (the first variant).
pub fn tactic_rows(id: &str, level: u32) -> Vec<Row> {
    tactic_rows_v(id, level, 0)
}

/// Cut 115 §2: the second variant's main rows, ahead of the card from L1 — each marked whether it leads the set (before
/// the stance's guard rows: `boss first` swings at the boss before the heal, down to 20 %).
fn variant_main(id: &str) -> Option<Vec<(Row, bool)>> {
    Some(match id {
        // the boss before anything: he walks rested (a rest under 70 % between fights) and swings at him before the heal
        "boss_focus" => vec![(r(vec![tag("boss"), n("hp>", 20)], Verb::arg("attack", "tag:boss")), true), (r(vec![n("hp<", 70)], Verb::new("rest")), false)],
        // archers wearing him down are the sign to go home
        "kite_archers" => vec![(r(vec![tag("ranged"), n("hp<", 50)], Verb::new("return")), false)],
        // a thief in reach of a hurt hero is the sign to go home with what he carries
        "thief_guard" => vec![(r(vec![tag("thief"), n("hp<", 70)], Verb::new("return")), false)],
        // burn them when fire is packed, else pop them in reach, before the step back
        "gas_step" => vec![(r(vec![tag("gas")], Verb::arg("throw", "fire,tag:gas")), true), (r(vec![tag("gas"), n("hp>", 25)], Verb::arg("attack", "tag:gas")), true)],
        // stand in any crowd and break it, the weakest first, before the heal — down to 20 %
        "pack_break" => vec![(r(vec![n("foes>=", 2), n("adj>=", 1), n("hp>", 20)], Verb::arg("attack", "lowest")), true)],
        // into a corridor from two foes, not three; outnumbered three to one and under half, home
        "corridor_fighting" => vec![(r(vec![n("foes>=", 3), n("hp<", 50)], Verb::new("return")), false), (r(vec![n("foes>=", 2)], Verb::new("back_corridor")), false)],
        _ => return None,
    })
}

/// The rows a worn tactic's variant puts ahead of the stance's guard rows (`variant_main`'s leading rows).
pub fn tactic_lead_rows(id: &str, variant: u8) -> Vec<Row> {
    match variant_main(id) {
        Some(rows) if variant >= 1 => rows.into_iter().filter(|x| x.1).map(|x| x.0).collect(),
        _ => Vec::new(),
    }
}

/// A tactic's rows at a level with its variant (the second leads with its own row from L1).
pub fn tactic_rows_v(id: &str, level: u32, variant: u8) -> Vec<Row> {
    if id == "cadence" { return vec![r(vec![tag("mirror")], Verb::arg("tactic", id))]; }
    if WALL_TACTICS.contains(&id) {
        let when = match id {
            "reflect_read" => vec![tag("reflect_melee")],
            "noise_discipline" => vec![Cond::t("in", "deep"), n("hp<", 90)],
            _ => vec![Cond::t("in", "deep")],
        };
        return vec![r(when, Verb::arg("tactic", id))];
    }
    let card = r(vec![], Verb::arg("tactic", id));
    if variant >= 1 {
        if let Some(main) = variant_main(id) {
            // (a leading row is compiled ahead of the guard rows: `tactic_lead_rows`)
            let mut v: Vec<Row> = main.into_iter().filter(|x| !x.1).map(|x| x.0).collect();
            v.push(card);
            return v;
        }
    }
    let mut v = vec![card];
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
    let mut rows = vec![guard(scope_drill_row(boss, crate::facts::counter_row(boss)))];
    if boss == "lich" {
        rows.push(guard(scope_drill_row(boss, r(vec![tag("boss")], Verb::arg("attack", "tag:boss")))));
    }
    if boss == "foundry_master" {
        // Keep the original deep-only scope and yield to sustain. The attack's selector
        // already requires a buffer, so a separate tag condition would be redundant.
        rows.push(guard(r(vec![n("depth>=", 19)], Verb::arg("attack", "tag:buffer"))));
    }
    rows
}

/// A boss's counter targets its known distinguishing tag, leaving unrelated bosses to their drill.
/// Stored counter facts keep their original fingerprint; only generated/copy templates are scoped.
fn scope_drill_row(boss: &str, mut row: Row) -> Row {
    if boss == "lurker_queen" {
        // Only the generated silence template changes scope. Legacy saved drills used
        // `blind`, which spent the Queen's scrolls on ordinary lurkers before D28.
        // Saved bodies and explicitly authored pen rows retain their conditions.
        let counter = row.verb == Verb::arg("read", "silence")
            && row.conds.iter().all(|c| c.k == "hp>"
                || c.k == "foe_tag" && matches!(c.t.as_deref(), Some("boss" | "blind" | "brood")));
        if counter {
            for cond in &mut row.conds {
                if cond.k == "foe_tag" { cond.t = Some("brood".into()); }
            }
        }
        return row;
    }
    let tag = match boss {
        "bloat_mother" => "gas",
        "lich" => "undead",
        "mirror_king" => "mirror",
        _ => return row,
    };
    for cond in &mut row.conds {
        if cond.k == "foe_tag" && cond.t.as_deref() == Some("boss") {
            cond.t = Some(tag.into());
        }
    }
    row
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

/// A learned drill yields to the stance currently worn; stored rows and player rows stay intact.
fn effective_drill_rows(d: &Drill, heal: i32) -> Vec<Row> {
    d.rows.iter().cloned().map(|row| {
        let mut row = scope_drill_row(&d.boss, row);
        if d.boss == "foundry_master" && row.verb == Verb::arg("attack", "tag:buffer")
            && row.conds.len() == 2 && row.conds[0] == tag("buffer")
            && (row.conds[1] == n("depth>=", 19) || row.conds[1].k == "hp>") {
            // Both legacy templates keep their saved form; the effective generated row
            // restores the original floor scope and yields to the current stance's heal.
            row.conds = vec![n("depth>=", 19), n("hp>", heal)];
        }
        for cond in &mut row.conds {
            if cond.k == "hp>" { cond.n = Some(heal); }
        }
        row
    }).collect()
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
    // An explicitly chosen mirror counter must precede generic learned boss attacks,
    // while yielding to the same heal threshold as learned counter drills.
    if p.tactics.iter().any(|id| id == "cadence") {
        rows.extend(tagged(vec![r(vec![tag("mirror"), n("hp>", heal)], Verb::arg("tactic", "cadence"))], "tactic:cadence"));
    }
    // Later counter drills precede older generic boss attacks. Player rows remain above them.
    for d in p.drills.iter().rev().filter(|d| !d.revoked) {
        let origin = format!("drill:{}", d.boss);
        rows.extend(effective_drill_rows(d, heal).into_iter().map(|row| row.from(&origin)));
    }
    // Cut 115 §2: a variant that commits leads the stance's guard rows
    for t in &p.tactics {
        rows.extend(tagged(tactic_lead_rows(t, p.variants.get(t).copied().unwrap_or(0)), &format!("tactic:{t}")));
    }
    let origin = format!("stance:{}", p.stance);
    let (guard, fallback) = if p.stance == CUSTOM { (p.custom.clone(), Vec::new()) } else { stance_rows(&p.stance, level, best) };
    rows.extend(tagged(guard, &origin));
    if let Some(row)=crate::specialization::row(l) {rows.push(row);}
    for t in &p.tactics {
        if t == "cadence" { continue; }
        rows.extend(tagged(tactic_rows_v(t, p.level(t), p.variants.get(t).copied().unwrap_or(0)), &format!("tactic:{t}")));
    }
    if let Some(t) = &p.temperament {
        rows.extend(tagged(temperament_rows(t, p.level(t)), &format!("temper:{t}")));
    }
    // Chosen policy acts before automatic gun handling; generic stance attacks follow.
    if let Some(row)=crate::firearm::row(l) {rows.push(row);}
    rows.extend(tagged(fallback, &origin));
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
///
/// `by_origin` (the public door, `Game::set_rules`): the origin tag is ownership — every row
/// still tagged with a package's origin is that package's, whatever its tokens. The client
/// retags a row the player edits, moves out of the packages or a patch rewrites (`player` /
/// `patch`); an untouched package row that a recompile has since changed (a stance's home row
/// deepening with the best depth, a level's heal threshold) is the package's stale copy, not a
/// row the player wrote. Rater A on c4705f9: any edit from a copy older than the last run's
/// recompile turned those stale rows into five unasked `player` rows and `5/4 · drop one`.
/// Without it (a projection of a core-made candidate, whose rewritten rows keep the package's
/// origin) a package row that no longer matches the compiled one is the edit's.
pub fn absorb(l: &mut LineageState, set: &RuleSet, by_origin: bool) {
    let compiled = compile(l);
    let is_ours = |row: &Row| row.is_pkg() && (by_origin || compiled.rows.iter().any(|c| c == row && c.origin == row.origin));
    if l.pkg.stance == CUSTOM {
        // the custom stance is the pen's own set: every row but the drills
        l.pkg.custom = set.rows.iter().filter(|row| !row.origin.as_deref().is_some_and(|o| o.starts_with("drill:") || ((o.starts_with("style:")||o=="class:gunner")&&is_ours(row)))).cloned().map(|mut row| {
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

/// The state a prospective public rule edit produces. Historical sets and replay rules
/// stay literal unless their caller explicitly requests this projection.
pub fn project_edit(l: &LineageState, set: &RuleSet) -> LineageState {
    let mut edited = l.clone();
    if edited.pkg.literal {
        let i = edited.active_set.min(edited.sets.len() - 1);
        edited.sets[i] = set.clone();
    } else {
        absorb(&mut edited, set, false);
        recompile(&mut edited);
    }
    edited
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
fn pending_arrivals(l: &LineageState) -> Vec<String> {
    // Cut 30 (PROGRESSION_V2 §4): a drip, not a dump — `Guarded` with the Warlord met, `Bold` a day
    // after, `Hunter` a day after that; the tactics one per band boss slain or per day since the Warlord
    // fell (a card the lineage already owned arrives with the first). A day is the lineage's clock.
    let met = |k: &str| l.pkg.meets.get(k).copied().unwrap_or(0) > 0 || l.facts.contains(&format!("foe:{k}")) || l.kills.contains(k);
    let day = l.day;
    let since = |id: &str| l.pkg.arrived.get(id).map(|d| day > *d).unwrap_or(false);
    let slain = crate::descent::BOSS_DEPTHS.iter().filter(|(k, _)| l.kills.contains(*k)).count() as u32;
    let tactics_owned = l.pkg.owned.iter().filter(|id| drip(id) && def(id).is_some_and(|d| d.kind == Kind::Tactic)).count() as u32;
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
            (Kind::Tactic, "cadence") => rhythm_available(l),
            (Kind::Tactic, w) if WALL_TACTICS.contains(&w) => wall_met(l, w) && l.kills.contains("goblin_warlord"),
            (Kind::Stance, "steady") => true,
            (Kind::Stance, "guarded") => met("goblin_captain") || met("goblin_warlord"),
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
            if d.kind == Kind::Tactic && drip(d.id) {
                tactics_new += 1;
            }
            new.push(d.id.to_string());
        }
    }
    new
}

pub fn arrive(l: &mut LineageState) -> Vec<String> {
    let new = pending_arrivals(l);
    if !l.pkg.arrived.contains_key("tactics") && new.iter().any(|id| drip(id) && def(id).is_some_and(|d| d.kind == Kind::Tactic)) {
        l.pkg.arrived.insert("tactics".into(), l.day);
    }
    for id in &new {
        l.pkg.owned.insert(id.clone());
        l.pkg.arrived.insert(id.clone(), l.day);
    }
    new
}

/// A package's trigger as it stands (owner check, 2026-10-02: `⊘ a day on` read as nothing): a stance of
/// the drip arrives the day after the one before it — `tomorrow`, `next send` once that day has come,
/// `after <name>` while the one before is still to come.
fn trigger_now(l: &LineageState, d: &PackageDef, pending: &[String]) -> String {
    if d.kind == Kind::Tactic && drip(d.id) && l.kills.contains("goblin_warlord") {
        return if pending.iter().any(|id| id == d.id) { "next send" } else { "bosses or days" }.into();
    }
    let before = match d.id {
        "bold" => "guarded",
        "hunter" => "bold",
        _ => return d.trigger.into(),
    };
    match l.pkg.arrived.get(before) {
        Some(day) if l.day > *day => "next send".into(),
        Some(_) => "tomorrow".into(),
        None => format!("after {}", name(before)),
    }
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
/// The words for "wait": a boss's counter that only his drill can bring (the pen closed, no package of it here).
pub const COUNTER_WAIT: &str = "wait for drill";

/// Blind 1fb7786 (A: `try: attack boss` beside `boss focus ⊘ slay Warlord` — circular): what a player can take now
/// against `boss`'s counter `row`, in ≤ 3 words, and whether it is takeable: the row itself with the pen open; the
/// package that carries it when one has arrived (`boss focus`, `mirror read`); a revoked drill restored; else the
/// drill to come (`wait for drill`, not takeable) — never a package still locked.
pub fn counter_offer(l: &LineageState, boss: &str, row: &Row) -> (String, bool) {
    if l.pkg.literal || l.pkg.pen_open {
        return (crate::facts::counter_text(row), true);
    }
    let carries = |id: &str| {
        let d = def(id);
        let card = d.and_then(|d| d.card).unwrap_or(id);
        let lv = l.pkg.level(id).max(1);
        let v = l.pkg.variants.get(id).copied().unwrap_or(0);
        row.verb == Verb::arg("tactic", id)
            || tactic_rows_v(id, lv, v).iter().chain(tactic_lead_rows(id, v).iter()).any(|r| r.verb == row.verb)
            || crate::meta::unlock_rows(card).is_some_and(|rows| rows.iter().any(|r| r.verb == row.verb))
    };
    if let Some(d) = PACKAGES.iter().find(|d| d.kind == Kind::Tactic && available(l, d.id) && tactic_slots(l) > 0 && carries(d.id)) {
        return (d.name.to_string(), true);
    }
    if l.pkg.drills.iter().any(|d| d.boss == boss && d.revoked) {
        return ("restore drill".into(), true);
    }
    (COUNTER_WAIT.into(), false)
}

/// The Mirror King's counter (blind 1fb7786, A: `COUNTER: CADENCE` with `Mirror rhythm · ⊘ Clear dungeon` — a
/// counter that could not be slotted): it arrives when the King is met, as Cut 110's wall tactics do (and stays
/// with a cleared dungeon, as before).
fn rhythm_available(l: &LineageState) -> bool {
    let king = "mirror_king";
    l.ended || l.endgame.as_ref().is_some_and(|p| p.cleared.is_some()) || l.pkg.meets.get(king).copied().unwrap_or(0) > 0 || l.facts.contains(&format!("foe:{king}")) || l.kills.contains(king)
}
fn available(l: &LineageState, id: &str) -> bool {
    l.pkg.owned.contains(id) || id == "cadence" && rhythm_available(l)
}
pub fn equip(l: &mut LineageState, id: &str, slot: usize) -> Result<(), String> {
    if l.pkg.literal {
        return Err("no packages".into());
    }
    let d = def(id).ok_or("unknown package")?;
    if !available(l, id) {
        return Err("not yet".into());
    }
    match d.kind {
        Kind::Stance => l.pkg.stance = id.into(),
        Kind::Tactic => {
            let slots = tactic_slots(l);
            if slot >= slots.max(1) || slots == 0 {
                return Err("slot closed".into());
            }
            if id == "cadence" { l.pkg.owned.insert(id.into()); }
            l.pkg.taught.remove(id);
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
        Kind::Tactic => {
            l.pkg.tactics.retain(|t| t != id);
            l.pkg.taught.remove(id);
        }
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
    // blind 77030eb (A: "unbowed → light hands → iron gut without my choosing"): a personality the
    // player picked stays worn — card 1 of the new offer (in it when the draw skipped it); a change is
    // the player's pick, never the wake's. A lineage that never picked keeps the wake's draw (IDLE: the
    // keep without a pick walled IDLE at D23 — 3/16 seeds by day 12)
    if let Some(prev) = l.pkg.temperament.clone().filter(|t| l.pkg.temperament_chosen && ids.contains(&t.as_str())) {
        if let Some(i) = cards.iter().position(|c| *c == prev) {
            cards.remove(i);
        } else {
            cards.pop();
        }
        cards.insert(0, prev);
    }
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
    l.pkg.temperament_chosen = true;
    recompile(l);
    Ok(())
}

/// Marks a level spend costs (the next level's number): ◆2 for L2 … ◆5 for L5.
pub fn level_price(l: &LineageState, id: &str) -> Option<u32> {
    let lv = l.pkg.level(id);
    (lv < MAX_LEVEL).then_some(lv + 1)
}

/// Blind 77030eb (B: "levelling corridor fighting L1→L5 dropped D33 75%→25% with no reason"): the rows
/// a package's next level brings (or changes to) — its rows at the next level not among its rows now,
/// for the level button's why. Empty at the top level and for a level that changes no row.
pub fn level_adds(l: &LineageState, id: &str) -> Vec<Row> {
    let Some(d) = def(id) else { return Vec::new() };
    let lv = l.pkg.level(id);
    if lv >= MAX_LEVEL {
        return Vec::new();
    }
    let rows = |lv: u32| -> Vec<Row> {
        match d.kind {
            Kind::Stance => {
                let (g, f) = stance_rows(id, lv, l.best_depth);
                g.into_iter().chain(f).collect()
            }
            Kind::Tactic => tactic_rows_v(id, lv, l.pkg.variants.get(id).copied().unwrap_or(0)),
            Kind::Temperament => temperament_rows(id, lv),
        }
    };
    let now = rows(lv);
    rows(lv + 1).into_iter().filter(|r| !now.contains(r)).collect()
}

/// Cut 111 / Cut 115 §2: pick a tactic's variant (`variants`): owned, from L1. Free, instant, revocable; the
/// player's own pick (a fix's `taught` credit ends).
pub fn set_variant(l: &mut LineageState, id: &str, v: u8) -> Result<(), String> {
    if variants(id).is_none() || v > 1 {
        return Err("no such variant".into());
    }
    if !l.pkg.owned.contains(id) {
        return Err("not yet".into());
    }
    if v == 0 { l.pkg.variants.remove(id); } else { l.pkg.variants.insert(id.into(), v); }
    l.pkg.taught.remove(id);
    recompile(l);
    Ok(())
}

/// Cut 115 §4: a death's fix taken (`Death.pick`): the tactic worn — the first open slot, else the last — in the
/// variant that answers the death, credited `taught` (the fix did the learning) until the player re-picks it.
pub fn take_fix(l: &mut LineageState, id: &str, v: u8) -> Result<(), String> {
    let slots = tactic_slots(l);
    if slots == 0 {
        return Err("slot closed".into());
    }
    if !l.pkg.tactics.iter().any(|t| t == id) {
        let slot = l.pkg.tactics.len().min(slots - 1);
        equip(l, id, slot)?;
    }
    if variants(id).is_some() {
        set_variant(l, id, v)?;
    }
    l.pkg.taught.insert(id.to_string());
    Ok(())
}

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
            "goblin_warlord" => crate::balance::get().drill_meeting,
            _ if deep => crate::balance::get().deep_drill_days,
            "foundry_master" => crate::balance::get().drill_days + 1,
            _ => crate::balance::get().drill_days,
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
/// within reach of the record and not skipped by its starting floor) and whose item
/// the hero can name. Beating a boss once does not remove it from a later send
/// that still starts above its floor.
pub fn quartermaster(l: &LineageState) -> Vec<String> {
    let mut out = Vec::new();
    for d in l.pkg.drills.iter().filter(|d| !d.revoked) {
        let Some(depth) = crate::descent::boss_depth(&d.boss) else { continue };
        if l.best_depth + 2 < depth || l.start.max(1) > depth {
            continue;
        }
        if let Some(k) = drill_item(d) {
            if crate::item::is_identified(&l.facts, &l.flavours, &k) && !out.contains(&k) {
                out.push(k);
            }
        }
    }
    // (and what the pen's own rows name — a counter written before its drill is packed as the drill's
    // would be: a `read silence` row with no scroll on the shelf answers nothing)
    // (a boss's counter row only, and while the boss is in reach, as a drill's)
    if l.pkg.pen_open && !l.pkg.literal {
        for (boss, _) in crate::descent::BOSS_DEPTHS {
            let counter = crate::facts::counter_row(boss);
            if !l.pkg.pen.iter().any(|row| row.verb == counter.verb) {
                continue;
            }
            let Some(depth) = l.rules().route().boss_depth(boss) else { continue };
            if l.best_depth + 2 < depth || l.start.max(1) > depth {
                continue;
            }
            let d = Drill { boss: boss.to_string(), rows: vec![counter], revoked: false, announced: false };
            if let Some(k) = drill_item(&d) {
                if crate::item::is_identified(&l.facts, &l.flavours, &k) && !out.contains(&k) {
                    out.push(k);
                }
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
    // Cut 115 §4: a tactic (or a variant) that answers the death is the fix, before a purchase
    if let Some(pick) = pick_lever(l, cause) {
        return Some(pick);
    }
    let step = crate::kit::ladders(l).into_iter().filter_map(|lad| lad.next.map(|n| (n.price, n.label))).min_by_key(|x| x.0);
    if let Some((price, label)) = step.filter(|(p, _)| *p as i32 <= l.gold) {
        let _ = price;
        return Some(crate::wire::Lever { kind: "spend".into(), text: label, ..Default::default() });
    }
    let boss = crate::descent::BOSS_DEPTHS.iter().map(|(k, _)| *k).find(|k| *k == cause);
    if let Some(b) = boss {
        if !l.pkg.drills.iter().any(|d| d.boss == b) && l.pkg.owned.contains("hunter") && l.pkg.stance != "hunter" {
            return Some(crate::wire::Lever { kind: "package".into(), text: "Hunter".into(), ..Default::default() });
        }
        let scar = l.pkg.scar(b, &l.kills) / crate::balance::get().scar_pct;
        return Some(crate::wire::Lever { kind: "wait".into(), text: if scar > 0 { format!("scarred ×{scar}") } else { "drill next".into() }, ..Default::default() });
    }
    Some(crate::wire::Lever { kind: "wait".into(), text: format!("{} L{}", name(&l.pkg.stance), (l.pkg.level(&l.pkg.stance) + 1).min(MAX_LEVEL)), ..Default::default() })
}

/// Cut 115 §4: a death's tactic fix as a lever (`kind` `tactic`, `text` `gas step · burn`, the id and variant).
pub fn pick_lever(l: &LineageState, cause: &str) -> Option<crate::wire::Lever> {
    fix_pick(l, cause).map(|(id, v, text)| crate::wire::Lever { kind: "tactic".into(), text, id: Some(id), variant: Some(v as u32) })
}

/// A row's package label for the verdict and the trace (`Steady`, `drill · Warlord`), if it is one.
pub fn row_label(row: &Row) -> Option<String> {
    let o = row.origin.as_deref()?;
    let (kind, id) = o.split_once(':')?;
    match kind {
        "stance" | "tactic" | "temper" => Some(name(id).to_string()),
        "drill" => Some(format!("drill · {}", crate::sifter::boss_short(id))),
        "style" => crate::specialization::Style::parse(id).map(|s|s.name().into()),
        "class" if id=="gunner" => Some("Gunner".into()),
        _ => None,
    }
}

// ---------------------------------------------------------------- Cut 115: builds

/// Cut 115 §1 (owner, 2026-10-09: "builds from tactics"): a pair of worn picks that forms a named build, with one
/// small effect only that pair has. Each holds a tactic (IDLE wears none: its games are unchanged). The effect
/// plays in the sim (`turn::damage_hero` / `damage_monster`, read off the compiled rows' origins: `build_mask`).
pub struct Synergy {
    pub id: &'static str,
    /// The build's name (≤ 2 words).
    pub name: &'static str,
    /// The two picks (package ids: a stance, a tactic, a temperament).
    pub pair: [&'static str; 2],
    /// What it does (≤ 6 words).
    pub effect: &'static str,
}

pub const SYNERGIES: [Synergy; 7] = [
    Synergy { id: "bulwark", name: "Bulwark", pair: ["guarded", "corridor_fighting"], effect: "−1 melee taken in corridors" },
    Synergy { id: "duelist", name: "Duelist", pair: ["bold", "boss_focus"], effect: "+2 damage one on one" },
    Synergy { id: "marksman", name: "Marksman", pair: ["hunter", "kite_archers"], effect: "+3 on a fresh foe" },
    Synergy { id: "ghost", name: "Ghost", pair: ["skittish", "kite_archers"], effect: "−2 at range · −1 when hurt" },
    Synergy { id: "scavenger", name: "Scavenger", pair: ["light_hands", "pack_break"], effect: "+2 damage to packs" },
    Synergy { id: "iron_lungs", name: "Iron lungs", pair: ["iron_gut", "gas_step"], effect: "−2 gas and poison taken" },
    Synergy { id: "warden", name: "Warden", pair: ["steady", "thief_guard"], effect: "−2 from telegraphed blows" },
];

/// The effects' sizes (`turn.rs` reads them).
pub const BULWARK_ARMOUR: i32 = 1;
pub const DUELIST_EDGE: i32 = 2;
pub const MARKSMAN_EDGE: i32 = 3;
pub const GHOST_COVER: i32 = 2;
/// Ghost: under this hp % every blow is a point lighter.
pub const GHOST_HURT: i32 = 35;
pub const SCAVENGER_EDGE: i32 = 2;
pub const IRON_LUNGS: i32 = 2;
pub const WARDEN_GUARD: i32 = 2;

/// Cut 117 §3: the compiled set wears package `id` (a stance's or a tactic's rows are in it — `descent::affix_breakers`' read).
pub fn wears(rules: &RuleSet, id: &str) -> bool {
    rules.rows.iter().filter_map(|r| r.origin.as_deref()).any(|o| o.split_once(':').is_some_and(|(k, x)| matches!(k, "stance" | "tactic") && x == id))
}

/// A synergy's bit in `build_mask`.
pub fn synergy_bit(id: &str) -> u8 {
    SYNERGIES.iter().position(|s| s.id == id).map_or(0, |i| 1 << i)
}

thread_local! {
    /// Diagnostics only (`examples/builds.rs`): the synergies' effects switched off, to weigh what each pays.
    pub static SYNERGIES_OFF: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// The synergies a compiled set plays (a bit each, `synergy_bit`): both picks of a pair have rows in it. A
/// literal set (the harnesses' bots, the pen's custom stance) carries no package origins and plays none.
/// (One pass over the rows: it is read on every blow.)
pub fn build_mask(rules: &RuleSet) -> u8 {
    if SYNERGIES_OFF.with(|c| c.get()) {
        return 0;
    }
    let mut have: u32 = 0;
    for o in rules.rows.iter().filter_map(|r| r.origin.as_deref()) {
        let Some((kind, id)) = o.split_once(':') else { continue };
        if !matches!(kind, "stance" | "tactic" | "temper") {
            continue;
        }
        for (i, s) in SYNERGIES.iter().enumerate() {
            for (j, p) in s.pair.iter().enumerate() {
                if *p == id {
                    have |= 1 << (2 * i + j);
                }
            }
        }
    }
    let mut mask = 0;
    for i in 0..SYNERGIES.len() {
        if have >> (2 * i) & 3 == 3 {
            mask |= 1 << i;
        }
    }
    mask
}

/// The synergy a lineage's worn picks form (the first of the list).
pub fn synergy_of(l: &LineageState) -> Option<&'static Synergy> {
    let p = &l.pkg;
    if p.literal {
        return None;
    }
    let worn = p.equipped();
    SYNERGIES.iter().find(|s| s.pair.iter().all(|id| worn.iter().any(|w| w == id)))
}

/// A tactic's noun in a build's title (`Guarded skirmisher`).
fn tactic_noun(id: &str) -> &'static str {
    match id {
        "boss_focus" => "slayer",
        "corridor_fighting" => "skirmisher",
        "kite_archers" => "ranger",
        "thief_guard" => "keeper",
        "gas_step" => "dodger",
        "pack_break" => "breaker",
        "cadence" => "dancer",
        "reflect_read" => "reader",
        "noise_discipline" => "prowler",
        "deep_march" => "marcher",
        _ => "fighter",
    }
}
fn temper_noun(id: &str) -> &'static str {
    match id {
        "skittish" => "runner",
        "unbowed" => "stalwart",
        "light_hands" => "magpie",
        "iron_gut" => "taster",
        _ => "hero",
    }
}

/// Cut 115 §1: the build's two-word name from what the player wore — the synergy's when a pair forms one, else
/// the stance and the first tactic (`Guarded skirmisher`), else the stance and a picked temperament (`Bold
/// runner`); `None` while nothing is the player's (the school stance, no tactic, the wake's draw: IDLE).
pub fn build_name(l: &LineageState) -> Option<String> {
    let p = &l.pkg;
    if p.literal {
        return None;
    }
    if let Some(s) = synergy_of(l) {
        return Some(s.name.to_string());
    }
    let stance = if p.stance == CUSTOM { "Written".to_string() } else { name(&p.stance).to_string() };
    if let Some(t) = p.tactics.first() {
        return Some(format!("{stance} {}", tactic_noun(t)));
    }
    match p.temperament.as_deref().filter(|_| p.temperament_chosen) {
        Some(t) => Some(format!("{stance} {}", temper_noun(t))),
        None if p.stance != "steady" => Some(format!("{stance} hero")),
        None => None,
    }
}

/// The build on the wire (`Packages.build`).
pub fn build_wire(l: &LineageState) -> Option<crate::wire::BuildWire> {
    let title = build_name(l)?;
    let s = synergy_of(l);
    let p = &l.pkg;
    let mut picks = vec![name_of_pick(&p.stance, p)];
    picks.extend(p.tactics.iter().map(|t| name_of_pick(t, p)));
    if let Some(t) = p.temperament.as_deref().filter(|_| p.temperament_chosen) {
        picks.push(name(t).to_string());
    }
    Some(crate::wire::BuildWire { name: title, synergy: s.map(|s| s.id.to_string()), effect: s.map(|s| s.effect.to_string()), picks })
}

/// A worn pick as the build names it (`corridor fighting · at two`).
fn name_of_pick(id: &str, p: &PkgState) -> String {
    match variants(id) {
        Some(v) => format!("{} · {}", name(id), v[p.variants.get(id).copied().unwrap_or(0).min(1) as usize]),
        None => name(id).to_string(),
    }
}

/// Cut 115 §1: who chose a row — `picked` (a package the player equipped, a variant he set, a row he wrote),
/// `taught` (a drill, a death's fix), `default` (the school stance, the wake's temperament, a trait's own step),
/// `chores`. `origin` is the meters' key (`Meter.origins`): a row's origin, `chores`, `trait`, or empty (a row
/// with no origin: a literal set's, the player's).
pub fn credit(origin: &str, p: &PkgState) -> &'static str {
    let (kind, id) = origin.split_once(':').unwrap_or((origin, ""));
    match kind {
        "chores" => "chores",
        "trait" | "class" => "default",
        "patch" | "drill" => "taught",
        "tactic" if p.taught.contains(id) => "taught",
        "tactic" | "style" | "player" | "card" | "" => "picked",
        "stance" if id == "steady" => "default",
        "stance" => "picked",
        "temper" if p.temperament_chosen => "picked",
        _ => "default",
    }
}

/// The credit order the report reads.
pub const CREDITS: [&str; 4] = ["picked", "taught", "default", "chores"];

/// A meter's fires by credit (shares of all fires, 0..1, in `CREDITS` order; empty without fires).
pub fn credit_shares(origins: &BTreeMap<String, u32>, p: &PkgState) -> Vec<crate::wire::CreditShare> {
    let total: u32 = origins.values().sum();
    if total == 0 {
        return Vec::new();
    }
    CREDITS.iter().filter_map(|c| {
        let fires: u32 = origins.iter().filter(|(o, _)| credit(o, p) == *c).map(|(_, n)| *n).sum();
        (fires > 0).then(|| crate::wire::CreditShare { credit: c.to_string(), fires, share: (fires as f64 / total as f64 * 1000.0).round() / 1000.0 })
    }).collect()
}

/// Cut 115 §4: the tactic (and variant) that answers a death by `cause` (the killer's kind, or a hazard), when one
/// has arrived and a slot is open, and it is not worn so already: (id, variant, `gas step · burn`).
pub fn fix_pick(l: &LineageState, cause: &str) -> Option<(String, u8, String)> {
    if l.pkg.literal || tactic_slots(l) == 0 {
        return None;
    }
    let tags: Vec<&str> = match cause {
        "gas" | "poison" => vec!["gas"],
        _ if crate::descent::boss_depth(cause).is_some() || crate::defs::monster_def(cause).kind == cause => crate::defs::monster_def(cause).tags.to_vec(),
        _ => return None,
    };
    let fire = crate::item::is_identified(&l.facts, &l.flavours, "fire");
    let answer = |t: &str| -> Option<(&'static str, u8)> {
        Some(match t {
            "reflect_melee" => ("reflect_read", 0),
            "mirror" => ("cadence", 0),
            "blind" => ("noise_discipline", 0),
            "summoner" => ("boss_focus", 0),
            "boss" => ("boss_focus", 1),
            "ranged" => ("kite_archers", 0),
            "gas" => ("gas_step", u8::from(fire)),
            "thief" => ("thief_guard", 0),
            "pack" => ("pack_break", 1),
            _ => return None,
        })
    };
    // (a boss's own counter first — the mirror, the reflection, the brood — then what any of his kind answers)
    let order = ["reflect_melee", "mirror", "blind", "summoner", "ranged", "gas", "thief", "pack", "boss"];
    // Cut 117 §2 (client QA: the King's TRY alternated cadence ↔ boss focus in one slot): the tactic `take_fix` would
    // take off (the last slot, all slots full) — a pick that would replace another answer to this same killer is no fix
    let answers: Vec<&str> = order.iter().filter(|t| tags.contains(t)).filter_map(|t| answer(t).map(|a| a.0)).collect();
    let slots = tactic_slots(l);
    let replaced = (l.pkg.tactics.len() >= slots).then(|| l.pkg.tactics.get(slots - 1)).flatten();
    for t in order.iter().filter(|t| tags.contains(t)) {
        let Some((id, v)) = answer(t) else { continue };
        if !available(l, id) {
            continue;
        }
        let on = l.pkg.tactics.iter().any(|x| x == id);
        let worn = on && l.pkg.variants.get(id).copied().unwrap_or(0) == v;
        if worn {
            continue;
        }
        if !on && replaced.is_some_and(|r| r != id && answers.contains(&r.as_str())) {
            continue;
        }
        let text = match variants(id) {
            Some(names) => format!("{} · {}", name(id), names[v as usize]),
            None => name(id).to_string(),
        };
        return Some((id.to_string(), v, text));
    }
    None
}

// ---------------------------------------------------------------- the wire

/// The lineage's packages on the wire (`Lineage.packages`).
pub fn wire(l: &LineageState) -> crate::wire::PackagesWire {
    let p = &l.pkg;
    let pending = pending_arrivals(l);
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
                description: description(d.id).into(),
                kind: d.kind.word().into(),
                level: level_of(runs),
                runs,
                next_at: next_at(runs),
                slot: slot_of(d.id),
                owned: available(l, d.id),
                trigger: if available(l, d.id) { String::new() } else { trigger_now(l, d, &pending) },
                level_price: p.owned.contains(d.id).then(|| level_price(l, d.id)).flatten(),
                variants: variants(d.id).map(|v| v.iter().map(|x| x.to_string()).collect()).unwrap_or_default(),
                variant: (variants(d.id).is_some() && available(l, d.id)).then(|| p.variants.get(d.id).copied().unwrap_or(0) as u32),
                level_adds: if p.owned.contains(d.id) { level_adds(l, d.id) } else { Vec::new() },
            }
        })
        .collect();
    if p.stance == CUSTOM {
        all.insert(0, crate::wire::PackageWire { id: CUSTOM.into(), name: "custom".into(), description: description(CUSTOM).into(), kind: "stance".into(), level: 1, slot: Some(0), owned: true, ..Default::default() });
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
        drills: p.drills.iter().map(|d| crate::wire::DrillWire { boss: d.boss.clone(), rows: effective_drill_rows(d, heal_pct(&p.stance, p.level(&p.stance))), revoked: d.revoked, scar: p.scar(&d.boss, &l.kills) }).collect(),
        scars: p.meets.iter().filter(|(k, _)| crate::descent::boss_depth(k).is_some()).map(|(k, _)| (k.clone(), p.scar(k, &l.kills))).filter(|(_, s)| *s > 0).collect(),
        pen_open: p.pen_open,
        pen_needs: crate::systems::pen_needs(l),
        rows,
        literal: p.literal,
        build: build_wire(l),
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
    /// The paired sends (same seeds on both panels), and of them the ones this move ended better
    /// and worse than the worn set (`paired`).
    #[serde(default)]
    pub n: u32,
    #[serde(default)]
    pub better: u32,
    #[serde(default)]
    pub worse: u32,
    /// Cut 115 §3: the move read at each wall the lineage has met (its waystone's sends, paired), deepest first —
    /// `better at D8 Warlord · worse at D28 Queen`. The client's chooser only (`options_for`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub walls: Vec<WallRead>,
    /// Cut 117 §1 (blind 8cf9050 A: the 8-sample previews swing between reads): the paired read's own noise —
    /// the 95 % half-width of `d_past` over the paired sends (1.96 · sd of the per-send differences / √n) —
    /// and `even`: the move's better and worse sends are within chance of each other (a sign test,
    /// |better − worse| ≤ 1.96 · √(better + worse)) and `d_past` within its band. An even move shows no
    /// delta (`even`), never a sign that a re-read could flip.
    #[serde(default)]
    pub noise: f64,
    #[serde(default)]
    pub even: bool,
}

/// Cut 117 §1: (noise, even) of a paired read — `PkgOption.noise` / `even` — from the two panels' outcome
/// ranks (`outcome_rank`) and the record they are past.
pub fn paired_noise(base: &[u32], with: &[u32], best: u32) -> (f64, bool) {
    let n = base.len().min(with.len());
    if n == 0 {
        return (0.0, true);
    }
    let past = |r: u32| -> f64 { if r / 4 > best { 1.0 } else { 0.0 } };
    let d: Vec<f64> = (0..n).map(|i| past(with[i]) - past(base[i])).collect();
    let mean = d.iter().sum::<f64>() / n as f64;
    let var = if n > 1 { d.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64 } else { 0.0 };
    let noise = 1.96 * var.sqrt() / (n as f64).sqrt();
    let (_, better, worse) = paired(base, with);
    let sign = (better as f64 - worse as f64).abs() <= 1.96 * ((better + worse) as f64).sqrt();
    (noise, sign && mean.abs() <= noise.max(1e-9))
}

/// Cut 115 §3: a move at one wall — the sends from the waystone under it, paired against the worn set's on the
/// same seeds: how many the move ended better (past the wall rather than under it, or as far and a better exit)
/// and how many worse.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct WallRead {
    pub depth: u32,
    /// The wall's boss (`Warlord`).
    pub boss: String,
    pub n: u32,
    pub better: u32,
    pub worse: u32,
}

/// Walls read per move at most (the deepest first).
pub const WALL_READS: usize = 3;

/// The walls a lineage's compare reads: each lit waystone's next band boss at or under the record's next floor,
/// deepest first — (stone, wall).
pub fn compare_walls(l: &LineageState) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = l.stones().into_iter().filter_map(|s| {
        let wall = crate::descent::BOSS_DEPTHS.iter().map(|(_, d)| *d).find(|d| *d >= s)?;
        (wall <= l.best_depth + 1).then_some((s, wall))
    }).collect();
    out.sort_by_key(|x| std::cmp::Reverse(x.1));
    out.dedup_by_key(|x| x.1);
    out.truncate(WALL_READS);
    out
}

/// A set's sends from `stone` ranked at `wall` (past it first, then the exit).
fn wall_ranks(g: &crate::engine::Game, set: &RuleSet, stone: u32, wall: u32, sims: u32) -> Vec<u32> {
    use crate::engine::ExitTier;
    let mut w = g.sim_clone();
    w.lineage.start = stone;
    crate::forecast::camp_panel_outcomes(&w, set, sims).iter().map(|r| r.max_depth.min(wall + 1) * 4 + match r.tier { ExitTier::Death => 0, ExitTier::Bank => 2, _ => 1 }).collect()
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
        if !available(l, d.id) {
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

/// Make a move (`equip` / `level` / `variant`: a worn tactic's variant `slot`).
pub fn apply(l: &mut LineageState, id: &str, action: &str, slot: usize) -> Result<(), String> {
    match action {
        "level" => spend_level(l, id).map(|_| ()),
        "variant" => set_variant(l, id, slot.min(255) as u8),
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
/// One send's outcome as the paired comparison orders it: the floor reached first, then how it
/// ended (died < turned back < full haul) — the camp's sends play the same seeds, so a move's
/// send `i` and the worn set's send `i` met the same dungeon (`forecast_tag`).
fn outcome_rank(r: &crate::forecast::SimResult) -> u32 {
    use crate::engine::ExitTier;
    r.max_depth * 4 + match r.tier { ExitTier::Death => 0, ExitTier::Bank => 2, _ => 1 }
}

type Panels = ((f64, f64, f64, f64, f64), Option<f64>, Vec<u32>);

fn panels(g: &crate::engine::Game, set: &RuleSet, sims: u32, stone: Option<u32>) -> Panels {
    if sims < crate::forecast::REFINE_SIMS {
        // Ordinary Tactics reads depths and exit tiers, never skipped-floor gold. Skip that
        // independent ledger forecast and isolate its incomplete panel cache.
        // Explicit refinement retains its existing serialized camp-quality state.
        let rs = crate::forecast::camp_panel_outcomes(g, set, sims);
        let own = read_shares(g, &rs);
        let wall = stone.map(|s| {
            let mut w = g.sim_clone();
            w.lineage.start = s;
            read_shares(&w, &crate::forecast::camp_panel_outcomes(&w, set, sims)).0
        });
        return (own, wall, rs.iter().map(outcome_rank).collect());
    }

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
    let panel = if wall_start > 1 && start > 1 && start < wall_start && !cached {
        // (the deeper passage first: the shallower one is its sims cut, then the panel reads it)
        let w = wall.as_ref().expect("a wall start");
        let d1 = passage_run(w, set, wall_start);
        passage_from(w, set, wall_start, &d1);
        passage_from(g, set, start, &d1);
        camp_panel(g, set, sims)
    } else {
        let panel = camp_panel(g, set, sims);
        if let Some(w) = wall.as_ref().filter(|_| wall_start > 1 && g.lineage.start == 1 && !cached) {
            passage_from(w, set, wall_start, &panel);
        }
        panel
    };
    (read_shares(g, &panel), wall.map(|w| shares(&w, set, sims).0), panel.iter().map(outcome_rank).collect())
}

/// The camp's package prices (`sims` sends each, on the camp's seeds), best move first — read twice:
/// from where the sends start (the walk and the night), and at the wall (`d_wall`: the share past the
/// record from the deepest lit waystone at or above it, the wall's own panel as `wall::search` reads it).
/// A move that answers the wall reads there, under the noise of the floors above it; one that only helps
/// at the frontier while the walk to it suffers reads on the first panel. (Read from the stone alone, a
/// stance that walks home early looked best.)
fn option_panel_key(g: &crate::engine::Game, set: &RuleSet, sims: u32, stone: Option<u32>) -> (String, Option<String>, String) {
    let own = crate::forecast::panel_key(g, set, sims);
    let wall = stone.map(|s| {
        let mut w = g.sim_clone();
        w.lineage.start = s;
        crate::forecast::panel_key(&w, set, sims)
    });
    (own, wall, crate::forecast::rules_key(set))
}

/// Complete package-query inputs, including the ordered choices and their purchase prices.
/// The caller must read this and the prices from the same unchanged game.
pub fn options_key(g: &crate::engine::Game, sims: u32) -> String {
    options_key_from(g, sims, candidates(&g.lineage))
}

/// The chooser's moves: equips named `(id, slot)`, and (Cut 115 §3) a worn tactic's other variant named
/// `(id#v, v)` — `corridor_fighting#1` wears `at two`.
fn selected_candidates(g: &crate::engine::Game, choices: &[(String, usize)]) -> Vec<(String, String, usize)> {
    let mut out: Vec<(String, String, usize)> = candidates(&g.lineage).into_iter().filter(|(id, action, slot)| action == "equip" && choices.iter().any(|(want, at)| want == id && at == slot)).collect();
    let p = &g.lineage.pkg;
    for (want, _) in choices {
        let Some((id, v)) = want.split_once('#') else { continue };
        let Ok(v) = v.parse::<usize>() else { continue };
        let worn = p.tactics.iter().any(|t| t == id);
        let now = p.variants.get(id).copied().unwrap_or(0) as usize;
        if worn && variants(id).is_some() && v <= 1 && v != now && !out.iter().any(|m| m.0 == id && m.1 == "variant") {
            out.push((id.to_string(), "variant".to_string(), v));
        }
    }
    out
}

pub fn options_for_key(g: &crate::engine::Game, sims: u32, choices: &[(String, usize)]) -> String {
    options_key_from(g, sims, selected_candidates(g, choices))
}

fn options_key_from(g: &crate::engine::Game, sims: u32, candidates: Vec<(String, String, usize)>) -> String {
    let best = g.lineage.best_depth;
    let stone = g.lineage.stones().into_iter().filter(|w| *w <= best && *w > g.lineage.start.max(1)).max();
    let mut moves = Vec::new();
    for (id, action, slot) in candidates {
        let mut c = g.sim_clone();
        if apply(&mut c.lineage, &id, &action, slot).is_err() { continue; }
        let price = if action == "level" { level_price(&g.lineage, &id).unwrap_or(0) } else { 0 };
        moves.push((id, action, slot, price, option_panel_key(&c, &compile(&c.lineage), sims, stone)));
    }
    serde_json::to_string(&(best, crate::forecast::sim_width(), crate::forecast::parallel_sims(), crate::balance::get(), option_panel_key(g, g.lineage.rules(), sims, stone), moves)).expect("package query key")
}

pub fn options(g: &crate::engine::Game, sims: u32) -> Vec<PkgOption> {
    options_from(g, sims, candidates(&g.lineage))
}

/// Equip previews for explicitly selected legal choices; an empty selection runs no panels.
pub fn options_for(g: &crate::engine::Game, sims: u32, choices: &[(String, usize)]) -> Vec<PkgOption> {
    let moves = selected_candidates(g, choices);
    if moves.is_empty() { return Vec::new(); }
    let mut out = options_from(g, sims, moves);
    // Cut 115 §3: each move read at the walls the lineage has met (the worn set's sends once per wall)
    let walls = compare_walls(&g.lineage);
    if !walls.is_empty() {
        let base: Vec<Vec<u32>> = walls.iter().map(|(s, w)| wall_ranks(g, g.lineage.rules(), *s, *w, sims)).collect();
        for o in out.iter_mut() {
            let mut c = g.sim_clone();
            if apply(&mut c.lineage, &o.id, &o.action, o.slot).is_err() { continue; }
            let set = compile(&c.lineage);
            o.walls = walls.iter().zip(&base).map(|((s, w), b)| {
                let (n, better, worse) = paired(b, &wall_ranks(&c, &set, *s, *w, sims));
                WallRead { depth: *w, boss: crate::descent::boss_for(*w).map(crate::sifter::boss_short).unwrap_or("boss").into(), n, better, worse }
            }).collect();
        }
    }
    out
}

fn options_from(g: &crate::engine::Game, sims: u32, moves: Vec<(String, String, usize)>) -> Vec<PkgOption> {
    let best = g.lineage.best_depth;
    let stone = g.lineage.stones().into_iter().filter(|w| *w <= best && *w > g.lineage.start.max(1)).max();
    let (base, wall_base, base_ranks) = panels(g, g.lineage.rules(), sims, stone);
    // Preserve the complete query's native panel execution policy for exact subset answers.
    let full_count = candidates(&g.lineage).len();
    let threaded = full_count > 1 && 2 * full_count >= crate::forecast::sim_width();
    let key = |g: &crate::engine::Game, set: &RuleSet| option_panel_key(g, set, sims, stone);
    let mut groups = std::collections::BTreeMap::new();
    groups.insert(key(g, g.lineage.rules()), 0usize);
    let mut jobs = Vec::new();
    let mut plan = Vec::new();
    for m in &moves {
        let mut c = g.sim_clone();
        if apply(&mut c.lineage, &m.0, &m.1, m.2).is_err() { continue; }
        let set = compile(&c.lineage);
        let k = key(&c, &set);
        let group = *groups.entry(k).or_insert_with(|| { jobs.push(m.clone()); jobs.len() });
        plan.push((m.clone(), group));
    }
    // Preserve worker-local sequential budget execution when only one distinct job remains.
    if threaded && crate::forecast::parallel_sims() && jobs.len() <= 1 {
        jobs = moves.clone();
        plan = moves.iter().cloned().enumerate().map(|(i, m)| (m, i + 1)).collect();
    }
    let one = |g: &crate::engine::Game, (id, action, slot): &(String, String, usize)| {
        let mut c = g.sim_clone();
        apply(&mut c.lineage, id, action, *slot).ok()?;
        let set = compile(&c.lineage);
        Some(panels(&c, &set, sims, stone))
    };
    let mut measured = vec![Some((base, wall_base, base_ranks.clone()))];
    measured.extend(if threaded {
        crate::forecast::par_map(g, jobs, one)
    } else { jobs.iter().map(|m| one(g, m)).collect() });
    let mut out: Vec<PkgOption> = plan.into_iter().filter_map(|((id, action, slot), group)| {
        let ((past, bank, death, reach, mean), wall, ranks) = measured[group].clone()?;
        let d_wall = match (wall_base, wall) { (Some(a), Some(b)) => b - a, _ => 0.0 };
        let price = if action == "level" { level_price(&g.lineage, &id).unwrap_or(0) } else { 0 };
        let (n, better, worse) = paired(&base_ranks, &ranks);
        let (noise, even) = paired_noise(&base_ranks, &ranks, best);
        Some(PkgOption { id, action, slot, price, past, bank, death, reach, mean, d_past: past - base.0, d_bank: bank - base.1, d_death: death - base.2, d_reach: reach - base.3, d_mean: mean - base.4, d_wall, n, better, worse, walls: Vec::new(), noise, even })
    }).collect();
    out.sort_by(|a, b| score(b).total_cmp(&score(a)));
    out
}

/// Blind c4705f9 (A, B: `compare outcomes` read `all similar` nearly always): the paired read of a
/// move — of the sends both panels ran on the same seeds (`n`), how many the move ended better
/// (deeper, or as deep and a better exit) and how many worse. The shares' bands treated the two
/// panels as independent draws of five; the pairs are the resolution the panel actually has.
pub fn paired(base: &[u32], with: &[u32]) -> (u32, u32, u32) {
    let n = base.len().min(with.len());
    let better = (0..n).filter(|&i| with[i] > base[i]).count() as u32;
    let worse = (0..n).filter(|&i| with[i] < base[i]).count() as u32;
    (n as u32, better, worse)
}

/// A move's worth: the sends past the record, then those reaching it and banked, the floors the
/// sends reach (a tenth a floor: the long walk from D1 the record sits under), less those that die.
pub fn score(o: &PkgOption) -> f64 {
    o.d_past + 0.3 * o.d_reach + 0.2 * o.d_bank - 0.2 * o.d_death + 0.1 * o.d_mean + o.d_wall
}
