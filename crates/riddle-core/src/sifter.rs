//! Cut 5 §1: the sifter, rewritten around episodes. An episode is a span of a run with a low
//! point and a resolution; `Run.arc` tracks the live one (hp low-water since the last
//! resolution, the row that fired at it, the threat present, items used, allies lost) and each
//! closed episode becomes one story line of three beats, ≤ 12 words, from the tables below:
//!
//! `Two jackals took him to 3 HP; R2 drank; banked $58.`
//! `The Warlord took him to 9 HP; R4 bashed him; first boss.`
//! `The nest took him to 5 HP; greed took the gold; jackal Uleth fell.`
//!
//! The reel is the top three episodes of an absence by score (low-point depth × resolution
//! weight) plus the best-depth run's closing episode, never two with the same (threat,
//! resolution). §3: the hero's voice, a trait × moment table, is here too.
use crate::engine::{kind_title, Ctx, LineageState, Run};
use crate::hero::Trait;
use crate::rules::{word_count, Verb};
use crate::wire::{Highlight, HighlightArc};
use serde::{Deserialize, Serialize};

/// A story line's word budget.
pub const STORY_WORDS: usize = 12;
/// A low at or under this share of max HP is a low point; recovering past `RECOVER_PCT`
/// afterwards seals the episode (its resolution is the next one the run reaches).
pub const LOW_PCT: i32 = 25;
/// Cut 28 §4: a story's `took him to N HP` needs N at or under this share of his max.
pub const HURT_PCT: i32 = 50;
/// Cut 28 §4 (AV: the same line four runs running): a reel holds a line's shape (its numbers aside)
/// at most this many times.
pub const SHAPE_MAX: usize = 2;

/// A reel line's shape: its text with every number as `N` (the client's `reelShape`).
pub fn shape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut num = false;
    for c in text.chars() {
        if c.is_ascii_digit() {
            if !num {
                out.push('N');
            }
            num = true;
        } else {
            num = false;
            out.push(c);
        }
    }
    out
}
pub const RECOVER_PCT: i32 = 60;
/// Sealed episodes waiting for a resolution (the deepest lows are kept).
pub const SEALED_MAX: usize = 2;
/// Cut 2 §2: recovering a named heir's bones.
pub const BONES: i32 = 6;
/// §3: the hero speaks at most once per this many ticks, never in a fight's first ten.
pub const VOICE_EVERY: u32 = 100;
pub const VOICE_FIGHT_QUIET: u32 = 10;

/// The hero action recorded for an episode (row −1 trait, −2 chores).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Act {
    pub row: i32,
    pub verb: Verb,
    /// The kind the verb acted on, and whether it was a boss (`R4 bashed him`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(default)]
    pub boss: bool,
    /// QA on a946e04 (qaS: `A goblin took him to 3 HP; no row; died to gas.` under R2 `hp <
    /// 40% → return`): the walk home a `return` / `bank` row committed to (`Run.homeward`) —
    /// `row` is that row and the beat reads `R2 returning`, never `no row`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub walk: bool,
}

impl Default for Act {
    /// No action recorded: the chores' `no row fired`.
    fn default() -> Act {
        Act { row: -2, verb: Verb::new("wait"), target: None, boss: false, walk: false }
    }
}

/// The setup beat's shape.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Setup {
    /// `<threat> took|cornered|chased him to N HP` (`took him down` at 0).
    #[default]
    Hurt,
    /// `The vault held a mail`.
    Vault,
    /// `Uleth the jackal came back`.
    Stray,
    /// `Untouched` / `Untouched by the Warlord`.
    Untouched,
    /// Cut 7 §3: `A captive, chained` (the gate answered by `free_captive`).
    Captive,
}

/// The end beat. `Pending` is a sealed episode waiting for the run's next resolution.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Resolution {
    #[default]
    Pending,
    Banked {
        gold: i32,
    },
    Reached {
        depth: u32,
    },
    FirstBoss {
        kind: String,
    },
    BossSlain {
        kind: String,
    },
    Returned {
        /// Cut 12 §4: the gold brought home (the routine line reads `returned $54`).
        #[serde(default)]
        gold: i32,
    },
    /// The run hit the turn cap (`lost the thread`). `stalled` is Cut 7's flag, kept for
    /// saved episodes; since Cut 13 §1 a stall resolves as `Stalled` below.
    Lost {
        #[serde(default)]
        stalled: bool,
    },
    /// Cut 13 §1: the run shuffled on one floor; `cause` is the guard's moment (`goblin
    /// archer, no path` · `paced`) — the same words as the stall record's trace.
    Stalled {
        cause: String,
    },
    Died {
        cause: String,
    },
    /// Cut 24 §1: a boss that could not be hurt drove him off his floor (a return).
    DrivenOff {
        kind: String,
    },
    Fell {
        kind: String,
        name: String,
    },
    /// Cut 14 (QA on 56f2a1d): a sealed low point the hero walked away from, in a run that
    /// later died — `An ogre took him to 1 HP; R3 attacked; lived.` The death is its own
    /// episode's; two `died to` lines for one death made the reel disagree with the tally.
    Survived,
}

impl Resolution {
    /// Boss kills and a companion's fall close the live episode only; floors and exits
    /// resolve the sealed ones too.
    pub fn live_only(&self) -> bool {
        matches!(self, Resolution::FirstBoss { .. } | Resolution::BossSlain { .. } | Resolution::Fell { .. })
    }
    pub fn is_exit(&self) -> bool {
        matches!(self, Resolution::Banked { .. } | Resolution::Returned { .. } | Resolution::Lost { .. } | Resolution::Stalled { .. } | Resolution::Died { .. } | Resolution::DrivenOff { .. })
    }
}

/// A closed (or sealed) episode.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Episode {
    pub t: u32,
    pub depth: u32,
    pub setup: Setup,
    pub low_hp: i32,
    pub max_hp: i32,
    /// Foe kinds present at the low point, the boss first, then the most numerous; a hazard
    /// (`gas`) or a situation (`shrine`, `nest`, `stray`, `vault`) when no foe was.
    pub threat: Vec<(String, u32)>,
    pub cornered: bool,
    pub chased: bool,
    pub act: Act,
    pub trait_: Trait,
    /// `shrine | vault | nest | stray` when a situation shaped the episode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub situation: Option<String>,
    /// The vault item's label (`a mail`), the stray's kind.
    #[serde(default)]
    pub detail: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub items_used: Vec<String>,
    #[serde(default)]
    pub allies_lost: Vec<String>,
    pub resolution: Resolution,
    /// Cut 8B §1: a combo both of whose rows fired within three hero actions of the low point
    /// (`the bait landed` is the turn beat instead of the single row).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combo: Option<String>,
    /// Cut 12 §4: the floor's one situation (`nest`) when the episode closed, for the routine
    /// line (`D6, the nest: returned $54.`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub twist: Option<String>,
}

/// The live episode (`Run.arc`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Arc {
    pub start_t: u32,
    /// (hp, tick) of the low-water mark since the last resolution.
    pub low: Option<(i32, u32)>,
    pub max_hp: i32,
    pub threat: Vec<(String, u32)>,
    pub cause: String,
    pub cornered: bool,
    pub chased: bool,
    /// The first action after the low point, and whether it is still awaited.
    pub row: Option<crate::shared::Shared<Act>>,
    pub row_pending: bool,
    /// The last hero action (the row when the episode has no low).
    pub last: Option<crate::shared::Shared<Act>>,
    pub items_used: Vec<String>,
    pub allies_lost: Vec<String>,
    pub situation: Option<String>,
    pub detail: String,
    pub voiced_low: bool,
    /// Sealed lows awaiting a resolution.
    pub sealed: Vec<Episode>,
    /// Cut 8B §1: the last few hero actions as (action number, act), the action number of the
    /// low point's act, and the combo credited once two adjacent rows fired around it.
    #[serde(default)]
    pub acts: Vec<(u32, crate::shared::Shared<Act>)>,
    #[serde(default)]
    pub low_act: Option<u32>,
    #[serde(default)]
    pub combo: Option<String>,
}

/// Cut 8B §1: hero actions kept for combo credit (the low's act and its neighbours).
const COMBO_ACTS: usize = 4;

impl Arc {
    pub fn has_low(&self) -> bool {
        self.low.is_some()
    }
    fn low_pct(&self) -> i32 {
        match self.low {
            Some((hp, _)) => hp * 100 / self.max_hp.max(1),
            None => 100,
        }
    }
    fn reset(&mut self, t: u32) {
        let sealed = std::mem::take(&mut self.sealed);
        *self = Arc { start_t: t, sealed, ..Arc::default() };
    }
    pub fn to_episode(&self, run: &Run, res: Resolution) -> Episode {
        // Cut 28 §4 (AV: `A goblin took him to 40 HP` — his max, no low at all): a low above
        // `HURT_PCT` of his max is no low point — the line is the run's routine one.
        let (low_hp, setup) = match self.low {
            Some((hp, _)) if hp * 100 <= HURT_PCT * self.max_hp.max(1) => (hp, Setup::Hurt),
            Some(_) => (run.hero.hp, Setup::Untouched),
            None => (run.hero.hp, Setup::Untouched),
        };
        Episode {
            t: run.turn,
            depth: run.depth,
            setup,
            low_hp,
            max_hp: if self.low.is_some() { self.max_hp } else { run.hero.max_hp },
            threat: self.threat.clone(),
            cornered: self.cornered,
            chased: self.chased,
            act: self.row.as_deref().or(self.last.as_deref()).cloned().unwrap_or_default(),
            trait_: run.trait_,
            situation: self.situation.clone(),
            detail: self.detail.clone(),
            name: String::new(),
            items_used: self.items_used.clone(),
            allies_lost: self.allies_lost.clone(),
            resolution: res,
            combo: self.combo.clone(),
            twist: (*run.floor_twist).clone(),
        }
    }
    /// Cut 8B §1: two adjacent rows (`a` then `b`, in row order) fired within the three hero
    /// actions around the low point's act — the combo the pair names.
    fn combo_around(&self) -> Option<String> {
        let k = self.low_act?;
        let lo = k.saturating_sub(1);
        let hi = k + 1;
        for (i, (ni, ai)) in self.acts.iter().enumerate() {
            if *ni < lo || ai.row < 0 || ai.walk {
                continue;
            }
            for (nj, aj) in self.acts.iter().skip(i + 1) {
                if *nj > hi || aj.row != ai.row + 1 || aj.walk {
                    continue;
                }
                if let Some(name) = crate::rules::combo_name(&ai.verb, &aj.verb) {
                    return Some(name.into());
                }
            }
        }
        None
    }
}

// ---------------------------------------------------------------- tracking hooks

/// Foe kinds in view, the boss first, then the most numerous, then by name.
pub fn threat_now(run: &Run) -> Vec<(String, u32)> {
    let map = &run.floor.map;
    let mut v: Vec<(String, u32)> = Vec::new();
    for m in run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && !m.dormant && map.is_visible(m.pos)) {
        let key = if m.nest { "nest".to_string() } else if m.stray { "stray".to_string() } else if let Some(s) = &m.situation { s.clone() } else { m.kind.clone() };
        match v.iter_mut().find(|(k, _)| *k == key) {
            Some(e) => e.1 += 1,
            None => v.push((key, 1)),
        }
    }
    v.sort_by(|a, b| {
        let boss = |k: &str| crate::defs::monster_def(k).boss && crate::defs::MONSTERS.iter().any(|m| m.kind == k);
        boss(&b.0).cmp(&boss(&a.0)).then(b.1.cmp(&a.1)).then(a.0.cmp(&b.0))
    });
    v
}

/// The hero was hurt (hp > 0 after the blow). A new low since the last resolution records the
/// threat, the cause and waits for the row that answers it. Returns true when this is the
/// first time the live arc fell to `LOW_PCT` (the voice's `low` moment).
pub fn on_hurt(run: &mut Run, cause: &str, flag: Option<&str>) -> bool {
    let hp = run.hero.hp;
    if run.arc.low.is_some_and(|(h, _)| hp >= h) {
        return false;
    }
    let mut threat = threat_now(run);
    let key = match flag {
        Some(f) => f.to_string(),
        None => cause.to_string(),
    };
    if threat.is_empty() {
        threat.push((key.clone(), 1));
    } else if let Some(i) = threat.iter().position(|(k, _)| *k == key) {
        // The kind that drew blood leads unless a boss is present.
        if i > 0 && !crate::defs::monster_def(&threat[0].0).boss {
            let e = threat.remove(i);
            threat.insert(0, e);
        }
    }
    let hp_pos = run.hero.pos;
    let adj = run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && m.pos.adjacent(hp_pos)).count();
    let fast = crate::defs::MONSTERS.iter().any(|m| m.kind == cause && m.tags.contains(&"fast"));
    let fleeing = run.arc.last.as_ref().is_some_and(|a| matches!(a.verb.v.as_str(), "retreat" | "back_corridor" | "kite" | "blink"));
    let a = &mut run.arc;
    a.low = Some((hp, run.turn));
    a.max_hp = run.hero.max_hp;
    a.threat = threat;
    a.cause = cause.to_string();
    a.cornered = adj >= 2;
    a.chased = fast || fleeing;
    a.row = None;
    a.row_pending = true;
    a.situation = match flag {
        Some(f) => Some(f.to_string()),
        None if cause == "shrine" => Some("shrine".into()),
        None => None,
    };
    if a.situation.is_some() {
        a.detail = cause.to_string();
    }
    let low = a.low_pct() <= LOW_PCT;
    if low && !a.voiced_low {
        a.voiced_low = true;
        return true;
    }
    false
}

/// The hero acted: record it (the row at the low point when one is awaited). Returns true
/// when the action recovered the hero past `RECOVER_PCT` after a low at `LOW_PCT` (the
/// episode is sealed; the voice's `resolved` moment).
pub fn on_action(run: &mut Run, row: i32, verb: &Verb) {
    let target = run.last_target.and_then(|id| run.monsters.iter().find(|m| m.id == id)).map(|m| (m.kind.clone(), m.is_boss()));
    // The walk home is the committing row's (the chores step it: `ai::chore`).
    let walk = row == -2 && matches!(verb.v.as_str(), "return" | "bank") && run.homeward.is_some_and(|h| h >= 0);
    let row = if walk { run.homeward.unwrap_or(row) } else { row };
    // One act, shared by the window, the low's row and `last` (and the history ring's copies).
    let act = crate::shared::Shared::new(Act { row, verb: verb.clone(), target: target.as_ref().map(|t| t.0.clone()), boss: target.is_some_and(|t| t.1), walk });
    let turn = run.turn;
    let actions = run.actions;
    let a = &mut run.arc;
    if a.row_pending {
        a.row = Some(act.clone());
        a.row_pending = false;
        a.low_act = Some(actions);
    }
    // Cut 8B §1: the act joins the window; a combo around the low point is credited once.
    a.acts.push((actions, act.clone()));
    if a.acts.len() > COMBO_ACTS {
        a.acts.remove(0);
    }
    if a.combo.is_none() && a.low_act.is_some_and(|k| actions <= k + 1) {
        a.combo = a.combo_around();
    }
    // A situation opened this action (a stray tamed) takes the action as its turn beat.
    for e in a.sealed.iter_mut().filter(|e| e.t == turn && e.setup == Setup::Stray && e.act == Act::default()) {
        e.act = (*act).clone();
    }
    a.last = Some(act);
}

/// Recovered past `RECOVER_PCT` after a low at `LOW_PCT`: the episode is sealed.
pub fn recovered(run: &Run) -> bool {
    run.arc.has_low() && run.arc.low_pct() <= LOW_PCT && run.hero.hp_pct() > RECOVER_PCT && !run.arc.row_pending
}

/// Seal the live episode (resolution pending) and start a fresh one.
pub fn seal(run: &mut Run) {
    if !run.arc.has_low() {
        return;
    }
    let ep = run.arc.to_episode(run, Resolution::Pending);
    push_sealed(run, ep);
    let t = run.turn;
    run.arc.reset(t);
}

fn push_sealed(run: &mut Run, ep: Episode) {
    run.arc.sealed.push(ep);
    if run.arc.sealed.len() > SEALED_MAX {
        // Keep the deepest lows.
        let pct = |e: &Episode| if e.setup == Setup::Hurt { e.low_hp * 100 / e.max_hp.max(1) } else { 50 };
        let i = (0..run.arc.sealed.len()).max_by_key(|&i| pct(&run.arc.sealed[i])).unwrap();
        run.arc.sealed.remove(i);
    }
}

/// A situation opened a low-less episode of its own (a vault chosen, a stray tamed).
pub fn open_situation(run: &mut Run, setup: Setup, situation: &str, detail: &str, name: &str) {
    let mut ep = run.arc.to_episode(run, Resolution::Pending);
    ep.setup = setup;
    ep.low_hp = run.hero.hp;
    ep.max_hp = run.hero.max_hp;
    ep.threat = vec![(situation.to_string(), 1)];
    ep.situation = Some(situation.to_string());
    ep.detail = detail.to_string();
    ep.name = name.to_string();
    ep.act = run.arc.last.as_deref().cloned().unwrap_or_default();
    push_sealed(run, ep);
}

/// A resolution reached: the live episode closes (and the sealed ones, unless the
/// resolution is a boss kill or a companion's fall). An exit with nothing to tell closes an
/// `Untouched` episode so every run has a closing line.
pub fn resolve(run: &mut Run, res: Resolution) {
    let mut out: Vec<Episode> = Vec::new();
    if !res.live_only() {
        for mut e in std::mem::take(&mut run.arc.sealed) {
            e.resolution = if matches!(res, Resolution::Died { .. }) { Resolution::Survived } else { res.clone() };
            e.t = run.turn;
            out.push(e);
        }
    }
    let always = matches!(res, Resolution::Died { .. } | Resolution::Fell { .. } | Resolution::FirstBoss { .. } | Resolution::BossSlain { .. });
    if run.arc.has_low() || always || (res.is_exit() && out.is_empty()) {
        let mut e = run.arc.to_episode(run, res.clone());
        // A boss or a killer names the threat when nothing has drawn blood yet.
        if e.threat.is_empty() {
            e.threat = match &res {
                Resolution::FirstBoss { kind } | Resolution::BossSlain { kind } => vec![(kind.clone(), 1)],
                Resolution::Died { cause } => vec![(cause.clone(), 1)],
                _ => threat_now(run),
            };
        }
        if let Resolution::Died { cause } = &res {
            if e.setup == Setup::Untouched {
                // One blow from full health.
                e.setup = Setup::Hurt;
                e.low_hp = 0;
                // `X took him down` names the killer (QA on 1a2a4a9: the threat in view led
                // — `A goblin took him down; …; died on D8` — whoever landed the blow).
                match e.threat.iter().position(|(k, _)| cause_key(k) == cause_key(cause)) {
                    Some(i) => {
                        let k = e.threat.remove(i);
                        e.threat.insert(0, k);
                    }
                    None => e.threat.insert(0, (cause.clone(), 1)),
                }
            }
        }
        if matches!(res, Resolution::FirstBoss { .. } | Resolution::BossSlain { .. }) {
            // The boss leads its own episode.
            let kind = match &res {
                Resolution::FirstBoss { kind } | Resolution::BossSlain { kind } => kind.clone(),
                _ => String::new(),
            };
            if let Some(i) = e.threat.iter().position(|(k, _)| *k == kind) {
                let b = e.threat.remove(i);
                e.threat.insert(0, b);
            } else {
                e.threat.insert(0, (kind, 1));
            }
        }
        out.push(e);
    }
    let t = run.turn;
    run.arc.reset(t);
    run.episodes.extend(out);
}

// ---------------------------------------------------------------- the grammar

/// Verb → past tense (every hero verb, chore and companion verb; cards below).
pub const PAST: &[(&str, &str)] = &[
    ("attack", "attacked"),
    ("retreat", "retreated"),
    ("back_corridor", "took the corridor"),
    ("drink", "drank"),
    ("read", "read"),
    ("throw", "threw"),
    ("descend", "went down"),
    ("bank", "banked"),
    ("return", "returned"),
    ("rest", "rested"),
    ("pick_up", "picked up"),
    ("free_captive", "freed the captive"),
    ("shield_bash", "bashed"),
    ("vanish", "vanished"),
    ("tame", "tamed"),
    ("recall", "recalled"),
    ("send", "sent the pack"),
    ("shoot", "shot"),
    ("burst", "burst"),
    ("steal", "stole"),
    ("split", "split"),
    ("flank", "flanked"),
    ("drain", "drained"),
    ("follow", "followed"),
    ("cleave", "cleaved"),
    ("taunt", "taunted"),
    ("second_wind", "caught breath"),
    ("bulwark", "raised the bulwark"),
    ("backstab", "backstabbed"),
    ("smoke", "smoked"),
    ("ambush", "ambushed"),
    ("shadowstep", "shadowstepped"),
    ("kite", "kited"),
    ("volley", "loosed a volley"),
    ("trap", "set a trap"),
    ("mark", "marked"),
    ("double_shot", "double-shot"),
    ("bolt", "bolted"),
    ("ward", "warded"),
    ("blink", "blinked"),
    ("slow", "slowed"),
    ("nova", "cast nova"),
    ("hold", "held"),
    ("pray", "prayed"),
    ("mimic", "mimicked"),
    // chores
    ("explore", "explored"),
    ("wait", "waited"),
    ("shuffle", "shuffled"),
    ("paralysed", "froze"),
    ("stumble", "stumbled"),
    ("stuck", "stalled"),
    ("cornered", "stood"),
];

/// Tactic card → past tense.
pub const CARD_PAST: &[(&str, &str)] = &[
    ("corridor_fighting", "held the corridor"),
    ("kite_archers", "kited"),
    ("stair_dance", "danced the stairs"),
    ("gas_step", "stepped clear"),
    ("pack_break", "broke the pack"),
    ("thief_guard", "guarded the pack"),
    ("boss_focus", "aimed"),
    ("last_stand", "stood"),
    ("cadence", "changed cadence"),
    ("noise_discipline", "kept quiet"),
    ("reflect_read", "read the mirror"),
    ("deep_march", "marched"),
    ("phalanx", "held the line"),
    ("hit_and_fade", "hit and faded"),
    ("hawkeye", "aimed"),
    ("archmage", "cast"),
];

/// Short past forms for the multi-word ones (the row beat's last resort).
pub const PAST_SHORT: &[(&str, &str)] = &[
    ("back_corridor", "backed"),
    ("descend", "descended"),
    ("pick_up", "looted"),
    ("free_captive", "freed"),
    ("send", "sent"),
    ("second_wind", "breathed"),
    ("bulwark", "braced"),
    ("volley", "volleyed"),
    ("trap", "trapped"),
    ("nova", "cast"),
    ("corridor_fighting", "held"),
    ("stair_dance", "danced"),
    ("gas_step", "sidestepped"),
    ("pack_break", "broke"),
    ("thief_guard", "guarded"),
    ("cadence", "varied"),
    ("noise_discipline", "hushed"),
    ("reflect_read", "read"),
    ("phalanx", "held"),
    ("hit_and_fade", "faded"),
];

/// Trait phrases (row −1): the trait as a noun and what it did.
pub const TRAIT_PAST: &[&str] = &["took the gold", "grabbed", "ran", "held", "drank", "read", "tried it"];
/// Chore beats that name no verb.
pub const NO_ROW: &[&str] = &["no row fired", "paralysed, no row", "confused, no row", NO_ROW_SHORT[0], NO_ROW_SHORT[1], NO_ROW_SHORT[2]];

pub fn past_tense(verb: &Verb) -> String {
    past_tense_form(verb, false)
}

/// `short`: the one-word form of a multi-word past (`held` for `held the corridor`).
pub fn past_tense_form(verb: &Verb, short: bool) -> String {
    let key = if verb.v == "tactic" { verb.a.as_deref().unwrap_or("") } else { verb.v.as_str() };
    if short {
        if let Some((_, p)) = PAST_SHORT.iter().find(|(k, _)| *k == key) {
            return p.to_string();
        }
    }
    let table: &[(&str, &str)] = if verb.v == "tactic" { CARD_PAST } else { PAST };
    table.iter().find(|(k, _)| *k == key).map(|(_, p)| p.to_string()).unwrap_or_else(|| "acted".into())
}

/// Every past-tense form the turn beat may carry (the gate's table).
pub fn past_forms() -> Vec<&'static str> {
    let mut v: Vec<&str> = PAST.iter().map(|(_, p)| *p).collect();
    v.extend(CARD_PAST.iter().map(|(_, p)| *p));
    v.extend(PAST_SHORT.iter().map(|(_, p)| *p));
    v.extend(TRAIT_PAST);
    v.push("acted");
    // (the walk home: `R2 returning`)
    v.push("returning");
    v.push("banking");
    v
}

pub fn boss_short(kind: &str) -> &'static str {
    match kind {
        "goblin_warlord" => "Warlord",
        "bloat_mother" => "Mother",
        "lich" => "Lich",
        "foundry_master" => "Master",
        "lurker_queen" => "Queen",
        "mirror_king" => "King",
        _ => "boss",
    }
}

fn is_boss(kind: &str) -> bool {
    crate::defs::MONSTERS.iter().any(|m| m.kind == kind && m.boss)
}

fn is_monster(kind: &str) -> bool {
    crate::defs::MONSTERS.iter().any(|m| m.kind == kind)
}

pub fn plural(title: &str) -> String {
    let (head, last) = match title.rfind(' ') {
        Some(i) => (&title[..=i], &title[i + 1..]),
        None => ("", title),
    };
    let p = if last.ends_with('y') && !last.ends_with("ey") && !last.ends_with("ay") {
        format!("{}ies", &last[..last.len() - 1])
    } else if last.ends_with('s') || last.ends_with('x') || last.ends_with("ch") || last.ends_with("sh") {
        format!("{last}es")
    } else {
        format!("{last}s")
    };
    format!("{head}{p}")
}

fn article(title: &str) -> &'static str {
    match title.chars().next().map(|c| c.to_ascii_lowercase()) {
        Some('a' | 'e' | 'i' | 'o' | 'u') => "an",
        _ => "a",
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn number_word(n: u32) -> &'static str {
    match n {
        2 => "Two",
        3 => "Three",
        4 => "Four",
        5 => "Five",
        6 => "Six",
        7 => "Seven",
        8 => "Eight",
        9 => "Nine",
        _ => "Many",
    }
}

/// `gas` · `the Warlord` · `a jackal` · `an ogre` — how a cause reads after `died to` or
/// `fell to`.
pub fn cause_phrase(cause: &str) -> String {
    match cause {
        "gas" | "burst" => "gas".into(),
        "fire" | "poison" => cause.into(),
        "shrine" => "the shrine".into(),
        "nest" => "the nest".into(),
        "stray" => "a stray".into(),
        "den" => "the den".into(),
        // QA on 912e135 (qaW: `The lock took him to 2 HP` — "the lock is not a foe in LEARNED or DEATHS"): the lock is its bloats;
        // QA on 524827b (qaAA: `Lock bloats` — "named nowhere else"): by the name LEARNED and the bestiary give them, `bloat`
        "lock" => "a bloat".into(),
        "captive" => "the captive".into(),
        "hunger" => "the hunger".into(),
        k if is_boss(k) => format!("the {}", boss_short(k)),
        k if is_monster(k) => {
            let t = kind_title(k);
            format!("{} {t}", article(&t))
        }
        other => other.replace('_', " "),
    }
}

/// The threat as a subject: `Two jackals` · `The Warlord` · `Gas` · `The nest`.
fn subject(ep: &Episode, short: bool) -> String {
    let Some((kind, n)) = ep.threat.first() else { return "Something".into() };
    let n = *n;
    match kind.as_str() {
        "gas" | "burst" => "Gas".into(),
        "fire" => "Fire".into(),
        "poison" => "Poison".into(),
        "shrine" => "The shrine".into(),
        "nest" => "The nest".into(),
        "vault" => "The cage".into(),
        "den" => "The den".into(),
        "lock" => if n == 1 { "A bloat".into() } else { "Bloats".into() },
        "captive" => "The captive".into(),
        "hunger" => "The hunger".into(),
        "stray" => {
            if short {
                "The stray".into()
            } else {
                format!("A stray {}", kind_title(&ep.detail))
            }
        }
        "none" => "Nothing".into(),
        k if is_boss(k) => {
            if short {
                boss_short(k).into()
            } else {
                format!("The {}", boss_short(k))
            }
        }
        k => {
            let t = kind_title(k);
            if n == 1 {
                // Short: the title's last word (`An archer` for a goblin archer).
                let t = if short { t.rsplit(' ').next().unwrap_or(&t).to_string() } else { t };
                format!("{} {t}", capitalize(article(&t)))
            } else if short {
                capitalize(&plural(&t))
            } else {
                format!("{} {}", number_word(n), plural(&t))
            }
        }
    }
}

fn setup_phrase(ep: &Episode, short: bool) -> String {
    match ep.setup {
        Setup::Vault => "The cage held three".into(),
        Setup::Captive => "A captive, chained".into(),
        Setup::Stray => {
            if short {
                format!("{} came back", ep.name)
            } else {
                format!("{} the {} came back", ep.name, kind_title(&ep.detail))
            }
        }
        Setup::Untouched => {
            if ep.threat.is_empty() || short {
                "Untouched".into()
            } else {
                format!("Untouched by {}", lower_first(&subject(ep, false)))
            }
        }
        Setup::Hurt => {
            let subj = subject(ep, short);
            if ep.low_hp <= 0 {
                format!("{subj} took him down")
            } else {
                let v = if ep.cornered {
                    "cornered"
                } else if ep.chased {
                    "chased"
                } else {
                    "took"
                };
                format!("{subj} {v} him to {} HP", ep.low_hp)
            }
        }
    }
}

/// `A jackal` → `a jackal` (the subject's leading article or number word).
fn lower_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn turn_phrase(ep: &Episode, short: bool) -> String {
    if ep.setup == Setup::Vault {
        return format!("he took the {}", ep.detail);
    }
    let a = &ep.act;
    // Cut 8B §1: a combo that landed at the low point is the turn beat.
    if let Some(c) = ep.combo.as_deref().filter(|_| a.row >= 0) {
        // Short: one word (`the hit-and-fade landed`).
        return if short { format!("the {} landed", crate::rules::combo_slug(c)) } else { format!("the {c} landed") };
    }
    let past = past_tense_form(&a.verb, short);
    if a.walk && a.row >= 0 {
        return format!("R{} {}", a.row + 1, if a.verb.v == "bank" { "banking" } else { "returning" });
    }
    if a.row >= 0 {
        let base = format!("R{} {past}", a.row + 1);
        if !short && a.boss && crate::turn::targets_foes(&a.verb) {
            return format!("{base} him");
        }
        return base;
    }
    if a.row == -1 {
        return match (ep.trait_, a.verb.v.as_str()) {
            (Trait::Greedy, "pick_up") => if short { "greed grabbed" } else { "greed took the gold" }.into(),
            (Trait::Greedy, _) => format!("greed {past}"),
            (Trait::Cowardly, "retreat") => "cowardice ran".into(),
            (Trait::Cowardly, _) => format!("cowardice {past}"),
            (Trait::Brave, "attack" | "hold") => "bravery held".into(),
            (Trait::Brave, _) => format!("bravery {past}"),
            (Trait::Curious, "drink") => "curiosity drank".into(),
            (Trait::Curious, "read") => "curiosity read".into(),
            (Trait::Curious, _) => format!("curiosity {past}"),
        };
    }
    match a.verb.v.as_str() {
        "wait" | "cornered" | "stuck" | "shuffle" | "" => "no row fired".into(),
        "paralysed" => "paralysed, no row".into(),
        "stumble" => "confused, no row".into(),
        _ if short => "no row fired".into(),
        _ => format!("the chores {past}"),
    }
}

pub fn resolution_phrase(res: &Resolution, short: bool) -> String {
    resolution_form(res, if short { 2 } else { 0 })
}

/// `level` 0 long (`died to a goblin archer`), 1 mid (`died to archer`), 2 short (`died`).
fn resolution_form(res: &Resolution, level: u8) -> String {
    match res {
        Resolution::Pending => "…".into(),
        Resolution::Banked { gold } => format!("banked ${}", gold.max(&0)),
        Resolution::Reached { depth } => format!("reached D{depth}"),
        Resolution::FirstBoss { .. } => "first boss".into(),
        Resolution::BossSlain { .. } => "boss slain".into(),
        Resolution::Returned { .. } => "returned".into(),
        Resolution::Lost { stalled } => if level > 0 { "returned" } else if *stalled { "stalled" } else { "lost the thread" }.into(),
        // Cut 13 §1: `stalled, archer no path`; the short form is the word alone.
        Resolution::Stalled { cause } => if level > 1 { "stalled".into() } else { format!("stalled, {}", stall_short(cause)) },
        Resolution::Died { cause } => match level {
            0 => format!("died to {}", cause_phrase(cause)),
            1 => {
                let c = cause_phrase(cause);
                format!("died to {}", c.rsplit(' ').next().unwrap_or(&c))
            }
            _ => "died".into(),
        },
        // QA on 524827b (qaAA: `An ogre took him to 17 HP; R5 attacked; driven off.` — the Warlord drove him off): the mid form keeps
        // the boss in three words (`fled the Warlord`); only the short form (the boss already the setup's subject) drops him
        Resolution::DrivenOff { kind } => match level {
            0 => format!("driven off by the {}", boss_short(kind)),
            1 => format!("fled the {}", boss_short(kind)),
            _ => "driven off".into(),
        },
        Resolution::Fell { kind, name } => {
            if level > 0 {
                format!("{name} fell")
            } else {
                format!("{} {name} fell", kind_title(kind))
            }
        }
        Resolution::Survived => "lived".into(),
    }
}

/// The story line: three beats, ≤ 12 words; each beat has a short form used in turn (the
/// threat, then the resolution, then the row — the turn beat is what the line is for) when
/// the long ones overflow.
pub fn story_line(ep: &Episode) -> String {
    if let Some(line) = routine_line(ep) {
        return line;
    }
    // QA on 1a2a4a9 (qaO: `The lock took him to 4 HP; no row fired; died.` · `Spectral blades
    // cornered him to 5 HP; no row fired; died.` beside DEATHS `fire ×1`): a death whose
    // killer is not the setup's threat keeps its killer — the end never shortens to a bare
    // `died`; the no-row beat shortens to `no row` first.
    let killer_named = match &ep.resolution {
        Resolution::Died { cause } => ep.threat.first().is_some_and(|(k, _)| cause_key(k) == cause_key(cause)),
        // QA on 524827b (qaAA): a drive-off keeps the boss who drove him off unless he is the setup's threat
        Resolution::DrivenOff { kind } => ep.threat.first().is_some_and(|(k, _)| k == kind),
        _ => true,
    };
    let mut s = String::new();
    for (rs, ts, es) in [(false, false, 0), (false, true, 0), (false, true, 1), (true, true, 1), (true, true, 2)] {
        let es = if killer_named { es } else { es.min(1) };
        // A death from full health has its killer in the setup (`An ogre took him down`); the
        // exit then says where, not who again (`died on D7`, not `died to an ogre`).
        let end = match &ep.resolution {
            Resolution::Died { .. } if ep.setup == Setup::Hurt && ep.low_hp <= 0 && killer_named => format!("died on D{}", ep.depth),
            res => resolution_form(res, es),
        };
        let mut turn = turn_phrase(ep, rs);
        if !killer_named && rs {
            if let Some(i) = NO_ROW[..3].iter().position(|x| *x == turn) {
                turn = NO_ROW_SHORT[i].into();
            } else if ep.combo.is_some() && ep.act.row >= 0 {
                // (the combo's `the chokepoint landed` gives way to its row's `R3 held`)
                turn = turn_phrase(&Episode { combo: None, ..ep.clone() }, rs);
            }
        }
        s = format!("{}; {turn}; {end}.", setup_phrase(ep, ts));
        if word_count(&s) <= STORY_WORDS {
            return s;
        }
    }
    crate::chronicle::clamp_words(&s, STORY_WORDS)
}

/// The no-row beats' short forms (`NO_ROW`'s first three, in order), for a death line that
/// must still name its killer.
pub const NO_ROW_SHORT: [&str; 3] = ["no row", "paralysed", "confused"];

/// Cut 12 §4: the routine line — a send that came home with nothing to tell (no low point)
/// reads the floor and its situation, the row that ended it, and what it brought:
/// `D6, the nest: R3 returned $54.` · `D2: returned $8.` · `D5, the vault: lost the thread.`
/// (rater P: "runs 4–8 repeated the D4–D6 archer/jackal loop with near-identical `returned
/// $NN` endings" — the floors now differ, and the line says how). `None` for any other episode.
pub fn routine_line(ep: &Episode) -> Option<String> {
    if ep.setup != Setup::Untouched {
        return None;
    }
    let res = match &ep.resolution {
        Resolution::Returned { gold } => format!("returned ${}", gold.max(&0)),
        Resolution::Banked { gold } => format!("banked ${}", gold.max(&0)),
        Resolution::Lost { stalled } => if *stalled { "stalled" } else { "lost the thread" }.into(),
        Resolution::Stalled { cause } => format!("stalled, {}", stall_short(cause)),
        Resolution::DrivenOff { kind } => format!("driven off by the {}", boss_short(kind)),
        _ => return None,
    };
    let home = matches!(ep.resolution, Resolution::Returned { .. } | Resolution::Banked { .. });
    let row = if home && ep.act.row >= 0 && matches!(ep.act.verb.v.as_str(), "return" | "bank") { format!("R{} ", ep.act.row + 1) } else { String::new() };
    let twist = ep.twist.as_deref().map(|t| format!(", the {}", crate::situations::twist_word(t))).unwrap_or_default();
    Some(format!("D{}{twist}: {row}{res}.", ep.depth))
}

/// Cut 12 §4: is this a routine line (`D6, the nest: R3 returned $54.`)? Its shape check.
pub fn routine_ok(text: &str) -> bool {
    let Some(body) = text.strip_suffix('.') else { return false };
    let Some((head, tail)) = body.split_once(": ") else { return false };
    let (depth, twist) = head.split_once(", the ").map(|(d, t)| (d, Some(t))).unwrap_or((head, None));
    let depth_ok = depth.strip_prefix('D').is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
    let twist_ok = twist.is_none_or(|t| crate::situations::TWISTS.iter().any(|k| crate::situations::twist_word(k) == t));
    let tail = match tail.split_once(' ') {
        Some((r, rest)) if r.starts_with('R') && r[1..].chars().all(|c| c.is_ascii_digit()) && r.len() > 1 => rest,
        _ => tail,
    };
    let res_ok = tail == "lost the thread" || stalled_ok(tail) || driven_ok(tail) || ["returned $", "banked $"].iter().any(|k| tail.strip_prefix(k).is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit())));
    depth_ok && twist_ok && res_ok && word_count(text) <= STORY_WORDS
}

/// Cut 13 §1: the stall's cause as the reel line reads it — `goblin archer, no path` →
/// `archer no path` (the title's last word, the reason), `paced` as is.
pub fn stall_short(cause: &str) -> String {
    match cause.split_once(", ") {
        Some((kind, why)) => format!("{} {why}", kind.rsplit(' ').next().unwrap_or(kind)),
        None => cause.to_string(),
    }
}

/// Cut 13 §1: the cause as the chronicle note reads it — `the archer, no path` · `paced`.
pub fn stall_note_cause(cause: &str) -> String {
    match cause.split_once(", ") {
        Some((kind, why)) => format!("the {}, {why}", kind.rsplit(' ').next().unwrap_or(kind)),
        None => cause.to_string(),
    }
}

/// Cut 24 §1: the grammar's driven-off resolution — `driven off`, `driven off by the <boss>`, or `fled the <boss>`.
pub fn driven_ok(end: &str) -> bool {
    end == "driven off" || end.strip_prefix("driven off by the ").or_else(|| end.strip_prefix("fled the ")).is_some_and(|b| crate::descent::BOSS_DEPTHS.iter().any(|(k, _)| boss_short(k) == b))
}

/// Cut 13 §1: the grammar's stall resolution — `stalled` alone, or `stalled, <word> no path`
/// / `stalled, <word> across water` with a monster title's last word, or `stalled, paced`;
/// Cut 18 §4: or the rules' loop (`stalled, R2 retreat ↔ explore` · `stalled, R1 retreat paced`).
pub fn stalled_ok(end: &str) -> bool {
    if end == "stalled" {
        return true;
    }
    let Some(cause) = end.strip_prefix("stalled, ") else { return false };
    if cause == "paced" || crate::turn::loop_cause_ok(cause) {
        return true;
    }
    let Some(word) = cause.strip_suffix(" no path").or_else(|| cause.strip_suffix(" across water")) else { return false };
    crate::defs::MONSTERS.iter().any(|m| kind_title(m.kind).to_lowercase().rsplit(' ').next() == Some(word))
}

/// The gate's check: three beats, ≤ 12 words, a setup form, a turn beat with a table verb
/// (or a no-row beat), a resolution from the set — or a routine line (Cut 12 §4).
pub fn story_ok(text: &str) -> bool {
    if routine_ok(text) {
        return true;
    }
    let Some(body) = text.strip_suffix('.') else { return false };
    let beats: Vec<&str> = body.split("; ").collect();
    if beats.len() != 3 || word_count(text) > STORY_WORDS {
        return false;
    }
    let setup = beats[0];
    let setup_ok = setup == "Untouched"
        || setup.starts_with("Untouched by ")
        || setup == "The cage held three"
        || setup == "A captive, chained"
        || setup.ends_with(" came back")
        || setup.ends_with(" took him down")
        || (setup.ends_with(" HP") && [" took him to ", " cornered him to ", " chased him to "].iter().any(|v| setup.contains(v)));
    let turn = beats[1];
    let forms = past_forms();
    let combo_ok = crate::rules::COMBOS.iter().any(|c| turn == format!("the {} landed", c.name) || turn == format!("the {} landed", crate::rules::combo_slug(c.name)));
    let turn_ok = NO_ROW.contains(&turn) || combo_ok || (setup == "The cage held three" && turn.starts_with("he took the ")) || {
        let (head, rest) = match turn.split_once(' ') {
            Some(x) => x,
            None => return false,
        };
        let rest = rest.strip_suffix(" him").unwrap_or(rest);
        let head_ok = (head.starts_with('R') && head[1..].chars().all(|c| c.is_ascii_digit()) && head.len() > 1)
            || ["greed", "cowardice", "bravery", "curiosity"].contains(&head)
            || (head == "the" && rest.starts_with("chores "));
        let rest = rest.strip_prefix("chores ").unwrap_or(rest);
        head_ok && forms.contains(&rest)
    };
    let end = beats[2];
    let end_ok = ["banked $", "reached D", "first boss", "boss slain", "returned", "lost the thread", "died", "lived"].iter().any(|k| end.starts_with(k)) || stalled_ok(end) || driven_ok(end) || end.ends_with(" fell");
    setup_ok && turn_ok && end_ok
}

/// Cut 12 §4: the row a routine line credits (`D6, the nest: R3 returned $54.`), if any.
fn routine_row(text: &str) -> bool {
    routine_ok(text) && text.split_once(": ").is_some_and(|(_, tail)| tail.starts_with('R') && tail.chars().nth(1).is_some_and(|c| c.is_ascii_digit()))
}

/// Does the line name a row, a trait or a companion (the tell-a-friend proxy)?
pub fn names_agent(h: &Highlight) -> bool {
    if routine_ok(&h.text) {
        return routine_row(&h.text);
    }
    let Some(body) = h.text.strip_suffix('.') else { return false };
    let beats: Vec<&str> = body.split("; ").collect();
    if beats.len() != 3 {
        return false;
    }
    let turn = beats[1];
    // Cut 8B §1: a combo names two rows.
    let row = (turn.starts_with('R') && turn.chars().nth(1).is_some_and(|c| c.is_ascii_digit())) || (turn.starts_with("the ") && turn.ends_with(" landed"));
    let trait_ = ["greed ", "cowardice ", "bravery ", "curiosity "].iter().any(|t| turn.starts_with(t));
    let companion = beats[2].ends_with(" fell") || beats[0].ends_with(" came back");
    row || trait_ || companion
}

// ---------------------------------------------------------------- scoring and the reel

/// Resolution weight (deaths of heirs with deeds and first bosses weigh most).
fn weight(res: &Resolution, named: bool) -> i32 {
    match res {
        Resolution::Pending => 1,
        Resolution::Banked { .. } => 3,
        Resolution::Reached { .. } => 2,
        Resolution::FirstBoss { .. } => 5,
        Resolution::BossSlain { .. } => 2,
        Resolution::Returned { .. } | Resolution::Lost { .. } | Resolution::Stalled { .. } | Resolution::Survived => 1,
        Resolution::DrivenOff { .. } => 2,
        Resolution::Died { .. } => {
            if named {
                5
            } else {
                3
            }
        }
        Resolution::Fell { .. } => 3,
    }
}

/// Score = low-point depth (1–5) × resolution weight, +2 for a situation.
pub fn score(ep: &Episode, named: bool) -> i32 {
    let depth = match ep.setup {
        Setup::Hurt => {
            let pct = (ep.low_hp * 100 / ep.max_hp.max(1)).clamp(0, 100);
            1 + (100 - pct) / 25
        }
        _ => 1,
    };
    depth * weight(&ep.resolution, named) + if ep.situation.is_some() { 2 } else { 0 }
}

/// Cut 13: the tally's word for a death cause — a bloat's `burst` is `gas`, as the reel says.
pub fn cause_key(cause: &str) -> String {
    match cause {
        "burst" => "gas".into(),
        other => other.into(),
    }
}

pub fn threat_key(ep: &Episode) -> String {
    match ep.setup {
        Setup::Vault => "vault".into(),
        Setup::Stray => "stray".into(),
        Setup::Captive => "captive".into(),
        _ => ep.threat.first().map(|(k, _)| k.clone()).unwrap_or_else(|| "none".into()),
    }
}

/// The resolution without its number (`banked $` · `reached D` · `fell`), for the pair test.
pub fn res_key(resolution: &str) -> String {
    if resolution.ends_with(" fell") {
        return "fell".into();
    }
    if resolution.starts_with("died") {
        return "died".into();
    }
    resolution.trim_end_matches(|c: char| c.is_ascii_digit()).to_string()
}

pub fn pair(h: &Highlight) -> Option<(String, String)> {
    h.arc.as_ref().map(|a| (a.threat.clone(), res_key(&a.resolution)))
}

pub fn to_highlight(run: &Run, ep: &Episode, named: bool) -> Highlight {
    Highlight {
        pattern: "episode".into(),
        score: score(ep, named),
        t: ep.t,
        run_id: run.id,
        text: story_line(ep),
        arc: Some(HighlightArc { low_hp: ep.low_hp.max(0) as u32, row: ep.act.row, threat: threat_key(ep), resolution: resolution_phrase(&ep.resolution, false) }),
    }
}

/// A finished run's episodes as highlights. `named`: the heir has deeds (its death weighs
/// more). Unresolved sealed episodes (none after an exit) are dropped.
/// QA on 92eb880 (qaN: two reel lines `…; reached D3.` while every run of the night ended at
/// D4–6): a low the hero walked down from reads the depth the run went on to reach, not the
/// floor after the low's (the reel is read as the run's outcome).
pub fn sift_with(run: &Run, named: bool) -> Vec<Highlight> {
    run.episodes
        .iter()
        .filter(|e| e.resolution != Resolution::Pending)
        .map(|e| match e.resolution {
            Resolution::Reached { depth } if run.max_depth > depth => {
                let mut e = e.clone();
                e.resolution = Resolution::Reached { depth: run.max_depth };
                to_highlight(run, &e, named)
            }
            _ => to_highlight(run, e, named),
        })
        .collect()
}

/// Highlights for a finished run against the lineage.
pub fn sift(run: &Run, l: &LineageState) -> Vec<Highlight> {
    let named = !l.heir_deeds.is_empty() || !run.boss_kills.is_empty();
    sift_with(run, named)
}

/// Cut 9 §6: does the highlight's turn beat name a row or a combo (`R2 drank`, `the bait
/// landed`) rather than a trait or `no row fired`?
pub fn names_row(h: &Highlight) -> bool {
    if routine_ok(&h.text) {
        return routine_row(&h.text);
    }
    let Some(body) = h.text.strip_suffix('.') else { return false };
    let beats: Vec<&str> = body.split("; ").collect();
    if beats.len() != 3 {
        return false;
    }
    let turn = beats[1];
    (turn.starts_with('R') && turn.chars().nth(1).is_some_and(|c| c.is_ascii_digit())) || (turn.starts_with("the ") && turn.ends_with(" landed"))
}

/// The reel: the best-depth run's closing episode leads, then the top episodes by score to
/// three — never two with the same (threat, resolution), and never a pair in `recent` (the
/// last three absences' reels, Cut 9 §6); an episode whose turn beat names a row or a combo
/// outranks one that reads `no row fired`. Other highlights (`bones`) fill to four. When the
/// best run's closing pair was shown lately its latest fresh episode leads instead; when
/// nothing at all is fresh, the closing episode is the reel alone (the one repeat allowed).
pub fn reel(highlights: &[Highlight], best_run: Option<u32>, recent: &[(String, String)]) -> Vec<Highlight> {
    let mut eps: Vec<&Highlight> = highlights.iter().filter(|h| h.arc.is_some()).collect();
    eps.sort_by(|a, b| names_row(b).cmp(&names_row(a)).then(b.score.cmp(&a.score)).then(a.run_id.cmp(&b.run_id)).then(a.t.cmp(&b.t)));
    let mut out: Vec<Highlight> = Vec::new();
    let mut seen: Vec<(String, String)> = recent.to_vec();
    let fresh = |h: &Highlight, seen: &[(String, String)], out: &[Highlight]| !seen.contains(&pair(h).unwrap()) && !out.iter().any(|o| o.text == h.text) && out.iter().filter(|o| shape(&o.text) == shape(&h.text)).count() < SHAPE_MAX;
    let mut mine: Vec<&Highlight> = eps.iter().copied().filter(|h| Some(h.run_id) == best_run).collect();
    mine.sort_by_key(|b| std::cmp::Reverse(b.t));
    if let Some(c) = mine.iter().find(|h| fresh(h, &seen, &out)) {
        seen.push(pair(c).unwrap());
        out.push((*c).clone());
    }
    // Three by rank after the lead (the lead's own slot is its own).
    let cap = 3 + out.len();
    for h in &eps {
        if out.len() >= cap {
            break;
        }
        if !fresh(h, &seen, &out) {
            continue;
        }
        seen.push(pair(h).unwrap());
        out.push((*h).clone());
    }
    if out.is_empty() {
        if let Some(c) = mine.first() {
            out.push((*c).clone());
        }
    }
    let mut rest: Vec<&Highlight> = highlights.iter().filter(|h| h.arc.is_none()).collect();
    rest.sort_by(|a, b| b.score.cmp(&a.score).then(a.run_id.cmp(&b.run_id)));
    for h in rest {
        if out.len() >= 4 {
            break;
        }
        if out.iter().filter(|o| shape(&o.text) == shape(&h.text)).count() >= SHAPE_MAX {
            continue;
        }
        out.push(h.clone());
    }
    out
}

// ---------------------------------------------------------------- §3 the hero's voice

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moment {
    Low,
    Resolved,
    GoldWithFoes,
    UnknownDrink,
    BossSeen,
}

/// Trait × moment, ≤ 3 words each.
pub fn voice_line(trait_: Trait, m: Moment) -> &'static str {
    match (trait_, m) {
        (Trait::Greedy, Moment::Low) => "not the gold",
        (Trait::Greedy, Moment::Resolved) => "still mine",
        (Trait::Greedy, Moment::GoldWithFoes) => "worth it",
        (Trait::Greedy, Moment::UnknownDrink) => "free, so",
        (Trait::Greedy, Moment::BossSeen) => "big purse",
        (Trait::Cowardly, Moment::Low) => "not today",
        (Trait::Cowardly, Moment::Resolved) => "still here",
        (Trait::Cowardly, Moment::GoldWithFoes) => "quick, quick",
        (Trait::Cowardly, Moment::UnknownDrink) => "hold my nose",
        (Trait::Cowardly, Moment::BossSeen) => "oh no",
        (Trait::Brave, Moment::Low) => "come on then",
        (Trait::Brave, Moment::Resolved) => "next",
        (Trait::Brave, Moment::GoldWithFoes) => "mine now",
        (Trait::Brave, Moment::UnknownDrink) => "bottoms up",
        (Trait::Brave, Moment::BossSeen) => "there you are",
        (Trait::Curious, Moment::Low) => "interesting",
        (Trait::Curious, Moment::Resolved) => "noted",
        (Trait::Curious, Moment::GoldWithFoes) => "shiny",
        (Trait::Curious, Moment::UnknownDrink) => "let's see",
        (Trait::Curious, Moment::BossSeen) => "so that's you",
    }
}

/// The hero speaks: at most once per `VOICE_EVERY` ticks, never in a fight's first ten.
pub fn voice(run: &mut Run, cx: &mut Ctx, m: Moment) -> bool {
    if run.voice_t.is_some_and(|t| run.turn < t + VOICE_EVERY) || run.fight_t.is_some_and(|t| run.turn < t + VOICE_FIGHT_QUIET) {
        return false;
    }
    run.voice_t = Some(run.turn);
    crate::chronicle::callout(run, cx, voice_line(run.trait_, m));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ep(threat: &[(&str, u32)], low: i32, row: i32, verb: &str, res: Resolution) -> Episode {
        Episode {
            t: 1,
            depth: 2,
            setup: Setup::Hurt,
            low_hp: low,
            max_hp: 36,
            threat: threat.iter().map(|(k, n)| (k.to_string(), *n)).collect(),
            act: Act { row, verb: Verb::new(verb), target: None, boss: false, walk: false },
            trait_: Trait::Greedy,
            resolution: res,
            ..Default::default()
        }
    }
    /// QA on 1a2a4a9 (qaO): a death line whose setup names another threat still names the
    /// killer — never a bare `died`.
    #[test]
    fn a_death_line_names_its_killer() {
        let mut blades = ep(&[("spectral_blade", 2)], 5, -2, "wait", Resolution::Died { cause: "fire".into() });
        blades.cornered = true;
        assert_eq!(story_line(&blades), "Spectral blades cornered him to 5 HP; no row; died to fire.");
        let lock = ep(&[("lock", 1)], 4, -2, "wait", Resolution::Died { cause: "gas".into() });
        assert_eq!(story_line(&lock), "A bloat took him to 4 HP; no row; died to gas.");
        let mut monkey = ep(&[("monkey", 1)], 2, -2, "wait", Resolution::Died { cause: "goblin_archer".into() });
        monkey.chased = true;
        let s = story_line(&monkey);
        assert!(s.ends_with("died to archer.") || s.ends_with("died to a goblin archer."), "{s}");
        let mut held = ep(&[("hunger", 1)], 1, -2, "paralysed", Resolution::Died { cause: "ghoul".into() });
        held.cornered = true;
        assert_eq!(story_line(&held), "The hunger cornered him to 1 HP; paralysed; died to ghoul.");
        for e in [&blades, &lock, &monkey, &held] {
            assert!(story_ok(&story_line(e)), "{}", story_line(e));
        }
        // the killer's own line may still shorten to `died`
        let own = ep(&[("gas", 1)], 4, -2, "wait", Resolution::Died { cause: "burst".into() });
        assert!(story_line(&own).starts_with("Gas took him to 4 HP;"), "{}", story_line(&own));
    }
    #[test]
    fn story_lines_follow_the_grammar() {
        let e = ep(&[("jackal", 2)], 3, 1, "drink", Resolution::Banked { gold: 58 });
        assert_eq!(story_line(&e), "Two jackals took him to 3 HP; R2 drank; banked $58.");
        let mut b = ep(&[("goblin_warlord", 1), ("goblin", 2)], 9, 3, "shield_bash", Resolution::FirstBoss { kind: "goblin_warlord".into() });
        b.act.boss = true;
        assert_eq!(story_line(&b), "The Warlord took him to 9 HP; R4 bashed him; first boss.");
        let g = ep(&[("jackal", 1)], 12, -1, "pick_up", Resolution::Fell { kind: "jackal".into(), name: "Uleth".into() });
        assert_eq!(story_line(&g), "A jackal took him to 12 HP; greed grabbed; Uleth fell.");
        let g2 = ep(&[("gas", 1)], 12, -1, "pick_up", Resolution::Banked { gold: 12 });
        assert_eq!(story_line(&g2), "Gas took him to 12 HP; greed took the gold; banked $12.");
        let d = ep(&[("gas", 1)], 4, -2, "wait", Resolution::Died { cause: "gas".into() });
        assert_eq!(story_line(&d), "Gas took him to 4 HP; no row fired; died to gas.");
        let long = ep(&[("goblin_archer", 3)], 3, 2, "tactic", Resolution::Died { cause: "goblin_archer".into() });
        let mut long = long;
        long.act.verb = Verb::arg("tactic", "corridor_fighting");
        long.cornered = true;
        let s = story_line(&long);
        assert!(word_count(&s) <= STORY_WORDS, "{s}");
        assert!(story_ok(&s), "{s}");
        for e in [&e, &b, &g, &d] {
            assert!(story_ok(&story_line(e)), "{}", story_line(e));
        }
        assert!(!story_ok("Down to 2 HP, then banked $313."));
        assert!(!story_ok("Two jackals took him to 3 HP; R2 flew; banked $58."));
    }
    #[test]
    fn plurals_and_causes() {
        assert_eq!(plural("jackal"), "jackals");
        assert_eq!(plural("pink jelly"), "pink jellies");
        assert_eq!(plural("goblin archer"), "goblin archers");
        assert_eq!(cause_phrase("ogre"), "an ogre");
        assert_eq!(cause_phrase("lich"), "the Lich");
        assert_eq!(cause_phrase("burst"), "gas");
    }
    #[test]
    fn voice_lines_are_three_words() {
        for t in Trait::ALL {
            for m in [Moment::Low, Moment::Resolved, Moment::GoldWithFoes, Moment::UnknownDrink, Moment::BossSeen] {
                assert!(word_count(voice_line(t, m)) <= 3, "{}", voice_line(t, m));
            }
        }
    }
}
