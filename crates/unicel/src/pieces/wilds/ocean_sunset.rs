//! ocean sunset: golden hour at sea. The sun rests on the horizon beyond a dark
//! pine headland, heaped cloud overhead lit from below, a glitter path running
//! across the water toward us, and long swells rolling in, their crests
//! catching the light. After ascii.rest's ocean-sunset.ts (MIT, @bas3line).
//!
//! The headland and its pines, the sloop, the cloud deck on its plane, the low
//! bars, and the sea that mirrors the finished sky facet by facet are
//! ascii.rest's, shaded here per pixel instead of per cell. Where ascii.rest
//! draws each of 200 × 100 square cells as a dot as big as it is bright, this
//! draws 320 × 80 cells of two square pixels each, an upper half block over a
//! back colour ([`crate::pixels`]): 1.6 times the detail each way, solid
//! colour, and gradients dithered in colour rather than in dot size. The
//! palette adds the dusk violets and indigos a solid pixel needs where a small
//! dot was enough. The sun's lower limb is flattened a little more toward the
//! horizon, as refraction squashes a setting sun, and the glitter path
//! carries one-pixel sparkles that come and go from frame to frame. In one ink
//! each pixel is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::{Pixels, Rgb};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Ocean sunset's meta.
pub static META: Meta = Meta {
    name: "ocean sunset",
    slug: "ocean-sunset",
    category: Category::Wilds,
    note: "the sun sets past a pine headland, lit cloud, glitter on rolling sea",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: dusk violets, roses, the sun's golds, the reds of the
    // rim, mauves, peach, the indigos of the high sky and lilacs. Then the
    // darks of the rock, the pines and the sea toward us.
    palette: Some(&[
        "#140f2c", "#1e1640", "#2b1c52", "#3d2266", "#552a74", "#6e2c78", "#90327c", "#b23c7c", "#d24f7a", "#ea6a78", "#c2306a", "#ff5d8f", "#ff8a9a",
        "#7a2456", "#f05d5e", "#f2804f", "#f99a43", "#ffb44a", "#ffcd62", "#ffe39a", "#fff5d8", "#a8323c", "#d2453c", "#e8603e", "#2f1a3e", "#47234f",
        "#63305c", "#874266", "#ac5b70", "#ff9e86", "#ffc4a2", "#ffdcc0", "#0e1230", "#171a46", "#232059", "#33286a", "#4a2f78", "#1c2a5c", "#2e4078",
        "#7d3f8f", "#a24f98", "#c8649e", "#de8bb5", "#0b0817", "#07050f", "#0e0a20", "#130d2c", "#1a1238", "#100c26", "#1c1030", "#28143a", "#36183e",
    ]),
    ground: Some("#0b0817"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// The horizon, in ascii.rest's rows.
const HZ: f64 = 56.0;
/// The pixel rows above the horizon: those whose middle is above row 56.
const SKY: usize = 90;
const SUN: (f64, f64) = (134.0, HZ - 4.5);
/// The sun's radius.
const SR: f64 = 8.5;
/// The sloop's mast.
const BOAT: f64 = 112.0;
/// Wind chop on the swell: [dir x, dir z, wavenumber, slope, speed, offset].
const CHOP: [[f64; 6]; 4] =
    [[-0.45, 0.89, 22.0, 0.07, 2.1, 0.4], [0.5, 0.87, 37.0, 0.06, 2.9, 2.1], [-0.15, 0.99, 61.0, 0.05, 3.7, 4.4], [0.3, 0.95, 97.0, 0.04, 4.6, 1.3]];
/// Pines on the headland: (x, height, half width at the foot).
const PINES: [(f64, f64, f64); 8] =
    [(2.5, 10.0, 2.6), (8.0, 13.0, 3.0), (13.0, 18.0, 3.6), (18.5, 11.0, 2.8), (24.0, 20.0, 3.8), (30.0, 12.0, 2.9), (35.0, 8.0, 2.3), (39.5, 5.0, 1.8)];
/// The headland's extent: columns and rows.
const LANDW: f64 = 60.0;
const LANDH: f64 = HZ + 10.0;
/// The cloud deck's size, on its own plane.
const U: usize = 640;
const V: usize = 128;
/// The low bars' width in columns: they wrap, so they can drift forever.
const CW: usize = 720;

/// What stands in front of the sky and the sea.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Open,
    Rock,
    Pine,
    Boat,
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / S - 0.5
}

/// The pixel a scene position (a cell's `x + 0.5`, `y`) falls in, along one axis.
fn to_px(pos: f64, n: usize) -> usize {
    ((pos * S).floor().max(0.0) as usize).min(n - 1)
}

/// The sky's gradient, top to horizon, as [stop, r, g, b].
const STOPS: [[f64; 4]; 6] =
    [[0.0, 0.08, 0.1, 0.3], [0.3, 0.15, 0.1, 0.34], [0.52, 0.4, 0.15, 0.4], [0.72, 0.68, 0.27, 0.4], [0.88, 0.8, 0.32, 0.4], [1.0, 0.88, 0.4, 0.4]];

/// The sky's colour `v` of the way down to the horizon.
fn gradient(v: f64) -> [f64; 3] {
    let mut i = 1;
    while i < STOPS.len() - 1 && v > STOPS[i][0] {
        i += 1;
    }
    let (a, b) = (STOPS[i - 1], STOPS[i]);
    let k = clamp((v - a[0]) / (b[0] - a[0]));
    [mix(a[1], b[1], k), mix(a[2], b[2], k), mix(a[3], b[3], k)]
}

/// The headland's skyline at x.
fn ridge(x: f64) -> f64 {
    HZ + 2.5 - 11.0 * smooth(54.0, 32.0, x).powf(0.7) - 2.4 * (-((x - 19.0) / 9.0).powi(2)).exp() - 2.0 * fbm(x * 0.21, 3.3, 3)
}

/// The row where the headland's foot meets the sea at x.
fn shore(x: f64) -> f64 {
    HZ + 1.5 + 5.0 * smooth(54.0, 4.0, x)
}

/// A sea stack off the point: its top at x, or far below the frame.
fn stack(x: f64) -> f64 {
    if x > 52.0 && x < 58.0 {
        HZ - 3.5 - 1.6 * fbm(x * 0.6, 8.1, 2) + 2.5 * ((x - 55.0) / 3.0).powi(4)
    } else {
        1e9
    }
}

/// The cloud deck's density at (u, v) on its plane.
fn deck_at(u: f64, v: f64) -> f64 {
    let q = fbm_wrap(u * 0.0125, v * 0.06, 2, 8.0);
    let patch = fbm_wrap(u * 0.00625, v * 0.03 + 7.0, 2, 4.0);
    fbm_wrap(u * 0.040625 + q * 1.6, v * 0.3, 5, 26.0) + 0.25 * (patch - 0.5)
}

/// The low bars' density at (x, y): long thin bars low over the horizon.
fn lower(x: f64, y: f64) -> f64 {
    let b = fbm_wrap(x / 40.0, y * 0.3, 4, CW as f64 / 40.0);
    let gaps = fbm_wrap(x / 120.0, 5.0, 2, CW as f64 / 120.0);
    b + 0.01 + 0.5 * (gaps - 0.5) - 0.35 * smooth(HZ - 14.0, HZ - 20.0, y) - 0.25 * smooth(HZ - 4.0, HZ - 1.0, y)
}

/// The sunset: the clear sky, the headland and the sloop, the cloud deck and
/// the low bars, built once; this frame's sky for the sea to mirror; and the
/// picture.
pub struct OceanSunset {
    sky: Vec<Rgb>,
    mat: Vec<Mat>,
    land: Vec<Rgb>,
    deck: Vec<f32>,
    deck_lit: Vec<f32>,
    thin_top: Vec<f32>,
    lo_cover: Vec<f32>,
    lo_lit: Vec<f32>,
    mirror: Vec<Rgb>,
    pixels: Pixels,
}

/// An ocean sunset; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // --- the clear sky ---
    let mut sky = vec![[0.0f32; 3]; SKY * W];
    for py in 0..SKY {
        let y = row_y(py);
        for px in 0..W {
            let x = col_x(px);
            let dx = x + 0.5 - SUN.0;
            let dy = (y - SUN.1) * 1.6;
            let d = dx.hypot(dy);
            // rose and violet everywhere, burning gold only toward the sun
            let near = (-dx.abs() / 62.0).exp();
            let v = y / HZ;
            let mut g = gradient(v.powf(1.0 + 0.12 * (1.0 - near)));
            let warm = (-dx.abs() / 36.0).exp() * smooth(0.45, 1.0, v) * 0.9;
            let lift = mix(0.6, 1.0, v);
            g = [mix(g[0], lift, warm), mix(g[1], 0.64 * lift, warm), mix(g[2], 0.3, warm)];
            let fall = mix(1.0, 0.86 + 0.14 * near, smooth(0.6, 1.0, v));
            let glow = (-d / 40.0).exp() * 0.15 + (-d / 9.0).exp() * 0.1;
            // high haze in long thin bands, so no stretch of sky is one flat tone
            let haze = 0.82 + 0.36 * fbm(x * 0.03, y * 0.12, 3);
            let vig = (1.0 - 0.08 * ((x + 0.5 - 100.0).abs() / 100.0).powi(2)) * haze * fall;
            sky[py * W + px] = [
                (g[0] * (0.85 + 0.15 * near) * vig + glow) as f32,
                (g[1] * (0.65 + 0.35 * near) * vig + glow * 0.7) as f32,
                (g[2] * (1.1 - 0.2 * near) * vig + glow * 0.3) as f32,
            ];
        }
    }

    // --- the headland and its pines ---
    let mut mat = vec![Mat::Open; W * H];
    let mut land = vec![[0.0f32; 3]; W * H];
    let land_rows = (LANDH * S).ceil() as usize;
    let land_cols = (LANDW * S).ceil() as usize;
    // the rock's seaward edge on each pixel row, for the light on the cliff face
    let edge: Vec<f64> = (0..land_rows)
        .map(|py| {
            let y = row_y(py);
            (0..land_cols).map(col_x).filter(|&x| y >= ridge(x + 0.5) && y < shore(x + 0.5)).fold(-1.0, f64::max)
        })
        .collect();
    for (py, &edge) in edge.iter().enumerate() {
        let y = row_y(py);
        for px in 0..land_cols {
            let (x, k) = (col_x(px), py * W + px);
            let xc = x + 0.5;
            let (t0, st) = (ridge(xc), stack(xc));
            let mut c = None;
            let on_land = y >= t0 && y < shore(xc);
            if on_land || (y >= st && y < HZ + 1.2) {
                let is_stack = !on_land;
                let top = if is_stack { st } else { t0 };
                // dark violet rock in rough strata
                let s = 0.6 + 0.8 * fbm(x * 0.22, y * 0.7, 3);
                let (cr, cg, cb) = (0.06 * s, 0.042 * s, 0.15 * s);
                // the sun is low and to the right: slopes that drop toward it catch it
                let facing = clamp(0.4 + if is_stack { 0.5 } else { (ridge(xc + 1.2) - ridge(xc - 1.2)) * 0.45 });
                let rim = smooth(top + 2.4, top + 0.4, y) * facing;
                // the cliff face, warm where it turns to the sun, broken by crags
                let crag = smooth(0.35, 0.75, fbm(x * 0.4 + 3.0, y * 0.5, 3));
                let face = if is_stack {
                    smooth(53.5, 57.5, xc) * 0.7
                } else if edge > 30.0 {
                    (-(edge - x) / 4.0).exp() * smooth(HZ + 5.0, HZ - 4.0, y)
                } else {
                    0.0
                };
                let lit = clamp((rim * 0.9).max(face * (0.3 + 0.7 * crag) * 0.75));
                c = Some((Mat::Rock, mix(cr, 0.95, lit), mix(cg, 0.38, lit * 0.95), mix(cb, 0.3, lit * 0.9)));
            }
            for &(tx, th, tw) in &PINES {
                let tb = ridge(tx);
                let dy = y - (tb - th);
                // a conifer: a spire that widens in tiers of branches
                let tier = (dy + th * 0.3) / 2.4;
                let half = (dy / th) * tw * (0.5 + 0.75 * (tier - tier.floor())) + 0.35;
                let ex = xc - tx;
                if dy >= 0.0 && y < tb + 1.5 && ex.abs() <= half {
                    let s = 0.7 + 0.6 * hash(x * 13.0 + y.floor(), 5.0);
                    // the right flank faces the sun
                    let e = if ex > 0.0 && ex > half - 1.1 { 0.3 + 0.3 * smooth(th, 0.0, dy) } else { 0.0 };
                    c = Some((Mat::Pine, mix(0.045 * s, 0.85, e), mix(0.03 * s, 0.3, e), mix(0.1 * s, 0.3, e)));
                }
            }
            if let Some((m, r, g, b)) = c {
                mat[k] = m;
                land[k] = [r as f32, g as f32, b as f32];
            }
        }
    }
    // a small sloop out on the water, dark against the glow left of the sun
    for py in to_px(HZ - 10.0, H)..to_px(HZ + 4.0, H) {
        let y = row_y(py);
        for px in to_px(BOAT - 7.0, W)..=to_px(BOAT + 8.0, W) {
            let ex = col_x(px) + 0.5 - BOAT;
            let hull = (HZ + 1.0..HZ + 3.0).contains(&y) && (ex - 0.3).abs() < 4.6 - (y - HZ - 1.0) * 1.3;
            let mast = ex.abs() < 0.5 && (HZ - 9.0..HZ + 1.0).contains(&y);
            let main = ex > 0.0 && (HZ - 8.5..HZ + 0.5).contains(&y) && ex < 0.6 + (y - (HZ - 8.5)) * 0.42;
            let jib = ex < 0.0 && (HZ - 7.0..HZ + 0.5).contains(&y) && -ex < (y - (HZ - 7.0)) * 0.36;
            if hull || mast || main || jib {
                // the sails are thin enough to glow a little with the sun behind them
                let glow = if main { 0.12 + 0.18 * smooth(0.0, 3.5, ex) } else { 0.0 };
                mat[py * W + px] = Mat::Boat;
                land[py * W + px] = [(0.05 + glow) as f32, (0.03 + glow * 0.45) as f32, (0.09 + glow * 0.35) as f32];
            }
        }
    }

    // --- the cloud deck: a sheet of heaped cloud seen from below, laid out on
    // its own plane so it shrinks and flattens toward the horizon ---
    let mut deck = vec![0.0f32; U * V];
    let mut deck_lit = vec![0.0f32; U * V];
    for v in 0..V {
        for u in 0..U {
            let d = deck_at(u as f64, v as f64);
            deck[v * U + u] = d as f32;
            // the side of each heap that faces the horizon, and the sun, is lit
            deck_lit[v * U + u] = clamp(0.5 + (d - deck_at(u as f64, v as f64 + 2.5)) * 9.0) as f32;
        }
    }
    // thinning toward the zenith, ragged rather than a ruled line
    let thin_top = (0..SKY * W).map(|k| fbm(col_x(k % W) * 0.05 + 11.0, (row_y(k / W) - 0.5) * 0.15, 2) as f32).collect();

    // --- low bars over the horizon: a wrapping field that drifts ---
    let mut lo_cover = vec![0.0f32; CW * SKY];
    let mut lo_lit = vec![0.0f32; CW * SKY];
    for py in 0..SKY {
        let y = row_y(py);
        for x in 0..CW {
            let dl = lower(x as f64, y);
            lo_cover[py * CW + x] = smooth(0.58, 0.66, dl) as f32;
            lo_lit[py * CW + x] = clamp(0.5 + (dl - lower(x as f64, y + 1.2)) * 6.0) as f32;
        }
    }

    Box::new(OceanSunset { sky, mat, land, deck, deck_lit, thin_top, lo_cover, lo_lit, mirror: vec![[0.0; 3]; SKY * W], pixels: Pixels::new(&META) })
}

impl OceanSunset {
    /// The sky above the horizon, and this frame's sky for the sea to mirror.
    fn sky(&mut self, t: f64) {
        let (d_up, d_lo) = (t * 0.9, t * 1.7);
        for py in 0..SKY {
            let y = row_y(py);
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let [r, g, b] = self.sky[k];
                let (mut cr, mut cg, mut cb) = (f64::from(r), f64::from(g), f64::from(b));
                let dx = x + 0.5 - SUN.0;
                let dy = y - SUN.1;
                // the disc, a little flattened, and more so toward the horizon
                // as the air bends its lower limb up
                let squash = 0.9 - 0.16 * smooth(0.0, HZ - SUN.1, dy);
                let ds = dx.hypot(dy / squash);
                let mut disc = 0.0;
                if ds < SR + 0.6 {
                    // hot at the core and redder at the rim
                    let e4 = (ds / SR).powi(4);
                    disc = smooth(SR + 0.45, SR - 0.45, ds);
                    cr = mix(cr, 1.0, disc);
                    cg = mix(cg, 0.95 - 0.2 * e4, disc);
                    cb = mix(cb, 0.78 - 0.38 * e4, disc);
                }
                let near = (-(dx * dx + dy * dy * 2.5).sqrt() / 45.0).exp();
                let v = y / HZ;
                // the cloud deck, found on its plane: far rows are far away
                let dd_ = 58.0 / (HZ - y + 2.5);
                // a band of heaped cloud mid-sky: clear above and clear in the low glow
                let far = smooth(4.8, 2.7, dd_) * smooth(1.08, 1.7, dd_);
                let mut cloud = 0.0;
                if far > 0.0 {
                    let su = dx * dd_ + d_up + 64200.0;
                    let sv = (dd_ - 0.9) * 10.0;
                    let (iu, iv) = (su.floor(), sv.floor());
                    let (fu, fv) = (su - iu, sv - iv);
                    let iu = (iu as i64).rem_euclid(U as i64) as usize;
                    let iu1 = (iu + 1) % U;
                    let a0 = iv as usize * U;
                    let a1 = a0 + U;
                    let lerp = |f: &[f32], a: usize| f64::from(f[a + iu]) + f64::from(f[a + iu1] - f[a + iu]) * fu;
                    let (d0, d1) = (lerp(&self.deck, a0), lerp(&self.deck, a1));
                    let dd = d0 + (d1 - d0) * fv - 0.22 * (1.0 - far) * f64::from(self.thin_top[k]);
                    let c = smooth(0.56, 0.68, dd) * far.sqrt();
                    if c > 0.01 {
                        cloud = c;
                        let (l0, l1) = (lerp(&self.deck_lit, a0), lerp(&self.deck_lit, a1));
                        let l = l0 + (l1 - l0) * fv;
                        // undersides glow gold toward the sun and rose away
                        // from it, and more the lower they sit; thick cores
                        // stay dusk violet
                        let az = (-dx.abs() / 60.0).exp();
                        let low = v * v;
                        let thin = 1.0 - smooth(0.6, 0.76, dd);
                        let b = clamp(smooth(0.45, 0.85, l).powf(1.5) * (0.55 + 0.25 * low + 0.35 * az) + 0.3 * thin * az * low);
                        let warm = clamp(az * (0.1 + 1.2 * low));
                        let (hr, hg, hb) = (1.0, mix(0.45, 0.72, warm), mix(0.52, 0.38, warm));
                        let (sr, sg, sb) = (mix(0.1, 0.2, v), mix(0.065, 0.07, v), mix(0.23, 0.27, v));
                        cr = mix(cr, mix(sr, hr, b), c);
                        cg = mix(cg, mix(sg, hg, b), c);
                        cb = mix(cb, mix(sb, hb, b), c);
                    }
                }
                // the low bars, dark against the glow with burning edges
                let sx = x + d_lo;
                let ix = sx.floor();
                let fx = sx - ix;
                let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
                let (i0, i1) = (py * CW + i0, py * CW + (i0 + 1) % CW);
                // thinned over the sun and kept off the sky behind the headland
                let lc = f64::from(self.lo_cover[i0]) + f64::from(self.lo_cover[i1] - self.lo_cover[i0]) * fx;
                let c = lc * (1.0 - 0.75 * (-(dx / 13.0).powi(2)).exp()) * (1.0 - 0.85 * smooth(76.0, 50.0, x));
                if c > 0.01 {
                    cloud = f64::max(cloud, c);
                    let l = f64::from(self.lo_lit[i0]) + f64::from(self.lo_lit[i1] - self.lo_lit[i0]) * fx;
                    let rim = l * l * (0.35 + 0.9 * near);
                    let a = (c * 1.2).min(1.0) * 0.92;
                    cr = mix(cr, 0.3 + 0.7 * rim, a);
                    cg = mix(cg, 0.09 + 0.55 * rim, a);
                    cb = mix(cb, 0.24 + 0.18 * rim, a);
                }
                // the first stars, a pixel big, high up where the sky has gone to indigo
                if y < 22.0 && cloud < 0.05 && hash(px as f64, py as f64 * 5.0 + 3.0) > 0.9973 {
                    let (fx, fy) = (px as f64, py as f64);
                    let tw = 0.55 + 0.45 * (t * (0.8 + hash(fy, fx) * 1.6) + hash(fx, fy) * TAU).sin();
                    let st = tw * smooth(22.0, 4.0, y - 0.5) * 0.75;
                    (cr, cg, cb) = (cr.max(st * 0.95), cg.max(st * 0.85), cb.max(st));
                }
                // the sea is too rough to mirror the disc whole: it gives back glitter
                let keep = 1.0 - 0.75 * disc;
                self.mirror[k] = [(cr * keep) as f32, (cg * keep) as f32, (cb * keep) as f32];
                let c = if self.mat[k] == Mat::Open { [cr as f32, cg as f32, cb as f32] } else { self.land[k] };
                self.pixels.set(px, py, c);
            }
        }
    }

    /// The sea below the horizon: each pixel a facet of water that mirrors
    /// whatever part of the sky its tilt points it at.
    fn sea(&mut self, t: f64) {
        let ground = [0.043, 0.031, 0.09];
        let frame = (t * 15.0).round();
        for py in SKY..H {
            let y = row_y(py);
            let dz = y - HZ;
            let z = 36.0 / dz; // distance out
            let rows = 36.0 / (dz * dz); // how much sea one row spans
            let swell_amt = smooth(8.0, 26.0, dz);
            let hot = (-dz / 9.0).exp();
            let pw = 3.0 + dz * 0.8; // the glitter path, wider as it comes toward us
            let (fx, fy) = ((0.9 / (1.0 + dz * 0.06)) * 0.35, 1.6 / (1.0 + dz * 0.05));
            let fres = 0.24 + 0.46 * (-dz / 8.0).exp();
            let depth = smooth(HZ, 100.0, y);
            let fade = smooth(106.0, 96.0, y); // the bottom rows sink into the ground
            for px in 0..W {
                let k = py * W + px;
                if self.mat[k] != Mat::Open {
                    self.pixels.set(px, py, self.land[k]);
                    continue;
                }
                let x = col_x(px);
                let dx = x + 0.5 - SUN.0;
                let xx = (x + 0.5 - 100.0) / dz;
                // the long swell rolls toward us; shorter chop rides on it
                let p = 13.0 * (z + 0.05 * xx) + 2.2 * fbm(xx * 0.35 + 3.0, z * 0.4, 2) + t * 0.8;
                let (wv, slope) = p.sin_cos();
                let mut sz = 0.11 * slope * clamp(1.4 / (13.0 * rows));
                let mut sxl = 0.015 * slope;
                for &[kx, kz, kk, a, w, f] in &CHOP {
                    let ph = kk * (kx * xx + kz * z) - w * t + f;
                    let c = ph.cos() * a * clamp(1.6 / (kk * rows));
                    sz += c * kz;
                    sxl += c * kx;
                }
                let q = to_px(HZ - 0.5 - dz * 0.85 - 70.0 * sz, SKY) * W + to_px(x + 0.5 - 60.0 * sxl, W);
                // broken into long horizontal ripples, each giving back a
                // little more or less of the sky: violet near the horizon,
                // deepening to indigo toward us
                let rip = noise(x * fx * 1.4 + 13.0 - t * 0.15, y * fy * 1.1 + t * 0.25);
                let rf = fres * (0.5 + 1.1 * rip * rip);
                let [mr, mg, mb] = self.mirror[q];
                let mut cr = f64::from(mr) * rf * 0.8 + 0.03 - 0.01 * depth;
                let mut cg = f64::from(mg) * rf * 0.75 + 0.022;
                let mut cb = f64::from(mb) * rf + 0.095 + 0.03 * depth;
                // ripple facets that catch the sun: short dashes far out,
                // longer close in, each turning toward the sun and away again
                let n = 0.55 * noise(x * fx + t * 0.3, y * fy * 1.3 - t * 0.4) + 0.45 * noise(x * fx * 1.7 - t * 0.4, y * fy * 2.0 + t * 0.3 + 40.0);
                let wave = (p / TAU).floor();
                let crest = (0.5 + 0.5 * wv).powi(7) * swell_amt * smooth(0.4, 0.65, noise(xx * 2.5 + 7.0, wave * 3.7));
                let path = (-(dx / pw).powi(2)).exp();
                let soft = (-(dx / (pw * 1.6)).powi(2)).exp();
                // the near face of a swell is in shadow
                if slope > 0.0 {
                    let d = 1.0 - 0.7 * swell_amt * slope;
                    (cr, cg, cb) = (cr * d, cg * d, cb * d);
                }
                let th = 0.84 - 0.3 * path * (0.45 + 0.55 * hot);
                let mut glint = smooth(th, th + 0.08, n) * (0.25 + 0.75 * path) * soft;
                // and single pixels of glitter that flash for a frame in the path
                if hash(px as f64 * 7.0 + frame * 131.0, py as f64 * 13.0 - frame * 17.0) > 1.0 - 0.06 * path * smooth(th - 0.25, th, n) {
                    glint += 0.9;
                }
                let white = path * path * smooth(th + 0.04, th + 0.2, n);
                cr += 0.2 * soft * (0.4 + hot) + glint * 1.1;
                cg += 0.1 * soft * (0.4 + hot) + glint * (0.55 + 0.3 * hot + 0.35 * white);
                cb += 0.04 * soft + glint * (0.2 + 0.25 * hot + 0.45 * white);
                // the crests catch it: gold in the path, rose out to the sides
                let catch = crest * (0.15 + 0.85 * soft) * (0.6 + 0.6 * n);
                cr += catch * 0.95;
                cg += catch * mix(0.3, 0.7, soft);
                cb += catch * mix(0.5, 0.3, soft);
                // the sun's own column: a gap under the disc, then broken dashes
                let dash = smooth(0.4, 0.7, noise(x * 0.18 + t * 0.25, y * 0.9 - t * 0.5));
                let col = (-(dx / (1.0 + dz * 0.22)).powi(2)).exp() * (-dz / 5.0).exp() * (0.6 + 0.4 * n) * smooth(0.0, 3.0, dz) * dash;
                cr += col;
                cg += col * 0.8;
                cb += col * 0.45;
                // the headland upside down in the water, broken by ripples
                if x < LANDW + 4.0 && dz < 26.0 {
                    let xs = (x + 0.5 + 1.3 * (y * 1.1 + t * 1.2 + x * 0.05).sin()).clamp(0.0, LANDW - 0.01);
                    let ft = shore(xs);
                    if y > ft {
                        let m = to_px(2.0 * ft - y, H) * W + to_px(xs, W);
                        if matches!(self.mat[m], Mat::Rock | Mat::Pine) {
                            let [lr, lg, lb] = self.land[m];
                            let a = 0.8 * smooth(ft + 24.0, ft + 4.0, y);
                            cr = mix(cr, f64::from(lr) * 0.8 + 0.02, a);
                            cg = mix(cg, f64::from(lg) * 0.7 + 0.015, a);
                            cb = mix(cb, f64::from(lb) * 0.85 + 0.05, a);
                        }
                        // a line of surf where rock meets water
                        let foam = smooth(ft + 1.6, ft + 0.3, y) * smooth(0.45, 0.8, noise(x * 0.5 - t * 0.4, t * 0.3));
                        cr += 0.5 * foam;
                        cg += 0.3 * foam;
                        cb += 0.35 * foam;
                    }
                }
                // and the sloop's, shorter and more broken
                if dz > 2.5 && dz < 14.0 && x > BOAT - 8.0 && x < BOAT + 8.0 {
                    let xs = x + 0.5 + 0.9 * (y * 1.3 + t * 1.6).sin();
                    if self.mat[to_px(2.0 * (HZ + 3.0) - y, H) * W + to_px(xs, W)] == Mat::Boat {
                        let a = 0.7 * smooth(HZ + 14.0, HZ + 4.0, y) * (0.6 + 0.4 * rip);
                        cr = mix(cr, 0.04, a);
                        cg = mix(cg, 0.03, a);
                        cb = mix(cb, 0.1, a);
                    }
                }
                // a crisp dark line at the horizon for the sun to sit on
                if dz < 1.5 {
                    (cr, cg, cb) = (cr * 0.55, cg * 0.55, cb * 0.55);
                }
                self.pixels.set(px, py, [mix(ground[0], cr, fade) as f32, mix(ground[1], cg, fade) as f32, mix(ground[2], cb, fade) as f32]);
            }
        }
    }
}

impl Piece for OceanSunset {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        self.sky(t);
        self.sea(t);
        self.pixels.draw(grid, env);
    }
}
