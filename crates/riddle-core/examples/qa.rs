//! Cut 13 §6: the wire invariants (docs/ITERATION_SPEED.md §1.1) — a native "QA player" that
//! plays the protocol through the engine on seeds 1..=30 and asserts what the QA players
//! reconciled by hand. One line per invariant with counts; exit 1 on any failure. ~700
//! thread-seconds: ~45 s alone on the cores; `tools/gates.mjs` runs it beside the gate table.
//! One job pool: a seed's prelude plays the protocol to the night and hands the night's legs
//! (forecast vs sends per depth, the short list, the stall, salvage and shadow legs, the cold
//! cache) to the pool on copies of the game — the checks and their counts are the sequential
//! order's. `METRICS_PHASES=1` prints each job's wall and each leg's thread-seconds.
//!   cargo run --profile fast --example qa [-- --seeds 30 --threads 24]
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

/// `METRICS_PHASES=1`: thread-seconds per leg of `play`, summed over the seeds (stderr).
#[derive(Default, Clone)]
struct Laps(BTreeMap<&'static str, f64>, Option<std::time::Instant>);

impl Laps {
    /// The time since the previous lap (or since the start) goes to `name`.
    fn lap(&mut self, name: &'static str) {
        let now = std::time::Instant::now();
        let t = self.1.replace(now).unwrap_or(now);
        *self.0.entry(name).or_default() += (now - t).as_secs_f64();
    }
    fn merge(&mut self, other: &Laps) {
        for (k, v) in &other.0 {
            *self.0.entry(k).or_default() += v;
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

/// QA on 1a2a4a9 (qaO: the keep sheet's `sold blink $2 · fear ×2 $3`, the report's SALVAGED
/// `teleport ×9 · identify ×4` while the forge said `brittle scroll?`): no item label on the
/// wire names a potion or scroll kind the lineage has not identified. `names` are the wire's
/// item labels (salvage and spent rows, `spent_on`, found and pending items, forge rows).
fn check_names<'a>(t: &mut Tally, g: &Game, seed: u64, names: impl IntoIterator<Item = &'a str>, at: &str) {
    let l = &g.lineage;
    let hidden: Vec<String> = l.flavours.potion.keys().chain(l.flavours.scroll.keys()).filter(|k| !is_identified(&l.facts, &l.flavours, k)).map(|k| k.replace('_', " ")).collect();
    for name in names {
        let n = name.replace('_', " ");
        let n = n.strip_suffix(" potion").or_else(|| n.strip_suffix(" scroll")).unwrap_or(&n).to_string();
        t.check("no wire label names an unidentified kind", !hidden.contains(&n), || format!("seed {seed} {at}: `{name}`"));
    }
}

/// The exit lines' and the report's item labels (`check_names`).
fn check_report_names(t: &mut Tally, g: &Game, seed: u64, r: &riddle_core::ReturnReport, at: &str) {
    let mut names: Vec<&str> = Vec::new();
    names.extend(r.salvaged.iter().map(|x| x.kind.as_str()));
    names.extend(r.spent.iter().map(|x| x.kind.as_str()));
    names.extend(r.found.iter().map(|x| x.label.as_str()));
    for x in &r.exits {
        names.extend(x.salvaged.iter().map(|s| s.kind.as_str()));
        names.extend(x.spent_on.iter().map(|s| s.as_str()));
    }
    let forge: Vec<String> = g.lineage().forge.keys().cloned().collect();
    names.extend(forge.iter().map(|s| s.as_str()));
    names.extend(r.stolen.iter().map(|x| x.label.as_str()));
    names.extend(r.shelved.iter().map(|x| x.kind.as_str()));
    for x in &r.exits {
        names.extend(x.stolen.iter().map(|s| s.as_str()));
    }
    check_names(t, g, seed, names.iter().copied(), at);
    // QA on a946e04 (qaS: SALVAGED `blue potion? ×8` beside `poison ×3`; STOLEN `black potion?`
    // beside LEARNED `confusion (black)`): no report label names a flavour whose kind the
    // lineage has identified (`Lineage.renamed`'s keys). (An exit line is its exit's: it may
    // predate the identification — the client reads it through `renamed`.)
    let renamed = g.lineage().renamed;
    let mut now: Vec<&str> = Vec::new();
    now.extend(r.salvaged.iter().map(|x| x.kind.as_str()));
    now.extend(r.spent.iter().map(|x| x.kind.as_str()));
    now.extend(r.found.iter().map(|x| x.label.as_str()));
    now.extend(r.stolen.iter().map(|x| x.label.as_str()));
    now.extend(r.shelved.iter().map(|x| x.kind.as_str()));
    now.extend(forge.iter().map(|s| s.as_str()));
    for n in now {
        t.check("no wire label names a flavour whose kind is identified", !renamed.contains_key(n), || format!("seed {seed} {at}: `{n}` is {}", renamed[n]));
    }
}

/// QA on 1a2a4a9 (qaO: `hp 3 foes 2 · R2 foes not ≥1` under `foes ≥ 1 → attack`): a trace
/// turn never gives `foes not ≥N` beside a `foes` column of N or more — the reason says why
/// they did not count (`foes fleeing` / `foes appeared after`).
fn check_foe_reasons(t: &mut Tally, seed: u64, trace: &riddle_core::Trace, at: &str) {
    for x in &trace.turns {
        for w in x.rows.iter().flatten() {
            let n = w.why.strip_prefix("foes not ≥").and_then(|n| n.parse::<i32>().ok());
            t.check("no `foes not ≥N` beside a foes column of N+", n.is_none_or(|n| x.foes < n), || format!("seed {seed} {at}: t{} foes {} (rules {}) · R{} {}", x.t, x.foes, x.rule_foes, w.row + 1, w.why));
            t.check("a row reason is from the table", riddle_core::turn::row_reason_ok(&w.why), || format!("seed {seed} {at}: `{}`", w.why));
        }
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

/// QA on 1a2a4a9 (qaP: the short list reshuffled between two opens; PENDING and UNLOCKS named
/// different cards): the short list is the same from `unlocks()`, `unlock_deltas()` and
/// `unlocks()` again, holds at most `SHORT_LIST` cards and the pinned one, and the report's
/// unlock lines are its available cards.
fn check_short(t: &mut Tally, g: &Game, seed: u64) {
    let pick = |c: Vec<riddle_core::UnlockInfo>| c.into_iter().filter(|u| u.short).map(|u| u.id).collect::<Vec<_>>();
    let a = pick(g.unlocks());
    let b = pick(g.unlock_deltas());
    let c = pick(g.unlocks());
    let pinned: Vec<String> = g.unlocks().into_iter().filter(|u| u.pinned).map(|u| u.id).collect();
    t.check("the unlock short list is stable across reads", a == b && b == c && a.len() <= riddle_core::meta::SHORT_LIST.max(pinned.len()) && pinned.iter().all(|p| a.contains(p)), || format!("seed {seed}: {a:?} · {b:?} · {c:?} · pinned {pinned:?}"));
    let pending: Vec<String> = riddle_core::meta::pending(g).into_iter().filter_map(|l| l.strip_prefix("unlock ").and_then(|x| x.split(' ').next()).map(String::from)).collect();
    let avail: Vec<String> = g.unlocks().into_iter().filter(|u| u.short && u.available).map(|u| u.id).collect();
    t.check("PENDING's unlocks are the short list's available cards", pending == avail, || format!("seed {seed}: {pending:?} vs {avail:?}"));
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
/// The bank rows' depths `check_forecast_bank_at` plays for this lineage.
fn bank_depths(g: &Game) -> Vec<u32> {
    let mut depths = vec![5, 8, g.lineage.best_depth.max(1)];
    depths.sort();
    depths.dedup();
    depths
}

/// One depth of the check (the depths are independent copies of the lineage: one job each).
fn check_forecast_bank_at(t: &mut Tally, bank: &mut BankTally, g: &Game, seed: u64, d: u32) {
    let mut h = g.clone();
    h.lineage.unlocks.extend(["row5", "row6", "row7", "row8", "throw"].map(String::from));
    {
        let mut set = riddle_core::probes::good();
        let at = set.rows.iter().position(|r| r.verb.v == "bank").expect("the good set banks");
        if d != 5 {
            set.rows[at] = Row::new(vec![Cond::n("depth>=", d as i32)], Verb::new("bank"));
        }
        if h.set_rules_raw(set).is_err() {
            return;
        }
        // `BANK_SIMS` of the forecast's own panel (its first seeds; the bars and the ends are
        // one panel at any count), to keep the job's time.
        let f = riddle_core::forecast::forecast_with(&h, h.lineage.rules(), BANK_SIMS);
        let Some(e) = f.ends.clone() else { return };
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
struct BankTally([(f64, f64, f64, f64); 2], StallSum);

/// QA on 1a2a4a9 (qaP): Σ over the stall leg's records — (the unpatched set's forecast stall
/// share, the top patch's set's, records).
#[derive(Default, Clone, Copy)]
struct StallSum(f64, f64, u32);

/// Cut 14 §1–2: a death screen offers nothing under its baseline (a `dice` death's candidates
/// kept under the bar are flagged `below_bar`), and a `dice` margin names no unused item.
fn check_death(t: &mut Tally, g: &Game, seed: u64, d: &riddle_core::Death) {
    // QA on e75ec29 (qaQ: the gem applied `83 %` above two `100 %`; `92 %` above `100 %`): the
    // head survives within `SURVIVE_BAND` of the best shown — on a boss death the escape family
    // does not count as the best (Cut 6 §8; a boss death's escape patches sit below the rest).
    // QA on 778fa1b (qaU: `hp < 40% → return · survives 100%` led; applied, `D5 −54`): nor does
    // an exit patch that costs `EXIT_COST` of reach beside a patch that beats the base
    // (`trace::counts_as_best`).
    let boss = g.deaths.get(&d.run_id).is_some_and(|r| r.boss.is_some());
    let counts = |p: &&riddle_core::wire::Patch| riddle_core::trace::counts_as_best(p, &d.patches, d.baseline, boss);
    if let (Some(head), Some(best)) = (d.patches.first(), d.patches.iter().filter(counts).map(|p| p.survive).reduce(f64::max)) {
        let ok = !counts(&head) || head.survive >= best - riddle_core::trace::SURVIVE_BAND - 1e-9;
        t.check("the top patch survives within 10 pts of the best shown", ok, || format!("seed {seed} run {}: {} {:.2} vs best {best:.2} · {}", d.run_id, head.row.describe(), head.survive, d.verdict));
    }
    // QA on 778fa1b: every shown patch says whether it ends the run (`exits`), and a costly
    // exit never leads a list with an alternative that beats the base.
    for p in &d.patches {
        t.check("a patch's `exits` ⇔ its row returns or banks", p.exits == riddle_core::trace::patch_exits(p), || format!("seed {seed} run {}: {} exits {}", d.run_id, p.row.describe(), p.exits));
    }
    if let Some(head) = d.patches.first() {
        let pinned = g.deaths.get(&d.run_id).is_some_and(|r| r.counter.as_ref() == Some(&head.row)) || head.root.is_some() || head.remove || head.replace;
        t.check("a costly exit never leads beside an alternative", pinned || !(riddle_core::trace::costly_exit(head) && riddle_core::trace::exit_alternative(&d.patches, d.baseline)), || format!("seed {seed} run {}: {} reach {:+.2} · {:?}", d.run_id, head.row.describe(), head.forecast_delta, d.patches.iter().map(|p| (p.row.describe(), p.survive)).collect::<Vec<_>>()));
    }
    // QA on 778fa1b (qaU: `foe: boss → attack boss` offered while the editor's lists had
    // neither): a patch's row is written in the lineage's vocabulary — every cond a token the
    // editor offers, the verb one it lists (a card row is its card's; the unlock pseudo-patch
    // names a locked cond by design).
    let v = g.vocabulary();
    for p in d.patches.iter().filter(|p| p.insert_at >= 0 && !p.remove && !p.row.is_card()) {
        let conds = p.row.conds.iter().all(|c| v.conds.iter().any(|x| x.same_token(c)));
        t.check("a patch's conds and verb are in the vocabulary", conds && v.verbs.contains(&p.row.verb), || format!("seed {seed} run {}: {} (conds {conds}, verb {})", d.run_id, p.row.describe(), v.verbs.contains(&p.row.verb)));
    }
    // QA on e75ec29 (qaQ: `survives 100% · base 100%` under GAP): replays that all survive
    // unpatched did not reproduce the death — it is never a `gap` or a `row`.
    t.check("a gap or row verdict has a baseline under 100 %", !(d.verdict == "gap" || d.verdict == "row") || d.baseline < 1.0 - 1e-9, || format!("seed {seed} run {}: {} base {:.2}", d.run_id, d.verdict, d.baseline));
    // QA on a946e04 (qaS: an archer death's notes `Goblin Captain: summoner.`, the run's last
    // note a floor earlier; a poison death's `The black one: confusion.`): a death's note names
    // its killer or the harm that killed him (a stall's notes say what it paid).
    if d.verdict != "stall" {
        let gamble = g.deaths.get(&d.run_id).is_some_and(|r| r.gamble_row.is_some());
        let names = |n: &str| riddle_core::trace::death_note_names(n, &d.cause) || (gamble && ["poison", "gas", "fire"].iter().any(|c| riddle_core::trace::death_note_names(n, c)));
        t.check("a death's note names the killer or the killing harm", d.notes.iter().all(|n| names(n)), || format!("seed {seed} run {}: {} · {:?}", d.run_id, d.cause, d.notes));
        // (qaS: `slain by goblin_archer`)
        let slain = d.morgue.lines().nth(1).and_then(|l| l.split(" · ").find_map(|p| p.strip_prefix("slain by "))).unwrap_or("");
        t.check("the morgue names the killer by its title", !slain.contains('_'), || format!("seed {seed} run {}: slain by {slain}", d.run_id));
    }
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
        // (Cut 6 §8: on a boss death a lone return is joined by the best other candidate — the
        // wall's counter even at 0 %, flagged `below_bar`: named as what was tried, not advice.)
        t.check("no patch survives 0 % beside one that survives", !any_survives || p.survive > 0.0 || p.below_bar, || format!("seed {seed} run {}: {} survives {:.2} · base {:.2}", d.run_id, p.row.describe(), p.survive, d.baseline));
    }
    if d.verdict == "dice" {
        t.check("no dice death's margin names an unused item", !d.margin.contains("unused"), || format!("seed {seed} run {}: `{}`", d.run_id, d.margin));
    }
    // QA on 1a2a4a9 (qaO: `heal unused · 1 unknown unused` beside `R1 no unknown`).
    let said_none = d.trace.turns.last().and_then(|x| x.rows.as_ref()).is_some_and(|rows| rows.iter().any(|w| w.why == "no unknown"));
    t.check("a margin's unknown count never contradicts a row's `no unknown`", !(said_none && d.margin.contains("unknown unused")), || format!("seed {seed} run {}: `{}`", d.run_id, d.margin));
    check_foe_reasons(t, seed, &d.trace, &format!("run {} death trace", d.run_id));
    // QA on 1a2a4a9 (qaP: `+ drop R1` on every patch, R1 the return row).
    if let Some(rules) = &d.rules {
        for p in d.patches.iter().filter(|p| p.drops.is_some()) {
            let r = p.drops.and_then(|i| rules.rows.get(i as usize));
            let ok = r.is_some_and(|r| !r.is_card() && !matches!(r.verb.v.as_str(), "return" | "bank") && r.verb.v != p.row.verb.v);
            t.check("a patch's `drops` is never an exit row or its own verb's", ok, || format!("seed {seed} run {}: {} drops {:?}", d.run_id, p.row.describe(), r.map(|r| r.describe())));
        }
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
fn check_forecast_reads(t: &mut Tally, lp: &mut Laps, pool: &Pool, g: &mut Game, seed: u64, died: Option<u32>) {
    let reads = |g: &mut Game, lp: &mut Laps| {
        let _ = g.lineage();
        let _ = g.vocabulary();
        let _ = g.unlocks();
        lp.lap("reads: lineage, vocabulary, unlocks");
        let _ = g.unlock_deltas();
        lp.lap("reads: unlock_deltas");
        let _ = g.supply_catalogue();
        if let Some(id) = died {
            let _ = g.death(id);
        }
        lp.lap("reads: supplies, death");
        let r = g.lineage.rules().clone();
        let _ = g.set_rules(r.clone());
        let l = g.loadout.clone();
        g.loadout(l);
        lp.lap("reads: set_rules, loadout");
        // QA on 1a2a4a9 (qaO: `death 17% ↔ 13%`, `D4 68%±13 ↔ 67%±9` on opening the vault,
        // `D5 82%±11 → 85%±7` on opening `edit`): what the camp's panels send when opened or
        // re-applied — the cage picker's panels, the same preferences and repeat again, the
        // loadout in another order, the same rows re-tagged by the editor.
        let _ = g.cage_forecast();
        let (vp, kp, rep) = (g.lineage.vault_pref.clone(), g.lineage.keep_pref.clone(), g.lineage().repeat);
        let _ = g.set_vault_pref(&vp);
        let _ = g.set_keep_pref(&kp);
        lp.lap("reads: cage_forecast");
        g.set_restock(rep);
        let mut l = g.loadout.clone();
        l.reverse();
        g.loadout(l);
        lp.lap("reads: prefs, reversed loadout");
        // (each pass re-tags every row the other way: `player` ↔ `patch`)
        let mut tagged = r;
        for row in tagged.rows.iter_mut() {
            row.origin = Some(if row.origin.as_deref() == Some("player") { "patch" } else { "player" }.into());
        }
        let _ = g.set_rules(tagged);
        lp.lap("reads: re-tagged set_rules");
        let _ = g.cage_forecast();
        lp.lap("reads: cage_forecast again");
    };
    let first = g.forecast();
    lp.lap("reads: first forecast");
    reads(g, lp);
    let again = g.forecast();
    lp.lap("reads: forecast again");
    t.check("forecast → every sheet's reads → forecast: identical", again == first, || format!("seed {seed}: {:?} → {:?}", first.depths.iter().map(|d| d.reach).collect::<Vec<_>>(), again.depths.iter().map(|d| d.reach).collect::<Vec<_>>()));
    // QA on a946e04 (qaS: `return 90% death 10%` → `88% · 12%` across a reload): save → load →
    // forecast is the same forecast.
    let reload = |g: &Game| Game::load(&g.save()).map(|h| h.forecast());
    t.check("save → load → forecast: identical", reload(g).as_ref() == Ok(&again), || format!("seed {seed}"));
    if !seed.is_multiple_of(3) {
        return;
    }
    let refined = g.forecast_refine();
    lp.lap("refine: forecast_refine");
    reads(g, lp);
    // A session's other panels (patch deltas, edited sets, the cage's) fill the memo: a full
    // one never drops the panel the camp shows (the cage picker's clear-all did — qaO).
    for i in 0..riddle_core::forecast::PANEL_CACHE_MAX {
        g.panel_cache.borrow_mut().insert(format!("qa filler {i}"), Vec::new());
    }
    let _ = g.cage_forecast();
    let after = g.forecast();
    lp.lap("refine: filler, cage, forecast");
    t.check("save → load → forecast: identical (refined)", reload(g).as_ref() == Ok(&after), || format!("seed {seed}"));
    t.check("after the refine a forecast read is the refined panel", after == refined, || format!("seed {seed}: refined {:?} read {:?}", refined.depths.iter().map(|d| d.reach).collect::<Vec<_>>(), after.depths.iter().map(|d| d.reach).collect::<Vec<_>>()));
    // (a copy of this state from a cold cache: a job of its own, the values compared there)
    let cold = g.clone();
    cold.panel_cache.borrow_mut().clear();
    cold.forecast_cache.borrow_mut().clear();
    cold.refined_panels.borrow_mut().clear();
    pool.push(60, "cold cache", seed, move |o, _| {
        o.t.check("a cold cache reads the same two passes", cold.forecast() == first && cold.forecast_refine() == refined, || format!("seed {seed}"));
    });
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

/// QA on 1a2a4a9 (qaO: `The lock took him to 4 HP; no row fired; died.` · `Spectral blades
/// cornered him to 5 HP; no row fired; died.` beside DEATHS `fire ×1`): every reel line that
/// ends in a death names the run's killer — its arc's resolution is `died to <the record's
/// cause>`, and the line's end is `died to <the cause, or its last word>`, or `died on D<n>` /
/// a bare `died` only when the setup's threat is the killer.
fn check_reel(t: &mut Tally, g: &Game, seed: u64, r: &riddle_core::ReturnReport) {
    use riddle_core::sifter::{cause_key, cause_phrase};
    for h in &r.reel {
        let Some(arc) = &h.arc else { continue };
        if !arc.resolution.starts_with("died") {
            continue;
        }
        let Some(rec) = g.deaths.get(&h.run_id).filter(|x| !x.stall) else { continue };
        let cause = &rec.death.cause;
        let phrase = cause_phrase(cause);
        let end = h.text.strip_suffix('.').and_then(|b| b.rsplit("; ").next()).unwrap_or("");
        let killer_led = cause_key(&arc.threat) == cause_key(cause);
        let ok = arc.resolution == format!("died to {phrase}")
            && match end.strip_prefix("died to ") {
                Some(w) => phrase.ends_with(w),
                None => (end == "died" || end.starts_with("died on D")) && killer_led,
            };
        t.check("a reel `died` line names the run's death cause", ok, || format!("seed {seed} run {}: `{}` · arc {} / {} · cause {cause}", h.run_id, h.text, arc.threat, arc.resolution));
        t.check("a reel line is a story line", riddle_core::sifter::story_ok(&h.text), || format!("seed {seed} run {}: `{}`", h.run_id, h.text));
        // QA on a946e04 (qaS: `A goblin took him to 3 HP; no row; died to gas.` under R2 `hp <
        // 40% → return`): a `no row` beat means no row acted at the low point — the death
        // trace's first turn at the low HP is the chores', and not the walk home a return row
        // committed to (that reads `R2 returning`).
        let turn = h.text.split("; ").nth(1).unwrap_or("");
        if riddle_core::sifter::NO_ROW.contains(&turn) {
            let at_low = rec.death.trace.turns.iter().find(|x| x.hp == arc.low_hp as i32);
            if let Some(x) = at_low {
                t.check("a reel `no row` line had no row act at its low point", x.row < 0 && !matches!(x.verb.v.as_str(), "return" | "bank"), || format!("seed {seed} run {}: `{}` · t{} R{} {} hp {}", h.run_id, h.text, x.t, x.row + 1, x.verb.short(), x.hp));
            }
        }
    }
}

/// QA on a946e04 (qaS: `A den of thieves. Nothing lost to the den.` in the run whose report
/// said `stolen red potion?`): watched sends from a copy of the night's camp — a den's
/// `Nothing lost` note is said only while every theft of the run so far was got back.
fn check_den_leg(t: &mut Tally, g: &Game, seed: u64) {
    let mut h = g.clone();
    for _ in 0..6 {
        h.send();
        let (mut out, mut back) = (0i32, 0i32);
        for _ in 0..4000 {
            let r = h.step(50);
            for e in &r.events {
                match e {
                    Ev::Steal { .. } => out += 1,
                    Ev::Note { text, .. } if text.starts_with("Got the ") && text.ends_with(" back.") => back += 1,
                    Ev::Note { t: at, text } if text == "Nothing lost to the den." => {
                        t.check("`Nothing lost to the den` never beside a theft still out", out == back, || format!("seed {seed} t{at}: {out} thefts, {back} got back"));
                    }
                    _ => {}
                }
            }
            if r.run_over {
                break;
            }
        }
        if h.pending_exit.is_some() {
            let _ = h.keep(vec![]);
        }
    }
}

/// QA on a946e04 (qaT: `waystone D5 ×16 · −$800` over one absence, the repeat starved; with a
/// short purse the sends went from D1 unsaid): a copy of the lineage with a waystone lit and
/// chosen, four hours offline — a night's tolls are one pass per chosen start (a night is
/// `NIGHT_RUNS` runs); from an empty purse no toll is paid and the report names the sends that
/// went from D1.
fn check_waystone_leg(t: &mut Tally, g: &Game, seed: u64) {
    let d = riddle_core::engine::WAYSTONES[0];
    for rich in [true, false] {
        let mut h = g.clone();
        if !h.lineage.waystones.contains(&d) {
            h.lineage.waystones.push(d);
            h.lineage.waystones.sort_unstable();
        }
        if h.set_start(d).is_err() {
            return;
        }
        let top = if rich { 2000 - h.lineage.gold } else { -h.lineage.gold };
        h.lineage.gold_move(top, "qa purse");
        let before = h.lineage.night_runs;
        let r = h.run_offline(4 * 3600);
        let tolls: u32 = r.spent.iter().filter(|x| x.kind.starts_with("waystone")).map(|x| x.n).sum();
        let nights = (before + r.runs).div_ceil(riddle_core::engine::NIGHT_RUNS).max(1);
        if rich {
            t.check("a night's waystone tolls ≤ 1 pass per chosen start", tolls <= nights && r.start_short.is_none(), || format!("seed {seed}: {tolls} tolls over {} runs ({nights} nights) · short {:?}", r.runs, r.start_short));
        } else {
            let ok = tolls == 0 || r.start_short.is_some();
            let shorted = r.start_short.as_ref().map(|s| s.runs).unwrap_or(0);
            t.check("a short purse's sends from D1 are named (report `start_short`)", ok && (tolls > 0 || shorted > 0 || r.runs == 0), || format!("seed {seed}: {tolls} tolls · short {:?} · {} runs", r.start_short, r.runs));
        }
    }
}

/// QA on a946e04 (qaT: SALVAGED `heal ×2 · $4` while heal was packed and repeated): watched
/// sends from a copy of the night's camp with heal packed and repeated — a found potion of a
/// packed kind is never on a bank/return exit's keep sheet (to be salvaged) while the shelf
/// has room and holds fewer of that kind than the repeat packs.
fn check_found_supply_leg(t: &mut Tally, g: &Game, seed: u64) {
    let mut h = g.clone();
    let heal = "heal";
    if let Some(f) = riddle_core::item::ident_fact(&h.lineage.flavours, heal) {
        h.lineage.facts.insert(f);
    }
    let mut set = h.lineage.rules().clone();
    if !h.lineage.row_kinds().contains(heal) {
        h.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
        set.rows.insert(0, Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", heal)));
        if h.set_rules_raw(set).is_err() {
            return;
        }
    }
    h.lineage.gold_move(400 - h.lineage.gold.min(400), "qa purse");
    h.set_restock(true);
    if h.buy_supply(heal).is_err() {
        return;
    }
    for _ in 0..6 {
        h.send();
        for _ in 0..4000 {
            let r = h.step(50);
            if r.run_over {
                break;
            }
        }
        if let Some(p) = h.pending_exit.clone() {
            if p.tier != ExitTier::Death {
                let l = &h.lineage;
                let cap = l.supply_cap();
                let cat = h.supply_catalogue();
                for it in p.items.iter().filter(|i| i.is_consumable() && i.kind == heal && !p.brought.contains(&i.id) && i.is_known(&l.facts, &l.flavours)) {
                    let on_shelf = l.supplies.iter().filter(|s| s.kind == heal).count();
                    let plan = l.last_supplies.iter().filter(|k| *k == heal).count();
                    let room = l.supplies.len() < cap && on_shelf < plan && cat.iter().any(|e| e.kind == heal);
                    t.check("a found packed-kind potion is shelved, not salvaged, while the shelf has room", !room, || format!("seed {seed}: {} on the keep sheet · shelf {}/{cap} · {heal} {on_shelf} of {plan}", it.kind, l.supplies.len()));
                }
            }
            let _ = h.keep(vec![]);
        }
        if h.lineage.gold < 100 {
            h.lineage.gold_move(100, "qa purse");
        }
    }
}

/// QA on a946e04 (qaS: a row the editor marks dead — `↑ R3` — moved the forecast D5 72 → 74 %):
/// the set with a row its engagement row shadows appended plays the set without it, sim for
/// sim (the camp's seeds, run on their own, not through the memo), and reads the same forecast.
fn check_dead_row(t: &mut Tally, g: &Game, seed: u64) {
    let mut h = g.clone();
    h.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    let base = h.lineage.rules().clone();
    let Some(eng) = base.rows.iter().find(|r| matches!(r.verb.v.as_str(), "attack" | "shoot") && !r.is_card()) else { return };
    let mut row = eng.clone();
    row.conds.push(Cond::n("hp<", 50));
    let mut dead = base.clone();
    dead.rows.push(row);
    if dead.own_rows() > h.lineage.max_rows() || h.lineage.shadowed_by(&dead).last().copied().flatten().is_none() {
        return;
    }
    let sims = riddle_core::forecast::FORECAST_SIMS;
    let tag = riddle_core::forecast::forecast_tag(&h, h.lineage.best_depth + 1);
    let a = riddle_core::forecast::simulate_budget(&h, &base, sims, tag, u32::MAX, riddle_core::forecast::CAMP_TICK_BUDGET);
    let b = riddle_core::forecast::simulate_budget(&h, &dead, sims, tag, u32::MAX, riddle_core::forecast::CAMP_TICK_BUDGET);
    t.check("a shadowed row changes no sim", a == b, || format!("seed {seed}: reach Σ {} vs {}", a.iter().map(|r| r.max_depth).sum::<u32>(), b.iter().map(|r| r.max_depth).sum::<u32>()));
    let f0 = h.forecast();
    if h.set_rules_raw(dead).is_err() {
        return;
    }
    let f1 = h.forecast();
    t.check("a shadowed row reads the same forecast", f0.depths == f1.depths && f0.ends == f1.ends && f0.causes == f1.causes, || format!("seed {seed}: {:?} → {:?}", f0.depths.iter().map(|d| d.reach).collect::<Vec<_>>(), f1.depths.iter().map(|d| d.reach).collect::<Vec<_>>()));
}

/// The label check with salvage in it: a copy of the lineage with `hp < 50% → return` on top
/// (a return cuts 40 % of the pack to salvage before the sheet) for two hours offline.
fn check_salvage_leg(t: &mut Tally, g: &Game, seed: u64) {
    let mut h = g.clone();
    h.lineage.unlocks.extend(["row5", "row6", "row7", "row8"].map(String::from));
    let mut set = g.lineage.rules().clone();
    set.rows.insert(0, Row::new(vec![Cond::n("hp<", 50)], Verb::new("return")));
    if h.set_rules_raw(set).is_err() {
        return;
    }
    let r = h.run_offline(2 * 3600);
    check_report_names(t, &h, seed, &r, "return leg");
    // (and one watched exit's sheet and cut)
    h.send();
    for _ in 0..4000 {
        let s = h.step(50);
        if s.run_over {
            if let Some(pe) = &s.exit_pending {
                check_names(t, &h, seed, pe.items.iter().map(|i| i.label.as_str()), "return leg sheet");
            }
            if let Some(x) = h.last_exit.clone() {
                check_names(t, &h, seed, x.salvaged.iter().map(|s| s.kind.as_str()), "return leg cut");
            }
            break;
        }
    }
}

/// QA on 1a2a4a9 (qaP): a copy of the lineage that knows every tag and plays a set that
/// paces (`foe: ranged → retreat` over the engagement row) for four hours; each stall record's
/// top patch (a cut, a swap or an insert) is applied, and the two sets' forecast stall shares
/// go into the pooled check (`StallSum`). The record's verdict and its patches are checked
/// as the night's stalls are.
fn check_stall_leg(t: &mut Tally, sum: &mut StallSum, g: &Game, seed: u64) {
    let mut h = g.clone();
    riddle_core::probes::learn_everything(&mut h);
    h.lineage.unlocks.insert("row5".into());
    let set = riddle_core::RuleSet {
        rows: vec![
            Row::new(vec![Cond::n("hp<", 40)], Verb::arg("drink", "heal")),
            Row::new(vec![Cond::t("foe_tag", "ranged")], Verb::new("retreat")),
            Row::new(vec![Cond::n("hp<", 20)], Verb::new("return")),
            Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        ],
        name: None,
    };
    if h.set_rules_raw(set).is_err() {
        return;
    }
    h.run_offline(4 * 3600);
    let ids: Vec<u32> = h.deaths.iter().filter(|(_, rec)| rec.stall).map(|(id, _)| *id).collect();
    for id in ids {
        let Some(d) = h.death(id) else { continue };
        check_death(t, &h, seed, &d);
        let Some(p) = d.patches.iter().find(|p| p.insert_at >= 0 && !p.below_bar) else { continue };
        let before = h.forecast();
        let mut k = h.clone();
        if k.set_rules_raw(riddle_core::trace::patched_rules(h.lineage.rules(), p, h.lineage.max_rows())).is_err() {
            continue;
        }
        let after = k.forecast();
        if let (Some(a), Some(b)) = (before.ends, after.ends) {
            sum.0 += a.stall;
            sum.1 += b.stall;
            sum.2 += 1;
        }
    }
}

/// What a job adds to the totals.
#[derive(Default)]
struct Out {
    t: Tally,
    bank: BankTally,
    lp: Laps,
}

impl BankTally {
    fn merge(&mut self, b: &BankTally) {
        for (a, x) in self.0.iter_mut().zip(b.0) {
            a.0 += x.0;
            a.1 += x.1;
            a.2 += x.2;
            a.3 += x.3;
        }
        self.1 .0 += b.1 .0;
        self.1 .1 += b.1 .1;
        self.1 .2 += b.1 .2;
    }
}

type JobFn = Box<dyn FnOnce(&mut Out, &Pool) + Send>;

/// A job: a seed's prelude (the protocol played to the night, then the night's own checks), or
/// one of the legs it hands off — each on its own copy of the game, so a leg's copy is the
/// state the sequential order gave it and the checks are the same checks.
struct Job {
    prio: u32,
    seq: u64,
    name: &'static str,
    seed: u64,
    f: JobFn,
}

impl PartialEq for Job {
    fn eq(&self, o: &Self) -> bool {
        (self.prio, self.seq) == (o.prio, o.seq)
    }
}
impl Eq for Job {}
impl PartialOrd for Job {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}
impl Ord for Job {
    /// The highest priority first, then the oldest.
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        self.prio.cmp(&o.prio).then(o.seq.cmp(&self.seq))
    }
}

/// One queue for every thread, longest work first (`prio`); a job may push more. The workers
/// stop when the queue is empty and no job is running (a running one may still push).
#[derive(Default)]
struct Pool {
    /// (the queue, jobs pushed, jobs queued or running)
    q: Mutex<(std::collections::BinaryHeap<Job>, u64, usize)>,
    cv: std::sync::Condvar,
}

impl Pool {
    fn push(&self, prio: u32, name: &'static str, seed: u64, f: impl FnOnce(&mut Out, &Pool) + Send + 'static) {
        let mut q = self.q.lock().unwrap();
        q.1 += 1;
        q.2 += 1;
        let seq = q.1;
        q.0.push(Job { prio, seq, name, seed, f: Box::new(f) });
        self.cv.notify_one();
    }
    fn work(&self, total: &Mutex<Out>, phases: bool, t0: std::time::Instant) {
        loop {
            let job = {
                let mut q = self.q.lock().unwrap();
                loop {
                    if let Some(j) = q.0.pop() {
                        break j;
                    }
                    if q.2 == 0 {
                        return;
                    }
                    q = self.cv.wait(q).unwrap();
                }
            };
            let ts = std::time::Instant::now();
            let mut o = Out::default();
            o.lp.lap("start");
            (job.f)(&mut o, self);
            o.lp.lap(job.name);
            if phases {
                eprintln!("job {:<18} seed {:>2} {:>5.1}s (ends at {:.1}s)", job.name, job.seed, ts.elapsed().as_secs_f64(), t0.elapsed().as_secs_f64());
            }
            {
                let mut all = total.lock().unwrap();
                all.t.merge(o.t);
                all.bank.merge(&o.bank);
                all.lp.merge(&o.lp);
            }
            let mut q = self.q.lock().unwrap();
            q.2 -= 1;
            if q.2 == 0 {
                self.cv.notify_all();
            }
        }
    }
}

/// A seed: the protocol through the engine (send → exit → `death()` → patch → the camp's
/// reads → buy → the night), the night's legs handed to the pool on copies of the game, then
/// the night's deaths, the shop and the save on the game itself.
fn play(o: &mut Out, pool: &Pool, seed: u64) {
    let Out { t, lp, .. } = o;
    let mut g = Game::new(seed);
    check_gold(t, &g, seed, "new");
    check_needs(t, &g, seed);
    check_forecast(t, &g, seed);
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
        // Cut 20 §4: the stake names what a death keeps beside the exit row's keep.
        let st = &r.snapshot.stake;
        t.check("the stake's death keep == the death tier's share of carried", st.death_keep == st.loot.max(0) * ExitTier::Death.pct() / 100, || format!("seed {seed}: death keep {} of carried {}", st.death_keep, st.loot));
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
                check_labels(t, &g, seed, &p.items, "exit sheet");
                check_names(t, &g, seed, pe.items.iter().map(|i| i.label.as_str()), "exit sheet");
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
    check_gold(t, &g, seed, "after the run");
    // Cut 20 §4 (AC: a silent repeat charge on death): a re-pack charged at a death's exit is
    // a `repeat` line in the gold ledger and on the death record's ledger line.
    if let (Some(id), Some(line)) = (died, g.last_exit.clone()) {
        if line.spent > 0 {
            let repeat: i32 = g.lineage.gold_ledger.iter().filter(|x| x.why.starts_with("repeat") && x.delta < 0).map(|x| -x.delta).sum();
            let rec = g.death(id).and_then(|d| d.line).map(|l| l.spent);
            t.check("a death's repeat charge is a `repeat` ledger line", repeat >= line.spent && rec == Some(line.spent), || format!("seed {seed} run {id}: spent {} · repeat lines ${repeat} · record {rec:?}", line.spent));
        }
    }
    lp.lap("first run (new, send, exit)");
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
        // trace's last turn), and its first patch cuts that row. Cut 21 §3: or the set's own
        // row whose unknown gamble dealt the death (`DeathRec.gamble_row`) — its cut is shown
        // first, or second behind a patch surviving `SURVIVE_BAND` more.
        if d.verdict == "row" || d.cause_row.is_some() {
            let last = d.trace.turns.last().map(|x| x.row);
            let gamble = g.deaths.get(&id).and_then(|r| r.gamble_row).map(|r| r as i32);
            let own = d.cause_row.is_some_and(|r| d.rules.as_ref().is_some_and(|rs| rs.rows.get(r as usize).is_some_and(|x| !x.is_card())));
            let cuts = |p: &riddle_core::wire::Patch| (p.remove || p.replace) && Some(p.insert_at) == d.cause_row.map(|r| r as i32);
            let ok = d.verdict == "row"
                && own
                && match gamble.filter(|r| Some(*r) == d.cause_row.map(|c| c as i32)) {
                    Some(_) => d.patches.iter().take(2).any(cuts),
                    None => d.cause_row.map(|r| r as i32) == last && d.patches.first().is_some_and(cuts),
                };
            t.check("a `row` verdict names a row that fired on the death tick", ok, || format!("seed {seed} run {id}: {} R{:?} last {:?}", d.verdict, d.cause_row, last));
        }
        check_death(t, &g, seed, &d);
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
            t.check("the top patch applies through set_rules", g.set_rules(patched.clone()).is_ok(), || format!("seed {seed}: {}", p.row.describe()));
            // QA on e75ec29 (qaQ: `hp < 20% → rest · survives 100%` applied, then `R1 rest · not
            // safe` in every later trace): the top patch, applied, acts in the next six sends
            // from camp at least once (a copy of the game; the protocol goes on unpatched by it).
            if let Some(pos) = if p.replace { Some(p.insert_at as usize) } else { patched.rows.iter().position(|r| *r == p.row) } {
                let mut h = g.clone();
                let mut fired = 0u32;
                for _ in 0..6 {
                    h.send();
                    let mut runs_fired = false;
                    for _ in 0..4000 {
                        let r = h.step(50);
                        runs_fired |= r.events.iter().any(|e| matches!(e, Ev::Rule { row, .. } if *row == pos as i32));
                        if r.run_over {
                            break;
                        }
                    }
                    if h.pending_exit.is_some() {
                        let _ = h.keep(vec![]);
                    }
                    fired += runs_fired as u32;
                }
                t.check("the top patch, applied, acts in one of the next 6 sends", fired > 0, || format!("seed {seed} run {id}: {} at R{} · {} · fired in 0 of 6", p.row.describe(), pos + 1, d.verdict));
            }
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
    lp.lap("first death (death, deltas, patch forecasts)");
    check_forecast_reads(t, lp, pool, &mut g, seed, died);
    lp.lap("forecast reads");
    // Buy the first affordable unlock.
    if let Some(u) = g.unlocks().into_iter().find(|u| u.available && u.cost <= g.lineage.marks) {
        let marks = g.lineage.marks;
        t.check("an available unlock buys", g.buy(&u.id).is_ok() && g.lineage.marks == marks - u.cost, || format!("seed {seed}: {}", u.id));
    }
    check_needs(t, &g, seed);
    // QA on a946e04 (qaT: three cards bought, three rows inserted): an unmeasured catalogue
    // never says `auto_insert`; a measured one only where the card's reach holds and its stall
    // rise is ≤ `CARD_STALL_RISE`.
    t.check("an unmeasured card never auto-inserts", g.unlocks().iter().all(|u| !u.auto_insert), || format!("seed {seed}"));
    for u in g.unlock_deltas().iter().filter(|u| u.auto_insert) {
        t.check("an auto-inserted card is measured, reach not down, stall ≤ 5 pts", u.delta.is_some_and(|d| d >= -1e-9) && u.stall.is_some_and(|s| s <= riddle_core::meta::CARD_STALL_RISE + 1e-9), || format!("seed {seed}: {} Δ{:?} stall {:?}", u.id, u.delta, u.stall));
    }
    check_gold_buy(t, &g, seed);
    lp.lap("unlock + gold buy");
    // QA on e75ec29 (qaQ: `⊘ drink heal · unknown` in the picker, R1 `drink heal`): the
    // vocabulary offers every drink / read a saved set holds.
    {
        let v = g.vocabulary();
        let held: Vec<riddle_core::rules::Verb> = g.lineage.sets.iter().flat_map(|s| s.rows.iter()).filter(|r| matches!(r.verb.v.as_str(), "drink" | "read")).map(|r| r.verb.clone()).collect();
        let missing: Vec<String> = held.iter().filter(|x| !v.verbs.contains(x)).map(|x| x.short()).collect();
        t.check("the vocabulary offers every drink / read a set holds", missing.is_empty(), || format!("seed {seed}: {missing:?}"));
    }
    // The night.
    let before = g.lineage.gold;
    let bounty_before = g.lineage.bounty;
    let chronicle_before = g.lineage.chronicle.len();
    let deeds_before: Vec<String> = g.lineage.heir_deeds.clone();
    let r = g.run_offline(8 * 3600);
    // QA on e75ec29 (qaQ: `bounty D10 · missed`, no bounty on the camp before the absence): the
    // report's bounty is the one the camp showed when the absence began.
    t.check("the report's bounty is the camp's before the absence", r.bounty.as_ref().is_none_or(|b| Some(b.depth) == bounty_before), || format!("seed {seed}: report {:?} · camp {bounty_before:?}", r.bounty));
    // QA on e75ec29 (qaQ: BONES 12 finds, the chronicle's heirs 8): every pile the report says
    // was found is named by a finder — the absence's chronicle lines and the live heir's deeds.
    if g.lineage.chronicle.len() < riddle_core::engine::CHRONICLE_CAP {
        let bones = |line: &str| -> usize { line.split(" · ").filter(|p| p.starts_with("found ♟") && p.ends_with("'s bones")).map(|p| p.matches('♟').count()).sum() };
        let lines: usize = g.lineage.chronicle[chronicle_before.min(g.lineage.chronicle.len())..].iter().map(|l| bones(l)).sum();
        let live: usize = g.lineage.heir_deeds.iter().filter(|d| d.starts_with("found ♟") && !deeds_before.contains(d)).count();
        t.check("the report's bones == the finds the chronicle names", lines + live == r.bones_found.len(), || format!("seed {seed}: report {} · chronicle {lines} · live heir {live}", r.bones_found.len()));
    }
    // QA on e75ec29 (qaR: packed heals stolen on D1, no report line): the report's stolen
    // items are the night's thefts less what runs got back.
    let stolen: u32 = r.stolen.iter().map(|x| x.n).sum();
    t.check("report stolen == thefts − got back", stolen == g.batch.thefts - g.batch.recovered, || format!("seed {seed}: report {stolen} · thefts {} · got back {}", g.batch.thefts, g.batch.recovered));
    check_names(t, &g, seed, r.stolen.iter().map(|x| x.label.as_str()), "report stolen");
    // QA on e75ec29 (qaR: `+$40 heir purse` after one death, none after three): each death
    // either topped the purse up or found it full.
    if let Some(gs) = &r.gold {
        let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
        t.check("the heir purse: a top-up per death below the line", gs.wake_cap == riddle_core::engine::WAKE_PAY && gs.wake_n <= deaths && (gs.wake_n == 0) == (gs.wake == 0), || format!("seed {seed}: wake ${} over {} of {deaths} deaths, cap ${}", gs.wake, gs.wake_n, gs.wake_cap));
    }
    // QA on e75ec29 (qaQ: `rested 333m` beside 16 runs × `rest 20m`): an absence of deaths
    // rested one wake per run.
    if !r.sampled && r.banked == 0 && r.returned == 0 {
        let want = r.runs as u64 * g.rest_after(0, ExitTier::Death) as u64 / riddle_core::offline::TICKS_PER_SECOND;
        t.check("an absence of deaths rested one wake per run", r.rested_s == want, || format!("seed {seed}: rested {}s · {} runs · want {want}s", r.rested_s, r.runs));
    }
    lp.lap("night (8 h offline)");
    let deaths: u32 = r.deaths.iter().map(|d| d.n).sum();
    t.check("report runs == deaths + banked + returned", r.runs == deaths + r.banked + r.returned, || format!("seed {seed}: {} runs · {deaths} deaths · {} banked · {} returned", r.runs, r.banked, r.returned));
    let rows: i32 = r.salvaged.iter().map(|x| x.gold).sum();
    let head = r.gold.as_ref().map(|x| x.salvage).unwrap_or(0);
    t.check("report SALVAGED rows sum to the header's salvage", rows == head, || format!("seed {seed}: rows ${rows} vs +${head}"));
    let b = &g.batch;
    let spent: i32 = b.spent.values().map(|(_, c)| *c).sum();
    t.check("night gold: earned + salvage + wake pay − spent == delta", b.gold_earned + b.salvage_gold + b.wake_pay - spent == g.lineage.gold - before, || format!("seed {seed}: {} + {} + {} − {spent} vs {}", b.gold_earned, b.salvage_gold, b.wake_pay, g.lineage.gold - before));
    t.check("report spent == the batch's", r.spent.iter().map(|s| s.gold).sum::<i32>() == spent, || format!("seed {seed}"));
    check_labels(t, &g, seed, &g.batch.found.clone(), "report found");
    for f in &r.found {
        t.check("report `found` never labels `?` when known", !f.known || !f.label.contains('?'), || format!("seed {seed}: {f:?}"));
    }
    check_gold(t, &g, seed, "after the night");
    // The night's legs, longest first, each on a copy of the game as it is now (none of them
    // changes it; the sequential order ran them on it here).
    let at = |pool: &Pool, prio: u32, name: &'static str, g: &Game, f: fn(&mut Out, &Game, u64)| {
        let h = g.clone();
        pool.push(prio, name, seed, move |o, _| f(o, &h, seed));
    };
    for d in bank_depths(&g) {
        let h = g.clone();
        pool.push(80, "forecast vs sends", seed, move |o, _| check_forecast_bank_at(&mut o.t, &mut o.bank, &h, seed, d));
    }
    at(pool, 75, "check_short", &g, |o, g, seed| check_short(&mut o.t, g, seed));
    at(pool, 70, "stall leg", &g, |o, g, seed| check_stall_leg(&mut o.t, &mut o.bank.1, g, seed));
    at(pool, 65, "night forecast", &g, |o, g, seed| {
        check_needs(&mut o.t, g, seed);
        check_forecast(&mut o.t, g, seed);
    });
    at(pool, 55, "salvage leg", &g, |o, g, seed| check_salvage_leg(&mut o.t, g, seed));
    at(pool, 50, "shadow leg", &g, |o, g, seed| check_shadowed(&mut o.t, g, seed));
    at(pool, 45, "den leg", &g, |o, g, seed| check_den_leg(&mut o.t, g, seed));
    at(pool, 45, "waystone leg", &g, |o, g, seed| check_waystone_leg(&mut o.t, g, seed));
    at(pool, 45, "found supply leg", &g, |o, g, seed| check_found_supply_leg(&mut o.t, g, seed));
    if seed.is_multiple_of(3) {
        at(pool, 45, "dead row leg", &g, |o, g, seed| check_dead_row(&mut o.t, g, seed));
    }
    check_report_names(t, &g, seed, &r, "report");
    for x in &r.exits {
        if let Some(tr) = &x.trace {
            check_foe_reasons(t, seed, tr, &format!("run {} exit trace", x.run_id));
        }
    }
    check_reel(t, &g, seed, &r);
    lp.lap("report checks");
    if let Some(d) = &r.worst_death {
        t.check("worst death carries a verdict and a trace", (d.verdict == "gap" || d.verdict == "dice" || d.verdict == "stall" || d.verdict == "row") && !d.trace.turns.is_empty(), || format!("seed {seed}: {}", d.verdict));
        check_death(t, &g, seed, d);
    }
    // One more of the night's deaths (the last one that is not the worst), in full.
    if let Some(id) = g.deaths.iter().rev().find(|(id, rec)| !rec.stall && Some(**id) != r.worst_death_id).map(|(id, _)| *id) {
        if let Some(d) = g.death(id) {
            check_death(t, &g, seed, &d);
        }
    }
    lp.lap("night deaths");
    // Every stall record: its verdict, a firing patch, the reel's cause == the trace's.
    let stall_ids: Vec<u32> = g.deaths.iter().filter(|(_, rec)| rec.stall).map(|(id, _)| *id).collect();
    for id in stall_ids {
        let Some(d) = g.death(id) else { continue };
        let rec = g.deaths.get(&id).cloned().unwrap();
        let fires = d.patches.iter().any(|p| riddle_core::trace::patch_fired_rate(&g, &rec, p) >= 0.5);
        t.check("a stall's verdict names a firing patch", d.verdict == "stall" && fires, || format!("seed {seed} run {id}: {} {:?}", d.verdict, d.patches.iter().map(|p| p.row.describe()).collect::<Vec<_>>()));
        check_death(t, &g, seed, &d);
        let cause = d.cause.strip_prefix("stalled · ").unwrap_or(&d.cause);
        let want = format!("stalled, {}", riddle_core::sifter::stall_short(cause));
        let lines: Vec<&riddle_core::Highlight> = g.batch.highlights.iter().filter(|h| h.run_id == id && h.arc.is_some()).collect();
        if !lines.is_empty() {
            t.check("a stall's reel cause == its trace cause", lines.iter().any(|h| h.arc.as_ref().unwrap().resolution == want), || format!("seed {seed} run {id}: want `{want}`, lines {:?}", lines.iter().map(|h| h.text.clone()).collect::<Vec<_>>()));
        }
    }
    lp.lap("night stalls");
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
    check_gold(t, &g, seed, "after the shop");
    // A save round-trips the lineage.
    let h = Game::load(&g.save()).expect("load");
    t.check("save/load round-trips the lineage", h.lineage() == g.lineage(), || format!("seed {seed}"));
    lp.lap("shop + save");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 30);
    riddle_core::forecast::set_parallel_sims(false);
    let t0 = std::time::Instant::now();
    // `METRICS_PHASES=1`: every job's wall and each leg's thread-seconds to stderr.
    let phases = std::env::var("METRICS_PHASES").is_ok();
    // The seeds whose camp reads run the refine leg (every third) are the long preludes: first.
    let pool = Arc::new(Pool::default());
    for seed in 1..=seeds {
        pool.push(if seed.is_multiple_of(3) { 40 } else { 30 }, "prelude", seed, move |o, pool| play(o, pool, seed));
    }
    let total = Arc::new(Mutex::new(Out::default()));
    // `--threads N` leaves cores to what runs beside it (gates.mjs: the table and the dayplayer's
    // chains).
    let threads = (get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(4)) as usize).max(1);
    let hs: Vec<_> = (0..threads)
        .map(|_| {
            let (pool, total) = (Arc::clone(&pool), Arc::clone(&total));
            std::thread::spawn(move || pool.work(&total, phases, t0))
        })
        .collect();
    for h in hs {
        h.join().unwrap();
    }
    let Out { mut t, bank: banks, lp: laps } = std::mem::take(&mut *total.lock().unwrap());
    if phases {
        let total: f64 = laps.0.values().sum();
        let mut v: Vec<_> = laps.0.iter().filter(|x| *x.1 > 0.05).collect();
        v.sort_by(|a, b| b.1.total_cmp(a.1));
        for (k, x) in v {
            eprintln!("leg {k:<44} {x:>7.1} thread-s  {:>4.1} %", 100.0 * x / total.max(1e-9));
        }
        eprintln!("legs total {total:.1} thread-s");
    }
    // The forecast's bank share, summed over the seeds, within the pooled 95 % band of the
    // real sends' (the two samples' binomial ±).
    for (name, (fb, fn_, rb, rn)) in ["forecast bank ≈ real sends (good set, Σ seeds)", "forecast bank ≈ real sends (depth ≥ d → bank, Σ seeds)"].into_iter().zip(banks.0) {
        let (pf, pr) = (fb / fn_.max(1.0), rb / rn.max(1.0));
        let p = (fb + rb) / (fn_ + rn).max(1.0);
        let band = 1.96 * (p * (1.0 - p) * (1.0 / fn_.max(1.0) + 1.0 / rn.max(1.0))).sqrt();
        t.check(name, (pf - pr).abs() <= band + 1e-9, || format!("forecast {pf:.3} over {fn_} sims vs sends {pr:.3} over {rn}, band ±{band:.3}"));
        println!("{name}: forecast {pf:.3} ({fn_} sims) · sends {pr:.3} ({rn}) · ±{band:.3}");
    }
    // QA on 1a2a4a9 (qaP: the stall patch `survives 100%`, the next run stalled in the same
    // loop, and the camp showed no stall line): over the stall leg's records, the top patch's
    // set forecasts fewer stalls than the set that stalled.
    let StallSum(base, patched, n) = banks.1;
    t.check("a stall's top patch forecasts fewer stalls (Σ records)", n > 0 && patched < base, || format!("{n} records: stall share Σ {base:.2} → {patched:.2}"));
    println!("stall leg: {n} records · forecast stall share Σ {base:.2} → {patched:.2} with the top patch");
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
