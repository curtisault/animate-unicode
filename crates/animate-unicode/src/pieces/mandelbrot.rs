//! mandelbrot: the Mandelbrot set, zooming slowly into seahorse valley, the
//! crack between the main cardioid and the disc to its left, then back out.
//! After ascii.rest's mandelbrot.ts (MIT, @bas3line).
//!
//! Worked out once per octant dot, eight a cell, so the 64×26 cells are a
//! 128×104 picture. The glow on the boundary sets how many dots are lit, by
//! ordered dithering, and each dot takes its colour from its smooth escape
//! time through a palette that cycles, so the bands round the set run in
//! colour as the zoom goes in.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The Mandelbrot set's meta.
pub static META: Meta = Meta {
    name: "mandelbrot",
    slug: "mandelbrot",
    category: Category::Generative,
    note: "a zoom into seahorse valley in octants, its bands in colour",
    cols: 64,
    rows: 26,
    fps: 15,
    charset: Charset::Extended,
    // A cycle the bands run through.
    palette: Some(&["#1d3b8f", "#2f6ec2", "#5aa8e0", "#a8e0f0", "#f4f0d0", "#f6c86a", "#ec8a3a", "#c4502e", "#8a2a4a", "#4f1f6a"]),
    ground: Some("#05060c"),
    ..Meta::DEFAULT
};

/// Seconds to zoom in and back out; start just after the turn, so the zoom
/// is already under way.
const P: f64 = 48.0;
const T0: f64 = 1.5;
/// The deepest zoom: one seahorse's curled tail.
const DEEP: f64 = 8.699_514_748_210_191; // ln 6000
/// Where the zoom heads: the centre of that curl.
const TX: f64 = -0.74435;
const TY: f64 = 0.12119;
/// The centre and width of the whole set.
const CX: f64 = -0.75;
const SPAN: f64 = 3.0;
/// Octaves of cells out from the set over which the glow fades, wide then narrow.
const WIDE: f64 = 3.0;
const NARROW: f64 = 1.2;
/// Escape-time steps from one palette colour to the next.
const BAND: f64 = 6.0;
const COLORS: usize = 10;

/// The Mandelbrot set's dots.
pub struct Mandelbrot {
    dots: ColorOctants,
}

/// The Mandelbrot set; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Mandelbrot { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Mandelbrot {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let z = (DEEP * (1.0 - (TAU * (t + T0) / P).cos()) / 2.0).exp();
        // Moving the centre with the zoom keeps the target still on screen.
        let (cx, cy) = (TX + (CX - TX) / z, TY - TY / z);
        let dx = SPAN / z / w as f64; // dots are square
        let cell = dx * 2.0; // the glow is measured in cells, as ascii.rest's
        let max = (120.0 + 60.0 * z.log2()).round() as usize;
        let edge = WIDE + (NARROW - WIDE) * z.ln() / DEEP;
        self.dots.clear();
        for py in 0..h {
            let y0 = cy - (py as f64 + 0.5 - h as f64 / 2.0) * dx;
            for px in 0..w {
                let x0 = cx + (px as f64 + 0.5 - w as f64 / 2.0) * dx;
                // Inside the main cardioid or the disc beside it: no need to iterate.
                let q = (x0 - 0.25).powi(2) + y0 * y0;
                let inside = q * (q + x0 - 0.25) < 0.25 * y0 * y0 || (x0 + 1.0).powi(2) + y0 * y0 < 0.0625;
                let (mut x, mut y, mut u, mut v, mut n, mut m) = (0.0, 0.0, 0.0, 0.0, 0, 0.0);
                if !inside {
                    while n < max {
                        let nu = 2.0 * (x * u - y * v) + 1.0;
                        v = 2.0 * (x * v + y * u);
                        u = nu;
                        let (xx, yy) = (x * x, y * y);
                        y = 2.0 * x * y + y0;
                        x = xx - yy + x0;
                        m = x * x + y * y;
                        if m > 1e6 {
                            break;
                        }
                        n += 1;
                    }
                }
                if inside || n == max {
                    // The set itself: bare on a dark page, inked on paper.
                    if env.paper && dither(px, py, 0.5) {
                        self.dots.set(px as i64, py as i64, 0);
                    }
                    continue;
                }
                // A glow that hugs the boundary, wide on the whole set and
                // narrower deep down so the curls stay apart.
                let d = 0.5 * m.sqrt() * m.ln() / (u * u + v * v).sqrt() / cell;
                let glow = (1.0 - (1.0 + d).log2() / edge).max(0.0);
                if glow <= 0.02 || !dither(px, py, 0.08 + 0.92 * glow) {
                    continue;
                }
                // Smooth escape time, so the bands do not step.
                let smooth = n as f64 + 1.0 - (0.5 * m.ln()).ln() / std::f64::consts::LN_2;
                let c = ((smooth / BAND).rem_euclid(COLORS as f64)) as u8;
                self.dots.set(px as i64, py as i64, c.min(COLORS as u8 - 1));
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
