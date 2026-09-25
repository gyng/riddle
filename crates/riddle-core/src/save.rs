//! Save and load: the whole game as versioned JSON.
use crate::engine::{Game, SAVE_VERSION};

pub fn save(game: &Game) -> String {
    serde_json::to_string(game).unwrap_or_default()
}

pub fn load(text: &str) -> Result<Game, String> {
    let mut g: Game = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if g.version != SAVE_VERSION {
        return Err(format!("save version {} unsupported", g.version));
    }
    // Cut 6 §5: counter facts carry their row.
    crate::facts::upgrade_counter_facts(&mut g.lineage.facts);
    // Cut 9 §7: the graveyard's last five deaths stay answerable.
    g.max_deaths = g.max_deaths.max(crate::engine::KEPT_DEATHS);
    // QA on e75ec29: a load is a camp — its bounty floor is the one on the screen.
    g.bounty_seen = g.lineage.bounty;
    // Cut 21 §1: a lineage from before the waystones lights them from its banks.
    if g.lineage.waystones.is_empty() {
        if let Some(&d) = g.lineage.banked_depths.iter().next_back() {
            g.lineage.light_waystones(d);
        }
    }
    Ok(g)
}
