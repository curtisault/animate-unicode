//! ruby: the Ruby logo, a cut red gem, with a glint crossing it every few
//! seconds. After ascii.rest's ruby.ts (MIT, @bas3line).
//!
//! Drawn from devicon's ruby-original.svg (MIT) by scripts/logo.py, into
//! art/ruby.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from ruby-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The ruby logo's meta.
pub static META: Meta = Meta {
    name: "ruby",
    slug: "ruby",
    category: Category::Logos,
    note: "the cut red gem, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 7 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#fefbfb", "#960f05", "#e37a63", "#cc2312", "#f0b7a9", "#da4a32", "#b31405", "#fefcfc", "#bb635d", "#eda99a", "#de7065", "#f5d0c7", "#e7897a",
        "#ce665d", "#fffefe", "#e0b7b4", "#f7d7d0", "#f0bdb8", "#fbe9e5", "#f4c9c2", "#e8b9b4", "#fefbfb", "#bd1306", "#e37a63", "#cc2312", "#f0b7a9",
        "#da4a32", "#bd1505", "#fefcfc", "#d4665d", "#eda99a", "#de7065", "#f5d0c7", "#e7897a", "#d4675d", "#fffefe", "#ebb8b4", "#f7d7d0", "#f0bdb8",
        "#fbe9e5", "#f4c9c2", "#ebb9b4",
    ]),
    ..Meta::DEFAULT
};

/// The ruby logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/ruby.txt"), Loop::Glint, options)
}
