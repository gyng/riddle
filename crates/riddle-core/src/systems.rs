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

/// The curriculum in order (docs/PROGRESSION.md §6). Day 0: the send, the shaft, the forecast's
/// headline, the dial (the 4 preset rows' numbers).
pub const SYSTEMS: &[SystemDef] = &[
    sys("send", ""),
    sys("dial", ""),
    sys("headline", ""),
    sys("edit", "first death"),
    sys("death", "first death"),
    sys("exits", "first gold home"),
    sys("loadout", "first gold home"),
    sys("unlocks", "first mark"),
    sys("reorder", "first plateau"),
    sys("vs", "first plateau"),
    sys("tags", "first foe fact"),
    sys("party", "first stray"),
    sys("cage", "first cage"),
    sys("walls", "meet Warlord"),
    sys("divergence", "meet Warlord"),
    sys("forge", "slay Warlord"),
    sys("start", "slay Warlord"),
    sys("route", "D5 fork twice"),
    sys("oaths", "plateau or Warlord"),
    sys("automations", "meet Lich"),
    sys("route2", "an oath kept"),
    sys("heir_pick", "an oath kept"),
    sys("class", "second class"),
    // Cut 30: heir traits — the born slot and the wake's cards (heir 3, or a death past D5); the
    // blood slot and the trait conditions (heir 5).
    sys("traits", "heir 3"),
    sys("blood", "heir 5"),
];

/// The systems open on day 0.
pub const DAY0: [&str; 3] = ["send", "dial", "headline"];

/// Whether `id`'s trigger has happened for this lineage (read off its state; `plateau` — the
/// absence just met its first plateau — is the one moment state does not keep).
fn triggered(l: &LineageState, id: &str, plateau: bool) -> bool {
    let warlord_met = crate::meta::bosses_met(l) >= 1;
    match id {
        "send" | "dial" | "headline" => true,
        "edit" | "death" => !l.graveyard.is_empty() || l.heir > 1,
        "exits" | "loadout" => l.gold_ledger.iter().any(|x| x.delta > 0 && crate::engine::is_exit_why(&x.why)) || !l.banked_depths.is_empty(),
        "unlocks" => l.marks > 0 || l.rank > 0 || l.unlocks.iter().any(|u| crate::meta::def(u).is_some_and(|d| d.via == crate::meta::Via::Marks)),
        "reorder" | "vs" => plateau,
        "tags" => l.facts.iter().any(|f| f.starts_with("foe:") && f.matches(':').count() == 2 && !f.ends_with(":studied")),
        "party" => l.facts.contains("stray") || l.all_companions().next().is_some(),
        "cage" => l.facts.contains("vault"),
        "walls" | "divergence" => warlord_met,
        "forge" | "start" => l.kills.contains("goblin_warlord") || crate::kit::KIT_SLOTS.iter().any(|s| crate::kit::owned(l, s) > 0) || !l.waystones.is_empty(),
        "route" => l.forks_seen.get(&5).is_some_and(|n| *n >= 2) || !l.rules().route.is_empty(),
        "oaths" => plateau || warlord_met || crate::oath::open(l),
        "automations" => crate::meta::bosses_met(l) >= 3,
        "route2" => l.unlocks.contains("route2"),
        "heir_pick" => l.unlocks.contains("heir_pick"),
        "traits" => crate::traits::arrived(l),
        "blood" => crate::traits::blood_open(l),
        "class" => ["ranger", "caster"].iter().any(|c| l.unlocks.contains(*c)) || l.classes.iter().any(|(k, c)| k != "fighter" && (c.xp > 0 || c.level > 1)),
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
