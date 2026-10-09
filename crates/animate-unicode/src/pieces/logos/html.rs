//! html: the HTML logo, an orange shield with a 5, with a glint crossing it
//! every few seconds. After ascii.rest's html.ts (MIT, @bas3line).
//!
//! Drawn from devicon's html5-original.svg (MIT) by scripts/logo.py, into
//! art/html.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from html5-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo::{self, Loop};
use crate::{Category, Charset, Meta, Options, Piece};

/// The html logo's meta.
pub static META: Meta = Meta {
    name: "html",
    slug: "html",
    category: Category::Logos,
    note: "the orange shield with a 5, glinting now and then",
    cols: 50,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the glint.
    palette: Some(&[
        "#e44d26", "#ffffff", "#ebebea", "#f1652a", "#ed8b72", "#ffffff", "#f2f2f1", "#f69b75", "#f7cabe", "#ffffff", "#f9f9f9", "#fbd1bf", "#e44d26",
        "#ffffff", "#ebebea", "#f1652a", "#ed8b72", "#ffffff", "#f2f2f1", "#f69b75", "#f7cabe", "#ffffff", "#f9f9f9", "#fbd1bf",
    ]),
    ..Meta::DEFAULT
};

/// The html logo; `shine` is the seconds between glints, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/html.txt"), Loop::Glint, options)
}
