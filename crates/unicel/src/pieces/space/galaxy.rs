//! galaxy: a two-armed spiral seen at a tilt. The arm pattern turns slowly as
//! a whole while its stars orbit faster near the core, as in a real disc.
//! After ascii.rest's galaxy.ts (MIT, @bas3line).
//!
//! The light is ascii.rest's, sampled once per braille dot, and each dot is a
//! star: it shows where the light at it beats a threshold fixed for that dot
//! at random, so the brighter the sky the denser the stars, with no pattern
//! to them. The core is yellow, the arms blue-white with a few pink knots
//! where stars are forming, the dust lane along each arm's inner edge dark
//! brown, and the stars that orbit on their own glint white. It glows on
//! empty sky, so on paper it prints as a negative.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The galaxy's meta.
pub static META: Meta = Meta {
    name: "galaxy",
    slug: "galaxy",
    category: Category::Space,
    note: "a spiral of braille stars turning: a yellow core, blue arms, dust",
    cols: 64,
    rows: 26,
    fps: 20,
    charset: Charset::Extended,
    // Dust; the disc's faint blue; the arms; a bright arm; the core's white
    // and yellow; a pink knot; an orbiting star; a background star.
    palette: Some(&["#6a4430", "#4a5a9e", "#7a9cf2", "#cad8ff", "#fff0d4", "#ffd06e", "#ff78a8", "#ffffff", "#7a7e98"]),
    ground: Some("#05050c"),
    ..Meta::DEFAULT
};

const DUST: u8 = 0;
const FAINT: u8 = 1;
const ARM: u8 = 2;
const BRIGHT: u8 = 3;
const WARM: u8 = 4;
const CORE: u8 = 5;
const KNOT: u8 = 6;
const STAR: u8 = 7;
const BACK: u8 = 8;
/// Disc radius in columns, the cosine of the tilt (how round the disc looks),
/// the long axis's turn off level, and the arms' turn in radians a second.
const R: f64 = 28.0;
const INC: f64 = 0.78;
const PA: f64 = -0.14;
const PATTERN: f64 = 0.1;

/// What never changes at a dot: the bulge and disc light, the arm's phase
/// there, how far the arms reach to it, how much the dust can take, the
/// dot's star threshold, and whether a pink knot can form there.
struct Spot {
    k: usize,
    base: f32,
    phase: f32,
    reach: f32,
    dust: f32,
    bulge: f32,
    threshold: f32,
    knot: bool,
}

/// The galaxy: every dot inside it, its orbiting stars (radius, angle, speed), the background stars, and the dots.
pub struct Galaxy {
    spots: Vec<Spot>,
    stars: Vec<(f64, f64, f64)>,
    back: Vec<usize>,
    dots: ColorDots,
}

/// A galaxy; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
    let (spa, cpa) = PA.sin_cos();
    let wind = 1.0 / 0.5f64.tan(); // log spiral pitch, open enough to read as two arms
    let mut rand = Mulberry32::new(11);
    // Stars that orbit on their own along the outer disc, on a flat rotation curve.
    let stars = (0..110).map(|_| (0.3 + 0.62 * rand.next_f64().sqrt(), rand.next_f64() * TAU, 0.2 + 0.2 * rand.next_f64())).collect();
    let dots = ColorDots::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let (mut spots, mut back) = (vec![], vec![]);
    for j in 0..h {
        for i in 0..w {
            // In columns from the centre, y down with rows doubled.
            let (x, y) = ((i as f64 + 0.5) / 2.0 - cx, ((j as f64 + 0.5) / 4.0 - cy) * 2.0);
            let u = (x * cpa + y * spa) / R;
            let v = (-x * spa + y * cpa) / (R * INC);
            let r = u.hypot(v);
            if r > 1.1 {
                if r > 1.25 && rand.next_f64() < 0.003 {
                    back.push(j * w + i);
                }
                continue;
            }
            // The bulge is a squat ball, so on screen it looks rounder than the disc.
            let bulge = 1.6 * (-(u.hypot(v * INC * 1.4) / 0.17).powi(2)).exp();
            let disc = 0.15 * (-r / 0.3).exp();
            // Arms grow out of the bulge, then dim and thin away to two loose tips.
            let s = ((r - 0.12) / 0.16).clamp(0.0, 1.0);
            let reach = s * s * (3.0 - 2.0 * s) * (1.3 - 0.8 * r) * (-(r / 0.8).powi(3)).exp();
            spots.push(Spot {
                k: j * w + i,
                base: (bulge + disc) as f32,
                phase: (v.atan2(u) - (r + 1e-6).ln() * wind) as f32,
                reach: reach as f32,
                dust: (0.75 * reach * (r / 0.3).min(1.0)) as f32,
                bulge: (bulge / (bulge + disc + 1e-9)) as f32,
                threshold: (0.07 + 0.9 * rand.next_f64()) as f32,
                knot: rand.next_f64() < 0.05,
            });
        }
    }
    Box::new(Galaxy { spots, stars, back, dots })
}

impl Piece for Galaxy {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cx, cy) = (META.cols as f64 / 2.0, META.rows as f64 / 2.0);
        let turn = -PATTERN * t;
        let w = self.dots.width();
        self.dots.clear();
        for &k in &self.back {
            self.dots.set((k % w) as i64, (k / w) as i64, BACK);
        }
        for s in &self.spots {
            let wv = 2.0 * (f64::from(s.phase) - turn);
            let arm = (0.5 + 0.5 * wv.cos()).powf(3.5);
            let dust = (0.5 + 0.5 * (wv - 1.3).cos()).powi(8); // on each arm's inner edge
            let reach = f64::from(s.reach);
            let shade = 1.0 - f64::from(s.dust) * dust;
            let field = (f64::from(s.base) + arm * reach) * shade;
            let b = 1.0 - (-1.8 * field).exp();
            if b <= f64::from(s.threshold) {
                continue;
            }
            let color = if s.bulge > 0.6 {
                if b > 0.85 {
                    CORE
                } else {
                    WARM
                }
            } else if shade < 0.6 {
                DUST
            } else if arm * reach > 0.45 {
                if s.knot {
                    KNOT
                } else {
                    BRIGHT
                }
            } else if arm * reach > 0.12 {
                ARM
            } else {
                FAINT
            };
            self.dots.set((s.k % w) as i64, (s.k / w) as i64, color);
        }
        // Inner stars sweep round faster: angular speed falls off as 1 / r.
        let (spa, cpa) = PA.sin_cos();
        for &(r, a0, b) in &self.stars {
            let a = a0 - b * t / (r + 0.08);
            let (u, v) = (r * a.cos() * R, r * a.sin() * R * INC);
            let (x, y) = (u * cpa - v * spa, u * spa + v * cpa);
            self.dots.set(((cx + x) * 2.0).floor() as i64, ((cy + y / 2.0) * 4.0).floor() as i64, STAR);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
