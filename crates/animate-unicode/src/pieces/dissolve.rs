//! dissolve: a word in smooth stroked capitals crumbles letter by letter, its
//! dust sinking out of each letter, while the next word's dust gathers onto
//! its strokes. After ascii.rest's dissolve.ts (MIT, @bas3line).
//!
//! The strokes, the layout and the timing are ascii.rest's, but the word is
//! drawn in braille, eight dots to a cell, so each dot is a grain of sand: a
//! stroke wears away dot by dot, the grains that let go fall out of it one
//! dot each, and the next word's grains fly in to settle on its strokes. Each
//! word has its own colour, and its grains fade as they fall.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// Dissolve's meta.
pub static META: Meta = Meta {
    name: "dissolve",
    slug: "dissolve",
    category: Category::Type,
    note: "words crumbling to braille dust that gathers into the next word",
    cols: 64,
    rows: 13,
    fps: 20,
    charset: Charset::Extended,
    options: Some(r#"{"words": ["hello", "hola", "ciao", "salut"]}"#),
    // A word's colour, then its dust: rose, amber, teal, violet, word by word.
    palette: Some(&["#f06a8a", "#8a3a52", "#f4b04a", "#8a6428", "#4ad0c0", "#267a70", "#a88af0", "#584a8a"]),
    ground: Some("#0c0a10"),
    ..Meta::DEFAULT
};

/// Seconds each word rests, and the change to the next.
const HOLD: f64 = 1.6;
const TRANS: f64 = 2.6;
/// Letter by letter, and grain by grain within one.
const STAGGER: f64 = 0.2;
const JITTER: f64 = 0.22;
/// A grain wearing away, one filling in, the new word's lag.
const ERODE: f64 = 0.3;
const FADE: f64 = 0.5;
const LATER: f64 = 1.0;
/// Stroke radius, a little more on slanted strokes so they hold their weight.
const R: f64 = 0.5;
const RD: f64 = 0.6;
/// Columns from one letter's last stroke to the next one's first.
const GAP: f64 = 4.0;

/// Capitals as polylines in a 4 by 6 box, y down.
fn strokes(ch: char) -> Option<&'static [&'static [f64]]> {
    Some(match ch {
        'A' => &[&[0.0, 6.0, 1.6, 0.0, 2.4, 0.0, 4.0, 6.0], &[0.7, 4.0, 3.3, 4.0]],
        'B' => &[&[0.0, 0.0, 0.0, 6.0, 3.0, 6.0, 4.0, 5.0, 4.0, 4.0, 3.0, 3.0, 0.0, 3.0], &[0.0, 0.0, 2.5, 0.0, 3.5, 1.0, 3.5, 2.0, 2.5, 3.0]],
        'C' => &[&[4.0, 1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 5.0, 1.0, 6.0, 3.0, 6.0, 4.0, 5.0]],
        'D' => &[&[0.0, 0.0, 0.0, 6.0, 2.5, 6.0, 4.0, 4.5, 4.0, 1.5, 2.5, 0.0, 0.0, 0.0]],
        'E' => &[&[4.0, 0.0, 0.0, 0.0, 0.0, 6.0, 4.0, 6.0], &[0.0, 3.0, 3.0, 3.0]],
        'F' => &[&[4.0, 0.0, 0.0, 0.0, 0.0, 6.0], &[0.0, 3.0, 3.0, 3.0]],
        'G' => &[&[4.0, 1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 5.0, 1.0, 6.0, 3.0, 6.0, 4.0, 5.0, 4.0, 3.0, 2.0, 3.0]],
        'H' => &[&[0.0, 0.0, 0.0, 6.0], &[4.0, 0.0, 4.0, 6.0], &[0.0, 3.0, 4.0, 3.0]],
        'I' => &[&[1.0, 0.0, 3.0, 0.0], &[2.0, 0.0, 2.0, 6.0], &[1.0, 6.0, 3.0, 6.0]],
        'J' => &[&[1.5, 0.0, 4.0, 0.0], &[3.0, 0.0, 3.0, 5.0, 2.0, 6.0, 1.0, 6.0, 0.0, 5.0]],
        'K' => &[&[0.0, 0.0, 0.0, 6.0], &[4.0, 0.0, 0.0, 3.6], &[1.5, 2.25, 4.0, 6.0]],
        'L' => &[&[0.0, 0.0, 0.0, 6.0, 4.0, 6.0]],
        'M' => &[&[0.0, 6.0, 0.0, 0.0, 2.0, 3.5, 4.0, 0.0, 4.0, 6.0]],
        'N' => &[&[0.0, 6.0, 0.0, 0.0, 4.0, 6.0, 4.0, 0.0]],
        'O' => &[&[1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 5.0, 3.0, 6.0, 1.0, 6.0, 0.0, 5.0, 0.0, 1.0, 1.0, 0.0]],
        'P' => &[&[0.0, 6.0, 0.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 3.0, 3.0, 0.0, 3.0]],
        'Q' => &[&[1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 5.0, 3.0, 6.0, 1.0, 6.0, 0.0, 5.0, 0.0, 1.0, 1.0, 0.0], &[2.4, 4.4, 4.0, 6.2]],
        'R' => &[&[0.0, 6.0, 0.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 3.0, 3.0, 0.0, 3.0], &[2.2, 3.0, 4.0, 6.0]],
        'S' => &[&[4.0, 1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 5.0, 3.0, 6.0, 1.0, 6.0, 0.0, 5.0]],
        'T' => &[&[0.0, 0.0, 4.0, 0.0], &[2.0, 0.0, 2.0, 6.0]],
        'U' => &[&[0.0, 0.0, 0.0, 5.0, 1.0, 6.0, 3.0, 6.0, 4.0, 5.0, 4.0, 0.0]],
        'V' => &[&[0.0, 0.0, 2.0, 6.0, 4.0, 0.0]],
        'W' => &[&[0.0, 0.0, 1.0, 6.0, 2.0, 2.0, 3.0, 6.0, 4.0, 0.0]],
        'X' => &[&[0.0, 0.0, 4.0, 6.0], &[4.0, 0.0, 0.0, 6.0]],
        'Y' => &[&[0.0, 0.0, 2.0, 3.0], &[4.0, 0.0, 2.0, 3.0, 2.0, 6.0]],
        'Z' => &[&[0.0, 0.0, 4.0, 0.0, 0.0, 6.0, 4.0, 6.0]],
        '0' => &[&[1.0, 0.0, 2.0, 0.0, 3.0, 1.0, 3.0, 5.0, 2.0, 6.0, 1.0, 6.0, 0.0, 5.0, 0.0, 1.0, 1.0, 0.0]],
        '1' => &[&[0.5, 1.0, 1.5, 0.0, 1.5, 6.0], &[0.5, 6.0, 2.5, 6.0]],
        '2' => &[&[0.0, 1.0, 1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 0.0, 6.0, 4.0, 6.0]],
        '3' => &[&[0.0, 1.0, 1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 3.0, 3.0, 1.5, 3.0], &[3.0, 3.0, 4.0, 4.0, 4.0, 5.0, 3.0, 6.0, 1.0, 6.0, 0.0, 5.0]],
        '4' => &[&[3.0, 6.0, 3.0, 0.0, 0.0, 4.0, 4.0, 4.0]],
        '5' => &[&[4.0, 0.0, 0.0, 0.0, 0.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 5.0, 3.0, 6.0, 0.0, 6.0]],
        '6' => &[&[3.5, 0.0, 1.5, 0.0, 0.0, 1.5, 0.0, 5.0, 1.0, 6.0, 3.0, 6.0, 4.0, 5.0, 4.0, 4.0, 3.0, 3.0, 1.0, 3.0, 0.0, 4.0]],
        '7' => &[&[0.0, 0.0, 4.0, 0.0, 1.5, 6.0]],
        '8' => &[&[
            1.0, 3.0, 0.0, 2.0, 0.0, 1.0, 1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 3.0, 3.0, 1.0, 3.0, 0.0, 4.0, 0.0, 5.0, 1.0, 6.0, 3.0, 6.0, 4.0, 5.0, 4.0,
            4.0, 3.0, 3.0,
        ]],
        '9' => &[&[0.5, 6.0, 2.5, 6.0, 4.0, 4.5, 4.0, 1.0, 3.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 2.0, 1.0, 3.0, 4.0, 3.0]],
        '.' => &[&[2.0, 6.0, 2.0, 6.0]],
        ',' => &[&[2.0, 5.6, 1.5, 7.0]],
        '!' => &[&[2.0, 0.0, 2.0, 4.0], &[2.0, 6.0, 2.0, 6.0]],
        '?' => &[&[0.0, 1.0, 1.0, 0.0, 3.0, 0.0, 4.0, 1.0, 4.0, 2.0, 2.0, 3.5, 2.0, 4.2], &[2.0, 6.0, 2.0, 6.0]],
        '\'' => &[&[2.0, 0.0, 2.0, 1.5]],
        '-' => &[&[1.0, 3.0, 3.0, 3.0]],
        _ => return None,
    })
}

fn glyph(ch: char) -> &'static [&'static [f64]] {
    strokes(ch).unwrap_or(if ch == ' ' { &[] } else { strokes('?').unwrap_or(&[]) })
}

/// A glyph's left and right edges.
fn span(g: &[&[f64]]) -> (f64, f64) {
    let xs = g.iter().flat_map(|p| p.iter().step_by(2));
    xs.fold(None, |m: Option<(f64, f64)>, &x| Some(m.map_or((x, x), |(lo, hi)| (lo.min(x), hi.max(x))))).unwrap_or((0.0, 2.0))
}

fn hash(a: i64, b: i64) -> f64 {
    let mut h = (a as u32).wrapping_mul(374_761_393).wrapping_add((b as u32).wrapping_mul(668_265_263));
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(x: f64) -> f64 {
    x.clamp(0.0, 1.0).powi(2) * (3.0 - 2.0 * x.clamp(0.0, 1.0))
}

/// Distance from (x, y) to the segment a-b, less the stroke's radius.
fn seg(x: f64, y: f64, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let l = dx * dx + dy * dy;
    let k = if l > 0.0 { (((x - a.0) * dx + (y - a.1) * dy) / l).clamp(0.0, 1.0) } else { 0.0 };
    (x - a.0 - k * dx).hypot(y - a.1 - k * dy) - if dx != 0.0 && dy != 0.0 { RD } else { R }
}

/// A word baked: which dots are inked, and which letter each column belongs to.
struct Word {
    cover: Vec<bool>,
    slot: Vec<u8>,
}

/// The words, the dots, and the dust falling or gathering this frame.
pub struct Dissolve {
    words: Vec<Word>,
    dots: ColorDots,
    dust: Vec<(f32, u8)>,
}

/// Dissolve; reads `words`, a list or one string split at commas and spaces.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let given: Vec<String> = match options.json("words") {
        Some(serde_json::Value::Array(a)) => a.iter().map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_string)).collect(),
        Some(serde_json::Value::String(s)) => s.split([',', ' ', '\t', '\n']).map(str::to_string).collect(),
        _ => vec!["hello".into(), "hola".into(), "ciao".into(), "salut".into()],
    };
    let mut list: Vec<Vec<char>> = given.iter().filter(|w| !w.is_empty()).map(|w| w.to_uppercase().chars().collect()).collect();
    if list.is_empty() {
        list.push("HELLO".chars().collect());
    }
    // One unit is a row tall and two columns wide, smaller if the longest word needs it.
    let gap_at = |u: f64| (GAP * u).round().max(2.0);
    let width = |ch: char, u: f64| {
        let (lo, hi) = span(glyph(ch));
        (2.0 * u * (hi - lo)).round()
    };
    let wide = |w: &[char], u: f64| w.iter().map(|&ch| width(ch, u) + gap_at(u)).sum::<f64>() - gap_at(u) + 4.0 * u * R;
    let mut unit = (1.0f64).min((rows as f64 - 2.0) / (6.0 + 2.0 * R));
    while unit > 0.3 && list.iter().map(|w| wide(w, unit)).fold(0.0, f64::max) > (cols - 4) as f64 {
        unit -= 0.02;
    }
    let gap = gap_at(unit);
    // Each dot is half a column wide and a quarter row tall; it is inked
    // where the stroke covers its middle. The
    // pen advances in whole columns, so every gap between letters is the same.
    let (dw, dh) = (cols * 2, rows * 4);
    let words = list
        .iter()
        .map(|w| {
            let mut cover = vec![false; (dw * dh) as usize];
            let y0 = rows as f64 / 2.0 - 3.0 * unit;
            let mut pen = ((cols as f64 - wide(w, unit)) / 2.0 + 2.0 * unit * R).round();
            let mut centres = vec![];
            for &ch in w {
                let g = glyph(ch);
                let (lo, _) = span(g);
                let wd = width(ch, unit);
                centres.push(pen + wd / 2.0);
                let (c0, c1) = ((pen as i64 - 3).max(0), (pen as i64 + wd as i64 + 3).min(cols));
                for y in 0..dh {
                    for x in c0 * 2..c1 * 2 {
                        let (gx, gy) = (lo + ((x as f64 + 0.5) / 2.0 - pen) / (2.0 * unit), ((y as f64 + 0.5) / 4.0 - y0) / unit);
                        let mut d = 9f64;
                        for p in g {
                            for k in (0..p.len().saturating_sub(3)).step_by(2) {
                                d = d.min(seg(gx, gy, (p[k], p[k + 1]), (p[k + 2], p[k + 3])));
                            }
                        }
                        if d <= 0.0 {
                            cover[(y * dw + x) as usize] = true;
                        }
                    }
                }
                pen += wd + gap;
            }
            let slot = (0..cols)
                .map(|c| {
                    let mut best = 0;
                    for (i, &x) in centres.iter().enumerate() {
                        if (x - c as f64).abs() < (centres[best] - c as f64).abs() {
                            best = i;
                        }
                    }
                    best as u8
                })
                .collect();
            Word { cover, slot }
        })
        .collect();
    Box::new(Dissolve { words, dots: ColorDots::new(META.cols, META.rows), dust: vec![(0.0, 0); (dw * dh) as usize] })
}

impl Piece for Dissolve {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (dw, dh) = (META.cols as i64 * 2, META.rows as i64 * 4);
        let count = self.words.len();
        let period = count as f64 * (HOLD + TRANS);
        let u = (t + 1.0).rem_euclid(period);
        let n = ((u / (HOLD + TRANS)).floor() as usize).min(count - 1);
        let tau = u - n as f64 * (HOLD + TRANS) - HOLD;
        let (a_word, b_word) = (&self.words[n], &self.words[(n + 1) % count]);
        let (a_ink, b_ink) = (2 * (n % 4) as u8, 2 * ((n + 1) % 4) as u8);
        let ni = n as i64;
        self.dust.fill((0.0, 0));
        self.dots.clear();
        let dust = &mut self.dust;
        // A grain at (x, y) in dots, keeping the brightest where two land.
        let mut splat = |x: f64, y: f64, d: f64, color: u8| {
            let (x, y) = (x.round() as i64, y.round() as i64);
            if (0..dw).contains(&x) && (0..dh).contains(&y) {
                let k = (y * dw + x) as usize;
                if f64::from(dust[k].0) < d {
                    dust[k] = (d as f32, color);
                }
            }
        };
        for y in 0..dh {
            for x in 0..dw {
                let k = (y * dw + x) as usize;
                let ki = k as i64;
                let c = (x / 2) as usize;
                let (a, b) = (a_word.cover[k], b_word.cover[k]);
                let (mut v, mut color) = (if a { 1.0 } else { 0.0 }, a_ink);
                if tau > 0.0 {
                    v = 0.0;
                    // The old dot wears thin, then about half of them let go as a
                    // grain that falls out of the letter, drifting a little right,
                    // and fades.
                    if a {
                        let go = f64::from(a_word.slot[c]) * STAGGER + hash(ki, ni) * JITTER;
                        let e = (tau - go) / ERODE;
                        if hash(ki, ni + 7) > 0.5 {
                            v = 1.0 - ease(e);
                        } else if e < 1.0 {
                            v = 1.0 - 0.5 * e.max(0.0);
                        } else {
                            let (age, life) = (tau - go - ERODE, 0.5 + 0.35 * hash(ki, ni + 57));
                            if age < life {
                                let (vx, vy) = (0.6 + 1.4 * hash(ki, ni + 31), 0.3 + 0.6 * hash(ki, ni + 83));
                                // Cells a second, so twice that across in dots and four times down.
                                splat(x as f64 + 2.0 * vx * age, y as f64 + 4.0 * (vy * age + 4.0 * age * age), 1.0 - 0.7 * age / life, a_ink);
                            }
                        }
                    }
                    // The new dot's grain, for about half of them, gathers in
                    // toward its stroke from a cell or two away, and the stroke
                    // fills in around it.
                    if b {
                        let land = LATER + f64::from(b_word.slot[c]) * STAGGER + hash(ki, ni + 977) * JITTER;
                        if tau >= land {
                            let f = ease((tau - land) / FADE);
                            let w = 0.4 + 0.6 * f;
                            if w > v {
                                (v, color) = (w, b_ink);
                            }
                        } else if hash(ki, ni + 9) < 0.55 {
                            let life = 0.45 + 0.3 * hash(ki, ni + 211);
                            let f = (land - tau) / life;
                            if f < 1.0 {
                                #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the grains start where theirs do
                                let th = 6.283 * hash(ki, ni + 401);
                                let (q, d) = (f * f, 0.8 + 1.2 * hash(ki, ni + 613));
                                splat(x as f64 + 2.0 * 1.8 * d * th.cos() * q, y as f64 + 4.0 * d * th.sin() * q, 0.35 + 0.65 * (1.0 - f), b_ink);
                            }
                        }
                    }
                }
                if dither(x as usize, y as usize, v) {
                    self.dots.set(x, y, color);
                }
            }
        }
        // A grain is a whole dot; one fading out takes its word's dust colour.
        for (k, &(d, color)) in self.dust.iter().enumerate() {
            if d > 0.0 && self.dots.get(k as i64 % dw, k as i64 / dw).is_none() {
                self.dots.set(k as i64 % dw, k as i64 / dw, if d < 0.55 { color + 1 } else { color });
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
