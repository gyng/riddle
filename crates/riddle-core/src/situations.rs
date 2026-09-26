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
//!
//! Cut 12 §4: from D3 every floor rolls **one** situation of its band, never the previous
//! floor's kind (`place_twist`); each band shows its flagship (the den, the lock, the
//! captive, the hunger) exactly once, on a floor drawn from the run's seed. The four above
//! are no longer fixed to D3 / D6 / D9 / D12: `Run.floor_twist` says what the floor holds.
use crate::chronicle::{callout, note};
use crate::descent::SITUATION_DEPTHS;
use crate::engine::{Ctx, Lost, Run};
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

/// The band situation's home floor (Cut 7): the first floor of the band it is the flagship of.
pub fn depth_of(what: &str) -> Option<u32> {
    SITUATION_DEPTHS.iter().find(|(k, _)| *k == what).map(|(_, d)| *d)
}

/// The band situation this floor holds, if any (Cut 12 §4: the floor's roll, not its depth).
pub fn at(run: &Run) -> Option<&str> {
    run.floor_twist.as_deref().filter(|t| BAND_KINDS.contains(t))
}

/// The four band situations (Cut 7 §3), each a fact, a token and an episode.
pub const BAND_KINDS: [&str; 4] = ["den", "lock", "captive", "hunger"];
/// Cut 12 §4: every kind a floor can roll, the one word the interstitial and the reel use.
pub const TWISTS: [&str; 8] = ["den", "lock", "captive", "nest", "shrine", "vault", "stray", "hunger"];

/// Cut 13: a situation's word on screen. The three-item cage is the `vault` inside (facts,
/// tokens, tiles) and the *cage* to the player: four QA players and a rater read `A vault:
/// three under a cage` / `Took the X from the vault` as the camp's VAULT.
pub fn twist_word(id: &str) -> &str {
    match id {
        "vault" => "cage",
        other => other,
    }
}
/// The room kinds (a room's interior tile): the only ones a boss floor rolls.
const ROOM_KINDS: [&str; 3] = ["nest", "vault", "shrine"];

/// Cut 12 §4: a depth band — its floors, its flagship (shown exactly once in the band, on a
/// floor drawn from the run's seed) and the kinds its other floors roll from. Caves (the Fens,
/// the Deep) have no stairs room for a lock; the Warrens' bands keep the Cut 7 order (the
/// den's band, then the lock's), the Fens the captive's then the hunger's.
pub struct Band {
    pub first: u32,
    pub last: u32,
    pub flagship: Option<&'static str>,
    pub pool: &'static [&'static str],
}

const WARRENS_POOL: &[&str] = &["den", "nest", "vault", "shrine", "stray"];
const WARRENS_DEEP_POOL: &[&str] = &["lock", "den", "nest", "vault", "shrine", "stray"];
/// Caves have no stairs room for a lock; the room kinds sit on an open spot instead.
const FENS_POOL: &[&str] = &["captive", "den", "hunger", "nest", "vault", "shrine", "stray"];
/// The Crypt keeps its hunger; below it the floors are the biomes' own (the Foundry's forges
/// light it, the Deep and the Sanctum have their own systems) — a hunger on every eighth
/// floor to D33 rested the shipped best set to death (FULL D29 80 % → 47 %).
const CRYPT_POOL: &[&str] = &["den", "lock", "captive", "hunger", "nest", "vault", "shrine", "stray"];
const ROOMS_POOL: &[&str] = &["den", "lock", "captive", "nest", "vault", "shrine", "stray"];
const DEEP_POOL: &[&str] = &["captive", "den", "nest", "vault", "shrine", "stray"];

pub fn band(depth: u32) -> Option<Band> {
    band_on(crate::descent::Route::BASE, depth)
}

/// Cut 26 §1: the band a floor rolls from on a route — a biome keeps its situations wherever it
/// sits: the Burrows their lock, the Fens the captive's floors then the hunger's (the last two of
/// five, the last two of four), the Crypt, the Foundry, the Deep and the Sanctum their pools. The
/// base order keeps its Cut 12 bands exactly (the den's band runs to D5, the Burrows' first).
pub fn band_on(route: crate::descent::Route, depth: u32) -> Option<Band> {
    use crate::descent::Biome;
    let b = |first, last, flagship, pool| Some(Band { first, last, flagship, pool });
    let biome = route.biome(depth);
    let den_to = if route.biome(5) == Biome::Burrows { 5 } else { 4 };
    if (3..=den_to).contains(&depth) {
        return b(3, den_to, Some("den"), WARRENS_POOL);
    }
    if !(3..crate::descent::ENDING_DEPTH).contains(&depth) {
        return None;
    }
    let (first, last) = route.span(biome);
    match biome {
        Biome::Warrens => None,
        Biome::Burrows => b(first.max(den_to + 1), last, Some("lock"), WARRENS_DEEP_POOL),
        Biome::Fens => {
            let split = last - 1;
            if depth < split {
                b(first, split - 1, Some("captive"), FENS_POOL)
            } else {
                b(split, last, Some("hunger"), FENS_POOL)
            }
        }
        Biome::Crypt => b(first, last, None, CRYPT_POOL),
        Biome::Foundry | Biome::Sanctum => b(first, last, None, ROOMS_POOL),
        Biome::Deep => b(first, last, None, DEEP_POOL),
    }
}

/// Cut 12 §4: the floor the band's flagship sits on — drawn from the run's seed among the
/// band's floors that are neither a boss's nor a floor the stray is due on.
fn flagship_depth(run: &Run, band: &Band, lost: &[Lost]) -> Option<u32> {
    band.flagship?;
    let due: Vec<u32> = stray_due_depths(run, band, lost);
    let cands: Vec<u32> = (band.first..=band.last).filter(|d| run.route.boss(*d).is_none() && !due.contains(d)).collect();
    if cands.is_empty() {
        return None;
    }
    let mut rng = crate::rng::Rng::derive(run.seed, crate::rng::hash_str("flagship") ^ band.first as u64);
    Some(cands[rng.below(cands.len() as u32) as usize])
}

/// The floors of the band a stray is forced on: the lineage's first jackal's floor (Cut 8B
/// §3), and D5 while a lost companion waits (Cut 5 §4: by the Warrens' fifth floor).
fn stray_due_depths(run: &Run, band: &Band, lost: &[Lost]) -> Vec<u32> {
    let mut due = Vec::new();
    if let Some((d, _)) = &run.first_stray {
        if (band.first..=band.last).contains(d) {
            due.push(*d);
        }
    }
    if !lost.is_empty() && (band.first..=band.last).contains(&5) {
        due.push(5);
    }
    due
}

/// Cut 12 §4: roll and place this floor's one situation (D3+). The order of choice: a kind
/// forced by a trial; the stray on a floor it is due; the band's flagship on its floor; else
/// one of the band's pool, never the previous floor's kind nor the flagship (it has its own
/// floor). A kind that cannot be placed here (a lock with no doors, a cave with no room)
/// falls through to the next; the floor's word is what was placed.
pub fn place_twist(run: &mut Run, lost: &[Lost]) {
    let depth = run.depth;
    let Some(band) = band_on(run.route, depth) else { return };
    let mut rng = run.rng.side(crate::rng::hash_str("twist") ^ depth as u64);
    let last = run.last_twist.clone();
    let stray_ok = crate::engine::wild_for(run, lost).is_some();
    let boss = run.route.boss(depth).is_some();
    let flag_at = flagship_depth(run, &band, lost);
    let mut order: Vec<&str> = Vec::new();
    if let Some(k) = run.next_twist.take() {
        if let Some(k) = TWISTS.iter().find(|t| **t == k) {
            order.push(k);
        }
    }
    if stray_ok && stray_due_depths(run, &band, lost).contains(&depth) {
        order.push("stray");
    }
    if flag_at == Some(depth) {
        if let Some(f) = band.flagship {
            order.push(f);
        }
    }
    // The pool, shuffled, minus the flagship (its own floor), the last floor's kind, the
    // next band's flagship on this band's last floor (so it never repeats across the seam),
    // the stray when none is due, and — on a boss's floor — everything but the room kinds.
    let next_flag = if depth == band.last { band_on(run.route, depth + 1).and_then(|b| b.flagship) } else { None };
    let mut pool: Vec<&str> = band.pool.iter().copied().filter(|k| Some(*k) != band.flagship && Some(*k) != last.as_deref() && Some(*k) != next_flag && (*k != "stray" || stray_ok) && (!boss || ROOM_KINDS.contains(k))).collect();
    rng.shuffle(&mut pool);
    order.extend(pool);
    let mut used: Vec<usize> = Vec::new();
    let skip = std::env::var("RIDDLE_SKIP_TWIST").unwrap_or_default();
    for kind in order {
        if skip.split(',').any(|k| k == kind) { continue; }
        let placed = match kind {
            "den" => place_den(run),
            "lock" => place_lock(run),
            "captive" => place_captive(run),
            "hunger" => place_hunger(run),
            "stray" => match crate::engine::wild_for(run, lost) {
                Some((k, name)) => crate::engine::place_stray(run, &mut rng, &k, &name, &mut used),
                None => false,
            },
            k => crate::engine::place_room_kind(run, &mut rng, k, false, &mut used),
        };
        if placed {
            run.floor_twist = Some(kind.into());
            return;
        }
    }
}

fn has_situation(run: &Run, what: &str) -> bool {
    run.monsters.iter().any(|m| m.situation.as_deref() == Some(what))
}

/// Three monkeys asleep round the down stairs (within two tiles).
fn place_den(run: &mut Run) -> bool {
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
    if chosen.len() < 3 {
        return false;
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
    true
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

fn place_lock(run: &mut Run) -> bool {
    let lock = lock_tiles(run);
    if lock.is_empty() {
        return false;
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
    true
}

/// A captive chained on the down stairs.
fn place_captive(run: &mut Run) -> bool {
    let down = run.floor.stairs_down;
    run.monsters.retain(|m| m.pos != down || m.ally);
    let id = run.new_id();
    let mut m = Monster::spawn(id, "captive", down, run.depth);
    m.situation = Some("captive".into());
    m.neutral = true;
    m.awake = false;
    run.monsters.push(m);
    true
}

/// A shrine a few steps down the path from the entrance; three wraiths already hunting.
fn place_hunger(run: &mut Run) -> bool {
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
    let Some(p) = cands.first().copied() else { return false };
    run.floor.map.set(p, Tile::Shrine);
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
    true
}

/// `on_see: den | lock | captive | hunger` — the situation is in view (or, for the hunger,
/// biting: the floor unlit).
pub fn sees(run: &Run, what: &str) -> bool {
    let map = &run.floor.map;
    match what {
        "den" | "lock" => run.monsters.iter().any(|m| m.hp > 0 && m.hostile() && m.situation.as_deref() == Some(what) && map.is_visible(m.pos)),
        "captive" => run.monsters.iter().any(|m| m.hp > 0 && m.neutral && m.situation.as_deref() == Some("captive") && (map.is_visible(m.pos) || run.freeing == Some(m.id))),
        "hunger" => hunger_floor(run) && !lit(run),
        _ => false,
    }
}

pub fn hunger_floor(run: &Run) -> bool {
    at(run) == Some("hunger") && run.tile_pos(Tile::Shrine).is_some()
}

/// Light on the hunger floor: the shrine lit, or a lantern in the pack.
pub fn lit(run: &Run) -> bool {
    run.lit || run.hero.inv.iter().any(|i| i.kind == "lantern")
}

/// The situations in view become facts and count for the run (once per floor).
pub fn seen(run: &mut Run, cx: &mut Ctx) {
    let Some(what) = at(run).map(str::to_string) else { return };
    let what = what.as_str();
    // The den is met on sight; the lock's reek and the captive's cry carry to the entrance.
    let met = match what {
        "hunger" => false, // met by its first bite
        "den" => run.monsters.iter().any(|m| m.situation.as_deref() == Some("den") && m.hp > 0 && run.floor.map.is_visible(m.pos)),
        _ => has_situation(run, what),
    };
    if met && run.met_situation(what) {
        // Cut 24 §2: each drawn from its pool (`chronicle::variant`).
        let text = crate::chronicle::variant(run, if what == "den" || what == "lock" { what } else { "captive" });
        note(run, cx, text);
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
    // (the field as far as the sense reaches: a farther lock reads its distance or −1, out of it either way)
    crate::turn::hero_dist_within(run, SENSE_STEPS);
    let hd = &run.hero_dist;
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
    if at(run) != Some("den") {
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
    // Cut 19 §5: once the lineage has lost to a den, it sleeps through most runs.
    if !den_pounces(run) {
        return;
    }
    run.met_situation("den");
    run.den_wakes += 1;
    learn(run, cx, "den".into());
    let text = crate::chronicle::variant(run, "den_wakes");
    note(run, cx, text);
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
        // Cut 20 §1: one theft per run — once something is stolen, the rest bolt empty-handed.
        if !run.stolen.is_empty() {
            run.monsters[mi].fear = crate::ai::THIEF_FLEE;
            // The den was not answered — it simply had nothing left to take.
            run.den_bolted = true;
        } else if run.monsters[mi].pos.adjacent(hp) {
            snatch(run, cx, mi);
        }
    }
    // The den has woken: the run's later dens are thinned too.
    run.den_thin = true;
}

/// Cut 19 §5 (rater AB: "the reel repeats `A thief snatched the teleport scroll`"): the den
/// pounces on every floor of a lineage that has never lost to one; after (`Run.den_thin`), on
/// 1 in 3 — drawn from the run's seed and the floor, so a replay of the run agrees. Cut 20 §1
/// (AC, AD: "the den wakes on nearly every run"): thinned for every lineage once its first den
/// has woken (`LineageState::den_wakes`), and for the rest of a run whose den has.
pub fn den_pounces(run: &Run) -> bool {
    !run.den_thin || crate::rng::splitmix(run.seed ^ ((run.depth as u64) << 32) ^ 0xDE_7E1F).is_multiple_of(3)
}

/// A den thief takes one thing, then runs. Cut 22 §2: what the run found first, a
/// vault-brought item next, a coin pile's worth, a packed supply only when the pack holds
/// nothing else — never what he wears (Cut 25 §5; `ai::thief_pick`; was: a brought item, else the
/// pack's first — the heal just bought).
fn snatch(run: &mut Run, cx: &mut Ctx, mi: usize) {
    let Some(take) = crate::ai::thief_pick(run, false, false) else { return };
    let (it, amount) = crate::ai::thief_take(run, take);
    let (_, _, label) = crate::item::describe(&it, cx.facts, cx.flavours);
    run.den_stolen.push(it.id);
    run.den_snatches += 1;
    crate::provenance::stolen(run, cx, &it.kind, "monkey", true, &label);
    let coins = it.kind == "gold";
    let id = run.monsters[mi].id;
    run.stolen_ids.push(it.id);
    run.stolen_kinds.push((it.id, it.kind.clone(), amount.unwrap_or(0)));
    run.monsters[mi].stolen = Some(it);
    run.monsters[mi].fleeing = true;
    cx.events.push(Ev::Steal { t: run.turn, id, item: label.clone(), amount });
    run.stolen.push((run.turn, label.clone()));
    run.stolen_labels.push((run.stolen_ids.last().copied().unwrap_or(0), label.clone()));
    if coins {
        note(run, cx, format!("A thief snatched ${}.", amount.unwrap_or(0)));
    } else {
        note(run, cx, format!("A thief snatched the {label}."));
    }
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
        let text = crate::chronicle::variant(run, "hunger");
        note(run, cx, text);
        learn(run, cx, "hunger".into());
        crate::sifter::on_hurt(run, "hunger", Some("hunger"));
    }
    // QA on 1a2a4a9: the bite names what it took (`hunger −1 max`, an `Ev::MaxHp`).
    // Cut 25 §3: with no foe in view, the stretch is a drain (`starving`, once).
    crate::turn::drain_mark(run, cx, "hunger");
    callout(run, cx, "hunger −1 max");
    cx.events.push(Ev::MaxHp { t: run.turn, id: crate::engine::HERO_ID, max: run.hero.max_hp, delta: -1, cause: "hunger".into() });
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
    // (the den's line is fixed: the report's thefts are read against it)
    let text = match what {
        "den" => "Nothing lost to the den.".to_string(),
        "lock" => crate::chronicle::variant(run, "pass_lock"),
        "captive" => crate::chronicle::variant(run, "pass_captive"),
        _ => crate::chronicle::variant(run, "pass_hunger"),
    };
    note(run, cx, text);
}

/// The floor is being left: the den and the lock are judged now.
pub fn on_leave_floor(run: &mut Run, cx: &mut Ctx) {
    let met = |run: &Run, what: &str| run.situations.iter().any(|(_, s)| s == what);
    match at(run).map(str::to_string).as_deref() {
        Some("den") if met(run, "den") => {
            // (stolen coins got back are in the carry: no longer out — Cut 22 §2)
            let has = |run: &Run, id: u32| run.hero.inv.iter().chain(run.hero.weapon.iter()).chain(run.hero.armour.iter()).any(|i| i.id == id) || !run.stolen_ids.contains(&id);
            // QA on a946e04 (qaS: `A den of thieves. Nothing lost to the den.` in the report
            // that said `stolen red potion?`): nothing is lost while any theft of the run is
            // still out — a thief outside the den (a monkey on the den's floor, or on an
            // earlier one) counts too (`Run.stolen_ids`: what no run got back).
            if !run.den_bolted && run.den_stolen.iter().all(|id| has(run, *id)) && run.stolen_ids.is_empty() {
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
    run.den_bolted = false;
    run.lock_tiles.clear();
    run.lit = false;
}

/// Cut 24 §2: a floor's arrival event rolls on this share of the non-boss floors from D2.
pub const OMEN_PCT: u32 = 33;

/// Cut 24 §2 (AK: "the same floors, the same lines"): one new floor event per biome, met on
/// arrival (the floor's own side stream; a boss's floor has none) and said in one line drawn
/// from its pool (`chronicle::variant`): the Warrens' hoard, the Foundry's ingot, the Deep's
/// drowned purse and the Sanctum's offerings (a gold pile beside him), the Burrows' dead delver
/// (a flask beside him), the Fens' spring (a fifth of his HP back), the Crypt's bell (the alert
/// +1). Nothing below the Fens heals: the walls from D23 are their counters' (FULL−D28 kitted
/// must not pass the Queen on a spring).
pub fn omen(run: &mut Run, cx: &mut Ctx) {
    use crate::descent::Biome;
    let depth = run.depth;
    if depth < 2 || run.route.boss(depth).is_some() {
        return;
    }
    let mut rng = run.rng.side(crate::rng::hash_str("omen") ^ depth as u64);
    if !rng.chance(OMEN_PCT) {
        return;
    }
    let biome = run.biome();
    let kind = format!("omen:{}", biome.name());
    if crate::chronicle::pool(&kind).is_empty() {
        return;
    }
    let at = run.hero.pos;
    match biome {
        Biome::Warrens | Biome::Foundry | Biome::Deep | Biome::Sanctum => {
            let mut it = crate::item::Item::new(run.new_item_id(), "gold");
            it.amount = ((rng.range(6, 12) * depth as i32 + crate::engine::GOLD_DIVISOR / 2) / crate::engine::GOLD_DIVISOR).max(1);
            crate::turn::drop_near(run, at, it);
        }
        Biome::Burrows => {
            let k = if rng.chance(60) { "heal" } else { "strength" };
            let it = crate::item::Item::new(run.new_item_id(), k);
            crate::turn::drop_near(run, at, it);
        }
        Biome::Crypt => {
            run.alert = (run.alert + 1).min(8);
        }
        _ => {
            let h = &mut run.hero;
            h.hp = (h.hp + (h.max_hp / 5).max(1)).min(h.max_hp);
            cx.events.push(Ev::Hurt { t: run.turn, id: crate::engine::HERO_ID, dmg: 0, hp: run.hero.hp, cause: "rest".into() });
        }
    }
    let text = crate::chronicle::variant(run, &kind);
    note(run, cx, text);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Game;
    use crate::rng::Rng;

    /// Cut 12 §4: from D3 every floor holds exactly one situation, never the previous
    /// floor's kind; each band shows its flagship once (the den in D3–5, the lock in D6–7,
    /// the captive in D9–11, the hunger on D12); a boss's floor keeps to the room kinds; D1–2
    /// carry no twist word; over 8 floors on 30 seeds the sequence holds ≥ 4 kinds.
    #[test]
    fn every_floor_from_d3_rolls_one_situation() {
        let mut kinds_seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for seed in 1..=30u64 {
            let mut g = Game::new(seed);
            g.start_run(Some(seed));
            let mut seq: Vec<Option<String>> = Vec::new();
            for d in 1..=13u32 {
                g.descend_to(d);
                let run = g.run.as_ref().unwrap();
                let twist = run.floor_twist.clone();
                if d <= 2 {
                    assert!(twist.is_none(), "seed {seed} D{d}: the doorstep has no twist word");
                } else if crate::descent::boss_for(d).is_some() {
                    assert!(twist.as_deref().is_none_or(|t| ["nest", "vault", "shrine"].contains(&t)), "seed {seed} D{d}: a boss floor rolled {twist:?}");
                } else {
                    let t = twist.clone().unwrap_or_else(|| panic!("seed {seed} D{d}: no situation"));
                    assert!(TWISTS.contains(&t.as_str()), "{t}");
                    // The word matches the floor.
                    let band_count = run.monsters.iter().filter(|m| m.situation.as_deref() == Some(t.as_str())).count();
                    match t.as_str() {
                        "den" => assert_eq!(band_count, 3, "seed {seed} D{d} den"),
                        "lock" => assert!(band_count >= 1, "seed {seed} D{d} lock"),
                        "captive" => assert!(run.monsters.iter().any(|m| m.situation.as_deref() == Some("captive") && m.neutral && m.pos == run.floor.stairs_down), "seed {seed} D{d} captive"),
                        "hunger" => assert!(run.tile_pos(Tile::Shrine).is_some() && band_count == 3, "seed {seed} D{d} hunger"),
                        "nest" => assert!(run.tile_pos(Tile::Nest).is_some() && run.monsters.iter().any(|m| m.nest), "seed {seed} D{d} nest"),
                        "vault" => assert!(run.tile_pos(Tile::Vault).is_some() && run.vault_cage.len() == 3, "seed {seed} D{d} vault"),
                        "shrine" => assert!(run.tile_pos(Tile::Shrine).is_some(), "seed {seed} D{d} shrine"),
                        "stray" => assert!(run.monsters.iter().any(|m| m.stray), "seed {seed} D{d} stray"),
                        _ => unreachable!(),
                    }
                    // One situation: no other band kind on the floor, and a den or a lock on
                    // one floor only.
                    for other in BAND_KINDS {
                        if other != t {
                            assert!(!run.monsters.iter().any(|m| m.situation.as_deref() == Some(other)), "seed {seed} D{d}: {t} and {other}");
                        }
                    }
                    if let (Some(prev), Some(cur)) = (seq.last().cloned().flatten(), twist.clone()) {
                        assert_ne!(prev, cur, "seed {seed} D{d}: the same twist twice running");
                    }
                    kinds_seen.insert(t);
                }
                seq.push(twist);
                if g.run.as_ref().unwrap().over.is_some() {
                    break;
                }
            }
            let has = |k: &str, from: u32, to: u32| (from..=to).any(|d| seq.get(d as usize - 1).cloned().flatten().as_deref() == Some(k));
            assert!(has("den", 3, 5), "seed {seed}: no den in D3–5: {seq:?}");
            assert!(has("lock", 6, 7), "seed {seed}: no lock in D6–7: {seq:?}");
            assert!(has("captive", 9, 11), "seed {seed}: no captive in D9–11: {seq:?}");
            assert_eq!(seq[11].as_deref(), Some("hunger"), "seed {seed}: D12 is the hunger's: {seq:?}");
            let distinct: std::collections::BTreeSet<&str> = seq[2..10].iter().filter_map(|t| t.as_deref()).collect();
            assert!(distinct.len() >= 4, "seed {seed}: {distinct:?} over D3–10");
        }
        assert!(kinds_seen.len() >= 7, "{kinds_seen:?}");
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
        assert_eq!(depth_of("den"), Some(3));
        assert_eq!(depth_of("lock"), Some(6));
        assert_eq!(depth_of("captive"), Some(9));
        assert_eq!(depth_of("hunger"), Some(12));
        for (k, d) in SITUATION_DEPTHS {
            assert_eq!(band(d).unwrap().flagship, Some(k), "{k}'s band starts at its Cut 7 floor");
            assert_eq!(band(d).unwrap().first, d);
        }
        // A trial can choose the floor's kind.
        let mut g = Game::new(3);
        g.start_run(Some(3));
        g.descend_to_twist(4, "lock");
        assert_eq!(g.run.as_ref().unwrap().floor_twist.as_deref(), Some("lock"));
        assert_eq!(at(g.run.as_ref().unwrap()), Some("lock"));
        let _ = Rng::new(1);
    }
}
