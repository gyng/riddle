//! Fourteen simulated days of a plausible player (docs/CUT2.md): check in N times a day, read
//! the worst death, apply its top patch when it clearly beats the baseline, insert the boss
//! counter row once a wall has held two days and the counter fact is known, buy the cheapest
//! affordable unlock, field the kennel. Prints a per-day table per seed and the Cut 2 probes;
//! `--gate` checks the bars and exits non-zero on a failed one (never weaken a bar).
//!   cargo run -q --profile fast -p riddle-core --example dayplayer -- [--seeds 3] [--days 14] [--checkins 3] [--gate]
use riddle_core::rules::{Cond, Row, Verb};
use riddle_core::Game;

#[derive(Default, Clone)]
struct Day {
    best: u32,
    unlocks: u32,
    edits: u32,
    empty: u32,
    checkins: u32,
    runs: u32,
    banked: u32,
    deaths: u32,
    marks: u32,
    /// Marks unspent at the worst check-in of the day (after the day's decisions).
    marks_max: u32,
    gold: i32,
    facts: usize,
    level: u32,
    rank: u32,
    rows: usize,
    kennel: usize,
    bones: usize,
    xp: u32,
    counter: bool,
    /// Cut 3: the lineage ascended this day (the second act starts).
    ascended: bool,
}

struct SeedOut {
    table: Vec<Day>,
    /// First day the class reached L10 (1-based), if any.
    l10_day: Option<usize>,
    /// Longest run of days without a new best depth while the wall's counter was known (or
    /// the stall was not at a boss wall at all).
    stall: usize,
    rules: String,
    /// Cut 3: ascension level at the end of the fortnight.
    ascension: u32,
}

fn boss_at(depth: u32) -> Option<&'static str> {
    riddle_core::descent::boss_for(depth)
}

/// The row a human who read the boss fact would add (fighter vocabulary). Cut 3: the Foundry
/// Master wants the smiths first and no melee on him (`reflect_read`), the Lurker Queen a
/// silence read on sight, the Mirror King the `cadence` card.
fn counter_rows(boss: &str) -> Vec<Row> {
    match boss {
        "goblin_warlord" => vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))],
        "bloat_mother" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 6)], Verb::arg("throw", "fire,tag:boss"))],
        "lich" => vec![
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:summoned")),
            Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss")),
        ],
        "foundry_master" => vec![
            Row::new(vec![Cond::t("foe_tag", "reflect_melee")], Verb::arg("tactic", "reflect_read")),
            Row::new(vec![Cond::t("foe_tag", "buffer"), Cond::n("depth>=", 16)], Verb::arg("attack", "tag:buffer")),
        ],
        "lurker_queen" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 25)], Verb::arg("read", "silence"))],
        "mirror_king" => vec![Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 30)], Verb::arg("tactic", "cadence"))],
        _ => vec![Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))],
    }
}

/// The unlock a counter row needs (a tactic card, or `throw`), if any.
fn row_unlock(row: &Row) -> Option<String> {
    match (row.verb.v.as_str(), row.verb.a.as_deref()) {
        ("tactic", Some(card)) => Some(card.to_string()),
        ("throw", _) => Some("throw".into()),
        _ => None,
    }
}

fn insert_row(g: &mut Game, row: Row, at: usize) -> bool {
    let max_rows = g.vocabulary().max_rows;
    let mut rules = g.lineage.rules().clone();
    if rules.rows.contains(&row) {
        return false;
    }
    if rules.rows.len() >= max_rows {
        // Make room from the bottom, but never throw away the plain attack row (the set's spine).
        let plain = |r: &Row| r.verb.v == "attack" && !r.verb.a.as_deref().is_some_and(|a| a.starts_with("tag:"));
        let drop = (0..rules.rows.len()).rev().find(|&i| !plain(&rules.rows[i])).unwrap_or(rules.rows.len() - 1);
        rules.rows.remove(drop);
    }
    let at = at.min(rules.rows.len());
    rules.rows.insert(at, row);
    g.set_rules(rules).is_ok()
}

fn play(seed: u64, days: usize, checkins: u64, verbose: bool) -> SeedOut {
    let interval = 24 * 3600 / checkins;
    let mut g = Game::new(seed);
    let mut table: Vec<Day> = Vec::new();
    let mut l10_day = None;
    let mut last_best = 0u32;
    let mut stalled_days = 0usize; // days at the current best
    let mut counters_done: Vec<String> = Vec::new();
    let mut stall_best = 0usize;
    let mut stall_cur = 0usize;
    for day in 0..days {
        let mut d = Day::default();
        for _ in 0..checkins {
            let rep = riddle_core::offline::run_offline_quick(&mut g, interval);
            d.checkins += 1;
            d.runs += rep.runs;
            d.banked += rep.banked + rep.returned;
            d.deaths += rep.deaths.iter().map(|x| x.n).sum::<u32>();
            d.xp += rep.xp.gained;
            let mut decided = false;
            // 0. Cut 3: the ending reached — ascend with `no_rest` and keep counting. The
            //    descent, marks and unlocks start over; the rules stay.
            if g.lineage.ended {
                let variant = "no_rest";
                if g.ascend(variant).is_ok() {
                    if verbose {
                        eprintln!("  day {} ascended ({variant}); unlocks kept {:?}", day + 1, g.lineage.unlocks);
                    }
                    d.ascended = true;
                    decided = true;
                    last_best = 0;
                    stalled_days = 0;
                    stall_cur = 0;
                    counters_done.clear();
                }
            }
            // 1. The worst death: patch if the top candidate clearly beats the baseline.
            if let Some(id) = rep.worst_death_id {
                if let Some(death) = g.death(id) {
                    if death.verdict == "gap" {
                        if let Some(p) = death.patches.first() {
                            if p.survive > death.baseline + 0.15 && insert_row(&mut g, p.row.clone(), p.insert_at) {
                                d.edits += 1;
                                decided = true;
                            }
                        }
                    }
                }
            }
            // 2. A wall held two days and the counter is known: a human who read the fact
            //    inserts the counter row (once per boss).
            let best = g.lineage.best_depth;
            if let Some(boss) = boss_at(best) {
                let fact = format!("boss:{boss}:counter");
                if stalled_days >= 2 && g.lineage.facts.contains(&fact) && !counters_done.contains(&boss.to_string()) {
                    let rows = counter_rows(boss);
                    // The editor offers `throw fire` and lets the player aim it; `attack tag:T` needs the tag known.
                    let usable_now = |g: &Game| {
                        let vocab = g.vocabulary();
                        let has_verb = |r: &Row| vocab.verbs.iter().any(|v| v.v == r.verb.v && v.a.as_deref().map(|a| a.split(',').next().unwrap_or(a)) == r.verb.a.as_deref().map(|a| a.split(',').next().unwrap_or(a)));
                        rows.iter().all(|r| has_verb(r) && r.conds.iter().all(|c| vocab.conds.iter().any(|v| v.k == c.k && v.t == c.t)))
                    };
                    // A counter that needs an unlock (a card, `throw`) is bought before anything else.
                    let missing: Vec<String> = rows.iter().filter_map(row_unlock).filter(|u| !g.lineage.unlocks.contains(u)).collect();
                    for u in &missing {
                        if g.buy(u).is_ok() {
                            d.unlocks += 1;
                            decided = true;
                        }
                    }
                    let usable = usable_now(&g);
                    if usable {
                        for (i, r) in rows.into_iter().enumerate() {
                            insert_row(&mut g, r, i);
                        }
                        counters_done.push(boss.to_string());
                        d.edits += 1;
                        d.counter = true;
                        decided = true;
                    } else if boss == "lurker_queen" {
                        // Silence not yet identified: no rest on her floor (`noise_discipline`)
                        // does what a human who read the fact would do; the identified scroll
                        // row goes in when the flavour is known.
                        let rest = g.lineage.rules().rows.iter().position(|r| r.verb.v == "rest");
                        if g.lineage.unlocks.contains("noise_discipline") || g.buy("noise_discipline").is_ok() {
                            let row = Row::new(vec![Cond::n("hp<", 90)], Verb::arg("tactic", "noise_discipline"));
                            let mut rules = g.lineage.rules().clone();
                            if let Some(i) = rest {
                                rules.rows[i] = row;
                                if g.set_rules(rules).is_ok() {
                                    d.edits += 1;
                                    decided = true;
                                }
                            } else if insert_row(&mut g, row, usize::MAX) {
                                d.edits += 1;
                                decided = true;
                            }
                        }
                    } else if boss == "bloat_mother" && g.lineage.unlocks.contains("throw") {
                        // `throw fire` needs fire identified: meanwhile throw whatever is in the
                        // pack at her (the throw identifies it).
                        let row = Row::new(vec![Cond::t("foe_tag", "boss"), Cond::n("depth>=", 6)], Verb::arg("throw", "unknown,tag:boss"));
                        if !g.lineage.rules().rows.contains(&row) && insert_row(&mut g, row, 0) {
                            d.edits += 1;
                            decided = true;
                        }
                    }
                }
            }
            // 2b. Stalled two days anywhere (a wall's counter, if any, already in): a human who
            //     reads "60 runs · 60 returned · best D9" first tells the heir to rest between
            //     fights, then returns later (threshold −10, floor 20), then moves the escape
            //     row under the fighting rows.
            if stalled_days >= 2 && boss_at(best).is_none_or(|b| counters_done.contains(&b.to_string())) {
                let mut rules = g.lineage.rules().clone();
                let has_rest = rules.rows.iter().any(|r| r.verb.v == "rest");
                if !has_rest && rep.deaths.is_empty() {
                    let at = rules.rows.len();
                    if insert_row(&mut g, Row::new(vec![Cond::n("hp<", 70)], Verb::new("rest")), at) {
                        d.edits += 1;
                        decided = true;
                        stalled_days = 0;
                    }
                } else if let Some(i) = rules.rows.iter().position(|r| r.verb.v == "return" && r.conds.iter().any(|c| c.k == "hp<")) {
                    let last = rules.rows.len() - 1;
                    let c = rules.rows[i].conds.iter_mut().find(|c| c.k == "hp<").unwrap();
                    let n = c.n.unwrap_or(30);
                    if n > 20 {
                        c.n = Some(n - 10);
                    } else if i < last {
                        let r = rules.rows.remove(i);
                        rules.rows.push(r);
                    }
                    if rules != *g.lineage.rules() && g.set_rules(rules).is_ok() {
                        d.edits += 1;
                        decided = true;
                        stalled_days = 0;
                    }
                }
            }
            // 3. The cheapest affordable unlock; when marks pile up past a reserve of 8 the
            //    visit keeps spending, cheapest first, until they do not.
            let mut bought = 0;
            loop {
                if bought > 0 && g.lineage.marks <= 8 {
                    break;
                }
                let mut opts: Vec<_> = g.unlocks().into_iter().filter(|u| !u.owned && u.available && u.cost <= g.lineage.marks).collect();
                opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                let Some(u) = opts.first() else { break };
                if g.buy(&u.id).is_err() {
                    break;
                }
                if verbose {
                    eprintln!("  day {} buy {} ({}) marks left {}", day + 1, u.id, u.cost, g.lineage.marks);
                }
                bought += 1;
                d.unlocks += 1;
                decided = true;
            }
            if verbose && g.lineage.marks > 8 {
                let gated: Vec<String> = g.unlocks().into_iter().filter(|u| !u.owned).map(|u| format!("{}:{}{}", u.id, u.cost, u.needs.as_ref().map(|n| format!("[{n}]")).unwrap_or_default())).collect();
                eprintln!("  day {} marks {} unspent; catalogue: {}", day + 1, g.lineage.marks, gated.join(" "));
            }
            // 3b. Bring the vault (a human sends the heir out with what it has).
            let ids: Vec<u32> = g.lineage.vault.iter().map(|i| i.id).collect();
            g.loadout(ids);
            // 4. Field the kennel.
            let slots = g.lineage.party_slots() as usize;
            if g.lineage.party.len() < slots && !g.lineage.kennel.is_empty() {
                let mut ids: Vec<u32> = g.lineage.party.iter().map(|c| c.id).collect();
                let mut k: Vec<_> = g.lineage.kennel.iter().filter(|c| !ids.contains(&c.id)).collect();
                k.sort_by_key(|c| std::cmp::Reverse(c.level));
                for c in k.iter().take(slots - ids.len()) {
                    ids.push(c.id);
                }
                if g.set_party(ids).is_ok() {
                    decided = true;
                }
            }
            if !decided && rep.learned.is_empty() && rep.pending.is_empty() {
                d.empty += 1;
            }
            if verbose && g.lineage.best_depth > last_best {
                eprintln!("  day {} new best D{} L{} unlocks {:?} bests {:?}", day + 1, g.lineage.best_depth, g.lineage.class_level(), g.lineage.unlocks, rep.bests);
                eprintln!("    rules {}", g.export_rules().split_whitespace().collect::<Vec<_>>().join(" "));
            }
            d.marks_max = d.marks_max.max(g.lineage.marks);
        }
        d.best = g.lineage.best_depth;
        d.marks = g.lineage.marks;
        d.gold = g.lineage.gold;
        d.facts = g.lineage.facts.len();
        d.level = g.lineage.classes.get(g.lineage.class.name()).map(|c| c.level).unwrap_or(1);
        d.rank = g.lineage.rank;
        d.rows = g.lineage.rules().rows.len();
        d.kennel = g.lineage.kennel.len() + g.lineage.party.len();
        d.bones = g.lineage.bones.len();
        if d.level >= 10 && l10_day.is_none() {
            l10_day = Some(day + 1);
        }
        // Stall bookkeeping: a wall whose counter is still unknown is not the player's fault,
        // and the bottom (D31) is the ending, not a stall.
        if d.best >= riddle_core::descent::ENDING_DEPTH || d.best > last_best || d.ascended {
            last_best = d.best;
            stalled_days = 0;
            stall_cur = 0;
        } else {
            stalled_days += 1;
            let counts = match boss_at(d.best) {
                Some(boss) => g.lineage.facts.contains(&format!("boss:{boss}:counter")),
                None => true,
            };
            if counts {
                stall_cur += 1;
                stall_best = stall_best.max(stall_cur);
            } else {
                stall_cur = 0;
            }
        }
        table.push(d);
    }
    SeedOut { table, l10_day, stall: stall_best, rules: g.export_rules().replace('\n', " ").split_whitespace().collect::<Vec<_>>().join(" "), ascension: g.lineage.ascension }
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let get = |k: &str, d: u64| a.iter().position(|x| x == k).and_then(|i| a.get(i + 1)).and_then(|s| s.parse().ok()).unwrap_or(d);
    let seeds = get("--seeds", 3);
    let days = get("--days", 14) as usize;
    let checkins = get("--checkins", 3);
    let gate = a.iter().any(|x| x == "--gate");
    let t0 = std::time::Instant::now();
    let verbose = a.iter().any(|x| x == "--verbose");
    // Seeds in parallel (each is ~4 min of simulation).
    let outs: Vec<SeedOut> = std::thread::scope(|sc| {
        let hs: Vec<_> = (1..=seeds).map(|s| sc.spawn(move || play(s, days, checkins, verbose))).collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (i, o) in outs.iter().enumerate() {
        println!("seed {}: ascension {} · rules now = {}", i + 1, o.ascension, o.rules);
        println!("day  best  unl  edit  empty/ci  runs  bank  deaths  marks  max  gold  facts  L  rank  rows  pets  bones  xp");
        for (i, d) in o.table.iter().enumerate() {
            println!(
                "{:>3}  {:>4}  {:>3}  {:>3}{}  {:>4}/{:<3}  {:>4}  {:>4}  {:>6}  {:>5}  {:>3}  {:>4}  {:>5}  {:>1}  {:>4}  {:>4}  {:>4}  {:>5}  {:>4}",
                i + 1,
                d.best,
                d.unlocks,
                d.edits,
                if d.ascended {
                    "^"
                } else if d.counter {
                    "*"
                } else {
                    " "
                },
                d.empty,
                d.checkins,
                d.runs,
                d.banked,
                d.deaths,
                d.marks,
                d.marks_max,
                d.gold,
                d.facts,
                d.level,
                d.rank,
                d.rows,
                d.kennel,
                d.bones,
                d.xp
            );
        }
    }
    // Probes and bars (docs/CUT2.md).
    let n = outs.len() as f64;
    let unlock_days: Vec<usize> = outs.iter().map(|o| o.table.iter().filter(|d| d.unlocks > 0).count()).collect();
    let unlock_mean = unlock_days.iter().sum::<usize>() as f64 / n;
    let empty: u32 = outs.iter().map(|o| o.table.iter().map(|d| d.empty).sum::<u32>()).sum();
    let cis: u32 = outs.iter().map(|o| o.table.iter().map(|d| d.checkins).sum::<u32>()).sum();
    let empty_pct = 100.0 * empty as f64 / cis.max(1) as f64;
    let marks_max = outs.iter().map(|o| o.table.iter().skip(2).map(|d| d.marks_max).max().unwrap_or(0)).max().unwrap_or(0);
    let l10_min = outs.iter().filter_map(|o| o.l10_day).min();
    let stall = outs.iter().map(|o| o.stall).max().unwrap_or(0);
    let final_best: Vec<u32> = outs.iter().map(|o| o.table.last().map(|d| d.best).unwrap_or(0)).collect();
    let runs_day: f64 = outs.iter().map(|o| o.table.iter().map(|d| d.runs as f64).sum::<f64>() / days as f64).sum::<f64>() / n;
    println!("\nprobes over {seeds} seeds × {days} days × {checkins}/day  ({:.0}s)", t0.elapsed().as_secs_f64());
    let asc: Vec<u32> = outs.iter().map(|o| o.ascension).collect();
    println!("  final best depth per seed       {final_best:?}   ending at {} · ascensions {asc:?}", riddle_core::descent::ENDING_DEPTH);
    println!("  expeditions per day (mean)      {runs_day:.1}");
    println!("  days with ≥1 unlock per seed    {unlock_days:?}");
    let bars: Vec<(String, String, bool)> = vec![
        (format!("Days with ≥ 1 unlock ≥ 10 / {days} (mean)"), format!("{unlock_mean:.1}"), unlock_mean >= 10.0),
        ("Marks unspent at any check-in after day 2 ≤ 8".into(), format!("{marks_max}"), marks_max <= 8),
        ("Empty check-ins ≤ 15%".into(), format!("{empty_pct:.0}%"), empty_pct <= 15.0),
        ("Class L10 not before day 7".into(), l10_min.map(|d| format!("day {d}")).unwrap_or_else(|| "never".into()), l10_min.is_none_or(|d| d >= 7)),
        ("Longest best-depth stall (counter known) ≤ 3 days".into(), format!("{stall}"), stall <= 3),
    ];
    println!();
    println!("{:<52} {:>10}  result", "bar", "value");
    let mut fails = 0;
    for (name, value, ok) in &bars {
        println!("{:<52} {:>10}  {}", name, value, if *ok { "PASS" } else { "FAIL" });
        if !ok {
            fails += 1;
        }
    }
    println!("dayplayer: {}", if fails > 0 { "FAIL" } else { "all PASS" });
    if gate && fails > 0 {
        eprintln!("{fails} bar(s) FAIL");
        std::process::exit(1);
    }
}
