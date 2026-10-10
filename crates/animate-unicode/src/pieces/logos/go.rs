//! go: the Go logo, the word GO with speed lines, with a sheen crossing it
//! every few seconds. After ascii.rest's go.ts (MIT, @bas3line).
//!
//! Drawn from devicon's go-original-wordmark.svg (MIT) by scripts/logo.py, into
//! art/go.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The go logo's meta.
pub static META: Meta = Meta {
    name: "go",
    slug: "go",
    category: Category::Logos,
    note: "go with its speed lines, glinting now and then",
    cols: 79,
    rows: 16,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#00b0db", "#59cce8", "#b3e7f4", "#00b0db", "#59cce8", "#b3e7f4"]),
    ..Meta::DEFAULT
};

/// The go logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/go.txt"), options)
}
