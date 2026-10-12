//! Logos drawn from SVG: the code the logos, companies and distros share, so
//! each of them is its data and its meta rather than a copy of this.
//!
//! `scripts/logo.py` reduces each logo's SVG to octants, with a wedge where an
//! edge cuts a cell at a slant, and writes it to its category's
//! `art/<slug>.txt`: the drawing in colour, the colour of each cell and of what
//! is behind it (base-36 digits into the logo's colours; where two colours meet
//! in a cell, one is the glyph and the other its back colour), and the drawing
//! in one ink, `rows` lines each. [`make`] reads that and passes a sheen over
//! it now and then: light crossing glossy enamel, a soft band leaning like a
//! slash with a bright line at its heart and a fainter one after it, easing in
//! and out. It is worked out for each octant dot, and a filled cell is
//! dithered between two strengths of its colour (the glyph over a back colour),
//! so the light is smooth rather than stepped cell by cell. In one ink only the
//! bright line passes, as a thread of dots taken out of the logo.
//!
//! The palette is ascii.rest's: the logo's n colours for a light page, then
//! twice lighter for the light's two strengths, then the same three for a dark
//! page, lifted where they would sink into it.

use crate::subcell::{Encoding, Octant};
use crate::{Env, Grid, Meta, Options, Piece};

/// Seconds before the first sheen, and that one takes to cross.
const START: f64 = 0.5;
const PASS: f64 = 2.0;
/// Dots the sheen shifts left a dot down (so it leans like a slash); its soft
/// band's and its bright line's widths in dots; how far behind the line the
/// fainter one runs, and how strong that is.
const LEAN: f64 = 0.6;
const BAND: f64 = 7.0;
const LINE: f64 = 0.9;
const AFTER: f64 = 5.0;
const ECHO: f64 = 0.55;
/// A 2 × 4 cell's dots as dither thresholds, so a level between two strengths
/// lights that share of the dots.
const BAYER: [f64; 8] = [0.0625, 0.5625, 0.8125, 0.3125, 0.1875, 0.6875, 0.9375, 0.4375];

/// A logo: its two drawings, the colours of each cell, and where the sheen runs.
struct Logo {
    meta: &'static Meta,
    colour: Vec<char>,
    /// Each colour cell's dots, if its glyph is an octant (or a block that stands in for one).
    masks: Vec<Option<u8>>,
    ink: Vec<u8>,
    back: Vec<Option<u8>>,
    mono: Vec<char>,
    mono_masks: Vec<Option<u8>>,
    /// The logo's colours.
    n: usize,
    /// Seconds between sheens, 0 for none.
    every: f64,
    /// Where the sheen's middle starts and how far it travels, in dots along
    /// x + LEAN y.
    from: f64,
    span: f64,
}

/// One block of `art`, after the line `# <name>`, as `cols × rows` cells.
fn block(art: &str, name: &str, cols: usize, rows: usize) -> Vec<char> {
    let mut lines = art.lines().skip_while(|l| *l != format!("# {name}")).skip(1).take_while(|l| !l.starts_with("# "));
    let mut out = Vec::with_capacity(cols * rows);
    for _ in 0..rows {
        let line = lines.next().unwrap_or("");
        out.extend(line.chars().chain(std::iter::repeat(' ')).take(cols));
    }
    out
}

/// The dot mask of an octant glyph (or of the block or quadrant that stands
/// in for one), or None for a wedge.
fn octant(ch: char) -> Option<u8> {
    (1..=255).find(|&m| Octant::glyph(m) == ch)
}

/// A logo from its meta and the art `scripts/logo.py` wrote for it, with a
/// sheen every `shine` seconds as `options` say (5 by default; 0 keeps it still).
pub fn make(meta: &'static Meta, art: &str, options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (meta.cols, meta.rows);
    let colour = block(art, "colour", cols, rows);
    let mono = block(art, "one ink", cols, rows);
    let ink = block(art, "ink", cols, rows).iter().map(|c| c.to_digit(36).unwrap_or(0) as u8).collect();
    let back = block(art, "back", cols, rows).iter().map(|c| c.to_digit(36).map(|d| d as u8)).collect();
    let every = options.f64("shine", 5.0);
    let every = if every > 0.0 { every.max(PASS + 0.5) } else { 0.0 };
    // The sheen crosses the logo's ink, edge to edge, rather than the whole frame.
    let (mut lo, mut hi) = (f64::MAX, f64::MIN);
    for (i, &c) in colour.iter().chain(&mono).enumerate() {
        if c != ' ' {
            let (x, y) = ((i % (cols * rows)) % cols, (i % (cols * rows)) / cols);
            let u = 2.0 * x as f64 + LEAN * 4.0 * y as f64;
            lo = lo.min(u);
            hi = hi.max(u + 2.0 + LEAN * 4.0);
        }
    }
    let reach = 2.5 * BAND + AFTER;
    let masks = colour.iter().map(|&c| octant(c)).collect();
    let mono_masks = mono.iter().map(|&c| octant(c)).collect();
    let n = meta.palette.map_or(1, |p| p.len() / 6);
    Box::new(Logo { meta, colour, masks, ink, back, mono, mono_masks, n, every, from: lo - reach, span: hi - lo + 2.0 * reach })
}

impl Logo {
    /// How strongly the sheen lights dot (gx, gy) when its middle is at `at`:
    /// 0 for not at all to 2 for the brightest.
    fn light(at: f64, gx: f64, gy: f64) -> f64 {
        let d = gx + LEAN * gy - at;
        if d.abs() > 3.0 * BAND + AFTER {
            return 0.0;
        }
        let band = (-(d / BAND).powi(2)).exp();
        let line = (-(d / LINE).powi(2)).exp() + ECHO * (-((d + AFTER) / LINE).powi(2)).exp();
        (0.9 * band + 1.2 * line).min(2.0)
    }
}

impl Piece for Logo {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows, n) = (self.meta.cols, self.meta.rows, self.n);
        let since = t - START;
        let p = if self.every > 0.0 && since >= 0.0 { (since % self.every) / PASS } else { 2.0 };
        // Eased: it gathers speed into the logo and slows leaving it.
        let at = if p < 1.0 { self.from + self.span * p * p * (3.0 - 2.0 * p) } else { f64::MIN / 2.0 };
        let base = if env.paper { 0 } else { 3 * n };
        let tier = |k: usize, c: u8| (base + k.min(2) * n + usize::from(c)) as u8;
        grid.clear();
        for y in 0..rows {
            for x in 0..cols {
                let i = y * cols + x;
                let dot = |d: usize| Self::light(at, (2 * x + d % 2) as f64 + 0.5, (4 * y + d / 2) as f64 + 0.5);
                let (gx, gy) = (x as i64, y as i64);
                if env.mono {
                    // One ink: the bright line alone, a thread of dots taken out.
                    let ch = self.mono[i];
                    if ch == ' ' {
                        continue;
                    }
                    let mut out = ch;
                    if let Some(mask) = self.mono_masks[i] {
                        let mut keep = mask;
                        for d in 0..8 {
                            let u = (2 * x + d % 2) as f64 + 0.5 + LEAN * ((4 * y + d / 2) as f64 + 0.5) - at;
                            if u.abs() < 0.75 {
                                keep &= !(1 << d);
                            }
                        }
                        if keep != 0 {
                            out = Octant::glyph(keep);
                        }
                    }
                    grid.set(gx, gy, out, 0);
                    continue;
                }
                let ch = self.colour[i];
                if ch == ' ' {
                    continue;
                }
                let ink = self.ink[i];
                // How lit each dot is, dithered to a whole strength.
                let mut level = [0usize; 8];
                let mut sum = 0.0;
                for (d, l) in level.iter_mut().enumerate() {
                    let v = dot(d);
                    sum += v;
                    *l = ((v + BAYER[d]).floor() as usize).min(2);
                }
                match (self.back[i], self.masks[i]) {
                    // Two colours: each lifted by the cell's strength.
                    (Some(b), _) => {
                        let k = ((sum / 8.0 + BAYER[(y % 2) * 4 + x % 4]).floor() as usize).min(2);
                        grid.set(gx, gy, ch, tier(k, ink));
                        grid.set_back(gx, gy, tier(k, b));
                    }
                    // A filled cell: its brighter dots in the brighter strength over the dimmer.
                    (None, Some(255)) => {
                        let (lo, hi) = (*level.iter().min().unwrap_or(&0), *level.iter().max().unwrap_or(&0));
                        if lo == hi {
                            grid.set(gx, gy, '█', tier(lo, ink));
                        } else {
                            let mask = (0..8).filter(|&d| level[d] * 2 > lo + hi).fold(0u8, |m, d| m | 1 << d);
                            grid.set(gx, gy, Octant::glyph(mask.max(1)), tier(hi, ink));
                            grid.set_back(gx, gy, tier(lo, ink));
                        }
                    }
                    // An edge: one strength, from its own lit dots (all of a wedge's).
                    (None, mask) => {
                        let m = mask.unwrap_or(255);
                        let (s, c) = (0..8).filter(|&d| m >> d & 1 == 1).fold((0.0f64, 0.0f64), |(s, c), d| (s + dot(d), c + 1.0));
                        let k = ((s / c.max(1.0) + BAYER[(y % 2) * 4 + x % 4]).floor() as usize).min(2);
                        grid.set(gx, gy, ch, tier(k, ink));
                    }
                }
            }
        }
    }
}
