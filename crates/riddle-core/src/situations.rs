//! Cut 7 §3: one situation per floor band that a fixed preset cannot pass but one row solves.
//! Each is a fact, a token (`on_see: <fact>`) and an episode.
//!
//! - **D3 the thief's den**: three monkeys asleep round the down stairs. A hero who steps up
//!   to the stairs is pounced on: each snatches something (the vault-brought item first, then
//!   the pack, then the weapon in hand) and runs. A raid (`on_see: den → attack nearest`, a
//!   free blow on a sleeper) scatters them empty-handed; `foes>=` never sees sleepers, so the
//!   preset walks into it. Pass: leave D3 with everything snatched back in the pack.
//! - **D6 the gas lock**: every door of the down-stairs room holds a bloat (up to four); they
//!   stir when the hero comes near, drift at him, swell the moment they stand beside him and
//!   burst one action later. A hero who steps back (`foe_tag gas · adj≥1 → retreat`) watches
//!   them pop at arm's length; one who swings (or stands) takes the cloud. Pass: leave D6 with
//!   ≤ `LOCK_PASS_GAS` gas damage taken.
//! - **D9 the captive gate**: a captive chained on the down stairs; no swapping past. Freed
//!   (`free_captive`) it fights for the hero; cut down it is `no_friends`. Pass: freed. The
//!   preset's `attack nearest` takes the coward's way (a chained captive counts as a foe once
//!   adjacent), so no set ever stalls at the gate.
//! - **D12 the crypt's hunger**: the Crypt reaches up into the Fens' last floors — three
//!   hunting wraiths, and the floor itself takes a point of max HP every `HUNGER_TURNS` turns
//!   unless the hero carries a lantern or has lit the floor's shrine (`pray`: free here).
//!   Pass: leave D12 lit or lantern-lit. Row: `on_see: shrine → pray`.
use crate::chronicle::{callout, note};
use crate::descent::SITUATION_DEPTHS;
use crate::engine::{Ctx, Run};
use crate::facts::learn;
use crate::geom::Pos;
use crate::monster::Monster;
use crate::tiles::Tile;
use crate::wire::Ev;

/// Gas damage a hero may take on the lock floor and still have passed it (two turns' worth).
pub const LOCK_PASS_GAS: i32 = 6;
/// Turns between the hunger's bites on an unlit D12.
pub const HUNGER_TURNS: u32 = 12;
/// Ticks a lock bloat swells before it bursts (it swells once beside the hero: one action to
/// step back).
pub const LOCK_FUSE: i32 = 10;
pub const LOCK_REACH: i32 = 1;
/// How far a lock bloat or a den thief senses the hero (path steps).
pub const SENSE_STEPS: i32 = 5;

pub fn depth_of(what: &str) -> Option<u32> {
    SITUATION_DEPTHS.iter().find(|(k, _)| *k == what).map(|(_, d)| *d)
}

pub fn at(depth: u32) -> Option<&'static str> {
    SITUATION_DEPTHS.iter().find(|(_, d)| *d == depth).map(|(k, _)| *k)
}

fn has_situation(run: &Run, what: &str) -> bool {
    run.monsters.iter().any(|m| m.situation.as_deref() == Some(what))
}

/// Place the band's situation on a fresh floor (after `populate_floor`).
pub fn place(run: &mut Run) {
    match at(run.depth) {
        Some("den") => place_den(run),
        Some("lock") => place_lock(run),
        Some("captive") => place_captive(run),
        Some("hunger") => place_hunger(run),
        _ => {}
    }
}

/// Three monkeys asleep round the down stairs (within two tiles).
fn place_den(run: &mut Run) {
    let down = run.floor.stairs_down;
    let up = run.floor.stairs_up;
    let mut spots: Vec<Pos> = run.floor.open_tiles().into_iter().filter(|p| !run.occupied(*p) && *p != down && p.cheb(down) <= 2).collect();
    spots.sort_by_key(|p| (p.cheb(down), p.x, p.y));
    // Sleepers never wall the stairs off: each spot must leave a way through.
    let mut chosen: Vec<Pos> = Vec::new();
    for p in spots {
        if chosen.len() >= 3 {
            break;
        }
        let map = &run.floor.map;
        let d = map.bfs(up, false, &|q| q == p || chosen.contains(&q));
        if d[map.idx(down)] > 0 {
            chosen.push(p);
        }
    }
    for p in chosen {
        let id = run.new_id();
        let mut m = Monster::spawn(id, "monkey", p, run.depth);
        m.situation = Some("den".into());
        m.dormant = true;
        m.awake = false;
        // Wiry and slippery: a den thief lives through a blow or two (a hero who swings at
        // whatever is nearest is flanked before the third goes down).
        m.max_hp = 10;
        m.hp = 10;
        m.def = 2;
        run.monsters.push(m);
    }
}

/// The lock: every way into the down-stairs room — the passable tiles outside it that touch
/// it (its doors and the corridor mouths). No route to the stairs avoids them. At most
/// `LOCK_MAX` bloats, the doors nearest the stairs first.
pub const LOCK_MAX: usize = 4;

fn lock_tiles(run: &Run) -> Vec<Pos> {
    let down = run.floor.stairs_down;
    let Some(room) = run.floor.rooms.iter().find(|r| r.contains(down)).copied() else { return Vec::new() };
    let map = &run.floor.map;
    let mut lock: Vec<Pos> = Vec::new();
    for y in room.y - 1..=room.y + room.h {
        for x in room.x - 1..=room.x + room.w {
            let p = Pos::new(x, y);
            if room.contains(p) || !map.in_bounds(p) || !map.passable(p) || matches!(map.get(p), Tile::StairsUp | Tile::StairsDown) {
                continue;
            }
            if p.neighbours8().into_iter().any(|q| room.contains(q) && map.can_step(p, q)) {
                lock.push(p);
            }
        }
    }
    lock.sort_by_key(|p| (p.cheb(down), p.x, p.y));
    lock.truncate(LOCK_MAX);
    // Each door's corridor holds two more bloats behind it (a one-door room still locks).
    let doors = lock.clone();
    for d in doors {
        let mut cur = d;
        for _ in 0..2 {
            if lock.len() >= LOCK_MAX {
                break;
            }
            let in_room = |q: Pos| run.floor.rooms.iter().any(|r| r.contains(q));
            let next = cur.neighbours8().into_iter().find(|q| map.in_bounds(*q) && matches!(map.get(*q), Tile::Floor | Tile::Door) && map.can_step(cur, *q) && !lock.contains(q) && !in_room(*q) && q.cheb(down) > cur.cheb(down));
            let Some(q) = next else { break };
            lock.push(q);
            cur = q;
        }
    }
    lock.sort_by_key(|p| (p.x, p.y));
    lock
}

fn place_lock(run: &mut Run) {
    let lock = lock_tiles(run);
    if lock.is_empty() {
        return;
    }
    // Nothing stands or lies in the lock but its bloats.
    run.items.retain(|fi| !lock.contains(&fi.pos));
    run.monsters.retain(|m| !lock.contains(&m.pos) || m.ally);
    for p in &lock {
        let id = run.new_id();
        let mut m = Monster::spawn(id, "bloat", *p, run.depth);
        m.situation = Some("lock".into());
        m.awake = false;
        run.monsters.push(m);
    }
    run.lock_tiles = lock;
}

/// A captive chained on the down stairs.
fn place_captive(run: &mut Run) {
    let down = run.floor.stairs_down;
    run.monsters.retain(|m| m.pos != down || m.ally);
    let id = run.new_id();
    let mut m = Monster::spawn(id, "captive", down, run.depth);
    m.situation = Some("captive".into());
    m.neutral = true;
    m.awake = false;
    run.monsters.push(m);
}

/// A shrine a few steps down the path from the entrance; three wraiths already hunting.
fn place_hunger(run: &mut Run) {
    let up = run.floor.stairs_up;
    let hero = run.hero.pos;
    let dist = run.floor.map.bfs(up, false, &|_| false);
    let map = &run.floor.map;
    let mut cands: Vec<Pos> = (0..map.tiles.len())
        .filter(|&i| map.tiles[i] == Tile::Floor && (4..=7).contains(&dist[i]))
        .map(|i| map.pos(i))
        .filter(|p| !run.occupied(*p) && run.item_at(*p).is_none())
        .collect();
    cands.sort_by_key(|p| (dist[map.idx(*p)], p.x, p.y));
    if let Some(p) = cands.first().copied() {
        run.floor.map.set(p, Tile::Shrine);
    }
    let mut spots: Vec<Pos> = run.floor.open_tiles().into_iter().filter(|p| !run.occupied(*p) && p.cheb(hero) > 8).collect();
    spots.sort_by_key(|p| (dist[run.floor.map.idx(*p)], p.x, p.y));
    let n = spots.len();
    for k in 0..3 {
        let Some(p) = spots.get(n / 4 + k * n / 4).copied() else { break };
        let id = run.new_id();
        let mut m = Monster::spawn(id, "wraith", p, run.depth);
        m.situation = Some("hunger".into());
        m.awake = true;
        m.last_seen = Some(hero);
        run.monsters.push(m);
    }
}

/// `on_see: den | lock | captive | hunger` — the situation is in view (or, for the hunger,
/// biting: the floor unlit).
pub fn sees(run: &Run, what: &str) -> bool {
    let map = &run.floor.map;
    match what {
        "den" | "lock" => run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.situation.as_deref() == Some(what) && map.is_visible(m.pos)),
        "captive" => run.monsters.iter().any(|m| m.hp > 0 && m.neutral && m.situation.as_deref() == Some("captive") && map.is_visible(m.pos)),
        "hunger" => hunger_floor(run) && !lit(run),
        _ => false,
    }
}

pub fn hunger_floor(run: &Run) -> bool {
    at(run.depth) == Some("hunger") && run.tile_pos(Tile::Shrine).is_some()
}

/// Light on the hunger floor: the shrine lit, or a lantern in the pack.
pub fn lit(run: &Run) -> bool {
    run.lit || run.hero.inv.iter().any(|i| i.kind == "lantern")
}

/// The situations in view become facts and count for the run (once per floor).
pub fn seen(run: &mut Run, cx: &mut Ctx) {
    let Some(what) = at(run.depth) else { return };
    // The den is met on sight; the lock's reek and the captive's cry carry to the entrance.
    let met = match what {
        "hunger" => false, // met by its first bite
        "den" => run.monsters.iter().any(|m| m.situation.as_deref() == Some("den") && m.hp > 0 && run.floor.map.is_visible(m.pos)),
        _ => has_situation(run, what),
    };
    if met && run.met_situation(what) {
        let text = match what {
            "den" => "A den of thieves.",
            "lock" => "The air stings: bloats ahead.",
            _ => "A cry from the dark: a captive, chained.",
        };
        note(run, cx, text.into());
        learn(run, cx, what.into());
    }
    // Den thieves and lock bloats stir when the hero is close or in their sight.
    wake(run, cx, what);
}

fn wake(run: &mut Run, cx: &mut Ctx, what: &str) {
    if what != "lock" {
        return;
    }
    if !has_situation(run, what) || run.monsters.iter().all(|m| m.situation.as_deref() != Some(what) || m.awake) {
        return;
    }
    let hp = run.hero.pos;
    let hd = crate::turn::hero_dist(run).to_vec();
    let map = &run.floor.map;
    let near = run.monsters.iter().any(|m| {
        m.hp > 0 && m.situation.as_deref() == Some(what) && {
            let d = hd[map.idx(m.pos)];
            map.is_visible(m.pos) || (0..=SENSE_STEPS).contains(&d)
        }
    });
    if !near {
        return;
    }
    let visible = run.monsters.iter().any(|m| m.situation.as_deref() == Some(what) && map.is_visible(m.pos));
    for m in run.monsters.iter_mut().filter(|m| m.hp > 0 && m.situation.as_deref() == Some(what)) {
        m.awake = true;
        m.dormant = false;
        m.last_seen = Some(hp);
    }
    if visible {
        callout(run, cx, "they stir");
    }
}

/// Before the hero acts: a hero on the down stairs with den thieves still asleep round them
/// is pounced on — each snatches one thing and runs.
pub fn before_action(run: &mut Run, cx: &mut Ctx) {
    if at(run.depth) != Some("den") {
        return;
    }
    let hp = run.hero.pos;
    let down = run.floor.stairs_down;
    if hp != down {
        return;
    }
    let thieves: Vec<usize> = (0..run.monsters.len()).filter(|&i| run.monsters[i].hp > 0 && run.monsters[i].dormant && run.monsters[i].situation.as_deref() == Some("den")).collect();
    if thieves.is_empty() {
        return;
    }
    run.met_situation("den");
    learn(run, cx, "den".into());
    note(run, cx, "The den wakes: thieves on every side.".into());
    callout(run, cx, "thieves!");
    for mi in thieves {
        let m = &mut run.monsters[mi];
        m.dormant = false;
        m.awake = true;
        m.last_seen = Some(hp);
        // Step in beside the hero if there is room.
        if !m.pos.adjacent(hp) {
            let free = hp.neighbours8().into_iter().find(|q| run.floor.map.passable(*q) && !run.occupied(*q));
            if let Some(q) = free {
                let id = run.monsters[mi].id;
                run.monsters[mi].pos = q;
                cx.events.push(Ev::Move { t: run.turn, id, x: q.x, y: q.y });
            }
        }
        if run.monsters[mi].pos.adjacent(hp) {
            snatch(run, cx, mi);
        }
    }
}

/// A den thief takes one thing: a vault-brought pack item, else any pack item, else the
/// weapon in hand; then it runs.
fn snatch(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let inv = &run.hero.inv;
    let pick = inv.iter().position(|i| run.brought.contains(&i.id)).or_else(|| (!inv.is_empty()).then_some(0));
    let it = match pick {
        Some(i) => run.hero.inv.remove(i),
        None => match run.hero.weapon.take() {
            Some(w) => w,
            None => return,
        },
    };
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    run.den_stolen.push(it.id);
    crate::provenance::stolen(run, cx, &it.kind, "monkey", true, &label);
    let before = run.loot;
    run.loot_add(-it.value());
    let amount = (before > run.loot).then(|| before - run.loot);
    let id = run.monsters[mi].id;
    run.monsters[mi].stolen = Some(it);
    run.monsters[mi].fleeing = true;
    cx.events.push(Ev::Steal { t: run.turn, id, item: label.clone(), amount });
    run.stolen.push((run.turn, label.clone()));
    note(run, cx, format!("A thief snatched the {label}."));
    match amount {
        Some(g) => callout(run, cx, &format!("stolen ${g}")),
        None => callout(run, cx, "stolen!"),
    }
    crate::facts::learn_tag(run, cx, "monkey", "thief");
}

/// A blow on a sleeping den thief: it wakes alone and runs empty-handed (the raid).
pub fn raided(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let m = &mut run.monsters[mi];
    m.dormant = false;
    m.awake = true;
    m.fear = 1000;
    if run.met_situation("den") {
        learn(run, cx, "den".into());
    }
    callout(run, cx, "it bolts");
}

/// A lock bloat's turn: drift at the hero; beside him (`LOCK_REACH`) it swells and holds, and
/// `LOCK_FUSE` ticks later it bursts (its own death pops the gas). Returns true when it acted.
pub fn lock_bloat_act(run: &mut Run, cx: &mut Ctx, mi: usize) -> bool {
    if run.monsters[mi].situation.as_deref() != Some("lock") || !run.monsters[mi].awake {
        return run.monsters[mi].situation.as_deref() == Some("lock");
    }
    let m = &run.monsters[mi];
    let swollen = m.introduced;
    if swollen && m.cooldown == 0 {
        let hp = m.hp;
        crate::turn::damage_monster(run, cx, mi, hp, &crate::turn::Src::Burst);
        return true;
    }
    if swollen {
        return true; // holds, swelling
    }
    if m.pos.cheb(run.hero.pos) <= LOCK_REACH {
        let id = m.id;
        let visible = run.floor.map.is_visible(m.pos);
        let m = &mut run.monsters[mi];
        m.introduced = true;
        m.cooldown = LOCK_FUSE;
        m.telegraph = Some("swells".into());
        cx.events.push(Ev::Telegraph { t: run.turn, id, what: "swells".into() });
        if visible {
            callout(run, cx, "it swells");
        }
        return true;
    }
    false
}

/// The hunger bites on an unlit D12: −1 max HP (never below 5) every `HUNGER_TURNS` turns.
pub fn hunger_tick(run: &mut Run, cx: &mut Ctx) {
    if !hunger_floor(run) || lit(run) || run.floor_turn == 0 || !run.floor_turn.is_multiple_of(HUNGER_TURNS * crate::engine::TICKS_PER_TURN) {
        return;
    }
    if run.hero.max_hp <= 5 {
        return;
    }
    run.hero.max_hp -= 1;
    run.hero.hp = run.hero.hp.min(run.hero.max_hp);
    if run.met_situation("hunger") {
        note(run, cx, "The floor is hungry. Find light.".into());
        learn(run, cx, "hunger".into());
        crate::sifter::on_hurt(run, "hunger", Some("hunger"));
    }
    callout(run, cx, "hunger");
    cx.events.push(Ev::Hurt { t: run.turn, id: crate::engine::HERO_ID, dmg: 0, hp: run.hero.hp, cause: "hunger".into() });
}

/// `pray` on the hunger floor lights the shrine: free, no swap, the wraiths lose their nerve.
pub fn light_shrine(run: &mut Run, cx: &mut Ctx) {
    run.lit = true;
    run.prayed = true;
    for m in run.monsters.iter_mut().filter(|m| m.situation.as_deref() == Some("hunger")) {
        m.fear = 60;
    }
    run.met_situation("hunger");
    learn(run, cx, "hunger".into());
    note(run, cx, "Lit the shrine. The hunger lifts.".into());
    callout(run, cx, "lit");
    pass(run, cx, "hunger");
}

/// Record a situation passed (once per run).
pub fn pass(run: &mut Run, cx: &mut Ctx, what: &str) {
    if run.passed.iter().any(|p| p == what) {
        return;
    }
    run.passed.push(what.into());
    let text = match what {
        "den" => "Nothing lost to the den.",
        "lock" => "Through the lock, barely touched.",
        "captive" => "The captive fights for him.",
        _ => "Light held.",
    };
    note(run, cx, text.into());
}

/// The floor is being left: the den and the lock are judged now.
pub fn on_leave_floor(run: &mut Run, cx: &mut Ctx) {
    let met = |run: &Run, what: &str| run.situations.iter().any(|(_, s)| s == what);
    match at(run.depth) {
        Some("den") if met(run, "den") => {
            let has = |run: &Run, id: u32| run.hero.inv.iter().chain(run.hero.weapon.iter()).chain(run.hero.armour.iter()).any(|i| i.id == id);
            if run.den_stolen.iter().all(|id| has(run, *id)) {
                pass(run, cx, "den");
            }
        }
        Some("lock") if met(run, "lock") => {
            if run.gas_dmg_floor <= LOCK_PASS_GAS {
                pass(run, cx, "lock");
            }
        }
        Some("hunger") if met(run, "hunger") && lit(run) => pass(run, cx, "hunger"),
        _ => {}
    }
    run.gas_dmg_floor = 0;
    run.den_stolen.clear();
    run.lock_tiles.clear();
    run.lit = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Game;
    use crate::rng::Rng;

    fn floor_at(seed: u64, depth: u32) -> Game {
        let mut g = Game::new(seed);
        g.start_run(Some(seed));
        g.descend_to(depth);
        g
    }

    #[test]
    fn every_band_floor_has_its_situation() {
        for seed in 1..=12u64 {
            let g = floor_at(seed, 3);
            let run = g.run.as_ref().unwrap();
            assert_eq!(run.monsters.iter().filter(|m| m.situation.as_deref() == Some("den")).count(), 3, "seed {seed} den");
            let g = floor_at(seed, 6);
            let run = g.run.as_ref().unwrap();
            let bloats = run.monsters.iter().filter(|m| m.situation.as_deref() == Some("lock")).count();
            assert!(bloats >= 1, "seed {seed} lock");
            // The lock is a lock: no way down with its tiles blocked (unless a room has more
            // doors than `LOCK_MAX`, which the bloat count shows).
            let map = &run.floor.map;
            let d = map.bfs(run.floor.stairs_up, false, &|p| run.lock_tiles.contains(&p));
            assert!(d[map.idx(run.floor.stairs_down)] < 0 || bloats == LOCK_MAX, "seed {seed}: a way round the lock");
            let d = map.bfs(run.floor.stairs_up, false, &|_| false);
            assert!(d[map.idx(run.floor.stairs_down)] > 0, "seed {seed}: the stairs still connect");
            let g = floor_at(seed, 9);
            let run = g.run.as_ref().unwrap();
            assert!(run.monsters.iter().any(|m| m.situation.as_deref() == Some("captive") && m.neutral && m.pos == run.floor.stairs_down), "seed {seed} captive");
            let g = floor_at(seed, 12);
            let run = g.run.as_ref().unwrap();
            assert!(run.tile_pos(Tile::Shrine).is_some(), "seed {seed} hunger shrine");
            assert_eq!(run.monsters.iter().filter(|m| m.situation.as_deref() == Some("hunger")).count(), 3, "seed {seed} wraiths");
            for d in [1u32, 2, 4, 5, 7, 8, 10, 11, 13] {
                let g = floor_at(seed, d);
                assert!(g.run.as_ref().unwrap().monsters.iter().all(|m| m.situation.is_none()), "D{d} carries no band situation");
            }
        }
    }

    /// The gate's shape at a small sample: the preset fails each situation, the one-row answer
    /// passes most (the full numbers are the metrics table).
    #[test]
    fn preset_fails_and_the_row_answers() {
        for (what, _) in SITUATION_DEPTHS {
            let mut p = 0;
            let mut a = 0;
            let mut met = 0;
            for seed in 1..=8u64 {
                let (m, passed, _) = crate::probes::situation_trial(seed, what, false);
                met += m as u32;
                p += passed as u32;
                let (_, passed, _) = crate::probes::situation_trial(seed, what, true);
                a += passed as u32;
            }
            assert!(met >= 7, "{what}: met {met}/8");
            assert!(p <= 2, "{what}: the preset passed {p}/8");
            assert!(a >= 4, "{what}: the answer passed {a}/8");
        }
    }

    #[test]
    fn depth_table() {
        assert_eq!(at(3), Some("den"));
        assert_eq!(at(6), Some("lock"));
        assert_eq!(at(9), Some("captive"));
        assert_eq!(at(12), Some("hunger"));
        assert_eq!(depth_of("lock"), Some(6));
        let _ = Rng::new(1);
    }
}
