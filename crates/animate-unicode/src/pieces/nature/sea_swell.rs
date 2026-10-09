//! sea swell: a long ocean swell seen side on, a rounded crest over a long
//! flat trough. A sailboat rides it, pitching with the slope under its hull,
//! while the crests hide and show the far horizon. After ascii.rest's
//! sea-swell.ts (MIT, @bas3line).
//!
//! The swell, its flecks and the boat's ride are ascii.rest's. It is drawn in
//! octants: the sea's line is a curve at a dot's resolution, and the boat is
//! drawn dot by dot, a red hull under a white sail cut on the diagonal, so it
//! pitches smoothly to any angle rather than among three poses. In colour the
//! sea is blue, darkening with depth, white where the crest steepens, under a
//! pale sun low over the far sea.

use crate::util::{dither, Mulberry32};
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// The sea swell's meta.
pub static META: Meta = Meta {
    name: "sea swell",
    slug: "sea-swell",
    category: Category::Nature,
    note: "an octant swell lifting a sailboat, blue water, a low pale sun",
    cols: 64,
    rows: 14,
    fps: 20,
    charset: Charset::Extended,
    // The sea's line, its foam, flecks near and deep, the far sea; the sail,
    // the hull, the mast; the sun.
    palette: Some(&["#4aa6dc", "#e6f4ff", "#2e78b4", "#1c4c7c", "#3a6a92", "#f6f4ec", "#c8402c", "#8a6a4a", "#ffe2a4"]),
    ground: Some("#0a1220"),
    ..Meta::DEFAULT
};

const SURFACE: u8 = 0;
const FOAM: u8 = 1;
const FLECK: u8 = 2;
const DEEP: u8 = 3;
const FAR: u8 = 4;
const SAIL: u8 = 5;
const HULL_RED: u8 = 6;
const MAST: u8 = 7;
const SUN: u8 = 8;
/// The far sea's row, and the still water line in rows from the top.
const HORIZON: f64 = 4.0;
const MEAN: f64 = 6.1;
/// The hull's length in columns, and the column of the boat's left end.
const HULL: f64 = 10.0;
const BOAT_X: f64 = 18.0;
/// Each wave: length in columns, waves a second, trough to crest in rows,
/// crest sharpness. The second runs back against the first, so the crests
/// rise and fall in height; both come back round every 16 seconds.
const WAVES: [[f64; 4]; 2] = [[64.0, 0.0625, 3.8, 0.5], [32.0, -0.0625, 0.9, 0.3]];
/// How far the swell's crest runs ahead of its trough, in columns; seconds
/// into the loop where t = 0 falls; and the swell's wavenumber in rows, so
/// the motion dies away with depth.
const LEAN: f64 = 2.0;
const START: f64 = 2.5;
const K: f64 = TAU / 32.0;

/// One trochoid, -0.5 in the trough to 0.5 on the crest.
fn trochoid(u: f64, q: f64) -> f64 {
    let mut v = u;
    for _ in 0..4 {
        v = u + q / TAU * (TAU * v).sin();
    }
    0.5 * (TAU * v).cos()
}

/// The surface in rows from the top at column x, the swell's crest sheared
/// forward so its front face is the steeper one.
fn surface(x: f64, t: f64) -> f64 {
    let [l, v, a, q] = WAVES[0];
    let mut h = trochoid(x / l - t * v, q);
    for _ in 0..3 {
        h = trochoid((x - LEAN * (h + 0.5)) / l - t * v, q);
    }
    let [l2, v2, a2, q2] = WAVES[1];
    MEAN - a * h - a2 * trochoid(x / l2 - t * v2, q2)
}

/// The sea: the flecks (column, depth, run), the far sea's flecks (row, column, length), and the dots.
pub struct SeaSwell {
    flecks: Vec<(f64, f64, f64)>,
    far: Vec<(f64, f64, f64)>,
    dots: ColorOctants,
}

/// A sea swell; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let cols = META.cols as f64;
    let mut rand = Mulberry32::new(21);
    // Ripples in the water: short runs near the top, fewer and finer the
    // deeper they lie. A few more lie on the far sea under the horizon.
    let mut flecks = vec![];
    let mut d = 1.5;
    while d < 10.0 {
        let (gap, run) = (
            3.0 + d * 2.0,
            if d < 3.0 {
                3.0
            } else if d < 5.5 {
                2.0
            } else {
                1.0
            },
        );
        let from = (rand.next_f64() * cols).floor();
        let mut i = 0.0;
        while i < cols - run {
            let n = 1.0 + (rand.next_f64() * run).floor();
            flecks.push(((from + i).rem_euclid(cols), d, n));
            i += n + (gap * (0.6 + rand.next_f64())).floor();
        }
        d += 1.0;
    }
    let mut far = vec![];
    for r in 1..4 {
        let mut c = (rand.next_f64() * 12.0).floor();
        while c < cols {
            far.push((HORIZON + f64::from(r), c, if rand.next_f64() < 0.5 { 1.0 } else { 2.0 }));
            c += 11.0 + (rand.next_f64() * 14.0).floor();
        }
    }
    Box::new(SeaSwell { flecks, far, dots: ColorOctants::new(META.cols, META.rows) })
}

/// Whether a point of the boat, in columns from its left end and rows above
/// the water (a row is two columns), is part of it, and what.
fn boat(u: f64, v: f64) -> Option<u8> {
    // The hull: a deck the boat's length, a keel narrower.
    if (-0.35..0.55).contains(&v) {
        let inset = (0.55 - v) / 0.9 * 1.4;
        return (u > inset && u < HULL - inset).then_some(HULL_RED);
    }
    // The mast, and the sail beside it, cut on the diagonal from its head to the boom.
    if (3.3..3.7).contains(&u) && (0.55..4.3).contains(&v) {
        return Some(MAST);
    }
    let up = (v - 0.85) / 3.2;
    ((0.0..1.0).contains(&up) && u >= 3.85 && u < 3.85 + 3.0 * (1.0 - up)).then_some(SAIL)
}

impl Piece for SeaSwell {
    fn frame(&mut self, time: f64, _: &Env, grid: &mut Grid) {
        let t = time + START;
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let sy: Vec<f64> = (0..=w).map(|x| surface(x as f64 / 2.0, t)).collect();
        self.dots.clear();
        // The sun, low over the far sea, and its far sea only over the near
        // trough, stopping short of it.
        for j in 0..(HORIZON * 4.0) as i64 {
            for i in 98..112 {
                let (dx, dy) = (i as f64 + 0.5 - 105.0, j as f64 + 0.5 - (HORIZON * 4.0 - 1.0));
                if dx.hypot(dy) < 5.0 {
                    self.dots.set(i, j, SUN);
                }
            }
        }
        let clear = |x: i64| HORIZON < sy[x.clamp(0, w) as usize] - 0.9;
        let hy = (HORIZON * 4.0) as i64 + 3;
        for x in 0..w {
            if clear(x - 2) && clear(x) && clear(x + 2) {
                self.dots.set(x, hy, FAR);
            }
        }
        for &(r, c, len) in &self.far {
            for k in 0..(len * 2.0) as i64 {
                let x = (c * 2.0) as i64 + k;
                if x < w && clear(x) && r < sy[x as usize] - 1.0 {
                    self.dots.set(x, (r * 4.0) as i64 + 2, FAR);
                }
            }
        }
        // The near water: flecks ride the swell, less the deeper they lie.
        for &(c, d, run) in &self.flecks {
            for k in 0..(run * 2.0) as i64 {
                let x = (c * 2.0) as i64 + k;
                if x >= w {
                    continue;
                }
                let y = sy[x as usize];
                let r = MEAN + (y - MEAN) * (-K * d).exp() + d;
                if r > y + 0.4 && r < META.rows as f64 {
                    self.dots.set(x, (r * 4.0) as i64, if d < 3.0 { FLECK } else { DEEP });
                }
            }
        }
        // The sea's line, white where the face steepens toward the crest.
        for x in 0..w {
            let (a, b) = (sy[x as usize] * 4.0, sy[x as usize + 1] * 4.0);
            let steep = (b - a).abs() > 1.2;
            // Two dots thick, so the near sea's line reads over the far one.
            for y in a.min(b).round() as i64..=a.max(b).round() as i64 + 1 {
                self.dots.set(x, y, if steep && dither(x as usize, y as usize, 0.7) { FOAM } else { SURFACE });
            }
        }
        // The boat surges a little with the water, forward on the crests and
        // back in the troughs, and pitches to the slope along its hull.
        let [l, v, ..] = WAVES[0];
        let c0 = BOAT_X - 1.5 * (TAU * ((BOAT_X + HULL / 2.0) / l - t * v)).sin();
        let (y0, y1) = (surface(c0 + 1.0, t), surface(c0 + HULL - 1.0, t));
        let mid = surface(c0 + HULL / 2.0, t).min((y0 + y1) / 2.0);
        // A row is two columns, so the slope in the picture is twice the rows over the columns.
        let angle = (2.0 * (y0 - y1) / (HULL - 2.0)).atan() * 0.8;
        let (sa, ca) = angle.sin_cos();
        let (px, py) = (c0 + HULL / 2.0, mid);
        for j in ((py - 5.0) * 4.0) as i64..((py + 1.0) * 4.0) as i64 {
            for i in ((px - 7.0) * 2.0) as i64..((px + 7.0) * 2.0) as i64 {
                if i < 0 || i >= w || j < 0 || j >= h {
                    continue;
                }
                // Into the boat's own frame, in columns either way.
                let (dx, dy) = ((i as f64 + 0.5) / 2.0 - px, -((j as f64 + 0.5) / 4.0 - py) * 2.0);
                let (u, vv) = (dx * ca + dy * sa + HULL / 2.0, (-dx * sa + dy * ca) / 2.0);
                if let Some(color) = boat(u, vv) {
                    self.dots.set(i, j, color);
                } else if (-0.6..0.9).contains(&vv) && u > -0.5 && u < HULL + 0.5 {
                    // A dot clear round the hull, so the sea's line stops short of it.
                    self.dots.unset(i, j);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
