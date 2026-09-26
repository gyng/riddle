//! Cut 26 §3: lanes that want different sets. At a fork (D5, D9) the near stair (the band's own
//! biome) and the far one (the next biome, early) are two lanes; a set is measured on each by
//! its reach of the band's last floor (the boss's), paired seeds (the same sims, the same floors
//! above the fork). The per-lane best set is the plateau search's (`climb`), seeded from EDITED
//! and from every cohort set. Shared by `examples/lanes.rs` and `metrics.rs`.
#![allow(dead_code)]
use riddle_core::descent::{Route, BANDS, FORKS};
use riddle_core::engine::ExitTier;
use riddle_core::forecast::SimResult;
use riddle_core::{Cond, Game, Row, RuleSet, Verb};

/// The forks the gate measures.
pub const LANE_FORKS: [u32; 2] = [5, 9];
/// The tag every lane measure shares (base and edits replay the same seeds).
pub const TAG: u64 = 0x1A7E_5026;

/// The two lanes of a fork: the near stair (the base order) and the far one.
pub fn lanes(fork: u32) -> [Route; 2] {
    [Route::BASE, Route::from_forks(&[fork]).expect("a fork")]
}

/// The bar: the band's end — its boss passed (the next band's first floor); `LANE_BAR=last`
/// reads it at the band's last floor (the boss's) instead.
pub fn bar(fork: u32) -> u32 {
    let i = FORKS.iter().position(|f| *f == fork).expect("a fork");
    BANDS[i].1 + (std::env::var("LANE_BAR").ok().as_deref() != Some("last")) as u32
}

/// The floor the lane's sends start on: D1 for the first fork (reached in the first hour), the
/// fork's own waystone below it (lit on both lanes).
pub fn start(fork: u32) -> u32 {
    if fork == FORKS[0] {
        1
    } else {
        fork
    }
}

/// The heir's class level a lineage has by the fork (`start`): a fresh one at D1, a D9 one's.
pub fn level(fork: u32) -> u32 {
    if fork == FORKS[0] {
        std::env::var("LANE_L5").ok().and_then(|v| v.parse().ok()).unwrap_or(3)
    } else {
        std::env::var("LANE_L9").ok().and_then(|v| v.parse().ok()).unwrap_or(7)
    }
}

/// The own rows a lineage has by the fork: 4 at D5 (the fresh cap — the first fork arrives in
/// the first hour, `§4`'s agent: the preset and a patch), 8 at D9.
pub fn rows_cap(fork: u32) -> usize {
    let d = if fork == FORKS[0] { 4 } else { 8 };
    std::env::var(format!("LANE_ROWS{fork}")).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

/// A lineage at the fork: every token and card a cohort set might hold, `rows_cap` own rows,
/// every fact (a player who reached the fork has met the kinds), the class level of `level`, the
/// shelf packed for the set, the fork's waystone lit on both lanes. The set is cut to the cap
/// (its last own rows fall off, as the editor cuts).
pub fn lane_game(set: &RuleSet, seed: u64, fork: u32) -> Game {
    let mut g = Game::new(seed);
    g.max_deaths = 100_000;
    let cap = rows_cap(fork);
    // The first fork's lineage (the first hour) owns no card: a card row is a row outside the cap.
    let first = fork == FORKS[0];
    let card = |id: &str| riddle_core::meta::TACTIC_CARDS.iter().chain(riddle_core::meta::MASTERY_CARDS.iter()).chain(riddle_core::meta::TIER2_CARDS.iter()).any(|c| *c == id);
    for u in riddle_core::meta::UNLOCKS {
        let row = u.id.strip_prefix("row").and_then(|n| n.parse::<usize>().ok());
        if row.is_none_or(|n| n <= cap) && !(first && card(u.id)) {
            g.lineage.unlocks.insert(u.id.into());
        }
    }
    let mut set = set.clone();
    if first {
        set.rows.retain(|r| !r.is_card());
        for r in set.rows.iter_mut() {
            r.conds.retain(|c| c.k != "in");
        }
    }
    let set = &set;
    riddle_core::probes::learn_everything(&mut g);
    // The first fork is seen from D4's stairs, before either lane is entered: no `in:` cond yet.
    if first {
        for b in riddle_core::descent::BASE_ORDER {
            g.lineage.facts.remove(&format!("biome:{}", b.name()));
        }
    }
    let class = g.lineage.class;
    g.lineage.classes.insert(class.name().into(), riddle_core::wire::ClassProg { level: level(fork), xp: 0, next: 0 });
    for f in FORKS {
        g.lineage.facts.insert(format!("fork:{f}"));
    }
    let s = start(fork);
    if s > 1 {
        for r in lanes(fork) {
            let _ = g.lineage.light_waystones_on(s, r);
        }
    }
    g.lineage.start = s;
    g.lineage.gold = 1_000_000;
    g.set_rules_raw(set.fit(cap)).unwrap_or_else(|e| panic!("lane set {:?}: {e}", set.name));
    fill_shelf(&mut g, extra_heals(fork));
    g.lineage.gold = 0;
    g
}

/// The heals a lineage packs past the set's own kinds by the fork: none in the first hour, two by D9.
pub fn extra_heals(fork: u32) -> usize {
    let d = if fork == FORKS[0] { 0 } else { 2 };
    std::env::var(format!("LANE_HEALS{fork}")).ok().and_then(|v| v.parse().ok()).unwrap_or(d)
}

/// Two heals and one of every other kind the set's rows name, then `extra` heals (to the cap).
pub fn fill_shelf(g: &mut Game, extra: usize) {
    g.clear_supplies();
    let kinds = g.lineage.row_kinds();
    let cat = g.supply_catalogue();
    for k in kinds.iter().filter(|k| cat.iter().any(|e| e.kind == **k) && k.as_str() != "leash") {
        for _ in 0..if k == "heal" { 2 } else { 1 } {
            let _ = g.buy_supply(k);
        }
    }
    for _ in 0..extra.min(riddle_core::kit::SUPPLY_CAP_MAX) {
        if g.buy_supply("heal").is_err() {
            break;
        }
    }
}

/// `n` paired sends of `set` on `route` from the lineage's start, each to its exit or past the bar.
pub fn sends(g: &Game, set: &RuleSet, route: Route, fork: u32, n: u32) -> Vec<SimResult> {
    let rules = set.clone().with_route(route);
    riddle_core::forecast::simulate_budget_from(g, &rules, n, TAG ^ fork as u64, bar(fork) + 1, u64::MAX, Vec::new())
}

/// A lane read: the share reaching the bar, the bank share, and gold per hour (kept gold over
/// the sends' ticks; a send stopped past the bar keeps its return share).
#[derive(Clone, Copy, Debug, Default)]
pub struct Read {
    pub reach: f64,
    pub bank: f64,
    pub gold_hr: f64,
    pub n: usize,
    /// The sends' mean deepest floor (the search's footing on a 0 % plateau).
    pub depth: f64,
}

impl Read {
    /// The search's score: the reach, and two points a floor of mean depth (a set that clears
    /// nothing yet still climbs toward one that does).
    pub fn score(&self) -> f64 {
        self.reach + 0.02 * self.depth
    }
}

pub fn read(rs: &[SimResult], fork: u32) -> Read {
    let n = rs.len().max(1) as f64;
    let reach = rs.iter().filter(|r| r.max_depth >= bar(fork)).count() as f64 / n;
    let bank = rs.iter().filter(|r| r.tier == ExitTier::Bank && !r.timed_out).count() as f64 / n;
    let ticks: f64 = rs.iter().map(|r| r.ticks as f64).sum::<f64>().max(1.0);
    let gold: f64 = rs.iter().map(|r| r.loot_kept as f64).sum();
    let per_hour = 3600.0 * riddle_core::offline::TICKS_PER_SECOND as f64;
    let depth = rs.iter().map(|r| r.max_depth as f64).sum::<f64>() / n;
    Read { reach, bank, gold_hr: gold / ticks * per_hour, n: rs.len(), depth }
}

/// The candidate rows the search may add: the stock of `lever::one_row_edits` and, per foe tag
/// of the lanes' kinds, the answers a player writes (`foe: T → attack T | retreat | corridor`),
/// the boss throws, and each row again under `in: <lane>`.
pub fn stock(g: &Game, fork: u32) -> Vec<Row> {
    let vocab = g.vocabulary();
    let i = FORKS.iter().position(|f| *f == fork).unwrap();
    let biomes = [riddle_core::descent::BASE_ORDER[i], riddle_core::descent::BASE_ORDER[i + 1]];
    let mut tags: Vec<&str> = Vec::new();
    for b in biomes {
        for k in riddle_core::defs::biome_kinds(b) {
            for t in riddle_core::defs::monster_def(k).tags {
                if !tags.contains(t) && vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some(*t)) {
                    tags.push(t);
                }
            }
        }
        if let Some(bk) = riddle_core::descent::biome_boss(b) {
            for t in riddle_core::defs::monster_def(bk).tags {
                if !tags.contains(t) && vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some(*t)) {
                    tags.push(t);
                }
            }
        }
    }
    let mut rows = vec![
        Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest")),
        Row::new(vec![Cond::n("hp<", 40)], Verb::new("return")),
        Row::new(vec![Cond::n("hp<", 30)], Verb::new("bank")),
        Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal")),
        Row::new(vec![Cond::n("hp<", 90)], Verb::new("rest")),
        Row::new(vec![Cond::n("foes>=", 3)], Verb::new("back_corridor")),
        Row::new(vec![Cond::n("adj>=", 2)], Verb::new("retreat")),
    ];
    for t in &tags {
        let c = Cond::t("foe_tag", t);
        rows.push(Row::new(vec![c.clone()], Verb::arg("attack", &format!("tag:{t}"))));
        rows.push(Row::new(vec![c.clone(), Cond::n("adj>=", 1)], Verb::new("retreat")));
        rows.push(Row::new(vec![c.clone()], Verb::new("back_corridor")));
    }
    for k in ["fire", "poison", "caustic"] {
        let v = Verb::arg("throw", &format!("{k},tag:boss"));
        if vocab.verbs.iter().any(|x| x.v == "throw" && x.a.as_deref().is_some_and(|a| a.starts_with(k))) {
            rows.push(Row::new(vec![Cond::t("foe_tag", "boss")], v));
        }
    }
    let base: Vec<Row> = rows.clone();
    // (`in: <lane>` needs the lane entered: not at the first fork, seen from D4's stairs)
    for b in biomes.into_iter().filter(|_| fork != FORKS[0]) {
        for r in &base {
            if r.conds.len() < 2 && r.conds.iter().all(|c| c.k != "hp<") {
                let mut conds = r.conds.clone();
                conds.push(Cond::t("in", b.name()));
                rows.push(Row::new(conds, r.verb.clone()));
            }
        }
    }
    rows
}

/// Every one-row edit of `set` (dropped, moved up, a number notched) and every stock row
/// inserted at the top, under the lineage's row cap.
pub fn edits(g: &Game, set: &RuleSet, fork: u32) -> Vec<RuleSet> {
    let mut out: Vec<RuleSet> = Vec::new();
    let n = set.rows.len();
    let mut push = |s: RuleSet| {
        if s != *set && s.validate().is_ok() && !out.contains(&s) {
            out.push(s);
        }
    };
    for i in 0..n {
        if n > 1 {
            let mut s = set.clone();
            s.rows.remove(i);
            push(s);
        }
        for j in 0..i {
            let mut s = set.clone();
            let r = s.rows.remove(i);
            s.rows.insert(j, r);
            push(s);
        }
        for (ci, c) in set.rows[i].conds.iter().enumerate() {
            let Some(v) = c.n else { continue };
            let step = if c.k.starts_with("hp") { 10 } else { 1 };
            for sign in [-1, 1] {
                let nv = v + sign * step;
                if nv < 1 || (c.k.starts_with("hp") && nv > 95) {
                    continue;
                }
                let mut s = set.clone();
                s.rows[i].conds[ci].n = Some(nv);
                push(s);
            }
        }
    }
    let own = set.own_rows();
    for r in stock(g, fork) {
        if set.rows.contains(&r) {
            continue;
        }
        if own < g.lineage.max_rows() {
            for at in [0usize, n.saturating_sub(1)] {
                let mut s = set.clone();
                s.rows.insert(at.min(s.rows.len()), r.clone());
                push(s);
            }
        }
        // In place of any own row (the cap's trade).
        for i in 0..n {
            if set.rows[i].is_card() {
                continue;
            }
            let mut s = set.clone();
            s.rows[i] = r.clone();
            push(s);
        }
    }
    // Every row scoped to one lane (`· in: <biome>`), where it has room for a cond.
    let i0 = FORKS.iter().position(|f| *f == fork).unwrap();
    for b in [riddle_core::descent::BASE_ORDER[i0], riddle_core::descent::BASE_ORDER[i0 + 1]].into_iter().filter(|_| fork != FORKS[0]) {
        for i in 0..n {
            if set.rows[i].is_card() || set.rows[i].conds.len() >= 2 || set.rows[i].conds.iter().any(|c| c.k == "in") {
                continue;
            }
            let mut s = set.clone();
            s.rows[i].conds.push(Cond::t("in", b.name()));
            push(s);
        }
    }
    out
}

/// The plateau search on one lane: from `seed_set`, take the best edit (screened on `n / 2`
/// sends, confirmed on `n`) while it gains ≥ `min_gain` reach; at most `steps` steps. Returns
/// the set and its read on `n` sends.
pub fn climb(g: &Game, seed_set: &RuleSet, route: Route, fork: u32, n: u32, steps: u32, min_gain: f64) -> (RuleSet, Read) {
    let mut cur = seed_set.clone();
    let mut cur_read = read(&sends(g, &cur, route, fork, n), fork);
    let screen = (n / 3).max(16);
    for _ in 0..steps {
        let cands = edits(g, &cur, fork);
        let base_s = read(&sends(g, &cur, route, fork, screen), fork).score();
        let mut scored: Vec<(f64, usize)> = riddle_core::forecast::par_map(g, (0..cands.len()).collect(), |g, &i| (read(&sends(g, &cands[i], route, fork, screen), fork).score() - base_s, i));
        scored.sort_by(|a, b| b.0.total_cmp(&a.0));
        let top: Vec<usize> = scored.iter().take(4).filter(|(d, _)| *d >= min_gain / 2.0).map(|(_, i)| *i).collect();
        let reads: Vec<Read> = riddle_core::forecast::par_map(g, top.clone(), |g, &i| read(&sends(g, &cands[i], route, fork, n), fork));
        let best = top.iter().zip(reads).filter(|(_, r)| r.score() >= cur_read.score() + min_gain).max_by(|a, b| a.1.score().total_cmp(&b.1.score()));
        match best {
            Some((&i, r)) => {
                cur = cands[i].clone();
                cur_read = r;
            }
            None => break,
        }
    }
    (cur, cur_read)
}

/// The seed sets of the search: EDITED (`probes::good`) and every cohort set.
pub fn seed_sets() -> Vec<(String, RuleSet)> {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../eval/cards");
    let mut out: Vec<(String, RuleSet)> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let stem = name.strip_suffix(".rules.json")?.to_string();
                    Some((stem, RuleSet::parse(&std::fs::read_to_string(e.path()).ok()?).ok()?))
                })
                .collect()
        })
        .unwrap_or_default();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.insert(0, ("EDITED".into(), riddle_core::probes::good()));
    out
}

/// One lane's search result: the seed set it climbed from, the set, its read.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Found {
    pub fork: u32,
    pub lane: usize,
    pub seed_set: String,
    pub set: RuleSet,
    pub reach: f64,
}

/// The gate's reads of the per-lane best sets on paired seeds: per lane, the best set's reach
/// on its own lane and on the other, and gold/hr on its own.
#[derive(Clone, Debug, Default)]
pub struct Gate {
    pub own: [f64; 2],
    pub cross: [f64; 2],
    pub gold: [f64; 2],
    /// EDITED's best per lane, on its own lane.
    pub edited: [f64; 2],
}

impl Gate {
    /// Cross-lane loss of lane `l`'s best set: the other lane's best minus it, on the other lane.
    pub fn loss(&self, l: usize) -> f64 {
        self.own[1 - l] - self.cross[l]
    }
    pub fn gold_ratio(&self) -> f64 {
        self.gold[0].max(self.gold[1]) / self.gold[0].min(self.gold[1]).max(1.0)
    }
}

/// One lineage seed's reads of every candidate the search found at `fork` (deduplicated by
/// set and lane): per candidate (its index in `found`), its reach on its own lane, on the other
/// lane, and its gold/hr on its own. The gate picks each lane's best on the pooled reads (the
/// search's own 48-sim numbers carry a winner's curse; the gate reads every candidate afresh).
pub fn gate_seed(found: &[Found], fork: u32, seed: u64, n: u32) -> Vec<(usize, f64, f64, f64)> {
    gate_candidates(found, fork).into_iter().map(|i| gate_one(found, fork, seed, n, i)).collect()
}

/// The candidates `gate_seed` reads at `fork`: each distinct (lane, set) once, in `found` order.
pub fn gate_candidates(found: &[Found], fork: u32) -> Vec<usize> {
    (0..found.len()).filter(|&i| found[i].fork == fork && !found[..i].iter().any(|o| o.fork == fork && o.lane == found[i].lane && o.set.rows == found[i].set.rows)).collect()
}

/// One candidate's read on one seed (its own game): `gate_seed`'s entry for `i` — the metrics
/// pool runs these as jobs of their own (a seed's 21 candidates were one 3-minute job).
pub fn gate_one(found: &[Found], fork: u32, seed: u64, n: u32, i: usize) -> (usize, f64, f64, f64) {
    let lanes = lanes(fork);
    let f = &found[i];
    let game = lane_game(&f.set, seed, fork);
    let set = game.lineage.rules().clone();
    let own = read(&sends(&game, &set, lanes[f.lane], fork, n), fork);
    let cross = read(&sends(&game, &set, lanes[1 - f.lane], fork, n), fork);
    (i, own.reach, cross.reach, own.gold_hr)
}

/// The gate from the pooled `gate_seed` reads: each lane's best candidate (the highest mean own
/// reach; a tie goes to the one found first), its reach on the other lane, gold/hr, and EDITED's
/// best per lane. Returns the gate and the two winners' indices in `found`.
pub fn gate_pool(found: &[Found], fork: u32, reads: &[Vec<(usize, f64, f64, f64)>]) -> Option<(Gate, [usize; 2])> {
    let k = reads.len().max(1) as f64;
    let mut mean: std::collections::BTreeMap<usize, (f64, f64, f64)> = Default::default();
    for r in reads {
        for &(i, own, cross, gold) in r {
            let e = mean.entry(i).or_default();
            e.0 += own / k;
            e.1 += cross / k;
            e.2 += gold / k;
        }
    }
    let best = |l: usize, only: Option<&str>| -> Option<usize> {
        mean.iter().filter(|(i, _)| found[**i].fork == fork && found[**i].lane == l && only.is_none_or(|o| found[**i].seed_set == o)).fold(None, |b: Option<(usize, f64)>, (i, m)| if b.is_none_or(|b| m.0 > b.1 + 1e-9) { Some((*i, m.0)) } else { b }).map(|b| b.0)
    };
    let (b0, b1) = (best(0, None)?, best(1, None)?);
    let mut g = Gate::default();
    for (l, b) in [(0usize, b0), (1, b1)] {
        g.own[l] = mean[&b].0;
        g.cross[l] = mean[&b].1;
        g.gold[l] = mean[&b].2;
        g.edited[l] = best(l, Some("EDITED")).map(|e| mean[&e].0).unwrap_or(g.own[l]);
    }
    Some((g, [b0, b1]))
}

/// Cut 26 §3: rule-set diversity — distinct row multisets (order and origin ignored) ÷ the sets
/// that clear the bar (≥ 50 % reach) on their lane.
pub fn diversity(found: &[Found]) -> (usize, usize) {
    let clears: Vec<&Found> = found.iter().filter(|f| f.reach >= 0.5).collect();
    let mut keys: Vec<String> = clears
        .iter()
        .map(|f| {
            let mut rows: Vec<String> = f.set.rows.iter().map(|r| serde_json::to_string(&(&r.conds, &r.verb)).unwrap()).collect();
            rows.sort();
            rows.join("|")
        })
        .collect();
    keys.sort();
    keys.dedup();
    (keys.len(), clears.len())
}
