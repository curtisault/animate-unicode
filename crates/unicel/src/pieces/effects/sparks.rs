//! sparks: an angle grinder biting into a steel plate. Sparks leave the cut
//! in a tight hot cone, arc under gravity, skip on the plate and cool. After
//! ascii.rest's sparks.ts (MIT, @bas3line).
//!
//! The grinder, the sparks' flight and their skips are ascii.rest's. The
//! sparks are braille, each a dot with a short streak back along its path
//! while it is hot; the grinder and the plate are octants, sampled per dot.
//! In colour the sparks cool as they fly, white at the bite, then yellow,
//! orange and red to a dull ember on the plate; the disc is dark with a
//! silver rim and a yellow label turning, the guard grey, the handle black
//! and the motor teal, over a steel plate.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, ColorOctants, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::TAU;

/// Sparks' meta.
pub static META: Meta = Meta {
    name: "sparks",
    slug: "sparks",
    category: Category::Effects,
    note: "an octant grinder throwing braille sparks, white cooling to red",
    cols: 64,
    rows: 20,
    fps: 30,
    charset: Charset::Extended,
    // Sparks, ember to white; the disc, its rim, its label, its nut; the
    // guard; the handle; the motor, its vents; the plate and its edge.
    palette: Some(&[
        "#5a140c", "#c8381a", "#ff8a2a", "#ffe07a", "#ffffff", "#34363c", "#c8ccd4", "#f2c42a", "#16161a", "#7a7e88", "#2a2a2e", "#2a8aa0", "#0e3a44",
        "#5a5e66", "#a4a8b2",
    ]),
    ground: Some("#08090c"),
    ..Meta::DEFAULT
};

const HEAT: Ramp = Ramp::new(0, 5);
const DISC: u8 = 5;
const RIM_SILVER: u8 = 6;
const LABEL: u8 = 7;
const NUT: u8 = 8;
const GUARD: u8 = 9;
const HANDLE: u8 = 10;
const MOTOR: u8 = 11;
const VENT: u8 = 12;
const PLATE: u8 = 13;
const PLATE_EDGE: u8 = 14;
/// The plate's top face in rows (sparks skip along the row above it), the
/// disc's radius in rows (twice that in columns), and how far the disc cuts
/// into the plate.
const FLOOR: f64 = 17.0;
const RD: f64 = 3.7;
const SINK: f64 = 0.3;
const DX: f64 = 24.0;
const DY: f64 = FLOOR - RD + SINK;
/// Gravity in rows a second squared; sparks a second at full pressure; the
/// longest a spark glows; speed kept up and along at each bounce.
const G: f64 = 30.0;
const RATE: f64 = 320.0;
const LIFE: f64 = 1.3;
const BOUNCE: f64 = 0.3;
const SKID: f64 = 0.55;

fn hash(i: i64, s: i64) -> f64 {
    let mut h = ((i as u32) ^ 0x9e37_79b9).wrapping_mul(0x85eb_ca6b) ^ ((s + 1) as u32).wrapping_mul(0xc2b2_ae35);
    h = (h ^ (h >> 15)).wrapping_mul(0x2c1b_3c6d);
    h = (h ^ (h >> 12)).wrapping_mul(0x297a_2d39);
    f64::from(h ^ (h >> 15)) / 4_294_967_296.0
}

/// Where the rim comes up out of the cut: the sparks leave from there.
fn ex() -> f64 {
    DX + 2.0 * (RD * RD - (RD - SINK).powi(2)).sqrt()
}

/// A spark: when it left the cut, its speed across and up, how long it glows.
struct Spark {
    t0: f64,
    vx: f64,
    vy: f64,
    life: f64,
}

/// Where a spark is at age a (x, y rows down, vx, vy upward, bounces): it
/// hops along the plate, losing speed each time it lands.
fn at(s: &Spark, mut a: f64) -> Option<(f64, f64, f64, f64, u32)> {
    let (mut x, mut vx, mut vy) = (ex(), s.vx, s.vy);
    for b in 0..3 {
        let tt = 2.0 * vy / G;
        if a < tt {
            return Some((x + vx * a, FLOOR - 0.5 - vy * a + G * a * a / 2.0, vx, vy - G * a, b));
        }
        x += vx * tt;
        a -= tt;
        vx *= SKID;
        vy *= BOUNCE;
    }
    None
}

/// Spark i: when it left the cut, its speed, angle and how long it glows.
/// Most leave close along the tangent of the rim; the grinder bites harder
/// and softer by turns.
fn spark(i: i64) -> Option<Spark> {
    let t0 = (i as f64 + hash(i, 1)) / RATE;
    let press = 0.72 + 0.28 * (t0 * 1.9).sin() * (t0 * 0.7 + 1.0).sin();
    if hash(i, 2) > press {
        return None;
    }
    let mut deg = 4.0 + 30.0 * ((hash(i, 3) + hash(i, 6)) / 2.0);
    if hash(i, 7) < 0.1 {
        deg += 25.0 * hash(i, 8); // a few kicked higher off the guard
    }
    let (ang, v) = (deg.to_radians(), 34.0 + 44.0 * hash(i, 4));
    Some(Spark { t0, vx: v * ang.cos(), vy: v * ang.sin() / 2.0, life: 0.3 + (LIFE - 0.3) * hash(i, 5).powf(0.8) })
}

/// The grinder at a point in cells: its light and its part, or None: the
/// disc in front, then its guard, the side handle and the motor body.
fn tool(x: f64, y: f64, spin: f64) -> Option<(f64, u8)> {
    let (gx, gy) = (x - DX, (DY - y) * 2.0); // in columns, y up
    let (d, ang) = (gx.hypot(gy) / 2.0, gy.atan2(gx)); // d in rows
    if d < RD {
        if d < 0.5 {
            return Some((0.1, NUT));
        }
        if !(1.05..=RD - 0.55).contains(&d) {
            return Some((0.95, RIM_SILVER));
        }
        // The printed label: one bright arc that turns with the disc.
        let k = (ang - spin).rem_euclid(TAU);
        if d > 1.7 && d < 2.6 && k < 1.3 {
            return Some((0.8, LABEL));
        }
        return Some((0.3, DISC));
    }
    // The guard wraps the half of the disc toward the operator.
    if d > RD + 0.45 && d < RD + 1.35 && ang > 0.75 && ang < 3.8 {
        return Some((if d < RD + 0.9 { 1.0 } else { 0.7 }, GUARD));
    }
    // The side handle stands up from the gearhead: a knurled grip with a
    // rounded end on a short stem, lit from the left.
    let hx = x - (DX - 3.5);
    let (top, grip, base) = (1.2, 6.6, DY - RD - 1.2);
    if y > top && y < base && hx.abs() < if y < grip { 1.6 } else { 0.9 } {
        if y < top + 0.6 && hx.abs() > 1.1 {
            return None;
        }
        let knurl = y > top + 1.0 && y < grip - 0.5 && (y * 2.0).floor() as i64 % 2 == 0;
        return Some((
            if hx > 0.3 {
                0.4
            } else if knurl {
                0.7
            } else {
                0.95
            },
            HANDLE,
        ));
    }
    // The motor body, a cylinder lit from above, vented near the back.
    let by = (y - (DY - 0.4)) / 2.0;
    if x > 1.0 && x < DX && by.abs() < 1.0 {
        if x < 2.5 && by.abs() > 0.55 {
            return None;
        }
        if x > 4.0 && x < 11.0 && (by + 0.1).abs() < 0.4 && x.floor() as i64 % 2 == 0 {
            return Some((0.1, VENT));
        }
        return Some((
            if by < -0.5 {
                0.95
            } else if by < 0.1 {
                0.75
            } else if by < 0.55 {
                0.45
            } else {
                0.25
            },
            MOTOR,
        ));
    }
    None
}

/// The grinder's sparks and its octant body.
pub struct Sparks {
    sparks: ColorDots,
    tool: ColorOctants,
}

/// Sparks; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Sparks { sparks: ColorDots::new(META.cols, META.rows), tool: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Sparks {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (w, h) = (self.tool.width() as i64, self.tool.height() as i64);
        let floor = (FLOOR * 4.0) as i64;
        self.sparks.clear();
        for i in ((t - LIFE) * RATE).floor() as i64 - 1..=(t * RATE).floor() as i64 {
            let Some(s) = spark(i) else { continue };
            let a = t - s.t0;
            if a < 0.0 || a > s.life {
                continue;
            }
            let Some((x, y, vx, vy, b)) = at(&s, a) else { continue };
            let heat = (1.0 - a / s.life).powf(1.4) * if b > 0 { 0.5 } else { 1.0 }; // landing on the plate cools it
                                                                                     // In flight, a hot spark draws a streak back along its path.
            if b == 0 && heat > 0.35 {
                let n = ((vx.hypot(2.0 * vy) * 0.04 * heat) * 4.0).ceil() as i64;
                for k in 1..=n {
                    if let Some(q) = at(&s, (a - k as f64 / n as f64 * 0.04 * heat).max(0.0)) {
                        let (qx, qy) = ((q.0 * 2.0) as i64, (q.1 * 4.0) as i64);
                        if qy < floor {
                            self.sparks.set(qx, qy, HEAT.at(heat * 0.7));
                        }
                    }
                }
            }
            let (px, py) = ((x * 2.0) as i64, (y * 4.0) as i64);
            if py < floor {
                self.sparks.set(px, py, HEAT.at(heat));
            }
        }
        // The grinder over the sparks, and the plate under it all.
        self.tool.clear();
        let spin = t * TAU * 0.6; // the bottom of the disc runs toward the spray
        for j in 0..floor {
            for i in 0..w {
                let (x, y) = ((i as f64 + 0.5) / 2.0, (j as f64 + 0.5) / 4.0);
                let Some((v, part)) = tool(x, y, spin) else { continue };
                let ink = if env.paper { 1.05 - v } else { 0.3 + 0.75 * v };
                if part == NUT || part == VENT || dither(i as usize, j as usize, ink) {
                    self.tool.set(i, j, part);
                }
            }
        }
        for i in 0..w {
            for j in floor..h {
                let edge = j < floor + 2;
                if edge || dither(i as usize, j as usize, if env.paper { 0.7 } else { 0.35 }) {
                    self.tool.set(i, j, if edge { PLATE_EDGE } else { PLATE });
                }
            }
        }
        // The bite: white heat where the rim comes out of the cut.
        let (bx, by) = ((ex() * 2.0) as i64, floor - 2);
        let flick = hash((t * 30.0).floor() as i64, 9) < 0.5;
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 1), (2, 1)] {
            if flick || dx >= 0 {
                self.sparks.set(bx + dx, by + dy, HEAT.at(1.0));
            }
        }
        grid.clear();
        self.tool.draw(grid, 0, 0);
        self.sparks.draw(grid, 0, 0);
    }
}
