//! wave interference: two dippers in a round ripple tank, seen from above.
//! Where their waves cancel the water lies still, and those calm lines fan
//! out and swing as the dippers drift. After ascii.rest's
//! wave-interference.ts (MIT, @bas3line).
//!
//! The waves are ascii.rest's, worked out per octant dot, eight a character.
//! Crests are cyan going to white at their peaks and troughs deep blue, each
//! dithered by how high or low the water stands, so where the waves cancel
//! the dots die away and the still lines show as dark lanes finer than a
//! character, fanning out from between the dippers.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// Wave interference's meta.
pub static META: Meta = Meta {
    name: "wave interference",
    slug: "wave-interference",
    category: Category::Physics,
    note: "a ripple tank in octants, crests cyan, troughs blue, still lines dark",
    cols: 72,
    rows: 26,
    fps: 24,
    charset: Charset::Extended,
    // Troughs, shallow to deep; crests, low to high; the dippers; the rim.
    palette: Some(&["#1c3a6a", "#1a2c5a", "#14204a", "#2a8aa0", "#4ac0d0", "#a8eef4", "#ffffff", "#5a6270"]),
    ground: Some("#060a14"),
    options: Some(r#"{"wavelength": 5, "separation": 2.2}"#),
    ..Meta::DEFAULT
};

/// Seconds per wave.
const PERIOD: f64 = 1.0;
const TROUGH: Ramp = Ramp::new(0, 3);
const CREST: Ramp = Ramp::new(3, 3);
const DIPPER: u8 = 6;
const RIM: u8 = 7;

/// The tank: its wavelength and how many wavelengths apart the dippers sit, and the dots.
pub struct WaveInterference {
    lam: f64,
    separation: f64,
    dots: ColorOctants,
}

/// Wave interference; reads `wavelength` (in rows, a column being half a
/// row) and `separation` (in wavelengths).
pub fn make(options: &Options) -> Box<dyn Piece> {
    let lam = options.f64("wavelength", 5.0).max(3.0);
    let separation = options.f64("separation", 2.2);
    Box::new(WaveInterference { lam, separation, dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for WaveInterference {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let k = TAU / self.lam;
        // The tank's middle and its half width and height, in rows.
        let (cx, cy) = (META.cols as f64 / 4.0, META.rows as f64 / 2.0);
        let (a, b) = (cx - 0.6, cy - 0.4);
        let w = TAU * t / PERIOD;
        // The dippers drift apart and together and their axis rocks; both repeat within 90 seconds.
        let half = (b - 1.5).min(self.lam * self.separation.abs() * (1.0 + 0.2 * (TAU * t / 30.0).sin()) / 2.0);
        let tilt = 0.25 * (TAU * t / 45.0).sin();
        let s = [(cx - half * tilt.sin(), cy - half * tilt.cos()), (cx + half * tilt.sin(), cy + half * tilt.cos())];
        let (dw, dh) = (self.dots.width(), self.dots.height());
        self.dots.clear();
        for py in 0..dh {
            let y = (py as f64 + 0.5) / 4.0;
            for px in 0..dw {
                let x = (px as f64 + 0.5) / 4.0;
                let q = (((x - cx) / a).powi(2) + ((y - cy) / b).powi(2)).sqrt();
                if q >= 1.0 {
                    continue;
                }
                let d1 = ((x - s[0].0).powi(2) + (y - s[0].1).powi(2)).sqrt();
                let d2 = ((x - s[1].0).powi(2) + (y - s[1].1).powi(2)).sqrt();
                let (a1, a2) = (1.0 / (1.0 + d1 * 0.1).sqrt(), 1.0 / (1.0 + d2 * 0.1).sqrt());
                // The surface height, out of the most two waves can give; the shallows by the rim calm it.
                let calm = ((1.0 - q) / 0.2).min(1.0);
                let h = (a1 * (k * d1 - w).cos() + a2 * (k * d2 - w).cos()) / (a1 + a2) * calm * calm * (3.0 - 2.0 * calm);
                // Height is ink either way, crest or trough, so the still lines are bare on paper too.
                if dither(px, py, h.abs() * 1.15) {
                    self.dots.set(px as i64, py as i64, if h > 0.0 { CREST.at(h) } else { TROUGH.at(-h) });
                }
            }
        }
        // The tank's rim, and each dipper in a little calm of its own.
        let steps = 900;
        for i in 0..steps {
            let p = i as f64 / steps as f64 * TAU;
            self.dots.set(((cx + a * p.cos()) * 4.0) as i64, ((cy + b * p.sin()) * 4.0) as i64, RIM);
        }
        for &(sx, sy) in &s {
            let (x, y) = ((sx * 4.0) as i64, (sy * 4.0) as i64);
            for dy in -3i64..=3 {
                for dx in -3i64..=3 {
                    let r2 = dx * dx + dy * dy;
                    if r2 <= 2 {
                        self.dots.set(x + dx, y + dy, DIPPER);
                    } else if r2 <= 10 {
                        self.dots.unset(x + dx, y + dy);
                    }
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
