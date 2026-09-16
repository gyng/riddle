//! Meta: unlock catalogue (Cut 2 §3), purchases, pending decisions.
use crate::engine::{Game, LineageState};
use crate::facts::{has_boss_counter, has_tag_fact};
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
    UnlockDef { id: "vault4", cost: 10, prereq: Some("vault3") },
    UnlockDef { id: "rogue", cost: 4, prereq: None },
    UnlockDef { id: "ranger", cost: 6, prereq: None },
    UnlockDef { id: "caster", cost: 8, prereq: None },
    UnlockDef { id: "tame", cost: 2, prereq: None },
    UnlockDef { id: "throw", cost: 2, prereq: None },
    UnlockDef { id: "cond_alert", cost: 2, prereq: None },
    UnlockDef { id: "cond_turns", cost: 2, prereq: None },
    UnlockDef { id: "cond_loot", cost: 2, prereq: None },
    UnlockDef { id: "cond_on_kill", cost: 2, prereq: None },
    UnlockDef { id: "cond_on_see", cost: 2, prereq: None },
    UnlockDef { id: "cond_party_hp", cost: 2, prereq: None },
    UnlockDef { id: "corridor_fighting", cost: 3, prereq: None },
    UnlockDef { id: "kite_archers", cost: 3, prereq: None },
    UnlockDef { id: "stair_dance", cost: 3, prereq: None },
    UnlockDef { id: "gas_step", cost: 3, prereq: None },
    UnlockDef { id: "pack_break", cost: 3, prereq: None },
    UnlockDef { id: "thief_guard", cost: 3, prereq: None },
    UnlockDef { id: "boss_focus", cost: 3, prereq: None },
    UnlockDef { id: "last_stand", cost: 3, prereq: None },
    UnlockDef { id: "quartermaster", cost: 5, prereq: None },
    UnlockDef { id: "auto_supply", cost: 4, prereq: None },
    UnlockDef { id: "auto_insure", cost: 6, prereq: None },
    UnlockDef { id: "incubator", cost: 4, prereq: None },
    UnlockDef { id: "supply_cap_5", cost: 3, prereq: None },
    UnlockDef { id: "bone_sense", cost: 3, prereq: None },
    UnlockDef { id: "third_tag", cost: 6, prereq: None },
];

/// The eight tactic cards (Cut 2 §3) plus the four mastery cards (class L10).
pub const TACTIC_CARDS: [&str; 8] = ["corridor_fighting", "kite_archers", "stair_dance", "gas_step", "pack_break", "thief_guard", "boss_focus", "last_stand"];
pub const MASTERY_CARDS: [&str; 4] = ["phalanx", "hit_and_fade", "hawkeye", "archmage"];

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

/// The fact/trophy gate of an unlock: `None` when open, else the human-readable need.
pub fn gate(l: &LineageState, id: &str) -> Option<String> {
    let need = |ok: bool, text: &str| if ok { None } else { Some(text.to_string()) };
    match id {
        "party_slot_2" => need(l.tamed_kinds() >= 1, "tame 1"),
        "party_slot_3" => need(l.tamed_kinds() >= 3, "tame 3"),
        "ranger" => need(l.bosses_slain() >= 1, "slay a boss"),
        "caster" => need(l.bosses_slain() >= 2, "slay 2 bosses"),
        "tame" => need(l.facts.contains("item:leash"), "find a leash"),
        "cond_alert" => need(l.facts.contains("alert:rising"), "see the alert rise"),
        "cond_on_kill" => need(!l.kills.is_empty(), "a kill"),
        "cond_on_see" => need(l.facts.iter().any(|f| f.starts_with("foe:")), "meet a foe"),
        "cond_party_hp" => need(l.tamed_kinds() >= 1, "tame 1"),
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
        _ => None,
    }
}

pub fn catalogue(l: &LineageState) -> Vec<UnlockInfo> {
    UNLOCKS
        .iter()
        .map(|u| {
            let owned = l.unlocks.contains(u.id);
            let prereq_ok = u.prereq.is_none_or(|p| l.unlocks.contains(p));
            let needs = gate(l, u.id).or_else(|| if prereq_ok { None } else { u.prereq.map(|p| p.to_string()) });
            let available = !owned && prereq_ok && needs.is_none() && l.marks >= u.cost;
            UnlockInfo { id: u.id.into(), cost: u.cost, owned, available, needs: if owned { None } else { needs } }
        })
        .collect()
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

/// Decisions waiting at camp: affordable unlocks, flagged rules, patches, vault, party.
pub fn pending(game: &Game) -> Vec<String> {
    let l = &game.lineage;
    let mut out = Vec::new();
    for u in catalogue(l) {
        if u.available {
            out.push(format!("unlock {} ({})", u.id, u.cost));
        }
    }
    let rules = l.rules();
    if rules.rows.is_empty() {
        out.push("rows: none".into());
    }
    if game.batch.runs > 0 {
        for (i, r) in rules.rows.iter().enumerate().take(l.max_rows()) {
            if game.batch.row_fired.get(i).copied().unwrap_or(1) == 0 {
                out.push(format!("R{} never fired: {}", i + 1, r.describe()));
            }
        }
    }
    if let Some(id) = game.batch.worst_death {
        if let Some(rec) = game.deaths.get(&id) {
            if rec.death.verdict == "gap" {
                if let Some(p) = rec.death.patches.first() {
                    out.push(format!("patch D{}: {}", rec.death.depth, p.row.describe()));
                }
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
        assert_eq!(UNLOCKS.len(), 35);
        let ids: Vec<&str> = UNLOCKS.iter().map(|u| u.id).collect();
        for c in TACTIC_CARDS {
            assert!(ids.contains(&c), "{c}");
        }
        for (_, u) in COND_UNLOCKS {
            assert!(ids.contains(&u), "{u}");
        }
        let cost: u32 = UNLOCKS.iter().map(|u| u.cost).sum();
        assert_eq!(cost, 2 + 4 + 7 + 11 + 4 + 9 + 3 + 6 + 10 + 4 + 6 + 8 + 2 + 2 + 12 + 24 + 5 + 4 + 6 + 4 + 3 + 3 + 6);
        let l = LineageState::new(1);
        let cat = catalogue(&l);
        let kite = cat.iter().find(|u| u.id == "kite_archers").unwrap();
        assert_eq!(kite.needs.as_deref(), Some("fact: ranged"));
        assert!(cat.iter().find(|u| u.id == "row6").unwrap().needs.as_deref() == Some("row5"));
        assert!(cat.iter().find(|u| u.id == "cond_turns").unwrap().needs.is_none());
    }
}
