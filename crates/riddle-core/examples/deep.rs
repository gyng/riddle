//! Cut 25 §4: a deep lineage (AM's shape on 075d8e2: seed 2501, his exported set, the armour
//! ladder three steps up, a fighter played to D11) after an 8 h absence — written as a save for
//! the client's fixture (`web/tests/fixtures/deep.json`) and profiled: the camp's first paint
//! (`forecast`), the refine, the forge's measures (`kit::deltas`), the cage and the start sheet.
//!   cargo run -q --profile fast -p riddle-core --example deep -- [--out web/tests/fixtures/deep.json] [--seq]
use riddle_core::{Game, RuleSet};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args.iter().position(|a| a == "--out").and_then(|i| args.get(i + 1)).cloned();
    let load = args.iter().position(|a| a == "--load").and_then(|i| args.get(i + 1)).cloned();
    // `--seq`: one core (the wasm's shape); else the native pool.
    if args.iter().any(|a| a == "--seq") {
        riddle_core::forecast::set_parallel_sims(false);
    }
    let mut g = match &load {
        Some(p) => riddle_core::save::load(&std::fs::read_to_string(p).expect("save")).expect("loads"),
        None => build(),
    };
    println!("lineage: best D{} · heir {} · L{} · kit {:?} · gold ${} · rows {}", g.lineage.best_depth, g.lineage.heir, g.lineage.class_level(), g.lineage.kit, g.lineage.gold, g.lineage.rules().rows.len());
    if let Some(p) = &out {
        let text = riddle_core::save::save(&g);
        std::fs::write(p, &text).expect("write");
        println!("wrote {p} ({} KB)", text.len() / 1024);
    }
    profile(&mut g);
}

fn build() -> Game {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards/075d8e2.raterAM.rules.json")).expect("AM's set");
    let set = RuleSet::parse(&text).expect("parses");
    let mut g = Game::new_literal(2501);
    for u in ["row5", "row6", "row7", "row8", "cond_alert", "supply_cap_5"] {
        g.lineage.unlocks.insert(u.into());
    }
    g.set_rules_raw(set).expect("AM's set");
    // His night's income and his forge (leather, leather +1, mail): played to D11 from D0.
    g.lineage.gold = 400;
    let mut hours = 0;
    while g.lineage.best_depth < 11 && hours < 96 {
        let _ = riddle_core::offline::run_offline_quick(&mut g, 4 * 3600);
        hours += 4;
        while riddle_core::kit::owned(&g.lineage, "armour") < 3 && riddle_core::kit::buy(&mut g, "armour").is_ok() {}
        for _ in 0..2 {
            let _ = g.buy_supply("heal");
        }
    }
    println!("played {hours} h to D{}", g.lineage.best_depth);
    // The absence the rater came back from.
    let _ = g.run_offline(8 * 3600);
    g
}

fn time<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let r = f();
    println!("  {label:<28} {:>7.0} ms", t.elapsed().as_secs_f64() * 1e3);
    r
}

fn profile(g: &mut Game) {
    let rules = g.lineage.rules().clone();
    let sims = riddle_core::forecast::camp_sims(g, &rules);
    println!("camp: {sims} sims · known to D{}", g.lineage.best_depth + 1);
    let f = time("forecast (first paint)", || g.forecast());
    let n = g.panel_cache.borrow().values().next().map(|v| v.len()).unwrap_or(0);
    let ticks: u64 = g.panel_cache.borrow().values().next().map(|v| v.iter().map(|r| r.ticks as u64).sum()).unwrap_or(0);
    println!("    {n} sims · {ticks} ticks ({:.0} per sim) · ends {:?}", ticks as f64 / n.max(1) as f64, f.ends.as_ref().map(|e| (e.bank, e.death)));
    let _ = time("forecast_refine", || g.forecast_refine());
    let k = time("kit::deltas (forge)", || riddle_core::kit::deltas(g));
    for l in &k {
        if let Some(n) = &l.next {
            println!("    {} {} · D{:?} {:+.2} ±{:.2} · bank {:+.2}", l.slot, n.label, n.depth, n.delta.unwrap_or(0.0), n.pm.unwrap_or(0.0), n.bank.unwrap_or(0.0));
        }
    }
    let _ = time("cage_forecast", || g.cage_forecast());
    let _ = time("start_forecast", || g.start_forecast());
    // An edit: the bank row a floor deeper, then its paired move against the set.
    let mut edit = rules.clone();
    if let Some(r) = edit.rows.iter_mut().find(|r| r.verb.v == "bank") {
        if let Some(c) = r.conds.iter_mut().find(|c| c.k == "depth>=") {
            c.n = c.n.map(|n| n + 1);
        }
    }
    let _ = g.set_rules_raw(edit);
    let _ = time("edit: forecast", || g.forecast());
    let _ = time("edit: forecast_vs", || g.forecast_vs(&rules));
    let _ = time("edit: refine", || g.forecast_refine());
    // Cut 27 §2: the edit's scene, after the refine (the camp asks it then: the refined panels).
    let _ = time("edit: forecast_vs (refined)", || g.forecast_vs(&rules));
    let d = time("edit: divergence", || g.divergence(&rules));
    match &d {
        Some(d) => println!("    seed {} · t{} D{} · sent {} → {:?} D{} · new {} → {:?} D{} · moved {:.2} inside {} · {} + {} events", d.seed, d.tick, d.depth, d.sent.text, d.sent_end.tier, d.sent_end.depth, d.new.text, d.new_end.tier, d.new_end.depth, d.moved, d.inside, d.sent.events.len(), d.new.events.len()),
        None => println!("    no divergence"),
    }
    // Cut 27 §1: the send's fold.
    let _ = g.set_rules_raw(rules);
    let f = g.forecast();
    println!("  fold_to {:?} · clears {:?}", f.fold_to, f.depths.iter().map(|d| (d.depth, d.clear.map(|c| (c * 100.0).round() as i32))).collect::<Vec<_>>());
    let _ = g.send();
    let line = time("send → fold", || g.fold());
    println!("    D{}–{} · {:.0}% · +${} · {:?} · {} floors · {} KB", line.from, line.to, line.clear * 100.0, line.gold, line.chips, line.floors.len(), serde_json::to_string(&line).map(|s| s.len() / 1024).unwrap_or(0));
}
