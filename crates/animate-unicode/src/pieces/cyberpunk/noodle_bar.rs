//! noodle bar: a ramen stall on a rainy night, close enough to smell it. A
//! striped awning keeps the rain off a lit counter and five red stools; under
//! two bare bulbs a cook in a headband, one eye a red implant, stirs the small
//! pot and now and then lifts a basket of noodles out of the stockpot, and the
//! steam billows up past the awning into the light of the big sign above, ラーメン
//! in magenta tubes in a cyan frame, which buzzes, stutters on its faulty メ, and
//! every so often shorts out altogether. A bowl in amber neon puffs its steam
//! beside it. Two customers hunch over their bowls with their backs to us, one
//! hooded, a visor glowing cyan round the sides of the head, the other shaven,
//! acid-green circuits on the scalp and a chrome arm lifting the chopsticks.
//!
//! The rain comes down in three depths, thinner under the awning, and sheets
//! off its scalloped edge in a curtain of drips that splash at the stools'
//! feet. The wet street gives the stall back upside down in its puddles, the
//! sign rippling at the bottom, rings spreading where the drops land. Every so
//! often someone crosses in front with a lit umbrella, its shaft a tube of
//! light that pools on the wet ground. Round the edges: a lit window with a
//! television behind its blinds, an air conditioner dripping, a cat's eyes
//! between beer crates, a hologram menu glitching on the back wall, and down
//! the alley to the right, towers fading into the smog, a searchlight and a
//! craft passing.

use crate::noise::{clamp, fbm, hash, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// Noodle bar's meta.
pub static META: Meta = Meta {
    name: "noodle bar",
    slug: "noodle-bar",
    category: Category::Cyberpunk,
    note: "a ramen stall in the rain, steam rising into its buzzing neon sign",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // The night's near-blacks, violets and navies; the stall's warm woods and
    // bulb light; the awning's reds; the neon magentas, cyans and greens; the
    // steam's lit greys; a few metals and the white of a hot tube.
    palette: Some(&[
        "#06050a", "#0b0812", "#110c1b", "#181126", "#211733", "#2b1d42", "#382654", "#0b0f1a", "#121a2b", "#1c263d", "#283550", "#1a1214", "#2a1a18",
        "#3d2416", "#5a3218", "#7d461d", "#a65e24", "#d07e30", "#f0a246", "#ffc46e", "#ffe0a8", "#fff4dc", "#3a0c16", "#5e1424", "#8e1e30", "#c8303c",
        "#ff4a3a", "#3e0c34", "#6a1258", "#9c1a7e", "#d02aa2", "#ff4fc4", "#ff92dc", "#ffd2f0", "#06222e", "#0a3e4e", "#0f6478", "#1696ae", "#2cd2e8",
        "#8cf2fc", "#dafcff", "#16300e", "#2e6414", "#62c41c", "#b4ff56", "#3a3346", "#585068", "#7e7590", "#a8a0b8", "#d4cee0", "#a04a64", "#e07890",
        "#2a3a8c", "#4a6aff", "#4a4e5c", "#7a8090", "#ffffff", "#4c3070", "#6c4a96", "#0c1c22", "#24140c", "#8c7a70", "#bfae9f", "#e8dccf",
    ]),
    ground: Some("#06050a"),
    ..Meta::DEFAULT
};

/// The picture in pixels.
const W: usize = 320;
const H: usize = 160;
/// Where the stall stands on the street.
const GROUND: f64 = 134.0;
/// The awning's ends and its top.
const AWN_L: f64 = 44.0;
const AWN_R: f64 = 276.0;
const AWN_TOP: f64 = 42.0;
/// The counter's top.
const COUNTER: f64 = 102.0;
/// The main sign's box, the hanging sign's and the bowl's: x0, y0, x1, y1.
const SIGN: [f64; 4] = [78.0, 2.0, 242.0, 40.0];
const VSIGN: [f64; 4] = [20.0, 56.0, 40.0, 120.0];
const BOWL: [f64; 4] = [252.0, 8.0, 276.0, 34.0];
/// The bulbs hanging under the awning.
const BULBS: [(f64, f64); 2] = [(125.0, 64.0), (226.0, 64.0)];
/// The stools, and which two have customers.
const STOOLS: [f64; 5] = [72.0, 106.0, 144.0, 184.0, 232.0];
const A_X: f64 = 106.0;
const B_X: f64 = 144.0;
/// The cook's head, at rest.
const COOK: (f64, f64) = (204.0, 71.0);
/// The hologram menu on the back wall.
const HOLO: [f64; 4] = [60.0, 62.0, 94.0, 86.0];
/// The lit window, top left.
const WIN: [f64; 4] = [6.0, 6.0, 34.0, 26.0];
/// The passer-by comes round every this many seconds.
const PASS: f64 = 24.0;

type C = [f64; 3];

const MAGENTA: C = [1.0, 0.2, 0.72];
const CYAN: C = [0.16, 0.86, 1.0];
const AMBER: C = [1.0, 0.6, 0.16];
const GREEN: C = [0.5, 1.0, 0.18];
const WARM: C = [1.0, 0.68, 0.4];

#[inline]
fn mixc(a: C, b: C, k: f64) -> C {
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

#[inline]
fn addc(a: C, b: C, k: f64) -> C {
    [a[0] + b[0] * k, a[1] + b[1] * k, a[2] + b[2] * k]
}

#[inline]
fn scale(a: C, k: f64) -> C {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// Distance from (px, py) to the segment `s` (x0, y0, x1, y1).
#[inline]
fn seg_d(px: f64, py: f64, s: &[f64; 4]) -> f64 {
    let (dx, dy) = (s[2] - s[0], s[3] - s[1]);
    let l2 = dx * dx + dy * dy;
    let h = if l2 > 0.0 { (((px - s[0]) * dx + (py - s[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (px - s[0] - dx * h).hypot(py - s[1] - dy * h)
}

/// Signed distance to an ellipse, near enough.
#[inline]
fn sd_ellipse(dx: f64, dy: f64, rx: f64, ry: f64) -> f64 {
    ((dx / rx).hypot(dy / ry) - 1.0) * rx.min(ry)
}

/// Signed distance to a box of half-sizes (hx, hy).
#[inline]
fn sd_box(dx: f64, dy: f64, hx: f64, hy: f64) -> f64 {
    let (qx, qy) = (dx.abs() - hx, dy.abs() - hy);
    qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0)
}

/// Distance from a point to a rectangle (0 inside).
fn rect_d(x: f64, y: f64, r: &[f64; 4]) -> f64 {
    let dx = (r[0] - x).max(x - r[2]).max(0.0);
    let dy = (r[1] - y).max(y - r[3]).max(0.0);
    dx.hypot(dy)
}

fn inside(x: f64, y: f64, r: &[f64; 4]) -> bool {
    x >= r[0] && x < r[2] && y >= r[1] && y < r[3]
}

/// Whether a point inside a rectangle is on its one-pixel border.
fn on_edge(x: f64, y: f64, r: &[f64; 4]) -> bool {
    !inside(x, y, &[r[0] + 1.0, r[1] + 1.0, r[2] - 1.0, r[3] - 1.0])
}

/// A soft bump: up from `a` to `a + e`, down again from `b - e` to `b`.
fn pulse(p: f64, a: f64, b: f64, e: f64) -> f64 {
    smooth(a, a + e, p) * smooth(b, b - e, p)
}

/// The awning's scalloped bottom edge at column x.
fn hem(x: f64) -> f64 {
    let u = (x - AWN_L).rem_euclid(12.0);
    56.0 + 3.0 * (1.0 - ((u - 6.0) / 6.0).powi(2))
}

/// A polyline as segments.
fn poly(pts: &[(f64, f64)]) -> Vec<[f64; 4]> {
    pts.windows(2).map(|w| [w[0].0, w[0].1, w[1].0, w[1].1]).collect()
}

/// A glyph's strokes, drawn in a unit box, placed at (x0, y0) at size `s`.
fn place(unit: &[[f64; 4]], x0: f64, y0: f64, s: f64) -> Vec<[f64; 4]> {
    unit.iter().map(|g| [x0 + g[0] * s, y0 + g[1] * s, x0 + g[2] * s, y0 + g[3] * s]).collect()
}

/// ラ, ー, メ and ン as tube strokes in a unit box.
fn ramen() -> [Vec<[f64; 4]>; 4] {
    let mut ra = vec![[0.22, 0.08, 0.78, 0.08], [0.12, 0.36, 0.88, 0.36]];
    ra.extend(poly(&[(0.88, 0.36), (0.84, 0.6), (0.7, 0.78), (0.5, 0.9), (0.26, 0.96)]));
    let choon = vec![[0.06, 0.5, 0.94, 0.5]];
    let mut me = poly(&[(0.84, 0.06), (0.64, 0.46), (0.42, 0.72), (0.12, 0.94)]);
    me.push([0.26, 0.32, 0.8, 0.84]);
    let mut n = vec![[0.12, 0.16, 0.36, 0.36]];
    n.extend(poly(&[(0.1, 0.9), (0.42, 0.82), (0.68, 0.6), (0.9, 0.18)]));
    [ra, choon, me, n]
}

/// Four made-up kanji for the hanging sign.
fn kanji() -> [Vec<[f64; 4]>; 4] {
    [
        vec![[0.1, 0.12, 0.9, 0.12], [0.5, 0.12, 0.5, 0.9], [0.2, 0.5, 0.8, 0.5], [0.06, 0.9, 0.94, 0.9]],
        vec![[0.18, 0.06, 0.82, 0.06], [0.18, 0.06, 0.18, 0.94], [0.82, 0.06, 0.82, 0.94], [0.18, 0.94, 0.82, 0.94], [0.18, 0.5, 0.82, 0.5]],
        vec![[0.12, 0.3, 0.88, 0.3], [0.12, 0.3, 0.12, 0.7], [0.88, 0.3, 0.88, 0.7], [0.12, 0.7, 0.88, 0.7], [0.5, 0.02, 0.5, 0.98]],
        vec![[0.5, 0.04, 0.5, 0.48], [0.5, 0.48, 0.08, 0.96], [0.5, 0.48, 0.92, 0.96], [0.16, 0.22, 0.3, 0.4], [0.84, 0.22, 0.7, 0.4]],
    ]
}

/// A neon tube: its strokes' core and glow over a box round them.
struct Lamp {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    core: Vec<f32>,
    glow: Vec<f32>,
    col: C,
}

fn lamp(segs: &[[f64; 4]], r: f64, col: C) -> Lamp {
    let pad = r + 14.0;
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for s in segs {
        x0 = x0.min(s[0]).min(s[2]);
        y0 = y0.min(s[1]).min(s[3]);
        x1 = x1.max(s[0]).max(s[2]);
        y1 = y1.max(s[1]).max(s[3]);
    }
    let x0 = (x0 - pad).max(0.0) as usize;
    let y0 = (y0 - pad).max(0.0) as usize;
    let x1 = ((x1 + pad) as usize).min(W);
    let y1 = ((y1 + pad) as usize).min(H);
    let (w, h) = (x1 - x0, y1 - y0);
    let mut core = vec![0.0f32; w * h];
    let mut glow = vec![0.0f32; w * h];
    for ly in 0..h {
        for lx in 0..w {
            let (x, y) = ((x0 + lx) as f64 + 0.5, (y0 + ly) as f64 + 0.5);
            let d = segs.iter().map(|s| seg_d(x, y, s)).fold(f64::MAX, f64::min);
            let e = (d - r).max(0.0);
            core[ly * w + lx] = smooth(r + 0.6, r - 0.4, d) as f32;
            glow[ly * w + lx] = (0.42 * (-e / 1.8).exp() + 0.2 * (-e / 6.0).exp()) as f32;
        }
    }
    Lamp { x0, y0, w, h, core, glow, col }
}

/// The interior's light from the two bulbs, 0–1.
fn bulb_light(x: f64, y: f64) -> f64 {
    let mut l = 0.3;
    for &(bx, by) in &BULBS {
        let d = (x - bx).hypot((y - by) * 1.2);
        l += 0.75 * (-d / 46.0).exp() + 0.35 * (-d / 10.0).exp();
    }
    l.min(1.3)
}

/// The building's wet brick wall.
fn wall(x: f64, y: f64) -> C {
    let row = (y / 4.0).floor();
    let off = if (row as i64).rem_euclid(2) == 0 { 0.0 } else { 4.5 };
    let bx = ((x + off) / 9.0).floor();
    let mortar = y.rem_euclid(4.0) < 1.0 || (x + off).rem_euclid(9.0) < 1.0;
    let v = 0.75 + 0.35 * hash(bx + 300.0, row) + 0.5 * (fbm(x * 0.08, y * 0.08, 3) - 0.5);
    let mut c = [0.085 * v, 0.06 * v, 0.1 * v];
    if mortar {
        c = scale(c, 0.55);
    }
    let stain = smooth(0.55, 0.78, noise(x * 0.6, y * 0.015));
    addc(c, [0.025, 0.025, 0.045], stain)
}

/// The sky down the alley: smog lit from below.
fn smog(y: f64) -> C {
    let v = (y / 112.0).min(1.0);
    [0.06 + 0.3 * v * v, 0.04 + 0.08 * v * v, 0.11 + 0.24 * v * v]
}

/// Down the alley to the right: towers fading into the smog.
fn alley(x: f64, y: f64) -> C {
    let sky = smog(y);
    if y >= 112.0 {
        // the alley's wet floor, giving back the glow
        let g = 0.5 + 0.5 * noise(x * 0.3, y * 1.5);
        return mixc([0.03, 0.02, 0.04], smog(224.0 - y), 0.35 * g);
    }
    let mut c = sky;
    for &(x0, x1, top, hz) in &[(298.0, 320.0, 4.0, 0.72), (279.0, 297.0, 14.0, 0.58), (290.0, 309.0, 36.0, 0.4)] {
        if x >= x0 && x < x1 && y >= top {
            let mut b: C = [0.03, 0.025, 0.06];
            let (u, v) = (x - x0, y - top);
            if u.rem_euclid(3.0) < 1.0 && v.rem_euclid(4.0) >= 2.0 && v.rem_euclid(4.0) < 3.0 && hash(x * 3.0, (v / 4.0).floor() + top) > 0.55 {
                b = if hash(x, top) > 0.3 { [0.9, 0.6, 0.3] } else { [0.3, 0.8, 0.9] };
            }
            c = mixc(b, sky, hz + (1.0 - hz) * 0.7 * smooth(60.0, 112.0, y));
        }
    }
    if (291.0..293.0).contains(&x) && (42.0..84.0).contains(&y) && (y - 42.0).rem_euclid(7.0) < 5.5 {
        c = mixc([1.0, 0.3, 0.45], sky, 0.35);
    }
    c
}

/// Whether a pixel looks down the alley.
fn in_alley(x: f64, y: f64) -> bool {
    y < GROUND && (x >= AWN_R || (x >= 270.0 && y >= hem(x)))
}

/// What a static pixel is: its colour lit by the bulbs, how much of the neon
/// it catches, and whether the cook stands in front of it.
struct Px {
    c: C,
    resp: f64,
    deep: bool,
}

#[allow(clippy::too_many_lines)]
fn paint(x: f64, y: f64) -> Px {
    let px = |c: C, resp: f64| Px { c, resp, deep: false };
    // the hanging sign, on its bracket from the left post
    if inside(x, y, &VSIGN) {
        let edge = on_edge(x, y, &VSIGN);
        return px(if edge { [0.1, 0.09, 0.13] } else { [0.03, 0.025, 0.05] }, 0.3);
    }
    if (38.0..50.0).contains(&x) && (57.0..59.0).contains(&y) {
        return px([0.14, 0.13, 0.17], 0.8);
    }
    // the main sign's box
    if inside(x, y, &SIGN) {
        let edge = on_edge(x, y, &SIGN);
        if edge {
            return px([0.12, 0.1, 0.15], 0.5);
        }
        let v = 0.8 + 0.4 * fbm(x * 0.1, y * 0.1, 2);
        return px([0.035 * v, 0.025 * v, 0.05 * v], 0.25);
    }
    // the sign's feet on the awning
    if (40.0..AWN_TOP).contains(&y) && ((96.0..99.0).contains(&x) || (221.0..224.0).contains(&x)) {
        return px([0.1, 0.09, 0.12], 0.6);
    }
    // the awning
    if (AWN_L..AWN_R).contains(&x) && y >= AWN_TOP && y < hem(x) {
        if y < AWN_TOP + 2.0 {
            return px([0.2, 0.17, 0.22], 1.2);
        }
        let u = (x - AWN_L) / 12.0;
        let red = (u.floor() as i64).rem_euclid(2) == 0;
        let base: C = if red { [0.3, 0.05, 0.085] } else { [0.2, 0.17, 0.19] };
        let fold = 0.8 + 0.2 * (u.fract() * PI).sin();
        let sheen = 0.06 * smooth(0.58, 0.8, noise(x * 0.9, y * 0.07));
        let mut c = addc(scale(base, fold), [1.0, 1.0, 1.0], sheen);
        if y > hem(x) - 1.5 {
            c = scale(c, 0.55);
        }
        return px(c, 0.35 + 1.1 * smooth(52.0, AWN_TOP, y));
    }
    // the posts
    if (AWN_TOP..GROUND).contains(&y) && ((50.0..55.0).contains(&x) || (265.0..270.0).contains(&x)) {
        let u = if x < 100.0 { x - 50.0 } else { x - 265.0 };
        let c = if u < 1.0 {
            [0.2, 0.18, 0.24]
        } else if u >= 4.0 {
            [0.16, 0.11, 0.09]
        } else {
            [0.07, 0.065, 0.09]
        };
        return px(c, 0.7);
    }
    let stall = (55.0..265.0).contains(&x);
    // the stools
    for &sx in &STOOLS {
        let dx = x - sx;
        if ((dx / 8.5).powi(2) + ((y - 114.5) / 1.9).powi(2)) <= 1.0 {
            let l = 0.75 + 0.25 * (1.0 - (dx / 8.5).abs());
            return px([0.62 * l, 0.13 * l, 0.15 * l], 0.05);
        }
        if dx.abs() <= 8.0 && (114.5..118.0).contains(&y) {
            return px([0.26, 0.04, 0.07], 0.05);
        }
        if (dx.abs() <= 5.0 && (125.0..126.0).contains(&y)) || (dx.abs() <= 1.0 && (118.0..131.0).contains(&y)) {
            return px(if dx < 0.0 { [0.4, 0.36, 0.42] } else { [0.18, 0.16, 0.2] }, 0.05);
        }
        if ((dx / 6.0).powi(2) + ((y - 132.0) / 1.4).powi(2)) <= 1.0 {
            return px([0.12, 0.11, 0.14], 0.05);
        }
    }
    // the counter: its lit top, the edge, a cyan strip under the lip, the panel
    if stall && (COUNTER..GROUND).contains(&y) {
        let l = bulb_light(x, 90.0);
        let c = if y < COUNTER + 4.0 {
            let grain = 0.85 + 0.25 * noise(x * 0.4, y * 3.0);
            scale([0.55, 0.33, 0.18], l * grain * (0.75 + 0.1 * (y - COUNTER)))
        } else if y < COUNTER + 5.0 {
            scale([0.85, 0.56, 0.32], l)
        } else if y < COUNTER + 6.0 {
            [0.55, 0.95, 1.0]
        } else if y < 127.0 {
            let slat = if (x - 55.0).rem_euclid(8.0) < 1.0 { 0.55 } else { 1.0 };
            let v = 0.8 + 0.3 * noise(x * 0.2, y * 0.6);
            addc([0.12 * v * slat, 0.06 * v * slat, 0.05 * v * slat], CYAN, 0.2 * (-(y - COUNTER - 6.0) / 4.0).exp())
        } else if (129.0..130.0).contains(&y) {
            [0.22, 0.2, 0.25]
        } else {
            [0.03, 0.025, 0.04]
        };
        return px(c, 0.05);
    }
    if stall && y >= hem(x) - 0.5 && y < COUNTER {
        let l = bulb_light(x, y);
        // the bulbs and their cords
        for &(bx, by) in &BULBS {
            if (x - bx).abs() < 0.6 && y < by - 2.0 {
                return px([0.1, 0.08, 0.08], 0.0);
            }
            if ((x - bx) / 2.4).powi(2) + ((y - by) / 2.8).powi(2) <= 1.0 {
                return px([1.0, 0.96, 0.8], 0.0);
            }
        }
        // the stockpot and the small pot, the cook behind them
        let pot = |x0: f64, x1: f64, top: f64| -> Option<C> {
            if x < x0 || x >= x1 || y < top - 1.5 {
                return None;
            }
            let (cx, hw) = ((x0 + x1) / 2.0, (x1 - x0) / 2.0);
            let u = (x - cx) / hw;
            if y < top + 1.5 {
                // the rim, round the dark broth
                if ((x - cx) / (hw - 1.5)).powi(2) + ((y - top) / 1.0).powi(2) < 1.0 {
                    return Some([0.22, 0.12, 0.06]);
                }
                return Some(scale([0.9, 0.8, 0.65], l));
            }
            let sh = 0.45 + 0.6 * (1.0 - u * u) + 0.9 * smooth(0.22, 0.0, (u + 0.55).abs()) - 0.25 * smooth(0.6, 1.0, u);
            let band = if (y - top - 3.0).abs() < 0.6 { 0.7 } else { 1.0 };
            Some(scale([0.6, 0.58, 0.62], sh * l * band))
        };
        if let Some(c) = pot(162.0, 188.0, 85.0).or_else(|| pot(216.0, 236.0, 95.0)) {
            return px(c, 0.0);
        }
        if (236.0..246.0).contains(&x) && (95.0..96.5).contains(&y) {
            return px([0.12, 0.1, 0.1], 0.0);
        }
        let deep = |c: C| Px { c, resp: 0.06, deep: true };
        // the shelf, with jars and bottles
        if (96.0..160.0).contains(&x) && (74.0..76.0).contains(&y) {
            return deep(scale([0.75, 0.48, 0.26], l));
        }
        if (96.0..160.0).contains(&x) && (76.0..78.0).contains(&y) {
            return deep(scale([0.2, 0.12, 0.07], l));
        }
        for &(jx, w, h, kind) in &[
            (98.0, 5.0, 8.0, 0),
            (104.0, 5.0, 6.0, 1),
            (111.0, 2.0, 10.0, 2),
            (115.0, 2.0, 9.0, 1),
            (134.0, 6.0, 7.0, 0),
            (141.0, 5.0, 8.0, 2),
            (148.0, 2.0, 10.0, 1),
            (152.0, 5.0, 6.0, 0),
        ] {
            let neck = w < 3.0 && y < 74.0 - h + 3.0;
            if x >= jx && x < jx + w && y >= 74.0 - h && y < 74.0 && (!neck || x < jx + 1.0) {
                let glass: C = match kind {
                    0 => [0.7, 0.4, 0.12],
                    1 => [0.15, 0.4, 0.2],
                    _ => [0.55, 0.52, 0.5],
                };
                let hi = if x < jx + 1.0 { 1.6 } else { 0.9 };
                return deep(scale(glass, l * hi));
            }
        }
        // the rail of ladles and strainers
        if (160.0..198.0).contains(&x) && (61.0..62.0).contains(&y) {
            return deep(scale([0.6, 0.58, 0.6], l));
        }
        for i in 0..5 {
            let ux = 164.0 + 7.0 * i as f64;
            let ladle = i < 3;
            if (x - ux).abs() < 0.6 && (62.0..70.0).contains(&y) {
                return deep(scale([0.5, 0.48, 0.5], l));
            }
            if (70.0..74.0).contains(&y) && (x - ux).abs() <= if ladle { 2.0 } else { 3.0 - (y - 70.0) * 0.5 } {
                let mesh = if !ladle && hash(x, y) > 0.5 { 0.4 } else { 1.0 };
                return deep(scale([0.45, 0.43, 0.46], l * mesh));
            }
        }
        // the paper menu slips on the right
        for i in 0..4 {
            let tx = 241.0 + 6.0 * i as f64;
            let bottom = 84.0 + 3.0 * hash(i as f64, 4.0);
            if (x - tx - 2.0).abs() < 0.6 && (59.0..62.0).contains(&y) {
                return deep([0.1, 0.08, 0.07]);
            }
            if x >= tx && x < tx + 5.0 && (62.0..bottom).contains(&y) {
                let u = x - tx;
                let stroke = ((1.0..4.0).contains(&u) && (y - 64.0).rem_euclid(4.0) < 1.0 && hash(tx, y.floor()) > 0.3)
                    || ((u - 2.0).abs() < 0.6 && hash(tx + 5.0, (y / 3.0).floor()) > 0.55 && y > 64.0);
                let c = if stroke { [0.12, 0.06, 0.05] } else { scale([0.92, 0.8, 0.58], l * 1.05) };
                return deep(c);
            }
        }
        // stacks of bowls behind the first customer
        for i in 0..3 {
            let sx = 64.0 + 9.0 * i as f64;
            let top = 93.0 + 2.0 * (i % 2) as f64;
            if x >= sx && x < sx + 7.0 && y >= top && y < COUNTER {
                let band = (y - top).rem_euclid(3.0) < 1.0;
                let c = if band { [0.2, 0.3, 0.65] } else { [0.85, 0.82, 0.76] };
                let side = if x < sx + 1.0 { 1.15 } else { 0.85 };
                return deep(scale(c, l * side));
            }
        }
        // the back wall: lit planks
        let plank = if (y - 57.0).rem_euclid(7.0) < 1.0 { 0.5 } else { 1.0 };
        let grain = 0.8 + 0.35 * fbm(x * 0.05, y * 0.6, 3);
        return deep(scale([0.46, 0.27, 0.15], l * plank * grain));
    }
    // outside the stall: the alley, or the wall and what's on it
    if in_alley(x, y) {
        return px(alley(x, y), 0.25);
    }
    if inside(x, y, &WIN) {
        let edge = on_edge(x, y, &WIN);
        return px(if edge { [0.03, 0.025, 0.04] } else { [0.08, 0.08, 0.16] }, 0.0);
    }
    if (6.0..34.0).contains(&x) && (26.0..28.0).contains(&y) {
        return px([0.16, 0.14, 0.18], 0.6);
    }
    // the air conditioner, its fan and a rust stain
    if (4.0..30.0).contains(&x) && (31.0..47.0).contains(&y) {
        let (dx, dy) = (x - 16.5, y - 39.0);
        let r = dx.hypot(dy);
        if r < 6.0 {
            let spoke = ((dy.atan2(dx) * 3.0 / PI).rem_euclid(1.0) < 0.2) || r < 1.5;
            return px(if spoke { [0.2, 0.2, 0.24] } else { [0.04, 0.04, 0.05] }, 0.3);
        }
        let top = if y < 32.0 { 1.5 } else { 1.0 };
        return px(scale([0.17, 0.17, 0.21], top * (0.85 + 0.2 * noise(x * 0.5, y * 0.5))), 0.6);
    }
    // the drain pipe
    if (12.0..15.0).contains(&x) && (47.0..GROUND).contains(&y) {
        return px(if x < 13.0 { [0.2, 0.18, 0.22] } else { [0.09, 0.08, 0.11] }, 0.6);
    }
    // a stack of beer crates and a cat sitting beside them
    if (2.0..24.0).contains(&x) && (112.0..GROUND).contains(&y) {
        let (u, v) = ((x - 2.0).rem_euclid(5.5), (y - 112.0).rem_euclid(11.0));
        let hole = u > 1.5 && u < 4.0 && v > 2.0 && v < 9.0;
        let c = if hole { [0.03, 0.01, 0.02] } else { [0.28, 0.06, 0.08] };
        return px(scale(c, 0.6 + 0.4 * smooth(GROUND, 112.0, y)), 0.3);
    }
    let cat = ((x - 31.0) / 5.0).powi(2) + ((y - 130.5) / 4.0).powi(2) <= 1.0
        || ((x - 31.0) / 3.2).powi(2) + ((y - 125.5) / 2.8).powi(2) <= 1.0
        || ((x - 28.8).abs() < 0.8 && (122.0..124.0).contains(&y))
        || ((x - 33.2).abs() < 0.8 && (122.0..124.0).contains(&y))
        || ((x - 36.5).abs() < 0.8 && (127.0..GROUND).contains(&y));
    if cat {
        return px([0.015, 0.012, 0.02], 0.0);
    }
    let mut c = wall(x, y);
    if y > 100.0 {
        c = scale(c, 0.7 + 0.3 * smooth(GROUND, 100.0, y));
    }
    px(c, 0.85)
}

/// The noodle bar: the static picture and how much neon each pixel catches,
/// the signs' tubes, the street's puddles, and a buffer to paint in.
pub struct NoodleBar {
    base: Vec<C>,
    resp: Vec<f32>,
    deep: Vec<bool>,
    /// The main sign's light, the hanging sign's and the bowl's, falling off.
    near_m: Vec<f32>,
    near_c: Vec<f32>,
    near_a: Vec<f32>,
    lamps: Vec<Lamp>,
    puddle: Vec<f32>,
    asphalt: Vec<C>,
    steam: Vec<f32>,
    buf: Vec<C>,
    pixels: Pixels,
}

/// A noodle bar; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let mut base = vec![[0.0; 3]; n];
    let mut resp = vec![0.0f32; n];
    let mut deep = vec![false; n];
    let mut near_m = vec![0.0f32; n];
    let mut near_c = vec![0.0f32; n];
    let mut near_a = vec![0.0f32; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
            near_m[k] = (if inside(x, y, &SIGN) { 0.3 } else { (-rect_d(x, y, &SIGN) / 26.0).exp() }) as f32;
            near_c[k] = (if inside(x, y, &VSIGN) { 0.3 } else { (-rect_d(x, y, &VSIGN) / 14.0).exp() }) as f32;
            near_a[k] = (-rect_d(x, y, &BOWL) / 13.0).exp() as f32;
            if py as f64 >= GROUND {
                continue;
            }
            let p = paint(x, y);
            base[k] = p.c;
            resp[k] = p.resp as f32;
            deep[k] = p.deep;
        }
    }
    // the tubes: ラーメン in magenta, its frame in cyan, the bowl in amber
    // with two puffs of steam, the hanging sign's frame and its four kanji
    let mut lamps = vec![];
    for (i, g) in ramen().iter().enumerate() {
        lamps.push(lamp(&place(g, 94.0 + 36.0 * i as f64, 9.0, 24.0), 1.15, MAGENTA));
    }
    let frame = |r: [f64; 4]| vec![[r[0], r[1], r[2], r[1]], [r[2], r[1], r[2], r[3]], [r[2], r[3], r[0], r[3]], [r[0], r[3], r[0], r[1]]];
    lamps.push(lamp(&frame([82.0, 6.0, 238.0, 36.0]), 0.7, CYAN));
    let mut bowl: Vec<[f64; 4]> =
        poly(&(0..=10).map(|i| (263.0 + 11.0 * (i as f64 * PI / 10.0).cos(), 21.0 + 10.0 * (i as f64 * PI / 10.0).sin())).collect::<Vec<_>>());
    bowl.push([251.0, 21.0, 275.0, 21.0]);
    bowl.push([258.0, 34.0, 268.0, 34.0]);
    bowl.push([265.0, 19.0, 279.0, 8.0]);
    bowl.push([267.0, 20.0, 280.0, 12.0]);
    lamps.push(lamp(&bowl, 0.8, AMBER));
    for sx in [256.0, 262.0] {
        lamps.push(lamp(&poly(&[(sx, 17.0), (sx + 1.5, 14.0), (sx - 1.0, 11.0), (sx + 1.0, 8.0), (sx, 5.0)]), 0.6, [1.0, 0.85, 0.55]));
    }
    lamps.push(lamp(&frame([22.5, 58.5, 37.5, 117.5]), 0.6, CYAN));
    for (i, g) in kanji().iter().enumerate() {
        lamps.push(lamp(&place(g, 24.0, 62.0 + 14.0 * i as f64, 12.0), 0.7, GREEN));
    }

    // the street: wet asphalt, puddles where it dips
    let mut puddle = vec![0.0f32; n];
    let mut asphalt = vec![[0.0; 3]; n];
    for py in GROUND as usize..H {
        for px in 0..W {
            let (x, y) = (px as f64, py as f64);
            let k = py * W + px;
            let d = fbm(x * 0.025, y * 0.11, 3);
            puddle[k] = smooth(0.43, 0.5, d) as f32;
            let g = 0.8 + 0.4 * hash(x, y) * 0.5 + 0.3 * fbm(x * 0.2, y * 0.5, 2);
            asphalt[k] = if py < GROUND as usize + 1 { [0.16, 0.12, 0.12] } else { [0.022 * g, 0.02 * g, 0.03 * g] };
        }
    }
    Box::new(NoodleBar {
        base,
        resp,
        deep,
        near_m,
        near_c,
        near_a,
        lamps,
        puddle,
        asphalt,
        steam: vec![0.0; n],
        buf: vec![[0.0; 3]; n],
        pixels: Pixels::new(&META),
    })
}

/// A customer's pose: where the head is, how big, and how far the right
/// elbow is lifted.
struct Sitter {
    cx: f64,
    hx: f64,
    hy: f64,
    hr: f64,
    lift: f64,
}

impl Sitter {
    /// Signed distance to the figure, and which part: 0 the head, 1 the
    /// body, 2 the right arm.
    fn sd(&self, x: f64, y: f64) -> (f64, u8) {
        let cx = self.cx;
        let head = (x - self.hx).hypot(y - self.hy) - self.hr;
        let neck = seg_d(x, y, &[self.hx, self.hy + 3.0, cx, 95.0]) - 3.4;
        let back = sd_ellipse(x - cx, y - 104.0, 14.5, 12.0);
        let seat = sd_box(x - cx, y - 111.0, 10.5, 5.5) - 2.0;
        let arm_l = seg_d(x, y, &[cx - 11.0, 97.0, cx - 17.0, 105.0]) - 3.4;
        let arm_r = seg_d(x, y, &[cx + 11.0, 97.0, cx + 17.0 + 2.0 * self.lift, 105.0 - 4.0 * self.lift]) - 3.4;
        let body = neck.min(back).min(seat).min(arm_l);
        if head < body && head < arm_r {
            (head, 0)
        } else if arm_r < body {
            (arm_r, 2)
        } else {
            (body, 1)
        }
    }
}

/// The cook's pose: the head, the stirring hand, the hand in the stockpot.
struct Cook {
    hx: f64,
    hy: f64,
    stir: (f64, f64),
    lift: (f64, f64),
    up: f64,
}

impl Cook {
    fn sd(&self, x: f64, y: f64) -> (f64, u8) {
        let head = (x - self.hx).hypot(y - self.hy) - 6.5;
        let neck = seg_d(x, y, &[self.hx, self.hy + 3.0, 204.0, 82.0]) - 4.2;
        let torso = sd_ellipse(x - 204.0, y - 87.5, 14.0, 6.5).min(sd_box(x - 204.0, y - 98.0, 9.0, 9.0) - 3.0);
        let (l_el, r_el) = ((186.0 + 2.0 * self.up, 94.0 - 12.0 * self.up), (222.0, 92.0));
        let arm_l = seg_d(x, y, &[194.0, 86.0, l_el.0, l_el.1]).min(seg_d(x, y, &[l_el.0, l_el.1, self.lift.0, self.lift.1])) - 2.8;
        let arm_r = seg_d(x, y, &[214.0, 86.0, r_el.0, r_el.1]).min(seg_d(x, y, &[r_el.0, r_el.1, self.stir.0, self.stir.1])) - 2.8;
        let ladle = seg_d(x, y, &[self.stir.0, self.stir.1, self.stir.0 + 1.0, 99.0]) - 0.6;
        let basket = if self.up > 0.15 { sd_box(x - self.lift.0, y - self.lift.1 - 4.5, 3.0, 3.0) - 0.5 } else { f64::MAX };
        let body = head.min(neck).min(torso);
        let arms = arm_l.min(arm_r);
        let mut best = (body, if head <= neck.min(torso) { 0 } else { 1 });
        if arms < best.0 {
            best = (arms, 2);
        }
        if ladle < best.0 {
            best = (ladle, 3);
        }
        if basket < best.0 {
            best = (basket, 4);
        }
        best
    }
}

impl NoodleBar {
    /// The rim of a silhouette lit from behind and above: 0 deep inside, up
    /// to 1 on the top edge.
    fn rim(sd: &dyn Fn(f64, f64) -> f64, x: f64, y: f64, d: f64) -> f64 {
        if d < -2.6 {
            return 0.0;
        }
        let top = if sd(x, y - 1.8) > 0.0 { 1.0 } else { 0.0 };
        let side = if sd(x - 1.8, y) > 0.0 || sd(x + 1.8, y) > 0.0 { 0.55 } else { 0.0 };
        f64::max(top, side)
    }

    fn put(&mut self, px: usize, py: usize, c: C, a: f64) {
        if px < W && py < H && a > 0.0 {
            let k = py * W + px;
            self.buf[k] = mixc(self.buf[k], c, a.min(1.0));
        }
    }

    fn draw_cook(&mut self, t: f64) {
        // lifting the noodles every eleven seconds, shaking them three times
        let p = t.rem_euclid(11.0);
        let up = pulse(p, 5.6, 9.2, 0.7);
        let shake = (p * TAU * 2.4).sin() * 1.6 * pulse(p, 6.3, 8.6, 0.3);
        let w = t * 2.2;
        let cook = Cook {
            hx: COOK.0 - 1.5 * up + 0.4 * (w * 0.5).sin(),
            hy: COOK.1 + 1.0 - up * 1.2 + 0.4 * (w * 1.0).sin(),
            stir: (226.0 + 2.5 * w.cos(), 91.5 + 0.9 * w.sin()),
            lift: (176.0 + 3.0 * up, 87.0 - 14.0 * up + shake),
            up,
        };
        let f = |x: f64, y: f64| cook.sd(x, y).0;
        for py in 56..COUNTER as usize {
            for px in 160..245 {
                let k = py * W + px;
                let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                let (d, part) = cook.sd(x, y);
                if d > 0.6 || (!self.deep[k] && (part < 2 || y < 60.0)) {
                    continue;
                }
                let a = clamp(0.6 - d);
                let mut c: C = match part {
                    0 => [0.07, 0.045, 0.045],
                    1 if y > 88.0 => [0.11, 0.1, 0.15],
                    1 => [0.05, 0.045, 0.07],
                    2 => [0.09, 0.055, 0.045],
                    3 => [0.45, 0.43, 0.45],
                    _ => [0.25, 0.22, 0.2],
                };
                if part == 0 {
                    let (dx, dy) = (x - cook.hx, y - cook.hy);
                    // the headband, knotted, and one red eye
                    if (-3.6..-1.6).contains(&dy) {
                        c = [0.78, 0.74, 0.68];
                    }
                    if (dx - 2.6).abs() < 1.1 && (dy - 0.6).abs() < 0.6 {
                        c = [1.0, 0.18, 0.1];
                    }
                }
                if part == 4 {
                    // noodles in the basket, dripping
                    if hash(x * 3.0, y.floor() + (t * 15.0).floor()) > 0.45 {
                        c = [0.95, 0.82, 0.48];
                    }
                }
                let r = Self::rim(&f, x, y, d);
                c = addc(c, WARM, 0.55 * r * bulb_light(x, y));
                self.buf[k] = mixc(self.buf[k], c, a);
            }
        }
        // the eye's glow
        let (ex, ey) = (cook.hx + 2.6, cook.hy + 0.6);
        for py in (ey as usize - 4)..(ey as usize + 5) {
            for px in (ex as usize - 4)..(ex as usize + 5) {
                let d = (px as f64 + 0.5 - ex).hypot(py as f64 + 0.5 - ey);
                let k = py * W + px;
                self.buf[k] = addc(self.buf[k], [1.0, 0.15, 0.1], 0.35 * (-d / 1.6).exp());
            }
        }
        // drips from the lifted basket
        if up > 0.3 {
            for i in 0..3 {
                let a = (t * 2.5 + i as f64 / 3.0).fract();
                let (dx, dy) = (cook.lift.0 - 2.0 + 2.0 * i as f64, cook.lift.1 + 9.0 + a * 10.0);
                if dy < 85.0 {
                    self.put(dx as usize, dy as usize, [0.95, 0.85, 0.6], 0.8 * up);
                }
            }
        }
    }

    /// Steam rising from a source, added into `self.steam`.
    fn plume(&mut self, t: f64, sx: f64, sy: f64, strength: f64, seed: f64) {
        let y0 = (sy - 130.0).max(0.0) as usize;
        let (x0, x1) = ((sx - 40.0).max(0.0) as usize, ((sx + 70.0) as usize).min(W));
        for py in y0..sy as usize {
            let y = py as f64 + 0.5;
            let h = sy - y;
            let fade = smooth(0.0, 6.0, h) * smooth(128.0, 55.0, h);
            let w = 2.5 + h * 0.3;
            for px in x0..x1 {
                let x = px as f64 + 0.5;
                let wob = (noise(x * 0.03 + seed, (y + t * 9.0) * 0.03) - 0.5) * (5.0 + h * 0.3);
                let cx = sx + h * 0.2 + wob;
                let env = (-((x - cx) / w).powi(2)).exp() * fade;
                if env < 0.02 {
                    continue;
                }
                let n = fbm(x * 0.07 + seed, (y + t * 15.0) * 0.06, 3);
                let a = (clamp((n - 0.3) * 3.2) + 0.3 * smooth(45.0, 0.0, h)) * env * strength;
                let k = py * W + px;
                self.steam[k] += a as f32;
            }
        }
    }

    fn draw_sitters(&mut self, t: f64) {
        // A, hooded, slurps every five seconds or so; B lifts the chopsticks
        let pa = t.rem_euclid(5.3);
        let sa = pulse(pa, 2.0, 3.6, 0.4);
        let pb = (t + 2.2).rem_euclid(7.1);
        let sb = pulse(pb, 1.0, 3.0, 0.5);
        let sitters = [
            Sitter { cx: A_X, hx: A_X - 0.5, hy: 89.0 + 1.6 * sa, hr: 7.6, lift: 0.3 * sa },
            Sitter { cx: B_X, hx: B_X + 0.5, hy: 88.5 - 0.8 * sb + 0.6 * (t * 0.7).sin() * 0.5, hr: 6.4, lift: sb },
        ];
        let visor = 0.75 + 0.25 * (t * 3.1).sin() * if hash((t * 15.0).floor(), 33.0) > 0.92 { 0.0 } else { 1.0 };
        for (i, s) in sitters.iter().enumerate() {
            let f = |x: f64, y: f64| s.sd(x, y).0;
            for py in 78..120 {
                for px in (s.cx as usize - 26)..(s.cx as usize + 26) {
                    let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                    let k = py * W + px;
                    let (d, part) = s.sd(x, y);
                    let (dx, dy) = (x - s.hx, y - s.hy);
                    if i == 0 {
                        // the visor's light spilling round the hood
                        let g = (-(dx.abs() - s.hr).abs().hypot((dy - 1.0) * 1.4) / 2.6).exp();
                        if d > 0.0 {
                            self.buf[k] = addc(self.buf[k], CYAN, 0.55 * g * visor);
                        }
                    }
                    if d > 0.6 {
                        continue;
                    }
                    let a = clamp(0.6 - d);
                    let mut c: C = match (i, part) {
                        (0, _) => [0.05, 0.05, 0.085],
                        (_, 0) => [0.09, 0.06, 0.06],
                        (_, 1) => [0.08, 0.04, 0.06],
                        _ => [0.09, 0.09, 0.12],
                    };
                    let r = Self::rim(&f, x, y, d);
                    c = addc(c, WARM, 0.5 * r * bulb_light(x, y.min(98.0)));
                    if i == 0 && part == 0 && (dy - 1.0).abs() < 1.0 {
                        // the visor band, brightest where it wraps to the front
                        let side = 0.3 + 0.7 * smooth(s.hr - 4.0, s.hr, dx.abs());
                        c = mixc(c, mixc(CYAN, [1.0, 1.0, 1.0], 0.4 * side), side * visor);
                    }
                    if i == 1 && part == 0 {
                        // circuits on the scalp, data running down them
                        let lines = [[0.5, -6.0, 3.5, -4.0], [3.5, -4.0, 4.5, 0.0], [4.5, 0.0, 3.0, 3.5], [-4.5, 2.5, -1.5, 2.5]];
                        let dl = lines.iter().map(|l| seg_d(dx, dy, l)).fold(f64::MAX, f64::min);
                        let flow = 0.55 + 0.45 * (t * 6.0 - dy * 0.9).sin();
                        c = addc(c, GREEN, (smooth(0.9, 0.2, dl) * 0.85 + 0.12 * (-dl / 1.2).exp()) * flow);
                    }
                    if i == 1 && part == 1 && (dy - 7.5).abs() < 0.6 && (dx.abs() < 2.5) && (dx + 2.5).rem_euclid(2.0) < 1.0 {
                        // the neck port's lights
                        let on = hash((t * 4.0).floor(), dx.floor() + 9.0) > 0.4;
                        c = if on { [0.7, 1.0, 0.3] } else { [0.1, 0.2, 0.06] };
                    }
                    if i == 1 && part == 2 {
                        // the chrome arm: a highlight on top, a green seam
                        let el = (B_X + 17.0 + 2.0 * s.lift, 105.0 - 4.0 * s.lift);
                        let seam = seg_d(x, y, &[B_X + 11.0, 97.0, el.0, el.1]);
                        c = addc(c, GREEN, 0.6 * smooth(0.5, 0.0, seam) * (0.6 + 0.4 * (t * 2.0).sin()));
                    }
                    self.buf[k] = mixc(self.buf[k], c, a);
                }
            }
        }
    }

    fn draw_passer(&mut self, t: f64) -> Option<(f64, C)> {
        let k = ((t + 1.67) / PASS).floor();
        let ph = (t + 1.67).rem_euclid(PASS);
        let left = k == 0.0 || hash(k, 41.0) > 0.5;
        let dir = if left { -1.0 } else { 1.0 };
        let travel = ph * 36.0;
        if travel > 390.0 {
            return None;
        }
        let xp = if left { 350.0 - travel } else { -30.0 + travel };
        let col = [CYAN, MAGENTA, GREEN, AMBER][(hash(k, 42.0) * 4.0) as usize % 4];
        let col = if k == 0.0 { CYAN } else { col };
        let step = travel / 11.0 * PI;
        let swing = step.sin() * 4.5;
        let bob = step.sin().abs() * 0.8;
        let ux = xp + 9.0 * dir;
        let hy = 103.0 - bob;
        let sd = |x: f64, y: f64| -> f64 {
            let head = (x - xp - 0.8 * dir).hypot(y - hy) - 5.4;
            let neck = seg_d(x, y, &[xp, hy + 3.0, xp, 113.0 - bob]) - 2.6;
            let shoulders = sd_ellipse(x - xp, y - (115.5 - bob), 8.5, 4.0);
            let coat = sd_box(x - xp, y - (126.0 - bob), 6.0 + (y - 116.0).max(0.0) * 0.08, 9.5) - 2.0;
            let leg_a = seg_d(x, y, &[xp - 2.5, 134.0 - bob, xp - 2.5 + swing, 157.0]) - 2.3;
            let leg_b = seg_d(x, y, &[xp + 2.5, 134.0 - bob, xp + 2.5 - swing, 157.0]) - 2.3;
            let arm = seg_d(x, y, &[xp + 6.0 * dir, 115.0 - bob, ux - dir, 121.0 - bob]).min(seg_d(x, y, &[ux - dir, 121.0 - bob, ux, 117.0 - bob])) - 2.2;
            head.min(neck).min(shoulders).min(coat).min(leg_a).min(leg_b).min(arm)
        };
        let (x0, x1) = ((xp - 40.0).max(0.0) as usize, ((xp + 41.0).max(0.0) as usize).min(W));
        for py in 74..H {
            for px in x0..x1 {
                let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                let k = py * W + px;
                let mut c = self.buf[k];
                let d = sd(x, y);
                if d < 0.6 {
                    let r = Self::rim(&sd, x, y, d);
                    let body = addc([0.035, 0.03, 0.05], col, 0.45 * r);
                    c = mixc(c, body, clamp(0.6 - d));
                }
                // the canopy: clear, its rim and ribs lit
                let (dx, dy) = (x - ux, 92.0 - y);
                let e = (dx / 26.0).powi(2) + (dy / 13.5).powi(2);
                if dy >= 0.0 && e <= 1.0 {
                    let ribs =
                        [0.0, 0.5, -0.5, 1.0, -1.0].iter().map(|&f| (dx - 26.0 * f * (dy / 13.5).mul_add(-1.0, 1.0).max(0.0)).abs()).fold(f64::MAX, f64::min);
                    let edge = smooth(0.84, 1.0, e) + if dy < 1.0 { 1.0 } else { 0.0 };
                    c = mixc(c, addc(scale(c, 0.6), col, 0.18), 0.6);
                    c = addc(c, col, 0.55 * edge.min(1.0) + 0.18 * smooth(1.0, 0.0, ribs));
                    // rain bursting on top
                    if e > 0.75 && hash(x * 7.0 + (t * 15.0).floor(), y) > 0.93 {
                        c = addc(c, [1.0, 1.0, 1.0], 0.5);
                    }
                }
                // the shaft, a tube of light
                let ds = seg_d(x, y, &[ux, 78.5, ux, 119.0 - bob]);
                c = mixc(c, mixc(col, [1.0, 1.0, 1.0], 0.6), smooth(1.2, 0.3, ds));
                c = addc(c, col, 0.5 * (-ds / 2.2).exp() + 0.12 * (-ds / 9.0).exp());
                self.buf[k] = c;
            }
        }
        Some((ux, col))
    }
}

impl Piece for NoodleBar {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let nf = (t * 15.0 + 1e-6).floor();

        // The signs. The big one stutters and shorts out every thirteen
        // seconds; its メ is loose; the bowl's steam puffs in turn.
        let ph = t.rem_euclid(13.0);
        let sign_on = if (8.0..8.5).contains(&ph) {
            f64::from(u8::from(hash(nf, 91.0) > 0.45))
        } else if (8.5..9.2).contains(&ph) {
            0.0
        } else if (9.2..9.6).contains(&ph) {
            if hash(nf, 92.0) > 0.5 {
                1.0
            } else {
                0.1
            }
        } else {
            1.0
        };
        let mut lv = [1.0f64; 13];
        for (g, l) in lv.iter_mut().enumerate().take(4) {
            let mut on = sign_on;
            if g == 2 && hash((t / 0.9).floor(), 77.0) < 0.32 && hash(nf, 78.0) < 0.55 {
                on = 0.0;
            }
            *l = on * (0.88 + 0.12 * hash(nf, 50.0 + g as f64));
        }
        lv[4] = (0.85 + 0.1 * (t * 1.7).sin()) * if hash(nf, 60.0) < 0.02 { 0.3 } else { 1.0 };
        lv[5] = 0.93 + 0.07 * hash(nf, 61.0);
        let puff = (t * 2.5).floor().rem_euclid(4.0);
        lv[6] = f64::from(u8::from(puff >= 1.0));
        lv[7] = f64::from(u8::from(puff >= 2.0));
        lv[8] = 0.9;
        lv[11] = if hash(nf, 70.0) < 0.08 { 0.0 } else { 1.0 };
        let m_level = (lv[0] + lv[1] + lv[2] + lv[3]) / 4.0;
        let c_level = 0.95;
        let a_level = lv[5];

        // 1. the stall, the walls and the alley, with the neon's spill
        for k in 0..GROUND as usize * W {
            let r = f64::from(self.resp[k]);
            let mut c = self.base[k];
            c = addc(c, MAGENTA, 0.36 * r * f64::from(self.near_m[k]) * m_level);
            c = addc(c, CYAN, 0.3 * r * f64::from(self.near_c[k]) * c_level);
            c = addc(c, AMBER, 0.3 * r * f64::from(self.near_a[k]) * a_level);
            self.buf[k] = c;
        }

        // 2. the small lives round the edges
        // the television behind the blinds
        let tv = 0.35 + 0.65 * hash((t * 3.0).floor(), 5.0);
        let tv_hue = mixc([0.3, 0.4, 0.9], [0.7, 0.4, 0.9], hash((t * 0.7).floor(), 6.0));
        for py in WIN[1] as usize + 1..WIN[3] as usize - 1 {
            for px in WIN[0] as usize + 1..WIN[2] as usize - 1 {
                let slat = (py - WIN[1] as usize).is_multiple_of(3);
                let v = if slat { 0.12 } else { 0.4 + 0.6 * (py as f64 - WIN[1]) / 20.0 };
                self.buf[py * W + px] = addc([0.05, 0.04, 0.08], tv_hue, 0.45 * v * tv);
            }
        }
        // the air conditioner's drip
        let da = (t / 1.6).fract() * 1.6;
        let dy = 47.0 + 80.0 * da * da;
        if dy < GROUND {
            self.put(24, dy as usize, [0.55, 0.55, 0.7], 0.8);
        }
        // the cat's eyes, blinking, now and then looking away
        let cp = t.rem_euclid(7.0);
        if !(3.0..3.2).contains(&cp) && hash((t / 7.0).floor(), 9.0) > 0.3 {
            for ex in [29usize, 32] {
                self.put(ex, 125, [0.75, 1.0, 0.3], 1.0);
                self.buf[124 * W + ex] = addc(self.buf[124 * W + ex], GREEN, 0.15);
            }
        }
        // down the alley: a searchlight in the smog, and a craft crossing
        let (ox, oy) = (300.0, 150.0);
        let th = -PI / 2.0 + 0.55 * (t * 0.33).sin();
        let (bx, by) = (th.cos(), th.sin());
        let craft = (330.0 - t.rem_euclid(17.0) * 14.0, 22.0 + (t * 1.3).sin());
        for py in 0..112 {
            for px in 270..W {
                let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                if !in_alley(x, y) {
                    continue;
                }
                let k = py * W + px;
                let (rx, ry) = (x - ox, y - oy);
                let along = rx * bx + ry * by;
                let perp = (rx * by - ry * bx).abs();
                if along > 0.0 {
                    let b = (-(perp / (2.0 + along * 0.07)).powi(2)).exp() * 0.16 * smooth(20.0, 60.0, along);
                    self.buf[k] = addc(self.buf[k], [0.75, 0.65, 0.95], b);
                }
                let (cx, cy) = (x - craft.0, y - craft.1);
                if sd_ellipse(cx, cy, 6.0, 1.6) < 0.0 {
                    self.buf[k] = [0.04, 0.03, 0.06];
                }
                if (cy - 2.2).abs() < 0.6 && cx.abs() < 4.0 {
                    self.buf[k] = mixc(CYAN, [1.0; 3], 0.3);
                }
                self.buf[k] = addc(self.buf[k], CYAN, 0.25 * (-(cx.hypot((cy - 2.5) * 2.0)) / 3.0).exp());
                let blink = (t * 1.5).fract() < 0.25;
                if blink && cx.hypot(cy) < 8.0 && ((cx + 5.0).abs() < 0.7 && cy.abs() < 0.7) {
                    self.buf[k] = [1.0, 0.25, 0.2];
                }
                if (cx - 5.0).abs() < 0.7 && cy.abs() < 0.7 && (t * 1.5 + 0.5).fract() < 0.25 {
                    self.buf[k] = [1.0, 1.0, 1.0];
                }
            }
        }
        // the hologram menu: scanlines, a sweeping band, a glitch now and then
        let gw = (t / 4.3).floor();
        let glitch = hash(gw, 31.0) < 0.6 && (t / 4.3).fract() * 4.3 < 0.4;
        let holo_a = 0.5 + 0.1 * (t * 3.0).sin() - if glitch { 0.15 } else { 0.0 };
        let sweep = HOLO[1] + (t * 9.0).rem_euclid(30.0);
        for py in HOLO[1] as usize..HOLO[3] as usize {
            let band = (py as f64 / 3.0).floor();
            let off = if glitch { ((hash(nf, band) - 0.5) * 8.0).round() } else { 0.0 };
            for px in HOLO[0] as usize..HOLO[2] as usize {
                let (x, y) = (px as f64 - off - HOLO[0], py as f64 - HOLO[1]);
                let (w, h) = (HOLO[2] - HOLO[0], HOLO[3] - HOLO[1]);
                let border = x < 1.0 || x >= w - 1.0 || y < 1.0 || y >= h - 1.0;
                let title = (2.0..5.0).contains(&y) && (2.0..w - 2.0).contains(&x);
                let row = ((y - 7.0) / 3.0).floor();
                let line = y >= 7.0 && (y - 7.0).rem_euclid(3.0) < 1.0 && row < 5.0 && {
                    let len = 10.0 + 12.0 * hash(row, 3.0);
                    (3.0..3.0 + len).contains(&x) || (w - 8.0..w - 3.0).contains(&x)
                };
                let inb = (0.0..w).contains(&x);
                let mut v = if !inb {
                    0.0
                } else if border || line {
                    1.0
                } else if title {
                    0.75
                } else {
                    0.18
                };
                if py % 2 == 1 {
                    v *= 0.55;
                }
                v += 0.35 * (-(py as f64 - sweep).abs() / 1.5).exp() * f64::from(u8::from(inb));
                let k = py * W + px;
                let c = if glitch && hash(band, 7.0) > 0.6 { MAGENTA } else { CYAN };
                self.buf[k] = addc(scale(self.buf[k], 1.0 - 0.3 * holo_a), c, v * holo_a * 0.8);
            }
        }

        // 3. the cook, behind the pots
        self.draw_cook(t);

        // 4. the tubes
        let lamps = std::mem::take(&mut self.lamps);
        for (i, l) in lamps.iter().enumerate() {
            let lev = lv[i];
            let hot = mixc(l.col, [1.0, 1.0, 1.0], 0.55);
            let glass = scale(l.col, 0.16);
            let tube = mixc(glass, hot, lev);
            for ly in 0..l.h {
                for lx in 0..l.w {
                    let j = ly * l.w + lx;
                    let k = (l.y0 + ly) * W + l.x0 + lx;
                    let (core, glow) = (f64::from(l.core[j]), f64::from(l.glow[j]));
                    let mut c = self.buf[k];
                    if core > 0.0 {
                        c = mixc(c, tube, core);
                    }
                    self.buf[k] = addc(c, l.col, glow * lev);
                }
            }
        }
        self.lamps = lamps;

        // 5. the steam, out of the pots and the bowls, up into the sign's light
        self.steam.iter_mut().for_each(|s| *s = 0.0);
        let lift = pulse(t.rem_euclid(11.0), 5.8, 10.0, 1.0);
        self.plume(t, 176.0, 85.0, 1.25 + 0.9 * lift, 0.0);
        self.plume(t, 226.0, 95.0, 0.55, 40.0);
        self.plume(t, A_X + 2.0, 100.0, 0.3, 80.0);
        self.plume(t, B_X + 2.0, 100.0, 0.3, 120.0);
        for k in 0..GROUND as usize * W {
            let s = f64::from(self.steam[k]);
            if s <= 0.004 {
                continue;
            }
            let (x, y) = ((k % W) as f64, (k / W) as f64);
            let under = (AWN_L..AWN_R).contains(&x) && y > hem(x);
            let lit = if under {
                scale([0.86, 0.8, 0.74], 0.6 + 0.35 * bulb_light(x, y))
            } else {
                let m = f64::from(self.near_m[k]) * m_level;
                addc(addc([0.3, 0.27, 0.34], MAGENTA, 0.5 * m), CYAN, 0.12 * f64::from(self.near_c[k]))
            };
            let a = s.min(1.0) * if under { 0.85 } else { 0.6 };
            self.buf[k] = addc(mixc(self.buf[k], lit, a), lit, 0.15 * a);
        }
        // 6. the customers, in front of the counter
        self.draw_sitters(t);

        // 7. the wet street: the stall upside down in the puddles, rippling
        let g = GROUND as usize;
        let rings: Vec<(f64, f64, f64, f64)> = (0..30)
            .filter_map(|i| {
                let fi = i as f64;
                let per = 0.8 + 0.7 * hash(fi, 1.0);
                let tt = t + per * hash(fi, 2.0);
                let c = (tt / per).floor();
                let age = (tt / per).fract();
                let (rx, ry) = (hash(fi, c * 7.0 + 3.0) * W as f64, GROUND + 3.0 + hash(fi, c * 7.0 + 4.0) * 23.0);
                (age < 1.0).then_some((rx, ry, 1.0 + age * 9.0, 1.0 - age))
            })
            .collect();
        for py in g..H {
            let d = py as f64 - GROUND + 0.5;
            for px in 0..W {
                let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                let k = py * W + px;
                let p = f64::from(self.puddle[k]);
                let rx = x + (noise(x * 0.12, y * 0.8 + t * 2.5) - 0.5) * (1.0 + 3.0 * p);
                let ry = GROUND - d * (1.0 + 0.155 * d) + (noise(x * 0.25 + 7.0, y * 0.6 - t * 1.7) - 0.5) * 3.0;
                let sx = (rx.max(0.0) as usize).min(W - 1);
                let sample = |yy: f64| self.buf[(yy.clamp(0.0, GROUND - 1.0) as usize) * W + sx];
                let st = 1.0 + 0.15 * d;
                let refl = scale(addc(addc(sample(ry), sample(ry - st), 1.0), sample(ry - 2.0 * st), 1.0), 1.0 / 3.0);
                let gloss = 0.25 + 0.7 * p;
                let mut c = addc(scale(self.asphalt[k], 1.0 - 0.5 * p), refl, gloss);
                for &(cx, cy, r, a) in &rings {
                    let e = (x - cx).hypot((y - cy) * 3.2);
                    if (e - r).abs() < 0.8 {
                        c = addc(c, [0.5, 0.5, 0.6], 0.25 * a * (0.3 + 0.7 * p));
                    }
                }
                if py == g {
                    c = mixc(c, self.asphalt[k], 0.6);
                }
                self.buf[k] = c;
            }
        }

        // 8. someone crossing with a lit umbrella, and its light on the wet ground
        let passer = self.draw_passer(t);
        if let Some((ux, col)) = passer {
            for py in 140..H {
                for px in (ux - 40.0).max(0.0) as usize..((ux + 40.0).max(0.0) as usize).min(W) {
                    let (x, y) = (px as f64 + 0.5, py as f64 + 0.5);
                    let k = py * W + px;
                    let p = f64::from(self.puddle[k]);
                    let pool = (-((x - ux) / 16.0).powi(2) - ((y - 156.0) / 5.0).powi(2)).exp();
                    let streak = (-((x - ux) / 2.0).powi(2)).exp() * smooth(150.0, 158.0, y) * p;
                    self.buf[k] = addc(self.buf[k], col, 0.22 * pool * (0.4 + 0.6 * p) + 0.5 * streak);
                }
            }
        }

        // 9. the rain: three depths, thinner under the awning, none under the
        // umbrella; the curtain off the awning's edge; splashes
        let layers = [(170.0, 4.0, 30.0, 0.2, 0.22), (235.0, 7.0, 46.0, 0.2, 0.38), (310.0, 11.0, 80.0, 0.12, 0.55)];
        for py in 0..H {
            let y = py as f64 + 0.5;
            for px in 0..W {
                let x = px as f64 + 0.5;
                if let Some((ux, _)) = passer {
                    if (x - ux).abs() < 25.0 && y > 92.0 {
                        continue;
                    }
                }
                let sheltered = (AWN_L..AWN_R).contains(&x) && y > hem(x) && y < GROUND;
                let mut a = 0.0f64;
                for (li, &(v, len, period, dens, bright)) in layers.iter().enumerate() {
                    if sheltered && li < 2 {
                        continue;
                    }
                    let lf = li as f64;
                    let col = (x - 0.13 * y + lf * 1000.0).floor();
                    let speed = v * (0.85 + 0.3 * hash(col, lf + 9.0));
                    let s = y - t * speed + hash(col, lf) * period;
                    let pos = s.rem_euclid(period);
                    let drop = (s / period).floor();
                    if pos < len && hash(col, drop + lf * 1000.0) < dens * if sheltered { 0.5 } else { 1.0 } {
                        a = a.max(bright * (0.4 + 0.6 * (pos / len)));
                    }
                }
                if a > 0.0 {
                    let k = py * W + px;
                    let b = self.buf[k];
                    let lit = [b[0] * 1.7 + 0.11, b[1] * 1.7 + 0.12, b[2] * 1.7 + 0.17];
                    self.buf[k] = mixc(b, lit, a);
                }
            }
        }
        // the curtain of drips off each scallop
        for i in 0..19 {
            let fi = i as f64;
            let xd = AWN_L + 6.0 + 12.0 * fi;
            if xd >= AWN_R {
                break;
            }
            let heavy = hash(fi, 3.0) < 0.4;
            let period = if heavy { 8.0 + 6.0 * hash(fi, 4.0) } else { 18.0 + 30.0 * hash(fi, 4.0) };
            let speed = 200.0 * (0.9 + 0.2 * hash(fi, 5.0));
            for py in 59..GROUND as usize - 1 {
                let y = py as f64;
                let s = (y - t * speed + hash(fi, 6.0) * period).rem_euclid(period);
                let a = if s < 3.0 {
                    0.6
                } else if heavy {
                    0.1
                } else {
                    0.0
                };
                if a > 0.0 {
                    let k = py * W + xd as usize;
                    let b = self.buf[k];
                    self.buf[k] = mixc(b, [b[0] * 1.5 + 0.2, b[1] * 1.5 + 0.18, b[2] * 1.5 + 0.22], a);
                }
            }
            if hash(nf, fi + 200.0) < 0.7 {
                let o = (hash(nf, fi + 300.0) * 5.0) as usize;
                let (sx, sy) = (xd as usize + o - 2, GROUND as usize - 1 - (o % 2));
                self.put(sx, sy, [0.7, 0.68, 0.75], 0.6);
            }
        }
        // drops splashing on the street
        for i in 0..36 {
            let fi = i as f64;
            let (sx, sy) = ((hash(nf, fi + 400.0) * W as f64) as usize, GROUND as usize + 2 + (hash(nf, fi + 500.0) * 24.0) as usize);
            let k = sy.min(H - 1) * W + sx;
            let b = self.buf[k];
            let c = [b[0] * 1.6 + 0.2, b[1] * 1.6 + 0.2, b[2] * 1.6 + 0.25];
            self.put(sx, sy, c, 0.7);
            if sx > 0 && sx + 1 < W && sy > 0 {
                self.put(sx - 1, sy - 1, c, 0.35);
                self.put(sx + 1, sy - 1, c, 0.35);
            }
        }
        for (k, c) in self.buf.iter().enumerate() {
            self.pixels.set(k % W, k / W, [c[0] as f32, c[1] as f32, c[2] as f32]);
        }
        self.pixels.draw(grid, env);
    }
}
