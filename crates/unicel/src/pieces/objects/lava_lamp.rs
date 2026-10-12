//! lava lamp: a lava lamp on its base. Wax pools on the floor of the glass,
//! blobs swell off it, climb, meet and part, and sink again, lit from the
//! bulb underneath. After ascii.rest's lava-lamp.ts (MIT, @bas3line).
//!
//! The blobs, the pool and the light are ascii.rest's, the wax worked out at
//! every octant dot, so the blobs are round and smooth and neck as they part.
//! The glass's sides are fine lines of dots and the cap and base are solid.
//! In colour the wax glows from a deep magenta-red at the top of the glass
//! to orange near the bulb, outlined dark where it meets the clear oil; the
//! metal is silver, and the bulb throws a soft warm glow round the base.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// The lava lamp's meta.
pub static META: Meta = Meta {
    name: "lava lamp",
    slug: "lava-lamp",
    category: Category::Objects,
    note: "octant wax blobs rising and merging, magenta to orange, a warm glow",
    cols: 30,
    rows: 27,
    fps: 15,
    charset: Charset::Extended,
    // The wax, cool to hot; its edge; the glass; the metal, dark and lit; the glow.
    palette: Some(&["#a01e5a", "#e0305a", "#ff5a3a", "#ff9a32", "#ffd06a", "#5a1430", "#6a8aa8", "#4a4e58", "#b4b8c4", "#7a3a20"]),
    ground: Some("#0b0a0e"),
    ..Meta::DEFAULT
};

const WAX: Ramp = Ramp::new(0, 5);
const EDGE: u8 = 5;
const GLASS: u8 = 6;
const METAL: u8 = 7;
const SHINE: u8 = 8;
const GLOW: u8 = 9;
/// A cell in em (the wax is measured in em); seconds in at frame 0 (one
/// blob near the cap, one climbing, one leaving the pool); the field's
/// threshold; the highest a blob's centre goes and the glass's floor, in em.
const CW: f64 = 0.6;
const CH: f64 = 1.2;
const T0: f64 = 9.0;
const THRESH: f64 = 0.3;
const TOP: f64 = 8.6;
const FLOOR: f64 = 24.4;
/// Blobs: seconds per trip, where in the trip, side of the middle, sway, reach.
const BLOBS: [(f64, f64, f64, f64, f64); 3] = [(24.0, 0.42, -0.5, 0.7, 3.0), (27.0, 0.08, 0.6, 0.6, 2.7), (31.0, 0.74, 0.1, 0.8, 3.3)];
/// The rows of the glass, its floor row last.
const GLASS_ROWS: (f64, f64) = (4.0, 21.0);

/// The lamp's half width at a row, in columns: cap, glass, waist, base.
fn half(r: f64) -> f64 {
    if r < 1.0 {
        1.9
    } else if r < 4.0 {
        0.9 + r
    } else if r < 20.0 {
        4.1 + 0.26 * (r - 4.0)
    } else if r < 21.0 {
        7.6
    } else {
        6.8 + (r - 21.0)
    }
}

fn fall(q: f64) -> f64 {
    if q < 1.0 {
        (1.0 - q) * (1.0 - q)
    } else {
        0.0
    }
}

/// A fixed number in [0, 1) for each dot, for a scatter with no pattern to it.
fn scatter(i: i64, j: i64) -> f64 {
    let mut h = (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    f64::from(h ^ (h >> 13)) / 4_294_967_296.0
}

fn smooth(e: f64) -> f64 {
    let e = e.clamp(0.0, 1.0);
    e * e * (3.0 - 2.0 * e)
}

/// The inside of the glass, in em from the middle, at a row.
fn room(r: f64) -> f64 {
    (half(r.clamp(GLASS_ROWS.0, GLASS_ROWS.1 - 1.0)) - 1.1) * CW
}

/// The lamp's dots.
pub struct LavaLamp {
    dots: ColorOctants,
}

/// A lava lamp; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(LavaLamp { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for LavaLamp {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let mid = cols / 2.0;
        let (w, h) = (self.dots.width() as i64, self.dots.height() as i64);
        let m = (0.5f64 * 0.5 + 0.85 * 0.85).sqrt();
        let (ly, lz) = (0.5 / m, 0.85 / m); // toward the light: below, and in front
        let tt = T0 + t;
        // Each blob: drawn out by the climb, round again at the turns, kept
        // clear of the glass and smaller where the glass is narrow.
        let mut balls: Vec<(f64, f64, f64, f64, f64)> = BLOBS
            .iter()
            .map(|&(period, phase, side, sway, rad)| {
                let a = TAU * (tt / period + phase);
                let y = TOP + (FLOOR - TOP) * (1.0 + a.cos()) / 2.0;
                let stretch = 1.0 + 0.3 * a.sin().abs();
                let (sx, sy) = (stretch, 1.0 / (stretch * stretch));
                let fit = room(((y - 0.67 * rad / sy.sqrt()) / CH).floor());
                let r = rad.min(fit * sx.sqrt() / 0.85);
                let reach = 0.8 * r / sx.sqrt();
                let x = side + sway * (a * 0.5 + phase * 9.0).sin();
                (x.clamp(reach - fit, fit - reach), y, r * r, sx, sy)
            })
            .collect();
        // The pool: three low lumps either side of the middle, shifting their weight.
        for j in -1..=1 {
            let jf = f64::from(j);
            balls.push((jf * 2.5 + 0.35 * (tt * 0.31 + jf * 2.1).sin(), FLOOR + 0.6 + 0.3 * (tt * 0.43 + jf * 1.3).sin(), 2.6 * 2.6, 0.3, 1.0));
        }
        let field = |x: f64, y: f64| {
            let f: f64 = balls.iter().map(|&(bx, by, r2, sx, sy)| fall(((x - bx).powi(2) * sx + (y - by).powi(2) * sy) / r2)).sum();
            // A meniscus: the pool climbs a little up the glass at either side.
            let edge = (x.abs() / room(GLASS_ROWS.1 - 1.0)).min(1.0);
            f + 0.55 * smooth((y - FLOOR + 0.9 + 2.0 * edge.powi(4)) / 1.1)
        };
        self.dots.clear();
        // The glow round the base, from the bulb inside it: a thin random scatter of warm dots.
        for j in ((GLASS_ROWS.1 - 3.0) * 4.0) as i64..h {
            for i in 0..w {
                let (x, y) = ((i as f64 + 0.5) / 2.0 - mid, (j as f64 + 0.5) / 4.0);
                let d = (x / 2.0 / 8.5).hypot((y - 22.5) / 4.5);
                if d < 1.0 && x.abs() > half(y) + 0.5 && scatter(i, j) < 0.2 * (1.0 - d) {
                    self.dots.set(i, j, GLOW);
                }
            }
        }
        let inside_glass = |y: f64| y >= GLASS_ROWS.0 && y < GLASS_ROWS.1;
        let is_wax = |i: i64, j: i64| {
            let (x, y) = (((i as f64 + 0.5) / 2.0 - mid) * CW, (j as f64 + 0.5) / 4.0);
            inside_glass(y) && x.abs() < room(y) && field(x, y * CH) >= THRESH
        };
        for j in 0..h {
            let y = (j as f64 + 0.5) / 4.0;
            let hw = half(y);
            for i in 0..w {
                let x = (i as f64 + 0.5) / 2.0 - mid;
                // The glass's sides, the cap and the base.
                if x.abs() > hw - 0.5 && x.abs() < hw {
                    self.dots.set(
                        i,
                        j,
                        if inside_glass(y) {
                            GLASS
                        } else if x < 0.0 {
                            SHINE
                        } else {
                            METAL
                        },
                    );
                    continue;
                }
                if !inside_glass(y) && y < rows && x.abs() < hw {
                    if y < GLASS_ROWS.0 || (y > GLASS_ROWS.1 && dither(i as usize, j as usize, 0.75)) {
                        self.dots.set(i, j, if x < -hw * 0.4 { SHINE } else { METAL });
                    }
                    continue;
                }
                if !is_wax(i, j) {
                    continue;
                }
                // The wax as a dome over its outline, its slope from the field's,
                // outlined where it meets the clear oil.
                if !is_wax(i - 1, j) || !is_wax(i + 1, j) || !is_wax(i, j - 1) || !is_wax(i, j + 1) {
                    self.dots.set(i, j, EDGE);
                    continue;
                }
                let (ex, ey) = (x * CW, y * CH);
                let f = field(ex, ey);
                let gx = (field(ex + 0.3, ey) - field(ex - 0.3, ey)) / 0.6;
                let gy = (field(ex, ey + 0.3) - field(ex, ey - 0.3)) / 0.6;
                let k = 0.8 / (f - THRESH + 0.04).sqrt();
                let (nx, ny) = (-gx * k, -gy * k);
                let lit = ((ny * ly + lz) / (nx * nx + ny * ny + 1.0).sqrt()).max(0.0);
                let warm = smooth((ey - TOP) / (FLOOR - TOP)); // hotter nearer the bulb
                let v = (0.05 + 0.75 * lit + 0.2 * warm).min(1.0);
                let ink = if env.paper { 1.05 - v } else { 0.3 + 0.7 * v };
                if dither(i as usize, j as usize, ink) {
                    self.dots.set(i, j, WAX.at(0.6 * warm + 0.4 * lit));
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
