//! erlang: the Erlang logo, an e between brackets, with a sheen crossing it
//! every few seconds. After ascii.rest's erlang.ts (MIT, @bas3line).
//!
//! Drawn from devicon's erlang-original.svg (MIT) by scripts/logo.py, into
//! art/erlang.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from erlang-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The erlang logo's meta.
pub static META: Meta = Meta {
    name: "erlang",
    slug: "erlang",
    category: Category::Logos,
    note: "the red e in brackets, glinting now and then",
    cols: 70,
    rows: 23,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#ad0534", "#ca5d7b", "#e6b4c2", "#bc0538", "#d35d7e", "#ebb4c3"]),
    ..Meta::DEFAULT
};

/// The erlang logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/erlang.txt"), options)
}
