//! mintlify: the Mintlify logo, a leaf on a stem cut out of a green leaf, with
//! a sheen crossing it every few seconds. After ascii.rest's mintlify.ts (MIT,
//! @bas3line).
//!
//! Drawn from Simple Icons' mintlify.svg (CC0) by scripts/logo.py, into
//! art/mintlify.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colour, and a deeper shade of it on a light page, where it
//! would fade into it; in a `<pre>` it is one ink, from the same drawing with
//! its white left bare. The logo is a trademark of its owner, shown here to
//! name the company.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The mintlify logo's meta.
pub static META: Meta = Meta {
    name: "mintlify",
    slug: "mintlify",
    category: Category::Companies,
    note: "the leaf within a leaf, in mint green, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#12a670", "#65c5a2", "#b8e4d4", "#18e299", "#69ecbd", "#baf6e0"]),
    ..Meta::DEFAULT
};

/// The mintlify logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/mintlify.txt"), options)
}
