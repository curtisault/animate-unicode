//! varanasi ghats: dusk on the ganga. Stepped ghats run along the bank and
//! away toward the afterglow, temple spires black against an indigo to amber
//! sky. Priests raise the aarti lamps on the steps, diyas drift downstream on
//! the dark water, and a boatman rows slowly across the bright reach. After
//! ascii.rest's varanasi-ghats.ts (MIT, @bas3line).
//!
//! The ghats, the spires, the sky's heat ramp, the clouds, the lamps and the
//! water, and how each is shaded, are ascii.rest's, shaded here per pixel
//! instead of per cell. Where ascii.rest draws each of 200 × 100 square cells
//! as a dot as big as it is bright, this draws 320 × 80 cells of two square
//! pixels each, an upper half block over a back colour ([`crate::pixels`]):
//! 1.6 times the detail each way, solid colour, and gradients dithered in
//! colour rather than in dot size. The palette adds the dark tones a solid
//! pixel needs (stone, water and cloud in shadow) where a small dot was
//! enough. In one ink each pixel is octant dots, as many as it is bright.
//! Added: smoke from the aarti lamps, a thin haze curling up from each flame
//! and drifting off with the evening air; and each lamp's road on the water
//! no longer blinks out when its flame dips toward the waterline.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// Varanasi ghats' meta.
pub static META: Meta = Meta {
    name: "varanasi ghats",
    slug: "varanasi-ghats",
    category: Category::Cities,
    note: "dusk aarti on the ganga, lamps on the steps and diyas on the water",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: the sky from indigo through violet, plum and rose to coral
    // and gold, the stone in shadow and in lamplight, the flame, the dark water
    // and the starlight. Then the dusk's darks: stone, figures, water and
    // cloud in shadow.
    palette: Some(&[
        "#141230", "#1d1a42", "#272257", "#332b6a", "#45357a", "#5a4388", "#7259a8", "#6d3a73", "#8a4677", "#a8527a", "#c4607a", "#d97a92", "#b866a0",
        "#9a68c8", "#c868b0", "#e8707a", "#d9705f", "#e88552", "#f29f4a", "#f8b85a", "#fcd07a", "#ffe3a6", "#fff2d2", "#1a1220", "#24182a", "#301f33",
        "#3f2838", "#523340", "#3a2c4c", "#4d3a64", "#5e3524", "#7d4527", "#a0582a", "#c06e2e", "#d98a46", "#ffffff", "#fff6d8", "#ffd860", "#ff9f2a",
        "#f06a1e", "#c8401a", "#8a2a16", "#0f1a2c", "#162540", "#20325a", "#d6d4ee", "#0b0812", "#0d090f", "#130d17", "#090609", "#160e18", "#070916",
        "#0a0e22", "#debca2", "#130d26", "#1c1030", "#2a1410", "#3e2014", "#7a4850", "#b75b51", "#aa735e", "#f0a482", "#b5947f", "#6b5e62",
    ]),
    ground: Some("#0b0812"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
/// The far bank.
const HZ: f64 = 60.0;
/// Where the ghats meet the far bank.
const END: f64 = 150.0;
/// The afterglow, sitting on the far bank.
const GLOW: (f64, f64) = (156.0, HZ - 1.5);
const FLAME: [f64; 3] = [1.0, 0.62, 0.22];
const WASH: [f64; 3] = [1.0, 0.5, 0.2];
/// The cloud deck: its width in columns (it wraps, so it can drift forever) and its depth.
const CW: usize = 800;
const CH: f64 = HZ - 6.0;
const BS: f64 = 0.85;
const GROUND: [f64; 3] = [0.043, 0.031, 0.071];

/// Heat, then colour: indigo overhead, violet, plum, rose, coral, amber, gold.
const RAMP: [[f64; 4]; 12] = [
    [0.0, 0.085, 0.07, 0.24],
    [0.14, 0.13, 0.1, 0.34],
    [0.28, 0.22, 0.14, 0.44],
    [0.42, 0.34, 0.17, 0.48],
    [0.52, 0.47, 0.21, 0.47],
    [0.62, 0.6, 0.26, 0.42],
    [0.7, 0.7, 0.31, 0.36],
    [0.78, 0.8, 0.39, 0.3],
    [0.86, 0.88, 0.5, 0.26],
    [0.92, 0.95, 0.65, 0.32],
    [0.97, 1.0, 0.8, 0.48],
    [1.0, 1.0, 0.92, 0.7],
];

/// Temple spires: centre x, height in rows, half width at the base.
const SPIRES: [(f64, f64, f64); 6] = [(118.0, 33.0, 5.2), (64.0, 19.0, 4.0), (31.0, 15.0, 3.6), (139.0, 8.0, 1.6), (90.0, 11.0, 2.4), (129.0, 10.0, 2.0)];
/// Aarti stations on the near ghat, unevenly spaced: x, streak width, streak length.
const AARTI: [(f64, f64, f64); 4] = [(11.0, 1.15, 1.25), (24.0, 0.8, 0.85), (39.0, 1.25, 1.1), (54.0, 0.75, 0.75)];
/// Umbrellas on the far steps.
const UMBS: [f64; 4] = [70.0, 83.0, 98.0, 109.0];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Water,
    Bank,
    Steps,
    Bld,
    Spire,
    Umb,
    Fig,
    Plat,
}

/// The middle of pixel column `px`, in ascii.rest's columns (cell `x` spans x to x + 1).
fn xc(px: usize) -> f64 {
    (px as f64 + 0.5) / S
}

/// The middle of pixel row `py`, in ascii.rest's rows (its `y = r + 0.5`).
fn yc(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The pixel columns whose middles lie in [a, b), in ascii.rest's columns.
fn xs(a: f64, b: f64) -> std::ops::Range<usize> {
    let p = |v: f64| (v * S - 0.5).ceil().clamp(0.0, W as f64) as usize;
    p(a)..p(b)
}

/// The pixel rows whose middles lie in [a, b), in ascii.rest's rows.
fn ys(a: f64, b: f64) -> std::ops::Range<usize> {
    let p = |v: f64| (v * S - 0.5).ceil().clamp(0.0, H as f64) as usize;
    p(a)..p(b)
}

/// The heat ramp's colour at heat h.
fn ramp(h: f64) -> [f64; 3] {
    let h = clamp(h);
    let mut j = 0;
    while j < RAMP.len() - 2 && h > RAMP[j + 1][0] {
        j += 1;
    }
    let (a, b) = (RAMP[j], RAMP[j + 1]);
    let k = clamp((h - a[0]) / (b[0] - a[0]));
    [mix(a[1], b[1], k), mix(a[2], b[2], k), mix(a[3], b[3], k)]
}

/// The waterline along the ghats, near at the left and running away to the right.
fn wl_f(x: f64) -> f64 {
    if x < END {
        HZ + 0.5 + 20.0 * (1.0 - x / END).powf(1.6)
    } else {
        HZ + 0.5
    }
}

/// The ghats' scale at x: big near, small far.
fn sc_f(x: f64) -> f64 {
    0.45 + 1.75 * (1.0 - x / END).max(0.0).powf(1.3)
}

/// The top step at x.
fn step_top(x: f64) -> f64 {
    wl_f(x) - 9.5 * sc_f(x) * (0.3 + 0.7 * smooth(END, END - 16.0, x))
}

/// The dusk sky's heat: low overhead, rising toward the horizon, hottest at the glow.
fn sky_heat(x: f64, y: f64) -> f64 {
    let v = clamp(y / HZ);
    let (dx, dy) = ((x - GLOW.0).abs(), (GLOW.1 - y).abs());
    // a wide warm band along the horizon, and a small hot core on the bank
    let wide = 0.46 * (-dx / 55.0 - dy / 23.0).exp();
    let core = 0.24 * (-((dx / 13.0).powi(2) + (dy / 4.5).powi(2)).sqrt()).exp();
    0.06 + 0.42 * v * v + wide + core
}

/// A building along the ghats: x0, x1, height above the steps in scale units, seed.
type Bld = (f64, f64, f64, f64);

/// What stands at (xc, y) on the bank, before any light: its material, its
/// colour, how much of the lamps' wash it takes, and (for the sky) its warmth.
struct Spot {
    m: Mat,
    c: [f64; 3],
    al: f64,
    warm: f64,
}

/// The roofline at x: the top of the building there, or the top step.
fn roof_of(blds: &[Bld], x: f64) -> f64 {
    blds.iter().find(|b| x >= b.0 && x < b.1).map_or(step_top(x), |b| step_top(x) - b.2 * sc_f(x))
}

/// The bank at a point, in ascii.rest's units: x a column's middle, y a row's.
fn spot(blds: &[Bld], xc: f64, y: f64) -> Spot {
    let wl = wl_f(xc);
    if y >= wl {
        return Spot { m: Mat::Water, c: [0.0; 3], al: 0.25, warm: 0.0 };
    }
    let (mut m, mut c, mut al) = (Mat::Sky, [0.0; 3], 0.0);
    let s = sc_f(xc);

    // the far bank: an embankment and clumps of trees against the glow
    let clump = smooth(0.42, 0.72, fbm(xc * 0.11, 3.0, 3)) * smooth(160.0, 172.0, xc);
    let bank_top = HZ - 2.2 - 0.6 * fbm(xc * 0.4, 7.0, 2) - 4.0 * clump;
    if xc >= END - 4.0 && y >= bank_top {
        (m, c) = (Mat::Bank, [0.1, 0.07, 0.125]);
    }

    if xc < END {
        let st = step_top(xc);
        // buildings and their rooftop pavilions
        let bi = blds.iter().position(|b| xc >= b.0 && xc < b.1);
        let roof = bi.map_or(st, |i| st - blds[i].2 * s);
        let mut top = roof;
        if let Some(i) = bi {
            let (x0, x1, _, hb) = blds[i];
            let (mid, half) = ((x0 + x1) / 2.0, (x1 - x0) / 2.0);
            if hb > 0.45 {
                // a chhatri: a small dome on four posts
                let cx = if hb > 0.75 { x0 + half * 0.4 } else { mid };
                let dx = (xc - cx).abs() / (1.6 * s);
                if dx < 1.0 {
                    top = top.min(roof - 1.4 * s - 1.5 * s * (1.0 - dx * dx).sqrt());
                }
                if dx < 1.1 && dx > 0.7 {
                    top = top.min(roof - 1.4 * s);
                }
            }
            if (xc - x0).abs() < 0.6 * s || (xc - x1).abs() < 0.6 * s {
                top = top.min(roof - 0.6 * s); // parapet ends
            }
        }
        if y >= top && y < st {
            m = Mat::Bld;
            // backlit stone, a little violet from the sky, darker low down
            let fy = (y - top) / (st - top).max(1.0);
            c = [0.075 * (1.1 - 0.4 * fy), 0.05 * (1.1 - 0.4 * fy), 0.09 * (1.1 - 0.3 * fy)];
            al = 0.5;
            // rows of arched openings, a fair number lit on the near ghats
            if let Some(i) = bi.filter(|_| s > 0.6) {
                let (fx, fz) = ((xc - blds[i].0) / (2.4 * s), (st - y) / (3.0 * s));
                let (ix, iz) = (fx.floor(), fz.floor());
                if fx - ix > 0.35 && fx - ix < 0.7 && fz - iz > 0.25 && fz - iz < 0.75 && iz >= 1.0 && st - y < blds[i].2 * s - 1.5 * s {
                    if hash(i as f64 * 31.0 + ix, iz * 7.0) < if s > 1.2 { 0.28 } else { 0.2 } {
                        c = [c[0] + 0.5, c[1] + 0.28, c[2] + 0.08];
                    } else {
                        (c, al) = ([c[0] * 0.55, c[1] * 0.55, c[2] * 0.65], 0.2);
                    }
                }
            }
        }
        if y >= st {
            // the steps: treads catch the violet sky, risers fall in shadow
            m = Mat::Steps;
            let sp = 1.6 * s;
            let near = (wl - y) / (wl - st);
            if sp < 2.2 {
                // too far to count the steps: a faint alternating tread
                let odd = (wl - y).floor() as i64 & 1 == 1;
                c = if odd { [0.19 * 0.55, 0.13 * 0.55, 0.25 * 0.58] } else { [0.19, 0.13, 0.25] };
                al = if odd { 0.4 } else { 0.75 };
            } else if ((wl - y) / sp) % 1.0 < 0.6 {
                (c, al) = ([0.24 + 0.05 * (1.0 - near), 0.15, 0.2], 1.0);
            } else {
                (c, al) = ([0.05, 0.035, 0.06], 0.3);
            }
            // the wet bottom steps darker
            if wl - y < 1.2 * s {
                (c, al) = ([c[0] * 0.6, c[1] * 0.6, c[2] * 0.75], al * 0.6);
            }
        }
    }

    for &(sx, sh, sw) in &SPIRES {
        let h = roof_of(blds, sx) + 1.0 - y;
        if !(-1.0..=sh + 2.5).contains(&h) {
            continue;
        }
        let f = h / sh;
        // the curved shikhara, an amalaka disc and the kalash on top
        let mut hw = if f <= 1.0 { sw * (1.0 - f.powf(1.8)).max(0.0).powf(0.6) } else { 0.0 };
        if f > 0.92 && f < 1.02 {
            hw = hw.max(sw * 0.32);
        }
        if f >= 1.02 && h < sh + 2.2 {
            hw = hw.max(0.35 + 0.2 * ((h - sh) / 2.2 * PI).sin());
        }
        if h < 0.5 {
            hw = sw * 1.15;
        }
        if (xc - sx).abs() <= hw {
            let band = if (h / 1.6).floor() as i64 % 2 != 0 { 0.8 } else { 1.0 };
            (m, c, al) = (Mat::Spire, [0.085 * band, 0.055 * band, 0.095 * band], 0.35);
        }
    }

    // the big umbrellas on the far steps
    for &ux in &UMBS {
        let (s2, wl2) = (sc_f(ux), wl_f(ux));
        let cy = wl2 - (wl2 - step_top(ux)) * 0.55;
        let (dx, dy) = ((xc - ux) / (3.0 * s2), (cy - y) / (1.4 * s2));
        if dy > 0.0 && dy < 1.0 && dx.abs() < (1.0 - dy * dy).sqrt() {
            (m, c, al) = (Mat::Umb, [0.07, 0.045, 0.075], 0.4);
        }
        if dy <= 0.0 && dy > -0.25 && dx.abs() < 1.0 {
            (m, c, al) = (Mat::Umb, [0.05, 0.03, 0.05], 0.2);
        }
        if (xc - ux).abs() < 0.5 && y > cy && y < cy + 2.2 * s2 {
            (m, c, al) = (Mat::Umb, [0.04, 0.03, 0.04], 0.1);
        }
    }

    // the priests on their platforms, dark against the lit steps
    for &(ax, _, _) in &AARTI {
        let s2 = sc_f(ax);
        let (u, base) = (1.3 * s2, wl_f(ax) - 2.6 * s2);
        let (dx, fy) = ((xc - ax) / u, (base - y) / u);
        // a dhoti, a broad-shouldered torso, a neck and the head
        let bw = if fy < 0.0 {
            -1.0
        } else if fy < 2.2 {
            0.66 - 0.04 * fy
        } else if fy < 4.1 {
            0.58 + 0.32 * smooth(2.2, 3.7, fy)
        } else if fy < 4.5 {
            0.28
        } else {
            -1.0
        };
        if dx.abs() < bw || (dx * 1.05).hypot(fy - 5.05) < 0.62 {
            (m, c, al) = (Mat::Fig, [0.035, 0.022, 0.035], 0.04);
        }
        if y >= base && y < base + 0.7 * s2 && (xc - ax).abs() < 2.3 * u {
            (m, c, al) = (Mat::Plat, [0.12, 0.07, 0.06], 0.8);
        }
    }

    let mut warm = 0.0;
    if m == Mat::Sky {
        let (x, r) = (xc - 0.5, y - 0.5);
        let veil = 0.92 + 0.16 * fbm(x * 0.05, r * 0.1, 3);
        let hh = sky_heat(xc, y) + 0.05 * (fbm(x * 0.07 + 11.0, r * 0.16, 2) - 0.5) + 0.1 * (fbm(x * 0.025 + 3.0, r * 0.09 + 5.0, 3) - 0.5);
        warm = smooth(0.3, 0.9, hh);
        let k = ramp(hh);
        c = [k[0] * veil, k[1] * veil, k[2] * veil];
    }
    Spot { m, c, al, warm }
}

/// The boat at local coordinates: lx along it, ly up from the waterline.
fn boat_at(lx: f64, ly: f64) -> f64 {
    const L: f64 = 9.0;
    if lx.abs() < L && ly > -0.6 && ly < 1.2 + 1.4 * (lx.abs() / L).powi(3) {
        return 1.0;
    }
    if lx > 4.6 && lx < 5.8 && ly > 0.0 && ly < 6.6 {
        return 1.0; // the boatman
    }
    if (lx - 5.2).abs() < 0.7 && (ly - 7.2).abs() < 0.7 {
        return 1.0;
    }
    let (ox, oy) = (lx - 5.6, ly - 5.4); // his oar, raked back into the water
    let along = ox * 0.42 - oy * 0.91;
    if along > 0.0 && along < 8.0 && (ox * 0.91 + oy * 0.42).abs() < 0.32 {
        return 1.0;
    }
    if lx > -4.0 && lx < -0.5 && ly > 0.0 && ly < 2.6 - 0.25 * (lx + 2.2).abs() {
        return 1.0; // a passenger
    }
    0.0
}

/// The ghats: what each pixel is and its lit colour, the water's mirror, the
/// cloud deck, the lamps, and this frame's light.
pub struct VaranasiGhats {
    mat: Vec<Mat>,
    col: Vec<[f32; 3]>,
    alb: Vec<f32>,
    warm: Vec<f32>,
    refl: Vec<[f32; 3]>,
    /// The lit pixel each water pixel mirrors, if any (`u32::MAX`: none).
    mir: Vec<u32>,
    /// The electric lamps' roads on the water.
    sref: Vec<f32>,
    cover: Vec<f32>,
    clit: Vec<f32>,
    /// The cloud deck's depth in pixel rows.
    ch: usize,
    /// The small diyas on the steps: pixel, weight, phase.
    step_lamps: Vec<(usize, f32, f64)>,
    /// The floating diyas: x at t = 0, y, phase, size.
    diyas: Vec<[f64; 4]>,
    /// This frame's lamplight, the warm light the water gives back, and the priests' raised arms.
    dyn_: Vec<[f32; 3]>,
    dref: Vec<f32>,
    arm: Vec<bool>,
    pixels: Pixels,
}

/// Varanasi ghats; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    // buildings along the ghats
    let mut blds: Vec<Bld> = vec![];
    let mut x = -6.0;
    while x < END - 10.0 {
        let s = sc_f(x.max(0.0));
        let w = (8.0 + 10.0 * hash(x, 3.0)) * s;
        let low = if x > 110.0 { 0.8 } else { 1.0 } * (0.45 + 0.55 * smooth(END - 8.0, END - 34.0, x));
        blds.push((x, x + w, (7.0 + 9.0 * hash(x, 4.0)) * low, hash(x, 5.0)));
        x += w;
    }

    let mut mat = vec![Mat::Sky; n];
    let mut col = vec![[0.0f64; 3]; n];
    let mut alb = vec![0.0f32; n];
    let mut warm = vec![0.0f32; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            let p = spot(&blds, xc(px), yc(py));
            (mat[k], col[k], alb[k], warm[k]) = (p.m, p.c, p.al as f32, p.warm as f32);
        }
    }

    // Rim light: silhouette edges facing the glow take its orange light, and
    // their tops the sky's colour.
    let mut rim = vec![[0.0f64; 3]; n];
    for py in 1..H - 1 {
        for px in 3..W - 3 {
            let k = py * W + px;
            if matches!(mat[k], Mat::Sky | Mat::Water) {
                continue;
            }
            let x = xc(px) - 0.5;
            let toward: isize = if x < GLOW.0 { 1 } else { -1 };
            let strong = 0.75 * (-(x - GLOW.0).abs() / 42.0).exp();
            let a = (1..=3).find(|&o| mat[(k as isize + o * toward) as usize] == Mat::Sky).map_or(0.0, |o| strong * [1.0, 0.7, 0.4][o as usize - 1]);
            rim[k] = [0.85 * a, 0.45 * a, 0.26 * a];
            for (o, w) in [(1, 0.3), (2, 0.15)] {
                if mat[k - o * W] == Mat::Sky {
                    let s = col[k - o * W];
                    rim[k] = [rim[k][0] + s[0] * w, rim[k][1] + s[1] * w, rim[k][2] + s[2] * w];
                    break;
                }
            }
        }
    }
    for (c, r) in col.iter_mut().zip(&rim) {
        *c = [c[0] + r[0], c[1] + r[1], c[2] + r[2]];
    }

    // small diyas lining the steps near the aarti, twinkling: picked on
    // ascii.rest's cells, drawn as a soft point of light
    let mut step_lamps = vec![];
    for x in 2..118 {
        let x = x as f64;
        let s = sc_f(x + 0.5);
        let mut r = step_top(x + 0.5).floor();
        while r < wl_f(x + 0.5) - 1.0 {
            let near = (-((x - 33.0) / 36.0).powi(2)).exp();
            if spot(&blds, x + 0.5, r + 0.5).m == Mat::Steps
                && hash(x * 3.0 + 7.0, r * 5.0) < 0.035 * near + 0.006
                && ((wl_f(x + 0.5) - r - 0.5) / (1.6 * s)) % 1.0 < 0.3
            {
                let ph = hash(x, r) * 40.0;
                for py in ys(r - 0.7, r + 1.7) {
                    for px in xs(x - 0.7, x + 1.7) {
                        let d = (xc(px) - x - 0.5).hypot(yc(py) - r - 0.5);
                        let w = (-(d / 0.55).powi(2)).exp();
                        if w > 0.05 && mat[py * W + px] == Mat::Steps {
                            step_lamps.push((py * W + px, w as f32, ph));
                        }
                    }
                }
            }
            r += 1.0;
        }
    }

    // electric lamps along the far ghats, each with its thin road on the water
    let mut sref = vec![0.0f32; n];
    let mut x = 58.0;
    while x < END - 3.0 {
        let s = sc_f(x);
        let ly = step_top(x) - 0.4 * s + (wl_f(x) - step_top(x)) * 0.55 * hash(x, 78.0).powi(2);
        let halo = 1.0 + 3.2 * s;
        let rr = (halo * 3.0).ceil();
        for py in ys(ly - rr, ly + rr + 0.5) {
            for px in xs(x - rr, x + rr + 0.5) {
                let k = py * W + px;
                if matches!(mat[k], Mat::Water | Mat::Sky) {
                    continue;
                }
                let d = (xc(px) - x).hypot(yc(py) - ly);
                // the lamp, and the pool of light it throws on the stone round it
                let v = (-(d / 0.6).powi(2)).exp() * 1.3 + (-d / 1.4).exp() * 0.12;
                let p = (-d / halo).exp() * 0.3 * f64::from(alb[k]);
                col[k] = [col[k][0] + v + p * WASH[0], col[k][1] + v * 0.76 + p * WASH[1], col[k][2] + v * 0.42 + p * WASH[2]];
            }
        }
        let (wl, width, len) = (wl_f(x), 0.35 + 0.35 * s, 9.0 * s);
        for py in ys(wl.floor(), TH) {
            let a = (-(yc(py) - wl) / len).exp() * 0.5;
            if a < 0.01 {
                break;
            }
            for px in xs(x - 3.5, x + 3.5) {
                sref[py * W + px] += (a * (-((xc(px) - x) / width).powi(2)).exp()) as f32;
            }
        }
        x += (5.0 + 10.0 * hash(x, 77.0)) * s.max(0.8);
    }

    // The reflection: the bank and sky mirrored at the waterline.
    let mut refl = vec![[0.0f32; 3]; n];
    let mut mir = vec![u32::MAX; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            if mat[k] != Mat::Water {
                continue;
            }
            let (x, y) = (xc(px), yc(py));
            let (wl, d) = (wl_f(x), y - wl_f(x));
            let mut acc = [0.0; 3];
            for o in 0..2 {
                let o = f64::from(o);
                let ym = wl - d * 0.85 - 0.4 - o * 0.8;
                // the rippled water stretches the sky's light down toward us
                let sky = ramp(sky_heat(x, (wl - d * 0.36 - 1.0 - o * 0.6).max(0.0)));
                let q = if ym < 0.0 { None } else { Some((ym * S).floor() as usize * W + px) };
                match q {
                    Some(q) if !matches!(mat[q], Mat::Water | Mat::Sky) => {
                        // tilted ripples under the bank still catch a little sky
                        for c in 0..3 {
                            acc[c] += col[q][c] + sky[c] * 0.22;
                        }
                        if o == 0.0 {
                            mir[k] = q as u32;
                        }
                    }
                    _ => (0..3).for_each(|c| acc[c] += sky[c]),
                }
            }
            let a = 0.8 - 0.4 * smooth(HZ, TH, y - 0.5);
            refl[k] = acc.map(|v| (v / 2.0 * a) as f32);
        }
    }

    // Clouds: long dusk streaks, dark on top and lit from below; stretched,
    // warped noise gathered into three loose bands.
    let density = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.005, y * 0.03, 3, 4.0);
        let d = fbm_wrap(x * 0.0125 + q * 1.8, y * 0.09 + q * 0.9, 5, 10.0);
        let env = 0.03 * (-((y - 13.0) / 4.0).powi(2)).exp() + 0.1 * (-((y - 29.0) / 5.5).powi(2)).exp();
        // and one thin, broken streak low down, to cross the glow
        let c = 46.5 + 3.0 * (fbm_wrap(x * 0.01, 9.5, 2, 8.0) - 0.5);
        let th = 1.0 + 0.9 * fbm_wrap(x * 0.05, 3.5, 2, 40.0);
        let streak =
            (-((y - c) / th).powi(2)).exp() * smooth(0.42, 0.6, fbm_wrap(x * 0.0125, 21.5, 3, 10.0)) * (0.75 + 0.5 * fbm_wrap(x * 0.05, y * 0.3, 3, 40.0));
        (d + env - 0.08).max(0.36 + 0.3 * streak)
    };
    let ch = (0..H).take_while(|&py| yc(py) < CH).count();
    let mut cover = vec![0.0f32; CW * ch];
    let mut clit = vec![0.0f32; CW * ch];
    for py in 0..ch {
        let y = yc(py);
        for x in 0..CW {
            let x = x as f64;
            let d = density(x, y);
            let i = py * CW + x as usize;
            cover[i] = smooth(0.47, 0.62, d) as f32;
            // the undersides catch the light from below the horizon, the tops go dark
            clit[i] = clamp(0.4 + (d - density(x, y + 2.0)) * 5.5 + (density(x, y - 2.5) - d) * 1.5) as f32;
        }
    }

    // the floating diyas
    let diyas = (0..34)
        .map(|i| {
            let i = f64::from(i);
            [hash(i, 62.0) * 260.0 - 30.0, HZ + 5.0 + 34.0 * hash(i, 61.0).powf(1.3), hash(i, 63.0) * 50.0, 0.6 + 0.6 * hash(i, 64.0)]
        })
        .collect();

    Box::new(VaranasiGhats {
        mat,
        col: col.into_iter().map(|c| c.map(|v| v as f32)).collect(),
        alb,
        warm,
        refl,
        mir,
        sref,
        cover,
        clit,
        ch,
        step_lamps,
        diyas,
        dyn_: vec![[0.0; 3]; n],
        dref: vec![0.0; n],
        arm: vec![false; n],
        pixels: Pixels::new(&META),
    })
}

impl VaranasiGhats {
    /// A flame: a white-hot heart inside an orange halo; the priests are not lit.
    fn add_glow(&mut self, fx: f64, fy: f64, core: f64, halo: f64, amp: f64) {
        let r = (halo * 3.2).ceil();
        for py in ys(fy - r, fy + r + 0.5) {
            for px in xs(fx - r, fx + r + 0.5) {
                let k = py * W + px;
                let d = (xc(px) - fx).hypot((yc(py) - fy) * 0.85);
                let am = if self.mat[k] == Mat::Fig { amp * 0.15 } else { amp };
                let (c, v) = ((-(d / core).powi(2)).exp() * 1.6 * am, (-d / halo).exp() * 0.35 * am);
                let g = &mut self.dyn_[k];
                *g = [g[0] + (c + v * FLAME[0]) as f32, g[1] + (c * 0.86 + v * FLAME[1]) as f32, g[2] + (c * 0.55 + v * FLAME[2]) as f32];
            }
        }
    }

    /// The lamps' wide warm wash on the stone, by how much each surface takes it.
    fn add_wash(&mut self, fx: f64, fy: f64, halo: f64, amp: f64) {
        let r = (halo * 2.6).ceil();
        for py in ys(fy - r, fy + r + 0.5) {
            for px in xs(fx - r, fx + r + 0.5) {
                let k = py * W + px;
                let a = f64::from(self.alb[k]);
                if a == 0.0 {
                    continue;
                }
                let v = (-(xc(px) - fx).hypot(yc(py) - fy) / halo).exp() * amp * a;
                let g = &mut self.dyn_[k];
                *g = [g[0] + (v * WASH[0]) as f32, g[1] + (v * WASH[1]) as f32, g[2] + (v * WASH[2]) as f32];
            }
        }
    }

    /// A flame's road on the water below it.
    fn add_streak(&mut self, fx: f64, wl: f64, width: f64, len: f64, amp: f64) {
        for py in ys(wl.max(0.0).floor(), TH) {
            let dy = yc(py) - wl;
            let a = (-dy / len).exp() * amp * smooth(-0.5, 1.0, dy);
            if a < 0.01 {
                // (ascii.rest stops at the first faint row, even above the waterline)
                if dy <= 0.5 {
                    continue;
                }
                break;
            }
            for px in xs(fx - 3.0 * width - 1.0, fx + 3.0 * width + 1.0) {
                let k = py * W + px;
                if self.mat[k] == Mat::Water {
                    self.dref[k] += (a * (-((xc(px) - fx) / width).powi(2)).exp()) as f32;
                }
            }
        }
    }

    /// Aarti smoke: a thin haze curling up from a raised lamp and drifting off.
    fn add_smoke(&mut self, t: f64, fx: f64, fy: f64, u: f64, i: f64) {
        let top = fy - 22.0 * u.min(1.6);
        for py in ys(top, fy) {
            let y = yc(py);
            let up = (fy - y) / (fy - top); // 0 at the flame, 1 at the top
                                            // the plume leans off with the air, widening and wavering as it climbs
            let cx = fx + up * up * 9.0 * u + (y * 0.35 - t * 0.9 + i * 3.0).sin() * up * 1.6 * u;
            let wid = (0.5 + 2.6 * up) * u;
            for px in xs(cx - 2.0 * wid, cx + 2.0 * wid) {
                let k = py * W + px;
                if self.mat[k] == Mat::Fig || self.arm[k] {
                    continue;
                }
                let x = xc(px);
                let wisp = noise(x * 0.5 - t * 0.3 + i * 17.0, y * 0.35 + t * 0.8);
                let a = (-((x - cx) / wid).powi(2)).exp() * smooth(0.0, 0.12, up) * (1.0 - up) * smooth(0.3, 0.8, wisp) * 0.3;
                let g = &mut self.dyn_[k];
                *g = [g[0] + (a * 0.85) as f32, g[1] + (a * 0.72) as f32, g[2] + (a * 0.78) as f32];
            }
        }
    }
}

impl Piece for VaranasiGhats {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        self.dyn_.fill([0.0; 3]);
        self.dref.fill(0.0);
        self.arm.fill(false);

        // aarti: each lamp is raised and turned in slow circles
        for (i, &(ax, sw, sl)) in AARTI.iter().enumerate() {
            let i = i as f64;
            let s = sc_f(ax);
            let (u, base) = (1.3 * s, wl_f(ax) - 2.6 * s);
            let ph = t * 1.1 + i * 1.3;
            let (fx, fy) = (ax + 0.9 * u + ph.cos() * 1.1 * u, base - 7.2 * u + ph.sin() * 0.6 * u);
            // the arm from the shoulder to the lamp
            let (sx0, sy0) = (ax + 0.62 * u, base - 3.8 * u);
            let (ex, ey) = (fx - sx0, fy + 0.6 * u - sy0);
            let el = ex * ex + ey * ey;
            for py in ys(sy0.min(fy) - 1.0, sy0.max(fy) + 1.5) {
                for px in xs(sx0.min(fx) - 1.0, sx0.max(fx) + 1.5) {
                    let (dx, dy) = (xc(px) - sx0, yc(py) - sy0);
                    let q = clamp((dx * ex + dy * ey) / el);
                    if (dx - q * ex).hypot(dy - q * ey) < 0.28 * u {
                        self.arm[py * W + px] = true;
                    }
                }
            }
            let fl = 0.85 + 0.15 * (t * 13.0 + i * 5.0).sin() * (t * 7.3 + i).sin();
            self.add_glow(fx, fy, 0.6 * u, 1.7 * u, fl);
            self.add_wash(fx, fy + 2.0 * u, 7.5 * s, 0.34 * fl);
            self.add_streak(fx, wl_f(fx), 0.8 * s * sw, 15.0 * s * sl, 0.75 * fl);
            self.add_smoke(t, fx, fy - 0.8 * u, u, i);
        }
        // the step lamps
        for &(k, w, ph) in &self.step_lamps {
            let v = (0.7 + 0.3 * (t * 5.0 + ph).sin()) * f64::from(w);
            let g = &mut self.dyn_[k];
            *g = [g[0] + (v * FLAME[0]) as f32, g[1] + (v * FLAME[1] * 0.95) as f32, g[2] + (v * FLAME[2]) as f32];
        }
        // diyas drifting downstream, nearer ones faster
        for i in 0..self.diyas.len() {
            let [x0, y, ph, sz] = self.diyas[i];
            let near = (y - HZ) / (TH - HZ);
            let x = (x0 + t * (0.25 + 0.9 * near)).rem_euclid(260.0) - 30.0;
            if !(-3.0..=TW + 3.0).contains(&x) || y < wl_f(x) + 0.8 {
                continue;
            }
            let fl = 0.8 + 0.2 * (t * 9.0 + ph).sin();
            let s = (0.35 + 0.9 * near) * sz;
            self.add_glow(x, y - 0.3, 0.45 + 0.3 * s, 0.6 + 1.2 * s, fl * 0.8);
            self.add_streak(x, y + 0.2, 0.3 + 0.35 * s, 2.0 + 5.0 * s, 0.6 * fl);
        }

        let drift = t * 0.9 + 720.0; // starts with the long streaks over the glow
        let bx = (159.0 - t * 0.1 + 24.0).rem_euclid(250.0) - 24.0;
        let by = 71.0 + (t * 0.9).sin() * 0.15;
        let star = [0.839, 0.831, 0.933];
        let sub = 0.5 / S; // the boat's 2 × 2 samples, a quarter pixel from the middle

        for py in 0..H {
            let y = yc(py);
            for px in 0..W {
                let k = py * W + px;
                let (x, xm) = (xc(px) - 0.5, xc(px));
                let m = self.mat[k];
                let base = self.col[k];
                let (mut cr, mut cg, mut cb) = (f64::from(base[0]), f64::from(base[1]), f64::from(base[2]));
                let mut fade = 1.0;

                if m == Mat::Sky {
                    if py < self.ch {
                        let sx = x + drift;
                        let ix = sx.floor();
                        let fx = sx - ix;
                        let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
                        let (i0, i1) = (py * CW + i0, py * CW + (i0 + 1) % CW);
                        let c = f64::from(self.cover[i0]) + f64::from(self.cover[i1] - self.cover[i0]) * fx;
                        if c > 0.01 {
                            // high streaks glow rose from below; low ones stand dark
                            // against the afterglow with only their undersides lit,
                            // gold near the glow
                            let l = f64::from(self.clit[i0]) + f64::from(self.clit[i1] - self.clit[i0]) * fx;
                            let lit = mix(0.3 + 0.7 * l, 0.9 * l * l, smooth(12.0, 30.0, y));
                            let rc = ramp(0.56 + 0.38 * f64::from(self.warm[k]) + 0.08 * l);
                            cr = mix(cr, mix(0.075, rc[0], lit), c);
                            cg = mix(cg, mix(0.05, rc[1], lit), c);
                            cb = mix(cb, mix(0.15, rc[2], lit), c);
                        } else if y < 24.0 && hash(px as f64, py as f64 * 3.0 + 11.0) > 0.9973 {
                            // a star, a pixel big, twinkling
                            let (fx, fy) = (px as f64, py as f64);
                            let tw = 0.4 + 0.25 * (t * (1.3 + hash(fx, fy) * 2.0) + hash(fy, fx) * TAU).sin();
                            let b = 0.45 + 0.85 * tw;
                            (cr, cg, cb) = (cr.max(star[0] * b), cg.max(star[1] * b), cb.max(star[2] * b));
                        }
                    }
                } else if m == Mat::Water {
                    let v = (y - HZ) / (TH - HZ);
                    let w = 0.6 * noise(x * 0.05 + t * 0.08, y * 0.45 - t * 0.45) + 0.4 * noise(x * 0.16 - t * 0.15, y * 0.95 - t * 0.9);
                    let swell = 0.55 + 0.9 * w;
                    (cr, cg, cb) = (0.035 * swell, 0.045 * swell, 0.12 * swell);
                    let wob = (noise(x * 0.04 + 7.0, y * 0.3 - t * 0.6) - 0.5) * (1.2 + 4.0 * v);
                    let sx = ((x + wob).clamp(0.0, TW - 1.001) + 0.5) * S - 0.5;
                    let sx = sx.clamp(0.0, W as f64 - 1.001);
                    let (i0, fx) = (py * W + sx as usize, sx.fract());
                    let lerp = |a: f32, b: f32| f64::from(a) + f64::from(b - a) * fx;
                    let dash = smooth(0.25, 0.75, noise(x * 0.12 + 3.0, y * 0.8 - t * 1.1));
                    // broken bands, except in the bright reach under the glow
                    let reach = (-((xm - GLOW.0) / 15.0).powi(2)).exp() * smooth(HZ + 2.5, HZ + 5.5, y);
                    let da = mix(0.15 + 1.1 * dash, 0.7 + 0.5 * dash, reach);
                    let (r0, r1) = (self.refl[i0], self.refl[i0 + 1]);
                    cr += lerp(r0[0], r1[0]) * da;
                    cg += lerp(r0[1], r1[1]) * da;
                    cb += lerp(r0[2], r1[2]) * da;
                    // the lamplit stone above, given back in broken bands
                    let q = self.mir[k];
                    if q != u32::MAX {
                        let g = self.dyn_[q as usize];
                        cr += f64::from(g[0]) * 0.4 * da;
                        cg += f64::from(g[1]) * 0.4 * da;
                        cb += f64::from(g[2]) * 0.4 * da;
                    }
                    let fl = (lerp(self.dref[i0], self.dref[i0 + 1]) + lerp(self.sref[i0], self.sref[i0 + 1])) * (0.2 + 1.1 * dash);
                    cr += fl * FLAME[0];
                    cg += fl * FLAME[1];
                    cb += fl * FLAME[2];
                    // a narrow road of glitter straight under the glow
                    let gx = (xm - GLOW.0) / (1.3 + (y - HZ) * 0.15);
                    if gx.abs() < 3.0 {
                        let road = (-gx * gx).exp() * (-(y - HZ) / 24.0).exp();
                        let rip = noise(x * 0.45 + y * 0.1 - t * 0.3, y * 1.3 - t * 1.6);
                        let glint = smooth(0.42, 0.78, 0.45 * w + 0.55 * rip) * road;
                        cr += glint + 0.12 * road;
                        cg += 0.78 * glint + 0.07 * road;
                        cb += 0.42 * glint + 0.04 * road;
                    }
                    fade = smooth(TH + 4.0, TH - 16.0, y);
                }

                // the boat and its dark reflection
                let lx = xm - bx;
                if lx > -14.0 && lx < 15.0 && y - 0.5 > by - 11.0 && y - 0.5 < by + 11.0 {
                    let (mut cov, mut rc) = (0.0, 0.0);
                    for (sx2, sy2) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                        let (bpx, bpy) = ((lx + sx2 * sub / 2.0) / BS, (by - (y + sy2 * sub / 2.0)) / BS);
                        cov += boat_at(bpx, bpy);
                        if m == Mat::Water {
                            rc += boat_at(bpx, -bpy * 0.9);
                        }
                    }
                    (cov, rc) = (cov / 4.0, rc / 4.0);
                    if cov > 0.0 {
                        (cr, cg, cb) = (mix(cr, 0.035, cov), mix(cg, 0.022, cov), mix(cb, 0.035, cov));
                    } else if rc > 0.0 {
                        (cr, cg, cb) = (cr * (1.0 - 0.8 * rc), cg * (1.0 - 0.8 * rc), cb * (1.0 - 0.75 * rc));
                    }
                }

                let g = self.dyn_[k];
                let (gr, gg, gb) = (f64::from(g[0]), f64::from(g[1]), f64::from(g[2]));
                if self.arm[k] {
                    (cr, cg, cb) = (0.035 + gr * 0.12, 0.022 + gg * 0.12, 0.035 + gb * 0.12);
                } else {
                    (cr, cg, cb) = (cr + gr, cg + gg, cb + gb);
                }
                let c = [mix(GROUND[0], cr, fade) as f32, mix(GROUND[1], cg, fade) as f32, mix(GROUND[2], cb, fade) as f32];
                self.pixels.set(px, py, c);
            }
        }
        self.pixels.draw(grid, env);
    }
}
