//! planet: a ringed gas giant, banded and turning, lit from the upper left.
//! The rings throw a shadow across the globe and the globe one across the
//! rings. After ascii.rest's planet.ts (MIT, @bas3line).
//!
//! The globe, its clouds, the rings and both shadows are ascii.rest's,
//! sampled once per octant dot. The geometry is worked out once; a frame only
//! turns the clouds. In colour the zones are cream and the belts tan to brown
//! with a dark line along each edge, the storm is white, the pole cools to
//! grey-blue, the bright ring is pale gold beside a dusky inner ring, and the
//! shadows fall dark across both.

use crate::geom::{dot, unit, V3};
use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The planet's meta.
pub static META: Meta = Meta {
    name: "planet",
    slug: "planet",
    category: Category::Space,
    note: "a ringed gas giant in octants, cream zones and brown belts, shadows",
    cols: 64,
    rows: 18,
    fps: 20,
    charset: Charset::Extended,
    // The clouds, darkest belt to brightest zone; the pole; the rings, dusky
    // to bright; a star.
    palette: Some(&["#4a2c1c", "#7a5034", "#a8784e", "#cfa46c", "#ecd2a0", "#fff4dc", "#8a9cb0", "#6a5e4e", "#b8a684", "#ecdcb4", "#868aa0"]),
    ground: Some("#05050a"),
    ..Meta::DEFAULT
};

const CLOUD: Ramp = Ramp::new(0, 6);
const POLE: u8 = 6;
const RING: Ramp = Ramp::new(7, 3);
const STAR: u8 = 10;
/// The globe's radius in rows, how far the ring plane opens toward the eye,
/// how far the axis leans in the picture, and seconds a turn.
const RP: f64 = 6.5;
const TILT: f64 = 0.4;
const ROLL: f64 = 0.2;
const DAY: f64 = 18.0;
/// The rings by radius, in globe radii: from, to, opacity at from, at to, and
/// brightness. The inner one is dark dust, a dusky band where it crosses the globe.
const RINGS: [[f64; 5]; 3] = [[1.22, 1.48, 0.3, 0.5, 0.1], [1.5, 1.9, 0.85, 1.0, 1.0], [1.98, 2.16, 0.8, 0.65, 0.85]];

fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    let u = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// Ring opacity at radius r in globe radii, and the brightness of what is there.
fn ring(r: f64) -> (f64, f64) {
    RINGS.iter().find(|&&[a, b, ..]| r >= a && r < b).map_or((0.0, 0.0), |&[a, b, p, q, w]| (p + (q - p) * (r - a) / (b - a), w))
}

/// Cloud brightness at a latitude and longitude: pale zones and dark belts
/// whose edges ripple with longitude, a dark line along each edge, an oval
/// storm, and darker poles.
fn cloud(lat: f64, lon: f64) -> f64 {
    let w = lat + 0.05 * (4.0 * lon + 6.0 * lat).sin() + 0.03 * (9.0 * lon - 4.0 * lat + 1.0).sin();
    let s = (w * 7.2).cos(); // a pale zone on the equator, belts either side
    let mut v = 0.5 + 0.42 * smooth(-0.35, 0.35, s) - 0.28 * (-(s / 0.16).powi(2)).exp();
    v += 0.06 * (lon * 7.0 + lat * 21.0).sin();
    let dl = (lon + 0.6).rem_euclid(TAU) - PI;
    let d = (dl * lat.cos() / 0.42).powi(2) + ((lat - 0.42) / 0.12).powi(2);
    if d < 1.0 {
        v = if d > 0.45 { 0.3 } else { 1.0 };
    }
    v * (1.0 - 0.35 * lat.sin().abs().powi(4))
}

/// What a dot sees, worked out once: the globe (its latitude and longitude,
/// the light on it, and how much ring is in front of it, with that ring's
/// light), or only ring.
enum Sight {
    Globe { lat: f32, lon: f32, light: f32, op: f32, ring: f32 },
    Ring(f32),
}

/// The planet: what every dot sees, the stars (dot and phase), and the dots.
pub struct Planet {
    sights: Vec<(usize, Sight)>,
    stars: Vec<(usize, f64)>,
    dots: ColorOctants,
}

/// A planet; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let (sr, cr) = ROLL.sin_cos();
    let sun: V3 = unit([-0.42, 0.34, 0.84]);
    let axis: V3 = [0.0, TILT.cos(), TILT.sin()]; // the pole, in view space before the roll
    let e2: V3 = [0.0, -TILT.sin(), TILT.cos()]; // the equator's direction toward the eye
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let mut sights = vec![];
    for j in 0..h {
        for i in 0..w {
            // In rows from the centre, y up.
            let (px, py) = (((i as f64 + 0.5) / 2.0 - cols / 2.0) * 0.5, rows / 2.0 - (j as f64 + 0.5) / 4.0);
            let (x, y) = (cr * px - sr * py, sr * px + cr * py); // undo the roll
            let r2 = x * x + y * y;
            let mut globe = None;
            let mut gz = f64::NEG_INFINITY;
            if r2 < RP * RP {
                let z = (RP * RP - r2).sqrt();
                let n = [x / RP, y / RP, z / RP];
                gz = z;
                let lat = dot(n, axis).asin();
                let lon = n[0].atan2(dot(n, e2));
                let lit = smooth(-0.08, 0.4, dot(n, sun));
                // The rings' shadow: follow the light from here to the ring plane.
                let s = -(y * axis[1] + z * axis[2]) / dot(sun, axis);
                let shade = if s > 0.0 {
                    1.0 - 0.75 * ring(((x + s * sun[0]).powi(2) + (y + s * sun[1]).powi(2) + (z + s * sun[2]).powi(2)).sqrt() / RP).0
                } else {
                    1.0
                };
                globe = Some((lat, lon, 0.86 * lit * shade * (0.7 + 0.3 * n[2])));
            }
            // The ring plane where this line of sight crosses it.
            let z = -y * axis[1] / axis[2];
            let (op, albedo) = ring((r2 + z * z).sqrt() / RP);
            let (op, light) = if op > 0.0 && z > gz {
                // The globe's shadow on the rings, on the side away from the sun.
                let (b, c) = (x * sun[0] + y * sun[1] + z * sun[2], r2 + z * z - RP * RP);
                (op, op * albedo * if b < 0.0 && b * b > c { 0.04 } else { 1.0 })
            } else {
                (0.0, 0.0)
            };
            let k = j * w + i;
            match globe {
                Some((lat, lon, l)) => sights.push((k, Sight::Globe { lat: lat as f32, lon: lon as f32, light: l as f32, op: op as f32, ring: light as f32 })),
                None if op > 0.0 => sights.push((k, Sight::Ring(light as f32))),
                None => {}
            }
        }
    }
    let mut rand = Mulberry32::new(7);
    let stars = (0..14)
        .map(|_| {
            let (c, r) = (1.0 + (rand.next_f64() * (cols - 2.0)).floor(), (rand.next_f64() * rows).floor());
            ((r as usize * 4 + 1) * w + c as usize * 2, rand.next_f64() * TAU)
        })
        .collect();
    Box::new(Planet { sights, stars, dots })
}

impl Piece for Planet {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let spin = TAU * t / DAY;
        let w = self.dots.width();
        self.dots.clear();
        for (k, sight) in &self.sights {
            let (x, y) = (k % w, k / w);
            let (v, color) = match *sight {
                Sight::Globe { lat, lon, light, op, ring } => {
                    let (lat, op) = (f64::from(lat), f64::from(op));
                    let c = cloud(lat, f64::from(lon) - spin);
                    let globe = c * f64::from(light);
                    // A ring crossing the globe reads brighter than the globe behind it.
                    let v = globe * (1.0 - op) + f64::from(ring);
                    let color = if op > 0.3 {
                        RING.at(f64::from(ring) / op.max(0.01))
                    } else if lat.abs() > 1.05 && globe > 0.15 {
                        POLE
                    } else {
                        CLOUD.at(c * (0.35 + 0.65 * f64::from(light) / 0.86))
                    };
                    (v, color)
                }
                Sight::Ring(v) => (f64::from(v), RING.at(f64::from(v))),
            };
            // The planet is lit, not shaded, so it is ink by its light; on paper, by its shadow.
            let ink = if env.paper { 1.0 - v } else { 0.1 + 0.9 * v };
            if v > 0.02 && dither(x, y, ink) {
                self.dots.set(x as i64, y as i64, color);
            }
        }
        // Stars go only where nothing else is, and one in three slowly dims.
        for &(k, p) in &self.stars {
            let (x, y) = ((k % w) as i64, (k / w) as i64);
            if self.dots.get(x, y).is_none() && !(p < 2.0 && (t * 0.9 + p * 3.0).sin() < -0.3) {
                self.dots.set(x, y, STAR);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
