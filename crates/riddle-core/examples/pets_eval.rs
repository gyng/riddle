//! Cut 119's eval framework (docs/CUT119_COMPANIONS.md, "The eval framework" and its research amendments): one table
//! on the companion system, from the dayplayer's own IDLE and PICKED fortnights (`dayplayer::walk`, 3 check-ins a
//! day) and paired panels from their wall snapshots (the first check-in standing under each band wall; sends from
//! the deepest lit stone at or above it, the record at the wall, drills revoked, as `builds.rs` weighs a pick).
//!
//! - pet value: the party worn vs removed (paired sims), Δ past · bank · death and Δ camp score (builds.rs's
//!   `past + 0.3·reach + 0.2·bank − 0.2·death + 0.1·mean depth`, in points) ± the seed SD;
//! - pet share of damage dealt and of blows drawn, acts per run per pet (worn panels; `riddle_core::petstats`);
//! - pet deaths per 10 runs, named causes, recovered (loss-egg hatched or stray re-tamed) by day 14 and the days to it;
//! - the level curve (every owned pet; the longest-serving pet), tenure (heirs served), count and kinds, PICKED's
//!   pick rate per kind (party slots);
//! - the D1–D4 pickup walk (sends from D1 at the D8 snapshot, party worn vs removed);
//! - carry fetched on deaths (0 before Cut 119);
//! - the best kind per wall (each kind seen, alone in the party at the lineage's best pet level), its margin in SDs.
//!
//! Deterministic; seeds and jobs run in parallel (sims in place). Diagnostic, never the gate table.
//!   cargo run -q --profile fast -p riddle-core --example pets_eval -- [--seeds 16] [--sims 24] [--kind-sims 12] [--days 14] [--bots idle,picked[,tuned]]
#[path = "dayplayer.rs"]
#[allow(dead_code, unused_imports, unused_variables, unused_mut, clippy::all)]
mod dayplayer;
#[path = "jobcache_lib/mod.rs"]
mod jobcache;

use riddle_core::engine::ExitTier;
use riddle_core::petstats::{self, Stat};
use riddle_core::wire::Companion;
use riddle_core::Game;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

const WALLS: [u32; 5] = [8, 13, 18, 23, 28];
const ALL_BOTS: [&str; 3] = ["idle", "picked", "tuned"];
const DW: f64 = 0.2;
const KINDS: [&str; 8] = ["rat", "jackal", "goblin_archer", "monkey", "bloat", "pink_jelly", "skeleton", "ogre"];

fn pool<T: Sync, R: Send>(items: &[T], f: impl Fn(&T) -> R + Sync) -> Vec<R> {
    let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let threads = std::env::var("THREADS").ok().and_then(|v| v.parse().ok()).unwrap_or(threads);
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

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    let n = v.len();
    if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2.0 }
}

fn mean_sd(v: &[f64]) -> (f64, f64) {
    if v.is_empty() {
        return (f64::NAN, f64::NAN);
    }
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = if v.len() > 1 { v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) } else { 0.0 };
    (m, var.sqrt())
}

/// One bot's fortnight on one seed.
#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Fort {
    bot: String,
    snaps: Vec<(u32, String)>,
    /// per day end: (all owned pets' levels, the longest-serving pet's level, pets owned, party size)
    days: Vec<(Vec<u32>, u32, usize, usize)>,
    kinds: BTreeSet<String>,
    /// party slots filled, by kind, summed over check-ins
    slots: BTreeMap<String, u32>,
    /// per pet id: (first heir, last heir) seen in the party
    tenure: BTreeMap<u32, (u32, u32)>,
    /// live counters over the fortnight
    live: [u64; petstats::N],
    /// (check-in index, pets fell) and (check-in index, pets recovered)
    fell_at: Vec<(u32, u64)>,
    rec_at: Vec<(u32, u64)>,
}

fn fortnight(seed: u64, bot: &'static str, days: usize) -> Fort {
    let mut f = Fort { bot: bot.to_string(), ..Default::default() };
    let mut last = petstats::snapshot();
    let start = last;
    let mut k = 0u32;
    dayplayer::walk(seed, days, bot, |day, ci, g| {
        let now = petstats::snapshot();
        let d = petstats::delta(&last, &now);
        last = now;
        k += 1;
        if d[Stat::PetsFell as usize] > 0 {
            f.fell_at.push((k, d[Stat::PetsFell as usize]));
        }
        let rec = d[Stat::LossHatched as usize] + d[Stat::StraysRetamed as usize];
        if rec > 0 {
            f.rec_at.push((k, rec));
        }
        let l = &g.lineage;
        for c in &l.party {
            *f.slots.entry(c.kind.clone()).or_default() += 1;
            let e = f.tenure.entry(c.id).or_insert((l.heir, l.heir));
            e.1 = l.heir;
        }
        for c in l.all_companions() {
            f.kinds.insert(c.kind.clone());
        }
        for w in WALLS {
            if l.best_depth + 1 >= w && l.best_depth <= w && !f.snaps.iter().any(|(x, _)| *x == w) {
                f.snaps.push((w, g.save()));
            }
        }
        if ci == 2 {
            let levels: Vec<u32> = l.all_companions().map(|c| c.level).collect();
            let longest = l
                .all_companions()
                .max_by_key(|c| (f.tenure.get(&c.id).map_or(0, |t| t.1 - t.0 + 1), c.level, std::cmp::Reverse(c.id)))
                .map_or(0, |c| c.level);
            f.days.push((levels, longest, l.all_companions().count(), l.party.len()));
            let _ = day;
        }
    });
    f.live = petstats::delta(&start, &petstats::snapshot());
    f
}

#[derive(Clone)]
enum Wear {
    Worn,
    Removed,
    Kind(String),
    /// sends from D1 (the pickup walk), worn or not
    FromD1(bool),
}

struct Panel {
    v: [f64; 5],
    c: [u64; petstats::N],
    sends: usize,
    party: usize,
}

fn panel(save: &str, wall: u32, wear: &Wear, sims: u32) -> Panel {
    let g = Game::load(save).expect("snapshot");
    let mut c = g.sim_clone();
    match wear {
        Wear::FromD1(_) => c.lineage.start = 1,
        _ => {
            c.lineage.start = c.lineage.stones().into_iter().filter(|s| *s <= wall).max().unwrap_or(1);
            c.lineage.best_depth = c.lineage.best_depth.max(wall);
        }
    }
    for d in c.lineage.pkg.drills.iter_mut() {
        d.revoked = true;
    }
    let l = &mut c.lineage;
    match wear {
        Wear::Worn | Wear::FromD1(true) => {}
        Wear::Removed | Wear::FromD1(false) => {
            let mut p = std::mem::take(&mut l.party);
            l.kennel.append(&mut p);
        }
        Wear::Kind(k) => {
            let level = l.all_companions().map(|c| c.level).max().unwrap_or(1).max(1);
            let mut p = std::mem::take(&mut l.party);
            l.kennel.append(&mut p);
            let d = riddle_core::defs::monster_def(k);
            let tags: Vec<String> = d.tags.iter().map(|t| t.to_string()).collect();
            let rules = riddle_core::probes::default_companion_rules(&tags, level);
            l.party.push(Companion { id: 990_000, kind: k.clone(), name: "Probe".into(), level, tags, gen: 0, rules, max_rows: 1 + level as usize, hp: d.hp, max_hp: d.hp });
        }
    }
    let party = c.lineage.party.len();
    let set = riddle_core::packages::compile(&c.lineage);
    let before = petstats::snapshot();
    let rs = riddle_core::forecast::camp_panel(&c, &set, sims);
    let cd = petstats::delta(&before, &petstats::snapshot());
    let k = rs.len().max(1) as f64;
    let past = rs.iter().filter(|r| r.max_depth > wall).count() as f64 / k;
    let bank = rs.iter().filter(|r| r.tier == ExitTier::Bank).count() as f64 / k;
    let death = rs.iter().filter(|r| r.tier == ExitTier::Death).count() as f64 / k;
    let reach = rs.iter().filter(|r| r.max_depth >= wall).count() as f64 / k;
    let mean = rs.iter().map(|r| r.max_depth as f64).sum::<f64>() / k;
    Panel { v: [past, bank, death, reach, mean], c: cd, sends: rs.len(), party }
}

/// builds.rs's camp score, in points.
fn score(m: &[f64; 5]) -> f64 {
    100.0 * (m[0] + 0.3 * m[3] + 0.2 * m[1] - DW * m[2] + 0.1 * m[4])
}

fn arg(args: &[String], name: &str, def: u64) -> u64 {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()).unwrap_or(def)
}

fn main() {
    // (the fortnights' forecasts and verdicts on all cores — their sims never touch the live counters, and a panel is
    // bit-identical at any width; the paired panels below run in place, so their counters are this thread's)
    riddle_core::forecast::set_parallel_sims(true);
    let t0 = std::time::Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let seeds = arg(&args, "--seeds", 16);
    let sims = arg(&args, "--sims", 24) as u32;
    let kind_sims = arg(&args, "--kind-sims", 12) as u32;
    let days = arg(&args, "--days", 14) as usize;
    let bots: Vec<&'static str> = match args.iter().position(|a| a == "--bots").and_then(|i| args.get(i + 1)) {
        Some(v) => ALL_BOTS.iter().copied().filter(|b| v.split(',').any(|x| x == *b)).collect(),
        None => vec!["idle", "picked"],
    };

    // 1. the fortnights
    let jobs: Vec<(u64, &'static str)> = bots.iter().flat_map(|b| (1..=seeds).map(move |s| (s, *b))).collect();
    // (kept across runs under `RIDDLE_SRC_KEY`, as the gate jobs: `jobcache_lib`)
    let forts = pool(&jobs, |(s, b)| jobcache::cached("pets-fortnight", &[include_str!("dayplayer.rs"), include_str!("pets_eval.rs")], &format!("{b} {s} {days}"), || fortnight(*s, b, days)));
    eprintln!("fortnights {:.0}s", t0.elapsed().as_secs_f64());
    riddle_core::forecast::set_parallel_sims(false);
    // (the kinds weighed: every kind the lineages owned, and one of each tag a default row reads — pack, ranged,
    // thief, gas, splitter, undead — with a plain biter and a tank)
    let kinds: Vec<String> = forts.iter().flat_map(|f| f.kinds.iter().cloned()).chain(KINDS.iter().map(|k| k.to_string())).collect::<BTreeSet<_>>().into_iter().collect();

    // 2. the panels
    let mut pj: Vec<(usize, usize, Wear, u32)> = Vec::new(); // (fort, snap, wear, sims)
    for (fi, f) in forts.iter().enumerate() {
        for (si, (w, _)) in f.snaps.iter().enumerate() {
            pj.push((fi, si, Wear::Worn, sims));
            pj.push((fi, si, Wear::Removed, sims));
            for k in &kinds {
                pj.push((fi, si, Wear::Kind(k.clone()), kind_sims));
            }
            if *w == 8 {
                pj.push((fi, si, Wear::FromD1(true), sims));
                pj.push((fi, si, Wear::FromD1(false), sims));
            }
        }
    }
    let panels = pool(&pj, |(fi, si, wear, n)| {
        let (w, save) = &forts[*fi].snaps[*si];
        panel(save, *w, wear, *n)
    });
    eprintln!("panels {} in {:.0}s", pj.len(), t0.elapsed().as_secs_f64());
    let find = |fi: usize, si: usize, pred: &dyn Fn(&Wear) -> bool| -> Option<&Panel> { pj.iter().position(|(a, b, w, _)| *a == fi && *b == si && pred(w)).map(|i| &panels[i]) };

    println!("pets_eval — {seeds} seeds × {days} days, {} (dayplayer, 3 check-ins), sims {sims} (kinds {kind_sims}); score = builds.rs camp score (DW {DW})", bots.iter().map(|b| b.to_uppercase()).collect::<Vec<_>>().join(" + "));
    println!("walls reached: {}", bots.iter().map(|b| format!("{} {}", b.to_uppercase(), WALLS.iter().map(|w| format!("D{w} {}", forts.iter().filter(|f| f.bot == *b && f.snaps.iter().any(|s| s.0 == *w)).count())).collect::<Vec<_>>().join(" "))).collect::<Vec<_>>().join(" · "));

    // pet value
    println!("\n## pet value: party worn − removed (paired), mean over snapshots; Δscore ± seed SD; (n with a party / n)");
    for b in bots.iter().copied() {
        let mut line = format!("{:<7}", b.to_uppercase());
        for w in WALLS {
            let mut d = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
            let mut withp = 0;
            for (fi, f) in forts.iter().enumerate().filter(|(_, f)| f.bot == b) {
                if let Some(si) = f.snaps.iter().position(|s| s.0 == w) {
                    let on = find(fi, si, &|x| matches!(x, Wear::Worn)).unwrap();
                    let off = find(fi, si, &|x| matches!(x, Wear::Removed)).unwrap();
                    withp += (on.party > 0) as u32;
                    d.0.push(100.0 * (on.v[0] - off.v[0]));
                    d.1.push(100.0 * (on.v[1] - off.v[1]));
                    d.2.push(100.0 * (on.v[2] - off.v[2]));
                    d.3.push(score(&on.v) - score(&off.v));
                }
            }
            if d.0.is_empty() {
                line += &format!(" | D{w} —");
                continue;
            }
            let (m, sd) = mean_sd(&d.3);
            line += &format!(" | D{w} past {:+.1} bank {:+.1} death {:+.1} score {m:+.1}±{sd:.1} ({withp}/{})", mean_sd(&d.0).0, mean_sd(&d.1).0, mean_sd(&d.2).0, d.0.len());
        }
        println!("{line}");
    }

    // shares and acts
    println!("\n## pet share (worn panels, snapshots with a party): damage dealt · blows drawn · acts per run per pet (row acts)");
    for b in bots.iter().copied() {
        let mut line = format!("{:<7}", b.to_uppercase());
        let mut tot = [0u64; petstats::N];
        let mut pet_sends = 0usize;
        for w in WALLS {
            let mut c = [0u64; petstats::N];
            let mut ps = 0usize;
            for (fi, f) in forts.iter().enumerate().filter(|(_, f)| f.bot == b) {
                if let Some(si) = f.snaps.iter().position(|s| s.0 == w) {
                    let on = find(fi, si, &|x| matches!(x, Wear::Worn)).unwrap();
                    if on.party == 0 {
                        continue;
                    }
                    for i in 0..petstats::N {
                        c[i] += on.c[i];
                        tot[i] += on.c[i];
                    }
                    ps += on.sends * on.party;
                }
            }
            pet_sends += ps;
            line += &format!(" | D{w} {}", shares(&c, ps));
        }
        println!("{line} | all {}", shares(&tot, pet_sends));
    }

    // deaths, recovery
    println!("\n## pet deaths and recovery (live fortnight runs)");
    for b in bots.iter().copied() {
        let fs: Vec<&Fort> = forts.iter().filter(|f| f.bot == b).collect();
        let sum = |s: Stat| fs.iter().map(|f| f.live[s as usize]).sum::<u64>();
        let (runs, fell, named, hatched, retamed, tamed) = (sum(Stat::Runs), sum(Stat::PetsFell), sum(Stat::PetsFellNamed), sum(Stat::LossHatched), sum(Stat::StraysRetamed), sum(Stat::Tamed));
        // (each fall matched to the next recovery, first in first out; days at 3 check-ins a day)
        let mut lat = Vec::new();
        let mut recovered = 0u64;
        for f in &fs {
            let mut q: Vec<u32> = f.fell_at.iter().flat_map(|(k, n)| std::iter::repeat_n(*k, *n as usize)).collect();
            q.reverse();
            for (k, n) in &f.rec_at {
                for _ in 0..*n {
                    if let Some(at) = q.last().copied().filter(|at| at <= k) {
                        q.pop();
                        recovered += 1;
                        lat.push((*k - at) as f64 / 3.0);
                    }
                }
            }
        }
        println!(
            "{:<7} runs {runs} · tamed {tamed} · pets fell {fell} = {:.2} per 10 runs · named cause {:.0}% · loss-eggs hatched {hatched} · strays re-tamed {retamed} · recovered by day {days} {:.0}% · median days to recovery {:.1} (check-in grain)",
            b.to_uppercase(),
            10.0 * fell as f64 / runs.max(1) as f64,
            100.0 * named as f64 / fell.max(1) as f64,
            100.0 * recovered as f64 / fell.max(1) as f64,
            median(lat)
        );
    }

    // levels, tenure, counts, kinds
    println!("\n## level curve (day end; seeds with a pet): median of every owned pet's level · the longest-serving pet's level (median over seeds) · seeds with a pet");
    for b in bots.iter().copied() {
        let fs: Vec<&Fort> = forts.iter().filter(|f| f.bot == b).collect();
        let mut line = format!("{:<7}", b.to_uppercase());
        for d in [3usize, 7, 14] {
            let at: Vec<&(Vec<u32>, u32, usize, usize)> = fs.iter().filter_map(|f| f.days.get(d - 1)).filter(|x| !x.0.is_empty()).collect();
            let all: Vec<f64> = at.iter().flat_map(|x| x.0.iter().map(|l| *l as f64)).collect();
            line += &format!(" | day {d}: L{:.1} · longest L{:.1} · {}/{}", median(all), median(at.iter().map(|x| x.1 as f64).collect()), at.len(), fs.len());
        }
        println!("{line}");
    }
    println!("\n## tenure (heirs a pet served in the party; first to last heir seen at a check-in) · old hound (≥ 3 heirs) by day {days}");
    for b in bots.iter().copied() {
        let fs: Vec<&Fort> = forts.iter().filter(|f| f.bot == b).collect();
        let all: Vec<f64> = fs.iter().flat_map(|f| f.tenure.values().map(|t| (t.1 - t.0 + 1) as f64)).collect();
        let hound = fs.iter().filter(|f| f.tenure.values().any(|t| t.1 - t.0 + 1 >= 3)).count();
        let longest: Vec<f64> = fs.iter().map(|f| f.tenure.values().map(|t| (t.1 - t.0 + 1) as f64).fold(0.0, f64::max)).collect();
        println!("{:<7} pets fielded {} · median heirs served {:.1} · longest per seed median {:.1} · old hound on {hound}/{} seeds", b.to_uppercase(), all.len(), median(all), median(longest), fs.len());
    }
    println!("\n## pet count and kinds over the fortnight");
    for b in bots.iter().copied() {
        let fs: Vec<&Fort> = forts.iter().filter(|f| f.bot == b).collect();
        let mut line = format!("{:<7}", b.to_uppercase());
        for d in [1usize, 3, 7, 14] {
            let owned: Vec<f64> = fs.iter().filter_map(|f| f.days.get(d - 1)).map(|x| x.2 as f64).collect();
            let party: Vec<f64> = fs.iter().filter_map(|f| f.days.get(d - 1)).map(|x| x.3 as f64).collect();
            line += &format!(" | day {d}: owned {:.1} party {:.1}", mean_sd(&owned).0, mean_sd(&party).0);
        }
        let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
        for f in &fs {
            for k in &f.kinds {
                *kinds.entry(k).or_default() += 1;
            }
        }
        println!("{line} (mean over seeds)");
        println!("        kinds ever owned (seeds): {}", kinds.iter().map(|(k, n)| format!("{k} {n}")).collect::<Vec<_>>().join(" · "));
        let mut slots: BTreeMap<&str, u32> = BTreeMap::new();
        for f in &fs {
            for (k, n) in &f.slots {
                *slots.entry(k).or_default() += n;
            }
        }
        let total: u32 = slots.values().sum();
        let mut sv: Vec<_> = slots.into_iter().collect();
        sv.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        println!("        party slots by kind (pick rate, {total} slot·check-ins): {}", sv.iter().map(|(k, n)| format!("{k} {:.0}%", 100.0 * *n as f64 / total.max(1) as f64)).collect::<Vec<_>>().join(" · "));
    }

    // pickup
    println!("\n## D1–D4 pickup walk: sends from D1 at the D8 snapshot, per send: ticks walking to loot (steps) / ticks on D1–D4");
    for b in bots.iter().copied() {
        let mut line = format!("{:<7}", b.to_uppercase());
        for worn in [true, false] {
            let (mut mt, mut st, mut et, mut n) = (0u64, 0u64, 0u64, 0usize);
            for (fi, f) in forts.iter().enumerate().filter(|(_, f)| f.bot == b) {
                if let Some(si) = f.snaps.iter().position(|s| s.0 == 8) {
                    let p = find(fi, si, &|x| matches!(x, Wear::FromD1(w) if *w == worn)).unwrap();
                    mt += p.c[Stat::PickupMilliTicks as usize];
                    st += p.c[Stat::PickupSteps as usize];
                    et += p.c[Stat::EarlyTicks as usize];
                    n += p.sends;
                }
            }
            let n = n.max(1) as f64;
            line += &format!(" | party {}: {:.0} ticks ({:.1} steps) of {:.0} = {:.1}%", if worn { "worn" } else { "removed" }, mt as f64 / 1000.0 / n, st as f64 / n, et as f64 / n, 100.0 * mt as f64 / 1000.0 / et.max(1) as f64);
        }
        println!("{line}");
    }
    println!("\n## carry fetched on deaths: 0 % of lost carry (no fetch before Cut 119)");

    // best kind per wall
    println!("\n## best kind per wall (each kind alone in the party at the lineage's best pet level, {kind_sims} sims; both bots' snapshots): best (tags) score Δ vs none · margin over the 2nd in seed SDs of the paired difference");
    for w in WALLS {
        let snaps: Vec<(usize, usize)> = forts.iter().enumerate().filter_map(|(fi, f)| f.snaps.iter().position(|s| s.0 == w).map(|si| (fi, si))).collect();
        if snaps.is_empty() {
            println!("D{w}: —");
            continue;
        }
        let none: Vec<f64> = snaps.iter().map(|(fi, si)| score(&find(*fi, *si, &|x| matches!(x, Wear::Removed)).unwrap().v)).collect();
        let per: Vec<(String, Vec<f64>)> = kinds.iter().map(|k| (k.clone(), snaps.iter().map(|(fi, si)| score(&find(*fi, *si, &|x| matches!(x, Wear::Kind(y) if y == k)).unwrap().v)).collect())).collect();
        let mut ranked: Vec<(String, f64, &Vec<f64>)> = per.iter().map(|(k, v)| (k.clone(), mean_sd(v).0, v)).collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
        let (k1, m1, v1) = &ranked[0];
        let nm = mean_sd(&none).0;
        let margin = ranked.get(1).map(|(_, m2, v2)| {
            let diffs: Vec<f64> = v1.iter().zip(v2.iter()).map(|(a, b)| a - b).collect();
            let sd = mean_sd(&diffs).1;
            (m1 - m2) / if sd > 0.0 { sd } else { f64::NAN }
        });
        let tags = riddle_core::defs::monster_def(k1).tags.join(",");
        println!(
            "D{w} ({} snaps): best {k1} ({tags}) {:+.1} vs none · margin {:.2} SD | {}",
            snaps.len(),
            m1 - nm,
            margin.unwrap_or(f64::NAN),
            ranked.iter().map(|(k, m, _)| format!("{k} {:+.1}", m - nm)).collect::<Vec<_>>().join(" ")
        );
    }
    // (the bots may field no pet at all: what a pet does when one is given, from the kind panels)
    println!("\n## a pet given (the kind panels above, by wall): pet share of damage dealt · of blows drawn · acts per run per pet (row acts)");
    for k in &kinds {
        let mut line = format!("{k:<14}");
        let mut tot = [0u64; petstats::N];
        let mut tot_ps = 0usize;
        for w in WALLS {
            let mut c = [0u64; petstats::N];
            let mut ps = 0usize;
            for (i, (fi, si, wear, _)) in pj.iter().enumerate() {
                if matches!(wear, Wear::Kind(y) if y == k) && forts[*fi].snaps[*si].0 == w {
                    for x in 0..petstats::N {
                        c[x] += panels[i].c[x];
                        tot[x] += panels[i].c[x];
                    }
                    ps += panels[i].sends * panels[i].party;
                }
            }
            tot_ps += ps;
            if ps > 0 {
                line += &format!(" | D{w} {}", shares(&c, ps));
            }
        }
        println!("{line} | all {}", shares(&tot, tot_ps));
    }
    eprintln!("total {:.0}s", t0.elapsed().as_secs_f64());
}

fn shares(c: &[u64; petstats::N], pet_sends: usize) -> String {
    let dealt = c[Stat::HeroDealt as usize] + c[Stat::PetDealt as usize] + c[Stat::AllyDealt as usize];
    let blows = c[Stat::BlowsHero as usize] + c[Stat::BlowsPet as usize] + c[Stat::BlowsAlly as usize];
    format!(
        "dmg {:.1}% · blows {:.1}% · acts {:.1} ({:.1})",
        100.0 * c[Stat::PetDealt as usize] as f64 / dealt.max(1) as f64,
        100.0 * c[Stat::BlowsPet as usize] as f64 / blows.max(1) as f64,
        c[Stat::PetActs as usize] as f64 / pet_sends.max(1) as f64,
        c[Stat::PetRowActs as usize] as f64 / pet_sends.max(1) as f64
    )
}
