//! bouncing balls: rubber balls dropping onto a floor and knocking into each
//! other, squashing on every landing; a ball that comes to rest crouches and
//! leaps. After ascii.rest's bouncing-balls.ts (MIT, @bas3line).
//!
//! The physics is ascii.rest's, step for step, from its own seeded generator.
//! Each ball is drawn per octant dot, eight a cell, as a lit rubber sphere in
//! its own colour, shaded by ordered dithering with a highlight set in from
//! the rim, and squashed to keep its area; its shadow on the floor narrows
//! and pales as it rises.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// The bouncing balls' meta.
pub static META: Meta = Meta {
    name: "bouncing balls",
    slug: "bouncing-balls",
    category: Category::Physics,
    note: "rubber balls in octants, each its colour, squashing on landing",
    cols: 64,
    rows: 22,
    fps: 30,
    charset: Charset::Extended,
    // Five balls of four shades each, dimmest first; the floor; the shadow.
    palette: Some(&[
        "#5a1414", "#a32424", "#e04a3a", "#ffb0a0", "#143a5a", "#2464a3", "#3a92e0", "#a8d8ff", "#5a4a10", "#a3861a", "#e0bc2a", "#fff0a0", "#14502a",
        "#249048", "#3ac868", "#b0f4c0", "#40145a", "#7224a3", "#a63ae0", "#e0b0ff", "#9aa0aa", "#3a3e46",
    ]),
    ground: Some("#0c0d12"),
    options: Some(r#"{"balls": 3}"#),
    ..Meta::DEFAULT
};

/// Gravity, rows a second squared; the physics step; the contact stiffness
/// (a ball of radius r rings at STIFF / r).
const G: f64 = 80.0;
const H: f64 = 1.0 / 900.0;
const STIFF: f64 = 120.0;
/// Radii, in rows (a column is half a row), and how much bounce each keeps.
const SIZES: [f64; 5] = [3.0, 2.3, 1.7, 2.6, 2.0];
const BOUNCE: [f64; 5] = [0.84, 0.82, 0.8, 0.83, 0.81];
/// Seconds between the balls' first leaps, and played before the first frame.
const SPREAD: f64 = 2.1;
const WARM: f64 = 9.1;
const FLOOR_INK: u8 = 20;
const SHADOW: u8 = 21;

/// One ball, in rows: its radius, spring and damping, place, speed, how long
/// it has rested, its crouch (if crouching) and how far it is squashed.
struct Ball {
    r: f64,
    k: f64,
    c: f64,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    rest: f64,
    crouch: Option<f64>,
    sq: f64,
}

/// The balls, their generator, the clock, and the dots.
pub struct BouncingBalls {
    bs: Vec<Ball>,
    rand: Mulberry32,
    last: f64,
    dots: ColorOctants,
}

/// The floor's width in rows, and the row the floor line sits on.
const W: f64 = META.cols as f64 / 2.0;
const FLOOR: f64 = (META.rows - 1) as f64;

impl BouncingBalls {
    fn launch(&mut self, i: usize) {
        let (a, b) = (self.rand.next_f64(), self.rand.next_f64());
        let ball = &mut self.bs[i];
        // High enough to fill the frame, never so high the ball leaves it.
        ball.vy = (2.0 * G * (FLOOR - 1.5 - 2.0 * ball.r) * (0.75 + 0.25 * a)).sqrt();
        ball.vx = (W / 2.0 - ball.x) * 0.3 + (b - 0.5) * 10.0;
        ball.y = ball.r; // spring straight back to round
        ball.crouch = None;
        ball.rest = 0.0;
    }

    fn step(&mut self) {
        for i in 0..self.bs.len() {
            let b = &mut self.bs[i];
            if let Some(c) = b.crouch {
                // Settled: crouch into the floor for a moment, then spring up.
                let c = c + H;
                b.crouch = Some(c);
                let u = (c / 0.45).clamp(0.0, 1.0);
                b.sq = 0.3 * b.r * u * u * (3.0 - 2.0 * u);
                b.y = b.r - b.sq;
                b.vx *= 1.0 - 6.0 * H;
                if u >= 1.0 {
                    self.launch(i);
                }
                continue;
            }
            b.vy -= G * H;
            let d = b.r - b.y;
            if d > 0.0 {
                b.vy += (b.k * d - b.c * b.vy).max(0.0) * H;
                b.vx *= 1.0 - 0.8 * H;
                b.rest += H;
                if b.rest > 0.3 && b.crouch.is_none() {
                    b.crouch = Some(0.0);
                }
            } else {
                b.rest = 0.0;
            }
            b.sq = d.max(0.0);
            // A knock from below never sends a ball out of the top of the frame.
            b.vy = b.vy.min((2.0 * G * (FLOOR - 1.2 - b.r - b.y).max(0.0)).sqrt());
            b.x += b.vx * H;
            b.y += b.vy * H;
            if b.x < b.r {
                b.x = b.r;
                b.vx = b.vx.abs() * 0.85;
            }
            if b.x > W - b.r {
                b.x = W - b.r;
                b.vx = -b.vx.abs() * 0.85;
            }
        }
        // Balls that touch trade momentum along the line between their centres.
        let n = self.bs.len();
        for i in 0..n {
            for j in i + 1..n {
                let (lo, hi) = self.bs.split_at_mut(j);
                let (a, b) = (&mut lo[i], &mut hi[0]);
                let (dx, dy) = (b.x - a.x, b.y - a.y);
                let dist = (dx * dx + dy * dy).sqrt();
                let gap = a.r + b.r - dist;
                if gap <= 0.0 || dist == 0.0 {
                    continue;
                }
                let (nx, ny) = (dx / dist, dy / dist);
                let (ma, mb) = (a.r * a.r, b.r * b.r);
                let vn = (b.vx - a.vx) * nx + (b.vy - a.vy) * ny;
                if vn < 0.0 {
                    let jm = -1.85 * vn / (1.0 / ma + 1.0 / mb);
                    a.vx -= jm * nx / ma;
                    a.vy -= jm * ny / ma;
                    b.vx += jm * nx / mb;
                    b.vy += jm * ny / mb;
                }
                let (pa, pb) = (mb / (ma + mb), ma / (ma + mb));
                a.x -= nx * gap * pa;
                a.y -= ny * gap * pa;
                b.x += nx * gap * pb;
                b.y += ny * gap * pb;
                if a.crouch.is_some() && ny < 0.0 {
                    a.crouch = None;
                }
                if b.crouch.is_some() && ny > 0.0 {
                    b.crouch = None;
                }
            }
        }
    }

    fn advance(&mut self, dt: f64) {
        for _ in 0..(dt / H).round() as usize {
            self.step();
        }
    }
}

/// Bouncing balls; reads `balls` (1 to 5).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let n = options.f64("balls", 3.0).round().clamp(1.0, SIZES.len() as f64) as usize;
    let bs = (0..n)
        .map(|i| {
            let (r, e) = (SIZES[i], BOUNCE[i]);
            let w = STIFF / r;
            let z = -e.ln() / (std::f64::consts::PI.powi(2) + e.ln().powi(2)).sqrt();
            let x = W * (0.2 + 0.6 * i as f64 / (n.max(2) - 1) as f64);
            // They start resting on the floor and take turns to leap, so they never move in step.
            Ball { r, k: w * w, c: 2.0 * z * w, x, y: r, vx: 0.0, vy: 0.0, rest: 0.0, crouch: Some(-SPREAD * i as f64), sq: 0.0 }
        })
        .collect();
    let mut b = BouncingBalls { bs, rand: Mulberry32::new(11), last: 0.0, dots: ColorOctants::new(META.cols, META.rows) };
    for _ in 0..(WARM * 30.0) as usize {
        b.advance(1.0 / 30.0);
    }
    Box::new(b)
}

impl Piece for BouncingBalls {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        self.advance((t - self.last).clamp(0.0, 0.1));
        self.last = t;
        let w = self.dots.width() as i64;
        // Four dots to a row; the floor line is the top of the floor row.
        let floor = (FLOOR * 4.0) as i64;
        self.dots.clear();
        for x in 0..w {
            self.dots.set(x, floor, FLOOR_INK);
        }
        // The light from the upper left, and halfway between it and the eye.
        let l = crate::geom::unit([-0.45, 0.55, 0.7]);
        let half = crate::geom::unit([l[0], l[1], l[2] + 1.0]);
        let mut order: Vec<usize> = (0..self.bs.len()).collect();
        // Smaller balls are nearer, so they draw last.
        order.sort_by(|&a, &b| self.bs[b].r.total_cmp(&self.bs[a].r));
        for &i in &order {
            let b = &self.bs[i];
            // The shadow narrows and pales as the ball rises.
            let h = (b.y - b.r).max(0.0) / (FLOOR - 2.0 * b.r);
            let rx = b.r * b.r / (b.r - b.sq);
            let sw = rx * 4.0 * (1.0 - 0.6 * h);
            let cx = b.x * 4.0;
            for x in (cx - sw).floor() as i64..(cx + sw).ceil() as i64 {
                let edge = ((x as f64 + 0.5 - cx) / sw).abs();
                for dy in 1..4 {
                    if x >= 0 && dither(x as usize, (floor + dy) as usize, (1.0 - edge) * (1.0 - h) * 0.8 / dy as f64) {
                        self.dots.set(x, floor + dy, SHADOW);
                    }
                }
            }
        }
        for &i in &order {
            let b = &self.bs[i];
            // A squashed ball keeps its area: shorter by what it has sunk, wider to match.
            let ry = b.r - b.sq;
            let rx = b.r * b.r / ry;
            let (cx, cy) = (b.x * 4.0, (FLOOR - ry - (b.y - b.r).max(0.0)) * 4.0);
            let ramp = Ramp::new(i as u8 * 4, 4);
            for y in (cy - ry * 4.0).floor() as i64..(cy + ry * 4.0).ceil() as i64 {
                for x in (cx - rx * 4.0).floor() as i64..(cx + rx * 4.0).ceil() as i64 {
                    if x < 0 || y < 0 || y >= floor || x >= w {
                        continue;
                    }
                    let (u, v) = ((x as f64 + 0.5 - cx) / (rx * 4.0), (y as f64 + 0.5 - cy) / (ry * 4.0));
                    let q = u * u + v * v;
                    if q >= 1.0 {
                        continue;
                    }
                    let n = [u, -v, (1.0 - q).sqrt()];
                    let d = (n[0] * l[0] + n[1] * l[1] + n[2] * l[2]).max(0.0);
                    let s = (n[0] * half[0] + n[1] * half[1] + n[2] * half[2]).max(0.0).powi(40);
                    // The floor lights the underside a little.
                    let lit = (0.14 + 0.66 * d + 0.5 * s + 0.12 * v.max(0.0)).min(1.0);
                    let ink = if env.paper { 1.05 - lit } else { 0.3 + 0.75 * lit };
                    if dither(x as usize, y as usize, ink) {
                        self.dots.set(x, y, ramp.at(lit));
                    } else {
                        self.dots.unset(x, y);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
