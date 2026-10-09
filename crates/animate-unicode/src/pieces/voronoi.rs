//! voronoi: the plane split into cells, each the ground nearest one drifting
//! seed. After ascii.rest's voronoi.ts (MIT, @bas3line).
//!
//! The seeds and their orbits are ascii.rest's, from its seeded generator.
//! Every octant dot, eight a character, finds its nearest seed, so the 64×22
//! characters are a 128×88 map. Each cell has its own colour, brighter and
//! denser toward its seed; the borders are the dots nearly as close to a
//! second seed as to the first, left dark, so every edge is sharp at the
//! dots' resolution.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Voronoi's meta.
pub static META: Meta = Meta {
    name: "voronoi",
    slug: "voronoi",
    category: Category::Generative,
    note: "voronoi cells in octants, each its own colour, sharp-edged",
    cols: 64,
    rows: 22,
    fps: 15,
    charset: Charset::Extended,
    // Thirteen hues, each dim then bright; then the seeds.
    palette: Some(&[
        "#7a2a3a", "#e05a72", "#7a4a1e", "#ec9a4a", "#6a6a1a", "#d8d24e", "#2e6a2a", "#6ad866", "#1e6a5a", "#4ad8b8", "#1e5a7a", "#4ab8ec", "#2a3a8a",
        "#6a86f0", "#4a2a8a", "#9a6af0", "#6a2a7a", "#d06ae4", "#7a2a5a", "#ec5ab0", "#5a4a3a", "#b8a088", "#3a5a4a", "#88c0a4", "#4a4a6a", "#a0a0d8",
        "#ffffff",
    ]),
    ground: Some("#0b0b10"),
    ..Meta::DEFAULT
};

/// A character cell, in ems.
const CW: f64 = 0.6;
const RH: f64 = 1.2;
/// Seconds for every seed to come back round.
const PERIOD: f64 = 30.0;
/// Past this ratio of nearest to second-nearest distance, a dot is border.
const EDGE: f64 = 0.9;
const SEED: u8 = 26;

/// A seed: its home, the radius it circles at, its turns a period and its phase.
struct Seed {
    hx: f64,
    hy: f64,
    r: f64,
    turns: f64,
    p: f64,
}

/// The seeds, where they are this frame, and the dots.
pub struct Voronoi {
    seeds: Vec<Seed>,
    at: Vec<(f64, f64)>,
    dots: ColorOctants,
}

/// Voronoi; it takes no options. The phase's 6.283 is ascii.rest's, kept so
/// the seeds start where its do.
#[allow(clippy::approx_constant)]
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (x, y) = (META.cols as f64 * CW, META.rows as f64 * RH);
    // Homes on a staggered grid of 4, 5 and 4; each seed circles its home a
    // whole number of times a period, so the loop closes.
    let mut rand = Mulberry32::new(7);
    let mut seeds = vec![];
    for (row, n) in [4, 5, 4].into_iter().enumerate() {
        for i in 0..n {
            let hx = (i as f64 + 0.5 + (rand.next_f64() - 0.5) * 0.5) / n as f64 * x;
            let hy = (row as f64 + 0.5 + (rand.next_f64() - 0.5) * 0.4) / 3.0 * y;
            let r = 2.2 + rand.next_f64() * 1.4;
            let turns = (if rand.next_f64() < 0.5 { 1.0 } else { -1.0 }) * (1.0 + (rand.next_f64() * 2.0).floor());
            let p = rand.next_f64() * 6.283;
            seeds.push(Seed { hx: hx.clamp(r + 1.0, x - r - 1.0), hy: hy.clamp(0.8 * r + 0.8, y - 0.8 * r - 0.8), r, turns, p });
        }
    }
    let n = seeds.len();
    Box::new(Voronoi { seeds, at: vec![(0.0, 0.0); n], dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Voronoi {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let a = TAU * t / PERIOD;
        for (s, at) in self.seeds.iter().zip(&mut self.at) {
            *at = (s.hx + s.r * (s.turns * a + s.p).cos(), s.hy + s.r * 0.8 * (s.turns * a + s.p).sin());
        }
        let (w, h) = (self.dots.width(), self.dots.height());
        // A dot is 0.3 em each way: half a cell across, a quarter down.
        let (dx, dy) = (CW / 2.0, RH / 4.0);
        self.dots.clear();
        for py in 0..h {
            let y = (py as f64 + 0.5) * dy;
            for px in 0..w {
                let x = (px as f64 + 0.5) * dx;
                let (mut d1, mut d2, mut near) = (f64::MAX, f64::MAX, 0);
                for (i, &(sx, sy)) in self.at.iter().enumerate() {
                    let d = (x - sx).powi(2) + (y - sy).powi(2);
                    if d < d1 {
                        (d2, d1, near) = (d1, d, i);
                    } else if d < d2 {
                        d2 = d;
                    }
                }
                // 0 at the seed, 1 on the border.
                let f = (d1 / d2).sqrt();
                if f > EDGE {
                    continue;
                }
                // Denser toward the seed (on paper, toward the rim, so the seed is clear).
                let ink = if env.paper { 0.2 + 0.7 * f } else { 0.95 - 0.65 * f };
                if dither(px, py, ink) {
                    self.dots.set(px as i64, py as i64, (near * 2 + usize::from(f < 0.45)) as u8);
                }
            }
        }
        // Each seed, a bright knot.
        for &(sx, sy) in &self.at {
            let (x, y) = ((sx / dx).floor() as i64, (sy / dy).floor() as i64);
            for (ox, oy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.dots.set(x + ox - 1, y + oy - 1, SEED);
            }
        }
        grid.clear();
        self.dots.draw_split(grid, 0, 0);
    }
}
