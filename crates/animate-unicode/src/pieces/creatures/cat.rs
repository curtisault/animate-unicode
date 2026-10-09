//! cat: a tabby asleep on its side, head down on its front paws. Its back
//! rises and falls with each breath, the tail tip lifts now and then, an ear
//! twitches, and a z drifts up. After ascii.rest's cat.ts (MIT, @bas3line).
//!
//! The pose, the breathing and the timings are ascii.rest's; the cat is
//! redrawn in octants from round shapes, a long body and haunch, a round
//! head with two ears, front paws under the chin and a tail along the
//! ground, so its back swells smoothly with each breath and the tail lifts in
//! a curve. It is a tabby: a warm orange-brown coat with dark stripes across
//! the back and haunch, drawn sparser than the coat so they show in one ink
//! too, a pale belly and paws, pink in the ears and on the nose, and its
//! closed eyes two dark crescents.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The cat's meta.
pub static META: Meta = Meta {
    name: "cat",
    slug: "cat",
    category: Category::Creatures,
    note: "an octant tabby asleep on its paws, striped, breathing slowly",
    cols: 56,
    rows: 15,
    fps: 15,
    charset: Charset::Extended,
    // The coat, its stripes, the belly and paws, pink, the eyes; the zs.
    palette: Some(&["#cc8a4c", "#6e4222", "#ecd2ac", "#ee98a6", "#3a2618", "#9ab0d8"]),
    ground: Some("#0e0d0c"),
    ..Meta::DEFAULT
};

const COAT: u8 = 0;
const STRIPE: u8 = 1;
const PALE: u8 = 2;
const PINK: u8 = 3;
const LID: u8 = 4;
const ZED: u8 = 5;
/// Seconds a loop, and a breath (four to a loop); when the tail tip lifts,
/// when an ear twitches, and how often and how long a z drifts up.
const LOOP: f64 = 16.0;
const BREATH: f64 = 4.0;
const FLICK: [(f64, f64); 2] = [(1.9, 3.4), (10.4, 11.6)];
const TWITCH: [(f64, f64); 3] = [(4.5, 4.8), (4.95, 5.3), (12.8, 13.1)];
const Z_EVERY: f64 = 1.6;
const Z_LIFE: f64 = 3.2;

fn ellipse(x: f64, y: f64, cx: f64, cy: f64, rx: f64, ry: f64) -> f64 {
    ((x - cx) / rx).powi(2) + ((y - cy) / ry).powi(2)
}

/// Whether (x, y) is within `w` rows of the segment a-b, a row two columns.
fn near(x: f64, y: f64, (ax, ay): (f64, f64), (bx, by): (f64, f64), w: f64) -> bool {
    let (dx, dy) = ((bx - ax) / 2.0, by - ay);
    let (px, py) = ((x - ax) / 2.0, y - ay);
    let k = ((px * dx + py * dy) / (dx * dx + dy * dy)).clamp(0.0, 1.0);
    (px - k * dx).hypot(py - k * dy) < w
}

/// The cat's paint at (x, y) in columns and rows and the part it is on
/// (head, ears, paws, tail, body), given how far its back is lifted, how far
/// its tail tip, and whether its ear is laid back.
fn paint(x: f64, y: f64, rise: f64, up: f64, ear_back: bool) -> Option<(u8, u8)> {
    // Eyes shut: a dark crescent each, and a pink nose.
    for ex in [37.4, 43.0] {
        let d = ellipse(x, y, ex, 6.7, 1.6, 0.55);
        if d < 1.0 && y > 6.7 {
            return Some((LID, 1));
        }
    }
    if ellipse(x, y, 40.2, 8.0, 0.9, 0.35) < 1.0 {
        return Some((PINK, 1));
    }
    // The ears: the left upright, the right upright or laid flat.
    let ear = |apex: (f64, f64), half: f64, base: f64| y >= apex.1 && y <= base && (x - apex.0).abs() < half * (y - apex.1) / (base - apex.1);
    let right = if ear_back { ear((46.5, 4.1), 3.0, 5.0) } else { ear((45.3, 2.6), 2.4, 4.9) };
    if ear((35.2, 2.6), 2.4, 4.9) || right {
        let inner = ear((35.2, 3.3), 1.1, 4.8) || (!ear_back && ear((45.3, 3.3), 1.1, 4.8));
        return Some((if inner { PINK } else { COAT }, 2));
    }
    // The head, resting on its paws.
    if ellipse(x, y, 40.2, 6.9, 7.2, 2.6) < 1.0 {
        // A tabby's forehead marks.
        let m = y < 5.6 && ((x - 38.6).abs() < 0.35 || (x - 40.2).abs() < 0.35 || (x - 41.8).abs() < 0.35);
        return Some((
            if m {
                STRIPE
            } else if y > 8.4 {
                PALE
            } else {
                COAT
            },
            1,
        ));
    }
    if ellipse(x, y, 37.6, 10.4, 2.6, 0.9) < 1.0 || ellipse(x, y, 43.6, 10.4, 2.6, 0.9) < 1.0 {
        return Some((PALE, 3));
    }
    // The tail, from the rump along the ground to its tip, which lifts.
    let tip = (2.6, 10.6 - 3.2 * up);
    let tail = [(10.0, 11.4), (6.4, 12.3), (3.4, 11.9), tip];
    if tail.windows(2).any(|s| near(x, y, s[0], s[1], 0.6)) {
        return Some((if ((x * 1.3).sin() > 0.4) && y > 11.0 { STRIPE } else { COAT }, 4));
    }
    // The body: long, its back swelling with each breath; the haunch at the rear.
    let back = 4.6 - 0.9 * rise;
    let in_body = ellipse(x, y, 22.0, 9.0 - rise * 0.3, 15.5, (12.2 - back) / 2.0) < 1.0;
    let in_haunch = ellipse(x, y, 13.6, 9.4, 5.2, 2.6) < 1.0;
    if in_body || in_haunch || (x > 30.0 && x < 37.0 && y > 6.5 && y < 11.6) {
        if y > 10.6 && x > 14.0 {
            return Some((PALE, 5));
        }
        // Stripes curving down the back and round the haunch.
        let s = if in_haunch && !in_body { (x - 13.6).hypot((y - 9.4) * 2.0) * 1.6 } else { x * 0.95 + (y - 8.0) * 0.7 };
        return Some((if s.sin() > 0.45 && y < 10.0 { STRIPE } else { COAT }, 5));
    }
    None
}

/// The cat's dots.
pub struct Cat {
    dots: ColorOctants,
}

/// A cat; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Cat { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Cat {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let u = t.rem_euclid(LOOP);
        let within = |u: f64, spans: &[(f64, f64)]| spans.iter().any(|&(a, b)| u >= a && u < b);
        // A breath: in, out a little slower, then a pause before the next.
        let p = (u + 0.4).rem_euclid(BREATH) / BREATH;
        let rise = if p < 0.35 {
            (PI / 2.0 * p / 0.35).sin().powi(2)
        } else if p < 0.8 {
            (PI / 2.0 * (p - 0.35) / 0.45).cos().powi(2)
        } else {
            0.0
        };
        // The tail tip lifts and settles; now and then an ear lies back.
        let up = FLICK.iter().find(|&&(a, b)| u >= a && u < b).map_or(0.0, |&(a, b)| (PI * (u - a) / (b - a)).sin());
        let ear_back = within(u, &TWITCH);
        // The coat is dense; the stripes sparser and the shut eyes gaps, so
        // they show in one ink too, and a dot is left out wherever one part
        // meets another, so the parts keep their outlines.
        self.dots.clear();
        let at: Vec<Option<(u8, u8)>> = (0..w * h).map(|k| paint((k % w) as f64 / 2.0 + 0.25, (k / w) as f64 / 4.0 + 0.125, rise, up, ear_back)).collect();
        for (k, a) in at.iter().enumerate() {
            let Some((c, part)) = *a else { continue };
            let other = |n: usize| at.get(n).copied().flatten().is_some_and(|(_, p)| p != part);
            let (i, j) = (k % w, k / w);
            if c == LID || (i + 1 < w && other(k + 1)) || other(k + w) || (c == STRIPE && !dither(i, j, 0.45)) {
                continue;
            }
            self.dots.set(i as i64, j as i64, c);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // Zs drift up from over the head, swaying and growing as they go.
        for k in -2..=(LOOP / Z_EVERY) as i64 {
            let age = u - k as f64 * Z_EVERY;
            if !(0.0..Z_LIFE).contains(&age) || within((k as f64 * Z_EVERY).rem_euclid(LOOP), &TWITCH) {
                continue;
            }
            let f = age / Z_LIFE;
            let c = (48.5 + f * 4.0 + (age * 2.6 + k as f64).sin() * 0.7).floor() as i64;
            let r = (3.7 - f * 3.7).floor() as i64;
            if r >= 0 && c < META.cols as i64 {
                grid.set(c, r, if f < 0.5 { 'z' } else { 'Z' }, ZED);
            }
        }
    }
}
