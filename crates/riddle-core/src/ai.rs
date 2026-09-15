//! Hero verbs (with real pathing) and tag-faithful monster AI.
use crate::chronicle::{callout, note};
use crate::defs::Cat;
use crate::engine::{Ctx, ExitTier, Run, HERO_ID};
use crate::facts::{learn, learn_tag};
use crate::geom::{Pos, DIRS8};
use crate::hero::Class;
use crate::item::{ident_fact, is_identified, Hint, Item};
use crate::monster::{Monster, Pending};
use crate::rules::Verb;
use crate::tiles::{OverlayKind, Tile, VISION};
use crate::turn::{cond_holds, damage_hero, damage_monster, descend, end_run, pickup_here, place_overlay, view, Src, View};
use crate::wire::Ev;

pub const THROW_RANGE: i32 = 6;
pub const BOW_RANGE: i32 = 6;

/// 80% to hit; damage = roll(atk) − def, min 0.
pub fn roll_hit(rng: &mut crate::rng::Rng, atk: (i32, i32), def: i32) -> (bool, i32) {
    let hit = rng.chance(80);
    let roll = rng.range(atk.0, atk.1);
    (hit, if hit { (roll - def).max(0) } else { 0 })
}

// ---------------------------------------------------------------- hero movement

pub fn move_hero(run: &mut Run, cx: &mut Ctx, q: Pos) {
    run.hero.pos = q;
    cx.events.push(Ev::Move { t: run.turn, id: HERO_ID, x: q.x, y: q.y });
    pickup_here(run, cx);
}

pub fn random_step(run: &mut Run, cx: &mut Ctx) {
    let hp = run.hero.pos;
    let cands: Vec<Pos> = DIRS8.iter().map(|d| hp.add(*d)).filter(|q| run.floor.map.can_step(hp, *q) && !run.occupied(*q)).collect();
    if cands.is_empty() {
        return;
    }
    let q = *run.rng.pick(&cands);
    move_hero(run, cx, q);
}

/// BFS from the hero over seen tiles; visible monsters block.
fn hero_bfs(run: &Run) -> (Vec<i32>, Vec<i32>) {
    let map = &run.floor.map;
    map.bfs_parent(run.hero.pos, true, &|p| run.monster_at(p).is_some_and(|mi| map.is_visible(run.monsters[mi].pos)))
}

fn step_towards(run: &mut Run, cx: &mut Ctx, goal: Pos, parent: &[i32]) -> bool {
    let hp = run.hero.pos;
    if let Some(q) = run.floor.map.first_step(parent, hp, goal) {
        if !run.occupied(q) {
            move_hero(run, cx, q);
            return true;
        }
    }
    false
}

/// Nearest reachable tile satisfying `pred` (by BFS distance), with the parent map.
fn nearest_tile(run: &Run, pred: &dyn Fn(Pos) -> bool) -> Option<(Pos, Vec<i32>)> {
    let (dist, parent) = hero_bfs(run);
    let map = &run.floor.map;
    let mut best: Option<(i32, Pos)> = None;
    for (i, d) in dist.iter().enumerate() {
        if *d <= 0 {
            continue;
        }
        let p = map.pos(i);
        if pred(p) && best.is_none_or(|(bd, _)| *d < bd) {
            best = Some((*d, p));
        }
    }
    best.map(|(_, p)| (p, parent))
}

fn is_frontier(run: &Run, p: Pos) -> bool {
    let map = &run.floor.map;
    if !map.is_seen(p) || !map.passable(p) {
        return false;
    }
    DIRS8.iter().any(|d| {
        let q = p.add(*d);
        map.in_bounds(q) && !map.is_seen(q)
    })
}

/// Explore toward the nearest frontier. Returns false when the floor is fully explored.
pub fn explore_step(run: &mut Run, cx: &mut Ctx) -> bool {
    if let Some((goal, parent)) = nearest_tile(run, &|p| is_frontier(run, p)) {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

fn nearest_item_step(run: &mut Run, cx: &mut Ctx, only_adjacent_free: bool) -> bool {
    let cands: Vec<Pos> = run
        .items
        .iter()
        .filter(|fi| run.floor.map.is_seen(fi.pos) && !(run.hero.inv_full() && fi.item.cat() != Cat::Gold))
        .map(|fi| fi.pos)
        .collect();
    if cands.is_empty() {
        return false;
    }
    let _ = only_adjacent_free;
    if let Some((goal, parent)) = nearest_tile(run, &|p| cands.contains(&p)) {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

/// Engine chore: items → explore → descend. Returns the verb performed.
pub fn chore(run: &mut Run, cx: &mut Ctx, v: &View) -> Verb {
    if v.adj == 0 && nearest_item_step(run, cx, false) {
        return Verb::new("pick_up");
    }
    if explore_step(run, cx) {
        return Verb::new("explore");
    }
    if descend_step(run, cx) {
        return Verb::new("descend");
    }
    if nearest_item_step(run, cx, false) {
        return Verb::new("pick_up");
    }
    Verb::new("wait")
}

fn descend_step(run: &mut Run, cx: &mut Ctx) -> bool {
    let s = run.floor.stairs_down;
    if run.hero.pos == s {
        descend(run, cx);
        return true;
    }
    if !run.floor.map.is_seen(s) {
        return false;
    }
    let (_, parent) = hero_bfs(run);
    step_towards(run, cx, s, &parent)
}

// ---------------------------------------------------------------- hero verbs

/// Execute a verb if it can execute now. Returns false to fall through.
pub fn try_verb(run: &mut Run, cx: &mut Ctx, verb: &Verb, v: &View) -> bool {
    try_verb_scoped(run, cx, verb, v, None)
}

/// As `try_verb`, with the row's `party:<kind>` scope for `recall`/`send`.
pub fn try_verb_scoped(run: &mut Run, cx: &mut Ctx, verb: &Verb, v: &View, scope: Option<&str>) -> bool {
    let a = verb.a.clone().unwrap_or_default();
    match verb.v.as_str() {
        "tame" => cx.unlocks.contains("tame") && verb_tame(run, cx, &a, v),
        "recall" => {
            let mut any = false;
            for mi in 0..run.monsters.len() {
                let m = &run.monsters[mi];
                if m.is_companion() && m.hp > 0 && scope.is_none_or(|k| m.kind == k) {
                    recall_companion(run, cx, mi);
                    any = true;
                }
            }
            any
        }
        "send" => {
            let mut any = false;
            if v.foes.is_empty() {
                return false;
            }
            for m in run.monsters.iter_mut() {
                if m.is_companion() && m.hp > 0 && !m.sent && scope.is_none_or(|k| m.kind == k) {
                    m.sent = true;
                    any = true;
                }
            }
            if any {
                callout(run, cx, "sic!");
            }
            any
        }
        "attack" => verb_attack(run, cx, &a, v, false),
        "shield_bash" => run.hero.class == Class::Fighter && run.hero.bash_cd == 0 && verb_attack(run, cx, "nearest", v, true),
        "retreat" => verb_retreat(run, cx, v),
        "back_corridor" => verb_back_corridor(run, cx, v),
        "drink" => verb_drink(run, cx, &a),
        "read" => verb_read(run, cx, &a, v),
        "throw" => (run.hero.class == Class::Rogue || cx.unlocks.contains("throw")) && verb_throw(run, cx, &a, v),
        "descend" => {
            if run.hero.pos == run.floor.stairs_down {
                descend(run, cx);
                true
            } else if descend_step(run, cx) {
                true
            } else {
                explore_step(run, cx)
            }
        }
        "bank" => {
            let s = run.floor.stairs_up;
            if run.hero.pos == s {
                end_run(run, cx, ExitTier::Bank);
                true
            } else {
                let (_, parent) = hero_bfs(run);
                step_towards(run, cx, s, &parent)
            }
        }
        "return" => {
            end_run(run, cx, ExitTier::Return);
            true
        }
        "rest" => {
            if run.hero.hp < run.hero.max_hp && v.foes.is_empty() && run.hero.poison.1 == 0 {
                run.hero.hp = (run.hero.hp + 1).min(run.hero.max_hp);
                true
            } else {
                false
            }
        }
        "pick_up" => nearest_item_step(run, cx, false),
        "free_captive" => verb_free_captive(run, cx),
        "vanish" => {
            if run.hero.class == Class::Rogue && run.hero.vanish_cd == 0 && !v.foes.is_empty() {
                run.hero.vanish_t = 3;
                run.hero.vanish_cd = 12;
                for m in run.monsters.iter_mut() {
                    m.last_seen = None;
                }
                callout(run, cx, "vanish");
                true
            } else {
                false
            }
        }
        "tactic" => verb_tactic(run, cx, &a, v),
        _ => false,
    }
}

fn pick_target(run: &Run, a: &str, v: &View) -> Option<usize> {
    match a {
        "" | "nearest" => v.nearest,
        "lowest" => v.lowest,
        s if s.starts_with("tag:") => {
            let t = &s[4..];
            v.foes.iter().copied().find(|&i| run.monsters[i].has_tag(t))
        }
        _ => v.nearest,
    }
}

fn verb_attack(run: &mut Run, cx: &mut Ctx, a: &str, v: &View, bash: bool) -> bool {
    let Some(mi) = pick_target(run, a, v) else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.adjacent(hp) {
        hero_attack(run, cx, mi, if bash { "shield_bash" } else { "attack" }, bash);
        return true;
    }
    if bash {
        return false;
    }
    if run.hero.ranged() && mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
        hero_attack(run, cx, mi, "shoot", false);
        return true;
    }
    // Approach: path to a tile adjacent to the target.
    let (dist, parent) = hero_bfs(run);
    let map = &run.floor.map;
    let goal = mp
        .neighbours8()
        .into_iter()
        .filter(|q| map.in_bounds(*q) && dist[map.idx(*q)] >= 0)
        .min_by_key(|q| (dist[map.idx(*q)], q.x, q.y));
    match goal {
        Some(g) if g == hp => false,
        Some(g) => step_towards(run, cx, g, &parent),
        None => false,
    }
}

pub fn hero_attack(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &str, bash: bool) {
    let atk = run.hero.atk();
    let def = run.monsters[mi].effective_def();
    let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Attack { t: run.turn, src: HERO_ID, dst: id, dmg, hit, verb: Some(verb.into()) });
    if verb != "shoot" {
        run.melee_used = true;
    }
    if bash {
        run.hero.bash_cd = 5;
    }
    if hit {
        if bash {
            run.monsters[mi].stun = 1;
            callout(run, cx, "bash");
        }
        damage_monster(run, cx, mi, dmg, &Src::Hero { ranged: verb == "shoot" });
        if run.monsters[mi].hp > 0 && !run.monsters[mi].awake {
            run.monsters[mi].awake = true;
            run.monsters[mi].last_seen = Some(run.hero.pos);
        }
    }
}

fn min_foe_dist(run: &Run, v: &View, p: Pos) -> i32 {
    v.foes.iter().map(|&i| run.monsters[i].pos.cheb(p)).min().unwrap_or(99)
}

/// Move to maximise distance from foes, preferring corridors and stairs.
fn verb_retreat(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if v.foes.is_empty() {
        return false;
    }
    let hp = run.hero.pos;
    let cur = min_foe_dist(run, v, hp);
    let map = &run.floor.map;
    let mut best: Option<(i32, Pos)> = None;
    for d in DIRS8 {
        let q = hp.add(d);
        if !map.can_step(hp, q) || run.occupied(q) || run.overlays.iter().any(|o| o.x == q.x && o.y == q.y) {
            continue;
        }
        let md = min_foe_dist(run, v, q);
        if md < cur {
            continue;
        }
        let sum: i32 = v.foes.iter().map(|&i| run.monsters[i].pos.cheb(q)).sum();
        let score = md * 8 + sum + map.is_corridor(q) as i32 * 3 + matches!(map.get(q), Tile::StairsDown | Tile::StairsUp) as i32 * 2;
        let base = cur * 8 + v.foes.iter().map(|&i| run.monsters[i].pos.cheb(hp)).sum::<i32>();
        if score > base && best.is_none_or(|(bs, _)| score > bs) {
            best = Some((score, q));
        }
    }
    match best {
        Some((_, q)) => {
            move_hero(run, cx, q);
            true
        }
        None => false,
    }
}

fn verb_back_corridor(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let hp = run.hero.pos;
    if run.floor.map.is_corridor(hp) {
        return false;
    }
    let foes: Vec<Pos> = v.foes.iter().map(|&i| run.monsters[i].pos).collect();
    let target = nearest_tile(run, &|p| run.floor.map.is_corridor(p) && !foes.iter().any(|f| f.adjacent(p)));
    if let Some((goal, parent)) = target {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

fn find_consumable(run: &Run, cx: &Ctx, cat: Cat, a: &str) -> Option<usize> {
    if a == "unknown" || a.is_empty() {
        let mut best: Option<(i32, usize)> = None;
        for (i, it) in run.hero.inv.iter().enumerate() {
            if it.cat() != cat || is_identified(cx.facts, cx.flavours, &it.kind) {
                continue;
            }
            let rank = match it.hint {
                Some(Hint::Benevolent) => 0,
                None => 1,
                Some(Hint::Malevolent) => 2,
            };
            if best.is_none_or(|(r, _)| rank < r) {
                best = Some((rank, i));
            }
        }
        best.map(|(_, i)| i)
    } else {
        if !is_identified(cx.facts, cx.flavours, a) {
            return None;
        }
        run.hero.inv.iter().position(|it| it.cat() == cat && it.kind == a)
    }
}

fn identify_used(run: &mut Run, cx: &mut Ctx, item: &Item) -> bool {
    let was_unknown = !is_identified(cx.facts, cx.flavours, &item.kind);
    if let Some(f) = ident_fact(cx.flavours, &item.kind) {
        learn(run, cx, f);
    }
    if was_unknown {
        let mal = !item.def().benevolent;
        run.gambles.push((run.turn, item.kind.clone(), mal));
        let (_, _, label) = crate::item::describe(item, cx.facts, cx.flavours);
        note(run, cx, format!("Gambled: {label}."));
    }
    was_unknown
}

fn verb_drink(run: &mut Run, cx: &mut Ctx, a: &str) -> bool {
    let Some(ii) = find_consumable(run, cx, Cat::Potion, a) else { return false };
    let kind = run.hero.inv[ii].kind.clone();
    // Sanity: no drinking a known heal at full HP.
    if kind == "heal" && is_identified(cx.facts, cx.flavours, "heal") && run.hero.hp >= run.hero.max_hp {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    identify_used(run, cx, &item);
    let outcome = match kind.as_str() {
        "heal" => {
            let add = run.hero.max_hp / 2;
            run.hero.hp = (run.hero.hp + add).min(run.hero.max_hp);
            run.hero.poison = (0, 0);
            run.drank_heal = true;
            format!("+{add} HP")
        }
        "strength" => {
            run.hero.str_bonus += 1;
            "stronger".into()
        }
        "speed" => {
            run.hero.speed_t = 3;
            "fast".into()
        }
        "invisibility" => {
            run.hero.invis_t = 5;
            for m in run.monsters.iter_mut() {
                m.last_seen = None;
            }
            "unseen".into()
        }
        "poison" => {
            run.hero.poison = (2, 4);
            "poisoned".into()
        }
        "caustic" => {
            let p = run.hero.pos;
            place_overlay(run, cx, p, 1, OverlayKind::Gas, 3);
            "gas".into()
        }
        "confusion" => {
            run.hero.confused = 3;
            "confused".into()
        }
        "fire" => {
            let p = run.hero.pos;
            place_overlay(run, cx, p, 1, OverlayKind::Fire, 2);
            "fire".into()
        }
        _ => "nothing".into(),
    };
    cx.events.push(Ev::Use { t: run.turn, item: format!("{kind} potion"), outcome });
    true
}

fn scroll_useless(run: &Run, cx: &Ctx, kind: &str, v: &View) -> bool {
    match kind {
        "mapping" => run.floor.map.seen_pct() >= 95,
        "identify" => !run.hero.inv.iter().any(|i| i.is_consumable() && !is_identified(cx.facts, cx.flavours, &i.kind)),
        "enchant" => run.hero.weapon.is_none() && run.hero.armour.is_none(),
        "fear" | "darkness" | "teleport" | "blink" => v.foes.is_empty(),
        "summon_ally" => run.allies().next().is_some(),
        "aggravate" => true,
        _ => false,
    }
}

fn verb_read(run: &mut Run, cx: &mut Ctx, a: &str, v: &View) -> bool {
    let Some(ii) = find_consumable(run, cx, Cat::Scroll, a) else { return false };
    let kind = run.hero.inv[ii].kind.clone();
    if is_identified(cx.facts, cx.flavours, &kind) && scroll_useless(run, cx, &kind, v) {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    identify_used(run, cx, &item);
    let outcome = match kind.as_str() {
        "teleport" => {
            let hp = run.hero.pos;
            let cands: Vec<Pos> = run.floor.open_tiles().into_iter().filter(|p| p.cheb(hp) >= 8 && !run.occupied(*p)).collect();
            if !cands.is_empty() {
                let q = *run.rng.pick(&cands);
                move_hero(run, cx, q);
            }
            "teleported".into()
        }
        "blink" => {
            let hp = run.hero.pos;
            let map = &run.floor.map;
            let mut cands: Vec<Pos> = Vec::new();
            for dy in -3..=3 {
                for dx in -3..=3 {
                    let q = hp.add((dx, dy));
                    if q != hp && map.passable(q) && !run.occupied(q) && map.los(hp, q) {
                        cands.push(q);
                    }
                }
            }
            if !cands.is_empty() {
                let q = *cands.iter().max_by_key(|q| (min_foe_dist(run, v, **q), -q.cheb(hp), q.x, q.y)).unwrap();
                move_hero(run, cx, q);
            }
            "blinked".into()
        }
        "fear" => {
            for &i in &v.foes {
                run.monsters[i].fear = 5;
            }
            "foes flee".into()
        }
        "mapping" => {
            run.floor.map.reveal_all();
            "mapped".into()
        }
        "identify" => {
            let target = run.hero.inv.iter().position(|i| i.is_consumable() && !is_identified(cx.facts, cx.flavours, &i.kind));
            match target {
                Some(ti) => {
                    let k = run.hero.inv[ti].kind.clone();
                    if let Some(f) = ident_fact(cx.flavours, &k) {
                        learn(run, cx, f);
                    }
                    format!("{k} known")
                }
                None => "nothing".into(),
            }
        }
        "enchant" => {
            if let Some(w) = run.hero.weapon.as_mut() {
                w.enchant += 1;
                "weapon +1".into()
            } else if let Some(ar) = run.hero.armour.as_mut() {
                ar.enchant += 1;
                "armour +1".into()
            } else {
                "nothing".into()
            }
        }
        "darkness" => {
            for &i in &v.foes {
                run.monsters[i].blind = 4;
                run.monsters[i].last_seen = None;
            }
            "darkness".into()
        }
        "summon_ally" => {
            let hp = run.hero.pos;
            let free = hp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
            if let Some(q) = free {
                let id = run.new_id();
                let depth = run.depth;
                let mut m = Monster::spawn(id, "spectral_hound", q, depth);
                m.ally = true;
                m.awake = true;
                m.ttl = Some(20);
                let e = crate::engine::monster_entity(&m, cx.facts);
                run.monsters.push(m);
                cx.events.push(Ev::Spawn { t: run.turn, e });
                cx.events.push(Ev::Ally { t: run.turn, id, state: "freed".into() });
            }
            "hound".into()
        }
        "aggravate" => {
            let hp = run.hero.pos;
            for m in run.monsters.iter_mut() {
                if m.hostile() {
                    m.awake = true;
                    m.last_seen = Some(hp);
                }
            }
            callout(run, cx, "aggravated!");
            "aggravated".into()
        }
        _ => "nothing".into(),
    };
    cx.events.push(Ev::Use { t: run.turn, item: format!("{kind} scroll"), outcome });
    true
}

fn verb_throw(run: &mut Run, cx: &mut Ctx, a: &str, v: &View) -> bool {
    let mut parts = a.split(',');
    let kind = parts.next().unwrap_or("").trim();
    let target_sel = parts.next().unwrap_or("nearest").trim();
    let Some(ii) = find_consumable(run, cx, Cat::Potion, kind) else { return false };
    let Some(mi) = pick_target(run, target_sel, v) else { return false };
    let hp = run.hero.pos;
    let mut land = run.monsters[mi].pos;
    if land.cheb(hp) > THROW_RANGE || !run.floor.map.los(hp, land) {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    identify_used(run, cx, &item);
    let pkind = item.kind.clone();
    let mut reflected = false;
    if run.monsters[mi].has_tag("reflect") {
        land = hp;
        reflected = true;
        let k = run.monsters[mi].kind.clone();
        learn_tag(run, cx, &k, "reflect");
        callout(run, cx, "reflected!");
    }
    let victim = if reflected { None } else { Some(mi) };
    let outcome = match pkind.as_str() {
        "poison" => {
            if let Some(m) = victim {
                run.monsters[m].poison = (2, 4);
            } else {
                run.hero.poison = (2, 4);
            }
            "poisoned".into()
        }
        "caustic" => {
            place_overlay(run, cx, land, 1, OverlayKind::Gas, 4);
            "gas".into()
        }
        "confusion" => {
            if let Some(m) = victim {
                run.monsters[m].confused = 3;
            } else {
                run.hero.confused = 3;
            }
            "confused".into()
        }
        "fire" => {
            place_overlay(run, cx, land, 1, OverlayKind::Fire, 2);
            "fire".into()
        }
        "heal" => {
            if let Some(m) = victim {
                let add = run.monsters[m].max_hp / 2;
                run.monsters[m].hp = (run.monsters[m].hp + add).min(run.monsters[m].max_hp);
            }
            "healed foe".into()
        }
        "strength" => {
            if let Some(m) = victim {
                run.monsters[m].atk.1 += 1;
            }
            "wasted".into()
        }
        _ => "wasted".into(),
    };
    cx.events.push(Ev::Use { t: run.turn, item: format!("{pkind} potion"), outcome });
    if run.hero.class == Class::Rogue || cx.unlocks.contains("throw") {
        // thrown = ranged; melee_used untouched
    }
    true
}

fn verb_free_captive(run: &mut Run, cx: &mut Ctx) -> bool {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let Some(ci) = run.monsters.iter().position(|m| m.hp > 0 && m.neutral && map.is_visible(m.pos)) else { return false };
    let cp = run.monsters[ci].pos;
    if cp.adjacent(hp) {
        let m = &mut run.monsters[ci];
        m.neutral = false;
        m.ally = true;
        m.awake = true;
        let id = m.id;
        cx.events.push(Ev::Ally { t: run.turn, id, state: "freed".into() });
        run.ally_freed.push(run.turn);
        note(run, cx, "Freed the captive. It followed.".into());
        callout(run, cx, "freed");
        return true;
    }
    let (dist, parent) = hero_bfs(run);
    let map = &run.floor.map;
    let goal = cp.neighbours8().into_iter().filter(|q| map.in_bounds(*q) && dist[map.idx(*q)] >= 0).min_by_key(|q| (dist[map.idx(*q)], q.x, q.y));
    match goal {
        Some(g) => step_towards(run, cx, g, &parent),
        None => false,
    }
}

/// Curious trait: use an unknown item when safe (never a malevolent-hinted one).
pub fn curious_use(run: &mut Run, cx: &mut Ctx) -> Option<Verb> {
    let pick = run
        .hero
        .inv
        .iter()
        .position(|i| i.is_consumable() && !is_identified(cx.facts, cx.flavours, &i.kind) && i.hint != Some(Hint::Malevolent))?;
    let cat = run.hero.inv[pick].cat();
    let v = view(run);
    let verb = if cat == Cat::Potion { Verb::arg("drink", "unknown") } else { Verb::arg("read", "unknown") };
    let ok = if cat == Cat::Potion { verb_drink(run, cx, "unknown") } else { verb_read(run, cx, "unknown", &v) };
    if ok {
        Some(verb)
    } else {
        None
    }
}

/// Step out of the line of sight of `from`, if a neighbouring tile does that.
fn break_los_step(run: &mut Run, cx: &mut Ctx, from: Pos) -> bool {
    let hp = run.hero.pos;
    let map = &run.floor.map;
    let q = DIRS8.iter().map(|d| hp.add(*d)).find(|q| map.can_step(hp, *q) && !run.occupied(*q) && !map.los(from, *q));
    match q {
        Some(q) => {
            move_hero(run, cx, q);
            true
        }
        None => false,
    }
}

/// Tactic cards: a named bundle of sub-rows occupying one row.
fn verb_tactic(run: &mut Run, cx: &mut Ctx, card: &str, v: &View) -> bool {
    if !cx.unlocks.contains(card) {
        return false;
    }
    let foes = v.foes.len() as i32;
    let in_corr = run.floor.map.is_corridor(run.hero.pos);
    match card {
        "corridor_fighting" => {
            if foes >= 2 && !in_corr && verb_back_corridor(run, cx, v) {
                return true;
            }
            if in_corr && v.adj >= 1 {
                return verb_attack(run, cx, "nearest", v, false);
            }
            if in_corr && foes >= 1 {
                return true; // hold the corridor
            }
            foes >= 1 && verb_attack(run, cx, "nearest", v, false)
        }
        "kite_archers" => {
            let drawing = v.foes.iter().copied().find(|&i| run.monsters[i].telegraph.as_deref() == Some("draws") && !run.monsters[i].pos.adjacent(run.hero.pos));
            if let Some(i) = drawing {
                let from = run.monsters[i].pos;
                if break_los_step(run, cx, from) {
                    return true;
                }
            }
            if v.foes.iter().any(|&i| run.monsters[i].has_tag("ranged")) {
                return verb_attack(run, cx, "tag:ranged", v, false);
            }
            false
        }
        "stair_dance" => {
            let on_stairs = run.hero.pos == run.floor.stairs_down;
            if on_stairs && foes >= 1 && run.hero.hp_pct() < 50 {
                descend(run, cx);
                return true;
            }
            if foes >= 2 && !on_stairs && run.floor.map.is_seen(run.floor.stairs_down) && descend_step(run, cx) {
                return true;
            }
            v.adj >= 1 && verb_attack(run, cx, "nearest", v, false)
        }
        _ => false,
    }
}

// ---------------------------------------------------------------- monsters

pub fn monster_turn(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32]) {
    if run.monsters[mi].hp <= 0 {
        return;
    }
    run.monsters[mi].energy += run.monsters[mi].speed;
    let mut acts = 0;
    while run.monsters[mi].energy >= 10 && run.over.is_none() && run.monsters[mi].hp > 0 {
        run.monsters[mi].energy -= 10;
        monster_act(run, cx, mi, hero_dist);
        acts += 1;
    }
    if acts >= 2 && run.monsters[mi].hp > 0 && run.floor.map.is_visible(run.monsters[mi].pos) && run.monsters[mi].has_tag("fast") {
        let k = run.monsters[mi].kind.clone();
        learn_tag(run, cx, &k, "fast");
    }
}

fn move_monster(run: &mut Run, cx: &mut Ctx, mi: usize, q: Pos) {
    let was_visible = run.floor.map.is_visible(run.monsters[mi].pos);
    run.monsters[mi].pos = q;
    if was_visible || run.floor.map.is_visible(q) {
        let id = run.monsters[mi].id;
        cx.events.push(Ev::Move { t: run.turn, id, x: q.x, y: q.y });
    }
}

fn can_see_hero(run: &Run, mi: usize) -> bool {
    let m = &run.monsters[mi];
    let h = &run.hero;
    if m.blind > 0 || h.untargetable() {
        return false;
    }
    let d = m.pos.cheb(h.pos);
    if h.invis_t > 0 {
        return d <= 1;
    }
    d <= VISION && run.floor.map.los(m.pos, h.pos)
}

fn approach(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32]) -> bool {
    let mp = run.monsters[mi].pos;
    let water_only = run.monsters[mi].has_tag("water");
    let map = &run.floor.map;
    let occ = |q: Pos| run.occupied(q) || (water_only && map.get(q) != Tile::Water);
    if let Some(q) = map.step_down(hero_dist, mp, &occ) {
        move_monster(run, cx, mi, q);
        return true;
    }
    false
}

fn step_away(run: &mut Run, cx: &mut Ctx, mi: usize, from: Pos, prefer_unseen: bool) -> bool {
    let mp = run.monsters[mi].pos;
    let water_only = run.monsters[mi].has_tag("water");
    let map = &run.floor.map;
    let cur = mp.cheb(from);
    let mut best: Option<(i32, Pos)> = None;
    for d in DIRS8 {
        let q = mp.add(d);
        if !map.can_step(mp, q) || run.occupied(q) || (water_only && map.get(q) != Tile::Water) {
            continue;
        }
        let dist = q.cheb(from);
        if dist < cur {
            continue;
        }
        let score = dist * 4 + if prefer_unseen && !map.is_visible(q) { 6 } else { 0 } + (dist > cur) as i32;
        if best.is_none_or(|(bs, _)| score > bs) {
            best = Some((score, q));
        }
    }
    match best {
        Some((_, q)) if q != mp => {
            move_monster(run, cx, mi, q);
            true
        }
        _ => false,
    }
}

fn wander(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let mp = run.monsters[mi].pos;
    let water_only = run.monsters[mi].has_tag("water");
    let cands: Vec<Pos> = DIRS8
        .iter()
        .map(|d| mp.add(*d))
        .filter(|q| run.floor.map.can_step(mp, *q) && !run.occupied(*q) && (!water_only || run.floor.map.get(*q) == Tile::Water))
        .collect();
    if !cands.is_empty() {
        let q = *run.rng.pick(&cands);
        move_monster(run, cx, mi, q);
    }
}

fn adjacent_ally(run: &Run, mi: usize) -> Option<usize> {
    let mp = run.monsters[mi].pos;
    run.monsters
        .iter()
        .enumerate()
        .filter(|(j, o)| *j != mi && o.hp > 0 && o.ally && o.pos.adjacent(mp))
        .min_by_key(|(_, o)| (o.hp, o.id))
        .map(|(j, _)| j)
}

/// Something to hit in melee: the hero, else an adjacent ally.
fn engaged(run: &Run, mi: usize) -> bool {
    let mp = run.monsters[mi].pos;
    (mp.adjacent(run.hero.pos) && !run.hero.untargetable()) || adjacent_ally(run, mi).is_some()
}

/// Monster attack on the hero (or, failing adjacency, an ally) with tag riders. `mult` doubles ogre hits.
fn monster_attack(run: &mut Run, cx: &mut Ctx, mi: usize, mult: i32, verb: &str) {
    let mp = run.monsters[mi].pos;
    if !(mp.adjacent(run.hero.pos) && !run.hero.untargetable()) || verb == "shoot" && !can_see_hero(run, mi) {
        if let Some(ai) = adjacent_ally(run, mi) {
            let m = &run.monsters[mi];
            let atk = (m.atk.0 * mult, m.atk.1 * mult);
            let def = run.monsters[ai].effective_def();
            let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
            let (src, dst) = (run.monsters[mi].id, run.monsters[ai].id);
            cx.events.push(Ev::Attack { t: run.turn, src, dst, dmg, hit, verb: Some(verb.into()) });
            if hit {
                damage_monster(run, cx, ai, dmg, &Src::Mon(mi));
            }
            return;
        }
        if verb != "shoot" {
            return;
        }
    }
    let m = &run.monsters[mi];
    let atk = (m.atk.0 * mult, m.atk.1 * mult);
    let def = run.hero.def();
    let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
    let id = run.monsters[mi].id;
    let kind = run.monsters[mi].kind.clone();
    cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit, verb: Some(verb.into()) });
    if !hit {
        return;
    }
    let cause = kind.clone();
    if run.monsters[mi].has_tag("thief") && dmg > 0 && run.monsters[mi].stolen.is_none() {
        let stealable: Vec<usize> = (0..run.hero.inv.len()).collect();
        if !stealable.is_empty() {
            let ii = stealable[run.rng.below(stealable.len() as u32) as usize];
            let it = run.hero.inv.remove(ii);
            let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
            run.loot -= it.value();
            run.monsters[mi].stolen = Some(it);
            run.monsters[mi].fleeing = true;
            cx.events.push(Ev::Steal { t: run.turn, id, item: label.clone() });
            run.stolen.push((run.turn, label.clone()));
            note(run, cx, format!("The monkey stole the {label}."));
            callout(run, cx, "stolen!");
            learn_tag(run, cx, &kind, "thief");
        }
    }
    if run.monsters[mi].has_tag("paralyse") && dmg > 0 && run.rng.chance(20) {
        run.hero.paralysed = 1;
        callout(run, cx, "paralysed");
        learn_tag(run, cx, &kind, "paralyse");
    }
    if run.monsters[mi].has_tag("drain") && dmg > 0 && run.hero.max_hp > 5 {
        run.hero.max_hp -= 1;
        run.hero.hp = run.hero.hp.min(run.hero.max_hp);
        callout(run, cx, "drained");
        learn_tag(run, cx, &kind, "drain");
    }
    if mult > 1 {
        learn_tag(run, cx, &kind, "heavy");
    }
    let _ = cause;
    damage_hero(run, cx, dmg, &Src::Mon(mi));
}

fn summon_near(run: &mut Run, cx: &mut Ctx, at: Pos, kind: &str, n: usize, ttl: Option<i32>) -> usize {
    let mut made = 0;
    let mut cands: Vec<Pos> = at.neighbours8().into_iter().filter(|q| run.floor.map.passable(*q) && !run.occupied(*q)).collect();
    for _ in 0..n {
        if cands.is_empty() {
            break;
        }
        let q = cands.remove(run.rng.below(cands.len() as u32) as usize);
        let id = run.new_id();
        let depth = run.depth;
        let mut m = Monster::spawn(id, kind, q, depth);
        m.awake = true;
        m.last_seen = Some(run.hero.pos);
        m.ttl = ttl;
        m.summoned = true;
        let e = crate::engine::monster_entity(&m, cx.facts);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e });
        made += 1;
    }
    made
}

fn telegraph(run: &mut Run, cx: &mut Ctx, mi: usize, what: &str, pending: Pending) {
    run.monsters[mi].telegraph = Some(what.into());
    run.monsters[mi].pending = Some(pending);
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Telegraph { t: run.turn, id, what: what.into() });
    if run.floor.map.is_visible(run.monsters[mi].pos) {
        let kind = run.monsters[mi].kind.clone();
        let title = run.monsters[mi].def().title.split_whitespace().last().unwrap_or("foe").to_string();
        callout(run, cx, &format!("{title} {what}"));
        learn_tag(run, cx, &kind, "telegraph");
    }
}

fn resolve_pending(run: &mut Run, cx: &mut Ctx, mi: usize, p: Pending, hero_dist: &[i32]) {
    let kind = run.monsters[mi].kind.clone();
    let mp = run.monsters[mi].pos;
    let visible = run.floor.map.is_visible(mp);
    let hp = run.hero.pos;
    match p {
        Pending::Shoot => {
            if can_see_hero(run, mi) && mp.cheb(hp) <= BOW_RANGE {
                monster_attack(run, cx, mi, 1, "shoot");
                if visible {
                    learn_tag(run, cx, &kind, "ranged");
                }
            }
        }
        Pending::HeavyHit => {
            if engaged(run, mi) {
                monster_attack(run, cx, mi, 2, "smash");
            } else {
                approach(run, cx, mi, hero_dist);
            }
        }
        Pending::Rally => {
            summon_near(run, cx, mi_pos(run, mi), "goblin", 2, None);
            for m in run.monsters.iter_mut() {
                if m.hostile() && m.kind.starts_with("goblin") && m.pos.cheb(mp) <= VISION {
                    m.buff_def = (2, 6);
                }
            }
            if visible {
                learn_tag(run, cx, &kind, "summoner");
                learn_tag(run, cx, &kind, "buffer");
                callout(run, cx, "rallied!");
            }
        }
        Pending::Swell => {
            place_overlay(run, cx, mp, 1, OverlayKind::Gas, 5);
            if visible {
                learn_tag(run, cx, &kind, "gas");
            }
        }
        Pending::Chant => {
            summon_near(run, cx, mi_pos(run, mi), "skeleton", 2, None);
            if visible {
                learn_tag(run, cx, &kind, "summoner");
                callout(run, cx, "skeletons!");
            }
        }
    }
    if run.monsters[mi].is_boss() && run.over.is_none() && visible {
        learn(run, cx, format!("boss:{kind}:counter"));
    }
}

fn mi_pos(run: &Run, mi: usize) -> Pos {
    run.monsters[mi].pos
}

fn packmates_ready(run: &Run, mi: usize) -> bool {
    let m = &run.monsters[mi];
    let n = run.monsters.iter().filter(|o| o.hp > 0 && o.kind == m.kind && o.hostile() && o.pos.cheb(m.pos) <= 6).count();
    n >= 2 || run.hero.hp_pct() < 50
}

fn monster_act(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32]) {
    let (stun, paralysed, neutral, ally, confused) = {
        let m = &run.monsters[mi];
        (m.stun, m.paralysed, m.neutral, m.ally, m.confused)
    };
    if stun > 0 || paralysed > 0 || neutral {
        return;
    }
    if ally {
        if run.monsters[mi].is_companion() {
            companion_act(run, cx, mi, hero_dist);
        } else {
            ally_act(run, cx, mi, hero_dist);
        }
        return;
    }
    if confused > 0 {
        wander(run, cx, mi);
        return;
    }
    let hp = run.hero.pos;
    let sees = can_see_hero(run, mi);
    if sees {
        run.monsters[mi].awake = true;
        run.monsters[mi].last_seen = Some(hp);
    }
    let m = run.monsters[mi].clone();
    let mp = m.pos;
    let adjacent = engaged(run, mi);
    if m.fear > 0 || m.fleeing {
        if m.fleeing && !sees && !run.floor.map.is_visible(mp) {
            return; // hidden with the loot
        }
        if !step_away(run, cx, mi, hp, m.has_tag("thief")) && adjacent && m.fear == 0 {
            monster_attack(run, cx, mi, 1, "attack");
        }
        return;
    }
    if !m.awake {
        if run.rng.chance(15) {
            wander(run, cx, mi);
        }
        return;
    }
    if let Some(p) = m.pending {
        run.monsters[mi].pending = None;
        run.monsters[mi].telegraph = None;
        resolve_pending(run, cx, mi, p, hero_dist);
        return;
    }
    let dist = mp.cheb(hp);
    let kind = m.kind.as_str();
    match kind {
        "goblin_archer" => {
            if sees && (2..=BOW_RANGE).contains(&dist) {
                telegraph(run, cx, mi, "draws", Pending::Shoot);
            } else if adjacent {
                if !step_away(run, cx, mi, hp, false) {
                    monster_attack(run, cx, mi, 1, "attack");
                }
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "ogre" => {
            if adjacent && m.pos.adjacent(hp) {
                telegraph(run, cx, mi, "winds up", Pending::HeavyHit);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "goblin_conjurer" => {
            if sees && m.cooldown == 0 {
                let made = summon_near(run, cx, hp, "spectral_blade", 2, Some(10));
                run.monsters[mi].cooldown = 12;
                if made > 0 && run.floor.map.is_visible(mp) {
                    callout(run, cx, "blades!");
                    learn_tag(run, cx, "goblin_conjurer", "caster");
                    learn_tag(run, cx, "goblin_conjurer", "summoner");
                }
            } else if adjacent {
                if !step_away(run, cx, mi, hp, false) {
                    monster_attack(run, cx, mi, 1, "attack");
                }
            } else if dist > 4 || !sees {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "jackal" | "ghoul" => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else if packmates_ready(run, mi) {
                chase(run, cx, mi, hero_dist, sees);
            } else if sees && dist < 3 {
                step_away(run, cx, mi, hp, false);
            } else if !sees {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "bloat" => {
            if !adjacent {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "eel" => {
            if run.floor.map.get(mp) != Tile::Water {
                return;
            }
            if adjacent {
                monster_attack(run, cx, mi, 1, "bite");
                learn_tag(run, cx, "eel", "water");
            } else if sees && dist <= 5 {
                approach(run, cx, mi, hero_dist);
            }
        }
        "goblin_warlord" => {
            if sees && m.cooldown == 0 {
                telegraph(run, cx, mi, "rallies", Pending::Rally);
                run.monsters[mi].cooldown = 10;
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "bloat_mother" => {
            if sees && m.cooldown == 0 && dist <= 3 && m.hp * 2 <= m.max_hp {
                telegraph(run, cx, mi, "swells", Pending::Swell);
                run.monsters[mi].cooldown = 8;
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        "lich" => {
            if sees && m.cooldown == 0 {
                telegraph(run, cx, mi, "chants", Pending::Chant);
                run.monsters[mi].cooldown = 9;
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
                if run.over.is_none() && run.hero.max_hp > 5 {
                    run.hero.max_hp -= 1;
                    run.hero.hp = run.hero.hp.min(run.hero.max_hp);
                }
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
        _ => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, hero_dist, sees);
            }
        }
    }
}

/// Approach the hero if seen, else walk to the last known position, else forget.
fn chase(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32], sees: bool) {
    if sees {
        approach(run, cx, mi, hero_dist);
        return;
    }
    let Some(target) = run.monsters[mi].last_seen else {
        wander(run, cx, mi);
        return;
    };
    let mp = run.monsters[mi].pos;
    if mp.cheb(target) <= 1 {
        run.monsters[mi].last_seen = None;
        wander(run, cx, mi);
        return;
    }
    // Greedy step toward the last seen position.
    let map = &run.floor.map;
    let water_only = run.monsters[mi].has_tag("water");
    let q = DIRS8
        .iter()
        .map(|d| mp.add(*d))
        .filter(|q| map.can_step(mp, *q) && !run.occupied(*q) && (!water_only || map.get(*q) == Tile::Water))
        .min_by_key(|q| (q.cheb(target), q.x, q.y));
    match q {
        Some(q) if q.cheb(target) < mp.cheb(target) => move_monster(run, cx, mi, q),
        _ => {
            // Blocked: fall back to the hero's field (it knows the map).
            if !approach(run, cx, mi, hero_dist) {
                run.monsters[mi].last_seen = None;
            }
        }
    }
}

fn ally_act(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32]) {
    let mp = run.monsters[mi].pos;
    let target = run
        .monsters
        .iter()
        .enumerate()
        .filter(|(j, o)| *j != mi && o.hp > 0 && o.hostile() && o.pos.adjacent(mp))
        .min_by_key(|(_, o)| (o.hp, o.id))
        .map(|(j, _)| j);
    if let Some(ti) = target {
        let atk = run.monsters[mi].atk;
        let def = run.monsters[ti].effective_def();
        let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
        let (src, dst) = (run.monsters[mi].id, run.monsters[ti].id);
        cx.events.push(Ev::Attack { t: run.turn, src, dst, dmg, hit, verb: Some("attack".into()) });
        if hit {
            damage_monster(run, cx, ti, dmg, &Src::Mon(mi));
        }
        return;
    }
    let hp = run.hero.pos;
    if mp.cheb(hp) > 2 {
        approach(run, cx, mi, hero_dist);
        return;
    }
    // Close in on a visible hostile nearby.
    let map = &run.floor.map;
    let foe = run
        .monsters
        .iter()
        .enumerate()
        .filter(|(j, o)| *j != mi && o.hp > 0 && o.hostile() && map.is_visible(o.pos) && o.pos.cheb(mp) <= 4)
        .min_by_key(|(_, o)| (o.pos.cheb(mp), o.id))
        .map(|(_, o)| o.pos);
    if let Some(fp) = foe {
        let q = DIRS8.iter().map(|d| mp.add(*d)).filter(|q| map.can_step(mp, *q) && !run.occupied(*q)).min_by_key(|q| (q.cheb(fp), q.x, q.y));
        if let Some(q) = q {
            if q.cheb(fp) < mp.cheb(fp) {
                move_monster(run, cx, mi, q);
            }
        }
    }
}

// ---------------------------------------------------------------- companions (Addendum A)

fn verb_tame(run: &mut Run, cx: &mut Ctx, a: &str, v: &View) -> bool {
    let Some(li) = run.hero.inv.iter().position(|i| i.kind == "leash" && i.amount > 0) else { return false };
    let sel = if a.is_empty() { "nearest" } else { a };
    let cand = v.foes.iter().copied().find(|&i| {
        let m = &run.monsters[i];
        let weak = m.hp * 100 / m.max_hp.max(1) < 25;
        let tag_ok = match sel.strip_prefix("tag:") {
            Some(t) => m.has_tag(t),
            None => true,
        };
        weak && tag_ok && !m.is_boss() && !m.summoned && !m.neutral
    });
    let Some(mi) = cand else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if !mp.adjacent(hp) {
        let (dist, parent) = hero_bfs(run);
        let map = &run.floor.map;
        let goal = mp.neighbours8().into_iter().filter(|q| map.in_bounds(*q) && dist[map.idx(*q)] >= 0).min_by_key(|q| (dist[map.idx(*q)], q.x, q.y));
        return match goal {
            Some(g) if g != hp => step_towards(run, cx, g, &parent),
            _ => false,
        };
    }
    // Spend the leash and the turn.
    run.hero.inv[li].amount -= 1;
    if run.hero.inv[li].amount <= 0 {
        run.hero.inv.remove(li);
    }
    let kind = run.monsters[mi].kind.clone();
    let chance = crate::engine::tame_chance(cx.facts, &kind);
    let ok = run.rng.chance(chance);
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Tame { t: run.turn, id, kind: kind.clone(), ok });
    if ok {
        let n = run.tamed.len() as u32;
        let cid = 1_000_000 + run.id * 100 + n;
        let name = crate::descent::grudge_name(&mut run.rng);
        {
            let m = &mut run.monsters[mi];
            m.ally = true;
            m.awake = true;
            m.fleeing = false;
            m.fear = 0;
            m.cid = Some(cid);
            m.name = Some(name.clone());
            m.level = 1;
            m.stolen = None;
            m.last_seen = None;
        }
        let rec = crate::engine::new_companion(cid, &run.monsters[mi], name.clone());
        run.companions.push(rec);
        run.tamed.push((run.turn, kind.clone()));
        learn(run, cx, format!("tamed:{kind}"));
        note(run, cx, format!("Tamed a {}: {}.", crate::engine::kind_title(&kind), name));
        callout(run, cx, "tamed!");
    } else {
        callout(run, cx, "slipped");
        monster_attack(run, cx, mi, 1, "attack");
    }
    true
}

fn recall_companion(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let Some(cid) = run.monsters[mi].cid else { return };
    if !run.recalled.contains(&cid) {
        run.recalled.push(cid);
    }
    let id = run.monsters[mi].id;
    run.monsters[mi].hp = 0;
    run.monsters[mi].cid = None;
    cx.events.push(Ev::Move { t: run.turn, id, x: -1, y: -1 });
    callout(run, cx, "recalled");
}

/// Foes as seen from a companion: shared party vision plus its own adjacency.
fn companion_view(run: &Run, mi: usize) -> View {
    let mp = run.monsters[mi].pos;
    let map = &run.floor.map;
    let mut foes: Vec<usize> = (0..run.monsters.len())
        .filter(|&i| {
            let m = &run.monsters[i];
            i != mi && m.hp > 0 && m.hostile() && (map.is_visible(m.pos) || m.pos.adjacent(mp))
        })
        .collect();
    foes.sort_by_key(|&i| (run.monsters[i].pos.cheb(mp), run.monsters[i].id));
    let adj = foes.iter().filter(|&&i| run.monsters[i].pos.adjacent(mp)).count() as i32;
    let nearest = foes.first().copied();
    let lowest = foes.iter().copied().min_by_key(|&i| (run.monsters[i].hp, run.monsters[i].id));
    View { foes, adj, nearest, lowest }
}

fn companion_cond(run: &Run, cx: &Ctx, mi: usize, v: &View, c: &crate::rules::Cond) -> bool {
    let m = &run.monsters[mi];
    let n = c.n.unwrap_or(0);
    let pct = m.hp * 100 / m.max_hp.max(1);
    match c.k.as_str() {
        "self_hp<" => pct < n,
        "self_hp>" => pct > n,
        "in_corridor" => run.floor.map.is_corridor(m.pos),
        "on_hurt" => m.hurt_since_action,
        _ => cond_holds(run, cx, v, c),
    }
}

fn companion_melee(run: &mut Run, cx: &mut Ctx, mi: usize, ti: usize, verb: &str, mult_num: i32) -> i32 {
    let atk = run.monsters[mi].atk;
    let atk = (atk.0 * mult_num / 2, atk.1 * mult_num / 2);
    let def = run.monsters[ti].effective_def();
    let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
    let (src, dst) = (run.monsters[mi].id, run.monsters[ti].id);
    cx.events.push(Ev::Attack { t: run.turn, src, dst, dmg, hit, verb: Some(verb.into()) });
    if hit {
        damage_monster(run, cx, ti, dmg, &Src::Mon(mi));
        if run.monsters[ti].hp > 0 && !run.monsters[ti].awake {
            run.monsters[ti].awake = true;
            run.monsters[ti].last_seen = Some(run.hero.pos);
        }
        dmg
    } else {
        0
    }
}

/// Step toward a target monster (BFS over the whole map, occupied tiles blocked).
fn companion_approach(run: &mut Run, cx: &mut Ctx, mi: usize, target: Pos) -> bool {
    let mp = run.monsters[mi].pos;
    let map = &run.floor.map;
    let dist = map.bfs(target, false, &|p| p != mp && run.occupied(p));
    let occ = |q: Pos| run.occupied(q);
    if let Some(q) = map.step_down(&dist, mp, &occ) {
        move_monster(run, cx, mi, q);
        return true;
    }
    false
}

fn try_companion_verb(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &Verb, v: &View, hero_dist: &[i32]) -> bool {
    let mp = run.monsters[mi].pos;
    let adj_target = v.foes.iter().copied().find(|&i| run.monsters[i].pos.adjacent(mp));
    match verb.v.as_str() {
        "attack" => {
            if let Some(ti) = adj_target {
                companion_melee(run, cx, mi, ti, "attack", 2);
                return true;
            }
            match v.nearest {
                Some(ti) if run.monsters[ti].pos.cheb(mp) <= 6 => {
                    let tp = run.monsters[ti].pos;
                    companion_approach(run, cx, mi, tp)
                }
                _ => false,
            }
        }
        "shoot" => {
            if !run.monsters[mi].has_tag("ranged") {
                return false;
            }
            let target = v.foes.iter().copied().find(|&i| {
                let tp = run.monsters[i].pos;
                (1..=BOW_RANGE).contains(&tp.cheb(mp)) && run.floor.map.los(mp, tp)
            });
            match target {
                Some(ti) => {
                    companion_melee(run, cx, mi, ti, "shoot", 2);
                    true
                }
                None => false,
            }
        }
        "burst" => {
            if !run.monsters[mi].has_tag("gas") || v.foes.is_empty() {
                return false;
            }
            let hp = run.monsters[mi].hp;
            callout(run, cx, "burst!");
            damage_monster(run, cx, mi, hp, &Src::Burst);
            true
        }
        "steal" => {
            if !run.monsters[mi].has_tag("thief") {
                return false;
            }
            let Some(ti) = adj_target.filter(|&t| !run.monsters[mi].stole_from.contains(&run.monsters[t].id)) else { return false };
            let dmg = companion_melee(run, cx, mi, ti, "steal", 2);
            if dmg > 0 && run.monsters.get(ti).is_some() {
                let tid = run.monsters[ti].id;
                let gold = 3 * run.depth as i32;
                run.loot += gold;
                run.monsters[mi].stole_from.push(tid);
                let id = run.monsters[mi].id;
                cx.events.push(Ev::Steal { t: run.turn, id, item: format!("gold ({gold})") });
            }
            true
        }
        "split" => {
            let m = &run.monsters[mi];
            if !m.has_tag("splitter") || m.hp * 2 <= m.max_hp || v.foes.is_empty() {
                return false;
            }
            let half = m.hp / 2;
            let Some(q) = mp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q)) else { return false };
            run.monsters[mi].hp -= half;
            let id = run.new_id();
            let kind = run.monsters[mi].kind.clone();
            let depth = run.depth;
            let mut child = Monster::spawn(id, &kind, q, depth);
            child.ally = true;
            child.awake = true;
            child.hp = half;
            child.max_hp = run.monsters[mi].max_hp;
            child.extra_tags = run.monsters[mi].extra_tags.clone();
            let e = crate::engine::monster_entity(&child, cx.facts);
            run.monsters.push(child);
            cx.events.push(Ev::Spawn { t: run.turn, e });
            callout(run, cx, "splits!");
            true
        }
        "flank" => {
            if !run.monsters[mi].has_tag("pack") {
                return false;
            }
            let Some(ti) = v.nearest else { return false };
            let tp = run.monsters[ti].pos;
            if tp.adjacent(mp) {
                let flanked = tp.adjacent(run.hero.pos);
                companion_melee(run, cx, mi, ti, "flank", if flanked { 3 } else { 2 });
                return true;
            }
            let hp = run.hero.pos;
            let map = &run.floor.map;
            let goal = tp
                .neighbours8()
                .into_iter()
                .filter(|q| map.passable(*q) && !run.occupied(*q))
                .max_by_key(|q| (q.cheb(hp), -q.x, -q.y));
            match goal {
                Some(g) => companion_approach(run, cx, mi, g),
                None => false,
            }
        }
        "drain" => {
            if !run.monsters[mi].has_tag("undead") {
                return false;
            }
            let Some(ti) = adj_target else { return false };
            let dmg = companion_melee(run, cx, mi, ti, "drain", 2);
            if dmg > 0 {
                let m = &mut run.monsters[mi];
                m.hp = (m.hp + dmg).min(m.max_hp);
            }
            true
        }
        "follow" => {
            if mp.cheb(run.hero.pos) > 2 {
                approach(run, cx, mi, hero_dist);
            }
            true
        }
        "recall" => {
            recall_companion(run, cx, mi);
            true
        }
        _ => false,
    }
}

/// A companion acts on its own rows; fallback: fight adjacent, stay within 2 of the hero.
fn companion_act(run: &mut Run, cx: &mut Ctx, mi: usize, hero_dist: &[i32]) {
    let Some(cid) = run.monsters[mi].cid else { return };
    let (rows, max_rows) = match run.companion(cid) {
        Some(c) => (c.rules.rows.clone(), c.max_rows),
        None => (Vec::new(), 2),
    };
    let v = companion_view(run, mi);
    if run.monsters[mi].sent {
        if v.foes.is_empty() {
            run.monsters[mi].sent = false;
        } else if try_companion_verb(run, cx, mi, &Verb::new("attack"), &v, hero_dist) {
            run.monsters[mi].hurt_since_action = false;
            return;
        }
    }
    for row in rows.iter().take(max_rows) {
        if !row.conds.iter().all(|c| companion_cond(run, cx, mi, &v, c)) {
            continue;
        }
        if try_companion_verb(run, cx, mi, &row.verb, &v, hero_dist) {
            run.monsters[mi].hurt_since_action = false;
            return;
        }
    }
    run.monsters[mi].hurt_since_action = false;
    let mp = run.monsters[mi].pos;
    if let Some(ti) = v.foes.iter().copied().find(|&i| run.monsters[i].pos.adjacent(mp)) {
        companion_melee(run, cx, mi, ti, "attack", 2);
        return;
    }
    if mp.cheb(run.hero.pos) > 2 {
        approach(run, cx, mi, hero_dist);
    }
}
