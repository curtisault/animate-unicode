//! plucked string: a string stretched over two bridges, slowed down. A finger
//! pulls it into a corner and lets go; the corner splits in two and runs back
//! and forth as the sum of its harmonics, the high ones dying first. After
//! ascii.rest's plucked-string.ts (MIT, @bas3line).
//!
//! The pluck and its harmonics are ascii.rest's. In braille the string is a
//! curve a dot wide, four dots to a row, so its corners stay sharp as they
//! run; and behind it its first three harmonics are drawn faintly, each in its
//! own colour, swelling and dying away, so the sum can be seen being made.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The plucked string's meta.
pub static META: Meta = Meta {
    name: "plucked string",
    slug: "plucked-string",
    category: Category::Physics,
    note: "a plucked string in braille, its first harmonics shown behind it",
    cols: 64,
    rows: 14,
    fps: 30,
    charset: Charset::Extended,
    // The string; the first three harmonics; the instrument; the finger.
    palette: Some(&["#fff2d0", "#e0606a", "#5ab0e0", "#7ad870", "#a8845a", "#e8c4a8"]),
    ground: Some("#0e0b09"),
    ..Meta::DEFAULT
};

/// The string at rest, in rows from the top; the bridges, as column edges;
/// harmonics summed.
const R0: f64 = 6.5;
const A: f64 = 9.0;
const B: f64 = 55.0;
const N: usize = 32;
/// Seconds for the fundamental, slowed down; between plucks; the finger
/// coming down, then pulling; and seconds in, the finger about to let go.
const PERIOD: f64 = 1.6;
const CYCLE: f64 = 6.0;
const DOWN: f64 = 0.55;
const PULL: f64 = 0.6;
const T0: f64 = 1.05;
/// Harmonics drawn behind the string.
const GHOSTS: usize = 3;
const STRING: u8 = 0;
const WOOD: u8 = 4;
const FINGER: u8 = 5;

/// ascii.rest's hash of (pluck, salt) into [0, 1).
fn hash(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b) ^ ((s + 1) as u32).wrapping_mul(0xc2b2_ae35);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

fn ease(u: f64) -> f64 {
    let u = u.clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Pluck k: where along the string, which way and how far, and its
/// triangle's harmonics. Plucks take turns pulling up and pressing down.
struct Pluck {
    p: f64,
    h: f64,
    b: [f64; N],
    col: i64,
}

fn pluck(k: i64) -> Pluck {
    let near = hash(k, 1) < 0.5;
    let p = if near { 0.2 + 0.18 * hash(k, 2) } else { 0.62 + 0.18 * hash(k, 2) };
    let h = (if k.rem_euclid(2) == 1 { -1.0 } else { 1.0 }) * (3.3 + 0.5 * hash(k, 3));
    let b = std::array::from_fn(|n| {
        let m = (n + 1) as f64;
        2.0 * h * (m * PI * p).sin() / (m * m * PI * PI * p * (1.0 - p))
    });
    Pluck { p, h, b, col: (A + p * (B - A)).floor() as i64 }
}

/// Harmonic n's height, s seconds after a pluck let go. Damping grows gently
/// with the harmonic, so the corner keeps its edge for a few passes.
fn amp(pk: &Pluck, n: usize, s: f64) -> f64 {
    let m = (n + 1) as f64;
    pk.b[n] * (2.0 * PI * m * s / PERIOD).cos() * (-(0.2 + 0.0022 * m * m) * s).exp()
}

/// The string: sin(n pi u) at each dot column, its height there, and the dots.
pub struct PluckedString {
    shape: Vec<[f64; N]>,
    y: Vec<f64>,
    dots: ColorDots,
}

/// A plucked string; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let shape = (0..dots.width())
        .map(|x| {
            let u = ((x as f64 + 0.5) / 2.0 - A) / (B - A);
            std::array::from_fn(|n| if u > 0.0 && u < 1.0 { ((n + 1) as f64 * PI * u).sin() } else { 0.0 })
        })
        .collect();
    let w = dots.width();
    Box::new(PluckedString { shape, y: vec![0.0; w], dots })
}

/// The box with an end block and a pin at each end, and a bridge near each.
fn base(grid: &mut Grid) {
    let (cols, r0) = (META.cols as i64, R0 as i64);
    let (a, b) = (A as i64, B as i64);
    for (pin, edge) in [(1, a), (cols - 2, b)] {
        grid.set(pin, r0, 'o', WOOD);
        for r in r0 + 1..12 {
            grid.set(pin - 1, r, '▐', WOOD);
            grid.set(pin, r, '█', WOOD);
            grid.set(pin + 1, r, '▌', WOOD);
        }
        for d in 1..=5 {
            grid.set(edge - d, r0 + d, '/', WOOD);
            grid.set(edge + d - 1, r0 + d, '\\', WOOD);
            if d == 5 {
                for c in edge - d + 1..edge + d - 1 {
                    grid.set(c, r0 + d, '_', WOOD);
                }
            }
        }
    }
    for c in (2..a).chain(b..cols - 2) {
        grid.set(c, r0, '─', WOOD);
    }
    for c in 0..cols {
        grid.set(c, 12, '▀', WOOD);
    }
    for c in 2..cols - 2 {
        grid.set(
            c,
            13,
            if c == 2 {
                '\\'
            } else if c == cols - 3 {
                '/'
            } else {
                '_'
            },
            WOOD,
        );
    }
}

impl Piece for PluckedString {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let t = T0 + time;
        let k = (t / CYCLE).floor() as i64;
        let s = t - k as f64 * CYCLE;
        let (pk, prev) = (pluck(k), pluck(k - 1));
        let held = DOWN + PULL;
        // Until the finger lets go, the last pluck rings on; then the new one.
        let (ring, at) = if s < held { (&prev, s + CYCLE - held) } else { (&pk, s - held) };
        let amps: Vec<f64> = (0..N).map(|n| amp(ring, n, at)).collect();
        let w = self.dots.width();
        for x in 0..w {
            self.y[x] = (0..N).map(|n| amps[n] * self.shape[x][n]).sum();
        }
        // The finger comes down to the string, hooking just under it to pull up
        // or resting on it to press down, then pulls it into a triangle.
        let up = pk.h > 0.0;
        let tri = |x: usize| {
            let u = ((x as f64 + 0.5) / 2.0 - A) / (B - A);
            if u <= 0.0 || u >= 1.0 {
                0.0
            } else if u < pk.p {
                pk.h * u / pk.p
            } else {
                pk.h * (1.0 - u) / (1.0 - pk.p)
            }
        };
        let mut pulling = 0.0;
        if s >= DOWN && s < held {
            pulling = ease((s - DOWN) / PULL);
            for x in 0..w {
                self.y[x] = (1.0 - pulling) * self.y[x] + pulling * tri(x);
            }
        }
        self.dots.clear();
        // The first harmonics, faint and dotted, behind the string; fading out while the finger holds it.
        for (n, &a) in amps.iter().enumerate().take(GHOSTS) {
            for x in (0..w).step_by(2) {
                let v = a * self.shape[x][n] * (1.0 - pulling);
                if self.shape[x][n] != 0.0 && v.abs() > 0.05 {
                    self.dots.set(x as i64, ((R0 - v) * 4.0).floor() as i64, n as u8 + 1);
                }
            }
        }
        // The string, a dot a column, joined where it runs steep.
        let (x0, x1) = ((A * 2.0) as usize, (B * 2.0) as usize);
        let mut last: Option<i64> = None;
        for x in x0..x1 {
            let y = ((R0 - self.y[x]) * 4.0).floor() as i64;
            let from = last.unwrap_or(y);
            for yy in from.min(y)..=from.max(y) {
                self.dots.set(x as i64, yy, STRING);
            }
            last = Some(y);
        }
        grid.clear();
        base(grid);
        self.dots.draw(grid, 0, 0);
        // The finger, from above the frame down to its rounded tip, in front of the string.
        let fc = pk.col;
        let under = i64::from(up);
        let string_row = |c: i64| (R0 - self.y[(c * 2 + 1) as usize]).floor() as i64;
        let tip = if s < DOWN {
            string_row(fc) + under - (10.0 * (1.0 - ease(s / DOWN))).round() as i64
        } else if s < held {
            string_row(fc) + under
        } else if s < held + 0.4 {
            let d = s - held;
            (R0 - pk.h).floor() as i64 + under - (25.0 * d + 40.0 * d * d).round() as i64
        } else {
            -1
        };
        if tip >= 0 {
            for r in 0..tip {
                grid.set(fc - 1, r, '│', FINGER);
                grid.set(fc, r, ' ', FINGER);
                grid.set(fc + 1, r, '│', FINGER);
            }
            grid.set(fc - 1, tip, '╰', FINGER);
            grid.set(fc, tip, '─', FINGER);
            grid.set(fc + 1, tip, '╯', FINGER);
        }
    }
}
