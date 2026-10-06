//! Actual next-absence choice comparisons from a saved multihero Session.
use riddle_core::bloodlines::Session;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if !(5..=6).contains(&args.len()) {
        return Err("choice_check SAVE HOURS CHOICES OUTPUT (CHOICES=id:slot,id:slot; optional UPGRADES=health,armour,damage,all)".into());
    }
    riddle_core::balance::configure_from_env()?;
    riddle_core::chronicle::use_shipping_words();
    riddle_core::forecast::set_parallel_sims(false);
    let source = Session::load(&std::fs::read_to_string(&args[1])?)?;
    let hours: u64 = args[2].parse()?;
    let seconds = hours
        .checked_mul(3600)
        .filter(|_| (1..=336).contains(&hours))
        .ok_or("invalid hours")?;
    let mut choices = Vec::new();
    for value in args[3].split(',') {
        let (id, slot) = value.rsplit_once(':').ok_or("choice must be id:slot")?;
        let slot: usize = slot.parse()?;
        if choices.contains(&(id.to_string(), slot)) {
            return Err("duplicate choice".into());
        }
        // Validate all requested choices against untouched state before work.
        let mut candidate = source.clone();
        candidate.equip_package(id, slot)?;
        choices.push((id.to_string(), slot));
    }
    let paths = args.get(5).map(|s| s.split(',').collect::<Vec<_>>()).unwrap_or_default();
    let mut prepared = vec![(None, source.clone(), 0)];
    for (i, path) in paths.iter().enumerate() {
        if paths[..i].contains(path) { return Err("duplicate upgrade path".into()); }
        let (game, spent) = upgrade_path(&source, path)?;
        prepared.push((Some(*path), game, spent));
    }
    // Validate the entire cross-product before playing any case.
    for (_, game, _) in &prepared {
        for (id, slot) in &choices {
            let mut candidate = game.clone();
            candidate.equip_package(id, *slot)?;
        }
    }
    let mut cases = Vec::new();
    for (path, prepared_game, spent) in prepared {
        for choice in std::iter::once(None).chain(choices.iter().map(Some)) {
            let mut game = prepared_game.clone();
            if let Some((id, slot)) = choice { game.equip_package(id, *slot)?; }
            let start = Instant::now();
            let report = game.run_offline_mode(seconds, false, true);
            let elapsed = start.elapsed().as_secs_f64();
            let mut case = serde_json::json!({
                "choice": choice.map(|(id, slot)| serde_json::json!({"id": id, "slot": slot})),
                "seconds": elapsed, "report": report, "save": game.save()
            });
            if let Some(path) = path {
                case["upgrade"] = serde_json::json!({"path":path,"spent":spent,
                    "before":source.lineage.bloodline,"after":prepared_game.lineage.bloodline});
            }
            cases.push(case);
        }
    }
    let output = serde_json::json!({"version":if args.len()==5 {1} else {2},"selected":source.selected,"hours":hours,"slots":source.others.len()+1,"cases":cases});
    std::fs::write(&args[4], serde_json::to_vec(&output)?)?;
    Ok(())
}

/// Prepare an independent legal path; the source remains untouched even on refusal.
fn upgrade_path(source: &Session, path: &str) -> Result<(Session, u32), String> {
    let ids: Vec<&str> = match path {
        "all" => riddle_core::legacy::IDS.to_vec(),
        id if riddle_core::legacy::IDS.contains(&id) => vec![id],
        _ => return Err(format!("unknown upgrade path {path}")),
    };
    let mut game = source.clone();
    let mut spent = 0;
    for id in ids {
        loop {
            let offer = riddle_core::legacy::offers(&game.lineage, game.run.is_some())
                .into_iter().find(|u| u.id == id).ok_or("missing upgrade offer")?;
            if offer.rank >= offer.cap { break; }
            game.upgrade_hero(id).map_err(|e| format!("{path}: {e}"))?;
            spent += offer.price;
        }
    }
    Ok((game, spent))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn town(points: u32) -> Session {
        let mut s = Session::new(1);
        s.active = riddle_core::Game::new_resident(1);
        s.lineage.bloodline.as_mut().unwrap().points = points;
        s
    }
    #[test]
    fn real_costs_partial_and_complete_paths() {
        let source = town(54); let before = source.save();
        let (all, spent) = upgrade_path(&source, "all").unwrap();
        assert_eq!(spent,54);assert_eq!(all.lineage.gold,source.lineage.gold);
        assert_eq!(all.lineage.bloodline.as_ref().unwrap().points,0);
        for id in riddle_core::legacy::IDS { assert_eq!(all.lineage.bloodline.as_ref().unwrap().upgrades[id],3); }
        let (same, spent) = upgrade_path(&all,"all").unwrap();assert_eq!(spent,0);assert_eq!(same.save(),all.save());
        let (mut partial, _) = upgrade_path(&source,"health").unwrap();
        partial.lineage.bloodline.as_mut().unwrap().upgrades.insert("damage".into(),2);
        let (finished,spent)=upgrade_path(&partial,"damage").unwrap();assert_eq!(spent,9);
        assert_eq!(finished.lineage.bloodline.as_ref().unwrap().upgrades["damage"],3);
        assert_eq!(source.save(),before);
    }
    #[test]
    fn refused_paths_do_not_mutate_source() {
        let mut s=town(17);let before=s.save();
        assert!(upgrade_path(&s,"health").is_err());assert!(upgrade_path(&s,"unknown").is_err());assert_eq!(s.save(),before);
        s.lineage.bloodline.as_mut().unwrap().points=54;s.send();let before=s.save();
        assert!(upgrade_path(&s,"all").is_err());assert_eq!(s.save(),before);
    }
    #[test]
    fn only_selected_bloodline_spends() {
        let mut s=town(54);let mut other=riddle_core::Game::new_resident(2);
        other.lineage.bloodline_id=2;other.lineage.bloodline.as_mut().unwrap().points=99;
        s.others.insert(2,other);s.next_id=3;
        let before=s.others[&2].save();let (changed,spent)=upgrade_path(&s,"armour").unwrap();
        assert_eq!(spent,18);assert_eq!(changed.selected,1);assert_eq!(changed.others[&2].save(),before);
        assert_eq!(s.lineage.bloodline.as_ref().unwrap().points,54);
        s.select_bloodline(2).unwrap();let before=s.others[&1].save();
        let (changed,spent)=upgrade_path(&s,"health").unwrap();assert_eq!(spent,18);
        assert_eq!(changed.selected,2);assert_eq!(changed.others[&1].save(),before);
        assert_eq!(changed.lineage.bloodline.as_ref().unwrap().points,81);
    }
}
