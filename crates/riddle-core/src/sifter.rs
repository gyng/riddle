//! The sifter: scores a run's moments into highlights. Reel = top 5 by score.
use crate::engine::{kind_title, LineageState, Run};
use crate::wire::Highlight;

pub const NEAR_DEATH: i32 = 5;
pub const COMEBACK: i32 = 8;
pub const FIRST_KILL: i32 = 3;
pub const ALLY_LOST: i32 = 4;
pub const GAMBLE: i32 = 2;
pub const STOLEN: i32 = 2;
pub const BOSS: i32 = 6;
/// Cut 2 §2: recovering a named heir's bones.
pub const BONES: i32 = 6;

fn hl(run: &Run, pattern: &str, score: i32, t: u32, text: String) -> Highlight {
    Highlight { pattern: pattern.into(), score, t, run_id: run.id, text: crate::chronicle::clamp_words(&text, 8) }
}

/// Cut 4: how the run ended, as the tail of a reel line (`banked $313` · `returned with $80`
/// · `fell on D4`); empty while the run is live.
pub fn end_phrase(run: &Run) -> String {
    use crate::engine::ExitTier;
    match run.over {
        Some(ExitTier::Bank) => format!("banked ${}", run.loot.max(0)),
        Some(ExitTier::Return) if run.timed_out => "lost the thread".into(),
        Some(ExitTier::Return) => format!("returned with ${}", run.loot.max(0) * ExitTier::Return.pct() / 100),
        Some(ExitTier::Death) => format!("fell on D{}", run.depth),
        None => String::new(),
    }
}

/// `first_kills`: kinds killed in this run for the first time in the lineage. Cut 4: every
/// line is setup + turn + end in ≤ 8 words (`Down to 2 HP, then banked $313.`); `bones` and
/// `first_kill` keep their shape.
pub fn sift_with(run: &Run, first_kills: &[String]) -> Vec<Highlight> {
    let mut out = Vec::new();
    let alive_end = run.over.is_some_and(|t| t != crate::engine::ExitTier::Death);
    let end = end_phrase(run);
    let then = |s: &str| if end.is_empty() { format!("{s}.") } else { format!("{s}, then {end}.") };
    let comma = |s: &str| if end.is_empty() { format!("{s}.") } else { format!("{s}, {end}.") };
    let mut near: Vec<u32> = run.near_deaths.clone();
    if alive_end {
        if let Some(t) = run.low10_t {
            near.push(t);
        }
    }
    let low_hp = if run.low_hp == i32::MAX { 1 } else { run.low_hp.max(1) };
    for t in near {
        out.push(hl(run, "near_death", NEAR_DEATH, t, then(&format!("Down to {low_hp} HP"))));
    }
    if let Some(t20) = run.low20_t.or(run.low10_t) {
        for (bt, kind) in &run.boss_kills {
            if *bt > t20 {
                out.push(hl(run, "comeback", COMEBACK, *bt, comma(&format!("Brink, slew the {}", kind_title(kind)))));
            }
        }
    }
    for (t, kind, _) in &run.kills {
        if kind.starts_with("spectral_") {
            continue; // summons are not a first kill
        }
        if first_kills.contains(kind) && !out.iter().any(|h| h.pattern == "first_kill" && h.text.contains(&kind_title(kind))) {
            out.push(hl(run, "first_kill", FIRST_KILL, *t, format!("First kill: {}.", kind_title(kind))));
        }
    }
    for (t, kind) in &run.ally_lost {
        out.push(hl(run, "ally_lost", ALLY_LOST, *t, comma(&format!("Lost the {} on D{}", kind_title(kind), run.depth))));
    }
    for (t, kind, mal) in &run.gambles {
        let survived = *mal && (run.gambles_survived.iter().any(|(gt, _)| gt == t) || alive_end);
        let score = GAMBLE + if survived { 2 } else { 0 };
        let text = if *mal { comma(&format!("Gambled, survived the {}", kind.replace('_', " "))) } else { comma(&format!("Gambled: it was {}", kind.replace('_', " "))) };
        out.push(hl(run, "gamble", score, *t, text));
    }
    for (t, label) in &run.stolen {
        out.push(hl(run, "stolen", STOLEN, *t, comma(&format!("A monkey stole the {label}"))));
    }
    for (t, kind) in &run.boss_kills {
        out.push(hl(run, "boss", BOSS, *t, comma(&format!("Slew the {}", kind_title(kind)))));
    }
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.t.cmp(&b.t)));
    // One entry per pattern per run: the highest-scoring instance (earliest on ties).
    let mut seen: Vec<String> = Vec::new();
    out.retain(|h| {
        if seen.contains(&h.pattern) {
            false
        } else {
            seen.push(h.pattern.clone());
            true
        }
    });
    out
}

/// Highlights for a finished run against the lineage before its kills were banked.
pub fn sift(run: &Run, l: &LineageState) -> Vec<Highlight> {
    let first: Vec<String> = run.kills.iter().map(|(_, k, _)| k.clone()).filter(|k| !l.kills.contains(k)).collect();
    sift_with(run, &first)
}

pub fn reel(highlights: &[Highlight]) -> Vec<Highlight> {
    let mut v = highlights.to_vec();
    v.sort_by(|a, b| b.score.cmp(&a.score).then(a.run_id.cmp(&b.run_id)).then(a.t.cmp(&b.t)));
    v.truncate(5);
    v
}
