//! c++: the C++ logo, a C and two plus signs on a hexagon, with a sheen
//! crossing it every few seconds. After ascii.rest's cpp.ts (MIT, @bas3line).
//!
//! Drawn from devicon's cplusplus-original.svg (MIT) by scripts/logo.py, into
//! art/cpp.txt: each cell holds the octant, or the wedge where an edge cuts it
//! at a slant, that best matches the logo through it. On a canvas it takes the
//! logo's colours, lifted on a dark page where they would sink into it; in a
//! `<pre>` it is one ink, from cplusplus-plain.svg. The logo is a trademark of
//! its owner, shown here to name the language.

use crate::logo;
use crate::{Category, Charset, Meta, Options, Piece};

/// The c++ logo's meta.
pub static META: Meta = Meta {
    name: "c++",
    slug: "cpp",
    category: Category::Logos,
    note: "c++ on a blue hexagon, glinting now and then",
    cols: 50,
    rows: 28,
    fps: 30,
    charset: Charset::Extended,
    options: Some(r#"{"shine": 5}"#),
    // The logo's 4 colours for a light page, then for a dark one; each as drawn, then twice lighter for the sheen.
    palette: Some(&[
        "#659ad2", "#ffffff", "#004482", "#00599c", "#9bbde2", "#ffffff", "#5985ae", "#5993bf", "#d1e1f2", "#ffffff", "#b3c7da", "#b3cde1", "#659ad2",
        "#ffffff", "#005fb5", "#0062ab", "#9bbde2", "#ffffff", "#5997cf", "#5999c8", "#d1e1f2", "#ffffff", "#b3cfe9", "#b3d0e6",
    ]),
    ..Meta::DEFAULT
};

/// The c++ logo; `shine` is the seconds between sheens, 0 to keep it still.
pub fn make(options: &Options) -> Box<dyn Piece> {
    logo::make(&META, include_str!("art/cpp.txt"), options)
}
