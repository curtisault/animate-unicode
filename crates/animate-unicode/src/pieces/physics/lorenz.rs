//! lorenz attractor: one trajectory of the Lorenz system. The near side of
//! each loop is drawn heavier than the far side, old loops fade out, and the
//! view sways. After ascii.rest's lorenz.ts (MIT, @bas3line).
//!
//! The trajectory and the view are ascii.rest's. In braille the path is a
//! curve a dot wide over a 120×88 picture, so the sheets of loops stay
//! apart, and each wing has its own colour, ice blue on one side of the
//! attractor and amber on the other, bright where the loop is new and near,
//! fading as it ages: the eye can see when the path changes wings.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The Lorenz attractor's meta.
pub static META: Meta = Meta {
    name: "lorenz attractor",
    slug: "lorenz",
    category: Category::Physics,
    note: "the lorenz attractor in braille, one wing blue, the other amber",
    cols: 60,
    rows: 22,
    fps: 30,
    charset: Charset::Extended,
    // One wing, faint to bright; the other; the tip.
    palette: Some(&["#1a2f5a", "#2a5296", "#4a86cc", "#9ccaf4", "#5a3410", "#96601c", "#d8962e", "#ffd486", "#ffffff"]),
    ground: Some("#08090e"),
    ..Meta::DEFAULT
};

/// Time per integration step, steps per second of play, and steps kept on screen.
const DT: f64 = 0.004;
const RATE: f64 = 125.0;
const TRAIL: usize = 2400;
/// Newest steps drawn as the bright tip.
const LEAD: usize = 26;
/// The view looks this far down onto the attractor; the turn that shows both
/// wings side by side; how far and how slowly it sways.
const TILT: f64 = 0.3;
const FACE: f64 = 2.36;
const SWAY: f64 = 0.6;
const SWAY_T: f64 = 48.0;
/// Past this share of the frame, rare wide loops are eased in.
const KNEE: f64 = 0.8;
/// How fast old loops fade, and the far side's weight against the near side's.
const AGE: f64 = 1.5;
const FAR: f64 = 0.3;
const TIP: u8 = 8;

/// The attractor: the trail, its head, the point, and the clock.
pub struct Lorenz {
    pts: Vec<[f64; 3]>,
    head: usize,
    p: [f64; 3],
    owed: f64,
    last: f64,
    dots: ColorDots,
}

fn f([x, y, z]: [f64; 3]) -> [f64; 3] {
    [10.0 * (y - x), x * (28.0 - z) - y, x * y - 8.0 / 3.0 * z]
}

impl Lorenz {
    /// One RK4 step of sigma = 10, rho = 28, beta = 8/3.
    fn step(&mut self) {
        let p = self.p;
        let add = |a: [f64; 3], k: [f64; 3], h: f64| [a[0] + h * k[0], a[1] + h * k[1], a[2] + h * k[2]];
        let k1 = f(p);
        let k2 = f(add(p, k1, DT / 2.0));
        let k3 = f(add(p, k2, DT / 2.0));
        let k4 = f(add(p, k3, DT));
        self.p = [0, 1, 2].map(|i| p[i] + DT / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]));
        self.pts[self.head] = self.p;
        self.head = (self.head + 1) % TRAIL;
    }
}

/// The Lorenz attractor; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut l = Lorenz { pts: vec![[0.0; 3]; TRAIL], head: 0, p: [1.0, 1.0, 1.0], owed: 0.0, last: 0.0, dots: ColorDots::new(META.cols, META.rows) };
    for _ in 0..24000 {
        l.step();
    }
    Box::new(l)
}

impl Piece for Lorenz {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        self.owed += (t - self.last).clamp(0.0, 0.1) * RATE;
        self.last = t;
        while self.owed >= 1.0 {
            self.step();
            self.owed -= 1.0;
        }
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        let turn = FACE + SWAY * (TAU * t / SWAY_T).sin();
        let (s, c) = turn.sin_cos();
        let (st, ct) = TILT.sin_cos();
        // A typical stretch of trail stands about 33 units tall, centred a
        // little above z = 25; the odd wide loop is squeezed into the last
        // dots rather than cut off.
        let (half_h, half_w) = (h / 2.0 - 6.0, w / 2.0 - 5.0);
        let k = 2.0 * half_h / 33.5;
        let ease = |a: f64| if a.abs() <= KNEE { a } else { a.signum() * (KNEE + (1.0 - KNEE) * ((a.abs() - KNEE) / (1.0 - KNEE)).tanh()) };
        // How deep the trail is from this side sets the depth scale.
        let sum: f64 = self.pts.iter().map(|p| (p[0] * s + p[1] * c).powi(2)).sum();
        let deep = 1.0 / (3.0 * (sum / TRAIL as f64).sqrt());
        self.dots.clear();
        let mut tip = (0, 0);
        let mut prev: Option<(i64, i64)> = None;
        for n in 0..TRAIL {
            let p = self.pts[(self.head + n) % TRAIL]; // oldest first
            let u = p[0] * c - p[1] * s;
            let v = p[0] * s + p[1] * c; // depth, positive is away
            let hh = (p[2] - 25.0) * ct - v * st;
            let x = (w / 2.0 + ease(u * k / half_w) * half_w).floor() as i64;
            let y = (h / 2.0 - ease((hh - 1.3) * k / half_h) * half_h).floor() as i64;
            // Recent and near is bright; old or far is faint, and the oldest is gone.
            let age = n as f64 / TRAIL as f64;
            let near = (0.5 - v * deep).clamp(0.0, 1.0);
            let weight = age.powf(AGE) * (FAR + (1.0 - FAR) * near * near);
            tip = (x, y);
            let from = prev.replace((x, y)).unwrap_or((x, y));
            let color = if n >= TRAIL - LEAD { TIP } else { Ramp::new(if p[0] < 0.0 { 0 } else { 4 }, 4).at(weight) };
            if weight < 0.04 && color != TIP {
                continue;
            }
            // Each step joined to the last, so the path is one unbroken curve.
            let ink = if env.paper { 0.6 + weight } else { 0.5 + 2.0 * weight };
            let dots = &mut self.dots;
            crate::grid::bresenham(from.0, from.1, x, y, |px, py| {
                if color == TIP || (px >= 0 && py >= 0 && dither(px as usize, py as usize, ink)) {
                    dots.set(px, py, color);
                }
            });
        }
        for (dx, dy) in [(1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
            self.dots.set(tip.0 + dx, tip.1 + dy, TIP);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
