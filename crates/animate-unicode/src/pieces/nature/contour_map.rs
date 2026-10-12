//! contour map: an island drawn as survey lines, with a dotted line offshore,
//! its hills slowly reshaping. After ascii.rest's contour-map.ts (MIT,
//! @bas3line).
//!
//! The island and its drifting swells are ascii.rest's. The lines are traced
//! in braille, at a dot's resolution: the height is taken at every dot's
//! corners and a dot is inked where a contour passes through it, so each line
//! is a dot thick at any angle and runs smooth round every bend, and lines
//! too crowded to part give way to every other one. In colour they are a
//! hypsometric map's, green at the coast through yellow-green, ochre and
//! brown to grey and white on the summits, with the dotted line offshore in
//! the sea's blue.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The contour map's meta.
pub static META: Meta = Meta {
    name: "contour map",
    slug: "contour-map",
    category: Category::Nature,
    note: "an island in braille contours coloured sea to snow, hills reshaping",
    cols: 72,
    rows: 20,
    fps: 8,
    charset: Charset::Extended,
    // The sea's dotted line; then each level up from the coast.
    palette: Some(&["#4a8ad8", "#4cb46a", "#94c858", "#d4c45a", "#cc8c48", "#9a6a4a", "#a8a8b0", "#f4f6fa"]),
    ground: Some("#081018"),
    ..Meta::DEFAULT
};

const SEA: u8 = 0;
const LAND: Ramp = Ramp::new(1, 7);
/// A cell's width and height in ems; the height between neighbouring
/// contour lines; where in the drift the loop is centred, in seconds; the
/// seconds for the hills to come back round; how far each swell's phase
/// wanders; lines closer than this in ems keep only every other one; and the
/// least the swells leave of the hill.
const CW: f64 = 0.6;
const RH: f64 = 1.2;
const LEVEL: f64 = 0.26;
const START: f64 = 280.0;
const PERIOD: f64 = 60.0;
const SWING: f64 = 1.15;
const CROWD: f64 = 0.55;
const FLOOR: f64 = 0.55;
/// Swells laid over one long hill: wavenumbers per em, phase, drift, height,
/// and where in the loop each one wanders, so the shape never just plays backwards.
const WAVES: [[f64; 6]; 5] = [
    [0.21, 0.08, 0.0, 0.055, 0.51, 0.0],
    [-0.13, 0.27, 1.7, -0.035, 0.42, 2.1],
    [0.34, -0.19, 4.1, 0.025, 0.31, 4.0],
    [0.09, 0.41, 2.6, 0.045, 0.25, 1.1],
    [-0.47, -0.12, 5.3, -0.065, 0.17, 5.2],
];

/// The map: the heights at every dot corner, the sea, and the dots.
pub struct ContourMap {
    h: Vec<f64>,
    sea: Vec<bool>,
    dots: ColorDots,
}

/// A contour map; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    Box::new(ContourMap { h: vec![0.0; (w + 1) * (h + 1)], sea: vec![false; w * h], dots })
}

/// The hill falls to nothing at the frame's edges, so every line closes
/// inside it. The swells never cut it below FLOOR, so the coast keeps no deep inlets.
fn height(x: f64, y: f64, phases: &[f64; 5]) -> f64 {
    let (hx, hy) = (META.cols as f64 / 2.0 * CW, META.rows as f64 / 2.0 * RH);
    let mut n = 1.0;
    for (wave, ph) in WAVES.iter().zip(phases) {
        n += wave[4] * (wave[0] * x + wave[1] * y + wave[2] + ph).sin();
    }
    n = FLOOR + (6.0 * (n - FLOOR)).exp().ln_1p() / 6.0;
    (-(x / 17.0).powi(2) - (y / 8.0).powi(2)).exp() * (1.0 - (x / hx).powi(2)) * (1.0 - (y / hy).powi(2)) * n
}

impl Piece for ContourMap {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let turn = TAU * t / PERIOD;
        let phases: [f64; 5] = std::array::from_fn(|i| WAVES[i][3] * START + WAVES[i][3].signum() * SWING * (turn + WAVES[i][5]).sin());
        let wc = w + 1;
        for j in 0..=h {
            for i in 0..=w {
                self.h[j * wc + i] = height((i as f64 / 2.0 - cols / 2.0) * CW, (j as f64 / 4.0 - rows / 2.0) * RH, &phases);
            }
        }
        let corners = |i: usize, j: usize| [self.h[j * wc + i], self.h[j * wc + i + 1], self.h[(j + 1) * wc + i], self.h[(j + 1) * wc + i + 1]];
        // The sea is the low ground reached from the frame's edge; a hollow inland is not.
        self.sea.fill(false);
        let low = |i: usize, j: usize| corners(i, j).iter().sum::<f64>() / 4.0 < LEVEL / 2.0;
        let mut stack: Vec<(usize, usize)> = vec![];
        for j in 0..h {
            for i in 0..w {
                if (i == 0 || j == 0 || i == w - 1 || j == h - 1) && low(i, j) {
                    self.sea[j * w + i] = true;
                    stack.push((i, j));
                }
            }
        }
        while let Some((i, j)) = stack.pop() {
            for (ni, nj) in [(i.wrapping_sub(1), j), (i + 1, j), (i, j.wrapping_sub(1)), (i, j + 1)] {
                if ni < w && nj < h && !self.sea[nj * w + ni] && low(ni, nj) {
                    self.sea[nj * w + ni] = true;
                    stack.push((ni, nj));
                }
            }
        }
        self.dots.clear();
        for j in 0..h {
            for i in 0..w {
                let c = corners(i, j);
                let (lo, hi) = c.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
                let mid = c.iter().sum::<f64>() / 4.0;
                let level = (mid / LEVEL).round() as i64;
                if level >= 1 && lo < level as f64 * LEVEL && hi >= level as f64 * LEVEL {
                    // How far apart the lines run here, in ems; where they crowd,
                    // every other one gives way. The coast never does.
                    let (gx, gy) = ((c[1] + c[3] - c[0] - c[2]) / (2.0 * CW / 2.0), (c[2] + c[3] - c[0] - c[1]) / (2.0 * RH / 4.0));
                    let gap = LEVEL / gx.hypot(gy).max(1e-9);
                    if gap > CROWD || level % 2 == 1 {
                        self.dots.set(i as i64, j as i64, LAND.at((level - 1) as f64 / 6.0));
                    }
                } else if lo < LEVEL / 2.0 && hi >= LEVEL / 2.0 && (i + j) % 3 == 0 {
                    // The dotted line runs only along the open sea, not round a hollow inland.
                    let near_sea = [(0i64, 0i64), (-1, 0), (1, 0), (0, -1), (0, 1)].iter().any(|&(di, dj)| {
                        let (ni, nj) = (i as i64 + di, j as i64 + dj);
                        ni >= 0 && nj >= 0 && (ni as usize) < w && (nj as usize) < h && self.sea[nj as usize * w + ni as usize]
                    });
                    if near_sea {
                        self.dots.set(i as i64, j as i64, SEA);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
