//! Cut 27 §1 (P2: "runs start where the doubt is; solved floors fold"): the watch's fold. The
//! sim still plays every floor — economy and determinism unchanged — but the floors the set
//! clears ≥ 95 % (the camp's panel at the send: `Game::fold_plan`) are played here in one call
//! and handed to the watch as one line, with every state change on them as a beat.
use crate::engine::{Game, HERO_ID};
use crate::wire::{Ev, FoldBeat, FoldFloor, FoldLine, Snapshot, StepResult};

/// A hero hurt to this share of his max hp (or under) is a death-adjacent dip: a beat.
pub const DIP_SHARE: f64 = 0.30;
/// The fold never plays more than this many ticks (a run's cap is far above a folded band's).
pub const FOLD_MAX_TICKS: u32 = 60_000;
/// Every this many ticks the floor's snapshot takes in what came into view since (`merge`).
const MERGE_EVERY: u32 = 10;

/// One folded floor while it is played.
struct Acc {
    depth: u32,
    snapshot: Snapshot,
    events: Vec<Ev>,
    loot0: i32,
    /// (hp, max, t) — the floor's lowest hero hp at or under `DIP_SHARE`.
    dip: Option<(i32, i32, u32)>,
    beats: Vec<FoldBeat>,
}

impl Game {
    /// Cut 27 §1: play the live run through the floors the send's set clears ≥ 95 % (from the
    /// floor it stands on to `fold_plan`'s last) and return the fold line (`FoldLine`). Nothing
    /// to fold — no plan, the run's floor below the bar, no live run — plays no tick.
    pub fn fold(&mut self) -> FoldLine {
        self.watched = true;
        let plan = self.fold_plan.take();
        let live = self.run.as_ref().filter(|r| r.over.is_none()).map(|r| r.depth);
        let from = live.unwrap_or(1);
        let to = plan.as_ref().map(|p| p.0).unwrap_or(0);
        let clear_of = |d: u32| plan.as_ref().and_then(|p| p.1.iter().find(|(x, _)| *x == d).map(|(_, c)| *c)).unwrap_or(1.0);
        if live.is_none() || to < from {
            let snapshot = self.last_snapshot.clone().unwrap_or_else(|| self.snapshot());
            return FoldLine { hp: 0, max_hp: 0, low: false, from, to: from.saturating_sub(1), clear: 1.0, gold: 0, beats: Vec::new(), chips: Vec::new(), floors: Vec::new(), step: StepResult { calm: Vec::new(), events: Vec::new(), snapshot, run_over: false, exit_pending: None } };
        }
        let mut all: Vec<Ev> = Vec::new();
        let first = std::mem::take(&mut self.events);
        let loot0 = self.run.as_ref().map_or(0, |r| r.loot);
        let mut acc = Acc { depth: from, snapshot: self.snapshot(), events: Vec::new(), loot0, dip: None, beats: Vec::new() };
        self.beats_of(&mut acc, &first);
        all.extend(first.iter().cloned());
        acc.events = first;
        let mut floors: Vec<FoldFloor> = Vec::new();
        let mut run_over = false;
        let mut n = 0u32;
        loop {
            let (over, depth) = self.run.as_ref().map_or((true, 0), |r| (r.over.is_some(), r.depth));
            if over {
                run_over = true;
                break;
            }
            if depth > to || n >= FOLD_MAX_TICKS {
                break;
            }
            let loot_before = self.run.as_ref().map_or(0, |r| r.loot);
            self.tick();
            n += 1;
            let evs = std::mem::take(&mut self.events);
            all.extend(evs.iter().cloned());
            let now = self.run.as_ref().map_or(acc.depth, |r| r.depth);
            if now != acc.depth {
                // the tick that went down: its events up to the descend are the old floor's
                let cut = evs.iter().position(|e| matches!(e, Ev::Descend { .. })).unwrap_or(evs.len());
                let (old, new) = evs.split_at(cut);
                self.beats_of(&mut acc, old);
                acc.events.extend(old.iter().cloned());
                // (the floor's gold is the loot it ended with, the tick that went down counted below)
                floors.push(close(acc, clear_of, loot_before));
                acc = Acc { depth: now, snapshot: self.snapshot(), events: Vec::new(), loot0: loot_before, dip: None, beats: Vec::new() };
                if now <= to {
                    self.beats_of(&mut acc, new);
                }
                acc.events.extend(new.iter().cloned());
            } else {
                self.beats_of(&mut acc, &evs);
                acc.events.extend(evs);
                if n.is_multiple_of(MERGE_EVERY) {
                    let s = self.snapshot();
                    merge(&mut acc.snapshot, &s);
                }
            }
        }
        // the floor the run ended on inside the fold is a folded floor too
        if run_over && acc.depth <= to {
            let s = self.snapshot();
            merge(&mut acc.snapshot, &s);
            let loot = self.run.as_ref().map_or(0, |r| r.loot);
            floors.push(close(acc, clear_of, loot));
        }
        let step = self.step_result(all, run_over);
        let clear = floors.iter().map(|f| f.clear).product::<f64>();
        let gold = floors.iter().map(|f| f.gold).sum();
        let beats: Vec<FoldBeat> = floors.iter().flat_map(|f| f.beats.iter().cloned()).collect();
        let mut chips = chips(&beats);
        let to = floors.last().map_or(from.saturating_sub(1), |f| f.depth);
        // Cut 28 §4 (AV: "`send skips rest` sent a 9/40 heir" — the fold handed him off hurt): the
        // hero as the watch gets him, and `hp 9/40` on the line when at or under half his max.
        let (hp, max_hp) = (step.snapshot.hero.entity.hp, step.snapshot.hero.entity.max_hp);
        let low = !floors.is_empty() && !step.run_over && hp * 2 <= max_hp;
        let chip = format!("hp {hp}/{max_hp}");
        if low && !chips.contains(&chip) {
            chips.push(chip);
        }
        FoldLine { from, to, clear, gold, beats, chips, floors, step, hp, max_hp, low }
    }

    /// The beats of `events` on the folded floor `acc` (the run as it stands after them).
    fn beats_of(&self, acc: &mut Acc, events: &[Ev]) {
        let Some(run) = self.run.as_ref() else { return };
        for e in events {
            if let Some(b) = beat(e, acc.depth, |id| run.monsters.iter().find(|m| m.id == id).is_some_and(|m| m.ally)) {
                acc.beats.push(b);
            }
            if let Ev::Hurt { id, hp, t, .. } = e {
                let max = run.hero.max_hp.max(1);
                if *id == HERO_ID && (*hp as f64) <= DIP_SHARE * max as f64 && acc.dip.is_none_or(|d| *hp < d.0) {
                    acc.dip = Some((*hp, max, *t));
                }
            }
        }
    }
}

/// A floor done: its gold (`loot` when it ended less its first), its dip as a beat, its clear.
fn close(mut acc: Acc, clear_of: impl Fn(u32) -> f64, loot: i32) -> FoldFloor {
    if let Some((hp, max, t)) = acc.dip {
        acc.beats.push(FoldBeat { depth: acc.depth, t, kind: "dip".into(), text: format!("hp {hp}/{max}") });
        acc.beats.sort_by_key(|b| b.t);
    }
    FoldFloor { depth: acc.depth, clear: clear_of(acc.depth), gold: loot - acc.loot0, snapshot: acc.snapshot, events: acc.events, beats: acc.beats }
}

/// Fold a later snapshot of the same floor into the floor's first (what `runlog.floorSnapshot`
/// builds on the client): entities and items first seen since, and every tile seen.
pub fn merge(acc: &mut Snapshot, s: &Snapshot) {
    if s.depth != acc.depth {
        return;
    }
    for e in &s.entities {
        if e.id != acc.hero.entity.id && !acc.entities.iter().any(|x| x.id == e.id) {
            acc.entities.push(e.clone());
        }
    }
    for i in &s.items {
        if !acc.items.iter().any(|x| x.id == i.id) {
            acc.items.push(i.clone());
        }
    }
    if s.seen.len() == acc.seen.len() {
        for (a, b) in acc.seen.iter_mut().zip(&s.seen) {
            *a |= *b;
        }
    }
}

/// Cut 27 §1: whether an event changes what the run or the lineage holds — the fold line must
/// carry it (`beat`); a hero hurt is carried as the floor's dip instead (`DIP_SHARE`).
pub fn state_change(e: &Ev) -> bool {
    match e {
        Ev::Steal { .. } | Ev::Use { .. } | Ev::Fact { .. } | Ev::MaxHp { .. } | Ev::Bones { .. } | Ev::Level { .. } | Ev::Hatch { .. } | Ev::Ally { .. } => true,
        Ev::Pickup { id, .. } => *id == HERO_ID,
        Ev::Tame { ok, .. } => *ok,
        // Cut 28b: the sworn oath kept or broken on a folded floor rides the fold line (a miss is the exit's)
        Ev::Oath { kept, cause, .. } => *kept || !cause.is_empty(),
        Ev::Callout { text, .. } => text.starts_with("passage ") || text == "boss down",
        _ => false,
    }
}

/// The beat an event is on a folded floor (`is_ally` says whether an id is a companion's).
pub fn beat(e: &Ev, depth: u32, is_ally: impl Fn(u32) -> bool) -> Option<FoldBeat> {
    let b = |t: u32, kind: &str, text: String| Some(FoldBeat { depth, t, kind: kind.into(), text: crate::chronicle::clamp_words(&text, 3) });
    match e {
        Ev::Steal { t, id, item, amount } => {
            if is_ally(*id) {
                b(*t, "pet", format!("pet stole ${}", amount.unwrap_or(0)))
            } else {
                b(*t, "theft", format!("stolen {}", item.strip_prefix("gold ").unwrap_or(item)))
            }
        }
        Ev::Pickup { t, id, item } if *id == HERO_ID => match item.strip_prefix("gold ") {
            Some(coins) => b(*t, "gold", coins.to_string()),
            None => b(*t, "find", format!("found {item}")),
        },
        Ev::Use { t, item, .. } => b(*t, "use", format!("used {item}")),
        Ev::Fact { t, fact } => b(*t, "fact", fact_words(fact)),
        Ev::MaxHp { t, delta, .. } => b(*t, "max_hp", format!("max {delta:+}")),
        Ev::Bones { t, items, .. } => b(*t, "bones", format!("bones · {items}")),
        Ev::Level { t, level, .. } => b(*t, "level", format!("level {level}")),
        Ev::Hatch { t, kind } => b(*t, "hatch", format!("hatched {}", kind.replace('_', " "))),
        Ev::Tame { t, kind, ok: true, .. } => b(*t, "pet", format!("tamed {}", kind.replace('_', " "))),
        Ev::Ally { t, state, .. } => b(*t, "pet", format!("pet {state}")),
        Ev::Oath { t, kept: true, .. } => b(*t, "oath", "oath kept".into()),
        Ev::Oath { t, kept: false, cause, .. } if !cause.is_empty() => b(*t, "oath", "oath broken".into()),
        Ev::Callout { t, text, .. } if text.starts_with("passage ") => b(*t, "passage", text.clone()),
        Ev::Callout { t, text, .. } if text == "boss down" => b(*t, "boss", text.clone()),
        _ => None,
    }
}

/// A fact as ≤ 3 words (`foe:jackal:pack` → `jackal pack`).
fn fact_words(fact: &str) -> String {
    let parts: Vec<&str> = fact.split([':', '=']).collect();
    let body = if parts.len() > 1 { parts[1..].join(" ") } else { fact.to_string() };
    body.replace('_', " ")
}

/// The fold line's chips: one per kind, in the order that matters (a theft first), each ≤ 3
/// words — the beat itself when it is alone, else a count (`2 thefts`); the dip is the lowest.
/// Gold pickups are the line's `+$` and get no chip.
pub fn chips(beats: &[FoldBeat]) -> Vec<String> {
    const ORDER: [(&str, &str); 12] = [("oath", ""), ("theft", "thefts"), ("dip", ""), ("boss", "bosses down"), ("pet", "pet beats"), ("find", "finds"), ("use", "used"), ("max_hp", ""), ("fact", "learned"), ("bones", "bones"), ("level", ""), ("hatch", "hatched")];
    let mut out = Vec::new();
    for (kind, many) in ORDER {
        let of: Vec<&FoldBeat> = beats.iter().filter(|b| b.kind == kind).collect();
        let Some(first) = of.first() else { continue };
        let chip = match kind {
            "dip" => of.iter().min_by_key(|b| b.text.split(['/', ' ']).nth(1).and_then(|n| n.parse::<i32>().ok()).unwrap_or(0)).map(|b| b.text.clone()).unwrap_or_default(),
            "max_hp" => format!("max {:+}", of.iter().filter_map(|b| b.text.strip_prefix("max ").and_then(|n| n.parse::<i32>().ok())).sum::<i32>()),
            "level" => of.last().map(|b| b.text.clone()).unwrap_or_default(),
            _ if of.len() == 1 => first.text.clone(),
            _ => format!("{} {many}", of.len()),
        };
        out.push(crate::chronicle::clamp_words(&chip, 3));
    }
    out
}

/// Cut 28 §3 (AU: at 1× ~50 s of `pick up ×N` and 9–12 s gaps deeper in the run): a tick with no
/// decision and no threat — no hostile awake in the hero's view, no row of the set acting (a chore,
/// a trait's move, a `pick up` or `rest` row is no decision), and no beat: no blow, no death, no
/// telegraph, theft, descent, fact, callout, use, tame, level, exit, and no find of a kind the
/// lineage has not found before. A drain's bite with no foe in view (hunger, poison) is dead time
/// too (Cut 25 §3). The watch plays such ticks at the travel rate in every mode.
pub fn calm_tick(run: &crate::engine::Run, found: &std::collections::BTreeSet<String>, events: &[Ev]) -> bool {
    if run.over.is_some() {
        return false;
    }
    if run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.awake && run.floor.map.is_visible(m.pos)) {
        return false;
    }
    for e in events {
        let loud = match e {
            Ev::Rule { row, verb, .. } => *row >= 0 && !matches!(verb.v.as_str(), "pick_up" | "rest"),
            Ev::Hurt { id, cause, .. } => !(*id == HERO_ID && matches!(cause.as_str(), "hunger" | "poison" | "starving")),
            // (a gold pile, an unknown flavour, a kind found before: a pick-up chain is calm; a new kind or an enchanted piece is a find of note)
            Ev::Pickup { item, .. } => !(item.starts_with("gold") || item.ends_with('?') || (found.contains(item.split(' ').next().unwrap_or("")) && !item.contains('+'))),
            Ev::Move { .. } | Ev::Note { .. } | Ev::Overlay { .. } | Ev::Rest { .. } | Ev::Drain { .. } | Ev::MaxHp { .. } => false,
            _ => true,
        };
        if loud {
            return false;
        }
    }
    true
}

/// Cut 28 §3: the calm stretches of a step — runs of consecutive calm ticks as `[from, to]` run ticks
/// (`StepResult.calm`), from each played tick's (tick, calm) in order.
pub fn calm_spans(ticks: &[(u32, bool)]) -> Vec<[u32; 2]> {
    let mut out: Vec<[u32; 2]> = Vec::new();
    let mut open: Option<[u32; 2]> = None;
    for &(t, calm) in ticks {
        match (calm, open.as_mut()) {
            (true, Some(s)) => s[1] = t,
            (true, None) => open = Some([t, t]),
            (false, _) => {
                out.extend(open.take());
            }
        }
    }
    out.extend(open);
    out
}

