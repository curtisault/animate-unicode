//! pond ripples: a pond from above in light rain. Each drop lands with a
//! splash and opens a ring, a fainter one following it in, that weakens to
//! dashes and dots before it reaches the bank. Where two rings cross they
//! peak. After ascii.rest's pond-ripples.ts (MIT, @bas3line).
//!
//! The rain is ascii.rest's, from the same hash. In braille the dots are
//! square, so each ring is a true circle a dot wide, where a character grid
//! could only manage an ellipse of strokes. Rings are water blue, bright
//! while strong and dim as they weaken; where two strong rings cross, the
//! crossing flashes white.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// Pond ripples' meta.
pub static META: Meta = Meta {
    name: "pond ripples",
    slug: "pond-ripples",
    category: Category::Physics,
    note: "raindrops opening true circles on a pond, crossings flashing white",
    cols: 64,
    rows: 20,
    fps: 20,
    charset: Charset::Extended,
    // The faint inner ring, a weakening ring, a full ring, a crossing; the bank.
    palette: Some(&["#1f4a6a", "#3a7aa8", "#7ac4ec", "#ffffff", "#4a6a3a"]),
    ground: Some("#071018"),
    ..Meta::DEFAULT
};

/// A dot in em (a cell is 0.6 by 1.2: two dots across, four down).
const DOT: f64 = 0.3;
/// Seconds into the rain: two rings open and a third drop about to land.
const START: f64 = 7.35;
/// Seconds between drops on average; seconds a ring lasts; em a second; em
/// from a ring to the fainter one inside it.
const EVERY: f64 = 1.5;
const LIFE: f64 = 4.0;
const SPEED: f64 = 2.6;
const WAVE: f64 = 1.5;
const FAINT: u8 = 0;
const WEAK: u8 = 1;
const FULL: u8 = 2;
const PEAK: u8 = 3;
const BANK: u8 = 4;

/// ascii.rest's hash of (drop, salt) into [0, 1).
fn rand(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b) ^ ((s + 1) as u32).wrapping_mul(0xc2b2_ae35);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    f64::from(h ^ (h >> 15)) / 4_294_967_296.0
}

/// The pond: which ring drew each dot, and the dots.
pub struct PondRipples {
    owner: Vec<i16>,
    dots: ColorDots,
}

/// Pond ripples; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let n = dots.width() * dots.height();
    Box::new(PondRipples { owner: vec![-1; n], dots })
}

impl Piece for PondRipples {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let t = START + time;
        let (w, h) = (self.dots.width(), self.dots.height());
        let (cx, cy) = (w as f64 * DOT / 2.0, h as f64 * DOT / 2.0);
        // The bank, its top and bottom flat along a row's edge.
        let (ax, ay) = (cx - 1.2, cy - 1.12);
        // How far in from the bank: 1 well inside, falling to 0 just short of it.
        let open = |x: f64, y: f64| {
            let e = ((1.0 - ((x - cx) / ax).powi(2) - ((y - cy) / ay).powi(2)) / 0.35).clamp(0.0, 1.0);
            e * e * (3.0 - 2.0 * e)
        };
        // The drops alive now, oldest first: where each fell, its age and how hard it struck.
        let mut drops = vec![];
        for i in ((t - LIFE) / EVERY).floor() as i64 - 1..=(t / EVERY).floor() as i64 + 1 {
            let age = t - (i as f64 + 0.7 * (rand(i, 0) - 0.5)) * EVERY;
            if !(0.0..=LIFE).contains(&age) {
                continue;
            }
            let (r, p) = (0.62 * rand(i, 1).sqrt(), TAU * rand(i, 2));
            drops.push((cx + ax * r * p.cos(), cy + ay * r * p.sin(), age, 0.8 + 0.2 * rand(i, 3)));
        }
        drops.sort_by(|a, b| b.2.total_cmp(&a.2));
        self.dots.clear();
        self.owner.fill(-1);
        let to_dot = |x: f64, y: f64| ((x / DOT).floor() as i64, (y / DOT).floor() as i64);
        for (who, &(x0, y0, age, a)) in drops.iter().enumerate() {
            let who = who as i16;
            let put = |x: f64, y: f64, level: u8, dots: &mut ColorDots, owner: &mut [i16]| {
                let (px, py) = to_dot(x, y);
                if px < 0 || py < 0 || px >= w as i64 || py >= h as i64 {
                    return;
                }
                let k = py as usize * w + px as usize;
                match dots.get(px, py) {
                    // Two strong rings crossing peak.
                    Some(FULL) if level == FULL && owner[k] != who => dots.set(px, py, PEAK),
                    Some(c) if c >= level => {}
                    _ => {
                        dots.set(px, py, level);
                        owner[k] = who;
                    }
                }
            };
            // The splash: a drop, then a small ring.
            if age < 0.4 {
                let r = if age < 0.15 { 0.0 } else { 0.45 };
                for k in 0..8 {
                    let p = k as f64 * TAU / 8.0;
                    put(x0 + r * p.cos(), y0 + r * p.sin(), FULL, &mut self.dots, &mut self.owner);
                }
                continue;
            }
            let rad = 0.5 + SPEED * age;
            let life = a * (1.0 - age / LIFE);
            // The ring: whole while strong, dashed as it weakens; a dot a step of arc.
            let steps = (TAU * rad / DOT * 1.5).ceil() as usize;
            for k in 0..steps {
                let p = k as f64 / steps as f64 * TAU;
                let (x, y) = (x0 + rad * p.cos(), y0 + rad * p.sin());
                let near = open(x, y);
                if life > 0.45 && near > 0.6 {
                    put(x, y, FULL, &mut self.dots, &mut self.owner);
                } else if life > 0.2 && near > 0.45 && ((p + PI) * rad / 0.9).floor() as i64 % 2 == 1 {
                    put(x, y, WEAK, &mut self.dots, &mut self.owner);
                }
            }
            // The fainter ring following it in, only while the first is strong; dotted.
            let r2 = rad - WAVE;
            if life > 0.55 && r2 > 1.4 {
                let steps = (TAU * r2 / DOT).ceil() as usize;
                for k in (0..steps).step_by(3) {
                    let p = k as f64 / steps as f64 * TAU;
                    put(x0 + r2 * p.cos(), y0 + r2 * p.sin(), FAINT, &mut self.dots, &mut self.owner);
                }
            }
        }
        // The bank all round.
        let steps = (TAU * ax.max(ay) / DOT * 1.5) as usize;
        for k in 0..steps {
            let p = k as f64 / steps as f64 * TAU;
            let (px, py) = to_dot(cx + ax * p.cos(), cy + ay * p.sin());
            self.dots.set(px, py, BANK);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
