//! digital clock: the local time as big seven-segment digits, HH:MM:SS, with
//! the colon blinking each second and the date set small underneath. After
//! ascii.rest's digital-clock.ts (MIT, @bas3line).
//!
//! In octants each segment has the shape of a real one, a long hexagon with
//! pointed ends, so the segments meet at the corners with a sliver between
//! them, as on an LED display. Lit segments glow red; unlit ones are a faint
//! dithered trace in a dark red, like the glass of a display that is off.

use crate::clock::Ticker;
use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};

/// The digital clock's meta.
pub static META: Meta = Meta {
    name: "digital clock",
    slug: "digital-clock",
    category: Category::Ui,
    note: "the local time in octant seven-segment digits, lit red",
    cols: 70,
    rows: 11,
    fps: 10,
    charset: Charset::Extended,
    options: Some(r#"{"hour12": false, "seconds": true, "date": true}"#),
    clock: true,
    // Lit, unlit, the date.
    palette: Some(&["#ff4a3a", "#3c1412", "#c8a8a0"]),
    ground: Some("#0c0707"),
    ..Meta::DEFAULT
};

const LIT: u8 = 0;
const UNLIT: u8 = 1;
const DATE: u8 = 2;
/// A digit is 8 cells wide and 7 tall: 16 by 28 dots. Segments are 4 dots thick.
const DW: i64 = 8;
const DH: i64 = 7;
const T: f64 = 4.0;
/// Which segments a to g each digit lights.
const DIGITS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];
const DAYS: [&str; 7] = ["sunday", "monday", "tuesday", "wednesday", "thursday", "friday", "saturday"];
const MONTHS: [&str; 12] = ["january", "february", "march", "april", "may", "june", "july", "august", "september", "october", "november", "december"];

/// Segments a to g in dots within a digit: along x (true) or y, the centre
/// line across, and the run along.
const SEGS: [(bool, f64, f64, f64); 7] = [
    (true, 2.0, 2.0, 14.0),    // a, top
    (false, 14.0, 2.0, 14.0),  // b, top right
    (false, 14.0, 14.0, 26.0), // c, bottom right
    (true, 26.0, 2.0, 14.0),   // d, bottom
    (false, 2.0, 14.0, 26.0),  // e, bottom left
    (false, 2.0, 2.0, 14.0),   // f, top left
    (true, 14.0, 2.0, 14.0),   // g, middle
];

/// The clock: its options, its ticker, and the dots.
pub struct DigitalClock {
    hour12: bool,
    seconds: bool,
    date: bool,
    ticker: Ticker,
    dots: ColorOctants,
}

/// A digital clock; reads `hour12`, `seconds` and `date`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(DigitalClock {
        hour12: options.bool("hour12", false),
        seconds: options.bool("seconds", true),
        date: options.bool("date", true),
        ticker: Ticker::default(),
        dots: ColorOctants::new(META.cols, META.rows),
    })
}

impl DigitalClock {
    /// Digit n (or none, for a dark one) with its top left at cell (x, y).
    fn digit(&mut self, x: i64, y: i64, n: Option<usize>) {
        let mask = n.map_or(0, |n| DIGITS[n]);
        let (ox, oy) = (x * 2, y * 4);
        for (s, &(across, mid, from, to)) in SEGS.iter().enumerate() {
            let lit = mask >> s & 1 == 1;
            // A long hexagon: full thickness in the middle, pointed at both ends.
            for along in from as i64 - 2..=to as i64 + 2 {
                for off in -2i64..2 {
                    let d = (off as f64 + 0.5).abs();
                    let a = along as f64 + 0.5;
                    if d >= T / 2.0 || a < from + d || a > to - d {
                        continue;
                    }
                    let (px, py) = if across { (ox + along, oy + mid as i64 + off) } else { (ox + mid as i64 + off, oy + along) };
                    if lit {
                        self.dots.set(px, py, LIT);
                    } else if dither(px as usize, py as usize, 0.3) {
                        self.dots.set(px, py, UNLIT);
                    }
                }
            }
        }
    }

    fn colon(&mut self, x: i64, y: i64, on: bool) {
        for row in [2, 4] {
            for dy in 0..4 {
                for dx in 0..4 {
                    let (px, py) = (x * 2 + dx, (y + row) * 4 + dy);
                    if on {
                        self.dots.set(px, py, LIT);
                    } else if dither(px as usize, py as usize, 0.3) {
                        self.dots.set(px, py, UNLIT);
                    }
                }
            }
        }
    }
}

impl Piece for DigitalClock {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let wall = self.ticker.now(t, env.now);
        let now = wall.local();
        let on = now.milli < 500;
        let top = if self.date { 1 } else { 2 };
        let mut h = now.hour;
        let pm = h >= 12;
        if self.hour12 {
            h = if h.is_multiple_of(12) { 12 } else { h % 12 };
        }
        let pairs: Vec<u32> = [h, now.minute, now.second].into_iter().take(if self.seconds { 3 } else { 2 }).collect();
        let width = pairs.len() as i64 * (2 * DW + 2) + (pairs.len() as i64 - 1) * 6;
        let mut x = (META.cols as i64 - width) / 2;
        self.dots.clear();
        for (i, &v) in pairs.iter().enumerate() {
            if i > 0 {
                self.colon(x + 2, top, on);
                x += 6;
            }
            // A 12-hour clock leaves its first digit dark rather than showing a zero.
            let first = if i == 0 && self.hour12 && v < 10 { None } else { Some(v as usize / 10) };
            self.digit(x, top, first);
            self.digit(x + DW + 2, top, Some(v as usize % 10));
            x += 2 * DW + 2;
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        if self.date {
            let mut line = format!("{} {} {} {}", DAYS[now.weekday as usize], now.day, MONTHS[now.month as usize - 1], now.year);
            if self.hour12 {
                line += if pm { "  pm" } else { "  am" };
            }
            grid.text_at((META.cols as i64 - line.len() as i64) / 2, top + DH + 1, &line, DATE);
        }
    }
}
