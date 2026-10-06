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
    /// Cut 26 §2: the set's route — the fork depths whose far stair the hero takes (`[5]`: the
    /// Fens at D5–8, the Burrows at D9–13; `descent::Route`). Written, never steered; empty is
    /// the near stair at every fork (the base order).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub route: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Vocabulary {
    pub conds: Vec<Cond>,
    pub verbs: Vec<Verb>,
    pub max_rows: usize,
    /// Cut 8B §1: the combo table (`COMBOS`), so the editor can name a pair as it is written.
    #[serde(default)]
    pub combos: Vec<Combo>,
    /// Cut 9 §1: condition tokens that exist but are gated for this lineage, each with the
    /// gate as the player reads it (`fact: pack`, `◆2`, `see: stray`, `tame a foe`). The sheet
    /// shows them dim; `set_rules` refuses a row that uses one.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub locked: Vec<LockedCond>,
    /// Cut 21 §3 (AE banked at D20; the picker stopped at 12): the deepest `depth ≥` the
    /// picker offers — the lineage's best + 2, never under 8 (every depth from 2 up to it).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub depth_max: u32,
    /// Cut 23 §3: every reason the core gives for a row not acting → its reason on tap, ≤ 3
    /// words (`turn::WHY_GLOSS`; keys are reason prefixes). Filled on the wire's vocabulary
    /// (`Game::vocabulary`), empty on stored copies.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub why_gloss: std::collections::BTreeMap<String, String>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Cut 9 §1: a gated condition token and its ≤ 3-word gate.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct LockedCond {
    pub cond: Cond,
    pub needs: String,
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

/// Cut 30 §2: the origins of package rows (`Row::is_pkg`).
pub const PKG_ORIGINS: [&str; 5] = ["stance:", "tactic:", "temper:", "drill:", "style:"];

pub const COND_KEYS: &[&str] = &[
    "hp<", "hp>", "foes>=", "adj>=", "foe_tag", "foe_hp<", "item", "unknown_item", "floor_seen>=", "depth>=",
    "alert>=", "in_corridor", "path_stairs", "ally", "loot>=", "turns>", "on_hurt", "on_kill", "on_see",
    // Addendum A (party scope, companion self)
    "party", "party_hp<", "self_hp<", "self_hp>",
    // Cut 26 §2: the biome the floor sits in (`in: fens`), open once the biome has been entered
    "in",
    // Cut 30 §4: the heir wears a trait (`trait: wrathful`); a worn gift's context holds now
    "trait", "gift_live",
    // Cut 30.5 (the owner: a run ends when he is out of supplies): no `t` in the pack (`lacks heal`)
    "lacks",
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
    "riposte", "hex",
    "fire", "reload", "close_burst",
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
            "lacks" => format!("no {}", self.t.clone().unwrap_or_default()),
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
            "in" => format!("in {}", self.t.clone().unwrap_or_default()),
            "trait" => self.t.clone().unwrap_or_default(),
            "gift_live" => "gift live".into(),
            other => other.to_string(),
        }
    }
    pub fn valid(&self) -> bool {
        COND_KEYS.contains(&self.k.as_str())
    }
    /// Cut 9 §1: the same token — key and tag; the number is the player's to edit.
    pub fn same_token(&self, o: &Cond) -> bool {
        self.k == o.k && self.t == o.t
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
            "close_burst" => "burst".into(),
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

impl Row {
    /// Cut 12 §1: a card's row (`tactic <card>`) — it sits outside the player's row cap, one
    /// per owned card.
    pub fn is_card(&self) -> bool {
        self.verb.v == "tactic"
    }
    /// Cut 30 §2: a package's row (compiled from a stance, a tactic, a temperament or a drill:
    /// `stance:steady`, `drill:lich`) — outside the player's row cap, like a card's.
    pub fn is_pkg(&self) -> bool {
        self.origin.as_deref().is_some_and(|o| PKG_ORIGINS.iter().any(|p| o.starts_with(p)))
    }
    /// The card a card row carries (`thief_guard`), if it is one.
    pub fn card(&self) -> Option<&str> {
        self.is_card().then(|| self.verb.a.as_deref().unwrap_or("")).filter(|c| !c.is_empty())
    }
}

impl RuleSet {
    pub fn parse(text: &str) -> Result<RuleSet, String> {
        let set: RuleSet = serde_json::from_str(text).map_err(|e| e.to_string())?;
        set.validate()?;
        Ok(set)
    }
    /// Cut 12 §1: the player's own rows — every row that is not a card's.
    pub fn own_rows(&self) -> usize {
        self.rows.iter().filter(|r| !r.is_card() && !r.is_pkg()).count()
    }
    /// Cut 12 §1: the card rows (one per card once validated).
    pub fn card_rows(&self) -> usize {
        self.rows.iter().filter(|r| r.is_card()).count()
    }
    /// Cut 12 §1: the rows in play under a cap on the player's own rows — every card row
    /// (the first per card) and the first `max_rows` own rows, in the set's order, with their
    /// indices in the set. A validated set passes through whole; a patched or replayed one
    /// is cut like the editor cuts it (the last own row falls off).
    /// QA on 92eb880 (qaM: `R3 fired 0 of 16 runs: hp < 30% → drink heal · heal unknown` under
    /// R1 `hp < 30% → return`; a new row `hp < 50% → attack nearest` under `foes ≥ 1 → attack
    /// nearest`, neither marked): per row of the set (by index), the earlier row in play that
    /// takes every moment it could fire — `None` when none does, or the row is not in play.
    /// Row A shadows a later row B when every condition of A holds whenever B's do (B's own
    /// conditions, plus a foe in view when B's verb strikes one), A's conditions are all
    /// usable (`usable`: owned, not locked), and A acts whenever it holds: its verb always
    /// executes (`hold`; Cut 19 §2: a `return` walks and can be blocked), or it is B's verb with the same scope (where A's attempt
    /// fails, B's same attempt fails too).
    pub fn shadowed_by(&self, max_rows: usize, usable: impl Fn(&Cond) -> bool) -> Vec<Option<usize>> {
        let act: Vec<(usize, &Row)> = self.active(max_rows).collect();
        let mut out = vec![None; self.rows.len()];
        for (k, &(j, b)) in act.iter().enumerate() {
            out[j] = act[..k].iter().find(|(_, a)| shadows(a, b, &usable)).map(|(i, _)| *i);
        }
        out
    }

    pub fn active(&self, max_rows: usize) -> impl Iterator<Item = (usize, &Row)> + '_ {
        let mut own = 0;
        let mut cards: Vec<&str> = Vec::new();
        self.rows.iter().enumerate().filter(move |(_, r)| match r.card() {
            Some(c) => {
                if cards.contains(&c) {
                    false
                } else {
                    cards.push(c);
                    true
                }
            }
            None if r.is_pkg() => true,
            None => {
                own += 1;
                own <= max_rows
            }
        })
    }
    /// Cut 12 §1: the set cut to `max_rows` own rows (`active`, materialised).
    pub fn fit(&self, max_rows: usize) -> RuleSet {
        RuleSet { rows: self.active(max_rows).map(|(_, r)| r.clone()).collect(), name: self.name.clone(), route: self.route.clone() }
    }
    /// Cut 12 §1: the door's check for a player's set — at most `max_rows` own rows (a card's
    /// row is the card's, outside the cap), one row per card, every card owned. Errors ≤ 6
    /// words, the number first (`5 own rows, 4 allowed`).
    pub fn check_rows(&self, max_rows: usize, owned: &std::collections::BTreeSet<String>) -> Result<(), String> {
        let own = self.own_rows();
        if own > max_rows {
            return Err(format!("{own} own rows, {max_rows} allowed"));
        }
        let mut cards: Vec<&str> = Vec::new();
        for r in &self.rows {
            let Some(c) = r.card() else { continue };
            if cards.contains(&c) {
                return Err(format!("two rows for {}", c.replace('_', " ")));
            }
            if !owned.contains(c) {
                return Err(format!("card not owned: {}", c.replace('_', " ")));
            }
            cards.push(c);
        }
        Ok(())
    }
    /// Cut 26 §2: the set's route (`descent::Route`; an invalid list reads as the base order —
    /// `validate` refuses it at the door).
    pub fn route(&self) -> crate::descent::Route {
        crate::descent::Route::from_forks(&self.route).unwrap_or_default()
    }
    /// The same set taking `route`.
    pub fn with_route(mut self, route: crate::descent::Route) -> RuleSet {
        self.route = route.forks();
        self
    }
    pub fn validate(&self) -> Result<(), String> {
        crate::descent::Route::from_forks(&self.route)?;
        if self.own_rows() > crate::engine::MAX_ROWS {
            return Err(format!("more than {} rows", crate::engine::MAX_ROWS));
        }
        if self.rows.len() > crate::engine::ROWS_TOTAL {
            return Err(format!("more than {} rows with cards", crate::engine::ROWS_TOTAL));
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

/// QA on 92eb880: does `a` (above) take every moment `b` (below) could fire? See
/// `RuleSet::shadowed_by`.
pub fn shadows(a: &Row, b: &Row, usable: &impl Fn(&Cond) -> bool) -> bool {
    if !a.conds.iter().all(usable) {
        return false;
    }
    // What changes how a verb acts besides its argument: the party scope and the den raid.
    fn scope(r: &Row) -> Vec<&Cond> {
        let mut v: Vec<&Cond> = r.conds.iter().filter(|c| c.k == "party" || (c.k == "on_see" && c.t.as_deref() == Some("den")) || (c.k == "foe_tag" && c.t.as_deref() == Some("thief"))).collect();
        v.sort_by(|x, y| (&x.k, &x.t, x.n).cmp(&(&y.k, &y.t, y.n)));
        v.dedup();
        v
    }
    // Cut 19 §2: `return` walks to the stairs like `bank` and fails when a foe stands in the
    // way, so only `hold` always acts.
    let always = a.verb.v == "hold";
    if !(always || (a.verb == b.verb && scope(a) == scope(b))) {
        return false;
    }
    // B's moments: its conditions, and a foe in view when its verb strikes one or a
    // condition reads one.
    let foe = (matches!(b.verb.v.as_str(), "attack" | "shield_bash" | "cleave" | "backstab" | "taunt") || b.conds.iter().any(|c| matches!(c.k.as_str(), "foe_tag" | "foe_hp<"))).then(|| Cond::n("foes>=", 1));
    a.conds.iter().all(|ca| b.conds.iter().chain(foe.as_ref()).any(|cb| cond_implies(cb, ca)))
}

/// Whenever `b` holds, `a` holds (the same token, a threshold at least as strict).
pub fn cond_implies(b: &Cond, a: &Cond) -> bool {
    if !b.same_token(a) {
        return false;
    }
    let (nb, na) = (b.n.unwrap_or(0), a.n.unwrap_or(0));
    match a.k.as_str() {
        "hp<" | "foe_hp<" | "party_hp<" | "self_hp<" => nb <= na,
        "hp>" | "self_hp>" | "turns>" | "foes>=" | "adj>=" | "floor_seen>=" | "depth>=" | "alert>=" | "loot>=" => nb >= na,
        _ => nb == na,
    }
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
        let set = RuleSet { rows, name: None, route: Vec::new() };
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
