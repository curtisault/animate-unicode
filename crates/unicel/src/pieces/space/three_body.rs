//! three-body: three equal masses chasing each other round one figure eight,
//! the periodic orbit of the three-body problem found by Moore and Simó.
//! After ascii.rest's three-body.ts (MIT, @bas3line).
//!
//! The orbit is integrated as ascii.rest does it and drawn as a faint dotted
//! line of braille. Each body is a ● in its own colour, red, green or blue,
//! with a braille trail along the eight behind it that thins dot by dot as it
//! ages, so the three read apart where they cross at the middle.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};

/// Three-body's meta.
pub static META: Meta = Meta {
    name: "three-body",
    slug: "three-body",
    category: Category::Space,
    note: "three coloured bodies on one figure eight, braille trails fading",
    cols: 65,
    rows: 15,
    fps: 30,
    charset: Charset::Extended,
    // The bodies; then the orbit.
    palette: Some(&["#ff6a5a", "#66f08e", "#6aa8ff", "#3a3e52"]),
    ground: Some("#05060b"),
    ..Meta::DEFAULT
};

const ORBIT: u8 = 3;
/// One lap in the orbit's own units of time, the steps it is integrated in,
/// the points kept along the path (evenly spaced in time), seconds per lap on
/// screen, and the points of tail behind each body.
const PERIOD: f64 = 6.325_913_98;
const STEPS: usize = 7200;
const N: usize = 720;
const LAP: f64 = 11.0;
const TRAIL: usize = 90;

/// Leapfrog over one lap, G = 1 and unit masses, keeping the first body's path.
fn integrate() -> Vec<(f64, f64)> {
    let mut p = [0.970_004_36, -0.243_087_53, -0.970_004_36, 0.243_087_53, 0.0, 0.0];
    let mut v = [0.466_203_68, 0.432_365_73, 0.466_203_68, 0.432_365_73, -0.932_407_37, -0.864_731_46];
    let acc = |p: &[f64; 6]| {
        let mut a = [0.0; 6];
        for i in (0..6).step_by(2) {
            for j in (0..6).step_by(2) {
                if i != j {
                    let (dx, dy) = (p[j] - p[i], p[j + 1] - p[i + 1]);
                    let r2 = dx * dx + dy * dy;
                    let r3 = r2 * r2.sqrt();
                    a[i] += dx / r3;
                    a[i + 1] += dy / r3;
                }
            }
        }
        a
    };
    let dt = PERIOD / STEPS as f64;
    let mut path = Vec::with_capacity(N);
    let mut a = acc(&p);
    for s in 0..STEPS {
        if s % (STEPS / N) == 0 {
            path.push((p[0], p[1]));
        }
        for k in 0..6 {
            v[k] += a[k] * dt / 2.0;
            p[k] += v[k] * dt;
        }
        a = acc(&p);
        for k in 0..6 {
            v[k] += a[k] * dt / 2.0;
        }
    }
    path
}

/// The eight: each path point in dots, the orbit's dots, and the dots.
pub struct ThreeBody {
    at: Vec<(i64, i64)>,
    orbit: Vec<(i64, i64)>,
    dots: ColorDots,
}

/// Three bodies; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as f64, META.rows as f64);
    let path = integrate();
    let x_max = path.iter().fold(0.0f64, |m, p| m.max(p.0.abs()));
    let y_max = path.iter().fold(0.0f64, |m, p| m.max(p.1.abs()));
    // The eight spans the width less a margin; a cell is twice as tall as wide.
    let sx = (cols - 7.0) / (2.0 * x_max);
    let sy = (y_max * sx * 0.5).round() / y_max;
    let at: Vec<(i64, i64)> = path
        .iter()
        .map(|&(x, y)| ((((cols - 1.0) / 2.0 + x * sx + 0.5) * 2.0).floor() as i64, (((rows - 1.0) / 2.0 - y * sy + 0.5) * 4.0).floor() as i64))
        .collect();
    // The orbit is a sparse dotted line, a dot about every three.
    let mut orbit: Vec<(i64, i64)> = vec![];
    for &d in &at {
        if orbit.last().is_none_or(|l| (l.0 - d.0).abs().max((l.1 - d.1).abs()) >= 3) {
            orbit.push(d);
        }
    }
    Box::new(ThreeBody { at, orbit, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for ThreeBody {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let head = ((t / LAP).rem_euclid(1.0) * N as f64) as usize;
        let at = |i: usize| self.at[i % N];
        self.dots.clear();
        for &(x, y) in &self.orbit {
            self.dots.set(x, y, ORBIT);
        }
        let heads: Vec<usize> = (0..3).map(|b| head + b * N / 3).collect();
        for (b, &h) in heads.iter().enumerate() {
            // Older points first, so the fresh trail is drawn over the old.
            for i in (1..=TRAIL).rev() {
                let (x, y) = at(h + N - i);
                let age = i as f64 / TRAIL as f64;
                if dither(x as usize, y as usize, 1.05 - age) {
                    self.dots.set(x, y, b as u8);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        for (b, &h) in heads.iter().enumerate() {
            let (x, y) = at(h);
            grid.set(x / 2, y / 4, '●', b as u8);
        }
    }
}
