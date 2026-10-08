//! Sub-cell canvases: a bitmap finer than the grid, drawn one glyph a cell.
//!
//! | Type          | Dots a cell | Glyphs                               | Charset    |
//! |---------------|-------------|--------------------------------------|------------|
//! | [`Dots`]      | 2 × 4       | braille, U+2800–28FF                 | `Extended` |
//! | [`Sextants`]  | 2 × 3       | U+1FB00–1FB3B, plus `▌ ▐ █`          | `Extended` |
//! | [`Quadrants`] | 2 × 2       | U+2596–259F, plus `▀ ▄ ▌ ▐ █`        | `Basic`    |
//!
//! Draw dots with `set`, `line` and `circle` in dot coordinates, then put the
//! picture on a grid with [`Canvas::draw`]. An empty cell is a space, never
//! U+2800, so `Grid::text` and the contract's "nearly empty" check see it as
//! blank. A piece that colours each cell itself reads [`Canvas::glyph`] per
//! cell instead of calling `draw`.
//!
//! The encodings are in docs/IMPLEMENTATION_PLAN.md, Appendix C.

use crate::grid::bresenham;
use crate::{util::braille, Grid};
use std::marker::PhantomData;

/// How a cell's dots become a glyph. A cell's dots are a mask with bit
/// `row * W + col` set for each dot, rows top to bottom and columns left to
/// right, so a 2-wide cell has its top-left dot in bit 0 and top-right in bit 1.
pub trait Encoding {
    /// Dots across a cell.
    const W: usize;
    /// Dots down a cell.
    const H: usize;
    /// The glyph for a cell's dot mask. An empty mask is a space.
    fn glyph(mask: u8) -> char;
}

/// Braille: 2 × 4 dots a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Braille;

/// Sextants from Symbols for Legacy Computing: 2 × 3 dots a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sextant;

/// Quadrant block elements: 2 × 2 dots a cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quadrant;

impl Encoding for Braille {
    const W: usize = 2;
    const H: usize = 4;
    fn glyph(mask: u8) -> char {
        if mask == 0 {
            return ' ';
        }
        braille([mask & 3, (mask >> 2) & 3, (mask >> 4) & 3, (mask >> 6) & 3])
    }
}

impl Encoding for Sextant {
    const W: usize = 2;
    const H: usize = 3;
    fn glyph(mask: u8) -> char {
        // U+1FB00 counts up through the 64 masks but leaves out the four that
        // exist elsewhere: empty, the left column, the right column and full.
        let n = u32::from(mask & 63);
        match n {
            0 => ' ',
            21 => '▌',
            42 => '▐',
            63 => '█',
            _ => char::from_u32(0x1FB00 + n - 1 - u32::from(n > 21) - u32::from(n > 42)).expect("U+1FB00–1FB3B are scalars"),
        }
    }
}

impl Encoding for Quadrant {
    const W: usize = 2;
    const H: usize = 2;
    fn glyph(mask: u8) -> char {
        // Indexed by TL = 1, TR = 2, BL = 4, BR = 8.
        const TABLE: [char; 16] = [' ', '▘', '▝', '▀', '▖', '▌', '▞', '▛', '▗', '▚', '▐', '▜', '▄', '▙', '▟', '█'];
        TABLE[usize::from(mask & 15)]
    }
}

/// A bitmap of `E::W × E::H` dots per cell over a `cols × rows` cell area.
///
/// Coordinates are in dots, `(0, 0)` top left. Every drawing method clips, so
/// shapes may overhang the edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Canvas<E: Encoding> {
    cols: usize,
    rows: usize,
    /// One dot mask per cell, row-major.
    masks: Vec<u8>,
    encoding: PhantomData<E>,
}

/// Braille dots: 2 × 4 a cell, so a 40 × 10 grid is an 80 × 40 bitmap.
pub type Dots = Canvas<Braille>;
/// Sextants: 2 × 3 a cell.
pub type Sextants = Canvas<Sextant>;
/// Quadrants: 2 × 2 a cell, in glyphs every monospace font has.
pub type Quadrants = Canvas<Quadrant>;

impl<E: Encoding> Canvas<E> {
    /// An empty canvas covering `cols × rows` cells. Panics on a zero size.
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0 && rows > 0, "a canvas needs at least one cell");
        Self { cols, rows, masks: vec![0; cols * rows], encoding: PhantomData }
    }

    /// Width in cells.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Height in cells.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Width in dots.
    pub fn width(&self) -> usize {
        self.cols * E::W
    }

    /// Height in dots.
    pub fn height(&self) -> usize {
        self.rows * E::H
    }

    /// Every dot off.
    pub fn clear(&mut self) {
        self.masks.fill(0);
    }

    /// The cell index and bit for an on-canvas dot.
    #[inline]
    fn locate(&self, x: i64, y: i64) -> Option<(usize, u8)> {
        if x < 0 || y < 0 || x as usize >= self.width() || y as usize >= self.height() {
            return None;
        }
        let (x, y) = (x as usize, y as usize);
        Some(((y / E::H) * self.cols + x / E::W, 1 << ((y % E::H) * E::W + x % E::W)))
    }

    /// Turns one dot on. Off-canvas dots are ignored.
    #[inline]
    pub fn set(&mut self, x: i64, y: i64) {
        if let Some((i, bit)) = self.locate(x, y) {
            self.masks[i] |= bit;
        }
    }

    /// Turns one dot off. Off-canvas dots are ignored.
    #[inline]
    pub fn unset(&mut self, x: i64, y: i64) {
        if let Some((i, bit)) = self.locate(x, y) {
            self.masks[i] &= !bit;
        }
    }

    /// True if the dot is on. Off-canvas dots are off.
    #[inline]
    pub fn get(&self, x: i64, y: i64) -> bool {
        self.locate(x, y).is_some_and(|(i, bit)| self.masks[i] & bit != 0)
    }

    /// A straight line of dots, both ends included (Bresenham).
    pub fn line(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        bresenham(x0, y0, x1, y1, |x, y| self.set(x, y));
    }

    /// A circle's outline of radius `r` dots around `(cx, cy)` (midpoint
    /// algorithm). Radius 0 is the centre dot.
    pub fn circle(&mut self, cx: i64, cy: i64, r: i64) {
        let (mut x, mut y, mut err) = (r.max(0), 0, 1 - r.max(0));
        while x >= y {
            for (px, py) in [(x, y), (y, x), (-y, x), (-x, y), (-x, -y), (-y, -x), (y, -x), (x, -y)] {
                self.set(cx + px, cy + py);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// The dot mask of a cell (bit `row * E::W + col`). Panics off the canvas.
    pub fn mask(&self, cx: usize, cy: usize) -> u8 {
        assert!(cx < self.cols && cy < self.rows, "cell ({cx}, {cy}) is off a {}x{} canvas", self.cols, self.rows);
        self.masks[cy * self.cols + cx]
    }

    /// The glyph for a cell: a space when it has no dots. Panics off the canvas.
    pub fn glyph(&self, cx: usize, cy: usize) -> char {
        E::glyph(self.mask(cx, cy))
    }

    /// Puts the picture on `grid` with this canvas's top-left cell at `(x, y)`,
    /// in one colour. Cells with no dots are left as they were, so a canvas
    /// can be laid over text or another canvas. Clipped to the grid.
    pub fn draw(&self, grid: &mut Grid, x: i64, y: i64, color: u8) {
        for cy in 0..self.rows {
            for cx in 0..self.cols {
                let m = self.masks[cy * self.cols + cx];
                if m != 0 {
                    grid.set(x + cx as i64, y + cy as i64, E::glyph(m), color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_map_to_braille_code_points() {
        let cell = |dots: &[(i64, i64)]| {
            let mut c = Dots::new(1, 1);
            dots.iter().for_each(|&(x, y)| c.set(x, y));
            c.glyph(0, 0)
        };
        assert_eq!(cell(&[]), ' ');
        assert_eq!(cell(&[(0, 0)]), '\u{2801}'); // dot 1
        assert_eq!(cell(&[(1, 0)]), '\u{2808}'); // dot 4
        assert_eq!(cell(&[(0, 2)]), '\u{2804}'); // dot 3
        assert_eq!(cell(&[(0, 3), (1, 3)]), '\u{28C0}'); // dots 7 and 8
        assert_eq!(cell(&[(0, 0), (1, 1)]), '\u{2811}'); // dots 1 and 5
        let all: Vec<_> = (0..4).flat_map(|y| [(0, y), (1, y)]).collect();
        assert_eq!(cell(&all), '\u{28FF}');
    }

    #[test]
    fn sextants_skip_the_four_glyphs_that_live_elsewhere() {
        assert_eq!(Sextant::glyph(0), ' ');
        assert_eq!(Sextant::glyph(1), '\u{1FB00}'); // BLOCK SEXTANT-1
        assert_eq!(Sextant::glyph(20), '\u{1FB13}'); // BLOCK SEXTANT-35
        assert_eq!(Sextant::glyph(21), '▌');
        assert_eq!(Sextant::glyph(22), '\u{1FB14}'); // BLOCK SEXTANT-235
        assert_eq!(Sextant::glyph(42), '▐');
        assert_eq!(Sextant::glyph(62), '\u{1FB3B}'); // BLOCK SEXTANT-23456, the last
        assert_eq!(Sextant::glyph(63), '█');
        let mut seen: Vec<char> = (0..64).map(Sextant::glyph).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 64, "every mask has its own glyph");
    }

    #[test]
    fn sextant_dots_land_in_the_right_bits() {
        let mut c = Sextants::new(1, 1);
        c.set(0, 1); // middle left
        c.set(0, 2); // bottom left
        assert_eq!(c.mask(0, 0), 4 | 16);
        assert_eq!(c.glyph(0, 0), '\u{1FB13}');
    }

    #[test]
    fn quadrants_cover_all_sixteen_masks() {
        let want = [
            ' ', '\u{2598}', '\u{259D}', '\u{2580}', '\u{2596}', '\u{258C}', '\u{259E}', '\u{259B}', '\u{2597}', '\u{259A}', '\u{2590}', '\u{259C}',
            '\u{2584}', '\u{2599}', '\u{259F}', '\u{2588}',
        ];
        for (m, w) in want.iter().enumerate() {
            assert_eq!(Quadrant::glyph(m as u8), *w, "mask {m}");
        }
    }

    #[test]
    fn braille_covers_all_256_masks() {
        let mut seen: Vec<char> = (0..=255).map(Braille::glyph).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 256);
    }

    #[test]
    fn set_unset_get_and_clipping() {
        let mut c = Dots::new(2, 1);
        assert_eq!((c.width(), c.height()), (4, 4));
        c.set(2, 3);
        assert!(c.get(2, 3));
        assert_eq!(c.mask(1, 0), 1 << 6, "dot (2, 3) is the bottom-left of cell 1");
        c.unset(2, 3);
        assert!(!c.get(2, 3));
        c.set(-1, 0);
        c.set(4, 0);
        c.set(0, 4);
        assert_eq!(c, Dots::new(2, 1), "off-canvas dots are ignored");
        assert!(!c.get(-1, -1));
    }

    #[test]
    fn line_and_circle_draw_the_expected_glyphs() {
        let mut d = Dots::new(1, 1);
        d.line(0, 0, 1, 3);
        assert_eq!(d.glyph(0, 0), braille([1, 1, 2, 2]));

        // A radius-1 circle is a plus: four dots around the centre.
        let mut q = Quadrants::new(2, 2);
        q.circle(1, 1, 1);
        let mut g = Grid::new(2, 2);
        q.draw(&mut g, 0, 0, 0);
        assert_eq!(g.text(), "▞▖\n▝ ");

        let mut point = Quadrants::new(1, 1);
        point.circle(0, 0, 0);
        assert_eq!(point.glyph(0, 0), '▘');
    }

    #[test]
    fn draw_leaves_empty_cells_alone_and_clips() {
        let mut q = Quadrants::new(2, 1);
        q.set(0, 0);
        let mut g = Grid::new(3, 1);
        g.fill('.', 0);
        q.draw(&mut g, 1, 0, 7);
        assert_eq!(g.text(), ".▘.");
        assert_eq!(g.color_at(1, 0), 7);
        q.draw(&mut g, 2, 0, 1); // the second cell is off the grid and empty anyway
        assert_eq!(g.text(), ".▘▘");
        q.draw(&mut g, -1, 0, 1); // the inked cell is off the grid
        assert_eq!(g.text(), ".▘▘");
    }
}
