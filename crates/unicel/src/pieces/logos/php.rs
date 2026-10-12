//! php: the PHP logo, php in an oval, with a sheen crossing it every few
//! seconds. After ascii.rest's php.ts (MIT, @bas3line).
//!
//! Drawn from devicon's php-original.svg (MIT) by scripts/logo.py, into
//! art/php.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from php-plain.svg. The logo is a trademark of its
//! owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The php logo's meta.
pub static META: Meta = Meta {
    name: "php",
    slug: "php",
    category: Category::Logos,
    note: "php in its purple oval, glinting now and then",
    cols: 69,
    rows: 19,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 6 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#abafd0", "#000000", "#494d88", "#fefefe", "#777bb3", "#65699d", "#c8cbe0", "#595959", "#898bb2", "#fefefe", "#a7a9ce", "#9b9ebf", "#e6e7f1",
        "#b3b3b3", "#c8cadb", "#ffffff", "#d6d7e8", "#d1d2e2", "#abafd0", "#000000", "#54589c", "#fefefe", "#777bb3", "#65699d", "#c8cbe0", "#595959",
        "#9092bf", "#fefefe", "#a7a9ce", "#9b9ebf", "#e6e7f1", "#b3b3b3", "#cccde1", "#ffffff", "#d6d7e8", "#d1d2e2",
    ]),
    ..Meta::DEFAULT
};

/// The php logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/php.txt"), options)
}
