//! helium: the Helium logo, a six-pointed asterisk, with a sheen crossing it
//! every few seconds. After ascii.rest's helium.ts (MIT, @bas3line).
//!
//! Drawn from Simple Icons' heliumbrowser.svg (CC0) by scripts/logo.py, into
//! art/helium.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colour, lifted on a dark page where it would sink into it; in a
//! `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the company.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The helium logo's meta.
pub static META: Meta = Meta {
    name: "helium",
    slug: "helium",
    category: Category::Companies,
    note: "the six-point asterisk, in blue, glinting now and then",
    cols: 49,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#3450d1", "#7b8de1", "#c2cbf1", "#3854d2", "#7e90e2", "#c3ccf2"]),
    ..Meta::DEFAULT
};

/// The helium logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/helium.txt"), options)
}
