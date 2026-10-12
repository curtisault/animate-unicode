//! tokyo rain: a narrow side street in tokyo at night, in the rain. Vertical
//! signs stand out from the walls on both sides in pink, cyan and amber, a
//! vending machine and an izakaya's red lanterns glow at street level, power
//! lines cross the strip of sky, and the wet road carries every light down in
//! smeared, rippling streaks. Someone under an umbrella walks slowly away.
//! After ascii.rest's tokyo-rain.ts (MIT, @bas3line).
//!
//! The street in one-point perspective, everything in it placed, sized,
//! fogged and mirrored in the wet road by its depth, the glow round every
//! light, the flickering tubes, the swinging lanterns, the walker, the
//! ripples and the rain, and how each is lit, are ascii.rest's, built and
//! shaded here per pixel instead of per cell. Where ascii.rest draws each of
//! 200 × 100 square cells as a dot as big as it is bright, this draws 320 ×
//! 80 cells of two square pixels each, an upper half block over a back
//! colour ([`crate::pixels`]): 1.6 times the detail each way, solid colour,
//! and gradients dithered in colour rather than in dot size. The signs are
//! laid out on the pixel grid rather than scaled from ascii.rest's cells, so
//! their letters keep even strokes (3 pixels where ascii.rest's are 2 cells,
//! 2 where they are 1, the narrowest signs a pixel wider to fit them), and
//! the chasing bulbs are 2 × 2 pixels. Wires, road lines, the lantern cords
//! and the rain are drawn a pixel thin, the tower's windows are a pixel each,
//! and the glow, gloss and smear lengths are scaled to the finer grid.
//! ascii.rest lifts its dim street with a tone curve for the dots to find it;
//! this keeps a gentler one (brightness to the 0.7), and gives the rain three
//! quarters of its light, since a solid streak shows more than a row of dots.
//! The palette adds the dark tones a solid pixel needs where a small dot was
//! enough. In one ink each pixel is octant dots, as many as it is bright.

use crate::noise::{clamp, fbm, fbm_wrap, hash, mix, noise, smooth};
use crate::pixels::Pixels;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// Tokyo rain's meta.
pub static META: Meta = Meta {
    name: "tokyo rain",
    slug: "tokyo-rain",
    category: Category::Cities,
    note: "a tokyo side street in the rain, neon signs shining on the wet road",
    cols: 320,
    rows: 80,
    fps: 15,
    charset: Charset::Extended,
    // ascii.rest's: the night's violets, the pinks, cyans, ambers and reds of
    // the signs, the vending machine's whites and cans, the street's greys.
    // Then the darks a solid pixel needs.
    palette: Some(&[
        "#100f22", "#17162f", "#201e3d", "#2b284e", "#383463", "#4a447a", "#4c4890", "#6a64c4", "#1d1520", "#2c1f2e", "#3e2b3e", "#553a52", "#6a3a74",
        "#8e4c8a", "#b4649c", "#d885ae", "#6e1248", "#a3196c", "#d8278c", "#ff47a8", "#ff85c6", "#ffc4e3", "#0c4656", "#106f80", "#16a0b2", "#28d2e2",
        "#7cecf4", "#ccfbfd", "#5e340a", "#93540f", "#cc7b16", "#f7a32a", "#ffc65e", "#ffe2a0", "#fff4d8", "#55100e", "#8a1a14", "#c22a1a", "#ee4626",
        "#ff7a4c", "#8ea2c4", "#bccbe6", "#e4ecfb", "#ffffff", "#2e62e0", "#79a2ff", "#20844c", "#43d884", "#e6c21e", "#fff07a", "#6a7896", "#9ba7c2",
        "#c9d0e2", "#262a36", "#3a3f4e", "#565c6c", "#07060d", "#0b0a16", "#120d18", "#1a1210", "#2e1a0c", "#081a20", "#140810", "#0f0f14",
    ]),
    ground: Some("#07060d"),
    ..Meta::DEFAULT
};

/// The picture in pixels: ascii.rest's 200 × 100 square cells at 1.6 pixels each.
const W: usize = 320;
const H: usize = 160;
const S: f64 = 1.6;
/// ascii.rest's frame height, in its rows.
const TSH: f64 = 100.0;
/// The vanishing point, where the street meets the horizon.
const VX: f64 = 100.0;
const HY: f64 = 50.0;
/// The horizon's pixel row.
const HYP: usize = 80;
/// How many cells a metre spans, a metre away.
const F: f64 = 46.0;
/// The camera's height above the road.
const EYE: f64 = 1.5;
/// From the middle of the street to the walls.
const HALF: f64 = 3.2;
/// How deep a sign box is.
const THICK: f64 = 0.3;
/// The rain haze hides most of what is further than this.
const ZF: f64 = 17.0;
/// Rows of mirror image kept below the frame, for the gloss to pull up:
/// ascii.rest's 70, in pixels.
const XH: usize = H + 112;
/// The low cloud's width in pixels: it wraps, so it can drift forever.
const CW: usize = 640;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mat {
    Sky,
    Wall,
    Road,
    Sign,
    Side,
    Vend,
    Pole,
    Wire,
    Tower,
}

impl Mat {
    /// How much of it the wet road gives back.
    fn give(self) -> f32 {
        match self {
            Mat::Sky => 0.25,
            Mat::Wall => 0.1,
            Mat::Road => 0.0,
            Mat::Sign => 1.0,
            Mat::Side => 0.5,
            Mat::Vend => 0.9,
            Mat::Pole => 0.12,
            Mat::Wire => 0.15,
            Mat::Tower => 0.2,
        }
    }
}

const PINK: [f64; 3] = [1.0, 0.2, 0.62];
const CYAN: [f64; 3] = [0.14, 0.88, 1.0];
const AMBER: [f64; 3] = [1.0, 0.6, 0.14];
const RED: [f64; 3] = [1.0, 0.16, 0.1];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Neon,
    Panel,
    White,
    Bulbs,
}

/// Blocky letters on a 5 by 5 grid: katakana and a few made-up characters.
fn glyph(name: &str) -> &'static [u8] {
    match name {
        "ra" => b".###. ..... ##### ....# ..##.",
        "-" => b"..#.. ..#.. ..#.. ..#.. ..#..",
        "me" => b"....# .#.#. ..#.. .#.#. #....",
        "n" => b"#.... .#..# ....# ...#. ###..",
        "ka" => b".#... ##### .#..# .#..# #..##",
        "o" => b"...#. ##### ..##. .#.#. #..#.",
        "ke" => b".#... .#### #..#. ...#. ..#..",
        "su" => b"####. ...#. ..#.. .#.#. #...#",
        "na" => b"..#.. ##### ..#.. ..#.. .#...",
        "tsu" => b"#.#.# .#..# ....# ...#. .##..",
        "ku" => b".#... .#### #...# ...#. .##..",
        "ho" => b"..#.. ##### ..#.. #.#.# ..#..",
        "te" => b"####. ..... ##### ..#.. .#...",
        "ru" => b".#.#. .#.#. .#.#. .#.## #..#.",
        "k1" => b"#.### ..#.# #.### ..#.# #.###",
        "k2" => b"##### #...# ##### #.#.. #.###",
        "k3" => b"#.### #.#.# ###.# #.#.# ###.#",
        "k4" => b"..#.. .#.#. ##### #.#.# #.###",
        _ => b"..#.. ##### .###. #.#.# ..#..",
    }
}

/// A sign standing out from a wall.
struct Sign {
    /// -1 the left wall, 1 the right.
    side: f64,
    /// Depth, in cells a metre.
    s: f64,
    /// The bottom's height above the road.
    hb: f64,
    /// Width in ascii.rest's cells.
    w: f64,
    /// A letter's stroke in ascii.rest's cells, or 0 for a plain strip.
    g: f64,
    kind: Kind,
    hue: [f64; 3],
    /// A strip's height in ascii.rest's cells.
    rows: f64,
    text: &'static str,
    /// Where its outer edge stands, if not at the wall.
    edge: f64,
    /// It flickers with this group.
    group: u8,
    /// This letter of it flickers alone.
    fail: Option<usize>,
}

/// A sign of letters, its width and stroke `wg`.
const fn sign(side: f64, s: f64, hb: f64, wg: (f64, f64), kind: Kind, hue: [f64; 3], text: &'static str) -> Sign {
    Sign { side, s, hb, w: wg.0, g: wg.1, kind, hue, rows: 0.0, text, edge: HALF, group: 0, fail: None }
}

/// A plain strip of light.
const fn strip(side: f64, s: f64, hb: f64, w: f64, kind: Kind, hue: [f64; 3], rows: f64) -> Sign {
    Sign { side, s, hb, w, g: 0.0, kind, hue, rows, text: "", edge: HALF, group: 0, fail: None }
}

const SIGNS: [Sign; 16] = [
    Sign { fail: Some(2), ..sign(-1.0, 27.0, 2.3, (16.0, 2.0), Kind::Neon, PINK, "ra - me n") },
    sign(-1.0, 17.5, 2.45, (14.0, 2.0), Kind::Panel, AMBER, "k1 k2 k3"),
    Sign { group: 1, ..sign(-1.0, 11.5, 2.55, (9.0, 1.0), Kind::Neon, CYAN, "k4 k5 k1") },
    sign(-1.0, 7.4, 2.6, (7.0, 1.0), Kind::White, RED, "k2 k3 k5"),
    strip(-1.0, 5.0, 2.7, 4.0, Kind::Panel, PINK, 18.0),
    strip(-1.0, 5.0, 6.6, 4.0, Kind::Neon, AMBER, 10.0),
    strip(-1.0, 3.4, 2.5, 3.0, Kind::Neon, CYAN, 14.0),
    strip(-1.0, 2.3, 2.8, 2.0, Kind::Panel, AMBER, 10.0),
    sign(1.0, 24.0, 2.55, (16.0, 2.0), Kind::Neon, CYAN, "ka ra o ke"),
    sign(1.0, 14.5, 2.5, (11.0, 1.0), Kind::Bulbs, PINK, "su na tsu ku"),
    sign(1.0, 6.6, 2.6, (7.0, 1.0), Kind::Panel, AMBER, "ho te ru"),
    Sign { group: 2, ..strip(1.0, 4.6, 2.7, 4.0, Kind::Neon, CYAN, 16.0) },
    strip(1.0, 4.6, 6.4, 4.0, Kind::Panel, PINK, 9.0),
    strip(1.0, 3.1, 2.6, 3.0, Kind::Panel, AMBER, 13.0),
    strip(1.0, 2.1, 2.9, 2.0, Kind::Neon, PINK, 10.0),
    // the andon: a lit box standing on the road outside the izakaya
    Sign { edge: 2.44, ..sign(1.0, 18.0, 0.14, (7.0, 1.0), Kind::White, RED, "k1 k4") },
];

/// The buildings along each side: from, to, height and front, in metres down
/// the street. Fronts: 0 a bare wall, 1 a shutter, 2 a lit shop, 3 a door, 4
/// the izakaya. A height of 0 is an alley between two buildings.
const LEFT: [(f64, f64, f64, u8); 9] = [
    (0.0, 2.55, 10.5, 1),
    (2.55, 4.6, 7.2, 3),
    (4.6, 7.0, 6.2, 2),
    (7.0, 10.5, 11.0, 3),
    (10.5, 15.0, 8.4, 2),
    (15.0, 22.0, 12.5, 0),
    (22.0, 32.0, 9.5, 2),
    (32.0, 60.0, 14.0, 0),
    (60.0, 1e5, 18.0, 0),
];
const RIGHT: [(f64, f64, f64, u8); 8] = [
    (0.0, 3.2, 9.6, 4),
    (3.2, 5.0, 8.2, 0),
    (5.0, 6.4, 0.0, 0),
    (6.4, 9.5, 7.4, 2),
    (9.5, 14.0, 13.5, 3),
    (14.0, 20.0, 8.0, 2),
    (20.0, 30.0, 11.0, 0),
    (30.0, 1e5, 16.0, 0),
];
/// Water tanks on the roofs: side, from, to, height.
const TANKS: [(f64, f64, f64, f64); 3] = [(-1.0, 5.3, 6.2, 1.7), (-1.0, 11.6, 12.7, 1.5), (1.0, 7.3, 8.1, 1.4)];
/// Masts on the roofs: side, at, height.
const MASTS: [(f64, f64, f64); 2] = [(-1.0, 13.8, 3.0), (1.0, 18.4, 2.4)];
/// Utility poles: across, along, height, thickness.
const POLES: [(f64, f64, f64, f64); 4] = [(2.9, 4.3, 9.4, 0.32), (-2.95, 9.6, 9.4, 0.3), (2.9, 15.0, 9.2, 0.28), (-2.95, 24.0, 9.2, 0.26)];
/// The wires between them: across, height, along, to across, height, along, and sag.
const WIRES: [[f64; 7]; 10] = [
    [2.9, 8.6, 4.3, -2.95, 8.9, 9.6, 0.45],
    [2.9, 8.2, 4.3, -2.95, 8.4, 9.6, 0.55],
    [2.9, 7.6, 4.3, -3.2, 5.9, 6.0, 0.3],
    [2.9, 9.0, 4.3, 2.9, 8.9, 15.0, 0.4],
    [2.9, 8.4, 4.3, 2.9, 8.5, 15.0, 0.5],
    [-2.95, 8.9, 9.6, -2.95, 8.9, 24.0, 0.35],
    [-2.95, 8.4, 9.6, 2.9, 8.5, 15.0, 0.4],
    [2.9, 8.5, 15.0, -2.95, 8.6, 24.0, 0.3],
    [2.9, 8.9, 15.0, 2.9, 8.8, 28.0, 0.3],
    [-2.95, 8.4, 24.0, 2.9, 8.4, 34.0, 0.25],
];
/// The izakaya's red lanterns: depth, across, centre height, swing phase.
const LANTERNS: [(f64, f64, f64, f64); 2] = [(26.3, 2.72, 1.88, 0.0), (14.8, 2.72, 1.95, 1.9)];
/// The vending machine's depth.
const VEND_S: f64 = 16.0;
/// The cans behind the vending machine's glass.
const CANS: [[f64; 3]; 7] = [[1.0, 0.16, 0.1], [0.15, 0.4, 1.0], [1.0, 0.55, 0.1], [0.2, 0.85, 0.45], [0.95, 0.95, 0.95], [1.0, 0.85, 0.15], [0.85, 0.2, 0.6]];
/// Rain in three depths: count, speed in rows a second, length, brightness, slant.
const LAYERS: [(usize, f64, f64, f64, f64); 3] = [(150, 40.0, 3.0, 0.18, 0.1), (190, 72.0, 6.0, 0.38, 0.13), (60, 125.0, 11.0, 0.7, 0.17)];
/// The glow of the busy street the alley runs into.
const END: [f64; 3] = [1.0, 0.7, 0.86];
/// The tone curve's power: brightness to the 0.7 (ascii.rest's dots take it
/// to the 0.64).
const LIFT: f32 = 0.7;
/// The flickering groups: a cyan sign, a cyan strip, and one failing letter.
const NG: usize = 3;

/// The far end's glow, seen through the rain, at (x, y) in ascii.rest's cells.
fn end_glow(x: f64, y: f64) -> f64 {
    let d = ((x - VX) * 0.9).hypot((y - HY + 2.0) * 1.25);
    (-d / 6.5).exp() * 0.95 + (-d / 16.0).exp() * 0.26 + (-d / 45.0).exp() * 0.08
}

/// The scene's y, in ascii.rest's rows, at the middle of pixel row `py`.
fn row_y(py: usize) -> f64 {
    (py as f64 + 0.5) / S
}

/// The scene's x at the middle of pixel column `px`, in ascii.rest's
/// columns: its `x + 0.5`.
fn mid_x(px: usize) -> f64 {
    (px as f64 + 0.5) / S
}

/// The pixel range covering ascii.rest's span `a..b`, clipped to `0..n`.
fn span(a: f64, b: f64, n: usize) -> std::ops::Range<usize> {
    let lo = (a * S).floor().max(0.0) as usize;
    let hi = ((b * S).ceil().max(0.0) as usize).min(n);
    lo..hi.max(lo)
}

/// The walker under the umbrella, in metres about their feet: 0 outside, 1
/// the figure, 2 the umbrella's lit crown, 3 the rest of the umbrella.
fn walker_shape(u: f64, v: f64, ph: f64) -> u8 {
    let v = v - 0.014 * ph.cos().abs();
    let (r, rim) = (0.56, 1.74);
    if v > rim - 0.07 {
        let q = u / r;
        if q > -1.0 && q < 1.0 {
            let top = rim + 0.4 * (1.0 - q * q).sqrt();
            let bottom = rim - 0.05 * (q * 2.5 * PI).cos().abs();
            if v <= top && v >= bottom {
                return if v > top - 0.06 { 2 } else { 3 };
            }
        }
        if u > -0.03 && u < 0.03 && v < rim + 0.5 {
            return 3;
        }
    }
    let (hy, hx) = ((v - 1.6) / 0.125, u / 0.1);
    if hx * hx + hy * hy < 1.0 {
        return 1;
    }
    if (0.5..=1.48).contains(&v) {
        let hw = if v > 1.36 { 0.2 * (1.0 - ((v - 1.36) / 0.12).powi(2)).sqrt() } else { 0.18 + 0.035 * smooth(0.85, 0.5, v) };
        if u > -hw && u < hw {
            return 1;
        }
    }
    // the arm up to the handle
    if v > 1.2 && v < rim && u > 0.01 && u < 0.07 {
        return 1;
    }
    for i in [-1.0, 1.0] {
        let lift = 0.09 * (ph + if i > 0.0 { PI } else { 0.0 }).sin().max(0.0);
        let lx = u - i * 0.075;
        if v >= lift && v < 0.62 && lx > -0.055 && lx < 0.055 {
            return 1;
        }
    }
    0
}

/// Where the walker is at t: walking out of the alley, then away down the
/// street. The walk is short, so the street is never empty for long, and it
/// keeps time with the traffic light.
struct Walk {
    xw: f64,
    z: f64,
    s: f64,
    fade: f64,
    ph: f64,
}

fn walker(t: f64) -> Walk {
    let tau = (t + 5.5).rem_euclid(24.0);
    // they turn from walking out of the alley to walking away, without a jolt
    let k = clamp((tau - 3.0) / 4.0);
    let ramp = if tau < 3.0 {
        0.0
    } else if tau < 7.0 {
        4.0 * (k * k * k - k * k * k * k / 2.0)
    } else {
        2.0 + tau - 7.0
    };
    let z = 5.6 + 0.06 * tau + 0.5 * ramp;
    Walk { xw: 4.4 - 4.05 * smooth(0.0, 5.5, tau), z, s: F / z, fade: smooth(15.0, 12.0, z), ph: tau * 1.7 * PI }
}

/// A flickering group's brightness at t: steady, then now and then a stutter.
fn flicker(gi: usize, t: f64) -> f64 {
    let f = (t * 15.0).floor();
    match gi {
        1 => {
            let c = (t + 3.0) % 9.0;
            if c < 0.9 {
                if hash(f, 11.0) > 0.45 {
                    1.0
                } else {
                    0.1
                }
            } else {
                0.96 + 0.04 * hash(f, 12.0)
            }
        }
        2 => {
            // this tube dies for a few seconds, then catches again
            let c = (t + 7.0) % 13.0;
            if c < 0.7 || (c > 4.0 && c < 4.8) {
                f64::from(u8::from(hash(f, 21.0) > 0.5))
            } else if c < 4.0 {
                0.0
            } else {
                1.0
            }
        }
        _ => {
            let c = (t + 1.5) % 7.0;
            if c < 1.2 {
                f64::from(u8::from(hash(f, 31.0) > 0.35))
            } else {
                1.0
            }
        }
    }
}

/// One lantern at a swing, each of its pixels and colour handed to `put`.
fn lantern(li: usize, swing: f64, mut put: impl FnMut(usize, f64, f64, f64)) {
    let (s, lx, lh, _) = LANTERNS[li];
    let (xc, yc) = (VX + lx * s, HY - (lh - EYE) * s);
    let top = HY - (lh + 0.62 - EYE) * s;
    for py in span(top, yc + 0.48 * s, H) {
        let y = row_y(py);
        let v = (yc - y) / s; // height about the centre
        let off = swing * (0.62 - v); // swinging from the hook above
        for px in span(xc - 0.36 * s - 1.0, xc + 0.36 * s + 1.0, W) {
            let u = (mid_x(px) - xc) / s - off;
            let k = py * W + px;
            if v > 0.44 {
                // the cord up to the eave, a pixel thin
                if u.abs() < 0.5 / (s * S) {
                    put(k, 0.04, 0.02, 0.02);
                }
                continue;
            }
            if v < -0.44 {
                continue;
            }
            if v.abs() > 0.38 {
                if u.abs() < 0.15 {
                    put(k, 0.06, 0.02, 0.02); // the black caps
                }
                continue;
            }
            let hw = 0.26 * (1.0 - (v / 0.46).powi(2)).sqrt();
            if u.abs() > hw {
                continue;
            }
            let q = u / hw;
            let mut b = 0.6 + 0.4 * (1.0 - q * q);
            if (v + 0.38) % 0.095 < 0.024 {
                b *= 0.5; // the ribs
            }
            // a dark character down the middle
            let letter = v.abs() < 0.24 && (u.abs() < 0.02 || (u.abs() < 0.08 && (v + 0.24) % 0.12 < 0.03));
            if letter {
                b *= 0.12;
            }
            put(k, b, b * 0.27, b * 0.11);
        }
    }
}

/// A box blur of `rad` pixels each way, `passes` times over.
fn blur(src: &[f32], rad: usize, passes: usize) -> Vec<f32> {
    let mut a = src.to_vec();
    let mut b = vec![0.0f32; W * H];
    let mut pre = vec![0.0f32; W.max(H) + 1];
    let n = (2 * rad + 1) as f32;
    for _ in 0..passes {
        for r in 0..H {
            for x in 0..W {
                pre[x + 1] = pre[x] + a[r * W + x];
            }
            for x in 0..W {
                b[r * W + x] = (pre[(x + rad + 1).min(W)] - pre[x.saturating_sub(rad)]) / n;
            }
        }
        for x in 0..W {
            for r in 0..H {
                pre[r + 1] = pre[r] + b[r * W + x];
            }
            for r in 0..H {
                a[r * W + x] = (pre[(r + rad + 1).min(H)] - pre[r.saturating_sub(rad)]) / n;
            }
        }
    }
    a
}

/// The glow round a channel of light: three blurs, near to far, ascii.rest's
/// radii of 2, 6 and 20 cells in pixels.
fn glow(e: &[f32]) -> Vec<f32> {
    let (a, b, c) = (blur(e, 3, 2), blur(e, 10, 3), blur(e, 32, 2));
    (0..W * H).map(|k| a[k] * 0.5 + b[k] + c[k] * 0.22).collect()
}

/// Three channels.
type Rgb3 = [Vec<f32>; 3];

/// The street as built: what each pixel is, how near, the light it gives off
/// and takes back.
struct Street {
    mat: Vec<Mat>,
    /// How near each pixel's surface is, in cells a metre; 0 for the sky.
    near: Vec<f32>,
    /// Light given off.
    e: Rgb3,
    /// How much light a surface takes back.
    a: Rgb3,
    /// How much of the glow round the lights reaches it.
    kg: Vec<f32>,
    grp: Vec<u8>,
    sky: Rgb3,
    wet: Vec<f32>,
}

impl Street {
    fn emit(&mut self, k: usize, r: f64, g: f64, b: f64) {
        for (c, v) in [r, g, b].into_iter().enumerate() {
            self.e[c][k] = v as f32;
            self.a[c][k] = 0.0;
        }
    }

    fn surface(&mut self, k: usize, r: f64, g: f64, b: f64, kg: f64) {
        for (c, v) in [r, g, b].into_iter().enumerate() {
            self.e[c][k] = 0.0;
            self.a[c][k] = v as f32;
        }
        self.kg[k] = kg as f32;
    }
}

/// The street at night: the still picture and its glow, the road's mirror,
/// what each flickering group adds to both, and the picture.
pub struct TokyoRain {
    mat: Vec<Mat>,
    near: Vec<f32>,
    wet: Vec<f32>,
    fog: Vec<f32>,
    /// The lit haze.
    hz: Rgb3,
    /// The picture before anything moves, and the glow there.
    c: Rgb3,
    g: Rgb3,
    /// The road's mirror.
    refl: Rgb3,
    /// Each flickering group's share of the picture (pixels, then three values
    /// each) and of the mirror.
    drop_c: Vec<(Vec<u32>, Vec<f32>)>,
    drop_r: Vec<Rgb3>,
    road: Vec<u32>,
    sky: Vec<u32>,
    cloud: Vec<f32>,
    /// The rain: x, y, speed, length, brightness, slant, in ascii.rest's cells.
    drops: Vec<[f64; 6]>,
    /// The chasing bulbs: pixel and place in the ring.
    bulbs: Vec<(u32, u32)>,
    /// This frame's picture, rain and the walker's hold on each pixel.
    f: Rgb3,
    rain: Vec<f32>,
    hold: Vec<f32>,
    pixels: Pixels,
}

/// A rainy tokyo street; it takes no options.
#[allow(clippy::needless_range_loop)] // the index reads several arrays at once
pub fn make(_: &Options) -> Box<dyn Piece> {
    let n = W * H;
    let z3 = || [vec![0.0f32; n], vec![0.0f32; n], vec![0.0f32; n]];
    let mut st = Street { mat: vec![Mat::Sky; n], near: vec![0.0; n], e: z3(), a: z3(), kg: vec![0.0; n], grp: vec![0; n], sky: z3(), wet: vec![0.0; n] };

    // --- the walls, the road and the sky, column by column -----------------
    let (mut wall_s, mut roof_y, mut bare_y, mut base_y) = (vec![0.0; W], vec![0.0; W], vec![0.0; W], vec![0.0; W]);
    let (mut col_i, mut col_h, mut tank_col) = (vec![0usize; W], vec![0.0; W], vec![false; W]);
    for px in 0..W {
        let xc = mid_x(px);
        let side = if xc < VX { -1.0 } else { 1.0 };
        let list: &[(f64, f64, f64, u8)] = if side < 0.0 { &LEFT } else { &RIGHT };
        let s0 = (xc - VX).abs() / HALF;
        let z = F / s0;
        let mut i = 0;
        while i < list.len() - 1 && z >= list[i].1 {
            i += 1;
        }
        let (mut s, mut hgt) = (s0, list[i].2);
        if hgt == 0.0 {
            // an alley: through it, the side wall of the next building along
            s = F / list[i].1;
            hgt = list[i + 1].2;
        }
        let mut extra = 0.0;
        for (ts, a, b, e) in TANKS {
            if ts == side && z >= a && z < b {
                extra = e;
                tank_col[px] = true;
            }
        }
        wall_s[px] = s;
        base_y[px] = HY + EYE * s;
        bare_y[px] = HY - (hgt - EYE) * s;
        roof_y[px] = HY - (hgt + extra - EYE) * s;
        col_i[px] = i;
        col_h[px] = hgt;
    }
    for (ms, at, e) in MASTS {
        let s = F / at;
        let x = (VX + ms * HALF * s).floor();
        for px in span(x, x + 1.0, W) {
            if mid_x(px).floor() == x {
                roof_y[px] = f64::min(roof_y[px], bare_y[px] - e * s);
            }
        }
    }
    // the caged bulb on the near left wall, over the shutter
    let (lbx, lby) = (VX - HALF * (F / 1.62), HY - (2.2 - EYE) * (F / 1.62));

    for py in 0..H {
        let y = row_y(py);
        for px in 0..W {
            let k = py * W + px;
            let xc = mid_x(px);
            let side = if xc < VX { -1.0 } else { 1.0 };
            if y >= base_y[px] {
                // the road: dark wet asphalt, a white line along each edge, a gutter of grates
                st.mat[k] = Mat::Road;
                let s = (y - HY) / EYE;
                let z = F / s;
                let xw = (xc - VX) / s;
                let ax = xw.abs();
                st.near[k] = s as f32;
                let mut a = 0.11 + 0.07 * fbm(xw * 2.2, z * 1.1, 2);
                // the paint's cover of this pixel
                let (lo, hi) = (ax - 0.5 / (s * S), ax + 0.5 / (s * S));
                let paint = clamp((hi.min(2.66) - lo.max(2.52)) * s * S);
                a = mix(a, 0.75, paint);
                if ax > 2.85 {
                    a = if z % 0.55 < 0.08 { 0.07 } else { 0.2 };
                }
                st.surface(k, a * 0.82, a * 0.86, a, 0.6);
                // the wet paint shines a little of its own, so the lines run on into the dark
                st.e[0][k] = (paint * 0.13) as f32;
                st.e[1][k] = (paint * 0.14) as f32;
                st.e[2][k] = (paint * 0.17) as f32;
                // puddles hold the reflections sharper
                st.wet[k] = (0.5 + 0.5 * smooth(0.42, 0.6, fbm(xw * 0.9 + 3.0, z * 0.35, 3))) as f32;
                continue;
            }
            let tank_leg = tank_col[px] && y < bare_y[px] && y > roof_y[px] && (HY - y) / wall_s[px] + EYE - col_h[px] < 0.45 && (xc.floor() as i64) & 1 == 0;
            if y < roof_y[px] || tank_leg {
                continue; // the sky
            }
            // a wall: its depth and height give where on the building this is
            st.mat[k] = Mat::Wall;
            let s = wall_s[px];
            st.near[k] = s as f32;
            let list: &[(f64, f64, f64, u8)] = if side < 0.0 { &LEFT } else { &RIGHT };
            let i = col_i[px];
            let (z0, z1, hgt, front) = list[i];
            let h = EYE + (HY - y) / s;
            let tone = 0.75 + 0.5 * hash(i as f64, side + 7.0);
            let tex = 0.8 + 0.35 * fbm(xc * 0.6, y * 0.6, 2);
            let (wr, wg, wb) = (0.3 * tone * tex, 0.28 * tone * tex, 0.33 * tone * tex);
            if hgt == 0.0 {
                // the far wall of the alley, with a bare bulb by a door
                let xw = (xc - VX) / s;
                let dl = (xw - 3.65).hypot((h - 2.9) * 0.8);
                if dl < 0.1 {
                    st.emit(k, 1.2, 1.0, 0.7);
                } else if h > 2.95 && h < 3.05 && (xw - 3.65).abs() < 0.16 {
                    st.surface(k, 0.05, 0.05, 0.06, 0.4);
                } else if (xw - 3.36).abs() < 0.05 {
                    st.surface(k, 0.12, 0.12, 0.14, 0.6);
                } else if xw > 3.85 && h < 2.1 {
                    st.surface(k, 0.06, 0.05, 0.05, 0.5);
                } else {
                    // the bulb's own pool of light on the wall, for someone to pass in front of
                    let pool = (-dl / 0.9).exp() * 0.55;
                    st.surface(k, wr * 0.8, wg * 0.8, wb * 0.8, 1.0);
                    st.e[0][k] = (pool * 0.9) as f32;
                    st.e[1][k] = (pool * 0.62) as f32;
                    st.e[2][k] = (pool * 0.36) as f32;
                }
                continue;
            }
            let z = F / s;
            if side < 0.0 && i == 0 {
                // a caged bulb on the near wall, over the shutter
                let dl = ((xc - lbx) / 1.4).hypot((y - lby) / 1.1);
                if dl < 1.0 {
                    st.emit(k, 1.3, 0.95, 0.6);
                    continue;
                }
                if dl < 1.9 && y < lby - 0.6 {
                    st.surface(k, 0.06, 0.06, 0.07, 0.5);
                    continue;
                }
            }
            if h > 2.6 {
                // upper floors: rows of windows, a few lit, a slab line between floors
                let fl = ((h - 2.9) / 2.8).floor();
                let fh = h - 2.9 - fl * 2.8;
                let bay = ((z - z0 - 0.5) / 1.9).floor();
                let bz = z - z0 - 0.5 - bay * 1.9;
                if h > 2.9 && fh > 0.55 && fh < 1.85 && bz > 0.3 && bz < 1.3 && z < z1 - 0.5 && h < col_h[px] - 0.5 {
                    let hh = hash(bay * 7.0 + fl, side * 31.0 + i as f64);
                    if hh < 0.36 {
                        let v = (0.3 + 0.3 * hash(bay, fl + 9.0)) * if fh % 0.17 < 0.045 { 0.55 } else { 1.0 };
                        if hh < 0.12 {
                            st.emit(k, 0.5 * v, 0.72 * v, v);
                        } else {
                            st.emit(k, v, 0.7 * v, 0.4 * v);
                        }
                    } else {
                        st.surface(k, 0.08, 0.09, 0.13, 0.9);
                    }
                } else if h > 2.9 && fh < 0.14 {
                    st.surface(k, wr * 1.25, wg * 1.25, wb * 1.25, 1.0);
                } else {
                    st.surface(k, wr, wg, wb, 1.0);
                }
                // the lit sign band above the shop front on the low building
                if side < 0.0 && i == 2 && z > 4.85 && z < 6.75 && h > 2.45 && h < 2.85 {
                    let m = (z - 4.85) % 0.42;
                    let v = if m > 0.1 && m < 0.3 && h > 2.53 && h < 2.77 { 0.12 } else { 0.9 };
                    st.emit(k, AMBER[0] * v, AMBER[1] * v, AMBER[2] * v);
                }
                continue;
            }
            // the shop fronts at street level
            st.surface(k, wr, wg, wb, 1.0);
            match front {
                1 => {
                    // the shutter, its ribs picked out by the caged bulb above it
                    let pool = (-((xc - lbx) * 0.7).hypot(y - lby) / 11.0).exp() * 0.42;
                    if h < 2.15 {
                        let rib = if h % 0.11 < 0.035 { 0.3 } else { 1.0 };
                        st.surface(k, 0.27 * rib, 0.29 * rib, 0.35 * rib, 1.0);
                        st.e[0][k] = (pool * rib) as f32;
                        st.e[1][k] = (pool * 0.72 * rib) as f32;
                        st.e[2][k] = (pool * 0.45 * rib) as f32;
                    } else {
                        st.surface(k, 0.16, 0.16, 0.2, 1.0);
                    }
                }
                2 => {
                    let zz = z - z0;
                    if zz > 0.3 && zz < z1 - z0 - 0.3 && h > 0.1 && h < 2.15 {
                        let cool = i == 4 && side < 0.0;
                        let shelf = (h - 0.75).abs() < 0.05 || (h - 1.3).abs() < 0.05 || (h - 1.82).abs() < 0.05;
                        let v = (if zz % 1.4 < 0.07 {
                            0.05
                        } else if shelf {
                            0.3
                        } else {
                            0.62
                        }) * (0.85 + 0.3 * hash((zz * 3.0).floor(), 5.0));
                        if cool {
                            st.emit(k, 0.7 * v, 0.85 * v, v);
                        } else {
                            st.emit(k, v, 0.72 * v, 0.44 * v);
                        }
                    }
                }
                3 => {
                    let zz = z - z0;
                    if zz > 0.35 && zz < 1.15 {
                        if h < 2.0 {
                            st.surface(k, 0.07, 0.05, 0.04, 0.8);
                        } else if h < 2.22 {
                            st.emit(k, 0.55, 0.38, 0.2);
                        }
                    } else if zz > 1.5 && zz < 2.5 && h > 1.0 && h < 1.9 {
                        if zz % 0.12 < 0.03 || (h - 1.0) % 0.18 < 0.035 {
                            st.surface(k, 0.06, 0.04, 0.03, 0.6);
                        } else {
                            st.emit(k, 0.5, 0.36, 0.2);
                        }
                    }
                }
                4 => {
                    st.surface(k, 0.3, 0.19, 0.12, 1.0); // dark wood
                    if z > 2.35 && z < 3.0 && h < 2.05 {
                        if h > 1.42 {
                            // the noren over the door: indigo cloth in panels, a white crest
                            let slit = (z - 2.35) % 0.217 < 0.028;
                            let e = ((z - 2.675) / 0.09).powi(2) + ((h - 1.76) / 0.13).powi(2);
                            if slit {
                                st.emit(k, 0.8, 0.5, 0.22);
                            } else if e < 1.0 && e > 0.35 {
                                st.emit(k, 0.75, 0.74, 0.72);
                            } else {
                                st.emit(k, 0.09, 0.1, 0.3);
                            }
                        } else {
                            // the room inside, warm, a counter across it, two people sat at it
                            let mut v = (if (h - 0.95).abs() < 0.06 { 0.35 } else { 0.5 + 0.35 * smooth(0.2, 1.4, h) }) * if h < 0.3 { 0.6 } else { 1.0 };
                            for pz in [2.52, 2.83] {
                                let head = ((z - pz) / 0.075).powi(2) + ((h - 1.3) / 0.1).powi(2) < 1.0;
                                let back = h > 0.9 && h < 1.22 && (z - pz).abs() < 0.1 + 0.05 * smooth(1.2, 0.95, h);
                                if head || back {
                                    v = 0.07;
                                }
                            }
                            st.emit(k, v, 0.6 * v, 0.28 * v);
                        }
                    } else if z > 1.5 && z < 1.85 && h > 1.05 && h < 1.8 {
                        if z % 0.11 < 0.03 || (h - 1.05) % 0.19 < 0.04 {
                            st.surface(k, 0.12, 0.07, 0.04, 0.6);
                        } else {
                            st.emit(k, 0.6, 0.38, 0.18);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // the sky: the city's glow on low rain cloud, brightest over the street's far end
    for py in 0..H {
        let y = row_y(py);
        let v = clamp(y / HY).powf(1.6);
        for px in 0..W {
            let k = py * W + px;
            let g = end_glow(mid_x(px), y);
            st.sky[0][k] = (0.11 + 0.16 * v + g * END[0]) as f32;
            st.sky[1][k] = (0.08 + 0.08 * v + g * END[1]) as f32;
            st.sky[2][k] = (0.24 + 0.16 * v + g * END[2]) as f32;
        }
    }

    // A tower far off beyond the street, its top lit. It is darker than the
    // cloud behind it, so it stands as a silhouette rather than melting into
    // the glow. Its lit windows are a pixel each.
    let tower_top = 15.0;
    for py in span(8.0, 38.0, H) {
        let r = row_y(py).floor();
        for px in span(96.0, 104.0, W) {
            let x = mid_x(px).floor();
            let k = py * W + px;
            if !(8.0..38.0).contains(&r) || !(96.0..=103.0).contains(&x) || st.mat[k] != Mat::Sky {
                continue;
            }
            let spire = x == 99.0 || x == 100.0;
            if r < tower_top && !(spire && r >= 9.0) {
                continue;
            }
            st.mat[k] = Mat::Tower;
            st.near[k] = 0.05;
            if r < tower_top {
                st.emit(k, 0.015, 0.012, 0.025);
            } else if r < tower_top + 2.0 {
                st.emit(k, 0.42, 0.36, 0.5);
            } else {
                let lit = px % 2 == 0 && py % 2 == 1 && hash(px as f64, py as f64 + 5.0) < 0.45;
                if lit {
                    st.emit(k, 0.32, 0.24, 0.18);
                } else {
                    st.emit(k, 0.015, 0.012, 0.025);
                }
            }
        }
    }

    // --- poles and wires ------------------------------------------------------
    for (p, &(pxw, pz, top, thick)) in POLES.iter().enumerate() {
        let s = F / pz;
        let (xc, hw) = (VX + pxw * s, thick / 2.0 * s);
        let (yt, yb) = (HY - (top - EYE) * s, HY + EYE * s);
        let arm = top - 0.45;
        for py in span(yt - 3.0, yb, H) {
            let y = row_y(py);
            let h = EYE + (HY - y) / s;
            for px in span(xc - 0.9 * s, xc + 0.9 * s, W) {
                let k = py * W + px;
                if f64::from(st.near[k]) >= s {
                    continue;
                }
                let x = mid_x(px);
                // how much of the pixel the pole covers, so the far ones stay thin
                let mut cov = 0.0;
                if (0.0..=top).contains(&h) {
                    cov = clamp(f64::min(px as f64 + 1.0, (xc + hw) * S) - f64::max(px as f64, (xc - hw) * S));
                }
                let on_arm = (h - arm).abs() < f64::max(0.06, 0.5 / (s * S)) && (x - xc).abs() < 0.65 * s;
                let can = p == 1 && h > 6.4 && h < 7.5 && x - xc > 0.0 && x - xc < 0.45 * s;
                if on_arm || can {
                    cov = cov.max(if can { 1.0 } else { clamp(0.12 * s * S) });
                }
                if cov < 0.05 {
                    continue;
                }
                let (mut cr, mut cg, mut cb) = (0.36, 0.36, 0.4);
                if p == 0 {
                    // the near pole: concrete, a yellow and black guard, a blue address plate
                    let u = (x - xc) / hw;
                    let round = 0.55 + 0.45 * clamp(1.0 - u * u).sqrt();
                    (cr, cg, cb) = (cr * round, cg * round, cb * round);
                    if h > 0.25 && h < 1.8 {
                        (cr, cg, cb) = if (h * 4.5 + u * 0.35).rem_euclid(1.0) < 0.5 { (0.9, 0.72, 0.08) } else { (0.05, 0.05, 0.05) };
                    } else if h > 1.95 && h < 2.35 && u.abs() < 0.85 {
                        let dot = (x.floor() + y.floor()) % 2.0 == 0.0 && h > 2.03 && h < 2.27;
                        (cr, cg, cb) = if dot { (0.85, 0.88, 0.95) } else { (0.12, 0.25, 0.75) };
                    }
                } else if can {
                    (cr, cg, cb) = (0.4, 0.42, 0.46);
                } else {
                    (cr, cg, cb) = (0.12, 0.12, 0.15);
                }
                for (c, v) in [cr, cg, cb].into_iter().enumerate() {
                    st.a[c][k] = mix(f64::from(st.a[c][k]), v, cov) as f32;
                    st.e[c][k] *= (1.0 - cov) as f32;
                }
                st.kg[k] = mix(f64::from(st.kg[k]), 1.0, cov) as f32;
                if cov > 0.5 {
                    st.mat[k] = Mat::Pole;
                    st.near[k] = s as f32;
                }
            }
        }
    }
    for [ax, ah, az, bx, bh, bz, sag] in WIRES {
        // in pixels, so the wires are a pixel thin
        let (mut qx, mut qy, mut qs) = (0.0f64, 0.0f64, 0.0f64);
        for i in 0..=160 {
            let q = f64::from(i) / 160.0;
            let (wx, wz, wh) = (mix(ax, bx, q), mix(az, bz, q), mix(ah, bh, q) - sag * 4.0 * q * (1.0 - q));
            let s = F / wz;
            let (sx, sy) = ((VX + wx * s) * S, (HY - (wh - EYE) * s) * S);
            if i > 0 {
                // a thin dark line from the last point to this one
                let minx = (qx.min(sx) - 1.0).floor().max(0.0) as usize;
                let maxx = ((qx.max(sx) + 1.0).ceil().max(0.0) as usize).min(W - 1);
                let miny = (qy.min(sy) - 1.0).floor().max(0.0) as usize;
                let maxy = ((qy.max(sy) + 1.0).ceil().max(0.0) as usize).min(H - 1);
                let (dx, dy) = (sx - qx, sy - qy);
                let len2 = (dx * dx + dy * dy).max(1e-6);
                for py in miny..=maxy {
                    for px in minx..=maxx {
                        let k = py * W + px;
                        let (fx, fy) = (px as f64 + 0.5, py as f64 + 0.5);
                        let kk = clamp(((fx - qx) * dx + (fy - qy) * dy) / len2);
                        let (ex, ey) = (qx + dx * kk - fx, qy + dy * kk - fy);
                        let cov = clamp(1.15 - ex.hypot(ey) * 1.5) * 0.9;
                        let s2 = mix(qs, s, kk);
                        if cov < 0.05 || f64::from(st.near[k]) >= s2 {
                            continue;
                        }
                        for (c, v) in [0.05, 0.05, 0.07].into_iter().enumerate() {
                            st.a[c][k] = mix(f64::from(st.a[c][k]), v, cov) as f32;
                            st.e[c][k] *= (1.0 - cov) as f32;
                        }
                        st.kg[k] = mix(f64::from(st.kg[k]), 0.8, cov) as f32;
                        let open = matches!(st.mat[k], Mat::Sky | Mat::Tower);
                        if open {
                            for c in 0..3 {
                                st.sky[c][k] *= (1.0 - cov) as f32;
                            }
                        }
                        if cov > 0.45 && st.mat[k] != Mat::Road {
                            if open {
                                st.mat[k] = Mat::Wire;
                            }
                            st.near[k] = st.near[k].max(0.01);
                        }
                    }
                }
            }
            (qx, qy, qs) = (sx, sy, s);
        }
    }

    // --- the vending machine: a lit front, and its side toward the street ----
    {
        let s = VEND_S;
        let s2 = s / (1.0 + 0.7 * s / F);
        for py in 0..H {
            let y = row_y(py);
            let h0 = EYE + (HY - y) / s;
            for px in span(40.0, 80.0, W) {
                let k = py * W + px;
                let u = (mid_x(px) - VX) / s + 3.15;
                if !((0.0..=0.95).contains(&u) && (0.0..=1.83).contains(&h0) && f64::from(st.near[k]) < s) {
                    continue;
                }
                st.mat[k] = Mat::Vend;
                st.near[k] = s as f32;
                st.grp[k] = 0;
                let (mut cr, mut cg, mut cb) = (0.62, 0.66, 0.74); // the white casing, lit by its own light
                if u > 0.05 && u < 0.9 && h0 > 0.05 && h0 < 1.78 {
                    if h0 > 1.5 {
                        (cr, cg, cb) = if h0 > 1.6 && h0 < 1.71 { (1.0, 0.18, 0.12) } else { (0.85, 0.93, 1.0) };
                    } else if h0 > 0.53 {
                        let band = if h0 > 1.13 {
                            0
                        } else if h0 > 0.83 {
                            1
                        } else {
                            2
                        };
                        let hb = h0 - [1.13, 0.83, 0.53][band];
                        let slot = ((u - 0.08) / 0.135).floor();
                        let su = u - 0.08 - slot * 0.135;
                        (cr, cg, cb) = (0.82, 0.9, 1.0);
                        if hb < 0.045 {
                            // the buttons under each row
                            let lit = su > 0.05 && su < 0.09 && (0.0..6.0).contains(&slot);
                            (cr, cg, cb) = if lit { (0.35, 0.75, 1.0) } else { (0.2, 0.22, 0.28) };
                        } else if hb > 0.07 && hb < 0.26 && (0.0..6.0).contains(&slot) && su > 0.02 && su < 0.1 {
                            let c = CANS[(hash(slot, band as f64 + 3.0) * CANS.len() as f64) as usize];
                            (cr, cg, cb) = (c[0] * 0.95, c[1] * 0.95, c[2] * 0.95);
                        }
                    } else {
                        (cr, cg, cb) = (0.3, 0.32, 0.38);
                        if u > 0.12 && u < 0.62 && h0 > 0.1 && h0 < 0.26 {
                            (cr, cg, cb) = (0.02, 0.02, 0.03);
                        }
                        if u > 0.7 && u < 0.86 && h0 > 0.3 && h0 < 0.46 {
                            let read = h0 > 0.38 && h0 < 0.44;
                            (cr, cg, cb) = if read { (0.2, 1.0, 0.5) } else { (0.06, 0.06, 0.08) };
                        }
                    }
                }
                st.emit(k, cr, cg, cb);
            }
        }
        // the side, from the front edge back toward the far end
        for px in span(VX - 2.2 * s, VX - 2.2 * s2, W) {
            let sc = (VX - mid_x(px)) / 2.2;
            if sc > s || sc < s2 {
                continue;
            }
            let along = (F / sc - F / s) / 0.7;
            for py in 0..H {
                let h = EYE + (HY - row_y(py)) / sc;
                let k = py * W + px;
                if !(0.0..=1.83).contains(&h) || f64::from(st.near[k]) >= sc {
                    continue;
                }
                st.mat[k] = Mat::Vend;
                st.near[k] = sc as f32;
                let seam = (along - 0.5).abs() < 0.04;
                let v = (if h > 1.72 { 0.5 } else { 0.36 }) * if seam { 0.6 } else { 1.0 } * (1.0 - 0.3 * along);
                st.emit(k, v * 0.92, v * 0.96, v * 1.05);
            }
        }
    }

    // --- the signs, far ones first, laid out on the pixel grid -----------------
    let mut bulbs = vec![];
    let mut order: Vec<usize> = (0..SIGNS.len()).collect();
    order.sort_by(|&a, &b| SIGNS[a].s.total_cmp(&SIGNS[b].s));
    for si in order {
        let sg = &SIGNS[si];
        let s = sg.s;
        let text: Vec<&str> = sg.text.split(' ').filter(|w| !w.is_empty()).collect();
        let n = text.len() as i64;
        // stroke, edge, width, margin and gap in pixels
        let gp = (sg.g * S).round() as i64;
        let ep = if sg.g >= 2.0 || sg.kind == Kind::Bulbs { 2 } else { 1 };
        let wp = ((sg.w * S).round() as i64).max(if gp > 0 { 5 * gp + 2 * ep } else { 0 });
        let xo = ((VX + sg.side * sg.edge * s) * S).round() as i64;
        let (x0, x1) = if sg.side < 0.0 { (xo, xo + wp) } else { (xo - wp, xo) };
        let y1 = ((HY - (sg.hb - EYE) * s) * S).round() as i64;
        let m = (if sg.kind == Kind::Bulbs { 3.0 } else { sg.g + 1.0 } * S).round() as i64;
        let gap = ((sg.g + 1.0) * S).round() as i64;
        let step = 5 * gp + gap;
        let hc = if gp > 0 { 2 * m + n * step - gap } else { (sg.rows * S).round() as i64 };
        let y0 = y1 - hc;
        let gx = x0 + (wp - 5 * gp) / 2;
        let hue = sg.hue;
        let pale = [mix(hue[0], 1.0, 0.32), mix(hue[1], 1.0, 0.32), mix(hue[2], 1.0, 0.32)];
        // the bulbs are 2 × 2 blocks round the edge, every other one, counted in reading order
        let mut ring = std::collections::HashMap::new();
        for py in y0.max(0)..y1.min(H as i64) {
            for px in x0.max(0)..x1.min(W as i64) {
                let k = py as usize * W + px as usize;
                if f64::from(st.near[k]) >= s {
                    continue;
                }
                let (i, j, jb) = (px - x0, py - y0, y1 - 1 - py);
                let side_edge = i < ep || i >= wp - ep;
                let edge = side_edge || j < ep || jb < ep;
                let (mut stroke, mut gi) = (false, -1);
                if gp > 0 {
                    gi = (j - m).div_euclid(step);
                    let (gr, gc) = (j - m - gi * step, px - gx);
                    if (0..n).contains(&gi) && (0..5 * gp).contains(&gr) && (0..5 * gp).contains(&gc) {
                        stroke = glyph(text[gi as usize])[((gr / gp) * 6 + gc / gp) as usize] == b'#';
                    }
                } else if sg.w > 2.0 && !side_edge {
                    stroke = (1..=3).contains(&(j % 6));
                }
                let (cr, cg, cb) = match sg.kind {
                    Kind::Neon if edge => (hue[0], hue[1], hue[2]),
                    Kind::Neon if stroke => (pale[0], pale[1], pale[2]),
                    Kind::Neon => (0.03 + hue[0] * 0.05, 0.025 + hue[1] * 0.05, 0.045 + hue[2] * 0.05),
                    Kind::Panel if stroke => (hue[0] * 0.07, hue[1] * 0.07, hue[2] * 0.07),
                    Kind::Panel if edge && sg.w > 3.0 => (hue[0] * 0.55, hue[1] * 0.55, hue[2] * 0.55),
                    Kind::Panel => (pale[0] * 0.92, pale[1] * 0.92, pale[2] * 0.92),
                    Kind::White if stroke => (hue[0] * 0.9, hue[1] * 0.9, hue[2] * 0.9),
                    Kind::White if edge => (hue[0] * 0.7, hue[1] * 0.7, hue[2] * 0.7),
                    Kind::White => (0.95, 0.93, 0.88),
                    Kind::Bulbs if edge => {
                        let (bi, bj) = (i / 2, j / 2);
                        if (bi + bj) % 2 == 0 {
                            let next = ring.len() as u32;
                            bulbs.push((k as u32, *ring.entry((bi, bj)).or_insert(next)));
                        }
                        (0.04, 0.03, 0.03)
                    }
                    Kind::Bulbs if stroke => (1.0, 0.95, 0.86),
                    Kind::Bulbs => (hue[0] * 0.42, hue[1] * 0.42, hue[2] * 0.42),
                };
                st.mat[k] = Mat::Sign;
                st.near[k] = s as f32;
                st.emit(k, cr, cg, cb);
                st.grp[k] = if sg.group > 0 {
                    sg.group
                } else if sg.fail.is_some_and(|f| f as i64 == gi) && stroke {
                    3
                } else {
                    0
                };
            }
        }
        // the box's side, running back toward the far end of the street
        let xin = (if sg.side < 0.0 { x1 } else { x0 }) as f64 / S;
        let xw_in = (xin - VX) / s;
        let s2 = s / (1.0 + THICK * s / F);
        let xend = VX + xw_in * s2;
        let (hb, ht) = (EYE + (HY - y1 as f64 / S) / s, EYE + (HY - y0 as f64 / S) / s);
        for px in span(xin.min(xend), xin.max(xend), W) {
            let sc = (mid_x(px) - VX) / xw_in;
            if sc > s || sc < s2 {
                continue;
            }
            let (yt, yb) = (HY - (ht - EYE) * sc, HY - (hb - EYE) * sc);
            for py in span(yt, yb, H) {
                let y = row_y(py);
                let k = py * W + px;
                if y < yt || y >= yb || f64::from(st.near[k]) >= sc {
                    continue;
                }
                let v = match sg.kind {
                    Kind::Neon => 0.14,
                    Kind::Bulbs => 0.3,
                    _ => 0.5,
                };
                let c = match sg.kind {
                    Kind::White => [0.95, 0.93, 0.88],
                    Kind::Neon => hue,
                    _ => pale,
                };
                st.mat[k] = Mat::Side;
                st.near[k] = sc as f32;
                st.emit(k, c[0] * v, c[1] * v, c[2] * v);
                st.grp[k] = sg.group;
            }
        }
    }

    // the andon's legs
    {
        let s = 18.0;
        let (y0, y1) = ((HY + (EYE - 0.14) * s).round(), (HY + EYE * s).round());
        for xw in [2.1, 2.4] {
            let x = (VX + xw * s).floor();
            for py in span(y0, y1, H) {
                for px in span(x, x + 1.0, W) {
                    let k = py * W + px;
                    if mid_x(px).floor() == x && f64::from(st.near[k]) < s {
                        st.mat[k] = Mat::Pole;
                        st.near[k] = s as f32;
                        st.surface(k, 0.05, 0.05, 0.06, 0.5);
                    }
                }
            }
        }
    }

    // --- the lanterns, at rest, for the glow ----------------------------------
    let mut lan = z3();
    for li in 0..LANTERNS.len() {
        lantern(li, 0.0, |k, r, g, b| {
            lan[0][k] = r as f32;
            lan[1][k] = g as f32;
            lan[2][k] = b as f32;
        });
    }

    // --- light: the glow round every light, what it falls on, the haze -------
    let mut fog = vec![0.0f32; n];
    let mut hz = z3();
    for k in 0..n {
        let s = f64::from(st.near[k]);
        fog[k] = if st.mat[k] == Mat::Tower {
            0.1
        } else if s > 0.0 {
            (1.0 - (-F / s / ZF).exp()) as f32
        } else {
            0.0
        };
        // the haze itself is lit, brightest down at the far end of the street
        let g = end_glow(mid_x(k % W), row_y(k / W)) * 1.1;
        hz[0][k] = (0.1 + g * END[0]) as f32;
        hz[1][k] = (0.07 + g * END[1]) as f32;
        hz[2][k] = (0.16 + g * END[2]) as f32;
    }
    // The picture before anything moves, from a set of lights: the colour of
    // each pixel and the glow there. Lights that flicker run it again with them off.
    let shade = |e: &Rgb3| -> (Rgb3, Rgb3) {
        let g: Rgb3 = std::array::from_fn(|c| glow(&e[c].iter().zip(&lan[c]).map(|(a, b)| a + b).collect::<Vec<f32>>()));
        let mut col = z3();
        for k in 0..n {
            let m = st.mat[k];
            for c in 0..3 {
                let gk = g[c][k];
                col[c][k] = if m == Mat::Sky || m == Mat::Wire {
                    let v = st.sky[c][k] + gk * 0.06;
                    if m == Mat::Wire {
                        v * 0.3 + gk * 0.3
                    } else {
                        v
                    }
                } else {
                    let kg = (st.kg[k] + if m == Mat::Sign { 0.05 } else { 0.0 }) * 2.4;
                    let amb = [0.06, 0.05, 0.09][c];
                    let v = e[c][k] + st.a[c][k] * (amb + gk * kg);
                    v + (hz[c][k] + gk * 0.3 - v) * fog[k]
                };
            }
        }
        (col, g)
    };

    // The wet road as a mirror: each pixel above it lands where its depth
    // says, the nearest winning, and the gloss smears what lands up and down
    // the road.
    let up_k = 0.96f32.powf(1.0 / S as f32);
    let down_k = 0.9f32.powf(1.0 / S as f32);
    let mirror = |col: &Rgb3| -> Rgb3 {
        let mut x3 = [vec![0.0f32; W * XH], vec![0.0f32; W * XH], vec![0.0f32; W * XH]];
        let mut xd = vec![-1.0f32; W * XH];
        for k in 0..n {
            let m = st.mat[k];
            if m == Mat::Road {
                continue;
            }
            let (px, py) = (k % W, k / W);
            let s = st.near[k];
            let rr = ((2.0 * HY + 2.0 * EYE * f64::from(s) - row_y(py)) * S).floor();
            if rr < HYP as f64 || rr >= XH as f64 {
                continue;
            }
            let rr = rr as usize;
            if rr < H && st.mat[rr * W + px] != Mat::Road {
                continue;
            }
            let q = rr * W + px;
            if s <= xd[q] {
                continue;
            }
            xd[q] = s;
            let f = m.give();
            for c in 0..3 {
                x3[c][q] = col[c][k] * f;
            }
        }
        let mut out = z3();
        let mut up = [vec![0.0f32; XH], vec![0.0f32; XH], vec![0.0f32; XH]];
        for px in 0..W {
            for r in HYP + 1..XH {
                let q = r * W + px;
                if xd[q] < 0.0 && xd[q - W] >= 0.0 {
                    for x in x3.iter_mut() {
                        x[q] = x[q - W];
                    }
                    xd[q] = 0.0;
                }
            }
            // The gloss: light pulled a long way up the road, so the near
            // signs, whose mirror images fall below the frame, still leave a
            // streak under them.
            for c in 0..3 {
                let mut u = 0.0;
                for r in (HYP..XH).rev() {
                    u = u * up_k + x3[c][r * W + px] * (1.0 - up_k);
                    up[c][r] = u;
                }
            }
            // and a shorter smear down the road toward us
            let mut d = [0.0f32; 3];
            for r in HYP..H {
                let q = r * W + px;
                for c in 0..3 {
                    d[c] = d[c] * down_k + x3[c][q] * (1.0 - down_k);
                }
                if st.mat[q] != Mat::Road {
                    continue;
                }
                let fres = (0.45 + 0.55 * smooth(TSH + 6.0, HY + 3.0, row_y(r))) as f32;
                for c in 0..3 {
                    out[c][q] = (x3[c][q] * 0.35 + d[c] * 0.5 + up[c][r] * 1.1) * fres;
                }
            }
        }
        out
    };

    let (c, g) = shade(&st.e);
    let mut refl = mirror(&c);

    // Each flickering group: what it adds, to the picture and to the road's
    // mirror, so a frame can take it away again when the group goes dark.
    let mut drop_c = vec![];
    let mut drop_r = vec![];
    for gi in 1..=NG as u8 {
        let mut e = st.e.clone();
        for k in 0..n {
            if st.grp[k] != gi {
                continue;
            }
            // a dark tube still shows a little; a dead letter shows its panel
            for ch in 0..3 {
                e[ch][k] = if gi == 3 { ([0.03, 0.025, 0.045][ch] + PINK[ch] * 0.05) as f32 } else { e[ch][k] * 0.08 };
            }
        }
        let (cg, _) = shade(&e);
        let (mut ks, mut vs) = (vec![], vec![]);
        for k in 0..n {
            let dv = [c[0][k] - cg[0][k], c[1][k] - cg[1][k], c[2][k] - cg[2][k]];
            if dv[0].abs() + dv[1].abs() + dv[2].abs() > 0.004 {
                ks.push(k as u32);
                vs.extend_from_slice(&dv);
            }
        }
        drop_c.push((ks, vs));
        let rg = mirror(&cg);
        drop_r.push(std::array::from_fn(|ch| refl[ch].iter().zip(&rg[ch]).map(|(a, b)| a - b).collect()));
    }

    // The far end's glow laid down the middle of the road: a long bright
    // path, wider as it nears, that the walker crosses.
    for k in 0..n {
        if st.mat[k] != Mat::Road {
            continue;
        }
        let s = f64::from(st.near[k]);
        let d = (mid_x(k % W) - VX).abs();
        let path = (-(d / (1.4 + 0.36 * s)).powf(1.6)).exp() * (0.16 + 0.7 * (-s / 12.0).exp());
        for ch in 0..3 {
            refl[ch][k] += (path * END[ch]) as f32;
        }
    }

    let road = (0..n).filter(|&k| st.mat[k] == Mat::Road).map(|k| k as u32).collect();
    let sky = (0..n).filter(|&k| st.mat[k] == Mat::Sky).map(|k| k as u32).collect();

    // low cloud: heaped noise that wraps, so it can drift forever
    let mut cloud = vec![0.0f32; CW * HYP];
    for py in 0..HYP {
        for px in 0..CW {
            cloud[py * CW + px] = fbm_wrap(px as f64 / S * 0.03, (row_y(py) - 0.5) * 0.09, 4, CW as f64 / S * 0.03) as f32;
        }
    }

    // rain in three depths
    let mut drops = vec![];
    for (li, &(count, v, len, b, sl)) in LAYERS.iter().enumerate() {
        for i in 0..count {
            let h = |m: f64| hash((i * 7 + li * 1001) as f64, m);
            drops.push([h(1.0) * 230.0 - 20.0, h(2.0) * (TSH + len + 6.0), v * (0.85 + 0.3 * h(3.0)), len, b * (0.75 + 0.5 * h(4.0)), sl]);
        }
    }

    Box::new(TokyoRain {
        mat: st.mat,
        near: st.near,
        wet: st.wet,
        fog,
        hz,
        c,
        g,
        refl,
        drop_c,
        drop_r,
        road,
        sky,
        cloud,
        drops,
        bulbs,
        f: z3(),
        rain: vec![0.0; n],
        hold: vec![0.0; n],
        pixels: Pixels::new(&META),
    })
}

impl Piece for TokyoRain {
    #[allow(clippy::needless_range_loop)] // the index reads several arrays at once
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        for c in 0..3 {
            self.f[c].copy_from_slice(&self.c[c]);
        }
        let mg = [1.0, flicker(1, t), flicker(2, t), flicker(3, t)];
        for gi in 1..=NG {
            let off = (1.0 - mg[gi]) as f32;
            if off <= 0.0 {
                continue;
            }
            let (ks, vs) = &self.drop_c[gi - 1];
            for (i, &k) in ks.iter().enumerate() {
                for c in 0..3 {
                    self.f[c][k as usize] -= vs[i * 3 + c] * off;
                }
            }
        }

        // the sky: cloud drifting slowly across the strip between the roofs
        let drift = t * 0.9 * S;
        for &k in &self.sky {
            let k = k as usize;
            let (px, py) = (k % W, k / W);
            let sx = px as f64 + drift;
            let ix = sx.floor();
            let fx = sx - ix;
            let i0 = (ix as usize) % CW;
            let (c0, c1) = (f64::from(self.cloud[py * CW + i0]), f64::from(self.cloud[py * CW + (i0 + 1) % CW]));
            let c = (0.72 + 0.7 * smooth(0.35, 0.75, c0 + (c1 - c0) * fx)) as f32;
            for ch in 0..3 {
                self.f[ch][k] *= c;
            }
        }

        // the walker, and the patch of road their reflection hides
        let wk = walker(t);
        let fs = wk.s;
        let (fx, fy) = (VX + wk.xw * fs, HY + EYE * fs);
        let fz = fs * 1.12; // drawn a touch tall, so they hold the middle of the frame
                            // their reflection lies on the street only once they are out of the alley
        let seen = wk.fade * smooth(3.5, 2.7, wk.xw);
        let bxs = span(fx - 0.62 * fz - 1.0, fx + 0.62 * fz + 1.0, W);

        // the road: the mirror, wavering and broken by the rain
        for &k in &self.road {
            let k = k as usize;
            let (px, py) = (k % W, k / W);
            let (x, y) = (mid_x(px) - 0.5, row_y(py));
            let v = (y - HY) / (TSH - HY);
            let wob = (noise(x * 0.07 + 7.0, y * 0.55 - t * 0.9) - 0.5) * (0.6 + 4.0 * v) + (noise(x * 0.3, y * 1.3 - t * 2.1) - 0.5) * (0.3 + 1.2 * v);
            let sx = (px as f64 + wob * S).clamp(0.0, W as f64 - 1.001);
            let j = py * W + sx as usize;
            let f = (sx - sx.floor()) as f32;
            // broken into bands that roll toward us, as rain stirs the puddles
            let dash = 0.3 + smooth(0.3, 0.65, noise(x * 0.09 + 3.0, y * (0.7 + 0.5 * v) - t * 1.5));
            let mut rv = [0.0f32; 3];
            for c in 0..3 {
                let r = &self.refl[c];
                rv[c] = r[j] + (r[j + 1] - r[j]) * f;
            }
            for gi in 1..=NG {
                let off = (1.0 - mg[gi]) as f32;
                if off <= 0.0 {
                    continue;
                }
                for c in 0..3 {
                    let d = &self.drop_r[gi - 1][c];
                    rv[c] -= (d[j] + (d[j + 1] - d[j]) * f) * off;
                }
            }
            let mut a = dash * f64::from(self.wet[k]);
            if seen > 0.0 && bxs.contains(&px) && y > fy && y < fy + 2.3 * fz && walker_shape((mid_x(px) - fx) / fz, (y - fy) / fz, wk.ph) != 0 {
                a *= 1.0 - 0.8 * seen;
            }
            for c in 0..3 {
                self.f[c][k] += rv[c].max(0.0) * a as f32;
            }
        }

        // the lanterns swing a little in the wind
        for (li, &(ls, _, _, phase)) in LANTERNS.iter().enumerate() {
            let sw = 0.05 * (t * 1.1 + phase).sin() + 0.015 * (t * 2.7 + li as f64).sin();
            let (near, fog, hz, f) = (&self.near, &self.fog, &self.hz, &mut self.f);
            lantern(li, sw, |k, r, g, b| {
                if f64::from(near[k]) > ls {
                    return;
                }
                let m = f64::from(fog[k]) * 0.3;
                for (c, v) in [r, g, b].into_iter().enumerate() {
                    f[c][k] = mix(v, f64::from(hz[c][k]), m) as f32;
                }
            });
        }

        // the bulbs chase round their sign
        let chase = (t * 6.0).floor() as u32;
        for &(k, i) in &self.bulbs {
            let on = !(i + chase).is_multiple_of(3);
            let c = if on { [1.0, 0.9, 0.7] } else { [0.22, 0.1, 0.06] };
            for ch in 0..3 {
                self.f[ch][k as usize] = c[ch];
            }
        }

        // the traffic light far down the street, and the tower's beacon
        {
            let c = t % 24.0;
            let lamp = if c < 14.0 {
                0
            } else if c < 17.0 {
                1
            } else {
                2
            };
            let k = 66 * W + 159 + 2 * lamp;
            let col = [[0.25, 1.0, 0.55], [1.0, 0.75, 0.15], [1.0, 0.15, 0.1]][lamp];
            for ch in 0..3 {
                self.f[ch][k] = col[ch];
                for d in [k - 1, k + 1, k - W, k + W] {
                    self.f[ch][d] += col[ch] * 0.12;
                }
            }
            let on = t % 1.6 < 0.35;
            let cb = if on { [1.0, 0.15, 0.1] } else { [0.2, 0.05, 0.05] };
            for k in [13 * W + 159, 13 * W + 160] {
                for ch in 0..3 {
                    self.f[ch][k] = cb[ch];
                }
            }
        }

        // the walker: a dark shape against the far glow, the umbrella's crown catching the neon
        self.hold.fill(0.0);
        if wk.fade > 0.0 {
            // near, they are a solid shape; the haze takes them only further off
            let f = 0.7 * smooth(8.0, 26.0, wk.z);
            for py in span(fy - 2.3 * fz - 1.0, fy + 1.0, H) {
                for px in bxs.clone() {
                    let k = py * W + px;
                    if f64::from(self.near[k]) >= fs {
                        continue;
                    }
                    let (mut cov, mut lit) = (0.0, 0.0);
                    for q in 0..4 {
                        let u = ((px as f64 + 0.25 + 0.5 * (q & 1) as f64) / S - fx) / fz;
                        let v = (fy - (py as f64 + 0.25 + 0.5 * (q >> 1) as f64) / S) / fz;
                        let sh = walker_shape(u, v, wk.ph);
                        if sh != 0 {
                            cov += 0.25;
                        }
                        if sh == 2 {
                            lit += 0.25;
                        }
                    }
                    if cov == 0.0 {
                        continue;
                    }
                    // dark, but for the wet crown of the umbrella catching the signs
                    let l = lit / cov;
                    let a = cov * wk.fade;
                    for (ch, base) in [0.006, 0.005, 0.01].into_iter().enumerate() {
                        let c = mix(base + l * f64::from(self.g[ch][k]) * 0.12, f64::from(self.hz[ch][k]), f);
                        self.f[ch][k] = mix(f64::from(self.f[ch][k]), c, a) as f32;
                    }
                    self.hold[k] = (a * (1.0 - f)) as f32;
                }
            }
        }

        // rings spreading where drops land on the road, and the splash at their start
        for i in 0..34 {
            let fi = f64::from(i);
            let per = 0.7 + 0.6 * hash(fi, 301.0);
            let tt = t + hash(fi, 302.0) * per;
            let c = (tt / per).floor();
            let a = (tt - c * per) / per;
            // spread evenly down the screen, so most land near enough to see
            let cy = 56.0 + 43.0 * hash(fi * 7.0 + c, 303.0);
            let xw = (hash(fi * 5.0 + c, 304.0) * 2.0 - 1.0) * 2.7;
            let s = (cy - HY) / EYE;
            let cx = VX + xw * s;
            let rad = (0.04 + 0.18 * a) * s;
            let ry = (rad * EYE * s / F).max(0.6);
            let b = (1.0 - a).powf(1.4) * 0.8;
            for py in span(cy - ry - 1.0, cy + ry + 1.0, H) {
                for px in span(cx - rad - 1.0, cx + rad + 1.0, W) {
                    let k = py * W + px;
                    if self.mat[k] != Mat::Road {
                        continue;
                    }
                    let d = ((mid_x(px) - cx) / rad.max(0.6)).hypot((row_y(py) - cy) / ry);
                    let ring = (-((d - 1.0) / 0.35).powi(2)).exp() * b + if a < 0.12 && d < 0.6 { 0.5 } else { 0.0 };
                    if ring < 0.02 {
                        continue;
                    }
                    for (ch, base) in [0.4, 0.44, 0.55].into_iter().enumerate() {
                        self.f[ch][k] += (ring * (base + f64::from(self.refl[ch][k]) * 2.0)) as f32;
                    }
                }
            }
        }

        // the rain, falling at three depths, a pixel thin
        self.rain.fill(0.0);
        for &[x0, y0, v, len, b, sl] in &self.drops {
            let span_y = TSH + len + 6.0;
            let head = (y0 + v * t) % span_y - 3.0;
            let lp = (len * S).round() as usize;
            for i in 0..lp {
                let y = head - i as f64 / S;
                let py = (y * S).floor();
                if py < 0.0 || py >= H as f64 {
                    continue;
                }
                let px = ((x0 + sl * y) * S).floor();
                if px < 0.0 || px >= W as f64 {
                    continue;
                }
                let k = py as usize * W + px as usize;
                let a = (b * (1.0 - 0.7 * i as f64 / lp as f64)) as f32;
                if a > self.rain[k] {
                    self.rain[k] = a;
                }
            }
        }

        let ground = [0.027f32, 0.024, 0.051];
        for py in 0..H {
            let fade = smooth(TSH + 8.0, TSH - 6.0, row_y(py)) as f32;
            for px in 0..W {
                let k = py * W + px;
                let mut c = [self.f[0][k], self.f[1][k], self.f[2][k]];
                // rain falls in front of everything, but lets the walker keep their shape
                // (three quarters of ascii.rest's light: a solid streak shows more than its dots)
                let rn = 0.75 * self.rain[k] * (1.0 - 0.85 * self.hold[k]);
                if rn > 0.0 {
                    // rain catches whatever light is round it
                    for (ch, base) in [0.42, 0.47, 0.6].into_iter().enumerate() {
                        c[ch] += rn * (base + self.g[ch][k] * 0.8);
                    }
                }
                // A lifted curve, gentler than ascii.rest's for its dots: most
                // of this street is lit dimly, and should still be seen.
                let peak = c[0].max(c[1]).max(c[2]).max(1e-4);
                let lift = peak.powf(LIFT - 1.0);
                let px_c = [
                    ground[0] + ((c[0] * lift).max(0.0) - ground[0]) * fade,
                    ground[1] + ((c[1] * lift).max(0.0) - ground[1]) * fade,
                    ground[2] + ((c[2] * lift).max(0.0) - ground[2]) * fade,
                ];
                self.pixels.set(px, py, px_c);
            }
        }
        self.pixels.draw(grid, env);
    }
}
