//! heartbeat: an ECG monitor. A PQRST trace sweeps left to right over a
//! dotted grid and its last pass, erasing a few columns ahead, beside a rate
//! readout. After ascii.rest's heartbeat.ts (MIT, @bas3line).
//!
//! The beats and the rate are ascii.rest's. In braille the trace is a true
//! waveform, two samples a column and four dots a row, so the P and T waves
//! are rounded and the R wave a sharp spike rather than a box-drawn corner;
//! it glows green like a phosphor screen, brightest at the pen and fading
//! back along the sweep, and the heart beside the rate is red.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// The heartbeat's meta.
pub static META: Meta = Meta {
    name: "heartbeat",
    slug: "heartbeat",
    category: Category::Data,
    note: "an ecg in braille, green phosphor fading behind the pen, bpm beside",
    cols: 72,
    rows: 15,
    fps: 25,
    charset: Charset::Extended,
    options: Some(r#"{"bpm": 72}"#),
    // The trace, oldest to the pen; the grid; text; the heart.
    palette: Some(&["#1e5a2e", "#2a8040", "#3aa852", "#5ad070", "#a8f4b8", "#2a3a30", "#8a9a90", "#f04a5a"]),
    ground: Some("#060a08"),
    ..Meta::DEFAULT
};

const GLOW: Ramp = Ramp::new(0, 5);
const GRID: u8 = 5;
const TEXT: u8 = 6;
const HEART: u8 = 7;
/// Columns per second, like 25 mm/s paper; trace width; erased columns
/// ahead of the pen; the baseline row; the first trace column.
const SPEED: f64 = 25.0;
const W: i64 = 46;
const GAP: i64 = 4;
const BASE: f64 = 10.0;
const X0: i64 = 2;
/// The R wave's column in a beat, for the rate.
const R_AT: f64 = 7.0;
/// Digits for the readout, 3×5, each pixel two columns wide and a row tall.
const FONT: [&str; 10] = [
    "###,#.#,#.#,#.#,###",
    "##.,.#.,.#.,.#.,###",
    "###,..#,###,#..,###",
    "###,..#,###,..#,###",
    "#.#,#.#,###,..#,..#",
    "###,#..,###,..#,###",
    "###,#..,###,#.#,###",
    "###,..#,..#,..#,..#",
    "###,#.#,###,#.#,###",
    "###,#.#,###,..#,###",
];
/// The heart swells on each beat, eases back and rests smaller between.
const HEARTS: [[&str; 3]; 3] = [[" ▄█▄█▄ ", "  ▀█▀  ", "       "], ["▄██▄██▄", " ▀███▀ ", "   ▀   "], ["▄██▄██▄", "▀█████▀", "  ▀█▀  "]];

/// One beat's height above the baseline, in rows, k columns after it
/// starts: P, a Q dip, the R spike, an S dip, and T.
fn wave(k: f64) -> f64 {
    let g = |c: f64, w: f64| (-((k - c) / w).powi(2)).exp();
    1.0 * g(1.5, 0.8) - 1.0 * g(6.3, 0.3) + 7.0 * g(R_AT, 0.28) - 1.3 * g(7.8, 0.3) + 2.0 * g(14.5, 1.4)
}

/// The monitor: its rate, the beats' starts in columns, the generator, and the dots.
pub struct Heartbeat {
    bpm: f64,
    beats: Vec<f64>,
    rand: Mulberry32,
    dots: ColorDots,
}

impl Heartbeat {
    /// Beat starts made as far as needed, left fractional so the rate
    /// averages true. Breathing sways it a little.
    fn extend(&mut self, g: f64) {
        while *self.beats.last().unwrap() <= g {
            let n = self.beats.len() as f64;
            let rate = self.bpm.clamp(30.0, 199.0) + 2.5 * (n * 0.8).sin() + (self.rand.next_f64() - 0.5) * 2.0;
            let last = *self.beats.last().unwrap();
            self.beats.push(last + SPEED * 60.0 / rate);
        }
    }

    /// The beat a column falls in.
    fn beat_of(&self, g: f64) -> usize {
        self.beats.partition_point(|&b| b <= g).saturating_sub(1)
    }

    /// The trace's height in rows at column g: this beat's wave, or the last
    /// one's T still running under it at a fast rate.
    fn level(&self, g: f64) -> f64 {
        let n = self.beat_of(g);
        let here = wave(g - self.beats[n]);
        if n > 0 {
            here + wave(g - self.beats[n - 1]) * f64::from(u8::from(g - self.beats[n - 1] < 20.0))
        } else {
            here
        }
    }
}

/// A heartbeat; reads `bpm`, held to 30 to 199.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(Heartbeat { bpm: options.f64("bpm", 72.0), beats: vec![-400.0], rand: Mulberry32::new(7240), dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Heartbeat {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let h = (3 * W + 31) as f64 + t * SPEED; // the newest sample, in columns
        self.extend(h + 2.0);
        let head = h.rem_euclid(W as f64);
        self.dots.clear();
        // The dotted grid: a dot every five columns on every third row, clear of the baseline.
        for c in (2..W).step_by(5) {
            let mut r = BASE as i64 - 6;
            while r < META.rows as i64 - 1 {
                if r != BASE as i64 {
                    self.dots.set((X0 + c) * 2, r * 4 + 2, GRID);
                }
                r += 3;
            }
        }
        // The trace, a sample a dot column, each joined to the last.
        let mut last: Option<i64> = None;
        for x in 0..W * 2 {
            let c = x as f64 / 2.0;
            let ahead = (c - head).rem_euclid(W as f64);
            if ahead > 0.0 && ahead <= GAP as f64 {
                last = None;
                continue;
            }
            let g = h - (head - c).rem_euclid(W as f64);
            let y = ((BASE - self.level(g)) * 4.0 + 2.0).round() as i64;
            // Brightest at the pen, fading back round the sweep.
            let age = (head - c).rem_euclid(W as f64) / W as f64;
            let color = GLOW.at(1.0 - age);
            let from = last.unwrap_or(y);
            for yy in from.min(y)..=from.max(y) {
                self.dots.set(X0 * 2 + x, yy, color);
            }
            last = Some(y);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        grid.text_at(X0, 1, "ecg  ii", TEXT);
        grid.text_at(X0 + W - 7, 1, "25 mm/s", TEXT);
        // The readout panel.
        let px = X0 + W + 3;
        let pw = META.cols as i64 - px - 1;
        for r in 1..META.rows as i64 - 1 {
            grid.set(px - 2, r, '│', GRID);
        }
        // The rate averages the last four beats that have shown their R wave.
        let mut n = self.beat_of(h - R_AT + 2.0);
        if self.beats[n] + R_AT > h {
            n -= 1;
        }
        let rr = (self.beats[n] - self.beats[n.saturating_sub(4)]) / 4.0;
        let shown = format!("{}", (SPEED * 60.0 / rr).round() as i64);
        grid.text_at(px, 2, "hr", TEXT);
        grid.text_at(px + pw - 3, 2, "bpm", TEXT);
        let step = if shown.len() > 2 { 7 } else { 8 };
        let mut dx = px + (pw - shown.len() as i64 * step + step - 6) / 2;
        for ch in shown.chars() {
            let f: Vec<&str> = FONT[ch.to_digit(10).unwrap_or(0) as usize].split(',').collect();
            for (r, row) in f.iter().enumerate() {
                for (c, b) in row.chars().enumerate() {
                    if b == '#' {
                        grid.text_at(dx + 2 * c as i64, 4 + r as i64, "██", GLOW.at(1.0));
                    }
                }
            }
            dx += step;
        }
        let since = h - (self.beats[n] + R_AT);
        let heart = HEARTS[if since < 4.0 {
            2
        } else if since < 9.0 {
            1
        } else {
            0
        }];
        for (i, s) in heart.iter().enumerate() {
            grid.text_at(px + (pw - 7) / 2, 10 + i as i64, s, HEART);
        }
    }
}
