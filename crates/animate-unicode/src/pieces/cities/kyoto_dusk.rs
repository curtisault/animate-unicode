//! kyoto dusk: a five-storey pagoda dark against an indigo to rose sky with a
//! thin crescent moon, a temple pond holding its reflection, and in front a
//! cherry tree in full bloom lit from below by a stone lantern. Petals fall
//! and drift through the frame, the lantern flickers, the pond shivers. After
//! ascii.rest's kyoto-dusk.ts (MIT, @bas3line).
//!
//! The pagoda, the hills and town, the cherry tree, the lantern, the clouds
//! and the pond, and how each is shaded, are ascii.rest's, shaded here per
//! pixel instead of per cell. Where ascii.rest draws each of 200 × 100 square
//! cells as a dot as big as it is bright, this draws 320 × 80 cells of two
//! square pixels each, an upper half block over a back colour
//! ([`crate::pixels`]): 1.6 times the detail each way, solid colour, and
//! gradients dithered in colour rather than in dot size. The palette adds the
//! dark tones a solid pixel needs (timber, bark, hills and stone in shadow)
//! where a small dot was enough. In one ink each pixel is octant dots, as many
//! as it is bright. The petals are a pixel or two now, wider as they turn
//! face-on, and the crescent moon's edges are smoothed.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Kyoto dusk's meta.
pub static META: Meta = Meta {
    name: "kyoto dusk",
    slug: "kyoto-dusk",
    category: Category::Cities,
    note: "a pagoda at dusk behind a cherry tree lit by a stone lantern",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: the sky from indigo through violet to rose and peach, the
    // blossom's pinks, the lantern's ambers, the timber, the moss and stone.
    // Then the dusk's darks.
    palette: Some(&[
        "#1c1d4a", "#262759", "#33316b", "#433d7c", "#574a8c", "#6e5a9a", "#8a5a92", "#a8668f", "#c4748f", "#db8a95", "#ec9f9c", "#f6b8a8", "#fbd0b8",
        "#fff1d8", "#fffaf0", "#7e5f9e", "#f9d3dc", "#f2b2c4", "#e28fab", "#c66e92", "#9c5279", "#74405f", "#ffd9a0", "#ffbb6a", "#f29a4a", "#d0763a",
        "#a35a35", "#2a2240", "#3a2c50", "#4a3860", "#5b4f86", "#7a6aa0", "#2c3a3a", "#3d4c46", "#56604f", "#9a6aa0", "#b47ea6", "#4a3038", "#6a4448",
        "#2b3a6a", "#3e4f86", "#0b0a16", "#0d0912", "#120d1f", "#140f24", "#1a1230", "#22183a", "#13133a", "#151519", "#1c1c22", "#0d0a14", "#6f1a41",
        "#992e5f", "#bd4976", "#7e4f37", "#a1777a", "#3b231c",
    ]),
    ground: Some("#0b0a16"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame, in its cells.
const TW: f64 = 200.0;
const TH: f64 = 100.0;
/// The far bank of the pond, where the pagoda stands.
const SHORE: f64 = 80.0;
/// The pagoda's axis.
const PX: f64 = 141.0;
const MOON: (f64, f64) = (176.0, 17.0);
/// The lantern's axis, foot and scale.
const LX: f64 = 53.0;
const LB: f64 = 98.0;
const LS: f64 = 1.8;
/// Its lit opening.
const LAMP: (f64, f64) = (LX, LB - (LB - 83.0) * LS);
/// The cloud streaks: the deck's width in columns (it wraps, so it can drift
/// forever), and the rows it spans.
const CW: usize = 500;
const C0: f64 = 34.0;
const C1: f64 = 68.0;
const GROUND: [f64; 3] = [0.043, 0.039, 0.086];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Hill,
    Town,
    Pagoda,
    Pond,
    Bank,
    Tree,
    Bloom,
    Stone,
    Flame,
}

/// The middle of pixel column `px`, in ascii.rest's columns (cell `x` spans x to x + 1).
fn xc(px: usize) -> f64 {
    (px as f64 + 0.5) / S
}

/// The middle of pixel row `py`, in ascii.rest's rows (its `y = r + 0.5`).
fn yc(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The pixel rows whose middles lie in [a, b), in ascii.rest's rows.
fn ys(a: f64, b: f64) -> std::ops::Range<usize> {
    let p = |v: f64| (v * S - 0.5).ceil().clamp(0.0, H as f64) as usize;
    p(a)..p(b)
}

/// The pixel nearest a point given as ascii.rest's cell indices (a cell
/// `x`'s middle is at x + 0.5), if it is on the picture.
fn pixel_at(x: f64, r: f64) -> Option<(usize, usize)> {
    let (px, py) = (((x + 0.5) * S - 0.5).round(), ((r + 0.5) * S - 0.5).round());
    (px >= 0.0 && px < W as f64 && py >= 0.0 && py < H as f64).then_some((px as usize, py as usize))
}

/// The distance from a point to a segment, and how far along it the nearest point is.
fn seg(px: f64, py: f64, ax: f64, ay: f64, bx: f64, by: f64) -> (f64, f64) {
    let (dx, dy) = (bx - ax, by - ay);
    let k = clamp(((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy));
    ((ax + dx * k - px).hypot(ay + dy * k - py), k)
}

/// Where each roof's eave sits, each roof's half-width, and each storey's.
const EAVE: [f64; 5] = [68.5, 58.0, 48.5, 40.0, 32.5];
const ROOF: [f64; 5] = [14.5, 13.4, 12.3, 11.2, 10.1];
const BODY: [f64; 5] = [6.4, 5.9, 5.4, 4.9, 4.4];

/// The pagoda about its axis: a shade code, or 0 for air. 1 body, 2 roof top,
/// 3 roof underside, 4 roof rim, 5 spire, 6 lit doorway.
fn pagoda(dx: f64, y: f64) -> u8 {
    let ax = dx.abs();
    // stone base
    if (76.0..SHORE + 0.5).contains(&y) {
        return u8::from(ax <= 9.5 - if y < 77.0 { 1.0 } else { 0.0 });
    }
    for i in 0..5 {
        let (e, rw) = (EAVE[i], ROOF[i]);
        // a roof: thin at its upturned tips, rising in a shallow concave curve to the body
        let u = ax / rw;
        if u <= 1.0 {
            let lift = 3.5 * u.powi(4);
            let bottom = e - lift + 0.6;
            let top = e - lift - 1.2 - 3.6 * (1.0 - u).powf(1.7);
            if y >= top && y < bottom {
                return if y < top + 0.9 {
                    4
                } else if y > bottom - 1.0 {
                    3
                } else {
                    2
                };
            }
        }
        // the storey beneath this roof, up from the roof below it (or the base)
        let floor = if i == 0 { 76.0 } else { EAVE[i - 1] - 3.6 };
        if y >= e + 0.6 && y < floor && ax <= BODY[i] {
            if i == 0 && ax <= 1.0 && y > 72.0 && y < 75.5 {
                return 6;
            }
            // a railed balcony under each roof
            return if y < e + 2.0 && ax <= BODY[i] + 1.2 { 3 } else { 1 };
        }
        if i > 0 && y >= e + 0.6 && y < e + 2.0 && ax <= BODY[i] + 1.2 {
            return 3;
        }
    }
    // the spire: a mast with nine rings and a flame-shaped finial
    let top = EAVE[4] - 1.2 - 3.6 - 0.2;
    if y < top && y >= 9.0 {
        if y >= top - 1.5 {
            return if ax <= 2.4 { 3 } else { 0 }; // the roof box
        }
        if y >= 15.0 {
            let ring = ((y - 15.0) / 1.2).floor() as i64 & 1 == 1;
            return if ax <= if ring { 1.4 } else { 0.55 } { 5 } else { 0 };
        }
        if y >= 12.0 {
            return if ax <= 1.3 - (y - 12.0) * 0.2 { 5 } else { 0 };
        }
        return if ax <= 0.6 { 5 } else { 0 };
    }
    0
}

/// The stone lantern in its plan: 0 air, 1 stone, 2 the lit opening, 3 roof and finial.
fn lantern(dx: f64, y: f64) -> u8 {
    let ax = dx.abs();
    let on = |b: bool, v: u8| if b { v } else { 0 };
    if (93.5..LB).contains(&y) {
        return on(ax <= 4.2 - if y < 94.5 { 0.8 } else { 0.0 }, 1); // foot
    }
    if (87.0..93.5).contains(&y) {
        return on(ax <= 1.4, 1); // post
    }
    if (85.5..87.0).contains(&y) {
        return on(ax <= 3.6 - if y < 86.2 { 0.6 } else { 0.0 }, 1); // platform
    }
    if (80.0..85.5).contains(&y) {
        if ax <= 1.7 && (80.8..84.8).contains(&y) {
            return 2; // lit opening
        }
        return on(ax <= 2.9, 1);
    }
    if (76.5..80.0).contains(&y) {
        // the roof, flaring out with upturned corners
        let u = (80.0 - y) / 3.5;
        let hw = 5.6 - 4.1 * u.powf(0.8) + if y > 79.2 { 0.6 } else { 0.0 };
        return on(ax <= hw, 3);
    }
    if (73.5..76.5).contains(&y) {
        return on(ax <= 1.3 - (y - 75.0).abs() * 0.25 || ax <= 0.5, 3); // finial
    }
    0
}

/// The lantern drawn a size up from its plan, standing on the bank.
fn lantern_at(xc: f64, y: f64) -> u8 {
    lantern((xc - LX) / LS, LB - (LB - y) / LS)
}

/// The sky at (x, y): indigo overhead, through violet, to rose and peach low
/// in the west (left), and the moon's halo.
fn sky_at(x: f64, y: f64) -> [f64; 3] {
    let v = clamp(y / SHORE);
    let west = (-(x - 80.0).abs() / 120.0).exp();
    let a = smooth(0.0, 0.5, v);
    let (mut r, mut g, mut b) = (mix(0.07, 0.22, a), mix(0.07, 0.14, a), mix(0.25, 0.38, a));
    let a = smooth(0.42, 0.86, v);
    (r, g, b) = (mix(r, 0.52, a), mix(g, 0.32, a), mix(b, 0.58, a));
    let a = smooth(0.78, 0.98, v) * (0.55 + 0.45 * west);
    (r, g, b) = (mix(r, 1.0, a), mix(g, 0.62, a), mix(b, 0.48, a));
    let band = (-(y - 74.0).abs() / 4.0).exp() * west * 0.18;
    (r, g, b) = (r + band, g + band * 0.66, b + band * 0.45);
    // a faint unevenness, so no stretch of sky is one flat tone
    let veil = 0.86 + 0.28 * fbm(x * 0.035, y * 0.09, 3);
    let d = (x - MOON.0).hypot(y - MOON.1);
    let halo = (-d / 5.0).exp() * 0.24 + (-d / 16.0).exp() * 0.07;
    [r * veil + halo * 0.75, g * veil + halo * 0.72, b * veil + halo]
}

/// The Higashiyama hills: low in the west where the glow is, rising behind the pagoda and the town.
fn hill_a(x: f64) -> f64 {
    77.5 - (3.0 + 13.0 * smooth(70.0, 175.0, x)) * (0.3 + 1.1 * fbm(x * 0.022 + 3.0, 1.0, 4))
}

/// The nearer ridge.
fn hill_b(x: f64) -> f64 {
    77.0 - 2.5 * fbm(x * 0.04 + 9.0, 2.0, 4)
}

/// Low tiled roofs along the far bank: a hipped roof, a short level ridge,
/// sloping ends, a gap between houses.
fn town(x: f64) -> f64 {
    let i = ((x + 3.0) / 11.0).floor();
    let f = (x + 3.0) / 11.0 - i;
    let h = 2.0 + hash(i, 5.0) * 2.5;
    let e = (f - 0.5).abs() * 2.0;
    if e > 0.86 {
        SHORE
    } else {
        SHORE - 1.0 - h + (e - 0.35).max(0.0) * 6.0
    }
}

/// Where the near bank meets the pond, by row.
fn bank_x(y: f64) -> f64 {
    38.0 + (y - SHORE) * 2.3 + 4.0 * fbm(y * 0.2, 4.0, 2)
}

/// ascii.rest's tone curve: a soft shoulder above 0.75, so bright light doesn't clip flat.
fn tone(v: f64) -> f64 {
    if v < 0.75 {
        v.max(0.0)
    } else {
        0.75 + 0.25 * (1.0 - (-(v - 0.75) * 4.0).exp())
    }
}

/// A falling petal: start, drift and fall speeds, sway, sway frequency, phase, tumble.
struct Petal {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    sw: f64,
    sf: f64,
    ph: f64,
    tum: f64,
}

/// The garden: what each pixel is, its colour, the reflection's source, the
/// lantern's light, the cloud streaks, the stars and the petals.
pub struct KyotoDusk {
    mat: Vec<Mat>,
    col: Vec<[f32; 3]>,
    /// The far side before the tree covers it, for the pond to mirror.
    src: Vec<[f32; 3]>,
    warmth: Vec<f32>,
    glow: Vec<f32>,
    /// The pixel rows the cloud deck spans: the first, and how many.
    c_py: usize,
    c_rows: usize,
    ccov: Vec<f32>,
    clit: Vec<f32>,
    /// The deepest pixel row of the far side, for the pond to mirror.
    shore_py: usize,
    /// Stars: pixel, phase, speed.
    stars: Vec<(usize, f64, f64)>,
    petals: Vec<Petal>,
    /// Petals afloat on the pond: x, row, speed.
    floaters: Vec<[f64; 3]>,
    pixels: Pixels,
}

/// Kyoto dusk; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;

    // the cherry tree's skeleton: trunk, limbs, and sprays drooping low over the lantern
    let mut branches: Vec<[f64; 6]> = vec![
        [16.0, 101.0, 20.0, 86.0, 5.2, 4.4],
        [20.0, 86.0, 27.0, 70.0, 4.4, 3.6],
        [27.0, 70.0, 31.0, 58.0, 3.6, 3.0],
        [31.0, 58.0, 50.0, 46.0, 3.0, 2.1],
        [50.0, 46.0, 74.0, 37.0, 2.1, 1.4],
        [74.0, 37.0, 98.0, 31.0, 1.4, 0.9],
        [98.0, 31.0, 116.0, 30.0, 0.9, 0.5],
        [31.0, 58.0, 24.0, 40.0, 2.6, 1.8],
        [24.0, 40.0, 12.0, 26.0, 1.8, 1.1],
        [12.0, 26.0, 0.0, 18.0, 1.1, 0.7],
        [29.0, 50.0, 42.0, 30.0, 2.0, 1.3],
        [42.0, 30.0, 52.0, 19.0, 1.3, 0.8],
        [52.0, 19.0, 70.0, 13.0, 0.8, 0.5],
        [50.0, 46.0, 60.0, 52.0, 1.2, 0.7],
        [74.0, 37.0, 86.0, 44.0, 1.0, 0.5],
        [42.0, 30.0, 66.0, 24.0, 1.0, 0.6],
        [66.0, 24.0, 90.0, 21.0, 0.6, 0.4],
        [24.0, 40.0, 34.0, 25.0, 1.0, 0.6],
        [12.0, 26.0, 6.0, 16.0, 0.7, 0.4],
        [98.0, 31.0, 104.0, 40.0, 0.6, 0.35],
        [40.0, 51.0, 44.0, 58.0, 1.1, 0.4],
        [58.0, 43.0, 62.0, 55.0, 0.9, 0.35],
        [70.0, 38.0, 76.0, 47.0, 0.7, 0.3],
    ];
    // short twigs off every limb, reaching up and out
    let limbs = branches.len();
    for b in 3..limbs {
        let [ax, ay, bx, by, w0, w1] = branches[b];
        let ang = (by - ay).atan2(bx - ax);
        for j in 0..3 {
            let (bf, jf) = (b as f64, f64::from(j));
            let k = 0.25 + 0.65 * hash(bf * 7.0 + jf, 31.0);
            let (sx, sy) = (mix(ax, bx, k), mix(ay, by, k));
            let turn = if j & 1 == 1 { 1.0 } else { -1.0 } * (0.5 + 0.6 * hash(bf, jf + 40.0));
            let len = 4.0 + 7.0 * hash(bf + jf, 41.0);
            let a2 = ang + turn - 0.25;
            branches.push([sx, sy, sx + a2.cos() * len, sy + a2.sin() * len, mix(w0, w1, k) * 0.55, 0.3]);
        }
    }
    // blossom clumps all along the limbs and twigs, thickest at their ends
    let mut clusters: Vec<(f64, f64, f64)> = vec![];
    for (b, &[ax, ay, bx, by, w0, _]) in branches.iter().enumerate().skip(3) {
        let len = (bx - ax).hypot(by - ay);
        let nn = 1.0 + (len / 1.7).round();
        let bf = b as f64;
        for i in 0..=nn as usize {
            let i = i as f64;
            let k = i / nn;
            if k < 0.3 && w0 > 2.0 {
                continue;
            }
            let (h1, h2, h3) = (hash(bf * 13.0 + i, 7.0), hash(bf * 5.0 + i, 11.0), hash(bf * 3.0 + i, 17.0));
            let (cx, cy) = (mix(ax, bx, k) + (h1 - 0.5) * 6.0, mix(ay, by, k) + (h2 - 0.5) * 4.5 - 1.0);
            // only the drooping sprays (and their twigs) hang below the crown
            let spray = (b >= limbs - 3 && b < limbs) || b >= limbs + 3 * (limbs - 6);
            // the crown is domed: thinner toward its top, never cut by the frame
            let dome = 9.0 + 10.0 * clamp((cx - 52.0).abs() / 70.0).powf(1.6);
            if cx > 122.0 || cy < dome || cy > if spray { 55.0 } else { 47.0 } {
                continue;
            }
            clusters.push((cx, cy, 1.4 + 1.6 * h3 + 0.8 * k));
        }
    }

    // build the static picture
    let mut mat = vec![Mat::Sky; n];
    let mut col = vec![[0.0f64; 3]; n];
    let mut warmth = vec![0.0f32; n];
    for py in 0..H {
        let y = yc(py);
        for px in 0..W {
            let k = py * W + px;
            let x = xc(px);
            let (mut m, mut c, mut warm) = (Mat::Sky, [0.0; 3], 0.0);
            if y < SHORE {
                c = sky_at(x, y);
                // distant hills in haze, then a nearer ridge
                let ha = hill_a(x);
                if y >= ha {
                    // the Higashiyama hills, flat and violet in the haze: wooded
                    // slopes, clumps of trees in the haze
                    m = Mat::Hill;
                    let s = sky_at(x, ha);
                    let d = smooth(ha, ha + 8.0, y);
                    let tex = fbm(x * 0.3, y * 0.45, 3);
                    let a = 0.6 + 0.2 * d + 0.25 * (tex - 0.5);
                    c = [mix(s[0], 0.14, a), mix(s[1], 0.08, a), mix(s[2], 0.24, a)];
                    // the ridge line catches the afterglow
                    let lip = smooth(ha + 1.4, ha, y) * 0.3;
                    c = [mix(c[0], s[0] * 1.1, lip), mix(c[1], s[1] * 1.05, lip), mix(c[2], s[2], lip)];
                }
                let hb = hill_b(x);
                if y >= hb {
                    m = Mat::Hill;
                    let tex = fbm(x * 0.3, y * 0.3, 3);
                    // mist settling along the water
                    let mist = smooth(hb + 1.0, SHORE, y) * 0.5;
                    c = [mix(0.1 + 0.05 * tex, 0.7, mist), mix(0.07 + 0.03 * tex, 0.44, mist), mix(0.17 + 0.05 * tex, 0.58, mist)];
                }
                let tn = town(x);
                if y >= tn && x > 70.0 {
                    m = Mat::Town;
                    c = [0.08, 0.06, 0.14];
                    // the roof ridges catch the sky
                    let tf = (((x + 3.0) / 11.0) % 1.0 - 0.5).abs() * 2.0;
                    if y < tn + 0.9 && tf < 0.45 {
                        c = [0.5, 0.31, 0.44];
                    }
                    // a few lit shoji under the eaves
                    if y > SHORE - 1.6 && y < SHORE - 0.5 && x % 4.0 < 1.2 && hash((x / 4.0).floor(), 7.0) > 0.74 {
                        c = [1.0, 0.66, 0.32];
                    }
                }
                let p = pagoda(x - PX, y);
                if p > 0 {
                    m = Mat::Pagoda;
                    let dx = x - PX;
                    let rim = (-(dx + 4.0).abs() / 30.0).exp(); // the sky to the west lights its left
                                                                // dark timber; the tiled roofs pick up the sky, their ridges most of all
                    c = match p {
                        1 => [0.07, 0.05, 0.12],
                        2 => [0.2 + 0.08 * rim, 0.13 + 0.03 * rim, 0.27 + 0.04 * rim],
                        3 => [0.05, 0.04, 0.1],
                        4 => [0.9 * rim + 0.25, 0.5 * rim + 0.15, 0.5 * rim + 0.3],
                        5 => [if dx < 0.0 { 0.5 } else { 0.3 }, 0.22, 0.32],
                        _ => [0.7, 0.42, 0.24],
                    };
                }
            } else if x < bank_x(y) {
                // the near bank where the lantern stands
                m = Mat::Bank;
                let tex = fbm(x * 0.3, y * 0.5, 3);
                c = [0.08 + 0.05 * tex, 0.08 + 0.05 * tex, 0.1 + 0.05 * tex];
                // fallen petals on the moss
                if hash(px as f64 * 7.0, py as f64 * 3.0) > 0.990 - 0.006 * smooth(SHORE + 4.0, TH, y) {
                    c = [0.42, 0.22, 0.3];
                }
                // the stone lip of the pond
                if x > bank_x(y) - 2.6 {
                    let s = 0.85 + 0.3 * hash(px as f64 * 3.0, py as f64 * 5.0);
                    c = [0.45 * s, 0.38 * s, 0.46 * s];
                }
                warm = 1.0;
            } else {
                m = Mat::Pond;
            }
            (mat[k], col[k], warmth[k]) = (m, c, warm);
        }
    }
    // the reflection source: the far side before the tree covers it
    let src = col.iter().map(|c| c.map(|v| v as f32)).collect();

    // the stone lantern
    for py in ys(45.0, TH) {
        let y = yc(py);
        for px in 0..W {
            let x = xc(px);
            if (x - LX).abs() > 15.0 {
                continue;
            }
            let l = lantern_at(x, y);
            if l == 0 {
                continue;
            }
            let k = py * W + px;
            let (ly, ldx) = (LB - (LB - y) / LS, (x - LX) / LS);
            if l == 2 {
                mat[k] = Mat::Flame;
                // hottest at the heart of the firebox
                let c = (-(ldx * ldx * 0.5 + (ly - 82.8).powi(2) * 0.35)).exp();
                col[k] = [0.85 + 0.15 * c, 0.5 + 0.38 * c, 0.2 + 0.4 * c * c];
                warmth[k] = 0.0;
            } else {
                mat[k] = Mat::Stone;
                // dark granite, its top surfaces catching the last of the sky
                let g = 0.8 + 0.4 * hash(px as f64 * 5.0, py as f64 * 3.0);
                let edge = if lantern_at(x, y - 1.0 / S) == 0 { 1.0 } else { 0.0 };
                let mut c = [(0.05 + 0.22 * edge) * g, (0.04 + 0.14 * edge) * g, (0.08 + 0.2 * edge) * g];
                // the roof's underside and the platform take the flame's light
                if (l == 3 && ly > 78.6) || (85.5..86.4).contains(&ly) {
                    let f = (-(ly - 82.8).abs() / 3.0).exp() * 0.75;
                    c = [c[0] + f, c[1] + f * 0.5, c[2] + f * 0.18];
                }
                col[k] = c;
                warmth[k] = 0.75;
            }
        }
    }

    // the cherry tree
    for py in 0..H {
        let y = yc(py);
        for px in 0..W {
            let x = xc(px);
            if x >= 130.0 {
                break;
            }
            let k = py * W + px;
            // blossom density: soft clouds, broken up by noise into clumps
            let (mut d, mut best, mut up) = (0.0, 0.0, 0.0);
            for &(cx, cy, rad) in &clusters {
                let (dx, dy) = (x - cx, (y - cy) * 1.15);
                if dx.abs() > rad * 1.8 || dy.abs() > rad * 1.8 {
                    continue;
                }
                let q = (dx * dx + dy * dy) / (rad * rad);
                if q < 3.0 {
                    let c = (-q * 1.4).exp();
                    d += c;
                    if c > best {
                        (best, up) = (c, (dy + dx * 0.4) / rad);
                    }
                }
            }
            // big holes where the sky shows through, small ones between clumps
            let (n1, n2) = (fbm(x * 0.11, y * 0.15, 3), noise(x * 0.5, y * 0.6));
            let bloom = (1.0 - (-d * 1.5).exp()) * (0.15 + 1.35 * n1) * (0.75 + 0.5 * n2);
            let (mut on_branch, mut bw) = (false, 0.0f64);
            for &[ax, ay, bx, by, w0, w1] in &branches {
                let (dist, kk) = seg(x, y, ax, ay, bx, by);
                let w = mix(w0, w1, kk) * 0.5 + 0.15;
                if dist < w {
                    on_branch = true;
                    bw = bw.max(dist / w * if x < ax + (bx - ax) * kk { -1.0 } else { 1.0 });
                }
            }
            if on_branch && bloom < 0.85 {
                // dark bark threading through the blossom; the side toward the lantern catches it
                let lit = smooth(0.35, 1.0, bw);
                (mat[k], col[k], warmth[k]) = (Mat::Tree, [0.05 + 0.03 * lit, 0.035 + 0.01 * lit, 0.07], 0.8);
            } else if bloom > 0.5 {
                // each spray pale pink on top where the sky lights it, deep rose beneath
                let inner = hash(px as f64 * 3.0, py as f64 * 7.0);
                let mut b = clamp(0.62 - 0.6 * up + 0.26 * (inner - 0.5) - 0.3 * smooth(0.75, 0.5, bloom));
                b = mix(b, 0.12, smooth(0.3, 1.1, up));
                // the lower crown sits in its own shade
                b *= 1.0 - 0.3 * smooth(28.0, 48.0, y);
                (mat[k], col[k], warmth[k]) = (Mat::Bloom, [0.34 + 0.64 * b, 0.08 + 0.62 * b * b, 0.2 + 0.5 * b], 1.0);
            }
        }
    }

    // the lantern's light: how far each pixel sits from the lit opening
    let mut glow = vec![0.0f32; n];
    for py in 0..H {
        let y = yc(py);
        for px in 0..W {
            let k = py * W + px;
            let x = xc(px);
            let d = (x - LAMP.0).hypot((y - LAMP.1) * 1.1);
            let mut g = (-d / 4.0).exp() * 0.6 + (-d / 11.0).exp() * 0.32 + (-d / 32.0).exp() * 0.1;
            // the blossom overhead takes the lamp's light from below, from further off
            if matches!(mat[k], Mat::Bloom | Mat::Tree) {
                g += (-d / 11.0).exp() * 1.6;
            }
            // and a pool of light on the moss round its foot
            if mat[k] == Mat::Bank {
                let (bx, by) = ((x - LX) / 20.0, (y - LB + 3.0) / 8.0);
                g += (-(bx * bx + by * by)).exp() * 0.6;
            }
            glow[k] = g as f32;
        }
    }

    // a few faint stars in the indigo
    let mut stars = vec![];
    for i in 0..400 {
        let i = f64::from(i);
        let (x, r) = ((hash(i, 91.0) * TW).floor(), (hash(i, 92.0) * 40.0).floor());
        if let Some((px, py)) = pixel_at(x, r) {
            if mat[py * W + px] == Mat::Sky && hash(i, 93.0) > 0.72 {
                stars.push((py * W + px, hash(i, 94.0) * TAU, 0.6 + hash(i, 95.0) * 1.6));
            }
        }
    }

    // thin streaks of cloud low in the sky, dark violet, their undersides lit
    // by the set sun; they wrap so they can drift forever
    let cdens = |x: f64, y: f64| fbm_wrap(x * 0.014, y * 0.13, 4, CW as f64 * 0.014) - 0.3 * (1.0 - smooth(C0, C0 + 10.0, y) * smooth(C1, C1 - 8.0, y));
    let rows = ys(C0, C1);
    let (c_py, c_rows) = (rows.start, rows.len());
    let mut ccov = vec![0.0f32; CW * c_rows];
    let mut clit = vec![0.0f32; CW * c_rows];
    for (j, py) in rows.enumerate() {
        let y = yc(py);
        for x in 0..CW {
            let xf = x as f64;
            let d = cdens(xf, y);
            ccov[j * CW + x] = smooth(0.56, 0.7, d) as f32;
            clit[j * CW + x] = clamp(0.5 + (d - cdens(xf - 1.0, y + 2.2)) * 7.0) as f32;
        }
    }

    // petals: each with its own drift, fall, sway and tumble; most of them
    // near the tree, thinning out downwind
    let petals = (0..50)
        .map(|i| {
            let i = f64::from(i);
            Petal {
                x: hash(i, 101.0).powf(2.2) * 140.0,
                y: hash(i, 102.0) * (TH + 20.0),
                vx: 3.0 + hash(i, 103.0) * 4.0,
                vy: 1.6 + hash(i, 104.0) * 2.2,
                sw: 1.0 + hash(i, 105.0) * 2.0,
                sf: 0.6 + hash(i, 106.0) * 1.2,
                ph: hash(i, 107.0) * TAU,
                tum: 2.0 + hash(i, 108.0) * 4.0,
            }
        })
        .collect();
    // petals afloat on the pond, drifting slowly
    let floaters = (0..26)
        .map(|i| {
            let i = f64::from(i);
            [hash(i, 111.0) * TW, SHORE + 3.0 + hash(i, 112.0) * 20.0, 0.3 + hash(i, 113.0) * 0.5]
        })
        .collect();

    Box::new(KyotoDusk {
        mat,
        col: col.into_iter().map(|c| c.map(|v| v as f32)).collect(),
        src,
        warmth,
        glow,
        c_py,
        c_rows,
        ccov,
        clit,
        shore_py: ys(0.0, SHORE).end - 1,
        stars,
        petals,
        floaters,
        pixels: Pixels::new(&META),
    })
}

impl Piece for KyotoDusk {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        // the flame breathes, with now and then a gutter
        let flick = 0.82 + 0.1 * (t * 7.3).sin() * (t * 3.1 + 1.0).sin() + 0.12 * (noise(t * 4.0, 3.3) - 0.5) * 2.0;
        let cdrift = (t * 0.8).rem_euclid(CW as f64);

        for py in 0..H {
            let y = yc(py);
            for px in 0..W {
                let k = py * W + px;
                let (x, xm) = (xc(px) - 0.5, xc(px));
                let m = self.mat[k];
                let (mut cr, mut cg, mut cb);
                let mut fade = 1.0;
                if m == Mat::Pond {
                    // the far side, upside down, shaken by small ripples
                    let depth = (y - SHORE) / (TH - SHORE);
                    let wob = (y * 1.7 - t * 2.0 + (x * 0.13 + t * 0.6).sin() * 1.4).sin() * (0.05 + 0.8 * depth);
                    let sx = ((x + wob + 0.5) * S - 0.5).round().clamp(0.0, W as f64 - 1.0) as usize;
                    let w = noise(x * 0.14 + t * 0.2, y * 1.1 - t * 0.8);
                    // ripples also tip the image up and down, so level lines break
                    // (drawn a little stretched, so the pagoda's lower roofs land in the pond)
                    let sr = SHORE - 1.0 - (y - 0.5 - SHORE) * 1.35 + (w - 0.5) * 3.0 * depth;
                    let sy = ((sr + 0.5) * S - 0.5).round().clamp(0.0, self.shore_py as f64) as usize;
                    let j = self.src[sy * W + sx];
                    // crisp and bright right under the bank, darker further out
                    let lit = mix(0.95, 0.6, smooth(SHORE + 1.0, SHORE + 6.0, y)) * (0.8 + 0.4 * w);
                    cr = f64::from(j[0]) * lit * 0.9;
                    cg = f64::from(j[1]) * lit * 0.92;
                    cb = f64::from(j[2]) * lit + 0.03;
                    let glint = smooth(0.74, 0.95, w) * 0.06;
                    cr += glint;
                    cg += glint * 0.8;
                    cb += glint;
                    // the lantern's light laid on the water as a broken warm streak
                    let sl = (-(xm - LX - 9.0 - (y - SHORE) * 0.25).abs() / (3.5 + depth * 3.0)).exp() * smooth(0.45, 0.8, w) * flick;
                    cr += sl;
                    cg += sl * 0.62;
                    cb += sl * 0.25;
                    fade = smooth(TH + 2.0, TH - 9.0, y);
                } else {
                    let c = self.col[k];
                    (cr, cg, cb) = (f64::from(c[0]), f64::from(c[1]), f64::from(c[2]));
                    if m == Mat::Sky && py >= self.c_py && py < self.c_py + self.c_rows {
                        let sx = x + cdrift;
                        let ix = sx.floor();
                        let fx = sx - ix;
                        let row = (py - self.c_py) * CW;
                        let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
                        let (i0, i1) = (row + i0, row + (i0 + 1) % CW);
                        let c = f64::from(self.ccov[i0]) + f64::from(self.ccov[i1] - self.ccov[i0]) * fx;
                        if c > 0.01 {
                            let l = f64::from(self.clit[i0]) + f64::from(self.clit[i1] - self.clit[i0]) * fx;
                            // brighter undersides toward the west and the horizon
                            let sun = l * (0.45 + 0.55 * smooth(C0, C1, y)) * (0.6 + 0.4 * (-(x - 80.0).abs() / 90.0).exp());
                            let a = c * 0.85;
                            cr = mix(cr, mix(0.2, 1.0, sun), a);
                            cg = mix(cg, mix(0.12, 0.58, sun), a);
                            cb = mix(cb, mix(0.27, 0.5, sun), a);
                        }
                    }
                }
                // lamplight on everything near the lantern
                let (wm, gl) = (f64::from(self.warmth[k]), f64::from(self.glow[k]) * flick);
                if wm > 0.0 {
                    let g = gl * wm;
                    (cr, cg, cb) = (cr + g, cg + g * 0.58, cb + g * 0.22);
                } else if m == Mat::Sky || m == Mat::Pond {
                    let g = gl * 0.6;
                    (cr, cg, cb) = (cr + g, cg + g * 0.6, cb + g * 0.3);
                }
                if m == Mat::Flame {
                    let f = 0.85 + 0.25 * flick;
                    (cr, cg, cb) = (f, 0.72 * f, 0.38 * f);
                }
                let c = [mix(GROUND[0], tone(cr), fade) as f32, mix(GROUND[1], tone(cg), fade) as f32, mix(GROUND[2], tone(cb), fade) as f32];
                self.pixels.set(px, py, c);
            }
        }

        // petals in the air: tumbling, catching the lamplight, wider as they turn face-on
        for p in &self.petals {
            let xx = (p.x + p.vx * t + p.sw * (t * p.sf + p.ph).sin()).rem_euclid(TW + 40.0) - 20.0;
            let yy = (p.y + p.vy * t + 0.6 * (t * p.sf * 1.7 + p.ph).sin()).rem_euclid(TH + 20.0) - 10.0;
            let Some((px, py)) = pixel_at(xx, yy) else { continue };
            let turn = (t * p.tum + p.ph).sin().abs();
            for px in px..(px + 1 + usize::from(turn > 0.6)).min(W) {
                let k = py * W + px;
                let m = self.mat[k];
                if m == Mat::Flame {
                    continue;
                }
                // dim against the dark bank and water
                let face = (0.55 + 0.45 * turn) * if matches!(m, Mat::Pond | Mat::Bank) { 0.6 } else { 1.0 };
                let lg = f64::from(self.glow[k]) * flick * 1.6;
                self.pixels.set(px, py, [tone(face + lg * 0.6) as f32, tone(0.76 * face + lg * 0.35) as f32, tone(0.84 * face + lg * 0.1) as f32]);
            }
        }

        // the moon: a thin crescent, lit on the side toward the set sun, its edges smoothed
        let moon = [1.0f32, 0.96, 0.86];
        for py in ys(MOON.1 - 6.5, MOON.1 + 6.5) {
            for px in 0..W {
                let (x, y) = (xc(px), yc(py));
                if (x - MOON.0).abs() > 6.5 {
                    continue;
                }
                let mut cov = 0.0;
                for (ox, oy) in [(-0.25, -0.25), (0.25, -0.25), (-0.25, 0.25), (0.25, 0.25)] {
                    let (dx, dy) = (x + ox / S - MOON.0, y + oy / S - MOON.1);
                    let (ex, ey) = (dx - 2.2, dy + 1.6); // the shadowed disc, offset up and right
                    if dx * dx + dy * dy < 5.2 * 5.2 && ex * ex + ey * ey > 4.9 * 4.9 {
                        cov += 0.25;
                    }
                }
                if cov > 0.0 {
                    let c = self.pixels.get(px, py);
                    self.pixels.set(px, py, [0, 1, 2].map(|i| c[i] + (moon[i] - c[i]) * cov));
                }
            }
        }
        // stars twinkle
        for &(k, ph, sp) in &self.stars {
            let s = (t * sp + ph).sin();
            if s > -0.3 {
                let b = if s > 0.7 { 1.0 } else { 0.6 };
                let (px, py) = (k % W, k / W);
                let c = self.pixels.get(px, py);
                self.pixels.set(px, py, [c[0].max(0.9 * b), c[1].max(0.88 * b), c[2].max(b)]);
            }
        }
        // petals resting on the pond
        for &[fx, fy, sp] in &self.floaters {
            let Some((px, py)) = pixel_at((fx + t * sp) % TW, fy + 0.3 * (t * 0.8 + fx).sin()) else { continue };
            for px in px..(px + 2).min(W) {
                if self.mat[py * W + px] == Mat::Pond {
                    self.pixels.set(px, py, [0.95, 0.72, 0.8]);
                }
            }
        }
        self.pixels.draw(grid, env);
    }
}
