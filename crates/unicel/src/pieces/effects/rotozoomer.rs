//! rotozoomer: the demo-scene classic. A wall of bevelled bricks set in
//! deep mortar, turned and scaled about the middle of the screen every
//! frame. After ascii.rest's rotozoomer.ts (MIT, @bas3line).
//!
//! The wall and its turn, zoom and drift are ascii.rest's. The texture is
//! sampled once per octant dot, so the bricks' edges stay sharp at any
//! angle; the mortar is left bare, the brick faces dense, the bevels lit
//! from the upper left solid and those in shadow sparse, so the relief shows
//! in one ink. In colour the bricks are terracotta, their top and left
//! bevels catching the light and the right and bottom ones dark.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The rotozoomer's meta.
pub static META: Meta = Meta {
    name: "rotozoomer",
    slug: "rotozoomer",
    category: Category::Effects,
    note: "a terracotta brick wall in octants, turning and zooming, bevels lit",
    cols: 64,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // The face; the top, left, right and bottom bevels.
    palette: Some(&["#b44c38", "#f4a080", "#d87460", "#74301e", "#521c12"]),
    ground: Some("#120c0a"),
    ..Meta::DEFAULT
};

/// Seconds for one full turn (the zoom and drift repeat in it); half the
/// mortar joint and the bevel's width, in brick heights.
const P: f64 = 16.0;
const M: f64 = 0.085;
const B: f64 = 0.15;
/// How densely each surface is drawn: the face, then the top, left, right
/// and bottom bevels, lit from the upper left.
const INK: [f64; 5] = [0.75, 1.0, 0.9, 0.45, 0.3];

/// The surface at (u, v) in brick heights, or None in the mortar: a brick
/// is two long and every other course is set half a brick over.
fn tex(u: f64, v: f64) -> Option<usize> {
    let iv = v.floor();
    let x = u + if (iv as i64) & 1 == 1 { 1.0 } else { 0.0 };
    let (lx, ly) = (x - 2.0 * (x / 2.0).floor() - M, v - iv - M);
    let (w, h) = (2.0 - 2.0 * M, 1.0 - 2.0 * M);
    if lx < 0.0 || ly < 0.0 || lx > w || ly > h {
        return None;
    }
    let e = ly.min(lx).min(w - lx).min(h - ly);
    Some(if e > B {
        0
    } else if e == ly {
        1
    } else if e == lx {
        2
    } else if e == w - lx {
        3
    } else {
        4
    })
}

/// The wall's dots.
pub struct Rotozoomer {
    dots: ColorOctants,
}

/// A rotozoomer; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Rotozoomer { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Rotozoomer {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width(), self.dots.height());
        let om = TAU / P;
        // The turn lingers where the courses run square to the screen.
        let th = om * t + 0.08;
        let a = th - 0.14 * (4.0 * th).sin();
        let l = 13.0 * (0.25 * (2.0 * om * t).cos()).exp(); // brick height, in cell widths
        let (ca, sa) = (a.cos() / l, a.sin() / l);
        // The wall drifts two bricks along and two courses up a loop, so it closes.
        let u0 = 4.0 * t / P + 0.7 * (om * t).sin() + 0.35;
        let v0 = 2.0 * t / P + 0.7 * (om * t).cos() - 0.2;
        self.dots.clear();
        for j in 0..h {
            let y = ((j as f64 + 0.5) / 4.0 - rows / 2.0) * 2.0;
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0 - cols / 2.0;
                let Some(s) = tex(x * ca + y * sa + u0, y * ca - x * sa + v0) else { continue };
                let ink = if env.paper { 1.15 - INK[s] } else { INK[s] };
                if dither(i, j, ink) {
                    self.dots.set(i as i64, j as i64, s as u8);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
