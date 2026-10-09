//! fountain: a tall jet and two pairs of arcs leave a nozzle, break into
//! drops on the way down and splash into a brim-full basin, rippling its
//! surface. After ascii.rest's fountain.ts (MIT, @bas3line).
//!
//! The drops are ascii.rest's, from the same hash, but each is a braille dot,
//! eight to a character: the jets rise as unbroken threads of water and fall
//! as spray, every drop coloured by its speed, white where it is quick and
//! deep blue where it hangs at the top. The surface ripples a dot at a time
//! where the drops land.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::PI;

/// The fountain's meta.
pub static META: Meta = Meta {
    name: "fountain",
    slug: "fountain",
    category: Category::Physics,
    note: "a fountain's jets as braille drops, blue to white by their speed",
    cols: 57,
    rows: 22,
    fps: 30,
    charset: Charset::Extended,
    // Water by speed, slow first; the stone.
    palette: Some(&["#1c4a8a", "#2a6fbf", "#4a9ade", "#86c4f0", "#c8e8fa", "#ffffff", "#9a968e"]),
    ground: Some("#080b12"),
    ..Meta::DEFAULT
};

/// Gravity, rows a second squared; drops leaving each jet a second; the
/// water line's row; the nozzle's mouth above the water, in rows; seconds in,
/// so every arc is already full; how far the upright jet climbs.
const G: f64 = 30.0;
const RATE: f64 = 80.0;
const WATER: usize = 17;
const NOZ: f64 = 6.5;
const T0: f64 = 8.0;
const TOP: f64 = 8.4;
/// Each jet: angle from upright in degrees, where its middle meets the water
/// in rows from the post, how much its drops wander in angle and speed, and
/// one drop in how many stays in sight once it breaks up on the way down.
const JETS: [(f64, f64, f64, f64, usize); 5] =
    [(0.0, 0.0, 0.004, 0.035, 3), (-17.0, -7.0, 0.006, 0.008, 4), (17.0, 7.0, 0.006, 0.008, 4), (-42.0, -10.4, 0.005, 0.006, 5), (42.0, 10.4, 0.005, 0.006, 5)];
/// "#" is the post's hollow middle: blank, and drops pass behind it.
const POST: [&str; 7] = ["_/^\\_", " |#| ", " |#| ", " |#| ", " |#| ", " |#| ", "_/#\\_"];
/// Fast enough to be white.
const FAST: f64 = 22.0;
const WET: Ramp = Ramp::new(0, 6);
const STONE: u8 = 6;

/// ascii.rest's hash of (drop, salt) into [0, 1).
fn hash(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b) ^ ((s + 1) as u32).wrapping_mul(0xc2b2_ae35);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    f64::from(h ^ (h >> 15)) / 4_294_967_296.0
}

/// Seconds from leaving the nozzle with upward speed vy until meeting the water.
fn flight(vy: f64) -> f64 {
    (vy + (vy * vy + 2.0 * G * NOZ).sqrt()) / G
}

/// A jet: its angle, launch speed, wander, and which drops stay in sight.
#[derive(Clone, Copy)]
struct Jet {
    a: f64,
    v: f64,
    wa: f64,
    wv: f64,
    keep: usize,
    j: usize,
}

/// One drop: when it left, its velocity, and how long it flies.
struct Drop {
    t0: f64,
    vx: f64,
    vy: f64,
    end: f64,
}

impl Jet {
    /// Drop i. The upright jet's pressure swells and eases every five seconds.
    fn drop(&self, i: i64) -> Drop {
        let s = self.j as i64 * 7 + 1;
        let pulse = if self.j > 0 { 1.0 } else { 0.95 + 0.05 * (i as f64 / RATE * 1.2566).sin() };
        let a = self.a + (hash(i, s) - 0.5) * 2.0 * self.wa * PI;
        let v = self.v * pulse * (1.0 + (hash(i, s + 1) - 0.5) * 2.0 * self.wv);
        let (vx, vy) = (v * a.sin(), v * a.cos());
        Drop { t0: (i as f64 + hash(i, s + 2) * 0.6) / RATE, vx, vy, end: flight(vy) }
    }
}

/// The fountain: its jets, and the dots.
pub struct Fountain {
    jets: Vec<Jet>,
    dots: ColorDots,
}

/// A fountain; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // The launch speed that lands each arc where it should, by bisection.
    let mut jets: Vec<Jet> = JETS
        .iter()
        .enumerate()
        .map(|(j, &(deg, land, wa, wv, keep))| {
            let a = deg * PI / 180.0;
            let mut v = (2.0 * G * TOP).sqrt();
            if deg != 0.0 {
                let (mut lo, mut hi) = (1.0, 60.0);
                for _ in 0..40 {
                    v = (lo + hi) / 2.0;
                    if (v * a.sin() * flight(v * a.cos())).abs() < land.abs() {
                        lo = v;
                    } else {
                        hi = v;
                    }
                }
            }
            Jet { a, v, wa, wv, keep, j }
        })
        .collect();
    jets.rotate_left(1); // the upright jet is drawn last, so it runs whole down to the nozzle
    Box::new(Fountain { jets, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Fountain {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let t = T0 + time;
        let cx = (META.cols / 2) as f64 + 0.5; // the middle of the nozzle's cell, in columns
                                               // Columns are two dots, rows four; a drop at (x rows across, row down).
        let at = |x: f64, row: f64| (((cx + 2.0 * x) * 2.0).floor() as i64, (row * 4.0).floor() as i64);
        let water = (WATER * 4 + 2) as i64;
        let mut rings: Vec<(f64, f64)> = vec![];
        let mut hits: Vec<f64> = vec![];
        let mut crowns: Vec<(f64, f64, i64, usize)> = vec![];
        self.dots.clear();
        for jet in &self.jets {
            for i in ((t - 3.2) * RATE).floor() as i64..=(t * RATE).ceil() as i64 {
                let d = jet.drop(i);
                let age = t - d.t0;
                if age < 0.0 {
                    continue;
                }
                if age >= d.end {
                    let (k, x) = (age - d.end, d.vx * d.end);
                    if k < 1.4 && hash(i, jet.j as i64 + 60) < 0.018 {
                        rings.push((x, k));
                    }
                    if k < 0.07 && i.rem_euclid(jet.keep as i64) == 0 {
                        hits.push(x);
                    }
                    if k < 0.4 && hash(i, jet.j as i64 + 70) < 0.03 {
                        crowns.push((x, k, i, jet.j));
                    }
                    continue;
                }
                let vy = d.vy - G * age;
                // The stream rises whole and breaks into drops past the top, a few in sight.
                if vy < 0.0 && i.rem_euclid(jet.keep as i64) != 0 {
                    continue;
                }
                let row = WATER as f64 - NOZ - d.vy * age + G * age * age / 2.0;
                if vy < 0.0 && jet.j == 0 && row > WATER as f64 - NOZ - 1.0 {
                    continue; // fallen back into the nozzle's throat
                }
                let (x, y) = at(d.vx * age, row);
                self.dots.set(x, y, WET.at((d.vx * d.vx + vy * vy).sqrt() / FAST));
            }
        }
        // The water line: calm, rising a dot where a wavelet runs out from a landing.
        let (lo, hi) = (((cx - 23.0) * 2.0) as i64, ((cx + 24.0) * 2.0) as i64);
        for x in lo..hi {
            let xr = (x as f64 / 2.0 + 0.25 - cx) / 2.0;
            let mut h = 0.0;
            for &(x0, k) in &rings {
                let d = ((xr - x0).abs() - 0.5 - 2.6 * k).abs();
                if d < 0.4 {
                    h += (1.0 - d * 2.5) * (1.0 - k / 1.4);
                }
            }
            self.dots.set(x, water - i64::from(h > 0.3), if h > 0.3 { 4 } else { 2 });
        }
        // Where a drop strikes, the surface jumps; a crown of two drops is
        // thrown up either side and falls back in.
        for &x in &hits {
            let (c, _) = at(x, 0.0);
            self.dots.set(c, water - 1, 5);
            self.dots.set(c, water - 2, 5);
        }
        for &(x, k, i, j) in &crowns {
            for side in [-1.0, 1.0] {
                let vx = side * (1.2 + hash(i, j as i64 + 40) * 1.4);
                let vy = 7.0 + hash(i, j as i64 + 50 + side as i64) * 4.0;
                let y = vy * k - G * k * k / 2.0;
                if y > 0.2 {
                    let (c, r) = at(x + vx * k, WATER as f64 + 0.5 - y);
                    self.dots.set(c, r, 5);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The basin and the post over the water and the drops.
        let basin = ["▀".repeat(49), format!("\\{}/", "_".repeat(45)), format!("|{}|", " ".repeat(13)), format!("___|{}|___", "_".repeat(13))];
        let post: Vec<String> = POST.iter().map(|s| s.to_string()).collect();
        let mid = (META.cols / 2) as i64;
        for (art, r0) in [(&basin[..], WATER + 1), (&post[..], WATER + 1 - POST.len())] {
            for (k, line) in art.iter().enumerate() {
                let c0 = mid - (line.chars().count() as i64 - 1) / 2;
                for (c, ch) in line.chars().enumerate() {
                    if ch != ' ' {
                        grid.set(c0 + c as i64, (r0 + k) as i64, if ch == '#' { ' ' } else { ch }, STONE);
                    }
                }
            }
        }
    }
}
