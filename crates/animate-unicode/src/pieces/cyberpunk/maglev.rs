//! maglev: a canyon of towers at night, seen from a walkway seven metres
//! above a wet street. An elevated guideway, two lines on one beam, comes
//! across the cross street in front of us and turns a quarter circle into the
//! canyon, running away between the towers to an arcology lost in the smog.
//! Every six and a half seconds a train takes the curve, in toward us and
//! out again in turn: a white glare far down the canyon that grows into a
//! long chrome body, its lit windows smeared by speed, a cyan stripe and a
//! cyan glow under it, sparks dropping from the beam, a blur at its tail, and
//! its light washing over the tower faces and the street below. The rail
//! charges up ahead of it and fades behind; between trains it hums with a
//! slow pulse. Blade signs flicker (one is failing), a holo billboard on the
//! nearest tower watches with a huge eye and swaps it for a slogan with a
//! glitch, steam rises from vents, cars crawl along the canyon floor, a
//! searchlight sweeps the cloud, and rain falls past the walkway's rail onto
//! a street that mirrors all of it.
//!
//! The scene is traced once, per pixel, from where we stand: the towers,
//! pillars and signs as boxes, and the guideway and the trains' envelope
//! swept along the curve slice by slice, each pixel's ray clipped exactly
//! against each slice's piece of it. What a ray passes through of each
//! line's envelope is kept as a run of s, so a frame only has to ask which
//! stretch each train fills to know which pixels it covers, nose and tail
//! included. The wet street mirrors each pixel above it onto the pixel that
//! sees it reflected, also worked out once, and a train is mirrored as it is
//! drawn. A frame then shades from these: the haze by depth, the trains'
//! light by distance, the street's ripples, the rain's three layers, a glow
//! round everything bright. In one ink the darks are pressed darker so the
//! lights carry the picture.

use crate::noise::{fbm, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::FRAC_PI_2;

/// Maglev's meta.
pub static META: Meta = Meta {
    name: "maglev",
    slug: "maglev",
    category: Category::Cyberpunk,
    note: "a maglev train tearing round a curve through a canyon of neon towers",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    palette: Some(&[
        // the night's near-blacks, violets and navies
        "#040309", "#08060f", "#0c0a17", "#110e20", "#17122b", "#1e1838", "#271f47", "#322858", "#40346c", "#544684", "#0a0f1e", "#101a30", "#182746",
        "#22385e", "#33507c", // the smog's plums and magentas
        "#2a0f2e", "#401442", "#5c1c5c", "#82267a", "#ad3192", "#d93aa6", "#ff52b8", "#ff94d2", "#ffd2ec", // cyans
        "#05262e", "#0a4452", "#0f6878", "#1694a6", "#22c4d6", "#5ae6f0", "#b4f8fc", // acid green
        "#2c4a08", "#5c8c10", "#9cd61e", "#d6ff62", // ambers and reds
        "#3e2006", "#7a420c", "#c47814", "#ffae36", "#ffdc96", "#4e0a12", "#a01826", "#f23440", // silvers and whites
        "#23263a", "#3a3e56", "#5a6080", "#868eab", "#b6bdd4", "#e2e7f4", "#ffffff", "#fff0d2",
        // the greys between: wet asphalt, steam, chrome in cyan light
        "#14141f", "#1f1e2e", "#2c2a3f", "#3d2a4a", "#56385e", "#2a1810", "#0c1c26", "#163444", "#4f7890", "#8fb4c8",
    ]),
    ground: Some("#040309"),
    ..Meta::DEFAULT
};

/// The picture in pixels.
const W: usize = 320;
const H: usize = 160;
/// The lens: focal length in pixels, the vanishing point, the horizon's row.
const F: f64 = 150.0;
const VX: f64 = 160.0;
const HY: f64 = 96.0;
/// Where we stand: on a walkway seven metres up, right of the canyon's middle.
const CX: f64 = 9.0;
const EYE: f64 = 7.0;
/// The canyon: half its width, where its blocks begin, where it ends.
const HW: f64 = 15.0;
const Z0: f64 = 30.0;
const ZEND: f64 = 950.0;
/// The guideway in plan: a straight across the near cross street from XL,
/// a quarter turn of radius RAD, then straight down the canyon at x = TX.
const TX: f64 = -2.0;
const RAD: f64 = 24.0;
const ZN: f64 = 18.0;
const XL: f64 = -70.0;
const LN: f64 = TX - RAD - XL;
const LA: f64 = RAD * FRAC_PI_2;
const S_END: f64 = 1250.0;
/// The beam's top, depth and half width; the train's floor, roof and half width.
const YR: f64 = 10.0;
const BH: f64 = 1.5;
const BW: f64 = 0.9;
/// Half the distance between the two lines.
const LANE: f64 = 2.1;
const TB: f64 = 10.3;
const TT: f64 = 13.6;
const TW: f64 = 1.7;
/// The train: its length, its speed, how often one comes, the blur it leaves.
const LEN: f64 = 110.0;
const SPEED: f64 = 75.0;
const PERIOD: f64 = 13.0;
const GHOST: f64 = 16.0;
/// How far the smog lets us see.
const HAZE: f64 = 280.0;
/// The walkway's rail: its top row.
const RAIL: usize = 146;

const MAGENTA: [f64; 3] = [1.0, 0.2, 0.66];
const CYAN: [f64; 3] = [0.15, 0.9, 1.0];
const AMBER: [f64; 3] = [1.0, 0.62, 0.14];
const ACID: [f64; 3] = [0.62, 1.0, 0.16];

/// The guideway's centre line at `s`: x, z and the unit tangent.
fn track(s: f64) -> (f64, f64, f64, f64) {
    if s < LN {
        (XL + s, ZN, 1.0, 0.0)
    } else if s < LN + LA {
        let th = -FRAC_PI_2 + (s - LN) / RAD;
        (TX - RAD + RAD * th.cos(), ZN + RAD + RAD * th.sin(), -th.sin(), th.cos())
    } else {
        (TX, ZN + RAD + (s - LN - LA), 0.0, 1.0)
    }
}

/// A line's centre at `s`: the guideway's, moved out to one side (lane 1 is ours).
fn on_lane(s: f64, lane: u8) -> (f64, f64) {
    let (x, z, tx, tz) = track(s);
    let off = LANE * (f64::from(lane) * 2.0 - 1.0);
    (x + tz * off, z - tx * off)
}

/// The ray through pixel (px, py), its z component 1, so a hit's t is its depth.
fn ray(px: f64, py: f64) -> [f64; 3] {
    [(px + 0.5 - VX) / F, (HY - py - 0.5) / F, 1.0]
}

/// Where a point lands on the picture.
fn project(x: f64, y: f64, z: f64) -> (f64, f64) {
    (VX + F * (x - CX) / z, HY - F * (y - EYE) / z)
}

/// Blocky letters on a 5 × 5 grid, made-up kanji.
const GLYPHS: [&[u8; 29]; 10] = [
    b"##### ..#.. .###. ..#.. #####",
    b"#.#.# ##### ..#.. .#.#. #...#",
    b"..#.. ##### #.#.# ##### ..#..",
    b".###. .#.#. .###. .#.#. .###.",
    b"#...# .#.#. ..#.. .#.#. #...#",
    b"##.## .#.#. ##### .#.#. ##.##",
    b".#... ##### .#.#. .#.#. ##.##",
    b"..#.. .###. #.#.# ..#.. .#.#.",
    b"##### #...# ##### #...# #...#",
    b"#.### #.#.. ##### ..#.# ###.#",
];

/// Whether glyph `g` is inked at grid cell (col, row).
fn glyph(g: usize, col: i64, row: i64) -> bool {
    (0..5).contains(&col) && (0..5).contains(&row) && GLYPHS[g % GLYPHS.len()][row as usize * 6 + col as usize] == b'#'
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Ground,
    Wall,
    Sign,
    Holo,
    Beam,
    Pillar,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bld(u8),
    Pillar,
    Sign(u8),
    Holo,
}

/// An axis-aligned box: a block, a pillar, a sign board.
#[derive(Clone, Copy)]
struct Bx {
    lo: [f64; 3],
    hi: [f64; 3],
    kind: Kind,
}

/// A blade sign: a board standing out from a wall, facing us, lettered top to bottom.
struct Blade {
    x: (f64, f64),
    y: (f64, f64),
    z: f64,
    col: [f64; 3],
}

const BLADES: [Blade; 8] = [
    Blade { x: (-HW, -HW + 2.4), y: (3.0, 17.0), z: 40.0, col: MAGENTA },
    Blade { x: (-HW, -HW + 2.2), y: (7.0, 25.0), z: 70.0, col: CYAN },
    Blade { x: (-HW, -HW + 2.2), y: (2.0, 20.0), z: 112.0, col: AMBER },
    Blade { x: (-HW - 2.0, -HW + 0.4), y: (12.0, 36.0), z: 176.0, col: ACID },
    Blade { x: (HW - 2.2, HW), y: (9.0, 22.0), z: 52.0, col: AMBER },
    Blade { x: (HW - 2.2, HW), y: (4.0, 26.0), z: 92.0, col: MAGENTA },
    Blade { x: (HW - 2.2, HW), y: (14.0, 40.0), z: 150.0, col: CYAN },
    // on the near tower's corner, the tallest
    Blade { x: (HW + 0.3, HW + 2.9), y: (1.0, 30.0), z: Z0 - 0.4, col: ACID },
];

/// The holo billboard on the near tower, in x and y.
const HOLO: ((f64, f64), (f64, f64)) = ((19.5, 34.0), (5.0, 17.0));

/// Where a ray enters a box, and the axis of the face it enters by.
fn ray_box(o: &[f64; 3], d: &[f64; 3], b: &Bx) -> Option<(f64, usize)> {
    let (mut t0, mut t1, mut ax) = (0.0f64, f64::MAX, 3);
    for i in 0..3 {
        if d[i].abs() < 1e-12 {
            if o[i] < b.lo[i] || o[i] > b.hi[i] {
                return None;
            }
            continue;
        }
        let (mut a, mut c) = ((b.lo[i] - o[i]) / d[i], (b.hi[i] - o[i]) / d[i]);
        if a > c {
            std::mem::swap(&mut a, &mut c);
        }
        if a > t0 {
            t0 = a;
            ax = i;
        }
        t1 = t1.min(c);
        if t0 > t1 {
            return None;
        }
    }
    (ax < 3).then_some((t0, ax))
}

/// What a pixel sees, fixed: its material, depth, position, which way it
/// faces (0 toward us, 1 east, 2 west, 3 up, 4 nowhere), what it belongs
/// to, and where on it.
#[derive(Clone, Copy)]
struct Hit {
    mat: Mat,
    depth: f32,
    pos: [f32; 3],
    face: u8,
    id: u16,
    a: f32,
    b: f32,
}

/// A stretch of the train's swept envelope that one pixel's ray passes
/// through: the slices at either end of it, and the nearest.
#[derive(Clone, Copy)]
struct Run {
    lane: u8,
    last: u32,
    s0: f32,
    s1: f32,
    d0: f32,
    d1: f32,
    u0: f32,
    v0: f32,
    u1: f32,
    v1: f32,
    sn: f32,
    dn: f32,
    un: f32,
    vn: f32,
}

/// A train on the line: its nose, which way it runs (+1 away from us), the
/// stretch it fills, its line (0 outbound, 1 inbound) and its number.
#[derive(Clone, Copy)]
struct Train {
    head: f64,
    dir: f64,
    lo: f64,
    hi: f64,
    lane: u8,
    cycle: f64,
}

/// The trains on the line at `t`: every PERIOD seconds one comes in round
/// the curve toward us and, half a period later, one goes out.
fn trains(t: f64) -> Vec<Train> {
    let mid = LN + LA * 0.5;
    let cyc = (t / PERIOD).floor();
    let mut out = vec![];
    for k in [cyc - 1.0, cyc, cyc + 1.0] {
        for lane in 0..2u8 {
            // at t = 0 an inbound one is half round the curve
            let (dir, tc) = if lane == 1 { (-1.0, k * PERIOD - 0.57) } else { (1.0, k * PERIOD + PERIOD * 0.5) };
            let head = mid + dir * SPEED * (t - tc);
            let tail = head - dir * LEN;
            let (lo, hi) = (head.min(tail), head.max(tail));
            if hi + GHOST > 0.0 && lo - GHOST < S_END {
                out.push(Train { head, dir, lo, hi, lane, cycle: k * 2.0 + f64::from(lane) });
            }
        }
    }
    out
}

/// The maglev scene: what each pixel sees and its fixed colour and haze,
/// the envelope runs, the wet street's mirror, and the picture.
pub struct Maglev {
    hit: Vec<Hit>,
    base: Vec<[f32; 3]>,
    fog: Vec<f32>,
    /// A sign's lettering, 0–1, or how wet the street is.
    ink: Vec<f32>,
    runs: Vec<Run>,
    run_at: Vec<u32>,
    /// Which pixel the wet street mirrors here, and how far that is.
    mirror: Vec<u32>,
    mirror_depth: Vec<f32>,
    beacons: Vec<(usize, usize, f64)>,
    buf: Vec<[f32; 3]>,
    tref: Vec<[f32; 3]>,
    tref_depth: Vec<f32>,
    tref_stamp: Vec<u32>,
    stamp: u32,
    bloom: Vec<[f32; 3]>,
    tmp: Vec<[f32; 3]>,
    pixels: Pixels,
}

/// The smog's colour on row `py`: violet overhead, a magenta glow low down.
fn smog(py: f64) -> [f64; 3] {
    let g = (-((py - (HY - 12.0)) / 30.0).powi(2)).exp();
    [0.06 + 0.26 * g, 0.04 + 0.07 * g, 0.12 + 0.22 * g]
}

/// The bloom's grid: a quarter of the picture each way.
const GW: usize = W / 4;
const GH: usize = H / 4;

/// A maglev scene; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let o = [CX, EYE, 0.0];
    let mut boxes: Vec<Bx> = vec![];
    // The canyon's blocks, each side a row of towers of their own heights,
    // some set back, with alleys between.
    for (si, side) in [-1.0f64, 1.0].into_iter().enumerate() {
        let mut z = Z0;
        let mut i = 0.0;
        while z < ZEND {
            let r = |k: f64| hash(i * 7.0 + k, si as f64 * 31.0 + 5.0);
            let len = 14.0 + 22.0 * r(1.0);
            let set = if i == 0.0 { 0.0 } else { (r(2.0) * 4.0).floor() };
            let h = if i == 0.0 { 160.0 } else { 45.0 + 150.0 * r(3.0) * r(3.0).sqrt() + 40.0 * r(6.0) };
            let inner = side * (HW + set);
            let outer = if i == 0.0 { side * 300.0 } else { side * (HW + set + 40.0) };
            boxes.push(Bx { lo: [inner.min(outer), 0.0, z], hi: [inner.max(outer), h, z + len], kind: Kind::Bld((i as usize * 2 + si) as u8) });
            z += len + if r(4.0) < 0.3 { 3.0 + 5.0 * r(5.0) } else { 0.0 };
            i += 1.0;
        }
    }
    // the skyline behind, and the arcology that closes the canyon
    let mut beacons = vec![];
    for i in 0..18 {
        let fi = i as f64;
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let x = side * (30.0 + 110.0 * hash(fi, 41.0));
        let z = 150.0 + 600.0 * hash(fi, 42.0);
        let w = 20.0 + 30.0 * hash(fi, 43.0);
        let top = 180.0 + 260.0 * hash(fi, 44.0);
        boxes.push(Bx { lo: [x - w, 0.0, z], hi: [x + w, top, z + w], kind: Kind::Bld(200 + i as u8) });
        let (bx, by) = project(x, top + 4.0, z);
        if (0.0..W as f64).contains(&bx) && (0.0..H as f64).contains(&by) {
            beacons.push((bx as usize, by as usize, fi));
        }
    }
    boxes.push(Bx { lo: [-90.0, 0.0, 1050.0], hi: [70.0, 520.0, 1200.0], kind: Kind::Bld(250) });
    // pillars under the guideway
    let mut pillars = vec![];
    let mut s = 14.0;
    while s < S_END {
        let (x, z, _, _) = track(s);
        if !(x < -HW - 1.0 && z > Z0) {
            boxes.push(Bx { lo: [x - 0.7, 0.0, z - 0.7], hi: [x + 0.7, YR - BH - 0.8, z + 0.7], kind: Kind::Pillar });
            pillars.push(s);
        }
        s += if s < 120.0 { 22.0 } else { 36.0 };
    }
    for (i, b) in BLADES.iter().enumerate() {
        boxes.push(Bx { lo: [b.x.0, b.y.0, b.z], hi: [b.x.1, b.y.1, b.z + 0.3], kind: Kind::Sign(i as u8) });
    }
    boxes.push(Bx { lo: [HOLO.0 .0, HOLO.1 .0, Z0 - 0.3], hi: [HOLO.0 .1, HOLO.1 .1, Z0 - 0.2], kind: Kind::Holo });

    let sky = Hit { mat: Mat::Sky, depth: 1e9, pos: [0.0; 3], face: 4, id: 0, a: 0.0, b: 0.0 };
    let mut hit = vec![sky; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            let d = ray(px as f64, py as f64);
            let mut best = 1e9;
            let mut h = hit[k];
            if d[1] < 0.0 {
                let t = EYE / -d[1];
                best = t;
                h = Hit { mat: Mat::Ground, depth: t as f32, pos: [(o[0] + d[0] * t) as f32, 0.0, t as f32], face: 3, id: 0, a: 0.0, b: 0.0 };
            }
            for (bi, b) in boxes.iter().enumerate() {
                if let Some((t, ax)) = ray_box(&o, &d, b) {
                    if t < best {
                        best = t;
                        let p = [o[0] + d[0] * t, o[1] + d[1] * t, t];
                        let face = match ax {
                            2 => 0,
                            _ if d[0] < 0.0 => 1,
                            _ => 2,
                        };
                        let (a, bb) = if face == 0 { (p[0], p[1]) } else { (p[2], p[1]) };
                        let across = (p[0] - b.lo[0]) / (b.hi[0] - b.lo[0]);
                        let down = (b.hi[1] - p[1]) / (b.hi[1] - b.lo[1]);
                        let (mat, id, a, bb) = match b.kind {
                            Kind::Bld(i) => (Mat::Wall, u16::from(i), a, bb),
                            Kind::Pillar => (Mat::Pillar, bi as u16, a, (b.lo[0] + b.hi[0]) / 2.0),
                            Kind::Sign(i) => (Mat::Sign, u16::from(i), across, down),
                            Kind::Holo => (Mat::Holo, 0, across, down),
                        };
                        h = Hit { mat, depth: t as f32, pos: [p[0] as f32, p[1] as f32, p[2] as f32], face, id, a: a as f32, b: bb as f32 };
                    }
                }
            }
            hit[k] = h;
        }
    }

    // The guideway's two beams, the crossheads on the pillars, and the
    // trains' envelopes, swept slice by slice along s. Each slice owns a
    // short straight piece of each, and a ray's way into that piece is
    // found exactly.
    let mut runs_px: Vec<Vec<Run>> = vec![vec![]; n];
    let mut s = 0.0;
    let mut i = 0u32;
    let reach = LANE + TW;
    let beam = [(1.0, 0.0, BW), (-1.0, 0.0, BW), (0.0, 1.0, YR), (0.0, -1.0, BH - YR)];
    let cross = [(1.0, 0.0, LANE + BW + 0.4), (-1.0, 0.0, LANE + BW + 0.4), (0.0, 1.0, YR - BH + 0.05), (0.0, -1.0, 0.9 + BH - YR)];
    // the train's section: straight sides, a chamfered roof and skirt
    let body = [
        (1.0, 0.0, TW),
        (-1.0, 0.0, TW),
        (0.0, 1.0, TT),
        (0.0, -1.0, -TB),
        (1.0, 0.9, TW + 0.9 * (TB + 2.2)),
        (-1.0, 0.9, TW + 0.9 * (TB + 2.2)),
        (1.0, -0.8, TW - 0.8 * (TB + 0.6)),
        (-1.0, -0.8, TW - 0.8 * (TB + 0.6)),
    ];
    while s < S_END {
        let ds = (track(s).1 / (F * 2.5)).clamp(0.03, 3.0);
        let (cx, cz, tx, tz) = track(s);
        let (nx, nz) = (tz, -tx);
        let head = pillars.iter().any(|&p| (s - p).abs() < 0.7);
        // the slice's box on the picture
        let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
        for (u, y) in [(-reach, YR - BH - 1.0), (reach, YR - BH - 1.0), (-reach, TT), (reach, TT)] {
            for al in [-ds, ds] {
                let (qx, qy) = project(cx + nx * u + tx * al, y, cz + nz * u + tz * al);
                x0 = x0.min(qx);
                x1 = x1.max(qx);
                y0 = y0.min(qy);
                y1 = y1.max(qy);
            }
        }
        let (bx0, bx1) = ((x0.floor() - 1.0).max(0.0) as usize, ((x1.ceil() + 1.0).max(0.0) as usize).min(W));
        let (by0, by1) = ((y0.floor() - 1.0).max(0.0) as usize, ((y1.ceil() + 1.0).max(0.0) as usize).min(H));
        for py in by0..by1 {
            for px in bx0..bx1 {
                let d = ray(px as f64, py as f64);
                let k = py * W + px;
                // the ray in the slice's frame: across, up, along
                let (u0, du) = ((o[0] - cx) * nx + (o[2] - cz) * nz, d[0] * nx + d[2] * nz);
                let (a0, da) = ((o[0] - cx) * tx + (o[2] - cz) * tz, d[0] * tx + d[2] * tz);
                // where it is within this slice's piece
                let (mut lo, mut hi) = (0.5, 1e6);
                let half = ds * 0.6;
                if da.abs() < 1e-9 {
                    if a0.abs() > half {
                        continue;
                    }
                } else {
                    let (p, q) = ((-half - a0) / da, (half - a0) / da);
                    lo = p.min(q).max(lo);
                    hi = p.max(q).min(hi);
                }
                if lo >= hi {
                    continue;
                }
                let at = |t: f64| (u0 + du * t, o[1] + d[1] * t, o[0] + d[0] * t, t);
                if head {
                    if let Some((t, _)) = clip(&cross, u0, du, o[1], d[1], lo, hi) {
                        if t < f64::from(hit[k].depth) {
                            let (_, _, qx, qz) = at(t);
                            hit[k] = Hit {
                                mat: Mat::Pillar,
                                depth: t as f32,
                                pos: [qx as f32, (o[1] + d[1] * t) as f32, qz as f32],
                                face: 0,
                                id: 0,
                                a: s as f32,
                                b: 0.0,
                            };
                        }
                    }
                }
                for lane in 0..2u8 {
                    let off = LANE * (f64::from(lane) * 2.0 - 1.0);
                    if let Some((t, _)) = clip(&beam, u0 - off, du, o[1], d[1], lo, hi) {
                        if t < f64::from(hit[k].depth) {
                            let (u, y, qx, qz) = at(t);
                            let u = u - off;
                            // up the side, or across the underside
                            let (face, b) = if u.abs() > 0.97 * BW { (0, (YR - y) / BH) } else { (3, u.abs() / BW) };
                            hit[k] = Hit {
                                mat: Mat::Beam,
                                depth: t as f32,
                                pos: [qx as f32, y as f32, qz as f32],
                                face,
                                id: u16::from(lane),
                                a: s as f32,
                                b: b as f32,
                            };
                        }
                    }
                    if let Some((t, _)) = clip(&body, u0 - off, du, o[1], d[1], lo, hi) {
                        let (u, y, _, _) = at(t);
                        let (sf, tf, uf, vf) = (s as f32, t as f32, ((u - off) / TW) as f32, ((y - TB) / (TT - TB)) as f32);
                        let rs = &mut runs_px[k];
                        match rs.iter_mut().rev().find(|r| r.lane == lane) {
                            Some(r) if r.last + 3 >= i => {
                                r.last = i;
                                (r.s1, r.d1, r.u1, r.v1) = (sf, tf, uf, vf);
                                if tf < r.dn {
                                    (r.sn, r.dn, r.un, r.vn) = (sf, tf, uf, vf);
                                }
                            }
                            _ => rs.push(Run { lane, last: i, s0: sf, s1: sf, d0: tf, d1: tf, u0: uf, v0: vf, u1: uf, v1: vf, sn: sf, dn: tf, un: uf, vn: vf }),
                        }
                    }
                }
            }
        }
        s += ds;
        i += 1;
    }
    let mut runs = vec![];
    let mut run_at = vec![0u32; n + 1];
    for k in 0..n {
        let mut rs: Vec<Run> = runs_px[k].iter().copied().filter(|r| r.dn < hit[k].depth).collect();
        rs.sort_by(|a, b| a.dn.total_cmp(&b.dn));
        rs.truncate(4);
        runs.extend(rs);
        run_at[k + 1] = runs.len() as u32;
    }

    // the fixed colours
    let mut base = vec![[0.0f32; 3]; n];
    let mut fog = vec![0.0f32; n];
    let mut ink = vec![0.0f32; n];
    for py in 0..H {
        for px in 0..W {
            let k = py * W + px;
            let h = hit[k];
            let f = if h.mat == Mat::Sky { 1.0 } else { 1.0 - (-f64::from(h.depth) / HAZE).exp() };
            let c = match h.mat {
                Mat::Wall => wall(&h, px, py),
                Mat::Pillar => {
                    let side = if h.face == 0 { 0.045 } else { 0.03 };
                    [side, side * 0.9, side * 1.5]
                }
                Mat::Beam => {
                    if h.face == 0 {
                        [0.1, 0.09, 0.15]
                    } else {
                        [0.06, 0.04, 0.09]
                    }
                }
                Mat::Ground => {
                    let (x, z) = (f64::from(h.pos[0]), f64::from(h.pos[2]));
                    let (c, wet) = street(x, z, f64::from(h.depth));
                    ink[k] = wet as f32;
                    c
                }
                Mat::Sign => {
                    ink[k] = blade_ink(&h) as f32;
                    let c = BLADES[h.id as usize].col;
                    [0.02 + 0.06 * c[0], 0.02 + 0.06 * c[1], 0.03 + 0.06 * c[2]]
                }
                Mat::Holo => wall(&Hit { id: 1, a: h.pos[0], b: h.pos[1], ..h }, px, py),
                Mat::Sky => [0.0; 3],
            };
            let sm = smog(py as f64);
            base[k] = [mix(c[0], sm[0], f) as f32, mix(c[1], sm[1], f) as f32, mix(c[2], sm[2], f) as f32];
            fog[k] = f as f32;
        }
    }

    // The wet street's mirror: every pixel above it lands, upside down, on
    // the street pixel that sees it reflected; the nearest wins, and a gap
    // takes the sky's mirror or the one above it.
    let mut mirror = vec![u32::MAX; n];
    let mut mirror_depth = vec![f32::MAX; n];
    for k in 0..n {
        let h = hit[k];
        if matches!(h.mat, Mat::Ground | Mat::Sky) {
            continue;
        }
        let ry = HY + F * (f64::from(h.pos[1]) + EYE) / f64::from(h.depth);
        let px = k % W;
        for r in [ry.floor(), ry.floor() + 1.0] {
            if r < 0.0 || r >= H as f64 {
                continue;
            }
            let q = r as usize * W + px;
            if hit[q].mat == Mat::Ground && hit[q].depth < h.depth && h.depth < mirror_depth[q] {
                mirror[q] = k as u32;
                mirror_depth[q] = h.depth;
            }
        }
    }
    for py in 0..H {
        for px in 0..W {
            let q = py * W + px;
            if hit[q].mat != Mat::Ground || mirror[q] != u32::MAX {
                continue;
            }
            let sky_row = 2.0 * HY - py as f64;
            if sky_row >= 0.0 && hit[sky_row as usize * W + px].mat == Mat::Sky {
                mirror[q] = (sky_row as usize * W + px) as u32;
            } else if py > 0 && hit[q - W].mat == Mat::Ground {
                mirror[q] = mirror[q - W];
                mirror_depth[q] = mirror_depth[q - W];
            }
        }
    }

    Box::new(Maglev {
        hit,
        base,
        fog,
        ink,
        runs,
        run_at,
        mirror,
        mirror_depth,
        beacons,
        buf: vec![[0.0; 3]; n],
        tref: vec![[0.0; 3]; n],
        tref_depth: vec![0.0; n],
        tref_stamp: vec![0; n],
        stamp: 0,
        bloom: vec![[0.0; 3]; GW * GH],
        tmp: vec![[0.0; 3]; GW * GH],
        pixels: Pixels::new(&META),
    })
}

/// A ray, as `u0 + du t` across and `y0 + dy t` up, clipped by a convex
/// section given as half-planes `au u + ay y <= c`, within `lo..hi`: where
/// it enters and leaves.
fn clip(planes: &[(f64, f64, f64)], u0: f64, du: f64, y0: f64, dy: f64, lo: f64, hi: f64) -> Option<(f64, f64)> {
    let (mut a, mut b) = (lo, hi);
    for &(au, ay, c) in planes {
        let p = au * u0 + ay * y0 - c;
        let q = au * du + ay * dy;
        if q.abs() < 1e-12 {
            if p > 0.0 {
                return None;
            }
        } else if q < 0.0 {
            a = a.max(-p / q);
        } else {
            b = b.min(-p / q);
        }
        if a >= b {
            return None;
        }
    }
    Some((a, b))
}

/// A tower's face: dark glass and steel, a grid of windows, some lit, and
/// on some towers a band of light every few floors.
fn wall(h: &Hit, px: usize, py: usize) -> [f64; 3] {
    let id = f64::from(h.id);
    let (a, b) = (f64::from(h.a), f64::from(h.b));
    let dark = [0.022, 0.02, 0.045];
    let foot = f64::from(h.depth) / F;
    let dens = 0.15 + 0.45 * hash(id, 3.0);
    let tint = hash(id, 4.0);
    let warm = [1.0, 0.74, 0.46];
    let cool = [0.5, 0.78, 1.0];
    let pink = [1.0, 0.4, 0.78];
    let pick = |q: f64| {
        if q < 0.45 {
            warm
        } else if q < 0.85 {
            cool
        } else {
            pink
        }
    };
    // bands of neon every few floors, on one tower in three
    if hash(id, 9.0) < 0.35 && h.face != 0 {
        let every = 12.0 + 8.0 * (hash(id, 10.0) * 3.0).floor();
        let band = b.rem_euclid(every);
        let th = (0.35f64).max(foot * 0.6);
        if band < th && b > 4.0 {
            let c = [MAGENTA, CYAN, AMBER][(hash(id, 11.0) * 3.0) as usize];
            let g = 0.8 * (0.35 / th).min(1.0) + 0.1;
            return [c[0] * g, c[1] * g, c[2] * g];
        }
    }
    // shops along the street: lit glass under a strip of neon
    if b < 4.2 && id < 200.0 {
        let unit = (a / 6.0).floor();
        let ua = a / 6.0 - unit;
        let q = hash(unit * 7.0 + id, 21.0);
        let c = [warm, CYAN, MAGENTA, ACID, AMBER][(q * 5.0) as usize];
        if q > 0.85 || !(0.05..0.95).contains(&ua) {
            return dark;
        }
        if (3.35..3.85).contains(&b) && (0.15..0.85).contains(&ua) {
            // the shop's name in a tube
            let g = 0.75 * hash(unit + id, 22.0).max(0.5);
            return [c[0] * g, c[1] * g, c[2] * g];
        }
        if b > 0.3 && b < 2.9 {
            // its window: a dim glow, brighter low, split by mullions
            let mull = ((a / 1.5).fract() < 0.1) && foot < 0.5;
            if mull {
                return dark;
            }
            let g = (0.08 + 0.14 * hash(unit + id, 23.0)) * (0.7 + 0.5 * (2.9 - b) / 2.6);
            return [0.03 + c[0] * g, 0.03 + c[1] * g, 0.04 + c[2] * g];
        }
        return dark;
    }
    if foot > 0.9 {
        // too far for single windows: their light, sparkled
        let g = dens * 0.32 * (0.3 + 1.4 * hash(px as f64 * 3.0 + id, py as f64 * 5.0));
        let c = pick(tint);
        return [dark[0] + c[0] * g, dark[1] + c[1] * g, dark[2] + c[2] * g];
    }
    let (fl, fc) = ((b / 3.2).floor(), (a / 1.6).floor());
    let (fb, fa) = (b / 3.2 - fl, a / 1.6 - fc);
    if (0.3..0.8).contains(&fb) && (0.15..0.85).contains(&fa) && hash(fc * 13.0 + id * 101.0, fl * 7.0 + 3.0) < dens {
        let q = hash(fc * 5.0 + id, fl * 11.0 + 1.0);
        let c = pick(if hash(fc, fl + id) < 0.7 { tint } else { q });
        let mut g = 0.2 + 0.5 * q;
        // up close: brighter toward the sill, and some with blinds half down
        g *= 0.8 + 0.4 * (0.8 - fb) / 0.5;
        if foot < 0.4 && hash(fc + 3.0, fl + id) < 0.3 && fb > 0.55 && ((fb * 3.2 / 0.25).fract() < 0.5) {
            g *= 0.45;
        }
        return [c[0] * g, c[1] * g, c[2] * g];
    }
    dark
}

/// The street at (x, z): its colour, and how wet (how mirror-like) it is.
fn street(x: f64, z: f64, depth: f64) -> ([f64; 3], f64) {
    let grain = 0.85 + 0.3 * noise(x * 1.3, z * 1.3);
    let mut c = [0.03 * grain, 0.028 * grain, 0.045 * grain];
    let puddle = smooth(0.48, 0.62, fbm(x * 0.12 + 3.0, z * 0.05, 3));
    let mut wet = 0.35 + 0.5 * puddle;
    let foot = depth / F;
    let line = |at: f64, w: f64| smooth(w + foot * 0.5, w * 0.5, (x - at).abs());
    if z > Z0 && x.abs() < HW {
        if x.abs() > HW - 3.5 {
            // the pavements, a step lighter, drier
            c = [0.04 * grain, 0.038 * grain, 0.055 * grain];
            wet *= 0.6;
        } else {
            // lane lines, dashed, and a solid pair in the middle
            let dash = smooth(0.4, 0.3, ((z / 9.0).fract() - 0.5).abs()) * (line(-5.75, 0.08) + line(5.75, 0.08));
            let mid = line(-0.25, 0.07) + line(0.25, 0.07);
            let g = (dash + mid).min(1.0) * 0.2;
            c = [c[0] + g * 0.9, c[1] + g * 0.85, c[2] + g * 0.7];
        }
    } else if z < Z0 && z > Z0 - 5.0 && x.abs() < HW - 3.5 && (x / 1.4).rem_euclid(1.0) < 0.5 {
        // the cross street: a crossing in zebra stripes
        c = [0.12 * grain, 0.12 * grain, 0.13 * grain];
        wet *= 0.7;
    }
    (c, wet)
}

/// A blade sign's lettering at a pixel: glyphs down its middle and a tube
/// round its edge, smoothed over the pixel.
fn blade_ink(h: &Hit) -> f64 {
    let bl = &BLADES[h.id as usize];
    let (w, ht) = (bl.x.1 - bl.x.0, bl.y.1 - bl.y.0);
    let step = f64::from(h.depth) / F;
    let mut sum = 0.0;
    for j in 0..3 {
        for i in 0..3 {
            let lx = f64::from(h.a) * w + (i as f64 - 1.0) / 3.0 * step;
            let ly = f64::from(h.b) * ht + (j as f64 - 1.0) / 3.0 * step;
            let e = 0.16;
            if lx < e || lx > w - e || ly < e || ly > ht - e {
                sum += 0.55;
                continue;
            }
            let cell = w * 0.95;
            let count = (ht / cell).floor();
            let off = (ht - count * cell) / 2.0;
            let ci = ((ly - off) / cell).floor();
            if ci < 0.0 || ci >= count {
                continue;
            }
            let m = 0.32;
            let gx = ((lx - m) / (w - 2.0 * m) * 5.0).floor() as i64;
            let gy = (((ly - off) - ci * cell - m * 0.8) / (cell - 1.6 * m) * 5.0).floor() as i64;
            if glyph(h.id as usize * 3 + ci as usize, gx, gy) {
                sum += 1.0;
            }
        }
    }
    sum / 9.0
}

/// How a sign is lit at `t`: steady mostly, sometimes stuttering, one of them failing.
fn flicker(id: f64, t: f64) -> f64 {
    let slot = (t * 15.0).floor();
    if id == 2.0 {
        // the broken one: buzzes, and goes out for a while every so often
        let w = (t / 5.0).floor();
        let out = w * 5.0 + 2.0 + hash(id, w) * 2.0;
        if t > out && t < out + 0.9 && hash(slot, 3.0) < 0.75 {
            return 0.08;
        }
        return 0.75 + 0.25 * hash(slot, id);
    }
    if hash(slot + id * 97.0, (t / 7.0).floor() + id) < 0.03 {
        0.25
    } else {
        1.0
    }
}

/// The holo billboard at (a, b) in 0–1 across and down: a huge eye that
/// looks about and blinks, then a slogan, swapped with a glitch.
fn holo(a: f64, b: f64, py: usize, t: f64) -> [f64; 3] {
    let cyc = 14.0;
    let ph = (t + 2.0).rem_euclid(cyc);
    // glitch: rows shift sideways for a moment at each swap, and now and then
    let slot = (t * 15.0).floor();
    let near_swap = ph < 0.35 || (ph > 7.0 && ph < 7.35);
    let glitchy = near_swap || hash(slot, 77.0) < 0.04;
    let band = (b * 9.0).floor();
    let mut a = a;
    if glitchy && hash(band, slot) < 0.5 {
        a += (hash(band + 3.0, slot) - 0.5) * 0.25;
    }
    let scan = if py.is_multiple_of(2) { 1.0 } else { 0.55 };
    let edge = !(0.02..=0.98).contains(&a) || !(0.025..=0.975).contains(&b);
    if !(0.0..=1.0).contains(&a) {
        return [0.0; 3];
    }
    let mut c = if edge { [0.1, 0.55, 0.65] } else { [0.02, 0.07, 0.1] };
    if ph < 7.0 {
        // the eye
        let (x, y) = ((a - 0.5) * 2.4, (b - 0.5) * 2.0);
        let blink_t = (t / 3.7).floor();
        let since = t - blink_t * 3.7 - 2.6 * hash(blink_t, 5.0);
        let open = if (0.0..0.3).contains(&since) { (since / 0.15 - 1.0).abs() } else { 1.0 };
        let lid = 0.62 * (1.0 - x * x).max(0.0).powf(0.8) * open;
        let look = (t / 1.6).floor();
        let k = smooth(0.0, 0.25, t - look * 1.6);
        let (gx0, gy0) = (hash(look - 1.0, 8.0) - 0.5, hash(look - 1.0, 9.0) - 0.5);
        let (gx1, gy1) = (hash(look, 8.0) - 0.5, hash(look, 9.0) - 0.5);
        let (ix, iy) = (mix(gx0, gx1, k) * 0.7, mix(gy0, gy1, k) * 0.3);
        let r = (x - ix).hypot(y - iy);
        if y.abs() < lid {
            c = [0.25, 0.2, 0.32];
            if r < 0.42 {
                let ring = 0.55 + 0.45 * ((y - iy).atan2(x - ix) * 9.0).sin().abs() * smooth(0.18, 0.4, r);
                c = [CYAN[0] * ring, CYAN[1] * ring, CYAN[2] * ring];
                if r < 0.17 {
                    c = [0.01, 0.02, 0.04];
                }
                if (x - ix + 0.09).hypot(y - iy + 0.1) < 0.07 {
                    c = [1.0, 1.0, 1.0];
                }
            }
        }
        if (y.abs() - lid).abs() < 0.05 && x.abs() < 0.98 {
            c = MAGENTA;
        }
    } else {
        // the slogan: two big glyphs, and a ticker under them
        let (gx, gy) = (a * 2.0, (b - 0.1) / 0.62);
        if (0.0..1.0).contains(&gy) {
            let gi = gx.floor();
            let (cx, cy) = (((gx - gi - 0.12) / 0.76 * 5.0).floor() as i64, (gy * 5.0).floor() as i64);
            if glyph(4 + gi as usize * 3, cx, cy) {
                c = MAGENTA;
            }
        } else if (0.78..0.92).contains(&b) {
            let tx = a * 8.0 + t * 1.5;
            let gi = tx.floor();
            let (cx, cy) = (((tx - gi) * 6.0).floor() as i64, ((b - 0.78) / 0.14 * 5.0).floor() as i64);
            if glyph(gi as usize, cx, cy) {
                c = AMBER;
            }
        }
    }
    let fl = if glitchy && hash(slot, 12.0) < 0.3 { 0.5 } else { 1.0 };
    [c[0] * scan * fl, c[1] * scan * fl, c[2] * scan * fl]
}

/// The light a train throws on a point: from lamps strung along it, falling
/// off with distance, more on a face turned toward them.
fn wash(lights: &[[f64; 3]], p: [f32; 3], face: u8) -> f64 {
    let (x, y, z) = (f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
    let mut sum = 0.0;
    for l in lights {
        let (dx, dy, dz) = (l[0] - x, l[1] - y, l[2] - z);
        let d2 = dx * dx + dy * dy + dz * dz;
        if d2 > 9000.0 {
            continue;
        }
        let d = d2.sqrt().max(0.01);
        let facing = match face {
            0 => -dz / d,
            1 => dx / d,
            2 => -dx / d,
            3 => dy / d,
            _ => 0.6,
        };
        sum += (0.15 + 0.85 * facing.max(0.0)) / (1.0 + d2 / 30.0);
    }
    sum
}

impl Maglev {
    /// The train at pixel k, if any is in front of `depth`: its colour, depth,
    /// height and how solid it is (the blur behind it is see-through).
    fn train_at(&self, k: usize, py: usize, depth: f32, trains: &[Train]) -> Option<([f64; 3], f32, f64, f64)> {
        let mut best: Option<([f64; 3], f32, f64, f64)> = None;
        let mut near = depth;
        for r in &self.runs[self.run_at[k] as usize..self.run_at[k + 1] as usize] {
            for tr in trains.iter().filter(|tr| tr.lane == r.lane) {
                // the stretch it fills, and the blur it leaves behind it
                let (lo, hi) = if tr.dir > 0.0 { (tr.lo - GHOST, tr.hi) } else { (tr.lo, tr.hi + GHOST) };
                let (a, b) = (f64::from(r.s0).max(lo), f64::from(r.s1).min(hi));
                if a > b {
                    continue;
                }
                let sn = f64::from(r.sn);
                let (dd, uu, vv, s, cap) = if sn >= a && sn <= b {
                    (r.dn, r.un, r.vn, sn, false)
                } else if sn < a {
                    let w = ((a - sn) / (f64::from(r.s1) - sn).max(1e-6)) as f32;
                    (r.dn + (r.d1 - r.dn) * w, r.un + (r.u1 - r.un) * w, r.vn + (r.v1 - r.vn) * w, a, true)
                } else {
                    let w = ((b - f64::from(r.s0)) / (sn - f64::from(r.s0)).max(1e-6)) as f32;
                    (r.d0 + (r.dn - r.d0) * w, r.u0 + (r.un - r.u0) * w, r.v0 + (r.vn - r.v0) * w, b, true)
                };
                if dd >= near {
                    continue;
                }
                let back = (tr.head - s) * tr.dir;
                let v = f64::from(vv);
                let side = f64::from(uu).abs() > 0.8 && !cap;
                if back < 6.0 && !cap && v > 0.3 + 0.7 * (back.max(0.0) / 6.0).sqrt() {
                    continue; // the nose's slope
                }
                let solid = if back > LEN { 0.55 * (1.0 - (back - LEN) / GHOST) } else { 1.0 };
                let col = train_colour(back, v, f64::from(uu), side, cap, tr);
                let f = 1.0 - (-f64::from(dd) / HAZE).exp();
                let sm = smog(py as f64);
                let col = [mix(col[0], sm[0], f), mix(col[1], sm[1], f), mix(col[2], sm[2], f)];
                if solid >= 1.0 {
                    near = dd;
                }
                let y = TB + v * (TT - TB);
                best = Some((col, dd, y, solid));
            }
        }
        best
    }
}

/// The train's skin: `back` metres behind its nose, `v` up its side, `u` across.
fn train_colour(back: f64, v: f64, u: f64, side: bool, cap: bool, tr: &Train) -> [f64; 3] {
    if cap && back < 1.0 {
        // the nose, head on: dark glass over a pair of lamps
        if (0.15..0.32).contains(&v) && u.abs() > 0.35 {
            return if tr.dir < 0.0 { [1.0, 1.0, 0.95] } else { [1.0, 0.15, 0.2] };
        }
        if (0.45..0.82).contains(&v) {
            return [0.05, 0.16, 0.22];
        }
        return [0.5, 0.53, 0.62];
    }
    if cap && back > LEN - 1.0 {
        // the tail
        if (0.15..0.32).contains(&v) {
            return if tr.dir < 0.0 { [1.0, 0.12, 0.18] } else { [1.0, 1.0, 0.95] };
        }
        return [0.42, 0.44, 0.52];
    }
    if v < 0.06 {
        return [0.25 * CYAN[0] + 0.1, 0.25 * CYAN[1] + 0.1, 0.25 * CYAN[2] + 0.12];
    }
    if v > 0.86 || !side {
        // the roof, catching the signs' pink
        let g = 0.32 + 0.2 * (1.0 - v);
        return [g + 0.12, g * 0.95, g * 1.1 + 0.06];
    }
    if (0.17..0.24).contains(&v) {
        let chase = 0.75 + 0.25 * ((back * 0.5 - tr.cycle) * 0.9).sin();
        return [CYAN[0] * chase + 0.15, CYAN[1] * chase + 0.1, CYAN[2] * chase + 0.05];
    }
    if (0.42..0.7).contains(&v) {
        if back < 5.0 {
            return [0.08, 0.3, 0.38]; // the cab's glass
        }
        let p = (back / 2.6).fract();
        // lit windows, smeared together by the speed
        let lit = smooth(0.08, 0.24, p) * smooth(0.92, 0.76, p);
        let g = 0.4 + 0.6 * lit;
        return [mix(0.05, 1.0, g), mix(0.06, 0.9, g), mix(0.1, 0.7, g)];
    }
    // chrome: dark low where it mirrors the street, bright high where it mirrors the sky's glow
    let g = if v < 0.42 { 0.3 + 0.3 * v } else { 0.55 + 0.4 * (v - 0.7) };
    let pink = if v < 0.42 { 0.1 * (0.42 - v) / 0.42 } else { 0.0 };
    [g * 0.92 + 0.05 + pink, g * 0.95, g * 1.08 + pink * 0.6]
}

impl Piece for Maglev {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let trains = trains(t);
        self.stamp = self.stamp.wrapping_add(1);
        // lamps along each train, for the light it throws
        let mut lights = vec![];
        for tr in &trains {
            for j in 0..7 {
                let s = tr.lo + (j as f64 + 0.5) / 7.0 * (tr.hi - tr.lo);
                if !(0.0..S_END).contains(&s) {
                    continue;
                }
                let (x, z) = on_lane(s, tr.lane);
                if z < 260.0 {
                    lights.push([x, YR + 1.0, z]);
                }
            }
        }
        // the rail: how bright it glows at s, charged ahead of a train and fading behind it
        let rail = |s: f64, lane: u16| -> f64 {
            let mut g: f64 = 0.45 + 0.5 * smooth(0.8, 1.0, ((s + t * 28.0) / 47.0).fract());
            for tr in trains.iter().filter(|tr| u16::from(tr.lane) == lane) {
                let ahead = (s - tr.head) * tr.dir;
                let behind = (tr.head - tr.dir * LEN - s) * tr.dir;
                if ahead > 0.0 {
                    g = g.max(1.2 * (-ahead / 30.0).exp());
                } else if behind > 0.0 {
                    g = g.max(1.6 * (-behind / SPEED / 0.9).exp());
                } else {
                    g = g.max(1.6);
                }
            }
            g
        };

        // the clouds' drift and the searchlight's sweep
        let drift = t * 2.2;
        let sweep = 0.35 * (t * 0.23).sin() - 0.1;
        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                let h = self.hit[k];
                if h.mat == Mat::Ground {
                    continue;
                }
                let mut c = self.base[k].map(f64::from);
                let fog = f64::from(self.fog[k]);
                match h.mat {
                    Mat::Sky => {
                        let (x, y) = (px as f64, py as f64);
                        let sm = smog(y);
                        let v = y / HY;
                        c = [mix(0.03, sm[0], v), mix(0.02, sm[1], v), mix(0.06, sm[2], v)];
                        let cl = fbm((x + drift) * 0.02, y * 0.06 + 3.0, 4);
                        let cover = smooth(0.42, 0.7, cl);
                        let lit = 0.5 + 0.8 * v;
                        c[0] = mix(c[0], 0.3 * lit, cover * 0.8);
                        c[1] = mix(c[1], 0.09 * lit, cover * 0.8);
                        c[2] = mix(c[2], 0.3 * lit, cover * 0.8);
                        // a searchlight from the arcology, sweeping the cloud
                        let (sx, sy) = (150.0, 70.0);
                        let ang = (x - sx).atan2(sy - y) - sweep;
                        let beam = (-(ang / 0.05).powi(2)).exp() * smooth(sy, sy - 20.0, y) * (0.4 + 0.6 * cover);
                        c = [c[0] + 0.35 * beam, c[1] + 0.3 * beam, c[2] + 0.45 * beam];
                    }
                    Mat::Sign => {
                        let bl = &BLADES[h.id as usize];
                        let g = f64::from(self.ink[k]) * flicker(f64::from(h.id), t) * (1.0 - fog * 0.7);
                        c = [c[0] + bl.col[0] * g, c[1] + bl.col[1] * g, c[2] + bl.col[2] * g];
                    }
                    Mat::Holo => {
                        let hc = holo(f64::from(h.a), f64::from(h.b), py, t);
                        c = [c[0] * 0.4 + hc[0], c[1] * 0.4 + hc[1], c[2] * 0.4 + hc[2]];
                    }
                    Mat::Beam => {
                        let s = f64::from(h.a);
                        let b = f64::from(h.b);
                        let g = rail(s, h.id) * (1.0 - fog);
                        // a strip of light along its side, and a glow under it
                        let strip = if h.face == 0 { smooth(0.08, 0.14, b) * smooth(0.34, 0.26, b) } else { 0.12 + 0.9 * smooth(0.55, 0.8, b) };
                        let g = g * strip;
                        c = [c[0] + CYAN[0] * 0.8 * g, c[1] + CYAN[1] * 0.8 * g, c[2] + CYAN[2] * 0.8 * g];
                    }
                    Mat::Pillar => {
                        // a red lamp near the top, blinking
                        let y = f64::from(h.pos[1]);
                        let lamp = h.id > 0 && h.face == 0 && (f64::from(h.a) - f64::from(h.b)).abs() < 0.3 && (YR - BH - 1.6..YR - BH - 1.15).contains(&y);
                        if lamp && (t * 1.3 + f64::from(h.id) * 0.37).fract() < 0.4 {
                            c = [1.0 * (1.0 - fog), 0.1, 0.12];
                        }
                    }
                    _ => {}
                }
                if !lights.is_empty() && h.mat != Mat::Sky {
                    let l = wash(&lights, h.pos, h.face) * (1.0 - fog) * 0.4;
                    c = [c[0] + l * 0.55, c[1] + l * 0.85, c[2] + l];
                }
                self.buf[k] = c.map(|v| v as f32);
            }
        }
        // beacons on the far towers' tops
        for &(bx, by, i) in &self.beacons {
            if (t * 0.9 + i * 0.31).fract() < 0.25 && self.hit[by * W + bx].mat == Mat::Sky {
                self.buf[by * W + bx] = [0.9, 0.12, 0.15];
            }
        }
        // the trains, over everything they are in front of; mirrored in the street
        for py in 0..H {
            for px in 0..W {
                let k = py * W + px;
                if self.run_at[k] == self.run_at[k + 1] {
                    continue;
                }
                if let Some((col, dd, y, solid)) = self.train_at(k, py, self.hit[k].depth, &trains) {
                    let c = self.buf[k];
                    let m = [mix(f64::from(c[0]), col[0], solid), mix(f64::from(c[1]), col[1], solid), mix(f64::from(c[2]), col[2], solid)];
                    self.buf[k] = m.map(|v| v as f32);
                    let ry = HY + F * (y + EYE) / f64::from(dd);
                    for r in [ry.floor(), ry.floor() + 1.0] {
                        if r < 0.0 || r >= H as f64 {
                            continue;
                        }
                        let q = r as usize * W + px;
                        let fresh = self.tref_stamp[q] != self.stamp || dd < self.tref_depth[q];
                        if self.hit[q].mat == Mat::Ground && self.hit[q].depth < dd && dd < self.mirror_depth[q] && fresh {
                            self.tref[q] = self.buf[k];
                            self.tref_depth[q] = dd;
                            self.tref_stamp[q] = self.stamp;
                        }
                    }
                }
            }
        }
        // the wet street
        for py in (HY as usize)..H {
            for px in 0..W {
                let k = py * W + px;
                let h = self.hit[k];
                if h.mat != Mat::Ground {
                    continue;
                }
                let wet = f64::from(self.ink[k]);
                let mut c = self.base[k].map(f64::from);
                // ripples from the rain shake the mirror sideways
                let rip = noise(px as f64 * 0.35, py as f64 * 1.3 + t * 9.0) - 0.5;
                let sx = (px as f64 + rip * (2.0 + 3.0 * wet)).round().clamp(0.0, W as f64 - 1.0) as usize;
                let mut r = [0.0f64; 3];
                let mut wsum = 0.0;
                for j in 0..3 {
                    let q = py.saturating_sub(j * 2) * W + sx;
                    if self.hit[q].mat != Mat::Ground {
                        continue;
                    }
                    let wj = 1.0 / (1.0 + j as f64);
                    let src = if self.tref_stamp[q] == self.stamp {
                        self.tref[q]
                    } else if self.mirror[q] != u32::MAX {
                        self.buf[self.mirror[q] as usize]
                    } else {
                        [0.0; 3]
                    };
                    for i in 0..3 {
                        r[i] += f64::from(src[i]) * wj;
                    }
                    wsum += wj;
                }
                let refl = 0.25 + 0.6 * wet;
                for i in 0..3 {
                    c[i] += r[i] / wsum.max(1e-6) * refl;
                }
                if !lights.is_empty() {
                    let l = wash(&lights, h.pos, 3) * (1.0 - f64::from(self.fog[k])) * 0.2;
                    c = [c[0] + l * 0.4, c[1] + l * 0.85, c[2] + l];
                }
                self.buf[k] = c.map(|v| v as f32);
            }
        }
        self.traffic(t);
        self.sparks(t, &trains);
        self.lamps(&trains);
        self.bloom_build();
        self.steam(t, &lights);
        self.rain(t);
        self.railing(t);
        self.bloom_add();
        // In one ink the dots need the darks darker for the lights to stand out.
        let curve = |v: f32| if env.mono { v.max(0.0).powf(1.35) * 1.15 } else { v };
        for py in 0..H {
            for px in 0..W {
                self.pixels.set(px, py, self.buf[py * W + px].map(curve));
            }
        }
        self.pixels.draw(grid, env);
    }
}

impl Maglev {
    /// Rain in three layers: fine and slow far off, long and fast up close,
    /// each streak catching the light behind it.
    fn rain(&mut self, t: f64) {
        let layers = [(150.0, 0.06, 0.035, 2.0, 0.22), (240.0, 0.1, 0.03, 4.0, 0.3), (380.0, 0.16, 0.02, 8.0, 0.42)];
        for (li, &(speed, slant, dens, len, bright)) in layers.iter().enumerate() {
            let lf = li as f64;
            let period = 50.0 + 20.0 * lf;
            for py in 0..H {
                for px in 0..W {
                    let u = px as f64 + slant * py as f64 - slant * speed * t;
                    let col = u.floor();
                    let y = py as f64 + speed * t + hash(col, lf + 40.0) * period;
                    let cyc = (y / period).floor();
                    if hash(col * 3.0 + lf, cyc) > dens * period / len {
                        continue;
                    }
                    let along = y - cyc * period - hash(col, cyc + 7.0) * (period - len);
                    if !(0.0..len).contains(&along) {
                        continue;
                    }
                    let k = py * W + px;
                    let c = self.buf[k];
                    let lum = 0.3 * c[0] + 0.5 * c[1] + 0.2 * c[2];
                    let g = bright * (0.5 + 0.5 * along / len) as f32;
                    let pick = [0.5 + c[0] * 1.5, 0.55 + c[1] * 1.5, 0.7 + c[2] * 1.5];
                    for i in 0..3 {
                        self.buf[k][i] = c[i] + g * (0.35 + lum) * pick[i] * 0.5;
                    }
                }
            }
        }
    }

    /// The glow at a pixel, from the blurred bright parts.
    fn glow(&self, px: usize, py: usize) -> [f32; 3] {
        self.bloom[(py / 4).min(GH - 1) * GW + (px / 4).min(GW - 1)]
    }

    /// Adds light to a pixel, if nothing nearer than `depth` hides it.
    fn add(&mut self, px: f64, py: f64, depth: f64, c: [f64; 3]) {
        if px < 0.0 || py < 0.0 || px >= W as f64 || py >= H as f64 {
            return;
        }
        let k = py as usize * W + px as usize;
        if f64::from(self.hit[k].depth) < depth {
            return;
        }
        for (o, v) in self.buf[k].iter_mut().zip(c) {
            *o += v as f32;
        }
    }

    /// Cars down in the canyon: headlights coming, tail-lights going, each
    /// drawn down the wet road in a streak.
    fn traffic(&mut self, t: f64) {
        let lanes = [(-10.5, -1.0, 11.0), (-6.5, -1.0, 15.0), (6.0, 1.0, 13.0), (10.0, 1.0, 9.0)];
        for (li, &(x, dir, v)) in lanes.iter().enumerate() {
            for c in 0..7 {
                let seed = (li * 10 + c) as f64;
                let len = 620.0;
                let z = 34.0 + (hash(seed, 1.0) * len + dir * v * (1.0 + 0.3 * hash(seed, 2.0)) * t).rem_euclid(len);
                let fade = (-z / HAZE).exp();
                let col = if dir < 0.0 { [1.0, 0.92, 0.75] } else { [1.0, 0.1, 0.12] };
                let g = fade * if dir < 0.0 { 1.0 } else { 0.8 };
                for side in [-0.75, 0.75] {
                    let (qx, qy) = project(x + side, 0.75, z);
                    self.add(qx, qy, z, [col[0] * g, col[1] * g, col[2] * g]);
                    // its reflection, a streak down the road
                    let (_, ry) = project(x + side, -0.75, z);
                    let n = (ry - qy).max(1.0) * 2.5;
                    let mut r = 1.0;
                    while r < n {
                        let y = qy + r;
                        let a = 0.5 * g * (1.0 - r / n);
                        let jx = qx + (noise(qx * 0.5, y * 0.7 + t * 6.0) - 0.5) * 2.0;
                        if y < H as f64 && self.hit[y as usize * W + jx.clamp(0.0, W as f64 - 1.0) as usize].mat == Mat::Ground {
                            self.add(jx, y, f64::MAX, [col[0] * a, col[1] * a, col[2] * a]);
                        }
                        r += 1.0;
                    }
                }
            }
        }
    }

    /// A train's lamps as glare: white ahead of one coming, red behind one
    /// going, seen from far down the canyon before the train itself is.
    fn lamps(&mut self, trains: &[Train]) {
        for tr in trains {
            for (end, s) in [(0, tr.head), (1, tr.head - tr.dir * LEN)] {
                if !(0.0..S_END).contains(&s) {
                    continue;
                }
                let white = (end == 0) == (tr.dir < 0.0);
                let (x, z) = on_lane(s, tr.lane);
                let (_, _, tx, tz) = track(s);
                // only the end that faces us shows its lamps
                let toward = -(tx * (x - CX) + tz * z) * if end == 0 { tr.dir } else { -tr.dir };
                if toward <= 0.0 {
                    continue;
                }
                let (qx, qy) = project(x, TB + 0.8, z);
                if qx < -20.0 || qx > W as f64 + 20.0 {
                    continue;
                }
                let fade = (-z / (HAZE * 1.4)).exp();
                let col = if white { [1.0, 0.97, 0.9] } else { [1.0, 0.12, 0.15] };
                let r = (90.0 / z).clamp(1.2, 10.0);
                let gain = if white { 1.3 } else { 0.8 } * fade;
                let n = r.ceil() as i64 * 2;
                for dy in -n..=n {
                    for dx in -n..=n {
                        let d = (dx as f64).hypot(dy as f64 * 1.6) / r;
                        let g = gain * ((-d * d * 3.0).exp() + 0.25 * (-d).exp());
                        if g > 0.01 {
                            self.add(qx + dx as f64, qy + dy as f64, z - 1.0, [col[0] * g, col[1] * g, col[2] * g]);
                        }
                    }
                }
            }
        }
    }

    /// Sparks thrown from under a train as it takes the curve: hot white,
    /// cooling to orange and red as they fall.
    fn sparks(&mut self, t: f64, trains: &[Train]) {
        let now = (t * 30.0).floor();
        for tr in trains {
            for j in 0..30 {
                let slot = now - j as f64;
                let tb = slot / 30.0;
                let age = t - tb;
                for m in 0..6 {
                    let r = |q: f64| hash(slot * 6.0 + m as f64 + tr.cycle * 1013.0, q);
                    let life = 0.4 + 0.6 * r(2.0);
                    if r(1.0) > 0.75 || age > life || age < 0.0 {
                        continue;
                    }
                    let head = tr.head - tr.dir * SPEED * (t - tb);
                    let se = head - tr.dir * LEN * r(3.0);
                    // only round the curve, where the field strains
                    if !(LN - 15.0..LN + LA + 10.0).contains(&se) {
                        continue;
                    }
                    let (x, z, tx, tz) = track(se);
                    let off = LANE * (f64::from(tr.lane) * 2.0 - 1.0);
                    let side = if r(4.0) < 0.5 { -1.0 } else { 1.0 };
                    let (nx, nz) = (tz, -tx);
                    let p0 = [x + nx * (off + side * BW), YR - BH * r(8.0), z + nz * (off + side * BW)];
                    let sp = 5.0 + 14.0 * r(5.0);
                    let out = 1.0 + 4.0 * r(6.0);
                    let v = [tx * tr.dir * sp + nx * side * out, -0.5 + 2.5 * r(7.0), tz * tr.dir * sp + nz * side * out];
                    let at = |a: f64| [p0[0] + v[0] * a, p0[1] + v[1] * a - 4.9 * a * a, p0[2] + v[2] * a];
                    let k = age / life;
                    let col = [1.0, mix(0.85, 0.25, k.sqrt()), mix(0.45, 0.02, k.sqrt())];
                    let g = 2.2 * (1.0 - k * k);
                    for q in 0..6 {
                        let p = at((age - 0.16 * q as f64 / 5.0).max(0.0));
                        if p[1] < 0.0 || p[2] < 1.0 {
                            continue;
                        }
                        let (qx, qy) = project(p[0], p[1], p[2]);
                        let gq = g * (1.0 - 0.15 * q as f64);
                        self.add(qx, qy, p[2], [col[0] * gq, col[1] * gq, col[2] * gq]);
                        if p[2] < 32.0 {
                            self.add(qx + 1.0, qy, p[2], [col[0] * gq * 0.6, col[1] * gq * 0.6, col[2] * gq * 0.6]);
                        }
                    }
                }
            }
        }
    }

    /// Steam from vents in the street, rising and drifting, lit by whatever
    /// glows round it.
    fn steam(&mut self, t: f64, lights: &[[f64; 3]]) {
        let vents = [(-7.0, 52.0, 1.0), (7.5, 86.0, 0.9), (13.5, 23.0, 0.8)];
        for (vi, &(vx, vz, size)) in vents.iter().enumerate() {
            let sc = F / vz;
            let (qx, qy) = project(vx, 0.0, vz);
            let tall = 15.0 * size;
            let wide = 9.0 * size;
            let (x0, x1) = ((qx - wide * sc).max(0.0) as usize, ((qx + wide * sc).max(0.0) as usize).min(W));
            let (y0, y1) = ((qy - tall * sc).max(0.0) as usize, (qy.max(0.0) as usize).min(H));
            let seed = vi as f64 * 17.0;
            for py in y0..y1 {
                for px in x0..x1 {
                    let k = py * W + px;
                    if f64::from(self.hit[k].depth) < vz {
                        continue;
                    }
                    let h = (qy - py as f64) / sc;
                    let lean = 0.15 * h + 0.6 * (h * 0.25 - t * 0.7 + seed).sin() * (h / tall);
                    let dx = (px as f64 - qx) / sc - lean;
                    let w = 0.9 * size + 0.45 * h;
                    let n = fbm(dx * 0.45 + seed, h * 0.3 - t * 1.1 / size, 3);
                    let dens = smooth(0.3, 0.62, n) * (-(dx / w).powi(2)).exp() * smooth(0.0, 1.2, h) * smooth(tall, tall * 0.3, h);
                    if dens < 0.01 {
                        continue;
                    }
                    let gl = self.glow(px, py);
                    let p = [vx as f32, h as f32, vz as f32];
                    let l = if lights.is_empty() { 0.0 } else { wash(lights, p, 4) * 0.4 } as f32;
                    let sc_col = [0.3 + gl[0] * 4.0 + l * 0.6, 0.26 + gl[1] * 4.0 + l * 0.85, 0.38 + gl[2] * 4.0 + l];
                    let a = (dens * 0.8).min(0.85) as f32;
                    for (o, v) in self.buf[k].iter_mut().zip(sc_col) {
                        *o = *o * (1.0 - a) + v * a;
                    }
                }
            }
        }
    }

    /// The walkway's rail along the bottom: a wet steel bar, a strip of
    /// magenta light under it, glass below with drops sliding down it.
    fn railing(&mut self, t: f64) {
        let top = RAIL;
        for px in 0..W {
            // the bar catches the light above it
            let above = self.buf[(top - 6) * W + px];
            for py in top..H {
                let k = py * W + px;
                let c = self.buf[k];
                let post = (54..59).contains(&px) || (262..267).contains(&px);
                let r = py - top;
                self.buf[k] = if r < 4 || post {
                    let sheen = if r == 0 || (post && (px == 54 || px == 262)) { 0.35 } else { 0.08 / (1.0 + r as f32) };
                    [0.035 + above[0] * sheen, 0.03 + above[1] * sheen, 0.055 + above[2] * sheen]
                } else if r == 4 {
                    let p = 0.8 + 0.2 * ((px as f64 * 0.05 - t * 2.0).sin() as f32);
                    [MAGENTA[0] as f32 * p, MAGENTA[1] as f32 * p, MAGENTA[2] as f32 * p]
                } else {
                    // through the glass: darker, colder, and lit pink just under the strip
                    let g = (0.6 * (-(r as f32 - 5.0) / 2.0).exp()) * 0.6;
                    [c[0] * 0.5 + 0.01 + g * MAGENTA[0] as f32, c[1] * 0.55 + 0.015 + g * MAGENTA[1] as f32, c[2] * 0.65 + 0.03 + g * MAGENTA[2] as f32]
                };
            }
            // the strip's glow on what's just above the rail
            for r in 1..4 {
                let k = (top - r) * W + px;
                let g = 0.18 / r as f32;
                self.buf[k][0] += g * MAGENTA[0] as f32;
                self.buf[k][1] += g * MAGENTA[1] as f32;
                self.buf[k][2] += g * MAGENTA[2] as f32;
            }
        }
        // drops on the glass, sliding down in fits and starts
        let span = (H - top - 6) as f64;
        for i in 0..36 {
            let fi = i as f64;
            let x = (hash(fi, 61.0) * W as f64) as usize;
            let speed = 1.0 + 4.0 * hash(fi, 62.0);
            let slide = t * speed + 3.0 * (t * 0.7 + fi).sin().max(0.0);
            let y = top + 6 + ((hash(fi, 63.0) * span + slide).rem_euclid(span)) as usize;
            let k = y * W + x;
            let gl = self.glow(x, y);
            for (o, g) in self.buf[k].iter_mut().zip(gl) {
                *o = *o * 0.5 + 0.12 + g * 2.5;
            }
            if y > top + 6 {
                for c in 0..3 {
                    self.buf[k - W][c] += 0.04;
                }
            }
        }
    }

    /// The bright parts, shrunk to a quarter and blurred: the glow round
    /// every light.
    fn bloom_build(&mut self) {
        for by in 0..GH {
            for bx in 0..GW {
                let mut s = [0.0f32; 3];
                for dy in 0..4 {
                    for dx in 0..4 {
                        let c = self.buf[(by * 4 + dy) * W + bx * 4 + dx];
                        let lum = 0.3 * c[0] + 0.55 * c[1] + 0.15 * c[2];
                        let k = ((lum - 0.4) / 0.6).clamp(0.0, 1.0);
                        for i in 0..3 {
                            s[i] += c[i] * k / 16.0;
                        }
                    }
                }
                self.bloom[by * GW + bx] = s;
            }
        }
        let kern = [0.2, 0.17, 0.12, 0.07, 0.03];
        for pass in 0..2 {
            for by in 0..GH {
                for bx in 0..GW {
                    let mut s = [0.0f32; 3];
                    for (j, &w) in kern.iter().enumerate() {
                        for sg in [-1i64, 1] {
                            if j == 0 && sg > 0 {
                                continue;
                            }
                            let (x, y) = if pass == 0 { (bx as i64 + sg * j as i64, by as i64) } else { (bx as i64, by as i64 + sg * j as i64) };
                            if x < 0 || y < 0 || x >= GW as i64 || y >= GH as i64 {
                                continue;
                            }
                            let c = self.bloom[y as usize * GW + x as usize];
                            for i in 0..3 {
                                s[i] += c[i] * w;
                            }
                        }
                    }
                    self.tmp[by * GW + bx] = s;
                }
            }
            std::mem::swap(&mut self.bloom, &mut self.tmp);
        }
    }

    /// The glow added back over the picture.
    fn bloom_add(&mut self) {
        for py in 0..H {
            for px in 0..W {
                let fx = (px as f32 + 0.5) / 4.0 - 0.5;
                let fy = (py as f32 + 0.5) / 4.0 - 0.5;
                let (x0, y0) = (fx.floor().max(0.0) as usize, fy.floor().max(0.0) as usize);
                let (x1, y1) = ((x0 + 1).min(GW - 1), (y0 + 1).min(GH - 1));
                let (ax, ay) = ((fx - x0 as f32).clamp(0.0, 1.0), (fy - y0 as f32).clamp(0.0, 1.0));
                let k = py * W + px;
                for i in 0..3 {
                    let top = self.bloom[y0 * GW + x0][i] * (1.0 - ax) + self.bloom[y0 * GW + x1][i] * ax;
                    let bot = self.bloom[y1 * GW + x0][i] * (1.0 - ax) + self.bloom[y1 * GW + x1][i] * ax;
                    self.buf[k][i] += 1.4 * (top * (1.0 - ay) + bot * ay);
                }
            }
        }
    }
}
