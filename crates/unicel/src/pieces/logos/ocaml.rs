//! ocaml: the OCaml logo, a camel on an orange square, with a sheen crossing it
//! every few seconds. After ascii.rest's ocaml.ts (MIT, @bas3line).
//!
//! Drawn from devicon's ocaml-original.svg (MIT) by scripts/logo.py, into
//! art/ocaml.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from ocaml-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The ocaml logo's meta.
pub static META: Meta = Meta {
    name: "ocaml",
    slug: "ocaml",
    category: Category::Logos,
    note: "the camel on an orange square, glinting now and then",
    cols: 64,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's colour for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&["#ee760a", "#f4a660", "#fad6b6", "#ee760a", "#f4a660", "#fad6b6"]),
    ..Meta::DEFAULT
};

/// The ocaml logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/ocaml.txt"), options)
}
