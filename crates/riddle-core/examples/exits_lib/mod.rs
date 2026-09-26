//! Cut 27 §3: bank and return trade. Each cohort set with an exit row plays twice on the same
//! lineage seed — every exit row a `bank` (the bank twin), every exit row a `return` (the return
//! twin) — `EXIT_HOURS` of watched sends from D1, each followed by its rest, the shelf packed
//! (as `cohort_gold`). Read per twin: gold per hour (net of the shelf, over the sends' ticks and
//! rests), the death share, and the reach at the bank twin's median floor.
use riddle_core::engine::ExitTier;
use riddle_core::{Game, RuleSet};
use std::collections::BTreeMap;

pub const EXIT_HOURS: u64 = 4;
/// Seeds per twin (at most the table's).
pub const EXIT_SEEDS: u64 = 12;
/// The win bar: percentage points on the death share and the reach, percent on gold per hour.
pub const EXIT_MARGIN: f64 = 10.0;

#[derive(Clone, Default, Debug)]
pub struct ExitTally {
    pub sends: u32,
    pub deaths: u32,
    /// The purse's move over the sends (home, salvage, the repeat's spending, tolls).
    pub net: i64,
    /// Ticks the sends and their rests took.
    pub ticks: u64,
    pub depths: Vec<u32>,
    /// Deaths on the walk home, and the walks' ticks (committed → exit or death) and count.
    pub walk_deaths: u32,
    pub walk_ticks: u64,
    pub walks: u32,
}

impl ExitTally {
    pub fn add(&mut self, o: &ExitTally) {
        self.sends += o.sends;
        self.deaths += o.deaths;
        self.net += o.net;
        self.ticks += o.ticks;
        self.depths.extend(o.depths.iter().copied());
        self.walk_deaths += o.walk_deaths;
        self.walk_ticks += o.walk_ticks;
        self.walks += o.walks;
    }
    pub fn gold_hr(&self) -> f64 {
        self.net as f64 * 3600.0 * riddle_core::offline::TICKS_PER_SECOND as f64 / self.ticks.max(1) as f64
    }
    pub fn death(&self) -> f64 {
        100.0 * self.deaths as f64 / self.sends.max(1) as f64
    }
    pub fn reach(&self, d: u32) -> f64 {
        100.0 * self.depths.iter().filter(|x| **x >= d).count() as f64 / self.depths.len().max(1) as f64
    }
}

pub fn has_exit(set: &RuleSet) -> bool {
    set.rows.iter().any(|r| matches!(r.verb.v.as_str(), "bank" | "return"))
}

/// `set` with every exit row's verb `to` (`bank` or `return`).
pub fn twin(set: &RuleSet, to: &str) -> RuleSet {
    let mut s = set.clone();
    for r in s.rows.iter_mut().filter(|r| matches!(r.verb.v.as_str(), "bank" | "return")) {
        r.verb.v = to.into();
    }
    s
}

pub fn run(set: &RuleSet, seed: u64, hours: u64) -> ExitTally {
    let mut g: Game = super::cohort_game(set, seed);
    g.lineage.gold = 400;
    super::pack_for(&mut g);
    let gold0 = g.lineage.gold as i64;
    let budget = hours * 3600 * riddle_core::offline::TICKS_PER_SECOND;
    let mut t = ExitTally::default();
    while t.ticks < budget {
        g.lineage.gold_ledger.clear();
        g.lineage.rest_left = 0;
        g.lineage.start = 1;
        g.start_run(None);
        g.events.clear();
        let mut n = 0u32;
        while g.run.as_ref().is_some_and(|r| r.over.is_none()) && n < riddle_core::engine::MAX_TURNS_PER_RUN {
            g.tick();
            g.events.clear();
            n += 1;
        }
        let (turns, tier, depth) = {
            let r = g.run.as_ref().unwrap();
            let tier = r.over.unwrap_or(ExitTier::Return);
            if let Some((at, ..)) = r.home_at {
                t.walks += 1;
                t.walk_ticks += (r.turn - at) as u64;
                t.walk_deaths += (tier == ExitTier::Death) as u32;
            }
            (r.turn, tier, r.max_depth)
        };
        t.ticks += turns as u64 + g.rest_after(turns, tier) as u64;
        g.finish_run();
        g.auto_keep();
        g.events.clear();
        t.sends += 1;
        t.deaths += (tier == ExitTier::Death) as u32;
        t.depths.push(depth);
    }
    t.net = g.lineage.gold as i64 - gold0;
    t
}

/// (set, seed, return twin) → tally.
pub type Exits = BTreeMap<(usize, u64, bool), ExitTally>;

/// The per-set table and the gate row: some set's return twin beats its bank twin by
/// `EXIT_MARGIN` on one of (gold/h, death, reach), and some set's bank twin beats its return twin.
pub fn report(sets: &[(String, RuleSet)], exits: &Exits, seeds: u64, hours: u64, rows: &mut Vec<(String, String, bool)>) {
    println!("\nbank vs return (Cut 27 §3; {hours} h watched from D1 × {seeds} seeds; each set's exit rows all bank │ all return)");
    let (mut ret_wins, mut bank_wins, mut n) = (0usize, 0usize, 0usize);
    for (si, (name, set)) in sets.iter().enumerate() {
        if !has_exit(set) {
            continue;
        }
        let pool = |ret: bool| {
            let mut t = ExitTally::default();
            for s in 1..=seeds {
                if let Some(x) = exits.get(&(si, s, ret)) {
                    t.add(x);
                }
            }
            t
        };
        let (b, r) = (pool(false), pool(true));
        if b.sends == 0 || r.sends == 0 {
            continue;
        }
        n += 1;
        let mut ds = b.depths.clone();
        ds.sort();
        let bar = ds[ds.len() / 2].max(1);
        let gold_move = |a: &ExitTally, o: &ExitTally| if o.gold_hr().abs() < 1e-9 { 0.0 } else { 100.0 * (a.gold_hr() - o.gold_hr()) / o.gold_hr().abs() };
        let wins = |a: &ExitTally, o: &ExitTally| -> Vec<String> {
            let mut w = Vec::new();
            if gold_move(a, o) >= EXIT_MARGIN {
                w.push(format!("gold/h {:+.0}%", gold_move(a, o)));
            }
            if o.death() - a.death() >= EXIT_MARGIN {
                w.push(format!("death {:+.0}", a.death() - o.death()));
            }
            if a.reach(bar) - o.reach(bar) >= EXIT_MARGIN {
                w.push(format!("D{bar} {:+.0}", a.reach(bar) - o.reach(bar)));
            }
            w
        };
        let (bw, rw) = (wins(&b, &r), wins(&r, &b));
        bank_wins += !bw.is_empty() as usize;
        ret_wins += !rw.is_empty() as usize;
        println!(
            "  {name}: bank {:.0} sends · ${:.0}/h · death {:.0}% (walk {:.0}%, {:.0} ticks) · D{bar} {:.0}% │ return {:.0} sends · ${:.0}/h · death {:.0}% (walk {:.0}%, {:.0} ticks) · D{bar} {:.0}% │ bank wins [{}] · return wins [{}]",
            b.sends as f64 / seeds as f64,
            b.gold_hr(),
            b.death(),
            100.0 * b.walk_deaths as f64 / b.sends.max(1) as f64,
            b.walk_ticks as f64 / b.walks.max(1) as f64,
            b.reach(bar),
            r.sends as f64 / seeds as f64,
            r.gold_hr(),
            r.death(),
            100.0 * r.walk_deaths as f64 / r.sends.max(1) as f64,
            r.walk_ticks as f64 / r.walks.max(1) as f64,
            r.reach(bar),
            bw.join(", "),
            rw.join(", "),
        );
    }
    if n > 0 {
        rows.push((format!("Bank and return each win a set by ≥ {EXIT_MARGIN:.0} pts ({n} sets)"), format!("return {ret_wins} · bank {bank_wins}"), ret_wins >= 1 && bank_wins >= 1));
    }
}
