//! analog clock: a round wall clock keeping local time. A lit bezel, a tick
//! for every minute and hour, and hour, minute and ticking second hands.
//! After ascii.rest's analog-clock.ts (MIT, @bas3line).
//!
//! The face and its light are ascii.rest's, drawn in braille: the bezel a
//! torus lit from the upper left and dithered dot by dot, a fine dot for each
//! minute and a short stroke for each hour, and every hand a straight line of
//! dots at its true angle, the hour hand three dots thick, the minute two,
//! the second one. In colour the bezel is brass, the hands are white with a
//! red second hand ending in a ring, and the numerals are cream.

use crate::clock::Ticker;
use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The clock's meta.
pub static META: Meta = Meta {
    name: "analog clock",
    slug: "analog-clock",
    category: Category::Objects,
    note: "a braille wall clock in local time, a brass bezel, a red second hand",
    cols: 47,
    rows: 23,
    fps: 8,
    charset: Charset::Extended,
    options: Some(r#"{"numerals": true, "seconds": true}"#),
    clock: true,
    // The bezel, shadow to light; the hour ticks; the minute dots; the hour
    // and minute hands; the second hand; the numerals.
    palette: Some(&["#5a4018", "#8c6a2c", "#c49a48", "#ecc87a", "#fff0c0", "#c8ccd8", "#6a6e80", "#f2f4fa", "#ff3a3a", "#ece2c8"]),
    ground: Some("#0b0c10"),
    ..Meta::DEFAULT
};

const BRASS: Ramp = Ramp::new(0, 5);
const TICK: u8 = 5;
const DOT: u8 = 6;
const HANDS: u8 = 7;
const SECOND: u8 = 8;
const NUMERAL: u8 = 9;
/// The bezel's middle, in columns (the face is drawn in columns, y up).
const R: f64 = 21.0;

/// The clock: whether it shows numerals and a second hand, its ticker, the
/// bezel's dots, and the dots.
pub struct AnalogClock {
    numerals: bool,
    seconds: bool,
    ticker: Ticker,
    bezel: Vec<(i64, i64, u8)>,
    dots: ColorDots,
}

/// An analog clock; reads `numerals` and `seconds`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
    let lm = (0.5f64 * 0.5 + 0.6 * 0.6 + 0.62 * 0.62).sqrt();
    let (lx, ly, lz) = (-0.5 / lm, 0.6 / lm, 0.62 / lm);
    // The bezel is a fixed picture: a torus cut across, lit from the upper left.
    let mut bezel = vec![];
    for j in 0..dots.height() {
        for i in 0..dots.width() {
            let (x, y) = ((i as f64 + 0.5) / 2.0 - cx, (cy - (j as f64 + 0.5) / 4.0) * 2.0);
            let d = x.hypot(y);
            let s = (d - R) / 1.3;
            if s.abs() >= 1.0 {
                continue;
            }
            let z = (1.0 - s * s).sqrt();
            let light = (s * x / d * lx + s * y / d * ly + z * lz).max(0.0);
            if dither(i, j, 0.45 + 0.55 * light) {
                bezel.push((i as i64, j as i64, BRASS.at(light)));
            }
        }
    }
    Box::new(AnalogClock { numerals: options.bool("numerals", true), seconds: options.bool("seconds", true), ticker: Ticker::default(), bezel, dots })
}

impl AnalogClock {
    /// A hand along clock angle a (radians from twelve, clockwise) from r0
    /// to r1 columns out, `thick` dots across.
    fn hand(&mut self, a: f64, r0: f64, r1: f64, thick: i64, color: u8) {
        let (cx, cy) = (META.cols as f64, META.rows as f64 * 2.0);
        // In dots, two a column and four a row, so the face is round: a column
        // across is two dots and a column's worth up is two dots too.
        let (sx, sy) = (a.sin() * 2.0, -a.cos() * 2.0);
        let (px, py) = (-sy / 2.0, sx / 2.0); // across the hand, a dot
        let steps = ((r1 - r0) * 2.0).ceil() as i64 * 2;
        for k in 0..=steps {
            let s = r0 + (r1 - r0) * k as f64 / steps as f64;
            for w in 0..thick {
                let off = w as f64 - (thick - 1) as f64 / 2.0;
                self.dots.set((cx + sx * s + px * off).floor() as i64, (cy + sy * s + py * off).floor() as i64, color);
            }
        }
    }
}

impl Piece for AnalogClock {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cx, cy) = (META.cols as f64, META.rows as f64 * 2.0);
        let now = self.ticker.now(t, env.now).local();
        let sec = f64::from(now.second);
        let min = f64::from(now.minute) + sec / 60.0;
        let hr = f64::from(now.hour % 12) + min / 60.0;
        self.dots.clear();
        for &(x, y, c) in &self.bezel {
            self.dots.set(x, y, c);
        }
        // A dot for each minute and a short stroke for each hour.
        for i in 0..60 {
            let a = f64::from(i) * TAU / 60.0;
            let (sx, sy) = (a.sin() * 2.0, -a.cos() * 2.0);
            if i % 5 != 0 {
                self.dots.set((cx + sx * 18.0).floor() as i64, (cy + sy * 18.0).floor() as i64, DOT);
            } else if !self.numerals || i % 15 != 0 {
                for s in [16.6, 17.1, 17.6, 18.1] {
                    self.dots.set((cx + sx * s).floor() as i64, (cy + sy * s).floor() as i64, TICK);
                }
            }
        }
        self.hand(hr * PI / 6.0, 1.0, 9.5, 3, HANDS);
        self.hand(min * PI / 30.0, 1.0, 15.0, 2, HANDS);
        if self.seconds {
            // A fine line with a ring at its tip and a weight on its tail.
            let a = sec * PI / 30.0;
            self.hand(a, -4.4, 13.6, 1, SECOND);
            let (tx, ty) = (cx + a.sin() * 2.0 * 14.6, cy - a.cos() * 2.0 * 14.6);
            for k in 0..12 {
                let b = f64::from(k) * TAU / 12.0;
                self.dots.set((tx + 1.5 * b.cos()).round() as i64, (ty + 1.5 * b.sin()).round() as i64, SECOND);
            }
            self.hand(a + PI, 3.0, 4.4, 3, SECOND);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The numerals over the hands; the centre over everything.
        let (ccx, ccy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
        let put = |grid: &mut Grid, x: f64, y: f64, s: &str| grid.text_at((ccx + x).floor() as i64, (ccy - y / 2.0).floor() as i64, s, NUMERAL);
        if self.numerals {
            put(grid, -1.0, 15.6, "12");
            put(grid, 15.2, 0.0, "3");
            put(grid, 0.0, -15.6, "6");
            put(grid, -15.2, 0.0, "9");
        }
        grid.set(ccx.floor() as i64, ccy.floor() as i64, '●', if self.seconds { SECOND } else { HANDS });
    }
}
