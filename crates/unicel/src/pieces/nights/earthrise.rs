//! earthrise: the Earth coming up over the lunar horizon. The sun is low on
//! the right, so every crater rim and boulder throws a long black shadow
//! across the grey ground, and the same light makes a gibbous Earth with a
//! clean line between day and night. The Earth turns, its clouds drift, it
//! climbs very slowly, and a few bright stars breathe. After ascii.rest's
//! earthrise.ts (MIT, @bas3line).
//!
//! The cratered heightfield, marched column by column from a camera standing
//! on it with real shadows marched toward the sun, the globe and its maps,
//! and how each is lit, are ascii.rest's, rendered here per pixel instead of
//! per cell. Where ascii.rest draws each of 200 × 100 square cells as a dot
//! as big as it is bright, this draws 320 × 80 cells of two square pixels
//! each, an upper half block over a back colour ([`crate::pixels`]): 1.6
//! times the detail each way, solid colour, and gradients dithered in colour
//! rather than in dot size. So the Earth is a solid globe 70 pixels across,
//! its continents, cloud spirals and limb glow drawn in full, on maps a third
//! finer than ascii.rest's; the ground is marched in finer steps, so the
//! boulders keep their outlines; the sunlit ground is a shade under
//! ascii.rest's, whose dots drew it in grey ink, so it stays grey rather than
//! white; and the bright stars carry a two-pixel cross. The palette adds the near-blacks a solid pixel needs (the sky, the
//! shadows, the Earth's night side) where a small dot was enough. In one ink
//! each pixel is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::{Pixels, Rgb};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// Earthrise's meta.
pub static META: Meta = Meta {
    name: "earthrise",
    slug: "earthrise",
    category: Category::Nights,
    note: "the earth rising over a cratered lunar horizon in long low sunlight",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: the regolith's greys, the night's blues, the Earth's
    // oceans, clouds, forests and deserts. Then the darks a solid pixel needs.
    palette: Some(&[
        "#18181b", "#232326", "#303033", "#414143", "#555556", "#6b6a69", "#83817d", "#9c9993", "#b6b2aa", "#cfcac1", "#e6e1d8", "#f7f4ee", "#0c1120",
        "#131a2e", "#1b2540", "#262f4a", "#dfe9ff", "#0a2259", "#0f2f72", "#15408c", "#1d53a6", "#2a69bf", "#4386d3", "#6eaeea", "#a8d3f6", "#e8f0fa",
        "#c2d0e3", "#8b9fbc", "#2f4a26", "#3b5a2c", "#5b7238", "#7a9150", "#77783f", "#8f8550", "#a8955e", "#6b5634", "#b9774a", "#8a4838", "#030408",
        "#08090b", "#0e0e10", "#121214", "#06102a", "#081841", "#1d2d1a", "#3c3020", "#47371f",
    ]),
    ground: Some("#030408"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame width, in its cells.
const TSW: f64 = 200.0;
/// The screen row of eye level; the horizon dips below it.
const EYE: f64 = 37.0;
/// The focal length, in cells.
const F: f64 = 112.0;
const CAM_H: f64 = 20.0;
/// The moon's radius, in ground units, for the falling horizon.
const RM: f64 = 1500.0;
const ZMAX: f64 = 520.0;
/// The Earth on screen, its lower edge still behind the horizon.
const EC: (f64, f64) = (141.0, 32.0);
const ER: f64 = 22.0;
/// Which face of the globe is turned to us at the start.
const LON0: f64 = 3.5;
/// It climbs RISE rows and settles back over RISE_T seconds.
const RISE: f64 = 5.0;
const RISE_T: f64 = 150.0;
/// The bright stars, which twinkle: column, row, brightness, period in seconds.
const BRIGHT: [(f64, f64, f64, f64); 4] = [(24.0, 9.0, 1.0, 4.6), (67.0, 27.0, 0.85, 3.4), (99.0, 12.0, 0.95, 5.8), (189.0, 7.0, 0.8, 4.1)];
/// A few big boulders in the near ground: x, z, radius.
const ROCKS: [(f64, f64, f64); 6] = [(30.0, 50.0, 2.4), (62.0, 63.0, 1.8), (12.0, 70.0, 1.3), (-4.0, 45.0, 1.2), (90.0, 55.0, 1.6), (46.0, 90.0, 1.4)];
/// The Earth's maps, equirectangular, wrapping in longitude.
const TW: usize = 256;
const TH: usize = 128;
/// Storms the clouds wind round: longitude, latitude, which way.
const STORMS: [(f64, f64, f64); 5] = [(0.9, 0.8, 1.0), (3.1, -0.85, -1.0), (4.6, 0.62, 1.0), (1.9, 0.25, 1.0), (5.6, -0.55, -1.0)];

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / S - 0.5
}

/// `v` scaled to unit length.
fn unit(v: [f64; 3]) -> [f64; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Toward the sun: low, from the right and a little behind us (z is forward).
fn sun() -> [f64; 3] {
    unit([0.94, 0.14, -0.3])
}

/// A bowl with a raised rim, `r` in ground units, `q` its distance in radii.
fn bowl(q: f64, r: f64) -> f64 {
    r * ((if q < 1.0 { -0.36 * (1.0 - q * q) } else { 0.0 }) + 0.14 * (-((q - 1.0) / 0.25).powi(2)).exp())
}

/// Craters scattered one to a cell.
fn craters(x: f64, z: f64, cell: f64, salt: f64, r0: f64, r1: f64, p: f64) -> f64 {
    let (ci, cj) = ((x / cell).floor(), (z / cell).floor());
    let mut h = 0.0;
    for j in [cj - 1.0, cj, cj + 1.0] {
        for i in [ci - 1.0, ci, ci + 1.0] {
            if hash(i + salt, j - salt) > p {
                continue;
            }
            let (cx, cz) = ((i + hash(i, j + salt * 3.0)) * cell, (j + hash(i + salt * 5.0, j)) * cell);
            let e = hash(j + salt, i - salt * 7.0);
            let r = cell * (r0 + (r1 - r0) * e * e);
            let (dx, dz) = (x - cx, z - cz);
            let d2 = dx * dx + dz * dz;
            if d2 > 4.0 * r * r {
                continue;
            }
            h += bowl(d2.sqrt() / r, r) * 0.95;
        }
    }
    h
}

/// The ground's height at (x, z).
fn height(x: f64, z: f64) -> f64 {
    let mut h = 6.0 * fbm(x * 0.006 + 50.0, z * 0.006 + 50.0, 3) + 0.6 * fbm(x * 0.04, z * 0.04, 2);
    // old, worn highlands at the edge of sight, rising to the left, and a
    // lower ridge in front of them
    if z > 150.0 {
        let lift = smooth(150.0, 300.0, z);
        // rounded massifs: folded noise, worn smooth
        let m = 1.0 - (2.0 * fbm(x * 0.006 + 7.0, z * 0.006, 4) - 1.0).abs();
        h += lift * (28.0 * (-((x + 260.0) / 230.0).powi(2)).exp() + 45.0 * (m * m - 0.35));
        let ridge = (-((z - 205.0) / 20.0).powi(2)).exp() * (-((x + 140.0) / 130.0).powi(2)).exp();
        h += ridge * 22.0 * (0.35 + fbm(x * 0.018 + 3.0, z * 0.01, 3));
    }
    // big basins only out in the middle distance, so we do not stand in one
    if z > 90.0 {
        h += craters(x, z, 110.0, 11.0, 0.14, 0.34, 0.55) * smooth(90.0, 150.0, z);
    }
    if z < 300.0 {
        h += craters(x, z, 34.0, 23.0, 0.12, 0.36, 0.85);
    }
    if z < 170.0 {
        h += craters(x, z, 10.0, 37.0, 0.12, 0.34, 0.85) * smooth(170.0, 110.0, z);
    }
    // one big crater in the near ground, off to the left
    let q = (x + 24.0).hypot(z - 58.0) / 16.0;
    if q < 2.0 {
        h += bowl(q, 16.0);
    }
    // boulders strewn close by, and a few big ones
    if z < 90.0 {
        let c = 5.0;
        let (ci, cj) = ((x / c).floor(), (z / c).floor());
        if hash(ci + 91.0, cj) < 0.14 {
            let (bx, bz) = ((ci + 0.2 + 0.6 * hash(ci, cj + 92.0)) * c, (cj + 0.2 + 0.6 * hash(ci + 93.0, cj)) * c);
            let br = 0.3 + 0.5 * hash(ci + 94.0, cj + 95.0).powi(2);
            let d2 = ((x - bx).powi(2) + (z - bz).powi(2)) / (br * br);
            if d2 < 4.0 {
                h += br * 0.9 * (-d2 * 1.4).exp();
            }
        }
        for (bx, bz, br) in ROCKS {
            let d2 = ((x - bx).powi(2) + (z - bz).powi(2)) / (br * br);
            // a squat, lumpy dome
            if d2 < 1.0 {
                h += br * (0.85 + 0.3 * noise(x * 1.3, z * 1.3)) * (1.0 - d2).sqrt();
            }
        }
    }
    h
}

/// The share-`p` value of `a`, sorted.
fn quantile(a: &[f32], p: f64) -> f32 {
    let mut s = a.to_vec();
    s.sort_by(f32::total_cmp);
    s[(p * (s.len() - 1) as f64).floor() as usize]
}

/// One of the Earth's maps at a longitude and a map row, bilinear.
fn sample(a: &[f32], lon: f64, vy: f64) -> f64 {
    let fx = ((lon / TAU) % 1.0 + 1.0) % 1.0 * TW as f64;
    let x0 = fx.floor();
    let ax = fx - x0;
    let x0 = (x0 as usize).min(TW - 1);
    let x1 = (x0 + 1) % TW;
    let y0 = vy.floor().clamp(0.0, (TH - 2) as f64);
    let ay = clamp(vy - y0);
    let y0 = y0 as usize * TW;
    let (a0, b, c, d) = (f64::from(a[y0 + x0]), f64::from(a[y0 + x1]), f64::from(a[y0 + TW + x0]), f64::from(a[y0 + TW + x1]));
    a0 + (b - a0) * ax + (c - a0) * ay + (a0 - b - c + d) * ax * ay
}

/// A twinkling star's pixel: where, how bright, its period and phase, and
/// whether it is the core or the faint cross round it.
struct Twinkle {
    px: usize,
    py: usize,
    s: f64,
    period: f64,
    phase: f64,
    core: bool,
}

/// The view: the ground and the sky lit once, the Earth's maps, and the
/// picture, of which each frame redraws only the Earth and the bright stars.
pub struct Earthrise {
    /// The still picture: the lit ground and the sky.
    base: Vec<Rgb>,
    /// The sky pixels the Earth and its air can reach, as it rises and settles.
    reach: Vec<(usize, usize)>,
    twinkle: Vec<Twinkle>,
    tex: [Vec<f32>; 3],
    sea: Vec<f32>,
    cloud: Vec<f32>,
    pixels: Pixels,
}

/// An earthrise; it takes no options.
#[allow(clippy::needless_range_loop)] // the index reads several arrays at once
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let sn = sun();

    // --- the ground: march each column from near to far --------------------
    let mut ground = vec![false; n];
    let mut top = vec![H; W];
    let (mut gx, mut gz, mut gy) = (vec![0.0f64; n], vec![0.0f64; n], vec![0.0f64; n]);
    let cam_y = height(0.0, 7.0) + CAM_H;
    for c in 0..W {
        let dir = (col_x(c) + 0.5 - TSW / 2.0) / F;
        let (mut top_r, mut z, mut prev_y, mut prev_z, mut prev_f) = (H, 26.0, 0.0, 0.0, 1e9);
        while z < ZMAX && top_r > 0 {
            let x = dir * z;
            let hy = height(x, z);
            let y = hy - z * z / (2.0 * RM);
            let yf = EYE - F * (y - cam_y) / z;
            let r0 = (yf * S - 0.5).ceil().max(0.0) as usize;
            for r in r0..top_r {
                // place the pixel between this sample and the last, by where its row falls
                let a = if prev_f > yf + 1e-6 { clamp((prev_f - row_y(r)) / (prev_f - yf)) } else { 1.0 };
                let k = r * W + c;
                ground[k] = true;
                gz[k] = if prev_z != 0.0 { mix(prev_z, z, a) } else { z };
                gx[k] = dir * gz[k];
                gy[k] = if prev_z != 0.0 { mix(prev_y, hy, a) } else { hy };
            }
            top_r = top_r.min(r0);
            (prev_f, prev_y, prev_z) = (yf, hy, z);
            // finer steps than ascii.rest's, for the finer rows
            z += (0.03 + z * 0.012) / S;
        }
        top[c] = top_r;
    }

    // Static light: the ground, everything but the Earth's disc.
    let mut base = vec![[0.0f32; 3]; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            if !ground[k] {
                continue;
            }
            let (x, z, y) = (gx[k], gz[k], gy[k]);
            let e = 0.1 + z * 0.004;
            let hx = (height(x + e, z) - height(x - e, z)) / (2.0 * e);
            let hz = (height(x, z + e) - height(x, z - e)) / (2.0 * e);
            let nl = (hx * hx + 1.0 + hz * hz).sqrt();
            let lam = (-hx * sn[0] + sn[1] - hz * sn[2]) / nl;
            // toward the eye, for the moon's own way of reflecting (Lommel-Seeliger)
            let (vx, vy, vz) = (-x, cam_y - y, -z);
            let vl = (vx * vx + vy * vy + vz * vz).sqrt();
            let mu = ((-hx * vx + vy - hz * vz) / (nl * vl)).max(0.02);
            let mut lit = 0.0;
            if lam > 0.0 {
                // march toward the sun; a soft edge for the sun's own width
                lit = 1.0;
                let mut s = 0.1 + z * 0.003;
                while s < 120.0 {
                    let (qx, qz, qy) = (x + sn[0] * s, z + sn[2] * s, y + sn[1] * s);
                    let d = qy - height(qx, qz);
                    if d < 0.0 {
                        lit = 0.0;
                        break;
                    }
                    lit = f64::min(lit, d * 30.0 / s);
                    s += 0.05 + s * 0.2;
                }
                lit = smooth(0.0, 1.0, lit);
            }
            // regolith: patchy, the maria darker
            let albedo = 0.5 + 0.9 * fbm(x * 0.025 + 3.0, z * 0.025, 3) + 0.3 * (fbm(x * 0.35, z * 0.35, 2) - 0.5)
                - 0.22 * smooth(0.46, 0.64, fbm(x * 0.0035, z * 0.0035 + 20.0, 3));
            let ls = if lam > 0.0 { (0.2 * lam / (lam + mu) + 2.6 * lam) * lit } else { 0.0 };
            // the near corners fall off a little, to frame the view
            let (xs, ys) = (col_x(px), row_y(py) - 0.5);
            let vig = 1.0 - 0.3 * smooth(84.0, 102.0, ys) * smooth(30.0, 100.0, (xs - 100.0).abs());
            // a touch under ascii.rest's, whose dots drew bright ground in grey ink
            let mut b = (1.0 - (-ls * 1.5).exp()) * albedo * vig * 0.85;
            // the far crest catches the sun along its whole length
            let crest = py - top[px];
            if crest < 3 && lit > 0.2 {
                b = b.max(if crest == 0 { 0.85 } else { 0.55 } * albedo);
            }
            // shadow is black, but for a breath of earthshine on what faces us
            let fill = if lit > 0.02 || b > 0.03 { 0.008 + 0.007 * clamp(-hz / nl + 0.5) } else { 0.0 };
            base[k] = [(b + fill * 0.75) as f32, (b * 0.95 + fill * 0.85) as f32, (b * 0.86 + fill * 1.2) as f32];
        }
    }

    // The sky: black, with faint stars that hold still, thicker along a
    // diagonal where the galaxy runs, and none round the Earth.
    let near = |x: f64, y: f64| (x + 0.5 - EC.0).hypot(y - (EC.1 - RISE / 2.0)) < 2.0 * ER;
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            if ground[k] {
                continue;
            }
            let (x, y) = (col_x(px), row_y(py));
            let band_d = (y - 0.5 - (4.0 + x * 0.32)) / 1.05;
            let band = (-(band_d / 10.0).powi(2)).exp() * smooth(0.35, 0.65, fbm(x * 0.05, (y - 0.5) * 0.08, 3)) * smooth(120.0, 70.0, x);
            let (fx, fy) = (px as f64, py as f64);
            // a cell is 2.56 pixels: a pixel star is that much rarer
            if hash(fx * 3.0 + 1.0, fy * 7.0 + 2.0) > 0.9977 - 0.02 * band && !near(x, y) {
                let m = hash(fx + 17.0, fy + 29.0).powi(3);
                let s = 0.2 + 0.5 * m;
                let tint = hash(fx + 5.0, fy + 77.0);
                let (tr, tg, tb) = if tint < 0.3 {
                    (0.84, 0.9, 1.0)
                } else if tint > 0.88 {
                    (1.0, 0.93, 0.84)
                } else {
                    (0.96, 0.96, 0.98)
                };
                base[k] = [(s * tr) as f32, (s * tg) as f32, (s * tb) as f32];
            }
        }
    }

    // --- the Earth ----------------------------------------------------------
    // Equirectangular maps, wrapping in longitude: surface colour and cloud.
    let tn = TW * TH;
    let mut elev = vec![0.0f32; tn];
    let mut cloud = vec![0.0f32; tn];
    for j in 0..TH {
        let v = (j as f64 + 0.5) / TH as f64;
        let lat = (0.5 - v) * PI;
        let al = lat.abs();
        for i in 0..TW {
            let u = i as f64 / TW as f64;
            let lon = u * TAU;
            let t = j * TW + i;
            let wx = fbm_wrap(u * 6.0, v * 3.0 + 9.0, 3, 6.0);
            elev[t] = (fbm_wrap(u * 8.0 + 1.6 * wx, v * 4.0, 5, 8.0) - 0.04 * smooth(1.2, 1.5, al)) as f32;
            // clouds: warped noise, wound into spirals around a few storms
            let (mut cx, mut cy) = (u * 14.0, v * 7.0);
            for (slon, slat, spin) in STORMS {
                let mut dl = lon - slon;
                dl -= (dl / TAU).round() * TAU;
                let (lx, ly) = (dl * lat.cos(), lat - slat);
                let dd = lx.hypot(ly);
                let a = spin * 5.0 * (-dd / 0.2).exp();
                if a * spin > 0.02 {
                    let (sa, ca) = a.sin_cos();
                    cx += (lx * ca - ly * sa - lx) / TAU * 14.0;
                    cy -= (lx * sa + ly * ca - ly) / PI * 7.0;
                }
            }
            // streaked along the winds, east to west
            let q = fbm_wrap(cx * 0.5 + 3.0, cy * 1.2, 3, 7.0);
            let n0 = fbm_wrap(cx + 1.6 * q, cy * 1.3 + 0.5 * q, 5, 14.0);
            // folded into filaments, the way weather fronts string out
            let n = 0.4 * n0 + 0.6 * (1.0 - (2.0 * fbm_wrap(cx * 1.5 + 2.2 * q, cy * 1.6 + 9.0, 4, 21.0) - 1.0).abs());
            // cloudy at the equator and in the storm belts, clearer in the subtropics
            let belt = 0.05 * (-(lat / 0.12).powi(2)).exp() - 0.07 * (-((al - 0.42) / 0.16).powi(2)).exp() + 0.05 * (-((al - 0.95) / 0.25).powi(2)).exp();
            cloud[t] = (n + belt) as f32;
        }
    }
    // thresholds by share of the globe: about three tenths land, a third cloud
    let (shore, c0, c1) = (f64::from(quantile(&elev, 0.7)), f64::from(quantile(&cloud, 0.6)), f64::from(quantile(&cloud, 0.86)));
    let mut tex = [vec![0.0f32; tn], vec![0.0f32; tn], vec![0.0f32; tn]];
    let mut sea = vec![0.0f32; tn];
    for j in 0..TH {
        let v = (j as f64 + 0.5) / TH as f64;
        let al = ((0.5 - v) * PI).abs();
        for i in 0..TW {
            let u = i as f64 / TW as f64;
            let t = j * TW + i;
            let e = f64::from(elev[t]);
            let land = smooth(shore - 0.004, shore + 0.006, e);
            let ice = smooth(1.22, 1.32, al + 0.12 * fbm_wrap(u * 12.0, v * 6.0, 2, 12.0));
            // desert in the subtropics and on high ground, forest and scrub
            // elsewhere, broken up finely so a continent is never one flat tone
            let grain = fbm_wrap(u * 36.0 + 2.0, v * 18.0, 3, 36.0) - 0.5;
            let arid =
                clamp((-((al - 0.4) / 0.22).powi(2)).exp() * (0.1 + 1.2 * fbm_wrap(u * 10.0 + 4.0, v * 5.0, 3, 10.0)) + (e - shore - 0.04) * 4.0 + 0.8 * grain);
            let relief = 1.0 + 1.2 * grain - 2.5 * (e - shore - 0.08).max(0.0);
            let (mut r, mut g, mut b) = (mix(0.13, 0.4, arid) * relief, mix(0.25, 0.34, arid) * relief, mix(0.08, 0.19, arid) * relief);
            // ocean, lighter over the shelves near the coasts
            let shelf = smooth(shore - 0.06, shore, e);
            let (or, og, ob) = (mix(0.025, 0.07, shelf), mix(0.11, 0.3, shelf), mix(0.38, 0.62, shelf));
            (r, g, b) = (mix(or, r, land), mix(og, g, land), mix(ob, b, land));
            (r, g, b) = (mix(r, 0.92, ice), mix(g, 0.95, ice), mix(b, 0.99, ice));
            tex[0][t] = r as f32;
            tex[1][t] = g as f32;
            tex[2][t] = b as f32;
            sea[t] = ((1.0 - land) * (1.0 - ice)) as f32;
            cloud[t] = smooth(c0, c1, f64::from(cloud[t])) as f32;
        }
    }

    // the sky pixels the Earth and its air can reach, as it rises and settles
    let mut reach = vec![];
    let (r0, r1) = ((EC.1 - RISE - ER - 13.0).max(0.0), EC.1 + ER + 3.0);
    let (x0, x1) = (EC.0 - ER - 13.0, EC.0 + ER + 14.0);
    for py in (r0 * S).floor() as usize..((r1 * S).ceil() as usize).min(H) {
        for px in (x0 * S).floor() as usize..((x1 * S).ceil() as usize).min(W) {
            if !ground[py * W + px] {
                reach.push((px, py));
            }
        }
    }
    // the bright stars and the faint cross each one carries
    let mut twinkle = vec![];
    for (x, r, s, period) in BRIGHT {
        let (cx, cy) = (((x + 0.5) * S) as usize, ((r + 0.5) * S) as usize);
        let phase = hash(x, r) * TAU;
        for (ox, oy, w) in [(0, 0, 1.0), (-1, 0, 0.3), (1, 0, 0.3), (0, -1, 0.3), (0, 1, 0.3), (-2, 0, 0.12), (2, 0, 0.12), (0, -2, 0.12), (0, 2, 0.12)] {
            let (px, py) = (cx.wrapping_add_signed(ox), cy.wrapping_add_signed(oy));
            if px < W && py < H && !ground[py * W + px] {
                twinkle.push(Twinkle { px, py, s: s * w, period, phase, core: w == 1.0 });
            }
        }
    }

    let mut pixels = Pixels::new(&META);
    pixels.pixels_mut().copy_from_slice(&base);
    Box::new(Earthrise { base, reach, twinkle, tex, sea, cloud, pixels })
}

impl Piece for Earthrise {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let sn = sun();
        let l = [sn[0], sn[1], -sn[2]]; // into screen space, z toward us
        let hv = unit([l[0], l[1], l[2] + 1.0]);
        let (st, ct) = 0.4f64.sin_cos();
        let (snod, cnod) = 0.22f64.sin_cos();
        let spin = t * 0.045;
        let drift = t * 0.012; // clouds run a little ahead of the ground
        let ey = EC.1 - RISE * (0.5 - 0.5 * (t / RISE_T * TAU).cos());
        // the map's rows to ascii.rest's, for the cloud shadow's offset
        let texel = TH as f64 / 96.0;
        for &(px, py) in &self.reach {
            let k = py * W + px;
            let [r0, g0, b0] = self.base[k];
            let (mut cr, mut cg, mut cb) = (f64::from(r0), f64::from(g0), f64::from(b0));
            let (dx, dy) = (col_x(px) + 0.5 - EC.0, row_y(py) - ey);
            let d = dx.hypot(dy);
            // the Earth's air, a thin blue rim on its sunlit side
            if (ER - 1.0..ER + 12.0).contains(&d) {
                let side = smooth(-0.3, 0.75, (dx * l[0] - dy * l[1]) / d);
                let mut g = (-(d - ER).max(0.0) / 1.2).exp() * 0.55 * side;
                if g < 0.05 {
                    g = 0.0; // no stray haze out in space
                }
                cr += 0.25 * g;
                cg += 0.52 * g;
                cb += g;
            }
            if d < ER {
                let (nx, ny) = (dx / ER, -dy / ER);
                let q2 = nx * nx + ny * ny;
                let nz = (1.0 - q2).sqrt();
                // into the globe's own frame: tip the pole toward us, then lean it
                let ax = nx * ct + ny * st;
                let ay0 = -nx * st + ny * ct;
                let ay = ay0 * cnod - nz * snod;
                let az = ay0 * snod + nz * cnod;
                let lat = ay.clamp(-1.0, 1.0).asin();
                let lon = ax.atan2(az) + LON0 + spin;
                let vy = (0.5 - lat / PI) * TH as f64 - 0.5;
                let ndl = nx * l[0] + ny * l[1] + nz * l[2];
                // full sun at the right limb, dimming toward the terminator, so
                // the disc reads as a ball
                let day = smooth(-0.005, 0.06, ndl) * (0.45 + 0.8 * clamp(ndl).sqrt());
                let dusk = (-((ndl - 0.02) / 0.035).powi(2)).exp();
                let cl = sample(&self.cloud, lon + drift, vy);
                // the cloud's own shadow, offset away from the sun
                let sh = sample(&self.cloud, lon + drift - 0.03, vy + 0.4 * texel);
                let sw = sample(&self.sea, lon, vy);
                let shade = 1.0 - 0.45 * sh * (1.0 - cl);
                let mut er = mix(sample(&self.tex[0], lon, vy) * shade, 0.95, cl);
                let mut eg = mix(sample(&self.tex[1], lon, vy) * shade, 0.97, cl);
                let mut eb = mix(sample(&self.tex[2], lon, vy) * shade, 1.0, cl);
                er *= day * (1.0 + 0.06 * dusk);
                eg *= day * (1.0 - 0.03 * dusk);
                eb *= day * (1.0 - 0.1 * dusk);
                let glint = (nx * hv[0] + ny * hv[1] + nz * hv[2]).max(0.0).powi(70) * 0.8 * sw * (1.0 - cl);
                let rim = (1.0 - nz).powf(2.2) * smooth(0.0, 0.3, ndl);
                er += glint * 0.95 + rim * 0.2;
                eg += glint * 0.92 + rim * 0.42;
                eb += glint * 0.85 + rim * 0.85;
                // a crisp edge where the disc meets space; the night side is a void
                let edge = smooth(1.0, 0.95, q2.sqrt());
                (cr, cg, cb) = (mix(cr, er, edge), mix(cg, eg, edge), mix(cb, eb, edge));
            }
            self.pixels.set(px, py, [cr as f32, cg as f32, cb as f32]);
        }
        for s in &self.twinkle {
            let v = s.s * (0.86 + 0.14 * (t / s.period * TAU + s.phase).sin());
            let c = if s.core {
                [v * 0.97, v * 0.98, v]
            } else {
                let [r, g, b] = self.base[s.py * W + s.px];
                [f64::from(r).max(v * 0.95), f64::from(g).max(v), f64::from(b).max(v * 1.15)]
            };
            self.pixels.set(s.px, s.py, [c[0] as f32, c[1] as f32, c[2] as f32]);
        }
        self.pixels.draw(grid, env);
    }
}
