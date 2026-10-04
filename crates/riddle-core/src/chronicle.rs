//! Chronicle notes (DCSS-style auto-notes), ≤ 8 words each.
use crate::engine::{Ctx, Run};
use crate::wire::Ev;

pub fn note(run: &mut Run, cx: &mut Ctx, text: String) {
    let text = clamp_words_owned(text, 8);
    cx.events.push(Ev::Note { t: run.turn, text: text.clone() });
    run.notes.push((run.turn, text));
    if run.notes.len() > 48 {
        run.notes.remove(0);
    }
}

/// Cut 24 §2 (AK: "the shrine line, run after run"): the one-line floor events, each a pool of
/// variants (≤ 8 words; each keeps its kind's opening words — `A shrine.` · `A cage:` · `The air
/// stings:` — which the client's beat test reads).
pub const EVENT_POOLS: &[(&str, &[&str])] = &[
    // Cut 28 §4 (AV: `R2 return saved him.` four runs running): the row that got him through a low,
    // in turn (`{r}` is `R2 return`), never the same words three runs running.
    ("saved", &["{r} saved him.", "{r} got him out.", "{r} pulled him through."]),
    ("shrine", &["A shrine. Pray, at a price.", "A shrine. Its candles still burn.", "A shrine. Coins on the step.", "A shrine. The idol watches him."]),
    ("vault", &["A cage: three inside, one to take.", "A cage: three things behind bars.", "A cage: take one, leave two.", "A cage: three prizes, one key."]),
    ("nest", &["A den. Something sleeps.", "A den. Breathing in the dark.", "A den. Gold among the bones.", "A den. Soft snoring ahead."]),
    ("den", &["A den of thieves.", "A den of thieves. Small, quick hands.", "A den of thieves. Something glints.", "A den of thieves. Whispers, then quiet."]),
    ("den_wakes", &["The den wakes: thieves on every side.", "The den wakes: hands everywhere.", "The den wakes: they were never asleep."]),
    ("lock", &["The air stings: bloats ahead.", "The air stings: bloats in the doors.", "The air stings: a bloat reek.", "The air stings: something swollen waits."]),
    ("captive", &["A cry from the dark: a captive, chained.", "A cry from the dark: chains rattle.", "A cry from the dark: someone begs."]),
    ("hunger", &["The floor is hungry. Find light.", "The floor is hungry. It feeds on him.", "The floor is hungry. Light keeps it back."]),
    ("pass_lock", &["Through the lock, barely touched.", "Past the bloats, still breathing.", "The lock behind him, clean."]),
    ("pass_captive", &["The captive fights for him.", "Freed, the captive fights beside him.", "The captive takes up a blade."]),
    ("pass_hunger", &["Light held.", "The light held; the hunger starved.", "Lit, the floor let him go."]),
    ("cut_captive", &["Cut the captive down. The stairs are clear.", "Cut the captive down. No one weeps.", "Cut the captive down. Silence after."]),
    // One new event kind per biome (`situations::omen`).
    ("omen:warrens", &["A loose brick: coins behind it.", "A rat's hoard: coins in the straw.", "A dropped purse in the dust."]),
    ("omen:burrows", &["A dead delver's pack.", "Old bones clutch a flask.", "A satchel under the rubble."]),
    ("omen:fens", &["A clean spring: he drinks.", "Rain through a crack: he drinks.", "Sweet water in the reeds: he drinks."]),
    ("omen:crypt", &["A bell tolls below.", "Bells in the dark: they stir.", "A far bell: something wakes."]),
    ("omen:foundry", &["A cooling ingot: worth coin.", "Slag with silver in it.", "A smith's lost purse."]),
    ("omen:deep", &["A drowned purse in the pool.", "Coins in a lurker's leavings.", "A glint under the black water."]),
    ("omen:sanctum", &["Offerings on a cold altar.", "Coins in a dry font.", "A pilgrim's purse, forgotten."]),
];

/// The pool of a floor event's kind.
/// Cut 28 §4: a `saved` note (`R2 return saved him.`, `… got him out.`, `… pulled him through.`).
pub fn is_saved_note(n: &str) -> bool {
    pool("saved").iter().any(|p| n.ends_with(p.trim_start_matches("{r}")))
}

pub fn pool(kind: &str) -> &'static [&'static str] {
    EVENT_POOLS.iter().find(|(k, _)| *k == kind).map(|(_, p)| *p).unwrap_or(&[])
}

/// Cut 24 §2: a floor event's line — never one this run or the last two showed while one is
/// left (`Run.event_recent`), else the one shown longest ago; the first choice turns with the
/// run's seed. Recorded for the lineage (`Run.event_used`).
pub fn variant(run: &mut Run, kind: &str) -> String {
    let pool = pool(kind);
    if pool.is_empty() {
        return String::new();
    }
    let n = pool.len();
    let recent = run.event_recent.get(kind).cloned().unwrap_or_default();
    let start = (crate::rng::hash_str(kind) ^ run.seed) as usize % n;
    let pick = (0..n).map(|i| (start + i) % n).find(|i| !recent.contains(&(*i as u8))).unwrap_or_else(|| {
        // all shown lately: the one whose last showing is oldest
        (0..n).min_by_key(|i| recent.iter().rposition(|r| *r as usize == *i).map_or(0, |p| p + 1)).unwrap_or(0)
    });
    run.event_recent.entry(kind.to_string()).or_default().push(pick as u8);
    run.event_used.push((kind.to_string(), pick as u8));
    pool[pick].to_string()
}

pub fn callout(run: &Run, cx: &mut Ctx, text: &str) {
    cx.events.push(Ev::Callout { t: run.turn, text: clamp_words(text, 3), why: None });
}

/// Cut 23 §3: a callout with its reason on tap (≤ 3 words).
pub fn callout_why(run: &Run, cx: &mut Ctx, text: &str, why: Option<&str>) {
    cx.events.push(Ev::Callout { t: run.turn, text: clamp_words(text, 3), why: why.map(|w| clamp_words(w, 3)) });
}

pub fn clamp_words(s: &str, max: usize) -> String {
    if crate::rules::word_count(s) <= max {
        return s.to_string();
    }
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut out: Vec<&str> = Vec::new();
    let mut n = 0;
    for w in words {
        if w.chars().any(|c| c.is_alphanumeric()) {
            n += 1;
            if n > max {
                break;
            }
        }
        out.push(w);
    }
    out.join(" ")
}

/// Keep an already-owned short line without allocating a second string.
pub fn clamp_words_owned(s: String, max: usize) -> String {
    if crate::rules::word_count(&s) <= max { s } else { clamp_words(&s, max) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn event_pools_hold_three_and_fit() {
        for (k, p) in EVENT_POOLS {
            assert!(p.len() >= 3, "{k}");
            for l in p.iter() {
                assert!(crate::rules::word_count(l) <= 8, "{l}");
            }
        }
    }
    #[test]
    fn clamps_long_text() {
        let s = "one two three four five six seven eight nine ten";
        assert_eq!(clamp_words(s, 8), "one two three four five six seven eight");
        assert_eq!(clamp_words("HP 31% → drink", 3), "HP 31% → drink");
    }
    #[test]
    fn clamping_preserves_short_spacing_and_counts_unicode_words() {
        for (input, max, expected) in [
            ("  α  → β \t", 2, "  α  → β \t"),
            ("  α  → β γ \t", 2, "α → β"),
            ("→ α β", 0, "→"),
            ("  → ! \t", 0, "  → ! \t"),
            ("HP 40% → drink heal", 3, "HP 40% → drink"),
        ] {
            assert_eq!(clamp_words(input, max), expected);
            assert_eq!(clamp_words_owned(input.to_owned(), max), expected);
        }
    }
}
