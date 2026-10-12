//! eclipse: a total solar eclipse. The new moon slides across the sun; the
//! last bead of sunlight flares into a diamond ring, the corona opens round
//! the black disk for totality, and the ring flashes again on the far side.
//! After ascii.rest's eclipse.ts (MIT, @bas3line).
//!
//! The crossing, the ragged limb, the streamers and the prominences are
//! ascii.rest's, sampled once per octant dot, so the sun's edge, the moon's
//! beads and the fine polar plumes are drawn eight to a cell. In colour the
//! sun is white-gold darkening to orange at its limb, the corona is pearl
//! fading to a cold blue in its streamers, the prominences are the pink of
//! hydrogen, and the diamond is white with its flares.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::FRAC_PI_2;

/// The eclipse's meta.
pub static META: Meta = Meta {
    name: "eclipse",
    slug: "eclipse",
    category: Category::Space,
    note: "the moon crosses a gold sun; a white diamond ring, then a pearl corona",
    cols: 64,
    rows: 27,
    fps: 20,
    charset: Charset::Extended,
    // The sun, limb to centre; the inner ring and the diamond; the corona, near
    // to far; prominences; the moon's rim; a star; the planet.
    palette: Some(&["#f08a2a", "#ffc850", "#fff4d0", "#ffffff", "#e4eaff", "#a4b0e6", "#ff5a82", "#4a4c5a", "#868ca6", "#fff0b8"]),
    ground: Some("#04050b"),
    ..Meta::DEFAULT
};

const SUN: Ramp = Ramp::new(0, 3);
const WHITE: u8 = 3;
const PEARL: u8 = 4;
const BLUE: u8 = 5;
const PROM: u8 = 6;
const RIM: u8 = 7;
const STAR: u8 = 8;
const PLANET: u8 = 9;

/// The sun's radius in rows (it spans twice as many columns), the moon's in
/// suns, seconds from one crossing to the next, the middle of totality, the
/// moon's path in radians below level, and how far off centre it passes.
const R: f64 = 6.0;
const MOON: f64 = 1.06;
const LOOP: f64 = 18.0;
const MID: f64 = 5.3;
const TILT: f64 = 0.2;
const MISS: f64 = 0.005;
/// Streamers: angle, width at the limb in radians, how fast they fade outward
/// in suns, and strength. They narrow as they go, so each ends in a point.
const STREAMERS: [[f64; 4]; 6] =
    [[0.1, 0.42, 0.44, 0.85], [3.28, 0.45, 0.44, 0.8], [0.85, 0.15, 0.5, 0.6], [2.35, 0.16, 0.48, 0.6], [-0.6, 0.15, 0.45, 0.55], [3.9, 0.14, 0.42, 0.5]];
/// Prominences: angle and size, small loops of gas just off the limb that the
/// moon covers and uncovers as it passes.
const PROMS: [[f64; 2]; 3] = [[2.9, 0.06], [-0.9, 0.05], [0.5, 0.04]];

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

fn wrap(a: f64) -> f64 {
    a.sin().atan2(a.cos())
}

/// What a dot of sky holds, fixed: the corona, the inner ring and the
/// prominences in it, the angle round the sun, and whether a star is there.
#[derive(Clone, Copy, Default)]
struct Sky {
    corona: f32,
    ring: f32,
    prom: f32,
    angle: f32,
    far: bool,
}

/// The eclipse: the moon's ragged limb, the sky at every dot, the stars, the planet's dot, and the dots.
pub struct Eclipse {
    limb: Vec<[f64; 3]>,
    sky: Vec<Sky>,
    stars: Vec<usize>,
    planet: usize,
    dots: ColorOctants,
}

impl Eclipse {
    fn moon_r(&self, a: f64) -> f64 {
        MOON + self.limb.iter().map(|&[k, p, h]| h * (k * a + p).sin()).sum::<f64>()
    }
}

/// An eclipse; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let mut rand = Mulberry32::new(11);
    // The moon's limb is ragged with valleys and peaks: Baily's beads.
    #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the beads are theirs
    let limb = (0..5).map(|k| [f64::from(3 + 4 * k) + (rand.next_f64() * 3.0).floor(), rand.next_f64() * 6.283, 0.006 / (1.0 + f64::from(k) * 0.4)]).collect();
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    // The corona's shape never changes, only how much of it shows: a bright
    // inner ring, a soft halo, the streamers and fine plumes at the poles.
    let mut sky = vec![Sky::default(); w * h];
    let mut stars = vec![];
    for j in 0..h {
        for i in 0..w {
            let (x, y) = (((i as f64 + 0.5) / 2.0 - cols / 2.0) / (2.0 * R), (rows / 2.0 - (j as f64 + 0.5) / 4.0) / R);
            let (d, a) = (x.hypot(y), y.atan2(x));
            if d < 1.0 {
                continue;
            }
            let streamers: f64 = STREAMERS
                .iter()
                .map(|&[at, wd, fall, amp]| {
                    let off = wrap(a - at) / (wd / d.powf(1.7));
                    amp * (-off * off).exp() * (-(d - 1.0) / fall).exp()
                })
                .sum();
            let pole = (-((a.abs() - FRAC_PI_2) / 0.4).powi(2)).exp() * (0.5 + 0.5 * (26.0 * a).cos());
            let prom: f64 = PROMS
                .iter()
                .map(|&[at, size]| {
                    let (px, py) = (at.cos() * (1.0 + size), at.sin() * (1.0 + size));
                    1.4 * (-(((x - px) / size).powi(2) + ((y - py) / (size * 0.8)).powi(2))).exp()
                })
                .sum();
            let corona = 0.25 * (-(d - 1.0) / 0.16).exp() + streamers + 0.3 * pole * (-(d - 1.0) / 0.2).exp();
            sky[j * w + i] = Sky { corona: corona as f32, ring: (-(d - 1.0) / 0.07).exp() as f32, prom: prom as f32, angle: a as f32, far: d > 1.35 };
            if d > 1.7 && corona < 0.02 && rand.next_f64() < 0.003 {
                stars.push(j * w + i);
            }
        }
    }
    // A planet, up and to the left.
    let planet = 2 * 4 * w + ((cols / 2.0 - 2.4 * 2.0 * R) * 2.0).round() as usize;
    Box::new(Eclipse { limb, sky, stars, planet, dots })
}

impl Piece for Eclipse {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width(), self.dots.height());
        let tau = (t - MID + LOOP / 2.0).rem_euclid(LOOP) - LOOP / 2.0;
        let d = 0.035 * tau + 0.00515 * tau.powi(3); // slow near the middle, fast at the ends
        let (mx, my) = (d * TILT.cos() + MISS * TILT.sin(), -d * TILT.sin() + MISS * TILT.cos());
        let off = mx.hypot(my);
        let gap = off + 1.0 - MOON; // the widest sliver of sun left, in suns; below 0 is totality
        let show = 1.0 - smooth(0.0, 0.07, gap); // the corona shows as the sunlight goes
        let flare = smooth(0.0, -0.03, gap);
        let bead = smooth(-0.004, 0.01, gap) * (1.0 - smooth(0.04, 0.09, gap));
        // The corona brightens as the eyes adapt through totality.
        let glow = 0.75 + 0.45 * smooth(0.0, -0.055, gap);
        self.dots.clear();
        // The sun and corona are light, not shade, so they keep their sense on
        // paper: there the sun is a disk of ink and the corona a halo of it.
        for j in 0..h {
            let y = (rows / 2.0 - (j as f64 + 0.5) / 4.0) / R;
            for i in 0..w {
                let x = ((i as f64 + 0.5) / 2.0 - cols / 2.0) / (2.0 * R);
                let (dx, dy) = (x - mx, y - my);
                let dm = dx.hypot(dy);
                let edge = if dm < MOON + 0.02 { self.moon_r(dy.atan2(dx)) } else { 0.0 };
                if dm < MOON - 0.02 || dm < edge {
                    // Before totality the moon's rim shows faintly against the sky.
                    if show < 0.5 && edge - dm < 0.025 && x * x + y * y >= 1.0 && dither(i, j, 0.5) {
                        self.dots.set(i as i64, j as i64, RIM);
                    }
                    continue;
                }
                let q = 1.0 - x * x - y * y;
                if q > 0.0 {
                    if dither(i, j, (0.72 + 0.28 * q.sqrt()).powf(0.7)) {
                        self.dots.set(i as i64, j as i64, SUN.at(q.sqrt()));
                    }
                    continue;
                }
                let s = self.sky[j * w + i];
                let a = f64::from(s.angle);
                let shimmer = 1.0 + 0.25 * (9.0 * a + 1.9 * t).sin() * (5.0 * a - 1.2 * t).sin();
                let ring = f64::from(s.ring + s.prom);
                // Faint light rounds to sky, as ascii.rest's ramp rounds it to a
                // blank, so the outer corona ends rather than speckling the frame.
                let v = ((f64::from(s.corona) + 1.3 * ring * shimmer) * glow * show).min(1.0).powf(0.8);
                if v > 0.08 && dither(i, j, (v - 0.08) / 0.92) {
                    let color = if s.prom > s.ring && s.prom > 0.2 {
                        PROM
                    } else if s.ring > 0.3 {
                        WHITE
                    } else if s.far {
                        BLUE
                    } else {
                        PEARL
                    };
                    self.dots.set(i as i64, j as i64, color);
                }
            }
        }
        if flare > 0.4 {
            for &k in self.stars.iter().chain([&self.planet]) {
                let (x, y) = ((k % w) as i64, (k / w) as i64);
                if self.dots.get(x, y).is_none() {
                    self.dots.set(x, y, if k == self.planet { PLANET } else { STAR });
                }
            }
        }
        // The diamond: one bead of sun on the limb, too bright for its dots,
        // with a flare either side and above and below.
        if bead > 0.05 {
            let bx = (cols / 2.0 - mx / off * 2.0 * R) * 2.0;
            let by = (rows / 2.0 + my / off * R) * 4.0;
            let (across, down) = (2.0 + 10.0 * bead, 1.0 + 7.0 * bead);
            let r = 1.0 + 1.5 * bead;
            for j in (by - down).floor() as i64..=(by + down).ceil() as i64 {
                for i in (bx - across).floor() as i64..=(bx + across).ceil() as i64 {
                    let (dx, dy) = (i as f64 + 0.5 - bx, j as f64 + 0.5 - by);
                    let arm = (dy.abs() < 0.6 && dx.abs() < across) || (dx.abs() < 0.6 && dy.abs() < down);
                    if dx.hypot(dy) < r || arm {
                        self.dots.set(i, j, WHITE);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
