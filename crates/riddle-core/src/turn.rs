//! The turn: hero action (rules → verbs → chores), monsters, overlays, clocks, vision.
use crate::ai;
use crate::defs::{monster_def, Cat};
use crate::descent::{biome_for, ENDING_DEPTH};
use crate::engine::{populate_floor, Ctx, ExitTier, Run, HERO_ID};
use crate::facts::{learn, learn_tag, tag_known};
use crate::gen::generate;
use crate::geom::{Pos, DIRS8};
use crate::hero::Trait;
use crate::item::{is_identified, Item};
use crate::monster::Monster;
use crate::rules::{Cond, Verb};
use crate::tiles::{Overlay, OverlayKind, Tile, VISION};
use crate::wire::{Ev, TraceTurn};
use crate::chronicle::{callout, note};

/// What the hero can see this action.
#[derive(Clone, Debug, Default)]
pub struct View {
    /// Visible hostile monster indices, nearest first.
    pub foes: Vec<usize>,
    pub adj: i32,
    pub nearest: Option<usize>,
    pub lowest: Option<usize>,
}

pub fn view(run: &Run) -> View {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let mut foes: Vec<usize> = (0..run.monsters.len())
        .filter(|&i| {
            let m = &run.monsters[i];
            m.hp > 0 && m.hostile() && map.is_visible(m.pos)
        })
        .collect();
    foes.sort_by_key(|&i| (run.monsters[i].pos.cheb(hp), run.monsters[i].id));
    let adj = foes.iter().filter(|&&i| run.monsters[i].pos.adjacent(hp)).count() as i32;
    let nearest = foes.first().copied();
    let lowest = foes.iter().copied().min_by_key(|&i| (run.monsters[i].hp, run.monsters[i].id));
    View { foes, adj, nearest, lowest }
}

pub fn step_turn(run: &mut Run, cx: &mut Ctx) {
    if run.over.is_some() {
        return;
    }
    run.turn += 1;
    run.floor_turn += 1;
    run.floor.map.update_vision(run.hero.pos, VISION);
    // Hero.
    run.hero.energy += run.hero.speed();
    let mut acted = false;
    while run.hero.energy >= 10 && run.over.is_none() {
        run.hero.energy -= 10;
        hero_action(run, cx);
        acted = true;
        run.floor.map.update_vision(run.hero.pos, VISION);
        crate::facts::on_vision(run, cx);
    }
    if !acted {
        let v = view(run);
        push_trace(run, -2, Verb::new("wait"), &v);
    }
    if run.over.is_some() {
        return;
    }
    run.monsters.retain(|m| m.hp > 0);
    // Monsters, on a shared distance field from the hero.
    let hero_dist = run.floor.map.bfs(run.hero.pos, false, &|_| false);
    let n = run.monsters.len();
    for mi in 0..n {
        if run.over.is_some() {
            return;
        }
        ai::monster_turn(run, cx, mi, &hero_dist);
    }
    run.monsters.retain(|m| m.hp > 0);
    if run.over.is_some() {
        return;
    }
    tick_overlays(run, cx);
    if run.over.is_some() {
        return;
    }
    tick_statuses(run, cx);
    if run.over.is_some() {
        return;
    }
    tick_alert(run, cx);
    run.monsters.retain(|m| m.hp > 0);
    run.floor.map.update_vision(run.hero.pos, VISION);
    crate::facts::on_vision(run, cx);
}

pub fn push_trace(run: &mut Run, row: i32, verb: Verb, v: &View) {
    let telegraphs: Vec<String> = v
        .foes
        .iter()
        .filter_map(|&i| run.monsters[i].telegraph.as_ref().map(|t| format!("{} {}", run.monsters[i].kind, t)))
        .collect();
    run.trace.push(TraceTurn { t: run.turn, row, verb, hp: run.hero.hp, foes: v.foes.len() as i32, telegraphs });
    if run.trace.len() > 40 {
        run.trace.remove(0);
    }
}

fn hero_action(run: &mut Run, cx: &mut Ctx) {
    let v = view(run);
    let hp_before = run.hero.hp;
    let (row, verb) = choose_and_act(run, cx, &v);
    // Trace records the state at the start of the action.
    let telegraphs: Vec<String> = v
        .foes
        .iter()
        .filter_map(|&i| run.monsters.get(i).and_then(|m| m.telegraph.as_ref().map(|t| format!("{} {}", m.kind, t))))
        .collect();
    run.trace.push(TraceTurn { t: run.turn, row, verb, hp: hp_before, foes: v.foes.len() as i32, telegraphs });
    if run.trace.len() > 40 {
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
        return (-2, Verb::new("paralysed"));
    }
    if run.hero.confused > 0 && run.rng.chance(50) {
        ai::random_step(run, cx);
        return (-2, Verb::new("stumble"));
    }
    let foes = v.foes.len() as i32;
    let hp_pct = run.hero.hp_pct();
    let tr = run.trait_;
    // Trait deviations, announced.
    if tr == Trait::Cowardly && hp_pct < 50 && foes >= 1 {
        let verb = Verb::new("retreat");
        if ai::try_verb(run, cx, &verb, v) {
            emit_rule(run, cx, -1, &verb, "cowardly → retreat");
            return (-1, verb);
        }
    }
    if tr == Trait::Greedy {
        let hp = run.hero.pos;
        let target = DIRS8
            .iter()
            .map(|d| hp.add(*d))
            .find(|q| run.item_at(*q).is_some() && !run.occupied(*q) && run.floor.map.can_step(hp, *q));
        if let Some(q) = target {
            ai::move_hero(run, cx, q);
            let verb = Verb::new("pick_up");
            emit_rule(run, cx, -1, &verb, "greedy → pick up");
            return (-1, verb);
        }
    }
    let rows: Vec<crate::rules::Row> = cx.rules.rows.iter().take(cx.max_rows).cloned().collect();
    let mut brave_said = false;
    for (i, row) in rows.iter().enumerate() {
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
            if matches!(row.verb.v.as_str(), "recall" | "send") {
                continue; // party orders are free actions
            }
            return (i as i32, row.verb.clone());
        }
    }
    if tr == Trait::Curious && foes == 0 && hp_pct >= 50 {
        if let Some(verb) = ai::curious_use(run, cx) {
            emit_rule(run, cx, -1, &verb, &format!("curious → {}", verb.short()));
            return (-1, verb);
        }
    }
    let verb = ai::chore(run, cx, v);
    emit_rule(run, cx, -2, &verb, &format!("chore → {}", verb.short()));
    (-2, verb)
}

pub fn emit_rule(run: &Run, cx: &mut Ctx, row: i32, verb: &Verb, text: &str) {
    cx.events.push(Ev::Rule { t: run.turn, row, verb: verb.clone(), text: crate::chronicle::clamp_words(text, 3) });
}

pub fn cond_holds(run: &Run, cx: &Ctx, v: &View, c: &Cond) -> bool {
    let n = c.n.unwrap_or(0);
    let t = c.t.as_deref().unwrap_or("");
    let h = &run.hero;
    match c.k.as_str() {
        "hp<" => h.hp_pct() < n,
        "hp>" => h.hp_pct() > n,
        "foes>=" => v.foes.len() as i32 >= n,
        "adj>=" => v.adj >= n,
        "foe_tag" => v.foes.iter().any(|&i| {
            let m = &run.monsters[i];
            m.has_tag(t) && tag_known(cx.facts, &m.kind, t)
        }),
        "foe_hp<" => v.nearest.is_some_and(|i| {
            let m = &run.monsters[i];
            m.hp * 100 / m.max_hp.max(1) < n
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
        "turns>" => run.floor_turn as i32 > n,
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
    pub fn tags(&self, run: &Run) -> Vec<String> {
        match self {
            Src::Hero { ranged } => {
                if *ranged {
                    vec!["ranged".into()]
                } else {
                    vec![]
                }
            }
            Src::Mon(i) => run.monsters[*i].tags(),
            Src::Gas | Src::Burst => vec!["gas".into()],
            Src::Fire => vec!["fire".into()],
            Src::Poison => vec!["poison".into()],
        }
    }
}

/// Is an entity alone (no friend within 2 tiles)?
pub fn is_lone(run: &Run, pos: Pos, hostile: bool) -> bool {
    if hostile {
        !run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.pos != pos && m.pos.cheb(pos) <= 2)
    } else {
        let hero_near = run.hero.pos != pos && run.hero.pos.cheb(pos) <= 2;
        !hero_near && !run.monsters.iter().any(|m| m.hp > 0 && m.ally && m.pos != pos && m.pos.cheb(pos) <= 2)
    }
}

/// Apply the counter table (Addendum A): returns the adjusted damage and the counter observed.
pub fn counter_damage(run: &Run, src: &Src, dmg: i32, target_tags: &[String], target_pos: Pos, target_hostile: bool) -> (i32, Option<(String, String)>) {
    let atk = src.tags(run);
    let on_water = run.floor.map.get(target_pos) == Tile::Water;
    for (a, b, immune) in crate::defs::COUNTERS {
        let (hit, winner_is_attacker) = if *immune {
            // defender's tag beats the attack's tag
            let def_has = target_tags.iter().any(|t| t == a) || (*a == "water" && on_water);
            (def_has && atk.iter().any(|t| t == b), false)
        } else {
            let tgt = if *b == "lone" { is_lone(run, target_pos, target_hostile) } else { target_tags.iter().any(|t| t == b) };
            (atk.iter().any(|t| t == a) && tgt, true)
        };
        if hit {
            let _ = winner_is_attacker;
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
    let (dmg, counter) = counter_damage(run, src, dmg, &[], run.hero.pos, false);
    if let Some((a, b)) = counter {
        learn(run, cx, crate::defs::counter_fact(&a, &b));
    }
    if dmg <= 0 {
        return;
    }
    run.hero.hp -= dmg;
    run.hurt_since_action = true;
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
    let cause = src.cause(run);
    let cause = cause.as_str();
    let dmg = dmg.max(0);
    let (dmg, counter) = {
        let m = &run.monsters[mi];
        counter_damage(run, src, dmg, &m.tags(), m.pos, m.hostile())
    };
    if let Some((a, b)) = counter {
        if run.floor.map.is_visible(run.monsters[mi].pos) {
            learn(run, cx, crate::defs::counter_fact(&a, &b));
        }
    }
    if run.monsters[mi].ally {
        run.monsters[mi].hurt_since_action = true;
    }
    run.monsters[mi].hp -= dmg;
    let (id, hp, kind, pos) = {
        let m = &run.monsters[mi];
        (m.id, m.hp, m.kind.clone(), m.pos)
    };
    cx.events.push(Ev::Hurt { t: run.turn, id, dmg, hp: hp.max(0), cause: cause.into() });
    let visible = run.floor.map.is_visible(pos);
    if hp > 0 {
        if run.monsters[mi].has_tag("splitter") && hp > 4 && dmg > 0 {
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
        run.kills.push((run.turn, kind.clone()));
        run.kills_floor += 1;
        run.kill_since_action = true;
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
    if let Some(it) = m.stolen {
        run.items.push(crate::engine::FloorItem { pos, item: it });
    }
    if m.has_tag("gas") {
        let (r, ttl) = if m.is_boss() { (2, 6) } else { (1, 4) };
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
            let p = centre.add((dx, dy));
            if !run.floor.map.in_bounds(p) || !run.floor.map.passable(p) {
                continue;
            }
            if let Some(o) = run.overlays.iter_mut().find(|o| o.x == p.x && o.y == p.y) {
                o.k = k;
                o.ttl = o.ttl.max(ttl);
                continue;
            }
            run.overlays.push(Overlay { x: p.x, y: p.y, k, ttl, spread: k == OverlayKind::Fire });
            cx.events.push(Ev::Overlay { t: run.turn, x: p.x, y: p.y, ov: k, ttl });
        }
    }
}

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
            damage_monster(run, cx, mi, dmg, &src);
        }
    }
    // Fire spreads once to adjacent floor.
    let spreading: Vec<Overlay> = run.overlays.iter().filter(|o| o.spread && o.k == OverlayKind::Fire).cloned().collect();
    for o in spreading {
        for d in DIRS8 {
            let q = Pos::new(o.x, o.y).add(d);
            if run.floor.map.get(q) == Tile::Floor && !run.overlays.iter().any(|x| x.x == q.x && x.y == q.y) {
                let ttl = (o.ttl - 1).max(1);
                run.overlays.push(Overlay { x: q.x, y: q.y, k: OverlayKind::Fire, ttl, spread: false });
                cx.events.push(Ev::Overlay { t: run.turn, x: q.x, y: q.y, ov: OverlayKind::Fire, ttl });
            }
        }
    }
    for o in run.overlays.iter_mut() {
        o.spread = false;
        o.ttl -= 1;
    }
    run.overlays.retain(|o| o.ttl > 0);
}

fn tick_statuses(run: &mut Run, cx: &mut Ctx) {
    if run.hero.poison.1 > 0 {
        let d = run.hero.poison.0;
        run.hero.poison.1 -= 1;
        damage_hero(run, cx, d, &Src::Poison);
        if run.over.is_some() {
            return;
        }
    }
    run.hero.tick_statuses();
    for mi in 0..run.monsters.len() {
        if run.monsters[mi].poison.1 > 0 {
            let d = run.monsters[mi].poison.0;
            run.monsters[mi].poison.1 -= 1;
            damage_monster(run, cx, mi, d, &Src::Poison);
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

/// The forward clock: every 30 turns on a floor the alert rises and a wanderer arrives.
fn tick_alert(run: &mut Run, cx: &mut Ctx) {
    if run.floor_turn % 30 != 0 || run.alert >= 8 {
        return;
    }
    run.alert += 1;
    let table = crate::defs::spawn_table(run.biome(), run.depth);
    let weights: Vec<u32> = table.iter().map(|t| if t.0 == "captive" || t.0 == "eel" { 0 } else { t.1 }).collect();
    let (kind, ..) = table[run.rng.weighted(&weights)];
    let hero = run.hero.pos;
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
    if run.alert == 3 || run.alert == 6 {
        callout(run, cx, "alert rising");
    }
}

/// Go down a floor (or reach the ending).
pub fn descend(run: &mut Run, cx: &mut Ctx) {
    // Floor survived bookkeeping for the sifter.
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
    run.gambles.clear();
    populate_floor(run, cx.grudges);
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
    let loot_kept = run.loot * tier.pct() / 100;
    cx.events.push(Ev::Exit { t: run.turn, tier: tier.name().into(), loot_kept });
    match tier {
        ExitTier::Bank => note(run, cx, format!("Banked {loot_kept} loot.")),
        ExitTier::Return => note(run, cx, format!("Returned with {loot_kept} loot.")),
        ExitTier::Death => {}
    }
}

/// Pick up whatever lies on the hero's tile.
pub fn pickup_here(run: &mut Run, cx: &mut Ctx) {
    let Some(ii) = run.item_at(run.hero.pos) else { return };
    let item = &run.items[ii].item;
    if item.cat() == Cat::Gold {
        let it = run.items.remove(ii).item;
        run.loot += it.amount;
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: format!("gold ({})", it.amount) });
        return;
    }
    if item.kind == "leash" {
        let it = run.items.remove(ii).item;
        run.loot += it.value();
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
        cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: "leash".into() });
        learn(run, cx, "item:leash".into());
        return;
    }
    if run.hero.inv_full() && !item_replaces_gear(&run.hero, item) {
        return;
    }
    let it = run.items.remove(ii).item;
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    run.loot += it.value();
    run.hero.auto_equip(it);
    cx.events.push(Ev::Pickup { t: run.turn, id: HERO_ID, item: label });
}

fn item_replaces_gear(h: &crate::hero::Hero, item: &Item) -> bool {
    match item.cat() {
        Cat::Weapon => {
            let cur = h.weapon.as_ref().map(|w| w.atk().0 + w.atk().1).unwrap_or(0);
            item.atk().0 + item.atk().1 > cur
        }
        Cat::Armour => item.def_bonus() > h.def(),
        _ => false,
    }
}

pub fn monster_kind_title(kind: &str) -> String {
    monster_def(kind).title.to_string()
}
