//! glider gun: Gosper's glider gun in Conway's life. It fires a glider every
//! 30 generations, and an eater below the frame swallows each one, so the
//! field repeats exactly. After ascii.rest's glider-gun.ts (MIT, @bas3line).
//!
//! One life cell is one octant dot, eight to a character, so the 56×20
//! characters hold a 112×80 field: the gun is a third of the width and its
//! glider lane runs twice as far before the eater. A cell is coloured by its
//! age, white when it is born, cooling through green to blue as it lasts, so
//! the gun's churn, the gliders and the still blocks each have their colour.

use crate::{Category, Charset, Env, Grid, Meta, Octants, Options, Piece};

/// The glider gun's meta.
pub static META: Meta = Meta {
    name: "glider gun",
    slug: "glider-gun",
    category: Category::Generative,
    note: "gosper's gun in octants, one cell a dot, coloured by its age",
    cols: 56,
    rows: 20,
    fps: 10,
    charset: Charset::Extended,
    // By age, newborn first.
    palette: Some(&["#f4fff6", "#9ff5a8", "#4fd884", "#2fae8c", "#2a7f9a", "#2b5694"]),
    ground: Some("#06090c"),
    ..Meta::DEFAULT
};

const GUN: [&str; 9] = [
    "........................o",
    "......................o.o",
    "............oo......oo............oo",
    "...........o...o....oo............oo",
    "oo........o.....o...oo",
    "oo........o...o.oo....o.o",
    "..........o.....o.......o",
    "...........o...o",
    "............oo",
];
const EATER: [&str; 4] = ["oo", "o", ".ooo", "...o"];
const PERIOD: usize = 30;
/// Dead cells round the field, never written.
const PAD: usize = 4;
/// The gun's top left in the picture, and how far down the glider lane the
/// eater sits, past the frame.
const GX: usize = 3;
const GY: usize = 3;
const K: usize = 72;
/// Ages past this all look the same.
const OLD: u8 = 5;

/// A life field: cells alive (1) or dead, and each live cell's age.
#[derive(Clone, PartialEq)]
struct Life {
    w: usize,
    h: usize,
    alive: Vec<u8>,
    age: Vec<u8>,
}

impl Life {
    fn new(w: usize, h: usize) -> Self {
        Life { w, h, alive: vec![0; w * h], age: vec![0; w * h] }
    }

    fn place(&mut self, shape: &[&str], x0: usize, y0: usize) {
        for (y, row) in shape.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                if c == 'o' {
                    self.alive[(PAD + y0 + y) * self.w + PAD + x0 + x] = 1;
                }
            }
        }
    }

    /// One generation. The outer ring is never written, so it stays dead.
    fn step(&mut self) {
        let w = self.w;
        let mut next = vec![0u8; self.alive.len()];
        for y in 1..self.h - 1 {
            for x in 1..w - 1 {
                let i = y * w + x;
                let a = &self.alive;
                let n = a[i - w - 1] + a[i - w] + a[i - w + 1] + a[i - 1] + a[i + 1] + a[i + w - 1] + a[i + w] + a[i + w + 1];
                next[i] = u8::from(n == 3 || (n == 2 && a[i] == 1));
            }
        }
        for (i, &n) in next.iter().enumerate() {
            self.age[i] = if n == 0 {
                0
            } else if self.alive[i] == 1 {
                (self.age[i] + 1).min(OLD)
            } else {
                0
            };
        }
        self.alive = next;
    }
}

/// The glider gun: one period of frames, worked out once.
pub struct GliderGun {
    cycle: Vec<Grid>,
}

/// The field after warming up: the lane full of gliders and every block grown old.
fn warm() -> Life {
    let (w, h) = (META.cols * 2 + 2 * PAD + 8, META.rows * 4 + 2 * PAD + 12);
    let mut life = Life::new(w, h);
    life.place(&GUN, GX, GY);
    life.place(&EATER, GX + 28 + K, GY + 15 + K);
    for _ in 0..4 * (K + 40) + 60 {
        life.step();
    }
    life
}

/// A glider gun; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut life = warm();
    let mut cycle = Vec::with_capacity(PERIOD);
    for _ in 0..PERIOD {
        let mut dots = Octants::new(META.cols, META.rows);
        let mut grid = Grid::new(META.cols, META.rows);
        for y in 0..dots.height() {
            for x in 0..dots.width() {
                if life.alive[(PAD + y) * life.w + PAD + x] == 1 {
                    dots.set(x as i64, y as i64);
                }
            }
        }
        // A character takes the age of its youngest cell, so births show.
        for cy in 0..META.rows {
            for cx in 0..META.cols {
                let m = dots.mask(cx, cy);
                if m == 0 {
                    continue;
                }
                let young = (0..8).filter(|b| m & (1 << b) != 0).map(|b| life.age[(PAD + cy * 4 + b / 2) * life.w + PAD + cx * 2 + b % 2]).min().unwrap_or(OLD);
                grid.set(cx as i64, cy as i64, dots.glyph(cx, cy), young);
            }
        }
        cycle.push(grid);
        life.step();
    }
    Box::new(GliderGun { cycle })
}

impl Piece for GliderGun {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let i = ((t * META.fps as f64 + 1e-6).floor() as usize) % PERIOD;
        grid.cells_mut().copy_from_slice(self.cycle[i].cells());
        grid.colors_mut().copy_from_slice(self.cycle[i].colors());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_field_repeats_every_thirty_generations() {
        let mut life = warm();
        let start = life.clone();
        for _ in 0..PERIOD {
            life.step();
        }
        assert!(start.alive == life.alive, "the gun and its eater keep a period of 30");
        let gliders = start.alive.iter().filter(|&&a| a == 1).count();
        // The gun is 36 cells, the eater 7; the lane holds several gliders of 5.
        assert!(gliders > 36 + 7 + 5 * 4, "the lane is full of gliders: {gliders} live cells");
    }
}
