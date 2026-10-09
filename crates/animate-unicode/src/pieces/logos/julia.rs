//! julia: the Julia logo, three dots, green, red and purple, with a glint
//! crossing it every few seconds. After ascii.rest's julia.ts (MIT, @bas3line).
//!
//! Drawn from devicon's julia-original.svg (MIT) by scripts/logo.py, into
//! art/julia.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from julia-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The julia logo's meta.
pub static META: Meta = Meta {
    name: "julia",
    slug: "julia",
    category: Category::Logos,
    note: "three dots, green, red and purple, glinting now and then",
    cols: 60,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#eeeeee", "#cb3c33", "#389826", "#9558b2", "#f4f4f4", "#dd807a", "#7ebc72", "#ba92cd", "#fafafa", "#efc5c2", "#c3e0be", "#dfcde8", "#eeeeee",
        "#cb3c33", "#389826", "#9558b2", "#f4f4f4", "#dd807a", "#7ebc72", "#ba92cd", "#fafafa", "#efc5c2", "#c3e0be", "#dfcde8",
    ]),
    ..Meta::DEFAULT
};

/// The julia logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/julia.txt"), Loop::Glint, options)
}
