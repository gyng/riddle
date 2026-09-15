//! Deterministic xorshift64* RNG. No `rand`, no wall clock.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Rng {
    s: u64,
}

pub fn splitmix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Stable FNV-1a hash of a string, for deriving seeds from names.
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

impl Rng {
    pub fn new(seed: u64) -> Rng {
        let mut s = splitmix(seed);
        if s == 0 {
            s = 0x9E37_79B9_7F4A_7C15;
        }
        Rng { s }
    }
    /// A stream derived from a seed and a tag; different tags never collide in practice.
    pub fn derive(seed: u64, tag: u64) -> Rng {
        Rng::new(splitmix(seed) ^ splitmix(tag.rotate_left(17)).wrapping_mul(3))
    }
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.s;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.s = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    /// Uniform in `0..n` (0 when n == 0).
    pub fn below(&mut self, n: u32) -> u32 {
        if n == 0 {
            0
        } else {
            (((self.next_u64() >> 32) * n as u64) >> 32) as u32
        }
    }
    /// Uniform inclusive range.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            lo
        } else {
            lo + self.below((hi - lo + 1) as u32) as i32
        }
    }
    /// True with probability `pct` percent.
    pub fn chance(&mut self, pct: u32) -> bool {
        self.below(100) < pct
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len() as u32) as usize]
    }
    pub fn shuffle<T>(&mut self, xs: &mut [T]) {
        for i in (1..xs.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            xs.swap(i, j);
        }
    }
    /// Index chosen proportionally to `weights` (all zero ⇒ 0).
    pub fn weighted(&mut self, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return 0;
        }
        let mut r = self.below(total);
        for (i, w) in weights.iter().enumerate() {
            if r < *w {
                return i;
            }
            r -= w;
        }
        weights.len() - 1
    }
    pub fn fork(&mut self) -> Rng {
        Rng::new(self.next_u64())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deterministic_stream() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }
    #[test]
    fn ranges_in_bounds() {
        let mut r = Rng::new(3);
        for _ in 0..1000 {
            let v = r.range(-2, 5);
            assert!((-2..=5).contains(&v));
            assert!(r.below(7) < 7);
        }
        assert_eq!(r.below(0), 0);
        assert_eq!(r.range(4, 4), 4);
    }
    #[test]
    fn weighted_respects_zero() {
        let mut r = Rng::new(9);
        for _ in 0..200 {
            assert_eq!(r.weighted(&[0, 5, 0]), 1);
        }
    }
}
