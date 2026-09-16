//! Floor generation: rooms+corridors (Warrens, Crypt) and cellular caves with water (Fens).
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

pub fn generate(rng: &mut Rng, biome: Biome, depth: u32) -> Floor {
    for attempt in 0..8 {
        let f = if biome.is_cave() { gen_cave(rng, depth) } else { gen_rooms(rng, biome, depth) };
        if let Some(mut f) = f {
            f.vision = biome.vision();
            let d = f.map.bfs(f.stairs_up, false, &|_| false);
            if d[f.map.idx(f.stairs_down)] > 0 || attempt == 7 {
                return f;
            }
        }
    }
    unreachable!("generator retries exhausted")
}

fn gen_rooms(rng: &mut Rng, biome: Biome, depth: u32) -> Option<Floor> {
    let w = MAX_W;
    let h = MAX_H;
    let mut map = Map::new(w, h, Tile::Wall);
    let mut rooms: Vec<Rect> = Vec::new();
    let target = 16 + (depth as usize / 4).min(3);
    for _ in 0..120 {
        if rooms.len() >= target {
            break;
        }
        let rw = rng.range(3, 7);
        let rh = rng.range(3, 6);
        let x = rng.range(1, w - rw - 1);
        let y = rng.range(1, h - rh - 1);
        let r = Rect { x, y, w: rw, h: rh };
        if rooms.iter().any(|o| o.overlaps(&r)) {
            continue;
        }
        rooms.push(r);
    }
    if rooms.len() < 3 {
        return None;
    }
    let mut room_mask = vec![false; (w * h) as usize];
    for r in &rooms {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                let p = Pos::new(x, y);
                map.set(p, Tile::Floor);
                let i = map.idx(p);
                room_mask[i] = true;
            }
        }
    }
    // Connect each room to the previous (L-corridors, 4-connected), plus a couple of loops.
    let mut order: Vec<usize> = (0..rooms.len()).collect();
    rng.shuffle(&mut order);
    for k in 1..order.len() {
        let a = rooms[order[k - 1]].centre();
        let b = rooms[order[k]].centre();
        carve_l(&mut map, &room_mask, rng, a, b);
    }
    for _ in 0..3 {
        let a = rooms[rng.below(rooms.len() as u32) as usize].centre();
        let b = rooms[rng.below(rooms.len() as u32) as usize].centre();
        if a != b {
            carve_l(&mut map, &room_mask, rng, a, b);
        }
    }
    // Crypt: a few chasm pits inside larger rooms.
    if biome == Biome::Crypt {
        for r in &rooms {
            if r.w >= 5 && r.h >= 4 && rng.chance(50) {
                let p = Pos::new(r.x + rng.range(1, r.w - 2), r.y + rng.range(1, r.h - 2));
                map.set(p, Tile::Chasm);
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

fn carve_l(map: &mut Map, room_mask: &[bool], rng: &mut Rng, a: Pos, b: Pos) {
    let horiz_first = rng.chance(50);
    let mid = if horiz_first { Pos::new(b.x, a.y) } else { Pos::new(a.x, b.y) };
    carve_line(map, room_mask, a, mid);
    carve_line(map, room_mask, mid, b);
}

fn carve_line(map: &mut Map, room_mask: &[bool], a: Pos, b: Pos) {
    let mut p = a;
    loop {
        let i = map.idx(p);
        if map.tiles[i] == Tile::Wall {
            // Door where the corridor enters a room boundary.
            let touches_room = DIRS4.iter().any(|d| {
                let q = p.step(*d);
                map.in_bounds(q) && room_mask[map.idx(q)]
            });
            map.tiles[i] = if touches_room { Tile::Door } else { Tile::Floor };
        }
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

fn gen_cave(rng: &mut Rng, depth: u32) -> Option<Floor> {
    let w = MAX_W;
    let h = MAX_H;
    let n = (w * h) as usize;
    let mut cells = vec![false; n]; // true = open
    for (i, c) in cells.iter_mut().enumerate() {
        let p = Pos::new(i as i32 % w, i as i32 / w);
        let edge = p.x == 0 || p.y == 0 || p.x == w - 1 || p.y == h - 1;
        *c = !edge && rng.chance(55);
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
    #[test]
    fn generation_is_deterministic() {
        let a = generate(&mut Rng::new(5), Biome::Warrens, 2);
        let b = generate(&mut Rng::new(5), Biome::Warrens, 2);
        assert_eq!(a, b);
    }
}
