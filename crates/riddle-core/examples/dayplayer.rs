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
const MILESTONES: [u32; 6] = [8, 13, 18, 23, 28, 33];
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

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
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
    /// Sends and those that stalled (the stall guard's timeout) over the fortnight.
    #[serde(default)]
    sends: u32,
    #[serde(default)]
    stalled: u32,
    /// Deaths, quests kept, and the best depth summed over the days (the climb's area).
    #[serde(default)]
    deaths: u32,
    #[serde(default)]
    quests: u32,
    #[serde(default)]
    depth_area: u32,
    /// PROGRESSION_V2 (reported, gated in Cut 31): systems open at day 1's end, the most systems and
    /// beats one report brought, days with something new, the longest run of days without.
    day1_systems: usize,
    max_systems: usize,
    max_beats: usize,
    new_days: usize,
    new_gap: usize,
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
    // (a swap must not cost the walk: the sends from where they start keep their floors — the wall's
    // answer is taken, not a stance that reads well at the wall and dies on the way to it)
    let Some(top) = opts.iter().find(|o| o.action == "equip" && o.d_mean >= -0.25 && o.d_death <= 0.05) else { return moved };
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
const PICK_SIMS: u32 = 40;
const PICK_BAR: f64 = 0.12;

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
    let mut last_key: Option<(usize, u32, Vec<String>)> = None;
    let mut cool = 0u32;
    let mut last_wall: Option<usize> = None;
    let mut gap = 0usize;
    for day in 0..days {
        let wealth0 = g.lineage.gold as i64 + g.lineage.town.bank as i64;
        let mut new_today = false;
        let level0 = g.lineage.class_level();
        let stages0 = riddle_core::town::stage_set(&g.lineage);
        let mut opened = false;
        for ci in 0..checkins {
            k += 1;
            let tuned = cfg.bot == Bot::Tuned && cfg.has("pen") && g.lineage.pkg.pen_open;
            // (the stall verdict and the death verdict cost more than the absence: TUNED reads the worst
            // death once a day and takes the wall's edit for a plateau)
            let rep = riddle_core::offline::run_offline_counts(&mut g, interval);
            out.checkins += 1;
            out.sends += rep.runs;
            out.stalled += rep.stalled;
            out.deaths += rep.deaths.iter().map(|d| d.n).sum::<u32>();
            // `DP_STALLS=1`: each stalled send's last turns (the rows that looped)
            if rep.stalled > 0 && std::env::var("DP_STALLS").is_ok() {
                for rec in g.deaths.values().filter(|r| r.death.verdict == "stall" && !r.death.trace.turns.is_empty()) {
                    let tail: Vec<String> = rec.death.trace.turns.iter().rev().take(8).map(|t| if t.row >= 0 { rec.rules.rows.get(t.row as usize).map(|r| format!("{}[{}]", r.describe(), r.origin.clone().unwrap_or_default())).unwrap_or_default() } else { format!("chore {}", t.verb.v) }).collect();
                    eprintln!("  stall D{} {}: {}", rec.death.depth, rec.death.cause, tail.join(" | "));
                }
                g.deaths.retain(|_, r| r.death.verdict != "stall");
            }
            out.grew += !rep.grew.is_empty() as u32;
            opened |= !rep.systems_opened.is_empty();
            // (a reveal is a unit: the pen's group is one)
            let mut triggers: Vec<&str> = rep.systems_opened.iter().filter_map(|id| riddle_core::systems::SYSTEMS.iter().find(|d| d.id == id).map(|d| d.trigger)).collect();
            triggers.dedup();
            let units = triggers.len();
            out.max_systems = out.max_systems.max(units);
            out.max_beats = out.max_beats.max(rep.packages.len() + units);
            new_today |= !rep.systems_opened.is_empty() || !rep.packages.is_empty() || !rep.bests.is_empty();
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
                eprintln!("  [{}] day {} ci {} runs {} bank {} ret {} stalled {} deaths {:?} best D{} wearing {:?}", cfg.label(), day + 1, ci, rep.runs, rep.banked, rep.returned, rep.stalled, rep.deaths.iter().map(|d| format!("{}×{}", d.cause, d.n)).collect::<Vec<_>>(), g.lineage.best_depth, g.lineage.pkg.equipped());
            }
            match cfg.bot {
                Bot::Idle => {}
                Bot::Random => {
                    // a random package each check-in, blind: any arrived one (the worn one included)
                    // into its slot — not a forecast's move list, which holds only moves worth making
                    let owned: Vec<String> = g.lineage.pkg.owned.iter().filter(|id| riddle_core::packages::def(id).is_some()).cloned().collect();
                    if !owned.is_empty() {
                        let id = owned[rng.below(owned.len() as u32) as usize].clone();
                        let slot = rng.below(2) as usize;
                        let _ = riddle_core::packages::equip(&mut g.lineage, &id, slot);
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
                        // the forecast is read when something new is on the shelf (a package arrived, a
                        // new record, a wake card) and once a day besides
                        // (a swap holds a day before the next is weighed, unless a package arrives)
                        let key = (g.lineage.pkg.owned.len(), g.lineage.best_depth, g.lineage.pkg.offer.clone());
                        let arrived = last_key.as_ref().is_some_and(|k| k.0 != key.0 || k.2 != key.2);
                        let fresh = arrived || (cool == 0 && (ci == 0 || last_key.as_ref() != Some(&key)));
                        let stance = g.lineage.pkg.equipped();
                        pick_package(&mut g, verbose, day, fresh);
                        cool = if g.lineage.pkg.equipped() != stance { checkins as u32 } else { cool.saturating_sub(1) };
                        last_key = Some((g.lineage.pkg.owned.len(), g.lineage.best_depth, g.lineage.pkg.offer.clone()));
                    }
                    if tuned {
                        let pets = cfg.has("pets");
                        write_own_rows(&mut g, pets);
                        if let Some(id) = rep.worst_death_id.filter(|_| ci == 0) {
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
                        // (the wall search is the harness's costliest call: at most every other day)
                        let fresh = g.lineage.wall_day != Some(g.lineage.day) && last_wall.is_none_or(|d| day >= d + 2);
                        if fresh && riddle_core::wall::at_wall(&g.lineage) {
                            last_wall = Some(day);
                        }
                        if let Some(w) = if fresh { g.wall_edit() } else { None } {
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
                        // (the purse keeps the shelf's money: three units — a few sends' potions at depth)
                        let reserve = 3 * riddle_core::kit::unit(g.lineage.best_depth) as i32;
                        forge(&mut g, reserve);
                    }
                    if cfg.has("bank") && riddle_core::town::built(&g.lineage, "bank") {
                        // the purse keeps the next forge step and the shelf's money; the rest earns
                        let next = riddle_core::kit::ladders(&g.lineage).iter().filter_map(|l| l.next.as_ref().map(|x| x.price as i32)).min().unwrap_or(0);
                        let spare = g.lineage.gold - next - 3 * riddle_core::kit::unit(g.lineage.best_depth) as i32;
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
        out.depth_area += best;
        out.quests = g.lineage.town.quests_done;
        out.gold_day.push(g.lineage.gold as i64 + g.lineage.town.bank as i64 - wealth0);
        out.stance_level_day.push(g.lineage.pkg.level(&g.lineage.pkg.stance));
        opened |= riddle_core::town::stage_set(&g.lineage).len() > stages0.len();
        out.stage_days += opened as usize;
        new_today |= g.lineage.class_level() > level0;
        if day == 0 {
            out.day1_systems = g.lineage.systems.len();
        }
        if new_today {
            out.new_days += 1;
            gap = 0;
        } else {
            gap += 1;
            out.new_gap = out.new_gap.max(gap);
        }
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
    // The bots fill the machine seed by seed; a panel's sims stay sequential inside a job (a TUNED
    // bot's wall search and verdicts on every core, 32 jobs at once, ran the load past 150).
    // (a panel's sims on `RIDDLE_THREADS` threads, 3 unless set, beside `--threads` jobs, 10 unless set:
    // ~30 at the peak — the jobs are mostly one thread, a PICKED or TUNED camp read widens for a moment)
    if std::env::var("RIDDLE_THREADS").is_err() {
        std::env::set_var("RIDDLE_THREADS", "3");
    }
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
    // (a leave-one-out is a TUNED fortnight less a system — the table's costliest job, ~25 CPU-min
    // each: `--loo-seeds`, 4 by default, the first seeds of the bots' own)
    let loo_seeds = get("--loo-seeds", 4);
    // (TUNED is a leave-one-out's base: the same seeds, `--tuned-seeds`, the bots' own by default)
    let tuned_seeds = get("--tuned-seeds", seeds);
    // Per-job results are kept under `target/gates/dp/` by the binary's own hash (a job is a pure function
    // of the binary, the bot, the seed and the days): a rerun of an unchanged binary reprints at once, and
    // an interrupted run resumes where it stopped.
    let bin_key = {
        let bytes = std::fs::read(std::env::current_exe().expect("exe")).unwrap_or_default();
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in &bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        format!("{h:016x}")
    };
    let cache_dir = std::path::PathBuf::from("target/gates/dp");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_of = |c: &Cfg, s: u64| cache_dir.join(format!("{bin_key}-{}-s{s}-d{days}-c{checkins}.json", c.label()));
    let jobs: Vec<(usize, u64)> = (0..cfgs.len())
        .flat_map(|c| {
            let n = if cfgs[c].without.is_some() { loo_seeds.min(seeds) } else if cfgs[c].bot == Bot::Tuned { tuned_seeds.min(seeds) } else { seeds };
            (1..=n).filter(|s| only.is_none_or(|o| o == *s)).map(move |s| (c, s))
        })
        .collect();
    // the longest first (TUNED and its leave-one-outs), so the short ones fill the cores at the end
    let mut jobs = jobs;
    jobs.sort_by_key(|(c, s)| (cfgs[*c].bot != Bot::Tuned, *s));
    let threads = get("--threads", 10) as usize;
    let next = std::sync::atomic::AtomicUsize::new(0);
    let results: std::sync::Mutex<Vec<(usize, SeedOut)>> = std::sync::Mutex::new(Vec::new());
    std::thread::scope(|sc| {
        for _ in 0..threads.max(1) {
            sc.spawn(|| loop {
                let i = next.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let Some(&(c, s)) = jobs.get(i) else { break };
                let t = std::time::Instant::now();
                let path = cache_of(&cfgs[c], s);
                let hit = if verbose { None } else { std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<SeedOut>(&t).ok()) };
                let o = match hit {
                    Some(o) => o,
                    None => {
                        let o = play(s, days, checkins, cfgs[c], verbose);
                        let _ = std::fs::write(&path, serde_json::to_string(&o).unwrap_or_default());
                        o
                    }
                };
                eprintln!("dayplayer: {} s{s} done in {:.0}s", cfgs[c].label(), t.elapsed().as_secs_f64());
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
    // Cut 30 §6, the kept rows re-derived on the new bots (docs/CUT30.md, deviations): content reach (was
    // COUNTERED ≥ D14 ≥ 50 %, a written counter set) is PICKED past the Mother's floor by day 2; stalls (was
    // ≤ 1 % on every cohort set) are ≤ 1 % of every bot's sends
    if !picked.is_empty() {
        let n = picked.len();
        let d14 = picked.iter().filter(|o| o.best_day.get(1).is_some_and(|b| *b >= 14)).count();
        bars.push(("Content reach: PICKED ≥ D14 by day 2 (≥ 50 %)".into(), format!("{d14}/{n}"), d14 * 2 >= n));
    }
    for (label, v) in [("IDLE", &idle), ("PICKED", &picked), ("TUNED", &tuned)] {
        if v.is_empty() {
            continue;
        }
        let worst = v.iter().map(|o| o.stalled as f64 / o.sends.max(1) as f64).fold(0.0f64, f64::max);
        bars.push((format!("Stalls ≤ 1 % of sends, every {label} seed"), format!("{:.2}%", 100.0 * worst), worst <= 0.01));
    }
    if !tuned.is_empty() && !picked.is_empty() {
        // (the owner, 2026-10-01: the pen is an optional late fine-tuning layer — it beats the packages at
        // the deepest walls by ≥ 15 %, and nothing needs it; was `TUNED ≥ 1.5× PICKED at D18, D23, D28`)
        let rs: Vec<f64> = [4, 5].iter().map(|&i| ratio(&picked, &tuned, i)).collect();
        bars.push(("TUNED beats PICKED by ≥ 15 % at D28, D33 (median hours)".into(), rs.iter().map(|r| format!("{r:.2}")).collect::<Vec<_>>().join(" · "), rs.iter().all(|r| *r >= 1.15)));
    }
    if !random.is_empty() && !idle.is_empty() {
        // (RANDOM plays IDLE's own sends until its first pick — the Warlord met, D8 — and most seeds reach
        // D13 a check-in or two later: at D13 it is IDLE's twin, never faster; the random picks tell at D23)
        let n = random.len();
        let not_faster = random.iter().zip(&idle).filter(|(r, i)| hours_or(r, 1, cap) >= hours_or(i, 1, cap)).count();
        let slower = random.iter().zip(&idle).filter(|(r, i)| hours_or(r, 3, cap) > hours_or(i, 3, cap)).count();
        let (a, b) = (100.0 * not_faster as f64 / n as f64, 100.0 * slower as f64 / n as f64);
        bars.push(("RANDOM never faster than IDLE to D13, slower to D23 (≥ 80 % of seeds each)".into(), format!("{a:.0}% · {b:.0}%"), a >= 80.0 && b >= 80.0));
    }
    if !tuned.is_empty() && !idle.is_empty() {
        let step = (24 / checkins) as f64;
        let loo_n = SYSTEMS.iter().map(|s| by(&format!("TUNED-{s}")).len()).max().unwrap_or(0).max(1);
        let d23_tuned = median(tuned.iter().take(loo_n).map(|o| hours_or(o, 3, cap)).collect());
        let d23_idle = median(idle.iter().take(loo_n).map(|o| hours_or(o, 3, cap)).collect());
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
            let n = v.len().min(tuned.len());
            let t = &tuned[..n];
            let w = &v[..n];
            let mean = |xs: &[SeedOut], f: &dyn Fn(&SeedOut) -> f64| xs.iter().map(f).sum::<f64>() / xs.len().max(1) as f64;
            // each system by its own output (the owner, 2026-10-01: systems are not measured on depth)
            let (with, without, unit, ok) = match s {
                "bank" => {
                    let g = |o: &SeedOut| o.gold_day.iter().sum::<i64>() as f64 / o.gold_day.len().max(1) as f64;
                    let (a, b) = (mean(t, &g), mean(w, &g));
                    (a, b, "$/day", a > b * 1.01)
                }
                "pets" => {
                    let d = |o: &SeedOut| 100.0 * o.deaths as f64 / o.sends.max(1) as f64;
                    let (a, b) = (mean(t, &d), mean(w, &d));
                    (a, b, "% deaths", a < b)
                }
                "quests" => {
                    let q = |o: &SeedOut| o.quests as f64;
                    let (a, b) = (mean(t, &q), mean(w, &q));
                    (a, b, "kept", a >= 1.0 && a > b)
                }
                "forge" | "pen" => {
                    let d = |o: &SeedOut| o.depth_area as f64 / o.best_day.len().max(1) as f64;
                    let (a, b) = (mean(t, &d), mean(w, &d));
                    (a, b, "mean best", a > b)
                }
                _ => {
                    let h = |o: &SeedOut| hours_or(o, 3, cap);
                    let (a, b) = (median(t.iter().map(h).collect()), median(w.iter().map(h).collect()));
                    (a, b, "h→D23", a + step <= b)
                }
            };
            moves.push((s, with, without, unit, ok));
        }
        if !moves.is_empty() {
            bars.push(("Nothing required: TUNED − S never slower than IDLE (± a check-in)".into(), if req_ok { "ok".into() } else { worst_req }, req_ok));
            let each = moves.iter().all(|m| m.4);
            bars.push(("Each system adds value by its own output (TUNED vs TUNED − S)".into(), moves.iter().map(|(s, a, b, u, _)| format!("{s} {a:.1}/{b:.1} {u}")).collect::<Vec<_>>().join(" · "), each));
            // (`none > 60 % of TUNED − IDLE` retired: the systems are measured by their own outputs, no
            // longer on one depth scale; `nothing required` and each system's own value replace it)
            let _ = (gap, d23_tuned);
        }
    }
    // PROGRESSION_V2 (reported now, gated in Cut 31)
    for (label, v) in [("IDLE", &idle), ("PICKED", &picked), ("TUNED", &tuned)] {
        if v.is_empty() {
            continue;
        }
        let med = |f: &dyn Fn(&SeedOut) -> usize| median(v.iter().map(|o| f(o) as f64).collect());
        println!(
            "info {label}: day-1 systems {:.0} (≤ 10) · most systems a report {} (≤ 1) · most beats a report {} (≤ 5) · days with something new {:.0}/{days} (≥ 27/30) · longest gap {} d (≤ 1)",
            med(&|o| o.day1_systems),
            v.iter().map(|o| o.max_systems).max().unwrap_or(0),
            v.iter().map(|o| o.max_beats).max().unwrap_or(0),
            med(&|o| o.new_days),
            v.iter().map(|o| o.new_gap).max().unwrap_or(0)
        );
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
