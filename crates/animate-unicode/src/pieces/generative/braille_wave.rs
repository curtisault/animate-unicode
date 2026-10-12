//! braille wave: two sine waves drawn with braille dots, eight to a cell, so a
//! 40×10 piece is an 80×40 plot. The first unicode-only piece: nothing here
//! could be drawn in ASCII at this resolution.

use crate::{Category, Charset, Dots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The braille wave's meta.
pub static META: Meta = Meta {
    name: "braille wave",
    slug: "braille-wave",
    category: Category::Generative,
    note: "two sine waves plotted in braille dots, eight to a cell",
    cols: 40,
    rows: 10,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// Two sine waves in braille dots.
pub struct BrailleWave {
    speed: f64,
    /// The plot, 2·cols dots wide and 4·rows tall.
    dots: Dots,
}

/// A braille wave; reads `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(BrailleWave { speed: options.f64("speed", 1.0), dots: Dots::new(META.cols, META.rows) })
}

impl Piece for BrailleWave {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        self.dots.clear();
        let t = t * self.speed;
        // Two waves of different frequency, each plotted as a solid vertical run between
        // neighbouring columns so the line never breaks where it is steep.
        for (k, (freq, amp, phase)) in [(2.0, 0.42, 0.0), (3.0, 0.28, 1.7)].into_iter().enumerate() {
            let wave = |x: usize| (h as f64 / 2.0) * (1.0 - amp * ((x as f64 / w as f64) * freq * TAU + phase + t * (1.5 + k as f64)).sin());
            for x in 0..w {
                let (y, py) = (wave(x), wave(x.saturating_sub(1)));
                let lo = y.min(py).floor().max(0.0) as i64;
                let hi = y.max(py).floor().min(h as f64 - 1.0) as i64;
                for yy in lo..=hi {
                    self.dots.set(x as i64, yy);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0, 0);
    }
}
