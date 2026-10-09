//! wind: gusts drawn as streamlines of a slowly bending flow, shown only
//! where a gust is passing. The streamlines are contours of one stream
//! function, so they bend together and never cross; the long gusts curl over
//! at their heads. After ascii.rest's wind.ts (MIT, @bas3line).
//!
//! The flow and the gusts are ascii.rest's; the lines are traced in braille,
//! a contour found every quarter row, so each streamline is a fine curve a
//! dot thick that fades from its tail to its head, and the longest gusts curl
//! over in a true spiral rather than a box-drawn hook. Autumn leaves ride the
//! gusts, tumbling as they go. In colour the air is a pale cyan, brightest at
//! each gust's head, and the leaves orange, red and gold.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::PI;

/// The wind's meta.
pub static META: Meta = Meta {
    name: "wind",
    slug: "wind",
    category: Category::Nature,
    note: "braille streamlines of gusts curling at their heads, leaves riding them",
    cols: 64,
    rows: 14,
    fps: 15,
    charset: Charset::Extended,
    // The air, tail to head; leaves.
    palette: Some(&["#3e5a74", "#7aa4c4", "#c4e4f6", "#e8862a", "#c8402c", "#e8c44a"]),
    ground: Some("#080b10"),
    ..Meta::DEFAULT
};

const AIR: Ramp = Ramp::new(0, 3);
const LEAVES: [u8; 3] = [3, 4, 5];
/// Rows between streamlines; gusts shorter than this many columns are
/// dropped; a gust this long may curl over at its head; and only the longest
/// few do.
const GAP: f64 = 2.0;
const MIN: f64 = 8.0;
const CURL: f64 = 16.0;
const MAX_CURLS: usize = 3;
/// The leaves riding the gusts.
const LEAF_COUNT: usize = 6;

fn hash(x: i64, y: i64, z: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1) ^ (z as u32).wrapping_mul(0x2545_f491);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn noise(x: f64, y: f64, z: f64) -> f64 {
    let (i, j, k) = (x.floor() as i64, y.floor() as i64, z.floor() as i64);
    let (u, v, w) = (ease(x - x.floor()), ease(y - y.floor()), ease(z - z.floor()));
    let n = |a, b, c| hash(i + a, j + b, k + c);
    let mix = |a: f64, b: f64, t: f64| a + (b - a) * t;
    mix(mix(mix(n(0, 0, 0), n(1, 0, 0), u), mix(n(0, 1, 0), n(1, 1, 0), u), v), mix(mix(n(0, 0, 1), n(1, 0, 1), u), mix(n(0, 1, 1), n(1, 1, 1), u), v), w)
}

/// The stream function, in rows: depth plus a slow bend that drifts right.
fn psi(x: f64, y: f64, t: f64) -> f64 {
    y + 5.0 * (noise(x * 0.04 - t * 0.05, y * 0.18, t * 0.02) + 0.35 * noise(x * 0.07 + 4.0, y * 0.4, t * 0.04))
}

/// One streamline: the dot row it crosses each dot column at (or -1), and
/// the dots where a gust shows along it.
#[derive(Clone)]
struct Line {
    row: Vec<i64>,
    on: Vec<bool>,
}

/// The wind: its streamlines this frame, and the dots.
pub struct Wind {
    lines: Vec<Line>,
    dots: ColorDots,
}

/// The wind; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    Box::new(Wind { lines: vec![], dots })
}

impl Piece for Wind {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let cols = META.cols as f64;
        // Each streamline k: where it is gusting, column by column. Gusts thin
        // out over the last 8 columns at either edge.
        let gusting = |k: usize, c: f64| noise(c * 0.09 - t * 0.5, k as f64 * 1.7, t * 0.05) * ((c + 1.0) / 8.0).min((cols - c) / 8.0).min(1.0) > 0.5;
        self.lines.clear();
        // Walk down each dot column a quarter row at a time; a streamline
        // crosses wherever the stream function passes a multiple of GAP.
        for x in 0..w {
            let c = (x as f64 + 0.5) / 2.0;
            let mut was = (psi(c, 0.0, t) / GAP).floor() as i64;
            for j in 0..h {
                let now = (psi(c, (j as f64 + 1.0) / 4.0, t) / GAP).floor() as i64;
                if now == was {
                    continue;
                }
                let k = now.max(was).max(0) as usize;
                was = now;
                if self.lines.len() <= k {
                    self.lines.resize(k + 1, Line { row: vec![-1; w], on: vec![false; w] });
                }
                let line = &mut self.lines[k];
                if line.row[x] < 0 {
                    line.row[x] = j as i64;
                    line.on[x] = gusting(k, c);
                }
            }
        }
        self.dots.clear();
        // Each gust: its span of dot columns. Short ones are dust, and one that
        // shows only a few dots is seen edge on or mostly out of the frame.
        let mut heads = vec![];
        for (k, line) in self.lines.iter().enumerate() {
            let mut x = 0;
            while x < w {
                if !line.on[x] {
                    x += 1;
                    continue;
                }
                let from = x;
                while x < w && line.on[x] {
                    x += 1;
                }
                let to = x - 1;
                let shown = (from..=to).filter(|&i| line.row[i] >= 0).count();
                if ((to - from + 1) as f64) < MIN * 2.0 || shown <= 10 {
                    continue;
                }
                // Faint at its tail, brightening to its head.
                for i in from..=to {
                    let y = line.row[i];
                    let f = (i - from) as f64 / (to - from).max(1) as f64;
                    if y >= 0 && dither(i, y as usize, 0.3 + f) {
                        self.dots.set(i as i64, y, AIR.at(f));
                    }
                }
                if (to - from + 1) as f64 >= CURL * 2.0 && line.row[to] >= 0 {
                    heads.push((to - from, k, to));
                }
            }
        }
        // The longest gusts curl over at their heads, up or down by turns: a
        // spiral that leaves the line going forward and winds in on itself.
        heads.sort_by_key(|h| std::cmp::Reverse(h.0));
        let mut curls: Vec<(i64, i64)> = vec![];
        for (_, k, to) in heads {
            let (x0, y0) = (to as i64, self.lines[k].row[to]);
            let d = if k % 2 == 1 { 1.0 } else { -1.0 };
            let r0 = 5.0;
            if curls.len() >= MAX_CURLS || y0 as f64 + d * 2.0 * r0 < 0.0 || y0 as f64 + d * 2.0 * r0 >= h as f64 || x0 + 6 >= w as i64 {
                continue;
            }
            // Two curls near each other read as a knot, so they keep well apart.
            if curls.iter().any(|&(cx, cy)| (cy - y0).abs() < 20 && (cx - x0).abs() < 32) {
                continue;
            }
            curls.push((x0, y0));
            let steps = 60;
            for s in 0..=steps {
                let phi = 1.6 * PI * s as f64 / steps as f64;
                let r = r0 * (1.0 - 0.3 * phi / PI);
                let (x, y) = (x0 as f64 + r * phi.sin(), y0 as f64 + d * (r0 - r * phi.cos()));
                self.dots.set(x.round() as i64, y.round() as i64, AIR.at(1.0));
            }
        }
        // Leaves ride along the streamlines, seen only while a gust carries them.
        for n in 0..LEAF_COUNT {
            let k = 1 + (n * 3) % self.lines.len().max(1);
            let speed = 6.0 + 4.0 * hash(n as i64, 1, 2);
            let x = ((hash(n as i64, 3, 4) * cols + speed * t).rem_euclid(cols + 10.0) - 5.0) * 2.0;
            let Some(line) = self.lines.get(k) else { continue };
            let xi = x.floor() as i64;
            if xi < 0 || xi >= w as i64 || !line.on[xi as usize] || line.row[xi as usize] < 0 {
                continue;
            }
            let (x, y) = (xi, line.row[xi as usize] - 1 + ((t * 3.0 + n as f64).sin() * 1.5).round() as i64);
            let turn = ((t * 6.0 + n as f64 * 1.7) as i64).rem_euclid(3);
            let shape: &[(i64, i64)] = match turn {
                0 => &[(0, 0), (1, 0), (0, 1)],
                1 => &[(0, 0), (1, 1)],
                _ => &[(0, 0), (0, 1), (1, 1)],
            };
            for &(dx, dy) in shape {
                self.dots.set(x + dx, y + dy, LEAVES[n % 3]);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
