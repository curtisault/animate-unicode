//! quadrant fire: the PSX fire spread, drawn in quadrant blocks in the Doom
//! fire palette. The first coloured piece: it exercises the canvas renderer,
//! the palette and the ground.
//!
//! The simulation is a straight port of ascii.rest's doom-fire.ts (MIT,
//! @bas3line): each step every cell climbs one row, drifts a column either way
//! and cools a little, on a grid of square cells two across and four down to a
//! character. Columns cool at different rates, so the fire stands up in
//! separate tongues over a bed that never dims. The unit test
//! `simulation_matches_the_typescript_original` holds it to the TS frames.
//!
//! What is new is the drawing: each character cell is a 2 × 2 quadrant glyph
//! (each quadrant two simulation cells tall), coloured from the 36 steps of
//! the Doom fire palette, so the flame's edge has twice the resolution of the
//! ASCII version and its body shades from deep red to white.

use crate::{Category, Env, Grid, Meta, Options, Piece, Quadrants, Ramp};

/// The quadrant fire's meta.
pub static META: Meta = Meta {
    name: "quadrant fire",
    slug: "quadrant-fire",
    category: Category::Effects,
    note: "the psx fire spread in quadrant blocks, shaded in the doom palette",
    cols: 60,
    rows: 18,
    fps: 24,
    palette: Some(&PALETTE),
    ground: Some("#070707"),
    ..Meta::DEFAULT
};

/// The Doom fire palette above its black, dimmest first. The black is the ground.
const PALETTE: [&str; 35] = [
    "#1f0707", "#2f0f07", "#470f07", "#571707", "#671f07", "#771f07", "#8f2707", "#9f2f07", "#af3f07", "#bf4707", "#c74707", "#df4f07", "#df5707", "#df5707",
    "#d75f07", "#d7670f", "#cf6f0f", "#cf770f", "#cf7f0f", "#cf8717", "#c78717", "#c78f17", "#c7971f", "#bf9f1f", "#bf9f1f", "#bfa727", "#bfa727", "#bfaf2f",
    "#b7af2f", "#b7b72f", "#b7b737", "#cfcf6f", "#dfdf9f", "#efefc7", "#ffffff",
];
const SHADE: Ramp = Ramp::over(&PALETTE);

const SX: usize = 2;
const SY: usize = 4;
const MAX: f64 = 36.0;
/// Spread steps a second.
const RATE: f64 = 24.0;
/// The share of the width that burns.
const BED: f64 = 0.72;
/// Tongues: place across the bed from -1 to 1, tallest as a share of the
/// frame, half width in cells. Between them the fire cools fast.
const TONGUES: [(f64, f64, f64); 4] = [(-0.66, 1.05, 11.0), (-0.18, 2.1, 15.0), (0.3, 1.6, 13.0), (0.73, 0.95, 9.0)];
/// Steps for a tongue's height to wander through one knot.
const DRIFT: f64 = 34.0;
const FLOOR: f64 = 0.16;
/// A quadrant shows once its heat clears the floor by this much.
const LIT: f64 = 0.03;
/// Below this a quadrant needs lit neighbours to show.
const FAINT: f64 = 0.3;

/// The fire: its heat field and the state of its spread.
pub struct QuadrantFire {
    w: usize,
    h: usize,
    rand: Mulberry32,
    heat: Vec<f32>,
    cool: Vec<f32>,
    tall: Vec<f32>,
    bed: Vec<f32>,
    n: u32,
    last: f64,
    acc: f64,
    quads: Quadrants,
}

/// A quadrant fire; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(QuadrantFire::new())
}

/// mulberry32, as doom-fire.ts has it, so the spread matches it step for step.
struct Mulberry32(u32);

impl Mulberry32 {
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let s = self.0;
        let mut t = (s ^ (s >> 15)).wrapping_mul(1 | s);
        t = t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t)) ^ t;
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

/// doom-fire.ts's integer hash of a lattice point, in [0, 1).
fn hash(x: f64, y: f64) -> f64 {
    // Math.imul on JS numbers: both sides to 32 bits, the low 32 bits of the product.
    let (x, y) = (x as i64 as u32, y as i64 as u32);
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// Value noise over the hash, eased between lattice points.
fn noise(x: f64, y: f64) -> f64 {
    let (i, j) = (x.floor(), y.floor());
    let (u, v) = (ease(x - i), ease(y - j));
    let (a, b, c, d) = (hash(i, j), hash(i + 1.0, j), hash(i, j + 1.0), hash(i + 1.0, j + 1.0));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

impl QuadrantFire {
    fn new() -> Self {
        let (cols, rows) = (META.cols, META.rows);
        let (w, h) = (cols * SX, rows * SY + 1);
        // The bed: white hot right across, dropping off over the last few columns.
        let bed = (0..w)
            .map(|x| {
                let d = ((2.0 * x as f64 + 1.0) / w as f64 - 1.0).abs() / BED;
                (if d < 0.88 {
                    1.0
                } else if d < 1.0 {
                    ease((1.0 - d) / 0.12)
                } else {
                    0.0
                }) as f32
            })
            .collect();
        let mut fire = Self {
            w,
            h,
            rand: Mulberry32(9),
            heat: vec![0.0; w * h],
            cool: vec![0.0; w],
            tall: vec![0.0; w],
            bed,
            n: 0,
            last: 0.0,
            acc: 0.0,
            quads: Quadrants::new(cols, rows),
        };
        for _ in 0..200 {
            fire.step();
        }
        fire
    }

    /// One step of the spread. Line for line doom-fire.ts's `step`.
    fn step(&mut self) {
        let (w, h) = (self.w, self.h);
        self.n += 1;
        let n = f64::from(self.n);
        // Each column cools at the rate that lets the nearest tongue reach its
        // height, and each tongue's height wanders on its own, so they stand
        // apart and rise and fall out of step.
        self.tall.fill(0.28);
        for (i, &(at, top, half)) in TONGUES.iter().enumerate() {
            let i = i as f64;
            let lift = top * (0.55 + 0.45 * (noise(i * 7.3, n / DRIFT) * 0.7 + noise(i * 3.1 + 40.0, n / (DRIFT * 0.4)) * 0.3));
            let mid = (w as f64 / 2.0) * (1.0 + at * BED) + 6.0 * (noise(i * 5.7 + 90.0, n / (DRIFT * 2.0)) - 0.5);
            let mut x = (mid - half).floor().max(0.0);
            while x < (w as f64).min(mid + half) {
                let d = (x + 0.5 - mid) / half;
                let k = x as usize;
                self.tall[k] = f64::from(self.tall[k]).max(lift * (1.0 - d.abs().powf(1.3))) as f32;
                x += 1.0;
            }
        }
        for x in 0..w {
            self.cool[x] = (MAX / ((META.rows * SY) as f64 * f64::from(self.tall[x]))) as f32;
        }
        let base = (h - 1) * w;
        for x in 0..w {
            self.heat[base + x] = (MAX * f64::from(self.bed[x])) as f32;
        }
        // The spread runs in place, so the last writer wins; alternating the scan
        // keeps that from turning into a wind. Pockets of extra cooling rise with
        // the flame, so near the top the tips tear off and fade on their own.
        for y in 1..h {
            let row = y * w;
            let up = 1.0 - y as f64 / h as f64;
            let flip = (y as u32 + self.n) & 1 == 1;
            for i in 0..w {
                let x = if flip { w - 1 - i } else { i };
                let r = self.rand.next();
                let to = x as i64
                    + if r < 0.3 {
                        -1
                    } else if r > 0.7 {
                        1
                    } else {
                        0
                    };
                if to < 0 || to >= w as i64 {
                    continue;
                }
                let tear = 1.0 + 2.2 * up * up * (noise(x as f64 / 5.0, (y as f64 + n) / 6.0) - 0.45).max(0.0);
                let v = f64::from(self.heat[row + x]) - self.rand.next() * 2.0 * f64::from(self.cool[x]) * tear;
                self.heat[row - w + to as usize] = if v > 0.0 { v as f32 } else { 0.0 };
            }
        }
    }

    /// Steps the spread up to play time `t`, as doom-fire.ts does: at most a
    /// quarter second at once, so a long pause does not run a burst of steps.
    fn advance(&mut self, t: f64) {
        self.acc += (t - self.last).clamp(0.0, 0.25);
        self.last = t;
        while self.acc >= 1.0 / RATE - 1e-6 {
            self.step();
            self.acc -= 1.0 / RATE;
        }
    }

    /// Heat in [0, 1] above the floor, of `k` simulation cells down from `(x, y)`.
    fn level(&self, x: usize, y: usize, k: usize) -> f64 {
        let sum: f64 = (0..k).map(|j| f64::from(self.heat[(y + j) * self.w + x])).sum();
        (sum / (k as f64 * MAX) - FLOOR) / (1.0 - FLOOR)
    }
}

impl Piece for QuadrantFire {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        self.advance(t);
        let (qw, qh) = (META.cols * 2, META.rows * 2);
        // Each quadrant is one simulation column and two rows.
        let level: Vec<f64> = (0..qw * qh).map(|q| self.level(q % qw, (q / qw) * 2, 2)).collect();
        let lit = |x: i64, y: i64| x >= 0 && y >= 0 && x < qw as i64 && y < qh as i64 && level[y as usize * qw + x as usize] > LIT;
        // A faint quadrant only stands with at least two lit neighbours, so
        // the tips keep clean edges instead of a haze of specks (doom-fire.ts
        // does the same with its faint glyphs).
        self.quads.clear();
        let mut heat = vec![(0.0_f64, 0u32); META.cols * META.rows];
        for qy in 0..qh as i64 {
            for qx in 0..qw as i64 {
                let v = level[qy as usize * qw + qx as usize];
                if v <= LIT {
                    continue;
                }
                if v < FAINT {
                    let around =
                        (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).filter(|&(dx, dy)| (dx, dy) != (0, 0) && lit(qx + dx, qy + dy)).count();
                    if around < 2 {
                        continue;
                    }
                }
                self.quads.set(qx, qy);
                let cell = &mut heat[(qy as usize / 2) * META.cols + qx as usize / 2];
                cell.0 += v;
                cell.1 += 1;
            }
        }
        // A cell takes the colour of its lit quadrants' mean heat.
        grid.clear();
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let (sum, count) = heat[cy * META.cols + cx];
                if count > 0 {
                    grid.set(cx as i64, cy as i64, self.quads.glyph(cx, cy), SHADE.at(sum / f64::from(count)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// doom-fire.ts's own drawing of the heat field, as ASCII: only for
    /// checking the simulation against the TS frames.
    fn ascii(fire: &QuadrantFire) -> String {
        const RAMP: &[u8] = b" .:;!+*#%@";
        let (cols, rows) = (META.cols, META.rows);
        let mut lv = vec![0usize; cols * rows];
        for cy in 0..rows {
            for cx in 0..cols {
                let mut sum = 0.0;
                for j in 0..SY {
                    let at = (cy * SY + j) * fire.w + cx * SX;
                    for i in 0..SX {
                        sum += f64::from(fire.heat[at + i]);
                    }
                }
                let v = (sum / ((SX * SY) as f64 * MAX) - FLOOR) / (1.0 - FLOOR);
                lv[cy * cols + cx] = if v <= 0.0 { 0 } else { (RAMP.len() - 1).min(1 + (v * (RAMP.len() - 1) as f64) as usize) };
            }
        }
        let mut lines = vec![];
        for cy in 0..rows {
            let mut line = String::new();
            for cx in 0..cols {
                let mut k = lv[cy * cols + cx];
                if k > 0 && k < 5 {
                    let (mut any, mut under) = (0, 0);
                    for dy in -1i64..=1 {
                        for dx in -1i64..=1 {
                            let (y, x) = (cy as i64 + dy, cx as i64 + dx);
                            if (dy != 0 || dx != 0) && y >= 0 && y < rows as i64 && x >= 0 && x < cols as i64 {
                                let o = lv[y as usize * cols + x as usize];
                                if o > 0 {
                                    any += 1;
                                }
                                if dy >= 0 && o > under {
                                    under = o;
                                }
                            }
                        }
                    }
                    if any < 2 || (k < 3 && under < 3) {
                        k = 0;
                    }
                }
                line.push(RAMP[k] as char);
            }
            lines.push(line);
        }
        lines.join("\n")
    }

    /// `fixtures/doom-fire.txt` holds doom-fire.ts's frames at t = 0, 1.3 and
    /// 7.7 from one instance, captured with `node -e` in ../ascii.
    #[test]
    fn simulation_matches_the_typescript_original() {
        let fixture = include_str!("../../tests/fixtures/doom-fire.txt");
        let mut fire = QuadrantFire::new();
        for block in fixture.split("\n\n") {
            let (head, body) = block.split_once('\n').unwrap();
            let t: f64 = head.strip_prefix("t=").unwrap().parse().unwrap();
            fire.advance(t);
            assert_eq!(ascii(&fire), body.trim_end_matches('\n'), "doom fire at t={t}");
        }
    }

    #[test]
    fn hash_and_prng_match_their_js_bit_tricks() {
        // Expected values from doom-fire.ts's mulberry32 and hash run in node,
        // scaled by 2^32 back to the integers they came from.
        let scale = 4_294_967_296.0;
        let mut r = Mulberry32(9);
        assert_eq!([r.next(), r.next(), r.next()].map(|x| x * scale), [853_534_204.0, 3_655_922_532.0, 608_880_792.0]);
        assert_eq!([hash(3.0, 7.0), hash(120.0, 73.0), hash(0.0, 0.0)].map(|x| x * scale), [2_420_582_120.0, 1_447_284_095.0, 0.0]);
    }
}
