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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Row {
    pub conds: Vec<Cond>,
    pub verb: Verb,
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
        Row { conds, verb }
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
