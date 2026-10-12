//! cherry blossom: a flowering cherry branch reaching in from the left. Each
//! gust shakes the blossoms on their twigs, and petals come loose and tumble
//! away on the wind. After ascii.rest's cherry-blossom.ts (MIT, @bas3line).
//!
//! The gusts, the blossoms' places and the petals' flight are ascii.rest's.
//! The wood is redrawn in braille, a limb that tapers smoothly from the left,
//! forks into a thin upper branch and a crooked hanging limb, and puts out
//! fine twigs, its bark dark with a lit upper edge. The blossoms are
//! ascii.rest's flowers with each petal a ● or • and each heart a ◉, in the
//! pale and deeper pinks of a somei-yoshino, and the loose petals tumble pink
//! on the wind, • to ° to · as they turn over.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The cherry blossom's meta.
pub static META: Meta = Meta {
    name: "cherry blossom",
    slug: "cherry-blossom",
    category: Category::Nature,
    note: "a braille cherry branch in pink blossom, petals tumbling on the wind",
    cols: 64,
    rows: 20,
    fps: 15,
    charset: Charset::Extended,
    // Bark and its lit edge; petals pale and deeper; the blossom's heart; a
    // loose petal, face and back.
    palette: Some(&["#5a3a30", "#9a6a56", "#ffd0de", "#ff9cc0", "#e03c78", "#ffb8d0", "#f088ac"]),
    ground: Some("#120e16"),
    ..Meta::DEFAULT
};

const BARK: u8 = 0;
const BARK_LIT: u8 = 1;
const PETAL: u8 = 2;
const SMALL: u8 = 3;
const HEART: u8 = 4;
const LOOSE: [u8; 2] = [5, 6];

/// Seconds a loop (two gusts), seconds between gusts and how long one lasts,
/// and the petals let go each loop.
const LOOP: f64 = 24.0;
const GUST: f64 = 12.0;
const BLOW: f64 = 4.5;
const PETALS: usize = 14;

/// The wood as polylines of (column, row, thickness in rows): the limb from
/// the left, its thin upper branch, the hanging limb, and the twigs.
const WOOD: [&[(f64, f64, f64)]; 8] = [
    &[(-1.0, 3.5, 2.0), (6.0, 4.0, 2.0), (15.0, 5.0, 1.9), (22.0, 6.0, 1.7), (29.0, 7.0, 1.4), (32.5, 7.9, 1.1)],
    &[(32.0, 7.6, 0.8), (39.0, 8.3, 0.6), (46.0, 9.0, 0.5), (54.0, 9.4, 0.4)],
    &[(32.5, 8.4, 1.0), (35.0, 10.2, 0.9), (35.8, 11.5, 0.8), (35.0, 12.6, 0.8), (34.0, 13.6, 0.7), (33.4, 14.5, 0.7), (34.4, 15.5, 0.6), (36.4, 16.6, 0.5)],
    &[(24.5, 5.8, 0.4), (24.5, 3.8, 0.3)],
    &[(36.5, 7.6, 0.4), (35.0, 5.2, 0.3)],
    &[(36.5, 7.6, 0.4), (39.5, 4.2, 0.3)],
    &[(54.0, 9.4, 0.4), (56.5, 6.8, 0.3)],
    &[(54.4, 9.6, 0.4), (56.5, 12.5, 0.3)],
];
/// Blossoms open, side on, small, cupped and partly shed, and buds: art,
/// row, column, how far a gust shakes it, and the petal it lets go (line,
/// column, length). In the art ( _ ) is a petal, a lone _ a small one and @
/// the heart.
type Flower = (&'static [&'static str], i64, i64, f64, Option<(usize, usize, usize)>);
const OPEN: &[&str] = &[" _(_)_", "(_)@(_)", " (_)(_)"];
const FLOWERS: [Flower; 9] = [
    (OPEN, 1, 21, 1.0, Some((2, 4, 3))),
    (&["    _", " _(_)", "(_)@", " (_)"], 1, 31, 1.0, Some((3, 1, 3))),
    (&[" _ _", "(_@_)"], 2, 37, 2.0, Some((0, 3, 1))),
    (&[" .-.", "(_@_)", " '|'"], 4, 54, 1.0, Some((0, 1, 3))),
    (&[" _(_)", "(_)@(_)", "    (_)"], 12, 56, 2.0, Some((0, 1, 4))),
    (OPEN, 16, 30, 0.0, Some((2, 1, 3))),
    (&[" _ _", "(_@_)", " (_)"], 15, 38, 0.0, Some((2, 1, 3))),
    (&["(_)(_)", " (@)"], 8, 11, 1.0, Some((0, 3, 3))),
    (&["o"], 19, 34, 1.0, None),
];
/// A petal turning over.
const TUMBLE: [char; 6] = ['•', '°', '·', '·', '·', '°'];

fn hash(a: i64, b: i64) -> f64 {
    let mut h = (a as u32).wrapping_mul(0x27d4_eb2d) ^ (b as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    f64::from(h ^ (h >> 16)) / 4_294_967_296.0
}

/// How hard a gust is blowing, 0 to 1.
fn gust(t: f64) -> f64 {
    let x = (t - 1.0).rem_euclid(GUST);
    if x < BLOW {
        (PI * x / BLOW).sin().powi(2)
    } else {
        0.0
    }
}

/// How far gusts have carried the air since time 0, in gust-seconds.
fn blown(t: f64) -> f64 {
    let n = ((t - 1.0) / GUST).floor();
    let x = t - 1.0 - n * GUST;
    n * BLOW * 0.5 + if x < BLOW { x / 2.0 - BLOW / (2.0 * PI) * (2.0 * PI * x / BLOW).sin() } else { BLOW / 2.0 }
}

/// A blossom's art as glyphs: line, column, glyph, colour.
fn glyphs(art: &[&str]) -> Vec<(usize, usize, char, u8)> {
    let mut out = vec![];
    for (i, line) in art.iter().enumerate() {
        let b: Vec<char> = line.chars().collect();
        for (j, &ch) in b.iter().enumerate() {
            let cupped = j > 0 && b[j - 1] == '(' && b.get(j + 1) == Some(&')');
            let g = match ch {
                '_' if cupped => Some(('●', PETAL)),
                '_' | '-' | 'o' => Some(('•', SMALL)),
                '@' => Some(('◉', HEART)),
                '.' => Some(('·', SMALL)),
                '|' => Some(('│', BARK_LIT)),
                _ => None,
            };
            if let Some((g, c)) = g {
                out.push((i, j, g, c));
            }
        }
    }
    out
}

/// A loose petal: when it lets go, its blossom, how fast it falls, how much
/// the wind takes it, how fast it turns, and its phase.
struct Petal {
    t0: f64,
    flower: usize,
    fall: f64,
    drag: f64,
    spin: f64,
    ph: f64,
}

/// The branch: its dots, each blossom's glyphs, the petals, and the dots.
pub struct CherryBlossom {
    wood: Vec<(i64, i64, u8)>,
    flowers: Vec<Vec<(usize, usize, char, u8)>>,
    petals: Vec<Petal>,
    dots: ColorDots,
}

/// A cherry branch; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    // Each limb as a band of dots between its edges, lit along its upper edge.
    let mut wood = vec![];
    for limb in WOOD {
        for seg in limb.windows(2) {
            let ((x0, y0, w0), (x1, y1, w1)) = (seg[0], seg[1]);
            let n = ((x1 - x0).abs() * 2.0).max((y1 - y0).abs() * 4.0).ceil() as i64 * 2;
            for s in 0..=n {
                let f = s as f64 / n as f64;
                let (x, y, w) = ((x0 + (x1 - x0) * f) * 2.0, (y0 + (y1 - y0) * f) * 4.0, (w0 + (w1 - w0) * f) * 4.0);
                let (top, bottom) = ((y - w / 2.0).round() as i64, (y + w / 2.0).round() as i64);
                for dy in top..=bottom.max(top) {
                    wood.push((x.floor() as i64, dy, if dy == top { BARK_LIT } else { BARK }));
                }
            }
        }
    }
    // Petals leave blossoms at their own moments, more of them in the gusts,
    // and ride the wind down and away. The high blossoms shed most.
    let from = [0, 1, 3, 2, 7, 0, 4, 1, 3, 7, 2, 0, 5, 6];
    let petals = (0..PETALS as i64)
        .map(|i| Petal {
            t0: if i % 2 == 1 { ((i / 2) % 2) as f64 * GUST + 1.2 + hash(i, 2) * 2.2 } else { hash(i, 1) * LOOP },
            flower: from[i as usize % from.len()],
            fall: 0.42 + hash(i, 5) * 0.22,
            drag: 0.8 + hash(i, 6) * 0.5,
            spin: 1.2 + hash(i, 8) * 0.8,
            ph: hash(i, 9) * 6.3,
        })
        .collect();
    let flowers = FLOWERS.iter().map(|f| glyphs(f.0)).collect();
    Box::new(CherryBlossom { wood, flowers, petals, dots: ColorDots::new(META.cols, META.rows) })
}

/// How far a blossom is shaken at time t: it trembles while a gust blows.
fn shake(k: usize, t: f64) -> i64 {
    let flex = FLOWERS[k].3;
    (flex * gust(t) * (0.8 + 0.35 * (t * 8.5 + k as f64 * 2.3).sin())).min(flex).round() as i64
}

impl Piece for CherryBlossom {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as i64, META.rows as i64);
        let u = t.rem_euclid(LOOP);
        self.dots.clear();
        for &(x, y, color) in &self.wood {
            self.dots.set(x, y, color);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        // Blossoms sit over the wood; one that has just let a petal go shows
        // the gap for a few seconds.
        let loose: Vec<usize> = self.petals.iter().filter(|p| (u - p.t0 + LOOP).rem_euclid(LOOP) < 3.0).map(|p| p.flower).collect();
        let mut taken = vec![false; (cols * rows) as usize];
        for (k, (art, (_, r0, c0, _, slot))) in self.flowers.iter().zip(FLOWERS.iter()).enumerate() {
            let dc = shake(k, t);
            for &(i, j, g, color) in art {
                let gap = loose.contains(&k) && slot.is_some_and(|(si, sj, sl)| i == si && j >= sj && j < sj + sl);
                let (r, c) = (r0 + i as i64, c0 + j as i64 + dc);
                if !gap && (0..rows).contains(&r) && (0..cols).contains(&c) {
                    grid.set(c, r, g, color);
                    taken[(r * cols + c) as usize] = true;
                }
            }
        }
        for p in &self.petals {
            let (_, r0, c0, _, slot) = FLOWERS[p.flower];
            let Some((si, sj, _)) = slot else { continue };
            for k in 0..2 {
                let age = (u - p.t0 + LOOP).rem_euclid(LOOP) + k as f64 * LOOP;
                let x = c0 as f64
                    + sj as f64
                    + 1.0
                    + shake(p.flower, t - age) as f64
                    + p.drag * (0.16 * age + 2.2 * (blown(t) - blown(t - age)))
                    + 1.1 * (age * 0.9 + p.ph).sin();
                let y = r0 as f64 + si as f64 + 0.5 + p.fall * age + 0.3 * (age * 1.9 + p.ph).sin();
                let (c, r) = (x.floor() as i64, y.floor() as i64);
                if !(0..rows).contains(&r) || !(0..cols).contains(&c) || taken[(r * cols + c) as usize] {
                    continue;
                }
                let turn = (age * p.spin + p.ph).floor() as usize;
                grid.set(c, r, TUMBLE[turn % TUMBLE.len()], LOOSE[turn % 2]);
            }
        }
    }
}
