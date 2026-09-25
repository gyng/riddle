//! Meta: unlock catalogue (Cut 2 §3), purchases, pending decisions.
use crate::engine::{Game, LineageState};
use crate::facts::{has_boss_counter, has_tag_fact};
use crate::rules::{Cond, Row, Verb};
use crate::wire::UnlockInfo;

pub struct UnlockDef {
    pub id: &'static str,
    pub cost: u32,
    pub prereq: Option<&'static str>,
}

/// The catalogue, in the contract's order. Costs in marks; `needs` (fact/trophy gates) are
/// in `gate` below and surfaced as `UnlockInfo.needs`.
pub const UNLOCKS: &[UnlockDef] = &[
    UnlockDef { id: "row5", cost: 2, prereq: None },
    UnlockDef { id: "row6", cost: 4, prereq: Some("row5") },
    UnlockDef { id: "row7", cost: 7, prereq: Some("row6") },
    UnlockDef { id: "row8", cost: 11, prereq: Some("row7") },
    UnlockDef { id: "party_slot_2", cost: 4, prereq: None },
    UnlockDef { id: "party_slot_3", cost: 9, prereq: Some("party_slot_2") },
    UnlockDef { id: "vault2", cost: 3, prereq: None },
    UnlockDef { id: "vault3", cost: 6, prereq: Some("vault2") },
    // Cut 24: 9 (was 10) — the dayplayer's deeper lineages held 9 marks with the rest gated.
    UnlockDef { id: "vault4", cost: 9, prereq: Some("vault3") },
    // Cut 8B §2: the rogue is free at the first bank (a second class in the first hour).
    UnlockDef { id: "rogue", cost: 0, prereq: None },
    UnlockDef { id: "ranger", cost: 6, prereq: None },
    UnlockDef { id: "caster", cost: 8, prereq: None },
    // Cut 8B §3: `tame` is owned from the start (the kennel's leash is on the shelf); cost 0.
    UnlockDef { id: "tame", cost: 0, prereq: None },
    UnlockDef { id: "throw", cost: 2, prereq: None },
    UnlockDef { id: "cond_alert", cost: 2, prereq: None },
    UnlockDef { id: "cond_turns", cost: 2, prereq: None },
    UnlockDef { id: "cond_loot", cost: 2, prereq: None },
    UnlockDef { id: "cond_on_kill", cost: 2, prereq: None },
    UnlockDef { id: "cond_on_see", cost: 2, prereq: None },
    UnlockDef { id: "cond_party_hp", cost: 2, prereq: None },
    UnlockDef { id: "corridor_fighting", cost: 3, prereq: None },
    // Cut 23 §3 (AJ: "paid cards are rows he could type"): a card whose rows are all in the
    // typed vocabulary is free (`card_carries`: kite archers, stair dance, noise discipline,
    // deep march); the paid ones carry a verb or a bound target no row can.
    UnlockDef { id: "kite_archers", cost: 0, prereq: None },
    UnlockDef { id: "stair_dance", cost: 0, prereq: None },
    UnlockDef { id: "gas_step", cost: 3, prereq: None },
    UnlockDef { id: "pack_break", cost: 3, prereq: None },
    UnlockDef { id: "thief_guard", cost: 3, prereq: None },
    UnlockDef { id: "boss_focus", cost: 3, prereq: None },
    UnlockDef { id: "last_stand", cost: 3, prereq: None },
    UnlockDef { id: "quartermaster", cost: 5, prereq: None },
    UnlockDef { id: "auto_insure", cost: 6, prereq: None },
    UnlockDef { id: "incubator", cost: 4, prereq: None },
    UnlockDef { id: "supply_cap_5", cost: 3, prereq: None },
    UnlockDef { id: "bone_sense", cost: 3, prereq: None },
    UnlockDef { id: "third_tag", cost: 6, prereq: None },
    // Cut 3: tier 2 (needs a boss). Cut 4: 8–12 (income after the cheap catalogue is ~2
    // marks a day plus a first-bank mark per depth; 14–18 stalled the fortnight's purchases).
    UnlockDef { id: "row9", cost: 8, prereq: Some("row8") },
    UnlockDef { id: "row10", cost: 12, prereq: Some("row9") },
    UnlockDef { id: "vault5", cost: 8, prereq: Some("vault4") },
    UnlockDef { id: "party_slot_4", cost: 8, prereq: Some("party_slot_3") },
    UnlockDef { id: "cadence", cost: 5, prereq: None },
    UnlockDef { id: "noise_discipline", cost: 0, prereq: None },
    UnlockDef { id: "reflect_read", cost: 5, prereq: None },
    UnlockDef { id: "deep_march", cost: 0, prereq: None },
    UnlockDef { id: "lantern_rig", cost: 6, prereq: None },
    UnlockDef { id: "recall_sense", cost: 8, prereq: None },
];

/// The eight tactic cards (Cut 2 §3) plus the four mastery cards (class L10).
pub const TACTIC_CARDS: [&str; 8] = ["corridor_fighting", "kite_archers", "stair_dance", "gas_step", "pack_break", "thief_guard", "boss_focus", "last_stand"];
pub const MASTERY_CARDS: [&str; 4] = ["phalanx", "hit_and_fade", "hawkeye", "archmage"];
/// Cut 3: the tier-2 tactic cards.
pub const TIER2_CARDS: [&str; 4] = ["cadence", "noise_discipline", "reflect_read", "deep_march"];

/// Condition tokens that are unlocks (Cut 2 §3): (token, unlock id).
pub const COND_UNLOCKS: [(&str, &str); 6] = [
    ("alert>=", "cond_alert"),
    ("turns>", "cond_turns"),
    ("loot>=", "cond_loot"),
    ("on_kill", "cond_on_kill"),
    ("on_see", "cond_on_see"),
    ("party_hp<", "cond_party_hp"),
];

/// The unlock a condition token needs, if any.
pub fn cond_unlock(k: &str) -> Option<&'static str> {
    COND_UNLOCKS.iter().find(|(t, _)| *t == k).map(|(_, u)| *u)
}

/// An unlock's cost in marks (0 for an unknown id).
pub fn unlock_cost(id: &str) -> u32 {
    UNLOCKS.iter().find(|u| u.id == id).map(|u| u.cost).unwrap_or(0)
}

/// The fact/trophy gate of an unlock: `None` when open, else the human-readable need.
pub fn gate(l: &LineageState, id: &str) -> Option<String> {
    let need = |ok: bool, text: &str| if ok { None } else { Some(text.to_string()) };
    match id {
        // Cut 9 §2/§10: the party slots read as the player does (`tame once`).
        "party_slot_2" => need(l.tamed_kinds() >= 1, "tame once"),
        "party_slot_3" => need(l.tamed_kinds() >= 3, "tame 3 kinds"),
        "rogue" => need(!l.banked_depths.is_empty(), "bank once"),
        "ranger" => need(l.bosses_slain() >= 1, "slay a boss"),
        // Cut 25 §6 (AM: `+1 row ⊘ slay 3 bosses` locked after four Warlord kills): the count is of
        // boss kinds slain (`LineageState::kills` holds kinds) — the copy says so.
        "caster" => need(l.bosses_slain() >= 2, "2 boss kinds"),
        "tame" => need(l.facts.contains("item:leash"), "find a leash"),
        "cond_alert" => need(l.facts.contains("alert:rising"), "see alert rise"),
        "cond_on_kill" => need(!l.kills.is_empty(), "a kill"),
        "cond_on_see" => need(l.facts.iter().any(|f| f.starts_with("foe:")), "meet a foe"),
        "cond_party_hp" => need(l.tamed_kinds() >= 1, "tame once"),
        "corridor_fighting" => need(has_tag_fact(&l.facts, "pack"), "fact: pack"),
        "kite_archers" => need(has_tag_fact(&l.facts, "ranged"), "fact: ranged"),
        "stair_dance" => need(has_boss_counter(&l.facts), "a boss counter"),
        "gas_step" => need(has_tag_fact(&l.facts, "gas"), "fact: gas"),
        "pack_break" => need(has_tag_fact(&l.facts, "fast"), "fact: fast"),
        "thief_guard" => need(has_tag_fact(&l.facts, "thief"), "fact: thief"),
        "boss_focus" => need(has_tag_fact(&l.facts, "boss"), "fact: boss"),
        "last_stand" => need(has_tag_fact(&l.facts, "heavy"), "fact: heavy"),
        "incubator" => need(l.eggs_laid >= 1 || !l.eggs.is_empty(), "an egg"),
        "bone_sense" => need(l.facts.iter().any(|f| f.starts_with("bones:")), "a death"),
        "third_tag" => need(!l.bred.is_empty(), "breed once"),
        // Cut 3 tier 2.
        "row9" | "vault5" => need(l.bosses_slain() >= 3, "3 boss kinds"),
        "row10" => need(l.bosses_slain() >= 4, "4 boss kinds"),
        "party_slot_4" => need(l.tamed_kinds() >= 6, "tame 6 kinds"),
        "cadence" => need(has_tag_fact(&l.facts, "mirror"), "fact: mirror"),
        "noise_discipline" => need(has_tag_fact(&l.facts, "blind"), "fact: blind"),
        "reflect_read" => need(has_tag_fact(&l.facts, "reflect_melee"), "fact: reflect_melee"),
        "deep_march" => need(l.facts.contains("biome:deep"), "enter the Deep"),
        "lantern_rig" => need(l.facts.contains("item:lantern"), "find a lantern"),
        "recall_sense" => need(l.facts.contains("item:recall"), "read a recall"),
        _ => None,
    }
}

/// Cut 12 §1: a tactic card (its row is `tactic <id>`).
pub fn is_tactic_card(id: &str) -> bool {
    TACTIC_CARDS.contains(&id) || TIER2_CARDS.contains(&id) || MASTERY_CARDS.contains(&id)
}

/// Cut 12 §1: where a card's row sits when bought — before the set's engagement row (the
/// first `attack` / `shoot`), else the end. Both cohort-8 raters moved every card up by hand
/// from the bottom, where it never fired below `attack nearest`.
pub fn card_insert_at(rules: &crate::rules::RuleSet) -> usize {
    rules.rows.iter().position(|r| matches!(r.verb.v.as_str(), "attack" | "shoot")).unwrap_or(rules.rows.len())
}

/// Cut 9 §2: the gate still shut on an unlock — its fact/trophy gate, else its prerequisite,
/// else (`◆2 more`) the marks it is short of. `None` when it is buyable now.
pub fn needs(l: &LineageState, u: &UnlockDef) -> Option<String> {
    gate(l, u.id)
        .or_else(|| u.prereq.filter(|p| !l.unlocks.contains(*p)).map(|p| p.to_string()))
        .or_else(|| (is_row_unlock(u.id) && l.rules().own_rows() < l.max_rows()).then(|| "fill rows".to_string()))
        .or_else(|| (l.marks < u.cost).then(|| format!("◆{} more", u.cost - l.marks)))
}

/// Cut 10 §3: a row unlock (`row5`…`row10`) reads `needs: fill rows` (a requirement, not a state — both QA players on 952e306 read `rows full` as one) while the active set
/// still has a free row — the card is dimmed, not bought by mistake (cohort 6, rater L: "2/4
/// rows made me waste ◆2 on +1 row"). `buy` does not refuse it (the bots buy rows ahead of
/// writing them); the client checks `available` as the core does.
pub fn is_row_unlock(id: &str) -> bool {
    id.strip_prefix("row").is_some_and(|n| n.parse::<u32>().is_ok())
}

pub fn catalogue(l: &LineageState) -> Vec<UnlockInfo> {
    let mut cat: Vec<UnlockInfo> = UNLOCKS
        .iter()
        .map(|u| {
            let owned = l.unlocks.contains(u.id);
            // Cut 9 §2: every card that is not `available` says why (a shut gate, a missing
            // prerequisite, or the marks it is short of), so no card sits disabled unexplained.
            let needs = if owned { None } else { needs(l, u) };
            let available = !owned && needs.is_none();
            // Cut 12 §1: a tactic card says where its row goes (before the engagement row).
            let insert_at = (!owned && is_tactic_card(u.id)).then(|| card_insert_at(l.rules()));
            let gold = if owned { 0 } else { unlock_gold(l, u, l.gold_buys) };
            // Cut 19 §3: the next row unlock is pinned to the short list.
            let pinned = !owned && is_row_unlock(u.id) && u.prereq.is_none_or(|p| l.unlocks.contains(p));
            // QA on a946e04: the chain's next step and its prices (`UnlockInfo.next`).
            let next = UNLOCKS.iter().find(|n| n.prereq == Some(u.id) && !l.unlocks.contains(n.id)).map(|n| crate::wire::NextUnlock { id: n.id.into(), cost: n.cost, gold: unlock_gold(l, n, l.gold_buys), gold_after_gold: unlock_gold(l, n, l.gold_buys + 1) });
            let gold_next = if owned { 0 } else { unlock_gold(l, u, l.gold_buys + 1) };
            UnlockInfo { id: u.id.into(), cost: u.cost, owned, available, needs, delta: None, rows: unlock_rows(u.id), insert_at, pm: None, gold, situation: card_situation(u.id), stall: None, pinned, short: false, auto_insert: false, next, gold_next, carries: card_carries(u.id).map(str::to_string) }
        })
        .collect();
    mark_short(l, &mut cat);
    cat
}

/// Cards on the camp's short list and the report's PENDING (`UnlockInfo.short`).
pub const SHORT_LIST: usize = 3;

/// QA on 1a2a4a9 (qaP: "+1 row · verb: throw · card: boss focus", then a minute later `verb:
/// throw` and `cond: alert` gone for `card: corridor fighting` and `card: stair dance`; the
/// report's PENDING and the camp's UNLOCKS named different cards): the short list is the
/// core's, a function of the lineage alone — never of a sim's delta, which a cache eviction
/// or a second pass moves. The next `+1 row` (pinned) and then, among the cards on offer (not
/// owned, the chain's previous step owned), what can be bought now (marks or gold), then what
/// only a gate shuts, then what is short of marks — each the cheaper first, then the
/// catalogue's order.
fn mark_short(l: &LineageState, cat: &mut [UnlockInfo]) {
    let offered = |u: &UnlockInfo| !u.owned && UNLOCKS.iter().find(|d| d.id == u.id).and_then(|d| d.prereq).is_none_or(|p| l.unlocks.contains(p));
    let marks_only = |u: &UnlockInfo| u.needs.as_deref().is_none_or(|n| n.starts_with('◆'));
    let rank = |u: &UnlockInfo| -> u32 {
        if u.available || (marks_only(u) && u.gold > 0 && l.gold >= u.gold as i32) {
            0
        } else if !marks_only(u) {
            1
        } else {
            2
        }
    };
    let mut order: Vec<usize> = (0..cat.len()).filter(|&i| offered(&cat[i]) && !cat[i].pinned).collect();
    order.sort_by_key(|&i| (rank(&cat[i]), cat[i].cost, i));
    let pins: Vec<usize> = (0..cat.len()).filter(|&i| cat[i].pinned).collect();
    let room = SHORT_LIST.saturating_sub(pins.len());
    for i in order.into_iter().take(room).chain(pins) {
        cat[i].short = true;
    }
}

/// Cut 6 §6: what a tactic card does, as rows (its sub-rows in order, the vocabulary's own
/// tokens), or an automation's effect as one row-like entry (`{conds: [], verb: {v: "auto",
/// a: "keeps best weapon+armour"}}`). `None` for rows, vaults, slots, classes, conditions and
/// verbs. Rows are the language: no sentences.
pub fn unlock_rows(id: &str) -> Option<Vec<Row>> {
    let tag = |t: &str| Cond::t("foe_tag", t);
    let n = Cond::n;
    let row = |conds: Vec<Cond>, v: &str| Row::new(conds, Verb::new(v));
    let rowa = |conds: Vec<Cond>, v: &str, a: &str| Row::new(conds, Verb::arg(v, a));
    let auto = |what: &str| Some(vec![Row::new(vec![], Verb::arg("auto", what))]);
    match id {
        "corridor_fighting" => Some(vec![row(vec![n("foes>=", 2)], "back_corridor"), rowa(vec![Cond::flag("in_corridor"), n("adj>=", 1)], "attack", "nearest"), row(vec![Cond::flag("in_corridor"), n("foes>=", 1)], "hold"), rowa(vec![n("foes>=", 1)], "attack", "nearest")]),
        "kite_archers" => Some(vec![row(vec![tag("ranged"), tag("telegraph")], "retreat"), rowa(vec![tag("ranged")], "attack", "tag:ranged")]),
        "stair_dance" => Some(vec![row(vec![n("hp<", 50), n("foes>=", 1)], "descend"), row(vec![n("foes>=", 2), Cond::flag("path_stairs")], "descend"), rowa(vec![n("adj>=", 1)], "attack", "nearest")]),
        "gas_step" => Some(vec![rowa(vec![tag("gas"), n("adj>=", 1)], "shoot", "tag:gas"), row(vec![tag("gas"), n("adj>=", 1), n("hp<", 60)], "retreat"), rowa(vec![tag("gas"), n("adj>=", 1)], "attack", "tag:gas"), row(vec![tag("gas")], "hold")]),
        "pack_break" => Some(vec![row(vec![n("foes>=", 2)], "back_corridor"), rowa(vec![n("adj>=", 1)], "attack", "lowest"), row(vec![n("foes>=", 2)], "hold"), rowa(vec![n("foes>=", 1)], "attack", "nearest")]),
        // Cut 12 §2: the raid first (`on see den → attack nearest`), so the thief answer
        // answers the den (rater O: "the thief guard card changed nothing — the monkey still
        // took the heal").
        "thief_guard" => Some(vec![rowa(vec![Cond::t("on_see", "den")], "attack", "nearest"), rowa(vec![tag("thief"), n("adj>=", 1)], "attack", "tag:thief"), rowa(vec![tag("thief")], "shoot", "tag:thief"), rowa(vec![tag("thief")], "throw", "fire,tag:thief"), row(vec![tag("thief")], "back_corridor")]),
        "boss_focus" => Some(vec![rowa(vec![tag("boss")], "attack", "tag:boss"), rowa(vec![tag("boss"), tag("gas")], "throw", "fire,tag:boss"), rowa(vec![tag("summoned")], "attack", "tag:summoned")]),
        "last_stand" => Some(vec![rowa(vec![n("hp<", 30), n("adj>=", 1)], "drink", "heal"), row(vec![n("hp<", 30), n("adj>=", 1)], "second_wind"), rowa(vec![n("hp<", 30), n("adj>=", 1)], "drink", "unknown"), rowa(vec![n("hp<", 30), n("adj>=", 1)], "throw", "fire,nearest"), rowa(vec![n("hp<", 30), n("adj>=", 1)], "attack", "lowest")]),
        "cadence" => Some(vec![rowa(vec![n("foes>=", 1)], "attack", "nearest"), row(vec![n("foes>=", 1)], "shield_bash"), row(vec![n("foes>=", 1)], "cleave"), rowa(vec![n("foes>=", 1)], "throw", "fire,nearest"), row(vec![n("foes>=", 1)], "hold")]),
        "noise_discipline" => Some(vec![row(vec![n("hp<", 90)], "rest"), row(vec![tag("blind"), n("hp<", 90)], "descend")]),
        "reflect_read" => Some(vec![rowa(vec![tag("reflect_melee")], "shoot", "tag:reflect_melee"), rowa(vec![tag("reflect_melee"), tag("boss")], "throw", "fire,tag:boss"), rowa(vec![tag("reflect_melee"), n("adj>=", 1)], "attack", "nearest"), row(vec![tag("reflect_melee")], "retreat")]),
        "deep_march" => Some(vec![row(vec![n("floor_seen>=", 40)], "descend")]),
        "phalanx" => Some(vec![row(vec![n("foes>=", 2)], "back_corridor"), row(vec![n("foes>=", 1)], "taunt"), rowa(vec![n("adj>=", 1)], "attack", "nearest")]),
        "hit_and_fade" => Some(vec![row(vec![n("adj>=", 1)], "backstab"), row(vec![n("adj>=", 1)], "vanish"), rowa(vec![n("adj>=", 1)], "attack", "nearest")]),
        "hawkeye" => Some(vec![row(vec![n("adj>=", 1)], "kite"), row(vec![n("foes>=", 2)], "volley"), rowa(vec![n("foes>=", 1)], "double_shot", "nearest"), rowa(vec![n("foes>=", 1)], "shoot", "nearest")]),
        "archmage" => Some(vec![row(vec![n("adj>=", 2)], "nova"), row(vec![n("adj>=", 1)], "ward"), row(vec![n("adj>=", 1), n("hp<", 50)], "blink"), rowa(vec![n("foes>=", 1)], "bolt", "nearest")]),
        "quartermaster" => auto("keeps best weapon+armour"),
        "auto_supply" => auto("rebuys last supplies"),
        "auto_insure" => auto("insures brought items"),
        "incubator" => auto("eggs hatch 1 rest"),
        "supply_cap_5" => auto("5 supplies"),
        "bone_sense" => auto("paths to bones"),
        "third_tag" => auto("breeds 3 tags"),
        "lantern_rig" => auto("+2 vision"),
        "recall_sense" => Some(vec![rowa(vec![n("hp<", 15)], "read", "recall")]),
        _ => None,
    }
}

/// Cut 23 §3 (AJ: "paid cards are rows he could type"): what a paid card holds that no typed
/// row can — a verb or a cond outside the typed vocabulary, or a pre-bound target — ≤ 4 words.
/// `None` for a card that carries nothing outside the vocabulary — those cost 0 (kite
/// archers, stair dance, noise discipline, deep march, and the mastery cards, which a class
/// earns) — and for every other unlock (`cards_carry_outside_the_vocabulary` test).
///
/// The paid cards and what they carry: `hold` (a verb no row types: stand and let the foe
/// come) in corridor fighting, gas step, pack break and cadence; `throw fire` bound to a tag
/// (`fire,tag:thief`, `fire,tag:boss`) in thief guard, boss focus, reflect read, and bound to
/// the nearest foe at 30 % in last stand (the vocabulary's throws are `unknown` or a kind at
/// `nearest` only once identified).
pub fn card_carries(id: &str) -> Option<&'static str> {
    Some(match id {
        "corridor_fighting" => "hold in corridors",
        "gas_step" => "hold off gas",
        "pack_break" => "hold vs packs",
        "thief_guard" => "fire at thieves",
        "boss_focus" => "fire at the boss",
        "last_stand" => "fire point blank",
        "cadence" => "hold, then fire",
        "reflect_read" => "fire at the boss",
        _ => return None,
    })
}

/// Cut 18 §5: the foe tag a tactic card answers (its card row's `foe_tag`: `kite_archers` →
/// `ranged`, `gas_step` → `gas`), for the shop's `vs archers` beside a `reach ~0` — the card
/// matters when that foe is on the floor. `None` for a card keyed on something else (`foes ≥
/// 2`, `hp < 30%`) and for every other unlock.
pub fn card_situation(id: &str) -> Option<String> {
    if !is_tactic_card(id) {
        return None;
    }
    card_conds(id)?.into_iter().find(|c| c.k == "foe_tag").and_then(|c| c.t)
}

/// Cut 4 §9: the row a tactic card or a verb unlock would add (its natural place, at the top
/// of the list), for the catalogue's forecast delta. `None` for anything else.
pub fn unlock_row(l: &LineageState, id: &str) -> Option<Row> {
    unlock_row_untagged(l, id).map(|r| r.from("card"))
}

/// A tactic card's row conditions (the moment it is for), when it has a canonical row.
fn card_conds(id: &str) -> Option<Vec<Cond>> {
    let tag = |t: &str| Some(vec![Cond::t("foe_tag", t)]);
    match id {
        "corridor_fighting" | "stair_dance" => Some(vec![Cond::n("foes>=", 2)]),
        "kite_archers" => tag("ranged"),
        "gas_step" => tag("gas"),
        "pack_break" => tag("pack"),
        "thief_guard" => tag("thief"),
        "boss_focus" => tag("boss"),
        "last_stand" => Some(vec![Cond::n("hp<", 30)]),
        "cadence" => tag("mirror"),
        "noise_discipline" => Some(vec![Cond::n("hp<", 90)]),
        "reflect_read" => tag("reflect_melee"),
        "deep_march" => Some(vec![Cond::n("depth>=", crate::descent::biome_first(crate::descent::Biome::Deep) as i32)]),
        _ => None,
    }
}

fn unlock_row_untagged(l: &LineageState, id: &str) -> Option<Row> {
    let tag = |t: &str| Cond::t("foe_tag", t);
    if let Some(conds) = card_conds(id) {
        return Some(Row::new(conds, Verb::arg("tactic", id)));
    }
    match id {
        "throw" => {
            if has_tag_fact(&l.facts, "boss") {
                Some(Row::new(vec![tag("boss")], Verb::arg("throw", "unknown,tag:boss")))
            } else {
                Some(Row::new(vec![Cond::n("foes>=", 2)], Verb::arg("throw", "unknown,nearest")))
            }
        }
        "tame" => Some(Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("tame", "nearest"))),
        _ => None,
    }
}

/// Cut 4 §9: the catalogue with `delta` filled in for every card or verb not yet owned whose
/// gate is open: the forecast reach at `best_depth + 1` with the unlock owned and its row
/// added, minus the reach without (`DELTA_SIMS` paired sims under `CATALOGUE_TICK_BUDGET`,
/// memoised on the game per lineage/rules/depth like every `reach_with`). With `compute`
/// false only deltas already memoised are filled (no sims: `unlocks()` stays instant;
/// `unlock_deltas()` pays once per camp visit).
///
/// Cut 10 §3: a tactic card's row is the bare `[card]` row the client inserts on `buy`
/// (`{conds: [], verb: tactic <id>}` at `insert_at` — Cut 12 §1: before the engagement row;
/// a card row sits outside `max_rows`), so the chip's number is the number the buy produces (cohort 6, rater L:
/// "gas step reach +47% did not move the forecast when bought"); a verb unlock's canonical
/// row (`throw`, `tame`), which the player writes, still goes at the top.
pub fn catalogue_with_deltas(game: &Game, compute: bool) -> Vec<UnlockInfo> {
    let l = &game.lineage;
    let mut cat = catalogue(l);
    let depth = l.best_depth + 1;
    // Cut 9 §3: the panel's seed sequence and count (`forecast_tag`, `FORECAST_SIMS`): the
    // base runs the panel's seeds under the catalogue budget and every candidate replays
    // exactly the seeds the base ran, so a chip's delta is the panel's own sims moved by one
    // row — a paired difference, not a second, smaller draw.
    let sims = crate::forecast::FORECAST_SIMS;
    let budget = crate::forecast::CATALOGUE_TICK_BUDGET;
    let rules = l.rules().clone();
    let tag = crate::forecast::forecast_tag(game, depth);
    let max_rows = l.max_rows();
    let mut base: Option<(f64, u32)> = None;
    for u in cat.iter_mut() {
        // A card short of marks still shows its delta (the marks are not a gate on the sim).
        let gated = gate(l, &u.id).is_some() || UNLOCKS.iter().find(|d| d.id == u.id).and_then(|d| d.prereq).is_some_and(|p| !l.unlocks.contains(p));
        if u.owned || gated {
            continue;
        }
        // Cut 14 §1: a condition unlock's delta is the set's own locked rows waking — the same
        // rules on a lineage that owns the condition, against the base. It is where a death's
        // lock root sends its number when the root patch sits under the baseline (nothing
        // under it is offered on the death screen; here it reads as information).
        let wakes_a_row = u.id.starts_with("cond_") && rules.rows.iter().any(|r| r.conds.iter().any(|c| cond_unlock(&c.k) == Some(u.id.as_str())));
        // (position, the patched set): Cut 18 §5 — a card is measured at each of its candidate
        // places (`card_positions`) and reads its best (both cohort-13 raters: "every card read
        // `reach ~0 at R4`", measured below rows that fired first).
        // QA on 92eb880: when every place stalls, the bottom is measured too (`fallback`).
        let mut fallback: Option<(Option<usize>, crate::rules::RuleSet)> = None;
        let variants: Vec<(Option<usize>, crate::rules::RuleSet)> = if wakes_a_row {
            vec![(None, rules.clone())]
        } else {
            let Some((row, _)) = delta_row(l, &u.id) else { continue };
            if rules.rows.contains(&row) {
                continue;
            }
            // A verb unlock carries no delta: its canonical row at the top is nobody's policy
            // (`foes ≥ 2 → throw unknown` read `reach −92% ±11` — QA on 50bb162; `+21%` bought and
            // nothing moved — QA on e0f87e7). A verb is a word for the player's own rows; a
            // card's delta is the card's rows where the buy puts them.
            if row.verb.v != "tactic" {
                continue;
            }
            let place = |at: usize| {
                let mut patched = rules.clone();
                patched.rows.insert(at.min(patched.rows.len()), row.clone());
                (Some(at), patched.fit(max_rows.max(1)))
            };
            let places = card_positions(&rules);
            let bottom = rules.rows.len();
            if !places.contains(&bottom) {
                fallback = Some(place(bottom));
            }
            places.into_iter().map(place).collect()
        };
        // The sim lineage owns the unlock (the verb must be in its vocabulary to fire; a
        // locked condition wakes).
        let mut g = game.sim_clone();
        g.lineage.unlocks.insert(u.id.clone());
        // The sim game's lookups (its own fingerprint) go through the parent's cache.
        g.forecast_cache = game.forecast_cache.clone();
        // The best place (`best_place`): the highest reach among the places that do not stall,
        // ties to the earlier candidate (the buy's old place).
        let pick = |u: &mut UnlockInfo, measured: Vec<Measured>, b: f64, b_stall: f64, n: u32| {
            let Some(m) = best_place(&measured, b_stall) else { return };
            u.delta = Some(m.reach - b);
            u.pm = Some(delta_pm(b, m.reach, n as usize));
            u.stall = Some(m.stall - b_stall);
            if m.at.is_some() {
                u.insert_at = m.at;
            }
        };
        if !compute {
            let b = game.forecast_cache.borrow().get(&crate::forecast::reach_key(game, &rules, depth, sims, tag, budget)).copied();
            let b_stall = crate::forecast::stall_cached(game, &rules, depth, sims, tag, budget);
            if let (Some((b, n)), Some(b_stall)) = (b, b_stall) {
                // Only a fully measured card reads (a card half-measured would name a worse place).
                let cached = |(at, p): &(Option<usize>, crate::rules::RuleSet)| {
                    let reach = crate::forecast::reach_cached(&g, p, depth, n.max(1), tag, u64::MAX)?;
                    let stall = crate::forecast::stall_cached(&g, p, depth, n.max(1), tag, u64::MAX)?;
                    Some(Measured { at: *at, reach, stall })
                };
                let measured: Option<Vec<Measured>> = variants.iter().map(cached).collect();
                if let Some(mut m) = measured {
                    if !m.iter().any(|x| calm(x, b_stall)) {
                        match fallback.as_ref().map(cached) {
                            Some(Some(x)) => m.push(x),
                            Some(None) => continue,
                            None => {}
                        }
                    }
                    pick(u, m, b, b_stall, n);
                }
            }
            continue;
        }
        let (base_reach, n) = *base.get_or_insert_with(|| crate::forecast::reach_counted(game, &rules, depth, sims, tag, budget));
        let base_stall = crate::forecast::stall_cached(game, &rules, depth, sims, tag, budget).unwrap_or(0.0);
        let measure = |(at, p): &(Option<usize>, crate::rules::RuleSet)| {
            let reach = crate::forecast::reach_paired(&g, p, depth, n, tag);
            let stall = crate::forecast::stall_cached(&g, p, depth, n.max(1), tag, u64::MAX).unwrap_or(0.0);
            Measured { at: *at, reach, stall }
        };
        let mut measured: Vec<Measured> = variants.iter().map(measure).collect();
        if !measured.iter().any(|x| calm(x, base_stall)) {
            measured.extend(fallback.as_ref().map(measure));
        }
        game.forecast_cache.borrow_mut().extend(g.forecast_cache.into_inner());
        pick(u, measured, base_reach, base_stall, n);
    }
    let cards = rules.card_rows();
    for u in cat.iter_mut() {
        u.auto_insert = auto_insert(u, cards);
    }
    cat
}

/// QA on e75ec29 (qaR: eight cards bought, eight rows auto-inserted — 14 rows, D7 76 % → 44 %,
/// stall 40 %): at most this many card rows are put in the set by a buy; more are the
/// player's to add.
pub const AUTO_CARDS: usize = 3;

/// Whether buying this card puts its row in the set (`UnlockInfo.auto_insert`): a tactic
/// card not owned, measured at its best place (`delta`, `stall`), whose reach there is not
/// down and whose stall share rises ≤ `CARD_STALL_RISE`, into a set holding fewer than
/// `AUTO_CARDS` card rows. An unmeasured card is not inserted (the measure decides).
/// A card joins the set on its own only when it measurably helps (its reach gain clears its own
/// ±; a zero-effect card is owned, and the player adds it — QA on a946e04).
pub fn auto_insert(u: &UnlockInfo, cards_in_set: usize) -> bool {
    let card = TACTIC_CARDS.contains(&u.id.as_str()) || TIER2_CARDS.contains(&u.id.as_str()) || MASTERY_CARDS.contains(&u.id.as_str());
    card && !u.owned && cards_in_set < AUTO_CARDS && u.delta.is_some_and(|d| d > u.pm.unwrap_or(0.0).max(1e-9)) && u.stall.is_some_and(|s| s <= CARD_STALL_RISE + 1e-9)
}

/// A card measured at one place: the reach at the catalogue's depth and the stall share of
/// the same sims.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measured {
    pub at: Option<usize>,
    pub reach: f64,
    pub stall: f64,
}

/// QA on 92eb880 (qaN: `corridor fighting` bought at its best place, R6, and the shaft went
/// bank 81 % → 44 %, stall 35 %): a place whose stall share rises more than this over the set
/// without the card is not a best place.
pub const CARD_STALL_RISE: f64 = 0.05;

/// A place that does not raise the stall share past `CARD_STALL_RISE`.
pub fn calm(m: &Measured, base_stall: f64) -> bool {
    m.stall - base_stall <= CARD_STALL_RISE + 1e-9
}

/// The best of a card's measured places: the highest reach among the places whose stall share
/// rises ≤ `CARD_STALL_RISE` over `base_stall`, ties to the earlier; when every place stalls,
/// the one that stalls least (ties to the higher reach, then the earlier) — the catalogue then
/// measures the bottom of the set too.
pub fn best_place(measured: &[Measured], base_stall: f64) -> Option<Measured> {
    let by_reach = measured.iter().filter(|m| calm(m, base_stall)).fold(None, |best: Option<&Measured>, x| if best.is_none_or(|b| x.reach > b.reach + 1e-9) { Some(x) } else { best });
    by_reach
        .or_else(|| measured.iter().fold(None, |best: Option<&Measured>, x| if best.is_none_or(|b| x.stall < b.stall - 1e-9 || ((x.stall - b.stall).abs() <= 1e-9 && x.reach > b.reach + 1e-9)) { Some(x) } else { best }))
        .copied()
}

/// Cut 18 §5: where a card's row may go, in order — where the buy used to put it (before the
/// engagement row, `card_insert_at`), the top, and before the set's first own row (the first
/// row that is not a card); deduplicated. The catalogue measures each and keeps the best.
pub fn card_positions(rules: &crate::rules::RuleSet) -> Vec<usize> {
    let first_own = rules.rows.iter().position(|r| r.verb.v != "tactic").unwrap_or(rules.rows.len());
    let mut out: Vec<usize> = Vec::new();
    for at in [card_insert_at(rules), 0, first_own] {
        if !out.contains(&at) {
            out.push(at);
        }
    }
    out
}

/// Cut 13 §5: the half-width of a catalogue delta — the base and patched reaches are shares
/// over the same `n` panel seeds; their half-widths (`forecast::half_width`) combine in
/// quadrature (`√(pm_base² + pm_patched²)`, the independent bound; the paired difference is
/// no wider). A delta inside it reads `reach ~0` on the client (both cohort-9 raters: "pack
/// break +7 % then −14 % a minute apart").
pub fn delta_pm(base: f64, patched: f64, n: usize) -> f64 {
    let a = crate::forecast::half_width(base, n);
    let b = crate::forecast::half_width(patched, n);
    (a * a + b * b).sqrt()
}

/// Cut 10 §3: the row a card's delta simulates and where it goes — a tactic card as the bare
/// `[card]` row where `buy` puts it (Cut 12 §1: `card_insert_at`, before the engagement
/// row), a verb unlock's canonical row at the top. `None` for anything without a row.
pub fn delta_row(l: &LineageState, id: &str) -> Option<(Row, usize)> {
    let row = unlock_row(l, id)?;
    if row.verb.v == "tactic" {
        Some((Row::new(vec![], Verb::arg("tactic", id)).from("card"), card_insert_at(l.rules())))
    } else {
        Some((row, 0))
    }
}

pub fn buy(game: &mut Game, id: &str) -> Result<(), String> {
    let def = UNLOCKS.iter().find(|u| u.id == id).ok_or("unknown unlock")?;
    let l = &mut game.lineage;
    if l.unlocks.contains(id) {
        return Err("already owned".into());
    }
    if def.prereq.is_some_and(|p| !l.unlocks.contains(p)) {
        return Err("prerequisite missing".into());
    }
    if let Some(n) = gate(l, id) {
        return Err(format!("needs {n}"));
    }
    if l.marks < def.cost {
        return Err("not enough marks".into());
    }
    l.marks -= def.cost;
    l.unlocks.insert(id.into());
    Ok(())
}

/// Cut 15 §2: gold per mark of an unlock's cost, at the first gold buy.
pub const GOLD_PER_MARK: u32 = 150;

/// Cut 15 §2: an unlock's gold price — `GOLD_PER_MARK × cost × (1 + gold_buys / 4)`, in
/// integers (`× (4 + gold_buys) / 4`): ◆3 is $450 at the first gold buy, $900 once four have
/// been made. A free unlock (cost 0) has no gold price (0: not gold-buyable).
pub fn gold_price(cost: u32, gold_buys: u32) -> u32 {
    GOLD_PER_MARK * cost * (4 + gold_buys) / 4
}

/// Cut 23 §1 (both cohort-18 raters: "a $1650 row" beside nothing else to buy): an unlock's
/// gold price — a row slot sits on the forge's ladder (`kit::row_gold`: a multiple of the
/// lineage's unit, never climbing with other gold buys); any other unlock is `gold_price`.
pub fn unlock_gold(l: &LineageState, u: &UnlockDef, gold_buys: u32) -> u32 {
    if u.cost == 0 {
        return 0;
    }
    crate::kit::row_gold(l, u.id).unwrap_or_else(|| gold_price(u.cost, gold_buys))
}

/// Cut 15 §2: buy an unlock with gold instead of marks — the same gates as `buy` (owned,
/// prerequisite, fact/trophy gate), the gold price instead of the marks; the marks are left
/// alone, the ledger reads `unlock <id>`, and the next gold price climbs.
pub fn buy_gold(game: &mut Game, id: &str) -> Result<(), String> {
    let def = UNLOCKS.iter().find(|u| u.id == id).ok_or("unknown unlock")?;
    let l = &mut game.lineage;
    if l.unlocks.contains(id) {
        return Err("already owned".into());
    }
    if def.cost == 0 {
        return Err("not for gold".into());
    }
    if def.prereq.is_some_and(|p| !l.unlocks.contains(p)) {
        return Err("prerequisite missing".into());
    }
    if let Some(n) = gate(l, id) {
        return Err(format!("needs {n}"));
    }
    crate::kit::lock_unit(l);
    let price = unlock_gold(l, def, l.gold_buys);
    if l.gold < price as i32 {
        return Err("not enough gold".into());
    }
    l.gold_move(-(price as i32), &format!("unlock {id}"));
    // (a row slot's price is the forge's ladder: it does not raise the other gold prices)
    if !is_row_unlock(id) {
        l.gold_buys += 1;
    }
    l.unlocks.insert(id.into());
    Ok(())
}

/// Decisions waiting at camp: affordable unlocks, flagged rules, patches, vault, party.
pub fn pending(game: &Game) -> Vec<String> {
    let l = &game.lineage;
    let mut out = Vec::new();
    // QA on 1a2a4a9: the report's unlocks are the camp's short list (`UnlockInfo.short`).
    for u in catalogue(l) {
        if u.available && u.short {
            out.push(format!("unlock {} ({})", u.id, u.cost));
        }
    }
    // Cut 23 §1: a forge step the purse pays now (`forge sword +1 · $300`).
    for lad in crate::kit::ladders(l) {
        if let Some(n) = lad.next.filter(|n| n.affordable) {
            out.push(format!("forge {} · ${}", n.label, n.price));
        }
    }
    let rules = l.rules();
    if rules.rows.is_empty() {
        out.push("rows: none".into());
    }
    // Cut 9 §8: over the absence's real runs (the same window as the reel), in numbers:
    // `R1 fired 0 of 15 runs: HP<30% → drink heal`.
    // Every row gets its line so a chunked absence can add the numbers up client-side; the
    // client shows the rows that fired in under a third of the runs.
    let real_runs = game.batch.run_ticks.len() as u32;
    if real_runs > 0 {
        let shadowed = l.shadowed_by(rules);
        for (i, r) in rules.active(l.max_rows()) {
            let n = game.batch.row_runs.get(i).copied().unwrap_or(0);
            // QA on 92eb880 (qaM: `R3 fired 0 of 16 runs: hp < 30% → drink heal · heal unknown`
            // under R1 `hp < 30% → return`): a row an earlier one shadows names that row — the
            // kind being unknown was never why it did not fire.
            if let Some(by) = shadowed.get(i).copied().flatten().filter(|_| n == 0) {
                out.push(format!("R{} fired {n} of {real_runs} runs: {} · shadowed by R{}", i + 1, r.describe(), by + 1));
                continue;
            }
            // A drink/read row that never fired because the kind is still unidentified says so
            // (QA on 56f2a1d: `R1 fired 0 of 17 runs: hp < 50% → drink heal` beside `heal ×16`
            // salvaged, with no reason on screen).
            let unknown = n == 0
                && matches!(r.verb.v.as_str(), "drink" | "read")
                && r.verb.a.as_deref().is_some_and(|k| k != "unknown" && !crate::item::is_identified(&l.facts, &l.flavours, k));
            let why = if unknown { format!(" · {} unknown", r.verb.a.as_deref().unwrap_or_default()) } else { String::new() };
            out.push(format!("R{} fired {n} of {real_runs} runs: {}{why}", i + 1, r.describe()));
        }
    }
    if let Some(id) = game.batch.worst_death {
        if let Some(rec) = game.deaths.get(&id) {
            if rec.death.verdict == "gap" {
                if let Some(p) = rec.death.patches.first() {
                    out.push(format!("patch D{}: {}", rec.death.depth, p.row.describe()));
                }
            }
            // Cut 19 §4: a `row` death's patch cuts the row that killed him.
            if let (true, Some(r)) = (rec.death.verdict == "row", rec.death.cause_row) {
                out.push(format!("patch D{}: cut R{}", rec.death.depth, r + 1));
            }
            // Cut 25 §2: an `order` death's patch moves his own row up.
            if let (true, Some(r), Some(o)) = (rec.death.verdict == "order", rec.death.cause_row, rec.death.order_over) {
                out.push(format!("patch D{}: move R{} above R{}", rec.death.depth, r + 1, o + 1));
            }
        }
    }
    if !game.batch.found.is_empty() {
        out.push(format!("vault: {} found", game.batch.found.len()));
    }
    if l.party.len() < l.party_slots() as usize && !l.kennel.is_empty() {
        out.push("party: choose".into());
    }
    if l.eggs.iter().any(|e| e.from_loss) && l.gold >= 50 {
        out.push("egg: hatch (50 gold)".into());
    }
    if !l.bones.is_empty() {
        out.push(format!("bones: {} on the floor", l.bones.len()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalogue_matches_the_contract() {
        assert_eq!(UNLOCKS.len(), 44, "35 (Cut 2) + 10 (Cut 3 tier 2) − auto_supply (Cut 19 §3: the repeat is free)");
        let ids: Vec<&str> = UNLOCKS.iter().map(|u| u.id).collect();
        for c in TACTIC_CARDS.iter().chain(TIER2_CARDS.iter()) {
            assert!(ids.contains(c), "{c}");
        }
        for (_, u) in COND_UNLOCKS {
            assert!(ids.contains(&u), "{u}");
        }
        let cost: u32 = UNLOCKS.iter().map(|u| u.cost).sum();
        // Cut 8B: the rogue and `tame` cost nothing (were 4 and 2: 6 off the Cut 3 sum).
        // Cut 19 §3: `auto_supply` (4) left the catalogue — the repeat is the free default.
        // Cut 23 §3: kite archers, stair dance (3 each), noise discipline and deep march (5 each)
        // carry nothing outside the typed vocabulary: free.
        // Cut 24 (the Warlord's drive-off teaches his counter: the dayplayer's lineages went
        // deeper, ranked up and held 9 marks with every card gated): vault4 10 → 9.
        assert_eq!(cost, 2 + 4 + 7 + 11 + 4 + 9 + 3 + 6 + 9 + 6 + 8 + 2 + 12 + 18 + 5 + 6 + 4 + 3 + 3 + 6 + 8 + 12 + 8 + 8 + 10 + 6 + 8);
        assert_eq!(UNLOCKS.iter().find(|u| u.id == "rogue").unwrap().cost, 0);
        assert_eq!(UNLOCKS.iter().find(|u| u.id == "tame").unwrap().cost, 0);
        let by = |id: &str| UNLOCKS.iter().find(|u| u.id == id).unwrap();
        assert_eq!(by("row9").prereq, Some("row8"));
        assert_eq!(by("row10").cost, 12);
        // Cut 4: tier 2 costs 8–12 (Cut 23 §3: or nothing, a card a player could type).
        for u in UNLOCKS.iter().skip(35) {
            assert!((5..=12).contains(&u.cost) || (u.cost == 0 && card_carries(u.id).is_none()), "{} costs {}", u.id, u.cost);
        }
        let l = LineageState::new(2);
        let cat = catalogue(&l);
        assert_eq!(cat.iter().find(|u| u.id == "row9").unwrap().needs.as_deref(), Some("3 boss kinds"));
        assert_eq!(cat.iter().find(|u| u.id == "cadence").unwrap().needs.as_deref(), Some("fact: mirror"));
        assert_eq!(cat.iter().find(|u| u.id == "lantern_rig").unwrap().needs.as_deref(), Some("find a lantern"));
        let l = LineageState::new(1);
        let cat = catalogue(&l);
        let kite = cat.iter().find(|u| u.id == "kite_archers").unwrap();
        assert_eq!(kite.needs.as_deref(), Some("fact: ranged"));
        assert!(cat.iter().find(|u| u.id == "row6").unwrap().needs.as_deref() == Some("row5"));
        // Cut 9 §2: short of marks is a need too (`◆2 more`); with the marks, none.
        assert_eq!(cat.iter().find(|u| u.id == "cond_turns").unwrap().needs.as_deref(), Some("◆2 more"));
        let mut l = LineageState::new(1);
        l.marks = 2;
        let cat = catalogue(&l);
        let ct = cat.iter().find(|u| u.id == "cond_turns").unwrap();
        assert!(ct.needs.is_none() && ct.available);
    }

    /// Cut 8B §2–3: the rogue costs nothing and opens at the first bank; `tame` is owned from
    /// the start with the kennel's leash on the shelf and its fact held.
    #[test]
    fn rogue_free_at_first_bank_and_tame_from_the_start() {
        let mut g = crate::engine::Game::new(7);
        let cat = catalogue(&g.lineage);
        let rogue = cat.iter().find(|u| u.id == "rogue").unwrap();
        assert_eq!((rogue.cost, rogue.available, rogue.needs.as_deref()), (0, false, Some("bank once")));
        assert!(cat.iter().find(|u| u.id == "tame").unwrap().owned);
        assert!(g.lineage.facts.contains("item:leash"));
        assert!(g.lineage.supplies.iter().any(|s| s.kind == "leash" && s.known && s.free));
        assert!(g.vocabulary().verbs.contains(&Verb::arg("tame", "nearest")));
        assert_eq!(buy(&mut g, "rogue").unwrap_err(), "needs bank once");
        // One bank, no marks to spare: the rogue is bought.
        g.lineage.banked_depths.insert(2);
        g.lineage.marks = 0;
        assert!(catalogue(&g.lineage).iter().find(|u| u.id == "rogue").unwrap().available);
        buy(&mut g, "rogue").unwrap();
        assert!(g.set_class("rogue").is_ok());
        // The free leash refunds nothing and is not rebought by the automation.
        g.clear_supplies();
        assert_eq!(g.lineage.gold, 0);
        assert!(g.lineage.gold_ledger.is_empty());
    }

    /// Cut 6 §6: every card and automation carries rows; rows, vaults, slots, classes,
    /// conditions and verbs do not. Card rows use real tokens (≤ 3 conds); automations are
    /// one `auto` row with a ≤ 4-word effect.
    #[test]
    fn cards_and_automations_carry_rows() {
        let l = LineageState::new(1);
        let cat = catalogue(&l);
        for u in &cat {
            let card = TACTIC_CARDS.contains(&u.id.as_str()) || TIER2_CARDS.contains(&u.id.as_str());
            let auto = matches!(u.id.as_str(), "quartermaster" | "auto_supply" | "auto_insure" | "incubator" | "supply_cap_5" | "bone_sense" | "third_tag" | "lantern_rig" | "recall_sense");
            match &u.rows {
                Some(rows) => {
                    assert!(card || auto, "{} carries rows", u.id);
                    assert!(!rows.is_empty() && rows.len() <= 5, "{}", u.id);
                    for r in rows {
                        assert!(r.conds.len() <= 3, "{}: {r:?}", u.id);
                        for c in &r.conds {
                            assert!(c.valid(), "{}: {c:?}", u.id);
                        }
                        if r.verb.v == "auto" {
                            assert!(crate::rules::word_count(r.verb.a.as_deref().unwrap_or("")) <= 4, "{}: {r:?}", u.id);
                        } else {
                            assert!(r.verb.valid(), "{}: {r:?}", u.id);
                        }
                    }
                }
                None => assert!(!card && !auto, "{} has no rows", u.id),
            }
        }
        for m in MASTERY_CARDS {
            assert!(unlock_rows(m).is_some(), "{m}");
        }
    }
}
