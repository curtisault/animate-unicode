//! octant sphere: a beach ball turning on a tilted axis, lit from the upper
//! left, drawn in octants (Unicode 16, U+1CD00 on): eight solid sub-cells a
//! cell, 2 × 4, so a 32×14 piece is a 64×56 picture of square dots. Its six
//! gores alternate white with red, yellow and blue; a cell shows only the dots
//! of the gore that holds most of it, so the seams between gores come out as
//! dark lines at the octants' resolution.

use crate::{Category, Charset, Env, Grid, Meta, Octants, Options, Piece};
use std::f64::consts::PI;

/// The octant sphere's meta.
pub static META: Meta = Meta {
    name: "octant sphere",
    slug: "octant-sphere",
    category: Category::Shapes,
    note: "a lit beach ball turning, drawn in octants, its seams a dot wide",
    cols: 32,
    rows: 14,
    fps: 30,
    charset: Charset::Extended,
    // Four runs of four shades, dimmest first: white, red, yellow, blue.
    palette: Some(&[
        "#55585e", "#8d9198", "#c4c8ce", "#f4f6f8", "#5a1414", "#952323", "#cf3434", "#f25b4b", "#5e4a0e", "#a07f16", "#dcb220", "#f8d64e", "#122a55",
        "#1f4a91", "#2d6bcf", "#5a92f0",
    ]),
    ground: Some("#0f141b"),
    options: Some(r#"{"speed": 1.0}"#),
    ..Meta::DEFAULT
};

const GORES: usize = 6;
const SHADES: usize = 4;
/// The axis leans this far towards the viewer.
const TILT: f64 = 0.38;

/// The ball: its speed, the per-dot gore and light, and the dots.
pub struct OctantSphere {
    speed: f64,
    gore: Vec<i8>,
    light: Vec<f32>,
    dots: Octants,
}

/// An octant sphere; reads `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let dots = Octants::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(OctantSphere { speed: options.f64("speed", 1.0), gore: vec![-1; n], light: vec![0.0; n], dots })
}

impl Piece for OctantSphere {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let spin = t * self.speed * 0.9;
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        let r = h.min(w) as f64 / 2.0 - 1.0;
        let (st, ct) = TILT.sin_cos();
        let (ss, cs) = spin.sin_cos();
        let l = {
            let (x, y, z): (f64, f64, f64) = (-0.5, 0.6, 0.62);
            let m = (x * x + y * y + z * z).sqrt();
            (x / m, y / m, z / m)
        };
        // Per dot: which gore it is on (or -1 off the ball) and how lit it is.
        for py in 0..h {
            for px in 0..w {
                let k = py * w + px;
                let (x, y) = ((px as f64 + 0.5 - cx) / r, (cy - py as f64 - 0.5) / r);
                let d2 = x * x + y * y;
                if d2 > 1.0 {
                    self.gore[k] = -1;
                    continue;
                }
                let z = (1.0 - d2).sqrt();
                // Into the ball's frame: undo the tilt (about x), then the spin
                // (about y). Gores only need the longitude.
                let z1 = y * st + z * ct;
                let (x2, z2) = (x * cs - z1 * ss, x * ss + z1 * cs);
                let lon = x2.atan2(z2) + PI; // 0..2π
                self.gore[k] = ((lon / (2.0 * PI) * GORES as f64) as usize % GORES) as i8;
                self.light[k] = (0.22 + 0.78 * (x * l.0 + y * l.1 + z * l.2).max(0.0)) as f32;
            }
        }
        // Per cell: the gore with most dots wins; only its dots are drawn, in
        // its colour at its mean light.
        self.dots.clear();
        grid.clear();
        for cy_ in 0..META.rows {
            for cx_ in 0..META.cols {
                let mut count = [0usize; GORES];
                let mut lit = [0.0f32; GORES];
                for dy in 0..4 {
                    for dx in 0..2 {
                        let k = (cy_ * 4 + dy) * w + cx_ * 2 + dx;
                        if let Ok(g) = usize::try_from(self.gore[k]) {
                            count[g] += 1;
                            lit[g] += self.light[k];
                        }
                    }
                }
                let Some(g) = (0..GORES).filter(|&g| count[g] > 0).max_by_key(|&g| (count[g], g)) else { continue };
                for dy in 0..4 {
                    for dx in 0..2 {
                        let k = (cy_ * 4 + dy) * w + cx_ * 2 + dx;
                        if self.gore[k] == g as i8 {
                            self.dots.set((cx_ * 2 + dx) as i64, (cy_ * 4 + dy) as i64);
                        }
                    }
                }
                // Even gores are white; odd ones red, yellow, blue in turn.
                let hue = if g % 2 == 0 { 0 } else { 1 + g / 2 };
                let mean = lit[g] / count[g] as f32;
                let shade = ((mean * SHADES as f32) as usize).min(SHADES - 1);
                grid.set(cx_ as i64, cy_ as i64, self.dots.glyph(cx_, cy_), (hue * SHADES + shade) as u8);
            }
        }
    }
}
