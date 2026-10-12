//! candlesticks: a price chart of five-minute candles, hollow when the price
//! rose and solid when it fell. The last candle forms live, then the chart
//! steps. After ascii.rest's candlesticks.ts (MIT, @bas3line).
//!
//! The prices are ascii.rest's, from its seeded generator. In a trading
//! screen's colours: rising candles hollow and green, falling ones solid
//! and red, the candle still forming in a brighter shade of its own, the
//! last price an amber line running out to the axis.

use crate::util::Mulberry32;
use crate::{Category, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The candlesticks' meta.
pub static META: Meta = Meta {
    name: "candlesticks",
    slug: "candlesticks",
    category: Category::Data,
    note: "green hollow up candles, red solid down ones, the last forming live",
    cols: 70,
    rows: 22,
    fps: 15,
    options: Some(r#"{"start": 100, "seed": 3}"#),
    // Up, down, up live, down live; axis; text; the last price.
    palette: Some(&["#4ac87a", "#e05a5a", "#9af0b8", "#ff9a9a", "#5c6370", "#c8ccd4", "#f4b04a"]),
    ground: Some("#0c0e12"),
    ..Meta::DEFAULT
};

const UP: u8 = 0;
const DOWN: u8 = 1;
const AXIS: u8 = 4;
const TEXT: u8 = 5;
const LAST: u8 = 6;
/// Candles on screen (four columns each), seconds per candle, price moves
/// per candle, the chart's rows, and candles already traded at t = 0.
const SLOTS: usize = 15;
const PERIOD: f64 = 1.4;
const TICKS: usize = 10;
const TOP: i64 = 2;
const H: i64 = 17;
const K0: usize = 50;
const STEPS: [f64; 13] = [0.05, 0.1, 0.2, 0.25, 0.5, 1.0, 2.0, 2.5, 5.0, 10.0, 20.0, 25.0, 50.0];

/// A candle: open, close, high, low.
#[derive(Clone, Copy)]
struct Candle {
    o: f64,
    c: f64,
    h: f64,
    l: f64,
}

/// The market: where it started, its seed, and every candle's path so far.
pub struct Candlesticks {
    start: f64,
    seed: f64,
    paths: Vec<Vec<f64>>,
}

impl Candlesticks {
    fn hash(&self, k: f64) -> f64 {
        Mulberry32::new((self.seed * 7919.0 + k * 104_729.0) as i64 as u32).next_f64()
    }

    fn trend(&self, k: usize) -> f64 {
        let u = k as f64 / 9.0;
        let (i, f) = (u.floor(), u.fract());
        let e = f * f * (3.0 - 2.0 * f);
        (self.hash(i) * (1.0 - e) + self.hash(i + 1.0) * e - 0.5) * self.start * 0.0017 * 0.55
    }

    /// Each candle's path of prices, made in order since it opens at the last close.
    fn path(&mut self, k: usize) -> &[f64] {
        let vol = self.start * 0.0017;
        while self.paths.len() <= k {
            let n = self.paths.len();
            let mut rand = Mulberry32::new((self.seed * 15_485_863.0 + n as f64 * 2_654_435_761.0) as i64 as u32);
            let mut p = if n > 0 { self.paths[n - 1][TICKS] } else { self.start };
            let mut out = vec![p];
            for _ in 0..TICKS {
                let g = (-2.0 * (1.0 - rand.next_f64()).ln()).sqrt() * (2.0 * PI * rand.next_f64()).cos();
                p += g * vol + self.trend(n) - (p - self.start) * 0.006;
                out.push(p);
            }
            self.paths.push(out);
        }
        &self.paths[k]
    }

    fn candle(&mut self, k: usize, upto: usize) -> Candle {
        let p = &self.path(k)[..=upto];
        Candle { o: p[0], c: p[upto], h: p.iter().copied().fold(f64::MIN, f64::max), l: p.iter().copied().fold(f64::MAX, f64::min) }
    }
}

/// Candlesticks; read `start` (the opening price) and `seed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(Candlesticks { start: options.f64("start", 100.0), seed: options.f64("seed", 3.0), paths: vec![] })
}

fn clock(k: usize) -> String {
    let m = (9 * 60 + 30 + k * 5) % 1440;
    format!("{:02}:{:02}", m / 60, m % 60)
}

impl Piece for Candlesticks {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let live = K0 + (t / PERIOD).floor() as usize;
        let ticks = 1 + (t.rem_euclid(PERIOD) / PERIOD * TICKS as f64) as usize;
        let cs: Vec<Candle> = (0..SLOTS)
            .map(|s| {
                let k = live + 1 + s - SLOTS;
                self.candle(k, if k == live { ticks.min(TICKS) } else { TICKS })
            })
            .collect();
        // Scale: a round price per row, centred on the finished candles.
        let done = &cs[..SLOTS - 1];
        let hi = cs.iter().map(|c| c.h).fold(f64::MIN, f64::max);
        let lo = cs.iter().map(|c| c.l).fold(f64::MAX, f64::min);
        let step = STEPS.iter().copied().find(|&s| (hi - lo) / s <= (H - 2) as f64).unwrap_or(100.0);
        let mid = (done.iter().map(|c| c.h).fold(f64::MIN, f64::max) + done.iter().map(|c| c.l).fold(f64::MAX, f64::min)) / 2.0;
        let mut top = (mid / step + (H - 1) as f64 / 2.0).round() * step;
        top = top.max((hi / step).ceil() * step).min((lo / step).floor() * step + (H - 1) as f64 * step);
        let row = |p: f64| TOP + ((top - p) / step).round() as i64;
        grid.clear();
        // Labels at the first round interval at least three rows apart.
        let axis = SLOTS as i64 * 4 + 1;
        let every = STEPS.iter().copied().find(|&s| s >= step * 3.0 - 1e-9 && (s / step - (s / step).round()).abs() < 1e-6).unwrap_or(step * 4.0);
        for r in TOP..TOP + H {
            let p = top - (r - TOP) as f64 * step;
            if (p / every - (p / every).round()).abs() < 1e-6 {
                grid.text_at(axis, r, &format!("┤ {p:.2}"), AXIS);
            } else {
                grid.set(axis, r, '│', AXIS);
            }
        }
        // The last price as a dotted line out to the axis.
        let last = cs[SLOTS - 1];
        let lr = row(last.c);
        for c in SLOTS as i64 * 4 - 1..axis {
            grid.set(c, lr, '┄', LAST);
        }
        grid.text_at(axis, lr, &format!("├ {:.2}", last.c), LAST);
        for (s, c) in cs.iter().enumerate() {
            let (x, up) = (s as i64 * 4, c.c >= c.o);
            let color = match (up, s == SLOTS - 1) {
                (true, false) => UP,
                (false, false) => DOWN,
                (true, true) => 2,
                (false, true) => 3,
            };
            let (rh, rl, rt, rb) = (row(c.h), row(c.l), row(c.o.max(c.c)), row(c.o.min(c.c)));
            for r in rh..=rl {
                grid.set(x + 1, r, '│', color);
            }
            if !up {
                for r in rt..=rb {
                    grid.text_at(x, r, "███", color);
                }
            } else if rt == rb {
                let mid = match (rh < rt, rl > rb) {
                    (true, true) => '┼',
                    (true, false) => '┴',
                    (false, true) => '┬',
                    _ => '─',
                };
                grid.text_at(x, rt, &format!("├{mid}┤"), color);
            } else {
                grid.text_at(x, rt, &format!("┌{}┐", if rh < rt { '┴' } else { '─' }), color);
                for r in rt + 1..rb {
                    grid.text_at(x, r, "│ │", color);
                }
                grid.text_at(x, rb, &format!("└{}┘", if rl > rb { '┬' } else { '─' }), color);
            }
        }
        // Time along the bottom, every fourth candle.
        let base = TOP + H;
        grid.text_at(0, base, &"─".repeat(axis as usize), AXIS);
        grid.set(axis, base, '┘', AXIS);
        for s in 0..SLOTS {
            let k = live + 1 + s - SLOTS;
            if !k.is_multiple_of(4) {
                continue;
            }
            grid.set(s as i64 * 4 + 1, base, '┴', AXIS);
            if s > 0 {
                grid.text_at(s as i64 * 4 - 1, base + 1, &clock(k), TEXT);
            }
        }
        let chg = (last.c - last.o) / last.o * 100.0;
        let head = format!("o {:.2}   h {:.2}   l {:.2}   c {:.2}   ", last.o, last.h, last.l, last.c);
        grid.text_at(0, 0, &head, TEXT);
        grid.text_at(head.len() as i64, 0, &format!("{}{chg:.2}%", if chg >= 0.0 { "+" } else { "" }), if chg >= 0.0 { UP } else { DOWN });
    }
}
