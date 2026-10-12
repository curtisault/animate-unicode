//! aurora: a curtain of light over a spruce treeline at night. Its lower hem
//! is brightest and ripples; streaks rise from it and fade out below a clear
//! sky of stars, and light drifts along it in slow surges. After ascii.rest's
//! aurora.ts (MIT, @bas3line).
//!
//! The curtain, the treeline and the stars are ascii.rest's. The light is
//! drawn in sextants, two dots across a cell and three down, dithered, so the
//! streaks stand as fine upright strokes. In colour it is an aurora's: the
//! hem the bright green of oxygen low down, the streaks shading up through
//! teal and violet to a magenta crown, the glow green on the treetops, under
//! twinkling stars. It is light, so on paper it prints as a negative.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorSextants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The aurora's meta.
pub static META: Meta = Meta {
    name: "aurora",
    slug: "aurora",
    category: Category::Nature,
    note: "a sextant aurora rippling green to magenta over spruces, under stars",
    cols: 64,
    rows: 20,
    fps: 20,
    charset: Charset::Extended,
    // The hem, the green above it, teal, violet, the magenta crown; the glow on
    // the trees; the trees and ridge; a star; a bright star.
    palette: Some(&["#b8ffcc", "#3cf07a", "#2ab8a8", "#7a5ad8", "#d046b8", "#1e8a54", "#14221c", "#8a92b0", "#e8ecff"]),
    ground: Some("#03050b"),
    ..Meta::DEFAULT
};

const HEM_LIT: u8 = 0;
const GREEN: u8 = 1;
const TEAL: u8 = 2;
const VIOLET: u8 = 3;
const CROWN: u8 = 4;
const GLOW: u8 = 5;
const TREES: u8 = 6;
const STAR: u8 = 7;
const BRIGHT: u8 = 8;
/// The row the hem ripples about, and the row the trees stand on.
const HEM: f64 = 12.0;
const RIDGE: i64 = 18;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn noise(x: f64, y: f64) -> f64 {
    let (i, j) = (x.floor() as i64, y.floor() as i64);
    let (u, v) = (ease(x - x.floor()), ease(y - y.floor()));
    let (a, b, c, d) = (hash(i, j), hash(i + 1, j), hash(i, j + 1), hash(i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// The scene: the treeline's glyphs, the stars (dot, bright, rate, phase), and the dots.
pub struct Aurora {
    land: Vec<Option<char>>,
    stars: Vec<(i64, i64, bool, f64, f64)>,
    dots: ColorSextants,
}

/// An aurora; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let mut rand = Mulberry32::new(5);
    // The treeline: spruces in stepped quarter blocks, a tier of branches
    // every two rows, each standing apart with sky between, on a low ridge.
    let mut land = vec![None; (cols * rows) as usize];
    for c in 0..cols {
        land[(c + RIDGE * cols) as usize] = Some(if (c as f64 * 0.13 + 1.0).sin() > 0.3 { '█' } else { '▄' });
        for r in RIDGE + 1..rows {
            land[(c + r * cols) as usize] = Some('█');
        }
    }
    let mut x = 1 + (rand.next_f64() * 2.0) as i64;
    while x < cols - 1 {
        let (a, b, c) = (rand.next_f64(), rand.next_f64(), rand.next_f64());
        let tall = 2 + (a * 2.2 + b * c * 2.5) as i64; // 2 to 5 rows
        for k in 0..tall {
            let (r, half) = (RIDGE - tall + k, 1 + (k >> 1));
            for c in (x - half + 1).max(0)..=(x + half).min(cols - 1) {
                let (left, right) = (c == x - half + 1, c == x + half);
                land[(c + r * cols) as usize] = Some(if left {
                    if k & 1 == 1 {
                        '▟'
                    } else {
                        '▗'
                    }
                } else if right {
                    if k & 1 == 1 {
                        '▙'
                    } else {
                        '▖'
                    }
                } else {
                    '█'
                });
            }
        }
        for c in (x - (tall >> 1)).max(0)..=(x + 1 + (tall >> 1)).min(cols - 1) {
            land[(c + RIDGE * cols) as usize] = Some('█');
        }
        x += 2 + 2 * (tall >> 1) + 1 + (rand.next_f64() * 3.0) as i64;
    }
    // Stars, a few of them bright, each twinkling on its own beat.
    let stars = (0..40)
        .map(|_| {
            let c = (rand.next_f64() * cols as f64) as i64;
            let r = (rand.next_f64() * rand.next_f64() * (HEM + 2.0)) as i64;
            (c * 2 + (r & 1), r * 3 + 1, rand.next_f64() < 0.2, 0.3 + rand.next_f64() * 1.1, rand.next_f64() * TAU)
        })
        .collect();
    Box::new(Aurora { land, stars, dots: ColorSextants::new(META.cols, META.rows) })
}

impl Piece for Aurora {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let cols = META.cols as i64;
        let (w, h) = (self.dots.width(), self.dots.height());
        self.dots.clear();
        for i in 0..w {
            let x = (i as f64 + 0.5) / 2.0;
            // The hem: a slow fold and a quicker ripple running along it.
            let y0 = HEM + 1.2 * (x * 0.12 + t * 0.35).sin() + 0.55 * (x * 0.31 - t * 0.7 + 1.0).sin();
            let slope = 1.2 * 0.12 * (x * 0.12 + t * 0.35).cos() + 0.55 * 0.31 * (x * 0.31 - t * 0.7 + 1.0).cos();
            // Light surges along the curtain but never leaves it, and gathers
            // where a fold is seen edge on.
            let lit = (0.62 + 0.4 * ease(noise(x * 0.07 - t * 0.25, 3.3))) * (1.0 + 0.9 * (slope.abs() * 2.0).min(1.0));
            // Streaks stand on the hem, sliding along it, each fading as it rises.
            let ray = ease(((noise(x * 0.9 - t * 1.4, 7.0 + t * 0.3) - 0.3) / 0.45).clamp(0.0, 1.0));
            let height = 1.5 + 6.5 * ray * (0.6 + 0.6 * noise(x * 0.12 + t * 0.15, 11.0));
            for j in 0..h {
                let d = y0 - (j as f64 + 0.5) / 3.0; // rows above the hem
                                                     // A bright hem, the streaks above it, and below it a glow that
                                                     // settles on the treetops, striped like the streaks.
                let hem = (-((d.abs() - 0.5).max(0.0) / if d > 0.0 { 0.6 } else { 0.35 }).powi(2)).exp();
                let above = if d >= 0.0 { (0.3 + 0.7 * ray) * (1.0 - d / height).max(0.0).powf(1.4) } else { (d / 1.5).exp() * (0.4 + 0.4 * ray) };
                let b = lit * hem.max(above) - 0.12;
                if b <= 0.0 {
                    continue;
                }
                let v = (b * 1.1).min(1.0);
                if dither(i, j, if env.paper { v } else { 0.15 + 0.85 * v }) {
                    let color = if d < 0.0 {
                        GLOW
                    } else if hem > 0.8 && v > 0.7 {
                        HEM_LIT
                    } else if d < 2.0 {
                        GREEN
                    } else if d < 3.5 {
                        TEAL
                    } else if d < 5.0 {
                        VIOLET
                    } else {
                        CROWN
                    };
                    self.dots.set(i as i64, j as i64, color);
                }
            }
        }
        for &(x, y, bright, rate, ph) in &self.stars {
            let tw = (t * rate * TAU + ph).sin();
            if self.dots.get(x, y).is_none() && (bright || tw > -0.6) {
                self.dots.set(x, y, if bright && tw > 0.3 { BRIGHT } else { STAR });
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for (k, ch) in self.land.iter().enumerate() {
            if let Some(ch) = ch {
                grid.set(k as i64 % cols, k as i64 / cols, *ch, TREES);
            }
        }
    }
}
