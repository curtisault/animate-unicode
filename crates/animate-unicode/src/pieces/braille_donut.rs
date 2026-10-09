//! braille donut: the donut's torus again, but drawn into a braille dot
//! bitmap, eight dots a cell, and shaded by ordered dithering: the brighter
//! the surface, the more of its dots are set. The same 40×22 cells as the
//! ASCII donut hold an 80×88 picture, which is the resolution unicode buys.

use crate::{Category, Charset, Dots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The braille donut's meta.
pub static META: Meta = Meta {
    name: "braille donut",
    slug: "braille-donut",
    category: Category::Shapes,
    note: "the donut in braille: eight dots a cell, shaded by dithering",
    cols: 40,
    rows: 22,
    fps: 30,
    charset: Charset::Extended,
    ..Meta::DEFAULT
};

/// Steps around the tube and around the ring: fine enough to cover every dot.
const TUBE: usize = 220;
const RING: usize = 520;

/// A 4×4 Bayer matrix: a dot is set when the shade beats its threshold.
const BAYER: [[f64; 4]; 4] = [[0.0, 8.0, 2.0, 10.0], [12.0, 4.0, 14.0, 6.0], [3.0, 11.0, 1.0, 9.0], [15.0, 7.0, 13.0, 5.0]];

/// The donut: sine tables for both circles, a z-buffer, a shade buffer and the dots.
pub struct BrailleDonut {
    tube: Vec<(f64, f64)>,
    ring: Vec<(f64, f64)>,
    depth: Vec<f32>,
    shade: Vec<f32>,
    dots: Dots,
}

/// A braille donut; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let circle = |n: usize| (0..n).map(|i| (i as f64 / n as f64 * TAU).sin_cos()).collect::<Vec<_>>();
    let dots = Dots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(BrailleDonut { tube: circle(TUBE), ring: circle(RING), depth: vec![0.0; n], shade: vec![-1.0; n], dots })
}

impl Piece for BrailleDonut {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let (r1, r2, k2) = (1.0, 2.0, 6.0_f64);
        // Dots are square (a cell is 1 × 2, its dots 2 × 4), so no aspect fix.
        let k1 = (h.min(w) as f64 - 2.0) / 2.0 / ((r1 + r2) / (k2 * k2 - (r1 + r2) * (r1 + r2)).sqrt());
        let m = (0.4_f64 * 0.4 + 1.0 + 1.0).sqrt();
        let (lx, ly, lz) = (-0.4 / m, 1.0 / m, -1.0 / m);
        let (a, b) = (1.0 + t * 0.8, 1.0 + t * 0.35);
        let (sa, ca) = a.sin_cos();
        let (sb, cb) = b.sin_cos();
        self.depth.fill(0.0);
        self.shade.fill(-1.0);
        for &(st, ct) in &self.tube {
            let hh = r2 + r1 * ct;
            for &(sp, cp) in &self.ring {
                let x = hh * (cb * cp + sa * sb * sp) - r1 * st * ca * sb;
                let y = hh * (sb * cp - sa * cb * sp) + r1 * st * ca * cb;
                let ooz = 1.0 / (k2 + ca * hh * sp + r1 * st * sa);
                let col = (w as f64 / 2.0 + k1 * ooz * x).floor();
                let row = (h as f64 / 2.0 - k1 * ooz * y).floor();
                if col < 0.0 || row < 0.0 || col >= w as f64 || row >= h as f64 {
                    continue;
                }
                let k = row as usize * w + col as usize;
                if (ooz as f32) <= self.depth[k] {
                    continue;
                }
                self.depth[k] = ooz as f32;
                let nx = ct * (cb * cp + sa * sb * sp) - st * ca * sb;
                let ny = ct * (sb * cp - sa * cb * sp) + st * ca * cb;
                let nz = ca * ct * sp + st * sa;
                self.shade[k] = (nx * lx + ny * ly + nz * lz).max(0.0) as f32;
            }
        }
        // Dither: the surface gets dots in proportion to its light (or its
        // shadow, on paper, where a dot is dark ink). A floor keeps the unlit
        // side faintly drawn, so the shape never loses its outline.
        self.dots.clear();
        for y in 0..h {
            for x in 0..w {
                let s = f64::from(self.shade[y * w + x]);
                if s < 0.0 {
                    continue;
                }
                // A steeper curve than linear, so highlights stand out of the mid-tones.
                let lit = 0.07 + 0.93 * s.powf(1.6);
                let ink = if env.paper { 1.0 - lit + 0.1 } else { lit };
                if ink * 16.0 > BAYER[y % 4][x % 4] {
                    self.dots.set(x as i64, y as i64);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0, 0);
    }
}
