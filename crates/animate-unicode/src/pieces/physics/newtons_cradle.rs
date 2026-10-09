//! newton's cradle: five chrome balls, each hung on a V of two strings from
//! an A-frame. An end ball swings in and knocks; the far ball swings out and
//! back. After ascii.rest's newtons-cradle.ts (MIT, @bas3line).
//!
//! The swing and the knock are ascii.rest's. In octants each ball is a disc
//! of solid dots, eight a character, and every dot is shaded as chrome is:
//! by what it reflects, the sky above, a dark band of horizon, the base
//! below, and a hard highlight where the lamp shows, so the balls look
//! polished and the highlight slides as they swing. The strings and legs are
//! lines a dot wide.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// Newton's cradle's meta.
pub static META: Meta = Meta {
    name: "newton's cradle",
    slug: "newtons-cradle",
    category: Category::Physics,
    note: "a newton's cradle in octants, its chrome balls catching the light",
    cols: 74,
    rows: 17,
    fps: 30,
    charset: Charset::Extended,
    // Chrome, darkest first, to the highlight; the strings; the frame.
    palette: Some(&["#1c2028", "#3a404c", "#5c6472", "#8a94a4", "#bcc6d4", "#e8eef6", "#ffffff", "#6a6e76", "#a48a5a"]),
    ground: Some("#0a0b0e"),
    options: Some(r#"{"lifted": 1}"#),
    ..Meta::DEFAULT
};

const N: usize = 5;
/// Ball radius, pivot to ball centre, and the pivots' row, in rows (a column is half a row).
const R: f64 = 2.0;
const L: f64 = 11.0;
const TOP: f64 = 2.0;
const AMP: f64 = 24.0 * PI / 180.0;
/// Seconds for one end to swing out and back, and for the knock to pass through the row.
const PERIOD: f64 = 1.5;
const KNOCK: f64 = 0.07;
/// Columns from a ball's pivot to each of its two string anchors.
const SPAN: f64 = 3.0;
const CHROME: Ramp = Ramp::new(0, 7);
const STRING: u8 = 7;
const FRAME: u8 = 8;

/// The swing of the left group (negative) and right group (positive), and
/// the knock passing through the middle.
fn pose(t: f64) -> (f64, f64, f64) {
    let cyc = PERIOD + 2.0 * KNOCK;
    let p = t.rem_euclid(cyc);
    let (w, q) = (TAU / PERIOD, PERIOD / 4.0);
    if p < q {
        (-AMP * (w * p).cos(), 0.0, 0.0)
    } else if p < q + KNOCK {
        (0.0, 0.0, (PI * (p - q) / KNOCK).sin())
    } else if p < 3.0 * q + KNOCK {
        (0.0, AMP * (w * (p - q - KNOCK)).sin(), 0.0)
    } else if p < 3.0 * q + 2.0 * KNOCK {
        (0.0, 0.0, -(PI * (p - 3.0 * q - KNOCK) / KNOCK).sin())
    } else {
        (-AMP * (w * (p - 3.0 * q - 2.0 * KNOCK)).sin(), 0.0, 0.0)
    }
}

/// How bright chrome is at a normal: the sky above, a dark horizon band, the
/// base below, and a hard highlight.
fn chrome(nx: f64, ny: f64, nz: f64) -> f64 {
    let m = (0.67f64 * 0.67 + 0.45 * 0.45 + 0.59 * 0.59).sqrt();
    let (lx, ly, lz) = (-0.67 / m, 0.45 / m, 0.59 / m);
    let ry = 2.0 * nz * ny; // where the view reflects to, up or down
    let env = if ry > 0.15 {
        0.3 + 0.25 * ry
    } else if ry > -0.5 {
        0.03
    } else {
        0.5 - 0.2 * (ry + 0.5)
    };
    let dl = nx * lx + ny * ly + nz * lz;
    let spec = (2.0 * nz * dl - lz).max(0.0).powi(10);
    (env + 0.12 * dl.max(0.0) + 3.5 * spec).min(1.0)
}

/// The cradle: how many balls lift, and the dots.
pub struct NewtonsCradle {
    lifted: usize,
    dots: ColorOctants,
}

/// Newton's cradle; reads `lifted` (1 to 4).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let lifted = options.f64("lifted", 1.0).round().clamp(1.0, (N - 1) as f64) as usize;
    Box::new(NewtonsCradle { lifted, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for NewtonsCradle {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let cx = META.cols as f64 / 2.0;
        let pivots: Vec<f64> = (0..N).map(|i| cx + (i as f64 - (N - 1) as f64 / 2.0) * 4.0 * R).collect();
        let base = TOP + L + R + 1.0; // the row the base sits on
        let rail = 26.0; // the top rail runs this many columns either side of centre
        let leg = rail + 0.5 + 0.5 * (base - TOP + 0.5); // the legs step out half a column a row
                                                         // Columns are two dots, rows four.
        let d = |c: f64, r: f64| ((c * 2.0).round() as i64, (r * 4.0).round() as i64);
        let (left, right, knock) = pose(t);
        self.dots.clear();
        // The legs splay out from the rail to the base.
        for side in [-1.0, 1.0] {
            let (a, b) = (d(cx + side * (rail + 0.5), TOP - 0.5), d(cx + side * leg, base));
            self.dots.line(a.0, a.1, b.0, b.1, FRAME);
        }
        let k = self.lifted;
        for (n, &pivot) in pivots.iter().enumerate() {
            // Only one end swings at a time, so a ball in both end groups takes whichever is moving.
            let th = if n < k { left } else { 0.0 } + if n >= N - k { right } else { 0.0 };
            // A knock nudges the row a fraction of a column toward the ball about to leave.
            let push = if (knock > 0.0 && n < N - k) || (knock < 0.0 && n >= k) { 0.5 * knock } else { 0.0 };
            let (sx, sy) = th.sin_cos();
            let (bx, by) = (pivot + push + 2.0 * L * sx, TOP + L * sy);
            // Each string ties on just off the top of the ball.
            let (ax, ay) = (pivot + push + 2.0 * (L - R) * sx, TOP + (L - R) * sy);
            for (px, tx, ty) in [(pivot - SPAN, ax - 0.5 * sy, ay + 0.25 * sx), (pivot + SPAN, ax + 0.5 * sy, ay - 0.25 * sx)] {
                let (a, b) = (d(px, TOP), d(tx, ty));
                self.dots.line(a.0, a.1, b.0, b.1, STRING);
            }
            // The ball, a disc of chrome dots, R rows across: 4R dots each way.
            let (cxd, cyd, rd) = (bx * 2.0, by * 4.0, R * 4.0);
            for y in (cyd - rd).floor() as i64..=(cyd + rd).ceil() as i64 {
                for x in (cxd - rd).floor() as i64..=(cxd + rd).ceil() as i64 {
                    let (nx, ny) = ((x as f64 + 0.5 - cxd) / rd, -(y as f64 + 0.5 - cyd) / rd);
                    let q = nx * nx + ny * ny;
                    if q >= 1.0 || x < 0 || y < 0 {
                        continue;
                    }
                    let v = chrome(nx, ny, (1.0 - q).sqrt());
                    // Bright metal is dense; on paper the shadows carry the ink.
                    let ink = if env.paper { 1.05 - v } else { 0.4 + 0.6 * v };
                    if dither(x as usize, y as usize, ink) {
                        self.dots.set(x, y, CHROME.at(v));
                    } else {
                        self.dots.unset(x, y);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The rail across the top, and the base the legs stand on.
        for c in (cx - rail) as i64..(cx + rail) as i64 {
            grid.set(c, TOP as i64 - 1, '═', FRAME);
        }
        let (b0, b1) = ((cx - leg).floor() as i64 - 1, (cx + leg).ceil() as i64);
        for c in b0..=b1 {
            grid.set(
                c,
                base as i64,
                if c == b0 {
                    '▝'
                } else if c == b1 {
                    '▘'
                } else {
                    '▀'
                },
                FRAME,
            );
        }
    }
}
