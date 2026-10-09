//! ferris wheel: a wheel of eight spokes turning slowly on a braced A-frame,
//! its cabins hanging upright from the rim and passing the boarding deck.
//! After ascii.rest's ferris-wheel.ts (MIT, @bas3line).
//!
//! The wheel and its frame are ascii.rest's, drawn in braille: the rim a
//! true circle, the spokes dotted lines from the hub, the A-frame's legs and
//! rungs fine strokes, so the wheel turns smoothly rather than in the steps
//! of a traced character rim. The cabins keep their box-drawn shape, hanging
//! upright. At night: the steel is pale blue-white, the bulbs between the
//! cabins chase warm yellow round the rim, and each cabin is its own colour.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The ferris wheel's meta.
pub static META: Meta = Meta {
    name: "ferris wheel",
    slug: "ferris-wheel",
    category: Category::Objects,
    note: "a braille ferris wheel at night, bulbs chasing round, coloured cabins",
    cols: 64,
    rows: 26,
    fps: 10,
    charset: Charset::Extended,
    // The rim; the spokes; the frame; a lit bulb; an unlit one; the hub; the
    // deck; then the cabins' eight colours.
    palette: Some(&[
        "#c8d4ec", "#5c6680", "#8a90a4", "#ffe070", "#4a3e24", "#eef0fa", "#5a5e6a", "#ff5a5a", "#ff9a3a", "#f4d64a", "#6ad86a", "#4ad0c8", "#5a8cff",
        "#a87aff", "#ff7ac0",
    ]),
    ground: Some("#06070e"),
    ..Meta::DEFAULT
};

const RIM: u8 = 0;
const SPOKE: u8 = 1;
const FRAME: u8 = 2;
const LIT: u8 = 3;
const DARK: u8 = 4;
const HUB: u8 = 5;
const DECK: u8 = 6;
const CABINS: u8 = 7;
/// Spokes (a cabin at the end of each), the rim's radius in rows, seconds a
/// revolution, and the bulbs round the rim.
const N: usize = 8;
const R: f64 = 10.0;
const TURN: f64 = 30.0;
const BULBS: usize = 24;

/// Dots along a line between two points in cells, every `gap` dots.
fn line(dots: &mut ColorDots, (x0, y0): (f64, f64), (x1, y1): (f64, f64), gap: usize, color: u8) {
    let (a, b) = ((x0 * 2.0, y0 * 4.0), (x1 * 2.0, y1 * 4.0));
    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs()).ceil().max(1.0) as usize;
    for s in (0..=n).step_by(gap) {
        let f = s as f64 / n as f64;
        dots.set((a.0 + (b.0 - a.0) * f).floor() as i64, (a.1 + (b.1 - a.1) * f).floor() as i64, color);
    }
}

/// The wheel's dots.
pub struct FerrisWheel {
    dots: ColorDots,
}

/// A ferris wheel; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(FerrisWheel { dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for FerrisWheel {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (cx, cy) = (32.5f64, 11.5f64); // the hub, in cells
        let ground = rows - 1.0;
        let a = TAU * t / TURN;
        self.dots.clear();
        // The A-frame: a pair of legs a side from the axle down to the ground,
        // with a rung between each pair every three rows.
        let top = cy.ceil();
        for s in [-1.0, 1.0] {
            let leg = |d: f64| (cx + s * d, top);
            let foot = |d: f64| (cx + s * (d + 0.9 * (ground - top)), ground);
            line(&mut self.dots, leg(1.2), foot(1.2), 1, FRAME);
            line(&mut self.dots, leg(3.8), foot(3.8), 1, FRAME);
            let mut r = top + 2.5;
            while r < ground {
                let d = 1.2 + 0.9 * (r - top);
                line(&mut self.dots, (cx + s * d, r), (cx + s * (d + 2.6), r), 1, FRAME);
                r += 3.0;
            }
        }
        // The rim, a true circle, and the spokes dotted from the hub to just short of it.
        for k in 0..720 {
            let p = f64::from(k) / 720.0 * TAU;
            self.dots.set(((cx + 2.0 * R * p.cos()) * 2.0) as i64, ((cy + R * p.sin()) * 4.0) as i64, RIM);
        }
        let mut ends = vec![];
        for k in 0..N {
            let th = a + k as f64 * TAU / N as f64;
            line(&mut self.dots, (cx + 3.0 * th.cos(), cy + 1.5 * th.sin()), (cx + (2.0 * R - 1.0) * th.cos(), cy + (R - 0.5) * th.sin()), 3, SPOKE);
            ends.push((cx + 2.0 * R * th.cos(), cy + R * th.sin()));
        }
        // Bulbs on the rim between the cabins, every other one lit in turn.
        let blink = (t / 0.6).floor() as usize % 2;
        for k in 0..BULBS {
            if k % (BULBS / N) == 0 {
                continue;
            }
            let th = a + k as f64 * TAU / BULBS as f64 + PI / BULBS as f64;
            let (x, y) = (((cx + 2.0 * R * th.cos()) * 2.0) as i64, ((cy + R * th.sin()) * 4.0) as i64);
            let color = if (k + blink) % 2 == 1 { LIT } else { DARK };
            for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                self.dots.set(x + dx - 1, y + dy - 1, color);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        grid.text_at((cx - 1.5) as i64, cy as i64, "(●)", HUB);
        // Cabins hang from each spoke end, always upright.
        for (k, &(x, y)) in ends.iter().enumerate() {
            let (c, r) = (x.floor() as i64, y.floor() as i64);
            let color = CABINS + k as u8;
            grid.set(c, r, '●', color);
            grid.text_at(c - 2, r + 1, "╭─┴─╮", color);
            grid.text_at(c - 2, r + 2, "╰───╯", color);
        }
        // The boarding deck under the lowest cabin, and the ground.
        grid.text_at((cx - 7.5) as i64, ground as i64 - 1, &"▄".repeat(15), DECK);
        grid.text_at(0, ground as i64, &"▀".repeat(cols as usize), DECK);
    }
}
