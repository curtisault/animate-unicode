//! fern: Barnsley's fern, grown by the chaos game, its leaflets slimmed a
//! little so they stay apart at this size. The frond fills in from a sparse
//! young one, then sways. After ascii.rest's fern.ts (MIT, @bas3line).
//!
//! The maps, the growth and the sway are ascii.rest's. Where it reads the way
//! each cell's points run and draws a stroke, this plots the points
//! themselves in braille, eight dots a cell, so every leaflet and its
//! pinnules show. In colour the frond has depth: the leaflets on one side of
//! the stem are a bright green in front, those on the other a darker green
//! behind it, the young growth at the tip paler, the rachis a woody green
//! over a little brown soil.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The fern's meta.
pub static META: Meta = Meta {
    name: "fern",
    slug: "fern",
    category: Category::Nature,
    note: "barnsley's fern in braille points, front leaflets bright, back dark",
    cols: 44,
    rows: 30,
    fps: 20,
    charset: Charset::Extended,
    // Leaflets behind, in front, young; the rachis; the soil.
    palette: Some(&["#2a6834", "#5cbc4c", "#b0e478", "#7a8e3a", "#6a4a30"]),
    ground: Some("#0a0e0a"),
    ..Meta::DEFAULT
};

const BACK: u8 = 0;
const FRONT: u8 = 1;
const YOUNG: u8 = 2;
const RACHIS: u8 = 3;
const SOIL: u8 = 4;
/// Points when fully grown, seconds from a young frond to a full one, and the
/// leaflets' width against Barnsley's own.
const N: usize = 36000;
const GROW: f64 = 3.0;
const SLIM: f64 = 0.55;
/// Barnsley's maps: a, b, c, d, e, f, and the cumulative chance.
const MAPS: [[f64; 7]; 4] = [
    [0.0, 0.0, 0.0, 0.16, 0.0, 0.0, 0.01],
    [0.85, 0.04, -0.04, 0.85, 0.0, 1.6, 0.85],
    [0.2 * SLIM, -0.26, 0.23 * SLIM, 0.22, 0.0, 1.6, 0.925],
    [-0.15 * SLIM, 0.28, 0.26 * SLIM, 0.24, 0.0, 0.44, 1.0],
];
/// Centres the frond over its root.
const SHIFT: f64 = -0.25;

/// The frond: every point (x, y, its colour), the rachis's nodes, a count of points a dot, and the dots.
pub struct Fern {
    points: Vec<(f32, f32, u8)>,
    spine: Vec<(f64, f64)>,
    count: Vec<u16>,
    shade: Vec<u8>,
    dots: ColorDots,
}

/// A fern; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(17);
    let (mut x, mut y) = (0.0, 0.0);
    let mut points = Vec::with_capacity(N);
    // A point's side: the last leaflet map that made it, the left leaflets
    // in front and the right behind. Near the tip, it is young growth.
    let mut side = 0;
    for i in -20..N as i64 {
        let p = rand.next_f64();
        let j = MAPS.iter().position(|q| p < q[6]).unwrap_or(3);
        let [a, b, c, d, e, f, _] = MAPS[j];
        (x, y) = (a * x + b * y + e, c * x + d * y + f);
        if j != 1 {
            side = j;
        }
        if i >= 0 {
            let color = if y > 8.6 {
                YOUNG
            } else if side == 2 {
                FRONT
            } else if side == 3 {
                BACK
            } else {
                RACHIS
            };
            points.push(((x + SHIFT) as f32, y as f32, color));
        }
    }
    // The rachis: the stem's foot carried up the frond by the second map,
    // one node a step, to where it closes on the tip.
    let mut spine = vec![(SHIFT, 0.0)];
    let (mut sx, mut sy) = (0.0, 0.0);
    for _ in 0..60 {
        (sx, sy) = (0.85 * sx + 0.04 * sy, -0.04 * sx + 0.85 * sy + 1.6);
        spine.push((sx + SHIFT, sy));
    }
    let dots = ColorDots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(Fern { points, spine, count: vec![0; n], shade: vec![0; n], dots })
}

impl Piece for Fern {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width(), self.dots.height());
        // Growth: a sparse young frond that rises to full height as its points
        // arrive. The sway comes in as the growth ends; the frond bends more
        // the higher up it is.
        let g = (t / GROW).min(1.0);
        let e = g * g * (3.0 - 2.0 * g);
        let n = (N as f64 * (0.05 + 0.95 * e)) as usize;
        let scale = (rows - 3.8) / 10.0 * (0.8 + 0.2 * e);
        let calm = ((t - GROW * 0.5) / GROW).clamp(0.0, 1.0);
        let sway = calm * (0.07 * (TAU * t / 5.0).sin() + 0.02 * (TAU * t / 2.0 + 1.0).sin());
        let (root_r, root_c) = (rows - 1.5, cols / 2.0);
        let place = |u0: f64, v0: f64| {
            let hh = v0 / 10.0;
            let a = sway * hh * hh.sqrt() * 1.6 + 0.012 * calm * hh * (t * 4.0 + v0 * 3.0).sin();
            let (sa, ca) = a.sin_cos();
            ((root_c + (u0 * ca + v0 * sa) * 2.0 * scale) * 2.0, (root_r - (-u0 * sa + v0 * ca) * scale) * 4.0)
        };
        self.count.fill(0);
        for &(px, py, color) in &self.points[..n] {
            let (x, y) = place(f64::from(px), f64::from(py));
            if x < 0.0 || y < 0.0 || x >= w as f64 || y >= h as f64 {
                continue;
            }
            let k = y as usize * w + x as usize;
            // A dot takes the colour of the frontmost leaflet in it.
            if self.count[k] == 0 || color == FRONT || (color == YOUNG && self.shade[k] != FRONT) {
                self.shade[k] = color;
            }
            self.count[k] = self.count[k].saturating_add(1);
        }
        // A dot shows once enough points land on it; fewer while the frond is young.
        let need = if n < N / 3 { 1 } else { 2 };
        self.dots.clear();
        for (k, &c) in self.count.iter().enumerate() {
            if c >= need {
                self.dots.set((k % w) as i64, (k / w) as i64, self.shade[k]);
            }
        }
        // The rachis, a dot a step from the root to the tip, curving over.
        let pts: Vec<(f64, f64)> = self.spine.iter().map(|&(u0, v0)| place(u0, v0)).collect();
        for p in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (p[0], p[1]);
            let steps = (x1 - x0).abs().max((y1 - y0).abs()).ceil().max(1.0) as i64;
            for s in 0..=steps {
                let f = s as f64 / steps as f64;
                self.dots.set((x0 + (x1 - x0) * f).floor() as i64, (y0 + (y1 - y0) * f).floor() as i64, RACHIS);
            }
        }
        // A little soil round the root.
        let rc = pts[0].0.floor() as i64;
        for x in rc - 12..=rc + 12 {
            if (x - rc).abs() <= 8 || x % 2 == 0 {
                self.dots.set(x, h as i64 - 2, SOIL);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
