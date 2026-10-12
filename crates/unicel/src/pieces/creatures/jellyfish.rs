//! jellyfish: a moon jelly swimming up through dark water. The bell squeezes
//! and relaxes, the tentacles follow the rim a beat late, and specks drift
//! by. After ascii.rest's jellyfish.ts (MIT, @bas3line).
//!
//! The stroke, the bell's shape and the trailing tentacles are ascii.rest's.
//! The bell is drawn in octants, dithered by how much jelly the eye looks
//! through, glassy in the middle and thick at its edges; the tentacles and
//! the frilled mouth arms below it are braille, threads a dot wide that fade
//! out toward their tips. In colour it glows as a moon jelly does: a
//! translucent blue bell deepening to violet at its edge, four pink
//! horseshoes showing through its top, pink frills and pale violet threads.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The jellyfish's meta.
pub static META: Meta = Meta {
    name: "jellyfish",
    slug: "jellyfish",
    category: Category::Creatures,
    note: "a glowing octant jelly pulsing upward, braille tentacles trailing",
    cols: 44,
    rows: 26,
    fps: 24,
    charset: Charset::Extended,
    // The bell, glassy to thick; its horseshoes; the rim; the mouth arms;
    // the threads, and their tips; specks.
    palette: Some(&["#5a9ad0", "#8ab8f0", "#c8a4ff", "#ff7ad4", "#e8b8ff", "#ff9ad8", "#b494e8", "#5a4a8a", "#3a4a64"]),
    ground: Some("#04060e"),
    ..Meta::DEFAULT
};

const BELL: [u8; 3] = [0, 1, 2];
const HORSESHOE: u8 = 3;
const RIM: u8 = 4;
const ARM: u8 = 5;
const THREAD: u8 = 6;
const TIP: u8 = 7;
const SPECK: u8 = 8;
/// Seconds per stroke, and the share of it spent squeezing.
const P: f64 = 2.2;
const SQ: f64 = 0.28;

fn ease(x: f64) -> f64 {
    x * x * (3.0 - 2.0 * x)
}

fn phase(t: f64) -> f64 {
    t.rem_euclid(P) / P
}

/// 0 relaxed, 1 fully squeezed.
fn squeeze(t: f64) -> f64 {
    let p = phase(t);
    if p < SQ {
        ease(p / SQ)
    } else {
        1.0 - ease((p - SQ) / (1.0 - SQ))
    }
}

/// How far the stroke has carried the body up: it lurches with the squeeze and sinks back.
fn lift(t: f64) -> f64 {
    let p = phase(t);
    if p < 0.4 {
        ease(p / 0.4)
    } else {
        1.0 - ease((p - 0.4) / 0.6)
    }
}

/// Where the rim sits (column, row) and how wide it is, given how squeezed the bell is.
fn rim_at(t: f64) -> (f64, f64, f64) {
    let c = squeeze(t);
    let x = META.cols as f64 / 2.0 + 1.5 * (TAU * t / (5.0 * P)).sin();
    (x, 11.0 - 0.8 * c - 1.4 * lift(t), (13.0 - 2.6 * c) * (1.0 - 0.3 * c))
}

/// How far the water has moved past: faster while the bell pushes.
fn travel(t: f64) -> f64 {
    let (k, p) = ((t / P).floor(), phase(t));
    0.9 * t + 2.4 * (k + if p < SQ { 0.8 * ease(p / SQ) } else { 0.8 + 0.2 * (p - SQ) / (1.0 - SQ) })
}

/// The jelly: specks (place, depth, phase), the braille below the rim and the octant bell.
pub struct Jellyfish {
    specks: Vec<(f64, f64, f64, f64)>,
    threads: ColorDots,
    bell: ColorOctants,
}

/// A jellyfish; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rnd = Mulberry32::new(7);
    let specks =
        (0..30).map(|_| (rnd.next_f64() * META.cols as f64, rnd.next_f64() * META.rows as f64, 0.4 + rnd.next_f64() * 0.6, rnd.next_f64() * 6.3)).collect();
    Box::new(Jellyfish { specks, threads: ColorDots::new(META.cols, META.rows), bell: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Jellyfish {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let rows = META.rows as f64;
        let (cx, ry, _) = rim_at(t);
        let c = squeeze(t);
        let (a, b) = (13.0 - 2.6 * c, 13.0 + 2.0 * c);
        self.threads.clear();
        let d = travel(t);
        for &(sx, sy, z, w) in &self.specks {
            let y = (sy + d * z).rem_euclid(rows);
            let x = sx + 0.6 * (t * 0.5 + w).sin();
            self.threads.set((x * 2.0) as i64, (y * 4.0) as i64, SPECK);
        }
        // Each point down a tentacle hangs from where the rim was a moment ago.
        let trail = |dd: f64, lag: f64| {
            let (wx, wy, half) = rim_at(t - dd * lag);
            (wx, ry + dd + (wy - ry).max(0.0) * 0.8, half)
        };
        // Eight threads, the outer ones shorter, so the tips stagger, each
        // fading out toward its tip.
        for i in 0..8 {
            let u = -0.92 + 1.84 * f64::from(i) / 7.0;
            let len = 8.5 + 3.2 * (1.0 - u.abs()) + f64::from((i * 3) % 5) * 0.8;
            let mut dd = 0.5;
            while dd < len {
                let (wx, wy, half) = trail(dd, 0.06);
                let wob = (dd * 0.5 - t * 3.0 + f64::from(i) * 1.9).sin();
                let x = wx + u * half * (1.0 - 0.01 * dd) + (0.05 + dd * 0.013) * dd * wob;
                let f = dd / len;
                let (px, py) = ((x * 2.0) as i64, (wy * 4.0) as i64);
                if dither(px.max(0) as usize, py.max(0) as usize, 1.25 - f) {
                    self.threads.set(px, py, if f > 0.75 { TIP } else { THREAD });
                }
                dd += 0.12;
            }
        }
        // Two frilled mouth arms, each a ribbon that widens where it turns face on.
        for i in 0..2 {
            let s = if i == 1 { 1.0 } else { -1.0 };
            let mut dd = 0.5;
            while dd < 10.0 {
                let (wx, wy, half) = trail(dd, 0.09);
                let x = wx + s * 2.0 * (1.0 - dd * 0.06) * (half / 10.0) + 0.9 * (dd * 0.6 - t * 2.2 + f64::from(i) * 2.5).sin();
                let tw = (dd * 1.1 + f64::from(i) * 1.3 - t * 1.6).cos();
                let wide = if dd / 10.0 > 0.8 {
                    0
                } else if tw.abs() < 0.35 {
                    1
                } else {
                    2 + i64::from(tw > 0.5)
                };
                for k in 0..wide.max(1) {
                    self.threads.set((x * 2.0) as i64 + k * s as i64, (wy * 4.0) as i64, ARM);
                }
                dd += 0.12;
            }
        }
        // The bell: a half dome, glassy in the middle and thick at the edges.
        // Ink stands for how much jelly the eye looks through, so it reads the
        // same on paper.
        self.bell.clear();
        let width = |s: f64| a * (1.0 - s * s).sqrt() * (1.0 - 0.3 * c * (1.0 - s).powi(2));
        let (w, _) = (self.bell.width(), self.bell.height());
        let rim_row = (ry * 4.0).floor() as i64;
        for j in 0..=rim_row {
            let y = (j as f64 + 0.5) / 4.0;
            let s = (ry - y) * 2.0 / b;
            if !(0.0..1.0).contains(&s) {
                continue;
            }
            for i in 0..w as i64 {
                let x = (i as f64 + 0.5) / 2.0 - cx;
                let wd = width(s);
                if x.abs() >= wd {
                    continue;
                }
                let u = (x / wd).clamp(-0.999, 0.999);
                let nz = (1.0 - u * u).sqrt() * (1.0 - s * s).sqrt();
                // Four pale horseshoes show through the top, two of them side on.
                let ph = u.asin();
                let g = (ph - 0.6).abs().min((ph + 0.6).abs());
                let ring = (1.0 - ((g / 0.36).hypot((s - 0.45) / 0.26) - 1.0).abs() * 3.5).max(0.0);
                // The crown is the thinnest, glassiest part, so it fades out up there.
                let crown = 1.0 - 0.6 * ease(((s - 0.62) / 0.38).max(0.0));
                let thick = (0.16 + 0.07 * s + 0.6 * (1.0 - nz).powi(2)) * crown;
                // They show through the jelly, so they thicken it only so far.
                let v = (thick + 0.3 * ring).min(0.5);
                let edge = 1.0 - nz < 0.08 && s < 0.97;
                if edge || dither(i as usize, j as usize, 0.1 + v) {
                    let color = if ring > 0.4 { HORSESHOE } else { BELL[((thick * 3.2) as usize).min(2)] };
                    self.bell.set(i, j, color);
                }
            }
        }
        // The rim, a frilled edge.
        for i in 0..w as i64 {
            let x = (i as f64 + 0.5) / 2.0 - cx;
            if x.abs() < width(0.0) {
                self.bell.set(i, rim_row + i64::from((i / 2) % 2 == 0), RIM);
            }
        }
        grid.clear();
        self.threads.draw(grid, 0, 0);
        self.bell.draw(grid, 0, 0);
    }
}
