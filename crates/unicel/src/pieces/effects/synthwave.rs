//! synthwave: the eighties horizon. A grid floor rolls toward the viewer
//! under a setting sun cut by thinning stripes, behind a ridge of mountains.
//! After ascii.rest's synthwave.ts (MIT, @bas3line).
//!
//! The grid's perspective, the sun's sinking stripes and the ridge are
//! ascii.rest's. The sun and the ridge are octants: the stripes are gaps a
//! dot high that sink and widen, and the ridge an outline that hides the sun
//! behind it. The floor is braille, its cross lines and rails fine lines at
//! a dot's resolution, so they meet at the vanishing point without
//! crowding. In colour it is neon: the sun gold through orange to hot pink,
//! the ridge outlined in cyan, the grid magenta glowing pink at the horizon.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// Synthwave's meta.
pub static META: Meta = Meta {
    name: "synthwave",
    slug: "synthwave",
    category: Category::Effects,
    note: "a neon braille grid rolling toward an octant sun striped gold to pink",
    cols: 65,
    rows: 28,
    fps: 20,
    charset: Charset::Extended,
    // The sun, top to bottom; the ridge; the grid; its haze at the horizon; a star.
    palette: Some(&["#ffe04a", "#ffb03a", "#ff7a4a", "#ff4a7a", "#ff2a9a", "#3ae8ff", "#e03ad8", "#ff7ad8", "#c8c0f0"]),
    ground: Some("#0c0418"),
    ..Meta::DEFAULT
};

const SUN: Ramp = Ramp::new(0, 5);
const RIDGE: u8 = 5;
const GRID: u8 = 6;
const HAZE: u8 = 7;
const STAR: u8 = 8;
/// The horizon (the first row of floor); the sun's radius and centre in half
/// rows; cross line n sits K / n rows below the horizon; rail k crosses k S
/// columns a row; rails each side of the middle one; cross lines passed a second.
const HZ: f64 = 12.0;
const SUN_R: f64 = 13.0;
const SUN_Y: f64 = 2.0 * HZ - 6.0;
const K: f64 = 24.0;
const S: f64 = 0.7;
const RAILS: i32 = 6;
const SPEED: f64 = 0.9;

/// The horizon: the ridge's heights, the stars, and the two canvases.
pub struct Synthwave {
    ridge: Vec<f64>,
    stars: Vec<(i64, i64, f64)>,
    sky: ColorOctants,
    floor: ColorDots,
}

/// Synthwave; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let cols = META.cols;
    let cx = cols as f64 / 2.0;
    // The ridge: heights in rows at each column boundary, a walk that climbs
    // toward the edges and keeps low under the sun.
    let mut rand = Mulberry32::new(5);
    let mut ridge = vec![6.0];
    for c in 1..=cols {
        let d = (c as f64 - cx).abs() / cx;
        let want = 1.0 + 6.0 * (d - 0.25) + 1.5 * (c as f64 * 0.45).sin();
        let h: f64 = ridge[c - 1];
        let r = rand.next_f64();
        let mut step = if r < 0.6 {
            (want - h).round().signum()
        } else if r < 0.8 {
            1.0
        } else {
            -1.0
        };
        if d < 0.25 {
            step = if h > 0.0 { -1.0 } else { 0.0 };
        } else if h + step < 0.0 || h + step > 7.0 {
            step = 0.0;
        }
        ridge.push(h + step);
    }
    let stars = (0..24).map(|_| ((rand.next_f64() * cols as f64) as i64 * 2, (rand.next_f64() * (HZ - 3.0)) as i64 * 4 + 1, rand.next_f64())).collect();
    Box::new(Synthwave { ridge, stars, sky: ColorOctants::new(META.cols, META.rows), floor: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Synthwave {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let cx = cols / 2.0;
        let (w, h) = (self.sky.width() as i64, self.sky.height() as i64);
        let hz = (HZ * 4.0) as i64;
        self.sky.clear();
        for &(x, y, p) in &self.stars {
            if (t * (0.8 + p) + p * 30.0).sin() > -0.4 {
                self.sky.set(x, y, STAR);
            }
        }
        // The ridge between columns, eased, as a height in rows at each dot.
        let ridge_at = |x: f64| {
            let c = (x.floor() as usize).min(self.ridge.len() - 2);
            let f = x - c as f64;
            self.ridge[c] * (1.0 - f) + self.ridge[c + 1] * f
        };
        // The sun, its lower half cut by gaps a dot high that sink, closer
        // together toward the horizon; the ridge hides it.
        let s = t * 0.6;
        let band = |k: f64| 0.2 * k + 0.3 * k * k / 26.0;
        for j in 0..hz {
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0;
                let y = (j as f64 + 0.5) / 4.0;
                if y > HZ - ridge_at(x) {
                    continue;
                }
                let p = y * 2.0; // half rows
                let (dx, dy) = (x - cx, p - SUN_Y);
                if dx * dx + dy * dy > SUN_R * SUN_R {
                    continue;
                }
                let k = p - (SUN_Y - 7.0);
                if k >= 0.0 && (band(k + 0.5) - s).floor() != (band(k) - s).floor() {
                    continue;
                }
                self.sky.set(i, j, SUN.at((p - (SUN_Y - SUN_R)) / (2.0 * SUN_R)));
            }
        }
        // The ridge's outline.
        for i in 0..w {
            let x = (i as f64 + 0.5) / 2.0;
            let (a, b) = (HZ - ridge_at(x), HZ - ridge_at(x + 0.5));
            if ridge_at(x) > 0.05 || ridge_at(x + 0.5) > 0.05 {
                for j in (a.min(b) * 4.0) as i64..=(a.max(b) * 4.0) as i64 {
                    self.sky.set(i, j.min(hz - 1), RIDGE);
                }
            }
        }
        // The floor: cross lines K / n rows down come on toward the viewer,
        // and rails fan from the vanishing point.
        self.floor.clear();
        let n0 = (t * SPEED).rem_euclid(1.0);
        for n in 1..400 {
            let e = K / (f64::from(n) - n0);
            if e >= rows - HZ {
                continue;
            }
            if e < 0.4 {
                break;
            }
            // Where the lines come within a couple of dots of each other, only
            // every second or fourth is drawn, so they thin into the haze.
            let gap = K / (f64::from(n) - n0).powi(2) * 4.0; // dots to the next line
            let every = if gap >= 2.0 {
                1
            } else if gap >= 1.0 {
                2
            } else {
                4
            };
            if n % every != 0 {
                continue;
            }
            let j = ((HZ + e) * 4.0) as i64;
            for i in 0..w {
                // Dashed where they crowd at the horizon, so they read as haze.
                if e > 1.5 || i % 2 == 0 {
                    self.floor.set(i, j, if e < 1.5 { HAZE } else { GRID });
                }
            }
        }
        let vx = (cx.floor() + 0.5) * 2.0;
        for m in -RAILS..=RAILS {
            let sl = f64::from(m) * S * 2.0 / 4.0; // dots across a dot down
            for j in hz..h {
                let x = vx + sl * (j - hz) as f64;
                if (0.0..w as f64).contains(&x) {
                    self.floor.set(x as i64, j, GRID);
                }
            }
        }
        for i in 0..w {
            self.floor.set(i, hz, HAZE);
        }
        grid.clear();
        self.sky.draw(grid, 0, 0);
        self.floor.draw(grid, 0, 0);
    }
}
