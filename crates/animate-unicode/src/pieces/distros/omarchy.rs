//! omarchy: the Omarchy logo, a square o built from interlocking brackets, with
//! a sheen crossing it every few seconds. After ascii.rest's omarchy.ts (MIT,
//! @bas3line).
//!
//! Drawn from Simple Icons' omarchy.svg (MIT, from omarchy.org) by
//! scripts/logo.py, into art/omarchy.txt: each cell holds the octant, or the
//! wedge where an edge cuts it at a slant, that best matches the logo through
//! it. On a canvas it takes the logo's colour, and a deeper shade of it on a
//! light page, where it would fade into it; in a `<pre>` it is one ink, from
//! the same drawing with its white left bare. The logo is a trademark of its
//! owner, shown here to name the distribution.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The omarchy logo's meta.
pub static META: Meta = Meta {
    name: "omarchy",
    slug: "omarchy",
    category: Category::Distros,
    note: "the square o of brackets, in green, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#6ea235", "#a1c37c", "#d4e3c2", "#9ece6a", "#c0df9e", "#e2f0d2"]),
    ..Meta::DEFAULT
};

/// The omarchy logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/omarchy.txt"), options)
}
