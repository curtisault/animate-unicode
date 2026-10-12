//! saptarishi: the seven stars of the Big Dipper where they really sit, with
//! Alcor beside Mizar, in a strip of twinkling sky crossed now and then by a
//! meteor. After ascii.rest's saptarishi.ts (MIT, @bas3line).
//!
//! The projection, the field stars and the meteors' timing are ascii.rest's.
//! The seven are drawn by magnitude, ● for the brightest and • for the rest,
//! each in the colour of its spectral class: Dubhe a K giant's orange, Alkaid
//! a B star's blue, the rest the white of A stars. The asterism's lines are
//! dotted braille, eight dots a cell, that draw themselves in from the
//! handle's tip round the bowl, hold, and fade; the meteors are braille
//! streaks, a white head with a tail that thins and cools behind it.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// Saptarishi's meta.
pub static META: Meta = Meta {
    name: "saptarishi",
    slug: "saptarishi",
    category: Category::Space,
    note: "the big dipper coloured by spectral class, its lines drawn in braille",
    cols: 64,
    rows: 18,
    fps: 12,
    charset: Charset::Extended,
    options: Some(r#"{"lines": true}"#),
    // K, A and B stars; the lines, faint to full; field stars, three tints;
    // the meteor's tail, cool to hot, and its head.
    palette: Some(&["#ffc48a", "#eef2ff", "#a8c2ff", "#2a3458", "#3e4c7c", "#5a6ca4", "#6a6e84", "#8a8494", "#7a8aa4", "#4a5888", "#9aaad8", "#ffffff"]),
    ground: Some("#04050c"),
    ..Meta::DEFAULT
};

const K_STAR: u8 = 0;
const A_STAR: u8 = 1;
const B_STAR: u8 = 2;
const LINE: Ramp = Ramp::new(3, 3);
const FIELD: [u8; 3] = [6, 7, 8];
const TAIL: Ramp = Ramp::new(9, 2);
const HEAD: u8 = 11;

/// Right ascension in hours, declination in degrees, visual magnitude, and
/// spectral colour: Dubhe, Merak, Phecda, Megrez, Alioth, Mizar, Alkaid.
const SEVEN: [(f64, f64, f64, u8); 7] = [
    (11.062, 61.751, 1.8, K_STAR),
    (11.031, 56.383, 2.4, A_STAR),
    (11.897, 53.695, 2.4, A_STAR),
    (12.257, 57.033, 3.3, A_STAR),
    (12.901, 55.96, 1.8, A_STAR),
    (13.399, 54.925, 2.2, A_STAR),
    (13.792, 49.313, 1.9, B_STAR),
];
/// The lines, from the handle's tip round the bowl, in the order they draw in.
const EDGES: [(usize, usize); 7] = [(6, 5), (5, 4), (4, 3), (3, 0), (0, 1), (1, 2), (2, 3)];
/// Seconds between meteors, give or take; the lines' cycle; how long they
/// take to draw in, hold and fade.
const METEOR: f64 = 11.0;
const CYCLE: f64 = 24.0;
const DRAW: f64 = 6.0;
const HOLD: f64 = 12.0;
const FADE: f64 = 4.0;

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

fn ease(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

/// The sky: the seven's cells, the line dots of each edge (dot, how far along),
/// the field stars (cell, glyph, twinkle rate, tint), whether lines show, and the dots.
pub struct Saptarishi {
    cells: Vec<(i64, i64)>,
    edges: Vec<Vec<(i64, i64, f64)>>,
    field: Vec<(i64, i64, char, f64, u8)>,
    lines: bool,
    dots: ColorDots,
}

/// Saptarishi; reads `lines`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    let (cols, rows) = (META.cols as i64, META.rows as i64);
    let (cf, rf) = (cols as f64, rows as f64);
    // Facing north, east on the left: a tangent-plane projection in cells,
    // where a row is two columns tall.
    let (a0, d0) = (186f64.to_radians(), 55.5f64.to_radians());
    let plane: Vec<(f64, f64)> = SEVEN
        .iter()
        .map(|&(ra, dec, ..)| {
            let (a, d) = ((ra * 15.0).to_radians() - a0, dec.to_radians());
            let k = d0.sin() * d.sin() + d0.cos() * d.cos() * a.cos();
            (-d.cos() * a.sin() / k, (d0.sin() * d.cos() * a.cos() - d0.cos() * d.sin()) / k)
        })
        .collect();
    let span = |f: fn(&(f64, f64)) -> f64| plane.iter().map(f).fold((f64::MAX, f64::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
    let ((x_lo, x_hi), (y_lo, y_hi)) = (span(|p| p.0), span(|p| p.1));
    let s = (cf - 12.0) / (x_hi - x_lo);
    let cells: Vec<(i64, i64)> = plane
        .iter()
        .map(|&(x, y)| ((cf / 2.0 - 0.5 + s * (x - (x_hi + x_lo) / 2.0)).round() as i64, (rf / 2.0 - 0.5 + s / 2.0 * (y - (y_hi + y_lo) / 2.0)).round() as i64))
        .collect();
    let mut used = vec![false; (cols * rows) as usize];
    let keep = |used: &mut [bool], c: i64, r: i64, mc: i64, mr: i64| {
        for j in -mr..=mr {
            for i in -mc..=mc {
                if (0..rows).contains(&(r + j)) && (0..cols).contains(&(c + i)) {
                    used[((r + j) * cols + c + i) as usize] = true;
                }
            }
        }
    };
    for &(c, r) in &cells {
        keep(&mut used, c, r, 3, 1);
    }
    // Each line is a dotted stroke of braille from star to star, every other
    // dot along it, stopping short of the stars at either end.
    let edges = EDGES
        .iter()
        .map(|&(i, j)| {
            let ((c0, r0), (c1, r1)) = (cells[i], cells[j]);
            let (x0, y0, x1, y1) = ((c0 as f64 + 0.5) * 2.0, (r0 as f64 + 0.5) * 4.0, (c1 as f64 + 0.5) * 2.0, (r1 as f64 + 0.5) * 4.0);
            let n = ((x1 - x0).hypot(y1 - y0) * 2.0).ceil() as i64;
            let mut run: Vec<(i64, i64, f64)> = vec![];
            for q in 0..=n {
                let f = q as f64 / n as f64;
                let (x, y) = (x0 + (x1 - x0) * f, y0 + (y1 - y0) * f);
                let (dx, dy) = (x.floor() as i64, y.floor() as i64);
                keep(&mut used, dx / 2, dy / 4, 1, 1);
                // Clear of both stars by about a cell and a half across, a row up and down.
                let clear = [(x0, y0), (x1, y1)].iter().all(|&(sx, sy)| ((x - sx) / 5.0).powi(2) + ((y - sy) / 6.0).powi(2) > 1.0);
                if clear && run.last().is_none_or(|&(lx, ly, _)| (lx - dx).abs().max((ly - dy).abs()) >= 2) {
                    run.push((dx, dy, f));
                }
            }
            run
        })
        .collect();
    // Field stars: sparse, never touching, thinning toward the ends. Most are
    // a faint . and a few a ·, and each now and then dims out for a moment.
    let mut field = vec![];
    for r in 0..rows {
        for c in 0..cols {
            if used[(r * cols + c) as usize] || hash(c, r) >= 0.058 * ease(((c.min(cols - 1 - c)) as f64 / 9.0).clamp(0.0, 1.0)) {
                continue;
            }
            let ch = if hash(r, c + 99) > 0.85 { '·' } else { '.' };
            field.push((c, r, ch, 0.35 + 0.7 * hash(c, r + 7), FIELD[(hash(c + 31, r) * 3.0) as usize % 3]));
            keep(&mut used, c, r, 2, 1);
        }
    }
    Box::new(Saptarishi { cells, edges, field, lines: options.bool("lines", true), dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Saptarishi {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        grid.clear();
        for (i, &(c, r, ch, rate, tint)) in self.field.iter().enumerate() {
            let x = t * rate;
            let (k, e) = (x.floor() as i64, ease(x - x.floor()));
            let w = hash(k, i as i64 + 7) + (hash(k + 1, i as i64 + 7) - hash(k, i as i64 + 7)) * e;
            if w >= 0.2 {
                grid.set(c, r, ch, tint);
            }
        }
        self.dots.clear();
        // The lines draw in round the dipper, hold, fade, and are gone a moment
        // before they draw again. Frame 0 is in the hold.
        if self.lines {
            let p = (t + 8.0).rem_euclid(CYCLE);
            let drawn = ease(((p - 1.0) / DRAW).clamp(0.0, 1.0)) * EDGES.len() as f64;
            let fade = 1.0 - ((p - 1.0 - DRAW - HOLD) / FADE).clamp(0.0, 1.0);
            for (e, run) in self.edges.iter().enumerate() {
                let reach = (drawn - e as f64).clamp(0.0, 1.0);
                for &(x, y, f) in run {
                    if f <= reach && dither(x as usize, y as usize, fade) {
                        self.dots.set(x, y, LINE.at(fade));
                    }
                }
            }
        }
        // A meteor every so often: a head with a tail that thins out behind it.
        let m0 = ((t - 2.0 + 2.5) / METEOR).floor().max(0.0) as i64;
        for m in [m0 - 1, m0] {
            if m < 0 {
                continue;
            }
            let start = 2.0 + m as f64 * METEOR + if m > 0 { hash(m, 3) * 4.0 - 2.0 } else { 0.0 };
            let age = (t - start) / 0.8;
            if age <= 0.0 || age >= 1.45 {
                continue;
            }
            let dir = if hash(m, 5) < 0.5 { 1.0 } else { -1.0 };
            let (len, drop) = (18.0 + 8.0 * hash(m, 6), 3.0 + 3.0 * hash(m, 7));
            let x0 = if dir > 0.0 { 4.0 + hash(m, 8) * (cols - len - 8.0) } else { cols - 5.0 - hash(m, 8) * (cols - len - 8.0) };
            let y0 = 0.6 + hash(m, 9) * (rows - drop - 2.2);
            let (front, back) = (age.min(1.0), (age - 0.45).max(0.0));
            let n = 160;
            for q in (back * n as f64).ceil() as i64..=(front * n as f64) as i64 {
                let f = q as f64 / n as f64;
                let (x, y) = ((x0 + dir * len * f) * 2.0, (y0 + drop * f) * 4.0);
                let fresh = (f - back) / (front - back).max(1e-6);
                if dither(x as usize, y as usize, 0.25 + 0.75 * fresh) {
                    self.dots.set(x as i64, y as i64, if age < 1.0 && fresh > 0.97 { HEAD } else { TAIL.at(fresh) });
                }
            }
        }
        self.dots.draw(grid, 0, 0);
        // The seven over everything, and Alcor a cell up and to the east of Mizar.
        for (i, &(c, r)) in self.cells.iter().enumerate() {
            let (_, _, mag, color) = SEVEN[i];
            grid.set(c, r, if mag < 2.0 { '●' } else { '•' }, color);
        }
        let (mc, mr) = self.cells[5];
        grid.set(mc - 1, mr - 1, '·', A_STAR);
    }
}
