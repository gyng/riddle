//! Cut 116 (docs/CUT116_VARIED_DESCENT.md): the varied descent's probes, on IDLE's own lineages
#![allow(dead_code)]
//! (`idle_lib::snapshots`: a fresh lineage at three check-ins a day, snapshot under each wall).
//!   affixes: per band boss × affix, from the wall's stone, paired panels (`camp_panel`'s seeds): the
//!            plain boss under the worn set against the affixed boss under the best arrived answer (the
//!            worn set, each arrived stance, each arrived tactic). Bar: the best answer passes the
//!            affixed wall at least as often as the worn set passes the plain one (no affix is a wall
//!            nothing arrived answers).
//!   forks:   at D9 and D14 (opened by Cut 116 §2), every arrived stance × tactic on both lanes from the
//!            fork's stone, reach past the band's boss (paired). Cut 26 §3's bar: each lane's best set
//!            loses ≥ 15 pts on the other lane (no set dominates both); and both lanes viable.
//!   guests:  over IDLE fortnights, the wandering champions' share of deaths, and their chronicle lines.
//!   cargo run -q --profile fast -p riddle-core --example varied_descent -- [--seeds 4] [--sims 48] [--days 14] [affixes|forks|guests]
#[path = "idle_lib/mod.rs"]
mod idle;

use riddle_core::descent::{affix_pool, guest_for, Affix, Route, BANDS};
use riddle_core::{packages, Game};
use std::sync::Mutex;

fn pool<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let next = std::sync::atomic::AtomicUsize::new(0);
    let out: Mutex<Vec<(usize, R)>> = Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(it) = items.get(i) else { break };
                let r = f(it);
                out.lock().unwrap().push((i, r));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, r)| r).collect()
}

/// The arrived answers on a lineage: the worn set, each other owned stance (at the worn stance's
/// level, as `idle::stance_past`), each owned tactic in the first slot.
fn answer_names(g: &Game) -> Vec<String> {
    let mut out = vec!["worn".to_string()];
    let worn = &g.lineage.pkg.stance;
    out.extend(idle::STANCES.iter().filter(|s| *s != worn && g.lineage.pkg.owned.contains(**s)).map(|s| s.to_string()));
    out.extend(g.lineage.pkg.owned.iter().filter(|id| packages::def(id).is_some_and(|d| d.kind == packages::Kind::Tactic) && !g.lineage.pkg.tactics.contains(*id)).cloned());
    out
}

/// The lineage wearing answer `name` (None: it cannot be worn).
fn wear(g: &Game, name: &str) -> Option<Game> {
    let mut c = g.sim_clone();
    if name == "worn" {
        return Some(c);
    }
    if packages::def(name).is_some_and(|d| d.kind == packages::Kind::Stance) {
        let runs = g.lineage.pkg.runs.get(&g.lineage.pkg.stance).copied().unwrap_or(0);
        c.lineage.pkg.runs.insert(name.to_string(), runs);
    }
    packages::equip(&mut c.lineage, name, 0).ok().map(|_| c)
}

/// The share of `sims` sends from the wall's stone that pass `wall`.
fn past(g: &Game, wall: u32, sims: u32) -> f64 {
    let mut c = g.sim_clone();
    c.lineage.start = c.lineage.stones().into_iter().filter(|s| *s <= wall).max().unwrap_or(1);
    c.lineage.best_depth = c.lineage.best_depth.max(wall);
    let set = packages::compile(&c.lineage);
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    rs.iter().filter(|r| r.max_depth > wall).count() as f64 / rs.len().max(1) as f64
}

/// The snapshot under the first wall deeper than `wall`, else the deepest.
fn later(v: &[(u32, String)], wall: u32) -> Option<&String> {
    v.iter().filter(|(w, _)| *w > wall).min_by_key(|(w, _)| *w).or_else(|| v.iter().max_by_key(|(w, _)| *w)).map(|(_, g)| g)
}

fn load(t: &str) -> Game {
    Game::load(t).expect("a snapshot loads")
}

fn pinned(g: &Game, pin: Vec<(String, Affix)>) -> Game {
    let mut c = g.sim_clone();
    c.lineage.affix_pin = Some(pin);
    c.lineage.guest_pin = Some(false);
    c
}

fn affixes(snaps: &[Vec<(u32, String)>], sims: u32) -> bool {
    println!("affix answerability (from the wall's stone, {sims} paired sends, {} IDLE lineages)", snaps.len());
    let mut ok_all = true;
    // Cut 117 §3: the boss × affix pairs whose best arrived answer differs from the plain wall's
    let (mut changed, mut pairs) = (0u32, 0u32);
    for wall in idle::WALLS {
        let Some(boss) = Route::BASE.boss(wall) else { continue };
        // (the lineage that has met him and gone on: IDLE's snapshot under the next wall, else its last —
        // under his own wall IDLE passes him ~0 %, a floor no answer can show against)
        let games: Vec<&String> = snaps.iter().filter_map(|v| later(v, wall)).collect();
        if games.is_empty() {
            continue;
        }
        // every arrived answer on every lineage, under `pin`: (worn, best answer, best read)
        let read = |pin: Vec<(String, Affix)>| -> (f64, String, f64, String) {
            let jobs: Vec<(&String, String)> = games.iter().flat_map(|g| answer_names(&load(g)).into_iter().map(move |n| (*g, n))).collect();
            let reads: Vec<Option<f64>> = pool(&jobs, |(g, n)| wear(&pinned(&load(g), pin.clone()), n).map(|c| past(&c, wall, sims)));
            let mut by: std::collections::BTreeMap<String, (f64, usize)> = Default::default();
            for ((_, n), r) in jobs.iter().zip(&reads) {
                let Some(r) = r else { continue };
                let e = by.entry(n.clone()).or_default();
                e.0 += r;
                e.1 += 1;
            }
            // (an answer is read over the lineages that have it)
            let worn = by.get("worn").map(|(s, n)| s / *n as f64).unwrap_or(0.0);
            let (best, bv) = by.iter().map(|(k, (s, n))| (k.clone(), s / *n as f64)).fold(("worn".to_string(), worn), |b, x| if x.1 > b.1 + 1e-9 { x } else { b });
            let mut all: Vec<(String, f64)> = by.iter().map(|(k, (s, n))| (k.clone(), s / *n as f64)).collect();
            all.sort_by(|a, b| b.1.total_cmp(&a.1));
            let top = all.iter().take(4).map(|(k, v)| format!("{k} {:.0}", 100.0 * v)).collect::<Vec<_>>().join(" · ");
            (worn, best, bv, top)
        };
        let (base, pbest, pbv, ptop) = read(Vec::new());
        println!("  D{wall} {boss}: plain — worn {:.0}% · best {pbest} {:.0}%   [{ptop}]", 100.0 * base, 100.0 * pbv);
        if pbv <= 0.0 {
            println!("    (no arrived answer passes him plain on these lineages: no signal, n/a)");
        }
        for a in affix_pool(boss) {
            let (worn, best, bv, top) = read(vec![(boss.to_string(), *a)]);
            // the bar: the best arrived answer passes the affixed wall at least as often as the worn set passes the plain one
            let ok = bv + 1e-9 >= base;
            ok_all &= ok;
            pairs += 1;
            changed += u32::from(best != pbest);
            println!("    {:<13} worn {:>3.0}% · best {:<18} {:>3.0}% (vs plain best {:+.0}){}  {}", a.word(), 100.0 * worn, best, 100.0 * bv, 100.0 * (bv - pbv), if best != pbest { " · answer changes" } else { "" }, if ok { "PASS" } else { "FAIL" });
            println!("        [{top}]");
        }
    }
    println!("  answer changes: {changed} of {pairs} boss × affix pairs (Cut 117 bar: ≥ 8 of 20)");
    ok_all
}

fn forks(snaps: &[Vec<(u32, String)>], sims: u32) -> bool {
    let mut ok_all = true;
    // (the D9 fork from IDLE's lineage past the Mother, the D14 fork from past the Lich: `later`)
    for (fork, wall) in [(9u32, 13u32), (14, 18)] {
        let band = BANDS.iter().position(|(a, _)| *a == fork).unwrap();
        let bar = BANDS[band].1 + 1;
        let lanes = [Route::BASE, Route::from_forks(&[fork]).unwrap()];
        let games: Vec<Game> = snaps.iter().filter_map(|v| later(v, wall).map(|g| load(g))).collect();
        // every arrived stance × (no tactic | each arrived tactic)
        let mut jobs: Vec<(usize, String, usize, String)> = Vec::new();
        for (i, g) in games.iter().enumerate() {
            let mut base = pinned(g, Vec::new());
            base.lineage.affix_pin = None;
            base.lineage.facts.insert(format!("fork:{fork}"));
            for r in lanes {
                let _ = base.lineage.light_waystones_on(fork, r);
            }
            base.lineage.start = fork;
            base.lineage.best_depth = base.lineage.best_depth.max(fork);
            let owned: Vec<String> = base.lineage.pkg.owned.iter().cloned().collect();
            let stances: Vec<&String> = owned.iter().filter(|id| packages::def(id).is_some_and(|d| d.kind == packages::Kind::Stance)).collect();
            let tactics: Vec<&String> = owned.iter().filter(|id| packages::def(id).is_some_and(|d| d.kind == packages::Kind::Tactic)).collect();
            let runs = base.lineage.pkg.runs.get(&base.lineage.pkg.stance).copied().unwrap_or(0);
            for s in &stances {
                for t in std::iter::once(None).chain(tactics.iter().map(Some)) {
                    let mut c = base.sim_clone();
                    c.lineage.pkg.runs.insert(s.to_string(), runs);
                    c.lineage.pkg.tactics.clear();
                    if packages::equip(&mut c.lineage, s, 0).is_err() {
                        continue;
                    }
                    if let Some(t) = t {
                        if packages::equip(&mut c.lineage, t, 0).is_err() {
                            continue;
                        }
                    } else {
                        packages::recompile(&mut c.lineage);
                    }
                    let name = format!("{s}{}", t.map(|t| format!("+{t}")).unwrap_or_default());
                    let save = c.save();
                    for l in 0..2 {
                        jobs.push((i, name.clone(), l, save.clone()));
                    }
                }
            }
        }
        let reads: Vec<f64> = pool(&jobs, |(_, _, l, c)| {
            let c = &load(c);
            let set = packages::compile(&c.lineage).with_route(lanes[*l]);
            let rs = riddle_core::forecast::camp_panel(c, &set, sims);
            rs.iter().filter(|r| r.max_depth >= bar).count() as f64 / rs.len().max(1) as f64
        });
        // (a set's read pooled over the lineages that have it)
        let mut by: std::collections::BTreeMap<String, [(f64, usize); 2]> = Default::default();
        for ((_, n, l, _), r) in jobs.iter().zip(&reads) {
            let e = by.entry(n.clone()).or_default();
            e[*l].0 += r;
            e[*l].1 += 1;
        }
        let mean = |e: &[(f64, usize); 2], l: usize| e[l].0 / e[l].1.max(1) as f64;
        println!("fork D{fork} ({} · {}; reach D{bar}, {sims} paired sends, {} IDLE lineages):", lanes[0].biome(fork).name(), lanes[1].biome(fork).name(), games.len());
        for (n, e) in &by {
            println!("    {n:<36} near {:>3.0}% · far {:>3.0}%", 100.0 * mean(e, 0), 100.0 * mean(e, 1));
        }
        if by.is_empty() {
            println!("  no lineage reached the fork: n=0");
            ok_all = false;
            continue;
        }
        let best = |l: usize| by.iter().fold(None::<(&String, f64)>, |b, (n, e)| if b.is_none_or(|b| mean(e, l) > b.1 + 1e-9) { Some((n, mean(e, l))) } else { b }).unwrap();
        let (b0, b1) = (best(0), best(1));
        let loss0 = b1.1 - mean(&by[b0.0], 1);
        let loss1 = b0.1 - mean(&by[b1.0], 0);
        let dominated = loss0 >= 0.15 - 1e-9 && loss1 >= 0.15 - 1e-9;
        let distinct = b0.0 != b1.0;
        let viable = b0.1.min(b1.1) >= 0.5 * b0.1.max(b1.1);
        println!("  near best {} {:.0}% · far best {} {:.0}% · cross-lane loss {:+.0} · {:+.0} pts (Cut 26: ≥ 15 each) {} · best sets differ {} · both viable (weaker ≥ ½) {}", b0.0, 100.0 * b0.1, b1.0, 100.0 * b1.1, 100.0 * loss0, 100.0 * loss1, if dominated { "PASS" } else { "FAIL" }, if distinct { "yes" } else { "no" }, if viable { "PASS" } else { "FAIL" });
        ok_all &= dominated && viable;
    }
    ok_all
}

fn guests(seeds: &[u64], days: usize) -> bool {
    let reads: Vec<(usize, usize, usize, usize)> = pool(seeds, |s| {
        let mut g = idle::fresh(*s);
        for _ in 0..days * 3 {
            riddle_core::offline::run_offline_counts(&mut g, 8 * 3600);
        }
        let l = &g.lineage;
        let route = l.rules().route();
        let deaths = l.graveyard.len();
        let by_guest = l.graveyard.iter().filter(|gr| (0..BANDS.len()).filter_map(|b| guest_for(l.seed, gr.heir, route, b)).any(|x| x.depth == gr.depth && x.kind == gr.cause)).count();
        let lines = l.chronicle.iter().filter(|c| c.contains(", the wandering")).count();
        if *s == 1 {
            for c in l.chronicle.iter().filter(|c| c.contains(", the wandering")).take(3) {
                println!("  chronicle: {c}");
            }
        }
        let named = l.grudges.iter().filter(|gr| (0..BANDS.len()).filter_map(|b| guest_for(l.seed, gr.heir, route, b)).any(|x| x.name == gr.name)).count();
        (deaths, by_guest, lines, named)
    });
    let (d, gd, lines, named) = reads.iter().fold((0, 0, 0, 0), |a, r| (a.0 + r.0, a.1 + r.1, a.2 + r.2, a.3 + r.3));
    let share = gd as f64 / d.max(1) as f64;
    println!("guests ({} IDLE fortnights): deaths {d} · by a wandering champion {gd} ({:.1}%) · grudges named for one {named} · chronicle lines {lines}", seeds.len(), 100.0 * share);
    let ok = share <= 0.05;
    println!("  guest deaths ≤ 5% of deaths: {}", if ok { "PASS" } else { "FAIL" });
    ok
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(d);
    let seeds: Vec<u64> = (1..=get("--seeds", 4)).collect();
    let sims = get("--sims", 48) as u32;
    let days = get("--days", 14) as usize;
    let only: Vec<&str> = args.iter().skip(1).filter(|a| ["affixes", "forks", "guests"].contains(&a.as_str())).map(|s| s.as_str()).collect();
    let want = |k: &str| only.is_empty() || only.contains(&k);
    let t = std::time::Instant::now();
    let mut ok = true;
    if want("affixes") || want("forks") {
        let snaps: Vec<Vec<(u32, String)>> = pool(&seeds, |s| idle::snapshots(*s, days).into_iter().map(|(w, g)| (w, g.save())).collect());
        if want("affixes") {
            ok &= affixes(&snaps, sims);
        }
        if want("forks") {
            ok &= forks(&snaps, sims);
        }
    }
    if want("guests") {
        ok &= guests(&seeds, days);
    }
    println!("varied_descent: {} ({:.0}s)", if ok { "all PASS" } else { "FAIL" }, t.elapsed().as_secs_f64());
}
