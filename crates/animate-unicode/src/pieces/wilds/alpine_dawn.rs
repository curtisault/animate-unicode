//! alpine dawn: jagged snow peaks catch the first pink light on their east
//! faces while their flanks stay in blue shadow. Mist pools along the far
//! shore and a still lake mirrors it all. The light warms toward gold as the
//! sun clears the ridge, the mist drifts, and slow ripples cross the water.
//! After ascii.rest's alpine-dawn.ts (MIT, @bas3line).
//!
//! The range, the treeline, the pines, the mist, the high cloud and the lake,
//! and how each is shaded, are ascii.rest's: the range is a heightfield
//! raymarched once, and the lake looks up the picture above it along each
//! reflected ray. Here both are done per pixel instead of per cell. Where
//! ascii.rest draws each of 200 × 100 square cells as a dot as big as it is
//! bright, this draws 320 × 80 cells of two square pixels each, an upper half
//! block over a back colour ([`crate::pixels`]): 1.6 times the detail each
//! way, solid colour, and gradients dithered in colour rather than in dot
//! size. The palette adds the dark tones a solid pixel needs where a small dot
//! was enough. In one ink each pixel is octant dots, as many as it is bright.
//!
//! Added: the alpenglow starts on the summits and slides down the faces over
//! the first minute or so (ascii.rest's sunlit line starts most of the way
//! down), and every 24 seconds a breath of wind crosses the lake, breaking the
//! mirror up in ripples as it passes.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Alpine dawn's meta.
pub static META: Meta = Meta {
    name: "alpine dawn",
    slug: "alpine-dawn",
    category: Category::Wilds,
    note: "snow peaks catching first light above a still, misty mountain lake",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: indigo to pale blue, mauves and roses, the dawn's peaches
    // and golds, umbers, slates, the lake's teals, lilacs. Then the darks.
    palette: Some(&[
        "#0e1430", "#151d40", "#1d2752", "#263365", "#314179", "#3e508c", "#4f62a0", "#6577b3", "#8090c4", "#9eaad3", "#bec6e2", "#4b3e6c", "#6a5482",
        "#8c6a92", "#b0829c", "#cf96a4", "#e8a9a8", "#f5bcaa", "#ffd0b0", "#ffe2c2", "#fff1e0", "#fdfaf6", "#ffc887", "#f7a965", "#f2a08f", "#e58a87",
        "#f8b59d", "#d97b7e", "#7a4c4a", "#a5654f", "#523a4a", "#1b2034", "#262c45", "#363c59", "#0a1418", "#0f1f24", "#162a2f", "#203a3c", "#a3a7c6",
        "#c6c3d8", "#e0d4dc", "#5a4f7e", "#7b6c9c", "#9a8cb6", "#b8a8c8", "#d8bccb", "#090c18", "#05080b", "#080d14", "#0b0f22", "#10162c", "#141a38",
        "#1a1f2c", "#231d36", "#2f2544", "#3b2d4a", "#0c1218", "#121a22", "#2a1e26", "#3e2a2c",
    ]),
    ground: Some("#090c18"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
/// Tangent of half the field of view, across the width.
const K: f64 = 0.62;
/// Eye level, in rows.
const HZ: f64 = 56.5;
/// The camera's height above the water.
const CAM: f64 = 1.5;
/// Distance to the far shore.
const SHORE_Z: f64 = 46.0;
/// First row of open water.
const SHORE: f64 = 62.0;
const SUN: (f64, f64) = (151.0, 50.0);
/// The mist sheet: its width in columns (it wraps) and its first row.
const MW: f64 = 480.0;
const M0: f64 = 36.0;
/// The high cloud: its width in columns (it wraps) and its last row.
const CW: f64 = 640.0;
const CR: f64 = 34.0;

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / S - 0.5
}

/// Sharp crests where plain noise crosses its middle: rock ribs and couloirs.
fn ridged(x: f64, y: f64, octaves: u32) -> f64 {
    let (mut s, mut n, mut amp, mut f) = (0.0, 0.0, 0.5, 1.0);
    for i in 0..octaves {
        let v = 1.0 - (2.0 * noise(x * f + f64::from(i) * 17.3, y * f) - 1.0).abs();
        s += amp * v * v;
        n += amp;
        amp *= 0.5;
        f *= 2.1;
    }
    s / n
}

/// The peaks as pyramids, each turned a little: x, z, height, reach, cos and
/// sin of the turn.
struct Range([[f64; 6]; 7]);

impl Range {
    /// Placed by where their summits should land on screen: column, row,
    /// distance, spread, turn.
    fn new() -> Self {
        let at = [
            [66.0, 11.0, 140.0, 0.95, 0.3],
            [38.0, 26.0, 115.0, 1.1, -0.15],
            [116.0, 22.0, 175.0, 0.9, 0.2],
            [92.0, 33.0, 150.0, 1.0, 0.1],
            [96.0, 25.0, 340.0, 1.1, 0.4],
            [180.0, 38.0, 200.0, 1.5, -0.1],
            [8.0, 30.0, 160.0, 1.2, 0.25],
        ];
        Range(at.map(|[sx, row, z, f, a]: [f64; 5]| {
            let h = CAM + ((HZ - row) / 100.0) * K * z - 1.5;
            [((sx - 100.0) / 100.0) * K * z, z, h, h * f, a.cos(), a.sin()]
        }))
    }

    /// The ground's height at (x, z).
    fn terrain(&self, x: f64, z: f64) -> f64 {
        let mut h: f64 = 0.0;
        for &[px, pz, ph, pr, c, s] in &self.0 {
            let (dx, dz) = (x - px, z - pz);
            let (rx, rz) = (dx * c - dz * s, dx * s + dz * c);
            h = h.max(ph * (1.0 - (rx.abs() + rz.abs()) / pr));
        }
        h = h.max((1.0 + 3.0 * fbm(x * 0.04, z * 0.04, 3)) * smooth(SHORE_Z, SHORE_Z + 15.0, z));
        // crags, deeper on the high ground
        h + ((ridged(x * 0.06, z * 0.06, 3) - 0.45) * 6.0 + (ridged(x * 0.2, z * 0.2, 2) - 0.45) * 1.6) * smooth(4.0, 22.0, h)
    }

    /// Distance along a ray from height `oy` with slope `v` and spread `u` to
    /// the terrain beyond the shore, or 0 when it reaches the sky.
    fn march(&self, u: f64, v: f64, oy: f64) -> f64 {
        let (mut z, mut prev) = (SHORE_Z, SHORE_Z);
        let mut i = 0;
        while i < 260 && z < 520.0 {
            let gap = oy + v * z - self.terrain(u * z, z);
            if gap < 0.0 {
                let (mut a, mut b) = (prev, z);
                for _ in 0..7 {
                    let m = (a + b) / 2.0;
                    if oy + v * m - self.terrain(u * m, m) < 0.0 {
                        b = m;
                    } else {
                        a = m;
                    }
                }
                return b;
            }
            prev = z;
            z += (gap * 0.45).max(0.35) + z * 0.002;
            i += 1;
        }
        0.0
    }
}

/// The near pines: x, the row of the tip, size.
const PINES: [(f64, f64, f64); 6] = [(9.0, 5.0, 1.15), (20.0, 38.0, 0.85), (32.0, 66.0, 0.5), (192.0, 10.0, 1.15), (181.0, 40.0, 0.75), (204.0, 26.0, 1.0)];

/// The range, raymarched; the sky; the shore, the lake's look-ups, the near
/// pines, the mist and the cloud, all at pixel resolution; and the picture.
pub struct AlpineDawn {
    /// Pixel rows above the open water.
    sr: usize,
    depth: Vec<f32>,
    alt: Vec<f32>,
    sun: Vec<f32>,
    snow: Vec<f32>,
    up: Vec<f32>,
    rim: Vec<bool>,
    tree_top: Vec<f32>,
    /// For each lake pixel, the row (ascii.rest's) its reflected ray sees.
    src: Vec<f32>,
    /// 0: none, 1: the bank, 2: a pine.
    fg: Vec<u8>,
    fg_shade: Vec<f32>,
    fg_rim: Vec<f32>,
    /// The mist sheet, from pixel row `m0`, `MW · S` samples a row.
    mist: Vec<f32>,
    m0: usize,
    /// The high cloud, `CW · S` samples a row.
    cloud: Vec<f32>,
    hz: Vec<f32>,
    sky: Vec<[f32; 3]>,
    glow: Vec<f32>,
    /// The picture above the water, kept for the lake to look up.
    above: Vec<[f32; 3]>,
    pixels: Pixels,
}

/// An alpine dawn; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let range = Range::new();
    // the direction of the sun
    let n0 = (0.9f64 * 0.9 + 0.3 * 0.3 + 0.14 * 0.14).sqrt();
    let l = [0.9 / n0, 0.3 / n0, 0.14 / n0];
    let n = W * H;
    let sr = (0..H).take_while(|&py| row_y(py) < SHORE).count();
    let ns = sr * W;

    // --- the range, raymarched once --------------------------------------
    let (mut depth, mut alt, mut sun, mut snow, mut up) = (vec![0.0f32; ns], vec![0.0f32; ns], vec![0.0f32; ns], vec![0.0f32; ns], vec![0.0f32; ns]);
    for py in 0..sr {
        let v = ((HZ - row_y(py)) / 100.0) * K;
        for px in 0..W {
            let u = ((col_x(px) + 0.5 - 100.0) / 100.0) * K;
            let z = range.march(u, v, CAM);
            if z == 0.0 {
                continue;
            }
            let k = py * W + px;
            let (wx, wy) = (u * z, CAM + v * z);
            let e = 0.35;
            let hx = (range.terrain(wx + e, z) - range.terrain(wx - e, z)) / (2.0 * e);
            let hz = (range.terrain(wx, z + e) - range.terrain(wx, z - e)) / (2.0 * e);
            let nl = (hx * hx + 1.0 + hz * hz).sqrt();
            let (nx, ny, nz) = (-hx / nl, 1.0 / nl, -hz / nl);
            let mut lit = (nx * l[0] + ny * l[1] + nz * l[2]).max(0.0);
            if lit > 0.0 {
                // cast shadow: walk toward the sun
                let mut s = 0.8;
                while s < 160.0 {
                    let (qx, qy, qz) = (wx + l[0] * s, wy + l[1] * s + 0.15, z + l[2] * s);
                    if qz < SHORE_Z {
                        break;
                    }
                    if qy < range.terrain(qx, qz) {
                        lit = 0.0;
                        break;
                    }
                    s += 0.6 + s * 0.04;
                }
            }
            depth[k] = z as f32;
            alt[k] = wy as f32;
            sun[k] = lit as f32;
            up[k] = ny as f32;
            let grain = fbm(wx * 0.4, wy * 0.25, 2);
            // rock shows through in couloirs running down the fall line
            let gully = ridged(wx * 0.22 + z * 0.05, wy * 0.045, 2);
            snow[k] = (smooth(0.36, 0.52, ny + 0.3 * (grain - 0.5)) * smooth(5.0, 11.0, wy + 5.0 * grain) * (1.0 - 0.75 * smooth(0.62, 0.85, gully))) as f32;
        }
    }
    // sunlit terrain with open sky directly above: the crest line
    let rim = (0..ns).map(|k| k >= W && depth[k] != 0.0 && depth[k - W] == 0.0 && sun[k] > 0.0).collect();

    // --- the far shore's treeline ------------------------------------------
    let mut tree_top: Vec<f32> = (0..W).map(|px| (SHORE - 0.6 - 1.2 * fbm(col_x(px) * 0.06, 2.3, 2)) as f32).collect();
    let mut tx = -2.0;
    while tx < TW + 2.0 {
        let tip = SHORE - 2.6 - hash(tx * 3.0, 8.0) * 4.0 - 1.5 * smooth(60.0, 0.0, tx);
        let slope = 1.1 + hash(tx * 5.0, 9.0) * 0.5;
        for (px, top) in tree_top.iter_mut().enumerate() {
            let d = (col_x(px) + 0.5 - tx).abs();
            if d < 6.0 {
                *top = top.min((tip + d * slope) as f32);
            }
        }
        tx += 1.6 + hash(tx * 9.0, 7.0) * 2.2;
    }

    // --- the lake: where each pixel's reflected ray lands in the picture above
    let mut src = vec![0.0f32; (H - sr) * W];
    for py in sr..H {
        let y = row_y(py);
        let v = ((HZ - y) / 100.0) * K;
        for px in 0..W {
            let x = col_x(px);
            let u = ((x + 0.5 - 100.0) / 100.0) * K;
            let z = range.march(u, -v, -CAM);
            let vs = -v - if z != 0.0 { (2.0 * CAM) / z } else { 0.0 };
            let mut row = HZ - (vs * 100.0) / K - 0.5;
            let mirror = 2.0 * SHORE - 1.0 - (y - 0.5); // the treeline stands on the shore
            if mirror >= f64::from(tree_top[px]) {
                row = mirror;
            }
            src[(py - sr) * W + px] = row as f32;
        }
    }

    // --- the near pines and the bank they stand on -------------------------
    let (mut fg, mut fg_shade, mut fg_rim) = (vec![0u8; n], vec![0.0f32; n], vec![0.0f32; n]);
    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let bank_l = 88.0 + 14.0 * smooth(0.0, 52.0, x) + 2.0 * fbm(x * 0.2, 1.0, 2);
            let bank_r = 90.0 + 12.0 * smooth(TW, TW - 40.0, x) + 2.0 * fbm(x * 0.2, 4.0, 2);
            if y > bank_l || y > bank_r {
                fg[k] = 1;
                fg_shade[k] = (0.15 * hash(px as f64, py as f64)) as f32;
            }
            for &(tx, tip, s) in &PINES {
                let d = y - tip;
                if d < 0.0 {
                    continue;
                }
                let tier = 3.4 * s;
                let f = d / tier - (d / tier).floor();
                let hw = (0.4 + d * 0.2) * (0.5 + 0.5 * f) * (1.0 + 0.6 * (hash(py as f64, tx.floor() * 7.0) - 0.5) * smooth(0.0, 8.0, d));
                let dx = x + 0.5 - tx;
                if dx.abs() <= hw {
                    fg[k] = 2;
                    // the outline catches the dawn sky: a warm rim on the side
                    // facing the sun, a dim cool one on the other, the inside
                    // stays black (a pixel thick here, where it was a cell)
                    let edge = hw - dx.abs() < 0.7;
                    let sunward = if tx < SUN.0 { dx > 0.0 } else { dx < 0.0 };
                    fg_shade[k] = if edge {
                        if sunward {
                            1.0
                        } else {
                            0.5
                        }
                    } else {
                        (0.3 * hash(px as f64 * 3.0, py as f64 * 5.0)) as f32
                    };
                    // the rim is light from the sky behind, so it fades out below the shore
                    fg_rim[k] = if edge { (smooth(70.0, 44.0, y) * (0.75 + 0.25 * hash(px as f64, py as f64 * 7.0))) as f32 } else { 0.0 };
                }
            }
        }
    }

    // --- mist, a wrapping sheet that drifts along the valley ---------------
    let m0 = (0..H).take_while(|&py| row_y(py).floor() < M0).count();
    let mw = (MW * S) as usize;
    let mut mist = vec![0.0f32; mw * (sr - m0)];
    for py in m0..sr {
        let y = row_y(py) - 0.5;
        for i in 0..mw {
            let x = i as f64 / S;
            let q = fbm_wrap(x * 0.0125, y * 0.1, 2, MW * 0.0125);
            mist[(py - m0) * mw + i] = fbm_wrap(x * 0.025 + q * 1.4, y * 0.22 + q, 4, MW * 0.025) as f32;
        }
    }

    // --- thin high cloud, streaked and lit from below by the sun -----------
    let cr = (0..H).take_while(|&py| row_y(py) < CR).count();
    let cw = (CW * S) as usize;
    let mut cloud = vec![0.0f32; cw * cr];
    for py in 0..cr {
        let y = row_y(py);
        for i in 0..cw {
            let x = i as f64 / S;
            let q = fbm_wrap(x * 0.0125, y * 0.12, 2, 8.0);
            let c = fbm_wrap(x * 0.025 + q * 2.0, y * 0.2 + q * 0.8, 4, 16.0);
            cloud[py * cw + i] = (smooth(0.52, 0.7, c - 0.06 * (y - 18.0).abs() / 10.0) * smooth(5.0, 13.0, y) * smooth(33.0, 24.0, y)) as f32;
        }
    }

    // a faint large-scale unevenness, so the open sky and the deep water are
    // never one flat tone
    let hz: Vec<f32> = (0..n).map(|k| fbm(col_x(k % W) * 0.03, (row_y(k / W) - 0.5) * 0.06, 3) as f32).collect();

    // the sky's colour at a point, for the sky itself and as haze on the peaks
    let mut sky = vec![[0.0f32; 3]; ns];
    let mut glow = vec![0.0f32; ns];
    for py in 0..sr {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            let v = clamp(y / 52.0);
            let east = smooth(20.0, 190.0, x);
            let (dx, dy) = (x + 0.5 - SUN.0, (y - SUN.1) * 2.2);
            let ds = dx.hypot(dy);
            let low = v.powf(1.9);
            // indigo overhead, a soft unevenness in it, then a pale lilac and
            // rose band behind the range, lighter than the mountains' shadowed flanks
            let veil = (f64::from(hz[k]) - 0.5) * 0.1 * (1.0 - v);
            sky[k] = [
                (0.03 + veil + low * (0.56 + 0.2 * east)) as f32,
                (0.04 + veil + low * (0.48 + 0.02 * east)) as f32,
                (0.13 + veil * 1.6 + low * (0.62 - 0.12 * east)) as f32,
            ];
            // the sun's glow, kept apart so it can breathe
            glow[k] = ((-ds / 6.0).exp() * 0.65 + (-ds / 15.0).exp() * 0.2 + (-ds / 50.0).exp() * 0.1) as f32;
        }
    }

    Box::new(AlpineDawn {
        sr,
        depth,
        alt,
        sun,
        snow,
        up,
        rim,
        tree_top,
        src,
        fg,
        fg_shade,
        fg_rim,
        mist,
        m0,
        cloud,
        hz,
        sky,
        glow,
        above: vec![[0.0; 3]; ns],
        pixels: Pixels::new(&META),
    })
}

/// A wrapping row of samples, `S` to a column, read at column `x` between samples.
fn sample(row: &[f32], x: f64) -> f64 {
    let s = x * S;
    let i = s.floor();
    let f = s - i;
    let n = row.len() as i64;
    let i0 = (i as i64).rem_euclid(n) as usize;
    let (a, b) = (f64::from(row[i0]), f64::from(row[(i0 + 1) % n as usize]));
    a + (b - a) * f
}

impl Piece for AlpineDawn {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let warm = 0.75 - 0.5 * (-t / 60.0).exp(); // rose first light warming toward gold
                                                   // The sunlit line creeps down the slopes: from the summits at first
                                                   // (ascii.rest's starts at 10, most of the way down).
        let line = 6.0 + 4.0 * (-t / 70.0).exp() + 14.0 * (-t / 24.0).exp();
        let drift = t * 1.1;
        let pulse = 1.0 + 0.06 * (t / 8.0 * TAU).sin(); // the sun's glow breathes
        let wisp: Vec<f64> = (0..W).map(|px| noise((col_x(px) + drift * 0.6) * 0.06, 3.7)).collect();

        // the warm light, from rose toward gold
        let (lr, lg, lb) = (1.0, mix(0.6, 0.8, warm), mix(0.55, 0.4, warm));
        let (mw, cw) = ((MW * S) as usize, (CW * S) as usize);
        let cr_rows = self.cloud.len() / cw;

        for py in 0..self.sr {
            let y = row_y(py);
            for (px, &wisp) in wisp.iter().enumerate() {
                let k = py * W + px;
                let x = col_x(px);
                let gl = f64::from(self.glow[k]) * pulse;
                let gg = 0.62 + 0.28 * smooth(0.15, 0.7, gl); // gold at the core, rose further out
                let [sr, sg, sb] = self.sky[k].map(f64::from);
                let (mut cr, mut cg, mut cb) = (sr + gl, sg + gl * gg, sb + gl * (gg - 0.22));
                let z = f64::from(self.depth[k]);
                if z != 0.0 {
                    let a = f64::from(self.alt[k]);
                    let sn = f64::from(self.snow[k]);
                    let lit = f64::from(self.sun[k]) * smooth(line, line + 7.0, a);
                    let amb = (0.55 + 0.45 * f64::from(self.up[k])) * (0.55 + 0.5 * smooth(4.0, 34.0, a));
                    // snow: deep blue in shadow, rose to gold in the sun, ending sharply
                    let sl = smooth(0.08, 0.24, lit);
                    // full on faces and summits glow gold, glancing light stays rose
                    let gold = clamp(0.6 * smooth(0.25, 0.8, lit) + 0.5 * smooth(14.0, 36.0, a));
                    let br = 0.78 + 0.3 * lit;
                    let snr = mix(0.13 * amb, lr * br, sl);
                    let sng = mix(0.17 * amb, mix(lg - 0.12, lg + 0.14, gold) * br, sl);
                    let snb = mix(0.36 * amb, mix(lb + 0.02, lb + 0.12, gold) * br, sl);
                    // rock: slate in shadow, warm umber in the sun
                    let (rr, rg, rb) = (mix(0.06, 0.4, sl), mix(0.07, 0.2, sl), mix(0.14, 0.2, sl));
                    cr = mix(rr, snr, sn);
                    cg = mix(rg, sng, sn);
                    cb = mix(rb, snb, sn);
                    // forested foothills
                    let wood = smooth(9.0, 4.0, a) * smooth(110.0, 75.0, z);
                    cr = mix(cr, 0.07, wood);
                    cg = mix(cg, 0.09, wood);
                    cb = mix(cb, 0.18, wood);
                    // distance hazes toward the sky behind, and haze settles in
                    // the far valleys so each ridge stands clear of the one behind it
                    let fog = smooth(16.0, 3.0, a) * smooth(70.0, 150.0, z) * 0.6;
                    let haze = fog.max(clamp(1.0 - (-(z - SHORE_Z) / 260.0).exp()) * 0.45);
                    cr = mix(cr, sr + gl, haze);
                    cg = mix(cg, sg + gl * gg, haze);
                    cb = mix(cb, sb + gl * (gg - 0.22), haze);
                    // the first light catches the crest itself in a bright line
                    if self.rim[k] && sl > 0.3 {
                        cr = mix(cr, 1.0, sl);
                        cg = mix(cg, mix(0.89, 0.95, warm), sl);
                        cb = mix(cb, mix(0.76, 0.88, warm), sl);
                    }
                } else {
                    // a few stars still out in the west
                    if y < 34.0 && hash(px as f64, py as f64 * 3.0 + 11.0) > 0.994 {
                        let (fx, fy) = (px as f64, py as f64);
                        let tw = 0.6 + 0.4 * (t * (1.5 + hash(fx, fy) * 3.0) + hash(fy, fx) * TAU).sin();
                        let s = tw * smooth(150.0, 40.0, x) * smooth(34.0, 6.0, y) * 0.75;
                        cr = cr.max(s * 0.9);
                        cg = cg.max(s * 0.92);
                        cb = cb.max(s);
                    }
                    // high cloud, rose-gold toward the sun and mauve away from it
                    if py < cr_rows {
                        let c = sample(&self.cloud[py * cw..(py + 1) * cw], x + t * 0.8) * (0.35 + 0.65 * smooth(40.0, 150.0, x));
                        if c > 0.01 {
                            let g = (-(x + 0.5 - SUN.0).hypot((y - SUN.1) * 1.6) / 55.0).exp();
                            let b = clamp(0.25 + 0.9 * g);
                            let (kr, kg, kb) = (mix(0.32, 1.0, b), mix(0.24, mix(0.62, 0.74, warm), b), mix(0.4, 0.5, b));
                            cr = mix(cr, kr, c * 0.75);
                            cg = mix(cg, kg, c * 0.75);
                            cb = mix(cb, kb, c * 0.75);
                        }
                    }
                    // the sun, just clearing the ridge
                    let ds = (x + 0.5 - SUN.0).hypot(y - SUN.1);
                    if ds < 4.5 {
                        let a = smooth(4.5, 3.3, ds);
                        cr = mix(cr, 1.0, a);
                        cg = mix(cg, 0.96, a);
                        cb = mix(cb, 0.86, a);
                    }
                }
                // mist pooled in the valley behind the shore, lit rose toward the sun
                let e = smooth(20.0, 170.0, x) * (0.6 + 0.4 * warm);
                let near = (-(x + 0.5 - SUN.0).abs() / 22.0).exp() * 0.3;
                let (mr, mg, mb) = (mix(0.48, 0.9, e) + near, mix(0.46, 0.7, e) + near * 0.75, 0.7 + near * 0.5);
                if py >= self.m0 {
                    let row = &self.mist[(py - self.m0) * mw..(py - self.m0 + 1) * mw];
                    let m = sample(row, x + drift);
                    // a ragged top edge: the sheet heaves in long swells and small tufts
                    let edge = 5.0 * (m - 0.5) + 4.0 * (wisp - 0.5);
                    let band = smooth(M0 + 14.0, SHORE - 3.0, y + edge);
                    let a = (0.3 + 0.7 * smooth(0.32, 0.64, m)) * band * 0.6;
                    cr = mix(cr, mr, a);
                    cg = mix(cg, mg, a);
                    cb = mix(cb, mb, a);
                    let top = f64::from(self.tree_top[px]);
                    if y >= top {
                        // the far shore's pines, dark against the mist
                        let s = 0.4 + 0.6 * hash(px as f64 * 7.0, py as f64 * 3.0);
                        cr = 0.04 + 0.03 * s;
                        cg = 0.06 + 0.04 * s;
                        cb = 0.1 + 0.05 * s;
                        // and low wisps drifting across their feet
                        let m = sample(row, x * 0.7 + drift * 1.9 + 211.0);
                        let a = smooth(0.45, 0.72, m) * smooth(top + 1.0, SHORE, y) * 0.6;
                        cr = mix(cr, mr, a);
                        cg = mix(cg, mg, a);
                        cb = mix(cb, mb, a);
                    }
                }
                self.above[k] = [cr as f32, cg as f32, cb as f32];
                self.pixels.set(px, py, self.above[k]);
            }
        }

        // the lake: the picture above, shaken a little by slow ripples, and
        // every 24 s a breath of wind crossing it, breaking up the mirror
        let ground = [0.035, 0.047, 0.094];
        let lr_rows = TH - SHORE;
        let gust_x = (t / 24.0).fract() * 340.0 - 70.0;
        for py in self.sr..H {
            let y = row_y(py);
            let d = (y - SHORE) / lr_rows;
            let fade = smooth(TH + 2.0, TH - 22.0, y);
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let c = if y.floor() == SHORE {
                    // a dark seam where the shore meets the water
                    [0.06, 0.12, 0.14]
                } else {
                    let gust = (-((x - gust_x) / 26.0).powi(2)).exp() * smooth(SHORE, SHORE + 8.0, y);
                    let w1 = noise(x * 0.045 + t * 0.06, y * 0.5 - t * 0.35);
                    let w2 = noise(x * 0.12 - t * 0.1, y * 1.1 - t * 0.7);
                    let sway = ((w1 - 0.5) * (0.4 + 1.4 * d) + (w2 - 0.5) * 0.5) * (1.0 + 3.0 * gust);
                    let sx = (px as f64 + sway * S).round().clamp(0.0, W as f64 - 1.0) as usize;
                    let row = f64::from(self.src[(py - self.sr) * W + px]) + (w2 - 0.5) * (0.6 * d + 2.0 * gust);
                    let sy = ((row + 0.5) * S - 0.5).round().clamp(0.0, self.sr as f64 - 1.0) as usize;
                    let refl = (0.68 - 0.32 * d) * (1.0 - 0.2 * gust);
                    let w3 = noise(x * 0.03 + t * 0.04, y * 1.9 - t * 0.45);
                    let lift = 1.0 + (w3 - 0.5) * (0.4 + 0.5 * d + 0.8 * gust); // long, faint ripple lines
                                                                                // the water gives back a little less colour than it was sent
                    let [ar, ag, ab] = self.above[sy * W + sx].map(f64::from);
                    let grey = (ar + ag + ab) / 3.0;
                    let deep = (f64::from(self.hz[k]) - 0.5) * 0.1 * d; // slow unevenness in the dark water
                    let mut cr = 0.02 + deep + mix(grey, ar, 0.75) * refl * lift;
                    let mut cg = 0.035 + deep + mix(grey, ag, 0.75) * refl * lift;
                    let mut cb = 0.07 + deep * 1.6 + mix(grey, ab, 0.75) * refl * lift;
                    // the sun's road
                    let road_w = 1.5 + (y - SHORE) * 0.45;
                    let road = (-((x + 0.5 - SUN.0) / road_w).powi(2)).exp();
                    let glint = smooth(0.55, 0.85, w2) * road * (0.5 + 0.5 * warm);
                    cr += glint;
                    cg += glint * 0.8;
                    cb += glint * 0.6;
                    [mix(ground[0], cr, fade) as f32, mix(ground[1], cg, fade) as f32, mix(ground[2], cb, fade) as f32]
                };
                self.pixels.set(px, py, c);
            }
        }

        // the near pines and the bank, black against it all, rimmed on the sun side
        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                if self.fg[k] == 0 {
                    continue;
                }
                let s = self.fg_shade[k];
                let (mut r, mut g, mut b) = (0.02 + 0.05 * s, 0.04 + 0.06 * s, 0.05 + 0.06 * s);
                let e = self.fg_rim[k];
                if e > 0.0 && s == 1.0 {
                    // warm on the side facing the sun
                    (r, g, b) = (r + (0.62 - r) * e, g + (0.4 - g) * e, b + (0.38 - b) * e);
                } else if e > 0.0 && s == 0.5 {
                    // cool lilac from the sky on the other
                    (r, g, b) = (r + (0.2 - r) * e, g + (0.22 - g) * e, b + (0.36 - b) * e);
                }
                self.pixels.set(px, py, [r, g, b]);
            }
        }
        self.pixels.draw(grid, env);
    }
}
