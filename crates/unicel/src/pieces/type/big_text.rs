//! big text: a banner in a hand-made five-row block font with a drop shadow
//! drawn in double lines, and a glint that sweeps across now and then. After
//! ascii.rest's big-text.ts (MIT, @bas3line).
//!
//! The font, the layout and the shadow are ascii.rest's. The letters are
//! drawn in octants, four dots to each pixel of the font, with every outer
//! corner taken off by a dot, so the block letters are softened rather than
//! square; they shade from rose to amber across the word, and the glint
//! passes over them in white.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// Big text's meta.
pub static META: Meta = Meta {
    name: "big text",
    slug: "big-text",
    category: Category::Type,
    note: "a soft-cornered octant banner, rose to amber, a white glint passing",
    cols: 66,
    rows: 8,
    fps: 24,
    charset: Charset::Extended,
    options: Some(r#"{"text": "hello"}"#),
    // Across the word, rose to amber; the glint; the shadow.
    palette: Some(&["#f05a8a", "#f26e78", "#f48468", "#f69a58", "#f8b44a", "#fff6e8", "#6a4a8a"]),
    ground: Some("#0d0b12"),
    ..Meta::DEFAULT
};

const ACROSS: Ramp = Ramp::new(0, 5);
const GLINT: u8 = 5;
const SHADOW: u8 = 6;

/// Five rows each, # is ink. Widths vary, so I is one pixel and M is five.
fn glyph(ch: char) -> Option<&'static str> {
    Some(match ch {
        'A' => ".##.|#..#|####|#..#|#..#",
        'B' => "###.|#..#|###.|#..#|###.",
        'C' => ".###|#...|#...|#...|.###",
        'D' => "###.|#..#|#..#|#..#|###.",
        'E' => "####|#...|###.|#...|####",
        'F' => "####|#...|###.|#...|#...",
        'G' => ".###|#...|#.##|#..#|.###",
        'H' => "#..#|#..#|####|#..#|#..#",
        'I' => "#|#|#|#|#",
        'J' => "...#|...#|...#|#..#|.##.",
        'K' => "#..#|#.#.|##..|#.#.|#..#",
        'L' => "#...|#...|#...|#...|####",
        'M' => "#...#|##.##|#.#.#|#...#|#...#",
        'N' => "#...#|##..#|#.#.#|#..##|#...#",
        'O' => ".##.|#..#|#..#|#..#|.##.",
        'P' => "###.|#..#|###.|#...|#...",
        'Q' => ".##.|#..#|#..#|#.#.|.#.#",
        'R' => "###.|#..#|###.|#.#.|#..#",
        'S' => ".###|#...|.##.|...#|###.",
        'T' => "#####|..#..|..#..|..#..|..#..",
        'U' => "#..#|#..#|#..#|#..#|.##.",
        'V' => "#...#|#...#|#...#|.#.#.|..#..",
        'W' => "#...#|#...#|#.#.#|##.##|#...#",
        'X' => "#...#|.#.#.|..#..|.#.#.|#...#",
        'Y' => "#...#|.#.#.|..#..|..#..|..#..",
        'Z' => "####|...#|..#.|.#..|####",
        '0' => "###|#.#|#.#|#.#|###",
        '1' => ".#.|##.|.#.|.#.|###",
        '2' => "###|..#|###|#..|###",
        '3' => "###|..#|.##|..#|###",
        '4' => "#.#|#.#|###|..#|..#",
        '5' => "###|#..|###|..#|###",
        '6' => "###|#..|###|#.#|###",
        '7' => "###|..#|..#|..#|..#",
        '8' => "###|#.#|###|#.#|###",
        '9' => "###|#.#|###|..#|###",
        '.' => ".|.|.|.|#",
        ',' => ".|.|.|#|#",
        '!' => "#|#|#|.|#",
        '?' => "###|..#|.##|...|.#.",
        '\'' => "#|#|.|.|.",
        ':' => ".|#|.|#|.",
        '-' => "...|...|###|...|...",
        '+' => "...|.#.|###|.#.|...",
        '=' => "...|###|...|###|...",
        '/' => "..#|..#|.#.|#..|#..",
        '_' => "...|...|...|...|###",
        ' ' => "..",
        _ => return None,
    })
}

/// Shadow cells by which neighbours they join: up 1, down 2, left 4, right 8.
const JOIN: [char; 16] = [' ', '║', '║', '║', '═', '╝', '╗', '╣', '═', '╚', '╔', '╠', '═', '╩', '╦', '╬'];

/// The banner: the shadow drawn once, the glint's run, the dots, and every
/// ink dot with its shade.
pub struct BigText {
    base: Grid,
    span: (f64, f64),
    dots: ColorOctants,
    pixels: Vec<(i64, i64, u8)>,
}

/// Big text; reads `text`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let glyphs: Vec<Vec<Vec<bool>>> = options
        .str("text", "hello")
        .to_uppercase()
        .chars()
        .filter_map(glyph)
        .map(|g| g.split('|').map(|row| row.chars().map(|c| c == '#').collect()).collect())
        .collect();
    // Each pixel is two columns by one row, so it is square; one column narrower if it would not fit.
    let span = |s: i64| glyphs.iter().map(|g: &Vec<Vec<bool>>| g[0].len() as i64 * s + 2).sum::<i64>() - 1;
    let s = if span(2) <= cols - 2 { 2 } else { 1 };
    let x0 = ((cols - span(s)) / 2).max(0);
    let y0 = (rows - 6) / 2;
    let mut ink = vec![false; (cols * rows) as usize];
    let mut x = x0;
    for g in &glyphs {
        for (y, line) in g.iter().enumerate() {
            for (i, &on) in line.iter().enumerate() {
                for k in 0..s {
                    let c = x + i as i64 * s + k;
                    if on && c < cols {
                        ink[((y0 + y as i64) * cols + c) as usize] = true;
                    }
                }
            }
        }
        x += g[0].len() as i64 * s + 2;
    }
    let at = |c: i64, r: i64| c >= 0 && c < cols && r >= 0 && r < rows && ink[(r * cols + c) as usize];
    // The shadow is the letters' outline moved half a cell right and down, so
    // each empty cell draws the outline edges that meet at its top left corner.
    let mut base = Grid::new(META.cols, META.rows);
    for r in 0..rows {
        for c in 0..cols {
            if at(c, r) {
                continue;
            }
            let (a, b, d) = (at(c - 1, r - 1), at(c, r - 1), at(c - 1, r));
            let edges = usize::from(a != b) | (usize::from(d) << 1) | (usize::from(a != d) << 2) | (usize::from(b) << 3);
            // Where two letters touch only at that corner, the shadow turns down and right.
            let ch = JOIN[if edges == 15 { 2 | 8 } else { edges }];
            if ch != ' ' {
                base.set(c, r, ch, SHADOW);
            }
        }
    }
    // Every ink dot, four to a cell's height and two to its width, with each
    // outer corner of the ink taken off.
    let mut pixels = vec![];
    for r in 0..rows {
        for c in 0..cols {
            if !at(c, r) {
                continue;
            }
            for dy in 0..4 {
                for dx in 0..2 {
                    let (up, down) = (dy == 0 && !at(c, r - 1), dy == 3 && !at(c, r + 1));
                    let (left, right) = (dx == 0 && !at(c - 1, r), dx == 1 && !at(c + 1, r));
                    if (up || down) && (left || right) {
                        continue;
                    }
                    let shade = ACROSS.at((c - x0) as f64 / span(s).max(1) as f64);
                    pixels.push((c * 2 + dx, r * 4 + dy, shade));
                }
            }
        }
    }
    Box::new(BigText { base, span: ((x0 - 2) as f64, (x0 + span(s) + 9) as f64), dots: ColorOctants::new(META.cols, META.rows), pixels })
}

impl Piece for BigText {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        // The glint crosses at an even pace, then rests out of sight. Frame 0 is at rest.
        let (sweep, period) = (2.4, 3.2);
        let u = (t - 0.5).rem_euclid(period) / sweep;
        let p = if u < 1.0 { self.span.0 + (self.span.1 - self.span.0) * u } else { -99.0 };
        self.dots.clear();
        for &(x, y, shade) in &self.pixels {
            let d = (x as f64 / 2.0 + 1.2 * y as f64 / 4.0 - p).abs();
            if d < 2.4 {
                self.dots.set(x, y, GLINT);
            } else if env.paper || dither(x as usize, y as usize, 0.85) {
                // A dense fill, so the solid glint is brighter even in one ink.
                self.dots.set(x, y, shade);
            }
        }
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        self.dots.draw(grid, 0, 0);
    }
}
