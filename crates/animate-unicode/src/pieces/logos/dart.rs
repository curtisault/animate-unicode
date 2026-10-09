//! dart: the Dart logo, the dart, in blue and teal, with a glint crossing it
//! every few seconds. After ascii.rest's dart.ts (MIT, @bas3line).
//!
//! Drawn from devicon's dart-original.svg (MIT) by scripts/logo.py, into
//! art/dart.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from dart-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The dart logo's meta.
pub static META: Meta = Meta {
    name: "dart",
    slug: "dart",
    category: Category::Logos,
    note: "the dart, in blue and teal, glinting now and then",
    cols: 56,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#00c4b3", "#0075c9", "#00a8e1", "#22d3c5", "#59d9ce", "#59a5dc", "#59c6ec", "#6fe2d9", "#b3ede8", "#b3d6ef", "#b3e5f6", "#bdf2ee", "#00c4b3",
        "#0075c9", "#00a8e1", "#22d3c5", "#59d9ce", "#59a5dc", "#59c6ec", "#6fe2d9", "#b3ede8", "#b3d6ef", "#b3e5f6", "#bdf2ee",
    ]),
    ..Meta::DEFAULT
};

/// The dart logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/dart.txt"), Loop::Glint, options)
}
