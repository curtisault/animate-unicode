//! chladni plate: sand on a square plate ringing in one of its modes. It lies
//! on the lines where two crossed standing waves cancel; when the tone moves
//! on, the grains are shaken loose and travel to the lines of the next
//! figure. After ascii.rest's chladni.ts (MIT, @bas3line).
//!
//! In braille every grain is a dot, so the 43×21 cells inside the rim hold
//! an 86×84 plate and the nodal lines are a dot wide, where box drawing gave
//! them a cell. Each figure's sand takes its own tint as it settles; grains in
//! flight are white, so the sand can be followed from one figure to the next.

use crate::util::Mulberry32;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece};
use std::f64::consts::PI;

/// The Chladni plate's meta.
pub static META: Meta = Meta {
    name: "chladni plate",
    slug: "chladni",
    category: Category::Physics,
    note: "sand in braille finding a ringing plate's lines, tinted by mode",
    cols: 45,
    rows: 23,
    fps: 12,
    charset: Charset::Extended,
    // A tint for each of the eight figures; sand in flight; the rim.
    palette: Some(&["#f2c46a", "#f08a5a", "#e8607a", "#c46ad8", "#7a8af0", "#5ac0e8", "#6ad8a0", "#c8e070", "#ffffff", "#8a8070"]),
    ground: Some("#0e0c0a"),
    ..Meta::DEFAULT
};

/// Each mode is cos(n pi x) cos(m pi y) + sign cos(m pi x) cos(n pi y). With
/// n and m both odd every figure is mirrored both ways.
const MODES: [(f64, f64, f64); 8] =
    [(1.0, 3.0, -1.0), (1.0, 5.0, -1.0), (3.0, 5.0, -1.0), (3.0, 7.0, -1.0), (5.0, 7.0, -1.0), (3.0, 7.0, 1.0), (3.0, 5.0, 1.0), (1.0, 5.0, 1.0)];
/// Seconds the sand takes to find the next figure, and that a figure rests.
const MOVE: f64 = 2.8;
const HOLD: f64 = 2.2;
const CYCLE: f64 = MOVE + HOLD;
/// The fourth figure at rest, just before the tone moves.
const START: f64 = 4.0 * CYCLE - 0.2;
const GRAINS: usize = 2600;
/// Share of the sand lying just off the lines.
const LOOSE: f64 = 0.1;
const FLYING: u8 = 8;
const RIM: u8 = 9;

/// One mode at (x, y) on a 1 by 1 plate, and its slope each way.
fn wave((n, m, s): (f64, f64, f64), x: f64, y: f64) -> (f64, f64, f64) {
    let (a, b) = ((n * PI * x).cos(), (m * PI * y).cos());
    let (c, d) = ((m * PI * x).cos(), (n * PI * y).cos());
    (
        a * b + s * c * d,
        -PI * (n * (n * PI * x).sin() * b + s * m * (m * PI * x).sin() * d),
        -PI * (m * a * (m * PI * y).sin() + s * n * c * (n * PI * y).sin()),
    )
}

/// The distance along a Hilbert curve of order 7 through (x, y), both under 128.
fn hilbert(mut x: u32, mut y: u32) -> u32 {
    let mut d = 0;
    let mut s = 64;
    while s > 0 {
        let (rx, ry) = (u32::from(x & s > 0), u32::from(y & s > 0));
        d += s * s * ((3 * rx) ^ ry);
        if ry == 0 {
            if rx == 1 {
                x = 127 - x;
                y = 127 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        s /= 2;
    }
    d
}

/// One change of figure: where each grain starts and lands, when it sets
/// off, how long it takes, and how far it is thrown off the straight way.
struct Trip {
    from: Vec<(f32, f32)>,
    to: Vec<(f32, f32)>,
    delay: Vec<f32>,
    span: Vec<f32>,
    sway: Vec<f32>,
}

/// The plate: a trip for each change of figure, and the dots.
pub struct Chladni {
    trips: Vec<Trip>,
    dots: ColorDots,
}

/// Chladni; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rnd = Mulberry32::new(18);
    // Inside the rim: one cell all round, in dots.
    let (iw, ih) = ((META.cols - 2) * 2, (META.rows - 2) * 4);
    // Where the sand lies on each figure: on every dot within half a dot of a
    // nodal line, spread evenly along them, a little loose beside them.
    let rest: Vec<Vec<(f32, f32)>> = MODES
        .iter()
        .map(|&mode| {
            let mut on = vec![];
            for y in 0..ih {
                for x in 0..iw {
                    let (v, fx, fy) = wave(mode, (x as f64 + 0.5) / iw as f64, (y as f64 + 0.5) / ih as f64);
                    let slope = ((fx / iw as f64).powi(2) + (fy / ih as f64).powi(2)).sqrt();
                    if v.abs() < 0.5 * slope.max(1e-9) {
                        on.push((x, y, fx / iw as f64, fy / ih as f64));
                    }
                }
            }
            for i in (1..on.len()).rev() {
                on.swap(i, (rnd.next_f64() * (i + 1) as f64) as usize);
            }
            (0..GRAINS)
                .map(|g| {
                    let loose = rnd.next_f64() < LOOSE;
                    let (x, y, ex, ey) = if loose { on[(rnd.next_f64() * on.len() as f64) as usize] } else { on[g % on.len()] };
                    let (mut x, mut y) = (x as f64, y as f64);
                    if loose {
                        // A dot or two off the line, across it.
                        let side = if rnd.next_f64() < 0.5 { -1.5 } else { 1.5 };
                        let m = (ex * ex + ey * ey).sqrt().max(1e-9);
                        x = (x + side * ex / m).clamp(0.0, iw as f64 - 1.0);
                        y = (y + side * ey / m).clamp(0.0, ih as f64 - 1.0);
                    }
                    ((x + 0.5) as f32, (y + 0.5) as f32)
                })
                .collect()
        })
        .collect();
    // For each change of figure, which resting place each grain makes for:
    // both sets in order along a Hilbert curve, paired off, so grains travel
    // to places near them.
    let trips = (0..MODES.len())
        .map(|k| {
            let (a, b) = (&rest[(k + MODES.len() - 1) % MODES.len()], &rest[k]);
            let order = |pts: &[(f32, f32)]| {
                let mut idx: Vec<usize> = (0..pts.len()).collect();
                idx.sort_by_key(|&i| hilbert(pts[i].0 as u32, pts[i].1 as u32));
                idx
            };
            let (oa, ob) = (order(a), order(b));
            let mut to = vec![(0.0, 0.0); GRAINS];
            for (i, &g) in oa.iter().enumerate() {
                to[g] = b[ob[i]];
            }
            // Sand where the new tone shakes hardest leaves first; a grain
            // already on a line barely stirs. Lengths in ems: a dot is 0.3.
            let (mut delay, mut span, mut sway) = (vec![0.0; GRAINS], vec![0.0; GRAINS], vec![0.0; GRAINS]);
            for g in 0..GRAINS {
                let (x0, y0) = a[g];
                let shake = (wave(MODES[k], f64::from(x0) / iw as f64, f64::from(y0) / ih as f64).0.abs() / 0.9).min(1.0);
                let len = ((f64::from(to[g].0 - x0)).powi(2) + (f64::from(to[g].1 - y0)).powi(2)).sqrt() * 0.3;
                let d = 0.15 + 0.8 * (1.0 - shake) + 0.3 * rnd.next_f64();
                delay[g] = d as f32;
                span[g] = if len < 0.8 { 1e-6 } else { (MOVE - 0.2 - d).min(0.35 + 0.16 * len + 0.2 * rnd.next_f64()) as f32 };
                sway[g] = ((rnd.next_f64() - 0.5) * 1.6) as f32;
            }
            Trip { from: a.clone(), to, delay, span, sway }
        })
        .collect();
    Box::new(Chladni { trips, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Chladni {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let tt = START + t;
        let n = (tt / CYCLE).floor();
        let tau = tt - n * CYCLE;
        let k = n as usize % MODES.len();
        let trip = &self.trips[k];
        let was = ((k + MODES.len() - 1) % MODES.len()) as u8;
        self.dots.clear();
        for g in 0..GRAINS {
            let u = ((tau - f64::from(trip.delay[g])) / f64::from(trip.span[g])).clamp(0.0, 1.0);
            let e = u * u * (3.0 - 2.0 * u);
            let ((x0, y0), (x1, y1)) = (trip.from[g], trip.to[g]);
            let (dx, dy) = (f64::from(x1 - x0), f64::from(y1 - y0));
            let len = (dx * dx + dy * dy).sqrt() + 1e-9;
            // Thrown a little off the straight way while it hops, landing true.
            let s = f64::from(trip.sway[g]) * (PI * e).sin() * (len / 6.0).min(1.0) * 2.0;
            let x = f64::from(x0) + dx * e - dy / len * s;
            let y = f64::from(y0) + dy * e + dx / len * s;
            let color = if u == 0.0 {
                was
            } else if u == 1.0 {
                k as u8
            } else {
                FLYING
            };
            // Inside the rim: one cell (two dots across, four down) in.
            self.dots.set(x.floor() as i64 + 2, y.floor() as i64 + 4, color);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
        let (c1, r1) = (META.cols as i64 - 1, META.rows as i64 - 1);
        for c in 1..c1 {
            grid.set(c, 0, '─', RIM);
            grid.set(c, r1, '─', RIM);
        }
        for r in 1..r1 {
            grid.set(0, r, '│', RIM);
            grid.set(c1, r, '│', RIM);
        }
        for (c, r, ch) in [(0, 0, '┌'), (c1, 0, '┐'), (0, r1, '└'), (c1, r1, '┘')] {
            grid.set(c, r, ch, RIM);
        }
    }
}
