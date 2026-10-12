//! misty forest: morning in a pine forest. Ridge after ridge of pines recedes
//! into fog, each paler than the one in front, with mist lying in sheets in
//! the valleys between them. A low sun sits behind the farthest trees and
//! sends beams slanting down through the fog to a clearing on the forest
//! floor. The fog drifts, the beams shimmer, and motes of dust float in the
//! light. After ascii.rest's misty-forest.ts (MIT, @bas3line).
//!
//! The ridges and their pines, the giants in front, the fog banks drifting at
//! each ridge's own speed, the high cloud, the beams and the motes, and how
//! each is shaded, are ascii.rest's, shaded here per pixel instead of per
//! cell. Where ascii.rest draws each of 200 × 100 square cells as a dot as big
//! as it is bright, this draws 320 × 80 cells of two square pixels each, an
//! upper half block over a back colour ([`crate::pixels`]): 1.6 times the
//! detail each way, solid colour, and gradients dithered in colour rather
//! than in dot size. The palette adds the near-blacks of the pines and the
//! deep teals of the sky and the fog a solid pixel needs where a small dot
//! was enough. A mote of dust is a pixel. In one ink each pixel is octant
//! dots, as many as it is bright, so the beams are dithered in octants.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::{Pixels, Rgb};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Misty forest's meta.
pub static META: Meta = Meta {
    name: "misty forest",
    slug: "misty-forest",
    category: Category::Wilds,
    note: "pine ridges fading into morning fog, sunbeams slanting through",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: sunlight and the warm fog round it, the cool fog, the
    // pines from the haze to the dark in front of us, moss and bark, and the
    // clear sky. Then the near-blacks of the pines and the floor, and the
    // deeper teals of the sky and the shadowed fog.
    palette: Some(&[
        "#fffbea", "#fff0c8", "#fbe2a6", "#f2c97e", "#e0a95e", "#b9834a", "#e4ebe6", "#cbe0dc", "#a8d2d0", "#86c0c2", "#68a7ac", "#4f8c93", "#3b7078",
        "#557570", "#456761", "#365952", "#2a4a44", "#1f3c37", "#172f2b", "#11231f", "#a4a35a", "#8a8f4c", "#6d7440", "#4d5530", "#7a5a3a", "#5b4532",
        "#a9c3cc", "#8eadb8", "#6f93a2", "#53798a", "#3d6172", "#2b4a5a", "#090f0e", "#030606", "#0c1816", "#0d1311", "#1a2a24", "#1c3a3e", "#152a30",
        "#203f48", "#2e5258", "#3e6a6c", "#5e8682", "#7a9c96", "#94b0aa", "#c8b88a",
    ]),
    ground: Some("#090f0e"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// The scene's width and height in ascii.rest's cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
const SUN: (f64, f64) = (146.0, 45.5);
const SUN_R: f64 = 3.4;
/// What a pixel shows besides the ridges 0 to [`NEAR`].
const SKY: i8 = -1;
const FLOOR: i8 = 5;
const GIANT: i8 = 6;

/// The ridges, far to near: [ridge row, its rise and fall, tree spacing, tree
/// heights from and to, haze, how fast its fog drifts, mist lying in the
/// valley below it, drifting fog in front of it, sunbeam in front of it].
const LAYERS: [[f64; 10]; 5] = [
    [50.0, 6.0, 1.6, 1.0, 2.5, 0.48, 0.8, 1.0, 0.14, 0.6],
    [59.0, 6.0, 2.0, 3.0, 5.5, 0.3, 1.3, 0.8, 0.16, 0.8],
    [69.0, 7.0, 2.6, 4.0, 7.0, 0.13, 2.0, 0.66, 0.18, 0.9],
    [80.0, 6.0, 3.4, 5.0, 9.0, 0.03, 2.8, 0.44, 0.14, 1.0],
    [90.0, 1.5, 10.0, 9.0, 24.0, 0.03, 3.4, 0.0, 0.08, 0.6],
];
const NEAR: usize = LAYERS.len() - 1;
/// The giants in front: [x, tip row, how far the boughs spread, tier height].
const GIANTS: [[f64; 4]; 2] = [[9.0, -12.0, 15.0, 7.0], [192.0, 12.0, 7.0, 5.0]];
/// Angle bins round the sun, for the beams.
const RA: usize = 720;
/// The fog banks' and the high cloud's width in columns: they wrap, so they can drift forever.
const FW: usize = 400;
/// The pixel rows the high cloud can reach: those above row 46.
const CH: usize = 74;
/// Sunbeams: wide shafts through the gaps, (angle, half width, strength).
const SHAFTS: [(f64, f64, f64); 7] =
    [(1.42, 0.05, 0.7), (1.66, 0.07, 1.0), (1.93, 0.05, 0.8), (2.18, 0.08, 1.0), (2.45, 0.05, 0.75), (2.7, 0.06, 0.9), (2.95, 0.04, 0.6)];

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

/// The fog's colour: cool and green-grey, warming to cream by the sun (`s`: nearness to it).
fn fog_rgb(s: f64) -> (f64, f64, f64) {
    (mix(0.7, 1.05, s), 0.92, mix(0.9, 0.6, s))
}

/// How much a pixel sits on a silhouette's sunward rim: `weights[0]` where
/// the pixel above or the next one sunward is open, `weights[n]` where the
/// first open one is n + 1 pixels sunward, else 0.
fn rim(open: impl Fn(i64, i64) -> bool, px: usize, py: usize, toward: i64, weights: &[f32]) -> f32 {
    let (x, y) = (px as i64, py as i64);
    if open(x, y - 1) {
        return weights[0];
    }
    (1..=weights.len() as i64).find(|&n| open(x + toward * n, y)).map_or(0.0, |n| weights[n as usize - 1])
}

/// One dust mote: (x, y, drift speed, bob phase, size).
type Mote = (f64, f64, f64, f64, f64);

/// The forest: each pixel's colour before the fog, the beams and the motes,
/// and how much of each it takes; the fog banks, the cloud and the beams'
/// angular pattern; the motes; and the picture.
pub struct MistyForest {
    base: Vec<Rgb>,
    sun_s: Vec<f32>,
    fog_amt: Vec<f32>,
    fog_speed: Vec<f32>,
    ray_amt: Vec<f32>,
    floor_lit: Vec<f32>,
    cloud_amt: Vec<f32>,
    abin: Vec<f32>,
    dist: Vec<f32>,
    fog: Vec<f32>,
    cloud: Vec<f32>,
    ray_a: Vec<f32>,
    ray_b: Vec<f32>,
    motes: Vec<Mote>,
    mote: Vec<f32>,
    mote_cells: Vec<usize>,
    pixels: Pixels,
}

/// A misty forest; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    // --- the layers, rasterised far to near so nearer ones cover farther ---
    let mut layer = vec![SKY; n];
    let mut edge = vec![0.0f32; n]; // how much a pixel sits on a silhouette's sunward rim
    let mut below = vec![0.0f32; n]; // rows below the layer's tree line
    let mut tex = vec![0.0f32; n]; // the lower edge of a tier of boughs, near trees only
    let mut ground = vec![0.0f32; n]; // the row a pixel's ridge rises from, for its valley mist
    for (i, l) in LAYERS.iter().enumerate() {
        let (y0, amp, gap, h0, h1) = (l[0], l[1], l[2], l[3], l[4]);
        let fi = i as f64;
        // nearer ridges dip toward the sun, a valley opening onto the light
        let ridge = |x: f64| {
            let dip = if i > 0 && i < NEAR { (2.0 + fi * 2.0) * (-((x - SUN.0 - 4.0) / 38.0).powi(2)).exp() } else { 0.0 };
            y0 + amp * (fbm(x * (0.018 + fi * 0.003), fi * 13.0 + 2.0, 3) - 0.5) * if i < 3 { 4.0 } else { 3.0 } + dip
        };
        let base: Vec<f64> = (0..W).map(|px| ridge(col_x(px))).collect();
        let mut fill = vec![false; n];
        let mut spire = vec![false; n];
        for py in 0..H {
            for px in 0..W {
                fill[py * W + px] = row_y(py) >= base[px];
            }
        }
        // pines: a spire of tiers, each tier flaring out and stepping back in
        let mut tx = -2.0 + hash(fi, 1.0) * gap;
        while tx < TW + 2.0 {
            // the nearest trees leave a clearing under the sun for the light to land in
            if !(i == NEAR && tx > 92.0 && tx < 140.0) {
                let th = h0 + (h1 - h0) * hash(tx * 7.0, fi + 5.0);
                let foot = ridge(tx);
                let tip = foot - th;
                let tier = 2.0 + th * 0.12;
                for py in to_px(tip, H)..=to_px(foot + 2.5, H) {
                    let y = row_y(py);
                    let d = y - tip;
                    if d < 0.0 || y >= foot + 2.5 {
                        continue;
                    }
                    let saw = (d % tier) / tier;
                    let half = d * 0.3 * (0.6 + 0.5 * saw) + 0.35;
                    for px in to_px(tx - half, W)..=to_px(tx + half, W) {
                        let dx = (col_x(px) + 0.5 - tx).abs();
                        if dx > half {
                            continue;
                        }
                        let k = py * W + px;
                        fill[k] = true;
                        spire[k] = true;
                        if i == NEAR {
                            tex[k] = (smooth(0.62, 0.95, saw) * smooth(0.3, 0.85, dx / half) * (0.55 + 0.45 * hash(px as f64, py as f64 * 5.0))) as f32;
                        }
                    }
                }
            }
            tx += gap * (0.7 + hash(tx, fi + 3.0) * 0.7);
        }
        // where the silhouette starts in each column, smoothed a little so the
        // mist line follows the forest rather than every single spire
        let top: Vec<f64> = (0..W).map(|px| (0..H).find(|&py| fill[py * W + px]).map_or(TH, |py| py as f64 / S)).collect();
        let line: Vec<f64> = (0..W)
            .map(|px| {
                let s: f64 = (-5i64..=5).map(|d| top[(px as i64 + d).clamp(0, W as i64 - 1) as usize]).sum();
                (s / 11.0).max(top[px])
            })
            .collect();
        let open = |x: i64, y: i64| x >= 0 && x < W as i64 && (y < 0 || !fill[y as usize * W + x as usize]);
        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                if !fill[k] {
                    continue;
                }
                layer[k] = if i == NEAR && !spire[k] { FLOOR } else { i as i8 };
                if i != NEAR {
                    tex[k] = 0.0;
                }
                below[k] = (row_y(py) - line[px]) as f32;
                ground[k] = base[px] as f32;
                // a rim where the sky (or a farther layer) shows beside or above
                let toward = if col_x(px) < SUN.0 { 1 } else { -1 };
                edge[k] = rim(open, px, py, toward, &[1.0, 0.75, 0.35]);
            }
        }
    }

    // --- what stands in front: a giant pine cut by the left of the frame,
    // and a smaller one at the right edge, so the frame is not a symmetric
    // curtain ---
    let mut giant = vec![false; n];
    for &[gx, tip, spread, tier] in &GIANTS {
        for py in to_px(tip, H)..H {
            let y = row_y(py);
            let d = y - tip;
            if d < 0.0 {
                continue;
            }
            let saw = (d % tier) / tier;
            // each tier of boughs sweeps out and droops, so the outline is a
            // stack of points rather than a straight edge where it meets the frame
            let half = (spread * (0.5 + 0.5 * saw)).min(d * 0.34 * (0.45 + 0.7 * saw) + 0.5);
            for px in to_px((gx - half).floor(), W)..to_px((gx + half).ceil() + 1.0, W) + 1 {
                let x = col_x(px);
                let dx = (x + 0.5 - gx).abs();
                // the side toward the frame stays full, so no sliver of sky shows there
                let outer = if (x + 0.5 - gx) * (gx - TW / 2.0) > 0.0 { 1.6 } else { 1.0 };
                let ragged = half * outer * (0.82 + 0.3 * noise(x * 0.5, (y - 0.5) * 0.4));
                if dx <= ragged || dx < 0.9 {
                    let k = py * W + px;
                    giant[k] = true;
                    tex[k] = (smooth(0.66, 0.96, saw) * smooth(0.25, 0.8, dx / ragged) * (0.5 + 0.5 * hash(px as f64 * 3.0, py as f64))) as f32;
                }
            }
        }
    }
    // a rim two cells deep on the side that faces the sun
    let open = |x: i64, y: i64| x >= 0 && x < W as i64 && (y < 0 || !giant[y as usize * W + x as usize]);
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            if giant[k] {
                layer[k] = GIANT;
                below[k] = 0.0;
                let toward = if col_x(px) < SUN.0 { 1 } else { -1 };
                edge[k] = rim(open, px, py, toward, &[1.0, 0.8, 0.55, 0.3]);
            }
        }
    }

    // --- static colour ---
    let mut base = vec![[0.0f32; 3]; n];
    let mut sun_s = vec![0.0f32; n]; // nearness to the sun, for the fog's warmth
    let mut fog_amt = vec![0.0f32; n]; // how much drifting fog shows in front of a pixel
    let mut fog_speed = vec![0.0f32; n];
    let mut ray_amt = vec![0.0f32; n]; // how much of a sunbeam the air in front of it holds
    let mut floor_lit = vec![0.0f32; n]; // where a beam that reaches the floor lights it
    let mut cloud_amt = vec![0.0f32; n]; // where high cloud can show, in the sky only
    let mut abin = vec![0.0f32; n];
    let mut dist = vec![0.0f32; n];
    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let l = layer[k];
            let (dx, dy) = (x + 0.5 - SUN.0, y - SUN.1);
            let ang = dy.atan2(dx);
            abin[k] = ((ang / TAU * RA as f64 + RA as f64) % RA as f64) as f32;
            let d = dx.hypot(dy);
            dist[k] = d as f32;
            let s = (-(dx * dx + dy * dy * 1.96).sqrt() / 30.0).exp();
            sun_s[k] = s as f32;
            let (fr, fg, fb) = fog_rgb(s);
            // a soft warm bloom over a wide radius, in the air and the fog
            let bloom = (-d / 16.0).exp() * 0.25 + (-d / 6.0).exp() * 0.1;
            let (mut cr, mut cg, mut cb);
            let ray;
            if l == SKY {
                // sky: deep overhead, paling to the fog band low down, warm by the sun
                let p = clamp(y / 52.0).powf(1.6);
                let veil = 0.95 + 0.1 * fbm(x * 0.04, y * 0.08, 3);
                (cr, cg, cb) = (mix(0.065, 0.36, p) * veil, mix(0.14, 0.58, p) * veil, mix(0.17, 0.62, p) * veil);
                // the fog band the far ridge stands in
                let band = 0.75 * smooth(28.0, 50.0, y) * (0.55 + 0.75 * fbm(x * 0.025, y * 0.16, 3));
                let (f0r, f0g, f0b) = fog_rgb(0.0);
                (cr, cg, cb) = (mix(cr, f0r, band), mix(cg, f0g, band), mix(cb, f0b, band));
                // warm in hue round the sun, but falling off in brightness, so
                // the disc stands in a halo rather than a flat blaze
                let (kw, kb) = (s * 0.55, 0.5 + 0.5 * (-d / 9.0).exp());
                (cr, cg, cb) = (mix(cr, fr * kb, kw), mix(cg, fg * kb, kw), mix(cb, fb * kb, kw));
                let glow = (-d / 3.0).exp() * 0.3 + (-d / 10.0).exp() * 0.18;
                cr += glow + bloom;
                cg += glow * 0.9 + bloom * 0.78;
                cb += glow * 0.66 + bloom * 0.45;
                // the disc itself, a little brighter in the middle
                let disc = smooth(SUN_R + 0.6, SUN_R - 0.4, d);
                (cr, cg, cb) = (mix(cr, 1.3, disc), mix(cg, 1.24, disc), mix(cb, 1.08, disc));
                fog_amt[k] = (0.3 * smooth(30.0, 50.0, y)) as f32;
                fog_speed[k] = 0.4;
                cloud_amt[k] = (0.75 * smooth(5.0, 15.0, y) * smooth(44.0, 28.0, y) * smooth(SUN_R + 2.0, SUN_R + 8.0, d)) as f32;
                ray = 0.35 * smooth(26.0, 46.0, y);
            } else if (l as usize) < NEAR {
                let li = l as usize;
                let lf = f64::from(l);
                let [.., haze, speed, m, drift, beam] = LAYERS[li];
                let below = f64::from(below[k]);
                // pine: dark teal, paling with distance; mist pooled just under
                // the tree line, and a sheet of it lying along the valley floor below
                let pool = smooth(2.5, 8.0 + lf * 1.5, below) * smooth(20.0, 11.0, below);
                // the sheet follows the lie of the land, smoothly, so it never streaks
                let sheet = (-((y - (f64::from(ground[k]) + 5.0)) / 3.0).powi(2)).exp() * (0.75 + 0.5 * fbm(x * 0.035, lf * 7.3, 3));
                let mist = clamp((pool * 0.5).max(sheet) * m);
                let h = clamp(haze + (1.0 - haze) * mist);
                let veil = 0.9 + 0.2 * fbm(x * 0.06, y * 0.12, 3);
                (cr, cg, cb) = (mix(0.02, fr * veil, h), mix(0.075, fg * veil, h), mix(0.07, fb * veil, h));
                cr += bloom * h;
                cg += bloom * 0.78 * h;
                cb += bloom * 0.45 * h;
                let rim = f64::from(edge[k]) * s * (0.2 + 0.6 * (1.0 - haze));
                cr += rim * 0.95;
                cg += rim * 0.8;
                cb += rim * 0.45;
                fog_amt[k] = drift as f32;
                fog_speed[k] = speed as f32;
                // the trees stop most of the light; the beams show in the mist
                // between (far off there is more air in front of them to hold the light)
                ray = beam * if li < 3 { 0.45 + 0.55 * clamp(mist / m) } else { 0.25 + 0.75 * clamp(mist / m) };
            } else if l as usize == NEAR {
                // the nearest pines: near-black, a faint teal on each tier's lower edge
                let g = f64::from(tex[k]) * 0.12;
                (cr, cg, cb) = (0.001 + g * 0.35, 0.004 + g, 0.004 + g * 0.9);
                let rim = f64::from(edge[k]) * (0.12 + 0.6 * s);
                cr += rim * 0.9;
                cg += rim * 0.7;
                cb += rim * 0.38;
                fog_amt[k] = LAYERS[NEAR][8] as f32;
                fog_speed[k] = LAYERS[NEAR][6] as f32;
                ray = 0.12;
            } else if l == FLOOR {
                // the forest floor: moss in clumps, sparse, fading out toward the frame
                let clump = smooth(0.42, 0.72, fbm(x * 0.09, y * 0.4, 3));
                let speck = if hash(px as f64 * 7.0, py as f64 * 11.0) > 0.55 { 1.0 } else { 0.35 };
                let m = (0.06 + 0.3 * clump) * speck * smooth(TH + 2.0, TH - 8.0, y);
                (cr, cg, cb) = (0.02 + m * 0.6, 0.03 + m * 0.62, 0.02 + m * 0.3);
                fog_amt[k] = 0.06;
                fog_speed[k] = 3.4;
                ray = 0.0;
                // dappled patches the beams can land on
                floor_lit[k] = (smooth(0.32, 0.56, fbm(x * 0.06 + 3.0, y * 0.24, 3)) * smooth(TH + 3.0, TH - 6.0, y)) as f32;
            } else {
                // the giants: near-black needles, the tiers drawn by a faint
                // teal edge, a warm rim toward the light
                let g = f64::from(tex[k]) * 0.15;
                (cr, cg, cb) = (0.001 + g * 0.35, 0.004 + g, 0.004 + g * 0.9);
                // warm on the side near the sun, a cool fog-lit edge far from it
                let warm = (-d / 50.0).exp();
                let rim = f64::from(edge[k]) * (0.13 + 0.55 * warm);
                cr += rim * mix(0.4, 0.95, warm);
                cg += rim * mix(0.75, 0.66, warm);
                cb += rim * mix(0.72, 0.32, warm);
                fog_amt[k] = 0.04;
                fog_speed[k] = 4.0;
                ray = 0.04;
            }
            // the beams fan out mostly down and to the west, the way the gaps face
            ray_amt[k] = (ray * (0.2 + 0.8 * smooth(1.25, 1.75, ang) * smooth(3.1, 2.6, ang))) as f32;
            base[k] = [cr as f32, cg as f32, cb as f32];
        }
    }

    // drifting fog banks: wide soft noise that wraps so it can slide forever
    let mut fog = vec![0.0f32; FW * H];
    for py in 0..H {
        let r = row_y(py) - 0.5;
        for u in 0..FW {
            fog[py * FW + u] = smooth(0.42, 0.75, fbm_wrap(u as f64 * 0.022, r * 0.09, 4, FW as f64 * 0.022)) as f32;
        }
    }
    // thin high cloud, long and flat, lit from below by the low sun
    let mut cloud = vec![0.0f32; FW * CH];
    for py in 0..CH {
        let r = row_y(py) - 0.5;
        for u in 0..FW {
            let u = u as f64;
            let q = fbm_wrap(u * 0.01, r * 0.05, 2, FW as f64 * 0.01);
            cloud[py * FW + u as usize] = smooth(0.44, 0.68, fbm_wrap(u * 0.016 + q * 1.5, r * 0.17, 4, FW as f64 * 0.016)) as f32;
        }
    }
    // sunbeams: the shafts, with a faint grain along each, and a slower
    // pattern sliding across them so they brighten and fade
    let ray_a = (0..RA)
        .map(|i| {
            let a = i as f64 / RA as f64 * TAU;
            let v = SHAFTS.iter().fold(0.0f64, |v, &(c, w, st)| v.max(st * smooth(w, w * 0.35, (a - c).abs())));
            (v * (0.8 + 0.2 * fbm_wrap(i as f64 * 0.4, 3.1, 2, RA as f64 * 0.4))) as f32
        })
        .collect();
    let ray_b = (0..RA).map(|i| smooth(0.4, 0.7, fbm_wrap(i as f64 * 0.03, 8.7, 2, RA as f64 * 0.03)) as f32).collect();
    // dust in the air
    let motes = (0..110)
        .map(|i| {
            let i = f64::from(i);
            (hash(i, 1.0) * TW, 50.0 + hash(i, 2.0) * 44.0, 0.3 + hash(i, 3.0) * 0.8, hash(i, 4.0) * TAU, hash(i, 5.0))
        })
        .collect();

    Box::new(MistyForest {
        base,
        sun_s,
        fog_amt,
        fog_speed,
        ray_amt,
        floor_lit,
        cloud_amt,
        abin,
        dist,
        fog,
        cloud,
        ray_a,
        ray_b,
        motes,
        mote: vec![0.0; n],
        mote_cells: vec![],
        pixels: Pixels::new(&META),
    })
}

/// A wrapping field's value at column `u`, linearly between its samples.
fn wrapped(field: &[f32], row: usize, u: f64) -> f64 {
    let ui = u.floor();
    let uf = u - ui;
    let i0 = (ui as i64).rem_euclid(FW as i64) as usize;
    let (f0, f1) = (f64::from(field[row * FW + i0]), f64::from(field[row * FW + (i0 + 1) % FW]));
    f0 + (f1 - f0) * uf
}

impl Piece for MistyForest {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        for &k in &self.mote_cells {
            self.mote[k] = 0.0;
        }
        self.mote_cells.clear();
        for &(mx, my, sp, ph, sz) in &self.motes {
            let x = (mx + t * sp + 2.5 * (t * 0.4 + ph).sin()).rem_euclid(TW);
            let y = my + 2.0 * (t * 0.3 + ph * 1.7).sin() - (t * sp * 0.2) % 6.0;
            if !(0.0..TH).contains(&y) {
                continue;
            }
            let k = to_px(y, H) * W + to_px(x, W);
            self.mote[k] = (0.5 + 0.5 * sz) as f32;
            self.mote_cells.push(k);
        }
        // the beams hold their places (the gaps in the trees do not move) and
        // only sway a hair; a second, slower pattern drifts across them
        let (shift_a, shift_b) = (2.0 * (t * 0.35).sin(), -t * 1.6);
        let pulse = 0.88 + 0.12 * (t * 0.7).sin();
        let bin = |a: f64| (a.floor() as i64).rem_euclid(RA as i64) as usize;

        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let [r, g, b] = self.base[k];
                let (mut cr, mut cg, mut cb) = (f64::from(r), f64::from(g), f64::from(b));
                let s = f64::from(self.sun_s[k]);
                let dist = f64::from(self.dist[k]);

                // the fog banks drift, nearer ones faster
                let fa = f64::from(self.fog_amt[k]);
                if fa > 0.01 {
                    let a = wrapped(&self.fog, py, x + t * f64::from(self.fog_speed[k])) * fa;
                    let (fr, fg, fb) = fog_rgb(s);
                    (cr, cg, cb) = (mix(cr, fr, a), mix(cg, fg, a), mix(cb, fb, a));
                }

                let ca = f64::from(self.cloud_amt[k]);
                if ca > 0.01 && py < CH {
                    let a = wrapped(&self.cloud, py, x + t * 0.6) * ca;
                    if a > 0.005 {
                        // cool grey-teal, warming to gold on the undersides near the sun
                        let w = (-dist / 40.0).exp();
                        (cr, cg, cb) = (mix(cr, mix(0.26, 0.95, w), a), mix(cg, mix(0.38, 0.72, w), a), mix(cb, mix(0.42, 0.45, w), a));
                    }
                }

                // the beams: brightest near the sun, fading with distance
                let (ra, fl) = (f64::from(self.ray_amt[k]), f64::from(self.floor_lit[k]));
                let ai = f64::from(self.abin[k]);
                let mut beam = 0.0;
                if ra > 0.01 || fl > 0.01 {
                    beam = f64::from(self.ray_a[bin(ai + shift_a)]) * (0.6 + 0.4 * f64::from(self.ray_b[bin(ai + shift_b)])) * pulse;
                }
                if ra > 0.01 && dist > SUN_R {
                    // lit shafts brighten the air, the shadows between them dim it
                    let fall = (-dist / 80.0).exp() * smooth(SUN_R, 12.0, dist);
                    let b = (beam - 0.3) * fall * ra * 2.2;
                    if b > 0.0 {
                        cr += b * 1.05;
                        cg += b * 0.8;
                        cb += b * 0.42;
                    } else {
                        (cr, cg, cb) = (cr * (1.0 + b), cg * (1.0 + b), cb * (1.0 + b));
                    }
                }
                if fl > 0.01 {
                    // a patch of sun on the moss where a beam lands
                    let b = beam * fl * 1.1;
                    cr += b;
                    cg += b * 0.78;
                    cb += b * 0.4;
                }

                let m = f64::from(self.mote[k]);
                if m > 0.0 && dist > 6.0 {
                    // a mote shows up where a beam catches it
                    let lit = f64::from(self.ray_a[bin(ai + shift_a)]) * (-dist / 90.0).exp();
                    let v = m * (0.1 + 1.6 * lit) * (ra * 2.0).min(1.0);
                    if v > 0.18 {
                        (cr, cg, cb) = (cr.max(v * 1.05), cg.max(v * 0.97), cb.max(v * 0.7));
                    }
                }
                self.pixels.set(px, py, [cr as f32, cg as f32, cb as f32]);
            }
        }
        self.pixels.draw(grid, env);
    }
}
