//! flag: a striped flag on a pole in a steady wind. Folds run from the pole
//! to the free edge and grow as they go; faces turned to the light are
//! bright, the far sides of folds dark. After ascii.rest's flag.ts (MIT,
//! @bas3line).
//!
//! The cloth is ascii.rest's, strip by strip, laid down per octant dot, eight
//! a character, so the hems ripple a quarter of a row at a time and the folds
//! are shaded by ordered dithering. The stripes are in colour, crimson and
//! cream, each in four shades of the light its fold catches.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The flag's meta.
pub static META: Meta = Meta {
    name: "flag",
    slug: "flag",
    category: Category::Physics,
    note: "a striped flag in octants, crimson and cream, lit by its folds",
    cols: 62,
    rows: 21,
    fps: 30,
    charset: Charset::Extended,
    // Crimson, then cream, four shades each; the pole.
    palette: Some(&["#4a0a14", "#86121f", "#c21f30", "#ec4a52", "#5a5040", "#9a8c70", "#d8c8a0", "#fff4dc", "#9aa0aa"]),
    ground: Some("#0b0c10"),
    options: Some(r#"{"stripes": 5}"#),
    ..Meta::DEFAULT
};

/// Seconds a loop; every rate below fits a whole number of times.
const LOOP: f64 = 12.0;
/// Folds along the cloth, and folds passing a point each second.
const WAVES: f64 = 2.3;
const RATE: f64 = 17.0 / LOOP;
/// Radians the folds lag from the top hem to the bottom one, so they run on a slant.
const SKEW: f64 = 1.4;
/// The cloth's length and its height at the pole, in rows (a column is half a row).
const LONG: f64 = 29.0;
const HIGH: f64 = 13.0;
/// How far the free edge billows toward and away from us, in rows.
const SWING: f64 = 4.0;
const STRIPS: usize = 600;
/// The pole's column, and the top hem at the pole, in rows.
const POLE: usize = 4;
const TOP: f64 = 2.5;
const POLE_INK: u8 = 8;

/// The flag: its stripes, the strips' places across each dot row, the light
/// gathered in each dot, and the dots.
pub struct Flag {
    bands: usize,
    xs: Vec<f64>,
    lit: Vec<f32>,
    hits: Vec<u16>,
    odd: Vec<u16>,
    dots: ColorOctants,
}

/// A flag; reads `stripes` (2 or more).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    let bands = options.f64("stripes", 5.0).round().max(2.0) as usize;
    Box::new(Flag { bands, xs: vec![0.0; (STRIPS + 1) * dots.height()], lit: vec![0.0; n], hits: vec![0; n], odd: vec![0; n], dots })
}

impl Piece for Flag {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let gust = 0.85 + 0.15 * (TAU * t / LOOP).sin();
        let ph = |u: f64, v: f64| TAU * (WAVES * u - RATE * t) - SKEW * v;
        // How far the cloth stands out of its plane, and its slope along the length.
        let z = |u: f64, v: f64| gust * SWING * u * ph(u, v).sin();
        let dz = |u: f64, v: f64| gust * SWING * (ph(u, v).sin() + u * TAU * WAVES * ph(u, v).cos()) / LONG;
        // The cloth can't stretch, so a strip turned away from us takes less
        // room across the frame. Places in columns, a dot row at a time.
        for y in 0..h {
            let row = (y as f64 + 0.5) / 4.0;
            let v = ((row - TOP) / HIGH).clamp(0.0, 1.0);
            self.xs[y] = POLE as f64 + 1.0;
            for i in 1..=STRIPS {
                let s = dz((i as f64 - 0.5) / STRIPS as f64, v);
                self.xs[i * h + y] = self.xs[(i - 1) * h + y] + 2.0 * LONG / STRIPS as f64 / (1.0 + s * s).sqrt();
            }
        }
        self.lit.fill(0.0);
        self.hits.fill(0);
        self.odd.fill(0);
        for i in 0..=STRIPS {
            let u = i as f64 / STRIPS as f64;
            // Nearer parts look taller, the hems rise and fall with each fold, and the free end droops.
            let zm = z(u, 0.5);
            let tall = HIGH * (1.0 + 0.04 * zm) - 0.8 * u;
            let top = TOP + 1.2 * u * u + 0.8 * u * gust * (ph(u, 0.5) + 0.8).sin() - (tall - HIGH) / 2.0;
            let (y0, y1) = (((top * 4.0).floor() as i64).max(0), (((top + tall) * 4.0).ceil() as i64).min(h as i64));
            for y in y0..y1 {
                let yu = y as usize;
                let row = (y as f64 + 0.5) / 4.0;
                if row < top || row > top + tall {
                    continue;
                }
                let x = (self.xs[i * h + yu] * 2.0).floor() as usize;
                if x >= w {
                    continue;
                }
                let k = yu * w + x;
                let v = ((row - top) / tall).clamp(0.0, 1.0);
                // Light comes from the left and in front, so a face turned toward the pole is lit.
                let s = dz(u, v);
                self.lit[k] += ((0.95 * s + 0.65) / (1.0 + s * s).sqrt() + 0.1).clamp(0.0, 1.0) as f32;
                self.hits[k] += 1;
                self.odd[k] += ((v * self.bands as f64 * 0.9999).floor() as u16) % 2;
            }
        }
        self.dots.clear();
        for k in 0..w * h {
            if self.hits[k] == 0 {
                continue;
            }
            let l = f64::from(self.lit[k]) / f64::from(self.hits[k]);
            let cream = self.odd[k] * 2 > self.hits[k];
            let (x, y) = (k % w, k / w);
            // Lit faces are dense, the far sides of folds sparse (on paper the other way).
            let ink = if env.paper { 1.0 - 0.6 * l } else { 0.35 + 0.65 * l };
            if dither(x, y, ink) {
                self.dots.set(x as i64, y as i64, Ramp::new(if cream { 4 } else { 0 }, 4).at(l));
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The pole, its cap, and the ground it stands in.
        let (pole, last) = (POLE as i64, META.rows as i64 - 1);
        grid.set(pole, TOP as i64 - 1, '●', POLE_INK);
        for r in TOP as i64..last {
            grid.set(pole, r, '┃', POLE_INK);
        }
        for c in pole - 3..=pole + 3 {
            grid.set(c, last, if c == pole { '┸' } else { '▁' }, POLE_INK);
        }
    }
}
