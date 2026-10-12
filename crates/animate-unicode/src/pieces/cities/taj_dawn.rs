//! taj dawn: the Taj Mahal at first light, seen down its long reflecting
//! canal between rows of cypress. The sun has just cleared the red sandstone
//! mosque on the left; the haze and the clouds drift, the canal's reflection
//! ripples, light glints along its far end, and a few birds cross. After
//! ascii.rest's taj-dawn.ts (MIT, @bas3line).
//!
//! The Taj, the mosque, the garden, the cypresses, the clouds and the haze,
//! and how each is shaded, are ascii.rest's, shaded here per pixel instead of
//! per cell. Where ascii.rest draws each of 200 × 100 square cells as a dot as
//! big as it is bright, this draws 320 × 80 cells of two square pixels each,
//! an upper half block over a back colour ([`crate::pixels`]): 1.6 times the
//! detail each way, solid colour, and gradients dithered in colour rather
//! than in dot size. The palette adds the dark violets, cypress greens and
//! sandstone a solid pixel needs where a small dot was enough. In one ink
//! each pixel is octant dots, as many as it is bright.
//!
//! Added here: the haze lifts and thickens again over five minutes, rising
//! off the garden as it thins, and the birds fly in front of the Taj (in
//! ascii.rest they pass behind it), so they are mirrored in the canal as
//! they cross it.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{FRAC_PI_2, TAU};

/// Taj dawn's meta.
pub static META: Meta = Meta {
    name: "taj dawn",
    slug: "taj-dawn",
    category: Category::Cities,
    note: "the taj mahal at sunrise, mirrored in its pool through morning haze",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: dawn violets, mauves and roses, the sun's golds, the
    // marble's lavender and pink-whites, the greens of the garden, the
    // sandstone. Then the dark violets, cypress greens and sandstone shades.
    palette: Some(&[
        "#2e2a55", "#3d3870", "#514a8a", "#6a62a3", "#8a83bd", "#5a3a5e", "#7a4d72", "#9a6284", "#b97a92", "#d495a2", "#b8705a", "#d98d66", "#efab78",
        "#f8c98e", "#ffe2ae", "#fff3d8", "#fffcf2", "#8e7f9e", "#ad9cb4", "#cbb6c4", "#e3cdd2", "#f3e0dc", "#2a4a3e", "#36584a", "#4a6c58", "#66845f",
        "#8a9a68", "#8a4a3e", "#b0624a", "#3a2f4a", "#4e3d5a", "#f6d6c0", "#e9bfae", "#5e6a40", "#7d8a52", "#a3a064", "#c2b274", "#0d0a13", "#110e24",
        "#17132f", "#1e1a3c", "#26214a", "#0a1410", "#0e1d17", "#13271f", "#1a3329", "#2a1620", "#3f1d25", "#55262c", "#6e3a3c", "#1c1424", "#2b2034",
        "#3a2a3e", "#4a3044", "#251c40", "#30285a", "#625070", "#6f5a60", "#8c7466", "#a48e6c", "#b89c7c", "#c8a88a",
    ]),
    ground: Some("#0d0a13"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const PX: f64 = 1.6;
/// The Taj's axis, and the canal's vanishing point.
const CX: f64 = 128.0;
/// Where the Taj meets the garden, and the far end of the canal.
const BASE: f64 = 68.0;
/// Eye level.
const HZ: f64 = 60.0;
/// Cells per metre on the Taj.
const TS: f64 = 0.7;
const SUN: (f64, f64) = (36.0, 40.0);
/// The mosque's axis and foot.
const MX: f64 = 45.0;
const MB: f64 = 67.0;
/// The reflection is foreshortened so the dome reaches the canal.
const KR: f64 = 1.55;
/// The cloud deck's width in cells and pixels (it wraps), and its height in pixels.
const CW: f64 = 400.0;
const CWP: usize = 640;
const CHP: usize = 80;
/// The haze strips' width, in cells and in pixels (they wrap).
const HWD: f64 = 400.0;
const HWP: usize = 640;
/// How long the haze takes to lift and settle again, in seconds.
const LIFT: f64 = 300.0;
const BIRD: [f64; 3] = [0.16, 0.1, 0.17];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    BgTree,
    Marble,
    Lawn,
    Walk,
    Pool,
    Cypress,
    Mosque,
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / PX
}

/// The scene's x, in ascii.rest's columns (a cell's left edge, as its `x`),
/// at pixel column `px`.
fn col_x(px: usize) -> f64 {
    (px as f64 + 0.5) / PX - 0.5
}

/// The onion dome's profile: swelling out past the drum, then drawn to a point.
fn onion(h: f64) -> f64 {
    if h < 0.3 {
        0.8 + 0.2 * (h / 0.3 * FRAC_PI_2).sin()
    } else {
        ((h - 0.3) / 0.7 * FRAC_PI_2).cos().powf(1.45)
    }
}

/// Pointed arch: half-width at height h above the springline, for an arch of
/// half-width w; zero above the apex.
fn arch(w: f64, h: f64) -> f64 {
    if h <= 0.0 {
        return w;
    }
    let c = w * 0.5;
    let r = w + c;
    let q = r * r - h * h;
    if q > 0.0 {
        (q.sqrt() - c).max(0.0)
    } else {
        0.0
    }
}

/// `v & 1` for a non-negative float, as JS's `Math.floor(v) & 1`.
fn odd(v: f64) -> bool {
    (v.floor() as i64) & 1 == 1
}

/// The Taj in metres: X across from the axis, Y up from the garden. Returns
/// (lit, recess) for marble, or `None` for air. lit runs 0 (shadow, facing
/// away from the sun) to 1 (facing it); recess darkens arches and niches.
fn taj(x: f64, y: f64) -> Option<(f64, f64)> {
    let ax = x.abs();
    let left = if x < 0.0 { 1.0 } else { -1.0 }; // +1 on the sun's side
    let flip = |c: f64| if x < 0.0 { -c } else { c };
    // finial
    let bulge = if (y - 74.5).abs() < 0.9 { 0.6 } else { 0.0 } + if (y - 77.0).abs() < 0.7 { 0.4 } else { 0.0 };
    if (72.0..80.5).contains(&y) && ax <= 0.75 + bulge {
        return Some((0.75 + 0.2 * left, 0.0));
    }
    // the great dome
    if (47.5..72.0).contains(&y) {
        let h = (y - 47.5) / 24.5;
        let hw = 15.0 * onion(h);
        if ax <= hw {
            // a sphere lit from the left and a little above
            let nx = x / (hw + 0.01);
            let nz = (1.0 - nx * nx).max(0.0).sqrt();
            return Some((clamp(0.3 + 0.5 * (-0.8 * nx + 0.4 * nz) + 0.2 * h), 0.0));
        }
    }
    // the four chhatris on the roof, two in view
    let cx = ax - 17.5;
    if cx.abs() <= 4.4 && (39.5..55.5).contains(&y) {
        let nx = flip(cx) / 4.4;
        if y < 41.0 {
            return Some((clamp(0.5 - 0.4 * nx * left), 0.0));
        }
        if y < 46.5 {
            if cx.abs() > 3.6 {
                return None;
            }
            let open = cx.abs() < 2.6 && cx.abs() > 0.6 && y < 45.5;
            return Some((clamp(0.5 - 0.45 * flip(cx) / 4.0 * left), if open { 0.75 } else { 0.0 }));
        }
        if y < 47.3 {
            return Some((clamp(0.55 - 0.4 * nx * left), 0.0));
        }
        let h = (y - 47.3) / 6.5;
        if h < 1.0 && cx.abs() <= 4.0 * onion(h) {
            return Some((clamp(0.55 - 0.6 * (flip(cx) / (4.0 * onion(h) + 0.01)) * left + 0.1 * h), 0.0));
        }
        if h >= 1.0 && cx.abs() < 0.6 {
            return Some((0.6, 0.0));
        }
    }
    // the drum under the dome
    if (40.0..47.5).contains(&y) && ax <= 12.2 {
        let nx = x / 12.2;
        let band = if y > 45.8 { 0.15 } else { 0.0 };
        return Some((clamp(0.45 - 0.55 * nx + band), 0.0));
    }
    // slender pinnacles at the corners of the portal and of the building
    if ((ax - 8.8).abs() < 0.75 && (40.0..47.5).contains(&y)) || ((ax - 28.2).abs() < 0.75 && (38.0..44.5).contains(&y)) {
        return Some((0.55 + 0.25 * left, 0.0));
    }
    // the main building
    if ax <= 28.5 && (7.0..40.0).contains(&y) {
        // the portal rises a little above the parapet
        if y >= 38.5 && ax > 9.0 && odd((ax + 0.5) / 1.5) {
            return None;
        }
        if ax <= 9.0 {
            // central portal: a calligraphy band round a deep pointed arch
            let hw = arch(6.0, y - 25.0);
            if ax <= hw && y < 25.0 + 9.0 {
                let door = ax <= 3.2 && ax <= arch(3.2, y - 15.5);
                return Some((0.35 + 0.15 * left, if door { 0.82 } else { 0.6 + 0.12 * (1.0 - smooth(25.0, 33.0, y)) }));
            }
            if ax <= hw + 0.8 && y < 25.0 + 10.2 {
                return Some((0.42, 0.35));
            }
            if ax > 7.6 && ax <= 9.0 && y < 40.0 {
                return Some((0.5 + 0.12 * left, 0.08));
            }
            return Some((0.5 + 0.08 * left, 0.0));
        }
        if ax <= 21.0 {
            // front face either side of the portal: two storeys of arched niches
            let nx = ax - 15.0;
            for (y0, y1) in [(9.0, 21.5), (24.5, 37.0)] {
                if y >= y0 && y < y1 && nx.abs() <= arch(3.6, y - (y1 - 4.5)) {
                    return Some((0.4 + 0.1 * left, 0.5));
                }
                if y >= y0 - 0.6 && y < y1 + 0.6 && nx.abs() <= arch(4.3, y - (y1 - 4.2)) && nx.abs() > 3.6 {
                    return Some((0.45, 0.22));
                }
            }
            return Some((0.5 + 0.1 * left, 0.0));
        }
        // chamfered corners, turned toward the sun on the left and away on the right
        let lit = 0.5 + 0.48 * left;
        let nx = ax - 24.7;
        for (y0, y1) in [(9.0, 21.5), (24.5, 37.0)] {
            if y >= y0 && y < y1 && nx.abs() <= arch(2.2, y - (y1 - 3.0)) {
                return Some((lit * 0.7, 0.45));
            }
        }
        return Some((lit, 0.0));
    }
    // minarets at the corners of the plinth, tapering, with three galleries
    let mx = ax - 44.0;
    if (7.0..57.0).contains(&y) {
        let s = flip(mx); // across the shaft, toward the sun negative
        let hw = 2.9 - 0.6 * (y - 7.0) / 40.0;
        for g in [19.5, 32.0, 44.5] {
            if y >= g && y < g + 1.4 && mx.abs() <= hw + 1.1 {
                return Some((clamp(0.5 - 0.42 * s / (hw + 1.1) * left), if y < g + 0.5 { 0.35 } else { 0.0 }));
            }
        }
        if y < 46.0 && mx.abs() <= hw {
            return Some((clamp(0.5 - 0.48 * (s / hw) * left), 0.0));
        }
        if (45.9..49.5).contains(&y) && mx.abs() <= 2.2 {
            return Some((clamp(0.5 - 0.4 * s / 2.2 * left), if mx.abs() < 1.4 && mx.abs() > 0.3 { 0.7 } else { 0.0 }));
        }
        if y >= 49.5 {
            let h = (y - 49.5) / 4.5;
            if h < 1.0 && mx.abs() <= 2.5 * onion(h) {
                return Some((clamp(0.55 - 0.55 * s / (2.5 * onion(h) + 0.01) * left), 0.0));
            }
            if h >= 1.0 && y < 56.0 && mx.abs() < 0.5 {
                return Some((0.6, 0.0));
            }
        }
    }
    // the plinth, with a row of shallow niches
    if ax <= 47.5 && (0.0..7.0).contains(&y) {
        if y > 6.2 {
            return Some((0.62, 0.0));
        }
        let k = (ax % 5.2) - 2.6;
        if y > 1.5 && y < 5.2 && k.abs() < 1.1 {
            return Some((0.42, 0.3));
        }
        return Some((0.52, 0.0));
    }
    None
}

/// The red sandstone mosque that flanks the Taj, in cells: three domes over
/// a five-bay front with a tall central portal. Returns 0 for wall, 1 for
/// dome, 2 for a recess, or -1 for air. It stands against the sun, so it is
/// mostly silhouette.
fn mosque(xc: f64, y: f64) -> i8 {
    let dx = xc - MX;
    let ax = dx.abs();
    // plinth
    if (MB - 2.5..MB).contains(&y) && ax <= 22.0 {
        return 0;
    }
    // end towers, each with a small kiosk on top
    let tx = (ax - 19.5).abs();
    if tx <= 1.3 && (MB - 13.0..MB - 2.5).contains(&y) {
        return 0;
    }
    if (MB - 15.5..MB - 13.0).contains(&y) && tx <= 1.8 * onion((MB - 13.0 - y) / 2.5) {
        return 1;
    }
    if tx < 0.35 && (MB - 16.5..MB - 15.5).contains(&y) {
        return 1;
    }
    // central portal, rising above the front
    if ax <= 5.5 && (MB - 15.0..MB - 2.5).contains(&y) {
        if y < MB - 14.3 && odd(xc) {
            return -1;
        }
        if ax <= arch(3.4, MB - 9.5 - y) && y >= MB - 13.5 {
            return 2;
        }
        return 0;
    }
    // the five-bay front and its parapet
    if ax <= 18.0 && (MB - 10.0..MB - 2.5).contains(&y) {
        let bay = ((ax - 5.5) % 4.2) - 2.1;
        if ax > 6.0 && y >= MB - 8.5 && bay.abs() <= arch(1.4, MB - 6.2 - y) {
            return 2;
        }
        return 0;
    }
    if ax <= 18.0 && (MB - 10.8..MB - 10.0).contains(&y) && odd(xc * 0.75) {
        return 0;
    }
    // side domes on drums
    let sx = (ax - 11.5).abs();
    if sx <= 2.6 && (MB - 12.0..MB - 10.0).contains(&y) {
        return 0;
    }
    if (MB - 17.0..MB - 12.0).contains(&y) && sx <= 3.6 * onion((MB - 12.0 - y) / 5.0) {
        return 1;
    }
    if sx < 0.35 && (MB - 18.5..MB - 17.0).contains(&y) {
        return 1;
    }
    // the great central dome
    if ax <= 4.0 && (MB - 16.5..MB - 15.0).contains(&y) {
        return 0;
    }
    if (MB - 23.5..MB - 16.5).contains(&y) && ax <= 5.2 * onion((MB - 16.5 - y) / 7.0) {
        return 1;
    }
    if ax < 0.4 && (MB - 25.5..MB - 23.5).contains(&y) {
        return 1;
    }
    -1
}

/// The sun's glow at (x, y): a hot core in a wide warm halo.
fn sun_glow(x: f64, y: f64) -> f64 {
    let (dx, dy) = (x - SUN.0, (y - SUN.1) * 1.25);
    let d = dx.hypot(dy);
    (-d / 4.0).exp() * 0.9 + (-d / 13.0).exp() * 0.42 + (-d / 40.0).exp() * 0.3
}

/// The dawn sky at (x, y), cell coordinates.
fn sky_at(x: f64, y: f64) -> [f64; 3] {
    let v = clamp(y / HZ);
    // deep violet overhead, mauve, then rose and peach toward the horizon
    let up = smooth(0.05, 0.8, v);
    let (mut r, mut g, mut b) = (mix(0.055, 0.3, up), mix(0.05, 0.19, up), mix(0.17, 0.36, up));
    let low = smooth(0.62, 1.0, v);
    (r, g, b) = (mix(r, 0.66, low), mix(g, 0.4, low), mix(b, 0.42, low));
    // cooler and dimmer on the side away from the sun
    let far = smooth(50.0, 200.0, x) * 0.18;
    (r, g, b) = (r * (1.0 - far), g * (1.0 - far * 0.8), b * (1.0 - far * 0.3));
    let glow = sun_glow(x, y) + (-(y - 57.0).abs() / 5.0).exp() * 0.25 * (-(x - SUN.0).abs() / 50.0).exp();
    (r, g, b) = (r + glow, g + glow * 0.78, b + glow * 0.48);
    // faint shafts of light fanning up from the sun through the haze
    let ang = (y - SUN.1).atan2(x - SUN.0);
    let ray = fbm(ang * 9.0 + 3.0, 1.7, 2);
    let rd = (x - SUN.0).hypot(y - SUN.1);
    let shaft = smooth(0.5, 0.75, ray) * (-rd / 45.0).exp() * smooth(4.0, 14.0, rd) * 0.12;
    [r + shaft, g + shaft * 0.8, b + shaft * 0.55]
}

/// The haze over the far garden, its colour by how near the sun it is.
fn mist(xc: f64) -> [f64; 3] {
    let mw = (-(xc - SUN.0).abs() / 90.0).exp();
    [0.62 + 0.25 * mw, 0.46 + 0.16 * mw, 0.55 + 0.02 * mw]
}

/// A gentle shoulder so the brightest light keeps its gradient.
fn tone(v: f64) -> f64 {
    let v = v.clamp(0.0, 4.0);
    if v < 0.75 {
        v
    } else {
        0.75 + 0.25 * (1.0 - (-(v - 0.75) * 4.0).exp())
    }
}

/// A cypress: its foot's x, the row of its foot, its height and half-width,
/// and its distance (scale) down the canal.
struct Tree {
    x: f64,
    base: f64,
    h: f64,
    w: f64,
    s: f64,
}

/// The garden at dawn: what each pixel is and its colour before the light
/// moves, how much haze and sun glow it takes, the cloud deck, the haze
/// strips, and the birds.
pub struct TajDawn {
    mat: Vec<Mat>,
    rgb: Vec<[f32; 3]>,
    haze_w: Vec<f32>,
    glow_a: Vec<f32>,
    cloud: Vec<f32>,
    cloud_lit: Vec<f32>,
    haze1: Vec<f32>,
    haze2: Vec<f32>,
    birds: [(f64, f64, f64); 3],
    bird: Vec<bool>,
    pixels: Pixels,
}

/// A dawn at the Taj; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;

    // Dawn cloud in a wrapping strip that drifts: broken noise, gathered into
    // a bank up and right of the sun and a lower one behind the dome.
    // [x, y, half-width, height above, depth below, strength]
    let banks: [[f64; 6]; 5] = [
        [76.0, 28.0, 40.0, 7.0, 3.5, 1.4],  // over the sun's right shoulder
        [152.0, 36.0, 52.0, 9.0, 4.0, 1.0], // behind the dome
        [16.0, 13.0, 32.0, 4.0, 2.5, 0.8],  // a high wisp
        [250.0, 24.0, 40.0, 8.0, 4.0, 1.0],
        [330.0, 33.0, 36.0, 7.0, 4.0, 0.95],
    ];
    let cdens = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.01, y * 0.04, 2, CW * 0.01);
        let nn = fbm_wrap(x * 0.025 + q * 1.6, y * 0.075 + q * 0.6, 5, CW * 0.025);
        let mut m: f64 = 0.0;
        for [bx, by, rx, up, down, a] in banks {
            let mut dx = x - bx;
            dx -= (dx / CW).round() * CW;
            let ex = dx / rx;
            let ey = (y - by) / if y < by { up } else { down };
            m = m.max((-ex * ex * ex * ex - ey * ey).exp() * a);
        }
        0.62 * m + 1.4 * (nn - 0.5) + 0.02
    };
    let mut cloud = vec![0.0f32; CWP * CHP];
    let mut cloud_lit = vec![0.0f32; CWP * CHP];
    for py in 0..CHP {
        let y = row_y(py);
        for i in 0..CWP {
            let x = i as f64 / PX;
            let d = cdens(x, y);
            cloud[py * CWP + i] = smooth(0.46, 0.66, d) as f32;
            // the side toward the sun (down and left) catches the light
            let toward = cdens(x - 2.0, y + 2.5);
            cloud_lit[py * CWP + i] = clamp(0.56 + (d - toward) * 0.75 - (d - 0.6) * 0.2 + 0.7 * (fbm_wrap(x * 0.09, y * 0.2, 2, CW * 0.09) - 0.5)) as f32;
        }
    }

    // --- the land ---
    // the far tree line: rounded crowns, lower behind the Taj
    let mut crowns = vec![];
    let (mut x, mut i) = (-12.0, 0.0);
    while x < 200.0 + 12.0 {
        let rad = 3.0 + hash(i, 61.0) * 4.5;
        crowns.push((x, 60.5 - hash(i, 62.0) * 2.5 - 2.5 * fbm(x * 0.03, 2.0, 2) + rad * 0.35, rad));
        x += 4.0 + hash(i, 60.0) * 5.0;
        i += 1.0;
    }
    let tops: Vec<f64> = (0..W)
        .map(|px| {
            let x = col_x(px);
            let mut t0: f64 = 64.0;
            for &(cx, cy, rad) in &crowns {
                let dx = x + 0.5 - cx;
                if dx.abs() < rad {
                    t0 = t0.min(cy - (rad * rad - dx * dx).sqrt() * 0.75);
                }
            }
            t0 - 0.6 * fbm(x * 0.5, 7.0, 2)
        })
        .collect();
    let pool_half = |y: f64| 0.5 * (y - HZ) + 1.0;
    let walk_half = |y: f64| 0.7 * (y - HZ) + 1.5;

    // cypress rows either side of the canal, near ones last
    let mut trees = vec![];
    for s in [8.5, 10.4, 13.0, 16.8, 22.5, 31.0] {
        for side in [-1.0, 1.0] {
            // an inner row along the walks, and a lower outer row further back
            for (off, tall) in [(4.3, 0.8), (2.45, 1.0)] {
                if off > 3.0 && (s > 20.0 || side > 0.0) {
                    continue;
                }
                if side < 0.0 && s > 30.0 {
                    continue; // leave the sun clear
                }
                let cxp = CX + side * off * s;
                trees.push(Tree {
                    x: cxp + (hash(s * 10.0, side + off) - 0.5) * 0.6,
                    base: HZ + s,
                    h: 1.6 * tall * s * (0.94 + 0.12 * hash(s * 7.0, side + 3.0 + off)),
                    w: 0.26 * s,
                    s,
                });
            }
        }
    }
    // long shadows of the cypresses, cast toward us and to the right
    let shadow_at = |xc: f64, y: f64| {
        let mut lit = 1.0;
        for tr in &trees {
            let (dx, dy) = (xc - tr.x, y - tr.base);
            if dy < -0.5 {
                continue;
            }
            let along = (dx * 0.6 + dy * 0.8) / tr.s;
            let across = (dx * 0.8 - dy * 0.6) / tr.s;
            if along > 0.0 && along < 3.0 && across.abs() < 0.17 * (1.0 - along / 3.4) + 0.03 {
                lit *= 0.08 + 0.92 * smooth(1.6, 3.0, along);
            }
        }
        lit
    };

    // the mosque's silhouette, and its edges toward the sun
    let mq: Vec<i8> = (0..n).map(|k| mosque(col_x(k % W) + 0.5, row_y(k / W))).collect();

    let mut mat = vec![Mat::Sky; n];
    let mut rgb = vec![[0.0f32; 3]; n];
    let mut haze_w = vec![0.0f32; n];
    let mut glow_a = vec![0.0f32; n];
    for py in 0..H {
        let y = row_y(py);
        for (px, &tt) in tops.iter().enumerate() {
            let k = py * W + px;
            let x = col_x(px);
            let xc = x + 0.5;
            let mut m = Mat::Sky;
            let (mut cr, mut cg, mut cb, mut hz) = (0.0, 0.0, 0.0, 0.0);
            if y < BASE && y >= tt {
                m = Mat::BgTree;
            } else if y >= BASE {
                let dx = (xc - CX).abs();
                m = if dx < pool_half(y) {
                    Mat::Pool
                } else if dx < walk_half(y) {
                    Mat::Walk
                } else {
                    Mat::Lawn
                };
            }
            match m {
                Mat::Sky => {
                    [cr, cg, cb] = sky_at(xc, y);
                    // never one flat tone: a faint unevenness in the dawn air
                    let veil = 0.88 + 0.24 * fbm(xc * 0.05, y * 0.09, 3);
                    (cr, cg, cb) = (cr * veil, cg * veil, cb * veil);
                    glow_a[k] = sun_glow(xc, y) as f32;
                    hz = 0.5 + 0.5 * smooth(20.0, 58.0, y);
                }
                Mat::BgTree => {
                    // distant trees, flattened by the haze; rimmed where the sun is close
                    let tex = fbm(xc * 0.25, y * 0.3, 3);
                    let depth = smooth(tt, BASE, y);
                    let sky = sky_at(xc, tt);
                    let a = 0.46 - 0.12 * depth + 0.12 * tex;
                    (cr, cg, cb) = (mix(0.1, sky[0], a), mix(0.09, sky[1], a), mix(0.15, sky[2], a));
                    let rim = smooth(tt + 1.8, tt, y) * (-(xc - SUN.0).abs() / 18.0).exp();
                    (cr, cg, cb) = (cr + 0.4 * rim, cg + 0.27 * rim, cb + 0.14 * rim);
                    // their feet lost in a bank of ground mist, broken into soft patches
                    let patch = 0.45 + 0.75 * smooth(0.3, 0.7, fbm(xc * 0.035 + 11.0, y * 0.18, 3));
                    let a = smooth(tt + 1.0, BASE + 1.0, y) * 0.75 * patch.min(1.0);
                    let mc = mist(xc);
                    (cr, cg, cb) = (mix(cr, mc[0], a), mix(cg, mc[1], a), mix(cb, mc[2], a));
                    hz = 0.8;
                }
                Mat::Lawn => {
                    let v = (y - BASE) / (100.0 - BASE);
                    let u = (xc - CX) / (y - HZ); // across the ground plane
                    let tex = fbm(u * 3.0, 40.0 / (y - HZ), 3);
                    let stripe = if odd(u * 1.4) { 1.0 } else { 0.55 }; // mown bands that run toward the Taj
                                                                        // low sun raking across the grass from the left: gold where
                                                                        // it lands, violet sky-light in the shade
                    let mut lit = (0.6 + 0.4 * (-(xc - SUN.0).abs() / 60.0).exp()) * stripe * (0.85 + 0.3 * tex);
                    lit *= shadow_at(xc, y);
                    let vig = 1.0 - 0.45 * smooth(0.5, 1.0, v) * (0.6 + 0.4 * smooth(40.0, 0.0, xc.min(200.0 - xc)));
                    let fall = 1.0 - 0.35 * v;
                    // sunlit grass turns from gold near the sun to rose further off
                    let warm = (-(xc - SUN.0).abs() / 80.0).exp();
                    // the shade holds the violet of the sky overhead, brighter in the open
                    let amb = 0.75 + 0.5 * stripe - 0.25 * v;
                    cr = (0.12 * amb + 0.56 * lit * fall) * vig;
                    cg = (0.09 * amb + (0.36 + 0.14 * warm) * lit * fall) * vig;
                    cb = (0.24 * amb + (0.26 - 0.1 * warm) * lit * fall) * vig;
                    // the far lawn sits in the haze
                    let far = smooth(BASE + 10.0, BASE, y) * 0.7;
                    let mc = mist(xc);
                    (cr, cg, cb) = (mix(cr, mc[0], far), mix(cg, mc[1], far), mix(cb, mc[2], far));
                    hz = 0.7 * (1.0 - v);
                }
                Mat::Walk => {
                    let v = (y - BASE) / (100.0 - BASE);
                    let lit = if xc < CX { 1.0 } else { 0.86 };
                    // paving joints, closer together into the distance
                    let joint = if (90.0 / (y - HZ)) % 1.0 < 0.14 { 0.72 } else { 1.0 } * (0.5 + 0.5 * shadow_at(xc, y));
                    let l = lit * joint;
                    (cr, cg, cb) = ((0.78 - 0.22 * v) * l, (0.57 - 0.16 * v) * l, (0.53 - 0.12 * v) * l);
                    hz = 0.3;
                }
                _ => {}
            }
            // the mosque, dark against the sun, its edges caught by it
            let q = mq[k];
            if q >= 0 {
                m = Mat::Mosque;
                let sg = sun_glow(xc, y);
                let a = match q {
                    1 => [0.2, 0.11, 0.19],
                    2 => [0.07, 0.04, 0.07],
                    _ => [0.31, 0.13, 0.15],
                };
                let air = 0.06 + 0.3 * smooth(MB - 8.0, MB, y); // the foot sinks into the mist
                (cr, cg, cb) = (mix(a[0], 0.62, air), mix(a[1], 0.44, air), mix(a[2], 0.5, air));
                // rim light where the pixel faces open sky toward the sun
                let lft = if px > 0 { mq[k - 1] } else { -1 };
                let up = if py > 0 { mq[k - W] } else { -1 };
                let toward = if xc < SUN.0 {
                    if px < W - 1 {
                        mq[k + 1]
                    } else {
                        -1
                    }
                } else {
                    lft
                };
                let rim = if up < 0 { 0.75 } else { 0.0 } + if toward < 0 { 0.9 } else { 0.0 } + if lft < 0 && up < 0 { 0.2 } else { 0.0 };
                let rim = f64::min(1.0, rim) * (0.25 + sg * 1.6).min(1.0) * 0.8;
                (cr, cg, cb) = (mix(cr, 1.0, rim), mix(cg, 0.7, rim), mix(cb, 0.48, rim));
                hz = 0.15;
            }
            // the Taj in front of the sky and the trees
            if y < BASE + 0.01 {
                if let Some((lit, rec)) = taj((xc - CX) / TS, (BASE - y) / TS) {
                    m = Mat::Marble;
                    // lavender shadow, pink-white front, gold where it faces the sun
                    let (sh, fr, gd) = ([0.42, 0.36, 0.56], [0.9, 0.76, 0.75], [1.0, 0.88, 0.72]);
                    let (a, b, kk) = if lit < 0.5 { (sh, fr, (lit * 2.0).powf(1.4)) } else { (fr, gd, (lit - 0.5) * 2.0) };
                    (cr, cg, cb) = (mix(a[0], b[0], kk), mix(a[1], b[1], kk), mix(a[2], b[2], kk));
                    if rec > 0.0 {
                        let d = 1.0 - rec * 0.78;
                        (cr, cg, cb) = (cr * d * 0.95, cg * d * 0.92, cb * d);
                    }
                    // a little grain in the stone, and the morning haze over the
                    // lower storeys
                    let g = 0.94 + 0.08 * hash(px as f64 * 3.0 + 1.0, py as f64 * 5.0 + 2.0);
                    let low = smooth(40.0, BASE, y) * 0.2;
                    let sky = sky_at(xc, y);
                    (cr, cg, cb) = (mix(cr * g, sky[0], low), mix(cg * g, sky[1], low), mix(cb * g, sky[2], low));
                    hz = 0.55;
                }
            }
            mat[k] = m;
            rgb[k] = [cr as f32, cg as f32, cb as f32];
            haze_w[k] = hz as f32;
        }
    }

    // cypresses, far to near: dark, with a warm rim on the side toward the sun
    for tr in &trees {
        let top = tr.base - tr.h;
        let sun_side = if tr.x < CX { 0.9 } else { 0.6 };
        let (py0, py1) = (((top * PX).floor().max(0.0)) as usize, (((tr.base + 0.6) * PX).ceil() as usize).min(H));
        let (px0, px1) = ((((tr.x - tr.w - 2.0) * PX).floor().max(0.0)) as usize, (((tr.x + tr.w + 2.0) * PX).ceil() as usize).min(W));
        for py in py0..py1 {
            let y = row_y(py);
            let u = (tr.base - y) / tr.h; // 0 at the foot, 1 at the tip
            if u > 1.0 {
                continue;
            }
            let p = if u < 0.06 {
                0.18
            } else if u < 0.3 {
                0.8 + 0.2 * (u / 0.3)
            } else {
                ((1.0 - u) / 0.7).powf(0.8)
            };
            for px in px0..px1 {
                let xc = col_x(px) + 0.5;
                let nn = noise(xc * 0.9, y * 0.55 + tr.s);
                let hw = tr.w * p * (0.88 + 0.26 * nn) + 0.35;
                let dx = (xc - tr.x) / hw;
                if dx.abs() > 1.0 {
                    continue;
                }
                let k = py * W + px;
                let leaf = fbm(xc * 0.6, y * 0.35 + tr.s * 3.0, 2);
                let rim = smooth(-0.2, -0.9, dx) * (0.4 + 0.6 * leaf) * (0.5 + 0.5 * (-(tr.x - SUN.0).abs() / 80.0).exp());
                // clumps of foliage, each a little lit on top
                let clump = smooth(0.45, 0.7, fbm(xc * 0.45, y * 0.3 + tr.s * 3.0, 3));
                let base = 0.5 + 0.5 * leaf + 0.5 * clump;
                let dist = smooth(30.0, 8.0, tr.s); // far trees take more haze
                let cr = mix(0.045 * base + sun_side * rim, 0.7, dist * 0.45);
                let cg = mix(0.1 * base + sun_side * 0.63 * rim, 0.5, dist * 0.45);
                let cb = mix(0.08 * base + sun_side * 0.27 * rim, 0.56, dist * 0.45);
                mat[k] = Mat::Cypress;
                rgb[k] = [cr as f32, cg as f32, cb as f32];
                haze_w[k] = ((0.3 + 0.4 * dist) * smooth(tr.base - tr.h * 0.7, tr.base, y)) as f32;
            }
        }
    }

    // drifting haze, two layers in strips that wrap
    let mut haze1 = vec![0.0f32; HWP * H];
    let mut haze2 = vec![0.0f32; HWP * H];
    for py in 0..H {
        let r = row_y(py) - 0.5;
        for i in 0..HWP {
            let x = i as f64 / PX;
            haze1[py * HWP + i] = smooth(0.4, 0.75, fbm_wrap(x * 0.018, r * 0.09, 4, HWD * 0.018)) as f32;
            haze2[py * HWP + i] = smooth(0.42, 0.8, fbm_wrap(x * 0.04 + 7.0, r * 0.16 + 3.0, 3, HWD * 0.04)) as f32;
        }
    }

    let birds = [0.0, 1.0, 2.0].map(|i| (58.0 + i * 8.0 + hash(i, 40.0) * 4.0, 30.0 - i * 2.5 + hash(i, 41.0) * 2.0, hash(i, 42.0) * TAU));
    Box::new(TajDawn { mat, rgb, haze_w, glow_a, cloud, cloud_lit, haze1, haze2, birds, bird: vec![false; n], pixels: Pixels::new(&META) })
}

/// A bird's pixels about its body, wings up and wings down.
const WINGS_UP: [(i64, i64); 7] = [(0, 0), (-1, -1), (1, -1), (-2, -1), (2, -1), (-3, -2), (3, -2)];
const WINGS_DOWN: [(i64, i64); 7] = [(0, 0), (-1, 0), (1, 0), (-2, 0), (2, 0), (-3, 1), (3, 1)];

impl Piece for TajDawn {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (d1, d2) = (t * 1.1 * PX, t * 2.3 * PX);
        let cd = t * 0.5 * PX;
        let breathe = 0.035 * (t / 6.0 * TAU).sin();
        // the haze lifting off the garden and settling again
        let lift = 0.5 - 0.5 * (t / LIFT * TAU).cos();
        let (thick, rise) = (1.0 - 0.55 * lift, 7.0 * lift);
        let ground = [0.051, 0.039, 0.075];

        // the last pixel row above the garden
        let top_row = ((BASE * PX - 0.5).floor()) as i64;
        // where the birds are this frame: in front of the sky and the Taj
        self.bird.fill(false);
        for &(x0, y0, ph) in &self.birds {
            let bx = (x0 + t * 3.2).rem_euclid(260.0) - 30.0;
            let by = y0 + (t * 0.4 + ph).sin() * 1.3;
            let (cx, cy) = (((bx + 0.5) * PX - 0.5).round() as i64, ((by + 0.5) * PX - 0.5).round() as i64);
            let wings = if (t * 7.0 + ph).sin() > 0.0 { WINGS_UP } else { WINGS_DOWN };
            for (dx, dy) in wings {
                let (x, y) = (cx + dx, cy + dy);
                if (0..W as i64).contains(&x) && (0..=top_row).contains(&y) && self.mat[y as usize * W + x as usize] != Mat::Cypress {
                    self.bird[y as usize * W + x as usize] = true;
                }
            }
        }

        for py in 0..H {
            let y = row_y(py);
            let hr = 0.03 + 0.42 * thick * (-(y - (63.0 - rise)).abs() / 6.0).exp();
            for px in 0..W {
                let k = py * W + px;
                let x = col_x(px);
                let m = self.mat[k];
                let mut fade = 1.0;
                let [mut cr, mut cg, mut cb] = self.rgb[k].map(f64::from);

                if m == Mat::Pool {
                    // the mirror: the scene above, flipped about the far end,
                    // shortened and shaken
                    let depth = (y - BASE) / (100.0 - BASE);
                    let wob = (y * 1.9 - t * 2.4 + (x * 0.11 + t * 0.7).sin() * 1.5).sin() * (0.2 + 0.7 * depth);
                    let sx = ((px as f64 + wob * PX).round() as i64).clamp(0, W as i64 - 1) as usize;
                    let jump = if (x * 0.3 + t * 1.3 + y).sin() > 0.6 { 1.0 } else { 0.0 };
                    let src = BASE - (y - BASE) * KR - jump;
                    let sy = ((src * PX - 0.5).round() as i64).clamp(0, top_row) as usize;
                    let j = sy * W + sx;
                    // stone reflects brighter than the sky does, so the Taj holds its shape
                    let dim = if self.mat[j] == Mat::Marble { 1.0 } else { 0.85 } - 0.1 * depth;
                    let s = if self.bird[j] { BIRD } else { self.rgb[j].map(f64::from) };
                    (cr, cg, cb) = (s[0] * dim * 0.9, s[1] * dim * 0.94, s[2] * dim + 0.05);
                    // ripples catching the sky
                    let w = noise(x * 0.12 + t * 0.15, y * 0.9 - t * 0.9);
                    let glint = smooth(0.75, 0.95, w) * 0.14;
                    (cr, cg, cb) = (cr + glint, cg + glint * 0.85, cb + glint * 0.75);
                    // a sparse line of light drifting across the far end
                    let r = y.floor();
                    if r > BASE && r <= BASE + 4.0 {
                        let gl = smooth(0.62, 0.85, noise(x * 0.55 - t * 0.9, (y - 0.5) * 2.7 + t * 0.2)) * (0.5 - 0.08 * (y - 0.5 - BASE));
                        (cr, cg, cb) = (cr + gl, cg + gl * 0.85, cb + gl * 0.6);
                    }
                    // a thin line of light where the far edge meets the plinth
                    if r == BASE {
                        (cr, cg, cb) = (cr + 0.2, cg + 0.16, cb + 0.12);
                    }
                    fade = 0.8 + 0.2 * smooth(102.0, 94.0, y);
                } else if m == Mat::Sky {
                    let ga = f64::from(self.glow_a[k]);
                    let gb = ga * breathe;
                    (cr, cg, cb) = (cr + gb, cg + gb * 0.78, cb + gb * 0.48);
                    if py < CHP {
                        // drifting dawn cloud: gold and rose underneath near the
                        // sun, violet-grey elsewhere
                        let sx = px as f64 + cd;
                        let ix = sx.floor();
                        let fx = sx - ix;
                        let i0 = (ix as usize) % CWP;
                        let (i0, i1) = (py * CWP + i0, py * CWP + (i0 + 1) % CWP);
                        let c = f64::from(self.cloud[i0]) + f64::from(self.cloud[i1] - self.cloud[i0]) * fx;
                        if c > 0.01 {
                            let l = f64::from(self.cloud_lit[i0]) + f64::from(self.cloud_lit[i1] - self.cloud_lit[i0]) * fx;
                            let near = (ga * 1.7).min(1.0);
                            // a mauve body, rose underneath, gold where the sun is close
                            let b = clamp(0.2 + 0.6 * l + near * 0.55 * l);
                            let (kr, kg, kb) = if b < 0.5 {
                                let q = b * 2.0;
                                (mix(0.16, 0.6, q), mix(0.12, 0.34, q), mix(0.3, 0.5, q))
                            } else {
                                let q = (b - 0.5) * 2.0;
                                (mix(0.6, 1.05, q), mix(0.34, 0.8, q), mix(0.5, 0.55, q))
                            };
                            let a = c.min(1.0) * 0.9;
                            (cr, cg, cb) = (mix(cr, kr, a), mix(cg, kg, a), mix(cb, kb, a));
                        }
                    }
                    if y < 24.0 && x > 96.0 && hash(px as f64, py as f64 * 3.0 + 11.0) > 0.995 {
                        // the last few stars, fading where the dawn reaches
                        let (fx, fy) = (px as f64, py as f64);
                        let tw = 0.7 + 0.3 * (t * (1.2 + hash(fx, fy) * 2.0) + hash(fy, fx) * TAU).sin();
                        let s = tw * 0.62 * smooth(24.0, 8.0, y - 0.5) * smooth(96.0, 140.0, x);
                        (cr, cg, cb) = (cr.max(s * 0.85), cg.max(s * 0.85), cb.max(s));
                    }
                    // the sun's disc, softened by the haze
                    let (dx, dy) = (x + 0.5 - SUN.0, y - SUN.1);
                    let dd = dx * dx + dy * dy;
                    if dd < 22.0 {
                        let a = smooth(22.0, 9.0, dd);
                        (cr, cg, cb) = (mix(cr, 1.05, a), mix(cg, 0.98, a), mix(cb, 0.86, a));
                    }
                }
                if self.bird[k] {
                    // birds, dark against the light
                    [cr, cg, cb] = BIRD;
                }

                // the morning haze, drifting
                let hw = f64::from(self.haze_w[k]);
                if hw > 0.0 || m == Mat::Pool {
                    let a = if m == Mat::Pool { 0.12 } else { hw } * hr;
                    let i1 = py * HWP + ((px as f64 + d1) as usize) % HWP;
                    let i2 = py * HWP + ((px as f64 + d2) as usize) % HWP;
                    let hz = a * (0.35 + 0.75 * f64::from(self.haze1[i1]) + 0.45 * f64::from(self.haze2[i2]));
                    (cr, cg, cb) = (mix(cr, 0.9, hz), mix(cg, 0.64, hz), mix(cb, 0.62, hz));
                }

                let c = [mix(ground[0], tone(cr), fade) as f32, mix(ground[1], tone(cg), fade) as f32, mix(ground[2], tone(cb), fade) as f32];
                self.pixels.set(px, py, c);
            }
        }
        self.pixels.draw(grid, env);
    }
}
