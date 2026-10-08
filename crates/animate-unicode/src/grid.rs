//! The frame buffer: `cols × rows` cells, each a code point and a palette index.
//!
//! Cells are `u32` code points rather than a string so that an astral glyph
//! (sextants at U+1FB00, say) is one cell like any other, and so JS can read
//! the buffer straight out of wasm memory as a `Uint32Array` with no copying
//! or decoding per frame.

/// A space: what `clear` fills the grid with.
pub const BLANK: u32 = 0x20;

/// The right half of a double-width glyph. The cell to its left holds the
/// glyph; renderers skip this one. Never a real code point.
pub const WIDE_TAIL: u32 = 0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    cols: usize,
    rows: usize,
    cells: Vec<u32>,
    colors: Vec<u8>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0 && rows > 0, "a grid needs at least one cell");
        Self { cols, rows, cells: vec![BLANK; cols * rows], colors: vec![0; cols * rows] }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Every cell a space in colour 0. Most pieces start a frame with this.
    pub fn clear(&mut self) {
        self.cells.fill(BLANK);
        self.colors.fill(0);
    }

    /// Every cell the given glyph and colour: a scene's sky, say.
    pub fn fill(&mut self, ch: char, color: u8) {
        self.cells.fill(ch as u32);
        self.colors.fill(color);
    }

    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.cols && y < self.rows, "({x}, {y}) is off a {}x{} grid", self.cols, self.rows);
        y * self.cols + x
    }

    #[inline]
    pub fn in_bounds(&self, x: i64, y: i64) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.cols && (y as usize) < self.rows
    }

    /// One glyph in colour 0. Off-grid coordinates are ignored, so a piece can
    /// draw a shape that reaches past the edge without checking each cell.
    #[inline]
    pub fn put(&mut self, x: i64, y: i64, ch: char) {
        self.set(x, y, ch, 0);
    }

    /// One glyph in a palette colour. Off-grid coordinates are ignored.
    #[inline]
    pub fn set(&mut self, x: i64, y: i64, ch: char, color: u8) {
        if self.in_bounds(x, y) {
            let i = self.index(x as usize, y as usize);
            self.cells[i] = ch as u32;
            self.colors[i] = color;
        }
    }

    /// A double-width glyph (CJK, most emoji) at `x`, with its tail at `x + 1`.
    /// Drawn only when both cells are on the grid.
    pub fn set_wide(&mut self, x: i64, y: i64, ch: char, color: u8) {
        if self.in_bounds(x, y) && self.in_bounds(x + 1, y) {
            let i = self.index(x as usize, y as usize);
            self.cells[i] = ch as u32;
            self.cells[i + 1] = WIDE_TAIL;
            self.colors[i] = color;
            self.colors[i + 1] = color;
        }
    }

    /// A string along a row, one glyph per cell, clipped to the grid.
    pub fn text_at(&mut self, x: i64, y: i64, s: &str, color: u8) {
        for (k, ch) in s.chars().enumerate() {
            self.set(x + k as i64, y, ch, color);
        }
    }

    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.cells[self.index(x, y)]
    }

    pub fn color_at(&self, x: usize, y: usize) -> u8 {
        self.colors[self.index(x, y)]
    }

    /// Row-major code points, `WIDE_TAIL` included: what the wasm shell reads.
    pub fn cells(&self) -> &[u32] {
        &self.cells
    }

    /// Row-major palette indices, parallel to `cells`.
    pub fn colors(&self) -> &[u8] {
        &self.colors
    }

    pub fn cells_mut(&mut self) -> &mut [u32] {
        &mut self.cells
    }

    pub fn colors_mut(&mut self) -> &mut [u8] {
        &mut self.colors
    }

    /// The frame as text: `rows` lines joined by `\n`, wide tails left out so a
    /// terminal or a `<pre>` lays the glyphs out at their real widths.
    pub fn text(&self) -> String {
        let mut s = String::with_capacity(self.cells.len() + self.rows);
        for (r, row) in self.cells.chunks(self.cols).enumerate() {
            if r > 0 {
                s.push('\n');
            }
            for &cp in row {
                if cp != WIDE_TAIL {
                    s.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                }
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_has_rows_lines_of_cols_glyphs() {
        let mut g = Grid::new(4, 2);
        g.put(0, 0, 'a');
        g.put(3, 1, '🬀'); // an astral sextant is one cell
        g.put(9, 9, 'x'); // off the grid, ignored
        let t = g.text();
        assert_eq!(t, "a   \n   🬀");
        assert!(t.lines().all(|l| l.chars().count() == 4));
    }

    #[test]
    fn wide_glyph_takes_two_cells_and_one_char_of_text() {
        let mut g = Grid::new(4, 1);
        g.set_wide(1, 0, '漢', 3);
        assert_eq!(g.get(1, 0), '漢' as u32);
        assert_eq!(g.get(2, 0), WIDE_TAIL);
        assert_eq!(g.color_at(2, 0), 3);
        assert_eq!(g.text(), " 漢 ");
        // No room for the tail: nothing drawn.
        g.set_wide(3, 0, '漢', 0);
        assert_eq!(g.get(3, 0), BLANK);
    }
}
