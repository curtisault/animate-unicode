//! swift: the Swift logo, a swift on an orange square, with a glint crossing it
//! every few seconds. After ascii.rest's swift.ts (MIT, @bas3line).
//!
//! Drawn from devicon's swift-original.svg (MIT) by scripts/logo.py, into
//! art/swift.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from swift-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The swift logo's meta.
pub static META: Meta = Meta {
    name: "swift",
    slug: "swift",
    category: Category::Logos,
    note: "the swift on an orange square, glinting now and then",
    cols: 58,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 2 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#f05138", "#fefefe", "#f58e7e", "#fefefe", "#fbcbc3", "#ffffff", "#f05138", "#fefefe", "#f58e7e", "#fefefe", "#fbcbc3", "#ffffff"]),
    ..Meta::DEFAULT
};

/// The swift logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/swift.txt"), Loop::Glint, options)
}
