//! arrow field: a flow drawn as arrows (U+2190–2199), one every other column
//! so they sit on a square grid. Three vortices, two turning one way and one
//! the other, wander round the frame in a current that runs left to right;
//! each arrow points the way the water goes, snapped to the nearest of eight.
//! Where the flow is slack the arrow becomes a dot.

use crate::{Category, Charset, Env, Grid, Meta, Options, Piece};
use std::f64::consts::{PI, TAU};

/// The arrow field's meta.
pub static META: Meta = Meta {
    name: "arrow field",
    slug: "arrow-field",
    category: Category::Physics,
    note: "three wandering vortices in a current, every arrow pointing downstream",
    cols: 48,
    rows: 16,
    fps: 15,
    charset: Charset::Extended,
    options: Some(r#"{"speed": 1.0}"#),
    ..Meta::DEFAULT
};

/// East, then counter-clockwise in eighths of a turn, as on screen (y up).
const ARROWS: [char; 8] = ['→', '↗', '↑', '↖', '←', '↙', '↓', '↘'];

/// The field; only its speed is state.
pub struct ArrowField {
    speed: f64,
}

/// An arrow field; reads `speed`.
pub fn make(options: &Options) -> Box<dyn Piece> {
    Box::new(ArrowField { speed: options.f64("speed", 1.0) })
}

impl Piece for ArrowField {
    fn frame(&mut self, t: f64, _: &Env, grid: &mut Grid) {
        let t = t * self.speed;
        // World units: a cell is 1 wide and 2 tall; y grows upward.
        let (w, h) = (META.cols as f64, 2.0 * META.rows as f64);
        let vortices = [
            (0.5 + 0.30 * (t * 0.21).cos(), 0.5 + 0.28 * (t * 0.17).sin(), 1.0),
            (0.5 + 0.32 * (t * 0.13 + 2.1).cos(), 0.5 + 0.30 * (t * 0.19 + 1.3).sin(), 1.0),
            (0.5 + 0.28 * (t * 0.16 + 4.2).cos(), 0.5 + 0.25 * (t * 0.23 + 3.7).sin(), -1.3),
        ];
        grid.clear();
        for row in 0..META.rows {
            for col in (0..META.cols).step_by(2) {
                let (x, y) = (col as f64 + 0.5, h - (2.0 * row as f64 + 1.0));
                // The current, then each vortex: a swirl that dies off with distance.
                let (mut vx, mut vy) = (0.35, 0.0);
                for &(fx, fy, strength) in &vortices {
                    let (dx, dy) = (x - fx * w, y - fy * h);
                    let r2 = dx * dx + dy * dy + 9.0;
                    vx += strength * -dy * 6.0 / r2;
                    vy += strength * dx * 6.0 / r2;
                }
                let ch = if vx.hypot(vy) < 0.12 {
                    '·'
                } else {
                    let eighth = (vy.atan2(vx).rem_euclid(TAU) / (PI / 4.0)).round() as usize % 8;
                    ARROWS[eighth]
                };
                grid.put(col as i64, row as i64, ch);
            }
        }
    }
}
