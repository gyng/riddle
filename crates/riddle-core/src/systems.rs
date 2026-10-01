//! Cut 29 §2 (docs/PROGRESSION.md §6, the owner's request): the system curriculum — the camp's
//! systems open one at a time, each on the moment that makes it mean something; no tutorial text
//! (the reveal is the lesson). Core data: which systems a lineage has open (`LineageState::systems`,
//! sticky), each one's trigger, and the ones opened since the camp last looked (the glint). The
//! systems gate the editor's vocabulary and the camp's tiles on the client; a sim replays whatever
//! the set holds, and the bots (`metrics::setup`) open every system (`open_all`) — their play is the
//! same as before. An old save opens what its lineage has already used (`update` reads state).
use crate::engine::LineageState;
use crate::wire::SystemInfo;

/// A system and the moment that opens it (≤ 3 words: the client's glint may name it).
pub struct SystemDef {
    pub id: &'static str,
    pub trigger: &'static str,
    /// Cut 30 (docs/PROGRESSION_V2.md §4): the lineage's age (hours, offline included) the system waits
    /// for besides its trigger, and the age that opens it without its trigger (0: none).
    pub min_age_h: u32,
    pub fallback_h: u32,
}

const fn sys(id: &'static str, trigger: &'static str) -> SystemDef {
    SystemDef { id, trigger, min_age_h: 0, fallback_h: 0 }
}

const fn aged(id: &'static str, trigger: &'static str, min_age_h: u32, fallback_h: u32) -> SystemDef {
    SystemDef { id, trigger, min_age_h, fallback_h }
}

/// The curriculum in order. Cut 30 (docs/CUT30.md, Reveal): day 0 is the camp — the send, the shaft's
/// headline; the first gold home brings the blacksmith and the exits; the first find kept the
/// storehouse; the first stray the party, the first tame the kennel; the Warlord met a second stance
/// (the stance panel), slain the tactics, the waystones and the quest board; a purse of a night's net
/// the bank; heir 3 the temperaments (the wake's cards); the Lich met a second tactic slot; the Mother
/// met (or a 3-day stall) **the pen** — the editor, the dial, the marks catalogue, the order, the vs line,
/// the tags, the walls, the divergence scene, the automations, the routes. The oath board is gone (the
/// quest board is its successor); the trait slots wait (not in Cut 30).
pub const SYSTEMS: &[SystemDef] = &[
    sys("send", ""),
    sys("headline", ""),
    sys("death", "first death"),
    sys("exits", "first gold home"),
    sys("forge", "first gold home"),
    sys("loadout", "first gold home"),
    sys("storehouse", "first find kept"),
    aged("party", "first stray", 1, 0),
    aged("kennel", "first tame", 1, 0),
    sys("cage", "first cage"),
    aged("stances", "meet Captain", 1, 0),
    aged("tactics", "slay Warlord", 4, 0),
    aged("bank", "a night's purse", 8, 0),
    aged("quests", "slay Warlord", 12, 0),
    aged("class", "second class", 16, 0),
    aged("start", "slay Warlord", 20, 0),
    aged("temperament", "heir 3", 24, 72),
    aged("tactic2", "meet Lich", 56, 0),
    aged("pen", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("edit", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("dial", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("unlocks", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("reorder", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("vs", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("tags", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("walls", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("divergence", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("route", "meet Mother", PEN_AGE_H, PEN_FALLBACK_H),
    aged("automations", "meet Lich", 96, 0),
    sys("route2", "an oath kept"),
    sys("heir_pick", "an oath kept"),
];

/// Cut 30 (PROGRESSION_V2 §4): the pen waits for the Mother met and an age of 72 h, and opens at 5 days
/// whatever the climb.
pub const PEN_AGE_H: u32 = 72;
pub const PEN_FALLBACK_H: u32 = 120;

/// The systems open on day 0.
pub const DAY0: [&str; 2] = ["send", "headline"];

/// The systems the pen brings (the Mother met, a 3-day stall, an old save's set).
pub const PEN: [&str; 10] = ["pen", "edit", "dial", "unlocks", "reorder", "vs", "tags", "walls", "divergence", "route"];

/// Whether `id`'s trigger has happened for this lineage (read off its state; `plateau` is kept for the
/// caller's signature — the order and the vs line come with the pen now).
fn triggered(l: &LineageState, id: &str, plateau: bool) -> bool {
    let _ = plateau;
    let warlord_met = crate::meta::bosses_met(l) >= 1 || l.pkg.meets.contains_key("goblin_warlord");
    let pen = l.pkg.pen_open || l.pkg.literal;
    match id {
        "send" | "headline" => true,
        "death" => !l.graveyard.is_empty() || l.heir > 1,
        "exits" | "loadout" | "forge" => l.gold_ledger.iter().any(|x| x.delta > 0 && crate::engine::is_exit_why(&x.why)) || !l.banked_depths.is_empty() || crate::town::built(l, "blacksmith"),
        "storehouse" => crate::town::built(l, "storehouse"),
        "party" => l.facts.contains("stray") || l.all_companions().next().is_some(),
        "kennel" => crate::town::built(l, "kennel"),
        "cage" => l.facts.contains("vault"),
        "stances" => warlord_met || l.facts.contains("foe:goblin_captain"),
        "tactics" | "start" | "quests" => l.kills.contains("goblin_warlord") || !l.waystones.is_empty() && l.best_depth >= 9,
        "bank" => crate::town::built(l, "bank"),
        "temperament" => crate::packages::temperament_open(l),
        "pen" | "edit" | "dial" | "unlocks" | "reorder" | "vs" | "tags" | "walls" | "divergence" | "route" => pen || crate::packages::mother_met(l),
        "tactic2" => crate::packages::tactic_slots(l) >= 2,
        "class" => ["ranger", "caster"].iter().any(|c| l.unlocks.contains(*c)) || l.classes.iter().any(|(k, c)| k != "fighter" && (c.xp > 0 || c.level > 1)),
        "automations" => pen && crate::meta::bosses_met(l) >= 3,
        "route2" => l.unlocks.contains("route2"),
        "heir_pick" => l.unlocks.contains("heir_pick"),
        _ => false,
    }
}

/// Open every system whose trigger has happened; the ids opened now (in curriculum order), also
/// queued on `systems_new` for the camp's glint.
pub fn update(l: &mut LineageState, plateau: bool) -> Vec<String> {
    // (a harness's lineage opens on the trigger alone, as before Cut 30)
    let gate = !l.pkg.literal;
    update_with(l, plateau, gate)
}

/// The next system to come (the first closed one in order, the queue's head first).
pub fn next(l: &LineageState) -> Option<crate::wire::RevealNext> {
    let s = SYSTEMS.iter().find(|s| l.reveal_queue.first().map_or(!l.systems.contains(s.id), |q| q == s.id))?;
    let t = triggered(l, s.id, false);
    let age = l.age_h();
    let wait = if t { s.min_age_h.saturating_sub(age) } else if s.fallback_h > 0 { s.fallback_h.saturating_sub(age) } else { 0 };
    Some(crate::wire::RevealNext { id: s.id.into(), trigger: s.trigger.into(), triggered: t, wait_h: wait })
}

/// Whether `s` is ready now: its trigger and its age, or its fallback age.
fn ready(l: &LineageState, s: &SystemDef, plateau: bool, gate: bool) -> bool {
    if !gate {
        return triggered(l, s.id, plateau);
    }
    // (a pen already open — an old save's — brings its group whatever the age)
    if l.pkg.pen_open && PEN.contains(&s.id) {
        return true;
    }
    let age = l.age_h();
    (triggered(l, s.id, plateau) && age >= s.min_age_h) || (s.fallback_h > 0 && age >= s.fallback_h)
}

/// `update`, the age gates and the reveal budget on (`gate`) or off (an old save's upgrade, a harness).
/// Cut 30 (PROGRESSION_V2 §4): at most `LineageState::reveal_left` new systems a report — a unit is the
/// run of systems that share a trigger (the blacksmith's three, the pen's group) — the rest wait in
/// `reveal_queue` in curriculum order.
pub fn update_with(l: &mut LineageState, plateau: bool, gate: bool) -> Vec<String> {
    let mut out = Vec::new();
    let mut queue = Vec::new();
    let mut i = 0;
    while i < SYSTEMS.len() {
        let s = &SYSTEMS[i];
        // the unit: this system and the next ones with its trigger and age
        let mut j = i + 1;
        while j < SYSTEMS.len() && !s.trigger.is_empty() && SYSTEMS[j].trigger == s.trigger && SYSTEMS[j].min_age_h == s.min_age_h {
            j += 1;
        }
        let unit: Vec<&SystemDef> = SYSTEMS[i..j].iter().filter(|u| !l.systems.contains(u.id) && ready(l, u, plateau, gate)).collect();
        i = j;
        if unit.is_empty() {
            continue;
        }
        let free = unit.iter().all(|u| DAY0.contains(&u.id));
        if gate && !free && l.reveal_left == 0 {
            queue.push(unit[0].id.to_string());
            continue;
        }
        if gate && !free {
            l.reveal_left -= 1;
        }
        for u in unit {
            l.systems.insert(u.id.to_string());
            out.push(u.id.to_string());
        }
    }
    l.reveal_queue = queue;
    // the pen's group is the packages' pen
    if l.systems.contains("pen") && !l.pkg.pen_open && !l.pkg.literal {
        l.pkg.pen_open = true;
        crate::packages::recompile(l);
    }
    for id in &out {
        if !DAY0.contains(&id.as_str()) && !l.systems_new.contains(id) {
            l.systems_new.push(id.clone());
        }
    }
    out
}

/// An old save opens what its lineage has used: every triggered system, and the plateau's two (the
/// order and the vs line) once any row of its sets is the player's own.
pub fn upgrade(l: &mut LineageState) {
    let fresh = l.systems.is_empty();
    let edited = l.sets.iter().any(|s| s.rows.iter().any(|r| r.origin.as_deref() != Some("preset")));
    update_with(l, fresh && edited, false);
    // Cut 30: the retired ids (the oath board, the trait slots) close; the pen's group is the old editor
    for gone in ["oaths", "traits", "blood"] {
        l.systems.remove(gone);
    }
    if fresh {
        l.systems_new.clear();
    }
}

/// Every system open (the bots: `metrics::setup`; nothing glints).
pub fn open_all(l: &mut LineageState) {
    for s in SYSTEMS {
        l.systems.insert(s.id.to_string());
    }
    l.systems_new.clear();
}

pub fn is_open(l: &LineageState, id: &str) -> bool {
    l.systems.contains(id)
}

/// The curriculum on the wire: every system in order, open or not, with its trigger; `new` the
/// ones opened since the camp last looked.
pub fn wire(l: &LineageState) -> Vec<SystemInfo> {
    SYSTEMS.iter().map(|s| SystemInfo { id: s.id.into(), open: l.systems.contains(s.id), trigger: s.trigger.into(), new: l.systems_new.iter().any(|x| x == s.id) }).collect()
}
