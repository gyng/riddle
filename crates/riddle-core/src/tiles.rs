//! Tiles, overlays and the floor map with vision bookkeeping.
use crate::geom::{Pos, DIRS4, DIRS8};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Tile {
    Floor,
    Wall,
    Door,
    StairsDown,
    StairsUp,
    Water,
    Chasm,
    /// Cut 5 §4 situations (floor-standing props the client draws): an altar (`pray`), a
    /// three-item cage (choose one; `vault_open` once taken), a jackal den with a gold pile.
    Shrine,
    Vault,
    VaultOpen,
    Nest,
}

impl Tile {
    pub fn passable(self) -> bool {
        !matches!(self, Tile::Wall | Tile::Chasm)
    }
    pub fn blocks_sight(self) -> bool {
        matches!(self, Tile::Wall)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlayKind {
    Gas,
    Fire,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Overlay {
    pub x: i32,
    pub y: i32,
    pub k: OverlayKind,
    pub ttl: i32,
    /// Fire spreads to adjacent floor once; internal, not on the wire.
    #[serde(default, skip_serializing)]
    pub spread: bool,
}

pub const VISION: i32 = 7;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Map {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub seen: Vec<bool>,
    pub visible: Vec<bool>,
    /// Corridor tiles (narrow passages), for `in_corridor` and retreat preference.
    pub corridor: Vec<bool>,
}

impl Map {
    pub fn new(w: i32, h: i32, fill: Tile) -> Map {
        let n = (w * h) as usize;
        Map { w, h, tiles: vec![fill; n], seen: vec![false; n], visible: vec![false; n], corridor: vec![false; n] }
    }
    pub fn in_bounds(&self, p: Pos) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.w && p.y < self.h
    }
    pub fn idx(&self, p: Pos) -> usize {
        (p.y * self.w + p.x) as usize
    }
    pub fn pos(&self, i: usize) -> Pos {
        Pos::new(i as i32 % self.w, i as i32 / self.w)
    }
    pub fn get(&self, p: Pos) -> Tile {
        if self.in_bounds(p) {
            self.tiles[self.idx(p)]
        } else {
            Tile::Wall
        }
    }
    pub fn set(&mut self, p: Pos, t: Tile) {
        if self.in_bounds(p) {
            let i = self.idx(p);
            self.tiles[i] = t;
        }
    }
    pub fn passable(&self, p: Pos) -> bool {
        self.get(p).passable()
    }
    pub fn is_seen(&self, p: Pos) -> bool {
        self.in_bounds(p) && self.seen[self.idx(p)]
    }
    pub fn is_visible(&self, p: Pos) -> bool {
        self.in_bounds(p) && self.visible[self.idx(p)]
    }
    pub fn is_corridor(&self, p: Pos) -> bool {
        self.in_bounds(p) && self.corridor[self.idx(p)]
    }
    /// 8-connected step without cutting wall corners.
    pub fn can_step(&self, from: Pos, to: Pos) -> bool {
        if !self.passable(to) || from.cheb(to) != 1 {
            return false;
        }
        if from.x != to.x && from.y != to.y {
            let a = Pos::new(from.x, to.y);
            let b = Pos::new(to.x, from.y);
            if self.get(a) == Tile::Wall || self.get(b) == Tile::Wall {
                return false;
            }
        }
        true
    }
    /// Line of sight: no sight-blocking tile strictly between `a` and `b` along `geom::line`
    /// (walked in place; the hot loop of `update_vision`, ~225 calls per hero move).
    pub fn los(&self, a: Pos, b: Pos) -> bool {
        if a == b {
            return true;
        }
        let (mut x0, mut y0) = (a.x, a.y);
        let (x1, y1) = (b.x, b.y);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
            if x0 == x1 && y0 == y1 {
                return true;
            }
            if self.get(Pos::new(x0, y0)).blocks_sight() {
                return false;
            }
        }
    }
    pub fn count_passable(&self) -> usize {
        self.tiles.iter().filter(|t| t.passable()).count()
    }
    pub fn count_seen_passable(&self) -> usize {
        self.tiles.iter().zip(self.seen.iter()).filter(|(t, s)| t.passable() && **s).count()
    }
    /// Percent of passable tiles seen (0..=100).
    pub fn seen_pct(&self) -> i32 {
        let total = self.count_passable().max(1);
        (self.count_seen_passable() * 100 / total) as i32
    }
    /// Recompute `visible` from `from` with radius and line of sight; marks seen.
    pub fn update_vision(&mut self, from: Pos, radius: i32) {
        for v in self.visible.iter_mut() {
            *v = false;
        }
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let p = from.step((dx, dy));
                if !self.in_bounds(p) {
                    continue;
                }
                if self.los(from, p) {
                    let i = self.idx(p);
                    self.visible[i] = true;
                    self.seen[i] = true;
                }
            }
        }
    }
    pub fn reveal_all(&mut self) {
        for s in self.seen.iter_mut() {
            *s = true;
        }
    }
    /// Mark corridor tiles: passable non-room tiles with few passable neighbours.
    pub fn compute_corridors(&mut self, room_mask: &[bool]) {
        for i in 0..self.tiles.len() {
            let p = self.pos(i);
            if !self.tiles[i].passable() || (room_mask.len() == self.tiles.len() && room_mask[i]) {
                self.corridor[i] = false;
                continue;
            }
            let open8 = DIRS8.iter().filter(|d| self.passable(p.step(**d))).count();
            let open4 = DIRS4.iter().filter(|d| self.passable(p.step(**d))).count();
            self.corridor[i] = open4 <= 2 && open8 <= 4;
        }
    }
    /// BFS distances over passable tiles from `start`; `blocked(p)` marks extra obstacles.
    /// `seen_only` restricts to seen tiles. Unreachable = -1.
    pub fn bfs(&self, start: Pos, seen_only: bool, blocked: &dyn Fn(Pos) -> bool) -> Vec<i32> {
        self.bfs_core(start, seen_only, blocked, None)
    }
    /// BFS with parents, for first-step extraction. Returns (dist, parent index or -1).
    pub fn bfs_parent(&self, start: Pos, seen_only: bool, blocked: &dyn Fn(Pos) -> bool) -> (Vec<i32>, Vec<i32>) {
        let mut parent = vec![-1i32; self.tiles.len()];
        let dist = self.bfs_core(start, seen_only, blocked, Some(&mut parent));
        (dist, parent)
    }
    /// The shared flood: 8-connected without cutting wall corners (`can_step`), FIFO in `DIRS8`
    /// order, so distances and parents are exactly those of a step-by-step walk. Index
    /// arithmetic throughout — this is the sim's hottest loop (every chore paths the floor).
    fn bfs_core(&self, start: Pos, seen_only: bool, blocked: &dyn Fn(Pos) -> bool, mut parent: Option<&mut Vec<i32>>) -> Vec<i32> {
        let n = self.tiles.len();
        let mut dist = vec![-1i32; n];
        if !self.in_bounds(start) {
            return dist;
        }
        let (w, h) = (self.w, self.h);
        let tiles = &self.tiles[..];
        let mut queue: Vec<usize> = Vec::with_capacity(n);
        let si = self.idx(start);
        dist[si] = 0;
        queue.push(si);
        let mut head = 0;
        while head < queue.len() {
            let pi = queue[head];
            head += 1;
            let d = dist[pi] + 1;
            let (px, py) = (pi as i32 % w, pi as i32 / w);
            for (dx, dy) in DIRS8 {
                let (qx, qy) = (px + dx, py + dy);
                if qx < 0 || qy < 0 || qx >= w || qy >= h {
                    continue;
                }
                let qi = (qy * w + qx) as usize;
                if !tiles[qi].passable() {
                    continue;
                }
                if dx != 0 && dy != 0 && (tiles[(py * w + qx) as usize] == Tile::Wall || tiles[(qy * w + px) as usize] == Tile::Wall) {
                    continue;
                }
                if (seen_only && !self.seen[qi]) || dist[qi] >= 0 || blocked(Pos::new(qx, qy)) {
                    continue;
                }
                dist[qi] = d;
                if let Some(parent) = parent.as_deref_mut() {
                    parent[qi] = pi as i32;
                }
                queue.push(qi);
            }
        }
        dist
    }
    /// First step from `start` toward `goal` along BFS parents (None if unreachable or equal).
    pub fn first_step(&self, parent: &[i32], start: Pos, goal: Pos) -> Option<Pos> {
        if goal == start || !self.in_bounds(goal) {
            return None;
        }
        let si = self.idx(start) as i32;
        let mut cur = self.idx(goal) as i32;
        if parent[cur as usize] < 0 {
            return None;
        }
        while parent[cur as usize] != si {
            cur = parent[cur as usize];
            if cur < 0 {
                return None;
            }
        }
        Some(self.pos(cur as usize))
    }
    /// From a BFS distance map rooted at some origin, the neighbour of `p` one step closer.
    pub fn step_down(&self, dist: &[i32], p: Pos, occupied: &dyn Fn(Pos) -> bool) -> Option<Pos> {
        let mut best: Option<(i32, Pos)> = None;
        for dir in DIRS8 {
            let q = p.step(dir);
            if !self.in_bounds(q) || !self.can_step(p, q) {
                continue;
            }
            let d = dist[self.idx(q)];
            if d < 0 || occupied(q) {
                continue;
            }
            if best.is_none_or(|(bd, _)| d < bd) {
                best = Some((d, q));
            }
        }
        best.filter(|(d, _)| *d < dist[self.idx(p)]).map(|(_, q)| q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn room() -> Map {
        let mut m = Map::new(7, 7, Tile::Wall);
        for y in 1..6 {
            for x in 1..6 {
                m.set(Pos::new(x, y), Tile::Floor);
            }
        }
        m
    }
    #[test]
    fn los_blocked_by_wall() {
        let mut m = room();
        m.set(Pos::new(3, 3), Tile::Wall);
        assert!(!m.los(Pos::new(1, 3), Pos::new(5, 3)));
        assert!(m.los(Pos::new(1, 1), Pos::new(5, 1)));
    }
    #[test]
    fn no_corner_cutting() {
        let mut m = room();
        m.set(Pos::new(2, 1), Tile::Wall);
        m.set(Pos::new(1, 2), Tile::Wall);
        assert!(!m.can_step(Pos::new(1, 1), Pos::new(2, 2)));
        assert!(m.can_step(Pos::new(3, 3), Pos::new(4, 4)));
    }
    #[test]
    fn bfs_reaches_and_blocks() {
        let m = room();
        let d = m.bfs(Pos::new(1, 1), false, &|_| false);
        assert_eq!(d[m.idx(Pos::new(5, 5))], 4);
        assert_eq!(d[m.idx(Pos::new(0, 0))], -1);
        let d2 = m.bfs(Pos::new(1, 1), false, &|p| p.x == 3);
        assert_eq!(d2[m.idx(Pos::new(5, 5))], -1);
    }
    #[test]
    fn vision_marks_seen() {
        let mut m = room();
        m.update_vision(Pos::new(3, 3), 7);
        assert!(m.is_visible(Pos::new(1, 1)));
        assert!(m.is_seen(Pos::new(5, 5)));
        assert_eq!(m.seen_pct(), 100);
    }
}
