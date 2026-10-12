//! desert night: a moonless desert under the milky way. The galaxy's bright
//! core sits low over a lone acacia on a dune crest and arches up across the
//! sky, split by its dark dust lane; a distant town warms the far horizon and
//! catches the dunes' faces. Stars twinkle, sand glints along the ridges, and
//! now and then a meteor crosses. After ascii.rest's desert-night.ts (MIT,
//! @bas3line).
//!
//! The dunes (a heightfield raymarched once, keeping each pixel's light and
//! shadow), the acacia, the town and the milky way, and how each is shaded,
//! are ascii.rest's, shaded here per pixel instead of per cell. Where
//! ascii.rest draws each of 200 × 100 square cells as a dot as big as it is
//! bright, this draws 320 × 80 cells of two square pixels each, an upper half
//! block over a back colour ([`crate::pixels`]): 1.6 times the detail each
//! way, solid colour, and gradients dithered in colour rather than in dot
//! size. The palette adds the night's dark navies, the sand's shadowed browns
//! and the town's dark ambers, which a solid pixel needs where a small dot
//! was enough. Stars and the town's lamps are a pixel each, a little rarer so
//! the sky holds as many per area. In colour each star is then drawn as a dot
//! by its magnitude, `·`, `•` or `●`, over its cell's sky, and a meteor as a
//! braille streak, finer than the pixels. In one ink each pixel is octant
//! dots, as many as it is bright, stars and meteor included.

use crate::noise::{clamp, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::subcell::{Braille, Encoding};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Desert night's meta.
pub static META: Meta = Meta {
    name: "desert night",
    slug: "desert-night",
    category: Category::Nights,
    note: "the milky way over a lone acacia on moonless dunes, meteors falling",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: the sky's navies to starlight, the core's creams and
    // ambers, the sand's mauves and roses, the galaxy's violets, star tints
    // and the town's oranges. Then the night's darks.
    palette: Some(&[
        "#0a1024", "#101830", "#16213f", "#1e2b50", "#283864", "#34477a", "#45598f", "#5a6fa6", "#7488bd", "#93a5d2", "#b6c3e4", "#d8e0f2", "#f4f6fb",
        "#fff3dc", "#ffe2b4", "#f5c98e", "#e2a86e", "#c4864f", "#9c6440", "#74492f", "#2a2230", "#3d2f3a", "#56404a", "#735358", "#946a62", "#b6836c",
        "#4a3b48", "#6b5562", "#8a6f7a", "#a88590", "#c9a3a3", "#e2bfb4", "#2a2448", "#3d3466", "#57498a", "#7a68a8", "#ffd8a8", "#cfe0ff", "#a9c4ff",
        "#ff9a52", "#e07a3e", "#0d0f1a", "#151827", "#04060c", "#060a18", "#08061a", "#0c0a14", "#140f1c", "#1d1520", "#2b1b1a", "#422a20", "#1a1214",
        "#0f1630", "#24160f", "#3d2012", "#5a2e14", "#7a3e17", "#9e5019",
    ]),
    ground: Some("#04060c"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
const K: f64 = 0.62;
/// Eye level, in ascii.rest's rows.
const HZ: f64 = 64.0;
const CAM: f64 = 6.0;
/// No dune is taller.
const HMAX: f64 = 16.0;
/// The far glow, just under the horizon.
const TOWN: (f64, f64) = (30.0, 66.0);
/// The galaxy's bright centre.
const CORE: (f64, f64) = (152.0, 38.0);
/// The milky way's arch: centre and radius.
const ARC: (f64, f64, f64) = (50.0, 209.8, 199.8);
/// Its angle at the core, and the sweep to the west edge.
const ARC_S: (f64, f64) = (-1.035, 0.8);
const ACACIA: f64 = 150.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Sand,
    Tree,
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

/// desert-night's own fractal noise: each octave shifted along x.
fn fbm(x: f64, y: f64, octaves: u32) -> f64 {
    let (mut s, mut n, mut amp, mut f) = (0.0, 0.0, 0.5, 1.0);
    for i in 0..octaves {
        s += amp * noise(x * f + f64::from(i) * 31.7, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.0;
    }
    s / n
}

/// Where a point sits across the dunes' wave: 0 at a trough, rising gently up
/// the windward side to the crest at 0.72, then dropping down the slip face.
fn phase(x: f64, z: f64) -> f64 {
    let q = (x * 0.9 + z * 0.42 + 26.0 * fbm(x * 0.014, z * 0.014, 3)) / 19.0;
    q - q.floor()
}

/// The crest line of the tall dune the acacia stands on, snaking toward us
/// from its summit.
fn crest_x(z: f64) -> f64 {
    let k = clamp((z - 22.0) / 52.0);
    -4.0 + 26.0 * k * k * (3.0 - 2.0 * k) + 5.0 * ((z - 22.0) * 0.09).sin()
}

/// The tall dune: a sharp crest, a gentle face on the town's side and a steep
/// one on the other.
fn ridge(x: f64, z: f64) -> f64 {
    if z < 8.0 {
        return 0.0;
    }
    let hc = if z <= 74.0 { 0.5 + 13.0 * clamp((z - 12.0) / 62.0).powf(1.25) } else { 13.5 * (1.0 - (z - 74.0) / 9.0) };
    let dx = x - crest_x(z);
    hc - if dx < 0.0 { -dx * 0.36 } else { dx * 0.8 }
}

/// The dunes' height at (x, z).
fn dunes(x: f64, z: f64) -> f64 {
    let u = phase(x, z);
    let prof = if u < 0.72 { (u / 0.72).powf(1.5) } else { ((1.0 - u) / 0.28).powf(0.75) };
    let rg = ridge(x, z);
    // the field lies low around the big dune so its faces stay clean
    let amp = (1.0 + 4.5 * fbm(x * 0.008 + 5.0, z * 0.008, 2)) * (1.0 - 0.8 * smooth(-3.0, 3.0, rg));
    (prof * amp).max(rg) + 0.3 * fbm(x * 0.05, z * 0.05, 2)
}

/// How far a ray through (u, v) on the screen runs before it meets the sand;
/// 0 when it never does.
fn march(u: f64, v: f64) -> f64 {
    let (mut z, mut prev) = (3.0, 3.0);
    for _ in 0..300 {
        if z >= 600.0 {
            break;
        }
        let y = CAM + v * z;
        if v > 0.0 && y > HMAX {
            return 0.0;
        }
        let gap = y - dunes(u * z, z);
        if gap < 0.0 {
            let (mut a, mut b) = (prev, z);
            for _ in 0..7 {
                let m = (a + b) / 2.0;
                if CAM + v * m - dunes(u * m, m) < 0.0 {
                    b = m;
                } else {
                    a = m;
                }
            }
            return b;
        }
        prev = z;
        z += (gap * 0.5).max(0.25) + z * 0.004;
    }
    0.0
}

/// The ray's screen position for a point at scene (x, y).
fn screen(x: f64, y: f64) -> (f64, f64) {
    (((x + 0.5 - TW / 2.0) / 100.0) * K, ((HZ - y) / 100.0) * K)
}

/// A colour's luminance.
fn lum(c: [f64; 3]) -> f64 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    [v[0] / n, v[1] / n, v[2] / n]
}

/// A star: its pixel, brightness, tint, twinkle rate and phase.
struct Star {
    k: usize,
    bright: f64,
    tint: [f64; 3],
    rate: f64,
    ph: f64,
}

impl Star {
    /// Its brightness at t, twinkling: the bright ones less deeply.
    fn at(&self, t: f64) -> f64 {
        let w = (t * self.rate + self.ph).sin();
        self.bright * if self.bright > 0.35 { 0.72 + 0.28 * w } else { 0.6 + 0.4 * w }
    }
}

/// Meteor `n`: when it starts, where, its heading and its speed. Meteor 0 is
/// already falling; meteor n starts somewhere in its 7 second slot.
fn meteor(n: i64) -> (f64, f64, f64, f64, f64, f64) {
    let f = n as f64;
    let start = if n == 0 { -0.35 } else { 7.0 * (f - 1.0) + 2.5 + hash(f, 71.0) * 2.5 };
    let x0 = if n == 0 { 199.0 } else { 30.0 + hash(f, 72.0) * 140.0 };
    let y0 = if n == 0 { 4.0 } else { 4.0 + hash(f, 73.0) * 18.0 };
    // each one heads across the sky rather than straight off its nearer edge
    let a = if n == 0 { 2.65 } else { (if x0 > 100.0 { 2.6 } else { 0.55 }) + (hash(f, 75.0) - 0.5) * 0.4 };
    (start, x0, y0, a.cos(), a.sin(), 52.0 + hash(f, 76.0) * 30.0)
}

/// The desert: what each pixel is, the sand's and the sky's light, the
/// glints, the canopy's rim, the stars and lamps, and the picture.
pub struct DesertNight {
    mat: Vec<Mat>,
    sky: Vec<[f32; 3]>,
    sand: Vec<[f32; 3]>,
    crest: Vec<f32>,
    bark: Vec<f32>,
    air: Vec<f32>,
    stars: Vec<Star>,
    lamps: Vec<(usize, f64, f64)>,
    frame: Vec<[f64; 3]>,
    /// This frame's meteor in colour: each cell it crosses, its braille
    /// dots and its brightness.
    streak: Vec<(usize, u8, f64)>,
    pixels: Pixels,
}

/// A desert night; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    // the town's light comes in low from ahead and to the left
    let l = unit([-0.8, 0.32, 0.5]);
    // the galaxy's cool fill, from up and to the right
    let g = unit([0.6, 0.5, 0.4]);

    // --- the dunes, raymarched once ---------------------------------------
    let mut mat = vec![Mat::Sky; n];
    let mut depth = vec![0.0f32; n];
    let mut sand = vec![[0.0f32; 3]; n];
    let mut crest = vec![0.0f32; n];
    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let x = col_x(px);
            let (u, v) = screen(x, y);
            let z = march(u, v);
            if z == 0.0 {
                continue;
            }
            let k = py * W + px;
            mat[k] = Mat::Sand;
            depth[k] = z as f32;
            let (sx, sy) = (u * z, CAM + v * z);
            let e = 0.2;
            let hx = (dunes(sx + e, z) - dunes(sx - e, z)) / (2.0 * e);
            let hz = (dunes(sx, z + e) - dunes(sx, z - e)) / (2.0 * e);
            let nl = (hx * hx + 1.0 + hz * hz).sqrt();
            let (nx, ny, nz) = (-hx / nl, 1.0 / nl, -hz / nl);
            let mut lit = (nx * l[0] + ny * l[1] + nz * l[2]).max(0.0);
            if lit > 0.0 {
                // a soft shadow: how close the ray to the light passes over the sand
                let mut sh: f64 = 1.0;
                let mut s = 0.6;
                while s < 90.0 {
                    let (qx, qy, qz) = (sx + l[0] * s, sy + l[1] * s + 0.05, z + l[2] * s);
                    if qy > HMAX {
                        break;
                    }
                    sh = sh.min((qy - dunes(qx, qz)) / (0.08 * s));
                    if sh <= 0.0 {
                        break;
                    }
                    s += 0.5 + s * 0.05;
                }
                lit *= clamp(sh);
            }
            let lit = (lit.powf(1.4) * 1.45).min(1.0); // faces square to the light stand out
                                                       // brightest along the crests, shading down each face into the trough
            let ph = phase(sx, z);
            let on_ridge = ridge(sx, z) >= dunes(sx, z) - 0.35;
            let dxc = sx - crest_x(z);
            let fall = if on_ridge { 0.5 + 0.5 * (-dxc.abs() / 7.0).exp() } else { 0.55 + 0.45 * smooth(0.2, 0.72, ph) };
            let near = 0.72 + 0.28 * smooth(5.0, 25.0, z);
            let reach = (0.55 + 0.45 * (-(x - TOWN.0).abs() / 90.0).exp()) * fall * near * smooth(300.0, 120.0, z);
            // starlight from the galaxy's side, and wind ripples across every slope
            let fill = 0.045 * (nx * g[0] + ny * g[1] + nz * g[2]).max(0.0);
            let ripple = (fbm(sx * 0.6 + z * 0.2, z * 0.6, 2) - 0.5) * 0.05;
            let amb = (0.02 + 0.03 * ny + ripple) * near;
            let mut cr = amb * 0.7 + fill * 0.45 + lit * reach;
            let mut cg = amb * 0.62 + fill * 0.65 + lit * 0.63 * reach;
            let mut cb = amb * 1.35 + fill * 1.2 + lit * 0.53 * reach;
            // the big dune's shadowed brink catches a line of starlight
            if on_ridge && z < 76.0 && dxc > 0.0 {
                let cell_w = z * K * 0.01;
                let rim = smooth(2.2 * cell_w, 0.6 * cell_w, dxc) * smooth(10.0, 20.0, z);
                cr += 0.1 * rim;
                cg += 0.14 * rim;
                cb += 0.24 * rim;
            }
            // distance thins the light into the horizon's haze
            let haze = clamp(1.0 - (-z / 260.0).exp()) * mix(0.7, 0.92, smooth(120.0, 300.0, z));
            let hg = (-(x - TOWN.0).abs() / 40.0).exp() * 0.18;
            sand[k] = [mix(cr, 0.07 + hg, haze) as f32, mix(cg, 0.085 + hg * 0.6, haze) as f32, mix(cb, 0.15 + hg * 0.3, haze) as f32];
            let big = if ridge(sx, z) > 0.5 && z < 76.0 { smooth(0.9, 0.0, dxc.abs()) } else { 0.0 };
            crest[k] = ((smooth(0.07, 0.0, (ph - 0.72).abs()) * smooth(260.0, 30.0, z)).max(big) * (0.3 + 0.7 * smooth(0.0, 0.2, lit))) as f32;
        }
    }

    // --- the acacia, on the crest under the galaxy's core -------------------
    // Its foot is where ascii.rest's sand starts in its column: march those cells.
    let base = (0..TH as usize)
        .find(|&r| {
            let (u, v) = screen(ACACIA, r as f64 + 0.5);
            march(u, v) != 0.0
        })
        .unwrap_or(TH as usize) as f64;
    let canopy_top = base - 16.0;
    // an umbrella: a low lumpy dome on top, thinning to the tips, tufts below
    const SPAN: f64 = 17.0;
    let canopy = |cell: f64, px: f64, py: f64| {
        let q = (px - ACACIA) / SPAN;
        if q.abs() > 1.08 {
            return false;
        }
        let top_y = canopy_top + 0.4 + 2.6 * q * q + 1.3 * (noise(px * 0.3, 7.1) - 0.5);
        let bot_y = canopy_top + 3.8 + 1.4 * (1.0 - q * q) - 1.4 * smooth(0.8, 1.08, q.abs())
            + 1.2 * (noise(px * 0.45, 3.3) - 0.5)
            + if hash(cell, 51.0) < 0.2 { 0.9 } else { 0.0 };
        py > top_y && py < bot_y
    };
    // limbs: from the fork up and out to the canopy
    let fork = (ACACIA + 0.3, base - 5.0);
    let limbs = [
        ((ACACIA, base + 1.0), fork, 1.3),
        (fork, (ACACIA - 9.0, canopy_top + 3.0), 0.8),
        (fork, (ACACIA + 1.5, canopy_top + 2.0), 0.75),
        (fork, (ACACIA + 10.0, canopy_top + 3.2), 0.8),
        (fork, (ACACIA - 4.0, canopy_top + 2.6), 0.55),
        ((ACACIA - 4.0, base - 9.0), (ACACIA - 14.0, canopy_top + 3.5), 0.55),
        ((ACACIA + 3.0, base - 8.0), (ACACIA + 15.0, canopy_top + 3.6), 0.5),
    ];
    for py in 0..H {
        let y = row_y(py);
        if y < canopy_top - 3.0 || y >= base + 2.0 {
            continue;
        }
        for px in 0..W {
            let x = col_x(px);
            if (x - ACACIA).abs() > 26.5 {
                continue;
            }
            let sx = x + 0.5;
            let mut on = canopy((x + 0.5).floor(), sx, y);
            for &((ax, ay), (bx, by), w) in &limbs {
                let (lx, ly) = (bx - ax, by - ay);
                let t = clamp(((sx - ax) * lx + (y - ay) * ly) / (lx * lx + ly * ly));
                let (dx, dy) = (sx - (ax + lx * t), y - (ay + ly * t));
                if dx * dx + dy * dy < (w * (1.0 - 0.4 * t)).powi(2) {
                    on = true;
                }
            }
            if on {
                mat[py * W + px] = Mat::Tree;
            }
        }
    }
    // the canopy's upper edge, rimmed by the bulge behind it
    let mut bark = vec![0.0f32; n];
    for py in 1..H {
        for px in 0..W {
            let k = py * W + px;
            if mat[k] != Mat::Tree || mat[k - W] != Mat::Sky {
                continue;
            }
            let (dx, dy) = (col_x(px) + 0.5 - CORE.0, (row_y(py) - 0.5 - CORE.1) * 1.3);
            bark[k] = (0.25 + 0.75 * (-dx.hypot(dy) / 18.0).exp()) as f32;
        }
    }

    // --- the town's lights, a few pinpricks along the far horizon -----------
    let mut lamps: Vec<(usize, f64, f64)> = vec![];
    let mut cols: Vec<f64> = vec![];
    let mut x = TOWN.0 - 9.0;
    while x <= TOWN.0 + 9.0 && lamps.len() < 5 {
        let px = ((x + 0.5) * S) as usize;
        let py = (0..H).find(|&py| depth[py * W + px] > 0.0).unwrap_or(H);
        let skip = py >= H || depth[py * W + px] < 140.0 || hash(x, 91.0) > 0.45 || cols.iter().any(|c| (c - x).abs() < 2.0);
        if !skip {
            cols.push(x);
            lamps.push((py * W + px, hash(x, 92.0) * TAU, 0.7 + hash(x, 93.0) * 1.4));
        }
        x += 1.0;
    }

    // --- the sky: gradient, the town's glow and the milky way --------------
    let mut sky = vec![[0.0f32; 3]; n];
    let mut air = vec![0.0f32; n];
    let mut band = vec![0.0f32; n];
    for py in 0..H {
        let y = row_y(py);
        let v = clamp(y / HZ);
        for px in 0..W {
            let k = py * W + px;
            let x = col_x(px);
            // a saturated navy, so the dither's specks read as colour
            let (mut cr, mut cg, mut cb) = (0.02 + 0.03 * v * v, 0.035 + 0.045 * v * v, 0.1 + 0.08 * v * v);
            // the town: an amber dome rising from below the horizon, its warmth
            // taking over from the navy rather than greying it
            let (tx, ty) = (x + 0.5 - TOWN.0, (y - TOWN.1) * 1.9);
            let td = tx.hypot(ty);
            let glow = (-td / 16.0).exp() * 0.4 + (-td / 60.0).exp() * 0.22;
            let cool = 1.0 - 0.85 * clamp((-td / 22.0).exp() * 1.5);
            cr = cr * cool + glow;
            cg = cg * cool + glow * 0.5;
            cb = cb * cool + glow * 0.16;
            // airglow, a faint blue sheen low down, kept off the town; it drifts
            // each frame. It peaks a little above the horizon, where the air
            // below thins it, and varies along the skyline so it never lies as
            // one flat strip
            let along = 0.72 + 0.5 * fbm(x * 0.025 + 3.0, y * 0.04, 2);
            air[k] = (smooth(0.3, 0.94, v).powi(3) * (1.0 - 0.3 * smooth(0.92, 1.02, v)) * 0.32 * cool * along) as f32;
            // the milky way, along an arc from the core up and over to the west
            let (ax, ay) = (x + 0.5 - ARC.0, y - ARC.1);
            let d = ax.hypot(ay) - ARC.2;
            let s_raw = (ARC_S.0 - ay.atan2(ax)) / ARC_S.1; // 0 at the core end
            let s = clamp(s_raw);
            let end = if s_raw < 0.0 { (-(s_raw / 0.05).powi(2)).exp() } else { 1.0 }; // the band ends at the core
            let lane_end = if s_raw < 0.0 { (-(s_raw / 0.11).powi(2)).exp() } else { 1.0 };
            let w = 9.0 + 8.0 * (1.0 - s).powf(1.4);
            let lane0 = (noise(s * 9.0, 3.3) - 0.5) * w * 0.5 * smooth(0.0, 0.25, s);
            let mut b = (-(d / w).powi(2)).exp();
            let clump = fbm(x * 0.09, y * 0.09, 4);
            b *= 0.35 + 1.1 * smooth(0.3, 0.75, clump);
            b *= (0.55 + 0.55 * (1.0 - s)) * end;
            // the core's bulge
            let (cx, cy) = (x + 0.5 - CORE.0, (y - CORE.1) * 1.3);
            let cd = cx.hypot(cy);
            let core = (-cd / 6.0).exp() * 0.5 + (-cd / 22.0).exp() * 0.25;
            b += core;
            // the dust lane, a crisp rift through the bulge, and its filaments
            let near_core = (-cd / 16.0).exp();
            let lane = (-((d - lane0) / (w * 0.12)).powi(2)).exp()
                * mix((0.55 + 0.6 * fbm(x * 0.12, y * 0.12, 3)) * (0.55 + 0.45 * (1.0 - s)), 1.0, near_core)
                * lane_end;
            let fil = smooth(0.58, 0.78, fbm(x * 0.16 + 9.0, y * 0.16, 3)) * 0.6;
            b *= clamp(1.0 - mix(0.92, 0.97, near_core) * lane) * (1.0 - fil * (-(d / (w * 1.3)).powi(2)).exp());
            band[k] = ((-(d / (w * 1.2)).powi(2)).exp() * end + core) as f32;
            // warm toward the core, cool along the arm
            let warm = clamp((-cd / 26.0).exp() * 1.2 + (1.0 - s) * 0.25);
            cr += b * mix(0.62, 1.0, warm) * 0.75;
            cg += b * mix(0.68, 0.9, warm) * 0.75;
            cb += b * mix(0.95, 0.74, warm) * 0.75;
            sky[k] = [cr as f32, cg as f32, cb as f32];
        }
    }

    // --- stars: thick in the band, a few bright ones everywhere -------------
    // A pixel is 1 / 2.56 of a cell, so each is that much less likely.
    let mut stars = vec![];
    for py in 0..H {
        let y = row_y(py);
        if y >= HZ + 2.0 {
            break;
        }
        for px in 0..W {
            let k = py * W + px;
            if mat[k] != Mat::Sky {
                continue;
            }
            let (fx, fy) = (px as f64, py as f64);
            let p = (0.015 + 0.12 * clamp(f64::from(band[k]))) / (S * S);
            if hash(fx * 7.0 + 3.0, fy * 13.0 + 1.0) > p {
                continue;
            }
            let m = hash(fx * 5.0 + 1.0, fy * 3.0 + 7.0).powi(12);
            let bright = (0.2 + 0.95 * m) * smooth(HZ + 1.0, HZ - 6.0, y);
            let c = hash(fx * 11.0, fy * 17.0 + 5.0);
            let tint = if c < 0.2 {
                [0.78, 0.86, 1.0]
            } else if c < 0.9 {
                [1.0, 1.0, 1.0]
            } else {
                [1.0, 0.82, 0.6]
            };
            stars.push(Star { k, bright, tint, rate: 1.5 + hash(fx, fy * 9.0) * 4.0, ph: hash(fy, fx * 3.0) * TAU });
        }
    }

    Box::new(DesertNight { mat, sky, sand, crest, bark, air, stars, lamps, frame: vec![[0.0; 3]; n], streak: vec![], pixels: Pixels::new(&META) })
}

impl Piece for DesertNight {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let f = &mut self.frame;
        let air0 = (HZ * 0.3).floor();
        for (k, c) in f.iter_mut().enumerate() {
            let (px, py) = (k % W, k / W);
            let to = |v: [f32; 3]| [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])];
            *c = match self.mat[k] {
                Mat::Sky => {
                    let mut c = to(self.sky[k]);
                    let y = row_y(py);
                    if y >= air0 && y < HZ + 2.0 {
                        // the airglow drifts slowly along the horizon
                        let a = f64::from(self.air[k]) * (0.86 + 0.28 * noise((col_x(px) - t * 0.3) * 0.03, (y - 0.5) * 0.08));
                        c[0] += a * 0.3;
                        c[1] += a * 0.5;
                        c[2] += a * 1.2;
                    }
                    c
                }
                Mat::Sand => {
                    let mut c = to(self.sand[k]);
                    let e = f64::from(self.crest[k]);
                    if e > 0.0 {
                        // sand glinting along the brink of each slip face
                        let h = hash(k as f64, 5.0);
                        let g = e * (t * (0.8 + 2.2 * h) + h * 40.0).sin().max(0.0).powi(8) * 0.5;
                        c[0] += g;
                        c[1] += g * 0.85;
                        c[2] += g * 0.7;
                    }
                    c
                }
                Mat::Tree => {
                    let e = f64::from(self.bark[k]);
                    [e * 0.42, e * 0.3, e * 0.2]
                }
            };
        }
        for s in &self.stars {
            let b = s.at(t);
            for (c, tint) in f[s.k].iter_mut().zip(s.tint) {
                *c += b * tint;
            }
        }
        for &(k, ph, rate) in &self.lamps {
            let g = 0.42 + 0.14 * (t * rate + ph).sin() * (t * rate * 2.3 + ph * 3.0).sin();
            f[k] = [g, 0.85 * g, 0.66 * g];
        }
        // a meteor, if one is crossing
        self.streak.clear();
        let n = ((t - 2.5) / 7.0).floor() as i64 + 1;
        for i in (n - 1).max(0)..=n + 1 {
            let (start, x0, y0, dx, dy, speed) = meteor(i);
            let age = t - start;
            if !(0.0..=0.9).contains(&age) {
                continue;
            }
            let (fade_in, fade_out) = (smooth(0.0, 0.12, age), smooth(0.9, 0.6, age));
            let (hx, hy) = (x0 + dx * speed * age, y0 + dy * speed * age * 0.75);
            let len = (speed * age).min(28.0);
            // Two samples a dot along the streak, head to tail, on braille's
            // 2 × 4 dots a cell: two dots each way to a pixel.
            let steps = (len * 4.0 * S).ceil() as usize;
            for j in 0..steps {
                let q = j as f64 / steps as f64;
                let (sx, sy) = (hx - dx * q * len, hy - dy * q * len * 0.75);
                let (bx, by) = (((sx + 0.5) * 2.0 * S - 0.5).round(), ((sy + 0.5) * 2.0 * S - 0.5).round());
                if bx < 0.0 || by < 0.0 || bx >= 2.0 * W as f64 || by >= 2.0 * H as f64 {
                    continue;
                }
                let (bx, by) = (bx as usize, by as usize);
                let k = by / 2 * W + bx / 2;
                if self.mat[k] != Mat::Sky {
                    continue;
                }
                let s = (1.0 - q) * fade_in * fade_out * 1.25;
                if env.mono {
                    let c = &mut f[k];
                    *c = [c[0].max(s * 0.9), c[1].max(s * 0.96), c[2].max(s)];
                } else {
                    let (cell, bit) = (by / 4 * W + bx / 2, 1u8 << ((by % 4) * 2 + bx % 2));
                    match self.streak.iter_mut().find(|d| d.0 == cell) {
                        Some(d) => (d.1, d.2) = (d.1 | bit, d.2.max(s)),
                        None => self.streak.push((cell, bit, s)),
                    }
                }
            }
        }
        let ground = [0.016, 0.024, 0.047];
        for py in 0..H {
            // the bottom rows sink into the ground
            let edge = smooth(TH + 1.0, TH - 12.0, row_y(py));
            for px in 0..W {
                let c = f[py * W + px];
                self.pixels.set(px, py, [mix(ground[0], c[0], edge) as f32, mix(ground[1], c[1], edge) as f32, mix(ground[2], c[2], edge) as f32]);
            }
        }
        self.pixels.draw(grid, env);
        if env.mono {
            return;
        }
        // In colour a star is a dot as big as it is bright, as ascii.rest's
        // are, over the sky its cell's other pixel shows; a meteor is a
        // braille streak, finer than the pixels, over the sky behind it.
        let rgb = |c: [f64; 3]| [c[0] as f32, c[1] as f32, c[2] as f32];
        for s in &self.stars {
            let (px, py) = (s.k % W, s.k / W);
            let b = s.at(t);
            let glyph = if b < 0.4 {
                '·'
            } else if b < 0.8 {
                '•'
            } else {
                '●'
            };
            let (lit, sky) = (f[s.k], f[(py ^ 1) * W + px]);
            if lum(sky) > 0.5 * lum(lit) {
                continue; // too faint against the band to be worth a dot: it stays a pixel
            }
            let ink = self.pixels.nearest(rgb([0, 1, 2].map(|i| lit[i].max(s.tint[i] * (0.35 + 0.65 * b).min(1.0)))));
            let back = self.pixels.nearest(rgb(sky));
            grid.set(px as i64, (py / 2) as i64, glyph, ink);
            grid.set_back(px as i64, (py / 2) as i64, back);
        }
        for &(cell, mask, s) in &self.streak {
            let (cx, cy) = (cell % W, cell / W);
            let (a, b) = (f[2 * cy * W + cx], f[(2 * cy + 1) * W + cx]);
            let sky = [0, 1, 2].map(|i| (a[i] + b[i]) / 2.0);
            let back = self.pixels.nearest(rgb(sky));
            // from the sky behind at the tail to white at the head
            let v = (s * 1.1).min(1.0);
            let ink = self.pixels.nearest(rgb([mix(sky[0], 0.9, v), mix(sky[1], 0.96, v), mix(sky[2], 1.0, v)]));
            grid.set(cx as i64, cy as i64, Braille::glyph(mask), ink);
            grid.set_back(cx as i64, cy as i64, back);
        }
        self.streak.clear();
    }
}
