//! aurora fjord: curtains of aurora ripple over a fjord between snowy
//! mountains. The still water holds a broken shimmer of them, and a red cabin
//! on the far shore keeps its lamps lit. After ascii.rest's aurora-fjord.ts
//! (MIT, @bas3line).
//!
//! The ranges, the spruce, the cabin, the two curtains and the mirror in the
//! water, and how each is shaded, are ascii.rest's, shaded here per pixel
//! instead of per cell. Where ascii.rest draws each of 200 × 100 square cells
//! as a dot as big as it is bright, this draws 320 × 80 cells of two square
//! pixels each, an upper half block over a back colour ([`crate::pixels`]):
//! 1.6 times the detail each way, solid colour, and gradients dithered in
//! colour rather than in dot size. Stars are a pixel each, a little rarer so
//! the sky holds as many per area. The palette adds the dark navies, rock,
//! spruce and timber a solid pixel needs where a small dot was enough. In one
//! ink each pixel is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{FRAC_PI_2, TAU};

/// Aurora fjord's meta.
pub static META: Meta = Meta {
    name: "aurora fjord",
    slug: "aurora-fjord",
    category: Category::Nights,
    note: "aurora curtains rippling over a still fjord, a cabin lit on the shore",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: night navies, the aurora's greens, teals and violets,
    // snow and rock, spruce, the cabin's reds and the lamps' golds. Then the
    // night's darks.
    palette: Some(&[
        "#0b1322", "#101b30", "#16243f", "#1e3050", "#2a3f63", "#0e3a33", "#11573f", "#167a4c", "#22a05c", "#3ccb73", "#7cf0a0", "#c8ffdc", "#0f5f5c",
        "#16877f", "#2cb5a6", "#7fe6d6", "#2a1f52", "#432b78", "#6a3c9f", "#9558c6", "#c48ae4", "#363a72", "#4f5596", "#1b4f63", "#1a2236", "#283350",
        "#3b4a6e", "#566a92", "#7d91b8", "#a9bad9", "#d6e0f2", "#9fc9cf", "#0f1a1c", "#173128", "#24493a", "#4a1517", "#7c2420", "#b23a2a", "#d65aa8",
        "#ffd27c", "#ffb04a", "#fff2c4", "#eef3ff", "#05080f", "#070c1a", "#0a1020", "#0d111e", "#141a2a", "#060b0e", "#0a1416", "#2a0f0d", "#5e1812",
        "#0a2420", "#123b2c",
    ]),
    ground: Some("#05080f"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
/// The waterline, in ascii.rest's rows.
const WL: f64 = 62.0;
/// The cabin's walls, x from and to.
const CAB: (f64, f64) = (142.0, 161.0);
const PANES: [(f64, f64); 2] = [(145.0, 148.0), (156.0, 159.0)];
const DOOR_X: (f64, f64) = (150.0, 152.0);
/// Pane centres and their strength.
const LAMPS: [(f64, f64); 2] = [(147.0, 1.0), (158.0, 0.8)];
/// The rays' lattice: four to an ascii.rest column, wrapping.
const RAYS: usize = 4 * TW as usize;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Air,
    Near,
    Far,
    Shore,
    Wall,
    Roof,
    Pane,
    Tree,
    Door,
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

/// The ascii.rest cell a scene x falls in.
fn cell(x: f64) -> f64 {
    (x + 0.5).floor()
}

/// Ranges as tent peaks (x, height, slope), roughened.
const LEFT: [(f64, f64, f64); 4] = [(25.0, 38.0, 1.3), (6.0, 29.0, 0.9), (50.0, 25.0, 1.0), (72.0, 13.0, 0.6)];
const RIGHT: [(f64, f64, f64); 4] = [(172.0, 30.0, 1.15), (194.0, 24.0, 0.85), (151.0, 16.0, 1.0), (212.0, 22.0, 0.6)];
const DISTANT: [(f64, f64, f64); 4] = [(100.0, 10.0, 0.5), (119.0, 12.0, 0.55), (86.0, 7.0, 0.45), (134.0, 8.0, 0.5)];

/// A range's height at x, and the peak it belongs to.
fn range(x: f64, peaks: &[(f64, f64, f64)], seed: f64, rough: f64) -> (f64, f64) {
    let (mut m, mut px) = (-99.0, 0.0);
    for &(cx, h, s) in peaks {
        let v = h - (x - cx).abs() * s;
        if v > m {
            (m, px) = (v, cx);
        }
    }
    let j = rough * (fbm(x * 0.09, seed, 3) - 0.5) + rough * 0.45 * (noise(x * 0.45, seed + 5.0) - 0.5);
    (m + j * (m.max(0.0) / 6.0).min(1.0), px)
}

/// The top of the shelf the cabin stands on.
fn shore_top(x: f64) -> f64 {
    WL - 2.2 * smooth(134.0, 140.0, x) * smooth(178.0, 168.0, x) - 0.6 * noise(x * 0.3, 2.0)
}

/// The land, built once: what each pixel is, its colour, the aurora light it
/// picks up and the rim it catches.
struct Land {
    mat: Vec<Mat>,
    col: Vec<[f64; 3]>,
    rec: Vec<f32>,
    rim: Vec<f32>,
}

impl Land {
    /// A spruce at tx, th tall and tw wide, its foot at tb, skipping pixels
    /// `guard` keeps.
    fn spruce(&mut self, tx: f64, th: f64, tw: f64, tb: f64, guard: impl Fn(Mat) -> bool) {
        let x0 = ((tx - tw - 1.5) * S).floor().max(0.0) as usize;
        let x1 = (((tx + tw + 1.5) * S).ceil().max(0.0) as usize).min(W);
        for py in 0..H {
            let y = row_y(py);
            let dy = y - (tb - th);
            if y >= WL || dy < 0.0 || y >= tb + 0.5 {
                continue;
            }
            let tier = (dy + th * 0.3) / 1.8;
            let w = (dy / th) * tw * (0.7 + 0.45 * (tier - tier.floor())) + 0.35;
            for px in x0..x1 {
                let k = py * W + px;
                let ex = col_x(px) + 0.5 - tx;
                if ex.abs() > w || guard(self.mat[k]) {
                    continue;
                }
                self.mat[k] = Mat::Tree;
                let h = hash((px * 13 + py) as f64, 5.0);
                let s = if ex < 0.0 { 0.7 } else { 0.3 }; // the aurora is up and left
                self.col[k] = [0.02 + 0.03 * s * h, 0.04 + 0.05 * s, 0.055 + 0.045 * s];
                self.rec[k] = 0.05;
                let edge = if ex < 0.0 && ex < -w + 1.0 { 0.45 * smooth(th, 0.0, dy) } else { 0.0 };
                self.rim[k] = smooth(1.6, 0.2, dy).max(edge) as f32;
            }
        }
    }
}

/// The fjord: the land, the lamplight on it, the sky's haze and stars, the
/// water's broken rows, and each frame's aurora.
pub struct AuroraFjord {
    land: Land,
    lamp_land: Vec<f32>,
    haze: Vec<f32>,
    star: Vec<f32>,
    eave: f64,
    /// Per pixel column: the main curtain's hem, height and strength, the far
    /// curtain's hem and strength, and the aurora's light on the land below.
    base_a: Vec<f64>,
    tall_a: Vec<f64>,
    env_a: Vec<f64>,
    base_b: Vec<f64>,
    env_b: Vec<f64>,
    light_x: Vec<f64>,
    rays_a: Vec<f64>,
    rays_b: Vec<f64>,
    /// The sky and the land above the waterline, this frame.
    above: Vec<[f64; 3]>,
    pixels: Pixels,
}

/// The rays at x, between lattice points, wrapping.
fn ray(arr: &[f64], u: f64) -> f64 {
    let s = u * 4.0;
    let i = s.floor();
    let f = s - i;
    let i = (i as i64).rem_euclid(RAYS as i64) as usize;
    arr[i] + (arr[i + 1] - arr[i]) * f
}

/// The pixel rows above the waterline.
fn land_rows() -> usize {
    (WL * S - 0.5).ceil() as usize
}

/// An aurora fjord; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let mut land = Land { mat: vec![Mat::Air; n], col: vec![[0.0; 3]; n], rec: vec![0.0; n], rim: vec![0.0; n] };
    let mut near_top = [0.0; W];
    let mut far_top = [0.0; W];
    let mut peak_x = [0.0; W];
    for px in 0..W {
        let x = col_x(px) + 0.5;
        let (hl, pl) = range(x, &LEFT, 3.1, 4.0);
        let (hr, pr) = range(x, &RIGHT, 7.7, 4.0);
        let (hd, _) = range(x, &DISTANT, 11.3, 2.2);
        near_top[px] = WL - hl.max(hr).max(0.0);
        peak_x[px] = if hl > hr { pl } else { pr };
        far_top[px] = WL - hd.max(0.0);
    }
    let cab_base = WL - 2.2;
    let rows = land_rows();

    for py in 0..rows {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            if y >= near_top[px] {
                land.mat[k] = Mat::Near;
                let pk = peak_x[px];
                let side = x + 0.5 - pk;
                let depth = y - near_top[px];
                let height = WL - near_top[px];
                // Faces turned toward the fjord catch the aurora; the ridge
                // between the faces wanders as it comes down from the peak.
                let ridge = pk + (depth + 1.0) * 0.6 * (noise(y * 0.1, pk) - 0.5);
                let inward = if pk < 100.0 { 1.0 } else { -1.0 };
                let face = smooth(-4.0, 4.0, (x + 0.5 - ridge) * inward);
                // ribs and couloirs running down the fall line, each with a lit side
                let u = x + depth * 0.45 * if side < 0.0 { -1.0 } else { 1.0 };
                let rib = |v: f64| fbm(v * 0.13, pk * 0.37, 3);
                let grad = (rib(u + 1.0) - rib(u - 1.0)) * 7.0 * inward;
                // the snow reaches further down the ribs than the couloirs, in fingers
                let reach = 3.0 + height * (0.62 + 0.42 * rib(u + 40.0));
                let streak = smooth(0.56, 0.64, fbm(u * 0.32, y * 0.035 + pk, 3)) * smooth(1.0, 5.0, depth);
                // a stand of spruce climbing the slope behind the cabin, jagged on top
                let cx = x + 0.5 - 152.0;
                let xi = cell(x);
                let jag = if (xi as i64) & 1 == 1 { 1.0 } else { 0.3 };
                let wood = WL - 18.5 + 9.0 * (cx / 19.0).powi(2) + 1.5 * (noise(xi * 0.7, 9.0) - 0.5) - 2.6 * hash(xi, 77.0) * jag;
                let snow = smooth(reach + 1.6, reach - 1.6, depth) * (1.0 - smooth(WL - 3.0, WL - 0.5, y)) * (1.0 - 0.5 * streak);
                let lit = clamp(0.35 + 0.55 * face + grad * 0.5);
                // blue-grey rock below the snow, a little lighter on the lit faces
                let rv = 0.75 + 0.5 * fbm(x * 0.4, y * 0.4, 2);
                let rock = [(0.05 + 0.03 * lit) * rv, (0.065 + 0.035 * lit) * rv, (0.11 + 0.05 * lit) * rv];
                let s = 0.34 + 0.66 * lit.powf(1.5);
                land.col[k] = [mix(rock[0], 0.62 * s + 0.04, snow), mix(rock[1], 0.7 * s + 0.05, snow), mix(rock[2], 0.86 * s + 0.09, snow)];
                land.rec[k] = (snow * (0.25 + 0.75 * lit) + 0.06) as f32;
                land.rim[k] = (smooth(2.2, 0.3, depth) * 0.5) as f32;
                let ri = y.floor();
                if y > wood && cx > -14.0 - 2.0 * hash(ri, 3.0) && cx < 13.0 + 2.0 * hash(ri, 4.0) {
                    land.mat[k] = Mat::Tree;
                    let h = hash((px * 13 + py) as f64, 5.0);
                    land.col[k] = [0.02 + 0.03 * h, 0.045 + 0.045 * h, 0.06 + 0.04 * h];
                    land.rec[k] = 0.05;
                    land.rim[k] = (smooth(wood + 1.6, wood + 0.2, y) * (0.7 + 0.3 * h)) as f32;
                }
            } else if y >= far_top[px] {
                land.mat[k] = Mat::Far;
                let depth = y - far_top[px];
                let snow = smooth(0.5, 0.62, fbm(x * 0.18, y * 0.25, 3) * 0.6 + (1.0 - depth / 7.0) * 0.55);
                let lit = 0.5 + 0.3 * (noise(x * 0.25, y * 0.1) - 0.5);
                land.col[k] = [mix(0.075, 0.22 * lit + 0.12, snow), mix(0.1, 0.27 * lit + 0.15, snow), mix(0.17, 0.36 * lit + 0.24, snow)];
                land.rec[k] = (0.2 + 0.3 * snow) as f32;
            }
            // the shelf the cabin stands on, snowed over
            if x > 132.0 && x < 180.0 && y >= shore_top(x) {
                land.mat[k] = Mat::Shore;
                let f = 0.6 + 0.3 * fbm(x * 0.3, y * 0.5, 2);
                land.col[k] = [0.26 * f, 0.3 * f, 0.42 * f];
                land.rec[k] = 0.4;
                land.rim[k] = 0.0;
            }
        }
    }

    // a spruce treeline along the foot of both ranges, open where the fjord runs in
    let solid = |m: Mat| matches!(m, Mat::Wall | Mat::Roof | Mat::Pane | Mat::Door | Mat::Shore);
    let mut x = -1.0;
    while x < TW + 2.0 {
        let h = hash((x * 7.0f64).floor(), 21.0);
        let open = smooth(96.0, 80.0, x) + smooth(122.0, 136.0, x);
        let on_shelf = x > 132.0 && x < 180.0;
        if open > 0.05 && !(x > 136.0 && x < 166.0) {
            let tall = if hash((x * 5.0f64).floor(), 8.0) > 0.7 { 2.4 } else { 0.0 };
            let th = (3.2 + hash((x * 3.0f64).floor(), 5.0) * 3.0 + tall) * open.min(1.0) * if on_shelf { 0.7 } else { 1.0 };
            if th > 1.5 {
                let (tx, tw) = (x + hash(x.floor(), 2.0), 1.0 + hash(x.floor(), 4.0) * 0.6);
                if on_shelf {
                    land.spruce(tx, th, tw, shore_top(x) - 0.4, solid);
                } else {
                    land.spruce(tx, th, tw, WL + 0.3, |_| false);
                }
            }
        }
        x += 2.0 + 2.2 * h;
    }

    // The cabin: red boards, a snowed roof, two warm panes, a door, a chimney.
    let eave = cab_base - 7.0;
    let roof_top = eave - 8.0;
    let (chim, chim_top) = (CAB.1 - 5.0, eave - 8.5);
    let ccx = (CAB.0 + CAB.1 + 1.0) / 2.0;
    for py in 0..rows {
        let y = row_y(py);
        for px in 0..W {
            let x = col_x(px);
            let xi = cell(x);
            if xi < CAB.0 - 3.0 || xi > CAB.1 + 3.0 {
                continue;
            }
            let k = py * W + px;
            if xi >= CAB.0 && xi <= CAB.1 && y >= eave && y < cab_base + 0.5 {
                land.mat[k] = Mat::Wall;
                let boards = if (py / 2) & 1 == 1 { 0.82 } else { 1.0 };
                let s = (0.5 + 0.5 * ((CAB.1 - x) / (CAB.1 - CAB.0))) * boards;
                land.col[k] = [0.7 * s, 0.14 * s, 0.11 * s];
                land.rim[k] = 0.0;
                for (a, b) in PANES {
                    if xi >= a && xi <= b && y >= eave + 1.5 && y < cab_base - 2.0 {
                        land.mat[k] = Mat::Pane;
                    }
                }
                if xi >= DOOR_X.0 && xi <= DOOR_X.1 && y >= eave + 1.5 {
                    land.mat[k] = Mat::Door;
                    land.col[k] = [0.16, 0.06, 0.05];
                }
            }
            let half = 12.5 - (eave - y) * 1.45;
            if y >= roof_top && y < eave && (x + 0.5 - ccx).abs() <= half {
                land.mat[k] = Mat::Roof;
                let snowy = y < eave - 1.0;
                let s = 0.78 + 0.22 * ((ccx - x - 0.5) / 12.0);
                land.col[k] = if snowy { [0.78 * s, 0.84 * s, 0.96 * s] } else { [0.12, 0.05, 0.06] };
                land.rec[k] = if snowy { 0.5 } else { 0.0 };
                land.rim[k] = 0.0;
            }
            if xi >= chim && xi <= chim + 1.0 && y >= chim_top && y < eave - 4.0 {
                land.mat[k] = Mat::Wall;
                let s = if xi == chim { 1.0 } else { 0.55 };
                land.col[k] = [0.2 * s, 0.21 * s, 0.27 * s];
                land.rim[k] = 0.0;
            }
        }
    }

    // a few spruce on the shelf, beside the cabin
    for (tx, th) in [(134.0, 7.0), (137.8, 10.0), (166.5, 9.0), (170.0, 6.0), (174.5, 8.0)] {
        land.spruce(tx, th, 2.2, shore_top(tx) + 0.5, |m| matches!(m, Mat::Wall | Mat::Roof | Mat::Pane | Mat::Door));
    }

    // the warm light the panes throw on the snow and the air around them
    let mut lamp_land = vec![0.0f32; rows * W];
    for py in 0..rows {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let m = land.mat[k];
            if matches!(m, Mat::Pane | Mat::Wall | Mat::Roof | Mat::Door) {
                continue;
            }
            let mut g = 0.0;
            for (lx, s) in LAMPS {
                let (wx, wy) = (col_x(px) + 0.5 - lx, y - (cab_base - 2.5));
                let d = (wx * wx * 0.6 + wy * wy * 2.2).sqrt();
                g += s * (-d / 5.5).exp() * 0.6;
            }
            let k_m = match m {
                Mat::Air => 0.3,
                Mat::Tree => 0.4,
                _ => 0.75 + 0.5 * hash(px as f64, py as f64 + 31.0),
            };
            lamp_land[k] = (g * k_m) as f32;
        }
    }

    // the sky's unevenness and its stars, a pixel each: a pixel is 1 / 2.56
    // of a cell, so each is that much less likely
    let haze = (0..rows * W).map(|k| fbm(col_x(k % W) * 0.04, (row_y(k / W) - 0.5) * 0.07, 3) as f32).collect();
    let odds = 0.014 / (S * S);
    let star = (0..rows * W)
        .map(|k| {
            let h = hash((k % W) as f64, (k / W) as f64 + 101.0);
            if h > 1.0 - odds {
                (0.35 + (h - 1.0 + odds) / odds * 0.63) as f32
            } else {
                0.0
            }
        })
        .collect();

    Box::new(AuroraFjord {
        land,
        lamp_land,
        haze,
        star,
        eave,
        base_a: vec![0.0; W],
        tall_a: vec![0.0; W],
        env_a: vec![0.0; W],
        base_b: vec![0.0; W],
        env_b: vec![0.0; W],
        light_x: vec![0.0; W],
        rays_a: vec![0.0; RAYS + 2],
        rays_b: vec![0.0; RAYS + 2],
        above: vec![[0.0; 3]; rows * W],
        pixels: Pixels::new(&META),
    })
}

impl AuroraFjord {
    /// The two curtains' hems, heights, strengths and rays at t, and the
    /// light they throw on the land.
    fn curtains(&mut self, t: f64) {
        for px in 0..W {
            let x = col_x(px);
            let u = x / TW;
            // the main curtain sweeps down from the upper left, low over the
            // fjord, and lifts again to the right; ripples travel along it and fold it
            self.base_a[px] = 5.0 + 40.0 * ((u / 0.6).clamp(0.0, 1.0) * FRAC_PI_2).sin().powf(1.5) - 13.0 * smooth(0.6, 0.95, u)
                + 2.6 * (x * 0.07 - t * 0.55).sin()
                + 1.4 * (x * 0.17 + t * 0.9 + 1.3).sin()
                + 2.2 * (x * 0.22 + t * 1.2).sin()
                + 4.0 * (fbm(x * 0.015 + t * 0.03, 4.2, 2) - 0.5);
            self.tall_a[px] = 11.0 + 8.0 * fbm(x * 0.03 - t * 0.05, 1.7, 2);
            self.env_a[px] = smooth(0.0, 0.2, u) * smooth(0.98, 0.72, u) * (0.4 + 0.8 * fbm(x * 0.022 - t * 0.07, 8.8, 3));
            // a fainter curtain behind, higher up, on the right
            self.base_b[px] = 14.0 + 4.0 * (x * 0.035 + t * 0.3 + 2.0).sin() + 1.6 * (x * 0.11 - t * 0.7).sin();
            self.env_b[px] = smooth(0.45, 0.7, u) * smooth(1.05, 0.85, u) * (0.25 + 0.5 * fbm(x * 0.03 + t * 0.05, 3.3, 2));
        }
        for i in 0..=RAYS + 1 {
            let u = i as f64 / 4.0;
            self.rays_a[i] = 0.14 + fbm(u * 0.6 + t * 0.35, t * 0.12, 3).powf(2.4) * 2.5;
            self.rays_b[i] = 0.1 + fbm(u * 0.45 - t * 0.2, 5.0 + t * 0.1, 3).powf(2.2) * 2.0;
        }
        for px in 0..W {
            let mut s = 0.0;
            for d in (-24..=24).step_by(6) {
                let xx = (px as f64 + f64::from(d) * S).round().clamp(0.0, W as f64 - 1.0) as usize;
                s += self.env_a[xx] + 0.4 * self.env_b[xx];
            }
            self.light_x[px] = s / 9.0;
        }
    }

    /// The sky and the land above the waterline at t, into `above`.
    fn shade_above(&mut self, t: f64, flick: f64) {
        let land = &self.land;
        for py in 0..land_rows() {
            let y = row_y(py);
            let v = y / WL;
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let m = land.mat[k];
                let (mut cr, mut cg, mut cb);
                if m == Mat::Air {
                    let hz = 0.85 + 0.3 * f64::from(self.haze[k]);
                    cr = (0.025 + 0.03 * v * v) * hz;
                    cg = (0.04 + 0.07 * v * v) * hz;
                    cb = (0.1 + 0.11 * v * v) * hz;
                    // curtain A: a sharp lower hem, rays rising and fading to violet
                    let mut a = 0.0;
                    let env_a = self.env_a[px];
                    let d = self.base_a[px] - y;
                    if d > -4.0 && d < 46.0 {
                        // its fold: how steeply the hem runs here, over two cells
                        let bend = (self.base_a[(px + 2).min(W - 1)] - self.base_a[px.saturating_sub(2)]).abs() * 2.0 / (4.0 / S);
                        let hc = self.tall_a[px];
                        let lean = ray(&self.rays_a, x + d * 0.22);
                        let prof = if d < 0.0 { (-d * d * 0.9).exp() } else { (1.0 - (-(d + 0.4) * 1.1).exp()) * (-d / hc).exp() };
                        // the rays, over a continuous bright band along the hem
                        let band = if d < 0.0 { (-d * d).exp() } else { (-d / 3.0).exp() };
                        a = (prof * lean + 0.3 * band * (0.55 + 0.45 * lean.min(1.4))) * env_a * (1.15 + 0.35 * bend);
                        let up = clamp(d / (hc * 1.6));
                        let (gk, vk) = (1.0 - smooth(0.0, 0.5, up), smooth(0.4, 0.9, up));
                        let tk = 1.0 - gk - vk;
                        cr += a * (0.18 * gk + 0.06 * tk + 0.42 * vk);
                        cg += a * (gk + 0.78 * tk + 0.16 * vk);
                        cb += a * (0.48 * gk + 0.7 * tk + 0.75 * vk);
                        // pink at the very hem where it is brightest
                        let hem = (-(d + 0.6).powi(2) * 1.2).exp() * smooth(0.3, 0.8, a) * 0.8 * (0.45 + 0.55 * lean.min(1.0));
                        cr += hem * 0.95;
                        cg *= 1.0 - 0.55 * (hem * 1.6).min(1.0);
                        cb += hem * 0.4;
                    }
                    // curtain B, further away, thinning out toward the top of the frame
                    let db = self.base_b[px] - y;
                    if db > -3.0 && db < 30.0 {
                        let prof = if db < 0.0 { (-db * db).exp() } else { (1.0 - (-(db + 0.4)).exp()) * (-db / 9.0).exp() };
                        let b = prof * ray(&self.rays_b, x + db * 0.18) * self.env_b[px] * 0.85 * smooth(0.0, 8.0, y);
                        let up = clamp(db / 12.0);
                        cr += b * (0.1 + 0.4 * up);
                        cg += b * (0.75 - 0.5 * up);
                        cb += b * (0.7 + 0.2 * up);
                        a += b;
                    }
                    // a little glow round the curtain, kept tight under the hem
                    // so the hem reads as an edge over dark sky
                    let below = y - self.base_a[px];
                    let gl = env_a * if below > 0.0 { (-below / 6.0).exp() * 0.1 } else { (below / 8.0).exp() * 0.18 };
                    cr += gl * 0.12;
                    cg += gl * 0.62;
                    cb += gl * 0.52;
                    // airglow on the horizon
                    let ag = (-(WL - y) / 10.0).exp() * (0.1 + 0.16 * self.light_x[px]);
                    cr += ag * 0.2;
                    cg += ag * 0.7;
                    cb += ag * 0.75;
                    let st = f64::from(self.star[k]);
                    if st > 0.0 {
                        let (fx, fy) = (px as f64, py as f64);
                        let tw = 0.7 + 0.3 * (t * (1.3 + 3.0 * hash(fx, fy)) + TAU * hash(fy, fx)).sin();
                        let s = st * tw * clamp(1.0 - a * 1.6) * smooth(WL - 2.0, 30.0, y);
                        (cr, cg, cb) = (cr.max(s * 0.9), cg.max(s * 0.94), cb.max(s));
                    }
                } else {
                    [cr, cg, cb] = land.col[k];
                    let lx = self.light_x[px];
                    let l = lx * f64::from(land.rec[k]);
                    cr += l * 0.03;
                    cg += l * 0.14;
                    cb += l * 0.1;
                    let e = f64::from(land.rim[k]);
                    if e > 0.0 {
                        let q = e * if m == Mat::Tree { 0.2 + 0.45 * lx } else { 0.05 + 0.22 * lx };
                        cr += q * 0.25;
                        cg += q * 0.95;
                        cb += q * 0.7;
                    }
                    if m == Mat::Pane {
                        let f = flick + 0.04 * (t * 5.3 + cell(x)).sin();
                        (cr, cg, cb) = (f, 0.76 * f, 0.38 * f);
                    } else if m == Mat::Door && cell(x) == DOOR_X.1 && y.floor() > self.eave + 1.0 {
                        // light through the crack of the door
                        (cr, cg, cb) = (0.55 * flick, 0.36 * flick, 0.14 * flick);
                    }
                }
                let lg = f64::from(self.lamp_land[k]) * flick;
                if lg > 0.0 {
                    cr += lg;
                    cg += lg * 0.62;
                    cb += lg * 0.26;
                }
                self.above[k] = [cr, cg, cb];
            }
        }
    }
}

impl Piece for AuroraFjord {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        self.curtains(t);
        let flick = 0.92 + 0.05 * (t * 2.3).sin() + 0.03 * (t * 7.1).sin();
        self.shade_above(t, flick);
        let rows = land_rows();
        let ground = [0.02, 0.031, 0.059];
        let to = |c: [f64; 3]| [c[0] as f32, c[1] as f32, c[2] as f32];
        for py in 0..rows {
            for px in 0..W {
                self.pixels.set(px, py, to(self.above[py * W + px]));
            }
        }
        let (above, mat) = (&self.above, &self.land.mat);
        for py in rows..H {
            let y = row_y(py);
            let dw = y - WL;
            let r = y - 0.5; // ascii.rest's row
            let deep = mix(1.0, 0.45, dw / (TH - WL));
            // the rows of the land above that this one mirrors, and the two
            // behind it that the reflection stretches over
            let ry = 2.0 * WL * S - 1.0 - py as f64;
            let at = |dy: f64| ((ry - dy).round().max(0.0) as usize).min(rows - 1) * W;
            let (r0, r1, r2) = (at(0.0), at(S), at(2.0 * S));
            let gap = hash(y.floor(), 404.0) < 0.3; // rows of the water that break the reflection into strips
            let fade = smooth(TH + 3.0, TH - 12.0, y);
            for px in 0..W {
                let x = col_x(px);
                // still water: the mirror image, stretched and broken by slow ripples
                let wave = noise(x * 0.04 + t * 0.05, r * 0.55 - t * 0.35);
                let sx = px as f64 + S * (0.5 + dw * 0.12) * (r * 1.3 + t * 1.6 + wave * 4.0).sin();
                let ix = sx.floor();
                let fx = sx - ix;
                let ix = (ix.max(0.0) as usize).min(W - 2);
                let (a0, a1, a2) = (r0 + ix, r1 + ix, r2 + ix);
                let ms = mat[a0];
                let sky = ms == Mat::Air;
                let mut kr = (if sky { 0.55 } else { 0.64 }) * (0.82 + 0.3 * wave) * deep;
                if sky && gap && wave < 0.45 {
                    kr *= 0.12;
                }
                // the cabin's own image is soft; the lamplight road below carries it
                if ms == Mat::Pane {
                    kr *= 0.4;
                } else if matches!(ms, Mat::Wall | Mat::Roof | Mat::Door) {
                    kr *= 0.7;
                }
                let gx = 1.0 - fx;
                let mut c = [0.0; 3];
                for (i, ch) in c.iter_mut().enumerate() {
                    let lerp = |a: usize| above[a][i] * gx + above[a + 1][i] * fx;
                    *ch = (lerp(a0) * 0.5 + lerp(a1) * 0.3 + lerp(a2) * 0.2) * kr;
                }
                if ms != Mat::Wall && ms != Mat::Door {
                    c[0] += 0.02;
                    c[1] += 0.035;
                    c[2] += 0.065;
                }
                // a pale line where the water meets the shore
                if y.floor() == WL {
                    let e = 0.3 * (0.3 + 0.7 * smooth(0.25, 0.75, noise(x * 0.3, t * 0.4)));
                    c[0] += e * 0.6;
                    c[1] += e * 0.85;
                    c[2] += e;
                }
                // the lamplight laid on the water as a broken golden road
                if dw < 14.0 && x > 138.0 && x < 166.0 {
                    let rip = noise(x * 0.5 - t * 0.2, r * 1.4 - t * 1.5);
                    for (lx, s) in LAMPS {
                        let q = (x + 0.5 - lx) / (1.0 + dw * 0.1);
                        let g = (-q * q).exp() * (-dw / 8.0).exp() * smooth(0.3, 0.65, rip) * s * 1.6 * flick;
                        c[0] += g;
                        c[1] += g * 0.68;
                        c[2] += g * 0.28;
                    }
                }
                // the bottom rows sink into the ground
                self.pixels.set(px, py, [mix(ground[0], c[0], fade) as f32, mix(ground[1], c[1], fade) as f32, mix(ground[2], c[2], fade) as f32]);
            }
        }
        self.pixels.draw(grid, env);
    }
}
