//! planetscale: the PlanetScale logo, a disc with a corner cut away in two
//! strokes, with a glint crossing it every few seconds. After ascii.rest's
//! planetscale.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' planetscale.svg (CC0) by scripts/logo.py, into
//! art/planetscale.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it is
//! black, and a light grey on a dark page, where black would sink into it; in a
//! `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the company.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The planetscale logo's meta.
pub static META: Meta = Meta {
    name: "planetscale",
    slug: "planetscale",
    category: Category::Companies,
    note: "the disc cut by two strokes, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#000000", "#595959", "#b3b3b3", "#e8ebef", "#f0f2f5", "#f8f9fa"]),
    ..Meta::DEFAULT
};

/// The planetscale logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/planetscale.txt"), Loop::Glint, options)
}
