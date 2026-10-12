//! butterfly: seen from above, beating its wings in bursts and gliding
//! between them. As a wing rises it turns edge on and narrows, its tip
//! nearer the eye. After ascii.rest's butterfly.ts (MIT, @bas3line).
//!
//! The wings' shapes and markings, the beat, the glide and the perspective
//! are ascii.rest's, sampled once per octant dot, so the wings' edges and
//! markings are drawn eight dots to a cell and narrow smoothly as they rise.
//! In colour it is a monarch's cousin: orange wings darkening at their roots,
//! black margins with a row of cream spots, a cream band across each
//! forewing, and on each hindwing an eyespot of blue ringed in gold.

use crate::util::dither;
use crate::{Category, Charset, ColorOctants, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The butterfly's meta.
pub static META: Meta = Meta {
    name: "butterfly",
    slug: "butterfly",
    category: Category::Creatures,
    note: "an octant butterfly flapping and gliding, orange, black, an eyespot",
    cols: 64,
    rows: 24,
    fps: 30,
    charset: Charset::Extended,
    // The wing, its root; the margin; the spots and band; the eyespot's
    // pupil, its blue, its gold ring; the body.
    palette: Some(&["#f28a26", "#a64a18", "#1e1612", "#f6ecd2", "#f4f8ff", "#2a48a8", "#f2c63c", "#3a2a22"]),
    ground: Some("#0c0e0a"),
    ..Meta::DEFAULT
};

const WING: u8 = 0;
const ROOT: u8 = 1;
const MARGIN: u8 = 2;
const PALE: u8 = 3;
const PUPIL: u8 = 4;
const IRIS: u8 = 5;
const RING: u8 = 6;
const BODY: u8 = 7;
/// Seconds a loop (the path and the bursts all divide it), wingbeats a
/// second, and how far the eye is above the wings, in columns.
const LOOP: f64 = 16.0;
const BEAT: f64 = 2.5;
const EYE: f64 = 120.0;

/// A wing, a teardrop wider at its outer end: centre, length, width, tilt.
struct Drop {
    x: f64,
    y: f64,
    a: f64,
    b: f64,
    r: f64,
}

const FORE: Drop = Drop { x: 9.6, y: 5.6, a: 11.6, b: 7.2, r: 0.5 };
const HIND: Drop = Drop { x: 7.0, y: -4.6, a: 8.8, b: 7.4, r: -0.62 };

fn drop(w: &Drop, x: f64, y: f64) -> (f64, f64, f64) {
    let (dx, dy) = (x - w.x, y - w.y);
    let (s, c) = w.r.sin_cos();
    let p = (dx * c + dy * s) / w.a;
    let q = (dy * c - dx * s) / (w.b * (0.58 + 0.42 * ((p + 1.0) / 2.0).clamp(0.0, 1.0)));
    (p, q, p * p + q * q)
}

/// How pale the wing is at (x, y) in its own plane, and what marking is
/// there; None off the wing.
fn wing(x: f64, y: f64) -> Option<(f64, u8)> {
    let root = x.hypot(y * 0.8) < 3.6;
    let (p, q, r) = drop(&FORE, x, y);
    if r < 1.0 {
        let (rho, be) = (r.sqrt(), q.atan2(p));
        // A dark margin round the tip, tapering toward the root, with one row
        // of pale spots in it.
        if rho > 0.76 + 0.2 * ((0.5 - p) / 0.8).clamp(0.0, 1.0) {
            return Some(if rho > 0.8 && rho < 0.95 && p > 0.3 && (be * 7.0 + 0.3).cos() > 0.45 { (0.9, PALE) } else { (0.24, MARGIN) });
        }
        // One pale band slanting across the middle.
        if (p * 0.85 + q * 0.5 - 0.02).abs() < 0.17 {
            return Some((0.9, PALE));
        }
        return Some(if root { (0.3, ROOT) } else { (0.55, WING) });
    }
    let (p, _, r) = drop(&HIND, x, y);
    if r >= 1.0 {
        return None;
    }
    // A round eyespot: a pale ring round a dark middle and a bright pupil.
    let e = (x - 9.4).hypot((y + 6.4) * 0.8);
    Some(if e < 1.0 {
        (1.0, PUPIL)
    } else if e < 2.3 {
        (0.1, IRIS)
    } else if e < 3.8 {
        (0.9, RING)
    } else if r.sqrt() > 0.8 && p > -0.3 {
        (0.24, MARGIN)
    } else if root {
        (0.3, ROOT)
    } else {
        (0.55, WING)
    })
}

/// The butterfly's dots.
pub struct Butterfly {
    dots: ColorOctants,
}

/// A butterfly; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Butterfly { dots: ColorOctants::new(META.cols, META.rows) })
}

impl Piece for Butterfly {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let (cols, rows) = (META.cols as f64, META.rows as f64);
        let (w, h) = (self.dots.width(), self.dots.height());
        let u = t.rem_euclid(LOOP);
        let om = TAU / LOOP;
        // Flap for four seconds, glide for three; the glide starts with the wings low.
        let g = u % 8.0;
        let flap = if g < 4.0 {
            1.0
        } else if g < 4.4 {
            1.0 - (g - 4.0) / 0.4
        } else if g > 7.6 {
            (g - 7.6) / 0.4
        } else {
            0.0
        };
        let beat = (TAU * BEAT * u).cos();
        let th = flap * (0.6 - 0.72 * beat) - 0.06 * (1.0 - flap);
        // A slow, wide meander. Drifting sideways, it banks: the wing on that side dips.
        let bx = cols / 2.0 + 9.0 * (om * u).sin() + 2.0 * (3.0 * om * u + 0.6).sin();
        let by = rows / 2.0 + 0.6 + 1.4 * (2.0 * om * u + 1.0).sin() + 0.35 * flap * (TAU * BEAT * u).sin();
        let bank = (0.16 - 0.1 * flap) * ((om * u).cos() + 0.67 * (3.0 * om * u + 0.6).cos()) / 1.67;
        let tilt = [th - bank, th + bank].map(|a| (a.cos(), a.sin()));
        let dim = 0.82 + 0.18 * th.cos(); // a raised wing catches less light
                                          // Where a point on screen (columns out from the body, y up) lands on
                                          // the raised wing, undoing the perspective.
        let sample = |sx: f64, sy: f64| {
            let (ct, st) = tilt[usize::from(sx >= 0.0)];
            let ax = sx.abs();
            let den = EYE * ct + ax * st;
            if den <= 0.0 {
                return None;
            }
            let x = ax * EYE / den;
            wing(x, sy / (EYE / (EYE - x * st)))
        };
        self.dots.clear();
        // The clubbed antennae go down first, so a raised wing hides them.
        let sway = 0.4 * (PI * u).sin();
        for side in [-1.0, 1.0] {
            for k in 0..=20 {
                let f = f64::from(k) / 20.0;
                let (x, y) = (bx + side * (0.5 + 2.6 * f + sway * f * f), by - 2.0 - 5.0 * f);
                self.dots.set((x * 2.0) as i64, (y * 4.0) as i64, BODY);
            }
            let (x, y) = (bx + side * (3.1 + sway), by - 7.0);
            for (dx, dy) in [(0, 0), (1, 0), (0, -1), (1, -1)] {
                self.dots.set((x * 2.0) as i64 + dx, (y * 4.0) as i64 + dy, BODY);
            }
        }
        for j in 0..h {
            for i in 0..w {
                let (sx, sy) = ((i as f64 + 0.5) / 2.0 - bx, (by - (j as f64 + 0.5) / 4.0) * 2.0);
                let Some((v, mark)) = sample(sx, sy) else { continue };
                let light = v * dim;
                // Light is ink on a dark page; on paper the dark markings are.
                let ink = if env.paper { 1.05 - light } else { 0.3 + 0.75 * light };
                if dither(i, j, ink) {
                    self.dots.set(i as i64, j as i64, mark);
                }
            }
        }
        // The body, head to tail, over the wing roots: a round head, a thick
        // thorax, a long tapering abdomen.
        for j in ((by - 1.6) * 4.0) as i64..((by + 4.6) * 4.0) as i64 {
            let y = j as f64 / 4.0 - by;
            let half = if y < -0.9 {
                0.75
            } else if y < 1.0 {
                0.85
            } else {
                0.6 * (1.0 - (y - 1.0) / 3.8) + 0.15
            };
            for i in ((bx - half) * 2.0).round() as i64..((bx + half) * 2.0).round() as i64 {
                self.dots.set(i, j, BODY);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
