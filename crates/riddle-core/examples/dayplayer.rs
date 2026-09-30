//! Cut 30 §6 (docs/CUT30.md): fourteen simulated days of the idle-first bots — IDLE (sends at each
//! check-in, never edits, picks, buys or deposits; the wake takes card 1), PICKED (IDLE + the
//! forecast's top package move, a blacksmith step when affordable, a bank deposit), TUNED (PICKED +
//! the pen once it opens: the situation answers, the worst death's patch, the stall patch, the wall's
//! edit, the kennel fielded) and RANDOM (random package picks, random rows once the pen opens) — and
//! TUNED's leave-one-outs (TUNED less one system: packages, pen, forge, pets, bank, quests). Prints the
//! per-seed milestones and the idle gates; `--gate` exits non-zero on a failed bar (never weaken a bar).
//!   cargo run -q --profile fast -p riddle-core --example dayplayer -- [--seeds 8] [--days 14] [--checkins 3] [--gate] [--bots idle,picked,tuned,random] [--loo] [--verbose]
use riddle_core::rules::{Row, RuleSet};
use riddle_core::rng::Rng;
use riddle_core::Game;

/// Milestones (Cut 30 §6: time-to-milestone = simulated hours to reach each).
const MILESTONES: [u32; 5] = [8, 13, 18, 23, 28];
const SYSTEMS: [&str; 6] = ["packages", "pen", "forge", "pets", "bank", "quests"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Bot {
    Idle,
    Picked,
    Tuned,
    Random,
}

impl Bot {
    fn name(self) -> &'static str {
        match self {
            Bot::Idle => "IDLE",
            Bot::Picked => "PICKED",
            Bot::Tuned => "TUNED",
            Bot::Random => "RANDOM",
        }
    }
}

#[derive(Clone, Copy)]
struct Cfg {
    bot: Bot,
    /// TUNED less one system (leave-one-out).
    without: Option<&'static str>,
}

impl Cfg {
    fn label(&self) -> String {
        match self.without {
            Some(s) => format!("{}-{s}", self.bot.name()),
            None => self.bot.name().to_string(),
        }
    }
    fn has(&self, s: &str) -> bool {
        self.without != Some(s)
    }
}

#[derive(Default, Clone)]
struct SeedOut {
    seed: u64,
    /// Hours to each milestone (`MILESTONES`), if reached.
    hours: Vec<Option<f64>>,
    best_day: Vec<u32>,
    /// Purse + bank change per day (the day's net).
    gold_day: Vec<i64>,
    /// Longest run of days with no new best before D23 was reached.
    stall: usize,
    /// The day the Mirror King fell (1-based), if he did.
    king_day: Option<usize>,
    stance_level_day: Vec<u32>,
    checkins: u32,
    grew: u32,
    stage_days: usize,
    rules: String,
}

fn boss_of(depth: u32) -> Option<&'static str> {
    riddle_core::descent::boss_for(depth)
}

/// An own row into the pen (the compiled set's top): room is made from the pen's bottom.
fn insert_row(g: &mut Game, row: Row, at: i32) -> bool {
    if at < 0 || !g.lineage.pkg.pen_open {
        return false;
    }
    let max_rows = g.vocabulary().max_rows;
    let mut rules = g.lineage.rules().clone();
    if rules.rows.contains(&row) {
        return false;
    }
    let pen: Vec<usize> = (0..rules.rows.len()).filter(|&i| !rules.rows[i].is_pkg()).collect();
    if !row.is_card() && rules.own_rows() >= max_rows {
        if let Some(&drop) = pen.iter().rev().find(|&&i| !rules.rows[i].is_card()) {
            rules.rows.remove(drop);
        } else {
            return false;
        }
    }
    let pen_len = rules.rows.iter().take_while(|r| !r.is_pkg()).count();
    let at = (at as usize).min(pen_len);
    rules.rows.insert(at, row.from("player"));
    g.set_rules(rules).is_ok()
}

/// The situation answers a player writes once the pen is open and the fact is held (the stray's
/// tame row only when pets are in play).
fn write_own_rows(g: &mut Game, pets: bool) -> u32 {
    if !g.lineage.pkg.pen_open {
        return 0;
    }
    let mut n = 0;
    let situations = std::iter::once("stray").filter(|_| pets).chain(riddle_core::descent::SITUATION_DEPTHS.iter().map(|(w, _)| *w));
    for what in situations {
        if !g.lineage.facts.contains(what) {
            continue;
        }
        let row = riddle_core::probes::situation_answer(what);
        let vocab = g.vocabulary();
        let ok = row.conds.iter().all(|c| vocab.conds.iter().any(|v| v.k == c.k && v.t == c.t)) && vocab.verbs.iter().any(|v| v.v == row.verb.v && v.a == row.verb.a);
        if !ok || g.lineage.rules().rows.contains(&row) {
            continue;
        }
        if g.lineage.rules().own_rows() < g.vocabulary().max_rows && insert_row(g, row, 0) {
            n += 1;
        }
    }
    n
}

/// PICKED's package move: the forecast's top swap or level spend, when it beats the set as it stands.
fn pick_package(g: &mut Game, verbose: bool, day: usize, swap: bool) -> bool {
    // the marks buy the worn stance's next level first (a level is progress the forecast need not price)
    let stance = g.lineage.pkg.stance.clone();
    let mut moved = false;
    while riddle_core::packages::level_price(&g.lineage, &stance).is_some_and(|m| m <= g.lineage.marks) {
        if g.spend_level(&stance).is_err() {
            break;
        }
        moved = true;
    }
    if !swap || riddle_core::packages::candidates(&g.lineage).iter().all(|c| c.1 == "level") {
        return moved;
    }
    let opts = g.package_options(PICK_SIMS);
    if verbose {
        eprintln!("  day {} options: {}", day + 1, opts.iter().map(|o| format!("{} {} {:+.2} (p{:.2} r{:.2} b{:.2} d{:.2})", o.action, o.id, riddle_core::packages::score(o), o.past, o.reach, o.bank, o.death)).collect::<Vec<_>>().join(" | "));
    }
    let Some(top) = opts.iter().find(|o| o.action == "equip") else { return moved };
    // a swap when it clearly helps (the panel's noise is ~±0.1 at these sims)
    if riddle_core::packages::score(top) > PICK_BAR && riddle_core::packages::apply(&mut g.lineage, &top.id, &top.action, top.slot).is_ok() {
        if verbose {
            eprintln!("  day {} {} {} (Δpast {:+.2} Δbank {:+.2} Δdeath {:+.2})", day + 1, top.action, top.id, top.d_past, top.d_bank, top.d_death);
        }
        return true;
    }
    moved
}

/// The forecast's panel for a package move, and the move it must clear.
const PICK_SIMS: u32 = 32;
const PICK_BAR: f64 = 0.1;

/// A blacksmith step whenever the purse pays it with `reserve` to spare.
fn forge(g: &mut Game, reserve: i32) -> u32 {
    let mut n = 0;
    loop {
        let lads = riddle_core::kit::ladders(&g.lineage);
        let Some((slot, price)) = lads.iter().filter_map(|l| l.next.as_ref().map(|x| (l.slot.clone(), x.price))).min_by_key(|x| x.1) else { break };
        if g.lineage.gold < price as i32 + reserve || riddle_core::kit::buy(g, &slot).is_err() {
            break;
        }
        n += 1;
    }
    n
}

fn field_kennel(g: &mut Game) {
    let slots = g.lineage.party_slots() as usize;
    if g.lineage.party.len() < slots && !g.lineage.kennel.is_empty() {
        let mut ids: Vec<u32> = g.lineage.party.iter().map(|c| c.id).collect();
        let mut k: Vec<_> = g.lineage.kennel.iter().filter(|c| !ids.contains(&c.id)).collect();
        k.sort_by_key(|c| std::cmp::Reverse(c.level));
        for c in k.iter().take(slots - ids.len()) {
            ids.push(c.id);
        }
        let _ = g.set_party(ids);
    }
}

fn play(seed: u64, days: usize, checkins: u64, cfg: Cfg, verbose: bool) -> SeedOut {
    let interval = 24 * 3600 / checkins;
    let mut g = Game::new(seed);
    if !cfg.has("quests") {
        g.lineage.town.off = true;
    }
    let mut out = SeedOut { seed, hours: vec![None; MILESTONES.len()], ..Default::default() };
    let mut rng = Rng::derive(seed, 0x5EED_0B07);
    let mut last_best = 0u32;
    let mut stall_cur = 0usize;
    let mut reached23 = false;
    let mut k = 0u64;
    for day in 0..days {
        let wealth0 = g.lineage.gold as i64 + g.lineage.town.bank as i64;
        let stages0 = riddle_core::town::stage_set(&g.lineage);
        let mut opened = false;
        for ci in 0..checkins {
            k += 1;
            let tuned = cfg.bot == Bot::Tuned && cfg.has("pen") && g.lineage.pkg.pen_open;
            let rep = if tuned { riddle_core::offline::run_offline_quick(&mut g, interval) } else { riddle_core::offline::run_offline_counts(&mut g, interval) };
            out.checkins += 1;
            out.grew += !rep.grew.is_empty() as u32;
            opened |= !rep.systems_opened.is_empty();
            let hours = (k * interval) as f64 / 3600.0;
            for (i, m) in MILESTONES.iter().enumerate() {
                if out.hours[i].is_none() && g.lineage.best_depth >= *m {
                    out.hours[i] = Some(hours);
                }
            }
            if out.king_day.is_none() && g.lineage.kills.contains("mirror_king") {
                out.king_day = Some(day + 1);
            }
            if verbose && !rep.packages.is_empty() {
                eprintln!("  [{}] day {} {}", cfg.label(), day + 1, rep.packages.join(" · "));
            }
            if verbose {
                eprintln!("  [{}] day {} ci {} runs {} bank {} ret {} deaths {:?} best D{}", cfg.label(), day + 1, ci, rep.runs, rep.banked, rep.returned, rep.deaths.iter().map(|d| format!("{}×{}", d.cause, d.n)).collect::<Vec<_>>(), g.lineage.best_depth);
            }
            match cfg.bot {
                Bot::Idle => {}
                Bot::Random => {
                    let cands = riddle_core::packages::candidates(&g.lineage);
                    if !cands.is_empty() && rng.chance(50) {
                        let (id, action, slot) = &cands[rng.below(cands.len() as u32) as usize];
                        let _ = riddle_core::packages::apply(&mut g.lineage, id, action, *slot);
                    }
                    if g.lineage.pkg.pen_open && rng.chance(34) {
                        let vocab = g.vocabulary();
                        let set = riddle_core::probes::random_rules(&mut rng, &vocab, 3);
                        let mut rules = g.lineage.rules().clone();
                        rules.rows.retain(|r| r.is_pkg());
                        for (i, r) in set.rows.into_iter().enumerate() {
                            rules.rows.insert(i, r.from("player"));
                        }
                        let _ = g.set_rules(rules);
                    }
                }
                Bot::Picked | Bot::Tuned => {
                    if cfg.has("packages") {
                        pick_package(&mut g, verbose, day, ci == 0);
                    }
                    if tuned {
                        let pets = cfg.has("pets");
                        write_own_rows(&mut g, pets);
                        if let Some(id) = rep.worst_death_id {
                            if let Some(death) = g.death(id) {
                                if death.verdict == "gap" {
                                    if let Some(p) = death.patches.first() {
                                        if p.survive > death.baseline + 0.15 && p.forecast_delta >= 0.0 {
                                            insert_row(&mut g, p.row.clone(), 0);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(stall) = &rep.stall {
                            if let Some(p) = stall.patches.first().filter(|p| !p.remove && !p.replace) {
                                insert_row(&mut g, p.row.clone(), 0);
                            }
                        }
                        let fresh = g.lineage.wall_day != Some(g.lineage.day);
                        if let Some(w) = g.wall_edit().filter(|_| fresh) {
                            if g.set_rules(w.rules.clone()).is_ok() {
                                if let Some(s) = w.start {
                                    let _ = g.set_start(s);
                                }
                                if verbose {
                                    eprintln!("  [{}] day {} wall D{}: {}", cfg.label(), day + 1, w.depth, w.edits.join(" ; "));
                                }
                            }
                        }
                        // room for the pen: the cheapest row unlock the marks pay for
                        let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.available && u.id.starts_with("row") && u.cost <= g.lineage.marks).collect();
                        opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                        if let Some(u) = opts.first() {
                            let _ = g.buy(&u.id);
                        }
                        if pets {
                            field_kennel(&mut g);
                        }
                    }
                    if cfg.has("forge") {
                        forge(&mut g, 150);
                    }
                    if cfg.has("bank") && riddle_core::town::built(&g.lineage, "bank") {
                        // the purse keeps the next forge step and the shelf's money; the rest earns
                        let next = riddle_core::kit::ladders(&g.lineage).iter().filter_map(|l| l.next.as_ref().map(|x| x.price as i32)).min().unwrap_or(0);
                        let spare = g.lineage.gold - next - 500;
                        if spare > 0 {
                            let _ = g.bank_deposit(spare);
                        }
                    }
                }
            }
            // a vault brought along (a human sends the heir out with what it has; every bot)
            let ids: Vec<u32> = g.lineage.vault.iter().map(|i| i.id).collect();
            g.loadout(ids);
        }
        let best = g.lineage.best_depth;
        out.best_day.push(best);
        out.gold_day.push(g.lineage.gold as i64 + g.lineage.town.bank as i64 - wealth0);
        out.stance_level_day.push(g.lineage.pkg.level(&g.lineage.pkg.stance));
        opened |= riddle_core::town::stage_set(&g.lineage).len() > stages0.len();
        out.stage_days += opened as usize;
        if best > last_best {
            last_best = best;
            stall_cur = 0;
        } else if !reached23 {
            stall_cur += 1;
            out.stall = out.stall.max(stall_cur);
        }
        reached23 |= best >= 23;
        if verbose {
            let p = &g.lineage.pkg;
            eprintln!("  [{}] s{} day {} D{} ${} bank {} L{} {} L{} drills {} kit {} pen {}", cfg.label(), seed, day + 1, best, g.lineage.gold, g.lineage.town.bank, g.lineage.class_level(), p.stance, p.level(&p.stance), p.drills.len(), riddle_core::kit::KIT_SLOTS.iter().map(|s| riddle_core::kit::owned(&g.lineage, s)).sum::<u32>(), p.pen_open);
        }
    }
    out.rules = g.lineage.rules().rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | ");
    out
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

/// Hours to a milestone, `cap` when never reached (14 days + one check-in: slower than any reach).
fn hours_or(o: &SeedOut, i: usize, cap: f64) -> f64 {
    o.hours[i].unwrap_or(cap)
}

/// The median over seeds of slower ÷ faster at milestone `i`; a milestone the faster bot reaches and
/// the slower never does counts as passed (∞).
fn ratio(slow: &[SeedOut], fast: &[SeedOut], i: usize) -> f64 {
    let rs: Vec<f64> = slow
        .iter()
        .zip(fast)
        .filter_map(|(s, f)| match (s.hours[i], f.hours[i]) {
            (_, None) => None,
            (None, Some(_)) => Some(f64::INFINITY),
            (Some(a), Some(b)) => Some(a / b.max(1.0)),
        })
        .collect();
    if rs.is_empty() {
        return f64::NAN;
    }
    median(rs)
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 8);
    let days = get("--days", 14) as usize;
    let checkins = get("--checkins", 3);
    let gate = a.iter().any(|x| x == "--gate");
    let verbose = a.iter().any(|x| x == "--verbose");
    let loo = a.iter().any(|x| x == "--loo") || gate;
    let only = a.iter().position(|x| x == "--only").and_then(|i| a.get(i + 1)).and_then(|s| s.parse::<u64>().ok());
    let bots_arg = a.iter().position(|x| x == "--bots").and_then(|i| a.get(i + 1)).cloned().unwrap_or_else(|| "idle,picked,tuned,random".into());
    let mut cfgs: Vec<Cfg> = bots_arg
        .split(',')
        .filter_map(|b| match b {
            "idle" => Some(Bot::Idle),
            "picked" => Some(Bot::Picked),
            "tuned" => Some(Bot::Tuned),
            "random" => Some(Bot::Random),
            _ => None,
        })
        .map(|bot| Cfg { bot, without: None })
        .collect();
    if loo {
        for s in SYSTEMS {
            cfgs.push(Cfg { bot: Bot::Tuned, without: Some(s) });
        }
    }
    let t0 = std::time::Instant::now();
    let jobs: Vec<(usize, u64)> = (0..cfgs.len()).flat_map(|c| (1..=seeds).filter(|s| only.is_none_or(|o| o == *s)).map(move |s| (c, s))).collect();
    let threads = get("--threads", std::thread::available_parallelism().map(|n| n.get() as u64).unwrap_or(8)) as usize;
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<Vec<(usize, SeedOut)>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads.max(1) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(&(c, s)) = jobs.get(i) else { break };
                let o = play(s, days, checkins, cfgs[c], verbose);
                results.lock().unwrap().push((c, o));
            });
        }
    });
    let mut res = results.into_inner().unwrap();
    res.sort_by_key(|(c, o)| (*c, o.seed));
    let by = |label: &str| -> Vec<SeedOut> { res.iter().filter(|(c, _)| cfgs[*c].label() == label).map(|(_, o)| o.clone()).collect() };
    let cap = (days as f64 + 1.0) * 24.0;
    for (ci, cfg) in cfgs.iter().enumerate() {
        println!("\n{} ({} seeds)", cfg.label(), res.iter().filter(|(c, _)| *c == ci).count());
        println!("seed  {}  best by day", MILESTONES.iter().map(|m| format!("{:>6}", format!("h→D{m}"))).collect::<Vec<_>>().join(" "));
        for (_, o) in res.iter().filter(|(c, _)| *c == ci) {
            println!(
                "{:>4}  {}  {:?}  stall {} king {:?} L{:?}",
                o.seed,
                o.hours.iter().map(|h| h.map(|x| format!("{x:>6.0}")).unwrap_or_else(|| format!("{:>6}", "-"))).collect::<Vec<_>>().join(" "),
                o.best_day,
                o.stall,
                o.king_day,
                o.stance_level_day
            );
            if verbose {
                println!("      rules: {}", o.rules);
            }
        }
    }
    println!("\nbots over {seeds} seeds × {days} days × {checkins}/day ({:.0}s)", t0.elapsed().as_secs_f64());
    let idle = by("IDLE");
    let picked = by("PICKED");
    let tuned = by("TUNED");
    let random = by("RANDOM");
    let mut bars: Vec<(String, String, bool)> = Vec::new();
    if !idle.is_empty() {
        let n = idle.len();
        let d8 = idle.iter().filter(|o| o.hours[0].is_some_and(|h| h <= 24.0)).count();
        bars.push(("IDLE reaches D8 by the end of day 1 (every seed)".into(), format!("{d8}/{n}"), d8 == n));
        let d13 = median(idle.iter().map(|o| hours_or(o, 1, cap) / 24.0).collect());
        bars.push(("IDLE reaches D13 by day 4 (median day)".into(), format!("{d13:.1}"), d13 <= 4.0));
        let d23: Vec<f64> = idle.iter().map(|o| hours_or(o, 3, cap) / 24.0).collect();
        let d23n = d23.iter().filter(|d| **d <= 12.0).count();
        let d23m = median(d23);
        bars.push(("IDLE reaches D23 by day 12 (≥ 6/8 seeds, median)".into(), format!("{d23n}/{n} · {d23m:.1}"), d23n * 8 >= 6 * n && d23m <= 12.0));
        let stall = idle.iter().map(|o| o.stall).max().unwrap_or(0);
        bars.push(("IDLE longest best-depth stall before D23 ≤ 4 d".into(), format!("{stall}"), stall <= 4));
        let gold_days = idle.iter().map(|o| o.gold_day.iter().filter(|g| **g > 0).count()).min().unwrap_or(0);
        bars.push((format!("IDLE net gold > 0 every day ({days}/{days}, every seed)"), format!("{gold_days}"), gold_days == days));
        let king = idle.iter().filter(|o| o.king_day.is_some()).count();
        bars.push(("IDLE does not slay the Mirror King within 14 days".into(), format!("{king}/{n}"), king == 0));
        let l3 = median(idle.iter().map(|o| o.stance_level_day.iter().position(|l| *l >= 3).map_or(99.0, |d| d as f64 + 1.0)).collect());
        let l5 = median(idle.iter().map(|o| o.stance_level_day.iter().position(|l| *l >= 5).map_or(99.0, |d| d as f64 + 1.0)).collect());
        bars.push(("Stance L3 by day 2, L5 by day 7 (IDLE median)".into(), format!("{l3:.0} · {l5:.0}"), l3 <= 2.0 && l5 <= 7.0));
        let grew: u32 = idle.iter().map(|o| o.grew).sum();
        let cis: u32 = idle.iter().map(|o| o.checkins).sum();
        bars.push(("Every IDLE check-in grows ≥ 1 track".into(), format!("{grew}/{cis}"), grew == cis));
        let sd = median(idle.iter().map(|o| o.stage_days as f64).collect());
        bars.push((format!("Days with a stage opened: IDLE ≥ 8/{days} (median)"), format!("{sd:.0}"), sd >= 8.0));
    }
    if !picked.is_empty() && !idle.is_empty() {
        let rs: Vec<f64> = [1, 2, 3].iter().map(|&i| ratio(&idle, &picked, i)).collect();
        bars.push(("PICKED ≥ 1.5× IDLE at D13, D18, D23".into(), rs.iter().map(|r| format!("{r:.2}")).collect::<Vec<_>>().join(" · "), rs.iter().all(|r| *r >= 1.5)));
        let mut pairs = 0;
        let mut ok = 0;
        for (i, _) in MILESTONES.iter().enumerate() {
            for (p, s) in picked.iter().zip(&idle) {
                if p.hours[i].is_none() && s.hours[i].is_none() {
                    continue;
                }
                pairs += 1;
                ok += (hours_or(p, i, cap) <= hours_or(s, i, cap)) as u32;
            }
        }
        let pct = 100.0 * ok as f64 / pairs.max(1) as f64;
        bars.push(("IDLE never out-paces PICKED (≥ 90 % of seed × milestone)".into(), format!("{pct:.0}%"), pct >= 90.0));
        let sd = median(picked.iter().map(|o| o.stage_days as f64).collect());
        bars.push((format!("Days with a stage opened: PICKED ≥ 10/{days} (median)"), format!("{sd:.0}"), sd >= 10.0));
    }
    if !tuned.is_empty() && !picked.is_empty() {
        let rs: Vec<f64> = [2, 3, 4].iter().map(|&i| ratio(&picked, &tuned, i)).collect();
        bars.push(("TUNED ≥ 1.5× PICKED at D18, D23, D28".into(), rs.iter().map(|r| format!("{r:.2}")).collect::<Vec<_>>().join(" · "), rs.iter().all(|r| *r >= 1.5)));
    }
    if !random.is_empty() && !idle.is_empty() {
        let slower = random.iter().zip(&idle).filter(|(r, i)| hours_or(r, 1, cap) > hours_or(i, 1, cap)).count();
        let pct = 100.0 * slower as f64 / random.len() as f64;
        bars.push(("RANDOM slower than IDLE to D13 (≥ 80 % of seeds)".into(), format!("{pct:.0}%"), pct >= 80.0));
    }
    if !tuned.is_empty() && !idle.is_empty() {
        let step = (24 / checkins) as f64;
        let d23_tuned = median(tuned.iter().map(|o| hours_or(o, 3, cap)).collect());
        let d23_idle = median(idle.iter().map(|o| hours_or(o, 3, cap)).collect());
        let gap = (d23_idle - d23_tuned).max(1.0);
        let mut worst_req = String::new();
        let mut req_ok = true;
        let mut moves = Vec::new();
        for s in SYSTEMS {
            let v = by(&format!("TUNED-{s}"));
            if v.is_empty() {
                continue;
            }
            // never slower than IDLE at any milestone (± one check-in)
            for (i, _) in MILESTONES.iter().enumerate() {
                for (x, y) in v.iter().zip(&idle) {
                    if hours_or(x, i, cap) > hours_or(y, i, cap) + step && (x.hours[i].is_some() || y.hours[i].is_some()) {
                        req_ok = false;
                        worst_req = format!("{s} s{} D{}", x.seed, MILESTONES[i]);
                    }
                }
            }
            let d23 = median(v.iter().map(|o| hours_or(o, 3, cap)).collect());
            moves.push((s, d23 - d23_tuned));
        }
        if !moves.is_empty() {
            bars.push(("Nothing required: TUNED − S never slower than IDLE (± a check-in)".into(), if req_ok { "ok".into() } else { worst_req }, req_ok));
            let each = moves.iter().all(|(_, m)| *m > step);
            let dom = moves.iter().map(|(_, m)| m / gap).fold(0.0f64, f64::max);
            bars.push(("Each system adds value (D23 moves past a check-in)".into(), moves.iter().map(|(s, m)| format!("{s} {m:+.0}h")).collect::<Vec<_>>().join(" "), each));
            bars.push(("None > 60 % of TUNED − IDLE (D23)".into(), format!("{:.0}% of {gap:.0}h", dom * 100.0), dom <= 0.6));
        }
    }
    println!();
    println!("{:<64} {:>18}  result", "bar", "value");
    let mut fails = 0;
    for (name, value, ok) in &bars {
        println!("{:<64} {:>18}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        if !ok {
            fails += 1;
        }
    }
    println!("dayplayer: {}", if fails > 0 { "FAIL" } else { "all PASS" });
    if gate && fails > 0 {
        eprintln!("{fails} bar(s) FAIL");
        std::process::exit(1);
    }
    let _ = RuleSet::default();
    let _ = boss_of(8);
}
