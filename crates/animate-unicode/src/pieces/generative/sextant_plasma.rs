//! sextant plasma: the old demoscene plasma, a sum of moving sine waves, drawn
//! as its contour lines. Each cell is six sub-cells (2 × 3, the sextants of
//! Symbols for Legacy Computing, U+1FB00 on), and a sub-cell is lit where the
//! field crosses from one band into the next, so the lines are three times
//! finer down a cell than a glyph could draw them. Each line takes its band's
//! colour.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece, Sextants};

/// The sextant plasma's meta.
pub static META: Meta = Meta {
    name: "sextant plasma",
    slug: "sextant-plasma",
    category: Category::Generative,
    note: "a plasma field drawn as contour lines in sextants, coloured by band",
    cols: 60,
    rows: 18,
    fps: 30,
    charset: Charset::Extended,
    palette: Some(&["#3b1f6e", "#5a2a8c", "#8a2f9c", "#b8378f", "#df4a74", "#f3685a", "#fb8f44", "#f9b93a", "#e9e051", "#a8e870", "#5fd89a", "#36b7c0"]),
    ground: Some("#0e0b16"),
    options: Some(r#"{"bands": 12, "speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// The plasma: band count, speed, and the field and canvas, kept between frames.
pub struct SextantPlasma {
    bands: f64,
    speed: f64,
    field: Vec<u8>,
    lines: Sextants,
}

/// A sextant plasma; reads `bands` (2–12, one colour each) and `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (w, h) = (META.cols * 2, META.rows * 3);
    Box::new(SextantPlasma {
        bands: options.f64("bands", 12.0).round().clamp(2.0, 12.0),
        speed: options.f64("speed", 1.0),
        field: vec![0; w * h],
        lines: Sextants::new(META.cols, META.rows),
    })
}

impl Piece for SextantPlasma {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let t = t * self.speed;
        let (w, h) = (self.lines.width(), self.lines.height());
        // A sub-cell is half a cell wide and a third tall, and a cell is twice
        // as tall as wide, so a sub-cell is 0.5 × 0.667: scale y to keep the
        // waves round.
        let (cx, cy) = (w as f64 / 2.0 + 18.0 * (t * 0.31).sin(), h as f64 / 2.0 * 1.333 + 10.0 * (t * 0.23).cos());
        for y in 0..h {
            for x in 0..w {
                let (fx, fy) = (x as f64, y as f64 * 1.333);
                let v = (fx / 9.0 + t * 0.9).sin()
                    + ((fy / 7.0 - t * 0.6).sin() + (fx / 13.0).cos()).sin()
                    + ((fx + fy) / 15.0 + t * 0.7).sin()
                    + (((fx - cx).powi(2) + (fy - cy).powi(2)).sqrt() / 8.0 - t * 1.3).sin();
                // v is in -4..4; bands across that range.
                let band = (((v + 4.0) / 8.0) * self.bands).floor().clamp(0.0, self.bands - 1.0);
                self.field[y * w + x] = band as u8;
            }
        }
        // A sub-cell is on a line where its band differs from the one to its
        // right or below: four-connected lines, one sub-cell thick.
        self.lines.clear();
        let mut colour = vec![0u8; META.cols * META.rows];
        for y in 0..h {
            for x in 0..w {
                let b = self.field[y * w + x];
                let right = x + 1 < w && self.field[y * w + x + 1] != b;
                let below = y + 1 < h && self.field[(y + 1) * w + x] != b;
                if right || below {
                    self.lines.set(x as i64, y as i64);
                    let c = &mut colour[(y / 3) * META.cols + x / 2];
                    *c = (*c).max(b);
                }
            }
        }
        grid.clear();
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let g = self.lines.glyph(cx, cy);
                if g != ' ' {
                    grid.set(cx as i64, cy as i64, g, colour[cy * META.cols + cx]);
                }
            }
        }
    }
}
