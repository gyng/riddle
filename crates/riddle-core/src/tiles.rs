//! Tiles, overlays and the floor map with vision bookkeeping.
use crate::geom::{line, Pos, DIRS4, DIRS8};
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
    pub fn los(&self, a: Pos, b: Pos) -> bool {
        let l = line(a, b);
        for p in &l[1..l.len().saturating_sub(1)] {
            if self.get(*p).blocks_sight() {
                return false;
            }
        }
        true
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
                let p = from.add((dx, dy));
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
            let open8 = DIRS8.iter().filter(|d| self.passable(p.add(**d))).count();
            let open4 = DIRS4.iter().filter(|d| self.passable(p.add(**d))).count();
            self.corridor[i] = open4 <= 2 && open8 <= 4;
        }
    }
    /// BFS distances over passable tiles from `start`; `blocked(p)` marks extra obstacles.
    /// `seen_only` restricts to seen tiles. Unreachable = -1.
    pub fn bfs(&self, start: Pos, seen_only: bool, blocked: &dyn Fn(Pos) -> bool) -> Vec<i32> {
        let n = self.tiles.len();
        let mut dist = vec![-1i32; n];
        let mut queue = std::collections::VecDeque::with_capacity(n);
        if !self.in_bounds(start) {
            return dist;
        }
        dist[self.idx(start)] = 0;
        queue.push_back(start);
        while let Some(p) = queue.pop_front() {
            let d = dist[self.idx(p)];
            for dir in DIRS8 {
                let q = p.add(dir);
                if !self.in_bounds(q) || !self.can_step(p, q) {
                    continue;
                }
                if seen_only && !self.seen[self.idx(q)] {
                    continue;
                }
                let qi = self.idx(q);
                if dist[qi] >= 0 || blocked(q) {
                    continue;
                }
                dist[qi] = d + 1;
                queue.push_back(q);
            }
        }
        dist
    }
    /// BFS with parents, for first-step extraction. Returns (dist, parent index or -1).
    pub fn bfs_parent(&self, start: Pos, seen_only: bool, blocked: &dyn Fn(Pos) -> bool) -> (Vec<i32>, Vec<i32>) {
        let n = self.tiles.len();
        let mut dist = vec![-1i32; n];
        let mut parent = vec![-1i32; n];
        let mut queue = std::collections::VecDeque::with_capacity(n);
        if !self.in_bounds(start) {
            return (dist, parent);
        }
        dist[self.idx(start)] = 0;
        queue.push_back(start);
        while let Some(p) = queue.pop_front() {
            let pi = self.idx(p);
            let d = dist[pi];
            for dir in DIRS8 {
                let q = p.add(dir);
                if !self.in_bounds(q) || !self.can_step(p, q) {
                    continue;
                }
                let qi = self.idx(q);
                if (seen_only && !self.seen[qi]) || dist[qi] >= 0 || blocked(q) {
                    continue;
                }
                dist[qi] = d + 1;
                parent[qi] = pi as i32;
                queue.push_back(q);
            }
        }
        (dist, parent)
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
            let q = p.add(dir);
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
