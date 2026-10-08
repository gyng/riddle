//! Floor generation: rooms+corridors (Warrens, Burrows, Crypt, Foundry, Sanctum) and cellular caves with water (Fens, Deep).
//! Cut 109: each floor draws a layout from its biome's pool (room sizes, shaped rooms — round, cross, ell,
//! pillared — winding corridors, a great hall) and each cave its own density, chambers and mere.
//! 32×32 (Cut 2 §1), connectivity stairs_up → stairs_down guaranteed.
use crate::descent::Biome;
use crate::geom::{Pos, DIRS4, DIRS8};
use crate::rng::Rng;
use crate::tiles::{Map, Tile};
use serde::{Deserialize, Serialize};

pub const MAX_W: i32 = 32;
pub const MAX_H: i32 = 32;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub fn centre(&self) -> Pos {
        Pos::new(self.x + self.w / 2, self.y + self.h / 2)
    }
    pub fn contains(&self, p: Pos) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.w && p.y < self.y + self.h
    }
    fn overlaps(&self, o: &Rect) -> bool {
        self.x - 1 < o.x + o.w && o.x - 1 < self.x + self.w && self.y - 1 < o.y + o.h && o.y - 1 < self.y + self.h
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Floor {
    pub map: Map,
    pub stairs_up: Pos,
    pub stairs_down: Pos,
    pub rooms: Vec<Rect>,
    /// Cut 3: the hero's sight radius on this floor (the Deep is dark: 4; lit biomes 7).
    #[serde(default = "default_vision")]
    pub vision: i32,
}

fn default_vision() -> i32 {
    crate::descent::VISION_LIT
}

impl Floor {
    /// Passable floor tiles (not stairs), for placement.
    pub fn open_tiles(&self) -> Vec<Pos> {
        (0..self.map.tiles.len())
            .filter(|i| matches!(self.map.tiles[*i], Tile::Floor))
            .map(|i| self.map.pos(i))
            .collect()
    }
    pub fn water_tiles(&self) -> Vec<Pos> {
        (0..self.map.tiles.len())
            .filter(|i| self.map.tiles[*i] == Tile::Water)
            .map(|i| self.map.pos(i))
            .collect()
    }
}

/// The floor's plan comes from its own stream: one draw from the caller's (Cut 109), so a change to the
/// generator never moves the rolls that follow it (combat, loot, situations).
pub const GEN_TAG: u64 = 0x6765_6e5f_706c_616e;

pub fn generate(caller: &mut Rng, biome: Biome, depth: u32) -> Floor {
    let rng = &mut Rng::derive(caller.next_u64(), GEN_TAG);
    for attempt in 0..8 {
        let f = if biome.is_cave() { gen_cave(rng, biome, depth) } else { gen_rooms(rng, biome, depth) };
        if let Some(mut f) = f {
            f.vision = biome.vision();
            let d = f.map.bfs(f.stairs_up, false, &|_| false);
            if d[f.map.idx(f.stairs_down)] > 0 || attempt == 7 {
                // Cut 109: a shaped room can leave a cell touching the rest only at a corner; every open tile is
                // reachable from the stairs (nothing spawns, drops or waits where no path goes)
                let mut cut = false;
                for (i, di) in d.iter().enumerate() {
                    if *di < 0 && f.map.tiles[i].passable() {
                        f.map.tiles[i] = Tile::Wall;
                        cut = true;
                    }
                }
                if cut {
                    let mask: Vec<bool> = (0..f.map.tiles.len()).map(|i| f.rooms.iter().any(|r| r.contains(f.map.pos(i))) && f.map.tiles[i].passable()).collect();
                    f.map.compute_corridors(if f.rooms.is_empty() { &[] } else { &mask });
                }
                return f;
            }
        }
    }
    unreachable!("generator retries exhausted")
}

/// Cut 109: a room's shape inside its bounding box (`Rect` stays the box: `room_ref`, the lock and
/// the situations read boxes; every shape keeps the box's centre open, so stairs and corridors
/// anchored there still land on floor).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Shape {
    Rect,
    /// an ellipse filling the box (≥ 5×4)
    Round,
    /// a plus: the box's four corners cut by a third (≥ 5×5)
    Cross,
    /// one corner quadrant cut away (≥ 4×4)
    Ell,
    /// a hall with a grid of single-tile pillars, never on its rim nor by its centre (≥ 5×5)
    Pillars,
}

/// Cut 109: a floor's layout — drawn per floor from its biome's pool, so two floors of one band
/// differ in plan, not only in tiles.
struct Layout {
    /// room width and height ranges (`rng.range`: upper bound exclusive)
    w: (i32, i32),
    h: (i32, i32),
    /// shape weights (summing to 100)
    shapes: &'static [(Shape, u32)],
    /// rooms aimed for, relative to the classic 16 + depth/4
    target: i32,
    /// corridors bend through a random waypoint (the Burrows' tunnels)
    wind: bool,
    /// a great hall first, in the middle of the floor (the Sanctum's nave)
    hall: bool,
    /// extra loop corridors
    loops: u32,
}

const CLASSIC: Layout = Layout { w: (3, 7), h: (3, 6), shapes: &[(Shape::Rect, 70), (Shape::Ell, 20), (Shape::Pillars, 10)], target: 0, wind: false, hall: false, loops: 3 };
const HALLS: Layout = Layout { w: (4, 9), h: (4, 8), shapes: &[(Shape::Rect, 45), (Shape::Pillars, 30), (Shape::Cross, 15), (Shape::Ell, 10)], target: -6, wind: false, hall: false, loops: 3 };
const TUNNELS: Layout = Layout { w: (4, 8), h: (4, 7), shapes: &[(Shape::Round, 60), (Shape::Ell, 20), (Shape::Rect, 20)], target: -5, wind: true, hall: false, loops: 2 };
const CRYPT: Layout = Layout { w: (5, 9), h: (4, 8), shapes: &[(Shape::Cross, 35), (Shape::Pillars, 35), (Shape::Rect, 30)], target: -5, wind: false, hall: false, loops: 3 };
const CATACOMB: Layout = Layout { w: (3, 6), h: (3, 5), shapes: &[(Shape::Rect, 80), (Shape::Ell, 20)], target: 3, wind: false, hall: false, loops: 6 };
const WORKS: Layout = Layout { w: (5, 10), h: (4, 8), shapes: &[(Shape::Pillars, 40), (Shape::Rect, 50), (Shape::Ell, 10)], target: -7, wind: false, hall: false, loops: 2 };
const SANCTUM: Layout = Layout { w: (4, 8), h: (4, 7), shapes: &[(Shape::Round, 35), (Shape::Cross, 35), (Shape::Rect, 30)], target: -5, wind: false, hall: true, loops: 3 };

fn layout_for(rng: &mut Rng, biome: Biome) -> &'static Layout {
    let roll = rng.below(100);
    match biome {
        Biome::Warrens => if roll < 70 { &CLASSIC } else { &HALLS },
        Biome::Burrows => if roll < 65 { &TUNNELS } else { &CLASSIC },
        Biome::Crypt => if roll < 65 { &CRYPT } else { &CATACOMB },
        Biome::Foundry => if roll < 70 { &WORKS } else { &HALLS },
        Biome::Sanctum => if roll < 70 { &SANCTUM } else { &CRYPT },
        _ => &CLASSIC,
    }
}

fn pick_shape(rng: &mut Rng, l: &Layout, r: &Rect) -> Shape {
    let mut roll = rng.below(100);
    let mut shape = Shape::Rect;
    for (s, wgt) in l.shapes {
        if roll < *wgt {
            shape = *s;
            break;
        }
        roll -= wgt;
    }
    let fits = match shape {
        Shape::Rect => true,
        Shape::Round => r.w >= 5 && r.h >= 4,
        Shape::Cross | Shape::Pillars => r.w >= 5 && r.h >= 5,
        Shape::Ell => r.w >= 4 && r.h >= 4,
    };
    if fits { shape } else { Shape::Rect }
}

/// The open cells of a shaped room (`Some(true)` floor, `Some(false)` a pillar, `None` cut away).
fn shape_cell(shape: Shape, r: &Rect, corner: u32, p: Pos) -> Option<bool> {
    let c = r.centre();
    let (dx, dy) = (p.x - r.x, p.y - r.y);
    match shape {
        Shape::Rect => Some(true),
        Shape::Round => {
            let fx = (p.x as f32 - (r.x as f32 + (r.w - 1) as f32 / 2.0)) / (r.w as f32 / 2.0);
            let fy = (p.y as f32 - (r.y as f32 + (r.h - 1) as f32 / 2.0)) / (r.h as f32 / 2.0);
            (fx * fx + fy * fy <= 1.2 || p == c).then_some(true)
        }
        Shape::Cross => {
            let (cw, ch) = (r.w / 3, r.h / 3);
            let side_x = dx < cw || r.w - 1 - dx < cw;
            let side_y = dy < ch || r.h - 1 - dy < ch;
            (!(side_x && side_y)).then_some(true)
        }
        Shape::Ell => {
            let right = corner & 1 == 1;
            let down = corner & 2 == 2;
            let cut_x = if right { p.x > c.x } else { p.x < c.x };
            let cut_y = if down { p.y > c.y } else { p.y < c.y };
            (!(cut_x && cut_y)).then_some(true)
        }
        Shape::Pillars => {
            let rim = dx == 0 || dy == 0 || dx == r.w - 1 || dy == r.h - 1;
            let pillar = !rim && dx % 2 == 1 && dy % 2 == 1 && p.cheb(c) > 1 && dx < r.w - 2 && dy < r.h - 2;
            Some(!pillar)
        }
    }
}

fn gen_rooms(rng: &mut Rng, biome: Biome, depth: u32) -> Option<Floor> {
    let w = MAX_W;
    let h = MAX_H;
    let mut map = Map::new(w, h, Tile::Wall);
    let layout = layout_for(rng, biome);
    let mut rooms: Vec<Rect> = Vec::new();
    let mut shapes: Vec<Shape> = Vec::new();
    let target = (16 + (depth as i32 / 4).min(3) + layout.target).max(5) as usize;
    if layout.hall {
        // the nave: a pillared ellipse-cornered hall across the middle of the floor
        let (hw, hh) = (9 + rng.range(0, 3), 7 + rng.range(0, 2));
        rooms.push(Rect { x: (w - hw) / 2 + rng.range(-3, 4), y: (h - hh) / 2 + rng.range(-3, 4), w: hw, h: hh });
        shapes.push(Shape::Pillars);
    }
    for _ in 0..160 {
        if rooms.len() >= target {
            break;
        }
        let rw = rng.range(layout.w.0, layout.w.1);
        let rh = rng.range(layout.h.0, layout.h.1);
        let x = rng.range(1, w - rw - 1);
        let y = rng.range(1, h - rh - 1);
        let r = Rect { x, y, w: rw, h: rh };
        if rooms.iter().any(|o| o.overlaps(&r)) {
            continue;
        }
        shapes.push(pick_shape(rng, layout, &r));
        rooms.push(r);
    }
    if rooms.len() < 3 {
        return None;
    }
    let n = (w * h) as usize;
    let mut room_mask = vec![false; n];
    let mut boxed = vec![false; n];
    for (r, shape) in rooms.iter().zip(&shapes) {
        let corner = rng.below(4);
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                let p = Pos::new(x, y);
                let i = map.idx(p);
                boxed[i] = true;
                if let Some(true) = shape_cell(*shape, r, corner, p) {
                    map.set(p, Tile::Floor);
                    room_mask[i] = true;
                }
            }
        }
    }
    let carve = Carve { room_mask: &room_mask, boxed: &boxed };
    // Connect each room to the previous, plus loops.
    let mut order: Vec<usize> = (0..rooms.len()).collect();
    rng.shuffle(&mut order);
    for k in 1..order.len() {
        let a = rooms[order[k - 1]].centre();
        let b = rooms[order[k]].centre();
        carve.path(&mut map, rng, a, b, layout.wind);
    }
    for _ in 0..layout.loops {
        let a = rooms[rng.below(rooms.len() as u32) as usize].centre();
        let b = rooms[rng.below(rooms.len() as u32) as usize].centre();
        if a != b {
            carve.path(&mut map, rng, a, b, layout.wind);
        }
    }
    // Crypt: a few chasm pits inside larger plain rooms.
    if biome == Biome::Crypt {
        for (r, shape) in rooms.iter().zip(&shapes) {
            if *shape == Shape::Rect && r.w >= 5 && r.h >= 4 && rng.chance(50) {
                let p = Pos::new(r.x + rng.range(1, r.w - 2), r.y + rng.range(1, r.h - 2));
                if p != r.centre() {
                    map.set(p, Tile::Chasm);
                }
            }
        }
    }
    // Stairs: up in a random room, down in the room farthest by path.
    let up_room = order[0];
    let stairs_up = rooms[up_room].centre();
    map.set(stairs_up, Tile::StairsUp);
    let dist = map.bfs(stairs_up, false, &|_| false);
    let mut best = (-1, up_room);
    for (i, r) in rooms.iter().enumerate() {
        let c = r.centre();
        let d = dist[map.idx(c)];
        if d > best.0 {
            best = (d, i);
        }
    }
    if best.0 <= 0 {
        return None;
    }
    let mut stairs_down = rooms[best.1].centre();
    if stairs_down == stairs_up {
        stairs_down = Pos::new(stairs_down.x + 1, stairs_down.y);
    }
    map.set(stairs_down, Tile::StairsDown);
    map.compute_corridors(&room_mask);
    Some(Floor { map, stairs_up, stairs_down, rooms, vision: biome.vision() })
}

/// Corridor carving: L-shaped, or bent through a waypoint. A wall inside
/// a room's box (a pillar, a cut corner) opens as floor; a door only where the corridor meets a
/// room from outside its box.
struct Carve<'a> {
    room_mask: &'a [bool],
    boxed: &'a [bool],
}

impl Carve<'_> {
    fn path(&self, map: &mut Map, rng: &mut Rng, a: Pos, b: Pos, wind: bool) {
        if wind {
            let lo = |u: i32, v: i32| u.min(v) - 3;
            let hi = |u: i32, v: i32| u.max(v) + 4;
            let wx = rng.range(lo(a.x, b.x), hi(a.x, b.x)).clamp(1, map.w - 2);
            let wy = rng.range(lo(a.y, b.y), hi(a.y, b.y)).clamp(1, map.h - 2);
            let way = Pos::new(wx, wy);
            self.l(map, rng, a, way);
            self.l(map, rng, way, b);
        } else {
            self.l(map, rng, a, b);
        }
    }
    fn l(&self, map: &mut Map, rng: &mut Rng, a: Pos, b: Pos) {
        let horiz_first = rng.chance(50);
        let mid = if horiz_first { Pos::new(b.x, a.y) } else { Pos::new(a.x, b.y) };
        self.line(map, a, mid);
        self.line(map, mid, b);
    }
    fn open(&self, map: &mut Map, p: Pos) {
        if !map.in_bounds(p) || p.x < 1 || p.y < 1 || p.x > map.w - 2 || p.y > map.h - 2 {
            return;
        }
        let i = map.idx(p);
        if map.tiles[i] != Tile::Wall {
            return;
        }
        let touches_room = DIRS4.iter().any(|d| {
            let q = p.step(*d);
            map.in_bounds(q) && self.room_mask[map.idx(q)]
        });
        map.tiles[i] = if touches_room && !self.boxed[i] { Tile::Door } else { Tile::Floor };
    }
    fn line(&self, map: &mut Map, a: Pos, b: Pos) {
        let mut p = a;
        loop {
            self.open(map, p);
            if p == b {
                break;
            }
            if p.x != b.x {
                p.x += (b.x - p.x).signum();
            } else {
                p.y += (b.y - p.y).signum();
            }
        }
    }
}

fn gen_cave(rng: &mut Rng, biome: Biome, depth: u32) -> Option<Floor> {
    let w = MAX_W;
    let h = MAX_H;
    let n = (w * h) as usize;
    let mut cells = vec![false; n]; // true = open
    // Cut 109: each cave floor its own density (tight passages · open grottoes) and chamber count
    let fill = 51 + rng.below(8);
    let chambers = rng.below(4);
    let lake = biome == Biome::Fens && rng.chance(40);
    for (i, c) in cells.iter_mut().enumerate() {
        let p = Pos::new(i as i32 % w, i as i32 / w);
        let edge = p.x == 0 || p.y == 0 || p.x == w - 1 || p.y == h - 1;
        *c = !edge && rng.chance(fill);
    }
    // Cut 109: one to three open chambers before the smoothing (big caverns among the passages)
    for _ in 0..chambers {
        let (cx, cy) = (rng.range(5, w - 5), rng.range(5, h - 5));
        let (rx, ry) = (rng.range(3, 6), rng.range(2, 5));
        for y in (cy - ry).max(1)..=(cy + ry).min(h - 2) {
            for x in (cx - rx).max(1)..=(cx + rx).min(w - 2) {
                let (fx, fy) = ((x - cx) as f32 / rx as f32, (y - cy) as f32 / ry as f32);
                if fx * fx + fy * fy <= 1.0 {
                    cells[(y * w + x) as usize] = true;
                }
            }
        }
    }
    for _ in 0..4 {
        let mut next = cells.clone();
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let p = Pos::new(x, y);
                let walls = DIRS8.iter().filter(|d| !cells[(p.step(**d).y * w + p.step(**d).x) as usize]).count();
                next[(y * w + x) as usize] = walls < 4 || (walls == 4 && cells[(y * w + x) as usize]);
            }
        }
        cells = next;
    }
    let mut map = Map::new(w, h, Tile::Wall);
    for (i, c) in cells.iter().enumerate() {
        if *c {
            map.tiles[i] = Tile::Floor;
        }
    }
    // Keep the largest component.
    let mut best: Vec<i32> = Vec::new();
    let mut best_count = 0;
    let mut visited = vec![false; n];
    for i in 0..n {
        if map.tiles[i] != Tile::Floor || visited[i] {
            continue;
        }
        let d = map.bfs(map.pos(i), false, &|_| false);
        let count = d.iter().filter(|x| **x >= 0).count();
        for (j, x) in d.iter().enumerate() {
            if *x >= 0 {
                visited[j] = true;
            }
        }
        if count > best_count {
            best_count = count;
            best = d;
        }
    }
    if best_count < 220 {
        return None;
    }
    for (i, d) in best.iter().enumerate() {
        if *d < 0 {
            map.tiles[i] = Tile::Wall;
        }
    }
    // Water: random-walk blobs, never on the stairs.
    let open: Vec<Pos> = (0..n).filter(|i| map.tiles[*i] == Tile::Floor).map(|i| map.pos(i)).collect();
    let blobs = 5 + (depth as i32 % 10 - 6).clamp(0, 3);
    for _ in 0..blobs {
        let mut p = *rng.pick(&open);
        for _ in 0..rng.range(8, 16) {
            if map.get(p) == Tile::Floor {
                map.set(p, Tile::Water);
            }
            let d = DIRS4[rng.below(4) as usize];
            let q = p.step(d);
            if map.in_bounds(q) && map.get(q) != Tile::Wall {
                p = q;
            }
        }
    }
    // Cut 109: the Fens' mere — one broad pool (wadeable: water never blocks a step)
    if lake {
        let c = *rng.pick(&open);
        let r = rng.range(3, 5);
        for y in c.y - r..=c.y + r {
            for x in c.x - r - 1..=c.x + r + 1 {
                let p = Pos::new(x, y);
                let (fx, fy) = ((x - c.x) as f32 / (r + 1) as f32, (y - c.y) as f32 / r as f32);
                if map.in_bounds(p) && map.get(p) == Tile::Floor && fx * fx + fy * fy <= 1.0 {
                    map.set(p, Tile::Water);
                }
            }
        }
    }
    let open: Vec<Pos> = (0..n).filter(|i| map.tiles[*i] == Tile::Floor).map(|i| map.pos(i)).collect();
    if open.len() < 40 {
        return None;
    }
    // Stairs far apart.
    let stairs_up = *rng.pick(&open);
    let dist = map.bfs(stairs_up, false, &|_| false);
    let mut far = stairs_up;
    let mut far_d = 0;
    for (i, d) in dist.iter().enumerate() {
        if *d > far_d && map.tiles[i] == Tile::Floor {
            far_d = *d;
            far = map.pos(i);
        }
    }
    if far_d < 12 {
        return None;
    }
    map.set(stairs_up, Tile::StairsUp);
    map.set(far, Tile::StairsDown);
    map.compute_corridors(&[]);
    Some(Floor { map, stairs_up, stairs_down: far, rooms: Vec::new(), vision: crate::descent::VISION_LIT })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rooms_connected_and_bounded() {
        for seed in 0..40u64 {
            let mut rng = Rng::new(seed);
            for (biome, depth) in [(Biome::Warrens, 1), (Biome::Crypt, 12), (Biome::Foundry, 17), (Biome::Sanctum, 28)] {
                let f = generate(&mut rng, biome, depth);
                assert!(f.map.w <= MAX_W && f.map.h <= MAX_H);
                let d = f.map.bfs(f.stairs_up, false, &|_| false);
                assert!(d[f.map.idx(f.stairs_down)] > 0, "seed {seed} {biome:?} unconnected");
                assert_eq!(f.map.get(f.stairs_up), Tile::StairsUp);
                assert_eq!(f.map.get(f.stairs_down), Tile::StairsDown);
                assert!(f.map.corridor.iter().any(|c| *c), "no corridors seed {seed}");
            }
        }
    }
    #[test]
    fn caves_have_water_and_connect() {
        for seed in 0..40u64 {
            let mut rng = Rng::new(seed + 100);
            let f = generate(&mut rng, Biome::Fens, 7);
            let d = f.map.bfs(f.stairs_up, false, &|_| false);
            assert!(d[f.map.idx(f.stairs_down)] > 0, "seed {seed} cave unconnected");
            assert!(!f.water_tiles().is_empty(), "seed {seed} no water");
            assert_eq!(f.vision, 7);
            let deep = generate(&mut rng, Biome::Deep, 22);
            assert_eq!(deep.vision, 4, "the Deep is dark");
            assert!(!deep.water_tiles().is_empty(), "seed {seed} deep has no water");
        }
    }
    /// Cut 109: every room biome yields shaped rooms and more than one plan across floors; every
    /// floor still connects, its stairs on floor at room centres.
    #[test]
    fn layouts_vary_and_connect() {
        for biome in [Biome::Warrens, Biome::Burrows, Biome::Crypt, Biome::Foundry, Biome::Sanctum, Biome::Fens, Biome::Deep] {
            let mut sigs = std::collections::HashSet::new();
            let mut shaped = 0;
            for seed in 0..60u64 {
                let mut rng = Rng::new(seed * 7 + 3);
                let f = generate(&mut rng, biome, 10);
                let d = f.map.bfs(f.stairs_up, false, &|_| false);
                assert!(d[f.map.idx(f.stairs_down)] > 0, "{biome:?} seed {seed} unconnected");
                let open = f.map.tiles.iter().filter(|t| t.passable()).count();
                sigs.insert((f.rooms.len(), open / 40));
                // a shaped room: a wall inside its box
                shaped += f.rooms.iter().filter(|r| (r.y..r.y + r.h).any(|y| (r.x..r.x + r.w).any(|x| f.map.get(Pos::new(x, y)) == Tile::Wall))).count();
                for r in &f.rooms {
                    assert!(f.map.passable(r.centre()), "{biome:?} seed {seed} room centre walled");
                }
            }
            assert!(sigs.len() >= 6, "{biome:?}: only {} distinct plans", sigs.len());
            if !biome.is_cave() {
                assert!(shaped >= 30, "{biome:?}: only {shaped} shaped rooms in 60 floors");
            }
        }
    }
    #[test]
    fn generation_is_deterministic() {
        let a = generate(&mut Rng::new(5), Biome::Warrens, 2);
        let b = generate(&mut Rng::new(5), Biome::Warrens, 2);
        assert_eq!(a, b);
    }
}
