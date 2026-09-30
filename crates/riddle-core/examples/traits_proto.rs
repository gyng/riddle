//! Cut 30 prototype (docs/TRAITS.md §6): generated heir traits as an example-local simulation hook,
//! measured on cohort sets. Nothing in `src/` changes: each sim is `forecast::sim_game` ticked here,
//! with the trait applied to the hero between ticks (a context trigger × a gift × a cost), and the
//! legacy temperaments held off in every arm (`Run.trait_floor` pinned to 1 — the once-a-floor
//! override never fires). Per cohort set (its lineage after `oath_lib::OATH_HOURS` of its own sends):
//!   · the set's lever: its best single-row edit's bank move and the largest row drop (neutral hero);
//!   · B0, the bank-optimal set under the neutral hero (`oath_lib`'s plateau search, 2 steps);
//! per (set, trait):
//!   · the trait's paired bank / death / gold move on the set as written (`MAIN` sims);
//!   · a plateau search from B0 under the trait (2 steps) → B_T; rows B_T and B0 differ by; and a
//!     confirmation on fresh seeds (`CONF` sims): the gain of B_T over B0 under the trait, and the
//!     same edit's gain under the neutral hero (the interaction is what makes it a build).
//!   RIDDLE_THREADS=8 nice cargo run -q --profile fast -p riddle-core --example traits_proto -- \
//!     [--sets AU,AV,AW,AX] [--traits all|controls|<filter>] [--seed 1] [--out scratchpad/traits.json]
#[path = "lever_lib/mod.rs"]
mod lever;
#[path = "oath_lib/mod.rs"]
mod oath_lib;
use riddle_core::engine::ExitTier;
use riddle_core::{Cond, Game, Row, RuleSet, Verb};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

const MAIN: u32 = 128;
const CONF: u32 = 160;
const SCREEN: u32 = 16;
const FULL: u32 = 64;
const TOP: usize = 3;
const STEPS: usize = 2;
const GAIN: f64 = 0.02;
const TAG_A: u64 = 0x7A17_0030;
const TAG_B: u64 = 0x7A17_0031;

// ---------------------------------------------------------------------------------------------
// The generator's parts: 6 triggers (5 + the `always` control) × 4 gifts × 4 costs (3 + none).

#[derive(Clone, Copy, Debug, PartialEq)]
enum When {
    Hurt,
    Crowded,
    Boss,
    Quiet,
    Deep,
    Always,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Gift {
    Fury,
    Guard,
    Quick,
    Mend,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Cost {
    None,
    Frail,
    Slow,
    Dim,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tr {
    when: When,
    gift: Gift,
    cost: Cost,
}

impl Tr {
    fn when_word(self) -> &'static str {
        match self.when {
            When::Hurt => "hurt",
            When::Crowded => "crowded",
            When::Boss => "boss",
            When::Quiet => "quiet",
            When::Deep => "deep",
            When::Always => "always",
        }
    }
    fn gift_word(self) -> &'static str {
        match self.gift {
            Gift::Fury => "fury",
            Gift::Guard => "guard",
            Gift::Quick => "quick",
            Gift::Mend => "mend",
        }
    }
    fn cost_word(self) -> &'static str {
        match self.cost {
            Cost::None => "",
            Cost::Frail => "frail",
            Cost::Slow => "slow",
            Cost::Dim => "dim",
        }
    }
    /// The lexicon name (docs/TRAITS.md §2): a gift×trigger word, then the cost's.
    fn name(self) -> String {
        let head = match (self.when, self.gift) {
            (When::Hurt, Gift::Fury) => "wrathful",
            (When::Hurt, Gift::Guard) => "stubborn",
            (When::Hurt, Gift::Quick) => "skittish",
            (When::Hurt, Gift::Mend) => "tough-blooded",
            (When::Crowded, Gift::Fury) => "brawler",
            (When::Crowded, Gift::Guard) => "back-to-wall",
            (When::Crowded, Gift::Quick) => "slippery",
            (When::Crowded, Gift::Mend) => "thick-skinned",
            (When::Boss, Gift::Fury) => "grudge-keeper",
            (When::Boss, Gift::Guard) => "unbowed",
            (When::Boss, Gift::Quick) => "duellist",
            (When::Boss, Gift::Mend) => "defiant",
            (When::Quiet, Gift::Fury) => "(quiet fury)",
            (When::Quiet, Gift::Guard) => "(quiet guard)",
            (When::Quiet, Gift::Quick) => "restless",
            (When::Quiet, Gift::Mend) => "light sleeper",
            (When::Deep, Gift::Fury) => "fen-born",
            (When::Deep, Gift::Guard) => "mud-hide",
            (When::Deep, Gift::Quick) => "night-eyed",
            (When::Deep, Gift::Mend) => "marsh-blood",
            (When::Always, Gift::Fury) => "strong",
            (When::Always, Gift::Guard) => "tough",
            (When::Always, Gift::Quick) => "fleet",
            (When::Always, Gift::Mend) => "hale",
        };
        match self.cost {
            Cost::None => head.to_string(),
            _ => format!("{head} · {}", self.cost_word()),
        }
    }
    fn code(self) -> String {
        format!("{}@{}{}", self.gift_word(), self.when_word(), if self.cost == Cost::None { String::new() } else { format!("·{}", self.cost_word()) })
    }
    /// The generator's static rejection: a gift its trigger can never use (no foe to hit or be hit by).
    fn no_op(self) -> bool {
        self.when == When::Quiet && matches!(self.gift, Gift::Fury | Gift::Guard)
    }
}

fn all_traits() -> Vec<Tr> {
    let mut v = Vec::new();
    for when in [When::Hurt, When::Crowded, When::Boss, When::Quiet, When::Deep] {
        for gift in [Gift::Fury, Gift::Guard, Gift::Quick, Gift::Mend] {
            for cost in [Cost::Frail, Cost::Slow, Cost::Dim] {
                v.push(Tr { when, gift, cost });
            }
        }
    }
    // the controls: a bare number, always on, no cost — the shape the generator must reject
    for gift in [Gift::Fury, Gift::Guard, Gift::Quick, Gift::Mend] {
        v.push(Tr { when: When::Always, gift, cost: Cost::None });
    }
    // and each trigger's gift with no cost (the rare tier's shape)
    for when in [When::Hurt, When::Crowded, When::Boss, When::Deep] {
        v.push(Tr { when, gift: Gift::Fury, cost: Cost::None });
        v.push(Tr { when, gift: Gift::Guard, cost: Cost::None });
    }
    v
}

// ---------------------------------------------------------------------------------------------
// The hook: the trait applied to a sim's hero between ticks.

#[derive(Default)]
struct Hook {
    fury_on: bool,
    depth: u32,
    vision: i32,
    fires: u32,
    t: u32,
}

fn trigger(run: &riddle_core::engine::Run, w: When) -> bool {
    let h = &run.hero;
    let foes = || run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && run.floor.map.is_visible(m.pos));
    match w {
        When::Hurt => h.hp_pct() < 50,
        When::Crowded => foes().filter(|m| m.pos.cheb(h.pos) <= 1).count() >= 2,
        When::Boss => foes().any(|m| m.is_boss()),
        When::Quiet => foes().next().is_none(),
        When::Deep => run.depth >= 9,
        When::Always => true,
    }
}

impl Hook {
    fn start(&mut self, run: &mut riddle_core::engine::Run, t: Option<Tr>) {
        let Some(t) = t else { return };
        if t.cost == Cost::Frail {
            let h = &mut run.hero;
            h.max_hp = (h.max_hp * 80 / 100).max(1);
            h.max_hp_base = h.max_hp;
            h.hp = h.hp.min(h.max_hp);
        }
    }
    fn before_tick(&mut self, run: &mut riddle_core::engine::Run, t: Option<Tr>) {
        // the legacy temperaments are off in every arm: the once-a-floor override has been spent
        run.trait_floor = run.trait_floor.max(1);
        let Some(t) = t else { return };
        self.t += 1;
        if run.depth != self.depth {
            self.depth = run.depth;
            self.vision = run.floor.vision;
        }
        let on = trigger(run, t.when);
        if on {
            self.fires += 1;
        }
        let h = &mut run.hero;
        match t.gift {
            Gift::Fury => {
                if on && !self.fury_on {
                    h.str_bonus += 2;
                    self.fury_on = true;
                } else if !on && self.fury_on {
                    h.str_bonus -= 2;
                    self.fury_on = false;
                }
            }
            Gift::Guard if on => h.ward_t = h.ward_t.max(2),
            Gift::Quick if on => h.speed_t = h.speed_t.max(2),
            Gift::Mend if on && self.t.is_multiple_of(4) && h.hp > 0 && h.hp < h.max_hp => h.hp += 1,
            _ => {}
        }
        match t.cost {
            Cost::Slow => h.energy -= 2,
            Cost::Dim => run.floor.vision = (self.vision - 2).max(2),
            _ => {}
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Sims.

#[derive(Clone, Copy, Default)]
struct Out {
    bank: bool,
    death: bool,
    gold: i32,
    fired: bool,
}

/// `g` playing `set`, its shelf packed for it (as `oath_lib::prepared`).
fn prepared(g: &Game, set: &RuleSet) -> Game {
    let mut p = g.sim_clone();
    let _ = p.set_rules_raw(set.clone());
    for r in &set.rows {
        if let Some(c) = r.card() {
            p.lineage.unlocks.insert(c.into());
        }
    }
    lever::fill_shelf(&mut p);
    p.lineage.last_supplies = p.lineage.supplies.iter().filter(|s| !s.free).map(|s| s.kind.clone()).collect();
    p
}

fn sims(g: &Game, set: &RuleSet, t: Option<Tr>, n: u32, tag: u64) -> Vec<Out> {
    let p = prepared(g, set);
    let passage = riddle_core::forecast::sim_passage(&p, set);
    (0..n)
        .map(|i| {
            let mut s = riddle_core::forecast::sim_game(&p, set, tag, i, passage);
            let mut hook = Hook::default();
            hook.start(s.run.as_mut().unwrap(), t);
            let mut k = 0;
            while s.run.as_ref().is_some_and(|r| r.over.is_none()) && k < riddle_core::forecast::SIM_MAX_TICKS {
                hook.before_tick(s.run.as_mut().unwrap(), t);
                s.tick();
                s.events.clear();
                k += 1;
            }
            let run = s.run.as_ref().unwrap();
            let tier = run.over.unwrap_or(ExitTier::Return);
            Out { bank: tier == ExitTier::Bank && !run.timed_out, death: tier == ExitTier::Death, gold: run.loot.max(0) * run.yield_pct(tier) / 100 + run.passage, fired: hook.fires > 0 }
        })
        .collect()
}

fn bank(o: &[Out]) -> f64 {
    o.iter().filter(|x| x.bank).count() as f64 / o.len().max(1) as f64
}

/// Paired mean and 95 % half-width of `f(a) − f(b)`.
fn paired(a: &[Out], b: &[Out], f: impl Fn(&Out) -> f64) -> (f64, f64) {
    let n = a.len().min(b.len());
    let d: Vec<f64> = a.iter().zip(b).map(|(x, y)| f(x) - f(y)).collect();
    let m = d.iter().sum::<f64>() / n as f64;
    let v = d.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n.max(2) - 1) as f64;
    (m, 1.96 * (v / n as f64).sqrt())
}

/// The one-row edits a search weighs: `oath_lib::edits` (drops, notches, the stock rows at the top,
/// at the safety end and in place of a row) plus two rows a hurt- or crowd-trait might want.
fn edits(g: &Game, set: &RuleSet) -> Vec<(String, RuleSet)> {
    let mut out = oath_lib::edits(g, set, None);
    let extra = [Row::new(vec![Cond::n("hp<", 40)], Verb::new("retreat")), Row::new(vec![Cond::n("hp<", 50)], Verb::arg("drink", "heal"))];
    for r in extra {
        if set.rows.contains(&r) {
            continue;
        }
        let at = riddle_core::meta::safety_end(set).min(set.rows.len());
        for pos in [0, at] {
            let mut s = set.clone();
            s.rows.insert(pos, r.clone());
            if s.validate().is_ok() && s.own_rows() <= g.lineage.max_rows() && !out.iter().any(|(_, o)| *o == s) {
                out.push((format!("+ {} at R{}", r.describe(), pos + 1), s));
            }
        }
    }
    out
}

/// A greedy plateau search on the bank share under `t` (as `oath_lib::climb`): (set, bank, edits, per-step gains).
fn climb(g: &Game, start: &RuleSet, t: Option<Tr>) -> (RuleSet, f64, Vec<String>, Vec<f64>) {
    let mut cur = start.clone();
    let mut cur_b = bank(&sims(g, &cur, t, FULL, TAG_A));
    let (mut taken, mut gains) = (Vec::new(), Vec::new());
    for _ in 0..STEPS {
        let cands = edits(g, &cur);
        let mut sc: Vec<(f64, usize)> = cands.iter().enumerate().map(|(i, (_, s))| (bank(&sims(g, s, t, SCREEN, TAG_A)), i)).collect();
        sc.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
        let mut best: Option<(f64, usize)> = None;
        for &(_, i) in sc.iter().take(TOP) {
            let b = bank(&sims(g, &cands[i].1, t, FULL, TAG_A));
            if best.is_none_or(|x| b > x.0 + 1e-9) {
                best = Some((b, i));
            }
        }
        match best {
            Some((b, i)) if b > cur_b + GAIN => {
                gains.push(b - cur_b);
                taken.push(cands[i].0.clone());
                cur = cands[i].1.clone();
                cur_b = b;
            }
            Some((b, _)) => {
                gains.push(b - cur_b);
                break;
            }
            None => break,
        }
    }
    (cur, cur_b, taken, gains)
}

struct SetCtx {
    name: String,
    set: RuleSet,
    best_depth: u32,
    base: Vec<Out>,
    b0: RuleSet,
    b0_edits: Vec<String>,
    /// The set's lever: best single-row edit's move (measured on `CONF`), largest row drop's loss.
    edit: (f64, String),
    drop: (f64, String),
    /// B0's own panel and its largest row drop (what a row carries on a plateau set).
    b0_base: Vec<Out>,
    b0_drop: (f64, String),
}

/// The largest bank loss of dropping one of `set`'s non-exit, non-engagement rows (neutral hero).
fn drop_lever(g: &Game, set: &RuleSet, base: &[Out]) -> (f64, String) {
    let mut drop = (0.0f64, "none".to_string());
    for i in 0..set.rows.len() {
        if matches!(set.rows[i].verb.v.as_str(), "bank" | "return") || lever::engagement(&set.rows[i]) {
            continue;
        }
        let mut s = set.clone();
        s.rows.remove(i);
        let d = -paired(&sims(g, &s, None, MAIN, TAG_B), base, |o| o.bank as u8 as f64).0;
        if d > drop.0 {
            drop = (d, format!("R{} {}", i + 1, set.rows[i].describe()));
        }
    }
    drop
}

fn set_ctx(name: &str, set: &RuleSet, seed: u64) -> (SetCtx, Game) {
    let g = oath_lib::lineage(set, seed);
    let base = sims(&g, set, None, MAIN, TAG_B);
    let (b0, _, b0_edits, _) = climb(&g, set, None);
    // the best single-row edit, measured on fresh seeds against the set
    let cands = edits(&g, set);
    let mut sc: Vec<(f64, usize)> = cands.iter().enumerate().map(|(i, (_, s))| (bank(&sims(&g, s, None, SCREEN, TAG_A)), i)).collect();
    sc.sort_by(|a, b| b.0.total_cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut edit = (0.0f64, "none".to_string());
    for &(_, i) in sc.iter().take(TOP) {
        let d = paired(&sims(&g, &cands[i].1, None, MAIN, TAG_B), &base, |o| o.bank as u8 as f64).0;
        if d > edit.0 {
            edit = (d, cands[i].0.clone());
        }
    }
    let drop = drop_lever(&g, set, &base);
    let b0_base = sims(&g, &b0, None, MAIN, TAG_B);
    let b0_drop = drop_lever(&g, &b0, &b0_base);
    (SetCtx { name: name.into(), set: set.clone(), best_depth: g.lineage.best_depth, base, b0, b0_edits, edit, drop, b0_base, b0_drop }, g)
}

struct Read {
    set: usize,
    tr: Tr,
    fired: f64,
    d_bank: (f64, f64),
    d_bank_b0: (f64, f64),
    d_death: (f64, f64),
    d_gold: (f64, f64),
    diff: usize,
    edits: Vec<String>,
    gain_t: (f64, f64),
    gain_n: (f64, f64),
}

fn measure(g: &Game, c: &SetCtx, si: usize, tr: Tr) -> Read {
    let with = sims(g, &c.set, Some(tr), MAIN, TAG_B);
    let fired = with.iter().filter(|o| o.fired).count() as f64 / with.len() as f64;
    let d_bank = paired(&with, &c.base, |o| o.bank as u8 as f64);
    let d_death = paired(&with, &c.base, |o| o.death as u8 as f64);
    let d_gold = paired(&with, &c.base, |o| o.gold as f64);
    let d_bank_b0 = paired(&sims(g, &c.b0, Some(tr), MAIN, TAG_B), &c.b0_base, |o| o.bank as u8 as f64);
    let (bt, _, edits, _) = climb(g, &c.b0, Some(tr));
    let diff = oath_lib::row_diff(&bt, &c.b0);
    let (mut gain_t, mut gain_n) = ((0.0, 0.0), (0.0, 0.0));
    if diff > 0 {
        let bank_of = |o: &Out| o.bank as u8 as f64;
        gain_t = paired(&sims(g, &bt, Some(tr), CONF, TAG_B ^ 7), &sims(g, &c.b0, Some(tr), CONF, TAG_B ^ 7), bank_of);
        gain_n = paired(&sims(g, &bt, None, CONF, TAG_B ^ 7), &sims(g, &c.b0, None, CONF, TAG_B ^ 7), bank_of);
    }
    Read { set: si, tr, fired, d_bank, d_bank_b0, d_death, d_gold, diff, edits, gain_t, gain_n }
}

/// A build: the trait's best set differs from B0 and the difference pays under the trait and not
/// (as much) without it — confirmed on fresh seeds.
fn over_lever(r: &Read, c: &SetCtx) -> bool {
    r.d_bank.0.abs() > c.edit.0.max(c.drop.0).max(0.005) || r.d_bank_b0.0.abs() > c.b0_drop.0.max(0.005)
}

fn is_build(r: &Read) -> bool {
    r.diff >= 1 && r.gain_t.0 >= 0.03 && r.gain_t.0 - r.gain_n.0 >= 0.03
}

fn main() {
    riddle_core::forecast::set_parallel_sims(false);
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let seed: u64 = get("--seed").and_then(|s| s.parse().ok()).unwrap_or(1);
    let threads: usize = std::env::var("RIDDLE_THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(8).clamp(1, 8);
    let want = get("--sets").unwrap_or_else(|| "631fe23.raterAU,631fe23.raterAV,9720ff7.raterAW,9720ff7.raterAX".into());
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let sets: Vec<(String, RuleSet)> = want
        .split(',')
        .map(|w| {
            let w = if w.contains('.') { w.to_string() } else { ["631fe23.rater", "9720ff7.rater"].iter().map(|p| format!("{p}{w}")).find(|n| std::path::Path::new(&format!("{dir}/{n}.rules.json")).exists()).unwrap_or(w.into()) };
            let s = RuleSet::parse(&std::fs::read_to_string(format!("{dir}/{w}.rules.json")).unwrap()).unwrap();
            (w, s)
        })
        .collect();
    let tf = get("--traits").unwrap_or_else(|| "all".into());
    let traits: Vec<Tr> = all_traits()
        .into_iter()
        .filter(|t| match tf.as_str() {
            "all" => true,
            "controls" => t.cost == Cost::None,
            f => t.code().contains(f) || t.name().contains(f),
        })
        .collect();
    let rejected: Vec<Tr> = traits.iter().copied().filter(|t| t.no_op()).collect();
    let traits: Vec<Tr> = traits.into_iter().filter(|t| !t.no_op()).collect();
    let t0 = std::time::Instant::now();
    eprintln!("{} sets × {} traits ({} rejected statically: {}) on {threads} threads", sets.len(), traits.len(), rejected.len(), rejected.iter().map(|t| t.code()).collect::<Vec<_>>().join(", "));

    // per set: the lineage, the base panel, B0, the lever (one set per thread)
    let ctxs: Mutex<Vec<(usize, SetCtx, Game)>> = Mutex::new(Vec::new());
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads.min(sets.len()) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((n, s)) = sets.get(i) else { break };
                let (c, g) = set_ctx(n, s, seed);
                eprintln!("  {n}: D{} · base bank {:.0}% · B0 {:?} · edit {:+.1} `{}` · drop {:+.1} `{}` ({:.0}s)", c.best_depth, 100.0 * bank(&c.base), c.b0_edits, 100.0 * c.edit.0, c.edit.1, 100.0 * c.drop.0, c.drop.1, t0.elapsed().as_secs_f64());
                ctxs.lock().unwrap().push((i, c, g));
            });
        }
    });
    let mut ctxs = ctxs.into_inner().unwrap();
    ctxs.sort_by_key(|c| c.0);
    let games: Vec<Game> = ctxs.iter().map(|c| c.2.sim_clone()).collect();
    let ctxs: Vec<SetCtx> = ctxs.into_iter().map(|c| c.1).collect();

    let jobs: Vec<(usize, Tr)> = traits.iter().flat_map(|t| (0..ctxs.len()).map(move |s| (s, *t))).collect();
    let reads: Mutex<Vec<Read>> = Mutex::new(Vec::new());
    let next = AtomicUsize::new(0);
    std::thread::scope(|sc| {
        for _ in 0..threads {
            let gs: Vec<Game> = games.iter().map(|g| g.sim_clone()).collect();
            let (next, jobs, ctxs, reads) = (&next, &jobs, &ctxs, &reads);
            sc.spawn(move || loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                let Some((si, t)) = jobs.get(i) else { break };
                let r = measure(&gs[*si], &ctxs[*si], *si, *t);
                if i % 16 == 0 {
                    eprintln!("  [{i}/{}] {:.0}s", jobs.len(), t0.elapsed().as_secs_f64());
                }
                reads.lock().unwrap().push(r);
            });
        }
    });
    let mut reads = reads.into_inner().unwrap();
    reads.sort_by(|a, b| a.tr.code().cmp(&b.tr.code()).then(a.set.cmp(&b.set)));

    println!("\n## sets (seed {seed}; lineage after {} h)", oath_lib::OATH_HOURS);
    for c in &ctxs {
        println!("{:<18} D{:<2} base bank {:>3.0}% death {:>3.0}% · B0 {:?} · lever: edit {:+.1} `{}` · drop {:+.1} `{}` · B0 bank {:.0}% drop {:+.1} `{}`", c.name, c.best_depth, 100.0 * bank(&c.base), 100.0 * c.base.iter().filter(|o| o.death).count() as f64 / c.base.len() as f64, c.b0_edits, 100.0 * c.edit.0, c.edit.1, 100.0 * c.drop.0, c.drop.1, 100.0 * bank(&c.b0_base), 100.0 * c.b0_drop.0, c.b0_drop.1);
    }
    println!("\n## per trait × set: fired% · Δbank ±pm · Δdeath · Δgold · diff rows · gain under trait / without · build?");
    let mut json = Vec::new();
    for r in &reads {
        let c = &ctxs[r.set];
        let lever = c.edit.0.max(c.drop.0);
        println!(
            "{:<22} {:<28} {:<18} fired {:>3.0}% · bank {:+5.1}±{:<4.1} death {:+5.1} gold {:+6.1} · diff {} {:?} · gain {:+.1}±{:.1} / {:+.1} · {}{}",
            r.tr.code(),
            r.tr.name(),
            c.name,
            100.0 * r.fired,
            100.0 * r.d_bank.0,
            100.0 * r.d_bank.1,
            100.0 * r.d_death.0,
            r.d_gold.0,
            r.diff,
            r.edits,
            100.0 * r.gain_t.0,
            100.0 * r.gain_t.1,
            100.0 * r.gain_n.0,
            if is_build(r) { "BUILD" } else { "number" },
            if over_lever(r, c) { format!(" · OVER LEVER (B0 {:+.1} vs drop {:+.1})", 100.0 * r.d_bank_b0.0, 100.0 * c.b0_drop.0) } else { format!(" · B0 {:+.1}", 100.0 * r.d_bank_b0.0) }
        );
        json.push(format!(
            "{{\"trait\":\"{}\",\"name\":\"{}\",\"set\":\"{}\",\"fired\":{:.3},\"dBank\":{:.4},\"dBankPm\":{:.4},\"dBankB0\":{:.4},\"dDeath\":{:.4},\"dGold\":{:.2},\"diff\":{},\"edits\":{:?},\"gainT\":{:.4},\"gainTPm\":{:.4},\"gainN\":{:.4},\"build\":{},\"lever\":{:.4}}}",
            r.tr.code(),
            r.tr.name(),
            c.name,
            r.fired,
            r.d_bank.0,
            r.d_bank.1,
            r.d_bank_b0.0,
            r.d_death.0,
            r.d_gold.0,
            r.diff,
            r.edits,
            r.gain_t.0,
            r.gain_t.1,
            r.gain_n.0,
            is_build(r),
            lever
        ));
    }
    println!("\n## per trait: builds on N sets · max |Δbank| vs the set's lever · verdict");
    let mut pass = 0;
    let mut n = 0;
    for t in &traits {
        let rs: Vec<&Read> = reads.iter().filter(|r| r.tr == *t).collect();
        let builds = rs.iter().filter(|r| is_build(r)).count();
        let over = rs.iter().filter(|r| over_lever(r, &ctxs[r.set])).count();
        let ok = builds >= 1 && over == 0;
        n += 1;
        pass += ok as u32;
        let maxd = rs.iter().map(|r| r.d_bank.0.abs()).fold(0.0, f64::max);
        println!("{:<22} {:<28} builds {}/{} · max |Δbank| {:>4.1} · over lever {} · {}", t.code(), t.name(), builds, rs.len(), 100.0 * maxd, over, if ok { "PASS" } else if builds == 0 { "number" } else { "too strong" });
    }
    println!("\n{pass}/{n} traits pass (≥ 1 set where the best set differs and pays only with the trait; no set where the trait outweighs the lever) · {:.0}s", t0.elapsed().as_secs_f64());
    if let Some(out) = get("--out") {
        std::fs::write(out, format!("[\n{}\n]\n", json.join(",\n"))).unwrap();
    }
}
