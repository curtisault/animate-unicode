//! smoke: a thread of smoke from a stick of incense. It leaves the ember as
//! a straight line, starts to waver, frays into curls and thins to nothing.
//! After ascii.rest's smoke.ts (MIT, @bas3line).
//!
//! The puffs and the air that carries them are ascii.rest's, from its seeded
//! generator. In braille the thread is a line a dot wide joining the puffs
//! in order, so its waver and its curls are smooth, and the frayed haze is
//! dithered by its density, pale grey thinning to nothing. The ember glows
//! and flickers through orange and red.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// Smoke's meta.
pub static META: Meta = Meta {
    name: "smoke",
    slug: "smoke",
    category: Category::Physics,
    note: "incense smoke in braille, a thread fraying into grey haze",
    cols: 44,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // Haze, thin to thick; the thread; the ember, dull to bright; the stick and bowl.
    palette: Some(&["#3c4048", "#5c626c", "#8a909a", "#c4c8d0", "#e8ecf0", "#8a2a10", "#e0601a", "#ffc04a", "#6a5a48"]),
    ground: Some("#0a0a0c"),
    ..Meta::DEFAULT
};

/// The ember's row; puffs a second leaving it; seconds a puff lasts; rows a second near the ember.
const TIP: f64 = 17.0;
const RATE: f64 = 70.0;
const LIFE: f64 = 8.0;
const RISE: f64 = 3.4;
/// Ages over which the thread gives way to haze.
const FRAY: (f64, f64) = (2.3, 3.6);
/// Loose eddies higher up, as waves of a stream function: wavelength in rows, heading, drift, strength.
const WAVES: [(f64, f64, f64, f64); 4] = [(8.0, 0.4, 0.5, 1.0), (5.5, 2.3, -0.8, 0.9), (3.6, 1.1, 1.1, 0.6), (11.0, -1.0, 0.3, 0.8)];
const HAZE: Ramp = Ramp::new(0, 4);
const THREAD: u8 = 4;
const EMBER: Ramp = Ramp::new(5, 3);
const WOOD: u8 = 8;

fn smooth(a: f64, b: f64, x: f64) -> f64 {
    let u = ((x - a) / (b - a)).clamp(0.0, 1.0);
    u * u * (3.0 - 2.0 * u)
}

/// A puff: where it is, in rows from the ember, its age, and the noise that
/// decides where the thread breaks.
struct Puff {
    x: f64,
    y: f64,
    age: f64,
    cut: f64,
}

/// The smoke: its eddies, puffs, generator, clock, haze, and the dots.
pub struct Smoke {
    waves: Vec<(f64, f64, f64, f64, f64)>,
    puffs: Vec<Puff>,
    rand: Mulberry32,
    clock: f64,
    made: f64,
    last: f64,
    haze: Vec<f32>,
    dots: ColorDots,
}

impl Smoke {
    /// The air at (x, y), in rows from the ember. It rises and slows as it
    /// cools; a row of eddies riding up the plume makes the thread waver and
    /// roll, and looser eddies above break it up.
    fn flow(&self, x: f64, y: f64, t: f64) -> (f64, f64) {
        let k = 1.05;
        let s = k * (y - RISE * 0.8 * t) + 0.4 * (t * 0.37).sin();
        let g = (-(x / 2.2).powi(2)).exp();
        let lift = 1.7 * smooth(1.5, 7.0, y) * (1.0 - 0.5 * smooth(9.0, 15.0, y));
        let (mut u, mut v) = (lift * g * s.cos(), lift * g * s.sin() * 2.0 * x / (2.2 * 2.2 * k));
        let (e, yy) = (smooth(6.0, 12.0, y), y - RISE * 0.7 * t);
        for &(kx, ky, w, a, ph) in &self.waves {
            let c = (kx * x + ky * yy + w * t + ph).cos() * a * e;
            u += c * ky / 1.2;
            v -= c * kx / 1.2;
        }
        (u + 0.15 * (t * 0.55).sin() * smooth(0.5, 5.0, y), RISE * (1.0 - 0.35 * smooth(3.0, 15.0, y)) + 0.5 * v)
    }

    fn step(&mut self, dt: f64) {
        self.clock += dt;
        while self.made < self.clock * RATE {
            let age = self.clock - self.made / RATE;
            let cut = self.rand.next_f64() * 0.7 + 0.3 * (self.made * 0.09).sin();
            self.puffs.push(Puff { x: 0.0, y: age * RISE, age, cut });
            self.made += 1.0;
        }
        let t = self.clock;
        for i in 0..self.puffs.len() {
            let (u, v) = self.flow(self.puffs[i].x, self.puffs[i].y, t);
            let p = &mut self.puffs[i];
            p.x += u * dt;
            p.y += v * dt;
            p.age += dt;
        }
        let gone = self.puffs.iter().take_while(|p| p.age > LIFE || p.y > TIP + 3.0).count();
        self.puffs.drain(..gone);
    }
}

/// Smoke; it takes no options.
#[allow(clippy::approx_constant)] // ascii.rest's 6.283, so the eddies start where its do
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rand = Mulberry32::new(11);
    let waves = WAVES
        .iter()
        .map(|&(len, ang, drift, amp)| {
            let k = TAU / len;
            (k * ang.cos(), k * ang.sin(), drift, amp * 1.6, rand.next_f64() * 6.283)
        })
        .collect();
    let dots = ColorDots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    let mut s = Smoke { waves, puffs: vec![], rand, clock: 0.0, made: 0.0, last: 0.0, haze: vec![0.0; n], dots };
    for _ in 0..270 {
        s.step(1.0 / 30.0); // nine seconds, so the column is full
    }
    Box::new(s)
}

impl Piece for Smoke {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let mut dt = (time - self.last).clamp(0.0, 0.1);
        self.last = time;
        while dt > 1e-9 {
            self.step(dt.min(1.0 / 30.0));
            dt -= 1.0 / 30.0;
        }
        let (w, h) = (self.dots.width(), self.dots.height());
        let cx = (META.cols / 2) as f64 + 0.5;
        // A puff's place in dots: two to a column, four to a row.
        let at = |p: &Puff| ((cx + 2.0 * p.x) * 2.0, (TIP + 0.5 - p.y) * 4.0);
        self.haze.fill(0.0);
        self.dots.clear();
        // An old puff has let go of the thread and spreads as haze, thinning as it goes.
        for p in &self.puffs {
            let loose = smooth(FRAY.0 - 0.4, FRAY.1, p.age);
            if loose <= 0.0 {
                continue;
            }
            let rad = 0.55 + 0.2 * (p.age - FRAY.0 + 0.4); // rows
            let m = loose * (-(p.age - FRAY.0) / 1.7).exp() / (rad * rad);
            let (px, py) = at(p);
            let r = rad * 4.0;
            for y in ((py - r).floor() as i64).max(0)..=((py + r).ceil() as i64).min(h as i64 - 1) {
                for x in ((px - r).floor() as i64).max(0)..=((px + r).ceil() as i64).min(w as i64 - 1) {
                    let d2 = ((x as f64 + 0.5 - px).powi(2) + (y as f64 + 0.5 - py).powi(2)) / (r * r);
                    if d2 < 1.0 {
                        self.haze[y as usize * w + x as usize] += (m * (1.0 - d2) * (1.0 - d2)) as f32;
                    }
                }
            }
        }
        for (k, &v) in self.haze.iter().enumerate() {
            let v = f64::from(v) * 0.5;
            if v < 0.05 {
                continue;
            }
            let (x, y) = (k % w, k / w);
            // Smoke is ink on either ground, so paper shades it the same.
            if dither(x, y, (v.sqrt() * 0.6).min(0.5)) {
                self.dots.set(x as i64, y as i64, HAZE.at(v.sqrt()));
            }
        }
        // The thread: puff to puff in the order they left the ember, broken
        // where the fraying lets go.
        for pair in self.puffs.windows(2) {
            let hold = 1.0 - smooth(FRAY.0, FRAY.1, pair[0].age);
            if hold < pair[0].cut * 0.9 {
                continue;
            }
            let ((x0, y0), (x1, y1)) = (at(&pair[0]), at(&pair[1]));
            self.dots.line(x0.floor() as i64, y0.floor() as i64, x1.floor() as i64, y1.floor() as i64, THREAD);
        }
        // The stick and its smouldering ember.
        let stick = (cx * 2.0).floor() as i64;
        let glow = 0.55 + 0.25 * (time * 1.9).sin() + 0.15 * (time * 4.7 + 1.0).sin() + 0.12 * (time * 11.3 + 2.0).sin();
        let tip = (TIP * 4.0) as i64 + 2;
        for y in tip + 2..(h as i64 - 8) {
            self.dots.set(stick, y, WOOD);
        }
        for (dx, dy) in [(0, 0), (-1, 0), (0, 1), (-1, 1)] {
            self.dots.set(stick + dx, tip + dy, EMBER.at(glow));
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // A bowl of ash.
        let sc = cx.floor() as i64;
        for (k, s) in ["._______┃_______.", "\\_______________/"].iter().enumerate() {
            for (i, ch) in s.chars().enumerate() {
                grid.set(sc - 8 + i as i64, META.rows as i64 - 2 + k as i64, ch, WOOD);
            }
        }
    }
}
