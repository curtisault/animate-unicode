//! playstation: the PlayStation logo, a P standing on a flat S, with a glint
//! crossing it every few seconds. After ascii.rest's playstation.ts (MIT,
//! @bas3line).
//!
//! Drawn from Simple Icons' playstation.svg (CC0) by scripts/logo.py, into
//! art/playstation.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colour, lifted on a dark page where it would sink into it;
//! in a `<pre>` it is one ink, from the same drawing with its white left bare.
//! The logo is a trademark of its owner, shown here to name the company.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The playstation logo's meta.
pub static META: Meta = Meta {
    name: "playstation",
    slug: "playstation",
    category: Category::Companies,
    note: "the p standing on a flat s, in blue, glinting now and then",
    cols: 70,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#0070d1", "#59a2e1", "#b3d4f1", "#0070d1", "#59a2e1", "#b3d4f1"]),
    ..Meta::DEFAULT
};

/// The playstation logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/playstation.txt"), Loop::Glint, options)
}
