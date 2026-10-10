//! elixir: the Elixir logo, a purple drop, with a sheen crossing it every few
//! seconds. After ascii.rest's elixir.ts (MIT, @bas3line).
//!
//! Drawn from devicon's elixir-original.svg (MIT) by scripts/logo.py, into
//! art/elixir.txt: each cell holds the octant, or the wedge where an edge cuts
//! it at a slant, that best matches the logo through it. On a canvas it takes
//! the logo's colours, lifted on a dark page where they would sink into it; in
//! a `<pre>` it is one ink, from the same drawing with its white left bare. The
//! logo is a trademark of its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The elixir logo's meta.
pub static META: Meta = Meta {
    name: "elixir",
    slug: "elixir",
    category: Category::Logos,
    note: "the purple drop, glinting now and then",
    cols: 38,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 7 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#9c88ad", "#f4f3f4", "#664a78", "#cec5d7", "#816a92", "#46265c", "#afa2b9", "#bfb2ca", "#f8f7f8", "#9c89a7", "#dfd9e5", "#ad9eb8", "#877295",
        "#cbc3d2", "#e1dbe6", "#fcfbfc", "#d1c9d7", "#f0eef3", "#d9d2de", "#c8bece", "#e7e3ea", "#9c88ad", "#f4f3f4", "#715285", "#cec5d7", "#816a92",
        "#7d44a4", "#afa2b9", "#bfb2ca", "#f8f7f8", "#a38fb0", "#dfd9e5", "#ad9eb8", "#ab85c4", "#cbc3d2", "#e1dbe6", "#fcfbfc", "#d4cbda", "#f0eef3",
        "#d9d2de", "#d8c7e4", "#e7e3ea",
    ]),
    ..Meta::DEFAULT
};

/// The elixir logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/elixir.txt"), options)
}
