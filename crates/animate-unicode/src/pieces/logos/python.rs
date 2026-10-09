//! python: the Python logo, two snakes, one blue and one yellow, with a glint
//! crossing it every few seconds. After ascii.rest's python.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's python-original.svg (MIT) by scripts/logo.py, into
//! art/python.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from python-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The python logo's meta.
pub static META: Meta = Meta {
    name: "python",
    slug: "python",
    category: Category::Logos,
    note: "two snakes, blue and yellow, glinting now and then",
    cols: 51,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 3 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#417fb0", "#ffe05c", "#336d9d", "#84accc", "#ffeb95", "#7aa0bf", "#c6d9e7", "#fff6ce", "#c2d3e2", "#417fb0", "#ffe05c", "#336d9d", "#84accc",
        "#ffeb95", "#7aa0bf", "#c6d9e7", "#fff6ce", "#c2d3e2",
    ]),
    ..Meta::DEFAULT
};

/// The python logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/python.txt"), Loop::Glint, options)
}
