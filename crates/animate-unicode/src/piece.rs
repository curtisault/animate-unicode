//! The piece contract.

use crate::{clock::Wall, Grid};

/// What the piece is told about where it is drawn.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Env {
    /// True when the text is dark on a light ground, so a shaded piece can flip its ramp.
    pub paper: bool,
    /// True when the frame is shown in one ink (a `<pre>`, a terminal without
    /// colour) rather than in the palette, so a piece whose parts differ only by
    /// colour can draw a version that reads without it.
    pub mono: bool,
    /// The wall clock, for a piece with `meta.clock`. `None` where there is
    /// no clock to read (the contract test, a still): such a piece then shows
    /// [`Wall::FIXED`], so its frames stay deterministic.
    pub now: Option<Wall>,
}

/// A piece: something that draws the picture at `t` seconds into a grid.
///
/// `frame` must be deterministic in `t` (a seeded PRNG, never a clock, unless
/// `meta.clock`), must fill exactly `meta.cols × meta.rows` (it is handed a
/// grid of that size and may assume it) and must draw only glyphs its
/// `meta.charset` allows. It may keep state between frames (a z-buffer, a
/// particle list) but must not depend on being called in order: the contract
/// test and the server's first frame call it at arbitrary times.
pub trait Piece {
    /// The picture at `t` seconds of play time, written into `grid`.
    fn frame(&mut self, t: f64, env: &Env, grid: &mut Grid);
}
