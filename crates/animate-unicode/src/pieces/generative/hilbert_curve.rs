//! hilbert curve: two Hilbert curves, each through every point of a grid,
//! joined end to end into one closed loop, with two lit runners circling it.
//! After ascii.rest's hilbert-curve.ts (MIT, @bas3line).
//!
//! In braille the curve goes one order finer, a 32 by 16 grid four dots a
//! step, where box drawing had room for 16 by 8. The loop is a faint line a
//! dot wide; each runner is a bright stroke two dots wide, amber or cyan,
//! fading back along a long trail, so the eye can follow it round the turns.

use crate::{Category, Charset, ColorDots, Env, Grid, Meta, Options, Piece, Ramp};

/// The Hilbert curve's meta.
pub static META: Meta = Meta {
    name: "hilbert curve",
    slug: "hilbert-curve",
    category: Category::Generative,
    note: "a braille loop of two hilbert curves, with two coloured runners",
    cols: 65,
    rows: 17,
    fps: 20,
    charset: Charset::Extended,
    // The loop; the amber runner's trail, faint to bright; the cyan one's.
    palette: Some(&["#2c3345", "#5a3a12", "#9a6418", "#e0a030", "#ffe08a", "#0f4a5a", "#1a8296", "#36c2d8", "#a8f2ff"]),
    ground: Some("#090b10"),
    ..Meta::DEFAULT
};

/// Grid points across and down, and dots from one to the next.
const GW: i64 = 32;
const GH: i64 = 16;
const STEP: i64 = 4;
/// The margin round the drawing, in dots.
const OX: i64 = 3;
const OY: i64 = 3;
/// A runner's lit stretch, and its speed, in dots.
const LEN: usize = 90;
const SPEED: f64 = 32.0;
const LOOP: u8 = 0;
const RUNNERS: [Ramp; 2] = [Ramp::new(1, 4), Ramp::new(5, 4)];

/// Jakub Cerveny's generalized Hilbert curve (gilbert2d), for any w by h grid.
#[allow(clippy::too_many_arguments)]
fn gilbert(pts: &mut Vec<(i64, i64)>, mut x: i64, mut y: i64, ax: i64, ay: i64, bx: i64, by: i64) {
    let (w, h) = ((ax + ay).abs(), (bx + by).abs());
    let (dax, day, dbx, dby) = (ax.signum(), ay.signum(), bx.signum(), by.signum());
    if h == 1 || w == 1 {
        let (dx, dy, n) = if h == 1 { (dax, day, w) } else { (dbx, dby, h) };
        for _ in 0..n {
            pts.push((x, y));
            x += dx;
            y += dy;
        }
        return;
    }
    let half = |v: i64| v.div_euclid(2);
    let (mut ax2, mut ay2, mut bx2, mut by2) = (half(ax), half(ay), half(bx), half(by));
    if 2 * w > 3 * h {
        if (ax2 + ay2).abs() % 2 == 1 && w > 2 {
            ax2 += dax;
            ay2 += day;
        }
        gilbert(pts, x, y, ax2, ay2, bx, by);
        gilbert(pts, x + ax2, y + ay2, ax - ax2, ay - ay2, bx, by);
    } else {
        if (bx2 + by2).abs() % 2 == 1 && h > 2 {
            bx2 += dbx;
            by2 += dby;
        }
        gilbert(pts, x, y, bx2, by2, ax2, ay2);
        gilbert(pts, x + bx2, y + by2, ax, ay, bx - bx2, by - by2);
        gilbert(pts, x + (ax - dax) + (bx2 - dbx), y + (ay - day) + (by2 - dby), -bx2, -by2, -(ax - ax2), -(ay - ay2));
    }
}

/// The loop's grid points: the left half runs down beside its right edge and
/// the right half back up beside its left, so each ends a step from where the
/// other starts.
fn points() -> Vec<(i64, i64)> {
    let mut pts = vec![];
    gilbert(&mut pts, GW / 2 - 1, 0, 0, GH, -GW / 2, 0);
    gilbert(&mut pts, GW / 2, GH - 1, 0, -GH, GW / 2, 0);
    pts
}

/// The loop as a run of dots, in order, and the dots.
pub struct HilbertCurve {
    path: Vec<(i64, i64)>,
    dots: ColorDots,
}

/// A Hilbert curve; it takes no options.
pub fn make(_: &Options) -> Box<dyn Piece> {
    let pts = points();
    let mut path = vec![];
    for (k, &(x, y)) in pts.iter().enumerate() {
        let (nx, ny) = pts[(k + 1) % pts.len()];
        for s in 0..STEP {
            path.push((OX + (x * STEP) + (nx - x) * s, OY + (y * STEP) + (ny - y) * s));
        }
    }
    Box::new(HilbertCurve { path, dots: ColorDots::new(META.cols, META.rows) })
}

impl Piece for HilbertCurve {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let n = self.path.len();
        self.dots.clear();
        for &(x, y) in &self.path {
            self.dots.set(x, y, LOOP);
        }
        // Two runners half a lap apart, each a stroke two dots wide fading
        // back along its trail.
        let head = (80.0 + t * SPEED) as usize;
        for (r, ramp) in RUNNERS.iter().enumerate() {
            let at = head + r * n / 2;
            for back in (0..LEN).rev() {
                let i = (at + n - back % n) % n;
                let (x, y) = self.path[i];
                let (px, py) = self.path[(i + n - 1) % n];
                let shade = 1.0 - back as f64 / LEN as f64;
                let c = ramp.at(shade);
                self.dots.set(x, y, c);
                // The second dot sits beside the line, across its way.
                if back < LEN * 2 / 3 {
                    let (dx, dy) = (x - px, y - py);
                    self.dots.set(x + dy.abs(), y + dx.abs(), c);
                }
            }
        }
        grid.clear();
        self.dots.draw(grid, 0, 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_loop_visits_every_point_once_and_closes() {
        let pts = points();
        assert_eq!(pts.len(), (GW * GH) as usize);
        let mut seen = pts.clone();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), pts.len(), "no point twice");
        for k in 0..pts.len() {
            let (a, b) = (pts[k], pts[(k + 1) % pts.len()]);
            assert_eq!((a.0 - b.0).abs() + (a.1 - b.1).abs(), 1, "step {k} is one grid step, the last closing the loop");
        }
    }
}
