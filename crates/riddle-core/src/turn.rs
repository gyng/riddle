//! The tick: energy scheduler (Addendum E), hero action (rules → verbs → chores), monsters,
//! overlays, clocks, vision. All durations are ticks; 10 ticks ≈ one turn at base speed.
use crate::ai;
use crate::chronicle::{callout, note};
use crate::defs::{monster_def, Cat};
use crate::descent::{biome_for, ENDING_DEPTH};
use crate::engine::{populate_floor, Ctx, ExitTier, Run, ACT_ENERGY, HERO_ID, TICKS_PER_TURN};
use crate::facts::{learn, learn_tag, tag_known};
use crate::gen::generate;
use crate::geom::{Pos, DIRS8};
use crate::hero::Trait;
use crate::item::{is_identified, Item};
use crate::monster::Monster;
use crate::rules::{Cond, Verb};
use crate::tiles::{Overlay, OverlayKind, Tile, VISION};
use crate::wire::{Ev, TraceTurn};

/// What the hero can see this action.
#[derive(Clone, Debug, Default)]
pub struct View {
    /// Visible hostile monster indices, nearest first.
    pub foes: Vec<usize>,
    pub adj: i32,
    pub nearest: Option<usize>,
    pub lowest: Option<usize>,
}

/// Foes the hero can engage: visible hostiles that are adjacent, or neither fleeing nor
/// given up on (unreachable / not closing). Rows count and target only these.
pub fn view(run: &Run) -> View {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let mut foes: Vec<usize> = (0..run.monsters.len())
        .filter(|&i| {
            let m = &run.monsters[i];
            m.hp > 0 && m.hostile() && map.is_visible(m.pos) && (m.pos.adjacent(hp) || (!m.fleeing && m.fear == 0 && !run.is_ignored(m.id)))
        })
        .collect();
    foes.sort_by_key(|&i| (run.monsters[i].pos.cheb(hp), run.monsters[i].id));
    let adj = foes.iter().filter(|&&i| run.monsters[i].pos.adjacent(hp)).count() as i32;
    let nearest = foes.first().copied();
    let lowest = foes.iter().copied().min_by_key(|&i| (run.monsters[i].hp, run.monsters[i].id));
    View { foes, adj, nearest, lowest }
}

/// The cached hero distance field, recomputed when the hero has moved.
pub fn hero_dist(run: &mut Run) -> &[i32] {
    if run.hero_dist_pos != Some(run.hero.pos) || run.hero_dist.len() != run.floor.map.tiles.len() {
        run.hero_dist = run.floor.map.bfs(run.hero.pos, false, &|_| false);
        run.hero_dist_pos = Some(run.hero.pos);
    }
    &run.hero_dist
}

pub fn tick(run: &mut Run, cx: &mut Ctx) {
    if run.over.is_some() {
        return;
    }
    run.turn += 1;
    run.floor_turn += 1;
    run.hero.energy += run.hero.speed();
    for m in run.monsters.iter_mut() {
        if m.hp > 0 {
            m.energy += m.effective_speed();
        }
    }
    let mut acted = false;
    // Hero first.
    while run.hero.energy >= ACT_ENERGY && run.over.is_none() {
        run.hero.energy -= ACT_ENERGY;
        hero_action(run, cx);
        acted = true;
        run.floor.map.update_vision(run.hero.pos, VISION);
        crate::facts::on_vision(run, cx);
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
}

fn hero_action(run: &mut Run, cx: &mut Ctx) {
    run.actions += 1;
    run.note_foes();
    oscillation_guard(run, cx);
    let v = view(run);
    let hp_before = run.hero.hp;
    let inv_before = run.hero.inv.len() + run.hero.weapon.is_some() as usize + run.hero.armour.is_some() as usize;
    let (row, verb) = choose_and_act(run, cx, &v);
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
    run.recent_pos.push(run.hero.pos);
    if run.recent_pos.len() > 12 {
        run.recent_pos.remove(0);
    }
    // Trace records the state at the start of the action.
    let telegraphs: Vec<String> = v
        .foes
        .iter()
        .filter_map(|&i| run.monsters.get(i).and_then(|m| m.telegraph.as_ref().map(|t| format!("{} {}", m.kind, t))))
        .collect();
    run.trace.push(TraceTurn { t: run.turn, row, verb, hp: hp_before, foes: v.foes.len() as i32, telegraphs });
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
    if run.hero.paralysed > 0 {
        let verb = Verb::new("paralysed");
        emit_rule(run, cx, -2, &verb, "paralysed");
        return (-2, verb);
    }
    if run.hero.confused > 0 && run.rng.chance(50) {
        ai::random_step(run, cx);
        let verb = Verb::new("stumble");
        emit_rule(run, cx, -2, &verb, "confused → stumble");
        return (-2, verb);
    }
    let foes = v.foes.len() as i32;
    let hp_pct = run.hero.hp_pct();
    let tr = run.trait_;
    // Trait deviations, announced: at most one per 5 actions, and never below 25% HP
    // (cowardice excepted, since fleeing at low HP is its point).
    let trait_ready = run.trait_last.is_none_or(|t| run.actions >= t + 5);
    if tr == Trait::Cowardly && hp_pct < 50 && foes >= 1 && run.cowardly_streak < 3 {
        let verb = Verb::new("retreat");
        if ai::try_verb(run, cx, &verb, v) {
            run.cowardly_streak += 1;
            run.trait_last = Some(run.actions);
            emit_rule(run, cx, -1, &verb, "cowardly → retreat");
            return (-1, verb);
        }
    }
    if foes == 0 {
        run.cowardly_streak = 0;
    }
    let trait_ok = trait_ready && hp_pct >= 25;
    if tr == Trait::Greedy && trait_ok && !run.items_ignored() {
        let hp = run.hero.pos;
        let target = DIRS8
            .iter()
            .map(|d| hp.step(*d))
            .find(|q| run.item_at(*q).is_some_and(|ii| can_take(&run.hero, &run.items[ii].item)) && !run.occupied(*q) && run.floor.map.can_step(hp, *q));
        if let Some(q) = target {
            ai::move_hero(run, cx, q);
            run.trait_last = Some(run.actions);
            let verb = Verb::new("pick_up");
            emit_rule(run, cx, -1, &verb, "greedy → pick up");
            return (-1, verb);
        }
    }
    // Sanity: nobody stands in gas or fire with no foe adjacent.
    if v.adj == 0 && ai::escape_hazard(run, cx, v) {
        let verb = Verb::new("explore");
        emit_rule(run, cx, -2, &verb, "hazard → step out");
        return (-2, verb);
    }
    let rows: Vec<crate::rules::Row> = cx.rules.rows.iter().take(cx.max_rows).cloned().collect();
    let mut brave_said = false;
    let stuck = run.stuck_until > run.actions;
    let suppressed = if run.row_suppressed.1 > run.actions { run.row_suppressed.0 } else { -9 };
    for (i, row) in rows.iter().enumerate() {
        if (stuck && targets_foes(&row.verb)) || i as i32 == suppressed {
            continue;
        }
        if !row.conds.iter().all(|c| cond_holds(run, cx, v, c)) {
            continue;
        }
        if tr == Trait::Brave && foes == 1 && matches!(row.verb.v.as_str(), "retreat" | "back_corridor") {
            if !brave_said {
                emit_rule(run, cx, -1, &Verb::new("attack"), "brave → hold");
                brave_said = true;
            }
            continue;
        }
        let scope = row.conds.iter().find(|c| c.k == "party").and_then(|c| c.t.clone());
        if ai::try_verb_scoped(run, cx, &row.verb, v, scope.as_deref()) {
            let text = row.text(hp_pct);
            emit_rule(run, cx, i as i32, &row.verb, &text);
            if i < run.row_fired.len() {
                run.row_fired[i] += 1;
            }
            if run.over.is_some() && run.exit_row.is_none() {
                run.exit_row = Some(i as i32);
            }
            if matches!(row.verb.v.as_str(), "recall" | "send") {
                continue; // party orders are free actions
            }
            return (i as i32, row.verb.clone());
        }
    }
    if tr == Trait::Curious && trait_ok && foes == 0 && hp_pct >= 50 {
        if let Some(verb) = ai::curious_use(run, cx) {
            run.trait_last = Some(run.actions);
            emit_rule(run, cx, -1, &verb, &format!("curious → {}", verb.short()));
            return (-1, verb);
        }
    }
    let verb = ai::chore(run, cx, v);
    emit_rule(run, cx, -2, &verb, &format!("chore → {}", verb.short()));
    (-2, verb)
}

/// If the last 12 actions visited ≤ 2 tiles with no damage dealt or taken, give up on the
/// visible foes for 30 actions and let chores proceed; one `stuck` chore event explains it.
fn oscillation_guard(run: &mut Run, cx: &mut Ctx) {
    if run.stuck_until > run.actions || run.recent_pos.len() < 12 {
        return;
    }
    let mut tiles: Vec<Pos> = run.recent_pos.clone();
    tiles.sort();
    tiles.dedup();
    if tiles.len() > 2 || run.actions.saturating_sub(run.last_damage_action) < 12 {
        return;
    }
    // Engaged in melee is not stuck: adjacent foes are always worth a row.
    let hp = run.hero.pos;
    let v = view(run);
    if v.foes.iter().any(|&i| run.monsters[i].pos.adjacent(hp)) {
        return;
    }
    let ids: Vec<u32> = v.foes.iter().map(|&i| run.monsters[i].id).collect();
    if ids.is_empty() {
        return;
    }
    for id in ids {
        run.ignore(id, 30);
    }
    run.stuck_until = run.actions + 30;
    run.recent_pos.clear();
    run.chase = None;
    let verb = Verb::new("stuck");
    emit_rule(run, cx, -2, &verb, "stuck → chores");
}

pub fn emit_rule(run: &Run, cx: &mut Ctx, row: i32, verb: &Verb, text: &str) {
    cx.events.push(Ev::Rule { t: run.turn, row, verb: verb.clone(), text: crate::chronicle::clamp_words(text, 3) });
}

/// Verbs that act on the visible foes (suppressed while the oscillation guard is up).
fn targets_foes(verb: &Verb) -> bool {
    matches!(
        verb.v.as_str(),
        "attack" | "shield_bash" | "throw" | "tame" | "cleave" | "backstab" | "ambush" | "shadowstep" | "send" | "taunt"
            | "shoot" | "volley" | "mark" | "double_shot" | "bolt" | "slow" | "drain"
    )
}

pub fn cond_holds(run: &Run, cx: &Ctx, v: &View, c: &Cond) -> bool {
    let n = c.n.unwrap_or(0);
    let t = c.t.as_deref().unwrap_or("");
    let h = &run.hero;
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
        "item" => h.has_kind(t) && is_identified(cx.facts, cx.flavours, t),
        "unknown_item" => h.inv.iter().any(|i| i.is_consumable() && !is_identified(cx.facts, cx.flavours, &i.kind)),
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
}

impl Src {
    pub fn cause(&self, run: &Run) -> String {
        match self {
            Src::Hero { .. } => "hero".into(),
            Src::Mon(i) => run.monsters[*i].kind.clone(),
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
    let (dmg, counter) = counter_damage(run, src, dmg, None, run.hero.pos, false);
    if let Some((a, b)) = counter {
        learn(run, cx, crate::defs::counter_fact(&a, &b));
    }
    if dmg <= 0 {
        return;
    }
    run.hero.hp -= dmg;
    run.hurt_since_action = true;
    run.last_damage_action = run.actions;
    if run.boss_seen_t.is_some() {
        run.hurt_since_boss = true;
    }
    cx.events.push(Ev::Hurt { t: run.turn, id: HERO_ID, dmg, hp: run.hero.hp.max(0), cause: cause.into() });
    let pct = run.hero.hp_pct();
    if run.hero.hp > 0 {
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
        run.hero.hp = 0;
        run.death_cause = Some(cause.to_string());
        run.death_blow = dmg;
        cx.events.push(Ev::Die { t: run.turn, id: HERO_ID, cause: cause.into() });
        let depth = run.depth;
        note(run, cx, format!("Slain by {} on D{}.", crate::engine::kind_title(cause), depth));
        end_run(run, cx, ExitTier::Death);
    }
}

/// Monster takes damage; handles counters, splits, pops, drops, kills. Returns true if it died.
pub fn damage_monster(run: &mut Run, cx: &mut Ctx, mi: usize, dmg: i32, src: &Src) -> bool {
    if run.monsters[mi].hp <= 0 {
        return false;
    }
    // The Warlord's shield wall: a goblin beside him takes any incidental blow from the
    // hero's side (unaimed swings, allies, companions). Hazards and aimed strikes go through.
    let mut mi = mi;
    if run.monsters[mi].kind == "goblin_warlord" && !run.aimed {
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
                // No goblin left in view: a reserve steps in from behind him to take the blow.
                let q = wp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q))?;
                let id = run.new_id();
                let depth = run.depth;
                let mut g = Monster::spawn(id, "goblin", q, depth);
                g.awake = true;
                g.last_seen = Some(run.hero.pos);
                g.summoned = true;
                g.extra_tags.push("summoned".into());
                let e = crate::engine::monster_entity(&g, cx.facts);
                run.monsters.push(g);
                cx.events.push(Ev::Spawn { t: run.turn, e });
                Some(run.monsters.len() - 1)
            });
            if let Some(g) = guard {
                if run.floor.map.is_visible(wp) {
                    callout(run, cx, "shielded");
                }
                mi = g;
            }
        }
    }
    let cause = src.cause(run);
    let cause = cause.as_str();
    let dmg = dmg.max(0);
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
    if matches!(src, Src::Hero { .. }) && dmg > 0 {
        run.last_damage_action = run.actions;
    }
    run.monsters[mi].hp -= dmg;
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
    let visible = run.floor.map.is_visible(pos);
    if hp > 0 {
        if run.monsters[mi].has_tag("splitter") && hp > 4 && dmg > 0 && !run.monsters[mi].ally {
            let half = hp / 2;
            run.monsters[mi].hp = hp - half;
            let free = pos.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
            if let Some(q) = free {
                let nid = run.new_id();
                let depth = run.depth;
                let mut child = Monster::spawn(nid, &kind, q, depth);
                child.hp = half;
                child.max_hp = run.monsters[mi].max_hp;
                child.awake = true;
                child.last_seen = run.monsters[mi].last_seen;
                let e = crate::engine::monster_entity(&child, cx.facts);
                run.monsters.push(child);
                cx.events.push(Ev::Spawn { t: run.turn, e });
                if visible {
                    callout(run, cx, "splits!");
                    learn_tag(run, cx, &kind, "splitter");
                }
            }
        }
        return false;
    }
    run.monsters[mi].hp = 0;
    cx.events.push(Ev::Die { t: run.turn, id, cause: cause.into() });
    let m = run.monsters[mi].clone();
    if m.ally {
        run.ally_lost.push((run.turn, kind.clone()));
        cx.events.push(Ev::Ally { t: run.turn, id, state: "lost".into() });
        if let Some(cid) = m.cid {
            let name = run.companion(cid).map(|c| c.name.clone()).unwrap_or_else(|| kind.clone());
            run.lost_companions.push((run.turn, name.clone()));
            note(run, cx, format!("{name} the {} fell.", crate::engine::kind_title(&kind)));
        } else {
            note(run, cx, format!("The {} fell.", crate::engine::kind_title(&kind)));
        }
    } else if !m.neutral {
        let depth = run.depth;
        run.kills.push((run.turn, kind.clone(), depth));
        run.kills_floor += 1;
        run.kill_since_action = true;
        crate::facts::on_kill(run, cx, &kind);
        if m.is_boss() {
            run.boss_kills.push((run.turn, kind.clone()));
            note(run, cx, format!("Slew the {}.", m.title()));
            callout(run, cx, "boss down");
            if !run.hurt_since_boss && !run.trophies_run.contains(&"boss_untouched".to_string()) {
                run.trophies_run.push("boss_untouched".into());
                note(run, cx, "Trophy: boss untouched.".into());
            }
        } else if m.grudge {
            note(run, cx, format!("{} is avenged.", m.title()));
        }
    }
    if let Some(it) = m.stolen.clone() {
        run.items.push(crate::engine::FloorItem { pos, item: it });
    }
    if m.has_tag("gas") {
        let (r, ttl) = if m.is_boss() { (2, 60) } else { (1, 40) };
        place_overlay(run, cx, pos, r, OverlayKind::Gas, ttl);
        if visible {
            callout(run, cx, "pops!");
            learn_tag(run, cx, &kind, "gas");
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

fn tick_poison(run: &mut Run, cx: &mut Ctx) {
    if run.hero.poison.1 > 0 {
        let d = run.hero.poison.0;
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

fn tick_statuses(run: &mut Run, cx: &mut Ctx) {
    if run.hero.poison.1 > 0 {
        run.hero.poison.1 -= 1;
    }
    run.hero.tick_statuses();
    if run.taunt_t > 0 {
        run.taunt_t -= 1;
    }
    for mi in 0..run.monsters.len() {
        if run.monsters[mi].poison.1 > 0 {
            run.monsters[mi].poison.1 -= 1;
        }
        run.monsters[mi].tick_statuses();
        if run.monsters[mi].ttl.is_some_and(|t| t <= 0) && run.monsters[mi].hp > 0 {
            run.monsters[mi].hp = 0;
            let id = run.monsters[mi].id;
            if run.monsters[mi].ally {
                cx.events.push(Ev::Ally { t: run.turn, id, state: "lost".into() });
            }
            cx.events.push(Ev::Die { t: run.turn, id, cause: "faded".into() });
        }
    }
}

/// Rests per alert step: resting is not free. From alert 5 a wandering pack comes for the
/// hero (spawned out of sight, nearby).
pub const REST_ALERT_EVERY: u32 = 8;

/// Resting raises the alert every `REST_ALERT_EVERY` rests; from alert 5 a pack comes.
pub fn rest_clock(run: &mut Run, cx: &mut Ctx) {
    run.rests += 1;
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
        let mut m = Monster::spawn(id, kind, pos, depth);
        m.awake = true;
        m.last_seen = Some(hero);
        let e = crate::engine::monster_entity(&m, cx.facts);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e });
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
        let mut m = Monster::spawn(id, kind, pos, depth);
        m.awake = true;
        m.last_seen = Some(hero);
        let e = crate::engine::monster_entity(&m, cx.facts);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e });
    }
    if run.alert == 3 || run.alert == 6 {
        callout(run, cx, "alert rising");
        learn(run, cx, "alert:rising".into());
    }
}

/// Go down a floor (or reach the ending).
pub fn descend(run: &mut Run, cx: &mut Ctx) {
    if let Some(t) = run.low10_t.take() {
        run.near_deaths.push(t);
    }
    run.low20_t = None;
    let floor_gambles: Vec<(u32, String, bool)> = run.gambles.clone();
    for (t, k, mal) in floor_gambles {
        if mal && !run.gambles_survived.iter().any(|(gt, _)| *gt == t) {
            run.gambles_survived.push((t, k));
        }
    }
    if run.kills_floor == 0 && run.depth >= 2 && !run.trophies_run.contains(&"pacifist_floor".to_string()) {
        run.trophies_run.push("pacifist_floor".into());
        note(run, cx, "Trophy: pacifist floor.".into());
    }
    let next = run.depth + 1;
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
    let biome = biome_for(next);
    let floor = generate(&mut run.rng, biome, next);
    run.depth = next;
    run.max_depth = run.max_depth.max(next);
    run.floor = floor;
    run.hero.pos = run.floor.stairs_up;
    run.hero_dist_pos = None;
    run.monsters.retain(|m| m.ally && m.hp > 0);
    let up = run.floor.stairs_up;
    let allies = std::mem::take(&mut run.monsters);
    for mut m in allies {
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
    run.rests = 0;
    run.chase = None;
    run.recent_pos.clear();
    run.stuck_until = 0;
    run.items_until = 0;
    run.pickup_streak = 0;
    run.gambles.clear();
    run.hero.second_wind_used = false;
    populate_floor(run, cx.grudges, cx.forge);
    crate::engine::place_bones(run);
    run.floor.map.update_vision(run.hero.pos, VISION);
    cx.events.push(Ev::Descend { t: run.turn, depth: next, biome: biome.name().into() });
    if biome_for(next - 1) != biome {
        learn(run, cx, format!("biome:{}", biome.name()));
    }
    note(run, cx, format!("D{}: {}.", next, biome.title()));
    if next == 5 {
        if !run.drank_heal && !run.trophies_run.contains(&"no_heal_D5".to_string()) {
            run.trophies_run.push("no_heal_D5".into());
            note(run, cx, "Trophy: no heal to D5.".into());
        }
        if !run.melee_used && !run.trophies_run.contains(&"ranged_only_D5".to_string()) {
            run.trophies_run.push("ranged_only_D5".into());
            note(run, cx, "Trophy: ranged only to D5.".into());
        }
    }
    crate::facts::on_vision(run, cx);
}

pub fn end_run(run: &mut Run, cx: &mut Ctx, tier: ExitTier) {
    if run.over.is_some() {
        return;
    }
    run.over = Some(tier);
    let loot_kept = if run.timed_out { 0 } else { run.loot * tier.pct() / 100 };
    cx.events.push(Ev::Exit { t: run.turn, tier: tier.name().into(), loot_kept });
    match tier {
        ExitTier::Bank => note(run, cx, format!("Banked {loot_kept} loot.")),
        ExitTier::Return => note(run, cx, format!("Returned with {loot_kept} loot.")),
        ExitTier::Death => {}
    }
}

/// Pick up whatever lies on the hero's tile.
pub fn pickup_here(run: &mut Run, cx: &mut Ctx) {
    // Several items may share a tile (a recovered kit that did not fit): take the first
    // that would change anything.
    let here = run.hero.pos;
    let Some(ii) = run.items.iter().position(|fi| fi.pos == here && can_take(&run.hero, &fi.item)) else { return };
    let item = &run.items[ii].item;
    if item.kind == "bones" {
        recover_bones(run, cx, ii);
        return;
    }
    if item.cat() == Cat::Gold {
        let it = run.items.remove(ii).item;
        run.loot += it.amount;
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: format!("gold ({})", it.amount) });
        return;
    }
    if item.kind == "leash" {
        let it = run.items.remove(ii).item;
        match run.hero.inv.iter_mut().find(|i| i.kind == "leash") {
            Some(l) => l.amount += it.amount.max(1),
            None => {
                if run.hero.inv_full() {
                    run.items.push(crate::engine::FloorItem { pos: run.hero.pos, item: it });
                    return;
                }
                let mut l = it;
                l.amount = l.amount.max(1);
                run.hero.inv.push(l);
            }
        }
        run.loot += 5;
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: "leash".into() });
        learn(run, cx, "item:leash".into());
        return;
    }
    if run.hero.inv_full() && !item_replaces_gear(&run.hero, item) {
        // A full pack swaps its cheapest consumable for a dearer one (a chore, silently).
        let swap = run.hero.inv.iter().enumerate().filter(|(_, i)| i.is_consumable()).min_by_key(|(_, i)| (i.value(), i.id)).map(|(k, i)| (k, i.value()));
        match swap {
            Some((k, v)) if item.is_consumable() && item.value() > v => {
                let dropped = run.hero.inv.remove(k);
                let here = run.hero.pos;
                let it = run.items.remove(ii).item;
                let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
                run.loot += it.value() - dropped.value();
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
    run.loot += it.value();
    run.hero.auto_equip(it);
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
    for it in pile.items {
        let value = it.value();
        let cat = it.cat();
        let replaced = run.hero.auto_equip(it.clone());
        let taken = match cat {
            Cat::Weapon => run.hero.weapon.as_ref().is_some_and(|w| w.id == it.id) || run.hero.inv.iter().any(|i| i.id == it.id),
            Cat::Armour => run.hero.armour.as_ref().is_some_and(|a| a.id == it.id) || run.hero.inv.iter().any(|i| i.id == it.id),
            _ => run.hero.inv.iter().any(|i| i.id == it.id),
        };
        if taken {
            run.loot += value;
        } else {
            drop_near(run, here, it);
        }
        if let Some(old) = replaced {
            if !run.hero.inv.iter().any(|i| i.id == old.id) {
                drop_near(run, here, old);
            }
        }
    }
    cx.events.push(Ev::Bones { t: run.turn, heir, items: n });
    note(run, cx, format!("Found heir {heir}'s bones: {n} items."));
    callout(run, cx, "bones");
}

/// Drop an item on the nearest free floor tile around `at` (its own tile if none).
fn drop_near(run: &mut Run, at: Pos, it: Item) {
    let free = std::iter::once(at)
        .chain(at.neighbours8())
        .find(|q| run.floor.map.in_bounds(*q) && run.floor.map.get(*q) == Tile::Floor && run.item_at(*q).is_none());
    run.items.push(crate::engine::FloorItem { pos: free.unwrap_or(at), item: it });
}

/// Would picking this up change anything (gold, leash, room in the pack, or better gear)?
pub fn can_take(h: &crate::hero::Hero, item: &Item) -> bool {
    if item.kind == "trap" {
        return false;
    }
    if item.kind == "bones" {
        return true;
    }
    matches!(item.cat(), Cat::Gold)
        || (item.kind == "leash" && h.inv.iter().any(|i| i.kind == "leash"))
        || !h.inv_full()
        || item_replaces_gear(h, item)
        || (item.is_consumable() && h.inv.iter().filter(|i| i.is_consumable()).map(|i| i.value()).min().is_some_and(|v| item.value() > v))
}

fn item_replaces_gear(h: &crate::hero::Hero, item: &Item) -> bool {
    match item.cat() {
        Cat::Weapon => {
            let cur = h.weapon.as_ref().map(|w| w.atk().0 + w.atk().1).unwrap_or(0);
            item.atk().0 + item.atk().1 > cur
        }
        Cat::Armour => item.def_bonus() > h.armour.as_ref().map(|a| a.def_bonus()).unwrap_or(0),
        _ => false,
    }
}

pub fn monster_kind_title(kind: &str) -> String {
    monster_def(kind).title.to_string()
}
