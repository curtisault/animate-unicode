//! heart: the implicit heart surface (x² + 9/4 y² + z² - 1)³ = x²z³ + 9/80
//! y²z³, ray-marched, swaying from side to side and beating lub-dub. After
//! ascii.rest's heart.ts (MIT, @bas3line).
//!
//! Marched once per octant dot, eight a cell, so the 52×24 cells are a 104×96
//! picture of square dots, shaded by ordered dithering in a ramp of reds with
//! a specular glint that slides over the surface as it sways.

use crate::geom::{dot, unit, V3};
use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The heart's meta.
pub static META: Meta = Meta {
    name: "heart",
    slug: "heart",
    category: Category::Shapes,
    note: "a glossy heart in octants, swaying and beating lub-dub",
    cols: 52,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // Reds, dimmest first, then the glint's pinks and white.
    palette: Some(&["#3a0710", "#5c0b19", "#821225", "#a91a31", "#cf2840", "#ec4459", "#f77a87", "#ffb3ba", "#fff0f2"]),
    ground: Some("#120609"),
    ..Meta::DEFAULT
};

/// Dots per unit across: TS's 18 cells, two dots a cell.
const K: f64 = 36.0;
const REDS: Ramp = Ramp::new(0, 6);
const GLINT: Ramp = Ramp::new(6, 3);

/// The same surface with the cube root taken, which keeps the gradient sane
/// along the waist. Inside is negative; z is up and y is the thin axis.
#[inline]
fn field(x: f64, y: f64, z: f64) -> f64 {
    x * x + 2.25 * y * y + z * z - 1.0 - z * (x * x + 0.1125 * y * y).cbrt()
}

/// The heart's dots.
pub struct Heart {
    dots: ColorOctants,
}

/// A heart; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Heart { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Heart {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0 + 5.2);
        // Lub, then a softer dub, once a second.
        let p = t.rem_euclid(1.0);
        let s = 1.0 + 0.075 * (-((p - 0.1) / 0.055).powi(2)).exp() + 0.045 * (-((p - 0.32) / 0.06).powi(2)).exp();
        let yaw = 0.62 * (t / 8.0 * TAU).sin();
        let (syw, cyw) = yaw.sin_cos();
        let (st, ct) = 0.18_f64.sin_cos();
        // Into the heart's own frame.
        let to_obj = |[x, y, z]: V3| -> V3 {
            let (y1, z1) = (ct * y - st * z, st * y + ct * z);
            [cyw * x + syw * y1, -syw * x + cyw * y1, z1]
        };
        let light = to_obj(unit([-0.4, -1.0, 0.65])); // upper left, in front (front is -y)
        let ray = to_obj([0.0, 1.0, 0.0]);
        let half = unit([light[0] - ray[0], light[1] - ray[1], light[2] - ray[2]]);

        self.dots.clear();
        for py in 0..h {
            let v = (cy - py as f64 - 0.5) / K / s;
            if !(-1.15..=1.4).contains(&v) {
                continue;
            }
            for px in 0..w {
                let u = (px as f64 + 0.5 - cx) / K / s;
                if !(-1.3..=1.3).contains(&u) {
                    continue;
                }
                // March from in front, where the heart cannot be, to behind it.
                let o = to_obj([u, -1.6, v]);
                let at = |d: f64| field(o[0] + d * ray[0], o[1] + d * ray[1], o[2] + d * ray[2]);
                let (mut d0, mut hit) = (0.45, None);
                let mut d = 0.51;
                while d <= 2.5 {
                    if at(d) < 0.0 {
                        let (mut a, mut b) = (d0, d);
                        for _ in 0..8 {
                            let mid = (a + b) / 2.0;
                            if at(mid) < 0.0 {
                                b = mid;
                            } else {
                                a = mid;
                            }
                        }
                        hit = Some(b);
                        break;
                    }
                    d0 = d;
                    d += 0.06;
                }
                let Some(d) = hit else { continue };
                let [x, y, z] = [o[0] + d * ray[0], o[1] + d * ray[1], o[2] + d * ray[2]];
                let q = x * x + 0.1125 * y * y + 1e-9;
                let wq = z / (3.0 * (q * q).cbrt());
                let n = unit([2.0 * x - wq * 2.0 * x, 4.5 * y - wq * 0.225 * y, 2.0 * z - q.cbrt()]);
                let diff = dot(n, light).max(0.0);
                let spec = dot(n, half).max(0.0).powi(40);
                let b = (0.1 + 0.62 * diff).min(1.0);
                // A dot is inked by the light it catches (on paper, by its shadow);
                // the glint is always solid.
                let ink = if env.paper { 1.05 - b } else { 0.18 + b };
                if spec > 0.55 {
                    self.dots.set(px as i64, py as i64, GLINT.at((spec - 0.55) / 0.45));
                } else if dither(px, py, ink) {
                    self.dots.set(px as i64, py as i64, REDS.at(b + 0.4 * spec));
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
