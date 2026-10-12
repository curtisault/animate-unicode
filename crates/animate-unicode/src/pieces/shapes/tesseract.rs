//! tesseract: a 4D hypercube turning in the xw and yz planes, projected 4D
//! to 3D to 2D. All 32 edges are drawn: solid on the side toward the 4D eye,
//! dotted on the far side, so the two cubes can be followed as they pass
//! through. After ascii.rest's tesseract.ts (MIT, @bas3line).
//!
//! In braille every edge is a true line, a dot wide, over a 108×112 picture,
//! and each is coloured by its fourth coordinate: amber toward the 4D eye,
//! violet away from it, so the cubes keep their colours as they swap places.

use crate::grid::bresenham;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::FRAC_PI_2;

/// The tesseract's meta.
pub static META: Meta = Meta {
    name: "tesseract",
    slug: "tesseract",
    category: Category::Shapes,
    note: "a 4d hypercube in braille, its edges coloured by their w",
    cols: 54,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    // From the far side in w (violet) to the near (amber).
    palette: Some(&["#3b2a78", "#5a3fb0", "#7e5fd6", "#a98bdc", "#d8b7a0", "#f0b45a", "#ffcf6e", "#fff0c4"]),
    ground: Some("#0b0a14"),
    ..Meta::DEFAULT
};

/// 4D eye to centre, and 3D eye to centre.
const W4: f64 = 3.0;
const D3: f64 = 7.0;
/// Radians a second in yz; xw turns four times as fast.
const YZ: f64 = 0.0875;
const BY_W: Ramp = Ramp::new(0, 8);

/// The hypercube: its edges, the widest it gets (to size the picture), and
/// the dots.
pub struct Tesseract {
    edges: Vec<(usize, usize)>,
    extent: (f64, f64),
    dots: ColorDots,
}

/// Vertex `i` of the hypercube: one bit a coordinate.
fn vertex(i: usize) -> [f64; 4] {
    [0, 1, 2, 3].map(|b| if i & (1 << b) != 0 { 1.0 } else { -1.0 })
}

/// Every vertex as seen at `t`, before the screen scale and centred on their
/// mean, with its w.
fn view(t: f64) -> [(f64, f64, f64); 16] {
    let (a, b) = (0.4 + t * 4.0 * YZ, 0.2 + t * YZ);
    let ((sa, ca), (sb, cb)) = (a.sin_cos(), b.sin_cos());
    let ((s0, c0), (s1, c1)) = (0.5_f64.sin_cos(), 0.32_f64.sin_cos());
    let mut out = [(0.0, 0.0, 0.0); 16];
    let (mut mx, mut my) = (0.0, 0.0);
    for (i, o) in out.iter_mut().enumerate() {
        let p = vertex(i);
        let (x, w) = (p[0] * ca - p[3] * sa, p[0] * sa + p[3] * ca);
        let (y, z) = (p[1] * cb - p[2] * sb, p[1] * sb + p[2] * cb);
        let f = W4 / (W4 - w);
        let (x, y, z) = (x * f, y * f, z * f);
        // A fixed view from up and to the side, so the cubes read as solids.
        let (x1, z1) = (x * c0 + z * s0, -x * s0 + z * c0);
        let (y2, z2) = (y * c1 - z1 * s1, y * s1 + z1 * c1);
        *o = (x1 / (D3 - z2), y2 / (D3 - z2), w);
        mx += o.0 / 16.0;
        my += o.1 / 16.0;
    }
    for o in &mut out {
        o.0 -= mx;
        o.1 -= my;
    }
    out
}

/// A tesseract; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut edges = vec![];
    for i in 0..16 {
        for b in [1, 2, 4, 8] {
            if i & b == 0 {
                edges.push((i, i | b));
            }
        }
    }
    // A quarter turn in yz maps the hypercube onto itself, so one loop of the
    // motion is that long: size the picture from the widest it gets in it.
    let (mut ex, mut ey) = (0.0f64, 0.0f64);
    let mut t = 0.0;
    while t < FRAC_PI_2 / YZ {
        for (x, y, _) in view(t) {
            ex = ex.max(x.abs());
            ey = ey.max(y.abs());
        }
        t += 0.03;
    }
    Box::new(Tesseract { edges, extent: (ex, ey), dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Tesseract {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        let k = ((w - 4.0) / 2.0 / self.extent.0).min((h - 4.0) / 2.0 / self.extent.1);
        let v = view(t);
        let s: Vec<(f64, f64)> = v.iter().map(|&(x, y, _)| (w / 2.0 + k * x, h / 2.0 - k * y)).collect();
        // w runs about -1.5 to 1.5 once projected; toward the eye is positive.
        let shade = |w: f64| ((w + 1.2) / 2.4).clamp(0.0, 1.0);
        self.dots.clear();
        // The far half first, dotted; the near half then draws over it.
        for near in [false, true] {
            for &(i, j) in &self.edges {
                let wm = (v[i].2 + v[j].2) / 2.0;
                if (wm >= 0.0) != near {
                    continue;
                }
                let ((x0, y0), (x1, y1)) = (s[i], s[j]);
                let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt().max(1.0);
                let (w0, w1) = (v[i].2, v[j].2);
                let mut step = 0;
                let dots = &mut self.dots;
                bresenham(x0.round() as i64, y0.round() as i64, x1.round() as i64, y1.round() as i64, |x, y| {
                    let u = (((x as f64 - x0).powi(2) + (y as f64 - y0).powi(2)).sqrt() / len).min(1.0);
                    let c = BY_W.at(shade(w0 + (w1 - w0) * u));
                    if near || (step % 3 != 2 && dots.get(x, y).is_none()) {
                        dots.set(x, y, c);
                    }
                    step += 1;
                });
            }
        }
        // Each vertex a small knot of dots, in its own w's colour.
        for (i, &(x, y)) in s.iter().enumerate() {
            let (x, y, c) = (x.round() as i64, y.round() as i64, BY_W.at(shade(v[i].2)));
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.dots.set(x + dx - 1, y + dy - 1, c);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
