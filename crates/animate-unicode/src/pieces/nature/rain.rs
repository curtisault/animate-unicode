//! rain: slanting rain in three depths over open ground. Each drop ends where
//! it meets the ground at its own depth, throwing up a crown on the ground or
//! opening a ring where it falls in a puddle. After ascii.rest's rain.ts
//! (MIT, @bas3line).
//!
//! The drops, their depths and where they land are ascii.rest's; it is drawn
//! in braille. A streak is a slanting line of dots, short and dim far off,
//! long and bright up close; a puddle is a sheet of fine level lines; a drop
//! that lands in one opens a true ring, an ellipse in the puddle's
//! perspective that widens and fades; one that lands on open ground throws
//! up a crown of spray that breaks and falls back either side. In colour the
//! rain pales with nearness, the water is a dark slate blue and its rings
//! catch the light.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::TAU;

/// Rain's meta.
pub static META: Meta = Meta {
    name: "rain",
    slug: "rain",
    category: Category::Nature,
    note: "braille rain at three depths, crowns on the ground, rings in puddles",
    cols: 64,
    rows: 20,
    fps: 30,
    charset: Charset::Extended,
    // Rain far, mid, near; spray; rings; the puddles; the horizon; grit.
    palette: Some(&["#465672", "#7a8cb2", "#c6d6f2", "#dce6ff", "#9cc2ea", "#22364e", "#3a4250", "#4c4a44"]),
    ground: Some("#080a10"),
    ..Meta::DEFAULT
};

const DEPTH: [u8; 3] = [0, 1, 2];
const SPRAY: u8 = 3;
const RING: u8 = 4;
const WATER: u8 = 5;
const HORIZON_LINE: u8 = 6;
const GRIT: u8 = 7;
/// The far edge of the ground, as a row.
const HORIZON: f64 = 11.0;
/// Each depth, far to near: drops, rows a second, streak length in rows, and
/// the rows it lands between.
const LAYERS: [(usize, f64, f64, f64, f64); 3] = [(46, 11.0, 1.0, 12.0, 13.9), (30, 19.0, 2.0, 14.0, 16.9), (16, 30.0, 4.0, 17.0, 19.9)];
/// Puddles, flat ovals in perspective: middle column and row, half width in
/// columns, half height in rows.
const PUDDLES: [(f64, f64, f64, f64); 3] = [(45.0, 13.5, 6.0, 0.7), (17.0, 15.5, 10.0, 1.6), (44.0, 18.5, 14.0, 1.6)];

fn hash(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b) ^ ((s + 1) as u32).wrapping_mul(0xc2b2_ae35);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    f64::from(h ^ (h >> 15)) / 4_294_967_296.0
}

/// The scene: which puddle each dot of ground is in, the fixed ground dots, and the dots.
pub struct Rain {
    pool: Vec<i8>,
    ground: Vec<(i64, i64, u8)>,
    dots: ColorDots,
}

/// Rain; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let (w, h) = (dots.width() as i64, dots.height() as i64);
    // Each puddle a sheet of still water with a ragged edge.
    let mut pool = vec![-1i8; (w * h) as usize];
    for (p, &(cx, cy, rx, ry)) in PUDDLES.iter().enumerate() {
        for j in ((cy - ry) * 4.0).floor() as i64..=((cy + ry) * 4.0) as i64 {
            let v = ((j as f64 + 0.5) / 4.0 - cy) / ry;
            if v.abs() >= 1.0 || j >= h {
                continue;
            }
            let half = rx * (1.0 - v * v).sqrt();
            let ragged = |s: i64| (hash(j / 2, p as i64 + s) - 0.5) * 2.5;
            let (a, b) = (((cx - half + ragged(20)) * 2.0).round() as i64, ((cx + half + ragged(30)) * 2.0).round() as i64);
            for i in a.max(0)..b.min(w) {
                pool[(j * w + i) as usize] = p as i8;
            }
        }
    }
    let mut ground = vec![];
    for i in 0..w {
        ground.push((i, (HORIZON * 4.0) as i64 + 3, HORIZON_LINE));
    }
    for j in (HORIZON * 4.0) as i64 + 4..h {
        for i in 0..w {
            let k = (j * w + i) as usize;
            if pool[k] >= 0 {
                // Still water: level lines every other dot row, the far side fainter.
                if j % 2 == 0 && (i + j / 2) % 3 != 0 {
                    ground.push((i, j, WATER));
                }
            } else {
                // Grit, growing coarser nearer.
                let near = (j as f64 / 4.0 - HORIZON) / (META.rows as f64 - HORIZON);
                if hash(j * w + i, 3) < 0.012 + 0.012 * near {
                    ground.push((i, j, GRIT));
                }
            }
        }
    }
    Box::new(Rain { pool, ground, dots })
}

impl Piece for Rain {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let cols = META.cols as f64;
        let pool = |x: i64, y: i64| if (0..w).contains(&x) && (0..h).contains(&y) { self.pool[(y * w + x) as usize] } else { -2 };
        self.dots.clear();
        for &(x, y, c) in &self.ground {
            self.dots.set(x, y, c);
        }
        let mut splashes = vec![];
        for (li, &(n, v, len, from, to)) in LAYERS.iter().enumerate() {
            let cycle = (to + len) / v + 0.6;
            for i in 0..n as i64 {
                let tt = t + hash(i, li as i64 * 3) * cycle;
                let k = (tt / cycle).floor();
                let u = tt - k * cycle;
                let id = i * 977 + k as i64 * 13 + li as i64;
                // Where this drop lands: its depth sets the row.
                let land = (from + (to - from) * hash(id, 1)).floor() + 0.75;
                let lx = (hash(id, 2) * (cols + 2.0)).floor() - 1.0 + 0.5;
                let head = -len + v * u;
                if head < land {
                    // The streak runs up and to the right of its head, a column a
                    // row, and passes behind the horizon and the water.
                    for d in 0..(len * 4.0) as i64 {
                        let r = head - d as f64 / 4.0;
                        let (x, y) = (((lx + (land - r)) * 2.0).floor() as i64, (r * 4.0).floor() as i64);
                        if r >= 0.0 && r < land && (y - (HORIZON * 4.0) as i64 - 3).abs() > 0 && pool(x, y) < 0 {
                            self.dots.set(x, y, DEPTH[li]);
                        }
                    }
                } else {
                    splashes.push((lx, land, (head - land) / v, li));
                }
            }
        }
        // Spray lands on open ground only, never in the water or on the horizon.
        let dry = |x: i64, y: i64| y > (HORIZON * 4.0) as i64 + 3 && pool(x, y) == -1;
        for (x, y, age, li) in splashes {
            let (cx, cy) = ((x * 2.0) as i64, (y * 4.0) as i64);
            let p = pool(cx, cy);
            if p >= 0 {
                // A ring on the water: a bead, then an ellipse in the puddle's
                // perspective that widens and fades.
                let life = [0.35, 0.6, 0.85][li];
                if age > life {
                    continue;
                }
                let f = age / life;
                let rx = (0.5 + f * (2.0 + 3.0 * li as f64)) * 2.0;
                let (_, _, prx, pry) = PUDDLES[p as usize];
                let ry = rx * pry / prx * 2.0;
                let steps = (rx * 4.0).max(6.0) as i64;
                for s in 0..steps {
                    let a = TAU * s as f64 / steps as f64;
                    let (ex, ey) = ((cx as f64 + rx * a.cos()).floor() as i64, (cy as f64 + ry * a.sin()).floor() as i64);
                    if pool(ex, ey) == p && dither(ex as usize, ey as usize, 1.1 - f) {
                        self.dots.set(ex, ey, RING);
                    }
                }
                continue;
            }
            // On the ground: far drops just bead; nearer ones throw up a crown
            // that breaks into drops falling back either side.
            let mut spray = |dx: i64, dy: i64| {
                if dry(cx + dx, cy + dy) {
                    self.dots.set(cx + dx, cy + dy, SPRAY);
                }
            };
            match li {
                0 if age < 0.1 => spray(0, 0),
                1 if age < 0.08 => {
                    for (dx, dy) in [(-1, -1), (1, -1), (0, 0)] {
                        spray(dx, dy);
                    }
                }
                1 if age < 0.22 => {
                    spray(-2, 0);
                    spray(2, 0);
                }
                2 if age < 0.09 => {
                    for (dx, dy) in [(-2, -3), (-1, -2), (0, -3), (0, -2), (1, -2), (2, -3), (0, 0), (-1, 0), (1, 0)] {
                        spray(dx, dy);
                    }
                }
                2 if age < 0.2 => {
                    for (dx, dy) in [(-3, -4), (3, -4), (-4, -3), (4, -3)] {
                        spray(dx, dy);
                    }
                }
                2 if age < 0.32 => {
                    for (dx, dy) in [(-4, 0), (4, 0), (-5, -1), (5, -1)] {
                        spray(dx, dy);
                    }
                }
                _ => {}
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
