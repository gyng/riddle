//! Cut 24 measures on the cohort sets (`eval/cards/<build>.<rater>[-<tag>].rules.json`):
//!   stretch — each set's longest no-HP stretch per send (`Batch.nohp`: p50 / p99 / max), the
//!             sends a boss drove off, over `hours` offline per seed (§1);
//!   runs    — one set's sends one by one, from D1, with a kit (`--kit W A`): depth, tier, the
//!             longest stretch, a boss driven off, ticks (§1: AL's Warlord loop);
//!   reel    — distinct reel/summary lines per 10 runs on each set (§2).
//!   cargo run -q --profile fast -p riddle-core --example cut24 -- stretch [seeds] [hours] [filter…]
use riddle_core::{Game, RuleSet};
use std::collections::BTreeSet;

fn lineage_for(set: &RuleSet, seed: u64) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
    for u in ["row5", "row6", "row7", "row8", "throw", "cond_alert", "cond_turns", "cond_loot", "cond_on_kill", "cond_on_see"] {
        g.lineage.unlocks.insert(u.into());
    }
    for k in ["heal", "poison", "fire", "teleport", "blink"] {
        if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
            g.lineage.facts.insert(f);
        }
    }
    for f in ["foe:jackal:pack", "foe:bloat:gas", "foe:goblin_archer:ranged", "foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss", "foe:monkey:thief"] {
        g.lineage.facts.insert(f.into());
    }
    for r in &set.rows {
        if let Some(c) = r.card() {
            g.lineage.unlocks.insert(c.into());
        }
        for c in &r.conds {
            if let Some(u) = riddle_core::meta::cond_unlock(&c.k) {
                g.lineage.unlocks.insert(u.into());
            }
        }
    }
    g.set_rules_raw(set.clone()).unwrap_or_else(|e| panic!("{:?}: {e}", set.name));
    g
}

fn sets(wanted: &[String]) -> Vec<(String, RuleSet)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut sets: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".rules.json")?.to_string();
            if !wanted.is_empty() && !wanted.iter().any(|w| stem.contains(w.as_str())) {
                return None;
            }
            Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
        })
        .collect();
    sets.sort_by(|a, b| a.0.cmp(&b.0));
    sets
}

fn pct(v: &[u32], p: f64) -> u32 {
    if v.is_empty() {
        return 0;
    }
    let mut s = v.to_vec();
    s.sort();
    s[((s.len() as f64 - 1.0) * p).round() as usize]
}

fn stretch(seeds: u64, hours: u64, kit: (u32, u32), sets: &[(String, RuleSet)]) {
    let jobs: Vec<(usize, u64)> = (0..sets.len()).flat_map(|i| (1..=seeds).map(move |s| (i, s))).collect();
    let out: Vec<(usize, Vec<u32>, u32, u32, u32)> = std::thread::scope(|sc| {
        let hs: Vec<_> = jobs
            .chunks(jobs.len().div_ceil(std::thread::available_parallelism().map_or(8, |n| n.get())).max(1))
            .map(|ch| {
                sc.spawn(move || {
                    ch.iter()
                        .map(|&(i, seed)| {
                            let mut g = lineage_for(&sets[i].1, seed);
                            g.lineage.kit.insert("weapon".into(), kit.0);
                            g.lineage.kit.insert("armour".into(), kit.1);
                            riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
                            let b = &g.batch;
                            let deaths = b.run_outcomes.iter().filter(|(_, c)| c.is_some()).count() as u32;
                            (i, b.nohp.clone(), b.driven_off, deaths, b.stalls)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        hs.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    let mut all = Vec::new();
    for (i, (name, _)) in sets.iter().enumerate() {
        let v: Vec<u32> = out.iter().filter(|o| o.0 == i).flat_map(|o| o.1.clone()).collect();
        let driven: u32 = out.iter().filter(|o| o.0 == i).map(|o| o.2).sum();
        let deaths: u32 = out.iter().filter(|o| o.0 == i).map(|o| o.3).sum();
        let stalls: u32 = out.iter().filter(|o| o.0 == i).map(|o| o.4).sum();
        println!("{name:<34} sends {:>4} · p50 {:>3} p90 {:>3} p99 {:>4} max {:>5} · >60 {:>3} · driven {driven:>3} · deaths {deaths:>3} · stalls {stalls}", v.len(), pct(&v, 0.5), pct(&v, 0.9), pct(&v, 0.99), v.iter().max().unwrap_or(&0), v.iter().filter(|&&x| x > 60).count());
        all.extend(v);
    }
    println!("ALL sends {} · p99 {} · max {}", all.len(), pct(&all, 0.99), all.iter().max().unwrap_or(&0));
}

fn runs(seed: u64, n: u32, kit: (u32, u32), set: &RuleSet, verbose: bool) {
    let mut g = lineage_for(set, seed);
    g.lineage.kit.insert("weapon".into(), kit.0);
    g.lineage.kit.insert("armour".into(), kit.1);
    if let Some(p) = g.lineage.classes.get_mut("fighter") {
        p.level = 3;
    }
    for i in 0..n {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let mut boss_t: Option<u32> = None;
        let mut last: Option<(i32, u32)> = None;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) {
            g.tick();
            g.events.clear();
            let r = g.run.as_ref().unwrap();
            if boss_t.is_none() && r.boss_seen_t.is_some() {
                boss_t = r.boss_seen_t;
            }
            if verbose {
                if let Some(b) = r.monsters.iter().find(|m| m.is_boss() && m.hp > 0) {
                    if boss_t.is_some() && last.is_none_or(|l| l.0 != b.hp) {
                        println!("   t{} a{} boss hp {} (hero {}/{}) still {:?} nohp {:?}", r.turn, r.actions, b.hp, r.hero.hp, r.hero.max_hp, r.boss_still, r.nohp);
                        last = Some((b.hp, r.actions));
                    }
                }
            }
        }
        let r = g.run.as_ref().unwrap();
        println!(
            "run {i:>2}: D{:<2} {:<7} {:<14} hp {:>2}/{:<2} ticks {:>6} boss@{:<6} nohp max {:>4} driven {:?}",
            r.max_depth,
            format!("{:?}", r.over.unwrap()),
            r.death_cause.clone().unwrap_or_default(),
            r.hero.hp,
            r.hero.max_hp,
            r.turn,
            boss_t.map(|t| t.to_string()).unwrap_or("-".into()),
            r.nohp.2,
            r.driven_off
        );
        g.finish_run();
        g.auto_keep();
    }
}

/// The moments a stretch passes 60: the set's sends one by one, the last 16 actions and the
/// foes in view at the crossing.
fn why(seed: u64, n: u32, set: &RuleSet, at: u32) {
    let mut g = lineage_for(set, seed);
    for i in 0..n {
        g.lineage.rest_left = 0;
        g.start_run(None);
        let mut said = false;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) {
            g.tick();
            g.events.clear();
            let r = g.run.as_ref().unwrap();
            let s = r.nohp.0.max(r.boss_still.map_or(0, |b| b.2));
            if s >= at && !said {
                said = true;
                println!("run {i} D{} t{} a{} hp {}/{} nohp {:?} still {:?}", r.depth, r.turn, r.actions, r.hero.hp, r.hero.max_hp, r.nohp, r.boss_still);
                for t in &r.trace {
                    println!("   t{} R{} {} hp {} foes {}", t.t, t.row + 1, t.verb.short(), t.hp, t.foes);
                }
                let hp = r.hero.pos;
                for m in r.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && r.floor.map.is_visible(m.pos)) {
                    println!("   foe {} #{} hp {}/{} d{} summoned {} dormant {}", m.kind, m.id, m.hp, m.max_hp, m.pos.cheb(hp), m.summoned, m.dormant);
                }
            }
            if s < at {
                said = false;
            }
        }
        g.finish_run();
        g.auto_keep();
    }
}

/// The FULL bot (every unlock and fact, fighter L10 mastered, every kit step): each boss
/// fight's longest still stretch (the boss in view, its HP unmoved) and how it ended.
fn boss(seeds: u64, n: u32, drop: Option<&str>) {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/presets/full.json")).unwrap();
    let mut set = RuleSet::parse(&text).unwrap();
    if let Some(d) = drop {
        set.rows.retain(|r| r.verb.a.as_deref() != Some(d));
    }
    let mut stats: std::collections::BTreeMap<String, Vec<(u32, bool)>> = Default::default();
    let mut nohp_all = Vec::new();
    for seed in 1..=seeds {
        let mut g = Game::new(seed);
        g.max_deaths = 100_000;
        for u in riddle_core::meta::UNLOCKS {
            g.lineage.unlocks.insert(u.id.into());
        }
        riddle_core::probes::learn_everything(&mut g);
        g.lineage.classes.insert("fighter".into(), riddle_core::wire::ClassProg { level: 10, xp: 0, next: 0 });
        g.lineage.unlocks.insert(riddle_core::hero::mastery_card(riddle_core::hero::Class::Fighter).into());
        riddle_core::kit::buy_all(&mut g.lineage);
        g.set_rules(set.clone()).unwrap();
        for _ in 0..n {
            g.lineage.rest_left = 0;
            g.start_run(None);
            let mut cur: Option<(u32, String, u32)> = None;
            let mut done: Vec<(u32, String, u32)> = Vec::new();
            let mut shown: Vec<u32> = Vec::new();
            while g.run.as_ref().is_some_and(|r| r.over.is_none()) {
                g.tick();
                g.events.clear();
                let r = g.run.as_ref().unwrap();
                if let Some((id, _, n)) = r.boss_still {
                    if n >= 61 && std::env::var("SHOW").is_ok() && !shown.contains(&id) {
                        shown.push(id);
                        println!("seed {seed} D{} a{} hp {}/{} still {:?} nohp {:?}", r.depth, r.actions, r.hero.hp, r.hero.max_hp, r.boss_still, r.nohp);
                        for t in &r.trace {
                            println!("   t{} R{} {} hp {} foes {}", t.t, t.row + 1, t.verb.short(), t.hp, t.foes);
                        }
                        let hp = r.hero.pos;
                        for m in r.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && r.floor.map.is_visible(m.pos)) {
                            println!("   foe {} #{} hp {}/{} d{} summoned {}", m.kind, m.id, m.hp, m.max_hp, m.pos.cheb(hp), m.summoned);
                        }
                    }
                    let kind = r.monsters.iter().find(|m| m.id == id).map(|m| m.kind.clone()).unwrap_or_default();
                    match cur.as_mut() {
                        Some(c) if c.0 == id => c.2 = c.2.max(n),
                        _ => {
                            if let Some(c) = cur.take() {
                                done.push(c);
                            }
                            cur = Some((id, kind, n));
                        }
                    }
                }
            }
            done.extend(cur);
            let r = g.run.as_ref().unwrap();
            nohp_all.push(r.nohp.2);
            for (id, kind, n) in done {
                let killed = r.monsters.iter().find(|m| m.id == id).is_none_or(|m| m.hp <= 0);
                stats.entry(kind).or_default().push((n, killed));
            }
            g.finish_run();
            g.auto_keep();
        }
    }
    for (k, v) in &stats {
        let ns: Vec<u32> = v.iter().map(|x| x.0).collect();
        let kills = v.iter().filter(|x| x.1).count();
        let kn: Vec<u32> = v.iter().filter(|x| x.1).map(|x| x.0).collect();
        println!("{k:<16} fights {:>3} · killed {kills:>3} · still p50 {:>3} p90 {:>3} max {:>4} · in kills max {:>4} · >60 {}", v.len(), pct(&ns, 0.5), pct(&ns, 0.9), ns.iter().max().unwrap_or(&0), kn.iter().max().unwrap_or(&0), ns.iter().filter(|&&x| x > 60).count());
    }
    println!("runs {} nohp p99 {} max {}", nohp_all.len(), pct(&nohp_all, 0.99), nohp_all.iter().max().unwrap_or(&0));
}

/// §2: a watched run's report reel is its last five chronicle notes (web `watch.ts`), led by the
/// exit's `news` (Cut 24): distinct lines (digits folded) per 10 runs, per set, sends from D1.
fn reel(seeds: u64, runs_n: usize, sets: &[(String, RuleSet)]) {
    let mut tot = Vec::new();
    for (name, set) in sets {
        let mut per10 = Vec::new();
        let mut per10_raw = Vec::new();
        for seed in 1..=seeds {
            let mut g = lineage_for(set, seed);
            let mut lines: Vec<String> = Vec::new();
            for _ in 0..runs_n {
                g.lineage.rest_left = 0;
                g.start_run(None);
                let mut notes: Vec<String> = Vec::new();
                let mut news: Vec<String> = Vec::new();
                while g.run.as_ref().is_some_and(|r| r.over.is_none()) {
                    g.tick();
                    for e in &g.events {
                        if let riddle_core::Ev::Note { text, .. } = e {
                            notes.push(text.clone());
                        }
                    }
                    g.events.clear();
                }
                g.finish_run();
                for e in &g.events {
                    if let riddle_core::Ev::Note { text, .. } = e {
                        notes.push(text.clone());
                    }
                }
                if let Some(l) = g.last_exit.take() {
                    news.extend(exit_news(&l));
                }
                g.events.clear();
                g.auto_keep();
                let n = notes.len();
                if std::env::var("SHOW").is_ok() {
                    println!("  news {:?} · notes {:?}", news, &notes[n.saturating_sub(5)..]);
                }
                lines.extend(news);
                lines.extend(notes[n.saturating_sub(5)..].iter().cloned());
            }
            let fold = |s: &String| s.chars().map(|c| if c.is_ascii_digit() { '#' } else { c }).collect::<String>();
            let d: BTreeSet<String> = lines.iter().map(fold).collect();
            let raw: BTreeSet<&String> = lines.iter().collect();
            per10.push(d.len() as f64 * 10.0 / runs_n as f64);
            per10_raw.push(raw.len() as f64 * 10.0 / runs_n as f64);
        }
        let m = per10.iter().sum::<f64>() / per10.len() as f64;
        println!("{name:<34} distinct reel lines / 10 runs: {m:.1} (digits folded) · {:.1} raw", per10_raw.iter().sum::<f64>() / per10_raw.len() as f64);
        tot.push(m);
    }
    println!("MEAN distinct / 10 runs (folded): {:.2}", tot.iter().sum::<f64>() / tot.len().max(1) as f64);
}

/// The stalls of a set over `hours` offline per seed: each stall's cause and trace.
fn stalls(seeds: u64, hours: u64, set: &RuleSet) {
    for seed in 1..=seeds {
        let mut g = lineage_for(set, seed);
        riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        for (id, r) in g.deaths.iter().filter(|(_, r)| r.stall) {
            println!("seed {seed} run {id}: D{} {} · {}", r.death.depth, r.death.cause, r.death.margin);
            for t in r.death.trace.turns.iter().rev().take(14).rev() {
                println!("   t{} R{} {} hp {} foes {} {:?}", t.t, t.row + 1, t.verb.short(), t.hp, t.foes, t.blocked);
            }
            if let Some(run) = &r.t10 {
                let hp = run.hero.pos;
                println!("   at t10 t{} hero ({},{}) stairs_down ({},{}) boss_still {:?} stuck_fires {}", run.turn, hp.x, hp.y, run.floor.stairs_down.x, run.floor.stairs_down.y, run.boss_still, run.stuck_fires);
                for m in run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && m.pos.cheb(hp) <= 8) {
                    println!("   foe {} #{} hp {}/{} at ({},{}) d{} awake {} dormant {} summoned {} vis {}", m.kind, m.id, m.hp, m.max_hp, m.pos.x, m.pos.y, m.pos.cheb(hp), m.awake, m.dormant, m.summoned, run.floor.map.is_visible(m.pos));
                }
            }
        }
    }
}

/// Deaths by cause and depth, and exits by tier, for a set over `hours` offline per seed.
fn causes(seeds: u64, hours: u64, set: &RuleSet) {
    let mut by: std::collections::BTreeMap<String, u32> = Default::default();
    let (mut sends, mut driven) = (0, 0);
    let off: u64 = std::env::var("SEED_OFF").ok().and_then(|s| s.parse().ok()).unwrap_or(0);
    for seed in (1 + off)..=(seeds + off) {
        let mut g = lineage_for(set, seed);
        riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        sends += g.batch.run_outcomes.len();
        driven += g.batch.driven_off;
        for (d, c) in &g.batch.run_outcomes {
            if let Some(c) = c {
                *by.entry(format!("{c} D{d}")).or_default() += 1;
            }
        }
    }
    let mut v: Vec<_> = by.into_iter().collect();
    v.sort_by_key(|b| std::cmp::Reverse(b.1));
    println!("sends {sends} deaths {} driven {driven} · {}", v.iter().map(|x| x.1).sum::<u32>(), v.iter().take(14).map(|(k, n)| format!("{k} ×{n}")).collect::<Vec<_>>().join(" · "));
}

/// The COUNTERED bot (metrics' setup) over `hours` offline per seed: best depths, drive-offs
/// by boss, stalls.
fn countered(seeds: u64, hours: u64) {
    let mut best = Vec::new();
    let mut by: std::collections::BTreeMap<String, u32> = Default::default();
    for seed in 1..=seeds {
        let mut g = Game::new(seed);
        g.max_deaths = 100_000;
        for u in ["row5", "row6", "row7", "row8", "tame", "throw"] {
            g.lineage.unlocks.insert(u.into());
        }
        g.lineage.facts.insert("item:leash".into());
        for k in ["fire", "poison"] {
            if let Some(f) = riddle_core::item::ident_fact(&g.lineage.flavours, k) {
                g.lineage.facts.insert(f);
            }
        }
        for f in ["foe:goblin_warlord:boss", "foe:bloat_mother:boss", "foe:lich:boss", "foe:skeleton:summoned"] {
            g.lineage.facts.insert(f.into());
        }
        g.set_rules_raw(riddle_core::probes::countered()).unwrap();
        riddle_core::offline::run_offline_quick(&mut g, hours * 3600);
        for e in &g.batch.exits {
            if let Some(d) = &e.driven {
                *by.entry(format!("{} D{}", d.boss, d.depth)).or_default() += 1;
            }
        }
        best.push(g.lineage.best_depth);
    }
    println!("best {:?} · ≥D14 {}/{} · driven {:?}", best, best.iter().filter(|&&b| b >= 14).count(), best.len(), by);
}

fn exit_news(l: &riddle_core::ExitLine) -> Vec<String> {
    l.news.iter().map(|n| n.text.clone()).collect()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kit = args.iter().position(|a| a == "--kit").map(|i| (args[i + 1].parse().unwrap(), args[i + 2].parse().unwrap())).unwrap_or((0, 0));
    let pos: Vec<String> = {
        let mut v = Vec::new();
        let mut skip = 0;
        for a in &args {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            if a == "-v" {
                continue;
            }
            if a == "--kit" {
                skip = 2;
                continue;
            }
            v.push(a.clone());
        }
        v
    };
    let mode = pos.first().map(String::as_str).unwrap_or("stretch");
    let num = |i: usize, d: u64| pos.get(i).and_then(|s| s.parse().ok()).unwrap_or(d);
    match mode {
        "stretch" => {
            let (seeds, hours) = (num(1, 8), num(2, 4));
            let filt: Vec<String> = pos.iter().skip(3).cloned().collect();
            stretch(seeds, hours, kit, &sets(&filt));
        }
        "runs" => {
            let (seed, n) = (num(1, 1), num(2, 10) as u32);
            let filt: Vec<String> = pos.iter().skip(3).cloned().collect();
            let s = sets(&filt);
            runs(seed, n, kit, &s[0].1, args.iter().any(|a| a == "-v"));
        }
        "why" => {
            let (seed, n, at) = (num(1, 1), num(2, 20) as u32, num(3, 61) as u32);
            let filt: Vec<String> = pos.iter().skip(4).cloned().collect();
            why(seed, n, &sets(&filt)[0].1, at);
        }
        "boss" => {
            let (seeds, n) = (num(1, 4), num(2, 10) as u32);
            boss(seeds, n, pos.get(3).map(String::as_str));
        }
        "stalls" => {
            let (seeds, hours) = (num(1, 8), num(2, 4));
            let filt: Vec<String> = pos.iter().skip(3).cloned().collect();
            stalls(seeds, hours, &sets(&filt)[0].1);
        }
        "causes" => {
            let (seeds, hours) = (num(1, 8), num(2, 4));
            let filt: Vec<String> = pos.iter().skip(3).cloned().collect();
            causes(seeds, hours, &sets(&filt)[0].1);
        }
        "countered" => countered(num(1, 8), num(2, 8)),
        "reel" => {
            let (seeds, n) = (num(1, 4), num(2, 10) as usize);
            let filt: Vec<String> = pos.iter().skip(3).cloned().collect();
            reel(seeds, n, &sets(&filt));
        }
        _ => eprintln!("modes: stretch | runs | reel"),
    }
}
