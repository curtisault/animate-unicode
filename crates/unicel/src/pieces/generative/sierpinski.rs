//! sierpinski: the chaos game. A point jumps halfway to a corner picked at
//! random, over and over, and every spot it lands on stays lit; then the
//! triangle is erased along one unbroken path. After ascii.rest's
//! sierpinski.ts (MIT, @bas3line).
//!
//! In braille the 59×26 cells are a 118×104 picture of square dots, so the
//! triangle is truly equilateral and shows a level more of its holes. Each
//! dot takes the colour of the corner the point jumped toward when it landed
//! there, rose, cyan or gold, and the three half-size copies of the triangle
//! come out in the three colours: the figure shows how it is made.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// Sierpinski's meta.
pub static META: Meta = Meta {
    name: "sierpinski",
    slug: "sierpinski",
    category: Category::Generative,
    note: "the chaos game in braille, each dot coloured by the corner it chose",
    cols: 59,
    rows: 26,
    fps: 20,
    charset: Charset::Extended,
    // The apex, left and right corners; the corner marks.
    palette: Some(&["#ff7a9a", "#5ad2ff", "#ffcf5a", "#f4f4f8"]),
    ground: Some("#0a0a12"),
    ..Meta::DEFAULT
};

/// Seconds from one empty frame to the next; of plotting, slow at first and
/// then a flood; the finished triangle stays until HOLD, and has been erased
/// along its path by CLEAR. Play opens on the finished triangle.
const CYCLE: f64 = 9.6;
const FILL: f64 = 6.0;
const HOLD: f64 = 7.3;
const CLEAR: f64 = 9.3;
const START: f64 = 6.1;
/// Throws a cycle, how fast the plotting speeds up, and throws a second underneath the flood.
const POINTS: usize = 40_000;
const RATE: f64 = 1.0;
const STEADY: f64 = 60.0;
/// Levels of the figure a dot is tested against: the smallest triangle is
/// then about a dot and a half tall.
const DEPTH: usize = 6;
const MARK: u8 = 3;

/// The game: its corners, each dot's place along the erasing path (-1 off
/// the figure) and the third of the triangle it is in, the throw that first
/// lit each dot and its corner, and the dots.
pub struct Sierpinski {
    corners: [(f64, f64); 3],
    rank: Vec<f32>,
    third: Vec<u8>,
    first: Vec<u32>,
    by: Vec<u8>,
    cycle: i64,
    dots: ColorDots,
}

impl Sierpinski {
    /// Where a point sits in the figure: -1 in a hole, else its place along
    /// the arrowhead path that runs from the left corner over the apex to the
    /// right, visiting every sub-triangle in turn.
    fn place(&self, x: f64, y: f64) -> f64 {
        let ((ax, ay), (lx, _), (rx, by)) = (self.corners[0], self.corners[1], self.corners[2]);
        let half = (rx - lx) / 2.0;
        let (v, s) = ((y - ay) / (by - ay), (x - ax) / half);
        let mut w = [1.0 - v, (v - s) / 2.0, (v + s) / 2.0];
        if w.iter().any(|&c| c < 0.0) {
            return -1.0;
        }
        let (mut p, mut at, mut step) = ([1usize, 0, 2], 0.0, 1.0);
        for _ in 0..DEPTH {
            let Some(k) = w.iter().position(|&c| c >= 0.5) else { return -1.0 };
            for (j, c) in w.iter_mut().enumerate() {
                *c = if j == k { 2.0 * *c - 1.0 } else { 2.0 * *c };
            }
            let i = p.iter().position(|&c| c == k).unwrap();
            step /= 3.0;
            at += i as f64 * step;
            p = match i {
                0 => [p[0], p[2], p[1]],
                2 => [p[1], p[0], p[2]],
                _ => p,
            };
        }
        at
    }

    /// Which half-size copy (0 apex, 1 left, 2 right) a point is in.
    fn third(&self, x: f64, y: f64) -> u8 {
        let ((ax, ay), (lx, _), (rx, by)) = (self.corners[0], self.corners[1], self.corners[2]);
        let (v, s) = ((y - ay) / (by - ay), (x - ax) / ((rx - lx) / 2.0));
        let w = [1.0 - v, (v - s) / 2.0, (v + s) / 2.0];
        (0..3).max_by(|&a, &b| w[a].total_cmp(&w[b])).unwrap_or(0) as u8
    }

    /// The game for cycle k: which throw first lit each dot, and toward which corner.
    fn build(&mut self, k: i64) {
        let mut rand = Mulberry32::new((k * 977 + 31) as u32);
        let w = self.dots.width();
        // Anything the game misses lights on the last throw, in its third's colour.
        self.first.fill(POINTS as u32 - 1);
        self.by.copy_from_slice(&self.third);
        let (mut x, mut y) = self.corners[(rand.next_f64() * 3.0) as usize];
        for n in 0..POINTS {
            let c = (rand.next_f64() * 3.0) as usize;
            let (cx, cy) = self.corners[c];
            x = (x + cx) / 2.0;
            y = (y + cy) / 2.0;
            let at = y.floor() as usize * w + x.floor() as usize;
            if self.first[at] > n as u32 {
                self.first[at] = n as u32;
                self.by[at] = c as u8;
            }
        }
    }
}

/// Sierpinski; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let (w, h) = (dots.width() as f64, dots.height() as f64);
    // An equilateral triangle on square dots.
    let (top, base) = (3.0, h - 3.0);
    let half = (base - top) / 3f64.sqrt();
    let corners = [(w / 2.0, top), (w / 2.0 - half, base), (w / 2.0 + half, base)];
    let n = dots.width() * dots.height();
    let mut s = Sierpinski { corners, rank: vec![-1.0; n], third: vec![0; n], first: vec![0; n], by: vec![0; n], cycle: -1, dots };
    // A dot belongs to the figure when two of its 16 sample points do; its
    // rank is the earliest of their places along the path.
    let iw = s.dots.width();
    for k in 0..n {
        let (qx, qy) = ((k % iw) as f64, (k / iw) as f64);
        let places: Vec<f64> = (0..16).map(|i| s.place(qx + ((i & 3) as f64 + 0.5) / 4.0, qy + ((i >> 2) as f64 + 0.5) / 4.0)).filter(|&a| a >= 0.0).collect();
        if places.len() >= 2 {
            s.rank[k] = places.iter().copied().fold(2.0, f64::min) as f32;
        }
        s.third[k] = s.third(qx + 0.5, qy + 0.5);
    }
    Box::new(s)
}

impl Piece for Sierpinski {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let u = t + START;
        let k = (u / CYCLE).floor() as i64;
        if k != self.cycle {
            self.cycle = k;
            self.build(k);
        }
        let tau = u - k as f64 * CYCLE;
        let scale = (POINTS as f64 - STEADY * FILL) / ((RATE * FILL).exp() - 1.0);
        let shown = if tau >= FILL { POINTS as f64 } else { (STEADY * tau + scale * ((RATE * tau).exp() - 1.0)).floor() };
        let gone = if tau <= HOLD { 0.0 } else { (tau - HOLD) / (CLEAR - HOLD) };
        let w = self.dots.width();
        self.dots.clear();
        for (i, &r) in self.rank.iter().enumerate() {
            if f64::from(r) >= gone && r >= 0.0 && f64::from(self.first[i]) < shown {
                self.dots.set((i % w) as i64, (i / w) as i64, self.by[i]);
            }
        }
        // The corners are marked while the game is played and once it is over.
        if !(FILL..CLEAR).contains(&tau) {
            for (c, &(x, y)) in self.corners.iter().enumerate() {
                let (x, y) = (x.round() as i64 + [0, 1, -2][c], y.round() as i64 + if c == 0 { 1 } else { -2 });
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    self.dots.set(x + dx, y + dy, MARK);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
