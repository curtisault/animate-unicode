//! twisted ring: a ring of square section that twists three quarters of a
//! turn in one lap, so its four faces join into one. The faces are flat
//! shaded and the corners between them inked as lines; it turns about its own
//! axis. After ascii.rest's twisted-ring.ts (MIT, @bas3line).
//!
//! In braille the 46×21 cells are a 92×84 picture: the faces are dithered by
//! the light they catch and the corners are solid lines a dot wide. Colour
//! runs along the one face: a face comes back to itself only after four laps,
//! so the colour does too, changing smoothly the whole way.

use crate::geom::{dot, unit, V3};
use crate::subcell::{Braille, Encoding};
use crate::{Category, Charset, ColorDots, Dots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, SQRT_2, TAU};

/// The twisted ring's meta.
pub static META: Meta = Meta {
    name: "twisted ring",
    slug: "twisted-ring",
    category: Category::Shapes,
    note: "a twisted square ring; its one face's colour takes four laps",
    cols: 46,
    rows: 21,
    fps: 30,
    charset: Charset::Extended,
    // Six hues, each in four shades, dimmest first; then the corners' ink.
    palette: Some(&[
        "#173a5e", "#24578c", "#3478bd", "#62a3e6", "#2c2266", "#43349a", "#5f4ccc", "#9784f0", "#4f1d5a", "#782c88", "#a440b6", "#d37ee0", "#5a1d33",
        "#8a2c4e", "#bd406b", "#ec7a9d", "#5a3a14", "#8a5a1e", "#bd7e2c", "#ecae62", "#174a3e", "#24725e", "#349a80", "#66ccae", "#eef2f8",
    ]),
    ground: Some("#0a0b12"),
    ..Meta::DEFAULT
};

/// Ring radius, and the section's apothem.
const R: f64 = 0.75;
const AP: f64 = 0.22;
/// Quarter turns of the section in one lap; any odd number gives one face.
const TWIST: f64 = 3.0;
/// Tilt from edge on, eye distance, and rows per unit at the ring's centre.
const TILT: f64 = 0.92;
const EYE: f64 = 6.0;
const SCALE: f64 = 9.2;
/// Rows the ring sits above centre, since its near side looks bigger.
const LIFT: f64 = 0.5;
/// Radians a second, and the angle at t = 0.
const SPIN: f64 = 0.32;
const P0: f64 = 0.4;
/// Samples round the ring and across each face.
const LAPS: usize = 1440;
const ACROSS: usize = 24;
const HUES: usize = 6;
const SHADES: usize = 4;
const CORNER: u8 = 24;

/// The ring's faces and its corner lines.
pub struct TwistedRing {
    faces: ColorDots,
    corners: Dots,
}

/// A twisted ring; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(TwistedRing { faces: ColorDots::new(META.cols, META.rows), corners: Dots::new(META.cols, META.rows) })
}

impl Piece for TwistedRing {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.faces.width(), self.faces.height());
        let (ss, cs) = (P0 + SPIN * t).sin_cos();
        let (st, ct) = TILT.sin_cos();
        let light = unit([-0.5, 0.75, 0.6]);
        let half = unit([light[0], light[1], light[2] + 1.0]);
        // A point on the ring, at angle a round it and (qr, qy) in its section,
        // spun about the axis and tipped toward the eye: its dot, its nearness,
        // and where it is in the world.
        let place = |ca: f64, sa: f64, qr: f64, qy: f64| {
            let (x, z) = ((R + qr) * ca, (R + qr) * sa);
            let (wx, z1) = (cs * x + ss * z, -ss * x + cs * z);
            let (wy, wz) = (ct * qy - st * z1, st * qy + ct * z1);
            let ooz = 1.0 / (EYE - wz);
            let k = SCALE * EYE * ooz;
            ((w as f64 / 2.0 + 4.0 * k * wx), (h as f64 / 2.0 - 4.0 * (LIFT + k * wy)), ooz, [wx, wy, wz])
        };
        // The world normal of the face whose normal makes angle n in the section at angle a.
        let normal = |ca: f64, sa: f64, n: f64| -> V3 {
            let (ox, oz) = (n.cos() * ca, n.cos() * sa);
            let z1 = -ss * ox + cs * oz;
            [cs * ox + ss * oz, ct * n.sin() - st * z1, st * n.sin() + ct * z1]
        };
        let toward = |q: V3, p: V3| -q[0] * p[0] - q[1] * p[1] + q[2] * (EYE - p[2]) > 0.0;

        self.faces.clear();
        for i in 0..LAPS {
            let a = i as f64 / LAPS as f64 * TAU - PI;
            let (sa, ca) = a.sin_cos();
            for f in 0..4 {
                // Face f's normal, in the plane of the section, turns as it goes round.
                let n = TWIST * a / 4.0 + (2 * f + 1) as f64 * PI / 4.0;
                let (sn, cn) = n.sin_cos();
                let q = normal(ca, sa, n);
                let d = dot(q, light).max(0.0);
                let glint = dot(q, half) > 0.995;
                let shade = if d < 0.25 {
                    0
                } else if d < 0.5 {
                    1
                } else if d < 0.75 {
                    2
                } else {
                    3
                };
                // How far along the one face: a lap moves face f onto face f + 3,
                // so face f is (4 - f) % 4 laps on from face 0.
                let along = ((4 - f) % 4) as f64 + (a + PI) / TAU;
                let hue = (along / 4.0 * HUES as f64) as usize % HUES;
                let color = if glint { CORNER } else { (hue * SHADES + shade) as u8 };
                for j in 0..=ACROSS {
                    let s = 2.0 * j as f64 / ACROSS as f64 - 1.0;
                    let (px, py, ooz, world) = place(ca, sa, AP * (cn - s * sn), AP * (sn + s * cn));
                    if toward(q, world) {
                        self.faces.plot(px.floor() as i64, py.floor() as i64, ooz as f32, color);
                    }
                }
            }
        }

        // The four corners, each a line round the ring, inked where both faces
        // that meet there turn to the eye and nothing nearer covers it.
        self.corners.clear();
        let steps = LAPS * 4;
        for f in 0..4 {
            for i in 0..=steps {
                let a = i as f64 / steps as f64 * TAU - PI;
                let m = TWIST * a / 4.0 + f as f64 * PI / 2.0;
                let (sa, ca) = a.sin_cos();
                let (px, py, ooz, world) = place(ca, sa, AP * SQRT_2 * m.cos(), AP * SQRT_2 * m.sin());
                if !toward(normal(ca, sa, m - PI / 4.0), world) || !toward(normal(ca, sa, m + PI / 4.0), world) {
                    continue;
                }
                let (x, y) = (px.floor() as i64, py.floor() as i64);
                if self.faces.depth(x, y) == f32::NEG_INFINITY || (ooz as f32) < self.faces.depth(x, y) - 0.0025 {
                    continue;
                }
                self.corners.set(x, y);
            }
        }

        // Dots in proportion to the light (on paper, to the shadow); the glint stays solid.
        let paper = env.paper;
        self.faces.dither_by(|c| {
            if c == CORNER {
                return 1.0;
            }
            let b = (c as usize % SHADES) as f64 / (SHADES - 1) as f64;
            if paper {
                0.85 - 0.6 * b
            } else {
                0.2 + 0.62 * b
            }
        });
        grid.clear();
        self.faces.draw(grid, 0, 0);
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let e = self.corners.mask(cx, cy);
                if e != 0 {
                    grid.set(cx as i64, cy as i64, Braille::glyph(e | self.faces.mask(cx, cy)), CORNER);
                }
            }
        }
    }
}
