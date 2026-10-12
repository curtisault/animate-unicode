//! epicycles: a five-pointed star as a Fourier series. Each circle turns at a
//! whole multiple of the first while riding the rim of the one before, and
//! the tip of the chain traces the star. After ascii.rest's epicycles.ts
//! (MIT, @bas3line).
//!
//! In braille the trace is a true curve a dot wide over a 120×96 picture,
//! fading from gold at the tip to embers at its tail. The first circles of
//! the chain show their rims and arms, dotted, each in its own colour, so the
//! eye can follow which wheel carries which.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{PI, TAU};

/// The epicycles' meta.
pub static META: Meta = Meta {
    name: "epicycles",
    slug: "epicycles",
    category: Category::Generative,
    note: "turning circles in braille, each its own colour, tracing a star",
    cols: 60,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // The trail, oldest first; then one colour for each circle shown; the tip.
    palette: Some(&["#5a2a0c", "#9a4e12", "#d88a1c", "#ffd25a", "#ff6b8b", "#5ec8ff", "#8dff8a", "#c79bff", "#ffb35a", "#ffffff"]),
    ground: Some("#0b0a10"),
    ..Meta::DEFAULT
};

/// Circles in the chain: every term a five-fold star has up to k = 39.
const TERMS: usize = 16;
/// Seconds to trace the figure once.
const PERIOD: f64 = 10.0;
/// Share of the figure kept behind the tip; past FADE of that it thins out.
const TRAIL: f64 = 0.9;
const FADE: f64 = 0.8;
/// Points along one trace.
const SAMPLES: usize = 2400;
/// Circles, from the largest, that show their rims and arms.
const SHOWN: usize = 5;
/// Where on the figure the tip is at t = 0: along the left arm.
const PHASE: f64 = 0.12;
const AGE: Ramp = Ramp::new(0, 4);
const WHEEL: u8 = 4;
const TIP: u8 = 9;

/// One term of the series: a circle of radius `r` turning `k` times a trace.
#[derive(Clone, Copy)]
struct Term {
    k: f64,
    re: f64,
    im: f64,
    r: f64,
}

/// The chain, the trace in dots, the screen transform, and the dots.
pub struct Epicycles {
    chain: Vec<Term>,
    path: Vec<(f64, f64)>,
    scale: f64,
    mid: (f64, f64),
    dots: ColorDots,
}

/// The star's outline, sampled evenly by length, with y up.
fn star(n: usize) -> Vec<(f64, f64)> {
    let v: Vec<(f64, f64)> = (0..10)
        .map(|j| {
            let a = PI / 2.0 + j as f64 * PI / 5.0;
            let r = if j % 2 == 1 { 0.4 } else { 1.0 };
            (r * a.cos(), r * a.sin())
        })
        .collect();
    (0..n)
        .map(|i| {
            let d = i as f64 / n as f64 * 10.0;
            let (j, f) = (d.floor() as usize, d.fract());
            let (a, b) = (v[j], v[(j + 1) % 10]);
            (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
        })
        .collect()
}

impl Epicycles {
    /// The tip of the first `upto` circles at angle `theta`.
    fn sum(&self, theta: f64, upto: usize) -> (f64, f64) {
        self.chain[..upto].iter().fold((0.0, 0.0), |(x, y), t| {
            let (s, c) = (t.k * theta).sin_cos();
            (x + t.re * c - t.im * s, y + t.re * s + t.im * c)
        })
    }

    fn to_dot(&self, (x, y): (f64, f64)) -> (f64, f64) {
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        (w / 2.0 + (x - self.mid.0) * self.scale, h / 2.0 - (y - self.mid.1) * self.scale)
    }
}

/// Epicycles; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // Discrete Fourier transform: every term is a circle of radius |c| turning
    // k times per trace. Keep the biggest, largest first.
    let pts = star(500);
    let n = pts.len() as f64;
    let mut terms: Vec<Term> = (-40..=40)
        .map(|k| {
            let (mut re, mut im) = (0.0, 0.0);
            for (i, &(x, y)) in pts.iter().enumerate() {
                let (s, c) = (-TAU * k as f64 * i as f64 / n).sin_cos();
                re += x * c - y * s;
                im += x * s + y * c;
            }
            Term { k: k as f64, re: re / n, im: im / n, r: (re * re + im * im).sqrt() / n }
        })
        .collect();
    terms.sort_by(|a, b| b.r.total_cmp(&a.r));
    terms.truncate(TERMS);
    let dots = ColorDots::new(META.cols, META.rows);
    let mut e = Epicycles { chain: terms, path: vec![], scale: 1.0, mid: (0.0, 0.0), dots };
    // Fit the traced figure to the frame.
    let raw: Vec<(f64, f64)> = (0..SAMPLES).map(|i| e.sum(i as f64 / SAMPLES as f64 * TAU, TERMS)).collect();
    let (x0, x1) = (raw.iter().map(|p| p.0).fold(f64::MAX, f64::min), raw.iter().map(|p| p.0).fold(f64::MIN, f64::max));
    let (y0, y1) = (raw.iter().map(|p| p.1).fold(f64::MAX, f64::min), raw.iter().map(|p| p.1).fold(f64::MIN, f64::max));
    e.scale = ((e.dots.width() as f64 - 16.0) / (x1 - x0)).min((e.dots.height() as f64 - 9.0) / (y1 - y0));
    e.mid = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    e.path = raw.iter().map(|&p| e.to_dot(p)).collect();
    Box::new(e)
}

impl Piece for Epicycles {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let u = (t / PERIOD + PHASE).rem_euclid(1.0);
        let head = u * SAMPLES as f64;
        let theta = u * TAU;
        self.dots.clear();
        // The trace behind the tip, oldest first so the newest lies on top;
        // past FADE it thins out by dithering.
        for back in (0..(SAMPLES as f64 * TRAIL) as usize).rev() {
            let i = (head as usize + SAMPLES - back) % SAMPLES;
            let age = back as f64 / SAMPLES as f64 / TRAIL;
            let (x, y) = self.path[i];
            let (xi, yi) = (x.floor() as i64, y.floor() as i64);
            let keep = if age < FADE { 1.0 } else { 1.0 - (age - FADE) / (1.0 - FADE) };
            if xi >= 0 && yi >= 0 && crate::util::dither(xi as usize, yi as usize, keep) {
                self.dots.set(xi, yi, AGE.at(1.0 - age));
            }
        }
        // The first circles: each rim dotted round its centre, and its arm,
        // in the circle's colour, never over the trace.
        let joints: Vec<(f64, f64)> = (0..=TERMS).map(|j| self.to_dot(self.sum(theta, j))).collect();
        let joint = |j: usize| joints[j];
        for j in 0..SHOWN {
            let (cx, cy) = joint(j);
            let (nx, ny) = joint(j + 1);
            let r = self.chain[j].r * self.scale;
            let color = WHEEL + j as u8;
            let steps = (r * TAU / 3.0).ceil().max(8.0) as usize;
            for s in 0..steps {
                let (sa, ca) = (s as f64 / steps as f64 * TAU).sin_cos();
                let (x, y) = ((cx + r * ca).floor() as i64, (cy + r * sa).floor() as i64);
                if self.dots.get(x, y).is_none() {
                    self.dots.set(x, y, color);
                }
            }
            let len = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();
            let n = (len / 2.0).ceil() as usize;
            for s in 1..n {
                let f = s as f64 / n as f64;
                let (x, y) = ((cx + (nx - cx) * f).floor() as i64, (cy + (ny - cy) * f).floor() as i64);
                if self.dots.get(x, y).is_none() {
                    self.dots.set(x, y, color);
                }
            }
        }
        // The centre, and the tip as a knot of dots.
        let (cx, cy) = joint(0);
        for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
            self.dots.set(cx.floor() as i64 + dx, cy.floor() as i64 + dy, WHEEL);
        }
        let (tx, ty) = joint(TERMS);
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1), (2, 0), (0, 2), (1, -1), (-1, 1)] {
            self.dots.set(tx.floor() as i64 + dx, ty.floor() as i64 + dy, TIP);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
