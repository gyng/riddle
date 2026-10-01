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
const MILESTONES: [u32; 7] = [8, 13, 18, 23, 28, 29, 33];
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
fn write_own_rows(g: &mut Game, pets: impl Fn() -> bool) -> u32 {
    if !g.lineage.pkg.pen_open {
        return 0;
    }
    let mut n = 0;
    // (the stray's row only when its fact is held — the loop's own test — and pets are in play: asked then)
    let stray = g.lineage.facts.contains("stray") && pets();
    let situations = std::iter::once("stray").filter(|_| stray).chain(riddle_core::descent::SITUATION_DEPTHS.iter().map(|(w, _)| *w));
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
const PICK_SIMS: u32 = 32;
const PICK_BAR: f64 = 0.12;

/// Whether `forge` would take a step (it buys the cheapest when the purse pays it with `reserve` to spare).
fn forge_due(g: &Game, reserve: i32) -> bool {
    let lads = riddle_core::kit::ladders(&g.lineage);
    lads.iter().filter_map(|l| l.next.as_ref().map(|x| x.price)).min().is_some_and(|price| g.lineage.gold >= price as i32 + reserve)
}

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

thread_local! {
    static PH: std::cell::RefCell<std::collections::BTreeMap<&'static str, f64>> = Default::default();
}
/// This process's CPU seconds (user + system, `/proc/self/stat`): a phase's sims run on worker threads.
fn proc_cpu() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    let f: Vec<&str> = s.rsplit(')').next().unwrap_or("").split_whitespace().collect();
    let t = |i: usize| f.get(i).and_then(|x| x.parse::<f64>().ok()).unwrap_or(0.0);
    (t(11) + t(12)) / 100.0
}
fn ph<R>(name: &'static str, f: impl FnOnce() -> R) -> R {
    let t = proc_cpu();
    let r = f();
    let dt = proc_cpu() - t;
    PH.with(|m| *m.borrow_mut().entry(name).or_default() += dt);
    r
}

/// What a bot's play reads of its configuration. Every read goes through `Ask` and is logged, so two
/// configurations that answer a check-in's reads alike play that check-in alike from the same state —
/// the fortnights of a seed share their common prefix (`Tree`): PICKED, TUNED and its leave-one-outs
/// are one game until a read tells them apart (TUNED's at the pen, TUNED − forge's at its first forge
/// step …), and TUNED − pen is PICKED throughout. A read is asked where its answer is used, never
/// earlier (`ask.has("forge")` only when a forge step is affordable), so the shared prefix is the
/// longest the code allows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Q {
    /// the bot's arm: 0 IDLE, 1 RANDOM, 2 PICKED or TUNED
    Arm,
    /// TUNED with its pen (the pen's rows written once it opens)
    Tuned,
    Has(&'static str),
}

fn answer(cfg: &Cfg, q: Q) -> u8 {
    match q {
        Q::Arm => match cfg.bot {
            Bot::Idle => 0,
            Bot::Random => 1,
            Bot::Picked | Bot::Tuned => 2,
        },
        Q::Tuned => (cfg.bot == Bot::Tuned && cfg.has("pen")) as u8,
        Q::Has(s) => cfg.has(s) as u8,
    }
}

struct Ask<'a> {
    cfg: &'a Cfg,
    log: std::cell::RefCell<Vec<(Q, u8)>>,
}

impl Ask<'_> {
    fn new(cfg: &Cfg) -> Ask<'_> {
        Ask { cfg, log: Default::default() }
    }
    fn q(&self, q: Q) -> u8 {
        let a = answer(self.cfg, q);
        self.log.borrow_mut().push((q, a));
        a
    }
    fn has(&self, s: &'static str) -> bool {
        self.q(Q::Has(s)) == 1
    }
    /// Whether `other` answers every read logged so far as this configuration did.
    fn same(&self, other: &Cfg) -> bool {
        self.log.borrow().iter().all(|(q, a)| answer(other, *q) == *a)
    }
}

/// One bot's fortnight as a state that advances a check-in at a time (`step`), cloned where the
/// configurations sharing it part.
#[derive(Clone)]
struct Play {
    seed: u64,
    days: usize,
    checkins: u64,
    interval: u64,
    verbose: bool,
    g: Game,
    out: SeedOut,
    rng: Rng,
    last_best: u32,
    stall_cur: usize,
    reached23: bool,
    k: u64,
    last_key: Option<(usize, u32, Vec<String>)>,
    cool: u32,
    last_wall: Option<usize>,
    gap: usize,
    // the day's
    wealth0: i64,
    new_today: bool,
    level0: u32,
    stages0: Vec<(String, String)>,
    opened: bool,
    /// the next check-in
    day: usize,
    ci: u64,
}

impl Play {
    fn new(seed: u64, days: usize, checkins: u64, verbose: bool, ask: &Ask) -> Play {
        let interval = 24 * 3600 / checkins;
        let mut g = Game::new(seed);
        if !ask.has("quests") {
            g.lineage.town.off = true;
        }
        let out = SeedOut { seed, hours: vec![None; MILESTONES.len()], ..Default::default() };
        let rng = Rng::derive(seed, 0x5EED_0B07);
        Play { seed, days, checkins, interval, verbose, g, out, rng, last_best: 0, stall_cur: 0, reached23: false, k: 0, last_key: None, cool: 0, last_wall: None, gap: 0, wealth0: 0, new_today: false, level0: 0, stages0: Vec::new(), opened: false, day: 0, ci: 0 }
    }

    fn done(&self) -> bool {
        self.day >= self.days
    }

    /// One check-in (its day's start before the first, the day's end after the last).
    fn step(&mut self, ask: &Ask) {
        let (checkins, interval, verbose, day, ci) = (self.checkins, self.interval, self.verbose, self.day, self.ci);
        let cfg = ask.cfg;
        if ci == 0 {
            let g = &self.g;
            self.wealth0 = g.lineage.gold as i64 + g.lineage.town.bank as i64;
            self.new_today = false;
            self.level0 = g.lineage.class_level();
            self.stages0 = riddle_core::town::stage_set(&g.lineage);
            self.opened = false;
        }
        let Play { g, out, rng, k, last_key, cool, last_wall, .. } = self;
        {
            *k += 1;
            let tuned = g.lineage.pkg.pen_open && ask.q(Q::Tuned) == 1;
            // (the stall verdict and the death verdict cost more than the absence: TUNED reads the worst
            // death once a day and takes the wall's edit for a plateau)
            let rep = ph("offline", || riddle_core::offline::run_offline_counts(g, interval));
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
            self.opened |= !rep.systems_opened.is_empty();
            // (a reveal is a unit: the pen's group is one)
            let mut triggers: Vec<&str> = rep.systems_opened.iter().filter_map(|id| riddle_core::systems::SYSTEMS.iter().find(|d| d.id == id).map(|d| d.trigger)).collect();
            triggers.dedup();
            let units = triggers.len();
            out.max_systems = out.max_systems.max(units);
            out.max_beats = out.max_beats.max(rep.packages.len() + units);
            self.new_today |= !rep.systems_opened.is_empty() || !rep.packages.is_empty() || !rep.bests.is_empty();
            let hours = (*k * interval) as f64 / 3600.0;
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
            match ask.q(Q::Arm) {
                0 => {}
                1 => {
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
                        let set = riddle_core::probes::random_rules(rng, &vocab, 3);
                        let mut rules = g.lineage.rules().clone();
                        rules.rows.retain(|r| r.is_pkg());
                        for (i, r) in set.rows.into_iter().enumerate() {
                            rules.rows.insert(i, r.from("player"));
                        }
                        let _ = g.set_rules(rules);
                    }
                }
                _ => {
                    if ask.has("packages") {
                        // the forecast is read when something new is on the shelf (a package arrived, a
                        // new record, a wake card) and once a day besides
                        // (a swap holds a day before the next is weighed, unless a package arrives)
                        let key = (g.lineage.pkg.owned.len(), g.lineage.best_depth, g.lineage.pkg.offer.clone());
                        let arrived = last_key.as_ref().is_some_and(|k| k.0 != key.0 || k.2 != key.2);
                        // (once a day, and when a package arrives: a new record alone re-reads nothing — the
                        // camp's panels are the harness's costliest call)
                        let fresh = arrived || (*cool == 0 && ci == 0);
                        let stance = g.lineage.pkg.equipped();
                        ph("pick", || pick_package(g, verbose, day, fresh));
                        *cool = if g.lineage.pkg.equipped() != stance { checkins as u32 } else { cool.saturating_sub(1) };
                        *last_key = Some((g.lineage.pkg.owned.len(), g.lineage.best_depth, g.lineage.pkg.offer.clone()));
                    }
                    if tuned {
                        let pets = || ask.has("pets");
                        write_own_rows(g, pets);
                        // a wall's counter, written once its fact is known (the drill is days away; at the deep
                        // walls a week): the boss on the record's floor or the next, not yet slain nor drilled
                        let best = g.lineage.best_depth;
                        let route = g.lineage.rules().route();
                        for b in [best, best + 1].into_iter().filter_map(|d| route.boss(d)) {
                            // (the Foundry's counter is its golems' fact, as its drill's)
                            let known = riddle_core::facts::boss_counter_known(&g.lineage.facts, b) || (b == "foundry_master" && riddle_core::facts::tag_known(&g.lineage.facts, "iron_golem", "reflect_melee"));
                            if known && !g.lineage.kills.contains(b) && !g.lineage.pkg.drills.iter().any(|d| d.boss == b) {
                                let heal = riddle_core::packages::heal_pct(&g.lineage.pkg.stance, g.lineage.pkg.level(&g.lineage.pkg.stance));
                                for row in riddle_core::packages::drill_rows(b, heal) {
                                    // (the card or the verb it needs, bought with marks or gold when the purse has it)
                                    let need = match (row.verb.v.as_str(), row.card()) {
                                        (_, Some(c)) => Some(c.to_string()),
                                        ("throw", _) => Some("throw".to_string()),
                                        _ => None,
                                    };
                                    if let Some(u) = need.filter(|u| !g.lineage.unlocks.contains(u)) {
                                        if g.buy(&u).is_err() {
                                            let _ = g.buy_unlock_gold(&u);
                                        }
                                    }
                                    insert_row(g, row, 0);
                                }
                            }
                        }
                        if let Some(id) = rep.worst_death_id.filter(|_| ci == 0) {
                            if let Some(death) = ph("death", || g.death(id)) {
                                if death.verdict == "gap" {
                                    if let Some(p) = death.patches.first() {
                                        if p.survive > death.baseline + 0.15 && p.forecast_delta >= 0.0 {
                                            insert_row(g, p.row.clone(), 0);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(stall) = &rep.stall {
                            if let Some(p) = stall.patches.first().filter(|p| !p.remove && !p.replace) {
                                insert_row(g, p.row.clone(), 0);
                            }
                        }
                        // (the wall search is the harness's costliest call: at most every other day)
                        let fresh = g.lineage.wall_day != Some(g.lineage.day) && last_wall.is_none_or(|d| day >= d + 2);
                        if fresh && riddle_core::wall::at_wall(&g.lineage) {
                            *last_wall = Some(day);
                        }
                        if let Some(w) = if fresh { ph("wall", || g.wall_edit()) } else { None } {
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
                        let mut opts: Vec<_> = ph("unlocks", || g.unlocks()).into_iter().filter(|u| !u.owned && u.available && u.id.starts_with("row") && u.cost <= g.lineage.marks).collect();
                        opts.sort_by(|a, b| a.cost.cmp(&b.cost).then(a.id.cmp(&b.id)));
                        if let Some(u) = opts.first() {
                            let _ = g.buy(&u.id);
                        }
                        // (the kennel fielded when pets are in play: nothing to field, nothing asked)
                        if g.lineage.party.len() < g.lineage.party_slots() as usize && !g.lineage.kennel.is_empty() && pets() {
                            field_kennel(g);
                        }
                    }
                    // (the purse keeps the shelf's money: three units — a few sends' potions at depth)
                    let reserve = 3 * riddle_core::kit::unit(g.lineage.best_depth) as i32;
                    if forge_due(g, reserve) && ask.has("forge") {
                        forge(g, reserve);
                    }
                    if riddle_core::town::built(&g.lineage, "bank") {
                        // the purse keeps the next forge step and the shelf's money; the rest earns
                        let next = riddle_core::kit::ladders(&g.lineage).iter().filter_map(|l| l.next.as_ref().map(|x| x.price as i32)).min().unwrap_or(0);
                        let spare = g.lineage.gold - next - 3 * riddle_core::kit::unit(g.lineage.best_depth) as i32;
                        if spare > 0 && ask.has("bank") {
                            let _ = g.bank_deposit(spare);
                        }
                    }
                }
            }
            // a vault brought along (a human sends the heir out with what it has; every bot)
            let ids: Vec<u32> = g.lineage.vault.iter().map(|i| i.id).collect();
            g.loadout(ids);
        }
        self.ci += 1;
        if self.ci == checkins {
            self.day_end(cfg);
            self.ci = 0;
            self.day += 1;
            if self.done() {
                self.out.rules = self.g.lineage.rules().rows.iter().map(|r| r.describe()).collect::<Vec<_>>().join(" | ");
            }
        }
    }

    fn day_end(&mut self, cfg: &Cfg) {
        let (seed, day, verbose) = (self.seed, self.day, self.verbose);
        let Play { g, out, .. } = self;
        let best = g.lineage.best_depth;
        if std::env::var("DP_PHASES").is_ok() {
            eprintln!("  [{}] s{seed} day {} D{best} pen {} cpu {:.0}", cfg.label(), day + 1, g.lineage.pkg.pen_open, proc_cpu());
        }
        out.best_day.push(best);
        out.depth_area += best;
        out.quests = g.lineage.town.quests_done;
        out.gold_day.push(g.lineage.gold as i64 + g.lineage.town.bank as i64 - self.wealth0);
        out.stance_level_day.push(g.lineage.pkg.level(&g.lineage.pkg.stance));
        self.opened |= riddle_core::town::stage_set(&g.lineage).len() > self.stages0.len();
        out.stage_days += self.opened as usize;
        self.new_today |= g.lineage.class_level() > self.level0;
        if day == 0 {
            out.day1_systems = g.lineage.systems.len();
        }
        if self.new_today {
            out.new_days += 1;
            self.gap = 0;
        } else {
            self.gap += 1;
            out.new_gap = out.new_gap.max(self.gap);
        }
        if best > self.last_best {
            self.last_best = best;
            self.stall_cur = 0;
        } else if !self.reached23 {
            self.stall_cur += 1;
            out.stall = out.stall.max(self.stall_cur);
        }
        self.reached23 |= best >= 23;
        if verbose {
            let p = &g.lineage.pkg;
            eprintln!("  [{}] s{} day {} D{} ${} bank {} L{} {} L{} drills {} kit {} pen {}", cfg.label(), seed, day + 1, best, g.lineage.gold, g.lineage.town.bank, g.lineage.class_level(), p.stance, p.level(&p.stance), p.drills.len(), riddle_core::kit::KIT_SLOTS.iter().map(|s| riddle_core::kit::owned(&g.lineage, s)).sum::<u32>(), p.pen_open);
        }
    }
}

/// One configuration's fortnight, alone.
#[allow(dead_code)]
fn play(seed: u64, days: usize, checkins: u64, cfg: Cfg, verbose: bool) -> SeedOut {
    let ask = Ask::new(&cfg);
    let mut p = Play::new(seed, days, checkins, verbose, &ask);
    while !p.done() {
        p.step(&Ask::new(&cfg));
    }
    p.out
}

/// A seed's configurations playing one game (`Play`) until a read parts them; `state` is `None` before
/// the lineage is made.
struct Group {
    seed: u64,
    members: Vec<usize>,
    state: Option<Play>,
}

/// The groups waiting, longest first (the TUNED family's fortnights before the others, then by the
/// check-ins left); a group that parts pushes the parting members back. The workers stop when the
/// queue is empty and no group is playing.
#[derive(Default)]
struct Pool {
    q: std::sync::Mutex<(Vec<(u64, u64, Group)>, u64, usize)>,
    cv: std::sync::Condvar,
}

impl Pool {
    fn push(&self, g: Group, cfgs: &[Cfg], days: usize, checkins: u64) {
        let weight = g.members.iter().map(|c| match cfgs[*c].bot {
            Bot::Tuned => 3,
            Bot::Picked => 2,
            _ => 1,
        });
        let left = g.state.as_ref().map_or(days as u64 * checkins, |p| (days - p.day) as u64 * checkins - p.ci);
        let prio = weight.max().unwrap_or(1) * left;
        let mut q = self.q.lock().unwrap();
        q.1 += 1;
        q.2 += 1;
        let seq = q.1;
        q.0.push((prio, u64::MAX - seq, g));
        self.cv.notify_one();
    }
    fn pop(&self) -> Option<Group> {
        let mut q = self.q.lock().unwrap();
        loop {
            if let Some(i) = (0..q.0.len()).max_by_key(|i| (q.0[*i].0, q.0[*i].1)) {
                return Some(q.0.swap_remove(i).2);
            }
            if q.2 == 0 {
                return None;
            }
            q = self.cv.wait(q).unwrap();
        }
    }
    fn done(&self) {
        let mut q = self.q.lock().unwrap();
        q.2 -= 1;
        if q.2 == 0 {
            self.cv.notify_all();
        }
    }
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
    let seeds_of = |c: usize| if cfgs[c].without.is_some() { loo_seeds.min(seeds) } else if cfgs[c].bot == Bot::Tuned { tuned_seeds.min(seeds) } else { seeds };
    // Per-job results are kept under `target/gates/dp/` (a job is a pure function of the code, the bot, the
    // seed and the days): a rerun reprints at once, and an interrupted run resumes where it stopped. The key
    // is the binary's own hash, or — under `tools/gates.mjs`, which sets `RIDDLE_SRC_KEY` to the hash of the
    // core's sources, the lockfile and the toolchain — that and this file less `main` (the bars below), so an
    // edit to a bar reprints the table from the jobs kept.
    let fnv = |bytes: &[u8], mut h: u64| {
        for b in bytes {
            h ^= *b as u64;
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        h
    };
    let bin_key = match std::env::var("RIDDLE_SRC_KEY") {
        Ok(k) => {
            let src = include_str!("dayplayer.rs");
            let body = src.find("\nfn main() {").map_or(src, |i| &src[..i]);
            let env = std::env::var("RIDDLE_SKIP_TWIST").unwrap_or_default();
            format!("src{:016x}", fnv(env.as_bytes(), fnv(body.as_bytes(), fnv(k.as_bytes(), 0xcbf2_9ce4_8422_2325))))
        }
        Err(_) => format!("{:016x}", fnv(&std::fs::read(std::env::current_exe().expect("exe")).unwrap_or_default(), 0xcbf2_9ce4_8422_2325)),
    };
    // (`DP_STALLS` prunes the stall records it prints: a different game, never kept)
    let keep = !verbose && std::env::var("DP_STALLS").is_err();
    let cache_dir = std::path::PathBuf::from("target/gates/dp");
    let _ = std::fs::create_dir_all(&cache_dir);
    let cache_of = |c: &Cfg, s: u64| cache_dir.join(format!("{bin_key}-{}-s{s}-d{days}-c{checkins}.json", c.label()));
    let results: std::sync::Mutex<Vec<(usize, SeedOut)>> = std::sync::Mutex::new(Vec::new());
    // Each seed's configurations to play (those not kept), as one group: the group plays as its first
    // member and parts where a member answers a read differently (`Ask`) — the parting members go on as a
    // group of their own from the check-in's start (`Play` cloned there). `--no-share`: each alone.
    let share = !verbose && !a.iter().any(|x| x == "--no-share");
    let mut groups: Vec<Group> = Vec::new();
    let max_seed = (0..cfgs.len()).map(|c| seeds_of(c)).max().unwrap_or(0);
    for s in (1..=max_seed).filter(|s| only.is_none_or(|o| o == *s)) {
        let mut members = Vec::new();
        for c in (0..cfgs.len()).filter(|c| s <= seeds_of(*c)) {
            let hit = if keep { std::fs::read_to_string(cache_of(&cfgs[c], s)).ok().and_then(|t| serde_json::from_str::<SeedOut>(&t).ok()) } else { None };
            match hit {
                Some(o) => results.lock().unwrap().push((c, o)),
                None => members.push(c),
            }
        }
        // (the longest bots lead: TUNED first, so a group's first member is the one that parts least)
        members.sort_by_key(|c| (cfgs[*c].bot != Bot::Tuned || cfgs[*c].without.is_some(), cfgs[*c].bot != Bot::Picked, *c));
        if share {
            if !members.is_empty() {
                groups.push(Group { seed: s, members, state: None });
            }
        } else {
            groups.extend(members.into_iter().map(|c| Group { seed: s, members: vec![c], state: None }));
        }
    }
    let threads = get("--threads", 10) as usize;
    let pool = Pool::default();
    for g in groups {
        pool.push(g, &cfgs, days, checkins);
    }
    std::thread::scope(|sc| {
        for _ in 0..threads.max(1) {
            sc.spawn(|| {
                while let Some(Group { seed: s, mut members, state }) = pool.pop() {
                    let t = std::time::Instant::now();
                    let lead = members[0];
                    let part = |ask: &Ask, members: &mut Vec<usize>, snap: Option<Play>| {
                        let (same, diff): (Vec<usize>, Vec<usize>) = members[1..].iter().partition(|m| ask.same(&cfgs[**m]));
                        if !diff.is_empty() {
                            pool.push(Group { seed: s, members: diff, state: snap }, &cfgs, days, checkins);
                        }
                        members.truncate(1);
                        members.extend(same);
                    };
                    let mut p = match state {
                        Some(p) => p,
                        None => {
                            let ask = Ask::new(&cfgs[lead]);
                            let p = Play::new(s, days, checkins, verbose, &ask);
                            part(&ask, &mut members, None);
                            p
                        }
                    };
                    while !p.done() {
                        let snap = (members.len() > 1).then(|| p.clone());
                        let ask = Ask::new(&cfgs[lead]);
                        p.step(&ask);
                        if members.len() > 1 {
                            part(&ask, &mut members, snap);
                        }
                    }
                    let phs = PH.with(|m| std::mem::take(&mut *m.borrow_mut()));
                    for &c in &members {
                        if keep {
                            let _ = std::fs::write(cache_of(&cfgs[c], s), serde_json::to_string(&p.out).unwrap_or_default());
                        }
                        results.lock().unwrap().push((c, p.out.clone()));
                    }
                    eprintln!("dayplayer: {} s{s} done in {:.0}s {}", members.iter().map(|c| cfgs[*c].label()).collect::<Vec<_>>().join(" = "), t.elapsed().as_secs_f64(), phs.iter().map(|(k, v)| format!("{k} {v:.0}")).collect::<Vec<_>>().join(" "));
                    pool.done();
                }
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
        bars.push((format!("Days with a stage opened: IDLE ≥ 8/{days} (median)"), format!("{sd:.1}"), sd >= 8.0));
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
        bars.push((format!("Days with a stage opened: PICKED ≥ 10/{days} (median)"), format!("{sd:.1}"), sd >= 10.0));
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
        // the deepest walls by ≥ 15 %, and nothing needs it; was `TUNED ≥ 1.5× PICKED at D18, D23, D28`;
        // round 6: past D28 — D29 — not the D28 floor, which the walls above it gate)
        let rs: Vec<f64> = [5, 6].iter().map(|&i| ratio(&picked, &tuned, i)).collect();
        bars.push(("TUNED beats PICKED by ≥ 15 % at D29, D33 (median hours)".into(), rs.iter().map(|r| format!("{r:.2}")).collect::<Vec<_>>().join(" · "), rs.iter().all(|r| *r >= 1.15)));
    }
    // (the owner, round 6: a random package is mostly a good one, so RANDOM is weighed against the picker —
    // never ahead of PICKED on any seed at D13 or D23; the RANDOM-vs-IDLE row prints as retired)
    let mut retired: Vec<(String, String)> = Vec::new();
    if !random.is_empty() && !picked.is_empty() {
        let n = random.len().min(picked.len());
        let at = |i: usize| random.iter().zip(&picked).filter(|(r, p)| hours_or(r, i, cap) >= hours_or(p, i, cap)).count();
        let (a, b) = (at(1), at(3));
        bars.push(("RANDOM never beats PICKED at D13, D23 (every seed)".into(), format!("{a}/{n} · {b}/{n}"), a == n && b == n));
    }
    if !random.is_empty() && !idle.is_empty() {
        let n = random.len();
        let not_faster = random.iter().zip(&idle).filter(|(r, i)| hours_or(r, 1, cap) >= hours_or(i, 1, cap)).count();
        let slower = random.iter().zip(&idle).filter(|(r, i)| hours_or(r, 3, cap) > hours_or(i, 3, cap)).count();
        let (a, b) = (100.0 * not_faster as f64 / n as f64, 100.0 * slower as f64 / n as f64);
        retired.push(("RANDOM never faster than IDLE to D13, slower to D23 (≥ 80 % each)".into(), format!("{a:.0}% · {b:.0}%")));
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
    for (name, value) in &retired {
        println!("{:<64} {:>18}  retired", name, value);
    }
    println!("dayplayer: {}", if fails > 0 { "FAIL" } else { "all PASS" });
    if gate && fails > 0 {
        eprintln!("{fails} bar(s) FAIL");
        std::process::exit(1);
    }
    let _ = RuleSet::default();
}
