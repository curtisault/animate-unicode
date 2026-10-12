//! glitch: a word set in a blocky pixel face holds clean, then breaks up in
//! short digital bursts: slices slide sideways and lose registration, cells
//! corrupt. After ascii.rest's glitch.ts (MIT, @bas3line).
//!
//! The face, the bursts and every random draw are ascii.rest's, so each frame
//! breaks up the same way. What it adds is the signal's colour: the word is
//! white, the copy a slid slice leaves behind is cyan, smears drag out in
//! magenta, a tear line flashes yellow, and corrupt cells come up in any of
//! the three, so a burst splits the picture into its channels.

use super::marquee::font;
use crate::util::Mulberry32;
use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// Glitch's meta.
pub static META: Meta = Meta {
    name: "glitch",
    slug: "glitch",
    category: Category::Type,
    note: "a pixel word tearing into sliding slices, split into cyan and magenta",
    cols: 64,
    rows: 13,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"text": "noise"}"#),
    // The word, the copy left behind, smears, a tear, static; corrupt cells in the first three after it.
    palette: Some(&["#f4f6fa", "#3ae0f0", "#f03ac8", "#f0e04a", "#5a5e6a", "#5af0ff", "#ff5ad8", "#fff07a"]),
    ground: Some("#0a0a10"),
    ..Meta::DEFAULT
};

const INK: u8 = 0;
const COPY: u8 = 1;
const SMEAR: u8 = 2;
const TEAR: u8 = 3;
const STATIC: u8 = 4;
const CORRUPT: u8 = 5;
/// One cycle: clean holds of at least 1.5 s broken by bursts of (start, end,
/// peak strength), and glitch states a second.
const PERIOD: f64 = 7.0;
const STEPS: f64 = 15.0;
const BURSTS: [(f64, f64, f64); 3] = [(0.8, 1.2, 0.55), (2.75, 3.3, 1.0), (4.85, 5.3, 0.75)];
const NOISE: [char; 13] = ['▓', '▒', '░', '▚', '▞', '▙', '▟', '▛', '▜', '#', '%', '=', '+'];

/// The text as seven rows of pixels, each glyph trimmed to its inked columns.
fn bitmap(text: &str) -> [Vec<bool>; 7] {
    let mut px: [Vec<bool>; 7] = Default::default();
    for ch in text.to_uppercase().chars() {
        let g = font(ch).or_else(|| font('?')).unwrap_or([0; 7]);
        let (mut lo, mut hi) = (4, 0);
        for x in 0..5 {
            if g.iter().any(|&b| b & (16 >> x) != 0) {
                (lo, hi) = (lo.min(x), hi.max(x));
            }
        }
        if ch == ' ' {
            (lo, hi) = (0, 2);
        }
        if !px[0].is_empty() {
            px.iter_mut().for_each(|row| row.push(false));
        }
        for (y, row) in px.iter_mut().enumerate() {
            row.extend((lo..=hi).map(|x| g[y] & (16 >> x) != 0));
        }
    }
    px
}

/// How hard the picture is breaking at cycle time u, 0 when clean.
fn strength(u: f64) -> f64 {
    BURSTS.iter().find(|&&(s, e, _)| u >= s && u < e).map_or(0.0, |&(s, e, peak)| peak * (0.35 + 0.65 * (PI * (u - s) / (e - s)).sin()))
}

/// The word: its clean pixels, its first row, and the picture being broken.
pub struct Glitch {
    clean: Vec<bool>,
    y0: i64,
    out: Vec<(char, u8)>,
}

impl Glitch {
    fn ink(&self, r: i64, c: i64) -> bool {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        r >= 0 && r < rows && c >= 0 && c < cols && self.clean[(r * cols + c) as usize]
    }
}

/// Glitch; reads `text`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let px = bitmap(options.str("text", "noise"));
    let w = px[0].len() as i64;
    let sx = if w * 2 <= cols - 8 { 2 } else { 1 }; // pixels two cells wide when the word fits
    let x0 = ((cols - w * sx) >> 1).max(0);
    let y0 = (rows - 7) >> 1;
    let mut clean = vec![false; (cols * rows) as usize];
    for (y, row) in px.iter().enumerate() {
        for c in x0..cols {
            let x = (c - x0) / sx;
            if x < w && row[x as usize] {
                clean[((y0 + y as i64) * cols + c) as usize] = true;
            }
        }
    }
    Box::new(Glitch { clean, y0, out: vec![(' ', INK); (cols * rows) as usize] })
}

impl Piece for Glitch {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        let y0 = self.y0;
        let u = t.rem_euclid(PERIOD);
        let step = (u * STEPS + 1e-6).floor() as i64;
        let mut rng = Mulberry32::new((step * 7919 + 17) as u32);
        let mut rand = || rng.next_f64();
        let mut s = strength(step as f64 / STEPS);
        if s > 0.0 && rand() < 0.12 {
            s = 0.0; // a clean flicker inside a burst
        }
        grid.clear();
        if s == 0.0 {
            for (k, &on) in self.clean.iter().enumerate() {
                if on {
                    grid.set(k as i64 % cols, k as i64 / cols, '█', INK);
                }
            }
            return;
        }
        // Corrupt cells take a channel by where they are, without a draw of their own.
        let channel = |k: usize| CORRUPT + ((k as u32).wrapping_mul(2_654_435_761) ^ (step as u32).wrapping_mul(40_503)) as u8 % 3;
        let mut out = std::mem::take(&mut self.out);
        out.fill((' ', INK));
        let at = |r: i64, c: i64| (r * cols + c) as usize;

        // Bands of rows slide sideways.
        let mut shift = vec![0i64; rows as usize];
        let bands = 1 + (rand() * (1.0 + 2.0 * s)).floor() as i64;
        for _ in 0..bands {
            let top = y0 - 1 + (rand() * 8.0).floor() as i64;
            let h = 1 + (rand() * 3.0).floor() as i64;
            let dx = (if rand() < 0.5 { -1 } else { 1 }) * (3 + (rand() * (2.0 + 8.0 * s)).floor() as i64);
            for r in top..(top + h).min(rows) {
                shift[r as usize] += dx;
            }
        }
        // The slid slices lose registration: a second channel stays behind
        // where the slice was.
        let split = rand() < 0.4 + 0.5 * s;
        let sub = 0.02 + 0.08 * s * rand();
        let holes = 0.01 + 0.04 * s;
        for r in 0..rows {
            for c in 0..cols {
                if self.ink(r, c - shift[r as usize]) {
                    let v = rand();
                    let k = at(r, c);
                    out[k] = if v < sub {
                        (NOISE[(rand() * NOISE.len() as f64).floor() as usize], channel(k))
                    } else if v < sub + holes {
                        (' ', INK)
                    } else {
                        ('█', INK)
                    };
                }
            }
        }
        // It only shows on slices that slid well clear, where it clears the
        // main ink by a column, so it reads as a copy left behind rather than
        // a shadow.
        if split {
            let main: Vec<char> = out.iter().map(|&(ch, _)| ch).collect();
            let bare = |k: usize, c: i64| main[k] == ' ' && (c == 0 || main[k - 1] == ' ') && (c == cols - 1 || main[k + 1] == ' ');
            for r in (0..rows).filter(|&r| shift[r as usize].abs() >= 5) {
                for c in 0..cols {
                    let k = at(r, c);
                    if bare(k, c) && self.ink(r, c) {
                        out[k] = ('▒', COPY);
                    }
                }
            }
        }
        // Smears: a row's pixels drag out to one side and fade.
        let smears = if rand() < 0.45 * s { 1 + (rand() * 2.0).floor() as i64 } else { 0 };
        for _ in 0..smears {
            let r = y0 + (rand() * 7.0).floor() as i64;
            let mut c = (rand() * cols as f64).floor() as i64;
            while c < cols - 1 && out[at(r, c)].0 != '█' {
                c += 1;
            }
            let len = 4 + (rand() * 14.0 * s).floor() as i64;
            for j in (1..=len).take_while(|j| c + j < cols) {
                out[at(r, c + j)] = (
                    if j > len - 2 {
                        '░'
                    } else if j > len - 5 {
                        '▒'
                    } else {
                        '▓'
                    },
                    SMEAR,
                );
            }
        }
        // Macroblocks: a patch of the picture copied in from somewhere else.
        let blocks = if rand() < 0.5 * s { 1 + (rand() * 2.0).floor() as i64 } else { 0 };
        for _ in 0..blocks {
            let (bw, bh) = (3 + (rand() * 10.0).floor() as i64, 1 + (rand() * 2.0).floor() as i64);
            let (br, bc) = (y0 + (rand() * 7.0).floor() as i64, (rand() * (cols - bw) as f64).floor() as i64);
            let (dr, dc) = ((rand() * 5.0).floor() as i64 - 2, (rand() * 21.0).floor() as i64 - 10);
            for r in br..(br + bh).min(rows) {
                for c in bc..bc + bw {
                    out[at(r, c)] = if self.ink(r + dr, c + dc) {
                        (if rand() < 0.3 { '▓' } else { '█' }, INK)
                    } else if rand() < 0.15 * s {
                        ('░', STATIC)
                    } else {
                        (' ', INK)
                    };
                }
            }
        }
        // A tear line through the word, or just above or below it.
        if rand() < 0.5 * s {
            let r = y0 - 1 + (rand() * 9.0).floor() as i64;
            let a = (rand() * cols as f64 * 0.6).floor() as i64;
            let len = 6 + (rand() * cols as f64 * 0.5).floor() as i64;
            let g = ['▀', '▄', '─', '═'][(rand() * 4.0).floor() as usize];
            for c in a..cols.min(a + len) {
                let k = at(r, c);
                if out[k].0 == ' ' || rand() < 0.5 {
                    out[k] = (g, TEAR);
                }
            }
        }
        // At full strength the whole picture sometimes jumps a row.
        let jump = if s > 0.7 && rand() < 0.25 {
            if rand() < 0.5 {
                1
            } else {
                -1
            }
        } else {
            0
        };
        for r in 0..rows {
            let from = r - jump;
            if (0..rows).contains(&from) {
                for c in 0..cols {
                    let (ch, color) = out[at(from, c)];
                    if ch != ' ' {
                        grid.set(c, r, ch, color);
                    }
                }
            }
        }
        self.out = out;
    }
}
