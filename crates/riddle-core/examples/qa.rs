//! Cut 13 §6: the wire invariants (docs/ITERATION_SPEED.md §1.1) — a native "QA player" that
//! plays the protocol through the engine on seeds 1..=30 and asserts what the QA players
//! reconciled by hand. One line per invariant with counts; exit 1 on any failure. Runs in
//! seconds; `tools/gates.mjs` runs it beside the gate table.
//!   cargo run --profile fast --example qa [-- --seeds 30 --threads 4]
use riddle_core::engine::{salvage_coins, salvage_value, GOLD_DIVISOR, GOLD_LEDGER_CAP};
use riddle_core::item::{is_identified, to_inv};
use riddle_core::engine::ExitTier;
use riddle_core::rules::{Cond, Row, Verb};
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

/// QA on 3d71c33 (`D8 44% · bank 57%` under `depth ≥ 8 → bank`; `bank 42%`, then 0 of 16
/// banked overnight): on a copy of the lineage running the good set with its bank row at
/// `depth ≥ d` (d = the good set's own 5, 8 and the lineage's best), the forecast's bank
/// share is never above the reach of D`d` when the bars show it, and — summed over the seeds
/// (`BankTally`) — it is what sends from this camp do: `SENDS` real sends on the night's own
/// run seeds per set.
fn check_forecast_bank(t: &mut Tally, bank: &mut BankTally, g: &Game, seed: u64) {
    let mut h = g.clone();
    h.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    let mut depths = vec![5, 8, h.lineage.best_depth.max(1)];
    depths.sort();
    depths.dedup();
    for d in depths {
        let mut set = riddle_core::probes::good();
        let at = set.rows.iter().position(|r| r.verb.v == "bank").expect("the good set banks");
        if d != 5 {
            set.rows[at] = Row::new(vec![Cond::n("depth>=", d as i32)], Verb::new("bank"));
        }
        if h.set_rules_raw(set).is_err() {
            continue;
        }
        // `BANK_SIMS` of the forecast's own panel (its first seeds; the bars and the ends are
        // one panel at any count), to keep the job's time.
        let f = riddle_core::forecast::forecast_with(&h, h.lineage.rules(), BANK_SIMS);
        let Some(e) = f.ends.clone() else { continue };
        if let Some(r) = f.depths.iter().find(|x| x.depth == d) {
            t.check("forecast bank ≤ reach at the bank row's depth", e.bank <= r.reach + 1e-9, || format!("seed {seed} depth ≥ {d}: bank {:.2} · D{d} {:.2}", e.bank, r.reach));
        }
        // The panel's nominal count (a budget-cut panel ran fewer: the band is then narrower
        // than its own, the stricter reading).
        let n = BANK_SIMS as f64;
        let mut banked = 0u32;
        for j in 0..SENDS {
            let mut s = h.clone();
            s.lineage.next_run_id += j;
            s.lineage.rest_left = 0;
            s.start_run(None);
            s.run_to_end(riddle_core::engine::MAX_TURNS_PER_RUN);
            banked += (s.run.as_ref().unwrap().over == Some(ExitTier::Bank)) as u32;
        }
        let k = if d == 5 { 0 } else { 1 };
        bank.0[k].0 += e.bank * n;
        bank.0[k].1 += n;
        bank.0[k].2 += banked as f64;
        bank.0[k].3 += SENDS as f64;
    }
}

/// Real sends per set per seed for the forecast-vs-sends check, and the forecast's sims.
/// Cut 19: 18 (was 6) — at 6 the sends' sample (186) read 0.763 against the panel's 0.832
/// after the night moved; at 12 0.782, at 18 0.803: the gap was the sends' own noise (a fresh
/// lineage's panel and 60 sends agree to 0.01, before and after a night), so the check runs on
/// a sample that can tell. The band narrows with it.
const SENDS: u32 = 18;
const BANK_SIMS: u32 = 20;

/// Σ over seeds, per [the good set, its bank-depth variants]: (forecast banks, forecast sims,
/// real banks, real sends).
#[derive(Default, Clone)]
struct BankTally([(f64, f64, f64, f64); 2]);

/// Cut 14 §1–2: a death screen offers nothing under its baseline (a `dice` death's candidates
/// kept under the bar are flagged `below_bar`), and a `dice` margin names no unused item.
fn check_death(t: &mut Tally, seed: u64, d: &riddle_core::Death) {
    // QA on 92eb880 (qaM: `R2 attack saved him.` on the death of a hero R2 was fighting for).
    t.check("a death's notes never say `saved him`", d.notes.iter().all(|n| !n.contains("saved him")), || format!("seed {seed} run {}: {:?}", d.run_id, d.notes));
    // QA on 92eb880 (qaM: `DICE` over three `survives 100% · below bar`): a dice death whose
    // patches none beat the base says so, and only then.
    let beaten = d.patches.iter().any(|p| p.survive > d.baseline + 1e-9);
    t.check("`nothing_beats_base` ⇔ a dice death no patch beats", d.nothing_beats_base == (d.verdict == "dice" && !d.patches.is_empty() && !beaten), || format!("seed {seed} run {}: {} base {:.2} flag {}", d.run_id, d.verdict, d.baseline, d.nothing_beats_base));
    for p in &d.patches {
        t.check("no death screen carries a patch under its baseline (unless below_bar)", p.below_bar || p.survive >= d.baseline - 1e-9, || format!("seed {seed} run {}: {} survives {:.2} · base {:.2} · {}", d.run_id, p.row.describe(), p.survive, d.baseline, d.verdict));
    }
    // Cut 15 §6: 0 % candidates show only when nothing survives (a dice death is never empty).
    let any_survives = d.patches.iter().any(|p| p.survive > 0.0);
    for p in d.patches.iter().filter(|p| p.insert_at >= 0) {
        t.check("no patch survives 0 % beside one that survives", !any_survives || p.survive > 0.0, || format!("seed {seed} run {}: {} survives {:.2} · base {:.2}", d.run_id, p.row.describe(), p.survive, d.baseline));
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

/// QA on 92eb880 (qaM: "D6 32 → 38 → 50 just from opening sheets"): the camp's forecast is a
/// function of what the player controls — a forecast, then every engine read the camp's
/// sheets make (lineage, vocabulary, the unlock shelf and its deltas, the supply shop, the
/// death screen, the same rules and loadout set again), then a forecast: identical. After the
/// refine pass a read is the refined panel (never back to the first pass), and from a cold
/// cache both passes are what they were. Every third seed runs the refine leg (100 sims).
fn check_forecast_reads(t: &mut Tally, g: &mut Game, seed: u64, died: Option<u32>) {
    let reads = |g: &mut Game| {
        let _ = g.lineage();
        let _ = g.vocabulary();
        let _ = g.unlocks();
        let _ = g.unlock_deltas();
        let _ = g.supply_catalogue();
        if let Some(id) = died {
            let _ = g.death(id);
        }
        let r = g.lineage.rules().clone();
        let _ = g.set_rules(r);
        let l = g.loadout.clone();
        g.loadout(l);
    };
    let first = g.forecast();
    reads(g);
    let again = g.forecast();
    t.check("forecast → every sheet's reads → forecast: identical", again == first, || format!("seed {seed}: {:?} → {:?}", first.depths.iter().map(|d| d.reach).collect::<Vec<_>>(), again.depths.iter().map(|d| d.reach).collect::<Vec<_>>()));
    if !seed.is_multiple_of(3) {
        return;
    }
    let refined = g.forecast_refine();
    reads(g);
    let after = g.forecast();
    t.check("after the refine a forecast read is the refined panel", after == refined, || format!("seed {seed}: refined {:?} read {:?}", refined.depths.iter().map(|d| d.reach).collect::<Vec<_>>(), after.depths.iter().map(|d| d.reach).collect::<Vec<_>>()));
    let cold = g.clone();
    cold.panel_cache.borrow_mut().clear();
    cold.forecast_cache.borrow_mut().clear();
    t.check("a cold cache reads the same two passes", cold.forecast() == first && cold.forecast_refine() == refined, || format!("seed {seed}"));
}

/// QA on 92eb880 (qaM: `R3 fired 0 of 16 runs: hp < 30% → drink heal · heal unknown` under R1
/// `hp < 30% → return`): after the night, a row that never fired and that an earlier row
/// shadows names it on its pending line, and a shadowed row never fires.
fn check_shadowed(t: &mut Tally, g: &Game, seed: u64) {
    check_shadowed_on(t, g, seed);
    // The qaM set on a copy: `hp < 30% → return` on top, the lineage's rows, `hp < 20% →
    // return` under them (shadowed by R1), one hour offline.
    let mut h = g.clone();
    h.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 30)], Verb::new("return")));
    // Cut 19 §2: a return walks (a foe in the way fails it), so it shadows only a later return
    // it always pre-empts — the qaM heal row under it is no longer shadowed.
    set.rows.push(Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")));
    if h.set_rules_raw(set).is_err() {
        return;
    }
    h.run_offline(3600);
    let last = h.lineage.rules().rows.len() - 1;
    t.check("the qaM shadow is found (R1 return over a later return)", h.lineage.shadowed_by(h.lineage.rules()).get(last).copied().flatten() == Some(0), || format!("seed {seed}"));
    check_shadowed_on(t, &h, seed);
}

fn check_shadowed_on(t: &mut Tally, g: &Game, seed: u64) {
    let rules = g.lineage.rules();
    let shadowed = g.lineage.shadowed_by(rules);
    let pending = riddle_core::meta::pending(g);
    for (i, by) in shadowed.iter().enumerate() {
        let Some(by) = by else { continue };
        let n = g.batch.row_runs.get(i).copied().unwrap_or(0);
        t.check("a shadowed row never fires", n == 0, || format!("seed {seed}: R{} (shadowed by R{}) fired in {n} runs", i + 1, by + 1));
        let line = pending.iter().find(|l| l.starts_with(&format!("R{} fired ", i + 1)));
        t.check("a shadowed row's pending line names its shadow", line.is_none_or(|l| l.ends_with(&format!(" · shadowed by R{}", by + 1))), || format!("seed {seed}: {line:?}"));
    }
}

fn play(seed: u64) -> (Tally, BankTally) {
    let mut t = Tally::default();
    let mut bank = BankTally::default();
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
                // `ExitPending.worth` is the engine's salvage arithmetic, item by item: the
                // whole's coins (`(carry + Σ cents) ÷ 100`) apportioned by largest remainder.
                let want = salvage_coins(&p.items, p.pct, g.lineage.gold_carry);
                let cents: i32 = p.items.iter().map(|i| salvage_value(&i.kind) * p.pct / GOLD_DIVISOR).sum();
                t.check("`ExitPending.worth` == the salvage arithmetic", pe.worth == want && want.iter().sum::<i32>() == (g.lineage.gold_carry + cents).max(0) / 100, || format!("seed {seed}: worth {:?} vs {want:?}", pe.worth));
                check_labels(&mut t, &g, seed, &p.items, "exit sheet");
                let (gold, head) = (g.lineage.gold, g.batch.salvage_gold);
                g.keep(vec![]).unwrap();
                let sum: i32 = want.iter().sum();
                // QA on 92eb880 (qaM: the sheet's `aggravate $2`, the report's `$1`): exact.
                t.check("keep-nothing salvages Σ worth exactly", g.lineage.gold - gold == sum && g.batch.salvage_gold - head == sum, || format!("seed {seed}: salvaged ${} vs Σ worth ${sum}", g.lineage.gold - gold));
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
            // (a cut — `remove` / `replace` — names a row of the set by design)
            let dup = p.insert_at >= 0 && !p.remove && !p.replace && rules.rows.iter().any(|r| r.conds == p.row.conds && r.verb == p.row.verb);
            t.check("no patch already in the set", !dup, || format!("seed {seed} run {id}: {} is R{}", p.row.describe(), rules.rows.iter().position(|r| *r == p.row).map(|i| i + 1).unwrap_or(0)));
        }
        t.check("a death has a verdict and a trace", (d.verdict == "gap" || d.verdict == "dice" || d.verdict == "row") && !d.trace.turns.is_empty(), || format!("seed {seed} run {id}: {} · {} turns", d.verdict, d.trace.turns.len()));
        // Cut 19 §4: a `row` verdict names a row of the set that acted on the death tick (the
        // trace's last turn), and its first patch cuts that row.
        if d.verdict == "row" || d.cause_row.is_some() {
            let last = d.trace.turns.last().map(|x| x.row);
            let ok = d.verdict == "row" && d.cause_row.is_some_and(|r| Some(r as i32) == last && d.rules.as_ref().is_some_and(|rs| rs.rows.get(r as usize).is_some_and(|x| !x.is_card()))) && d.patches.first().is_some_and(|p| (p.remove || p.replace) && Some(p.insert_at) == last);
            t.check("a `row` verdict names a row that fired on the death tick", ok, || format!("seed {seed} run {id}: {} R{:?} last {:?}", d.verdict, d.cause_row, last));
        }
        check_death(&mut t, seed, &d);
        t.check("death()'s patches are camp_pending until death_deltas", d.patches.iter().all(|p| p.camp_pending), || format!("seed {seed} run {id}"));
        // QA on 23ed91f (`rest · reach +8%`, then the camp's D6 34 % → 64 %): the patch's
        // `reach` (`death_deltas`) is the camp forecast's own move at its depth once applied.
        // Sampled on every third seed: the number is not an estimate of the camp's but the
        // camp's panel itself (same seeds, budget, lineage state), so it holds exactly or a
        // code path differs (insert position, lineage state) — any seed exercises those paths,
        // and 30/30 were exact when every seed ran; four panels a seed are the leg's cost.
        let deltas = if seed.is_multiple_of(3) { g.death_deltas(id) } else { None };
        if let Some(ps) = &deltas {
            // Cut 19 §4: the camp's numbers may re-rank the unpinned patches — the same patches,
            // and death() reads that order from then on.
            let key = |p: &riddle_core::wire::Patch| (serde_json::to_string(&p.row).unwrap_or_default(), p.insert_at);
            let mut a: Vec<_> = ps.iter().map(key).collect();
            let mut b: Vec<_> = d.patches.iter().map(key).collect();
            a.sort();
            b.sort();
            let again = g.death(id).map(|x| x.patches.iter().map(key).collect::<Vec<_>>());
            t.check("death_deltas returns death()'s patches, measured", a == b && ps.iter().all(|p| !p.camp_pending) && again == Some(ps.iter().map(key).collect()), || format!("seed {seed} run {id}"));
        }
        let top = deltas.as_ref().unwrap_or(&d.patches).iter().find(|p| p.insert_at >= 0).cloned();
        if let Some(p) = top {
            let patched = riddle_core::offline::apply_patch(&rules, &p, g.lineage.max_rows());
            let before = deltas.is_some().then(|| g.forecast());
            t.check("the top patch applies through set_rules", g.set_rules(patched).is_ok(), || format!("seed {seed}: {}", p.row.describe()));
            if let Some(before) = before {
                let after = g.forecast();
                let bar = |f: &riddle_core::Forecast| f.depths.iter().find(|x| x.depth == p.forecast_depth).map(|x| (x.reach, x.pm.unwrap_or(0.0)));
                let ok = match (bar(&before), bar(&after)) {
                    (Some((b, _)), Some((a, pm))) => ((a - b) - p.forecast_delta).abs() <= pm.max(p.forecast_pm) + 1e-9,
                    _ => false,
                };
                t.check("a patch's reach == the camp forecast's move once applied (± its bar)", ok, || format!("seed {seed} run {id}: {} at R{} · reach {:+.3} at D{} · camp {:?} → {:?}", p.row.describe(), p.insert_at + 1, p.forecast_delta, p.forecast_depth, bar(&before), bar(&after)));
            }
        }
    }
    check_forecast_reads(&mut t, &mut g, seed, died);
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
    let rows: i32 = r.salvaged.iter().map(|x| x.gold).sum();
    let head = r.gold.as_ref().map(|x| x.salvage).unwrap_or(0);
    t.check("report SALVAGED rows sum to the header's salvage", rows == head, || format!("seed {seed}: rows ${rows} vs +${head}"));
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
    check_shadowed(&mut t, &g, seed);
    check_forecast_bank(&mut t, &mut bank, &g, seed);
    if let Some(d) = &r.worst_death {
        t.check("worst death carries a verdict and a trace", (d.verdict == "gap" || d.verdict == "dice" || d.verdict == "stall" || d.verdict == "row") && !d.trace.turns.is_empty(), || format!("seed {seed}: {}", d.verdict));
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
    (t, bank)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 30);
    riddle_core::forecast::set_parallel_sims(false);
    let t0 = std::time::Instant::now();
    let jobs = Arc::new(Mutex::new((1..=seeds).collect::<Vec<u64>>()));
    let out = Arc::new(Mutex::new(Tally::default()));
    let banks = Arc::new(Mutex::new(BankTally::default()));
    // `--threads N` leaves cores to what runs beside it (gates.mjs: the table's quiet
    // per-tick measurement and the dayplayer's chains).
    let threads = (get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4)) as usize).min(seeds as usize).max(1);
    let hs: Vec<_> = (0..threads)
        .map(|_| {
            let jobs = Arc::clone(&jobs);
            let out = Arc::clone(&out);
            let banks = Arc::clone(&banks);
            std::thread::spawn(move || loop {
                let seed = jobs.lock().unwrap().pop();
                let Some(seed) = seed else { break };
                let (t, b) = play(seed);
                out.lock().unwrap().merge(t);
                let mut all = banks.lock().unwrap();
                for (a, x) in all.0.iter_mut().zip(b.0) {
                    a.0 += x.0;
                    a.1 += x.1;
                    a.2 += x.2;
                    a.3 += x.3;
                }
            })
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let mut t = out.lock().unwrap().clone();
    // The forecast's bank share, summed over the seeds, within the pooled 95 % band of the
    // real sends' (the two samples' binomial ±).
    for (name, (fb, fn_, rb, rn)) in ["forecast bank ≈ real sends (good set, Σ seeds)", "forecast bank ≈ real sends (depth ≥ d → bank, Σ seeds)"].into_iter().zip(banks.lock().unwrap().0) {
        let (pf, pr) = (fb / fn_.max(1.0), rb / rn.max(1.0));
        let p = (fb + rb) / (fn_ + rn).max(1.0);
        let band = 1.96 * (p * (1.0 - p) * (1.0 / fn_.max(1.0) + 1.0 / rn.max(1.0))).sqrt();
        t.check(name, (pf - pr).abs() <= band + 1e-9, || format!("forecast {pf:.3} over {fn_} sims vs sends {pr:.3} over {rn}, band ±{band:.3}"));
        println!("{name}: forecast {pf:.3} ({fn_} sims) · sends {pr:.3} ({rn}) · ±{band:.3}");
    }
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
