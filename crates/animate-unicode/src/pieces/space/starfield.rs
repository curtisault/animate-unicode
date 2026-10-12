//! starfield: flying through stars at warp. Each star drifts out from the
//! vanishing point as a dot, then streaks as it rushes past the edge. After
//! ascii.rest's starfield.ts (MIT, @bas3line).
//!
//! The flight is ascii.rest's, star for star; the streaks are braille, a dot
//! wide and as long as the star moved while the shutter was open, so they
//! lengthen smoothly as a star nears and thin out toward their tails. In
//! colour the stars are warm ahead, amber-white round the vanishing point,
//! and cool to blue as they rush out to the edges.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The starfield's meta.
pub static META: Meta = Meta {
    name: "starfield",
    slug: "starfield",
    category: Category::Space,
    note: "braille stars streaking past at warp, warm ahead and blue at the edges",
    cols: 64,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"stars": 60, "speed": 1}"#),
    // Ahead to the edges: amber, warm white, white, pale blue, blue.
    palette: Some(&["#ffc87a", "#fff0d6", "#eef2ff", "#a6c2ff", "#6a8cff"]),
    ground: Some("#03040a"),
    ..Meta::DEFAULT
};

const HUE: Ramp = Ramp::new(0, 5);
/// Focal length in columns per unit at depth 1; the near and far planes; the
/// depth at which a star shows; and the depth the shutter spans.
const F: f64 = 10.0;
const NEAR: f64 = 0.04;
const FAR: f64 = 1.3;
const SEEN: f64 = 1.05;
const SHUTTER: f64 = 0.16;

/// The starfield: each star's phase, the seconds from far plane to near, how many, and the dots.
pub struct Starfield {
    phase: Vec<f64>,
    life: f64,
    count: usize,
    dots: ColorDots,
}

/// A starfield; reads `stars` and `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let count = options.f64("stars", 60.0).clamp(1.0, 400.0) as usize;
    let mut rand = Mulberry32::new(7);
    let phase = (0..count).map(|_| rand.next_f64()).collect();
    Box::new(Starfield { phase, life: 4.0 / options.f64("speed", 1.0).max(0.01), count, dots: ColorDots::new(META.cols, META.rows) })
}

impl Starfield {
    /// Where star i is on its k-th pass: a direction inside its own slice of
    /// the circle, so the field stays even, and a distance off the axis that
    /// is never too small, so the vanishing point stays clear.
    fn place(&self, i: usize, k: i64) -> (f64, f64) {
        let mut r = Mulberry32::new((i as i64 * 7919 + k * 104_729 + 13) as u32);
        let a = (i as f64 + r.next_f64()) / self.count as f64 * TAU;
        let d = 0.25 + 0.75 * r.next_f64().sqrt();
        (a.cos() * d, a.sin() * d)
    }
}

impl Piece for Starfield {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
        let mut order: Vec<(f64, usize, i64)> = (0..self.count)
            .map(|i| {
                let u = self.phase[i] + t / self.life;
                (FAR - (FAR - NEAR) * (u - u.floor()), i, u.floor() as i64)
            })
            .collect();
        // Far stars first, so the nearer ones are drawn over them.
        order.sort_by(|a, b| b.0.total_cmp(&a.0));
        self.dots.clear();
        for (z, i, k) in order {
            if z > SEEN {
                continue;
            }
            let (x, y) = self.place(i, k);
            // In dots: two a column, four a row, and a row is half as tall in the picture.
            let head = ((cx + x / z * F) * 2.0, (cy + y / z * F * 0.5) * 4.0);
            let zt = z + SHUTTER;
            let tail = ((cx + x / zt * F) * 2.0, (cy + y / zt * F * 0.5) * 4.0);
            // Warm ahead, cold at the edges, by how far out the head is.
            let out = ((head.0 / 2.0 - cx) / cx).hypot((head.1 / 4.0 - cy) / cy);
            let color = HUE.at(out / 1.1);
            if z > 0.5 {
                // Far stars are single dots, and the farthest blink in and out of sight.
                if z < 0.9 || dither(head.0 as usize, head.1 as usize, (SEEN - z) * 4.0) {
                    self.dots.set(head.0.floor() as i64, head.1.floor() as i64, color);
                }
                continue;
            }
            // Tail to head, a dot a step, thinning toward the tail.
            let n = (head.0 - tail.0).abs().max((head.1 - tail.1).abs()).ceil().max(1.0) as i64;
            for s in 0..=n {
                let f = s as f64 / n as f64;
                let (dx, dy) = (tail.0 + (head.0 - tail.0) * f, tail.1 + (head.1 - tail.1) * f);
                if f > 0.85 || dither(dx as usize, dy as usize, 0.2 + f) {
                    self.dots.set(dx.floor() as i64, dy.floor() as i64, color);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
