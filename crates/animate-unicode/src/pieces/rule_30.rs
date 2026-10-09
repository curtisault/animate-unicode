//! rule 30: an elementary cellular automaton grown from one live cell, each
//! generation a row, scrolling up. It runs rule 30, then 90, then 110, each
//! from a fresh seed. After ascii.rest's rule-30.ts (MIT, @bas3line).
//!
//! One cell is one octant dot, so the 64×20 characters hold 128 cells across
//! and 80 generations down, four to a character row: rule 30's chaos runs
//! twice as wide, rule 90 grows six levels of triangles instead of five, and
//! each rule has its own colour.

use crate::{Category, Charset, Env, Grid, Meta, Octants, Options, Piece};

/// Rule 30's meta.
pub static META: Meta = Meta {
    name: "rule 30",
    slug: "rule-30",
    category: Category::Generative,
    note: "rules 30, 90 and 110 in octants, one cell a dot, each its colour",
    cols: 64,
    rows: 20,
    fps: 8,
    charset: Charset::Extended,
    // Rule 30, rule 90, rule 110.
    palette: Some(&["#ff9a4a", "#6ad0ff", "#b98aff"]),
    ground: Some("#0a0a10"),
    ..Meta::DEFAULT
};

/// Each rule, the cell its seed is in, and how many generations it runs:
/// short enough that its triangle never reaches the sides. Rule 90 stops on
/// the row that closes its sixth level of triangles; rule 110 grows only to
/// the left, so its seed sits well right of centre. Every count is a multiple
/// of four, so the tape scrolls a whole character row at a time.
const RULES: [(u8, usize, usize); 3] = [(30, 63, 64), (90, 63, 64), (110, 104, 96)];
/// Blank generations between one rule and the next.
const GAP: usize = 16;
/// Cells past each side, so the edges never show.
const EXTRA: usize = 96;

/// Every generation of the loop, cropped to the frame, and which rule made it.
pub struct Rule30 {
    tape: Vec<(Vec<bool>, u8)>,
}

/// The whole loop of generations.
fn tape() -> Vec<(Vec<bool>, u8)> {
    let cols = META.cols * 2;
    let w = cols + 2 * EXTRA;
    let mut tape = vec![];
    for (r, &(rule, at, gens)) in RULES.iter().enumerate() {
        let mut cur = vec![false; w];
        cur[EXTRA + at] = true;
        for _ in 0..gens {
            tape.push((cur[EXTRA..EXTRA + cols].to_vec(), r as u8));
            let mut next = vec![false; w];
            for i in 1..w - 1 {
                let k = (u8::from(cur[i - 1]) << 2) | (u8::from(cur[i]) << 1) | u8::from(cur[i + 1]);
                next[i] = (rule >> k) & 1 == 1;
            }
            cur = next;
        }
        for _ in 0..GAP {
            tape.push((vec![false; cols], r as u8));
        }
    }
    tape
}

/// Rule 30; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Rule30 { tape: tape() })
}

impl Piece for Rule30 {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let rows = self.tape.len() / 4; // character rows in the loop
                                        // The newest generation is always the bottom row. At t = 0 the first
                                        // triangle sits two rows down from the top.
        let top = (t * META.fps as f64).floor() as i64 - 2;
        let mut dots = Octants::new(META.cols, META.rows);
        grid.clear();
        for r in 0..META.rows {
            let k = (top + r as i64).rem_euclid(rows as i64) as usize;
            for g in 0..4 {
                for (x, &on) in self.tape[4 * k + g].0.iter().enumerate() {
                    if on {
                        dots.set(x as i64, (4 * r + g) as i64);
                    }
                }
            }
            let color = self.tape[4 * k].1;
            for c in 0..META.cols {
                if dots.mask(c, r) != 0 {
                    grid.set(c as i64, r as i64, dots.glyph(c, r), color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_grow_as_they_should() {
        let tape = tape();
        assert_eq!(tape.len() % 4, 0, "whole character rows");
        // Rule 30's third generation from one cell: 1 1 0 0 1 around the seed.
        let row = |g: usize, from: usize| -> String { tape[g].0[from..from + 5].iter().map(|&b| if b { '1' } else { '0' }).collect() };
        assert_eq!(row(2, 61), "11001");
        // Rule 90 is Pascal's triangle mod 2, and row 63 of it is all odd: its
        // last generation has every other cell on, from edge to edge.
        let last = &tape[64 + GAP + 63].0;
        assert!((0..=126).all(|x| last[x] == (x % 2 == 0)), "the sixth level closes");
    }
}
