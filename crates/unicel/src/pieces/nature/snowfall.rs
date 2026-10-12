//! snowfall: snow drifting down in three depths on a light wind, swaying as
//! it falls. Flakes settle where they land, building a drift in the fence's
//! lee, a ridge along each rail and a cap on every post. After ascii.rest's
//! snowfall.ts (MIT, @bas3line).
//!
//! The fall, the wind and the settling are ascii.rest's. The flakes are
//! braille and sized by their depth: a single dim dot far off, two dots
//! between, a cluster of four close up, so the near snow sweeps past bigger
//! and faster. The drift, the rails' ridges and the posts' caps keep their
//! eighth blocks, so their surfaces rise an eighth of a row at a time. In
//! colour the snow is white over a blue-shadowed body, the far flakes grey
//! with distance, the fence a dark brown.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// Snowfall's meta.
pub static META: Meta = Meta {
    name: "snowfall",
    slug: "snowfall",
    category: Category::Nature,
    note: "braille snow sized by depth, building white drifts on a fence",
    cols: 64,
    rows: 22,
    fps: 20,
    charset: Charset::Extended,
    // Flakes far, mid, near; the snow's surface and its body; the fence.
    palette: Some(&["#727c96", "#bcc6dc", "#ffffff", "#f4f8ff", "#b4c4de", "#6a4a32"]),
    ground: Some("#0a0d14"),
    ..Meta::DEFAULT
};

const FLAKE: [u8; 3] = [0, 1, 2];
const SNOW: u8 = 3;
const BODY: u8 = 4;
const FENCE: u8 = 5;
const EIGHTHS: [char; 9] = [' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
/// On paper the drift's surface is a line, by where it crosses a cell.
const LINE: [char; 3] = ['▔', '─', '▁'];
/// The fence posts' columns, the posts' top row, the rails' rows, and a bare
/// rail's thickness in eighths of a row.
const POSTS: [usize; 4] = [9, 20, 31, 42];
const TOP: usize = 10;
const RAILS: [usize; 2] = [11, 14];
const RAIL: usize = 2;
/// Each depth: flakes, fall speed and drift in cells a second, and sway.
const LAYERS: [(usize, f64, f64, f64); 3] = [(44, 1.1, 0.6, 0.5), (30, 1.8, 1.1, 1.0), (13, 2.8, 1.8, 1.6)];
/// The dots each depth's flake covers, from its place.
const SHAPES: [&[(i64, i64)]; 3] = [&[(0, 0)], &[(0, 0), (1, 0)], &[(0, 0), (1, 0), (0, 1), (1, 1)]];

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// A flake: its depth, where it is, its sway's phase, and its weight.
struct Flake {
    li: usize,
    x: f64,
    y: f64,
    ph: f64,
    w: f64,
}

/// The snow: the ground's rolling snow and the fresh snow on it, the wind
/// along the ground, the snow on each rail and post, the flakes, the
/// generator, the last frame's time, and the dots.
pub struct Snowfall {
    ground: Vec<f64>,
    extra: Vec<f64>,
    wind: Vec<f64>,
    rail: [Vec<f64>; 2],
    cap: [f64; 4],
    flakes: Vec<Flake>,
    rand: Mulberry32,
    last: f64,
    dots: ColorDots,
}

fn in_fence(c: usize) -> bool {
    c > POSTS[0] && c < POSTS[POSTS.len() - 1]
}

impl Snowfall {
    fn spawn(&mut self, i: usize, top: bool) {
        let cols = META.cols as f64;
        let rows = META.rows as f64;
        let f = &mut self.flakes[i];
        f.x = self.rand.next_f64() * cols;
        f.y = if top { -self.rand.next_f64() * 2.0 } else { self.rand.next_f64() * rows };
        #[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the sway is theirs
        let ph = self.rand.next_f64() * 6.283;
        f.ph = ph;
        f.w = 0.8 + self.rand.next_f64() * 0.45;
    }

    fn step(&mut self, dt: f64, t: f64) {
        let (cols, rows) = (META.cols, META.rows as f64);
        for i in 0..self.flakes.len() {
            let f = &mut self.flakes[i];
            let (_, v, wind, sway) = LAYERS[f.li];
            let y0 = f.y;
            f.y += v * f.w * dt;
            f.x = (f.x + (wind + sway * (t * f.w * 1.3 + f.ph).cos()) * dt).rem_euclid(cols as f64);
            let (li, x, y) = (f.li, f.x, f.y);
            let c = (x as usize).min(cols - 1);
            // Flakes at the fence's depth may come to rest on a rail or a post top.
            if li == 1 {
                if let Some(k) = RAILS.iter().position(|&r| y0 < r as f64 + 0.5 && y >= r as f64 + 0.5) {
                    if in_fence(c) && !POSTS.contains(&c) && self.rand.next_f64() < 0.7 {
                        for (d, a) in [(-1i64, 0.3), (0, 0.6), (1, 0.3)] {
                            let cc = (c as i64 + d) as usize;
                            self.rail[k][cc] = (self.rail[k][cc] + a).min(5.0);
                        }
                        self.spawn(i, true);
                        continue;
                    }
                }
                if let Some(p) = POSTS.iter().position(|&pc| pc == c) {
                    if y0 < TOP as f64 && y >= TOP as f64 {
                        self.cap[p] = (self.cap[p] + 1.0).min(13.0);
                        self.spawn(i, true);
                        continue;
                    }
                }
            }
            if y < rows - self.ground[c] - self.extra[c] {
                continue;
            }
            // The two nearer depths settle where they land, more where the wind drops them.
            if li > 0 {
                self.extra[c] += 0.3 * self.wind[c];
            }
            self.spawn(i, true);
        }
        // Fresh snow evens out between neighbours and packs down slowly.
        let next: Vec<f64> = (0..cols)
            .map(|c| {
                let e = self.extra[c];
                let (l, r) = (if c > 0 { self.extra[c - 1] } else { e }, if c + 1 < cols { self.extra[c + 1] } else { e });
                e + 1.5 * dt * (l + r - 2.0 * e)
            })
            .collect();
        for (e, n) in self.extra.iter_mut().zip(next) {
            *e = n * (1.0 - 0.02 * dt);
        }
        for r in &mut self.rail {
            for v in r.iter_mut() {
                *v -= *v * 0.004 * dt;
            }
        }
        for c in &mut self.cap {
            *c -= *c * 0.01 * dt;
        }
    }
}

/// Snowfall; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let cols = META.cols;
    let mut rand = Mulberry32::new(7);
    // The ground's snow in rows at each column: rolling, swept into a long
    // drift in the fence's lee that ends in a steep face.
    let ground = (0..cols)
        .map(|c| {
            let c = c as f64;
            let lee = 3.6 * smooth(36.0, 55.0, c) * (1.0 - smooth(55.0, 59.0, c));
            1.0 + 0.55 * (c * 0.19 + 1.3).sin() + 0.3 * (c * 0.47 + 0.4).sin() + lee
        })
        .collect();
    let wind = (0..cols).map(|c| 0.7 + 1.6 * (-((c as f64 - 52.0) / 7.0).powi(2)).exp()).collect();
    let rail = [0, 1].map(|k| (0..cols).map(|c| 2.0 + 0.7 * (c as f64 * 0.45 + f64::from(k) * 2.0).sin() + 0.4 * rand.next_f64()).collect());
    let mut snow =
        Snowfall { ground, extra: vec![0.3; cols], wind, rail, cap: [7.0; 4], flakes: vec![], rand, last: 0.0, dots: ColorDots::new(META.cols, META.rows) };
    for (li, &(n, ..)) in LAYERS.iter().enumerate() {
        for _ in 0..n {
            snow.flakes.push(Flake { li, x: 0.0, y: 0.0, ph: 0.0, w: 0.0 });
            let i = snow.flakes.len() - 1;
            snow.spawn(i, false);
        }
    }
    // Twenty seconds of snow before the first frame.
    for i in 0..400 {
        snow.step(0.05, f64::from(i) * 0.05);
    }
    Box::new(snow)
}

impl Piece for Snowfall {
    fn frame(&mut self, time: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols, META.rows);
        let mut dt = (time - self.last).clamp(0.0, 0.1);
        self.last = time;
        while dt > 1e-9 {
            self.step(dt.min(0.05), 20.0 + time);
            dt -= 0.05;
        }
        // Flakes first: the fence and the snow stand in front of them, so a
        // flake shows only in open air.
        self.dots.clear();
        for f in &self.flakes {
            let (x, y) = ((f.x * 2.0).floor() as i64, (f.y * 4.0).floor() as i64);
            for &(dx, dy) in SHAPES[f.li] {
                self.dots.set(x + dx, y + dy, FLAKE[f.li]);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The fence: posts, rails thickened by the snow lying on them, a cap on each post.
        for (k, &r) in RAILS.iter().enumerate() {
            for c in POSTS[0] + 1..POSTS[POSTS.len() - 1] {
                grid.set(c as i64, r as i64, EIGHTHS[(RAIL + self.rail[k][c] as usize).min(8)], SNOW);
            }
        }
        for (p, &c) in POSTS.iter().enumerate() {
            for r in TOP..rows {
                grid.set(c as i64, r as i64, '┃', FENCE);
            }
            let n = self.cap[p] as usize;
            grid.set(c as i64, TOP as i64 - 1, EIGHTHS[(1 + n / 2).min(8)], SNOW);
            if n > 8 {
                grid.set(c as i64 - 1, TOP as i64 - 1, EIGHTHS[n - 8], SNOW);
                grid.set(c as i64 + 1, TOP as i64 - 1, EIGHTHS[n - 8], SNOW);
            }
        }
        // The drift: on a dark ground, solid under an eighth-block surface; on
        // paper, a line over a light fill, with an edge down any step of two rows.
        let top: Vec<f64> = (0..cols).map(|c| rows as f64 - self.ground[c] - self.extra[c]).collect();
        for (c, &s) in top.iter().enumerate() {
            let r = s.floor();
            let f = s - r;
            for rr in r as usize + 1..rows {
                grid.set(c as i64, rr as i64, if env.paper { '░' } else { '█' }, BODY);
            }
            let ch = if env.paper { LINE[((f * 3.0) as usize).min(2)] } else { EIGHTHS[(((1.0 - f) * 8.0).round() as usize).max(1)] };
            grid.set(c as i64, r as i64, ch, SNOW);
        }
        if env.paper {
            for c in 0..cols - 1 {
                let (a, b) = (top[c].floor() as i64, top[c + 1].floor() as i64);
                for r in a + 1..b {
                    grid.set(c as i64 + 1, r, '▏', SNOW);
                }
                for r in b + 1..a {
                    grid.set(c as i64, r, '▕', SNOW);
                }
            }
        }
    }
}
