//! Actual next-absence choice comparisons from a saved multihero Session.
use riddle_core::bloodlines::Session;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 5 {
        return Err("choice_check SAVE HOURS CHOICES OUTPUT (CHOICES=id:slot,id:slot)".into());
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
    let mut cases = Vec::new();
    for choice in std::iter::once(None).chain(choices.iter().map(Some)) {
        let mut game = source.clone();
        if let Some((id, slot)) = choice {
            game.equip_package(id, *slot)?;
        }
        let start = Instant::now();
        let report = game.run_offline_mode(seconds, false, true);
        let elapsed = start.elapsed().as_secs_f64();
        cases.push(serde_json::json!({
            "choice": choice.map(|(id, slot)| serde_json::json!({"id": id, "slot": slot})),
            "seconds": elapsed, "report": report, "save": game.save()
        }));
    }
    let output = serde_json::json!({"version":1,"selected":source.selected,"hours":hours,"slots":source.others.len()+1,"cases":cases});
    std::fs::write(&args[4], serde_json::to_vec(&output)?)?;
    Ok(())
}
