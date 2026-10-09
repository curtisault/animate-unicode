//! braille lissajous: a Lissajous figure traced in braille dots, its head
//! solid and its tail thinning out behind it, the phase between the two axes
//! drifting so the figure slowly turns over. Eight dots a cell: a 40×12 piece
//! is an 80×48 plot.
//!
//! The picture is a function of `t` alone: the tail is the curve's last stretch
//! of parameter, sampled fresh each frame, with a fixed hash deciding which
//! old dots have faded.

use crate::{Category, Charset, Dots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The braille lissajous's meta.
pub static META: Meta = Meta {
    name: "braille lissajous",
    slug: "braille-lissajous",
    category: Category::Generative,
    note: "a lissajous figure traced in braille, its tail thinning behind it",
    cols: 40,
    rows: 12,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"a": 3, "b": 2, "speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// How much of the curve's parameter the tail covers, in turns.
const TAIL: f64 = 0.85;
/// Samples along the tail: enough that neighbours are under a dot apart.
const SAMPLES: usize = 1400;

/// The figure: its frequencies and a dot canvas.
pub struct BrailleLissajous {
    a: f64,
    b: f64,
    speed: f64,
    dots: Dots,
}

/// A braille lissajous; reads `a` and `b` (the frequencies across and down)
/// and `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(BrailleLissajous {
        a: options.f64("a", 3.0).clamp(1.0, 12.0),
        b: options.f64("b", 2.0).clamp(1.0, 12.0),
        speed: options.f64("speed", 1.0),
        dots: Dots::new(META.cols, META.rows),
    })
}

/// A fixed pseudo-random fraction for sample `k`, so the same dots fade at
/// the same age in every frame and the tail does not shimmer.
fn grain(k: usize) -> f64 {
    let mut h = (k as u32).wrapping_mul(0x9E37_79B9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    f64::from(h & 0xFFFF) / 65_536.0
}

impl Piece for BrailleLissajous {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let t = t * self.speed;
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        // The phase between the axes drifts, turning the figure over.
        let phase = TAU * 0.25 + t * 0.21;
        let head = t * 0.09; // in turns of the parameter
        let point = |s: f64| {
            let u = s * TAU;
            let x = (w - 1.0) * 0.5 * (1.0 + 0.96 * (self.a * u + phase).sin());
            let y = (h - 1.0) * 0.5 * (1.0 + 0.92 * (self.b * u).sin());
            (x.round() as i64, y.round() as i64)
        };
        self.dots.clear();
        let mut prev = point(head);
        for k in 1..SAMPLES {
            let age = k as f64 / SAMPLES as f64;
            let p = point(head - age * TAIL);
            // The newest third is a solid line; after that a dot stays only
            // while its grain is under what is left of its life.
            let keep = age < 0.33 || grain(k) < (1.0 - age) * 1.4 - 0.2;
            if keep {
                if age < 0.33 {
                    self.dots.line(prev.0, prev.1, p.0, p.1);
                } else {
                    self.dots.set(p.0, p.1);
                }
            }
            prev = p;
        }
        grid.clear();
        self.dots.draw(grid, 0, 0, 0);
    }
}
