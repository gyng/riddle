//! The tick: energy scheduler (Addendum E), hero action (rules → verbs → chores), monsters,
//! overlays, clocks, vision. All durations are ticks; 10 ticks ≈ one turn at base speed.
use crate::ai;
use crate::chronicle::{callout, note};
use crate::defs::{monster_def, Cat};
use crate::descent::ENDING_DEPTH;
use crate::engine::{populate_floor, Ctx, ExitTier, Run, ACT_ENERGY, HERO_ID, TICKS_PER_TURN, VAULT_GRACE};
use crate::sifter::{self, Moment, Resolution};
use crate::facts::{learn, learn_tag, tag_known};
use crate::gen::generate;
use crate::geom::{Pos, DIRS8};
use crate::item::Item;
use crate::rules::{Cond, Verb};
use crate::tiles::{Overlay, OverlayKind, Tile};
use crate::wire::{Because, Ev, RowWhy, TraceTurn};

/// What the hero can see this action.
#[derive(Clone, Debug, Default)]
pub struct View {
    /// Visible hostile monster indices, nearest first. Cut 4: every hostile in view — this is
    /// what `foes>=` counts (the player's mental model); melee targeting uses `engage`.
    pub foes: Vec<usize>,
    /// The subset the hero can engage in melee: adjacent, or neither fleeing nor given up on
    /// (unreachable / not closing).
    pub engage: Vec<usize>,
    pub adj: i32,
    pub nearest: Option<usize>,
    pub lowest: Option<usize>,
}

/// Foes in view. `foes` is every visible hostile; `engage` the ones a melee row may chase.
/// Cut 5 §4: a sleeping den is scenery, not a foe, until it wakes (`on_see: nest`, greed or a
/// `pick_up` row are the ways in).
pub fn view(run: &Run) -> View {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let mut foes: Vec<usize> = (0..run.monsters.len())
        .filter(|&i| {
            let m = &run.monsters[i];
            // Cut 7 §3: a captive chained across the stairs is a foe once the hero stands
            // beside it (the coward's way through the gate).
            let chained = m.neutral && m.situation.as_deref() == Some("captive") && m.pos.adjacent(hp);
            // A thief running with its loot is not a threat to hide from (`foes ≥ 3 → to
            // corridor` held a corridor against three fleeing monkeys, cohort 10); it stays
            // a `foe: thief` target.
            let running = m.fleeing && m.stolen.is_some() && !m.pos.adjacent(hp);
            m.hp > 0 && (m.hostile() || chained) && !m.dormant && !running && map.is_visible(m.pos)
        })
        .collect();
    foes.sort_by_key(|&i| (run.monsters[i].pos.cheb(hp), run.monsters[i].id));
    let engage: Vec<usize> = foes
        .iter()
        .copied()
        .filter(|&i| {
            let m = &run.monsters[i];
            m.pos.adjacent(hp) || (!m.fleeing && m.fear == 0 && !run.is_ignored(m.id))
        })
        .collect();
    let adj = foes.iter().filter(|&&i| run.monsters[i].pos.adjacent(hp)).count() as i32;
    let nearest = engage.first().copied();
    let lowest = engage.iter().copied().min_by_key(|&i| (run.monsters[i].hp, run.monsters[i].id));
    View { foes, engage, adj, nearest, lowest }
}

/// QA on 92eb880 (qaN): the trace's `foes` column is the player's count — every hostile the
/// hero can see, running thieves and foes the guard gave up on included (those exclusions are
/// for decisions: `threats()`, `View::engage`, and `foes>=` keeps its own count, `rule_foes`).
/// A sleeping den is still scenery (Cut 5 §4). `sight` adds foes in the hero's line of sight
/// inside his radius that the last vision pass has not marked yet (the blow that ends a run
/// lands before the tick's vision pass).
pub fn seen_foes(run: &Run, sight: Option<i32>) -> i32 {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    run.monsters
        .iter()
        .filter(|m| {
            let chained = m.neutral && m.situation.as_deref() == Some("captive") && m.pos.adjacent(hp);
            m.hp > 0
                && (m.hostile() || chained)
                && !m.dormant
                && (map.is_visible(m.pos) || sight.is_some_and(|r| m.pos.cheb(hp) <= r && map.los(hp, m.pos)))
        })
        .count() as i32
}

/// The last trace row counts every foe seen until the next action (the clip the row plays:
/// a den's pounce inside the stairs step, an archer stepping into view to shoot).
fn trace_seen(run: &mut Run, sight: Option<i32>) {
    if run.trace.is_empty() {
        return;
    }
    let n = seen_foes(run, sight);
    let rose = run.trace.last().is_some_and(|t| n > t.foes);
    // A thief running now is counted apart from one that stepped into view (QA on 1a2a4a9).
    let running = if rose { (n - view(run).foes.len() as i32).max(0) } else { 0 };
    // Only a rise writes the row (`foes` is already the max otherwise): the trace's rows are
    // shared with the history ring's copies (`shared.rs`), and a write copies the row.
    if rose {
        if let Some(t) = run.trace.last_mut() {
            t.foes = t.foes.max(n);
            foe_reasons(t, running);
        }
    }
}

/// QA on 1a2a4a9 (qaO: `hp 3 foes 2 · R2 foes not ≥1` under `foes ≥ 1 → attack`): the `foes`
/// column is every hostile seen until the next action, the rules count what was in view and
/// not running with loot when they were read (`TraceTurn.rule_foes`). A `foes not ≥N` beside
/// a column of N or more says why the foes did not count: `foes fleeing` (thieves running
/// with loot) or `foes appeared after` (they came into view after the rows were read).
fn foe_reasons(t: &mut TraceTurn, running: i32) {
    let (foes, counted) = (t.foes, t.rule_foes);
    let Some(rows) = t.rows.as_mut() else { return };
    for w in rows.iter_mut() {
        let Some(n) = w.why.strip_prefix("foes not ≥").and_then(|x| x.parse::<i32>().ok()) else { continue };
        if foes >= n && n > counted {
            w.why = if counted + running >= n { "foes fleeing" } else { "foes appeared after" }.into();
            w.because = None;
        }
    }
}

/// The cached hero distance field, recomputed when the hero has moved (the whole floor).
pub fn hero_dist(run: &mut Run) -> &[i32] {
    hero_flood(run, None, None);
    &run.hero_dist
}

/// The hero distance field settled for every tile within `steps` of him: those read their
/// distance, any farther tile its distance or −1 (not yet flooded).
pub fn hero_dist_within(run: &mut Run, steps: i32) -> &[i32] {
    hero_flood(run, None, Some(steps));
    &run.hero_dist
}

/// The hero distance field settled as far as `to` (the whole floor when `to` is unreachable):
/// every tile nearer the hero than `to` has its distance, `to` too — all a monster's step down
/// the field reads (`Map::step_down` takes a neighbour only if it is nearer than the monster's
/// own tile). Tiles past it may still read −1. The floods of the monsters' approach were a
/// fifth of a sim's time, most of each past the monster.
pub fn hero_dist_to(run: &mut Run, to: Pos) -> &[i32] {
    let at = run.floor.map.in_bounds(to).then(|| run.floor.map.idx(to));
    hero_flood(run, Some(at.unwrap_or(usize::MAX)), None);
    &run.hero_dist
}

fn hero_flood(run: &mut Run, until: Option<usize>, within: Option<i32>) {
    if run.hero_dist_pos != Some(run.hero.pos) || run.hero_dist.len() != run.floor.map.tiles.len() {
        run.floor.map.flood_start(run.hero.pos, &mut run.hero_dist, &mut run.hero_flood, &mut run.hero_flood_head);
        run.hero_dist_pos = Some(run.hero.pos);
    }
    // (an out-of-bounds `to` has no tile to wait for: the flood runs out, as `None`)
    let until = until.filter(|&u| u < run.hero_dist.len());
    run.floor.map.flood_resume(&mut run.hero_dist, &mut run.hero_flood, &mut run.hero_flood_head, until, within);
}

pub fn tick(run: &mut Run, cx: &mut Ctx) {
    if run.over.is_some() {
        return;
    }
    run.turn += 1;
    run.floor_turn += 1;
    crate::firearm::tick(run, cx);
    // Cut 5 §4: an opened vault waits `VAULT_GRACE` ticks for `choose` (the client's sheet),
    // then the preference picks — watched or not, so a verdict replay stays faithful.
    if run.vault_choice.as_ref().is_some_and(|(t0, _)| run.turn >= t0 + VAULT_GRACE) {
        vault_take(run, cx, None);
    }
    run.hero.energy += run.hero.speed();
    for m in run.monsters.iter_mut() {
        if m.hp > 0 {
            m.energy += m.effective_speed();
        }
    }
    // Cut 7 §3: the hunger bites on an unlit D12.
    crate::situations::hunger_tick(run, cx);
    let mut acted = false;
    // Hero first.
    while run.hero.energy >= ACT_ENERGY && run.over.is_none() {
        run.hero.energy -= ACT_ENERGY;
        hero_action(run, cx);
        acted = true;
        let vision = run.vision(cx.unlocks);
        run.floor.map.update_vision(run.hero.pos, vision);
        crate::facts::on_vision(run, cx);
        crate::facts::fork_seen(run, cx);
        for m in run.monsters.iter_mut() {
            m.acts_since_hero = 0;
        }
    }
    if run.over.is_some() {
        return;
    }
    run.monsters.retain(|m| m.hp > 0);
    // Then monsters, by id (spawn order).
    let n = run.monsters.len();
    for mi in 0..n {
        if run.over.is_some() {
            return;
        }
        while run.monsters[mi].hp > 0 && run.monsters[mi].energy >= ACT_ENERGY && run.over.is_none() {
            run.monsters[mi].energy -= ACT_ENERGY;
            ai::monster_act(run, cx, mi);
            acted = true;
            run.monsters[mi].acts_since_hero += 1;
            if run.monsters[mi].acts_since_hero >= 2
                && run.monsters[mi].hp > 0
                && run.monsters[mi].has_tag("fast")
                && run.floor.map.is_visible(run.monsters[mi].pos)
            {
                let k = run.monsters[mi].kind.clone();
                learn_tag(run, cx, &k, "fast");
            }
        }
    }
    run.monsters.retain(|m| m.hp > 0);
    if run.over.is_some() {
        return;
    }
    if run.turn.is_multiple_of(TICKS_PER_TURN) {
        tick_overlays(run, cx);
        if run.over.is_some() {
            return;
        }
        tick_poison(run, cx);
        if run.over.is_some() {
            return;
        }
        tick_regen_and_auras(run, cx);
        if run.over.is_some() {
            return;
        }
    }
    // The floor clock runs from the descend, not from a multiple of ten ticks.
    tick_alert(run, cx);
    if run.over.is_some() {
        return;
    }
    tick_statuses(run, cx);
    run.monsters.retain(|m| m.hp > 0);
    let _ = acted;
    crate::facts::on_vision(run, cx);
    trace_seen(run, None);
}

/// Cut 27 §3 (AO, AS: bank beat return on every number — both walked to the up-stairs, the
/// same walk at 60 % against 100 %; on the cohort sets most deaths of a banking set were on
/// that walk): the ticks a return walks before he is out where he stands (his own actions:
/// `RETURN_TICKS / TICKS_PER_TURN`). A bank must reach the stairs.
pub const RETURN_TICKS: u32 = 60;

/// The committed walk home is a return's and its `RETURN_TICKS` are spent.
pub fn return_due(run: &Run) -> bool {
    run.over.is_none() && run.homeward.is_some() && !run.homeward_bank && run.home_at.is_some_and(|(t, ..)| run.turn >= t + RETURN_TICKS)
}

/// Ticks left of a committed return's walk (`None`: no return walking).
pub fn return_left(run: &Run) -> Option<u32> {
    if run.homeward.is_none() || run.homeward_bank {
        return None;
    }
    run.home_at.map(|(t, ..)| (t + RETURN_TICKS).saturating_sub(run.turn))
}

/// Cut 27 §3: the carry's scent — the tiles a bank walk's gold is heard across: `BANK_SCENT_BASE`
/// plus one per `BANK_SCENT_PER` coins carried, at most `BANK_SCENT_MAX`.
pub const BANK_SCENT_BASE: i32 = 4;
pub const BANK_SCENT_PER: i32 = 25;
pub const BANK_SCENT_MAX: i32 = 20;

/// Cut 27 §3: a bank exposes the carry — when the walk to the up-stairs commits, the floor's
/// hostiles within the carry's scent (`BANK_SCENT_*`) wake and come for where he stands (a boss,
/// a dormant den or cage stays put). A return (60 %, out where he stands) leaves no trail.
pub fn gold_scent(run: &mut Run, cx: &mut Ctx) {
    let r = (BANK_SCENT_BASE + run.loot.max(0) / BANK_SCENT_PER).min(BANK_SCENT_MAX);
    let at = run.hero.pos;
    let mut woke = 0;
    for m in run.monsters.iter_mut() {
        if m.hp <= 0 || !m.hostile() || m.is_boss() || m.dormant || m.nest || m.situation.is_some() || m.pos.cheb(at) > r {
            continue;
        }
        woke += (!m.awake) as u32;
        m.awake = true;
        m.last_seen = Some(at);
    }
    if woke > 0 {
        callout(run, cx, "gold draws them");
    }
}

fn hero_action(run: &mut Run, cx: &mut Ctx) {
    run.actions += 1;
    run.note_foes();
    // Cut 3: known mirrors (visible, `reflect_melee` fact) are terrain to the chores.
    let map = &run.floor.map;
    run.mirrors = run
        .monsters
        .iter()
        .filter(|m| m.hp > 0 && m.hostile() && m.reflects_melee() && map.is_visible(m.pos) && tag_known(cx.facts, &m.kind, "reflect_melee"))
        .map(|m| m.pos)
        .collect();
    // Cut 5 §4: sleeping dens the hero has seen are terrain too (kept two tiles clear).
    run.dens = run.monsters.iter().filter(|m| m.hp > 0 && m.nest && m.dormant && map.is_seen(m.pos)).map(|m| m.pos).collect();
    // Cut 7 §3: the thief's den's sleepers are terrain the chores walk round.
    run.sleepers = run.monsters.iter().filter(|m| m.hp > 0 && m.dormant && m.situation.is_some()).map(|m| m.pos).collect();
    oscillation_guard(run, cx);
    // Cut 5 §4: a den wakes when the hero comes within two tiles of it.
    wake_nest(run, cx);
    // Cut 7 §3: the thief's den pounces on a hero at the stairs.
    crate::situations::before_action(run, cx);
    let v = view(run);
    // Cut 30 §1: which of the heir's gifts are live at this action (never a verb).
    crate::traits::on_action(run, cx, &v.foes);
    // Cut 5 §3: the fight clock (no hero lines in a fight's first ten ticks).
    if v.foes.is_empty() {
        run.fight_t = None;
    } else if run.fight_t.is_none() {
        run.fight_t = Some(run.turn);
    }
    // Cut 25 §3: a foe in view ends a drain stretch (the next bite with none is a new one).
    if !v.foes.is_empty() {
        run.drain_on = None;
    }
    let seen_before = seen_foes(run, None);
    let hp_before = run.hero.hp;
    // QA on 524827b (qaAB): the blows before this action (the trace row's own); those its tick deals after roll on
    let blows_before = run.blows.len();
    // (the events before this action: a hero `Ev::Pickup` after them is something taken — `pickup_dry`)
    let ev_before = cx.events.len();
    let inv_before = run.hero.inv.len() + run.hero.weapon.is_some() as usize + run.hero.armour.is_some() as usize;
    run.last_hit_verb = None;
    // Cut 3 `reflect_read`: the bow goes back in the pack once no mirror is in view (free).
    if run.bow_swap.is_some() && !v.foes.iter().any(|&i| run.monsters[i].reflects_melee()) {
        let melee = run.bow_swap.take().unwrap();
        if let Some(bow) = run.hero.weapon.replace(melee) {
            if run.hero.inv_full() {
                let here = run.hero.pos;
                run.note_gone(bow.id, &bow.kind, "left", 1);
                drop_near(run, here, bow);
            } else {
                run.hero.inv.push(bow);
            }
        }
    }
    // Cut 27 §3: a return walks from anywhere — `RETURN_TICKS` after it committed he is out
    // where he stands (the stairs, if he reaches them first, as before); a bank still walks to
    // the up-stairs.
    if return_due(run) {
        if run.exit_row.is_none() {
            run.exit_row = run.homeward;
        }
        end_run(run, cx, ExitTier::Return);
        return;
    }
    // Cut 23 §3: which rows' conds held at the decision, for the why-not tally.
    let held = if cx.sim { Vec::new() } else { rows_held(run, cx, &v) };
    let (row, verb) = choose_and_act(run, cx, &v);
    if !cx.sim {
        tally_rows(run, cx, &held, row);
    }
    // Cut 7 §4: the last 30 ticks before a foreseeable exit are announced.
    foresee_ending(run, cx, &verb, &v);
    // Cut 5 §1: the episode records the action (the row at the low point when one is awaited)
    // and seals itself once the hero has recovered from a low.
    sifter::on_action(run, row, &verb);
    if sifter::recovered(run) {
        sifter::seal(run);
        sifter::voice(run, cx, Moment::Resolved);
    }
    // Cut 5 §4: standing on the vault opens it; situations in view are facts.
    if run.over.is_none() {
        if run.floor.map.get(run.hero.pos) == Tile::Vault {
            vault_open(run, cx);
        }
        situations_seen(run, cx);
    }
    // Cut 3: the Mirror King remembers the hero's last three verbs while he watches — the verb
    // of the hit landed (attack, shoot, cleave…), else the action itself. Cut 5: a paralysed
    // turn is no action (a sentinel's gaze was resetting his mirror for the hero).
    // Cut 13: a trait's deviation (row −1) is not the hero's rhythm — the King mirrors what the
    // rules do; a coward's step back between two blows was passing D33 without the cadence card.
    let king_watching = run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.kind == "mirror_king" && run.floor.map.is_visible(m.pos));
    if king_watching && verb.v != "paralysed" && row != -1 {
        let used = run.last_hit_verb.take().unwrap_or_else(|| verb.v.clone());
        run.verb_ring.push(used);
        while run.verb_ring.len() > 3 {
            run.verb_ring.remove(0);
        }
    }
    // Same-row loop guard: one row firing 40 actions straight with no blood drawn either way
    // is a stalemate (a bloat that follows a retreating hero forever); rest it for 30 actions.
    if row == -1 {
        // trait deviations are transparent to the streak
    } else if row >= 0 && row == run.row_streak.0 {
        run.row_streak.1 += 1;
        if run.row_streak.1 >= 40 && run.actions.saturating_sub(run.last_damage_action) >= 40 {
            run.row_suppressed = (row, run.actions + 30);
            run.row_streak = (-9, 0);
            emit_rule(run, cx, -2, &Verb::new("stuck"), "stuck → chores");
        }
    } else {
        run.row_streak = (row, 1);
    }
    outpaced_guard(run, cx, row, &verb);
    nohp_guard(run, cx, row, &verb);
    // pick_up sanity: three picks in a row must have put something in the pack.
    if verb.v == "pick_up" {
        if run.pickup_streak == 0 {
            run.pickup_inv = inv_before;
        }
        run.pickup_streak += 1;
        let inv_now = run.hero.inv.len() + run.hero.weapon.is_some() as usize + run.hero.armour.is_some() as usize;
        if inv_now > run.pickup_inv || run.hero.inv.iter().any(|i| i.kind == "leash" && i.amount > 1) {
            run.pickup_streak = 0;
        } else if run.pickup_streak >= 3 {
            run.items_until = run.actions + 20;
            run.pickup_streak = 0;
        }
    } else {
        run.pickup_streak = 0;
    }
    // Cut 25 §3: a `pick up` chore that stands on what it walked to and still leaves it lying there
    // (the pack would not take it after all) gives that item up for the floor — whatever the
    // steps between (`pick up` ↔ `explore` paced a full pack over one scroll 616 times on D6).
    let inv_after = run.hero.inv.len() + run.hero.weapon.is_some() as usize + run.hero.armour.is_some() as usize;
    if verb.v == "pick_up" && row == -2 && inv_after <= inv_before {
        let here = run.hero.pos;
        let stuck: Vec<u32> = run.items.iter().filter(|fi| fi.pos == here && !run.skip_items.contains(&fi.item.id)).map(|fi| fi.item.id).collect();
        run.skip_items.extend(stuck);
    }
    // QA on 524827b (seed 30's D5: `pick up` chores between a row's attacks — the streak guard
    // above resets on every other action — 60 on the floor with nothing taken): the floor's dry
    // `pick up` chores are counted whatever came between; at `PICKUP_DRY_MAX` the floor's items
    // are given up until the stairs.
    // Something taken is any hero pickup the action made (the qa leg's reset, `Ev::Pickup`): gold
    // and a stacked potion leave the pack's slots as they were — counted as dry, a sweep's 40th
    // coin gave the floor's heals up (FULL's deaths, `dice` 19.8 → 27.4%).
    let took = inv_after > inv_before || cx.events.get(ev_before..).is_some_and(|es| es.iter().any(|e| matches!(e, Ev::Pickup { id: HERO_ID, .. })));
    // Cut 30 §1: `sure` and `light hands` act after the verb (energy back); never choose it.
    crate::traits::after_action(run, cx, &verb, took, &v.foes);
    if took {
        run.pickup_dry = 0;
    } else if verb.v == "pick_up" && row == -2 {
        run.pickup_dry += 1;
        if run.pickup_dry >= PICKUP_DRY_MAX {
            run.items_until = u32::MAX;
        }
    }
    run.recent_pos.push(run.hero.pos);
    if run.recent_pos.len() > 12 {
        run.recent_pos.remove(0);
    }
    // Trace records the state at the start of the action. A telegraph reads as its callout
    // does (`warlord rallies`, the title's last word), never the kind (`goblin_warlord`).
    let telegraphs: Vec<String> = v
        .foes
        .iter()
        .filter_map(|&i| run.monsters.get(i).and_then(|m| m.telegraph.as_ref().map(|t| format!("{} {}", m.def().title.split_whitespace().last().unwrap_or("foe").to_lowercase(), t))))
        .collect();
    let blocked = run.blocked_now.take();
    // Cut 6 §3: every row above the one that acted, with its reason (none when R1 acted).
    let whys = std::mem::take(&mut run.rows_why);
    let rows = if whys.is_empty() { None } else { Some(whys) };
    let before: Vec<crate::wire::TraceBlow> = run.blows.drain(..blows_before.min(run.blows.len())).collect();
    let mut turn = TraceTurn { max_hp: run.hero.max_hp, t: run.turn, row, verb, hp: hp_before, foes: seen_before.max(v.foes.len() as i32), rule_foes: v.foes.len() as i32, telegraphs, blocked, rows, blows: before, gift: run.gift.mark.take() };
    foe_reasons(&mut turn, (seen_before - v.foes.len() as i32).max(0));
    run.trace.push(turn.into());
    if run.trace.len() > 16 {
        run.trace.remove(0);
    }
    run.hurt_last = run.hurt_since_action;
    run.hurt_since_action = false;
    run.kill_last = run.kill_since_action;
    run.kill_since_action = false;
    run.new_seen = false;
}

fn choose_and_act(run: &mut Run, cx: &mut Ctx, v: &View) -> (i32, Verb) {
    run.rows_why.clear();
    if run.hero.paralysed > 0 {
        let verb = Verb::new("paralysed");
        emit_rule(run, cx, -2, &verb, "paralysed");
        all_rows_why(run, cx, "paralysed", None);
        return (-2, verb);
    }
    if run.hero.confused > 0 && run.rng.chance(50) {
        ai::random_step(run, cx);
        let verb = Verb::new("stumble");
        emit_rule(run, cx, -2, &verb, "confused → stumble");
        all_rows_why(run, cx, "confused", None);
        return (-2, verb);
    }
    // Cut 5 §5: bail — a queued `return`, the rules untouched.
    if run.bail {
        run.bail = false;
        let verb = Verb::new("return");
        end_run(run, cx, ExitTier::Return);
        emit_rule(run, cx, -2, &verb, "bail → return");
        all_rows_why(run, cx, "bail", None);
        return (-2, verb);
    }
    let hp_pct = run.hero.hp_pct();
    let hp_now = run.hero.hp;
    let was_low = run.low20_t.is_some() || run.low10_t.is_some();
    // Cut 30 §2: a temperament acts only through its rows (`packages::temperament_rows`); the old
    // per-floor overrides (the coward's retreat, the greedy grab, the brave hold, the curious use)
    // are gone — no action is chosen by a trait.
    // Sanity: nobody stands in gas or fire with no foe adjacent.
    let hz = {
        let hp = run.hero.pos;
        run.overlays.iter().find(|o| o.x == hp.x && o.y == hp.y).map(|o| match o.k {
            crate::tiles::OverlayKind::Gas => "gas",
            crate::tiles::OverlayKind::Fire => "fire",
        })
    };
    if v.adj == 0 && ai::escape_hazard(run, cx, v) {
        let verb = Verb::new("explore");
        emit_rule(run, cx, -2, &verb, "hazard → step out");
        once_rows_why(run, cx, &format!("hazard first · {}", hz.unwrap_or("gas")));
        return (-2, verb);
    }
    // Cut 3 `recall_sense`: below 15% with a recall scroll in the pack, read it (a free row).
    if hp_pct < 15 && cx.unlocks.contains("recall_sense") && run.hero.inv.iter().any(|i| i.kind == "recall") {
        let verb = Verb::arg("read", "recall");
        if ai::try_verb(run, cx, &verb, v) {
            emit_rule(run, cx, -2, &verb, "recall sense");
            all_rows_why(run, cx, "recall sense", None);
            return (-2, verb);
        }
    }
    // Cut 12 §1: the rows in play — every card row and the first `max_rows` own rows, each
    // with its index in the set (`R3` is the set's third row wherever the cards sit).
    // (borrowed from the set, which nothing here changes; the lent row is the run's, copied out)
    let set: &crate::rules::RuleSet = cx.rules;
    let lent = run.lent_row.clone();
    let mut rows: Vec<(usize, &crate::rules::Row)> = set.active(cx.max_rows()).collect();
    // Cut 5 §4: the row a shrine lent for this run (last, lowest priority; index past the set).
    if let Some(r) = &lent {
        rows.push((set.rows.len(), r));
    }
    // Cut 23 §2: the committed walk home gave way to a row under it that answers (`answers_on_walk`).
    let mut walk_only = false;
    let stuck = run.stuck_until > run.actions;
    let suppressed = if run.row_suppressed.1 > run.actions { run.row_suppressed.0 } else { -9 };
    run.last_target = None;
    run.blocked_now = None;
    for (k, (i, row)) in rows.iter().map(|(i, r)| (*i, *r)).enumerate() {
        // QA on a946e04 (qaS: a row the editor marks dead — `hp < 50% → attack nearest` under
        // `foes ≥ 1 → attack nearest`, `↑ R3` — moved the forecast): a row an earlier row
        // shadows (`rules::shadows`, the editor's own test) never acts. It fired whenever the
        // earlier row's identical verb failed with a side effect (a chase given up marks the
        // foe ignored, so the same `attack nearest` one row down picked another) or the row
        // guard held the earlier row alone.
        if let Some((j, _)) = rows[..k].iter().find(|(_, a)| (a.verb == row.verb || a.verb.v == "hold") && crate::rules::shadows(a, row, &|c: &Cond| row_usable(cx, c))) {
            // (a reason is the trace's: a sim keeps none — `row_why` — so none is written)
            if !cx.sim {
                row_why(run, cx, i, &format!("same as R{}", j + 1), None, None);
            }
            continue;
        }
        // The guard ignores the foes at range it paced in front of; one at the hero's elbow
        // is always worth a row (QA on e0f87e7: ten `pick up` rows with a jackal adjacent and
        // `foes ≥ 1 → attack nearest` reading `stuck`).
        if stuck && v.adj == 0 && targets_foes(&row.verb) {
            row_why(run, cx, i, "stuck", None, None);
            continue;
        }
        if i as i32 == suppressed || (run.rows_rested.1 > run.actions && run.rows_rested.0.contains(&(i as i32))) {
            row_why(run, cx, i, "row guard", None, None);
            continue;
        }
        // Cut 6 §3: the first condition that does not hold names the reason (a hunting row
        // still walks, below, with its conditions lapsed).
        let failing = row.conds.iter().find(|c| !cond_holds(run, cx, v, c));
        let holds = failing.is_none();
        // Cut 23 §2 (AJ: every death `return too late` at 1–2 HP; the cohort sets with a way
        // home died on the walk in 70–100 % of their deaths, jackals biting a back that never
        // turned, a heal row under the return never read): once a `return` / `bank` row has
        // committed the walk (`Run.homeward`), the walk is the chore — the committed row no
        // longer takes every action from the rows under it. A row below that answers on the
        // walk (a drink, a read, a throw, or a strike at a foe at the elbow) acts; when none
        // does, the walk goes on.
        if holds && run.homeward.is_some() && matches!(row.verb.v.as_str(), "return" | "bank") && rows[k + 1..].iter().any(|(_, r)| answers_on_walk(run, cx, v, r)) {
            row_why(run, cx, i, "going home", None, None);
            walk_only = true;
            continue;
        }
        // (under a committed walk only the rows that answer on it act)
        if walk_only && holds && !answers_on_walk(run, cx, v, row) {
            // QA on 524827b (qaAB: `R5 drink heal · going home` at 4/40 hp, dying): a drink or a read the walk did not stop
            // for had nothing to use — that is its reason, not the walk
            let item_row = matches!(row.verb.v.as_str(), "drink" | "read");
            let why = if !item_row { "going home" } else if row.verb.a.as_deref().is_none_or(|a| a == "unknown") { "no unknown" } else { "no item" };
            row_why(run, cx, i, why, None, None);
            continue;
        }
        if let Some(c) = failing.filter(|_| !cx.sim) {
            row_why(run, cx, i, &cond_reason(run, cx, c), Some(row), Some(c));
        }
        let scope = row.conds.iter().find(|c| c.k == "party").and_then(|c| c.t.clone());
        run.raiding = row.conds.iter().any(|c| (c.k == "on_see" && c.t.as_deref() == Some("den")) || (c.k == "foe_tag" && c.t.as_deref() == Some("thief")));
        run.acting_row = i as i32;
        let acted = holds && ai::try_verb_scoped(run, cx, &row.verb, v, scope.as_deref());
        run.acting_row = -1;
        if acted {
            let text = rule_text(run, row, hp_pct);
            emit_rule_owned(run, cx, i as i32, &row.verb, text);
            if i < run.row_fired.len() {
                run.row_fired[i] += 1;
            }
            if run.over.is_some() && run.exit_row.is_none() {
                run.exit_row = Some(i as i32);
            }
            if run.over.is_none() && run.homeward.is_none() && matches!(row.verb.v.as_str(), "return" | "bank") {
                run.homeward = Some(i as i32);
                run.home_at = Some((run.turn, hp_pct, run.hero.hp));
                run.homeward_bank = row.verb.v == "bank";
                // Cut 28b: a `return` committed breaks a `no return` oath at the row that did it
                if !run.homeward_bank {
                    run.home_return = true;
                    crate::oath::beat(run, cx, i as i32);
                }
                if run.homeward_bank {
                    gold_scent(run, cx);
                }
            }
            // Cut 4: the first row to act after the hero fell to ≤ 20 % is the one the
            // chronicle credits if the floor is survived.
            // Cut 24 §5 (AL: `Down to 1 HP. R2 drink heal saved him.` — R2 drank at 4 HP, then
            // `R4 read teleport` at 1 HP got him out): the row that fired at the floor's lowest
            // HP (the latest at a tie).
            // (low before the row acted: a blow its own action drew — a bloat it popped — is no rescue)
            if was_low && (run.saved_by.is_none() || hp_now <= run.saved_low) {
                run.saved_by = Some(i as i32);
                run.saved_low = hp_now;
            }
            if matches!(row.verb.v.as_str(), "recall" | "send") {
                row_why(run, cx, i, "fired, free", None, None);
                continue; // party orders are free actions
            }
            // Cut 4: the foe this row acted on is hunted when it steps out of view.
            run.hunt = if targets_foes(&row.verb) { run.last_target.map(|id| (id, i as i32)) } else { None };
            return (i as i32, row.verb.clone());
        }
        // Cut 4: the row that last acted on a foe now out of view walks to where it was seen.
        if targets_foes(&row.verb) && run.hunt.is_some_and(|(_, r)| r == i as i32) {
            if let Some(kind) = ai::hunt_step(run, cx) {
                let text = format!("hunt {}", crate::engine::kind_title(&kind).to_lowercase());
                emit_rule(run, cx, i as i32, &row.verb, &text);
                if i < run.row_fired.len() {
                    run.row_fired[i] += 1;
                }
                if !holds && !cx.sim {
                    run.rows_why.pop();
                }
                return (i as i32, row.verb.clone());
            }
        }
        if !holds {
            continue;
        }
        // Cut 4: a row whose conditions hold but whose verb cannot execute is shown as such
        // (`R1 retreat ✗ no path`), the first such row per action; cards fall through by design.
        if row.verb.v == "tactic" {
            // QA on 912e135 (qaW: `card passed · card not triggered` for `kite archers` with an
            // archer drawing in the trace): a card keyed on a foe says whether that foe was in
            // view — `card idle` (none) or `card blocked` (there, and its move had no way).
            let why = match crate::meta::card_situation(row.verb.a.as_deref().unwrap_or("")) {
                Some(tag) if v.foes.iter().any(|&m| run.monsters[m].has_tag(&tag)) => "card blocked",
                Some(_) => "card idle",
                None => "card passed",
            };
            row_why(run, cx, i, why, None, None);
        } else {
            let reason = ai::block_reason(run, cx, &row.verb, v);
            row_why(run, cx, i, reason, Some(row), None);
            if run.blocked_now.is_none() {
                run.blocked_now = Some(format!("R{} {} ✗ {reason}", i + 1, row.verb.short()));
                // The verb's own short word (`corridor ✗ no path`), never the id's first half
                // (`back ✗ no path` named no row — QA on 50bb162).
                let short = format!("{} ✗ {reason}", row.verb.short().split(' ').next().unwrap_or(&row.verb.v));
                if run.blocked_last.as_deref() != Some(&short) {
                    // Cut 23 §3: the reason on tap (`read ✗ no use` → `nothing to learn`).
                    crate::chronicle::callout_why(run, cx, &short, why_gloss(&row.verb.v, reason));
                }
                run.blocked_last = Some(short);
            }
        }
    }
    if run.blocked_now.is_none() {
        run.blocked_last = None;
    }
    let verb = ai::chore(run, cx, v);
    let text = if verb.v == "cornered" { "cornered, no orders".to_string() } else { format!("chore → {}", verb.short()) };
    emit_rule(run, cx, -2, &verb, &text);
    (-2, verb)
}

/// Cut 6 §3: the reason table — every `TraceTurn.rows[].why` is one of these shapes, ≤ 3
/// words.
///
/// Conditions: `hp not <N%`, `hp not >N%`, `foes not ≥N` (`foes fleeing` / `foes appeared
/// after` when the trace's `foes` column reaches N: `foe_reasons`), `adj not ≥N`, `not in view` (a foe
/// tag), `no weak foe`, `none held`, `no unknown`, `seen not ≥N%`, `depth not ≥N`, `alert not
/// ≥N`, `not corridor`, `no path`, `no ally`, `loot not ≥N`, `turns not >N`, `not hurt`, `no
/// kill`, `nothing new`, `no <tile> seen`, `no <kind>`, `party hp ok`, `locked cond`.
///
/// Verbs whose conditions held: `ai::block_reason` (`no path`, `no target`, `no line`, `no
/// bow`, `cooldown`, `no item`, `unknown item`, `no unknown`, `no use`, `no leash`, `none weak`, `not safe`,
/// `no stairs`, `no way`, `going home`, `prayed`, `no shrine`), `card passed`, `brave held`, `fired, free`.
///
/// Guards and pre-emptions: `stuck`, `row guard`, `trait first`, `hazard first`, `recall
/// sense`, `paralysed`, `confused`, `bail`.
pub const ROW_REASONS: &[&str] = &[
    "reloading", "empty gun", "gun full",
    "hp not <", "hp not >", "foes not ≥", "foes fleeing", "foes appeared after", "adj not ≥", "not in view", "no weak foe", "none held", "no unknown", "seen not ≥",
    "depth not ≥", "alert not ≥", "not corridor", "no path", "no ally", "loot not ≥", "turns not >", "not hurt", "no kill",
    "nothing new", "no ", "party hp ok", "locked cond", "no target", "no line", "no bow", "cooldown", "no item", "no use",
    "no leash", "none weak", "not safe", "no stairs", "no way", "going home", "prayed", "no shrine", "unknown item", "card passed", "card idle", "card blocked", "given up", "brave held", "fired, free",
    "stuck", "row guard", "same as R", "trait first", "hazard first", "recall sense", "paralysed", "confused", "bail",
];

/// Cut 23 §2: a row that answers on the walk home — its conds hold and it drinks, reads or
/// throws, or strikes (not steps away from) a foe at the hero's elbow that outpaces him (a
/// walk leaves a slower or an even-paced foe behind; a faster one bites the back that turns).
fn answers_on_walk(run: &Run, cx: &Ctx, v: &View, r: &crate::rules::Row) -> bool {
    let h = &run.hero;
    let held = |kind: &str, cat: Option<Cat>| match kind {
        "" | "unknown" => h.inv.iter().any(|i| cat.is_none_or(|c| i.cat() == c) && i.is_consumable() && !i.is_known(cx.facts, cx.flavours)),
        k => h.inv.iter().any(|i| i.kind == k && i.is_known(cx.facts, cx.flavours)),
    };
    let arg = r.verb.a.as_deref().unwrap_or("");
    let answers = match r.verb.v.as_str() {
        "return" | "bank" | "descend" | "retreat" | "back_corridor" | "rest" | "explore" | "pick_up" => false,
        // (a drink or a read of something held — the walk does not stop for a row with nothing to use)
        "drink" => held(arg, Some(Cat::Potion)),
        "read" => held(arg, Some(Cat::Scroll)),
        "throw" => held(arg.split(',').next().unwrap_or(""), None) && !v.foes.is_empty(),
        _ => targets_foes(&r.verb) && v.foes.iter().any(|&i| {
            let m = &run.monsters[i];
            m.pos.adjacent(run.hero.pos) && m.effective_speed() > run.hero.speed()
        }),
    };
    answers && r.conds.iter().all(|c| cond_holds(run, cx, v, c))
}

/// Cut 23 §3: each active row of the set (not a lent row) — whether its conds held at the
/// decision, and else its first failing cond's key (`unmet_key`).
fn rows_held(run: &Run, cx: &Ctx, v: &View) -> Vec<(usize, bool, Option<String>)> {
    cx.rules
        .active(cx.max_rows())
        .map(|(i, r)| {
            let failing = r.conds.iter().find(|c| !cond_holds(run, cx, v, c));
            (i, failing.is_none(), failing.map(unmet_key))
        })
        .collect()
}

/// Cut 23 §3: a failing cond's key in the tally — the thing never met for a tag, a sight, a
/// party member or an item (`gas`, `den`, `heal`), else the cond as written (`hp<30`).
pub fn unmet_key(c: &Cond) -> String {
    match (c.k.as_str(), c.t.as_deref()) {
        ("foe_tag" | "on_see" | "party" | "item", Some(t)) if !t.is_empty() => t.to_string(),
        (k, _) => match c.n {
            Some(n) => format!("{k}{n}"),
            None => k.to_string(),
        },
    }
}

/// Cut 23 §3: one action's why-not for every row: an action counted, the rows whose conds
/// held counted `matched`, the acting row `fired`; a row whose conds held that did not act
/// counts its reason (`no item`, `row guard`, `same as R2`, or `R3 first` when a row above
/// took the action before it was read); a row whose conds did not hold counts its first
/// failing cond.
fn tally_rows(run: &Run, cx: &mut Ctx, held: &[(usize, bool, Option<String>)], acted: i32) {
    let need = held.iter().map(|(i, _, _)| i + 1).max().unwrap_or(0);
    if cx.tally.len() < need {
        cx.tally.resize(need, Default::default());
    }
    let pre = run.rows_why.first().map(|w| w.why.clone());
    for (i, h, unmet) in held {
        let t = &mut cx.tally[*i];
        t.actions += 1;
        if *i as i32 == acted {
            t.fired += 1;
        }
        if *h {
            t.matched += 1;
            if *i as i32 != acted {
                let why = match run.rows_why.iter().find(|w| w.row == *i) {
                    Some(w) => w.why.clone(),
                    None if acted >= 0 => format!("R{} first", acted + 1),
                    None => pre.clone().unwrap_or_else(|| "chore first".into()),
                };
                if why != "fired, free" {
                    *t.blocked.entry(why).or_insert(0) += 1;
                }
            }
        } else if let Some(u) = unmet {
            *t.unmet.entry(u.clone()).or_insert(0) += 1;
        }
    }
}

/// Cut 23 §3: a row's tally as the tablet reads it (`Lineage.row_why`): `fired/actions acts`, then
/// what kept it quiet — its conds never met (`0/164 acts · no gas met`), or blocked when they held
/// (`2/164 acts · blocked · no scroll`) when the block outnumbers the fires.
pub fn row_stat(row: &crate::rules::Row, t: &crate::engine::RowTally) -> crate::wire::RowStat {
    let top = |m: &std::collections::BTreeMap<String, u32>| m.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0))).map(|(k, n)| crate::wire::WhyCount { why: k.clone(), n: *n });
    let blocked = top(&t.blocked);
    let unmet = top(&t.unmet);
    // QA on 524827b (qaAA: `0/587 · blocked` — "587 of what"): the count's unit, the hero's actions
    let head = format!("{}/{} acts", t.fired, t.actions);
    let text = if t.matched == 0 {
        match unmet.as_ref() {
            Some(u) => {
                let tagged = row.conds.iter().any(|c| matches!(c.k.as_str(), "foe_tag" | "on_see" | "party" | "item") && c.t.as_deref() == Some(u.why.as_str()));
                if tagged {
                    let verb = if row.conds.iter().any(|c| c.k == "item" && c.t.as_deref() == Some(u.why.as_str())) { "held" } else { "met" };
                    format!("{head} · no {} {verb}", u.why.replace('_', " "))
                } else {
                    format!("{head} · never {}", u.why)
                }
            }
            None => head,
        }
    } else {
        match blocked.as_ref() {
            Some(b) if b.n > t.fired => format!("{head} · blocked · {}", b.why),
            _ => head,
        }
    };
    crate::wire::RowStat { sends: t.sends, actions: t.actions, fired: t.fired, matched: t.matched, blocked, unmet, text }
}

/// Cut 23 §3 (AJ: "`read ✗ no use`, `attack ✗ no target` I could not explain"): every reason
/// a row does not act (`ROW_REASONS`' verb blocks, guards and pre-emptions) → its reason on
/// tap, ≤ 3 words. Keys are reason prefixes (`same as R` covers `same as R2`).
pub const WHY_GLOSS: &[(&str, &str)] = &[
    ("reloading", "reload in progress"),
    ("empty gun", "needs reloading"),
    ("gun full", "already loaded"),
    ("no short gun", "needs short gun"),
    ("no gun", "needs a gun"),
    ("no path", "way blocked"),
    ("no target", "no foe reachable"),
    ("no line", "shot blocked"),
    ("no bow", "needs a bow"),
    ("cooldown", "skill recharging"),
    ("no item", "none in pack"),
    ("unknown item", "kind unidentified"),
    ("no unknown", "no unknowns held"),
    ("no use", "no effect now"),
    ("no leash", "needs a leash"),
    ("none weak", "none weak enough"),
    ("not safe", "foes too near"),
    ("no stairs", "stairs not found"),
    ("going home", "heading home"),
    ("prayed", "prayed already"),
    ("no shrine", "no shrine here"),
    ("no way", "exit unreachable"),
    ("card passed", "its rows idle"),
    ("card idle", "no trigger foe"),
    // QA on 0c6e126 (qaY: `given up · chase given up` — one segment twice): the gloss says what the reason did not
    // QA on 308f045 (qaAC: `given up · out of reach` — "given up by whom, what was out of reach"): who gave up what
    ("given up", "hero quit chasing"),
    // QA on 524827b (qaAB: `foes fleeing · foes running off`, `row guard` — the gloss restated the reason): what the row did not do
    ("foes fleeing", "melee skips runners"),
    ("card blocked", "its move blocked"),
    ("brave held", "bravery held it"),
    ("stuck", "loop guard waits"),
    ("row guard", "paused: it looped"),
    ("same as R", "earlier row covers"),
    ("trait first", "trait acted first"),
    ("hazard first", "left the hazard"),
    ("recall sense", "recall read first"),
    ("paralysed", "cannot act"),
    ("confused", "stumbled instead"),
    ("bail", "called home"),
    ("locked cond", "cond not bought"),
    ("fired, free", "free action"),
];

/// Cut 23 §3: the gloss of a block reason for the verb it blocked — `read ✗ no use` →
/// `nothing to learn`, `drink ✗ no use` → `not needed now`; else `WHY_GLOSS`.
pub fn why_gloss(verb: &str, why: &str) -> Option<&'static str> {
    match (verb, why) {
        ("read", "no use") => return Some("nothing to learn"),
        ("drink", "no use") => return Some("not needed now"),
        ("throw", "no line") => return Some("no clear throw"),
        _ => {}
    }
    WHY_GLOSS.iter().filter(|(k, _)| why.starts_with(k)).max_by_key(|(k, _)| k.len()).map(|(_, g)| *g)
}

/// A condition this lineage can use (`LineageState::shadowed_by`'s test): a row whose
/// condition it does not own never fires, so it shadows nothing.
fn row_usable(cx: &Ctx, c: &Cond) -> bool {
    // Cut 26 §6: `on_see: K` is gated by its situation's fact alone (`cond_holds`), never by the
    // bare `on_see` cond's unlock.
    if c.k == "on_see" && c.t.as_deref().is_some_and(|t| !t.is_empty()) {
        return c.t.as_deref().is_some_and(|t| cx.facts.contains(t));
    }
    // Cut 30 §4: `trait <head>` / `gift live` are gated by the trait's fact.
    if matches!(c.k.as_str(), crate::traits::COND_TRAIT | crate::traits::COND_LIVE) {
        return crate::traits::cond_usable(cx.facts, c);
    }
    crate::meta::cond_unlock(&c.k).is_none_or(|u| cx.unlocks.contains(u))
}

/// A reason is from the table (a prefix match: the numbered shapes carry their number).
pub fn row_reason_ok(why: &str) -> bool {
    crate::rules::word_count(why) <= 3 && ROW_REASONS.iter().any(|r| why.starts_with(r))
}

/// Cut 26 §6: `cond_reason` for tests and tools.
pub fn row_why_of(run: &Run, cx: &Ctx, c: &Cond) -> String {
    cond_reason(run, cx, c)
}

/// The reason a condition does not hold, ≤ 3 words (`hp not <30%` reads `hp 8% ≥ 30%`).
fn cond_reason(run: &Run, cx: &Ctx, c: &Cond) -> String {
    let n = c.n.unwrap_or(0);
    let t = c.t.as_deref().unwrap_or("");
    // Cut 26 §6 (AP: `R3 descend · locked cond` beside `see: hunger`, a token his sheet had
    // offered): `on_see: K` is the situation's fact's, not the bare cond's unlock — it reads `no
    // hunger seen` (or `locked cond` while the fact is unknown, as the run holds it).
    let tagged_see = c.k == "on_see" && !t.is_empty();
    if matches!(c.k.as_str(), crate::traits::COND_TRAIT | crate::traits::COND_LIVE) {
        return crate::traits::cond_reason(cx, c);
    }
    if (tagged_see && !cx.facts.contains(t)) || (!tagged_see && crate::meta::cond_unlock(&c.k).is_some_and(|u| !cx.unlocks.contains(u))) {
        return "locked cond".into();
    }
    match c.k.as_str() {
        "hp<" => format!("hp not <{n}%"),
        "hp>" => format!("hp not >{n}%"),
        "foes>=" => format!("foes not ≥{n}"),
        "adj>=" => format!("adj not ≥{n}"),
        "foe_tag" => "not in view".into(),
        "foe_hp<" => "no weak foe".into(),
        "item" => "none held".into(),
        "lacks" => format!("has {t}"),
        "unknown_item" => "no unknown".into(),
        "floor_seen>=" => format!("seen not ≥{n}%"),
        "depth>=" => format!("depth not ≥{n}"),
        "in" => format!("not in {t}"),
        "alert>=" => format!("alert not ≥{n}"),
        "in_corridor" => "not corridor".into(),
        "path_stairs" => "no path".into(),
        "ally" => "no ally".into(),
        "loot>=" => format!("loot not ≥{n}"),
        "turns>" => format!("turns not >{n}"),
        "on_hurt" => "not hurt".into(),
        "on_kill" => "no kill".into(),
        "on_see" if t.is_empty() => "nothing new".into(),
        "on_see" => format!("no {t} seen"),
        "party" => format!("no {}", crate::engine::kind_title(t).to_lowercase()),
        "party_hp<" => "party hp ok".into(),
        _ => {
            let _ = run;
            "locked cond".into()
        }
    }
}

/// One row's reason. Sims (forecasts, verdict replays) never show a trace: no accounting.
/// Cut 11 §1: a state reason (`no item`, `none held`, `no path`, `not in view`, `cooldown`,
/// `locked cond`) carries its `because` from the provenance log; `row` is the row (a verb
/// block), `cond` the failing condition.
fn row_why(run: &mut Run, cx: &mut Ctx, i: usize, why: &str, row: Option<&crate::rules::Row>, cond: Option<&Cond>) {
    if !cx.sim {
        let because = crate::provenance::because_for(run, cx, why, row, cond);
        run.rows_why.push(RowWhy { row: i, why: why.into(), because });
    }
}

/// Every row (the unlocked ones plus a lent row) with one reason: the action was decided
/// before the rules were read (a trait, a hazard step, paralysis, the bail). Cut 13 §2: a
/// trait's pre-emption carries its `because` (`cowardly ran first`) on every row it held.
fn all_rows_why(run: &mut Run, cx: &Ctx, why: &str, because: Option<Because>) {
    if cx.sim {
        return;
    }
    let mut idx: Vec<usize> = cx.rules.active(cx.max_rows()).map(|(i, _)| i).collect();
    if run.lent_row.is_some() {
        idx.push(cx.rules.rows.len());
    }
    run.rows_why = idx.into_iter().map(|i| RowWhy { row: i, why: why.into(), because: because.clone() }).collect();
}

/// Cut 15 §6: a pre-emption that held every row, said once — one `RowWhy` on the first
/// active row (`R1 hazard first · gas`) instead of the same words on each (V: "`hazard
/// first` on every row reads as nothing"). The client renders `rows` verbatim, so the footer
/// reads it once; a trace turn whose `rows` is this single entry stands for all of them.
fn once_rows_why(run: &mut Run, cx: &Ctx, why: &str) {
    if cx.sim {
        return;
    }
    run.rows_why = cx.rules.active(cx.max_rows()).map(|(i, _)| i).next().map(|i| RowWhy { row: i, why: why.into(), because: None }).into_iter().collect();
}

/// Policy retreats in one engagement (a foe in view throughout, no blow on the hero) before the
/// outpaced guard rests the retreating row.
pub const OUTPACED_RETREATS: u32 = 20;
/// A bloodless dance (`Batch.dances`): `DANCE_ACTIONS` foe-facing row actions of one engagement
/// with no blood drawn either way, `DANCE_MOVES` of them policy retreats (the guard's kind: the
/// player's own row stepping away, on no condition of his HP) the guard did not rest — more
/// than it allows, so a dance is one the guard missed.
pub const DANCE_ACTIONS: u32 = 60;
pub const DANCE_MOVES: u32 = 30;

/// QA on 778fa1b (qaV: `R4 foe: heavy → retreat` / `R6 attack nearest` taking turns before two
/// ogres for six minutes at 21/42 — each step back kept pace with the pursuers, each wind-up
/// missed, and the hero crossed the room, so the oscillation guard never saw a pacing; the
/// same-row guard's streak broke on every attack, and blood on a goblin between reset every
/// clock): a retreat that cannot shake its pursuers — `OUTPACED_RETREATS` retreats of one
/// engagement (a foe in view throughout) with no blow on the hero — is rested for 30 actions
/// (`row guard` on its trace line), and the next row decides (the attack row fights). The count
/// starts again when no foe is in view, on a new floor, or when the hero is hit (a retreat under
/// blows is not a dance).
///
/// Only a retreat the player wrote as a policy toward foes: a row conditioned on the hero's HP
/// (`hp < 40% → retreat`) is a flee for a low moment — the player's "not now", which the guard
/// does not overrule by sending him to fight at 40 % (resting FULL's flee walked FULL−D28 past
/// the Lich without silence on 9 of 30 seeds: the wall is the counter's); a card's step is the
/// card's tactic (`gas_step` away from a bloat), not the player's row.
fn outpaced_guard(run: &mut Run, cx: &mut Ctx, row: i32, verb: &Verb) {
    // A retreat the player wrote toward foes: his own row (a card's tactic is the game's),
    // stepping away, on no condition of his HP.
    let policy = row >= 0
        && matches!(verb.v.as_str(), "retreat" | "back_corridor")
        && cx.rules.rows.get(row as usize).or(run.lent_row.as_ref()).is_some_and(|r| !r.is_card() && !r.conds.iter().any(|c| c.k == "hp<"));
    // (the dance measure: blood drawn on this action, or taken since the last one, ends it)
    if run.actions.saturating_sub(run.last_damage_action) <= 1 {
        run.bloodless = (0, 0, run.bloodless.2);
    } else if row >= 0 && targets_foes(verb) {
        run.bloodless.0 += 1;
        run.bloodless.1 += policy as u32;
        if run.bloodless.1 >= DANCE_MOVES {
            run.bloodless.2 = run.bloodless.2.max(run.bloodless.0);
        }
    }
    let hp = run.hero.pos;
    let near = run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && !m.dormant && !m.summoned && run.floor.map.is_visible(m.pos)).map(|m| m.pos.cheb(hp)).min();
    let Some(near) = near else {
        // (out of the engagement: the dance and the retreats count again from the next)
        run.retreats = (0, 0);
        run.bloodless.0 = 0;
        run.bloodless.1 = 0;
        return;
    };
    if run.hurt_last || run.hurt_since_action {
        run.retreats = (0, 0);
    }
    if !policy {
        return;
    }
    // (the nearest foe after the step, for the record)
    run.retreats = (run.retreats.0 + 1, near);
    if run.retreats.0 >= OUTPACED_RETREATS {
        run.row_suppressed = (row, run.actions + 30);
        run.row_streak = (-9, 0);
        run.retreats = (0, 0);
        // (the guard answered this dance: the measure counts the next one)
        run.bloodless.0 = 0;
        run.bloodless.1 = 0;
        emit_rule(run, cx, -2, &Verb::new("stuck"), "stuck → chores");
    }
}

/// Cut 24 §1: the hero's actions in one engagement (a foe in view; `NOHP_CLEAR` actions out of
/// sight end it) with no HP moved on either side (`Run.nohp`) before the guard ends the dance.
/// 60: the cohort sets' fights that move never go 60 actions without a blow landing (p50 of the
/// sends' longest stretch 20–40), and the FULL set's countered boss fights' still stretches peak
/// at 27–40 (the Lich, the Warlord).
pub const NOHP_ACTIONS: u32 = 60;
/// Cut 24 §1: the hero's actions with a boss in view and its HP unmoved before it wins its
/// fight (`Run.boss_still`).
pub const BOSS_STILL: u32 = 60;
/// Actions with no foe in view that end an engagement (a retreat out of sight and back is one).
const NOHP_CLEAR: u32 = 10;

/// Cut 24 §1: a fight that cannot progress ends. The measure (`Run.nohp`): the hero's actions
/// with a foe in view since HP last moved on either side (a summon's blood, or a summon's blow,
/// is no progress: reserves are endless). A boss in view whose HP has not moved for
/// `BOSS_STILL` of the hero's actions — nor has one of his own summons (a Lich's skeletons, a
/// rally's goblins; never a shield-wall reserve) fallen — has won (`Run.boss_still`): the hero
/// is driven off (`driven_off`). A dance (`NOHP_ACTIONS`) rests the rows that danced.
fn nohp_guard(run: &mut Run, cx: &mut Ctx, row: i32, verb: &Verb) {
    if run.over.is_some() {
        return;
    }
    let map = &run.floor.map;
    let in_view = run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && !m.dormant && map.is_visible(m.pos));
    if in_view {
        // (every action counts while a foe is in view: a dance whose other half is a chore —
        // `R8 pack break ↔ descend` — is as dead to watch)
        run.nohp.1 = 0;
        run.nohp.0 += 1;
    } else {
        run.nohp.1 += 1;
        if run.nohp.1 >= NOHP_CLEAR {
            run.nohp.0 = 0;
        }
    }
    let boss = run.monsters.iter().position(|m| m.hp > 0 && m.hostile() && m.is_boss() && map.is_visible(m.pos));
    if let Some(bi) = boss {
        let (id, hp) = (run.monsters[bi].id, run.monsters[bi].hp);
        run.boss_still = match run.boss_still {
            Some((bid, bhp, n)) if bid == id && bhp == hp => Some((id, hp, n + 1)),
            _ => Some((id, hp, 0)),
        };
    }
    let still = run.boss_still.map_or(0, |b| b.2);
    run.nohp.2 = run.nohp.2.max(run.nohp.0).max(still);
    // A boss whose HP has not moved in `BOSS_STILL` of the hero's actions with it in view (his
    // blows all shrugged — a shield wall, a counter unwritten) wins its fight.
    if let Some(bi) = boss.filter(|_| still >= BOSS_STILL) {
        driven_off(run, cx, bi);
        return;
    }
    // A dance (both bars still for `NOHP_ACTIONS` foe-facing actions): the rows that danced
    // rest for 30 actions and the next row decides — the attack row fights, or the chores go on.
    if run.nohp.0 >= NOHP_ACTIONS {
        let mut rows: Vec<i32> = run.trace.iter().filter(|t| t.row >= 0 && targets_foes(&t.verb)).map(|t| t.row).collect();
        if row >= 0 && targets_foes(verb) {
            rows.push(row);
        }
        rows.sort();
        rows.dedup();
        run.nohp.0 = 0;
        if !rows.is_empty() {
            run.rows_rested = (rows, run.actions + 30);
            run.row_streak = (-9, 0);
            emit_rule(run, cx, -2, &Verb::new("stuck"), "stuck → chores");
        }
    }
}

/// Cut 24 §1: the boss at `bi` drove the hero off his floor — a `return`-tier exit whose line
/// reads `no counter` with the boss's defence and the counter to learn (`ExitLine.driven`).
/// Watching him shrug every blow teaches the counter, if the telegraph had not.
pub fn driven_off(run: &mut Run, cx: &mut Ctx, bi: usize) {
    let kind = run.monsters[bi].kind.clone();
    run.driven_off = Some(kind.clone());
    // (the player's own way home carries the pack out; with none written, it is dropped)
    run.driven_lost = !cx.rules.active(cx.max_rows()).any(|(_, r)| matches!(r.verb.v.as_str(), "return" | "bank")) && run.lent_row.as_ref().is_none_or(|r| !matches!(r.verb.v.as_str(), "return" | "bank"));
    crate::facts::learn_boss_counter(run, cx, &kind);
    callout(run, cx, "driven off");
    emit_rule(run, cx, -2, &Verb::new("return"), "driven off");
    let depth = run.depth;
    note(run, cx, format!("Driven off D{depth} by the {}.", crate::sifter::boss_short(&kind)));
    end_run(run, cx, ExitTier::Return);
}

/// If the last 12 actions visited ≤ 2 tiles with no damage dealt or taken, give up on the
/// visible foes for 30 actions and let chores proceed; one `stuck` chore event explains it.
fn oscillation_guard(run: &mut Run, cx: &mut Ctx) {
    if run.stuck_until > run.actions || run.recent_pos.len() < 12 {
        return;
    }
    // (more than two distinct tiles in the window: not pacing — counted in place, every action)
    let p = &run.recent_pos;
    let other = p.iter().find(|q| **q != p[0]);
    let paced = other.is_none_or(|o| p.iter().all(|q| *q == p[0] || q == o));
    if !paced || run.actions.saturating_sub(run.last_damage_action) < 12 {
        return;
    }
    // Engaged in melee is not stuck: adjacent foes are always worth a row.
    let hp = run.hero.pos;
    let v = view(run);
    // A summoned foe at the elbow is the loop itself, not an engagement (QA on 23ed91f).
    if v.foes.iter().any(|&i| run.monsters[i].pos.adjacent(hp) && !run.monsters[i].summoned) {
        return;
    }
    let ids: Vec<u32> = v.foes.iter().map(|&i| run.monsters[i].id).collect();
    if ids.is_empty() {
        return;
    }
    // Cut 13 §1: the guard's moment — the nearest foe it gave up on and why (`goblin archer,
    // no path` · `eel, across water`) — is the stall's cause on the record, the reel line and
    // the chronicle note; the first guard's tick is the verdict's checkpoint.
    // Cut 18 §4: when the pacing was the rules' own — two actors taking turns (`R2 retreat ↔
    // explore`) or one row stepping back and forth (`R1 retreat paced`) — the loop is the
    // cause, and it stays the floor's cause through the later guards (whose windows are the
    // guard's own waiting); the verdict's first patch addresses that row (`stuck_row`).
    if let Some((cause, row)) = row_loop(&run.trace) {
        // Cut 27 §4: a card taking turns with a chore (`R8 pack ↔ pick up`) is the engine's own
        // loop — the card's sub-rows are not the player's — counted apart (the metrics' row).
        let card = cx.rules.rows.get(row as usize).is_some_and(|r| r.verb.v == "tactic");
        let chore = run.trace.iter().rev().take(LOOP_WINDOW).any(|t| t.row == -2 && t.verb.v != "stuck");
        run.card_loops += (card && chore) as u32;
        run.loop_causes.push(cause.clone());
        // Cut 30 §2: a package's row in the loop (its rows are ours, not the player's) rests for the
        // floor's next 300 actions and the chores go on — no stall is counted against the send
        if cx.rules.rows.get(row as usize).is_some_and(|r| r.is_pkg()) {
            run.rows_rested = (vec![row], run.actions + 300);
            run.row_streak = (-9, 0);
            run.stuck_until = run.actions + 30;
            run.recent_pos.clear();
            run.chase = None;
            emit_rule(run, cx, -2, &Verb::new("stuck"), "stuck → chores");
            return;
        }
        run.stuck_cause = Some(cause);
        run.stuck_row = Some(row);
    } else if run.stuck_row.is_none() {
        if let Some(&i) = v.foes.iter().find(|&&i| !run.monsters[i].summoned).or(v.foes.first()) {
            let m = &run.monsters[i];
            let why = if run.floor.map.get(m.pos) == crate::tiles::Tile::Water { "across water" } else { "no path" };
            run.stuck_cause = Some(format!("{}, {why}", crate::engine::kind_title(&m.kind).to_lowercase()));
        }
    }
    if run.stuck_fires == 0 {
        run.stuck_first_t = Some(run.turn);
    }
    // Until the floor changes (`u32::MAX`, blood does not lift it): thirty actions let the
    // same unreachable foe pull the hero back into the pacing, three guards a stall.
    for id in ids {
        run.ignore(id, u32::MAX);
    }
    // A pacing made of `pick up` chores (an item the hero steps toward and away from) gives
    // the floor's items up as well, for the floor: the last DEFAULT stalls were all this.
    let pickups = run.trace.iter().rev().take(12).filter(|t| t.row == -2 && t.verb.v == "pick_up").count();
    if pickups >= 6 {
        run.items_until = u32::MAX;
    }
    run.stuck_until = run.actions + 30;
    run.stuck_fires += 1;
    run.recent_pos.clear();
    run.chase = None;
    // The rows that read `stuck` for the next 30 actions link back here (QA on 952e306:
    // "`stuck` unexplained"): `← paced 12 turns, foes ignored · t`.
    crate::provenance::stuck(run, cx);
    let verb = Verb::new("stuck");
    emit_rule(run, cx, -2, &verb, "stuck → chores");
}

/// QA on 524827b: a floor's `pick up` chores with nothing taken (whatever came between) after
/// which the floor's items are given up (the qa invariant's bar is 50).
pub const PICKUP_DRY_MAX: u32 = 40;

/// Cut 18 §4: the rules' loop in the guard's window (the last `LOOP_WINDOW` actions): exactly
/// two actors taking turns, each at least `LOOP_MIN` times, one of them a row (`R2 retreat ↔
/// explore`, `R5 corridor ↔ R8 attack`), or one moving row alone (`R1 retreat paced`) — ≤ 4
/// words.
/// Returns the cause and the row it names (the moving row of two, else the higher one).
pub fn row_loop<T: std::borrow::Borrow<TraceTurn>>(trace: &[T]) -> Option<(String, i32)> {
    let win: Vec<&TraceTurn> = trace.iter().rev().take(LOOP_WINDOW).map(|t| t.borrow()).filter(|t| t.verb.v != "stuck").collect();
    if win.len() < LOOP_WINDOW - 2 {
        return None;
    }
    let mut keys: Vec<(i32, String, usize)> = Vec::new();
    for t in &win {
        if t.row < 0 && t.row != -2 {
            return None; // a trait's or a curious use is not the rules' loop
        }
        let word = loop_word(t);
        match keys.iter_mut().find(|k| k.0 == t.row && k.1 == word) {
            Some(k) => k.2 += 1,
            None => keys.push((t.row, word, 1)),
        }
    }
    let side = |k: &(i32, String, usize)| if k.0 >= 0 { format!("R{} {}", k.0 + 1, k.1) } else { k.1.clone() };
    match keys.as_slice() {
        // One row alone is a loop when it moves the hero (a retreat stepping back and forth);
        // a targeting row pacing before an unreachable foe keeps the foe's cause (`archer no path`).
        [a] if a.0 >= 0 && win.iter().all(|t| MOVING.contains(&t.verb.v.as_str())) => Some((format!("{} paced", side(a)), a.0)),
        [a, b] if (a.0 >= 0 || b.0 >= 0) && a.2 >= LOOP_MIN && b.2 >= LOOP_MIN => {
            let (a, b) = if b.0 >= 0 && (a.0 < 0 || b.0 < a.0) { (b, a) } else { (a, b) };
            let moving = |k: &(i32, String, usize)| k.0 >= 0 && win.iter().any(|t| t.row == k.0 && MOVING.contains(&t.verb.v.as_str()));
            let row = if b.0 >= 0 && moving(b) && !moving(a) { b.0 } else { a.0 };
            Some((format!("{} ↔ {}", side(a), side(b)), row))
        }
        _ => None,
    }
}

const LOOP_WINDOW: usize = 12;
const LOOP_MIN: usize = 3;
/// The verbs that move the hero off his tile (a loop's usual half).
const MOVING: &[&str] = &["retreat", "back_corridor", "blink", "shadowstep", "vanish", "smoke", "descend", "explore"];

/// A trace turn's actor word: a row's verb as the callout reads it, one word (`retreat`,
/// `corridor`, `attack`, `drink`); a chore's short (`explore`, `pick up`).
fn loop_word(t: &TraceTurn) -> String {
    let s = t.verb.short();
    if t.row >= 0 {
        s.split(' ').next().unwrap_or(&t.verb.v).to_string()
    } else {
        s
    }
}

/// Cut 18 §4: a stall cause in the loop form `row_loop` writes.
pub fn loop_cause_ok(cause: &str) -> bool {
    let row_side = |s: &str| s.split_once(' ').is_some_and(|(r, w)| r.len() > 1 && r.starts_with('R') && r[1..].chars().all(|c| c.is_ascii_digit()) && !w.is_empty() && !w.contains(' '));
    if let Some(side) = cause.strip_suffix(" paced") {
        return row_side(side);
    }
    let Some((a, b)) = cause.split_once(" ↔ ") else { return false };
    let chore = |s: &str| !s.is_empty() && !s.starts_with('R') && crate::rules::word_count(s) <= 2;
    (row_side(a) && (row_side(b) || chore(b))) && crate::rules::word_count(cause) <= 4
}

/// Callout text for a fired row: `cond → verb`, or for a verb that acted on a foe the act and
/// its kind (`attack goblin`, `shoot goblin archer`; Cut 4), ≤ 3 words.
fn rule_text(run: &Run, row: &crate::rules::Row, hp_pct: i32) -> String {
    if targets_foes(&row.verb) {
        if let Some(m) = run.last_target.and_then(|id| run.monsters.iter().find(|m| m.id == id)) {
            return format!("{} {}", row.verb.short(), crate::engine::kind_title(&m.kind).to_lowercase());
        }
    }
    row.text(hp_pct)
}

/// The hero's max hp moved by `delta` (already applied): the `max_hp` event, and (Cut 28 §2) the
/// run's own record of the step for the traces (`Run.max_steps` → `Trace.max_steps`).
pub fn hero_max_hp(run: &mut Run, cx: &mut Ctx, delta: i32, cause: &str) {
    cx.events.push(Ev::MaxHp { t: run.turn, id: crate::engine::HERO_ID, max: run.hero.max_hp, delta, cause: cause.into() });
    if !cx.sim {
        run.max_steps.push(crate::wire::MaxStep { t: run.turn, max: run.hero.max_hp, delta, cause: cause.into() });
    }
}

pub fn emit_rule(run: &Run, cx: &mut Ctx, row: i32, verb: &Verb, text: &str) {
    cx.events.push(Ev::Rule { t: run.turn, row, verb: verb.clone(), text: crate::chronicle::clamp_words(text, 3) });
}

fn emit_rule_owned(run: &Run, cx: &mut Ctx, row: i32, verb: &Verb, text: String) {
    cx.events.push(Ev::Rule { t: run.turn, row, verb: verb.clone(), text: crate::chronicle::clamp_words_owned(text, 3) });
}

/// Verbs that act on the visible foes (suppressed while the oscillation guard is up).
pub fn targets_foes(verb: &Verb) -> bool {
    matches!(
        verb.v.as_str(),
        "attack" | "shield_bash" | "throw" | "tame" | "cleave" | "backstab" | "ambush" | "shadowstep" | "send" | "taunt"
            | "shoot" | "volley" | "mark" | "double_shot" | "bolt" | "slow" | "drain" | "fire" | "close_burst"
            // a tactic card acts on the foes it sees; while stuck it must yield like any targeting row
            | "tactic" | "back_corridor" | "retreat"
    )
}

pub fn cond_holds(run: &Run, cx: &Ctx, v: &View, c: &Cond) -> bool {
    let n = c.n.unwrap_or(0);
    let t = c.t.as_deref().unwrap_or("");
    let h = &run.hero;
    // Cut 5 §4: `on_see: nest | shrine | vault` — a situation tile in view (gated by its fact,
    // not by the `cond_on_see` unlock).
    if c.k == "on_see" && !t.is_empty() {
        return cx.facts.contains(t) && run.sees_situation(t);
    }
    // Cut 30 §4: the heir's trait and its gift live (fact-gated).
    if matches!(c.k.as_str(), crate::traits::COND_TRAIT | crate::traits::COND_LIVE) {
        return crate::traits::cond_holds(run, cx, c);
    }
    // Cut 2 §3: some condition tokens are unlocks; a row using one the lineage does not own
    // never fires.
    if crate::meta::cond_unlock(&c.k).is_some_and(|u| !cx.unlocks.contains(u)) {
        return false;
    }
    match c.k.as_str() {
        "hp<" => h.hp_pct() < n,
        "hp>" => h.hp_pct() > n,
        "foes>=" => v.foes.len() as i32 >= n,
        "adj>=" => v.adj >= n,
        "foe_tag" => v.foes.iter().any(|&i| {
            let m = &run.monsters[i];
            m.has_tag(t) && tag_known(cx.facts, &m.kind, t)
        }),
        // Cut 2 §5: reading a foe's wounds needs the kind studied (five kills).
        "foe_hp<" => v.foes.iter().any(|&i| {
            let m = &run.monsters[i];
            crate::facts::is_studied(cx.facts, &m.kind) && m.hp * 100 / m.max_hp.max(1) < n
        }),
        "item" => h.inv.iter().chain(h.weapon.iter()).chain(h.armour.iter()).any(|i| i.kind == t && i.is_known(cx.facts, cx.flavours)),
        "unknown_item" => h.inv.iter().any(|i| i.is_consumable() && !i.is_known(cx.facts, cx.flavours)),
        "lacks" => !h.inv.iter().any(|i| i.kind == t),
        "floor_seen>=" => run.floor.map.seen_pct() >= n,
        "depth>=" => run.depth as i32 >= n,
        "alert>=" => run.alert >= n,
        "in_corridor" => run.floor.map.is_corridor(h.pos),
        "path_stairs" => {
            let s = run.floor.stairs_down;
            run.floor.map.is_seen(s) && {
                let d = run.floor.map.bfs(h.pos, true, &|p| run.monster_at(p).is_some());
                d[run.floor.map.idx(s)] >= 0
            }
        }
        "ally" => run.allies().next().is_some(),
        "loot>=" => run.loot >= n,
        "turns>" => (run.floor_turn / TICKS_PER_TURN) as i32 > n,
        "on_hurt" => run.hurt_since_action,
        "on_kill" => run.kill_since_action,
        "on_see" => run.new_seen,
        "party" => run.party_alive().any(|m| m.kind == t),
        "party_hp<" => run.party_alive().any(|m| m.hp * 100 / m.max_hp.max(1) < n),
        // Cut 26 §2: the floor's biome on the run's route (a lane's rows fire in its lane).
        "in" => run.biome().name() == t,
        _ => false,
    }
}

/// Where damage comes from, for causes and counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Src {
    Hero { ranged: bool },
    Mon(usize),
    Gas,
    Fire,
    Poison,
    Burst,
    /// Cut 3: a blow sent back by a reflecting monster (never reflected again).
    Reflect(usize),
}

impl Src {
    pub fn cause(&self, run: &Run) -> String {
        match self {
            Src::Hero { .. } => "hero".into(),
            Src::Mon(i) | Src::Reflect(i) => run.monsters[*i].kind.clone(),
            Src::Gas => "gas".into(),
            Src::Fire => "fire".into(),
            Src::Poison => "poison".into(),
            Src::Burst => "burst".into(),
        }
    }
    pub fn has_tag(&self, run: &Run, tag: &str) -> bool {
        match self {
            Src::Hero { ranged } => *ranged && tag == "ranged",
            Src::Mon(i) => run.monsters[*i].has_tag(tag),
            Src::Reflect(_) => false,
            Src::Gas | Src::Burst => tag == "gas",
            Src::Fire => tag == "fire",
            Src::Poison => tag == "poison",
        }
    }
}

/// Is a monster alone (no friend within 2 tiles)? The hero has no tags and is never "lone".
pub fn is_lone(run: &Run, pos: Pos, hostile: bool) -> bool {
    if pos == run.hero.pos {
        return false;
    }
    if hostile {
        !run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.pos != pos && m.pos.cheb(pos) <= 2)
    } else {
        let hero_near = run.hero.pos != pos && run.hero.pos.cheb(pos) <= 2;
        !hero_near && !run.monsters.iter().any(|m| m.hp > 0 && m.ally && m.pos != pos && m.pos.cheb(pos) <= 2)
    }
}

/// Apply the counter table (Addendum A): the adjusted damage and the counter observed.
pub fn counter_damage(run: &Run, src: &Src, dmg: i32, target: Option<usize>, target_pos: Pos, target_hostile: bool) -> (i32, Option<(String, String)>) {
    let on_water = run.floor.map.get(target_pos) == Tile::Water;
    let tgt_has = |t: &str| target.is_some_and(|i| run.monsters[i].has_tag(t));
    for (a, b, immune) in crate::defs::COUNTERS {
        let hit = if *immune {
            let def_has = tgt_has(a) || (*a == "water" && on_water);
            def_has && src.has_tag(run, b)
        } else {
            let tgt = if *b == "lone" { is_lone(run, target_pos, target_hostile) } else { tgt_has(b) };
            src.has_tag(run, a) && tgt
        };
        if hit {
            let out = if *immune { 0 } else { dmg * 3 / 2 };
            return (out, Some((a.to_string(), b.to_string())));
        }
    }
    (dmg, None)
}

/// Hero takes damage from `src`.
pub fn damage_hero(run: &mut Run, cx: &mut Ctx, dmg: i32, src: &Src) {
    if dmg <= 0 || run.over.is_some() {
        return;
    }
    let cause = src.cause(run);
    let cause = cause.as_str();
    let dmg=match src {Src::Mon(i) if run.monsters[*i].hostile()&&run.monsters[*i].hex_t>0=>(dmg-crate::specialization::HEX_REDUCTION).max(0),_=>dmg};
    let (dmg, counter) = counter_damage(run, src, dmg, None, run.hero.pos, false);
    if let Some((a, b)) = counter {
        learn(run, cx, crate::defs::counter_fact(&a, &b));
    }
    // Cut 3: fire resistance shrugs off fire (hazard or fire-tagged bites).
    let dmg = if run.hero.resist_fire_t > 0 && src.has_tag(run, "fire") { 0 } else { dmg };
    let dmg=if run.hero.legacy_effects!=0 {
        let mut dmg=dmg;
        if crate::legacy::has(&run.hero,crate::legacy::CLEAR_LUNGS)&&(src.has_tag(run,"gas")||src.has_tag(run,"poison")) {dmg=(dmg-2).max(0);}
        if crate::legacy::has(&run.hero,crate::legacy::FIREWARD)&&src.has_tag(run,"fire") {dmg/=2;}
        if crate::legacy::has(&run.hero,crate::legacy::BRACE)&&i64::from(run.hero.hp)*4<i64::from(run.hero.max_hp) {dmg=(dmg-2).max(0);}
        dmg
    }else{dmg};
    if dmg <= 0 {
        return;
    }
    // Cut 3: a mirror scroll sends the next blow back to whoever struck.
    if run.hero.mirror_charge > 0 {
        if let Src::Mon(i) = src {
            let i = *i;
            run.hero.mirror_charge -= 1;
            let mid = run.monsters[i].id;
            cx.events.push(Ev::Attack { t: run.turn, src: HERO_ID, dst: mid, dmg, hit: true, verb: Some("mirror".into()) });
            callout(run, cx, "mirrored");
            damage_monster(run, cx, i, dmg, &Src::Reflect(i));
            return;
        }
    }
    let riposte=if run.hero.riposte_t>0&&crate::specialization::has(&run.hero,crate::specialization::Style::Sentinel) {
        match src {Src::Mon(i) if run.monsters[*i].hp>0&&run.monsters[*i].hostile()
            &&!run.monsters[*i].has_tag("ranged")&&run.monsters[*i].pos.cheb(run.hero.pos)<=1=>Some(*i),_=>None}
    }else {None};
    let dmg=if riposte.is_some() {run.hero.riposte_t=0;dmg/2+dmg%2}else{dmg};
    let hp_before=run.hero.hp;
    run.hero.hp -= dmg;
    // Cut 24 §1: the hero's HP moved — the fight is going somewhere, unless a summon drew it
    // (a boss's endless reserves are no progress either way).
    if !matches!(src, Src::Mon(i) if run.monsters[*i].summoned) {
        run.fight_moved();
    }
    // QA on a946e04 (qaT: `R1 drank poison at 17/36 hp`, then a goblin's blow — ~30 deaths,
    // never a `row`): the harm the last unknown gamble has dealt so far (its poison, its
    // caustic cloud, its fire, inside `trace::GAMBLE_WINDOW`) — a death it made the difference
    // in is the gamble's (`trace::gamble_row`).
    if let Some((t, k, true)) = run.gambles.last() {
        if crate::trace::gamble_cause(k) == Some(cause) && run.turn.saturating_sub(*t) <= crate::trace::GAMBLE_WINDOW {
            run.gamble_harm += dmg;
        }
    }
    // QA on 778fa1b (qaV): and the harm of the hero's own throw (`Run.own_throw`).
    if let Some((t, k)) = &run.own_throw {
        if crate::trace::gamble_cause(k) == Some(cause) && run.turn.saturating_sub(*t) <= crate::trace::GAMBLE_WINDOW {
            run.own_throw_harm += dmg;
        }
    }
    run.hurt_since_action = true;
    run.last_damage_action = run.actions;
    // Cut 4: blood drawn ends every stalemate guard — the oscillation guard and the same-row
    // guard suppressed the attack row for 30 actions while a pack bit the hero (five `wait`
    // chores at 9 → 1 HP), and a foe given up on as unreachable is worth engaging once it hits.
    run.unstick();
    if run.boss_seen_t.is_some() {
        run.hurt_since_boss = true;
    }
    cx.events.push(Ev::Hurt { t: run.turn, id: HERO_ID, dmg, hp: run.hero.hp.max(0), cause: cause.into() });
    if let Src::Mon(i)=src {
        let m=&mut run.monsters[*i];
        let amount=crate::endgame::leech(m,run.hero.pos,dmg.min(hp_before.max(0)));
        if amount>0 {cx.events.push(Ev::Recover{t:run.turn,id:m.id,amount,hp:m.hp,src:crate::wire::RecoverySource::Leeching});}
    }
    // Cut 25 §3 (AN: an offline trace 14 → 0 on one `goblin −2` row): every blow since the hero's
    // last action, for the death trace's rows (`Trace.blows`).
    if run.blows.len() < BLOWS_CAP {
        run.blows.push(crate::wire::TraceBlow { t: run.turn, by: cause.into(), dmg, hp: run.hero.hp.max(0) });
    } else if let Some(last) = run.blows.last_mut() {
        // (past the cap the last row carries the rest: the rows still add up to the hp lost)
        last.dmg += dmg;
        last.hp = run.hero.hp.max(0);
    }
    // QA on 524827b (qaAA): the hp lost since full, per cause — a blow from full hp starts it over.
    if run.hero.hp + dmg >= run.hero.max_hp {
        run.hp_lost.clear();
    }
    if run.hp_lost.is_empty() {
        run.hp_lost_from = run.hero.hp + dmg;
    }
    match run.hp_lost.iter_mut().find(|(c, _)| c == cause) {
        Some(e) => e.1 += dmg,
        None => run.hp_lost.push((cause.into(), dmg)),
    }
    let pct = run.hero.hp_pct();
    if run.hero.hp > 0 {
        run.low_hp = run.low_hp.min(run.hero.hp);
        // Cut 5 §1: the episode's low point (and the hero's word on it).
        let flag = match src {
            Src::Mon(i) | Src::Reflect(i) => {
                let m = &run.monsters[*i];
                if m.nest {
                    Some("nest")
                } else if m.stray {
                    Some("stray")
                } else if m.situation.as_deref() == Some("den") {
                    Some("den")
                } else {
                    None
                }
            }
            // Cut 7 §3: gas on the lock floor is the lock's.
            Src::Gas | Src::Burst if !run.lock_tiles.is_empty() => Some("lock"),
            _ => None,
        };
        if sifter::on_hurt(run, cause, flag) {
            sifter::voice(run, cx, Moment::Low);
        }
        if pct <= 10 && run.low10_t.is_none() {
            run.low10_t = Some(run.turn);
            let hp = run.hero.hp;
            note(run, cx, format!("Down to {hp} HP."));
            callout(run, cx, "near death");
        } else if pct <= 20 && run.low20_t.is_none() {
            run.low20_t = Some(run.turn);
        }
    }
    if run.hero.hp <= 0 {
        run.death_short = 1 - run.hero.hp;
        run.hero.hp = 0;
        run.death_cause = Some(cause.to_string());
        run.death_modifiers = match src { Src::Mon(i) | Src::Reflect(i) => run.monsters[*i].modifiers, _ => None };
        run.death_blow = dmg;
        run.death_t = Some(run.turn);
        cx.events.push(Ev::Die { t: run.turn, id: HERO_ID, cause: cause.into() });
        let depth = run.depth;
        note(run, cx, format!("Slain by {} on D{}.", crate::engine::kind_title(cause), depth));
        // The trace's last row names the killer in its count (qaN: `foes 0` under an archer's shot).
        let sight = run.vision(cx.unlocks);
        trace_seen(run, Some(sight));
        end_run(run, cx, ExitTier::Death);
    }
    if let Some(i)=riposte.filter(|i|run.hero.hp>0&&run.over.is_none()&&run.monsters[*i].hp>0) {
        crate::ai::riposte_hit(run,cx,i);
    }
}

/// Monster takes damage; handles counters, splits, pops, drops, kills. Returns true if it died.
pub fn damage_monster(run: &mut Run, cx: &mut Ctx, mi: usize, dmg: i32, src: &Src) -> bool {
    if run.monsters[mi].hp <= 0 {
        return false;
    }
    // Cut 28 §1: a boss the fire hurt (the `fire` oath: slain by a run that burned him).
    if *src == Src::Fire && run.monsters[mi].is_boss() && !run.burned.contains(&run.monsters[mi].kind) {
        let k = run.monsters[mi].kind.clone();
        run.burned.push(k);
    }
    // The Warlord's shield wall: a goblin beside him takes any incidental blow from the
    // hero's side (unaimed swings, allies, companions). Hazards and aimed strikes go through.
    let mut mi = mi;
    // Cut 16 §4: broken, he has no wall to step behind.
    if run.monsters[mi].kind == "goblin_warlord" && !run.aimed && !run.monsters[mi].broken {
        let from_hero_side = match src {
            Src::Hero { .. } => true,
            Src::Mon(j) => run.monsters[*j].ally,
            _ => false,
        };
        if from_hero_side {
            let wp = run.monsters[mi].pos;
            // Any goblin in his view interposes; only with the goblins gone do stray swings land.
            let nearest = run
                .monsters
                .iter()
                .enumerate()
                .filter(|(_, o)| o.hp > 0 && o.hostile() && o.kind == "goblin" && o.pos.cheb(wp) <= crate::tiles::VISION)
                .min_by_key(|(_, o)| (o.pos.cheb(wp), o.id))
                .map(|(k, _)| k);
            let guard = nearest.or_else(|| {
                // No goblin left in view: a reserve steps in from behind him to take the blow —
                // squeezing in two tiles out when he stands boxed in (a Warlord cornered in a
                // corridor took four unaimed swings and died to `attack nearest`: TRIVIAL
                // passed D8).
                let free = |q: &Pos| run.floor.map.passable(*q) && !run.occupied(*q);
                let q = wp.neighbours8().into_iter().find(free).or_else(|| {
                    (-2..=2).flat_map(|dy| (-2..=2).map(move |dx| wp.step((dx, dy)))).find(|q| q.cheb(wp) == 2 && free(q) && run.floor.map.los(wp, *q))
                })?;
                let id = run.new_id();
                let depth = run.depth;
                let mut g = crate::endgame::spawn(run, id, "goblin", q, depth, false);
                g.awake = true;
                g.last_seen = Some(run.hero.pos);
                g.summoned = true;
                g.reserve = true;
                g.extra_tags.push("summoned".into());
                let e = crate::engine::monster_entity(&g, cx.facts);
                run.monsters.push(g);
                cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
                Some(run.monsters.len() - 1)
            });
            if run.floor.map.is_visible(wp) {
                callout(run, cx, "shielded");
            }
            match guard {
                Some(g) => mi = g,
                // Nowhere for a reserve to stand: the shield turns the blow aside.
                None => return false,
            }
        }
    }
    // Cut 3: `reflect_melee` — a melee blow (the hero's, or an ally's) lands on the attacker.
    if run.monsters[mi].reflects_melee() && dmg > 0 {
        let reflected = match src {
            Src::Hero { ranged: false } => run.monsters[mi].pos.cheb(run.hero.pos) <= 2,
            Src::Mon(j) => run.monsters[*j].ally,
            _ => false,
        };
        if reflected {
            let kind = run.monsters[mi].kind.clone();
            let id = run.monsters[mi].id;
            let visible = run.floor.map.is_visible(run.monsters[mi].pos);
            if visible {
                learn_tag(run, cx, &kind, "reflect_melee");
                callout(run, cx, "reflected!");
            }
            match src {
                Src::Hero { .. } => {
                    cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit: true, verb: Some("reflect".into()) });
                    damage_hero(run, cx, dmg, &Src::Reflect(mi));
                }
                Src::Mon(j) => {
                    let j = *j;
                    let jid = run.monsters[j].id;
                    cx.events.push(Ev::Attack { t: run.turn, src: id, dst: jid, dmg, hit: true, verb: Some("reflect".into()) });
                    damage_monster(run, cx, j, dmg, &Src::Reflect(mi));
                }
                _ => {}
            }
            return false;
        }
    }
    let cause = src.cause(run);
    let cause = cause.as_str();
    let mut dmg = dmg.max(0);
    // Cut 23 §1 (the forge's full kit broke the D28 wall without its counter: FULL−D28 kitted
    // passed on 16 of 30 seeds, the Queen dead to four blows before her second call): while a
    // lurker she called lives, her brood shields her — half of every blow. Silence (no calls)
    // or clearing the brood first is the answer.
    if dmg > 0 && run.monsters[mi].kind == "lurker_queen" && run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.summoned && m.kind == "lurker") {
        dmg = (dmg + 1) / 2;
    }
    let (dmg, counter) = {
        let m = &run.monsters[mi];
        counter_damage(run, src, dmg, Some(mi), m.pos, m.hostile())
    };
    if let Some((a, b)) = counter {
        if run.floor.map.is_visible(run.monsters[mi].pos) {
            learn(run, cx, crate::defs::counter_fact(&a, &b));
        }
    }
    run.monsters[mi].hurt_since_action = true;
    // QA on 23ed91f (qaL): blood on a summoned foe is no progress — a conjurer out of reach
    // sends two blades every 150 ticks, the hero cuts them down, and the guard never saw a
    // pacing (runs to the 120 000-tick cap, 3 000 blades killed).
    if matches!(src, Src::Hero { .. }) && dmg > 0 && !run.monsters[mi].summoned {
        run.last_damage_action = run.actions;
        run.stuck_until = 0;
        run.row_suppressed = (-9, 0);
    }
    // Cut 5 §4: a blow on a sleeper wakes the whole den. Cut 7 §3: a den thief caught napping
    // bolts alone.
    if run.monsters[mi].dormant {
        if run.monsters[mi].situation.as_deref() == Some("den") {
            crate::situations::raided(run, cx, mi);
        } else {
            wake_den(run, cx);
        }
    }
    // Cut 20 §2: a companion already fallen back (≤ 30 % hp) dies only cornered; the blow
    // that would kill it from above that is still a kill.
    let dmg = if dmg >= run.monsters[mi].hp && crate::ai::pet_wounded(&run.monsters[mi]) && !crate::ai::pet_cornered(run, mi) { run.monsters[mi].hp - 1 } else { dmg };
    run.monsters[mi].hp -= dmg;
    // Cut 24 §1: a real foe's HP moved (a summon's does not: reserves are endless), by any
    // hand but another foe's.
    if dmg > 0 && run.monsters[mi].hostile() && !matches!(src, Src::Mon(j) if !run.monsters[*j].ally) {
        if !run.monsters[mi].summoned {
            run.fight_moved();
        } else if !run.monsters[mi].reserve {
            // (a boss's own summons cut down — a Lich's skeletons, a rally's goblins — are its
            // fight moving: they are finite, or the counter itself; a reserve is a shrug, and
            // another foe's summons — a conjurer's blades — are not the boss's)
            let kind = run.monsters[mi].kind.clone();
            if let Some(b) = run.boss_still {
                if run.monsters.iter().find(|m| m.id == b.0).is_some_and(|m| crate::engine::boss_escort(&m.kind) == kind) {
                    run.boss_still = Some((b.0, b.1, 0));
                }
            }
        }
    }
    if run.monsters[mi].kind == "bloat_mother" && run.monsters[mi].hp > 0 && matches!(src, Src::Hero { ranged: false }) {
        let at = run.monsters[mi].pos;
        place_overlay(run, cx, at, 2, OverlayKind::Gas, 30);
        if run.floor.map.is_visible(at) {
            callout(run, cx, "vents!");
            learn_tag(run, cx, "bloat_mother", "gas");
        }
    }
    let (id, hp, kind, pos) = {
        let m = &run.monsters[mi];
        (m.id, m.hp, m.kind.clone(), m.pos)
    };
    cx.events.push(Ev::Hurt { t: run.turn, id, dmg, hp: hp.max(0), cause: cause.into() });
    // Cut 16 §4: the Warlord breaks at half hp, once.
    if kind == "goblin_warlord" && hp > 0 && hp * 2 <= run.monsters[mi].max_hp && !run.monsters[mi].broken {
        crate::ai::warlord_break(run, cx, mi);
    }
    let visible = run.floor.map.is_visible(pos);
    if hp > 0 {
        // Cut 3: an echo splits on a ranged hit (arrow, bolt, thrown potion), like a jelly on any.
        let echo = run.monsters[mi].has_tag("echo") && matches!(src, Src::Hero { ranged: true });
        if (run.monsters[mi].has_tag("splitter") || echo) && hp > 4 && dmg > 0 && !run.monsters[mi].ally {
            let half = hp / 2;
            run.monsters[mi].hp = hp - half;
            let free = pos.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
            if let Some(q) = free {
                let nid = run.new_id();
                let depth = run.depth;
                let mut child = crate::endgame::spawn_split(nid, &kind, q, depth, run.monsters[mi].modifiers);
                child.hp = half;
                child.max_hp = run.monsters[mi].max_hp;
                child.awake = true;
                child.last_seen = run.monsters[mi].last_seen;
                let e = crate::engine::monster_entity(&child, cx.facts);
                run.monsters.push(child);
                cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
                if visible {
                    callout(run, cx, "splits!");
                    learn_tag(run, cx, &kind, if echo { "echo" } else { "splitter" });
                }
            }
        }
        return false;
    }
    run.monsters[mi].hp = 0;
    cx.events.push(Ev::Die { t: run.turn, id, cause: cause.into() });
    let m = run.monsters[mi].clone();
    if crate::legacy::has(&run.hero,crate::legacy::RENEWAL)&&m.hostile()&&!m.summoned&&matches!(src,Src::Hero{..}) {
        run.hero.hp=(run.hero.hp+1).min(run.hero.max_hp);
    }
    if m.ally {
        run.ally_lost.push((run.turn, kind.clone()));
        cx.events.push(Ev::Ally { t: run.turn, id, state: "lost".into() });
        if let Some(cid) = m.cid {
            let name = run.companion(cid).map(|c| c.name.clone()).unwrap_or_else(|| kind.clone());
            run.lost_companions.push((run.turn, name.clone()));
            run.fell_why.push((name.clone(), format!("fell D{} to {}", run.depth, cause.replace('_', " "))));
            note(run, cx, format!("{name} the {} fell.", crate::engine::kind_title(&kind)));
            // Cut 10 §3: the callout uses the chronicle's verb (`Ashar slain` read as a foe).
            callout(run, cx, &format!("{name} fell"));
            // Cut 5 §1: a companion's fall closes the episode.
            run.arc.allies_lost.push(name.clone());
            sifter::resolve(run, Resolution::Fell { kind: kind.clone(), name });
        } else {
            run.arc.allies_lost.push(kind.clone());
            note(run, cx, format!("The {} fell.", crate::engine::kind_title(&kind)));
            callout(run, cx, &format!("{} fell", crate::engine::kind_title(&kind).to_lowercase()));
        }
    } else if m.neutral && m.situation.as_deref() == Some("captive") {
        // Cut 7 §3: the coward's way through the gate.
        let text = crate::chronicle::variant(run, "cut_captive");
        note(run, cx, text);
        callout(run, cx, "no friends");
        if !run.trophies_run.contains(&"no_friends".to_string()) {
            run.trophies_run.push("no_friends".into());
        }
    } else if !m.neutral && m.summoned {
        // QA on 23ed91f (qaL): a summoned foe — a conjurer's blade, a rallied goblin, a lich's
        // skeleton — is no kill for the lineage: no XP, no renown, no bestiary or `studied`
        // count (a conjurer out of reach paid `fighter +598 · renown +1054` for one run). The
        // rows still see it (`on_kill`, the floor's count).
        run.kills_floor += 1;
        run.kill_since_action = true;
        run.summoned_kills += 1;
    } else if !m.neutral {
        let depth = run.depth;
        run.kills.push((run.turn, kind.clone(), depth));
        run.kills_floor += 1;
        run.kill_since_action = true;
        crate::facts::on_kill(run, cx, &kind);
        if m.is_boss() {
            // Cut 24 §1: with the boss down his summons are no endless reserve — their blood is
            // the fight moving again (the guards read it: a Warlord slain, then three stalls
            // against his last goblins, their blows `no progress`).
            let escort = crate::engine::boss_escort(&kind);
            for o in run.monsters.iter_mut().filter(|o| o.hp > 0 && o.summoned && o.kind == escort) {
                o.summoned = false;
                o.reserve = false;
            }
            run.boss_kills.push((run.turn, kind.clone()));
            note(run, cx, format!("Slew the {}.", m.title()));
            crate::oath::beat(run, cx, run.acting_row);   // Cut 28b: a `slay` / `fire` oath is kept here
            callout(run, cx, "boss down");
            // Cut 5 §1: a boss dying closes the episode (`first boss` the first time the
            // lineage kills the kind).
            let first = cx.kill_counts.get(&kind).copied().unwrap_or(0) <= 1;
            let res = if first { Resolution::FirstBoss { kind: kind.clone() } } else { Resolution::BossSlain { kind: kind.clone() } };
            sifter::resolve(run, res);
            if !run.hurt_since_boss && !run.trophies_run.contains(&"boss_untouched".to_string()) {
                run.trophies_run.push("boss_untouched".into());
                if !cx.trophies.iter().any(|t| t == "boss_untouched") { note(run, cx, "Trophy: boss untouched.".into()); }
            }
        } else if m.grudge {
            // Cut 19 §5 (rater AA: `Zeleth the goblin archer is avenged.` twice): avenged once;
            // a later kill of the same named foe is `slain`.
            let name = m.name.clone().unwrap_or_default();
            if m.avenged || run.avenged.contains(&name) {
                note(run, cx, format!("{} slain.", m.title()));
            } else {
                note(run, cx, format!("{} is avenged.", m.title()));
                run.avenged.push(name);
            }
        }
    }
    if let Some(it) = m.stolen.clone() {
        // Cut 20 §1: a killed thief drops what it stole where it fell; picked up, it is
        // `Got the heal back.` (`pickup_here`).
        run.items.push(crate::engine::FloorItem { pos, item: it });
    }
    if m.has_tag("gas") {
        // Cut 7 §3: a lock bloat's cloud is quick — it bursts hard (a hero beside it takes the
        // burst at once) and clears in two turns.
        let lock = m.situation.as_deref() == Some("lock");
        let (r, ttl) = if m.is_boss() { (2, 60) } else if lock { (1, 20) } else { (1, 40) };
        place_overlay(run, cx, pos, r, OverlayKind::Gas, ttl);
        if visible {
            callout(run, cx, "pops!");
            learn_tag(run, cx, &kind, "gas");
        }
        if lock {
            run.lock_last_pop = run.turn;
            if run.hero.pos.cheb(pos) <= 1 && run.over.is_none() {
                run.gas_dmg_floor += 3;
                damage_hero(run, cx, 3, &Src::Burst);
            }
        }
    }
    true
}

pub fn place_overlay(run: &mut Run, cx: &mut Ctx, centre: Pos, r: i32, k: OverlayKind, ttl: i32) {
    for dy in -r..=r {
        for dx in -r..=r {
            let p = centre.step((dx, dy));
            if !run.floor.map.in_bounds(p) || !run.floor.map.passable(p) {
                continue;
            }
            if let Some(o) = run.overlays.iter_mut().find(|o| o.x == p.x && o.y == p.y) {
                let changed = o.k != k;
                o.k = k;
                o.ttl = o.ttl.max(ttl);
                if changed {
                    let ttl = o.ttl;
                    cx.events.push(Ev::Overlay { t: run.turn, x: p.x, y: p.y, ov: k, ttl });
                }
                continue;
            }
            run.overlays.push(Overlay { x: p.x, y: p.y, k, ttl, spread: k == OverlayKind::Fire });
            cx.events.push(Ev::Overlay { t: run.turn, x: p.x, y: p.y, ov: k, ttl });
        }
    }
}

/// Every 10 ticks: hazards bite, fire spreads once, overlays age.
fn tick_overlays(run: &mut Run, cx: &mut Ctx) {
    let overlays = run.overlays.clone();
    for o in &overlays {
        let p = Pos::new(o.x, o.y);
        let (dmg, src) = match o.k {
            OverlayKind::Gas => (3, Src::Gas),
            OverlayKind::Fire => (5, Src::Fire),
        };
        if run.hero.pos == p {
            // Cut 7 §3: gas that is the lock's — while its bloats stand, or just after a burst.
            let lock_live = run.monsters.iter().any(|m| m.hp > 0 && m.situation.as_deref() == Some("lock"));
            if src == Src::Gas && (lock_live || run.turn <= run.lock_last_pop + 40) {
                run.gas_dmg_floor += dmg;
            }
            damage_hero(run, cx, dmg, &src);
            if run.over.is_some() {
                return;
            }
        }
        if let Some(mi) = run.monster_at(p) {
            if src == Src::Gas && run.monsters[mi].has_tag("gas") {
                // Gas creatures breathe it; the Bloat Mother heals in it.
                if run.monsters[mi].kind == "bloat_mother" {
                    let m = &mut run.monsters[mi];
                    m.hp = (m.hp + 3).min(m.max_hp);
                }
                continue;
            }
            damage_monster(run, cx, mi, dmg, &src);
        }
    }
    let spreading: Vec<Overlay> = run.overlays.iter().filter(|o| o.spread && o.k == OverlayKind::Fire).cloned().collect();
    for o in spreading {
        for d in DIRS8 {
            let q = Pos::new(o.x, o.y).step(d);
            if run.floor.map.get(q) == Tile::Floor && !run.overlays.iter().any(|x| x.x == q.x && x.y == q.y) {
                let ttl = (o.ttl - TICKS_PER_TURN as i32).max(TICKS_PER_TURN as i32);
                run.overlays.push(Overlay { x: q.x, y: q.y, k: OverlayKind::Fire, ttl, spread: false });
                cx.events.push(Ev::Overlay { t: run.turn, x: q.x, y: q.y, ov: OverlayKind::Fire, ttl });
            }
        }
    }
    for o in run.overlays.iter_mut() {
        o.spread = false;
        o.ttl -= TICKS_PER_TURN as i32;
    }
    run.overlays.retain(|o| o.ttl > 0);
}

/// Cut 25 §3: the blows a death trace itemises after the last action (`Run.blows`).
pub const BLOWS_CAP: usize = 24;

/// Cut 25 §3: a drain bite (a hunger bite, poison) — with no foe in view, the stretch's first
/// sends `Ev::Drain` with its word (`starving`, `poisoned`); once per stretch (`Run.drain_on`,
/// cleared when a foe comes into view and at the stairs).
pub fn drain_mark(run: &mut Run, cx: &mut Ctx, cause: &str) {
    if seen_foes(run, None) > 0 {
        return;
    }
    let word = match cause {
        "hunger" => "starving",
        "poison" => "poisoned",
        _ => "drained",
    };
    if run.drain_on.as_deref() == Some(word) {
        return;
    }
    run.drain_on = Some(word.into());
    cx.events.push(Ev::Drain { t: run.turn, cause: word.into() });
}

fn tick_poison(run: &mut Run, cx: &mut Ctx) {
    if run.hero.poison.1 > 0 {
        let d = run.hero.poison.0;
        if d > 0 {
            drain_mark(run, cx, "poison");
        }
        damage_hero(run, cx, d, &Src::Poison);
        if run.over.is_some() {
            return;
        }
    }
    for mi in 0..run.monsters.len() {
        if run.monsters[mi].poison.1 > 0 && run.monsters[mi].hp > 0 {
            let d = run.monsters[mi].poison.0;
            damage_monster(run, cx, mi, d, &Src::Poison);
        }
    }
}

/// Cut 3, every 10 ticks: `regen` monsters heal 2 unless poisoned; the hero's regen potion
/// heals 2; a siren's aura (2 tiles) confuses the hero unless clarity holds.
fn tick_regen_and_auras(run: &mut Run, cx: &mut Ctx) {
    if run.hero.regen_t > 0 && run.hero.hp < run.hero.max_hp {
        run.hero.hp = (run.hero.hp + 2).min(run.hero.max_hp);
    }
    // The Lurker Queen mends 2 while any lurker she called still lives.
    let called = run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.summoned && m.kind == "lurker");
    if called {
        for m in run.monsters.iter_mut().filter(|m| m.kind == "lurker_queen" && m.hp > 0 && m.hp < m.max_hp) {
            m.hp = (m.hp + 2).min(m.max_hp);
        }
    }
    let hp = run.hero.pos;
    let mut aura = false;
    for mi in 0..run.monsters.len() {
        let m = &run.monsters[mi];
        if m.hp <= 0 {
            continue;
        }
        if m.has_tag("regen") && m.poison.1 == 0 && m.hp < m.max_hp && m.hostile() {
            let visible = run.floor.map.is_visible(m.pos);
            let kind = m.kind.clone();
            let m = &mut run.monsters[mi];
            m.hp = (m.hp + 2).min(m.max_hp);
            if visible {
                learn_tag(run, cx, &kind, "regen");
            }
        }
        // Dungeon regeneration is instance metadata, not a permanent kind fact.
        if run.difficulty > 0 {
            let m = &mut run.monsters[mi];
            if m.modifiers.is_some_and(|mods| mods.has(crate::endgame::REGENERATING))
                && !m.has_tag("regen") && m.awake && m.hostile() && m.poison.1 == 0 && m.hp < m.max_hp {
                m.hp = (m.hp + 1).min(m.max_hp);
            }
        }
        let m = &run.monsters[mi];
        if m.has_tag("aura") && m.hostile() && m.awake && m.pos.cheb(hp) <= 2 && run.floor.map.los(m.pos, hp) {
            aura = true;
            if run.floor.map.is_visible(m.pos) {
                let kind = m.kind.clone();
                learn_tag(run, cx, &kind, "aura");
            }
        }
    }
    if aura && run.hero.clarity_t == 0 {
        if run.hero.confused == 0 {
            callout(run, cx, "confused");
        }
        run.hero.confused = run.hero.confused.max(12);
    }
}

/// Cut 3: a noise at `at` heard `radius` tiles around. Blind hunters go there; the Lurker
/// Queen calls her lurkers. Silence (the scroll) swallows every noise near the hero.
pub fn noise(run: &mut Run, cx: &mut Ctx, at: Pos, radius: i32) {
    if run.hero.silence_t > 0 {
        return;
    }
    run.noise = Some((at, run.turn));
    // The Queen hears twelve tiles around and keeps up to eight called lurkers in the hunt
    // (they fade after 150 ticks: silence lets the storm pass). Cut 23 §1: eight (was six) —
    // the kitted FULL−D28 outlasted six.
    let called = run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && m.summoned && m.kind == "lurker").count();
    const QUEEN_PACK: usize = 8;
    for mi in 0..run.monsters.len() {
        let m = &run.monsters[mi];
        let queen = m.kind == "lurker_queen";
        let hears = if queen { 12 } else { radius };
        if m.hp <= 0 || !m.hostile() || !m.is_blind() || m.pos.cheb(at) > hears {
            continue;
        }
        let m = &mut run.monsters[mi];
        m.awake = true;
        m.last_seen = Some(at);
        if queen && m.cooldown == 0 && m.pending.is_none() && called < QUEEN_PACK {
            m.cooldown = 15;
            ai::telegraph(run, cx, mi, "listens", crate::monster::Pending::Call);
        }
    }
}

fn tick_statuses(run: &mut Run, cx: &mut Ctx) {
    if run.hero.poison.1 > 0 {
        run.hero.poison.1 -= 1;
    }
    run.hero.tick_statuses();
    if run.taunt_t > 0 {
        run.taunt_t -= 1;
    }
    let t = run.turn;
    for m in run.monsters.iter_mut() {
        if m.poison.1 > 0 {
            m.poison.1 -= 1;
        }
        m.tick_statuses();
        if m.ttl.is_some_and(|t| t <= 0) && m.hp > 0 {
            m.hp = 0;
            if m.ally {
                cx.events.push(Ev::Ally { t, id: m.id, state: "lost".into() });
            }
            cx.events.push(Ev::Die { t, id: m.id, cause: "faded".into() });
        }
    }
}

/// Rests per alert step: resting is not free. From alert 5 a wandering pack comes for the
/// hero (spawned out of sight, nearby).
pub const REST_ALERT_EVERY: u32 = 8;

/// Resting raises the alert every `REST_ALERT_EVERY` rests; from alert 5 a pack comes.
/// Cut 3: every rest is a noise (radius 8) — the Deep's hunters come for it.
pub fn rest_clock(run: &mut Run, cx: &mut Ctx) {
    run.rests += 1;
    run.rested = true;
    crate::oath::beat(run, cx, run.acting_row);   // Cut 28b: a `no rest` oath breaks here
    let at = run.hero.pos;
    noise(run, cx, at, 8);
    if !run.rests.is_multiple_of(REST_ALERT_EVERY) {
        return;
    }
    run.alert = (run.alert + 1).min(8);
    if run.alert < 5 {
        return;
    }
    let table = crate::defs::spawn_table(run.biome(), run.depth);
    let packs: Vec<(&str, u32, i32, i32)> = table.iter().copied().filter(|t| t.2 >= 2).collect();
    let pool = if packs.is_empty() { table.clone() } else { packs };
    let weights: Vec<u32> = pool.iter().map(|t| if t.0 == "captive" || t.0 == "eel" { 0 } else { t.1 }).collect();
    let (kind, _, gmin, gmax) = pool[run.rng.weighted(&weights)];
    let hero = run.hero.pos;
    let cands: Vec<Pos> = run
        .floor
        .open_tiles()
        .into_iter()
        .filter(|p| !run.floor.map.is_visible(*p) && (3..=8).contains(&p.cheb(hero)) && !run.occupied(*p))
        .collect();
    if cands.is_empty() {
        return;
    }
    let anchor = *run.rng.pick(&cands);
    let n = run.rng.range(gmin.max(2), gmax.max(2));
    for k in 0..n {
        let pos = if k == 0 {
            anchor
        } else {
            match anchor.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q)) {
                Some(q) => q,
                None => continue,
            }
        };
        let id = run.new_id();
        let depth = run.depth;
        let mut m = crate::endgame::spawn(run, id, kind, pos, depth, true);
        m.awake = true;
        m.last_seen = Some(hero);
        let e = crate::engine::monster_entity(&m, cx.facts);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
    }
    callout(run, cx, "they heard you");
}

/// The forward clock: every `ALERT_EVERY` ticks on a floor the alert rises and wanderers
/// arrive, more of them as the alert climbs (1 + alert/4). Lingering policies pay for it.
pub const ALERT_EVERY: u32 = 800;

fn tick_alert(run: &mut Run, cx: &mut Ctx) {
    if run.floor_turn == 0 || !run.floor_turn.is_multiple_of(ALERT_EVERY) || run.alert >= 8 {
        return;
    }
    run.alert += 1;
    let table = crate::defs::spawn_table(run.biome(), run.depth);
    let weights: Vec<u32> = table.iter().map(|t| if t.0 == "captive" || t.0 == "eel" { 0 } else { t.1 }).collect();
    let hero = run.hero.pos;
    let n = 1 + run.alert / 4;
    for _ in 0..n {
        let (kind, ..) = table[run.rng.weighted(&weights)];
        let cands: Vec<Pos> = run
            .floor
            .open_tiles()
            .into_iter()
            .filter(|p| !run.floor.map.is_visible(*p) && p.cheb(hero) >= 6 && !run.occupied(*p))
            .collect();
        if cands.is_empty() {
            return;
        }
        let pos = *run.rng.pick(&cands);
        let id = run.new_id();
        let depth = run.depth;
        let mut m = crate::endgame::spawn(run, id, kind, pos, depth, true);
        m.awake = true;
        m.last_seen = Some(hero);
        let e = crate::engine::monster_entity(&m, cx.facts);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
    }
    if run.alert == 3 || run.alert == 6 {
        callout(run, cx, "alert rising");
        learn(run, cx, "alert:rising".into());
    }
}

/// Go down a floor (or reach the ending).
/// Cut 22 §3: the tag of a floor's own random stream (`descend`).
pub const FLOOR_STREAM: u64 = 0xF100_2201_0000_0000;

pub fn descend(run: &mut Run, cx: &mut Ctx) {
    note_saved(run, cx);
    // Cut 5 §1: the floor changing after a low resolves the episode; §4: an open vault is
    // settled by the preference before the stairs.
    if run.vault_choice.is_some() {
        vault_take(run, cx, None);
    }
    sifter::resolve(run, Resolution::Reached { depth: run.depth + 1 });
    if let Some(t) = run.low10_t.take() {
        run.near_deaths.push(t);
    }
    run.low20_t = None;
    run.saved_by = None;
    run.nohp.0 = 0;
    run.boss_still = None;
    let floor_gambles: Vec<(u32, String, bool)> = run.gambles.clone();
    for (t, k, mal) in floor_gambles {
        if mal && !run.gambles_survived.iter().any(|(gt, _)| *gt == t) {
            run.gambles_survived.push((t, k));
        }
    }
    if run.kills_floor == 0 && run.depth >= 2 && !run.trophies_run.contains(&"pacifist_floor".to_string()) {
        run.trophies_run.push("pacifist_floor".into());
        if !cx.trophies.iter().any(|t| t == "pacifist_floor") { note(run, cx, "Trophy: a floor passed without a kill.".into()); }
    }
    // Cut 7 §3: the band's situation is judged as the floor is left.
    crate::situations::on_leave_floor(run, cx);
    // QA on 524827b (qaAA: `attack thief · not in view ← monkey slain D1` a floor under the den
    // that woke thieves round him on D4): the hostiles in view as he takes the stairs are left
    // behind — each one's `last seen D4`, as one stepping out of view is (`facts::on_vision`).
    if !cx.sim {
        let map = &run.floor.map;
        let mut left: Vec<String> = run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && map.is_visible(m.pos)).map(|m| m.kind.clone()).collect();
        left.dedup();
        for kind in left {
            crate::provenance::seen(run, cx, &kind, 0, 0);
        }
    }
    let next = run.depth + 1;
    // Cut 3: chalk in the pack marks the floor left behind (`chalk:<depth>`): the next heir here
    // goes straight for the stairs.
    if let Some(i) = run.hero.inv.iter().position(|i| i.kind == "chalk") {
        let it = &mut run.hero.inv[i];
        it.amount -= 1;
        let gone = it.amount <= 0;
        if gone {
            run.hero.inv.remove(i);
        }
        let d = run.depth;
        if gone {
            crate::provenance::spent(run, cx, "chalk", format!("chalk marked D{d}"));
        }
        learn(run, cx, format!("chalk:{d}"));
    }
    if next >= ENDING_DEPTH {
        run.depth = next;
        run.max_depth = run.max_depth.max(next);
        run.ended = true;
        cx.events.push(Ev::Descend { t: run.turn, depth: next, biome: "bottom".into() });
        note(run, cx, "The bottom. Nothing below.".into());
        callout(run, cx, "the bottom");
        end_run(run, cx, ExitTier::Bank);
        return;
    }
    let biome = run.route.biome(next);
    // Cut 22 §3 (AH: "most edits moved the forecast less than its ±10–13 error"): in a
    // forecast's sim, each floor draws from a stream of its own — (run seed, depth) — so two
    // sims on one seed walk into the same floors whatever their rules did above (common random
    // numbers: the camp's paired edit delta is a difference of the same dungeons, not of two
    // draws). The same distribution as a send's, whose floors (and replays) go on drawing from
    // the run's stream, unchanged (`Run.floor_streams`).
    if run.floor_streams {
        run.rng = crate::rng::Rng::derive(run.seed, FLOOR_STREAM ^ next as u64);
    }
    let floor = generate(&mut run.rng, biome, next);
    run.depth = next;
    if next > run.max_depth {
        run.depth_t.push((next, run.turn));
    }
    run.max_depth = run.max_depth.max(next);
    // Cut 30.5 (the owner, 2026-10-02): a new record is a checkpoint, not an exit — the carry so far is secured
    // (safe whatever the exit) and the hero carries on (the watch stamps its `NEW BEST D5` from `Run.best_at_send`)
    if next > run.record_mark {
        run.record_mark = next;
        run.secured += run.loot.max(0);
        run.loot = 0;
    }
    run.floor = floor;
    run.hero.pos = run.floor.stairs_up;
    run.hero_dist_pos = None;
    run.monsters.retain(|m| m.ally && m.hp > 0);
    let up = run.floor.stairs_up;
    let allies = std::mem::take(&mut run.monsters);
    for mut m in allies {
        // Cut 20 §2: a companion comes down the stairs at ≥ 60 % of its hp, back in the fight.
        if m.is_companion() {
            if let Some(c) = m.cid.and_then(|cid| run.companion(cid)) {
                let max = crate::engine::pet_max_hp(c, next);
                if max > m.max_hp {
                    m.hp += max - m.max_hp;
                    m.max_hp = max;
                }
            }
            m.hp = m.hp.max((m.max_hp * crate::ai::PET_DESCENT_PCT + 99) / 100).min(m.max_hp);
            m.fleeing = false;
        }
        let free = up.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q) && *q != up);
        m.pos = free.unwrap_or(up);
        run.monsters.push(m);
    }
    run.items.clear();
    run.overlays.clear();
    run.floor_turn = 0;
    run.alert = 0;
    run.kills_floor = 0;
    run.boss_seen_t = None;
    run.hurt_since_boss = false;
    run.seen_ids.clear();
    run.last_visible = vec![u32::MAX];
    run.ignored.clear();
    run.known_foes.clear();
    run.hunt = None;
    run.last_target = None;
    run.rests = 0;
    run.chase = None;
    run.recent_pos.clear();
    run.stuck_until = 0;
    run.stuck_fires = 0;
    run.stuck_first_t = None;
    run.stuck_cause = None;
    run.stuck_row = None;
    run.freeing = None;
    run.card_fell = None;
    run.trait_floor = 0;
    run.items_until = 0;
    run.pickup_streak = 0;
    run.skip_items.clear();
    run.pickup_dry = 0;
    run.drain_on = None;
    run.gambles.clear();
    run.own_throw = None;
    run.retreats = (0, 0);
    run.hero.second_wind_used = false;
    // Cut 3: drained max HP (wraiths, the Lich) comes back a floor at a time — a floor's debt,
    // not the run's (a 30-floor descent would otherwise arrive in the Foundry at 19 HP).
    run.hero.max_hp = (run.hero.max_hp + 5).min(run.hero.max_hp_base);
    run.noise = None;
    run.verb_ring.clear();
    run.blind_seen.clear();
    run.vault_cage.clear();
    run.dens.clear();
    run.tempted = false;
    populate_floor(run, cx.grudges, cx.forge, cx.hunter);
    crate::engine::place_situations(run, cx.lost);
    crate::engine::place_bones(run);
    if cx.facts.contains(&format!("chalk:{next}")) {
        let s = run.floor.stairs_down;
        let i = run.floor.map.idx(s);
        run.floor.map.seen[i] = true;
    }
    let vision = run.vision(cx.unlocks);
    run.floor.map.update_vision(run.hero.pos, vision);
    cx.events.push(Ev::Descend { t: run.turn, depth: next, biome: biome.name().into() });
    if run.route.biome(next - 1) != biome {
        learn(run, cx, format!("biome:{}", biome.name()));
    }
    note(run, cx, format!("D{}: {}.", next, biome.title()));
    // Cut 24 §2: the biome's arrival event, on a third of its floors.
    crate::situations::omen(run, cx);
    if next == 5 {
        if !run.drank_heal && !run.trophies_run.contains(&"no_heal_D5".to_string()) {
            run.trophies_run.push("no_heal_D5".into());
            if !cx.trophies.iter().any(|t| t == "no_heal_D5") { note(run, cx, "Trophy: no heal to D5.".into()); }
        }
        if !run.melee_used && !run.trophies_run.contains(&"ranged_only_D5".to_string()) {
            run.trophies_run.push("ranged_only_D5".into());
            if !cx.trophies.iter().any(|t| t == "ranged_only_D5") { note(run, cx, "Trophy: ranged only to D5.".into()); }
        }
    }
    crate::facts::on_vision(run, cx);
}

/// Cut 7 §4: `Ev::Ending { ticks }` before an exit the engine can foresee — a bank walk-out or
/// the bottom's stairs within three steps (~30 ticks at base speed), or death in the air
/// (hp ≤ 15% with a hostile adjacent). Once per 100 ticks; a run that has already ended this
/// action (`bail`, recall, the bottom reached) gets a `0` so the viewer knows it was instant.
fn foresee_ending(run: &mut Run, cx: &mut Ctx, verb: &Verb, v: &View) {
    let t = run.turn;
    if run.ending_t.is_some_and(|e| t < e + 100) {
        return;
    }
    // Chebyshev distance bounds the path length from below: no BFS until the goal is close.
    let steps_to = |run: &Run, goal: Pos| -> i32 {
        if run.hero.pos.cheb(goal) > 3 {
            return i32::MAX;
        }
        let d = run.floor.map.bfs(run.hero.pos, false, &|_| false);
        d[run.floor.map.idx(goal)]
    };
    let dying = run.hero.hp_pct() <= 15 && v.adj >= 1;
    // Cut 19 §2: a return walks to the up-stairs as a bank does.
    // Cut 27 §3: a return is out where he stands once its walk is spent.
    let banking = (matches!(verb.v.as_str(), "bank" | "return") && (0..=3).contains(&steps_to(run, run.floor.stairs_up))) || return_left(run).is_some_and(|l| l <= 30);
    let bottom = verb.v == "descend" && run.depth + 1 >= ENDING_DEPTH && (0..=3).contains(&steps_to(run, run.floor.stairs_down));
    let ticks = if run.over.is_some() {
        Some(0)
    } else if dying || banking || bottom {
        Some(30)
    } else {
        None
    };
    if let Some(ticks) = ticks {
        run.ending_t = Some(t);
        cx.events.push(Ev::Ending { t, ticks });
    }
}

/// Cut 4: the hero fell to ≤ 20 % on this floor and lived to leave it — the chronicle names
/// the row that saved him (`R3 rest saved him`). Cut 15 §6: `saved`, not `caught` — the note
/// is only written when he lived (a descend or a home exit), and `caught him` on a run he
/// survived read as the row's fault (V).
fn note_saved(run: &mut Run, cx: &mut Ctx) {
    if run.low20_t.is_none() && run.low10_t.is_none() {
        return;
    }
    if let Some(r) = run.saved_by.take() {
        if let Some(row) = cx.rules.rows.get(r as usize) {
            let who = format!("R{} {}", r + 1, row.verb.short());
            let tpl = crate::chronicle::variant(run, "saved");
            let text = if tpl.is_empty() { format!("{who} saved him.") } else { tpl.replace("{r}", &who) };
            note(run, cx, text);
        }
    }
}

pub fn end_run(run: &mut Run, cx: &mut Ctx, tier: ExitTier) {
    if run.over.is_some() {
        return;
    }
    if tier != ExitTier::Death {
        note_saved(run, cx);
    }
    run.over = Some(tier);
    // Cut 28b: the oath's fate, if the run has not said it yet — before the exit's own event
    let row = if run.acting_row >= 0 { run.acting_row } else { run.exit_row.or(run.homeward).unwrap_or(-1) };
    crate::oath::beat(run, cx, row);
    let loot_kept = run.kept(tier);
    // Cut 5 §1: the exit resolves every open episode.
    let res = match tier {
        ExitTier::Bank => Resolution::Banked { gold: loot_kept },
        // Cut 13 §1: a stall names its cause (the reel line reads what the trace does).
        ExitTier::Return if run.timed_out && run.stuck_fires >= crate::engine::STALL_FIRES => Resolution::Stalled { cause: run.stuck_cause.clone().unwrap_or_else(|| "paced".into()) },
        ExitTier::Return if run.timed_out => Resolution::Lost { stalled: false },
        // Cut 24 §1: a boss that could not be hurt drove him off.
        ExitTier::Return if run.driven_off.is_some() => Resolution::DrivenOff { kind: run.driven_off.clone().unwrap_or_default() },
        ExitTier::Return => Resolution::Returned { gold: loot_kept },
        ExitTier::Death => Resolution::Died { cause: run.death_cause.clone().unwrap_or_else(|| "unknown".into()) },
    };
    sifter::resolve(run, res);
    cx.events.push(Ev::Exit { t: run.turn, tier: tier.name().into(), loot_kept, line: None, trace: None });
    match tier {
        ExitTier::Bank => note(run, cx, format!("Banked ${loot_kept}.")),
        // QA on 308f045 (qaAC: the reel's `Returned with $0.` under `0 RETURNED · 1 DRIVEN`): a drive-off and a
        // stall end on their own note (`Driven off D8 by the Warlord.`, `Stalled: …`) — a drive-off that kept
        // its share says what it brought home, never that it returned.
        ExitTier::Return if run.timed_out => {}
        ExitTier::Return if run.driven_off.is_some() => {
            if loot_kept > 0 {
                note(run, cx, format!("Came home with ${loot_kept}."));
            }
        }
        ExitTier::Return => note(run, cx, format!("Returned with ${loot_kept}.")),
        ExitTier::Death => {}
    }
}

// ---------------------------------------------------------------- Cut 5 §4 situations

/// Situations in view become facts (`shrine`, `vault`, `nest`, `stray`) and count for the run.
fn situations_seen(run: &mut Run, cx: &mut Ctx) {
    crate::situations::seen(run, cx);
    if run.depth > 5 {
        return;
    }
    let map = &run.floor.map;
    let mut seen: Vec<&str> = Vec::new();
    let mut look = |i: usize| {
        let k = match map.tiles[i] {
            Tile::Shrine => "shrine",
            Tile::Vault | Tile::VaultOpen => "vault",
            Tile::Nest => "nest",
            _ => return,
        };
        if !seen.contains(&k) {
            seen.push(k);
        }
    };
    match map.visible_rows() {
        // The visible tiles in index order: those of the square the vision last ran on, row by row
        // (none outside it is visible).
        Some(rows) => {
            for i in rows.flatten() {
                if map.visible[i] {
                    look(i);
                }
            }
        }
        None => {
            // … else 32 at a time: a block with none in view (most of the map) is one branch-free
            // OR, not 32 tests.
            const BLOCK: usize = 32;
            for (b, block) in map.visible.chunks(BLOCK).enumerate() {
                if !block.iter().fold(false, |a, v| a | v) {
                    continue;
                }
                for (j, _) in block.iter().enumerate().filter(|(_, v)| **v) {
                    look(b * BLOCK + j);
                }
            }
        }
    }
    let stray = run.monsters.iter().find(|m| m.stray && m.hp > 0 && map.is_visible(m.pos)).map(|m| (m.name.clone().unwrap_or_default(), m.kind.clone()));
    for k in seen {
        if run.met_situation(k) {
            // Cut 24 §2: each drawn from its pool (`chronicle::variant`).
            let note_text = crate::chronicle::variant(run, k);
            note(run, cx, note_text);
            learn(run, cx, k.into());
        }
    }
    if let Some((name, kind)) = stray {
        if run.met_situation("stray") {
            // Cut 27 §5 (AT: "is that my hatched jackal?"): the line says how it was lost.
            let why = cx.lost.iter().rev().find(|l| l.name == name).map(|l| l.why.clone()).filter(|w| !w.is_empty());
            match why {
                // (a note is ≤ 8 words: the name is the player's own pet's)
                Some(w) => note(run, cx, format!("{name}, gone wild: {w}.")),
                None => note(run, cx, format!("{name} the {}, gone wild.", crate::engine::kind_title(&kind))),
            }
            learn(run, cx, "stray".into());
        }
    }
}

/// The den wakes when the hero steps within two tiles of a sleeper.
fn wake_nest(run: &mut Run, cx: &mut Ctx) {
    let hp = run.hero.pos;
    let near = run.monsters.iter().any(|m| m.nest && m.dormant && m.hp > 0 && m.pos.cheb(hp) <= 2);
    if near {
        wake_den(run, cx);
    }
}

/// Every sleeper of the den on this floor wakes (the hero's step, or a blow on one of them).
pub fn wake_den(run: &mut Run, cx: &mut Ctx) {
    let hp = run.hero.pos;
    // They stir one after another (the first bites at once, the rest a turn apart): a den is
    // a fight that builds, not a wall that lands — a worn hero still gets to answer it.
    let mut n = 0;
    for m in run.monsters.iter_mut().filter(|m| m.nest && m.dormant && m.hp > 0) {
        m.dormant = false;
        m.awake = true;
        m.last_seen = Some(hp);
        m.stun = n * TICKS_PER_TURN as i32;
        n += 1;
    }
    if n > 0 {
        run.met_situation("nest");
        note(run, cx, "The nest wakes.".into());
        callout(run, cx, "the nest wakes");
        learn(run, cx, "nest".into());
    }
}

/// The hero stands on the vault: the cage opens. Watched, the choice waits for `choose`
/// (`Snapshot.vault_choice`); otherwise the preference picks on the next tick.
fn vault_open(run: &mut Run, cx: &mut Ctx) {
    let here = run.hero.pos;
    run.floor.map.set(here, Tile::VaultOpen);
    let cage = std::mem::take(&mut run.vault_cage);
    if cage.is_empty() {
        return;
    }
    run.met_situation("vault");
    learn(run, cx, "vault".into());
    run.vault_choice = Some((run.turn, cage));
    note(run, cx, "The cage opens: one is his.".into());
    callout(run, cx, "choose one");
}

/// The cage item the vault preference takes (its category's first, else the first) — also the
/// snapshot's `VaultChoice.pick` (Cut 19 §1).
pub fn vault_pick(items: &[crate::item::Item], pref: &str) -> usize {
    let pref_cat = match pref {
        "armour" => Cat::Armour,
        "potion" => Cat::Potion,
        "scroll" => Cat::Scroll,
        _ => Cat::Weapon,
    };
    items.iter().position(|i| i.cat() == pref_cat).unwrap_or(0)
}

/// Take one item from the opened vault (`id`, or the preference's pick); the rest vanish.
pub fn vault_take(run: &mut Run, cx: &mut Ctx, id: Option<u32>) {
    let Some((_, items)) = run.vault_choice.take() else { return };
    let pick = id.and_then(|id| items.iter().position(|i| i.id == id)).unwrap_or_else(|| vault_pick(&items, cx.vault_pref));
    let Some(it) = items.into_iter().nth(pick) else { return };
    run.caged.push(it.id);
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    let here = run.hero.pos;
    run.loot_add(it.value());
    crate::provenance::found(run, cx, &it.kind, &label);
    // QA on 778fa1b (qaV: `took leather +1` from the cage with the pack full — in no report
    // line): the cage's pick is a find; put down for want of room, it is `left` there.
    run.note_found(it.id, &it.kind, it.amount.max(1));
    let replaced = if run.hero.inv_full() && !item_replaces_gear(&run.hero, &it) {
        run.note_gone(it.id, &it.kind, "left", 1);
        drop_near(run, here, it);
        None
    } else {
        run.hero.auto_equip(it)
    };
    if let Some(old) = replaced {
        if !run.hero.inv.iter().any(|i| i.id == old.id && i.kind == old.kind) {
            if run.hero.inv_full() {
                run.note_gone(old.id, &old.kind, "left", 1);
                drop_near(run, here, old);
            } else {
                run.hero.inv.push(old);
            }
        }
    }
    cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label.clone() });
    note(run, cx, format!("Took the {label} from the cage{}", if label.ends_with('?') { "" } else { "." }));
    sifter::open_situation(run, crate::sifter::Setup::Vault, "vault", &label, "");
}

/// `pray`: at the shrine, a fifth of max HP for the run buys a row (lent from the player's
/// other saved sets: the first row there the active set lacks) or a trait swap; `pray trait`
/// always swaps, `pray row` swaps when no row can be lent.
pub fn pray(run: &mut Run, cx: &mut Ctx, want_row: bool) {
    // Cut 7 §3: the hunger's shrine is lit, not bargained with.
    if crate::situations::hunger_floor(run) && !run.lit {
        crate::situations::light_shrine(run, cx);
        return;
    }
    run.prayed = true;
    let cost = run.hero.max_hp * crate::engine::PRAY_COST_PCT / 100;
    let before = run.hero.max_hp;
    run.hero.max_hp = (run.hero.max_hp - cost).max(1);
    run.hero.max_hp_base = (run.hero.max_hp_base - cost).max(1);
    run.hero.hp = run.hero.hp.min(run.hero.max_hp);
    // QA on 308f045 (qaAC): the shrine's price moves the HUD's max at its tick, named.
    crate::turn::hero_max_hp(run, cx, run.hero.max_hp - before, "shrine");
    // The shrine's price opens an episode of its own.
    if run.arc.has_low() {
        sifter::seal(run);
    }
    sifter::on_hurt(run, "shrine", None);
    let lent = if want_row {
        let active = &cx.sets[cx.active_set.min(cx.sets.len() - 1)];
        cx.sets.iter().enumerate().filter(|(i, _)| *i != cx.active_set).flat_map(|(_, s)| s.rows.iter()).find(|r| !active.rows.contains(r)).cloned()
    } else {
        None
    };
    let what = match lent {
        Some(r) => {
            run.lent_row = Some(r);
            "a row lent".to_string()
        }
        None => {
            run.trait_ = run.trait_.swap();
            format!("now {}", run.trait_.name())
        }
    };
    run.met_situation("shrine");
    learn(run, cx, "shrine".into());
    // QA on 524827b (qaAB: `first: the shrine` on a death whose lines never showed a shrine — they read `Prayed: …`): the
    // line names the shrine the news does
    note(run, cx, format!("Prayed at a shrine: {what}, −{cost} max HP."));
    callout(run, cx, "prayed");
}

/// A pack swap's move of the carried gold (`raw`: the find's value less what the dropped
/// item counted): what it took off the carry, in coins as the stake shows it, is the run's
/// `swapped` (QA on 778fa1b: `−$37 swapped` on the strip, in no ledger).
fn swap_loot(run: &mut Run, cx: &Ctx, raw: i32, dropped: &crate::item::Item) {
    let before = run.loot;
    run.loot_add(raw);
    let cost = (before - run.loot).max(0);
    run.swapped += cost;
    if cost > 0 {
        let (_, _, label) = crate::item::describe(dropped, cx.facts, cx.flavours);
        run.swap_left.push(label);
    }
}

/// Pick up whatever lies on the hero's tile. Cut 20 §1: an item a thief stole this run,
/// taken back, is a note (`Got the heal back.`).
pub fn pickup_here(run: &mut Run, cx: &mut Ctx) {
    if run.stolen_ids.is_empty() {
        pickup_item_here(run, cx);
        return;
    }
    let here = run.hero.pos;
    let back: Vec<(u32, Option<i32>)> = run.items.iter().filter(|fi| fi.pos == here && run.stolen_ids.contains(&fi.item.id)).map(|fi| (fi.item.id, (fi.item.kind == "gold").then_some(fi.item.amount))).collect();
    pickup_item_here(run, cx);
    for (id, coins) in back {
        // Cut 22 §2: stolen coins picked up are in the carry, not the pack (`Got $6 back.`).
        let (label, text) = match coins {
            Some(n) if !run.items.iter().any(|fi| fi.item.id == id) => (format!("${n}"), format!("Got ${n} back.")),
            Some(_) => continue,
            None => {
                let Some(it) = run.hero.inv.iter().chain(run.hero.weapon.iter()).chain(run.hero.armour.iter()).find(|i| i.id == id) else { continue };
                let (_, _, label) = crate::item::describe(it, cx.facts, cx.flavours);
                let label = label.trim_end_matches('?').to_string();
                let text = format!("Got the {label} back.");
                (label, text)
            }
        };
        run.stolen_ids.retain(|x| *x != id);
        run.recovered.push((run.turn, label));
        note(run, cx, text);
        callout(run, cx, "got it back");
    }
}

fn pickup_item_here(run: &mut Run, cx: &mut Ctx) {
    // Several items may share a tile (a recovered kit that did not fit): take the first
    // that would change anything.
    let here = run.hero.pos;
    let Some(ii) = run.items.iter().position(|fi| fi.pos == here && (can_take(&run.hero, &fi.item) || (queen_wants(run, &fi.item) && queen_slot(run, cx).is_some()))) else { return };
    let item = &run.items[ii].item;
    if item.kind == "bones" {
        recover_bones(run, cx, ii);
        return;
    }
    if item.cat() == Cat::Gold {
        let it = run.items.remove(ii).item;
        run.loot_add_gold(it.amount);
        if run.bounty == Some(run.depth) {
            run.bounty_gold += it.amount;
        }
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: format!("gold ${}", it.amount) });
        // Cut 5 §3: gold under a foe's nose.
        if !view(run).foes.is_empty() {
            sifter::voice(run, cx, Moment::GoldWithFoes);
        }
        return;
    }
    if crate::defs::FACT_MISC.contains(&item.kind.as_str()) && !run.hero.inv_full() {
        // Cut 3: a found tool is a fact (tokens and unlocks open on it).
        let it = run.items.remove(ii).item;
        let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
        let kind = it.kind.clone();
        run.loot_add(it.value());
        run.note_found(it.id, &kind, it.amount.max(1));
        run.hero.inv.push(it);
        crate::provenance::found(run, cx, &kind, &label);
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
        learn(run, cx, format!("item:{kind}"));
        return;
    }
    if item.kind == "leash" {
        let it = run.items.remove(ii).item;
        // QA on 778fa1b (qaV): a found leash's units are counted on the stack they join (a
        // thief's take coming back is not a find).
        let (units, own) = (it.amount.max(1), it.id);
        let coming_back = run.stolen_ids.contains(&own) || run.supplies.contains(&own) || run.brought.contains(&own);
        match run.hero.inv.iter_mut().find(|i| i.kind == "leash") {
            Some(l) => {
                l.amount += units;
                let stack = l.id;
                if coming_back {
                    run.note_found(own, "leash", units);
                } else {
                    for _ in 0..units {
                        run.found_units.push((stack, "leash".into()));
                    }
                }
            }
            None => {
                if run.hero.inv_full() {
                    run.items.push(crate::engine::FloorItem { pos: run.hero.pos, item: it });
                    return;
                }
                let mut l = it;
                l.amount = l.amount.max(1);
                run.note_found(own, "leash", units);
                run.hero.inv.push(l);
            }
        }
        run.loot_add(5);
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: "leash".into() });
        crate::provenance::found(run, cx, "leash", "leash");
        learn(run, cx, "item:leash".into());
        return;
    }
    // Cut 29 (the Lurker Queen, D28): a full pack of enchanted summons and spares walked past
    // every silence scroll from D24 (205 floors, none held) — its value (18) is under an
    // enchanted scroll's. From D24 a silence takes the slot of an unread summon or a spare.
    if run.hero.inv_full() && queen_wants(run, item) {
        if let Some(k) = queen_slot(run, cx) {
            swap_in(run, cx, ii, k);
            return;
        }
    }
    if run.hero.inv_full() && !item_replaces_gear(&run.hero, item) {
        // Cut 3: a full pack keeps one spare weapon and one spare armour; a second spare makes
        // way for a consumable or a bow (spares are salvage; potions and scrolls are the run).
        // A bow (the answer to reflected melee) takes the slot of any spare melee weapon.
        let wants_slot = item.is_consumable() || item.def().ranged;
        if wants_slot {
            let need = if item.def().ranged && !run.hero.inv.iter().any(|i| i.def().ranged) { 1 } else { 2 };
            let spare = |cat: Cat| -> Option<usize> {
                let spares: Vec<usize> = run.hero.inv.iter().enumerate().filter(|(_, i)| i.cat() == cat && !i.def().ranged && !row_needs(run, cx, i) && !crate::kit::is_kit_id(i.id)).map(|(k, _)| k).collect();
                if spares.len() >= need {
                    spares.into_iter().min_by_key(|&k| (run.hero.inv[k].value(), run.hero.inv[k].id))
                } else {
                    None
                }
            };
            if let Some(k) = spare(Cat::Weapon).or_else(|| spare(Cat::Armour)) {
                let dropped = run.hero.inv.remove(k);
                let here = run.hero.pos;
                let it = run.items.remove(ii).item;
                let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
                let raw = it.value() - run.loot_value(&dropped);
                swap_loot(run, cx, raw, &dropped);
                crate::provenance::spent(run, cx, &dropped.kind, format!("swapped for the {}", it.kind.replace('_', " ")));
                crate::provenance::found(run, cx, &it.kind, &label);
                run.note_gone(dropped.id, &dropped.kind, "left", dropped.amount.max(1));
                run.note_found(it.id, &it.kind, 1);
                run.hero.inv.push(it);
                run.items.push(crate::engine::FloorItem { pos: here, item: dropped });
                cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
                return;
            }
        }
        // A full pack swaps its cheapest consumable for a dearer one (a chore, silently);
        // Cut 3: a third copy of a kind makes way for a kind the pack lacks (a full pack of
        // summons and recalls walked past every silence scroll in the Deep).
        // Cut 12 §2: never what a row needs — a kind a `drink` / `read` / `throw` row names,
        // or a supply packed at camp (rater O: `swapped for the poison` took the bought heal
        // `hp<30 → drink heal` was written for).
        let dup = duplicate_slot(&run.hero, item).filter(|&k| !row_needs(run, cx, &run.hero.inv[k]) && !queen_keeps(run, &run.hero.inv[k]));
        let swap = dup.or_else(|| run.hero.inv.iter().enumerate().filter(|(_, i)| i.is_consumable() && !row_needs(run, cx, i) && !queen_keeps(run, i)).min_by_key(|(_, i)| (i.value(), i.id)).map(|(k, _)| k));
        let swap = swap.map(|k| (k, if dup.is_some() { i32::MIN } else { run.hero.inv[k].value() }));
        match swap {
            Some((k, v)) if item.is_consumable() && item.value() > v => {
                let dropped = run.hero.inv.remove(k);
                let here = run.hero.pos;
                let it = run.items.remove(ii).item;
                let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
                let raw = it.value() - run.loot_value(&dropped);
                swap_loot(run, cx, raw, &dropped);
                crate::provenance::spent(run, cx, &dropped.kind, format!("swapped for the {}", it.kind.replace('_', " ")));
                crate::provenance::found(run, cx, &it.kind, &label);
                run.note_gone(dropped.id, &dropped.kind, "left", dropped.amount.max(1));
                run.note_found(it.id, &it.kind, 1);
                run.hero.inv.push(it);
                run.items.push(crate::engine::FloorItem { pos: here, item: dropped });
                cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
            }
            _ => {}
        }
        return;
    }
    let it = run.items.remove(ii).item;
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    run.loot_add(it.value());
    crate::provenance::found(run, cx, &it.kind, &label);
    run.note_found(it.id, &it.kind, 1);
    // QA on 778fa1b (qaV: a found `leather +1` in no named place): the piece a better one
    // displaces with the pack full is put down here, not dropped from the world.
    if let Some(old) = run.hero.auto_equip(it) {
        if !run.hero.inv.iter().any(|i| i.id == old.id && i.kind == old.kind) {
            let here = run.hero.pos;
            run.note_gone(old.id, &old.kind, "left", 1);
            drop_near(run, here, old);
        }
    }
    cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
}

/// Cut 29: the band where the Lurker Queen's counter (a silence scroll) earns a pack slot.
pub const QUEEN_PACK_DEPTH: u32 = 24;

/// Cut 29: from `QUEEN_PACK_DEPTH`, a silence scroll is worth a slot while the pack holds fewer
/// than two (the Queen's summons are read down one scroll at a time).
pub fn queen_wants(run: &Run, item: &Item) -> bool {
    item.kind == "silence" && run.depth >= QUEEN_PACK_DEPTH && run.hero.inv.iter().filter(|i| i.kind == "silence").count() < 2
}

/// Whether a held item is the Queen's counter the pack keeps (never swapped out for a dearer
/// consumable: a silence taken for a summon would be traded back for it on the next step).
fn queen_keeps(run: &Run, it: &Item) -> bool {
    it.kind == "silence" && run.depth >= QUEEN_PACK_DEPTH
}

/// The slot a silence takes (`queen_wants`): an unread summon (no row names it, not packed at
/// camp) first, else a spare weapon or armour (not the forged kit, not a bow, not brought from the
/// vault) — the cheapest.
pub fn queen_slot(run: &Run, cx: &Ctx) -> Option<usize> {
    let free = |i: &Item| !row_needs(run, cx, i) && !crate::kit::is_kit_id(i.id) && !run.brought.contains(&i.id);
    let pick = |f: &dyn Fn(&Item) -> bool| run.hero.inv.iter().enumerate().filter(|(_, i)| free(i) && f(i)).min_by_key(|(_, i)| (i.value(), i.id)).map(|(k, _)| k);
    pick(&|i| i.kind == "summon_ally").or_else(|| pick(&|i| matches!(i.cat(), Cat::Weapon | Cat::Armour) && !i.def().ranged))
}

/// A full pack puts down its slot `k` and takes the floor item `ii` in its place (a chore).
fn swap_in(run: &mut Run, cx: &mut Ctx, ii: usize, k: usize) {
    let dropped = run.hero.inv.remove(k);
    let here = run.hero.pos;
    let it = run.items.remove(ii).item;
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    let raw = it.value() - run.loot_value(&dropped);
    swap_loot(run, cx, raw, &dropped);
    crate::provenance::spent(run, cx, &dropped.kind, format!("swapped for the {}", it.kind.replace('_', " ")));
    crate::provenance::found(run, cx, &it.kind, &label);
    run.note_gone(dropped.id, &dropped.kind, "left", dropped.amount.max(1));
    run.note_found(it.id, &it.kind, 1);
    run.hero.inv.push(it);
    run.items.push(crate::engine::FloorItem { pos: here, item: dropped });
    cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
}

/// Cut 2 §2: a later heir steps on a bones pile and recovers the kit (what does not fit the
/// pack lies where it stood). A note always; the highlight is settled at the exit.
fn recover_bones(run: &mut Run, cx: &mut Ctx, ii: usize) {
    let here = run.hero.pos;
    let heir = run.items.remove(ii).item.amount as u32;
    let Some(pile) = run.bones.iter().find(|b| b.heir == heir).cloned() else { return };
    run.bones_found.push(heir);
    let n = pile.items.len() as u32;
    // QA on 778fa1b (qaV): a pile's items carry their old run's ids, which this run's own may
    // share — a find is told by its id and kind (`Run.found_units`). The pack's own test stays by
    // id: a pile item whose id a pack item bears reads as taken, and when the full pack in fact
    // left it, it is neither carried nor put down (it leaves the world, as before). Telling it
    // by id and kind (put down beside the pile) walked FULL−D28 past the Lich without silence on
    // 8 of 30 seeds (3 before): the D28 wall is held in part by this loss — a content decision,
    // left to the contract; the find is not counted (it was not taken).
    let held = |h: &crate::hero::Hero, id: u32, kind: &str| h.inv.iter().chain(h.weapon.iter()).chain(h.armour.iter()).filter(|i| i.id == id && i.kind == kind).count();
    for it in pile.items {
        let value = it.value();
        let cat = it.cat();
        let before = held(&run.hero, it.id, &it.kind);
        let replaced = run.hero.auto_equip(it.clone());
        let taken = match cat {
            Cat::Weapon => run.hero.weapon.as_ref().is_some_and(|w| w.id == it.id) || run.hero.inv.iter().any(|i| i.id == it.id),
            Cat::Armour => run.hero.armour.as_ref().is_some_and(|a| a.id == it.id) || run.hero.inv.iter().any(|i| i.id == it.id),
            _ => run.hero.inv.iter().any(|i| i.id == it.id),
        };
        if taken {
            run.loot_add(value);
            let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
            crate::provenance::found(run, cx, &it.kind, &label);
            if held(&run.hero, it.id, &it.kind) > before {
                run.note_found(it.id, &it.kind, it.amount.max(1));
            }
        } else {
            drop_near(run, here, it);
        }
        if let Some(old) = replaced {
            if !run.hero.inv.iter().any(|i| i.id == old.id && i.kind == old.kind) {
                run.note_gone(old.id, &old.kind, "left", 1);
                drop_near(run, here, old);
            }
        }
    }
    cx.events.push(Ev::Bones { t: run.turn, heir, items: n });
    note(run, cx, format!("Found heir {heir}'s bones: {n} items."));
    callout(run, cx, "bones");
}

/// Drop an item on the nearest free floor tile around `at` (its own tile if none).
pub fn drop_near(run: &mut Run, at: Pos, it: Item) {
    let free = std::iter::once(at)
        .chain(at.neighbours8())
        .find(|q| run.floor.map.in_bounds(*q) && run.floor.map.get(*q) == Tile::Floor && run.item_at(*q).is_none());
    run.items.push(crate::engine::FloorItem { pos: free.unwrap_or(at), item: it });
}

/// Cut 3: the pack slot a third copy of a consumable kind gives up for a kind not held twice.
/// Cut 12 §2: does a row need this item? The kinds a `drink` / `read` / `throw` row of the
/// set names (a card's sheet rows and the lent row too, `recall` under `recall_sense`), and
/// every supply packed at camp (by id). The pickup swap chores never drop one.
pub fn row_needs(run: &Run, cx: &Ctx, it: &Item) -> bool {
    if run.supplies.contains(&it.id) {
        return true;
    }
    let kind = it.kind.as_str();
    if kind == "recall" && cx.unlocks.contains("recall_sense") {
        return true;
    }
    let names = |r: &crate::rules::Row| matches!(r.verb.v.as_str(), "drink" | "read" | "throw") && r.verb.a.as_deref().and_then(|a| a.split(',').next()) == Some(kind);
    cx.rules.active(cx.max_rows()).map(|(_, r)| r).chain(run.lent_row.iter()).any(|r| names(r) || r.card().is_some_and(|c| crate::meta::unlock_rows_any(c, names)))
}

fn duplicate_slot(h: &crate::hero::Hero, item: &Item) -> Option<usize> {
    if !item.is_consumable() {
        return None;
    }
    let count = |k: &str| h.inv.iter().filter(|i| i.kind == k).count();
    if count(&item.kind) >= 2 {
        return None;
    }
    // Never for something cheaper (an aggravate scroll and a third poison swapped for ever).
    h.inv
        .iter()
        .enumerate()
        .filter(|(_, i)| i.is_consumable() && count(&i.kind) >= 3 && item.value() >= i.value())
        .min_by_key(|(_, i)| (i.value(), i.id))
        .map(|(k, _)| k)
}

/// Would picking this up change anything (gold, leash, room in the pack, or better gear)?
/// `can_take` as `pickup_here` will actually decide it — with a full pack, only when the swap
/// it would make touches nothing a row needs (Cut 12 §2). The chores path by this: by
/// `can_take` alone a hero stood on a scroll its full pack would never take and read
/// `pick up` every action until the stall guard ended the run (cohort 9, both raters' first
/// gripe: "the most expensive outcome in the game").
pub fn would_take(run: &Run, cx: &Ctx, item: &Item) -> bool {
    would_take_in(run, cx, item, &PackRead::default())
}

/// What `would_take` and `can_take` read of the pack alone — the same for every item one look
/// weighs (`ai::nearest_item_step` weighs each seen item against an unchanged pack, run and rules) —
/// worked out once, on first read.
#[derive(Default)]
pub struct PackRead {
    /// `row_needs` of each pack slot.
    needs: std::cell::OnceCell<Vec<bool>>,
    /// A ranged item in the pack.
    ranged: std::cell::OnceCell<bool>,
    /// The weapon and armour spares `can_take` counts (not ranged, not the forged kit).
    spares: std::cell::OnceCell<[usize; 2]>,
    /// … and those `would_take` counts (no row needs them either).
    free_spares: std::cell::OnceCell<[usize; 2]>,
    /// The cheapest consumable's value.
    cheapest: std::cell::OnceCell<Option<i32>>,
    /// The swap a full pack makes for a dearer consumable when no duplicate gives way.
    swap: std::cell::OnceCell<Option<usize>>,
    /// `queen_slot`.
    queen: std::cell::OnceCell<Option<usize>>,
}

impl PackRead {
    fn needs(&self, run: &Run, cx: &Ctx) -> &[bool] {
        self.needs.get_or_init(|| run.hero.inv.iter().map(|i| row_needs(run, cx, i)).collect())
    }
    fn ranged(&self, h: &crate::hero::Hero) -> bool {
        *self.ranged.get_or_init(|| h.inv.iter().any(|i| i.def().ranged))
    }
    fn queen(&self, run: &Run, cx: &Ctx) -> Option<usize> {
        *self.queen.get_or_init(|| queen_slot(run, cx))
    }
}

/// The weapon and armour items of `h`'s pack `keep` passes, by category.
fn spare_counts(h: &crate::hero::Hero, keep: impl Fn(usize, &Item) -> bool) -> [usize; 2] {
    let mut n = [0, 0];
    for (k, i) in h.inv.iter().enumerate() {
        let c = match i.cat() {
            Cat::Weapon => 0,
            Cat::Armour => 1,
            _ => continue,
        };
        if keep(k, i) {
            n[c] += 1;
        }
    }
    n
}

/// `would_take`, the pack read through `pk` (`PackRead`: one per unchanged pack, run and rules).
pub fn would_take_in(run: &Run, cx: &Ctx, item: &Item, pk: &PackRead) -> bool {
    let h = &run.hero;
    if !(can_take_in(h, item, pk) || (queen_wants(run, item) && pk.queen(run, cx).is_some())) {
        return false;
    }
    if !h.inv_full() || item.kind == "bones" || item.cat() == Cat::Gold || (item.kind == "leash" && h.inv.iter().any(|i| i.kind == "leash")) || item_replaces_gear(h, item) {
        return true;
    }
    if queen_wants(run, item) && pk.queen(run, cx).is_some() {
        return true;
    }
    if item.is_consumable() || item.def().ranged {
        let need = if item.def().ranged && !pk.ranged(h) { 1 } else { 2 };
        // Cut 25 §3: the forged kit is never put down (`pickup_here`'s own spares skip it) — counted here as a
        // spare it made a full pack walk onto a scroll it could not take, step off, walk back: `pick up ×441`.
        let spare = pk.free_spares.get_or_init(|| {
            let needs = pk.needs(run, cx);
            spare_counts(h, |k, i| !i.def().ranged && !needs[k] && !crate::kit::is_kit_id(i.id))
        });
        if spare[0] >= need || spare[1] >= need {
            return true;
        }
    }
    let needs = pk.needs(run, cx);
    let dup = duplicate_slot(h, item).filter(|&k| !needs[k] && !queen_keeps(run, &h.inv[k]));
    let swap = dup.or_else(|| *pk.swap.get_or_init(|| h.inv.iter().enumerate().filter(|(k, i)| i.is_consumable() && !needs[*k] && !queen_keeps(run, i)).min_by_key(|(_, i)| (i.value(), i.id)).map(|(k, _)| k)));
    let swap = swap.map(|k| if dup.is_some() { i32::MIN } else { h.inv[k].value() });
    matches!(swap, Some(v) if item.is_consumable() && item.value() > v)
}

pub fn can_take(h: &crate::hero::Hero, item: &Item) -> bool {
    can_take_in(h, item, &PackRead::default())
}

/// `can_take`, the pack read through `pk`.
fn can_take_in(h: &crate::hero::Hero, item: &Item, pk: &PackRead) -> bool {
    if item.kind == "trap" {
        return false;
    }
    if item.kind == "bones" {
        return true;
    }
    // (read only when the spares are: a pack with room takes it before)
    let need = || if item.def().ranged && !pk.ranged(h) { 1 } else { 2 };
    let second_spare = |c: usize| pk.spares.get_or_init(|| spare_counts(h, |_, i| !i.def().ranged && !crate::kit::is_kit_id(i.id)))[c] >= need();
    matches!(item.cat(), Cat::Gold)
        || (item.kind == "leash" && h.inv.iter().any(|i| i.kind == "leash"))
        || !h.inv_full()
        || item_replaces_gear(h, item)
        || (item.is_consumable() && pk.cheapest.get_or_init(|| h.inv.iter().filter(|i| i.is_consumable()).map(|i| i.value()).min()).is_some_and(|v| item.value() > v))
        || ((item.is_consumable() || item.def().ranged) && (second_spare(0) || second_spare(1)))
        || duplicate_slot(h, item).is_some()
}

fn item_replaces_gear(h: &crate::hero::Hero, item: &Item) -> bool {
    match item.cat() {
        Cat::Weapon => {
            let cur = h.weapon.as_ref().map(|w| w.atk().0 + w.atk().1).unwrap_or(0);
            item.atk().0 + item.atk().1 > cur
        }
        Cat::Armour => crate::hero::armour_worth(item) > h.armour.as_ref().map(crate::hero::armour_worth).unwrap_or(i32::MIN),
        _ => false,
    }
}

pub fn monster_kind_title(kind: &str) -> String {
    monster_def(kind).title.to_string()
}
