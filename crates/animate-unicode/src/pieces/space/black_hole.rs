//! black hole: a Schwarzschild black hole and its disk, seen almost edge on.
//! Rays are traced once through the bent space; then only the gas turns,
//! faster near the hole, the side coming toward us brighter. After
//! ascii.rest's black-hole.ts (MIT, @bas3line).
//!
//! The geodesics, the disk and its gas are ascii.rest's, but a ray is traced
//! for every octant dot, eight a cell, so the lensed arc over the hole and the
//! thin image under it are drawn at dot resolution, each dot lit by ordered
//! dithering. The gas is coloured by its temperature, white-hot at the inner
//! edge to deep red at the rim, the side coming toward us bluer and hotter
//! than the side going away; a thin photon ring rims the shadow.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The black hole's meta.
pub static META: Meta = Meta {
    name: "black hole",
    slug: "black-hole",
    category: Category::Space,
    note: "a black hole in octants, its disk white-hot to red, one side brighter",
    cols: 64,
    rows: 20,
    fps: 15,
    charset: Charset::Extended,
    // Cool to hot; the photon ring; the stars.
    palette: Some(&["#4a0c08", "#7a160c", "#b02c10", "#e0561a", "#f68c34", "#ffc266", "#ffe8b8", "#f4f6ff", "#ffd9a0", "#7a86a8"]),
    ground: Some("#040306"),
    ..Meta::DEFAULT
};

const HEAT: Ramp = Ramp::new(0, 8);
const RING: u8 = 8;
const STAR: u8 = 9;
/// Camera distance, the disk's inner and outer edge, in Schwarzschild radii.
const D: f64 = 30.0;
const IN: f64 = 3.0;
const OUT: f64 = 9.0;
/// The camera's height above the disk plane, in radians, and half the view's width as a tangent.
const TILT: f64 = 0.15;
const SPAN: f64 = 0.34;
/// Exposure, and the faintest light that shows. Brighter than ascii.rest's
/// 1.6: a dithered dot is lit or not, so a dot needs more light to read as
/// strongly as one of its ramp's characters.
const GAIN: f64 = 2.2;
const FLOOR: f64 = 0.03;
/// Seconds until the gas comes round to where it started; rings of gas, each
/// turning a whole number of times a loop; turns a loop at the inner edge.
const LOOP: f64 = 60.0;
const BANDS: usize = 12;
const SPIN: f64 = 7.0;
/// The table of paths: impact parameters, how many, and the step.
const BMAX: f64 = 12.0;
const NB: usize = 1500;
const H: f64 = 0.02;
const TAB: usize = 256;
/// The impact parameter below which a ray falls in, and how far past it the photon ring shows.
const CRIT: f64 = 2.598_076;
const RIM: f64 = 0.05;

/// Each path's u = 1/r at every step of the angle it sweeps from the camera.
/// A ray stays in a plane through the hole, and its path there depends only
/// on its impact parameter b; in that plane u'' = 1.5u² - u, stepped with RK4
/// until it falls in or leaves.
fn paths() -> Vec<Vec<f32>> {
    let f = |u: f64| 1.5 * u * u - u;
    (0..NB)
        .map(|n| {
            let b = (n as f64 + 0.5) / NB as f64 * BMAX;
            let (mut u, mut w) = (1.0 / D, (1.0 - (b / D).powi(2)).sqrt() / b);
            let mut us = vec![u as f32];
            while u < 1.0 && (u > 1.0 / (OUT + 1.0) || w > 0.0) {
                let (k1, l1) = (w, f(u));
                let (k2, l2) = (w + H / 2.0 * l1, f(u + H / 2.0 * k1));
                let (k3, l3) = (w + H / 2.0 * l2, f(u + H / 2.0 * k2));
                let (k4, l4) = (w + H * l3, f(u + H * k3));
                u += H / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
                w += H / 6.0 * (l1 + 2.0 * l2 + 2.0 * l3 + l4);
                us.push(u as f32);
            }
            us
        })
        .collect()
}

/// Where a ray crosses the disk: the band below it and how far toward the
/// next, its angle round the hole, and its weight.
struct Crossing {
    band: usize,
    frac: f32,
    phi: f32,
    weight: f32,
}

/// What a dot sees: the disk (its crossings and the colour of the nearest),
/// the photon ring, a star, or nothing.
enum Sight {
    Disk(Vec<Crossing>, u8),
    Ring(f32),
    Star,
    Dark,
}

/// The hole: what every dot sees, each band's ring of gas and its turns a loop, and the dots.
pub struct BlackHole {
    sights: Vec<Sight>,
    bands: Vec<(Vec<f32>, f64)>,
    dots: ColorOctants,
}

/// A black hole; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let cw = 2.0 * SPAN / cols; // one cell's width as a tangent; it stands twice as tall
    let (s, c) = TILT.sin_cos();
    let table = paths();
    // Each band's gas is a ring of noise: a few whole harmonics, so it closes.
    let mut rand = Mulberry32::new(17);
    let bands = (0..BANDS)
        .map(|b| {
            let r = IN + (OUT - IN) * b as f64 / (BANDS - 1) as f64;
            let turns = (SPIN * (IN / r).powf(1.5)).round().max(1.0);
            #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the gas is theirs
            let waves: Vec<(f64, f64, f64)> =
                [2.0, 3.0, 5.0, 8.0, 13.0].iter().map(|&k: &f64| (k, rand.next_f64() * 6.283, (0.5 + rand.next_f64()) / k.sqrt())).collect();
            let tab: Vec<f64> = (0..TAB).map(|i| waves.iter().map(|&(k, p, a)| a * (k * TAU * i as f64 / TAB as f64 + p).sin()).sum()).collect();
            let (lo, hi) = tab.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
            (tab.iter().map(|&v| (0.45 + 0.55 * (v - lo) / (hi - lo)) as f32).collect(), turns)
        })
        .collect();
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let mut sights = Vec::with_capacity(w * h);
    for j in 0..h {
        // Half a row high: the arc over the hole is the taller one.
        let y = (rows / 2.0 + 0.5 - (j as f64 + 0.5) / 4.0) * 2.0 * cw;
        for i in 0..w {
            let x = ((i as f64 + 0.5) / 2.0 - cols / 2.0) * cw;
            // The ray's plane: e1 toward the camera, e2 along the ray. It
            // crosses the disk where its height s cos a + e2y sin a is zero,
            // every half turn.
            let n = (x * x + y * y + 1.0).sqrt();
            let d = [x / n, (y * c - s) / n, (y * s + c) / n];
            let out = d[1] * s - d[2] * c;
            let side = (1.0 - out * out).sqrt();
            let e2 = [d[0] / side, (d[1] - out * s) / side, (d[2] + out * c) / side];
            let b = D * side;
            let us = &table[((b * NB as f64 / BMAX) as usize).min(NB - 1)];
            let falls = us[us.len() - 1] >= 1.0;
            // Images that wind more than once round the hole are thinner than
            // a dot, so they are left out; the photon ring stands for them.
            let end = ((us.len() - 1) as f64 * H).min(5.6);
            let (mut crossings, mut color, mut light) = (vec![], 0, 1.0);
            let mut a = (f64::atan2(-s, e2[1]) + TAU).rem_euclid(PI);
            while a < end {
                let p = a / H;
                let q = p as usize;
                let r = 1.0 / (f64::from(us[q]) + f64::from(us[q + 1] - us[q]) * (p - q as f64));
                if r > IN && r < OUT {
                    let phi = f64::atan2(a.sin() * e2[2] - a.cos() * c, a.sin() * e2[0]);
                    let edge = ((r - IN) / 0.7).min(1.0) * (1.0 - ((r - IN) / (OUT - IN)).powi(2));
                    let doppler = 1.0 - 0.5 * phi.cos();
                    let pos = (r - IN) / (OUT - IN) * (BANDS - 1) as f64;
                    let band = (pos as usize).min(BANDS - 2);
                    if crossings.is_empty() {
                        // Hotter inward, and shifted hotter on the side coming toward us.
                        let heat = (IN / r).powf(0.75) * (0.55 + 0.45 * doppler);
                        color = HEAT.at(heat);
                    }
                    crossings.push(Crossing {
                        band,
                        frac: (pos - band as f64) as f32,
                        phi: phi as f32,
                        weight: (light * edge * doppler * (IN / r).powf(1.5)) as f32,
                    });
                    light *= 0.45; // a crossing seen through the disk is dimmer
                }
                a += PI;
            }
            sights.push(if !crossings.is_empty() {
                Sight::Disk(crossings, color)
            } else if !falls && b - CRIT < RIM {
                // Brighter on the approaching side, the left.
                Sight::Ring((0.55 - 0.35 * x / (cols / 2.0 * cw)) as f32)
            } else if !falls && rand.next_f64() < 0.003 {
                Sight::Star
            } else {
                Sight::Dark
            });
        }
    }
    Box::new(BlackHole { sights, bands, dots })
}

impl Piece for BlackHole {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let turn = TAU * t.rem_euclid(LOOP) / LOOP;
        let w = self.dots.width();
        let at = |b: usize, a: f64| {
            let (tab, _) = &self.bands[b];
            f64::from(tab[((a / TAU * TAB as f64).floor() as i64).rem_euclid(TAB as i64) as usize])
        };
        self.dots.clear();
        // The disk is light, not shade, so it keeps its sense on paper too.
        for (k, sight) in self.sights.iter().enumerate() {
            let (x, y) = (k % w, k / w);
            match sight {
                Sight::Disk(crossings, color) => {
                    let mut sum = 0.0;
                    for cr in crossings {
                        let phi = f64::from(cr.phi);
                        let g0 = at(cr.band, phi - self.bands[cr.band].1 * turn);
                        let g1 = at(cr.band + 1, phi - self.bands[cr.band + 1].1 * turn);
                        sum += f64::from(cr.weight) * (g0 + (g1 - g0) * f64::from(cr.frac));
                    }
                    let v = ((sum * GAIN - FLOOR).max(0.0) / (1.0 - FLOOR)).min(1.0).powf(0.6);
                    if dither(x, y, v) {
                        self.dots.set(x as i64, y as i64, *color);
                    }
                }
                Sight::Ring(v) if dither(x, y, f64::from(*v)) => self.dots.set(x as i64, y as i64, RING),
                Sight::Star => self.dots.set(x as i64, y as i64, STAR),
                _ => {}
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
