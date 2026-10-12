//! spring: a coil spring seen from a little above, a weight hanging from it
//! and bouncing in simple harmonic motion. The coils bunch and stretch. After
//! ascii.rest's spring.ts (MIT, @bas3line).
//!
//! In braille the coil is a true helix at four dots a row, so it stretches
//! smoothly rather than a whole row at a time. Its back half is a thin dim
//! line, its front half a thick bright one that clears a dot round itself,
//! so the wire reads as passing in front. The weight is a brass drum, lit
//! from the left and shaded by ordered dithering.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::{FRAC_PI_2, TAU};

/// The spring's meta.
pub static META: Meta = Meta {
    name: "spring",
    slug: "spring",
    category: Category::Shapes,
    note: "a brass weight bouncing on a steel coil, drawn in braille",
    cols: 40,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    // Steel, dimmest first; brass; the ceiling.
    palette: Some(&["#3a4048", "#69717c", "#a4acb6", "#e6ebf0", "#4a3410", "#86621e", "#c49434", "#f2cd72", "#5b5f66"]),
    ground: Some("#0c0d10"),
    ..Meta::DEFAULT
};

/// From the top of the coil at the back to its foot at the front.
const TURNS: f64 = 3.5;
/// Coil radius, and how far the near side of a coil sits below the far side, in dots.
const RC: f64 = 15.0;
const BULGE: f64 = 7.0;
/// Dots per turn at rest, its swing, and the swing's period in seconds.
const P0: f64 = 14.0;
const AMP: f64 = 2.4;
const PERIOD: f64 = 1.4;
/// The top of the coil, in dots.
const TOP: f64 = 12.0;
/// Samples along the wire.
const STEPS: usize = 2400;
/// The drum: radius, the lid's half height, and the side's height, in dots.
const DRUM_R: f64 = 15.0;
const LID: f64 = 4.0;
const SIDE: f64 = 12.0;
const STEEL: Ramp = Ramp::new(0, 4);
const BRASS: Ramp = Ramp::new(4, 4);
const CEILING: u8 = 8;

/// The spring's dots.
pub struct Spring {
    dots: ColorDots,
}

/// A spring; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    Box::new(Spring { dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for Spring {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let w = self.dots.width() as f64;
        let cx = w / 2.0;
        let pitch = P0 + AMP * (t / PERIOD * TAU).cos();
        let th0 = -FRAC_PI_2;
        let point = |i: usize| {
            let th = th0 + i as f64 / STEPS as f64 * TURNS * TAU;
            (cx + RC * th.cos(), TOP + (th - th0) / TAU * pitch + BULGE * (th.sin() + 1.0), th)
        };
        self.dots.clear();
        // The back of the coil first, a dot thick; then the front, two dots
        // thick, clearing what lies just above and below it.
        for front in [false, true] {
            for i in 0..=STEPS {
                let (x, y, th) = point(i);
                if (th.sin() > 0.0) != front {
                    continue;
                }
                // Lit from the left; the back half in shadow.
                let lit = 0.5 - 0.4 * th.cos() + 0.3 * th.sin();
                let (xi, yi) = (x.floor() as i64, y.floor() as i64);
                let c = STEEL.at(if front { lit } else { lit * 0.4 });
                if front {
                    for dy in [-2, -1, 2, 3] {
                        if self.dots.depth(xi, yi + dy) < 0.0 {
                            self.dots.unset(xi, yi + dy);
                        }
                    }
                    self.dots.plot(xi, yi + 1, 1.0, c);
                }
                self.dots.plot(xi, yi, if front { 1.0 } else { -1.0 }, c);
            }
        }
        // The wire up to the ceiling, and the hook down to the drum.
        let (top_x, foot) = (cx.floor() as i64, point(STEPS).1);
        for y in 8..TOP as i64 + 1 {
            self.dots.set(top_x, y, STEEL.at(0.6));
        }
        let lid = foot + 9.0;
        for y in foot as i64 + 1..(lid - LID) as i64 + 1 {
            self.dots.set(top_x, y, STEEL.at(0.7));
        }
        // The drum: its side lit from the left, then its lid, with an eye for the hook.
        let paper = env.paper;
        let ink = |b: f64| if paper { 0.95 - 0.7 * b } else { 0.2 + 0.75 * b };
        let r = DRUM_R as i64;
        for dx in -r..=r {
            let u = dx as f64 / DRUM_R;
            let rim = (1.0 - u * u).max(0.0).sqrt() * LID;
            let x = (cx + dx as f64).floor();
            let lit = (0.55 - 0.45 * u).clamp(0.0, 1.0);
            let (top, bottom) = (lid + rim, lid + SIDE + rim);
            let mut y = lid - rim;
            while y <= bottom {
                let on_lid = y < top;
                let b = if on_lid { 0.75 - 0.15 * u } else { lit };
                let edge = (y - (lid - rim)).abs() < 1.0 || (y - top).abs() < 1.0 || (y - bottom).abs() < 1.0 || dx.abs() == r;
                if edge || crate::util::dither(x as usize, y as usize, ink(b)) {
                    self.dots.plot(x as i64, y.floor() as i64, 2.0, BRASS.at(if edge { 0.5 } else { b }));
                }
                y += 1.0;
            }
        }
        let eye = (cx.floor() as i64, lid.floor() as i64);
        for (dx, dy) in [(-1, -1), (0, -1), (1, -1), (-1, 0), (1, 0), (-1, 1), (0, 1), (1, 1)] {
            self.dots.set(eye.0 + dx, eye.1 + dy, STEEL.at(1.0));
        }
        self.dots.unset(eye.0, eye.1);

        grid.clear();
        self.dots.draw(grid, 0, 0);
        // The ceiling the spring hangs from: a hatched beam and a plate with the clamp.
        let left = META.cols as i64 / 2 - 11;
        for i in 0..22 {
            grid.set(left + i, 0, '╱', CEILING);
            grid.set(left + i, 1, if i == 11 { '┬' } else { '─' }, CEILING);
        }
    }
}
