//! plasma: the old demo-scene effect. Four sine fields, one a set of rings
//! round a wandering centre, summed and read as soft blobs. After ascii.rest's
//! plasma.ts (MIT, @bas3line).
//!
//! The field is worked out per octant dot and cut into twelve bands of
//! colour, ink-blue through violet and rose to gold; a cell keeps only the
//! dots of its main band, so the bands meet along edges finer than a
//! character. The brighter the band, the more of its dots are lit, so in one
//! ink the blobs still read by density. sextant-plasma draws the same kind of
//! field as contour lines; this is the filled one.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The plasma's meta.
pub static META: Meta = Meta {
    name: "plasma",
    slug: "plasma",
    category: Category::Generative,
    note: "the demo-scene plasma in octants, filled in twelve bands of colour",
    cols: 64,
    rows: 22,
    fps: 24,
    charset: Charset::Extended,
    // Twelve bands, darkest first.
    palette: Some(&["#120a3a", "#1d1466", "#2a1d8f", "#4a23a8", "#7026b4", "#9a2aaa", "#c4348e", "#e44a6e", "#f4704e", "#fa9a3c", "#fcc443", "#fff08a"]),
    ground: Some("#06040f"),
    ..Meta::DEFAULT
};

/// Seconds a loop; every term turns a whole number of times in one.
const P: f64 = 30.0;
/// Ramp sweeps per unit of the summed field: low, so the blobs are broad.
const GAIN: f64 = 0.32;
const BANDS: usize = 12;

/// The plasma's dots.
pub struct Plasma {
    dots: ColorOctants,
}

/// A plasma; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Plasma { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Plasma {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let a = TAU / P * t;
        let (sa, ca) = a.sin_cos();
        // The rings' centre.
        let (cx, cy) = (14.0 * a.sin(), 9.0 * (2.0 * a).cos());
        // Dot centres in cell widths, rows at their true height: two dots a
        // cell width across, four a cell (two cell widths) down.
        let xs: Vec<f64> = (0..w).map(|px| (px as f64 + 0.5) / 2.0 - META.cols as f64 / 2.0).collect();
        let along_x: Vec<f64> = xs.iter().map(|x| (x * 0.11 + 3.0 * a).sin()).collect();
        self.dots.clear();
        for py in 0..h {
            let y = (py as f64 + 0.5) / 2.0 - META.rows as f64;
            let along_y = (y * 0.13 - 2.0 * a).sin();
            for (px, &x) in xs.iter().enumerate() {
                let mut v = along_x[px] + along_y;
                v += ((x * ca + y * sa) * 0.09 + 4.0 * a).sin();
                v += (((x - cx).powi(2) + (y - cy).powi(2)).sqrt() * 0.17 - 5.0 * a).sin();
                // Up the bands and back down, evenly, drifting through them once a loop.
                let u = v * GAIN + a / PI;
                let k = (u - 2.0 * (u / 2.0 + 0.5).floor()).abs();
                let band = ((k * BANDS as f64) as usize).min(BANDS - 1);
                let b = band as f64 / (BANDS - 1) as f64;
                let ink = if env.paper { 0.95 - 0.6 * b } else { 0.35 + 0.65 * b };
                if dither(px, py, ink) {
                    self.dots.set(px as i64, py as i64, band as u8);
                }
            }
        }
        grid.clear();
        self.dots.draw_split(grid, 0, 0);
    }
}
