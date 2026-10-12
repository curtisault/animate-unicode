//! cube: a cube drawn the way a drafting sheet draws it, in parallel
//! projection with only the near edges and faces, turning by quarters. After
//! ascii.rest's cube.ts (MIT, @bas3line).
//!
//! In braille the 32×18 cells are a 64×72 picture of square dots: the faces
//! are shaded by ordered dithering, more dots where more light falls, and the
//! edges are solid lines over them. Each pair of opposite faces has its own
//! colour, so a quarter turn reads as one colour handing over to the next.

use crate::geom::{dot, unit, V3};
use crate::subcell::{Braille, Encoding};
use crate::util::dither;
use crate::{Category, Charset, ColorDots, Dots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::FRAC_PI_2;

/// The cube's meta.
pub static META: Meta = Meta {
    name: "cube",
    slug: "cube",
    category: Category::Shapes,
    note: "a cube in braille, its faces dithered, turning a quarter at a time",
    cols: 32,
    rows: 18,
    fps: 24,
    charset: Charset::Extended,
    // Three runs of four, dimmest first: the x faces red, y faces green, z
    // faces blue; then the edges' ink.
    palette: Some(&[
        "#4a1d1f", "#86312f", "#c4473d", "#f2705c", "#173a26", "#26683d", "#3d9a55", "#6fcf7e", "#1b2a52", "#2a4890", "#3c6cc9", "#6f9bf0", "#e8ecf2",
    ]),
    ground: Some("#0d1117"),
    ..Meta::DEFAULT
};

/// Dots per half-edge: TS's 4.95 rows, four dots a row.
const S: f64 = 19.8;
/// The resting yaw: long edges flat, receding edges one column a row.
const REST: f64 = 0.21;
/// Looking down on the top face.
const PITCH: f64 = -0.43;
/// Seconds to turn a quarter, then seconds at rest after each turn.
const TURN: f64 = 4.0;
const HOLD: f64 = 2.0;
const EDGE: u8 = 12;

/// The cube's dots: faces in colour, edges on their own layer.
pub struct Cube {
    faces: ColorDots,
    edges: Dots,
}

/// A cube; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Cube { faces: ColorDots::new(META.cols, META.rows), edges: Dots::new(META.cols, META.rows) })
}

/// Corner `i` of the cube: bit 0 is x, bit 1 y, bit 2 z.
fn corner(i: usize) -> V3 {
    [if i & 1 != 0 { 1.0 } else { -1.0 }, if i & 2 != 0 { 1.0 } else { -1.0 }, if i & 4 != 0 { 1.0 } else { -1.0 }]
}

/// Each face: its axis, its sign, and its corners in order around it.
fn faces() -> [(usize, f64, [usize; 4]); 6] {
    let mut f = [(0, 0.0, [0; 4]); 6];
    for k in 0..3 {
        let others: Vec<usize> = [1, 2, 4].into_iter().enumerate().filter(|&(j, _)| j != k).map(|(_, b)| b).collect();
        let (a, b) = (others[0], others[1]);
        for s in 0..2 {
            let base = s << k;
            f[k * 2 + s] = (k, if s == 1 { 1.0 } else { -1.0 }, [base, base | a, base | a | b, base | b]);
        }
    }
    f
}

/// Whether `(x, y)` is inside the convex quad `q`.
fn inside(q: &[(f64, f64); 4], x: f64, y: f64) -> bool {
    let mut sign = 0.0;
    for i in 0..4 {
        let ((x0, y0), (x1, y1)) = (q[i], q[(i + 1) % 4]);
        let d = (x1 - x0) * (y - y0) - (y1 - y0) * (x - x0);
        if sign != 0.0 && d * sign < 0.0 {
            return false;
        }
        if d != 0.0 {
            sign = d.signum();
        }
    }
    true
}

impl Piece for Cube {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.faces.width() as f64, self.faces.height() as f64);
        // A quintic ease, so each quarter starts and stops without a jolt.
        let ease = |u: f64| {
            let u = u.clamp(0.0, 1.0);
            u * u * u * (u * (6.0 * u - 15.0) + 10.0)
        };
        let n = (t / (TURN + HOLD)).floor();
        let yaw = REST + FRAC_PI_2 * (n + ease((t - n * (TURN + HOLD)) / TURN));
        let (sy, cy) = yaw.sin_cos();
        let (sp, cp) = PITCH.sin_cos();
        let turn = |[x, y, z]: V3| -> V3 {
            let z1 = z * cy - x * sy;
            [x * cy + z * sy, y * cp - z1 * sp, y * sp + z1 * cp]
        };
        let light = unit([-0.35, 0.6, -0.72]); // from the upper left, toward the eye at -z
        let p: Vec<(f64, f64)> = (0..8)
            .map(|i| {
                let [x, y, _] = turn(corner(i));
                (w / 2.0 + x * S, h / 2.0 - y * S)
            })
            .collect();

        self.faces.clear();
        self.edges.clear();
        for (k, s, q) in faces() {
            let mut normal = [0.0; 3];
            normal[k] = s;
            let n = turn(normal);
            // A face shows when it turns toward the eye by more than a sliver.
            if n[2] >= -0.12 {
                continue;
            }
            let lit = dot(n, light).max(0.0);
            let quad = [p[q[0]], p[q[1]], p[q[2]], p[q[3]]];
            let ramp = Ramp::new(k as u8 * 4, 4);
            // More dots where more light falls (on paper, where less does), and
            // never none, so a face in shadow still reads as a face.
            let shade = 0.12 + 0.62 * lit;
            let ink = if env.paper { 0.86 - shade } else { shade };
            let (x0, x1) = (quad.iter().map(|v| v.0).fold(f64::MAX, f64::min), quad.iter().map(|v| v.0).fold(f64::MIN, f64::max));
            let (y0, y1) = (quad.iter().map(|v| v.1).fold(f64::MAX, f64::min), quad.iter().map(|v| v.1).fold(f64::MIN, f64::max));
            for y in y0.max(0.0) as usize..(y1.ceil() as usize).min(h as usize) {
                for x in x0.max(0.0) as usize..(x1.ceil() as usize).min(w as usize) {
                    if inside(&quad, x as f64 + 0.5, y as f64 + 0.5) && dither(x, y, ink) {
                        self.faces.set(x as i64, y as i64, ramp.at(lit));
                    }
                }
            }
            for i in 0..4 {
                let (a, b) = (quad[i], quad[(i + 1) % 4]);
                self.edges.line(a.0.round() as i64, a.1.round() as i64, b.0.round() as i64, b.1.round() as i64);
            }
        }

        // Faces first; a cell an edge passes through keeps the face's dots
        // and adds the edge's, in the edge's ink.
        grid.clear();
        self.faces.draw(grid, 0, 0);
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let e = self.edges.mask(cx, cy);
                if e != 0 {
                    let g = Braille::glyph(e | self.faces.mask(cx, cy));
                    grid.set(cx as i64, cy as i64, g, EDGE);
                }
            }
        }
    }
}
