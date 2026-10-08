//! donut: a torus turning on two axes, lit from the upper left. After Andy
//! Sloane's donut.c, with the ring sized so it never clips at any angle.
//!
//! A straight port of ascii.rest's donut.ts (MIT, @bas3line): the reference
//! piece, whose frames `tests/contract.rs` checks against the TS original.

use crate::{util::ramp, Category, Env, Grid, Meta, Options, Piece};

/// The donut's meta.
pub static META: Meta = Meta {
    name: "donut",
    slug: "donut",
    category: Category::Shapes,
    note: "a lit torus turning on two axes, after donut.c",
    cols: 40,
    rows: 22,
    fps: 30,
    ..Meta::DEFAULT
};

const RAMP: [char; 12] = ['.', ',', '-', '~', ':', ';', '=', '!', '*', '#', '$', '@'];

/// The donut, with its z-buffer.
pub struct Donut {
    depth: Vec<f32>,
}

/// A donut; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Donut { depth: vec![0.0; META.cols * META.rows] })
}

impl Piece for Donut {
    // 6.283 and the step sizes are donut.ts's: the fixture test holds this port to the TS frames cell for cell.
    #[allow(clippy::approx_constant)]
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let r1 = 1.0; // tube radius
        let r2 = 2.0; // ring radius
        let k2 = 6.0_f64; // eye to centre
        let aspect = 0.5; // a cell is about twice as tall as it is wide
        let k1 = (cols - 2.0) / 2.0 / ((r1 + r2) / (k2 * k2 - (r1 + r2) * (r1 + r2)).sqrt());
        let m = (0.4_f64 * 0.4 + 1.0 + 1.0).sqrt();
        let (lx, ly, lz) = (-0.4 / m, 1.0 / m, -1.0 / m);

        let a = 1.0 + t * 0.8;
        let b = 1.0 + t * 0.35;
        grid.clear();
        self.depth.fill(0.0);
        let (ca, sa, cb, sb) = (a.cos(), a.sin(), b.cos(), b.sin());
        let mut th: f64 = 0.0;
        while th < 6.283 {
            let (ct, st) = (th.cos(), th.sin());
            let mut ph: f64 = 0.0;
            while ph < 6.283 {
                let (cp, sp) = (ph.cos(), ph.sin());
                let h = r2 + r1 * ct;
                let x = h * (cb * cp + sa * sb * sp) - r1 * st * ca * sb;
                let y = h * (sb * cp - sa * cb * sp) + r1 * st * ca * cb;
                let ooz = 1.0 / (k2 + ca * h * sp + r1 * st * sa);
                let col = (cols / 2.0 + k1 * ooz * x).floor();
                let row = (rows / 2.0 - k1 * aspect * ooz * y).floor();
                ph += 0.03;
                if col < 0.0 || col >= cols || row < 0.0 || row >= rows {
                    continue;
                }
                let k = grid.index(col as usize, row as usize);
                if (ooz as f32) <= self.depth[k] {
                    continue;
                }
                self.depth[k] = ooz as f32;
                // The surface normal is the same rotation applied to the tube's circle.
                let nx = ct * (cb * cp + sa * sb * sp) - st * ca * sb;
                let ny = ct * (sb * cp - sa * cb * sp) + st * ca * cb;
                let nz = ca * ct * sp + st * sa;
                let lum = (nx * lx + ny * ly + nz * lz).max(0.0);
                grid.cells_mut()[k] = ramp(&RAMP, lum, env.paper) as u32;
            }
            th += 0.07;
        }
    }
}
