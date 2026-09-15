//! Chronicle notes (DCSS-style auto-notes), ≤ 8 words each.
use crate::engine::{Ctx, Run};
use crate::wire::Ev;

pub fn note(run: &mut Run, cx: &mut Ctx, text: String) {
    let text = clamp_words(&text, 8);
    cx.events.push(Ev::Note { t: run.turn, text: text.clone() });
    run.notes.push((run.turn, text));
    if run.notes.len() > 200 {
        run.notes.remove(0);
    }
}

pub fn callout(run: &Run, cx: &mut Ctx, text: &str) {
    cx.events.push(Ev::Callout { t: run.turn, text: clamp_words(text, 3) });
}

pub fn clamp_words(s: &str, max: usize) -> String {
    let words: Vec<&str> = s.split_whitespace().collect();
    if crate::rules::word_count(s) <= max {
        return s.to_string();
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clamps_long_text() {
        let s = "one two three four five six seven eight nine ten";
        assert_eq!(clamp_words(s, 8), "one two three four five six seven eight");
        assert_eq!(clamp_words("HP 31% → drink", 3), "HP 31% → drink");
    }
}
