//! pendulum wave: a row of pendulums seen from above, hung from one rail.
//! Bob n swings 60 + n times every 150 seconds, so the row drifts from one
//! wave into two, three, a braid and a zigzag, then lines up again once a
//! cycle. After ascii.rest's pendulum-wave.ts (MIT, @bas3line).
//!
//! In octants each bob is a round disc of dots that moves a dot at a time,
//! four to a row, so the waves glide instead of stepping. Each bob is
//! coloured by where it is in its swing, round the colour wheel, so bobs in
//! step share a colour and the waves show as bands of hue running down the
//! row; a short streak behind each bob shows its speed.

use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The pendulum wave's meta.
pub static META: Meta = Meta {
    name: "pendulum wave",
    slug: "pendulum-wave",
    category: Category::Physics,
    note: "a pendulum wave in octants, each bob coloured by its phase",
    cols: 67,
    rows: 19,
    fps: 30,
    charset: Charset::Extended,
    // Eight hues round the wheel, each bright then dim; the rail.
    palette: Some(&[
        "#ff5a5a", "#8a2e2e", "#ffa84a", "#8a5a26", "#f4e04a", "#857a26", "#6ae06a", "#327a32", "#4ad8d8", "#267a7a", "#5a8aff", "#2e4a8a", "#a86aff",
        "#5a368a", "#ff6ad0", "#8a3670", "#7a808c",
    ]),
    ground: Some("#0a0a0e"),
    ..Meta::DEFAULT
};

/// Bobs, columns from one to the next, seconds until every bob lines up
/// again, swings the slowest makes in one cycle, and seconds of motion blur.
const N: usize = 21;
const GAP: usize = 3;
const CYCLE: f64 = 150.0;
const BASE: f64 = 60.0;
const BLUR: f64 = 0.07;
/// A wave and a half across the row at t = 0.
const T0: f64 = 1.5 * CYCLE / (N - 1) as f64;
const HUES: usize = 8;
const RAIL: u8 = 16;

/// The pendulum wave's dots.
pub struct PendulumWave {
    dots: ColorOctants,
}

/// A pendulum wave; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(PendulumWave { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for PendulumWave {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let t = time + T0;
        let left = (META.cols - (N - 1) * GAP - 1) / 2;
        let h = self.dots.height() as f64;
        // The swing spans every row but the first and last; in dots, four a row.
        let y = |n: usize, t: f64| 4.0 + (1.0 - (TAU * (BASE + n as f64) * t / CYCLE).cos()) * (h - 8.0) / 2.0;
        self.dots.clear();
        // The rail, straight above every bob at rest, a little past each end.
        let rail = (h / 2.0) as i64;
        let (x0, x1) = (((left - 1) * 2) as i64, ((left + (N - 1) * GAP + 1) * 2 + 1) as i64);
        for x in x0..=x1 {
            self.dots.set(x, rail, RAIL);
        }
        for n in 0..N {
            let cx = ((left + n * GAP) * 2) as f64 + 1.0;
            // Where it is in its swing: 0 at one end, half at the other.
            let phase = ((BASE + n as f64) * t / CYCLE).rem_euclid(1.0);
            let hue = ((phase * HUES as f64) as usize % HUES) as u8;
            let (now, then) = (y(n, t), y(n, t - BLUR));
            // The streak of the last instant, dim, then the bob over it.
            let (a, b) = (now.min(then), now.max(then));
            let mut s = a;
            while s <= b {
                self.dots.set(cx as i64, s as i64, hue * 2 + 1);
                s += 1.0;
            }
            for dy in -2i64..=2 {
                for dx in -2i64..=1 {
                    let (fx, fy) = (dx as f64 + 0.5, dy as f64);
                    if fx * fx + fy * fy <= 5.0 {
                        self.dots.set(cx as i64 + dx, now.round() as i64 + dy, hue * 2);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw_split(grid, 0, 0);
    }
}
