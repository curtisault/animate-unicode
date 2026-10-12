//! skeleton: the loading placeholder for a card. An avatar, a name and a
//! byline, an image and lines of text in light shade, with a shimmer
//! sweeping across them while the content is on its way. After ascii.rest's
//! skeleton.ts (MIT, @bas3line).
//!
//! The shimmer is colour as well as shade: the placeholders are a cool grey
//! that the passing band lifts through steps of light toward a pale
//! blue-white, as a page's skeleton shimmers, and the card's rounded border
//! sits dim behind it all.

use crate::{Category, Env, Grid, Meta, Options, Piece, Ramp};

/// The skeleton's meta.
pub static META: Meta = Meta {
    name: "skeleton",
    slug: "skeleton",
    category: Category::Ui,
    note: "a loading card with a shimmer of colour sweeping across it",
    cols: 52,
    rows: 23,
    fps: 24,
    options: Some(r#"{"lines": 3, "image": true, "avatar": true}"#),
    // The placeholder grey lifting to the shimmer's core; the border.
    palette: Some(&["#3a3f4a", "#4a5262", "#64708a", "#8aa0c4", "#c8dcf4", "#4a4f5a"]),
    ground: Some("#121318"),
    ..Meta::DEFAULT
};

/// Seconds from one sweep to the next; where in the sweep play begins;
/// columns the band leans per row; the content's first and last column.
const PERIOD: f64 = 1.8;
const START: f64 = 0.3;
const SLANT: f64 = 0.8;
const LEFT: i64 = 3;
const RIGHT: i64 = 48;
const LIGHT: Ramp = Ramp::new(0, 5);
const BORDER: u8 = 5;

/// A block of the card: its height in rows, and its runs of (row, first column, last column).
type Block = (i64, Vec<(i64, i64, i64)>);

/// The card: its placeholder cells, the base drawing, and its middle row.
pub struct Skeleton {
    mask: Vec<bool>,
    base: Grid,
    mid: f64,
}

/// A skeleton; reads `lines`, `image` and `avatar`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let full = RIGHT - LEFT + 1;
    // Each block is a height and runs of (row, first column, last column).
    let mut blocks: Vec<Block> = vec![];
    if options.bool("avatar", true) {
        // A circle six rows tall, kept where a cell is at least half inside.
        let mut runs = vec![];
        let (ax, r_) = (LEFT as f64 + 6.0, 3.0);
        for r in 0..6 {
            let (mut c0, mut c1) = (-1, -1);
            for c in LEFT..LEFT + 12 {
                let mut hit = 0;
                for j in 0..4 {
                    for i in 0..4 {
                        let x = (c as f64 + 0.125 + i as f64 * 0.25 - ax) / 2.0;
                        let y = r as f64 + 0.125 + j as f64 * 0.25 - r_;
                        if x * x + y * y < r_ * r_ {
                            hit += 1;
                        }
                    }
                }
                if hit >= 8 {
                    if c0 < 0 {
                        c0 = c;
                    }
                    c1 = c;
                }
            }
            runs.push((r, c0, c1));
        }
        // The name and byline sit level with the avatar's shoulders.
        runs.extend([(1, LEFT + 15, LEFT + 32), (4, LEFT + 15, LEFT + 26)]);
        blocks.push((6, runs));
    } else {
        blocks.push((3, vec![(0, LEFT, LEFT + 17), (2, LEFT, LEFT + 10)]));
    }
    if options.bool("image", true) {
        blocks.push((6, (0..6).map(|r| (r, LEFT, RIGHT)).collect()));
    }
    // As many lines as fit, the last one cut short like the end of a paragraph.
    let used: i64 = blocks.iter().map(|b| b.0 + 1).sum();
    let n = options.f64("lines", 3.0).floor().max(0.0).min(((rows - 4 - used + 1) / 2) as f64) as i64;
    if n > 0 {
        let runs = (0..n)
            .map(|i| {
                let f = if i == n - 1 {
                    if n == 1 {
                        0.72
                    } else {
                        0.58
                    }
                } else {
                    [1.0, 0.91, 0.96, 0.87][i as usize % 4]
                };
                (i * 2, LEFT, LEFT + (full as f64 * f).round() as i64 - 1)
            })
            .collect();
        blocks.push((n * 2 - 1, runs));
    }
    // The card is as tall as its blocks and sits in the middle of the frame.
    let inner: i64 = blocks.iter().map(|b| b.0).sum::<i64>() + blocks.len() as i64 - 1;
    let h = inner + 4;
    let top = (rows - h) / 2;
    let mut mask = vec![false; (cols * rows) as usize];
    let mut y = top + 2;
    for (bh, runs) in &blocks {
        for &(r, c0, c1) in runs {
            for c in c0..=c1 {
                mask[((y + r) * cols + c) as usize] = true;
            }
        }
        y += bh + 1;
    }
    // The card's rounded border never changes.
    let mut base = Grid::new(META.cols, META.rows);
    let bot = top + h - 1;
    for c in 1..cols - 1 {
        base.set(c, top, '─', BORDER);
        base.set(c, bot, '─', BORDER);
    }
    for r in top + 1..bot {
        base.set(0, r, '│', BORDER);
        base.set(cols - 1, r, '│', BORDER);
    }
    for (c, r, ch) in [(0, top, '╭'), (cols - 1, top, '╮'), (0, bot, '╰'), (cols - 1, bot, '╯')] {
        base.set(c, r, ch, BORDER);
    }
    Box::new(Skeleton { mask, base, mid: top as f64 + h as f64 / 2.0 })
}

impl Piece for Skeleton {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        grid.cells_mut().copy_from_slice(self.base.cells());
        grid.colors_mut().copy_from_slice(self.base.colors());
        let cols = META.cols;
        // Base, edge of the band, its core. On a dark page the band brightens
        // the shade; on paper it lifts it toward the page.
        let shade = if env.paper { ['▒', '░', '·'] } else { ['░', '▒', '▓'] };
        // The band runs from just off the left edge to just off the right, so the wrap is unseen.
        let x = -12.0 + (t / PERIOD + START).rem_euclid(1.0) * (cols as f64 + 24.0);
        for (k, &on) in self.mask.iter().enumerate() {
            if !on {
                continue;
            }
            let (r, c) = ((k / cols) as f64, (k % cols) as f64);
            let d = (c + 0.5 - x - (self.mid - r) * SLANT).abs();
            let g = if d < 0.5 {
                2
            } else if d < 4.5 {
                1
            } else {
                0
            };
            // The colour fades in over a wider band than the glyphs, so the sweep is soft.
            let light = (1.0 - d / 8.0).max(0.0);
            grid.set((k % cols) as i64, (k / cols) as i64, shade[g], LIGHT.at(light));
        }
    }
}
