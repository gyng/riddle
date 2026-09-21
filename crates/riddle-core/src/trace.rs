//! Death records: trace, margin, morgue, and the verdict (gap/dice) with candidate patches
//! found by replaying the last ~100 ticks with one added row from the unlocked vocabulary.
//!
//! Candidates are family-shaped (Cut 2 §6): a row or two per family — retreat, consumable,
//! escape, dive, targeting, ally, ID policy, terrain — with arguments the hero actually had:
//! items in the checkpoint pack, tags among the foes it met in its last ten actions.
use crate::defs::{monster_def, Cat};
use crate::engine::{DeathRec, ExitTier, Game, Root, Run};
use crate::item::Flavours;
use crate::provenance::ProvKind;
use crate::rng::{splitmix, Rng};
use crate::rules::{Cond, Row, RuleSet, Vocabulary};
use crate::wire::{Because, Death, Ev, Patch, PatchRoot, Trace};
use std::collections::{BTreeMap, BTreeSet};

pub const REPLAYS: u32 = 12;
pub const SURVIVE_BAR: f64 = 0.6;
/// The verdict replay starts at least this many hero turns before death.
pub const MIN_WINDOW: usize = 8;
/// A patch row must fire in this share of its replays to count as tested at all.
pub const FIRED_BAR: f64 = 0.5;
/// How many survival-ranked candidates get a full forecast delta before the final cut.
pub const DELTA_CANDIDATES: usize = 6;
/// A patch must beat the unpatched baseline by this much to count as the fix.
pub const PATCH_MARGIN: f64 = 0.15;
pub const TRACE_LEN: usize = 10;
pub const MAX_CANDIDATES: usize = 16;
pub const HP_THRESHOLDS: [i32; 4] = [20, 30, 40, 50];
/// Targeting rows offered per death (boss first).
pub const TARGET_TAGS: usize = 4;
/// An awake, once-seen foe this close at death counts as met even if out of sight at the end.
pub const CONTEXT_RANGE: i32 = 8;
/// Patches shown on the death screen.
pub const SHOWN: usize = 3;
/// Cut 4: a replay runs on past the killing blow while an awake hostile is in view, at most
/// this many ticks (surviving the moment of the blow is not surviving the fight).
pub const ENCOUNTER_TICKS: u32 = 300;
/// Cut 11 §4: on a `dice` death with no patch over the bar, candidates are measured in full
/// (no early exit), one family each, until this many have fired in half their replays; the
/// best are kept, flagged `below_bar`.
pub const DICE_CANDIDATES: usize = 3;
/// Cut 11 §2: a root-cause patch's `root.text` is at most this many words.
pub const ROOT_WORDS: usize = 6;

pub fn death_record(game: &Game, run: &Run) -> DeathRec {
    record(game, run, false)
}

/// Cut 13 §1: a stalled run's record — a death-style screen for the most expensive outcome
/// in the game (both cohort-9 raters). The cause is the guard's moment (`stalled · goblin
/// archer, no path`), the margin `keeps $0`, the trace and the row accounting the run's; the
/// checkpoint is the first guard's tick and a replay "survives" when it leaves the floor or
/// the guard stays quiet (`Replayer`). `Death.verdict` reads `stall`, never `gap` / `dice`.
pub fn stall_record(game: &Game, run: &Run) -> DeathRec {
    record(game, run, true)
}

fn record(game: &Game, run: &Run, stall: bool) -> DeathRec {
    let turns: Vec<_> = run.trace.iter().rev().take(TRACE_LEN).rev().cloned().collect();
    let provenance = crate::provenance::all(&game.prov);
    let chain = chain_of(turns.last());
    let root = if stall { None } else { root_of(&game.prov, game.lineage.rules(), turns.last()) };
    let cause = if stall {
        format!("stalled · {}", run.stuck_cause.as_deref().unwrap_or("paced"))
    } else {
        run.death_cause.clone().unwrap_or_else(|| "unknown".into())
    };
    // Cut 10 §3: `3 hp short` (was `3 over`, which no rater could read): the HP that would have
    // kept the hero standing through the killing blow. A stall keeps nothing: `keeps $0`.
    let mut margin = if stall { "keeps $0".to_string() } else { format!("{} hp short", run.death_short.max(1)) };
    let facts = &game.lineage.facts;
    let fl = &game.lineage.flavours;
    if !stall && run.hero.inv.iter().any(|i| i.kind == "heal" && i.is_known(facts, fl)) {
        margin.push_str(" · heal unused");
    }
    let unknown = run.hero.inv.iter().filter(|i| i.is_consumable() && !i.is_known(facts, fl)).count();
    if !stall && unknown > 0 {
        margin.push_str(&format!(" · {unknown} unknown unused"));
    }
    let rules = game.lineage.rules().clone();
    let vocab = context_vocab(game, run);
    let death = Death {
        run_id: run.id,
        depth: run.depth,
        cause: cause.clone(),
        margin,
        verdict: if stall { "stall" } else { "dice" }.into(),
        baseline: 0.0,
        trace: Trace { turns, provenance },
        patches: Vec::new(),
        morgue: morgue(game, run, &rules, stall),
        line: None,
        chain,
        rules: Some(rules.clone()),
        // The beat the morgue alone carried — not the headline's own notes (`Slain by …`,
        // `Down to 2 HP.`, `Returned with $0.`), which the screen already says; a stall's note
        // stays (it says what the stall paid).
        notes: run.notes.iter().rev().filter(|(_, n)| !(n.starts_with("Slain by") || n.starts_with("Down to ") || n.starts_with("Returned with") || n.starts_with("Lost the thread") || n.ends_with(": studied.") || n.starts_with("Met a ") || n.starts_with("Learned"))).take(2).map(|(_, n)| n.clone()).collect::<Vec<_>>().into_iter().rev().collect(),
    };
    let n = game.history.len();
    let pick = if stall {
        // Cut 13 §1: the stall's checkpoint is the first guard's tick — the last history entry
        // at or before it. The ring holds `HISTORY_TURNS × HISTORY_STRIDE` ticks and the three
        // guards span ≥ 60 hero actions, so the first guard is usually older than the ring:
        // the oldest entry stands in for it (the replay still starts inside the pacing).
        let first = run.stuck_first_t.unwrap_or(0);
        game.history.iter().rposition(|(r, _)| r.turn <= first).unwrap_or(0).min(n.saturating_sub(MIN_WINDOW))
    } else {
        // Checkpoint: the most recent history entry where the hero still had ≥ 50% HP, but at
        // least MIN_WINDOW turns before death so a patch has room to act; else the oldest entry.
        game.history
            .iter()
            .enumerate()
            .filter(|(i, _)| i + MIN_WINDOW <= n)
            .filter(|(_, (r, _))| r.hero.hp_pct() >= 50)
            .map(|(i, _)| i)
            .next_back()
            .unwrap_or(0)
    };
    let (t10, t10_facts) = match game.history.get(pick) {
        Some((r, f)) => (Some(r.clone()), f.clone()),
        None => (None, BTreeSet::new()),
    };
    // The kill counts as they stood at the checkpoint: the lineage's now, less this run's
    // kills after it (`Run.kills` carries the tick of each).
    let t10_kill_counts = t10.as_ref().map(|c| {
        let mut counts = game.lineage.kill_counts.clone();
        for (t, kind, _) in run.kills.iter().filter(|(t, _, _)| *t > c.turn) {
            let _ = t;
            if let Some(n) = counts.get_mut(kind) {
                *n = n.saturating_sub(1);
            }
        }
        counts
    });
    let t10_lineage = t10.as_ref().map(|_| crate::engine::CheckpointLineage::of(&game.lineage));
    let boss = if stall { None } else { boss_of(run, &cause) };
    DeathRec { death, t10, t10_facts, rules, vocab, verdict_done: false, deltas_done: false, deltas_n: 0, shaped: false, death_tick: run.turn, boss, counter: None, root, stall, t10_kill_counts, t10_lineage }
}

/// Cut 11 §2: the chain — the `because` entries of the killing turn's rows, in row order,
/// deduped by text.
fn chain_of(last: Option<&crate::wire::TraceTurn>) -> Option<Vec<Because>> {
    let rows = last?.rows.as_ref()?;
    let mut out: Vec<Because> = Vec::new();
    for b in rows.iter().filter_map(|w| w.because.as_ref()) {
        if !out.iter().any(|o| o.text == b.text) {
            out.push(b.clone());
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Cut 11 §2: the killing turn's root when a row above the fired one was emptied by a theft
/// (the first such row) or reads `locked cond` (the first such row; a theft root wins).
pub fn root_of(prov: &[crate::provenance::Prov], rules: &RuleSet, last: Option<&crate::wire::TraceTurn>) -> Option<Root> {
    let rows = last?.rows.as_ref()?;
    for w in rows {
        let Some(b) = &w.because else { continue };
        let theft = prov.iter().any(|p| p.kind == ProvKind::Stolen && p.t == b.t && p.text == b.text);
        if theft {
            // `den took the heal, D3` → `den took the heal` (≤ 6 words).
            let text = b.text.split(", D").next().unwrap_or(&b.text).to_string();
            return Some(Root { kind: "theft".into(), text, row: w.row, unlock: None });
        }
    }
    for w in rows {
        if w.why != "locked cond" {
            continue;
        }
        let Some(b) = &w.because else { continue };
        // The set's own row (a lent row's lock is the shrine's, not the player's to buy).
        let Some(unlock) = rules.rows.get(w.row).and_then(|r| r.conds.iter().find_map(|c| crate::meta::cond_unlock(&c.k))) else { continue };
        return Some(Root { kind: "lock".into(), text: b.text.clone(), row: w.row, unlock: Some(unlock.to_string()) });
    }
    None
}

/// Cut 6 §5: the boss the hero died under — the cause when it is a boss kind, else a living
/// boss in view or awake within `CONTEXT_RANGE`.
fn boss_of(run: &Run, cause: &str) -> Option<String> {
    if monster_def(cause).boss {
        return Some(cause.to_string());
    }
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let near = run
        .monsters
        .iter()
        .find(|m| m.hp > 0 && m.is_boss() && (map.is_visible(m.pos) || (m.awake && m.pos.cheb(hp) <= CONTEXT_RANGE)))
        .map(|m| m.kind.clone());
    // Cut 10 §2: a death on a boss's own floor while the boss lives is a wall death whatever
    // took the blow (his rallied goblins, out of his sight) — the counter still gets its pin.
    near.or_else(|| run.monsters.iter().find(|m| m.hp > 0 && m.is_boss() && crate::descent::boss_depth(&m.kind) == Some(run.depth)).map(|m| m.kind.clone()))
}

/// Cut 10 §2: the boss counter row a death's patches pin at the top, when the boss is known,
/// its counter fact held, its verb in the lineage's vocabulary (the whole of it, not the
/// death's context cut — a boss out of sight at the end is exactly the death whose answer is
/// his row) and no row of the set already carries that verb (a set that has the row
/// somewhere is not patched with a second copy). Whether it fires from the checkpoint is the
/// replays' call (`FIRED_BAR`).
pub fn pinnable_counter(game: &Game, rec: &DeathRec) -> Option<Row> {
    let kind = rec.boss.clone()?;
    let row = crate::facts::boss_counter_row(&game.lineage.facts, &kind)?;
    if has_counter_verb(&rec.rules, &row) || !game.vocabulary().verbs.contains(&row.verb) {
        return None;
    }
    Some(row)
}

/// A row with the counter's verb (`attack tag:boss` under any conditions) is in the set.
pub fn has_counter_verb(rules: &RuleSet, counter: &Row) -> bool {
    // A card row counts by the rows it carries (QA on e0f87e7: `try: attack boss` beside an
    // owned `boss focus` whose first row is `foe: boss → attack boss`).
    rules.rows.iter().any(|r| r.verb == counter.verb || r.card().and_then(crate::meta::unlock_rows).is_some_and(|rows| rows.iter().any(|x| x.verb == counter.verb)))
}

fn morgue(game: &Game, run: &Run, rules: &RuleSet, stall: bool) -> String {
    let mut s = String::new();
    s.push_str(&format!("Riddle morgue · seed {} · heir {} · run {}\n", game.lineage.seed, run.heir, run.id));
    if stall {
        s.push_str(&format!("D{} ({}) · tick {} · stalled · {} · ${} carried, kept $0\n", run.depth, run.biome().name(), run.turn, run.stuck_cause.as_deref().unwrap_or("paced"), run.loot.max(0)));
    } else {
        s.push_str(&format!(
            "D{} ({}) · tick {} · slain by {} · blow {} · {} hp short\n",
            run.depth,
            run.biome().name(),
            run.turn,
            run.death_cause.clone().unwrap_or_default(),
            run.death_blow,
            run.death_short.max(1)
        ));
    }
    s.push_str(&format!("class {} L{} · trait {}\n", run.hero.class.name(), run.hero.level, run.trait_.name()));
    s.push_str("rules:\n");
    for (i, r) in rules.rows.iter().enumerate() {
        s.push_str(&format!("  R{} {}\n", i + 1, r.describe()));
    }
    s.push_str("trace:\n");
    for t in run.trace.iter().rev().take(TRACE_LEN).rev() {
        let row = match t.row {
            -1 => "trait".to_string(),
            -2 => "chore".to_string(),
            r => format!("R{}", r + 1),
        };
        // Cut 6 §3: the rows above the one that acted, with their reasons.
        let rows = t.rows.as_ref().map(|r| r.iter().map(|w| format!("R{} {}", w.row + 1, w.why)).collect::<Vec<_>>().join(" · ")).unwrap_or_default();
        s.push_str(&format!("  t{} {} {} hp {} foes {} {} {}\n", t.t, row, t.verb.short(), t.hp, t.foes, t.telegraphs.join(","), rows));
    }
    s.push_str("notes:\n");
    for (t, n) in run.notes.iter().rev().take(8).rev() {
        s.push_str(&format!("  t{t} {n}\n"));
    }
    s.push_str(&format!("rules json: {}\n", serde_json::to_string(rules).unwrap_or_default()));
    s
}

// ---------------------------------------------------------------- what the hero met

/// Tags of the hostiles met over the last ten actions — visible at death, awake and near, or
/// killed since the trace began — and whether a captive was in reach.
fn context(run: &Run, from_t: u32) -> (BTreeSet<String>, bool) {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let mut tags = BTreeSet::new();
    let mut captive = false;
    for m in &run.monsters {
        if m.hp <= 0 {
            continue;
        }
        let near = map.is_visible(m.pos) || (m.awake && run.seen_ids.contains(&m.id) && m.pos.cheb(hp) <= CONTEXT_RANGE);
        if !near {
            continue;
        }
        if m.neutral {
            captive = true;
        } else if m.hostile() {
            tags.extend(m.tags());
        }
    }
    for (t, kind, _) in &run.kills {
        if *t >= from_t {
            tags.extend(monster_def(kind).tags.iter().map(|s| s.to_string()));
        }
    }
    (tags, captive)
}

/// The unlocked vocabulary cut to this death: foe tags only for foes the hero met in its last
/// ten actions, `free_captive` only with a captive in reach. Candidates never name what was
/// not there.
fn context_vocab(game: &Game, run: &Run) -> Vocabulary {
    let mut vocab = game.vocabulary();
    let from_t = run.trace.get(run.trace.len().saturating_sub(TRACE_LEN)).map(|t| t.t).unwrap_or(run.turn);
    let (tags, captive) = context(run, from_t);
    vocab.conds.retain(|c| c.k != "foe_tag" || c.t.as_ref().is_some_and(|t| tags.contains(t)));
    vocab.verbs.retain(|v| match (v.v.as_str(), v.a.as_deref()) {
        ("attack" | "tame", Some(a)) if a.starts_with("tag:") => tags.contains(&a[4..]),
        ("free_captive", _) => captive,
        _ => true,
    });
    vocab
}

// ---------------------------------------------------------------- candidates

/// The family a candidate row belongs to; the death screen shows at most one per family.
pub fn family(row: &Row) -> &'static str {
    let a = row.verb.a.as_deref().unwrap_or("");
    let gas = row.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some("gas"));
    match row.verb.v.as_str() {
        "retreat" | "back_corridor" if gas => "terrain",
        "retreat" | "back_corridor" | "vanish" | "smoke" | "shadowstep" => "retreat",
        "drink" if a == "unknown" => "id",
        "read" if a == "unknown" => "id",
        "read" if matches!(a, "teleport" | "blink") => "escape",
        "drink" | "read" | "throw" | "second_wind" => "consumable",
        "return" | "bank" => "escape",
        "rest" => "rest",
        "descend" => "dive",
        "free_captive" | "tame" | "recall" | "send" => "ally",
        _ => "targeting",
    }
}

/// The trace shows the set resting already (a rest row fired): no rest candidate then.
fn trace_rules_rest(trace: &Trace) -> bool {
    trace.turns.iter().any(|t| t.row >= 0 && t.verb.v == "rest")
}

/// The hp threshold (from {20, 30, 40, 50}) nearest the hero's hp% at the trace's first row;
/// ties go to the higher one, which fires sooner.
fn hp_threshold(state: &Run, trace: &Trace) -> i32 {
    let pct = match trace.turns.first() {
        Some(t) if state.hero.max_hp > 0 => (t.hp * 100 / state.hero.max_hp).clamp(0, 100),
        _ => state.hero.hp_pct(),
    };
    *HP_THRESHOLDS.iter().min_by_key(|&&n| ((n - pct).abs(), -n)).unwrap_or(&30)
}

/// The candidate rows: one family at a time, ≤ `MAX_CANDIDATES`, every argument something the
/// hero held at the checkpoint or met in the trace, every token in `vocab`.
pub fn candidates(vocab: &Vocabulary, state: &Run, facts: &BTreeSet<String>, flavours: &Flavours, trace: &Trace) -> Vec<Row> {
    let has_cond = |k: &str| vocab.conds.iter().any(|c| c.k == k);
    let has_tag = |t: &str| vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some(t));
    let verb = |v: &str, a: Option<&str>| vocab.verbs.iter().find(|x| x.v == v && x.a.as_deref() == a).cloned();
    let n = hp_threshold(state, trace);
    let low = Cond::n("hp<", n);
    let max_foes = trace.turns.iter().map(|t| t.foes).max().unwrap_or(0);
    let pack = if max_foes >= 2 { 2 } else { 1 };
    let held = |cat: Cat| state.hero.inv.iter().filter(move |i| i.cat() == cat);
    let known = |i: &crate::item::Item| i.is_known(facts, flavours);
    let mut out: Vec<Row> = Vec::new();
    // Rest (Cut 5): a set that never rests wanders worn; `hp<N → rest` is the honest patch for
    // a death a reseeded replay mostly survives (the encounter was luck, the low hp was not).
    // First in the list: on a tie it is among the candidates whose forecast delta is tried.
    if has_cond("hp<") && !trace_rules_rest(trace) {
        if let Some(v) = verb("rest", None) {
            out.push(Row::new(vec![low.clone()], v));
        }
    }
    // Retreat: hp<N · foes>=M → retreat | back_corridor.
    if max_foes >= 1 && has_cond("hp<") && has_cond("foes>=") {
        for v in ["retreat", "back_corridor"] {
            if let Some(v) = verb(v, None) {
                out.push(Row::new(vec![low.clone(), Cond::n("foes>=", pack)], v));
            }
        }
    }
    // Consumables: hp<N → drink heal, else another known benevolent potion in the pack.
    let potions: Vec<String> = held(Cat::Potion).filter(|i| known(i) && i.def().benevolent).map(|i| i.kind.clone()).collect();
    if let Some(k) = potions.iter().find(|k| *k == "heal").or(potions.first()) {
        if let Some(v) = verb("drink", Some(k)) {
            out.push(Row::new(vec![low.clone()], v));
        }
    }
    // Escape: hp<N → return; hp<N → read teleport | blink when held and known.
    if let Some(v) = verb("return", None) {
        out.push(Row::new(vec![low.clone()], v));
    }
    let scrolls: Vec<String> = held(Cat::Scroll).filter(|i| known(i)).map(|i| i.kind.clone()).collect();
    if let Some(k) = ["teleport", "blink"].iter().find(|k| scrolls.iter().any(|s| s == *k)) {
        if let Some(v) = verb("read", Some(k)) {
            out.push(Row::new(vec![low.clone()], v));
        }
    }
    // Dive: floor_seen>=60 → descend.
    if has_cond("floor_seen>=") {
        if let Some(v) = verb("descend", None) {
            out.push(Row::new(vec![Cond::n("floor_seen>=", 60)], v));
        }
    }
    // Targeting: foe_tag:T → attack tag:T, only for tags met in the trace (the vocab was cut to
    // them at record time); boss first.
    if max_foes >= 1 {
        let mut tags: Vec<String> = vocab
            .verbs
            .iter()
            .filter(|v| v.v == "attack")
            .filter_map(|v| v.a.as_deref().and_then(|a| a.strip_prefix("tag:")).map(str::to_string))
            .filter(|t| has_tag(t))
            .collect();
        tags.sort_by_key(|t| t != "boss");
        for t in tags.into_iter().take(TARGET_TAGS) {
            if let Some(v) = verb("attack", Some(&format!("tag:{t}"))) {
                out.push(Row::new(vec![Cond::t("foe_tag", &t)], v));
            }
        }
    }
    // Ally: foes>=1 → free_captive (the vocab keeps the verb only with a captive in reach).
    if has_cond("foes>=") {
        if let Some(v) = verb("free_captive", None) {
            out.push(Row::new(vec![Cond::n("foes>=", 1)], v));
        }
    }
    // ID policy: hp<N → drink | read unknown, only when unknown items were held.
    if has_cond("hp<") {
        if held(Cat::Potion).any(|i| !known(i)) {
            if let Some(v) = verb("drink", Some("unknown")) {
                out.push(Row::new(vec![low.clone()], v));
            }
        }
        if held(Cat::Scroll).any(|i| !known(i)) {
            if let Some(v) = verb("read", Some("unknown")) {
                out.push(Row::new(vec![low.clone()], v));
            }
        }
    }
    // Terrain: foe_tag:gas · adj>=1 → retreat (step back from the bloat before it bursts).
    if has_tag("gas") && has_cond("adj>=") {
        if let Some(v) = verb("retreat", None) {
            out.push(Row::new(vec![Cond::t("foe_tag", "gas"), Cond::n("adj>=", 1)], v));
        }
    }
    // Cut 11 §4: a telegraph preceded the blow — `foe_tag:telegraph → retreat` is a named
    // alternative even when it ends under the bar (`dice` is never empty).
    if let Some(row) = telegraph_row(vocab, trace) {
        if !out.contains(&row) {
            out.push(row);
        }
    }
    out.truncate(MAX_CANDIDATES);
    out
}

/// Cut 11 §4: `foe_tag:telegraph → retreat` when a telegraph shows in the trace and the
/// lineage owns the tag.
pub fn telegraph_row(vocab: &Vocabulary, trace: &Trace) -> Option<Row> {
    if !trace.turns.iter().any(|t| !t.telegraphs.is_empty()) {
        return None;
    }
    let has_tag = vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some("telegraph"));
    let v = vocab.verbs.iter().find(|x| x.v == "retreat" && x.a.is_none())?;
    has_tag.then(|| Row::new(vec![Cond::t("foe_tag", "telegraph")], v.clone()))
}

/// Cut 13 §1: a stall's candidates — the rows that leave the floor the hero paced on: the
/// dive (`path_stairs → descend`; `floor_seen ≥ N → descend` at the threshold the
/// checkpoint's floor already met) and the escape (`depth ≥ D → return` / `bank`, D the
/// stall's floor: the run pays out instead of forfeiting). Every token from the vocabulary;
/// a targeting row is never offered (the foe the guard gave up on was unreachable).
pub fn stall_candidates(vocab: &Vocabulary, state: &Run) -> Vec<Row> {
    let has_cond = |k: &str| vocab.conds.iter().any(|c| c.k == k);
    let verb = |v: &str| vocab.verbs.iter().find(|x| x.v == v && x.a.is_none()).cloned();
    let mut out: Vec<Row> = Vec::new();
    if let Some(v) = verb("descend") {
        if has_cond("path_stairs") {
            out.push(Row::new(vec![Cond::flag("path_stairs")], v.clone()));
        }
        if has_cond("floor_seen>=") {
            let seen = state.floor.map.seen_pct();
            // The editor's own thresholds (`floor_seen>=` 25 · 50 · 75 · 100): a patch at 60 %
            // could not be set by hand (QA on 50bb162).
            let n = [100, 75, 50, 25].into_iter().find(|n| seen >= *n).unwrap_or(25);
            out.push(Row::new(vec![Cond::n("floor_seen>=", n)], v));
        }
    }
    if has_cond("depth>=") {
        for v in ["return", "bank"] {
            if let Some(v) = verb(v) {
                out.push(Row::new(vec![Cond::n("depth>=", state.depth as i32)], v));
            }
        }
    }
    out
}

// ---------------------------------------------------------------- replays

/// Replays of the checkpoint under one rule set. The game is cloned once; each replay resets
/// exactly what a tick can touch (`Game::tick` and `Ctx`): the run, the facts, the kill counts,
/// the tick counters, the events.
struct Replayer {
    g: Game,
    run: Run,
    facts: BTreeSet<String>,
    kill_counts: BTreeMap<String, u32>,
    total_turns: u64,
    ended: bool,
    ticks: u32,
    seed: u64,
    /// Cut 13 §1: a stall's replays — (the checkpoint's depth, its guard count): a replay
    /// survives when it leaves the floor (descends, returns or banks) or ends the stall (no
    /// guard fires before the window's end).
    stall: Option<(u32, u32)>,
}

impl Replayer {
    fn new(base: &Game, rules: &RuleSet, ticks: u32, stall: bool) -> Option<Replayer> {
        let mut g = base.sim_clone();
        g.set_rules(rules.clone()).ok()?;
        let run = base.run.clone()?;
        let stall = stall.then_some((run.depth, run.stuck_fires));
        Some(Replayer {
            g,
            run,
            facts: base.lineage.facts.clone(),
            kill_counts: base.lineage.kill_counts.clone(),
            total_turns: base.lineage.total_turns,
            ended: base.lineage.ended,
            ticks,
            seed: base.lineage.seed,
            stall,
        })
    }

    /// One reseeded replay of the last ticks. Returns (survived, watched row fired).
    fn replay(&mut self, nonce: u64, watch_row: i32) -> (bool, bool) {
        let mut run = self.run.clone();
        run.rng = Rng::derive(self.seed ^ splitmix(nonce), run.turn as u64);
        self.g.run = Some(run);
        self.g.lineage.facts = self.facts.clone();
        self.g.lineage.kill_counts = self.kill_counts.clone();
        self.g.lineage.total_turns = self.total_turns;
        self.g.lineage.ended = self.ended;
        self.g.events.clear();
        let mut fired = false;
        // Cut 4: the window ends at the killing blow, then runs on while a hostile is still in
        // view (up to `ENCOUNTER_TICKS`): surviving the tick of the blow is not surviving the
        // fight, and a boss wall scored 0.9+ baselines that way.
        let mut n = 0;
        loop {
            if n >= self.ticks + ENCOUNTER_TICKS {
                break;
            }
            // A stall's window ends where the stall did (the unreachable foe stays in view).
            if n >= self.ticks && (self.stall.is_some() || !self.g.run.as_ref().is_some_and(|r| crate::turn::view(r).foes.iter().any(|&i| r.monsters[i].awake))) {
                break;
            }
            self.g.tick();
            n += 1;
            if !fired {
                fired = self.g.events.iter().any(|e| matches!(e, Ev::Rule { row, .. } if *row == watch_row));
            }
            self.g.events.clear();
            if self.g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                break;
            }
        }
        let survived = match self.stall {
            None => self.g.run.as_ref().is_some_and(|r| r.over != Some(ExitTier::Death)),
            Some((depth, fires)) => self.g.run.as_ref().is_some_and(|r| match r.over {
                Some(ExitTier::Death) => false,
                Some(_) => !r.timed_out,
                None => r.depth > depth || r.stuck_fires <= fires,
            }),
        };
        (survived, fired)
    }
}

/// The checkpoint game the replays start from, and how many ticks to the death.
fn replay_base(game: &Game, rec: &DeathRec) -> Option<(Game, u32)> {
    let t10 = rec.t10.clone()?;
    let mut base = game.sim_clone();
    base.lineage.facts = rec.t10_facts.clone();
    if let Some(k) = &rec.t10_kill_counts {
        base.lineage.kill_counts = k.clone();
    }
    if let Some(l) = &rec.t10_lineage {
        l.apply(&mut base.lineage);
    }
    base.lineage.heir = t10.heir;
    base.lineage.trait_ = t10.trait_;
    base.lineage.class = t10.hero.class;
    base.lineage.party.clear();
    base.lineage.supplies.clear();
    // Cut 4: the window runs to the killing blow (the trace's last turn is the hero's last
    // *action*; the blow lands up to a turn later, and a replay cut there scored a hero at 1 HP
    // as "survived" — baselines of 0.9–1.0 on most deaths), plus one turn of grace for the
    // reseeded timing.
    let last_t = rec.death.trace.turns.last().map(|t| t.t).unwrap_or(t10.turn + 100);
    let end = rec.death_tick.max(last_t);
    let ticks = (end.saturating_sub(t10.turn)).max(1) + crate::engine::TICKS_PER_TURN;
    base.run = Some(t10);
    Some((base, ticks))
}

/// Where a patch may go: the top, and before the row that fired most in the trace.
fn insert_positions(rules: &RuleSet, trace: &Trace) -> Vec<usize> {
    let mut counts = vec![0u32; rules.rows.len()];
    for t in &trace.turns {
        if t.row >= 0 && (t.row as usize) < counts.len() {
            counts[t.row as usize] += 1;
        }
    }
    let fired_idx = counts.iter().enumerate().max_by_key(|(i, c)| (**c, usize::MAX - *i)).and_then(|(i, c)| (*c > 0 && i > 0).then_some(i));
    std::iter::once(0).chain(fired_idx).collect()
}

fn max_rows(rec: &DeathRec) -> usize {
    rec.vocab.max_rows.max(rec.rules.own_rows() + 1)
}

fn patched(rec: &DeathRec, row: &Row, pos: usize) -> RuleSet {
    let mut rules = rec.rules.clone();
    rules.rows.insert(pos.min(rules.rows.len()), row.clone());
    rules.fit(max_rows(rec))
}

/// Cut 11 §2: a patch's rule set — the row inserted at `insert_at`; on an unlock
/// pseudo-patch (`insert_at` −1) the set unchanged when it already has the row (the lock's
/// own row), else the row at the top (the den raid the unlock makes writable).
pub fn patched_rules(rules: &RuleSet, p: &Patch, max_rows: usize) -> RuleSet {
    let mut rules = rules.clone();
    if p.insert_at < 0 && rules.rows.contains(&p.row) {
        return rules;
    }
    rules.rows.insert((p.insert_at.max(0) as usize).min(rules.rows.len()), p.row.clone());
    rules.fit(max_rows.max(1))
}

/// The replay seed for a row at a position: a function of the row itself, so a patch's fired
/// share can be re-measured later with the very same replays.
fn row_nonce(row: &Row, pos: usize, i: u32) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in serde_json::to_string(row).unwrap_or_default().bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h ^ ((pos as u64) << 16) ^ i as u64
}

/// Replays one patched rule set. `None` when the row rarely fires or the hero does not reach
/// `bar` (Cut 4: the baseline plus `PATCH_MARGIN`, never above `SURVIVE_BAR`, so a patch that
/// only moves the forecast is still scored at a high baseline); otherwise (survive share,
/// fired share) over all replays.
fn score(rp: &mut Replayer, row: &Row, pos: usize, bar: f64) -> Option<(f64, f64)> {
    let (mut survived, mut failed, mut fired) = (0u32, 0u32, 0u32);
    for i in 0..REPLAYS {
        let (s, f) = rp.replay(row_nonce(row, pos, i), pos as i32);
        fired += f as u32;
        if s {
            survived += 1;
        } else {
            failed += 1;
        }
        if i == 0 && !f {
            return None; // the row never fires here: identical to the original
        }
        if failed as f64 > REPLAYS as f64 * (1.0 - bar) {
            return None;
        }
        if (i + 1 - fired) as f64 > REPLAYS as f64 * (1.0 - FIRED_BAR) {
            return None;
        }
    }
    let rate = survived as f64 / REPLAYS as f64;
    let fired = fired as f64 / REPLAYS as f64;
    (rate >= bar && fired >= FIRED_BAR).then_some((rate, fired))
}

/// Cut 6 §5: (survive share, fired share) of a row over all replays, no early exit — for the
/// boss counter row, which is shown whatever its numbers once it proves executable.
fn measure(rp: &mut Replayer, row: &Row, pos: usize) -> (f64, f64) {
    let (mut survived, mut fired) = (0u32, 0u32);
    for i in 0..REPLAYS {
        let (s, f) = rp.replay(row_nonce(row, pos, i), pos as i32);
        survived += s as u32;
        fired += f as u32;
    }
    (survived as f64 / REPLAYS as f64, fired as f64 / REPLAYS as f64)
}

/// Cut 11 §4: (survive share, fired share) of a row inserted at the top, over the death's own
/// replays, no early exit (tests and probes).
pub fn measure_row(game: &Game, rec: &DeathRec, row: &Row) -> Option<(f64, f64)> {
    let (base, ticks) = replay_base(game, rec)?;
    let mut rp = Replayer::new(&base, &patched(rec, row, 0), ticks, rec.stall)?;
    Some(measure(&mut rp, row, 0))
}

/// Cut 6 §5: on a boss death with the counter known, the counter row (when not already in the
/// rules and executable — its verb in the death's vocabulary, fired in ≥ `FIRED_BAR` of the
/// replays) is pinned first among the patches: it is, by construction, the best candidate.
fn pin_counter(game: &Game, rec: &mut DeathRec, base: &Game, ticks: u32) -> Option<Patch> {
    let row = pinnable_counter(game, rec)?;
    let mut rp = Replayer::new(base, &patched(rec, &row, 0), ticks, rec.stall)?;
    let (survive, fired) = measure(&mut rp, &row, 0);
    if fired < FIRED_BAR {
        return None;
    }
    rec.counter = Some(row.clone());
    Some(Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false })
}

/// Cut 11 §2: the root-cause patch. A theft root: `foe_tag:thief → attack tag:thief` (the
/// `thief_guard` card's row when the card is owned and not in the set), measured at the top
/// like the counter (`measure`: no early exit — the thief is usually floors back, so the row
/// rarely fires from the checkpoint; the forecast delta is its number). A lock root: the
/// locked row itself at `insert_at` −1, its survival measured with the condition unlocked.
/// Nothing when the set already carries the answer.
fn root_patch(game: &Game, rec: &mut DeathRec, base: &Game, ticks: u32) -> Option<Patch> {
    let root = rec.root.clone()?;
    let (row, unlock) = match root.kind.as_str() {
        "theft" => thief_row(game, &rec.rules, root.text.starts_with("den took"))?,
        "lock" => (rec.rules.rows.get(root.row)?.clone(), root.unlock.clone()),
        _ => return None,
    };
    if let Some(r) = rec.root.as_mut() {
        r.unlock = unlock.clone();
    }
    // The unlock pseudo-patch reads the unlock (`◆2 cond: on see`); a writable row reads the
    // root it answers (`den took the heal`).
    let text = match &unlock {
        Some(u) => unlock_label(u),
        None => root.text.clone(),
    };
    let mut b = base.sim_clone();
    unlock_base(&mut b, rec);
    let insert_at = if unlock.is_some() { -1 } else { 0 };
    let patch = Patch { row: row.clone(), insert_at, survive: 0.0, forecast_delta: 0.0, replace: false, remove: false, root: Some(PatchRoot { text }), below_bar: false };
    let rules = patched_rules(&rec.rules, &patch, max_rows(rec));
    let pos = rules.rows.iter().position(|r| *r == row).unwrap_or(0);
    let mut rp = Replayer::new(&b, &rules, ticks, rec.stall)?;
    let (survive, _fired) = measure(&mut rp, &row, pos);
    Some(Patch { survive, ..patch })
}

/// Cut 11 §2: an unlock as the root text: `◆2 cond: on see`, `◆3 card: thief guard`.
pub fn unlock_label(id: &str) -> String {
    let cost = crate::meta::unlock_cost(id);
    match id.strip_prefix("cond_") {
        Some(k) => format!("◆{cost} cond: {}", k.replace('_', " ")),
        None => format!("◆{cost} card: {}", id.replace('_', " ")),
    }
}

/// Cut 11 §2: the row that answers a theft, and the unlock it needs when the lineage cannot
/// write it yet. A den's snatch is answered by the raid (`on_see: den → attack nearest`; the
/// sleeping den is scenery to `foe_tag`), needing `cond_on_see`; a thief's blow by the
/// `thief_guard` card when owned, else `foe_tag:thief → attack tag:thief`. `None` when the
/// set already carries the answer or the lineage lacks the fact.
pub fn thief_row(game: &Game, rules: &RuleSet, den: bool) -> Option<(Row, Option<String>)> {
    let l = &game.lineage;
    let has_card = rules.rows.iter().any(|r| r.verb.v == "tactic" && r.verb.a.as_deref() == Some("thief_guard"));
    if den {
        // Cut 12 §2: the thief guard card raids the den too (its first row).
        if !l.facts.contains("den") || has_card || rules.rows.iter().any(|r| r.conds.iter().any(|c| c.k == "on_see" && c.t.as_deref() == Some("den"))) {
            return None;
        }
        let row = crate::probes::situation_answer("den");
        let unlock = (!l.unlocks.contains("cond_on_see")).then(|| "cond_on_see".to_string());
        return Some((row, unlock));
    }
    let has_row = rules.rows.iter().any(|r| r.verb.a.as_deref() == Some("tag:thief"));
    if has_card || has_row {
        return None;
    }
    if l.unlocks.contains("thief_guard") {
        if let Some(r) = crate::meta::unlock_row(l, "thief_guard") {
            return Some((r, None));
        }
    }
    let vocab = game.vocabulary();
    let cond = vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some("thief"));
    let verb = vocab.verbs.iter().find(|v| v.v == "attack" && v.a.as_deref() == Some("tag:thief"))?;
    cond.then(|| (Row::new(vec![Cond::t("foe_tag", "thief")], verb.clone()), None))
}

/// Cut 6 §8: on a boss death the escape family ranks below targeting (the counter first when
/// pinned), and a `return`/`bank` patch is never shown alone.
fn boss_order(rec: &mut DeathRec) {
    if rec.boss.is_none() {
        return;
    }
    let escape = |p: &Patch| family(&p.row) == "escape";
    let counter = rec.counter.clone();
    let is_counter = |p: &Patch| counter.as_ref() == Some(&p.row);
    let mut ps = std::mem::take(&mut rec.death.patches);
    let mut ordered: Vec<Patch> = ps.iter().filter(|p| is_counter(p)).cloned().collect();
    ps.retain(|p| !is_counter(p));
    let (esc, rest): (Vec<Patch>, Vec<Patch>) = ps.into_iter().partition(escape);
    ordered.extend(rest);
    ordered.extend(esc);
    rec.death.patches = ordered;
}

/// The survival a scored candidate must reach: the baseline plus the margin, capped at
/// `SURVIVE_BAR` (so at a high baseline the forecast can still decide).
pub fn survive_bar(baseline: f64) -> f64 {
    (baseline + PATCH_MARGIN).min(SURVIVE_BAR)
}

/// Share of the death's own replays in which a patch's row fired (the selection's replays,
/// re-run; for tests and the gate table).
pub fn patch_fired_rate(game: &Game, rec: &DeathRec, p: &Patch) -> f64 {
    let Some((mut base, ticks)) = replay_base(game, rec) else { return 0.0 };
    // An unlock pseudo-patch: the token unlocked; the watched row is the set's own or the top.
    if p.root.is_some() {
        unlock_base(&mut base, rec);
    }
    let rules = patched_rules(&rec.rules, p, max_rows(rec));
    let pos = rules.rows.iter().position(|r| *r == p.row).unwrap_or(0);
    let Some(mut rp) = Replayer::new(&base, &rules, ticks, rec.stall) else { return 0.0 };
    let fired = (0..REPLAYS).filter(|&i| rp.replay(row_nonce(&p.row, pos, i), pos as i32).1).count();
    fired as f64 / REPLAYS as f64
}

/// Cut 11 §2: a game with the root's unlock owned (the lock's condition; `cond_on_see` for
/// the den raid), for measuring the pseudo-patch.
fn unlock_base(base: &mut Game, rec: &DeathRec) {
    if let Some(u) = rec.root.as_ref().and_then(|r| r.unlock.clone()) {
        base.lineage.unlocks.insert(u);
    }
}

/// One patch per family, keeping the first (best-ranked) of each. Cut 11 §2: a root-cause
/// patch is its own family (the theft's answer beside the moment's).
fn one_per_family(patches: Vec<Patch>) -> Vec<Patch> {
    let mut seen: Vec<&'static str> = Vec::new();
    patches
        .into_iter()
        .filter(|p| {
            if p.root.is_some() {
                return true;
            }
            let f = family(&p.row);
            if seen.contains(&f) {
                false
            } else {
                seen.push(f);
                true
            }
        })
        .collect()
}

/// Compute the verdict and patches for a recorded death. Cut 4: `gap` iff the best candidate
/// beats the unpatched baseline by `PATCH_MARGIN` (survival over the reseeded replays), or —
/// when none does — some candidate moves the forecast at the death's depth by `DELTA_BAR`
/// (the deltas are simulated here, so the verdict is final); else `dice`.
pub fn compute_verdict(game: &Game, rec: &mut DeathRec) {
    if rec.verdict_done {
        return;
    }
    rec.verdict_done = true;
    let Some((base, ticks)) = replay_base(game, rec) else { return };
    let positions = insert_positions(&rec.rules, &rec.death.trace);
    // Baseline: the unpatched rules under the same reseeded replays. If they survive most of
    // the time the death was the dice, not the policy; a patch must beat the baseline clearly.
    let baseline = match Replayer::new(&base, &rec.rules, ticks, rec.stall) {
        Some(mut rp) => (0..REPLAYS).filter(|&i| rp.replay(0xBA5E_0000 | i as u64, -99).0).count() as f64 / REPLAYS as f64,
        None => 0.0,
    };
    rec.death.baseline = baseline;
    let bar = survive_bar(baseline);
    let mut cands = match &rec.t10 {
        Some(t10) if rec.stall => stall_candidates(&rec.vocab, t10),
        Some(t10) => candidates(&rec.vocab, t10, &rec.t10_facts, &game.lineage.flavours, &rec.death.trace),
        None => Vec::new(),
    };
    // A row the set already carries is not a patch (cohort 8, rater O: `foe: boss → attack
    // boss` offered at the top while it sat at R2). Moving a row is the editor's job.
    cands.retain(|r| !rec.rules.rows.iter().any(|x| x.conds == r.conds && x.verb == r.verb));
    // Cut 10 §2: the boss's counter row is only ever measured at the top (`pin_counter`); tried
    // "before the row that fired most" as well, that placement could outscore the top on the
    // moment's replays and hide the placement that passes the wall (cohort 6, rater K).
    let pinnable = pinnable_counter(game, rec);
    // Each (row, position) is scored on its own replays; natively they run on all cores
    // (`forecast::par_map`), collected in the same order as the loop they replace.
    let jobs: Vec<(usize, usize)> = cands.iter().enumerate().filter(|(_, r)| pinnable.as_ref() != Some(*r)).flat_map(|(ci, _)| positions.iter().map(move |&pos| (ci, pos))).collect();
    let rec_ref: &DeathRec = rec;
    let scored: Vec<Option<(f64, Row, usize)>> = crate::forecast::par_map(&base, jobs, |base, &(ci, pos)| {
        let row = &cands[ci];
        let mut rp = Replayer::new(base, &patched(rec_ref, row, pos), ticks, rec_ref.stall)?;
        score(&mut rp, row, pos, bar).map(|(rate, _fired)| (rate, row.clone(), pos))
    });
    let mut scored: Vec<(f64, Row, usize)> = scored.into_iter().flatten().collect();
    // Rank by how much the row beats the unpatched baseline; ties: a conditioned row (a policy)
    // beats an unconditioned one, then fewer conditions, then the top position.
    scored.sort_by(|a, b| {
        (b.0 - baseline)
            .partial_cmp(&(a.0 - baseline))
            .unwrap()
            .then(a.1.conds.is_empty().cmp(&b.1.conds.is_empty()))
            .then(a.1.conds.len().cmp(&b.1.conds.len()))
            .then(a.2.cmp(&b.2))
    });
    let mut edge_gap = scored.first().is_some_and(|best| best.0 - baseline >= PATCH_MARGIN - 1e-9);
    let mut patches: Vec<Patch> = scored.into_iter().map(|(rate, row, pos)| Patch { row, insert_at: pos as i32, survive: rate, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false }).collect();
    // Cut 11 §2: the chain's root — the theft's answer or the unlock — measured at the top,
    // scored like any other (its edge counts for the verdict; its delta is simulated first).
    if let Some(r) = root_patch(game, rec, &base, ticks) {
        edge_gap |= r.survive - baseline >= PATCH_MARGIN - 1e-9;
        patches.retain(|p| p.row != r.row);
        patches.insert(0, r);
    }
    // Cut 6 §5: the boss's counter row leads (its own family's first). Cut 10 §2: its edge
    // at the top counts for the verdict like any scored candidate's.
    if let Some(c) = pin_counter(game, rec, &base, ticks) {
        edge_gap |= c.survive - baseline >= PATCH_MARGIN - 1e-9;
        patches.retain(|p| p.row != c.row);
        patches.insert(0, c);
    }
    rec.death.patches = one_per_family(patches);
    boss_order(rec);
    if rec.stall {
        // Cut 13 §1: a stall is its own verdict; its patches are ranked by the forecast like
        // a death's, but the word on the screen is `stall`.
        rec.death.verdict = "stall".into();
    } else if edge_gap {
        rec.death.verdict = "gap".into();
    } else {
        // No patch survives the moment clearly: the forecast decides (a row that costs no
        // survival here but gains floors is still a gap in the policy).
        forecast_deltas(game, rec, VERDICT_DELTA_CANDIDATES, true);
        if rec.death.patches.iter().any(|p| p.forecast_delta >= DELTA_BAR - 1e-9) {
            rec.death.verdict = "gap".into();
        }
    }
}

/// Simulate the candidates' forecast deltas at the death's depth (`DELTA_SIMS` paired sims
/// each), in survival-edge order, up to `limit` patches; with `until_gap` the loop stops at the
/// first delta that makes the death a gap. `rec.deltas_n` counts the leading patches done, so
/// the death screen's full pass only adds the rest.
fn forecast_deltas(game: &Game, rec: &mut DeathRec, limit: usize, until_gap: bool) {
    if rec.deltas_done || rec.death.patches.is_empty() {
        rec.deltas_done = true;
        return;
    }
    let depth = (rec.death.depth + 1).min(game.lineage.best_depth + 1).max(1);
    let sims = crate::forecast::DELTA_SIMS;
    let baseline = rec.death.baseline;
    if rec.deltas_n == 0 {
        // Forecasts cost ~20 sims each: rank by survival edge first and only forecast the top few.
        rec.death.patches.sort_by(|a, b| (b.survive - baseline).partial_cmp(&(a.survive - baseline)).unwrap().then(a.row.conds.len().cmp(&b.row.conds.len()).reverse()));
        // Cut 11 §2: the root-cause patch is simulated first (it is what the chain answers).
        if let Some(i) = rec.death.patches.iter().position(|p| p.root.is_some()) {
            let r = rec.death.patches.remove(i);
            rec.death.patches.insert(0, r);
        }
        // Cut 6 §5: the pinned counter row keeps its place at the head (and gets a delta).
        if let Some(i) = rec.counter.as_ref().and_then(|c| rec.death.patches.iter().position(|p| p.row == *c)) {
            let c = rec.death.patches.remove(i);
            rec.death.patches.insert(0, c);
        }
        rec.death.patches.truncate(DELTA_CANDIDATES);
    }
    let base = crate::forecast::reach_with(game, &rec.rules, depth, sims, 0xDE17A);
    let limit = limit.min(rec.death.patches.len());
    let max_rows = max_rows(rec);
    while rec.deltas_n < limit {
        let p = rec.death.patches[rec.deltas_n].clone();
        let rules = patched_rules(&rec.rules, &p, max_rows);
        // An unlock pseudo-patch's delta is measured on a lineage that owns the unlock.
        let r = if p.insert_at < 0 {
            let mut g = game.sim_clone();
            unlock_base(&mut g, rec);
            crate::forecast::reach_with(&g, &rules, depth, sims, 0xDE17A)
        } else {
            crate::forecast::reach_with(game, &rules, depth, sims, 0xDE17A)
        };
        rec.death.patches[rec.deltas_n].forecast_delta = r - base;
        rec.deltas_n += 1;
        if until_gap && r - base >= DELTA_BAR - 1e-9 {
            break;
        }
    }
    if rec.deltas_n >= rec.death.patches.len() {
        rec.deltas_done = true;
    }
}

/// Candidates whose forecast delta the verdict itself simulates when no patch has a survival
/// edge (the best by edge; the death screen fills in the rest).
pub const VERDICT_DELTA_CANDIDATES: usize = 3;

/// Fill in each patch's full-forecast delta at the death's depth and shape the shown list.
pub fn compute_deltas(game: &Game, rec: &mut DeathRec) {
    if rec.shaped {
        return;
    }
    rec.shaped = true;
    // Cut 11 §4: a `dice` death with nothing over the bar still names its alternative — the
    // best candidates measured in full, flagged `below_bar`, with their deltas.
    if rec.death.verdict == "dice" {
        if rec.death.patches.is_empty() {
            dice_fallback(game, rec);
        }
        dice_telegraph(game, rec);
    }
    forecast_deltas(game, rec, DELTA_CANDIDATES, false);
    if rec.death.patches.is_empty() {
        return;
    }
    let baseline = rec.death.baseline;
    // A patch must beat the baseline by 0.15 or move the forecast by 0.02; an unconditioned row
    // must beat the baseline by 0.30. Rank by the forecast delta when any patch moves it, else
    // by (survive − baseline); show three, never two of one family.
    let pre_retain = rec.death.patches.clone();
    let counter = rec.counter.clone();
    // Cut 11 §2: the root-cause patch always stays — it is the chain's answer, shown with its
    // honest numbers; it leads when its forecast delta reaches the best symptom patch's, and
    // takes the last slot otherwise (the moment's better fix first, the root still named).
    let best_symptom = pre_retain.iter().filter(|p| p.root.is_none()).map(|p| p.forecast_delta).fold(f64::NEG_INFINITY, f64::max);
    let root_leads = |p: &Patch| p.root.is_some() && p.forecast_delta >= best_symptom - 1e-9;
    let is_dice = rec.death.verdict == "dice";
    rec.death.patches.retain(|p| {
        let edge = p.survive - baseline;
        counter.as_ref() == Some(&p.row) || p.root.is_some() || (is_dice && p.below_bar) || ((edge >= PATCH_MARGIN - 1e-9 || p.forecast_delta >= DELTA_BAR - 1e-9) && (!p.row.conds.is_empty() || edge >= 0.3))
    });
    rank_patches(&mut rec.death.patches, baseline);
    boss_order(rec);
    let patches = std::mem::take(&mut rec.death.patches);
    rec.death.patches = one_per_family(patches);
    rec.death.patches.truncate(SHOWN);
    if let Some(r) = pre_retain.iter().find(|p| p.root.is_some()) {
        let pinned = usize::from(rec.death.patches.first().is_some_and(|p| counter.as_ref() == Some(&p.row)));
        let at = rec.death.patches.iter().position(|p| p.row == r.row);
        if root_leads(r) {
            // Shown at the head (after the pinned counter) whatever the cut above did.
            if at != Some(pinned) {
                if let Some(i) = at {
                    rec.death.patches.remove(i);
                }
                rec.death.patches.insert(pinned.min(rec.death.patches.len()), r.clone());
                rec.death.patches.truncate(SHOWN);
            }
        } else if at.is_none() {
            rec.death.patches.truncate(SHOWN - 1);
            rec.death.patches.push(r.clone());
        }
    }
    // A `gap` never shows an empty list: fall back to the best survivors even without an edge.
    // Cut 11 §4: nor does a `dice` — its best candidates are shown under the bar.
    if rec.death.patches.is_empty() {
        let mut all = pre_retain.clone();
        all.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap().then(b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap()));
        rec.death.patches = one_per_family(all);
        boss_order(rec);
        rec.death.patches.truncate(SHOWN);
    }
    if is_dice {
        // The telegraph's answer leads a dice death (after the pinned counter): it is *the*
        // alternative the screen names, whatever its number.
        if let Some(t) = telegraph_row(&rec.vocab, &rec.death.trace) {
            if let Some(p) = pre_retain.iter().find(|p| p.row == t) {
                let pinned = usize::from(rec.death.patches.first().is_some_and(|p| counter.as_ref() == Some(&p.row)));
                rec.death.patches.retain(|x| x.row != t);
                rec.death.patches.insert(pinned.min(rec.death.patches.len()), p.clone());
                rec.death.patches.truncate(SHOWN);
            }
        }
        let bar = survive_bar(baseline);
        for p in rec.death.patches.iter_mut() {
            p.below_bar = p.survive < bar - 1e-9;
        }
    }
    // Cut 6 §8: on a boss death a `return` is never the only patch — the best other scored
    // candidate joins it (giving up is not the answer to a wall).
    if rec.boss.is_some() && rec.death.patches.len() == 1 && family(&rec.death.patches[0].row) == "escape" {
        let mut others: Vec<Patch> = pre_retain.into_iter().filter(|p| family(&p.row) != "escape").collect();
        others.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap().then(b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap()));
        if let Some(o) = others.into_iter().next() {
            rec.death.patches.insert(0, o);
        }
    }
}

/// Cut 11 §4: on a `dice` death after a telegraph, `foe_tag:telegraph → retreat` is measured
/// in full and joins the list (`below_bar` when it is) unless a retreat-family patch is
/// already there — the screen names the telegraph's answer with its number.
fn dice_telegraph(game: &Game, rec: &mut DeathRec) {
    let Some(row) = telegraph_row(&rec.vocab, &rec.death.trace) else { return };
    if rec.death.patches.iter().any(|p| p.row == row || family(&p.row) == "retreat") {
        return;
    }
    let Some((base, ticks)) = replay_base(game, rec) else { return };
    let Some(mut rp) = Replayer::new(&base, &patched(rec, &row, 0), ticks, rec.stall) else { return };
    let (survive, fired) = measure(&mut rp, &row, 0);
    if fired < FIRED_BAR {
        return;
    }
    let below_bar = survive < survive_bar(rec.death.baseline) - 1e-9;
    rec.death.patches.push(Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar });
    rec.deltas_done = false;
}

/// Cut 11 §4: on a `dice` death with no candidate over the bar, measure `DICE_CANDIDATES`
/// in full at the top — the telegraph retreat first when a telegraph preceded the blow, then
/// the candidate list's head — and keep them (best survival first, `below_bar`) so the
/// screen names the alternative and its number. Their deltas follow in `forecast_deltas`.
fn dice_fallback(game: &Game, rec: &mut DeathRec) {
    let Some((base, ticks)) = replay_base(game, rec) else { return };
    let Some(t10) = rec.t10.clone() else { return };
    let mut rows: Vec<Row> = Vec::new();
    if let Some(r) = telegraph_row(&rec.vocab, &rec.death.trace) {
        rows.push(r);
    }
    // The candidate list, one family each (the telegraph retreat keeps its family), measured
    // in order until `DICE_CANDIDATES` have fired in half their replays.
    for r in candidates(&rec.vocab, &t10, &rec.t10_facts, &game.lineage.flavours, &rec.death.trace) {
        if rec.rules.rows.iter().any(|x| x.conds == r.conds && x.verb == r.verb) {
            continue;
        }
        if !rows.iter().any(|x| family(x) == family(&r)) {
            rows.push(r);
        }
    }
    let mut measured: Vec<(Patch, f64)> = Vec::new();
    for row in rows {
        if measured.iter().filter(|(_, f)| *f >= FIRED_BAR).count() >= DICE_CANDIDATES {
            break;
        }
        let Some(mut rp) = Replayer::new(&base, &patched(rec, &row, 0), ticks, rec.stall) else { continue };
        let (survive, fired) = measure(&mut rp, &row, 0);
        measured.push((Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: true }, fired));
    }
    let mut out: Vec<Patch> = measured.iter().filter(|(_, f)| *f >= FIRED_BAR).map(|(p, _)| p.clone()).collect();
    if out.is_empty() {
        // Pure dice (the baseline survives, nothing gets to fire): the row that fired most,
        // if any did — the one alternative the moment even reached. Cut 12: when none did
        // (a 3-HP descent into a pack: ten turns with nothing a row could do), the
        // best-surviving candidate is still named, flagged below the bar — Cut 11 §4, a dice
        // death is never empty. (Its survival is the baseline's; the honest fix for such a
        // death is a checkpoint before the descent, not a row at the moment.)
        if let Some((p, _)) = measured.iter().filter(|(_, f)| *f > 0.0).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).or_else(|| measured.iter().max_by(|a, b| a.0.survive.partial_cmp(&b.0.survive).unwrap())) {
            out.push(p.clone());
        }
    }
    out.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap());
    rec.death.patches = one_per_family(out);
    rec.deltas_done = false;
    rec.deltas_n = 0;
}

/// A patch moves the forecast when its delta reaches this (Cut 4: also a `gap` on its own);
/// below `DELTA_SINK` it is demoted under every other patch (a row that survives the moment
/// but costs floors).
pub const DELTA_BAR: f64 = 0.02;
pub const DELTA_SINK: f64 = -0.05;

/// Rank by the forecast delta first when any patch moves it, then by the survival edge over the
/// baseline (so `hp<20 → return` at Δ0 no longer beats `hp<20 → drink unknown` at Δ+5%); a
/// patch below `DELTA_SINK` sinks below all others.
pub fn rank_patches(patches: &mut [Patch], baseline: f64) {
    let by_delta = patches.iter().any(|p| p.forecast_delta > DELTA_BAR);
    let edge = |p: &Patch| p.survive - baseline;
    patches.sort_by(|a, b| {
        let sink = (a.forecast_delta < DELTA_SINK).cmp(&(b.forecast_delta < DELTA_SINK));
        let delta = b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap();
        let surv = edge(b).partial_cmp(&edge(a)).unwrap();
        sink.then(if by_delta { delta.then(surv) } else { surv.then(delta) })
    });
}

/// The full death for a run id, computing verdict and deltas on first request.
pub fn death(game: &mut Game, run_id: u32) -> Option<Death> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    compute_deltas(game, &mut rec);
    let d = rec.death.clone();
    game.deaths.insert(run_id, rec);
    Some(d)
}

/// Verdict only (no forecast deltas), for the metrics.
pub fn verdict(game: &mut Game, run_id: u32) -> Option<String> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    let v = rec.death.verdict.clone();
    game.deaths.insert(run_id, rec);
    Some(v)
}

#[cfg(test)]
mod tests_trace {
    use super::*;
    use crate::gen::Floor;
    use crate::geom::Pos;
    use crate::item::Item;
    use crate::monster::Monster;
    use crate::rules::Verb;
    use crate::tiles::{Map, Tile, VISION};

    /// A game with a live run on an open 16×12 room, no monsters or items.
    fn arena(seed: u64) -> Game {
        let mut g = Game::new(seed);
        g.max_deaths = 1000;
        g.start_run(Some(seed.wrapping_mul(7) + 3));
        let run = g.run.as_mut().unwrap();
        let mut map = Map::new(16, 12, Tile::Wall);
        for y in 1..11 {
            for x in 1..15 {
                map.set(Pos::new(x, y), Tile::Floor);
            }
        }
        let up = Pos::new(1, 1);
        let down = Pos::new(14, 10);
        map.set(up, Tile::StairsUp);
        map.set(down, Tile::StairsDown);
        map.compute_corridors(&[]);
        run.floor = Floor { map, stairs_up: up, stairs_down: down, rooms: Vec::new(), vision: VISION };
        run.monsters.clear();
        run.items.clear();
        run.overlays.clear();
        run.hero.pos = Pos::new(4, 5);
        run.hero_dist_pos = None;
        run.last_visible = vec![u32::MAX];
        run.floor.map.update_vision(run.hero.pos, VISION);
        g.events.clear();
        g.history.clear();
        g
    }

    fn add_monster(g: &mut Game, kind: &str, x: i32, y: i32) {
        let run = g.run.as_mut().unwrap();
        let id = run.new_id();
        let depth = run.depth;
        let mut m = Monster::spawn(id, kind, Pos::new(x, y), depth);
        m.awake = true;
        m.last_seen = Some(run.hero.pos);
        run.monsters.push(m);
    }

    fn give(g: &mut Game, kind: &str) {
        let run = g.run.as_mut().unwrap();
        let id = run.new_item_id();
        run.hero.inv.push(Item::new(id, kind));
    }

    fn set_rules(g: &mut Game, rows: Vec<Row>) {
        g.set_rules(RuleSet { rows, name: None }).unwrap();
    }

    fn attack_rules(g: &mut Game) {
        set_rules(g, vec![Row::new(vec![Cond::n("foes>=", 1)], Verb::arg("attack", "nearest"))]);
    }

    /// Tick until the hero dies; the recorded death's run id.
    fn die(g: &mut Game) -> u32 {
        for _ in 0..600 {
            g.tick();
            g.events.clear();
            if g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                let id = g.run.as_ref().unwrap().id;
                assert_eq!(g.run.as_ref().unwrap().over, Some(ExitTier::Death), "the run ended without a death");
                g.finish_run();
                return id;
            }
        }
        panic!("the hero did not die");
    }

    fn tag_fact(g: &mut Game, kind: &str, tag: &str) {
        g.lineage.facts.insert(format!("foe:{kind}:{tag}"));
    }

    fn families(patches: &[Patch]) -> Vec<&'static str> {
        patches.iter().map(|p| family(&p.row)).collect()
    }

    #[test]
    fn dying_low_with_unknown_potions_offers_the_id_policy() {
        let mut g = arena(4);
        g.run.as_mut().unwrap().hero.hp = 3;
        for _ in 0..3 {
            give(&mut g, "heal"); // unidentified: no flavour fact
        }
        for (x, y) in [(5, 5), (5, 6), (6, 4)] {
            add_monster(&mut g, "goblin", x, y);
        }
        attack_rules(&mut g);
        let id = die(&mut g);
        let d = g.death(id).unwrap();
        assert!(d.trace.turns.first().is_some_and(|t| t.hp <= 3), "{:?}", d.trace);
        assert_eq!(d.verdict, "gap");
        let p = d
            .patches
            .iter()
            .find(|p| p.row.verb.v == "drink" && p.row.verb.a.as_deref() == Some("unknown"))
            .unwrap_or_else(|| panic!("no drink-unknown patch: {:?}", d.patches));
        assert_eq!(p.row.conds.len(), 1);
        assert_eq!(p.row.conds[0].k, "hp<");
        assert_eq!(p.row.conds[0].n, Some(20), "3 of 36 HP is 8%: the nearest threshold is 20");
    }

    #[test]
    fn archer_pack_death_targets_ranged_and_never_an_absent_tag() {
        let mut g = arena(5);
        for t in ["ranged", "telegraph"] {
            tag_fact(&mut g, "goblin_archer", t);
        }
        // Known tags that no foe in this death carries.
        tag_fact(&mut g, "jackal", "pack");
        tag_fact(&mut g, "eel", "water");
        tag_fact(&mut g, "lich", "boss");
        g.run.as_mut().unwrap().hero.hp = 9;
        for (x, y) in [(8, 3), (8, 5), (8, 7), (9, 5)] {
            add_monster(&mut g, "goblin_archer", x, y);
        }
        set_rules(&mut g, vec![Row::new(vec![], Verb::new("hold"))]);
        let id = die(&mut g);
        let rec = g.deaths.get(&id).unwrap().clone();
        let present = ["ranged", "telegraph"];
        let only_present = |rows: &[Row]| {
            for r in rows {
                for c in r.conds.iter().filter(|c| c.k == "foe_tag") {
                    assert!(present.contains(&c.t.as_deref().unwrap()), "{r:?}");
                }
                if let Some(t) = r.verb.a.as_deref().and_then(|a| a.strip_prefix("tag:")) {
                    assert!(present.contains(&t), "{r:?}");
                }
            }
        };
        let cands = candidates(&rec.vocab, rec.t10.as_ref().unwrap(), &rec.t10_facts, &g.lineage.flavours, &rec.death.trace);
        assert!(cands.len() <= MAX_CANDIDATES);
        assert!(cands.contains(&Row::new(vec![Cond::t("foe_tag", "ranged")], Verb::arg("attack", "tag:ranged"))), "{cands:?}");
        only_present(&cands);
        assert!(!cands.iter().any(|r| r.verb.v == "free_captive"), "no captive was there");
        let d = g.death(id).unwrap();
        let rows: Vec<Row> = d.patches.iter().map(|p| p.row.clone()).collect();
        only_present(&rows);
        assert!(d.patches.len() <= SHOWN);
    }

    #[test]
    fn shown_patches_fired_in_half_their_replays_and_one_per_family() {
        let mut g = arena(2);
        g.lineage.facts.insert(crate::item::ident_fact(&g.lineage.flavours, "heal").unwrap());
        give(&mut g, "heal");
        give(&mut g, "heal");
        give(&mut g, "teleport"); // unknown scroll: the ID family has a read row too
        g.run.as_mut().unwrap().hero.hp = 14;
        for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6), (3, 4)] {
            add_monster(&mut g, "goblin", x, y);
        }
        attack_rules(&mut g);
        let id = die(&mut g);
        let d = g.death(id).unwrap();
        assert_eq!(d.verdict, "gap");
        assert!(!d.patches.is_empty() && d.patches.len() <= SHOWN, "{:?}", d.patches);
        let rec = g.deaths.get(&id).unwrap().clone();
        for p in &d.patches {
            let fired = patch_fired_rate(&g, &rec, p);
            assert!(fired >= FIRED_BAR, "{} fired in {:.0}% of replays", p.row.describe(), fired * 100.0);
            assert!(p.survive >= survive_bar(d.baseline) || p.forecast_delta >= DELTA_BAR, "{p:?} at baseline {}", d.baseline);
            assert!(p.row.conds.iter().filter(|c| c.k == "hp<").all(|c| HP_THRESHOLDS.contains(&c.n.unwrap())));
        }
        let fams = families(&d.patches);
        let mut uniq = fams.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), fams.len(), "two of one family: {fams:?}");
        // The verdict's pre-delta list is one per family too, so the cut to three cannot pair.
        let mut rec2 = g.deaths.get(&id).unwrap().clone();
        rec2.verdict_done = false;
        compute_verdict(&g, &mut rec2);
        let fams = families(&rec2.death.patches);
        let mut uniq = fams.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), fams.len(), "{fams:?}");
    }

    fn patch(verb: Verb, survive: f64, delta: f64) -> Patch {
        Patch { row: Row::new(vec![Cond::n("hp<", 20)], verb), insert_at: 0, survive, forecast_delta: delta, replace: false, remove: false, root: None, below_bar: false }
    }

    #[test]
    fn a_patch_that_moves_the_forecast_outranks_a_safer_one_that_does_not() {
        let mut ps = vec![patch(Verb::new("return"), 1.0, 0.0), patch(Verb::arg("drink", "unknown"), 0.4, 0.05)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::arg("drink", "unknown"));
        // No patch moves the forecast: the survival edge decides.
        let mut ps = vec![patch(Verb::arg("drink", "unknown"), 0.4, 0.01), patch(Verb::new("return"), 1.0, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::new("return"));
        // A patch that costs floors sinks below everything, whatever its survival.
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.2), patch(Verb::new("retreat"), 0.7, 0.0), patch(Verb::arg("drink", "unknown"), 0.4, 0.05)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["drink", "retreat", "return"]);
    }

    #[test]
    fn families_are_derived_from_the_row() {
        assert_eq!(family(&Row::new(vec![Cond::n("hp<", 30), Cond::n("foes>=", 2)], Verb::new("retreat"))), "retreat");
        assert_eq!(family(&Row::new(vec![Cond::t("foe_tag", "gas"), Cond::n("adj>=", 1)], Verb::new("retreat"))), "terrain");
        assert_eq!(family(&Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "unknown"))), "id");
        assert_eq!(family(&Row::new(vec![Cond::n("hp<", 30)], Verb::arg("drink", "heal"))), "consumable");
        assert_eq!(family(&Row::new(vec![Cond::n("hp<", 30)], Verb::arg("read", "teleport"))), "escape");
        assert_eq!(family(&Row::new(vec![Cond::n("hp<", 30)], Verb::new("return"))), "escape");
        assert_eq!(family(&Row::new(vec![Cond::n("floor_seen>=", 60)], Verb::new("descend"))), "dive");
        assert_eq!(family(&Row::new(vec![Cond::t("foe_tag", "boss")], Verb::arg("attack", "tag:boss"))), "targeting");
        assert_eq!(family(&Row::new(vec![Cond::n("foes>=", 1)], Verb::new("free_captive"))), "ally");
    }

    #[test]
    fn replays_are_deterministic() {
        let mut g = arena(6);
        g.run.as_mut().unwrap().hero.hp = 12;
        for (x, y) in [(5, 5), (5, 6), (4, 6), (3, 6)] {
            add_monster(&mut g, "goblin", x, y);
        }
        attack_rules(&mut g);
        let id = die(&mut g);
        let rec = g.deaths.get(&id).unwrap().clone();
        let mut a = rec.clone();
        let mut b = rec.clone();
        compute_verdict(&g, &mut a);
        compute_verdict(&g, &mut b);
        assert_eq!(a.death, b.death);
    }

    /// Verdict speed (Cut 2 §6: ≤ 0.4 s native); measured on the optimised profiles only.
    #[cfg(not(debug_assertions))]
    #[test]
    fn a_verdict_takes_under_point_six_seconds() {
        let mut g = Game::new(3);
        g.run_offline(2 * 3600);
        let ids: Vec<u32> = g.deaths.iter().filter(|(_, r)| !r.verdict_done).map(|(id, _)| *id).take(3).collect();
        assert!(!ids.is_empty(), "no unjudged death in two hours");
        let mut best = f64::MAX;
        for id in ids {
            let t = std::time::Instant::now();
            verdict(&mut g, id);
            best = best.min(t.elapsed().as_secs_f64());
        }
        assert!(best < 0.6, "fastest verdict {best:.2}s");
    }
}

#[cfg(test)]
mod tests_faithful {
    use super::*;
    /// Cut 4: a replay from the checkpoint with the checkpoint's own rng reproduces the death
    /// inside the replay window (the window reaches the killing blow, not just the last action).
    #[test]
    fn replay_without_reseed_reproduces_the_death() {
        let mut g = Game::new(3);
        g.max_deaths = 1000;
        g.run_offline(2 * 3600);
        let ids: Vec<u32> = g.deaths.keys().copied().collect();
        assert!(ids.len() >= 3, "too few deaths in two hours: {}", ids.len());
        for id in &ids {
            let rec = g.deaths.get(id).unwrap().clone();
            let (base, ticks) = replay_base(&g, &rec).expect("checkpoint");
            let mut rp = Replayer::new(&base, &rec.rules, ticks, rec.stall).expect("replayer");
            rp.g.run = Some(rp.run.clone());
            rp.g.lineage.facts = rp.facts.clone();
            rp.g.lineage.kill_counts = rp.kill_counts.clone();
            rp.g.lineage.total_turns = rp.total_turns;
            rp.g.events.clear();
            for _ in 0..ticks {
                rp.g.tick();
                rp.g.events.clear();
                if rp.g.run.as_ref().is_none_or(|r| r.over.is_some()) {
                    break;
                }
            }
            let r = rp.g.run.as_ref().unwrap();
            // Cut 13 §1: a stall record replays to the same stall.
            let want = if rec.stall { Some(ExitTier::Return) } else { Some(ExitTier::Death) };
            assert_eq!(r.over, want, "death {id}: checkpoint t{} D{} replay ended at t{} D{} hp {} (death tick {} D{})", rp.run.turn, rp.run.depth, r.turn, r.depth, r.hero.hp, rec.death_tick, rec.death.depth);
            assert!(!rec.stall || (r.timed_out && r.stuck_fires >= crate::engine::STALL_FIRES), "stall {id} did not replay as a stall");
            assert_eq!(r.turn, rec.death_tick, "death {id}");
        }
    }
}
