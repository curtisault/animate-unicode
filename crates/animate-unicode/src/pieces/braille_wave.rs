//! braille wave: two sine waves drawn with braille dots, eight to a cell, so a
//! 40×10 piece is an 80×40 plot. The first unicode-only piece: nothing here
//! could be drawn in ASCII at this resolution.

use crate::{util::braille, Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

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

pub struct BrailleWave {
    speed: f64,
    /// One byte per dot: the plot, 2·cols wide and 4·rows tall.
    dots: Vec<u8>,
}

pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(BrailleWave { speed: options.f64("speed", 1.0), dots: vec![0; META.cols * 2 * META.rows * 4] })
}

impl Piece for BrailleWave {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (META.cols * 2, META.rows * 4);
        self.dots.fill(0);
        let t = t * self.speed;
        // Two waves of different frequency, each plotted as a solid vertical run between
        // neighbouring columns so the line never breaks where it is steep.
        for (k, (freq, amp, phase)) in [(2.0, 0.42, 0.0), (3.0, 0.28, 1.7)].into_iter().enumerate() {
            let wave = |x: usize| (h as f64 / 2.0) * (1.0 - amp * ((x as f64 / w as f64) * freq * TAU + phase + t * (1.5 + k as f64)).sin());
            for x in 0..w {
                let (y, py) = (wave(x), wave(x.saturating_sub(1)));
                let lo = y.min(py).floor().max(0.0) as usize;
                let hi = y.max(py).floor().min(h as f64 - 1.0) as usize;
                for yy in lo..=hi {
                    self.dots[yy * w + x] = 1;
                }
            }
        }
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let mut cell = [0u8; 4];
                for (r, d) in cell.iter_mut().enumerate() {
                    let row = (cy * 4 + r) * w + cx * 2;
                    *d = self.dots[row] | (self.dots[row + 1] << 1);
                }
                let ch = if cell == [0; 4] { ' ' } else { braille(cell) };
                grid.put(cx as i64, cy as i64, ch);
            }
        }
    }
}
