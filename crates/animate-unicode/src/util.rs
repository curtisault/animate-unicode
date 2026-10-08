//! Small things most pieces want: a seeded PRNG (never `rand` from a clock:
//! frames must be deterministic in `t`), a ramp lookup, braille encoding.

/// xorshift64*, seeded. Cheap, good enough for art, the same on every platform.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    /// A generator from a seed. Any seed is fine, 0 included.
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f64()
    }

    /// Uniform in 0..n.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

/// The glyph of a ramp (`" .:-=+*#%@"`, say) for a brightness in [0, 1],
/// flipped for paper so dark cells stay dark on a light ground.
pub fn ramp(ramp: &[char], v: f64, paper: bool) -> char {
    let n = ramp.len();
    let i = ((v.clamp(0.0, 1.0) * (n - 1) as f64) + 0.5) as usize;
    ramp[if paper { n - 1 - i } else { i }]
}

/// A braille cell (U+2800 + bits) from a 2-wide, 4-tall dot mask: `dots[row]`
/// holds the left dot in bit 0 and the right dot in bit 1. Eight dots a cell,
/// so a `cols × rows` grid is a `2·cols × 4·rows` bitmap.
pub fn braille(dots: [u8; 4]) -> char {
    // Dot numbering: left column 1,2,3,7 and right column 4,5,6,8, top to bottom.
    const LEFT: [u32; 4] = [0x01, 0x02, 0x04, 0x40];
    const RIGHT: [u32; 4] = [0x08, 0x10, 0x20, 0x80];
    let mut bits = 0;
    for (r, &d) in dots.iter().enumerate() {
        if d & 1 != 0 {
            bits |= LEFT[r];
        }
        if d & 2 != 0 {
            bits |= RIGHT[r];
        }
    }
    char::from_u32(0x2800 + bits).unwrap()
}

/// 0..1 as an ease-in-out curve.
pub fn smoothstep(x: f64) -> f64 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..1000 {
            let x = a.f64();
            assert_eq!(x, b.f64());
            assert!((0.0..1.0).contains(&x));
        }
        assert_ne!(Rng::new(1).next_u64(), Rng::new(2).next_u64());
    }

    #[test]
    fn braille_dots_map_to_the_right_bits() {
        assert_eq!(braille([0, 0, 0, 0]), '\u{2800}');
        assert_eq!(braille([1, 0, 0, 0]), '\u{2801}'); // dot 1
        assert_eq!(braille([2, 0, 0, 0]), '\u{2808}'); // dot 4
        assert_eq!(braille([0, 0, 0, 3]), '\u{28C0}'); // dots 7 and 8
        assert_eq!(braille([3, 3, 3, 3]), '\u{28FF}');
    }

    #[test]
    fn ramp_flips_on_paper() {
        let r: Vec<char> = " .:#".chars().collect();
        assert_eq!(ramp(&r, 0.0, false), ' ');
        assert_eq!(ramp(&r, 1.0, false), '#');
        assert_eq!(ramp(&r, 1.0, true), ' ');
    }
}
