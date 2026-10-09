//! From a piece's grid to a terminal: what each terminal cell shows, and the
//! escape codes that bring the screen up to date. Pure functions over strings,
//! so the tests can check exactly what would be written.
//!
//! After ascii.rest's terminal.ts by @bas3line (MIT), with code-point cells:
//! a wide glyph is one cell and a `WIDE_TAIL`, and the terminal is trusted to
//! draw it two columns wide.

use animate_unicode::{Grid, Meta, WIDE_TAIL};
use std::fmt::Write;

/// A colour as 0xrrggbb.
pub type Rgb = u32;

/// How much of a square cell each of a scene's dots inks, for mixing its
/// colour into the ground when two rows share a terminal cell.
fn ink(ch: char) -> f64 {
    match ch {
        ' ' => 0.0,
        '·' => 0.15,
        '•' => 0.45,
        '●' => 0.9,
        '░' => 0.25,
        '▒' => 0.5,
        '▓' => 0.75,
        '█' => 1.0,
        _ => 0.5,
    }
}

/// `#rrggbb` as a number; anything else is black.
pub fn rgb(hex: &str) -> Rgb {
    u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0)
}

/// True if a colour is dark (luma under half).
pub fn dark(c: Rgb) -> bool {
    let (r, g, b) = ((c >> 16) & 255, (c >> 8) & 255, c & 255);
    0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b) < 128.0
}

fn mix(a: Rgb, b: Rgb, k: f64) -> Rgb {
    [16, 8, 0].iter().fold(0, |out, &s| {
        let (x, y) = (f64::from((a >> s) & 255), f64::from((b >> s) & 255));
        out | (((x + (y - x) * k).round() as u32) << s)
    })
}

/// One terminal cell: a glyph (`'\0'` for the right half of a wide one), and
/// its ink and ground, `None` for the terminal's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
}

const TAIL: char = '\0';

/// A piece's frames as terminal cells. A terminal cell is twice as tall as it
/// is wide, like a piece's usual cell; a square-celled piece (`cell: 1`)
/// shows two of its rows in each terminal row.
pub struct Painter {
    pub cols: usize,
    pub rows: usize,
    square: bool,
    palette: Option<Vec<Rgb>>,
    ground: Option<Rgb>,
    pub cells: Vec<Cell>,
}

impl Painter {
    /// In colour when `colour` and the piece has a palette; else in the
    /// terminal's own ink, like any text.
    pub fn new(meta: &Meta, colour: bool) -> Self {
        let square = meta.cell == 1;
        let rows = if square { meta.rows.div_ceil(2) } else { meta.rows };
        let palette = meta.palette.filter(|_| colour).map(|p| p.iter().map(|c| rgb(c)).collect::<Vec<_>>());
        let ground = if palette.is_some() { meta.ground.map(rgb) } else { None };
        Painter { cols: meta.cols, rows, square, palette, ground, cells: vec![Cell { ch: ' ', fg: None, bg: None }; meta.cols * rows] }
    }

    /// The ground a coloured piece is drawn on, if it has one.
    pub fn ground(&self) -> Option<Rgb> {
        self.ground
    }

    fn ink_of(&self, grid: &Grid, x: usize, y: usize) -> Option<Rgb> {
        let p = self.palette.as_ref()?;
        Some(p.get(usize::from(grid.color_at(x, y))).copied().unwrap_or(p[0]))
    }

    /// Fills `cells` from a frame.
    pub fn paint(&mut self, grid: &Grid) {
        let glyph = |x: usize, y: usize| -> char {
            if y >= grid.rows() {
                return ' ';
            }
            let cp = grid.get(x, y);
            if cp == WIDE_TAIL {
                TAIL
            } else {
                char::from_u32(cp).unwrap_or('\u{FFFD}')
            }
        };
        for y in 0..self.rows {
            for x in 0..self.cols {
                let k = y * self.cols + x;
                self.cells[k] = if !self.square {
                    let ch = glyph(x, y);
                    let fg = if ch == ' ' || ch == TAIL { None } else { self.ink_of(grid, x, y) };
                    Cell { ch, fg, bg: self.ground }
                } else if self.palette.is_some() {
                    // Two square rows in one terminal row: an upper half block,
                    // its ink the top row's colour and its ground the bottom's,
                    // each mixed into the piece's ground by how much it inks.
                    let shade = |yy: usize| -> Option<Rgb> {
                        let ch = glyph(x, yy);
                        if ch == ' ' || ch == TAIL {
                            return self.ground;
                        }
                        let ink = self.ink_of(grid, x, yy)?;
                        Some(self.ground.map_or(ink, |g| mix(g, ink, self::ink(ch))))
                    };
                    let (top, bottom) = (shade(2 * y), shade(2 * y + 1));
                    match (top, bottom) {
                        (t, b) if t == b => Cell { ch: ' ', fg: None, bg: t },
                        (None, b) => Cell { ch: '▄', fg: b, bg: None },
                        (t, b) => Cell { ch: '▀', fg: t, bg: b },
                    }
                } else {
                    // As text, the top row's glyph unless it is blank.
                    let (a, b) = (glyph(x, 2 * y), glyph(x, 2 * y + 1));
                    let pick = if a == ' ' || a == TAIL { b } else { a };
                    Cell { ch: if pick == TAIL { ' ' } else { pick }, fg: None, bg: None }
                };
            }
        }
    }

    /// The frame as plain text, rows joined by newlines, wide tails left out:
    /// what a pipe gets.
    pub fn text(&self) -> String {
        self.cells.chunks(self.cols).map(|row| row.iter().filter(|c| c.ch != TAIL).map(|c| c.ch).collect::<String>()).collect::<Vec<_>>().join("\n")
    }
}

/// Unchanged cells a run of changed ones is carried over, rather than jumping
/// the cursor past them: about what a jump costs.
const GAP: usize = 4;

/// What the terminal shows, so each frame writes only the cells that changed.
pub struct Screen {
    sent: Vec<Option<Cell>>,
    /// True when the terminal was smaller than the piece at the last draw.
    pub cropped: bool,
}

fn sgr(out: &mut String, layer: u8, c: Rgb) {
    let _ = write!(out, "\x1b[{layer};2;{};{};{}m", (c >> 16) & 255, (c >> 8) & 255, c & 255);
}

impl Screen {
    pub fn new(cells: usize) -> Self {
        Screen { sent: vec![None; cells], cropped: false }
    }

    /// Forget what was sent: the next draw writes every cell (after a resize,
    /// when the piece has moved on the screen).
    pub fn forget(&mut self) {
        self.sent.fill(None);
    }

    /// The escape codes that bring a `term_cols × term_rows` terminal from what
    /// was sent to `p`'s cells, centred, cropped to the middle if too small.
    pub fn draw(&mut self, p: &Painter, term_cols: usize, term_rows: usize) -> String {
        let (w, h) = (p.cols.min(term_cols), p.rows.min(term_rows));
        self.cropped = w < p.cols || h < p.rows;
        // Where the part that fits starts in the piece, and where it goes on the screen (1-based).
        let (sx, sy) = ((p.cols - w) / 2, (p.rows - h) / 2);
        let (left, top) = ((term_cols - w) / 2 + 1, (term_rows - h) / 2 + 1);
        let cells = &p.cells;
        let changed = |sent: &[Option<Cell>], k: usize| match sent[k] {
            None => true,
            Some(s) => s.ch != cells[k].ch || s.bg != cells[k].bg || (cells[k].ch != ' ' && s.fg != cells[k].fg),
        };
        let mut out = String::new();
        for y in 0..h {
            let row = (sy + y) * p.cols;
            let mut x = sx;
            while x < sx + w {
                if !changed(&self.sent, row + x) {
                    x += 1;
                    continue;
                }
                // A run of changed cells, carried over short stretches of unchanged ones.
                let (mut end, mut gap) = (x + 1, 0);
                for j in x + 1..sx + w {
                    if changed(&self.sent, row + j) {
                        end = j + 1;
                        gap = 0;
                    } else {
                        gap += 1;
                        if gap > GAP {
                            break;
                        }
                    }
                }
                // Each run starts in the terminal's own colours and goes back
                // to them, so no run depends on another.
                let _ = write!(out, "\x1b[{};{}H", top + y, left + x - sx);
                let (mut ink, mut under): (Option<Rgb>, Option<Rgb>) = (None, None);
                let mut coloured = false;
                for k in row + x..row + end {
                    let c = cells[k];
                    let col = k - row;
                    // A wide glyph's tail: the terminal moved past it when it drew
                    // the glyph. Unless the glyph was cropped away or this run
                    // starts here; then a space keeps the columns right.
                    let ch = if c.ch == TAIL {
                        if k > row + x && cells[k - 1].ch != TAIL && col > sx {
                            self.sent[k] = Some(c);
                            continue;
                        }
                        ' '
                    } else if col + 1 < p.cols && cells[k + 1].ch == TAIL && col + 1 >= sx + w {
                        ' ' // its tail would fall outside what fits
                    } else {
                        c.ch
                    };
                    if c.bg != under {
                        match c.bg {
                            Some(b) => sgr(&mut out, 48, b),
                            None => out.push_str("\x1b[49m"),
                        }
                        under = c.bg;
                        coloured |= c.bg.is_some();
                    }
                    if ch != ' ' && c.fg != ink {
                        match c.fg {
                            Some(f) => sgr(&mut out, 38, f),
                            None => out.push_str("\x1b[39m"),
                        }
                        ink = c.fg;
                        coloured |= c.fg.is_some();
                    }
                    out.push(ch);
                    self.sent[k] = Some(c);
                }
                if coloured {
                    out.push_str("\x1b[0m");
                }
                x = end;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use animate_unicode::Meta;

    fn meta(cols: usize, rows: usize, palette: Option<&'static [&'static str]>, ground: Option<&'static str>) -> Meta {
        Meta { name: "t", slug: "t", note: "t", cols, rows, palette, ground, ..Meta::DEFAULT }
    }

    #[test]
    fn plain_text_centres_and_writes_each_changed_run_once() {
        let m = meta(3, 1, None, None);
        let mut g = Grid::new(3, 1);
        g.text_at(0, 0, "abc", 0);
        let mut p = Painter::new(&m, true);
        p.paint(&g);
        let mut s = Screen::new(3);
        // A 7×3 terminal: the piece goes in row 2, columns 3 to 5.
        assert_eq!(s.draw(&p, 7, 3), "\x1b[2;3Habc");
        assert_eq!(s.draw(&p, 7, 3), "", "nothing changed, nothing sent");
        g.put(2, 0, 'Z');
        p.paint(&g);
        assert_eq!(s.draw(&p, 7, 3), "\x1b[2;5HZ");
    }

    #[test]
    fn colour_comes_from_the_palette_over_the_ground() {
        let m = meta(2, 1, Some(&["#000000", "#ff8000"]), Some("#102030"));
        let mut g = Grid::new(2, 1);
        g.set(0, 0, '#', 1);
        let mut p = Painter::new(&m, true);
        p.paint(&g);
        let out = Screen::new(2).draw(&p, 2, 1);
        assert_eq!(out, "\x1b[1;1H\x1b[48;2;16;32;48m\x1b[38;2;255;128;0m# \x1b[0m");
        // mono: no colour at all
        let mut p = Painter::new(&m, false);
        p.paint(&g);
        assert_eq!(Screen::new(2).draw(&p, 2, 1), "\x1b[1;1H# ");
    }

    #[test]
    fn a_wide_glyph_is_written_once_and_its_tail_skipped() {
        let m = meta(4, 1, None, None);
        let mut g = Grid::new(4, 1);
        g.set_wide(1, 0, '漢', 0);
        let mut p = Painter::new(&m, false);
        p.paint(&g);
        assert_eq!(Screen::new(4).draw(&p, 4, 1), "\x1b[1;1H 漢 ");
        assert_eq!(p.text(), " 漢 ");
    }

    #[test]
    fn cropping_never_splits_a_wide_glyph() {
        // 4 wide in a 2-column terminal: columns 1 and 2 show, so the glyph
        // at 0 loses its head and the one at 2 its tail; both become spaces.
        let m = meta(4, 1, None, None);
        let mut g = Grid::new(4, 1);
        g.set_wide(0, 0, '漢', 0);
        g.set_wide(2, 0, '字', 0);
        let mut p = Painter::new(&m, false);
        p.paint(&g);
        let mut s = Screen::new(4);
        assert_eq!(s.draw(&p, 2, 1), "\x1b[1;1H  ");
        assert!(s.cropped);
    }

    #[test]
    fn a_small_terminal_shows_the_middle() {
        let m = meta(5, 3, None, None);
        let mut g = Grid::new(5, 3);
        for (y, row) in ["abcde", "fghij", "klmno"].iter().enumerate() {
            g.text_at(0, y as i64, row, 0);
        }
        let mut p = Painter::new(&m, false);
        p.paint(&g);
        let mut s = Screen::new(15);
        assert_eq!(s.draw(&p, 3, 1), "\x1b[1;1Hghi");
        assert!(s.cropped);
        s.forget();
        assert!(!s.draw(&p, 9, 5).is_empty());
        assert!(!s.cropped);
    }

    #[test]
    fn square_cells_pair_rows_as_half_blocks() {
        let m = Meta { cell: 1, ..meta(1, 2, Some(&["#000000", "#ffffff"]), Some("#000000")) };
        let mut g = Grid::new(1, 2);
        g.set(0, 0, '█', 1);
        let mut p = Painter::new(&m, true);
        p.paint(&g);
        assert_eq!(p.rows, 1);
        assert_eq!(p.cells[0], Cell { ch: '▀', fg: Some(0xffffff), bg: Some(0) });
    }

    #[test]
    fn colours_parse_and_judge_darkness() {
        assert_eq!(rgb("#102030"), 0x102030);
        assert!(dark(rgb("#070707")) && !dark(rgb("#f4efe2")));
    }
}
