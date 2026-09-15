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

fn hl(run: &Run, pattern: &str, score: i32, t: u32, text: String) -> Highlight {
    Highlight { pattern: pattern.into(), score, t, run_id: run.id, text: crate::chronicle::clamp_words(&text, 8) }
}

/// `first_kills`: kinds killed in this run for the first time in the lineage.
pub fn sift_with(run: &Run, first_kills: &[String]) -> Vec<Highlight> {
    let mut out = Vec::new();
    let alive_end = run.over.is_some_and(|t| t != crate::engine::ExitTier::Death);
    let mut near: Vec<u32> = run.near_deaths.clone();
    if alive_end {
        if let Some(t) = run.low10_t {
            near.push(t);
        }
    }
    for t in near {
        out.push(hl(run, "near_death", NEAR_DEATH, t, "Near death, then the floor survived.".into()));
    }
    if let Some(t20) = run.low20_t.or(run.low10_t) {
        for (bt, kind) in &run.boss_kills {
            if *bt > t20 {
                out.push(hl(run, "comeback", COMEBACK, *bt, format!("Comeback: slew the {} from the brink.", kind_title(kind))));
            }
        }
    }
    for (t, kind, _) in &run.kills {
        if first_kills.contains(kind) && !out.iter().any(|h| h.pattern == "first_kill" && h.text.contains(&kind_title(kind))) {
            out.push(hl(run, "first_kill", FIRST_KILL, *t, format!("First kill: {}.", kind_title(kind))));
        }
    }
    for (t, kind) in &run.ally_lost {
        out.push(hl(run, "ally_lost", ALLY_LOST, *t, format!("Lost the {} on D{}.", kind_title(kind), run.depth)));
    }
    for (t, kind, mal) in &run.gambles {
        let survived = *mal && (run.gambles_survived.iter().any(|(gt, _)| gt == t) || alive_end);
        let score = GAMBLE + if survived { 2 } else { 0 };
        let text = if *mal { format!("Gambled and survived the {}.", kind.replace('_', " ")) } else { format!("Gambled: it was {}.", kind.replace('_', " ")) };
        out.push(hl(run, "gamble", score, *t, text));
    }
    for (t, label) in &run.stolen {
        out.push(hl(run, "stolen", STOLEN, *t, format!("A monkey stole the {label}.")));
    }
    for (t, kind) in &run.boss_kills {
        out.push(hl(run, "boss", BOSS, *t, format!("Slew the {}.", kind_title(kind))));
    }
    out.sort_by(|a, b| b.score.cmp(&a.score).then(a.t.cmp(&b.t)));
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
