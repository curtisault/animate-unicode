//! landscape: a range over a still lake, under the sky as it is right now for
//! whoever is looking: the sun climbing or sinking by the local hour, or the
//! moon in its real phase among the stars, with the range mirrored in the
//! water. After ascii.rest's landscape.ts (MIT, @bas3line).
//!
//! The range, the sun's and the moon's arcs and the moon's phase, the clouds,
//! the stars, the ripples and the path of light are ascii.rest's. The sky, the
//! land and the lake are a picture of square pixels, two to a cell
//! ([`crate::pixels`]), and wherever two things meet inside a cell (the
//! ridge against the sky, a cloud's edge, the sun's or the moon's limb, the
//! ridge's reflection) the cell is cut in octants, so the range is a solid
//! silhouette with a smooth crest and the moon's terminator falls between
//! dots. The sun's rays, the stars, the ripples and the glints on the path
//! are braille. The sky's colour follows the hour: night blue, a warm band
//! low on the side of the sunrise or the sunset, a blue day with a halo round
//! the sun; the clouds and the land take the same light. In one ink there are
//! no pixels: the ridge, the clouds and the reflection are drawn as octant
//! outlines, the land stippled, as ascii.rest draws them.

use crate::clock::Ticker;
use crate::noise::smooth;
use crate::pixels::{Pixels, Rgb};
use crate::subcell::{Braille, Encoding, Octant};
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// Landscape's meta.
pub static META: Meta = Meta {
    name: "landscape",
    slug: "landscape",
    category: Category::Nature,
    note: "a range and a lake under the sun or moon of the local hour",
    cols: 64,
    rows: 18,
    fps: 8,
    charset: Charset::Extended,
    options: Some(r#"{"hour": null, "phase": null}"#),
    clock: true,
    // A bright star, a faint one, the moon and its seas, the ground, the low
    // sun and the high; then, fitted to the colours the sky, the land, the
    // clouds and the lake take round the clock, dark to light: night blues,
    // dusk violets and browns, day blues and greys, the sunset's golds, the
    // clouds' whites.
    palette: Some(&[
        "#fffaea", "#dfe6ff", "#ecebdf", "#a8a8a4", "#05070f", "#ff8a3a", "#fff2c8", "#0a0d19", "#0f1221", "#12172a", "#1e1b27", "#181e36", "#252432",
        "#2a2b3e", "#382e37", "#303846", "#2c3656", "#413848", "#503c3a", "#3d4357", "#55444c", "#3b506f", "#4b4f5f", "#6a4f49", "#39588c", "#485d79",
        "#5f5a66", "#805e4f", "#50688a", "#5c6977", "#79686b", "#4171b5", "#5c718e", "#976e57", "#4d7ab9", "#727a81", "#667b98", "#5883be", "#977e73",
        "#74859e", "#848b92", "#b9825d", "#658dc2", "#b0907b", "#7297c7", "#8c96a5", "#7fa1cc", "#cba07e", "#ff974b", "#8dacd1", "#9bb9da", "#c1b8b4",
        "#acbccd", "#ffb476", "#ebc494", "#ffbe8b", "#c7cdd4", "#ffd29f", "#d2d7de", "#dde0e5", "#ffe8bd", "#f4f1e3", "#fff5d6", "#f4f3f5",
    ]),
    ground: Some("#05070f"),
    ..Meta::DEFAULT
};

const COLS: usize = 64;
const ROWS: usize = 18;
/// The bottom edge of this row is the waterline.
const SHORE: i64 = 11;
/// The waterline, in rows.
const WATER: f64 = (SHORE + 1) as f64;
/// The sun with its rays, centred on row 2, column 5; rows 1 to 3 are its disc.
const SUN: [&str; 5] = ["  \\  |  /  ", "   .---.   ", "--(     )--", "   '---'   ", "  /  |  \\  "];
/// The moon's rim, drawn where it is lit, round a face (#) that takes the light.
const MOON: [&str; 5] = ["  .-''-.  ", " /######\\ ", "|########|", " \\######/ ", "  '-..-'  "];
/// Face cells in its darker seas, as (row, column).
const MARIA: [(usize, usize); 4] = [(1, 4), (2, 6), (2, 7), (3, 3)];
/// Clouds with flat bases: # is a cloud's body, which hides what is behind it.
const CLOUD: [&str; 4] = ["        .-~~-.          ", "   .-~~'######'-.  .-.  ", " .'##############'~###'.", "(______________________)"];
const PUFF: [&str; 3] = ["   .-~~-.   ", " .'######'. ", "(__________)"];
/// Each cloud: shape, row, columns behind the first. They keep their spacing
/// as they drift, clear of each other and of the highest peak.
const CLOUDS: [(&[&str], i64, f64); 2] = [(&CLOUD, 0, 0.0), (&PUFF, 2, 32.0)];
/// Columns a second.
const DRIFT: f64 = 0.4;
/// Peaks as column, height, and how far they reach west and east.
const PEAKS: [(f64, f64, f64, f64); 4] = [(13.0, 5.0, 10.0, 5.0), (33.0, 4.0, 5.0, 6.0), (43.0, 6.0, 6.0, 11.0), (56.0, 3.0, 4.0, 5.0)];
/// Stars as row, column, glyph, and how dark it must be: the Plough first,
/// its brightest out first after sunset, then fainter ones.
const STARS: [(i64, i64, char, f64); 18] = [
    (0, 45, '*', 0.0),
    (2, 45, '.', 0.15),
    (3, 50, '.', 0.15),
    (2, 52, '.', 0.3),
    (2, 56, '*', 0.0),
    (3, 59, '.', 0.1),
    (4, 61, '*', 0.0),
    (0, 7, '.', 0.5),
    (1, 24, '\'', 0.7),
    (0, 34, '.', 0.4),
    (2, 3, '.', 0.8),
    (3, 13, '\'', 0.6),
    (2, 30, '.', 0.9),
    (4, 37, '.', 0.75),
    (1, 39, '\'', 0.55),
    (3, 21, '.', 0.85),
    (5, 4, '.', 0.65),
    (0, 17, '.', 0.45),
];

/// ascii.rest's landscape hash of two integers to 0–1 (each truncated to 32
/// bits, as `Math.imul` does).
fn hash(a: i64, b: i64) -> f64 {
    let h = (a as i32).wrapping_mul(0x27d4_eb2d) ^ (b as i32).wrapping_mul(0x1656_67b1);
    let h = (h ^ ((h as u32) >> 15) as i32).wrapping_mul(0x85eb_ca6b_u32 as i32);
    ((h ^ ((h as u32) >> 13) as i32) as u32) as f64 / 4_294_967_296.0
}

/// The moon's age in lunations at a moment, from a known new moon (6 Jan
/// 2000, 18:14 UTC).
fn lunation(unix: f64) -> f64 {
    let days = unix * 1000.0 / 864e5;
    ((((days - 10962.76) / 29.530_588_853) % 1.0) + 1.0) % 1.0
}

/// Where a body stands for its hour on a 12 hour arc that starts at 0, as
/// the column and row of its middle.
fn place(u: f64) -> (i64, i64) {
    let alt = (PI * u).sin();
    ((5.0 + (COLS as f64 - 11.0) * u).round() as i64, (SHORE as f64 + 2.0 - SHORE as f64 * alt.max(0.0).powf(0.45)).round() as i64)
}

/// The range, fixed: heights at the column edges, the stroke and first row
/// of land in each column, the outline in the water, and the ripples.
struct Range {
    /// Heights in rows above the waterline at each column's left edge.
    e: Vec<i64>,
    /// The ridge stroke in each column and the row it sits in.
    range: Vec<(i64, char)>,
    /// The range's outline upside down in the water: row, column, glyph.
    mirror: Vec<(i64, usize, char)>,
    /// Ripples: row, start column, length, columns a second.
    ripples: Vec<(usize, f64, i64, f64)>,
}

impl Range {
    fn new() -> Self {
        // Peaks steep at the summit and easing out, as one stroke per column
        // (up, down or along) with the row it sits in.
        let mut e: Vec<i64> = vec![];
        for x in 0..=COLS {
            let xf = x as f64;
            let f = PEAKS.iter().map(|&(px, h, w, o)| h * (1.0 - (xf - px).abs() / if xf < px { w } else { o }).max(0.0).powf(1.3)).fold(0.0, f64::max);
            let f = f.round() as i64;
            e.push(if x > 0 { (e[x - 1] - 1).max((e[x - 1] + 1).min(f)) } else { f });
        }
        let range: Vec<(i64, char)> = (0..COLS)
            .map(|c| {
                if e[c + 1] > e[c] {
                    (SHORE - e[c], '/')
                } else if e[c + 1] < e[c] {
                    (SHORE - e[c + 1], '\\')
                } else {
                    (SHORE - e[c], '_')
                }
            })
            .collect();
        // The slopes only, the flats along the shore left out.
        let mut mirror = vec![];
        for (c, &(r, ch)) in range.iter().enumerate() {
            match ch {
                '/' => mirror.push((2 * SHORE + 1 - r, c, '\\')),
                '\\' => mirror.push((2 * SHORE + 1 - r, c, '/')),
                _ if r < SHORE => mirror.push((2 * SHORE + 1 - r, c, '-')),
                _ => {}
            }
        }
        // One short ripple a row below the first, a little longer toward the near shore.
        let ripples = (2..)
            .take_while(|k| SHORE + k < ROWS as i64)
            .map(|k| ((SHORE + k) as usize, (hash(k, 7) * COLS as f64).floor(), 2 + i64::from(k > 4), if k % 2 == 1 { 0.3 } else { -0.25 }))
            .collect();
        Range { e, range, mirror, ripples }
    }

    /// The first row of land in column c.
    fn top(&self, c: usize) -> i64 {
        self.range[c].0
    }

    /// Whether (x, y), in columns and rows, is land: under the ridge's line,
    /// straight between the heights at the column edges, or on the shore
    /// along the waterline.
    fn land(&self, x: f64, y: f64) -> bool {
        if !(0.0..WATER).contains(&y) || x < 0.0 || x >= COLS as f64 {
            return false;
        }
        let c = x.floor() as usize;
        let s = self.e[c] as f64 + (self.e[c + 1] - self.e[c]) as f64 * (x - c as f64);
        WATER - y < s || y >= WATER - 0.25
    }

    /// The land's height above (x, y), in rows: how deep in it a point is.
    fn depth(&self, x: f64, y: f64) -> f64 {
        let c = (x.floor().max(0.0) as usize).min(COLS - 1);
        let s = self.e[c] as f64 + (self.e[c + 1] - self.e[c]) as f64 * (x - c as f64);
        (s - (WATER - y)).max(0.0)
    }

    /// The land seen in the water: the range upside down, without the shore.
    fn mirrored(&self, x: f64, y: f64) -> bool {
        let y = 2.0 * WATER - y;
        y < WATER - 0.25 && self.land(x, y)
    }
}

/// A cloud cell: the glyph of ascii.rest's shape there, and how far down the
/// cloud it is, 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Puff {
    ch: char,
    down: f64,
}

/// One moment of the landscape as ascii.rest works it out.
struct Moment {
    /// 0 at six in the morning, 1 at six in the evening, to 2.
    sun_u: f64,
    day: bool,
    /// The water's glints change twice a second.
    step: i64,
    /// The moon's age in lunations.
    age: f64,
    /// The sun's or the moon's middle, as column and row.
    cx: i64,
    cy: i64,
    /// Whether the moon is up (at night).
    up: bool,
    /// How full the moon is (1 for the sun).
    full: f64,
    /// How many cells of the sun's or the moon's body show.
    lit: usize,
    /// The clouds, by row and column.
    cloud: Vec<Vec<Option<Puff>>>,
    /// The stars that show, as row, column and glyph.
    stars: Vec<(usize, usize, char)>,
    /// The sun's rays that show, as row, column and glyph.
    rays: Vec<(usize, usize, char)>,
    /// Cells of ripple, and of the path of light with its glyph.
    ripples: Vec<(usize, usize)>,
    path: Vec<(usize, usize, char)>,
    /// The frame as ascii.rest draws it.
    text: Vec<Vec<char>>,
}

impl Moment {
    fn cloud_at(&self, r: i64, c: i64) -> bool {
        r >= 0 && c >= 0 && (r as usize) < ROWS && (c as usize) < COLS && self.cloud[r as usize][c as usize].is_some()
    }
}

/// ascii.rest's frame at play time t, hour h and moon age `age`.
#[allow(clippy::needless_range_loop)] // the index reads several arrays at once
fn moment(rg: &Range, t: f64, h: f64, age: f64) -> Moment {
    let cols = COLS as i64;
    let mut g: Vec<Vec<char>> = (0..ROWS)
        .map(|r| {
            (0..COLS)
                .map(|c| {
                    let (top, ch) = rg.range[c];
                    let r = r as i64;
                    if r > SHORE {
                        ' '
                    } else if r == top {
                        ch
                    } else if r > top && (c as i64 + r) % 2 == 1 {
                        '.'
                    } else {
                        ' '
                    }
                })
                .collect()
        })
        .collect();
    let sky = |r: i64, c: i64| r >= 0 && c >= 0 && c < cols && r < rg.top(c as usize);
    let ground = |r: i64, c: i64| c >= 0 && c < cols && r >= rg.top(c as usize) && r <= SHORE;

    let sun_u = (((h - 6.0) % 24.0) + 24.0) % 24.0 / 12.0;
    let day = sun_u < 1.0;
    let dark = if day { 0.0 } else { ((PI * (sun_u - 1.0)).sin() * 5.0).min(1.0) };
    let step = (t * 2.0).floor() as i64;
    // The moon keeps the sun's hours, late by its age: full moons rise at dusk.
    let moon_u = (((sun_u - age * 2.0) % 2.0) + 2.0) % 2.0;
    let (cx, cy) = if day { place(sun_u) } else { place(moon_u.min(1.5)) };
    let up = day || moon_u < 1.0;

    // Clouds drift across in front of it all, and come round again. They set
    // out from across the sky, so the first frame shows the sun or moon clear.
    let mut cloud: Vec<Vec<Option<Puff>>> = vec![vec![None; COLS]; ROWS];
    let w0 = CLOUD[0].len() as f64;
    let span = cols as f64 + w0;
    for (shape, r0, d) in CLOUDS {
        let x = (((cx as f64 + 15.0 + d + w0 + DRIFT * t) % span) - w0).floor() as i64;
        for (i, line) in shape.iter().enumerate() {
            for (k, ch) in line.chars().enumerate() {
                let (r, c) = (r0 + i as i64, x + k as i64);
                if ch != ' ' && sky(r, c) {
                    cloud[r as usize][c as usize] = Some(Puff { ch, down: i as f64 / (shape.len() - 1) as f64 });
                }
            }
        }
    }
    let mut m =
        Moment { sun_u, day, step, age, cx, cy, up, full: 1.0, lit: 0, cloud, stars: vec![], rays: vec![], ripples: vec![], path: vec![], text: vec![] };

    // Stars keep a cell clear of clouds and of the moon.
    let near = |m: &Moment, r: i64, c: i64| {
        for dr in -1..=1 {
            for dc in -1..=1 {
                if m.cloud_at(r + dr, c + dc) {
                    return true;
                }
            }
        }
        up && !day && (c - cx).abs() <= 6 && (r - cy).abs() <= 3
    };
    for &(row, col, ch, need) in &STARS {
        if dark <= need || !sky(row, col) || near(&m, row, col) {
            continue;
        }
        // The faint ones twinkle.
        let ch = if need > 0.3 && hash(col, step) < 0.15 { ' ' } else { ch };
        g[row as usize][col as usize] = ch;
        if ch != ' ' {
            m.stars.push((row as usize, col as usize, ch));
        }
    }

    // Draws a cell of the sun or moon, counting what shows of its body. A
    // ray or rim stroke beside land or cloud would read as part of it, so it goes.
    let put = |m: &mut Moment, g: &mut Vec<Vec<char>>, r: i64, c: i64, ch: char, solid: bool, body: bool| {
        if !sky(r, c) || m.cloud_at(r, c) {
            return false;
        }
        if !solid {
            for dr in -1..=1 {
                for dc in -1..=1 {
                    if ground(r + dr, c + dc) || m.cloud_at(r + dr, c + dc) {
                        return false;
                    }
                }
            }
        }
        g[r as usize][c as usize] = ch;
        if body {
            m.lit += 1;
        }
        true
    };
    if day {
        for (i, line) in SUN.iter().enumerate() {
            for (k, ch) in line.chars().enumerate() {
                let (r, c) = (cy - 2 + i as i64, cx - 5 + k as i64);
                let disc = if i == 2 { k > 1 && k < 9 } else { i % 2 == 1 && k > 2 && k < 8 };
                if ch != ' ' && put(&mut m, &mut g, r, c, ch, disc, disc) && !disc {
                    m.rays.push((r as usize, c as usize, ch));
                }
            }
        }
    } else if up {
        // Lit where the sunward side is: waxing on the right, waning on the left.
        let k = (2.0 * PI * (age % 0.5)).cos();
        let shine = |x: f64, y: f64| x * x + y * y <= 1.0 && (age < 0.5) == (x > k * (1.0 - y * y).max(0.0).sqrt());
        let edge = if k > 0.25 {
            ')'
        } else if k < -0.25 {
            '('
        } else {
            '|'
        };
        m.full = (1.0 - (2.0 * PI * age).cos()) / 2.0;
        for (i, line) in MOON.iter().enumerate() {
            for (j, ch) in line.chars().enumerate() {
                if ch == ' ' {
                    continue;
                }
                let (r, c) = (cy - 2 + i as i64, cx - 5 + j as i64);
                let (x, y) = ((j as f64 + 0.5 - 5.0) / 5.0, (i as f64 + 0.5 - 2.5) / 2.5);
                if ch != '#' {
                    // A rim cell is lit by the edge of the face it lies on.
                    let s = 0.97 / x.hypot(y);
                    if shine(x * s, y * s) {
                        put(&mut m, &mut g, r, c, ch, false, true);
                    }
                    continue;
                }
                let on = (0..9).filter(|q| shine(x + f64::from(q % 3 - 1) / 15.0, y + f64::from(q / 3 - 1) / 7.5)).count();
                if on >= 6 {
                    put(&mut m, &mut g, r, c, if MARIA.contains(&(i, j)) { '.' } else { ':' }, true, true);
                } else if on >= 2 {
                    put(&mut m, &mut g, r, c, edge, true, true);
                } else if sky(r, c) {
                    g[r as usize][c as usize] = ' ';
                }
            }
        }
    }
    for (r, row) in m.cloud.iter().enumerate().take(SHORE as usize + 1) {
        for (c, p) in row.iter().enumerate() {
            if let Some(p) = p {
                g[r][c] = if p.ch == '#' { ' ' } else { p.ch };
            }
        }
    }

    // The water: the range's outline upside down, a few ripples drifting
    // along it, and a path of light under the sun or moon kept clear of the rest.
    for &(r, c, ch) in &rg.mirror {
        if r < ROWS as i64 {
            g[r as usize][c] = ch;
        }
    }
    let mut ripples = vec![];
    for &(r, c0, n, v) in &rg.ripples {
        let wrap = (cols + 4) as f64;
        let x = ((((c0 + v * t) % wrap) + wrap) % wrap).floor() as i64 - 2;
        // A ripple keeps a cell clear of the reflection on either side.
        let clear = (x - 1..=x + n).all(|c| c < 0 || c >= cols || g[r][c as usize] == ' ');
        if clear {
            for c in x.max(0)..(x + n).min(cols) {
                g[r][c as usize] = '-';
                ripples.push((r, c as usize));
            }
        }
    }
    let mut path = vec![];
    if m.lit > 2 {
        let mid = cx as f64 + 0.5;
        for r in SHORE as usize + 1..ROWS {
            // A thin moon lays a narrow, broken path; the sun or a full moon a broad one.
            let w = 0.6 + (r as f64 - SHORE as f64) * (0.15 + m.full * 0.45);
            let mut c = (mid - w - 1.0).floor() as i64;
            while c as f64 <= mid + w + 1.0 {
                if c >= 0 && c < cols {
                    let d = (c as f64 + 0.5 - mid).abs();
                    let ch = if d > w || hash(c * 31 + r as i64, step) < 0.2 + 0.4 * (1.0 - m.full) {
                        ' '
                    } else if d < w * 0.5 && m.full > 0.5 {
                        '='
                    } else {
                        '-'
                    };
                    g[r][c as usize] = ch;
                    ripples.retain(|&(rr, cc)| (rr, cc) != (r, c as usize));
                    if ch != ' ' {
                        path.push((r, c as usize, ch));
                    }
                }
                c += 1;
            }
        }
    }
    m.ripples = ripples;
    m.path = path;
    m.text = g;
    m
}

/// A colour from 0xrrggbb.
fn rgb(n: u32) -> Rgb {
    [((n >> 16) & 255) as f32 / 255.0, ((n >> 8) & 255) as f32 / 255.0, (n & 255) as f32 / 255.0]
}

fn lerp(a: Rgb, b: Rgb, k: f64) -> Rgb {
    let k = k.clamp(0.0, 1.0) as f32;
    [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k]
}

fn scale(a: Rgb, k: f32) -> Rgb {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// What is at a point of the picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Mat {
    // In front first: a cell cut in two shows the first in front of the other.
    Land,
    Cloud,
    Sun,
    MoonLit,
    Maria,
    MoonDark,
    Sky,
}

/// A cloud glyph's body as octant dots, filled from the bottom: `.` low,
/// `-` and `~` higher, the rest whole, the base's ends rounded.
fn puff_mask(ch: char) -> u8 {
    match ch {
        '.' => 0b1100_0000,
        '-' => 0b1111_0000,
        '~' => 0b1111_1100,
        '(' => 0b0111_1110,
        ')' => 0b1011_1101,
        _ => 0b1111_1111,
    }
}

/// The light of the hour: how much day, how much twilight, and on which side.
struct Hour {
    day: f64,
    dusk: f64,
    /// The column of the warm side of the sky.
    side: f64,
}

fn hour(m: &Moment) -> Hour {
    let elev = (PI * m.sun_u).sin();
    let side = if m.day {
        place(m.sun_u).0 as f64
    } else if m.sun_u < 1.5 {
        COLS as f64 - 6.0
    } else {
        5.0
    };
    Hour { day: smooth(-0.06, 0.4, elev), dusk: (-((elev - 0.04) / 0.2).powi(2)).exp(), side }
}

/// The landscape: the range, the clock, and the picture.
pub struct Landscape {
    range: Range,
    hour: Option<f64>,
    phase: Option<f64>,
    ticker: Ticker,
    pixels: Pixels,
}

/// A landscape. Options: `hour` (0 to 24, or null for the local time) and
/// `phase` (the moon's age in lunations, 0 to 1, or null for its real phase now).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let hour = options.json("hour").and_then(|v| v.as_f64());
    let phase = options.json("phase").and_then(|v| v.as_f64());
    Box::new(Landscape { range: Range::new(), hour, phase, ticker: Ticker::default(), pixels: Pixels::new(&META) })
}

/// The 8 dots of cell (c, r), as bit, x and y in columns and rows.
fn subdots(c: usize, r: usize) -> impl Iterator<Item = (u8, f64, f64)> {
    (0..8u8).map(move |d| (d, c as f64 + (f64::from(d % 2) + 0.5) / 2.0, r as f64 + (f64::from(d / 2) + 0.5) / 4.0))
}

/// The scene at one moment, with its colours.
struct View<'a> {
    rg: &'a Range,
    m: &'a Moment,
    hr: Hour,
}

impl View<'_> {
    /// What is at (x, y) above the water.
    fn mat(&self, x: f64, y: f64) -> Mat {
        let m = self.m;
        if self.rg.land(x, y) {
            return Mat::Land;
        }
        if x < 0.0 || y < 0.0 || x >= COLS as f64 || y >= WATER {
            return Mat::Sky;
        }
        let (c, r) = (x.floor() as usize, y.floor() as usize);
        if let Some(p) = m.cloud[r][c] {
            let d = ((y - r as f64) * 4.0) as u8 * 2 + ((x - c as f64) * 2.0) as u8;
            return if puff_mask(p.ch) >> d.min(7) & 1 == 1 { Mat::Cloud } else { Mat::Sky };
        }
        let (bx, by) = (m.cx as f64 + 0.5, m.cy as f64 + 0.5);
        if m.day {
            if (x - bx).hypot(2.0 * (y - by)) < 3.3 {
                return Mat::Sun;
            }
        } else if m.up {
            let (u, v) = ((x - m.cx as f64) / 5.0, (y - by) / 2.5);
            if u * u + v * v <= 1.0 {
                let k = (2.0 * PI * (m.age % 0.5)).cos();
                if (m.age < 0.5) != (u > k * (1.0 - v * v).max(0.0).sqrt()) {
                    return Mat::MoonDark;
                }
                let maria = [(-0.12, -0.4, 0.2), (0.4, 0.02, 0.24), (-0.32, 0.42, 0.18), (0.12, 0.5, 0.12)];
                return if maria.iter().any(|&(mx, my, mr)| (u - mx).hypot(v - my) < mr) { Mat::Maria } else { Mat::MoonLit };
            }
        }
        Mat::Sky
    }

    /// The sky at (x, y): night blue to day blue overhead, paler low down,
    /// warm low on the side of the sunrise or sunset, and a halo round the
    /// sun or a faint one round the moon.
    fn sky(&self, x: f64, y: f64) -> Rgb {
        let (m, hr) = (self.m, &self.hr);
        let zenith = lerp(lerp(rgb(0x060914), rgb(0x3a6cb4), hr.day), rgb(0x2a2a52), 0.5 * hr.dusk);
        let low = lerp(rgb(0x18223f), rgb(0xa8c8e6), hr.day);
        let side = (-((x - hr.side) / 26.0).powi(2)).exp();
        let warm = lerp(rgb(0x9a5a78), rgb(0xf0a060), side);
        let low = lerp(low, warm, hr.dusk * (0.45 + 0.55 * side));
        let mut c = lerp(zenith, low, (y / WATER).powf(1.5));
        let d = (x - (m.cx as f64 + 0.5)).hypot(2.0 * (y - (m.cy as f64 + 0.5)));
        if m.day {
            c = lerp(c, lerp(rgb(0xffe0b0), rgb(0xfff4dc), hr.day), 0.55 * (-d / 5.0).exp() + 0.15 * (-d / 14.0).exp());
        } else if m.up {
            c = lerp(c, rgb(0x8890a8), 0.22 * m.full * (-(d - 4.0).max(0.0) / 5.0).exp());
        }
        c
    }

    /// The land: a silhouette in the hour's light, its crest catching the
    /// sky, darker deeper in.
    fn land(&self, x: f64, y: f64) -> Rgb {
        let hr = &self.hr;
        let base = lerp(lerp(rgb(0x04060c), rgb(0x2c3a46), hr.day), rgb(0x2a2036), 0.6 * hr.dusk);
        let rim = lerp(lerp(rgb(0x121828), rgb(0x5a6e7a), hr.day), rgb(0x6a4660), 0.6 * hr.dusk);
        lerp(rim, base, smooth(0.0, 1.4, self.rg.depth(x, y)))
    }

    /// A cloud: white by day, pink and gold at the ends of it, slate at
    /// night; lit on top, greyer at its base.
    fn cloud(&self, x: f64, y: f64) -> Rgb {
        let (m, hr) = (self.m, &self.hr);
        let (c, r) = ((x.floor().max(0.0) as usize).min(COLS - 1), (y.floor().max(0.0) as usize).min(ROWS - 1));
        let down = m.cloud[r][c].map_or(0.5, |p| p.down);
        let night = lerp(rgb(0x3a4258), rgb(0x262c3e), down);
        let night = lerp(night, rgb(0x5a6278), if m.up { 0.4 * m.full } else { 0.0 });
        let dayc = lerp(rgb(0xf4f4f6), rgb(0xb0bcc8), down);
        let dusk = lerp(rgb(0xf8cc94), rgb(0x80606a), down);
        lerp(lerp(night, dayc, hr.day), dusk, hr.dusk)
    }

    /// The sun's disc, orange low down and near white high up.
    fn sun(&self) -> Rgb {
        lerp(rgb(0xff8a3a), rgb(0xfff2c8), smooth(0.0, 0.6, (PI * self.m.sun_u).sin()))
    }

    /// A material's colour at (x, y).
    fn colour(&self, k: Mat, x: f64, y: f64) -> Rgb {
        match k {
            Mat::Land => self.land(x, y),
            Mat::Cloud => self.cloud(x, y),
            Mat::Sun => self.sun(),
            Mat::MoonLit => rgb(0xecebdf),
            Mat::Maria => rgb(0xa8a8a4),
            Mat::MoonDark => lerp(self.sky(x, y), rgb(0x3a4258), 0.5),
            Mat::Sky => self.sky(x, y),
        }
    }

    /// The light the sun or the moon lays on the water.
    fn shine(&self) -> Rgb {
        if self.m.day {
            self.sun()
        } else {
            rgb(0xecebdf)
        }
    }

    /// The lake at (x, y), showing the range's reflection or the sky's: both
    /// mirrored and darkened, the path of light laid soft under the sun or moon.
    fn water(&self, x: f64, y: f64, range: bool) -> Rgb {
        let (m, ry) = (self.m, 2.0 * WATER - y);
        let f = (y - WATER) / (ROWS as f64 - WATER);
        let deep = lerp(rgb(0x05070f), rgb(0x1c2a42), self.hr.day);
        let mut c = if range { scale(self.land(x, ry), 0.7) } else { lerp(scale(self.sky(x, ry), 0.8), deep, 0.2 + 0.35 * f) };
        if m.lit > 2 {
            let w = 0.6 + (y - SHORE as f64) * (0.15 + m.full * 0.45);
            let glow = (-((x - (m.cx as f64 + 0.5)) / w).powi(2)).exp();
            c = lerp(c, self.shine(), 0.3 * glow * (0.4 + 0.6 * m.full));
        }
        c
    }

    /// An octant cell where two materials meet: the mask of the one in
    /// front, its colour, and the colour behind it.
    fn cut(&self, c: usize, r: usize, at: impl Fn(f64, f64) -> Mat) -> Option<(u8, Mat, Mat)> {
        let mats: Vec<(u8, Mat)> = subdots(c, r).map(|(d, x, y)| (d, at(x, y))).collect();
        let front = mats.iter().map(|&(_, k)| k).min()?;
        if mats.iter().all(|&(_, k)| k == front) {
            return None;
        }
        let mask = mats.iter().filter(|&&(_, k)| k == front).fold(0u8, |a, &(d, _)| a | 1 << d);
        let mut rest: Vec<Mat> = mats.iter().map(|&(_, k)| k).filter(|&k| k != front).collect();
        rest.sort();
        let back = *rest.iter().max_by_key(|&&k| (rest.iter().filter(|&&j| j == k).count(), std::cmp::Reverse(k))).unwrap_or(&Mat::Sky);
        Some((mask, front, back))
    }
}

/// A ray's dots: those along the line out from the sun's middle in the
/// cell's direction, snapped to the nearest eighth of a turn.
fn ray_mask(m: &Moment, r: usize, c: usize) -> u8 {
    let (bx, by) = (m.cx as f64 + 0.5, 2.0 * (m.cy as f64 + 0.5));
    let a = (2.0 * (r as f64 + 0.5) - by).atan2(c as f64 + 0.5 - bx);
    let a = (a / (PI / 4.0)).round() * (PI / 4.0);
    let (ux, uy) = (a.cos(), a.sin());
    let mut mask = subdots(c, r).filter(|&(_, x, y)| ((x - bx) * uy - (2.0 * y - by) * ux).abs() < 0.42).fold(0u8, |k, (d, _, _)| k | 1 << d);
    if mask == 0 {
        mask = 0b0011_1100;
    }
    mask
}

/// Glints on the path of light: dots scattered afresh twice a second, dense
/// for `=`, sparse for `-`.
fn glint_mask(m: &Moment, r: usize, c: usize, ch: char) -> u8 {
    let (p, rows) = if ch == '=' { (0.7, 0b1111) } else { (0.55, 0b0110) };
    let mut mask = 0u8;
    for d in 0..8u8 {
        if rows & (1 << (d / 2)) != 0 && hash(c as i64 * 8 + i64::from(d), r as i64 * 977 + m.step) < p {
            mask |= 1 << d;
        }
    }
    mask | 1 << (2 + (hash(c as i64, r as i64 + m.step * 31) * 4.0) as u8)
}

/// A star's glyph: `*` a bullet, `.` and `'` one braille dot, low and high.
fn star_glyph(ch: char) -> char {
    match ch {
        '*' => '•',
        '\'' => '⠁',
        _ => '⠄',
    }
}

impl Piece for Landscape {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let wall = self.ticker.now(t, env.now);
        let now = wall.local();
        let h = self.hour.unwrap_or(f64::from(now.hour) + f64::from(now.minute) / 60.0);
        let age = self.phase.unwrap_or_else(|| lunation(wall.unix));
        let m = moment(&self.range, t, h, age);
        let v = View { rg: &self.range, m: &m, hr: hour(&m) };
        let rg = &self.range;
        let shore = SHORE as usize;
        if env.mono {
            // One ink: the ridge, the clouds and the reflection as outlines,
            // the land stippled, the sun and the lit moon solid.
            grid.clear();
            // Off the frame's sides a cloud carries on, so it is not outlined there.
            let cloud = |x: f64, y: f64| v.mat(x.clamp(0.25, COLS as f64 - 0.25), y) == Mat::Cloud;
            for r in 0..ROWS {
                for c in 0..COLS {
                    let mut mask = 0u8;
                    for (d, x, y) in subdots(c, r) {
                        let on = if r <= shore {
                            match v.mat(x, y) {
                                Mat::Land => !rg.land(x, y - 0.25) || !rg.land(x - 0.5, y) || !rg.land(x + 0.5, y) || ((c + r) % 2 == 1 && d == 4),
                                Mat::Cloud => !cloud(x, y - 0.25) || !cloud(x, y + 0.25) || !cloud(x - 0.5, y) || !cloud(x + 0.5, y),
                                Mat::Sun | Mat::MoonLit => true,
                                _ => false,
                            }
                        } else {
                            rg.mirrored(x, y) && (!rg.mirrored(x, y + 0.25) || !rg.mirrored(x - 0.5, y) || !rg.mirrored(x + 0.5, y))
                        };
                        if on {
                            mask |= 1 << d;
                        }
                    }
                    if mask != 0 {
                        grid.set(c as i64, r as i64, Octant::glyph(mask), 0);
                    }
                }
            }
        } else {
            for py in 0..2 * ROWS {
                let y = (py as f64 + 0.5) / 2.0;
                for px in 0..COLS {
                    let x = px as f64 + 0.5;
                    let c = if y < WATER { v.colour(v.mat(x, y), x, y) } else { v.water(x, y, rg.mirrored(x, y)) };
                    self.pixels.set(px, py, c);
                }
            }
            self.pixels.draw(grid, env);
            // Cells where two things meet, cut in octants.
            for r in 0..ROWS {
                for c in 0..COLS {
                    let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
                    let (mask, ink, back) = if r <= shore {
                        let Some((mask, front, behind)) = v.cut(c, r, |x, y| v.mat(x, y)) else { continue };
                        (mask, v.colour(front, x, y), v.colour(behind, x, y))
                    } else {
                        let at = |x: f64, y: f64| if rg.mirrored(x, y) { Mat::Land } else { Mat::Sky };
                        let Some((mask, ..)) = v.cut(c, r, at) else { continue };
                        (mask, v.water(x, y, true), v.water(x, y, false))
                    };
                    let (ink, back) = (self.pixels.nearest(ink), self.pixels.nearest(back));
                    grid.set(c as i64, r as i64, Octant::glyph(mask), ink);
                    grid.set_back(c as i64, r as i64, back);
                }
            }
        }
        // The braille and the stars over it all, on the colour behind them.
        let mut put = |grid: &mut Grid, r: usize, c: usize, ch: char, ink: Rgb, back: Rgb| {
            grid.set(c as i64, r as i64, ch, self.pixels.nearest(ink));
            if !env.mono {
                grid.set_back(c as i64, r as i64, self.pixels.nearest(back));
            }
        };
        for &(r, c, ch) in &m.stars {
            let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
            put(grid, r, c, star_glyph(ch), if ch == '*' { rgb(0xfffaea) } else { rgb(0xdfe6ff) }, v.sky(x, y));
        }
        for &(r, c, _) in &m.rays {
            let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
            put(grid, r, c, Braille::glyph(ray_mask(&m, r, c)), v.sun(), v.sky(x, y));
        }
        for &(r, c) in &m.ripples {
            let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
            let water = v.water(x, y, rg.mirrored(x, y));
            put(grid, r, c, Braille::glyph(0b0000_1100), lerp(water, v.sky(x, 1.0), 0.45), water);
        }
        for &(r, c, ch) in &m.path {
            let (x, y) = (c as f64 + 0.5, r as f64 + 0.5);
            let ink = if ch == '=' { lerp(v.shine(), rgb(0xfffaea), 0.4) } else { v.shine() };
            put(grid, r, c, Braille::glyph(glint_mask(&m, r, c, ch)), ink, v.water(x, y, rg.mirrored(x, y)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(m: &Moment) -> String {
        m.text.iter().map(|row| row.iter().collect::<String>()).collect::<Vec<_>>().join("\n")
    }

    #[test]
    fn the_moon_s_age_matches_ascii_rest() {
        // landscape.ts's lunation, from node, for 2026-10-08 10:09:30 UTC and 2000-01-06 18:14 UTC.
        assert!((lunation(crate::clock::Wall::FIXED.unix) - LUNATION_FIXED).abs() < 1e-9);
        let new = lunation(947_182_440.0);
        assert!(new.min(1.0 - new) < 1e-3);
    }

    #[test]
    fn matches_ascii_rests_frames() {
        // landscape.ts's frames, from node, at the given hour, moon age and play time.
        let rg = Range::new();
        for (h, age, t, want) in FRAMES {
            assert_eq!(text(&moment(&rg, t, h, age)), want.trim_matches('\n'), "hour {h}, age {age}, t {t}");
        }
    }

    const FRAMES: [(f64, f64, f64, &str); 8] = [
        (
            9.0,
            0.3,
            0.0,
            r#"
                                         .-~~-.                 
                                    .-~~'      '-.  .-.         
               \  |  /            .'              '~   '.       
                .---.            (______________________)       
             --(     )--                                        
                '---'                                           
               /  |  \                     __                   
           _/\                            / .\                  
         _/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            -=-         \                      /\     /    
 --  -\          -=-          \     /--\          /-  \   /     
--     -\       -===-          \  /-    \       /-     \/-      
         -\    --== --      --  \/       \    /-                
           -\  -- ==--                --- \  /                  
              --=====--                    --          ---      
"#,
        ),
        (
            22.0,
            0.4,
            3.0,
            r#"
       .         .              ''-.         *           .-~~-. 
                        '     ::.:::\               .-~~'      '
-.                           :::::..:|       .    .'            
  '.         '       .        :.::::/            (______________
____)                           ..-'                            
                                                                
                                           __                   
           _/\                            / .\                  
         _/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            /           \  -=                  /\     /    
--   -\         /               -=   --\          /-  \   /     
--     -\      /               -===-    \       /-     \/-      
         -\   /            --   = =      \    /-                
           -\/                --= =-  --- \  /                  
                              --===-       --         ---       
"#,
        ),
        (
            3.0,
            0.15,
            3.0,
            r#"
       .          .-~~-.          .          *                  
             .-~~'      '-.  .-.       '                        
   .       .'              '~   '.           .-~~-.     *       
          (______________________)         .'      '.      .    
                                     .    (__________)       *  
    .                                                           
                                           __                   
           _/\                            / .\                  
         _/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            /           \                      /\     /    
--   -\         /             \     /--\          /-  \   /     
--     -\      /               \  /-    \       /-     \/-      
         -\   /            --   \/       \    /-                
           -\/                        --- \  /                  
                                           --         ---       
"#,
        ),
        (
            17.5,
            0.5,
            0.0,
            r#"
                                                                
  .-.                                                           
'~   '.           .-~~-.                                        
______)         .'      '.                                      
               (__________)                                     
                                                                
                                           __                   
           _/\                            / .\       \  |  /    
         _/. .\                 /\       / . .\_      .---.     
       _/ . . .\               /. \_    / . . . \_   ( /\_ )--  
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            /           \                      /\ -=- /    
 --  -\         /             \     /--\          /-   -=-      
--     -\      /               \  /-    \       /-    -===-     
         -\   /             --  \/       \    /-     --== --    
           -\/                        --- \  /       --===-     
                                           --       --=== = -   
"#,
        ),
        (
            20.0,
            0.6,
            11.5,
            r#"
       .         .      .-~~-.               *                  
                   .-~~'      '-.  .-.                          
                 .'              '~   '.     .     .-~~-.       
             '  (______________________)         .'      '.     
                                                (__________) *  
                                                                
                                           __                   
           _/\                            / .\                  
         _/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            /           \                      /\     /    
     -\         /             \     /--\          /-  \   /     
   --  -\      /               \  /-    \       /-     \/-      
         -\   /          --     \/       \    /-                
           -\/                            \  /                  
                                           --       ---         
"#,
        ),
        (
            6.3,
            0.9,
            40.0,
            r#"
                                             .-~~-.             
                                        .-~~'      '-.  .-.     
                                      .'              '~   '.   
                                     (______________________)   
                                                                
                                                                
                                           __                   
           _/\                            / .\                  
   \     _/. .\                 /\       / . .\_                
    .--_/ . . .\               /. \_    / . . . \_     /\_      
   ( _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
      =-         /           \                      /\     /    
     - -        /             \     /--\          /-  \   /     
     ===-   -- /               \  /-    \       /-     \/-      
   - === -    /   --            \/       \    /-                
     == --  \/                            \  /    ---           
  -- == =--                                --                   
"#,
        ),
        (
            1.0,
            0.75,
            7.0,
            r#"
       .                          .-~~-.                        
                        '    .-~~'      '-.  .-.                
   .                       .'              '~   '.  .   *    .-~
                     .    (______________________)         .'   
                                                          (_____
      .-'                                                       
     /::.                                  __                   
    |::::  _/\                            / .\                  
     \:.:_/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \    =       /           \                      /\     /    
-    -\ -       /             \     /--\          /-  \   /     
  --    -=     /               \  /-    \       /-     \/-      
       -===   /           --    \/       \    /-                
        ==   /                            \  /                  
       -=  -                               --        ---        
"#,
        ),
        (
            13.0,
            0.05,
            90.0,
            r#"
       .-~~-.                    \  |  /                        
  .-~~'      '-.  .-.             .---.                         
.'              '~   '.          (.-~~-.                        
______________________)         .'      '.                      
                               (__________)                     
                                                                
                                           __                   
           _/\                            / .\                  
         _/. .\                 /\       / . .\_                
       _/ . . .\               /. \_    / . . . \_     /\_      
     _/. . . . .\             /. . .\__/ . . . . .\_  /. .\     
____/ . . . . . .\___________/. . . . . . . . . . . \/. . .\____
    \            /           \     -=-              /\     /    
     -\         /             \    -=-        --  /-  \   /     
       -\      /           --  \  -== - \       /-     \/-      
     --  -\   /                   -===-  \    /-                
           -\/                   - ===--  \  /                  
                                 -=== =--  --                   
"#,
        ),
    ];
    const LUNATION_FIXED: f64 = 0.8997091975086278;
}
