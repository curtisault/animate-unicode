//! zig: the Zig logo, a slanted Z between brackets, with a sheen crossing it
//! every few seconds. After ascii.rest's zig.ts (MIT, @bas3line).
//!
//! Drawn from devicon's zig-original.svg (MIT) by scripts/logo.py, into
//! art/zig.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The zig logo's meta.
pub static META: Meta = Meta {
    name: "zig",
    slug: "zig",
    category: Category::Logos,
    note: "the slanted z in brackets, glinting now and then",
    cols: 61,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#fba81e", "#fcc66d", "#fee5bc", "#fba81e", "#fcc66d", "#fee5bc"]),
    ..Meta::DEFAULT
};

/// The zig logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/zig.txt"), options)
}
