//! Cut 11 §1: causes have causes. A small per-run provenance log — the last event that emptied
//! or filled each pack slot (`monkey took the heal, D3`, `drunk at 2/36 hp`, `found on D2`),
//! the use that started a cooldown, the tile a foe was last seen on, the blocker of the last
//! `no path` — and the `because` the row accounting attaches to a state reason from it.
//!
//! Cost: one entry per inventory change, cooldown start, target lost or path block; nothing
//! per tick. Sims and verdict replays (`Ctx.sim`) skip it entirely. The log lives on `Game`
//! (`Game.prov`, reached as `Ctx.prov`), not on `Run`: the history ring clones the run every
//! ten ticks and 64 entries of strings would ride along.
use crate::engine::{Ctx, Run};
use crate::rules::{Cond, Row};
use crate::tiles::{OverlayKind, Tile};
use crate::wire::Because;
use serde::{Deserialize, Serialize};

/// Entries kept per run (oldest evicted).
pub const PROV_CAP: usize = 64;
/// A `because` text is at most this many words.
pub const BECAUSE_WORDS: usize = 8;

/// What kind of event an entry records (the root-cause patch reads it).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProvKind {
    /// A theft (den snatch, a thief's blow, a monkey).
    Stolen,
    /// Drunk, read or thrown by a row, a chore or a trait.
    Used,
    /// Picked up, chosen from a vault, recovered from bones.
    Found,
    /// A pack item consumed some other way (a chalk mark, a leash on a tame, a swap).
    Spent,
    /// A hostile stepped out of view.
    Seen,
    /// A `no path` blocker.
    Path,
    /// A class verb's cooldown started.
    Cooldown,
}

/// One provenance entry. `key` is `item:<kind>` · `cooldown:<verb>` · `seen:<kind>` · `path`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Prov {
    pub t: u32,
    pub depth: u32,
    pub kind: ProvKind,
    pub key: String,
    pub text: String,
}

impl Prov {
    pub fn because(&self) -> Because {
        Because { text: self.text.clone(), t: self.t, depth: self.depth }
    }
}

/// Record an event. `replace`: an entry with the same key stands in for it (finds, sightings,
/// blockers, cooldowns — only the latest matters); thefts and uses append (each is a story).
/// Sims record nothing.
pub fn log(run: &Run, cx: &mut Ctx, kind: ProvKind, key: String, text: String, replace: bool) {
    if cx.sim {
        return;
    }
    if replace {
        if let Some(i) = cx.prov.iter().rposition(|p| p.key == key) {
            cx.prov.remove(i);
        }
    }
    cx.prov.push(Prov { t: run.turn, depth: run.depth, kind, key, text });
    if cx.prov.len() > PROV_CAP {
        let n = cx.prov.len() - PROV_CAP;
        cx.prov.drain(..n);
    }
}

/// The latest entry for a key.
pub fn last<'a>(prov: &'a [Prov], key: &str) -> Option<&'a Prov> {
    prov.iter().rev().find(|p| p.key == key)
}

/// The whole log as wire `because` entries, oldest first (`Trace.provenance`); `None` when
/// empty.
pub fn all(prov: &[Prov]) -> Option<Vec<Because>> {
    (!prov.is_empty()).then(|| prov.iter().map(Prov::because).collect())
}

// ---------------------------------------------------------------- recording helpers

/// A theft: `monkey took the heal, D3` (the den's snatch reads `den took …`). `kind` is the
/// item's kind, `by` the thief's, `label` the item as the hero knows it.
pub fn stolen(run: &Run, cx: &mut Ctx, kind: &str, by: &str, by_den: bool, label: &str) {
    let who = if by_den { "den".to_string() } else { crate::engine::kind_title(by).to_lowercase() };
    let text = format!("{who} took the {}, D{}", short_label(label), run.depth);
    log(run, cx, ProvKind::Stolen, format!("item:{kind}"), text, false);
}

/// A use: `R4 drank heal at 33/36 hp` when a row's verb did it (QA on 952e306: "which rule
/// drank it? R4 not named"), `drunk heal at 2/36 hp` from a chore or a trait (`read` /
/// `thrown` alike); `hp` is the HP before the effect.
pub fn used(run: &Run, cx: &mut Ctx, verb: &str, kind: &str, hp: i32) {
    let kind_word = kind.replace('_', " ");
    let text = if run.acting_row >= 0 {
        let did = match verb {
            "drunk" => "drank",
            "thrown" => "threw",
            v => v,
        };
        format!("R{} {did} {kind_word} at {hp}/{} hp", run.acting_row + 1, run.hero.max_hp.max(1))
    } else {
        format!("{verb} {kind_word} at {hp}/{} hp", run.hero.max_hp.max(1))
    };
    log(run, cx, ProvKind::Used, format!("item:{kind}"), text, false);
}

/// The oscillation guard fired: `paced 12 turns, foes ignored` (the rows' `stuck` reason
/// links to it; one entry per guard).
pub fn stuck(run: &Run, cx: &mut Ctx) {
    log(run, cx, ProvKind::Path, "stuck".into(), "paced 12 turns, foes ignored".into(), true);
}

/// A find: `found heal on D2` (replaces the slot's last find).
pub fn found(run: &Run, cx: &mut Ctx, kind: &str) {
    let text = format!("found {} on D{}", kind.replace('_', " "), run.depth);
    log(run, cx, ProvKind::Found, format!("item:{kind}"), text, true);
}

/// A pack item spent some other way: `chalk marked D3`, `leash spent on tame`, `swapped for
/// a heal potion`.
pub fn spent(run: &Run, cx: &mut Ctx, kind: &str, text: String) {
    log(run, cx, ProvKind::Spent, format!("item:{kind}"), text, false);
}

/// A class verb's cooldown started (`used shield bash`; the because reads the ticks left).
pub fn cooldown(run: &Run, cx: &mut Ctx, verb: &str) {
    let text = format!("used {}", verb.replace('_', " "));
    log(run, cx, ProvKind::Cooldown, format!("cooldown:{verb}"), text, true);
}

/// A hostile stepped out of view: `jackal last seen D6 (17,3)`.
pub fn seen(run: &Run, cx: &mut Ctx, kind: &str, x: i32, y: i32) {
    let text = format!("{} last seen D{} ({x},{y})", crate::engine::kind_title(kind).to_lowercase(), run.depth);
    log(run, cx, ProvKind::Seen, format!("seen:{kind}"), text, true);
}

/// `heal potion` → `heal`, `teleport scroll` → `teleport`; anything else as is.
fn short_label(label: &str) -> String {
    label.trim_end_matches(" potion").trim_end_matches(" scroll").to_string()
}

// ---------------------------------------------------------------- the because

/// The `because` of a row's reason, when the reason is a state and the run has an event for
/// it. `row` is the row (for a verb block), `cond` the failing condition (for a condition
/// reason).
pub fn because_for(run: &mut Run, cx: &mut Ctx, why: &str, row: Option<&Row>, cond: Option<&Cond>) -> Option<Because> {
    if cx.sim {
        return None;
    }
    match why {
        "no item" => {
            let a = row?.verb.a.as_deref().unwrap_or("");
            let kind = a.split(',').next().unwrap_or("");
            item_because(run, cx, kind)
        }
        "none held" => {
            let kind = cond.and_then(|c| c.t.as_deref())?;
            item_because(run, cx, kind)
        }
        "not in view" => {
            let tag = cond.and_then(|c| c.t.as_deref())?;
            Some(view_because(run, cx, tag))
        }
        "cooldown" => {
            let verb = row?.verb.v.as_str();
            let left = cooldown_left(run, verb);
            let t = last(cx.prov, &format!("cooldown:{verb}")).map(|p| (p.t, p.depth)).unwrap_or((run.turn, run.depth));
            Some(Because { text: format!("cooldown {left} ticks left"), t: t.0, depth: t.1 })
        }
        "locked cond" => {
            let c = cond?;
            let id = crate::meta::cond_unlock(&c.k)?;
            let cost = crate::meta::unlock_cost(id);
            Some(Because { text: format!("◆{cost} cond: {}", cond_word(&c.k)), t: run.turn, depth: run.depth })
        }
        "stuck" => last(cx.prov, "stuck").map(Prov::because),
        "no path" => {
            let text = path_blocker(run)?;
            // One entry per block: the tick points at the first action the blocker held.
            if last(cx.prov, "path").is_none_or(|p| p.text != text) {
                log(run, cx, ProvKind::Path, "path".into(), text, true);
            }
            last(cx.prov, "path").map(Prov::because)
        }
        _ => None,
    }
}

/// Why a tagged foe is `not in view`, most telling first: one is in view but its tag is not
/// yet a fact (`tag unlearned`); one is in view asleep (`asleep in view` — the den); one
/// stepped out of view (`jackal last seen D6 (17,3)`); the last one was slain (`jackal slain
/// D3`); none was met this run (`never met`).
fn view_because(run: &Run, cx: &Ctx, tag: &str) -> Because {
    let map = &run.floor.map;
    let now = |text: &str| Because { text: text.into(), t: run.turn, depth: run.depth };
    let mut tagged = run.monsters.iter().filter(|m| m.hp > 0 && m.has_tag(tag) && map.is_visible(m.pos)).peekable();
    if tagged.peek().is_some() {
        let (mut unlearned, mut asleep) = (false, false);
        for m in tagged {
            unlearned |= m.hostile() && !m.dormant && !crate::facts::tag_known(cx.facts, &m.kind, tag);
            asleep |= m.dormant;
        }
        if unlearned {
            return now("tag unlearned");
        }
        if asleep {
            return now("asleep in view");
        }
    }
    if let Some(p) = cx.prov.iter().rev().find(|p| p.kind == ProvKind::Seen && p.key.strip_prefix("seen:").is_some_and(|k| crate::defs::monster_def(k).tags.contains(&tag))) {
        return p.because();
    }
    if let Some((t, kind, depth)) = run.kills.iter().rev().find(|(_, k, _)| crate::defs::monster_def(k).tags.contains(&tag)) {
        return Because { text: format!("{} slain D{depth}", crate::engine::kind_title(kind).to_lowercase()), t: *t, depth: *depth };
    }
    now("never met")
}

/// The last event that emptied a slot; `never found` when the run has no event for the kind;
/// nothing when the last event filled it (the item left some way the log did not see).
fn item_because(run: &Run, cx: &Ctx, kind: &str) -> Option<Because> {
    if kind.is_empty() || kind == "unknown" {
        return None;
    }
    match last(cx.prov, &format!("item:{kind}")) {
        Some(p) if p.kind == ProvKind::Found => None,
        Some(p) => Some(p.because()),
        None => Some(Because { text: "never found".into(), t: run.turn, depth: run.depth }),
    }
}

/// Ticks left on a class verb's cooldown.
pub fn cooldown_left(run: &Run, verb: &str) -> i32 {
    let h = &run.hero;
    match verb {
        "shield_bash" => h.bash_cd,
        "cleave" => h.cleave_cd,
        "double_shot" => h.double_cd,
        "volley" => h.volley_cd,
        "blink" => h.blink_cd,
        "nova" => h.nova_cd,
        "vanish" => h.vanish_cd,
        "bulwark" => h.bulwark_cd,
        "ward" => h.ward_cd,
        _ => 0,
    }
    .max(0)
}

/// The condition token as the unlock card names it (`alert`, `turns`, `loot`, `on kill`, `on
/// see`, `party hp`).
pub fn cond_word(k: &str) -> String {
    k.trim_end_matches(['>', '=', '<']).replace('_', " ")
}

/// What blocks the way, ≤ 4 words: a captive chained on the stairs, a gas cloud near the hero,
/// the lock's bloats, a visible foe across water or out of reach, water round the hero, foes
/// on every side, an ally in the way; nothing when none of these.
pub fn path_blocker(run: &mut Run) -> Option<String> {
    let hp = run.hero.pos;
    let stairs = run.floor.stairs_down;
    // A visible hostile the hero cannot walk to: in the water, or nowhere on the distance field.
    let dist = crate::turn::hero_dist(run).to_vec();
    let map = &run.floor.map;
    let visible: Vec<usize> = (0..run.monsters.len()).filter(|&i| run.monsters[i].hp > 0 && run.monsters[i].hostile() && !run.monsters[i].dormant && map.is_visible(run.monsters[i].pos)).collect();
    if visible.iter().any(|&i| map.get(run.monsters[i].pos) == Tile::Water) {
        return Some("foe across water".into());
    }
    if !visible.is_empty() && visible.iter().all(|&i| dist.get(map.idx(run.monsters[i].pos)).is_none_or(|d| *d < 0) && !run.monsters[i].pos.adjacent(hp)) {
        return Some("no way to it".into());
    }
    // Every foe in view is one the hero gave up chasing (three approaches without closing) or
    // one that runs (a fleeing foe is only struck when adjacent).
    if !visible.is_empty() && visible.iter().all(|&i| run.is_ignored(run.monsters[i].id)) {
        return Some("chase given up".into());
    }
    if !visible.is_empty() && visible.iter().all(|&i| (run.monsters[i].fleeing || run.monsters[i].fear > 0) && !run.monsters[i].pos.adjacent(hp)) {
        return Some("foe fleeing".into());
    }
    if run.monsters.iter().any(|m| m.hp > 0 && m.neutral && m.situation.as_deref() == Some("captive") && m.pos == stairs) {
        return Some("captive chained the way".into());
    }
    if run.overlays.iter().any(|o| o.k == OverlayKind::Gas && crate::geom::Pos::new(o.x, o.y).cheb(hp) <= 4 && map.is_visible(crate::geom::Pos::new(o.x, o.y))) {
        return Some("gas cloud, this room".into());
    }
    if run.monsters.iter().any(|m| m.hp > 0 && m.situation.as_deref() == Some("lock") && m.pos.cheb(stairs) <= 6) {
        return Some("bloats seal the stair".into());
    }
    if hp.neighbours8().into_iter().filter(|q| map.in_bounds(*q) && map.get(*q) == Tile::Water).count() >= 3 {
        return Some("water in the way".into());
    }
    let open = hp.neighbours8().into_iter().filter(|q| map.passable(*q)).count();
    let blocked = hp.neighbours8().into_iter().filter(|q| map.passable(*q) && run.foe_blocks(*q)).count();
    if open > 0 && blocked >= open {
        return Some("foes on every side".into());
    }
    if blocked > 0 {
        return Some("foes hold the way".into());
    }
    if run.monsters.iter().any(|m| m.hp > 0 && !m.hostile() && m.pos.adjacent(hp)) {
        return Some("ally in the way".into());
    }
    None
}

/// A because text is ≤ `BECAUSE_WORDS` words (tests and the gate table).
pub fn because_ok(text: &str) -> bool {
    !text.is_empty() && crate::rules::word_count(text) <= BECAUSE_WORDS
}
