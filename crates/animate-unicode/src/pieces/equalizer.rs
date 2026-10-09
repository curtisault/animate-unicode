//! equalizer: a spectrum analyser over a short drum and bass loop. Bars jump
//! to each band's level and sink back, and peak caps hold, then drop slowly.
//! After ascii.rest's equalizer.ts (MIT, @bas3line).
//!
//! The loop and the analyser are ascii.rest's. The bars are lit as a
//! hardware analyser's are, by height: green at the bottom, yellow higher
//! up, red at the top, so a loud band climbs into the warm colours; the
//! peak caps are white.

use crate::{Category, Env, Grid, Meta, Options, Piece, Ramp};

/// The equalizer's meta.
pub static META: Meta = Meta {
    name: "equalizer",
    slug: "equalizer",
    category: Category::Data,
    note: "spectrum bars lit green to yellow to red by height, caps falling",
    cols: 62,
    rows: 16,
    fps: 30,
    options: Some(r#"{"bpm": 120}"#),
    // The bars from the bottom row up; the caps; the axis.
    palette: Some(&["#3ac85a", "#5ad85a", "#8ae05a", "#c8e04a", "#f0d84a", "#f4a84a", "#f4704a", "#e8484a", "#ffffff", "#5c6370"]),
    ground: Some("#0a0b0e"),
    ..Meta::DEFAULT
};

const HEIGHT: Ramp = Ramp::new(0, 8);
const CAP: u8 = 8;
const AXIS: u8 = 9;
/// Half octaves from 31 Hz to 16 kHz.
const BANDS: usize = 19;
const LABELS: [&str; 10] = ["31", "63", "125", "250", "500", "1k", "2k", "4k", "8k", "16k"];
/// Plot height in rows.
const PH: i64 = 12;
const EIGHTHS: [char; 7] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇'];
/// Bar fall in heights a second; seconds a cap waits before it drops; how fast it speeds up.
const FALL: f32 = 2.2;
const HOLD: f32 = 0.5;
const GRAVITY: f32 = 1.1;
const KICK_GAIN: f64 = 1.4;
const BASS_GAIN: f64 = 0.34;
const SNARE_GAIN: f64 = 0.95;
const HAT_GAIN: f64 = 0.75;
const CHORD_GAIN: f64 = 0.15;
const KNEE: f64 = 1.4;
/// The loudest a band reaches, so caps always show.
const HEADROOM: f64 = 0.95;
/// Sixteen steps a bar. The loop is four bars, the last with a fill.
const KICK: [&str; 4] = ["x.....x.x...x...", "x.....x.x.....x.", "x.....x.x...x...", "x.....x.x.x.x.xx"];
const SNARE: [&str; 4] = ["....x.......x...", "....x.......x..x", "....x.......x...", "....x..x..x.xxxx"];
const HAT: [&str; 4] = ["x.x.x.xxx.x.x.xx"; 4];
/// The bass's band on each beat, and the chord's bands.
const BASS: [f64; 16] = [0.0, 0.0, 2.0, 3.0, 1.0, 1.0, 3.0, 2.0, 0.0, 0.0, 2.0, 4.0, 3.0, 2.0, 1.0, 0.0];
const CHORDS: [[f64; 3]; 4] = [[8.0, 10.0, 13.0], [7.0, 10.0, 12.0], [9.0, 11.0, 13.0], [8.0, 11.0, 14.0]];

fn hash(n: i64) -> f64 {
    let mut n = n as u32;
    n = (n ^ (n >> 16)).wrapping_mul(0x7feb_352d);
    n = (n ^ (n >> 15)).wrapping_mul(0x846c_a68b);
    f64::from(n ^ (n >> 16)) / 4_294_967_296.0
}

/// Smooth flutter per band, a pure function of time.
fn flutter(b: usize, x: f64) -> f64 {
    let (i, f) = (x.floor() as i64, x.fract());
    let u = f * f * (3.0 - 2.0 * f);
    hash(b as i64 * 7919 + i) * (1.0 - u) + hash(b as i64 * 7919 + i + 1) * u
}

/// The analyser: the step length, the bars, caps, their speeds and holds, and the clock.
pub struct Equalizer {
    step: f64,
    bar: [f32; BANDS],
    peak: [f32; BANDS],
    vel: [f32; BANDS],
    held: [f32; BANDS],
    level: [f64; BANDS],
    last: f64,
    warm: f64,
}

impl Equalizer {
    /// Time since the last hit of a pattern, looking back up to a bar.
    fn since(&self, t: f64, pat: &[&str; 4]) -> f64 {
        let s = (t / self.step).floor() as i64;
        for k in 0..16 {
            let j = s - k;
            let p = pat[j.div_euclid(16).rem_euclid(4) as usize];
            if p.as_bytes()[j.rem_euclid(16) as usize] == b'x' {
                return t - j as f64 * self.step;
            }
        }
        9.0
    }

    fn spectrum(&mut self, t: f64) {
        let kick = (-self.since(t, &KICK) / 0.14).exp();
        let snare = (-self.since(t, &SNARE) / 0.11).exp();
        let hat = (-self.since(t, &HAT) / 0.045).exp();
        let beats = t / (self.step * 4.0);
        let (beat, frac) = (beats.floor() as i64, beats.fract());
        let bass = BASS[beat.rem_euclid(16) as usize] + 0.6;
        let chord = CHORDS[beat.div_euclid(4).rem_euclid(4) as usize];
        // The chord ducks under each beat and swells back, as if sidechained.
        let pump = 0.5 + 0.5 * (1.0 - (-frac * 4.0).exp());
        for b in 0..BANDS {
            let bf = b as f64;
            let n = flutter(b, t * 9.0);
            let mut sum = 0.03 + 0.08 * n - 0.001 * bf // the room
                + KICK_GAIN * kick * (-((bf - 1.2) / 1.4).powi(2)).exp()
                // The bass note and its octave, two bands up.
                + BASS_GAIN * (0.45 + 0.55 * (-frac * 3.0).exp()) * ((-((bf - bass) / 0.9).powi(2)).exp() + 0.5 * (-((bf - bass - 2.0) / 1.1).powi(2)).exp())
                + SNARE_GAIN * (0.75 + 0.25 * n) * snare * (-((bf - 9.0) / 4.2).powi(2)).exp()
                + HAT_GAIN * hat * (-((bf - 16.0) / 2.2).powi(2)).exp() * (0.6 + 0.4 * n);
            // The chord rings in the mids, each tone with a weaker overtone.
            for c in chord {
                sum += CHORD_GAIN * pump * (0.7 + 0.3 * n) * ((-(bf - c).powi(2)).exp() + 0.5 * (-(bf - c - 2.0).powi(2)).exp());
            }
            // A soft knee, so loud bands still move instead of pinning at the top.
            self.level[b] = HEADROOM * (1.0 - (-KNEE * sum).exp());
        }
    }

    fn advance(&mut self, t: f64) {
        let dt = (t - self.last).clamp(0.0, 0.1) as f32;
        self.last = t;
        self.spectrum(t);
        for b in 0..BANDS {
            self.bar[b] = (self.level[b] as f32).max(self.bar[b] - FALL * dt);
            if self.bar[b] >= self.peak[b] {
                self.peak[b] = self.bar[b];
                self.held[b] = HOLD;
                self.vel[b] = 0.0;
            } else {
                self.held[b] -= dt;
                if self.held[b] < 0.0 {
                    self.vel[b] += GRAVITY * dt;
                    self.peak[b] = self.bar[b].max(self.peak[b] - self.vel[b] * dt);
                }
            }
        }
    }

    /// Play a few bars in so frame 0 is mid song, just after a kick.
    fn warm_up(&mut self) {
        self.last = 0.0;
        (self.bar, self.peak, self.vel, self.held) = ([0.0; BANDS], [0.0; BANDS], [0.0; BANDS], [0.0; BANDS]);
        for i in 0..=90 {
            self.advance(self.warm * i as f64 / 90.0);
        }
    }
}

/// An equalizer; reads `bpm`, held to 60 to 180.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let step = 15.0 / options.f64("bpm", 120.0).clamp(60.0, 180.0); // seconds per sixteenth
    let mut e = Equalizer {
        step,
        bar: [0.0; BANDS],
        peak: [0.0; BANDS],
        vel: [0.0; BANDS],
        held: [0.0; BANDS],
        level: [0.0; BANDS],
        last: 0.0,
        warm: step * (16.0 * 3.0 + 6.0) + 0.04,
    };
    e.warm_up();
    Box::new(e)
}

impl Piece for Equalizer {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        if t + self.warm < self.last {
            self.warm_up();
        }
        self.advance(t + self.warm);
        let x0 = (META.cols as i64 - (BANDS as i64 * 3 - 1)) / 2;
        let ay = 1 + PH; // axis row
        grid.clear();
        for c in x0 - 1..=x0 + BANDS as i64 * 3 - 1 {
            grid.set(c, ay, '─', AXIS);
        }
        for (i, s) in LABELS.iter().enumerate() {
            grid.text_at(x0 + i as i64 * 6 + 1 - s.len() as i64 / 2, ay + 1, s, AXIS);
        }
        let shade = |r: i64| HEIGHT.at(r as f64 / (PH - 1) as f64);
        for b in 0..BANDS {
            let x = x0 + b as i64 * 3;
            let h = (f64::from(self.bar[b].min(1.0)) * PH as f64 * 8.0).round() as i64;
            let (full, part) = (h >> 3, h & 7);
            for r in 0..full {
                grid.text_at(x, ay - 1 - r, "██", shade(r));
            }
            if part > 0 {
                let ch = EIGHTHS[part as usize - 1].to_string().repeat(2);
                grid.text_at(x, ay - 1 - full, &ch, shade(full));
            }
            // The cap is one thin line, drawn once a clear cell parts it from the bar.
            let pr = ((f64::from(self.peak[b].min(1.0)) * PH as f64 * 8.0).round() as i64).min(PH * 8 - 1) >> 3;
            if pr >= if part > 0 { full } else { full - 1 } + 2 {
                grid.text_at(x, ay - 1 - pr, "▁▁", CAP);
            }
        }
    }
}
