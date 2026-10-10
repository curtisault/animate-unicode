//! javascript: the JavaScript logo, a yellow square lettered JS, with a sheen
//! crossing it every few seconds. After ascii.rest's javascript.ts (MIT,
//! @bas3line).
//!
//! Drawn from devicon's javascript-original.svg (MIT) by scripts/logo.py, into
//! art/javascript.txt: each cell holds the octant, or the wedge where an edge
//! cuts it at a slant, that best matches the logo through it. On a canvas it
//! takes the logo's colours, lifted on a dark page where they would sink into
//! it; in a `<pre>` it is one ink, from javascript-plain.svg. The logo is a
//! trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The javascript logo's meta.
pub static META: Meta = Meta {
    name: "javascript",
    slug: "javascript",
    category: Category::Logos,
    note: "js on a yellow square, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#f5df50", "#323330", "#f9ea8d", "#7a7a78", "#fcf5cb", "#c2c2c1", "#f5df50", "#323330", "#f9ea8d", "#7a7a78", "#fcf5cb", "#c2c2c1"]),
    ..Meta::DEFAULT
};

/// The javascript logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/javascript.txt"), options)
}
