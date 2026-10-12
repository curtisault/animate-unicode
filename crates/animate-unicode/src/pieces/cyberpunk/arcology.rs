//! arcology: a megacity at night, seen from high up. Rows of towers recede
//! into a magenta-violet smog, each row paler and softer than the one in
//! front, their feet lost in the glow of the streets below, and in the middle
//! a colossal arcology climbs in terraces through the low cloud to a needle
//! spire, its crown ringed with turning lights and a beacon at the tip.
//! Lifts run up and down its seams, and a news ticker crawls round its
//! waist.
//!
//! Flying traffic streams across in lanes at several depths, white head
//! lights one way and red tail lights the other: slow specks far off,
//! faster streaks between the towers, and close by full-size cars with
//! underglow. Holographic billboards on the towers cycle through glyphs and
//! shapes in neon, scanned and flickering, now and then tearing in a glitch,
//! and once in a while all of them surge at once. Searchlights sweep the
//! cloud, aircraft warning lights blink on every mast, and every so often a
//! patrol spinner crosses the city with its strobes going and its spotlight
//! raking the towers below.

use crate::noise::{clamp, fbm_wrap, hash, mix, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Arcology's meta.
pub static META: Meta = Meta {
    name: "arcology",
    slug: "arcology",
    category: Category::Cyberpunk,
    note: "a megacity from on high: an arcology in the smog, sky traffic, holograms",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // The night's near-blacks, violets and navies first, then the smog's
    // magentas, the neons (pink, cyan, acid green, amber, red), and the
    // whites of the lamps and lights.
    palette: Some(&[
        "#05040a", "#08060e", "#0a0712", "#0e0a1a", "#130d22", "#1a1030", "#22143c", "#2c1848", "#381c56", "#0a0c1a", "#10162a", "#182040", "#160a1c",
        "#1e0c26", "#2a1030", "#3a1a4a", "#4a2060", "#52224e", "#5e2870", "#6e2c5c", "#763080", "#8e3a8c", "#a8489a", "#c45aa8", "#7a1050", "#b0186c",
        "#e0288c", "#ff4aa8", "#ff8ccc", "#ffc8e6", "#0a3a4a", "#0e6070", "#1694a8", "#22c8dc", "#6ee8f4", "#c8fafc", "#1a5a20", "#3ca02c", "#7ee83a",
        "#d0ff8a", "#5a300a", "#a0581a", "#e08a20", "#ffb848", "#ffe0a0", "#5a0c14", "#a01822", "#e02a34", "#ff5a5a", "#fff4e8", "#ffffff", "#e8e0f8",
        "#2a2a5a", "#44407a", "#6a5ea0", "#9a88c4", "#cabce4", "#3a2440", "#5a3048", "#8a4058",
    ]),
    ground: Some("#05040a"),
    ..Meta::DEFAULT
};

/// The picture in pixels.
const W: usize = 320;
const H: usize = 160;
/// The cloud deck's and the smog's width: they wrap, so they can drift forever.
const CW: usize = 640;
/// The cloud deck's rows.
const HC: usize = 64;
/// The arcology's middle.
const AX: f64 = 160.0;

/// Depths, far to near: the sky, then each row of towers.
const SKY: f32 = 9.0;
const FAR: f32 = 4.0;
const BACK: f32 = 3.0;
const ARCO: f32 = 2.5;
const MID: f32 = 2.0;
const NEAR: f32 = 1.0;

/// What a pixel is.
const K_SKY: u8 = 0;
const K_SOLID: u8 = 1;
const K_WINDOW: u8 = 2;
const K_BOARD: u8 = 3;
const K_NEON: u8 = 4;

const PINK: [f64; 3] = [1.0, 0.22, 0.64];
const CYAN: [f64; 3] = [0.15, 0.86, 1.0];
const ACID: [f64; 3] = [0.55, 1.0, 0.2];
const AMBER: [f64; 3] = [1.0, 0.66, 0.18];
const RED: [f64; 3] = [1.0, 0.14, 0.18];
const NEONS: [[f64; 3]; 4] = [PINK, CYAN, ACID, AMBER];
/// What a lit window is: warm, cool, pale violet, pink.
const WARM: [[f64; 3]; 4] = [[1.0, 0.7, 0.34], [0.55, 0.86, 1.0], [0.86, 0.8, 1.0], [1.0, 0.5, 0.78]];

/// A row of towers: how deep it is, how hazy, where its streets glow, and its windows.
struct Layer {
    depth: f32,
    haze: f64,
    street: f64,
    body: [f64; 3],
    wdx: i64,
    wdy: i64,
    wlit: f64,
    wbright: f64,
    roof: i64,
}

const FAR_L: Layer = Layer { depth: FAR, haze: 0.66, street: 92.0, body: [0.06, 0.03, 0.1], wdx: 2, wdy: 3, wlit: 0.22, wbright: 0.55, roof: 0 };
const BACK_L: Layer = Layer { depth: BACK, haze: 0.48, street: 118.0, body: [0.05, 0.025, 0.09], wdx: 2, wdy: 3, wlit: 0.26, wbright: 0.6, roof: 1 };
const MID_L: Layer = Layer { depth: MID, haze: 0.24, street: 168.0, body: [0.035, 0.02, 0.07], wdx: 3, wdy: 3, wlit: 0.3, wbright: 0.7, roof: 2 };
const NEAR_L: Layer = Layer { depth: NEAR, haze: 0.03, street: 230.0, body: [0.02, 0.012, 0.04], wdx: 3, wdy: 4, wlit: 0.34, wbright: 0.8, roof: 0 };

#[derive(Clone, Copy, PartialEq, Eq)]
enum Style {
    /// A narrow sign down a tower's face, glyphs writing on inside a chasing frame.
    Sign,
    /// A screen cycling through shapes.
    Screen,
    /// A band of glyphs crawling sideways.
    Ticker,
}

/// A holographic billboard, and the pixels round it that its glow reaches.
struct Board {
    x0: usize,
    y0: usize,
    w: usize,
    h: usize,
    depth: f32,
    style: Style,
    hue: usize,
    phase: f64,
    halo: Vec<(u32, f32)>,
}

/// A board's light this frame.
#[derive(Clone, Copy)]
struct Show {
    mode: usize,
    age: f64,
    col: [f64; 3],
    level: f64,
    glitch: bool,
    slot: f64,
}

/// An aircraft warning light (or a rooftop lamp): where, how deep, and its blink.
struct Light {
    x: usize,
    y: usize,
    depth: f32,
    period: f64,
    phase: f64,
    col: [f64; 3],
}

/// A lane of flying traffic.
struct Lane {
    y0: f64,
    amp: f64,
    freq: f64,
    depth: f32,
    fog: f64,
    speed: f64,
    count: usize,
    size: u8,
    seed: f64,
}

const LANES: [Lane; 9] = [
    Lane { y0: 63.0, amp: 1.0, freq: 0.02, depth: 3.6, fog: 0.5, speed: -6.0, count: 22, size: 1, seed: 1.0 },
    Lane { y0: 65.0, amp: 1.0, freq: 0.02, depth: 3.6, fog: 0.5, speed: 5.0, count: 22, size: 1, seed: 2.0 },
    Lane { y0: 79.0, amp: 2.5, freq: 0.017, depth: 2.7, fog: 0.4, speed: -11.0, count: 18, size: 1, seed: 3.0 },
    Lane { y0: 82.0, amp: 2.5, freq: 0.017, depth: 2.7, fog: 0.4, speed: 10.0, count: 18, size: 1, seed: 4.0 },
    Lane { y0: 97.0, amp: 3.5, freq: 0.013, depth: 2.2, fog: 0.25, speed: 18.0, count: 16, size: 2, seed: 5.0 },
    Lane { y0: 101.0, amp: 3.5, freq: 0.013, depth: 2.2, fog: 0.25, speed: -16.0, count: 16, size: 2, seed: 6.0 },
    Lane { y0: 121.0, amp: 3.0, freq: 0.011, depth: 1.5, fog: 0.12, speed: 30.0, count: 9, size: 3, seed: 7.0 },
    Lane { y0: 125.0, amp: 3.0, freq: 0.011, depth: 1.5, fog: 0.12, speed: -27.0, count: 9, size: 3, seed: 8.0 },
    Lane { y0: 150.0, amp: 2.0, freq: 0.009, depth: 0.5, fog: 0.0, speed: -58.0, count: 4, size: 4, seed: 9.0 },
];

/// A searchlight: its foot, depth, the angle it leans at, and its sweep.
struct Beam {
    x: f64,
    y: f64,
    depth: f32,
    lean: f64,
    sweep: f64,
    rate: f64,
    phase: f64,
}

const BEAMS: [Beam; 4] = [
    Beam { x: 52.0, y: 104.0, depth: MID, lean: -0.3, sweep: 0.32, rate: 0.19, phase: 0.4 },
    Beam { x: 128.0, y: 112.0, depth: MID, lean: 0.15, sweep: 0.28, rate: 0.13, phase: 2.6 },
    Beam { x: 226.0, y: 104.0, depth: MID, lean: 0.25, sweep: 0.3, rate: 0.16, phase: 4.1 },
    Beam { x: 300.0, y: 121.0, depth: NEAR, lean: -0.35, sweep: 0.22, rate: 0.11, phase: 1.0 },
];

/// The patrol spinner's round, and how long it is in sight.
const PATROL: f64 = 26.0;
const PATROL_SEEN: f64 = 12.5;

/// The arcology's half width at height y, in terraces (0 above the spire).
fn arco_half(y: f64) -> f64 {
    const TIERS: [(f64, f64, f64, f64); 7] = [
        (-1.0, 9.0, 0.5, 0.6),
        (9.0, 22.0, 2.0, 4.0),
        (22.0, 40.0, 6.0, 8.0),
        (40.0, 66.0, 12.0, 15.0),
        (66.0, 96.0, 20.0, 24.0),
        (96.0, 128.0, 31.0, 35.0),
        (128.0, 161.0, 44.0, 48.0),
    ];
    for &(a, b, wa, wb) in &TIERS {
        if y >= a && y < b {
            return mix(wa, wb, (y - a) / (b - a));
        }
    }
    0.0
}

/// The terrace tops, where the arcology steps in.
const TERRACES: [f64; 6] = [9.0, 22.0, 40.0, 66.0, 96.0, 128.0];

/// Katakana-ish glyphs, 5 × 5, a bit a cell, made of a few strokes each.
fn glyphs() -> Vec<u32> {
    let strokes: [&[(u32, u32)]; 10] = [
        &[(0, 0), (1, 0), (2, 0), (3, 0), (4, 0)],
        &[(0, 2), (1, 2), (2, 2), (3, 2), (4, 2)],
        &[(0, 4), (1, 4), (2, 4), (3, 4), (4, 4)],
        &[(2, 0), (2, 1), (2, 2), (2, 3), (2, 4)],
        &[(1, 0), (1, 1), (1, 2), (1, 3), (0, 4)],
        &[(3, 0), (3, 1), (3, 2), (3, 3), (3, 4)],
        &[(4, 0), (4, 1), (3, 2), (2, 3), (0, 4)],
        &[(1, 1), (2, 2), (3, 3), (4, 4)],
        &[(0, 1), (1, 1), (2, 1), (3, 1), (4, 1), (4, 2)],
        &[(0, 3), (1, 3), (2, 3), (3, 3), (4, 3)],
    ];
    (0..40)
        .map(|i| {
            let mut m = 0u32;
            let n = 2 + (hash(i as f64, 71.0) * 2.2) as usize;
            for s in 0..n {
                for &(x, y) in strokes[(hash(i as f64, 72.0 + s as f64) * 10.0) as usize % 10] {
                    m |= 1 << (y * 5 + x);
                }
            }
            m
        })
        .collect()
}

/// A glyph's bit at (x, y), off outside it.
fn bit(g: u32, x: i64, y: i64) -> bool {
    (0..5).contains(&x) && (0..5).contains(&y) && g >> (y * 5 + x) & 1 == 1
}

/// The air at a pixel: the night sky up top, the smog's violet to magenta at
/// the skyline, warm with street light below, brightest round the arcology.
fn air(x: f64, y: f64) -> [f64; 3] {
    let v = clamp(y / 66.0);
    let low = smooth(66.0, 160.0, y);
    let glow = (-((x - AX) / 95.0).powi(2)).exp() * smooth(8.0, 70.0, y);
    // the band of light the streets throw up into the smog
    let band = (-((y - 80.0) / 22.0).powi(2)).exp();
    [
        0.025 + 0.3 * v * v + 0.14 * glow + 0.1 * low + 0.12 * band,
        0.016 + 0.07 * v * v + 0.04 * glow + 0.06 * low + 0.07 * band,
        0.055 + 0.32 * v * v + 0.12 * glow - 0.06 * low + 0.06 * band,
    ]
}

/// The city: each pixel's depth, colour, haze and kind, the cloud deck and
/// the smog's drift, the boards and lights, and the picture.
pub struct Arcology {
    depth: Vec<f32>,
    base: Vec<[f32; 3]>,
    fog: Vec<f32>,
    kind: Vec<u8>,
    arg: Vec<u8>,
    air: Vec<[f32; 3]>,
    cover: Vec<f32>,
    lit: Vec<f32>,
    smog: Vec<f32>,
    boards: Vec<Board>,
    lights: Vec<Light>,
    glyphs: Vec<u32>,
    out: Vec<[f64; 3]>,
    pixels: Pixels,
}

/// Paints the city into its buffers, back to front.
struct Painter {
    depth: Vec<f32>,
    base: Vec<[f32; 3]>,
    fog: Vec<f32>,
    kind: Vec<u8>,
    arg: Vec<u8>,
    lights: Vec<Light>,
}

impl Painter {
    fn put(&mut self, x: i64, y: i64, depth: f32, c: [f64; 3], fog: f64, kind: u8) {
        if !(0..W as i64).contains(&x) || !(0..H as i64).contains(&y) {
            return;
        }
        let k = y as usize * W + x as usize;
        if depth > self.depth[k] {
            return;
        }
        self.depth[k] = depth;
        self.base[k] = [c[0] as f32, c[1] as f32, c[2] as f32];
        self.fog[k] = fog as f32;
        self.kind[k] = kind;
    }

    /// One tower of a row: `style` 0 flat, 1 set back, 2 slanted, 3 stepped
    /// to a point; with windows, rim light, and maybe a mast.
    fn tower(&mut self, l: &Layer, x0: f64, w: f64, top: f64, style: u32, seed: f64) {
        let tone = WARM[(hash(seed, 3.0) * 4.0) as usize % 4];
        let alt = WARM[(hash(seed, 4.0) * 4.0) as usize % 4];
        let lit = l.wlit * (0.5 + hash(seed, 5.0));
        let bands = hash(seed, 6.0) > 0.78;
        let xa = x0.floor() as i64;
        let xb = (x0 + w).ceil() as i64;
        for px in xa.max(0)..xb.min(W as i64) {
            let fx = px as f64 + 0.5 - x0;
            if fx < 0.0 || fx > w {
                continue;
            }
            let ttop = match style {
                1 => {
                    if fx < w * 0.22 || fx > w * 0.78 {
                        top + w * 0.5 + 2.0
                    } else {
                        top
                    }
                }
                2 => top + fx / w * w * 0.45,
                3 => top + ((fx - w / 2.0).abs() / 2.0).floor() * 2.0,
                _ => top,
            };
            let y0 = ttop.ceil() as i64;
            // the roof's plane, seen from above
            if fx > 1.0 && fx < w - 1.0 {
                for py in y0 - l.roof..y0 {
                    let e = if py == y0 - l.roof { 0.07 } else { 0.03 };
                    self.put(px, py, l.depth, [l.body[0] + e, l.body[1] + e * 0.7, l.body[2] + e * 1.4], l.haze, K_SOLID);
                }
            }
            for py in y0.max(0)..H as i64 {
                let y = py as f64;
                let fog = l.haze + (1.0 - l.haze) * 0.92 * smooth(l.street - 50.0, l.street, y);
                let shade = 0.8 + 0.4 * smooth(y0 as f64 + 20.0, y0 as f64, y);
                let mut c = [l.body[0] * shade, l.body[1] * shade, l.body[2] * shade];
                let mut kind = K_SOLID;
                if fx < 1.0 {
                    // the left edge catches the magenta of the signs
                    c = [c[0] + 0.12, c[1] + 0.02, c[2] + 0.1];
                } else if fx > w - 1.0 {
                    // the right edge a cold cyan
                    c = [c[0] + 0.02, c[1] + 0.08, c[2] + 0.12];
                } else if py == y0 {
                    c = [c[0] + 0.06, c[1] + 0.04, c[2] + 0.1];
                } else {
                    let lx = px - xa - 1;
                    let ly = py - top.ceil() as i64 - 2;
                    if bands && ly > 0 && ly % 7 == 0 {
                        let s = l.wbright * 0.75;
                        c = [alt[0] * s, alt[1] * s, alt[2] * s];
                        kind = K_NEON;
                    } else if lx >= 0 && ly >= 0 && lx % l.wdx == 0 && ly % l.wdy == 0 && hash(seed * 31.0 + lx as f64, ly as f64) < lit {
                        let s = l.wbright * (0.55 + 0.45 * hash(lx as f64, seed + ly as f64));
                        let tc = if hash(seed + lx as f64, ly as f64 * 3.0) > 0.85 { alt } else { tone };
                        c = [tc[0] * s, tc[1] * s, tc[2] * s];
                        kind = K_WINDOW;
                    }
                }
                self.put(px, py, l.depth, c, fog, kind);
            }
        }
        // a mast with a warning light on the taller ones
        if hash(seed, 8.0) > 0.45 && style != 3 {
            let mx = (x0 + w * (0.3 + 0.4 * hash(seed, 9.0))).floor() as i64;
            let mh = (3.0 + hash(seed, 10.0) * (w * 0.6).min(10.0)) as i64;
            let ty = top.ceil() as i64;
            for py in ty - mh..ty {
                let fog = l.haze;
                self.put(mx, py, l.depth, [0.06, 0.04, 0.1], fog, K_SOLID);
            }
            if mx >= 0 && (mx as usize) < W && ty - mh > 0 {
                self.lights.push(Light {
                    x: mx as usize,
                    y: (ty - mh - 1) as usize,
                    depth: l.depth,
                    period: 1.4 + hash(seed, 11.0) * 1.2,
                    phase: hash(seed, 12.0),
                    col: RED,
                });
            }
        }
    }

    /// A procedural row of towers across the whole width.
    fn row(&mut self, l: &Layer, tops: (f64, f64), widths: (f64, f64), climb: f64, seed: f64) {
        let mut x = -3.0 - hash(seed, 1.0) * 4.0;
        let mut i = 0.0;
        while x < W as f64 {
            let s = seed * 100.0 + i;
            let w = widths.0 + hash(s, 1.0) * (widths.1 - widths.0);
            // taller toward the arcology, as a city climbs to its centre
            let rise = climb * smooth(150.0, 0.0, (x + w / 2.0 - AX).abs());
            let top = tops.0 + clamp(hash(s, 2.0) * (1.0 - rise) - rise.min(0.0) * 0.6) * (tops.1 - tops.0);
            let style = (hash(s, 7.0) * 4.0) as u32;
            self.tower(l, x, w, top, style, s);
            x += w + hash(s, 13.0) * 3.0 - 0.5;
            i += 1.0;
        }
    }

    /// The arcology: terraces of lit floors climbing to a needle spire.
    fn arcology(&mut self) {
        let haze = 0.12;
        for py in 0..H {
            let y = py as f64;
            let hw = arco_half(y);
            if hw <= 0.0 {
                continue;
            }
            let fog = haze + (1.0 - haze) * 0.8 * smooth(110.0, 175.0, y);
            for px in 0..W {
                let x = px as f64 + 0.5;
                let dx = x - AX;
                if dx.abs() > hw {
                    continue;
                }
                let u = dx / hw; // -1 to 1 across
                                 // a rounded body: darker toward the edges, the right lit cyan, the left magenta
                let round = (1.0 - u * u).max(0.0).sqrt();
                let mut c = [0.02 + 0.03 * round, 0.014 + 0.02 * round, 0.045 + 0.05 * round];
                let mut kind = K_SOLID;
                let rim = (1.5 / hw).min(0.5);
                if u < -1.0 + rim * 2.0 {
                    c = [c[0] + 0.5, c[1] + 0.08, c[2] + 0.4];
                } else if u > 1.0 - rim * 2.0 {
                    c = [c[0] + 0.06, c[1] + 0.4, c[2] + 0.5];
                } else if u < -0.75 {
                    c = [c[0] + 0.14 * (-u - 0.75) / 0.25, c[1] + 0.01, c[2] + 0.1 * (-u - 0.75) / 0.25];
                } else if u > 0.75 {
                    c = [c[0] + 0.02, c[1] + 0.1 * (u - 0.75) / 0.25, c[2] + 0.14 * (u - 0.75) / 0.25];
                }
                let terrace = TERRACES.iter().any(|&ty| (y - ty).abs() < 0.6);
                if terrace {
                    // a ledge of amber lamps at each step
                    let on = (px % 3) == 0;
                    c = if on { [1.0 * 0.9, 0.62 * 0.9, 0.24 * 0.9] } else { [0.16, 0.09, 0.12] };
                    kind = if on { K_WINDOW } else { K_SOLID };
                } else if hw > 3.0 {
                    let col = (dx + 200.0).floor() as i64;
                    let rib = col.rem_euclid(6) == 0;
                    let seam = [-18, -9, 0, 9, 18].contains(&(dx.floor() as i64));
                    if seam && y > 24.0 {
                        c = [0.14, 0.2, 0.32];
                    } else if rib {
                        c = [c[0] + 0.03, c[1] + 0.025, c[2] + 0.05];
                    } else if py % 2 == 0 && col.rem_euclid(2) == 1 {
                        let h = hash(col as f64, y * 1.7);
                        // dense small lights: the arcology is a city of its own
                        if h < 0.42 * (0.4 + 0.6 * round) {
                            let tc = if hash(col as f64 * 3.0, y) > 0.8 { WARM[1] } else { WARM[0] };
                            let s = 0.4 + 0.4 * round * hash(y, col as f64);
                            c = [tc[0] * s, tc[1] * s, tc[2] * s];
                            kind = K_WINDOW;
                        }
                    }
                } else if hw > 1.2 {
                    // the crown's struts
                    c = [0.12 + 0.1 * round, 0.08, 0.2 + 0.1 * round];
                } else {
                    c = [0.24, 0.18, 0.34];
                }
                self.put(px as i64, py as i64, ARCO, c, fog, kind);
            }
        }
        // red lights at each terrace's corners
        for (i, &ty) in TERRACES.iter().enumerate().skip(1) {
            let hw = arco_half(ty + 0.5);
            for side in [-1.0, 1.0] {
                let x = (AX + side * (hw - 1.0)).floor() as usize;
                self.lights.push(Light { x, y: ty as usize - 1, depth: ARCO, period: 2.0, phase: i as f64 * 0.13, col: RED });
            }
        }
    }

    /// One of the near towers, built by hand: a rooftop seen from above, with
    /// parapet, plant, and lamps.
    fn rooftop(&mut self, x0: i64, w: i64, top: i64, seed: f64) {
        // the roof's plane, seen from above, a little lighter than the walls
        for py in top - 3..top {
            for px in x0 + 1..x0 + w - 1 {
                let edge = py == top - 3;
                let c = if edge { [0.1, 0.07, 0.14] } else { [0.045, 0.03, 0.07] };
                self.put(px, py, NEAR, c, 0.05, K_SOLID);
            }
        }
        // plant on the roof: a box and a tank
        let bx = x0 + 3 + (hash(seed, 1.0) * (w - 10) as f64) as i64;
        for py in top - 6..top - 2 {
            for px in bx..bx + 4 {
                let c = if py == top - 6 { [0.12, 0.08, 0.16] } else { [0.04, 0.03, 0.06] };
                self.put(px, py, NEAR, c, 0.04, K_SOLID);
            }
        }
        // a mast at one end, its light blinking
        let mx = x0 + w - 4 - (hash(seed, 2.0) * 4.0) as i64;
        let mh = 8 + (hash(seed, 3.0) * 8.0) as i64;
        for py in top - 3 - mh..top - 3 {
            self.put(mx, py, NEAR, [0.08, 0.06, 0.12], 0.03, K_SOLID);
        }
        self.lights.push(Light { x: mx as usize, y: (top - 4 - mh) as usize, depth: NEAR, period: 1.6 + hash(seed, 4.0), phase: hash(seed, 5.0), col: RED });
        // lamps on the parapet's corners
        for px in [x0 + 1, x0 + w - 2] {
            self.lights.push(Light { x: px as usize, y: (top - 3) as usize, depth: NEAR, period: 2.4, phase: hash(seed, px as f64), col: [1.0, 0.9, 0.7] });
        }
    }

    /// A vertical neon strip down a tower's edge.
    fn strip(&mut self, x: i64, y0: i64, y1: i64, depth: f32, col: [f64; 3], fog: f64) {
        for py in y0..y1 {
            self.put(x, py, depth, [col[0] * 0.9, col[1] * 0.9, col[2] * 0.9], fog, K_NEON);
        }
    }
}

/// An arcology over a smoggy megacity; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let mut p = Painter { depth: vec![SKY; n], base: vec![[0.0; 3]; n], fog: vec![0.0; n], kind: vec![K_SKY; n], arg: vec![0; n], lights: vec![] };

    // back to front: far, back, the arcology, mid, near
    p.row(&FAR_L, (48.0, 64.0), (3.0, 9.0), 0.35, 1.0);
    p.row(&BACK_L, (60.0, 86.0), (6.0, 14.0), 0.35, 2.0);
    p.arcology();
    p.row(&MID_L, (82.0, 116.0), (10.0, 20.0), -0.7, 3.0);
    // mid towers that carry boards
    p.tower(&MID_L, 14.0, 20.0, 86.0, 0, 301.0);
    p.tower(&MID_L, 62.0, 18.0, 80.0, 0, 302.0);
    p.tower(&MID_L, 202.0, 16.0, 92.0, 0, 303.0);
    p.tower(&MID_L, 262.0, 20.0, 84.0, 1, 304.0);
    p.strip(62, 80, 160, MID, PINK, 0.24);
    p.strip(281, 84, 160, MID, CYAN, 0.24);

    // the near towers, by hand: x, width, roof
    const NEARS: [(i64, i64, i64); 6] = [(-2, 40, 110), (46, 30, 132), (100, 24, 142), (194, 32, 138), (236, 42, 106), (284, 38, 126)];
    for (i, &(x0, w, top)) in NEARS.iter().enumerate() {
        p.tower(&NEAR_L, x0 as f64, w as f64, top as f64, 0, 400.0 + i as f64);
        p.rooftop(x0, w, top, 500.0 + i as f64);
    }
    p.strip(236, 106, 160, NEAR, CYAN, 0.03);
    p.strip(37, 110, 160, NEAR, PINK, 0.03);
    p.strip(225, 138, 160, NEAR, ACID, 0.03);

    let mut boards = vec![
        // near: a tall sign down the left tower, a big screen and a small one on the right
        Board { x0: 26, y0: 115, w: 9, h: 38, depth: NEAR, style: Style::Sign, hue: 0, phase: 2.0, halo: vec![] },
        Board { x0: 243, y0: 113, w: 28, h: 22, depth: NEAR, style: Style::Screen, hue: 1, phase: 0.0, halo: vec![] },
        Board { x0: 291, y0: 133, w: 16, h: 12, depth: NEAR, style: Style::Screen, hue: 2, phase: 11.0, halo: vec![] },
        Board { x0: 52, y0: 138, w: 18, h: 9, depth: NEAR, style: Style::Ticker, hue: 3, phase: 1.0, halo: vec![] },
        // mid: smaller screens and a sign
        Board { x0: 17, y0: 92, w: 13, h: 12, depth: MID, style: Style::Screen, hue: 3, phase: 5.0, halo: vec![] },
        Board { x0: 65, y0: 85, w: 7, h: 26, depth: MID, style: Style::Sign, hue: 1, phase: 1.7, halo: vec![] },
        Board { x0: 205, y0: 97, w: 10, h: 10, depth: MID, style: Style::Screen, hue: 0, phase: 15.0, halo: vec![] },
        Board { x0: 266, y0: 90, w: 13, h: 13, depth: MID, style: Style::Screen, hue: 2, phase: 9.5, halo: vec![] },
        // the arcology's ticker, round its waist
        Board { x0: 140, y0: 74, w: 41, h: 7, depth: ARCO, style: Style::Ticker, hue: 3, phase: 0.0, halo: vec![] },
    ];
    for (i, b) in boards.iter_mut().enumerate() {
        for py in b.y0..b.y0 + b.h {
            for px in b.x0..b.x0 + b.w {
                let k = py * W + px;
                if p.depth[k] >= b.depth {
                    p.depth[k] = b.depth;
                    p.kind[k] = K_BOARD;
                    p.arg[k] = i as u8;
                }
            }
        }
        // the glow round it, on what is level with it or behind
        let r = 7i64;
        for py in b.y0 as i64 - r..(b.y0 + b.h) as i64 + r {
            for px in b.x0 as i64 - r..(b.x0 + b.w) as i64 + r {
                if !(0..W as i64).contains(&px) || !(0..H as i64).contains(&py) {
                    continue;
                }
                let dx = (b.x0 as f64 - px as f64).max(px as f64 - (b.x0 + b.w - 1) as f64).max(0.0);
                let dy = (b.y0 as f64 - py as f64).max(py as f64 - (b.y0 + b.h - 1) as f64).max(0.0);
                let d = dx.hypot(dy);
                let k = py as usize * W + px as usize;
                if d > 0.0 && d < r as f64 && p.depth[k] >= b.depth {
                    b.halo.push((k as u32, (0.3 * (-d / 2.6).exp()) as f32));
                }
            }
        }
    }

    let air = (0..n).map(|k| air((k % W) as f64 + 0.5, (k / W) as f64 + 0.5).map(|v| v as f32)).collect();

    // The low cloud: heaps of noise that wrap, lit from beneath by the city.
    let density = |x: f64, y: f64| {
        let q = fbm_wrap(x * 0.0125, y * 0.03, 3, 8.0);
        let d = fbm_wrap(x * 0.025 + q * 2.2, y * 0.06 + q * 1.3, 5, 16.0);
        d + 0.1 * smooth(8.0, 22.0, y) * smooth(56.0, 34.0, y) - 0.14 * smooth(12.0, 0.0, y) - 0.2 * smooth(46.0, 62.0, y)
    };
    let mut cover = vec![0.0f32; CW * HC];
    let mut lit = vec![0.0f32; CW * HC];
    for py in 0..HC {
        let y = py as f64 + 0.5;
        for x in 0..CW {
            let d = density(x as f64, y);
            cover[py * CW + x] = smooth(0.5, 0.64, d) as f32;
            let below = density(x as f64, y + 3.0);
            lit[py * CW + x] = clamp(0.45 + (d - below) * 9.0 + 0.4 * smooth(24.0, 50.0, y)) as f32;
        }
    }
    // the smog's drift: slow swells of haze
    let smog = (0..CW * H).map(|k| fbm_wrap((k % CW) as f64 * 0.0125, (k / CW) as f64 * 0.045, 3, 8.0) as f32).collect();

    Box::new(Arcology {
        depth: p.depth,
        base: p.base,
        fog: p.fog,
        kind: p.kind,
        arg: p.arg,
        air,
        cover,
        lit,
        smog,
        boards,
        lights: p.lights,
        glyphs: glyphs(),
        out: vec![[0.0; 3]; n],
        pixels: Pixels::new(&META),
    })
}

impl Arcology {
    /// The cloud's cover and light at a pixel, drifted.
    fn cloud(&self, px: usize, py: usize, drift: f64) -> (f64, f64) {
        if py >= HC {
            return (0.0, 0.0);
        }
        let sx = px as f64 + drift;
        let ix = sx.floor();
        let fx = sx - ix;
        let i0 = (ix as i64).rem_euclid(CW as i64) as usize;
        let (a, b) = (py * CW + i0, py * CW + (i0 + 1) % CW);
        let c = f64::from(self.cover[a]) + f64::from(self.cover[b] - self.cover[a]) * fx;
        let l = f64::from(self.lit[a]) + f64::from(self.lit[b] - self.lit[a]) * fx;
        (c, l)
    }

    /// How a board shows this frame.
    fn show(b: &Board, i: usize, t: f64) -> Show {
        let tt = t + b.phase;
        let (cycle, mode) = match b.style {
            Style::Screen => (4.5, 4),
            Style::Sign => (4.0, 1),
            Style::Ticker => (6.0, 2),
        };
        let n = (tt / cycle).floor();
        let age = tt - n * cycle;
        let mode_i = (n.rem_euclid(mode as f64)) as usize;
        let slot = (t * 6.0).floor();
        let surge = t.rem_euclid(13.0);
        let glitch = hash(slot, i as f64 * 7.0 + 3.0) > 0.95 || age < 0.2 || (9.0..9.4).contains(&surge);
        let hue = if b.style == Style::Screen { (b.hue + mode_i) % 4 } else { b.hue };
        let mut col = NEONS[hue];
        if glitch && hash(slot, i as f64 + 40.0) > 0.5 {
            col = NEONS[(hue + 1) % 4];
        }
        let flicker = 0.88 + 0.12 * hash((t * 15.0).floor(), i as f64 + 9.0);
        let level = if (9.0..9.4).contains(&surge) { 1.35 } else { flicker };
        Show { mode: mode_i, age, col, level, glitch, slot }
    }

    /// A board's light at local (u, v), 0–1.
    fn content(&self, b: &Board, i: usize, s: &Show, u: i64, v: i64, t: f64) -> f64 {
        let (w, h) = (b.w as i64, b.h as i64);
        if u < 0 || u >= w {
            return 0.05;
        }
        match b.style {
            Style::Sign => {
                if u == 0 || u == w - 1 || v == 0 || v == h - 1 {
                    // a frame of chasing bulbs
                    let p = if v == 0 {
                        u
                    } else if u == w - 1 {
                        w + v
                    } else if v == h - 1 {
                        2 * w + h - u
                    } else {
                        2 * w + 2 * h - v
                    };
                    return if (p + (t * 9.0) as i64).rem_euclid(4) == 0 { 1.0 } else { 0.3 };
                }
                let set = ((t + b.phase) / 4.0).floor();
                // written on from the top as each set comes up
                if (v as f64) > s.age * 50.0 {
                    return 0.06;
                }
                let gx = u - (w - 5) / 2;
                let gy = v - 2;
                let g = self.glyphs[(hash(set * 5.0 + (gy / 6) as f64, i as f64) * 40.0) as usize % 40];
                if bit(g, gx, gy.rem_euclid(6)) && gy >= 0 && gy / 6 < (h - 3) / 6 {
                    1.0
                } else {
                    0.08
                }
            }
            Style::Ticker => {
                if v == 0 || v == h - 1 {
                    return 0.35;
                }
                let us = u + (t * 11.0) as i64;
                let gi = us.div_euclid(6);
                let g = self.glyphs[(hash(gi as f64, i as f64 * 3.0) * 40.0) as usize % 40];
                let gy = v - (h - 5) / 2;
                if bit(g, us.rem_euclid(6), gy) {
                    1.0
                } else {
                    0.07
                }
            }
            Style::Screen => {
                let (cx, cy) = ((w - 1) as f64 / 2.0, (h - 1) as f64 / 2.0);
                let (x, y) = (u as f64 - cx, v as f64 - cy);
                let r = cx.min(cy) * 0.85;
                let a = s.age;
                let on = match s.mode {
                    0 => {
                        // a triangle turning in a ring
                        let rot = t * 1.4;
                        let mut d: f64 = 9.0;
                        for j in 0..3 {
                            let (a0, a1) = (rot + j as f64 * TAU / 3.0, rot + (j + 1) as f64 * TAU / 3.0);
                            let (x0, y0, x1, y1) = (a0.cos() * r, a0.sin() * r, a1.cos() * r, a1.sin() * r);
                            let (ex, ey) = (x1 - x0, y1 - y0);
                            let k = clamp(((x - x0) * ex + (y - y0) * ey) / (ex * ex + ey * ey));
                            d = d.min((x - x0 - ex * k).hypot(y - y0 - ey * k));
                        }
                        let ring = (x.hypot(y) - r - 1.2).abs();
                        if d < 0.75 {
                            1.0
                        } else if ring < 0.5 {
                            0.45
                        } else {
                            0.0
                        }
                    }
                    1 => {
                        // rings pulsing out from the middle
                        let d = x.hypot(y);
                        let f = (d - a * 7.0) / 4.0;
                        if f - f.floor() < 0.3 && d <= r + 1.0 {
                            1.0 - 0.5 * d / (r + 1.0)
                        } else {
                            0.0
                        }
                    }
                    2 => {
                        // a big glyph, two pixels a stroke, changing twice a second
                        let g = self.glyphs[(hash((t * 2.0).floor(), i as f64 * 5.0) * 40.0) as usize % 40];
                        let gx = ((x + 5.0) / 2.0).floor() as i64;
                        let gy = ((y + 5.0) / 2.0).floor() as i64;
                        if bit(g, gx, gy) {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    _ => {
                        // a coin turning: a diamond squeezed by its spin
                        let sx = (t * 2.6).cos().abs().max(0.08) * r;
                        let e = x.abs() / sx + y.abs() / r;
                        if (e - 1.0).abs() < 0.16 / sx.min(r) * r || e < 0.35 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                };
                let edge = u == 0 || u == w - 1 || v == 0 || v == h - 1;
                if edge {
                    0.4
                } else {
                    0.14 + 0.86 * on
                }
            }
        }
    }
}

impl Piece for Arcology {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let drift_c = t * 0.9;
        let drift_s = t * 2.2;
        let shows: Vec<Show> = self.boards.iter().enumerate().map(|(i, b)| Self::show(b, i, t)).collect();
        let beams: Vec<(f64, f64, f32, f64, f64)> = BEAMS
            .iter()
            .map(|b| {
                let a = b.lean + b.sweep * (t * b.rate * TAU / 2.0 + b.phase).sin();
                (b.x, b.y, b.depth, a.sin(), a.cos())
            })
            .collect();

        for py in 0..H {
            let y = py as f64 + 0.5;
            for px in 0..W {
                let k = py * W + px;
                let x = px as f64 + 0.5;
                let d = self.depth[k];
                let kind = self.kind[k];
                let a = self.air[k];
                let air = [f64::from(a[0]), f64::from(a[1]), f64::from(a[2])];
                let mut cover = 0.0;
                let mut c;
                if kind == K_SKY {
                    c = air;
                    let (cv, l) = self.cloud(px, py, drift_c);
                    cover = cv;
                    if cv > 0.01 {
                        // shadowed violet on top, magenta-lit beneath, brighter over the arcology
                        let near = (-((x - AX) / 60.0).powi(2)).exp();
                        let b = clamp(l * (0.75 + 0.35 * near));
                        let ramp = |lo: f64, md: f64, hi: f64| if b < 0.5 { mix(lo, md, b * 2.0) } else { mix(md, hi, (b - 0.5) * 2.0) };
                        let k2 = (cv * 1.1).min(1.0);
                        c = [mix(c[0], ramp(0.05, 0.22, 0.62), k2), mix(c[1], ramp(0.03, 0.08, 0.26), k2), mix(c[2], ramp(0.1, 0.27, 0.6), k2)];
                    }
                } else {
                    let b = self.base[k];
                    c = [f64::from(b[0]), f64::from(b[1]), f64::from(b[2])];
                    match kind {
                        K_WINDOW => {
                            // now and then a window goes dark for a while, a few flicker
                            let h = hash(k as f64, 17.0);
                            let slot = (t / 5.0 + h * 9.0).floor();
                            if hash(k as f64, slot) < 0.07 {
                                c = [c[0] * 0.15, c[1] * 0.15, c[2] * 0.15];
                            } else if h > 0.992 && (t * 19.0 + h * 50.0).sin() > 0.3 {
                                c = [c[0] * 0.4, c[1] * 0.4, c[2] * 0.4];
                            }
                        }
                        K_NEON => {
                            let f = if hash((t * 15.0).floor(), (k % 7919) as f64) > 0.985 { 0.4 } else { 1.0 };
                            c = [c[0] * f, c[1] * f, c[2] * f];
                        }
                        K_BOARD => {
                            let i = self.arg[k] as usize;
                            let bd = &self.boards[i];
                            let s = shows[i];
                            let (u, v) = ((px - bd.x0) as i64, (py - bd.y0) as i64);
                            // torn rows when it glitches
                            let shift = if s.glitch && hash(v as f64, s.slot * 13.0 + i as f64) > 0.55 {
                                ((hash(v as f64 + 50.0, s.slot) - 0.5) * 9.0).round() as i64
                            } else {
                                0
                            };
                            let uu = u + shift;
                            let (lr, lg, lb) = if s.glitch {
                                (self.content(bd, i, &s, uu + 1, v, t), self.content(bd, i, &s, uu, v, t), self.content(bd, i, &s, uu - 1, v, t))
                            } else {
                                let l = self.content(bd, i, &s, uu, v, t);
                                (l, l, l)
                            };
                            // scanlines, and a bright bar rolling down
                            let roll = (t * 22.0 + b_phase(i)).rem_euclid(bd.h as f64 + 12.0);
                            let scan = if v % 2 == 1 { 0.55 } else { 1.0 } * if (v as f64 - roll).abs() < 1.0 { 1.6 } else { 1.0 };
                            let g = s.level * scan;
                            // translucent: the wall shows through under the light
                            let wall = 0.5;
                            c = [c[0] * wall + s.col[0] * lr * g + 0.04, c[1] * wall + s.col[1] * lg * g + 0.02, c[2] * wall + s.col[2] * lb * g + 0.05];
                        }
                        _ => {}
                    }
                    if d == ARCO && py < 58 {
                        // wisps of the low cloud across the spire
                        let (cv, l) = self.cloud(px, py, drift_c);
                        let k2 = cv * 0.7;
                        let b = clamp(l * 0.9);
                        c = [mix(c[0], mix(0.16, 0.6, b), k2), mix(c[1], mix(0.06, 0.24, b), k2), mix(c[2], mix(0.2, 0.58, b), k2)];
                    }
                    // the smog: thicker where it swells, drifting
                    let sm = f64::from(self.smog[py * CW + ((px as f64 + drift_s * (1.0 + f64::from(d) * 0.1)) as usize % CW)]);
                    let f = clamp(f64::from(self.fog[k]) * (0.7 + 0.6 * sm));
                    let f = if kind == K_SOLID { f } else { f * 0.75 };
                    c = [mix(c[0], air[0], f), mix(c[1], air[1], f), mix(c[2], air[2], f)];
                }
                // searchlights, in the air in front of whatever is behind them
                for &(bx, by, bd, sa, ca) in &beams {
                    if d <= bd {
                        continue;
                    }
                    let (dx, dy) = (x - bx, y - by);
                    let along = dx * sa - dy * ca;
                    if along <= 0.0 {
                        continue;
                    }
                    let perp = dx * ca + dy * sa;
                    let wd = 0.9 + along * 0.05;
                    if perp.abs() > wd * 3.0 {
                        continue;
                    }
                    let i = (-(perp / wd).powi(2)).exp() * smooth(0.0, 18.0, along) * (-along / 150.0).exp();
                    let i = i * if kind == K_SKY { 0.42 + 0.9 * cover } else { 0.2 };
                    c = [c[0] + i * 0.75, c[1] + i * 0.8, c[2] + i];
                }
                self.out[k] = c;
            }
        }

        // the boards' glow on what is round them
        for (b, s) in self.boards.iter().zip(&shows) {
            let g = s.level * 0.55;
            for &(k, w) in &b.halo {
                let o = &mut self.out[k as usize];
                let w = f64::from(w) * g;
                o[0] += s.col[0] * w;
                o[1] += s.col[1] * w;
                o[2] += s.col[2] * w;
            }
        }

        // lifts riding the arcology's seams
        for (j, sx) in [-18i64, -9, 0, 9, 18].into_iter().enumerate() {
            for car in 0..2 {
                let span = 120.0 - (sx.abs() as f64) * 2.5;
                let ph = hash(j as f64, car as f64 + 30.0);
                let p = ((t / (span / 7.0)) + ph).rem_euclid(2.0);
                let p = if p > 1.0 { 2.0 - p } else { p };
                let ly = (150.0 - p * (span)) as i64;
                let lx = (AX as i64 + sx) as usize;
                for dy in 0..2 {
                    let yy = ly + dy;
                    if (24..H as i64).contains(&yy) {
                        let k = yy as usize * W + lx;
                        if self.depth[k] == ARCO {
                            let o = &mut self.out[k];
                            *o = [mix(0.9, o[0], 0.25), mix(0.95, o[1], 0.25), mix(1.0, o[2], 0.25)];
                        }
                    }
                }
            }
        }

        // the crown: two rings round the spire, their lights turning opposite ways, and the beacon
        for &(ry0, rx, ryr, spin, count) in &[(19.0, 15.0, 3.0, 0.9, 10), (33.0, 11.0, 2.2, -0.6, 8)] {
            for j in 0..96 {
                let a = j as f64 * TAU / 96.0;
                let (ix, iy) = ((AX + a.cos() * rx).floor() as usize, (ry0 + a.sin() * ryr).floor() as usize);
                let k = iy * W + ix;
                if a.sin() <= 0.0 && self.depth[k] <= ARCO {
                    continue;
                }
                let o = &mut self.out[k];
                *o = [mix(o[0], 0.22, 0.6), mix(o[1], 0.3, 0.6), mix(o[2], 0.5, 0.6)];
            }
            for j in 0..count {
                let a = t * spin + j as f64 * TAU / count as f64;
                let (lx, ly) = (AX + a.cos() * rx, ry0 + a.sin() * ryr);
                let front = a.sin() > 0.0;
                let (ix, iy) = (lx.floor() as usize, ly.floor() as usize);
                let k = iy * W + ix;
                if !front && self.depth[k] <= ARCO {
                    continue;
                }
                let col = if j % 2 == 0 { [1.0, 0.95, 0.85] } else { CYAN };
                let g = if front { 1.0 } else { 0.55 };
                self.glow(ix, iy, 0.0, [col[0] * g, col[1] * g, col[2] * g], 1.4);
            }
        }
        let beacon = 0.5 + 0.5 * (t * TAU / 1.6).sin();
        let beacon = beacon.powi(3);
        self.glow(AX as usize, 0, 0.0, [beacon, 0.2 * beacon, 0.25 * beacon], 2.5 * beacon + 0.4);
        self.glow(AX as usize, 1, 0.0, [0.6 + beacon * 0.4, 0.15, 0.2], 0.0);

        // warning lights on the masts
        for li in 0..self.lights.len() {
            let l = &self.lights[li];
            let on = (t / l.period + l.phase).rem_euclid(1.0) < 0.4;
            let (x, y, d, col) = (l.x, l.y, l.depth, l.col);
            if self.depth[y * W + x] < d {
                continue;
            }
            let g = if on { 1.0 } else { 0.18 };
            self.glow(x, y, d, [col[0] * g, col[1] * g, col[2] * g], if on { 1.2 } else { 0.0 });
        }

        self.traffic(t);
        self.patrol(t);

        let pix = self.pixels.pixels_mut();
        for (p, c) in pix.iter_mut().zip(&self.out) {
            *p = [c[0] as f32, c[1] as f32, c[2] as f32];
        }
        self.pixels.draw(grid, env);
    }
}

/// A board's own offset for its rolling scan bar.
fn b_phase(i: usize) -> f64 {
    hash(i as f64, 91.0) * 40.0
}

impl Arcology {
    /// A light at (x, y): the pixel set, and a soft glow `r` round it on
    /// whatever is no nearer than `depth`.
    fn glow(&mut self, x: usize, y: usize, depth: f32, col: [f64; 3], r: f64) {
        if x >= W || y >= H {
            return;
        }
        self.out[y * W + x] = col;
        if r <= 0.0 {
            return;
        }
        let ri = r.ceil() as i64 + 1;
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let (px, py) = (x as i64 + dx, y as i64 + dy);
                if (dx == 0 && dy == 0) || !(0..W as i64).contains(&px) || !(0..H as i64).contains(&py) {
                    continue;
                }
                let k = py as usize * W + px as usize;
                if self.depth[k] < depth {
                    continue;
                }
                let w = 0.45 * (-((dx * dx + dy * dy) as f64).sqrt() / r).exp();
                let o = &mut self.out[k];
                o[0] += col[0] * w;
                o[1] += col[1] * w;
                o[2] += col[2] * w;
            }
        }
    }

    /// Adds light to a pixel, if nothing nearer than `depth` stands there.
    fn add(&mut self, x: i64, y: i64, depth: f32, col: [f64; 3], w: f64) {
        if !(0..W as i64).contains(&x) || !(0..H as i64).contains(&y) {
            return;
        }
        let k = y as usize * W + x as usize;
        if self.depth[k] <= depth {
            return;
        }
        let o = &mut self.out[k];
        o[0] += col[0] * w;
        o[1] += col[1] * w;
        o[2] += col[2] * w;
    }

    /// Sets a pixel, if nothing nearer than `depth` stands there.
    fn put(&mut self, x: i64, y: i64, depth: f32, col: [f64; 3]) {
        if !(0..W as i64).contains(&x) || !(0..H as i64).contains(&y) {
            return;
        }
        let k = y as usize * W + x as usize;
        if self.depth[k] > depth {
            self.out[k] = col;
        }
    }

    /// The flying traffic, lane by lane.
    fn traffic(&mut self, t: f64) {
        let span = W as f64 + 60.0;
        for l in &LANES {
            let dir = l.speed.signum();
            for i in 0..l.count {
                // bunched a little, as traffic is
                let off = (i as f64 + 0.7 * hash(i as f64, l.seed)) / l.count as f64 * span;
                let x = (off + l.speed * t).rem_euclid(span) - 30.0;
                let y = l.y0 + l.amp * (x * l.freq + l.seed).sin();
                let (xi, yi) = (x.floor() as i64, y.floor() as i64);
                let smog = |c: [f64; 3]| {
                    let a = air(x, y);
                    [mix(c[0], a[0], l.fog * 0.6), mix(c[1], a[1], l.fog * 0.6), mix(c[2], a[2], l.fog * 0.6)]
                };
                let white = smog([1.0, 0.95, 0.85]);
                let red = smog(RED);
                let trail = (l.speed.abs() * 0.18).max(2.0) as i64;
                // white heads one way, red tails the other
                let lead = if dir < 0.0 { white } else { red };
                match l.size {
                    1 | 2 => {
                        for j in 1..=trail {
                            let w = 0.55 * (1.0 - j as f64 / (trail + 1) as f64);
                            self.add(xi - j * dir as i64, yi, l.depth, lead, w);
                        }
                        self.put(xi, yi, l.depth, lead);
                        if l.size == 2 {
                            self.put(xi - dir as i64, yi, l.depth, [lead[0] * 0.7, lead[1] * 0.7, lead[2] * 0.7]);
                        }
                    }
                    _ => {
                        // a car side on: a dark body, a head lamp and a tail lamp, underglow
                        let len = if l.size == 4 { 7 } else { 4 };
                        let glow = if hash(i as f64, l.seed + 5.0) > 0.5 { PINK } else { CYAN };
                        for j in 1..=trail * 2 {
                            let w = 0.5 * (1.0 - j as f64 / (trail * 2 + 1) as f64);
                            self.add(xi - (len + j) * dir as i64, yi, l.depth, red, w);
                        }
                        for j in 0..len {
                            let xx = xi - j * dir as i64;
                            self.put(xx, yi - 1, l.depth, [0.06, 0.05, 0.1]);
                            self.put(xx, yi, l.depth, [0.1, 0.08, 0.15]);
                            if l.size == 4 {
                                self.put(xx, yi - 2, l.depth, if j > 1 && j < len - 2 { [0.18, 0.28, 0.4] } else { [0.05, 0.04, 0.08] });
                            }
                            self.add(xx, yi + 1, l.depth, glow, 0.6);
                            self.add(xx, yi + 2, l.depth, glow, 0.2);
                        }
                        self.put(xi, yi, l.depth, [1.0, 0.97, 0.9]);
                        self.put(xi - (len - 1) * dir as i64, yi, l.depth, RED);
                        for j in 1..4 {
                            self.add(xi + j * dir as i64, yi, l.depth, [1.0, 0.95, 0.8], 0.5 / j as f64);
                        }
                    }
                }
            }
        }
    }

    /// Now and then a patrol spinner crosses, strobing, its spotlight raking down.
    fn patrol(&mut self, t: f64) {
        let tp = (t + 4.3).rem_euclid(PATROL);
        if tp > PATROL_SEEN {
            return;
        }
        let x = 340.0 - tp * 29.0;
        let y = 54.0 + 2.0 * (tp * 1.1).sin();
        let depth = 1.8;
        // the spotlight: a cone down and forward, lighting what it falls on
        let (sa, ca) = (-0.25 + 0.15 * (tp * 0.7).sin(), 1.0f64);
        let norm = sa.hypot(ca);
        let (sa, ca) = (sa / norm, ca / norm);
        for py in (y as usize + 2)..H {
            for px in 0..W {
                let (dx, dy) = (px as f64 + 0.5 - x, py as f64 + 0.5 - y - 2.0);
                let along = dx * sa + dy * ca;
                if along <= 0.0 {
                    continue;
                }
                let perp = dx * ca - dy * sa;
                let wd = 1.0 + along * 0.22;
                if perp.abs() > wd * 1.6 {
                    continue;
                }
                let k = py * W + px;
                if self.depth[k] <= depth {
                    continue;
                }
                let i = (-(perp / wd).powi(4)).exp() * (-along / 80.0).exp() * if self.kind[k] == K_SKY { 0.2 } else { 0.5 };
                let o = &mut self.out[k];
                o[0] += i * 0.85;
                o[1] += i * 0.9;
                o[2] += i;
            }
        }
        let (xi, yi) = (x.floor() as i64, y.floor() as i64);
        for dx in -4..=4i64 {
            self.put(xi + dx, yi, depth, [0.08, 0.07, 0.12]);
            if dx.abs() < 3 {
                self.put(xi + dx, yi - 1, depth, if dx.abs() < 2 { [0.2, 0.3, 0.42] } else { [0.08, 0.07, 0.12] });
            }
        }
        self.put(xi - 4, yi - 1, depth, [0.08, 0.07, 0.12]);
        self.put(xi + 4, yi + 1, depth, [0.08, 0.07, 0.12]);
        self.put(xi - 4, yi + 1, depth, [0.08, 0.07, 0.12]);
        let flip = (t * 7.0).floor() as i64 % 2 == 0;
        let (a, b) = if flip { ([1.0, 0.15, 0.2], [0.1, 0.12, 0.3]) } else { ([0.3, 0.06, 0.08], [0.25, 0.45, 1.0]) };
        if xi > 2 && (xi as usize) < W - 3 && yi > 1 {
            self.glow((xi - 2) as usize, (yi - 2) as usize, depth, a, if flip { 2.0 } else { 0.0 });
            self.glow((xi + 2) as usize, (yi - 2) as usize, depth, b, if flip { 0.0 } else { 2.0 });
        }
        self.put(xi - 5, yi, depth, [1.0, 0.97, 0.9]);
    }
}
