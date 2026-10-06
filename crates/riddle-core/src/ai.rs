//! Hero verbs (with real pathing) and tag-faithful monster AI.
use crate::chronicle::{callout, note};
use crate::defs::Cat;
use crate::engine::{Ctx, ExitTier, Run, HERO_ID};
use crate::facts::{learn, learn_tag};
use crate::geom::{Pos, DIRS8};
use crate::item::{ident_fact, is_identified, Hint, Item};
use crate::monster::{Monster, Pending};
use crate::rules::Verb;
use crate::hero::{class_has_verb, Class};
use crate::tiles::{OverlayKind, Tile, VISION};
use crate::turn::{cond_holds, damage_hero, damage_monster, descend, end_run, noise, pickup_here, place_overlay, view, Src, View};
use crate::wire::Ev;

pub const THROW_RANGE: i32 = 6;
pub const BOW_RANGE: i32 = 6;
/// Cut 2 §4: the ranger keeps this range with `kite`; the caster's bolt.
pub const KITE_RANGE: i32 = 3;
/// Cut 2 §1: the chores descend once this much of the floor is seen.
pub const CHORE_DESCEND_SEEN: i32 = 60;
pub const BOLT_ATK: (i32, i32) = (2, 5);
/// Cut 20 §1: ticks a thief runs once the run's one theft is spent (it has nothing to take).
pub const THIEF_FLEE: i32 = 600;

/// 80% to hit; damage = roll(atk) − def, min 0.
pub fn roll_hit(rng: &mut crate::rng::Rng, atk: (i32, i32), def: i32) -> (bool, i32) {
    roll_hit_pct(rng, atk, def, 80)
}

/// `roll_hit` at `pct` to hit (Cut 25 §1: the forged weapon's steps are aim, `Hero::hit_pct`).
pub fn roll_hit_pct(rng: &mut crate::rng::Rng, atk: (i32, i32), def: i32, pct: u32) -> (bool, i32) {
    let hit = rng.chance(pct);
    let roll = rng.range(atk.0, atk.1);
    (hit, if hit { (roll - def).max(0) } else { 0 })
}

// ---------------------------------------------------------------- hero movement

pub fn move_hero(run: &mut Run, cx: &mut Ctx, q: Pos) {
    if q!=run.hero.pos {crate::firearm::cancel_aim(run,cx);}
    run.hero.pos = q;
    cx.events.push(Ev::Move { t: run.turn, id: HERO_ID, x: q.x, y: q.y });
    pickup_here(run, cx);
}

pub fn random_step(run: &mut Run, cx: &mut Ctx) {
    let hp = run.hero.pos;
    let cands: Vec<Pos> = DIRS8.iter().map(|d| hp.step(*d)).filter(|q| run.floor.map.can_step(hp, *q) && !run.occupied(*q)).collect();
    if cands.is_empty() {
        return;
    }
    let q = *run.rng.pick(&cands);
    move_hero(run, cx, q);
}

/// The hero's flood: over seen tiles from the hero, `hero_avoids`' obstacles.
/// Chores and approaches path through monsters (a hostile on the first step simply stops the
/// step; the rules decide what to do about it). Paths never depend on what is in view, so
/// explore and descend cannot disagree about which corridor is open.
///
/// The nearest of `goals` by (distance, x, y) on a flood from the hero over seen tiles with
/// `blocked` (the hero's flood's shape), and the flood's parents: it stops at the first distance holding
/// a goal — every nearer tile, that distance's goals and the paths to them are the full flood's —
/// where the whole floor was flooded to choose among a target's eight neighbours. `None`: none
/// reachable (the flood ran out).
fn nearest_goal(run: &Run, blocked: &impl Fn(Pos) -> bool, goals: &[Pos]) -> (Option<Pos>, Vec<i32>) {
    let map = &run.floor.map;
    let at: Vec<usize> = goals.iter().filter(|q| map.in_bounds(**q)).map(|q| map.idx(*q)).collect();
    let mut found = None;
    let (_, parent) = map.bfs_parent_layers(run.hero.pos, true, blocked, |_, layer| {
        found = layer.iter().filter(|i| at.contains(i)).map(|&i| map.pos(i)).min_by_key(|q| (q.x, q.y));
        found.is_some()
    });
    (found, parent)
}

/// The hero's flood's obstacles. Cut 7: lingering gas is terrain too when another way exists.
fn hero_avoids(run: &Run, avoid: bool) -> impl Fn(Pos) -> bool + '_ {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    move |p| avoid && p != hp && (map.get(p) == Tile::Water || run.mirrors.iter().any(|m| m.cheb(p) <= 1) || run.in_den_zone(p) || run.sleepers.contains(&p) || run.overlays.iter().any(|o| o.x == p.x && o.y == p.y))
}

/// The hero's flood's parents toward `goal` (the flood stops there — `Map::bfs_parent_to`).
fn hero_path_to(run: &Run, goal: Pos) -> Vec<i32> {
    run.floor.map.bfs_parent_to(run.hero.pos, true, &hero_avoids(run, true), goal)
}

/// Cut 19 §2: the way home (`bank`, `return`) steps round the awake foes on it where the
/// floor allows (a foe in the only corridor still stops him — the row then fails and the next
/// one acts).
fn hero_path_home(run: &Run, goal: Pos) -> Vec<i32> {
    let avoid = hero_avoids(run, true);
    let foes: Vec<Pos> = run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && !m.dormant).map(|m| m.pos).collect();
    run.floor.map.bfs_parent_to(run.hero.pos, true, &|q| avoid(q) || foes.contains(&q), goal)
}

/// A path step toward `goal`, avoiding water when possible.
fn path_step(run: &Run, goal: Pos) -> Option<Pos> {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    if let Some(q) = map.first_step(&map.bfs_parent_to(hp, true, &hero_avoids(run, true), goal), hp, goal) {
        return Some(q);
    }
    map.first_step(&map.bfs_parent_to(hp, true, &hero_avoids(run, false), goal), hp, goal)
}

fn step_towards(run: &mut Run, cx: &mut Ctx, goal: Pos, parent: &[i32]) -> bool {
    let hp = run.hero.pos;
    let step = run.floor.map.first_step(parent, hp, goal).or_else(|| path_step(run, goal));
    if let Some(q) = step {
        if let Some(mi) = run.monster_at(q) {
            // Cut 7 §3: a captive chained across the stairs cannot be swapped past.
            if !run.monsters[mi].hostile() && run.monsters[mi].situation.as_deref() != Some("captive") {
                // Swap places with the ally (or a plain captive) in the way.
                run.monsters[mi].pos = hp;
                let id = run.monsters[mi].id;
                cx.events.push(Ev::Move { t: run.turn, id, x: hp.x, y: hp.y });
                move_hero(run, cx, q);
                return true;
            }
            // Cut 7 §3: a sleeping thief with no way round it is shoved awake (it bolts).
            if run.monsters[mi].dormant && run.monsters[mi].situation.is_some() {
                hero_attack(run, cx, mi, "attack", false);
                return true;
            }
            return false;
        }
        move_hero(run, cx, q);
        return true;
    }
    false
}

/// Nearest reachable tile satisfying `pred` (by BFS distance, then tile index), with the parent
/// map (final along the path to it — `Map::bfs_nearest`).
fn nearest_tile(run: &Run, pred: &dyn Fn(Pos) -> bool) -> Option<(Pos, Vec<i32>)> {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    for avoid in [true, false] {
        let found = map.bfs_nearest(hp, true, &hero_avoids(run, avoid), pred);
        if found.is_some() {
            return found;
        }
    }
    None
}

fn is_frontier(run: &Run, p: Pos) -> bool {
    let map = &run.floor.map;
    if !map.is_seen(p) || !map.passable(p) {
        return false;
    }
    DIRS8.iter().any(|d| {
        let q = p.step(*d);
        map.in_bounds(q) && !map.is_seen(q)
    })
}

/// The nearest frontier and the first step toward it (diagnostics).
pub fn frontier_target(run: &Run) -> Option<(Pos, Option<Pos>)> {
    nearest_tile(run, &|p| is_frontier(run, p)).map(|(goal, parent)| (goal, run.floor.map.first_step(&parent, run.hero.pos, goal)))
}

/// Explore toward the nearest frontier. Returns false when the floor is fully explored.
pub fn explore_step(run: &mut Run, cx: &mut Ctx) -> bool {
    if let Some((goal, parent)) = nearest_tile(run, &|p| is_frontier(run, p)) {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

/// `chore`: the chores leave a sleeping den's gold alone (a `pick_up` row does not).
fn nearest_item_step(run: &mut Run, cx: &mut Ctx, chore: bool) -> bool {
    // (one look: the pack read once for every item — `turn::PackRead`)
    let pack = crate::turn::PackRead::default();
    let cands: Vec<Pos> = run
        .items
        .iter()
        // (every test is pure: the cheap ones first)
        .filter(|fi| run.floor.map.is_seen(fi.pos) && !(chore && run.skip_items.contains(&fi.item.id)) && !(chore && run.in_den_zone(fi.pos)) && crate::turn::would_take_in(run, cx, &fi.item, &pack))
        .map(|fi| fi.pos)
        .collect();
    if cands.is_empty() {
        return false;
    }
    if let Some((goal, parent)) = nearest_tile(run, &|p| cands.contains(&p)) {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

/// Cut 5 §4: a step toward the nearest den gold in view (greed's temptation).
pub fn den_gold_step(run: &mut Run, cx: &mut Ctx) -> bool {
    let map = &run.floor.map;
    let cands: Vec<Pos> = run.items.iter().filter(|fi| fi.item.kind == "gold" && map.is_visible(fi.pos)).map(|fi| fi.pos).collect();
    if let Some((goal, parent)) = nearest_tile(run, &|p| cands.contains(&p)) {
        return step_towards(run, cx, goal, &parent);
    }
    false
}

fn in_hazard(run: &Run, p: Pos) -> bool {
    run.overlays.iter().any(|o| o.x == p.x && o.y == p.y)
}

/// Path to the stairs or the nearest frontier ignoring foes (used once the hero has idled);
/// swaps past friends, stops before hostiles.
fn push_through(run: &mut Run, cx: &mut Ctx) -> Option<Verb> {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    let (dist, parent) = map.bfs_parent(hp, true, &|_| false);
    let stairs = run.floor.stairs_down;
    let mut goal: Option<(i32, Pos, &str)> = None;
    if map.is_seen(stairs) && dist[map.idx(stairs)] > 0 && (!stairs_sealed(run) || hp.cheb(stairs) > 2) {
        goal = Some((dist[map.idx(stairs)], stairs, "descend"));
    }
    for (i, d) in dist.iter().enumerate() {
        if *d <= 0 {
            continue;
        }
        let p = map.pos(i);
        if is_frontier(run, p) && goal.is_none_or(|(gd, _, _)| *d < gd) {
            goal = Some((*d, p, "explore"));
        }
    }
    let (_, goal, verb) = goal?;
    let q = map.first_step(&parent, hp, goal)?;
    if run.monster_at(q).is_some_and(|mi| run.monsters[mi].hostile()) {
        // A hostile in the way is the rules' business, not the chores'.
        return None;
    }
    if step_towards(run, cx, goal, &parent) {
        return Some(Verb::new(verb));
    }
    None
}

/// Step out of gas or fire if standing in it and a clear tile is adjacent.
pub fn escape_hazard(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let hp = run.hero.pos;
    if !in_hazard(run, hp) {
        return false;
    }
    let q = DIRS8
        .iter()
        .map(|d| hp.step(*d))
        .filter(|q| run.floor.map.can_step(hp, *q) && !run.occupied(*q) && !in_hazard(run, *q))
        .min_by_key(|q| (min_foe_dist(run, v, *q) < 2, q.x, q.y));
    match q {
        Some(q) => {
            move_hero(run, cx, q);
            true
        }
        None => {
            // Cut 7: deep in a cloud (every neighbour gassed) — walk toward the nearest clean
            // tile rather than stand and breathe it.
            let (dist, parent) = run.floor.map.bfs_parent(hp, false, &|p| run.occupied(p));
            let map = &run.floor.map;
            let goal = (0..map.tiles.len())
                .filter(|&i| dist[i] > 0 && !in_hazard(run, map.pos(i)))
                .min_by_key(|&i| (dist[i], i))
                .map(|i| map.pos(i));
            match goal.and_then(|g| run.floor.map.first_step(&parent, hp, g)) {
                Some(q) => {
                    move_hero(run, cx, q);
                    true
                }
                None => false,
            }
        }
    }
}

/// Cut 3 chore: an identified enchant scroll or strength potion has no downside — the hero
/// uses it when nothing is in view rather than carry it (a full pack of them blocked every
/// throwable in the Foundry).
///
/// Cut 27 §5 (AS: `R1 fired 0 of 8 runs: foe: boss → drink strength` beside a trace's `← drunk
/// strength at 20/42 hp D7` — the chore drank it before any boss came): a kind a row of the set
/// uses by name is the row's to use, never the chore's.
fn use_boosts(run: &mut Run, cx: &mut Ctx) -> Option<Verb> {
    let row_uses = |cx: &Ctx, v: &str, k: &str| cx.rules.active(cx.max_rows()).any(|(_, r)| r.verb.v == v && r.verb.a.as_deref() == Some(k));
    let enchant_known = is_identified(cx.facts, cx.flavours, "enchant") && !row_uses(cx, "read", "enchant");
    let strength_known = is_identified(cx.facts, cx.flavours, "strength") && !row_uses(cx, "drink", "strength");
    if enchant_known && run.hero.weapon.is_some() && run.hero.inv.iter().any(|i| i.kind == "enchant") {
        let v = view(run);
        if verb_read(run, cx, "enchant", &v) {
            return Some(Verb::arg("read", "enchant"));
        }
    }
    if strength_known && run.hero.inv.iter().any(|i| i.kind == "strength") && verb_drink(run, cx, "strength") {
        return Some(Verb::arg("drink", "strength"));
    }
    None
}

/// Engine chore: out of hazards → items → explore → descend. Returns the verb performed.
pub fn chore(run: &mut Run, cx: &mut Ctx, v: &View) -> Verb {
    if escape_hazard(run, cx, v) {
        return Verb::new("explore");
    }
    // Cut 4: with an awake hostile at the hero's elbow and no row that acts, the chores say
    // so (`cornered`, not `wait`): stepping clear or swinging back would be a policy the
    // player never wrote (PASSIVE must lose every seed).
    let biting = v.foes.iter().any(|&i| run.monsters[i].awake && run.monsters[i].pos.adjacent(run.hero.pos));
    if v.foes.is_empty() {
        if let Some(verb) = use_boosts(run, cx) {
            return verb;
        }
    }
    // Cut 3: a chalked floor (`chalk:<depth>`) is walked straight to its stairs (whatever lies
    // on the way is picked up in passing).
    // QA on 1a2a4a9: walking home (`Run.homeward`), the walk is the chore — to the up-stairs
    // and out at the committing row's tier — and the chores never take him down a floor.
    let home = run.homeward.is_some();
    if home {
        let verb = Verb::new(if run.homeward_bank { "bank" } else { "return" });
        let s = run.floor.stairs_up;
        if run.hero.pos == s {
            if run.exit_row.is_none() {
                run.exit_row = run.homeward;
            }
            end_run(run, cx, if run.homeward_bank { ExitTier::Bank } else { ExitTier::Return });
            return verb;
        }
        let parent = hero_path_home(run, s);
        if step_towards(run, cx, s, &parent) {
            return verb;
        }
    }
    if v.adj == 0 && !home && cx.facts.contains(&format!("chalk:{}", run.depth)) && !stairs_sealed(run) && chalk_step(run, cx) {
        return Verb::new("descend");
    }
    if v.adj == 0 && !run.items_ignored() && nearest_item_step(run, cx, true) {
        return Verb::new("pick_up");
    }
    // Cut 5 §4: a vault seen on this floor is worth the walk — for a hero above half health
    // (a worn one crossing a floor for loot met every wanderer on the way; those deaths were dice).
    if v.adj == 0 && run.hero.hp_pct() >= 50 && vault_step(run, cx) {
        return Verb::new("explore");
    }
    // Cut 2 §3 `bone_sense`: the hero paths to the bones on this floor, seen or not.
    if v.adj == 0 && cx.unlocks.contains("bone_sense") && bones_step(run, cx) {
        return Verb::new("pick_up");
    }
    // The chore threshold for `descend` is 60% seen (Cut 2 §1): once that much of the floor
    // is known and the way down is too, the chores go down rather than sweep the rest. A
    // heir with no rows at all has no orders and only wanders (the chores never carry it).
    // (unsealed stairs neither stood on nor seen: `descend_step` declines without a side effect, so
    // the floor's seen share — a pass over every tile — is not read)
    let s = run.floor.stairs_down;
    if !home && !cx.rules.rows.is_empty() && (run.hero.pos == s || run.floor.map.is_seen(s)) && run.floor.map.seen_pct() >= CHORE_DESCEND_SEEN && !stairs_sealed(run) && descend_step(run, cx) {
        return Verb::new("descend");
    }
    if explore_step(run, cx) {
        return Verb::new("explore");
    }
    if !home && descend_step(run, cx) {
        return Verb::new("descend");
    }
    if !run.items_ignored() && nearest_item_step(run, cx, true) {
        return Verb::new("pick_up");
    }
    // Everything is walled off by foes for a while: path through them and bump whoever
    // stands in the way (a deadlock breaker, not a fighting style).
    if run.idle_actions >= 6 {
        if let Some(verb) = push_through(run, cx) {
            return verb;
        }
    }
    // Nothing to do: after 20 idle actions, shuffle so a blocked hero does not idle forever.
    run.idle_actions += 1;
    if run.idle_actions >= 20 {
        run.idle_actions = 0;
        random_step(run, cx);
        return Verb::new("shuffle");
    }
    if biting {
        return Verb::new("cornered");
    }
    Verb::new("wait")
}

/// Cut 4: the hunt. The foe the last foe-targeting row acted on has stepped out of view: walk
/// toward the tile it was last seen on. Returns its kind when a step was taken; `None` (and
/// the hunt dropped) when it is dead, in view again, forgotten, given up on, or the tile is
/// reached or unreachable.
pub fn hunt_step(run: &mut Run, cx: &mut Ctx) -> Option<String> {
    let (id, _) = run.hunt?;
    let Some(mi) = run.monsters.iter().position(|m| m.id == id && m.hp > 0 && m.hostile()) else {
        run.hunt = None;
        return None;
    };
    if run.floor.map.is_visible(run.monsters[mi].pos) || run.is_ignored(id) {
        run.hunt = None;
        return None;
    }
    let Some(&(goal, at)) = run.known_foes.get(&id) else {
        run.hunt = None;
        return None;
    };
    let hp = run.hero.pos;
    if run.actions.saturating_sub(at) > crate::engine::REMEMBER_ACTIONS || goal.cheb(hp) <= 1 {
        run.hunt = None;
        return None;
    }
    let parent = hero_path_to(run, goal);
    if step_towards(run, cx, goal, &parent) {
        Some(run.monsters[mi].kind.clone())
    } else {
        run.hunt = None;
        None
    }
}

/// A class verb still cooling down.
fn class_cooldown(run: &Run, verb: &Verb) -> bool {
    let h = &run.hero;
    match verb.v.as_str() {
        "riposte" | "hex" => h.special_cd > 0,
        "shield_bash" => h.bash_cd > 0,
        "cleave" => h.cleave_cd > 0,
        "double_shot" => h.double_cd > 0,
        _ => false,
    }
}

/// Cut 4: why a row whose conditions held could not act (≤ 2 words).
pub fn block_reason(run: &Run, cx: &Ctx, verb: &Verb, v: &View) -> &'static str {
    let holds = |kind: &str| run.hero.inv.iter().any(|i| i.kind == kind);
    // Cut 6 §2/§3: an item of the kind in the pack that the hero cannot name (a found
    // potion, flavour unidentified) reads `unknown item`, not `no item`.
    let held_known = |kind: &str| run.hero.inv.iter().any(|i| i.kind == kind && i.is_known(cx.facts, cx.flavours));
    match verb.v.as_str() {
        "riposte" | "hex" => {
            let style=if verb.v=="riposte" {crate::specialization::Style::Sentinel}else{crate::specialization::Style::Hexbinder};
            if !crate::specialization::has(&run.hero,style) {"class locked"}
            else if run.hero.special_cd>0 {"cooldown"}
            else if style==crate::specialization::Style::Sentinel {"no enemy"}
            else if let Some(mi)=pick_target_from(run,verb.a.as_deref().unwrap_or("nearest"),&v.foes) {
                let m=&run.monsters[mi];
                if m.hex_t>0 {"already hexed"}else if m.pos.cheb(run.hero.pos)>BOW_RANGE {"too far"}
                else if !run.floor.map.los(run.hero.pos,m.pos) {"no sight"}else {"no use"}
            }else {"no enemy"}
        }

        "retreat" | "back_corridor" | "blink" | "shadowstep" | "vanish" | "smoke" => "no path",
        "drink" | "read" => {
            let a = verb.a.as_deref().unwrap_or("");
            let cat = if verb.v == "drink" { Cat::Potion } else { Cat::Scroll };
            // `drink unknown` with no unknown potion in the pack is `no unknown` (QA on 952e306:
            // `R4 no use` beside `5 unknown unused` — the unknowns were scrolls; Cut 18 §4,
            // rater Z: `drink ✗ no item` beside four known heals read as a contradiction — the
            // callout drops the argument, so the reason names what is missing); `no use` is
            // the verb's own refusal of something held (a known heal at full HP).
            let holds_unknown = run.hero.inv.iter().any(|i| i.cat() == cat && !i.is_known(cx.facts, cx.flavours));
            if a.is_empty() || a == "unknown" {
                if holds_unknown { "no use" } else { "no unknown" }
            } else if held_known(a) {
                "no use"
            } else if holds(a) {
                "unknown item"
            } else {
                "no item"
            }
        }
        "throw" => {
            let k = verb.a.as_deref().unwrap_or("").split(',').next().unwrap_or("");
            if !k.is_empty() && k != "unknown" && !holds(k) {
                "no item"
            } else if v.foes.is_empty() {
                "no target"
            } else {
                "no line"
            }
        }
        "smoke_retreat" => {
            if !class_has_verb(run.hero.class,run.hero.level,"smoke_retreat") {"class locked"}
            else if run.gun_skills.as_ref().is_some_and(|s|s.smoke_ready>run.turn) {"cooldown"}else {"no path"}
        }
        "aimed_shot" => {
            if !class_has_verb(run.hero.class,run.hero.level,"aimed_shot") {"class locked"}
            else if crate::firearm::Profile::of(run.hero.weapon_kind()).is_none_or(|p|p.capacity!=1) {"long gun only"}
            else if run.gun_reload.is_some() {"reloading"}
            else if run.gun_skills.as_ref().and_then(|s|s.aim).is_some() {"aim held"}else {"no target"}
        }
        "fast_reload" => {
            if !class_has_verb(run.hero.class,run.hero.level,"fast_reload") {"class locked"}
            else if run.gun_skills.as_ref().is_some_and(|s|s.fast_ready>run.turn) {"cooldown"}
            else if run.gun_reload.is_some() {"reloading"}else {"gun full"}
        }
        "finishing_shot" => {
            if !class_has_verb(run.hero.class,run.hero.level,"finishing_shot") {"class locked"}
            else if run.gun_skills.as_ref().is_some_and(|s|s.finish_ready>run.turn) {"cooldown"}
            else if let Some(mi)=pick_target_from(run,verb.a.as_deref().unwrap_or("nearest"),&v.foes) {
                if i64::from(run.monsters[mi].hp)*4>i64::from(run.monsters[mi].max_hp) {"foe too healthy"}else {"no line"}
            }else {"no enemy"}
        }
        "fire" | "close_burst" => {
            if crate::firearm::Profile::of(run.hero.weapon_kind()).is_none() { "no gun" }
            else if run.gun_reload.is_some() { "reloading" }
            else if run.hero.weapon.as_ref().and_then(|w|w.firearm).is_none_or(|c|c.loaded < if verb.v=="close_burst" {2}else{1}) { "empty gun" }
            else if let Some(mi)=pick_target_from(run,verb.a.as_deref().unwrap_or("nearest"),&v.foes) {
                let p=crate::firearm::Profile::of(run.hero.weapon_kind()).unwrap();let m=&run.monsters[mi];
                if m.pos.cheb(run.hero.pos)>p.range {"too far"}
                else if !run.floor.map.is_visible(m.pos)||!run.floor.map.los(run.hero.pos,m.pos) {"no sight"}
                else if verb.v=="close_burst"&&p.capacity!=2 {"no short gun"}else {"no use"}
            }else {"no enemy"}
        }
        "reload" => {
            if crate::firearm::Profile::of(run.hero.weapon_kind()).is_none() {"no gun"}
            else if run.gun_reload.is_some() {"reloading"}
            else {"gun full"}
        }
        "shoot" | "volley" | "double_shot" => {
            if !run.hero.ranged() {
                "no bow"
            } else if v.foes.is_empty() {
                "no target"
            } else {
                "no line"
            }
        }
        "tame" => {
            if !holds("leash") {
                "no leash"
            } else {
                "none weak"
            }
        }
        "rest" => "not safe",
        "descend" if run.homeward.is_some() => "going home",
        "descend" => "no stairs",
        "pray" => {
            if run.prayed {
                "prayed"
            } else {
                "no shrine"
            }
        }
        "return" | "bank" | "recall" => "no way",
        _ if crate::turn::targets_foes(verb) => {
            // QA on 912e135 (qaW: `attack ✗ no target` with a jackal and a goblin on screen; `R5 attack nearest · no target` beside
            // `foes 2`): foes in view that a melee row will not go for say why — running, or given up on
            if v.engage.is_empty() && !v.foes.is_empty() && !matches!(verb.v.as_str(), "shoot" | "volley" | "double_shot" | "throw" | "bolt" | "slow" | "drain" | "mark") {
                if v.foes.iter().all(|&i| run.monsters[i].fleeing || run.monsters[i].fear > 0) {
                    "foes fleeing"
                } else {
                    "given up"
                }
            } else if v.engage.is_empty() {
                "no target"
            } else if class_cooldown(run, verb) {
                "cooldown"
            } else {
                "no path"
            }
        }
        _ => "no use",
    }
}

/// Cut 5 §4: step toward this floor's unopened vault once it has been seen.
fn vault_step(run: &mut Run, cx: &mut Ctx) -> bool {
    if run.vault_cage.is_empty() {
        return false;
    }
    let Some(goal) = run.tile_pos(Tile::Vault) else { return false };
    if !run.floor.map.is_seen(goal) || run.hero.pos == goal {
        return false;
    }
    let parent = hero_path_to(run, goal);
    step_towards(run, cx, goal, &parent)
}

fn bones_step(run: &mut Run, cx: &mut Ctx) -> bool {
    let Some(goal) = run.items.iter().find(|fi| fi.item.kind == "bones").map(|fi| fi.pos) else { return false };
    let map = &run.floor.map;
    let (_, parent) = map.bfs_parent(run.hero.pos, false, &|p| map.get(p) == Tile::Water && p != run.hero.pos);
    if map.first_step(&parent, run.hero.pos, goal).is_none() {
        let (_, parent) = map.bfs_parent(run.hero.pos, false, &|_| false);
        return step_towards(run, cx, goal, &parent);
    }
    step_towards(run, cx, goal, &parent)
}

/// Cut 3: a chore step (the stairs when known and open, else the nearest frontier) along a path
/// that never comes within a tile of the monsters in `avoid`. False when no such path exists.
fn route_around(run: &mut Run, cx: &mut Ctx, avoid: &[usize]) -> bool {
    let hp = run.hero.pos;
    let spots: Vec<Pos> = avoid.iter().map(|&i| run.monsters[i].pos).collect();
    let map = &run.floor.map;
    let (dist, parent) = map.bfs_parent(hp, true, &|p| spots.iter().any(|m| m.cheb(p) <= 1) || (map.get(p) == Tile::Water && p != hp));
    let stairs = run.floor.stairs_down;
    let mut goal: Option<(i32, Pos)> = None;
    if map.is_seen(stairs) && !stairs_sealed(run) && dist[map.idx(stairs)] > 0 && !cx.rules.rows.is_empty() && map.seen_pct() >= CHORE_DESCEND_SEEN {
        goal = Some((dist[map.idx(stairs)], stairs));
    }
    if goal.is_none() {
        for (i, d) in dist.iter().enumerate() {
            if *d <= 0 {
                continue;
            }
            let p = map.pos(i);
            if is_frontier(run, p) && goal.is_none_or(|(gd, _)| *d < gd) {
                goal = Some((*d, p));
            }
        }
    }
    if goal.is_none() && map.is_seen(stairs) && !stairs_sealed(run) && dist[map.idx(stairs)] > 0 {
        goal = Some((dist[map.idx(stairs)], stairs));
    }
    let Some((_, goal)) = goal else { return false };
    if goal == stairs && hp == stairs {
        descend(run, cx);
        return true;
    }
    step_towards(run, cx, goal, &parent)
}

fn near_of(run: &Run, idx: &[usize], hp: Pos) -> usize {
    idx.iter().copied().min_by_key(|&i| (run.monsters[i].pos.cheb(hp), run.monsters[i].id)).unwrap()
}

fn can_step_clear_of(run: &Run, mi: usize) -> bool {
    clear_step(run, mi).is_some()
}

/// Cut 3: a step that gains ground on monster `mi` without walking into a dead end — the
/// neighbour farthest from it by path whose side of the map (with the monster's tile blocked)
/// still has room. False when no neighbour gains anything.
fn step_clear_of(run: &mut Run, cx: &mut Ctx, mi: usize) -> bool {
    match clear_step(run, mi) {
        Some(q) => {
            move_hero(run, cx, q);
            true
        }
        None => false,
    }
}

fn clear_step(run: &Run, mi: usize) -> Option<Pos> {
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    let map = &run.floor.map;
    let from_foe = map.bfs(mp, false, &|_| false);
    let cur = from_foe[map.idx(hp)];
    let mut best: Option<(i32, Pos)> = None;
    for d in DIRS8 {
        let q = hp.step(d);
        if !map.can_step(hp, q) || run.occupied(q) || in_hazard(run, q) {
            continue;
        }
        let far = from_foe[map.idx(q)];
        if far < 0 || far < cur {
            continue;
        }
        let room = map.bfs(q, false, &|p| p == mp || run.monster_at(p).is_some_and(|k| run.monsters[k].hostile())).iter().filter(|x| **x >= 0).count() as i32;
        let score = far * 100 + room.min(60) - if map.is_corridor(q) { 5 } else { 0 };
        if room >= 12 && best.is_none_or(|(b, _)| score > b) {
            best = Some((score, q));
        }
    }
    best.map(|(_, q)| q).filter(|q| *q != hp)
}

/// Cut 3: straight to the stairs over the whole map (a chalked floor); descends on arrival.
fn chalk_step(run: &mut Run, cx: &mut Ctx) -> bool {
    let goal = run.floor.stairs_down;
    if run.hero.pos == goal {
        descend(run, cx);
        return true;
    }
    let map = &run.floor.map;
    let (_, parent) = map.bfs_parent(run.hero.pos, false, &|p| map.get(p) == Tile::Water && p != run.hero.pos);
    if map.first_step(&parent, run.hero.pos, goal).is_none() {
        let (_, parent) = map.bfs_parent(run.hero.pos, false, &|_| false);
        return step_towards(run, cx, goal, &parent);
    }
    step_towards(run, cx, goal, &parent)
}

/// A living boss seals the way down: the wall is impassable until the policy beats it.
pub fn stairs_sealed(run: &Run) -> bool {
    run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.is_boss())
}

fn descend_step(run: &mut Run, cx: &mut Ctx) -> bool {
    let s = run.floor.stairs_down;
    if stairs_sealed(run) {
        // The wall: walk up to it (the boss is there to be seen and fought), never through.
        if !run.floor.map.is_seen(s) || run.hero.pos.cheb(s) <= 2 {
            return false;
        }
        let parent = hero_path_to(run, s);
        return step_towards(run, cx, s, &parent);
    }
    if run.hero.pos == s {
        descend(run, cx);
        return true;
    }
    if !run.floor.map.is_seen(s) {
        return false;
    }
    let parent = hero_path_to(run, s);
    if !step_towards(run, cx, s, &parent) {
        return false;
    }
    // The step that lands on the stairs goes down in the same action: with two den thieves
    // circling at one and two tiles, `foes ≥ 2 → to corridor` took every other action and the
    // hero never had the stairs' turn (cohort 10, rater T: the stall on D4).
    if run.hero.pos == s && run.over.is_none() {
        // The den's pounce is "before the hero acts on the stairs": it gets its moment here.
        let snatched = run.stolen.len();
        crate::situations::before_action(run, cx);
        if run.stolen.len() == snatched && run.over.is_none() {
            descend(run, cx);
        }
    }
    true
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
        "fire" => fire_gun(run, cx, &a, v, false),
        "reload" => crate::firearm::reload(run, cx),
        "close_burst" => class_has_verb(run.hero.class,run.hero.level,"close_burst") && fire_gun(run, cx, &a, v, true),
        "aimed_shot" => aim_gun(run,cx,&a,v),
        "smoke_retreat" => smoke_gun(run,cx,v),
        "fast_reload" => crate::firearm::reload_fast(run,cx),
        "finishing_shot" => class_has_verb(run.hero.class,run.hero.level,"finishing_shot")&&fire_gun_mode(run,cx,&a,v,false,true),
        "gunner_tactic" => gunner_tactic(run,cx,v),
        "shield_bash" => class_has_verb(run.hero.class, run.hero.level, "shield_bash") && run.hero.bash_cd == 0 && verb_attack(run, cx, "nearest", v, true),
        "riposte" => {
            if crate::specialization::has(&run.hero,crate::specialization::Style::Sentinel)&&run.hero.special_cd==0&&v.adj>0 {
                run.hero.riposte_t=crate::specialization::RIPOSTE_DURATION;run.hero.special_cd=crate::specialization::COOLDOWN;callout(run,cx,"riposte ready");true
            }else {false}
        }
        "hex" => {
            if !crate::specialization::has(&run.hero,crate::specialization::Style::Hexbinder)||run.hero.special_cd>0 {return false;}
            let Some(mi)=pick_target_from(run,&a,&v.foes) else {return false;};
            let m=&run.monsters[mi];
            if m.hex_t>0||!m.hostile()||m.hp<=0||m.pos.cheb(run.hero.pos)>BOW_RANGE||!run.floor.map.los(run.hero.pos,m.pos) {return false;}
            run.last_target=Some(run.monsters[mi].id);
            run.monsters[mi].hex_t=crate::specialization::HEX_DURATION;run.hero.special_cd=crate::specialization::COOLDOWN;callout(run,cx,"hexed");true
        }
        "retreat" => verb_retreat(run, cx, v),
        "back_corridor" => verb_back_corridor(run, cx, v),
        "drink" => verb_drink(run, cx, &a),
        "read" => verb_read(run, cx, &a, v),
        "throw" => (class_has_verb(run.hero.class, run.hero.level, "throw") || cx.unlocks.contains("throw")) && verb_throw(run, cx, &a, v),
        "cleave" => class_has_verb(run.hero.class, run.hero.level, "cleave") && verb_cleave(run, cx, v),
        "taunt" => class_has_verb(run.hero.class, run.hero.level, "taunt") && verb_taunt(run, cx, v),
        "second_wind" => {
            if class_has_verb(run.hero.class, run.hero.level, "second_wind") && !run.hero.second_wind_used && run.hero.hp < run.hero.max_hp {
                run.hero.second_wind_used = true;
                let add = run.hero.max_hp * 3 / 10;
                run.hero.hp = (run.hero.hp + add).min(run.hero.max_hp);
                callout(run, cx, "second wind");
                true
            } else {
                false
            }
        }
        "bulwark" => {
            if class_has_verb(run.hero.class, run.hero.level, "bulwark") && run.hero.bulwark_cd == 0 && !v.foes.is_empty() {
                run.hero.bulwark_t = 30;
                run.hero.bulwark_cd = 80;
                callout(run, cx, "bulwark");
                true
            } else {
                false
            }
        }
        "backstab" => class_has_verb(run.hero.class, run.hero.level, "backstab") && verb_backstab(run, cx, v, 2),
        "ambush" => class_has_verb(run.hero.class, run.hero.level, "ambush") && run.hero.vanish_t > 0 && verb_backstab(run, cx, v, 3),
        "smoke" => {
            if class_has_verb(run.hero.class, run.hero.level, "smoke") && !v.foes.is_empty() && run.hero.vanish_cd <= 90 {
                let hp = run.hero.pos;
                for m in run.monsters.iter_mut() {
                    if m.hostile() && m.pos.cheb(hp) <= 2 {
                        m.blind = 30;
                        m.last_seen = None;
                    }
                }
                run.hero.vanish_cd += 30;
                callout(run, cx, "smoke");
                true
            } else {
                false
            }
        }
        "shadowstep" => class_has_verb(run.hero.class, run.hero.level, "shadowstep") && verb_shadowstep(run, cx, v),
        "descend" => {
            if stairs_sealed(run) || run.homeward.is_some() {
                false
            } else if run.hero.pos == run.floor.stairs_down {
                descend(run, cx);
                true
            } else if descend_step(run, cx) {
                true
            } else {
                explore_step(run, cx)
            }
        }
        // Cut 19 §2: a return walks like a bank — to this floor's up-stairs, where it exits at
        // 60 % (`ExitTier::Return`). Foes can reach him on the way: a hurt hero walking home is
        // the run's tension, not a door out of it (rater AB: `hp < 20% → return` made death 1 %).
        // The player's own `bail` stays instant (`turn::choose_and_act`), as does a recall scroll.
        "bank" | "return" => {
            let tier = if verb.v == "bank" { ExitTier::Bank } else { ExitTier::Return };
            let s = run.floor.stairs_up;
            if run.hero.pos == s {
                end_run(run, cx, tier);
                true
            } else {
                let parent = hero_path_home(run, s);
                step_towards(run, cx, s, &parent)
            }
        }
        "rest" => cx.variant != "no_rest" && verb_rest(run, cx, v),
        "pick_up" => !run.items_ignored() && nearest_item_step(run, cx, false),
        "free_captive" => verb_free_captive(run, cx),
        "vanish" => {
            if class_has_verb(run.hero.class, run.hero.level, "vanish") && run.hero.vanish_cd == 0 && !v.foes.is_empty() {
                run.hero.vanish_t = 30;
                run.hero.vanish_cd = 120;
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
        "hold" => true,
        // Cut 5 §4: the shrine.
        "pray" => cx.facts.contains("shrine") && verb_pray(run, cx, &a),
        // Cut 2 §4: ranger
        "shoot" => class_has_verb(run.hero.class, run.hero.level, "shoot") && verb_shoot(run, cx, &a, v),
        "kite" => class_has_verb(run.hero.class, run.hero.level, "kite") && verb_kite(run, cx, v),
        "volley" => class_has_verb(run.hero.class, run.hero.level, "volley") && verb_volley(run, cx, v),
        "trap" => class_has_verb(run.hero.class, run.hero.level, "trap") && verb_trap(run, cx, v),
        "mark" => class_has_verb(run.hero.class, run.hero.level, "mark") && verb_mark(run, cx, &a, v),
        "double_shot" => class_has_verb(run.hero.class, run.hero.level, "double_shot") && verb_double_shot(run, cx, &a, v),
        // Cut 2 §4: caster
        "bolt" => class_has_verb(run.hero.class, run.hero.level, "bolt") && verb_bolt(run, cx, &a, v),
        "ward" => {
            if class_has_verb(run.hero.class, run.hero.level, "ward") && run.hero.ward_cd == 0 && !v.foes.is_empty() {
                run.hero.ward_t = 30;
                run.hero.ward_cd = 100;
                callout(run, cx, "ward");
                true
            } else {
                false
            }
        }
        "blink" => class_has_verb(run.hero.class, run.hero.level, "blink") && verb_blink(run, cx, v),
        "slow" => class_has_verb(run.hero.class, run.hero.level, "slow") && verb_slow(run, cx, &a, v),
        "nova" => class_has_verb(run.hero.class, run.hero.level, "nova") && verb_nova(run, cx, v),
        "drain" => class_has_verb(run.hero.class, run.hero.level, "drain") && verb_drain(run, cx, v),
        _ => false,
    }
}

/// Rest: +4 HP with no foe in view, not poisoned, not in a hazard. Every rest is a noise.
fn verb_rest(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if run.hero.hp < run.hero.max_hp && v.foes.is_empty() && run.hero.poison.1 == 0 && !in_hazard(run, run.hero.pos) {
        // Cut 30: `rested` — a live gift heals more on a rest.
        let extra = crate::traits::rest_extra_now(run, cx);
        let inherited=i32::from(crate::legacy::has(&run.hero,crate::legacy::RESTORATION));
        run.hero.hp = (run.hero.hp + 4 + extra + inherited).min(run.hero.max_hp);
        crate::turn::rest_clock(run, cx);
        true
    } else {
        false
    }
}

/// Cut 5 §4: `pray row|trait` — walk to the shrine seen on this floor and pray once per run.
fn verb_pray(run: &mut Run, cx: &mut Ctx, a: &str) -> bool {
    // Cut 7 §3: the hunger's shrine can be lit whatever was prayed above.
    let hunger_unlit = crate::situations::hunger_floor(run) && !run.lit;
    if run.prayed && !hunger_unlit {
        return false;
    }
    let Some(goal) = run.tile_pos(Tile::Shrine) else { return false };
    if !run.floor.map.is_seen(goal) {
        return false;
    }
    if run.hero.pos == goal {
        crate::turn::pray(run, cx, a != "trait");
        return true;
    }
    let parent = hero_path_to(run, goal);
    step_towards(run, cx, goal, &parent)
}

/// Cut 3: the Mirror King reflects a verb used three times running (the ring holds the last
/// two; `verb` would be the third).
fn mirror_reflects(run: &Run, mi: usize, verb: &str) -> bool {
    let m = &run.monsters[mi];
    m.kind == "mirror_king" && crate::endgame::repeated(&run.verb_ring, verb, m.modifiers.is_some_and(|mods| mods.tight_mirror))
}

/// The blow comes back and the mirror keeps its measure: he heals what he sent back.
fn mirror_learn(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let kind = run.monsters[mi].kind.clone();
    learn_tag(run, cx, &kind, "mirror");
    crate::facts::learn_boss_counter(run, cx, &kind);
    callout(run, cx, "mirrored!");
}

/// Cut 23 §1 (the forge's full kit broke the D33 wall without its counter — the harder the
/// blow, the faster he fell between his mirrors): he heals twice what he sends back, so a
/// rhythm that repeats gains nothing on him however hard it hits.
pub const MIRROR_HEAL: i32 = 2;

fn mirror_heal(run: &mut Run, mi: usize, dmg: i32) {
    let m = &mut run.monsters[mi];
    m.hp = (m.hp + MIRROR_HEAL * dmg.max(0)).min(m.max_hp);
}

/// A deterministic reactive strike still obeys the King's current rhythm and
/// ordinary melee reflections. It does not overwrite the next hero action's verb.
pub(crate) fn riposte_hit(run:&mut Run,cx:&mut Ctx,mi:usize) {
    let id=run.monsters[mi].id;let dmg=crate::specialization::COUNTER_DAMAGE;run.melee_used=true;
    if mirror_reflects(run,mi,"riposte") {
        cx.events.push(Ev::Attack{t:run.turn,src:id,dst:HERO_ID,dmg,hit:true,verb:Some("mirror".into())});
        mirror_learn(run,cx,mi);mirror_heal(run,mi,dmg);damage_hero(run,cx,dmg,&Src::Reflect(mi));
    }else {
        cx.events.push(Ev::Attack{t:run.turn,src:HERO_ID,dst:id,dmg,hit:true,verb:Some("riposte".into())});
        callout(run,cx,"riposte");damage_monster(run,cx,mi,dmg,&Src::Hero{ranged:false});
    }
}

/// Cut 5: the Mirror King mirrors the pack too — an ally's third blow of a kind in a row
/// comes back on it (and heals him). Returns true when the blow was sent back.
fn ally_mirror(run: &mut Run, cx: &mut Ctx, mi: usize, ti: usize, verb: &str, hit: bool, dmg: i32) -> bool {
    if run.monsters[ti].kind != "mirror_king" {
        return false;
    }
    let tight = run.monsters[ti].modifiers.is_some_and(|mods| mods.tight_mirror);
    let ring = &mut run.monsters[mi].verb_ring;
    let third = crate::endgame::repeated(ring, verb, tight);
    ring.push(verb.to_string());
    while ring.len() > 3 {
        ring.remove(0);
    }
    if !third {
        return false;
    }
    let (src, dst) = (run.monsters[ti].id, run.monsters[mi].id);
    cx.events.push(Ev::Attack { t: run.turn, src, dst, dmg, hit, verb: Some("mirror".into()) });
    if run.floor.map.is_visible(run.monsters[ti].pos) {
        mirror_learn(run, cx, ti);
    }
    if hit {
        mirror_heal(run, ti, dmg);
        damage_monster(run, cx, mi, dmg, &Src::Reflect(ti));
    }
    true
}

// ---------------------------------------------------------------- ranger (Cut 2 §4)

/// A shot at `sel` within bow range and line of sight, else a step toward it. Needs a bow.
fn verb_shoot(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View) -> bool {
    if crate::firearm::Profile::of(run.hero.weapon_kind()).is_some() {
        return fire_gun(run, cx, sel, v, false);
    }
    if !run.hero.ranged() {
        return false;
    }
    let Some(mi) = pick_target(run, sel, v) else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
        hero_attack(run, cx, mi, "shoot", false);
        return true;
    }
    approach_target(run, cx, mi)
}

/// Keep range 3: step away from a foe closer than that, shoot one at range, else close a step.
fn verb_kite(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let Some(mi) = v.nearest else { return false };
    let hp = run.hero.pos;
    let d = run.monsters[mi].pos.cheb(hp);
    if d < KITE_RANGE && verb_retreat(run, cx, v) {
        return true;
    }
    if run.hero.ranged() {
        let mp = run.monsters[mi].pos;
        if mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
            hero_attack(run, cx, mi, "shoot", false);
            return true;
        }
    }
    if d > KITE_RANGE {
        return approach_target(run, cx, mi);
    }
    false
}

/// Every foe on the line through the nearest target, to bow range; cooldown 8 turns.
fn verb_volley(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if run.hero.volley_cd > 0 || !run.hero.ranged() {
        return false;
    }
    let Some(mi) = v.nearest else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if !run.floor.map.los(hp, mp) {
        return false;
    }
    let (dx, dy) = ((mp.x - hp.x).signum(), (mp.y - hp.y).signum());
    let mut targets = Vec::new();
    let mut p = hp;
    for _ in 0..BOW_RANGE {
        p = p.step((dx, dy));
        if !run.floor.map.in_bounds(p) || run.floor.map.get(p) == Tile::Wall {
            break;
        }
        if let Some(k) = run.monster_at(p) {
            if run.monsters[k].hostile() {
                targets.push(k);
            }
        }
    }
    if targets.is_empty() {
        return false;
    }
    run.hero.volley_cd = 80;
    callout(run, cx, "volley");
    for k in targets {
        if run.monsters[k].hp > 0 {
            hero_attack(run, cx, k, "shoot", false);
        }
    }
    true
}

/// Lay a trap on the tile toward the nearest foe: the first hostile onto it is stunned 20 ticks.
fn verb_trap(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let Some(mi) = v.nearest else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.adjacent(hp) {
        return false;
    }
    let q = hp.step(((mp.x - hp.x).signum(), (mp.y - hp.y).signum()));
    if !run.floor.map.passable(q) || run.occupied(q) || run.item_at(q).is_some() {
        return false;
    }
    let id = run.new_item_id();
    run.items.push(crate::engine::FloorItem { pos: q, item: Item::new(id, "trap") });
    callout(run, cx, "trap set");
    true
}

/// Mark a target: it takes ×1.5 from the hero for 30 ticks.
fn verb_mark(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View) -> bool {
    let Some(mi) = pick_target(run, sel, v) else { return false };
    if run.monsters[mi].marked > 0 {
        return false;
    }
    run.monsters[mi].marked = 30;
    callout(run, cx, "marked");
    true
}

fn verb_double_shot(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View) -> bool {
    if run.hero.double_cd > 0 || !run.hero.ranged() {
        return false;
    }
    let Some(mi) = pick_target(run, sel, v) else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.cheb(hp) > BOW_RANGE || !run.floor.map.los(hp, mp) {
        return false;
    }
    run.hero.double_cd = 40;
    crate::provenance::cooldown(run, cx, "double_shot");
    callout(run, cx, "double shot");
    hero_attack(run, cx, mi, "shoot", false);
    if run.monsters.get(mi).is_some_and(|m| m.hp > 0) {
        hero_attack(run, cx, mi, "shoot", false);
    }
    true
}

/// Path to a tile adjacent to the target (shared by the ranged verbs).
fn approach_target(run: &mut Run, cx: &mut Ctx, mi: usize) -> bool {
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    // (the nearest free tile beside the target: `nearest_goal` floods only as far as it)
    let goals: Vec<Pos> = mp.neighbours8().into_iter().filter(|q| run.floor.map.in_bounds(*q) && (!run.occupied(*q) || *q == hp)).collect();
    let (goal, parent) = nearest_goal(run, &hero_avoids(run, true), &goals);
    match goal {
        Some(g) if g != hp => step_towards(run, cx, g, &parent),
        _ => false,
    }
}

// ---------------------------------------------------------------- caster (Cut 2 §4)

/// A bolt (2–5, no ammo) at `sel` within range and sight, else a step toward it.
fn verb_bolt(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View) -> bool {
    let Some(mi) = pick_target(run, sel, v) else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
        let def = run.monsters[mi].effective_def();
        let (hit, dmg) = roll_hit(&mut run.rng, (BOLT_ATK.0 + run.hero.str_bonus, BOLT_ATK.1 + run.hero.str_bonus), def);
        let id = run.monsters[mi].id;
        run.last_hit_verb = Some("bolt".into());
        projectile(run, cx, HERO_ID, id, hp, mp);
        if mirror_reflects(run, mi, "bolt") {
            cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit, verb: Some("mirror".into()) });
            mirror_learn(run, cx, mi);
            if hit {
                mirror_heal(run, mi, dmg);
                damage_hero(run, cx, dmg, &Src::Reflect(mi));
            }
            return true;
        }
        cx.events.push(Ev::Attack { t: run.turn, src: HERO_ID, dst: id, dmg, hit, verb: Some("bolt".into()) });
        if hit {
            let dmg = if run.monsters[mi].marked > 0 { dmg * 3 / 2 } else { dmg };
            damage_monster(run, cx, mi, dmg, &Src::Hero { ranged: true });
            if run.monsters.get(mi).is_some_and(|m| m.hp > 0 && !m.awake) {
                run.monsters[mi].awake = true;
                run.monsters[mi].last_seen = Some(run.hero.pos);
            }
        }
        return true;
    }
    approach_target(run, cx, mi)
}

/// Blink up to 3 tiles away from the foes (no scroll); cooldown 5 turns.
fn verb_blink(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if run.hero.blink_cd > 0 || v.foes.is_empty() {
        return false;
    }
    let hp = run.hero.pos;
    let map = &run.floor.map;
    let mut cands: Vec<Pos> = Vec::new();
    for dy in -3..=3 {
        for dx in -3..=3 {
            let q = hp.step((dx, dy));
            if q != hp && map.passable(q) && !run.occupied(q) && map.los(hp, q) && !in_hazard(run, q) {
                cands.push(q);
            }
        }
    }
    let cur = min_foe_dist(run, v, hp);
    let Some(q) = cands.iter().max_by_key(|q| (min_foe_dist(run, v, **q), -q.cheb(hp), q.x, q.y)).copied() else { return false };
    if min_foe_dist(run, v, q) <= cur {
        return false;
    }
    run.hero.blink_cd = 50;
    move_hero(run, cx, q);
    callout(run, cx, "blink");
    true
}

/// Slow a target (speed −5) for 30 ticks.
fn verb_slow(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View) -> bool {
    let Some(mi) = pick_target(run, sel, v) else { return false };
    if run.monsters[mi].slow_t > 0 {
        return false;
    }
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.cheb(hp) > BOW_RANGE || !run.floor.map.los(hp, mp) {
        return false;
    }
    run.monsters[mi].slow_t = 30 + 5*i32::from(crate::legacy::has(&run.hero,crate::legacy::CONTROL))
        + 15*i32::from(crate::legacy::has(&run.hero,crate::legacy::DEBILITATE));
    callout(run, cx, "slowed");
    true
}

/// Fire on the eight tiles around the caster; cooldown 15 turns.
fn verb_nova(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if run.hero.nova_cd > 0 || v.adj == 0 {
        return false;
    }
    let hp = run.hero.pos;
    run.hero.nova_cd = 150;
    callout(run, cx, "nova");
    for q in hp.neighbours8() {
        if run.floor.map.in_bounds(q) && run.floor.map.passable(q) {
            place_overlay(run, cx, q, 0, OverlayKind::Fire, 20);
        }
    }
    // The blast lands at once on whoever stands in it.
    for q in hp.neighbours8() {
        if let Some(k) = run.monster_at(q) {
            if run.monsters[k].hostile() {
                damage_monster(run, cx, k, 5, &Src::Fire);
            }
        }
    }
    true
}

/// Steal 5 hp from an adjacent foe.
fn verb_drain(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let hp = run.hero.pos;
    let Some(mi) = v.foes.iter().copied().find(|&i| run.monsters[i].pos.adjacent(hp)) else { return false };
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Attack { t: run.turn, src: HERO_ID, dst: id, dmg: 5, hit: true, verb: Some("drain".into()) });
    let before = run.monsters[mi].hp;
    damage_monster(run, cx, mi, 5, &Src::Hero { ranged: false });
    let taken = before - run.monsters.get(mi).map(|m| m.hp.max(0)).unwrap_or(0);
    run.hero.hp = (run.hero.hp + taken.clamp(0, 5)).min(run.hero.max_hp);
    callout(run, cx, "drain");
    true
}

/// Attack targets: fleeing foes are not chased unless adjacent (a fool's errand). Cut 4: a
/// melee pick draws from `engage` (foes the hero has not given up on); a ranged pick from
/// every foe in view (an archer out of reach on foot is still in bow range).
fn pick_target(run: &mut Run, a: &str, v: &View) -> Option<usize> {
    let mi = pick_target_from(run, a, &v.foes);
    run.last_target = mi.map(|i| run.monsters[i].id);
    mi
}

fn pick_melee_target(run: &mut Run, a: &str, v: &View) -> Option<usize> {
    let mut mi = pick_target_from(run, a, &v.engage);
    // Cut 5 §4: with nothing awake to fight, a melee row may raid a sleeping den (a free
    // blow, and the den wakes) — `on_see: nest → attack` is the raid; `foes>=` never sees
    // sleepers, so the default set walks past.
    // Cut 7 §3: the thief's den is raided only by a row that names it.
    if mi.is_none() && v.engage.is_empty() {
        let map = &run.floor.map;
        let raiding = run.raiding;
        let sleepers: Vec<usize> = (0..run.monsters.len()).filter(|&i| run.monsters[i].hp > 0 && run.monsters[i].hostile() && run.monsters[i].dormant && map.is_visible(run.monsters[i].pos) && (raiding || run.monsters[i].situation.is_none())).collect();
        mi = pick_target_from(run, a, &sleepers);
    }
    run.last_target = mi.map(|i| run.monsters[i].id);
    mi
}

fn pick_target_from(run: &Run, a: &str, foes: &[usize]) -> Option<usize> {
    let hp = run.hero.pos;
    let ok = |i: &usize| {
        let m = &run.monsters[*i];
        !(m.fleeing || m.fear > 0) || m.pos.adjacent(hp)
    };
    // An adjacent match beats a distant one: never walk past a foe that is already biting.
    let adjacent = |i: &usize| run.monsters[*i].pos.adjacent(hp);
    match a {
        "lowest" => foes.iter().copied().filter(ok).min_by_key(|&i| (!adjacent(&i), run.monsters[i].hp, run.monsters[i].id)),
        s if s.starts_with("tag:") => {
            let t = &s[4..];
            let tagged = |i: &usize| run.monsters[*i].has_tag(t);
            foes.iter().copied().filter(ok).filter(tagged).find(adjacent).or_else(|| foes.iter().copied().filter(ok).find(tagged))
        }
        _ => {
            // Nearest: adjacent first; among those the boss comes last (its guards are the
            // wall; the boss is a deliberate target via `tag:boss`).
            let key = |i: &usize| (!adjacent(i), run.monsters[*i].is_boss(), run.monsters[*i].pos.cheb(hp), run.monsters[*i].id);
            foes.iter().copied().filter(ok).min_by_key(key)
        }
    }
}

fn verb_attack(run: &mut Run, cx: &mut Ctx, a: &str, v: &View, bash: bool) -> bool {
    if !bash && crate::firearm::Profile::of(run.hero.weapon_kind()).is_some() {
        if let Some(mi)=pick_target_from(run,a,&v.foes) {
            if gun_sidearm(run,cx,mi) {return true;}
        }
        if fire_gun(run, cx, a, v, false) { return true; }
        // Authored attack may approach; it never silently refills an empty gun.
        let Some(mi) = pick_target_from(run, a, &v.foes) else { return false; };
        let range = crate::firearm::Profile::of(run.hero.weapon_kind()).unwrap().range;
        if run.monsters[mi].pos.cheb(run.hero.pos) > range { return approach_target(run, cx, mi); }
        return false;
    }
    let Some(mi) = pick_melee_target(run, a, v) else { return false };
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    // Cut 3: a spear reaches two tiles.
    let reach = mp.adjacent(hp) || (!bash && run.hero.weapon_kind() == "spear" && mp.cheb(hp) == 2 && run.floor.map.los(hp, mp));
    if reach {
        run.chase = None;
        // An aimed strike (`attack tag:boss`) or a bash goes through the Warlord's guards.
        run.aimed = a == "tag:boss" || bash;
        hero_attack(run, cx, mi, if bash { "shield_bash" } else { "attack" }, bash);
        run.aimed = false;
        return true;
    }
    // Chase progress: three approaches without closing the distance and the target is
    // given up on for 30 actions (the row falls through).
    let id = run.monsters[mi].id;
    let d = mp.cheb(hp);
    let boss = run.monsters[mi].is_boss();
    let stalled = match run.chase {
        Some((cid, last, n)) if cid == id => {
            if d < last {
                run.chase = Some((id, d, 0));
                false
            } else {
                run.chase = Some((id, last.min(d), n + 1));
                n + 1 >= 3
            }
        }
        _ => {
            run.chase = Some((id, d, 0));
            false
        }
    };
    if stalled && !boss {
        // A boss is always worth the chase; anything else that will not close is given up on.
        run.ignore(id, 30);
        run.chase = None;
        return false;
    }
    if bash {
        return false;
    }
    if run.hero.ranged() && mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
        hero_attack(run, cx, mi, "shoot", false);
        return true;
    }
    // Hold a corridor while an awake foe keeps closing in, rather than stepping out to meet it.
    // Once the foe stops moving for three actions (a lurking pack), go and get it.
    let d = mp.cheb(hp);
    if run.floor.map.is_corridor(hp) && run.monsters[mi].awake && d <= 5 && !run.monsters[mi].has_tag("ranged") {
        if d == run.hold_dist {
            run.hold_streak += 1;
        } else {
            run.hold_dist = d;
            run.hold_streak = 0;
        }
        if run.hold_streak < 3 {
            return true;
        }
    } else {
        run.hold_dist = -1;
        run.hold_streak = 0;
    }
    // Approach: path to a tile adjacent to the target, around other monsters (Cut 7: and
    // round lingering gas or fire) if there is a way, otherwise straight through whoever
    // stands in the way.
    // The goal is the nearest free tile beside the target by (distance, x, y) — on the flood round
    // the others when one is reachable that way, else on the hero's plain flood's; each flood stops at its
    // nearest (`nearest_goal`). The hero beside the target is his own nearest (distance 0): no
    // step, whichever flood.
    if mp.neighbours8().contains(&hp) {
        return false;
    }
    let goals: Vec<Pos> = mp.neighbours8().into_iter().filter(|q| run.floor.map.in_bounds(*q) && !run.occupied(*q)).collect();
    let around = |p: Pos| run.monster_at(p).is_some_and(|k| k != mi && run.monsters[k].hostile()) || run.overlays.iter().any(|o| o.x == p.x && o.y == p.y);
    let (goal, parent) = match nearest_goal(run, &around, &goals) {
        (Some(g), parent) => (Some(g), parent),
        _ => nearest_goal(run, &hero_avoids(run, true), &goals),
    };
    match goal {
        Some(g) if g == hp => false,
        Some(g) => {
            // Something hostile on the first step (a guard in the way)? Cut through it.
            if let Some(q) = run.floor.map.first_step(&parent, hp, g) {
                if let Some(bi) = run.monster_at(q) {
                    if run.monsters[bi].hostile() {
                        hero_attack(run, cx, bi, "attack", false);
                        return true;
                    }
                }
            }
            step_towards(run, cx, g, &parent)
        }
        None => {
            // No known path: close in greedily along the line of sight.
            let q = DIRS8
                .iter()
                .map(|d| hp.step(*d))
                .filter(|q| run.floor.map.can_step(hp, *q) && !run.occupied(*q) && !run.overlays.iter().any(|o| o.x == q.x && o.y == q.y))
                .min_by_key(|q| (q.cheb(mp), q.x, q.y));
            match q {
                Some(q) if q.cheb(mp) < hp.cheb(mp) => {
                    move_hero(run, cx, q);
                    true
                }
                _ if run.overlays.iter().any(|o| mp.cheb(Pos::new(o.x, o.y)) <= 1) => {
                    // Cut 7: the target stands in gas or fire — wait it out, not walk in.
                    false
                }
                _ => {
                    // Unreachable: give up on it for a while.
                    run.ignore(id, 30);
                    run.chase = None;
                    false
                }
            }
        }
    }
}

pub fn hero_attack(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &str, bash: bool) -> bool {
    hero_attack_mult(run, cx, mi, verb, bash, 1)
}

pub fn hero_attack_mult(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &str, bash: bool, mult: i32) -> bool {
    // Every old ranged attack path must reserve ammunition too. Dedicated gun
    // actions call the shared hit resolver after their one chamber commitment.
    if let Some(profile) = crate::firearm::Profile::of(run.hero.weapon_kind()) {
        if !reserve_gun(run, mi, profile, false) { return false; }
        let prepared=profile.capacity==1&&run.gun_skills.as_ref().and_then(|s|s.aim).is_some_and(|a|
            a.target==run.monsters[mi].id&&a.from==run.hero.pos&&run.hero.weapon.as_ref().is_some_and(|w|w.id==a.item));
        if let Some(s)=run.gun_skills.as_mut() {s.aim=None;}
        let aimed=run.aimed;run.aimed|=prepared;
        hero_attack_roll(run, cx, mi, if prepared {"aimed_shot"}else{"fire"}, false, mult, Some((profile,prepared)));
        run.aimed=aimed;
        return true;
    }
    hero_attack_roll(run, cx, mi, verb, bash, mult, None);
    true
}

// Match the ordinary attack view: an adjacent chained gate is attackable,
// while distant captives and freed allies remain protected. Otherwise a gun
// can reach the stairs at full health and repeatedly refuse the only target.
fn gun_target_alive(run: &Run, m: &Monster) -> bool {
    let chained=m.neutral&&m.situation.as_deref()==Some("captive")&&m.pos.adjacent(run.hero.pos);
    m.hp>0&&(m.hostile()||chained)&&!m.dormant
}
fn reserve_gun(run: &mut Run, mi: usize, profile: crate::firearm::Profile, burst: bool) -> bool {
    if run.gun_reload.is_some() { return false; }
    let Some(m) = run.monsters.get(mi) else { return false; };
    let target = crate::firearm::ShotTarget {
        from: run.hero.pos, to: m.pos, hostile_alive: gun_target_alive(run,m),
        visible: run.floor.map.is_visible(m.pos), los: run.floor.map.los(run.hero.pos, m.pos),
    };
    let Some(mut chambers) = run.hero.weapon.as_ref().and_then(|w| w.firearm) else { return false; };
    if chambers.fire(profile, target, burst).is_err() { return false; }
    run.hero.weapon.as_mut().unwrap().firearm = Some(chambers);
    true
}

/// Pure target selection and complete validation precede chamber/RNG/memory
/// mutation. Spread has a fixed three-entry list and resolves ordinary hits.
pub(crate) fn fire_gun(run: &mut Run, cx: &mut Ctx, sel: &str, v: &View, burst: bool) -> bool {
    fire_gun_mode(run,cx,sel,v,burst,false)
}

fn aim_gun(run:&mut Run,cx:&mut Ctx,sel:&str,v:&View)->bool {
    if !class_has_verb(run.hero.class,run.hero.level,"aimed_shot")||run.gun_reload.is_some() {return false;}
    let Some(w)=run.hero.weapon.as_ref() else {return false;};
    let Some(p)=crate::firearm::Profile::of(&w.kind).filter(|p|p.capacity==1) else {return false;};
    let Some(mi)=pick_target_from(run,sel,&v.foes) else {return false;};
    let m=&run.monsters[mi];
    let aim=crate::firearm::Aim {item:w.id,target:m.id,from:run.hero.pos};
    if run.gun_skills.as_ref().and_then(|s|s.aim)==Some(aim) {return false;}
    let Some(mut c)=w.firearm else {return false;};
    if c.fire(p,crate::firearm::ShotTarget {from:run.hero.pos,to:m.pos,hostile_alive:gun_target_alive(run,m),
        visible:run.floor.map.is_visible(m.pos),los:run.floor.map.los(run.hero.pos,m.pos)},false).is_err() {return false;}
    run.gun_skills.get_or_insert_with(Default::default).aim=Some(aim);
    run.last_target=Some(m.id);callout(run,cx,"aim steady");true
}

fn smoke_gun(run:&mut Run,cx:&mut Ctx,v:&View)->bool {
    if !class_has_verb(run.hero.class,run.hero.level,"smoke_retreat")||
        run.gun_skills.as_ref().is_some_and(|s|s.smoke_ready>run.turn) {return false;}
    let Some(mi)=v.nearest else {return false;};
    let Some(q)=clear_step(run,mi) else {return false;};
    let Some(at)=run.turn.checked_add(crate::firearm::SMOKE_COOLDOWN) else {return false;};
    let hp=run.hero.pos;
    for m in &mut run.monsters {if m.hp>0&&m.hostile()&&m.pos.cheb(hp)<=2 {m.blind=m.blind.max(30);m.last_seen=None;}}
    run.gun_skills.get_or_insert_with(Default::default).smoke_ready=at;
    move_hero(run,cx,q);callout(run,cx,"smoke retreat");true
}

fn gun_sidearm(run:&mut Run,cx:&mut Ctx,mi:usize)->bool {
    if run.hero.class!=Class::Gunner||run.bow_swap.is_some()||
        crate::firearm::Profile::of(run.hero.weapon_kind()).is_none() {return false;}
    let m=&run.monsters[mi];
    if !m.reflects_ranged()||!cx.facts.contains(&format!("foe:{}:reflect",m.kind)) {return false;}
    let Some(i)=run.hero.inv.iter().enumerate().filter(|(_,w)|w.cat()==Cat::Weapon&&!w.def().ranged)
        .max_by_key(|(_,w)|{let(a,b)=w.atk();a+b}).map(|(i,_)|i) else {return false;};
    run.last_target=Some(m.id);
    crate::firearm::cancel_aim(run,cx);
    let sidearm=run.hero.inv.remove(i);
    run.bow_swap=run.hero.weapon.replace(sidearm);
    callout(run,cx,"sidearm");true
}
fn gunner_tactic(run:&mut Run,cx:&mut Ctx,v:&View)->bool {
    if run.hero.class!=Class::Gunner {return false;}
    if run.bow_swap.as_ref().is_some_and(|w|crate::firearm::Profile::of(&w.kind).is_some()) {
        let sel=if v.foes.iter().any(|&i|run.monsters[i].is_boss()) {"tag:boss"}else{"nearest"};
        return verb_attack(run,cx,sel,v,false);
    }
    let Some(w)=run.hero.weapon.as_ref() else {return false;};
    let Some(p)=crate::firearm::Profile::of(&w.kind) else {return false;};
    let Some(chambers)=w.firearm else {return false;};
    // Reload remains an absolute commitment while smoke buys breathing room.
    // Safety/player rows still run first; no free movement or implicit ammo refill.
    let hp_pct=run.hero.hp*100/run.hero.max_hp.max(1);
    if v.adj>0&&hp_pct<35&&(run.gun_reload.is_some()||chambers.loaded>0)&&smoke_gun(run,cx,v) {return true;}
    if run.gun_reload.is_some() {
        // Spend the otherwise idle action on ordinary breathing room. This
        // neither completes the reload nor adds Smoke's blindness/cooldown.
        if v.adj>0 {verb_retreat(run,cx,v);}
        return true;
    }
    if chambers.loaded==0 {return crate::firearm::reload_fast(run,cx)||crate::firearm::reload(run,cx);}
    if v.foes.is_empty() {return false;}
    let sel=if v.foes.iter().any(|&i|run.monsters[i].is_boss()&&run.monsters[i].pos.cheb(run.hero.pos)<=p.range) {"tag:boss"}else{"nearest"};
    if let Some(mi)=pick_target_from(run,sel,&v.foes) {
        if gun_sidearm(run,cx,mi) {return true;}
    }
    if class_has_verb(run.hero.class,run.hero.level,"finishing_shot")&&fire_gun_mode(run,cx,sel,v,false,true) {return true;}
    // Spend preparation/chambers on a shot opportunity, not on every sighting.
    // A normal shot already finishing a weak foe needs no setup; spread only
    // counts the same legal forward neighbours the actual shot can hit.
    if let Some(mi)=pick_target_from(run,sel,&v.foes) {
        let min_damage=|i:usize| (run.hero.atk().0-(run.monsters[i].effective_def()-p.armour_piercing).max(0)).max(1);
        let needs_damage=run.monsters[mi].hp>min_damage(mi);
        if p.capacity==2&&class_has_verb(run.hero.class,run.hero.level,"close_burst") {
            let primary=run.monsters[mi].pos;
            let spread=needs_damage||v.foes.iter().copied().any(|i|i!=mi&&run.monsters[i].hp>0&&run.monsters[i].hostile()&&
                !run.monsters[i].dormant&&crate::firearm::in_spread(run.hero.pos,primary,run.monsters[i].pos)&&
                run.floor.map.is_visible(run.monsters[i].pos)&&run.floor.map.los(run.hero.pos,run.monsters[i].pos)&&
                run.monsters[i].hp>min_damage(i));
            if spread&&fire_gun(run,cx,sel,v,true) {return true;}
        }
        if p.capacity==1&&needs_damage&&run.gun_skills.as_ref().and_then(|s|s.aim).is_none()&&aim_gun(run,cx,sel,v) {return true;}
    }
    fire_gun(run,cx,sel,v,false)||verb_attack(run,cx,sel,v,false)
}

fn fire_gun_mode(run:&mut Run,cx:&mut Ctx,sel:&str,v:&View,burst:bool,finish:bool)->bool {
    let Some(profile) = crate::firearm::Profile::of(run.hero.weapon_kind()) else { return false; };
    let Some(mi) = pick_target_from(run, sel, &v.foes) else { return false; };
    let finish_ready=if finish {
        let m=&run.monsters[mi];
        if i64::from(m.hp)*4>i64::from(m.max_hp)||run.gun_skills.as_ref().is_some_and(|s|s.finish_ready>run.turn) {return false;}
        let Some(at)=run.turn.checked_add(crate::firearm::FINISH_COOLDOWN) else {return false;};Some(at)
    }else {None};
    if !reserve_gun(run, mi, profile, burst) { return false; }
    let prepared=profile.capacity==1&&run.gun_skills.as_ref().and_then(|s|s.aim).is_some_and(|a|
        a.target==run.monsters[mi].id&&a.from==run.hero.pos&&run.hero.weapon.as_ref().is_some_and(|w|w.id==a.item));
    if let Some(s)=run.gun_skills.as_mut() {s.aim=None;}
    if let Some(at)=finish_ready {run.gun_skills.get_or_insert_with(Default::default).finish_ready=at;}
    run.last_target = Some(run.monsters[mi].id);
    let mut targets = [None; 3];
    targets[0] = Some(mi);
    if profile.capacity == 2 {
        let primary = run.monsters[mi].pos;
        let mut n = 1;
        for &i in &v.foes {
            let m = &run.monsters[i];
            if i != mi && m.hp > 0 && m.hostile() && !m.dormant &&
                crate::firearm::in_spread(run.hero.pos, primary, m.pos) &&
                run.floor.map.is_visible(m.pos) && run.floor.map.los(run.hero.pos, m.pos) {
                targets[n] = Some(i); n += 1;
                if n == targets.len() { break; }
            }
        }
    }
    let aimed = run.aimed;
    let verb = if finish {"finishing_shot"}else if prepared {"aimed_shot"}else if burst { "close_burst" } else { "fire" };
    for i in targets.into_iter().flatten() {
        if run.over.is_some() { break; }
        // A deliberate boss target cannot make incidental scatter pellets
        // aimed too: a secondary Warlord still has his ordinary shield wall.
        run.aimed = i == mi && (sel == "tag:boss"||prepared);
        hero_attack_roll(run, cx, i, verb, false, if burst||finish { 2 } else { 1 }, Some((profile,prepared)));
    }
    run.aimed = aimed;
    true
}

fn hero_attack_roll(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &str, bash: bool, mult: i32, gun: Option<(crate::firearm::Profile,bool)>) {
    let ranged = gun.is_some() || verb == "shoot";
    let atk = run.hero.atk();
    let atk = (atk.0 * mult, atk.1 * mult);
    let prepared=gun.is_some_and(|(_,prepared)|prepared);
    let atk=if prepared {(atk.0*3/2,atk.1*3/2)}else{atk};
    let def = run.monsters[mi].effective_def();
    let def = gun.map_or(def, |(p,_)| (def - p.armour_piercing).max(0));
    let (hit, dmg) = roll_hit_pct(&mut run.rng, atk, def, if prepared {100}else{run.hero.hit_pct()});
    let id = run.monsters[mi].id;
    run.last_hit_verb = Some(verb.into());
    if ranged {
        let (from, to) = (run.hero.pos, run.monsters[mi].pos);
        projectile(run, cx, HERO_ID, id, from, to);
        if run.monsters[mi].reflects_ranged() {
            // The arrow comes back.
            let kind = run.monsters[mi].kind.clone();
            cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit, verb: Some("reflect".into()) });
            learn_tag(run, cx, &kind, "reflect");
            callout(run, cx, "reflected!");
            if hit {
                damage_hero(run, cx, dmg, &Src::Reflect(mi));
            }
            return;
        }
    } else {
        // Cut 3: a swing is a noise (the Deep's hunters come for it).
        let at = run.hero.pos;
        noise(run, cx, at, 6);
    }
    // Cut 3: the Mirror King throws the third of a kind back.
    if mirror_reflects(run, mi, verb) {
        cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit, verb: Some("mirror".into()) });
        mirror_learn(run, cx, mi);
        if hit {
            mirror_heal(run, mi, dmg);
            damage_hero(run, cx, dmg, &Src::Reflect(mi));
        }
        return;
    }
    cx.events.push(Ev::Attack { t: run.turn, src: HERO_ID, dst: id, dmg, hit, verb: Some(verb.into()) });
    if !ranged {
        run.melee_used = true;
    }
    if bash {
        run.hero.bash_cd = 50;
        crate::provenance::cooldown(run, cx, "shield_bash");
    }
    if hit {
        if bash || (run.monsters[mi].kind == "goblin_warlord" && run.aimed) {
            run.monsters[mi].pending = None;
            run.monsters[mi].telegraph = None;
        }
        if bash {
            run.monsters[mi].stun = 10 + 5*i32::from(crate::legacy::has(&run.hero,crate::legacy::CONTROL));
            callout(run, cx, "bash");
        }
        // Cut 3: a mace stuns one hit in ten.
        if !bash && !ranged && run.hero.weapon_kind() == "mace" && run.rng.chance(10) {
            run.monsters[mi].stun = 10 + 5*i32::from(crate::legacy::has(&run.hero,crate::legacy::CONTROL));
            callout(run, cx, "stunned");
        }
        let dmg = if run.monsters[mi].marked > 0 { dmg * 3 / 2 } else { dmg };
        damage_monster(run, cx, mi, dmg, &Src::Hero { ranged });
        if run.monsters[mi].hp > 0 && !run.monsters[mi].awake {
            run.monsters[mi].awake = true;
            run.monsters[mi].last_seen = Some(run.hero.pos);
        }
    }
}

/// Cut 16: the foes a retreat runs from — every foe in view but those the oscillation guard
/// gave up on for the floor (one at the hero's elbow always counts). A conjurer, a lurking
/// jackal and a thief holding at three tiles kept `foes ≥ 3 → to corridor` pulling the hero
/// into a corridor the chores walked it back out of, after the guard had given up on all three
/// — the guard fired again and again, a stall (rater X on 238bd67, seeds 11 and 28).
fn threats(run: &Run, v: &View) -> Vec<usize> {
    let hp = run.hero.pos;
    v.foes.iter().copied().filter(|&i| run.monsters[i].pos.adjacent(hp) || !run.given_up(run.monsters[i].id)).collect()
}

fn min_foe_dist(run: &Run, v: &View, p: Pos) -> i32 {
    v.foes.iter().map(|&i| run.monsters[i].pos.cheb(p)).min().unwrap_or(99)
}

/// Move to maximise distance from foes, preferring corridors and stairs.
fn verb_retreat(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if threats(run, v).is_empty() {
        return false;
    }
    let hp = run.hero.pos;
    let cur = min_foe_dist(run, v, hp);
    let map = &run.floor.map;
    let mut best: Option<(i32, Pos)> = None;
    for d in DIRS8 {
        let q = hp.step(d);
        if !map.can_step(hp, q) || run.occupied(q) || run.overlays.iter().any(|o| o.x == q.x && o.y == q.y) {
            continue;
        }
        let md = min_foe_dist(run, v, q);
        if md < cur {
            continue;
        }
        let sum: i32 = v.foes.iter().map(|&i| run.monsters[i].pos.cheb(q)).sum();
        // Cut 7: room to keep going counts (a retreat into a dead end is a corner).
        let open = |p: Pos| DIRS8.iter().filter(|d| map.can_step(p, p.step(**d)) && !run.occupied(p.step(**d))).count() as i32;
        let score = md * 8 + sum + map.is_corridor(q) as i32 * 3 + matches!(map.get(q), Tile::StairsDown | Tile::StairsUp) as i32 * 2 + open(q);
        let base = cur * 8 + v.foes.iter().map(|&i| run.monsters[i].pos.cheb(hp)).sum::<i32>() + open(hp);
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

/// Fall back to a corridor within reach (≤ 5 steps); farther is a chase, not a retreat.
fn verb_back_corridor(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let hp = run.hero.pos;
    if run.floor.map.is_corridor(hp) {
        return false;
    }
    let foes: Vec<Pos> = threats(run, v).iter().map(|&i| run.monsters[i].pos).collect();
    if foes.is_empty() && !v.foes.is_empty() {
        return false;
    }
    // (the flood as far as 5 steps: every tile within reach, and its path, is the full flood's)
    let (dist, parent) = run.floor.map.bfs_parent_layers(run.hero.pos, true, &hero_avoids(run, true), |d, _| d > 5);
    let map = &run.floor.map;
    let mut best: Option<(i32, Pos)> = None;
    for (i, d) in dist.iter().enumerate() {
        if *d <= 0 || *d > 5 {
            continue;
        }
        let p = map.pos(i);
        if !map.is_corridor(p) {
            continue;
        }
        let crowded = foes.iter().filter(|f| f.adjacent(p)).count() as i32;
        let score = *d + crowded * 2;
        if best.is_none_or(|(bs, _)| score < bs) {
            best = Some((score, p));
        }
    }
    match best {
        Some((_, goal)) => step_towards(run, cx, goal, &parent),
        None => false,
    }
}

fn find_consumable(run: &Run, cx: &Ctx, cat: Cat, a: &str) -> Option<usize> {
    find_consumable_for(run, cx, cat, a, false)
}

/// `to_throw`: an unknown potion for throwing is picked malevolent-hint first (Cut 2: a hero
/// that has read "counter: range" throws what it has at the boss).
fn find_consumable_for(run: &Run, cx: &Ctx, cat: Cat, a: &str, to_throw: bool) -> Option<usize> {
    if a == "unknown" || a.is_empty() {
        let mut best: Option<(i32, usize)> = None;
        for (i, it) in run.hero.inv.iter().enumerate() {
            if it.cat() != cat || it.is_known(cx.facts, cx.flavours) {
                continue;
            }
            let rank = match it.hint {
                Some(Hint::Benevolent) => 0,
                None => 1,
                Some(Hint::Malevolent) => 2,
            };
            let rank = if to_throw { 2 - rank } else { rank };
            if best.is_none_or(|(r, _)| rank < r) {
                best = Some((rank, i));
            }
        }
        best.map(|(_, i)| i)
    } else {
        // Cut 6 §2: named by the row — an item bought, crafted or vaulted by that name is
        // usable before its flavour is identified; a found one needs the flavour fact.
        run.hero.inv.iter().position(|it| it.cat() == cat && it.kind == a && it.is_known(cx.facts, cx.flavours))
    }
}

fn identify_used(run: &mut Run, cx: &mut Ctx, item: &Item) -> bool {
    let was_unknown = !item.is_known(cx.facts, cx.flavours);
    if let Some(f) = ident_fact(cx.flavours, &item.kind) {
        learn(run, cx, f);
    }
    if item.kind == "recall" {
        // Cut 3: knowing the recall scroll gates `recall_sense`.
        learn(run, cx, "item:recall".into());
    }
    if was_unknown {
        let mal = !item.def().benevolent;
        run.gambles.push((run.turn, item.kind.clone(), mal));
        run.gamble_harm = 0;
        let (_, _, label) = crate::item::describe(item, cx.facts, cx.flavours);
        note(run, cx, format!("Gambled: {label}{}", if label.ends_with('?') { "" } else { "." }));
        // Cut 5 §3: a word before the unknown goes down.
        crate::sifter::voice(run, cx, crate::sifter::Moment::UnknownDrink);
    }
    // Cut 5 §1: the episode remembers what was used.
    run.arc.items_used.push(item.kind.clone());
    was_unknown
}

fn verb_drink(run: &mut Run, cx: &mut Ctx, a: &str) -> bool {
    let Some(ii) = find_consumable(run, cx, Cat::Potion, a) else { return false };
    let kind = run.hero.inv[ii].kind.clone();
    // Sanity: no drinking a known heal at full HP.
    if kind == "heal" && run.hero.inv[ii].is_known(cx.facts, cx.flavours) && run.hero.hp >= run.hero.max_hp {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    run.note_gone(item.id, &item.kind, "used", 1);
    run.drinks += 1;
    let hp = run.hero.hp;
    crate::provenance::used(run, cx, "drunk", &kind, hp);
    identify_used(run, cx, &item);
    // Forge tier: +25% effect per tier (Addendum D).
    let boost = 100 + 25 * item.enchant.max(0);
    let outcome = match kind.as_str() {
        "heal" => {
            // Cut 30: `thin` — a heal potion heals ⅔.
            let add = run.hero.max_hp / 2 * boost / 100 * crate::traits::heal_pct(run) / 100;
            let add = if crate::legacy::has(&run.hero,crate::legacy::MENDING) {add*125/100}else{add};
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
            run.hero.speed_t = 30 * boost / 100;
            "fast".into()
        }
        "invisibility" => {
            run.hero.invis_t = 50 * boost / 100;
            for m in run.monsters.iter_mut() {
                m.last_seen = None;
            }
            "unseen".into()
        }
        "poison" => {
            run.hero.poison = (2, 40);
            "poisoned".into()
        }
        "caustic" => {
            let p = run.hero.pos;
            place_overlay(run, cx, p, 1, OverlayKind::Gas, 30);
            "gas".into()
        }
        "confusion" => {
            run.hero.confused = 30;
            "confused".into()
        }
        "fire" => {
            let p = run.hero.pos;
            place_overlay(run, cx, p, 1, OverlayKind::Fire, 20);
            "fire".into()
        }
        // Cut 3
        "regen" => {
            run.hero.regen_t = 60 * boost / 100;
            "regen".into()
        }
        "resist_fire" => {
            run.hero.resist_fire_t = 60 * boost / 100;
            "fireproof".into()
        }
        "clarity" => {
            run.hero.clarity_t = 60 * boost / 100;
            run.hero.confused = 0;
            "clear".into()
        }
        _ => "nothing".into(),
    };
    // Cut 30: `iron gut` — a malevolent drink harms half.
    crate::traits::after_drink(run, cx, &kind);
    // Cut 13 §3: a use to no effect (a heal at full HP, a kind with nothing to do) is not
    // rebought by the restock.
    if outcome == "nothing" || (kind == "heal" && hp >= run.hero.max_hp) {
        run.wasted_kinds.push(kind.clone());
    }
    cx.events.push(Ev::Use { t: run.turn, item: format!("{kind} potion"), outcome });
    true
}

fn scroll_useless(run: &Run, cx: &Ctx, kind: &str, v: &View) -> bool {
    match kind {
        "mapping" => run.floor.map.seen_pct() >= 95,
        "identify" => !run.hero.inv.iter().any(|i| i.is_consumable() && !i.is_known(cx.facts, cx.flavours)),
        "enchant" => run.hero.weapon.is_none() && run.hero.armour.is_none(),
        "fear" | "darkness" | "teleport" | "blink" => v.foes.is_empty(),
        "summon_ally" => run.allies().next().is_some(),
        "aggravate" => true,
        // Cut 3
        "silence" => run.hero.silence_t > 0,
        "mirror" => run.hero.mirror_charge > 0,
        "earthquake" => {
            let hp = run.hero.pos;
            let map = &run.floor.map;
            !(-2..=2).any(|dy| (-2..=2).any(|dx| quakeable(map, hp.step((dx, dy)))))
        }
        _ => false,
    }
}

/// Cut 3: a wall the earthquake may open (never the map's rim).
fn quakeable(map: &crate::tiles::Map, p: Pos) -> bool {
    map.in_bounds(p) && map.get(p) == Tile::Wall && p.x > 0 && p.y > 0 && p.x < map.w - 1 && p.y < map.h - 1
}

fn verb_read(run: &mut Run, cx: &mut Ctx, a: &str, v: &View) -> bool {
    let Some(ii) = find_consumable(run, cx, Cat::Scroll, a) else { return false };
    let kind = run.hero.inv[ii].kind.clone();
    if is_identified(cx.facts, cx.flavours, &kind) && scroll_useless(run, cx, &kind, v) {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    run.note_gone(item.id, &item.kind, "used", 1);
    let hp = run.hero.hp;
    crate::provenance::used(run, cx, "read", &kind, hp);
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
                    let q = hp.step((dx, dy));
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
                run.monsters[i].fear = 50;
            }
            "foes flee".into()
        }
        "mapping" => {
            run.floor.map.reveal_all();
            "mapped".into()
        }
        "identify" => {
            let target = run.hero.inv.iter().position(|i| i.is_consumable() && !i.is_known(cx.facts, cx.flavours));
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
                w.enchanted += 1;
                "weapon +1".into()
            } else if let Some(ar) = run.hero.armour.as_mut() {
                ar.enchant += 1;
                ar.enchanted += 1;
                "armour +1".into()
            } else {
                "nothing".into()
            }
        }
        "darkness" => {
            for &i in &v.foes {
                run.monsters[i].blind = 40;
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
                m.ttl = Some(200);
                let e = crate::engine::monster_entity(&m, cx.facts);
                run.monsters.push(m);
                cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
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
        // Cut 3
        "recall" => {
            // Home from anywhere, with everything: the escape of the deep.
            callout(run, cx, "recalled");
            end_run(run, cx, ExitTier::Bank);
            "recalled".into()
        }
        "silence" => {
            // 200 ticks: twenty quiet actions, a boss fight's worth (the contract's 100 left
            // the Lurker Queen calling again before a fighter could close and finish her).
            run.hero.silence_t = 200;
            callout(run, cx, "silence");
            "silent".into()
        }
        "earthquake" => {
            let hp = run.hero.pos;
            let mut opened = 0;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let q = hp.step((dx, dy));
                    if quakeable(&run.floor.map, q) {
                        run.floor.map.set(q, Tile::Floor);
                        opened += 1;
                    }
                }
            }
            run.floor.map.compute_corridors(&[]);
            run.hero_dist_pos = None;
            callout(run, cx, "quake!");
            format!("{opened} walls fell")
        }
        "mirror" => {
            run.hero.mirror_charge = 1;
            callout(run, cx, "mirror");
            "mirrored".into()
        }
        _ => "nothing".into(),
    };
    if outcome == "nothing" {
        run.wasted_kinds.push(kind.clone());
    }
    cx.events.push(Ev::Use { t: run.turn, item: format!("{kind} scroll"), outcome });
    true
}

fn verb_throw(run: &mut Run, cx: &mut Ctx, a: &str, v: &View) -> bool {
    let mut parts = a.split(',');
    let kind = parts.next().unwrap_or("").trim();
    let target_sel = parts.next().unwrap_or("nearest").trim();
    // Cut 3: a bell needs no target — it is thrown away from the foes to lure the hunters.
    if kind == "bell" {
        return throw_bell(run, cx, v);
    }
    let ii = if crate::defs::THROWABLE_MISC.contains(&kind) {
        run.hero.inv.iter().position(|i| i.kind == kind)
    } else {
        find_consumable_for(run, cx, Cat::Potion, kind, true)
    };
    let Some(ii) = ii else { return false };
    let Some(mi) = pick_target(run, target_sel, v) else { return false };
    throw_item_at(run, cx, ii, mi)
}

/// Cut 3: a bell lands up to six tiles away, on the far side from the foes: +3 alert there
/// and a noise the whole floor hears — the blind hunters go to it, not to the hero.
fn throw_bell(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let Some(ii) = run.hero.inv.iter().position(|i| i.kind == "bell") else { return false };
    let hp = run.hero.pos;
    let map = &run.floor.map;
    let mut cands: Vec<Pos> = Vec::new();
    for dy in -THROW_RANGE..=THROW_RANGE {
        for dx in -THROW_RANGE..=THROW_RANGE {
            let q = hp.step((dx, dy));
            if q.cheb(hp) >= 3 && map.in_bounds(q) && map.passable(q) && map.los(hp, q) {
                cands.push(q);
            }
        }
    }
    let Some(land) = cands.iter().copied().max_by_key(|q| (min_foe_dist(run, v, *q), q.cheb(hp), -q.x, -q.y)) else { return false };
    let bell = run.hero.inv.remove(ii);
    run.note_gone(bell.id, &bell.kind, "used", 1);
    let hero_hp = run.hero.hp;
    crate::provenance::used(run, cx, "thrown", "bell", hero_hp);
    projectile(run, cx, HERO_ID, 0, hp, land);
    run.alert = (run.alert + 3).min(8);
    run.hero.silence_t = 0;
    noise(run, cx, land, 40);
    callout(run, cx, "bell!");
    cx.events.push(Ev::Use { t: run.turn, item: "bell".into(), outcome: "rang".into() });
    true
}

/// Throw pack item `ii` at monster `mi` (within range and sight), with the potion's effect.
fn throw_item_at(run: &mut Run, cx: &mut Ctx, ii: usize, mi: usize) -> bool {
    let hp = run.hero.pos;
    let mut land = run.monsters[mi].pos;
    if land.cheb(hp) > THROW_RANGE || !run.floor.map.los(hp, land) {
        return false;
    }
    let item = run.hero.inv.remove(ii);
    run.note_gone(item.id, &item.kind, "used", 1);
    let hero_hp = run.hero.hp;
    crate::provenance::used(run, cx, "thrown", &item.kind, hero_hp);
    if item.cat() == Cat::Potion {
        identify_used(run, cx, &item);
    }
    let pkind = item.kind.clone();
    let boost = 100 + 25 * item.enchant.max(0);
    run.last_hit_verb = Some("throw".into());
    projectile(run, cx, HERO_ID, run.monsters[mi].id, hp, land);
    let mut reflected = false;
    if run.monsters[mi].reflects_ranged() {
        land = hp;
        reflected = true;
        let k = run.monsters[mi].kind.clone();
        learn_tag(run, cx, &k, "reflect");
        callout(run, cx, "reflected!");
    } else if mirror_reflects(run, mi, "throw") {
        land = hp;
        reflected = true;
        mirror_learn(run, cx, mi);
    }
    let victim = if reflected { None } else { Some(mi) };
    // QA on 778fa1b (qaV): a throw whose harm reaches the hero — a blast over his own tile, a
    // poison sent back — is his own harm (`Run.own_throw`; the death's `own fire`, its `row`).
    let self_hit = match pkind.as_str() {
        "fire" | "caustic" => land.cheb(hp) <= 1,
        "poison" => victim.is_none(),
        _ => false,
    };
    if self_hit {
        run.own_throw = Some((run.turn, pkind.clone()));
        run.own_throw_harm = 0;
    }
    let outcome = match pkind.as_str() {
        "poison" => {
            // Stacks: each dose is another 8 over 40 ticks.
            let p = (2 * boost / 100 + i32::from(crate::legacy::has(&run.hero,crate::legacy::VENOM)), 40);
            if let Some(m) = victim {
                let cur = run.monsters[m].poison;
                run.monsters[m].poison = (p.0.max(cur.0), cur.1 + p.1);
            } else {
                let cur = run.hero.poison;
                run.hero.poison = (p.0.max(cur.0), cur.1 + p.1);
            }
            "poisoned".into()
        }
        "caustic" => {
            place_overlay(run, cx, land, 1, OverlayKind::Gas, 40 * boost / 100);
            "gas".into()
        }
        "confusion" => {
            if let Some(m) = victim {
                run.monsters[m].confused = 30;
            } else {
                run.hero.confused = 30;
            }
            "confused".into()
        }
        "fire" => {
            place_overlay(run, cx, land, 1, OverlayKind::Fire, 20 * boost / 100);
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
        // Cut 3
        "clarity" => {
            // Clears confusion in the 3×3 around the landing tile (the hero's own included).
            if run.hero.pos.cheb(land) <= 1 {
                run.hero.confused = 0;
            }
            for m in run.monsters.iter_mut() {
                if m.pos.cheb(land) <= 1 {
                    m.confused = 0;
                }
            }
            "cleared".into()
        }
        "salt" => {
            let mut routed = 0;
            for m in run.monsters.iter_mut() {
                if m.hostile() && m.has_tag("undead") && m.pos.cheb(land) <= 1 {
                    m.fear = 50;
                    routed += 1;
                }
            }
            if routed > 0 {
                callout(run, cx, "undead flee");
            }
            format!("{routed} fled")
        }
        _ => "wasted".into(),
    };
    let label = if item.cat() == Cat::Potion { format!("{pkind} potion") } else { pkind.clone() };
    cx.events.push(Ev::Use { t: run.turn, item: label, outcome });
    true
}

/// A projectile event along the line from `from` to `to` (the hit lands in the same tick).
pub fn projectile(run: &Run, cx: &mut Ctx, src: u32, dst: u32, from: Pos, to: Pos) {
    let path: Vec<[i32; 2]> = crate::geom::line(from, to).into_iter().map(|p| [p.x, p.y]).collect();
    cx.events.push(Ev::Projectile { t: run.turn, src, dst, path });
}

fn verb_free_captive(run: &mut Run, cx: &mut Ctx) -> bool {
    let map = &run.floor.map;
    let hp = run.hero.pos;
    // Cut 27 §4: the captive already set out for first, though a corner hides it now (`Run.freeing`).
    // (and the chained captive in view before any other neutral: the row's `on see: captive` saw it —
    // walking to another one lost it from view, and the chore walked back)
    let Some(ci) = run
        .monsters
        .iter()
        .position(|m| m.hp > 0 && m.neutral && run.freeing == Some(m.id))
        .or_else(|| run.monsters.iter().position(|m| m.hp > 0 && m.neutral && m.situation.as_deref() == Some("captive") && map.is_visible(m.pos)))
        .or_else(|| run.monsters.iter().position(|m| m.hp > 0 && m.neutral && map.is_visible(m.pos)))
    else {
        return false;
    };
    let cp = run.monsters[ci].pos;
    if cp.adjacent(hp) {
        let m = &mut run.monsters[ci];
        m.neutral = false;
        m.ally = true;
        m.awake = true;
        let id = m.id;
        let chained = m.situation.take().is_some();
        cx.events.push(Ev::Ally { t: run.turn, id, state: "freed".into() });
        run.ally_freed.push(run.turn);
        note(run, cx, "Freed the captive. It followed.".into());
        callout(run, cx, "freed");
        if chained {
            crate::situations::pass(run, cx, "captive");
            crate::sifter::open_situation(run, crate::sifter::Setup::Captive, "captive", "captive", "");
        }
        return true;
    }
    let (goal, parent) = nearest_goal(run, &hero_avoids(run, true), &cp.neighbours8());
    match goal {
        Some(g) => {
            let id = run.monsters[ci].id;
            let moved = step_towards(run, cx, g, &parent);
            if moved && run.monsters[ci].situation.as_deref() == Some("captive") {
                run.freeing = Some(id);
            }
            moved
        }
        None => false,
    }
}

/// Curious trait: use an unknown item when safe (never a malevolent-hinted one).
pub fn curious_use(run: &mut Run, cx: &mut Ctx) -> Option<Verb> {
    let pick = run
        .hero
        .inv
        .iter()
        .position(|i| i.is_consumable() && !i.is_known(cx.facts, cx.flavours) && i.hint != Some(Hint::Malevolent))?;
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
/// Ticks inside which kiting the same archer again means it is not coming (`kite_archers`).
const KITE_WINDOW: u32 = 40;
/// Actions the `pack break` card keeps going for a pack it stepped out to meet.
const PACK_GO: u32 = 12;
/// Cut 27 §4: the distance inside which the `pack break` card falls back to a corridor.
const PACK_NEAR: i32 = 6;

fn break_los_step(run: &mut Run, cx: &mut Ctx, from: Pos) -> bool {
    let hp = run.hero.pos;
    let map = &run.floor.map;
    let q = DIRS8.iter().map(|d| hp.step(*d)).find(|q| map.can_step(hp, *q) && !run.occupied(*q) && !map.los(from, *q));
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
        "gunslinger" => run.hero.class==Class::Gunner&&run.hero.level>=10&&gunner_tactic(run,cx,v),
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
                let id = run.monsters[i].id;
                // An archer kited once inside `KITE_WINDOW` that is drawing again has held its
                // ground while a chore walked the hero back into its view (cohort 10: the
                // card and the pick-up chore alternated for 12 turns); this time, close.
                let held = run.kited.is_some_and(|(kid, t)| kid == id && run.turn.saturating_sub(t) <= KITE_WINDOW);
                if !held {
                    let from = run.monsters[i].pos;
                    if break_los_step(run, cx, from) {
                        run.kited = Some((id, run.turn));
                        return true;
                    }
                }
            }
            if v.foes.iter().any(|&i| run.monsters[i].has_tag("ranged")) {
                return verb_attack(run, cx, "tag:ranged", v, false);
            }
            false
        }
        "phalanx" => {
            if foes >= 2 && !in_corr && verb_back_corridor(run, cx, v) {
                return true;
            }
            if foes >= 1 && run.taunt_t == 0 && verb_taunt(run, cx, v) {
                return true;
            }
            v.adj >= 1 && verb_attack(run, cx, "nearest", v, false)
        }
        "hit_and_fade" => {
            if verb_backstab(run, cx, v, if run.hero.vanish_t > 0 { 3 } else { 2 }) {
                return true;
            }
            if run.hero.vanish_cd == 0 && v.adj >= 1 {
                run.hero.vanish_t = 30;
                run.hero.vanish_cd = 120;
                callout(run, cx, "vanish");
                return true;
            }
            v.adj >= 1 && verb_attack(run, cx, "nearest", v, false)
        }
        "stair_dance" => {
            let on_stairs = run.hero.pos == run.floor.stairs_down && !stairs_sealed(run);
            if on_stairs && foes >= 1 && run.hero.hp_pct() < 50 {
                descend(run, cx);
                return true;
            }
            if foes >= 2 && !on_stairs && run.floor.map.is_seen(run.floor.stairs_down) && descend_step(run, cx) {
                return true;
            }
            v.adj >= 1 && verb_attack(run, cx, "nearest", v, false)
        }
        // Cut 2 §3. gas_step: never stand in gas and never pop a bloat beside you — step
        // out of hazards, kill gas foes at range when possible, otherwise back off a step
        // before the melee, then fight.
        "gas_step" => {
            if escape_hazard(run, cx, v) {
                return true;
            }
            let hp = run.hero.pos;
            let gas_adj = v.foes.iter().copied().find(|&i| run.monsters[i].has_tag("gas") && run.monsters[i].pos.adjacent(hp));
            if let Some(i) = gas_adj {
                if run.hero.ranged() {
                    return hero_attack(run, cx, i, "shoot", false);
                }
                let n_gas = v.foes.iter().filter(|&&j| run.monsters[j].has_tag("gas")).count();
                if n_gas >= 1 && run.hero.hp_pct() < 60 && verb_retreat(run, cx, v) {
                    return true;
                }
                return verb_attack(run, cx, "tag:gas", v, false);
            }
            if v.foes.iter().any(|&i| run.monsters[i].has_tag("gas")) {
                if run.hero.ranged() {
                    return verb_attack(run, cx, "tag:gas", v, false);
                }
                if let Some(other) = v.foes.iter().copied().find(|&i| !run.monsters[i].has_tag("gas") && run.monsters[i].pos.adjacent(hp)) {
                    return hero_attack(run, cx, other, "attack", false);
                }
                // wait for the bloat to drift in; it dies to the next swing on open ground — Cut 27 §4: a
                // bloat the chase gave up on (not in `engage`: no path, across water) never drifts in, and the
                // card stood before it until the guard called the send a stall (`bloat, no path` · AS's set)
                return v.foes.iter().any(|&i| run.monsters[i].has_tag("gas") && v.engage.contains(&i));
            }
            false
        }
        // pack_break: split a fast pack — fall back to a corridor, then kill the weakest
        // one adjacent so the pack loses its nerve, never chase the ones hanging back.
        "pack_break" => {
            // The pack is the foes the guard has not given up on (`threats()`): a pack it paced in
            // front of and gave up on for the floor held the card in its corridor, and the guard
            // fired again on the same jackals (QA on 23ed91f, qaL run 5).
            // Cut 27 §4 (AS: `R8 pack ↔ pick up`, 12 turns standing among four foes on D5): nor a foe
            // the chase gave up on for now (not in `engage`: unreachable, not closing) — the card
            // backed into its corridor from it, the `pick up` chore stepped out toward an item,
            // and back, until the guard called the send a stall. A foe at the elbow still counts.
            let hp = run.hero.pos;
            let pack: Vec<usize> = threats(run, v).into_iter().filter(|&i| v.engage.contains(&i) || run.monsters[i].pos.adjacent(hp)).collect();
            let foes = pack.len() as i32;
            let going = run.pack_go > run.actions;
            // Cut 27 §4: and only from a pack within `PACK_NEAR` — two goblins idling nine tiles off
            // were out of sight from the corridor's mouth, so the `pick up` chore stepped out, saw
            // them, and the card stepped back in (`R8 pack ↔ pick up`); a pack that far is held below.
            let near = pack.iter().map(|&i| run.monsters[i].pos.cheb(hp)).min().is_some_and(|d| d <= PACK_NEAR);
            // (and once from a tile: a chore that walked the hero back out to it undoes the fall-back —
            // `R8 pack ↔ pick up` before two archers — so from there the card holds or goes)
            if foes >= 2 && near && !in_corr && !going && run.card_fell != Some(hp) && verb_back_corridor(run, cx, v) {
                run.card_fell = Some(hp);
                return true;
            }
            if v.adj >= 1 {
                return verb_attack(run, cx, "lowest", v, false);
            }
            // hold: a pack that hangs back is not worth stepping out for — unless it hangs back
            // to shoot or conjure, and then it never comes (QA on 23ed91f, qaL: a conjurer and
            // an archer held at range on D6; the guard called 6.5 % of that set's sends stalls).
            let shoots = pack.iter().any(|&i| ["ranged", "caster", "summoner"].iter().any(|t| run.monsters[i].has_tag(t)));
            if foes >= 2 && !shoots && !going {
                // The hold lasts while the pack closes: its nearest member three actions at the
                // same distance is a pack that will not come — held forever it was the guard's
                // pacing (qaL run 5: a jackal and a goblin at 4–7 tiles). The corridor hold's own
                // clock (`hold_dist`), so the approach below goes on from it.
                let d = run.monsters[pack[0]].pos.cheb(run.hero.pos);
                if d == run.hold_dist {
                    run.hold_streak += 1;
                } else {
                    run.hold_dist = d;
                    run.hold_streak = 0;
                }
                if run.hold_streak < 3 {
                    return true;
                }
            }
            if foes >= 1 && verb_attack(run, cx, "nearest", v, false) {
                // Going for it: the next action does not fall back to the corridor the step left
                // (qaL run 5: `attack nearest` one step out, `back to corridor` one step in).
                if v.adj == 0 && !going {
                    run.pack_go = run.actions + PACK_GO;
                }
                return true;
            }
            false
        }
        // thief_guard: a thief in view is killed first while it is adjacent; one that flees
        // with the loot is shot or pelted; nothing else is chased. Cut 12 §2: a sleeping den
        // in view is raided first (`on see den → attack nearest`: a free blow, and the
        // thieves bolt empty-handed) — the card answers the den, not only the thief.
        "thief_guard" => {
            let hp = run.hero.pos;
            if v.engage.is_empty() && crate::situations::sees(run, "den") && run.monsters.iter().any(|m| m.hp > 0 && m.dormant && m.situation.as_deref() == Some("den")) {
                run.raiding = true;
                if verb_attack(run, cx, "nearest", v, false) {
                    return true;
                }
            }
            let thief = v.foes.iter().copied().find(|&i| run.monsters[i].has_tag("thief"));
            let Some(i) = thief else { return false };
            if run.monsters[i].pos.adjacent(hp) {
                return hero_attack(run, cx, i, "attack", false);
            }
            let mp = run.monsters[i].pos;
            if run.monsters[i].stolen.is_some() && mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp) {
                if run.hero.ranged() {
                    return hero_attack(run, cx, i, "shoot", false);
                }
                for k in ["fire", "poison", "caustic", "confusion"] {
                    if verb_throw(run, cx, &format!("{k},tag:thief"), v) {
                        return true;
                    }
                }
            }
            // Keep the pack away from a thief that is actually coming (≤ 3 tiles): back into a
            // corridor if one is at hand. A thief loitering at range is not a reason to shuffle
            // (rater F: `to corridor` ↔ `pick up` for minutes at frozen HP).
            // Cut 27 §4: nor from one the chase gave up on for now (not in `engage`) — the card
            // backed in, the chores (`descend`, `pick up`) stepped out, and back (`R3 thief ↔ descend`).
            if !in_corr && mp.cheb(hp) <= 3 && v.engage.contains(&i) && run.card_fell != Some(hp) && verb_back_corridor(run, cx, v) {
                run.card_fell = Some(hp);
                return true;
            }
            false
        }
        // boss_focus: the boss's own counter — the Warlord himself (aimed), the Bloat
        // Mother at range, the Lich's summons first — then the boss.
        "boss_focus" => {
            let Some(bi) = v.foes.iter().copied().find(|&i| run.monsters[i].is_boss()) else { return false };
            let kind = run.monsters[bi].kind.clone();
            match kind.as_str() {
                "bloat_mother" => {
                    if run.hero.ranged() {
                        return verb_attack(run, cx, "tag:boss", v, false);
                    }
                    for k in ["fire", "poison", "caustic"] {
                        if verb_throw(run, cx, &format!("{k},tag:boss"), v) {
                            return true;
                        }
                    }
                    verb_attack(run, cx, "tag:boss", v, false)
                }
                "lich" => {
                    if v.foes.iter().any(|&i| run.monsters[i].has_tag("summoned")) {
                        return verb_attack(run, cx, "tag:summoned", v, false);
                    }
                    verb_attack(run, cx, "tag:boss", v, false)
                }
                // Cut 3: the Queen mends while her called lurkers live — the adjacent ones first.
                "lurker_queen" => {
                    let hp = run.hero.pos;
                    if let Some(i) = v.foes.iter().copied().find(|&i| run.monsters[i].summoned && run.monsters[i].kind == "lurker" && run.monsters[i].pos.adjacent(hp)) {
                        return hero_attack(run, cx, i, "attack", false);
                    }
                    verb_attack(run, cx, "tag:boss", v, false)
                }
                _ => verb_attack(run, cx, "tag:boss", v, false),
            }
        }
        // last_stand: below 30% with foes adjacent — every heal, the second wind, then the
        // weakest foe, throwing whatever is left; no retreat.
        "last_stand" => {
            if v.adj == 0 || run.hero.hp_pct() >= 30 {
                return false;
            }
            if verb_drink(run, cx, "heal") {
                return true;
            }
            if class_has_verb(run.hero.class, run.hero.level, "second_wind") && !run.hero.second_wind_used {
                run.hero.second_wind_used = true;
                let add = run.hero.max_hp * 3 / 10;
                run.hero.hp = (run.hero.hp + add).min(run.hero.max_hp);
                callout(run, cx, "second wind");
                return true;
            }
            if verb_drink(run, cx, "unknown") {
                return true;
            }
            for k in ["fire", "poison", "confusion"] {
                if verb_throw(run, cx, &format!("{k},nearest"), v) {
                    return true;
                }
            }
            verb_attack(run, cx, "lowest", v, false)
        }
        // Cut 3 tier 2. cadence: two of a verb, then another — the Mirror King reflects the
        // third of a kind. The off-beat still hurts when it can (bash, cleave, a throw, the
        // class's specials), else it is a feint.
        "cadence" => {
            if v.foes.is_empty() {
                return false;
            }
            // Preserve an authored cadence rule across a class switch. Guns
            // alternate actual shot verbs and reload instead of borrowing bow
            // skills whose old helpers assume unlimited ammunition.
            if run.hero.class==Class::Gunner {
                if let Some(p)=crate::firearm::Profile::of(run.hero.weapon_kind()) {
                    if run.gun_reload.is_some() {return true;}
                    let loaded=run.hero.weapon.as_ref().and_then(|w|w.firearm).map_or(0,|c|c.loaded);
                    if loaded==0 {return crate::firearm::reload_fast(run,cx)||crate::firearm::reload(run,cx);}
                    let sel=if v.foes.iter().any(|&i|run.monsters[i].is_boss()) {"tag:boss"}else{"nearest"};
                    if run.verb_ring.last().is_some_and(|s|s=="fire")&&run.hero.level>=3 {
                        if p.capacity==1 {
                            if run.gun_skills.as_ref().and_then(|s|s.aim).is_none() {return aim_gun(run,cx,sel,v);}
                        }else if loaded<2 {return crate::firearm::reload_fast(run,cx)||crate::firearm::reload(run,cx);}
                        else {return fire_gun(run,cx,sel,v,true);}
                    }
                    return fire_gun(run,cx,sel,v,false);
                }
            }
            let n = run.verb_ring.len();
            let tight = v.foes.iter().any(|&i| run.monsters[i].kind == "mirror_king" && run.monsters[i].modifiers.is_some_and(|mods| mods.tight_mirror));
            let repeat = if tight { n >= 1 } else { n >= 2 && run.verb_ring[n - 1] == run.verb_ring[n - 2] };
            let last = run.verb_ring.last().cloned().unwrap_or_default();
            let basic = match run.hero.class {
                Class::Caster => "bolt",
                _ if run.hero.ranged() => "shoot",
                _ => "attack",
            };
            if !(repeat && last == basic) {
                // Cut 29 (the dayplayer's `always → cadence`, bought for the Mirror King, struck the
                // Warlord's shield-goblins until he drove the hero off, every send, for nine days):
                // the plain blow goes to a boss in view first, as the boss's own counter would.
                let target = if v.foes.iter().any(|&i| run.monsters[i].is_boss()) { "tag:boss" } else { "nearest" };
                let hit = match basic {
                    "bolt" => verb_bolt(run, cx, target, v),
                    _ => verb_attack(run, cx, target, v, false),
                };
                return hit || (target != "nearest" && match basic {
                    "bolt" => verb_bolt(run, cx, "nearest", v),
                    _ => verb_attack(run, cx, "nearest", v, false),
                });
            }
            let hp = run.hero.pos;
            if last != "shield_bash" && class_has_verb(run.hero.class, run.hero.level, "shield_bash") && run.hero.bash_cd == 0 && v.adj >= 1 && verb_attack(run, cx, "nearest", v, true) {
                return true;
            }
            if last != "cleave" && class_has_verb(run.hero.class, run.hero.level, "cleave") && verb_cleave(run, cx, v) {
                return true;
            }
            if last != "throw" {
                for k in ["fire", "poison", "caustic", "confusion"] {
                    if verb_throw(run, cx, &format!("{k},nearest"), v) {
                        return true;
                    }
                }
            }
            if last != "shoot" && (verb_volley(run, cx, v) || verb_double_shot(run, cx, "nearest", v)) {
                return true;
            }
            if last != "drain" && class_has_verb(run.hero.class, run.hero.level, "drain") && verb_drain(run, cx, v) {
                return true;
            }
            if basic != "attack" && last != "attack" && v.foes.iter().any(|&i| run.monsters[i].pos.adjacent(hp)) {
                let i = v.foes.iter().copied().find(|&i| run.monsters[i].pos.adjacent(hp)).unwrap();
                return hero_attack(run, cx, i, "attack", false);
            }
            callout(run, cx, "feint");
            true
        }
        // noise_discipline: rest only while no blind hunter seen on this floor still lives;
        // otherwise keep moving (the stairs if known, else the frontier) instead of resting.
        "noise_discipline" => {
            if run.blind_foe_known() {
                if v.foes.is_empty() && run.hero.hp < run.hero.max_hp {
                    return descend_step(run, cx) || explore_step(run, cx);
                }
                return false;
            }
            verb_rest(run, cx, v)
        }
        // reflect_read: never melee a foe that reflects it — shoot or throw at it, fight the
        // others (a warden on its arrow face is one of them), back off from one adjacent,
        // and walk away from one that is not.
        "reflect_read" => {
            let hp = run.hero.pos;
            let refl: Vec<usize> = v.foes.iter().copied().filter(|&i| run.monsters[i].reflects_melee()).collect();
            if refl.is_empty() {
                return false;
            }
            let shootable = |run: &Run, i: usize| {
                let mp = run.monsters[i].pos;
                !run.monsters[i].reflects_ranged() && mp.cheb(hp) <= BOW_RANGE && run.floor.map.los(hp, mp)
            };
            if run.hero.ranged() {
                if let Some(i) = refl.iter().copied().find(|&i| shootable(run, i)) {
                    return hero_attack(run, cx, i, "shoot", false);
                }
            } else if let Some(bi) = run.hero.inv.iter().position(|i| i.def().ranged) {
                // A bow in the pack goes up (an action); it comes back down by itself once no
                // mirror is in view.
                if refl.iter().any(|&i| shootable(run, i)) {
                    let bow = run.hero.inv.remove(bi);
                    crate::provenance::spent(run, cx, &bow.kind, "bow up, in hand".into());
                    run.bow_swap = run.hero.weapon.replace(bow);
                    callout(run, cx, "bow up");
                    return true;
                }
            }
            // Throwables go to a boss, or to a mirror that has the hero cornered; fire and gas
            // only from two tiles off (the blast is 3×3).
            let cornered = refl.iter().any(|&i| run.monsters[i].pos.adjacent(hp)) && !can_step_clear_of(run, near_of(run, &refl, hp));
            for k in ["poison", "confusion", "fire", "caustic"] {
                let Some(ii) = find_consumable_for(run, cx, Cat::Potion, k, true) else { continue };
                let blast = matches!(k, "fire" | "caustic");
                let target = refl
                    .iter()
                    .copied()
                    .filter(|&i| !run.monsters[i].reflects_ranged() && (!blast || run.monsters[i].pos.cheb(hp) >= 2 || run.hero.resist_fire_t > 0))
                    .find(|&i| run.monsters[i].is_boss() || cornered);
                if let Some(i) = target {
                    if throw_item_at(run, cx, ii, i) {
                        return true;
                    }
                }
            }
            // A boss with the hero in its reach and fire in the pack: step back first, then throw.
            if refl.iter().any(|&i| run.monsters[i].is_boss() && run.monsters[i].pos.adjacent(hp))
                && ["fire", "caustic"].iter().any(|k| find_consumable_for(run, cx, Cat::Potion, k, true).is_some())
                && step_clear_of(run, cx, near_of(run, &refl, hp))
            {
                return true;
            }
            if let Some(other) = v.foes.iter().copied().find(|&i| !run.monsters[i].reflects_melee() && run.monsters[i].pos.adjacent(hp)) {
                return hero_attack(run, cx, other, "attack", false);
            }
            // Nothing to throw: the mirror is not worth a swing. It is terrain now — the chores'
            // goals (the stairs, the frontier) are walked around it, never within a tile of it;
            // with no way around, gain ground on it (it is slow and forgets what it cannot
            // see). Items wait: they lure the hero into corners.
            run.items_until = run.items_until.max(run.actions + 10);
            let near = near_of(run, &refl, hp);
            if route_around(run, cx, &refl) {
                return true;
            }
            if step_clear_of(run, cx, near) {
                return true;
            }
            // Boxed in: hold. It comes on slowly and forgets what it cannot see; the
            // oscillation guard opens the chores again if this drags on.
            true
        }
        // deep_march: in the dark (vision ≤ 4) the way down is taken at 40% seen.
        "deep_march" => {
            if run.vision(cx.unlocks) > 4 || run.floor.map.seen_pct() < 40 || stairs_sealed(run) || !v.foes.is_empty() {
                return false;
            }
            descend_step(run, cx)
        }
        // Mastery cards (class L10). hawkeye: the ranger's whole ladder in one row.
        "hawkeye" => {
            if v.adj >= 1 && verb_kite(run, cx, v) {
                return true;
            }
            if foes >= 2 && verb_volley(run, cx, v) {
                return true;
            }
            if verb_double_shot(run, cx, "nearest", v) {
                return true;
            }
            verb_shoot(run, cx, "nearest", v)
        }
        // archmage: the caster's — ward when pressed, nova when surrounded, bolt otherwise.
        "archmage" => {
            if v.adj >= 2 && verb_nova(run, cx, v) {
                return true;
            }
            if v.adj >= 1 && run.hero.ward_cd == 0 && run.hero.ward_t == 0 {
                run.hero.ward_t = 30;
                run.hero.ward_cd = 100;
                callout(run, cx, "ward");
                return true;
            }
            if v.adj >= 1 && run.hero.hp_pct() < 50 && verb_blink(run, cx, v) {
                return true;
            }
            verb_bolt(run, cx, "nearest", v)
        }
        _ => false,
    }
}

// ---------------------------------------------------------------- monsters

fn move_monster(run: &mut Run, cx: &mut Ctx, mi: usize, q: Pos) {
    let from = run.monsters[mi].pos;
    let was_visible = run.floor.map.is_visible(from);
    run.monsters[mi].pos = q;
    // Cut 3: a slag crawler leaves fire where it crawled (no spread).
    if run.monsters[mi].kind == "slag_crawler" && run.floor.map.get(from) == Tile::Floor && !run.overlays.iter().any(|o| o.x == from.x && o.y == from.y) {
        run.overlays.push(crate::tiles::Overlay { x: from.x, y: from.y, k: OverlayKind::Fire, ttl: 20, spread: false });
        cx.events.push(Ev::Overlay { t: run.turn, x: from.x, y: from.y, ov: OverlayKind::Fire, ttl: 20 });
        if was_visible {
            learn_tag(run, cx, "slag_crawler", "fire");
        }
    }
    if was_visible || run.floor.map.is_visible(q) {
        let id = run.monsters[mi].id;
        cx.events.push(Ev::Move { t: run.turn, id, x: q.x, y: q.y });
    }
    // The ranger's trap (Cut 2 §4): the first hostile onto it is stunned for 20 ticks.
    if run.monsters[mi].hostile() {
        if let Some(ii) = run.item_at(q).filter(|&ii| run.items[ii].item.kind == "trap") {
            run.items.remove(ii);
            run.monsters[mi].stun = 20 + 5*i32::from(crate::legacy::has(&run.hero,crate::legacy::CONTROL));
            run.monsters[mi].pending = None;
            run.monsters[mi].telegraph = None;
            if run.floor.map.is_visible(q) {
                callout(run, cx, "trapped");
            }
        }
    }
}

fn can_see_hero(run: &Run, mi: usize) -> bool {
    let m = &run.monsters[mi];
    let h = &run.hero;
    if m.blind > 0 || m.is_blind() || h.untargetable() {
        return false;
    }
    let d = m.pos.cheb(h.pos);
    if h.invis_t > 0 {
        return d <= 1;
    }
    // Cut 3: the dark cuts everyone's sight (the floor's radius), not only the hero's.
    d <= run.floor.vision && run.floor.map.los(m.pos, h.pos)
}

fn approach(run: &mut Run, cx: &mut Ctx, mi: usize) -> bool {
    // The cached field itself (no copy per monster per tick): nothing below writes it — settled
    // as far as the monster's own tile (`hero_dist_to`: all `step_down` reads).
    let mp = run.monsters[mi].pos;
    crate::turn::hero_dist_to(run, mp);
    let water_only = run.monsters[mi].has_tag("water");
    let map = &run.floor.map;
    let occ = |q: Pos| run.occupied(q) || (water_only && map.get(q) != Tile::Water);
    let step = map.step_down(&run.hero_dist, mp, &occ);
    if let Some(q) = step {
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
        let q = mp.step(d);
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
        .map(|d| mp.step(*d))
        .filter(|q| run.floor.map.can_step(mp, *q) && !run.occupied(*q) && (!water_only || run.floor.map.get(*q) == Tile::Water))
        .collect();
    if !cands.is_empty() {
        let q = *run.rng.pick(&cands);
        move_monster(run, cx, mi, q);
    }
}

/// A boss holds the stairs (it is placed within 3 of them and its life seals them): with no
/// hero to chase it walks back to its post rather than wander off. A Warlord drifting 20 tiles
/// from the stairs left the hero waiting at a sealed wall with nothing in view — `wait` for
/// 5000 ticks, a stall (rater X on 238bd67, seed 9). False when it is at its post.
fn boss_to_post(run: &mut Run, cx: &mut Ctx, mi: usize) -> bool {
    let m = &run.monsters[mi];
    let post = run.floor.stairs_down;
    if !m.is_boss() || !m.hostile() || m.pos.cheb(post) <= 3 {
        return false;
    }
    let map = &run.floor.map;
    let home = map.bfs(post, false, &|_| false);
    if let Some(q) = map.step_down(&home, m.pos, &|q| run.occupied(q)) {
        move_monster(run, cx, mi, q);
        return true;
    }
    false
}

/// `wander`, except that a boss off its post goes back to it.
fn drift(run: &mut Run, cx: &mut Ctx, mi: usize) {
    if !boss_to_post(run, cx, mi) {
        wander(run, cx, mi);
    }
}

fn adjacent_ally(run: &Run, mi: usize) -> Option<usize> {
    if run.taunt_t > 0 {
        return None;
    }
    let mp = run.monsters[mi].pos;
    run.monsters
        .iter()
        .enumerate()
        .filter(|(j, o)| *j != mi && o.hp > 0 && o.ally && o.pos.adjacent(mp))
        // Cut 20 §2: a companion fallen back is hit last (it has left the fight).
        .min_by_key(|(_, o)| (pet_wounded(o), o.hp, o.id))
        .map(|(j, _)| j)
}

/// Something to hit in melee: the hero, else an adjacent ally.
fn engaged(run: &Run, mi: usize) -> bool {
    let mp = run.monsters[mi].pos;
    (mp.adjacent(run.hero.pos) && !run.hero.untargetable()) || adjacent_ally(run, mi).is_some()
}

/// Cut 22 §2: what a thief can take from the hero.
pub(crate) enum Take {
    Inv(usize),
    Coins(i32),
}

/// Cut 22 §2 (AH: "the monkey stole the heal potion on D1 … 8 seconds after I paid $40"): a
/// thief takes what the run found first (its own loot), then a vault-brought item (the run's
/// stake, Cut 7), then a coin pile's worth of the carried gold (`$4 + 2 × depth`: `stolen $6`
/// on D1), and a packed supply only when the pack holds nothing else; `weapon`: last, the
/// weapon in hand (the den's snatch). `random`: among the kind's candidates by the run's rng,
/// else the first. A forge imp (`potions_only`) takes potions only, never coins.
///
/// Cut 23 §5 (AI: "the leash stolen nearly every run"; 0.24 leash thefts a send over the
/// cohort sets): a leash — the only pet gear, the kennel's free one included — is taken only
/// when the pack holds nothing else, after the packed supplies.
pub(crate) fn thief_pick(run: &mut Run, potions_only: bool, random: bool) -> Option<Take> {
    let pet_gear = |run: &Run, i: usize| run.hero.inv[i].kind == "leash";
    // Cut 24 §3 (AL: a thief took the leather +1 just forged): the kit is never a thief's.
    let kit = |run: &Run, i: usize| crate::kit::is_kit_id(run.hero.inv[i].id);
    let found = |run: &Run, i: usize| !pet_gear(run, i) && !kit(run, i) && !run.brought.contains(&run.hero.inv[i].id) && !run.supplies.contains(&run.hero.inv[i].id);
    let brought = |run: &Run, i: usize| !pet_gear(run, i) && !kit(run, i) && run.brought.contains(&run.hero.inv[i].id);
    for class in 0..2 {
        let c: Vec<usize> = (0..run.hero.inv.len())
            .filter(|&i| !potions_only || run.hero.inv[i].cat() == Cat::Potion)
            .filter(|&i| if class == 0 { found(run, i) } else { brought(run, i) })
            .collect();
        if !c.is_empty() {
            let k = if random { run.rng.below(c.len() as u32) as usize } else { 0 };
            return Some(Take::Inv(c[k]));
        }
    }
    if !potions_only && run.loot > 0 {
        return Some(Take::Coins(run.loot.min(4 + 2 * run.depth as i32)));
    }
    for gear in [false, true] {
        let c: Vec<usize> = (0..run.hero.inv.len()).filter(|&i| (!potions_only || run.hero.inv[i].cat() == Cat::Potion) && pet_gear(run, i) == gear && !kit(run, i)).collect();
        if !c.is_empty() {
            let k = if random { run.rng.below(c.len() as u32) as usize } else { 0 };
            return Some(Take::Inv(c[k]));
        }
    }
    // Cut 25 §5 (AN: a monkey took the weapon in hand on D1–D3 in five runs): what the hero wears
    // is his, not loot — the weapon in hand and the armour on him are never a thief's.
    None
}

/// Takes `take` off the hero: the item (coins as a gold pile the thief drops when killed) and
/// what it cost the carried gold (Cut 10 §3: `$26 → $10` read as a bug without it — the gold
/// `loot_add` takes off the run for the item's value; coins are their own worth).
pub(crate) fn thief_take(run: &mut Run, take: Take) -> (Item, Option<i32>) {
    let before = run.loot;
    let it = match take {
        Take::Inv(i) => {
            let it = run.hero.inv.remove(i);
            run.loot_add(-run.loot_value(&it));
            it
        }
        Take::Coins(n) => {
            let mut it = Item::new(run.new_item_id(), "gold");
            it.amount = n;
            run.loot_add_gold(-n);
            it
        }
    };
    if it.kind != "gold" {
        run.note_gone(it.id, &it.kind, "stolen", it.amount.max(1));
    }
    let amount = (before > run.loot).then(|| before - run.loot);
    (it, amount)
}

/// Monster attack on the hero (or, failing adjacency, an ally) with tag riders. `mult` doubles ogre hits.
fn monster_attack(run: &mut Run, cx: &mut Ctx, mi: usize, mult: i32, verb: &str) {
    let mp = run.monsters[mi].pos;
    let hero_in_reach = mp.adjacent(run.hero.pos) && !run.hero.untargetable();
    if !hero_in_reach || (verb == "shoot" && !can_see_hero(run, mi)) {
        if let Some(ai) = adjacent_ally(run, mi) {
            let m = &run.monsters[mi];
            let a = m.effective_atk();
            let atk = (a.0 * mult, a.1 * mult);
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
    let a = m.effective_atk();
    let atk = (a.0 * mult, a.1 * mult);
    // Cut 25 §1: armour blunts a blow, never negates it (`Hero::blunt`).
    let (hit, roll) = roll_hit(&mut run.rng, atk, 0);
    let dmg = if hit { run.hero.blunt(roll) } else { 0 };
    let id = run.monsters[mi].id;
    let kind = run.monsters[mi].kind.clone();
    cx.events.push(Ev::Attack { t: run.turn, src: id, dst: HERO_ID, dmg, hit, verb: Some(verb.into()) });
    if !hit {
        return;
    }
    let cause = kind.clone();
    if verb != "shoot" {
        // Cut 3: blows on the hero are noise too (fights draw the Deep's hunters).
        let at = run.hero.pos;
        noise(run, cx, at, 6);
    }
    // Cut 20 §1: a thief steals once per run — after a theft, a thief's blow finds nothing it
    // dares take and it runs (the carried item is the stake of the first fight, not a tax).
    if run.monsters[mi].has_tag("thief") && dmg > 0 && run.monsters[mi].stolen.is_none() && !run.stolen.is_empty() && !run.monsters[mi].ally {
        run.monsters[mi].fear = run.monsters[mi].fear.max(THIEF_FLEE);
    } else if run.monsters[mi].has_tag("thief") && dmg > 0 && run.monsters[mi].stolen.is_none() {
        // A forge imp only steals potions. Cut 22 §2: what was found first, a coin pile's
        // worth next, a packed supply last (`thief_pick`).
        let potions_only = kind == "forge_imp";
        if let Some(take) = thief_pick(run, potions_only, true) {
            let (it, amount) = thief_take(run, take);
            // Cut 7 §3: a den thief's theft counts against the den.
            if run.monsters[mi].situation.as_deref() == Some("den") {
                run.den_stolen.push(it.id);
            }
            let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
            let by_den = run.monsters[mi].situation.as_deref() == Some("den");
            crate::provenance::stolen(run, cx, &it.kind, &kind, by_den, &label);
            let coins = it.cat() == Cat::Gold;
            run.stolen_ids.push(it.id);
            run.stolen_kinds.push((it.id, it.kind.clone(), amount.unwrap_or(0)));
            run.monsters[mi].stolen = Some(it);
            run.monsters[mi].fleeing = true;
            cx.events.push(Ev::Steal { t: run.turn, id, item: label.clone(), amount });
            run.stolen.push((run.turn, label.clone()));
            run.stolen_labels.push((run.stolen_ids.last().copied().unwrap_or(0), label.clone()));
            // An unknown's label ends in `?`; the note takes no second stop (`black potion?.`).
            if coins {
                note(run, cx, format!("The {} stole ${}.", crate::engine::kind_title(&kind), amount.unwrap_or(0)));
            } else {
                note(run, cx, format!("The {} stole the {label}{}", crate::engine::kind_title(&kind), if label.ends_with('?') { "" } else { "." }));
            }
            match amount {
                Some(g) => callout(run, cx, &format!("stolen ${g}")),
                None => callout(run, cx, "stolen!"),
            }
            learn_tag(run, cx, &kind, "thief");
        }
    }
    if run.monsters[mi].has_tag("paralyse") && dmg > 0 && run.rng.chance(20) {
        run.hero.paralysed = 12; // covers the hero's next action at base speed
        callout(run, cx, "paralysed");
        learn_tag(run, cx, &kind, "paralyse");
    }
    if run.monsters[mi].has_tag("drain") && dmg > 0 && run.hero.max_hp > 5 {
        run.hero.max_hp -= 1;
        run.hero.hp = run.hero.hp.min(run.hero.max_hp);
        // QA on 308f045 (qaAC: `36/36` → `22/22` → `12/24`, "the cause?"): every max-HP loss is an event naming it
        crate::turn::hero_max_hp(run, cx, -1, "drain");
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
    if cands.len() < n {
        // Crowded: the reserves squeeze in two tiles out.
        for dy in -2..=2 {
            for dx in -2..=2 {
                let q = at.step((dx, dy));
                if q.cheb(at) == 2 && run.floor.map.passable(q) && !run.occupied(q) {
                    cands.push(q);
                }
            }
        }
    }
    for _ in 0..n {
        if cands.is_empty() {
            break;
        }
        let q = cands.remove(run.rng.below(cands.len() as u32) as usize);
        let id = run.new_id();
        let depth = run.depth;
        let mut m = crate::endgame::spawn(run, id, kind, q, depth, false);
        m.awake = true;
        m.last_seen = Some(run.hero.pos);
        m.ttl = ttl;
        m.summoned = true;
        if !m.has_tag("summoned") {
            m.extra_tags.push("summoned".into());
        }
        let e = crate::engine::monster_entity(&m, cx.facts);
        let seen = run.floor.map.is_visible(q);
        run.monsters.push(m);
        cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
        if seen {
            // Cut 2 §5: what the hero sees conjured is a fact it can target (`attack tag:summoned`).
            learn_tag(run, cx, kind, "summoned");
        }
        made += 1;
    }
    made
}

pub fn telegraph(run: &mut Run, cx: &mut Ctx, mi: usize, what: &str, pending: Pending) {
    run.monsters[mi].telegraph = Some(what.into());
    run.monsters[mi].pending = Some(pending);
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Telegraph { t: run.turn, id, what: what.into() });
    if run.floor.map.is_visible(run.monsters[mi].pos) {
        let kind = run.monsters[mi].kind.clone();
        // The title's last word, lowercase as every callout is (`warlord rallies`, `bloat swells`).
        let title = run.monsters[mi].def().title.split_whitespace().last().unwrap_or("foe").to_lowercase();
        // Cut 23 §3 (AJ: "`IT SHELLS` I could not explain"): the shout says what comes next on tap.
        crate::chronicle::callout_why(run, cx, &format!("{title} {what}"), Some(pending.why()));
        learn_tag(run, cx, &kind, "telegraph");
        if run.monsters[mi].is_boss() {
            // Every boss telegraphs its mechanic on the first turn; seeing it is the counter fact.
            crate::facts::learn_boss_counter(run, cx, &kind);
        }
    }
}

/// Cut 23 §1: what the Lurker Queen's called lurkers add to a wild lurker's bite (2–5 → 4–7).
pub const BROOD_BITE: i32 = 2;

/// Living summons that belong to a boss (goblins for the Warlord, skeletons for the Lich).
/// Rallies the Warlord has in him (two goblins each): past these he fights alone.
const WARLORD_RESERVES: u32 = 6;

fn boss_summons(run: &Run, kind: &str) -> usize {
    let esc = match kind {
        "goblin_warlord" => "goblin",
        "lich" => "skeleton",
        _ => return 0,
    };
    run.monsters.iter().filter(|m| m.hp > 0 && m.hostile() && m.summoned && m.kind == esc).count()
}

fn resolve_pending(run: &mut Run, cx: &mut Ctx, mi: usize, p: Pending) {
    let kind = run.monsters[mi].kind.clone();
    let mp = run.monsters[mi].pos;
    let visible = run.floor.map.is_visible(mp);
    let hp = run.hero.pos;
    match p {
        Pending::Shoot => {
            if can_see_hero(run, mi) && mp.cheb(hp) <= BOW_RANGE {
                let id = run.monsters[mi].id;
                projectile(run, cx, id, HERO_ID, mp, hp);
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
                approach(run, cx, mi);
            }
        }
        Pending::Rally if run.monsters[mi].broken => {}
        Pending::Rally => {
            // Cut 7 §1: the Captain's one rally brings a single goblin.
            let n = if kind == "goblin_captain" { 1 } else { 2 };
            summon_near(run, cx, mi_pos(run, mi), "goblin", n, None);
            run.monsters[mi].rallies += 1;
            if kind == "goblin_warlord" {
                warlord_buff(run, cx, mi);
            }
            if visible {
                learn_tag(run, cx, &kind, "summoner");
                callout(run, cx, "rallied!");
            }
        }
        Pending::Swell => {
            place_overlay(run, cx, mp, 1, OverlayKind::Gas, 50);
            if visible {
                learn_tag(run, cx, &kind, "gas");
            }
        }
        Pending::Chant => {
            let before = run.monsters.len();
            summon_near(run, cx, mi_pos(run, mi), "skeleton", 2, None);
            for m in run.monsters[before..].iter_mut() {
                // Spectral: brittle, but they keep coming while any stand.
                m.max_hp = 4;
                m.hp = 4;
                m.def = 0;
            }
            if visible {
                learn_tag(run, cx, &kind, "summoner");
                callout(run, cx, "skeletons!");
            }
        }
        // Cut 3. The sentinel's gaze: the hero within two tiles and in sight is stunned.
        Pending::Gaze => {
            if mp.cheb(hp) <= 2 && run.floor.map.los(mp, hp) && !run.hero.untargetable() {
                run.hero.paralysed = 12;
                if visible {
                    callout(run, cx, "stunned");
                    learn_tag(run, cx, &kind, "gaze");
                }
            }
            if engaged(run, mi) {
                monster_attack(run, cx, mi, 1, "attack");
            }
        }
        // The warden turns its other face.
        Pending::Flip => {
            run.monsters[mi].warden_ranged = !run.monsters[mi].warden_ranged;
            if visible {
                callout(run, cx, if run.monsters[mi].warden_ranged { "mirrors arrows" } else { "mirrors blades" });
                learn_tag(run, cx, &kind, "reflect_melee");
                learn_tag(run, cx, &kind, "reflect");
            }
        }
        // The Lurker Queen calls two lurkers to the noise she heard.
        Pending::Call => {
            let before = run.monsters.len();
            summon_near(run, cx, mi_pos(run, mi), "lurker", 2, Some(150));
            let heard = run.noise.map(|(p, _)| p);
            // Cut 23 §1: her called brood bites harder than a wild lurker (`BROOD_BITE`): a kitted
            // hero's armour shrugged the wild ones off and the wall fell without its counter.
            for m in run.monsters[before..].iter_mut() {
                m.last_seen = heard;
                m.atk = (m.atk.0 + BROOD_BITE, m.atk.1 + BROOD_BITE);
            }
            if visible {
                learn_tag(run, cx, &kind, "summoner");
                callout(run, cx, "lurkers!");
            }
        }
        // The Mirror King's warning: the next of a kind comes back. He strikes meanwhile.
        Pending::Mirror => {
            if engaged(run, mi) {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                approach(run, cx, mi);
            }
        }
        // The Foundry Master's hammer: a double blow, like the ogre's.
        Pending::Hammer => {
            if engaged(run, mi) {
                monster_attack(run, cx, mi, 2, "hammer");
            } else {
                approach(run, cx, mi);
            }
        }
    }
    if run.monsters[mi].is_boss() && run.over.is_none() && visible {
        crate::facts::learn_boss_counter(run, cx, &kind);
    }
}

/// Cut 3: +`def` for `ticks` to every hostile in the buffer's view (itself included).
fn buff_allies(run: &mut Run, cx: &mut Ctx, mi: usize, def: i32, ticks: i32, word: &str) {
    let mp = run.monsters[mi].pos;
    let kind = run.monsters[mi].kind.clone();
    let mut any = false;
    for m in run.monsters.iter_mut() {
        if m.hostile() && m.hp > 0 && m.pos.cheb(mp) <= VISION && m.buff_def.1 < ticks {
            m.buff_def = (def.max(m.buff_def.0), ticks);
            any = true;
        }
    }
    if any && run.floor.map.is_visible(mp) {
        learn_tag(run, cx, &kind, "buffer");
        callout(run, cx, word);
    }
}

/// Shield-buff: +2 def for 30 ticks to every goblin in the Warlord's view.
fn warlord_buff(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let mp = run.monsters[mi].pos;
    let mut any = false;
    for m in run.monsters.iter_mut() {
        if m.hostile() && m.kind == "goblin" && m.pos.cheb(mp) <= VISION {
            m.buff_def = (1, 30);
            any = true;
        }
    }
    if any && run.floor.map.is_visible(mp) {
        learn_tag(run, cx, "goblin_warlord", "buffer");
        callout(run, cx, "shields up");
    }
}

/// Cut 16 §4: the Warlord's phase two — speed and damage he gains when he breaks.
pub const WARLORD_BREAK_SPEED: i32 = 2;
pub const WARLORD_BREAK_ATK: (i32, i32) = (1, 1);

/// Cut 16 §4: at half hp the Warlord breaks, once — the rally he was calling is dropped, the
/// shields on his goblins end, and he charges faster and hitting harder, with no more rallies
/// and no wall to step behind (`turn::damage_monster`). The beat is `warlord breaks`.
pub fn warlord_break(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let m = &mut run.monsters[mi];
    if m.broken {
        return;
    }
    m.broken = true;
    m.speed += WARLORD_BREAK_SPEED;
    m.atk = (m.atk.0 + WARLORD_BREAK_ATK.0, m.atk.1 + WARLORD_BREAK_ATK.1);
    if m.pending == Some(Pending::Rally) {
        m.pending = None;
        m.telegraph = None;
    }
    m.awake = true;
    let mp = m.pos;
    for o in run.monsters.iter_mut() {
        if o.hostile() && o.kind == "goblin" && o.buff_def.1 > 0 {
            o.buff_def = (0, 0);
        }
    }
    if run.floor.map.is_visible(mp) {
        callout(run, cx, "warlord breaks");
        note(run, cx, "The Warlord breaks.".into());
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

pub fn monster_act(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let (stun, paralysed, neutral, ally, confused) = {
        let m = &run.monsters[mi];
        (m.stun, m.paralysed, m.neutral, m.ally, m.confused)
    };
    if stun > 0 || paralysed > 0 || neutral {
        return;
    }
    if ally {
        if run.monsters[mi].is_companion() {
            companion_act(run, cx, mi);
        } else {
            ally_act(run, cx, mi);
        }
        return;
    }
    if confused > 0 {
        wander(run, cx, mi);
        return;
    }
    let hp = run.hero.pos;
    // Cut 5 §4: a den's sleepers wake to the hero's step (two tiles), not to sight.
    if run.monsters[mi].dormant {
        return;
    }
    // Cut 7 §3: a lock bloat waits for the hero to come near (`situations::wake`), then swells
    // and bursts on its fuse.
    if run.monsters[mi].situation.as_deref() == Some("lock") && !run.monsters[mi].awake {
        return;
    }
    if crate::situations::lock_bloat_act(run, cx, mi) {
        return;
    }
    let sees = can_see_hero(run, mi);
    if sees {
        run.monsters[mi].awake = true;
        run.monsters[mi].last_seen = Some(hp);
    }
    let (mp, fear, fleeing, awake, pending, cooldown, m_hp, m_max, is_thief) = {
        let m = &run.monsters[mi];
        (m.pos, m.fear, m.fleeing, m.awake, m.pending, m.cooldown, m.hp, m.max_hp, m.has_tag("thief"))
    };
    let adjacent = engaged(run, mi);
    if fear > 0 || fleeing {
        if fleeing && !sees && !run.floor.map.is_visible(mp) {
            return; // hidden with the loot
        }
        if !step_away(run, cx, mi, hp, is_thief) && adjacent && fear == 0 {
            monster_attack(run, cx, mi, 1, "attack");
        }
        return;
    }
    if !awake {
        if run.rng.chance(15) {
            drift(run, cx, mi);
        }
        return;
    }
    if let Some(p) = pending {
        run.monsters[mi].pending = None;
        run.monsters[mi].telegraph = None;
        resolve_pending(run, cx, mi, p);
        return;
    }
    let dist = mp.cheb(hp);
    let kind_owned = run.monsters[mi].kind.clone();
    let kind = kind_owned.as_str();
    match kind {
        "goblin_archer" => {
            if sees && (2..=BOW_RANGE).contains(&dist) {
                telegraph(run, cx, mi, "draws", Pending::Shoot);
            } else if adjacent {
                if !step_away(run, cx, mi, hp, false) {
                    monster_attack(run, cx, mi, 1, "attack");
                }
            } else {
                chase(run, cx, mi, sees);
            }
        }
        "ogre" => {
            if adjacent && mp.adjacent(hp) {
                telegraph(run, cx, mi, "winds up", Pending::HeavyHit);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        "goblin_conjurer" => {
            if sees && cooldown == 0 {
                let made = summon_near(run, cx, hp, "spectral_blade", 2, Some(60));
                run.monsters[mi].cooldown = 150;
                if made > 0 && run.floor.map.is_visible(mp) {
                    callout(run, cx, "blades!");
                    learn_tag(run, cx, "goblin_conjurer", "caster");
                    learn_tag(run, cx, "goblin_conjurer", "summoner");
                }
            } else if adjacent {
                if !(run.rng.chance(30) && step_away(run, cx, mi, hp, false)) {
                    monster_attack(run, cx, mi, 1, "attack");
                }
            } else if dist > 4 || !sees {
                chase(run, cx, mi, sees);
            }
        }
        "jackal" | "ghoul" => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else if packmates_ready(run, mi) {
                chase(run, cx, mi, sees);
            } else if sees && dist < 3 {
                step_away(run, cx, mi, hp, false);
            } else if !sees {
                chase(run, cx, mi, sees);
            }
        }
        "bloat" => {
            if !adjacent {
                chase(run, cx, mi, sees);
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
                approach(run, cx, mi);
            }
        }
        // The Warlord is a wall of goblins: whenever no goblin stands beside him he rallies two
        // more, and every 30 ticks he shield-buffs the goblins in view. Attrition beats
        // attack-nearest; the counter is to go for him (`attack tag:boss`) or stun him.
        // Cut 16 §4: broken, he charges — no rally, no shields.
        "goblin_warlord" if run.monsters[mi].broken => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        "goblin_warlord" => {
            let guards = run.monsters.iter().filter(|o| o.hp > 0 && o.hostile() && o.kind == "goblin" && o.pos.adjacent(mp)).count();
            let in_view = run.monsters.iter().filter(|o| o.hp > 0 && o.hostile() && o.kind == "goblin" && o.pos.cheb(mp) <= VISION).count();
            let hurt = run.monsters[mi].hurt_since_action;
            run.monsters[mi].hurt_since_action = false;
            let reserves = run.monsters[mi].rallies < WARLORD_RESERVES;
            if ((sees && guards < 2) || hurt) && in_view < 4 && reserves {
                if guards == 0 && boss_summons(run, "goblin_warlord") > 0 {
                    // Caught unguarded after the first rally: the reserves are already
                    // there, they step in at once (no window for incidental swings).
                    resolve_pending(run, cx, mi, Pending::Rally);
                } else {
                    telegraph(run, cx, mi, "rallies", Pending::Rally);
                }
            } else if sees && cooldown == 0 {
                warlord_buff(run, cx, mi);
                run.monsters[mi].cooldown = 30;
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else if dist > 3 || guards >= 2 {
                // Guarded, he presses the hero himself: a wall that never bites is a stalemate.
                chase(run, cx, mi, sees);
            }
        }
        // Cut 7 §1: the Captain is the Warlord's lesson in small — one rally (two goblins) on
        // first sight, no shield wall, then he fights like a goblin with a longer reach.
        "goblin_captain" => {
            if sees && !run.monsters[mi].introduced {
                run.monsters[mi].introduced = true;
                telegraph(run, cx, mi, "rallies", Pending::Rally);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // The Bloat Mother vents a 5×5 cloud on every melee hit she takes and heals in gas:
        // melee-only sets choke. The counter is range (bow, thrown potions).
        "bloat_mother" => {
            if sees && m_hp == m_max && cooldown == 0 {
                telegraph(run, cx, mi, "swells", Pending::Swell);
                run.monsters[mi].cooldown = 80;
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // The Lich reflects anything ranged and, while any of its skeletons stand, chants two
        // more every 60 ticks. The counter is to melee the summons down, then the Lich.
        "lich" => {
            let summons = boss_summons(run, "lich");
            if sees && cooldown == 0 {
                telegraph(run, cx, mi, "chants", Pending::Chant);
                run.monsters[mi].cooldown = if summons > 0 { 60 } else { 100 };
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
                if run.over.is_none() && run.hero.max_hp > 5 {
                    run.hero.max_hp -= 1;
                    run.hero.hp = run.hero.hp.min(run.hero.max_hp);
                    crate::turn::hero_max_hp(run, cx, -1, "drain");
                }
            } else if summons == 0 || dist > 4 {
                chase(run, cx, mi, sees);
            }
        }
        // ---- Cut 3, the Foundry
        // An iron golem is a slow wall: it presses while it sees the hero and forgets at once
        // when it does not (walking away from it works; nothing else does without range).
        "iron_golem" => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else if sees && dist <= 4 {
                approach(run, cx, mi);
            } else {
                run.monsters[mi].last_seen = None;
                if run.rng.chance(15) {
                    wander(run, cx, mi);
                }
            }
        }
        // A bell sentinel rings on sight: +2 alert, and the whole floor hears it.
        "bell_sentinel" => {
            if sees && cooldown == 0 {
                run.monsters[mi].cooldown = 200;
                run.alert = (run.alert + 2).min(8);
                if run.floor.map.is_visible(mp) {
                    callout(run, cx, "bell!");
                    learn_tag(run, cx, kind, "alarm");
                }
                noise(run, cx, mp, 40);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else if sees && dist < 3 {
                step_away(run, cx, mi, hp, false);
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // A slag crawler winds up like an ogre; the fire it leaves is in `move_monster`.
        "slag_crawler" => {
            if adjacent && mp.adjacent(hp) {
                telegraph(run, cx, mi, "heaves", Pending::HeavyHit);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // A smith armours every ally in view (+3 def, 30 ticks) and keeps its distance.
        "smith" => {
            if sees && cooldown == 0 {
                run.monsters[mi].cooldown = 30;
                buff_allies(run, cx, mi, 3, 30, "armours");
            } else if adjacent && !(run.rng.chance(50) && step_away(run, cx, mi, hp, false)) {
                monster_attack(run, cx, mi, 1, "attack");
            } else if !adjacent && (dist > 3 || !sees) {
                chase(run, cx, mi, sees);
            }
        }
        // ---- the Deep
        // A mirror shade copies the hero's class verb: bash, backstab, shoot or bolt.
        "mirror_shade" => {
            let class = run.hero.class;
            let ranged_copy = matches!(class, Class::Ranger | Class::Caster);
            if ranged_copy && sees && (2..=BOW_RANGE).contains(&dist) {
                let id = run.monsters[mi].id;
                projectile(run, cx, id, HERO_ID, mp, hp);
                monster_attack(run, cx, mi, 1, if class == Class::Ranger { "shoot" } else { "bolt" });
                if run.floor.map.is_visible(mp) {
                    learn_tag(run, cx, kind, "mirror");
                }
            } else if adjacent {
                match class {
                    Class::Fighter => {
                        monster_attack(run, cx, mi, 1, "bash");
                        if run.over.is_none() && run.rng.chance(25) {
                            run.hero.paralysed = 12;
                            callout(run, cx, "bashed");
                        }
                    }
                    Class::Rogue => {
                        let unaware = run.hero.confused > 0 || run.hero.paralysed > 0 || run.monsters.iter().filter(|o| o.hp > 0 && o.hostile() && o.pos.adjacent(hp)).count() >= 2;
                        monster_attack(run, cx, mi, if unaware { 2 } else { 1 }, "backstab");
                    }
                    _ => monster_attack(run, cx, mi, 1, "attack"),
                }
                if run.floor.map.is_visible(mp) {
                    learn_tag(run, cx, kind, "mirror");
                }
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // A siren's aura (2 tiles, in `tick_regen_and_auras`) does the work; it closes and bites.
        "siren" => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // ---- the Sanctum
        // A warden flips its face every 40 ticks, announced.
        "warden" => {
            if sees && cooldown == 0 {
                run.monsters[mi].cooldown = 40;
                telegraph(run, cx, mi, "shifts", Pending::Flip);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // An acolyte heals the most hurt ally in reach (5 HP every 30 ticks) and hangs back.
        "acolyte" => {
            let hurt = run
                .monsters
                .iter()
                .enumerate()
                .filter(|(j, o)| *j != mi && o.hp > 0 && o.hostile() && o.hp < o.max_hp && o.pos.cheb(mp) <= 5)
                .min_by_key(|(_, o)| (o.hp * 100 / o.max_hp.max(1), o.id))
                .map(|(j, _)| j);
            if let Some(j) = hurt.filter(|_| cooldown == 0) {
                run.monsters[mi].cooldown = 30;
                let m = &mut run.monsters[j];
                m.hp = (m.hp + 5).min(m.max_hp);
                if run.floor.map.is_visible(mp) {
                    callout(run, cx, "heals");
                    learn_tag(run, cx, kind, "healer");
                }
            } else if adjacent && !(sees && step_away(run, cx, mi, hp, false)) {
                monster_attack(run, cx, mi, 1, "attack");
            } else if !adjacent && (dist > 3 || !sees) {
                chase(run, cx, mi, sees);
            }
        }
        // A sentinel's gaze, telegraphed, stuns whoever stands within two tiles.
        "sentinel" => {
            if sees && dist <= 2 && cooldown == 0 {
                run.monsters[mi].cooldown = 60;
                telegraph(run, cx, mi, "gazes", Pending::Gaze);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // ---- Cut 3 bosses
        // The Foundry Master: melee comes back at the hero (`reflect_melee`, in
        // `damage_monster`); his smiths keep him armoured; hurt, he armours himself. The counter
        // is the smiths first, then range or throws.
        "foundry_master" => {
            let hurt = run.monsters[mi].hurt_since_action;
            run.monsters[mi].hurt_since_action = false;
            if sees && !run.monsters[mi].introduced {
                run.monsters[mi].introduced = true;
                telegraph(run, cx, mi, "hammers", Pending::Hammer);
            } else if hurt && cooldown == 0 {
                run.monsters[mi].cooldown = 40;
                buff_allies(run, cx, mi, 2, 40, "armours");
            } else if adjacent && mp.adjacent(hp) && run.rng.chance(30) {
                telegraph(run, cx, mi, "hammers", Pending::Hammer);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        // The Lurker Queen is blind: she goes to the last noise and calls lurkers to it
        // (`noise` → `Pending::Call`). Silence, or a fast kill, is the counter.
        "lurker_queen" => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, false);
            }
        }
        // The Mirror King: a verb used three times running comes back (`mirror_reflects`);
        // he warns on the first repeat, and on his first sight of the hero.
        "mirror_king" => {
            let n = run.verb_ring.len();
            let repeat = if run.monsters[mi].modifiers.is_some_and(|mods| mods.tight_mirror) { n >= 1 } else { n >= 2 && run.verb_ring[n - 1] == run.verb_ring[n - 2] };
            if sees && cooldown == 0 && (repeat || !run.monsters[mi].introduced) {
                run.monsters[mi].introduced = true;
                run.monsters[mi].cooldown = 20;
                telegraph(run, cx, mi, "mirrors", Pending::Mirror);
            } else if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
        _ => {
            if adjacent {
                monster_attack(run, cx, mi, 1, "attack");
            } else {
                chase(run, cx, mi, sees);
            }
        }
    }
}

/// Approach the hero if seen, else walk to the last known position, else forget.
fn chase(run: &mut Run, cx: &mut Ctx, mi: usize, sees: bool) {
    if sees {
        approach(run, cx, mi);
        return;
    }
    let Some(target) = run.monsters[mi].last_seen else {
        drift(run, cx, mi);
        return;
    };
    let mp = run.monsters[mi].pos;
    if mp.cheb(target) <= 1 {
        run.monsters[mi].last_seen = None;
        drift(run, cx, mi);
        return;
    }
    // Greedy step toward the last seen position.
    let map = &run.floor.map;
    let water_only = run.monsters[mi].has_tag("water");
    let q = DIRS8
        .iter()
        .map(|d| mp.step(*d))
        .filter(|q| map.can_step(mp, *q) && !run.occupied(*q) && (!water_only || map.get(*q) == Tile::Water))
        .min_by_key(|q| (q.cheb(target), q.x, q.y));
    match q {
        Some(q) if q.cheb(target) < mp.cheb(target) => move_monster(run, cx, mi, q),
        _ => {
            // Blocked: fall back to the hero's field (it knows the map).
            if !approach(run, cx, mi) {
                run.monsters[mi].last_seen = None;
            }
        }
    }
}

fn ally_act(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let mp = run.monsters[mi].pos;
    let target = run
        .monsters
        .iter()
        .enumerate()
        .filter(|(j, o)| *j != mi && o.hp > 0 && o.hostile() && !o.dormant && o.pos.adjacent(mp))
        .min_by_key(|(_, o)| (o.hp, o.id))
        .map(|(j, _)| j);
    if let Some(ti) = target {
        let atk = run.monsters[mi].effective_atk();
        let def = run.monsters[ti].effective_def();
        let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
        if ally_mirror(run, cx, mi, ti, "attack", hit, dmg) {
            return;
        }
        let (src, dst) = (run.monsters[mi].id, run.monsters[ti].id);
        cx.events.push(Ev::Attack { t: run.turn, src, dst, dmg, hit, verb: Some("attack".into()) });
        if hit {
            damage_monster(run, cx, ti, dmg, &Src::Mon(mi));
        }
        return;
    }
    let hp = run.hero.pos;
    if mp.cheb(hp) > 2 {
        approach(run, cx, mi);
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
        let q = DIRS8.iter().map(|d| mp.step(*d)).filter(|q| map.can_step(mp, *q) && !run.occupied(*q)).min_by_key(|q| (q.cheb(fp), q.x, q.y));
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
        // Cut 5 §4: a stray (a lost heir's companion) takes the leash whatever its wounds.
        let weak = m.hp * 100 / m.max_hp.max(1) < 25 || m.stray;
        let tag_ok = match sel.strip_prefix("tag:") {
            Some(t) => m.has_tag(t),
            None => true,
        };
        weak && tag_ok && !m.is_boss() && !m.summoned && !m.neutral
    });
    let Some(mi) = cand else { return false };
    run.last_target = Some(run.monsters[mi].id);
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if !mp.adjacent(hp) {
        let (goal, parent) = nearest_goal(run, &hero_avoids(run, true), &mp.neighbours8());
        return match goal {
            Some(g) if g != hp => step_towards(run, cx, g, &parent),
            _ => false,
        };
    }
    // Spend the leash and the turn.
    run.hero.inv[li].amount -= 1;
    if run.hero.inv[li].amount <= 0 {
        run.hero.inv.remove(li);
        crate::provenance::spent(run, cx, "leash", "leash spent on tame".into());
    }
    let kind = run.monsters[mi].kind.clone();
    let stray = run.monsters[mi].stray;
    let chance = if stray { crate::engine::STRAY_TAME } else { crate::engine::tame_chance(cx.facts, &kind) };
    let ok = run.rng.chance(chance);
    let id = run.monsters[mi].id;
    cx.events.push(Ev::Tame { t: run.turn, id, kind: kind.clone(), ok });
    if ok {
        let n = run.tamed.len() as u32;
        let cid = 1_000_000 + run.id * 100 + n;
        // A stray keeps the name a previous heir gave it.
        // Cut 29 §6 (AX): a grudge tamed keeps its name — its grudge closes as tamed (never
        // avenged: `LineageState::grudges`, at the run's end).
        let grudge = run.monsters[mi].grudge.then(|| run.monsters[mi].name.clone()).flatten();
        let name = match run.monsters[mi].name.clone().filter(|_| stray).or(grudge.clone()) {
            Some(n) => n,
            None => crate::descent::grudge_name(&mut run.rng),
        };
        if let Some(g) = grudge {
            run.monsters[mi].grudge = false;
            run.tamed_grudges.push(g);
        }
        {
            let m = &mut run.monsters[mi];
            crate::endgame::normalise_tamed(m, run.depth);
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
        let mut rec = crate::engine::new_companion(cid, &run.monsters[mi], name.clone());
        if stray {
            run.monsters[mi].stray = false;
            rec.gen = cx.lost.iter().find(|l| l.name == name).map(|l| l.gen).unwrap_or(0);
            run.strays_tamed.push(name.clone());
            run.met_situation("stray");
            crate::sifter::open_situation(run, crate::sifter::Setup::Stray, "stray", &kind, &name);
        }
        run.companions.push(rec);
        run.tamed.push((run.turn, kind.clone()));
        learn(run, cx, format!("tamed:{kind}"));
        note(run, cx, format!("Tamed a {}: {}.", crate::engine::kind_title(&kind), name));
        crate::oath::beat(run, cx, run.acting_row);   // Cut 28b: a `tame a new kind` oath is kept here
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
            // Cut 5 §4: a sleeping den is scenery to the pack as to the hero.
            i != mi && m.hp > 0 && m.hostile() && !m.dormant && (map.is_visible(m.pos) || m.pos.adjacent(mp))
        })
        .collect();
    foes.sort_by_key(|&i| (run.monsters[i].pos.cheb(mp), run.monsters[i].id));
    let adj = foes.iter().filter(|&&i| run.monsters[i].pos.adjacent(mp)).count() as i32;
    let nearest = foes.first().copied();
    let lowest = foes.iter().copied().min_by_key(|&i| (run.monsters[i].hp, run.monsters[i].id));
    View { engage: foes.clone(), foes, adj, nearest, lowest }
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
    let atk = run.monsters[mi].effective_atk();
    let atk = (atk.0 * mult_num / 2, atk.1 * mult_num / 2);
    let def = run.monsters[ti].effective_def();
    let (hit, dmg) = roll_hit(&mut run.rng, atk, def);
    if ally_mirror(run, cx, mi, ti, verb, hit, dmg) {
        return 0;
    }
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

fn try_companion_verb(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &Verb, v: &View) -> bool {
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
                    let (src, dst, tp) = (run.monsters[mi].id, run.monsters[ti].id, run.monsters[ti].pos);
                    projectile(run, cx, src, dst, mp, tp);
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
                let gold = (3 * run.depth as i32 + crate::engine::GOLD_DIVISOR / 2) / crate::engine::GOLD_DIVISOR;
                run.loot_add_gold(gold);
                run.monsters[mi].stole_from.push(tid);
                let id = run.monsters[mi].id;
                cx.events.push(Ev::Steal { t: run.turn, id, item: format!("gold ${gold}"), amount: Some(gold) });
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
            cx.events.push(Ev::Spawn { t: run.turn, e:Box::new(e) });
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
        // Cut 3: a bred `mirror` companion copies the hero's class verb — the fighter's bash
        // (stun), the rogue's backstab (×1.5), the ranger's shot and the caster's bolt at range.
        "mimic" => {
            if !run.monsters[mi].has_tag("mirror") {
                return false;
            }
            match run.hero.class {
                Class::Ranger | Class::Caster | Class::Gunner => {
                    let target = v.foes.iter().copied().find(|&i| {
                        let tp = run.monsters[i].pos;
                        (1..=BOW_RANGE).contains(&tp.cheb(mp)) && run.floor.map.los(mp, tp)
                    });
                    let Some(ti) = target else { return false };
                    let (src, dst, tp) = (run.monsters[mi].id, run.monsters[ti].id, run.monsters[ti].pos);
                    projectile(run, cx, src, dst, mp, tp);
                    companion_melee(run, cx, mi, ti, if run.hero.class == Class::Caster { "bolt" } else { "shoot" }, 2);
                    true
                }
                Class::Fighter => {
                    let Some(ti) = adj_target else { return false };
                    if companion_melee(run, cx, mi, ti, "bash", 2) > 0 && run.monsters.get(ti).is_some_and(|m| m.hp > 0) {
                        run.monsters[ti].stun = 10;
                    }
                    true
                }
                Class::Rogue => {
                    let Some(ti) = adj_target else { return false };
                    companion_melee(run, cx, mi, ti, "backstab", 3);
                    true
                }
            }
        }
        "follow" => {
            if mp.cheb(run.hero.pos) > 2 {
                approach(run, cx, mi);
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

/// Cut 4: a companion's row is announced as `<kind>: <verb>` (`jackal: flank`), once per
/// streak of the same verb, and only in the hero's view.
fn companion_callout(run: &mut Run, cx: &mut Ctx, mi: usize, verb: &str) {
    if run.monsters[mi].last_verb == verb {
        return;
    }
    run.monsters[mi].last_verb = verb.to_string();
    if !run.floor.map.is_visible(run.monsters[mi].pos) || matches!(verb, "follow") {
        return;
    }
    let kind = crate::engine::kind_title(&run.monsters[mi].kind).to_lowercase();
    callout(run, cx, &format!("{kind}: {}", verb.replace('_', " ")));
}

/// A companion acts on its own rows; fallback: fight adjacent, stay within 2 of the hero.
/// Cut 20 §2 (AD: "8 pets lost in a session"): a companion at ≤ `PET_FLEE_PCT` of its hp
/// is out of the fight — it falls back to the hero's side and engages nothing — until a
/// descent heals it (`turn::descend`). Wounded like this it dies only cornered
/// (`pet_cornered`, `turn::damage_monster`).
pub const PET_FLEE_PCT: i32 = 30;
/// Cut 20 §2: a companion's hp after each descent, at least (percent of max).
pub const PET_DESCENT_PCT: i32 = 60;

pub fn pet_wounded(m: &Monster) -> bool {
    m.is_companion() && m.hp > 0 && m.hp * 100 <= m.max_hp * PET_FLEE_PCT
}

/// Awake hostiles adjacent to `p`.
fn threats_at(run: &Run, p: Pos) -> usize {
    run.monsters.iter().filter(|o| o.hp > 0 && o.hostile() && !o.dormant && o.pos.cheb(p) <= 1).count()
}

/// Cut 20 §2: a wounded companion with nowhere to step — every way out walled or stood on.
pub fn pet_cornered(run: &Run, mi: usize) -> bool {
    let mp = run.monsters[mi].pos;
    let map = &run.floor.map;
    !mp.neighbours8().into_iter().any(|q| map.in_bounds(q) && map.can_step(mp, q) && !run.occupied(q))
}

/// Cut 20 §2: the wounded companion's turn — to the hero's side, away from the foes.
fn pet_fall_back(run: &mut Run, cx: &mut Ctx, mi: usize) {
    if !run.monsters[mi].fleeing {
        run.monsters[mi].fleeing = true;
        run.monsters[mi].sent = false;
        if run.floor.map.is_visible(run.monsters[mi].pos) {
            let name = run.monsters[mi].name.clone().unwrap_or_else(|| crate::engine::kind_title(&run.monsters[mi].kind));
            callout(run, cx, &format!("{name} falls back"));
        }
    }
    let hp = run.hero.pos;
    let mp = run.monsters[mi].pos;
    if mp.cheb(hp) > 2 && threats_at(run, mp) == 0 {
        approach(run, cx, mi);
        return;
    }
    let map = &run.floor.map;
    let score = |q: Pos| (threats_at(run, q), q.cheb(hp).max(1), q.x, q.y);
    let best = std::iter::once(mp)
        .chain(mp.neighbours8().into_iter().filter(|q| map.in_bounds(*q) && map.can_step(mp, *q) && !run.occupied(*q)))
        .min_by_key(|q| score(*q));
    if let Some(q) = best {
        if q != mp && score(q) < score(mp) {
            move_monster(run, cx, mi, q);
        }
    }
}

fn companion_act(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let Some(cid) = run.monsters[mi].cid else { return };
    if pet_wounded(&run.monsters[mi]) {
        run.monsters[mi].hurt_since_action = false;
        pet_fall_back(run, cx, mi);
        return;
    }
    if run.monsters[mi].fleeing {
        run.monsters[mi].fleeing = false;
    }
    let (rows, max_rows) = match run.companion(cid) {
        Some(c) => (c.rules.rows.clone(), c.max_rows),
        None => (Vec::new(), 2),
    };
    let v = companion_view(run, mi);
    if run.monsters[mi].sent {
        if v.foes.is_empty() {
            run.monsters[mi].sent = false;
        } else if try_companion_verb(run, cx, mi, &Verb::new("attack"), &v) {
            run.monsters[mi].hurt_since_action = false;
            return;
        }
    }
    for row in rows.iter().take(max_rows) {
        if !row.conds.iter().all(|c| companion_cond(run, cx, mi, &v, c)) {
            continue;
        }
        if try_companion_verb(run, cx, mi, &row.verb, &v) {
            run.monsters[mi].hurt_since_action = false;
            companion_callout(run, cx, mi, &row.verb.v);
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
        approach(run, cx, mi);
    }
}

// ---------------------------------------------------------------- class verbs (Addendum C)

fn verb_cleave(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if run.hero.cleave_cd > 0 || v.adj == 0 {
        return false;
    }
    let hp = run.hero.pos;
    let targets: Vec<usize> = v.foes.iter().copied().filter(|&i| run.monsters[i].pos.adjacent(hp)).collect();
    run.hero.cleave_cd = 60;
    crate::provenance::cooldown(run, cx, "cleave");
    callout(run, cx, "cleave");
    for mi in targets {
        if run.monsters[mi].hp > 0 {
            hero_attack(run, cx, mi, "cleave", false);
        }
    }
    true
}

fn verb_taunt(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    if v.foes.is_empty() || run.taunt_t > 0 {
        return false;
    }
    let hp = run.hero.pos;
    for &i in &v.foes {
        run.monsters[i].awake = true;
        run.monsters[i].last_seen = Some(hp);
        run.monsters[i].pending = None;
        run.monsters[i].telegraph = None;
    }
    run.taunt_t = 30;
    callout(run, cx, "taunt");
    true
}

/// Rogue: double (or triple, from vanish) damage on an unaware or stunned adjacent foe.
fn verb_backstab(run: &mut Run, cx: &mut Ctx, v: &View, mult: i32) -> bool {
    let hp = run.hero.pos;
    let target = v.foes.iter().copied().find(|&i| {
        let m = &run.monsters[i];
        m.pos.adjacent(hp) && (!m.awake || m.stun > 0 || run.hero.vanish_t > 0)
    });
    let Some(mi) = target else { return false };
    hero_attack_mult(run, cx, mi, if mult >= 3 { "ambush" } else { "backstab" }, false, mult);
    if mult >= 3 {
        run.hero.vanish_t = 0;
    }
    true
}

/// Rogue: blink to the tile behind the nearest foe within 3.
fn verb_shadowstep(run: &mut Run, cx: &mut Ctx, v: &View) -> bool {
    let hp = run.hero.pos;
    let Some(mi) = v.foes.iter().copied().find(|&i| run.monsters[i].pos.cheb(hp) <= 3) else { return false };
    let mp = run.monsters[mi].pos;
    let behind = Pos::new(mp.x + (mp.x - hp.x).signum(), mp.y + (mp.y - hp.y).signum());
    let cands: Vec<Pos> = std::iter::once(behind)
        .chain(behind.neighbours8())
        .filter(|q| run.floor.map.passable(*q) && !run.occupied(*q) && q.adjacent(mp) && *q != hp)
        .collect();
    let Some(q) = cands.first().copied() else { return false };
    move_hero(run, cx, q);
    callout(run, cx, "shadowstep");
    true
}
