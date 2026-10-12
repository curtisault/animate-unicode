//! ruled mountains: ruled lines with a range rising out of their middle. Each
//! line hides what it rises over, and the peaks drift slowly from the back
//! lines to the front. After ascii.rest's ruled-mountains.ts (MIT,
//! @bas3line).
//!
//! The lines, the peaks and their drift are ascii.rest's. Where ascii.rest
//! draws each line in whole rows of _ / and \, these are braille, at four
//! times the vertical resolution, so the flanks run as true slopes; every
//! line is drawn front to back and only where it rises above everything
//! nearer, so the hidden parts are removed dot by dot. In colour the range
//! recedes into a cold haze: the back lines a dim blue-violet, the front ones
//! nearly white, the lit flanks stippled in their line's colour.

use crate::util::dither;
use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// The ruled mountains' meta.
pub static META: Meta = Meta {
    name: "ruled mountains",
    slug: "ruled-mountains",
    category: Category::Nature,
    note: "braille ruled lines raising a range, hidden lines removed, hazy blue",
    cols: 64,
    rows: 20,
    fps: 10,
    charset: Charset::Extended,
    // Back to front.
    palette: Some(&["#3c4070", "#545a8c", "#7076a8", "#9298c4", "#b6badc", "#dcdef2"]),
    ground: Some("#090a12"),
    ..Meta::DEFAULT
};

const HAZE: Ramp = Ramp::new(0, 6);
/// Ruled lines back to front, rows between them, the tallest peak on each
/// line in rows (nearer stands taller), lines a second the peaks come
/// forward, and seconds into the drift where t = 0 falls.
const N: usize = 6;
const GAP: f64 = 3.0;
const AMP: [f64; N] = [5.0, 5.5, 6.0, 6.6, 7.2, 8.0];
const SPEED: f64 = 0.2;
const START: f64 = 30.0;
/// Each peak keeps to its lane, so the range stays balanced: lane in columns
/// from the middle, how far it wanders, its height against the line's
/// tallest, rows of steep flank west and east before it eases to a shoulder,
/// where a lesser summit sits on its flank (0 for none), then its wandering.
const PEAKS: [[f64; 10]; 4] = [
    [-16.0, 3.0, 0.55, 2.0, 1.0, 0.0, 0.71, 0.4, 0.93, 1.9],
    [-5.0, 3.0, 0.85, 3.0, 2.0, -4.0, 0.53, 2.6, 1.21, 4.1],
    [6.0, 3.0, 1.0, 2.0, 4.0, 5.0, 0.97, 4.4, 0.67, 0.3],
    [18.0, 3.0, 0.6, 3.0, 2.0, 0.0, 0.61, 1.2, 1.07, 5.2],
];

/// A peak's height at distance d from its summit: a row a column down its
/// steep upper flank, then half that, a shoulder, to the foot.
fn tent(h: f64, d: f64, steep: f64) -> f64 {
    h - if d < steep { d } else { steep + (d - steep) / 2.0 }
}

/// The range: each dot column's horizon, a line's heights, and the dots.
pub struct RuledMountains {
    horizon: Vec<f64>,
    f: Vec<f64>,
    dots: ColorDots,
}

/// Ruled mountains; they take no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let dots = ColorDots::new(META.cols, META.rows);
    let w = dots.width();
    Box::new(RuledMountains { horizon: vec![0.0; w], f: vec![0.0; w + 1], dots })
}

impl RuledMountains {
    /// The heights along a line at every dot column edge, the shape the peaks
    /// take at drift s, all of them under one envelope.
    fn profile(&mut self, s: f64, amp: f64) {
        let mid = META.cols as f64 / 2.0;
        self.f.fill(0.0);
        for [lane, wander, tall, west, east, sub, fm, pm, fa, pa] in PEAKS {
            let m = mid + lane + wander * (fm * s + pm).sin();
            let h = amp * tall * (0.55 + 0.45 * (fa * s + pa).sin());
            for (i, f) in self.f.iter_mut().enumerate() {
                let d = i as f64 / 2.0 - m;
                let mut y = tent(h, d.abs(), if d < 0.0 { west } else { east });
                if sub != 0.0 {
                    y = y.max(tent(h - 2.0, (d - sub).abs(), 1.5));
                }
                *f = f.max(y);
            }
        }
    }
}

impl Piece for RuledMountains {
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid) {
        let rows = META.rows as f64;
        let (w, h) = (self.dots.width(), self.dots.height() as f64);
        self.dots.clear();
        self.horizon.fill(h);
        // Stipple is ink: on a light page it marks the shaded east flanks, on
        // a dark one the lit west flanks.
        let lit_rising = !env.paper;
        // Front to back, so each line only draws where nothing nearer has.
        for i in (0..N).rev() {
            let base = (rows - 1.0 - (N - 1 - i) as f64 * GAP) * 4.0 + 3.0;
            // Line i takes the shape line i - 1 had 1 / SPEED seconds ago.
            self.profile((i as f64 - (START + t) * SPEED) * 0.9, AMP[i]);
            let color = HAZE.at(i as f64 / (N - 1) as f64);
            for x in 0..w {
                let (a, b) = (base - self.f[x] * 4.0, base - self.f[x + 1] * 4.0);
                let below = self.horizon[x];
                // The line through this dot column, a run of dots wherever it is
                // steep, each shown only above everything nearer.
                let (top, bottom) = (a.min(b).round() as i64, a.max(b).round() as i64);
                for y in top..=bottom.max(top) {
                    if (y as f64) < below {
                        self.dots.set(x as i64, y, color);
                    }
                }
                // Stipple down the lit flank to whatever is nearer.
                if (b < a) == lit_rising && (a - b).abs() > 0.2 {
                    for y in bottom + 2..(below.min(base) as i64) {
                        if dither(x, y as usize, 0.14) {
                            self.dots.set(x as i64, y, color);
                        }
                    }
                }
                self.horizon[x] = below.min(top as f64);
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}
