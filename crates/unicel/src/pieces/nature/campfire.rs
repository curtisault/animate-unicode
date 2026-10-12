//! campfire: two logs lying crossed on the ground, burning where they meet.
//! Tongues of flame sway and tear over a low body of fire, the bark chars and
//! glows beneath them, and sparks lift off the tips as they climb. After
//! ascii.rest's campfire.ts (MIT, @bas3line).
//!
//! The tongues, the tearing noise, the rule that only flame joined to the
//! fire is drawn, the logs and the sparks are ascii.rest's, worked out per
//! octant dot instead of per cell, so the flame's ragged edge and the round
//! logs are eight times as fine. In colour the fire runs from deep red at its
//! tips through orange and yellow to white at its heart; the bark is dark and
//! lit along its top, charred where the fire licks it, with embers glowing
//! orange in the char; the near log's cut end shows its rings; coals glow on
//! the ground and the sparks cool from gold to red as they climb.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};

/// The campfire's meta.
pub static META: Meta = Meta {
    name: "campfire",
    slug: "campfire",
    category: Category::Nature,
    note: "an octant log fire, red to white, the bark glowing, sparks rising",
    cols: 52,
    rows: 22,
    fps: 20,
    charset: Charset::Extended,
    // Fire, coolest to hottest; embers, dim to hot; bark, lit bark, char; the
    // cut end and its rings; a spark, and a cooling one.
    palette: Some(&[
        "#5a0c04", "#8c1a06", "#c4320a", "#ea5e14", "#f88e20", "#ffbc3e", "#ffe486", "#fff8e2", "#7a1c08", "#d8461a", "#ff9c3c", "#3a2416", "#6c4628",
        "#1e1612", "#b48a5a", "#7a5a3a", "#ffd474", "#c45a1c",
    ]),
    ground: Some("#0b0806"),
    ..Meta::DEFAULT
};

const FIRE: Ramp = Ramp::new(0, 8);
const EMBER: Ramp = Ramp::new(8, 3);
const BARK: u8 = 11;
const BARK_LIT: u8 = 12;
const CHAR: u8 = 13;
const END: u8 = 14;
const RING: u8 = 15;
const SPARK: u8 = 16;
const SPARK_COOL: u8 = 17;

/// The row the flames stand on.
const BED: f64 = 18.4;
/// Tongues: columns off centre, height in rows, half width, phase.
const TONGUES: [(f64, f64, f64, f64); 6] =
    [(-7.4, 9.5, 3.4, 2.4), (-4.4, 12.5, 3.5, 0.0), (-1.4, 15.0, 3.5, 1.7), (1.7, 15.5, 3.5, 3.1), (4.7, 12.5, 3.5, 4.4), (7.6, 10.0, 3.4, 5.6)];
/// The row the logs' axes cross at; each log: the side its near end lies on,
/// half its length in columns, rows it drops a column toward that end, and
/// its radius in rows.
const MID: f64 = 16.8;
const LOGS: [(f64, f64, f64, f64); 2] = [(1.0, 22.0, 0.115, 1.4), (-1.0, 22.0, 0.115, 1.4)];
const SPARKS: usize = 9;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn noise(x: f64, y: f64) -> f64 {
    let (i, j) = (x.floor() as i64, y.floor() as i64);
    let (u, v) = (ease(x - x.floor()), ease(y - y.floor()));
    let (a, b, c, d) = (hash(i, j), hash(i + 1, j), hash(i, j + 1), hash(i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// A dot of log: its colour, how hot the fire makes it, and whether it is on
/// a near half, in front of the flame.
#[derive(Clone, Copy)]
struct Wood {
    color: u8,
    heat: f32,
    near: bool,
}

/// The fire: the logs by dot, the coals (dot, weight, phase), the sparks, the heat field, and the dots.
pub struct Campfire {
    logs: Vec<Option<Wood>>,
    coals: Vec<(usize, f64, f64)>,
    sparks: Vec<[f64; 5]>,
    fire: Vec<f32>,
    dots: ColorOctants,
}

/// A campfire; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (dots.width(), dots.height());
    let cx = META.cols as f64 / 2.0;
    // Each log as dots: the near half of a log, toward us, is drawn over the
    // far half of the other.
    let mut logs: Vec<Option<Wood>> = vec![None; w * h];
    for &(side, half, k, r0) in &LOGS {
        let seed = if side > 0.0 { 3 } else { 8 };
        for i in ((cx - half) * 2.0) as usize..((cx + half) * 2.0) as usize {
            let x = (i as f64 + 0.5) / 2.0 - cx;
            let axis = MID + k * x * side;
            let front = x * side > 0.0; // the near end is cut and shows its rings
            let warm = (-(x / 13.0).powi(2)).exp();
            let from_end = half - x.abs(); // columns in from the end
            for j in ((axis - r0) * 4.0).floor() as usize..=((axis + r0) * 4.0).ceil() as usize {
                let v = ((j as f64 + 0.5) / 4.0 - axis) / r0; // -1 on top to 1 underneath
                                                              // Round, so it narrows toward its ends a little where it is cut.
                if v.abs() > 1.0 || (from_end < 0.6 && v.abs() > (from_end / 0.6).sqrt()) {
                    continue;
                }
                let k = j * w + i;
                if logs[k].is_some_and(|l| l.near && !front) {
                    continue;
                }
                let top = v < -0.25;
                let (color, heat) = if front && from_end < 1.0 {
                    // The cut end: pale wood in rings.
                    (if ((v.abs() * 3.0 + from_end) as i64) % 2 == 0 { END } else { RING }, 0.0)
                } else {
                    // Bark in runs along the log, lit on top and plainer in its own shadow.
                    let u = x * side + if top { 0.0 } else { 1.7 } + 40.0;
                    let run = (u / 1.3).floor() as i64;
                    let ridged = hash(run, seed + if top { 0 } else { 5 }) < 0.6;
                    let color = if top && warm > 0.5 {
                        CHAR
                    } else if top || (v < 0.2 && ridged) {
                        BARK_LIT
                    } else {
                        BARK
                    };
                    (color, warm * if top { 0.7 } else { 0.3 })
                };
                logs[k] = Some(Wood { color, heat: heat as f32, near: front });
            }
        }
    }
    // Coals on the ground under the crossing, brightest in the middle.
    let mut coals = vec![];
    for j in ((BED.ceil() - 1.0) * 4.0) as usize..h {
        for i in 0..w {
            let x = (i as f64 + 0.5) / 2.0 - cx;
            if logs[j * w + i].is_some() || x.abs() > 10.0 {
                continue;
            }
            let r = j as f64 / 4.0;
            coals.push((j * w + i, (-(x / 6.0).powi(2)).exp() * (1.3 - (r + 1.0 - BED) * 0.35), hash(i as i64, j as i64 + 60) * 9.0));
        }
    }
    // Sparks: each rises from the flames on its own period, wavering.
    let sparks = (0..SPARKS as i64)
        .map(|i| {
            [(hash(i, 1) - 0.5) * 12.0, 2.6 + hash(i, 2) * 2.4, (i as f64 + hash(i, 3) * 0.5) / SPARKS as f64, 7.0 + hash(i, 4) * 6.0, (hash(i, 5) - 0.5) * 8.0]
        })
        .collect();
    Box::new(Campfire { logs, coals, sparks, fire: vec![0.0; w * h], dots })
}

impl Piece for Campfire {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let cx = META.cols as f64 / 2.0;
        let breath = 0.85 + 0.3 * noise(t * 3.0, 7.5);
        let bed = (BED * 4.0).ceil() as usize;
        self.fire.fill(0.0);
        for j in 0..bed {
            let hgt = BED - (j as f64 + 0.5) / 4.0;
            // A low body of fire across the crossing, and the tongues rising
            // out of it, the outer ones leaning in toward the middle.
            let low = hgt / (5.5 * breath);
            let lifts: Vec<f64> = TONGUES.iter().map(|&(_, h0, _, ph)| hgt / (h0 * (0.72 + 0.5 * noise(t * 1.6 + ph * 3.0, ph * 5.0)) * breath)).collect();
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0 - cx;
                let (mut m, mut top) =
                    if low < 1.0 { ((1.0 - (x / (10.5 * (1.0 - low).sqrt() + 0.5)).powi(2)) * (0.85 - low * 0.5), low * 0.5) } else { (0.0, 0.0) };
                for (&(x0, _, w0, ph), &lift) in TONGUES.iter().zip(&lifts) {
                    if lift >= 1.0 {
                        continue;
                    }
                    let sway = 1.4 * lift.powf(1.4) * (hgt * 0.55 - t * 5.5 + ph).sin() + 0.5 * lift * (t * 8.3 + ph * 2.0).sin();
                    let q = (x - x0 * (1.0 - 0.45 * lift) - sway) / (w0 * (1.0 - lift).powf(0.6) + 0.5);
                    let f = (1.0 - q * q) * (1.0 - lift * 0.6);
                    if f > m {
                        (m, top) = (f, lift);
                    }
                }
                // Rising noise tears the fire, more toward the tips than the base.
                let n = noise(x * 0.45, (hgt - t * 6.0) * 0.35) * 0.6 + noise(x * 0.9 + 9.0, (hgt - t * 8.0) * 0.7) * 0.4;
                self.fire[j * w + i] = (m - 0.55 * n * (0.35 + top * 0.65)) as f32;
            }
        }
        // Only flame that joins the body of the fire is drawn: a tongue that
        // tears free goes out.
        let mut stack: Vec<usize> = (0..w).map(|i| (bed - 1) * w + i).filter(|&k| self.fire[k] > 0.03).collect();
        for &k in &stack {
            self.fire[k] += 2.0;
        }
        while let Some(k) = stack.pop() {
            let (i, j) = (k % w, k / w);
            for (di, dj) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0)] {
                let (ni, nj) = (i as i64 + di, j as i64 + dj);
                if ni < 0 || nj < 0 || ni >= w as i64 {
                    continue;
                }
                let n = nj as usize * w + ni as usize;
                if self.fire[n] > 0.03 && self.fire[n] < 2.0 {
                    self.fire[n] += 2.0;
                    stack.push(n);
                }
            }
        }
        self.dots.clear();
        // The near halves of the logs stand in front of all but the hottest
        // flame; the far halves go behind it. Fire is light, so it keeps its
        // sense on paper, the hottest parts the densest ink.
        for j in 0..h {
            for i in 0..w {
                let k = j * w + i;
                let v = f64::from(self.fire[k]) - 2.0;
                let log = self.logs[k];
                if v > 0.03 && !log.is_some_and(|l| l.near && v < 0.6) {
                    let heat = ((v - 0.03) / 0.85).min(1.0);
                    if dither(i, j, 0.2 + heat) {
                        self.dots.set(i as i64, j as i64, FIRE.at(heat));
                    }
                } else if let Some(l) = log {
                    let (c, r) = (i as f64 / 2.0, j as f64 / 4.0);
                    let g = f64::from(l.heat) * (0.45 + 0.75 * noise(c * 0.7 + 3.0, t * 2.2 + r * 3.0)) - 0.3;
                    self.dots.set(i as i64, j as i64, if g > 0.0 { EMBER.at(g * 1.6) } else { l.color });
                }
            }
        }
        // Coals glow on the ground under the crossing.
        for &(k, wt, ph) in &self.coals {
            let b = wt * (0.35 + 0.8 * noise(ph + t * 1.3, ph * 3.0)) - 0.2;
            let (i, j) = (k % w, k / w);
            if b > 0.0 && dither(i, j, b) && self.dots.get(i as i64, j as i64).is_none() {
                self.dots.set(i as i64, j as i64, EMBER.at(b));
            }
        }
        for &[sx, period, phase, rise, drift] in &self.sparks {
            let u = (t / period + phase).rem_euclid(1.0);
            if u > 0.8 {
                continue;
            }
            let y = BED - 8.0 - u * rise;
            let x = cx + sx + drift * u + (u * 11.0 + phase * 6.0).sin();
            let (i, j) = ((x * 2.0).floor() as i64, (y * 4.0).floor() as i64);
            if j >= 0 && self.dots.get(i, j).is_none() {
                self.dots.set(i, j, if u < 0.4 { SPARK } else { SPARK_COOL });
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
