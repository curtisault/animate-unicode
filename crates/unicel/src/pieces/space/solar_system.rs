//! solar system: planets seen from above on dotted orbits. Each period goes
//! as the orbit's size to the power 1.5, Kepler's third law. After
//! ascii.rest's solar-system.ts (MIT, @bas3line).
//!
//! The four orbits, their sizes and periods are ascii.rest's, but each is now
//! a true ellipse with the sun at a focus, drawn in dotted braille, and each
//! planet is placed by solving Kepler's equation, so it hurries round
//! perihelion and dawdles at aphelion, its trail stretching and shrinking to
//! match. The planets are ● in their own colours, a grey world, a blue one
//! with its moon, a red one and a ringed gold giant, over a sun that glows
//! white-gold to orange.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The solar system's meta.
pub static META: Meta = Meta {
    name: "solar system",
    slug: "solar-system",
    category: Category::Space,
    note: "four coloured planets on braille ellipses, each hurrying past the sun",
    cols: 61,
    rows: 29,
    fps: 20,
    charset: Charset::Extended,
    options: Some(r#"{"speed": 1}"#),
    // The sun, edge to core, and its glow; then each planet's colour and its
    // orbit's faint tint; the moon; the giant's ring.
    palette: Some(&[
        "#ff8a24", "#ffc23e", "#fff2c0", "#8a3c12", "#b8b2aa", "#4a4844", "#4a9cff", "#1e3a66", "#ff5a3c", "#5a2218", "#f0c060", "#5a4a24", "#d8dce4",
        "#c8a868",
    ]),
    ground: Some("#04050a"),
    ..Meta::DEFAULT
};

const SUN: Ramp = Ramp::new(0, 3);
const GLOW: u8 = 3;
const MOON: u8 = 12;
const RING: u8 = 13;

/// A planet: its orbit's half width in columns (rows count double), its
/// eccentricity, where perihelion points, its mean anomaly at t = 0, its
/// colour (the orbit's tint follows it), and whether it has a moon or a ring.
struct Body {
    a: f64,
    e: f64,
    peri: f64,
    m0: f64,
    color: u8,
    moon: bool,
    ring: bool,
}

/// Orbits open out with distance, as they do: ascii.rest's sizes, now eccentric.
const PLANETS: [Body; 4] = [
    Body { a: 9.6, e: 0.2, peri: 1.0, m0: 0.5, color: 4, moon: false, ring: false },
    Body { a: 14.5, e: 0.06, peri: 4.0, m0: 2.3, color: 6, moon: true, ring: false },
    Body { a: 21.0, e: 0.09, peri: 2.0, m0: 4.1, color: 8, moon: false, ring: false },
    Body { a: 28.0, e: 0.03, peri: 0.0, m0: 5.9, color: 10, moon: false, ring: true },
];
/// Seconds for the innermost orbit at speed 1, and how much of its orbit a
/// planet's trail covers.
const T1: f64 = 3.0;
const TRAIL: f64 = 0.07;

/// Where a body is, in columns and rows from the sun, at mean anomaly m.
fn place(b: &Body, m: f64) -> (f64, f64) {
    // Kepler's equation, m = E - e sin E, by Newton's method.
    let mut ea = m;
    for _ in 0..6 {
        ea -= (ea - b.e * ea.sin() - m) / (1.0 - b.e * ea.cos());
    }
    let (x, y) = (b.a * (ea.cos() - b.e), b.a * (1.0 - b.e * b.e).sqrt() * ea.sin());
    let (s, c) = b.peri.sin_cos();
    (x * c - y * s, (x * s + y * c) / 2.0)
}

/// The system: the speed, the orbits' dots and the sun's (with their colour), and the dots.
pub struct SolarSystem {
    speed: f64,
    fixed: Vec<(i64, i64, u8)>,
    sun: Vec<(i64, i64, f64)>,
    dots: ColorDots,
}

/// A solar system; reads `speed` (1 is the innermost orbit in three seconds).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cx, cy) = ((META.cols as f64 - 1.0) / 2.0 + 0.5, (META.rows as f64 - 1.0) / 2.0 + 0.5);
    // Each orbit a dot about every two dots of arc, in its planet's faint tint.
    let mut fixed = vec![];
    for b in &PLANETS {
        let n = (TAU * b.a).round() as usize;
        let mut last = (i64::MIN / 2, i64::MIN / 2);
        for k in 0..n * 4 {
            let (x, y) = place(b, k as f64 / (n * 4) as f64 * TAU);
            let d = (((cx + x) * 2.0).floor() as i64, ((cy + y) * 4.0).floor() as i64);
            if (d.0 - last.0).abs().max((d.1 - last.1).abs()) >= 2 {
                fixed.push((d.0, d.1, b.color + 1));
                last = d;
            }
        }
    }
    // The sun, a disc 4.4 columns across its radius, and a glow round it.
    let mut sun = vec![];
    for j in -12..=12 {
        for i in -12..=12 {
            let d = f64::from(i).hypot(f64::from(j)) / 8.8;
            if d < 1.4 {
                sun.push(((cx * 2.0) as i64 + i64::from(i), (cy * 4.0) as i64 + i64::from(j), d));
            }
        }
    }
    Box::new(SolarSystem { speed: options.f64("speed", 1.0), fixed, sun, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for SolarSystem {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cx, cy) = ((META.cols as f64 - 1.0) / 2.0 + 0.5, (META.rows as f64 - 1.0) / 2.0 + 0.5);
        self.dots.clear();
        for &(x, y, color) in &self.fixed {
            self.dots.set(x, y, color);
        }
        for &(x, y, d) in &self.sun {
            if d < 1.0 && dither(x as usize, y as usize, 1.0 - 0.45 * d * d) {
                self.dots.set(x, y, SUN.at(1.0 - d));
            } else if d >= 1.0 {
                // The glow breathes a little.
                let v = 0.3 * (1.4 - d) / 0.4 * (1.0 + 0.3 * (t * 1.3 + f64::from((x * 7 + y * 3) as i32 % 11)).sin());
                if v > 0.06 && dither(x as usize, y as usize, v) {
                    self.dots.set(x, y, GLOW);
                }
            }
        }
        let mut glyphs = vec![];
        for b in &PLANETS {
            let period = T1 * (b.a / PLANETS[0].a).powf(1.5) / self.speed.max(1e-6);
            let m = b.m0 + TAU * t / period;
            // The trail walks back along the orbit through the same span of
            // time, so it is long where the planet is fast and short where it is slow.
            let steps = 40;
            for k in 1..=steps {
                let f = k as f64 / steps as f64;
                let (x, y) = place(b, m - TAU * TRAIL * f);
                let (dx, dy) = ((cx + x) * 2.0, (cy + y) * 4.0);
                if dither(dx as usize, dy as usize, 1.0 - f) {
                    self.dots.set(dx.floor() as i64, dy.floor() as i64, b.color);
                }
            }
            let (x, y) = place(b, m);
            let (c, r) = ((cx + x).floor() as i64, (cy + y).floor() as i64);
            glyphs.push((c, r, '●', b.color));
            if b.ring {
                glyphs.push((c - 1, r, '─', RING));
                glyphs.push((c + 1, r, '─', RING));
            }
            if b.moon {
                let mo = TAU * t * self.speed / 1.1;
                glyphs.push(((cx + x + 2.4 * mo.cos()).floor() as i64, (cy + y - 1.2 * mo.sin()).floor() as i64, '°', MOON));
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for (c, r, ch, color) in glyphs {
            grid.set(c, r, ch, color);
        }
    }
}
