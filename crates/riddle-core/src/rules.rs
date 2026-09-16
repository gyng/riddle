//! Rule rows: conditions, verbs, rule sets (wire types) and validation.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Cond {
    pub k: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Verb {
    pub v: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub a: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, Default)]
pub struct Row {
    pub conds: Vec<Cond>,
    pub verb: Verb,
    /// Cut 7 §2: where the row came from — `preset` (the shipped two rows, tagged on a new
    /// lineage), `card` (a bought card's row), `patch` / `player` (tagged by the client). Not
    /// part of a row's identity: two rows with the same conds and verb are equal whatever
    /// their origin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

impl PartialEq for Row {
    fn eq(&self, o: &Row) -> bool {
        self.conds == o.conds && self.verb == o.verb
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct RuleSet {
    pub rows: Vec<Row>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Vocabulary {
    pub conds: Vec<Cond>,
    pub verbs: Vec<Verb>,
    pub max_rows: usize,
    /// Cut 8B §1: the combo table (`COMBOS`), so the editor can name a pair as it is written.
    #[serde(default)]
    pub combos: Vec<Combo>,
}

/// Cut 8B §1: a combo on the wire — two verb patterns (`shield_bash`, `drink unknown`) that
/// name adjacent rows, and the name the engine gives the pair (`opener`, `hit and fade`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Combo {
    pub a: String,
    pub b: String,
    pub name: String,
}

/// Cut 8B §1: a combo found in a set — the two row indices (0-based, adjacent) and its name.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ComboHit {
    pub rows: [usize; 2],
    pub name: String,
}

pub struct ComboDef {
    pub a: &'static str,
    pub b: &'static str,
    pub name: &'static str,
}

/// Cut 8B §1: adjacent-row verb pairs the engine resolves as one move, and their names. A
/// pattern is a verb key (`throw` matches any throw) or `verb arg` (`drink unknown`). The
/// first row's verb matches `a`, the next row's `b`. Names are engine data (≤ 3 words).
pub const COMBOS: &[ComboDef] = &[
    ComboDef { a: "shield_bash", b: "backstab", name: "opener" },
    ComboDef { a: "shield_bash", b: "attack", name: "opener" },
    ComboDef { a: "throw", b: "retreat", name: "hit and fade" },
    ComboDef { a: "throw", b: "back_corridor", name: "hit and fade" },
    ComboDef { a: "taunt", b: "cleave", name: "bait" },
    ComboDef { a: "shoot", b: "kite", name: "kite" },
    ComboDef { a: "shoot", b: "retreat", name: "kite" },
    ComboDef { a: "vanish", b: "backstab", name: "ambush" },
    ComboDef { a: "pray", b: "descend", name: "pilgrim" },
    ComboDef { a: "tame", b: "send", name: "handler" },
    ComboDef { a: "drink unknown", b: "attack", name: "gambler" },
    ComboDef { a: "back_corridor", b: "attack", name: "chokepoint" },
];

/// Does a combo pattern name this verb? `throw` matches every throw; `drink unknown` only the
/// gamble.
pub fn verb_is(pat: &str, v: &Verb) -> bool {
    match pat.split_once(' ') {
        Some((key, arg)) => v.v == key && v.a.as_deref() == Some(arg),
        None => v.v == pat,
    }
}

/// The combo two verbs make when written on adjacent rows (`a` above `b`), if any.
pub fn combo_name(a: &Verb, b: &Verb) -> Option<&'static str> {
    COMBOS.iter().find(|c| verb_is(c.a, a) && verb_is(c.b, b)).map(|c| c.name)
}

/// The table for the wire.
pub fn combo_table() -> Vec<Combo> {
    COMBOS.iter().map(|c| Combo { a: c.a.into(), b: c.b.into(), name: c.name.into() }).collect()
}

/// Every combo in a set, in row order (a row may open one combo and close another).
pub fn combos_in(rules: &RuleSet) -> Vec<ComboHit> {
    rules.rows.windows(2).enumerate().filter_map(|(i, w)| combo_name(&w[0].verb, &w[1].verb).map(|name| ComboHit { rows: [i, i + 1], name: name.into() })).collect()
}

/// A combo name as one word for the chronicle (`hit-and-fade`).
pub fn combo_slug(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join("-")
}

pub const COND_KEYS: &[&str] = &[
    "hp<", "hp>", "foes>=", "adj>=", "foe_tag", "foe_hp<", "item", "unknown_item", "floor_seen>=", "depth>=",
    "alert>=", "in_corridor", "path_stairs", "ally", "loot>=", "turns>", "on_hurt", "on_kill", "on_see",
    // Addendum A (party scope, companion self)
    "party", "party_hp<", "self_hp<", "self_hp>",
];

pub const VERB_KEYS: &[&str] = &[
    "attack", "retreat", "back_corridor", "drink", "read", "throw", "descend", "bank", "return", "rest",
    "pick_up", "free_captive", "shield_bash", "vanish", "tactic",
    // Addendum A (tame, party orders, companion verbs)
    "tame", "recall", "send", "shoot", "burst", "steal", "split", "flank", "drain", "follow",
    // Addendum C (class ladder)
    "cleave", "taunt", "second_wind", "bulwark", "backstab", "smoke", "ambush", "shadowstep",
    // Cut 2 §4 (ranger and caster ladders; `shoot` and `drain` above)
    "kite", "volley", "trap", "mark", "double_shot", "bolt", "ward", "blink", "slow", "nova",
    // hold position (always executes; not offered by the editor)
    "hold",
    // Cut 5 §4: the shrine (`pray row` / `pray trait`)
    "pray",
];

impl Cond {
    pub fn n(k: &str, n: i32) -> Cond {
        Cond { k: k.into(), n: Some(n), t: None }
    }
    pub fn t(k: &str, t: &str) -> Cond {
        Cond { k: k.into(), n: None, t: Some(t.into()) }
    }
    pub fn flag(k: &str) -> Cond {
        Cond { k: k.into(), n: None, t: None }
    }
    /// ≤ 2 words, for callouts.
    pub fn short(&self) -> String {
        let n = self.n.unwrap_or(0);
        match self.k.as_str() {
            "hp<" => format!("HP<{n}%"),
            "hp>" => format!("HP>{n}%"),
            "foes>=" => format!("foes {n}+"),
            "adj>=" => format!("adj {n}+"),
            "foe_tag" => self.t.clone().unwrap_or_default(),
            "foe_hp<" => format!("foe<{n}%"),
            "item" => format!("has {}", self.t.clone().unwrap_or_default()),
            "unknown_item" => "unknown".into(),
            "floor_seen>=" => format!("seen {n}%"),
            "depth>=" => format!("D{n}+"),
            "alert>=" => format!("alert {n}+"),
            "in_corridor" => "corridor".into(),
            "path_stairs" => "stairs".into(),
            "ally" => "ally".into(),
            "loot>=" => format!("loot {n}+"),
            "turns>" => format!("turns {n}+"),
            "on_hurt" => "hurt".into(),
            "on_kill" => "kill".into(),
            "on_see" => match &self.t {
                Some(t) => format!("see {t}"),
                None => "see".into(),
            },
            "party" => self.t.clone().unwrap_or_default(),
            "party_hp<" => format!("pet<{n}%"),
            "self_hp<" => format!("self<{n}%"),
            "self_hp>" => format!("self>{n}%"),
            other => other.to_string(),
        }
    }
    pub fn valid(&self) -> bool {
        COND_KEYS.contains(&self.k.as_str())
    }
}

impl Verb {
    pub fn new(v: &str) -> Verb {
        Verb { v: v.into(), a: None }
    }
    pub fn arg(v: &str, a: &str) -> Verb {
        Verb { v: v.into(), a: Some(a.into()) }
    }
    /// ≤ 2 words, for callouts.
    pub fn short(&self) -> String {
        let a = self.a.clone().unwrap_or_default();
        match self.v.as_str() {
            "attack" => match a.as_str() {
                "" | "nearest" => "attack".into(),
                "lowest" => "hit weakest".into(),
                s => format!("hit {}", s.trim_start_matches("tag:")),
            },
            "back_corridor" => "corridor".into(),
            "drink" | "read" | "throw" => {
                if a.is_empty() || a == "unknown" {
                    format!("{} ?", self.v)
                } else {
                    format!("{} {}", self.v, a.split(',').next().unwrap_or(""))
                }
            }
            "shield_bash" => "bash".into(),
            "free_captive" => "free".into(),
            "double_shot" => "double shot".into(),
            "tactic" => a.replace('_', " "),
            "pray" => format!("pray {a}").trim().to_string(),
            other => other.to_string(),
        }
    }
    pub fn valid(&self) -> bool {
        VERB_KEYS.contains(&self.v.as_str())
    }
}

impl Row {
    pub fn new(conds: Vec<Cond>, verb: Verb) -> Row {
        Row { conds, verb, origin: None }
    }
    /// The same row tagged with an origin (`preset` · `card` · `patch` · `player`).
    pub fn from(mut self, origin: &str) -> Row {
        self.origin = Some(origin.into());
        self
    }
    /// Callout text: first condition + verb, ≤ 3 words.
    pub fn text(&self, hp_pct: i32) -> String {
        let c = match self.conds.first() {
            None => "always".to_string(),
            Some(c) if c.k == "hp<" || c.k == "hp>" => format!("HP {hp_pct}%"),
            Some(c) => c.short(),
        };
        format!("{} → {}", c, self.verb.short())
    }
    pub fn describe(&self) -> String {
        let cs: Vec<String> = self.conds.iter().map(|c| c.short()).collect();
        let c = if cs.is_empty() { "always".to_string() } else { cs.join(" · ") };
        format!("{} → {}", c, self.verb.short())
    }
}

impl RuleSet {
    pub fn parse(text: &str) -> Result<RuleSet, String> {
        let set: RuleSet = serde_json::from_str(text).map_err(|e| e.to_string())?;
        set.validate()?;
        Ok(set)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.rows.len() > crate::engine::MAX_ROWS {
            return Err(format!("more than {} rows", crate::engine::MAX_ROWS));
        }
        for (i, r) in self.rows.iter().enumerate() {
            if r.conds.len() > 2 {
                return Err(format!("row {} has more than 2 conditions", i + 1));
            }
            for c in &r.conds {
                if !c.valid() {
                    return Err(format!("row {}: unknown condition {}", i + 1, c.k));
                }
            }
            if !r.verb.valid() {
                return Err(format!("row {}: unknown verb {}", i + 1, r.verb.v));
            }
        }
        Ok(())
    }
    pub fn to_text(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
    /// Cut 8B §1: the set's combos, in row order.
    pub fn combos(&self) -> Vec<ComboHit> {
        combos_in(self)
    }
}

pub fn word_count(s: &str) -> usize {
    s.split_whitespace().filter(|w| w.chars().any(|c| c.is_alphanumeric())).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wire_shapes() {
        let c: Cond = serde_json::from_str(r#"{"k":"hp<","n":40}"#).unwrap();
        assert_eq!(c, Cond::n("hp<", 40));
        assert_eq!(serde_json::to_string(&c).unwrap(), r#"{"k":"hp<","n":40}"#);
        let v: Verb = serde_json::from_str(r#"{"v":"attack","a":"tag:caster"}"#).unwrap();
        assert_eq!(serde_json::to_string(&v).unwrap(), r#"{"v":"attack","a":"tag:caster"}"#);
        let r = RuleSet::parse(r#"{"rows":[{"conds":[{"k":"foe_tag","t":"pack"}],"verb":{"v":"retreat"}}]}"#).unwrap();
        assert_eq!(r.rows[0].conds[0].t.as_deref(), Some("pack"));
        assert!(serde_json::to_string(&r).unwrap().contains(r#""verb":{"v":"retreat"}"#));
    }
    /// Cut 8B §1: combos are adjacent pairs from the table, in row order; a pattern with an
    /// argument matches only that argument; a card row is never part of one.
    #[test]
    fn combos_are_named_adjacent_pairs() {
        let rows = vec![
            Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::new("taunt")),
            Row::new(vec![Cond::n("adj>=", 2)], Verb::new("cleave")),
            Row::new(vec![Cond::n("foes>=", 3)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::flag("unknown_item")], Verb::arg("drink", "unknown")),
            Row::new(vec![], Verb::arg("tactic", "boss_focus")),
        ];
        let set = RuleSet { rows, name: None };
        let hits = set.combos();
        assert_eq!(hits.len(), 2, "{hits:?}");
        assert_eq!((hits[0].rows, hits[0].name.as_str()), ([1, 2], "bait"));
        assert_eq!((hits[1].rows, hits[1].name.as_str()), ([3, 4], "chokepoint"));
        // `drink heal → attack` is not the gamble; `throw fire → retreat` is a hit and fade.
        assert_eq!(combo_name(&Verb::arg("drink", "heal"), &Verb::arg("attack", "nearest")), None);
        assert_eq!(combo_name(&Verb::arg("drink", "unknown"), &Verb::arg("attack", "lowest")), Some("gambler"));
        assert_eq!(combo_name(&Verb::arg("throw", "fire,tag:boss"), &Verb::new("retreat")), Some("hit and fade"));
        assert_eq!(combo_name(&Verb::new("shield_bash"), &Verb::new("backstab")), Some("opener"));
        for c in COMBOS {
            assert!(word_count(c.name) <= 3, "{}", c.name);
            assert!(VERB_KEYS.contains(&c.a.split(' ').next().unwrap()) && VERB_KEYS.contains(&c.b.split(' ').next().unwrap()), "{} → {}", c.a, c.b);
        }
        assert_eq!(combo_slug("hit and fade"), "hit-and-fade");
        assert_eq!(combo_table().len(), COMBOS.len());
    }
    #[test]
    fn validation() {
        assert!(RuleSet::parse(r#"{"rows":[{"conds":[],"verb":{"v":"fly"}}]}"#).is_err());
        let many = r#"{"rows":[{"conds":[{"k":"hp<","n":1},{"k":"hp<","n":2},{"k":"hp<","n":3}],"verb":{"v":"rest"}}]}"#;
        assert!(RuleSet::parse(many).is_err());
    }
    #[test]
    fn callout_text_three_words() {
        let r = Row::new(vec![Cond::n("hp<", 40), Cond::n("foes>=", 2)], Verb::new("back_corridor"));
        let t = r.text(31);
        assert_eq!(t, "HP 31% → corridor");
        assert!(word_count(&t) <= 3);
        let r2 = Row::new(vec![Cond::t("foe_tag", "caster")], Verb::arg("attack", "tag:caster"));
        assert!(word_count(&r2.text(90)) <= 3, "{}", r2.text(90));
        let r3 = Row::new(vec![], Verb::arg("drink", "unknown"));
        assert!(word_count(&r3.text(90)) <= 3, "{}", r3.text(90));
    }
}
