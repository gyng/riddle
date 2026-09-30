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
}

const fn sys(id: &'static str, trigger: &'static str) -> SystemDef {
    SystemDef { id, trigger }
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
    sys("party", "first stray"),
    sys("kennel", "first tame"),
    sys("cage", "first cage"),
    sys("stances", "meet Warlord"),
    sys("tactics", "slay Warlord"),
    sys("start", "slay Warlord"),
    sys("quests", "slay Warlord"),
    sys("bank", "a night's purse"),
    sys("temperament", "heir 3"),
    sys("tactic2", "meet Lich"),
    sys("class", "second class"),
    sys("pen", "meet Mother"),
    sys("edit", "meet Mother"),
    sys("dial", "meet Mother"),
    sys("unlocks", "meet Mother"),
    sys("reorder", "meet Mother"),
    sys("vs", "meet Mother"),
    sys("tags", "meet Mother"),
    sys("walls", "meet Mother"),
    sys("divergence", "meet Mother"),
    sys("route", "meet Mother"),
    sys("automations", "meet Lich"),
    sys("route2", "an oath kept"),
    sys("heir_pick", "an oath kept"),
];

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
        "stances" => warlord_met,
        "tactics" | "start" | "quests" => l.kills.contains("goblin_warlord") || !l.waystones.is_empty() && l.best_depth >= 9,
        "bank" => crate::town::built(l, "bank"),
        "temperament" => crate::packages::temperament_open(l),
        "tactic2" => crate::packages::tactic_slots(l) >= 2,
        "class" => ["ranger", "caster"].iter().any(|c| l.unlocks.contains(*c)) || l.classes.iter().any(|(k, c)| k != "fighter" && (c.xp > 0 || c.level > 1)),
        "automations" => pen && crate::meta::bosses_met(l) >= 3,
        "route2" => l.unlocks.contains("route2"),
        "heir_pick" => l.unlocks.contains("heir_pick"),
        p if PEN.contains(&p) => pen,
        _ => false,
    }
}

/// Open every system whose trigger has happened; the ids opened now (in curriculum order), also
/// queued on `systems_new` for the camp's glint.
pub fn update(l: &mut LineageState, plateau: bool) -> Vec<String> {
    let mut out = Vec::new();
    for s in SYSTEMS {
        if !l.systems.contains(s.id) && triggered(l, s.id, plateau) {
            l.systems.insert(s.id.to_string());
            out.push(s.id.to_string());
        }
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
    update(l, fresh && edited);
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
