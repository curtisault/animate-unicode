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

/// A frame: `cols × rows` cells, each a code point and a palette index.
///
/// Every drawing method clips, so shapes may overhang the edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grid {
    cols: usize,
    rows: usize,
    cells: Vec<u32>,
    colors: Vec<u8>,
}

impl Grid {
    /// A grid of spaces in colour 0. Panics on a zero size.
    pub fn new(cols: usize, rows: usize) -> Self {
        assert!(cols > 0 && rows > 0, "a grid needs at least one cell");
        Self { cols, rows, cells: vec![BLANK; cols * rows], colors: vec![0; cols * rows] }
    }

    /// Width in cells.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Height in cells.
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

    /// The row-major index of an on-grid cell, for `cells_mut` and `colors_mut`.
    #[inline]
    pub fn index(&self, x: usize, y: usize) -> usize {
        debug_assert!(x < self.cols && y < self.rows, "({x}, {y}) is off a {}x{} grid", self.cols, self.rows);
        y * self.cols + x
    }

    /// True if `(x, y)` is a cell of this grid.
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
    /// Drawn only when both cells are on the grid. Debug builds check that the
    /// glyph really is two columns wide (East Asian Width W or F).
    pub fn set_wide(&mut self, x: i64, y: i64, ch: char, color: u8) {
        debug_assert_eq!(unicode_width::UnicodeWidthChar::width(ch), Some(2), "{ch:?} is not a double-width glyph");
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

    /// A rectangle filled with one glyph, clipped to the grid.
    pub fn fill_rect(&mut self, x: i64, y: i64, w: i64, h: i64, ch: char, color: u8) {
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = ((x + w).min(self.cols as i64), (y + h).min(self.rows as i64));
        for yy in y0..y1 {
            for xx in x0..x1 {
                let i = self.index(xx as usize, yy as usize);
                self.cells[i] = ch as u32;
                self.colors[i] = color;
            }
        }
    }

    /// A straight line of one glyph from `(x0, y0)` to `(x1, y1)`, both ends
    /// included (Bresenham). The off-grid part is clipped.
    pub fn line(&mut self, x0: i64, y0: i64, x1: i64, y1: i64, ch: char, color: u8) {
        bresenham(x0, y0, x1, y1, |x, y| self.set(x, y, ch, color));
    }

    /// Copies `src` onto this grid with its top-left cell at `(x, y)`: a sprite.
    ///
    /// Spaces in `src` are transparent, so a sprite's background does not
    /// erase what is behind it. A wide glyph is copied only when both of its
    /// cells land on this grid, and a tail is never copied without its glyph.
    pub fn blit(&mut self, src: &Grid, x: i64, y: i64) {
        for sy in 0..src.rows {
            for sx in 0..src.cols {
                let i = src.index(sx, sy);
                let cp = src.cells[i];
                if cp == BLANK || cp == WIDE_TAIL {
                    continue;
                }
                let (dx, dy) = (x + sx as i64, y + sy as i64);
                if !self.in_bounds(dx, dy) {
                    continue;
                }
                let wide = sx + 1 < src.cols && src.cells[i + 1] == WIDE_TAIL;
                if wide && !self.in_bounds(dx + 1, dy) {
                    continue;
                }
                let d = self.index(dx as usize, dy as usize);
                self.cells[d] = cp;
                self.colors[d] = src.colors[i];
                if wide {
                    self.cells[d + 1] = WIDE_TAIL;
                    self.colors[d + 1] = src.colors[i];
                }
            }
        }
    }

    /// The code point at an on-grid cell.
    pub fn get(&self, x: usize, y: usize) -> u32 {
        self.cells[self.index(x, y)]
    }

    /// The palette index at an on-grid cell.
    pub fn color_at(&self, x: usize, y: usize) -> u8 {
        self.colors[self.index(x, y)]
    }

    /// Row-major code points, `WIDE_TAIL` included: what the wasm shell reads.
    pub fn cells(&self) -> &[u32] {
        &self.cells
    }

    /// One row's code points, `WIDE_TAIL` included. Panics off the grid.
    pub fn row(&self, y: usize) -> &[u32] {
        assert!(y < self.rows, "row {y} is off a {}-row grid", self.rows);
        &self.cells[y * self.cols..(y + 1) * self.cols]
    }

    /// Row-major palette indices, parallel to `cells`.
    pub fn colors(&self) -> &[u8] {
        &self.colors
    }

    /// The code points for a hot loop. Index with [`Grid::index`]; a piece
    /// that writes here must keep its indices on the grid itself.
    pub fn cells_mut(&mut self) -> &mut [u32] {
        &mut self.cells
    }

    /// The palette indices for a hot loop, parallel to `cells_mut`.
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

/// Calls `plot` for every point on the line from `(x0, y0)` to `(x1, y1)`,
/// both ends included, in order from the first end. Integer Bresenham, so
/// the same line on every platform.
pub(crate) fn bresenham(x0: i64, y0: i64, x1: i64, y1: i64, mut plot: impl FnMut(i64, i64)) {
    let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
    let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
    let (mut x, mut y, mut err) = (x0, y0, dx + dy);
    loop {
        plot(x, y);
        if x == x1 && y == y1 {
            return;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
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

    #[test]
    fn row_is_one_row_of_cells() {
        let mut g = Grid::new(3, 2);
        g.text_at(0, 1, "abc", 0);
        assert_eq!(g.row(1), &['a' as u32, 'b' as u32, 'c' as u32]);
        assert_eq!(g.row(0), &[BLANK; 3]);
    }

    #[test]
    fn fill_rect_clips() {
        let mut g = Grid::new(4, 3);
        g.fill_rect(-1, 1, 3, 9, '#', 2);
        assert_eq!(g.text(), "    \n##  \n##  ");
        assert_eq!(g.color_at(1, 2), 2);
        g.fill_rect(2, 0, 0, 3, 'x', 0); // zero width draws nothing
        assert_eq!(g.get(2, 0), BLANK);
    }

    #[test]
    fn line_includes_both_ends_in_every_direction() {
        let mut g = Grid::new(5, 3);
        g.line(0, 0, 4, 2, '*', 1);
        assert_eq!(g.text(), "*    \n **  \n   **");
        // Backwards, ties break the other way, but the ends and the count hold.
        let mut back = Grid::new(5, 3);
        back.line(4, 2, 0, 0, '*', 1);
        let inked = |g: &Grid| g.cells().iter().filter(|&&c| c == '*' as u32).count();
        assert_eq!((back.get(0, 0), back.get(4, 2), inked(&back)), ('*' as u32, '*' as u32, inked(&g)));
        let mut dot = Grid::new(2, 2);
        dot.line(1, 1, 1, 1, 'o', 0);
        assert_eq!(dot.text(), "  \n o");
    }

    #[test]
    fn line_clips_past_the_edge() {
        let mut g = Grid::new(3, 1);
        g.line(-5, 0, 10, 0, '-', 0);
        assert_eq!(g.text(), "---");
    }

    #[test]
    fn blit_treats_spaces_as_transparent_and_clips() {
        let mut sprite = Grid::new(3, 2);
        sprite.text_at(0, 0, "a b", 4);
        sprite.text_at(0, 1, "cde", 5);
        let mut g = Grid::new(4, 2);
        g.fill(':', 0);
        g.blit(&sprite, 2, 0);
        assert_eq!(g.text(), "::a:\n::cd");
        assert_eq!(g.color_at(2, 0), 4);
        assert_eq!(g.color_at(3, 0), 0, "the space in the sprite kept what was behind it");
        g.blit(&sprite, -2, -1);
        assert_eq!(g.text(), "e:a:\n::cd");
    }

    #[test]
    fn blit_never_splits_a_wide_glyph() {
        let mut sprite = Grid::new(2, 1);
        sprite.set_wide(0, 0, '漢', 1);
        let mut g = Grid::new(3, 1);
        g.blit(&sprite, 2, 0); // the tail would land off the grid
        assert_eq!(g.text(), "   ");
        g.blit(&sprite, -1, 0); // only the tail would land on the grid
        assert_eq!(g.cells(), &[BLANK; 3]);
        g.blit(&sprite, 1, 0);
        assert_eq!(g.cells(), &[BLANK, '漢' as u32, WIDE_TAIL]);
    }
}
