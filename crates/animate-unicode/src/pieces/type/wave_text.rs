//! wave text: a swell rolls into a line of text from the left, lifts and
//! drops each letter in turn, runs off the right, and the line settles level
//! again. After ascii.rest's wave-text.ts (MIT, @bas3line).
//!
//! The swell is ascii.rest's, letter for letter; what it adds is the water's
//! colour, each letter shaded by its height on the wave: white-cyan on a
//! crest, teal at rest, deep blue in a trough, so the swell can be seen
//! travelling along the line even where it barely lifts a letter.

use crate::{Category, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, PI};

/// The wave text's meta.
pub static META: Meta = Meta {
    name: "wave text",
    slug: "wave-text",
    category: Category::Type,
    note: "a swell running through a line of text, letters coloured by height",
    cols: 36,
    rows: 5,
    fps: 15,
    options: Some(r#"{"text": "making waves", "amp": 1.25, "wavelength": 18, "speed": 8, "spacing": 0, "settle": true}"#),
    // Trough to crest.
    palette: Some(&["#1e4a8a", "#2a6aa8", "#3a8ab8", "#4ab0c0", "#7ad0d8", "#b8ecf0", "#f4fcff"]),
    ground: Some("#081018"),
    ..Meta::DEFAULT
};

const HEIGHT: Ramp = Ramp::new(0, 7);

/// The wave: its letters, their spacing and first column, its shape, and the
/// pass's length.
pub struct WaveText {
    chars: Vec<char>,
    step: i64,
    x0: i64,
    amp: f64,
    w: f64,
    v: f64,
    settle: bool,
    start: f64,
    pass: f64,
    l: f64,
}

impl WaveText {
    /// The envelope: half a wavelength to build, a wavelength and a half at
    /// full height, half a wavelength to fade, then level water.
    fn envelope(&self, u: f64) -> f64 {
        let (rise, top) = (self.w / 2.0, 1.5 * self.w);
        if !self.settle {
            1.0
        } else if u < rise {
            (FRAC_PI_2 * (u / rise)).sin().powi(2)
        } else if u < rise + top {
            1.0
        } else if u < self.pass {
            (FRAC_PI_2 * ((u - rise - top) / rise)).cos().powi(2)
        } else {
            0.0
        }
    }

    fn height(&self, u: f64) -> f64 {
        let w = if self.settle { u.rem_euclid(self.l) } else { u };
        self.amp * self.envelope(w) * (2.0 * PI * w / self.w).sin()
    }
}

/// Wave text; reads `text`, `amp` (rows to a crest), `wavelength` (letters
/// crest to crest), `speed` (letters a second), `spacing` and `settle`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as f64);
    let base = (rows - 1.0) / 2.0;
    let amp = options.f64("amp", 1.25).max(0.0).min(base + 0.45);
    let w = options.f64("wavelength", 18.0).max(4.0);
    let v = options.f64("speed", 8.0).max(0.5);
    let step = 1 + options.f64("spacing", 0.0).floor().max(0.0) as i64;
    let settle = options.bool("settle", true);
    let chars: Vec<char> = options.str("text", "making waves").chars().take(((cols - 1) / step + 1) as usize).collect();
    let n = chars.len() as f64;
    let x0 = (cols - (chars.len() as i64 - 1) * step - 1) / 2;
    let pass = w + 1.5 * w;
    // Settling, t = 0 has the front just reaching the first letter, so the
    // line starts level. Endless, it starts with a crest over the middle
    // letter, where the wave is flattest.
    let start = if settle { 0.0 } else { w / 4.0 + (n - 1.0) / 2.0 };
    Box::new(WaveText { chars, step, x0, amp, w, v, settle, start, pass, l: pass + n })
}

impl Piece for WaveText {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let base = (META.rows as i64 - 1) / 2;
        let s = self.start + t * self.v;
        grid.clear();
        for (i, &ch) in self.chars.iter().enumerate() {
            if ch == ' ' {
                continue;
            }
            let u = s - i as f64;
            let h = self.height(u);
            let lift = h.round() as i64;
            let x = self.x0 + i as i64 * self.step;
            let shade = HEIGHT.at((h / self.amp.max(1e-9) + 1.0) / 2.0);
            grid.set(x, base - lift, ch, shade);
            // A letter that has only just stepped off the level line leaves a dot
            // where it sat, so the step reads as motion rather than a jump.
            if lift > 0 && h < 0.7 && self.height(u + 0.5) > h {
                grid.set(x, base, '·', HEIGHT.at(0.5));
            }
        }
    }
}
