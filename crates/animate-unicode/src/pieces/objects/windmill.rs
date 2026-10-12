//! windmill: a Dutch smock mill on a low rise, its four lattice sails
//! turning in front of the cap, a stage round its waist and clouds drifting
//! behind. After ascii.rest's windmill.ts (MIT, @bas3line).
//!
//! The mill, its sails and the clouds are ascii.rest's. The sails and the
//! clouds are braille: each sail a stock, an edge and a ladder of laths a
//! dot thick that turn smoothly at any angle and leave the sky showing
//! through, the clouds soft heaps of dots lit along their tops. The mill is
//! octants, its three boarded faces lit from the left, a thatched cap, a
//! stage round its waist and a door and windows. In colour the mill is
//! painted green, the cap thatch brown, the stage rail white, the sails'
//! laths cream on dark stocks, and the clouds white over a green rise.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, TAU};

/// The windmill's meta.
pub static META: Meta = Meta {
    name: "windmill",
    slug: "windmill",
    category: Category::Objects,
    note: "a green octant smock mill turning braille lattice sails, white clouds",
    cols: 64,
    rows: 28,
    fps: 20,
    charset: Charset::Extended,
    // The boards, shadow to light; the door and windows; the cap; the stage;
    // the sails' stocks and laths; clouds; grass.
    palette: Some(&["#244230", "#3a6448", "#5a8a64", "#141a16", "#a07a44", "#eceef2", "#5a3a26", "#e2d4b2", "#d8dce6", "#4a7a3a"]),
    ground: Some("#0e1420"),
    ..Meta::DEFAULT
};

const BOARDS: Ramp = Ramp::new(0, 3);
const DOOR: u8 = 3;
const CAP: u8 = 4;
const STAGE_RAIL: u8 = 5;
const STOCK: u8 = 6;
const LATH: u8 = 7;
const CLOUD: u8 = 8;
const GRASS: u8 = 9;
/// The hub (column, row); a sail's reach from it in rows; the lattice's
/// width; between the laths; where the lattice starts along the stock;
/// seconds a turn; the body's rows; its half width at the top and the foot,
/// in rows; and the stage's row.
const HX: f64 = 32.0;
const HY: f64 = 11.5;
const SAIL: f64 = 10.8;
const WIDTH: f64 = 3.0;
const LATHS: f64 = 2.25;
const FOOT: f64 = 1.8;
const PERIOD: f64 = 9.0;
const TOP: f64 = 12.0;
const GROUND: f64 = 24.5;
const WT: f64 = 3.6;
const WB: f64 = 5.6;
const STAGE: f64 = 18.0;
/// A cloud: puffs (x, y, radius) about its origin, its row, its speed in
/// columns a second, and where it starts.
type Cloud = (&'static [(f64, f64, f64)], f64, f64, f64);
const CLOUDS: [Cloud; 3] = [
    (&[(0.0, 0.0, 1.6), (2.2, -0.9, 2.3), (4.7, -0.4, 2.0), (6.7, 0.2, 1.4)], 4.4, 1.1, 26.0),
    (&[(0.0, 0.0, 1.3), (1.8, -0.6, 1.7), (3.5, 0.0, 1.3)], 2.4, 0.7, 64.0),
    (&[(0.0, 0.0, 1.4), (2.0, -0.8, 2.0), (4.1, -0.5, 1.7), (5.8, 0.2, 1.2)], 17.4, 1.5, 84.0),
];

fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

/// The windmill: the mill's still dots, and the braille and octant canvases.
pub struct Windmill {
    mill: Vec<(i64, i64, u8)>,
    air: ColorDots,
    body: ColorOctants,
}

/// A windmill; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let body = ColorOctants::new(META.cols, META.rows);
    let (w, h) = (body.width() as i64, body.height() as i64);
    let mut mill = vec![];
    for j in 0..h {
        for i in 0..w {
            let (x, y) = (((i as f64 + 0.5) / 2.0 - HX) / 2.0, (j as f64 + 0.5) / 4.0);
            // The land: a level line, and grass thinning toward us.
            if y >= GROUND {
                let d = y - GROUND + 0.5;
                if (y - GROUND) < 0.25 || hash(i, j) < 0.18 / d {
                    mill.push((i, j, GRASS));
                }
                continue;
            }
            // The body: three faces of an octagon in boards, lit from the left,
            // with a door at its foot and a window above each stage.
            if y > TOP {
                let wd = WT + (WB - WT) * (y - TOP) / (GROUND - TOP);
                if x.abs() < wd {
                    let u = x / wd;
                    let door = (x.abs() < 0.8 && y > GROUND - 2.2) || ((x + 0.2).abs() < 0.4 && ((y - TOP - 2.5).abs() < 0.5 || (y - STAGE - 2.5).abs() < 0.5));
                    if door {
                        mill.push((i, j, DOOR));
                        continue;
                    }
                    let v = if u < -0.4 {
                        0.8
                    } else if u < 0.4 {
                        0.55
                    } else {
                        0.3
                    };
                    // Boards: a dark seam every row.
                    if j % 4 != 0 && dither(i as usize, j as usize, 0.35 + 0.65 * v) {
                        mill.push((i, j, BOARDS.at(v)));
                    }
                    continue;
                }
            }
            // The cap: a rounded hood of thatch over the top, lit from the upper left.
            let q = (x / (WT + 0.6)).hypot((y - TOP - 0.2) / 3.8);
            if y <= TOP + 0.5 && q < 1.0 {
                let (nx, ny) = (x / (WT + 0.6), (TOP + 0.2 - y) / 3.8);
                let lit = (-0.5 * nx + 0.5 * ny + 0.7 * (1.0 - q * q).max(0.0).sqrt()).max(0.0);
                if dither(i as usize, j as usize, 0.3 + 0.75 * lit) {
                    mill.push((i, j, CAP));
                }
            }
        }
    }
    // The stage: a deck on struts, a rail along it.
    let sw = WT + (WB - WT) * (STAGE - TOP) / (GROUND - TOP) + 1.8;
    for i in ((HX - 2.0 * sw) * 2.0) as i64..=((HX + 2.0 * sw) * 2.0) as i64 {
        mill.push((i, (STAGE * 4.0) as i64 + 1, STAGE_RAIL));
        mill.push((i, (STAGE * 4.0) as i64 + 2, STAGE_RAIL));
        mill.push((i, (STAGE * 4.0) as i64 - 3, STAGE_RAIL));
        if (i / 2) % 2 == 0 {
            for j in (STAGE * 4.0) as i64 - 3..(STAGE * 4.0) as i64 + 1 {
                mill.push((i, j, STAGE_RAIL));
            }
        }
    }
    Box::new(Windmill { mill, air: ColorDots::new(META.cols, META.rows), body })
}

impl Piece for Windmill {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let cols = META.cols as f64;
        let (w, h) = (self.air.width() as i64, self.air.height() as i64);
        self.air.clear();
        // The clouds: heaps of round puffs lit from above with flat undersides,
        // dense along their tops and thinning to a rim.
        for &(puffs, row, speed, start) in &CLOUDS {
            let span = cols + 36.0;
            let ox = (start + speed * t).rem_euclid(span) - 22.0;
            for j in ((row - 4.0) * 4.0).max(0.0) as i64..((row + 1.0) * 4.0) as i64 {
                for i in ((ox - 6.0) * 2.0).max(0.0) as i64..(((ox + 22.0) * 2.0) as i64).min(w) {
                    let (px, py) = (((i as f64 + 0.5) / 2.0 - ox) / 2.0, (j as f64 + 0.5) / 4.0 - row);
                    if py > 0.8 {
                        continue;
                    }
                    let lit = puffs
                        .iter()
                        .filter_map(|&(a, b, rad)| {
                            let (nx, ny) = ((px - a) / rad, (py - b) / rad);
                            let q = nx * nx + ny * ny;
                            (q < 1.0).then(|| -0.3 * nx - 0.8 * ny + 0.5 * (1.0 - q).sqrt())
                        })
                        .fold(-1.0f64, f64::max);
                    if lit > -0.5 && dither(i as usize, j as usize, 0.15 + 0.45 * lit) {
                        self.air.set(i, j, CLOUD);
                    }
                }
            }
        }
        self.body.clear();
        for &(x, y, c) in &self.mill {
            self.body.set(x, y, c);
        }
        // The sails: for each, a stock from the hub, an edge, and a ladder of
        // laths across the lattice between them, the sky showing through.
        let mut sails = ColorDots::new(META.cols, META.rows);
        let turn = TAU * t / PERIOD; // square to the frame at the start
        for s in 0..4 {
            let a = turn + f64::from(s) * FRAC_PI_2;
            let (ay, ax) = a.sin_cos();
            for j in 0..h {
                for i in 0..w {
                    let (x, y) = (((i as f64 + 0.5) / 2.0 - HX) / 2.0, HY - (j as f64 + 0.5) / 4.0);
                    let (u, v) = (x * ax + y * ay, x * ay - y * ax); // along the stock, and across to the lattice
                    if !(0.6..=SAIL + 0.3).contains(&u) || !(-0.2..=WIDTH + 0.2).contains(&v) {
                        continue;
                    }
                    if v.abs() < 0.17 {
                        sails.set(i, j, STOCK);
                    } else if u > FOOT && (v - WIDTH).abs() < 0.13 {
                        sails.set(i, j, LATH);
                    } else if u > FOOT && v > 0.0 && v < WIDTH {
                        let off = (u - FOOT + LATHS / 2.0).rem_euclid(LATHS) - LATHS / 2.0;
                        if off.abs() < 0.11 || (v - WIDTH / 2.0).abs() < 0.09 {
                            sails.set(i, j, LATH);
                        }
                    }
                }
            }
        }
        grid.clear();
        self.air.draw(grid, 0, 0);
        self.body.draw(grid, 0, 0);
        sails.draw(grid, 0, 0);
        grid.text_at(HX as i64 - 1, HY as i64, "(●)", STOCK);
    }
}
