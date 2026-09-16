//! Presets, bots and companion defaults used by the examples and tests.
use crate::hero::Class;
use crate::rng::Rng;
use crate::rules::{Cond, Row, RuleSet, Verb, Vocabulary};

/// The shipped class preset (DEFAULT bot).
pub fn preset(class: Class) -> RuleSet {
    match class {
        Class::Fighter => RuleSet {
            name: Some("fighter".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            ],
        },
        Class::Rogue => RuleSet {
            name: Some("rogue".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("hp<", 40), Cond::n("foes>=", 2)], Verb::new("vanish")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            ],
        },
        Class::Ranger => RuleSet {
            name: Some("ranger".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("adj>=", 1)], Verb::new("kite")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("shoot", "nearest")),
            ],
        },
        Class::Caster => RuleSet {
            name: Some("caster".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("adj>=", 1)], Verb::new("ward")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("bolt", "nearest")),
            ],
        },
    }
}

/// A genuinely good set within the vocabulary a player has after unlocking rows to 8 and
/// identifying the common items (EDITED bot). Shipped as presets/good.json. Cut 2: banks
/// when hurt past D5 (yield follows the exit), so the good player comes home with the loot.
pub fn good() -> RuleSet {
    RuleSet {
        name: Some("good".into()),
        rows: vec![
            Row::new(vec![Cond::n("hp<", 35)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("hp<", 30), Cond::n("depth>=", 5)], Verb::new("bank")),
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 6)], Verb::arg("throw", "fire,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
            Row::new(vec![Cond::n("foes>=", 3), Cond::n("hp<", 70)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![Cond::n("floor_seen>=", 60)], Verb::new("descend")),
        ],
    }
}

/// The four-row set from the browser playtest that once reached the ending (TRIVIAL bot).
pub fn trivial() -> RuleSet {
    RuleSet {
        name: Some("trivial".into()),
        rows: vec![
            Row::new(vec![], Verb::arg("tame", "nearest")),
            Row::new(vec![Cond::n("hp<", 50), Cond::n("foes>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![], Verb::arg("attack", "nearest")),
        ],
    }
}

/// TRIVIAL plus the boss counters: range for the Bloat Mother, the Warlord himself, the
/// Lich's summons first (COUNTERED bot).
pub fn countered() -> RuleSet {
    RuleSet {
        name: Some("countered".into()),
        rows: vec![
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 6)], Verb::arg("throw", "fire,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 6)], Verb::arg("throw", "poison,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
            Row::new(vec![], Verb::arg("tame", "nearest")),
            Row::new(vec![Cond::n("hp<", 50), Cond::n("foes>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![], Verb::arg("attack", "nearest")),
        ],
    }
}

/// Random rows from a vocabulary (RANDOM bot).
pub fn random_rules(rng: &mut Rng, vocab: &Vocabulary, rows: usize) -> RuleSet {
    let mut out = Vec::new();
    for _ in 0..rows {
        let n = rng.below(3) as usize;
        let mut conds = Vec::new();
        for _ in 0..n {
            let mut c = rng.pick(&vocab.conds).clone();
            if c.n.is_some() {
                c.n = Some(rng.range(0, 100));
            }
            conds.push(c);
        }
        let verb = rng.pick(&vocab.verbs).clone();
        out.push(Row::new(conds, verb));
    }
    RuleSet { rows: out, name: Some("random".into()) }
}

/// Default rows for a companion, derived from its tags (Addendum A).
pub fn default_companion_rules(tags: &[String], level: u32) -> RuleSet {
    let max = 1 + level as usize;
    let mut rows = Vec::new();
    let has = |t: &str| tags.iter().any(|x| x == t);
    if has("gas") {
        rows.push(Row::new(vec![Cond::n("self_hp<", 30), Cond::n("adj>=", 1)], Verb::new("burst")));
    }
    if has("ranged") {
        rows.push(Row::new(vec![Cond::n("foes>=", 1)], Verb::new("shoot")));
    }
    if has("undead") {
        rows.push(Row::new(vec![Cond::n("adj>=", 1)], Verb::new("drain")));
    }
    if has("thief") {
        rows.push(Row::new(vec![Cond::n("adj>=", 1)], Verb::new("steal")));
    }
    if has("pack") {
        rows.push(Row::new(vec![Cond::n("foes>=", 1)], Verb::new("flank")));
    }
    if has("splitter") {
        rows.push(Row::new(vec![Cond::n("foes>=", 2)], Verb::new("split")));
    }
    rows.push(Row::new(vec![Cond::n("adj>=", 1)], Verb::new("attack")));
    rows.push(Row::new(vec![Cond::n("self_hp<", 25)], Verb::new("follow")));
    rows.truncate(max);
    RuleSet { rows, name: None }
}

/// Give a game every fact (LEARNED bot).
pub fn learn_everything(g: &mut crate::engine::Game) {
    for m in crate::defs::MONSTERS {
        g.lineage.facts.insert(format!("foe:{}", m.kind));
        for t in m.tags {
            g.lineage.facts.insert(format!("foe:{}:{}", m.kind, t));
        }
        if m.boss {
            g.lineage.facts.insert(format!("boss:{}:counter", m.kind));
        }
        g.lineage.facts.insert(format!("foe:{}:studied", m.kind));
    }
    g.lineage.facts.insert("alert:rising".into());
    for i in crate::defs::ITEMS {
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, i.kind) {
            g.lineage.facts.insert(f);
        }
    }
    for b in ["warrens", "fens", "crypt"] {
        g.lineage.facts.insert(format!("biome:{b}"));
    }
    for (a, b, _) in crate::defs::COUNTERS {
        g.lineage.facts.insert(crate::defs::counter_fact(a, b));
    }
    g.lineage.facts.insert("item:leash".into());
}

/// Two level-3 bred companions with default rows (PETS bot).
pub fn pets_party() -> Vec<crate::wire::Companion> {
    let mk = |id: u32, kind: &str, name: &str, tags: &[&str]| {
        let tags: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
        let def = crate::defs::monster_def(kind);
        crate::wire::Companion {
            id,
            kind: kind.into(),
            name: name.into(),
            level: 3,
            tags: tags.clone(),
            gen: 1,
            rules: default_companion_rules(&tags, 3),
            max_rows: 4,
            hp: def.hp,
            max_hp: def.hp,
        }
    };
    vec![
        mk(900_001, "goblin_archer", "Skix", &["ranged", "telegraph", "gas"]),
        mk(900_002, "ghoul", "Vrak", &["undead", "pack", "paralyse"]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_validate() {
        for c in Class::ALL {
            assert!(preset(c).validate().is_ok());
        }
        assert!(good().validate().is_ok());
        assert!(good().rows.len() <= 8);
    }
    #[test]
    fn companion_rows_capped_by_level() {
        let r = default_companion_rules(&["ranged".into(), "gas".into()], 1);
        assert_eq!(r.rows.len(), 2);
        let r = default_companion_rules(&["ranged".into(), "gas".into()], 3);
        assert_eq!(r.rows.len(), 4);
    }
}
