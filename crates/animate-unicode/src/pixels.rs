//! A picture of square pixels, two to a cell: what the scenes, and the skies
//! of landscape and sunrise, are painted into.
//!
//! A cell is twice as tall as it is wide, so an upper half block `▀` in one
//! colour over a back colour ([`Grid::set_back`]) is two square pixels of any
//! two colours. A piece paints [`Pixels`] in full colour, one `[r, g, b]` in
//! 0–1 a pixel, and [`Pixels::draw`] puts it on the grid. In colour each pixel
//! takes the palette colour nearest it, ordered-dithered so gradients don't
//! band. In one ink ([`Env::mono`]) there is one colour and no back, so each
//! pixel becomes 2 × 2 octant dots instead, as many lit as it is bright
//! (as it is dark on paper), so the picture still reads as a halftone.

use crate::subcell::{Encoding, Octant};
use crate::{Env, Grid, Meta};

/// A colour, each channel 0–1.
pub type Rgb = [f32; 3];

/// A 4 × 4 Bayer matrix, as thresholds in 0–1.
const BAYER: [f32; 16] = [0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0];

/// How far the colour dither moves a channel either way, so a pixel between
/// two palette colours takes each in proportion.
const SPREAD: f32 = 0.07;

/// `#rrggbb` as a colour; anything else is black.
pub fn hex(s: &str) -> Rgb {
    let n = u32::from_str_radix(s.trim_start_matches('#'), 16).unwrap_or(0);
    [((n >> 16) & 255) as f32 / 255.0, ((n >> 8) & 255) as f32 / 255.0, (n & 255) as f32 / 255.0]
}

/// A picture `cols` pixels wide and `2 × rows` tall, and the palette it is drawn in.
pub struct Pixels {
    w: usize,
    h: usize,
    px: Vec<Rgb>,
    palette: Vec<Rgb>,
    /// The nearest palette colour, filled in as asked for, on a 32-step cube (255: not yet).
    lut: Vec<u8>,
}

impl Pixels {
    /// A black picture the size of `meta`'s frame, in its palette (which it must have).
    pub fn new(meta: &Meta) -> Self {
        let palette = meta.palette.expect("a picture needs a palette").iter().map(|c| hex(c)).collect();
        Pixels { w: meta.cols, h: meta.rows * 2, px: vec![[0.0; 3]; meta.cols * meta.rows * 2], palette, lut: vec![255; 32 * 32 * 32] }
    }

    /// Width in pixels: the frame's columns.
    pub fn width(&self) -> usize {
        self.w
    }

    /// Height in pixels: twice the frame's rows.
    pub fn height(&self) -> usize {
        self.h
    }

    /// Sets an on-picture pixel; off it, nothing.
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, c: Rgb) {
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = c;
        }
    }

    /// An on-picture pixel's colour.
    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Rgb {
        self.px[y * self.w + x]
    }

    /// The pixels, row-major, for a hot loop.
    pub fn pixels_mut(&mut self) -> &mut [Rgb] {
        &mut self.px
    }

    /// The palette index nearest a colour, weighting green most as the eye
    /// does (ascii.rest's scenes weigh it the same).
    pub fn nearest(&mut self, c: Rgb) -> u8 {
        let q = |v: f32| ((v.clamp(0.0, 1.0) * 31.99) as usize).min(31);
        let k = (q(c[0]) << 10) | (q(c[1]) << 5) | q(c[2]);
        if self.lut[k] == 255 {
            let mut best = (0, f32::MAX);
            for (i, p) in self.palette.iter().enumerate() {
                let (dr, dg, db) = (p[0] - c[0], p[1] - c[1], p[2] - c[2]);
                let d = 0.3 * dr * dr + 0.5 * dg * dg + 0.2 * db * db;
                if d < best.1 {
                    best = (i, d);
                }
            }
            self.lut[k] = best.0 as u8;
        }
        self.lut[k]
    }

    /// The picture on the grid, every cell replaced: in colour as half blocks,
    /// in one ink as octant dots.
    pub fn draw(&mut self, grid: &mut Grid, env: &Env) {
        let rows = self.h / 2;
        if env.mono {
            for r in 0..rows {
                for x in 0..self.w {
                    let mut mask = 0u8;
                    for d in 0..8 {
                        let (dx, dy) = (d % 2, d / 2);
                        let c = self.px[(2 * r + dy / 2) * self.w + x];
                        // Lifted a little: dots on a dark page read darker than the colour they stand for.
                        let lum = (0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]).clamp(0.0, 1.0).powf(0.75);
                        let ink = if env.paper { 1.0 - lum } else { lum };
                        let (gx, gy) = (2 * x + dx, 4 * r + dy);
                        if ink > (BAYER[(gy & 3) * 4 + (gx & 3)] + 0.5) / 16.0 {
                            mask |= 1 << d;
                        }
                    }
                    grid.set(x as i64, r as i64, Octant::glyph(mask), 0);
                    grid.set_back(x as i64, r as i64, crate::NO_BACK);
                }
            }
            return;
        }
        for r in 0..rows {
            for x in 0..self.w {
                let mut at = |y: usize| {
                    let o = (BAYER[(y & 3) * 4 + (x & 3)] + 0.5) / 16.0 - 0.5;
                    let c = self.px[y * self.w + x];
                    self.nearest([c[0] + o * SPREAD, c[1] + o * SPREAD, c[2] + o * SPREAD])
                };
                let (top, bottom) = (at(2 * r), at(2 * r + 1));
                let (x, y) = (x as i64, r as i64);
                if top == bottom {
                    grid.set(x, y, '█', top);
                    grid.set_back(x, y, top);
                } else {
                    grid.set(x, y, '▀', top);
                    grid.set_back(x, y, bottom);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static META: Meta = Meta { name: "t", slug: "t", cols: 2, rows: 1, palette: Some(&["#000000", "#ffffff", "#ff0000"]), ..Meta::DEFAULT };

    #[test]
    fn two_pixels_a_cell_in_colour() {
        let mut p = Pixels::new(&META);
        p.set(0, 0, [1.0, 0.0, 0.0]);
        p.set(0, 1, [1.0, 1.0, 1.0]);
        p.set(1, 0, [0.0, 0.0, 0.0]);
        p.set(1, 1, [0.0, 0.0, 0.0]);
        let mut g = Grid::new(2, 1);
        p.draw(&mut g, &Env::default());
        assert_eq!((char::from_u32(g.get(0, 0)), g.color_at(0, 0), g.back_at(0, 0)), (Some('▀'), 2, 1));
        assert_eq!((char::from_u32(g.get(1, 0)), g.back_at(1, 0)), (Some('█'), 0));
    }

    #[test]
    fn in_one_ink_a_pixel_is_dots_as_bright_as_it() {
        let mut p = Pixels::new(&META);
        p.set(0, 0, [1.0, 1.0, 1.0]);
        p.set(0, 1, [1.0, 1.0, 1.0]);
        let mut g = Grid::new(2, 1);
        p.draw(&mut g, &Env { mono: true, ..Env::default() });
        assert_eq!((char::from_u32(g.get(0, 0)), char::from_u32(g.get(1, 0)), g.back_at(0, 0)), (Some('█'), Some(' '), crate::NO_BACK));
        p.draw(&mut g, &Env { mono: true, paper: true, ..Env::default() });
        assert_eq!((char::from_u32(g.get(0, 0)), char::from_u32(g.get(1, 0))), (Some(' '), Some('█')));
    }
}
