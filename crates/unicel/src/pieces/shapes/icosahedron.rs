//! icosahedron: a wireframe icosahedron turning on two axes. Edges on the
//! near side are solid lines, the hidden ones behind are dotted. After
//! ascii.rest's icosahedron.ts (MIT, @bas3line).
//!
//! In braille the lines can run at any angle, a dot wide, over a 92×88
//! picture. Each edge is coloured by how near it is, ice-white in front to
//! deep blue behind, and each vertex is a small knot of dots.

use crate::geom::{M3, V3};
use crate::grid::bresenham;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// The icosahedron's meta.
pub static META: Meta = Meta {
    name: "icosahedron",
    slug: "icosahedron",
    category: Category::Shapes,
    note: "a braille icosahedron turning, coloured by depth, hidden edges dotted",
    cols: 46,
    rows: 22,
    fps: 30,
    charset: Charset::Extended,
    // Far to near.
    palette: Some(&["#1b2f5e", "#24478c", "#2f66b8", "#4a8ad8", "#76b2ec", "#acd6f6", "#e6f4ff"]),
    ground: Some("#070b14"),
    ..Meta::DEFAULT
};

/// Eye to centre, in circumradii.
const D: f64 = 4.0;
const DEPTH: Ramp = Ramp::new(0, 7);

/// The solid: vertices, edges, faces, the faces round each edge and vertex,
/// and the dots.
pub struct Icosahedron {
    v: Vec<V3>,
    edges: Vec<(usize, usize)>,
    faces: Vec<[usize; 3]>,
    edge_faces: Vec<Vec<usize>>,
    vertex_faces: Vec<Vec<usize>>,
    dots: ColorDots,
}

/// An icosahedron; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let p = (1.0 + 5f64.sqrt()) / 2.0;
    let n = 1.0 / (1.0 + p * p).sqrt();
    let mut v = vec![];
    for a in [-1.0, 1.0] {
        for b in [-1.0, 1.0] {
            v.extend([[0.0, a * n, b * p * n], [a * n, b * p * n, 0.0], [b * p * n, 0.0, a * n]]);
        }
    }
    // Edges join vertices one edge length (2n) apart; faces are triangles of edges.
    let near = |i: usize, j: usize| {
        let (a, b): (V3, V3) = (v[i], v[j]);
        (((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() - 2.0 * n).abs() < 1e-6
    };
    let (mut edges, mut faces) = (vec![], vec![]);
    for i in 0..12 {
        for j in i + 1..12 {
            if !near(i, j) {
                continue;
            }
            edges.push((i, j));
            for k in j + 1..12 {
                if near(i, k) && near(j, k) {
                    faces.push([i, j, k]);
                }
            }
        }
    }
    let edge_faces = edges.iter().map(|&(i, j)| (0..faces.len()).filter(|&m| faces[m].contains(&i) && faces[m].contains(&j)).collect()).collect();
    let vertex_faces = (0..12).map(|i| (0..faces.len()).filter(|&m| faces[m].contains(&i)).collect()).collect();
    Box::new(Icosahedron { v, edges, faces, edge_faces, vertex_faces, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Icosahedron {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        let k = ((w - 6.0) / 2.0).min((h - 8.0) / 2.0) * (D * D - 1.0).sqrt();
        let turn = M3::rx(0.6 + t * 0.25).mul(&M3::ry(0.3 + t * 0.15));
        let r: Vec<V3> = self.v.iter().map(|&p| turn.apply(p)).collect();
        // Toward the eye is +z here; perspective from D away.
        let s: Vec<(f64, f64)> = r
            .iter()
            .map(|p| {
                let f = k / (D - p[2]);
                (w / 2.0 + f * p[0], h / 2.0 - f * p[1])
            })
            .collect();
        // A face is in view when its outward normal (its centre, here) points to the eye.
        let seen: Vec<bool> = self
            .faces
            .iter()
            .map(|f| {
                let c = f.iter().fold([0.0; 3], |c, &i| [c[0] + r[i][0], c[1] + r[i][1], c[2] + r[i][2]]);
                -c[0] * c[0] - c[1] * c[1] + c[2] * (3.0 * D - c[2]) > 0.0
            })
            .collect();

        self.dots.clear();
        // Hidden edges first, every third dot; the near ones then draw over them.
        for pass in [false, true] {
            for (e, &(i, j)) in self.edges.iter().enumerate() {
                if self.edge_faces[e].iter().any(|&m| seen[m]) != pass {
                    continue;
                }
                let ((x0, y0), (x1, y1)) = (s[i], s[j]);
                let (z0, z1) = (r[i][2], r[j][2]);
                let len = ((x1 - x0).powi(2) + (y1 - y0).powi(2)).sqrt().max(1.0);
                let mut step = 0;
                let dots = &mut self.dots;
                bresenham(x0.round() as i64, y0.round() as i64, x1.round() as i64, y1.round() as i64, |x, y| {
                    let u = (((x as f64 - x0).powi(2) + (y as f64 - y0).powi(2)).sqrt() / len).min(1.0);
                    let z = z0 + (z1 - z0) * u;
                    let shade = (z + 1.0) / 2.0;
                    if pass {
                        dots.set(x, y, DEPTH.at(0.25 + 0.75 * shade));
                    } else if step % 3 == 0 && dots.get(x, y).is_none() {
                        dots.set(x, y, DEPTH.at(0.25 * shade));
                    }
                    step += 1;
                });
            }
            for i in 0..12 {
                if self.vertex_faces[i].iter().any(|&m| seen[m]) != pass || !pass {
                    continue;
                }
                let (x, y) = (s[i].0.round() as i64, s[i].1.round() as i64);
                let c = DEPTH.at(0.4 + 0.6 * (r[i][2] + 1.0) / 2.0);
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
                    self.dots.set(x + dx, y + dy, c);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
