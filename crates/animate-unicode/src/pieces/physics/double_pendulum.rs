//! double pendulum: two equal rods hung end to end from one pivot, stepped
//! with RK4. Chaotic, so it never repeats; the lower bob leaves a fading
//! trail. After ascii.rest's double-pendulum.ts (MIT, @bas3line).
//!
//! The model is ascii.rest's, its integrator and the energy it holds. In
//! braille the rods are true lines over a 96×100 picture and the trail is a
//! curve a dot wide, twice as long, coloured by how fast the tip was going:
//! deep blue where it hung, through teal and gold to white where it whipped
//! round, thinning out as it ages.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// The double pendulum's meta.
pub static META: Meta = Meta {
    name: "double pendulum",
    slug: "double-pendulum",
    category: Category::Physics,
    note: "a chaotic double pendulum in braille, its trail coloured by speed",
    cols: 48,
    rows: 25,
    fps: 30,
    charset: Charset::Extended,
    // The trail by speed, slow first; the rods; the bobs and pivot.
    palette: Some(&["#1f3a7a", "#2a64b0", "#2fa0b8", "#5ac890", "#d8c850", "#f4924a", "#fff2d8", "#9aa4b4", "#f4f6fa"]),
    ground: Some("#0a0b10"),
    ..Meta::DEFAULT
};

const G: f64 = 9.81;
/// Integrator step and model seconds per second of play.
const H: f64 = 1.0 / 240.0;
const SPEED: f64 = 0.6;
/// Steps between trail samples, and samples kept.
const TAP: usize = 2;
const TRAIL: usize = 220;
/// Both angles from straight down, then their rates; and model seconds run before the first frame.
const START: [f64; 4] = [2.55, 2.9, 0.0, 0.0];
const WARM: f64 = 7.0;
/// Tip speed (in rod lengths a model second) at the top of the trail's colours.
const FAST: f64 = 9.0;
const TRAIL_INK: Ramp = Ramp::new(0, 7);
const ROD: u8 = 7;
const BOB: u8 = 8;

/// Unit masses and rods. s = [angle 1, angle 2, rate 1, rate 2].
fn deriv([a, b, p, q]: [f64; 4]) -> [f64; 4] {
    let d = a - b;
    let (sd, cd, den) = (d.sin(), d.cos(), 3.0 - (2.0 * d).cos());
    [
        p,
        q,
        (-3.0 * G * a.sin() - G * (a - 2.0 * b).sin() - 2.0 * sd * (q * q + p * p * cd)) / den,
        (2.0 * sd * (2.0 * p * p + 2.0 * G * a.cos() + q * q * cd)) / den,
    ]
}

fn rk4(s: [f64; 4]) -> [f64; 4] {
    let add = |u: [f64; 4], k: [f64; 4], h: f64| [u[0] + k[0] * h, u[1] + k[1] * h, u[2] + k[2] * h, u[3] + k[3] * h];
    let k1 = deriv(s);
    let k2 = deriv(add(s, k1, H / 2.0));
    let k3 = deriv(add(s, k2, H / 2.0));
    let k4 = deriv(add(s, k3, H));
    [0, 1, 2, 3].map(|i| s[i] + H / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
}

fn kinetic([a, b, p, q]: [f64; 4]) -> f64 {
    p * p + 0.5 * q * q + p * q * (a - b).cos()
}

fn potential([a, b, _, _]: [f64; 4]) -> f64 {
    -2.0 * G * a.cos() - G * b.cos()
}

/// The pendulum: its state, the trail (tip place and speed), the clock, and the dots.
pub struct DoublePendulum {
    s: [f64; 4],
    e0: f64,
    trail: Vec<(f64, f64, f64)>,
    steps: usize,
    acc: f64,
    last: f64,
    dots: ColorDots,
}

impl DoublePendulum {
    /// Rod length in dots, the pivot, and a point at (sin, cos) sums.
    fn layout(&self) -> (f64, f64, f64) {
        let (w, h) = (self.dots.width() as f64, self.dots.height() as f64);
        // Both rods reach any edge but none clips.
        ((h / 2.0 - 2.5) / 2.0, w / 2.0, h / 2.0)
    }

    fn tip(&self, s: [f64; 4]) -> (f64, f64) {
        let (l, cx, cy) = self.layout();
        (cx + l * (s[0].sin() + s[1].sin()), cy + l * (s[0].cos() + s[1].cos()))
    }

    fn step(&mut self) {
        self.s = rk4(self.s);
        // Hold the energy where it began, so long runs neither die down nor run away.
        let (k, room) = (kinetic(self.s), self.e0 - potential(self.s));
        if k > 1e-9 && room > 0.0 {
            let f = (room / k).sqrt();
            self.s[2] *= f;
            self.s[3] *= f;
        }
        self.steps += 1;
        if self.steps.is_multiple_of(TAP) {
            // The tip's speed, in rod lengths a model second.
            let [a, b, p, q] = self.s;
            let (vx, vy) = (p * a.cos() + q * b.cos(), -p * a.sin() - q * b.sin());
            let (x, y) = self.tip(self.s);
            self.trail.push((x, y, (vx * vx + vy * vy).sqrt()));
            if self.trail.len() > TRAIL {
                self.trail.remove(0);
            }
        }
    }

    fn reset(&mut self) {
        (self.s, self.steps, self.acc, self.last) = (START, 0, 0.0, 0.0);
        self.trail.clear();
        for _ in 0..(WARM / H) as usize {
            self.step();
        }
    }
}

/// A double pendulum; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut p = DoublePendulum {
        s: START,
        e0: kinetic(START) + potential(START),
        trail: vec![],
        steps: 0,
        acc: 0.0,
        last: 0.0,
        dots: ColorDots::new(META.cols, META.rows),
    };
    p.reset();
    Box::new(p)
}

impl Piece for DoublePendulum {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        if t < self.last {
            self.reset();
        }
        self.acc += (t - self.last).clamp(0.0, 0.1) * SPEED;
        self.last = t;
        while self.acc >= H {
            self.step();
            self.acc -= H;
        }
        self.dots.clear();
        // Oldest first, a segment at a time so a fast swing leaves no gaps;
        // the oldest thin out by dithering.
        let n = self.trail.len();
        for i in 1..n {
            let ((x0, y0, _), (x1, y1, v)) = (self.trail[i - 1], self.trail[i]);
            let age = 1.0 - i as f64 / n as f64;
            let c = TRAIL_INK.at(v / FAST);
            let steps = ((x1 - x0).abs().max((y1 - y0).abs()) * 1.5).ceil().max(1.0) as usize;
            for j in 0..steps {
                let f = j as f64 / steps as f64;
                let (x, y) = ((x0 + (x1 - x0) * f).floor(), (y0 + (y1 - y0) * f).floor());
                if x >= 0.0 && y >= 0.0 && dither(x as usize, y as usize, 1.15 - age) {
                    self.dots.set(x as i64, y as i64, c);
                }
            }
        }
        let (l, cx, cy) = self.layout();
        let (x1, y1) = (cx + l * self.s[0].sin(), cy + l * self.s[0].cos());
        let (x2, y2) = self.tip(self.s);
        let r = |v: f64| v.round() as i64;
        self.dots.line(r(cx), r(cy), r(x1), r(y1), ROD);
        self.dots.line(r(x1), r(y1), r(x2), r(y2), ROD);
        // The pivot, and the two bobs as small round knots, the lower one larger.
        for (x, y, big) in [(cx, cy, false), (x1, y1, false), (x2, y2, true)] {
            let rad: i64 = if big { 2 } else { 1 };
            for dy in -rad..=rad {
                for dx in -rad..=rad {
                    if dx * dx + dy * dy <= rad * rad + 1 {
                        self.dots.set(r(x) + dx, r(y) + dy, BOB);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
