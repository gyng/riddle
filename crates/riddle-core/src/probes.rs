//! Presets, bots and companion defaults used by the examples and tests.
use crate::hero::Class;
use crate::rng::Rng;
use crate::rules::{Cond, Row, RuleSet, Verb, Vocabulary};

/// The shipped class preset (DEFAULT bot). Cut 7 §2: two rows for the fighter (the editor
/// opens with two empty slots), each tagged `origin: "preset"`.
pub fn preset(class: Class) -> RuleSet {
    let mut set = preset_rows(class);
    for r in set.rows.iter_mut() {
        r.origin = Some("preset".into());
    }
    set
}

fn preset_rows(class: Class) -> RuleSet {
    match class {
        Class::Gunner => RuleSet {
            name:Some("gunner".into()),route:Vec::new(),rows:vec![
                Row::new(vec![Cond::n("hp<",30)],Verb::arg("drink","heal")),
                Row::new(vec![],Verb::new("reload")),
                Row::new(vec![Cond::n("foes>=",1)],Verb::arg("fire","nearest")),
            ],
        },
        Class::Fighter => RuleSet {
            name: Some("fighter".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            ],
            route: Vec::new(),
        },
        Class::Rogue => RuleSet {
            name: Some("rogue".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("hp<", 40), Cond::n("foes>=", 2)], Verb::new("vanish")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            ],
            route: Vec::new(),
        },
        Class::Ranger => RuleSet {
            name: Some("ranger".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("adj>=", 1)], Verb::new("kite")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("shoot", "nearest")),
            ],
            route: Vec::new(),
        },
        Class::Caster => RuleSet {
            name: Some("caster".into()),
            rows: vec![
                Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
                Row::new(vec![Cond::n("adj>=", 1)], Verb::new("ward")),
                Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("bolt", "nearest")),
            ],
            route: Vec::new(),
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
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 9)], Verb::arg("throw", "fire,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
            Row::new(vec![Cond::n("foes>=", 3), Cond::n("hp<", 70)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![Cond::n("floor_seen>=", 60)], Verb::new("descend")),
        ],
        route: Vec::new(),
    }
}

/// Cut 3: the shipped best set (FULL bot) — every boss counter, every unlock assumed. Shipped
/// as presets/full.json. Rows 4–6 are the Cut 3 boss counters (the gate removes each in turn):
/// `cadence` for the Mirror King (Cut 7: D33), `silence` for the Lurker Queen (D28; read when
/// her called lurkers show, before she is seen), `reflect_read` for the Foundry Master (D23;
/// smiths first).
pub fn full() -> RuleSet {
    RuleSet {
        name: Some("full".into()),
        rows: vec![
            Row::new(vec![Cond::n("hp<", 35)], Verb::arg("drink", "heal")),
            // QA on 1a2a4a9: a return is a commitment (`Run.homeward`) — the old `hp < 40 % ·
            // D5+ → return` went home from every scratch the heal had not answered (FULL ≥ D29
            // 63 % → 43 %). It held the walls by walking off and turning back; a retreat does
            // that now, and the return is the way out at 20 %.
            Row::new(vec![Cond::n("hp<", 40), Cond::n("depth>=", 5)], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 20), Cond::n("depth>=", 5)], Verb::new("return")),
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 33)], Verb::arg("tactic", "cadence")),
            Row::new(vec![Cond::t("foe_tag", "summoned"), Cond::n("depth>=", 28)], Verb::arg("read", "silence")),
            Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")),
            Row::new(vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 19)], Verb::arg("attack", "tag:buffer")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("tactic", "boss_focus")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
            Row::new(vec![Cond::n("foes>=", 3), Cond::n("hp<", 70)], Verb::new("back_corridor")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::arg("tactic", "noise_discipline")),
        ],
        route: Vec::new(),
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
        route: Vec::new(),
    }
}

/// TRIVIAL plus the boss counters: range for the Bloat Mother, the Warlord himself, the
/// Lich's summons first (COUNTERED bot). Cut 7: and the crypt's hunger answered (a player
/// who reads the D12 fact lights the shrine; the D9 captive is cut down, the coward's way).
pub fn countered() -> RuleSet {
    RuleSet {
        name: Some("countered".into()),
        rows: vec![
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 9)], Verb::arg("throw", "fire,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 9)], Verb::arg("throw", "poison,tag:boss")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
            Row::new(vec![Cond::t("on_see", "hunger")], Verb::arg("pray", "row")),
            Row::new(vec![], Verb::arg("tame", "nearest")),
            Row::new(vec![Cond::n("hp<", 50), Cond::n("foes>=", 1)], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
            Row::new(vec![], Verb::arg("attack", "nearest")),
        ],
        route: Vec::new(),
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
    RuleSet { rows: out, name: Some("random".into()), route: Vec::new() }
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
    RuleSet { rows, name: None, route: Vec::new() }
}

/// Give a game every fact (LEARNED bot).
pub fn learn_everything(g: &mut crate::engine::Game) {
    for m in crate::defs::MONSTERS {
        g.lineage.facts.insert(format!("foe:{}", m.kind));
        for t in m.tags {
            g.lineage.facts.insert(format!("foe:{}:{}", m.kind, t));
        }
        if m.boss {
            g.lineage.facts.insert(crate::facts::boss_counter_fact(m.kind));
        }
        g.lineage.facts.insert(format!("foe:{}:studied", m.kind));
    }
    g.lineage.facts.insert("alert:rising".into());
    for k in ["goblin", "skeleton", "lurker"] {
        g.lineage.facts.insert(format!("foe:{k}:summoned"));
    }
    for i in crate::defs::ITEMS {
        if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, i.kind) {
            g.lineage.facts.insert(f);
        }
    }
    for b in crate::descent::Biome::ALL {
        g.lineage.facts.insert(format!("biome:{}", b.name()));
    }
    for k in crate::defs::FACT_MISC {
        g.lineage.facts.insert(format!("item:{k}"));
    }
    g.lineage.facts.insert("item:recall".into());
    for (a, b, _) in crate::defs::COUNTERS {
        g.lineage.facts.insert(crate::defs::counter_fact(a, b));
    }
    g.lineage.facts.insert("item:leash".into());
}

/// Cut 7 §3: the one-row answer to a band situation (the fighter's vocabulary).
pub fn situation_answer(what: &str) -> Row {
    match what {
        "den" => Row::new(vec![Cond::t("on_see", "den")], Verb::arg("attack", "nearest")),
        "lock" => Row::new(vec![Cond::t("foe_tag", "gas"), Cond::n("adj>=", 1)], Verb::new("retreat")),
        "captive" => Row::new(vec![Cond::t("on_see", "captive")], Verb::new("free_captive")),
        // Cut 8B §3: the stray takes the kennel's leash.
        "stray" => Row::new(vec![Cond::t("on_see", "stray")], Verb::arg("tame", "nearest")),
        _ => Row::new(vec![Cond::t("on_see", "hunger")], Verb::arg("pray", "row")),
    }
}

/// Cut 7 §3 gate probe: a fighter shaped for the band's depth (levelled, in mail below the
/// Warrens' doorstep) starts on the situation's floor with the shipped preset — plus the
/// one-row answer when `answered` — and plays the floor out. Returns (met, passed, left the
/// floor alive).
pub fn situation_trial(seed: u64, what: &str, answered: bool) -> (bool, bool, bool) {
    let depth = crate::situations::depth_of(what).expect("a band situation");
    let mut set = preset(Class::Fighter);
    if answered {
        set.rows.insert(0, situation_answer(what));
    }
    let g = floor_trial_twist(seed, depth, set, Some(what));
    let run = g.run.as_ref().expect("the run is live or over");
    let met = run.situations.iter().any(|(_, s)| s == what);
    let passed = run.passed.iter().any(|s| s == what);
    (met, passed, run.depth > depth)
}

/// Cut 12 §2 gate probe: the den's snatches on one seed — the shipped preset alone, and the
/// preset with the thief guard card's bare row where `buy` puts it (before `attack nearest`).
/// Returns (snatches without the card, snatches with it). Cut 20 §1: the den's own snatches
/// (`Run.den_snatches`) — with one theft a run, a wandering monkey's theft on the floor (the
/// same tick with the card or without) was a third of the count and no den's.
pub fn den_guard_trial(seed: u64) -> (u32, u32) {
    let set = preset(Class::Fighter);
    let g = floor_trial_with(seed, 3, set.clone(), Some("den"), &[]);
    let without = g.run.as_ref().map(|r| r.den_snatches).unwrap_or(0);
    let mut carded = set;
    let at = crate::meta::card_insert_at(&carded);
    carded.rows.insert(at, Row::new(vec![], Verb::arg("tactic", "thief_guard")).from("card"));
    let g = floor_trial_with(seed, 3, carded, Some("den"), &["thief_guard"]);
    let with = g.run.as_ref().map(|r| r.den_snatches).unwrap_or(0);
    (without, with)
}

/// Cut 12 §4 gate probe: the situation words of D3–10 on one seed (a fresh run walked down
/// the stairs), for the distinct-kinds and no-repeat bars.
pub fn twist_sequence(seed: u64) -> Vec<Option<String>> {
    let mut g = crate::engine::Game::new_literal(seed);
    g.sim = true;
    g.start_run(Some(seed));
    let mut out = Vec::new();
    for d in 3..=10u32 {
        g.descend_to(d);
        out.push(g.run.as_ref().and_then(|r| (*r.floor_twist).clone()));
    }
    out
}

/// Cut 7: a shaped fighter plays one floor with `set` (the situation trial's engine; also the
/// Captain's floor measure). Returns the game with the run live or over.
pub fn floor_trial(seed: u64, depth: u32, set: RuleSet) -> crate::engine::Game {
    floor_trial_twist(seed, depth, set, None)
}

/// Cut 12 §4: `floor_trial` with the floor's situation chosen (`twist`), since a band's
/// situation now sits on a floor drawn from the run's seed.
pub fn floor_trial_twist(seed: u64, depth: u32, set: RuleSet, twist: Option<&str>) -> crate::engine::Game {
    floor_trial_with(seed, depth, set, twist, &[])
}

/// Cut 12 §2: `floor_trial_twist` on a lineage that owns `unlocks` (a card's trial).
pub fn floor_trial_with(seed: u64, depth: u32, set: RuleSet, twist: Option<&str>, unlocks: &[&str]) -> crate::engine::Game {
    let mut g = crate::engine::Game::new_literal(seed);
    for f in ["den", "lock", "captive", "hunger", "shrine", "foe:bloat:gas", "foe:monkey:thief"] {
        g.lineage.facts.insert(f.into());
    }
    for u in unlocks {
        g.lineage.unlocks.insert((*u).into());
    }
    g.set_rules(set).expect("trial rules");
    let level = match depth {
        0..=3 => 1,
        4..=6 => 2,
        7..=9 => 4,
        _ => 6,
    };
    g.lineage.classes.insert("fighter".into(), crate::wire::ClassProg { level, xp: 0, next: 0 });
    if depth >= 6 {
        let id = g.lineage.next_vault_id;
        g.lineage.next_vault_id += 1;
        let mut it = crate::item::Item::new(id, "mail");
        it.enchant = 1;
        g.lineage.vault.push(it);
        g.loadout(vec![id]);
    }
    g.sim = true;
    g.start_run(Some(seed ^ 0x51));
    match twist {
        Some(t) => g.descend_to_twist(depth, t),
        None => g.descend_to(depth),
    }
    let mut n = 0;
    while g.run.as_ref().is_some_and(|r| r.over.is_none() && r.depth == depth) && n < 6000 {
        g.tick();
        g.events.clear();
        n += 1;
    }
    g
}

/// Cut 10 §2: the sims a `try` row's forecast runs (the gate's own number; the camp's
/// forecast runs `FORECAST_SIMS`).
pub const COUNTER_TRIAL_SIMS: u32 = 16;

/// Cut 10 §2 gate probe: a lineage that has met the Warlord (best D8, his counter fact
/// known, level 4 in +1 mail) with `good.json` minus its boss rows — a set that walks to D8
/// and cannot pass him. Returns (the forecast's D9 row names the counter, D9 reach as is,
/// with the counter row at the top, with it at the end).
pub fn counter_trial(seed: u64) -> (bool, f64, f64, f64) {
    let (mut g, set) = warlord_lineage(seed);
    let reach9 = |g: &mut crate::engine::Game, set: RuleSet| -> crate::wire::ForecastDepth {
        g.set_rules(set).expect("trial rules");
        let f = crate::forecast::forecast_with(g, g.lineage.rules(), COUNTER_TRIAL_SIMS);
        f.depths.into_iter().find(|d| d.depth == 9).expect("known to D9")
    };
    let base = reach9(&mut g, set.clone());
    let counter = crate::facts::counter_row("goblin_warlord");
    let named = base.try_.as_ref().is_some_and(|t| t.row == counter && t.boss == "goblin_warlord" && t.text == "attack boss");
    let mut top = set.clone();
    top.rows.insert(0, counter.clone());
    let with_top = reach9(&mut g, top);
    let mut end = set;
    end.rows.push(counter);
    let with_end = reach9(&mut g, end);
    (named, base.reach, with_top.reach, with_end.reach)
}

/// Cut 18 §3 gate probe: the counter trial's lineage — the forecast's D8 and D9 rows for the
/// set without the counter row, then with it at the top.
pub fn wall_trial(seed: u64) -> [(crate::wire::ForecastDepth, crate::wire::ForecastDepth); 2] {
    let (mut g, set) = warlord_lineage(seed);
    let mut top = set.clone();
    top.rows.insert(0, crate::facts::counter_row("goblin_warlord"));
    [set, top].map(|s| {
        g.set_rules(s).expect("trial rules");
        let f = crate::forecast::forecast_with(&g, g.lineage.rules(), COUNTER_TRIAL_SIMS);
        let d = |n: u32| f.depths.iter().find(|d| d.depth == n).cloned().expect("known to D9");
        (d(8), d(9))
    })
}

/// The counter trial's lineage (best D8, the Warlord's counter known, fighter 4 in +1 mail)
/// and its set (`good.json` minus the boss rows).
fn warlord_lineage(seed: u64) -> (crate::engine::Game, RuleSet) {
    let mut g = crate::engine::Game::new_literal(seed);
    g.lineage.best_depth = 8;
    for u in ["row5", "row6", "row7", "row8"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.lineage.classes.insert("fighter".into(), crate::wire::ClassProg { level: 4, xp: 0, next: 0 });
    g.lineage.facts.insert(crate::facts::boss_counter_fact("goblin_warlord"));
    g.lineage.facts.insert("foe:goblin_warlord:boss".into());
    if let Some(f) = crate::item::ident_fact(&g.lineage.flavours, "heal") {
        g.lineage.facts.insert(f);
    }
    let id = g.lineage.next_vault_id;
    g.lineage.next_vault_id += 1;
    let mut it = crate::item::Item::new(id, "mail");
    it.enchant = 1;
    g.lineage.vault.push(it);
    g.loadout(vec![id]);
    let mut set = good();
    set.rows.retain(|r| !matches!(r.verb.a.as_deref(), Some("tag:boss") | Some("fire,tag:boss")));
    (g, set)
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
            life: Default::default(),
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
        assert!(full().validate().is_ok());
        // Cut 12 §1: the cap is on own rows; a card's row sits outside it (FULL: 7 + 4 cards).
        assert!(full().own_rows() <= crate::engine::MAX_ROWS);
        assert!(full().rows.len() <= crate::engine::ROWS_TOTAL);
    }
    #[test]
    fn companion_rows_capped_by_level() {
        let r = default_companion_rules(&["ranged".into(), "gas".into()], 1);
        assert_eq!(r.rows.len(), 2);
        let r = default_companion_rules(&["ranged".into(), "gas".into()], 3);
        assert_eq!(r.rows.len(), 4);
    }
}
