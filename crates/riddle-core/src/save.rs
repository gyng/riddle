//! Save and load: the whole game as versioned JSON.
use crate::engine::{Game, SAVE_VERSION};

pub fn save(game: &Game) -> String {
    // QA on a946e04: the refined panel keys of the lineage as it stands travel with the save
    // (a key of an earlier lineage state can never be read again; the rest are the memo's).
    let lk = format!("{}:", crate::forecast::lineage_key(game));
    let now: std::collections::BTreeSet<String> = game.refined_panels.borrow().iter().filter(|k| k.starts_with(&lk)).cloned().collect();
    let all = game.refined_panels.replace(now);
    let text = serde_json::to_string(game).unwrap_or_default();
    game.refined_panels.replace(all);
    text
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
    // Cut 28 §1: a lineage from before the oaths draws its board.
    crate::oath::refresh(&mut g.lineage);
    // Cut 30 §2: a save from before the packages: its set becomes the `custom` stance, the pen open
    // (its rules keep working; drills come above them) — before the curriculum reads the pen.
    if g.lineage.pkg_v == 0 {
        crate::packages::migrate(&mut g.lineage);
        g.lineage.pkg_v = 1;
        crate::systems::update_with(&mut g.lineage, false, false);
        g.lineage.systems_new.clear();
    }
    // Cut 29: a save from before the curriculum (no system open) owns the free vocabulary its gates
    // opened and opens the systems it has used (a later save keeps what it holds: a load is no event).
    if g.lineage.systems.is_empty() {
        crate::meta::grant_free(&mut g.lineage);
        crate::systems::upgrade(&mut g.lineage);
    }
    // Cut 21 §1: a lineage from before the waystones lights them from its banks.
    if g.lineage.waystones.is_empty() {
        if let Some(&d) = g.lineage.banked_depths.iter().next_back() {
            g.lineage.light_waystones(d);
        }
    }
    // Cut 30 §1: a save from before the traits maps its temperament onto a shape.
    crate::traits::upgrade(&mut g.lineage);
    // Cut 30.5: a save from before the works tree: its workers up to the scout hired (no player regresses)
    crate::tree::upgrade(&mut g.lineage);
    if g.lineage.town.gold_v == 0 {
        // The chest is part of total gold already: release it without another ledger credit.
        g.lineage.town.auto_collect = true;
        g.lineage.tree.chest = 0;
        g.lineage.town.gold_v = 1;
    }
    crate::legacy::ensure(&mut g.lineage);
    // Refresh generated camp rows before the editor can absorb a legacy compiled row as authored.
    // A live replay keeps its row indices until the normal run-end recompile. Literal sets are unchanged.
    if g.run.is_none() {
        crate::packages::recompile(&mut g.lineage);
    }
    Ok(g)
}
