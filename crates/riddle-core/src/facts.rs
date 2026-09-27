//! Facts: what the hero learns by observation. Facts gate condition tokens.
use crate::engine::{Ctx, Run};
use crate::rules::{Cond, Row, Verb};
use crate::wire::Ev;
use std::collections::BTreeSet;

/// Record a fact; emits `fact` (and returns true) only when new.
pub fn learn(run: &mut Run, cx: &mut Ctx, fact: String) -> bool {
    if cx.facts.contains(&fact) {
        return false;
    }
    cx.facts.insert(fact.clone());
    run.learned.push(fact.clone());
    cx.events.push(Ev::Fact { t: run.turn, fact: fact.clone() });
    let note = fact_note(&fact);
    if !note.is_empty() {
        crate::chronicle::note(run, cx, note);
    }
    true
}

/// Cut 26 §2: the hero sees a fork's two stairs (the down stairs of the floor above a band whose
/// stairs offer the near biome and the next one; not when the fork above took its far stair):
/// the `fork:<depth>` fact, once per lineage (the editor shows the fork from then on), and the
/// `TWO STAIRS` callout, once per run and fork. The route decides which stair the hero takes.
pub fn fork_seen(run: &mut Run, cx: &mut Ctx) {
    let next = run.depth + 1;
    if run.fork_seen >= next || !crate::descent::fork_open_for(run.route2, next) || !run.route.fork_open(next) || !run.floor.map.is_visible(run.floor.stairs_down) {
        return;
    }
    run.fork_seen = next;
    learn(run, cx, format!("fork:{next}"));
    crate::chronicle::callout(run, cx, "TWO STAIRS");
}

pub fn learn_tag(run: &mut Run, cx: &mut Ctx, kind: &str, tag: &str) -> bool {
    learn(run, cx, format!("foe:{kind}:{tag}"))
}

/// Cut 2 §5, third tier: a kill counts toward the kind's `studied` fact (five kills).
pub fn on_kill(run: &mut Run, cx: &mut Ctx, kind: &str) {
    if kind.starts_with("spectral_") {
        return;
    }
    let n = cx.kill_counts.entry(kind.to_string()).or_insert(0);
    *n += 1;
    if *n >= crate::engine::STUDIED_KILLS {
        learn(run, cx, format!("foe:{kind}:studied"));
    }
}

pub fn is_studied(facts: &BTreeSet<String>, kind: &str) -> bool {
    facts.contains(&format!("foe:{kind}:studied"))
}

fn fact_note(fact: &str) -> String {
    // Cut 6 §5: the counter fact carries a row (`boss:<kind>:counter=attack tag:boss`).
    if let Some(rest) = fact.strip_prefix("boss:") {
        if let Some((kind, _)) = rest.split_once(":counter") {
            return format!("{}: counter learned.", crate::engine::kind_title(kind));
        }
    }
    let parts: Vec<&str> = fact.split(':').collect();
    match parts.as_slice() {
        ["foe", kind] => { let t = crate::engine::kind_title(kind); format!("Met {} {t}.", if t.starts_with(|c: char| "aeiouAEIOU".contains(c)) { "an" } else { "a" }) }
        ["foe", kind, "studied"] => format!("{}: studied.", crate::engine::kind_title(kind)),
        ["foe", kind, tag] => format!("{}: {}.", crate::engine::kind_title(kind), tag),
        ["bones", d] => format!("Bones lie on D{d}."),
        ["alert", "rising"] => "The dungeon listens.".into(),
        ["biome", b] => format!("Entered the {b}."),
        ["item", "leash"] => "Found a leash.".into(),
        ["item", "lantern"] => "Found a lantern.".into(),
        ["item", "bell" | "salt" | "chalk"] => format!("Found {}.", parts[1]),
        ["item", "mirror_shard"] => "Found a mirror shard.".into(),
        ["item", "recall"] => "Recall: a way home.".into(),
        ["chalk", d] => format!("D{d} is chalked."),
        ["ascended", v] => format!("Ascended: {}.", v.replace('_', " ")),
        ["tamed", kind] => format!("Tamed a {}.", crate::engine::kind_title(kind)),
        ["shrine"] | ["vault"] | ["nest"] | ["stray"] | ["den"] | ["lock"] | ["captive"] | ["hunger"] => String::new(),
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
    facts.iter().any(|f| f.starts_with("foe:") && f.ends_with(&suffix) && f.matches(':').count() == 2 && tag != "studied")
}

pub fn tag_known(facts: &BTreeSet<String>, kind: &str, tag: &str) -> bool {
    facts.contains(&format!("foe:{kind}:{tag}"))
}

pub fn has_boss_counter(facts: &BTreeSet<String>) -> bool {
    facts.iter().any(|f| f.starts_with("boss:") && f.contains(":counter"))
}

// ---------------------------------------------------------------- Cut 6 §5: counter rows

/// The counter row of a boss kind: what beats it, as one row the player can write.
pub fn counter_row(kind: &str) -> Row {
    let boss = Cond::t("foe_tag", "boss");
    match kind {
        "bloat_mother" => Row::new(vec![boss], Verb::arg("throw", "fire,tag:boss")),
        "lich" => Row::new(vec![Cond::t("foe_tag", "summoned")], Verb::arg("attack", "tag:summoned")),
        "foundry_master" => Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")),
        "lurker_queen" => Row::new(vec![boss], Verb::arg("read", "silence")),
        "mirror_king" => Row::new(vec![boss], Verb::arg("tactic", "cadence")),
        _ => Row::new(vec![boss], Verb::arg("attack", "tag:boss")),
    }
}

/// Cut 24 §1: a boss's defence, ≤ 3 words — what shrugs the blows of a set without its counter
/// (the `no counter` exit names it: `Warlord · shields up · attack boss`). QA on 524827b (qaAA:
/// `shield wall` "appears nowhere else"): the Warlord's is the watch's own callout, `shields up`.
pub fn boss_trait(kind: &str) -> &'static str {
    match kind {
        "goblin_warlord" => "shields up",
        "bloat_mother" => "heals in gas",
        "lich" => "endless dead",
        "foundry_master" => "reflects blows",
        "lurker_queen" => "brood shields",
        "mirror_king" => "mirrors verbs",
        _ => "unhurt",
    }
}

/// The row as it reads in the fact: `attack tag:boss` · `throw fire,tag:boss` · `read silence`.
pub fn counter_row_key(row: &Row) -> String {
    match &row.verb.a {
        Some(a) => format!("{} {a}", row.verb.v),
        None => row.verb.v.clone(),
    }
}

/// ≤ 3 words: `attack boss` · `throw fire, boss` · `read silence` · `reflect read` · `cadence`.
pub fn counter_text(row: &Row) -> String {
    let a = row.verb.a.as_deref().unwrap_or("");
    match row.verb.v.as_str() {
        "attack" => format!("attack {}", a.trim_start_matches("tag:")),
        "throw" => {
            let mut it = a.split(',');
            let k = it.next().unwrap_or("");
            match it.next() {
                Some(sel) => format!("throw {k}, {}", sel.trim_start_matches("tag:")),
                None => format!("throw {k}"),
            }
        }
        "tactic" => a.replace('_', " "),
        v => format!("{v} {a}").trim().to_string(),
    }
}

/// The fact a boss's telegraph teaches: `boss:goblin_warlord:counter=attack tag:boss` (the old
/// key `boss:<kind>:counter` stays its prefix, so gates and `needs` strings still match).
pub fn boss_counter_fact(kind: &str) -> String {
    format!("boss:{kind}:counter={}", counter_row_key(&counter_row(kind)))
}

/// The counter of `kind` is known (either fact form).
pub fn boss_counter_known(facts: &BTreeSet<String>, kind: &str) -> bool {
    let key = format!("boss:{kind}:counter");
    facts.iter().any(|f| f.starts_with(&key))
}

/// The known counter row of `kind`, if its fact is held.
pub fn boss_counter_row(facts: &BTreeSet<String>, kind: &str) -> Option<Row> {
    boss_counter_known(facts, kind).then(|| counter_row(kind))
}

/// Learn a boss's counter (once, whatever the fact's form).
pub fn learn_boss_counter(run: &mut Run, cx: &mut Ctx, kind: &str) -> bool {
    if boss_counter_known(cx.facts, kind) {
        return false;
    }
    learn(run, cx, boss_counter_fact(kind))
}

/// Cut 6 §5: a save's old-form counter facts (`boss:<kind>:counter`) take the row.
pub fn upgrade_counter_facts(facts: &mut BTreeSet<String>) {
    let old: Vec<String> = facts.iter().filter(|f| f.starts_with("boss:") && f.ends_with(":counter")).cloned().collect();
    for f in old {
        let kind = f.trim_start_matches("boss:").trim_end_matches(":counter").to_string();
        facts.remove(&f);
        facts.insert(boss_counter_fact(&kind));
    }
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
    // Cut 11 §1: a hostile that stepped out of view leaves its last tile in the provenance
    // log (`not in view` → `last seen D6 (17,3)`).
    if !cx.sim {
        let lost: Vec<(String, i32, i32)> = run
            .last_visible
            .iter()
            .filter(|id| !ids.contains(id))
            .filter_map(|id| run.monsters.iter().find(|m| m.id == *id && m.hp > 0 && m.hostile()))
            .map(|m| (m.kind.clone(), m.pos.x, m.pos.y))
            .collect();
        for (kind, x, y) in lost {
            crate::provenance::seen(run, cx, &kind, x, y);
        }
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
        // Cut 29 (the D28 probe: `noise_discipline`'s `fact: blind` was never learned, the card unbuyable):
        // a blind hunter is seen to hunt by sound — the tag is a sight fact.
        if m.has_tag("blind") && m.hostile() && want("blind") {
            sight_facts.push(format!("foe:{kind}:blind"));
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
        // Cut 3: a blind hunter seen on this floor holds `noise_discipline`'s rest.
        if run.monsters[i].hostile() && run.monsters[i].is_blind() && !run.blind_seen.contains(&id) {
            run.blind_seen.push(id);
        }
    }
    if new_seen {
        run.new_seen = true;
    }
    // Boss first sight (for the untouched trophy) and callout.
    for &i in &visible {
        if run.monsters[i].is_boss() && run.boss_seen_t.is_none() {
            run.boss_seen_t = Some(run.turn);
            run.hurt_since_boss = false;
            run.wall_seen = true;   // Cut 28b: the oath board opens (`LineageState::oath_open`)
            let title = run.monsters[i].title();
            cx.events.push(Ev::Callout { t: run.turn, text: title.clone(), why: None });
            crate::chronicle::note(run, cx, format!("The {title} waits."));
            // Cut 5 §1/§3: the boss leads its own episode; the hero has a word for it.
            crate::sifter::seal(run);
            crate::sifter::voice(run, cx, crate::sifter::Moment::BossSeen);
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
        assert!(boss_counter_known(&f, "lich"));
        upgrade_counter_facts(&mut f);
        assert!(f.contains("boss:lich:counter=attack tag:summoned"), "{f:?}");
        assert!(boss_counter_known(&f, "lich") && has_boss_counter(&f) && !boss_counter_known(&f, "goblin_warlord"));
        assert_eq!(boss_counter_fact("goblin_warlord"), "boss:goblin_warlord:counter=attack tag:boss");
        assert_eq!(boss_counter_fact("bloat_mother"), "boss:bloat_mother:counter=throw fire,tag:boss");
        for k in ["goblin_warlord", "bloat_mother", "lich", "foundry_master", "lurker_queen", "mirror_king"] {
            let t = counter_text(&counter_row(k));
            assert!(crate::rules::word_count(&t) <= 3, "{t}");
        }
        assert_eq!(counter_text(&counter_row("bloat_mother")), "throw fire, boss");
        assert_eq!(counter_text(&counter_row("lurker_queen")), "read silence");
    }
    #[test]
    fn notes_are_short() {
        for f in ["foe:jackal", "foe:jackal:pack", "biome:fens", "boss:lich:counter", "boss:lich:counter=attack tag:summoned", "item:blue=heal"] {
            let n = fact_note(f);
            assert!(crate::rules::word_count(&n) <= 8, "{n}");
        }
    }
}
