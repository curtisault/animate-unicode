//! rust: the Rust logo, an R in a cog, in Ferris orange, with a glint crossing
//! it every few seconds. After ascii.rest's rust.ts (MIT, @bas3line).
//!
//! Drawn from devicon's rust-original.svg (MIT) by scripts/logo.py, into
//! art/rust.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The rust logo's meta.
pub static META: Meta = Meta {
    name: "rust",
    slug: "rust",
    category: Category::Logos,
    note: "the r in its cog, in ferris orange, glinting now and then",
    cols: 64,
    rows: 32,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&["#f74c00", "#fa8b59", "#fdc9b3", "#f74c00", "#fa8b59", "#fdc9b3"]),
    ..Meta::DEFAULT
};

/// The rust logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/rust.txt"), Loop::Glint, options)
}
