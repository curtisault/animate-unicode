//! Logos drawn from SVG: the code the logos, companies and distros share, so
//! each of them is its data and its meta rather than a copy of this.
//!
//! `scripts/logo.py` reduces each logo's SVG to octants, with a wedge where an
//! edge cuts a cell at a slant, and writes it to its category's
//! `art/<slug>.txt`: the drawing in colour, the colour of each cell (a base-36
//! digit into the logo's colours), and the drawing in one ink, `rows` lines
//! each. [`make`] reads that and plays one of ascii.rest's two loops over it
//! (MIT, @bas3line): a glint leaning like a slash that crosses the logo now and
//! then (its logos and companies), or a scan line that runs down it and
//! scrambles what it passes (its distros).
//!
//! The palette is ascii.rest's: the logo's n colours for a light page, then
//! twice lighter for the loop's two strengths, then the same three for a dark
//! page, lifted where they would sink into it.

use crate::subcell::{Encoding, Octant};
use crate::{Env, Grid, Meta, Options, Piece};

/// Which loop a logo plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loop {
    /// A glint crossing it, with the option `shine`: seconds between glints.
    Glint,
    /// A scan line running down it, with the option `scan`: seconds between scans.
    Scan,
}

/// Seconds before the first glint or scan, and that it takes to cross.
const START: f64 = 0.5;
const PASS: f64 = 2.0;
/// The glint's half width in cells, and the cells it shifts left a row down.
const HALF: f64 = 5.0;
const LEAN: f64 = 0.9;
/// Rows the scan line scrambles, and rows of afterglow behind it.
const HEAD: f64 = 2.0;
const TRAIL: f64 = 5.0;

/// A logo: its two drawings, the colour of each cell, and where its loop runs.
struct Logo {
    meta: &'static Meta,
    kind: Loop,
    colour: Vec<char>,
    ink: Vec<u8>,
    mono: Vec<char>,
    /// The logo's colours.
    n: usize,
    /// Seconds between loops, 0 for none.
    every: f64,
    /// Where the loop starts and how far it runs: for the glint in x + LEAN y,
    /// for the scan line in rows.
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

/// A logo from its meta and the art `scripts/logo.py` wrote for it, playing
/// `kind` every so many seconds as `options` say (5 by default; 0 keeps it still).
pub fn make(meta: &'static Meta, art: &str, kind: Loop, options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (meta.cols, meta.rows);
    let colour = block(art, "colour", cols, rows);
    let mono = block(art, "one ink", cols, rows);
    let ink = block(art, "ink", cols, rows).iter().map(|c| c.to_digit(36).unwrap_or(0) as u8).collect();
    let every = options.f64(if kind == Loop::Glint { "shine" } else { "scan" }, 5.0);
    let every = if every > 0.0 { every.max(PASS + 0.5) } else { 0.0 };
    // The loop crosses the logo's ink, edge to edge, rather than the whole frame.
    let inked = || colour.iter().chain(&mono).enumerate().filter(|(_, &c)| c != ' ').map(|(i, _)| ((i % (cols * rows)) % cols, (i % (cols * rows)) / cols));
    let (from, span) = match kind {
        Loop::Glint => {
            let along = inked().map(|(x, y)| x as f64 + LEAN * y as f64);
            let (lo, hi) = along.fold((f64::MAX, f64::MIN), |(lo, hi), a| (lo.min(a), hi.max(a)));
            (lo - HALF, hi - lo + 2.0 * HALF)
        }
        Loop::Scan => {
            let (first, last) = inked().fold((rows, 0), |(f, l), (_, y)| (f.min(y), l.max(y)));
            (first as f64, (last + 1 - first) as f64 + HEAD + TRAIL)
        }
    };
    let n = meta.palette.map_or(1, |p| p.len() / 6);
    Box::new(Logo { meta, kind, colour, ink, mono, n, every, from, span })
}

/// The dot mask of an octant glyph (or of the block or quadrant that stands
/// in for one), or None for a wedge.
fn octant(ch: char) -> Option<u8> {
    (1..=255).find(|&m| Octant::glyph(m) == ch)
}

/// ascii.rest's scan-line hash: Math.imul's wrapping products on 32 bits.
fn hash(x: i32, y: i32, k: i32) -> u32 {
    let h = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663) ^ k.wrapping_mul(83_492_791);
    let h = (h ^ ((h as u32) >> 13) as i32).wrapping_mul(1_274_126_177);
    (h ^ ((h as u32) >> 16) as i32) as u32
}

impl Piece for Logo {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows, n) = (self.meta.cols, self.meta.rows, self.n);
        let pic = if env.mono { &self.mono } else { &self.colour };
        let since = t - START;
        let at = if self.every > 0.0 && since >= 0.0 { self.from + self.span * (since % self.every) / PASS } else { f64::MIN };
        let tick = (t * f64::from(self.meta.fps) / 2.0).floor() as i32; // the noise changes every other frame
        let base = if env.paper { 0 } else { 3 * n };
        grid.clear();
        for y in 0..rows {
            for x in 0..cols {
                let i = y * cols + x;
                let mut ch = pic[i];
                if ch == ' ' {
                    continue;
                }
                let level = match self.kind {
                    Loop::Glint => {
                        let d = (x as f64 + LEAN * y as f64 - at).abs();
                        let k = if d < HALF { 1.0 - d / HALF } else { 0.0 };
                        let k = k * k * (3.0 - 2.0 * k);
                        // Cells at least half full turn to slashes a dot apart in its
                        // core; thinner edges and wedges keep their shape.
                        if let Some(mask) = (k > 0.55).then(|| octant(ch)).flatten().filter(|m| m.count_ones() >= 4) {
                            let mut slash = 0u8;
                            for d in 0..8 {
                                if !(2 * x + d % 2 + 4 * y + d / 2).is_multiple_of(3) {
                                    slash |= 1 << d;
                                }
                            }
                            ch = Octant::glyph((mask & slash).max(1));
                        }
                        usize::from(k > 0.25) + usize::from(k > 0.6)
                    }
                    Loop::Scan => {
                        let d = at - y as f64; // rows since the line reached this one
                        if (0.0..HEAD).contains(&d) {
                            // Scrambled to octant noise, a quarter of the cells left be.
                            let h = hash(x as i32, y as i32, tick);
                            if !h.is_multiple_of(4) {
                                ch = Octant::glyph(((h >> 2) as u8).max(1));
                            }
                            2
                        } else {
                            usize::from((HEAD..HEAD + TRAIL).contains(&d))
                        }
                    }
                };
                grid.set(x as i64, y as i64, ch, (base + level * n + usize::from(self.ink[i])) as u8);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_matches_ascii_rests() {
        // ascii.rest's hash in ubuntu.ts, run in node.
        assert_eq!([hash(0, 0, 0), hash(3, 7, 11), hash(57, 27, 1234), hash(-1, 5, 99_999)], [0, 4_083_866_278, 4_151_082_410, 2_731_375_650]);
    }
}
