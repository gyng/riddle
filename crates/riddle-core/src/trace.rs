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
/// QA on 1a2a4a9: how long past the stall's own window a stall replay may take to leave the
/// floor, a walk home included (Cut 19 §2's `HOME_TICKS` was the walk's alone).
pub const LEAVE_TICKS: u32 = 1500;
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
    let chain = chain_of(&turns);
    let root = if stall { None } else { root_of(&game.prov, game.lineage.rules(), turns.last()) };
    let cause = if stall {
        format!("stalled · {}", run.stuck_cause.as_deref().unwrap_or("paced"))
    } else {
        own_cause(run).or_else(|| run.death_cause.clone()).unwrap_or_else(|| "unknown".into())
    };
    // Cut 10 §3: `3 hp short` (was `3 over`, which no rater could read): the HP that would have
    // kept the hero standing through the killing blow. A stall keeps nothing: `keeps $0`.
    // Cut 14 §2: `heal unused` / `N unknown unused` are the verdict's to add (`margin_lines`):
    // only when the candidate that drinks it survived the replays — a `dice` death's margin
    // never names an item the replays say would not have saved him (cohort 10, rater S).
    let margin = if stall { "keeps $0".to_string() } else { format!("{} hp short", run.death_short.max(1)) };
    let facts = &game.lineage.facts;
    let fl = &game.lineage.flavours;
    let heal_held = !stall && run.hero.inv.iter().any(|i| i.kind == "heal" && i.is_known(facts, fl));
    let unknown_held = if stall { 0 } else { run.hero.inv.iter().filter(|i| i.is_consumable() && !i.is_known(facts, fl)).count() as u32 };
    let unknown_scrolls = if stall { 0 } else { run.hero.inv.iter().filter(|i| i.cat() == crate::defs::Cat::Scroll && !i.is_known(facts, fl)).count() as u32 };
    let rules = game.lineage.rules().clone();
    let vocab = context_vocab(game, run);
    let death = Death {
        run_id: run.id,
        depth: run.depth,
        cause: cause.clone(),
        margin,
        verdict: if stall { "stall" } else { "dice" }.into(),
        baseline: 0.0,
        replays: REPLAYS,
        trace: Trace { turns, provenance, blow: if stall { None } else { crate::engine::death_blow(run) }, blows: if stall { Vec::new() } else { crate::engine::death_blows(run) } },
        patches: Vec::new(),
        morgue: morgue(game, run, &rules, stall),
        line: None,
        chain,
        rules: Some(rules.clone()),
        // The beat the morgue alone carried — not the headline's own notes (`Slain by …`,
        // `Down to 2 HP.`, `Returned with $0.`), which the screen already says; a stall's note
        // stays (it says what the stall paid). QA on 92eb880 (qaM: `R2 attack saved him.` above
        // a GAP on the hero's death): a floor he lived through earlier is not this screen's beat —
        // a `saved him` note never reaches a death's notes.
        notes: death_notes(run, stall),
        nothing_beats_base: false,
        cause_row: None,
        order_over: None,
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
    let loop_row = if stall { run.stuck_row.and_then(|r| usize::try_from(r).ok()).filter(|&r| r < rules.rows.len()) } else { None };
    let row_fired = run.row_fired.clone();
    let gamble_row = if stall { None } else { gamble_row(run, &rules) };
    let home = run.home_at.map(|(t, hp, _)| (hp, run.turn.saturating_sub(t)));
    DeathRec { death, home, t10, t10_facts, rules, vocab, verdict_done: false, deltas_done: false, deltas_n: 0, shaped: false, death_tick: run.turn, boss, counter: None, root, stall, t10_kill_counts, t10_lineage, heal_held, unknown_held, unknown_scrolls, root_under_base: false, camp_key: 0, loop_row, row_fired, low_fired: Vec::new(), floor_window: false, floor: game.floor_start.clone().filter(|(f, _)| f.id == run.id && f.depth == run.depth), gamble_row, chase_row: None, moves: Vec::new() }
}

/// QA on a946e04: how far back from the end a death's notes reach (ticks; 60 hero turns).
pub const NOTE_WINDOW: u32 = 600;

/// The notes the screen already says (the headline's own), or that no death is about.
fn note_noise(n: &str) -> bool {
    n.ends_with(" saved him.") || n.starts_with("Slain by") || n.starts_with("Down to ") || n.starts_with("Returned with") || n.starts_with("Lost the thread") || n.ends_with(": studied.") || n.starts_with("Met a ") || n.starts_with("Met an ") || n.starts_with("Learned")
}

/// Cut 13 §4 / QA on a946e04 (qaS: an archer death's notes read `Goblin Captain: summoner.`
/// — the run's last note, a floor earlier; a poison death's `The black one: confusion.
/// Gambled: confusion potion.`): a death's notes are about the death — the last two notes of
/// the death floor within `NOTE_WINDOW` ticks of the end that name the killer (its facts: `goblin
/// archer: ranged.`; `The air stings: bloats ahead.` for a bloat's gas) or the item whose harm
/// killed him (`The blue one: poison.` · `Gambled: poison potion.` on a poison death). None
/// when no note is (the screen has the headline). A stall keeps its last two notes (they say
/// what the stall paid).
fn death_notes(run: &Run, stall: bool) -> Vec<String> {
    let floor_t = run.turn.saturating_sub(run.floor_turn);
    let cause = run.death_cause.as_deref().unwrap_or("");
    // The gamble whose harm made the difference (`gamble_row`'s test): its notes name it.
    let gamble = run.gambles.last().filter(|(t, k, mal)| *mal && run.turn.saturating_sub(*t) <= GAMBLE_WINDOW && gamble_cause(k).is_some_and(|c| c == cause || run.gamble_harm >= run.death_short.max(1))).and_then(|(_, k, _)| gamble_cause(k));
    let keep = |t: u32, n: &str| -> bool {
        if note_noise(n) {
            return false;
        }
        if stall {
            return true;
        }
        t >= floor_t && run.turn.saturating_sub(t) <= NOTE_WINDOW && (death_note_names(n, cause) || gamble.is_some_and(|k| death_note_names(n, k)))
    };
    let mut v: Vec<String> = run.notes.iter().rev().filter(|(t, n)| keep(*t, n)).take(2).map(|(_, n)| n.clone()).collect();
    v.reverse();
    v
}

/// QA on a946e04: whether a note names the death's killer — the monster's title, a situation
/// its tag plays (a bloat's `gas`), or the item kind whose harm is the cause (`poison` · `fire`
/// · `caustic` for gas). A title is matched whole (`goblin` is not `goblin archer`), plural
/// allowed.
pub fn death_note_names(note: &str, cause: &str) -> bool {
    // (`own fire`: the hero's own harm names as the harm does)
    let cause = cause.strip_prefix("own ").unwrap_or(cause);
    let lower = note.to_lowercase();
    let words: Vec<String> = match cause {
        "poison" => vec!["poison".into()],
        "fire" => vec!["fire".into(), "burn".into()],
        "gas" | "burst" | "caustic" => vec!["gas".into(), "caustic".into(), "bloat".into()],
        "" => Vec::new(),
        c => {
            let mut v = vec![crate::engine::kind_title(c).to_lowercase()];
            if crate::defs::MONSTERS.iter().any(|m| m.kind == c) && monster_def(c).tags.contains(&"gas") {
                v.push("gas".into());
            }
            v
        }
    };
    // Longer monster titles that contain a word (`goblin archer` ⊃ `goblin`) do not count as it.
    let longer: Vec<String> = crate::defs::MONSTERS.iter().map(|m| m.title.to_lowercase()).collect();
    words.iter().any(|w| {
        let mut from = 0;
        while let Some(i) = lower[from..].find(w.as_str()) {
            let at = from + i;
            let before_ok = at == 0 || !lower[..at].chars().next_back().is_some_and(|c| c.is_alphanumeric());
            let rest = &lower[at + w.len()..];
            let rest = rest.strip_prefix('s').unwrap_or(rest);
            let after_ok = !rest.chars().next().is_some_and(|c| c.is_alphanumeric());
            let in_longer = longer.iter().any(|t| t.len() > w.len() && t.starts_with(w.as_str()) && lower[at..].starts_with(t.as_str()));
            if before_ok && after_ok && !in_longer {
                return true;
            }
            from = at + w.len();
        }
        false
    })
}

/// Cut 21 §3: how long after a gamble its harm still counts as the gamble's (ticks) — a fire
/// overlay burns 20 turns, a poison runs 40 ticks.
pub const GAMBLE_WINDOW: u32 = 300;

/// Cut 21 §3: the harm a gambled kind deals, as the death cause reads it.
pub fn gamble_cause(kind: &str) -> Option<&'static str> {
    match kind {
        "fire" => Some("fire"),
        "poison" => Some("poison"),
        "caustic" => Some("gas"),
        _ => None,
    }
}

/// Cut 21 §3: the set's own row whose unknown gamble dealt this death (`DeathRec.gamble_row`):
/// the cause is the harm of a malevolent kind gambled on this floor within `GAMBLE_WINDOW`
/// ticks, and the trace's action at that tick is a `drink unknown` / `read unknown` row of the
/// set. A gamble the trait or a chore made names no row.
///
/// QA on a946e04 (qaT: ~30 deaths all `gap`, `R1 drank poison at 17/36 hp` among them): or
/// the last gamble's harm made the difference — a poison that took 8 HP from a hero a
/// goblin's blow then killed 3 HP short (`Run.gamble_harm ≥ Run.death_short`): without the
/// gamble he stood through the blow.
///
/// QA on 778fa1b (qaV: `fire · D6 · GAP`, the trace `R3 card last stand` 10 → 3 → 0): or the
/// hero's own throw whose harm reached him (`Run.own_throw`: a fire blast on his own tile), by
/// a `throw` row — and a card row counts as the player's here: the card is his choice, and its
/// gamble or its throw is the row that killed him.
fn gamble_row(run: &Run, rules: &RuleSet) -> Option<usize> {
    own_harm(run).and_then(|(t, _, thrown)| own_harm_row(run, rules, t, thrown))
}

/// The tick and kind of the hero's own harm that dealt this death (a malevolent gamble, or
/// his own throw — `true`), the latest that did: its harm is the death's cause, or (the last
/// one) it took at least the HP he died short.
fn own_harm(run: &Run) -> Option<(u32, String, bool)> {
    let cause = run.death_cause.as_deref()?;
    let last = run.gambles.len().checked_sub(1);
    let window = |t: u32| run.turn.saturating_sub(t) <= GAMBLE_WINDOW;
    let gamble = run
        .gambles
        .iter()
        .enumerate()
        .rev()
        .find(|(i, (t, k, mal))| *mal && window(*t) && (gamble_cause(k) == Some(cause) || (Some(*i) == last && gamble_cause(k).is_some() && run.gamble_harm >= run.death_short.max(1))))
        .map(|(_, (t, k, _))| (*t, k.clone(), false));
    let thrown = run.own_throw.as_ref().filter(|(t, k)| window(*t) && (gamble_cause(k) == Some(cause) || (gamble_cause(k).is_some() && run.own_throw_harm >= run.death_short.max(1)))).map(|(t, k)| (*t, k.clone(), true));
    match (gamble, thrown) {
        (Some(g), Some(o)) => Some(if o.0 >= g.0 { o } else { g }),
        (g, o) => g.or(o),
    }
}

/// The row of `rules` that acted at tick `t`: the set's own gamble row (`drink` / `read
/// unknown`) or, for a throw (`thrown`), its `throw` row — or a card whose rows carry such a
/// verb (the trace names the card, not its row).
fn own_harm_row(run: &Run, rules: &RuleSet, t: u32, thrown: bool) -> Option<usize> {
    let turn = run.trace.iter().rev().find(|x| x.t <= t)?;
    let at = usize::try_from(turn.row).ok()?;
    let row = rules.rows.get(at)?;
    let gamble = |v: &crate::rules::Verb| matches!(v.v.as_str(), "drink" | "read") && v.a.as_deref() == Some("unknown");
    let fits = |v: &crate::rules::Verb| if thrown { v.v == "throw" } else { gamble(v) };
    let ok = match row.card() {
        Some(card) => crate::meta::unlock_rows(card).is_some_and(|rows| rows.iter().any(|r| fits(&r.verb))),
        None => fits(&turn.verb) && row.verb == turn.verb,
    };
    ok.then_some(at)
}

/// QA on 778fa1b (qaV: the banner's `FIRE` read as a monster until the morgue said `slain by
/// fire`): a death the hero's own harm dealt — his gamble's or his throw's fire, gas or poison
/// was the killing blow — is named as his (`own fire`) on the death record.
pub fn own_cause(run: &Run) -> Option<String> {
    let cause = run.death_cause.as_deref()?;
    let (_, k, _) = own_harm(run)?;
    (gamble_cause(&k) == Some(cause)).then(|| format!("own {cause}"))
}

/// Cut 11 §2: the chain — the `because` entries of the killing turn's rows, in row order,
/// deduped by text. Cut 21 §3: then the earlier turns' (newest first), so a row that did not
/// fire over the last ticks is explained on each (AF: `hp < 30% → bank` at 9/42 for six ticks).
fn chain_of(turns: &[crate::wire::TraceTurn]) -> Option<Vec<Because>> {
    let mut out: Vec<Because> = Vec::new();
    for turn in turns.iter().rev() {
        for b in turn.rows.iter().flatten().filter_map(|w| w.because.as_ref()) {
            if !out.iter().any(|o| o.text == b.text) {
                out.push(b.clone());
            }
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
    // QA on 778fa1b (qaU: `foe: boss → attack boss` beside an editor whose lists had neither):
    // the counter is offered only as a row the editor can write — its verb and every cond in
    // the lineage's vocabulary (`foe: boss` opens on the boss's sight, the counter on his
    // telegraph; a counter known before its tag waits for it).
    let v = game.vocabulary();
    if has_counter_verb(&rec.rules, &row) || !v.verbs.contains(&row.verb) || !row.conds.iter().all(|c| v.conds.iter().any(|x| x.same_token(c))) {
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
            // QA on a946e04 (qaS: `slain by goblin_archer`): the killer's title, as the notes say it.
            crate::engine::kind_title(run.death_cause.as_deref().unwrap_or_default()),
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
pub fn candidates(vocab: &Vocabulary, rules: &RuleSet, state: &Run, facts: &BTreeSet<String>, flavours: &Flavours, trace: &Trace) -> Vec<Row> {
    let has_cond = |k: &str| vocab.conds.iter().any(|c| c.k == k);
    let has_tag = |t: &str| vocab.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some(t));
    let verb = |v: &str, a: Option<&str>| vocab.verbs.iter().find(|x| x.v == v && x.a.as_deref() == a).cloned();
    let n = hp_threshold(state, trace);
    let low = Cond::n("hp<", n);
    let max_foes = trace.turns.iter().map(|t| t.rule_foes).max().unwrap_or(0);
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
    // Escape: hp<N → return; hp<N → read teleport | blink when held and known. Cut 19 §2: a
    // return walks to the up-stairs, so the way home is also tried 20 points sooner (a hurt
    // hero who starts home late meets the floor on the way).
    if let Some(v) = verb("return", None) {
        out.push(Row::new(vec![low.clone()], v.clone()));
        out.push(Row::new(vec![Cond::n("hp<", (n + 20).min(70))], v));
    }
    let scrolls: Vec<String> = held(Cat::Scroll).filter(|i| known(i)).map(|i| i.kind.clone()).collect();
    if let Some(k) = ["teleport", "blink"].iter().find(|k| scrolls.iter().any(|s| s == *k)) {
        if let Some(v) = verb("read", Some(k)) {
            out.push(Row::new(vec![low.clone()], v));
        }
    }
    // Engagement (Cut 19 §2): `foes>=1 → attack nearest` when no row of the set strikes the
    // foe in view — the missing row of a set that only walks, rests or runs (a return now
    // walks home, so it no longer answers every such death on its own).
    let engages = |r: &Row| r.verb.v == "attack" && r.conds.iter().all(|c| matches!(c.k.as_str(), "foes>=" | "adj>="));
    if max_foes >= 1 && has_cond("foes>=") && !rules.rows.iter().any(engages) {
        if let Some(v) = verb("attack", Some("nearest")) {
            out.push(Row::new(vec![Cond::n("foes>=", 1)], v));
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
            // QA on 1a2a4a9 (qaP: `R2 ↻ foe: gas · adjacent ≥ 1 → retreat · survives 100%`, and
            // the next run stalled in the same loop): a stall's replay survives only by leaving
            // the floor (Cut 13 §1) — it runs on past the stall's own window, up to
            // `LEAVE_TICKS`, until he goes down, comes home or the guard ends the run; a guard
            // that merely stayed quiet to the window's end is not the loop broken.
            if let Some((depth, _)) = self.stall {
                if n >= self.ticks + LEAVE_TICKS || self.g.run.as_ref().is_some_and(|r| r.depth > depth) {
                    break;
                }
            } else {
                if n >= self.ticks + ENCOUNTER_TICKS {
                    break;
                }
                if n >= self.ticks && !self.g.run.as_ref().is_some_and(|r| crate::turn::view(r).foes.iter().any(|&i| r.monsters[i].awake)) {
                    break;
                }
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
            Some((depth, _)) => self.g.run.as_ref().is_some_and(|r| match r.over {
                Some(ExitTier::Death) => false,
                Some(_) => !r.timed_out,
                None => r.depth > depth,
            }),
        };
        (survived, fired)
    }
}

impl Replayer {
    /// QA on e75ec29: one reseeded replay from the floor's start (`DeathRec.floor`), to the
    /// run's end, the floor left, or `ticks` (the death's own time on the floor plus
    /// `ENCOUNTER_TICKS`): (survived — not dead at the end —, the watched row acted). With
    /// `until_fired` it stops at the row's first act (`floor_fired` needs no more).
    fn floor_replay(&mut self, nonce: u64, watch_row: i32, until_fired: bool) -> (bool, bool) {
        let mut run = self.run.clone();
        run.rng = Rng::derive(self.seed ^ splitmix(nonce), run.turn as u64);
        let depth = run.depth;
        self.g.run = Some(run);
        self.g.lineage.facts = self.facts.clone();
        self.g.lineage.kill_counts = self.kill_counts.clone();
        self.g.lineage.total_turns = self.total_turns;
        self.g.lineage.ended = self.ended;
        self.g.events.clear();
        let mut fired = false;
        for _ in 0..self.ticks {
            self.g.tick();
            fired |= self.g.events.iter().any(|e| matches!(e, Ev::Rule { row, .. } if *row == watch_row));
            self.g.events.clear();
            if fired && until_fired {
                return (true, true);
            }
            if self.g.run.as_ref().is_none_or(|r| r.over.is_some() || r.depth != depth) {
                break;
            }
        }
        (self.g.run.as_ref().is_some_and(|r| r.over != Some(ExitTier::Death)), fired)
    }
}

/// QA on e75ec29: the floor's-start game the floor replays start from (`DeathRec.floor`), and
/// their tick cap. `None` without a floor start.
fn floor_base(game: &Game, rec: &DeathRec) -> Option<(Game, u32)> {
    let (start, facts) = rec.floor.clone()?;
    let mut base = game.sim_clone();
    base.lineage.facts = facts;
    if let Some(k) = &rec.t10_kill_counts {
        base.lineage.kill_counts = k.clone();
    }
    if let Some(l) = &rec.t10_lineage {
        l.apply(&mut base.lineage);
    }
    base.lineage.heir = start.heir;
    base.lineage.trait_ = start.trait_;
    base.lineage.class = start.hero.class;
    base.lineage.party.clear();
    base.lineage.supplies.clear();
    let ticks = rec.death_tick.saturating_sub(start.turn) + ENCOUNTER_TICKS;
    base.run = Some(start);
    Some((base, ticks))
}

/// The nonce of a floor replay of a row at a position (`row_nonce`, another stream).
fn floor_nonce(row: &Row, pos: usize, i: u32) -> u64 {
    row_nonce(row, pos, i) ^ 0xF100_0000
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
    // Cut 18 §4: the loop patch swaps or deletes the row it names, as the client applies it.
    // Cut 25 §2: a move reorders the set.
    if p.insert_at >= 0 && (p.replace || p.remove || is_move(p)) {
        return crate::offline::apply_patch(&rules, p, max_rows);
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
    Some(Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None })
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
    let patch = Patch { row: row.clone(), insert_at, survive: 0.0, forecast_delta: 0.0, replace: false, remove: false, root: Some(PatchRoot { text }), below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None };
    let rules = patched_rules(&rec.rules, &patch, max_rows(rec));
    let pos = rules.rows.iter().position(|r| *r == row).unwrap_or(0);
    let mut rp = Replayer::new(&b, &rules, ticks, rec.stall)?;
    let (survive, _fired) = measure(&mut rp, &row, pos);
    Some(Patch { survive, ..patch })
}

/// Cut 18 §4: a stall patch that swaps or deletes a row of the set (only the loop patch does).
pub fn is_loop_patch(p: &Patch) -> bool {
    p.insert_at >= 0 && (p.replace || p.remove)
}

/// Cut 18 §4: on a stall whose cause is the rules' loop (`R2 retreat ↔ explore`), the patch
/// that addresses the row it names: the row narrowed to the moment it is for (a moving row
/// gains `adj ≥ 1` — it steps away from a foe at the elbow, not from one in view) or the row
/// deleted, whichever ends the stall in more of the stall's own replays (ties: narrowed).
fn loop_patch(rec: &DeathRec, base: &Game, ticks: u32) -> Option<Patch> {
    cut_patch(rec, base, ticks, rec.loop_row?)
}

/// The patch that cuts the set's row `at` (Cut 18 §4's loop patch; Cut 19 §4's `row`
/// verdict): the row narrowed to the moment it is for (a moving row at the elbow only; a way
/// home 20 points sooner), or deleted — whichever survives more of the death's own replays
/// (ties: the change over the cut).
fn cut_patch(rec: &DeathRec, base: &Game, ticks: u32, at: usize) -> Option<Patch> {
    let mut best: Option<Patch> = None;
    for mut p in cut_candidates(rec, at) {
        let rules = patched_rules(&rec.rules, &p, max_rows(rec));
        let mut rp = Replayer::new(base, &rules, ticks, rec.stall)?;
        p.survive = measure(&mut rp, &p.row, at).0;
        if best.as_ref().is_none_or(|b| p.survive > b.survive + 1e-9) {
            best = Some(p);
        }
    }
    best
}

/// Cut 25 §2: the ways to cut the set's row `at` (`cut_patch`): narrowed (a moving row at the
/// elbow only; a way home 20 points sooner), then deleted.
fn cut_candidates(rec: &DeathRec, at: usize) -> Vec<Patch> {
    let Some(row) = rec.rules.rows.get(at).cloned() else { return Vec::new() };
    let mut cands: Vec<Patch> = Vec::new();
    let moving = matches!(row.verb.v.as_str(), "retreat" | "back_corridor" | "blink" | "shadowstep" | "vanish" | "smoke");
    if moving && row.conds.len() < 2 && !row.conds.iter().any(|c| c.k == "adj>=") && rec.vocab.conds.iter().any(|c| c.k == "adj>=") {
        let mut narrowed = row.clone();
        narrowed.conds.push(Cond::n("adj>=", 1));
        cands.push(Patch { row: narrowed, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: true, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None });
    }
    // Cut 19 §2/§4: a way home written too late (the walk met the floor) — the same row 20
    // points sooner (`hp < 20% → return` → `hp < 40%`), beside cutting it.
    let late = row.conds.iter().position(|c| c.k == "hp<" && c.n.is_some_and(|n| n <= 50));
    if let (false, true, Some(i)) = (rec.stall, matches!(row.verb.v.as_str(), "return" | "bank"), late) {
        let mut sooner = row.clone();
        sooner.conds[i].n = sooner.conds[i].n.map(|n| n + 20);
        cands.push(Patch { row: sooner, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: true, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None });
    }
    cands.push(Patch { row: row.clone(), insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: false, remove: true, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None });
    cands
}

/// Cut 25 §2 (AN: his own gas retreat paced him at 2 hp until an archer's arrows ended it —
/// DICE, `nothing beats unpatched 0/12`): the death's moment was lost before its checkpoint, so
/// the `row` verdict is also read over the floor — every own row that acted in the death's last
/// `CHASE_TURNS` actions, cut, replayed `FLOOR_REPLAYS` times from the floor's start; the best
/// cut surviving `ROW_BAR` and beating the floor's base by `PATCH_MARGIN`, with that base.
fn floor_row_cause(game: &Game, rec: &DeathRec) -> Option<(Patch, f64)> {
    let (base, ticks) = floor_base(game, rec)?;
    let fbase = {
        let mut rp = Replayer::new(&base, &rec.rules, ticks, false)?;
        (0..FLOOR_REPLAYS).filter(|&i| rp.floor_replay(0xF10B_0000 | i as u64, -99, false).0).count() as f64 / FLOOR_REPLAYS as f64
    };
    if fbase > 1.0 - PATCH_MARGIN + 1e-9 {
        return None;
    }
    let turns = &rec.death.trace.turns;
    let mut ats: Vec<usize> = turns[turns.len().saturating_sub(CHASE_TURNS)..].iter().rev().filter_map(|t| usize::try_from(t.row).ok()).collect();
    ats.dedup();
    let mut seen: Vec<usize> = Vec::new();
    let mut best: Option<Patch> = None;
    for at in ats {
        if seen.contains(&at) || !rec.rules.rows.get(at).is_some_and(cuttable) {
            continue;
        }
        seen.push(at);
        for mut p in cut_candidates(rec, at) {
            let rules = patched_rules(&rec.rules, &p, max_rows(rec));
            let Some(mut rp) = Replayer::new(&base, &rules, ticks, false) else { continue };
            p.survive = (0..FLOOR_REPLAYS).filter(|&i| rp.floor_replay(floor_nonce(&p.row, at, i), at as i32, false).0).count() as f64 / FLOOR_REPLAYS as f64;
            if p.survive >= ROW_BAR - 1e-9 && p.survive - fbase >= PATCH_MARGIN - 1e-9 && best.as_ref().is_none_or(|b| p.survive > b.survive + 1e-9) {
                best = Some(p);
            }
        }
    }
    best.map(|p| (p, fbase))
}

/// Cut 19 §4: a cut row must survive this share of the death's replays for the `row`
/// verdict (and beat the unpatched replays by `PATCH_MARGIN`: a death the rules survive
/// anyway is the dice, whatever acted last).
pub const ROW_BAR: f64 = 0.5;

/// Cut 19 §4: the `row` verdict's patch — the dying action (the trace's last turn) was an own
/// row of the set (a card's row is the card's and a preset row the game's, never the
/// player's; an engagement row is excluded, below), and the set without it
/// (or with it narrowed, `cut_patch`) survives ≥ `ROW_BAR` of the death's replays and beats
/// the baseline by `PATCH_MARGIN`. `None` otherwise.
fn row_cause(rec: &DeathRec, base: &Game, ticks: u32, baseline: f64) -> Option<Patch> {
    // Cut 25 §2 (AN: his own gas retreat fired five times at 2 hp — DICE): every own row that
    // acted in the last `CHASE_TURNS` actions is a cut candidate, not only the dying action's;
    // the one whose cut survives most is the verdict's (ties: the latest to act).
    let turns = &rec.death.trace.turns;
    let tail = &turns[turns.len().saturating_sub(CHASE_TURNS)..];
    let mut ats: Vec<usize> = Vec::new();
    for t in tail.iter().rev() {
        if let Ok(at) = usize::try_from(t.row) {
            if !ats.contains(&at) {
                ats.push(at);
            }
        }
    }
    let mut best: Option<Patch> = None;
    for at in ats {
        let Some(row) = rec.rules.rows.get(at) else { continue };
        if !cuttable(row) {
            continue;
        }
        let Some(p) = cut_patch(rec, base, ticks, at) else { continue };
        if p.survive >= ROW_BAR - 1e-9 && p.survive - baseline >= PATCH_MARGIN - 1e-9 && best.as_ref().is_none_or(|b| p.survive > b.survive + 1e-9) {
            best = Some(p);
        }
    }
    best
}

/// A row whose cut can be the `row` verdict: the player's own — not a card's, not the shipped
/// preset's (Cut 21 §3: a gamble — `drink unknown`, `read unknown` — is the player's whatever its
/// origin: a preset row kept is a row the player kept) — and not a strike: an engagement row's
/// death is the fight's (cutting the set's only strike "survives" by never fighting — the
/// answer there is a row above it, a `gap`). Cut 25 §2 (AN: his own `foe: gas → retreat` fired
/// five times at 2 hp — DICE): a row that moves him (`retreat`, `to corridor`, `shadowstep`)
/// strikes nothing — its cut is his.
fn cuttable(row: &Row) -> bool {
    let gamble = matches!(row.verb.v.as_str(), "drink" | "read") && row.verb.a.as_deref() == Some("unknown");
    let moves = matches!(row.verb.v.as_str(), "retreat" | "back_corridor" | "shadowstep");
    !(row.is_card() || (row.origin.as_deref() == Some("preset") && !gamble) || (crate::turn::targets_foes(&row.verb) && !moves))
}

/// Cut 25 §2 (qa): each own row (not a card's, not the preset's, not an engagement row) that
/// acted in the death's last `CHASE_TURNS` actions, with its cut's survival over the death's
/// replays (`cut_patch`) — a `dice` death has none that survives `ROW_BAR` and beats the base.
pub fn recent_own_cuts(game: &Game, rec: &DeathRec) -> Vec<(usize, f64)> {
    let Some((base, ticks)) = replay_base(game, rec) else { return Vec::new() };
    let turns = &rec.death.trace.turns;
    let mut ats: Vec<usize> = turns[turns.len().saturating_sub(CHASE_TURNS)..].iter().filter_map(|t| usize::try_from(t.row).ok()).collect();
    ats.sort();
    ats.dedup();
    ats.into_iter()
        .filter(|&at| rec.rules.rows.get(at).is_some_and(cuttable))
        .filter_map(|at| cut_patch(rec, &base, ticks, at).map(|p| (at, p.survive)))
        .collect()
}

/// Cut 25 §2: the actions an `order` verdict reads back over (as a chase is read).
pub const ORDER_TURNS: usize = CHASE_TURNS;
/// Cut 25 §2: a moved row must survive this share of the death's replays (and beat the
/// unpatched base by `PATCH_MARGIN`) for the `order` verdict.
pub const ORDER_BAR: f64 = 0.5;

/// Whether `p` moves one of the set's own rows (Cut 25 §2).
pub fn is_move(p: &Patch) -> bool {
    p.moves_from.is_some()
}

/// Cut 25 §2: the order's candidates — every own row of the set that sat under the rows that
/// acted in the death's last `ORDER_TURNS` actions (a row above it won each of them), moved
/// above the topmost of those rows; kept when it acts there in at least `FIRED_BAR` of the
/// death's replays (its conditions held: shadowed while matched), with its survival.
fn order_moves(rec: &DeathRec, base: &Game, ticks: u32) -> Vec<Patch> {
    let turns = &rec.death.trace.turns;
    let tail = &turns[turns.len().saturating_sub(ORDER_TURNS)..];
    let acted: Vec<usize> = tail.iter().filter_map(|t| usize::try_from(t.row).ok()).collect();
    let Some(&top) = acted.iter().min() else { return Vec::new() };
    let jobs: Vec<usize> = ((top + 1)..rec.rules.rows.len()).filter(|j| !acted.contains(j) && !rec.rules.rows[*j].is_card()).collect();
    let max_rows = max_rows(rec);
    let rec_ref: &DeathRec = rec;
    let moved: Vec<Option<Patch>> = crate::forecast::par_map(base, jobs, |base, &j| {
        let p = Patch { row: rec_ref.rules.rows[j].clone(), insert_at: top as i32, survive: 0.0, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: Some(j as i32) };
        let rules = patched_rules(&rec_ref.rules, &p, max_rows);
        let mut rp = Replayer::new(base, &rules, ticks, false)?;
        let (survive, fired) = measure(&mut rp, &p.row, top);
        (fired >= FIRED_BAR - 1e-9).then_some(Patch { survive, ..p })
    });
    moved.into_iter().flatten().collect()
}

/// Cut 22 §4: the last trace turns a chase is read over.
pub const CHASE_TURNS: usize = 5;

/// Cut 22 §4: the attack row that chased into the death — a `row` verdict naming it. The
/// player's own row (not a card's, not the preset's) whose verb strikes a chosen foe (an
/// `attack` / `shoot` at anything but the nearest, or on a condition of its own), that took
/// most of the last `CHASE_TURNS` actions (at least two, the dying one among them or just
/// before a way home), while the set keeps another row that engages foes (cutting it is not
/// "never fight": the fallback strikes instead). Cutting it, or narrowing it to foes beside
/// him (`adj ≥ 1`: no chase through a pack), must survive `ROW_BAR` and beat the unpatched
/// base by `PATCH_MARGIN`. Never at a boss (the counter row is the fight).
fn chase_cause(rec: &DeathRec, base: &Game, ticks: u32, baseline: f64) -> Option<Patch> {
    if rec.boss.is_some() || rec.stall {
        return None;
    }
    let turns = &rec.death.trace.turns;
    let tail = &turns[turns.len().saturating_sub(CHASE_TURNS)..];
    let chase = |r: &Row| {
        matches!(r.verb.v.as_str(), "attack" | "shoot") && !r.is_card() && r.origin.as_deref() != Some("preset") && (r.verb.a.as_deref().is_some_and(|a| a != "nearest") || r.conds.iter().any(|c| c.k != "foes>="))
    };
    let mut counts: BTreeMap<usize, usize> = BTreeMap::new();
    for t in tail {
        if let Ok(i) = usize::try_from(t.row) {
            if rec.rules.rows.get(i).is_some_and(chase) {
                *counts.entry(i).or_insert(0) += 1;
            }
        }
    }
    let (&at, &n) = counts.iter().max_by_key(|(i, n)| (**n, std::cmp::Reverse(**i)))?;
    // The dying action was the chase's, or a way home it left too late (the walk after it).
    let last = tail.last()?;
    let last_row = usize::try_from(last.row).ok().and_then(|i| rec.rules.rows.get(i));
    let ended = last.row == at as i32 || last_row.is_some_and(|r| matches!(r.verb.v.as_str(), "return" | "bank"));
    if n < 2 || !ended {
        return None;
    }
    let fallback = rec.rules.rows.iter().enumerate().any(|(i, r)| i != at && matches!(r.verb.v.as_str(), "attack" | "shoot" | "cleave" | "shield_bash" | "tactic"));
    if !fallback {
        return None;
    }
    let row = rec.rules.rows[at].clone();
    let mut cands = vec![Patch { row: row.clone(), insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: false, remove: true, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }];
    if !row.conds.iter().any(|c| c.k == "adj>=") && rec.vocab.conds.iter().any(|c| c.k == "adj>=") {
        let mut narrowed = row.clone();
        narrowed.conds.push(Cond::n("adj>=", 1));
        cands.insert(0, Patch { row: narrowed, insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: true, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None });
    }
    let mut best: Option<Patch> = None;
    for mut p in cands {
        let rules = patched_rules(&rec.rules, &p, max_rows(rec));
        let mut rp = Replayer::new(base, &rules, ticks, false)?;
        p.survive = measure(&mut rp, &p.row, at).0;
        if best.as_ref().is_none_or(|b| p.survive > b.survive + 1e-9) {
            best = Some(p);
        }
    }
    best.filter(|p| p.survive >= ROW_BAR - 1e-9 && p.survive - baseline >= PATCH_MARGIN - 1e-9)
}

/// Cut 19 §4: `Patch.drops` on every insert of the shown list when the set's own rows are at
/// the cap — the own row that fired least in the dead run (`DeathRec.row_fired`; ties: the
/// lowest in the list, the row the editor's cut would take).
///
/// QA on 1a2a4a9 (qaP: `+ drop R1 …` on all three patches, R1 his return row — an exit row
/// fires once a run, so it was always the least fired): never an exit row (`return` /
/// `bank`), never the row the patch is about (one with the patch's own verb, the row the
/// verdict names, the root's row); the least fired of the rest, else no `drops` (the client's
/// drop sheet decides).
fn set_drops(rec: &mut DeathRec) {
    let full = rec.rules.own_rows() >= rec.vocab.max_rows;
    let about = [rec.death.cause_row.map(|r| r as usize), rec.root.as_ref().map(|r| r.row)];
    let rules = rec.rules.clone();
    let fired = rec.row_fired.clone();
    // QA on 912e135 (qaX: the lead patch `drops R1 · 1/238 fires`, R1 the heal row the death's chain had just credited — `← R1 drank
    // heal at 1/36 hp`): a row that acted in the death's own trace, or that its chain names, is the death's evidence — never dropped.
    let mut acted: Vec<usize> = rec.death.trace.turns.iter().filter(|t| t.row >= 0).map(|t| t.row as usize).collect();
    let links = rec.death.trace.turns.iter().flat_map(|t| t.rows.iter().flatten().filter_map(|w| w.because.as_ref()));
    for b in rec.death.chain.iter().flatten().chain(links) {
        if let Some(n) = b.text.strip_prefix('R').and_then(|x| x.split(' ').next()).and_then(|n| n.parse::<usize>().ok()) {
            acted.push(n.saturating_sub(1));
        }
    }
    let least = |p: &Patch| -> Option<i32> {
        rules
            .rows
            .iter()
            .enumerate()
            .filter(|(i, r)| !r.is_card() && !matches!(r.verb.v.as_str(), "return" | "bank") && r.verb.v != p.row.verb.v && !about.contains(&Some(*i)) && !acted.contains(i))
            .map(|(i, _)| (fired.get(i).copied().unwrap_or(0), i))
            .fold(None, |best: Option<(u32, usize)>, x| if best.is_none_or(|b| x.0 <= b.0) { Some(x) } else { best })
            .map(|(_, i)| i as i32)
    };
    for p in rec.death.patches.iter_mut() {
        let inserts = if p.insert_at < 0 { !rec.rules.rows.contains(&p.row) } else { !(p.replace || p.remove) };
        p.drops = if inserts && full { least(p) } else { None };
    }
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
    // QA on e75ec29: a floor-window verdict's patches were selected on the floor's replays.
    if rec.floor_window && p.insert_at >= 0 && !p.remove && p.root.is_none() {
        let Some((base, ticks)) = floor_base(game, rec) else { return 0.0 };
        let rules = patched_rules(&rec.rules, p, max_rows(rec));
        let pos = if p.replace { p.insert_at as usize } else { rules.rows.iter().position(|r| *r == p.row).unwrap_or(0) };
        let Some(mut rp) = Replayer::new(&base, &rules, ticks, false) else { return 0.0 };
        let fired = (0..FLOOR_REPLAYS).filter(|&i| rp.floor_replay(floor_nonce(&p.row, pos, i), pos as i32, true).1).count();
        return fired as f64 / FLOOR_REPLAYS as f64;
    }
    let Some((mut base, ticks)) = replay_base(game, rec) else { return 0.0 };
    // An unlock pseudo-patch: the token unlocked; the watched row is the set's own or the top.
    if p.root.is_some() {
        unlock_base(&mut base, rec);
    }
    // Cut 19 §4: a cut (the loop patch, the `row` verdict's) acts by the row's absence: it
    // "fires" where the row it cuts acts in the unpatched replays; a swap where its new row does.
    let (rules, pos) = if p.remove && p.insert_at >= 0 {
        (rec.rules.clone(), p.insert_at as usize)
    } else {
        let rules = patched_rules(&rec.rules, p, max_rows(rec));
        let pos = if p.replace && p.insert_at >= 0 { p.insert_at as usize } else { rules.rows.iter().position(|r| *r == p.row).unwrap_or(0) };
        (rules, pos)
    };
    let Some(mut rp) = Replayer::new(&base, &rules, ticks, rec.stall) else { return 0.0 };
    let fired = (0..REPLAYS).filter(|&i| rp.replay(row_nonce(&p.row, pos, i), pos as i32).1).count();
    fired as f64 / REPLAYS as f64
}

/// QA on e75ec29 (qaQ: `hp < 20% → rest · survives 100%` applied, and every later run read
/// `R1 rest · not safe` and died to the same archer): the death's window starts at a
/// checkpoint where the hero is often already low (the ring holds 300 ticks), and a row that
/// acts there — a rest while the killer is out of view — need not act at all in a run. The
/// share of `FLOOR_REPLAYS` reseeded replays of the patched set from the floor's start
/// (`DeathRec.floor`) in which the patch's row acts before he leaves the floor (or the death's
/// own time on it, plus `ENCOUNTER_TICKS`, runs out). `None` when the record has no floor start
/// or the patch is not an inserted row of its own (an unlock, a cut, a root's answer).
pub fn floor_fired(game: &Game, rec: &DeathRec, p: &Patch) -> Option<f64> {
    if p.insert_at < 0 || p.remove || p.root.is_some() {
        return None;
    }
    let (base, ticks) = floor_base(game, rec)?;
    let rules = patched_rules(&rec.rules, p, max_rows(rec));
    let pos = if p.replace { p.insert_at as usize } else { rules.rows.iter().position(|r| *r == p.row)? };
    let mut rp = Replayer::new(&base, &rules, ticks, false)?;
    let need = (FLOOR_REPLAYS as f64 * FIRED_BAR).ceil() as u32;
    let (mut fired, mut missed) = (0u32, 0u32);
    for i in 0..FLOOR_REPLAYS {
        if rp.floor_replay(floor_nonce(&p.row, pos, i), pos as i32, true).1 {
            fired += 1;
        } else {
            missed += 1;
        }
        // Early out once the bar is decided either way (the share is only compared to it).
        if fired >= need || missed > FLOOR_REPLAYS - need {
            break;
        }
    }
    Some(fired as f64 / (fired + missed).max(1) as f64)
}

/// Replays from the floor's start per shown patch (`floor_fired`).
pub const FLOOR_REPLAYS: u32 = 12;

/// QA on e75ec29: the patches whose row acts in under `FIRED_BAR` of the floor replays
/// (`floor_fired`) are not offered — unless none acts, then the list stands (a dice death still
/// names what was tried) unless `allow_empty`; a pinned boss counter and a stall's loop patch
/// stay whatever.
fn drop_floor_silent(game: &Game, rec: &mut DeathRec, allow_empty: bool) {
    if rec.floor.is_none() || rec.death.patches.is_empty() {
        return;
    }
    let counter = rec.counter.clone();
    let jobs: Vec<Patch> = rec.death.patches.clone();
    let rec_ref: &DeathRec = rec;
    let rates: Vec<Option<f64>> = crate::forecast::par_map(game, jobs, |g, p| {
        if counter.as_ref() == Some(&p.row) || is_loop_patch(p) {
            return None;
        }
        floor_fired(g, rec_ref, p)
    });
    let silent = |i: usize| rates[i].is_some_and(|r| r < FIRED_BAR - 1e-9);
    if !allow_empty && (0..rates.len()).all(silent) {
        return;
    }
    let mut i = 0;
    rec.death.patches.retain(|_| {
        let keep = !silent(i);
        i += 1;
        keep
    });
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
    // Cut 18 §4: a stall whose cause is the rules' loop measures its rows just above the row
    // the cause names as well, and takes that place on a tie — the way out then acts where the
    // loop did (`R2 retreat ↔ explore`: before R2); the loop patch itself swaps or deletes it.
    let loop_at = rec.loop_row.filter(|_| rec.stall);
    let mut positions = insert_positions(&rec.rules, &rec.death.trace);
    if let Some(at) = loop_at {
        if !positions.contains(&at) {
            positions.push(at);
        }
    }
    // Baseline: the unpatched rules under the same reseeded replays. If they survive most of
    // the time the death was the dice, not the policy; a patch must beat the baseline clearly.
    let baseline = match Replayer::new(&base, &rec.rules, ticks, rec.stall) {
        Some(mut rp) => (0..REPLAYS).filter(|&i| rp.replay(0xBA5E_0000 | i as u64, -99).0).count() as f64 / REPLAYS as f64,
        None => 0.0,
    };
    rec.death.baseline = baseline;
    let bar = survive_bar(baseline);
    // Cut 19 §4: the dying action was a row of the set, and cutting that row saves him.
    // Cut 22 §4 (AH: R4 `attack ranged` chasing an archer through six foes, sealed GAP): an
    // attack row that chased into the death is the player's too — like a gamble, whatever an
    // added row would survive.
    let chase_cut = if rec.stall { None } else { chase_cause(rec, &base, ticks, baseline) };
    let row_cut = if rec.stall || chase_cut.is_some() { None } else { row_cause(rec, &base, ticks, baseline) };
    let mut cands = match &rec.t10 {
        Some(t10) if rec.stall => stall_candidates(&rec.vocab, t10),
        Some(t10) => candidates(&rec.vocab, &rec.rules, t10, &rec.t10_facts, &game.lineage.flavours, &rec.death.trace),
        None => Vec::new(),
    };
    // A row the set already carries is not a patch (cohort 8, rater O: `foe: boss → attack
    // boss` offered at the top while it sat at R2). Moving a row is the editor's job.
    cands.retain(|r| !rec.rules.rows.iter().any(|x| x.conds == r.conds && x.verb == r.verb));
    // QA on 0c6e126: a row using an item the next heir can neither carry nor buy is no patch.
    let camp = camp_state(game);
    cands.retain(|r| pack_need(&camp, r).is_some());
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
    // Cut 14 §2: did the item's own row save him? (the margin's `heal unused` / `N unknown
    // unused`, written below once the verdict is known)
    let uses = |row: &Row, a: &str| matches!(row.verb.v.as_str(), "drink" | "read") && row.verb.a.as_deref() == Some(a);
    let heal_saves = scored.iter().any(|(rate, row, _)| uses(row, "heal") && *rate >= MARGIN_BAR - 1e-9);
    // QA on 1a2a4a9: per kind — a `drink unknown` that saved him speaks for the potions, a
    // `read unknown` for the scrolls.
    let unknown_saves = |verb: &str| scored.iter().any(|(rate, row, _)| row.verb.v == verb && uses(row, "unknown") && *rate >= MARGIN_BAR - 1e-9);
    let unknown_saves = (unknown_saves("drink"), unknown_saves("read"));
    // Rank by how much the row beats the unpatched baseline; ties: a conditioned row (a policy)
    // beats an unconditioned one, then fewer conditions, then the top position.
    scored.sort_by(|a, b| {
        (b.0 - baseline)
            .partial_cmp(&(a.0 - baseline))
            .unwrap()
            .then(a.1.conds.is_empty().cmp(&b.1.conds.is_empty()))
            .then(a.1.conds.len().cmp(&b.1.conds.len()))
            .then((Some(b.2) == loop_at).cmp(&(Some(a.2) == loop_at)))
            .then(a.2.cmp(&b.2))
    });
    let mut edge_gap = scored.first().is_some_and(|best| best.0 - baseline >= PATCH_MARGIN - 1e-9);
    // QA on e75ec29: a cut that survives `SURVIVE_BAND` less than an added row is not the
    // verdict (`survival_first` would take its head): the missing row is — a `gap`.
    let boss = rec.boss.is_some();
    let row_cut = row_cut.filter(|c| !scored.iter().any(|(rate, row, _)| (!row.conds.is_empty() || rate - baseline >= 0.3 - 1e-9) && !(boss && family(row) == "escape") && *rate > c.survive + SURVIVE_BAND + 1e-9));
    // Cut 21 §3: the gamble's harm killed him — his own unknown row is the verdict, whatever
    // an added row would survive (the cut leads the patches as any `row` verdict's does).
    // (A death the unpatched replays all survive was not reproduced: never `row` — the dice.)
    let mut gamble_at = rec.gamble_row.filter(|_| !rec.stall && baseline < 1.0 - 1e-9);
    let own_cut = gamble_at.map(|at| cut_patch(rec, &base, ticks, at).unwrap_or_else(|| Patch { row: rec.rules.rows[at].clone(), insert_at: at as i32, survive: 0.0, forecast_delta: 0.0, replace: false, remove: true, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }));
    // QA on 778fa1b (qaV): a card's own throw or gamble is the verdict when the set without the
    // card lives longer than it did (the card's cut beats the unpatched replays) — otherwise
    // the moment killed him whatever the card did, and the verdict is the moment's.
    if gamble_at.is_some_and(|at| rec.rules.rows[at].is_card()) && own_cut.as_ref().is_some_and(|p| p.survive <= baseline + 1e-9) {
        gamble_at = None;
        rec.gamble_row = None;
    }
    let chased = gamble_at.is_none() && chase_cut.is_some();
    if chased {
        rec.chase_row = chase_cut.as_ref().map(|p| p.insert_at.max(0) as usize);
    }
    let row_cut = match gamble_at {
        Some(_) => own_cut,
        None => row_cut.or(chase_cut),
    };
    // Cut 25 §2: the order — a row of the set that would have acted, under a row that won every
    // tick; moved up it survives `ORDER_BAR` and beats the base by `PATCH_MARGIN`: `order`
    // (unless the player's own gamble dealt the death, a cut of the set survives more, or an
    // added row survives `SURVIVE_BAND` more). Every move that acts is a patch whatever.
    rec.moves = if rec.stall { Vec::new() } else { order_moves(rec, &base, ticks) };
    let best_added = scored.iter().filter(|(_, row, _)| !(boss && family(row) == "escape")).map(|(rate, _, _)| *rate).fold(f64::MIN, f64::max);
    let order = rec
        .moves
        .iter()
        .filter(|p| p.survive >= ORDER_BAR - 1e-9 && p.survive - baseline >= PATCH_MARGIN - 1e-9)
        .max_by(|a, b| a.survive.partial_cmp(&b.survive).unwrap().then(b.moves_from.cmp(&a.moves_from)))
        .filter(|p| gamble_at.is_none() && row_cut.as_ref().is_none_or(|c| p.survive >= c.survive - 1e-9) && p.survive >= best_added - SURVIVE_BAND - 1e-9)
        .cloned();
    let row_cut = if order.is_some() { None } else { row_cut };
    if order.is_some() {
        rec.chase_row = None;
    }
    let mut patches: Vec<Patch> = scored.into_iter().map(|(rate, row, pos)| Patch { row, insert_at: pos as i32, survive: rate, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }).collect();
    // Cut 11 §2: the chain's root — the theft's answer or the unlock — measured at the top,
    // scored like any other (its edge counts for the verdict; its delta is simulated first).
    if let Some(r) = root_patch(game, rec, &base, ticks) {
        edge_gap |= r.survive - baseline >= PATCH_MARGIN - 1e-9;
        rec.root_under_base = r.survive < baseline - 1e-9;
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
    // A loop patch that never ends the stall in its replays is not offered (the rows above it
    // already stand at the loop's row).
    if let Some(p) = loop_patch(rec, &base, ticks).filter(|p| p.survive > 1e-9) {
        rec.death.patches.insert(0, p);
    }
    // Cut 19 §4: the `row` verdict's patch (the row cut) leads, as a stall's loop patch does.
    if let Some(p) = &row_cut {
        rec.death.patches.retain(|x| !(x.row == p.row && x.insert_at == p.insert_at));
        // Cut 21 §3: a gamble's cut that survives `SURVIVE_BAND` less than the best shown
        // follows it (the head is always within the band of the best); the verdict still
        // names the row.
        let best = rec.death.patches.iter().filter(|x| !(boss && family(&x.row) == "escape")).map(|x| x.survive).fold(f64::MIN, f64::max);
        let at = if (gamble_at.is_some() || chased) && p.survive < best - SURVIVE_BAND - 1e-9 { 1.min(rec.death.patches.len()) } else { 0 };
        rec.death.patches.insert(at, p.clone());
    }
    if rec.stall {
        // Cut 13 §1: a stall is its own verdict; its patches are ranked by the forecast like
        // a death's, but the word on the screen is `stall`.
        rec.death.verdict = "stall".into();
    } else if let Some(p) = &order {
        // Cut 25 §2: the player's order killed him — `order`, naming both rows (`R5 under R2`).
        rec.death.verdict = "order".into();
        rec.death.cause_row = p.moves_from.map(|f| f as u32);
        rec.death.order_over = Some(p.insert_at as u32);
    } else if let Some(p) = &row_cut {
        // Cut 19 §4 (rater AA: `R2 drink unknown fired` → GAP on the fire potion he drank): the
        // row the player wrote killed him — `row`, naming it; `gap` is for a missing row.
        rec.death.verdict = "row".into();
        rec.death.cause_row = Some(p.insert_at as u32);
    } else if edge_gap {
        rec.death.verdict = "gap".into();
    } else {
        // No patch survives the moment clearly: the forecast decides (a row that costs no
        // survival here but gains floors is still a gap in the policy).
        forecast_deltas(game, rec, VERDICT_DELTA_CANDIDATES, true);
        // Cut 14 §1: a row under the baseline is never offered (`drop_under_base`), so its
        // floors make a gap only while something at or over the baseline is left to show —
        // else the moment was the dice, and the dice fallback names what was tried.
        let offered = rec.death.patches.iter().any(|p| p.survive >= baseline - 1e-9);
        // QA on e75ec29 (qaQ: `survives 100% · base 100%` under GAP on the rules that just
        // died): unpatched replays that all survive do not reproduce the death — the moment
        // was the dice, whatever a row's floors (the dice screen says `nothing beats base`).
        let reproduced = baseline < 1.0 - 1e-9;
        if reproduced && offered && rec.death.patches.iter().any(|p| p.forecast_delta >= DELTA_BAR - 1e-9) {
            rec.death.verdict = "gap".into();
        }
    }
    // QA on e75ec29 (qaQ: `survives 100% · base 100%`; qaR: DICE beside `survives 100% · base
    // 92%`): when the moment's replays hardly reproduce the death (a base no patch can clear by
    // `PATCH_MARGIN`), the verdict widens its window to the floor's start (`floor_verdict`).
    let (mut heal_saves, mut unknown_saves) = (heal_saves, unknown_saves);
    // Cut 25 §2: a dice death whose own recent row, cut, survives the floor — `row`, on the floor's numbers.
    if rec.death.verdict == "dice" && !rec.stall && rec.root.is_none() && rec.boss.is_none() && gamble_at.is_none() {
        if let Some((p, fbase)) = floor_row_cause(game, rec) {
            rec.death.verdict = "row".into();
            rec.death.cause_row = Some(p.insert_at as u32);
            rec.death.baseline = fbase;
            rec.death.patches = vec![p];
            rec.floor_window = true;
            rec.deltas_n = 0;
            rec.deltas_done = false;
        }
    }
    if rec.death.verdict == "dice" && !rec.stall && baseline >= 1.0 - PATCH_MARGIN - 1e-9 && rec.root.is_none() && rec.boss.is_none() {
        if let Some((h, u)) = floor_verdict(game, rec, &cands, pinnable.as_ref()) {
            heal_saves = h;
            unknown_saves = u;
        }
    }
    margin_lines(rec, heal_saves, unknown_saves);
}

/// QA on e75ec29: the floor-window verdict. The unpatched set replayed `FLOOR_REPLAYS` times
/// from the floor's start (`DeathRec.floor`) — the base; when that dies at least `PATCH_MARGIN`
/// of the time, each candidate row at the top is replayed the same way, and one that survives
/// `PATCH_MARGIN` more than the base and acts in ≥ `FIRED_BAR` of its replays makes the death a
/// `gap` measured on the floor (`DeathRec.floor_window`: the baseline and every patch's
/// survival are the floor's). `None` (the death stays `dice`) otherwise. Returns the margin's
/// (heal saves, unknown saves) from the floor's numbers.
fn floor_verdict(game: &Game, rec: &mut DeathRec, cands: &[Row], pinnable: Option<&Row>) -> Option<(bool, (bool, bool))> {
    let (base, ticks) = floor_base(game, rec)?;
    let fbase = {
        let mut rp = Replayer::new(&base, &rec.rules, ticks, false)?;
        (0..FLOOR_REPLAYS).filter(|&i| rp.floor_replay(0xF10B_0000 | i as u64, -99, false).0).count() as f64 / FLOOR_REPLAYS as f64
    };
    if fbase > 1.0 - PATCH_MARGIN + 1e-9 {
        return None;
    }
    let need = fbase + PATCH_MARGIN;
    let jobs: Vec<usize> = (0..cands.len()).filter(|&i| pinnable != Some(&cands[i])).collect();
    let rec_ref: &DeathRec = rec;
    let scored: Vec<Option<(f64, Row)>> = crate::forecast::par_map(&base, jobs, |base, &ci| {
        let row = &cands[ci];
        let rules = patched(rec_ref, row, 0);
        let mut rp = Replayer::new(base, &rules, ticks, false)?;
        let (mut survived, mut failed, mut fired) = (0u32, 0u32, 0u32);
        for i in 0..FLOOR_REPLAYS {
            let (s, f) = rp.floor_replay(floor_nonce(row, 0, i), 0, false);
            survived += s as u32;
            failed += !s as u32;
            fired += f as u32;
            // Early outs: the bar out of reach, or the row too rarely acting.
            if failed as f64 > FLOOR_REPLAYS as f64 * (1.0 - need) + 1e-9 || (i + 1 - fired) as f64 > FLOOR_REPLAYS as f64 * (1.0 - FIRED_BAR) + 1e-9 {
                return None;
            }
        }
        let rate = survived as f64 / FLOOR_REPLAYS as f64;
        (rate >= need - 1e-9 && fired as f64 >= FLOOR_REPLAYS as f64 * FIRED_BAR - 1e-9).then(|| (rate, row.clone()))
    });
    let mut scored: Vec<(f64, Row)> = scored.into_iter().flatten().collect();
    if scored.is_empty() {
        return None;
    }
    let uses = |row: &Row, v: &str, a: &str| row.verb.v == v && row.verb.a.as_deref() == Some(a);
    let heal = scored.iter().any(|(r, row)| (uses(row, "drink", "heal") || uses(row, "read", "heal")) && *r >= MARGIN_BAR - 1e-9);
    let unknown = (scored.iter().any(|(r, row)| uses(row, "drink", "unknown") && *r >= MARGIN_BAR - 1e-9), scored.iter().any(|(r, row)| uses(row, "read", "unknown") && *r >= MARGIN_BAR - 1e-9));
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap().then(a.1.conds.is_empty().cmp(&b.1.conds.is_empty())).then(a.1.conds.len().cmp(&b.1.conds.len())));
    let patches: Vec<Patch> = scored.into_iter().map(|(rate, row)| Patch { row, insert_at: 0, survive: rate, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }).collect();
    rec.death.patches = one_per_family(patches);
    rec.death.baseline = fbase;
    rec.death.verdict = "gap".into();
    rec.floor_window = true;
    rec.deltas_n = 0;
    rec.deltas_done = false;
    Some((heal, unknown))
}

/// Cut 14 §2: the margin's `· heal unused` and `· N unknown unused` — only on a `gap`, and
/// only when the candidate that uses the item (`hp<N → drink heal`, `→ drink | read unknown`)
/// survived at least `MARGIN_BAR` of the replays and reached the death's bar (it is among the
/// patches): the line then never contradicts the verdict. A `dice` death's margin is the HP
/// line alone; what would have helped is the patch the screen names (Cut 11 §4).
///
/// QA on 1a2a4a9 (qaO: `heal unused · 1 unknown unused` beside `R1 no unknown`, R1 `drink
/// unknown` and the one unknown a scroll): the count is of the unknowns the set could have
/// used — potions when a row drinks unknowns, scrolls when one reads them (either, while the
/// set has no such row) — and only the kind whose own candidate saved him; and never beside a
/// row the death's last turn says had `no unknown`. `heal unused` needs a known heal held.
fn margin_lines(rec: &mut DeathRec, heal_saves: bool, unknown_saves: (bool, bool)) {
    if rec.death.verdict != "gap" {
        return;
    }
    if rec.heal_held && heal_saves {
        rec.death.margin.push_str(" · heal unused");
    }
    let (potions, scrolls) = unknown_unused(rec, unknown_saves);
    let n = potions + scrolls;
    if n > 0 {
        // QA on 912e135 (qaW: `3 unknown unused` beside `left` listing six unknowns, one a
        // potion): the count names what it counts — the unknowns held that the set could use,
        // by kind (`3 unknown potions unused`); both kinds keep the bare noun.
        let what = match (potions, scrolls) {
            (1, 0) => "unknown potion",
            (_, 0) => "unknown potions",
            (0, 1) => "unknown scroll",
            (0, _) => "unknown scrolls",
            _ => "unknowns",
        };
        rec.death.margin.push_str(&format!(" · {n} {what} unused"));
        // QA on e75ec29 (qaQ: `2 unknown unused` on a `curious · drinks unknowns` hero): the
        // trait uses an unknown only when clear (no foe in view, ≥ 50 % HP, once a floor —
        // `turn::decide`); the row that would have used them in the fight is the gap, and the
        // margin says why the trait did not.
        // QA on 0c6e126 (qaY: `curious: clear only` beside an unidentified `clear potion?` in the
        // bones — "clear" read as the flavour): the word is the moment's, not a potion's.
        if rec.t10.as_ref().is_some_and(|r| r.trait_ == crate::hero::Trait::Curious) {
            rec.death.margin.push_str(" · curious: safe only");
        }
    }
    // QA on 778fa1b (qaV: `ogre · D5 · GAP` while the trace's five turns were `R2 attack nearest`
    // at 4 HP into `ogre winds up` — "`row`, or a line saying why the attack row is not the
    // cause"): an engagement row striking into the killer's telegraph is the fight's, not the
    // row's (cutting the strike "survives" by never fighting); the gap is the row above it that
    // answers the telegraph, and the margin says so.
    // QA on 912e135 (qaX: `telegraph unanswered · GAP` over one patch, `hp < 40% → return`, that answers no telegraph): the
    // margin names the gap only while a patch that answers it is offered (a `foe: telegraph` row), as `heal unused` does.
    let answered = rec.death.patches.iter().any(|p| p.row.conds.iter().any(|c| c.k == "foe_tag" && c.t.as_deref() == Some("telegraph")));
    if answered && telegraph_unanswered(rec) {
        rec.death.margin.push_str(" · telegraph unanswered");
    }
}

/// The dying action was a foe-facing row of the set while the killer showed its telegraph
/// (`ogre winds up` on the trace's last turn).
pub fn telegraph_unanswered(rec: &DeathRec) -> bool {
    let Some(last) = rec.death.trace.turns.last() else { return false };
    let cause = rec.death.cause.as_str();
    // (a heavy blow wound up in plain sight — an ogre's, a slag crawler's: an archer's draw is
    // its shot, and the answer to it is the strike)
    if last.row < 0 || !crate::turn::targets_foes(&last.verb) || !crate::defs::MONSTERS.iter().any(|m| m.kind == cause && m.tags.contains(&"telegraph") && m.tags.contains(&"heavy")) {
        return false;
    }
    let word = monster_def(cause).title.split_whitespace().last().unwrap_or("foe").to_lowercase();
    last.telegraphs.iter().any(|t| t.strip_prefix(word.as_str()).is_some_and(|r| r.starts_with(' ')))
}

/// The margin's `N unknown unused` count (`margin_lines`).
fn unknown_unused(rec: &DeathRec, (potion_saves, scroll_saves): (bool, bool)) -> (u32, u32) {
    let reads = |v: &str| rec.rules.rows.iter().any(|r| r.verb.v == v && r.verb.a.as_deref() == Some("unknown"));
    let (drinks, reads) = (reads("drink"), reads("read"));
    let any_row = drinks || reads;
    let said_none = rec.death.trace.turns.last().and_then(|t| t.rows.as_ref()).is_some_and(|rows| rows.iter().any(|w| w.why == "no unknown"));
    if said_none {
        return (0, 0);
    }
    let scrolls = rec.unknown_scrolls.min(rec.unknown_held);
    let potions = rec.unknown_held - scrolls;
    let p = if potion_saves && (drinks || !any_row) { potions } else { 0 };
    let s = if scroll_saves && (reads || !any_row) { scrolls } else { 0 };
    (p, s)
}

/// Cut 14 §2: the survival a consumable's row must reach for the margin to call it unused.
pub const MARGIN_BAR: f64 = 0.5;

/// Cut 14 §1: nothing under the baseline is offered (cohort 10, rater S: "a patch worse than
/// base — survives 42 % · base 50 % — was offered first") — the root patch and the pinned
/// counter included (a lock root's number is the unlock sheet's delta, `meta::
/// catalogue_with_deltas`). A `dice` death's candidates kept under the bar (Cut 11 §4,
/// `below_bar`) stay: the client renders them as what was tried, not as advice.
/// QA on 0c6e126 (qaY): the named item a row uses — `drink invisibility`, `read teleport`,
/// `throw fire,tag:boss` — none for `unknown`, a verb that uses no item, or an unknown kind.
pub fn row_item(row: &Row) -> Option<&str> {
    if !matches!(row.verb.v.as_str(), "drink" | "read" | "throw") {
        return None;
    }
    let a = row.verb.a.as_deref()?.split(',').next()?;
    (a != "unknown" && crate::defs::ITEMS.iter().any(|d| d.kind == a)).then_some(a)
}

/// QA on 0c6e126: the kinds the next heir packs from `camp` (the camp a patch is applied in,
/// `camp_state`): the shelf (bought, found and free supplies), the loadout's vault items, and
/// the repeat's kinds (the send re-packs them).
pub fn next_pack_kinds(camp: &Game) -> BTreeSet<String> {
    let l = &camp.lineage;
    let mut out: BTreeSet<String> = l.supplies.iter().map(|s| s.kind.clone()).collect();
    out.extend(camp.loadout.iter().filter_map(|id| l.vault.iter().find(|v| v.id == *id)).map(|v| v.kind.clone()));
    if !l.restock_off {
        out.extend(l.last_supplies.iter().filter(|k| !l.last_wasted.contains(k)).cloned());
    }
    out
}

/// QA on 0c6e126 (qaY: `drink invisibility · survives 92%` offered and applied, and the next
/// heir carried none): whether a patch row can be offered to the next heir — `Some(None)` when
/// he carries what it uses (or it uses no named item), `Some(Some(buy))` when the shop sells
/// it and the purse and the shelf can take it now (offered with its purchase), `None` when
/// neither (not offered: a replay with the item in hand measures a pack he will not have).
pub fn pack_need(camp: &Game, row: &Row) -> Option<Option<crate::wire::PatchBuy>> {
    let Some(k) = row_item(row) else { return Some(None) };
    if next_pack_kinds(camp).contains(k) {
        return Some(None);
    }
    let e = camp.supply_catalogue().into_iter().find(|e| e.kind == k)?;
    let l = &camp.lineage;
    (l.gold >= e.price && l.supplies.len() < l.supply_cap()).then(|| Some(crate::wire::PatchBuy { kind: k.to_string(), label: e.label, price: e.price }))
}

/// QA on 0c6e126: each patch's purchase (`Patch.buys`), and no patch the next heir could
/// neither carry nor buy (`pack_need`).
fn mark_buys(game: &Game, rec: &mut DeathRec) {
    let camp = camp_state(game);
    rec.death.patches.retain_mut(|p| match pack_need(&camp, &p.row) {
        Some(b) => {
            p.buys = b;
            true
        }
        None => false,
    });
}

/// The game a patch's reach is measured on: `g` with the patch's purchase made (`Patch.buys`).
fn with_buy(g: &Game, p: &Patch) -> Option<Game> {
    let b = p.buys.as_ref()?;
    let mut c = g.sim_clone();
    c.buy_supply(&b.kind).ok()?;
    Some(c)
}

fn drop_under_base(rec: &mut DeathRec) {
    let base = rec.death.baseline;
    rec.death.patches.retain(|p| p.below_bar || p.survive >= base - 1e-9);
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
        // Cut 18 §4: a stall's loop patch leads (it addresses the row the cause names).
        if let Some(i) = rec.death.patches.iter().position(is_loop_patch) {
            let l = rec.death.patches.remove(i);
            rec.death.patches.insert(0, l);
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
        } else if let Some(g) = with_buy(game, &p) {
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
/// edge (the best by edge). Cut 14 §1: the same six the death screen measures — with
/// nothing under the baseline in the list, a high-baseline death's few remaining rows (a
/// `rest` or `descend` that never fires in the window, at the baseline exactly) are the
/// ones whose floors decide it, and three by edge did not always reach them.
pub const VERDICT_DELTA_CANDIDATES: usize = DELTA_CANDIDATES;

/// Fill in each patch's full-forecast delta at the death's depth and shape the shown list.
pub fn compute_deltas(game: &Game, rec: &mut DeathRec) {
    if rec.shaped {
        return;
    }
    rec.shaped = true;
    shape_patches(game, rec);
    splice_moves(rec);
}

/// Cut 25 §2: the moves join the shaped list — an `order` verdict's move leads it; every other
/// move that acted (a shadowed-while-matched row) follows the shown patches, under the bar when
/// it does not beat it. A move is the set's own row: it adds, drops and buys nothing.
fn splice_moves(rec: &mut DeathRec) {
    if rec.moves.is_empty() {
        return;
    }
    let baseline = rec.death.baseline;
    let bar = survive_bar(baseline).max(baseline);
    let mut moves = rec.moves.clone();
    for p in moves.iter_mut() {
        p.below_bar = p.survive < bar - 1e-9 || p.survive <= baseline + 1e-9;
        p.exits = patch_exits(p);
    }
    let lead = (rec.death.verdict == "order").then(|| moves.iter().position(|p| p.moves_from.map(|f| f as u32) == rec.death.cause_row && Some(p.insert_at as u32) == rec.death.order_over)).flatten();
    if let Some(i) = lead {
        let p = moves.remove(i);
        rec.death.patches.insert(0, p);
        rec.death.patches.truncate(SHOWN);
    }
    moves.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap().then(a.moves_from.cmp(&b.moves_from)));
    rec.death.patches.extend(moves);
}

fn shape_patches(game: &Game, rec: &mut DeathRec) {
    drop_under_base(rec);
    mark_buys(game, rec);
    // QA on e75ec29: a row the floor never reaches is not offered (`floor_fired`); a dice
    // death's list may empty here — its fallback then names rows that do act.
    drop_floor_silent(game, rec, rec.death.verdict == "dice");
    // Cut 11 §4: a `dice` death with nothing over the bar still names its alternative — the
    // best candidates measured in full, flagged `below_bar`, with their deltas.
    if rec.death.verdict == "dice" {
        if rec.death.patches.is_empty() {
            dice_fallback(game, rec);
        }
        dice_telegraph(game, rec);
    }
    // Cut 19 §2: a stall with no way out that survives (a return now walks, and a floor's
    // wall can stop it) still names what was tried, under the bar.
    if rec.stall && rec.death.patches.is_empty() {
        dice_fallback(game, rec);
    }
    drop_floor_silent(game, rec, false);
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
    let keeps_below = is_dice || rec.stall;
    rec.death.patches.retain(|p| {
        let edge = p.survive - baseline;
        counter.as_ref() == Some(&p.row) || p.root.is_some() || is_loop_patch(p) || (keeps_below && p.below_bar) || ((edge >= PATCH_MARGIN - 1e-9 || p.forecast_delta >= DELTA_BAR - 1e-9) && (!p.row.conds.is_empty() || edge >= 0.3))
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
    if rec.death.patches.iter().all(is_loop_patch) {
        let mut all: Vec<Patch> = pre_retain.iter().filter(|p| !is_loop_patch(p)).cloned().collect();
        all.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap().then(b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap()));
        rec.death.patches.extend(one_per_family(all));
        boss_order(rec);
        rec.death.patches.truncate(SHOWN);
    }
    // Cut 18 §4: a stall's loop patch is shown first whatever the ranking did (its row is
    // the one the cause names), unless it is hopeless beside a surviving alternative (below);
    // the way out stays beside it (the fallback above counts it out).
    if let Some(l) = pre_retain.iter().find(|p| is_loop_patch(p)) {
        rec.death.patches.retain(|p| !is_loop_patch(p));
        rec.death.patches.insert(0, l.clone());
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
        // Under the bar — or under the baseline itself (a bar capped at `SURVIVE_BAR` sits
        // under a high baseline): either way the screen says `nothing beats base`.
        let bar = survive_bar(baseline).max(baseline);
        for p in rec.death.patches.iter_mut() {
            p.below_bar = p.survive < bar - 1e-9 || (p.root.is_none() && p.survive <= baseline + 1e-9) || rec.low_fired.contains(&p.row);   // equal to base is not better (QA on 56f2a1d: `survives 100% · base 100%` offered)
        }
    }
    // Cut 15 §6: a below-the-bar candidate that survives 0 % is not shown (U: `survives 0% ·
    // base 0%`): the dice screen names only alternatives that survive.
    // 0 % candidates go (U: `survives 0% · base 0%`) — unless nothing else survives: a dice
    // death still names its alternative (Cut 11 §4), even a hopeless one.
    // … and any 0 % row (the pinned boss counter too — QA on 3d71c33: `attack boss · survives
    // 0%` beside a 100 % candidate); the root's unlock pseudo-patch is measured apart.
    let hopeless = |p: &Patch| p.insert_at >= 0 && p.survive <= 1e-9;
    if rec.death.patches.iter().any(|p| !hopeless(p)) {
        rec.death.patches.retain(|p| !hopeless(p));
    }
    // Cut 6 §8: on a boss death a `return` is never the only patch — the best other scored
    // candidate joins it (giving up is not the answer to a wall).
    if rec.boss.is_some() && rec.death.patches.len() == 1 && family(&rec.death.patches[0].row) == "escape" {
        let mut others: Vec<Patch> = pre_retain.into_iter().filter(|p| family(&p.row) != "escape" && !(p.below_bar && p.survive <= 1e-9)).collect();
        others.sort_by(|a, b| b.survive.partial_cmp(&a.survive).unwrap().then(b.forecast_delta.partial_cmp(&a.forecast_delta).unwrap()));
        if let Some(mut o) = others.into_iter().next() {
            // QA on the Cut 20 gate (`boss → hit boss · survives 0%` beside a return that
            // survives): a 0 % joiner is named as what was tried — below the bar, not advice.
            if o.survive <= 1e-9 && rec.death.patches[0].survive > 1e-9 {
                o.below_bar = true;
            }
            rec.death.patches.insert(0, o);
        }
    }
    survival_first(rec);
    // QA on 92eb880: a dice death none of whose shown patches survives more than the
    // unpatched rules (a 100 % baseline: the replays win the fight he lost) says so.
    rec.death.nothing_beats_base = is_dice && !rec.death.patches.is_empty() && rec.death.patches.iter().all(|p| p.survive <= baseline + 1e-9);
    set_drops(rec);
    mark_exits(&mut rec.death.patches);
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
    let below_bar = survive < survive_bar(rec.death.baseline) - 1e-9 || survive <= rec.death.baseline + 1e-9;
    rec.death.patches.push(Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None });
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
    if let Some(r) = telegraph_row(&rec.vocab, &rec.death.trace).filter(|_| !rec.stall) {
        rows.push(r);
    }
    // The candidate list, one family each (the telegraph retreat keeps its family), measured
    // in order until `DICE_CANDIDATES` have fired in half their replays. Cut 19 §2: a stall
    // none of whose ways out survives names what was tried the same way (its own candidates).
    let cands = if rec.stall { stall_candidates(&rec.vocab, &t10) } else { candidates(&rec.vocab, &rec.rules, &t10, &rec.t10_facts, &game.lineage.flavours, &rec.death.trace) };
    let camp = camp_state(game);
    for r in cands {
        if rec.rules.rows.iter().any(|x| x.conds == r.conds && x.verb == r.verb) || pack_need(&camp, &r).is_none() {
            continue;
        }
        if !rows.iter().any(|x| family(x) == family(&r)) {
            rows.push(r);
        }
    }
    let mut measured: Vec<(Patch, f64)> = Vec::new();
    for row in rows {
        // Cut 15 §6: only a candidate that survives counts toward the list (a 0 % one is
        // dropped below), so the search goes on down the candidates past them.
        if measured.iter().filter(|(p, f)| *f >= FIRED_BAR && p.survive > 1e-9).count() >= DICE_CANDIDATES {
            break;
        }
        let Some(mut rp) = Replayer::new(&base, &patched(rec, &row, 0), ticks, rec.stall) else { continue };
        let (survive, fired) = measure(&mut rp, &row, 0);
        measured.push((Patch { row, insert_at: 0, survive, forecast_delta: 0.0, replace: false, remove: false, root: None, below_bar: true, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }, fired));
    }
    // Cut 15 §6: a candidate that survives 0 % is no alternative (U: `survives 0% · base 0%`).
    if measured.iter().any(|(p, _)| p.survive > 1e-9) {
        measured.retain(|(p, _)| p.survive > 1e-9);
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
            rec.low_fired.push(p.row.clone());
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

/// Cut 19 §4: survival decides a rank when two patches' survival differs by more than this;
/// within it, the forecast's reach does.
pub const SURVIVE_BAND: f64 = 0.10;

/// Rank the patches: survival first, reach inside `SURVIVE_BAND` (Cut 19 §4, rater AA: `+ drop
/// one … survives 33 %` on a `+0 %` reach led two `survives 100 %` rows). Each place goes to
/// the patch with the best reach (when any patch moves the forecast by `DELTA_BAR`; else the
/// best survival) among those within the band of the best survival left — an order, not a
/// pairwise rule (which is not transitive). A patch below `DELTA_SINK` (a row that survives
/// the moment but costs floors) yields its place to any other within the band; QA on
/// e75ec29 (qaQ: `rest · 83%` above two `100%` rows whose ranking reach was −8 %, one of
/// twelve sims): it no longer sinks under patches that survive more than the band less.
/// Ties keep the incoming order.
///
/// QA on 778fa1b (qaU: `hp < 40% → return · survives 100%` led `read unknown · 67%`; applied,
/// the camp read `D5 −54`): an exit patch whose reach costs `EXIT_COST` or more
/// (`costly_exit`) survives the moment by ending the run early — it does not set the band's
/// top, nor take a place, while a patch that is not one beats the baseline by
/// `PATCH_MARGIN` (`exit_alternative`): it does not lead (from the second place on the
/// order is the usual one — the safe way home stays on the list, named for its cost). At
/// equal survival and reach (a reach still pending reads alike), a non-exit patch goes first.
pub fn rank_patches(patches: &mut [Patch], baseline: f64) {
    let by_delta = patches.iter().any(|p| p.forecast_delta > DELTA_BAR);
    let mut pool: Vec<Patch> = patches.to_vec();
    let mut out: Vec<Patch> = Vec::with_capacity(patches.len());
    while !pool.is_empty() {
        let alt = out.is_empty() && exit_alternative(&pool, baseline);
        let eligible = |p: &Patch| !(alt && costly_exit(p));
        let top = pool.iter().filter(|p| eligible(p)).map(|p| p.survive).fold(f64::NEG_INFINITY, f64::max);
        let within = |p: &Patch| eligible(p) && p.survive >= top - SURVIVE_BAND - 1e-9;
        let any_afloat = pool.iter().any(|p| within(p) && p.forecast_delta >= DELTA_SINK);
        let mut best: Option<usize> = None;
        for (i, p) in pool.iter().enumerate() {
            if !within(p) || (any_afloat && p.forecast_delta < DELTA_SINK) {
                continue;
            }
            let better = best.is_none_or(|b| {
                let q = &pool[b];
                let (d, s) = (p.forecast_delta - q.forecast_delta, p.survive - q.survive);
                let tie = d.abs() <= 1e-9 && s.abs() <= 1e-9;
                if tie {
                    !patch_exits(p) && patch_exits(q)
                } else if by_delta {
                    d > 1e-9 || (d.abs() <= 1e-9 && s > 1e-9)
                } else {
                    s > 1e-9 || (s.abs() <= 1e-9 && d > 1e-9)
                }
            });
            if better {
                best = Some(i);
            }
        }
        out.push(pool.remove(best.expect("a patch within the band")));
    }
    patches.clone_from_slice(&out);
}

/// QA on 778fa1b: the patch's row ends the run (`return` / `bank`): `Patch.exits`.
pub fn exits(row: &Row) -> bool {
    matches!(row.verb.v.as_str(), "return" | "bank")
}

/// QA on 778fa1b: an exit patch's reach cost (the forecast's move, a 0..1 share) at or past
/// which it is `costly_exit` — two sims in the verdict's twelve, five in the camp's fifty.
pub const EXIT_COST: f64 = 0.10;

/// QA on 778fa1b: an exit patch that buys the moment with floors (`forecast_delta ≤
/// −EXIT_COST`, the verdict's estimate or the camp's number).
/// (An insert only: a cut that narrows the set's own exit row — the `row` verdict's answer —
/// keeps its place by the verdict's rules.)
pub fn costly_exit(p: &Patch) -> bool {
    patch_exits(p) && !p.replace && p.forecast_delta <= -EXIT_COST + 1e-9
}

/// The patch puts an exit row in the set (a cut of one does not: it removes it).
pub fn patch_exits(p: &Patch) -> bool {
    exits(&p.row) && !p.remove
}

/// QA on 778fa1b: among `patches`, one that is not a costly exit, is advice (not
/// `below_bar`, inserts a row or cuts one) and beats `baseline` by `PATCH_MARGIN` — while
/// one does, a costly exit neither sets the best survival nor leads.
pub fn exit_alternative(patches: &[Patch], baseline: f64) -> bool {
    patches.iter().any(|p| !costly_exit(p) && !p.below_bar && p.insert_at >= 0 && p.survive >= baseline + PATCH_MARGIN - 1e-9)
}

/// Whether `p` counts toward the best survival the head must stay within `SURVIVE_BAND` of
/// (`survival_first`, and the qa invariant): not the escape family on a boss death (Cut 6
/// §8), not a costly exit beside an alternative (QA on 778fa1b).
pub fn counts_as_best(p: &Patch, all: &[Patch], baseline: f64, boss: bool) -> bool {
    let boss_escape = boss && family(&p.row) == "escape";
    let costly = costly_exit(p) && exit_alternative(all, baseline);
    // Cut 25 §2: a move is the order's lesson, listed after the shown patches (it leads only on `order`).
    !boss_escape && !costly && !is_move(p)
}

/// Set `exits` on each patch (QA on 778fa1b).
pub fn mark_exits(patches: &mut [Patch]) {
    for p in patches.iter_mut() {
        p.exits = patch_exits(p);
    }
}

/// QA on e75ec29 (qaQ: the gem applied `92 %` above `100 %`, `83 %` above two `100 %`): the
/// patch the gem applies survives within `SURVIVE_BAND` of the best shown. A pinned head (the
/// boss counter, a cut, the root, a dice death's telegraph answer) keeps it only so; else the
/// first patch in the list within the band of the best leads. On a boss death the escape
/// family is not counted as the best (Cut 6 §8: giving up is not the answer to a wall).
/// A `row` verdict whose cut loses the head this way is a `gap` (a missing row saves more).
fn survival_first(rec: &mut DeathRec) {
    // Cut 25 §2: an `order` verdict's move leads whatever (the player's own row, misplaced).
    if rec.death.verdict == "order" {
        return;
    }
    let boss = rec.boss.is_some();
    let all = rec.death.patches.clone();
    let baseline = rec.death.baseline;
    let counts = |p: &Patch| counts_as_best(p, &all, baseline, boss);
    let Some(best) = rec.death.patches.iter().filter(|p| counts(p)).map(|p| p.survive).reduce(f64::max) else { return };
    let ok = |p: &Patch| counts(p) && p.survive >= best - SURVIVE_BAND - 1e-9;
    if rec.death.patches.first().is_none_or(ok) {
        return;
    }
    let Some(i) = rec.death.patches.iter().position(ok) else { return };
    // (Cut 22 §4: only when the head that lost was the verdict's cut — a gamble's or a chase's
    // cut that already follows the best keeps its `row`.)
    let head_cut = rec.death.patches.first().is_some_and(|h| (h.remove || h.replace) && rec.death.cause_row == Some(h.insert_at.max(0) as u32));
    let p = rec.death.patches.remove(i);
    rec.death.patches.insert(0, p);
    // A gamble's or a chase's cut follows the best and the verdict still names the row.
    let own = rec.death.cause_row.map(|r| r as usize);
    if head_cut && own.is_some() && (rec.gamble_row == own || rec.chase_row == own) {
        return;
    }
    if rec.death.verdict == "row" && head_cut {
        rec.death.verdict = "gap".into();
        rec.death.cause_row = None;
    }
}

/// The full death for a run id, computing verdict and deltas on first request.
pub fn death(game: &mut Game, run_id: u32) -> Option<Death> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    compute_deltas(game, &mut rec);
    let mut d = rec.death.clone();
    // QA on 23ed91f: the camp's numbers are `death_deltas`'s (four camp panels — seconds in
    // wasm): until measured on this camp state, the shown patches carry the verdict's own
    // ranking deltas, flagged `camp_pending`.
    let fresh = rec.camp_key != 0 && rec.camp_key == camp_key(game);
    for p in d.patches.iter_mut() {
        p.camp_pending = !fresh;
    }
    game.deaths.insert(run_id, rec);
    Some(d)
}

/// QA on 23ed91f: the death's shown patches with the camp's own reach deltas (`camp_deltas`):
/// `forecast_delta` is the camp bar's move at `forecast_depth` once the patch is applied, on
/// the camp's panel and lineage state, `forecast_pm` that bar's ±. Memoised: the panels land
/// in `Game.panel_cache` (the camp after the tap reads them) and the numbers on the record.
pub fn death_deltas(game: &mut Game, run_id: u32) -> Option<Vec<Patch>> {
    let mut rec = game.deaths.get(&run_id)?.clone();
    compute_verdict(game, &mut rec);
    compute_deltas(game, &mut rec);
    camp_deltas(game, &mut rec);
    let d = rec.death.patches.clone();
    game.deaths.insert(run_id, rec);
    Some(d)
}

fn camp_key(game: &Game) -> u64 {
    crate::forecast::lineage_key(&camp_state(game)).max(1)
}

/// The camp a patch is applied in: the game with its pending exit resolved as the night
/// resolves it (a death's is empty; a stall's keeps by `keep_pref`), no run.
pub fn camp_state(game: &Game) -> Game {
    let mut g = game.sim_clone();
    g.run = None;
    if let Some(p) = &game.pending_exit {
        g.pending_exit = Some(p.clone());
        g.auto_keep();
    }
    g
}

/// QA on 23ed91f (`hp < 20% → rest · reach +8%`, then the camp's D6 34 % → 64 %; `to corridor
/// · reach +8%`, then D5 79 % → 78 %): the ranking deltas are 12 paired sims on their own
/// seeds, which the camp never shows. A shown patch's `forecast_delta` is the camp's own:
/// its bar at `forecast_depth` after the patch (applied as the client applies it — `insert_at`,
/// the set fitted to `max_rows`) less the bar before, on the camp's panel seeds and the camp's
/// lineage state; `forecast_pm` is that bar's 95 % half-width. Measured again when the camp
/// state moved since (a trait, a purchase, the shelf).
fn camp_deltas(game: &Game, rec: &mut DeathRec) {
    if rec.death.patches.is_empty() {
        return;
    }
    let g = camp_state(game);
    let key = crate::forecast::lineage_key(&g).max(1);
    if rec.camp_key == key {
        return;
    }
    rec.camp_key = key;
    let depth = (rec.death.depth + 1).min(g.lineage.best_depth + 1).max(1);
    let (base, _) = crate::forecast::camp_reach(&g, &rec.rules, depth);
    let max_rows = max_rows(rec);
    let mut unlocked = g.sim_clone();
    unlock_base(&mut unlocked, rec);
    for p in rec.death.patches.iter_mut() {
        let rules = patched_rules(&rec.rules, p, max_rows);
        // QA on 0c6e126: a patch offered with its purchase is measured with it bought.
        let bought = with_buy(&g, p);
        let (r, n) = crate::forecast::camp_reach(if p.insert_at < 0 { &unlocked } else { bought.as_ref().unwrap_or(&g) }, &rules, depth);
        p.forecast_delta = r - base;
        p.forecast_depth = depth;
        p.forecast_pm = crate::forecast::half_width(r, n as usize);
        p.camp_pending = false;
    }
    // Cut 19 §4: the camp's numbers rank the list again (survival first, reach within the
    // band) — the pinned heads keep their places.
    rerank_free(rec);
    survival_first(rec);
    mark_exits(&mut rec.death.patches);
    // The camp reads these panels next (the base, the tapped patch's set).
    for c in [g, unlocked] {
        for (k, v) in c.panel_cache.into_inner() {
            crate::forecast::panel_insert(game, k, v);
        }
    }
}

/// Cut 19 §4: `rank_patches` over the shown list's unpinned patches, in their own slots — the
/// pinned ones (the boss counter, a cut, the root patch, a dice death's telegraph answer) stay
/// where `compute_deltas` put them; on a boss death the escape family stays below the rest.
fn rerank_free(rec: &mut DeathRec) {
    let counter = rec.counter.clone();
    let tele = if rec.death.verdict == "dice" { telegraph_row(&rec.vocab, &rec.death.trace) } else { None };
    let pinned = |p: &Patch| counter.as_ref() == Some(&p.row) || is_loop_patch(p) || is_move(p) || p.root.is_some() || tele.as_ref() == Some(&p.row);
    let slots: Vec<usize> = (0..rec.death.patches.len()).filter(|&i| !pinned(&rec.death.patches[i])).collect();
    let mut free: Vec<Patch> = slots.iter().map(|&i| rec.death.patches[i].clone()).collect();
    rank_patches(&mut free, rec.death.baseline);
    if rec.boss.is_some() {
        let (rest, esc): (Vec<Patch>, Vec<Patch>) = free.into_iter().partition(|p| family(&p.row) != "escape");
        free = rest.into_iter().chain(esc).collect();
    }
    for (k, &i) in slots.iter().enumerate() {
        rec.death.patches[i] = free[k].clone();
    }
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
        let cands = candidates(&rec.vocab, &rec.rules, rec.t10.as_ref().unwrap(), &rec.t10_facts, &g.lineage.flavours, &rec.death.trace);
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
        Patch { row: Row::new(vec![Cond::n("hp<", 20)], verb), insert_at: 0, survive, forecast_delta: delta, replace: false, remove: false, root: None, below_bar: false, forecast_depth: 0, forecast_pm: 0.0, camp_pending: false, drops: None, exits: false, buys: None, moves_from: None }
    }

    #[test]
    fn a_patch_that_moves_the_forecast_outranks_a_safer_one_that_does_not() {
        // Survival within the band: the reach decides.
        let mut ps = vec![patch(Verb::new("return"), 1.0, 0.0), patch(Verb::arg("drink", "unknown"), 0.92, 0.05)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::arg("drink", "unknown"));
        // Cut 19 §4 (rater AA): a patch that survives much less does not lead on a small reach gain.
        let mut ps = vec![patch(Verb::arg("drink", "unknown"), 0.33, 0.05), patch(Verb::new("return"), 1.0, -0.02), patch(Verb::new("retreat"), 1.0, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["retreat", "return", "drink"]);
        // No patch moves the forecast: the survival edge decides.
        let mut ps = vec![patch(Verb::arg("drink", "unknown"), 0.4, 0.01), patch(Verb::new("return"), 1.0, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::new("return"));
        // A patch that costs floors yields its place to any other within the band …
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.2), patch(Verb::new("retreat"), 0.95, 0.0), patch(Verb::arg("drink", "unknown"), 0.92, 0.05)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["drink", "retreat", "return"]);
        // … but never to one that survives more than the band less (QA on e75ec29, qaQ: `rest ·
        // 83%` led two `100%` rows whose ranking reach was one sim in twelve under zero).
        let mut ps = vec![patch(Verb::new("rest"), 0.83, 0.167), patch(Verb::new("back_corridor"), 1.0, -0.083), patch(Verb::new("descend"), 1.0, -0.083)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["back_corridor", "descend", "rest"]);
        // QA on 778fa1b (qaU): an exit that costs `EXIT_COST` of reach does not lead while a
        // patch beats the base by `PATCH_MARGIN` — the best of the others does, and the exit
        // keeps the second place (the safe way home, named for its cost).
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.2), patch(Verb::new("retreat"), 0.7, 0.0), patch(Verb::arg("drink", "unknown"), 0.65, 0.05)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["drink", "return", "retreat"]);
        // (seed 2015's first death: `return · 100 % · D6 −52` over `read unknown · 67 % · +0`)
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.52), patch(Verb::arg("read", "unknown"), 0.67, 0.0), patch(Verb::new("retreat"), 0.25, -0.06)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["read", "return", "retreat"]);
        // With no alternative over the base's margin the exit leads as before …
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.52), patch(Verb::new("retreat"), 0.1, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::new("return"));
        // … and a cheap exit (reach within `EXIT_COST`) is ranked like any patch.
        let mut ps = vec![patch(Verb::new("return"), 1.0, -0.05), patch(Verb::arg("read", "unknown"), 0.67, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps[0].row.verb, Verb::new("return"));
        // At equal survival and reach (a reach still pending), the non-exit patch goes first.
        let mut ps = vec![patch(Verb::new("return"), 1.0, 0.0), patch(Verb::new("retreat"), 1.0, 0.0)];
        rank_patches(&mut ps, 0.0);
        assert_eq!(ps.iter().map(|p| p.row.verb.v.as_str()).collect::<Vec<_>>(), ["retreat", "return"]);
        // `exits` names the rows that end the run; a cut of one does not.
        let mut ps = vec![patch(Verb::new("return"), 1.0, 0.0), patch(Verb::new("bank"), 1.0, 0.0), patch(Verb::new("retreat"), 1.0, 0.0)];
        ps[1].remove = true;
        mark_exits(&mut ps);
        assert_eq!(ps.iter().map(|p| p.exits).collect::<Vec<_>>(), [true, false, false]);
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
