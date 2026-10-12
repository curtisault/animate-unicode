//! julia set: the Julia set of z² + c, drawn as a glow on its edge, with c
//! going round a circle so the set melts from one shape into the next. After
//! ascii.rest's julia-set.ts (MIT, @bas3line).
//!
//! Worked out per octant dot, eight a cell, so the 66×24 cells are a 132×96
//! picture. The glow's strength sets both how many dots are lit (ordered
//! dithering) and their colour, deep violet far from the edge through rose
//! and gold to white on it.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The Julia set's meta.
pub static META: Meta = Meta {
    name: "julia set",
    slug: "julia-set",
    category: Category::Generative,
    note: "a julia set in octants, glowing violet to gold to white at its edge",
    cols: 66,
    rows: 24,
    fps: 20,
    charset: Charset::Extended,
    // The glow, faintest first.
    palette: Some(&["#2a0f4a", "#4a1670", "#741e86", "#a42a7e", "#d24a6a", "#f07a52", "#fbb045", "#ffe08a", "#fff8e8"]),
    ground: Some("#07040c"),
    ..Meta::DEFAULT
};

/// Seconds for c to go once round.
const P: f64 = 40.0;
/// c's circle: it stays close to the Mandelbrot set's edge.
const CX: f64 = -0.4;
const R: f64 = 0.68;
/// c starts at -0.4 + 0.68i.
const A0: f64 = PI / 2.0;
const MAX: usize = 120;
/// The width of the view.
const SPAN: f64 = 3.7;
/// How far the glow reaches from the edge, in cell widths.
const REACH: f64 = 4.0;
const GLOW: Ramp = Ramp::new(0, 9);

/// The Julia set's glow per dot, and the dots.
pub struct JuliaSet {
    glow: Vec<f32>,
    dots: ColorOctants,
}

/// A Julia set; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(JuliaSet { glow: vec![0.0; n], dots })
}

impl Piece for JuliaSet {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let a = A0 + TAU * t / P;
        let (cr, ci) = (CX + R * a.cos(), R * a.sin());
        let dx = SPAN / w as f64; // dots are square
        let cell = SPAN / META.cols as f64; // the glow's reach is in cells
                                            // The set is symmetric through the origin: work out the top half and
                                            // turn it round for the bottom.
        for py in 0..h / 2 {
            let y0 = (py as f64 + 0.5 - h as f64 / 2.0) * dx;
            for px in 0..w {
                let (mut x, mut y) = ((px as f64 + 0.5 - w as f64 / 2.0) * dx, y0);
                let (mut ex, mut ey, mut m, mut n) = (1.0, 0.0, 0.0, 0);
                while n < MAX {
                    let (xx, yy) = (x * x, y * y);
                    m = xx + yy;
                    if m > 1e4 {
                        break;
                    }
                    // The orbit's derivative, for the distance to the set.
                    let t2 = 2.0 * (x * ex - y * ey);
                    ey = 2.0 * (x * ey + y * ex);
                    ex = t2;
                    y = 2.0 * x * y + ci;
                    x = xx - yy + cr;
                    n += 1;
                }
                let k = if n == MAX {
                    0.0 // inside: the glow is on the edge alone
                } else {
                    let z = m.sqrt();
                    let d = 0.5 * z * z.ln() / (ex * ex + ey * ey).sqrt() / cell;
                    (1.0 - (d / REACH).powf(0.6)).max(0.0)
                };
                self.glow[py * w + px] = k as f32;
                self.glow[(h - 1 - py) * w + (w - 1 - px)] = k as f32;
            }
        }
        self.dots.clear();
        for py in 0..h {
            for px in 0..w {
                let k = f64::from(self.glow[py * w + px]);
                if k <= 0.04 {
                    continue;
                }
                // The glow is the ink, on paper too, so it keeps its sense there.
                if dither(px, py, 0.1 + 0.9 * k) {
                    self.dots.set(px as i64, py as i64, GLOW.at(k));
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
