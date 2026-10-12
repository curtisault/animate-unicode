//! not found: a 404 page. The code in heavy block digits with a double-line
//! shadow, a small ghost drifting to and fro beneath it, and a line of copy.
//! After ascii.rest's not-found.ts (MIT, @bas3line).
//!
//! The digits and their shadow are ascii.rest's, shaded rose to amber across
//! the code. The ghost is drawn in octants, eight dots a character, so it is
//! a smooth shape, a round head and a hem that ripples as it floats, where
//! half blocks gave it steps; its eyes look the way it is drifting.

use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The 404's meta.
pub static META: Meta = Meta {
    name: "not found",
    slug: "not-found",
    category: Category::Ui,
    note: "a 404 in rose to amber, a smooth octant ghost drifting beneath",
    cols: 60,
    rows: 21,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"code": "404", "title": "page not found", "message": "the page you asked for has moved or never existed"}"#),
    // The digits, rose to amber; the shadow; the ghost; its pupils; the title; the message.
    palette: Some(&["#f05a8a", "#f2707a", "#f4886a", "#f6a05a", "#f8b84a", "#5a4a7a", "#e8e4f8", "#3a5ad8", "#e5e9f0", "#8a90a0"]),
    ground: Some("#0d0c14"),
    ..Meta::DEFAULT
};

const SHADOW: u8 = 5;
const GHOST: u8 = 6;
const PUPIL: u8 = 7;
const TITLE: u8 = 8;
const COPY: u8 = 9;
/// A 3×5 font, each font column drawn 4 cells wide and the rows 2,1,2,1,2
/// cells tall, so a digit is 12×8 and strokes are as thick across as down.
const FONT: [&str; 10] = [
    "###,#.#,#.#,#.#,###",
    "##.,.#.,.#.,.#.,###",
    "###,..#,###,#..,###",
    "###,..#,###,..#,###",
    "#.#,#.#,###,..#,..#",
    "###,#..,###,..#,###",
    "###,#..,###,#.#,###",
    "###,..#,..#,..#,..#",
    "###,#.#,###,#.#,###",
    "###,#.#,###,..#,###",
];
const RH: [i64; 5] = [2, 1, 2, 1, 2];
const DW: i64 = 12;
const DH: i64 = 8;
const GAP: i64 = 4;
const TOP: i64 = 1;
/// Seconds for one trip there and back.
const PERIOD: f64 = 9.0;
/// The ghost, in dots: 16 across, 20 down.
const GW: f64 = 16.0;
const GH: f64 = 20.0;

/// The page: the digits and copy drawn once, and the ghost's dots.
pub struct NotFound {
    base: Grid,
    dots: ColorOctants,
}

/// A 404 page; reads `code`, `title` and `message`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let mut digits: Vec<usize> = options.str("code", "404").chars().filter_map(|c| c.to_digit(10)).take(3).map(|d| d as usize).collect();
    if digits.is_empty() {
        digits = vec![4, 0, 4];
    }
    let width = digits.len() as i64 * DW + (digits.len() as i64 - 1) * GAP + 1;
    let x0 = (cols - width) / 2;
    let mut face = vec![false; (cols * rows) as usize];
    for (i, &d) in digits.iter().enumerate() {
        let mut r = TOP;
        for (fr, bits) in FONT[d].split(',').enumerate() {
            for _ in 0..RH[fr] {
                for (fc, b) in bits.chars().enumerate() {
                    if b == '#' {
                        for c in 0..4 {
                            face[(r * cols + x0 + i as i64 * (DW + GAP) + fc as i64 * 4 + c) as usize] = true;
                        }
                    }
                }
                r += 1;
            }
        }
    }
    // The shadow is the face moved one cell down and right, traced in double lines.
    let at = |r: i64, c: i64| r >= 0 && c >= 0 && r < rows && c < cols && face[(r * cols + c) as usize];
    let mut base = Grid::new(META.cols, META.rows);
    for r in 0..rows {
        for c in 0..cols {
            if at(r, c) {
                // Rose to amber across the code.
                let shade = ((c - x0) as f64 / width as f64 * 4.0).round() as u8;
                base.set(c, r, '█', shade.min(4));
                continue;
            }
            let (l, u, ul) = (at(r, c - 1), at(r - 1, c), at(r - 1, c - 1));
            let ch = match (l, u, ul) {
                (true, true, _) => '╔',
                (true, false, true) => '║',
                (true, false, false) => '╗',
                (false, true, true) => '═',
                (false, true, false) => '╚',
                (false, false, true) => '╝',
                _ => continue,
            };
            base.set(c, r, ch, SHADOW);
        }
    }
    let centre = |s: &str| (cols - s.chars().count() as i64) / 2;
    let title = options.string("title", "page not found");
    let message = options.string("message", "the page you asked for has moved or never existed");
    base.text_at(centre(&title), TOP + DH + 9, &title, TITLE);
    base.text_at(centre(&message), TOP + DH + 10, &message, COPY);
    Box::new(NotFound { base, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for NotFound {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let ph = t / PERIOD * TAU;
        // Its left edge in dots, two to a column, and its top, bobbing a little
        // and always a clear row below the digits' shadow.
        let gx = (META.cols as f64 / 2.0 - 4.0 + 18.0 * ph.sin()) * 2.0;
        let gy = ((TOP + DH + 2) as f64 + 0.5) * 4.0 + 2.0 * (t / 1.5 * TAU).sin();
        let right = ph.cos() >= 0.0; // heading right
        self.dots.clear();
        let r = GW / 2.0;
        for y in 0..GH as i64 {
            for x in 0..GW as i64 {
                let (fx, fy) = (x as f64 + 0.5, y as f64 + 0.5);
                // A round head over a straight body, its hem a running wave.
                let head = fy < r && (fx - r).powi(2) + (fy - r).powi(2) < r * r;
                let hem = GH - 3.0 + 1.6 * ((fx / GW * 3.0 + t * 2.0) * TAU / 2.0 + PI / 2.0).sin();
                let body = fy >= r && fy < hem;
                if !(head || body) {
                    continue;
                }
                // Eye holes, two dots across and four down, each with a pupil
                // in the lower corner it is heading toward.
                let eye = [4i64, 10].iter().find(|&&ex| (ex..ex + 3).contains(&x) && (6..10).contains(&y)).copied();
                let (px, py) = ((gx + fx).floor() as i64, (gy + fy).floor() as i64);
                match eye {
                    Some(ex) => {
                        let pupil = y >= 8 && if right { x > ex } else { x <= ex + 1 };
                        if pupil {
                            self.dots.set(px, py, PUPIL);
                        }
                    }
                    None => self.dots.set(px, py, GHOST),
                }
            }
        }
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        self.dots.draw(grid, 0, 0);
    }
}
