//! Grid positions, distances, lines.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

pub const DIRS8: [(i32, i32); 8] = [(0, -1), (1, 0), (0, 1), (-1, 0), (1, -1), (1, 1), (-1, 1), (-1, -1)];
pub const DIRS4: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

impl Pos {
    pub const fn new(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }
    pub fn cheb(self, o: Pos) -> i32 {
        (self.x - o.x).abs().max((self.y - o.y).abs())
    }
    pub fn adjacent(self, o: Pos) -> bool {
        self != o && self.cheb(o) <= 1
    }
    pub fn add(self, d: (i32, i32)) -> Pos {
        Pos::new(self.x + d.0, self.y + d.1)
    }
    pub fn neighbours8(self) -> [Pos; 8] {
        let mut out = [self; 8];
        for (i, d) in DIRS8.iter().enumerate() {
            out[i] = self.add(*d);
        }
        out
    }
}

/// Bresenham line from `a` to `b`, inclusive of both ends.
pub fn line(a: Pos, b: Pos) -> Vec<Pos> {
    let mut out = Vec::with_capacity(16);
    let (mut x0, mut y0) = (a.x, a.y);
    let (x1, y1) = (b.x, b.y);
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        out.push(Pos::new(x0, y0));
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn line_endpoints_and_length() {
        let l = line(Pos::new(0, 0), Pos::new(5, 2));
        assert_eq!(l.first(), Some(&Pos::new(0, 0)));
        assert_eq!(l.last(), Some(&Pos::new(5, 2)));
        assert_eq!(l.len(), 6);
    }
    #[test]
    fn cheb_distance() {
        assert_eq!(Pos::new(0, 0).cheb(Pos::new(3, -2)), 3);
        assert!(Pos::new(1, 1).adjacent(Pos::new(2, 2)));
        assert!(!Pos::new(1, 1).adjacent(Pos::new(1, 1)));
    }
}
