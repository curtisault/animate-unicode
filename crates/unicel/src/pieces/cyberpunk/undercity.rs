//! undercity: looking down a vast shaft in the city's underbelly. Walkways
//! ring it level under level, their edge lights drawing green, amber and
//! teal lines that close in on the depths; pipes run down the walls, cables
//! sag across and catwalks bridge it. Down the far wall a cascade of glowing
//! coolant spills from an outlet and falls away into the haze, lighting
//! everything near it acid green, beside a sign in amber tubes. The side
//! walls are server racks, every light blinking to its own beat, and amber
//! beacons turn on the railings, sweeping their light round the walls.
//!
//! Drones drift through the shaft at different depths, their searchlights
//! sweeping through the steam that rises from below, and the depths glow
//! and breathe. Every nine seconds someone welds at a railing and a shower
//! of sparks falls into the shaft; every six and a half the picture
//! glitches: a scanline tears down across the whole view, and bands of it
//! slip sideways and split their colours. The shaft is ray-cast once, when
//! the piece is made (where every pixel lands, what it is and how far off),
//! so a frame only lights it, and everything bright glows into the haze.

use crate::noise::{clamp, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Undercity's meta.
pub static META: Meta = Meta {
    name: "undercity",
    slug: "undercity",
    category: Category::Cyberpunk,
    note: "a shaft under the city, coolant falling into the glowing depths",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // The teal-blacks the shaft is made of first, then its greys, the haze's
    // greens, the acid greens of the coolant, the ambers of the beacons and
    // signs, the teals of the lights, a red, and the whites.
    palette: Some(&[
        "#020706", "#040c0b", "#061210", "#081815", "#0b1f1c", "#0e2723", "#12302b", "#163a34", "#1b453e", "#215149", "#285e55", "#0a0f10", "#121a1b",
        "#1b2528", "#26333a", "#36464c", "#4d6066", "#6c8285", "#2f6e5c", "#3e8570", "#529c82", "#6bb595", "#8ccfaa", "#24470f", "#346b10", "#4a9a12",
        "#6fd11a", "#9dff3a", "#c8ff7a", "#ecffc4", "#1a1208", "#3a2206", "#5c3608", "#8a520a", "#c27a0c", "#ffa516", "#ffc451", "#ffe39a", "#0e4a52",
        "#127080", "#1fa3b0", "#46d6d6", "#a6f4ee", "#7a1410", "#ff3b2e", "#dff7ea", "#ffffff", "#2a1c0e", "#0b0a14", "#14121f", "#93a8a6", "#c2d2cc",
    ]),
    ground: Some("#020706"),
    ..Meta::DEFAULT
};

/// The picture in pixels.
const W: usize = 320;
const H: usize = 160;
/// The focal length in pixels.
const F: f64 = 130.0;
/// How far the camera looks down, in radians.
const PITCH: f64 = 0.98;
/// The camera, near the top of the shaft (y grows downward, z away from us).
const CAM: [f64; 3] = [-1.5, 0.0, 0.0];
/// The shaft's half width, and its near and far walls.
const XW: f64 = 11.0;
const ZN: f64 = -2.0;
const ZF: f64 = 18.0;
/// How far the haze lets you see.
const FOG: f64 = 20.0;
/// The coolant's outlet, on the far wall.
const OUT_X: f64 = 2.6;
const OUT_Y: f64 = 9.0;
/// How often the picture glitches, in seconds.
const GLITCH: f64 = 6.5;

/// What a ray lands on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Void,
    Left,
    Right,
    Far,
    Near,
    Deck,
    Rail,
    Pipe,
    Cable,
}

/// What lights itself in a pixel, each frame.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Glow {
    None,
    /// A rack light: its id, and its colour in the param.
    Led,
    /// The coolant: across it in the param, -1 to 1.
    Coolant,
    /// A sign's stroke: its glyph in the id.
    Sign,
    /// A walkway's edge light: its level in the id, its colour in the param.
    Strip,
    /// A lit window on the far wall.
    Window,
}

/// An axis-aligned box: walkways, rails and posts.
#[derive(Clone, Copy)]
struct Slab {
    mn: [f64; 3],
    mx: [f64; 3],
    mat: Mat,
    level: usize,
}

/// A cylinder: `axis` 0 runs along x, 1 along y, 2 along z; `c` is its
/// centre in the other two axes, in order, and `span` its extent along its own.
#[derive(Clone, Copy)]
struct Cyl {
    axis: usize,
    c: [f64; 2],
    r: f64,
    span: [f64; 2],
}

/// A ray's first hit: how far, the surface normal, what, and which level.
struct Hit {
    t: f64,
    n: [f64; 3],
    mat: Mat,
    level: usize,
}

/// A walkway level: its depth, its edge light's colour, which wall ledges
/// it has, and where its bridge crosses, if it has one.
struct Level {
    y: f64,
    tint: f32,
    ledges: [bool; 3],
    bridge: Option<f64>,
}

/// A drone's flight: its loop's period, centre, swing, and its searchlight's turn.
struct Drone {
    period: f64,
    c: [f64; 3],
    a: [f64; 3],
    phase: f64,
    spin: f64,
}

/// A warning beacon on a railing.
struct Beacon {
    p: [f64; 3],
    spin: f64,
    phase: f64,
    /// Where it shows on screen, and how big.
    at: (f64, f64, f64),
    /// The bearing from it to the camera, in the x–z plane.
    facing: f64,
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    scale(a, 1.0 / dot(a, a).sqrt())
}

/// The ray through the middle of pixel (px, py), not normalised.
fn ray(px: f64, py: f64) -> [f64; 3] {
    let (dx, dy) = ((px + 0.5 - W as f64 / 2.0) / F, (py + 0.5 - H as f64 / 2.0) / F);
    let (s, c) = PITCH.sin_cos();
    [dx, dy * c + s, c - dy * s]
}

/// A point on screen: (x, y, depth along the view), or `None` behind us.
fn project(p: [f64; 3]) -> Option<(f64, f64, f64)> {
    let r = sub(p, CAM);
    let (s, c) = PITCH.sin_cos();
    let depth = r[1] * s + r[2] * c;
    if depth < 0.3 {
        return None;
    }
    let down = r[1] * c - r[2] * s;
    Some((W as f64 / 2.0 + F * r[0] / depth - 0.5, H as f64 / 2.0 + F * down / depth - 0.5, depth))
}

/// Where the coolant falls at depth y: its middle and half width.
fn coolant(y: f64) -> (f64, f64) {
    (OUT_X + 0.25 * (y * 0.15).sin() + 0.1 * (y * 0.41).sin(), 0.75 + 0.016 * (y - OUT_Y).max(0.0))
}

/// The haze at a depth: deep teal near the top, glowing green far below.
fn haze(y: f64) -> [f64; 3] {
    let g = smooth(0.0, 50.0, y);
    let g2 = smooth(20.0, 80.0, y);
    [0.02 + 0.08 * g + 0.45 * g2, 0.06 + 0.26 * g + 0.68 * g2, 0.06 + 0.18 * g + 0.3 * g2]
}

/// Katakana-ish glyphs, five strokes wide and seven tall, a row to a byte.
const GLYPHS: [[u8; 7]; 6] = [
    [0b00100, 0b11111, 0b00101, 0b00101, 0b01001, 0b01001, 0b10110],
    [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111, 0b00000],
    [0b01000, 0b01000, 0b01110, 0b01001, 0b01000, 0b01000, 0b01000],
    [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111, 0b00000],
    [0b00100, 0b11111, 0b00100, 0b00100, 0b00100, 0b01000, 0b10000],
    [0b11111, 0b00001, 0b00110, 0b00100, 0b00100, 0b01000, 0b10000],
];

/// The far wall's sign: its left edge, top, and stroke size.
const SIGN: (f64, f64, f64) = (-8.0, 11.2, 0.32);

/// The slab test: the distance in and the normal, if the ray meets the box.
fn hit_slab(o: [f64; 3], inv: [f64; 3], mn: [f64; 3], mx: [f64; 3], best: f64) -> Option<(f64, [f64; 3])> {
    let (mut t0, mut t1, mut n) = (0.0f64, best, [0.0; 3]);
    for a in 0..3 {
        let (ta, tb) = ((mn[a] - o[a]) * inv[a], (mx[a] - o[a]) * inv[a]);
        let (near, far, sign) = if ta < tb { (ta, tb, -1.0) } else { (tb, ta, 1.0) };
        if near > t0 {
            t0 = near;
            n = [0.0; 3];
            n[a] = sign;
        }
        t1 = t1.min(far);
        if t0 > t1 {
            return None;
        }
    }
    (t0 > 0.0 && t0 < best).then_some((t0, n))
}

/// The cylinder test: the distance in and the normal.
fn hit_cyl(o: [f64; 3], d: [f64; 3], c: &Cyl, best: f64) -> Option<(f64, [f64; 3])> {
    let (i, j) = match c.axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    };
    let (ox, oy) = (o[i] - c.c[0], o[j] - c.c[1]);
    let (a, b, k) = (d[i] * d[i] + d[j] * d[j], ox * d[i] + oy * d[j], ox * ox + oy * oy - c.r * c.r);
    let disc = b * b - a * k;
    if disc < 0.0 || a < 1e-12 {
        return None;
    }
    let t = (-b - disc.sqrt()) / a;
    if t <= 0.0 || t >= best {
        return None;
    }
    let along = o[c.axis] + d[c.axis] * t;
    if along < c.span[0] || along > c.span[1] {
        return None;
    }
    let mut n = [0.0; 3];
    n[i] = (ox + d[i] * t) / c.r;
    n[j] = (oy + d[j] * t) / c.r;
    Some((t, n))
}

/// The shaft, ray-cast: what each pixel is, how it is lit before anything
/// moves, where it is, and the moving things' loops.
pub struct Undercity {
    /// The surface's own light, already dimmed by the haze in front of it.
    surf: Vec<[f32; 3]>,
    /// The haze in front of it.
    fogc: Vec<[f32; 3]>,
    /// How much of the surface shows through the haze.
    vis: Vec<f32>,
    /// Where the ray landed, and how far it went.
    pos: Vec<[f32; 3]>,
    dist: Vec<f32>,
    glow: Vec<Glow>,
    id: Vec<u32>,
    param: Vec<f32>,
    /// Rising steam: a texture that wraps top to bottom.
    steam: Vec<f32>,
    /// Where the steam gathers.
    steam_mask: Vec<f32>,
    drones: Vec<Drone>,
    beacons: Vec<Beacon>,
    /// Where the sparks fall from.
    spark: [f64; 3],
    buf: Vec<[f32; 3]>,
    /// What glows, blurred into a halo round it.
    bloom: Vec<[f32; 3]>,
    spare: Vec<[f32; 3]>,
    pixels: Pixels,
}

/// The steam texture's width: the picture's and some to drift across.
const SW: usize = W + 48;

/// The levels of walkways, from the top down: depth, edge light (0 green,
/// 1 amber, 2 teal), ledges on the left, right and far walls, and where a
/// bridge crosses.
const LEVELS: [(f64, f32, [bool; 3], f64); 14] = [
    (4.0, 1.0, [false, false, true], -1.0),
    (9.5, 0.0, [true, false, true], -1.0),
    (15.5, 2.0, [false, true, true], -1.0),
    (21.0, 1.0, [true, true, true], 7.0),
    (27.5, 0.0, [true, true, false], -1.0),
    (34.0, 2.0, [true, false, true], -1.0),
    (41.0, 1.0, [false, true, true], -1.0),
    (48.5, 0.0, [true, true, true], 9.0),
    (56.0, 1.0, [true, true, true], -1.0),
    (64.0, 2.0, [true, true, true], -1.0),
    (73.0, 0.0, [true, true, true], -1.0),
    (83.0, 1.0, [true, true, true], -1.0),
    (94.0, 2.0, [true, true, true], -1.0),
    (106.0, 0.0, [true, true, true], -1.0),
];

/// The levels of walkways, from the top down.
fn levels() -> Vec<Level> {
    LEVELS.iter().map(|&(y, tint, ledges, b)| Level { y, tint, ledges, bridge: (b >= 0.0).then_some(b) }).collect()
}

/// The walkways, rails and posts, grouped by level with each group's bounds.
fn slabs(levels: &[Level]) -> Vec<(Slab, Vec<Slab>)> {
    let mut groups = vec![];
    for (k, l) in levels.iter().enumerate() {
        let y = l.y;
        let mut g = vec![];
        let mut deck = |mn: [f64; 3], mx: [f64; 3]| g.push(Slab { mn, mx, mat: Mat::Deck, level: k });
        let wl = 2.0 + hash(k as f64, 7.0) * 1.2;
        let wr = 2.0 + hash(k as f64, 8.0) * 1.2;
        if l.ledges[0] {
            deck([-XW, y, ZN], [-XW + wl, y + 0.5, ZF]);
        }
        if l.ledges[1] {
            deck([XW - wr, y, ZN], [XW, y + 0.5, ZF]);
        }
        let (cx, cw) = coolant(y);
        let gap = (cx - cw - 0.5, cx + cw + 0.5);
        if l.ledges[2] {
            if y > OUT_Y {
                deck([-XW, y, ZF - 1.2], [gap.0, y + 0.5, ZF]);
                deck([gap.1, y, ZF - 1.2], [XW, y + 0.5, ZF]);
            } else {
                deck([-XW, y, ZF - 1.2], [XW, y + 0.5, ZF]);
            }
        }
        if let Some(z) = l.bridge {
            deck([-XW, y, z], [XW, y + 0.3, z + 0.9]);
        }
        // the railings: a top rail and posts along each open edge
        let mut rails = vec![];
        let mut rail = |mn: [f64; 3], mx: [f64; 3]| rails.push(Slab { mn, mx, mat: Mat::Rail, level: k });
        let post = 2.6;
        if l.ledges[0] {
            let x = -XW + wl - 0.1;
            rail([x, y - 1.1, ZN], [x + 0.1, y - 1.0, ZF]);
            let mut z = ZN + 0.5;
            while z < ZF {
                rail([x, y - 1.1, z], [x + 0.1, y, z + 0.1]);
                z += post;
            }
        }
        if l.ledges[1] {
            let x = XW - wr;
            rail([x, y - 1.1, ZN], [x + 0.1, y - 1.0, ZF]);
            let mut z = ZN + 1.5;
            while z < ZF {
                rail([x, y - 1.1, z], [x + 0.1, y, z + 0.1]);
                z += post;
            }
        }
        if let Some(z0) = l.bridge {
            for z in [z0, z0 + 0.8] {
                rail([-XW, y - 1.0, z], [XW, y - 0.92, z + 0.1]);
                let mut x = -XW + 1.0 + hash(k as f64, 11.0) * 2.0;
                while x < XW {
                    rail([x, y - 1.0, z], [x + 0.1, y, z + 0.1]);
                    x += post;
                }
            }
        }
        g.extend(rails);
        let mut mn = [f64::MAX; 3];
        let mut mx = [f64::MIN; 3];
        for s in &g {
            for a in 0..3 {
                mn[a] = mn[a].min(s.mn[a]);
                mx[a] = mx[a].max(s.mx[a]);
            }
        }
        if !g.is_empty() {
            groups.push((Slab { mn, mx, mat: Mat::Deck, level: k }, g));
        }
    }
    groups
}

/// The pipes: bundles down the walls, conduits across, and the outlet.
fn pipes() -> Vec<Cyl> {
    let down = [-120.0, 400.0];
    let mut v = vec![];
    for (z, r) in [(1.2, 0.45), (2.2, 0.3), (2.9, 0.3), (9.5, 0.6)] {
        v.push(Cyl { axis: 1, c: [-XW + r + 0.05, z], r, span: down });
    }
    for (z, r) in [(5.0, 0.5), (6.1, 0.35), (12.5, 0.4)] {
        v.push(Cyl { axis: 1, c: [XW - r - 0.05, z], r, span: down });
    }
    for (x, r) in [(-0.4, 0.45), (-1.3, 0.3), (6.5, 0.4)] {
        v.push(Cyl { axis: 1, c: [x, ZF - r - 0.05], r, span: down });
    }
    // conduits across the shaft
    v.push(Cyl { axis: 0, c: [13.0, 7.0], r: 0.7, span: [-XW, XW] });
    v.push(Cyl { axis: 0, c: [13.9, 7.0], r: 0.35, span: [-XW, XW] });
    v.push(Cyl { axis: 0, c: [27.0, 10.5], r: 0.6, span: [-XW, XW] });
    v.push(Cyl { axis: 2, c: [-XW + 0.8, 20.0], r: 0.5, span: [ZN, ZF] });
    // the outlet
    v.push(Cyl { axis: 2, c: [OUT_X, OUT_Y - 0.75], r: 0.75, span: [ZF - 0.9, ZF] });
    v
}

/// The cables: each sags from one wall to the other.
const CABLES: [([f64; 3], [f64; 3], f64); 5] = [
    ([-XW, 4.0, 5.0], [XW, 5.0, 11.0], 4.0),
    ([-XW, 9.0, 13.0], [XW, 7.5, 6.0], 3.0),
    ([-XW, 15.0, 3.0], [XW, 16.0, 14.0], 4.5),
    ([-XW, 26.0, 10.0], [XW, 25.0, 4.0], 3.5),
    ([-XW, 38.0, 6.0], [XW, 40.0, 12.0], 4.0),
];

/// The first hit along a ray from the camera.
fn cast(d: [f64; 3], groups: &[(Slab, Vec<Slab>)], pipes: &[Cyl]) -> Hit {
    let o = CAM;
    let inv = [1.0 / d[0], 1.0 / d[1], 1.0 / d[2]];
    // the shaft's walls
    let mut best = f64::MAX;
    let mut hit = Hit { t: f64::MAX, n: [0.0; 3], mat: Mat::Void, level: 0 };
    let walls =
        [(0, -XW, Mat::Left, [1.0, 0.0, 0.0]), (0, XW, Mat::Right, [-1.0, 0.0, 0.0]), (2, ZF, Mat::Far, [0.0, 0.0, -1.0]), (2, ZN, Mat::Near, [0.0, 0.0, 1.0])];
    for (a, p, mat, n) in walls {
        let t = (p - o[a]) * inv[a];
        if t > 0.0 && t < best {
            best = t;
            hit = Hit { t, n, mat, level: 0 };
        }
    }
    if o[1] + d[1] * best > 400.0 {
        best = (400.0 - o[1]) / d[1];
        hit = Hit { t: best, n: [0.0, -1.0, 0.0], mat: Mat::Void, level: 0 };
    }
    for (bound, g) in groups {
        if hit_slab(o, inv, bound.mn, bound.mx, best).is_none() && !(0..3).all(|a| o[a] >= bound.mn[a] && o[a] <= bound.mx[a]) {
            continue;
        }
        for s in g {
            if let Some((t, n)) = hit_slab(o, inv, s.mn, s.mx, best) {
                best = t;
                hit = Hit { t, n, mat: s.mat, level: s.level };
            }
        }
    }
    for c in pipes {
        if let Some((t, n)) = hit_cyl(o, d, c, best) {
            best = t;
            hit = Hit { t, n, mat: Mat::Pipe, level: 0 };
        }
    }
    hit
}

/// An undercity; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let lv = levels();
    let groups = slabs(&lv);
    let pipes = pipes();
    let mut surf = vec![[0.0f32; 3]; n];
    let mut fogc = vec![[0.0f32; 3]; n];
    let mut vis = vec![0.0f32; n];
    let mut pos = vec![[0.0f32; 3]; n];
    let mut dist = vec![0.0f32; n];
    let mut glow = vec![Glow::None; n];
    let mut id = vec![0u32; n];
    let mut param = vec![0.0f32; n];
    let mut mats = vec![Mat::Void; n];
    let mut normals = vec![[0.0f64; 3]; n];
    let mut levels_at = vec![0usize; n];

    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            let d = norm(ray(px as f64, py as f64));
            let h = cast(d, &groups, &pipes);
            let p = add(CAM, scale(d, h.t));
            mats[k] = h.mat;
            normals[k] = h.n;
            levels_at[k] = h.level;
            pos[k] = [p[0] as f32, p[1] as f32, p[2] as f32];
            dist[k] = h.t as f32;
        }
    }

    // the cables, drawn over what they pass in front of
    for &(a, b, sag) in &CABLES {
        let steps = 900;
        for i in 0..=steps {
            let u = i as f64 / steps as f64;
            let mut p = add(a, scale(sub(b, a), u));
            p[1] += sag * (1.0 - (2.0 * u - 1.0).powi(2));
            let Some((sx, sy, depth)) = project(p) else { continue };
            let w = (0.05 * F / depth).clamp(0.6, 3.0);
            let r = (w / 2.0).floor() as i64;
            for oy in -r..=r.max(0) {
                let (x, y) = (sx.round() as i64, sy.round() as i64 + oy);
                if x < 0 || y < 0 || x >= W as i64 || y >= H as i64 {
                    continue;
                }
                let k = y as usize * W + x as usize;
                let dd = dot(sub(p, CAM), sub(p, CAM)).sqrt();
                if dd < f64::from(dist[k]) {
                    dist[k] = dd as f32;
                    pos[k] = [p[0] as f32, p[1] as f32, p[2] as f32];
                    mats[k] = Mat::Cable;
                    normals[k] = [0.0, -1.0, 0.0];
                }
            }
        }
    }

    for k in 0..n {
        let p = pos[k].map(f64::from);
        let (x, y, z) = (p[0], p[1], p[2]);
        let nn = normals[k];
        let m = mats[k];
        let lvl = levels_at[k];
        // the light: a dim teal from the city above, the green welling up
        // from below, stronger the deeper, and the coolant's glow
        let up = smooth(0.0, 70.0, y);
        let from_below = nn[1].max(0.0) * (0.25 + 1.1 * up) + 0.3 + 0.3 * up;
        let from_above = (-nn[1]).max(0.0) * 0.45 * (-y / 25.0).exp() + 0.55;
        let (ccx, ccw) = coolant(y);
        let dxz = ((x - ccx).abs() - ccw).max(0.0).hypot(ZF - z);
        let cool = (-dxz / 2.2).exp() * smooth(OUT_Y - 2.0, OUT_Y + 2.0, y) * 1.4;
        let mut light = [
            0.18 * from_below + 0.3 * from_above + 0.55 * cool,
            0.55 * from_below + 0.5 * from_above + 1.0 * cool,
            0.36 * from_below + 0.55 * from_above + 0.3 * cool,
        ];
        let mut alb = [0.0f64; 3];
        match m {
            Mat::Void | Mat::Near => {}
            Mat::Left | Mat::Right | Mat::Far => {
                let (u, v) = if m == Mat::Far { (x, y) } else { (z, y) };
                // panels, their seams, grime
                let seam = ((u / 2.4).fract() < 0.05 || (v / 1.7).fract() < 0.06) as u8 as f64;
                let grime = noise(u * 0.8, v * 0.35) * 0.5 + noise(u * 3.0, v * 2.0) * 0.25;
                let base = 0.18 + 0.12 * grime - 0.1 * seam;
                alb = [base * 0.75, base, base * 0.95];
                // which band between levels this is
                let band = lv.iter().position(|l| l.y > v).unwrap_or(lv.len());
                let racks = m != Mat::Far;
                let (top, bot) = (if band == 0 { -5.0 } else { lv[band - 1].y + 0.6 }, if band < lv.len() { lv[band].y - 0.2 } else { 999.0 });
                if racks && v > top && v < bot {
                    // a server rack: dark fronts, and rows of small lights
                    let col = (u / 1.05).floor();
                    let fu = (u / 1.05).fract();
                    alb = [0.07, 0.1, 0.1];
                    let row = (v / 0.24).floor();
                    let fv = (v / 0.24).fract();
                    if fu < 0.08 {
                        alb = [0.2, 0.25, 0.24];
                    } else if fv < 0.12 {
                        alb = [0.04, 0.05, 0.05];
                    } else if hash(col * 3.0 + 1.0, row) > 0.5 {
                        // a unit with its lights in a row, the rack's colour mostly
                        let slot = ((fu - 0.08) / 0.92 * 5.0).floor();
                        let fs = ((fu - 0.08) / 0.92 * 5.0).fract();
                        alb = [0.1, 0.14, 0.14];
                        if fv > 0.35 && fv < 0.75 && fs > 0.2 && fs < 0.7 && slot < 3.0 {
                            glow[k] = Glow::Led;
                            id[k] = (hash(col * 7.0 + slot, row * 3.0 + m as u8 as f64) * 1e6) as u32 + 1;
                            let theme = hash(col, 50.0 + m as u8 as f64);
                            let hc = hash(col + slot * 13.0, row + 5.0);
                            param[k] = if theme < 0.2 {
                                if hc < 0.8 {
                                    1.0
                                } else {
                                    0.0
                                }
                            } else if theme < 0.32 {
                                2.0
                            } else if hc < 0.92 {
                                0.0
                            } else {
                                1.0
                            };
                        }
                    }
                }
                if m == Mat::Far {
                    let (cx, cw) = coolant(v);
                    let a = (u - cx) / cw;
                    if v > OUT_Y - 0.1 && a.abs() < 1.0 {
                        glow[k] = Glow::Coolant;
                        param[k] = a as f32;
                    } else if v > OUT_Y - 0.1 && a.abs() < 2.2 {
                        // the wet wall either side of it
                        alb = [0.04, 0.08, 0.06];
                    } else if v > top && v < bot && v > 3.0 && v < 30.0 {
                        // lit windows in the far wall
                        let (wu, wv) = (u / 1.3, (v - top) / 1.1);
                        if wu.fract() > 0.25 && wu.fract() < 0.75 && wv.fract() > 0.35 && wv.fract() < 0.75 && wv < ((bot - top) / 1.1).floor() {
                            let hw = hash(wu.floor() + 40.0, wv.floor() + band as f64 * 17.0);
                            alb = [0.03, 0.04, 0.04];
                            if hw > 0.55 {
                                glow[k] = Glow::Window;
                                id[k] = (hw * 1e6) as u32;
                                param[k] = if hw > 0.88 { 1.0 } else { 0.0 };
                            }
                        }
                    }
                    // the sign: a dark panel, glyphs in amber tubes
                    let (sx0, sy0, st) = SIGN;
                    let (gu, gv) = ((u - sx0) / st, (v - sy0) / st);
                    if (-1.0..(6.0 * 4.0)).contains(&gu) && (-1.0..8.0).contains(&gv) {
                        alb = [0.02, 0.025, 0.03];
                        let gi = (gu / 6.0).floor();
                        let (cu, cv) = ((gu - gi * 6.0).floor() as i64, gv.floor() as i64);
                        if (0..5).contains(&cu) && (0..7).contains(&cv) && gi >= 0.0 {
                            let g = GLYPHS[(gi as usize) % GLYPHS.len()];
                            if g[cv as usize] >> (4 - cu) & 1 == 1 {
                                glow[k] = Glow::Sign;
                                id[k] = gi as u32;
                            }
                        }
                    }
                }
            }
            Mat::Deck => {
                let top = nn[1] < -0.5;
                let under = nn[1] > 0.5;
                if top {
                    // grating
                    let g = ((x + z) * 4.0).fract() < 0.4;
                    let b = if g { 0.12 } else { 0.07 };
                    alb = [b * 0.85, b, b * 0.95];
                } else if under {
                    alb = [0.08, 0.11, 0.1];
                } else {
                    // the edge: an edge light along its lower half, hazard
                    // stripes along the upper
                    let along = if nn[0].abs() > 0.5 { z } else { x };
                    let fy = (y - lv[lvl].y) / 0.5;
                    if fy > 0.4 {
                        glow[k] = Glow::Strip;
                        id[k] = lvl as u32 * 64 + (along / 3.0).floor().rem_euclid(64.0) as u32;
                        param[k] = lv[lvl].tint;
                        alb = [0.1, 0.12, 0.11];
                    } else if ((along + y) / 0.5).fract() < 0.5 {
                        alb = [0.55, 0.36, 0.06];
                    } else {
                        alb = [0.05, 0.05, 0.04];
                    }
                }
            }
            Mat::Rail => alb = [0.12, 0.15, 0.14],
            Mat::Pipe => {
                let b = 0.2 + 0.08 * noise(x * 0.5 + z, y * 0.3);
                alb = [b * 0.8, b, b * 0.95];
                // a rim of the depths' light along the pipes
                light = light.map(|l| l * 1.2);
            }
            Mat::Cable => alb = [0.04, 0.05, 0.05],
        }
        if m == Mat::Pipe && nn[1].abs() < 0.2 {
            // banded: clamps along the vertical pipes
            if (y / 2.0).fract() < 0.1 {
                alb = alb.map(|c| c * 0.4);
            }
        }
        let fog = if m == Mat::Void { 1.0 } else { 1.0 - (-f64::from(dist[k]) / FOG).exp() };
        let hz = haze(y.min(400.0) * 0.75 + 4.0);
        // the coolant's light in the air near it
        let air = (-dxz / 5.0).exp() * smooth(OUT_Y - 3.0, OUT_Y + 6.0, y) * 0.25 * fog;
        for c in 0..3 {
            let s = alb[c] * light[c];
            surf[k][c] = (s * (1.0 - fog)) as f32;
        }
        fogc[k] = [(hz[0] * fog + air * 0.35) as f32, (hz[1] * fog + air) as f32, (hz[2] * fog + air * 0.3) as f32];
        vis[k] = (1.0 - fog) as f32;
    }

    // Steam: a texture that wraps top to bottom, so it can rise forever.
    let mut steam = vec![0.0f32; SW * H];
    for y in 0..H {
        for x in 0..SW {
            let (fx, fy) = (x as f64, y as f64);
            let q = fbm_wrap(fy * 0.02, fx * 0.05, 3, H as f64 * 0.02);
            steam[y * SW + x] = fbm_wrap(fy * 0.03 + q * 1.2, fx * 0.07 + q * 2.0, 4, H as f64 * 0.03) as f32;
        }
    }
    let mut steam_mask = vec![0.0f32; n];
    let outlet = project([OUT_X, OUT_Y + 30.0, ZF - 0.5]).map_or(170.0, |p| p.0);
    for py in 0..H {
        for px in 0..W {
            let (x, y) = (px as f64, py as f64);
            let low = smooth(30.0, 160.0, y);
            let plume = (-((x - outlet) / 40.0).powi(2)).exp() * smooth(60.0, 150.0, y);
            steam_mask[py * W + px] = (0.25 + 0.6 * low + 0.5 * plume) as f32;
        }
    }

    let drones = vec![
        Drone { period: 36.0, c: [-1.0, 7.5, 8.0], a: [4.5, 1.2, 3.0], phase: 0.3, spin: 0.7 },
        Drone { period: 28.0, c: [3.0, 15.0, 7.0], a: [3.5, 1.5, 3.5], phase: 2.2, spin: -0.9 },
        Drone { period: 44.0, c: [-2.0, 27.0, 8.0], a: [5.0, 2.0, 3.0], phase: 4.1, spin: 0.5 },
    ];

    // beacons at the ends of some walkways
    let mut beacons = vec![];
    for (k, l) in lv.iter().enumerate().take(9) {
        let kf = k as f64;
        if l.ledges[0] && hash(kf, 31.0) > 0.4 {
            let wl = 2.0 + hash(kf, 7.0) * 1.2;
            beacons.push([-XW + wl - 0.05, l.y - 1.3, 1.0 + hash(kf, 32.0) * 10.0]);
        }
        if l.ledges[1] && hash(kf, 33.0) > 0.4 {
            let wr = 2.0 + hash(kf, 8.0) * 1.2;
            beacons.push([XW - wr + 0.05, l.y - 1.3, 2.0 + hash(kf, 34.0) * 10.0]);
        }
    }
    let beacons = beacons
        .into_iter()
        .enumerate()
        .filter_map(|(i, p)| {
            let at = project(p)?;
            let fi = i as f64;
            Some(Beacon { p, spin: 3.2 + hash(fi, 41.0) * 2.0, phase: hash(fi, 42.0) * TAU, at, facing: (CAM[2] - p[2]).atan2(CAM[0] - p[0]) })
        })
        .collect();

    Box::new(Undercity {
        surf,
        fogc,
        vis,
        pos,
        dist,
        glow,
        id,
        param,
        steam,
        steam_mask,
        drones,
        beacons,
        spark: [-XW + 2.0 + hash(1.0, 7.0) * 1.2, LEVELS[1].0 - 0.5, 8.0],
        buf: vec![[0.0; 3]; n],
        bloom: vec![[0.0; 3]; n],
        spare: vec![[0.0; 3]; n],
        pixels: Pixels::new(&META),
    })
}

impl Drone {
    /// Where the drone is at t, and where its searchlight points.
    fn at(&self, t: f64) -> ([f64; 3], [f64; 3]) {
        let w = t / self.period * TAU + self.phase;
        let p = [self.c[0] + self.a[0] * w.sin(), self.c[1] + self.a[1] * (2.0 * w).sin(), self.c[2] + self.a[2] * (w + 0.8).cos()];
        let s = t * self.spin + self.phase * 3.0;
        (p, norm([0.85 * s.cos(), 1.0 + 0.3 * (t * 0.37 + self.phase).sin(), 0.85 * s.sin()]))
    }
}

/// How often someone welds, in seconds.
const SPARKS: f64 = 9.0;

/// The searchlight's half angle's cosine.
const CONE: f64 = 0.955;
/// How far a searchlight reaches.
const REACH: f64 = 16.0;

impl Piece for Undercity {
    #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let breath = 1.0 + 0.1 * (t * TAU / 9.0).sin();
        // the drones this frame: position, light, and the beam on screen
        let drones: Vec<_> = self
            .drones
            .iter()
            .map(|d| {
                let (p, dir) = d.at(t);
                let apex = project(p);
                let end = project(add(p, scale(dir, REACH * 0.8)));
                let dp = dot(sub(p, CAM), sub(p, CAM)).sqrt();
                (p, dir, apex, end, dp)
            })
            .collect();
        let beacons: Vec<_> = self.beacons.iter().map(|b| (b.p, t * b.spin + b.phase)).collect();
        let rise = t * 7.0;
        let rise2 = t * 13.0;
        let sway = 10.0 + 8.0 * (t * 0.15).sin();
        let coolant_t = t * 7.5;
        let flick = |id: u32, rate: f64| hash(f64::from(id), (t * rate).floor());

        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                let p = self.pos[k].map(f64::from);
                let vis = f64::from(self.vis[k]);
                let dk = f64::from(self.dist[k]);
                let s = self.surf[k];
                let fc = self.fogc[k];
                let mut c = [f64::from(s[0]), f64::from(s[1]), f64::from(s[2])];
                let deep = smooth(30.0, 120.0, p[1]);
                let bk = 1.0 + (breath - 1.0) * deep;
                for ch in 0..3 {
                    c[ch] += f64::from(fc[ch]) * bk;
                }

                // what lights itself
                let mut e = [0.0f64; 3];
                match self.glow[k] {
                    Glow::None => {}
                    Glow::Led => {
                        let id = self.id[k];
                        let rate = 0.6 + hash(f64::from(id), 3.0) * 5.0;
                        let on = flick(id, rate) > 0.45;
                        let b = if on { 1.0 } else { 0.06 };
                        e = match self.param[k] as u8 {
                            0 => [0.35 * b, 1.0 * b, 0.25 * b],
                            1 => [1.0 * b, 0.62 * b, 0.1 * b],
                            _ => [0.2 * b, 0.85 * b, 0.9 * b],
                        };
                    }
                    Glow::Coolant => {
                        let a = f64::from(self.param[k]);
                        let v = p[1];
                        let core = (1.0 - a * a).max(0.0).sqrt();
                        let streak = noise(a * 5.0 + 3.0, v * 0.35 - coolant_t * 0.32);
                        let fine = noise(a * 11.0 + 20.0, v * 1.3 - coolant_t * 1.1);
                        let b = core * (0.35 + 0.75 * streak) + 0.15;
                        let foam = smooth(0.62, 0.85, fine * 0.6 + streak * 0.5) * core;
                        let head = smooth(OUT_Y + 1.5, OUT_Y, v) * 0.6;
                        e = [0.3 * b + 0.7 * foam + head * 0.5, 0.95 * b + 0.9 * foam + head, 0.12 * b + 0.65 * foam + head * 0.4];
                    }
                    Glow::Sign => {
                        let gi = self.id[k];
                        // one glyph stutters now and then
                        let bad = gi == 2 && flick(7, 9.0) > 0.6 && (t % 5.0) < 1.6;
                        let b = if bad { 0.15 } else { 0.92 + 0.08 * (t * 40.0).sin() };
                        e = [1.0 * b, 0.6 * b, 0.12 * b];
                    }
                    Glow::Strip => {
                        let id = self.id[k];
                        let dead = hash(f64::from(id), 77.0) > 0.93 && flick(id, 11.0) > 0.4;
                        let b = if dead { 0.1 } else { 1.0 };
                        e = match self.param[k] as u8 {
                            0 => [0.45 * b, 1.0 * b, 0.2 * b],
                            1 => [1.0 * b, 0.6 * b, 0.08 * b],
                            _ => [0.2 * b, 0.8 * b, 0.85 * b],
                        };
                    }
                    Glow::Window => {
                        let id = self.id[k];
                        let b = 0.5 + 0.15 * flick(id, 0.7);
                        e = if self.param[k] > 0.5 { [0.15 * b, 0.6 * b, 0.6 * b] } else { [0.85 * b, 0.5 * b, 0.15 * b] };
                    }
                }
                // lit surfaces: the beacons' sweep and the searchlights' pools
                let mut l = [0.0f64; 3];
                for &(b, a) in &beacons {
                    let dv = sub(p, b);
                    let d2 = dot(dv, dv);
                    if d2 < 36.0 && d2 > 0.01 {
                        let bearing = dv[2].atan2(dv[0]);
                        let sweep = (a - bearing).cos().max(0.0).powi(6);
                        let f = (1.0 - d2.sqrt() / 6.0).powi(2) * (sweep * 1.6 + 0.08);
                        l[0] += f;
                        l[1] += f * 0.55;
                        l[2] += f * 0.08;
                    }
                }
                for &(dp, dir, _, _, _) in &drones {
                    let dv = sub(p, dp);
                    let dd = dot(dv, dv).sqrt();
                    if dd < REACH * 1.6 && dd > 0.1 {
                        let cs = dot(dv, dir) / dd;
                        if cs > CONE - 0.02 {
                            let f = smooth(CONE - 0.02, CONE + 0.02, cs) * 3.0 / (1.0 + dd * dd * 0.02);
                            l[0] += f * 0.8;
                            l[1] += f;
                            l[2] += f * 0.9;
                        }
                    }
                }
                let ev = vis.sqrt();
                for ch in 0..3 {
                    c[ch] += e[ch] * ev + l[ch] * 0.6 * vis;
                }
                let mut bl = [(e[0] * ev) as f32, (e[1] * ev) as f32, (e[2] * ev) as f32];

                // steam, rising at two speeds, lit by the depths
                let (x, y) = (px as f64, py as f64);
                let s1 = self.steam_at(x + sway, y + rise);
                let s2 = self.steam_at(x * 0.7 + 30.0 - sway * 0.5, y * 0.8 + rise2);
                let dens = (smooth(0.55, 0.8, s1) * 0.6 + smooth(0.58, 0.85, s2) * 0.45) * f64::from(self.steam_mask[k]);
                let sl = 0.12 + 0.2 * smooth(40.0, 160.0, y);
                c[0] += dens * sl * 0.45;
                c[1] += dens * sl;
                c[2] += dens * sl * 0.75;

                // the searchlight beams through the air, brighter in steam
                for &(dp, dir, apex, end, _) in &drones {
                    let (Some(a), Some(b)) = (apex, end) else { continue };
                    let (vx, vy) = (x - a.0, y - a.1);
                    let (sx, sy) = (b.0 - a.0, b.1 - a.1);
                    let len2 = sx * sx + sy * sy;
                    if len2 < 1.0 {
                        continue;
                    }
                    let u = (vx * sx + vy * sy) / len2;
                    if !(0.0..1.0).contains(&u) {
                        continue;
                    }
                    let (qx, qy) = (vx - u * sx, vy - u * sy);
                    let perp = (qx * qx + qy * qy).sqrt();
                    let r_end = REACH * 0.8 * (1.0 - CONE * CONE).sqrt() / CONE * F / b.2;
                    let width = 0.6 + u * r_end;
                    if perp > width * 2.0 {
                        continue;
                    }
                    // hidden where something stands in front of the beam
                    let bp = sub(add(dp, scale(dir, REACH * 0.8 * u)), CAM);
                    if dk * dk < dot(bp, bp) {
                        continue;
                    }
                    let f = (1.0 - u).powf(1.3) * (-(perp / width).powi(2)).exp() * (0.45 + 1.2 * dens) * 1.3;
                    c[0] += f * 0.75;
                    c[1] += f * 0.95;
                    c[2] += f * 0.85;
                }

                // the beacons themselves, and their glow in the air
                for b in &self.beacons {
                    let (bx, by, bd) = b.at;
                    let (dx, dy) = (x - bx, y - by);
                    let r2 = dx * dx + dy * dy;
                    if r2 > 400.0 || dk + 0.5 < (bd / PITCH.cos().max(0.3)).min(bd * 1.4) && r2 > 4.0 {
                        continue;
                    }
                    let a = t * b.spin + b.phase;
                    let flash = (a - b.facing).cos().max(0.0).powi(8);
                    let size = (0.25 * F / bd).max(1.0);
                    let core = smooth(size + 0.6, size * 0.4, r2.sqrt());
                    let halo = (-r2.sqrt() / (2.0 + 4.0 * flash)).exp() * (0.15 + 0.7 * flash);
                    let g = core * (0.5 + 0.8 * flash) + halo;
                    c[0] += g;
                    c[1] += g * 0.58;
                    c[2] += g * 0.1;
                    bl[0] += (core * flash) as f32;
                    bl[1] += (core * flash * 0.58) as f32;
                }
                self.buf[k] = [c[0] as f32, c[1] as f32, c[2] as f32];
                self.bloom[k] = bl;
            }
        }

        // the drones themselves, over everything behind them
        for &(dp, _, apex, _, ddrone) in &drones {
            let Some((sx, sy, depth)) = apex else { continue };
            let sc = F / depth;
            let hw = 0.6 * sc;
            let hh = 0.17 * sc;
            let x0 = (sx - hw - 2.0).floor().max(0.0) as usize;
            let x1 = ((sx + hw + 2.0).ceil() as usize).min(W - 1);
            let y0 = (sy - hh * 2.0 - 2.0).floor().max(0.0) as usize;
            let y1 = ((sy + hh * 2.0 + 2.0).ceil() as usize).min(H - 1);
            let blink = (t * 1.7 + dp[0]).fract() < 0.15;
            for y in y0..=y1 {
                for x in x0..=x1 {
                    let k = y * W + x;
                    if f64::from(self.dist[k]) < ddrone {
                        continue;
                    }
                    let (dx, dy) = ((x as f64 + 0.5 - sx - 0.5) / hw, (y as f64 + 0.5 - sy - 0.5) / hh);
                    let body = dx * dx + dy * dy < 1.0 && dy > -0.9;
                    let rotor = dy > -2.2 && dy < -1.4 && (dx.abs() > 0.55 && dx.abs() < 1.25);
                    let mut c = None;
                    if body || rotor {
                        let rim = smooth(0.2, 1.0, dy) * 0.12;
                        c = Some([0.03 + rim * 0.4, 0.04 + rim, 0.045 + rim * 0.7]);
                    }
                    // the lamp, under the nose, and the nav lights
                    if dy > 0.0 && dy < 1.4 && dx.abs() < 0.22 + 0.5 / hw {
                        c = Some([0.95, 1.0, 0.92]);
                    }
                    if (dx.abs() - 1.0).abs() < 0.6 / hw.max(1.0) + 0.12 && dy.abs() < 0.6 {
                        let on = if dx < 0.0 { blink } else { !blink };
                        c = Some(if dx < 0.0 {
                            if on {
                                [1.0, 0.25, 0.18]
                            } else {
                                [0.3, 0.06, 0.05]
                            }
                        } else if on {
                            [0.5, 1.0, 0.3]
                        } else {
                            [0.1, 0.25, 0.08]
                        });
                    }
                    if let Some(cc) = c {
                        self.buf[k] = cc.map(|v: f64| v as f32);
                        // the lights glow
                        if cc[0] + cc[1] > 0.6 {
                            self.bloom[k] = cc.map(|v: f64| v as f32 * 1.5);
                        }
                    }
                }
            }
        }

        // now and then, sparks: someone welding at a railing, a shower
        // falling into the shaft
        let phase = (t + 0.8).rem_euclid(SPARKS);
        let burst = ((t + 0.8) / SPARKS).floor();
        if phase < 4.0 {
            let src = self.spark;
            for j in 0..140 {
                let jf = f64::from(j);
                let born = hash(jf, burst + 3.0) * 2.4;
                let age = phase - born;
                if !(0.0..1.6).contains(&age) {
                    continue;
                }
                let v = [0.5 + 3.0 * hash(jf, burst + 4.0), -2.0 + 2.4 * hash(jf, burst + 5.0), -2.0 + 4.0 * hash(jf, burst + 6.0)];
                let at = |a: f64| [src[0] + v[0] * a, src[1] + v[1] * a + 4.9 * a * a, src[2] + v[2] * a];
                let (Some(p0), Some(p1)) = (project(at(age)), project(at((age - 0.07).max(0.0)))) else { continue };
                let heat = 1.0 - age / 1.6;
                let col = [1.0f32, (0.45 + 0.5 * heat) as f32, (0.1 + 0.6 * heat * heat) as f32].map(|c| c * (0.6 + 0.9 * heat as f32));
                let dd = dot(sub(at(age), CAM), sub(at(age), CAM)).sqrt();
                let steps = ((p0.0 - p1.0).abs().max((p0.1 - p1.1).abs()).ceil() as usize).max(1);
                for i in 0..=steps {
                    let f = i as f64 / steps as f64;
                    let (x, y) = (mix(p0.0, p1.0, f).round(), mix(p0.1, p1.1, f).round());
                    if x < 0.0 || y < 0.0 || x >= W as f64 || y >= H as f64 {
                        continue;
                    }
                    let k = y as usize * W + x as usize;
                    if f64::from(self.dist[k]) < dd {
                        continue;
                    }
                    let fade = 1.0 - f as f32 * 0.6;
                    self.buf[k] = col.map(|c| c * fade);
                    self.bloom[k] = col.map(|c| c * fade * 0.8);
                }
            }
            // the torch itself, while the burst lasts
            if phase < 2.4 {
                if let Some((x, y, _)) = project(src) {
                    let fl = (0.7 + 0.3 * hash(t * 15.0, 9.0)) as f32;
                    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                        let (x, y) = (x.round() as usize + dx, y.round() as usize + dy);
                        if x < W && y < H {
                            self.buf[y * W + x] = [fl, fl, fl * 0.9];
                            self.bloom[y * W + x] = [2.0 * fl, 2.0 * fl, 2.2 * fl];
                        }
                    }
                }
            }
        }

        // the glow round everything lit: the bloom blurred twice, added back
        for _ in 0..2 {
            blur(&mut self.bloom, &mut self.spare, 4);
        }
        for (b, g) in self.buf.iter_mut().zip(&self.bloom) {
            for ch in 0..3 {
                b[ch] += g[ch] * 0.7;
            }
        }

        // the glitch: a tear runs down the picture, bands slip and split
        let cycle = (t / GLITCH).floor();
        let since = t - cycle * GLITCH - GLITCH * 0.55;
        let tearing = (0.0..0.6).contains(&since);
        let frame_no = (t * 15.0).round();
        // and a smaller slip between, two frames long
        let blip = (0.0..0.14).contains(&(t - cycle * GLITCH - GLITCH * 0.15));
        let surge = if tearing && hash(frame_no, 13.0) > 0.5 { 1.25 } else { 1.0 };
        for py in 0..H {
            let mut shift = 0.0;
            let mut split = 0.0;
            let mut line = 0.0;
            if tearing {
                let ty = since / 0.6 * (H as f64 + 20.0) - 10.0;
                let d = py as f64 - ty;
                if d.abs() < 1.0 {
                    line = 1.0;
                }
                if d < 0.0 && d > -14.0 {
                    shift = (1.0 + d / 14.0) * (hash(cycle, 5.0) - 0.5) * 30.0;
                    split = 2.0;
                }
                for j in 0..3 {
                    let bj = j as f64;
                    let y0 = hash(cycle * 3.0 + bj, frame_no) * H as f64;
                    let h = 1.0 + hash(bj, cycle + frame_no) * 7.0;
                    if (py as f64) >= y0 && (py as f64) < y0 + h {
                        shift += (hash(frame_no + bj, cycle) - 0.5) * 50.0;
                        split = 3.0;
                    }
                }
            }
            if blip {
                let y0 = hash(cycle, 17.0) * (H as f64 - 12.0);
                if (py as f64) >= y0 && (py as f64) < y0 + 3.0 + hash(cycle, 18.0) * 6.0 {
                    shift = (hash(cycle, 19.0) - 0.5) * 36.0;
                    split = 2.0;
                }
            }
            for px in 0..W {
                let sample = |dx: f64| {
                    let x = (px as f64 - shift + dx).round().rem_euclid(W as f64) as usize;
                    self.buf[py * W + x]
                };
                let mut c = if shift == 0.0 && split == 0.0 {
                    self.buf[py * W + px]
                } else {
                    let (r, g, b) = (sample(split), sample(0.0), sample(-split));
                    [r[0], g[1], b[2]]
                };
                if line > 0.0 {
                    let n = hash(px as f64, frame_no) as f32;
                    c = [0.5 + 0.5 * n, 1.0, 0.4 + 0.4 * n];
                }
                let c = c.map(|v| clamp(f64::from(v) * surge) as f32);
                self.pixels.set(px, py, c);
            }
        }
        self.pixels.draw(grid, env);
    }
}

/// A box blur `r` pixels each way, across then down, in place (`spare`
/// is scratch the same size).
fn blur(img: &mut [[f32; 3]], spare: &mut [[f32; 3]], r: usize) {
    let norm = 1.0 / (2 * r + 1) as f32;
    for y in 0..H {
        let row = &img[y * W..(y + 1) * W];
        let mut acc = [0.0f32; 3];
        for x in 0..=r {
            for ch in 0..3 {
                acc[ch] += row[x.min(W - 1)][ch];
            }
        }
        for ch in 0..3 {
            acc[ch] += row[0][ch] * r as f32;
        }
        for x in 0..W {
            spare[y * W + x] = acc.map(|a| a * norm);
            let (add, sub) = (row[(x + r + 1).min(W - 1)], row[x.saturating_sub(r)]);
            for ch in 0..3 {
                acc[ch] += add[ch] - sub[ch];
            }
        }
    }
    for x in 0..W {
        let mut acc = [0.0f32; 3];
        for y in 0..=r {
            for ch in 0..3 {
                acc[ch] += spare[y.min(H - 1) * W + x][ch];
            }
        }
        for ch in 0..3 {
            acc[ch] += spare[x][ch] * r as f32;
        }
        for y in 0..H {
            img[y * W + x] = acc.map(|a| a * norm);
            let (add, sub) = (spare[(y + r + 1).min(H - 1) * W + x], spare[y.saturating_sub(r) * W + x]);
            for ch in 0..3 {
                acc[ch] += add[ch] - sub[ch];
            }
        }
    }
}

impl Undercity {
    /// The steam's density at a pixel, wrapping top to bottom.
    fn steam_at(&self, x: f64, y: f64) -> f64 {
        let xi = (x.max(0.0) as usize).min(SW - 1);
        let yi = (y.floor() as i64).rem_euclid(H as i64) as usize;
        f64::from(self.steam[yi * SW + xi])
    }
}
