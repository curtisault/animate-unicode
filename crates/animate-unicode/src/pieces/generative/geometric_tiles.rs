//! geometric tiles: a floor of triangle tiles (◤ ◥ ◢ ◣, from Geometric Shapes)
//! turning a quarter at a time in a ripple that spreads from the centre, each
//! tile turning against its neighbours so the floor reads as pinwheels. Each
//! wave crest passes as a ring of diamonds, and a dot marks the centre.
//!
//! Geometric shapes are icons inside the cell, not cell-filling blocks, so the
//! tiles stand apart; the pattern is in their turning, not in joins.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};

/// The geometric tiles' meta.
pub static META: Meta = Meta {
    name: "geometric tiles",
    slug: "geometric-tiles",
    category: Category::Generative,
    note: "triangle tiles turning in rings from the centre, diamonds on crests",
    cols: 40,
    rows: 14,
    fps: 20,
    charset: Charset::Extended,
    options: Some(r#"{"speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// A quarter turn clockwise at a time: upper left, upper right, lower right, lower left.
const TURNS: [char; 4] = ['◤', '◥', '◢', '◣'];

/// The tiles; only their speed is state.
pub struct GeometricTiles {
    speed: f64,
}

/// Geometric tiles; reads `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(GeometricTiles { speed: options.f64("speed", 1.0) })
}

impl Piece for GeometricTiles {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let t = t * self.speed;
        let (cx, cy) = ((META.cols as f64 - 2.0) / 2.0, (META.rows as f64 - 1.0) / 2.0);
        grid.clear();
        // Tiles in every other column: two cells across is as wide as one is
        // tall, so the tiles sit on a square grid.
        for y in 0..META.rows {
            for x in (0..META.cols).step_by(2) {
                let (dx, dy) = (x as f64 - cx, 2.0 * (y as f64 - cy));
                let d = (dx * dx + dy * dy).sqrt();
                let wave = t * 0.8 - d * 0.085;
                let ch = if d < 1.5 {
                    '●'
                } else if wave.rem_euclid(1.0) > 0.86 {
                    '◆'
                } else {
                    // Each tile points around the centre by its quarter of the
                    // circle, and the ripple turns it a quarter each quarter wave:
                    // ring after ring of rosettes, turning outward.
                    let quarter = ((dy.atan2(dx) / std::f64::consts::FRAC_PI_2).floor() as i64).rem_euclid(4);
                    let turn = (wave * 4.0).floor() as i64;
                    TURNS[(quarter + turn).rem_euclid(4) as usize]
                };
                grid.put(x as i64, y as i64, ch);
            }
        }
    }
}
