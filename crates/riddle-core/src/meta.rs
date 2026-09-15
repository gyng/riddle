//! Meta: unlock catalogue, purchases, pending decisions.
use crate::engine::{Game, LineageState};
use crate::facts::{has_boss_counter, has_tag_fact};
use crate::wire::UnlockInfo;

pub struct UnlockDef {
    pub id: &'static str,
    pub cost: u32,
    pub prereq: Option<&'static str>,
}

pub const UNLOCKS: &[UnlockDef] = &[
    UnlockDef { id: "row5", cost: 2, prereq: None },
    UnlockDef { id: "row6", cost: 3, prereq: Some("row5") },
    UnlockDef { id: "row7", cost: 4, prereq: Some("row6") },
    UnlockDef { id: "row8", cost: 5, prereq: Some("row7") },
    UnlockDef { id: "rogue", cost: 4, prereq: None },
    UnlockDef { id: "vault2", cost: 3, prereq: None },
    UnlockDef { id: "vault3", cost: 5, prereq: Some("vault2") },
    UnlockDef { id: "corridor_fighting", cost: 3, prereq: None },
    UnlockDef { id: "kite_archers", cost: 3, prereq: None },
    UnlockDef { id: "stair_dance", cost: 3, prereq: None },
    UnlockDef { id: "throw", cost: 2, prereq: None },
    UnlockDef { id: "party_slot_2", cost: 4, prereq: None },
    UnlockDef { id: "tame", cost: 2, prereq: None },
];

/// Fact gates on unlocks: tactic cards need the matching knowledge; tame needs a leash seen.
fn fact_gate(l: &LineageState, id: &str) -> bool {
    match id {
        "corridor_fighting" => has_tag_fact(&l.facts, "pack"),
        "kite_archers" => has_tag_fact(&l.facts, "ranged"),
        "stair_dance" => has_boss_counter(&l.facts),
        "tame" => l.facts.contains("item:leash"),
        _ => true,
    }
}

pub fn catalogue(l: &LineageState) -> Vec<UnlockInfo> {
    UNLOCKS
        .iter()
        .map(|u| {
            let owned = l.unlocks.contains(u.id);
            let available = !owned && u.prereq.is_none_or(|p| l.unlocks.contains(p)) && fact_gate(l, u.id) && l.marks >= u.cost;
            UnlockInfo { id: u.id.into(), cost: u.cost, owned, available }
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
    if !fact_gate(l, id) {
        return Err("fact missing".into());
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
    out
}
