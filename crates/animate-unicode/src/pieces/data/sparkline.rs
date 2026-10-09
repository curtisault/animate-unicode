//! sparkline: live metrics as sparklines, scrolling left as new samples
//! arrive, each with its label, its latest value and the span's range.
//! After ascii.rest's sparkline.ts (MIT, @bas3line).
//!
//! The signals are ascii.rest's, from its seeded generators. Each line is a
//! braille area chart two rows tall: eight dots of height, and two samples to
//! a character, so the same width holds twice the history at twice the
//! resolution of a row of blocks. Each series has its own colour, its latest
//! value bright at the right.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// The sparklines' meta.
pub static META: Meta = Meta {
    name: "sparkline",
    slug: "sparkline",
    category: Category::Data,
    note: "three live braille sparklines, two samples a cell, values at right",
    cols: 72,
    rows: 11,
    fps: 12,
    charset: Charset::Extended,
    options: Some(
        r#"{"series": [{"label": "cpu", "unit": "%", "lo": 0, "hi": 100}, {"label": "memory", "unit": " gb", "lo": 1.2, "hi": 3.4, "digits": 1}, {"label": "requests", "unit": "/s", "lo": 0, "hi": 260}], "range": true, "rate": 4}"#,
    ),
    // Three series colours; labels and axis; values.
    palette: Some(&["#4ac8e8", "#c088f0", "#7ad87a", "#6a707c", "#f0f2f6"]),
    ground: Some("#0c0d11"),
    ..Meta::DEFAULT
};

const LABEL: u8 = 3;
const VALUE: u8 = 4;
/// Label, value and min/max widths; series shown (each takes two rows and a gap).
const LW: i64 = 10;
const VW: usize = 7;
const RW: usize = 5;
const MAX: usize = 3;

/// A series: its label, unit, fixed scale (or none, to fit what is shown), and decimals.
struct Series {
    label: String,
    unit: String,
    lo: Option<f64>,
    hi: Option<f64>,
    digits: usize,
}

/// A sample signal's state.
#[derive(Clone)]
struct Signal {
    v: f64,
    busy: u32,
    cap: f64,
    n: f64,
}

/// Sample signal i, 0 to 1, from its own state and generator.
fn sample(i: usize, s: &mut Signal, r: &mut Mulberry32) -> f64 {
    match i % 3 {
        // Mostly idle, with bursts of work that come and go.
        0 => {
            if s.busy > 0 {
                s.busy -= 1;
            } else if r.next_f64() < 0.06 {
                s.busy = 4 + (r.next_f64() * 9.0) as u32;
            }
            s.v += ((if s.busy > 0 { 0.78 } else { 0.22 }) - s.v) * 0.55 + (r.next_f64() - 0.5) * 0.3;
            s.v = s.v.clamp(0.02, 0.99);
            s.v
        }
        // A heap that fills unevenly and falls back when it is collected.
        1 => {
            s.v += 0.01 + r.next_f64() * 0.035;
            if s.v > s.cap {
                s.v = 0.1 + r.next_f64() * 0.14;
                s.cap = 0.73 + r.next_f64() * 0.22;
            }
            s.v
        }
        // A slow swell in traffic with jitter on every sample.
        _ => {
            let base = 0.48 + 0.27 * (s.n / 14.0).sin();
            s.n += 1.0;
            s.v += (base - s.v) * 0.7 + (r.next_f64() + r.next_f64() - 1.0) * 0.23;
            s.v = s.v.clamp(0.015, 1.0);
            s.v
        }
    }
}

/// The sparklines: their series, whether the range shows, the sample rate,
/// the samples on screen, the rings of samples and their generators.
pub struct Sparkline {
    series: Vec<Series>,
    range: bool,
    rate: f64,
    w: usize,
    made: usize,
    rings: Vec<Vec<f64>>,
    states: Vec<Signal>,
    rands: Vec<Mulberry32>,
    dots: ColorDots,
}

impl Sparkline {
    fn reset(&mut self) {
        self.made = 0;
        self.rings = vec![vec![0.0; self.w]; self.series.len()];
        self.rands = (0..self.series.len()).map(|i| Mulberry32::new(911 + i as u32 * 7919)).collect();
        self.states = vec![Signal { v: 0.4, busy: 0, cap: 0.82, n: 0.0 }; self.series.len()];
    }

    fn fill(&mut self, need: usize) {
        if need < self.made {
            self.reset();
        }
        while self.made < need {
            for i in 0..self.series.len() {
                let v = sample(i, &mut self.states[i], &mut self.rands[i]);
                self.rings[i][self.made % self.w] = v;
            }
            self.made += 1;
        }
    }
}

/// Sparklines; read `series` (each `{label, unit, lo, hi, digits}`), `range` and `rate` (samples a second).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let series: Vec<Series> = options
        .json("series")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .take(MAX)
                .map(|s| Series {
                    label: s.get("label").and_then(|v| v.as_str()).unwrap_or("").chars().take(LW as usize - 1).collect(),
                    unit: s.get("unit").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    lo: s.get("lo").and_then(|v| v.as_f64()),
                    hi: s.get("hi").and_then(|v| v.as_f64()),
                    digits: s.get("digits").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                })
                .collect()
        })
        .unwrap_or_default();
    let range = options.bool("range", true);
    let rate = options.f64("rate", 4.0).max(0.05);
    // Two samples a character, across what the labels, values and range leave.
    let cells = META.cols - 2 - LW as usize - 2 - VW - if range { 2 + RW + 1 + RW } else { 0 } - 2;
    let mut s =
        Sparkline { series, range, rate, w: cells * 2, made: 0, rings: vec![], states: vec![], rands: vec![], dots: ColorDots::new(META.cols, META.rows) };
    s.reset();
    Box::new(s)
}

impl Piece for Sparkline {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        self.fill(self.w * 2 + (t * self.rate).floor() as usize);
        let x0 = 2 + LW;
        let cells = self.w as i64 / 2;
        let n = self.series.len();
        // The header, the series (two rows each, a row apart) and the time axis, centred.
        let span = (n.max(1) * 3 - 1) as i64;
        let top = (META.rows as i64 - span - 2) / 2 + 1;
        grid.clear();
        self.dots.clear();
        let fmt = |s: &Series, v: f64, unit: bool| format!("{:.*}{}", s.digits, v, if unit { s.unit.as_str() } else { "" });
        if self.range {
            let x = x0 + cells + 2 + VW as i64 + 2;
            grid.text_at(x, top - 1, &format!("{:>RW$} {:>RW$}", "min", "max"), LABEL);
        }
        let ago = format!("{}s ago", (self.w as f64 / self.rate).round());
        grid.text_at(x0, top + span, &ago, LABEL);
        grid.text_at(x0 + cells - 3, top + span, "now", LABEL);
        for (i, s) in self.series.iter().enumerate() {
            let y = top + 3 * i as i64;
            let (lo, hi) = (s.lo.unwrap_or(0.0), s.hi.unwrap_or(100.0));
            // What is on screen, oldest first, in the series' own units.
            let win: Vec<f64> = (0..self.w).map(|j| lo + self.rings[i][(self.made + j) % self.w] * (hi - lo)).collect();
            let (min, max) = win.iter().fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
            let d = (hi - lo).max(1e-9);
            // An area eight dots tall, a sample a dot column, rising from the floor.
            for (j, &v) in win.iter().enumerate() {
                let level = (((v - lo) / d * 7.0).round() as i64).clamp(0, 7) + 1;
                for k in 0..level {
                    self.dots.set(x0 * 2 + j as i64, (y + 2) * 4 - 1 - k, i as u8);
                }
            }
            grid.text_at(2, y + 1, &s.label, LABEL);
            grid.text_at(x0 + cells, y + 1, "▏", LABEL);
            grid.text_at(x0 + cells + 2, y + 1, &format!("{:>VW$}", fmt(s, *win.last().unwrap_or(&0.0), true)), VALUE);
            if self.range {
                let x = x0 + cells + 2 + VW as i64 + 2;
                grid.text_at(x, y + 1, &format!("{:>RW$} {:>RW$}", fmt(s, min, false), fmt(s, max, false)), LABEL);
            }
        }
        self.dots.draw(grid, 0, 0);
    }
}
