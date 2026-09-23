//! Cut 13 §6: the wire invariants (docs/ITERATION_SPEED.md §1.1) — a native "QA player" that
//! plays the protocol through the engine on seeds 1..=30 and asserts what the QA players
//! reconciled by hand. One line per invariant with counts; exit 1 on any failure. Runs in
//! seconds; `tools/gates.mjs` runs it beside the gate table.
//!   cargo run --profile fast --example qa [-- --seeds 30 --threads 4]
use riddle_core::engine::{salvage_value, GOLD_DIVISOR, GOLD_LEDGER_CAP};
use riddle_core::item::{is_identified, to_inv};
use riddle_core::{Ev, Game};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

/// Per invariant: (checks, failures, the first failure's text).
#[derive(Default, Clone)]
struct Tally(BTreeMap<&'static str, (u32, u32, String)>);

impl Tally {
    fn check(&mut self, name: &'static str, ok: bool, detail: impl FnOnce() -> String) {
        let e = self.0.entry(name).or_insert((0, 0, String::new()));
        e.0 += 1;
        if !ok {
            e.1 += 1;
            if e.2.is_empty() {
                e.2 = detail();
            }
        }
    }
    fn merge(&mut self, other: Tally) {
        for (k, (n, f, d)) in other.0 {
            let e = self.0.entry(k).or_insert((0, 0, String::new()));
            e.0 += n;
            e.1 += f;
            if e.2.is_empty() {
                e.2 = d;
            }
        }
    }
}

/// The gold header is the ledger's sum while the ledger has not evicted a line.
fn check_gold(t: &mut Tally, g: &Game, seed: u64, at: &str) {
    if g.lineage.gold_ledger.len() >= GOLD_LEDGER_CAP {
        return;
    }
    let sum: i32 = g.lineage.gold_ledger.iter().map(|l| l.delta).sum();
    t.check("gold header == ledger sum", sum == g.lineage.gold, || format!("seed {seed} {at}: header ${} ledger ${sum}", g.lineage.gold));
}

/// A learned flavour never labels `?`: every item the lineage can name is known on the wire.
fn check_labels(t: &mut Tally, g: &Game, seed: u64, items: &[riddle_core::item::Item], at: &str) {
    let l = &g.lineage;
    for it in items {
        let w = to_inv(it, &l.facts, &l.flavours);
        let ok = !is_identified(&l.facts, &l.flavours, &it.kind) || (w.known && !w.label.contains('?'));
        t.check("a learned flavour never labels `?`", ok, || format!("seed {seed} {at}: {} labelled {}", it.kind, w.label));
    }
}

/// A gate's `needs` never names a fact the lineage holds.
fn check_needs(t: &mut Tally, g: &Game, seed: u64) {
    let l = &g.lineage;
    for u in g.unlocks() {
        let Some(n) = u.needs.as_deref() else { continue };
        let held = match n.strip_prefix("fact: ") {
            Some(tag) => riddle_core::facts::has_tag_fact(&l.facts, tag),
            None => match n.strip_prefix("find a ") {
                Some(k) => l.facts.contains(&format!("item:{k}")),
                None => false,
            },
        };
        t.check("a gate's `needs` never names a held fact", !held, || format!("seed {seed}: {} needs `{n}`", u.id));
    }
}

/// The forecast's ends sum to 1 and a 0 % death share lists no killers.
fn check_forecast(t: &mut Tally, g: &Game, seed: u64) {
    let f = g.forecast();
    if let Some(e) = &f.ends {
        let sum = e.bank + e.return_ + e.death + e.stall;
        t.check("forecast ends sum to 1 ± 1e-9", (sum - 1.0).abs() < 1e-9, || format!("seed {seed}: ends sum {sum}"));
        t.check("forecast death 0 ⇒ causes empty", e.death > 0.0 || f.causes.is_empty(), || format!("seed {seed}: death {} causes {:?}", e.death, f.causes));
    }
}

/// Cut 14 §1–2: a death screen offers nothing under its baseline (a `dice` death's candidates
/// kept under the bar are flagged `below_bar`), and a `dice` margin names no unused item.
fn check_death(t: &mut Tally, seed: u64, d: &riddle_core::Death) {
    for p in &d.patches {
        t.check("no death screen carries a patch under its baseline (unless below_bar)", p.below_bar || p.survive >= d.baseline - 1e-9, || format!("seed {seed} run {}: {} survives {:.2} · base {:.2} · {}", d.run_id, p.row.describe(), p.survive, d.baseline, d.verdict));
    }
    for p in d.patches.iter().filter(|p| p.below_bar) {
        t.check("no below-bar candidate survives 0 %", p.survive > 0.0, || format!("seed {seed} run {}: {} survives {:.2} · base {:.2}", d.run_id, p.row.describe(), p.survive, d.baseline));
    }
    if d.verdict == "dice" {
        t.check("no dice death's margin names an unused item", !d.margin.contains("unused"), || format!("seed {seed} run {}: `{}`", d.run_id, d.margin));
    }
}

/// Cut 15 §2: on a copy of the lineage given gold, the first gold-buyable unlock buys with
/// gold at `UnlockInfo.gold` — gold down by the price (a ledger line `unlock <id>`), marks
/// untouched — and the next gold price of every other card climbs by a quarter of its first.
fn check_gold_buy(t: &mut Tally, g: &Game, seed: u64) {
    let mut h = g.clone();
    h.lineage.gold_move(100_000, "test");
    let before = h.unlocks();
    let Some(u) = before.iter().find(|u| !u.owned && u.gold > 0 && u.needs.as_deref().is_none_or(|n| n.starts_with('◆'))) else { return };
    let (gold, marks, buys) = (h.lineage.gold, h.lineage.marks, h.lineage.gold_buys);
    let ok = h.buy_unlock_gold(&u.id).is_ok();
    let line = h.lineage.gold_ledger.last().map(|l| (l.delta, l.why.clone()));
    let after = h.unlocks();
    let climbed = before.iter().filter(|b| b.id != u.id && !b.owned && b.gold > 0).all(|b| after.iter().any(|a| a.id == b.id && a.gold == riddle_core::meta::gold_price(b.cost, buys + 1) && a.gold > b.gold));
    t.check(
        "a gold buy spends gold not marks and raises the next price",
        ok && h.lineage.unlocks.contains(&u.id) && h.lineage.gold == gold - u.gold as i32 && h.lineage.marks == marks && line == Some((-(u.gold as i32), format!("unlock {}", u.id))) && climbed,
        || format!("seed {seed}: {} ◆{} ${} · ok {ok} · gold {gold} → {} · marks {marks} → {} · {line:?} · climbed {climbed}", u.id, u.cost, u.gold, h.lineage.gold, h.lineage.marks),
    );
    check_gold(t, &h, seed, "after a gold buy");
}

fn play(seed: u64) -> Tally {
    let mut t = Tally::default();
    let mut g = Game::new(seed);
    check_gold(&mut t, &g, seed, "new");
    check_needs(&mut t, &g, seed);
    check_forecast(&mut t, &g, seed);
    // The camp's shelf goes out as the run's supplies; the vault loadout as `brought`.
    let shelf: Vec<String> = g.lineage.supplies.iter().map(|s| s.kind.clone()).collect();
    g.send();
    {
        let run = g.run.as_ref().unwrap();
        let mut packed: Vec<String> = run.supplies.iter().filter_map(|id| run.hero.inv.iter().chain(run.hero.weapon.iter()).chain(run.hero.armour.iter()).find(|i| i.id == *id).map(|i| i.kind.clone())).collect();
        let mut want = shelf.clone();
        packed.sort();
        want.sort();
        t.check("camp supplies == the run's packed kinds", packed == want, || format!("seed {seed}: shelf {want:?} packed {packed:?}"));
        t.check("`brought` == the loadout", g.snapshot().stake.brought.len() == run.brought.len(), || format!("seed {seed}"));
    }
    // Run to the exit.
    let mut died = None;
    let mut n = 0;
    loop {
        let r = g.step(50);
        n += 1;
        if r.run_over {
            if r.events.iter().any(|e| matches!(e, Ev::Exit { tier, .. } if tier == "death")) {
                died = Some(r.snapshot.run.id);
            }
            if let (Some(p), Some(pe)) = (g.pending_exit.clone(), r.exit_pending.as_ref()) {
                // `ExitPending.worth` is the engine's salvage arithmetic, item by item.
                let want: Vec<i32> = p.items.iter().map(|i| (salvage_value(&i.kind) * p.pct / GOLD_DIVISOR + 50) / 100).collect();
                t.check("`ExitPending.worth` == the salvage arithmetic", pe.worth == want, || format!("seed {seed}: worth {:?} vs {want:?}", pe.worth));
                check_labels(&mut t, &g, seed, &p.items, "exit sheet");
                let gold = g.lineage.gold;
                g.keep(vec![]).unwrap();
                let got = g.lineage.gold_ledger.iter().rev().take_while(|l| l.t == g.lineage.total_turns).filter(|l| l.why == "salvage").map(|l| l.delta).sum::<i32>();
                let sum: i32 = want.iter().sum();
                t.check("keep-nothing salvages ≈ Σ worth (± rounding)", (got - sum).abs() <= want.len() as i32 && g.lineage.gold >= gold, || format!("seed {seed}: salvaged ${got} vs Σ worth ${sum}"));
            }
            break;
        }
        if n > 4000 {
            break;
        }
    }
    check_gold(&mut t, &g, seed, "after the run");
    // The death: its verdict, its patches (none already in the set), the top one applied.
    if let Some(id) = died {
        let d = g.death(id).expect("a death record");
        let rules = g.lineage.rules().clone();
        for p in &d.patches {
            let dup = p.insert_at >= 0 && rules.rows.iter().any(|r| r.conds == p.row.conds && r.verb == p.row.verb);
            t.check("no patch already in the set", !dup, || format!("seed {seed} run {id}: {} is R{}", p.row.describe(), rules.rows.iter().position(|r| *r == p.row).map(|i| i + 1).unwrap_or(0)));
        }
        t.check("a death has a verdict and a trace", (d.verdict == "gap" || d.verdict == "dice") && !d.trace.turns.is_empty(), || format!("seed {seed} run {id}: {} · {} turns", d.verdict, d.trace.turns.len()));
        check_death(&mut t, seed, &d);
        if let Some(p) = d.patches.iter().find(|p| p.insert_at >= 0) {
            let patched = riddle_core::offline::apply_patch(&rules, p, g.lineage.max_rows());
            t.check("the top patch applies through set_rules", g.set_rules(patched).is_ok(), || format!("seed {seed}: {}", p.row.describe()));
        }
    }
    // Buy the first affordable unlock.
    if let Some(u) = g.unlocks().into_iter().find(|u| u.available && u.cost <= g.lineage.marks) {
        let marks = g.lineage.marks;
        t.check("an available unlock buys", g.buy(&u.id).is_ok() && g.lineage.marks == marks - u.cost, || format!("seed {seed}: {}", u.id));
    }
    check_needs(&mut t, &g, seed);
    check_gold_buy(&mut t, &g, seed);
    // The night.
    let before = g.lineage.gold;
    let r = g.run_offline(8 * 3600);
    let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
    t.check("report runs == deaths + banked + returned", r.runs == deaths + r.banked + r.returned, || format!("seed {seed}: {} runs · {deaths} deaths · {} banked · {} returned", r.runs, r.banked, r.returned));
    let b = &g.batch;
    let spent: i32 = b.spent.values().map(|(_, c)| *c).sum();
    t.check("night gold: earned + salvage + wake pay − spent == delta", b.gold_earned + b.salvage_gold + b.wake_pay - spent == g.lineage.gold - before, || format!("seed {seed}: {} + {} + {} − {spent} vs {}", b.gold_earned, b.salvage_gold, b.wake_pay, g.lineage.gold - before));
    t.check("report spent == the batch's", r.spent.iter().map(|s| s.gold).sum::<i32>() == spent, || format!("seed {seed}"));
    check_labels(&mut t, &g, seed, &g.batch.found.clone(), "report found");
    for f in &r.found {
        t.check("report `found` never labels `?` when known", !f.known || !f.label.contains('?'), || format!("seed {seed}: {f:?}"));
    }
    check_gold(&mut t, &g, seed, "after the night");
    check_needs(&mut t, &g, seed);
    check_forecast(&mut t, &g, seed);
    if let Some(d) = &r.worst_death {
        t.check("worst death carries a verdict and a trace", (d.verdict == "gap" || d.verdict == "dice" || d.verdict == "stall") && !d.trace.turns.is_empty(), || format!("seed {seed}: {}", d.verdict));
        check_death(&mut t, seed, d);
    }
    // One more of the night's deaths (the last one that is not the worst), in full.
    if let Some(id) = g.deaths.iter().rev().find(|(id, rec)| !rec.stall && Some(**id) != r.worst_death_id).map(|(id, _)| *id) {
        if let Some(d) = g.death(id) {
            check_death(&mut t, seed, &d);
        }
    }
    // Every stall record: its verdict, a firing patch, the reel's cause == the trace's.
    let stall_ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| rec.stall).map(|(id, _)| *id).collect();
    for id in stall_ids {
        let Some(d) = g.death(id) else { continue };
        let rec = g.deaths.get(&id).cloned().unwrap();
        let fires = d.patches.iter().any(|p| riddle_core::trace::patch_fired_rate(&g, &rec, p) >= 0.5);
        t.check("a stall's verdict names a firing patch", d.verdict == "stall" && fires, || format!("seed {seed} run {id}: {} {:?}", d.verdict, d.patches.iter().map(|p| p.row.describe()).collect::<Vec<_>>()));
        check_death(&mut t, seed, &d);
        let cause = d.cause.strip_prefix("stalled · ").unwrap_or(&d.cause);
        let want = format!("stalled, {}", riddle_core::sifter::stall_short(cause));
        let lines: Vec<&riddle_core::Highlight> = g.batch.highlights.iter().filter(|h| h.run_id == id && h.arc.is_some()).collect();
        if !lines.is_empty() {
            t.check("a stall's reel cause == its trace cause", lines.iter().any(|h| h.arc.as_ref().unwrap().resolution == want), || format!("seed {seed} run {id}: want `{want}`, lines {:?}", lines.iter().map(|h| h.text.clone()).collect::<Vec<_>>()));
        }
    }
    // Buy a supply and drop it: the refund is its price.
    if g.lineage.gold < 100 {
        let top = 100 - g.lineage.gold;
        g.lineage.gold_move(top, "qa top-up");
    }
    if let Some(entry) = g.supply_catalogue().into_iter().find(|s| s.price <= g.lineage.gold && g.lineage.supplies.len() < g.lineage.supply_cap()) {
        let gold = g.lineage.gold;
        if g.buy_supply(&entry.kind).is_ok() {
            let id = g.lineage.supplies.iter().rev().find(|s| s.kind == entry.kind && !s.free).map(|s| s.id).unwrap();
            g.drop_supply(id).unwrap();
            t.check("a dropped supply's refund == its price", g.lineage.gold == gold, || format!("seed {seed}: {} price {} refund {}", entry.kind, entry.price, g.lineage.gold - (gold - entry.price)));
        }
    }
    check_gold(&mut t, &g, seed, "after the shop");
    // A save round-trips the lineage.
    let h = Game::load(&g.save()).expect("load");
    t.check("save/load round-trips the lineage", h.lineage() == g.lineage(), || format!("seed {seed}"));
    t
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 30);
    riddle_core::forecast::set_parallel_sims(false);
    let t0 = std::time::Instant::now();
    let jobs = Arc::new(Mutex::new((1..=seeds).collect::<Vec<u64>>()));
    let out = Arc::new(Mutex::new(Tally::default()));
    // `--threads N` leaves cores to what runs beside it (gates.mjs: the table's quiet
    // per-tick measurement and the dayplayer's chains).
    let threads = (get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4)) as usize).min(seeds as usize).max(1);
    let hs: Vec<_> = (0..threads)
        .map(|_| {
            let jobs = Arc::clone(&jobs);
            let out = Arc::clone(&out);
            std::thread::spawn(move || loop {
                let seed = jobs.lock().unwrap().pop();
                let Some(seed) = seed else { break };
                let t = play(seed);
                out.lock().unwrap().merge(t);
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let t = out.lock().unwrap();
    let mut fails = 0;
    println!("{:<58} {:>8}  result", "invariant", "checks");
    for (name, (n, f, d)) in &t.0 {
        println!("{:<58} {:>8}  {}{}", name, n, if *f == 0 { "PASS".to_string() } else { format!("FAIL ({f})") }, if d.is_empty() { String::new() } else { format!("  — {d}") });
        fails += f;
    }
    println!("qa: {} ({seeds} seeds, {:.1}s)", if fails == 0 { "all PASS" } else { "FAIL" }, t0.elapsed().as_secs_f64());
    if fails > 0 {
        std::process::exit(1);
    }
}
