//! harmonograph: a pen on two pendulums, one swinging across and one up and
//! down, both slowly running down, so the figure winds in on itself to a
//! dense centre. While the next figure starts, the last is lifted from the
//! outside in. After ascii.rest's harmonograph.ts (MIT, @bas3line).
//!
//! In braille the pen draws a true curve a dot wide, over a 128×112 sheet,
//! so the passes stay apart where strokes in a character ran together. The
//! ink changes colour as the pen goes, rose on the first wide swings through
//! violet to blue at the centre, so the order the figure was drawn in shows.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, FRAC_PI_4, TAU};

/// The harmonograph's meta.
pub static META: Meta = Meta {
    name: "harmonograph",
    slug: "harmonograph",
    category: Category::Physics,
    note: "a harmonograph in braille, its ink turning rose to blue inward",
    cols: 64,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    // The ink from the first passes to the last; the pen.
    palette: Some(&["#f07a8a", "#e0609a", "#c050b0", "#9050c8", "#6a5ad8", "#4a70e0", "#3a90e8", "#ffffff"]),
    ground: Some("#0c0b10"),
    ..Meta::DEFAULT
};

/// Pendulum time per sample, too short to skip a dot.
const STEP: f64 = 0.0008;
/// Swings across and up and down: frequency and phase each, how much faster
/// the up and down swing runs down, how many passes, and how far in it winds.
const FIGURES: [(f64, f64, f64, f64, f64, f64, f64); 3] =
    [(1.0, FRAC_PI_2, 1.02, 0.0, 1.15, 9.0, 0.12), (2.006, 0.0, 1.0, 0.0, 1.0, 6.0, 0.15), (1.0, FRAC_PI_4, 0.985, 0.0, 0.9, 8.0, 0.15)];
/// Seconds drawing and showing a figure; the lift runs under the next figure's start.
const DRAW: f64 = 10.0;
const SHOW: f64 = 2.5;
const LIFT: f64 = 3.0;
const PERIOD: f64 = DRAW + SHOW;
/// The first figure, a little over half drawn.
const START: f64 = 6.0;
const INK: Ramp = Ramp::new(0, 7);
const PEN: u8 = 7;

/// One figure: how long it runs in pendulum time, and the pen's dots in order.
struct Sheet {
    end: f64,
    pts: Vec<(i64, i64)>,
}

fn sheet((fx, px, fy, py, lean, passes, wind): (f64, f64, f64, f64, f64, f64, f64), w: usize, h: usize) -> Sheet {
    let end = TAU * passes;
    let n = (end / STEP) as usize + 1;
    let mut raw = Vec::with_capacity(n);
    let (mut x0, mut x1, mut y0, mut y1) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for i in 0..n {
        let t = i as f64 * STEP;
        // Friction at the pivots takes the same off each swing, so the passes come in evenly.
        let a = 1.0 - (1.0 - wind) * t / end;
        let (x, y) = (a * (fx * t + px).sin(), -(a.powf(lean)) * (fy * t + py).sin());
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
        raw.push((x, y));
    }
    // Centred on the sheet, four fifths of it across and down. A cell is
    // twice as tall as wide, so the figure keeps ascii.rest's proportions.
    let (sx, sy) = ((0.8 * w as f64 - 2.0) / (x1 - x0), (0.8 * h as f64 - 2.4) / (y1 - y0));
    let mut pts: Vec<(i64, i64)> = Vec::with_capacity(n);
    for (x, y) in raw {
        let p = ((w as f64 / 2.0 + (x - (x0 + x1) / 2.0) * sx).floor() as i64, (h as f64 / 2.0 + (y - (y0 + y1) / 2.0) * sy).floor() as i64);
        pts.push(p);
    }
    Sheet { end, pts }
}

/// The harmonograph: its figures, and the dots.
pub struct Harmonograph {
    sheets: Vec<Sheet>,
    dots: ColorDots,
}

/// A harmonograph; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let sheets = FIGURES.iter().map(|&f| sheet(f, dots.width(), dots.height())).collect();
    Box::new(Harmonograph { sheets, dots })
}

impl Piece for Harmonograph {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let tt = START + t;
        let idx = (tt / PERIOD).floor() as usize;
        let local = tt - idx as f64 * PERIOD;
        let n = self.sheets.len();
        self.dots.clear();
        // The last figure, lifted in the order it went down, outer swings first.
        if local < LIFT {
            let old = &self.sheets[(idx + n - 1) % n];
            let from = (local / LIFT * old.pts.len() as f64) as usize;
            for (i, &(x, y)) in old.pts.iter().enumerate().skip(from) {
                self.dots.set(x, y, INK.at(i as f64 / old.pts.len() as f64));
            }
        }
        let s = &self.sheets[idx % n];
        let tau = local.min(DRAW) / DRAW * s.end;
        let upto = ((tau / STEP) as usize).min(s.pts.len() - 1);
        for (i, &(x, y)) in s.pts.iter().enumerate().take(upto + 1) {
            self.dots.set(x, y, INK.at(i as f64 / s.pts.len() as f64));
        }
        if local < DRAW {
            let (x, y) = s.pts[upto];
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
                self.dots.set(x + dx, y + dy, PEN);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
