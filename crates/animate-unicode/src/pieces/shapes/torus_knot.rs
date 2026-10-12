//! torus knot: a (2,3) trefoil knot as a lit tube, z-buffered so the strands
//! pass over and under each other, turning slowly about its axis. After
//! ascii.rest's torus-knot.ts (MIT, @bas3line).
//!
//! In braille the 60×28 cells are a 120×112 picture: the tube is z-buffered
//! per dot and shaded by ordered dithering in a ramp of sea-glass greens with
//! a white glint, and where one strand passes over another the lower one
//! breaks off two dots short of it, so each crossing reads.

use crate::geom::{dot, unit};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The torus knot's meta.
pub static META: Meta = Meta {
    name: "torus knot",
    slug: "torus-knot",
    category: Category::Shapes,
    note: "a trefoil knot as a lit tube in braille, turning slowly",
    cols: 60,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    // Dark to light, then the glint.
    palette: Some(&["#0f3b3a", "#155a55", "#1d7a70", "#2a9a8a", "#45b9a2", "#74d4bb", "#a9ead4", "#f2fffb"]),
    ground: Some("#071312"),
    ..Meta::DEFAULT
};

const TUBE: f64 = 0.28;
/// Samples along the knot and round the tube: enough to cover every dot.
const NU: usize = 1600;
const NV: usize = 56;
/// Eye to centre.
const D: f64 = 11.0;
/// A depth step this big, between far apart parts of the knot, is a crossing.
const GAP: f32 = (TUBE * 0.9) as f32;
const LIT: Ramp = Ramp::new(0, 8);

/// The knot: surface points and normals (built once), the screen scale, a
/// per-dot record of which step along the knot it shows, and the dots.
pub struct TorusKnot {
    p: Vec<[f32; 3]>,
    n: Vec<[f32; 3]>,
    k: f64,
    along: Vec<u16>,
    cut: Vec<bool>,
    dots: ColorDots,
}

/// Spin about the knot's own axis, nodding between 0.2 and 0.76 rad of tilt
/// so the over and under strands separate in depth as it turns.
fn view(t: f64) -> (f64, f64) {
    (t * 0.3, 0.48 - 0.28 * (t * 0.35).cos())
}

/// A torus knot; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // The knot's centre line: twice round the z axis, three times through the hole.
    let curve = |u: f64| {
        let r = 2.0 + (3.0 * u).cos();
        [r * (2.0 * u).cos(), r * (2.0 * u).sin(), -(3.0 * u).sin()]
    };
    let (mut p, mut n) = (Vec::with_capacity(NU * NV), Vec::with_capacity(NU * NV));
    for i in 0..NU {
        let u = i as f64 / NU as f64 * TAU;
        let (c, d) = (curve(u), curve(u + 1e-3));
        let tn = unit([d[0] - c[0], d[1] - c[1], d[2] - c[2]]);
        // A frame across the tube: one axis in the xy plane, the other from a cross product.
        let a = unit([tn[1], -tn[0], 0.0]);
        let b = [-tn[2] * a[1], tn[2] * a[0], tn[0] * a[1] - tn[1] * a[0]];
        for j in 0..NV {
            let (sv, cv) = (j as f64 / NV as f64 * TAU).sin_cos();
            let nn = [a[0] * cv + b[0] * sv, a[1] * cv + b[1] * sv, b[2] * sv];
            n.push(nn.map(|x| x as f32));
            p.push([c[0] + TUBE * nn[0], c[1] + TUBE * nn[1], c[2] + TUBE * nn[2]].map(|x| x as f32));
        }
    }
    // Size the picture from the widest the knot gets over its whole motion.
    let (mut mx, mut my) = (0.0f64, 0.0f64);
    let mut s: f64 = 0.0;
    while s < TAU {
        let mut tilt: f64 = 0.2;
        while tilt <= 0.77 {
            let ((ss, cs), (st, ct)) = (s.sin_cos(), tilt.sin_cos());
            for q in p.iter().step_by(7) {
                let (x, y0, z0) = (f64::from(q[0]), f64::from(q[1]), f64::from(q[2]));
                let (rx, w) = (cs * x - ss * y0, ss * x + cs * y0);
                let (ry, rz) = (ct * w - st * z0, st * w + ct * z0);
                mx = mx.max(rx.abs() / (D - rz));
                my = my.max(ry.abs() / (D - rz));
            }
            tilt += 0.07;
        }
        s += 0.1;
    }
    let dots = ColorDots::new(META.cols, META.rows);
    let k = ((dots.width() as f64 - 4.0) / 2.0 / mx).min((dots.height() as f64 - 4.0) / 2.0 / my);
    let size = dots.width() * dots.height();
    Box::new(TorusKnot { p, n, k, along: vec![0; size], cut: vec![false; size], dots })
}

impl Piece for TorusKnot {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let (s, tilt) = view(t);
        let ((ss, cs), (st, ct)) = (s.sin_cos(), tilt.sin_cos());
        let r = [cs, -ss, 0.0, ct * ss, ct * cs, -st, st * ss, st * cs, ct].map(|x| x as f32);
        let light = unit([-0.35, 0.45, 1.0]);
        // Half vector for a glint, between the light and the eye (+z).
        let half = unit([light[0], light[1], light[2] + 1.0]);
        let (cx, cy, k) = (w as f32 / 2.0, h as f32 / 2.0, self.k as f32);
        self.dots.clear();
        for (q, (p, n)) in self.p.iter().zip(&self.n).enumerate() {
            let nz = r[6] * n[0] + r[7] * n[1] + r[8] * n[2];
            if nz < -0.15 {
                continue; // faces away from the eye
            }
            let x = r[0] * p[0] + r[1] * p[1];
            let y = r[3] * p[0] + r[4] * p[1] + r[5] * p[2];
            let z = r[6] * p[0] + r[7] * p[1] + r[8] * p[2];
            let ooz = 1.0 / (D as f32 - z);
            let (px, py) = ((cx + k * ooz * x).floor() as i64, (cy - k * ooz * y).floor() as i64);
            let nx = r[0] * n[0] + r[1] * n[1];
            let ny = r[3] * n[0] + r[4] * n[1] + r[5] * n[2];
            let nn = [f64::from(nx), f64::from(ny), f64::from(nz)];
            let diff = dot(nn, light).max(0.0);
            let spec = dot(nn, half).max(0.0).powi(24);
            let b = (0.08 + diff * 0.8 + spec * 0.5).min(1.0);
            if self.dots.plot(px, py, z, LIT.at(b)) {
                self.along[py as usize * w + px as usize] = (q / NV) as u16;
            }
        }
        // Where one strand passes over another, cut the lower one back two
        // dots from the upper one's outline, so the crossing reads as a gap.
        let over = |me: usize, them: (i64, i64), dots: &ColorDots, along: &[u16]| {
            let (x, y) = them;
            if dots.get(x, y).is_none() || dots.depth(x, y) <= dots.depth((me % w) as i64, (me / w) as i64) + GAP {
                return false;
            }
            let s = along[y as usize * w + x as usize].abs_diff(along[me]) as usize;
            s.min(NU - s) > NU / 16
        };
        for i in 0..w * h {
            let (x, y) = ((i % w) as i64, (i / w) as i64);
            self.cut[i] = self.dots.get(x, y).is_some()
                && [(1, 0), (-1, 0), (0, 1), (0, -1), (2, 0), (-2, 0), (0, 2), (0, -2)]
                    .iter()
                    .any(|&(dx, dy)| over(i, (x + dx, y + dy), &self.dots, &self.along));
        }
        for i in 0..w * h {
            if self.cut[i] {
                self.dots.unset((i % w) as i64, (i / w) as i64);
            }
        }
        // Dots in proportion to the light (on paper, to the shadow).
        let paper = env.paper;
        self.dots.dither_by(|c| {
            let b = f64::from(c) / 7.0;
            if paper {
                1.08 - b
            } else {
                0.18 + b
            }
        });
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
