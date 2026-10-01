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
    /// What `visible` was last worked out from (`update_vision`): not the map's state, never saved,
    /// and equal on every map.
    #[serde(skip)]
    pub vis_from: VisFrom,
}

/// `update_vision`'s inputs when it last ran by the walls' bits — (from, radius, w, h, the square's
/// sight-blocking tiles): the same inputs give the same `visible`, already there, and every tile of
/// it already `seen` (only `update_vision` writes `visible`; nothing unsees a tile).
#[derive(Clone, Debug, Default)]
pub struct VisFrom(Option<(Pos, i32, i32, i32, [u64; LOS_MASK_WORDS])>);

impl PartialEq for VisFrom {
    fn eq(&self, _: &VisFrom) -> bool {
        true
    }
}
impl Eq for VisFrom {}

impl Map {
    pub fn new(w: i32, h: i32, fill: Tile) -> Map {
        let n = (w * h) as usize;
        Map { w, h, tiles: vec![fill; n], seen: vec![false; n], visible: vec![false; n], corridor: vec![false; n], vis_from: VisFrom::default() }
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
        // Between two in-bounds ends every Bresenham point is in bounds (inside their box):
        // index the tiles straight (225 of these per tick from `update_vision`).
        if self.in_bounds(a) && self.in_bounds(b) {
            let (w, tiles) = (self.w, &self.tiles[..]);
            Self::los_by(a, b, |x, y| tiles[(y * w + x) as usize].blocks_sight())
        } else {
            Self::los_by(a, b, |x, y| self.get(Pos::new(x, y)).blocks_sight())
        }
    }
    #[inline(always)]
    fn los_by(a: Pos, b: Pos, mut blocks: impl FnMut(i32, i32) -> bool) -> bool {
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
            if blocks(x0, y0) {
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
        // Both counts in one pass (25–34 k calls per 8 h).
        if self.seen.len() != self.tiles.len() {
            return (self.count_seen_passable() * 100 / self.count_passable().max(1)) as i32;
        }
        let (mut total, mut seen) = (0usize, 0usize);
        for (t, s) in self.tiles.iter().zip(self.seen.iter()) {
            let p = t.passable() as usize;
            total += p;
            seen += p & (*s as usize);
        }
        (seen * 100 / total.max(1)) as i32
    }
    /// Recompute `visible` from `from` with radius and line of sight; marks seen.
    pub fn update_vision(&mut self, from: Pos, radius: i32) {
        let memo = self.vis_from.0.take();
        let clear = |visible: &mut [bool]| {
            for v in visible.iter_mut() {
                *v = false;
            }
        };
        if !self.in_bounds(from) || !(0..=LOS_TABLE_MAX).contains(&radius) {
            clear(&mut self.visible);
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
            return;
        }
        // `los` from an in-bounds `from` to each in-bounds tile of the square, walked from
        // the precomputed offsets (`los_table`): the same points in the same order.
        let table = los_table(radius);
        let (w, tiles) = (self.w, &self.tiles[..]);
        if radius <= LOS_MASK_MAX {
            // The square's sight-blocking tiles as bits, read once; a tile is in view when no
            // point of its walk is one (`LosTable.masks`: each walk's points as bits) — the same
            // test as the walk's, on every in-bounds end (their points are in the square).
            let (side, h) = (2 * radius + 1, self.h);
            let mut walls = [0u64; LOS_MASK_WORDS];
            // (a row's in-bounds stretch of the square as one slice, its walls as a run of bits)
            let (x0, x1) = ((from.x - radius).max(0), (from.x + radius).min(w - 1));
            for dy in -radius..=radius {
                let y = from.y + dy;
                if y < 0 || y >= h || x0 > x1 {
                    continue;
                }
                let row = &tiles[(y * w + x0) as usize..=(y * w + x1) as usize];
                let mut bits: u64 = 0;
                for (j, t) in row.iter().enumerate() {
                    bits |= (t.blocks_sight() as u64) << j;
                }
                let bit = ((dy + radius) * side + x0 - from.x + radius) as usize;
                walls[bit >> 6] |= bits << (bit & 63);
                if (bit & 63) + row.len() > 64 {
                    walls[(bit >> 6) + 1] |= bits >> (64 - (bit & 63));
                }
            }
            let key = (from, radius, w, h, walls);
            self.vis_from.0 = Some(key);
            if memo == Some(key) {
                return;
            }
            clear(&mut self.visible);
            let mut k = 0;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let m = &table.masks[k];
                    k += 1;
                    let p = from.step((dx, dy));
                    // (branch-free: the six words' overlaps ORed)
                    let hit = (m[0] & walls[0]) | (m[1] & walls[1]) | (m[2] & walls[2]) | (m[3] & walls[3]) | (m[4] & walls[4]) | (m[5] & walls[5]);
                    if hit == 0 && self.in_bounds(p) {
                        let i = self.idx(p);
                        self.visible[i] = true;
                        self.seen[i] = true;
                    }
                }
            }
            return;
        }
        clear(&mut self.visible);
        let mut k = 0;
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (a, b) = table.spans[k];
                k += 1;
                let p = from.step((dx, dy));
                if !self.in_bounds(p) {
                    continue;
                }
                if table.offs[a as usize..b as usize].iter().all(|&(ox, oy)| !tiles[((from.y + oy as i32) * w + from.x + ox as i32) as usize].blocks_sight()) {
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
    pub fn bfs(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized)) -> Vec<i32> {
        self.bfs_core(start, seen_only, blocked, None)
    }
    /// BFS with parents, for first-step extraction. Returns (dist, parent index or -1).
    pub fn bfs_parent(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized)) -> (Vec<i32>, Vec<i32>) {
        let mut parent = vec![-1i32; self.tiles.len()];
        let dist = self.bfs_core(start, seen_only, blocked, Some(&mut parent));
        (dist, parent)
    }
    /// The nearest tile (> 0 steps) satisfying `pred` over `bfs_parent`'s flood — least
    /// distance, then least index, exactly what a scan of the full distance map yields — and
    /// the parents. The flood stops at the first layer holding one, so the parents are final
    /// only along paths no longer than it (all `first_step` toward that tile reads).
    pub fn bfs_nearest(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized), pred: &dyn Fn(Pos) -> bool) -> Option<(Pos, Vec<i32>)> {
        let mut parent = vec![-1i32; self.tiles.len()];
        let mut found = None;
        self.bfs_layers(start, seen_only, blocked, Some(&mut parent), |d, layer| {
            if d > 0 {
                found = layer.iter().copied().filter(|&i| pred(self.pos(i))).min();
            }
            found.is_some()
        });
        found.map(|i| (self.pos(i), parent))
    }
    /// `bfs_parent` stopped by `stop(d, layer)` (`bfs_layers`): every tile nearer than the layer it
    /// stopped at, and that layer's, has the full flood's distance and parent; farther ones −1.
    pub fn bfs_parent_layers(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized), stop: impl FnMut(i32, &[usize]) -> bool) -> (Vec<i32>, Vec<i32>) {
        let mut parent = vec![-1i32; self.tiles.len()];
        let dist = self.bfs_layers(start, seen_only, blocked, Some(&mut parent), stop);
        (dist, parent)
    }
    /// `bfs_parent` stopped once `goal` is reached: the parents along its path are final (a
    /// tile's parent is set when it is discovered), which is all `first_step` toward it reads.
    pub fn bfs_parent_to(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized), goal: Pos) -> Vec<i32> {
        let mut parent = vec![-1i32; self.tiles.len()];
        let gi = if self.in_bounds(goal) { self.idx(goal) } else { usize::MAX };
        self.bfs_layers(start, seen_only, blocked, Some(&mut parent), |_, layer| layer.contains(&gi));
        parent
    }
    fn bfs_core(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized), parent: Option<&mut Vec<i32>>) -> Vec<i32> {
        self.bfs_layers(start, seen_only, blocked, parent, |_, _| false)
    }
    /// The shared flood: 8-connected without cutting wall corners (`can_step`), FIFO in `DIRS8`
    /// order, so distances and parents are exactly those of a step-by-step walk. Index
    /// arithmetic throughout — this is the sim's hottest loop (every chore paths the floor).
    /// Generic over `blocked` so the common `&|_| false` flood compiles without a call per tile.
    /// `stop(d, layer)` sees each distance's complete layer (every tile at `d`, in discovery
    /// order) as its first tile is expanded; `true` ends the flood there.
    #[inline(always)]
    fn bfs_layers(&self, start: Pos, seen_only: bool, blocked: &(impl Fn(Pos) -> bool + ?Sized), mut parent: Option<&mut Vec<i32>>, mut stop: impl FnMut(i32, &[usize]) -> bool) -> Vec<i32> {
        let n = self.tiles.len();
        let mut dist = vec![-1i32; n];
        if !self.in_bounds(start) {
            return dist;
        }
        let w = self.w as usize;
        let tiles = &self.tiles[..];
        let seen = &self.seen[..];
        let mut queue: Vec<usize> = Vec::with_capacity(n);
        let si = self.idx(start);
        dist[si] = 0;
        queue.push(si);
        let mut head = 0;
        let mut layer = -1;
        while head < queue.len() {
            let pi = queue[head];
            if dist[pi] != layer {
                layer = dist[pi];
                if stop(layer, &queue[head..]) {
                    break;
                }
            }
            head += 1;
            let d = dist[pi] + 1;
            // Every test is pure, so their order is free: the visited one first (most
            // neighbours of a flood are), the closure last.
            each_step(tiles, w, self.h as usize, pi, &mut dist, |dist, qi, qx, qy| {
                if (seen_only && !seen[qi]) || blocked(Pos::new(qx, qy)) {
                    return;
                }
                dist[qi] = d;
                if let Some(parent) = parent.as_deref_mut() {
                    parent[qi] = pi as i32;
                }
                queue.push(qi);
            });
        }
        dist
    }
    /// `bfs(start, false, no blocks)` in resumable steps: `dist`/`queue`/`head` hold a flood
    /// begun by `flood_start` and carried on here in the same FIFO, `DIRS8` order with the same
    /// tests, so every distance it assigns is the full flood's. With `until`, it stops once
    /// that tile has a distance (after finishing the tile being expanded); with `within`, once
    /// every tile that near has one (the next to expand is that far: the queue is in distance
    /// order); neither runs it out.
    pub fn flood_start(&self, start: Pos, dist: &mut Vec<i32>, queue: &mut Vec<u32>, head: &mut usize) {
        dist.clear();
        dist.resize(self.tiles.len(), -1);
        queue.clear();
        *head = 0;
        if self.in_bounds(start) {
            let si = self.idx(start);
            dist[si] = 0;
            queue.push(si as u32);
        }
    }
    pub fn flood_resume(&self, dist: &mut [i32], queue: &mut Vec<u32>, head: &mut usize, until: Option<usize>, within: Option<i32>) {
        let (w, h) = (self.w as usize, self.h as usize);
        let tiles = &self.tiles[..];
        while *head < queue.len() {
            let pi = queue[*head] as usize;
            if until.is_some_and(|u| dist[u] >= 0) || within.is_some_and(|l| dist[pi] >= l) {
                return;
            }
            *head += 1;
            let d = dist[pi] + 1;
            each_step(tiles, w, h, pi, dist, |dist, qi, _, _| {
                dist[qi] = d;
                queue.push(qi as u32);
            });
        }
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

/// The flood's steps from tile `pi` of a `w` × `h` map, in `DIRS8` order: each neighbour in bounds,
/// not yet reached (read first: most neighbours of a flood were), passable, and not across a
/// wall's corner (`Map::can_step`) is passed to `step` with its coordinates (`dist` < 0 is not yet reached). The neighbours by index
/// offset and the row's edges tested once — the floods' inner loop (`bfs_layers`, `flood_resume`).
#[inline(always)]
fn each_step(tiles: &[Tile], w: usize, h: usize, pi: usize, dist: &mut [i32], mut step: impl FnMut(&mut [i32], usize, i32, i32)) {
    let (py, px) = (pi / w, pi % w);
    let (l, r, u, d) = (px > 0, px + 1 < w, py > 0, py + 1 < h);
    let (x, y) = (px as i32, py as i32);
    let open = |dist: &[i32], q: usize| dist[q] < 0 && tiles[q].passable();
    // (a diagonal step: neither orthogonal tile it passes between is a wall)
    let corner = |a: usize, b: usize| tiles[a] != Tile::Wall && tiles[b] != Tile::Wall;
    // DIRS8: (0,-1) (1,0) (0,1) (-1,0) (1,-1) (1,1) (-1,1) (-1,-1)
    if u && open(dist, pi - w) {
        step(dist, pi - w, x, y - 1);
    }
    if r && open(dist, pi + 1) {
        step(dist, pi + 1, x + 1, y);
    }
    if d && open(dist, pi + w) {
        step(dist, pi + w, x, y + 1);
    }
    if l && open(dist, pi - 1) {
        step(dist, pi - 1, x - 1, y);
    }
    if r && u && open(dist, pi + 1 - w) && corner(pi + 1, pi - w) {
        step(dist, pi + 1 - w, x + 1, y - 1);
    }
    if r && d && open(dist, pi + 1 + w) && corner(pi + 1, pi + w) {
        step(dist, pi + 1 + w, x + 1, y + 1);
    }
    if l && d && open(dist, pi - 1 + w) && corner(pi - 1, pi + w) {
        step(dist, pi - 1 + w, x - 1, y + 1);
    }
    if l && u && open(dist, pi - 1 - w) && corner(pi - 1, pi - w) {
        step(dist, pi - 1 - w, x - 1, y - 1);
    }
}

/// The largest vision radius with a precomputed walk (`update_vision`; larger ones walk `los`).
const LOS_TABLE_MAX: i32 = 32;

/// `los`'s Bresenham walk from (0, 0) to every offset of a radius's square, in
/// `update_vision`'s order: the points strictly between the ends (the walk depends only on
/// the ends' difference, so it is the same from any origin).
struct LosTable {
    spans: Vec<(u32, u32)>,
    offs: Vec<(i8, i8)>,
    /// Radius ≤ `LOS_MASK_MAX`: each walk's points as bits of the square (row-major from the
    /// top-left corner), for `update_vision`'s one-pass test.
    masks: Vec<[u64; LOS_MASK_WORDS]>,
}

/// The largest radius whose square fits `LOS_MASK_WORDS` words of bits (19 × 19 = 361 ≤ 384).
const LOS_MASK_MAX: i32 = 9;
const LOS_MASK_WORDS: usize = 6;

fn los_table(radius: i32) -> std::rc::Rc<LosTable> {
    thread_local! {
        static TABLES: std::cell::RefCell<Vec<Option<std::rc::Rc<LosTable>>>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    TABLES.with(|t| {
        let mut t = t.borrow_mut();
        let r = radius as usize;
        if t.len() <= r {
            t.resize(r + 1, None);
        }
        t[r].get_or_insert_with(|| {
            let (mut spans, mut offs) = (Vec::new(), Vec::new());
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    let a = offs.len() as u32;
                    if (dx, dy) != (0, 0) {
                        Map::los_by(Pos::new(0, 0), Pos::new(dx, dy), |x, y| {
                            offs.push((x as i8, y as i8));
                            false
                        });
                    }
                    spans.push((a, offs.len() as u32));
                }
            }
            let side = 2 * radius + 1;
            let masks = if radius <= LOS_MASK_MAX {
                spans
                    .iter()
                    .map(|&(a, b)| {
                        let mut m = [0u64; LOS_MASK_WORDS];
                        for &(ox, oy) in &offs[a as usize..b as usize] {
                            let bit = ((oy as i32 + radius) * side + ox as i32 + radius) as usize;
                            m[bit >> 6] |= 1 << (bit & 63);
                        }
                        m
                    })
                    .collect()
            } else {
                Vec::new()
            };
            std::rc::Rc::new(LosTable { spans, offs, masks })
        })
        .clone()
    })
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
