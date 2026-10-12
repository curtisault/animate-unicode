//! kite: a diamond kite on a long string from off the bottom left. Gusts
//! lift it and pull the string straight, lulls let it sag, and the tail
//! flutters. After ascii.rest's kite.ts (MIT, @bas3line).
//!
//! The wind, the kite's ride and roll, the tail and the sagging string are
//! ascii.rest's. The kite is drawn in octants, its sail's edges cut on the
//! diagonal a dot at a time, so it can ride and roll smoothly rather than a
//! cell at a time; the string and the tail are braille, fine curves with
//! bows tied along the tail. In colour it is a harlequin kite, red and gold
//! panels crossed by its spars, with bows of every colour on its tail and a
//! white string.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The kite's meta.
pub static META: Meta = Meta {
    name: "kite",
    slug: "kite",
    category: Category::Objects,
    note: "a harlequin octant kite riding the wind, a braille string and tail",
    cols: 64,
    rows: 24,
    fps: 15,
    charset: Charset::Extended,
    // The panels, red and gold; the spars; the string; the tail; its bows.
    palette: Some(&["#e8343a", "#f6c03a", "#5a3a22", "#e8ecf4", "#c8ccd8", "#4ac8ff", "#6adc6a", "#ff7ad0"]),
    ground: Some("#0a0e18"),
    ..Meta::DEFAULT
};

const RED: u8 = 0;
const GOLD: u8 = 1;
const SPAR: u8 = 2;
const STRING: u8 = 3;
const TAIL: u8 = 4;
const BOWS: [u8; 3] = [5, 6, 7];
/// Seconds before the wind repeats; half the spar in columns (above it the
/// sail climbs B / 2 rows, below it drops B rows); rows of tail.
const P: f64 = 40.0;
const B: f64 = 8.0;
const TAIL_ROWS: f64 = 7.0;

/// The kite: its sail and spars, and its string and tail.
pub struct Kite {
    sail: ColorOctants,
    lines: ColorDots,
}

/// A kite; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Kite { sail: ColorOctants::new(META.cols, META.rows), lines: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Kite {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let rows = META.rows as f64;
        let ph = TAU * t / P;
        let wind = 0.5 + 0.3 * (2.0 * ph).sin() + 0.2 * (5.0 * ph + 1.3).sin();
        // The kite rides up in a gust and drifts a little sideways.
        let mid = 45.5 + 0.7 * (ph + 0.4).sin();
        let rs = 7.0 + 1.4 * (0.5 - wind); // the spar's row
                                           // It rolls into the wind, the bottom half of the sail standing off the top half.
        let roll = (5.0 * ph).sin() + 0.35 * (13.0 * ph + 1.0).sin();
        let lo = mid - roll * 0.9;
        self.sail.clear();
        let (w, h) = (self.sail.width() as i64, self.sail.height() as i64);
        for j in 0..h {
            let y = (j as f64 + 0.5) / 4.0 - rs; // rows below the spar
            if !(-B / 2.0..B).contains(&y) {
                continue;
            }
            // Above the spar the edges climb two columns a row; below they drop one.
            let (centre, half) = if y < 0.0 { (mid, B + 2.0 * y) } else { (mid + (lo - mid) * (y / B).min(1.0) * 2.0, B - y) };
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0 - centre;
                if x.abs() >= half {
                    continue;
                }
                // The spars across and down, and two opposite panels in each
                // colour, the gold ones sparser, so the harlequin shows in one ink too.
                let spar = x.abs() < 0.3 || y.abs() < 0.15;
                let red = (x < 0.0) == (y < 0.0);
                if spar || red || dither(i as usize, j as usize, 0.55) {
                    self.sail.set(
                        i,
                        j,
                        if spar {
                            SPAR
                        } else if red {
                            RED
                        } else {
                            GOLD
                        },
                    );
                }
            }
        }
        self.lines.clear();
        // The tail streams downwind from the bottom corner, its flutter growing
        // toward the free end, with a bow tied every few rows.
        let lean = 0.15 + 0.35 * wind;
        let amp = (0.35 + 0.3 * wind) / 3.5;
        let tail_x = |s: f64| lo + 2.0 * (lo - mid) + lean * s + amp * s * (2.2 * s - 15.0 * ph).sin();
        let mut s = 0.0;
        while s < TAIL_ROWS {
            let (x, y) = (tail_x(s), rs + B + s);
            if y < rows {
                self.lines.set((x * 2.0) as i64, (y * 4.0) as i64, TAIL);
            }
            s += 0.05;
        }
        for (n, s) in [1.5, 3.5, 5.5].into_iter().enumerate() {
            let (x, y) = ((tail_x(s) * 2.0) as i64, ((rs + B + s) * 4.0) as i64);
            for (dx, dy) in [(-2, -1), (-2, 1), (-1, 0), (1, 0), (2, -1), (2, 1)] {
                self.lines.set(x + dx, y + dy, BOWS[n]);
            }
        }
        // The string comes in from a hand far off the bottom left, sagging as
        // the wind drops and pulling straighter in a gust.
        let (hand, knot) = ((-20.0, rows + 3.0), (mid - B, rs + 0.1));
        let sag = 1.7 - 1.1 * wind;
        let mut c = 0.0;
        while c < knot.0 {
            let u = (c - hand.0) / (knot.0 - hand.0);
            let y = hand.1 + (knot.1 - hand.1) * u + 4.0 * sag * u * (1.0 - u);
            self.lines.set((c * 2.0) as i64, (y * 4.0) as i64, STRING);
            c += 0.25;
        }
        grid.clear();
        self.lines.draw(grid, 0, 0);
        self.sail.draw(grid, 0, 0);
    }
}
