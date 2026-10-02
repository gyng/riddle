//! Cut 30 §3–5 (docs/CUT30.md, docs/TOWN.md): the town's core — its buildings (each built on its
//! trigger, the next plot staked), the bank (deposits earn interest each night, capped), the four
//! tracks (character · items · scale · town: the stage, the next one and its trigger) and what grew
//! on each over an absence; and the quest board (one plain goal, a reward, a progress bar, no stake).
//! Everything here is derived from the lineage or deterministic on it; the town's walkers are the
//! client's (cosmetic, never read back).
use crate::engine::LineageState;
use crate::wire::{GrewLine, QuestWire, TownWire, TrackWire};
use serde::{Deserialize, Serialize};

/// The buildings v1, in the order the plots are staked, each with its trigger (≤ 3 words).
pub const BUILDINGS: [(&str, &str); 4] = [("blacksmith", "first gold home"), ("storehouse", "first find kept"), ("kennel", "first tame"), ("bank", "a night's purse")];

/// A night's interest on the bank, in percent, and the bank's cap in nights of net income.
pub const BANK_PCT: i32 = 2;
pub const BANK_NIGHTS: i32 = 3;

/// The lineage's town (`LineageState::town`).
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Town {
    /// Buildings built, in the order they were (id, the day built).
    #[serde(default)]
    pub built: Vec<(String, u32)>,
    /// Gold in the bank, and the interest it paid in all.
    #[serde(default)]
    pub bank: i32,
    #[serde(default)]
    pub interest: i32,
    /// The quest on the board (from the Warlord slain), the quests kept, the day of the last free
    /// swap.
    #[serde(default)]
    pub quest: Option<Quest>,
    #[serde(default)]
    pub quests_done: u32,
    #[serde(default)]
    pub swap_day: Option<u32>,
    #[serde(default)]
    pub quest_seq: u32,
    /// A harness's switch (the dayplayer's leave-one-out): the board never opens.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub off: bool,
    /// The day on the lineage's clock inside an absence (the offline loop's, as its sends finish):
    /// `LineageState::day` moves at an absence's start, the board's day at every day it crosses.
    #[serde(skip)]
    pub today: u32,
}

fn triggered(l: &LineageState, id: &str) -> bool {
    match id {
        "blacksmith" => !l.banked_depths.is_empty() || l.gold_ledger.iter().any(|x| x.delta > 0 && crate::engine::is_exit_why(&x.why)),
        "storehouse" => !l.vault.is_empty() || l.facts.contains("vault"),
        "kennel" => l.tamed_kinds() > 0 || l.all_companions().next().is_some(),
        "bank" => l.last_night_net > 0 && l.gold >= l.last_night_net,
        _ => false,
    }
}

pub fn built(l: &LineageState, id: &str) -> bool {
    l.town.built.iter().any(|(b, _)| b == id)
}

/// Build every building whose trigger has happened; the ids built now.
pub fn update(l: &mut LineageState) -> Vec<String> {
    let mut out = Vec::new();
    for (id, _) in BUILDINGS {
        if !built(l, id) && triggered(l, id) {
            let day = l.day;
            l.town.built.push((id.to_string(), day));
            out.push(id.to_string());
        }
    }
    out
}

/// A building's look (1–3): the blacksmith by its forge steps, the bank by its balance.
pub fn level(l: &LineageState, id: &str) -> u32 {
    match id {
        "blacksmith" => {
            let steps: u32 = crate::kit::KIT_SLOTS.iter().map(|s| crate::kit::owned(l, s)).sum();
            1 + (steps >= 3) as u32 + (steps >= 7) as u32
        }
        "bank" => 1 + (l.town.bank >= bank_cap(l) / 2 && l.town.bank > 0) as u32 + (l.town.bank >= bank_cap(l) && l.town.bank > 0) as u32,
        _ => 1,
    }
}

/// The bank holds at most `BANK_NIGHTS` nights of net income (never under 100).
pub fn bank_cap(l: &LineageState) -> i32 {
    (BANK_NIGHTS * l.last_night_net.max(l.last_day_net / 4)).max(100)
}

/// Deposit gold (refused before the bank is built; capped).
pub fn deposit(l: &mut LineageState, amount: i32) -> Result<i32, String> {
    if !built(l, "bank") {
        return Err("no bank yet".into());
    }
    let room = (bank_cap(l) - l.town.bank).max(0);
    let n = amount.min(crate::tree::purse(l)).min(room).max(0);
    if n == 0 {
        return Err(if room == 0 { "bank full".into() } else { "no gold".into() });
    }
    l.gold_move(-n, "bank deposit");
    l.town.bank += n;
    Ok(n)
}

/// Withdraw gold (all of it at most).
pub fn withdraw(l: &mut LineageState, amount: i32) -> Result<i32, String> {
    let n = amount.min(l.town.bank).max(0);
    if n == 0 {
        return Err("bank empty".into());
    }
    l.town.bank -= n;
    l.gold_move(n, "bank withdraw");
    Ok(n)
}

/// A night ended: the bank pays its interest (never negative; the balance may pass the cap by it).
pub fn night(l: &mut LineageState) -> i32 {
    let i = (l.town.bank.max(0) * BANK_PCT / 100).max(0);
    l.town.bank += i;
    l.town.interest += i;
    l.tree.ledger += i as i64;
    i
}

// ---------------------------------------------------------------- tracks

/// A track's stages v1 (docs/CUT30.md §4), each with its trigger.
pub fn stages(track: &str) -> &'static [(&'static str, &'static str)] {
    match track {
        // (each band boss's drill is a stage of the hero's: the idle climb's own milestones)
        "character" => &[
            ("Steady", ""),
            ("second stance", "meet Captain"),
            ("Warlord drilled", "meet Warlord twice"),
            ("a tactic", "slay Warlord"),
            ("pets", "first stray"),
            ("a class", "first bank"),
            ("a temperament", "heir 3"),
            ("the pen", "meet Mother"),
            ("Mother drilled", "meet Mother twice"),
            ("tactic slot 2", "meet Lich"),
            ("Lich drilled", "meet Lich twice"),
            ("Foundry drilled", "meet golems twice"),
            ("Queen drilled", "meet Queen twice"),
            ("King drilled", "meet King twice"),
        ],
        "items" => &[("pack of 3", ""), ("storehouse", "first find kept"), ("blacksmith steps", "first gold home"), ("a counter packed", "a drill's item")],
        // (a waystone lit deeper is the scale's next stage: the sends can start there)
        // (each band boss slain is a stage too: the descent opens past him for good)
        "scale" => &[("one hero", ""), ("party slot 2", "a second slot"), ("waystones", "slay Warlord"), ("Mother slain", "slay Mother"), ("Lich slain", "slay Lich"), ("Master slain", "slay Master"), ("Queen slain", "slay Queen"), ("the bottom", "reach D33"), ("waystone D14", "bank at D14"), ("waystone D19", "bank at D19"), ("waystone D24", "bank at D24"), ("waystone D29", "bank at D29"), ("party slots 3–4", "a fourth slot")],
        "town" => &[("camp", ""), ("blacksmith", "first gold home"), ("storehouse", "first find kept"), ("kennel", "first tame"), ("bank", "a night's purse")],
        _ => &[],
    }
}

pub const TRACKS: [&str; 4] = ["character", "items", "scale", "town"];

/// Whether a track's stage has come for this lineage.
pub fn reached(l: &LineageState, track: &str, stage: &str) -> bool {
    let met = |k: &str| l.pkg.meets.get(k).copied().unwrap_or(0) > 0 || l.facts.contains(&format!("foe:{k}")) || l.kills.contains(k);
    match (track, stage) {
        (_, "Steady" | "pack of 3" | "one hero" | "camp") => true,
        ("character", "second stance") => met("goblin_captain") || met("goblin_warlord"),
        ("character", "a tactic") | ("scale", "waystones") => l.kills.contains("goblin_warlord"),
        ("character", "pets") => l.facts.contains("stray") || l.all_companions().next().is_some(),
        ("character", "a class") => l.classes.iter().any(|(k, c)| k != "fighter" && (c.xp > 0 || c.level > 1)) || ["rogue", "ranger", "caster"].iter().any(|c| l.unlocks.contains(*c)),
        ("character", "a temperament") => crate::packages::temperament_open(l),
        ("character", "tactic slot 2") => crate::packages::tactic_slots(l) >= 2,
        ("character", "the pen") => l.pkg.pen_open,
        ("items", "storehouse") | ("town", "storehouse") => built(l, "storehouse"),
        ("items", "blacksmith steps") => crate::kit::KIT_SLOTS.iter().any(|s| crate::kit::owned(l, s) > 0) || built(l, "blacksmith"),
        ("items", "a counter packed") => !crate::packages::quartermaster(l).is_empty() || l.pkg.drills.iter().any(|d| crate::packages::drill_item(d).is_some()),
        ("character", d) if d.ends_with(" drilled") => {
            let boss = match d.trim_end_matches(" drilled") {
                "Warlord" => "goblin_warlord",
                "Mother" => "bloat_mother",
                "Lich" => "lich",
                "Foundry" => "foundry_master",
                "Queen" => "lurker_queen",
                _ => "mirror_king",
            };
            l.pkg.drills.iter().any(|x| x.boss == boss)
        }
        ("scale", s) if s.ends_with(" slain") => {
            let boss = match s.trim_end_matches(" slain") {
                "Mother" => "bloat_mother",
                "Lich" => "lich",
                "Master" => "foundry_master",
                _ => "lurker_queen",
            };
            l.kills.contains(boss)
        }
        ("scale", w) if w.starts_with("waystone D") => w.trim_start_matches("waystone D").parse::<u32>().is_ok_and(|d| l.stones().contains(&d)),
        // (the King's floor: the descent's last band seen)
        ("scale", "the bottom") => l.best_depth >= crate::descent::BANDS[crate::descent::BANDS.len() - 1].1,
        ("scale", "party slot 2") => l.party_slots() >= 2,
        ("scale", "party slots 3–4") => l.party_slots() >= 4,
        ("town", b) => built(l, b),
        _ => false,
    }
}

/// The four tracks on the wire: each one's stage (the last reached in order), its next stage and
/// trigger, and progress toward it when the trigger is numeric.
pub fn tracks(l: &LineageState) -> Vec<TrackWire> {
    TRACKS
        .iter()
        .map(|t| {
            let st = stages(t);
            let done: Vec<bool> = st.iter().map(|(s, _)| reached(l, t, s)).collect();
            let stage = st.iter().zip(&done).filter(|(_, d)| **d).map(|((s, _), _)| *s).next_back().unwrap_or(st[0].0);
            let next = st.iter().zip(&done).find(|(_, d)| !**d).map(|((s, tr), _)| (*s, *tr));
            let progress = match next.map(|n| n.0) {
                Some("bank") => (l.last_night_net > 0).then(|| (l.gold as f64 / l.last_night_net as f64).clamp(0.0, 1.0)),
                Some("a temperament") => Some((l.heir as f64 / 3.0).min(1.0)),
                _ => None,
            };
            TrackWire { id: t.to_string(), stage: stage.into(), stages: done.iter().filter(|d| **d).count() as u32, next: next.map(|n| n.0.to_string()), trigger: next.map(|n| n.1.to_string()), progress }
        })
        .collect()
}

/// The stages reached on every track (for `grew`: a stage that opened between two looks).
pub fn stage_set(l: &LineageState) -> Vec<(String, String)> {
    let mut v = Vec::new();
    // (every package that arrives is a stage of the character's: a stance, a tactic of the drip, a
    // temperament — §2 "each arrives on a stage trigger")
    for id in l.pkg.owned.iter().filter(|id| crate::packages::def(id).is_some() && id.as_str() != "steady") {
        v.push(("character".to_string(), format!("+{}", crate::packages::name(id))));
    }
    // (a building stepped to its next look is a stage of the town's: the forge's steps, the bank's fill)
    for (b, _) in &l.town.built {
        for look in 2..=level(l, b) {
            v.push(("town".to_string(), format!("{b} look {look}")));
        }
    }
    for t in TRACKS {
        for (s, _) in stages(t) {
            if reached(l, t, s) {
                v.push((t.to_string(), s.to_string()));
            }
        }
    }
    v
}

/// A lineage's state as `grew` compares it (before and after an absence).
#[derive(Clone, Debug, Default)]
pub struct Snap {
    pub xp: u64,
    pub class_level: u32,
    /// Each package's level (id, level).
    pub pkg_level: Vec<(String, u32)>,
    pub gold: i64,
    pub kit: u32,
    pub finds: usize,
    pub best: u32,
    pub party: usize,
    pub buildings: usize,
    pub bank: i32,
    pub stages: Vec<(String, String)>,
    pub drills: usize,
}

pub fn snap(l: &LineageState) -> Snap {
    Snap {
        xp: l.classes.values().map(|c| c.xp as u64 + 1000 * c.level as u64).sum(),
        class_level: l.class_level(),
        pkg_level: l.pkg.runs.keys().map(|k| (k.clone(), l.pkg.level(k))).collect(),
        gold: l.gold as i64 + l.town.bank as i64,
        kit: crate::kit::KIT_SLOTS.iter().map(|s| crate::kit::owned(l, s)).sum(),
        finds: l.vault.len() + l.found_kinds.len(),
        best: l.best_depth,
        party: l.all_companions().count(),
        buildings: l.town.built.len(),
        bank: l.town.bank,
        stages: stage_set(l),
        drills: l.pkg.drills.len(),
    }
}

/// What grew on each track between two snaps (the report leads with it). Per track, the most
/// important first — a new stage, a package that arrived, a level, then xp and gold — each fact once
/// (no `opened` prefix: the stage's name is the news; a stage another line already says is left out).
pub fn grew(a: &Snap, b: &Snap) -> Vec<GrewLine> {
    let new: Vec<&(String, String)> = b.stages.iter().filter(|s| !a.stages.contains(s)).collect();
    let has = |t: &str, s: &str| new.iter().any(|(tt, ss)| tt == t && ss == s);
    let arrived = new.iter().any(|(t, s)| t == "character" && s.starts_with('+'));
    let mut out: Vec<GrewLine> = Vec::new();
    for track in TRACKS {
        let mut add = |what: String| out.push(GrewLine { track: track.into(), what });
        // 1 · the stages (a package's arrival after them; its slot's stage is the same news)
        for (t, s) in new.iter().filter(|(t, s)| t == track && !s.starts_with('+')) {
            let said = match (t.as_str(), s.as_str()) {
                ("character", "second stance" | "a tactic" | "a temperament") => arrived,
                // (the town's row names the building)
                ("items", "storehouse") => true,
                ("items", "blacksmith steps") => has("town", "blacksmith"),
                _ => false,
            };
            if !said {
                add(s.clone());
            }
        }
        match track {
            "character" => {
                for (_, s) in new.iter().filter(|(t, s)| t == "character" && s.starts_with('+')) {
                    add(s.clone());
                }
                if b.class_level > a.class_level {
                    add(format!("L{}", b.class_level));
                }
                for (id, lv) in &b.pkg_level {
                    if a.pkg_level.iter().find(|(k, _)| k == id).map_or(1, |(_, l)| *l) < *lv {
                        add(format!("{} L{lv}", crate::packages::name(id)));
                    }
                }
                if b.drills > a.drills && !new.iter().any(|(t, s)| t == "character" && s.ends_with(" drilled")) {
                    add("drilled".into());
                }
                if b.class_level <= a.class_level && b.xp > a.xp {
                    add("xp".into());
                }
            }
            "items" => {
                if b.gold > a.gold {
                    add(format!("+${}", b.gold - a.gold));
                }
                if b.finds > a.finds {
                    add("a find".into());
                }
                if b.kit > a.kit {
                    add("blacksmith step".into());
                }
            }
            "scale" => {
                if b.best > a.best {
                    add(format!("best D{}", b.best));
                }
                if b.party > a.party {
                    add("a companion".into());
                }
            }
            _ => {
                if b.bank > a.bank {
                    add("interest".into());
                }
            }
        }
    }
    out
}

/// The town on the wire.
pub fn wire(l: &LineageState) -> TownWire {
    let next = BUILDINGS.iter().find(|(id, _)| !built(l, id)).map(|(id, tr)| (id.to_string(), tr.to_string()));
    TownWire {
        buildings: l.town.built.iter().map(|(id, day)| crate::wire::BuildingWire { id: id.clone(), level: level(l, id), day: *day }).collect(),
        next: next.as_ref().map(|n| n.0.clone()),
        next_trigger: next.map(|n| n.1),
        bank: l.town.bank,
        bank_cap: bank_cap(l),
        interest: l.town.interest,
        quest: l.town.quest.as_ref().map(|q| quest_wire(l, q)),
        quests_done: l.town.quests_done,
        workers: if l.pkg.literal { Vec::new() } else { crate::tree::posts(l) },
    }
}

// ---------------------------------------------------------------- the quest board

/// A quest: one plain goal, a reward, progress (0..1), the day it was drawn. No stake.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Quest {
    pub id: u32,
    /// `reach` · `reach_no_return` · `bank` · `slay`.
    pub goal: String,
    pub depth: u32,
    #[serde(default)]
    pub boss: Option<String>,
    /// The reward's picture (`title` · `row` · `slot` · `card`).
    pub reward: String,
    /// Best progress so far, permille.
    #[serde(default)]
    pub progress: u32,
    pub day: u32,
    #[serde(default)]
    pub done: bool,
}

/// The board opens with the Warlord slain.
pub fn quests_open(l: &LineageState) -> bool {
    l.kills.contains("goblin_warlord") && !l.town.off && !l.pkg.literal
}

/// The goal line, ≤ 5 words (`reach D10 · no return`).
pub fn goal_text(q: &Quest) -> String {
    match q.goal.as_str() {
        "reach_no_return" => format!("reach D{} · no return", q.depth),
        "bank" => format!("home from D{}", q.depth),
        "slay" => format!("slay the {}", crate::sifter::boss_short(q.boss.as_deref().unwrap_or(""))),
        _ => format!("reach D{}", q.depth),
    }
}

fn quest_wire(l: &LineageState, q: &Quest) -> QuestWire {
    QuestWire { goal: goal_text(q), reward: q.reward.clone(), progress: q.progress as f64 / 1000.0, done: q.done, swap: l.town.swap_day != Some(l.day) }
}

/// The rewards, in the order the board offers them (the first one the lineage can still take).
fn reward_for(l: &LineageState, seq: u32) -> String {
    let rows = ["row5", "row6", "row7", "row8", "row9", "row10"];
    let slots = ["party_slot_2", "party_slot_3", "party_slot_4"];
    let order = ["title", "row", "slot", "card"];
    for k in 0..order.len() {
        let r = order[(seq as usize + k) % order.len()];
        let ok = match r {
            "row" => rows.iter().any(|u| !l.unlocks.contains(*u)),
            "slot" => slots.iter().any(|u| !l.unlocks.contains(*u)),
            _ => true,
        };
        if ok {
            return r.into();
        }
    }
    "title".into()
}

/// Draw the next quest (deterministic on the lineage and its sequence): a goal just past the record
/// or at the next boss.
pub fn draw(l: &mut LineageState) {
    let seq = l.town.quest_seq;
    l.town.quest_seq += 1;
    let mut rng = crate::rng::Rng::derive(l.seed, crate::rng::hash_str("quest") ^ seq as u64);
    let best = l.best_depth.max(1);
    let next_boss = crate::descent::BOSS_DEPTHS.iter().find(|(k, d)| !l.kills.contains(*k) && *d <= best + 2 && *k != "foundry_master" && l.pkg.drills.iter().any(|x| x.boss == *k && !x.revoked)).map(|(k, _)| k.to_string());
    let goal = match (rng.below(4), &next_boss) {
        (0, Some(_)) => "slay",
        (0, None) | (1, _) => "reach",
        (2, _) => "reach_no_return",
        _ => "bank",
    };
    let depth = match goal {
        // (goals a night of the sends from D1 keeps: the record's band, not its last floor)
        "reach" => best.saturating_sub(3).max(2),
        "reach_no_return" => best.saturating_sub(5).max(2),
        // (a cleared floor: the deepest lit waystone at or under the record — banked there before — else three under it)
        "bank" => l.stones().into_iter().filter(|s| *s <= best).max().unwrap_or(best.saturating_sub(5)).max(2),
        _ => crate::descent::boss_depth(next_boss.as_deref().unwrap_or("")).unwrap_or(best),
    };
    let reward = reward_for(l, seq);
    let day = today(l);
    l.town.quest = Some(Quest { id: seq, goal: goal.into(), depth, boss: if goal == "slay" { next_boss } else { None }, reward, progress: 0, day, done: false });
}

/// The board's day: the lineage's, or a later one an absence has crossed.
pub fn today(l: &LineageState) -> u32 {
    l.day.max(l.town.today)
}

/// A new day on the board: a quest kept on an earlier day gives way to the day's draw (from the
/// record as it stands), as does an empty board.
pub fn roll(l: &mut LineageState) {
    if quests_open(l) && l.town.quest.as_ref().is_none_or(|q| q.done && q.day < today(l)) {
        draw(l);
    }
}

/// One free swap a day: a fresh quest in place of the one on the board.
pub fn swap(l: &mut LineageState) -> Result<(), String> {
    if !quests_open(l) {
        return Err("no board yet".into());
    }
    if l.town.swap_day == Some(l.day) {
        return Err("swapped today".into());
    }
    l.town.swap_day = Some(l.day);
    draw(l);
    Ok(())
}

/// A run's progress on the quest (`max_depth`, the exit, whether a return was committed, the bosses
/// it slew); a kept quest pays its reward. The line for the report (`QUEST DONE`), if kept now.
pub fn on_run(l: &mut LineageState, max_depth: u32, tier: crate::engine::ExitTier, returned: bool, _exit_depth: u32, slew: &[String]) -> Option<String> {
    if !quests_open(l) {
        return None;
    }
    // a new quest the day after one was kept (or at the board's first look)
    roll(l);
    let day = today(l);
    let q = l.town.quest.as_mut()?;
    if q.done {
        return None;
    }
    let depth = q.depth.max(1);
    let (p, kept) = match q.goal.as_str() {
        "reach" => ((max_depth * 1000 / depth).min(1000), max_depth >= depth),
        "reach_no_return" => (if returned { 0 } else { (max_depth * 1000 / depth).min(1000) }, !returned && max_depth >= depth),
        // (home from a cleared floor: a bank or a return that reached it)
        "bank" => ((max_depth * 1000 / depth).min(999), tier != crate::engine::ExitTier::Death && max_depth >= depth),
        _ => {
            let b = q.boss.clone().unwrap_or_default();
            ((max_depth * 900 / depth).min(900), slew.contains(&b))
        }
    };
    q.progress = q.progress.max(if kept { 1000 } else { p });
    if !kept {
        return None;
    }
    q.done = true;
    q.day = day;
    let reward = q.reward.clone();
    let text = goal_text(q);
    l.town.quests_done += 1;
    grant(l, &reward, &text);
    Some(format!("QUEST DONE · {text}"))
}

fn grant(l: &mut LineageState, reward: &str, text: &str) {
    match reward {
        "row" => {
            if let Some(u) = ["row5", "row6", "row7", "row8", "row9", "row10"].iter().find(|u| !l.unlocks.contains(**u)) {
                l.unlocks.insert(u.to_string());
            }
        }
        "slot" => {
            if let Some(u) = ["party_slot_2", "party_slot_3", "party_slot_4"].iter().find(|u| !l.unlocks.contains(**u)) {
                l.unlocks.insert(u.to_string());
            }
        }
        "card" => {
            let id = l.pkg.stance.clone();
            let runs = l.pkg.runs.entry(id).or_insert(0);
            *runs = crate::packages::next_at(*runs).unwrap_or(*runs);
        }
        _ => {
            l.titles.push(text.to_string());
            l.marks += 2;
        }
    }
}
