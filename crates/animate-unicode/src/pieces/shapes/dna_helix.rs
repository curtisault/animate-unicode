//! dna helix: a double helix seen side on, turning about its own axis, with
//! base pairs between the backbones, and the grooves alternate wide and narrow
//! as in B-DNA. After ascii.rest's dna-helix.ts (MIT, @bas3line).
//!
//! In braille the 64×15 cells are a 128×60 picture. Every point is drawn
//! farthest first: the near backbone is two dots thick and clears a dot round
//! itself, so it reads as passing in front, and the far one is a single dot
//! line. Each base pair is coloured half by half, A with T and C with G, from
//! a seeded sequence; everything dims as it turns away.

use crate::util::Rng;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};
use std::f64::consts::PI;

/// The DNA helix's meta.
pub static META: Meta = Meta {
    name: "dna helix",
    slug: "dna-helix",
    category: Category::Shapes,
    note: "a double helix in braille, its base pairs coloured A T C G",
    cols: 64,
    rows: 15,
    fps: 20,
    charset: Charset::Extended,
    // The backbone, dimmest first, then A, T, C, G, each far then near.
    palette: Some(&["#3b4250", "#6b7587", "#a3adbd", "#dfe5ee", "#2f6b3a", "#5fd06f", "#7a2a2e", "#f0626a", "#244a7a", "#5c9cf0", "#7a6418", "#f2cf4a"]),
    ground: Some("#0b0f16"),
    ..Meta::DEFAULT
};

/// Radius in dots (TS's 5 rows).
const A: f64 = 20.0;
/// Dots a turn (TS's 42 columns).
const PITCH: f64 = 84.0;
/// The second backbone trails the first by this, not by half a turn.
const GROOVE: f64 = 0.64 * PI;
/// Dots between base pairs, ten and a half a turn.
const PAIR: usize = 8;
/// Radians a second, and the angle at t = 0.
const SPEED: f64 = 0.5;
const A0: f64 = 2.2;
/// Dots left clear at each end.
const END: usize = 6;
const BACKBONE: Ramp = Ramp::new(0, 4);

/// One point to draw: where, how near (-1 behind the axis to 1 in front), its
/// colour, and whether it is a backbone (which clears round itself).
#[derive(Clone, Copy)]
struct Mark {
    x: i64,
    y: i64,
    z: f64,
    color: u8,
    backbone: bool,
}

/// The helix: its base sequence (0 A, 1 T, 2 C, 3 G, one per pair), the marks
/// of a frame and the dots.
pub struct DnaHelix {
    bases: Vec<u8>,
    marks: Vec<Mark>,
    dots: ColorDots,
}

/// A DNA helix; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let mut rng = Rng::new(0xD1A);
    Box::new(DnaHelix { bases: (0..64).map(|_| rng.below(4) as u8).collect(), marks: Vec::with_capacity(4096), dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for DnaHelix {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let (w, h) = (self.dots.width(), self.dots.height());
        let a = A0 + SPEED * t;
        let k = 2.0 * PI / PITCH;
        // Backbone s at x: its row and its depth. Rows run down the page, so
        // this winds right-handed.
        let at = |x: f64, s: usize| {
            let th = k * (x - w as f64 / 2.0) - a - s as f64 * GROOVE;
            (h as f64 / 2.0 + A * th.cos(), -th.sin())
        };
        let (lo, hi) = (END, w - 1 - END);
        self.marks.clear();

        // Base pairs: from one backbone to the other at the same place on the
        // axis, each half in its own base's colour. None near the ends.
        let first = lo + 6 + ((hi - lo - 12) % PAIR) / 2;
        for (i, x) in (first..=hi - 6).step_by(PAIR).enumerate() {
            let base = self.bases[i % self.bases.len()];
            let pair = base ^ 1; // A-T and C-G
            let ((y0, z0), (y1, z1)) = (at(x as f64 + 0.5, 0), at(x as f64 + 0.5, 1));
            let (r0, r1) = (y0.min(y1).ceil() as i64, y0.max(y1).floor() as i64);
            for y in r0..=r1 {
                let u = (y as f64 + 0.5 - y0) / (y1 - y0);
                let z = z0 + u * (z1 - z0);
                let b = if u < 0.5 { base } else { pair };
                let color = 4 + b * 2 + u8::from(z > 0.0);
                self.marks.push(Mark { x: x as i64, y, z: z - 0.05, color, backbone: false });
            }
        }
        // The backbones, sampled finely enough to stay unbroken where steep.
        for s in 0..2 {
            let mut x = lo as f64;
            while x <= hi as f64 {
                let (y, z) = at(x, s);
                let (xi, yi) = (x as i64, y.floor() as i64);
                // One dot thick behind the axis, two in front of it.
                let color = BACKBONE.at((z + 1.0) / 2.0);
                self.marks.push(Mark { x: xi, y: yi, z, color, backbone: true });
                if z > 0.2 {
                    self.marks.push(Mark { x: xi, y: yi + 1, z, color, backbone: true });
                }
                x += 0.25;
            }
        }
        // Farthest first. A near backbone clears what lies well behind it just
        // above and below, so the crossings read.
        self.marks.sort_by(|p, q| p.z.total_cmp(&q.z));
        self.dots.clear();
        for m in &self.marks {
            if m.backbone && m.z > 0.4 {
                for dy in [-2, -1, 2, 3] {
                    if self.dots.depth(m.x, m.y + dy) < (m.z - 0.8) as f32 {
                        self.dots.unset(m.x, m.y + dy);
                    }
                }
            }
            self.dots.plot(m.x, m.y, m.z as f32, m.color);
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
