//! gauge: a semicircle dial, a band filled up to the reading, and a needle
//! that springs to each new value, overshoots, and settles. After
//! ascii.rest's gauge.ts (MIT, @bas3line).
//!
//! The readings and the needle's spring are ascii.rest's. In braille, four
//! dots to each of its half-block pixels, the rim, the ticks and the needle
//! are lines a dot wide; the band under the rim is coloured by zone, green,
//! then amber, then red near the top of the scale, solid up to the reading
//! and a faint trace beyond it.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The gauge's meta.
pub static META: Meta = Meta {
    name: "gauge",
    slug: "gauge",
    category: Category::Data,
    note: "a braille dial, its band green to amber to red, the needle springing",
    cols: 55,
    rows: 18,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"label": "load", "unit": "%", "min": 0, "max": 100}"#),
    // The band's zones, lit; the same, unlit; the rim and ticks; the needle; text.
    palette: Some(&["#4ac870", "#f0b040", "#e85050", "#1e4a2c", "#5a4418", "#5a2020", "#8a909c", "#ffffff", "#c8ccd4"]),
    ground: Some("#0c0d11"),
    ..Meta::DEFAULT
};

const RIM: u8 = 6;
const NEEDLE: u8 = 7;
const TEXT: u8 = 8;
/// Seconds per reading, the spring's damping (about a quarter overshoot)
/// and natural frequency in rad/s.
const HOLD: f64 = 2.6;
const ZETA: f64 = 0.38;
const W0: f64 = 5.2;
/// Rim radius in ascii.rest's pixels (a cell wide, half a cell tall): two dots each.
const R: f64 = 22.0;

/// The gauge: its scale and label, the readings so far and the spring's
/// state as each began, its generator, and the dots.
pub struct Gauge {
    min: f64,
    max: f64,
    label: String,
    unit: String,
    targets: Vec<f64>,
    states: Vec<(f64, f64)>,
    rand: Mulberry32,
    dots: ColorDots,
}

/// The spring from (x0, v0) toward T, s seconds on: value and velocity.
fn spring(x0: f64, v0: f64, target: f64, s: f64) -> (f64, f64) {
    let a = ZETA * W0;
    let w = W0 * (1.0 - ZETA * ZETA).sqrt();
    let (aa, bb) = (x0 - target, (v0 + a * (x0 - target)) / w);
    let (e, c, n) = ((-a * s).exp(), (w * s).cos(), (w * s).sin());
    (target + e * (aa * c + bb * n), e * (-a * (aa * c + bb * n) + w * (-aa * n + bb * c)))
}

impl Gauge {
    fn value(&mut self, t: f64) -> f64 {
        let k = (t / HOLD).floor().max(0.0) as usize;
        while self.targets.len() <= k {
            let last = *self.targets.last().unwrap();
            let mut next = 12.0 + self.rand.next_f64() * 76.0;
            while (next - last).abs() < 22.0 {
                next = 12.0 + self.rand.next_f64() * 76.0;
            }
            let &(x, v) = self.states.last().unwrap();
            self.states.push(spring(x, v, last, HOLD));
            self.targets.push(next);
        }
        let (x, v) = self.states[k];
        spring(x, v, self.targets[k], t - k as f64 * HOLD).0
    }
}

/// A gauge; reads `label`, `unit`, `min` and `max`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(Gauge {
        min: options.f64("min", 0.0),
        max: options.f64("max", 100.0),
        label: options.string("label", "load"),
        unit: options.string("unit", "%"),
        targets: vec![55.0],
        states: vec![(30.0, 0.0)],
        rand: Mulberry32::new(42),
        dots: ColorDots::new(META.cols, META.rows),
    })
}

impl Piece for Gauge {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let v = self.value(t + HOLD - 0.4).clamp(-1.0, 101.0);
        let angle = |v: f64| PI * (1.0 - v / 100.0);
        let an = angle(v);
        // The pivot in dots: ascii.rest's pixel (27, 27), two dots a pixel.
        let (cx, cy) = (55.0, 55.0);
        let at = |r: f64, a: f64| ((cx + r * a.cos()).round() as i64, (cy - r * a.sin()).round() as i64);
        self.dots.clear();
        // The band under the rim, by zone, lit up to the reading.
        let (r0, r1) = ((R - 4.1) * 2.0, (R - 2.1) * 2.0);
        for y in 0..=cy as i64 {
            for x in 0..self.dots.width() as i64 {
                let (dx, dy) = (x as f64 + 0.5 - cx, cy - (y as f64 + 0.5));
                let d = (dx * dx + dy * dy).sqrt();
                if dy < -1.0 || d <= r0 || d >= r1 {
                    continue;
                }
                let a = dy.max(0.0).atan2(dx);
                let at_v = (1.0 - a / PI) * 100.0;
                let zone = if at_v < 60.0 {
                    0
                } else if at_v < 80.0 {
                    1
                } else {
                    2
                };
                if a >= an - 0.02 {
                    self.dots.set(x, y, zone);
                } else if dither(x as usize, y as usize, 0.25) {
                    self.dots.set(x, y, zone + 3);
                }
            }
        }
        // The rim, a tick every 10, and the needle with its hub.
        let mut a = 0.0;
        while a <= PI {
            let (x, y) = at(R * 2.0, a);
            self.dots.set(x, y, RIM);
            a += 0.005;
        }
        for k in 0..=10 {
            let a = angle(k as f64 * 10.0);
            for r in [(R - 8.0) * 2.0, (R - 7.5) * 2.0, (R - 7.0) * 2.0] {
                let (x, y) = at(r, a);
                self.dots.set(x, y, RIM);
            }
        }
        let mut r = 3.0;
        while r <= (R - 6.0) * 2.0 {
            let (x, y) = at(r, an);
            self.dots.set(x, y, NEEDLE);
            r += 0.4;
        }
        for dy in -3i64..=3 {
            for dx in -3i64..=3 {
                if dx * dx + dy * dy <= 9 {
                    self.dots.set(cx as i64 + dx, cy as i64 + dy, RIM);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // Numbers every 20 just outside the rim; the two ends sit under it.
        for k in 0..=5 {
            let v20 = k as f64 * 20.0;
            let a = angle(v20);
            let s = format!("{}", (self.min + (self.max - self.min) * v20 / 100.0).round() as i64);
            let ends = k == 0 || k == 5;
            let rr = if ends { R } else { R + 2.5 };
            let x = 27.0 + rr * a.cos() + if ends { 0.0 } else { s.len() as f64 / 2.0 * a.cos() };
            let y = if ends { 27.0 + 3.0 } else { 27.0 - (rr + 1.0) * a.sin() };
            grid.text_at((x - (s.len() as f64 - 1.0) / 2.0).round() as i64, ((y + 0.5) / 2.0).floor() as i64, &s, TEXT);
        }
        let shown = (self.min + (self.max - self.min) * v.clamp(0.0, 100.0) / 100.0).round() as i64;
        let reading = if self.unit.is_empty() || self.unit.starts_with('%') || self.unit.starts_with('°') {
            format!("{shown}{}", self.unit)
        } else {
            format!("{shown} {}", self.unit)
        };
        let centre = |s: &str| (27.0 + 0.5 - s.chars().count() as f64 / 2.0).round() as i64;
        grid.text_at(centre(&reading), 15, &reading, NEEDLE);
        grid.text_at(centre(&self.label), 16, &self.label, TEXT);
    }
}
