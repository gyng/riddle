//! Facts: what the hero learns by observation. Facts gate condition tokens.
use crate::engine::{Ctx, Run};
use crate::wire::Ev;
use std::collections::BTreeSet;

/// Record a fact; emits `fact` (and returns true) only when new.
pub fn learn(run: &mut Run, cx: &mut Ctx, fact: String) -> bool {
    if cx.facts.contains(&fact) {
        return false;
    }
    cx.facts.insert(fact.clone());
    cx.events.push(Ev::Fact { t: run.turn, fact: fact.clone() });
    let note = fact_note(&fact);
    if !note.is_empty() {
        crate::chronicle::note(run, cx, note);
    }
    true
}

pub fn learn_tag(run: &mut Run, cx: &mut Ctx, kind: &str, tag: &str) -> bool {
    learn(run, cx, format!("foe:{kind}:{tag}"))
}

fn fact_note(fact: &str) -> String {
    let parts: Vec<&str> = fact.split(':').collect();
    match parts.as_slice() {
        ["foe", kind] => format!("Met a {}.", crate::engine::kind_title(kind)),
        ["foe", kind, tag] => format!("{}: {}.", crate::engine::kind_title(kind), tag),
        ["biome", b] => format!("Entered the {b}."),
        ["boss", kind, "counter"] => format!("{}: counter learned.", crate::engine::kind_title(kind)),
        ["item", "leash"] => "Found a leash.".into(),
        ["tamed", kind] => format!("Tamed a {}.", crate::engine::kind_title(kind)),
        ["counter", rest] => format!("Learned: {}.", rest.replace('>', " beats ")),
        ["item", rest] => {
            let mut it = rest.split('=');
            let fl = it.next().unwrap_or("");
            let k = it.next().unwrap_or("");
            format!("The {} one: {}.", fl, k.replace('_', " "))
        }
        _ => String::new(),
    }
}

/// Any known fact carrying this tag (any monster kind).
pub fn has_tag_fact(facts: &BTreeSet<String>, tag: &str) -> bool {
    let suffix = format!(":{tag}");
    facts.iter().any(|f| f.starts_with("foe:") && f.ends_with(&suffix) && f.matches(':').count() == 2)
}

pub fn tag_known(facts: &BTreeSet<String>, kind: &str, tag: &str) -> bool {
    facts.contains(&format!("foe:{kind}:{tag}"))
}

pub fn has_boss_counter(facts: &BTreeSet<String>) -> bool {
    facts.iter().any(|f| f.starts_with("boss:") && f.ends_with(":counter"))
}

/// Sight-based facts, called after every vision update. Cheap when nothing changed.
pub fn on_vision(run: &mut Run, cx: &mut Ctx) {
    let map = &run.floor.map;
    let visible: Vec<usize> =
        (0..run.monsters.len()).filter(|i| run.monsters[*i].hp > 0 && map.is_visible(run.monsters[*i].pos)).collect();
    let ids: Vec<u32> = visible.iter().map(|&i| run.monsters[i].id).collect();
    if ids == run.last_visible {
        return;
    }
    run.last_visible = ids;
    let mut sight_facts: Vec<String> = Vec::new();
    let mut new_seen = false;
    for &i in &visible {
        let m = &run.monsters[i];
        if !run.seen_ids.contains(&m.id) {
            new_seen = true;
        }
        if m.ally {
            continue;
        }
        let kind = m.kind.as_str();
        let want = |tag: &str| !cx.facts.contains(&format!("foe:{kind}:{tag}"));
        if !cx.facts.contains(&format!("foe:{kind}")) {
            sight_facts.push(format!("foe:{kind}"));
        }
        if m.has_tag("undead") && want("undead") {
            sight_facts.push(format!("foe:{kind}:undead"));
        }
        if m.has_tag("boss") && want("boss") {
            sight_facts.push(format!("foe:{kind}:boss"));
        }
        if m.has_tag("ally") && m.neutral && want("ally") {
            sight_facts.push(format!("foe:{kind}:ally"));
        }
        if m.has_tag("water") && run.floor.map.get(m.pos) == crate::tiles::Tile::Water && want("water") {
            sight_facts.push(format!("foe:{kind}:water"));
        }
        if m.has_tag("pack") && want("pack") {
            let same = visible.iter().filter(|j| run.monsters[**j].kind == m.kind && run.monsters[**j].hostile()).count();
            if same >= 2 {
                sight_facts.push(format!("foe:{kind}:pack"));
            }
        }
    }
    for &i in &visible {
        let id = run.monsters[i].id;
        run.seen_ids.insert(id);
    }
    if new_seen {
        run.new_seen = true;
    }
    // Boss first sight (for the untouched trophy) and callout.
    for &i in &visible {
        if run.monsters[i].is_boss() && run.boss_seen_t.is_none() {
            run.boss_seen_t = Some(run.turn);
            run.hurt_since_boss = false;
            let title = run.monsters[i].title();
            cx.events.push(Ev::Callout { t: run.turn, text: title.clone() });
            crate::chronicle::note(run, cx, format!("The {title} waits."));
        }
    }
    for f in sight_facts {
        learn(run, cx, f);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tag_lookup() {
        let mut f = BTreeSet::new();
        f.insert("foe:jackal:pack".to_string());
        f.insert("foe:jackal".to_string());
        assert!(has_tag_fact(&f, "pack"));
        assert!(!has_tag_fact(&f, "fast"));
        assert!(tag_known(&f, "jackal", "pack"));
        assert!(!has_boss_counter(&f));
        f.insert("boss:lich:counter".into());
        assert!(has_boss_counter(&f));
    }
    #[test]
    fn notes_are_short() {
        for f in ["foe:jackal", "foe:jackal:pack", "biome:fens", "boss:lich:counter", "item:blue=heal"] {
            let n = fact_note(f);
            assert!(crate::rules::word_count(&n) <= 8, "{n}");
        }
    }
}
